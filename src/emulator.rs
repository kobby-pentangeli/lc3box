pub mod instruction;
pub mod registers;
pub mod vm;

pub use instruction::{self as inst, Opcode};
pub use registers::Registers;
pub use vm::VM;

/// The LC-3 has 65536 memory locations,
/// the max addressable by `u16`, 2^16.
pub const MEMORY_SIZE: usize = u16::MAX as usize;

pub fn execute_program(vm: &mut VM) {
    while vm.registers.pc < MEMORY_SIZE as u16 {
        let inst = vm.read_memory(vm.registers.pc);
        vm.registers.pc += 1;
        execute_instruction(inst, vm)
    }
}

/// Executes a single LC-3 instruction.
///
/// # Execution flow
/// 1. Decode instruction using first 4 bits as opcode
/// 2. Dispatch to appropriate instruction handler
/// 3. Handle invalid opcodes via VM error reporting
pub fn execute_instruction(instruction: u16, vm: &mut VM) {
    match Opcode::get(instruction) {
        Some(Opcode::Br) => inst::br(instruction, vm),
        Some(Opcode::Add) => inst::add(instruction, vm),
        Some(Opcode::Ld) => inst::ld(instruction, vm),
        Some(Opcode::St) => inst::st(instruction, vm),
        Some(Opcode::Jsr) => inst::jsr(instruction, vm),
        Some(Opcode::And) => inst::and(instruction, vm),
        Some(Opcode::Ldr) => inst::ldr(instruction, vm),
        Some(Opcode::Str) => inst::str(instruction, vm),
        Some(Opcode::Not) => inst::not(instruction, vm),
        Some(Opcode::Ldi) => inst::ldi(instruction, vm),
        Some(Opcode::Sti) => inst::sti(instruction, vm),
        Some(Opcode::Jmp) => inst::jmp(instruction, vm),
        Some(Opcode::Lea) => inst::lea(instruction, vm),
        Some(Opcode::Trap) => inst::trap(instruction, vm),
        _ => {}
    }
}
