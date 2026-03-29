## 1. CLI And Aggregation Model

- [x] 1.1 Add mutually exclusive `--days` and `--weekly` flags to the `usage` command and wire the selected grouping mode through command dispatch
- [x] 1.2 Introduce usage grouping data structures and period-key generation for daily and ISO-week buckets based on session file modification time
- [x] 1.3 Implement grouped usage aggregation alongside the existing flat model aggregation path without changing default behavior

## 2. Output Rendering

- [x] 2.1 Add grouped static table rendering for `vct usage --table --days` and `vct usage --table --weekly`
- [x] 2.2 Add grouped JSON output for `vct usage --json --days` and `vct usage --json --weekly`
- [x] 2.3 Keep existing non-grouped `usage` output behavior unchanged when no grouping flag is provided

## 3. Verification And Documentation

- [x] 3.1 Add or update integration tests for CLI flag conflicts, daily grouping, and weekly grouping output shapes
- [x] 3.2 Add unit tests for period-key generation and grouped aggregation behavior
- [x] 3.3 Update README and CLI help text to document grouped usage reporting and the use of session file modification time as the grouping source

## 4. Grouped Table Layout Refinement

- [x] 4.1 Change grouped `usage --table` rendering to a single flattened usage table with `Period` as the first column
- [x] 4.2 Add grouped `Daily Averages (by Provider)` rendering as a single flattened table with `Period` as the first column
- [x] 4.3 Add or update tests to verify grouped table mode no longer emits one standalone table per period
- [x] 4.4 Visually merge repeated `Period` cells in grouped usage tables by leaving repeated period values blank after the first row in each period block
- [x] 4.5 Replace grouped period totals summary with a provider-based summary table that includes a `Provider` column
- [x] 4.6 Extend grouped aggregation metadata or calculation flow so weekly provider summaries can compute `Tokens/Day`, `Cost/Day`, and `Active Days` per provider within each week
