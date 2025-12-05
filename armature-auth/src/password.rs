// Password hashing and verification

use crate::{AuthError, Result};
use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString, rand_core::OsRng,
    },
};

/// Password hashing algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// Bcrypt (slower but battle-tested)
    Bcrypt,
    /// Argon2 (modern, recommended)
    Argon2,
}

/// Password hasher
#[derive(Clone)]
pub struct PasswordHasher {
    algorithm: HashAlgorithm,
}

impl PasswordHasher {
    /// Create a new password hasher
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Hash a password
    pub fn hash(&self, password: &str) -> Result<String> {
        match self.algorithm {
            HashAlgorithm::Bcrypt => self.hash_bcrypt(password),
            HashAlgorithm::Argon2 => self.hash_argon2(password),
        }
    }

    /// Hash with bcrypt
    fn hash_bcrypt(&self, password: &str) -> Result<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))
    }

    /// Hash with argon2
    fn hash_argon2(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        Ok(password_hash.to_string())
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new(HashAlgorithm::Argon2)
    }
}

/// Password verifier
pub trait PasswordVerifier {
    /// Verify a password against a hash
    fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}

impl PasswordVerifier for PasswordHasher {
    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        // Auto-detect algorithm from hash format
        if hash.starts_with("$2") {
            // Bcrypt format
            bcrypt::verify(password, hash)
                .map_err(|e| AuthError::PasswordVerifyError(e.to_string()))
        } else if hash.starts_with("$argon2") {
            // Argon2 format
            let parsed_hash = PasswordHash::new(hash)
                .map_err(|e| AuthError::PasswordVerifyError(e.to_string()))?;

            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        } else {
            Err(AuthError::PasswordVerifyError(
                "Unknown hash format".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_hashing() {
        let hasher = PasswordHasher::new(HashAlgorithm::Bcrypt);
        let password = "test-password-123";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$2"));

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_argon2_hashing() {
        let hasher = PasswordHasher::new(HashAlgorithm::Argon2);
        let password = "test-password-456";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$argon2"));

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_auto_detect_algorithm() {
        let bcrypt_hasher = PasswordHasher::new(HashAlgorithm::Bcrypt);
        let argon2_hasher = PasswordHasher::new(HashAlgorithm::Argon2);

        let password = "test-password";

        let bcrypt_hash = bcrypt_hasher.hash(password).unwrap();
        let argon2_hash = argon2_hasher.hash(password).unwrap();

        // Should work regardless of hasher algorithm
        let verifier = PasswordHasher::default();
        assert!(verifier.verify(password, &bcrypt_hash).unwrap());
        assert!(verifier.verify(password, &argon2_hash).unwrap());
    }
}
