//! Get track metadata to return in the API.
use crate::{deezer, DbConn};
use eyre::{Context, Result};
use serde::Serialize;

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

impl Meta {
    /// Get track metadata from the database by ID.
    pub async fn get(db: &mut DbConn, id: deezer::Id) -> Result<Self> {
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
