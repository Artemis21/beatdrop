use std::sync::OnceLock;

use crate::{DbConn, Error, ResultExt};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use rocket::{
    request::{self, FromRequest},
    Request,
};
use serde::Serialize;
use sha2::{Digest, Sha512};

static AUTH_CONFIG: OnceLock<AuthConfig> = OnceLock::new();

#[derive(Debug)]
struct AuthConfig {
    pub key: Hmac<Sha512>,
    pub session_lifetime: chrono::Duration,
}

impl From<&crate::Config> for AuthConfig {
    fn from(config: &crate::Config) -> Self {
        Self {
            key: Hmac::new_from_slice(config.session_key.as_bytes()).expect("any length is valid"),
            session_lifetime: chrono::Duration::from_std(config.session_lifetime.into())
                .expect("session lifetime to fit in chrono::Duration"),
        }
    }
}

pub fn init(config: &crate::Config) {
    AUTH_CONFIG
        .set(config.into())
        .expect("user::init must only be called once");
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    secret_hash: Vec<u8>,
}

impl User {
    pub async fn create(db: &mut DbConn) -> Result<(Self, String), Error> {
        let secret: [u8; 28] = rand::random();
        let secret_hash: [u8; 64] = sha2::Sha512::digest(secret).into();
        let user = sqlx::query_as!(
            User,
            "INSERT INTO account (secret_hash) VALUES ($1) RETURNING *",
            &secret_hash
        )
        .fetch_one(db)
        .await
        .wrap_err("error inserting new user account")?;
        let login_bytes = user
            .id
            .to_be_bytes()
            .iter()
            .chain(secret.iter())
            .copied()
            .collect::<Vec<_>>();
        let login = URL_SAFE_NO_PAD.encode(login_bytes);
        Ok((user, login))
    }

    async fn from_id(id: i32, db: &mut DbConn) -> Result<Option<Self>, Error> {
        sqlx::query_as!(User, "SELECT * FROM account WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .map_err(Error::from)
    }

    pub async fn from_session(session: AuthHeader<'_>, db: &mut DbConn) -> Result<Self, Error> {
        let conf = AUTH_CONFIG
            .get()
            .expect("User::from_session used before initialisation");
        let claim: AuthClaim = session
            .0
            .verify_with_key(&conf.key)
            .map_err(|_| Error::Auth("invalid session token"))?;
        if claim.created_at + conf.session_lifetime < Utc::now() {
            return Err(Error::Auth("session expired"));
        }
        let Some(user) = Self::from_id(claim.account_id, db).await? else {
            return Err(Error::Auth("invalid session token"));
        };
        Ok(user)
    }

    pub async fn from_login(login: &str, db: &mut DbConn) -> Result<Self, Error> {
        let login_bytes = URL_SAFE_NO_PAD
            .decode(login)
            .map_err(|_| Error::Auth("invalid secret"))?;
        if login_bytes.len() != 32 {
            return Err(Error::Auth("invalid secret"));
        }
        let (id_bytes, attempt_secret) = login_bytes.split_at(4);
        let id = i32::from_be_bytes(id_bytes.try_into().expect("id to be 4 bytes"));
        let Some(user) = Self::from_id(id, db).await? else {
            return Err(Error::Auth("invalid secret"));
        };
        let attempt_hash: [u8; 64] = sha2::Sha512::digest(attempt_secret).into();
        if constant_time_eq::constant_time_eq(&user.secret_hash, &attempt_hash) {
            Ok(user)
        } else {
            Err(Error::Auth("invalid secret"))
        }
    }

    pub fn session_token(&self) -> String {
        let conf = AUTH_CONFIG
            .get()
            .expect("User::session_token used before initialisation");
        let header = jwt::Header {
            algorithm: jwt::AlgorithmType::Hs512,
            ..Default::default()
        };
        let created_at = Utc::now();
        let claim = AuthClaim {
            account_id: self.id,
            created_at,
        };
        let token = jwt::Token::new(header, claim)
            .sign_with_key(&conf.key)
            .expect("signing to succeed");
        token.as_str().to_string()
    }

    pub async fn set_display_name(
        self,
        display_name: Option<&str>,
        db: &mut DbConn,
    ) -> Result<Self, Error> {
        sqlx::query_as!(
            User,
            "UPDATE account SET display_name = $1 WHERE id = $2 RETURNING *",
            display_name.as_deref(),
            self.id
        )
        .fetch_one(db)
        .await
        .map_err(Error::from)
    }

    pub async fn delete(self, db: &mut DbConn) -> Result<(), Error> {
        sqlx::query!("DELETE FROM account WHERE id = $1", self.id)
            .execute(db)
            .await
            .map_err(Error::from)
            .map(|_| ())
    }
}

pub struct AuthHeader<'r>(&'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHeader<'r> {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        req.headers()
            .get_one("Authorization")
            .and_then(|s| s.strip_prefix("Bearer "))
            .map_or_else(
                || {
                    request::Outcome::Error((
                        rocket::http::Status::Unauthorized,
                        Error::Auth("missing session token"),
                    ))
                },
                |token| request::Outcome::Success(Self(token)),
            )
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthClaim {
    account_id: i32,
    created_at: DateTime<Utc>,
}
