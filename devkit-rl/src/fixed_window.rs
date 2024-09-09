use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A fixed window rate limiter.
///
/// This struct implements a rate limiter based on the fixed window algorithm.
/// It allows a certain number of requests (tokens) within a fixed time window.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use devkit_rl::FixedWindow; // Replace with your actual crate name
///
/// const SIZE: u64 = 10;
/// const INTERVAL: Duration = Duration::from_secs(1);
///
/// let bucket = FixedWindow::new(SIZE, Some(INTERVAL));
///
/// assert!(bucket.allow());
/// ```
#[derive(Debug, Clone)]
pub struct FixedWindow {
    inner: Arc<Mutex<FixedWindowInner>>,
}

/// Inner data for the fixed window rate limiter.
///
/// This struct stores the configuration and state of the rate limiter, such as
/// the size of the window, the current count of requests, the time interval of
/// the window, and the last time the window was updated.
#[derive(Debug)]
struct FixedWindowInner {
    /// Maximum number of allowed requests within the time window.
    size: u64,
    /// Current count of requests within the current window.
    count: u64,
    /// Duration of the time window.
    interval: Duration,
    /// The time when the window was last updated.
    last_update: Instant,
    /// The time when the next window starts.
    next_win_time: Instant,
}

impl FixedWindow {
    /// Creates a new `FixedWindow` rate limiter.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of requests allowed within each time window.
    /// * `interval` - Optional duration of the time window. Defaults to 1 second if not provided.
    ///
    /// # Returns
    ///
    /// A new `FixedWindow` instance.
    pub fn new(size: u64, interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FixedWindowInner::new(size, interval))),
        }
    }

    /// Checks if a single request is allowed in the current time window.
    ///
    /// This is a convenience method for `allow_n(1)`.
    ///
    /// # Returns
    ///
    /// `true` if the request is allowed, `false` if it exceeds the limit.
    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    /// Checks if `n` requests are allowed in the current time window.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of requests to allow.
    ///
    /// # Returns
    ///
    /// `true` if the requests are allowed, `false` if they exceed the limit.
    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self.inner.lock().expect("Failed to lock fixed window");

        let now = Instant::now();

        // Check if the current time is beyond the next window time
        if now >= inner.next_win_time {
            // Calculate how many windows have passed
            let pass_win_count = (now - inner.last_update).div_duration_f64(inner.interval) as u32;
            inner.count = 0; // Reset count for the new window
            inner.last_update = inner.last_update + inner.interval * pass_win_count;
            inner.next_win_time = inner.last_update + inner.interval;
        }

        // Check if the new requests exceed the window size
        if inner.count + n > inner.size {
            false
        } else {
            inner.count += n;
            true
        }
    }
}

impl FixedWindowInner {
    /// Creates a new `FixedWindowInner` with the given size and interval.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of requests allowed in each window.
    /// * `interval` - Optional duration of the time window. Defaults to 1 second if not provided.
    ///
    /// # Returns
    ///
    /// A new `FixedWindowInner` instance.
    pub fn new(size: u64, interval: Option<Duration>) -> Self {
        let now = Instant::now();
        let interval = interval.unwrap_or(Duration::from_secs(1));
        let next_win_time = now + interval;

        Self {
            size,
            count: 0,
            interval,
            last_update: now,
            next_win_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_window_allow_n_out_of_size_should_failed() {
        const SIZE: u64 = 5;
        const INTERVAL: Duration = Duration::from_millis(1);

        let bucket = FixedWindow::new(SIZE, Some(INTERVAL));
        assert!(!bucket.allow_n(SIZE + 1));
    }

    #[test]
    fn fixed_window_should_work() {
        const SIZE: u64 = 10;
        const INTERVAL: Duration = Duration::from_millis(1);

        let bucket = FixedWindow::new(SIZE, Some(INTERVAL));

        // first 10 tokens should be allowed
        for _ in 0..SIZE {
            assert!(bucket.allow());
        }

        // in current window, no more token should be allowed
        for _ in 0..SIZE {
            assert!(!bucket.allow());
        }

        // sleep for 1 interval to generate new tokens,
        // here we make 5 tokens, should be allowed.
        std::thread::sleep(INTERVAL);
        for _ in 0..SIZE / 2 {
            assert!(bucket.allow());
        }
        // sleep half of interval, still in current window
        // the rest of 5 tokens should be allowed
        std::thread::sleep(INTERVAL / 2);
        for _ in 0..SIZE / 2 {
            assert!(bucket.allow());
        }
    }
}
