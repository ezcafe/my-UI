use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    Encryption,
    #[error("Decryption failed")]
    Decryption,
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Invalid secret length: must be at least 32 bytes")]
    InvalidSecret,
}

pub fn derive_key(secret: &str) -> Result<[u8; 32], CryptoError> {
    if secret.len() < 32 {
        return Err(CryptoError::InvalidSecret);
    }
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    Ok(hasher.finalize().into())
}

pub fn encrypt_json<T: Serialize>(data: &T, secret: &str) -> Result<String, CryptoError> {
    let key = derive_key(secret)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::Encryption)?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let plaintext = serde_json::to_vec(data)?;
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|_| CryptoError::Encryption)?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

#[allow(deprecated)]
pub fn decrypt_json<T: for<'de> Deserialize<'de>>(encrypted_b64: &str, secret: &str) -> Result<T, CryptoError> {
    let key = derive_key(secret)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::Decryption)?;

    let combined = BASE64.decode(encrypted_b64)?;
    if combined.len() < 12 {
        return Err(CryptoError::Decryption);
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| CryptoError::Decryption)?;
    let data = serde_json::from_slice(&plaintext)?;
    Ok(data)
}
