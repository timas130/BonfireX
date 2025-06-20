use crate::context::GlobalContext;
use async_graphql::{Context, ID};
use bfx_core::service::id_encryption::IdType;
use tonic::Status;

pub trait IdEncryptor {
    /// Encrypt an ID
    fn encrypt_id(&self, id_type: IdType, id: i64) -> ID;

    /// Decrypt an encrypted ID
    ///
    /// # Errors
    ///
    /// - If the ID type doesn't match
    /// - If the decryption fails
    fn decrypt_id(&self, id_type: IdType, id: &ID) -> Result<i64, Box<Status>>;
}

impl IdEncryptor for Context<'_> {
    fn encrypt_id(&self, id_type: IdType, id: i64) -> ID {
        let req = self.data_unchecked::<GlobalContext>();
        req.encrypt_id(id_type, id)
    }

    fn decrypt_id(&self, id_type: IdType, id: &ID) -> Result<i64, Box<Status>> {
        let req = self.data_unchecked::<GlobalContext>();
        req.decrypt_id(id_type, id)
    }
}

impl IdEncryptor for GlobalContext {
    fn encrypt_id(&self, id_type: IdType, id: i64) -> ID {
        ID(self.id_encryptor.encrypt_id(id_type, id))
    }

    fn decrypt_id(&self, id_type: IdType, id: &ID) -> Result<i64, Box<Status>> {
        self.id_encryptor.decrypt_id(id_type, &id.0)
    }
}
