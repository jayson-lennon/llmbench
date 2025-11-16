use openrouter::completions::{
    Response,
    request::{Content, Message},
    response::Choice,
};

/// Create a new user message.
pub(in crate::feat::bench) fn user_message<M>(msg: M) -> Message
where
    M: Into<String>,
{
    let msg = msg.into();
    Message::User {
        content: Content::Plain(msg),
        name: None,
        cache_control: None,
    }
}

/// Create a new assistant message.
pub(in crate::feat::bench) fn assistant_message<M>(msg: M) -> Message
where
    M: Into<String>,
{
    let msg = msg.into();
    Message::Assistant {
        content: Some(Content::Plain(msg)),
        name: None,
        tool_calls: None,
    }
}

pub(in crate::feat::bench) trait ResponseExt {
    fn get_assistant_message(&self) -> Option<String>;
}

impl ResponseExt for Response {
    fn get_assistant_message(&self) -> Option<String> {
        if let Some(choice) = self.choices.first() {
            match choice {
                Choice::NonStreaming(choice) => choice.message.content.clone(),
                _ => None,
            }
        } else {
            None
        }
    }
}

pub(in crate::feat::bench) trait StringBenchExt {
    fn alphanumeric_only(&self) -> String;
    fn lowercase(&self) -> String;
}

impl StringBenchExt for String {
    /// Includes:
    /// - characters
    /// - numbers
    /// - whitespace
    fn alphanumeric_only(&self) -> String {
        self.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn lowercase(&self) -> String {
        self.to_lowercase()
    }
}
