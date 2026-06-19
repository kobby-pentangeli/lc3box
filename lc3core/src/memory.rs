//! LC-3 memory map: address-space size and the fixed region and device-register
//! addresses defined by the architecture.

/// The number of addressable memory locations: `2^16` 16-bit words
/// (`x0000`–`xFFFF`).
pub const MEMORY_SIZE: usize = 1 << 16;

/// The address at which user programs conventionally begin execution.
pub const PC_START: u16 = 0x3000;

/// Start of the trap vector table (`x0000`–`x00FF`).
pub const TRAP_VECTOR_TABLE: u16 = 0x0000;

/// Start of the interrupt vector table (`x0100`–`x01FF`).
pub const INTERRUPT_VECTOR_TABLE: u16 = 0x0100;

/// Start of the operating-system and supervisor-stack region (`x0200`–`x2FFF`).
pub const SUPERVISOR_SPACE: u16 = 0x0200;

/// Start of the region available to user programs (`x3000`–`xFDFF`).
pub const USER_SPACE: u16 = 0x3000;

/// Last address available to user programs.
pub const USER_SPACE_END: u16 = 0xFDFF;

/// Start of the memory-mapped device register region (`xFE00`–`xFFFF`).
pub const DEVICE_REGISTERS: u16 = 0xFE00;

/// Keyboard status register: bit 15 is set when a character is ready.
pub const KBSR: u16 = 0xFE00;

/// Keyboard data register: the character read from the keyboard.
pub const KBDR: u16 = 0xFE02;

/// Display status register: bit 15 is set when the display is ready.
pub const DSR: u16 = 0xFE04;

/// Display data register: a character written here is shown on the console.
pub const DDR: u16 = 0xFE06;

/// Machine control register: clearing bit 15 halts the processor.
pub const MCR: u16 = 0xFFFE;
