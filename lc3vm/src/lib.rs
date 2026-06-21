//! Virtual machine for the Little Computer 3 (LC-3).
//!
//! `lc3vm` executes pre-assembled LC-3 object files against the instruction-set
//! kernel in [`lc3core`]. It owns the runtime state---[`Memory`] and
//! [`Registers`]---and the fetch–decode–execute loop in [`Lc3VM`].
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

mod console;
mod error;
mod memory;
mod registers;
mod vm;

pub use console::RawMode;
pub use error::Error;
pub use memory::Memory;
pub use registers::Registers;
pub use vm::Lc3VM;
