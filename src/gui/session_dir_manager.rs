use std::path::PathBuf;
use std::{
    fs::{self, DirEntry},
    path::Path,
};

use anyhow::Result;
use chrono::Local;

pub struct SessionDirManager {
    output_dir: PathBuf,
    dirs: Vec<PathBuf>,
    selected: Option<usize>,
}

impl SessionDirManager {
    pub fn new(dir: &Path) -> Self {
        let dirs = get_dirs(dir).unwrap();
        let mut manager = Self {
            output_dir: dir.into(),
            dirs,
            selected: None,
        };
        manager.select_latest();
        manager
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    fn select_latest(&mut self) {
        let dirs = get_dirs(&self.output_dir).unwrap();
        self.selected = dirs
            .into_iter()
            .enumerate()
            .max_by_key(|(_, dir)| dir.metadata().unwrap().modified().unwrap())
            .map(|(index, _)| index);
    }

    fn get_new(&self) -> PathBuf {
        let date_string = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        self.output_dir.join(&date_string)
    }

    pub fn get_currently_selected(&self) -> PathBuf {
        match self.selected {
            Some(index) => self.dirs[index].clone(),
            None => self.get_new(),
        }
    }

    pub fn iter_relative_paths(&self) -> Box<Iterator<Item = String> + '_> {
        Box::new(
            self.dirs
                .iter()
                .map(|dir| dir.file_stem().unwrap().to_str().unwrap().to_owned()),
        )
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
        .filter(move |path| predicate(path))
        .map(|path| path.to_owned()))
}

fn iter_dirs(dir: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    get_entries_with_predicate(dir, Path::is_dir)
}

pub fn get_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(iter_dirs(dir)?.collect())
}
