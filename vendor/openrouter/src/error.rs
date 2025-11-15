//! <https://openrouter.ai/docs/errors>
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenRouter puts the returned data into a `error` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub error: Error,
    pub user_id: Option<String>,
}

impl Deref for Response {
    type Target = Error;
    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Error {
    /// Please refer to the OpenRouter documentation on what the
    /// various codes mean. Some codes get reused for various things.
    /// For example 404 is not listed, but is returned when trying to
    /// look at a generation ID that does not exist.
    pub code: u16,
    pub message: String,
    pub metadata: Option<Value>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error(code: {}, message: {}, metadata: {:?}",
            self.code, self.message, self.metadata
        )
    }
}

impl Error {
    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{Error, Response};

    #[test]
    fn count() {
        let file = read_to_string("./responses/missing_credentials.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Error {
            code: 401,
            message: "No auth credentials found".to_string(),
            metadata: None,
        };

        assert_eq!(expected, response.error);
    }

    #[test]
    fn internal_error() {
        let file = read_to_string("./responses/error/internal_error.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Error {
            code: 502,
            message: "Provider returned error".to_string(),
            metadata: Some(json!({
                "raw": "{\n  \"error\": {\n    \"code\": 500,\n    \"message\": \"An internal error has occurred. Please retry or report in https://developers.generativeai.google/guide/troubleshooting\",\n    \"status\": \"INTERNAL\"\n  }\n}\n",
                "provider_name": "Google AI Studio",
                "isDownstreamPipeClean": true,
                "isErrorUpstreamFault": true,
            })),
        };

        assert_eq!(expected, response.error);
    }
}
