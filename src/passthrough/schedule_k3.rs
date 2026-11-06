// Coverage: Schedule K-3 (1065 / 1120S) — Supported for Parts I–III (2026-10-09; 2026
//   drafts f1065sk3 and f1120sk3 of June 2026, i1065sk3 and i1120sk3 2025 final (no 2026 revision),
//   i1116 2026 draft of 9/30/26, Schedule B (Form 1116) Rev. 12-2026 draft, Pub 514, IRC 702(a)(6),
//   705(a)(2), 901, 904(a), (c), (d), (f), (j), 905(a), 909, 1366(a)(1)(A), 1367(a)(2), 1373(a), 26 CFR
//   1.861-8(e)(15), 1.861-9T(d)(1), 1.904-4(c)(3), (n)(1)(ii)). Only the parts a partner's or
//   shareholder's Form 1116 reads are modelled.
//   The printed ScheduleK3Paper belongs to its K-1 statements; the native projection
//   is PartnerK1/ScorpK1.schedule_k3. Its header and supported Part III fields,
//   Part II Section 1 and Part II Section 2 have independent AI Import actions.
//   Each action preserves the other sections and the K-1. Removing the K-1 removes it.
//   Part II line 24 (gross income) by country in the passive (c) and general (d) columns is Form
//   1116 line 1a of that category (rows `countries`, one `ForeignWithholding` answer per passive
//   row, grouped with the 1099 passive sources before the high-tax kickout); columns (a) and (f)
//   (`us_gross`) are U.S. source for a U.S. resident and enter line 3e with the rows in place of
//   the K-1's net amounts (CreditBasis.k3_correction: K-3 gross income less the K-1's Schedule E
//   Part II amount and box 5/6a (1120-S boxes 4/5a)). Section 2 deductions of columns (c)/(d) are
//   Form 1116 line 2 of the category (lines 27-30 are `K3Capital`: Form 1116 line 1a through
//   foreign_capital Worksheets A/B, and the Line 2 and Line 15 worksheets); column (f) deductions other than research, charity
//   and interest are apportioned by gross income on line 3b; interest of lines 39-43 in (c)/(d) is
//   already apportioned (line 4b) and column (f) business (line 41) and passive activity (line 43)
//   interest is apportioned by the Part III Section 2 line 6b/6d assets pooled over the K-3s only
//   (inventor decision 2026-10-09; the small-foreign-income election assigns all interest to U.S.
//   income). Part III Section 4 line 1 taxes by country and category are Form 1116 Part II line 8,
//   line 2 reductions (including section 909 splitter taxes) are line 12 spread over the category's
//   rows by income and floor each row at zero, and unused taxes are Schedule B line 6 through the
//   existing carryover and carryback code. Under the section 904(j) payee-statement election a K-3
//   with only passive rows is a payee statement. A K-1 box 21 / box 16 code F tax without a K-3 is
//   never credited (it reduces basis and counts toward the $300/$600 test) and raises
//   WARN_K1_FOREIGN_TAX_WITHOUT_K3. Part I box 2 (OilAndGas), 10 (Loans), Part II columns (b)/(e),
//   lines 11-15/27-30 in (c)/(d) and line 32 in (f) are `K3Item` scope entries that drop the claim
//   with WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED; Part I box 6 and the CFC/PFIC parts VI and VII
//   (1120-S V and VI) refuse the estimate (ISSUE_PARTNER_K1_INTERNATIONAL and
//   ISSUE_SCORP_K1_INTERNATIONAL). Excluded by name: item E answers other than those, Part I boxes
//   1, 3, 4, 5, 7, 8, 9, 11, 12, 13 (derived or information only), Part II column (a) deductions
//   and line 38 charity (U.S. source), Part III Section 1 (research apportionment, column (f)
//   Research is a scope entry), Section 2 lines 1-5, 6a, 6c, 7 and 8, Section 3 (1065 FDDEI), line 3
//   redeterminations (Schedule C, Form 7204, amended return) and Section 5 (section 743(b)); Parts
//   IV (section 250 deduction of domestic corporations), V (distributions from foreign
//   corporations, section 959 PTEP accounts that exist only after the refused section 951/1293
//   inclusions), VIII (section 960 deemed-paid credit of corporations and section 962 electors), IX
//   (BEAT), X (foreign partners' character and source), XI (section 871(m) covered partnerships),
//   XII (qualified derivatives dealers) and XIII (foreign partners' section 864(c)(8) deemed
//   sale) of Form 1065, and Parts IV to VII of Form 1120-S, none of which has an amount on an
//   individual U.S. resident's Form 1116 or Schedule B. Amended K-3s replace the original.
//   Refused by name with the K-1 (ISSUE_FTC_PASS_THROUGH, narrowed): a K-1 with a Schedule E Part
//   II amount and no K-3, a K-1 whose loss or deductions are limited this year (the K-3 is
//   pre-limitation), an estate or trust K-1 business or rental amount, S corporation loan gain;
//   ISSUE_FTC_K3_INTEREST: column (f) interest without line 6b/6d assets.
//   Evidence: schedule_k3.rs, extended_rules.rs (foreign_credit), federal_rules.rs (CreditBasis),
//   unknowns.rs (credits, income), k1_page.rs, partnership.rs, scorp.rs; tests/foreign/foreign.rs
//   k3_passive_tax_with_a_schedule_k3_matches_hand_computation,
//   k3_general_category_with_apportioned_interest_matches_hand_computation,
//   k1_foreign_tax_without_a_k3_is_never_credited, tests/passthrough/interview.rs k3_ai_import_*.
#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Schedule K-3
/// The international schedule a partnership or S corporation attaches to your K-1. The foreign tax credit (Form 1116) uses its Parts II and III.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this Schedule K-3 (Form 1065 or Form 1120-S) and its attached statements exactly. It belongs to the supplied year only when the entity's tax year ends in that year (a calendar-year schedule for that year, or a fiscal year ending in it); report a mismatch as an error. Report illegible or conflicting values as errors, and never infer a value the schedule does not show. A schedule printed on several pages is one form; copies of the same schedule count once. Amounts are U.S. dollars; parentheses or a minus sign mark a negative amount. `countries` has one element for each country code with an amount in Part II line 24 column (b), category ForeignBranch, column (c), category Passive, or column (d), category General: `country` is the two-letter code as printed, `income` that line 24 amount, `qualified` the Part II line 8 amount for the same country and column (0 when none), and `tax` the sum, for that country, of every row of the foreign-tax section line 1 (Form 1065 Part III Section 4, Form 1120-S Part III Section 3) in the same category, column (c) foreign branch, (d) passive or (e) general, adding its U.S., Foreign and Partner amounts. `us_gross` is Part II line 24 column (a) plus column (f). `branch` reads column (b), `passive` column (c) and `general` column (d): `deductions` is the sum of Part II lines 25, 26, 31 through 37 and 44 through 50, `interest` the sum of lines 39 through 43, `reduction` the total of the foreign-tax section's line 2 in the matching tax column, and `capital` the column's line 11 as `short_gain`, line 12 as `long_gain`, line 27 as `short_loss` and line 28 as `long_loss`, each a positive amount. `partner_deductions` is the sum of Part II lines 25, 26, 31, 33 through 37 and 44 through 50 in column (f). `partner_interest`: `business` is line 41 column (f), `passive_activity` line 43 column (f); `business_assets` is Part III Section 2 line 6b and `passive_assets` line 6d, each with `us` columns (a) plus (f), `branch` column (b), `passive` column (c) and `general` column (d), all zero when Section 2 is blank. Return zero for a clearly blank amount. `items` lists each entry at most once: OtherCategories for any amount in Part II column (e) or in the foreign-tax section's section 951A or other-category columns; ForeignRateGroups for any amount in Part II lines 13, 14, 15, 29 or 30 in column (b), (c) or (d); Research for line 32 column (f); OilAndGas when Part I box 2 is checked; Loans when Part I box 10 is checked; DisallowedDeduction when Part I box 6 is checked; ForeignCorporations when item E answers Yes for Part VI or VII of Form 1065, or Part V or VI of Form 1120-S. The imported `countries` and `items` lists replace the existing lists. Do not read Part I boxes 1, 3, 4, 5, 7, 8, 9, 11, 12 or 13, Part III Sections 1, 3 and 5 of Form 1065 or Section 1 of Form 1120-S, the foreign-tax section's line 3, or Parts IV, V and VIII through XIII; each passive country's withholding treatment comes from the person's own records.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct ScheduleK3 {
    /// Foreign income and taxes by country
    /// One row for each country and category in Part II line 24, with that country's foreign taxes from Part III. The foreign withholding treatment of a passive country is its own answer below. Do not repeat these amounts among the foreign tax credit's own source records.
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Country
    pub countries: Vec<K3Country>,
    /// Foreign withholding treatment of the passive countries
    /// One answer for each country that has a passive row above, from your own records of how the tax was withheld; the Schedule K-3 import never reads it, so an imported schedule keeps your answers for the countries it still lists. Compare the withholding with the row's income; keep a net-basis tax (OTH, OTHS, OTHR) or no tax under "No foreign withholding tax".
    #[libertas_unordered]
    /// ----
    /// Country
    pub withholding: Vec<K3Withholding>,
    /// U.S. and partner-sourced gross income — Part II line 24, columns (a) and (f)
    /// Column (f) income is U.S. income for a U.S. citizen or resident whose tax home is in the United States.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub us_gross: i64,
    /// Foreign branch category — column (b)
    #[libertas_llm_doc_field]
    pub branch: K3CategoryAmounts,
    /// Passive category — column (c)
    #[libertas_llm_doc_field]
    pub passive: K3CategoryAmounts,
    /// General category — column (d)
    #[libertas_llm_doc_field]
    pub general: K3CategoryAmounts,
    /// Deductions left for you to allocate — Part II column (f)
    /// Lines 25, 26, 31, 33 through 37 and 44 through 50. Not research expenses (line 32), charitable contributions (line 38) or interest (lines 39 through 43).
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub partner_deductions: i64,
    /// Interest expense left for you to apportion
    #[libertas_llm_doc_field]
    pub partner_interest: K3PartnerInterest,
    /// Other entries on this Schedule K-3
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Entry
    pub items: Vec<K3Item>,
}

/// Country
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct K3Country {
    /// Country code
    /// The two-letter code printed on the row, for example JA for Japan or CA for Canada.
    #[libertas_llm_doc_field]
    // Legal source identifier: Schedule K-3 uses two-letter country codes.
    #[libertas_size(min = 2, max = 2)]
    pub country: String,
    /// Category
    #[libertas_llm_doc_field]
    pub category: K3Category,
    /// Gross income — Part II line 24
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub income: i64,
    /// Qualified dividends included — Part II line 8
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Foreign taxes — Part III, foreign taxes line 1
    /// Every type of tax for this country in this category, paid or accrued.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax: i64,
}

/// Withholding treatment of a passive country
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K3Withholding {
    /// Country code
    /// The two-letter code of a country with a passive row above.
    // Legal source identifier: Schedule K-3 uses two-letter country codes.
    #[libertas_size(min = 2, max = 2)]
    pub country: String,
    /// Foreign withholding treatment on this country's taxes
    /// Withholding taxes carry type codes beginning with WHT. Compare the withholding with the country's passive income; keep a net-basis tax (OTH, OTHS, OTHR) or no tax under "No foreign withholding tax".
    pub treatment: crate::ForeignWithholding,
}

/// Category
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum K3Category {
    /// Passive category — column (c)
    Passive,
    /// General category — column (d)
    General,
    /// Foreign branch category — column (b)
    ForeignBranch,
}

/// Category amounts
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3CategoryAmounts {
    /// Deductions — Part II lines 25, 26, 31 through 37 and 44 through 50
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deductions: i64,
    /// Interest expense — Part II lines 39 through 43
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Reduction of taxes — Part III, foreign taxes line 2
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub reduction: i64,
    /// Capital gains and losses — Part II lines 11, 12, 27 and 28
    #[libertas_llm_doc_field]
    pub capital: K3Capital,
}

/// Capital gains and losses
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3Capital {
    /// Net short-term capital gain — line 11
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub short_gain: i64,
    /// Net long-term capital gain — line 12
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub long_gain: i64,
    /// Net short-term capital loss — line 27
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub short_loss: i64,
    /// Net long-term capital loss — line 28
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub long_loss: i64,
}

/// Interest expense left for you to apportion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K3PartnerInterest {
    /// Business interest — Part II line 41, column (f)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub business: i64,
    /// Passive activity interest — Part II line 43, column (f)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub passive_activity: i64,
    /// Assets attracting business interest — Part III Section 2 line 6b
    #[libertas_llm_doc_field]
    pub business_assets: K3Assets,
    /// Assets attracting passive activity interest — Part III Section 2 line 6d
    #[libertas_llm_doc_field]
    pub passive_assets: K3Assets,
}

/// Assets
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3Assets {
    /// U.S. and partner-sourced — columns (a) and (f)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub us: i64,
    /// Foreign branch category — column (b)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub branch: i64,
    /// Passive category — column (c)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub passive: i64,
    /// General category — column (d)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub general: i64,
}

/// Other entry on this Schedule K-3
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum K3Item {
    /// Section 951A or other-category amounts
    /// Part II column (e), or the section 951A or other-category columns of the foreign taxes in Part III.
    OtherCategories,
    /// Foreign 28%, unrecaptured section 1250 or section 1231 amounts — Part II lines 13, 14, 15, 29 or 30, column (b), (c) or (d)
    ForeignRateGroups,
    /// Research and experimental expenses for you to apportion — Part II line 32, column (f)
    Research,
    /// Foreign oil and gas taxes — Part I box 2
    OilAndGas,
    /// Partner or shareholder loan transactions — Part I box 10
    Loans,
    /// Section 267A disallowed deduction — Part I box 6
    DisallowedDeduction,
    /// Controlled foreign corporation or PFIC information
    /// Form 1065 Part VI or VII, or Form 1120-S Part V or VI, applies (item E).
    ForeignCorporations,
}

/// Schedule K-3
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum K3Attached {
    /// Not attached
    /// The K-1 box "Schedule K-3 is attached" is not checked.
    NotAttached,
    /// Attached
    Attached(crate::ScheduleK3),
}

// Printed layout: IRS Schedule K-3 (Forms 1065 and 1120-S), 2026 drafts
// created June 10, 2026. Only the portions read by the existing Form 1116
// calculation are collected; the other parts remain outside its coverage.
/// Schedule K-3
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ScheduleK3Paper {
    /// Schedule K-3 (Form 1065)
    Partnership(K3PartnershipPaper),
    /// Schedule K-3 (Form 1120-S)
    Scorp(K3ScorpPaper),
}
/// Partner’s Share of Income, Deductions, Credits, etc.—International
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the printed Schedule K-3 (Form 1065), retaining its year, parties, header checkboxes, Part I, the Part II category code and the supported Part III fields in their printed order. Part II Gross Income and Deductions each have their own import action; preserve their existing values and import those sections there. Copy beginning and ending dates only when printed; return null when blank, and never infer dates from the calendar year. Copy each line and column separately, including country codes and attached continuation rows. A total line and its country subrows are different printed rows: put the total in total and the lettered country rows in countries. In each country row, record one cell for each printed numeric column, including printed zero; omit blank cells. Use the column exactly as printed, without classifying it. A blank cell reads zero. Never add, classify or allocate amounts. Foreign taxes are Part III Section 4. Copy line 1's Paid and Accrued checkboxes and each row's type of tax and country. Copy reduction line 2's printed total, without adding its detail rows again. Clearly blank numeric boxes are zero; illegible or conflicting values are errors. Import neither a withholding-treatment answer nor a calculated foreign-credit summary. Do not read the excluded parts or expand tax coverage.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct K3PartnershipPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub year: i32,
    /// Tax year beginning
    /// [NullText]
    /// No beginning date printed
    /// [SetText]
    /// The printed beginning date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub beginning: Option<u32>,
    /// Tax year ending
    /// [NullText]
    /// No ending date printed
    /// [SetText]
    /// The printed ending date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub ending: Option<u32>,
    /// A–B Partnership’s employer identification number (EIN), name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub entity: crate::PrintedParty,
    /// C Partner’s social security number (SSN) or taxpayer identification number (TIN); D Name, address, city, state, and ZIP code for partner entered in box C
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// E1 Does Part I apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_1: Option<bool>,
    /// E2 Does Part II apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_2: Option<bool>,
    /// E3 Does Part III apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_3: Option<bool>,
    /// E4 Does Part IV apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_4: Option<bool>,
    /// E5 Does Part V apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_5: Option<bool>,
    /// E6 Does Part VI apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_6: Option<bool>,
    /// E7 Does Part VII apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_7: Option<bool>,
    /// E8 Does Part VIII apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_8: Option<bool>,
    /// E9 Does Part IX apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_9: Option<bool>,
    /// E10 Does Part X apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_10: Option<bool>,
    /// E11 Does Part XI apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_11: Option<bool>,
    /// E12 Does Part XII apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_12: Option<bool>,
    /// E13 Does Part XIII apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_13: Option<bool>,
    /// F Amended K-3
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amended: bool,
    /// Part I — Other Current Year International Information
    #[libertas_llm_doc_field]
    pub part_i: K3PaperOtherInformation,
    /// Part II — Foreign Tax Credit Limitation
    #[libertas_llm_doc_field]
    pub part_ii: K3PaperPartII1065,
    /// Part III Section 2 — Interest Expense Apportionment Factors
    #[libertas_llm_doc_field]
    pub interest_assets: K3PaperInterestAssets,
    /// Part III Section 4 — Foreign Taxes
    #[libertas_llm_doc_field]
    pub foreign_taxes: K3PaperForeignTaxes,
}
/// Shareholder’s Share of Income, Deductions, Credits, etc.—International
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the printed Schedule K-3 (Form 1120-S), retaining its year, parties, header checkboxes, Part I, the Part II category code and the supported Part III fields in their printed order. Part II Gross Income and Deductions each have their own import action; preserve their existing values and import those sections there. Copy beginning and ending dates only when printed; return null when blank, and never infer dates from the calendar year. Copy each line and column separately, including country codes and attached continuation rows. A total line and its country subrows are different printed rows: put the total in total and the lettered country rows in countries. In each country row, record one cell for each printed numeric column, including printed zero; omit blank cells. Use the column exactly as printed, without classifying it. A blank cell reads zero. Never add, classify or allocate amounts. Foreign taxes are Part III Section 3. Copy line 1's Paid and Accrued checkboxes and each row's type of tax and country. Copy reduction line 2's printed total, without adding its detail rows again. Clearly blank numeric boxes are zero; illegible or conflicting values are errors. Import neither a withholding-treatment answer nor a calculated foreign-credit summary. Do not read the excluded parts or expand tax coverage.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct K3ScorpPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub year: i32,
    /// Tax year beginning
    /// [NullText]
    /// No beginning date printed
    /// [SetText]
    /// The printed beginning date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub beginning: Option<u32>,
    /// Tax year ending
    /// [NullText]
    /// No ending date printed
    /// [SetText]
    /// The printed ending date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub ending: Option<u32>,
    /// A–B Corporation’s employer identification number (EIN), name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub entity: crate::PrintedParty,
    /// C–D Shareholder’s identifying number, name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// E1 Does Part I apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_1: Option<bool>,
    /// E2 Does Part II apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_2: Option<bool>,
    /// E3 Does Part III apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_3: Option<bool>,
    /// E4 Does Part IV apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_4: Option<bool>,
    /// E5 Does Part V apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_5: Option<bool>,
    /// E6 Does Part VI apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_6: Option<bool>,
    /// E7 Does Part VII apply?
    /// [NullText]
    /// No answer printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub part_7: Option<bool>,
    /// F Amended K-3
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amended: bool,
    /// Part I — Other Current Year International Information
    #[libertas_llm_doc_field]
    pub part_i: K3PaperOtherInformation,
    /// Part II — Foreign Tax Credit Limitation
    #[libertas_llm_doc_field]
    pub part_ii: K3PaperPartII1120S,
    /// Part III Section 2 — Interest Expense Apportionment Factors
    #[libertas_llm_doc_field]
    pub interest_assets: K3PaperInterestAssets,
    /// Part III Section 3 — Foreign Taxes
    #[libertas_llm_doc_field]
    pub foreign_taxes: K3PaperForeignTaxes,
}
/// Part I — Other Current Year International Information
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperOtherInformation {
    /// 2 Foreign oil and gas taxes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2: bool,
    /// 6 Section 267A disallowed deduction
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6: bool,
    /// 10 Partner / shareholder loan transactions
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_10: bool,
}
/// Foreign Tax Credit Limitation — printed columns
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperColumns {
    /// (a) U.S. source
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub us: i64,
    /// (b) Foreign branch category income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub branch: i64,
    /// (c) Passive category income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub passive: i64,
    /// (d) General category income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub general: i64,
    /// (e) Other
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// (f) Sourced by partner / shareholder
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub sourced: i64,
    /// (g) Total
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub total: i64,
}
/// A lettered country row
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperCountryRow {
    /// Country code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub country: String,
    /// Printed columns
    #[libertas_llm_doc_field]
    pub amounts: Vec<K3PaperCountryCell>,
}
/// A printed numeric cell on a lettered country row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K3PaperCountryCell {
    /// Column
    #[libertas_llm_doc_field]
    #[libertas_unique(1)]
    pub column: K3PaperColumn,
    /// Amount
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
}
/// Printed column
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum K3PaperColumn {
    /// (a) U.S. source
    Us,
    /// (b) Foreign branch category income
    Branch,
    /// (c) Passive category income
    Passive,
    /// (d) General category income
    General,
    /// (e) Other
    Other,
    /// (f) Sourced by partner / shareholder
    Sourced,
    /// (g) Total
    Total,
}
impl K3PaperCountryCell {
    fn category(&self) -> Option<K3Category> {
        let Self { column, amount: _ } = self;
        match column {
            K3PaperColumn::Branch => Some(K3Category::ForeignBranch),
            K3PaperColumn::Passive => Some(K3Category::Passive),
            K3PaperColumn::General => Some(K3Category::General),
            K3PaperColumn::Us
            | K3PaperColumn::Other
            | K3PaperColumn::Sourced
            | K3PaperColumn::Total => None,
        }
    }
    fn amount(&self) -> i64 {
        self.amount
    }
    fn amount_mut(&mut self) -> &mut i64 {
        &mut self.amount
    }
}
impl K3PaperCountryRow {
    fn category(&self, category: K3Category) -> Result<i64, crate::validation::ValidationIssue> {
        k3_paper_sum(
            self.amounts
                .iter()
                .filter(|cell| cell.category() == Some(category))
                .map(K3PaperCountryCell::amount),
        )
    }
    fn other(&self) -> bool {
        self.amounts.iter().any(|cell| cell.column == K3PaperColumn::Other && cell.amount != 0)
    }
}
/// A gross-income line and its lettered country rows
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperIncomeLine {
    /// Total
    #[libertas_llm_doc_field]
    pub total: K3PaperColumns,
    /// A–C Country rows and continuations
    #[libertas_llm_doc_field]
    pub countries: Vec<K3PaperCountryRow>,
}
/// Printed columns on this row
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperSourcedColumns {
    /// (f) Sourced by partner / shareholder
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub sourced: i64,
    /// (g) Total
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub total: i64,
}
impl K3PaperSourcedColumns {
    fn columns(&self) -> K3PaperColumns {
        let Self { sourced, total } = self;
        K3PaperColumns {
            us: 0,
            branch: 0,
            passive: 0,
            general: 0,
            other: 0,
            sourced: *sourced,
            total: *total,
        }
    }
}
impl From<K3PaperColumns> for K3PaperSourcedColumns {
    fn from(columns: K3PaperColumns) -> Self {
        let K3PaperColumns { us: _, branch: _, passive: _, general: _, other: _, sourced, total } =
            columns;
        Self { sourced, total }
    }
}
/// Printed columns on this row
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperUsTotalColumns {
    /// (a) U.S. source
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub us: i64,
    /// (g) Total
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub total: i64,
}
impl K3PaperUsTotalColumns {
    fn columns(&self) -> K3PaperColumns {
        let Self { us, total } = self;
        K3PaperColumns {
            us: *us,
            branch: 0,
            passive: 0,
            general: 0,
            other: 0,
            sourced: 0,
            total: *total,
        }
    }
}
impl From<K3PaperColumns> for K3PaperUsTotalColumns {
    fn from(columns: K3PaperColumns) -> Self {
        let K3PaperColumns { us, branch: _, passive: _, general: _, other: _, sourced: _, total } =
            columns;
        Self { us, total }
    }
}
/// A–C SIC code and R&E expenses
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperResearchRow {
    /// SIC code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub sic_code: String,
    /// Printed columns
    #[libertas_llm_doc_field]
    pub amounts: K3PaperSourcedColumns,
}
/// Part II — Foreign Tax Credit Limitation
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct K3PaperPartII {
    /// (e) Other — category code
    pub other_category_code: String,
    /// 1 Sales
    pub line_1: K3PaperIncomeLine,
    /// 2 Gross income from performance of services
    pub line_2: K3PaperIncomeLine,
    /// 3 Gross rental real estate income
    pub line_3: K3PaperIncomeLine,
    /// 4 Other gross rental income
    pub line_4: K3PaperIncomeLine,
    /// 5 Guaranteed payments (Form 1065)
    pub line_5: K3PaperIncomeLine,
    /// 6 Interest income
    pub line_6: K3PaperIncomeLine,
    /// 7 Ordinary dividends (exclude amount on line 8)
    pub line_7: K3PaperIncomeLine,
    /// 8 Qualified dividends
    pub line_8: K3PaperIncomeLine,
    /// 10 Royalties and license fees
    pub line_10: K3PaperIncomeLine,
    /// 11 Net short-term capital gain
    pub line_11: K3PaperIncomeLine,
    /// 12 Net long-term capital gain
    pub line_12: K3PaperIncomeLine,
    /// 13 Collectibles (28%) gain
    pub line_13: K3PaperIncomeLine,
    /// 14 Unrecaptured section 1250 gain
    pub line_14: K3PaperIncomeLine,
    /// 15 Net section 1231 gain
    pub line_15: K3PaperIncomeLine,
    /// 16 Section 986(c) gain
    pub line_16: K3PaperIncomeLine,
    /// 17 Section 987 gain
    pub line_17: K3PaperIncomeLine,
    /// 18 Section 988 gain
    pub line_18: K3PaperIncomeLine,
    /// 19 Section 951(a) inclusions (Form 1120-S)
    pub line_19: K3PaperIncomeLine,
    /// 20 Other income (see instructions)
    pub line_20: K3PaperIncomeLine,
    /// 21 Section 951A(a) inclusions (Form 1120-S)
    pub line_21: K3PaperIncomeLine,
    /// 24 Total gross income (combine lines 1 through 23)
    pub line_24: K3PaperIncomeLine,
    /// 25 Expenses allocable to sales income
    pub line_25: K3PaperColumns,
    /// 26 Expenses allocable to gross income from performance of services
    pub line_26: K3PaperColumns,
    /// 27 Net short-term capital loss
    pub line_27: K3PaperColumns,
    /// 28 Net long-term capital loss
    pub line_28: K3PaperColumns,
    /// 29 Collectibles loss
    pub line_29: K3PaperColumns,
    /// 30 Net section 1231 loss
    pub line_30: K3PaperColumns,
    /// 31 Other losses
    pub line_31: K3PaperColumns,
    /// 32 Research & experimental (R&E) expenses
    pub line_32: Vec<K3PaperResearchRow>,
    /// 33 Allocable rental expenses—depreciation, depletion, and amortization
    pub line_33: K3PaperColumns,
    /// 34 Allocable rental expenses—other than depreciation, depletion, and amortization
    pub line_34: K3PaperColumns,
    /// 35 Allocable royalty and licensing expenses—depreciation, depletion, and amortization
    pub line_35: K3PaperColumns,
    /// 36 Allocable royalty and licensing expenses—other than depreciation, depletion, and amortization
    pub line_36: K3PaperColumns,
    /// 37 Depreciation not included on line 33 or line 35
    pub line_37: K3PaperColumns,
    /// 38 Charitable contributions
    pub line_38: K3PaperColumns,
    /// 39 Interest expense specifically allocable under Regulations section 1.861-10(e)
    pub line_39: K3PaperColumns,
    /// 40 Other interest expense specifically allocable under Regulations section 1.861-10T
    pub line_40: K3PaperColumns,
    /// 41 Other interest expense—business
    pub line_41: K3PaperColumns,
    /// 42 Other interest expense—investment
    pub line_42: K3PaperColumns,
    /// 43 Other interest expense—passive activity
    pub line_43: K3PaperColumns,
    /// 44 Section 59(e)(2) expenditures, excluding R&E expenses on line 32
    pub line_44: K3PaperColumns,
    /// 45 Foreign taxes not creditable but deductible
    pub line_45: K3PaperColumns,
    /// 46 Section 986(c) loss
    pub line_46: K3PaperColumns,
    /// 47 Section 987 loss
    pub line_47: K3PaperColumns,
    /// 48 Section 988 loss
    pub line_48: K3PaperColumns,
    /// 49 Other allocable deductions (see instructions)
    pub line_49: K3PaperColumns,
    /// 50 Other apportioned share of deductions (see instructions)
    pub line_50: K3PaperColumns,
    /// 54 Total deductions (combine lines 25 through 53)
    pub line_54: K3PaperColumns,
    /// 55 Net income (loss) (subtract line 54 from line 24)
    pub line_55: K3PaperColumns,
}
/// Part II — Foreign Tax Credit Limitation
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperPartII1065 {
    /// (e) Other — category code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub other_category_code: String,
    /// Section 1 — Gross Income
    pub income: K3PaperIncome1065,
    /// Section 2 — Deductions
    pub deductions: K3PaperDeductions1065,
}
/// Part II Section 1 — Gross Income
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read only Part II Section 1 of this printed Schedule K-3 (Form 1065), including all continuation pages for that section. A schedule printed on several pages is one form; duplicate copies count once. Copy every supported printed line and column separately, including printed signs and zeroes. A clearly blank numeric cell is zero. Never add, classify, allocate or derive amounts. Keep each total row separate from its lettered country rows. In a country row, retain the printed country code and one cell for each printed numeric column; omit blank country cells. Do not change another section, the K-3 header, the K-1, source identities, or personal withholding answers. Report illegible or conflicting facts and a supplied tax-year mismatch as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K3PaperIncome1065 {
    /// 1 Sales
    #[libertas_llm_doc_field]
    pub line_1: K3PaperIncomeLine,
    /// 2 Gross income from performance of services
    #[libertas_llm_doc_field]
    pub line_2: K3PaperIncomeLine,
    /// 3 Gross rental real estate income
    #[libertas_llm_doc_field]
    pub line_3: K3PaperIncomeLine,
    /// 4 Other gross rental income
    #[libertas_llm_doc_field]
    pub line_4: K3PaperIncomeLine,
    /// 5 Guaranteed payments
    #[libertas_llm_doc_field]
    pub line_5: K3PaperColumns,
    /// 6 Interest income
    #[libertas_llm_doc_field]
    pub line_6: K3PaperIncomeLine,
    /// 7 Ordinary dividends (exclude amount on line 8)
    #[libertas_llm_doc_field]
    pub line_7: K3PaperIncomeLine,
    /// 8 Qualified dividends
    #[libertas_llm_doc_field]
    pub line_8: K3PaperIncomeLine,
    /// 10 Royalties and license fees
    #[libertas_llm_doc_field]
    pub line_10: K3PaperIncomeLine,
    /// 11 Net short-term capital gain
    #[libertas_llm_doc_field]
    pub line_11: K3PaperIncomeLine,
    /// 12 Net long-term capital gain
    #[libertas_llm_doc_field]
    pub line_12: K3PaperIncomeLine,
    /// 13 Collectibles (28%) gain
    #[libertas_llm_doc_field]
    pub line_13: K3PaperIncomeLine,
    /// 14 Unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    pub line_14: K3PaperIncomeLine,
    /// 15 Net section 1231 gain
    #[libertas_llm_doc_field]
    pub line_15: K3PaperIncomeLine,
    /// 16 Section 986(c) gain
    #[libertas_llm_doc_field]
    pub line_16: K3PaperColumns,
    /// 17 Section 987 gain
    #[libertas_llm_doc_field]
    pub line_17: K3PaperColumns,
    /// 18 Section 988 gain
    #[libertas_llm_doc_field]
    pub line_18: K3PaperColumns,
    /// 20 Other income (see instructions)
    #[libertas_llm_doc_field]
    pub line_20: K3PaperIncomeLine,
    /// 24 Total gross income (combine lines 1 through 23)
    #[libertas_llm_doc_field]
    pub line_24: K3PaperIncomeLine,
}
/// Part II Section 2 — Deductions
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read only Part II Section 2 of this printed Schedule K-3 (Form 1065), including all continuation pages for that section. A schedule printed on several pages is one form; duplicate copies count once. Copy every supported printed line and column separately, including printed signs and zeroes. A clearly blank numeric cell is zero. Never add, classify, allocate or derive amounts. Read the printed totals on lines 54 and 55 without recomputing them. Do not change another section, the K-3 header, the K-1, source identities, or personal withholding answers. Report illegible or conflicting facts and a supplied tax-year mismatch as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K3PaperDeductions1065 {
    /// 25 Expenses allocable to sales income
    #[libertas_llm_doc_field]
    pub line_25: K3PaperColumns,
    /// 26 Expenses allocable to gross income from performance of services
    #[libertas_llm_doc_field]
    pub line_26: K3PaperColumns,
    /// 27 Net short-term capital loss
    #[libertas_llm_doc_field]
    pub line_27: K3PaperColumns,
    /// 28 Net long-term capital loss
    #[libertas_llm_doc_field]
    pub line_28: K3PaperColumns,
    /// 29 Collectibles loss
    #[libertas_llm_doc_field]
    pub line_29: K3PaperColumns,
    /// 30 Net section 1231 loss
    #[libertas_llm_doc_field]
    pub line_30: K3PaperColumns,
    /// 31 Other losses
    #[libertas_llm_doc_field]
    pub line_31: K3PaperColumns,
    /// 32 Research & experimental (R&E) expenses
    #[libertas_llm_doc_field]
    pub line_32: Vec<K3PaperResearchRow>,
    /// 33 Allocable rental expenses—depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_33: K3PaperColumns,
    /// 34 Allocable rental expenses—other than depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_34: K3PaperColumns,
    /// 35 Allocable royalty and licensing expenses—depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_35: K3PaperColumns,
    /// 36 Allocable royalty and licensing expenses—other than depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_36: K3PaperColumns,
    /// 37 Depreciation not included on line 33 or line 35
    #[libertas_llm_doc_field]
    pub line_37: K3PaperColumns,
    /// 38 Charitable contributions
    #[libertas_llm_doc_field]
    pub line_38: K3PaperUsTotalColumns,
    /// 39 Interest expense specifically allocable under Regulations section 1.861-10(e)
    #[libertas_llm_doc_field]
    pub line_39: K3PaperColumns,
    /// 40 Other interest expense specifically allocable under Regulations section 1.861-10T
    #[libertas_llm_doc_field]
    pub line_40: K3PaperColumns,
    /// 41 Other interest expense—business
    #[libertas_llm_doc_field]
    pub line_41: K3PaperColumns,
    /// 42 Other interest expense—investment
    #[libertas_llm_doc_field]
    pub line_42: K3PaperColumns,
    /// 43 Other interest expense—passive activity
    #[libertas_llm_doc_field]
    pub line_43: K3PaperColumns,
    /// 44 Section 59(e)(2) expenditures, excluding R&E expenses on line 32
    #[libertas_llm_doc_field]
    pub line_44: K3PaperColumns,
    /// 45 Foreign taxes not creditable but deductible
    #[libertas_llm_doc_field]
    pub line_45: K3PaperColumns,
    /// 46 Section 986(c) loss
    #[libertas_llm_doc_field]
    pub line_46: K3PaperColumns,
    /// 47 Section 987 loss
    #[libertas_llm_doc_field]
    pub line_47: K3PaperColumns,
    /// 48 Section 988 loss
    #[libertas_llm_doc_field]
    pub line_48: K3PaperColumns,
    /// 49 Other allocable deductions (see instructions)
    #[libertas_llm_doc_field]
    pub line_49: K3PaperColumns,
    /// 50 Other apportioned share of deductions (see instructions)
    #[libertas_llm_doc_field]
    pub line_50: K3PaperColumns,
    /// 54 Total deductions (combine lines 25 through 53)
    #[libertas_llm_doc_field]
    pub line_54: K3PaperColumns,
    /// 55 Net income (loss) (subtract line 54 from line 24)
    #[libertas_llm_doc_field]
    pub line_55: K3PaperColumns,
}
impl From<&K3PaperPartII1065> for K3PaperPartII {
    fn from(paper: &K3PaperPartII1065) -> Self {
        let K3PaperPartII1065 {
            other_category_code,
            income:
                K3PaperIncome1065 {
                    line_1,
                    line_2,
                    line_3,
                    line_4,
                    line_5,
                    line_6,
                    line_7,
                    line_8,
                    line_10,
                    line_11,
                    line_12,
                    line_13,
                    line_14,
                    line_15,
                    line_16,
                    line_17,
                    line_18,
                    line_20,
                    line_24,
                },
            deductions:
                K3PaperDeductions1065 {
                    line_25,
                    line_26,
                    line_27,
                    line_28,
                    line_29,
                    line_30,
                    line_31,
                    line_32,
                    line_33,
                    line_34,
                    line_35,
                    line_36,
                    line_37,
                    line_38,
                    line_39,
                    line_40,
                    line_41,
                    line_42,
                    line_43,
                    line_44,
                    line_45,
                    line_46,
                    line_47,
                    line_48,
                    line_49,
                    line_50,
                    line_54,
                    line_55,
                },
        } = paper;
        Self {
            other_category_code: other_category_code.clone(),
            line_1: line_1.clone(),
            line_2: line_2.clone(),
            line_3: line_3.clone(),
            line_4: line_4.clone(),
            line_5: K3PaperIncomeLine { total: line_5.clone(), countries: Vec::new() },
            line_6: line_6.clone(),
            line_7: line_7.clone(),
            line_8: line_8.clone(),
            line_10: line_10.clone(),
            line_11: line_11.clone(),
            line_12: line_12.clone(),
            line_13: line_13.clone(),
            line_14: line_14.clone(),
            line_15: line_15.clone(),
            line_16: K3PaperIncomeLine { total: line_16.clone(), countries: Vec::new() },
            line_17: K3PaperIncomeLine { total: line_17.clone(), countries: Vec::new() },
            line_18: K3PaperIncomeLine { total: line_18.clone(), countries: Vec::new() },
            line_19: K3PaperIncomeLine::default(),
            line_20: line_20.clone(),
            line_21: K3PaperIncomeLine::default(),
            line_24: line_24.clone(),
            line_25: line_25.clone(),
            line_26: line_26.clone(),
            line_27: line_27.clone(),
            line_28: line_28.clone(),
            line_29: line_29.clone(),
            line_30: line_30.clone(),
            line_31: line_31.clone(),
            line_32: line_32.clone(),
            line_33: line_33.clone(),
            line_34: line_34.clone(),
            line_35: line_35.clone(),
            line_36: line_36.clone(),
            line_37: line_37.clone(),
            line_38: line_38.columns(),
            line_39: line_39.clone(),
            line_40: line_40.clone(),
            line_41: line_41.clone(),
            line_42: line_42.clone(),
            line_43: line_43.clone(),
            line_44: line_44.clone(),
            line_45: line_45.clone(),
            line_46: line_46.clone(),
            line_47: line_47.clone(),
            line_48: line_48.clone(),
            line_49: line_49.clone(),
            line_50: line_50.clone(),
            line_54: line_54.clone(),
            line_55: line_55.clone(),
        }
    }
}
impl From<K3PaperPartII> for K3PaperPartII1065 {
    fn from(paper: K3PaperPartII) -> Self {
        let K3PaperPartII {
            other_category_code,
            line_1,
            line_2,
            line_3,
            line_4,
            line_5,
            line_6,
            line_7,
            line_8,
            line_10,
            line_11,
            line_12,
            line_13,
            line_14,
            line_15,
            line_16,
            line_17,
            line_18,
            line_19: _,
            line_20,
            line_21: _,
            line_24,
            line_25,
            line_26,
            line_27,
            line_28,
            line_29,
            line_30,
            line_31,
            line_32,
            line_33,
            line_34,
            line_35,
            line_36,
            line_37,
            line_38,
            line_39,
            line_40,
            line_41,
            line_42,
            line_43,
            line_44,
            line_45,
            line_46,
            line_47,
            line_48,
            line_49,
            line_50,
            line_54,
            line_55,
        } = paper;
        Self {
            other_category_code,
            income: K3PaperIncome1065 {
                line_1,
                line_2,
                line_3,
                line_4,
                line_5: line_5.total,
                line_6,
                line_7,
                line_8,
                line_10,
                line_11,
                line_12,
                line_13,
                line_14,
                line_15,
                line_16: line_16.total,
                line_17: line_17.total,
                line_18: line_18.total,
                line_20,
                line_24,
            },
            deductions: K3PaperDeductions1065 {
                line_25,
                line_26,
                line_27,
                line_28,
                line_29,
                line_30,
                line_31,
                line_32,
                line_33,
                line_34,
                line_35,
                line_36,
                line_37,
                line_38: line_38.into(),
                line_39,
                line_40,
                line_41,
                line_42,
                line_43,
                line_44,
                line_45,
                line_46,
                line_47,
                line_48,
                line_49,
                line_50,
                line_54,
                line_55,
            },
        }
    }
}
/// Part II — Foreign Tax Credit Limitation
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperPartII1120S {
    /// (e) Other — category code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub other_category_code: String,
    /// Section 1 — Gross Income
    pub income: K3PaperIncome1120S,
    /// Section 2 — Deductions
    pub deductions: K3PaperDeductions1120S,
}
/// Part II Section 1 — Gross Income
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read only Part II Section 1 of this printed Schedule K-3 (Form 1120-S), including all continuation pages for that section. A schedule printed on several pages is one form; duplicate copies count once. Copy every supported printed line and column separately, including printed signs and zeroes. A clearly blank numeric cell is zero. Never add, classify, allocate or derive amounts. Keep each total row separate from its lettered country rows. In a country row, retain the printed country code and one cell for each printed numeric column; omit blank country cells. Do not change another section, the K-3 header, the K-1, source identities, or personal withholding answers. Report illegible or conflicting facts and a supplied tax-year mismatch as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K3PaperIncome1120S {
    /// 1 Sales
    #[libertas_llm_doc_field]
    pub line_1: K3PaperIncomeLine,
    /// 2 Gross income from performance of services
    #[libertas_llm_doc_field]
    pub line_2: K3PaperIncomeLine,
    /// 3 Gross rental real estate income
    #[libertas_llm_doc_field]
    pub line_3: K3PaperIncomeLine,
    /// 4 Other gross rental income
    #[libertas_llm_doc_field]
    pub line_4: K3PaperIncomeLine,
    /// 6 Interest income
    #[libertas_llm_doc_field]
    pub line_6: K3PaperIncomeLine,
    /// 7 Ordinary dividends (exclude amount on line 8)
    #[libertas_llm_doc_field]
    pub line_7: K3PaperIncomeLine,
    /// 8 Qualified dividends
    #[libertas_llm_doc_field]
    pub line_8: K3PaperIncomeLine,
    /// 10 Royalties and license fees
    #[libertas_llm_doc_field]
    pub line_10: K3PaperIncomeLine,
    /// 11 Net short-term capital gain
    #[libertas_llm_doc_field]
    pub line_11: K3PaperIncomeLine,
    /// 12 Net long-term capital gain
    #[libertas_llm_doc_field]
    pub line_12: K3PaperIncomeLine,
    /// 13 Collectibles (28%) gain
    #[libertas_llm_doc_field]
    pub line_13: K3PaperIncomeLine,
    /// 14 Unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    pub line_14: K3PaperIncomeLine,
    /// 15 Net section 1231 gain
    #[libertas_llm_doc_field]
    pub line_15: K3PaperIncomeLine,
    /// 16 Section 986(c) gain
    #[libertas_llm_doc_field]
    pub line_16: K3PaperColumns,
    /// 17 Section 987 gain
    #[libertas_llm_doc_field]
    pub line_17: K3PaperColumns,
    /// 18 Section 988 gain
    #[libertas_llm_doc_field]
    pub line_18: K3PaperColumns,
    /// 19 Section 951(a) inclusions
    #[libertas_llm_doc_field]
    pub line_19: K3PaperIncomeLine,
    /// 20 Other income (see instructions)
    #[libertas_llm_doc_field]
    pub line_20: K3PaperIncomeLine,
    /// 21 Section 951A(a) inclusions
    #[libertas_llm_doc_field]
    pub line_21: K3PaperColumns,
    /// 24 Total gross income (combine lines 1 through 23)
    #[libertas_llm_doc_field]
    pub line_24: K3PaperIncomeLine,
}
/// Part II Section 2 — Deductions
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read only Part II Section 2 of this printed Schedule K-3 (Form 1120-S), including all continuation pages for that section. A schedule printed on several pages is one form; duplicate copies count once. Copy every supported printed line and column separately, including printed signs and zeroes. A clearly blank numeric cell is zero. Never add, classify, allocate or derive amounts. Read the printed totals on lines 54 and 55 without recomputing them. Do not change another section, the K-3 header, the K-1, source identities, or personal withholding answers. Report illegible or conflicting facts and a supplied tax-year mismatch as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K3PaperDeductions1120S {
    /// 25 Expenses allocable to sales income
    #[libertas_llm_doc_field]
    pub line_25: K3PaperColumns,
    /// 26 Expenses allocable to gross income from performance of services
    #[libertas_llm_doc_field]
    pub line_26: K3PaperColumns,
    /// 27 Net short-term capital loss
    #[libertas_llm_doc_field]
    pub line_27: K3PaperColumns,
    /// 28 Net long-term capital loss
    #[libertas_llm_doc_field]
    pub line_28: K3PaperColumns,
    /// 29 Collectibles loss
    #[libertas_llm_doc_field]
    pub line_29: K3PaperColumns,
    /// 30 Net section 1231 loss
    #[libertas_llm_doc_field]
    pub line_30: K3PaperColumns,
    /// 31 Other losses
    #[libertas_llm_doc_field]
    pub line_31: K3PaperColumns,
    /// 32 Research & experimental (R&E) expenses
    #[libertas_llm_doc_field]
    pub line_32: Vec<K3PaperResearchRow>,
    /// 33 Allocable rental expenses—depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_33: K3PaperColumns,
    /// 34 Allocable rental expenses—other than depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_34: K3PaperColumns,
    /// 35 Allocable royalty and licensing expenses—depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_35: K3PaperColumns,
    /// 36 Allocable royalty and licensing expenses—other than depreciation, depletion, and amortization
    #[libertas_llm_doc_field]
    pub line_36: K3PaperColumns,
    /// 37 Depreciation not included on line 33 or line 35
    #[libertas_llm_doc_field]
    pub line_37: K3PaperColumns,
    /// 38 Charitable contributions
    #[libertas_llm_doc_field]
    pub line_38: K3PaperUsTotalColumns,
    /// 39 Interest expense specifically allocable under Regulations section 1.861-10(e)
    #[libertas_llm_doc_field]
    pub line_39: K3PaperColumns,
    /// 40 Other interest expense specifically allocable under Regulations section 1.861-10T
    #[libertas_llm_doc_field]
    pub line_40: K3PaperColumns,
    /// 41 Other interest expense—business
    #[libertas_llm_doc_field]
    pub line_41: K3PaperSourcedColumns,
    /// 42 Other interest expense—investment
    #[libertas_llm_doc_field]
    pub line_42: K3PaperSourcedColumns,
    /// 43 Other interest expense—passive activity
    #[libertas_llm_doc_field]
    pub line_43: K3PaperSourcedColumns,
    /// 44 Section 59(e)(2) expenditures, excluding R&E expenses on line 32
    #[libertas_llm_doc_field]
    pub line_44: K3PaperColumns,
    /// 45 Foreign taxes not creditable but deductible
    #[libertas_llm_doc_field]
    pub line_45: K3PaperColumns,
    /// 46 Section 986(c) loss
    #[libertas_llm_doc_field]
    pub line_46: K3PaperColumns,
    /// 47 Section 987 loss
    #[libertas_llm_doc_field]
    pub line_47: K3PaperColumns,
    /// 48 Section 988 loss
    #[libertas_llm_doc_field]
    pub line_48: K3PaperColumns,
    /// 49 Other allocable deductions (see instructions)
    #[libertas_llm_doc_field]
    pub line_49: K3PaperColumns,
    /// 50 Other apportioned share of deductions (see instructions)
    #[libertas_llm_doc_field]
    pub line_50: K3PaperColumns,
    /// 54 Total deductions (combine lines 25 through 53)
    #[libertas_llm_doc_field]
    pub line_54: K3PaperColumns,
    /// 55 Net income (loss) (subtract line 54 from line 24)
    #[libertas_llm_doc_field]
    pub line_55: K3PaperColumns,
}
impl From<&K3PaperPartII1120S> for K3PaperPartII {
    fn from(paper: &K3PaperPartII1120S) -> Self {
        let K3PaperPartII1120S {
            other_category_code,
            income:
                K3PaperIncome1120S {
                    line_1,
                    line_2,
                    line_3,
                    line_4,
                    line_6,
                    line_7,
                    line_8,
                    line_10,
                    line_11,
                    line_12,
                    line_13,
                    line_14,
                    line_15,
                    line_16,
                    line_17,
                    line_18,
                    line_19,
                    line_20,
                    line_21,
                    line_24,
                },
            deductions:
                K3PaperDeductions1120S {
                    line_25,
                    line_26,
                    line_27,
                    line_28,
                    line_29,
                    line_30,
                    line_31,
                    line_32,
                    line_33,
                    line_34,
                    line_35,
                    line_36,
                    line_37,
                    line_38,
                    line_39,
                    line_40,
                    line_41,
                    line_42,
                    line_43,
                    line_44,
                    line_45,
                    line_46,
                    line_47,
                    line_48,
                    line_49,
                    line_50,
                    line_54,
                    line_55,
                },
        } = paper;
        Self {
            other_category_code: other_category_code.clone(),
            line_1: line_1.clone(),
            line_2: line_2.clone(),
            line_3: line_3.clone(),
            line_4: line_4.clone(),
            line_5: K3PaperIncomeLine::default(),
            line_6: line_6.clone(),
            line_7: line_7.clone(),
            line_8: line_8.clone(),
            line_10: line_10.clone(),
            line_11: line_11.clone(),
            line_12: line_12.clone(),
            line_13: line_13.clone(),
            line_14: line_14.clone(),
            line_15: line_15.clone(),
            line_16: K3PaperIncomeLine { total: line_16.clone(), countries: Vec::new() },
            line_17: K3PaperIncomeLine { total: line_17.clone(), countries: Vec::new() },
            line_18: K3PaperIncomeLine { total: line_18.clone(), countries: Vec::new() },
            line_19: line_19.clone(),
            line_20: line_20.clone(),
            line_21: K3PaperIncomeLine { total: line_21.clone(), countries: Vec::new() },
            line_24: line_24.clone(),
            line_25: line_25.clone(),
            line_26: line_26.clone(),
            line_27: line_27.clone(),
            line_28: line_28.clone(),
            line_29: line_29.clone(),
            line_30: line_30.clone(),
            line_31: line_31.clone(),
            line_32: line_32.clone(),
            line_33: line_33.clone(),
            line_34: line_34.clone(),
            line_35: line_35.clone(),
            line_36: line_36.clone(),
            line_37: line_37.clone(),
            line_38: line_38.columns(),
            line_39: line_39.clone(),
            line_40: line_40.clone(),
            line_41: line_41.columns(),
            line_42: line_42.columns(),
            line_43: line_43.columns(),
            line_44: line_44.clone(),
            line_45: line_45.clone(),
            line_46: line_46.clone(),
            line_47: line_47.clone(),
            line_48: line_48.clone(),
            line_49: line_49.clone(),
            line_50: line_50.clone(),
            line_54: line_54.clone(),
            line_55: line_55.clone(),
        }
    }
}
impl From<K3PaperPartII> for K3PaperPartII1120S {
    fn from(paper: K3PaperPartII) -> Self {
        let K3PaperPartII {
            other_category_code,
            line_1,
            line_2,
            line_3,
            line_4,
            line_5: _,
            line_6,
            line_7,
            line_8,
            line_10,
            line_11,
            line_12,
            line_13,
            line_14,
            line_15,
            line_16,
            line_17,
            line_18,
            line_19,
            line_20,
            line_21,
            line_24,
            line_25,
            line_26,
            line_27,
            line_28,
            line_29,
            line_30,
            line_31,
            line_32,
            line_33,
            line_34,
            line_35,
            line_36,
            line_37,
            line_38,
            line_39,
            line_40,
            line_41,
            line_42,
            line_43,
            line_44,
            line_45,
            line_46,
            line_47,
            line_48,
            line_49,
            line_50,
            line_54,
            line_55,
        } = paper;
        Self {
            other_category_code,
            income: K3PaperIncome1120S {
                line_1,
                line_2,
                line_3,
                line_4,
                line_6,
                line_7,
                line_8,
                line_10,
                line_11,
                line_12,
                line_13,
                line_14,
                line_15,
                line_16: line_16.total,
                line_17: line_17.total,
                line_18: line_18.total,
                line_19,
                line_20,
                line_21: line_21.total,
                line_24,
            },
            deductions: K3PaperDeductions1120S {
                line_25,
                line_26,
                line_27,
                line_28,
                line_29,
                line_30,
                line_31,
                line_32,
                line_33,
                line_34,
                line_35,
                line_36,
                line_37,
                line_38: line_38.into(),
                line_39,
                line_40,
                line_41: line_41.into(),
                line_42: line_42.into(),
                line_43: line_43.into(),
                line_44,
                line_45,
                line_46,
                line_47,
                line_48,
                line_49,
                line_50,
                line_54,
                line_55,
            },
        }
    }
}
/// Part III Section 2 — Interest Expense Apportionment Factors
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperInterestAssets {
    /// (e) Other — category code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub other_category_code: String,
    /// (e) Other — country code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub other_country_code: String,
    /// 6b Assets attracting business interest expense
    #[libertas_llm_doc_field]
    pub line_6b: K3PaperColumns,
    /// 6d Assets attracting passive activity interest expense
    #[libertas_llm_doc_field]
    pub line_6d: K3PaperColumns,
}
/// Foreign taxes — printed U.S., Foreign and Partner / Shareholder columns
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperTaxSourceColumns {
    /// U.S.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub us: i64,
    /// Foreign
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign: i64,
    /// Partner / Shareholder
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub owner: i64,
}
/// Foreign taxes — printed category columns
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperTaxColumns {
    /// (b) Section 951A category income — U.S.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub section_951a_us: i64,
    /// (b) Section 951A category income — Foreign
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub section_951a_foreign: i64,
    /// (c) Foreign branch category income
    #[libertas_llm_doc_field]
    pub branch: K3PaperTaxSourceColumns,
    /// (d) Passive category income
    #[libertas_llm_doc_field]
    pub passive: K3PaperTaxSourceColumns,
    /// (e) General category income
    #[libertas_llm_doc_field]
    pub general: K3PaperTaxSourceColumns,
    /// (f) Other
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// (g) Total
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub total: i64,
}
/// Direct (section 901 or 903) foreign taxes — a printed lettered row
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperTaxRow {
    /// Country code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub country: String,
    /// (a) Type of tax
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub tax_type: String,
    /// Printed category columns
    #[libertas_llm_doc_field]
    pub amounts: K3PaperTaxColumns,
}
/// Part III — Foreign Taxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K3PaperForeignTaxes {
    /// (f) Other — category code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub other_category_code: String,
    /// 1 Direct (section 901 or 903) foreign taxes — Paid
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub paid: bool,
    /// 1 Direct (section 901 or 903) foreign taxes — Accrued
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub accrued: bool,
    /// 1 A–F Direct foreign taxes and continuations
    #[libertas_llm_doc_field]
    pub line_1: Vec<K3PaperTaxRow>,
    /// 2 Reduction of taxes (total)
    #[libertas_llm_doc_field]
    pub line_2: K3PaperTaxColumns,
}

impl ScheduleK3Paper {
    pub(crate) fn native(&self, withholding: &[K3Withholding]) -> ScheduleK3 {
        let mut native = ScheduleK3::from(self);
        native.withholding = withholding.to_vec();
        native
    }
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let variant = match self {
            Self::Partnership(_) => "Partnership",
            Self::Scorp(_) => "Scorp",
        };
        let native = match self.checked_native() {
            Ok(native) => native,
            Err(issue) => {
                let located = if issue.has_field_prefix(&["foreign_taxes"])
                    || issue.has_field_prefix(&["interest_assets"])
                {
                    issue
                } else {
                    issue.in_field("part_ii")
                };
                return Some(located.in_field(variant));
            }
        };
        let (_, income, _, _, _) = self.printed_parts();
        native.issue_with_mode(false).map(|issue| {
            if issue.key == "FOREIGN_QUALIFIED_ABOVE_INCOME" {
                let country = native
                    .countries
                    .iter()
                    .find(|row| row.qualified > row.income)
                    .expect("qualified income issue");
                let index = income
                    .line_8
                    .countries
                    .iter()
                    .position(|row| {
                        row.country == country.country
                            && row
                                .amounts
                                .iter()
                                .any(|cell| cell.category() == Some(country.category))
                    })
                    .expect("printed qualified country");
                let row = &income.line_8.countries[index];
                let cell = row
                    .amounts
                    .iter()
                    .position(|cell| cell.category() == Some(country.category))
                    .expect("printed qualified column");
                crate::validation::ValidationIssue::new(issue.key)
                    .at_field("amount")
                    .in_list("amounts", cell)
                    .in_list("countries", index)
                    .in_field("line_8")
                    .in_field("income")
                    .in_field("part_ii")
                    .in_field(variant)
            } else {
                issue.in_field(variant)
            }
        })
    }
    fn printed_parts(
        &self,
    ) -> (
        &K3PaperOtherInformation,
        alloc::borrow::Cow<'_, K3PaperPartII>,
        &K3PaperInterestAssets,
        &K3PaperForeignTaxes,
        bool,
    ) {
        match self {
            Self::Partnership(K3PartnershipPaper {
                year: _,
                beginning: _,
                ending: _,
                entity: _,
                recipient: _,
                part_1: _,
                part_2: _,
                part_3: _,
                part_4: _,
                part_5: _,
                part_6,
                part_7,
                part_8: _,
                part_9: _,
                part_10: _,
                part_11: _,
                part_12: _,
                part_13: _,
                amended: _,
                part_i,
                part_ii,
                interest_assets,
                foreign_taxes,
            }) => (
                part_i,
                alloc::borrow::Cow::Owned(K3PaperPartII::from(part_ii)),
                interest_assets,
                foreign_taxes,
                *part_6 == Some(true) || *part_7 == Some(true),
            ),
            Self::Scorp(K3ScorpPaper {
                year: _,
                beginning: _,
                ending: _,
                entity: _,
                recipient: _,
                part_1: _,
                part_2: _,
                part_3: _,
                part_4: _,
                part_5,
                part_6,
                part_7: _,
                amended: _,
                part_i,
                part_ii,
                interest_assets,
                foreign_taxes,
            }) => (
                part_i,
                alloc::borrow::Cow::Owned(K3PaperPartII::from(part_ii)),
                interest_assets,
                foreign_taxes,
                *part_5 == Some(true) || *part_6 == Some(true),
            ),
        }
    }
}
impl K3PaperColumns {
    fn category(&self, category: K3Category) -> i64 {
        let Self { us: _, branch, passive, general, other: _, sourced: _, total: _ } = self;
        match category {
            K3Category::ForeignBranch => *branch,
            K3Category::Passive => *passive,
            K3Category::General => *general,
        }
    }
    fn assets(&self) -> Result<K3Assets, crate::validation::ValidationIssue> {
        let Self { us, branch, passive, general, other: _, sourced, total: _ } = self;
        Ok(K3Assets {
            us: k3_paper_sum([*us, *sourced])?,
            branch: *branch,
            passive: *passive,
            general: *general,
        })
    }
}
impl K3PaperTaxSourceColumns {
    fn amount(&self) -> Result<i64, crate::validation::ValidationIssue> {
        let Self { us, foreign, owner } = self;
        k3_paper_sum([*us, *foreign, *owner])
    }
}
impl K3PaperTaxColumns {
    fn category(&self, category: K3Category) -> Result<i64, crate::validation::ValidationIssue> {
        let Self {
            section_951a_us: _,
            section_951a_foreign: _,
            branch,
            passive,
            general,
            other: _,
            total: _,
        } = self;
        match category {
            K3Category::ForeignBranch => branch,
            K3Category::Passive => passive,
            K3Category::General => general,
        }
        .amount()
    }
    fn other_categories(&self) -> bool {
        self.section_951a_us != 0 || self.section_951a_foreign != 0 || self.other != 0
    }
}
fn k3_paper_sum(
    values: impl IntoIterator<Item = i64>,
) -> Result<i64, crate::validation::ValidationIssue> {
    crate::kernel::ratio::checked_sum(values).ok_or_else(|| {
        crate::validation::ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_record()
    })
}
impl From<&ScheduleK3Paper> for ScheduleK3 {
    fn from(paper: &ScheduleK3Paper) -> Self {
        paper.checked_native().expect("admitted Schedule K-3 source amounts")
    }
}
impl ScheduleK3Paper {
    fn checked_native(&self) -> Result<ScheduleK3, crate::validation::ValidationIssue> {
        let paper = self;
        let (part_i, part_ii, assets, taxes, foreign_corporations) = paper.printed_parts();
        let K3PaperOtherInformation { box_2, box_6, box_10 } = part_i;
        let K3PaperPartII {
            other_category_code: _,
            line_1,
            line_2,
            line_3,
            line_4,
            line_5,
            line_6,
            line_7,
            line_8,
            line_10,
            line_11,
            line_12,
            line_13,
            line_14,
            line_15,
            line_16,
            line_17,
            line_18,
            line_19,
            line_20,
            line_21,
            line_24,
            line_25,
            line_26,
            line_27,
            line_28,
            line_29,
            line_30,
            line_31,
            line_32,
            line_33,
            line_34,
            line_35,
            line_36,
            line_37,
            line_38,
            line_39,
            line_40,
            line_41,
            line_42,
            line_43,
            line_44,
            line_45,
            line_46,
            line_47,
            line_48,
            line_49,
            line_50,
            line_54,
            line_55,
        } = part_ii.as_ref();
        let K3PaperInterestAssets {
            other_category_code: _,
            other_country_code: _,
            line_6b,
            line_6d,
        } = assets;
        let K3PaperForeignTaxes {
            other_category_code: _,
            paid: _,
            accrued: _,
            line_1: tax_rows,
            line_2: tax_reductions,
        } = taxes;
        let mut countries = Vec::new();
        for (index, row) in line_24.countries.iter().enumerate() {
            let K3PaperCountryRow { country, amounts: _ } = row;
            if line_24.countries[..index].iter().any(|earlier| earlier.country == *country) {
                continue;
            }
            for category in [K3Category::ForeignBranch, K3Category::Passive, K3Category::General] {
                if line_24
                    .countries
                    .iter()
                    .filter(|row| row.country == *country)
                    .any(|row| row.amounts.iter().any(|cell| cell.category() == Some(category)))
                {
                    let income = k3_paper_sum(
                        line_24
                            .countries
                            .iter()
                            .filter(|row| row.country == *country)
                            .map(|row| row.category(category))
                            .collect::<Result<Vec<_>, _>>()?,
                    )?;
                    let qualified = k3_paper_sum(
                        line_8
                            .countries
                            .iter()
                            .filter(|row| row.country == *country)
                            .map(|row| row.category(category))
                            .collect::<Result<Vec<_>, _>>()?,
                    )?;
                    let tax = k3_paper_sum(
                        tax_rows
                            .iter()
                            .filter(|row| row.country == *country)
                            .map(|row| row.amounts.category(category))
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|issue| issue.in_field("line_1").in_field("foreign_taxes"))?,
                    )
                    .map_err(|issue| issue.in_field("line_1").in_field("foreign_taxes"))?;
                    countries.push(K3Country {
                        country: country.clone(),
                        category,
                        income,
                        qualified,
                        tax,
                    });
                }
            }
        }
        let deductions = [
            line_25, line_26, line_31, line_33, line_34, line_35, line_36, line_37, line_44,
            line_45, line_46, line_47, line_48, line_49, line_50,
        ];
        let interest = [line_39, line_40, line_41, line_42, line_43];
        let category = |category| -> Result<K3CategoryAmounts, crate::validation::ValidationIssue> {
            Ok(K3CategoryAmounts {
                deductions: k3_paper_sum(deductions.iter().map(|line| line.category(category)))?,
                interest: k3_paper_sum(interest.iter().map(|line| line.category(category)))?,
                reduction: tax_reductions
                    .category(category)
                    .map_err(|issue| issue.in_field("line_2").in_field("foreign_taxes"))?,
                capital: K3Capital {
                    short_gain: line_11.total.category(category),
                    long_gain: line_12.total.category(category),
                    short_loss: line_27.category(category),
                    long_loss: line_28.category(category),
                },
            })
        };
        let mut items = Vec::new();
        if [
            line_1, line_2, line_3, line_4, line_5, line_6, line_7, line_8, line_10, line_11,
            line_12, line_13, line_14, line_15, line_16, line_17, line_18, line_19, line_20,
            line_21, line_24,
        ]
        .iter()
        .any(|line| line.total.other != 0 || line.countries.iter().any(K3PaperCountryRow::other))
            || [
                line_25, line_26, line_27, line_28, line_29, line_30, line_31, line_33, line_34,
                line_35, line_36, line_37, line_38, line_39, line_40, line_41, line_42, line_43,
                line_44, line_45, line_46, line_47, line_48, line_49, line_50, line_54, line_55,
            ]
            .iter()
            .any(|line| line.other != 0)
            || tax_rows.iter().any(|row| row.amounts.other_categories())
            || tax_reductions.other_categories()
        {
            items.push(K3Item::OtherCategories);
        }
        if [line_13, line_14, line_15].iter().any(|line| {
            [K3Category::ForeignBranch, K3Category::Passive, K3Category::General].iter().any(
                |category| {
                    line.total.category(*category) != 0
                        || line.countries.iter().any(|row| {
                            row.amounts.iter().any(|cell| {
                                cell.category() == Some(*category) && cell.amount() != 0
                            })
                        })
                },
            )
        }) || [line_29, line_30]
            .iter()
            .any(|line| line.branch != 0 || line.passive != 0 || line.general != 0)
        {
            items.push(K3Item::ForeignRateGroups);
        }
        if line_32.iter().any(|row| row.amounts.sourced != 0) {
            items.push(K3Item::Research);
        }
        if *box_2 {
            items.push(K3Item::OilAndGas);
        }
        if *box_10 {
            items.push(K3Item::Loans);
        }
        if *box_6 {
            items.push(K3Item::DisallowedDeduction);
        }
        if foreign_corporations {
            items.push(K3Item::ForeignCorporations);
        }
        Ok(ScheduleK3 {
            countries,
            withholding: Vec::new(),
            us_gross: k3_paper_sum([line_24.total.us, line_24.total.sourced])?,
            branch: category(K3Category::ForeignBranch)?,
            passive: category(K3Category::Passive)?,
            general: category(K3Category::General)?,
            partner_deductions: k3_paper_sum(
                [
                    line_25, line_26, line_31, line_33, line_34, line_35, line_36, line_37,
                    line_44, line_45, line_46, line_47, line_48, line_49, line_50,
                ]
                .iter()
                .map(|line| line.sourced),
            )?,
            partner_interest: K3PartnerInterest {
                business: line_41.sourced,
                passive_activity: line_43.sourced,
                business_assets: line_6b
                    .assets()
                    .map_err(|issue| issue.in_field("line_6b").in_field("interest_assets"))?,
                passive_assets: line_6d
                    .assets()
                    .map_err(|issue| issue.in_field("line_6d").in_field("interest_assets"))?,
            },
            items,
        })
    }
}

// A community working view divides the same aggregates the native K-3 owns.
// Prefix allocation among printed cells retains the existing aggregate rounding.
impl ScheduleK3Paper {
    pub(crate) fn divided(&self, share: crate::community::Share) -> Self {
        use crate::community::Divide;
        fn split<'a>(
            values: impl IntoIterator<Item = &'a mut i64>,
            share: crate::community::Share,
        ) {
            use crate::kernel::ratio::exact;
            let mut total = exact::whole(0);
            for value in values {
                let before = share.filer_exact(&total);
                total += exact::whole(*value);
                // filer_exact rounds each owner-first prefix in the wide carrier.
                let through = share.filer_exact(&total);
                let allocated = through - before;
                *value = exact::rounded(&allocated).expect("admitted K-3 allocation");
            }
        }
        fn tax(values: &mut K3PaperTaxSourceColumns, share: crate::community::Share) {
            split([&mut values.us, &mut values.foreign, &mut values.owner], share);
        }
        fn parts(
            income: &mut K3PaperPartII,
            assets: &mut K3PaperInterestAssets,
            taxes: &mut K3PaperForeignTaxes,
            share: crate::community::Share,
        ) {
            split([&mut income.line_24.total.us, &mut income.line_24.total.sourced], share);
            let columns: [fn(&mut K3PaperColumns) -> &mut i64; 3] =
                [|row| &mut row.branch, |row| &mut row.passive, |row| &mut row.general];
            for select in columns {
                split(
                    [
                        select(&mut income.line_25),
                        select(&mut income.line_26),
                        select(&mut income.line_31),
                        select(&mut income.line_33),
                        select(&mut income.line_34),
                        select(&mut income.line_35),
                        select(&mut income.line_36),
                        select(&mut income.line_37),
                        select(&mut income.line_44),
                        select(&mut income.line_45),
                        select(&mut income.line_46),
                        select(&mut income.line_47),
                        select(&mut income.line_48),
                        select(&mut income.line_49),
                        select(&mut income.line_50),
                    ],
                    share,
                );
                split(
                    [
                        select(&mut income.line_39),
                        select(&mut income.line_40),
                        select(&mut income.line_41),
                        select(&mut income.line_42),
                        select(&mut income.line_43),
                    ],
                    share,
                );
                *select(&mut income.line_11.total) =
                    select(&mut income.line_11.total).divided(share);
                *select(&mut income.line_12.total) =
                    select(&mut income.line_12.total).divided(share);
                *select(&mut income.line_27) = select(&mut income.line_27).divided(share);
                *select(&mut income.line_28) = select(&mut income.line_28).divided(share);
                *select(&mut assets.line_6b) = select(&mut assets.line_6b).divided(share);
                *select(&mut assets.line_6d) = select(&mut assets.line_6d).divided(share);
            }
            split(
                [
                    &mut income.line_25.sourced,
                    &mut income.line_26.sourced,
                    &mut income.line_31.sourced,
                    &mut income.line_33.sourced,
                    &mut income.line_34.sourced,
                    &mut income.line_35.sourced,
                    &mut income.line_36.sourced,
                    &mut income.line_37.sourced,
                    &mut income.line_44.sourced,
                    &mut income.line_45.sourced,
                    &mut income.line_46.sourced,
                    &mut income.line_47.sourced,
                    &mut income.line_48.sourced,
                    &mut income.line_49.sourced,
                    &mut income.line_50.sourced,
                ],
                share,
            );
            income.line_41.sourced = income.line_41.sourced.divided(share);
            income.line_43.sourced = income.line_43.sourced.divided(share);
            for row in [&mut assets.line_6b, &mut assets.line_6d] {
                split([&mut row.us, &mut row.sourced], share);
            }
            for amounts in
                [&mut taxes.line_2.branch, &mut taxes.line_2.passive, &mut taxes.line_2.general]
            {
                tax(amounts, share);
            }
            let mut countries = Vec::<String>::new();
            for row in &income.line_24.countries {
                if !countries.contains(&row.country) {
                    countries.push(row.country.clone());
                }
            }
            for country in countries {
                for category in
                    [K3Category::ForeignBranch, K3Category::Passive, K3Category::General]
                {
                    split(
                        income
                            .line_24
                            .countries
                            .iter_mut()
                            .filter(|row| row.country == country)
                            .flat_map(|row| row.amounts.iter_mut())
                            .filter(|cell| cell.category() == Some(category))
                            .map(K3PaperCountryCell::amount_mut),
                        share,
                    );
                    split(
                        income
                            .line_8
                            .countries
                            .iter_mut()
                            .filter(|row| row.country == country)
                            .flat_map(|row| row.amounts.iter_mut())
                            .filter(|cell| cell.category() == Some(category))
                            .map(K3PaperCountryCell::amount_mut),
                        share,
                    );
                }
                let categories: [fn(&mut K3PaperTaxColumns) -> &mut K3PaperTaxSourceColumns; 3] =
                    [|row| &mut row.branch, |row| &mut row.passive, |row| &mut row.general];
                for select in categories {
                    split(
                        taxes.line_1.iter_mut().filter(|row| row.country == country).flat_map(
                            |row| {
                                let amounts = select(&mut row.amounts);
                                [&mut amounts.us, &mut amounts.foreign, &mut amounts.owner]
                            },
                        ),
                        share,
                    );
                }
            }
        }
        let mut saved = self.clone();
        match &mut saved {
            Self::Partnership(paper) => {
                let mut income = K3PaperPartII::from(&paper.part_ii);
                parts(&mut income, &mut paper.interest_assets, &mut paper.foreign_taxes, share);
                paper.part_ii = income.into();
            }
            Self::Scorp(paper) => {
                let mut income = K3PaperPartII::from(&paper.part_ii);
                parts(&mut income, &mut paper.interest_assets, &mut paper.foreign_taxes, share);
                paper.part_ii = income.into();
            }
        }
        saved
    }
}

// Arithmetic safety: gross, foreign_tax and collect still sum native source
// cents into i64, including Form 1116 group aggregates. Widen with those owners.
const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;
const SIGNED: core::ops::RangeInclusive<i64> = -100_000_000_000..=100_000_000_000;

impl K3Country {
    fn passive(&self) -> bool {
        self.category == K3Category::Passive
    }
}

impl K3Capital {
    fn valid(&self) -> bool {
        [self.short_gain, self.long_gain, self.short_loss, self.long_loss]
            .iter()
            .all(|v| MONEY.contains(v))
    }

    fn lines(&self) -> [i64; 4] {
        [self.short_gain, self.long_gain, self.short_loss, self.long_loss]
    }
}

impl K3CategoryAmounts {
    fn valid(&self) -> bool {
        [self.deductions, self.interest, self.reduction].iter().all(|v| MONEY.contains(v))
            && self.capital.valid()
    }
}

impl K3Assets {
    fn valid(&self) -> bool {
        [self.us, self.branch, self.passive, self.general].iter().all(|v| MONEY.contains(v))
    }

    fn total(&self) -> i64 {
        self.us + self.branch + self.passive + self.general
    }
}

impl K3Item {
    // The items that make the K-3 outside the Form 1116 calculation: the
    // claim is not made (WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED).
    fn leaves_credit_out(self) -> bool {
        matches!(
            self,
            Self::OtherCategories
                | Self::ForeignRateGroups
                | Self::Research
                | Self::OilAndGas
                | Self::Loans
        )
    }

    // The items whose income or tax the App cannot compute: the estimate is
    // refused by name, in the key of the K-1 form that carries the K-3.
    pub(crate) fn refusal(self, scorp: bool) -> Option<(&'static str, &'static str)> {
        matches!(self, Self::DisallowedDeduction | Self::ForeignCorporations).then(|| {
            if scorp {
                (
                    "ISSUE_SCORP_K1_INTERNATIONAL",
                    crate::fixture_english!(
                        "Schedule K-3 (Form 1120-S) Parts V and VI (sections 951(a)(1) and 951A inclusions, passive foreign investment companies) and the Part I box 6 section 267A disallowed deduction are not calculated."
                    ),
                )
            } else {
                (
                    "ISSUE_PARTNER_K1_INTERNATIONAL",
                    crate::fixture_english!(
                        "Schedule K-3 (Form 1065) Parts VI and VII (sections 951(a)(1) and 951A inclusions, passive foreign investment companies) and the Part I box 6 section 267A disallowed deduction are not calculated."
                    ),
                )
            }
        })
    }
}

impl ScheduleK3 {
    // The Form 1116 grouping of a row: the general category is group 0 and a
    // passive row joins the withholding groups 1-4 beside the 1099 sources,
    // by the country's own withholding answer (none is group 3 or 4 by tax).
    pub(crate) fn group(&self, row: &K3Country) -> usize {
        if crate::year_constants::is_section_901j_country(&row.country) {
            return crate::form_1116::SANCTIONED;
        }
        match row.category {
            K3Category::General => 0,
            K3Category::ForeignBranch => crate::form_1116::BRANCH,
            K3Category::Passive => self
                .withholding
                .iter()
                .find(|answer| answer.country == row.country)
                .map_or(&crate::ForeignWithholding::None, |answer| &answer.treatment)
                .group(row.tax),
        }
    }

    // The schedule as saved: an answer for a country that has no passive row
    // is not owned by the current schedule and goes.
    pub(crate) fn pruned(&self) -> Self {
        let mut saved = self.clone();
        saved.withholding.retain(|answer| {
            self.countries.iter().any(|row| row.passive() && row.country == answer.country)
        });
        saved
    }

    // The first relationship that fails, by its client rule's key (the server
    // twin of the schema rules on this type and K3Country).
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some((index, _)) =
            self.items.iter().enumerate().find(|(index, item)| self.items[..*index].contains(item))
        {
            return Some(ValidationIssue::new("K3_ITEM_TWICE").at_list_item("items", index));
        }
        if let Some((index, _)) =
            self.countries.iter().enumerate().find(|(_, row)| row.qualified > row.income)
        {
            return Some(
                ValidationIssue::new("FOREIGN_QUALIFIED_ABOVE_INCOME")
                    .at_field("qualified")
                    .in_list("countries", index),
            );
        }
        if let Some((index, _)) = self.withholding.iter().enumerate().find(|(index, answer)| {
            self.withholding[..*index].iter().any(|earlier| earlier.country == answer.country)
        }) {
            return Some(
                ValidationIssue::new("K3_WITHHOLDING_COUNTRY_TWICE")
                    .at_field("country")
                    .in_list("withholding", index),
            );
        }
        if require_answers
            && let Some((index, _)) = self.countries.iter().enumerate().find(|(_, row)| {
                row.passive()
                    && !self.withholding.iter().any(|answer| answer.country == row.country)
            })
        {
            return Some(
                ValidationIssue::new("K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY")
                    .at_field("country")
                    .in_list("countries", index),
            );
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        self.valid_with_mode(true)
    }
    pub(crate) fn valid_with_mode(&self, require_answers: bool) -> bool {
        self.issue_with_mode(require_answers).is_none()
            // Arithmetic safety: gross/taxes, ratio::allocate_prefix total/cumulative
            // weights and foreign collect/Form 1116 groups retain native i64
            // receivers. Exact remaining allocation has different prefix rounding;
            // withholding is metadata and does not need this monetary bound.
            && self.countries.len() <= 40
            // Schedule K-3 uses two-character country identifiers.
            && self.withholding.iter().all(|answer| answer.country.len() == 2)
            && self.countries.iter().all(|row| {
                row.country.len() == 2
                    && [row.income, row.qualified, row.tax].iter().all(|v| MONEY.contains(v))
            })
            && SIGNED.contains(&self.us_gross)
            && self.branch.valid()
            && self.passive.valid()
            && self.general.valid()
            && MONEY.contains(&self.partner_deductions)
            && [self.partner_interest.business, self.partner_interest.passive_activity]
                .iter()
                .all(|v| MONEY.contains(v))
            && self.partner_interest.business_assets.valid()
            && self.partner_interest.passive_assets.valid()
    }

    // Interest the small-foreign-income election would assign to U.S. income.
    pub(crate) fn has_interest(&self) -> bool {
        self.branch.interest != 0
            || self.passive.interest != 0
            || self.general.interest != 0
            || self.partner_interest.business != 0
            || self.partner_interest.passive_activity != 0
    }

    // An item outside the Form 1116 calculation: the claim is dropped.
    pub(crate) fn credit_left_out(&self) -> bool {
        self.items.iter().any(|item| item.leaves_credit_out())
    }

    pub(crate) fn refusals(&self, scorp: bool) -> Vec<(&'static str, &'static str)> {
        let mut found: Vec<(&'static str, &'static str)> = Vec::new();
        for refusal in self.items.iter().filter_map(|item| item.refusal(scorp)) {
            if !found.contains(&refusal) {
                found.push(refusal);
            }
        }
        found
    }

    // Foreign taxes the K-3 shows, whole cents, before the line 2 reductions.
    pub(crate) fn taxes(&self) -> i64 {
        self.countries.iter().map(|row| row.tax).sum()
    }

    // Part II line 24 gross income of Form 1116 line 3e, whole cents:
    // columns (a) and (f) with the passive and general rows.
    pub(crate) fn gross(&self) -> i64 {
        self.us_gross + self.countries.iter().map(|row| row.income).sum::<i64>()
    }

    // A payee statement for the section 904(j) election: passive income
    // alone, with nothing outside the calculation.
    pub(crate) fn passive_only(&self) -> bool {
        self.items.is_empty()
            && self.passive.deductions == 0
            && self.branch == K3CategoryAmounts::default()
            && self.general == K3CategoryAmounts::default()
            && self.countries.iter().all(K3Country::passive)
    }
}

// The Schedule K-3 of every K-1 on the return.
// Keep the actual originating K-1 alongside its native community-adjusted
// schedule. Identity is provenance only; owner is the actual source answer.
pub(crate) struct AttachedK3<'a> {
    pub(crate) source: i64,
    pub(crate) owner: crate::Owner,
    pub(crate) schedule: alloc::borrow::Cow<'a, ScheduleK3>,
}
pub(crate) fn attached(d: &crate::FederalDraft) -> Vec<alloc::borrow::Cow<'_, ScheduleK3>> {
    attached_sources(d).into_iter().map(|entry| entry.schedule).collect()
}
pub(crate) fn attached_sources(d: &crate::FederalDraft) -> Vec<AttachedK3<'_>> {
    d.income
        .iter()
        .filter_map(|entry| {
            let schedule = match crate::community::record(d, entry) {
                alloc::borrow::Cow::Borrowed(income) => match income {
                    crate::FederalIncome::Partnership { data } => {
                        data.schedule_k3.as_ref().map(alloc::borrow::Cow::Borrowed)
                    }
                    crate::FederalIncome::Scorp { data } => {
                        crate::ScorpK1::from(data).schedule_k3.map(alloc::borrow::Cow::Owned)
                    }
                    _ => None,
                },
                alloc::borrow::Cow::Owned(income) => match income {
                    crate::FederalIncome::Partnership { data } => {
                        data.schedule_k3.map(alloc::borrow::Cow::Owned)
                    }
                    crate::FederalIncome::Scorp { data } => {
                        crate::ScorpK1::from(&data).schedule_k3.map(alloc::borrow::Cow::Owned)
                    }
                    _ => None,
                },
            };
            schedule.map(|schedule| AttachedK3 {
                source: entry.id,
                owner: entry.owner.owned(),
                schedule,
            })
        })
        .collect()
}

// K-1 box 21 / box 16 code F foreign taxes of the K-1s with no Schedule K-3,
// whole cents: never credited, counted toward the section 904(j) test.
pub(crate) fn k1_taxes_without_k3(d: &crate::FederalDraft) -> i64 {
    crate::community::entries(d)
        .iter()
        .map(|entry| match &entry.income {
            crate::FederalIncome::Partnership { data } if data.schedule_k3.is_none() => {
                data.separate.as_ref().map_or(0, |items| items.foreign_taxes)
            }
            crate::FederalIncome::Scorp { data } => {
                let data = crate::ScorpK1::from(data);
                if data.schedule_k3.is_none() { data.foreign_taxes } else { 0 }
            }
            _ => 0,
        })
        .sum()
}

// Amounts conserving their total when split by weights (equal shares when
// every weight is zero); no weights, no shares.
use crate::kernel::ratio::allocate_prefix as spread;

// One country row of a Schedule K-3 as Form 1116 reads it, in cents: its
// group (see `ForeignOrdinaryTax::group`), line 24 income, tax, the line 2
// reduction and qualified dividends.
pub(crate) struct K3Row {
    pub source: Option<(i64, crate::Owner)>,
    pub country: String,
    pub group: usize,
    pub income: i64,
    // The tax as the schedule shows it (Part II column (w)) and the line 12
    // reduction taken off it.
    pub tax: i64,
    pub reduction: i64,
    pub qualified: i64,
}

// What the Schedule K-3s add to Form 1116, by group (see
// ForeignOrdinaryTax::group), in cents.
#[derive(Default)]
pub(crate) struct K3Credit {
    pub rows: Vec<K3Row>,
    pub income: [i64; crate::form_1116::SLOTS],
    pub tax: [i64; crate::form_1116::SLOTS],
    pub qualified: [i64; crate::form_1116::SLOTS],
    // Line 2: deductions of columns (b), (c) and (d), already allocated.
    pub deductions: [i64; crate::form_1116::SLOTS],
    // Line 4b: interest of columns (b), (c) and (d) and the apportioned
    // column (f).
    pub interest: [i64; crate::form_1116::SLOTS],
    // Lines 11, 12, 27 and 28 (short-term gain, long-term gain, short-term
    // loss, long-term loss) of the three columns, inside the line 24 income.
    pub capital: [[i64; 4]; crate::form_1116::SLOTS],
    // Line 3b: column (f) deductions, apportioned by gross income.
    pub partner_deductions: i64,
    // Line 12 reductions that reduced a row's tax.
    pub reductions: i64,
    // The entries (by position) whose column (f) interest has no Part III
    // assets to apportion it and so follows the entry's foreign income.
    pub interest_assumed: Vec<usize>,
}

impl K3Credit {
    pub(crate) fn row_count(&self) -> usize {
        self.rows.len()
    }
}

// The rows, deductions, interest and taxes of every Schedule K-3 on the
// return. `small_interest` is the small-foreign-income election, which
// assigns all interest to U.S. income.
pub(crate) fn credit(
    entries: &[impl core::borrow::Borrow<ScheduleK3>],
    small_interest: bool,
    limits: &[K3Limits],
) -> K3Credit {
    let entries: Vec<_> = entries.iter().map(|entry| (entry.borrow(), None)).collect();
    credit_with_sources(&entries, small_interest, limits)
}
pub(crate) fn owned_credit(
    entries: &[AttachedK3<'_>],
    small_interest: bool,
    limits: &[K3Limits],
) -> K3Credit {
    let entries: Vec<_> = entries
        .iter()
        .map(|entry| (entry.schedule.as_ref(), Some((entry.source, entry.owner))))
        .collect();
    credit_with_sources(&entries, small_interest, limits)
}
// The sole native financial traversal: source context travels with its own
// schedule before country/category merging and positional K-1 tax limits.
fn credit_with_sources(
    entries: &[(&ScheduleK3, Option<(i64, crate::Owner)>)],
    small_interest: bool,
    limits: &[K3Limits],
) -> K3Credit {
    use crate::form_1116::BRANCH;
    let mut out = K3Credit::default();
    // Passive rows of every K-3 (group, income) for the pooled column (f)
    // interest and the passive deductions of a K-3 without rows.
    let mut passive_rows: Vec<(usize, i64)> = Vec::new();
    let mut business = (0, K3Assets::default());
    let mut activity = (0, K3Assets::default());
    for (position, (k3, source)) in entries.iter().enumerate() {
        let limit = limits.get(position).copied().unwrap_or(K3Limits::FULL);
        let scale_tax = |v: i64| fraction(v, limit.tax.0, limit.tax.1);
        let scale = |v: i64| fraction(v, limit.deductions.0, limit.deductions.1);
        for category in [K3Category::ForeignBranch, K3Category::Passive, K3Category::General] {
            let rows: Vec<&K3Country> =
                k3.countries.iter().filter(|row| row.category == category).collect();
            let amounts = match category {
                K3Category::ForeignBranch => &k3.branch,
                K3Category::Passive => &k3.passive,
                K3Category::General => &k3.general,
            };
            let weights: Vec<i64> = rows.iter().map(|row| row.income).collect();
            let reduction = spread(amounts.reduction, &weights);
            for (index, row) in rows.iter().enumerate() {
                let row_tax = scale_tax(row.tax);
                let tax = phase::floor(row_tax - reduction.get(index).copied().unwrap_or(0), 0);
                out.reductions += row_tax - tax;
                let group = k3.group(row);
                out.income[group] += row.income;
                out.tax[group] += tax;
                out.qualified[group] += row.qualified;
                out.rows.push(K3Row {
                    source: *source,
                    country: row.country.clone(),
                    group,
                    income: row.income,
                    tax: row_tax,
                    reduction: row_tax - tax,
                    qualified: row.qualified,
                });
                if category == K3Category::Passive {
                    passive_rows.push((group, row.income));
                }
            }
            let default_group = match category {
                K3Category::ForeignBranch => BRANCH,
                K3Category::Passive => 3,
                K3Category::General => 0,
            };
            let groups: Vec<usize> = if rows.is_empty() {
                alloc::vec![default_group]
            } else {
                rows.iter().map(|row| k3.group(row)).collect()
            };
            let weights = if rows.is_empty() { alloc::vec![0] } else { weights };
            for (group, share) in groups.iter().zip(spread(scale(amounts.deductions), &weights)) {
                out.deductions[*group] += share;
            }
            if !small_interest {
                for (group, share) in groups.iter().zip(spread(scale(amounts.interest), &weights)) {
                    out.interest[*group] += share;
                }
            }
            for (line, value) in amounts.capital.lines().into_iter().enumerate() {
                for (group, share) in groups.iter().zip(spread(value, &weights)) {
                    out.capital[*group][line] += share;
                }
            }
        }
        out.partner_deductions += scale(k3.partner_deductions);
        let interest = &k3.partner_interest;
        business.0 += scale(interest.business);
        activity.0 += scale(interest.passive_activity);
        for (assets, add) in [
            (&mut business.1, &interest.business_assets),
            (&mut activity.1, &interest.passive_assets),
        ] {
            assets.us += add.us;
            assets.branch += add.branch;
            assets.passive += add.passive;
            assets.general += add.general;
        }
    }
    // Reg. 1.861-9T(d)(1): the column (f) interest follows the Part III
    // Section 2 assets, pooled over these K-3s (inventor decision of
    // 2026-10-09: the person's other businesses are not in the pool).
    if !small_interest {
        for (which, (interest, assets)) in [&business, &activity]
            .into_iter()
            .map(|(interest, assets)| (*interest, assets))
            .enumerate()
        {
            if interest == 0 {
                continue;
            }
            let total = assets.total();
            if total == 0 {
                // No assets to apportion with: each entry's interest follows
                // its own foreign income (conservative), noted per entry.
                for (position, (k3, _)) in entries.iter().enumerate() {
                    let own = if which == 0 {
                        k3.partner_interest.business
                    } else {
                        k3.partner_interest.passive_activity
                    };
                    if own == 0 || k3.countries.is_empty() {
                        continue;
                    }
                    let weights: Vec<i64> = k3.countries.iter().map(|row| row.income).collect();
                    for (row, share) in k3.countries.iter().zip(spread(own, &weights)) {
                        out.interest[k3.group(row)] += share;
                    }
                    if !out.interest_assumed.contains(&position) {
                        out.interest_assumed.push(position);
                    }
                }
                continue;
            }
            out.interest[0] += fraction(interest, assets.general, total);
            out.interest[BRANCH] += fraction(interest, assets.branch, total);
            let passive = fraction(interest, assets.passive, total);
            if passive != 0 {
                let (groups, weights): (Vec<usize>, Vec<i64>) = if passive_rows.is_empty() {
                    (alloc::vec![3], alloc::vec![0])
                } else {
                    passive_rows.iter().copied().unzip()
                };
                for (group, share) in groups.iter().zip(spread(passive, &weights)) {
                    out.interest[*group] += share;
                }
            }
        }
    }
    out
}

// The Form 1116 line 3e correction of one K-1, whole dollars: the K-3's gross
// income in place of the K-1's net amounts that are in adjusted gross income
// (its Schedule E Part II amount, interest, dividends and royalty receipts).
// Royalty deductions are restored separately when replacing net income with
// receipts, so subtract the gross royalties here before adding K-3 gross.
pub(crate) fn gross_correction(k3: &ScheduleK3, in_agi: i64, portfolio: i64) -> i64 {
    dollars(k3.gross()) - in_agi - dollars(portfolio)
}

// What the Schedule K-3s do to Form 1116 line 3e and the pass-through refusal,
// from the passive-pool result (extended_rules::RentalResult.k1_amounts).
pub(crate) struct K3Basis {
    // Line 3e correction, whole dollars.
    pub correction: i64,
    // Schedule E Part II amounts of the K-1s that have a K-3, whole dollars.
    pub amounts: i64,
    // A K-1 with a K-3 holds back a loss or deduction this year.
    pub limited: bool,
    // What each K-3's K-1 allows this year, in the order of `attached`.
    pub limits: Vec<K3Limits>,
}

// The part of a Schedule K-3 the K-1's basis and at-risk limits allow this year
// (Reg. 1.704-1(d)(2), 1.1366-2(a)(5)): the schedule is before those limits, so
// its foreign taxes follow the foreign tax class and its deductions and interest
// the ordinary class, each as allowed over allowed plus held back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct K3Limits {
    pub tax: (i64, i64),
    pub deductions: (i64, i64),
}

impl K3Limits {
    pub(crate) const FULL: Self = Self { tax: (1, 1), deductions: (1, 1) };
}

pub(crate) fn basis(
    d: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
) -> K3Basis {
    use crate::k1_loss::K1LossClass;
    let mut out = K3Basis { correction: 0, amounts: 0, limited: false, limits: Vec::new() };
    for entry in &d.income {
        // A K-1 whose limits cannot be figured has no ledger and is refused
        // elsewhere.
        let ledger =
            pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger);
        // A loss or deduction that this K-1's basis or at-risk limit holds
        // back this year, or a foreign-tax carry it releases: the Schedule K-3
        // is before such limits, so its category deductions would overstate
        // what the return allows (section 904(a)).
        let (k3, portfolio, limited) = match &entry.income {
            crate::FederalIncome::Partnership { data } => match data.schedule_k3.clone() {
                Some(k3) => (
                    k3,
                    data.separate
                        .as_ref()
                        .map_or(0, |items| items.interest + items.dividends + items.royalties),
                    ledger.is_some_and(|ledger| {
                        ledger.held_back()
                            || (data.regular.carried.foreign_taxes != 0
                                && ledger.basis_allowed[K1LossClass::ForeignTaxes].get()
                                    > data.separate.as_ref().map_or(0, |v| v.foreign_taxes))
                    }),
                ),
                None => continue,
            },
            crate::FederalIncome::Scorp { data } => {
                let data = crate::ScorpK1::from(data);
                match data.schedule_k3.clone() {
                    Some(k3) => (
                        k3,
                        data.interest + data.dividends + data.royalties,
                        ledger.is_some_and(|ledger| {
                            ledger.held_back()
                                || (data.carried.foreign_taxes != 0
                                    && ledger.basis_allowed[K1LossClass::ForeignTaxes].get()
                                        > data.foreign_taxes)
                        }),
                    ),
                    None => continue,
                }
            }
            _ => continue,
        };
        let net: i64 = pool.k1_amounts.iter().filter(|v| v.id == entry.id).map(|v| v.net).sum();
        out.correction += gross_correction(&k3, net, portfolio);
        out.amounts += net;
        out.limited |= limited;
        out.limits.push(match ledger {
            Some(ledger) if ledger.limited() => {
                let share = |allowed: i64, held: i64| {
                    if allowed + held > 0 { (allowed, allowed + held) } else { (1, 1) }
                };
                K3Limits {
                    tax: share(
                        ledger.basis_allowed[K1LossClass::ForeignTaxes].get(),
                        ledger.basis_suspended[K1LossClass::ForeignTaxes].get(),
                    ),
                    deductions: share(
                        ledger.risk_allowed[K1LossClass::Ordinary].get(),
                        ledger.basis_suspended[K1LossClass::Ordinary].get()
                            + ledger.risk_suspended[K1LossClass::Ordinary].get(),
                    ),
                }
            }
            _ => K3Limits::FULL,
        });
    }
    out
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3Country {
    amounts: [income, qualified, tax],
    gains: [],
    facts: [country, category],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3Assets {
    amounts: [us, branch, passive, general],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3Capital {
    amounts: [short_gain, long_gain, short_loss, long_loss],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3CategoryAmounts {
    amounts: [deductions, interest, reduction, capital],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3PartnerInterest {
    amounts: [business, passive_activity, business_assets, passive_assets],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K3Withholding {
    amounts: [],
    gains: [],
    facts: [country, treatment],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScheduleK3 {
    amounts: [
        countries,
        withholding,
        us_gross,
        branch,
        passive,
        general,
        partner_deductions,
        partner_interest
    ],
    gains: [],
    facts: [items],
});
