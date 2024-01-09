//! Database connection and migration setup.
#![allow(clippy::option_if_let_else)]
// triggered by Rocket macro
use rocket::request::{self, FromRequest, Request};
use rocket_db_pools::{sqlx, Database};

use crate::{deezer, Error, ResultExt};

/// The main (and only) database connection pool.
#[derive(Database, Clone)]
#[database("main")]
pub struct Main(sqlx::PgPool);

/// A request guard for getting a database transaction.
pub struct Transaction<'c> {
    tx: sqlx::Transaction<'c, sqlx::Postgres>,
    #[cfg(debug_assertions)]
    ended: TxEnded,
}

/// A `Drop` type used to warn when a transaction is dropped without being committed
/// or rolled back.
#[cfg(debug_assertions)]
struct TxEnded {
    ended: bool,
    context: String,
}

#[cfg(debug_assertions)]
impl TxEnded {
    fn new(ctx: impl ToString) -> Self {
        Self {
            ended: false,
            context: ctx.to_string(),
        }
    }

    fn end(&mut self) {
        self.ended = true;
    }
}

#[cfg(debug_assertions)]
impl std::ops::Drop for TxEnded {
    fn drop(&mut self) {
        if !self.ended {
            eprintln!(
                "A transaction for {} was dropped without committing or rolling back!",
                self.context
            );
        }
    }
}

impl<'c> std::ops::Deref for Transaction<'c> {
    type Target = sqlx::Transaction<'c, sqlx::Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<'c> std::ops::DerefMut for Transaction<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

impl Transaction<'_> {
    pub async fn commit(mut self) -> Result<(), Error> {
        self.tx
            .commit()
            .await
            .wrap_err("error committing transaction for request")?;
        #[cfg(debug_assertions)]
        {
            self.ended.end();
        }
        Ok(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Transaction<'r> {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let db = Main::fetch(req.rocket())
            .expect("application is running, so database must be initialised");
        let tx = db.begin().await;
        match tx {
            Ok(tx) => request::Outcome::Success(Self {
                tx,
                #[cfg(debug_assertions)]
                ended: TxEnded::new(req),
            }),
            Err(e) => {
                return request::Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    e.into(),
                ))
            }
        }
    }
}

/// A request guard for getting a database connection.
pub type Db<'c> = Transaction<'c>; // Connection<Main>;

/// The database connection type.
///
/// Methods which need access to the database should take a `&mut DbConn` argument.
pub type DbConn = sqlx::PgConnection;

/// Run database migrations as a Rocket fairing.
pub async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    if let Some(db) = Main::fetch(&rocket) {
        sqlx::migrate!("./migrations").run(&db.0).await.unwrap();
        Ok(rocket)
    } else {
        Err(rocket)
    }
}

impl From<i32> for deezer::Id {
    #[allow(clippy::cast_sign_loss)]
    fn from(id: i32) -> Self {
        Self(id as u32)
    }
}

impl From<deezer::Id> for i32 {
    #[allow(clippy::cast_possible_wrap)]
    fn from(id: deezer::Id) -> Self {
        id.0 as Self
    }
}

impl std::fmt::Display for deezer::Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A wrapper around `Option<deezer::Id>`, used for deserialising database rows with sqlx.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct DeezerOptionId(Option<deezer::Id>);

impl From<Option<i32>> for DeezerOptionId {
    fn from(id: Option<i32>) -> Self {
        match id {
            Some(id) => Self(Some(id.into())),
            None => Self(None),
        }
    }
}

impl std::ops::Deref for DeezerOptionId {
    type Target = Option<deezer::Id>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
