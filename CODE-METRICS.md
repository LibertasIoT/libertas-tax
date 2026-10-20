# Tax app code metrics

**Generated:** 2026-10-20 03:16:18 UTC

**Local time:** 2026-10-19 23:16:18 EDT (-0400, America/New_York)

**Git revision:** `5c31a8a` (development history; this repository's initial snapshot was made from it)

**Source:** The libertas-tax App source of this snapshot. Counted source files match the snapshot exactly.

The tax app contains **254,627 code lines and 5,539 implemented functions**, excluding tests. Rust contributes 253,813 lines across 280 files; the display template contributes another 814 lines. The two table extraction tools counted in the previous report are not part of this snapshot and are no longer counted.

Tax calculations and validation account for 144,715 lines, or 56.8% of the app. Within that category, the sample-based responsibility estimate assigns 22.8% to arithmetic, 22.8% to coordination and data preparation, 21.8% to input validation, 16.0% to result output, 9.0% to tax treatment and eligibility, and 7.8% to scope and assumption handling.

**Core return calculation is estimated at about 86,700 code LOC, or 34.0% of the app**, with a practical range of 77,000–97,000 LOC. It spans arithmetic, tax treatment, calculation preparation and computed form values, including helpers housed in model and page-mapping methods. Including the separately counted tax reference tables brings that footprint to about 90,600 LOC, or 35.6% of the app.

## Counting basis

- Code LOC counts physical lines containing source code. Blank lines and comment-only lines are excluded; a line with both code and a comment counts once.
- Source comments include Rust documentation comments, which can supply UI labels and help. Their text is excluded from code LOC. Schema attributes and string literals remain code; every line of a multi-line string literal counts.
- Both test directories and embedded test-only declarations, blocks, parameters and arguments are excluded. Production files are the modules reached from `src/lib.rs` outside test-only `cfg` attributes, so test-only module files beside production code are excluded as well. Production branches of conditional code remain included.
- Functions are explicit definitions, including methods and nested functions. Closures, trait signatures without bodies, and macro-generated functions are excluded.
- Dependencies, build output, test fixtures, Cargo manifests and locks, documentation and this report are excluded.
- The counted files are 280 Rust files and one display JSON file.

**Measured and estimated values.** File, category, tax-area, module and function counts are measurements under these rules. The responsibility shares, the return-calculation estimate and the breakdown of the largest function are estimates from reviewed random samples of code lines, described below.

The counting method was rerun on the revision of the previous report. It reproduces that report's 147 files, 2,383 functions, every category's function count and the exact message, table and helper totals. Interview/page mapping and schema declarations are within 0.2%, and the small endpoint, persistence and wiring buckets differ by at most 12 lines each. Tax processing is measured 2.4% lower, because this counter excludes multi-line test-only message arguments in full. Comparisons with the previous report carry that small difference.

The app categories partition the total. The tax-area and responsibility tables are two independent views of the same 144,715-line tax-processing subtotal. Percentages are rounded.

## App categories

| Category | Code LOC | Functions | App LOC share |
| --- | ---: | ---: | ---: |
| Tax calculations and validation | 144,715 | 4,236 | 56.83% |
| Interview, navigation and page mapping | 31,894 | 1,213 | 12.53% |
| Schema and data declarations | 41,833 | 0 | 16.43% |
| Localized messages and line labels | 28,619 | 0 | 11.24% |
| Tax reference tables | 3,910 | 5 | 1.54% |
| Libertas platform integration | 126 | 9 | 0.05% |
| App cache and return persistence | 1,236 | 44 | 0.49% |
| Crate wiring, reexports and macros | 1,226 | 0 | 0.48% |
| Shared helpers | 254 | 32 | 0.10% |
| Display rules in `display.json` | 814 | 0 | 0.32% |
| **Total** | **254,627** | **5,539** | **100%** |

Declarations are counted separately from their methods. For example, a tax model declaration belongs to the schema category, while its conversion or default methods belong to interview and page mapping. The localized message catalog and reference-table modules are also separate from tax-processing logic.

Functions are assigned by module: the interview, walk, model, protocol and page modules make up interview and page mapping; the storage-record, section, model and format-migration modules in `src/core` make up cache and persistence; `common.rs`, `dates.rs` and `validation.rs` in `src/core` are shared helpers. Tax reference tables now include the annual tax parameters in `year_constants.rs` and three form-line catalogs, besides the annuity, EIC, RMD and MACRS tables.

## Tax areas

These categories count specialist tax processing by primary module responsibility, following the domain directories of `src`. The federal return engine, including the shared calculation kernels in `src/kernel`, serves several tax areas and remains separate. Functions in extended_rules.rs are assigned to their relevant areas by the file's topical sections; its shared scaffolding remains with the federal engine. Spousal attribution is a new row: community-property and injured-spouse allocation now has its own directory. Some calculation helpers live in model/page-mapping methods outside this subtotal and are included in the return-calculation estimate below.

| Tax area | Code LOC | Functions | Tax-processing LOC share |
| --- | ---: | ---: | ---: |
| Federal return engine and shared support | 30,511 | 408 | 21.1% |
| Business, pass-throughs, QBI and loss limits | 20,662 | 787 | 14.3% |
| Capital gains, property sales and depreciation | 19,191 | 647 | 13.3% |
| Health insurance and medical accounts | 10,741 | 363 | 7.4% |
| Retirement, pensions and Social Security | 9,967 | 325 | 6.9% |
| Farming | 8,275 | 261 | 5.7% |
| Rentals and passive losses | 6,812 | 104 | 4.7% |
| Spousal attribution, community property and injured spouse | 6,777 | 220 | 4.7% |
| Energy, vehicle and business credits | 6,590 | 287 | 4.6% |
| Family, education and dependent credits | 5,700 | 207 | 3.9% |
| Foreign income, credits and reporting | 4,889 | 119 | 3.4% |
| Wages and other source income | 3,329 | 159 | 2.3% |
| Debt cancellation and claim-of-right repayment | 3,057 | 76 | 2.1% |
| Other taxes, household payroll and payments | 2,882 | 79 | 2.0% |
| Personal deductions and charitable gifts | 2,703 | 135 | 1.9% |
| Unknown answers, omissions and scope policy | 2,469 | 46 | 1.7% |
| State-source record validation | 160 | 13 | 0.1% |
| **Total** | **144,715** | **4,236** | **100%** |

The federal engine, business, capital/property and health groups together account for 56.0% of the tax-processing code. Specialist code outside the shared engine totals 114,204 lines and 3,828 functions. These percentages describe source distribution; they do not measure tax-situation coverage.

## Responsibilities

This is an **approximate semantic classification estimated from a sample**. A random sample of 400 tax-processing code lines was reviewed in its surrounding production code; each sampled line was assigned once, to its main purpose, and the shares were applied to the 144,715-line population. Unlike the previous report, which classified the whole category and counted functions by their dominant responsibility, this estimate is line-based, so functions are not counted per responsibility.

| Responsibility | Lines in sample | Estimated code LOC | Approximate 95% interval | Tax-processing LOC share |
| --- | ---: | ---: | ---: | ---: |
| Input and consistency validation | 87 | 31,500 | 25,600–37,300 | 21.8% |
| Tax eligibility, treatment and limits | 36 | 13,000 | 9,000–17,100 | 9.0% |
| Arithmetic and amount aggregation | 91 | 32,900 | 27,000–38,900 | 22.8% |
| Result lines, explanations and provenance | 64 | 23,200 | 18,000–28,300 | 16.0% |
| Scope, omissions and conservative assumptions | 31 | 11,200 | 7,400–15,000 | 7.8% |
| Coordination, data preparation and accessors | 91 | 32,900 | 27,000–38,900 | 22.8% |
| **Total** | **400** | **144,700** | | **100%** |

Estimated LOC are rounded to the nearest 100. The responsibilities have these meanings:

- **Input and consistency validation:** rejects malformed or inconsistent facts, including invalid dates, incompatible amounts, duplicate records and unreconciled histories.
- **Tax eligibility, treatment and limits:** selects the applicable treatment, eligibility, rate bands, elections, exclusions, deduction caps and phaseouts.
- **Arithmetic and amount aggregation:** computes amounts through rates, allocation, rounding, subtraction and accumulation.
- **Result lines, explanations and provenance:** assembles displayed forms and calculation lines, explanatory messages, input references and source references.
- **Scope, omissions and conservative assumptions:** handles supported scope, excluded items, retained unknowns, conservative choices and their notices.
- **Coordination, data preparation and accessors:** sequences component calculations, selects and prepares records, converts representations and retrieves values.

Arithmetic, coordination and validation are the three largest responsibilities at a little over one fifth of the category each. Validation, tax treatment and scope/assumption handling together account for about 55,700 lines, or 38.5%. A complete calculation also depends on preparation, tax treatment and output; the arithmetic row measures its numerical operations.

Declarative validation in schema attributes remains in the schema category outside this subtotal. Rules, validation, arithmetic and output are mixed in many functions; file names ending in rules do not identify validation-only code.

## Return calculation estimate

For this estimate, **return calculation** transforms supplied tax facts into numerical tax results and computed form values. It includes selecting and preparing numerical inputs, applying tax treatment and eligibility, calculating and coordinating dependent amounts, retaining numerical values needed by later calculations, and assigning computed values to form lines.

Input validity and rejection checks, supported-scope and unknown-answer handling, explanatory messages, provenance and diagnostic-only traces are counted separately. Interview navigation, conversion between editable answers and stored records, display formatting, declarations, cache/storage and platform integration are also outside this calculation estimate. An accessor that selects an amount used by a calculation is included; a general record or presentation accessor is excluded.

Each sampled tax-processing line also received a calculation decision, and a separate sample of 150 interview/page-mapping lines was reviewed for calculation helpers. Both samples were drawn without replacement using Python's random generator with seed `20261019`. Each sampled line was reviewed in its surrounding production code and function; ambiguous samples received wider source review. Statement binding and structural lines follow the enclosing operation's purpose, while separate message/provenance arguments remain outside calculation.

| Calculation responsibility | Calculation lines in sample | Estimated code LOC |
| --- | ---: | ---: |
| Arithmetic and amount aggregation | 90 of 91 | 32,600 |
| Tax eligibility, treatment and limits | 34 of 36 | 12,300 |
| Calculation preparation and coordination from the coordination bucket | 66 of 91 | 23,900 |
| Computed form values and calculation work from the result-output bucket | 41 of 64 | 14,800 |
| Numerical work inside scope and omission handling | 1 of 31 | 400 |
| Calculation helpers housed in model/page-mapping methods | 13 of 150 | 2,800 |
| **Core return calculation** | | **86,700** |
| Tax reference tables, counted separately from calculation logic | | 3,900 |
| **Calculation logic plus reference tables** | | **90,600** |

Values in this table are rounded to the nearest 100 LOC; the total uses unrounded values. No sampled validation line was calculation work. Within the 144,715-line tax-processing category, 232 of 400 sampled lines (58.0%) were calculation work, about 83,900 LOC. Another approximately 2,800 LOC comes from the 31,894-line interview/page-mapping category, principally numerical model helpers. Thus return calculation is a functional view across the existing app categories, rather than an additional category to add to the app total.

The unrounded estimate is `144,715 × 232/400 + 31,894 × 13/150`, approximately 86,699 LOC. This is an estimate of source footprint, not an exact function boundary or a percentage of tax situations implemented.

Sampling alone gives an approximate 95% interval of 79,600–93,800 LOC, conditional on the sample judgments. The **77,000–97,000 LOC practical range** is broader to allow for semantic boundaries; it is not a separate statistical confidence interval. The range applies to calculation logic before adding the 3,910 LOC of reference tables.

The previous report's 30,000 LOC estimate kept its arithmetic and treatment rows from a full classification and sampled only the mixed buckets; this estimate samples the whole tax-processing category. Both use the same definition, but most of the growth reflects added tax processing: that category grew from 63,911 to 144,715 lines.

The 650 reviewed samples (400 tax-processing lines, 150 interview/page-mapping lines and 100 lines of the largest function) record their source locations, responsibilities, calculation decisions and reasons. They are kept with the development history and are not part of this repository. Examples of the boundary include:

- In [federal_rules.rs](src/federal/federal_rules.rs), the amount and line arguments of a form-line call are calculation work; its input-reference arrays, IRS source URLs and explanation messages are supporting output.
- In [interview/mod.rs](src/federal/interview/mod.rs), an amount accessor that excludes IRA-basis payments from a community-property allocation input is calculation work; conversion of editable 1099-R codes into stored records is outside it.
- In [unknowns.rs](src/core/unknowns.rs), totals recomputed after unsupported activities are dropped remain scope handling, not calculation.

## Largest tax-processing modules

| Module | Tax-processing code LOC | Functions |
| --- | ---: | ---: |
| [federal_rules.rs](src/federal/federal_rules.rs) | 25,536 | 152 |
| [extended_rules.rs](src/federal/extended_rules.rs) | 6,849 | 95 |
| [general_business_credit.rs](src/credits/general_business_credit.rs) | 3,607 | 121 |
| [asset_sale.rs](src/investment/asset_sale.rs) | 2,827 | 75 |
| [retirement.rs](src/retirement/retirement.rs) | 2,753 | 64 |
| [unknowns.rs](src/core/unknowns.rs) | 2,469 | 46 |
| [section1231.rs](src/investment/section1231.rs) | 2,276 | 46 |
| [k1_activity.rs](src/passthrough/k1_activity.rs) | 2,195 | 78 |
| [owners.rs](src/attribution/injured_spouse/owners.rs) | 2,055 | 12 |
| [estate_trust.rs](src/passthrough/estate_trust.rs) | 1,965 | 61 |
| [farm_specials.rs](src/farm/farm_specials.rs) | 1,894 | 65 |
| [form_1116.rs](src/foreign/form_1116.rs) | 1,813 | 47 |

These module counts include only their tax-processing portion. Their type declarations remain in the separate schema category.

## Largest mixed functions

| Function | Module | Starting source line | Code LOC |
| --- | ---: | ---: | ---: |
| `estimate_with_ira_pass` | [federal_rules.rs](src/federal/federal_rules.rs) | 12,786 | 14,854 |
| `collect` | [federal_rules.rs](src/federal/federal_rules.rs) | 4,861 | 5,401 |
| `rentals_with_prepared_sources` | [extended_rules.rs](src/federal/extended_rules.rs) | 1,747 | 3,905 |
| `project_sources` | [owners.rs](src/attribution/injured_spouse/owners.rs) | 196 | 1,944 |
| `calculate` | [hsa.rs](src/health/hsa.rs) | 702 | 1,092 |
| `calculate` | [claim_repayment.rs](src/deductions/claim_repayment.rs) | 881 | 854 |
| `calculate_with_section179` | [archer_contributions.rs](src/health/archer_contributions.rs) | 833 | 747 |
| `calculate` | [general_business_credit.rs](src/credits/general_business_credit.rs) | 4,233 | 730 |

`estimate_with_ira_pass` and `collect` together contain 20,255 code lines, or 14.0% of all tax-processing code. They account for 79.3% of the tax-processing portion of federal_rules.rs.

The largest function illustrates the mixed responsibilities. The 14,854 lines of `estimate_with_ira_pass` are estimated from a separate random sample of 100 of its lines:

| Responsibility | Lines in sample | Estimated code LOC | Function LOC share |
| --- | ---: | ---: | ---: |
| Input and consistency validation | 4 | 600 | 4% |
| Tax eligibility, treatment and limits | 8 | 1,200 | 8% |
| Arithmetic and amount aggregation | 18 | 2,700 | 18% |
| Result lines, explanations and provenance | 30 | 4,500 | 30% |
| Scope, omissions and conservative assumptions | 17 | 2,500 | 17% |
| Coordination, data preparation and accessors | 23 | 3,400 | 23% |
| **Total** | **100** | **14,900** | **100%** |

With 100 sampled lines, each share carries an approximate 95% interval of up to ±9 percentage points. Result output remains this function's largest estimated responsibility, while its behavior coordinates a complete estimate; 63 of the 100 sampled lines were return-calculation work.

## Libertas platform integration

The endpoint and storage boundary separates into the following categories:

| Category | Code LOC | Functions |
| --- | ---: | ---: |
| Libertas platform integration | 126 | 9 |
| App cache and return persistence | 1,236 | 44 |
| Crate wiring, reexports and macros | 1,226 | 0 |
| **Total** | **2,588** | **53** |

**Libertas platform integration** covers endpoint registration, callback adaptation, correlated response/EditRequest delivery and the concrete Data API adapter. It comprises 50 lines in lib.rs and 76 in storage.rs:

- `handle_request` and `tax_interview` in [lib.rs](src/lib.rs).
- `exists`, `get`, `records`, `put` and `delete` of the Data API transaction adapter, and `load_or_initialize` and `persist` of the `DataStore` adapter, in [storage.rs](src/core/storage.rs).

**App cache and return persistence** covers the Returns implementation in storage.rs (LRU caching, loading, responding and saving accepted changes; 224 lines and 13 functions) and the per-record return layout: header, section and household/income record encoding and the frozen format-124 migration in `src/core` (1,012 lines and 31 functions).

**Crate wiring, reexports and macros** accounts for the remaining 1,226 lines in lib.rs. Module declarations, public reexports and the crate's declarative macros make up this bucket.

This split measures the explicit endpoint and storage boundary. SDK-dependent schema declarations, workflow code and formatted-output helpers retain their functional categories elsewhere in the app.

## Snapshot and interpretation

These are source metrics as of 2026-10-20 03:16:18 UTC, development Git revision `5c31a8a`. They are not release metadata or a runtime benchmark. Later source edits can change the counts and classifications.

Total, category, area, module and function counts are source measurements under the counting rules above. Responsibility assignments and the return-calculation estimate are sample-based estimates: a statement can simultaneously enforce eligibility, compute an amount and explain the result, so it is assigned to its main purpose rather than counted repeatedly.
