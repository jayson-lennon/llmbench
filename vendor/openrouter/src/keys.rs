//! <https://openrouter.ai/api/v1/auth/key>

use std::ops::Deref;

use serde::Deserialize;

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub data: Key,
}

impl Deref for Response {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Key {
    pub label: String,
    pub limit: Option<f64>,
    pub usage: Option<f64>,
    pub limit_remaining: Option<f64>,
    pub is_free_tier: bool,
    pub rate_limit: RateLimit,
    pub is_provisioning_key: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct RateLimit {
    pub requests: usize,
    pub interval: String,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::{Key, RateLimit, Response};

    #[test]
    fn key() {
        let file = read_to_string("./responses/key.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Key {
            label: "sk-or-v1-116...0ce".to_string(),
            limit: Some(5.0),
            usage: Some(0.1015644762),
            limit_remaining: Some(4.8984355238),
            is_free_tier: false,
            rate_limit: RateLimit {
                requests: 50,
                interval: "10s".to_string(),
            },
            is_provisioning_key: false,
        };

        assert_eq!(expected, *response);
    }
}
