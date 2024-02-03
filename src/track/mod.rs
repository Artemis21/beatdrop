//! Tools for working with the music data in the database.
use crate::{deezer, DbConn};
use eyre::{Context, Result};

mod bulk_insert;
mod insert;
mod meta;
mod music;
pub mod pick;
mod routes;
mod similar;

pub use meta::Meta;
pub use music::init;
pub use routes::routes;
pub use similar::similar;

/// Get a genre object from the database by ID.
pub async fn genre(db: &mut DbConn, id: deezer::Id) -> Result<deezer::Genre> {
    let genre = sqlx::query_as!(
        deezer::Genre,
        "SELECT id, title AS name, picture_url AS picture FROM genre
        WHERE id = $1",
        i32::from(id),
    )
    .fetch_one(db)
    .await
    .wrap_err("error querying genre")?;
    Ok(genre)
}

/// Get a clip of music from a track.
pub async fn clip(
    db: &mut DbConn,
    track_id: deezer::Id,
    time: std::ops::Range<chrono::Duration>,
) -> Result<Vec<u8>> {
    let preview_url = sqlx::query_scalar!(
        "SELECT preview_url FROM track WHERE id = $1",
        i32::from(track_id),
    )
    .fetch_one(db)
    .await
    .wrap_err("error querying track preview URL")?;
    music::clip(track_id.0, &preview_url, time)
        .await
        .wrap_err("error clipping music")
}

/// Get the given track from the database, or fetch it from Deezer if it's not there.
pub async fn get_or_fetch(db: &mut DbConn, id: deezer::Id) -> Result<Option<Meta>> {
    if let Some(track) = Meta::try_get(db, id).await? {
        return Ok(Some(track));
    }
    match deezer::track(id).await? {
        Some(track) => {
            insert::track_with_refs(db, &track)
                .await
                .wrap_err("inserting track into database with references")?;
            Ok(Some(track.into()))
        }
        None => Ok(None),
    }
}
