use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use lc3core::ObjectFile;
use lc3dsm::disassemble;

#[derive(Parser)]
#[command(name = "lc3dsm")]
#[command(about = "An LC-3 disassembler", long_about = None)]
struct Cli {
    /// Path to the LC-3 object file
    path: PathBuf,
    /// Path to write the assembly listing (defaults to standard output)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let bytes = std::fs::read(&cli.path)?;
    let object = ObjectFile::from_be_bytes(&bytes)?;
    let listing = disassemble(&object);
    match cli.output {
        Some(path) => std::fs::write(path, listing)?,
        None => print!("{listing}"),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lc3dsm: {error}");
            ExitCode::FAILURE
        }
    }
}
