use chrono::{DateTime, Utc};
use eyre::Context;
use serde::Serialize;

use crate::{deezer, track, DbConn, Error, User};

const fn seconds(n: i64) -> chrono::Duration {
    chrono::Duration::milliseconds(n * 1000)
}

#[derive(Clone, Copy)]
pub struct GenericConstants<const MAX_GUESSES: usize> {
    /// How long each clip is (from the start of the track).
    music_clip_lengths: [chrono::Duration; MAX_GUESSES],
    /// The durations into the timed game at which each clip unlocks.
    /// The final element is the time at which the game ends.
    timed_unlock_times: [chrono::Duration; MAX_GUESSES],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConstantsSerde {
    max_guesses: usize,
    music_clip_millis: Vec<u64>,
    timed_unlock_millis: Vec<u64>,
}

impl<const N: usize> From<&GenericConstants<N>> for ConstantsSerde {
    fn from(vals: &GenericConstants<N>) -> Self {
        let music_clip_millis = vals
            .music_clip_lengths
            .iter()
            .map(|duration| {
                u64::try_from(duration.num_milliseconds()).expect("clip lengths to be positive")
            })
            .collect();
        let timed_unlock_millis = vals
            .timed_unlock_times
            .iter()
            .map(|duration| {
                u64::try_from(duration.num_milliseconds()).expect("unlock times to be positive")
            })
            .collect();
        Self {
            max_guesses: N,
            music_clip_millis,
            timed_unlock_millis,
        }
    }
}

impl<const N: usize> Serialize for GenericConstants<N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ConstantsSerde::from(self).serialize(serializer)
    }
}

const MAX_GUESSES: usize = 7;
const MUSIC_CLIP_LENGTHS: [chrono::Duration; 7] = [
    seconds(1),
    seconds(2),
    seconds(4),
    seconds(7),
    seconds(11),
    seconds(16),
    seconds(30),
];
const TIMED_UNLOCK_TIMES: [chrono::Duration; 7] = [
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
    // 30s clip + 5s
    seconds(106),
];
const TIMED_GAME_LENGTH: chrono::Duration = TIMED_UNLOCK_TIMES[MAX_GUESSES - 1];

pub type Constants = GenericConstants<MAX_GUESSES>;
pub const CONSTANTS: Constants = Constants {
    music_clip_lengths: MUSIC_CLIP_LENGTHS,
    timed_unlock_times: TIMED_UNLOCK_TIMES,
};

#[derive(sqlx::FromRow)]
pub struct Row {
    id: i32,
    #[allow(dead_code)] // we use `SELECT *` to reduce boilerplate
    account_id: i32,
    started_at: DateTime<Utc>,
    is_daily: bool,
    is_timed: bool,
    genre_id: deezer::OptionId,
    won: Option<bool>,
    pub track_id: deezer::Id,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Guess {
    track_id: deezer::OptionId,
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
    won: Option<bool>,
    track: Option<track::Meta>,
    constants: Constants,
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
        .await
        .wrap_err("error querying game guesses")?;
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

    pub async fn new_guess(
        &mut self,
        db: &mut DbConn,
        track_id: Option<deezer::Id>,
    ) -> Result<(), Error> {
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
        .await
        .wrap_err("error inserting game guess")?;
        self.guesses.push(guess);
        Ok(())
    }

    pub fn time_unlocked(&self) -> chrono::Duration {
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
            .wrap_err("error setting game win state")?;
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
            won: self.won,
            guesses,
            track,
            constants: CONSTANTS,
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
        .wrap_err("error querying current game")?;
        match game {
            Some(game) => {
                let mut game = game.with_guesses(db).await?;
                game.end_if_over(db).await?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    pub async fn expect_current_game(&self, db: &mut DbConn) -> Result<Game, Error> {
        self.current_game(db)
            .await?
            .ok_or_else(|| Error::NotFound("no active game"))
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
        .wrap_err("error querying daily game")?;
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
