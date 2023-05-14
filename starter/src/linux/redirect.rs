use std::{error::Error, fs::OpenOptions, io, os::fd::RawFd};

use libc::{dup2, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

pub enum StandardInputOutput {
    Input,
    Output,
    Error,
}

pub fn open_null_device(opt: &OpenOptions) -> std::fs::File {
    opt.open("/dev/null").unwrap()
}

pub fn set_standard_input_output(stdio: StandardInputOutput, fd: RawFd) -> Result<(), Box<dyn Error>> {
    let id = match stdio {
        StandardInputOutput::Input => STDIN_FILENO,
        StandardInputOutput::Output => STDOUT_FILENO,
        StandardInputOutput::Error => STDERR_FILENO,
    };

    unsafe {
        if dup2(fd, id) < 0 {
            return Err(io::Error::last_os_error().into());
        }
    }

    Ok(())
}
