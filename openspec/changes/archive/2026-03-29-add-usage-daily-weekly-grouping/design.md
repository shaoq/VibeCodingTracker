## Context

The current `usage` command aggregates all scanned session usage into a single `model -> usage` map. Time information is only retained as provider-level active-day counts used for averages, so `vct usage --table` cannot answer daily or weekly reporting needs.

This change adds time-grouped reporting without breaking the existing default behavior. The user has confirmed that time buckets may be derived from session file modification time, which aligns with the current directory scanning pipeline and avoids provider-specific message timestamp normalization.

## Goals / Non-Goals

**Goals:**
- Add `--days` and `--weekly` as mutually exclusive grouping modes for `usage`
- Preserve current behavior when neither grouping flag is provided
- Support grouped output for `usage --table` and `usage --json`
- Define a single internal aggregation model that can bucket usage by day or ISO week before model-level summarization
- Render grouped table output as flattened period-first tables that remain easy to compare across many periods

**Non-Goals:**
- Changing the interactive `vct usage` TUI workflow
- Re-bucketing usage by per-message timestamps inside provider logs
- Changing `analysis` command behavior
- Redesigning the existing provider daily-average summary beyond what is needed for grouped output

## Decisions

### 1. Add grouping flags at the CLI level
The `usage` command will accept `--days` and `--weekly` as mutually exclusive flags.

Rationale:
- Matches the requested UX exactly
- Keeps default output backward compatible
- Avoids introducing a more abstract `--group-by` option in the first iteration

Alternatives considered:
- Add `--group-by day|week`
  Rejected for now because it changes the existing CLI style more than necessary

### 2. Introduce a time-bucketed aggregation layer in `usage`
The aggregation pipeline will produce grouped usage results keyed by a period identifier, then aggregate by model within each period.

Rationale:
- The current display layer only receives already-aggregated data, so grouped reporting cannot be implemented purely in rendering
- A structured grouped result can support both table and JSON output from the same source

Alternatives considered:
- Compute groups only inside `display_usage_table`
  Rejected because the raw `UsageResult` no longer contains enough time information once aggregation is complete

### 3. Use session file modification time as the period source
Daily grouping will use `YYYY-MM-DD`. Weekly grouping will use ISO week keys such as `YYYY-Www`.

Rationale:
- Reuses the existing scan metadata already collected in `collect_files_with_dates`
- Avoids provider-specific timestamp parsing differences and multi-record reconciliation
- Matches the user's accepted constraint

Alternatives considered:
- Use message/event timestamps from each provider log
  Rejected because it significantly increases parsing complexity and changes current semantics

### 4. Scope grouped reporting to `--table` and `--json`
The first implementation will define grouped behavior for static table and JSON outputs. The default TUI path remains unchanged.

Rationale:
- This is the user-requested path
- Keeps the change bounded and easier to verify
- Avoids mixing auto-refresh TUI behavior with grouped historical reports

Alternatives considered:
- Extend grouped modes to TUI immediately
  Rejected due to larger UI design and refresh-state complexity

### 5. Flatten grouped table sections into single tables
Grouped table mode will not print one standalone usage table per period. Instead, it will render:

- one usage table with `Period` as the first column
- one `Daily Averages (by Provider)` table with `Period` as the first column

Rationale:
- Repeated headers and borders make large grouped reports hard to scan
- A flattened layout is easier to compare across adjacent dates or weeks
- This matches the requested reporting format directly

Alternatives considered:
- Keep one table per period with a period heading
  Rejected because it is verbose and weak for cross-period comparison

### 6. Preserve provider averages in grouped table mode
Grouped table mode will continue to display `Daily Averages (by Provider)` after the main usage table, but the rows will be grouped into a single flattened table keyed by period.

Rationale:
- Maintains parity with existing non-grouped table mode
- Keeps provider-level summary information visible in grouped reports

Alternatives considered:
- Omit provider averages in grouped mode
  Rejected because it removes useful summary information users already expect from `usage --table`

### 7. Visually merge repeated period cells
When consecutive table rows belong to the same period, the renderer will show the period value only on the first row of that period block and leave later `Period` cells blank.

Rationale:
- Improves readability while keeping model rows and totals separate
- Achieves a merged look without requiring true rowspan support from the terminal table library

Alternatives considered:
- Repeat the period on every row
  Rejected because it adds unnecessary repetition
- Implement true merged cells
  Rejected because the current terminal table stack does not support it naturally

### 8. Restore grouped summary to provider dimension
The grouped summary section will not remain a simple period totals table. It will instead render a provider-based summary table with:

- `Period`
- `Provider`
- `Tokens/Day`
- `Cost/Day`
- `Active Days`

Rationale:
- Restores parity with the original non-grouped `usage --table` summary
- Fixes the current grouped weekly summary, which omits the provider dimension
- Better matches user expectations when comparing providers across periods

Alternatives considered:
- Keep the current period totals table
  Rejected because it changes summary semantics and removes provider visibility

## Risks / Trade-offs

- [File modification time may not equal conversation time] -> Document this behavior in CLI help and README, and keep the rule explicit in the spec
- [Grouped JSON shape differs from existing flat model map] -> Limit the new shape to grouped modes only and preserve current JSON output when no grouping flag is used
- [More aggregation structures increase implementation complexity] -> Keep the default flat path intact and add grouped data structures alongside it
- [Weekly labels may be misunderstood] -> Use ISO week formatting consistently in output and tests
- [Grouped provider averages need period-specific active-day counts] -> Track or derive provider activity within each period so daily averages remain meaningful for weekly output
- [Zero-usage periods can create noisy flattened rows] -> Decide during implementation whether to suppress empty periods or keep explicit zero rows consistently
- [Weekly provider averages cannot be derived from grouped model totals alone] -> Extend grouped aggregation metadata so each period retains provider-level active-day information

## Migration Plan

No data migration is required.

Implementation rollout:
1. Add CLI flags and validation
2. Add grouped aggregation models and calculators
3. Add grouped table/JSON rendering
4. Update tests and documentation

Rollback strategy:
- Remove the new flags and grouped aggregation path
- Existing default `usage` behavior remains unchanged

## Open Questions

- Whether grouped `--text` output should be added now or deferred
- Whether grouped table output should include period subtotal rows in addition to model rows
