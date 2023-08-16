pub(crate) mod addr;
pub(crate) mod ext;
#[cfg(not(any(windows, target_os = "espidf", target_os = "redox", target_os = "wasi")))]
pub(crate) mod msghdr;
pub(crate) mod read_sockaddr;
pub(crate) mod send_recv;
pub(crate) mod syscalls;
pub(crate) mod write_sockaddr;
