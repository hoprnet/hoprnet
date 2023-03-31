use libc;

#[inline(always)]
pub fn is_a_terminal() -> bool {
    unsafe {
        libc::isatty(libc::STDOUT_FILENO) == 1
    }
}

#[inline(always)]
pub fn is_a_color_terminal() -> bool {
    is_a_terminal()
}
