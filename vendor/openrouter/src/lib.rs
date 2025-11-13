//! This library includes helpers and structs to communicate with OpenRouter.

extern crate core;

use reqwest::{
    Client, Response, StatusCode,
    header::{ACCEPT, HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue},
};
use serde_json::Value;
use thiserror::Error;

/// Request and response types for completion endpoints.
pub mod completions;
/// Request and response types for the credit endpoint.
pub mod credits;
/// General error type for various endpoints.
pub mod error;
/// Request and Response types for the generation endpoint.
pub mod generation;
/// Request and Response types for the keys endpoints.
pub mod keys;
/// Request and Response types for the model endpoints.
pub mod models;
/// Types and implementations for known available providers on OpenRouter.
pub mod providers;

use crate::{
    completions::Request,
    credits::Credits,
    generation::Generation,
    keys::Key,
    models::{Model, Parameter, endpoints::Endpoints},
};

/// If a custom client is used, this user agent is provided so that
/// user agents are still identifiable.
pub static DEFAULT_USER_AGENT: &str = concat!(
    "crates.io/crates/",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

/// There are types that serialize and deserialize into the
pub struct OpenRouter {
    client: Client,

    api_key: String,
    site_url: HeaderValue,
    site_title: HeaderValue,
}

impl OpenRouter {
    /// Sets up a OpenRouter helper with a default [`Client`].
    /// The only option being set is the user agent to identify the crate being used.
    pub fn new<S>(api_key: S) -> Self
    where
        S: Into<String>,
    {
        #[expect(clippy::expect_used, reason = "this can not panic")]
        Self {
            client: Client::builder()
                .user_agent(DEFAULT_USER_AGENT)
                .build()
                .expect("default user agent is valid, so building shouldn't fail"),
            api_key: api_key.into(),
            site_url: HeaderValue::from_static(""),
            site_title: HeaderValue::from_static(""),
        }
    }

    /// Replaces the default [`Client`] with a provided one.
    pub fn with_client(self, client: Client) -> Self {
        Self { client, ..self }
    }

    /// Sets the `http-referer` custom http header value.
    /// Together with [`Self::with_site_title`] this identifies the application on OpenRouter.
    pub fn with_site_url<S>(self, site_url: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let site_url = HeaderValue::try_from(site_url.into())?;

        Ok(Self { site_url, ..self })
    }

    /// Sets the `x-title` custom http header value.
    /// Together with [`Self::with_site_url`] this identifies the application on OpenRouter.
    pub fn with_site_title<S>(self, site_title: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let site_title = HeaderValue::try_from(site_title.into())?;

        Ok(Self { site_title, ..self })
    }

    async fn header(&self) -> Result<HeaderMap, Error> {
        let mut header = HeaderMap::new();

        header.insert(ACCEPT, HeaderValue::from_static("application/json"));

        if !self.site_url.is_empty() {
            header.insert(
                HeaderName::from_static("http-referer"),
                self.site_url.clone(),
            );
        }

        if !self.site_title.is_empty() {
            header.insert(HeaderName::from_static("x-title"), self.site_title.clone());
        }

        Ok(header)
    }

    /// Helper function for creating GET requests that attaches required and optional header values.
    async fn get(&self, url: &str) -> Result<Response, Error> {
        let header = self.header().await?;

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .headers(header)
            .send()
            .await?;

        Ok(response)
    }

    async fn post(&self, url: &str, body: Value) -> Result<Response, Error> {
        let header = self.header().await?;

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .headers(header)
            .body(body.to_string())
            .send()
            .await?;

        Ok(response)
    }

    /// Returns the amount of available models without model data.
    /// ---
    /// See: <https://openrouter.ai/docs/models>
    pub async fn models_count(&self) -> Result<usize, Error> {
        let response = self
            .get("https://openrouter.ai/api/v1/models/count")
            .await?;
        type_or_err(response, |r: models::count::Response| r.count).await
    }

    /// Returns all models available on OpenRouter. Optionally can be filtered with a list of
    /// parameters the model should support.
    /// ---
    /// See: <https://openrouter.ai/docs/models>
    pub async fn models(&self, parameters: &[Parameter]) -> Result<Vec<Model>, Error> {
        let response = if parameters.is_empty() {
            self.get("https://openrouter.ai/api/v1/models").await?
        } else {
            let parameters = parameters
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join(",");

            let url =
                format!("https://openrouter.ai/api/v1/models?supported_parameters={parameters}");
            self.get(&url).await?
        };

        type_or_err(response, |r: models::Response| r.data).await
    }

    /// This helper method uses [`Self::models`] and filters out the ones that are not free.
    pub async fn free_models(&self) -> Result<Vec<Model>, Error> {
        let all_models = self.models(&[]).await?;
        let free_models = all_models
            .into_iter()
            .filter(|m| m.pricing.free())
            .collect();

        Ok(free_models)
    }

    /// Using the ID provided in [`Self::models`], this API call will return a list of providers
    /// for the specific model. This allows filtering for certain supported features, context lengths
    /// or cost and can be used to specify or ignore providers for [`Self::completion`] and [`Self::chat_completion`].
    pub async fn model_endpoints<ID>(&self, id: ID) -> Result<Endpoints, Error>
    where
        ID: AsRef<str>,
    {
        let url = format!(
            "https://openrouter.ai/api/v1/models/{}/endpoints",
            id.as_ref()
        );

        let response = self.get(&url).await?;

        type_or_err(response, |r: models::endpoints::Response| r.data).await
    }

    /// Returns the amount of credits in the account and how many have been used.
    /// The difference of the two is the current balance.
    ///
    /// # Official OpenRouter Documentation
    /// <https://openrouter.ai/docs/crypto-api#detecting-low-balance>
    pub async fn credits(&self) -> Result<Credits, Error> {
        let response = self.get("https://openrouter.ai/api/v1/credits").await?;
        type_or_err(response, |r: credits::Response| r.data).await
    }

    /// This endpoint will use the provided API key and return information about it,
    /// like rate limits or the remaining credits.
    ///
    /// # Official OpenRouter Documentation
    /// <https://openrouter.ai/docs/api-reference/limits>
    pub async fn key(&self) -> Result<Key, Error> {
        let response = self.get("https://openrouter.ai/api/v1/auth/key").await?;
        type_or_err(response, |r: keys::Response| r.data).await
    }

    /// This endpoint returns metadata about a previous completion request.
    ///
    /// # Official OpenRouter Documentation
    /// <https://openrouter.ai/docs/api-reference/get-a-generation>
    pub async fn generation(&self, id: &str) -> Result<Generation, Error> {
        let url = format!("https://openrouter.ai/api/v1/generation?id={}", id);
        let response = self.get(&url).await?;

        type_or_err(response, |r: generation::Response| r.data).await
    }

    /// This endpoint should be preferred if the [`Request`] contains multiple messages.
    pub async fn chat_completion(&self, request: Request) -> Result<completions::Response, Error> {
        let response = self
            .post(
                "https://openrouter.ai/api/v1/chat/completions",
                serde_json::to_value(request)?,
            )
            .await?;

        type_or_err(response, |r: completions::Response| r).await
    }

    /// "Old school" text generation. This one should be preferred if a simple prompt is in the [`Request`].
    pub async fn completion(&self, request: Request) -> Result<completions::Response, Error> {
        let response = self
            .post(
                "https://openrouter.ai/api/v1/completions",
                serde_json::to_value(request)?,
            )
            .await?;

        type_or_err(response, |r: completions::Response| r).await
    }
}

/// Helper function that converts a response either into the type we expect if successful,
/// or our library error with a parsed OpenRouter error.
async fn type_or_err<T, R, E>(response: Response, extractor: E) -> Result<T, Error>
where
    E: Fn(R) -> T,
    R: for<'de> serde::Deserialize<'de>,
{
    let status = response.status();
    let content = response.text().await?;

    //println!("Response: {}", content);

    match status {
        StatusCode::OK => {
            match serialize_response(&content).await {
                Ok(r) => Ok(extractor(r)),
                Err(e) => {
                    // While not supposed to happen, after talks with OpenRouter admins,
                    // it would seem that for some providers an internal stream is started,
                    // and might then error out in the middle of response. This causes their
                    // backend to return a 200 while containing an error. In case they can not fix
                    // this, or don't want to, we wastefully try to parse it as an error.
                    // Should parsing that work fine, we're good, but in case that fails too we return
                    // the original error.

                    match serialize_response::<error::Response>(&content).await {
                        Ok(response) => Err(Error::OpenRouter(response.error)),
                        Err(_) => Err(e),
                    }
                }
            }
        }
        _ => {
            let response_type = serialize_response::<error::Response>(&content).await?;
            Err(Error::OpenRouter(response_type.error))
        }
    }
}

async fn serialize_response<R>(response: &str) -> Result<R, Error>
where
    R: for<'de> serde::Deserialize<'de>,
{
    // The below may seem a bit "manual", but helps in debugging responses from the API.
    match serde_json::from_str::<R>(response) {
        Ok(s) => Ok(s),
        Err(e) => {
            if cfg!(debug_assertions) {
                //println!("json serialization failed:\n{}", response.trim());
            }

            Err(Error::Serde(e))
        }
    }
}

/// [`OpenRouter`] specific error type. This will indicate what part of the communication with OpenRouter failed.
#[derive(Debug, Error)]
pub enum Error {
    /// Something with the request is wrong, or OpenRouter has a problem. Either way,
    /// more information can be found in [`error::Error`].
    #[error("OpenRouter returned an error")]
    OpenRouter(error::Error),
    /// [`reqwest`] encountered a problem. Most of the time this is network related.
    #[error("unexpected request error")]
    Reqwest(#[from] reqwest::Error),
    /// Serialization or deserialization encountered a problem. Usually this indicates a bug in the
    /// library or an API change. Please report these errors as issues on <https://gitlab.com/bit-refined/openrouter/-/issues>.
    #[error("de- or serialization failed")]
    Serde(#[from] serde_json::Error),
    /// Setting the application title or URL failed because the value itself is not a valid http header value.
    #[error("invalid header value")]
    InvalidHeader(#[from] InvalidHeaderValue),
}

#[cfg(all(test, feature = "integration_tests"))]
mod tests {
    use std::{collections::HashSet, env::var, time::Duration};

    use dotenv::dotenv;
    use reqwest::ClientBuilder;

    use super::{DEFAULT_USER_AGENT, Error, OpenRouter};
    use crate::{
        completions::{
            Request,
            request::{Content, Message, ProviderPreferences, Usage},
            response::Choice,
        },
        models::{InstructType, Modality, Parameter, Tokenizer},
        providers::Provider,
    };

    #[expect(clippy::expect_used)]
    pub(crate) fn test_instance() -> OpenRouter {
        let _ = dotenv();
        let api_key = var("OPENROUTER_API_KEY")
            .expect("either OPENROUTER_API_KEY should be set or a .env file has to exist with the key in it");

        let client = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        OpenRouter::new(api_key)
            .with_client(client)
            .with_site_title(DEFAULT_USER_AGENT)
            .unwrap()
            .with_site_url("https://crates.io/crates/openrouter/")
            .unwrap()
    }

    #[tokio::test]
    async fn count() {
        let openrouter = test_instance();

        let count = openrouter.models_count().await.unwrap();
        assert!(count > 0);
    }

    #[tokio::test]
    async fn current_models() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();

        assert!(!models.is_empty());
    }

    #[tokio::test]
    async fn count_and_models_same_length() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();
        let count = openrouter.models_count().await.unwrap();

        assert!(count > 0);
        assert_eq!(count, models.len());
    }

    #[tokio::test]
    async fn empty_api_key() {
        let openrouter = OpenRouter::new("");

        let Error::OpenRouter(error) = openrouter.credits().await.unwrap_err() else {
            panic!("response is not a OpenRouter error")
        };

        assert_eq!(401, error.code());
    }

    #[tokio::test]
    async fn missing_or_wrong_items_in_models_spec() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();

        let mut unknown_input_modalities = HashSet::new();
        let mut unknown_output_modalities = HashSet::new();
        let mut unknown_tokenizers = HashSet::new();
        let mut unknown_instruct_types = HashSet::new();

        for model in models {
            for modality in model.architecture.input_modalities {
                if let Modality::Unknown(unknown) = modality {
                    unknown_input_modalities.insert(unknown);
                }
            }

            for modality in model.architecture.output_modalities {
                if let Modality::Unknown(unknown) = modality {
                    unknown_output_modalities.insert(unknown);
                }
            }

            if let Tokenizer::Unknown(unknown) = model.architecture.tokenizer {
                unknown_tokenizers.insert(unknown);
            }

            if let Some(InstructType::Unknown(unknown)) = model.architecture.instruct_type {
                unknown_instruct_types.insert(unknown);
            }
        }

        let unknowns = !(unknown_input_modalities.is_empty()
            && unknown_output_modalities.is_empty()
            && unknown_tokenizers.is_empty()
            && unknown_instruct_types.is_empty());

        if !unknown_input_modalities.is_empty() {
            println!("unknown input modalities: {unknown_input_modalities:#?}");
        }

        if !unknown_output_modalities.is_empty() {
            println!("unknown output modalities: {unknown_output_modalities:#?}");
        }

        if !unknown_tokenizers.is_empty() {
            println!("unknown tokenizers: {unknown_tokenizers:#?}");
        }

        if !unknown_instruct_types.is_empty() {
            println!("unknown instruct types: {unknown_instruct_types:#?}");
        }

        if unknowns {
            panic!("API returned unknown architecture types")
        }
    }

    #[tokio::test]
    #[ignore] // At the moment there's a model that is indeed free but not tagged as such.
    async fn free_models_are_labelled() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();

        let mut free_but_not_labelled = Vec::new();
        let mut not_free_but_labelled = Vec::new();

        for model in models {
            let price = model.pricing.completion
                + model.pricing.prompt
                + model.pricing.request.unwrap_or(0.0)
                + model.pricing.image.unwrap_or(0.0);

            match (model.id.ends_with(":free"), price == 0.0) {
                (true, true) | (false, false) => {
                    // Expected, labelled as expected!
                }
                (true, false) => {
                    not_free_but_labelled.push((model.id, price));
                }
                (false, true) => {
                    free_but_not_labelled.push((model.id, price));
                }
            }

            if !not_free_but_labelled.is_empty() {
                println!("not free but labelled as such: {not_free_but_labelled:#?}");
            }

            if !free_but_not_labelled.is_empty() {
                println!("free but not labelled as such: {free_but_not_labelled:#?}");
            }

            if !not_free_but_labelled.is_empty() || !free_but_not_labelled.is_empty() {
                panic!("found wrongly labelled models")
            }
        }
    }

    #[tokio::test]
    async fn key() {
        let openrouter = test_instance();

        let key = openrouter.key().await.unwrap();

        // Can't really test much here so let's go with something that should work most of the time.
        assert!(key.rate_limit.requests > 0);
    }

    #[tokio::test]
    async fn example_completion() {
        let openrouter = test_instance();

        let models = openrouter.free_models().await.unwrap();
        let model_ids = models.into_iter().map(|m| m.id).take(3).collect();

        let request = Request {
            prompt: Some("Apples are usually ".to_string()),
            models: Some(model_ids),
            max_tokens: Some(100),
            ..Request::default()
        };

        // This test does nothing but essentially just check that the response we get is valid.
        let _response = openrouter.completion(request).await.unwrap();
    }

    #[tokio::test]
    async fn example_chat_completion() {
        let openrouter = test_instance();

        let models = openrouter.free_models().await.unwrap();
        let model_ids = models.into_iter().map(|m| m.id).take(3).collect();

        let request = Request {
            messages: Some(vec![Message::System {
                content: Content::Plain(
                    "This is a test, please just give a very short answer on who you are."
                        .to_string(),
                ),
                name: None,
                cache_control: None,
            }]),
            models: Some(model_ids),
            max_tokens: Some(100),
            usage: Some(Usage { include: true }),
            ..Request::default()
        };

        // This test does nothing but essentially just check that the response we get is valid.
        let _response = openrouter.chat_completion(request).await.unwrap();
    }

    #[tokio::test]
    async fn ensure_no_unknown_parameters() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();

        let mut parameters = HashSet::new();

        for model in models {
            let endpoints = openrouter.model_endpoints(dbg!(model.id)).await.unwrap();

            for endpoint in endpoints.endpoints {
                for parameter in endpoint.supported_parameters {
                    if let Parameter::Unknown(name) = parameter {
                        parameters.insert(name);
                    }
                }
            }
        }

        assert!(
            parameters.is_empty(),
            "parameters should be empty: {parameters:#?}"
        );
    }

    #[tokio::test]
    async fn ensure_model_filter_works() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();
        let filtered_models = openrouter
            .models(&[Parameter::Tools, Parameter::StructuredOutputs])
            .await
            .unwrap();

        assert!(models.len() > filtered_models.len());
    }

    #[tokio::test]
    async fn ensure_providers_are_supported() {
        let openrouter = test_instance();

        let models = openrouter.models(&[]).await.unwrap();

        let mut providers = HashSet::new();

        for model in models {
            let endpoints = openrouter.model_endpoints(model.id).await.unwrap();

            for endpoint in endpoints.endpoints {
                if let Provider::Custom(c) = endpoint.provider_name {
                    providers.insert(c);
                }
            }
        }

        assert_eq!(HashSet::new(), providers);
    }

    #[tokio::test]
    #[ignore]
    // This test is very request and token intensive, which is why we ignore it by default.
    // Futhermore, it doesn't fail unless there's a truly unexpected result.
    // Running it with `--nocapture` will produce a report.
    async fn ensure_reasoning_endpoints_include_reasoning() {
        let openrouter = test_instance();

        let models = openrouter.models(&[Parameter::Reasoning]).await.unwrap();

        for (index, model) in models.iter().enumerate() {
            println!("Model: {}/{} - {}", index + 1, models.len(), model.id);

            let endpoints = openrouter
                .model_endpoints(model.id.clone())
                .await
                .unwrap()
                .endpoints;

            for (index, endpoint) in endpoints.iter().enumerate() {
                print!(
                    "Endpoint: {}/{} - {:?}:",
                    index + 1,
                    endpoints.len(),
                    endpoint.provider_name
                );

                if !endpoint
                    .supported_parameters
                    .contains(&Parameter::Reasoning)
                {
                    println!(" NO SUPPORT");
                    continue;
                }

                let provider = endpoint.provider_name.clone();

                let request = Request {
                    model: Some(model.id.clone()),
                    provider: Some(ProviderPreferences {
                        allow_fallbacks: Some(false),
                        require_parameters: Some(true),
                        data_collection: None,
                        order: vec![provider],
                        ignore: vec![],
                        quantizations: vec![],
                        sort: None,
                    }),
                    messages: Some(vec![Message::System {
                        content: Content::Plain("How many words are in this question?".to_owned()),
                        name: None,
                        cache_control: None,
                    }]),
                    ..Request::default()
                };

                // println!(
                //     "Request: {}",
                //     serde_json::to_string_pretty(&request).unwrap()
                // );

                let response = match openrouter.chat_completion(request).await {
                    Ok(r) => r,
                    Err(e) => match e {
                        Error::OpenRouter(o) => {
                            println!(" OpenRouter error: {} - {}", o.code, o.message);
                            continue;
                        }
                        Error::Reqwest(r) => {
                            println!(" network error: {r}");
                            continue;
                        }
                        Error::Serde(_) => {
                            unreachable!()
                        }
                        Error::InvalidHeader(_) => {
                            unreachable!()
                        }
                    },
                };

                let choice = &response.choices[0];

                match choice {
                    Choice::NonChat(_) | Choice::Streaming(_) => {
                        panic!("unexpected choice type: {:?}", choice);
                    }
                    Choice::NonStreaming(msg) => match msg.message.reasoning {
                        None => {
                            println!(" NO REASONING");
                        }
                        Some(_) => {
                            println!(" OK");
                        }
                    },
                }
            }
        }
    }
}
