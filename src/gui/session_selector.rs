use iced::{widget::pick_list, Element};

use crate::{config::Config, recording_session::SessionPath, session_manager::SessionManager};

use super::Message;

pub struct SessionSelector {
    sessions: Vec<SessionPath>,
    selected: Option<SessionPath>,
}

impl SessionSelector {
    pub fn new(config: &Config) -> SessionSelector {
        let manager = SessionManager::new(&config.output_dir);
        let sessions = manager.iter().cloned().collect();
        Self {
            sessions,
            selected: manager.most_recent(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        pick_list(
            &self.sessions[..],
            self.selected.clone(),
            Message::SelectSession,
        )
        .into()
    }

    pub fn selected(&self) -> Option<SessionPath> {
        self.selected.clone()
    }
}
