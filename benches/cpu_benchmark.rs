use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_model2_rust::{
    cpu::NecV60,
    memory::Model2Memory,
};

fn benchmark_cpu_execution(c: &mut Criterion) {
    let mut cpu = NecV60::new();
    let mut memory = Model2Memory::new();
    
    // Préparer quelques instructions de test
    memory.write_u32(0x00000000, 0x00123456).unwrap(); // Instruction factice
    
    c.bench_function("cpu_single_step", |b| {
        b.iter(|| {
            cpu.step(black_box(&mut memory)).unwrap()
        })
    });
    
    c.bench_function("cpu_1000_cycles", |b| {
        b.iter(|| {
            cpu.run_cycles(black_box(1000), black_box(&mut memory)).unwrap()
        })
    });
}

fn benchmark_instruction_decoding(c: &mut Criterion) {
    use pixel_model2_rust::cpu::decode_instruction;
    
    c.bench_function("instruction_decode", |b| {
        b.iter(|| {
            decode_instruction(black_box(0x12345678), black_box(0x00000000))
        })
    });
}

criterion_group!(benches, benchmark_cpu_execution, benchmark_instruction_decoding);
criterion_main!(benches);