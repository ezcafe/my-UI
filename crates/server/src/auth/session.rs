use super::crypto::{decrypt_json, encrypt_json, CryptoError};
use serde::{Deserialize, Serialize};

pub const SESSION_COOKIE_NAME: &str = "__session";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

pub fn create_session_cookie_value(
    session: &SessionData,
    secret: &str,
) -> Result<String, CryptoError> {
    encrypt_json(session, secret)
}

pub fn extract_session_cookie_value(
    encrypted_val: &str,
    secret: &str,
) -> Result<SessionData, CryptoError> {
    decrypt_json(encrypted_val, secret)
}

pub fn build_session_set_cookie_header(encrypted_val: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax{}",
        SESSION_COOKIE_NAME, encrypted_val, secure_flag
    )
}

pub fn build_clear_session_cookie_header(secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        SESSION_COOKIE_NAME, secure_flag
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_size_under_cookie_limit() {
        let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
        let session = SessionData {
            user_id: "usr_998877665544".to_string(),
            username: "alexander_the_great".to_string(),
            access_token: "eyJh...sample-access-token-1234567890-abcdefghijklmnopqrstuvwxyz..."
                .to_string(),
            refresh_token: Some(
                "sample-refresh-token-9876543210-zyxwvutsrqponmlkjihgfedcba".to_string(),
            ),
            expires_at: 1732000000,
        };

        let encrypted = create_session_cookie_value(&session, secret).expect("Encryption failed");
        let header = build_session_set_cookie_header(&encrypted, true);

        // Standard browser cookie limit is 4096 bytes
        assert!(
            header.len() < 4096,
            "Session cookie exceeds 4KB limit: {} bytes",
            header.len()
        );
        // Verify it is comfortably compact (< 1KB)
        assert!(
            header.len() < 1024,
            "Session cookie should be well under 1KB: {} bytes",
            header.len()
        );

        let roundtrip =
            extract_session_cookie_value(&encrypted, secret).expect("Decryption failed");
        assert_eq!(roundtrip, session);
    }
}
