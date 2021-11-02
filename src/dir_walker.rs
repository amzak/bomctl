use std::path::{PathBuf};
use std::fs::{self, DirEntry};
use std::io;

pub struct DirWalker {
  iter: fs::ReadDir,
  recursive: bool,
  path_stack: Vec<DirEntry>
}

impl DirWalker {
  pub fn new(base_dir: &PathBuf, recursive: bool) -> Result<DirWalker, io::Error> {
    let iter = fs::read_dir(base_dir)?;

    return Ok(DirWalker {
      path_stack: Vec::new(),
      recursive: recursive,
      iter: iter
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
        if path_buf.is_dir() && self.recursive {
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
mod dir_walker_tests {
  use super::*;

  #[test]
  fn internal() {
      let current = std::env::current_dir().unwrap();
      println!("{:?}", current);
      let result = DirWalker::new(&current, true);

      if let Ok(walker) = result {
        for path in walker {
            println!("{:?}", path);
        }
      }
  }
}