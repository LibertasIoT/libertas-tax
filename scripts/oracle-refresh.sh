#!/usr/bin/env bash
# Explicit full-builder recapture, grouped diff and validated atomic promotion.
set -euo pipefail
if [[ $# -lt 1 || -z ${1//[[:space:]]/} ]]; then
  echo 'Usage: scripts/oracle-refresh.sh "declared change reason" [-- full-gate argv...]' >&2
  exit 2
fi
reason=$1
shift
if [[ ${1-} == -- ]]; then shift; fi
: "${CARGO_TARGET_DIR:?Set the shared absolute target directory and reserve it for this gate}"
[[ $CARGO_TARGET_DIR == /* ]] || { echo 'CARGO_TARGET_DIR must be absolute' >&2; exit 2; }
export CARGO_TARGET_DIR
cd "$(dirname "$0")/.."
mkdir -p "$CARGO_TARGET_DIR/oracle-refresh"
artifacts=$(mktemp -d "$CARGO_TARGET_DIR/oracle-refresh/run.XXXXXXXX")
mkdir "$artifacts/tmp"
export TMPDIR="$artifacts/tmp"
unset TAX_ORACLE_CAPTURE TAX_ORACLE_PACK TAX_ORACLE_REPLAY TAX_ORACLE_REFRESH TAX_ORACLE_DIFF
export TAX_ORACLE_CAPTURE="$artifacts/capture"
printf 'Oracle refresh artifacts: %s\n' "$artifacts"
if (( $# )); then
  "$@" -- --skip tests::cross::oracle::figure_equivalence_oracle 2>&1 | tee "$artifacts/gate.log"
else
  cargo fmt -p libertas-tax -- --check
  cargo test --locked -j8 -- --skip tests::cross::oracle::figure_equivalence_oracle 2>&1 | tee "$artifacts/gate.log"
  cargo clippy --locked -j8 --all-targets -- -D warnings
fi
unset TAX_ORACLE_CAPTURE
export TAX_ORACLE_PACK="$artifacts/capture"
export TAX_ORACLE_DIFF="$artifacts/diff.txt"
# Seeding records actual origins while retaining every old byte/digest. Use reason
# 'seed-provenance' only on the unchanged baseline before a declared refresh.
if [[ $reason != seed-provenance ]]; then export TAX_ORACLE_REFRESH="$reason"; fi
cargo test --locked -j8 --lib tests::cross::oracle::figure_equivalence_oracle -- --exact --nocapture 2>&1 | tee "$artifacts/replay.log"
printf 'Validated oracle promoted. Artifacts: %s\n' "$artifacts"
