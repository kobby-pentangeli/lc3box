pub mod instruction;
pub mod registers;
pub mod vm;

pub use instruction::Opcode;
pub use registers::Registers;
pub use vm::VM;

pub const MEMORY_SIZE: usize = u16::MAX as usize;

pub fn execute_program(vm: &mut VM) {
    while vm.registers.pc < MEMORY_SIZE as u16 {
        let inst = vm.read_memory(vm.registers.pc);
        vm.registers.pc += 1;
        execute_instruction(inst, vm)
    }
}

pub fn execute_instruction(instruction: u16, _vm: &mut VM) {
    let _opcode = Opcode::get(instruction);
    todo!()
}
