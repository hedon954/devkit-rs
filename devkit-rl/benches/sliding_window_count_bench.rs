use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use devkit_rl::SlidingWindowCount;

fn sliding_window_count_benchmark(c: &mut Criterion) {
    let tb = SlidingWindowCount::new(10, Duration::from_millis(1), 10);
    c.bench_function("sliding_window_count", |b| {
        b.iter(|| {
            tb.allow();
        })
    });
}

criterion_group!(benches, sliding_window_count_benchmark);
criterion_main!(benches);
