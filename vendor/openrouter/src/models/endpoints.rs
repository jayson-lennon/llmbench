use std::ops::Deref;

use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    models::{Architecture, Parameter, Pricing},
    providers::Provider,
};

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    pub data: Endpoints,
}

impl Deref for Response {
    type Target = Endpoints;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Endpoints {
    pub id: String,
    pub name: String,
    #[serde(with = "time::serde::timestamp")]
    pub created: OffsetDateTime,
    pub description: String,
    pub architecture: Architecture,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Endpoint {
    pub name: String,
    pub context_length: usize,
    pub pricing: Pricing,
    pub provider_name: Provider,
    pub tag: String,
    pub quantization: Option<String>,
    pub max_completion_tokens: Option<usize>,
    pub max_prompt_tokens: Option<usize>,
    pub status: Option<i64>,
    pub supported_parameters: Vec<Parameter>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::Response;

    #[test]
    fn endpoints() {
        let file = read_to_string("./responses/endpoints.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    // According to the docs at this time, the status in endpoints is a string. However, we received
    // an integer. Further investigation lead to this being a signed integer.
    #[test]
    fn status_changed_type() {
        let file = read_to_string("./responses/models/endpoints/openai-gpt3.5-turbo.json").unwrap();
        serde_json::from_str::<Response>(&file).unwrap();
    }

    #[test]
    fn regression_new_tag() {
        let file = read_to_string("./responses/models/endpoints/deepseek-r1.json").unwrap();
        serde_json::from_str::<Response>(&file).unwrap();
    }
}
