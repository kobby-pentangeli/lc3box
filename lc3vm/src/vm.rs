use std::io::{self, Write as _};
use std::path::Path;

use lc3core::{KBDR, KBSR, ObjectFile, Opcode, TrapVector, sign_extend};

use crate::{Error, Memory, Registers, console};

/// The main LC-3 emulator.
///
/// ## Memory Architecture
/// - 16-bit address space (0x0000-0xFFFF)
/// - First 0xFE00 addresses: general purpose memory
/// - 0xFE00-0xFFFF: Memory-mapped I/O registers
///
/// ## Execution Flow
/// 1. Fetch instruction from PC
/// 2. Decode opcode
/// 3. Execute instruction
/// 4. Update condition codes
pub struct Lc3VM {
    /// 16-bit addressable memory space
    pub memory: Memory,
    /// Processor registers and flags
    pub registers: Registers,
}

impl Default for Lc3VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Lc3VM {
    /// Creates a new VM in initial state.
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            registers: Registers::new(),
        }
    }

    /// Loads the LC-3 object file at `path` and returns a VM ready to run it.
    ///
    /// Returns [`Error::Io`] if the file cannot be read, [`Error::Object`] if
    /// its bytes are not a valid `.obj` image, and [`Error::ProgramOutOfRange`]
    /// if the image would not fit in memory.
    pub fn init_from_program(path: &Path) -> Result<Self, Error> {
        Self::load(&ObjectFile::from_be_bytes(&std::fs::read(path)?)?)
    }

    /// Builds a VM with `image` loaded at its origin and the program counter set
    /// there, ready to [`run`](Self::run).
    ///
    /// Returns [`Error::ProgramOutOfRange`] if the image's words would extend
    /// past the top of the address space.
    fn load(image: &ObjectFile) -> Result<Self, Error> {
        let mut vm = Self::new();
        vm.memory
            .region_mut(image.origin, image.words.len())
            .ok_or(Error::ProgramOutOfRange {
                origin: image.origin,
                words: image.words.len(),
            })?
            .copy_from_slice(&image.words);
        vm.registers.pc = image.origin;
        Ok(vm)
    }

    /// Runs the loaded program from the current program counter until it halts.
    ///
    /// Each iteration fetches the word at the program counter, advances the
    /// counter (wrapping at the top of the address space), and executes the
    /// instruction. Returns `Ok(())` once a `HALT` trap is reached, or an
    /// [`Error`] the moment the machine reaches an instruction it cannot run.
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let instruction = self.read_memory(self.registers.pc)?;
            self.registers.pc = self.registers.pc.wrapping_add(1);
            if let Flow::Halt = self.step(instruction)? {
                return Ok(());
            }
        }
    }

    /// Reads the word at `address`, polling the keyboard first when `address`
    /// is the keyboard status register so a program sees fresh input.
    fn read_memory(&mut self, address: u16) -> Result<u16, Error> {
        if address == KBSR {
            self.poll_keyboard()?;
        }
        Ok(self.memory.read(address))
    }

    /// Refreshes the keyboard registers from the console without blocking.
    ///
    /// A waiting key sets the ready bit of `KBSR` and lands in `KBDR`; with no
    /// key ready, `KBSR` is cleared, so a polling program simply tries again.
    fn poll_keyboard(&mut self) -> Result<(), Error> {
        match console::poll_char()? {
            Some(key) => {
                self.memory.write(KBSR, 1 << 15);
                self.memory.write(KBDR, u16::from(key));
            }
            None => self.memory.write(KBSR, 0),
        }
        Ok(())
    }

    /// Decodes and executes one instruction, reporting how execution proceeds.
    ///
    /// Ordinary instructions yield [`Flow::Continue`]; a `HALT` trap yields
    /// [`Flow::Halt`]. The privileged (`RTI`), reserved, and unknown-trap
    /// encodings have no defined effect in this user-mode VM, so rather than
    /// silently advancing past them they stop the machine with an [`Error`].
    fn step(&mut self, instruction: u16) -> Result<Flow, Error> {
        match Opcode::decode(instruction) {
            Opcode::Br => self.br(instruction),
            Opcode::Add => self.add(instruction),
            Opcode::Ld => self.ld(instruction)?,
            Opcode::St => self.st(instruction),
            Opcode::Jsr => self.jsr(instruction),
            Opcode::And => self.and(instruction),
            Opcode::Ldr => self.ldr(instruction)?,
            Opcode::Str => self.str(instruction),
            Opcode::Not => self.not(instruction),
            Opcode::Ldi => self.ldi(instruction)?,
            Opcode::Sti => self.sti(instruction)?,
            Opcode::Jmp => self.jmp(instruction),
            Opcode::Lea => self.lea(instruction),
            Opcode::Trap => return self.trap(instruction),
            Opcode::Rti => return Err(Error::PrivilegedInstruction(instruction)),
            Opcode::Res => return Err(Error::ReservedOpcode(instruction)),
        }
        Ok(Flow::Continue)
    }

    /// Branch to a PC-relative address if conditions are met.
    ///
    /// Tests the condition flags specified by bits `[11:9]` (N, Z, P):
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
    /// - Bits `[11:9]`: Condition flags (1 = test, 0 = ignore)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn br(&mut self, instruction: u16) {
        let pc_offset = sign_extend(instruction & 0x1FF, 9);
        let nzp = (instruction >> 9) & 0x7;

        if self.registers.cond.matches(nzp) {
            // PC-relative targets wrap within the 16-bit address space.
            self.registers.pc = self.registers.pc.wrapping_add(pc_offset);
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
    fn add(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr1 = (instruction >> 6) & 0x7;
        let imm_flag = (instruction >> 5) & 0x1;

        // Two's-complement addition is modular over 16 bits.
        let value = if imm_flag == 1 {
            let imm5 = sign_extend(instruction & 0x1F, 5);
            self.registers.get(sr1).wrapping_add(imm5)
        } else {
            let sr2 = instruction & 0x7;
            self.registers
                .get(sr1)
                .wrapping_add(self.registers.get(sr2))
        };

        self.registers.set(dr, value);
        self.registers.update_flags(dr);
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
    /// - Bits `[11:9]`: Destination register (DR)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn ld(&mut self, instruction: u16) -> Result<(), Error> {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let address = self.registers.pc.wrapping_add(pc_offset);
        let value = self.read_memory(address)?;
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
        Ok(())
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
    /// - Bits `[11:9]`: Source register (SR)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn st(&mut self, instruction: u16) {
        let sr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let address = self.registers.pc.wrapping_add(pc_offset);
        self.memory.write(address, self.registers.get(sr));
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
    /// - Bit `[11]`: Mode selector (1 = PC-relative, 0 = Base register)
    /// - PC-relative mode:
    ///   - Bits `[10:0]`: 11-bit signed offset (sign-extended to 16 bits)
    /// - Base register mode:
    ///   - Bits `[8:6]`: 3-bit base register identifier
    fn jsr(&mut self, instruction: u16) {
        let pc_offset = sign_extend(instruction & 0x7FF, 11);
        // Read the base register before R7 is overwritten, so `JSRR R7` jumps to
        // the original base value rather than the return address.
        let base = self.registers.get((instruction >> 6) & 0x7);
        let use_offset = (instruction >> 11) & 1 != 0;
        let return_address = self.registers.pc;

        self.registers.pc = if use_offset {
            return_address.wrapping_add(pc_offset)
        } else {
            base
        };
        self.registers.set(7, return_address);
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
    fn and(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr1 = (instruction >> 6) & 0x7;
        let imm_flag = (instruction >> 5) & 0x1;

        let value = if imm_flag == 1 {
            let imm5 = sign_extend(instruction & 0x1F, 5);
            self.registers.get(sr1) & imm5
        } else {
            let sr2 = instruction & 0x7;
            self.registers.get(sr1) & self.registers.get(sr2)
        };

        self.registers.set(dr, value);
        self.registers.update_flags(dr);
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
    /// - Bits `[11:9]`: Destination register (DR)
    /// - Bits `[8:6]`: Base register (BaseR)
    /// - Bits `[5:0]`: 6-bit signed offset (sign-extended to 16 bits)
    fn ldr(&mut self, instruction: u16) -> Result<(), Error> {
        let dr = (instruction >> 9) & 0x7;
        let base_reg = (instruction >> 6) & 0x7;
        let offset = sign_extend(instruction & 0x3F, 6);

        let address = self.registers.get(base_reg).wrapping_add(offset);
        let value = self.read_memory(address)?;
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
        Ok(())
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
    /// - Bits `[11:9]`: Source register (SR)
    /// - Bits `[8:6]`: Base register (BaseR)
    /// - Bits `[5:0]`: 6-bit signed offset (sign-extended to 16 bits)
    fn str(&mut self, instruction: u16) {
        let sr = (instruction >> 9) & 0x7;
        let base_reg = (instruction >> 6) & 0x7;
        let offset = sign_extend(instruction & 0x3F, 6);

        let address = self.registers.get(base_reg).wrapping_add(offset);
        self.memory.write(address, self.registers.get(sr));
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
    /// - Bits `[11:9]`: Destination register (DR)
    /// - Bits `[8:6]`: Source register (SR)
    fn not(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let sr = (instruction >> 6) & 0x7;

        self.registers.set(dr, !self.registers.get(sr));
        self.registers.update_flags(dr);
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
    /// - Bits `[11:9]`: Destination register (DR)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn ldi(&mut self, instruction: u16) -> Result<(), Error> {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let pointer = self.registers.pc.wrapping_add(pc_offset);
        let address = self.read_memory(pointer)?;
        let value = self.read_memory(address)?;
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
        Ok(())
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
    /// - Bits `[11:9]`: Source register (SR)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn sti(&mut self, instruction: u16) -> Result<(), Error> {
        let sr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let pointer = self.registers.pc.wrapping_add(pc_offset);
        let address = self.read_memory(pointer)?;
        self.memory.write(address, self.registers.get(sr));
        Ok(())
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
    fn jmp(&mut self, instruction: u16) {
        // `base_reg` is an arbitrary register, or R7 (`111`) for the `RET` case.
        let base_reg = (instruction >> 6) & 0x7;
        self.registers.pc = self.registers.get(base_reg);
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
    /// - Bits `[11:9]`: Destination register (DR)
    /// - Bits `[8:0]`: 9-bit signed offset (sign-extended to 16 bits)
    fn lea(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let address = self.registers.pc.wrapping_add(pc_offset);
        self.registers.set(dr, address);
        self.registers.update_flags(dr);
    }

    /// Executes a trap service routine for I/O operations.
    ///
    /// # Behavior
    /// - Saves the incremented PC in R7
    /// - Jumps to the trap handler address from the trap vector table
    /// - Handles 6 predefined system calls (GETC, OUT, PUTS, IN, PUTSP, HALT)
    ///
    /// # Trap Vector Mapping
    /// - 0x20 (GETC): Read single character to R0
    /// - 0x21 (OUT): Write character from R0
    /// - 0x22 (PUTS): Write null-terminated string
    /// - 0x23 (IN): Prompt and read character
    /// - 0x24 (PUTSP): Write packed byte string
    /// - 0x25 (HALT): Terminate execution
    fn trap(&mut self, instruction: u16) -> Result<Flow, Error> {
        let code = TrapVector::try_from(instruction & 0xFF).map_err(Error::UnknownTrap)?;

        match code {
            TrapVector::Getc => {
                let key = console::read_char()?;
                self.registers.set(0, u16::from(key));
            }

            TrapVector::Out => {
                let mut stdout = io::stdout().lock();
                stdout.write_all(&[self.registers.get(0).to_le_bytes()[0]])?;
                stdout.flush()?;
            }

            TrapVector::Puts => {
                let mut stdout = io::stdout().lock();
                let mut addr = self.registers.get(0);
                loop {
                    let word = self.read_memory(addr)?;
                    if word == 0 {
                        break;
                    }
                    stdout.write_all(&[word.to_le_bytes()[0]])?;
                    addr = addr.wrapping_add(1);
                }
                stdout.flush()?;
            }

            TrapVector::In => {
                let mut stdout = io::stdout().lock();
                stdout.write_all(b"Enter a character: ")?;
                stdout.flush()?;

                let key = console::read_char()?;
                // Raw mode disables terminal echo, so the trap echoes the key.
                stdout.write_all(&[key])?;
                stdout.flush()?;

                self.registers.set(0, u16::from(key));
            }

            TrapVector::Putsp => {
                let mut stdout = io::stdout().lock();
                let mut addr = self.registers.get(0);
                loop {
                    let word = self.read_memory(addr)?;
                    if word == 0 {
                        break;
                    }
                    // Two characters per word: low byte first, then the high
                    // byte unless it is the null padding of an odd-length string.
                    let [low, high] = word.to_le_bytes();
                    stdout.write_all(&[low])?;
                    if high != 0 {
                        stdout.write_all(&[high])?;
                    }
                    addr = addr.wrapping_add(1);
                }
                stdout.flush()?;
            }

            TrapVector::Halt => return Ok(Flow::Halt),
        }

        Ok(Flow::Continue)
    }
}

/// How execution should proceed after a single instruction.
#[derive(Debug, Clone, Copy)]
enum Flow {
    /// Continue with the next instruction.
    Continue,
    /// Stop: the program reached a `HALT`.
    Halt,
}

#[cfg(test)]
mod tests {
    use lc3core::{ConditionFlag, ObjectFile, PC_START};

    use super::Flow;
    use crate::{Error, Lc3VM};

    /// Loads `words` consecutively from `PC_START`, ready for [`Lc3VM::run`].
    fn vm_with(words: &[u16]) -> Lc3VM {
        let mut vm = Lc3VM::new();
        for (offset, &word) in words.iter().enumerate() {
            let address = PC_START.wrapping_add(u16::try_from(offset).expect("program fits"));
            vm.memory.write(address, word);
        }
        vm
    }

    #[test]
    fn add_immediate_executes_and_sets_condition_code() {
        // ADD R0, R0, #5 ; TRAP HALT
        let mut vm = vm_with(&[0x1025, 0xF025]);
        assert!(vm.run().is_ok());
        assert_eq!(vm.registers.get(0), 5);
        assert_eq!(vm.registers.cond, ConditionFlag::Positive);
    }

    #[test]
    fn halt_stops_the_machine_without_exiting_the_process() {
        let mut vm = vm_with(&[0xF025]);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn ordinary_instruction_continues() {
        let mut vm = Lc3VM::new();
        // ADD R0, R0, #0 — a no-op that must let execution continue.
        assert!(matches!(vm.step(0x1020), Ok(Flow::Continue)));
    }

    #[test]
    fn rti_outside_supervisor_mode_is_rejected() {
        let mut vm = Lc3VM::new();
        assert!(matches!(
            vm.step(0x8000),
            Err(Error::PrivilegedInstruction(0x8000))
        ));
    }

    #[test]
    fn reserved_opcode_is_rejected() {
        let mut vm = Lc3VM::new();
        assert!(matches!(
            vm.step(0xD000),
            Err(Error::ReservedOpcode(0xD000))
        ));
    }

    #[test]
    fn unknown_trap_vector_is_rejected() {
        let mut vm = Lc3VM::new();
        // TRAP xFF is not one of the six standard vectors.
        assert!(matches!(vm.step(0xF0FF), Err(Error::UnknownTrap(0xFF))));
    }

    #[test]
    fn load_places_words_at_origin_and_points_pc_there() {
        let image = ObjectFile {
            origin: 0x3000,
            words: vec![0x1234, 0x5678],
        };
        let vm = Lc3VM::load(&image).expect("image fits");

        assert_eq!(vm.registers.pc, 0x3000);
        assert_eq!(vm.memory.read(0x3000), 0x1234);
        assert_eq!(vm.memory.read(0x3001), 0x5678);
    }

    #[test]
    fn load_rejects_image_that_overruns_memory() {
        // Two words at 0xFFFF would need 0xFFFF and 0x10000; the latter does
        // not exist, so the image must be rejected rather than wrapping.
        let image = ObjectFile {
            origin: 0xFFFF,
            words: vec![0x0001, 0x0002],
        };
        assert!(matches!(
            Lc3VM::load(&image),
            Err(Error::ProgramOutOfRange {
                origin: 0xFFFF,
                words: 2
            })
        ));
    }
}
