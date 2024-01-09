//! Background tasks management.
use crate::{database, track, Error, ResultExt};
use rocket_db_pools::Database;
use std::future::Future;

use clokwerk::{AsyncScheduler, Job, TimeUnits};

/// Spawn a task to take care of running periodic background tasks.
pub async fn spawn(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::fairing::Result {
    let Some(db) = database::Main::fetch(&rocket) else {
        eprintln!("couldn't retrieve db pool to set up background tasks");
        return Err(rocket);
    };
    let db = db.clone();
    run_background_task("ensure daily chosen at startup", || {
        ensure_daily_chosen(&db)
    })
    .await;
    let mut scheduler = AsyncScheduler::with_tz(chrono::Utc);
    scheduler.every(1.day()).at("00:00").run(move || {
        let db = db.clone();
        async move {
            run_background_task("ensure daily chosen at midnight", || {
                ensure_daily_chosen(&db)
            })
            .await;
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
    F: FnMut() -> O + Send,
>(
    name: &'static str,
    task: F,
) {
    let future = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(task)) {
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
