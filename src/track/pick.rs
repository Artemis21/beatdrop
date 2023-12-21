//! Procedures to pick a track for a new game.
use crate::{DbConn, Error, deezer};
use super::bulk_insert::BulkInserter as BulkTrackInserter;

/// Pick any track from the database, preferring more popular tracks.
///
/// If no tracks are available, get fresh data and try again.
pub async fn any(db: &mut DbConn) -> Result<i32, Error> {
    if let Some(track) = try_pick_any(db).await? {
        return Ok(track);
    }
    refresh_all(db).await?;
    if let Some(track) = try_pick_any(db).await? {
        return Ok(track);
    }
    Err(Error::NotFound("no appropriate track found"))
}

/// Pick any track from the database, preferring more popular tracks.
async fn try_pick_any(db: &mut DbConn) -> Result<Option<i32>, Error> {
    let track = sqlx::query_scalar!(
        "SELECT track.id FROM track
        ORDER BY RANDOM() * track.deezer_rank DESC
        LIMIT 1",
    )
    .fetch_optional(db)
    .await?;
    Ok(track)
}

/// Pick a track from the specified genre, preferring more popular tracks.
///
/// If no tracks are available, get fresh data and try again.
pub async fn genre(
    db: &mut DbConn,
    genre_id: i32,
) -> Result<i32, Error> {
    if let Some(track) = try_pick_genre(db, genre_id).await? {
        return Ok(track);
    }
    refresh_genre(db, genre_id).await?;
    if let Some(track) = try_pick_genre(db, genre_id).await? {
        return Ok(track);
    }
    Err(Error::NotFound("no appropriate track found in the specified genre"))
}

/// Pick a track from the specified genre, preferring more popular tracks.
async fn try_pick_genre(db: &mut DbConn, genre_id: i32) -> Result<Option<i32>, Error> {
    let track = sqlx::query_scalar!(
        "SELECT track.id FROM track
        INNER JOIN album ON track.album_id = album.id
        INNER JOIN album_genre ON album.id = album_genre.album_id
        WHERE album_genre.genre_id = $1
        ORDER BY RANDOM() * track.deezer_rank DESC
        LIMIT 1",
        genre_id
    )
    .fetch_optional(db)
    .await?;
    Ok(track)
}

/// Get the current daily track, or pick a new one if none is set.
pub async fn daily(db: &mut DbConn) -> Result<i32, Error> {
    let track = sqlx::query_scalar!(
        "SELECT track_id FROM daily_track
        WHERE for_day = TIMEZONE('utc', NOW())::DATE"
    ).fetch_optional(&mut *db).await?;
    if let Some(track) = track {
        return Ok(track);
    }
    let track = pick_daily(&mut *db).await?;
    sqlx::query!(
        "INSERT INTO daily_track (track_id) VALUES ($1)",
        track
    )
    .execute(db)
    .await?;
    Ok(track)
}

/// Pick a track for today, preferring more popular tracks.
///
/// Tries to avoid repeating tracks from the past 100 days.
async fn pick_daily(db: &mut DbConn) -> Result<i32, Error> {
    // We only do this once a day, so it's fine to always refresh first.
    refresh_all(db).await?;
    if let Some(track) = try_pick_daily(db).await? {
        return Ok(track);
    }
    // If we have to, repeat a track from the past 100 days
    if let Some(track) = try_pick_any(db).await? {
        return Ok(track);
    }
    Err(Error::NotFound("no daily track found"))
}

/// Pick a track for today, preferring more popular tracks and avoiding tracks from the past 100 days.
async fn try_pick_daily(db: &mut DbConn) -> Result<Option<i32>, Error> {
    let track = sqlx::query_scalar!(
        "SELECT track.id FROM track
        LEFT JOIN daily_track ON track.id = daily_track.track_id
        WHERE daily_track.track_id IS NULL OR daily_track.for_day < TIMEZONE('utc', NOW()) - INTERVAL '100 DAY'
        ORDER BY RANDOM() * track.deezer_rank DESC
        LIMIT 1",
    )
    .fetch_optional(db)
    .await?;
    Ok(track)
}

/// Refresh the database with fresh data in the most popular genres from Deezer.
async fn refresh_all(db: &mut DbConn) -> Result<(), Error> {
    let genres = deezer::genres().await?;
    let mut inserter = BulkTrackInserter::new(db);
    for genre in genres {
        let chart = deezer::chart(genre.id).await?;
        for track in chart {
            inserter.insert_track(&track).await?;
        }
    }
    Ok(())
}

/// Refresh the database with fresh data in the specified genre from Deezer.
async fn refresh_genre(
    db: &mut DbConn,
    genre_id: i32,
) -> Result<(), Error> {
    let chart = deezer::chart(genre_id).await?;
    let mut inserter = BulkTrackInserter::new(db);
    for track in chart {
        inserter.insert_track(&track).await?;
    }
    Ok(())
}
