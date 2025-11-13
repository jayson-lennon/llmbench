use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use crate::{completions::response::ToolCall, providers::Provider};

// type Request = {
//   // Either "messages" or "prompt" is required
//   messages?: Message[];
//   prompt?: string;
//
//   // If "model" is unspecified, uses the user's default
//   model?: string; // See "Supported Models" section
//
//   // Allows to force the model to produce specific output format.
//   // See models page and note on this docs page for which models support it.
//   response_format?: { type: 'json_object' };
//
//   stop?: string | string[];
//   stream?: boolean; // Enable streaming
//
//   // See LLM Parameters (openrouter.ai/docs/parameters)
//   max_tokens?: number; // Range: [1, context_length)
//   temperature?: number; // Range: [0, 2]
//
//   // Tool calling
//   // Will be passed down as-is for providers implementing OpenAI's interface.
//   // For providers with custom interfaces, we transform and map the properties.
//   // Otherwise, we transform the tools into a YAML template. The model responds with an assistant message.
//   // See models supporting tool calling: openrouter.ai/models?supported_parameters=tools
//   tools?: Tool[];
//   tool_choice?: ToolChoice;
//
//   // Advanced optional parameters
//   seed?: number; // Integer only
//   top_p?: number; // Range: (0, 1]
//   top_k?: number; // Range: [1, Infinity) Not available for OpenAI models
//   frequency_penalty?: number; // Range: [-2, 2]
//   presence_penalty?: number; // Range: [-2, 2]
//   repetition_penalty?: number; // Range: (0, 2]
//   logit_bias?: { [key: number]: number };
//   top_logprobs: number; // Integer only
//   min_p?: number; // Range: [0, 1]
//   top_a?: number; // Range: [0, 1]
//
//   // Reduce latency by providing the model with a predicted output
//   // https://platform.openai.com/docs/guides/latency-optimization#use-predicted-outputs
//   prediction?: { type: 'content'; content: string };
//
//   // OpenRouter-only parameters
//   // See "Prompt Transforms" section: openrouter.ai/docs/transforms
//   transforms?: string[];
//   // See "Model Routing" section: openrouter.ai/docs/model-routing
//   models?: string[];
//   route?: 'fallback';
//   // See "Provider Routing" section: openrouter.ai/docs/provider-routing
//   provider?: ProviderPreferences;
//
//   // Whether to return the model's reasoning. Default false.
//   // Text will appear in the "reasoning" field on each message prior to those containing "content".
//   include_reasoning?: boolean;
// };
/// Represents a request to the model.
/// Note that both `messages` and `prompt` are optional at the type level;
/// additional validation would be needed to ensure at least one is provided.
#[derive(Serialize, Debug, Default, PartialEq)]
pub struct Request {
    /// Allows to define a chat history with various participants and mixed content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,

    /// Allows to define a raw prompt that the model will start completion at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// If "model" is unspecified, uses the user's default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Forces the model to produce a specific output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Allows defining one or more tokens for generation to stop at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Enable streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// LLM parameter: maximum tokens (Range: [1, context_length)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    /// LLM parameter: temperature (Range: [0, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Tool calling: tools will be passed down as-is (if supported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Advanced optional parameters:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Reduce latency by providing a predicted output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    /// OpenRouter-only parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Route>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,

    /// Whether to return the model's reasoning. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Hash, Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Stop {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub enum ResponseFormat {
    JsonObject,
}

impl Serialize for ResponseFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseFormat::JsonObject => {
                // Always serialize as { "type": "json_object" }
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "json_object")?;
                map.end()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Prediction {
    pub content: String,
}

impl Serialize for Prediction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize as { "type": "content", "content": <self.content> }
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", "content")?;
        map.serialize_entry("content", &self.content)?;
        map.end()
    }
}

/// Represents the only allowed value for `route`.
#[derive(Serialize, Debug, PartialEq)]
pub enum Route {
    #[serde(rename = "fallback")]
    Fallback,
}

/// OpenRouter routes requests to the best available providers for your model.
/// By default, requests are load balanced across the top providers to maximize uptime.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ProviderPreferences {
    /// Whether to allow backup providers to serve requests
    /// - true: (default) when the primary provider (or your custom providers in "order") is unavailable, use the next best provider.
    /// - false: use only the primary/custom provider, and return the upstream error if it's unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Whether to filter providers to only those that support the parameters you've provided.
    /// If this setting is omitted or set to false, then providers will receive only the parameters they support,
    /// and ignore the rest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Data collection setting. If no available model provider meets the requirement,
    /// your request will return an error.
    /// - allow: (default) allow providers which store user data non-transiently and may train on it
    /// - deny: use only providers which do not collect user data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<DataCollection>,
    /// An ordered list of provider names.
    /// The router will attempt to use the first provider in the subset of this list that supports your requested model,
    /// and fall back to the next if it is unavailable.
    /// If no providers are available, the request will fail with an error message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<Provider>,
    /// List of provider names to ignore.
    /// If provided, this list is merged with your account-wide ignored provider settings for this request.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<Provider>,
    /// A list of quantization levels to filter the provider by.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quantizations: Vec<Quantization>,
    /// The sorting strategy to use for this request, if "order" is not specified.
    /// When set, no load balancing is performed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sorting>,
}

#[derive(Serialize, Debug, Default, PartialEq)]
pub enum DataCollection {
    #[serde(rename = "allow")]
    #[default]
    Allow,
    #[serde(rename = "deny")]
    Deny,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum Quantization {
    #[serde(rename = "int8")]
    Int8,
    #[serde(rename = "fp6")]
    Fp6,
    #[serde(rename = "int4")]
    Int4,
    #[serde(rename = "fp8")]
    Fp8,
    #[serde(rename = "fp16")]
    Fp16,
    #[serde(rename = "bf16")]
    Bf16,
    #[serde(rename = "fp32")]
    Fp32,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum Sorting {
    #[serde(rename = "price")]
    Price,
    #[serde(rename = "throughput")]
    Throughput,
    #[serde(rename = "latency")]
    Latency,
}

// type Message =
//   | {
//       role: 'user' | 'assistant' | 'system';
//       // ContentParts are only for the "user" role:
//       content: string | ContentPart[];
//       // If "name" is included, it will be prepended like this
//       // for non-OpenAI models: `{name}: {content}`
//       name?: string;
//     }
//   | {
//       role: 'tool';
//       content: string;
//       tool_call_id: string;
//       name?: string;
//     };
/// Represents a Message which can be one of:
/// - A user/assistant/system message with content as either a plain string or an array of ContentPart.
/// - A tool message with additional tool_call_id.
#[derive(Hash, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    User {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Assistant {
        #[serde(skip_serializing_if = "skip_content_if_empty")]
        content: Option<Content>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    System {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Tool {
        content: String,
        tool_call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

fn skip_content_if_empty(content: &Option<Content>) -> bool {
    match content {
        None => true,
        Some(c) => match c {
            Content::Plain(c) => c.is_empty(),
            Content::Parts(p) => {
                if p.is_empty() {
                    return true;
                }

                p.iter().all(|c| match c {
                    ContentPart::Text { text } => text.is_empty(),
                    ContentPart::ImageUrl { image_url } => {
                        image_url.url.is_empty()
                            && image_url.detail.as_ref().is_none_or(|d| d.is_empty())
                    }
                    ContentPart::File {
                        filename,
                        file_data,
                    } => filename.is_empty() && file_data.is_empty(),
                })
            }
        },
    }
}

/// The content of a non-tool message, which can be a plain string or an array of content parts.
#[derive(Hash, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Content {
    Plain(String),
    Parts(Vec<ContentPart>),
}

// type ContentPart = TextContent | ImageContentPart;
/// Represents a part of content which can be either a text block or an image.
#[derive(Hash, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ContentPart {
    // type TextContent = {
    //   type: 'text';
    //   text: string;
    // };
    #[serde(rename = "text")]
    Text { text: String },
    // type ImageContentPart = {
    //   type: 'image_url';
    //   image_url: {
    //     url: string; // URL or base64 encoded image data
    //     detail?: string; // Optional, defaults to "auto"
    //   };
    // };
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    //{
    //    "type": "file",
    //    "file": {
    //        "filename": "document.pdf",
    //        "file_data": data_url
    //    }
    //}
    #[serde(rename = "file")]
    File { filename: String, file_data: String },
}

/// Represents an image content part.
#[derive(Hash, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImageUrl {
    /// URL or base64 encoded image data.
    pub url: String,
    /// Optional detail. Defaults to "auto". If equal to "auto", it will be omitted during serialization.
    #[serde(skip_serializing_if = "is_auto")]
    pub detail: Option<String>,
}

/// Returns true if the detail equals "auto".
fn is_auto(detail: &Option<String>) -> bool {
    detail.as_ref().map(|d| d == "auto").unwrap_or(true)
}

// type Tool = {
//   type: 'function';
//   function: FunctionDescription;
// };
#[derive(Hash, Deserialize, Clone, Debug, PartialEq)]
pub struct Tool {
    pub function: FunctionDescription,
}

impl Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", "function")?;
        map.serialize_entry("function", &self.function)?;
        map.end()
    }
}

// type FunctionDescription = {
//   description?: string;
//   name: string;
//   parameters: object; // JSON Schema object
// };
#[derive(Hash, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDescription {
    /// The name of the function.
    pub name: String,

    /// An optional description of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The parameters, represented as a JSON Schema object.
    /// It is recommended to use <https://crates.io/crates/schemars>.
    pub parameters: Value,
}

// type ToolChoice =
//   | 'none'
//   | 'auto'
//   | {
//       type: 'function';
//       function: {
//         name: string;
//       };
//     };
#[derive(Hash, Clone, Debug, Deserialize, PartialEq)]
pub enum ToolChoice {
    None,
    Auto,
    Required,
    Function(String),
}

impl Serialize for ToolChoice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ToolChoice::None => serializer.serialize_str("none"),
            ToolChoice::Auto => serializer.serialize_str("auto"),
            ToolChoice::Required => serializer.serialize_str("required"),
            ToolChoice::Function(name) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "function")?;

                // Inline struct to hold function data
                #[derive(Serialize)]
                struct FunctionField<'a> {
                    name: &'a String,
                }

                map.serialize_entry("function", &FunctionField { name })?;
                map.end()
            }
        }
    }
}

/// As described in the OpenRouter documentation, only [`Reasoning::effort`] or [`Reasoning::max_tokens`]
/// should be set.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub enum Effort {
    #[serde(rename = "high")]
    High,
    #[default]
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct Usage {
    pub include: bool,
}

#[derive(Hash, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    r#type: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::completions::request::{
        Content, ContentPart, FunctionDescription, ImageUrl, Message, Prediction, Request,
        ResponseFormat, Route, Stop, Tool, ToolChoice,
    };

    #[test]
    fn tool_choice() {
        let tool_choice = ToolChoice::None;
        let expected = json!("none");
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);

        let tool_choice = ToolChoice::Auto;
        let expected = json!("auto");
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);

        let tool_choice = ToolChoice::Function("test".to_string());
        let expected = json!({
            "type": "function",
            "function": { "name": "test" }
        });
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn function_description() {
        let fd = FunctionDescription {
            name: "sample_func".to_string(),
            description: Some("A sample function".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "arg1": { "type": "string" },
                    "arg2": { "type": "number" }
                },
                "required": ["arg1"]
            }),
        };

        let serialized = serde_json::to_string(&fd).unwrap();
        let expected = r#"{"name":"sample_func","description":"A sample function","parameters":{"type":"object","properties":{"arg1":{"type":"string"},"arg2":{"type":"number"}},"required":["arg1"]}}"#;
        assert_eq!(expected, serialized);
    }

    #[test]
    fn tool() {
        let function_description = FunctionDescription {
            description: Some("This is a test function".to_string()),
            name: "test_function".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "param1": { "type": "string" },
                    "param2": { "type": "integer" }
                },
                "required": ["param1"]
            }),
        };

        let tool = Tool {
            function: function_description,
        };

        let serialized = serde_json::to_string(&tool).unwrap();

        let expected: Value = json!({
            "type": "function",
            "function": {
                "description": "This is a test function",
                "name": "test_function",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "param1": { "type": "string" },
                        "param2": { "type": "integer" }
                    },
                    "required": ["param1"]
                }
            }
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, serialized_value);
    }

    #[test]
    fn text_content() {
        let content = ContentPart::Text {
            text: "Hello, world!".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "text",
            "text": "Hello, world!"
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, serialized_value);
    }

    #[test]
    fn image_content() {
        let content = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("custom detail".to_string()),
            },
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "image_url",
            "image_url": {
                "url": "https://example.com/image.png",
                "detail": "custom detail"
            }
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(serialized_value, expected);
    }

    #[test]
    fn image_url_default() {
        let content = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("auto".into()),
            },
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "image_url",
            "image_url": {
                "url": "https://example.com/image.png"
                // "detail" is omitted because it's the default value ("auto")
            }
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn user_plain() {
        let msg = Message::User {
            content: Content::Plain("Hello, world!".to_string()),
            name: Some("Alice".to_string()),
            cache_control: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "user",
            "content": "Hello, world!",
            "name": "Alice"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn user_parts() {
        let msg = Message::User {
            content: Content::Parts(vec![
                ContentPart::Text {
                    text: "Hello".to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "https://example.com/image.png".to_string(),
                        detail: Some("custom".to_string()),
                    },
                },
            ]),
            name: None,
            cache_control: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "user",
            "content": [
                { "type": "text", "text": "Hello" },
                { "type": "image_url", "image_url": { "url": "https://example.com/image.png", "detail": "custom" } }
            ]
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_assistant() {
        let msg = Message::Assistant {
            content: Some(Content::Plain("How can I help?".to_string())),
            name: None,
            tool_calls: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "assistant",
            "content": "How can I help?"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_system() {
        let msg = Message::System {
            content: Content::Plain("System message".to_string()),
            name: None,
            cache_control: Some(CacheControl {
                r#type: "ephemeral".to_string(),
            }),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "system",
            "content": "System message",
            "cache_control": {
                "type": "ephemeral",
            },
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_tool() {
        let msg = Message::Tool {
            content: "Tool response".to_string(),
            tool_call_id: "call123".to_string(),
            name: Some("ToolName".to_string()),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "tool",
            "content": "Tool response",
            "tool_call_id": "call123",
            "name": "ToolName"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn request() {
        // Prepare a sample logit_bias value.
        let mut logit_bias = HashMap::new();
        logit_bias.insert("1".to_string(), 0.5);

        // Build a Request instance with a mix of fields.
        let req = Request {
            messages: None,
            prompt: Some("Hello".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            response_format: Some(ResponseFormat::JsonObject),
            stop: Some(Stop::Multiple(vec![
                "STOP1".to_string(),
                "STOP2".to_string(),
            ])),
            stream: Some(true),
            max_tokens: Some(100),
            temperature: Some(0.5),
            tools: None,
            tool_choice: None,
            seed: Some(42),
            top_p: Some(0.95),
            top_k: Some(10),
            frequency_penalty: Some(0.2),
            presence_penalty: Some(0.3),
            repetition_penalty: Some(0.8),
            logit_bias: Some(logit_bias),
            top_logprobs: Some(5),
            min_p: Some(0.1),
            top_a: Some(0.2),
            user: Some("test".into()),
            prediction: Some(Prediction {
                content: "Predicted content".to_string(),
            }),
            transforms: Some(vec!["transform1".to_string(), "transform2".to_string()]),
            models: Some(vec!["model1".to_string(), "model2".to_string()]),
            route: Some(Route::Fallback),
            provider: None,
            reasoning: Some(Reasoning {
                effort: Some(Effort::High),
                max_tokens: None,
                exclude: Some(true),
            }),
            usage: None,
        };

        let serialized = serde_json::to_string(&req).unwrap();

        let expected: Value = json!({
            "prompt": "Hello",
            "model": "gpt-3.5-turbo",
            "response_format": { "type": "json_object" },
            "stop": ["STOP1", "STOP2"],
            "stream": true,
            "max_tokens": 100,
            "temperature": 0.5,
            "seed": 42,
            "top_p": 0.95,
            "top_k": 10,
            "frequency_penalty": 0.2,
            "presence_penalty": 0.3,
            "repetition_penalty": 0.8,
            "logit_bias": { "1": 0.5 },
            "top_logprobs": 5,
            "min_p": 0.1,
            "top_a": 0.2,
            "user": "test",
            "prediction": { "type": "content", "content": "Predicted content" },
            "transforms": ["transform1", "transform2"],
            "models": ["model1", "model2"],
            "route": "fallback",
            "reasoning": { "effort": "high", "exclude": true }
        });

        let actual: Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expected, actual);
    }
}
