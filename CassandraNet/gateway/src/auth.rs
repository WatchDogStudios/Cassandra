use anyhow::Result;
use axum::http::HeaderMap;
use subtle::ConstantTimeEq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStatus {
    Allow,
    Deny,
}

pub fn validate_api_key(headers: &HeaderMap) -> AuthStatus {
    if let Some(val) = headers.get("x-api-key") {
        if !val.is_empty() {
            return AuthStatus::Allow;
        }
    }
    AuthStatus::Deny
}

pub fn validate_jwt(headers: &HeaderMap) -> AuthStatus {
    if let Some(value) = headers.get("authorization") {
        if let Ok(s) = value.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if hs256_validate(token).unwrap_or(false) {
                    return AuthStatus::Allow;
                }
            }
        }
    }
    AuthStatus::Deny
}

pub fn hs256_generate(sub: &str) -> Result<String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use hmac::{Hmac, Mac};
    use serde_json::json;
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let secret = std::env::var("CASS_JWT_SECRET").unwrap_or_default();
    if secret.is_empty() {
        anyhow::bail!("CASS_JWT_SECRET not set");
    }
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let exp = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        + 3600) as i64;
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json!({"sub": sub, "exp": exp}))?);
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
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
    let secret = std::env::var("CASS_JWT_SECRET").unwrap_or_default();
    if secret.is_empty() {
        return Ok(false);
    }
    let signing_input = format!("{header_b64}.{payload_b64}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    let provided = URL_SAFE_NO_PAD.decode(sig_b64).unwrap_or_default();
    Ok(provided.len() == sig.len()
        && ConstantTimeEq::ct_eq(provided.as_slice(), sig.as_slice()).into())
}
