//! Create and validate session tokens.
//!
//! Session tokens are JSON Web Tokens (JWTs) signed with a secret key. The token contains the
//! account ID and the time the token was created.
use crate::{DbConn, User};
use chrono::{DateTime, Utc};
use eyre::Result;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha512;
use std::sync::OnceLock;

/// Config options for the session system.
#[derive(Debug)]
struct SessionConfig {
    /// The secret key used to sign sessions tokens.
    pub key: Hmac<Sha512>,
    /// How long a session token is valid for.
    pub session_lifetime: chrono::Duration,
}

/// The current session config, initialised once at startup.
static SESSION_CONFIG: OnceLock<SessionConfig> = OnceLock::new();

impl From<&crate::Config> for SessionConfig {
    fn from(config: &crate::Config) -> Self {
        Self {
            key: Hmac::new_from_slice(config.session_key.as_bytes()).expect("any length is valid"),
            session_lifetime: chrono::Duration::from_std(config.session_lifetime.into())
                .expect("session lifetime to fit in chrono::Duration"),
        }
    }
}

/// Initialise the session system. This must be called exactly once, before
/// creating or validating any session tokens.
pub fn init(config: &crate::Config) {
    SESSION_CONFIG
        .set(config.into())
        .expect("user::init must only be called once");
}

impl User {
    /// Get the user account for an otherwise valid session.
    ///
    /// The session must have already be validated, other than checking that
    /// the referenced user still exists.
    pub async fn from_session(claim: Session, db: &mut DbConn) -> Result<Option<Self>> {
        Self::from_id(db, claim.account_id).await
    }

    /// Create a new session token for this user.
    pub fn session_token(&self) -> String {
        let conf = SESSION_CONFIG
            .get()
            .expect("User::session_token used before initialisation");
        let header = jwt::Header {
            algorithm: jwt::AlgorithmType::Hs512,
            ..Default::default()
        };
        let created_at = Utc::now();
        let claim = Session {
            account_id: self.id,
            created_at,
        };
        let token = jwt::Token::new(header, claim)
            .sign_with_key(&conf.key)
            .expect("signing to succeed");
        token.as_str().to_string()
    }
}

/// The data contained within a session token.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// The ID of the account this session is for.
    account_id: i32,
    /// The time this session was created.
    ///
    /// This is used to check the session hasn't expired.
    created_at: DateTime<Utc>,
}

impl Session {
    /// Validate and decode a session from an `Authorization` header.
    pub fn from_auth_header(header: &str) -> Result<Self, &'static str> {
        let conf = SESSION_CONFIG
            .get()
            .expect("Session::from_auth_header used before initialisation");
        let token = header
            .strip_prefix("Bearer ")
            .ok_or("invalid auth header format")?;
        let claim: Self = token
            .verify_with_key(&conf.key)
            .map_err(|_| "invalid session token")?;
        if claim.created_at + conf.session_lifetime < Utc::now() {
            return Err("session token expired");
        }
        Ok(claim)
    }
}
