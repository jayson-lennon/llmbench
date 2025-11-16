use openrouter::completions::{
    Response,
    request::{Content, Message},
    response::Choice,
};

/// Create a new user message.
pub fn user_message<M>(msg: M) -> Message
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
pub fn assistant_message<M>(msg: M) -> Message
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
