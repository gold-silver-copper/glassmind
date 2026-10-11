# Quantities track handoff

Standalone package `glassmind-quantities`, automatically included by
the workspace's existing `crates/*` membership. Changes are confined to this new
crate and its Cargo.lock package entry. No shared CLI or workspace manifest edits.

## Implemented

- Checked, reduced i128 rational arithmetic; exact decimal/fraction parsing,
  arithmetic precedence, parentheses, unary signs, and English cardinal numbers
  through 999,999. Overflow and division by zero explicitly refuse.
- `TypedUnit<D>` and `Quantity<D>` for length, mass, duration, volume, absolute
  temperature and speed. Conversions check dimensions; addition accepts the same
  static dimension and excludes absolute temperature. Length / duration and
  speed * duration have typed results. Unit scales and temperature offsets are
  rational, never floating point.
- Gregorian dates in years 1..9999, leap centuries, signed day shifts, month/year
  shifts clamped to month end, weekdays and date differences. From/to differences
  are signed; between/and differences are absolute. Elapsed days exclude the start.
- Entity-bound inventories with checked receive/remove events, multi-event word
  problems, comparisons, groups, sharing, prices, percentages and rates. Entity,
  object and recipient mismatches refuse rather than silently mixing quantities.
- Fully consumed deterministic grammar, no LLM, learned model, network access or
  benchmark data lookup at runtime. Unit lexicons and numeral lexicons encode
  reusable concepts, not question/answer pairs.
- Public `solve(&str) -> Answer`, plus a standalone JSON CLI. `--jsonl` reads one
  JSON string per line and emits one tagged answer/refusal per line, including the
  operation used. Library modules expose the reasoning independently of parsing.

Example after building:

```sh
"${CARGO_TARGET_DIR:-target}/debug/glassmind-quantities" 'Convert 1/3 hours to minutes'
"${CARGO_TARGET_DIR:-target}/debug/glassmind-quantities" 'What date is 1 year after 2024-02-29?'
"${CARGO_TARGET_DIR:-target}/debug/glassmind-quantities" 'A train travels 120 kilometers in 90 minutes. What is its speed in m/s?'
```

## Measurement and data provenance

Both sets are locally generated synthetic **development** data, not an external
benchmark or independent language evaluation. Python's Fraction, Decimal,
datetime and calendar compute the answers independently of Rust. The generator
does not import Rust code, use its output as labels, or open external datasets.
It and the English grammar were authored in the same batch, so matching template
coverage is not evidence of open-ended language generalization.

| Source/version | Seed | Cases | Exact JSONL bytes | License |
| --- | ---: | ---: | ---: | --- |
| quantities-synthetic-dev-v1 | 20261010 | 2080 | 314258 | MIT OR Apache-2.0 |
| quantities-extended-dev-v1 | 20261011 | 1036 | 174437 | MIT OR Apache-2.0 |

Python version: 3.14.7. Each `dev/*-provenance.json` records the SHA-256 and full
source description. No SVAMP/MAWPS or other external items were downloaded,
trained on or evaluated; no held-out or test items were accessed.

| Development run | Right | Refusal | Wrong | Right includes expected refusals |
| --- | ---: | ---: | ---: | ---: |
| Initial core fixed sample (intermediate implementation) | 103 | 27 | 0 | 25 |
| Final core fixed sample | 130 | 0 | 0 | 25 |
| Final extended fixed sample | 92 | 5 | 0 | 32 |
| Full core | 2080 | 0 | 0 | 400 |
| Full extended | 976 | 60 | 0 | 362 |
| Full aggregate | **3056** | **60** | **0** | **762** |

The aggregate covers 3116 cases: **2294 substantive correct answers**, 762 correct
guard refusals, and 60 answerable paraphrases that the parser refuses. Refusal
counts in the table mean missing coverage on answerable questions. Unexpected
answers to invalid questions count as wrong. Reports contain per-category totals
and every development failure. `baseline-sample.json` is retained evidence from
the intermediate implementation; reproduction runs the final implementation.

Validation passed: targeted formatting check, clippy on all package targets with
warnings denied, build, both fixed samples, and both full dev suites. No unit
tests were added. No external benchmark score is claimed.

## Exact reproduction

From this worktree, run:

```sh
cd ~/code/glassmind
mkdir -p "$PWD/.quantities-tmp"
TMPDIR="$PWD/.quantities-tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=5s 600 bash crates/quantities/dev/reproduce.sh
```

`dev/reproduce.sh` contains the exact formatting, clippy, build, sample and full
evaluation commands. Every cargo invocation has TMPDIR, a capped scope, a process
session, a 600-second timeout and `-j 1` (except fmt). `RUST_TEST_THREADS=4` is set;
no test runner is invoked. `setsid --wait` is necessary here to retain supervision
when setsid forks. The adapter honors CARGO_TARGET_DIR or accepts `--binary`.
All generated files stay under the crate or the workspace-local scratch folder.
The script exits unsuccessfully if full dev contains a wrong answer. Its latest
successful output is preserved in `dev/reproduce.log`.

## Limits and next integration step

This is a bounded grammar, not a general word-problem reader. Broader narration
and paraphrases remain unsupported (the 60 measured misses). Number words are
supported in arithmetic expressions; most quantity/story forms still require
digits. Exponent notation, arbitrary compound dimensions, business calendars,
time zones, currency conversion and arbitrary precision beyond i128 are absent.
No external MAWPS/SVAMP adapter, trained language model or shared CLI integration
was implemented. The coordinator can call `glassmind_quantities::solve` from an
existing router and map tagged refusals normally; the binary requires no shared
application changes. All started compilation and evaluation processes finished.
