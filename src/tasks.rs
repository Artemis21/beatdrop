//! Background tasks management.
use crate::{database, game::Game, track};
use eyre::{Context, Result};
use rocket_db_pools::Database;
use std::{future::Future, sync::OnceLock};

use clokwerk::{AsyncScheduler, Job, TimeUnits};

/// A reference to the global database pool, for use in background tasks.
static DB_POOL: OnceLock<database::Main> = OnceLock::new();

/// Spawn a task to take care of running periodic background tasks.
pub async fn spawn(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    let Some(db) = database::Main::fetch(&rocket) else {
        eprintln!("couldn't retrieve db pool to set up background tasks");
        return Err(rocket);
    };
    DB_POOL
        .set(db.clone())
        .expect("background tasks must only be spawned once");
    run_background_task("ensure daily chosen at startup", ensure_daily_chosen).await;
    let mut scheduler = AsyncScheduler::with_tz(chrono::Utc);
    scheduler
        .every(1.day())
        .at("00:00")
        .run(|| run_background_task("ensure daily chosen at midnight", ensure_daily_chosen));
    scheduler
        .every(1.minute())
        .run(|| run_background_task("end timed-out games", end_timed_out_games));
    rocket::tokio::task::spawn(async move {
        // Check for new tasks once a minute.
        // FIXME: these tasks don't appear to be actually running
        scheduler.run_pending().await;
        rocket::tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    });
    Ok(rocket)
}

/// Run an async background task and catch any errors or panics.
async fn run_background_task<O: Future<Output = Result<()>> + Send, F: FnMut() -> O + Send>(
    name: &'static str,
    task: F,
) {
    let future = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(task)) {
        Ok(future) => future,
        Err(panic) => {
            eprintln!("panic in background task '{name}': {panic:?}");
            return;
        }
    };
    if let Err(e) = future.await {
        eprintln!("error in background task '{name}': {e:?}");
    }
}

/// Get a database connection from the pool.
async fn db_conn() -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>> {
    DB_POOL
        .get()
        .expect("connection requested by task before pool was initialised")
        .acquire()
        .await
        .wrap_err("error acquiring a database connection in a background task")
}

/// Ensure a daily track has been picked.
///
/// This should run at startup and UTC midnight. While a track will be picked when
/// requested if this doesn't run first, picking in advance speeds up response time
/// and also ensures that the database is populated with tracks and related data for
/// other tasks.
async fn ensure_daily_chosen() -> Result<()> {
    track::pick::daily(&mut *db_conn().await?)
        .await
        .wrap_err("error picking a daily track as a background task")?;
    Ok(())
}

/// End all timed-mode games which have run out of time.
async fn end_timed_out_games() -> Result<()> {
    Game::end_all_timed_out(&mut *db_conn().await?)
        .await
        .wrap_err("error ending timed-out games as a background task")?;
    Ok(())
}
