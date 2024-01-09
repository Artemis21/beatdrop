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

/// A request guard for getting a database connection.
///
/// Routes which need to make a single database query, or don't care about
/// consistency, should take a `Connection` argument.
pub type Connection = rocket_db_pools::Connection<Main>;

/// A request guard for getting a database transaction.
///
/// Routes which need to make multiple database queries and need to ensure
/// consistency should take a `Transaction` argument.
pub struct Transaction<'c>(sqlx::Transaction<'c, sqlx::Postgres>);

impl<'c> std::ops::Deref for Transaction<'c> {
    type Target = sqlx::Transaction<'c, sqlx::Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'c> std::ops::DerefMut for Transaction<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Transaction<'_> {
    /// Commit the transaction.
    pub async fn commit(self) -> Result<(), Error> {
        self.0
            .commit()
            .await
            .wrap_err("error committing transaction for request")?;
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
            Ok(tx) => request::Outcome::Success(Self(tx)),
            Err(e) => {
                return request::Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    e.into(),
                ))
            }
        }
    }
}

/// The database connection type.
///
/// Methods (other than routes) which need access to the database should take a
/// `&mut DbConn` argument.
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
