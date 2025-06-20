use crate::AuthCoreService;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};

impl AuthCoreService {
    /// Hash a password
    ///
    /// # Errors
    ///
    /// - If hashing the password fails (unlikely)
    pub fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        let argon2 = argon2::Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| anyhow::anyhow!(err))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify if the password matches the hash
    ///
    /// # Returns
    ///
    /// - `true` if the password matches the hash
    /// - `false` if it doesn't
    ///
    /// # Errors
    ///
    /// - If the hash couldn't be parsed
    pub fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        let argon2 = argon2::Argon2::default();

        let hash = PasswordHash::new(password_hash).map_err(|err| anyhow::anyhow!(err))?;
        let ok = argon2.verify_password(password.as_bytes(), &hash).is_ok();

        Ok(ok)
    }
}
