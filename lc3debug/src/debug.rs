//! The debugging engine: pure state transitions over a loaded program.
//!
//! [`Debugger`] owns an [`Lc3VM`], a breakpoint set, and the program it was
//! loaded from (so it can [`reset`](Debugger::reset)). It single-steps and
//! resumes the machine, manages breakpoints, reads and writes registers and
//! memory, and disassembles a memory window---each operation a plain method
//! returning either a [`Stop`] reason or borrowed state, with no terminal of its
//! own. A frontend drives it; the engine never reads or writes the console.

use std::collections::BTreeSet;

use lc3core::ObjectFile;
use lc3vm::{Lc3VM, Registers, Step};

use crate::cmd::Register;

/// Why a run of the machine stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stop {
    /// Completed the requested single steps without halting.
    Stepped,
    /// Reached a breakpoint; the address is the next instruction to run.
    Breakpoint(u16),
    /// Reached a `HALT` trap; the machine has stopped.
    Halted,
}

/// An interactive debugging session over a loaded LC-3 program.
pub struct Debugger {
    vm: Lc3VM,
    program: Vec<ObjectFile>,
    breakpoints: BTreeSet<u16>,
}

impl Debugger {
    /// Loads `program` into a fresh machine, ready at its entry point.
    ///
    /// Returns the VM's [`load_program`](Lc3VM::load_program) error if any
    /// segment would not fit in memory.
    pub fn new(program: Vec<ObjectFile>) -> Result<Self, lc3vm::Error> {
        let mut vm = Lc3VM::new();
        vm.load_program(&program)?;
        Ok(Self {
            vm,
            program,
            breakpoints: BTreeSet::new(),
        })
    }

    /// Reloads the original program into a fresh machine, returning to its entry
    /// point. Registers and memory are cleared, breakpoints are kept.
    pub fn reset(&mut self) -> Result<(), lc3vm::Error> {
        let mut vm = Lc3VM::new();
        vm.load_program(&self.program)?;
        self.vm = vm;
        Ok(())
    }

    /// Executes up to `count` instructions, stopping early at a `HALT`.
    ///
    /// Breakpoints do not interrupt an explicit step; only a `HALT`---reported
    /// as [`Stop::Halted`]---or a VM error ends the run before `count`
    /// instructions have run. Running the full count yields [`Stop::Stepped`].
    pub fn step(&mut self, count: u16) -> Result<Stop, lc3vm::Error> {
        for _ in 0..count {
            if let Step::Halted = self.vm.step()? {
                return Ok(Stop::Halted);
            }
        }
        Ok(Stop::Stepped)
    }

    /// Runs until a breakpoint, a `HALT`, or a VM error.
    ///
    /// Always executes at least one instruction, so a session resumed while
    /// sitting on a breakpoint leaves it rather than stopping on it again.
    pub fn resume(&mut self) -> Result<Stop, lc3vm::Error> {
        loop {
            if let Step::Halted = self.vm.step()? {
                return Ok(Stop::Halted);
            }
            let pc = self.vm.registers.pc;
            if self.breakpoints.contains(&pc) {
                return Ok(Stop::Breakpoint(pc));
            }
        }
    }

    /// Sets a breakpoint at `address`, returning whether it was newly added.
    pub fn add_breakpoint(&mut self, address: u16) -> bool {
        self.breakpoints.insert(address)
    }

    /// Clears the breakpoint at `address`, returning whether one was set.
    pub fn remove_breakpoint(&mut self, address: u16) -> bool {
        self.breakpoints.remove(&address)
    }

    /// The active breakpoint addresses, in ascending order.
    pub fn breakpoints(&self) -> impl Iterator<Item = u16> + '_ {
        self.breakpoints.iter().copied()
    }

    /// The current register file.
    pub fn registers(&self) -> &Registers {
        &self.vm.registers
    }

    /// Writes `value` to `register`.
    pub fn set_register(&mut self, register: Register, value: u16) {
        match register {
            Register::General(reg) => self.vm.registers.set(reg, value),
            Register::Pc => self.vm.registers.pc = value,
        }
    }

    /// Reads the word at `address`.
    pub fn read_memory(&self, address: u16) -> u16 {
        self.vm.memory.read(address)
    }

    /// Writes `value` to the word at `address`.
    pub fn write_memory(&mut self, address: u16, value: u16) {
        self.vm.memory.write(address, value);
    }

    /// Disassembles `len` words beginning at `address`, as an annotated listing.
    ///
    /// Reads the window straight from live memory, so it reflects any edits and
    /// self-modified code, and renders it through [`lc3dsm::disassemble`].
    pub fn disassemble(&self, address: u16, len: u16) -> String {
        let words = (0..len)
            .map(|offset| self.vm.memory.read(address.wrapping_add(offset)))
            .collect();
        lc3dsm::disassemble(&ObjectFile {
            origin: address,
            words,
        })
    }
}

#[cfg(test)]
mod tests {
    use lc3core::ObjectFile;

    use super::{Debugger, Stop};
    use crate::cmd::Register;

    /// A debugger over `words` loaded at the standard entry point `x3000`.
    fn debugger(words: &[u16]) -> Debugger {
        Debugger::new(vec![ObjectFile {
            origin: 0x3000,
            words: words.to_vec(),
        }])
        .expect("synthetic program fits in memory")
    }

    #[test]
    fn step_runs_exactly_the_requested_count_then_halts_early() {
        // ADD R0,R0,#1 ; ADD R0,R0,#1 ; HALT
        let mut dbg = debugger(&[0x1021, 0x1021, 0xF025]);

        assert_eq!(dbg.step(2).expect("step"), Stop::Stepped);
        assert_eq!(dbg.registers().get(0), 2);
        assert_eq!(dbg.registers().pc, 0x3002);

        // Five more requested, but the HALT ends the run after one.
        assert_eq!(dbg.step(5).expect("step"), Stop::Halted);
        assert_eq!(dbg.registers().get(0), 2);
    }

    #[test]
    fn resume_stops_at_breakpoints_and_a_cleared_one_no_longer_stops() {
        // ADD R0,R0,#1 (x3), HALT.
        let mut dbg = debugger(&[0x1021, 0x1021, 0x1021, 0xF025]);
        assert!(dbg.add_breakpoint(0x3001));
        assert!(dbg.add_breakpoint(0x3002));

        // Runs one instruction and stops at the first breakpoint, not before it
        // and not past it.
        assert_eq!(dbg.resume().expect("resume"), Stop::Breakpoint(0x3001));
        assert_eq!(dbg.registers().get(0), 1);

        // Clearing the next breakpoint lets the resumed run leave x3001 and pass
        // through x3002 to the HALT rather than stopping there.
        assert!(dbg.remove_breakpoint(0x3002));
        assert_eq!(dbg.resume().expect("resume"), Stop::Halted);
        assert_eq!(dbg.registers().get(0), 3);
    }

    #[test]
    fn breakpoints_lists_active_addresses_in_order() {
        let mut dbg = debugger(&[0xF025]);
        dbg.add_breakpoint(0x3005);
        dbg.add_breakpoint(0x3001);
        assert!(!dbg.add_breakpoint(0x3005)); // a duplicate is not re-added
        dbg.add_breakpoint(0x3003);
        dbg.remove_breakpoint(0x3001);

        assert_eq!(dbg.breakpoints().collect::<Vec<_>>(), vec![0x3003, 0x3005]);
    }

    #[test]
    fn register_edits_drive_execution() {
        // ADD R0, R1, R2 ; HALT
        let mut dbg = debugger(&[0x1042, 0xF025]);
        dbg.set_register(Register::General(1), 7);
        dbg.set_register(Register::General(2), 35);

        assert_eq!(dbg.step(1).expect("step"), Stop::Stepped);
        assert_eq!(dbg.registers().get(0), 42);
    }

    #[test]
    fn setting_the_program_counter_redirects_the_fetch() {
        // ADD R0,R0,#1 at x3000, HALT at x3001. Jumping the PC to the HALT must
        // skip the ADD entirely.
        let mut dbg = debugger(&[0x1021, 0xF025]);
        dbg.set_register(Register::Pc, 0x3001);

        assert_eq!(dbg.step(1).expect("step"), Stop::Halted);
        assert_eq!(dbg.registers().get(0), 0);
    }

    #[test]
    fn memory_edits_drive_execution() {
        // ADD R0,R0,#1 at x3000, HALT at x3001. Patching x3001 into a second ADD
        // before it is fetched must increment R0 again rather than halt.
        let mut dbg = debugger(&[0x1021, 0xF025]);
        dbg.write_memory(0x3001, 0x1021);

        assert_eq!(dbg.step(2).expect("step"), Stop::Stepped);
        assert_eq!(dbg.registers().get(0), 2);
    }

    #[test]
    fn reset_returns_to_the_entry_point_and_restores_clobbered_memory() {
        let mut dbg = debugger(&[0x1021, 0x1021, 0xF025]);
        dbg.add_breakpoint(0x3001);
        dbg.step(2).expect("step");
        dbg.write_memory(0x3000, 0xF025); // clobber the entry instruction

        dbg.reset().expect("reset");

        assert_eq!(dbg.registers().pc, 0x3000);
        assert_eq!(dbg.registers().get(0), 0);
        assert_eq!(dbg.read_memory(0x3000), 0x1021); // original word reloaded
        assert_eq!(dbg.breakpoints().collect::<Vec<_>>(), vec![0x3001]); // kept
    }

    #[test]
    fn disassemble_window_renders_decoded_instructions_at_their_addresses() {
        let dbg = debugger(&[0x1021, 0xF025]);
        let listing = dbg.disassemble(0x3000, 2);

        assert!(listing.contains("ADD R0, R0, #1"), "{listing}");
        assert!(
            listing.lines().any(|line| line.contains("HALT")),
            "{listing}"
        );
        assert!(listing.contains("x3000"), "{listing}");
    }
}
