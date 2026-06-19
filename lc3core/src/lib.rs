//! Shared instruction-set kernel for the Little Computer 3 (LC-3) toolbox.
//!
//! `lc3core` defines the opcode set, mnemonic and register parsing, the
//! register and condition-code model, the trap vectors and their aliases, the
//! memory-map constants, the `.obj` object-file format, the assembly pseudo-ops,
//! and the signed and unsigned field helpers---the vocabulary every tool in the
//! toolbox shares.

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
pub use register::{ConditionFlag, GP_REGISTER_COUNT, parse_branch_condition, parse_register};
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

/// Narrows a signed integer into a two's-complement field of `bit_count` bits.
///
/// The inverse of [`sign_extend`]: it takes a full-width signed value and, when
/// that value is representable in a `bit_count`-bit two's-complement field
/// (`-2^(bit_count-1)..=2^(bit_count-1)-1`), returns those low bits---the
/// encoding an assembler writes into an instruction. A value outside the field
/// yields `None`, so an out-of-range immediate or PC-relative offset is reported
/// rather than silently truncated.
///
/// `bit_count` must be in `1..=16`; any other width yields `None`.
pub const fn signed_field(value: i32, bit_count: u32) -> Option<u16> {
    if bit_count == 0 || bit_count > u16::BITS {
        return None;
    }

    let max = (1i32 << (bit_count - 1)) - 1;
    let min = -(1i32 << (bit_count - 1));
    if value < min || value > max {
        return None;
    }

    let mask = ((1u32 << bit_count) - 1) as u16;
    Some((value as u16) & mask)
}

/// Narrows a non-negative integer into an unsigned field of `bit_count` bits.
///
/// Used for fields the architecture reads as unsigned, such as the eight-bit
/// trap vector. Returns the low `bit_count` bits when `value` fits in
/// `0..=2^bit_count-1`, and `None` otherwise so an out-of-range value is
/// rejected rather than truncated.
///
/// `bit_count` must be in `1..=16`; any other width yields `None`.
pub const fn unsigned_field(value: i32, bit_count: u32) -> Option<u16> {
    if bit_count == 0 || bit_count > u16::BITS {
        return None;
    }

    let max = ((1u32 << bit_count) - 1) as i32;
    if value < 0 || value > max {
        return None;
    }

    Some(value as u16)
}

#[cfg(test)]
mod tests {
    use super::{sign_extend, signed_field, unsigned_field};

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

    #[test]
    fn signed_field_accepts_its_full_range_and_rejects_just_outside() {
        // imm5 spans -16..=15; offset9 spans -256..=255.
        assert_eq!(signed_field(15, 5), Some(0x0F));
        assert_eq!(signed_field(-16, 5), Some(0x10));
        assert_eq!(signed_field(16, 5), None);
        assert_eq!(signed_field(-17, 5), None);
        assert_eq!(signed_field(255, 9), Some(0x0FF));
        assert_eq!(signed_field(-256, 9), Some(0x100));
        assert_eq!(signed_field(256, 9), None);
    }

    #[test]
    fn signed_field_round_trips_through_sign_extend() {
        // Encoding then sign-extending must recover the original signed value.
        for &(value, width) in &[(-16i32, 5), (15, 5), (-256, 9), (255, 9), (-1024, 11)] {
            let encoded = signed_field(value, width).expect("value fits the field");
            assert_eq!(sign_extend(encoded, width) as i16 as i32, value);
        }
    }

    #[test]
    fn signed_field_rejects_degenerate_widths() {
        assert_eq!(signed_field(0, 0), None);
        assert_eq!(signed_field(0, 17), None);
    }

    #[test]
    fn unsigned_field_accepts_its_range_and_rejects_outside() {
        // trapvect8 spans 0..=255.
        assert_eq!(unsigned_field(0x25, 8), Some(0x25));
        assert_eq!(unsigned_field(255, 8), Some(0xFF));
        assert_eq!(unsigned_field(256, 8), None);
        assert_eq!(unsigned_field(-1, 8), None);
    }
}
