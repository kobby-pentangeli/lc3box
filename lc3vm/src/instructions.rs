use crate::{Opcode, Trapcode, VM};

/// Executes a single LC-3 instruction.
///
/// # Execution flow
/// 1. Decode instruction using first 4 bits as opcode
/// 2. Dispatch to appropriate instruction handler
/// 3. Handle invalid opcodes via VM error reporting
pub(crate) fn execute(instruction: u16, vm: &mut VM) {
    match Opcode::get(instruction) {
        Some(Opcode::Br) => br(instruction, vm),
        Some(Opcode::Add) => add(instruction, vm),
        Some(Opcode::Ld) => ld(instruction, vm),
        Some(Opcode::St) => st(instruction, vm),
        Some(Opcode::Jsr) => jsr(instruction, vm),
        Some(Opcode::And) => and(instruction, vm),
        Some(Opcode::Ldr) => ldr(instruction, vm),
        Some(Opcode::Str) => str(instruction, vm),
        Some(Opcode::Not) => not(instruction, vm),
        Some(Opcode::Ldi) => ldi(instruction, vm),
        Some(Opcode::Sti) => sti(instruction, vm),
        Some(Opcode::Jmp) => jmp(instruction, vm),
        Some(Opcode::Lea) => lea(instruction, vm),
        Some(Opcode::Trap) => trap(instruction, vm),
        _ => {}
    }
}

/// Branch to a PC-relative address if conditions are met.
///
/// Tests the condition flags specified by bits [11:9] (N, Z, P):
/// If any specified flag matches the current condition register state,
/// jumps to `PC + sign-extended PCOffset9`.
///
/// # Encoding
///
/// ```txt
/// 15           12 │11 │10 │ 9 │8                                 0
/// ┌───────────────┼───┼───┼───┼───────────────────────────────────┐
/// │      0000     │ N │ Z │ P │             PCOffset9             │
/// └───────────────┴───┴───┴───┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Condition flags (1 = test, 0 = ignore)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn br(instruction: u16, vm: &mut VM) {
    let pc_offset = sign_extend(instruction & 0x1FF, 9);
    let cond = (instruction >> 9) & 0x7;

    if cond & vm.registers.cond != 0 {
        // This is temporarily declared as `u32` to prevent overflow.
        let val = vm.registers.pc as u32 + pc_offset as u32;
        vm.registers.pc = val as u16;
    }
}

/// Performs addition, storing the result in a destination register.
///
/// Supports two addressing modes:
/// - **Register mode**: Adds values from two source registers.
/// - **Immediate mode**: Adds a register value and a sign-extended 5-bit immediate value.
///
/// Updates condition flags based on the result.
///
/// # Encoding
///
/// ```txt
/// Register Mode:
/// 15           12 │11        9│8         6│ 5 │4     3│2         0
/// ┌───────────────┼───────────┼───────────┼───┼───────┼───────────┐
/// │      0001     │     DR    │  SR1      │ 0 │  00   │    SR2    │
/// └───────────────┴───────────┴───────────┴───┴───────┴───────────┘
///
/// Immediate Mode:
/// 15           12 │11        9│8         6│ 5 │4                 0
/// ┌───────────────┼───────────┼───────────┼───┼───────────────────┐
/// │      0001     │     DR    │  SR1      │ 1 │       IMM5        │
/// └───────────────┴───────────┴───────────┴───┴───────────────────┘
/// ```
pub(crate) fn add(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let sr1 = (instruction >> 6) & 0x7;
    let imm_flag = (instruction >> 5) & 0x1;

    if imm_flag == 1 {
        let imm5 = sign_extend(instruction & 0x1F, 5);
        let val = imm5 as u32 + vm.registers.get(sr1) as u32;
        vm.registers.update(dr, val as u16);
    } else {
        let sr2 = instruction & 0x7;
        let val = vm.registers.get(sr1) as u32 + vm.registers.get(sr2) as u32;
        vm.registers.update(dr, val as u16);
    }
    vm.registers.update_cond_register(dr);
}

/// Loads a value from memory into a register using PC-relative addressing.
///
/// The target address is computed as `incremented PC + sign-extended PCOffset9`.
/// The value at this address is loaded into DR, and condition flags are updated.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8                                 0
/// ┌───────────────┼───────────┼───────────────────────────────────┐
/// │      0010     │     DR    │            PCOffset9              │
/// └───────────────┴───────────┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Destination register (DR)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn ld(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let pc_offset = sign_extend(instruction & 0x1FF, 9);
    let addr = pc_offset as u32 + vm.registers.pc as u32;

    let val = vm.read_memory(addr as u16);
    vm.registers.update(dr, val);
    vm.registers.update_cond_register(dr);
}

/// Stores a register value to memory using PC-relative addressing.
///
/// The target address is computed as `incremented PC + sign-extended PCOffset9`.
/// The value from register `SR` is stored at this memory location.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8                                 0
/// ┌───────────────┼───────────┼───────────────────────────────────┐
/// │      0011     │     SR    │            PCOffset9              │
/// └───────────────┴───────────┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Source register (SR)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn st(instruction: u16, vm: &mut VM) {
    let sr = (instruction >> 9) & 0x7;
    let pc_offset = sign_extend(instruction & 0x1FF, 9);

    let val = (vm.registers.pc as u32 + pc_offset as u32) as u16;
    vm.write_memory(val as usize, vm.registers.get(sr));
}

/// Jumps to a subroutine, saving the return address in R7.
///
/// Supports two addressing modes:
/// - **PC-relative (JSR)**: Jumps to `PC + sign-extended PCOffset11`
/// - **Base register (JSRR)**: Jumps to address in base register
///
/// # Encoding
///
/// ```txt
/// JSR Format (PC-relative):
///  15           12│11 │10
/// ┌───────────────┼───┼───────────────────────────────────────────┐
/// │      0100     │ 1 │                PCOffset11                 │
/// └───────────────┴───┴───────────────────────────────────────────┘
///
/// JSRR Format (Base register):
///  15           12│11 │10    9│8     6│5                         0
/// ┌───────────────┼───┼───────┼───────┼───────────────────────────┐
/// │      0100     │ 0 │   00  │ BaseR │           00000           │
/// └───────────────┴───┴───────┴───────┴───────────────────────────┘
/// ```
/// - Bit [11]: Mode selector (1 = PC-relative, 0 = Base register)
/// - PC-relative mode:
///   - Bits [10:0]: 11-bit signed offset (sign-extended to 16 bits)
/// - Base register mode:
///   - Bits [8:6]: 3-bit base register identifier
pub(crate) fn jsr(instruction: u16, vm: &mut VM) {
    let base_reg = (instruction >> 6) & 0x7;
    let pc_offset = sign_extend(instruction & 0x7FF, 11);
    let jsr_flag = (instruction >> 11) & 1;

    vm.registers.r7 = vm.registers.pc;

    if jsr_flag != 0 {
        // JSR case, the address to jump to is computed from PCOffset11
        let val = (vm.registers.pc as u32 + pc_offset as u32) as u16;
        vm.registers.pc = val;
    } else {
        // JSSR case, address to jump to lives in the BaseR
        vm.registers.pc = vm.registers.get(base_reg);
    }
}

/// Performs bitwise AND, storing the result in a destination register.
///
/// Supports two addressing modes:
/// - **Register mode**: ANDs values from two source registers.
/// - **Immediate mode**: ANDs a register value with a sign-extended 5-bit immediate value.
///
/// Updates condition flags based on the result.
///
/// # Encoding
///
/// ```txt
/// Register Mode:
/// 15           12 │11        9│8         6│ 5 │4     3│2         0
/// ┌───────────────┼───────────┼───────────┼───┼───────┼───────────┐
/// │      0101     │     DR    │  SR1      │ 0 │  00   │    SR2    │
/// └───────────────┴───────────┴───────────┴───┴───────┴───────────┘
///
/// Immediate Mode:
/// 15            12│11        9│8         6│ 5 │4                 0
/// ┌───────────────┼───────────┼───────────┼───┼───────────────────┐
/// │      0101     │     DR    │  SR1      │ 1 │       IMM5        │
/// └───────────────┴───────────┴───────────┴───┴───────────────────┘
/// ```
pub(crate) fn and(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let sr1 = (instruction >> 6) & 0x7;
    let imm_flag = (instruction >> 5) & 0x1;

    if imm_flag == 1 {
        let imm5 = sign_extend(instruction & 0x1F, 5);
        vm.registers.update(dr, vm.registers.get(sr1) & imm5);
    } else {
        let sr2 = instruction & 0x7;
        vm.registers
            .update(dr, vm.registers.get(sr1) & vm.registers.get(sr2));
    }
    vm.registers.update_cond_register(dr);
}

/// Loads a value from memory using base+offset addressing.
///
/// The effective address is computed as `BaseR + sign-extended Offset6`.
/// The value at this address is loaded into `DR`.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8             6│5                 0
/// ┌───────────────┼───────────┼───────────────┼───────────────────┐
/// │      0110     │     DR    │     BaseR     │     Offset6       │
/// └───────────────┴───────────┴───────────────┴───────────────────┘
/// ```
/// - Bits [11:9]: Destination register (DR)
/// - Bits [8:6]: Base register (BaseR)
/// - Bits [5:0]: 6-bit signed offset (sign-extended to 16 bits)
pub(crate) fn ldr(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let base_reg = (instruction >> 6) & 0x7;
    let offset = sign_extend(instruction & 0x3F, 6);

    let val = (vm.registers.get(base_reg) as u32 + offset as u32) as u16;
    let val = vm.read_memory(val);
    vm.registers.update(dr, val);
    vm.registers.update_cond_register(dr);
}

/// Stores a register value to memory using base+offset addressing.
///
/// The effective address is computed as `BaseR + sign-extended Offset6`.
/// The value from `SR` is stored at this memory location.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8         6│5                    0
/// ┌───────────────┼───────────┼───────────┼───────────────────────┐
/// │      0111     │     SR    │   BaseR   │        Offset6        │
/// └───────────────┴───────────┴───────────┴───────────────────────┘
/// ```
/// - Bits [11:9]: Source register (SR)
/// - Bits [8:6]: Base register (BaseR)
/// - Bits [5:0]: 6-bit signed offset (sign-extended to 16 bits)
pub(crate) fn str(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let base_reg = (instruction >> 6) & 0x7;
    let offset = sign_extend(instruction & 0x3F, 6);

    let val = (vm.registers.get(base_reg) as u32 + offset as u32) as u16;
    vm.write_memory(val as usize, vm.registers.get(dr));
}

/// Performs bitwise NOT (one's complement) on a register value.
///
/// # Encoding
///
/// ```txt
/// 15           12 │11        9│8         6│ 5 │4                 0
/// ┌───────────────┼───────────┼───────────┼───┼───────────────────┐
/// │      1001     │     DR    │     SR    │ 1 │       1111        │
/// └───────────────┴───────────┴───────────┴───┴───────────────────┘
/// ```
/// - Bits [11:9]: Destination register (DR)
/// - Bits [8:6]: Source register (SR)
pub(crate) fn not(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let sr = (instruction >> 6) & 0x7;

    vm.registers.update(dr, !vm.registers.get(sr));
    vm.registers.update_cond_register(dr);
}

/// Loads a value from memory using indirect addressing.
///
/// The effective address is computed as `PC + sign-extended PCOffset9`.
/// This address contains the final target address to load from.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8                                 0
/// ┌───────────────┼───────────┼───────────────────────────────────┐
/// │      1010     │     DR    │            PCOffset9              │
/// └───────────────┴───────────┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Destination register (DR)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn ldi(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let pc_offset = sign_extend(instruction & 0x1FF, 9);

    let init_addr = vm.read_memory(vm.registers.pc + pc_offset);
    let val = vm.read_memory(init_addr);
    vm.registers.update(dr, val);
    vm.registers.update_cond_register(dr);
}

/// Stores a register value to memory using indirect addressing.
///
/// The effective address is computed as `PC + sign-extended PCOffset9`.
/// This address contains the final target address to store to.
///
/// # Encoding
///
/// ```txt
///  15           12│11        9│8                                 0
/// ┌───────────────┼───────────┼───────────────────────────────────┐
/// │      1011     │     SR    │            PCOffset9              │
/// └───────────────┴───────────┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Source register (SR)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn sti(instruction: u16, vm: &mut VM) {
    let sr = (instruction >> 9) & 0x7;
    let pc_offset = sign_extend(instruction & 0x1FF, 9);

    let val = (vm.registers.pc as u32 + pc_offset as u32) as u16;
    let addr = vm.read_memory(val) as usize;
    vm.write_memory(addr, vm.registers.get(sr));
}

/// Jumps to the address contained in a base register (JMP)
/// or returns from subroutine (RET when BaseR = R7).
///
/// # Encoding
///
/// ```txt
/// JMP Format:
///  15           12│11        9│8         6│5                    0
/// ┌───────────────┼───────────┼───────────┼───────────────────────┐
/// │      1100     │    000    │   BaseR   │       000000          │
/// └───────────────┴───────────┴───────────┴───────────────────────┘
///
/// RET Format:
///  15           12│11        9│8         6│5                    0
/// ┌───────────────┼───────────┼───────────┼───────────────────────┐
/// │      1100     │    000    │    111    │       000000          │
/// └───────────────┴───────────┴───────────┴───────────────────────┘
/// ```
pub(crate) fn jmp(instruction: u16, vm: &mut VM) {
    // `base_reg` will either be an arbitrary register or the register 7 (`111`)
    // in which case it would be the `RET` operation
    let base_reg = (instruction >> 6) & 0x7;
    vm.registers.pc = vm.registers.get(base_reg);
}

/// Loads the effective address of a PC-relative offset into a register.
///
/// # Encoding
///
/// ```text
///  15           12│11        9│8                                 0
/// ┌───────────────┼───────────┼───────────────────────────────────┐
/// │      1110     │     DR    │            PCOffset9              │
/// └───────────────┴───────────┴───────────────────────────────────┘
/// ```
/// - Bits [11:9]: Destination register (DR)
/// - Bits [8:0]: 9-bit signed offset (sign-extended to 16 bits)
pub(crate) fn lea(instruction: u16, vm: &mut VM) {
    let dr = (instruction >> 9) & 0x7;
    let pc_offset = sign_extend(instruction & 0x1FF, 9);

    let val = (vm.registers.pc as u32 + pc_offset as u32) as u16;
    vm.registers.update(dr, val);
    vm.registers.update_cond_register(dr);
}

/// Executes a trap service routine for I/O operations.
///
/// # Behavior
/// - Saves the incremented PC in R7
/// - Jumps to the trap handler address from the trap vector table
/// - Handles 6 predefined system calls (GETC, OUT, PUTS, IN, PUTSP, HALT)
pub(crate) fn trap(instruction: u16, vm: &mut VM) {
    Trapcode::execute(instruction, vm);
}

/// Increases `x`'s bit count to 16 while preserving its sign.
///
/// If `x` is a signed positive number, we left-pad with zeroes.
/// If `x` is a signed negative number, we left-pad with ones.
///
/// `bit_count` is the original number of bits that `x` had.
///
/// See [Sign extension](https://en.wikipedia.org/wiki/Sign_extension)
fn sign_extend(mut x: u16, bit_count: u8) -> u16 {
    // If the sign bit of `x` is nonzero, then `x` is a
    // signed negative number, so we left-pad with
    // (16 - bit_count) ones
    if (x >> (bit_count - 1)) & 1 != 0 {
        x |= 0xFFFF << bit_count;
    }

    // If the sign bit is 0, return as is,
    // since it's already left-padded with zeroes
    x
}
