use crate::display::usage::averages::build_usage_summary;
use crate::models::{ProviderActiveDays, UsageResult};
use crate::pricing::{ModelPricingMap, fetch_model_pricing};
use std::collections::HashMap;

/// Displays token usage data as plain text (model: cost format)
pub fn display_usage_text(usage_data: &UsageResult, provider_days: &ProviderActiveDays) {
    if usage_data.is_empty() {
        println!("No usage data found");
        return;
    }

    // Fetch pricing data
    let pricing_map =
        fetch_model_pricing().unwrap_or_else(|_| ModelPricingMap::new(HashMap::new()));

    let summary = build_usage_summary(usage_data, provider_days, &pricing_map);

    if summary.rows.is_empty() {
        println!("No usage data found");
        return;
    }

    for row in &summary.rows {
        println!("{}: ${:.6}", row.display_model, row.cost);
    }
}
