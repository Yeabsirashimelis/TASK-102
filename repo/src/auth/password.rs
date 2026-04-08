use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::errors::AppError;

/// Validates password meets policy: min 12 chars, 1 upper, 1 lower, 1 digit.
pub fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    if password.len() < 12 {
        let mut err = validator::ValidationError::new("password_too_short");
        err.message = Some("Password must be at least 12 characters".into());
        return Err(err);
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        let mut err = validator::ValidationError::new("password_no_upper");
        err.message = Some("Password must contain at least one uppercase letter".into());
        return Err(err);
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        let mut err = validator::ValidationError::new("password_no_lower");
        err.message = Some("Password must contain at least one lowercase letter".into());
        return Err(err);
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        let mut err = validator::ValidationError::new("password_no_digit");
        err.message = Some("Password must contain at least one digit".into());
        return Err(err);
    }
    Ok(())
}

/// Hashes a password using Argon2id. Returns the PHC-format hash string.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
    Ok(hash.to_string())
}

/// Verifies a password against an Argon2id PHC-format hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        assert!(validate_password("Abcdefghij1k").is_ok());
    }

    #[test]
    fn test_too_short() {
        assert!(validate_password("Ab1defghijk").is_err());
    }

    #[test]
    fn test_no_upper() {
        assert!(validate_password("abcdefghij1k").is_err());
    }

    #[test]
    fn test_no_lower() {
        assert!(validate_password("ABCDEFGHIJ1K").is_err());
    }

    #[test]
    fn test_no_digit() {
        assert!(validate_password("Abcdefghijkl").is_err());
    }

    #[test]
    fn test_hash_and_verify() {
        let hash = hash_password("Abcdefghij1k").unwrap();
        assert!(verify_password("Abcdefghij1k", &hash).unwrap());
        assert!(!verify_password("WrongPassword1", &hash).unwrap());
    }

    #[test]
    fn test_exactly_12_chars_valid() {
        assert!(validate_password("Abcdefghij1k").is_ok());
    }

    #[test]
    fn test_11_chars_invalid() {
        assert!(validate_password("Abcdefghi1k").is_err());
    }

    #[test]
    fn test_all_lowercase_with_digit() {
        assert!(validate_password("abcdefghij1k").is_err());
    }

    #[test]
    fn test_all_uppercase_with_digit() {
        assert!(validate_password("ABCDEFGHIJ1K").is_err());
    }

    #[test]
    fn test_no_digit_long_enough() {
        assert!(validate_password("Abcdefghijkl").is_err());
    }

    #[test]
    fn test_special_chars_accepted() {
        assert!(validate_password("Abcdefghi1!@").is_ok());
    }

    #[test]
    fn test_hash_produces_different_hashes() {
        let h1 = hash_password("Abcdefghij1k").unwrap();
        let h2 = hash_password("Abcdefghij1k").unwrap();
        // Different salts produce different hashes
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let result = verify_password("anything", "not-a-valid-hash");
        assert!(result.is_err());
    }
}
