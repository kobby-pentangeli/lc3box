use std::fs::File;
use std::io::{self, BufReader, Read as _, Write as _};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};
use lc3core::{KBDR, KBSR, Opcode, TrapVector, sign_extend};

use crate::{Memory, Registers};

/// The main LC-3 emulator.
///
/// # Memory Architecture
/// - 16-bit address space (0x0000-0xFFFF)
/// - First 0xFE00 addresses: general purpose memory
/// - 0xFE00-0xFFFF: Memory-mapped I/O registers
///
/// # Execution Flow
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

    pub fn init_from_program(path: &Path) -> anyhow::Result<Self> {
        let mut vm = Self::new();

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let base_address = reader.read_u16::<BigEndian>()?;
        let mut address = base_address;

        loop {
            match reader.read_u16::<BigEndian>() {
                Ok(instruction) => {
                    vm.write_memory(address, instruction);
                    address = address.wrapping_add(1);
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(vm)
    }

    pub fn execute_program(&mut self) {
        loop {
            let inst = self.read_memory(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
            self.execute_instruction(inst)
        }
    }

    /// Loads the program `instruction` into the VM at the given memory `address`.
    fn write_memory(&mut self, address: u16, instruction: u16) {
        self.memory.write(address, instruction);
    }

    /// Retrieves a program instruction from the specified memory `address`.
    fn read_memory(&mut self, address: u16) -> u16 {
        if address == KBSR {
            self.handle_keyboard();
        }
        self.memory.read(address)
    }

    fn handle_keyboard(&mut self) {
        let mut buf = [0; 1];
        io::stdin()
            .read_exact(&mut buf)
            .expect("error reading from stdin");

        if buf[0] != 0 {
            self.write_memory(KBSR, 1 << 15);
            self.write_memory(KBDR, u16::from(buf[0]));
        } else {
            self.write_memory(KBSR, 0);
        }
    }

    /// Executes a single LC-3 instruction.
    ///
    /// # Execution flow
    /// 1. Decode instruction using first 4 bits as opcode
    /// 2. Dispatch to appropriate instruction handler
    /// 3. Handle invalid opcodes via VM error reporting
    fn execute_instruction(&mut self, instruction: u16) {
        match Opcode::decode(instruction) {
            Opcode::Br => self.br(instruction),
            Opcode::Add => self.add(instruction),
            Opcode::Ld => self.ld(instruction),
            Opcode::St => self.st(instruction),
            Opcode::Jsr => self.jsr(instruction),
            Opcode::And => self.and(instruction),
            Opcode::Ldr => self.ldr(instruction),
            Opcode::Str => self.str(instruction),
            Opcode::Not => self.not(instruction),
            Opcode::Ldi => self.ldi(instruction),
            Opcode::Sti => self.sti(instruction),
            Opcode::Jmp => self.jmp(instruction),
            Opcode::Lea => self.lea(instruction),
            Opcode::Trap => self.trap(instruction),
            Opcode::Rti | Opcode::Res => {}
        }
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
    fn ld(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let address = self.registers.pc.wrapping_add(pc_offset);
        let value = self.read_memory(address);
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
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
        self.write_memory(address, self.registers.get(sr));
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
    fn ldr(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let base_reg = (instruction >> 6) & 0x7;
        let offset = sign_extend(instruction & 0x3F, 6);

        let address = self.registers.get(base_reg).wrapping_add(offset);
        let value = self.read_memory(address);
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
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
        self.write_memory(address, self.registers.get(sr));
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
    fn ldi(&mut self, instruction: u16) {
        let dr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let pointer = self.registers.pc.wrapping_add(pc_offset);
        let address = self.read_memory(pointer);
        let value = self.read_memory(address);
        self.registers.set(dr, value);
        self.registers.update_flags(dr);
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
    fn sti(&mut self, instruction: u16) {
        let sr = (instruction >> 9) & 0x7;
        let pc_offset = sign_extend(instruction & 0x1FF, 9);

        let pointer = self.registers.pc.wrapping_add(pc_offset);
        let address = self.read_memory(pointer);
        self.write_memory(address, self.registers.get(sr));
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
    fn trap(&mut self, instruction: u16) {
        let code = TrapVector::try_from(instruction & 0xFF).expect("invalid trap vector");

        match code {
            TrapVector::Getc => {
                let mut buf = [0; 1];
                io::stdin()
                    .read_exact(&mut buf)
                    .expect("error reading from stdin");
                self.registers.set(0, u16::from(buf[0]));
            }

            TrapVector::Out => {
                let mut stdout = io::stdout().lock();
                let c = self.registers.get(0) as u8 as char;
                write!(stdout, "{c}").expect("failed to write to stdout");
                stdout.flush().expect("failed to flush stdout");
            }

            TrapVector::Puts => {
                let mut stdout = io::stdout().lock();
                let mut addr = self.registers.get(0);
                loop {
                    let c = self.read_memory(addr);
                    if c == 0 {
                        break;
                    }
                    write!(stdout, "{}", c as u8 as char).expect("failed to write to stdout");
                    addr = addr.wrapping_add(1);
                }
                stdout.flush().expect("failed to flush stdout");
            }

            TrapVector::In => {
                print!("Enter a character: ");
                io::stdout().flush().expect("failed to flush stdout");

                let mut buf = [0; 1];
                io::stdin()
                    .read_exact(&mut buf)
                    .expect("error reading from stdin");

                self.registers.set(0, u16::from(buf[0]));
            }

            TrapVector::Putsp => {
                let mut stdout = io::stdout().lock();
                let mut addr = self.registers.get(0);
                loop {
                    let c = self.read_memory(addr);
                    if c == 0 {
                        break;
                    }
                    let c1 = (c & 0xFF) as u8 as char;
                    write!(stdout, "{c1}").expect("failed to write to stdout");
                    let c2 = (c >> 8) as u8 as char;
                    if c2 != '\0' {
                        write!(stdout, "{c2}").expect("failed to write to stdout");
                    }
                    addr = addr.wrapping_add(1);
                }
                stdout.flush().expect("failed to flush stdout");
            }

            TrapVector::Halt => {
                println!("\nHALT detected!");
                io::stdout().flush().expect("failed to flush stdout");
                std::process::exit(1);
            }
        }
    }
}
