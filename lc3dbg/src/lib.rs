//! Interactive debugger for the Little Computer 3 (LC-3).
//!
//! `lc3dbg` drives an [`lc3vm`] virtual machine one instruction at a time: it
//! single-steps, runs to breakpoints, inspects and edits registers and memory,
//! and renders the program around the counter through [`lc3dsm`]. The
//! [`Debugger`] is the engine---pure state transitions over a loaded program,
//! with no terminal of its own---and [`parse`] turns one line of input into a
//! [`Command`]. [`repl()`] ties the two together into a ready-made terminal
//! frontend over any reader and writer.
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

mod cmd;
mod debug;
mod repl;

pub use cmd::{Command, ParseError, Register, parse};
pub use debug::{Debugger, Stop};
pub use repl::repl;
