use std::{error::Error, ffi::CString, io, path::Path};

use cstr::cstr;

use libc::{dup2, open, O_CREAT, O_TRUNC, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

pub fn redirect_standard_output_to_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let path = CString::new(path.to_str().unwrap())?;

    let fd = unsafe { open(path.as_ptr(), O_WRONLY | O_CREAT | O_TRUNC, 0o600) };
    if fd < 0 {
        return Err(io::Error::last_os_error().into());
    }

    let null_fd = unsafe { open(cstr!("/dev/null").as_ptr(), O_WRONLY) };
    if null_fd < 0 {
        panic!("open /dev/null failed");
    }

    unsafe {
        dup2(null_fd, STDIN_FILENO);
        dup2(fd, STDOUT_FILENO);
        dup2(fd, STDERR_FILENO);
    }

    Ok(())
}
