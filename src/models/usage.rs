use std::collections::HashSet;

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

/// Grouping mode for time-bucketed usage reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupingMode {
    Daily,
    Weekly,
}

/// Active day counts per provider within a single time period
#[derive(Debug, Clone, Default)]
pub struct PeriodProviderDays {
    pub claude: HashSet<String>,
    pub codex: HashSet<String>,
    pub copilot: HashSet<String>,
    pub gemini: HashSet<String>,
}

/// Usage data for a single time period (day or week)
#[derive(Debug, Clone)]
pub struct PeriodUsage {
    /// Period key: "YYYY-MM-DD" for daily, "YYYY-Www" for weekly
    pub period_key: String,
    /// Model -> usage data within this period
    pub models: UsageResult,
    /// Per-provider active day counts within this period
    pub provider_days: PeriodProviderDays,
}

/// Grouped usage data containing all periods
#[derive(Debug, Clone)]
pub struct GroupedUsageData {
    pub periods: Vec<PeriodUsage>,
    pub grouping_mode: GroupingMode,
}
