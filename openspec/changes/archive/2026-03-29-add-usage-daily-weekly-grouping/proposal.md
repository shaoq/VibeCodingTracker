## Why

`vct usage --table` currently aggregates all usage data by model across the full scan result. This makes it hard to understand how token and cost usage change over time, especially when users want daily or weekly reporting without exporting the raw session data.

## What Changes

- Add `--days` and `--weekly` flags to the `usage` command as mutually exclusive time-grouping modes.
- Keep the current default `usage` behavior unchanged when neither flag is provided.
- Introduce time-bucketed usage aggregation so usage rows can be grouped by day or by ISO week before model-level summarization.
- Support time-grouped output for `usage --table` and `usage --json`.
- Render grouped `usage --table` output as a single flattened table with the period key as the first column, instead of one table per period.
- Continue outputting `Daily Averages (by Provider)` in grouped table mode, also as a single flattened table with the period key as the first column.
- In grouped usage tables, consecutive rows from the same period should visually merge the period column by showing the period only on the first row of that block.
- Replace the current grouped period-totals summary with a provider-based summary that includes a `Provider` column in both daily and weekly grouped modes.
- Define time buckets using session file modification time, matching the current directory scanning model.

## Capabilities

### New Capabilities
- `usage-time-grouping`: Add daily and weekly usage aggregation modes for the `usage` command, including table and JSON output.

### Modified Capabilities

## Impact

- Affected CLI parsing for `usage` flags in `src/cli.rs` and command dispatch in `src/main.rs`
- Affected usage aggregation flow in `src/usage/calculator.rs` and related usage models
- Affected static output rendering in `src/display/usage/table.rs` and JSON output generation in `src/main.rs`
- May require additional grouped provider-average metadata for period-specific daily-average rows
- Weekly grouped provider summaries may require per-period provider active-day tracking so daily averages remain correct within each week
- Requires new tests for CLI behavior, grouped aggregation, and output shape
