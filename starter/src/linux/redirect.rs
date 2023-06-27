use std::{error::Error, io, os::fd::RawFd};

use libc::{dup2, STDERR_FILENO, STDOUT_FILENO};

pub enum StandardOutputID {
    Output,
    Error,
}

pub fn set_standard_input_output(stdio: StandardOutputID, fd: RawFd) -> Result<(), Box<dyn Error>> {
    let id = match stdio {
        StandardOutputID::Output => STDOUT_FILENO,
        StandardOutputID::Error => STDERR_FILENO,
    };

    unsafe {
        if dup2(fd, id) < 0 {
            return Err(io::Error::last_os_error().into());
        }
    }

    Ok(())
}
