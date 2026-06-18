use lc3core::{ConditionFlag, GP_REGISTER_COUNT, PC_START};

/// The processor's register state: eight general-purpose registers, the program
/// counter, and the current condition code.
///
/// General-purpose registers are addressed by the 3-bit register field of an
/// instruction. Masking that field to three bits keeps every index within the
/// fixed array, so access is total and never panics.
pub struct Registers {
    gpr: [u16; GP_REGISTER_COUNT],
    /// The program counter: the address of the next instruction to fetch.
    pub pc: u16,
    /// The condition code set by the most recent register-writing instruction.
    pub cond: ConditionFlag,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    /// Creates the initial register state: general-purpose registers cleared,
    /// the program counter at [`PC_START`], and a zero condition code.
    pub fn new() -> Self {
        Self {
            gpr: [0; GP_REGISTER_COUNT],
            pc: PC_START,
            cond: ConditionFlag::Zero,
        }
    }

    /// Reads general-purpose register `reg`, identified by its 3-bit field.
    pub fn get(&self, reg: u16) -> u16 {
        self.gpr[usize::from(reg) & 0x7]
    }

    /// Writes `value` to general-purpose register `reg`, identified by its
    /// 3-bit field.
    pub fn set(&mut self, reg: u16, value: u16) {
        self.gpr[usize::from(reg) & 0x7] = value;
    }

    /// Updates the condition code from the value now held in register `reg`.
    pub fn update_flags(&mut self, reg: u16) {
        self.cond = ConditionFlag::from_result(self.get(reg));
    }
}

#[cfg(test)]
mod tests {
    use lc3core::ConditionFlag;

    use super::Registers;

    #[test]
    fn flags_follow_the_sign_of_the_written_register() {
        let mut registers = Registers::new();

        registers.set(3, 0);
        registers.update_flags(3);
        assert_eq!(registers.cond, ConditionFlag::Zero);

        registers.set(3, 0x8000);
        registers.update_flags(3);
        assert_eq!(registers.cond, ConditionFlag::Negative);

        registers.set(3, 0x0001);
        registers.update_flags(3);
        assert_eq!(registers.cond, ConditionFlag::Positive);
    }
}
