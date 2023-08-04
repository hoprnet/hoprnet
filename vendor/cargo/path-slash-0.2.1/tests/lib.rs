use lazy_static::lazy_static;
use path_slash::{CowExt as _, PathBufExt as _, PathExt as _};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{PathBuf, MAIN_SEPARATOR};

lazy_static! {
    static ref FROM_SLASH_TESTS: Vec<(String, PathBuf)> = {
        [
            ("", ""),
            ("/", "/"),
            ("//", "/"),
            ("foo", "foo"),
            ("/foo", "/foo"),
            ("foo/", "foo/"),
            ("/foo/", "/foo/"),
            ("./foo", "./foo"),
            ("../foo", "../foo"),
            ("foo/.", "foo/."),
            ("foo/..", "foo/.."),
            ("foo/bar", "foo/bar"),
            ("foo//bar", "foo/bar"),
            ("foo/../bar", "foo/../bar"),
            ("foo/./bar", "foo/./bar"),
            ("/あ/い/う/え/お", "/あ/い/う/え/お"),
            ("あ/い/う/え/お/", "あ/い/う/え/お/"),
            ("/あ/い/う/え/お/", "/あ/い/う/え/お/"),
        ]
        .iter()
        .map(|item| {
            let (input, expected) = item;
            let expected = if cfg!(target_os = "windows") {
                let s = expected
                    .chars()
                    .map(|c| match c {
                        '/' => MAIN_SEPARATOR,
                        _ => c,
                    })
                    .collect::<String>();
                PathBuf::from(s)
            } else {
                PathBuf::from(expected)
            };
            (input.to_string(), expected)
        })
        .collect::<Vec<_>>()
    };
}

#[test]
fn from_slash() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        assert_eq!(&PathBuf::from_slash(input), expected);
    }
}

#[test]
fn from_slash_lossy() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input: &OsStr = input.as_ref();
        assert_eq!(&PathBuf::from_slash_lossy(input), expected);
    }
}

#[test]
fn from_backslash() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input = input.replace('/', r"\");
        assert_eq!(&PathBuf::from_backslash(input), expected);
    }
}

#[test]
fn from_backslash_lossy() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input = input.replace('/', r"\");
        let input: &OsStr = input.as_ref();
        assert_eq!(&PathBuf::from_backslash_lossy(input), expected);
    }
}

#[test]
fn cow_from_slash() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        assert_eq!(&Cow::from_slash(input), expected);
    }
}

#[test]
fn cow_from_slash_lossy() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input: &OsStr = input.as_ref();
        assert_eq!(&Cow::from_slash_lossy(input), expected);
    }
}

#[test]
fn cow_from_backslash() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input = input.replace('/', r"\");
        assert_eq!(&Cow::from_backslash(&input), expected);
    }
}

#[test]
fn cow_from_backslash_lossy() {
    for (input, expected) in FROM_SLASH_TESTS.iter() {
        let input = input.replace('/', r"\");
        let input: &OsStr = input.as_ref();
        assert_eq!(&Cow::from_backslash_lossy(input), expected);
    }
}

lazy_static! {
    static ref TO_SLASH_TESTS: Vec<(PathBuf, String)> = {
        [
            "",
            "/",
            "foo",
            "/foo",
            "foo/",
            "/foo/",
            "./foo",
            "../foo",
            "foo/..",
            "foo/bar",
            "foo/../bar",
            "あ/い/う/え/お/",
            "/あ/い/う/え/お",
            "/あ/い/う/え/お/",
        ]
        .iter()
        .map(|expected| {
            let input = if cfg!(target_os = "windows") {
                let s = expected
                    .chars()
                    .map(|c| match c {
                        '/' => MAIN_SEPARATOR,
                        _ => c,
                    })
                    .collect::<String>();
                PathBuf::from(s)
            } else {
                PathBuf::from(expected)
            };
            (input, expected.to_string())
        })
        .collect::<Vec<_>>()
    };
}

#[test]
fn to_slash_path() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(
            input.as_path().to_slash(),
            Some(Cow::Borrowed(expected.as_str()))
        );
    }
}

#[test]
fn to_slash_pathbuf() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(input.to_slash(), Some(Cow::Borrowed(expected.as_str())));
    }
}

#[test]
fn to_slash_cow() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(
            Cow::Borrowed(input.as_path()).to_slash(),
            Some(Cow::Borrowed(expected.as_str()))
        );
    }
}

#[test]
fn to_slash_lossy_path() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(&input.as_path().to_slash_lossy(), expected);
    }
}

#[test]
fn to_slash_lossy_pathbuf() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(&input.to_slash_lossy(), expected);
    }
}

#[test]
fn to_slash_lossy_cow() {
    for (input, expected) in TO_SLASH_TESTS.iter() {
        assert_eq!(&Cow::Borrowed(input.as_path()).to_slash_lossy(), expected);
    }
}

#[test]
fn from_slash_to_slash() {
    for (_, path) in TO_SLASH_TESTS.iter() {
        assert_eq!(
            PathBuf::from_slash(path).to_slash(),
            Some(Cow::Borrowed(path.as_str()))
        );
    }
}
