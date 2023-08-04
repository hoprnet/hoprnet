#![cfg(target_os = "windows")]

use path_slash::{CowExt as _, PathBufExt as _, PathExt as _};
use std::borrow::Cow;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

#[test]
fn with_driver_letter_to_slash() {
    let path = PathBuf::from_slash("C:/foo/bar");
    assert_eq!(path, PathBuf::from(r"C:\foo\bar"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, "C:/foo/bar");
}

#[test]
fn with_drive_letter_to_slash_lossy() {
    let path = PathBuf::from_slash("C:/foo/bar");
    assert_eq!(path, PathBuf::from(r"C:\foo\bar"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, "C:/foo/bar");
}

#[test]
fn with_drive_letter_but_no_path_to_slash() {
    let path = PathBuf::from_slash("C:");
    assert_eq!(path, PathBuf::from(r"C:"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, "C:");
}

#[test]
fn with_drive_letter_but_no_path_to_slash_lossy() {
    let path = PathBuf::from_slash("C:");
    assert_eq!(path, PathBuf::from(r"C:"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, "C:");
}

#[test]
fn with_verbatim_drive_letter_to_slash() {
    let path = PathBuf::from_slash(r"\\?\C:/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\?\C:\foo\bar"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, r"\\?\C:/foo/bar");
}

#[test]
fn with_verbatim_drive_letter_to_slash_lossy() {
    let path = PathBuf::from_slash(r"\\?\C:/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\?\C:\foo\bar"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, r"\\?\C:/foo/bar");
}

#[test]
fn with_unc_prefix_to_slash() {
    let path = PathBuf::from_slash(r"\\server\share/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\server\share\foo\bar"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, r"\\server\share/foo/bar");
}

#[test]
fn with_unc_prefix_to_slash_lossy() {
    let path = PathBuf::from_slash(r"\\server\share/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\server\share\foo\bar"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, r"\\server\share/foo/bar");
}

#[test]
fn with_unc_prefix_but_no_path_to_slash() {
    let path = PathBuf::from_slash(r"\\server\share");
    assert_eq!(path, PathBuf::from(r"\\server\share"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, r"\\server\share");
}

#[test]
fn with_unc_prefix_but_no_path_to_slash_lossy() {
    let path = PathBuf::from_slash(r"\\server\share");
    assert_eq!(path, PathBuf::from(r"\\server\share"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, r"\\server\share");
}

#[test]
fn with_verbatim_unc_prefix_to_slash() {
    let path = PathBuf::from_slash(r"\\?\UNC\server\share/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\?\UNC\server\share\foo\bar"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, r"\\?\UNC\server\share/foo/bar");
}

#[test]
fn with_verbatim_unc_prefix_to_slash_lossy() {
    let path = PathBuf::from_slash(r"\\?\UNC\server\share/foo/bar");
    assert_eq!(path, PathBuf::from(r"\\?\UNC\server\share\foo\bar"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, r"\\?\UNC\server\share/foo/bar");
}

#[test]
fn with_verbatim_unc_prefix_but_no_path_to_slash() {
    let path = PathBuf::from_slash(r"\\?\UNC\server\share");
    assert_eq!(path, PathBuf::from(r"\\?\UNC\server\share"));
    let slash = path.to_slash().unwrap();
    assert_eq!(slash, r"\\?\UNC\server\share");
}

#[test]
fn with_verbatim_unc_prefix_but_no_path_to_slash_lossy() {
    let path = PathBuf::from_slash(r"\\?\UNC\server\share");
    assert_eq!(path, PathBuf::from(r"\\?\UNC\server\share"));
    let slash = path.to_slash_lossy();
    assert_eq!(slash, r"\\?\UNC\server\share");
}

const UTF16_TEST_TO_SLASH: &[(&[u16], &str)] = &[
    (
        // あ\い\う\え\お
        &[
            0x3042, 0x005c, 0x3044, 0x005c, 0x3046, 0x005c, 0x3048, 0x005c, 0x304a,
        ],
        "あ/い/う/え/お",
    ),
    (
        // あ\い\う\え\お\
        &[
            0x3042, 0x005c, 0x3044, 0x005c, 0x3046, 0x005c, 0x3048, 0x005c, 0x304a, 0x005c,
        ],
        "あ/い/う/え/お/",
    ),
];

#[test]
fn utf16_encoded_os_str_to_slash() {
    for (b, s) in UTF16_TEST_TO_SLASH {
        let p = PathBuf::from(OsString::from_wide(b));
        assert_eq!(p.to_slash().unwrap(), *s);
    }
}

#[test]
fn utf16_encoded_os_str_to_slash_lossy() {
    for (b, s) in UTF16_TEST_TO_SLASH {
        let p = PathBuf::from(OsString::from_wide(b));
        assert_eq!(p.to_slash_lossy(), *s);
    }
}

const UTF16_TEST_FROM_SLASH: &[(&[u16], &str)] = &[
    (
        // あ/い/う/え/お
        &[
            0x3042, 0x002f, 0x3044, 0x002f, 0x3046, 0x002f, 0x3048, 0x002f, 0x304a,
        ],
        r"あ\い\う\え\お",
    ),
    (
        // あ/い/う/え/お/
        &[
            0x3042, 0x002f, 0x3044, 0x002f, 0x3046, 0x002f, 0x3048, 0x002f, 0x304a, 0x002f,
        ],
        r"あ\い\う\え\お\",
    ),
];

#[test]
fn utf16_encoded_os_str_pathbuf_from_slash() {
    for (s, b) in UTF16_TEST_FROM_SLASH {
        let o = OsString::from_wide(s);
        let p = PathBuf::from_slash_lossy(&o);
        assert_eq!(p, Path::new(b), "{:x?}", b);
    }
}

#[test]
fn utf16_encoded_os_str_cow_from_slash() {
    for (s, b) in UTF16_TEST_FROM_SLASH {
        let o = OsString::from_wide(s);
        let p = Cow::from_slash_lossy(&o);
        assert_eq!(p, Path::new(b), "{:x?}", b);
    }
}

const INVALID_UTF16_TO_SLASH: &[(&[u16], &str)] = &[
    (
        &[b'a' as u16, b'\\' as u16, 0xd800, b'b' as u16],
        "a/\u{FFFD}b",
    ),
    (
        &[b'a' as u16, b'\\' as u16, 0xd800, b'b' as u16, b'\\' as u16],
        "a/\u{FFFD}b/",
    ),
];

#[test]
fn invalid_utf16_seq_to_slash_lossy() {
    for (b, s) in INVALID_UTF16_TO_SLASH {
        let o = OsString::from_wide(b);
        let p = Path::new(&o);
        assert_eq!(p.to_slash_lossy(), *s, "{:x?}", b);
    }
}

#[test]
fn invalid_utf16_seq_to_slash() {
    for (b, _) in INVALID_UTF16_TO_SLASH {
        let o = OsString::from_wide(b);
        let p = Path::new(&o);
        assert_eq!(p.to_slash(), None, "{:x?}", b);
    }
}

const INVALID_UTF16_FROM_SLASH: &[(&[u16], &str)] = &[
    (
        &[b'a' as u16, b'/' as u16, 0xd800, b'b' as u16],
        "a\\\u{FFFD}b",
    ),
    (
        &[b'a' as u16, b'/' as u16, 0xd800, b'b' as u16, b'/' as u16],
        "a\\\u{FFFD}b\\",
    ),
];

#[test]
fn invalid_utf16_seq_pathbuf_from_slash() {
    for (b, s) in INVALID_UTF16_FROM_SLASH {
        let o = OsString::from_wide(b);
        let p = PathBuf::from_slash_lossy(&o);
        assert_eq!(p.to_str().unwrap(), *s, "{:x?}", b);
    }
}

#[test]
fn invalid_utf16_seq_cow_from_slash() {
    for (b, s) in INVALID_UTF16_FROM_SLASH {
        let o = OsString::from_wide(b);
        let p = Cow::from_slash_lossy(&o);
        assert_eq!(p.to_str().unwrap(), *s, "{:x?}", b);
    }
}
