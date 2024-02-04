//! The game response type, used for serialising games to JSON.
use super::logic::{Constants, CurrentGuess, GenericConstants, CONSTANTS};
use crate::{deezer, track, DbConn, Game};
use chrono::{DateTime, Utc};
use eyre::Result;
use serde::Serialize;

impl Game {
    /// Convert a game into a response, leaving out information that the user
    /// should not be able to see.
    pub async fn into_response(self, db: &mut DbConn) -> Result<Response> {
        let genre = match *self.genre_id {
            Some(genre_id) => Some(track::genre(db, genre_id).await?),
            None => None,
        };
        let timed_guess = if self.is_timed && self.won.is_none() {
            Some(self.current_guess())
        } else {
            None
        };
        let id = self.id;
        let started_at = self.started_at;
        let is_daily = self.is_daily;
        let is_timed = self.is_timed;
        let won = self.won;
        let track = match &self.won {
            Some(_) => Some(if let Some(track) = self.track_cache {
                track
            } else {
                track::Meta::get(db, self.track_id).await?
            }),
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
            id,
            started_at,
            is_daily,
            is_timed,
            genre,
            won,
            guesses,
            timed_guess,
            track,
            constants: CONSTANTS,
        })
    }
}

/// Response data for a game.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The game ID.
    id: i32,
    /// The time the game was started.
    started_at: DateTime<Utc>,
    /// If this is a daily mode game.
    is_daily: bool,
    /// If this is a timed mode game. Mutually exclusive with `is_daily`.
    is_timed: bool,
    /// If this is a genre-specific game, the genre ID. Otherwise `null`.
    ///
    /// Mutually exclusive with `is_daily`.
    genre: Option<deezer::Genre>,
    /// The guesses (or skips) made so far in this game.
    guesses: Vec<GuessResponse>,
    /// Timing information on the current guess, if this is an ongoing timed game.
    timed_guess: Option<CurrentGuess>,
    /// If the game has ended, whether the user won.
    won: Option<bool>,
    /// If the game has ended, the track that was being guessed.
    track: Option<track::Meta>,
    /// The game constants.
    constants: Constants,
}

/// Response data for an indivdual guess.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GuessResponse {
    /// The track that was guessed, or `null` if the guess was skipped.
    track: Option<track::Meta>,
    /// The time the guess was made.
    guessed_at: DateTime<Utc>,
}

/// A utility type for serialising the game constants.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConstantsSerde {
    /// The maximum number of guesses allowed in a game.
    max_guesses: usize,
    /// The lengths of the music clips, in milliseconds.
    music_clip_millis: Vec<u64>,
    /// The maximum amount of time allotted to each guess, in milliseconds.
    timed_guess_millis: Vec<u64>,
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
        let timed_guess_millis = vals
            .timed_guess_lengths
            .iter()
            .map(|duration| {
                u64::try_from(duration.num_milliseconds()).expect("guess lengths to be positive")
            })
            .collect();
        Self {
            max_guesses: N,
            music_clip_millis,
            timed_guess_millis,
        }
    }
}

impl<const N: usize> Serialize for GenericConstants<N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ConstantsSerde::from(self).serialize(serializer)
    }
}
