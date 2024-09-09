use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use devkit_rl::FixedWindow;

fn fixed_window_benchmark(c: &mut Criterion) {
    let tb = FixedWindow::new(10, Some(Duration::from_millis(1)));
    c.bench_function("fixed_winodws", |b| {
        b.iter(|| {
            tb.allow();
        })
    });
}

criterion_group!(benches, fixed_window_benchmark);
criterion_main!(benches);
