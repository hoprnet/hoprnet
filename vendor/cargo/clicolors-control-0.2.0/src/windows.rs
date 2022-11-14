use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_ERROR_HANDLE};
use winapi::um::handleapi::{INVALID_HANDLE_VALUE};
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
use winapi::um::processenv::{GetStdHandle};

const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;


pub fn is_a_terminal() -> bool {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut out = 0;
        GetConsoleMode(handle, &mut out) != 0
    }
}

#[cfg(feature="terminal_autoconfig")]
pub fn is_a_color_terminal() -> bool {
    if !is_a_terminal() {
        return false;
    }
    enable_ansi_mode()
}

#[cfg(not(feature="terminal_autoconfig"))]
pub fn is_a_color_terminal() -> bool {
    false
}

fn enable_ansi_on(handle: u32) -> bool {
    unsafe {
        let handle = GetStdHandle(handle);
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut dw_mode = 0;
        if GetConsoleMode(handle, &mut dw_mode) == 0 {
            return false;
        }

        dw_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if SetConsoleMode(handle, dw_mode) == 0 {
            return false;
        }

        true
    }
}

pub fn enable_ansi_mode() -> bool {
    enable_ansi_on(STD_OUTPUT_HANDLE) || enable_ansi_on(STD_ERROR_HANDLE)
}
