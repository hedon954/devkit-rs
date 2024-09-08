use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A thread-safe token bucket rate limiter.
///
/// This struct implements a token bucket, which is a mechanism to control the rate
/// at which actions can be performed. Tokens are added to the bucket at a fixed
/// rate, and actions can only proceed if there are enough tokens in the bucket.
///
/// The `TokenBucket` struct is thread-safe and can be shared across multiple threads.
///
/// # Example
/// ```
/// use std::time::Duration;
/// use devkit_rl::TokenBucket;
///
/// let bucket = TokenBucket::new(100, 10, Some(Duration::from_secs(1)));
/// assert!(bucket.allow()); // Allows 1 token
/// assert!(bucket.allow_n(5)); // Allows 5 tokens
/// ```
pub struct TokenBucket {
    inner: Arc<Mutex<TokenBucketInner>>,
}

struct TokenBucketInner {
    tokens: u64,
    capacity: u64,
    refill_rate: u64,
    refill_interval: Duration,
    last_refill_time: Instant,
}

impl TokenBucket {
    /// Creates a new `TokenBucket` with the specified capacity, refill rate, and optional refill interval.
    ///
    /// - `capacity`: The maximum number of tokens the bucket can hold.
    /// - `refill_rate`: The number of tokens to add during each refill interval.
    /// - `refill_interval`: The time duration between each refill. If `None` is provided, the default is 1 second.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of tokens in the bucket.
    /// * `refill_rate` - Number of tokens to refill per interval.
    /// * `refill_interval` - Interval between refills (optional).
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use devkit_rl::TokenBucket;
    ///
    /// let bucket = TokenBucket::new(100, 10, Some(Duration::from_secs(1)));
    /// ```
    pub fn new(capacity: u64, refill_rate: u64, refill_interval: Option<Duration>) -> Self {
        let inner = TokenBucketInner {
            tokens: capacity, // initially fill the bucket to capacity
            capacity,
            refill_rate,
            refill_interval: refill_interval.unwrap_or(Duration::from_secs(1)), // default to 1 second
            last_refill_time: Instant::now(),
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// Attempts to consume 1 token from the bucket.
    ///
    /// Returns `true` if the token was successfully consumed, or `false` if there are not enough tokens available.
    ///
    /// # Example
    /// ```
    /// use devkit_rl::TokenBucket;
    ///
    /// let bucket = TokenBucket::new(100, 10, Some(std::time::Duration::from_secs(1)));
    /// assert!(bucket.allow());
    /// ```
    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    /// Attempts to consume `n` tokens from the bucket.
    ///
    /// Returns `true` if `n` tokens were successfully consumed, or `false` if there are not enough tokens available.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of tokens to consume.
    ///
    /// # Example
    /// ```
    /// use devkit_rl::TokenBucket;
    ///
    /// let bucket = TokenBucket::new(100, 10, Some(std::time::Duration::from_secs(1)));
    /// assert!(bucket.allow_n(5));
    /// ```
    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self.inner.lock().expect("Failed to lock token bucket");

        inner.advance();

        if n > inner.tokens {
            false
        } else {
            inner.tokens -= n;
            true
        }
    }
}

impl TokenBucketInner {
    /// Advances the token bucket, adding tokens based on the elapsed time since the last refill.
    ///
    /// This method checks how much time has passed since the last token refill and adds tokens
    /// to the bucket accordingly, ensuring that the number of tokens in the bucket does not
    /// exceed its capacity.
    fn advance(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_refill_time;

        if elapsed < self.refill_interval {
            return;
        }

        let interval_count = elapsed.div_duration_f64(self.refill_interval) as u64;
        let tokens_to_add = interval_count * self.refill_rate;
        self.tokens = self.tokens.saturating_add(tokens_to_add);
        self.tokens = self.tokens.min(self.capacity);

        let passed_time = self
            .refill_interval
            .checked_mul(interval_count as u32)
            .expect("Failed to calculate passed time");

        self.last_refill_time = self
            .last_refill_time
            .checked_add(passed_time)
            .expect("Failed to update last refill time");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the behavior of the token bucket.
    ///
    /// - Verifies that tokens can be consumed up to the bucket's capacity.
    /// - Ensures that consuming more tokens than available results in failure.
    /// - Tests the refilling behavior over time.
    #[test]
    fn token_bucket_should_work() {
        const RATE: u64 = 10;
        const CAPACITY: u64 = 100;
        const INTERVAL: Duration = Duration::from_millis(1);

        let bucket = TokenBucket::new(CAPACITY, RATE, Some(INTERVAL));

        // first 100 tokens should be allowed
        for _ in 0..CAPACITY {
            assert!(bucket.allow());
        }

        // next 100 tokens should be rejected
        for _ in 0..CAPACITY {
            assert!(!bucket.allow());
        }

        // sleep for 1 interval, then 10 tokens should be allowed again
        std::thread::sleep(INTERVAL);
        for _ in 0..RATE {
            assert!(bucket.allow());
        }

        // the new token have been consumed, so 100 tokens should be rejected again
        for _ in 0..CAPACITY {
            assert!(!bucket.allow());
        }

        // sleep for lots of intervals, new tokens should be allowed,
        // and tokens should be replenished.
        std::thread::sleep(INTERVAL * 11);
        assert!(bucket.allow());
        assert_eq!(bucket.inner.lock().unwrap().tokens, CAPACITY - 1);
    }
}
