//! Database queries for inserting and updating music data in the database.
use crate::{
    deezer::{Album, Artist, Genre, Track, self},
    DbConn, Error, ResultExt,
};

/// Insert a track into the database, or update it if it already exists.
///
/// The referenced album and artist must already exist in the database.
pub async fn track(db: &mut DbConn, track: &Track) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO track (id, title, deezer_url, preview_url, deezer_rank, album_id, artist_id) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            deezer_url = EXCLUDED.deezer_url,
            preview_url = EXCLUDED.preview_url,
            deezer_rank = EXCLUDED.deezer_rank,
            album_id = EXCLUDED.album_id,
            artist_id = EXCLUDED.artist_id",
        i32::from(track.id),
        track.title,
        track.link,
        track.preview,
        track.rank,
        i32::from(track.album.id),
        i32::from(track.artist.id),
    ).execute(db).await.wrap_err("error inserting track")?;
    Ok(())
}

/// Insert an album into the database, or update it if it already exists.
pub async fn album(db: &mut DbConn, album: &Album) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO album (id, title, deezer_url, cover_art_url) VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            deezer_url = EXCLUDED.deezer_url,
            cover_art_url = EXCLUDED.cover_art_url",
        i32::from(album.id),
        album.title,
        album.link,
        album.cover,
    )
    .execute(db)
    .await.wrap_err("error inserting album")?;
    Ok(())
}

/// Insert a genre object into the database, or update it if it already exists.
pub async fn genre(db: &mut DbConn, genre: &Genre) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO genre (id, title, picture_url) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            picture_url = EXCLUDED.picture_url",
        i32::from(genre.id),
        genre.name,
        genre.picture,
    )
    .execute(db)
    .await.wrap_err("error inserting genre")?;
    Ok(())
}

/// Insert a many-to-many relationship between an album and a genre.
///
/// The referenced album and genre must already exist in the database.
///
/// If the relationship already exists, nothing will be done.
pub async fn album_genre(db: &mut DbConn, album_id: deezer::Id, genre_id: deezer::Id) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO album_genre (album_id, genre_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        i32::from(album_id),
        i32::from(genre_id),
    )
    .execute(db)
    .await.wrap_err("error inserting album-genre relationship")?;
    Ok(())
}

/// Insert an artist into the database, or update it if it already exists.
pub async fn artist(db: &mut DbConn, artist: &Artist) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO artist (id, title, deezer_url, picture_url) VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            deezer_url = EXCLUDED.deezer_url,
            picture_url = EXCLUDED.picture_url",
        i32::from(artist.id),
        artist.name,
        artist.link,
        artist.picture,
    )
    .execute(db)
    .await.wrap_err("error inserting artist")?;
    Ok(())
}
