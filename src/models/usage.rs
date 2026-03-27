use crate::constants::FastHashMap;

/// Token usage data aggregated by model (across all dates)
///
/// Structure: Model Name -> Usage Metrics
/// - Uses FastHashMap (ahash) for better performance than std HashMap
/// - Usage format varies by provider:
///   * Claude/Gemini: `{ input_tokens, output_tokens, cache_read_input_tokens, cache_creation_input_tokens }`
///   * Codex: `{ total_token_usage: { input_tokens, output_tokens } }`
pub type UsageResult = FastHashMap<String, serde_json::Value>;

/// Tracks the number of active days per AI provider
///
/// Used for calculating daily averages when data is aggregated by model only.
/// Day counts are derived from file modification dates during processing.
#[derive(Debug, Clone, Default)]
pub struct ProviderActiveDays {
    pub claude: usize,
    pub codex: usize,
    pub copilot: usize,
    pub gemini: usize,
    pub total: usize,
}
