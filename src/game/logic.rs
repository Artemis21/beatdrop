//! Game logic, including time calculations for timed games and win checking.
use crate::{DbConn, Game};
use chrono::{DateTime, Utc};
use eyre::Result;

/// Utility to construct a `chrono::Duration` from a number of seconds in a constant context.
const fn seconds(n: i64) -> chrono::Duration {
    chrono::Duration::milliseconds(n * 1000)
}

/// Game constants. This is generic over the maximum number of guesses, to
/// ensure that the correct number of clip lengths/unlock times are used. See
/// [`Constants`] for a type alias with the correct number of guesses.
#[derive(Clone, Copy)]
pub struct GenericConstants<const MAX_GUESSES: usize> {
    /// How long each clip is (from the start of the track).
    pub music_clip_lengths: [chrono::Duration; MAX_GUESSES],
    /// The durations into the timed game at which each clip unlocks.
    /// The final element is the time at which the game ends.
    pub timed_unlock_times: [chrono::Duration; MAX_GUESSES],
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
/// The times at which each clip unlocks.
const TIMED_UNLOCK_TIMES: [chrono::Duration; MAX_GUESSES] = [
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

/// The total length of music played (for the final guess/game over screen).
const FULL_CLIP_LENGTH: chrono::Duration = MUSIC_CLIP_LENGTHS[MAX_GUESSES - 1];

/// How long a timed game lasts.
const TIMED_GAME_LENGTH: chrono::Duration = TIMED_UNLOCK_TIMES[MAX_GUESSES - 1];

/// The constant game settings (a type alias for convenience, since only one number
/// of game is actually relevant).
pub type Constants = GenericConstants<MAX_GUESSES>;

/// The specific game constants.
pub const CONSTANTS: Constants = Constants {
    music_clip_lengths: MUSIC_CLIP_LENGTHS,
    timed_unlock_times: TIMED_UNLOCK_TIMES,
};

/// Calculate the moment such that if a user started a timed game started before
/// that moment, they would now have run out of time.
pub fn timed_game_cutoff() -> DateTime<Utc> {
    Utc::now() - super::logic::TIMED_GAME_LENGTH
}

impl Game {
    /// How much of the music should be available for the player to listen to.
    pub fn time_unlocked(&self) -> chrono::Duration {
        if self.is_over() {
            return FULL_CLIP_LENGTH;
        }
        let idx = if self.is_timed {
            let elapsed = Utc::now() - self.started_at;
            TIMED_UNLOCK_TIMES.iter().filter(|&&t| t < elapsed).count()
        } else {
            self.guesses.len()
        };
        MUSIC_CLIP_LENGTHS[idx]
    }

    /// Whether the game is over.
    ///
    /// Note that this only checks if `won` has been set. For timed games, this
    /// may not yet be the case even when the time has actually run out. Too make
    /// sure, see [`Game::end_if_over`] or [`Game::is_out_of_time`].
    pub fn is_over(&self) -> bool {
        self.won.is_some()
    }

    /// End the game if it is over.
    pub async fn end_if_over(&mut self, db: &mut DbConn) -> Result<()> {
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

    /// Whether the track has been guessed.
    fn is_guessed(&self) -> bool {
        self.guesses
            .iter()
            .any(|guess| *guess.track_id == Some(self.track_id))
    }

    /// Whether the player has run out of time.
    fn is_out_of_time(&self) -> bool {
        self.is_timed && self.started_at + TIMED_GAME_LENGTH < Utc::now()
    }

    /// Whether the player has run out of guesses.
    fn is_out_of_guesses(&self) -> bool {
        self.guesses.len() >= MAX_GUESSES
    }
}
