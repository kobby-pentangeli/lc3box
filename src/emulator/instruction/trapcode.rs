use super::VM;

pub enum Trapcode {
    Halt = 0x25,
}

impl Trapcode {
    pub(crate) fn execute(_instruction: u16, _vm: &mut VM) {
        todo!()
    }
}
