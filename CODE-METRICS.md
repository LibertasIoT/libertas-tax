# Tax app code metrics

**Generated:** 2026-11-06 02:35:48 UTC

**Local time:** 2026-11-05 21:35:48 EST (-0500, America/New_York)

**Git revision:** `18c687b438ef2963e5f037b9a4321e9d5323607f`

**Source:** The checked-in libertas-tax App source at this revision.

The App contains **345,725 code lines and 7,652 implemented functions**, excluding tests and test fixtures. Rust contributes 344,618 code lines across 690 files; the saved display template contributes 1,107 lines.

Tax calculations and validation account for **186,455 lines (53.9% of the App)**. Core return calculation is estimated at **107,300 code LOC (31.0%)**, with an approximate sampling-only 95% interval of **97,700–116,900 LOC**. Including reference tables brings the estimated footprint to **111,200 LOC**.

## Counting basis

- Code LOC counts physical lines with at least one non-comment source token. Blank lines and comment-only lines are excluded; mixed code/comment lines count once.
- Rust documentation comments, including UI labels and help, are excluded. Schema attributes and string literals remain code; every line of a multi-line string literal counts.
- Native production files are reached from `src/lib.rs` through module declarations, including `#[path]`. Test-only modules, declarations, blocks, parameters and arguments are excluded by evaluating `cfg(test)` as false; other conditional production branches remain included.
- The 19 topic `src/*/strings.rs` catalogs are counted separately as production localization authoring assets. Studio extracts these named catalogs for the App; their native Rust catalog views are test-only. The test-only catalog-view implementation and module wiring are excluded.
- Functions are explicit definitions, including methods and nested functions. Closures, bodiless trait signatures and macro-generated functions are excluded.
- Dependencies, build output, fixtures, Cargo metadata, documentation, extraction tools and `translations/` are excluded. The saved `display.json` is counted once; its authoring fragments are excluded to avoid counting the same rules twice.
- The counted source is 314,107 native-production Rust lines, 30,511 localization-catalog lines and 1,107 display-template lines.

**Display snapshot:** `display.json` contains 1,102 saved rules. The current source fragments assemble to 1,092 rules, and `tools/assemble-display --check` reports a mismatch. These measurements describe the saved output at the named revision.

**Measured and estimated values.** File, category, tax-area, module and function counts are source measurements under these rules. Responsibility shares, return calculation and the largest-function breakdown are estimates from 650 reviewed random code-line samples. App categories partition the total; tax areas and responsibilities are separate views of the tax-processing subtotal.

## Fields and localized strings

| Metric | Count |
| --- | ---: |
| Struct fields in production Rust declarations | 15,667 |
| Enum variant payload fields in production Rust declarations | 4,679 |
| **Total source field declarations** | **20,346** |
| Localized source string entries | 59,317 |
| Keyed message and line-label resources, included above | 11,324 |

The 61,131-line schema and data category includes internal Rust types. Its field count includes named and tuple fields, with tests excluded. Each declaration is counted once, regardless of how often its type is used. Enum choices and Union variant names are counted separately from fields.

The compiled App schema's fields, and those a filer can see, are counted with the screens under [Fields](#fields).

Localized strings are counted by source-document path, including labels, descriptions and string resources. Repeated text at different paths counts separately. Of the 59,317 entries, 40,209 contain text and 19,108 are empty. This string-entry count is separate from the 30,698 code lines assigned to localized messages and line labels.

The compiled document contains 11,324 keyed message and line-label resources in `StringResources`, all with nonempty text. These are included in the localization total. Each resource key counts once, regardless of its number of call sites; keys sharing the same text remain separate entries.

<!-- screen-metrics:start -->
## Screens

**190 distinct full-page designs** are declared by the reachable endpoint workflow and compiled Display Template, including **120 full list-child editor designs**. Of those list-child editors, **43** directly edit a printed-form declaration with a nonempty `LLMDocTrans.doc_type`.

| Screen kind | Designs |
| --- | ---: |
| Top-level Request editors | 47 |
| Top-level Response views | 9 |
| Full list-child editors, including concrete form alternatives | 120 |
| Full list-child read-only viewers | 14 |
| **Total full pages** | **190** |
| Compact Request popups, separate from full pages | 2 |

A screen is a reusable declaration design, not a record instance. Repeated children, Add versus Edit, conditional field values, tabs, folds, device widths and pane placement do not multiply it. A summary/grid List opens a complete child page; nested summary/grid Lists add their own pages. An inline Table, ordinary table row, Union chooser or form subsection adds no page by itself.

The counter follows DefaultRequest, NextRequest, NextResponse, WorkflowRequest, PrevRequest and declared links. WorkflowRequest carrier replies, calculation-only traffic, acknowledgement replies, confirmations and error dialogs are excluded. A server-delivered Request retains its full editor. The count describes possible declared surfaces across conditional branches; a particular return can visit fewer pages, and automatic forwarding can skip a declared landing.

List-child designs are deduplicated by resolved member declaration, its primary concrete Union alternative, and edit/view mode. An AskOnAdd document Union supplies concrete alternatives of the complete child editor; the generic wrapper and each form are not added together. Inline choices inside a form stay within that form's design. The JSON inventory records every page/path using each design.

The [page inventory](PAGE-INVENTORY.md) lists the top-level pages and full list-child designs.

### Fields

| Field count | Fields |
| --- | ---: |
| Fields in the compiled App schema | 12,099 |
| Fields a filer can see | 7,911 |

A field is a member of a Table: a top-level type, a Union variant payload, a protocol payload or a List element. Each declaration counts once, regardless of how often its type is used; List element descriptors and Union choice names add no fields. A filer can see a field when some path from an endpoint protocol variant reaches it without passing a field, variant or referenced declaration marked Hidden or ExcludeUI. Visibility is static: data-dependent presentation and Display Template layout do not change it.

Regenerate with the compiled Libertas Studio checkout:

```sh
node tools/screen-metrics.cjs --studio /path/to/libertas/vscode --update --json /tmp/tax-screen-metrics.json
```

The tool uses Studio's authoritative compiler, effective-attribute resolution, Request input classification and Display Template cascade. It reads saved source and updates this report section, including the field counts, and `PAGE-INVENTORY.md`. Schema content identity: `sha256:fcedd5cd0bed9980d987be9d6284eee0f1094911f2c809bbb22e845077e827f1`.
<!-- screen-metrics:end -->

## App categories

| Category | Code LOC | Functions | App LOC share |
| --- | ---: | ---: | ---: |
| Tax calculations and validation | 186,455 | 5,491 | 53.93% |
| Interview, navigation and page mapping | 58,631 | 2,040 | 16.96% |
| Schema and data declarations | 61,131 | 0 | 17.68% |
| Localized messages and line labels | 30,698 | 2 | 8.88% |
| Tax reference tables | 3,912 | 5 | 1.13% |
| Libertas platform integration | 154 | 11 | 0.04% |
| App cache and return persistence | 1,153 | 48 | 0.33% |
| Crate wiring, reexports, macros and output adapters | 1,997 | 8 | 0.58% |
| Shared helpers | 487 | 47 | 0.14% |
| Display rules in `display.json` | 1,107 | 0 | 0.32% |
| **Total** | **345,725** | **7,652** | **100%** |

Declarations are separated from their methods. Methods in model, page, editor, protocol and interview modules belong to interview/page mapping; numerical work inside them is included in the cross-category calculation estimate below. Persisted root and section-record implementations retain their cache/persistence category. Shared address/date/validation helpers are counted separately.

The module mapping follows the current layout: every `src/federal/interview/` child and every `*_page.rs`/`*_model.rs` joins the existing page/model category. Topic localization catalogs and section-label selection join the message category. Crate wiring includes the shared `line_emission.rs` output adapters, reexports and source/issue macros. These are source-role categories, not a count of every use of an SDK API.

## Tax areas

Tax processing is assigned by primary domain-module responsibility. Extracted `src/federal/rules/<domain>/` modules follow their named domain, including a separate rental group. The federal return engine and shared calculation kernels serve multiple areas and remain separate. Type declarations and page/model methods are outside this subtotal.

| Tax area | Code LOC | Functions | Tax-processing LOC share |
| --- | ---: | ---: | ---: |
| Business, pass-throughs, QBI and loss limits | 33,818 | 1,049 | 18.1% |
| Capital gains, property sales and depreciation | 32,145 | 1,061 | 17.2% |
| Federal return engine and shared support | 20,811 | 441 | 11.2% |
| Health insurance and medical accounts | 13,464 | 442 | 7.2% |
| Retirement, pensions and Social Security | 12,646 | 381 | 6.8% |
| Farming | 9,783 | 295 | 5.2% |
| Rentals and passive losses | 9,206 | 139 | 4.9% |
| Spousal attribution, community property and injured spouse | 8,961 | 245 | 4.8% |
| Energy, vehicle and business credits | 8,550 | 298 | 4.6% |
| Family, education and dependent credits | 8,326 | 309 | 4.5% |
| Personal deductions and charitable gifts | 6,848 | 232 | 3.7% |
| Foreign income, credits and reporting | 6,192 | 153 | 3.3% |
| Wages and other source income | 6,176 | 256 | 3.3% |
| Other taxes, household payroll and payments | 3,527 | 66 | 1.9% |
| Debt cancellation and claim-of-right repayment | 2,929 | 69 | 1.6% |
| Unknown answers, omissions and scope policy | 2,848 | 51 | 1.5% |
| State-source record validation | 225 | 4 | 0.1% |
| **Total** | **186,455** | **5,491** | **100%** |

Specialist code outside the shared federal engine totals 165,644 lines and 5,050 functions. Source distribution does not measure tax-situation coverage; [COVERAGE.md](COVERAGE.md) owns that scope.

## Responsibilities

This is an **approximate semantic classification estimated from a sample**. A random sample of 400 tax-processing code lines was reviewed in surrounding production source. Each line was assigned once to its dominant purpose; structural lines follow their enclosing operation. Shares are applied to the 186,455-line population. Functions are not counted by responsibility.

| Responsibility | Lines in sample | Estimated code LOC | Approximate 95% interval | Tax-processing LOC share |
| --- | ---: | ---: | ---: | ---: |
| Input and consistency validation | 79 | 36,800 | 29,600–44,100 | 19.8% |
| Tax eligibility, treatment and limits | 36 | 16,800 | 11,600–22,000 | 9.0% |
| Arithmetic and amount aggregation | 45 | 21,000 | 15,200–26,700 | 11.2% |
| Result lines, explanations and provenance | 64 | 29,800 | 23,100–36,500 | 16.0% |
| Scope, omissions and conservative assumptions | 34 | 15,800 | 10,800–20,900 | 8.5% |
| Coordination, data preparation and accessors | 142 | 66,200 | 57,500–74,900 | 35.5% |
| **Total** | **400** | **186,500** | | **100%** |

Estimates are rounded to the nearest 100 LOC; rounded rows can differ from rounded totals. The categories mean:

- **Validation:** validity and consistency checks, duplicate or incompatible facts, and refusal/error paths.
- **Tax treatment:** eligibility, applicable treatment, elections, tax limits and phaseouts.
- **Arithmetic:** numerical operations, amount aggregation, allocation and rounding.
- **Output:** result/form lines, explanatory messages, provenance and diagnostic traces.
- **Scope:** unsupported situations, retained unknowns, omissions and conservative assumptions.
- **Coordination:** sequencing, input selection/preparation, conversions and accessors.

Declarative validation remains in the schema category. A function can contain several responsibilities; a file named `rules` is not validation-only code.

## Return calculation estimate

Return calculation transforms supplied tax facts into numerical results and computed form values. It includes numerical input selection/preparation, eligibility and treatment used by calculations, dependent-calculation coordination, retained numerical intermediates and assignment of computed amounts to form lines.

Validity/refusal checks, supported-scope handling, explanatory messages, provenance and diagnostic traces are outside this estimate, as are navigation, editable/stored-record conversion, display formatting and persistence. Arithmetic used solely to check whether inputs are valid remains validation. Selecting an amount used in a tax calculation is calculation work.

Each tax-processing sample received a separate calculation decision. Another 150 interview/page-mapping lines were reviewed for numerical helpers. Samples were drawn without replacement using Python's random generator and seed `20261019`; every selected line was reviewed in its function and surrounding source.

| Calculation responsibility | Calculation lines in sample | Estimated code LOC |
| --- | ---: | ---: |
| Input and consistency validation | 0 of 79 | 0 |
| Tax eligibility, treatment and limits | 36 of 36 | 16,800 |
| Arithmetic and amount aggregation | 45 of 45 | 21,000 |
| Result lines, explanations and provenance | 33 of 64 | 15,400 |
| Scope, omissions and conservative assumptions | 0 of 34 | 0 |
| Coordination, data preparation and accessors | 101 of 142 | 47,100 |
| Calculation helpers in interview/page-mapping methods | 18 of 150 | 7,000 |
| **Core return calculation** | | **107,300** |
| Tax reference tables, counted separately | | 3,912 |
| **Calculation logic plus reference tables** | | **111,200** |

Within tax processing, 215 of 400 sampled lines (53.8%) were calculation work, about 100,200 LOC. The page/model sample contributes approximately 7,000 LOC. The unrounded estimate is `186,455 × 215/400 + 58,631 × 18/150 = 107,255.3` LOC.

The approximate sampling-only 95% interval is 97,700–116,900 LOC before reference tables, combining the two independent sample variances with finite-population correction. It is conditional on the semantic judgments; classification uncertainty is not quantified by this interval. This is a functional view across source categories, not another category to add to the App total.

## Largest tax-processing modules

| Module | Tax-processing code LOC | Functions |
| --- | ---: | ---: |
| [federal_rules.rs](src/federal/federal_rules.rs) | 10,424 | 79 |
| [asset_sale.rs](src/investment/asset_sale.rs) | 3,577 | 82 |
| [unknowns.rs](src/core/unknowns.rs) | 2,848 | 51 |
| [section1231.rs](src/investment/section1231.rs) | 2,776 | 56 |
| [general_business_credit.rs](src/credits/general_business_credit.rs) | 2,687 | 95 |
| [k1_activity.rs](src/passthrough/k1_activity.rs) | 2,419 | 90 |
| [form_8995_rules.rs](src/federal/rules/activity/form_8995_rules.rs) | 2,324 | 11 |
| [section179/mod.rs](src/cost_recovery/section179/mod.rs) | 2,264 | 55 |
| [owners.rs](src/attribution/injured_spouse/owners.rs) | 2,127 | 12 |
| [estate_trust.rs](src/passthrough/estate_trust.rs) | 2,097 | 72 |
| [partnership.rs](src/passthrough/partnership.rs) | 1,881 | 76 |
| [form_1116.rs](src/foreign/form_1116.rs) | 1,875 | 46 |

These module totals include only tax-processing lines; declarations retain their schema category.

## Largest mixed functions

| Function | Module | Starting source line | Code LOC |
| --- | --- | ---: | ---: |
| `estimate_with_ira_pass_with_claims` | [federal_rules.rs](src/federal/federal_rules.rs) | 6,274 | 5,294 |
| `project_sources` | [owners.rs](src/attribution/injured_spouse/owners.rs) | 196 | 2,016 |
| `report` | [reporting.rs](src/federal/rules/rentals/reporting.rs) | 50 | 1,763 |
| `project` | [direct.rs](src/federal/rules/rentals/direct.rs) | 35 | 1,386 |
| `collect` | [linked_sources.rs](src/federal/rules/rentals/linked_sources.rs) | 60 | 1,217 |
| `calculate` | [hsa.rs](src/health/hsa.rs) | 380 | 1,057 |
| `collect_with_claims` | [federal_rules.rs](src/federal/federal_rules.rs) | 3,870 | 1,004 |
| `collect_business` | [schedule_c_rules.rs](src/federal/rules/business/schedule_c_rules.rs) | 236 | 934 |

The largest function, `estimate_with_ira_pass_with_claims`, contains 5,294 code lines. Its mixed responsibilities were reviewed with a separate random sample of 100 lines:

| Responsibility | Lines in sample | Estimated code LOC | Function LOC share |
| --- | ---: | ---: | ---: |
| Input and consistency validation | 1 | 100 | 1% |
| Tax eligibility, treatment and limits | 2 | 100 | 2% |
| Arithmetic and amount aggregation | 15 | 800 | 15% |
| Result lines, explanations and provenance | 20 | 1,100 | 20% |
| Scope, omissions and conservative assumptions | 6 | 300 | 6% |
| Coordination, data preparation and accessors | 56 | 3,000 | 56% |
| **Total** | **100** | **5,300** | **100%** |

With 100 sampled lines, the largest possible normal-approximation sampling interval is about ±10 percentage points, conditional on the judgments. 88 sampled lines were return-calculation work. The two largest tax-processing functions together contain 7,310 code lines, or 3.9% of tax processing.

## Libertas platform integration

| Category | Code LOC | Functions |
| --- | ---: | ---: |
| Libertas platform integration | 154 | 11 |
| App cache and return persistence | 1,153 | 48 |
| Crate wiring, reexports, macros and output adapters | 1,997 | 8 |
| **Total** | **3,304** | **67** |

The platform boundary comprises endpoint handling and registration in [lib.rs](src/lib.rs), plus the concrete transaction and return-store adapters in [storage.rs](src/core/storage.rs). It includes 11 explicit definitions: `handle_request`, `tax_interview`, `get`, `has_records`, `records`, `delete_records`, `put`, `delete`, `try_load`, `reset_mismatched` and `persist_with`.

Cache and persistence covers accepted-return loading/saving, LRU behavior, section and household/income record encodings, and persisted-root methods. Schema declarations remain in the schema category. Crate wiring also includes the eight generic form-line output adapters in [line_emission.rs](src/federal/line_emission.rs).

This measures the explicit endpoint/storage boundary. Other SDK-dependent code retains its functional source category.

## Comparison with the public snapshot

The [public report](https://github.com/LibertasIoT/libertas-tax/blob/0fb6b04d6c22cb38dbf44934c973c73833ac41f3/CODE-METRICS.md) describes the October 20 snapshot. Rerunning the present lexical counter on that snapshot yields 255,143 code LOC and 5,548 functions, a small counting-method difference from its published 254,627 LOC and 5,539 functions. The comparison below uses the remeasured baseline.

| Measurement | Public snapshot, remeasured | Current source | Change |
| --- | ---: | ---: | ---: |
| App code LOC, including display | 255,143 | 345,725 | +90,582 (35.5%) |
| Implemented functions | 5,548 | 7,652 | +2,104 (37.9%) |
| Counted Rust files | 280 | 690 | +410 |
| Saved display-template LOC | 814 | 1,107 | +293 |

The category mapping recognizes the current model/page splits, topic message catalogs and domain rule modules. The current estimates use newly reviewed samples; the public report's semantic percentages were not carried forward. Smaller individual functions and more files partly reflect extraction into focused modules.

## Snapshot and interpretation

These are source measurements at `18c687b438ef`, not release metadata, a runtime benchmark or evidence of tax-law completeness. The source and saved display template were measured as checked in; translation files and other unrelated local files are outside the counting scope.

Totals, categories, areas, modules and functions are measurements under the stated lexical and module-mapping rules. Responsibilities and calculation are sample-based semantic estimates. Every sampled line receives one dominant responsibility; calculation is a separate decision, so supporting output and actual numerical work can be distinguished within the same function.
