//! Database models and routes for games.
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::logic::timed_game_cutoff;
use crate::{deezer, track, DbConn, User};
use eyre::{Context, Result};

/// A model directly representing a row in the `game` table.
#[derive(sqlx::FromRow)]
pub struct Row {
    /// The game ID.
    pub id: i32,
    /// The ID of the user who started the game.
    pub account_id: i32,
    /// The time the game was started.
    pub started_at: DateTime<Utc>,
    /// If this is a daily mode game.
    pub is_daily: bool,
    /// If this is a timed mode game. Mutually exclusive with `is_daily`.
    pub is_timed: bool,
    /// If this is a genre-specific game, the genre ID. Otherwise `null`.
    ///
    /// Mutually exclusive with `is_daily`.
    pub genre_id: deezer::OptionId,
    /// If the game has ended, whether the user won.
    pub won: Option<bool>,
    /// The ID of the track being guessed.
    pub track_id: deezer::Id,
}

/// A single guess in a game.
///
/// This acts as both a database model and an API response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Guess {
    /// The ID of the track that was guessed, or `null` if the guess was skipped.
    pub track_id: deezer::OptionId,
    /// The time the guess was made.
    pub guessed_at: DateTime<Utc>,
}

/// A game, including all guesses made.
pub struct Game {
    /// The game row.
    game: Row,
    /// The guesses made in this game.
    pub guesses: Vec<Guess>,
    /// Track metadata for the track being guessed. This is just a cache and
    /// will be `None` if it hasn't been fetched from the database yet.
    pub track_cache: Option<track::Meta>,
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
    /// Fetch the guesses associated with this game.
    async fn with_guesses(self, db: &mut DbConn) -> Result<Game> {
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
            track_cache: None,
        })
    }
}

impl Game {
    /// Create a new game.
    ///
    /// Does no validation of the game mode, already ongoing games, etc.
    pub async fn create(
        db: &mut DbConn,
        user_id: i32,
        genre_id: Option<deezer::Id>,
        daily: bool,
        timed: bool,
        track_id: deezer::Id,
    ) -> Result<Self> {
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
            track_cache: None,
        })
    }

    /// Get a game from the database by ID.
    pub async fn get(db: &mut DbConn, id: i32) -> Result<Option<Self>> {
        let Some(game) = sqlx::query_as!(Row, "SELECT * FROM game WHERE id = $1 FOR UPDATE", id)
            .fetch_optional(&mut *db)
            .await? else { return Ok(None) };
        let mut game = game.with_guesses(&mut *db).await?;
        game.auto_update(&mut *db).await?;
        Ok(Some(game))
    }

    /// Submit a new guess for this game.
    ///
    /// The guess is not validated in any way.
    pub async fn new_guess(&mut self, db: &mut DbConn, track_id: Option<deezer::Id>) -> Result<()> {
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

    /// Set the game as won or lost.
    pub async fn set_won(&mut self, db: &mut DbConn, won: bool) -> Result<()> {
        sqlx::query!("UPDATE game SET won = $1 WHERE id = $2", won, self.id,)
            .execute(db)
            .await
            .wrap_err("error setting game win state")?;
        self.won = Some(won);
        Ok(())
    }

    /// End any timed games where the user has timed out.
    ///
    /// All these games must be lost: if they were won, they would have been ended
    /// when the user submitted a winning guess.
    pub async fn end_all_timed_out(db: &mut DbConn) -> Result<()> {
        sqlx::query!(
            "UPDATE game SET won = false WHERE
                is_timed
                AND started_at < $1
                AND won IS NULL",
            timed_game_cutoff(),
        )
        .execute(db)
        .await
        .wrap_err("error ending timed out games")?;
        Ok(())
    }

    /// Get the track metadata, caching the result.
    pub async fn track(&mut self, db: &mut DbConn) -> Result<&track::Meta> {
        if self.track_cache.is_none() {
            self.track_cache = Some(track::Meta::get(&mut *db, self.track_id).await?);
        }
        Ok(self.track_cache.as_ref().unwrap())
    }
}

impl User {
    /// Get the ID of the user's ongoing game, if any.
    pub async fn ongoing_game_id(&self, db: &mut DbConn) -> Result<Option<i32>> {
        sqlx::query_scalar!(
            "SELECT id FROM game WHERE
                account_id = $1
                AND won IS NULL
                -- timed games can end without being updated in the database, so
                -- we have to check for that:
                AND NOT (is_timed AND started_at < $2)
                FOR UPDATE
            ",
            self.id,
            timed_game_cutoff(),
        )
        .fetch_optional(&mut *db)
        .await
        .wrap_err("error querying current game")
    }

    /// Get the ID of the user's daily game for today, if started.
    pub async fn daily_game_id(&self, db: &mut DbConn) -> Result<Option<i32>> {
        sqlx::query_scalar!(
            "SELECT id FROM game
            WHERE account_id = $1
                AND is_daily
                AND started_at >= DATE_TRUNC('day', TIMEZONE('utc', NOW()))
            FOR UPDATE",
            self.id,
        )
        .fetch_optional(&mut *db)
        .await
        .wrap_err("error querying daily game")
    }
}
