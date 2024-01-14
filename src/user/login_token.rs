//! Implements the login token generation and validation.
//!
//! Login tokens are made up of a user's ID and their login secret. The hash of
//! the login secret is stored locally, and compared to the hash of the secret
//! in the token on login attempts.
use crate::{DbConn, User};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use eyre::Result;
use sha2::{Digest, Sha512};

/// The secret bytes from a login token.
pub struct Secret([u8; 28]);

impl Secret {
    /// Generate a new login secret.
    pub fn new() -> Self {
        Self(rand::random())
    }

    /// Get the hash of this secret.
    ///
    /// This is the only thing that should be stored client side.
    pub fn hash(&self) -> [u8; 64] {
        Sha512::digest(self.0).into()
    }

    /// Create a token containing this secret and the given user ID.
    pub fn into_token(self, id: i32) -> String {
        let login_bytes = id
            .to_be_bytes()
            .iter()
            .chain(self.0.iter())
            .copied()
            .collect::<Vec<_>>();
        URL_SAFE_NO_PAD.encode(login_bytes)
    }

    /// Parse a login token into a user ID and secret.
    fn from_token(token: &str) -> Option<(i32, Self)> {
        let Ok(login_bytes) = URL_SAFE_NO_PAD.decode(token) else { return None; };
        if login_bytes.len() != 32 {
            return None;
        }
        let (id_bytes, secret) = login_bytes.split_at(4);
        let id = i32::from_be_bytes(id_bytes.try_into().expect("id to be 4 bytes"));
        Some((id, Self(secret.try_into().expect("secret to be 28 bytes"))))
    }

    /// Check if this secret matches a given hash.
    ///
    /// The hash must be the result of calling `hash` on another secret (but it
    /// could have been stored in a database since, for example).
    fn matches_hash(&self, hash: &[u8]) -> bool {
        constant_time_eq::constant_time_eq(&self.hash(), hash)
    }
}

impl User {
    /// Validate a login token and get the associated user account.
    pub async fn from_login_token(db: &mut DbConn, login: &str) -> Result<Option<Self>> {
        let Some((id, attempt)) = Secret::from_token(login) else {
            return Ok(None);
        };
        let Some(user) = Self::from_id(db, id).await? else {
            return Ok(None);
        };
        if attempt.matches_hash(&user.secret_hash) {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}
