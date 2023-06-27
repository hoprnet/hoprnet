#![cfg(not(target_os = "windows"))]

use path_slash::{CowExt as _, PathBufExt as _, PathExt as _};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const INVALID_UTF8_BACKSLASH: &[(&[u8], &str)] = &[
    (b"aaa\\\xc3", "aaa/\u{FFFD}"),
    (b"aaa\\\xc3\\", "aaa/\u{FFFD}/"),
];

#[test]
fn invalid_utf8_pathbuf_from_backslash() {
    for (b, s) in INVALID_UTF8_BACKSLASH {
        let o = OsStr::from_bytes(b);
        let p = PathBuf::from_backslash_lossy(o);
        assert_eq!(p.to_str().unwrap(), *s, "{:x?}", b);
    }
}

#[test]
fn invalid_utf8_cow_from_backslash() {
    for (b, s) in INVALID_UTF8_BACKSLASH {
        let o = OsStr::from_bytes(b);
        let p = Cow::from_backslash_lossy(o);
        assert_eq!(p.to_str().unwrap(), *s, "{:x?}", b);
    }
}

const INVALID_UTF8_TO_SLASH: &[(&[u8], &str)] = &[
    (b"aaa/\xc3", "aaa/\u{FFFD}"),
    (b"aaa/\xc3/", "aaa/\u{FFFD}/"),
];

#[test]
fn invalid_utf8_to_slash_lossy() {
    for (input, output) in INVALID_UTF8_TO_SLASH {
        let p = Path::new(OsStr::from_bytes(input));
        assert_eq!(p.to_slash_lossy(), *output, "{:x?}", input);
    }
}

#[test]
fn invalid_utf8_to_slash() {
    for (input, _) in INVALID_UTF8_TO_SLASH {
        let p = Path::new(OsStr::from_bytes(input));
        assert_eq!(p.to_slash(), None, "{:x?}", input);
    }
}
