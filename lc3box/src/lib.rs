//! The LC-3 toolbox as one umbrella crate.
//!
//! `lc3box` is both the unified command-line driver for the Little Computer 3
//! (LC-3) and an umbrella library that re-exports the focused tool crates under
//! short module names, so a dependent can reach the whole toolbox---or a single
//! tool---through one dependency:
//!
//! - [`kernel`] --- the shared instruction-set kernel ([`lc3core`]);
//! - [`vm`] --- the virtual machine ([`lc3vm`]);
//! - [`asm`] --- the assembler ([`lc3as`]);
//! - [`dsm`] --- the disassembler ([`lc3dsm`]).
//!
//! Each module is gated behind a like-named feature; `full` enables all four,
//! and the default `cli` feature additionally builds the `lc3box` binary. Depend
//! on `lc3box` with `default-features = false` and a single feature to pull in
//! one tool alone, or on the individual `lc3core` / `lc3vm` / `lc3as` / `lc3dsm`
//! crates for the most granular build.
//!
//! ```
//! use lc3box::{asm, dsm, vm};
//!
//! // The assembler, disassembler, and VM, all reached through the one crate.
//! let image = asm::assemble(".ORIG x3000\nHALT\n.END\n").expect("assembles");
//! let listing = dsm::disassemble(&image.blocks[0]);
//! assert!(listing.contains("HALT"));
//!
//! let mut machine = vm::Lc3VM::new();
//! machine.load_program(&image.blocks).expect("loads");
//! ```

#[cfg(feature = "asm")]
pub use lc3as as asm;
#[cfg(feature = "kernel")]
pub use lc3core as kernel;
#[cfg(feature = "dsm")]
pub use lc3dsm as dsm;
#[cfg(feature = "vm")]
pub use lc3vm as vm;
