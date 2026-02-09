mod code;
mod instructions;
mod registers;
mod vm;

pub use code::{Opcode, Trapcode};
pub use registers::Registers;
pub use vm::VM;

/// The LC-3 has 65536 memory locations,
/// the max addressable by `u16`, 2^16.
pub const MEMORY_SIZE: usize = u16::MAX as usize;

pub fn execute_program(vm: &mut VM) {
    while vm.registers.pc < MEMORY_SIZE as u16 {
        let inst = vm.read_memory(vm.registers.pc);
        vm.registers.pc += 1;
        instructions::execute(inst, vm)
    }
}
