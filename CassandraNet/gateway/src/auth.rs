use anyhow::Result;
use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, sync::RwLock};
use subtle::ConstantTimeEq;
#[cfg(feature = "db")]
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct ApiKeyConfig {
    #[serde(default)]
    scopes: Vec<String>,
}

/// Flag indicating whether CASS_API_KEYS env var is being used (for deprecation warning)
static USING_ENV_API_KEYS: Lazy<bool> = Lazy::new(|| std::env::var("CASS_API_KEYS").is_ok());

static API_KEYS: Lazy<RwLock<HashMap<String, ApiKeyConfig>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    if let Ok(raw) = std::env::var("CASS_API_KEYS") {
        // Log deprecation warning only once at startup
        tracing::warn!(
            target: "security",
            event = "auth.cass_api_keys_deprecated",
            "CASS_API_KEYS env var is deprecated. Migrate API keys to database storage."
        );
        for entry in raw.split(',').filter(|s| !s.trim().is_empty()) {
            let mut parts = entry.splitn(2, ':');
            if let (Some(key), Some(scope_str)) = (parts.next(), parts.next()) {
                let key = key.trim().to_string();
                let scopes: Vec<String> = scope_str
                    .split('|')
                    .filter_map(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    })
                    .collect();
                map.insert(key.to_string(), ApiKeyConfig { scopes });
            }
        }
    }
    RwLock::new(map)
});

#[cfg(feature = "db")]
#[derive(Debug, Clone)]
struct DbApiKeyConfig {
    id: Uuid,
    token_hash: Vec<u8>,
    scopes: Vec<String>,
}

/// Database-backed API key cache keyed by token_prefix.
///
/// Populated at startup (db builds) and kept up to date by API key management endpoints.
#[cfg(feature = "db")]
static DB_API_KEYS: Lazy<RwLock<HashMap<String, DbApiKeyConfig>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[cfg(feature = "db")]
pub fn replace_db_api_key_cache(records: Vec<(Uuid, String, String, Vec<String>)>) {
    // (id, token_prefix, token_hash_hex, scopes)
    let mut map = HashMap::new();
    for (id, prefix, token_hash_hex, scopes) in records {
        match hex::decode(token_hash_hex) {
            Ok(bytes) => {
                map.insert(
                    prefix,
                    DbApiKeyConfig {
                        id,
                        token_hash: bytes,
                        scopes,
                    },
                );
            }
            Err(_) => {
                tracing::warn!(api_key_id = %id, "db api key cache: invalid token_hash hex; skipping");
            }
        }
    }
    let mut guard = DB_API_KEYS.write().expect("db api key map poisoned");
    *guard = map;
}

#[cfg(feature = "db")]
pub fn upsert_db_api_key_cache(
    id: Uuid,
    prefix: String,
    token_hash_hex: String,
    scopes: Vec<String>,
) {
    let Ok(token_hash) = hex::decode(token_hash_hex) else {
        tracing::warn!(api_key_id = %id, "db api key cache: invalid token_hash hex; skipping");
        return;
    };
    let mut guard = DB_API_KEYS.write().expect("db api key map poisoned");
    guard.insert(
        prefix,
        DbApiKeyConfig {
            id,
            token_hash,
            scopes,
        },
    );
}

#[cfg(feature = "db")]
pub fn remove_db_api_key_cache(prefix: &str) {
    let mut guard = DB_API_KEYS.write().expect("db api key map poisoned");
    guard.remove(prefix);
}

#[derive(Debug, Clone)]
pub struct AuthStatus {
    pub allowed: bool,
    pub scopes: Vec<String>,
    #[allow(dead_code)]
    pub subject: Option<String>,
}

/// Validate an API key from headers.
///
/// Validation order:
/// 1. (When DB feature enabled) Check PostgresApiKeyStore first
/// 2. Fall back to CASS_API_KEYS env var (deprecated)
///
/// Note: When DB validation is implemented, add api_key_store to AppState
/// and pass it here, or use a global store reference.
pub fn validate_api_key(headers: &HeaderMap) -> AuthStatus {
    if let Some(val) = headers.get("x-api-key") {
        if let Ok(key) = val.to_str() {
            #[cfg(feature = "db")]
            {
                // DB-backed cache validation (preferred)
                if key.len() >= 12 {
                    let prefix = &key[..12];
                    let guard = DB_API_KEYS.read().expect("db api key map poisoned");
                    if let Some(cfg) = guard.get(prefix) {
                        use sha2::{Digest, Sha256};
                        let mut hasher = Sha256::new();
                        hasher.update(key.as_bytes());
                        let candidate = hasher.finalize();
                        if bool::from(candidate.as_slice().ct_eq(cfg.token_hash.as_slice())) {
                            return AuthStatus {
                                allowed: true,
                                scopes: cfg.scopes.clone(),
                                subject: Some(format!("api_key:{}", cfg.id)),
                            };
                        }
                        tracing::warn!(event = "auth.api_key_hash_mismatch", prefix = %prefix, "Rejecting API key with matching prefix but invalid hash");
                    }
                }
            }

            // Fallback to deprecated env var based validation
            let guard = API_KEYS.read().expect("api key map poisoned");
            if let Some(cfg) = guard.get(key) {
                if *USING_ENV_API_KEYS {
                    tracing::debug!(
                        event = "auth.api_key_env_fallback",
                        "API key validated from CASS_API_KEYS env var (deprecated)"
                    );
                }
                return AuthStatus {
                    allowed: true,
                    scopes: cfg.scopes.clone(),
                    subject: None,
                };
            }
            let key_prefix: String = key.chars().take(8).collect();
            tracing::warn!(event = "auth.api_key_unknown", key_prefix = %key_prefix, "Rejecting unfamiliar API key");
        }
    }
    AuthStatus {
        allowed: false,
        scopes: Vec::new(),
        subject: None,
    }
}

pub fn validate_jwt(headers: &HeaderMap) -> AuthStatus {
    if let Some(value) = headers.get("authorization") {
        if let Ok(s) = value.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if hs256_validate(token).unwrap_or(false) {
                    let payload = decode_payload(token).ok();
                    let scopes = payload
                        .as_ref()
                        .and_then(collect_scopes)
                        .unwrap_or_default();
                    let subject = payload
                        .as_ref()
                        .and_then(|p| p.get("sub"))
                        .and_then(Value::as_str)
                        .map(|s| s.to_string());
                    return AuthStatus {
                        allowed: true,
                        scopes,
                        subject,
                    };
                }
            }
        }
    }
    AuthStatus {
        allowed: false,
        scopes: Vec::new(),
        subject: None,
    }
}

pub fn has_scope(headers: &HeaderMap, required_scope: &str) -> bool {
    let api = validate_api_key(headers);
    if api.allowed && api.scopes.iter().any(|s| scope_matches(required_scope, s)) {
        return true;
    }
    let Some(value) = headers.get("authorization") else {
        return false;
    };
    let Ok(header) = value.to_str() else {
        return false;
    };
    let Some(token) = header.strip_prefix("Bearer ") else {
        return false;
    };
    if !hs256_validate(token).unwrap_or(false) {
        return false;
    }
    match decode_payload(token) {
        Ok(payload) => scopes_allow(&payload, required_scope),
        Err(_) => false,
    }
}

fn scope_matches(required: &str, present: &str) -> bool {
    if present == "*" || present == required {
        return true;
    }

    // Backward compatibility: legacy "admin" scope satisfies admin:read/admin:write only.
    if present == "admin" && (required == "admin:read" || required == "admin:write") {
        return true;
    }

    false
}

/// HS256 signing secret from `CASS_JWT_SECRET`. `None` when unset or empty:
/// there is deliberately no built-in fallback, so a misconfigured deployment
/// rejects every token instead of accepting ones signed with a public default.
pub fn jwt_secret() -> Option<Vec<u8>> {
    std::env::var("CASS_JWT_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .map(String::into_bytes)
}

pub fn hs256_generate(sub: &str) -> Result<String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use hmac::{Hmac, Mac};
    use serde_json::json;
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let Some(secret) = jwt_secret() else {
        anyhow::bail!("CASS_JWT_SECRET not set");
    };
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let exp = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        + 3600) as i64;
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"sub": sub, "exp": exp}))?);
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(&secret)?;
    mac.update(signing_input.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{sig}"))
}

pub fn hs256_validate(token: &str) -> Result<bool> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut parts = token.split('.');
    let (header_b64, payload_b64, sig_b64) = match (parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s)) => (h, p, s),
        _ => return Ok(false),
    };
    if parts.next().is_some() {
        return Ok(false);
    }
    let header_json = URL_SAFE_NO_PAD.decode(header_b64)?;
    if !header_json.windows(5).any(|w| w == b"HS256") {
        return Ok(false);
    }
    let Some(secret) = jwt_secret() else {
        return Ok(false);
    };
    let signing_input = format!("{header_b64}.{payload_b64}");
    let mut mac = HmacSha256::new_from_slice(&secret)?;
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    let provided = URL_SAFE_NO_PAD.decode(sig_b64).unwrap_or_default();
    let signature_ok = provided.len() == sig.len()
        && bool::from(ConstantTimeEq::ct_eq(provided.as_slice(), sig.as_slice()));
    if !signature_ok {
        return Ok(false);
    }
    let Ok(claims) = decode_payload(token) else {
        return Ok(false);
    };
    Ok(claims_usable_for_api(&claims, unix_now()))
}

/// A signed token only authorizes API calls while unexpired, and never when it
/// is a refresh token (those are only redeemable at `/auth/refresh`).
pub(crate) fn claims_usable_for_api(claims: &Value, now: i64) -> bool {
    let unexpired = claims
        .get("exp")
        .and_then(Value::as_i64)
        .is_some_and(|exp| exp > now);
    let is_refresh = claims.get("token_use").and_then(Value::as_str) == Some("refresh");
    unexpired && !is_refresh
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn decode_payload(token: &str) -> Result<Value> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let mut parts = token.split('.');
    let (_, payload_b64) = match (parts.next(), parts.next()) {
        (Some(_header), Some(payload)) => ((), payload),
        _ => return Err(anyhow::anyhow!("invalid token")),
    };
    let bytes = URL_SAFE_NO_PAD.decode(payload_b64)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn scopes_allow(payload: &Value, required: &str) -> bool {
    if let Some(scopes) = payload.get("scopes").and_then(|v| v.as_array()) {
        if scopes
            .iter()
            .flat_map(Value::as_str)
            .any(|s| scope_matches(required, s))
        {
            return true;
        }
    }
    if let Some(scope_str) = payload.get("scope").and_then(Value::as_str) {
        if scope_str
            .split_whitespace()
            .any(|s| scope_matches(required, s))
        {
            return true;
        }
    }
    false
}

fn collect_scopes(payload: &Value) -> Option<Vec<String>> {
    if let Some(scopes) = payload.get("scopes").and_then(|v| v.as_array()) {
        return Some(
            scopes
                .iter()
                .flat_map(Value::as_str)
                .map(|s| s.to_string())
                .collect(),
        );
    }
    if let Some(scope_str) = payload.get("scope").and_then(Value::as_str) {
        return Some(
            scope_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        );
    }
    None
}
