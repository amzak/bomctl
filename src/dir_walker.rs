use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::io;

pub struct DirWalker {
  iter: fs::ReadDir,
  path_stack: Vec<DirEntry>
}

impl DirWalker {
  pub fn new(baseDir: &PathBuf) -> Result<DirWalker, io::Error> {
    let iter = fs::read_dir(baseDir)?;
    let vec = Vec::new();

    return Ok(DirWalker {
      path_stack: vec,
      iter
    });
  }

  fn iter_on(&mut self, dir_entry: DirEntry) {
    let path = dir_entry.path();
    self.iter = fs::read_dir(path).unwrap();
  }
}

impl Iterator for DirWalker {
  type Item = PathBuf;

  fn next(&mut self) -> Option<PathBuf> {
    let path = self.iter.next();

    match path {
      Some(x) => {
        let result = x.unwrap();
        let path_buf = result.path();
        if path_buf.is_dir() {
          self.path_stack.push(result);
          return self.next();
        }
        return Some(result.path());
      },
      None => {
        let path_from_stack = self.path_stack.pop();

        if let Some(path) = path_from_stack {
            self.iter_on(path);
            return self.next();
        }
        else {
          return None;
        }
      }
    }
  }
}

#[cfg(test)]
mod dirWalkerTests {
  use super::*;

  #[test]
  fn internal() {
      let currentDir = env::current_dir().unwrap();
      println!("{:?}", currentDir);
      let walkerResult = DirWalker::new(&currentDir);

      if let Ok(walker) = walkerResult {
        for path in walker {
            println!("{:?}", path);
        }
      }
  }
}