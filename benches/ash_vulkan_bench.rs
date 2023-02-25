use adel::renderer;
use adel::window::WinitWindow;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc;

pub fn window_benchmark(c: &mut Criterion) {
    c.bench_function("adel_window", |b| b.iter(|| WinitWindow::new()));
}
pub fn vulkan_benchmark(c: &mut Criterion) {
    c.bench_function("adel_vulkan", |b| {
        b.iter(|| {
            let mut winit_window = WinitWindow::new();
            let window = winit_window.window().unwrap();
            // TODO: Tmp Values to send data to System without creating unique functions
            let (tx, rx): (mpsc::Sender<(u32, u32)>, mpsc::Receiver<(u32, u32)>) = mpsc::channel();

            let renderer_ash = renderer::RendererAsh::new(&window, rx).unwrap();
            std::process::exit(0);
        })
    });
}

//criterion_group!(benches, window_benchmark);
criterion_group!(benches, vulkan_benchmark);
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
