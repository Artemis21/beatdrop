//! A minimal wrapper for the parts of the Deezer API we care about.
//! API documentation: <https://developers.deezer.com/api>
use lazy_static::lazy_static;
use reqwest::Error;
use serde::{Serialize, Deserialize};

/// Base URL for the API.
const API_URL: &str = "https://api.deezer.com";

lazy_static! {
    /// A shared HTTP client for making requests to the API.
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

/// Fetch the "chart" (a list of popular tracks) for a given genre.
/// The genre with ID 0 is "all genres".
pub async fn chart(genre_id: Id) -> Result<Vec<Track>, Error> {
    let url = format!("{API_URL}/chart/{id}/tracks", id = genre_id);
    let data: DataWrap<_> = CLIENT.get(&url).send().await?.json().await?;
    Ok(data.data)
}

/// Get a list of common genres.
/// There does not seem to be a way to get a list of all genres, short enumerating
/// all possible genre IDs.
pub async fn genres() -> Result<Vec<Genre>, Error> {
    let url = format!("{API_URL}/genre", API_URL = API_URL);
    let data: DataWrap<_> = CLIENT.get(&url).send().await?.json().await?;
    Ok(data.data)
}

/// Get an album by its ID.
///
/// # Errors
///
/// Returns an error if the album does not exist, or if the API request fails
/// for another reason.
pub async fn album(album_id: Id) -> Result<Album, Error> {
    let url = format!("{API_URL}/album/{id}", id = album_id);
    let data: DataWrap<_> = CLIENT.get(&url).send().await?.json().await?;
    Ok(data.data)
}

/// A helper for serde deserialisation of API responses which are wrapped in
/// an object with a single `data` field.
#[derive(Debug, Deserialize)]
struct DataWrap<T> {
    /// The actual data.
    data: T,
}

/// An artist object returned by the API.
#[derive(Debug, Deserialize)]
pub struct Artist {
    /// Deezer ID
    pub id: Id,
    /// Artist name
    pub name: String,
    /// Link to the artist page on Deezer
    pub link: String,
    /// URL of an image of the artist
    pub picture: String,
}

/// A genre object returned by the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct Genre {
    /// Deezer ID
    pub id: Id,
    /// Genre name
    pub name: String,
    /// URL of an image representing the genre
    pub picture: String,
}

/// A track object returned by the API.
#[derive(Debug, Deserialize)]
pub struct Track {
    /// Deezer ID
    pub id: Id,
    /// Track title
    pub title: String,
    /// A track ranking (seems to be in the range 10,000 to 100,000)
    pub rank: i32,
    /// Link to the track on Deezer
    pub link: String,
    /// URL of a 30 second preview MP3
    pub preview: String,
    /// Artist object
    pub artist: Artist,
    /// Album object (missing some fields)
    pub album: PartialAlbum,
}

/// A partial album object returned by the API as part of a track object.
///
/// You can use the [`album`] function to get a full album object.
#[derive(Debug, Deserialize)]
pub struct PartialAlbum {
    /// Deezer ID
    pub id: Id,
    /// Album title
    pub title: String,
    /// URL of a cover art image for the album
    pub cover: String,
}

/// A full album object returned by the API.
#[derive(Debug, Deserialize)]
pub struct Album {
    /// Deezer ID
    pub id: Id,
    /// Album title
    pub title: String,
    /// URL of a cover art image for the album
    pub cover: String,
    /// Link to the album on Deezer
    pub link: String,
    /// A list of genres associated with the album
    pub genres: Vec<Genre>,
}

/// A wrapper around a Deezer ID.
///
/// This is mainly useful to handle conversions to and from the database representation,
/// which uses `i32` instead of `u32` (because SQL doesn't have unsigned integers).
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Id(pub u32);
