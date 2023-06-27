use std::{iter::once, ops::Deref};

pub trait Win32Strings {
    fn to_win32_utf16(&self) -> Vec<u16>;
}

impl<T: Deref<Target = str>> Win32Strings for T {
    fn to_win32_utf16(&self) -> Vec<u16> {
        self.encode_utf16().chain(once(0)).collect::<Vec<u16>>()
    }
}
