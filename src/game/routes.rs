//! API routes for managing games.
use crate::{deezer, game, track, ApiError, DbConn, Game, Session, Transaction};
use rocket::{get, http::ContentType, post, routes, serde::json::Json};
use serde::{Deserialize, Serialize};

/// Collect API routes for managing games.
pub fn routes() -> Vec<rocket::Route> {
    routes![new_game, recent_games, get_game, new_guess, get_clip,]
}

impl Session {
    /// Get a game belonging to the authenticated user, by ID.
    pub async fn game(self, db: &mut DbConn, game_id: i32) -> Result<Game, ApiError> {
        let user = self.user(db).await?;
        match Game::get(db, game_id).await? {
            Some(game) if game.account_id == user.id => Ok(game),
            Some(_) => Err(ApiError::forbidden("this game belongs to another user")),
            _ => Err(ApiError::not_found("no such game")),
        }
    }
}

/// Response body containing IDs of recent games
#[derive(Serialize)]
struct RecentGames {
    /// The ID of today's daily game, if it has been started.
    daily: Option<i32>,
    /// The ID of the user's ongoing game, if any.
    ongoing: Option<i32>,
}

/// Get the ID of the daily game and most recent game for the authenticated user.
#[get("/games")]
async fn recent_games(
    mut tx: Transaction<'_>,
    auth: Session,
) -> Result<Json<RecentGames>, ApiError> {
    let user = auth.user(&mut tx).await?;
    let daily = user.daily_game_id(&mut tx).await?;
    let ongoing = user.ongoing_game_id(&mut tx).await?;
    Ok(Json(RecentGames { daily, ongoing }))
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
#[post("/games", data = "<body>")]
async fn new_game(
    mut tx: Transaction<'_>,
    auth: Session,
    body: Json<NewGame>,
) -> Result<Json<game::Response>, ApiError> {
    let user = auth.user(&mut tx).await?;
    if user.ongoing_game_id(&mut tx).await?.is_some() {
        return Err(ApiError::conflict("user already has an ongoing game"));
    }
    let track_id = if body.daily {
        if body.genre_id.is_some() || body.timed {
            return Err(ApiError::bad_request(
                "daily games cannot be timed or have a genre",
            ));
        }
        if user.daily_game_id(&mut tx).await?.is_some() {
            return Err(ApiError::conflict(
                "user has already started the daily game today",
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

/// Get a game by ID. It must belong to the authenticated user.
#[get("/games/<id>")]
async fn get_game(
    mut tx: Transaction<'_>,
    auth: Session,
    id: i32,
) -> Result<Json<game::Response>, ApiError> {
    let game = auth.game(&mut tx, id).await?;
    Ok(Json(game.into_response(&mut tx).await?))
}

/// The request body for making a guess.
#[derive(Deserialize)]
struct NewGuess {
    /// The ID of the track to guess, or `null` to skip.
    track_id: Option<deezer::Id>,
}

/// Submit a guess for the authenticated user's active game.
#[post("/games/<id>/guesses", data = "<body>")]
async fn new_guess(
    mut tx: Transaction<'_>,
    auth: Session,
    id: i32,
    body: Json<NewGuess>,
) -> Result<Json<game::Response>, ApiError> {
    let mut game = auth.game(&mut tx, id).await?;
    if game.is_over() {
        return Err(ApiError::conflict("game is already over"));
    }
    let track_id = match body.track_id {
        Some(track_id) => match track::get_or_fetch(&mut tx, track_id).await? {
            None => return Err(ApiError::not_found("given track ID does not exist")),
            Some(guess) => {
                let answer = game.track(&mut tx).await?;
                if track::similar(&guess.title, &answer.title) {
                    Some(answer.id)
                } else {
                    Some(guess.id)
                }
            }
        },
        None => None,
    };
    game.new_guess(&mut tx, track_id, None).await?;
    game.auto_update(&mut tx).await?;
    let game = game.into_response(&mut tx).await?;
    tx.commit().await?;
    Ok(Json(game))
}

/// Get the music clip a user is allowed to listen to for a game.
#[get("/games/<id>/clip?<seek>")]
async fn get_clip(
    mut tx: Transaction<'_>,
    auth: Session,
    id: i32,
    seek: Option<u32>,
) -> Result<(ContentType, Vec<u8>), ApiError> {
    let game = auth.game(&mut tx, id).await?;
    let end = game.time_unlocked();
    let start = chrono::Duration::milliseconds(seek.unwrap_or(0).into());
    if start >= end {
        return Err(ApiError::forbidden(
            "cannot seek past end of unlocked music",
        ));
    }
    let bytes = track::clip(&mut tx, game.track_id, start..end).await?;
    Ok((ContentType::new("audio", "wav"), bytes))
}
