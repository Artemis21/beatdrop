//! Tools for working with the music data in the database.
use serde::Serialize;

use crate::{deezer, DbConn, Error, ResultExt};

mod bulk_insert;
mod insert;
mod music;
pub mod pick;

pub use music::init;

/// Get a genre object from the database by ID.
pub async fn genre(db: &mut DbConn, id: deezer::Id) -> Result<deezer::Genre, Error> {
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
) -> Result<Vec<u8>, Error> {
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

/// Track metadata returned as part of game objects in the API.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    /// The track's Deezer ID
    id: deezer::Id,
    /// Track title
    title: String,
    /// Link to the track on Deezer
    link: String,
    /// Artist name
    artist_name: String,
    /// Album title
    album_title: String,
    /// URL of an image of the album cover art
    album_cover: String,
}

/// Check if the given track exists, and if it is, make sure it is stored in the database.
pub async fn exists(db: &mut DbConn, id: deezer::Id) -> Result<bool, Error> {
    let exists_in_database =
        sqlx::query_scalar!("SELECT 1 FROM track WHERE track.id = $1", i32::from(id))
            .fetch_optional(&mut *db)
            .await
            .wrap_err("error checking if track exists")?
            .is_some();
    if exists_in_database {
        return Ok(true);
    }
    match deezer::track(id).await? {
        Some(track) => {
            insert::track_with_refs(db, &track)
                .await
                .wrap_err("inserting track into database with references")?;
            Ok(true)
        }
        None => Ok(false),
    }
}

impl Meta {
    /// Get track metadata from the database by ID.
    pub async fn get(db: &mut DbConn, id: deezer::Id) -> Result<Self, Error> {
        let track = sqlx::query_as!(
            Self,
            "SELECT
                track.id,
                track.title,
                track.deezer_url AS link,
                artist.title AS artist_name,
                album.title AS album_title,
                album.cover_art_url AS album_cover
            FROM track
            INNER JOIN artist ON track.artist_id = artist.id
            INNER JOIN album ON track.album_id = album.id
            WHERE track.id = $1",
            i32::from(id),
        )
        .fetch_one(db)
        .await
        .wrap_err("error querying track metadata")?;
        Ok(track)
    }
}

impl From<deezer::Track> for Meta {
    fn from(track: deezer::Track) -> Self {
        Self {
            id: track.id,
            title: track.title,
            link: track.link,
            artist_name: track.artist.name,
            album_title: track.album.title,
            album_cover: track.album.cover,
        }
    }
}
