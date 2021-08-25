use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cythan::{BasicCythan, ChunkedCythan, CompleteCythan, Cythan};

fn get_input(size: usize) -> Vec<usize> {
    (0..size).map(|x| x * 3 - 1).collect()
}

fn basic_bench(b: &mut Criterion) {
    let mut cythan = BasicCythan::new(black_box(get_input(1_000_000)));
    b.bench_function("basic cythan", |b| {
        b.iter(|| cythan.next());
    });
}

fn chunked_bench(b: &mut Criterion) {
    let mut cythan = ChunkedCythan::new(black_box(get_input(1_000_000)));
    b.bench_function("chunked cythan", |b| {
        b.iter(|| cythan.next());
    });
}

fn complete_bench(b: &mut Criterion) {
    let mut cythan = CompleteCythan::new(black_box(get_input(1_000_000)));
    b.bench_function("complete cythan", |b| {
        b.iter(|| cythan.next());
    });
}

criterion_group!(benches, basic_bench, chunked_bench, complete_bench);
criterion_main!(benches);
