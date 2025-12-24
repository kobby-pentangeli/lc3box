const PC_START: u16 = 0x3000;

enum ConditionFlag {
    Pos = 1 << 0, // Positive
    Zro = 1 << 1, // Zero
    Neg = 1 << 2, // Negative
}

pub struct Register {
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,
    // program counter
    pub pc: u16,
    // condition flag
    pub cond: u16,
}

impl Register {
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

    pub fn update_cond_register(&mut self, r: u16) {
        if self.get(r) == 0 {
            self.update(9, ConditionFlag::Zro as u16);
        } else if (self.get(r) >> 15) != 0 {
            // a 1 for MSB indicates negative
            self.update(9, ConditionFlag::Neg as u16);
        } else {
            self.update(9, ConditionFlag::Pos as u16);
        }
    }

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
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}
