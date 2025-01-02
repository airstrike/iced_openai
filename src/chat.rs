// chat.rs
use iced::widget::{
    button, column, container, horizontal_space, markdown, row, scrollable,
    text, text_input,
};
use iced::{alignment::Horizontal, Element, Fill, Task};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::assistant::{self, Client};
use crate::THEME;

#[derive(Debug, Clone, PartialEq)]
pub enum Sender {
    User,
    Assistant,
}

#[derive(Debug, Default, Clone)]
pub struct Chat {
    pub messages: Vec<ChatMessage>,
    pub input_value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    InputChanged(String),
    Submit,
    LinkClicked(markdown::Url),

    ResponseReceived(Result<assistant::Response, String>),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub content: String,
    pub parsed_content: Vec<markdown::Item>,
    pub sender: Sender,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct TransportMessage {
    pub content: String,
    pub sender: Sender,
}

impl From<&ChatMessage> for TransportMessage {
    fn from(msg: &ChatMessage) -> Self {
        Self {
            content: msg.content.clone(),
            sender: msg.sender.clone(),
        }
    }
}

impl ChatMessage {
    pub fn new(content: String, sender: Sender, timestamp: u64) -> Self {
        let parsed_content = markdown::parse(&content).collect();
        Self {
            content,
            parsed_content,
            sender,
            timestamp,
        }
    }
}

pub enum Action {
    Back,
    NewMessage(ChatMessage, Task<Message>),
}

pub fn update(
    message: Message,
    chat: &mut Chat,
    client: Client<async_openai::config::OpenAIConfig>,
) -> Option<Action> {
    match message {
        Message::Back => Some(Action::Back),
        Message::InputChanged(value) => {
            chat.input_value = value;
            None
        }
        Message::LinkClicked(link) => {
            open::that_in_background(link.as_str());
            None
        }
        Message::Submit => {
            if chat.input_value.is_empty() {
                return None;
            }

            let content = std::mem::take(&mut chat.input_value);
            let user_message = ChatMessage::new(
                content.clone(),
                Sender::User,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

            // Convert messages to transport format
            let transport_messages: Vec<TransportMessage> =
                chat.messages.iter().map(TransportMessage::from).collect();

            let transport_message = TransportMessage::from(&user_message);

            let task = Task::future(async move {
                assistant::request(
                    client,
                    &transport_messages,
                    transport_message,
                )
                .await
            })
            .map(Message::ResponseReceived);

            Some(Action::NewMessage(user_message, task))
        }
        Message::ResponseReceived(Ok(response)) => {
            let message = ChatMessage::new(
                response.content,
                Sender::Assistant,
                response.timestamp,
            );
            Some(Action::NewMessage(message, Task::none()))
        }
        Message::ResponseReceived(Err(error)) => {
            let message = ChatMessage::new(
                format!("Error: {}", error),
                Sender::Assistant,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            Some(Action::NewMessage(message, Task::none()))
        }
    }
}

pub fn view(chat: &Chat) -> Element<Message> {
    let messages = chat.messages.iter().fold(
        column![].spacing(10).width(Fill),
        |col, msg| {
            col.push(
                container(
                    column![
                        text(format!(
                            "{} • {}",
                            match msg.sender {
                                Sender::User => "You",
                                Sender::Assistant => "Assistant",
                            },
                            chrono::DateTime::from_timestamp(
                                msg.timestamp as i64,
                                0
                            )
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_else(|| "Invalid time".to_string())
                        ))
                        .size(12),
                        markdown(
                            &msg.parsed_content,
                            markdown::Settings::with_text_size(14),
                            markdown::Style::from_palette(THEME.palette()),
                        )
                        .map(Message::LinkClicked)
                    ]
                    .align_x(match msg.sender {
                        Sender::User => Horizontal::Right,
                        Sender::Assistant => Horizontal::Left,
                    }),
                )
                .style(|theme| {
                    if matches!(msg.sender, Sender::User) {
                        container::rounded_box(theme).background(
                            theme.extended_palette().background.weak.color,
                        )
                    } else {
                        container::rounded_box(theme).background(
                            theme.extended_palette().primary.base.color,
                        )
                    }
                })
                .padding(10)
                .align_x(if matches!(msg.sender, Sender::User) {
                    Horizontal::Right
                } else {
                    Horizontal::Left
                })
                .width(Fill),
            )
        },
    );

    container(
        column![
            row![horizontal_space(), button("Back").on_press(Message::Back),],
            container(
                scrollable(container(messages).padding(20))
                    .anchor_bottom()
                    .height(Fill)
            )
            .style(container::bordered_box),
            row![
                text_input("Type a message...", &chat.input_value)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::Submit)
                    .padding(5)
                    .width(Fill),
                button("Send").on_press(Message::Submit),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .width(Fill)
        .height(Fill),
    )
    .padding(20)
    .center(Fill)
    .into()
}
