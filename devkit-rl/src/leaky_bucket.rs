use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct LeakyBucket {
    inner: Arc<Mutex<LeakyBucketInner>>,
}

#[derive(Debug, Clone)]
struct LeakyBucketInner {
    capacity: u64,
    current_level: u64,
    leak_rate: u64,
    leak_interval: Duration,
    last_leaktime: Instant,
    queue: mpsc::Sender<oneshot::Sender<()>>,
}

impl LeakyBucket {
    pub fn new(leak_rate: u64, capacity: u64, leak_interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LeakyBucketInner::new(
                leak_rate,
                capacity,
                leak_interval,
            ))),
        }
    }

    pub fn allow(&self) -> bool {
        if !self.try_allow() {
            return false;
        }

        let rx = self.create_notify();

        let _ = rx.recv();
        self.leak();
        true
    }

    fn try_allow(&self) -> bool {
        let mut inner = self.inner.lock().expect("Failed to lock leaky bucket");
        inner.try_allow()
    }

    fn create_notify(&self) -> oneshot::Receiver<()> {
        let inner = self.inner.lock().expect("Failed to lock leaky bucket");

        let (tx, rx) = oneshot::channel();
        inner
            .queue
            .send(tx)
            .expect("Failed to send to leaky bucket");

        rx
    }

    fn leak(&self) {
        let mut inner = self.inner.lock().expect("Failed to lock leaky bucket");
        inner.leak();
    }
}

impl LeakyBucketInner {
    fn new(leak_rate: u64, capacity: u64, leak_interval: Option<Duration>) -> Self {
        let (tx, rx) = mpsc::channel();

        let res = Self {
            capacity,
            current_level: 0,
            leak_rate,
            leak_interval: leak_interval.unwrap_or(Duration::from_secs(1)),
            last_leaktime: Instant::now(),
            queue: tx,
        };

        let mut res_clone = res.clone();
        thread::spawn(move || {
            res_clone.start(rx);
        });

        res
    }

    fn start(&mut self, rx: mpsc::Receiver<oneshot::Sender<()>>) {
        loop {
            let now = Instant::now();
            if now - self.last_leaktime < self.leak_interval {
                thread::sleep(self.leak_interval - (now - self.last_leaktime));
            }
            self.last_leaktime = Instant::now();
            for _ in 0..self.leak_rate {
                if let Ok(tx) = rx.recv() {
                    let _ = tx.send(());
                }
            }
        }
    }

    fn try_allow(&mut self) -> bool {
        if self.current_level >= self.capacity {
            false
        } else {
            self.current_level += 1;
            true
        }
    }

    fn leak(&mut self) {
        if self.current_level > 0 {
            self.current_level -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use thread::sleep;

    use super::*;

    #[test]
    fn leaky_bucket_should_work() {
        const LEAK_RATE: u64 = 1;
        const CAPACITY: u64 = 5;
        const INTERVAL: Duration = Duration::from_millis(1);

        let bucket = LeakyBucket::new(LEAK_RATE, CAPACITY, Some(INTERVAL));

        for _ in 0..CAPACITY {
            let bucket_clone = bucket.clone();
            thread::spawn(move || {
                assert!(bucket_clone.allow());
                println!("time: {}", Utc::now().timestamp_millis());
            });
        }

        sleep(Duration::from_micros(100));
        assert!(!bucket.allow());
        sleep(Duration::from_millis(6));
    }
}
