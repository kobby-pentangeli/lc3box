#[derive(Debug)]
pub enum Opcode {
    Br = 0,
}

impl Opcode {
    pub fn get(instruction: u16) -> Option<Self> {
        match instruction >> 12 {
            0 => Some(Self::Br),
            _ => None,
        }
    }
}
