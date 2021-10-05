use std::{fs::{self, DirEntry}, path::{Path, PathBuf}};

use anyhow::Result;

fn get_entries_with_predicate<F>(
    folder: &Path,
    predicate: F,
) -> Result<impl Iterator<Item = PathBuf>>
where
    F: Fn(&Path) -> bool,
{
    let entries = fs::read_dir(folder)?;
    let dir_entries: std::io::Result<Vec<DirEntry>> = entries.collect();
    Ok(dir_entries?
        .into_iter()
        .map(|entry| entry.path())
        .filter(move |path| predicate(path))
        .map(|path| path.to_owned()))
}

fn iter_folders(folder: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    get_entries_with_predicate(folder, Path::is_dir)
}

pub fn get_folders(folder: &Path) -> Result<Vec<PathBuf>> {
    Ok(iter_folders(folder)?.collect())
}
