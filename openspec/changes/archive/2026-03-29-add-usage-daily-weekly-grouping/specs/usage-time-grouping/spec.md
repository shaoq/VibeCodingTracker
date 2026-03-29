## ADDED Requirements

### Requirement: Usage command supports explicit time-grouping modes
The `usage` command SHALL support `--days` and `--weekly` flags as mutually exclusive grouping modes for non-interactive reporting.

#### Scenario: Default usage remains unchanged
- **WHEN** the user runs `vct usage`, `vct usage --table`, or `vct usage --json` without `--days` or `--weekly`
- **THEN** the command SHALL preserve the existing behavior of aggregating usage across the full scan result by model

#### Scenario: Reject conflicting grouping flags
- **WHEN** the user runs `vct usage` with both `--days` and `--weekly`
- **THEN** the command SHALL fail argument parsing and report that the grouping flags cannot be used together

### Requirement: Usage command can aggregate by day
The system SHALL bucket usage by session file modification date when `--days` is provided, and SHALL summarize usage by model within each day bucket.

#### Scenario: Daily table output groups rows by date
- **WHEN** the user runs `vct usage --table --days`
- **THEN** the output SHALL render a single usage table instead of one table per day
- **AND** the first column of that table SHALL be the day identifier
- **AND** token and cost values within each day SHALL be aggregated by model only for session files whose modification date falls on that day

#### Scenario: Daily usage table visually merges repeated periods
- **WHEN** the user runs `vct usage --table --days`
- **THEN** consecutive rows that belong to the same day SHALL keep model rows separate
- **AND** only the first row of that day block SHALL display the day identifier in the `Period` column
- **AND** following rows in the same day block SHALL leave the `Period` cell blank

#### Scenario: Daily JSON output is grouped by date
- **WHEN** the user runs `vct usage --json --days`
- **THEN** the JSON output SHALL be grouped by `YYYY-MM-DD` keys
- **AND** each day key SHALL contain model-level usage aggregates for that day only

#### Scenario: Daily provider averages remain visible
- **WHEN** the user runs `vct usage --table --days`
- **THEN** the output SHALL include a `Daily Averages (by Provider)` section after the main usage table
- **AND** that section SHALL render a single table with the day identifier as the first column
- **AND** that section SHALL include a `Provider` column

### Requirement: Usage command can aggregate by ISO week
The system SHALL bucket usage by ISO week when `--weekly` is provided, and SHALL summarize usage by model within each week bucket.

#### Scenario: Weekly table output groups rows by ISO week
- **WHEN** the user runs `vct usage --table --weekly`
- **THEN** the output SHALL render a single usage table instead of one table per week
- **AND** the first column of that table SHALL be the ISO week identifier
- **AND** token and cost values within each week SHALL be aggregated by model only for session files whose modification date falls within that ISO week

#### Scenario: Weekly usage table visually merges repeated periods
- **WHEN** the user runs `vct usage --table --weekly`
- **THEN** consecutive rows that belong to the same ISO week SHALL keep model rows separate
- **AND** only the first row of that ISO week block SHALL display the week identifier in the `Period` column
- **AND** following rows in the same ISO week block SHALL leave the `Period` cell blank

#### Scenario: Weekly JSON output is grouped by ISO week
- **WHEN** the user runs `vct usage --json --weekly`
- **THEN** the JSON output SHALL be grouped by ISO week keys
- **AND** each week key SHALL contain model-level usage aggregates for that week only

#### Scenario: Weekly provider averages remain visible
- **WHEN** the user runs `vct usage --table --weekly`
- **THEN** the output SHALL include a `Daily Averages (by Provider)` section after the main usage table
- **AND** that section SHALL render a single table with the ISO week identifier as the first column
- **AND** that section SHALL include a `Provider` column
- **AND** `Tokens/Day` and `Cost/Day` SHALL be calculated from the number of active days for that provider within the ISO week
