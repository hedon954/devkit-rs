use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct FixedWindow {
    inner: Arc<Mutex<FixedWindowInner>>,
}

#[derive(Debug)]
struct FixedWindowInner {
    size: u64,
    count: u64,
    interval: Duration,
    last_update: Instant,
    next_win_time: Instant,
}

impl FixedWindow {
    pub fn new(size: u64, interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FixedWindowInner::new(size, interval))),
        }
    }

    pub fn allow(&self) -> bool {
        self.allow_n(1)
    }

    pub fn allow_n(&self, n: u64) -> bool {
        let mut inner = self.inner.lock().expect("Failed to lock fixed window");

        let now = Instant::now();

        if now >= inner.next_win_time {
            let pass_win_count = (now - inner.last_update).div_duration_f64(inner.interval) as u32;
            inner.count = 0;
            inner.last_update = inner.last_update + inner.interval * pass_win_count;
            inner.next_win_time = inner.last_update + inner.interval;
        }

        if inner.count + n > inner.size {
            false
        } else {
            inner.count += n;
            true
        }
    }
}

impl FixedWindowInner {
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
