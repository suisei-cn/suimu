use std::ffi::{OsStr, OsString};
use std::fs::DirEntry;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub trait FromInteractive: Sized {
    fn from_interactive() -> Result<Self>;
}

#[macro_export]
macro_rules! get_answer {
    ($answers: expr, $token: expr$(,)?) => {{
        get_answer!($answers, as_string, $token)
    }};
    ($answers: expr, $as_type: ident, $token: expr$(,)?) => {{
        let ans: &Answers = &$answers;
        let token: &str = $token;
        ans.get(token).unwrap().$as_type().unwrap().to_owned()
    }};
}

pub trait IsHidden {
    fn is_hidden(&self) -> bool;
}

impl IsHidden for Path {
    fn is_hidden(&self) -> bool {
        match self.file_name() {
            Some(filename) => filename.as_bytes()[0] == b'.',
            None => false,
        }
    }
}

impl IsHidden for PathBuf {
    fn is_hidden(&self) -> bool {
        self.as_path().is_hidden()
    }
}

impl IsHidden for DirEntry {
    fn is_hidden(&self) -> bool {
        self.path().is_hidden()
    }
}

pub trait Prefix {
    fn starts_with(&self, other: &Self) -> bool;
}

impl Prefix for OsString {
    fn starts_with(&self, other: &Self) -> bool {
        self.as_os_str().starts_with(other)
    }
}

impl Prefix for OsStr {
    fn starts_with(&self, other: &Self) -> bool {
        self.to_ascii_lowercase()
            .as_bytes()
            .iter()
            .zip(other.to_ascii_lowercase().as_bytes().iter())
            .fold(true, |acc, (a, b)| acc && a == b)
    }
}

#[test]
fn test_hidden() {
    assert!(PathBuf::from("/path/of/.hidden").is_hidden());
    assert!(!PathBuf::from("/path/of/not/hidden").is_hidden());
    assert!(PathBuf::from("/utf字符串/.测试hidden").is_hidden());
}

#[test]
fn test_prefix() {
    let a = OsString::from("abc测试测试");
    let b = OsString::from("abc测");
    let c = OsString::from("abc测测试");
    let d = OsString::from("bbc");

    assert!(a.starts_with(&b));
    assert!(c.starts_with(&b));
    assert!(!c.starts_with(&a));
    assert!(!c.starts_with(&d));
}
