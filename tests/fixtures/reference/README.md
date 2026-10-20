# External-reference cases

`<source>/<case>.json`, run by `src/tests/cross/reference.rs` (one test iterates every case).
Source PDFs are not committed: each case records URL, fetch date and SHA-256.

- `source`, `source_year`, `year_policy` (`same-year` | `year-independent` | `adapted` with derivation).
- `neutral_facts`: facts the source omits, each with its reason.
- `filing`, `birth`, `pages`: ordered page answers entered through the client Requests
  (`children`: `[first, last, yyyymmdd]`; `wages`: whole-dollar W-2 boxes). Add a page kind in the runner when a case needs it.
- `expected`: `{form, line, amount, provenance}` (`printed` | `option` | `hand`), whole dollars; forms `1040`, `8812`, `EIC`.
- `expected_state`, `expected_notes`: result state and review-note keys (known gaps).
- `known_gaps`: excluded lines with their ledger row. `expect_mismatch`: `{form, line, class, triage}`
  for a printed line the App differs on (triage classes A-F in the private `reference/triage.md`);
  reported, not failed, and the case fails once the App matches so the entry gets removed.
