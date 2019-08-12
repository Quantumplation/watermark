#[macro_use]

extern crate criterion;
use criterion::{Criterion, Throughput, Benchmark, BatchSize};
use watermark::WatermarkSet;

fn benchmarks(c: &mut Criterion) {
    let batch_size = 1000;
    let mut idx = 0;
    c.bench(
        "Insert",
        Benchmark::new(
            "In Order",
            move |b| {
                b.iter_batched(
                    WatermarkSet::default,
                    move |mut coll| {
                        for i in 1..batch_size {
                            coll.insert(i + 2*idx);
                        }
                        idx += batch_size;
                    },
                    BatchSize::SmallInput
                );
            }
        ).throughput(Throughput::Elements(batch_size)),
    );

    c.bench(
        "Insert",
        Benchmark::new(
            "Out of Order",
            move |b| {
                b.iter_batched(
                    WatermarkSet::default,
                    move |mut coll| {
                        for i in 1..(batch_size / 2) {
                            coll.insert(i + 2*idx);
                        }
                        for i in 1..(batch_size / 2) {
                            coll.insert(i + 2*idx + 1);
                        }
                        idx += batch_size;
                    },
                    BatchSize::SmallInput
                );
            }
        ).throughput(Throughput::Elements(batch_size)),
    );

    c.bench(
        "Contains",
        Benchmark::new(
            "Aligned",
            move |b| {
                b.iter_batched(
                    || {
                        let mut coll = WatermarkSet::default();
                        for i in 0..64*10 {
                            coll.insert(i);
                        }
                        coll
                    },
                    |coll| {
                        for i in 0..64*10 {
                            coll.contains(i);
                        }
                    },
                    BatchSize::SmallInput
                );
            }
        ).throughput(Throughput::Elements(64 * 10)),
    );

    c.bench(
        "Contains",
        Benchmark::new(
            "Unaligned",
            move |b| {
                b.iter_batched(
                    || {
                        let mut coll = WatermarkSet::default();
                        for i in 0..64*5 {
                            coll.insert(i * 2);
                        }
                        coll
                    },
                    |coll| {
                        for i in 0..64*10 {
                            coll.contains(i);
                        }
                    },
                    BatchSize::SmallInput
                );
            }
        ).throughput(Throughput::Elements(64*10)),
    );
                    
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
