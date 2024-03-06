//! A minimal wrapper for the parts of the Deezer API we care about.
//! API documentation: <https://developers.deezer.com/api>
use eyre::{eyre, Context, Result};
use lazy_static::lazy_static;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
// We don't actually use Rocket here, but Reqwest also uses this `Bytes` type and doesn't re-export it.
use crate::ratelimit::{Backoff, Ratelimit};
use rocket::http::hyper::body::Bytes;

/// Base URL for the API.
const API_URL: &str = "https://api.deezer.com";

lazy_static! {
    /// A shared HTTP client for making requests to the API.
    static ref CLIENT: reqwest::Client = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT_LANGUAGE, "en".parse().unwrap());
        reqwest::Client::builder().default_headers(headers).build().unwrap()
    };

    /// A rate limiter for the Deezer API.
    ///
    /// Deezer currently allows 50 requests per 5 seconds, and this configuration
    /// should align with that.
    static ref RATELIMIT: Ratelimit = Ratelimit::new(50, std::time::Duration::from_millis(100));
}

/// Make a request to the Deezer API, respecting the rate limit and retrying
/// if we hit it or the service is busy, with exponential backoff.
async fn send_request<T: DeserializeOwned + Send>(req: RequestBuilder) -> Result<Option<T>> {
    let backoff = Backoff::new(
        std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(300),
        2,
    );
    for delay in backoff {
        RATELIMIT.wait().await;
        let response = req
            .try_clone()
            .expect("reqwest request cloning should not fail")
            .send()
            .await
            .wrap_err("error sending request to Deezer")?
            .error_for_status()
            .wrap_err("Deezer API returned an HTTP error")?
            .json()
            .await
            .wrap_err("error deserialising Deezer API response")?;
        match response {
            Response::Data(data) => return Ok(Some(data)),
            Response::Error { error } => match error.code {
                ErrorCode::Ratelimited | ErrorCode::ServiceBusy => {
                    eprintln!(
                        "Deezer API returned a temporary error, retrying in {}s: {error}",
                        delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
                ErrorCode::NotFound => return Ok(None),
                ErrorCode::Unknown(_) => {
                    return Err(eyre!("Deezer API returned an unknown error: {error}"))
                }
            },
        }
    }
    unreachable!("backoff iterator should never end")
}

/// An extension trait allowing for the use of a custom send method on
/// [`RequestBuilder`] with postfix syntax.
#[rocket::async_trait]
trait RequestBuilderExt {
    /// Send a request to the Deezer API, returning `None` if the resource was not found.
    async fn try_deezer_fetch<T: DeserializeOwned + Send>(self) -> Result<Option<T>>;

    /// Send a request to the Deezer API, returning an error if the resource was not found.
    async fn deezer_fetch<T: DeserializeOwned + Send>(self) -> Result<T>;
}

#[rocket::async_trait]
impl RequestBuilderExt for RequestBuilder {
    async fn try_deezer_fetch<T: DeserializeOwned + Send>(self) -> Result<Option<T>> {
        send_request(self).await
    }

    async fn deezer_fetch<T: DeserializeOwned + Send>(self) -> Result<T> {
        send_request(self)
            .await
            .and_then(|data| data.ok_or_else(|| eyre!("requested resource not found")))
    }
}

/// Fetch the "chart" (a list of popular tracks) for a given genre.
/// The genre with ID 0 is "all genres".
pub async fn chart(genre_id: Id) -> Result<Vec<Track>> {
    let url = format!("{API_URL}/chart/{genre_id}/tracks");
    let data: DataWrap<_> = CLIENT
        .get(&url)
        .deezer_fetch()
        .await
        .wrap_err("error fetching genre chart")?;
    Ok(data.data)
}

/// Genres we don't want to show.
const GENRE_BLACKLIST: [u32; 2] = [
    0,   // All
    457, // Audiobooks
];

/// Get a list of common genres.
/// There does not seem to be a way to get a list of all genres, short of enumerating
/// all possible genre IDs.
pub async fn genres() -> Result<Vec<Genre>> {
    let url = format!("{API_URL}/genre");
    let genres = CLIENT
        .get(&url)
        .deezer_fetch::<DataWrap<Vec<Genre>>>()
        .await
        .wrap_err("error fetching genre list")?
        .data
        .into_iter()
        .filter(|genre| !GENRE_BLACKLIST.contains(&genre.id))
        .collect();
    Ok(genres)
}

/// Get an album by its ID.
///
/// # Errors
///
/// Returns an error if the album does not exist, or if the API request fails
/// for another reason.
pub async fn album(album_id: Id) -> Result<Album> {
    let url = format!("{API_URL}/album/{album_id}");
    CLIENT
        .get(&url)
        .deezer_fetch()
        .await
        .wrap_err("error fetching album")
}

/// Search for a track by name or artist.
pub async fn track_search(q: &str) -> Result<Vec<Track>> {
    let url = format!("{API_URL}/search/track");
    let data: DataWrap<_> = CLIENT
        .get(&url)
        .query(&[("q", q)])
        .deezer_fetch()
        .await
        .wrap_err("error searching tracks")?;
    Ok(data.data)
}

/// Fetch a track by ID, returning None if we could not deserialise the response (eg. not found).
pub async fn track(id: Id) -> Result<Option<Track>> {
    let url = format!("{API_URL}/track/{id}");
    CLIENT
        .get(&url)
        .try_deezer_fetch()
        .await
        .wrap_err("error fetching track")
}

/// Download a track from Deezer and save it to the music cache.
pub async fn track_preview(preview_url: &str) -> Result<Bytes> {
    // This isn't an API request so hopefully should be fine without ratelimiting.
    CLIENT
        .get(preview_url)
        .send()
        .await
        .wrap_err("error downloading a track preview")?
        .bytes()
        .await
        .wrap_err("error reading the response for a track preview download")
}

/// A helper for serde deserialisation of API responses which are wrapped in
/// an object with a single `data` field.
#[derive(Debug, Deserialize)]
pub struct DataWrap<T> {
    /// The actual data.
    data: T,
}

impl<T> std::ops::Deref for DataWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// A response from Deezer which may be an error.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    /// A successful response.
    Data(T),
    /// An error response.
    Error {
        /// The error object.
        error: Error,
    },
}

/// An error returned by the API.
#[derive(Debug, Deserialize)]
pub struct Error {
    /// The error "type".
    #[serde(rename = "type")]
    kind: String,
    /// The error message.
    message: String,
    /// The error code.
    code: ErrorCode,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({:?}): {}", self.kind, self.code, self.message)
    }
}

/// An error code from an API error.
#[derive(Debug, Deserialize)]
pub enum ErrorCode {
    /// We have exceeded the request quota.
    Ratelimited,
    /// The service is busy and we should try again later.
    ServiceBusy,
    /// The requested resource was not found.
    NotFound,
    /// Other error codes exist, but we treat them all as unresolvable issues.
    Unknown(u32),
}

impl From<u32> for ErrorCode {
    fn from(code: u32) -> Self {
        // https://developers.deezer.com/api/errors
        match code {
            4 => Self::Ratelimited,
            700 => Self::ServiceBusy,
            800 => Self::NotFound,
            code => Self::Unknown(code),
        }
    }
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
    pub genres: DataWrap<Vec<Genre>>,
}

/// A wrapper around a Deezer ID.
///
/// This is mainly useful to handle conversions to and from the database representation,
/// which uses `i32` instead of `u32` (because SQL doesn't have unsigned integers).
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Id(pub u32);

impl std::ops::Deref for Id {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A wrapper around `Option<deezer::Id>`, used for deserialising database rows with sqlx.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct OptionId(Option<Id>);

impl From<Option<i32>> for OptionId {
    fn from(id: Option<i32>) -> Self {
        id.map_or(Self(None), |id| Self(Some(id.into())))
    }
}

impl std::ops::Deref for OptionId {
    type Target = Option<Id>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
