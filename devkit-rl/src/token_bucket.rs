use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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

    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

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
