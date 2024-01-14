//! The Beatdrop backend server.
//!
//! This is implemented using [Rocket](https://rocket.rs/), a web framework for Rust.
//!
//! Database connection and queries are handled by [SQLx](https://github.com/launchbadge/sqlx).
//! The server also connects to the [Deezer](https://developers.deezer.com/api) API to fetch
//! music data.
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    missing_docs,
    clippy::missing_docs_in_private_items
)]
#![allow(clippy::no_effect_underscore_binding)] // triggered by Rocket macro
use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod api_error;
mod database;
mod deezer;
mod game;
mod tasks;
mod track;
mod user;
mod web;

use api_error::ApiError;
use database::{Connection, DbConn, Transaction};
use game::Game;
use user::{Session, User};

/// Read config, set up the database and build the Rocket instance.
#[rocket::launch]
fn rocket() -> _ {
    let config_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "beatdrop.toml".into());
    let config: Config = config::Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .add_source(config::Environment::with_prefix("BEATDROP"))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    let figment = rocket::Config::figment().merge(("databases.main.url", &config.db_url));
    track::init(&config);
    user::init(&config);
    rocket::custom(figment)
        .attach(database::Main::init())
        .attach(AdHoc::try_on_ignite("migrations", database::run_migrations))
        .attach(AdHoc::try_on_ignite("background tasks", tasks::spawn))
        .mount("/api", api_routes())
        .mount("/", web::routes(config.dev))
}

/// Collect all the API routes.
fn api_routes() -> Vec<rocket::Route> {
    let mut routes = game::routes();
    routes.extend(user::routes());
    routes.extend(track::routes());
    routes
}

/// Deserialisation struct for the configuration file.
#[derive(serde::Deserialize)]
pub struct Config {
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
