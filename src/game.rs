use chrono::{DateTime, Utc};
use eyre::Context;
use rocket::{
    http,
    outcome::try_outcome,
    request::{self, FromRequest},
};
use serde::Serialize;

use crate::{deezer, track, DbConn, Error, User, database};

const fn seconds(n: i64) -> chrono::Duration {
    chrono::Duration::milliseconds(n * 1000)
}

const MUSIC_CLIP_LENGTHS: [u32; 7] = [1, 2, 4, 7, 11, 16, 30];
const TIMED_UNLOCK_TIMES: [chrono::Duration; 6] = [
    // 1s clip + 5s
    seconds(6),
    // 2s clip + 5s
    seconds(13),
    // 4s clip + 5s
    seconds(22),
    // 7s clip + 5s
    seconds(34),
    // 11s clip + 5s
    seconds(50),
    // 16s clip + 5s
    seconds(71),
];
// Then the timed game ends after another 30s clip + 5s
const TIMED_GAME_LENGTH: chrono::Duration = chrono::Duration::milliseconds(106 * 1000);
const MAX_GUESSES: usize = 5;

#[derive(sqlx::FromRow)]
pub struct Row {
    id: i32,
    #[allow(dead_code)]  // we use `SELECT *` to reduce boilerplate
    account_id: i32,
    started_at: DateTime<Utc>,
    is_daily: bool,
    is_timed: bool,
    genre_id: database::DeezerOptionId,
    won: Option<bool>,
    pub track_id: deezer::Id,
}

#[derive(Serialize)]
struct Guess {
    track_id: database::DeezerOptionId,
    guessed_at: DateTime<Utc>,
}

pub struct Game {
    game: Row,
    guesses: Vec<Guess>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    id: i32,
    started_at: DateTime<Utc>,
    is_daily: bool,
    is_timed: bool,
    genre: Option<deezer::Genre>,
    guesses: Vec<GuessResponse>,
    unlocked_seconds: u32,
    won: Option<bool>,
    track: Option<track::Meta>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GuessResponse {
    track: Option<track::Meta>,
    guessed_at: DateTime<Utc>,
}

impl std::ops::Deref for Game {
    type Target = Row;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl std::ops::DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}

impl Row {
    async fn with_guesses(self, db: &mut DbConn) -> Result<Game, Error> {
        let guesses = sqlx::query_as!(
            Guess,
            "SELECT track_id, guessed_at FROM game_guess
            WHERE game_id = $1
            ORDER BY guess_number",
            self.id,
        )
        .fetch_all(db)
        .await.wrap_err("querying game guesses")?;
        Ok(Game {
            game: self,
            guesses,
        })
    }
}

impl Game {
    pub async fn create(
        db: &mut DbConn,
        user_id: i32,
        genre_id: Option<deezer::Id>,
        daily: bool,
        timed: bool,
        track_id: deezer::Id,
    ) -> Result<Self, Error> {
        let game = sqlx::query_as!(
            Row,
            "INSERT INTO game (account_id, is_daily, is_timed, genre_id, track_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *",
            user_id,
            daily,
            timed,
            genre_id.map(i32::from),
            i32::from(track_id),
        )
        .fetch_one(db)
        .await?;
        Ok(Self {
            game,
            guesses: Vec::new(),
        })
    }

    pub async fn new_guess(&mut self, db: &mut DbConn, track_id: Option<deezer::Id>) -> Result<(), Error> {
        let guess = sqlx::query_as!(
            Guess,
            "INSERT INTO game_guess (game_id, track_id, guess_number)
            VALUES ($1, $2, $3)
            RETURNING track_id, guessed_at",
            self.id,
            track_id.map(i32::from),
            i32::try_from(self.guesses.len()).expect("guess count to fit in i32"),
        )
        .fetch_one(db)
        .await.wrap_err("inserting game guess")?;
        self.guesses.push(guess);
        Ok(())
    }

    pub fn seconds_unlocked(&self) -> u32 {
        let idx = if self.is_timed {
            let elapsed = Utc::now() - self.started_at;
            TIMED_UNLOCK_TIMES.iter().filter(|&&t| t < elapsed).count()
        } else {
            self.guesses.len()
        };
        MUSIC_CLIP_LENGTHS[idx]
    }

    pub fn is_over(&self) -> bool {
        self.won.is_some()
    }

    pub async fn end_if_over(&mut self, db: &mut DbConn) -> Result<(), Error> {
        if self.is_over() {
            return Ok(());
        }
        if self.is_guessed() {
            self.set_won(db, true).await?;
        } else if self.is_out_of_time() || self.is_out_of_guesses() {
            self.set_won(db, false).await?;
        }
        Ok(())
    }

    fn is_guessed(&self) -> bool {
        self.guesses
            .iter()
            .any(|guess| *guess.track_id == Some(self.track_id))
    }

    fn is_out_of_time(&self) -> bool {
        self.is_timed && self.started_at + TIMED_GAME_LENGTH < Utc::now()
    }

    fn is_out_of_guesses(&self) -> bool {
        self.guesses.len() >= MAX_GUESSES
    }

    async fn set_won(&mut self, db: &mut DbConn, won: bool) -> Result<(), Error> {
        sqlx::query!("UPDATE game SET won = $1 WHERE id = $2", won, self.id,)
            .execute(db)
            .await
            .wrap_err("setting game win state")?;
        self.won = Some(won);
        Ok(())
    }

    pub async fn into_response(self, db: &mut DbConn) -> Result<Response, Error> {
        let track = match &self.won {
            Some(_) => Some(track::Meta::get(db, self.track_id).await?),
            None => None,
        };
        let genre = match *self.genre_id {
            Some(genre_id) => Some(track::genre(db, genre_id).await?),
            None => None,
        };
        let mut guesses = Vec::with_capacity(self.guesses.len());
        for guess in &self.guesses {
            let track = match *guess.track_id {
                Some(track_id) => Some(track::Meta::get(db, track_id).await?),
                None => None,
            };
            guesses.push(GuessResponse {
                track,
                guessed_at: guess.guessed_at,
            });
        }
        Ok(Response {
            id: self.id,
            started_at: self.started_at,
            is_daily: self.is_daily,
            is_timed: self.is_timed,
            genre,
            unlocked_seconds: self.seconds_unlocked(),
            won: self.won,
            guesses,
            track,
        })
    }
}

impl User {
    pub async fn current_game(&self, db: &mut DbConn) -> Result<Option<Game>, Error> {
        let game = sqlx::query_as!(
            Row,
            "SELECT * FROM game WHERE account_id = $1 ORDER BY started_at DESC LIMIT 1",
            self.id,
        )
        .fetch_optional(&mut *db)
        .await
        .wrap_err("querying current game")?;
        match game {
            Some(game) => {
                let mut game = game.with_guesses(db).await?;
                game.end_if_over(db).await?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    pub async fn daily_game(&self, db: &mut DbConn) -> Result<Option<Game>, Error> {
        let game = sqlx::query_as!(
            Row,
            "SELECT * FROM game
            WHERE account_id = $1
                AND is_daily
                AND started_at >= DATE_TRUNC('day', TIMEZONE('utc', NOW()))",
            self.id,
        )
        .fetch_optional(&mut *db)
        .await
        .wrap_err("querying daily game")?;
        match game {
            Some(game) => {
                let mut game = game.with_guesses(db).await?;
                game.end_if_over(db).await?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    pub async fn has_played_daily(&self, db: &mut DbConn) -> Result<bool, Error> {
        sqlx::query_scalar!(
            "SELECT 1 FROM game
            WHERE account_id = $1
                AND is_daily
                AND started_at >= DATE_TRUNC('day', TIMEZONE('utc', NOW()))",
            self.id
        )
        .fetch_optional(db)
        .await
        .map_err(Error::from)
        .map(|r| r.is_some())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Game {
    type Error = Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut db = try_outcome!(database::extract(req).await);
        let user = try_outcome!(req.guard::<crate::User>().await);
        match user.current_game(&mut db).await {
            Ok(Some(game)) => request::Outcome::Success(game),
            Ok(None) => {
                request::Outcome::Error((http::Status::NotFound, Error::NotFound("no active game")))
            }
            Err(err) => request::Outcome::Error((http::Status::InternalServerError, err)),
        }
    }
}

pub struct Maybe(Option<Game>);

impl Maybe {
    pub async fn into_response(self, db: &mut DbConn) -> Result<Option<Response>, Error> {
        match self.0 {
            Some(game) => Ok(Some(game.into_response(db).await?)),
            None => Ok(None),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Maybe {
    type Error = Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut db = try_outcome!(database::extract(req).await);
        let user = try_outcome!(req.guard::<crate::User>().await);
        match user.current_game(&mut db).await {
            Ok(game) => request::Outcome::Success(Self(game)),
            Err(err) => request::Outcome::Error((http::Status::InternalServerError, err)),
        }
    }
}
