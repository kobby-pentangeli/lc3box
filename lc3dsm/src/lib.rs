//! Disassembler for the Little Computer 3 (LC-3).
//!
//! `lc3dsm` is the inverse of the `lc3as` assembler: it turns a big-endian
//! `.obj` object file back into a readable, re-assemblable LC-3 assembly
//! listing. It sweeps the image word by word, decodes each word into its
//! instruction form, recovers labels from PC-relative references, and renders
//! the result as assembly framed by `.ORIG`/`.END`, with every line's address
//! and hex encoding carried as a trailing comment. Decoding goes through the
//! shared `lc3core` instruction-set kernel, so the disassembler and the
//! assembler agree on every bit of the instruction set---reassembling a
//! disassembled object reproduces the original image.
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

mod instruction;
mod listing;

pub use instruction::{Instruction, decode};
pub use listing::{disassemble, render_instruction};
