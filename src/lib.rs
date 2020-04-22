//! This crate exports a single trait, `Lexiclean`, with a single method,
//! `lexiclean`, implemented on `&Path`, that performs lexical path cleaning.
//!
//! Lexical path cleaning simplifies paths without looking at the underlying
//! filesystem. This means:
//!
//! - Normally, if `file` is a file and not a directory, the path `file/..` will
//!   fail to resolve to. Lexiclean resolves this to `.`
//!
//! - `Path::canonicalize` returns `io::Result<PathBuf>`, because it must make
//!   system calls, that might fail. Lexiclean does not make system calls, and
//!   thus cannot fail.
//!
//! - The path returned by lexiclean will only contain components present in the
//!   input path. This can make the resultant paths more legible for users,
//!   since `foo/..` will resolve to `.`, and not `/Some/absolute/directory`.
//!
//! - Lexiclean does not respect symlinks.
//!
//! - Lexiclean has only been lightly tested. In particular, it has not been
//!   tested with windows paths, which are very complicated, and can contain
//!   many types of components that the author of this crate never contemplated.
//!
//!   Additional test cases and bug fixes are most welcome!
use std::path::{Component, Path, PathBuf};

pub trait Lexiclean {
  fn lexiclean(self) -> PathBuf;
}

impl Lexiclean for &Path {
  fn lexiclean(self) -> PathBuf {
    if self.components().count() <= 1 {
      return self.to_owned();
    }

    let mut components = Vec::new();

    for component in self
      .components()
      .filter(|component| component != &Component::CurDir)
    {
      if component == Component::ParentDir {
        match components.last() {
          Some(Component::Normal(_)) => {
            components.pop();
          }
          Some(Component::ParentDir) | None => components.push(component),
          _ => {}
        }
      } else {
        components.push(component);
      }
    }

    components.into_iter().collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[rustfmt::skip]
  fn simple() {
    fn case(path: &str, want: &str) {
      assert_eq!(Path::new(path).lexiclean(), Path::new(want));
    }

    case("",                       "");
    case(".",                      ".");
    case("..",                     "..");
    case("../../../",              "../../..");
    case("./",                     ".");
    case("./..",                   "..");
    case("./../.",                 "..");
    case("./././.",                ".");
    case("/." ,                    "/");
    case("/..",                    "/");
    case("/../../../../../../../", "/");
    case("/././",                  "/");
    case("//foo/bar//baz",         "/foo/bar/baz");
    case("/foo",                   "/foo");
    case("/foo/../bar",            "/bar");
    case("/foo/./bar/.",           "/foo/bar");
    case("/foo/bar/..",            "/foo");
    case("bar//baz",               "bar/baz");
    case("foo",                    "foo");
    case("foo/./bar",              "foo/bar");
  }
}
