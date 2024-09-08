use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use devkit_rl::LeakyBucket;

fn leaky_bucket_benchmark(c: &mut Criterion) {
    let tb = LeakyBucket::new(10, 100, Some(Duration::from_millis(1)));
    c.bench_function("leaky_bucket", |b| {
        b.iter(|| {
            tb.allow();
        })
    });
}

criterion_group!(benches, leaky_bucket_benchmark);
criterion_main!(benches);
