//! Poll interval state with exponential backoff on error and reset on success.

use std::time::Duration;

pub struct BackoffState {
    /// Current delay before next poll.
    current_interval: Duration,
    /// Base interval when things are healthy.
    base_interval: Duration,
    /// Next backoff delay to use after an error (increases exponentially).
    next_backoff: Duration,
    /// Initial backoff after first error.
    backoff_initial: Duration,
    /// Multiplier for exponential backoff (e.g. 2).
    backoff_multiplier: u64,
    /// Cap for backoff delay.
    backoff_max: Duration,
}

impl BackoffState {
    pub fn new(config: &crate::config::Config) -> Self {
        Self {
            current_interval: config.poll_interval(),
            base_interval: config.poll_interval(),
            next_backoff: config.backoff_initial(),
            backoff_initial: config.backoff_initial(),
            backoff_multiplier: config.backoff_multiplier,
            backoff_max: config.backoff_max(),
        }
    }

    /// Duration to sleep before next poll.
    pub fn current_interval(&self) -> Duration {
        self.current_interval
    }

    /// Call after a successful poll: reset to base interval.
    pub fn on_success(&mut self) {
        self.current_interval = self.base_interval;
        self.next_backoff = self.backoff_initial;
    }

    /// Call after error (after quick retries exhausted): use backoff, then increase for next time.
    pub fn on_error(&mut self) {
        self.current_interval = self.next_backoff.min(self.backoff_max);
        self.next_backoff = Duration::from_secs(
            (self.next_backoff.as_secs() * self.backoff_multiplier).min(self.backoff_max.as_secs()),
        );
    }
}
