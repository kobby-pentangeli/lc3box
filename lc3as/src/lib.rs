//! Assembler for the Little Computer 3 (LC-3).
//!
//! `lc3as` translates LC-3 assembly source into the big-endian `.obj` object
//! files that the `lc3vm` virtual machine executes. It is a two-pass assembler:
//! a first pass lays out the location counter and builds the symbol table, and a
//! second pass resolves label references and encodes each statement into LC-3
//! machine words. Instruction encoding goes through the shared `lc3core`
//! instruction-set kernel, so the assembler and the virtual machine agree on
//! every bit of the instruction set.
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

mod codegen;
mod error;
mod lexer;
mod parser;

pub use codegen::{Image, assemble};
pub use error::{AsmError, LexError, ParseError};
pub(crate) use lexer::{Token, TokenKind, tokenize};
pub(crate) use parser::{Fill, Operation, Segment, Statement, Target, parse};
