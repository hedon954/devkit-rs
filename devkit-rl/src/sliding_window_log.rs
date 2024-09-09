use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct SlidingWindowLog {
    inner: Arc<Mutex<SlidingWindowLogInner>>,
}

#[derive(Debug)]
struct SlidingWindowLogInner {
    size: u64,
    interval: Duration,
    logs: Vec<Instant>,
}

impl SlidingWindowLog {
    pub fn new(size: u64, interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlidingWindowLogInner {
                size,
                interval: interval.unwrap_or(Duration::from_secs(1)),
                logs: Vec::with_capacity(size as usize),
            })),
        }
    }

    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("Failed to lock sliding window log");

        let now = Instant::now();

        // If the log is not null, try to accept the new requests directly.
        if inner.try_accept(n, now) {
            return true;
        }

        // If the log is full, remove old entries that are outside of the window.
        let interval = inner.interval;
        let threshold = now - interval;
        inner.remove_older_than(&threshold);

        // Try to accept the new requests again.
        inner.try_accept(n, now)
    }
}

impl SlidingWindowLogInner {
    fn try_accept(&mut self, n: u64, now: Instant) -> bool {
        if self.logs.len() as u64 + n <= self.size {
            self.append(n, now);
            true
        } else {
            false
        }
    }

    fn append(&mut self, n: u64, now: Instant) {
        self.logs.append(&mut vec![now; n as usize]);
    }

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
