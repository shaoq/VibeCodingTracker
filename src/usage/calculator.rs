use crate::cache::global_cache;
use crate::constants::{FastHashMap, capacity};
use crate::models::{
    GroupedUsageData, GroupingMode, PeriodProviderDays, PeriodUsage, ProviderActiveDays,
    Provider, UsageResult,
};
use crate::utils::{collect_files_with_dates, is_gemini_chat_file, is_json_file, resolve_paths};
use anyhow::Result;
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

/// Usage results with provider active day counts for daily averages
pub struct UsageData {
    pub models: UsageResult,
    pub provider_days: ProviderActiveDays,
}

/// Extracts token usage data from CodeAnalysis records
fn extract_conversation_usage_from_analysis(analysis: &Value) -> FastHashMap<String, Value> {
    let Some(records) = analysis.get("records").and_then(|r| r.as_array()) else {
        return FastHashMap::default();
    };

    // Pre-allocate HashMap using centralized capacity constant
    let mut conversation_usage = FastHashMap::with_capacity(capacity::MODELS_PER_SESSION);

    for record in records {
        let Some(record_obj) = record.as_object() else {
            continue;
        };

        let Some(conv_usage) = record_obj
            .get("conversationUsage")
            .and_then(|c| c.as_object())
        else {
            continue;
        };

        for (model, usage) in conv_usage {
            // Use entry API to avoid double lookup
            conversation_usage
                .entry(model.clone())
                .and_modify(|existing_usage| merge_usage_values(existing_usage, usage))
                .or_insert_with(|| usage.clone());
        }
    }

    conversation_usage
}

/// Aggregates token usage from all AI provider session directories
///
/// Scans Claude Code, Codex, Copilot, and Gemini session files, extracts token usage,
/// and aggregates by model. Returns usage data with provider active day counts.
pub fn get_usage_from_directories() -> Result<UsageData> {
    let paths = resolve_paths()?;
    let mut result = FastHashMap::with_capacity(capacity::MODEL_COMBINATIONS);

    let mut claude_dates: HashSet<String> = HashSet::new();
    let mut codex_dates: HashSet<String> = HashSet::new();
    let mut copilot_dates: HashSet<String> = HashSet::new();
    let mut gemini_dates: HashSet<String> = HashSet::new();

    if paths.claude_session_dir.exists() {
        process_usage_directory(
            &paths.claude_session_dir,
            &mut result,
            &mut claude_dates,
            is_json_file,
        )?;
    }

    if paths.codex_session_dir.exists() {
        process_usage_directory(
            &paths.codex_session_dir,
            &mut result,
            &mut codex_dates,
            is_json_file,
        )?;
    }

    if paths.copilot_session_dir.exists() {
        process_usage_directory(
            &paths.copilot_session_dir,
            &mut result,
            &mut copilot_dates,
            is_json_file,
        )?;
    }

    if paths.gemini_session_dir.exists() {
        process_usage_directory(
            &paths.gemini_session_dir,
            &mut result,
            &mut gemini_dates,
            is_gemini_chat_file,
        )?;
    }

    let mut all_dates: HashSet<&String> = HashSet::new();
    all_dates.extend(claude_dates.iter());
    all_dates.extend(codex_dates.iter());
    all_dates.extend(copilot_dates.iter());
    all_dates.extend(gemini_dates.iter());

    let provider_days = ProviderActiveDays {
        claude: claude_dates.len(),
        codex: codex_dates.len(),
        copilot: copilot_dates.len(),
        gemini: gemini_dates.len(),
        total: all_dates.len(),
    };

    Ok(UsageData {
        models: result,
        provider_days,
    })
}

fn process_usage_directory<P, F>(
    dir: P,
    result: &mut UsageResult,
    unique_dates: &mut HashSet<String>,
    filter_fn: F,
) -> Result<()>
where
    P: AsRef<Path>,
    F: Copy + Fn(&Path) -> bool + Sync + Send,
{
    let dir = dir.as_ref();
    let files = collect_files_with_dates(dir, filter_fn)?;

    // Process files in parallel with caching for better performance
    let file_results: Vec<(String, FastHashMap<String, Value>)> = files
        .par_iter()
        .filter_map(|file_info| {
            match global_cache().get_or_parse(&file_info.path) {
                Ok(analysis_arc) => {
                    // Use Arc to avoid deep cloning the entire analysis
                    let conversation_usage =
                        extract_conversation_usage_from_analysis(&analysis_arc);
                    Some((file_info.modified_date.clone(), conversation_usage))
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to analyze {}: {}",
                        file_info.path.display(),
                        e
                    );
                    None
                }
            }
        })
        .collect();

    // Merge parallel results sequentially (this part is fast)
    for (date, conversation_usage) in file_results {
        unique_dates.insert(date);

        for (model, usage_value) in conversation_usage {
            result
                .entry(model)
                .and_modify(|existing| merge_usage_values(existing, &usage_value))
                .or_insert(usage_value);
        }
    }

    Ok(())
}

fn merge_usage_values(existing: &mut Value, new: &Value) {
    use crate::utils::{accumulate_i64_fields, accumulate_nested_object};

    if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), new.as_object()) {
        // Handle Claude/Gemini format (has input_tokens)
        if existing_obj.contains_key("input_tokens") {
            accumulate_i64_fields(
                existing_obj,
                new_obj,
                &[
                    "input_tokens",
                    "cache_creation_input_tokens",
                    "cache_read_input_tokens",
                    "output_tokens",
                    "thoughts_tokens",
                    "tool_tokens",
                    "total_tokens",
                ],
            );

            if let Some(new_cache) = new_obj.get("cache_creation").and_then(|v| v.as_object()) {
                accumulate_nested_object(existing_obj, "cache_creation", new_cache);
            }
        }
        // Handle Codex format (has total_token_usage)
        else if existing_obj.contains_key("total_token_usage") {
            if let Some(new_total) = new_obj.get("total_token_usage").and_then(|v| v.as_object()) {
                accumulate_nested_object(existing_obj, "total_token_usage", new_total);
            }
        }
    }
}

/// Convert a YYYY-MM-DD date string to an ISO week key (YYYY-Www)
fn date_to_iso_week(date_str: &str) -> String {
    use chrono::Datelike;

    let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return date_str.to_string();
    };

    let iso_week = dt.iso_week();
    format!("{}-W{:02}", iso_week.year(), iso_week.week())
}

/// Convert a YYYY-MM-DD date string to the period key for the given grouping mode
fn to_period_key(date_str: &str, mode: GroupingMode) -> String {
    match mode {
        GroupingMode::Daily => date_str.to_string(),
        GroupingMode::Weekly => date_to_iso_week(date_str),
    }
}

/// Aggregates token usage from all AI provider session directories,
/// grouped by time period (day or ISO week).
pub fn get_grouped_usage_from_directories(mode: GroupingMode) -> Result<GroupedUsageData> {
    let paths = resolve_paths()?;

    // Collect (date, model_usage, provider) triples across all providers
    let mut all_file_results: Vec<(String, FastHashMap<String, Value>, Provider)> = Vec::new();

    if paths.claude_session_dir.exists() {
        let files = collect_files_with_dates(&paths.claude_session_dir, is_json_file)?;
        let results = process_grouped_files(&files)?;
        all_file_results.extend(results.into_iter().map(|(d, u)| (d, u, Provider::ClaudeCode)));
    }

    if paths.codex_session_dir.exists() {
        let files = collect_files_with_dates(&paths.codex_session_dir, is_json_file)?;
        let results = process_grouped_files(&files)?;
        all_file_results.extend(results.into_iter().map(|(d, u)| (d, u, Provider::Codex)));
    }

    if paths.copilot_session_dir.exists() {
        let files = collect_files_with_dates(&paths.copilot_session_dir, is_json_file)?;
        let results = process_grouped_files(&files)?;
        all_file_results.extend(results.into_iter().map(|(d, u)| (d, u, Provider::Copilot)));
    }

    if paths.gemini_session_dir.exists() {
        let files = collect_files_with_dates(&paths.gemini_session_dir, is_gemini_chat_file)?;
        let results = process_grouped_files(&files)?;
        all_file_results.extend(results.into_iter().map(|(d, u)| (d, u, Provider::Gemini)));
    }

    // Group by period key, tracking provider active days per period
    let mut period_models: FastHashMap<String, UsageResult> = FastHashMap::default();
    let mut period_provider_days: FastHashMap<String, PeriodProviderDays> = FastHashMap::default();

    for (date, conversation_usage, provider) in all_file_results {
        let period_key = to_period_key(&date, mode);

        let period_entry = period_models.entry(period_key.clone()).or_default();
        let provider_days = period_provider_days.entry(period_key).or_default();

        // Track which original dates this provider was active in this period
        match provider {
            Provider::ClaudeCode => {
                provider_days.claude.insert(date);
            }
            Provider::Codex => {
                provider_days.codex.insert(date);
            }
            Provider::Copilot => {
                provider_days.copilot.insert(date);
            }
            Provider::Gemini => {
                provider_days.gemini.insert(date);
            }
            Provider::Unknown => {}
        }

        for (model, usage_value) in conversation_usage {
            period_entry
                .entry(model)
                .and_modify(|existing| merge_usage_values(existing, &usage_value))
                .or_insert(usage_value);
        }
    }

    // Sort periods chronologically
    let mut periods: Vec<PeriodUsage> = period_models
        .into_iter()
        .map(|(period_key, models)| {
            let provider_days = period_provider_days.remove(&period_key).unwrap_or_default();
            PeriodUsage {
                period_key,
                models,
                provider_days,
            }
        })
        .collect();

    periods.sort_by(|a, b| a.period_key.cmp(&b.period_key));

    Ok(GroupedUsageData {
        periods,
        grouping_mode: mode,
    })
}

fn process_grouped_files(
    files: &[crate::utils::directory::FileInfo],
) -> Result<Vec<(String, FastHashMap<String, Value>)>> {
    let results: Vec<(String, FastHashMap<String, Value>)> = files
        .par_iter()
        .filter_map(|file_info| {
            match global_cache().get_or_parse(&file_info.path) {
                Ok(analysis_arc) => {
                    let conversation_usage =
                        extract_conversation_usage_from_analysis(&analysis_arc);
                    Some((file_info.modified_date.clone(), conversation_usage))
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to analyze {}: {}",
                        file_info.path.display(),
                        e
                    );
                    None
                }
            }
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_to_iso_week_regular() {
        // 2025-01-06 is a Monday in ISO week 2025-W02
        assert_eq!(date_to_iso_week("2025-01-06"), "2025-W02");
    }

    #[test]
    fn test_date_to_iso_week_year_boundary() {
        // 2024-12-30 is a Monday, belongs to ISO week 2025-W01
        assert_eq!(date_to_iso_week("2024-12-30"), "2025-W01");
    }

    #[test]
    fn test_date_to_iso_week_midweek() {
        // 2025-03-12 is a Wednesday in ISO week 2025-W11
        assert_eq!(date_to_iso_week("2025-03-12"), "2025-W11");
    }

    #[test]
    fn test_to_period_key_daily() {
        assert_eq!(to_period_key("2025-03-15", GroupingMode::Daily), "2025-03-15");
    }

    #[test]
    fn test_to_period_key_weekly() {
        let result = to_period_key("2025-03-12", GroupingMode::Weekly);
        assert!(result.starts_with("2025-W"), "Expected ISO week format, got: {}", result);
    }

    #[test]
    fn test_date_to_iso_week_invalid_input() {
        // Invalid date should return the input unchanged
        assert_eq!(date_to_iso_week("not-a-date"), "not-a-date");
    }
}
