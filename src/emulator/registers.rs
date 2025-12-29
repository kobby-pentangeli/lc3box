/// Memory address of the program counter (PC) register.
const PC_START: u16 = 0x3000;

/// Register number 9 is the `cond` register.
const RCOND_INDEX: u16 = 9;

/// The `RCOND` register stores condition flags that represent information about
/// the most recent computation. It's used for checking logical conditions.
/// The LC-3 uses only 3 condition flags which indicate
/// the sign of the previous computation.
///
/// In binary, with 3 bits only:
/// - 1 == 001
/// - 2 == 010
/// - 4 == 100
///
/// So we're essentially playing with the possible conditional flags settings!
/// Because the condition instruction will be `nzp` (neg, zero, pos)
/// and only one can be set at a time, it will either be 001 (positive set `nz1`),
/// 010 (zero set, `n1p`), and 100 (negative set, `1zp`).
/// These three binary values are 1, 2, and 4 respectively, in decimal!
enum ConditionFlag {
    /// Positive flag set, i.e., `nz1` for the `nzp` instruction.
    Pos = 1 << 0, // Positive

    /// Zero flag set, i.e., `n1p` for the `nzp` instruction.
    Zro = 1 << 1,

    /// Negative flag set, i.e., `1zp` for the `nzp` instruction.
    Neg = 1 << 2,
}

/// LC-3 has 10 registers: 8 general-purpose registers (`r0`...`r7`),
/// 1 program counter (`pc`) register, and one condition flag (`cond`) register.
/// The program counter stores a memory address of the next instruction to be executed.
pub struct Registers {
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,
    pub pc: u16,
    pub cond: u16,
}

impl Registers {
    /// Initializes all 10 registers with default values.
    /// The program counter starts at 0x3000 (`PC_START`).
    pub fn new() -> Self {
        Self {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: PC_START,
            cond: 0,
        }
    }

    /// Writes a `value` to a register given its `address` (index).
    pub fn update(&mut self, address: u16, value: u16) {
        match address {
            0 => self.r0 = value,
            1 => self.r1 = value,
            2 => self.r2 = value,
            3 => self.r3 = value,
            4 => self.r4 = value,
            5 => self.r5 = value,
            6 => self.r6 = value,
            7 => self.r7 = value,
            8 => self.pc = value,
            9 => self.cond = value,
            _ => panic!("Index out of bounds"),
        }
    }

    /// Reads the value at the given address (`index`) of a register.
    pub fn get(&self, address: u16) -> u16 {
        match address {
            0 => self.r0,
            1 => self.r1,
            2 => self.r2,
            3 => self.r3,
            4 => self.r4,
            5 => self.r5,
            6 => self.r6,
            7 => self.r7,
            8 => self.pc,
            9 => self.cond,
            _ => panic!("Index out of bounds"),
        }
    }

    /// Updates the condition register (`self.cond`) based on the last
    /// operation on a given general-purpose (`r`) register.
    pub fn update_cond_register(&mut self, r: u16) {
        if self.get(r) == 0 {
            self.update(RCOND_INDEX, ConditionFlag::Zro as u16);
        } else if (self.get(r) >> 15) != 0 {
            // a 1 for MSB indicates negative
            self.update(RCOND_INDEX, ConditionFlag::Neg as u16);
        } else {
            self.update(RCOND_INDEX, ConditionFlag::Pos as u16);
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
