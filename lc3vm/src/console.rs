//! Console I/O for the virtual machine: raw-mode terminal setup and the
//! blocking and non-blocking keyboard reads the trap and KBSR paths need.
//!
//! The Unix terminal layer is reached through the `termios` safe wrappers.
//! On platforms without that layer the terminal handling is inert: the VM
//! still builds and runs with line-buffered standard input, and the
//! non-blocking poll reports that no key is ready.

use std::io::{self, Read as _};

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
    /// never blocks. The blocking `read_char` restores `VMIN = 1` around its
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

/// Reads the next key if one is already available, without blocking.
///
/// Relies on the non-blocking raw mode established by [`RawMode`]: a read of
/// zero bytes means no key is ready. Off Unix, where that mode is unavailable,
/// it always reports no key.
#[cfg(unix)]
pub(crate) fn poll_char() -> io::Result<Option<u8>> {
    let mut buf = [0u8; 1];
    match io::stdin().read(&mut buf)? {
        0 => Ok(None),
        _ => Ok(Some(buf[0])),
    }
}

#[cfg(not(unix))]
pub(crate) fn poll_char() -> io::Result<Option<u8>> {
    Ok(None)
}

/// Blocks until the next key is available and returns it.
///
/// Restores blocking line behavior (`VMIN = 1`) for the duration of the read,
/// then returns the terminal to the non-blocking mode the KBSR poll depends on.
#[cfg(unix)]
pub(crate) fn read_char() -> io::Result<u8> {
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
pub(crate) fn read_char() -> io::Result<u8> {
    let mut buf = [0u8; 1];
    io::stdin().read_exact(&mut buf)?;
    Ok(buf[0])
}
