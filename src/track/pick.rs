//! Procedures to pick a track for a new game.
use eyre::eyre;

use super::bulk_insert::BulkInserter as BulkTrackInserter;
use crate::{deezer, DbConn};
use eyre::{Context, Result};

/// Pick any track from the database, preferring more popular tracks.
///
/// If no tracks are available, get fresh data and try again. Avoid tracks the
/// given user has recently played.
pub async fn any(db: &mut DbConn, user: i32) -> Result<deezer::Id> {
    if let Some(track) = try_pick_any(db, user).await? {
        return Ok(track);
    }
    refresh_all(db)
        .await
        .wrap_err("error fetching new tracks for the database")?;
    if let Some(track) = try_pick_any(db, user)
        .await
        .wrap_err("error trying to find any track in the database, right after refresh")?
    {
        return Ok(track);
    }
    Err(eyre!("couldn't find any track"))
}

/// Pick any track from the database, preferring more popular tracks and
/// avoiding tracks the given user has recently played.
async fn try_pick_any(db: &mut DbConn, user: i32) -> Result<Option<deezer::Id>> {
    let track = sqlx::query_scalar!(
        "SELECT track.id FROM track
        LEFT JOIN game ON track.id = game.track_id AND game.account_id = $1
        ORDER BY
            game.started_at ASC NULLS FIRST,
            RANDOM() * track.deezer_rank DESC
        LIMIT 1",
        user
    )
    .fetch_optional(db)
    .await
    .wrap_err("error querying for a random track")?;
    Ok(track.map(From::from))
}

/// Pick a track from the specified genre, preferring more popular tracks.
///
/// If no tracks are available, get fresh data and try again. Avoid tracks the
/// given user has recently played.
pub async fn genre(db: &mut DbConn, genre_id: deezer::Id, user: i32) -> Result<deezer::Id> {
    if let Some(track) = try_pick_genre(db, genre_id, user).await? {
        return Ok(track);
    }
    refresh_genre(db, genre_id).await?;
    if let Some(track) = try_pick_genre(db, genre_id, user)
        .await
        .wrap_err("error trying to find a track in the specified genre, right after refresh")?
    {
        return Ok(track);
    }
    Err(eyre!("couldn't find any track in the specified genre"))
}

/// Pick a track from the specified genre, preferring more popular tracks and
/// avoiding tracks the given user has recently played.
async fn try_pick_genre(
    db: &mut DbConn,
    genre_id: deezer::Id,
    user: i32,
) -> Result<Option<deezer::Id>> {
    let track = sqlx::query_scalar!(
        "SELECT track.id FROM track
        INNER JOIN album ON track.album_id = album.id
        INNER JOIN album_genre ON album.id = album_genre.album_id
        LEFT JOIN game ON track.id = game.track_id AND game.account_id = $2
        WHERE album_genre.genre_id = $1
        ORDER BY
            game.started_at ASC NULLS FIRST,
            RANDOM() * track.deezer_rank DESC
        LIMIT 1",
        i32::from(genre_id),
        user
    )
    .fetch_optional(db)
    .await
    .wrap_err("error querying for a random track in the specified genre")?;
    Ok(track.map(From::from))
}

/// Get the current daily track, or pick a new one if none is set.
pub async fn daily(db: &mut DbConn) -> Result<deezer::Id> {
    let track = sqlx::query_scalar!(
        "SELECT track_id FROM daily_track
        WHERE for_day = TIMEZONE('utc', NOW())::DATE"
    )
    .fetch_optional(&mut *db)
    .await
    .wrap_err("error querying for the daily track")?;
    if let Some(track) = track {
        return Ok(track.into());
    }
    let track = pick_daily(&mut *db).await?;
    sqlx::query!(
        "INSERT INTO daily_track (track_id) VALUES ($1)",
        i32::from(track)
    )
    .execute(db)
    .await
    .wrap_err("error inserting the daily track")?;
    Ok(track)
}

/// Pick a track for today, preferring more popular tracks.
///
/// Avoids repeating recent tracks.
async fn pick_daily(db: &mut DbConn) -> Result<deezer::Id> {
    // We only do this once a day, so it's fine to always refresh first.
    refresh_all(db).await?;
    sqlx::query_scalar!(
        "SELECT track.id FROM track
        LEFT JOIN daily_track ON track.id = daily_track.track_id
        ORDER BY
            daily_track.for_day ASC NULLS FIRST,
            RANDOM() * track.deezer_rank DESC
        LIMIT 1",
    )
    .fetch_one(db)
    .await
    .wrap_err("couldn't find any track for the daily")
    .map(From::from)
}

/// Refresh the database with fresh data in the most popular genres from Deezer.
async fn refresh_all(db: &mut DbConn) -> Result<()> {
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
async fn refresh_genre(db: &mut DbConn, genre_id: deezer::Id) -> Result<()> {
    let chart = deezer::chart(genre_id).await?;
    let mut inserter = BulkTrackInserter::new(db);
    for track in chart {
        inserter.insert_track(&track).await?;
    }
    Ok(())
}
