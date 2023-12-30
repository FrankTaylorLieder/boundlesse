use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_life::grid;

fn run_r(size: usize, generations: usize) {
    let mut universe = grid::Universe::new();

    let mut ab = true;
    for x in 0..size {
        for y in 0..size {
            if ab {
                universe
                    .grid
                    .set(grid::GridCoord::Valid(x as i64, y as i64), 1);
            }
            ab = !ab;
        }
    }

    let mut _total: u64 = 0;
    for _ in 0..generations {
        _total += universe.update() as u64;
    }

    //println!("After {generations}: {_total} cells lived");
}

pub fn grid_bench(c: &mut Criterion) {
    c.bench_function("100R", |b| b.iter(|| run_r(black_box(100), black_box(10))));
}

criterion_group!(benches, grid_bench);
criterion_main!(benches);
