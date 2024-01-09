//! Logic for handling calls to the API routes.
use crate::{
    database::{Connection, Transaction},
    deezer, game, track, AuthHeader, Error, Game, User,
};
use rocket::{delete, get, http::ContentType, patch, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};

/// Collect all the API endpoints.
#[allow(clippy::no_effect_underscore_binding)] // triggered by macro
pub fn routes() -> Vec<Route> {
    routes![
        new_account,
        login_secret,
        get_account,
        update_account,
        delete_account,
        new_game,
        get_active_game,
        get_daily_game,
        new_guess,
        get_clip,
        search_track,
    ]
}

/// The response body for a newly created account.
#[derive(Serialize)]
struct NewAccount {
    /// The secret login token for the new account.
    login: String,
}

/// Create a new account.
#[post("/account/me")]
async fn new_account(mut conn: Connection) -> Result<Json<NewAccount>, Error> {
    let (_user, secret) = User::create(&mut conn).await?;
    Ok(Json(NewAccount { login: secret }))
}

/// The request body for creating a session using a secret login token.
#[derive(Deserialize)]
struct SecretLogin {
    /// The secret login token.
    login: String,
}

/// The response body for a newly created session.
#[derive(Serialize)]
struct Session {
    /// The session token.
    session: String,
}

/// Create a new session using a secret login token.
#[post("/session/secret", data = "<body>")]
async fn login_secret(
    mut conn: Connection,
    body: Json<SecretLogin>,
) -> Result<Json<Session>, Error> {
    let user = User::from_login(&body.login, &mut conn).await?;
    let session = user.session_token();
    Ok(Json(Session { session }))
}

/// Get information on the authenticated user's account.
#[get("/account/me")]
async fn get_account(mut conn: Connection, auth: AuthHeader<'_>) -> Result<Json<User>, Error> {
    User::from_session(auth, &mut conn).await.map(Json)
}

/// The request body for updating an account.
#[derive(Deserialize)]
struct UpdateAccount {
    /// The new display name for the account, or `null` to keep the current one.
    display_name: Option<String>,
}

/// Update the authenticated user's account.
#[patch("/account/me", data = "<body>")]
async fn update_account(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
    body: Json<UpdateAccount>,
) -> Result<Json<User>, Error> {
    let mut user = User::from_session(auth, &mut tx).await?;
    if let Some(display_name) = &body.display_name {
        user = user.set_display_name(Some(display_name), &mut tx).await?;
    }
    tx.commit().await?;
    Ok(Json(user))
}

/// Delete the authenticated user's account.
#[delete("/account/me")]
async fn delete_account(mut tx: Transaction<'_>, auth: AuthHeader<'_>) -> Result<(), Error> {
    let user = User::from_session(auth, &mut tx).await?;
    user.delete(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

/// The request body for creating a new game.
#[derive(Deserialize)]
struct NewGame {
    /// The genre ID to restrict the game to, or `null` to allow any genre.
    genre_id: Option<deezer::Id>,
    /// Whether the game is a daily game. If it is, `genre_id` and `timed` must
    /// be `null` and `false` respectively.
    #[serde(default)]
    daily: bool,
    /// Whether the game is to be in timed mode.
    #[serde(default)]
    timed: bool,
}

/// Begin a new game for the authenticated user.
#[post("/game", data = "<body>")]
async fn new_game(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
    body: Json<NewGame>,
) -> Result<Json<game::Response>, Error> {
    let user = User::from_session(auth, &mut tx).await?;
    let last_game = user.current_game(&mut tx).await?;
    if let Some(last_game) = last_game {
        if !last_game.is_over() {
            return Err(Error::InvalidForm("user still has an ongoing game"));
        }
    }
    let track_id = if body.daily {
        if body.genre_id.is_some() || body.timed {
            return Err(Error::InvalidForm(
                "daily games cannot be timed or have a genre",
            ));
        }
        if user.has_played_daily(&mut tx).await? {
            return Err(Error::InvalidForm(
                "user has already played the daily game today",
            ));
        }
        track::pick::daily(&mut tx).await?
    } else if let Some(genre_id) = body.genre_id {
        track::pick::genre(&mut tx, genre_id).await?
    } else {
        track::pick::any(&mut tx).await?
    };
    let game = Game::create(
        &mut tx,
        user.id,
        body.genre_id,
        body.daily,
        body.timed,
        track_id,
    )
    .await?;
    let game = game.into_response(&mut tx).await?;
    tx.commit().await?;
    Ok(Json(game))
}

/// Get the authenticated user's active or most recent game.
#[get("/game")]
async fn get_active_game(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
) -> Result<Json<Option<game::Response>>, Error> {
    let user = User::from_session(auth, &mut tx).await?;
    let game = match user.current_game(&mut tx).await? {
        Some(game) => Some(game.into_response(&mut tx).await?),
        None => None,
    };
    Ok(Json(game))
}

/// Get the authenticated user's current daily game, if any.
#[get("/game/daily")]
async fn get_daily_game(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
) -> Result<Json<Option<game::Response>>, Error> {
    let user = User::from_session(auth, &mut tx).await?;
    let game = match user.daily_game(&mut tx).await? {
        Some(game) => Some(game.into_response(&mut tx).await?),
        None => None,
    };
    Ok(Json(game))
}

/// The request body for making a guess.
#[derive(Deserialize)]
struct NewGuess {
    /// The ID of the track to guess, or `null` to skip.
    track_id: Option<deezer::Id>,
}

/// Submit a guess for the authenticated user's active game.
///
/// Does nothing if the game is already over.
#[post("/game/guess", data = "<body>")]
async fn new_guess(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
    body: Json<NewGuess>,
) -> Result<Json<game::Response>, Error> {
    if let Some(track_id) = body.track_id {
        if !track::exists(&mut tx, track_id).await? {
            return Err(Error::NotFound("given track ID does not exist"));
        }
    }
    let user = User::from_session(auth, &mut tx).await?;
    let mut game = user.expect_current_game(&mut tx).await?;
    if !game.is_over() {
        game.new_guess(&mut tx, body.track_id).await?;
        game.end_if_over(&mut tx).await?;
    }
    let game = game.into_response(&mut tx).await?;
    tx.commit().await?;
    Ok(Json(game))
}

/// Get the music clip a user is allowed to listen to for a game.
#[get("/game/clip?<seek>")]
async fn get_clip(
    mut tx: Transaction<'_>,
    auth: AuthHeader<'_>,
    seek: Option<u32>,
) -> Result<(ContentType, Vec<u8>), Error> {
    let user = User::from_session(auth, &mut tx).await?;
    let game = user.expect_current_game(&mut tx).await?;
    let end = game.time_unlocked();
    let start = chrono::Duration::milliseconds(seek.unwrap_or(0).into());
    if start >= end {
        return Err(Error::InvalidForm("cannot seek past end of unlocked music"));
    }
    let bytes = track::clip(&mut tx, game.track_id, start..end).await?;
    Ok((ContentType::new("audio", "wav"), bytes))
}

/// JSON response to a track search query.
#[derive(Default, Serialize)]
struct SearchResults {
    /// The tracks that were found.
    tracks: Vec<track::Meta>,
}

/// Search for a track by name.
#[get("/track/search?<q>")]
async fn search_track(q: &str) -> Result<Json<SearchResults>, Error> {
    if q.is_empty() {
        return Ok(Json(SearchResults::default()));
    }
    let mut tracks = deezer::track_search(q).await?;
    tracks.sort_by_key(|track| std::cmp::Reverse(track.rank));
    let meta = tracks.into_iter().take(5).map(From::from).collect();
    Ok(Json(SearchResults { tracks: meta }))
}
