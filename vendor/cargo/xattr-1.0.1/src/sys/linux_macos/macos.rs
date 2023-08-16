use libc::{c_char, c_int, c_void, size_t, ssize_t};

const XATTR_NOFOLLOW: c_int = 0x0001;

#[inline(always)]
pub unsafe fn fremovexattr(fd: c_int, name: *const c_char) -> c_int {
    use libc::fremovexattr;
    fremovexattr(fd, name, 0)
}

#[inline(always)]
pub unsafe fn fsetxattr(
    fd: c_int,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
) -> c_int {
    use libc::fsetxattr;
    fsetxattr(fd, name, value, size, 0, 0)
}

#[inline(always)]
pub unsafe fn fgetxattr(
    fd: c_int,
    name: *const c_char,
    value: *mut c_void,
    size: size_t,
) -> ssize_t {
    use libc::fgetxattr;
    fgetxattr(fd, name, value, size, 0, 0)
}

#[inline(always)]
pub unsafe fn flistxattr(fd: c_int, buf: *mut c_char, size: size_t) -> ssize_t {
    use libc::flistxattr;
    flistxattr(fd, buf, size, 0)
}

#[inline(always)]
pub unsafe fn lremovexattr(path: *const c_char, name: *const c_char) -> c_int {
    use libc::removexattr;
    removexattr(path, name, XATTR_NOFOLLOW)
}

#[inline(always)]
pub unsafe fn lsetxattr(
    path: *const c_char,
    name: *const c_char,
    value: *const c_void,
    size: size_t,
) -> c_int {
    use libc::setxattr;
    setxattr(path, name, value, size, 0, XATTR_NOFOLLOW)
}

#[inline(always)]
pub unsafe fn lgetxattr(
    path: *const c_char,
    name: *const c_char,
    value: *mut c_void,
    size: size_t,
) -> ssize_t {
    use libc::getxattr;
    getxattr(path, name, value, size, 0, XATTR_NOFOLLOW)
}

#[inline(always)]
pub unsafe fn llistxattr(path: *const c_char, buf: *mut c_char, size: size_t) -> ssize_t {
    use libc::listxattr;
    listxattr(path, buf, size, XATTR_NOFOLLOW)
}
