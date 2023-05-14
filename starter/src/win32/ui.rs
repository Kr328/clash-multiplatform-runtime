use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

use crate::win32::strings::Win32Strings;

pub fn show_error_message(msg: &str) {
    let msg = msg.to_win32_utf16();
    let title = "Error".to_win32_utf16();

    unsafe {
        MessageBoxW(0, msg.as_ptr(), title.as_ptr(), MB_OK);
    }
}
