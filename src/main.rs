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
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    missing_docs,
    clippy::missing_docs_in_private_items
)]
#![allow(clippy::no_effect_underscore_binding)] // triggered by Rocket macro
use std::{future::Future, sync::Arc};

use clokwerk::{AsyncScheduler, Job, TimeUnits};
use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod api;
// TODO: Document/refactor these modules
mod database;
mod deezer;
mod errors;
#[allow(clippy::missing_docs_in_private_items)]
mod game;
mod track;
#[allow(clippy::missing_docs_in_private_items)]
mod user;
mod web;

use database::DbConn;
use errors::{Error, ResultExt};
use game::Game;
use user::{AuthHeader, User};

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
        .mount("/api", api::routes())
        .mount("/", web::routes(config.dev))
}
/*
/// Spawn a task to take care of running periodic background tasks.
async fn run_background_tasks(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    let Some(db) = database::Main::fetch(&rocket) else {
        eprintln!("couldn't retrieve db pool to set up background tasks");
        return Err(rocket);
    };
    let db = db.clone();
    run_background_task("ensure daily chosen at startup",  || {
        ensure_daily_chosen(&db)
    });
    let mut scheduler = AsyncScheduler::with_tz(chrono::Utc);
    scheduler.every(1.day()).at("00:00").run(move || {
        let db = db.clone();
        async move {
            if let Err(e) = ensure_daily_chosen(&db).await {
                eprintln!("{:?}", e.wrap_err("error picking daily track at midnight"));
            }
        }
    });
    rocket::tokio::task::spawn(async move {
        // Check for new tasks once a minute.
        scheduler.run_pending().await;
        rocket::tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    });
    Ok(rocket)
}

/// Run an async background task and catch any errors or panics.
async fn run_background_task<
    O: Future<Output = Result<(), Error>> + Send,
    F: FnMut() -> O + std::panic::UnwindSafe + Send,
>(
    name: &'static str,
    task: F,
) {
    let future = match std::panic::catch_unwind(task) {
        Ok(future) => future,
        Err(panic) => {
            eprintln!("panic in background task '{}': {:?}", name, panic);
            return;
        }
    };
    if let Err(e) = future.await {
        eprintln!("error in background task '{}': {:?}", name, e);
    }
}

/// Ensure a daily track has been picked.
///
/// This should run at startup and UTC midnight. While a track will be picked when
/// requested if this doesn't run first, picking in advance speeds up response time
/// and also ensures that the database is populated with tracks and related data for
/// other tasks.
async fn ensure_daily_chosen(db: &database::Main) -> Result<(), Error> {
    let mut conn = db
        .acquire()
        .await
        .wrap_err("error acquiring a database connection in a background task")?;
    track::pick::daily(&mut conn)
        .await
        .wrap_err("error picking a daily track as a background task")?;
    Ok(())
}
*/
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
