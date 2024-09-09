use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A sliding window rate limiter based on counting requests over a specified time window.
///
/// The `SlidingWindowCount` rate limiter divides the time window into multiple buckets
/// and counts requests within those buckets. The buckets slide with time, allowing
/// a precise control over how many requests are allowed in a given time window.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use devkit_rl::SlidingWindowCount;  // Replace `your_crate_name` with the actual crate name.
///
/// const SIZE: u64 = 20;
/// const BUCKET_COUNT: u64 = 10;
/// const WINDOW_INTERVAL: Duration = Duration::from_millis(10);
///
/// let swc = SlidingWindowCount::new(SIZE, WINDOW_INTERVAL, BUCKET_COUNT);
///
/// assert!(swc.allow());
/// ```
#[derive(Debug, Clone)]
pub struct SlidingWindowCount {
    inner: Arc<Mutex<SlidingWindowCountInner>>,
}

/// Inner structure that holds the state of the sliding window.
///
/// This structure tracks the number of requests in each bucket, the total size of the window,
/// and the interval for each bucket.
#[derive(Debug)]
struct SlidingWindowCountInner {
    /// Vector to store request counts for each bucket.
    buckets: Vec<u64>,
    /// Maximum number of requests allowed within the window.
    win_size: u64,
    /// Duration of each bucket.
    bucket_interval: Duration,
    /// The time when the buckets were last updated.
    last_update: Instant,
    /// The index of the most recently updated bucket.
    last_index: usize,
}

impl SlidingWindowCount {
    /// Creates a new `SlidingWindowCount` rate limiter.
    ///
    /// # Arguments
    ///
    /// * `win_size` - The maximum number of requests allowed within the sliding window.
    /// * `interval` - The total duration of the sliding window.
    /// * `bucket_count` - The number of buckets to divide the sliding window into.
    ///
    /// # Returns
    ///
    /// A new `SlidingWindowCount` instance.
    pub fn new(win_size: u64, interval: Duration, bucket_count: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowCountInner {
                buckets: vec![0; bucket_count as usize],
                win_size,
                bucket_interval: interval.div_f64(bucket_count as f64),
                last_update: Instant::now(),
                last_index: 0,
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
        let mut inner = self.inner.lock().unwrap();

        // Update the buckets based on the current time.
        inner.update_buckets();

        // Check if adding the new requests would exceed the window size.
        if inner.total_count() + n <= inner.win_size {
            inner.add_requests(n);
            true
        } else {
            false
        }
    }
}

impl SlidingWindowCountInner {
    /// Updates the state of the buckets to account for the time that has passed since the last update.
    ///
    /// This function calculates how many buckets have passed and clears the old buckets that
    /// are outside of the current window.
    fn update_buckets(&mut self) {
        let now = Instant::now();

        // Calculate how many buckets have passed since the last update.
        let bucket_passed = self.bucket_passed(now);

        // Clear the contents of the passed buckets.
        for i in 0..bucket_passed {
            let idx = (i + self.last_index) % self.buckets.len();
            self.buckets[idx] = 0;
        }

        // Update the index and time for the most recent bucket.
        self.last_index = (self.last_index + bucket_passed) % self.buckets.len();
        self.last_update = now;
    }

    /// Calculates how many buckets have passed since the last update.
    ///
    /// # Arguments
    ///
    /// * `now` - The current timestamp.
    ///
    /// # Returns
    ///
    /// The number of buckets that have passed since `last_update`.
    fn bucket_passed(&self, now: Instant) -> usize {
        let elapsed = now - self.last_update;
        let count = elapsed.div_duration_f64(self.bucket_interval) as usize;

        // If more buckets have passed than the total number of buckets, clear all buckets.
        if count > self.buckets.len() {
            self.buckets.len()
        } else {
            count
        }
    }

    /// Returns the total number of requests in the current sliding window.
    fn total_count(&self) -> u64 {
        self.buckets.iter().sum()
    }

    /// Adds the specified number of requests to the current bucket.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of requests to add.
    fn add_requests(&mut self, n: u64) {
        self.buckets[self.last_index] += n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sliding_window_count_should_work() {
        const SIZE: u64 = 20;
        const BUCKET_COUNT: u64 = 10;
        const WINDOW_INTERVAL: Duration = Duration::from_millis(BUCKET_COUNT);

        let swc = SlidingWindowCount::new(SIZE, WINDOW_INTERVAL, BUCKET_COUNT);

        // First 20 requests should be allowed.
        for _ in 0..SIZE {
            assert!(swc.allow());
        }

        // No more requests should be allowed in the current window.
        assert!(!swc.allow());
        assert_eq!(SIZE, swc.inner.lock().unwrap().total_count());

        // After sleeping for half of the window interval, some older tokens should be removed,
        // allowing new requests.
        std::thread::sleep(WINDOW_INTERVAL / 2);
        assert!(swc.allow());

        // After sleeping for a long time, all buckets should be cleared, allowing new requests.
        std::thread::sleep(WINDOW_INTERVAL * 2);
        assert!(swc.allow());
        assert_eq!(1, swc.inner.lock().unwrap().total_count());
    }
}
