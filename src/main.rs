use iced::{Element, Size, Task};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

mod assistant;
mod chat;
mod error;
mod list;

use assistant::{client, Client};
pub use chat::{Chat, ChatMessage, Sender};

pub const THEME: iced::Theme = iced::Theme::Light;

fn main() -> iced::Result {
    iced::application("iced • OpenAI chat example", App::update, App::view)
        .window_size(Size::new(800.0, 600.0))
        .theme(App::theme)
        .centered()
        .run_with(App::new)
}

#[derive(Debug)]
enum Screen {
    List,
    Chat(Option<usize>),
}

#[derive(Debug)]
enum Message {
    List(list::Message),
    Chat(Option<usize>, chat::Message),
}

struct App {
    screen: Screen,
    chats: HashMap<usize, chat::Chat>,
    pending_chat: chat::Chat,
    client: Client<async_openai::config::OpenAIConfig>,
    next_chat_id: AtomicUsize,
}

impl App {
    fn theme(&self) -> iced::Theme {
        THEME
    }

    fn new() -> (Self, Task<Message>) {
        let client = match client() {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Failed to initialize OpenAI client: {}", e);
                std::process::exit(1);
            }
        };

        (
            Self {
                screen: Screen::List,
                chats: HashMap::new(),
                pending_chat: Chat::default(),
                client,
                next_chat_id: AtomicUsize::new(0),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::List(list::Message::NewChat) => {
                self.screen = Screen::Chat(None);
                Task::none()
            }
            Message::List(list::Message::SelectChat(id)) => {
                self.screen = Screen::Chat(Some(id));
                Task::none()
            }
            Message::Chat(chat_id, msg) => {
                let chat = match chat_id {
                    Some(id) => self.chats.get_mut(&id),
                    None => Some(&mut self.pending_chat),
                };

                if let Some(action) = chat.and_then(|chat| {
                    chat::update(msg, chat, self.client.clone())
                }) {
                    match action {
                        chat::Action::Back => {
                            self.screen = Screen::List;
                            self.pending_chat = Chat::default();
                            Task::none()
                        }
                        chat::Action::NewMessage(message, task) => {
                            match chat_id {
                                Some(id) => {
                                    if let Some(chat) = self.chats.get_mut(&id)
                                    {
                                        chat.messages.push(message);
                                    }
                                    task.map(move |msg| {
                                        Message::Chat(Some(id), msg)
                                    })
                                }
                                None => {
                                    // This is the first message in a chat,
                                    // so we insert it into the hashmap.
                                    self.pending_chat.messages.push(message);
                                    let id = self
                                        .next_chat_id
                                        .fetch_add(1, Ordering::SeqCst);
                                    let chat =
                                        std::mem::take(&mut self.pending_chat);
                                    self.chats.insert(id, chat);
                                    self.screen = Screen::Chat(Some(id));
                                    task.map(move |msg| {
                                        Message::Chat(Some(id), msg)
                                    })
                                }
                            }
                        }
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::List => list::view(&self.chats).map(Message::List),
            Screen::Chat(id) => {
                let chat = match id {
                    Some(id) => &self.chats[id],
                    None => &self.pending_chat,
                };
                chat::view(chat).map(move |msg| Message::Chat(*id, msg))
            }
        }
    }
}
