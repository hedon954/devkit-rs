use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct SlidingWindowCount {
    inner: Arc<Mutex<SlidingWindowCountInner>>,
}

#[derive(Debug)]
struct SlidingWindowCountInner {
    buckets: Vec<u64>,
    win_size: u64,
    bucket_interval: Duration,
    last_update: Instant,
    last_index: usize,
}

impl SlidingWindowCount {
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

    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self.inner.lock().unwrap();

        inner.update_buckets();

        if inner.total_count() + n <= inner.win_size {
            inner.add_requests(n);
            true
        } else {
            false
        }
    }
}

impl SlidingWindowCountInner {
    fn update_buckets(&mut self) {
        let now = Instant::now();

        let bucket_passed = self.bucket_passed(now);

        for i in 0..bucket_passed {
            let idx = (i + self.last_index) % self.buckets.len();
            self.buckets[idx] = 0;
        }

        self.last_index = (self.last_index + bucket_passed) % self.buckets.len();
        self.last_update = now;
    }

    fn bucket_passed(&self, now: Instant) -> usize {
        let elapsed = now - self.last_update;
        let count = elapsed.div_duration_f64(self.bucket_interval) as usize;
        if count > self.buckets.len() {
            self.buckets.len()
        } else {
            count
        }
    }

    fn total_count(&self) -> u64 {
        self.buckets.iter().sum()
    }

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

        // first 20 requests should be allowed
        for _ in 0..SIZE {
            assert!(swc.allow());
        }

        // in current window, no more token should be allowed
        assert!(!swc.allow());
        assert_eq!(SIZE, swc.inner.lock().unwrap().total_count());

        // sleep for 1/2 interval, some older tokens would be removed,
        // new should be allowed.
        std::thread::sleep(WINDOW_INTERVAL / 2);
        assert!(swc.allow());

        // sleep for a long time, all buckets should be cleared.
        std::thread::sleep(WINDOW_INTERVAL * 2);
        assert!(swc.allow());
        assert_eq!(1, swc.inner.lock().unwrap().total_count());
    }
}
