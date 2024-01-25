extern crate libnghttp2_sys as ffi;

fn main() {
    unsafe {
        ffi::nghttp2_version(0);
    }
}
