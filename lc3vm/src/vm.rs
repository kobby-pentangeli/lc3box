use std::path::Path;

use lc3core::{KBDR, KBSR, ObjectFile, Opcode, TrapVector, sign_extend};

use crate::console::{Console, StdConsole};
use crate::{Error, Memory, Registers};

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
    /// Console the traps and keyboard poll read from and write to.
    console: Box<dyn Console>,
}

impl Default for Lc3VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Lc3VM {
    /// Creates a new VM in its initial state, driving the real terminal.
    pub fn new() -> Self {
        Self::with_console(Box::new(StdConsole::new()))
    }

    /// Creates a new VM in its initial state over the given `console`.
    pub(crate) fn with_console(console: Box<dyn Console>) -> Self {
        Self {
            memory: Memory::new(),
            registers: Registers::new(),
            console,
        }
    }

    /// Loads the LC-3 object file at `path` and returns a VM ready to run it.
    ///
    /// Returns [`Error::Io`] if the file cannot be read, [`Error::Object`] if
    /// its bytes are not a valid `.obj` image, and [`Error::ProgramOutOfRange`]
    /// if the image would not fit in memory.
    pub fn init_from_program(path: &Path) -> Result<Self, Error> {
        let mut vm = Self::new();
        vm.load_image(&ObjectFile::from_be_bytes(&std::fs::read(path)?)?)?;
        Ok(vm)
    }

    /// Loads `image` at its origin and points the program counter there, ready
    /// to [`run`](Self::run).
    ///
    /// Returns [`Error::ProgramOutOfRange`] if the image's words would extend
    /// past the top of the address space.
    pub(crate) fn load_image(&mut self, image: &ObjectFile) -> Result<(), Error> {
        self.memory
            .region_mut(image.origin, image.words.len())
            .ok_or(Error::ProgramOutOfRange {
                origin: image.origin,
                words: image.words.len(),
            })?
            .copy_from_slice(&image.words);
        self.registers.pc = image.origin;
        Ok(())
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
        match self.console.poll_char()? {
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
                let key = self.console.read_char()?;
                self.registers.set(0, u16::from(key));
            }

            TrapVector::Out => {
                self.console
                    .write_all(&[self.registers.get(0).to_le_bytes()[0]])?;
                self.console.flush()?;
            }

            TrapVector::Puts => {
                let mut addr = self.registers.get(0);
                loop {
                    let word = self.read_memory(addr)?;
                    if word == 0 {
                        break;
                    }
                    self.console.write_all(&[word.to_le_bytes()[0]])?;
                    addr = addr.wrapping_add(1);
                }
                self.console.flush()?;
            }

            TrapVector::In => {
                self.console.write_all(b"Enter a character: ")?;
                self.console.flush()?;

                let key = self.console.read_char()?;
                // Raw mode disables terminal echo, so the trap echoes the key.
                self.console.write_all(&[key])?;
                self.console.flush()?;

                self.registers.set(0, u16::from(key));
            }

            TrapVector::Putsp => {
                let mut addr = self.registers.get(0);
                loop {
                    let word = self.read_memory(addr)?;
                    if word == 0 {
                        break;
                    }
                    // Two characters per word: low byte first, then the high
                    // byte unless it is the null padding of an odd-length string.
                    let [low, high] = word.to_le_bytes();
                    self.console.write_all(&[low])?;
                    if high != 0 {
                        self.console.write_all(&[high])?;
                    }
                    addr = addr.wrapping_add(1);
                }
                self.console.flush()?;
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
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::io;
    use std::rc::Rc;

    use lc3core::{ConditionFlag, ObjectFile, PC_START};

    use super::Flow;
    use crate::console::Console;
    use crate::{Error, Lc3VM};

    /// In-memory console: scripted input bytes and a shared capture buffer for
    /// everything the VM writes.
    struct BufferConsole {
        input: VecDeque<u8>,
        output: Rc<RefCell<Vec<u8>>>,
    }

    impl BufferConsole {
        fn new(input: &[u8], output: Rc<RefCell<Vec<u8>>>) -> Self {
            Self {
                input: input.iter().copied().collect(),
                output,
            }
        }
    }

    impl Console for BufferConsole {
        fn poll_char(&mut self) -> io::Result<Option<u8>> {
            Ok(self.input.pop_front())
        }

        fn read_char(&mut self) -> io::Result<u8> {
            self.input
                .pop_front()
                .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))
        }

        fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
            self.output.borrow_mut().extend_from_slice(bytes);
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Loads `words` consecutively from `PC_START`, ready for [`Lc3VM::run`].
    fn vm_with(words: &[u16]) -> Lc3VM {
        let mut vm = Lc3VM::new();
        for (offset, &word) in words.iter().enumerate() {
            let address = PC_START.wrapping_add(u16::try_from(offset).expect("program fits"));
            vm.memory.write(address, word);
        }
        vm
    }

    /// A VM wired to a scripted-input console, paired with the shared buffer
    /// the console writes captured output into.
    fn vm_with_console(input: &[u8]) -> (Lc3VM, Rc<RefCell<Vec<u8>>>) {
        let output = Rc::new(RefCell::new(Vec::new()));
        let console = BufferConsole::new(input, Rc::clone(&output));
        (Lc3VM::with_console(Box::new(console)), output)
    }

    /// Decodes one of the workspace example programs.
    fn load_example(name: &str) -> ObjectFile {
        let path = format!("{}/../examples/{name}", env!("CARGO_MANIFEST_DIR"));
        ObjectFile::from_be_bytes(&std::fs::read(&path).expect("example file exists"))
            .expect("example is a valid object file")
    }

    /// Loads `name` over scripted `input` and drives up to `limit` instructions,
    /// stopping early on a clean halt or once the scripted input is exhausted.
    /// Any execution error other than that exhaustion fails the test. Returns
    /// the captured console output.
    fn run_example_bounded(name: &str, input: &[u8], limit: u64) -> Vec<u8> {
        let (mut vm, output) = vm_with_console(input);
        vm.load_image(&load_example(name))
            .expect("example fits in memory");

        for _ in 0..limit {
            let pc = vm.registers.pc;
            let Ok(instruction) = vm.read_memory(pc) else {
                break;
            };
            vm.registers.pc = pc.wrapping_add(1);
            match vm.step(instruction) {
                Ok(Flow::Continue) => {}
                Ok(Flow::Halt) => break,
                Err(Error::Io(error)) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(error) => panic!("{name} raised an execution error: {error}"),
            }
        }

        output.borrow().clone()
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
    fn add_register_mode_sums_two_registers() {
        let mut vm = Lc3VM::new();
        vm.registers.set(1, 7);
        vm.registers.set(2, 35);
        // ADD R0, R1, R2
        vm.step(0x1042).expect("add");
        assert_eq!(vm.registers.get(0), 42);
        assert_eq!(vm.registers.cond, ConditionFlag::Positive);
    }

    #[test]
    fn and_immediate_masks_the_source_register() {
        let mut vm = Lc3VM::new();
        vm.registers.set(1, 0xFFFF);
        // AND R0, R1, #15
        vm.step(0x506F).expect("and");
        assert_eq!(vm.registers.get(0), 0x000F);
        assert_eq!(vm.registers.cond, ConditionFlag::Positive);
    }

    #[test]
    fn not_complements_the_source_register() {
        let mut vm = Lc3VM::new();
        vm.registers.set(1, 0x00FF);
        // NOT R0, R1
        vm.step(0x907F).expect("not");
        assert_eq!(vm.registers.get(0), 0xFF00);
        assert_eq!(vm.registers.cond, ConditionFlag::Negative);
    }

    #[test]
    fn br_branches_only_when_a_condition_flag_matches() {
        let mut vm = Lc3VM::new();
        vm.registers.cond = ConditionFlag::Zero;

        // BRz #5 with the zero flag set: taken.
        vm.registers.pc = 0x3000;
        vm.step(0x0405).expect("br");
        assert_eq!(vm.registers.pc, 0x3005);

        // BRn #5 with the zero flag set: not taken.
        vm.registers.pc = 0x3000;
        vm.step(0x0805).expect("br");
        assert_eq!(vm.registers.pc, 0x3000);
    }

    #[test]
    fn br_applies_a_sign_extended_negative_offset() {
        let mut vm = Lc3VM::new();
        vm.registers.cond = ConditionFlag::Positive;
        vm.registers.pc = 0x3005;
        // BRp #-5
        vm.step(0x03FB).expect("br");
        assert_eq!(vm.registers.pc, 0x3000);
    }

    #[test]
    fn jmp_sets_the_pc_to_the_base_register() {
        let mut vm = Lc3VM::new();
        vm.registers.set(2, 0x8000);
        // JMP R2
        vm.step(0xC080).expect("jmp");
        assert_eq!(vm.registers.pc, 0x8000);
    }

    #[test]
    fn jsr_jumps_and_saves_the_return_address() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        // JSR #100
        vm.step(0x4864).expect("jsr");
        assert_eq!(vm.registers.pc, 0x3064);
        assert_eq!(vm.registers.get(7), 0x3000);
    }

    #[test]
    fn jsrr_reads_the_base_register_before_overwriting_r7() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        vm.registers.set(7, 0x4000);
        // JSRR R7 jumps to the old R7, not the freshly saved return address.
        vm.step(0x41C0).expect("jsrr");
        assert_eq!(vm.registers.pc, 0x4000);
        assert_eq!(vm.registers.get(7), 0x3000);
    }

    #[test]
    fn lea_loads_the_effective_address_without_dereferencing() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        // LEA R7, #10
        vm.step(0xEE0A).expect("lea");
        assert_eq!(vm.registers.get(7), 0x300A);
    }

    #[test]
    fn ld_loads_from_a_pc_relative_address() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        vm.memory.write(0x3005, 0xBEEF);
        // LD R3, #5
        vm.step(0x2605).expect("ld");
        assert_eq!(vm.registers.get(3), 0xBEEF);
        assert_eq!(vm.registers.cond, ConditionFlag::Negative);
    }

    #[test]
    fn st_stores_to_a_pc_relative_address() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        vm.registers.set(4, 0x1234);
        // ST R4, #2
        vm.step(0x3802).expect("st");
        assert_eq!(vm.memory.read(0x3002), 0x1234);
    }

    #[test]
    fn ldr_loads_from_base_plus_offset() {
        let mut vm = Lc3VM::new();
        vm.registers.set(1, 0x4000);
        vm.memory.write(0x4003, 0xCAFE);
        // LDR R2, R1, #3
        vm.step(0x6443).expect("ldr");
        assert_eq!(vm.registers.get(2), 0xCAFE);
    }

    #[test]
    fn str_stores_to_base_plus_offset() {
        let mut vm = Lc3VM::new();
        vm.registers.set(1, 0x4000);
        vm.registers.set(2, 0xABCD);
        // STR R2, R1, #1
        vm.step(0x7441).expect("str");
        assert_eq!(vm.memory.read(0x4001), 0xABCD);
    }

    #[test]
    fn ldi_follows_the_pointer_at_the_pc_relative_address() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        vm.memory.write(0x3001, 0x4000);
        vm.memory.write(0x4000, 0x7777);
        // LDI R5, #1
        vm.step(0xAA01).expect("ldi");
        assert_eq!(vm.registers.get(5), 0x7777);
    }

    #[test]
    fn sti_stores_through_the_pointer_at_the_pc_relative_address() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0x3000;
        vm.registers.set(6, 0x9999);
        vm.memory.write(0x3002, 0x5000);
        // STI R6, #2
        vm.step(0xBC02).expect("sti");
        assert_eq!(vm.memory.read(0x5000), 0x9999);
    }

    #[test]
    fn instructions_can_address_the_last_word_of_memory() {
        let mut vm = Lc3VM::new();
        vm.memory.write(0xFFFF, 0x55AA);
        vm.registers.set(1, 0xFFFF);
        // LDR R0, R1, #0 reads 0xFFFF without a bounds panic.
        vm.step(0x6040).expect("ldr");
        assert_eq!(vm.registers.get(0), 0x55AA);
    }

    #[test]
    fn program_counter_wraps_at_the_top_of_memory() {
        let mut vm = Lc3VM::new();
        vm.registers.pc = 0xFFFF;
        vm.memory.write(0xFFFF, 0x0000); // BR with no flags: a no-op
        vm.memory.write(0x0000, 0xF025); // TRAP HALT at the wrapped address
        vm.run()
            .expect("halts after the program counter wraps to 0x0000");
    }

    #[test]
    fn getc_reads_one_key_into_r0() {
        let (mut vm, _output) = vm_with_console(b"Q");
        vm.step(0xF020).expect("getc"); // TRAP GETC
        assert_eq!(vm.registers.get(0), u16::from(b'Q'));
    }

    #[test]
    fn out_writes_the_low_byte_of_r0() {
        let (mut vm, output) = vm_with_console(b"");
        vm.registers.set(0, u16::from(b'Z'));
        vm.step(0xF021).expect("out"); // TRAP OUT
        assert_eq!(output.borrow().as_slice(), b"Z");
    }

    #[test]
    fn puts_writes_a_null_terminated_string() {
        let (mut vm, output) = vm_with_console(b"");
        vm.memory.write(0x4000, u16::from(b'H'));
        vm.memory.write(0x4001, u16::from(b'i'));
        vm.memory.write(0x4002, 0x0000);
        vm.registers.set(0, 0x4000);
        vm.step(0xF022).expect("puts"); // TRAP PUTS
        assert_eq!(output.borrow().as_slice(), b"Hi");
    }

    #[test]
    fn in_prompts_echoes_and_stores_the_key() {
        let (mut vm, output) = vm_with_console(b"k");
        vm.step(0xF023).expect("in"); // TRAP IN
        assert_eq!(vm.registers.get(0), u16::from(b'k'));
        assert_eq!(output.borrow().as_slice(), b"Enter a character: k");
    }

    #[test]
    fn putsp_writes_two_packed_characters_per_word() {
        let (mut vm, output) = vm_with_console(b"");
        // "ABCD" packed low|high per word, then a null terminator word.
        vm.memory
            .write(0x4000, u16::from(b'A') | (u16::from(b'B') << 8));
        vm.memory
            .write(0x4001, u16::from(b'C') | (u16::from(b'D') << 8));
        vm.memory.write(0x4002, 0x0000);
        vm.registers.set(0, 0x4000);
        vm.step(0xF024).expect("putsp"); // TRAP PUTSP
        assert_eq!(output.borrow().as_slice(), b"ABCD");
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
        let mut vm = Lc3VM::new();
        vm.load_image(&image).expect("image fits");

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
        let mut vm = Lc3VM::new();
        assert!(matches!(
            vm.load_image(&image),
            Err(Error::ProgramOutOfRange {
                origin: 0xFFFF,
                words: 2
            })
        ));
    }

    #[test]
    fn hello_world_example_prints_its_greeting() {
        let (mut vm, output) = vm_with_console(b"");
        vm.load_image(&load_example("hello-world.obj"))
            .expect("image fits");
        vm.run().expect("hello-world runs to HALT");
        assert_eq!(output.borrow().as_slice(), b"Hello World!");
    }

    #[test]
    fn game_2048_example_initializes_and_renders() {
        // Drives a few moves; the scripted input runs out via GETC and stops it.
        let output = run_example_bounded("2048.obj", b"wasdq", 50_000_000);
        assert!(!output.is_empty(), "2048 renders its board");
    }

    #[test]
    fn game_rogue_example_initializes_and_renders() {
        let output = run_example_bounded("rogue.obj", b"wasd\r", 50_000_000);
        assert!(!output.is_empty(), "rogue renders its screen");
    }
}
