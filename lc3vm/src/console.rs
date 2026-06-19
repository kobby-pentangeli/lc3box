//! Console I/O for the virtual machine: raw-mode terminal setup and the
//! blocking and non-blocking keyboard reads the trap and KBSR paths need.
//!
//! The traps and the keyboard poll reach the outside world through the
//! [`Console`] trait rather than `std::io` directly, so the binary drives the
//! real terminal while tests drive scripted input and capture output. The Unix
//! terminal layer is reached through the `termios` safe wrappers; on platforms
//! without that layer the terminal handling is inert, so the VM still builds
//! and runs with line-buffered standard input and the poll reports no key.

use std::io::{self, BufWriter, Read as _, Write as _};

#[cfg(unix)]
use libc::STDIN_FILENO;
#[cfg(unix)]
use termios::{ECHO, ICANON, TCSANOW, Termios, VMIN, VTIME, tcsetattr};

/// RAII guard that switches the terminal into raw, non-blocking mode and
/// restores the previous settings when dropped.
pub struct RawMode {
    #[cfg(unix)]
    original: Termios,
}

#[cfg(unix)]
impl RawMode {
    /// Switches the controlling terminal to raw mode with non-blocking reads.
    ///
    /// Clearing `ICANON`/`ECHO` delivers keystrokes unbuffered and unechoed;
    /// `VMIN = 0`, `VTIME = 0` makes reads return immediately, so the KBSR poll
    /// never blocks. The blocking character read restores `VMIN = 1` around its
    /// own read.
    pub fn enable() -> io::Result<Self> {
        let original = Termios::from_fd(STDIN_FILENO)?;

        let mut raw = original;
        raw.c_lflag &= !(ICANON | ECHO);
        raw.c_cc[VMIN] = 0;
        raw.c_cc[VTIME] = 0;
        tcsetattr(STDIN_FILENO, TCSANOW, &raw)?;

        Ok(Self { original })
    }
}

#[cfg(unix)]
impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = tcsetattr(STDIN_FILENO, TCSANOW, &self.original);
    }
}

#[cfg(not(unix))]
impl RawMode {
    /// No-op terminal setup where the Unix raw-mode layer is unavailable.
    pub fn enable() -> io::Result<Self> {
        Ok(Self {})
    }
}

/// The console the VM reads keys from and writes characters to.
///
/// One implementation ([`StdConsole`]) drives the real terminal; tests supply
/// their own with scripted input and a capture buffer.
pub(crate) trait Console {
    /// Returns the next key if one is already available, without blocking.
    fn poll_char(&mut self) -> io::Result<Option<u8>>;

    /// Blocks until the next key is available and returns it.
    fn read_char(&mut self) -> io::Result<u8>;

    /// Writes `bytes` to the output.
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>;

    /// Flushes any buffered output to the underlying device.
    fn flush(&mut self) -> io::Result<()>;
}

/// The production console: non-blocking terminal input and buffered standard
/// output. Input relies on the non-blocking raw mode established by [`RawMode`].
pub(crate) struct StdConsole {
    out: BufWriter<io::Stdout>,
}

impl StdConsole {
    pub(crate) fn new() -> Self {
        Self {
            out: BufWriter::new(io::stdout()),
        }
    }
}

impl Console for StdConsole {
    fn poll_char(&mut self) -> io::Result<Option<u8>> {
        poll_terminal()
    }

    fn read_char(&mut self) -> io::Result<u8> {
        read_terminal()
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.out.write_all(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

/// Reads the next key without blocking, returning `None` when none is ready.
///
/// A read of zero bytes under [`RawMode`]'s `VMIN = 0` means no key is ready.
/// Off Unix, where that mode is unavailable, this always reports no key.
#[cfg(unix)]
fn poll_terminal() -> io::Result<Option<u8>> {
    let mut buf = [0u8; 1];
    match io::stdin().read(&mut buf)? {
        0 => Ok(None),
        _ => Ok(Some(buf[0])),
    }
}

#[cfg(not(unix))]
fn poll_terminal() -> io::Result<Option<u8>> {
    Ok(None)
}

/// Blocks until the next key is available and returns it.
///
/// Restores blocking line behavior (`VMIN = 1`) for the duration of the read,
/// then returns the terminal to the non-blocking mode the KBSR poll depends on.
#[cfg(unix)]
fn read_terminal() -> io::Result<u8> {
    let original = Termios::from_fd(STDIN_FILENO)?;

    let mut blocking = original;
    blocking.c_cc[VMIN] = 1;
    blocking.c_cc[VTIME] = 0;
    tcsetattr(STDIN_FILENO, TCSANOW, &blocking)?;

    let mut buf = [0u8; 1];
    let read = io::stdin().read_exact(&mut buf);
    let _ = tcsetattr(STDIN_FILENO, TCSANOW, &original);

    read.map(|()| buf[0])
}

#[cfg(not(unix))]
fn read_terminal() -> io::Result<u8> {
    let mut buf = [0u8; 1];
    io::stdin().read_exact(&mut buf)?;
    Ok(buf[0])
}
