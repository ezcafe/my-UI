use super::crypto::{decrypt_json, encrypt_json, CryptoError};
use serde::{Deserialize, Serialize};

pub const OIDC_TEMP_COOKIE_NAME: &str = "__oidc_temp";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OidcTempData {
    pub pkce_verifier: String,
    pub csrf_state: String,
}

pub fn create_temp_cookie_value(data: &OidcTempData, secret: &str) -> Result<String, CryptoError> {
    encrypt_json(data, secret)
}

pub fn extract_temp_cookie_value(
    encrypted_val: &str,
    secret: &str,
) -> Result<OidcTempData, CryptoError> {
    decrypt_json(encrypted_val, secret)
}

pub fn build_temp_set_cookie_header(encrypted_val: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; Path=/; Max-Age=300; HttpOnly; SameSite=Lax{}",
        OIDC_TEMP_COOKIE_NAME, encrypted_val, secure_flag
    )
}

pub fn build_clear_temp_cookie_header(secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        OIDC_TEMP_COOKIE_NAME, secure_flag
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_temp_cookie_encryption_and_roundtrip() {
        let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
        let data = OidcTempData {
            pkce_verifier: "pkce-verifier-test-string-123456789".to_string(),
            csrf_state: "csrf-state-abc-xyz".to_string(),
        };

        let encrypted = create_temp_cookie_value(&data, secret).expect("Encryption failed");
        assert_ne!(encrypted, data.pkce_verifier);
        assert!(!encrypted.contains("csrf-state"));

        let decrypted = extract_temp_cookie_value(&encrypted, secret).expect("Decryption failed");
        assert_eq!(decrypted, data);

        let clear_hdr = build_clear_temp_cookie_header(false);
        assert!(clear_hdr.contains("Max-Age=0"));
        assert!(clear_hdr.contains("Path=/"));
    }
}
