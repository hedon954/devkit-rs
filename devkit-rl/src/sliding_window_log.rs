use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A rate limiter that uses a sliding window log algorithm.
///
/// This rate limiter tracks requests over a sliding window period. Each request is
/// logged with a timestamp, and the rate limiter ensures that the number of requests
/// in a specified time window does not exceed the allowed limit.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use devkit_rl::SlidingWindowLog;  // Replace `your_crate_name` with the actual crate name.
///
/// const SIZE: u64 = 10;
/// const INTERVAL: Duration = Duration::from_secs(1);
///
/// let rl = SlidingWindowLog::new(SIZE, Some(INTERVAL));
///
/// assert!(rl.allow());
#[derive(Debug, Clone)]
pub struct SlidingWindowLog {
    inner: Arc<Mutex<SlidingWindowLogInner>>,
}

/// Inner structure for `SlidingWindowLog`.
///
/// This structure contains the main logic for managing the rate limiter,
/// including the request log, window size, and time interval.
#[derive(Debug)]
struct SlidingWindowLogInner {
    /// The maximum number of requests allowed within the time window.
    size: u64,
    /// The duration of the sliding window.
    interval: Duration,
    /// A vector storing the timestamps of requests.
    logs: Vec<Instant>,
}

impl SlidingWindowLog {
    /// Creates a new `SlidingWindowLog` rate limiter.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of requests allowed within the time window.
    /// * `interval` - The duration of the sliding window. Defaults to 1 second if not provided.
    ///
    /// # Returns
    ///
    /// A new `SlidingWindowLog` instance.
    pub fn new(size: u64, interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowLogInner {
                size,
                interval: interval.unwrap_or(Duration::from_secs(1)),
                logs: Vec::with_capacity(size as usize),
            })),
        }
    }

    /// Attempts to allow a single request.
    ///
    /// This is a convenience method that is equivalent to calling `allow_n(1)`.
    ///
    /// # Returns
    ///
    /// `true` if the request is allowed, `false` otherwise.
    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    /// Attempts to allow `n` requests.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of requests to allow.
    ///
    /// # Returns
    ///
    /// `true` if the requests are allowed, `false` if they exceed the limit.
    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("Failed to lock sliding window log");

        let now = Instant::now();

        // First attempt to accept the requests based on current logs.
        if inner.try_accept(n, now) {
            return true;
        }

        // Remove outdated logs outside the sliding window.
        let interval = inner.interval;
        let threshold = now - interval;
        inner.remove_older_than(&threshold);

        // Try again after cleaning up.
        inner.try_accept(n, now)
    }
}

impl SlidingWindowLogInner {
    /// Tries to accept `n` requests at the current time.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of requests to accept.
    /// * `now` - The current timestamp.
    ///
    /// # Returns
    ///
    /// `true` if the requests are accepted, `false` if they exceed the size limit.
    fn try_accept(&mut self, n: u64, now: Instant) -> bool {
        if self.logs.len() as u64 + n <= self.size {
            self.append(n, now);
            true
        } else {
            false
        }
    }

    /// Appends `n` requests to the log at the given timestamp.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of requests to log.
    /// * `now` - The current timestamp.
    fn append(&mut self, n: u64, now: Instant) {
        self.logs.append(&mut vec![now; n as usize]);
    }

    /// Removes all log entries older than the provided threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The timestamp representing the start of the valid time window.
    fn remove_older_than(&mut self, threshold: &Instant) {
        self.logs.retain(|t| t >= threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sliding_window_log_should_work() {
        const SIZE: u64 = 10;
        const INTERVAL: Duration = Duration::from_millis(1);

        let rl = SlidingWindowLog::new(SIZE, Some(INTERVAL));

        // first 10 tokens should be allowed
        for i in 0..SIZE {
            if i < SIZE - 3 {
                std::thread::sleep(INTERVAL / SIZE as u32); // just sleep 7/10 interval
            }
            assert!(rl.allow());
        }

        // in current window, no more token should be allowed
        assert!(!rl.allow());

        // sleep for half of interval, some older tokens would be removed,
        // new should be allowed.
        std::thread::sleep(INTERVAL / 2);
        assert!(rl.allow());
    }
}
