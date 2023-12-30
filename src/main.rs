//! The Beatdrop backend server.
//!
//! This is implemented using [Rocket](https://rocket.rs/), a web framework for Rust.
//! On startup, the server:
//! 1. reads a configuration file
//! 2. sets up a database pool
//! 3. runs migrations
//! 4. loads API routes from [`endpoints`]
//! 5. starts listening for requests
//!
//! Database connection and queries are handled by [SQLx](https://github.com/launchbadge/sqlx).
//! The server also connects to the [Deezer](https://developers.deezer.com/api) API to fetch
//! music data.
#![warn(clippy::all, clippy::pedantic, clippy::nursery, missing_docs, clippy::missing_docs_in_private_items)]
#![allow(clippy::no_effect_underscore_binding)]  // triggered by Rocket macro
use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod api;
// TODO: Document/refactor these modules
#[allow(clippy::missing_docs_in_private_items)]
mod game;
#[allow(clippy::missing_docs_in_private_items)]
mod user;
mod deezer;
mod track;
mod errors;
mod database;
mod web;

use user::{AuthConfig, User};
use game::Game;
use errors::{Error, ResultExt};
use database::{Db, DbConn};

/// Read config, set up the database and build the Rocket instance.
#[rocket::launch]
fn rocket() -> _ {
    let config_file = std::env::args().nth(1).unwrap_or_else(|| "beatdrop.toml".into());
    let config: Config = config::Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .add_source(config::Environment::with_prefix("BEATDROP"))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    let figment = rocket::Config::figment().merge(("databases.main.url", &config.db_url));
    rocket::custom(figment)
        .attach(database::Main::init())
        .attach(AdHoc::try_on_ignite("migrations", database::run_migrations))
        .manage(AuthConfig::from(&config))
        .manage(track::Config::from(&config))
        .mount("/api", api::routes())
        .mount("/", web::routes(config.dev))
}

/// Deserialisation struct for the configuration file.
#[derive(serde::Deserialize)]
struct Config {
    /// Database connection string (must be `postgres://`).
    db_url: String,
    /// Directory to store cached media files in.
    media_dir: std::path::PathBuf,
    /// Enable frontend development mode: serve the frontend from the filesystem instead of
    /// embedding it in the binary, and run `parcel watch` to rebuild on changes (with HMR!).
    /// This avoids having to rebuild and restart the server for frontend development.
    ///
    /// Yarn must be installed and the `yarn` command must be available on the path. This
    /// option is ignored in release builds.
    #[serde(default = "default_dev_mode")]
    dev: bool,
    /// Secret data to use as a private key for signing session tokens.
    session_key: String,
    /// How long a session token is valid for.
    #[serde(default = "default_session_lifetime")]
    session_lifetime: duration_string::DurationString,
}

/// Get the default configuration value for session lifetime.
fn default_session_lifetime() -> duration_string::DurationString {
    duration_string::DurationString::from_string("30d".into()).unwrap()
}

/// Enable dev mode by default in debug builds.
const fn default_dev_mode() -> bool {
    cfg!(debug_assertions)
}
