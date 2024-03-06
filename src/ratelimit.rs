//! Various ratelimiting utilities.
use std::{sync::Arc, time::Duration};

use tokio::sync::Semaphore;

/// A simple "leaky bucket" rate limiter. This is intended to be used as a
/// long-lived singleton, and will spawn a background task.
///
/// Based on [the example from the docs][1].
///
/// [1]: https://docs.rs/tokio/1.36.0/tokio/sync/struct.Semaphore.html#rate-limiting-using-a-token-bucket
pub struct Ratelimit(Arc<Semaphore>);

impl Ratelimit {
    /// Set up the ratelimiter and start the background task.
    ///
    /// `max_requests` is the maximum number of requests to allow at once.
    /// `increment_interval` is how long to wait before allowing another request.
    pub fn new(max_requests: usize, increment_interval: Duration) -> Self {
        let sem = Arc::new(Semaphore::new(max_requests));
        tokio::spawn({
            let sem = sem.clone();
            let mut interval = tokio::time::interval(increment_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            async move {
                loop {
                    interval.tick().await;
                    if sem.available_permits() < max_requests {
                        sem.add_permits(1);
                    }
                }
            }
        });
        Self(sem)
    }

    /// Acquire a permit to make a request. The future will resolve once a permit
    /// is available.
    pub async fn wait(&self) {
        self.0
            .acquire()
            .await
            .expect("semaphore shouldn't be closed")
            .forget();
    }
}

/// An exponential backoff iterator.
pub struct Backoff {
    /// The delay for the next iteration.
    delay: Duration,
    /// The maximum delay to allow.
    max_delay: Duration,
    /// The factor to multiply the delay by each time.
    factor: u32,
}

impl Backoff {
    /// Create a new backoff iterator.
    pub const fn new(initial_delay: Duration, max_delay: Duration, factor: u32) -> Self {
        Self {
            delay: initial_delay,
            max_delay,
            factor,
        }
    }
}

impl Iterator for Backoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.delay;
        self.delay = (self.delay * self.factor).min(self.max_delay);
        Some(current)
    }
}
