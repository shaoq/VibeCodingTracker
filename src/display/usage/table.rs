use crate::display::common::table::{
    add_totals_row, create_comfy_table, create_metric_cell, create_provider_cell,
};
use crate::display::usage::averages::{
    build_provider_average_rows, build_usage_summary, format_tokens_per_day,
};
use crate::models::{GroupedUsageData, GroupingMode, Provider, ProviderActiveDays, UsageResult};
use crate::pricing::{ModelPricingMap, calculate_cost, fetch_model_pricing};
use crate::utils::{extract_token_counts, format_number};
use comfy_table::{Cell, CellAlignment, Color, Table, presets::UTF8_FULL};
use owo_colors::OwoColorize;
use std::collections::HashMap;

/// Displays token usage data as a static table
pub fn display_usage_table(usage_data: &UsageResult, provider_days: &ProviderActiveDays) {
    if usage_data.is_empty() {
        println!("⚠️  No usage data found in Claude Code or Codex sessions");
        return;
    }

    println!("{}", "📊 Token Usage Statistics".bright_cyan().bold());
    println!();

    // Fetch pricing data
    let pricing_map = match fetch_model_pricing() {
        Ok(map) => map,
        Err(e) => {
            eprintln!("⚠️  Warning: Failed to fetch pricing data: {}", e);
            eprintln!("   Costs will be shown as $0.00");
            ModelPricingMap::new(HashMap::new())
        }
    };

    let summary = build_usage_summary(usage_data, provider_days, &pricing_map);

    if summary.rows.is_empty() {
        println!("⚠️  No usage data found in Claude Code or Codex sessions");
        return;
    }

    let rows = &summary.rows;
    let totals = &summary.totals;

    // Create table
    let mut table = create_comfy_table(
        vec![
            "Model",
            "Input",
            "Output",
            "Cache Read",
            "Cache Creation",
            "Total Tokens",
            "Cost (USD)",
        ],
        Color::Yellow,
    );

    // Add data rows
    for row in rows {
        table.add_row(vec![
            Cell::new(&row.display_model)
                .fg(Color::Green)
                .set_alignment(CellAlignment::Left),
            Cell::new(format_number(row.input_tokens))
                .fg(Color::White)
                .set_alignment(CellAlignment::Right),
            Cell::new(format_number(row.output_tokens))
                .fg(Color::White)
                .set_alignment(CellAlignment::Right),
            Cell::new(format_number(row.cache_read))
                .fg(Color::White)
                .set_alignment(CellAlignment::Right),
            Cell::new(format_number(row.cache_creation))
                .fg(Color::White)
                .set_alignment(CellAlignment::Right),
            Cell::new(format_number(row.total))
                .fg(Color::Magenta)
                .set_alignment(CellAlignment::Right),
            Cell::new(format!("${:.2}", row.cost))
                .fg(Color::Cyan)
                .set_alignment(CellAlignment::Right),
        ]);
    }

    // Add totals row
    add_totals_row(
        &mut table,
        vec![
            "TOTAL".to_string(),
            format_number(totals.input_tokens),
            format_number(totals.output_tokens),
            format_number(totals.cache_read),
            format_number(totals.cache_creation),
            format_number(totals.total),
            format!("${:.2}", totals.cost),
        ],
        Color::Red,
    );

    println!("{table}");
    println!();

    // Calculate and display daily averages
    let provider_rows = build_provider_average_rows(&summary.daily_averages);

    println!(
        "{}",
        "📈 Daily Averages (by Provider)".bright_magenta().bold()
    );
    println!();

    let mut avg_table = Table::new();
    avg_table.load_preset(UTF8_FULL).set_header(vec![
        Cell::new("Provider")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Left),
        Cell::new("Tokens/Day")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
        Cell::new("Cost/Day")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
        Cell::new("Active Days")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
    ]);

    for row in &provider_rows {
        let name = format!("{} {}", row.icon, row.label);
        let name_cell = create_provider_cell(name, row.table_color, row.emphasize);
        let tokens_cell = create_metric_cell(
            format_tokens_per_day(row.stats.avg_tokens()),
            row.table_color,
            row.emphasize,
        );
        let cost_cell = create_metric_cell(
            format!("${:.2}", row.stats.avg_cost()),
            row.table_color,
            row.emphasize,
        );
        let days_cell = create_metric_cell(
            format_number(row.stats.days_count as i64),
            row.table_color,
            row.emphasize,
        );

        avg_table.add_row(vec![name_cell, tokens_cell, cost_cell, days_cell]);
    }

    println!("{avg_table}");
    println!();
}

/// Displays grouped token usage data as a single flattened table with Period as first column
pub fn display_grouped_usage_table(grouped: &GroupedUsageData) {
    if grouped.periods.is_empty() {
        println!("No usage data found");
        return;
    }

    let mode_label = match grouped.grouping_mode {
        GroupingMode::Daily => "Daily",
        GroupingMode::Weekly => "Weekly",
    };

    println!(
        "{}",
        format!("Token Usage Statistics (Grouped by {})", mode_label)
            .bright_cyan()
            .bold()
    );
    println!();

    let pricing_map = match fetch_model_pricing() {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Warning: Failed to fetch pricing data: {}. Costs will be shown as $0.00", e);
            ModelPricingMap::new(HashMap::new())
        }
    };

    // Single flattened table with Period as first column
    let mut table = create_comfy_table(
        vec![
            "Period",
            "Model",
            "Input",
            "Output",
            "Cache Read",
            "Cache Creation",
            "Total Tokens",
            "Cost (USD)",
        ],
        Color::Yellow,
    );

    let mut grand_total_input: i64 = 0;
    let mut grand_total_output: i64 = 0;
    let mut grand_total_cache_read: i64 = 0;
    let mut grand_total_cache_creation: i64 = 0;
    let mut grand_total_tokens: i64 = 0;
    let mut grand_total_cost: f64 = 0.0;

    let mut current_period: Option<&str> = None;

    for period in &grouped.periods {
        let mut models: Vec<_> = period.models.iter().collect();
        models.sort_by_key(|(model, _)| *model);

        let is_new_period = current_period != Some(period.period_key.as_str());
        current_period = Some(&period.period_key);

        for (idx, (model, usage)) in models.iter().enumerate() {
            let counts = extract_token_counts(usage);
            let pricing_result = pricing_map.get(model);

            let cost = calculate_cost(
                counts.input_tokens,
                counts.output_tokens,
                counts.cache_read,
                counts.cache_creation,
                &pricing_result.pricing,
            );

            grand_total_input += counts.input_tokens;
            grand_total_output += counts.output_tokens;
            grand_total_cache_read += counts.cache_read;
            grand_total_cache_creation += counts.cache_creation;
            grand_total_tokens += counts.total;
            grand_total_cost += cost;

            // Show period only on the first row of each period block
            let period_cell = if is_new_period && idx == 0 {
                Cell::new(&period.period_key)
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Left)
            } else {
                Cell::new("").set_alignment(CellAlignment::Left)
            };

            table.add_row(vec![
                period_cell,
                Cell::new(model)
                    .fg(Color::Green)
                    .set_alignment(CellAlignment::Left),
                Cell::new(format_number(counts.input_tokens))
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_number(counts.output_tokens))
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_number(counts.cache_read))
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_number(counts.cache_creation))
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format_number(counts.total))
                    .fg(Color::Magenta)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format!("${:.2}", cost))
                    .fg(Color::Cyan)
                    .set_alignment(CellAlignment::Right),
            ]);
        }
    }

    add_totals_row(
        &mut table,
        vec![
            "".to_string(),
            "TOTAL".to_string(),
            format_number(grand_total_input),
            format_number(grand_total_output),
            format_number(grand_total_cache_read),
            format_number(grand_total_cache_creation),
            format_number(grand_total_tokens),
            format!("${:.2}", grand_total_cost),
        ],
        Color::Red,
    );

    println!("{table}");
    println!();

    // Provider-based summary table (period x provider with daily averages)
    println!(
        "{}",
        format!("Daily Averages (by Provider)",).bright_magenta().bold()
    );
    println!();

    let mut avg_table = Table::new();
    avg_table.load_preset(UTF8_FULL).set_header(vec![
        Cell::new("Period")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Left),
        Cell::new("Provider")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Left),
        Cell::new("Tokens/Day")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
        Cell::new("Cost/Day")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
        Cell::new("Active Days")
            .fg(Color::Magenta)
            .set_alignment(CellAlignment::Right),
    ]);

    let providers = [
        (Provider::ClaudeCode, "🤖 Claude Code"),
        (Provider::Codex, "🧠 OpenAI Codex"),
        (Provider::Copilot, "🐙 GitHub Copilot"),
        (Provider::Gemini, "✨ Gemini"),
    ];

    let mut summary_period_shown: Option<&str> = None;

    for period in &grouped.periods {
        let is_new_period = summary_period_shown != Some(period.period_key.as_str());
        summary_period_shown = Some(&period.period_key);

        for (provider_idx, (provider, provider_label)) in providers.iter().enumerate() {
            // Collect tokens and cost for this provider in this period
            let mut provider_tokens: i64 = 0;
            let mut provider_cost: f64 = 0.0;

            for (model, usage) in &period.models {
                if Provider::from_model_name(model) != *provider {
                    continue;
                }
                let counts = extract_token_counts(usage);
                let pricing_result = pricing_map.get(model);
                let cost = calculate_cost(
                    counts.input_tokens,
                    counts.output_tokens,
                    counts.cache_read,
                    counts.cache_creation,
                    &pricing_result.pricing,
                );
                provider_tokens += counts.total;
                provider_cost += cost;
            }

            // Determine active days for this provider in this period
            let active_days = match provider {
                Provider::ClaudeCode => period.provider_days.claude.len(),
                Provider::Codex => period.provider_days.codex.len(),
                Provider::Copilot => period.provider_days.copilot.len(),
                Provider::Gemini => period.provider_days.gemini.len(),
                Provider::Unknown => 0,
            };

            if active_days == 0 && provider_tokens == 0 {
                continue;
            }

            let tokens_per_day = if active_days > 0 {
                provider_tokens as f64 / active_days as f64
            } else {
                0.0
            };
            let cost_per_day = if active_days > 0 {
                provider_cost / active_days as f64
            } else {
                0.0
            };

            // Show period only on the first provider row of each period block
            let period_cell = if is_new_period && provider_idx == 0 {
                Cell::new(&period.period_key)
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Left)
            } else {
                Cell::new("").set_alignment(CellAlignment::Left)
            };

            avg_table.add_row(vec![
                period_cell,
                Cell::new(*provider_label)
                    .fg(Color::Green)
                    .set_alignment(CellAlignment::Left),
                Cell::new(format_tokens_per_day(tokens_per_day))
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
                Cell::new(format!("${:.2}", cost_per_day))
                    .fg(Color::Cyan)
                    .set_alignment(CellAlignment::Right),
                Cell::new(active_days.to_string())
                    .fg(Color::White)
                    .set_alignment(CellAlignment::Right),
            ]);
        }
    }

    println!("{avg_table}");
    println!();
}
