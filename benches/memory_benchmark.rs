use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_model2_rust::memory::{MemoryInterface, Model2Memory};

fn benchmark_memory_access(c: &mut Criterion) {
    let mut memory = Model2Memory::new();

    c.bench_function("memory_read_u32", |b| {
        b.iter(|| memory.read_u32(black_box(0x00001000)).unwrap())
    });

    c.bench_function("memory_write_u32", |b| {
        b.iter(|| {
            memory
                .write_u32(black_box(0x00001000), black_box(0x12345678))
                .unwrap()
        })
    });

    c.bench_function("memory_block_read", |b| {
        b.iter(|| {
            memory
                .read_block(black_box(0x00001000), black_box(1024))
                .unwrap()
        })
    });
}

fn benchmark_memory_mapping(c: &mut Criterion) {
    let memory = Model2Memory::new();

    c.bench_function("address_resolution", |b| {
        b.iter(|| memory.memory_map.resolve(black_box(0x00001000)))
    });
}

criterion_group!(benches, benchmark_memory_access, benchmark_memory_mapping);
criterion_main!(benches);
