//! This crate exports a single trait, `Lexiclean`, with a single method,
//! `lexiclean`, implemented on `&Path`, that performs lexical path cleaning.
//!
//! Lexical path cleaning simplifies paths without looking at the underlying
//! filesystem. This means:
//!
//! - if `file` is a file and not a directory, the path `file/..` will fail to
//!   resolve. Lexiclean resolves this to `.`
//!
//! - `Path::canonicalize` returns `io::Result<PathBuf>`, because it must make
//!   system calls that might fail. Lexiclean does not make system calls and
//!   thus cannot fail.
//!
//! - The path returned by lexiclean will only contain components present in
//!   the input path. This can make the resultant paths more legible for users,
//!   since `foo/..` will resolve to `.`, and not `/Some/absolute/directory`.
//!
//! - Lexiclean does not respect symlinks.
//!
//! Additional test cases and bug fixes are most welcome!

use std::path::{Component, Path, PathBuf};

pub trait Lexiclean {
  fn lexiclean(self) -> PathBuf;
}

impl Lexiclean for &Path {
  fn lexiclean(self) -> PathBuf {
    use Component::*;

    let mut components = Vec::new();

    for component in self.components() {
      match component {
        CurDir => {}
        Normal(_) | Prefix(_) | RootDir => components.push(component),
        ParentDir => match components.last() {
          Some(CurDir) => unreachable!(),
          Some(Normal(_)) => {
            components.pop();
          }
          Some(ParentDir) | Some(Prefix(_)) | None => components.push(component),
          Some(RootDir) => {}
        },
      }
    }

    if components.is_empty() {
      components.push(CurDir);
    }

    components.into_iter().collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[track_caller]
  fn case(path: &str, want: &str) {
    assert_eq!(Path::new(path).lexiclean(), Path::new(want));
    assert_eq!(Path::new(want).lexiclean(), Path::new(want));
  }

  #[test]
  fn empty_path_maps_to_current_dir() {
    case("", ".");
  }

  #[test]
  fn single_current_dir_is_preserved() {
    case(".", ".");
  }

  #[test]
  fn multiple_current_dirs_are_collapsed() {
    case("././.", ".");
  }

  #[test]
  fn leading_parent_dir_is_preserved() {
    case("..", "..");
  }

  #[test]
  fn multiple_parent_dirs_are_preserved() {
    case("../../..", "../../..");
  }

  #[test]
  fn trailing_slash_is_removed() {
    case("foo/", "foo");
  }

  #[test]
  fn leading_current_dir_is_removed() {
    case("./foo", "foo");
  }

  #[test]
  fn trailing_parent_dir_after_current_dir_is_preserved() {
    case("./..", "..");
  }

  #[test]
  fn trailing_current_dir_is_removed() {
    case("foo/.", "foo");
  }

  #[test]
  fn intermediate_current_dir_is_removed() {
    case("foo/./bar", "foo/bar");
  }

  #[test]
  fn parent_dir_after_root_is_removed() {
    case("/..", "/");
  }

  #[test]
  fn current_dir_after_root_is_removed() {
    case("/.", "/");
  }

  #[test]
  fn multiple_slashes_are_removed() {
    case("//foo//bar//", "/foo/bar");
  }

  #[test]
  fn normal_after_root_is_preserved() {
    case("/foo", "/foo");
  }

  #[test]
  fn intermediate_parent_dir_is_removed() {
    case("/foo/../bar", "/bar");
  }

  #[test]
  fn trailing_parent_dir_pops_normal() {
    case("/foo/bar/..", "/foo");
  }

  #[test]
  fn trailing_parent_dir_pops_normal_before_current() {
    case("/foo/bar/./..", "/foo");
  }

  #[test]
  fn normal_is_preserved() {
    case("foo", "foo");
  }

  #[test]
  fn parent_dir_after_normal_is_current() {
    case("foo/..", ".");
  }

  #[test]
  fn parent_dir_after_popped_normal_is_preserved() {
    case("foo/../..", "..");
  }

  #[test]
  fn parent_dir_pops_normal_after_parent_dir() {
    case("../foo/..", "..");
  }

  #[test]
  fn normal_after_preserved_parent_dir_is_preserved() {
    case("foo/../../bar", "../bar");
  }

  #[test]
  fn parent_dirs_after_root_are_removed_after_pop() {
    case("/foo/../..", "/");
  }

  #[test]
  fn normal_after_removed_parent_dir_is_preserved() {
    case("/../foo", "/foo");
  }

  #[test]
  #[cfg(windows)]
  fn parent_dir_after_disk_is_removed() {
    case(r"C:\..", r"C:\");
  }

  #[test]
  #[cfg(windows)]
  fn parent_dir_after_bare_disk_is_preserved() {
    case("C:..", "C:..");
  }

  #[test]
  #[cfg(windows)]
  fn parent_dir_pops_normal_after_bare_disk() {
    case(r"C:foo\..", "C:");
  }

  #[test]
  #[cfg(windows)]
  fn parent_dir_after_unc_share_is_removed() {
    case(r"\\foo\bar\..", r"\\foo\bar\");
  }
}
