use std::io::{self, Read as _, Write as _};

use crate::VM;

/// Operation codes for the LC-3.
///
/// LC-3 has 16 opcodes. Each instruction is 16 bits long, with
/// first 4 bits storing this [`Opcode`], and the rest of the bits are
/// reserved for the parameters.
#[derive(Debug)]
pub enum Opcode {
    /// Branch (0x0000)
    Br = 0,

    /// Add (0x0001)
    Add,

    /// Load (0x0010)
    Ld,

    /// Store (0x0011)
    St,

    /// Jump to subroutine (0x0100)
    Jsr,

    /// Bitwise AND (0x0101)
    And,

    /// Load register (0x0110)
    Ldr,

    /// Store register (0x0111)
    Str,

    /// Unused for now (0x1000)
    Rti,

    /// Bitwise NOT (0x1001)
    Not,

    /// Load indirect (0x1010)
    Ldi,

    /// Store indirect (0x1011)
    Sti,

    /// Jump (0x1100)
    Jmp,

    /// Reserved and unused for now (0x1101)
    Res,

    /// Load effective address (0x1110)
    Lea,

    /// Execute trap codes (0x1111)
    Trap,
}

/// TRAP Opcodes
pub enum Trapcode {
    /// Get character from keyboard
    Getc = 0x20,

    /// Output a character
    Out = 0x21,

    /// Output a word string
    Puts = 0x22,

    /// Input a string, i.e., print a prompt on the screen and
    /// read a single character from the keyboard.
    /// The character is echoed onto the console monitor, and its ASCII code is
    /// copied into R0. The high eight bits of R0 are cleared.
    In = 0x23,

    /// Output a byte string
    Putsp = 0x24,

    /// Halt the program
    Halt = 0x25,
}

impl Opcode {
    /// Extract the leftmost 4 bits of the given instruction.
    ///
    /// Since each instruction is 16 bits long (with the leftmost 4 storing the opcode),
    /// the extraction is achieved by shifting the bits 12 positions to the right.
    pub fn get(instruction: u16) -> Option<Self> {
        match instruction >> 12 {
            0 => Some(Self::Br),
            1 => Some(Self::Add),
            2 => Some(Self::Ld),
            3 => Some(Self::St),
            4 => Some(Self::Jsr),
            5 => Some(Self::And),
            6 => Some(Self::Ldr),
            7 => Some(Self::Str),
            8 => Some(Self::Rti),
            9 => Some(Self::Not),
            10 => Some(Self::Ldi),
            11 => Some(Self::Sti),
            12 => Some(Self::Jmp),
            13 => Some(Self::Res),
            14 => Some(Self::Lea),
            15 => Some(Self::Trap),
            _ => None,
        }
    }
}

impl Trapcode {
    /// Executes a trap service routine.
    ///
    /// # Trap Vector Mapping
    /// - 0x20 (GETC): Read single character to R0
    /// - 0x21 (OUT): Write character from R0
    /// - 0x22 (PUTS): Write null-terminated string
    /// - 0x23 (IN): Prompt and read character
    /// - 0x24 (PUTSP): Write packed byte string
    /// - 0x25 (HALT): Terminate execution
    pub(crate) fn execute(instruction: u16, vm: &mut VM) {
        let code = Self::try_from(instruction & 0xFF).expect("invalid trapcode");

        match code {
            Self::Getc => {
                let mut buf = [0; 1];
                io::stdin()
                    .read_exact(&mut buf)
                    .expect("error reading from stdin");
                vm.registers.r0 = buf[0] as u16;
            }

            Self::Out => {
                let mut stdout = io::stdout().lock();
                let c = vm.registers.r0 as u8 as char;
                write!(stdout, "{c}").expect("failed to write to stdout");
                stdout.flush().expect("failed to flush stdout");
            }

            Self::Puts => {
                let mut stdout = io::stdout().lock();
                let mut addr = vm.registers.r0;
                loop {
                    let c = vm.read_memory(addr);
                    if c == 0 {
                        break;
                    }
                    write!(stdout, "{}", c as u8 as char).expect("failed to write to stdout");
                    addr += 1;
                }
                stdout.flush().expect("failed to flush stdout");
            }

            Self::In => {
                print!("Enter a character: ");
                io::stdout().flush().expect("failed to flush stdout");

                let mut buf = [0; 1];
                io::stdin()
                    .read_exact(&mut buf)
                    .expect("error reading from stdin");

                vm.registers.update(0, buf[0] as u16);
            }

            Self::Putsp => {
                let mut stdout = io::stdout().lock();
                let mut addr = vm.registers.r0;
                loop {
                    let c = vm.read_memory(addr);
                    if c == 0 {
                        break;
                    }
                    let c1 = (c & 0xFF) as u8 as char;
                    write!(stdout, "{c1}").expect("failed to write to stdout");
                    let c2 = (c >> 8) as u8 as char;
                    if c2 != '\0' {
                        write!(stdout, "{c2}").expect("failed to write to stdout");
                    }
                    addr += 1;
                }
                stdout.flush().expect("failed to flush stdout");
            }

            Self::Halt => {
                println!("\nHALT detected!");
                io::stdout().flush().expect("failed to flush stdout");
                std::process::exit(1);
            }
        }
    }
}

impl TryFrom<u16> for Trapcode {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Self::Getc),
            0x21 => Ok(Self::Out),
            0x22 => Ok(Self::Puts),
            0x23 => Ok(Self::In),
            0x24 => Ok(Self::Putsp),
            0x25 => Ok(Self::Halt),
            _ => Err(value),
        }
    }
}
