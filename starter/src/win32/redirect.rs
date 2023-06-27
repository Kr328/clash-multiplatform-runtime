use std::{error::Error, io, os::windows::io::RawHandle};

use windows_sys::Win32::{
    Foundation::{SetHandleInformation, FALSE, HANDLE, HANDLE_FLAG_INHERIT, TRUE},
    System::Console::{AllocConsole, GetConsoleWindow, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
    UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
};

pub enum StandardInputOutput {
    Output,
    Error,
}

pub fn set_standard_input_output(stdio: StandardInputOutput, fd: RawHandle) -> Result<(), Box<dyn Error>> {
    unsafe {
        if AllocConsole() == TRUE {
            let console = GetConsoleWindow();

            ShowWindow(console, SW_HIDE);
        }
    }

    let id = match stdio {
        StandardInputOutput::Output => STD_OUTPUT_HANDLE,
        StandardInputOutput::Error => STD_ERROR_HANDLE,
    };

    unsafe {
        if SetHandleInformation(fd as HANDLE, HANDLE_FLAG_INHERIT, TRUE as u32) == FALSE {
            return Err(io::Error::last_os_error().into());
        }

        if SetStdHandle(id, fd as HANDLE) == FALSE {
            return Err(io::Error::last_os_error().into());
        }
    }

    Ok(())
}
