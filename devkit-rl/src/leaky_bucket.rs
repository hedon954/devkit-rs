use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

/// A leaky bucket rate limiter.
///
/// This implementation allows you to control the rate of events through a leaky bucket algorithm.
/// The bucket has a fixed capacity and leaks at a constant rate, allowing a maximum number of events
/// to pass through within a given interval.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use devkit_rl::LeakyBucket;
///
/// // Create a LeakyBucket with a leak rate of 1 event per second and a capacity of 5 events.
/// let bucket = LeakyBucket::new(1, 5, Some(Duration::from_secs(1)));
///
/// // Attempt to allow an event through the bucket.
/// assert!(bucket.allow());
/// ```
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
    /// Creates a new `LeakyBucket`.
    ///
    /// # Arguments
    ///
    /// * `leak_rate` - The rate at which the bucket leaks events per second.
    /// * `capacity` - The maximum capacity of the bucket.
    /// * `leak_interval` - The interval at which the bucket leaks events. If `None`, defaults to 1 second.
    ///
    /// # Returns
    ///
    /// Returns a new `LeakyBucket` instance.
    pub fn new(leak_rate: u64, capacity: u64, leak_interval: Option<Duration>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LeakyBucketInner::new(
                leak_rate,
                capacity,
                leak_interval,
            ))),
        }
    }

    /// Attempts to allow an event through the bucket.
    ///
    /// If the bucket has not reached its capacity and an event can be allowed,
    /// this method will return `true`. Otherwise, it returns `false`.
    ///
    /// This method blocks until the bucket's state is updated to reflect the allowance.
    ///
    /// # Returns
    ///
    /// Returns `true` if the event is allowed, `false` otherwise.
    pub fn allow(&self) -> bool {
        if !self.try_allow() {
            return false;
        }

        let rx = self.create_notify();

        let _ = rx.recv();
        self.leak();
        true
    }

    /// Attempts to allow an event through the bucket without blocking.
    ///
    /// This method checks if the event can be allowed immediately without blocking
    /// and updates the bucket's state accordingly.
    ///
    /// # Returns
    ///
    /// Returns `true` if the event is allowed, `false` otherwise.
    fn try_allow(&self) -> bool {
        let mut inner = self.inner.lock().expect("Failed to lock leaky bucket");
        inner.try_allow()
    }

    /// Creates a notification channel for the bucket.
    ///
    /// This method is used to create a one-shot channel that will be used to
    /// notify when an event can be allowed through the bucket.
    ///
    /// # Returns
    ///
    /// Returns a `oneshot::Receiver` that will receive the notification.
    fn create_notify(&self) -> oneshot::Receiver<()> {
        let inner = self.inner.lock().expect("Failed to lock leaky bucket");

        let (tx, rx) = oneshot::channel();
        inner
            .queue
            .send(tx)
            .expect("Failed to send to leaky bucket");

        rx
    }

    /// Updates the bucket's state to reflect that an event has been allowed.
    ///
    /// This method leaks the bucket to reflect the passage of time and allows
    /// an event through the bucket.
    fn leak(&self) {
        let mut inner = self.inner.lock().expect("Failed to lock leaky bucket");
        inner.leak();
    }
}

impl LeakyBucketInner {
    /// Creates a new `LeakyBucketInner`.
    ///
    /// # Arguments
    ///
    /// * `leak_rate` - The rate at which the bucket leaks events per second.
    /// * `capacity` - The maximum capacity of the bucket.
    /// * `leak_interval` - The interval at which the bucket leaks events. If `None`, defaults to 1 second.
    ///
    /// # Returns
    ///
    /// Returns a new `LeakyBucketInner` instance.
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

    /// Starts the leak process in a separate thread.
    ///
    /// This method continuously leaks events from the bucket based on the configured
    /// leak rate and interval. It listens for notifications and adjusts the bucket's state
    /// accordingly.
    ///
    /// # Arguments
    ///
    /// * `rx` - A receiver for one-shot notifications indicating when an event can be allowed.
    fn start(&mut self, rx: mpsc::Receiver<oneshot::Sender<()>>) {
        loop {
            let now = Instant::now();
            let wait_time = self.leak_interval.saturating_sub(now - self.last_leaktime);
            if wait_time > Duration::ZERO {
                thread::sleep(wait_time);
            }
            self.last_leaktime = Instant::now();
            for _ in 0..self.leak_rate {
                if let Ok(tx) = rx.recv() {
                    let _ = tx.send(());
                }
            }
        }
    }

    /// Attempts to allow an event through the bucket.
    ///
    /// This method increases the current level of the bucket if it is below capacity,
    /// indicating that an event has been allowed.
    ///
    /// # Returns
    ///
    /// Returns `true` if the event is allowed, `false` otherwise.
    fn try_allow(&mut self) -> bool {
        if self.current_level >= self.capacity {
            false
        } else {
            self.current_level += 1;
            true
        }
    }

    /// Leaks the bucket to reflect the passage of time.
    ///
    /// This method decreases the current level of the bucket if it is above zero,
    /// indicating that an event has leaked out of the bucket.
    fn leak(&mut self) {
        if self.current_level > 0 {
            self.current_level -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::thread::sleep;

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
