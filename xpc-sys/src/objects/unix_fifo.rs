use libc::{mkfifo, mode_t, open, tmpnam, O_RDONLY, O_WRONLY};
use std::os::unix::prelude::RawFd;
use std::{
    ffi::{CStr, CString},
    fs::{remove_file, File},
    io::Read,
    os::unix::prelude::FromRawFd,
    ptr::null_mut,
};

use crate::rs_strerror;

/// A simple wrapper around a UNIX FIFO
pub struct UnixFifo(pub CString);

impl UnixFifo {
    /// Create a new FIFO, make sure mode_t is 0oXXX!
    pub fn new(mode: mode_t) -> Result<Self, String> {
        let fifo_name = unsafe { CStr::from_ptr(tmpnam(null_mut())) };
        let err = unsafe { mkfifo(fifo_name.as_ptr(), mode) };

        if err == 0 {
            Ok(UnixFifo(fifo_name.to_owned()))
        } else {
            Err(rs_strerror(err))
        }
    }

    /// Open the FIFO as O_RDONLY, read until EOF, clean up fd before returning the buffer.
    pub fn block_and_read_bytes(&self) -> Vec<u8> {
        let Self(fifo_name) = self;

        let fifo_fd_read = unsafe { open(fifo_name.as_ptr(), O_RDONLY) };
        let mut file = unsafe { File::from_raw_fd(fifo_fd_read) };

        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("Must read bytes");

        unsafe { libc::close(fifo_fd_read) };

        buf
    }

    /// Open O_WRONLY, pass to fn and clean up before returning.
    pub fn with_writer<T>(&self, f: impl Fn(RawFd) -> T) -> T {
        let Self(fifo_name) = self;
        let fifo_fd_write = unsafe { open(fifo_name.as_ptr(), O_WRONLY) };
        let response = f(fifo_fd_write);
        unsafe { libc::close(fifo_fd_write) };
        response
    }
}

impl Drop for UnixFifo {
    fn drop(&mut self) {
        let Self(fifo_name) = self;

        remove_file(&fifo_name.to_string_lossy().to_string()).expect("Must tear down FIFO");
    }
}
