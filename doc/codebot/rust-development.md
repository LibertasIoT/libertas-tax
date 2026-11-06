# Tax App Rust development workflow

Use the canonical [Libertas CodeBot entry](https://github.com/LibertasIoT/libertas-doc/blob/main/CODEBOT.md)
and its [Rust card](https://github.com/LibertasIoT/libertas-doc/blob/main/codebot/rust.md).
The [Rust reference](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/languages/rust.md)
owns platform syntax, annotations, typed protocols, persistence and runtime
contracts. This App keeps its verification workflow here; platform documentation
is published only in libertas-doc.

## Required workflow

1. Read the [App authoring contract](../app-contract.md),
   [coverage audit](../../COVERAGE.md), owning source coverage comments
   and existing tests before choosing a complete form or rule. Maintainers also
   follow their locally installed instructions and private acceptance work.
2. Inspect the sibling `libertas-rs` checkout for the exact APIs used by this
   App's local path dependencies. Use Rust source as the schema owner; author
   source, ordinary project metadata and the source Display Template. Generated
   schemas, artifacts and deployment metadata belong to Studio's ordinary
   [compiler workflow](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md#schema-compilation-with-codex).
3. Implement the complete input, calculation, result and correction path. Use
   meaningful independent expected amounts and boundary/combination fixtures.
   Follow the shared [Data API failure contract](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/protocols.md#persistence-failure-and-the-no-status-api).
4. Run the applicable checks below. Schema or presentation changes also need
   ordinary Studio extraction and shared validation; native UI and live AI
   recognition require their own acceptance evidence.
5. Update the owning source coverage evidence and reconcile COVERAGE.md.
   Replace superseded facts and links at their owners. Report changed behavior,
   checks performed and remaining acceptance limits.

## Coverage evidence

Each form's audited status, supported lines, exclusions, tests and IRS sources
belong in a `// Coverage:` block at the top of its owning module; forms without
an owning module are listed in `src/lib.rs`. Update the block when coverage
changes and reconcile the sole [coverage document](../../COVERAGE.md).
Keep audit notes in plain `//` comments: `///` and `//!` become localized App
documentation and must contain user-facing meaning, not verification records
or author remarks.

## Verification

Run from this repository:

```sh
cargo fmt -p libertas-tax -- --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
git diff --check
```

Use focused regressions while implementing and the full applicable suite before
claiming completion. A passing build or mock test does not establish native UI,
provider recognition, tax-law completeness or a deployed version. Keep synthetic
fixtures beside their tests and private dated verification evidence with its
owning work item.

### Repeatable coverage measurements

Install [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) and the
toolchain's `llvm-tools-preview` component first. In Bash, set
`CARGO_TARGET_DIR` to your process's existing shared **absolute** target path.
Reserve that target and this worktree for the entire measurement. Run the full
verification gate once inside the instrumented subshell below; a maintainer's
full-gate wrapper can replace the three verification commands. Keep the default
test stack and thread settings.

```bash
(
  set -euo pipefail
  : "${CARGO_TARGET_DIR:?Set the shared absolute target directory}"
  export CARGO_TARGET_DIR
  export CARGO_LLVM_COV_TARGET_DIR="$CARGO_TARGET_DIR"
  export CARGO_LLVM_COV_BUILD_DIR="$CARGO_TARGET_DIR"
  artifacts="$CARGO_TARGET_DIR/coverage/$(git rev-parse --short=12 HEAD)"
  mkdir -p "$artifacts/tmp"
  export TMPDIR="$artifacts/tmp"
  { git rev-parse HEAD; git status --short; rustc -Vv;
    cargo llvm-cov --version; } > "$artifacts/coverage-context.txt"
  coverage_env=$(cargo llvm-cov show-env --sh)
  eval "$coverage_env"
  cargo llvm-cov clean --workspace
  cargo fmt -p libertas-tax -- --check
  cargo test --locked -j8 2>&1 | tee "$artifacts/tests.log"
  cargo clippy --locked --all-targets -- -D warnings
  cargo llvm-cov report | tee "$artifacts/summary.txt"
  cargo llvm-cov report --json --output-path "$artifacts/coverage.json"
  cargo llvm-cov report --lcov --output-path "$artifacts/lcov.info"
  cargo llvm-cov report --html --output-dir "$artifacts"
  cargo llvm-cov clean --workspace
)
```

`clean --workspace` removes stale measured-package artifacts and profiles while
retaining dependency builds; do not use bare `clean` on the shared target. Report
commands reuse the collected profiles without rerunning tests. Export all formats
before cleanup removes raw profiles and instrumented package artifacts. Exported
reports in the separate artifact directory survive cleanup. The final cleanup
prevents normal builds from reusing instrumented binaries; the subshell scopes
the instrumentation environment. Results identify executed code,
not tax-law completeness; these commands set no coverage threshold.

### Small mutation sample

Install [cargo-mutants](https://mutants.rs/) first. From a clean disposable
worktree, with exclusive use of the same shared target, run this separately from
the instrumented subshell. The fixed filter selects only three return-value
mutants of `trunc_i64`; existing ratio tests exercise signed division and its
failure boundaries. Listing records the selected sample without testing it.

```bash
(
  set -euo pipefail
  : "${CARGO_TARGET_DIR:?Set the shared absolute target directory}"
  export CARGO_TARGET_DIR
  artifacts="$CARGO_TARGET_DIR/coverage/$(git rev-parse --short=12 HEAD)"
  mkdir -p "$artifacts/tmp"
  export TMPDIR="$artifacts/tmp"
  { git rev-parse HEAD; git status --short; rustc -Vv;
    cargo mutants --version; } > "$artifacts/mutation-context.txt"
  sample=(--file src/kernel/ratio.rs --regex 'replace trunc_i64 -> i64 with'
          --no-shuffle)
  cargo mutants "${sample[@]}" --list --json > "$artifacts/sample.json"
  cargo mutants "${sample[@]}" --in-place --timeout 300 \
    --output "$artifacts/trunc-i64" --cargo-arg=--locked --cargo-arg=-j8 \
    -- --lib kernel::ratio::tests::
)
```

The normal baseline verifies the targeted tests before mutation. `--in-place`
reuses the shared build cache without cloning a target tree; do not edit, build
or commit concurrently. Review `git diff` afterward, especially after interruption,
because restoration can fail. Inspect `outcomes.json`, mutant diffs and logs under
`trunc-i64/mutants.out`; distinguish caught, missed, unviable and timed-out results.
A repeated run replaces that output (retaining one `mutants.out.old`); keep a
separate artifact directory if comparing runs. This deterministic sample is not
a project-wide mutation score.

The current protocol is in [protocol.rs](../../src/core/protocol.rs), stored types and
format marker in [model.rs](../../src/core/model.rs), and storage lifecycle in
[storage.rs](../../src/core/storage.rs). Extend those owners rather than retaining a
second platform manual here.

## Figure-equivalence oracle

The single `tests::cross::oracle::figure_equivalence_oracle` test reads
`tests/fixtures/oracle/corpus.zst` directly. Independent cases replay concurrently
with ordinary platform parallelism and default thread stacks. It checks the full
computed result, full native reply bytes and post-request interview state. Result
encoding v1 hashes all derived Debug fields, including private provenance; reply
hashing preserves the Response/Edit distinction.

The sole corpus file is a zstd-compressed record stream: `TAXORACLE01\n`, a
little-endian u64 input count, then u64-length-prefixed records. Each input record
contains its 64-byte ASCII SHA256 ID followed by native Avro bytes; the final
record is the JSON digest manifest. Input hashes, duplicate IDs, truncation and
trailing records are checked while loading. No extraction cache is needed.

Set `TAX_ORACLE_CAPTURE` to an absolute on-disk working directory when running
existing tests. Capture deduplicates inputs, rejects conflicting outputs and
records each actual named test thread in immutable provenance sidecars. Packing
includes the sorted test names in each manifest case; older manifests without
that metadata remain readable. Unnamed capture threads are refused rather than
assigned an invented test. Replay suppresses capture.

Reserve the worktree and its existing shared absolute `CARGO_TARGET_DIR` for the
whole command. First seed provenance on the unchanged baseline:

```bash
scripts/oracle-refresh.sh seed-provenance
```

Strict seeding retains every previous native input, case and expected digest;
missing or changed cases fail. It may add current builders' cases and attribution.
Then, only for a declared behavior, stored-shape or Rust representation change:

```bash
scripts/oracle-refresh.sh "Declared stored-shape change: describe it"
```

The command runs formatting, all tests with only the old oracle replay explicitly
filtered out, and strict Clippy, then packs and replays the candidate once before
atomically replacing `tests/fixtures/oracle/corpus.zst`. A failed gate or candidate
replay leaves the baseline untouched. Keep default test stacks and thread settings;
Cargo uses `-j8`. To use an existing full-gate wrapper instead of the default gate,
append `--` and its command arguments; the script appends the libtest
`-- --skip tests::cross::oracle::figure_equivalence_oracle` arguments to that wrapper.
The caller must reserve the shared target through the final replay as well.

Capture, logs, grouped diff and temporary working files are under a unique
`$CARGO_TARGET_DIR/oracle-refresh/run.*` directory. The compressed candidate is a
sibling of the destination to guarantee atomic promotion on its filesystem.
Review `diff.txt`: changes are grouped by test with case IDs, old/new output
digests and old/new case counts. Same-ID changes compare digests directly;
different IDs are additions/removals and are never paired by test or position.
Removed cases retain their seeded test attribution. Current-schema decoding
reports availability or its actual error; there is no decoder for retired App
bytes. Historical output values were never stored and cannot be recovered from
their hashes. Explain figure changes separately from declared shape-only changes.

The lower-level `TAX_ORACLE_PACK` path remains strict unless a nonempty
`TAX_ORACLE_REFRESH` reason and `TAX_ORACLE_DIFF` path explicitly request refresh.
`TAX_ORACLE_REPLAY` selects another packed file. Both paths validate the packed
candidate and replay it before promotion. Store native inputs, expected digests
and provenance only; full outputs are not corpus artifacts. Never refresh merely
to make an undeclared migration pass.
