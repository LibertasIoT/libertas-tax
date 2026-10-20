// Coverage: Schedule K-3 (1065 / 1120S) — Supported for Parts I–III (unit k3, 2026-10-09; 2026
//   drafts f1065sk3 and f1120sk3 of June 2026, i1065sk3 and i1120sk3 2025 final (no 2026 revision),
//   i1116 2026 draft of 9/30/26, Schedule B (Form 1116) Rev. 12-2026 draft, Pub 514, IRC 702(a)(6),
//   705(a)(2), 901, 904(a), (c), (d), (f), (j), 905(a), 909, 1366(a)(1)(A), 1367(a)(2), 1373(a), 26 CFR
//   1.861-8(e)(15), 1.861-9T(d)(1), 1.904-4(c)(3), (n)(1)(ii)). Only the parts a partner's or
//   shareholder's Form 1116 reads are modelled.
//   A ScheduleK3 is a field of its K-1 (K1Partnership/K1Scorp.schedule_k3: K3Attached; stored as
//   PartnerK1/ScorpK1.schedule_k3), with its own AI Import action; the K-1 import never reads
//   box 16 (1120-S box 14) and the K-3 import never touches the K-1. Removing the K-1 removes it.
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
    /// ----
    /// Country
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub countries: Vec<K3Country>,
    /// Foreign withholding treatment of the passive countries
    /// One answer for each country that has a passive row above, from your own records of how the tax was withheld; the Schedule K-3 import never reads it, so an imported schedule keeps your answers for the countries it still lists. Compare the withholding with the row's income; keep a net-basis tax (OTH, OTHS, OTHR) or no tax under "No foreign withholding tax".
    /// ----
    /// Country
    #[libertas_unordered]
    pub withholding: Vec<K3Withholding>,
    /// U.S. and partner-sourced gross income — Part II line 24, columns (a) and (f)
    /// Column (f) income is U.S. income for a U.S. citizen or resident whose tax home is in the United States.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub us_gross: i64,
    /// Foreign branch category — column (b)
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub branch: K3CategoryAmounts,
    /// Passive category — column (c)
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub passive: K3CategoryAmounts,
    /// General category — column (d)
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
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
    #[libertas_foldable(initially_open = false)]
    pub partner_interest: K3PartnerInterest,
    /// Other entries on this Schedule K-3
    /// ----
    /// Entry
    #[libertas_llm_doc_field]
    #[libertas_unordered]
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
    #[libertas_foldable(initially_open = false)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub short_gain: i64,
    /// Net long-term capital gain — line 12
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub long_gain: i64,
    /// Net short-term capital loss — line 27
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub short_loss: i64,
    /// Net long-term capital loss — line 28
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub long_loss: i64,
}

/// Interest expense left for you to apportion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K3PartnerInterest {
    /// Business interest — Part II line 41, column (f)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub business: i64,
    /// Passive activity interest — Part II line 43, column (f)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub branch: i64,
    /// Passive category — column (c)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub passive: i64,
    /// General category — column (d)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
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
        if let Some((index, _)) = self.countries.iter().enumerate().find(|(_, row)| {
            row.passive() && !self.withholding.iter().any(|answer| answer.country == row.country)
        }) {
            return Some(
                ValidationIssue::new("K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY")
                    .at_field("country")
                    .in_list("countries", index),
            );
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        self.rule().is_none()
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
                        data.schedule_k3.as_ref().map(alloc::borrow::Cow::Borrowed)
                    }
                    _ => None,
                },
                alloc::borrow::Cow::Owned(income) => match income {
                    crate::FederalIncome::Partnership { data } => {
                        data.schedule_k3.map(alloc::borrow::Cow::Owned)
                    }
                    crate::FederalIncome::Scorp { data } => {
                        data.schedule_k3.map(alloc::borrow::Cow::Owned)
                    }
                    _ => None,
                },
            };
            schedule.map(|schedule| AttachedK3 { source: entry.id, owner: entry.owner, schedule })
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
            crate::FederalIncome::Scorp { data } if data.schedule_k3.is_none() => {
                data.foreign_taxes
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
            crate::FederalIncome::Partnership { data } => match &data.schedule_k3 {
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
            crate::FederalIncome::Scorp { data } => match &data.schedule_k3 {
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
            },
            _ => continue,
        };
        let net: i64 = pool.k1_amounts.iter().filter(|v| v.id == entry.id).map(|v| v.net).sum();
        out.correction += gross_correction(k3, net, portfolio);
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
