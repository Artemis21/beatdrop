//! Efficiently insert multiple tracks into the database.
use super::insert;
use std::collections::HashSet;

use crate::{deezer, DbConn};
use eyre::{Context, Result};

/// A helper for efficiently inserting multiple tracks into the database.
pub struct BulkInserter<'a> {
    /// The database connection.
    db: &'a mut DbConn,
    /// The IDs of all genres that have already been inserted during this operation.
    inserted_genre_ids: HashSet<deezer::Id>,
    /// The IDs of all albums that have already been inserted during this operation.
    inserted_album_ids: HashSet<deezer::Id>,
    /// The IDs of all artists that have already been inserted during this operation.
    inserted_artist_ids: HashSet<deezer::Id>,
    /// The IDs of all tracks that have already been inserted during this operation.
    inserted_track_ids: HashSet<deezer::Id>,
}

impl<'a> BulkInserter<'a> {
    /// Create a new bulk inserter.
    pub fn new(db: &'a mut DbConn) -> Self {
        Self {
            db,
            inserted_genre_ids: HashSet::new(),
            inserted_album_ids: HashSet::new(),
            inserted_artist_ids: HashSet::new(),
            inserted_track_ids: HashSet::new(),
        }
    }

    /// Insert a track into the database, or update it if it already exists.
    pub async fn insert_track(&mut self, track: &deezer::Track) -> Result<()> {
        if self.inserted_track_ids.contains(&track.id) {
            return Ok(());
        }
        self.insert_album(&track.album).await?;
        self.insert_artist(&track.artist).await?;
        insert::track(&mut *self.db, track).await?;
        self.inserted_track_ids.insert(track.id);
        Ok(())
    }

    /// Insert an album into the database.
    ///
    /// If the album is not already in the database, the full album will be
    /// fetched from the Deezer API and inserted.
    ///
    /// If the album is already in the database, nothing will be done. This is
    /// to minimise calls to the Deezer API.
    async fn insert_album(&mut self, album: &deezer::PartialAlbum) -> Result<()> {
        if self.album_exists(album.id).await? {
            return Ok(());
        }
        let album = deezer::album(album.id).await?;
        insert::album(&mut *self.db, &album).await?;
        for genre in &*album.genres {
            self.insert_genre(genre).await?;
            insert::album_genre(&mut *self.db, album.id, genre.id).await?;
        }
        self.inserted_album_ids.insert(album.id);
        Ok(())
    }

    /// Check if an album exists in the database.
    async fn album_exists(&mut self, album_id: deezer::Id) -> Result<bool> {
        if self.inserted_album_ids.contains(&album_id) {
            return Ok(true);
        }
        let exists = sqlx::query_scalar!("SELECT 1 FROM album WHERE id = $1", i32::from(album_id),)
            .fetch_optional(&mut *self.db)
            .await
            .wrap_err("error querying if album exists")?;
        Ok(exists.is_some())
    }

    /// Insert a genre object into the database, or update it if it already exists.
    async fn insert_genre(&mut self, genre: &deezer::Genre) -> Result<()> {
        if self.inserted_genre_ids.contains(&genre.id) {
            return Ok(());
        }
        insert::genre(&mut *self.db, genre).await?;
        self.inserted_genre_ids.insert(genre.id);
        Ok(())
    }

    /// Insert an artist object into the database, or update it if it already exists.
    async fn insert_artist(&mut self, artist: &deezer::Artist) -> Result<()> {
        if self.inserted_artist_ids.contains(&artist.id) {
            return Ok(());
        }
        insert::artist(&mut *self.db, artist).await?;
        self.inserted_artist_ids.insert(artist.id);
        Ok(())
    }
}
