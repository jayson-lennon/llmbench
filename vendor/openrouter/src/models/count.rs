//! <https://openrouter.ai/api/v1/models/count>

use std::ops::Deref;

use serde::Deserialize;

/// OpenRouter puts the returned data into a `data` field.
/// A [`Deref`] implementation makes this type transparent.
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Response {
    data: Count,
}

impl Deref for Response {
    type Target = Count;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Count {
    pub count: usize,
}

impl Count {
    pub fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::Response;

    #[test]
    fn count() {
        let file = read_to_string("./responses/count.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        assert_eq!(231, response.count);
    }
}
