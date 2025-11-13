//! <https://openrouter.ai/api/v1/credits>

use std::ops::Deref;

use serde::Deserialize;

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub data: Credits,
}

impl Deref for Response {
    type Target = Credits;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Credits {
    pub total_credits: f64,
    pub total_usage: f64,
}

impl Credits {
    pub fn total_credits(&self) -> f64 {
        self.total_credits
    }

    pub fn total_usage(&self) -> f64 {
        self.total_usage
    }

    pub fn balance(&self) -> f64 {
        self.total_credits - self.total_usage
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::{Credits, Response};

    #[test]
    fn credits() {
        let file = read_to_string("./responses/credits.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Credits {
            total_credits: 50.0,
            total_usage: 42.0,
        };

        assert_eq!(expected, *response);
        assert_eq!(8.0, response.balance());
    }
}
