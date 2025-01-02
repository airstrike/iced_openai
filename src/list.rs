use iced::widget::{button, column, container, horizontal_space, row, text};
use iced::Alignment::Center;
use iced::{Element, Fill};
use std::collections::HashMap;

use crate::Chat;

#[derive(Debug, Clone)]
pub enum Message {
    NewChat,
    SelectChat(usize),
}

pub fn view(chats: &HashMap<usize, Chat>) -> Element<'_, Message> {
    let mut chat_list = column![].spacing(10).width(Fill);

    for (id, chat) in chats {
        // Take the first 50 characters of the last message. Append a '...' if
        // the message is longer.
        let last_message = chat
            .messages
            .last()
            .map(|message| {
                let mut content = message.content.clone();
                if content.len() > 50 {
                    content.truncate(50);
                    content.push_str("...");
                }
                content
            })
            .unwrap_or_else(|| "No messages yet".to_string());
        chat_list = chat_list.push(
            button(
                row![column![
                    text(format!("Chat {}", id)).size(16),
                    text(last_message).size(12).style(text::secondary)
                ]
                .width(Fill)
                .padding(10)]
                .width(Fill),
            )
            .on_press(Message::SelectChat(*id))
            .width(Fill),
        );
    }

    container(
        column![
            row![
                horizontal_space(),
                button("New Chat").on_press(Message::NewChat),
            ]
            .align_y(Center),
            column![text("Chats").size(32), chat_list]
                .spacing(20)
                .height(Fill)
                .width(Fill),
        ]
        .spacing(5),
    )
    .padding(20)
    .center(Fill)
    .into()
}
