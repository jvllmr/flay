# Flay integration tests gate

The full integration suite (`tests/test_integration.py`) is skipped by pytest
unless the `FLAY_INTEGRATION_TEST` env var is set. It is slow and must not run
as part of routine checks.

## Rules

1. Run the full integration suite only when BOTH are true:
   - It is a commit for this repository (or the user explicitly asked for or
     approved an integration-test run), and
   - The command sets the flag: `FLAY_INTEGRATION_TEST=1 uv run pytest
tests/test_integration.py`.
2. Never set `FLAY_INTEGRATION_TEST` for regular unit-test runs, linting, or
   exploratory verification. Plain `uv run pytest` is the default check.
3. If a task would benefit from integration coverage but is not at commit
   time, ask the user first. State the cost: integration suite runs only with
   the flag and is slow.
4. If integration tests fail at commit time, fix the failure before
   committing. Do not commit with a known integration failure unless the user
   says to.

## Commands

- Default (unit) check: `uv run pytest`
- Full integration suite, gated: `FLAY_INTEGRATION_TEST=1 uv run pytest tests/test_integration.py`
- Whole suite including integration, gated: `FLAY_INTEGRATION_TEST=1 uv run pytest`

# Benchmarks

The `benchmark` job in `.github/workflows/test.yml` benchmarks the
bundling/treeshaking process of the integration tests (the pytest-benchmark
`benchmark` fixture in `tests/test_integration.py` wraps
`bundle_package`/`treeshake_package`) and publishes the results to GitHub
Pages via `benchmark-action/github-action-benchmark`. Regular test jobs pass
`--benchmark-disable` so benchmarks run only in the benchmark job.

Not offending benchmark results is important in this repo: benchmark
regressions are failures. The CI job sets `fail-on-alert: true`, so treat any
benchmark alert as a regression to fix before committing or merging. Do not
commit changes that regress the integration benchmarks published on the Pages
site.
