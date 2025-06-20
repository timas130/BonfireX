use crate::service::environment::require_env;
use crate::status::{ErrorCode, StatusExt};
use aes_gcm_siv::aead::{Aead, Payload};
use aes_gcm_siv::{Aes256GcmSiv, KeyInit, Nonce};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use hkdf::Hkdf;
use sha2::Sha256;
use tonic::{Code, Status};

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum IdType {
    User = 1,
    Session,
    LoginAttempt,
    OAuthFlow,
    OAuthClient,
    Image,
    AuthSource,
}

#[derive(Clone)]
pub struct IdEncryptor {
    id_encryption_key: aes_gcm_siv::Key<Aes256GcmSiv>,
    nonce_salt: [u8; 16],
}

impl IdEncryptor {
    /// Encrypt an ID
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn encrypt_id(&self, id_type: IdType, id: i64) -> String {
        let data = id.to_be_bytes();
        let aad = [id_type as u8];

        // nonce1 (8b) <== id_type + id + nonce_salt
        let mut nonce_material = [0u8; 1 + 8 + 16];
        nonce_material[0] = id_type as u8;
        nonce_material[1..9].copy_from_slice(&data);
        nonce_material[9..].copy_from_slice(&self.nonce_salt);
        let hk1 = Hkdf::<Sha256>::new(None, &nonce_material);
        let mut nonce1 = [0u8; 8];
        // this will only panic if nonce1.len() is *very* big, so .unwrap() is fine
        hk1.expand(b"nonce1", &mut nonce1).unwrap();

        // nonce2 (12b) <== nonce1
        let hk2 = Hkdf::<Sha256>::new(None, &nonce1);
        let mut nonce2 = [0u8; 12];
        // this will only panic if nonce2.len() is *very* big, so .unwrap() is fine
        hk2.expand(b"nonce2", &mut nonce2).unwrap();

        let nonce = Nonce::from(nonce2);

        // encrypted id <== aes256gcmsiv(key: key, nonce: nonce2, plaintext: id, aad: id_type)
        let cipher = Aes256GcmSiv::new(&self.id_encryption_key);
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &data,
                    aad: &aad,
                },
            )
            // this will only throw an error if the payload is very big, so .unwrap() is fine
            .unwrap();

        // result <== id_type + nonce1 + ciphertext
        let mut result = Vec::with_capacity(1 + 8 + ciphertext.len());
        result.extend_from_slice(&[id_type as u8]);
        result.extend_from_slice(&nonce1);
        result.extend_from_slice(&ciphertext);

        BASE64_URL_SAFE_NO_PAD.encode(&result)
    }

    /// Decrypt an encrypted ID
    ///
    /// # Errors
    ///
    /// - If the ID type doesn't match
    /// - If the decryption fails
    #[allow(clippy::missing_panics_doc)]
    pub fn decrypt_id(&self, id_type: IdType, id: &str) -> Result<i64, Box<Status>> {
        let data = BASE64_URL_SAFE_NO_PAD.decode(id).map_err(|_| {
            Status::coded(Code::InvalidArgument, ErrorCode::InvalidId)
                .with_details("invalid encoding")
        })?;

        // data == id_type .. nonce1 .. ciphertext
        if data.len() != 1 + 8 + 24 {
            Err(Status::coded(Code::InvalidArgument, ErrorCode::InvalidId)
                .with_details("invalid data length"))?;
        }

        let data_id_type = data[0];
        if data_id_type != id_type as u8 {
            Err(Status::coded(Code::InvalidArgument, ErrorCode::InvalidId)
                .with_details("id type mismatch"))?;
        }

        let nonce = &data[1..9];
        let ciphertext = &data[9..];

        // nonce2 (12b) <== nonce1
        let hk2 = Hkdf::<Sha256>::new(None, nonce);
        let mut nonce2 = [0u8; 12];
        // this will only panic if nonce2.len() is *very* big, so .unwrap() is fine
        hk2.expand(b"nonce2", &mut nonce2).unwrap();

        let nonce = Nonce::from(nonce2);

        // id <== de_aes256gcmsiv(key: key, nonce: nonce2, ciphertext: ciphertext, aad: id_type)
        let cipher = Aes256GcmSiv::new(&self.id_encryption_key);
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: &[data_id_type],
                },
            )
            .map_err(|_| {
                Status::coded(Code::InvalidArgument, ErrorCode::InvalidId)
                    .with_details("decryption failed")
            })?;

        if plaintext.len() != 8 {
            Err(Status::coded(Code::InvalidArgument, ErrorCode::InvalidId)
                .with_details("decryption failed"))?;
        }

        // we've just checked that plaintext.len() == 8, so .unwrap() is fine
        let id = i64::from_be_bytes(plaintext[..8].try_into().unwrap());

        Ok(id)
    }
}

// utilities

/// Construct an [`IdEncryptor`] from the `ID_ENCRYPTION_KEY` environment variable
///
/// # Errors
///
/// - If the environment variable is not set
/// - Miscellaneous internal errors
pub fn require_id_encryptor() -> anyhow::Result<IdEncryptor> {
    let id_encryption_key = require_id_encryption_key()?;
    let nonce_salt = require_nonce_salt()?;

    Ok(IdEncryptor {
        id_encryption_key,
        nonce_salt,
    })
}

/// Derive the ID encryption key from the `ID_ENCRYPTION_KEY` environment variable
///
/// # Errors
///
/// - If the environment variable is not set
/// - Miscellaneous internal errors
fn require_id_encryption_key() -> anyhow::Result<aes_gcm_siv::Key<Aes256GcmSiv>> {
    let key = require_env("ID_ENCRYPTION_KEY")?;

    let hk = Hkdf::<Sha256>::new(None, key.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"id-encryption-key", &mut key)
        .map_err(|err| anyhow::anyhow!("invalid ID_ENCRYPTION_KEY: {err:?}"))?;

    Ok(key.into())
}

/// Derive the nonce salt used for ID encryption from the `ID_ENCRYPTION_KEY` environment variable
///
/// # Errors
///
/// - If the environment variable is not set
/// - Miscellaneous internal errors
fn require_nonce_salt() -> anyhow::Result<[u8; 16]> {
    let salt = require_env("ID_ENCRYPTION_KEY")?;

    let hk = Hkdf::<Sha256>::new(None, salt.as_bytes());
    let mut nonce_salt = [0u8; 16];
    hk.expand(b"nonce-salt", &mut nonce_salt)
        .map_err(|err| anyhow::anyhow!("invalid ID_ENCRYPTION_KEY: {err:?}"))?;

    Ok(nonce_salt)
}
