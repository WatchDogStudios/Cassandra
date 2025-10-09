use super::error::{PlatformError, PlatformResult};
use super::models::*;
use super::persistence::{ApiKeyStore, TenantStore};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use cncommon::auth::Scope;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct AuthService {
    tenants: Arc<dyn TenantStore>,
    api_keys: Arc<dyn ApiKeyStore>,
    secret: Arc<Vec<u8>>,
    default_ttl: Duration,
    default_refresh_ttl: Duration,
    issuer: String,
    default_audience: Option<String>,
}

impl AuthService {
    pub fn new(
        tenants: Arc<dyn TenantStore>,
        api_keys: Arc<dyn ApiKeyStore>,
        secret: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            tenants,
            api_keys,
            secret: Arc::new(secret.into()),
            default_ttl: Duration::minutes(60),
            default_refresh_ttl: Duration::hours(12),
            issuer: "cassantranet".to_string(),
            default_audience: None,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    pub fn with_refresh_ttl(mut self, ttl: Duration) -> Self {
        self.default_refresh_ttl = ttl;
        self
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    pub fn with_default_audience(mut self, audience: impl Into<String>) -> Self {
        self.default_audience = Some(audience.into());
        self
    }

    pub fn issue_api_key(
        &self,
        tenant_id: TenantId,
        label: impl Into<String>,
        scopes: Vec<Scope>,
    ) -> PlatformResult<ApiKey> {
        self.validate_scopes(&scopes)?;
        self.create_api_key(tenant_id, label.into(), scopes, None)
    }

    pub fn rotate_api_key(&self, id: ApiKeyId) -> PlatformResult<ApiKey> {
        let mut existing = self
            .api_keys
            .get_api_key(id)?
            .ok_or(PlatformError::NotFound("api_key"))?;
        if existing.revoked || existing.deleted_at.is_some() {
            return Err(PlatformError::InvalidInput("api key inactive"));
        }
        let new_key = self.create_api_key(
            existing.tenant_id,
            existing.label.clone(),
            existing.scopes.clone(),
            Some(existing.id),
        )?;
        existing.revoked = true;
        existing.deleted_at = Some(Utc::now());
        existing.rotated_to = Some(new_key.id);
        self.api_keys.update_api_key(existing)?;
        Ok(new_key)
    }

    pub fn soft_delete_api_key(&self, id: ApiKeyId) -> PlatformResult<()> {
        if let Some(mut record) = self.api_keys.get_api_key(id)? {
            record.deleted_at = Some(Utc::now());
            record.revoked = true;
            self.api_keys.update_api_key(record)
        } else {
            Err(PlatformError::NotFound("api_key"))
        }
    }

    pub fn revoke_api_key(&self, id: ApiKeyId) -> PlatformResult<()> {
        if let Some(mut record) = self.api_keys.get_api_key(id)? {
            record.revoked = true;
            self.api_keys.update_api_key(record)
        } else {
            Err(PlatformError::NotFound("api_key"))
        }
    }

    pub fn authenticate_api_key(&self, token: &str) -> PlatformResult<AuthContext> {
        let (prefix, secret) = parse_api_key(token)?;
        let mut record = self
            .api_keys
            .get_api_key_by_prefix(prefix)?
            .ok_or(PlatformError::Unauthorized)?;
        if record.revoked || record.deleted_at.is_some() {
            return Err(PlatformError::Forbidden);
        }
        if record.token_hash != hash_secret(secret) {
            return Err(PlatformError::Unauthorized);
        }
        let issued_at = Utc::now();
        let ttl = self.resolve_access_ttl(record.tenant_id, None)?;
        record.last_used_at = Some(issued_at);
        self.api_keys.update_api_key(record.clone())?;
        Ok(AuthContext {
            principal_id: record.id,
            principal_type: PrincipalType::ServiceAccount,
            tenant_id: record.tenant_id,
            scopes: record.scopes.clone(),
            issued_at,
            expires_at: issued_at + ttl,
            audience: self.default_audience.clone(),
            issuer: Some(self.issuer.clone()),
            session: None,
        })
    }

    pub fn issue_token_from_context(
        &self,
        mut context: AuthContext,
        ttl: Option<Duration>,
    ) -> PlatformResult<AuthToken> {
        let access_ttl = self.resolve_access_ttl(context.tenant_id, ttl)?;
        context.issued_at = Utc::now();
        context.expires_at = context.issued_at + access_ttl;
        if context.audience.is_none() {
            context.audience = self.default_audience.clone();
        }
        context.issuer = Some(self.issuer.clone());
        let nonce = Uuid::new_v4().to_string();
        let claims = TokenClaims::from_context(&context, TokenUse::Access, nonce);
        let token = sign_jwt(&claims, &self.secret)?;
        let refresh_token = self.issue_refresh_token(&context)?;
        Ok(AuthToken {
            token,
            context,
            refresh_token,
        })
    }

    pub fn issue_token_for_api_key(
        &self,
        token: &str,
        ttl: Option<Duration>,
    ) -> PlatformResult<AuthToken> {
        let ctx = self.authenticate_api_key(token)?;
        self.issue_token_from_context(ctx, ttl)
    }

    pub fn refresh_access_token(&self, refresh_token: &str) -> PlatformResult<AuthToken> {
        let claims = verify_jwt(refresh_token, &self.secret)?;
        self.ensure_claims_valid(&claims, TokenUse::Refresh)?;
        let context = AuthContext::from(claims);
        self.issue_token_from_context(context, None)
    }

    pub fn validate_token(&self, token: &str) -> PlatformResult<AuthContext> {
        let claims = verify_jwt(token, &self.secret)?;
        self.ensure_claims_valid(&claims, TokenUse::Access)?;
        Ok(AuthContext::from(claims))
    }

    pub fn list_keys(&self, tenant_id: TenantId) -> PlatformResult<Vec<ApiKeyRecord>> {
        self.api_keys.list_api_keys(tenant_id)
    }

    fn create_api_key(
        &self,
        tenant_id: TenantId,
        label: String,
        scopes: Vec<Scope>,
        rotation_parent: Option<ApiKeyId>,
    ) -> PlatformResult<ApiKey> {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let secret_b64 = URL_SAFE_NO_PAD.encode(secret_bytes);
        let id = Uuid::new_v4();
        let token_prefix = format!("{}", &id.to_string()[..8]);
        let token_hash = hash_secret(&secret_b64);
        let now = Utc::now();
        let record = ApiKeyRecord {
            id,
            tenant_id,
            label: label.clone(),
            scopes: scopes.clone(),
            token_prefix: token_prefix.clone(),
            token_hash,
            created_at: now,
            last_used_at: None,
            revoked: false,
            deleted_at: None,
            rotated_from: rotation_parent,
            rotated_to: None,
        };
        self.api_keys.insert_api_key(record)?;
        let value = format!("{token_prefix}.{secret_b64}");
        Ok(ApiKey {
            id,
            value,
            tenant_id,
            label,
            scopes,
            created_at: now,
            rotation_parent,
        })
    }

    fn validate_scopes(&self, scopes: &[Scope]) -> PlatformResult<()> {
        if scopes.is_empty() {
            return Err(PlatformError::InvalidInput("scopes required"));
        }
        let unique: HashSet<_> = scopes.iter().cloned().collect();
        if unique.len() != scopes.len() {
            return Err(PlatformError::InvalidInput("duplicate scopes"));
        }
        Ok(())
    }

    fn resolve_access_ttl(
        &self,
        tenant_id: TenantId,
        override_ttl: Option<Duration>,
    ) -> PlatformResult<Duration> {
        if let Some(ttl) = override_ttl {
            return Ok(ttl);
        }
        if let Some(settings) = self.tenant_settings(tenant_id)? {
            if let Some(seconds) = settings.token_ttl_seconds {
                if seconds > 0 {
                    return Ok(Duration::seconds(seconds));
                }
            }
        }
        Ok(self.default_ttl)
    }

    fn resolve_refresh_ttl(&self, tenant_id: TenantId) -> PlatformResult<Duration> {
        if let Some(settings) = self.tenant_settings(tenant_id)? {
            if let Some(seconds) = settings.refresh_token_ttl_seconds {
                if seconds > 0 {
                    return Ok(Duration::seconds(seconds));
                } else {
                    return Ok(Duration::zero());
                }
            }
        }
        Ok(self.default_refresh_ttl)
    }

    fn tenant_settings(&self, tenant_id: TenantId) -> PlatformResult<Option<TenantSettings>> {
        Ok(self
            .tenants
            .get_tenant(tenant_id)?
            .map(|tenant| tenant.settings))
    }

    fn issue_refresh_token(&self, context: &AuthContext) -> PlatformResult<Option<String>> {
        let refresh_ttl = self.resolve_refresh_ttl(context.tenant_id)?;
        if refresh_ttl <= Duration::zero() {
            return Ok(None);
        }
        let mut refresh_context = context.clone();
        refresh_context.expires_at = refresh_context.issued_at + refresh_ttl;
        let nonce = Uuid::new_v4().to_string();
        let refresh_claims = TokenClaims::from_context(&refresh_context, TokenUse::Refresh, nonce);
        let token = sign_jwt(&refresh_claims, &self.secret)?;
        Ok(Some(token))
    }

    fn ensure_claims_valid(
        &self,
        claims: &TokenClaims,
        expected_use: TokenUse,
    ) -> PlatformResult<()> {
        if claims.token_use != expected_use {
            return Err(PlatformError::Unauthorized);
        }
        if claims.exp < Utc::now() {
            return Err(PlatformError::Unauthorized);
        }
        if claims.iss != self.issuer {
            return Err(PlatformError::Unauthorized);
        }
        if let Some(expected) = &self.default_audience {
            if claims.aud.as_ref() != Some(expected) {
                return Err(PlatformError::Unauthorized);
            }
        }
        Ok(())
    }
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

fn parse_api_key(token: &str) -> PlatformResult<(&str, &str)> {
    let mut parts = token.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(prefix), Some(secret), None) if prefix.len() >= 4 => Ok((prefix, secret)),
        _ => Err(PlatformError::InvalidInput("malformed api key")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    tenant_id: String,
    scopes: Vec<String>,
    prn_type: String,
    aud: Option<String>,
    iss: String,
    #[serde(rename = "use")]
    token_use: TokenUse,
    nonce: String,
    session: Option<AuthSessionMetadata>,
    #[serde(with = "chrono::serde::ts_seconds")]
    iat: chrono::DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    exp: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum TokenUse {
    Access,
    Refresh,
}

impl TokenClaims {
    fn from_context(ctx: &AuthContext, token_use: TokenUse, nonce: String) -> Self {
        TokenClaims {
            sub: ctx.principal_id.to_string(),
            tenant_id: ctx.tenant_id.to_string(),
            scopes: ctx
                .scopes
                .iter()
                .map(|scope| scope.as_str().to_string())
                .collect(),
            prn_type: format!("{:?}", ctx.principal_type),
            aud: ctx.audience.clone(),
            iss: ctx
                .issuer
                .clone()
                .unwrap_or_else(|| "cassantranet".to_string()),
            token_use,
            nonce,
            session: ctx.session.clone(),
            iat: ctx.issued_at,
            exp: ctx.expires_at,
        }
    }
}

impl From<TokenClaims> for AuthContext {
    fn from(claims: TokenClaims) -> Self {
        AuthContext {
            principal_id: Uuid::parse_str(&claims.sub).unwrap_or_default(),
            principal_type: match claims.prn_type.as_str() {
                "Tenant" => PrincipalType::Tenant,
                "Agent" => PrincipalType::Agent,
                "ServiceAccount" => PrincipalType::ServiceAccount,
                _ => PrincipalType::Service,
            },
            tenant_id: Uuid::parse_str(&claims.tenant_id).unwrap_or_default(),
            scopes: claims
                .scopes
                .iter()
                .map(|scope| Scope::from(scope.as_str()))
                .collect(),
            issued_at: claims.iat,
            expires_at: claims.exp,
            audience: claims.aud,
            issuer: Some(claims.iss),
            session: claims.session,
        }
    }
}

fn sign_jwt(claims: &TokenClaims, secret: &[u8]) -> PlatformResult<String> {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(claims).map_err(|_| PlatformError::Internal("serialize claims"))?,
    );
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| PlatformError::Internal("invalid secret"))?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{signature}"))
}

fn verify_jwt(token: &str, secret: &[u8]) -> PlatformResult<TokenClaims> {
    let mut parts = token.split('.');
    let (header, payload, signature) = match (parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s)) => (h, p, s),
        _ => return Err(PlatformError::Unauthorized),
    };
    if parts.next().is_some() {
        return Err(PlatformError::Unauthorized);
    }
    let header_bytes = URL_SAFE_NO_PAD
        .decode(header)
        .map_err(|_| PlatformError::Unauthorized)?;
    if !header_bytes.windows(5).any(|w| w == b"HS256") {
        return Err(PlatformError::Unauthorized);
    }
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| PlatformError::Internal("invalid secret"))?;
    mac.update(signing_input.as_bytes());
    let expected = mac.finalize().into_bytes();
    let provided = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| PlatformError::Unauthorized)?;
    if provided.len() != expected.len()
        || !bool::from(provided.as_slice().ct_eq(expected.as_slice()))
    {
        return Err(PlatformError::Unauthorized);
    }
    let claims_bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| PlatformError::Unauthorized)?;
    let claims: TokenClaims =
        serde_json::from_slice(&claims_bytes).map_err(|_| PlatformError::Unauthorized)?;
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::persistence::{ApiKeyStore, InMemoryPersistence, TenantStore};

    #[test]
    fn api_key_issue_and_authenticate() {
        let storage = Arc::new(InMemoryPersistence::new());
        let tenant_store: Arc<dyn TenantStore> = storage.clone();
        let api_store: Arc<dyn ApiKeyStore> = storage.clone();
        let secret = b"secret".to_vec();
        let service = AuthService::new(tenant_store.clone(), api_store, secret);
        let tenant_id = Uuid::new_v4();
        tenant_store
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Test".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();
        let key = service
            .issue_api_key(tenant_id, "default", vec![Scope::TenantRead])
            .unwrap();
        let ctx = service
            .authenticate_api_key(&key.value)
            .expect("should authenticate");
        assert_eq!(ctx.tenant_id, tenant_id);
        assert_eq!(ctx.scopes, vec![Scope::TenantRead]);
        assert_eq!(ctx.issuer.as_deref(), Some("cassantranet"));
    }

    #[test]
    fn token_cycle() {
        let storage = Arc::new(InMemoryPersistence::new());
        let tenant_store: Arc<dyn TenantStore> = storage.clone();
        let api_store: Arc<dyn ApiKeyStore> = storage.clone();
        let secret = b"another-secret".to_vec();
        let service = AuthService::new(tenant_store.clone(), api_store, secret)
            .with_default_audience("cncore");
        let tenant_id = Uuid::new_v4();
        tenant_store
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Demo".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();
        let context = AuthContext {
            principal_id: Uuid::new_v4(),
            principal_type: PrincipalType::Tenant,
            tenant_id,
            scopes: vec![Scope::Admin],
            issued_at: Utc::now(),
            expires_at: Utc::now(),
            audience: Some("cncore".into()),
            issuer: Some("cassantranet".into()),
            session: Some(AuthSessionMetadata::default()),
        };
        let token = service
            .issue_token_from_context(context.clone(), Some(Duration::minutes(5)))
            .unwrap();
        let validated = service.validate_token(&token.token).unwrap();
        assert_eq!(validated.principal_id, context.principal_id);
        assert_eq!(validated.scopes, context.scopes);
        assert!(token.refresh_token.is_some());
        let refreshed = service
            .refresh_access_token(token.refresh_token.as_ref().unwrap())
            .unwrap();
        let refreshed_ctx = service.validate_token(&refreshed.token).unwrap();
        assert_eq!(refreshed_ctx.tenant_id, tenant_id);
    }

    #[test]
    fn api_key_rotation_and_soft_delete() {
        let storage = Arc::new(InMemoryPersistence::new());
        let tenant_store: Arc<dyn TenantStore> = storage.clone();
        let api_store: Arc<dyn ApiKeyStore> = storage.clone();
        let secret = b"rotate-secret".to_vec();
        let service = AuthService::new(tenant_store.clone(), api_store.clone(), secret);
        let tenant_id = Uuid::new_v4();
        tenant_store
            .insert_tenant(Tenant {
                id: tenant_id,
                name: "Rotate".into(),
                created_at: Utc::now(),
                settings: TenantSettings::default(),
            })
            .unwrap();
        let key = service
            .issue_api_key(tenant_id, "primary", vec![Scope::ApiKeyManage])
            .unwrap();
        let rotated = service.rotate_api_key(key.id).unwrap();
        assert_eq!(rotated.rotation_parent, Some(key.id));
        let original = api_store.get_api_key(key.id).unwrap().unwrap();
        assert!(original.revoked);
        assert_eq!(original.rotated_to, Some(rotated.id));
        service.soft_delete_api_key(rotated.id).unwrap();
        let rotated_record = api_store.get_api_key(rotated.id).unwrap().unwrap();
        assert!(rotated_record.deleted_at.is_some());
    }
}
