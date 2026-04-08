use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Key, Nonce};

use crate::errors::AppError;

#[derive(Clone)]
pub struct FieldEncryptor {
    key: Key<Aes256Gcm>,
}

impl FieldEncryptor {
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        Self {
            key: *Key::<Aes256Gcm>::from_slice(key_bytes),
        }
    }

    /// Encrypts plaintext, returning nonce (12 bytes) || ciphertext+tag.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypts data produced by `encrypt`. Input: nonce (12 bytes) || ciphertext+tag.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, AppError> {
        if data.len() < 13 {
            return Err(AppError::Internal("Ciphertext too short".into()));
        }
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&data[..12]);
        let plaintext = cipher
            .decrypt(nonce, &data[12..])
            .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0xAA; 32]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let enc = FieldEncryptor::new(&test_key());
        let plaintext = b"hello world sensitive data";
        let ciphertext = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let enc = FieldEncryptor::new(&test_key());
        let plaintext = b"same input";
        let ct1 = enc.encrypt(plaintext).unwrap();
        let ct2 = enc.encrypt(plaintext).unwrap();
        // Different nonces should produce different ciphertext
        assert_ne!(ct1, ct2);
        // But both decrypt to the same plaintext
        assert_eq!(enc.decrypt(&ct1).unwrap(), plaintext);
        assert_eq!(enc.decrypt(&ct2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_too_short() {
        let enc = FieldEncryptor::new(&test_key());
        let result = enc.decrypt(&[0u8; 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let enc = FieldEncryptor::new(&test_key());
        let mut ciphertext = enc.encrypt(b"secret").unwrap();
        // Tamper with the ciphertext
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        let result = enc.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let enc1 = FieldEncryptor::new(&[0xAA; 32]);
        let enc2 = FieldEncryptor::new(&[0xBB; 32]);
        let ciphertext = enc1.encrypt(b"data").unwrap();
        let result = enc2.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let enc = FieldEncryptor::new(&test_key());
        let ct = enc.encrypt(b"").unwrap();
        let pt = enc.decrypt(&ct).unwrap();
        assert!(pt.is_empty());
    }

    #[test]
    fn test_large_plaintext() {
        let enc = FieldEncryptor::new(&test_key());
        let data = vec![0x42u8; 10_000];
        let ct = enc.encrypt(&data).unwrap();
        let pt = enc.decrypt(&ct).unwrap();
        assert_eq!(pt, data);
    }
}
