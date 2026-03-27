use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Complete Gemini coding session with metadata and message history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSession {
    pub session_id: String,
    pub project_hash: String,
    pub start_time: String,
    pub last_updated: String,
    pub messages: Vec<GeminiMessage>,
}

/// Single message within a Gemini session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMessage {
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub content: String,
    #[serde(default)]
    pub thoughts: Vec<GeminiThought>,
    pub tokens: Option<GeminiTokens>,
    pub model: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<Value>,
}

/// Deserialize content that can be either a string or an array of {text: "..."} objects
fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Array(arr) => {
            // Extract text from [{text: "..."}, ...] format
            let texts: Vec<&str> = arr
                .iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect();
            Ok(texts.join("\n"))
        }
        Value::Null => Ok(String::new()),
        _ => Ok(value.to_string()),
    }
}

/// AI reasoning step captured during Gemini's thought process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiThought {
    pub subject: String,
    pub description: String,
    pub timestamp: String,
}

/// Token usage breakdown for a single Gemini message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiTokens {
    pub input: i64,
    pub output: i64,
    pub cached: i64,
    pub thoughts: i64,
    pub tool: i64,
    pub total: i64,
}
