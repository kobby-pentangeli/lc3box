use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use byteorder::{BigEndian, ReadBytesExt};
use clap::Parser;
use lc3vm::VM;
use termios::{ECHO, ICANON, TCSANOW, Termios, tcsetattr};

#[derive(Parser)]
#[command(name = "lc3box")]
#[command(about = "An LC-3 Virtual Machine", long_about = None)]
struct Cli {
    /// Path to the LC-3 object file
    path: PathBuf,
}

/// RAII guard that restores terminal settings on drop
struct TerminalGuard {
    original: Termios,
}

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
        let original = Termios::from_fd(libc::STDIN_FILENO)?;

        let mut raw = original;
        raw.c_lflag &= !(ICANON | ECHO);
        tcsetattr(libc::STDIN_FILENO, TCSANOW, &raw)?;

        Ok(Self { original })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = tcsetattr(libc::STDIN_FILENO, TCSANOW, &self.original);
    }
}

fn load_program(vm: &mut VM, path: &Path) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let base_address = reader.read_u16::<BigEndian>()?;
    let mut address = base_address as usize;

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(instruction) => {
                vm.write_memory(address, instruction);
                address += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let _guard = TerminalGuard::new()?;

    let mut vm = VM::new();
    load_program(&mut vm, &cli.path)?;
    lc3vm::execute_program(&mut vm);

    Ok(())
}

fn main() {
    // Ensure terminal is restored even on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Restore terminal before printing panic
        let _ = Termios::from_fd(libc::STDIN_FILENO).map(|t| {
            let _ = tcsetattr(libc::STDIN_FILENO, TCSANOW, &t);
        });
        original_hook(info);
    }));

    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
