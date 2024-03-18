use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_life::grid::{GridCoord, Universe, UniverseOld};

fn run_r(universe: &mut UniverseOld, generations: usize) {
    let mut _total: u64 = 0;
    for _ in 0..generations {
        _total += universe.update() as u64;
    }
}

fn run_generations(universe: &mut Universe, generations: usize) {
    let mut _total: u64 = 0;
    for _ in 0..generations {
        _total += universe.update() as u64;
    }
}

pub fn grid_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Life");

    let size = 100;
    let mut universe_orig = UniverseOld::new();

    let mut ab = true;
    for x in 0..size {
        for y in 0..size {
            if ab {
                universe_orig
                    .grid
                    .set(GridCoord::Valid(x as i64, y as i64), 1);
            }
            ab = !ab;
        }
    }

    let mut universe_generations = Universe::new();

    let mut ab = true;
    for x in 0..size {
        for y in 0..size {
            if ab {
                universe_generations
                    .grid
                    .set(GridCoord::Valid(x as i64, y as i64));
            }
            ab = !ab;
        }
    }

    group.bench_function("Orig", |b| {
        b.iter(|| run_r(&mut universe_orig, black_box(100)))
    });
    group.bench_function("Generations", |b| {
        b.iter(|| run_generations(&mut universe_generations, black_box(100)))
    });
}

criterion_group!(benches, grid_bench);
criterion_main!(benches);
