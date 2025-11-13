use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    #[serde(rename = "AI21")]
    AI21,
    #[serde(rename = "AionLabs")]
    AionLabs,
    #[serde(rename = "Alibaba")]
    Alibaba,
    #[serde(rename = "Amazon Bedrock")]
    AmazonBedrock,
    #[serde(rename = "Anthropic")]
    Anthropic,
    #[serde(rename = "AnyScale")]
    AnyScale,
    #[serde(rename = "Atoma")]
    Atoma,
    #[serde(rename = "Avian")]
    Avian,
    #[serde(rename = "Azure")]
    Azure,
    #[serde(rename = "BaseTen")]
    BaseTen,
    #[serde(rename = "Cent-ML")]
    CentML,
    #[serde(rename = "Cerebras")]
    Cerebras,
    #[serde(rename = "Chutes")]
    Chutes,
    #[serde(rename = "Cloudflare")]
    Cloudflare,
    #[serde(rename = "Cohere")]
    Cohere,
    #[serde(rename = "Crusoe")]
    Crusoe,
    #[serde(rename = "DeepInfra")]
    DeepInfra,
    #[serde(rename = "DeepSeek")]
    DeepSeek,
    #[serde(rename = "Enfer")]
    Enfer,
    #[serde(rename = "Featherless")]
    Featherless,
    #[serde(rename = "Fireworks")]
    Fireworks,
    #[serde(rename = "Friendli")]
    Friendli,
    #[serde(rename = "GMICloud")]
    GMICloud,
    #[serde(rename = "Google")]
    Google,
    #[serde(rename = "Google AI Studio")]
    GoogleAIStudio,
    #[serde(rename = "Groq")]
    Groq,
    #[serde(rename = "HuggingFace")]
    HuggingFace,
    #[serde(rename = "Hyperbolic")]
    Hyperbolic,
    #[serde(rename = "Hyperbolic 2")]
    Hyperbolic2,
    #[serde(rename = "Inception")]
    Inception,
    #[serde(rename = "InferenceNet")]
    InferenceNet,
    #[serde(rename = "Infermatic")]
    Infermatic,
    #[serde(rename = "Inflection")]
    Inflection,
    #[serde(rename = "InoCloud")]
    InoCloud,
    #[serde(rename = "Kluster")]
    Kluster,
    #[serde(rename = "Lambda")]
    Lambda,
    #[serde(rename = "Lepton")]
    Lepton,
    #[serde(rename = "Liquid")]
    Liquid,
    #[serde(rename = "Lynn")]
    Lynn,
    #[serde(rename = "Lynn 2")]
    Lynn2,
    #[serde(rename = "Mancer")]
    Mancer,
    #[serde(rename = "Mancer 2")]
    Mancer2,
    #[serde(rename = "Meta")]
    Meta,
    #[serde(rename = "Minimax")]
    Minimax,
    #[serde(rename = "Mistral")]
    Mistral,
    #[serde(rename = "Modal")]
    Modal,
    #[serde(rename = "NCompass")]
    NCompass,
    #[serde(rename = "Nebius")]
    Nebius,
    #[serde(rename = "NextBit")]
    NextBit,
    #[serde(rename = "Nineteen")]
    Nineteen,
    #[serde(rename = "Novita")]
    Novita,
    #[serde(rename = "OctoAI")]
    OctoAI,
    #[serde(rename = "OpenAI")]
    OpenAI,
    #[serde(rename = "OpenInference")]
    OpenInference,
    #[serde(rename = "Parasail")]
    Parasail,
    #[serde(rename = "Perplexity")]
    Perplexity,
    #[serde(rename = "Phala")]
    Phala,
    #[serde(rename = "Recursal")]
    Recursal,
    #[serde(rename = "Reflection")]
    Reflection,
    #[serde(rename = "Replicate")]
    Replicate,
    #[serde(rename = "SambaNova")]
    SambaNova,
    #[serde(rename = "SambaNova2")]
    SambaNova2,
    #[serde(rename = "SF Compute")]
    SFCompute,
    #[serde(rename = "Stealth")]
    Stealth,
    #[serde(rename = "Targon")]
    Targon,
    #[serde(rename = "Together")]
    Together,
    #[serde(rename = "Together 2")]
    Together2,
    #[serde(rename = "Ubicloud")]
    Ubicloud,
    #[serde(rename = "xAI")]
    XAI,
    #[serde(rename = "01.AI")]
    ZeroOneAI,
    #[serde(untagged)]
    Custom(String),
}
