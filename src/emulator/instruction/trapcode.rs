use std::io::{self, Read as _, Write as _};

use super::VM;

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
