use std::ops::Deref;

use serde::Deserialize;
use time::OffsetDateTime;

use crate::providers::Provider;

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Deserialize, Debug, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub data: Generation,
}

impl Deref for Response {
    type Target = Generation;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Generation {
    pub id: String,
    pub upstream_id: Option<String>,
    pub total_cost: f64,
    pub cache_discount: Option<f64>,
    pub provider_name: Option<Provider>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub model: String,
    pub app_id: Option<u32>,
    pub streamed: Option<bool>,
    pub cancelled: Option<bool>,
    pub latency: Option<u32>,
    pub moderation_latency: Option<u32>,
    pub generation_time: Option<f64>,
    pub tokens_prompt: Option<u32>,
    pub tokens_completion: Option<u32>,
    pub native_tokens_prompt: Option<u32>,
    pub native_tokens_completion: Option<u32>,
    pub native_tokens_reasoning: Option<u32>,
    pub num_media_prompt: Option<u32>,
    pub num_media_completion: Option<u32>,
    pub num_search_results: Option<u32>,
    pub origin: String,
    pub is_byok: bool,
    // This could probably be an enum, but I don't know all reasons yet.
    pub finish_reason: FinishReason,
    pub native_finish_reason: Option<String>,
    pub usage: f64,
}

// Taken from https://openrouter.ai/docs/api-reference/overview#finish-reason
#[derive(Deserialize, Debug, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub enum FinishReason {
    #[serde(rename = "tool_calls")]
    ToolCalls,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
    #[serde(rename = "error")]
    Error,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use time::{Date, Month, Time};

    use super::*;

    #[test]
    fn generation() {
        let file = read_to_string("./responses/generation.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Generation {
            id: "gen-1737842317-9ifH6Yzo5icEdfD0I8SI".to_string(),
            upstream_id: None,
            total_cost: 0.0,
            cache_discount: None,
            provider_name: Some(Provider::GoogleAIStudio),
            created_at: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::January, 25).unwrap(),
                Time::from_hms_micro(21, 58, 42, 249238).unwrap(),
            ),
            model: "google/gemini-2.0-flash-thinking-exp-01-21:free".to_string(),
            app_id: Some(167635),
            streamed: Some(true),
            cancelled: Some(false),
            latency: Some(3134),
            moderation_latency: None,
            generation_time: Some(1.0),
            tokens_prompt: Some(10),
            tokens_completion: Some(30),
            native_tokens_prompt: Some(4),
            native_tokens_completion: Some(521),
            native_tokens_reasoning: None,
            num_media_prompt: None,
            num_media_completion: None,
            num_search_results: None,
            origin: "https://openrouter.ai/".to_string(),
            is_byok: false,
            finish_reason: FinishReason::Stop,
            native_finish_reason: Some("STOP".to_string()),
            usage: 0.0,
        };

        assert_eq!(expected, response.data);
    }

    #[test]
    fn generation2() {
        let file = read_to_string("./responses/generation/2025-03-03.json").unwrap();
        serde_json::from_str::<Response>(&file).unwrap();
    }
}
