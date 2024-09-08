use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use devkit_rl::TokenBucket;

fn criterion_benchmark(c: &mut Criterion) {
    let tb = TokenBucket::new(10, 100, Some(Duration::from_millis(1)));
    c.bench_function("token_bucket", |b| {
        b.iter(|| {
            tb.allow();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
