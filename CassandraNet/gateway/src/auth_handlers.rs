use crate::state::audit::AuditEvent;
use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use cncore::platform::{PlatformError, User, UserId};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::net::SocketAddr;
use std::time::Duration as StdDuration;
use utoipa::ToSchema;

type HmacSha256 = Hmac<Sha256>;

const ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
const REFRESH_TOKEN_TTL_DAYS: i64 = 7;
const REFRESH_COOKIE_NAME: &str = "cass_refresh";
const LOGIN_RATE_LIMIT: u32 = 5;
const LOGIN_RATE_WINDOW_SECS: u64 = 60;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    email: String,
    scopes: Vec<String>,
    #[serde(rename = "use")]
    token_use: String,
    iat: i64,
    exp: i64,
}

pub struct AuthError {
    status: StatusCode,
    message: &'static str,
}

impl AuthError {
    fn new(status: StatusCode, message: &'static str) -> Self {
        Self { status, message }
    }

    fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "invalid credentials")
    }

    fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

impl From<PlatformError> for AuthError {
    fn from(err: PlatformError) -> Self {
        match err {
            PlatformError::NotFound(_) => AuthError::new(StatusCode::NOT_FOUND, "not found"),
            PlatformError::Unauthorized => AuthError::unauthorized(),
            PlatformError::Forbidden => AuthError::new(StatusCode::FORBIDDEN, "forbidden"),
            _ => AuthError::internal(),
        }
    }
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResponse>), AuthError> {
    let client_ip = addr.ip().to_string();
    let email = payload.email.trim().to_lowercase();
    tracing::debug!(%email, %client_ip, "login attempt");

    // Rate limit login attempts by IP
    if !state.rate_limiter.check_ip(
        &client_ip,
        "auth:login",
        LOGIN_RATE_LIMIT,
        StdDuration::from_secs(LOGIN_RATE_WINDOW_SECS),
    ) {
        state.audit.emit(
            AuditEvent::warn(
                "auth.login.rate_limited",
                serde_json::json!({
                    "email": email,
                    "reason": "too_many_attempts"
                }),
            )
            .with_ip(&client_ip),
        );
        return Err(AuthError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too many login attempts",
        ));
    }

    if email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::new(
            StatusCode::BAD_REQUEST,
            "email and password required",
        ));
    }

    let user = match state.user_store.get_user_by_email(&email).await {
        Ok(Some(u)) => {
            tracing::debug!(user_id = %u.id, "found user");
            u
        }
        Ok(None) => {
            tracing::debug!("user not found for email");
            state.audit.emit(
                AuditEvent::warn(
                    "auth.login.failed",
                    serde_json::json!({
                        "email": email,
                        "reason": "user_not_found"
                    }),
                )
                .with_ip(&client_ip),
            );
            return Err(AuthError::unauthorized());
        }
        Err(e) => {
            tracing::error!(error = %e, "db error");
            return Err(AuthError::internal());
        }
    };

    if !user.is_active {
        state.audit.emit(
            AuditEvent::warn(
                "auth.login.failed",
                serde_json::json!({
                    "email": email,
                    "user_id": user.id.to_string(),
                    "reason": "account_disabled"
                }),
            )
            .with_actor(user.id.to_string(), "user")
            .with_ip(&client_ip),
        );
        return Err(AuthError::new(StatusCode::FORBIDDEN, "account disabled"));
    }

    // Verify password with argon2
    if !verify_password(&payload.password, &user.password_hash) {
        tracing::debug!("password mismatch");
        state.audit.emit(
            AuditEvent::warn(
                "auth.login.failed",
                serde_json::json!({
                    "email": email,
                    "user_id": user.id.to_string(),
                    "reason": "invalid_password"
                }),
            )
            .with_actor(user.id.to_string(), "user")
            .with_ip(&client_ip),
        );
        return Err(AuthError::unauthorized());
    }

    // Update last login
    let mut updated_user = user.clone();
    updated_user.last_login_at = Some(Utc::now());
    let _ = state.user_store.update_user(updated_user).await;

    // Generate tokens
    let access_token = generate_access_token(&user, &state.jwt_secret)?;
    let refresh_token = generate_refresh_token(&user, &state.jwt_secret)?;

    // Set refresh token as httpOnly cookie
    let mut headers = HeaderMap::new();
    let cookie = build_refresh_cookie(&refresh_token, false);
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    // Emit success audit event
    state.audit.emit(
        AuditEvent::info(
            "auth.login.success",
            serde_json::json!({
                "email": email,
                "user_id": user.id.to_string(),
            }),
        )
        .with_actor(user.id.to_string(), "user")
        .with_ip(&client_ip),
    );

    let response = LoginResponse {
        access_token,
        expires_in: ACCESS_TOKEN_TTL_MINUTES * 60,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            display_name: user.display_name,
            tenant_id: user.tenant_id.map(|t| t.to_string()),
            scopes: user.scopes.iter().map(|s| s.as_str().to_string()).collect(),
        },
    };

    Ok((StatusCode::OK, headers, Json(response)))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<RefreshResponse>), AuthError> {
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(AuthError::unauthorized)?;

    let refresh_token =
        extract_cookie(cookie_header, REFRESH_COOKIE_NAME).ok_or_else(AuthError::unauthorized)?;

    let claims = verify_token(refresh_token, &state.jwt_secret)?;
    if claims.token_use != "refresh" {
        return Err(AuthError::unauthorized());
    }

    let user_id: UserId = claims.sub.parse().map_err(|_| AuthError::unauthorized())?;
    let user = state
        .user_store
        .get_user_by_id(user_id)
        .await
        .map_err(|_| AuthError::internal())?
        .ok_or_else(AuthError::unauthorized)?;

    if !user.is_active {
        return Err(AuthError::new(StatusCode::FORBIDDEN, "account disabled"));
    }

    let access_token = generate_access_token(&user, &state.jwt_secret)?;
    let new_refresh_token = generate_refresh_token(&user, &state.jwt_secret)?;

    let mut response_headers = HeaderMap::new();
    let cookie = build_refresh_cookie(&new_refresh_token, false);
    response_headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        response_headers,
        Json(RefreshResponse {
            access_token,
            expires_in: ACCESS_TOKEN_TTL_MINUTES * 60,
        }),
    ))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AuthError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(AuthError::unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(AuthError::unauthorized)?;

    let claims = verify_token(token, &state.jwt_secret)?;
    if claims.token_use != "access" {
        return Err(AuthError::unauthorized());
    }

    let user_id: UserId = claims.sub.parse().map_err(|_| AuthError::unauthorized())?;
    let user = state
        .user_store
        .get_user_by_id(user_id)
        .await
        .map_err(|_| AuthError::internal())?
        .ok_or_else(AuthError::unauthorized)?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        tenant_id: user.tenant_id.map(|t| t.to_string()),
        scopes: user.scopes.iter().map(|s| s.as_str().to_string()).collect(),
    }))
}

pub async fn logout() -> (HeaderMap, StatusCode) {
    let mut headers = HeaderMap::new();
    let cookie = build_refresh_cookie("", true);
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    (headers, StatusCode::NO_CONTENT)
}

fn generate_access_token(user: &User, secret: &[u8]) -> Result<String, AuthError> {
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        scopes: user.scopes.iter().map(|s| s.as_str().to_string()).collect(),
        token_use: "access".to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::minutes(ACCESS_TOKEN_TTL_MINUTES)).timestamp(),
    };
    sign_jwt(&claims, secret)
}

fn generate_refresh_token(user: &User, secret: &[u8]) -> Result<String, AuthError> {
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        scopes: vec![],
        token_use: "refresh".to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::days(REFRESH_TOKEN_TTL_DAYS)).timestamp(),
    };
    sign_jwt(&claims, secret)
}

fn sign_jwt(claims: &TokenClaims, secret: &[u8]) -> Result<String, AuthError> {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload =
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).map_err(|_| AuthError::internal())?);
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| AuthError::internal())?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{signature}"))
}

fn verify_token(token: &str, secret: &[u8]) -> Result<TokenClaims, AuthError> {
    let mut parts = token.split('.');
    let (header, payload, signature) = match (parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s)) => (h, p, s),
        _ => return Err(AuthError::unauthorized()),
    };
    if parts.next().is_some() {
        return Err(AuthError::unauthorized());
    }

    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| AuthError::internal())?;
    mac.update(signing_input.as_bytes());
    let expected = mac.finalize().into_bytes();
    let provided = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| AuthError::unauthorized())?;

    if provided.len() != expected.len() {
        return Err(AuthError::unauthorized());
    }

    use subtle::ConstantTimeEq;
    if !bool::from(provided.as_slice().ct_eq(expected.as_slice())) {
        return Err(AuthError::unauthorized());
    }

    let claims_bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AuthError::unauthorized())?;
    let claims: TokenClaims =
        serde_json::from_slice(&claims_bytes).map_err(|_| AuthError::unauthorized())?;

    if claims.exp < Utc::now().timestamp() {
        return Err(AuthError::unauthorized());
    }

    Ok(claims)
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AuthError::internal())
}

fn build_refresh_cookie(token: &str, expired: bool) -> String {
    let max_age = if expired {
        0
    } else {
        REFRESH_TOKEN_TTL_DAYS * 24 * 60 * 60
    };
    // Secure by default; set CASS_COOKIE_SECURE=false only for plain-HTTP
    // dev setups that are not on localhost (browsers treat localhost as secure).
    let secure = std::env::var("CASS_COOKIE_SECURE")
        .map(|v| !(v == "false" || v == "0"))
        .unwrap_or(true);
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; HttpOnly; SameSite=Strict; Path=/api/auth/refresh; Max-Age={}{}",
        REFRESH_COOKIE_NAME, token, max_age, secure_flag
    )
}

fn extract_cookie<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(name) {
            if let Some(value) = value.strip_prefix('=') {
                return Some(value);
            }
        }
    }
    None
}
