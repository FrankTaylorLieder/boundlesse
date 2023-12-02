use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_life::grid;

fn run_r(size: usize) {
    let mut grid = grid::SparseGrid::new();

    // TODO initialize the grid with a known pattern. Iterate the grid for N generations.
}

pub fn grid_bench(c: &mut Criterion) {
    println!("Hello");
    c.bench_function("100R", |b| b.iter(|| run_r(black_box(100))));
}

criterion_group!(benches, grid_bench);
criterion_main!(benches);
