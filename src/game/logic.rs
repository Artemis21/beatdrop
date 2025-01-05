//! Game logic, including time calculations for timed games and win checking.
use crate::{DbConn, Game};
use chrono::{DateTime, Utc};
use eyre::Result;
use serde::Serialize;

/// Utility to construct a `chrono::Duration` from a number of seconds in a constant context.
const fn seconds(n: i64) -> chrono::Duration {
    match chrono::Duration::try_milliseconds(n * 1000) {
        Some(duration) => duration,
        // unwrap/expect not yet const
        _ => panic!("duration should be in range"),
    }
}

/// Game constants. This is generic over the maximum number of guesses, to
/// ensure that the correct number of clip lengths/unlock times are used. See
/// [`Constants`] for a type alias with the correct number of guesses.
#[derive(Clone, Copy)]
pub struct GenericConstants<const MAX_GUESSES: usize> {
    /// How long each clip is (from the start of the track).
    pub music_clip_lengths: [chrono::Duration; MAX_GUESSES],
    /// The maximum amount of time allotted to each guess.
    pub timed_guess_lengths: [chrono::Duration; MAX_GUESSES],
}

/// The total allowed guesses in a game.
const MAX_GUESSES: usize = 7;
/// How long each clip is.
const MUSIC_CLIP_LENGTHS: [chrono::Duration; MAX_GUESSES] = [
    seconds(1),
    seconds(2),
    seconds(4),
    seconds(7),
    seconds(11),
    seconds(16),
    seconds(30),
];
/// The duration each guess is given in a timed game.
const TIMED_GUESS_LENGTHS: [chrono::Duration; MAX_GUESSES] = [
    // 1s clip + 5s
    seconds(6),
    // 2s clip + 5s
    seconds(7),
    // 4s clip + 5s
    seconds(9),
    // 7s clip + 5s
    seconds(12),
    // 11s clip + 5s
    seconds(16),
    // 16s clip + 5s
    seconds(21),
    // 30s clip + 5s
    seconds(35),
];
/// The sum of every guess length, the maximum time a timed game can last.
const MAX_TIMED_GAME_LENGTH: chrono::Duration = seconds(106);

/// The constant game settings (a type alias for convenience, since only one number
/// of game is actually relevant).
pub type Constants = GenericConstants<MAX_GUESSES>;

/// The specific game constants.
pub const CONSTANTS: Constants = Constants {
    music_clip_lengths: MUSIC_CLIP_LENGTHS,
    timed_guess_lengths: TIMED_GUESS_LENGTHS,
};

/// Calculate the moment such that if a user started a timed game started before
/// that moment, they would now have run out of time.
pub fn timed_game_cutoff() -> DateTime<Utc> {
    Utc::now() - MAX_TIMED_GAME_LENGTH
}

/// A type representing the current guess state in a timed game.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentGuess {
    /// Zero-indexed number of the current guess.
    number: usize,
    /// The time the current guess started at.
    started_at: DateTime<Utc>,
    /// The maximum number of milliseconds the guess may last.
    length_millis: u64,
}

impl Game {
    /// How much of the music should be available for the player to listen to.
    pub fn time_unlocked(&self) -> chrono::Duration {
        MUSIC_CLIP_LENGTHS[self.chunks_unlocked()]
    }

    /// How many "chunks" of music have been unlocked.
    ///
    /// This is theoretically the same as the number of guesses. However, in
    /// timed games, guesses can "time out" (get skipped automatically because
    /// the time allowed for them has run out). Timed out guesses should count
    /// towards the number of chunks unlocked, but may not yet be materialised
    /// in the database.
    ///
    /// Also note that this starts from zero, even though some music is
    /// available from the start of the game. In that respect, the return value
    /// is actually one less than the number of chunks of music available.
    pub fn chunks_unlocked(&self) -> usize {
        if self.is_over() {
            return MAX_GUESSES - 1; // the final guess doesn't unlock a new chunk
        }
        if self.is_timed {
            self.current_guess().number.min(MAX_GUESSES - 1)
        } else {
            self.guesses.len()
        }
    }

    /// Get the current timed guess state (result is meaningless in a non-timed game).
    ///
    /// If the game is over, only the `number` field is meaningful (it will be [`MAX_GUESSES`]).
    pub fn current_guess(&self) -> CurrentGuess {
        let started_at = self
            .guesses
            .last()
            .map_or(self.started_at, |g| g.guessed_at);
        let number = self.guesses.len();
        let length = CONSTANTS.timed_guess_lengths[self.guesses.len()];
        CurrentGuess {
            number,
            started_at,
            length_millis: u64::try_from(length.num_milliseconds())
                .expect("guess length is positive"),
        }
    }

    /// Whether the game is over.
    ///
    /// Note that this only checks if `won` has been set. For timed games, this
    /// may not yet be the case even when the time has actually run out. Too make
    /// sure, see [`Game::end_if_over`] or [`Game::is_out_of_time`].
    pub fn is_over(&self) -> bool {
        self.won.is_some()
    }

    /// Update any game state that depends on the current time.
    pub async fn auto_update(&mut self, db: &mut DbConn) -> Result<()> {
        if self.is_over() {
            return Ok(());
        }
        if self.is_timed && !self.is_out_of_guesses() {
            let mut last_guess_at = self
                .guesses
                .last()
                .map_or(self.started_at, |g| g.guessed_at);
            let mut current_length = CONSTANTS.timed_guess_lengths[self.guesses.len()];
            let now = Utc::now();
            while now - last_guess_at > current_length {
                last_guess_at += current_length;
                self.new_guess(db, None, Some(last_guess_at)).await?;
                if self.is_out_of_guesses() {
                    break;
                }
                current_length = CONSTANTS.timed_guess_lengths[self.guesses.len()];
            }
        }
        if self.is_guessed() {
            self.set_won(db, true).await?;
        } else if self.is_out_of_guesses() {
            self.set_won(db, false).await?;
        }
        Ok(())
    }

    /// Whether the track has been guessed.
    fn is_guessed(&self) -> bool {
        self.guesses
            .iter()
            .any(|guess| *guess.track_id == Some(self.track_id))
    }

    /// Whether the player has run out of guesses.
    fn is_out_of_guesses(&self) -> bool {
        self.guesses.len() >= MAX_GUESSES
    }
}
