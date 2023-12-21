//! Database connection and migration setup.
#![allow(clippy::option_if_let_else)]  // triggered by Rocket macro
use rocket::{request::Outcome, Request};
use rocket_db_pools::{sqlx, Connection, Database};

use crate::Error;

/// The main (and only) database connection pool.
#[derive(Database)]
#[database("main")]
pub struct Main(sqlx::PgPool);

/// A request guard for getting a database connection.
pub type Db = Connection<Main>;

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

/// Use the request guard to extract a database connection from a request.
pub async fn extract(req: &Request<'_>) -> Outcome<Db, Error> {
    req.guard::<Db>()
        .await
        .map_error(|(status, error)| error.map_or_else(|| (status, Error::DatabasePool(None)), |e| (status, e.into())))
}
