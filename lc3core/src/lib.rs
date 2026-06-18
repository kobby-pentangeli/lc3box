//! Shared instruction-set kernel for the Little Computer 3 (LC-3) toolbox.
//!
//! `lc3core` defines the opcode set and instruction encoding, the
//! register and condition-code model, the trap vectors, the memory-map
//! constants, the `.obj` object-file format, and the assembly pseudo-ops---
//! the vocabulary every tool in the toolbox shares.

mod object;
mod opcode;
mod pseudo;
mod register;
mod trap;

pub mod memory;

pub use memory::{KBDR, KBSR, MEMORY_SIZE, PC_START};
pub use object::{ObjectError, ObjectFile};
pub use opcode::Opcode;
pub use pseudo::PseudoOp;
pub use register::{ConditionFlag, GP_REGISTER_COUNT};
pub use trap::TrapVector;

/// Sign-extends the low `bit_count` bits of `value` to a full 16-bit word.
///
/// The value is treated as a two's-complement integer occupying its low
/// `bit_count` bits: if the most significant of those bits is set the result is
/// padded with ones, otherwise with zeros.
///
/// The function is total: a `bit_count` of zero or of 16 or more leaves `value`
/// unchanged, since neither case has a sign bit to extend.
pub const fn sign_extend(value: u16, bit_count: u32) -> u16 {
    if bit_count == 0 || bit_count >= u16::BITS {
        return value;
    }

    let field_mask = (1 << bit_count) - 1;
    let magnitude = value & field_mask;
    let sign_bit = 1 << (bit_count - 1);

    if magnitude & sign_bit != 0 {
        magnitude | !field_mask
    } else {
        magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::sign_extend;

    #[test]
    fn sign_extend_extends_negative_values_with_ones() {
        // 0x1FF is -1 in nine bits; 0x10 is -16 in five bits.
        assert_eq!(sign_extend(0x1FF, 9), 0xFFFF);
        assert_eq!(sign_extend(0x10, 5), 0xFFF0);
    }

    #[test]
    fn sign_extend_leaves_positive_values_unchanged() {
        assert_eq!(sign_extend(0x0FF, 9), 0x00FF);
        assert_eq!(sign_extend(0x0F, 5), 0x000F);
    }

    #[test]
    fn sign_extend_ignores_bits_above_the_field_width() {
        // A set bit outside the five-bit field must not leak into the result.
        assert_eq!(sign_extend(0xFFE0, 5), 0x0000);
    }

    #[test]
    fn sign_extend_degenerate_widths_are_identities() {
        assert_eq!(sign_extend(0xABCD, 0), 0xABCD);
        assert_eq!(sign_extend(0xABCD, 16), 0xABCD);
    }
}
