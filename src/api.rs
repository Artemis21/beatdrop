//! Logic for handling calls to the API routes.
use crate::{deezer, game, track, AuthConfig, Db, Error, Game, User};
use rocket::{
    delete, get, http::ContentType, patch, post, routes, serde::json::Json, Route, State,
};
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
async fn new_account(mut db: Db) -> Result<Json<NewAccount>, Error> {
    let (_user, secret) = User::create(&mut db).await?;
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
    mut db: Db,
    auth: &State<AuthConfig>,
    body: Json<SecretLogin>,
) -> Result<Json<Session>, Error> {
    let user = User::from_login(&body.login, &mut db).await?;
    let session = user.session_token(auth);
    Ok(Json(Session { session }))
}

/// Get information on the authenticated user's account.
#[get("/account/me")]
const fn get_account(user: User) -> Json<User> {
    Json(user)
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
    mut db: Db,
    mut user: User,
    body: Json<UpdateAccount>,
) -> Result<Json<User>, Error> {
    if let Some(display_name) = &body.display_name {
        user = user.set_display_name(Some(display_name), &mut db).await?;
    }
    Ok(Json(user))
}

/// Delete the authenticated user's account.
#[delete("/account/me")]
async fn delete_account(mut db: Db, user: User) -> Result<(), Error> {
    user.delete(&mut db).await?;
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
    mut db: Db,
    user: User,
    body: Json<NewGame>,
) -> Result<Json<game::Response>, Error> {
    let last_game = user.current_game(&mut db).await?;
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
        if user.has_played_daily(&mut db).await? {
            return Err(Error::InvalidForm(
                "user has already played the daily game today",
            ));
        }
        track::pick::daily(&mut db).await?
    } else if let Some(genre_id) = body.genre_id {
        track::pick::genre(&mut db, genre_id).await?
    } else {
        track::pick::any(&mut db).await?
    };
    let game = Game::create(
        &mut db,
        user.id,
        body.genre_id,
        body.daily,
        body.timed,
        track_id,
    )
    .await?;
    Ok(Json(game.into_response(&mut db).await?))
}

/// Get the authenticated user's active or most recent game.
#[get("/game")]
async fn get_active_game(
    mut db: Db,
    game: game::Maybe,
) -> Result<Json<Option<game::Response>>, Error> {
    Ok(Json(game.into_response(&mut db).await?))
}

/// Get the authenticated user's current daily game, if any.
#[get("/game/daily")]
async fn get_daily_game(mut db: Db, user: User) -> Result<Json<Option<game::Response>>, Error> {
    match user.daily_game(&mut db).await? {
        Some(game) => Ok(Json(Some(game.into_response(&mut db).await?))),
        None => Ok(Json(None)),
    }
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
    mut db: Db,
    mut game: Game,
    body: Json<NewGuess>,
) -> Result<Json<game::Response>, Error> {
    if !game.is_over() {
        game.new_guess(&mut db, body.track_id).await?;
        game.end_if_over(&mut db).await?;
    }
    Ok(Json(game.into_response(&mut db).await?))
}

/// Get the music clip a user is allowed to listen to for a game.
#[get("/game/clip?<seek>")]
async fn get_clip(
    mut db: Db,
    track_config: &State<track::Config>,
    game: Game,
    seek: Option<u32>,
) -> Result<(ContentType, Vec<u8>), Error> {
    let end = game.time_unlocked();
    let start = chrono::Duration::milliseconds(seek.unwrap_or(0).into());
    if start >= end {
        return Err(Error::InvalidForm("cannot seek past end of unlocked music"));
    }
    track::clip(&mut db, track_config, game.track_id, start..end)
        .await
        .map(|bytes| (ContentType::new("audio", "wav"), bytes))
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
    let tracks = deezer::track_search(q).await?;
    let meta = tracks.into_iter().map(From::from).collect();
    Ok(Json(SearchResults { tracks: meta }))
}
