use adel::window;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn window_benchmark(c: &mut Criterion) {
    c.bench_function("adel_window", |b| b.iter(|| window::WinitWindow::new()));
}

criterion_group!(benches, window_benchmark);
criterion_main!(benches);

/*
Criterion Notes for future Benchmarking
When testing functions that require initial one-off set up it's required to build it prior to running

fn my_bench(c: &mut Criterion) {
    // One-time setup code goes here
    c.bench_function("my_bench", |b| {
        // Per-sample (note that a sample can be many iterations) setup goes here
        b.iter(|| {
            // Measured code goes here
        });
    });
}
*/
