use std::path::PathBuf;

use iced::{
    widget::{button, Column},
    Element,
};

use crate::{config::Config, session_manager::SessionManager};

use super::Message;

pub struct SessionSelector {
    sessions: Vec<PathBuf>,
}

impl SessionSelector {
    pub fn new(config: &Config) -> SessionSelector {
        let manager = SessionManager::new(&config.output_dir);
        let sessions = manager.iter().cloned().collect();
        Self { sessions }
    }

    pub fn view(&self) -> Element<Message> {
        Column::with_children(self.sessions.iter().map(|path| {
            button(
                path.file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("<invalid path>"),
            )
            .on_press(Message::SelectSession(path.into()))
            .into()
        }))
        .into()
    }
}
