use std::{error::Error, fs::OpenOptions, io, os::windows::io::RawHandle};

use windows_sys::Win32::{
    Foundation::{SetHandleInformation, FALSE, HANDLE, HANDLE_FLAG_INHERIT, TRUE},
    System::Console::{AllocConsole, GetConsoleWindow, SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
    UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
};

pub enum StandardInputOutput {
    Input,
    Output,
    Error,
}

pub fn open_null_device(opt: &OpenOptions) -> std::fs::File {
    opt.open("nul:").unwrap()
}

pub fn set_standard_input_output(stdio: StandardInputOutput, fd: RawHandle) -> Result<(), Box<dyn Error>> {
    unsafe {
        if AllocConsole() == TRUE {
            let console = GetConsoleWindow();

            ShowWindow(console, SW_HIDE);
        }
    }

    let id = match stdio {
        StandardInputOutput::Input => STD_INPUT_HANDLE,
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
