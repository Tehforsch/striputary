mod plot;
mod session_gui;
mod session_selector;

use crate::config::Config;
use crate::recording_session::SessionPath;
use iced::application::application;
use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Element, Subscription, Task, Theme};
use log::error;

use self::session_gui::{SessionGui, SessionMessage};
use self::session_selector::SessionSelector;

const SCROLLBAR_WIDTH: f32 = 20.0;

pub struct Gui {
    session: Option<SessionGui>,
    session_selector: SessionSelector,
}

impl Gui {
    fn select_session(&mut self, path: SessionPath) {
        let session_gui = match SessionGui::new(path) {
            Err(e) => {
                error!("{}", e);
                return;
            }
            Ok(session_gui) => session_gui,
        };
        self.session = Some(session_gui);
    }

    pub fn start(config: &Config) {
        application("Striputary", Gui::update, Gui::view)
            .subscription(Gui::subscription)
            .theme(|_| Theme::CatppuccinFrappe)
            .run_with({
                let config = config.clone();
                move || Gui::new(&config)
            })
            .unwrap()
    }
}

impl Gui {
    fn new(config: &Config) -> (Self, Task<Message>) {
        let session_selector = SessionSelector::new(&config);
        let session = session_selector.selected().clone();
        let mut gui = Self {
            session_selector,
            session: None,
        };
        if let Some(session) = session {
            gui.select_session(session.clone());
        }
        (gui, Task::none())
    }

    fn update(&mut self, m: Message) {
        match m {
            Message::SelectSession(path) => {
                self.select_session(path);
            }
            Message::SessionMessage(m) => {
                if let Some(ref mut session) = self.session {
                    session.update(m)
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let selector = self.session_selector.view();
        let session_view = scrollable(
            self.session
                .as_ref()
                .map(|session| session.view())
                .unwrap_or(row![].into())
                .map(|message| Message::SessionMessage(message)),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(SCROLLBAR_WIDTH),
        ));
        let label = text("Select session:");
        let cut_songs =
            button("Cut songs").on_press(Message::SessionMessage(SessionMessage::CutSongs));
        row![
            column![label, selector, Space::with_height(50.0), cut_songs],
            session_view
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(ref session) = self.session {
            session.subscription().map(|m| Message::SessionMessage(m))
        } else {
            Subscription::none()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    SelectSession(SessionPath),
    SessionMessage(SessionMessage),
}
