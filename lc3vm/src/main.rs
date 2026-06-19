use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use lc3vm::{Lc3VM, RawMode};

#[derive(Parser)]
#[command(name = "lc3vm")]
#[command(about = "An LC-3 Virtual Machine", long_about = None)]
struct Cli {
    /// Path to the LC-3 object file
    path: PathBuf,
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut vm = Lc3VM::init_from_program(&cli.path)?;

    // Raw mode lasts only for the program's run; the guard restores the
    // terminal when this scope ends, including on an early error return.
    let _raw = RawMode::enable()?;
    vm.run()?;

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lc3vm: {error}");
            ExitCode::FAILURE
        }
    }
}
