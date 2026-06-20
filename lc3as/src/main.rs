use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use lc3as::{Image, assemble};

#[derive(Parser)]
#[command(name = "lc3as")]
#[command(about = "An LC-3 assembler", long_about = None)]
struct Cli {
    /// Path to the LC-3 assembly source file
    path: PathBuf,
    /// Path to write the object file (defaults to the source path with a `.obj`
    /// extension); a multi-segment program writes one file per segment
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let source = std::fs::read_to_string(&cli.path)?;
    let image = assemble(&source)?;
    let output = cli.output.unwrap_or_else(|| cli.path.with_extension("obj"));
    for path in write_image(&image, &output)? {
        println!("{}", path.display());
    }
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lc3as: {error}");
            ExitCode::FAILURE
        }
    }
}
