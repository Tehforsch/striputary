use std::fs::DirEntry;
use std::fs::{self};
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SessionIdentifier(usize);

pub struct SessionManager {
    dirs: Vec<PathBuf>,
}

impl SessionManager {
    pub fn new(dir: &Path) -> Self {
        let mut dirs = get_dirs(dir).unwrap();
        dirs.sort();
        dirs.reverse();
        Self { dirs }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> + '_ {
        self.dirs.iter()
    }
}

fn get_entries_with_predicate<F>(dir: &Path, predicate: F) -> Result<impl Iterator<Item = PathBuf>>
where
    F: Fn(&Path) -> bool,
{
    let entries = fs::read_dir(dir)?;
    let dir_entries: std::io::Result<Vec<DirEntry>> = entries.collect();
    Ok(dir_entries?
        .into_iter()
        .map(|entry| entry.path())
        .filter(move |path| predicate(path)))
}

fn iter_dirs(dir: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    get_entries_with_predicate(dir, Path::is_dir)
}

fn get_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(iter_dirs(dir)?.collect())
}
