//! Tools for working with the music data in the database.
use serde::Serialize;

use crate::{DbConn, Error, deezer};

pub mod pick;
mod insert;
mod bulk_insert;
mod music;

pub use music::Config;

/// Get a genre object from the database by ID.
pub async fn genre(db: &mut DbConn, id: i32) -> Result<deezer::Genre, Error> {
    let genre = sqlx::query_as!(
        deezer::Genre,
        "SELECT id, title AS name, picture_url AS picture FROM genre
        WHERE id = $1",
        id,
    ).fetch_one(db).await?;
    Ok(genre)
}

/// Get a clip of music from a track.
pub async fn clip(
    db: &mut DbConn,
    config: &Config,
    track_id: i32,
    seconds: std::ops::Range<u32>,
) -> Result<Vec<u8>, Error> {
    let preview_url = sqlx::query_scalar!(
        "SELECT preview_url FROM track WHERE id = $1",
        track_id,
    ).fetch_one(db).await?;
    music::clip(config, track_id, &preview_url, seconds).await
}

/// Track metadata returned as part of game objects in the API.
#[derive(Serialize)]
pub struct Meta {
    /// The track's Deezer ID
    id: i32,
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

impl Meta {
    /// Get track metadata from the database by ID.
    pub async fn get(db: &mut DbConn, id: i32) -> Result<Self, Error> {
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
            id,
        )
        .fetch_one(db)
        .await?;
        Ok(track)
    }
}
