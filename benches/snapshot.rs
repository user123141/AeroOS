use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

fn bench_blake3(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3");
    let data = vec![0xABu8; 4096];
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("hash_4k", |b| b.iter(|| blake3::hash(&data)));
    group.finish();
}

fn bench_lz4(c: &mut Criterion) {
    let mut group = c.benchmark_group("lz4");
    let data = vec![0xABu8; 4096];
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("compress_4k", |b| {
        b.iter(|| lz4_flex::compress_prepend_size(&data))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(5));
    targets = bench_blake3, bench_lz4
}
criterion_main!(benches);
