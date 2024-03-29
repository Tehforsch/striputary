use std::fs::DirEntry;
use std::fs::{self};
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;

use crate::cut::get_excerpt_collection;
use crate::excerpt_collection::ExcerptCollection;
use crate::recording_session::RecordingSession;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionIdentifier {
    Old(usize),
    New,
}

pub struct SessionManager {
    output_dir: PathBuf,
    dirs: Vec<PathBuf>,
    new_dir: PathBuf,
    selected: Option<SessionIdentifier>,
}

impl SessionManager {
    pub fn new(dir: &Path) -> Self {
        let mut dirs = get_dirs(dir).unwrap();
        dirs.sort();
        dirs.reverse();
        let mut manager = Self {
            output_dir: dir.into(),
            dirs,
            new_dir: get_new_name(dir),
            selected: None,
        };
        manager.select_latest();
        manager
    }

    pub fn select(&mut self, identifier: SessionIdentifier) {
        self.selected = Some(identifier);
    }

    fn select_latest(&mut self) {
        let dirs = get_dirs(&self.output_dir).unwrap();
        self.selected = dirs
            .into_iter()
            .enumerate()
            .max_by_key(|(_, dir)| dir.metadata().unwrap().modified().unwrap())
            .map(|(index, _)| SessionIdentifier::Old(index));
    }

    pub fn select_new(&mut self) {
        self.selected = Some(SessionIdentifier::New);
    }

    pub fn is_currently_selected(&self, identifier: &SessionIdentifier) -> bool {
        self.selected
            .map(|selected| selected == *identifier)
            .unwrap_or(false)
    }

    pub fn get_currently_selected(&self) -> Option<PathBuf> {
        Some(match self.selected? {
            SessionIdentifier::Old(index) => self.dirs[index].clone(),
            SessionIdentifier::New => self.new_dir.clone(),
        })
    }

    pub fn get_currently_selected_collection(&self) -> Option<ExcerptCollection> {
        let session_dir = self.get_currently_selected()?;
        if session_dir.is_dir() {
            RecordingSession::from_parent_dir(&session_dir)
                .map(|session| get_excerpt_collection(session))
                .map_err(|x| {
                    println!("{}", x);
                    x
                })
                .ok()
        } else {
            None
        }
    }

    pub fn iter_relative_paths_with_indices(
        &self,
    ) -> impl Iterator<Item = (SessionIdentifier, String)> + '_ {
        Box::new(
            self.enumerate()
                .map(|(index, dir)| (index, dir.file_stem().unwrap().to_str().unwrap().to_owned())),
        )
    }

    fn enumerate(&self) -> impl Iterator<Item = (SessionIdentifier, &PathBuf)> {
        self.dirs
            .iter()
            .enumerate()
            .map(|(index, dir)| (SessionIdentifier::Old(index), dir))
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

fn get_new_name(output_dir: &Path) -> PathBuf {
    let date_string = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    output_dir.join(&date_string)
}
