use std::{iter, ptr};

use windows::Win32::{
    Foundation::HANDLE,
    System::{
        DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        Ole::CF_UNICODETEXT,
    },
};

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub fn write_text(text: &str) -> Result<(), String> {
    let wide: Vec<u16> = text.encode_utf16().chain(iter::once(0)).collect();

    unsafe {
        OpenClipboard(None).map_err(|error| format!("Could not open the clipboard: {error}"))?;
        let _clipboard = ClipboardGuard;
        EmptyClipboard().map_err(|error| format!("Could not clear the clipboard: {error}"))?;

        let memory = GlobalAlloc(GMEM_MOVEABLE, wide.len() * size_of::<u16>())
            .map_err(|error| format!("Could not allocate clipboard memory: {error}"))?;
        let destination = GlobalLock(memory) as *mut u16;
        if destination.is_null() {
            return Err("Could not write to the clipboard.".to_owned());
        }
        ptr::copy_nonoverlapping(wide.as_ptr(), destination, wide.len());
        let _ = GlobalUnlock(memory);

        SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(memory.0)))
            .map_err(|error| format!("Could not set the clipboard text: {error}"))?;
    }

    Ok(())
}
