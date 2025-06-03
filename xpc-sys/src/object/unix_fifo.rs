use libc::{__error, mkfifo, mode_t, open, tmpnam, O_RDONLY, O_WRONLY};
use std::os::unix::prelude::RawFd;
use std::{
    ffi::{CStr, CString},
    fs::{remove_file, File},
    io::Read,
    os::unix::prelude::FromRawFd,
    ptr::null_mut,
};

use crate::rs_strerror;

/// A wrapper around a UNIX FIFO
pub struct UnixFifo(pub CString);

impl UnixFifo {
    /// Create a new FIFO, make sure mode_t is 0oXXX!
    #[must_use]
    pub fn new(mode: mode_t) -> Result<Self, String> {
        let fifo_name = unsafe { CStr::from_ptr(tmpnam(null_mut())) };
        let err = unsafe { mkfifo(fifo_name.as_ptr(), mode) };

        if err == 0 {
            Ok(UnixFifo(fifo_name.to_owned()))
        } else {
            Err(rs_strerror(err))
        }
    }

    /// Open O_RDONLY, read until EOF, close fd, return buffer.
    #[must_use]
    pub fn block_and_read_bytes(&self) -> Result<Vec<u8>, String> {
        let Self(fifo_name) = self;

        let fifo_fd_read = unsafe { open(fifo_name.as_ptr(), O_RDONLY) };
        log::info!("opened read fifo {}", fifo_fd_read);
        let mut file = unsafe { File::from_raw_fd(fifo_fd_read) };

        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("Must read bytes");

        Self::close(fifo_fd_read)?;
        Ok(buf)
    }

    /// Open O_WRONLY, call fn, close fd, yield result
    #[must_use]
    pub fn with_writer<T>(&self, f: impl Fn(RawFd) -> T) -> Result<T, String> {
        let Self(fifo_name) = self;
        let fifo_fd_write = unsafe { open(fifo_name.as_ptr(), O_WRONLY) };
        log::info!("opened write fifo {}", fifo_fd_write);
        let result = f(fifo_fd_write);
        Self::close(fifo_fd_write)?;
        Ok(result)
    }

    /// Wrap libc close()
    #[must_use]
    pub fn close(fd: RawFd) -> Result<(), String> {
        let err = unsafe { libc::close(fd) };

        if err == 0 {
            Ok(())
        } else {
            Err(rs_strerror(unsafe { *__error() }))
        }
    }
}

impl Drop for UnixFifo {
    fn drop(&mut self) {
        let Self(fifo_name) = self;

        remove_file(fifo_name.to_string_lossy().to_string()).expect("Must rm FIFO");
    }
}
