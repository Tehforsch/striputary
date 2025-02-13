mod plot;
mod session_gui;
mod session_selector;

use crate::config::Config;
use crate::recording_session::SessionPath;
use anyhow::Result;
use iced::application::application;
use iced::widget::{button, row};
use iced::{Element, Subscription, Task, Theme};
use log::error;

use self::session_gui::{SessionGui, SessionMessage};
use self::session_selector::SessionSelector;

pub struct Gui {
    session: Option<SessionGui>,
    session_selector: SessionSelector,
}

impl Gui {
    fn select_session(&mut self, path: SessionPath) -> Result<()> {
        self.session = Some(SessionGui::new(path)?);
        Ok(())
    }

    pub fn start(config: &Config) {
        application("Striputary", Gui::update, Gui::view)
            .subscription(Gui::subscription)
            .theme(|_| Theme::GruvboxDark)
            .run_with({
                let config = config.clone();
                move || Gui::new(&config)
            })
            .unwrap()
    }
}

// impl Application for Gui {
//     type Executor = executor::Default;

//     type Message = Message;

//     type Theme = Theme;

//     type Flags = Config;

//     fn title(&self) -> String {
//         "Striputary".to_string()
//     }

//     fn theme(&self) -> Theme {
//         Theme::GruvboxDark
//     }
// }

impl Gui {
    fn new(config: &Config) -> (Self, Task<Message>) {
        (
            Self {
                session_selector: SessionSelector::new(&config),
                session: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, m: Message) {
        match m {
            Message::SelectSession(path) => {
                if let Err(e) = self.select_session(path) {
                    error!("{}", e);
                }
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
        let session_view = self
            .session
            .as_ref()
            .map(|session| session.view())
            .unwrap_or(row![].into())
            .map(|message| Message::SessionMessage(message));
        let cut_songs =
            button("Cut songs").on_press(Message::SessionMessage(SessionMessage::CutSongs));
        row![session_view, cut_songs, selector].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(
            self.session
                .iter()
                .map(|s| s.subscription().map(|m| Message::SessionMessage(m))),
        )
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    SelectSession(SessionPath),
    SessionMessage(SessionMessage),
}
