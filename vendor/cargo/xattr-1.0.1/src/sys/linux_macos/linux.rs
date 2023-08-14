use libc::{c_char, c_int, c_void, size_t};
pub use libc::{fgetxattr, flistxattr, fremovexattr, lgetxattr, llistxattr,
           lremovexattr};

pub unsafe fn fsetxattr(
    fd: c_int,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
) -> c_int {
    use libc::fsetxattr;
    fsetxattr(fd, name, value, size, 0)
}

pub unsafe fn lsetxattr(
    path: *const c_char,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
) -> c_int {
    use libc::lsetxattr;
    lsetxattr(path, name, value, size, 0)
}
