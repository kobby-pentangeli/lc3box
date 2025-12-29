pub mod opcode;

pub use opcode::Opcode;

use super::VM;

pub(crate) fn br(instruction: u16, vm: &mut VM) {}

pub(crate) fn add(instruction: u16, vm: &mut VM) {}

pub(crate) fn ld(instruction: u16, vm: &mut VM) {}

pub(crate) fn st(instruction: u16, vm: &mut VM) {}

pub(crate) fn jsr(instruction: u16, vm: &mut VM) {}

pub(crate) fn and(instruction: u16, vm: &mut VM) {}

pub(crate) fn ldr(instruction: u16, vm: &mut VM) {}

pub(crate) fn str(instruction: u16, vm: &mut VM) {}

pub(crate) fn not(instruction: u16, vm: &mut VM) {}

pub(crate) fn ldi(instruction: u16, vm: &mut VM) {}

pub(crate) fn sti(instruction: u16, vm: &mut VM) {}

pub(crate) fn jmp(instruction: u16, vm: &mut VM) {}

pub(crate) fn lea(instruction: u16, vm: &mut VM) {}

pub(crate) fn trap(instruction: u16, vm: &mut VM) {}
