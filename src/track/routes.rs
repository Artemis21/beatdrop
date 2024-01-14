//! API routes for track resources.
use super::Meta;
use crate::{deezer, ApiError};
use rocket::{get, routes, serde::json::Json};
use serde::Serialize;

/// Collect API routes for track resources.
pub fn routes() -> Vec<rocket::Route> {
    routes![search_track]
}

/// JSON response to a track search query.
#[derive(Default, Serialize)]
struct SearchResults {
    /// The tracks that were found.
    tracks: Vec<Meta>,
}

/// Search for a track by name.
#[get("/tracks?<q>")]
async fn search_track(q: &str) -> Result<Json<SearchResults>, ApiError> {
    if q.is_empty() {
        return Ok(Json(SearchResults::default()));
    }
    let mut tracks = deezer::track_search(q).await?;
    tracks.sort_by_key(|track| std::cmp::Reverse(track.rank));
    let meta = tracks.into_iter().take(5).map(From::from).collect();
    Ok(Json(SearchResults { tracks: meta }))
}
