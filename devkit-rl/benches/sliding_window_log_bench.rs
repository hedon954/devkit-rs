use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use devkit_rl::SlidingWindowLog;

fn sliding_window_log_benchmark(c: &mut Criterion) {
    let tb = SlidingWindowLog::new(10, Some(Duration::from_millis(1)));
    c.bench_function("sliding_window_log", |b| {
        b.iter(|| {
            tb.allow();
        })
    });
}

criterion_group!(benches, sliding_window_log_benchmark);
criterion_main!(benches);
