use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use lc3vm::Lc3VM;

/// A self-contained nested-loop workload that keeps the fetch-decode-execute
/// path busy without any I/O: an outer counter (200) repeatedly resets an inner
/// counter (10000) and decrements it to zero, for roughly four million executed
/// instructions per run.
const PROGRAM: [u16; 9] = [
    0x2206, // LD   R1, OUTER
    0x2006, // LD   R0, INNER
    0x103F, // ADD  R0, R0, #-1
    0x03FE, // BRp  -2          ; inner loop
    0x127F, // ADD  R1, R1, #-1
    0x03FB, // BRp  -5          ; outer loop
    0xF025, // HALT
    0x00C8, // OUTER = 200
    0x2710, // INNER = 10000
];

fn loaded_vm() -> Lc3VM {
    let mut vm = Lc3VM::new();
    for (offset, &word) in PROGRAM.iter().enumerate() {
        let address = 0x3000u16 + u16::try_from(offset).expect("program fits");
        vm.memory.write(address, word);
    }
    vm
}

fn interpreter_loop(c: &mut Criterion) {
    c.bench_function("interpreter_loop_4m_instructions", |b| {
        b.iter_batched(
            loaded_vm,
            |mut vm| vm.run().expect("workload runs to HALT"),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, interpreter_loop);
criterion_main!(benches);
