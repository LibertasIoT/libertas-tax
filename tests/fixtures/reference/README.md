# External-reference cases

`<source>/<case>.json`, run by `src/tests/cross/reference.rs` (one test iterates every case).
Source PDFs are not committed: each case records URL, fetch date and SHA-256.

- `source`, `source_year`, `year_policy` (`same-year` | `year-independent` | `adapted` with derivation).
- `neutral_facts`: facts the source omits, each with its reason.
- `filing`, `birth`, `spouse_birth` (joint returns; defaults to `birth`), `pages`: ordered page answers entered through the client Requests
  (`children`: `[first, last, yyyymmdd]`; `wages`: W-2 boxes in dollars, cents allowed;
  `business`: one Schedule C business with `receipts`, `expenses` by field name, `other` rows, `special`, `startup`;
  `bought_production_credit`: a purchased Form 7207 credit;
  `dividends`: 1099-DIV boxes 1a, 1b, 2a; `farm`: one Schedule F farm with its Form 4136 gasoline lines;
  `car_loans`: Schedule 1-A Part IV interest; a W-2 `overtime` is box 12 code TT, `care_benefits` box 10;
  `documents`: every document of one topic page (interest, SSA-1099, 1099-R with its taxable-amount method,
  early Roth distribution; see `reference_pages.rs`);
  `ira_contributions`, `mortgages`, `home_office` (simplified method by month), `care_children`, `care`, `education`,
  `foreign_relatives` and `parent`). `dependent: true` makes the filer a child who can be claimed (Form 8615).
  Add a page kind in the runner when a case needs it.
- `expected`: `{form, line, amount, provenance}`, whole dollars. `provenance`: `printed` (copied from the source),
  `stated` (the source states the outcome in words, such as no credit), `derived` (printed inputs plus a statutory
  constant or rate named in `derivation`), `option`, `hand`, or `corrected` (the source figure is wrong, see
  `source_printed`). Forms `1040`, `8812`, `EIC`, `Sch1`, `Sch2`, `Sch3`, `SchA`, `SchC`, `SchF`, `SchSE`, `2441`, `3800`, `4562-B`, `5329`, `8615`, `8863`, and `calc`, a calculation line the App
  does not tag with a form line (`line` is its id, whole dollars rounded half up); a printed blank is 0.
- `expected_state`, `expected_notes`, `expected_issues`: result state, review-note keys and refusal keys (known gaps).
- `known_gaps`: excluded lines with their ledger row. `expect_mismatch`: `{form, line, class, triage}`
  for a printed line the App differs on (triage classes A-F in the private `reference/triage.md`);
  reported, not failed, and the case fails once the App matches so the entry gets removed.
