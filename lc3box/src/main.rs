//! Unified command-line driver for the LC-3 toolbox.
//!
//! `lc3box` is the single frontend over the LC-3 tool libraries, with one
//! subcommand per tool delegating to its library core: `run` executes a
//! program on the [`lc3vm`] virtual machine, `asm` assembles a source file
//! through [`lc3as`], `disasm` renders an object file's disassembly through
//! [`lc3dsm`], and `dbg` opens an interactive debugging session through
//! [`lc3dbg`], all built on the shared [`lc3core`] kernel.

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use lc3as::{Image, assemble};
use lc3core::ObjectFile;
use lc3dbg::{Debugger, repl};
use lc3dsm::disassemble;
use lc3vm::{Lc3VM, RawMode};

#[derive(Parser)]
#[command(name = "lc3box", version, about = "An LC-3 toolbox", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run an LC-3 program: an assembly source (`.asm`) is assembled in memory,
    /// an object file (`.obj`) is loaded directly, then executed
    Run {
        /// Path to the LC-3 program, an `.asm` source or an `.obj` object file
        path: PathBuf,
    },
    /// Assemble an LC-3 assembly source file into an object file
    Asm {
        /// Path to the LC-3 assembly source file
        path: PathBuf,
        /// Path to write the object file (defaults to the source path with a
        /// `.obj` extension); a multi-segment program writes one file per segment
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Disassemble an LC-3 object file into a re-assemblable assembly listing
    Disasm {
        /// Path to the LC-3 object file
        path: PathBuf,
        /// Path to write the assembly listing (defaults to standard output)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Debug an LC-3 program interactively: single-step, set breakpoints, and
    /// inspect registers and memory. Accepts an `.asm` source or `.obj` object
    Dbg {
        /// Path to the LC-3 program, an `.asm` source or an `.obj` object file
        path: PathBuf,
    },
}

/// Loads the program at `path` into its object segments, assembling it in memory
/// if it is `.asm` source and decoding it if it is a `.obj` object.
fn load_program(path: &Path) -> Result<Vec<ObjectFile>, Box<dyn Error>> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("asm") => Ok(assemble(&std::fs::read_to_string(path)?)?.blocks),
        Some("obj") => Ok(vec![ObjectFile::from_be_bytes(&std::fs::read(path)?)?]),
        _ => Err(format!("{}: expected a `.asm` or `.obj` file", path.display()).into()),
    }
}

/// Loads the program at `path` and runs it to completion on the virtual machine.
fn run(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut vm = Lc3VM::new();
    vm.load_program(&load_program(path)?)?;

    // Raw mode lasts only for the program's run; the guard restores the
    // terminal when this scope ends, including on an early error return.
    let _raw = RawMode::enable()?;
    vm.run()?;
    Ok(())
}

/// Assembles `path` and writes the resulting object file(s), echoing each path.
fn asm(path: &Path, output: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let source = std::fs::read_to_string(path)?;
    let image = assemble(&source)?;
    let output = output.unwrap_or_else(|| path.with_extension("obj"));
    for written in write_image(&image, &output)? {
        println!("{}", written.display());
    }
    Ok(())
}

/// Disassembles `path`, printing the listing to standard output or writing it
/// to `output`.
fn disasm(path: &Path, output: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    let object = ObjectFile::from_be_bytes(&bytes)?;
    let listing = disassemble(&object);
    match output {
        Some(path) => std::fs::write(path, listing)?,
        None => print!("{listing}"),
    }
    Ok(())
}

/// Loads the program at `path` and opens an interactive debugging session on it,
/// reading commands from standard input and writing results to standard output.
fn dbg(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut debugger = Debugger::new(load_program(path)?)?;
    repl(&mut debugger, io::stdin().lock(), io::stdout().lock())?;
    Ok(())
}

/// Writes each object block to disk, returning the paths written.
///
/// A single-segment program is written to `output`; a multi-segment program
/// writes one file per segment, each suffixed with its origin so the blocks do
/// not collide.
fn write_image(image: &Image, output: &Path) -> io::Result<Vec<PathBuf>> {
    match image.blocks.as_slice() {
        [block] => {
            std::fs::write(output, block.to_be_bytes())?;
            Ok(vec![output.to_path_buf()])
        }
        blocks => blocks
            .iter()
            .map(|block| {
                let path = segment_path(output, block.origin);
                std::fs::write(&path, block.to_be_bytes()).map(|()| path)
            })
            .collect(),
    }
}

/// Derives a per-segment object path from `output` and a segment `origin`.
fn segment_path(output: &Path, origin: u16) -> PathBuf {
    let stem = output.file_stem().map_or_else(
        || String::from("out"),
        |stem| stem.to_string_lossy().into_owned(),
    );
    output.with_file_name(format!("{stem}-{origin:04x}.obj"))
}

fn execute() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Command::Run { path } => run(&path),
        Command::Asm { path, output } => asm(&path, output),
        Command::Disasm { path, output } => disasm(&path, output),
        Command::Dbg { path } => dbg(&path),
    }
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lc3box: {error}");
            ExitCode::FAILURE
        }
    }
}
