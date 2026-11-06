// Schedule K-1 (Form 1065), 2026 draft, created March 24, 2026. The
// currently published Partner Instructions are the 2025 edition. Paper facts
// remain separate from partner worksheets and the calculation owner's view.
use crate::k1_page::K1Partnership;
use crate::*;
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// An entry exactly as printed in a coded K-1 box
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1CodeRow {
    /// Code as printed
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    // IRS K-1 codes have one or two letters; an attached statement adds * or uses * alone.
    // https://www.irs.gov/instructions/i1065sk1
    // https://www.irs.gov/instructions/i1120ssk
    #[libertas_pattern(r"([A-Z]{1,2}\*?|\*)?")]
    pub code: String,
    /// [NullText]
    /// No numeric amount is printed
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub amount: Option<i64>,
    /// Other printed text, including STMT, asterisks and statement references
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub text: String,
    /// The row explicitly refers to an attached statement
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub statement: bool,
}

/// Beginning and ending amounts in one printed liability row.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1MoneyPair {
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub beginning: Option<i64>,
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub ending: Option<i64>,
}
/// Partner’s share of liabilities — item K1, in printed row order.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1Liabilities {
    /// Nonrecourse
    #[libertas_llm_doc_field]
    pub nonrecourse: K1MoneyPair,
    /// Qualified nonrecourse financing
    #[libertas_llm_doc_field]
    pub qualified_nonrecourse: K1MoneyPair,
    /// Recourse
    #[libertas_llm_doc_field]
    pub recourse: K1MoneyPair,
}
impl K1Liabilities {
    pub(crate) fn beginning_total(&self) -> i64 {
        self.nonrecourse
            .beginning
            .unwrap_or(0)
            .checked_add(self.qualified_nonrecourse.beginning.unwrap_or(0))
            .and_then(|sum| sum.checked_add(self.recourse.beginning.unwrap_or(0)))
            .expect("source liability total admitted")
    }
    pub(crate) fn ending_total(&self) -> i64 {
        self.nonrecourse
            .ending
            .unwrap_or(0)
            .checked_add(self.qualified_nonrecourse.ending.unwrap_or(0))
            .and_then(|sum| sum.checked_add(self.recourse.ending.unwrap_or(0)))
            .expect("source liability total admitted")
    }
}
/// Beginning and ending percentages in one printed ownership row.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1PercentagePair {
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub beginning: Option<f64>,
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub ending: Option<f64>,
}
/// Partner’s share of profit, loss and capital — item J, in printed row order.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1SharePercentages {
    /// Profit
    #[libertas_llm_doc_field]
    pub profit: K1PercentagePair,
    /// Loss
    #[libertas_llm_doc_field]
    pub loss: K1PercentagePair,
    /// Capital
    #[libertas_llm_doc_field]
    pub capital: K1PercentagePair,
}

/// Partner’s Capital Account Analysis — item L
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1CapitalAccount {
    /// Beginning capital account
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub beginning: Option<i64>,
    /// Capital contributed during the year
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub contributed: Option<i64>,
    /// Current-year net income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub income: Option<i64>,
    /// Other increase (decrease)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub other: Option<i64>,
    /// Attached explanation of other increase (decrease)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub explanation: String,
    /// Withdrawals and distributions
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withdrawals: Option<i64>,
    /// Ending capital account
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub ending: Option<i64>,
}

/// Disregarded entity — item H2
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1DisregardedEntity {
    /// TIN
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub tin: String,
    /// Name
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub name: String,
}

/// Schedule K-1 (Form 1065) — Partner’s Share of Income, Deductions, Credits, etc.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1PartnershipPaper {
    /// Actual beginning and ending dates printed at the top of this K-1; null when unavailable.
    /// [NullText]
    /// The entity’s tax period is unavailable
    /// [SetText]
    /// The entity’s printed tax period is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub reporting_period: Option<K1EntityTaxPeriod>,
    /// Final K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_k1: bool,
    /// Amended K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amended_k1: bool,
    /// Items A and B — Partnership’s employer identification number, name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub partnership: PrintedParty,
    /// C — IRS center where partnership filed return
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub irs_center: String,
    /// D — Check if this is a publicly traded partnership (PTP)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub publicly_traded: bool,
    /// Items E and F — Partner’s SSN or TIN, name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub partner: PrintedParty,
    /// G — General partner or LLC member-manager
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub general_partner: bool,
    /// G — Limited partner or other LLC member
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub limited_partner: bool,
    /// H1 — Domestic partner
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub domestic_partner: bool,
    /// H1 — Foreign partner
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub foreign_partner: bool,
    /// H2 — If the partner is a disregarded entity (DE), enter the partner’s TIN and name.
    /// [NullText]
    /// No disregarded entity is printed
    /// [SetText]
    /// The printed disregarded entity is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub disregarded_entity: Option<K1DisregardedEntity>,
    /// I1 — What type of entity is the partner?
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub entity_type: String,
    /// I2 — If the partner is a retirement plan (IRA/SEP/Keogh/etc.), check here
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub retirement_plan: bool,
    /// J — Partner’s share of profit, loss, and capital
    #[libertas_llm_doc_field]
    pub share: K1SharePercentages,
    /// J — Check if decrease is due to sale of partnership interest
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub share_sale: bool,
    /// J — Check if decrease is due to exchange of partnership interest
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub share_exchange: bool,
    /// K1 — Partner’s share of liabilities
    #[libertas_llm_doc_field]
    pub liabilities: K1Liabilities,
    /// K2 — Item K1 includes liability amounts from lower-tier partnerships
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub lower_tier_liabilities: bool,
    /// K3 — Liability is subject to guarantees or other payment obligations by the partner
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub guaranteed_liabilities: bool,
    /// L — Partner’s Capital Account Analysis
    #[libertas_llm_doc_field]
    pub capital_account: K1CapitalAccount,
    /// M — Did the partner contribute property with a built-in gain (loss)?
    /// [NullText]
    /// Neither answer is marked
    /// [SetText]
    /// The printed answer is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub contributed_built_in_property: Option<bool>,
    /// N — Beginning net unrecognized section 704(c) gain or loss
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub unrecognized_704c_beginning: Option<i64>,
    /// N — Ending net unrecognized section 704(c) gain or loss
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub unrecognized_704c_ending: Option<i64>,
    /// 1 — Ordinary business income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub ordinary: Option<i64>,
    /// 2 — Net rental real estate income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub rental_real_estate: Option<i64>,
    /// 3 — Other net rental income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub other_rental: Option<i64>,
    /// 4a — Guaranteed payments for services
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub services: Option<i64>,
    /// 4b — Guaranteed payments for capital
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub capital: Option<i64>,
    /// 4c — Total guaranteed payments
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub total_guaranteed: Option<i64>,
    /// 5 — Interest income
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub interest: Option<i64>,
    /// 6a — Ordinary dividends
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub dividends: Option<i64>,
    /// 6b — Qualified dividends
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub qualified_dividends: Option<i64>,
    /// 6c — Dividend equivalents
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub dividend_equivalents: Option<i64>,
    /// 7 — Royalties
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub royalties: Option<i64>,
    /// 8 — Net short-term capital gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub short: Option<i64>,
    /// 9a — Net long-term capital gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub long: Option<i64>,
    /// 9b — Collectibles (28%) gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub collectibles: Option<i64>,
    /// 9c — Unrecaptured section 1250 gain
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub unrecaptured: Option<i64>,
    /// 10 — Net section 1231 gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub section1231: Option<i64>,
    /// 11 — Other income (loss)
    #[libertas_llm_doc_field]
    pub income: Vec<K1CodeRow>,
    /// 12 — Section 179 deduction
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section179: Option<i64>,
    /// 13 — Other deductions
    #[libertas_llm_doc_field]
    pub deductions: Vec<K1CodeRow>,
    /// 14 — Self-employment earnings (loss)
    /// Read only rows printed in box 14, each with its own printed code and amount. The amount printed in box 1 or any other box is never a box 14 row; an empty box is an empty list.
    #[libertas_llm_doc_field]
    pub self_employment: Vec<K1CodeRow>,
    /// 15 — Credits
    /// Read only rows printed in box 15, each with its own printed code. The amount printed in box 3 or any other box is never a box 15 row; an empty box is an empty list.
    #[libertas_llm_doc_field]
    pub credits: Vec<K1CodeRow>,
    /// 16 — Schedule K-3 is attached if checked
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub k3_attached: bool,
    /// 17 — Alternative minimum tax (AMT) items
    #[libertas_llm_doc_field]
    pub amt: Vec<K1CodeRow>,
    /// 18 — Tax-exempt income and nondeductible expenses
    #[libertas_llm_doc_field]
    pub exempt: Vec<K1CodeRow>,
    /// 19 — Distributions
    #[libertas_llm_doc_field]
    pub distributions: Vec<K1CodeRow>,
    /// 20 — Other information
    #[libertas_llm_doc_field]
    pub information: Vec<K1CodeRow>,
    /// 21 — Foreign taxes paid or accrued
    /// Copy only the amount printed in box 21; never borrow another box's amount.
    /// [NullText]
    /// Box 21 is blank
    /// [SetText]
    /// Printed foreign taxes paid or accrued are
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub foreign_taxes: Option<i64>,
    /// 22 — More than one activity for at-risk purposes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub multiple_at_risk: bool,
    /// 23 — More than one activity for passive activity purposes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub multiple_passive: bool,
}

/// Your partnership basis and at-risk facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerBasisNative {
    /// Opening basis and retained loss history
    pub start: PartnerBasisStartNative,
    /// Cash contributed during the year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub contributions: i64,
    /// Beginning liabilities at risk
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub debt_start: i64,
    /// Ending liabilities at risk
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub debt_end: i64,
    /// Other at-risk changes
    pub more_at_risk: PartnerAtRisk,
}

/// Opening basis and retained history, independently of editor choices.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PartnerBasisStartNative {
    /// First year holding the interest
    New,
    /// An interest held before this year
    Continuing { opening: Option<i64>, risk_history: ScorpRiskStart, carried: Box<PartnerCarried> },
}
impl From<&PartnerBasis> for PartnerBasisNative {
    fn from(page: &PartnerBasis) -> Self {
        Self {
            start: match &page.start {
                BasisStart::New => PartnerBasisStartNative::New,
                BasisStart::Continuing { opening, risk_history, carried } => {
                    PartnerBasisStartNative::Continuing {
                        opening: *opening,
                        risk_history: risk_history.clone(),
                        carried: Box::new(carried.clone()),
                    }
                }
            },
            contributions: page.contributions,
            debt_start: page.debt_start,
            debt_end: page.debt_end,
            more_at_risk: page.more_at_risk.clone(),
        }
    }
}
impl From<&PartnerBasisNative> for PartnerBasis {
    fn from(record: &PartnerBasisNative) -> Self {
        Self {
            start: match &record.start {
                PartnerBasisStartNative::New => BasisStart::New,
                PartnerBasisStartNative::Continuing { opening, risk_history, carried } => {
                    BasisStart::Continuing {
                        opening: *opening,
                        risk_history: risk_history.clone(),
                        carried: carried.as_ref().clone(),
                    }
                }
            },
            contributions: record.contributions,
            debt_start: record.debt_start,
            debt_end: record.debt_end,
            more_at_risk: record.more_at_risk.clone(),
        }
    }
}

/// Property and section 199A details actually assigned by the attached statements
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnershipRentalStatement {
    /// Section 1231 amount assigned to box 2
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate_section1231: i64,
    /// Unrecaptured section 1250 amount assigned to box 2
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate_unrecaptured: i64,
    /// Section 1231 amount assigned to box 3
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_section1231: i64,
    /// Unrecaptured section 1250 amount assigned to box 3
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_unrecaptured: i64,
    /// Section 199A amounts for box 2
    /// [NullText]
    /// The statement supplies no section 199A amounts for box 2
    /// [SetText]
    /// The statement's section 199A amounts for box 2 are
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub real_estate_qbi: Option<K1RentalQbi>,
    /// Section 199A amounts for box 3
    /// [NullText]
    /// The statement supplies no section 199A amounts for box 3
    /// [SetText]
    /// The statement's section 199A amounts for box 3 are
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub other_qbi: Option<K1RentalQbi>,
}

/// Source-assigned business and portfolio property character
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnershipPropertyStatement {
    /// Box 9c from selling another partnership interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Box 9c from estates, trusts, mutual funds or REITs
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Box 9c from the box 1 business
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_business: i64,
    /// Box 10 assigned to the box 1 business
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_business: i64,
}

/// Actual attached partnership statements
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnershipStatements {
    /// Box 20 code Z section 199A statement
    #[libertas_llm_doc_field]
    pub qbi: PartnerQbi,
    /// Amounts assigned to the rental activities
    #[libertas_llm_doc_field]
    pub rentals: PartnershipRentalStatement,
    /// Amounts assigned to business or portfolio property
    #[libertas_llm_doc_field]
    pub property: PartnershipPropertyStatement,
    /// Box 11 detailed character reported by the statement
    #[libertas_llm_doc_field]
    pub income: Vec<PartnershipIncomeStatement>,
    /// Box 15 credit code and Form 3800 line reported by the statement
    #[libertas_llm_doc_field]
    pub credits: Vec<PartnershipCreditStatement>,
    /// Actual investment-credit properties
    #[libertas_llm_doc_field]
    pub investment_property: Vec<K1InvestmentProperty>,
    /// Actual section 179 sale statements
    #[libertas_llm_doc_field]
    pub section179_sales: Vec<K1Section179Sale>,
    /// Attached Schedule K-3. Its existing supported source fields are retained.
    /// [NullText]
    /// No Schedule K-3 is entered
    /// [SetText]
    /// The attached Schedule K-3 is
    #[libertas_default($null)]
    pub schedule_k3: Option<crate::ScheduleK3Paper>,
    /// Other situations explicitly described by an attached statement
    #[libertas_llm_doc_field]
    pub situations: Vec<PartnershipStatementSituation>,
}

/// A situation established by an attached statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PartnershipStatementSituation {
    /// Rental property disposition needing separate worksheets
    RentalDisposition,
    /// REMIC residual interest or an unsupported box 11 allocation
    OtherIncome,
    /// Other information requiring separate worksheets
    OtherInformation,
    /// Agricultural or horticultural cooperative section 199A items
    CooperativeQbi,
    /// More than one activity of the same kind
    MultipleActivities,
}

/// Independent partnership personal records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerPersonalRecords {
    /// Section 469 participation records
    pub participation: Option<PartnerParticipation>,
    /// Section 1402 self-employment classification
    pub limited_for_se: Option<bool>,
    /// Basis and at-risk records
    pub basis: PartnerBasisNative,
    /// Personal-services source records
    pub income_source: K1IncomeSource,
    /// Independent AMT records
    pub amt: Option<PartnerAmtRecords>,
    /// Actual same-paper activity allocation of printed code C.
    #[libertas_default($null)]
    pub depletion_sources: Option<Vec<crate::K1DepletionSourceAllocation>>,
    /// What happened to the interest
    pub disposition: Option<PartnerDisposition>,
    /// Personal special situations
    pub special: Vec<PartnerSpecial>,
    /// Individual QBI aggregation election
    pub qbi_group: String,
    /// Retained QBI history and gross allocation
    pub qbi_records: PartnerQbiRecords,
    /// Fuel credit records
    pub fuel_credit: Option<PartnerFuelTaxCredit>,
    /// Earlier fuel credit records
    pub prior_fuel_credit: Option<PartnerPriorFuelCredit>,
    /// Interest holding period
    pub term: K1StockTerm,
    /// Trading classification
    pub trading: bool,
    /// Rental participation records
    pub rental_participation: Option<K1RentalParticipation>,
    /// Participation when the carried rental loss arose
    pub rental_carryover_active: bool,
    /// Rental activities disposed of in full
    pub rental_disposed: Vec<K1RentalActivity>,
    /// Regular section 179 carryover
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub section179_carryover: i64,
    /// Carryover belonging to sold property
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub section179_carryover_sold: i64,
    /// Independent AMT section 179 history
    pub amt_section179_income_carry: Option<Section179IncomeCarryHistory>,
    /// Automatic investment-credit confirmation
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    /// Low-income housing acquisition records
    pub housing_confirmed: Option<K1HousingAcquisition>,
    /// Credit origins keyed by the existing code and line owner
    pub credit_origins: Vec<PartnerCreditPersonal>,
    /// Completed investment-credit forms keyed by actual property identity
    pub investment_forms: Vec<PartnerInvestmentPersonal>,
    /// Personal withholding classification for passive K-3 countries
    pub k3_withholding: Vec<K3Withholding>,
    /// Your eligible-small-business treatment for code M, independent of printed facts.
    pub research_credit_line: Option<BusinessCredit>,
    /// Partner coverage records
    pub health_insurance: BusinessHealthInsurance,
    /// Premiums charged against distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub health_distributions: i64,
    /// Plan receiving partnership retirement payments
    pub retirement: PartnerRetirement,
}

/// Source-specific personal credit answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerCreditPersonal {
    /// Actual credit code
    pub code: K1CreditCode,
    /// Actual Form 3800 line
    pub line: Option<BusinessCredit>,
    /// Equipment origin
    pub carbon_oxide_origin: Option<CarbonOxideCreditOrigin>,
    /// Facility origin
    pub renewable_facility_origin: Option<RenewableFacilityCreditOrigin>,
}

/// Completed Form 3468 for an identified property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerInvestmentPersonal {
    /// Accepted property identity
    pub id: i64,
    /// Your completed Form 3468
    pub own_form: Option<OwnInvestmentCredit>,
}

/// Accepted partnership paper and actual attached source statements
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnershipSource {
    /// The actual printed paper
    pub paper: K1PartnershipPaper,
    /// The actual attached source statements
    pub statements: PartnershipStatements,
    /// Accepted document association, independent of printed identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: DocumentRecipient,
    /// How the App assigned the owner
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub owner_source: Option<crate::OwnerSource>,
    /// Your share of a joint document
    /// Percent of the document's amounts that are yours; your spouse's share is the rest.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub joint_share: Option<i32>,
}

/// Partnership document and its attached statements.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the actual Schedule K-1 (Form 1065) and supplied attached statements other than Schedule K-3. The attached Schedule K-3 has its own header and section import actions; preserve it and import its printed fields there. The 2026 paper follows the 2026 draft; current Partner Instructions are the 2025 edition. Retain each printed box and coded row separately, with its printed sign, text, asterisks and statement references. Part III prints two columns of boxes on shared lines: boxes 1 through 13 on the left and boxes 14 through 23 on the right. Each amount belongs only to the box whose number labels its column and is printed directly beneath that box's caption; an amount in the left column (such as box 1 or box 3) is never a box 14 or box 15 row. A coded box (11, 13 through 15, 17 through 20) has a row only where a code letter is printed in that box; with no printed code the box is an empty list. Box 12 section 179 deduction is the amount printed beneath its caption, read as printed. Every blank printed numeric box is null, including amounts, liabilities, capital accounts and ownership percentages; preserve a printed zero as zero. Never fill a blank box from another amount or a tax calculation. Every empty coded box is an empty list, including boxes 14 and 15; never create a placeholder row for an empty box. A printed coded row with no numeric amount has amount null. Read each amount only from its labeled box or an attached statement explicitly identifying that box and code. Never derive self-employment earnings from ordinary business income, or foreign taxes from section 179 or another amount. Never guess a missing amount. Read the actual entity period only; it belongs to the supplied year when the entity period ends in that year. Copy partnership and partner identification exactly, including masking and leading zeros. Never infer household ownership or personal worksheet answers. For code M research credit, never import its Form 3800 line or eligible-small-business treatment; that is the person's own answer. Read statement-specific allocations only when supplied; totals alone cannot determine activity allocation. Do not import personal, completed owner forms or personal K-3 withholding classification. Preserve accepted source identities in property statements. Report conflicting or illegible facts as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.form.partnership.federal_id,@.form.partner.federal_id,@.form.reporting_period.began,@.form.reporting_period.ended,@.form.unrecognized_704c_beginning,@.form.unrecognized_704c_ending,@.form.ordinary,@.form.rental_real_estate,@.form.other_rental,@.form.services,@.form.capital,@.form.total_guaranteed,@.form.interest,@.form.dividends,@.form.qualified_dividends,@.form.dividend_equivalents,@.form.royalties,@.form.short,@.form.long,@.form.collectibles,@.form.unrecaptured,@.form.section1231,@.form.section179,@.form.foreign_taxes",
    form_date_order = "MonthDayYear"
)]
pub struct K1PartnershipDocument {
    /// Schedule K-1 (Form 1065)
    #[libertas_llm_doc_field]
    pub form: K1PartnershipPaper,
    /// Attached statements
    #[libertas_llm_doc_field]
    pub statements: PartnershipStatements,
    /// Independent accepted personal records
    #[libertas_hidden]
    #[libertas_default($null)]
    pub personal: Option<PartnerPersonalRecords>,
}

impl Default for PartnerPersonalRecords {
    fn default() -> Self {
        Self {
            participation: None,
            limited_for_se: None,
            basis: PartnerBasisNative {
                start: PartnerBasisStartNative::Continuing {
                    opening: None,
                    risk_history: ScorpRiskStart::Unknown,
                    carried: Box::new(PartnerCarried {
                        basis_loss: 0,
                        basis_nondeductible: 0,
                        risk_loss: 0,
                        risk_nondeductible: 0,
                        deducted: 0,
                        recaptured: 0,
                        passive_loss: 0,
                        rental_real_estate_passive_loss: 0,
                        other_rental_passive_loss: 0,
                        rental_real_estate_section1231_passive_loss: 0,
                        other_rental_section1231_passive_loss: 0,
                        basis_short: 0,
                        basis_long: 0,
                        risk_short: 0,
                        risk_long: 0,
                        rental_real_estate_basis_loss: 0,
                        rental_real_estate_risk_loss: 0,
                        other_rental_basis_loss: 0,
                        other_rental_risk_loss: 0,
                        basis_section179: 0,
                        basis_charity: 0,
                        basis_foreign_taxes: 0,
                        se_basis: 0,
                        se_risk: 0,
                        se_passive: 0,
                        section1231_basis_loss: 0,
                        section1231_risk_loss: 0,
                        section1231_passive_loss: 0,
                    }),
                },
                contributions: 0,
                debt_start: 0,
                debt_end: 0,
                more_at_risk: PartnerAtRisk {
                    excluded_contributions: 0,
                    excluded_distributions: 0,
                    protected: 0,
                    released: 0,
                },
            },
            income_source: K1IncomeSource::NotNeeded,
            amt: None,
            depletion_sources: None,
            disposition: None,
            special: Vec::new(),
            qbi_group: String::new(),
            qbi_records: PartnerQbiRecords::default(),
            fuel_credit: None,
            prior_fuel_credit: None,
            term: K1StockTerm::NotNeeded,
            trading: false,
            rental_participation: None,
            rental_carryover_active: true,
            rental_disposed: Vec::new(),
            section179_carryover: 0,
            section179_carryover_sold: 0,
            amt_section179_income_carry: None,
            investment_confirmed: None,
            housing_confirmed: None,
            credit_origins: Vec::new(),
            investment_forms: Vec::new(),
            k3_withholding: Vec::new(),
            research_credit_line: None,
            health_insurance: BusinessHealthInsurance::None,
            health_distributions: 0,
            retirement: PartnerRetirement::None,
        }
    }
}
impl PartnerPersonalRecords {
    pub(crate) fn from_page(page: &K1Partnership) -> Self {
        Self {
            participation: page.participation.clone(),
            limited_for_se: page.limited_for_se,
            research_credit_line: page.more.credits.iter().find_map(|credit| {
                let credit: K1Credit = credit.clone().into();
                (credit.code == K1CreditCode::Research).then_some(credit.line).flatten()
            }),
            income_source: page.income_source.clone(),
            amt: page.amt.clone(),
            depletion_sources: None,
            disposition: page.disposition.clone(),
            special: page.special.clone(),
            qbi_group: page.qbi_group.clone(),
            basis: (&page.basis).into(),
            qbi_records: page.qbi_records_record(),
            fuel_credit: page.more.fuel_credit.clone(),
            prior_fuel_credit: page.more.prior_fuel_credit.clone(),
            term: page.more.term,
            trading: page.more.trading,
            rental_participation: page.rentals.participation,
            rental_carryover_active: page.rentals.carryover_active,
            rental_disposed: page.rentals.disposed.clone(),
            section179_carryover: page.more.section179_carryover,
            section179_carryover_sold: page.more.section179_carryover_sold,
            amt_section179_income_carry: page.more.amt_section179_income_carry.clone(),
            investment_confirmed: page.more.investment_confirmed,
            housing_confirmed: page.more.housing_confirmed,
            credit_origins: page
                .more
                .credits
                .iter()
                .map(|credit| {
                    let credit: K1Credit = credit.clone().into();
                    PartnerCreditPersonal {
                        code: credit.code,
                        line: credit.line,
                        carbon_oxide_origin: credit.carbon_oxide_origin,
                        renewable_facility_origin: credit.renewable_facility_origin,
                    }
                })
                .collect(),
            investment_forms: page
                .more
                .investment_property
                .iter()
                .filter_map(|property| {
                    property.own_form().map(|form| PartnerInvestmentPersonal {
                        id: property.id(),
                        own_form: Some(form.clone()),
                    })
                })
                .collect(),
            k3_withholding: match &page.schedule_k3 {
                K3Attached::NotAttached => Vec::new(),
                K3Attached::Attached(k3) => k3.withholding.clone(),
            },
            health_insurance: BusinessHealthInsurance::None,
            health_distributions: 0,
            retirement: PartnerRetirement::None,
        }
    }
    pub(crate) fn from_calculation(record: &PartnerK1) -> Option<Self> {
        let mut personal = Self::from_page(&K1Partnership::from_record("", record)?);
        personal.health_insurance = record.health_insurance.clone();
        personal.health_distributions = record.health_distributions;
        personal.retirement = record.retirement.clone();
        personal.research_credit_line =
            record.research_credit_line.or(personal.research_credit_line);
        Some(personal)
    }
}
fn code_amount(rows: &[K1CodeRow], code: &str) -> i64 {
    rows.iter()
        .filter(|row| row.code == code)
        .filter_map(|row| row.amount)
        .try_fold(0i64, |sum, amount| sum.checked_add(amount))
        .expect("source code total admitted")
}
fn has_code(rows: &[K1CodeRow], code: &str) -> bool {
    rows.iter().any(|row| row.code == code)
}
fn push_once(items: &mut Vec<PartnerK1Item>, item: PartnerK1Item) {
    if !items.contains(&item) {
        items.push(item)
    }
}

impl PartnershipSource {
    pub(crate) fn amt_depletion_adjustment(&self) -> Option<i64> {
        let rows = &self.paper.amt;
        if !crate::k1_depletion_sources::has_depletion(rows) {
            return Some(0);
        }
        // The ordinary adapter has one accepted activity identity. A mixed
        // activity/property paper needs the actual component source partition.
        if self.paper.rental_real_estate.is_some_and(|amount| amount != 0)
            || self.paper.other_rental.is_some_and(|amount| amount != 0)
            || self.paper.royalties.is_some_and(|amount| amount != 0)
            || self.paper.section1231.is_some_and(|amount| amount != 0)
            || !self.statements.investment_property.is_empty()
        {
            return None;
        }
        crate::k1_depletion_sources::printed_depletion(rows)
    }
    pub(crate) fn page(&self, personal: &PartnerPersonalRecords) -> K1Partnership {
        let form = &self.paper;
        let statements = &self.statements;
        let mut credits: Vec<K1CreditPage> = statements
            .credits
            .iter()
            .filter_map(|statement| {
                let line = if statement.code == K1CreditCode::Research {
                    personal.research_credit_line
                } else {
                    statement.line
                };
                if statement.code.needs_line()
                    && !line.is_some_and(|line| statement.code.line_allows(line))
                {
                    return None;
                }
                let amount = statement.amount.or_else(|| {
                    raw_code_amount(&form.credits, credit_code_name(&statement.code))
                })?;
                if amount <= 0 {
                    return None;
                }
                let answer = personal
                    .credit_origins
                    .iter()
                    .find(|answer| answer.code == statement.code && answer.line == line);
                Some(
                    K1Credit {
                        code: statement.code,
                        line,
                        amount,
                        carbon_oxide_origin: answer
                            .and_then(|answer| answer.carbon_oxide_origin.clone()),
                        renewable_facility_origin: answer
                            .and_then(|answer| answer.renewable_facility_origin.clone()),
                    }
                    .into(),
                )
            })
            .collect();
        // A credit without statement-specific line character uses its existing
        // code owner. Statement-dependent codes retain their source boundary.
        for row in &form.credits {
            if let Some(code) = credit_code(&row.code) {
                if statements.credits.iter().any(|credit| credit.code == code) {
                    continue;
                }
                if matches!(
                    code,
                    K1CreditCode::OtherRentalRealEstate
                        | K1CreditCode::OtherRental
                        | K1CreditCode::RenewableElectricity
                        | K1CreditCode::Transferred
                        | K1CreditCode::Other
                ) {
                    continue;
                }
                let line = if code == K1CreditCode::Research {
                    let Some(line) =
                        personal.research_credit_line.filter(|line| code.line_allows(*line))
                    else {
                        continue;
                    };
                    Some(line)
                } else {
                    None
                };
                if let Some(amount) = row.amount.filter(|amount| *amount > 0) {
                    let answer = personal
                        .credit_origins
                        .iter()
                        .find(|answer| answer.code == code && answer.line == line);
                    credits.push(
                        K1Credit {
                            code,
                            line,
                            amount,
                            carbon_oxide_origin: answer
                                .and_then(|answer| answer.carbon_oxide_origin.clone()),
                            renewable_facility_origin: answer
                                .and_then(|answer| answer.renewable_facility_origin.clone()),
                        }
                        .into(),
                    );
                }
            }
        }
        let mut properties = statements.investment_property.clone();
        for property in &mut properties {
            *property.own_form_mut() = personal
                .investment_forms
                .iter()
                .find(|answer| answer.id > 0 && answer.id == property.id())
                .and_then(|answer| answer.own_form.clone());
        }
        let income_totals = self.income_totals();
        // A character-only statement contributes no separate source amount.
        // Its calculated paper total belongs only to the calculation record.
        let income = income_totals
            .iter()
            .filter(|item| {
                !statements
                    .income
                    .iter()
                    .any(|statement| statement.code == item.code && statement.amount.is_none())
            })
            .map(|item| PartnerIncome { code: item.code, amount: item.amount })
            .collect();
        let deductions = form
            .deductions
            .iter()
            .filter_map(|row| {
                Some(PartnerDeduction { code: deduction_code(&row.code)?, amount: row.amount? })
            })
            .collect();
        let mut unentered = Vec::new();
        if form.dividend_equivalents.unwrap_or(0) != 0 {
            push_once(&mut unentered, PartnerK1Item::OtherIncome)
        }
        for situation in &statements.situations {
            push_once(
                &mut unentered,
                match situation {
                    PartnershipStatementSituation::RentalDisposition => {
                        PartnerK1Item::RentalDisposition
                    }
                    PartnershipStatementSituation::OtherIncome => PartnerK1Item::OtherIncome,
                    PartnershipStatementSituation::OtherInformation => {
                        PartnerK1Item::OtherInformation
                    }
                    PartnershipStatementSituation::CooperativeQbi => PartnerK1Item::CooperativeQbi,
                    PartnershipStatementSituation::MultipleActivities => {
                        PartnerK1Item::MultipleActivities
                    }
                },
            );
        }
        for row in &form.income {
            if !income_totals.iter().any(|item| income_code(item.code) == row.code) {
                push_once(&mut unentered, PartnerK1Item::OtherIncome)
            }
        }
        if has_code(&form.deductions, "O") {
            push_once(&mut unentered, PartnerK1Item::OtherDeductions)
        }
        for row in &form.credits {
            if row.code == "AX" {
                push_once(&mut unentered, PartnerK1Item::CreditRecapture)
            } else if row.code != "O"
                && row.code != "E"
                // A printed zero credit leaves nothing out of the estimate.
                && row.amount != Some(0)
                && !(row.code == "M" && row.amount.is_none_or(|amount| amount >= 0))
                && !credits.iter().any(|credit| {
                    credit_code_name(&K1Credit::from(credit.clone()).code) == row.code
                })
            {
                push_once(&mut unentered, PartnerK1Item::Credits)
            }
        }
        if form.amt.iter().any(|row| {
            !matches!(row.code.as_str(), "A" | "B")
                && !(row.code == "C"
                    && (matches!(personal.amt.as_ref(), Some(PartnerAmtRecords::Separate { .. }))
                        || self.amt_depletion_adjustment().is_some()))
        }) {
            push_once(&mut unentered, PartnerK1Item::AmtItems)
        }
        if form.distributions.iter().any(|row| !matches!(row.code.as_str(), "A" | "F")) {
            push_once(&mut unentered, PartnerK1Item::PropertyDistribution)
        }
        for row in &form.information {
            match row.code.as_str() {
                "T" => push_once(&mut unentered, PartnerK1Item::Depletion),
                "A" | "B" if row.amount.is_none_or(|amount| amount != 0) => {
                    push_once(&mut unentered, PartnerK1Item::OtherInformation)
                }
                "A" | "B" | "C" | "D" | "E" | "L" | "Z" | "AB" | "AC" | "AD" | "AI" | "AQ"
                | "AV" => {}
                // Informational codes already excluded by the existing owner.
                "U" | "AA" | "AJ" | "AN" | "AW" => {}
                _ => push_once(&mut unentered, PartnerK1Item::OtherInformation),
            }
        }
        if form.multiple_at_risk
            || form.multiple_passive
            || (has_code(&form.self_employment, "B") && has_code(&form.self_employment, "C"))
        {
            push_once(&mut unentered, PartnerK1Item::MultipleActivities)
        }
        let rentals = K1RentalBoxes {
            real_estate: form.rental_real_estate.unwrap_or(0),
            other: form.other_rental.unwrap_or(0),
            real_estate_section1231: statements.rentals.real_estate_section1231,
            real_estate_unrecaptured: statements.rentals.real_estate_unrecaptured,
            other_section1231: statements.rentals.other_section1231,
            other_unrecaptured: statements.rentals.other_unrecaptured,
            participation: personal.rental_participation,
            real_estate_qbi: statements.rentals.real_estate_qbi.clone(),
            other_qbi: statements.rentals.other_qbi.clone(),
            carryover_active: personal.rental_carryover_active,
            disposed: personal.rental_disposed.clone(),
        };
        let k3 = statements
            .schedule_k3
            .as_ref()
            .map(|source| source.native(&personal.k3_withholding).pruned());
        let allowed_investment_confirmations = crate::k1_page::investment_choices(&properties);
        let more = K1PartnershipOtherBoxes {
            calls: K1PartnershipCalls::from_credits(
                &credits.iter().map(K1CreditCallBox::from).collect::<Vec<_>>(),
            ),
            interest: form.interest.unwrap_or(0),
            dividends: form.dividends.unwrap_or(0),
            qualified_dividends: form.qualified_dividends.unwrap_or(0),
            royalties: form.royalties.unwrap_or(0),
            short: form.short.unwrap_or(0),
            long: form.long.unwrap_or(0),
            collectibles: form.collectibles.unwrap_or(0),
            unrecaptured_partnership: statements.property.unrecaptured_partnership,
            unrecaptured_fund: statements.property.unrecaptured_fund,
            unrecaptured_business: statements.property.unrecaptured_business,
            section1231: statements.property.section1231_business,
            section179_sales: statements.section179_sales.clone(),
            section179: form.section179.unwrap_or(0),
            section179_carryover: personal.section179_carryover,
            section179_carryover_sold: personal.section179_carryover_sold,
            cash_charity: code_amount(&form.deductions, "A"),
            medical_insurance: code_amount(&form.deductions, "M"),
            pensions: code_amount(&form.deductions, "R"),
            income,
            deductions,
            backup_withholding: code_amount(&form.credits, "O"),
            credits,
            investment_property: properties,
            allowed_investment_confirmations,
            investment_confirmed: personal.investment_confirmed,
            housing_confirmed: personal.housing_confirmed,
            exempt_interest: code_amount(&form.exempt, "A"),
            exempt_other: code_amount(&form.exempt, "B"),
            nondeductible: code_amount(&form.exempt, "C"),
            fuel_credit: personal.fuel_credit.clone(),
            prior_fuel_credit: personal.prior_fuel_credit.clone(),
            foreign_taxes: form.foreign_taxes.unwrap_or(0),
            deferred_compensation_tax: code_amount(&form.information, "AI"),
            crp_payments: code_amount(&form.information, "AQ"),
            sale_ordinary: code_amount(&form.information, "AB"),
            sale_collectibles: code_amount(&form.information, "AC"),
            sale_unrecaptured: code_amount(&form.information, "AD"),
            term: personal.term,
            trading: personal.trading,
            unentered,
            amt_section179_income_carry: personal.amt_section179_income_carry.clone(),
        };
        K1Partnership {
            printed: None,
            name: form.partnership.name.clone(),
            reporting_period: form.reporting_period,
            publicly_traded: form.publicly_traded,
            final_k1: form.final_k1,
            participation: personal.participation.clone(),
            limited_for_se: personal.limited_for_se,
            liabilities_start: form.liabilities.beginning_total(),
            liabilities_end: form.liabilities.ending_total(),
            ordinary: form.ordinary.unwrap_or(0),
            rentals,
            services: form.services.unwrap_or(0),
            capital: form.capital.unwrap_or(0),
            se: code_amount(&form.self_employment, "A"),
            se_gross: if has_code(&form.self_employment, "B") {
                SeGross::Farming { amount: code_amount(&form.self_employment, "B") }
            } else if has_code(&form.self_employment, "C") {
                SeGross::Nonfarm { amount: code_amount(&form.self_employment, "C") }
            } else {
                SeGross::None
            },
            distributions: code_amount(&form.distributions, "A")
                .checked_add(code_amount(&form.distributions, "F"))
                .expect("source distribution total admitted"),
            qbi: statements.qbi.clone(),
            qbi_records: Some(personal.qbi_records.clone()),
            more,
            schedule_k3: k3.map_or(K3Attached::NotAttached, K3Attached::Attached),
            basis: (&personal.basis).into(),
            income_source: personal.income_source.clone(),
            amt: personal.amt.clone(),
            disposition: personal.disposition.clone(),
            special: personal.special.clone(),
            qbi_group: personal.qbi_group.clone(),
        }
    }
    pub(crate) fn income_totals(&self) -> Vec<PartnerIncomeTotal> {
        let form = &self.paper;
        let statements = &self.statements;
        let mut income: Vec<PartnerIncomeTotal> = statements
            .income
            .iter()
            .filter_map(|statement| {
                Some(PartnerIncomeTotal {
                    code: statement.code,
                    amount: statement
                        .amount
                        .or_else(|| raw_code_amount(&form.income, income_code(statement.code)))?,
                })
            })
            .collect();
        for row in &form.income {
            if income.iter().any(|item| income_code(item.code) == row.code) {
                continue;
            }
            let code = match row.code.as_str() {
                "A" if row.amount.is_some_and(|amount| amount >= 0) => {
                    Some(PartnerIncomeCode::PortfolioIncome)
                }
                "C" => Some(PartnerIncomeCode::Section1256),
                "E" => Some(PartnerIncomeCode::DebtCancellation),
                "J" => Some(PartnerIncomeCode::TaxBenefitRecovery),
                "R" => Some(PartnerIncomeCode::SpeciallyAllocatedOrdinary),
                _ => None,
            };
            if let (Some(code), Some(amount)) = (code, row.amount) {
                income.push(PartnerIncomeTotal { code, amount })
            }
        }
        income
    }
    fn income_issue(&self) -> Option<crate::validation::ValidationIssue> {
        PartnerIncomeTotal::issue(&self.income_totals(), "income")
            .map(|issue| issue.in_field("more"))
    }
    pub(crate) fn calculation(&self, personal: &PartnerPersonalRecords) -> PartnerK1 {
        let mut record = self.page(personal).record_with_parent_amt_convention(false);
        record.income = self.income_totals();
        record.health_insurance = personal.health_insurance.clone();
        record.health_distributions = personal.health_distributions;
        record.retirement = personal.retirement.clone();
        record.research_credit_line = personal.research_credit_line;
        record.amt_record_choice =
            personal.amt.as_ref().map(|amt| matches!(amt, PartnerAmtRecords::Separate { .. }));
        if matches!(personal.amt, Some(PartnerAmtRecords::SameAsRegular)) {
            record.alternative = record.alternative.and_then(|mut alternative| {
                alternative.ordinary =
                    alternative.ordinary.checked_add(self.amt_depletion_adjustment()?)?;
                Some(alternative)
            });
        }
        record.source = Some(Box::new(self.clone()));
        record
    }
}
pub(crate) fn income_code(code: PartnerIncomeCode) -> &'static str {
    match code {
        PartnerIncomeCode::PortfolioIncome => "A",
        PartnerIncomeCode::Section1256 => "C",
        PartnerIncomeCode::DebtCancellation => "E",
        PartnerIncomeCode::BasisAdjustmentIncome => "F",
        PartnerIncomeCode::TaxBenefitRecovery => "J",
        PartnerIncomeCode::GamblingWinnings => "K",
        PartnerIncomeCode::GamblingLosses => "K",
        PartnerIncomeCode::PreferredStockShort => "Q",
        PartnerIncomeCode::PreferredStockLong => "Q",
        PartnerIncomeCode::SpeciallyAllocatedOrdinary => "R",
        PartnerIncomeCode::NonportfolioShort => "S",
        PartnerIncomeCode::NonportfolioLong => "S",
    }
}
fn deduction_code(code: &str) -> Option<PartnerDeductionCode> {
    Some(match code {
        "B" => PartnerDeductionCode::CashThirty,
        "C" => PartnerDeductionCode::NoncashFifty,
        "D" => PartnerDeductionCode::NoncashThirty,
        "E" => PartnerDeductionCode::CapitalGainPropertyThirty,
        "F" => PartnerDeductionCode::CapitalGainPropertyTwenty,
        "G" => PartnerDeductionCode::ContributionsHundred,
        "H" => PartnerDeductionCode::InvestmentInterest,
        "I" => PartnerDeductionCode::RoyaltyDeductions,
        "J" => PartnerDeductionCode::Section59e,
        "K" => PartnerDeductionCode::ExcessBusinessInterest,
        "L" => PartnerDeductionCode::PortfolioOther,
        "N" => PartnerDeductionCode::EducationalAssistance,
        "P" => PartnerDeductionCode::Preproductive,
        "S" => PartnerDeductionCode::Reforestation,
        "V" => PartnerDeductionCode::BasisAdjustmentDeduction,
        "W" => PartnerDeductionCode::SoilWater,
        "X" => PartnerDeductionCode::Production,
        "Y" => PartnerDeductionCode::Barriers,
        "Z" => PartnerDeductionCode::Itemized,
        "AA" => PartnerDeductionCode::ConstructionFund,
        "AB" => PartnerDeductionCode::EarlyWithdrawalPenalty,
        "AC" => PartnerDeductionCode::DistributionInterest,
        "AD" => PartnerDeductionCode::WorkingInterest,
        "AE" => PartnerDeductionCode::PortfolioDeductions,
        "ZZ" => PartnerDeductionCode::Other,
        _ => return None,
    })
}
pub(crate) fn credit_code(code: &str) -> Option<K1CreditCode> {
    Some(match code {
        "J" => K1CreditCode::WorkOpportunity,
        "N" => K1CreditCode::EmployerTipFica,
        "AE" => K1CreditCode::PensionStartup,
        "AF" => K1CreditCode::AutoEnrollment,
        "AG" => K1CreditCode::MilitarySpouse,
        "BA" => K1CreditCode::SmallEmployerHealth,
        "BB" => K1CreditCode::PaidLeave,
        "C" => K1CreditCode::LowIncomeHousingJ5,
        "D" => K1CreditCode::LowIncomeHousingOther,
        "A" => K1CreditCode::ZeroEmissionNuclear,
        "B" => K1CreditCode::AdvancedNuclear,
        "F" => K1CreditCode::OtherRentalRealEstate,
        "G" => K1CreditCode::OtherRental,
        "K" => K1CreditCode::DisabledAccess,
        "L" => K1CreditCode::EmpowermentZone,
        "M" => K1CreditCode::Research,
        "P" => K1CreditCode::CoalGasificationCooperative,
        "Q" => K1CreditCode::AdvancedEnergyCooperative,
        "R" => K1CreditCode::ManufacturingInvestmentCooperative,
        "S" => K1CreditCode::CleanElectricityCooperative,
        "T" => K1CreditCode::EnergyCooperative,
        "U" => K1CreditCode::RehabilitationCooperative,
        "V" => K1CreditCode::AdvancedManufacturingProduction,
        "W" => K1CreditCode::CleanElectricityProduction,
        "X" => K1CreditCode::CleanFuelProduction,
        "Y" => K1CreditCode::CleanHydrogen,
        "Z" => K1CreditCode::OrphanDrug,
        "AA" => K1CreditCode::EnhancedOilRecovery,
        "AB" => K1CreditCode::RenewableElectricity,
        "AC" => K1CreditCode::Biodiesel,
        "AD" => K1CreditCode::NewMarkets,
        "AH" => K1CreditCode::EmployerChildcare,
        "AI" => K1CreditCode::LowSulfurDiesel,
        "AJ" => K1CreditCode::RailroadTrack,
        "AK" => K1CreditCode::MarginalWells,
        "AL" => K1CreditCode::DistilledSpirits,
        "AM" => K1CreditCode::EnergyEfficientHome,
        "AO" => K1CreditCode::RefuelingProperty,
        "AV" => K1CreditCode::DifferentialWage,
        "AW" => K1CreditCode::CarbonOxide,
        "AY" => K1CreditCode::CleanVehicle,
        "AZ" => K1CreditCode::CommercialCleanVehicle,
        "BC" => K1CreditCode::Transferred,
        "ZZ" => K1CreditCode::Other,
        _ => return None,
    })
}
pub(crate) fn credit_code_name(code: &K1CreditCode) -> &'static str {
    match code {
        K1CreditCode::WorkOpportunity => "J",
        K1CreditCode::EmployerTipFica => "N",
        K1CreditCode::PensionStartup => "AE",
        K1CreditCode::AutoEnrollment => "AF",
        K1CreditCode::MilitarySpouse => "AG",
        K1CreditCode::SmallEmployerHealth => "BA",
        K1CreditCode::PaidLeave => "BB",
        K1CreditCode::LowIncomeHousingJ5 => "C",
        K1CreditCode::LowIncomeHousingOther => "D",
        K1CreditCode::ZeroEmissionNuclear => "A",
        K1CreditCode::AdvancedNuclear => "B",
        K1CreditCode::OtherRentalRealEstate => "F",
        K1CreditCode::OtherRental => "G",
        K1CreditCode::DisabledAccess => "K",
        K1CreditCode::EmpowermentZone => "L",
        K1CreditCode::Research => "M",
        K1CreditCode::CoalGasificationCooperative => "P",
        K1CreditCode::AdvancedEnergyCooperative => "Q",
        K1CreditCode::ManufacturingInvestmentCooperative => "R",
        K1CreditCode::CleanElectricityCooperative => "S",
        K1CreditCode::EnergyCooperative => "T",
        K1CreditCode::RehabilitationCooperative => "U",
        K1CreditCode::AdvancedManufacturingProduction => "V",
        K1CreditCode::CleanElectricityProduction => "W",
        K1CreditCode::CleanFuelProduction => "X",
        K1CreditCode::CleanHydrogen => "Y",
        K1CreditCode::OrphanDrug => "Z",
        K1CreditCode::EnhancedOilRecovery => "AA",
        K1CreditCode::RenewableElectricity => "AB",
        K1CreditCode::Biodiesel => "AC",
        K1CreditCode::NewMarkets => "AD",
        K1CreditCode::EmployerChildcare => "AH",
        K1CreditCode::LowSulfurDiesel => "AI",
        K1CreditCode::RailroadTrack => "AJ",
        K1CreditCode::MarginalWells => "AK",
        K1CreditCode::DistilledSpirits => "AL",
        K1CreditCode::EnergyEfficientHome => "AM",
        K1CreditCode::RefuelingProperty => "AO",
        K1CreditCode::DifferentialWage => "AV",
        K1CreditCode::CarbonOxide => "AW",
        K1CreditCode::CleanVehicle => "AY",
        K1CreditCode::CommercialCleanVehicle => "AZ",
        K1CreditCode::Transferred => "BC",
        K1CreditCode::Other => "ZZ",
    }
}

impl K1PartnershipDocument {
    pub(crate) fn research_credit_needed(&self) -> bool {
        self.form
            .credits
            .iter()
            .any(|row| row.code == "M" && row.amount.is_none_or(|amount| amount > 0))
            || self.statements.credits.iter().any(|detail| {
                detail.code == K1CreditCode::Research
                    && detail
                        .amount
                        .or_else(|| raw_code_amount(&self.form.credits, "M"))
                        .is_none_or(|amount| amount > 0)
            })
    }
    pub(crate) fn source(&self) -> PartnershipSource {
        let mut statements = self.statements.clone();
        // Source facts do not acquire personal answers from hidden document echoes.
        for property in &mut statements.investment_property {
            *property.own_form_mut() = None;
        }
        for credit in &mut statements.credits {
            if credit.code == K1CreditCode::Research {
                credit.line = None;
            }
        }
        PartnershipSource {
            paper: self.form.clone(),
            statements,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        }
    }
    pub(crate) fn calculation_page(&self) -> K1Partnership {
        self.source().page(&self.personal.clone().unwrap_or_default())
    }
    pub(crate) fn replace_personal_page(&mut self, page: &K1Partnership) {
        let mut personal = PartnerPersonalRecords::from_page(page);
        if let Some(old) = &self.personal {
            personal.depletion_sources = old.depletion_sources.clone();
            personal.health_insurance = old.health_insurance.clone();
            personal.health_distributions = old.health_distributions;
            personal.retirement = old.retirement.clone();
            personal.research_credit_line = old.research_credit_line;
        }
        self.personal = Some(personal);
    }
    pub(crate) fn record(&self) -> PartnerK1 {
        self.source().calculation(&self.personal.clone().unwrap_or_default())
    }
    pub(crate) fn from_record(_name: &str, record: &PartnerK1) -> Option<Self> {
        let mut source = record.source.as_deref()?.clone();
        source.bind_accepted_child_ids(record);
        Some(Self {
            form: source.paper,
            statements: source.statements,
            personal: Some(PartnerPersonalRecords::from_calculation(record)?),
        })
    }
    pub(crate) fn printed_admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        credit_statement_issue(
            self.statements
                .credits
                .iter()
                .enumerate()
                .filter(|(_, statement)| statement.code != K1CreditCode::Research),
        )
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.source().admission_issue() {
            return Some(issue);
        }
        if require_answers && let Some(issue) = self.source().required_issue() {
            return Some(issue);
        }
        if require_answers
            && let Some(issue) = self.source().amt_records_issue(
                self.personal.as_ref().and_then(|personal| personal.amt.as_ref()),
            )
        {
            return Some(issue);
        }
        self.source()
            .income_issue()
            .or_else(|| self.calculation_page().issue_with_mode(require_answers))
            .map(|issue| if require_answers { issue } else { self.source_issue_path(issue) })
    }
    pub(crate) fn topic_issue(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.source().admission_issue() {
            return Some(issue);
        }
        if require_answers && let Some(issue) = self.source().required_issue() {
            return Some(issue);
        }
        if let Some(issue) = self
            .source()
            .amt_records_issue(self.personal.as_ref().and_then(|personal| personal.amt.as_ref()))
        {
            return Some(issue);
        }
        self.source()
            .income_issue()
            .or_else(|| self.calculation_page().topic_issue(require_answers))
    }
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    fn source_issue_path(
        &self,
        issue: crate::validation::ValidationIssue,
    ) -> crate::validation::ValidationIssue {
        let mappings: &[(&[&'static str], &[&'static str])] = &[
            (&["ordinary"], &["form", "ordinary"]),
            (&["services"], &["form", "services"]),
            (&["capital"], &["form", "capital"]),
            (&["more", "qualified_dividends"], &["form", "qualified_dividends"]),
            (&["more", "dividends"], &["form", "dividends"]),
            (&["more", "medical_insurance"], &["form", "deductions"]),
            (&["more", "income"], &["statements", "income"]),
            (&["more", "deductions"], &["form", "deductions"]),
            (&["more", "credits"], &["statements", "credits"]),
            (&["more", "investment_property"], &["statements", "investment_property"]),
            (&["more", "section179_sales"], &["statements", "section179_sales"]),
            (&["qbi"], &["statements", "qbi"]),
            (&["schedule_k3"], &["statements", "schedule_k3"]),
            (&["reporting_period"], &["form", "reporting_period"]),
        ];
        for (from, to) in mappings {
            if issue.has_field_prefix(from) {
                return issue.map_field_prefix(from, to);
            }
        }
        // A personal relationship is corrected on its existing topic row.
        // A document Save does not claim that hidden personal field as input.
        crate::validation::ValidationIssue::new(issue.key)
    }
}
impl PartnershipSource {
    // IDs are issued by the existing native child owner. These are the same
    // submitted rows, in their unchanged source order, before any projection
    // reorders or divides them. Existing nonzero identities remain unchanged.
    pub(crate) fn bind_accepted_child_ids(&mut self, record: &PartnerK1) {
        for (source, accepted) in
            self.statements.investment_property.iter_mut().zip(&record.investment_property)
        {
            if source.id() == 0 {
                *source.id_mut() = accepted.id();
            }
        }
        for (source, accepted) in
            self.statements.section179_sales.iter_mut().zip(&record.section179_sales)
        {
            if source.id == 0 {
                source.id = accepted.id;
            }
        }
    }
}

/// The accepted identity and source facts for one partnership document.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StoredPartnershipDocument {
    pub id: i64,
    pub label: String,
    pub owner: Owner,
    pub community: CommunityShare,
    pub state_source: Option<IncomeStateSource>,
    pub refund_management: Option<RefundManagement>,
    pub source: PartnershipSource,
    pub personal: PartnerPersonalRecords,
}
impl Default for PartnershipStatements {
    fn default() -> Self {
        Self {
            qbi: PartnerQbi::NoStatement,
            rentals: PartnershipRentalStatement::default(),
            property: PartnershipPropertyStatement::default(),
            income: Vec::new(),
            credits: Vec::new(),
            investment_property: Vec::new(),
            section179_sales: Vec::new(),
            schedule_k3: None,
            situations: Vec::new(),
        }
    }
}

pub(crate) fn credit_statement_issue<'a>(
    statements: impl IntoIterator<Item = (usize, &'a PartnershipCreditStatement)>,
) -> Option<crate::validation::ValidationIssue> {
    for (index, statement) in statements {
        if statement.line.is_some_and(|line| {
            statement.code == K1CreditCode::Research
                || !statement.code.needs_line()
                || !statement.code.line_allows(line)
        }) {
            return Some(
                crate::validation::ValidationIssue::new("K1_CREDIT_CODE_TWICE")
                    .at_field("line")
                    .in_list("credits", index)
                    .in_field("statements"),
            );
        }
    }
    None
}

impl PartnershipSource {
    pub(crate) fn printed_names(&self) -> PrintedDocumentNames {
        PrintedDocumentNames {
            recipient: self.paper.partner.clone(),
            issuer: Some(self.paper.partnership.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        }
    }
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let failure = || ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
        for (field, rows) in [
            ("income", &self.paper.income),
            ("deductions", &self.paper.deductions),
            ("self_employment", &self.paper.self_employment),
            ("credits", &self.paper.credits),
            ("amt", &self.paper.amt),
            ("exempt", &self.paper.exempt),
            ("distributions", &self.paper.distributions),
            ("information", &self.paper.information),
        ] {
            let mut totals = alloc::collections::BTreeMap::<&str, i64>::new();
            for (index, row) in rows.iter().enumerate() {
                if let Some(amount) = row.amount {
                    let total = totals.entry(row.code.as_str()).or_default();
                    let Some(sum) = total.checked_add(amount) else {
                        return Some(
                            failure().at_field("amount").in_list(field, index).in_field("form"),
                        );
                    };
                    *total = sum;
                }
            }
            if field == "distributions"
                && totals
                    .get("A")
                    .copied()
                    .unwrap_or(0)
                    .checked_add(totals.get("F").copied().unwrap_or(0))
                    .is_none()
            {
                return Some(failure().at_field(field).in_field("form"));
            }
        }
        for (field, parts) in [
            (
                "beginning",
                [
                    self.paper.liabilities.nonrecourse.beginning.unwrap_or(0),
                    self.paper.liabilities.qualified_nonrecourse.beginning.unwrap_or(0),
                    self.paper.liabilities.recourse.beginning.unwrap_or(0),
                ],
            ),
            (
                "ending",
                [
                    self.paper.liabilities.nonrecourse.ending.unwrap_or(0),
                    self.paper.liabilities.qualified_nonrecourse.ending.unwrap_or(0),
                    self.paper.liabilities.recourse.ending.unwrap_or(0),
                ],
            ),
        ] {
            if parts.into_iter().try_fold(0i64, |sum, amount| sum.checked_add(amount)).is_none() {
                return Some(failure().at_field(field).in_field("liabilities").in_field("form"));
            }
        }
        for (field, parts) in [
            (
                "total_guaranteed",
                vec![self.paper.services.unwrap_or(0), self.paper.capital.unwrap_or(0)],
            ),
            (
                "unrecaptured",
                vec![
                    self.statements.property.unrecaptured_partnership,
                    self.statements.property.unrecaptured_fund,
                    self.statements.property.unrecaptured_business,
                    self.statements.rentals.real_estate_unrecaptured,
                    self.statements.rentals.other_unrecaptured,
                ],
            ),
            (
                "section1231",
                vec![
                    self.statements.property.section1231_business,
                    self.statements.rentals.real_estate_section1231,
                    self.statements.rentals.other_section1231,
                ],
            ),
        ] {
            if parts.into_iter().try_fold(0i64, |sum, amount| sum.checked_add(amount)).is_none() {
                return Some(failure().at_field(field).in_field("form"));
            }
        }
        if self.investment_property_totals().is_none() {
            return Some(failure().at_field("investment_property").in_field("statements"));
        }
        if let Some(issue) = credit_statement_issue(self.statements.credits.iter().enumerate()) {
            return Some(issue);
        }
        if let Some(k3) = &self.statements.schedule_k3
            && let Some(issue) = k3.admission_issue()
        {
            return Some(issue.in_field("schedule_k3").in_field("statements"));
        }
        None
    }
}

/// Source-only projection of the currently supported Schedule K-3 fields.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this Schedule K-3 (Form 1065 or Form 1120-S) and its attached statements exactly. It belongs to the supplied year only when the entity's tax year ends in that year (a calendar-year schedule for that year, or a fiscal year ending in it); report a mismatch as an error. Report illegible or conflicting values as errors, and never infer a value the schedule does not show. A schedule printed on several pages is one form; copies of the same schedule count once. Amounts are U.S. dollars; parentheses or a minus sign mark a negative amount. `countries` has one element for each country code with an amount in Part II line 24 column (b), category ForeignBranch, column (c), category Passive, or column (d), category General: `country` is the two-letter code as printed, `income` that line 24 amount, `qualified` the Part II line 8 amount for the same country and column (0 when none), and `tax` the sum, for that country, of every row of the foreign-tax section line 1 (Form 1065 Part III Section 4, Form 1120-S Part III Section 3) in the same category, column (c) foreign branch, (d) passive or (e) general, adding its U.S., Foreign and Partner amounts. `us_gross` is Part II line 24 column (a) plus column (f). `branch` reads column (b), `passive` column (c) and `general` column (d): `deductions` is the sum of Part II lines 25, 26, 31 through 37 and 44 through 50, `interest` the sum of lines 39 through 43, `reduction` the total of the foreign-tax section's line 2 in the matching tax column, and `capital` the column's line 11 as `short_gain`, line 12 as `long_gain`, line 27 as `short_loss` and line 28 as `long_loss`, each a positive amount. `partner_deductions` is the sum of Part II lines 25, 26, 31, 33 through 37 and 44 through 50 in column (f). `partner_interest`: `business` is line 41 column (f), `passive_activity` line 43 column (f); `business_assets` is Part III Section 2 line 6b and `passive_assets` line 6d, each with `us` columns (a) plus (f), `branch` column (b), `passive` column (c) and `general` column (d), all zero when Section 2 is blank. Return zero for a clearly blank amount. `items` lists each entry at most once: OtherCategories for any amount in Part II column (e) or in the foreign-tax section's section 951A or other-category columns; ForeignRateGroups for any amount in Part II lines 13, 14, 15, 29 or 30 in column (b), (c) or (d); Research for line 32 column (f); OilAndGas when Part I box 2 is checked; Loans when Part I box 10 is checked; DisallowedDeduction when Part I box 6 is checked; ForeignCorporations when item E answers Yes for Part VI or VII of Form 1065, or Part V or VI of Form 1120-S. The imported `countries` and `items` lists replace the existing lists. Do not read Part I boxes 1, 3, 4, 5, 7, 8, 9, 11, 12 or 13, Part III Sections 1, 3 and 5 of Form 1065 or Section 1 of Form 1120-S, the foreign-tax section's line 3, or Parts IV, V and VIII through XIII; each passive country's withholding treatment comes from the person's own records.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct PartnershipK3Source {
    /// Foreign income and taxes by country
    /// One row for each country and category in Part II line 24, with that country's foreign taxes from Part III. The foreign withholding treatment of a passive country is its own answer below. Do not repeat these amounts among the foreign tax credit's own source records.
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Country
    pub countries: Vec<K3Country>,
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
impl From<&ScheduleK3> for PartnershipK3Source {
    fn from(source: &ScheduleK3) -> Self {
        Self {
            countries: source.countries.clone(),
            us_gross: source.us_gross,
            branch: source.branch.clone(),
            passive: source.passive.clone(),
            general: source.general.clone(),
            partner_deductions: source.partner_deductions,
            partner_interest: source.partner_interest.clone(),
            items: source.items.clone(),
        }
    }
}

#[cfg(test)]
pub(crate) fn deduction_code_name(code: PartnerDeductionCode) -> &'static str {
    match code {
        PartnerDeductionCode::CashThirty => "B",
        PartnerDeductionCode::NoncashFifty => "C",
        PartnerDeductionCode::NoncashThirty => "D",
        PartnerDeductionCode::CapitalGainPropertyThirty => "E",
        PartnerDeductionCode::CapitalGainPropertyTwenty => "F",
        PartnerDeductionCode::ContributionsHundred => "G",
        PartnerDeductionCode::InvestmentInterest => "H",
        PartnerDeductionCode::RoyaltyDeductions => "I",
        PartnerDeductionCode::Section59e => "J",
        PartnerDeductionCode::ExcessBusinessInterest => "K",
        PartnerDeductionCode::PortfolioOther => "L",
        PartnerDeductionCode::EducationalAssistance => "N",
        PartnerDeductionCode::Preproductive => "P",
        PartnerDeductionCode::Reforestation => "S",
        PartnerDeductionCode::BasisAdjustmentDeduction => "V",
        PartnerDeductionCode::SoilWater => "W",
        PartnerDeductionCode::Production => "X",
        PartnerDeductionCode::Barriers => "Y",
        PartnerDeductionCode::Itemized => "Z",
        PartnerDeductionCode::ConstructionFund => "AA",
        PartnerDeductionCode::EarlyWithdrawalPenalty => "AB",
        PartnerDeductionCode::DistributionInterest => "AC",
        PartnerDeductionCode::WorkingInterest => "AD",
        PartnerDeductionCode::PortfolioDeductions => "AE",
        PartnerDeductionCode::Other => "ZZ",
    }
}

/// Character supplied by a box 11 attached statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnershipIncomeStatement {
    #[libertas_llm_doc_field]
    pub code: PartnerIncomeCode,
    /// [NullText]
    /// The paper supplies the amount; this statement only supplies character
    /// [SetText]
    /// The statement separately reports
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub amount: Option<i64>,
}
impl From<&PartnerIncome> for PartnershipIncomeStatement {
    fn from(item: &PartnerIncome) -> Self {
        Self { code: item.code, amount: Some(item.amount) }
    }
}
/// Credit line and any distinct amount actually supplied by an attached statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnershipCreditStatement {
    #[libertas_llm_doc_field]
    pub code: K1CreditCode,
    /// Form 3800 line actually reported by the statement
    /// For code M, leave absent: eligible-small-business treatment is your own answer.
    /// [NullText]
    /// No Form 3800 line is printed
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub line: Option<BusinessCredit>,
    /// [NullText]
    /// The paper supplies the amount; this statement only supplies the line
    /// [SetText]
    /// The statement separately reports
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub amount: Option<i64>,
}
impl From<&K1Credit> for PartnershipCreditStatement {
    fn from(item: &K1Credit) -> Self {
        Self {
            code: item.code,
            line: (item.code != K1CreditCode::Research).then_some(item.line).flatten(),
            amount: Some(item.amount),
        }
    }
}

fn raw_code_amount(rows: &[K1CodeRow], code: &str) -> Option<i64> {
    let values: Vec<_> = rows.iter().filter(|row| row.code == code).collect();
    if values.is_empty() || values.iter().any(|row| row.amount.is_none()) {
        return None;
    }
    values.into_iter().try_fold(0i64, |sum, row| sum.checked_add(row.amount?))
}

impl PartnershipSource {
    pub(crate) fn amt_records_issue(
        &self,
        amt: Option<&PartnerAmtRecords>,
    ) -> Option<crate::validation::ValidationIssue> {
        if matches!(amt, Some(PartnerAmtRecords::SameAsRegular))
            && self.paper.amt.iter().any(|row| {
                matches!(row.code.as_str(), "A" | "B")
                    && row.amount.is_some_and(|amount| amount != 0)
            })
        {
            return Some(
                crate::validation::ValidationIssue::new("DOCUMENT_K1_AMT_RECORDS").at_field("amt"),
            );
        }
        None
    }

    // These are the source operands reported by the four existing property
    // variants, before any credit rate, basis reduction or limitation.
    fn investment_property_totals(&self) -> Option<[i64; 4]> {
        let mut totals = [0i64; 4];
        for property in &self.statements.investment_property {
            let (index, amount) = match property {
                K1InvestmentProperty::RehabilitationRental { expenditures, .. } => {
                    (0, expenditures)
                }
                K1InvestmentProperty::RehabilitationOther { expenditures, .. } => (1, expenditures),
                K1InvestmentProperty::Energy { basis, .. } => (2, basis),
                K1InvestmentProperty::CleanElectricity { basis, .. } => (3, basis),
            };
            totals[index] = totals[index].checked_add(*amount)?;
        }
        Some(totals)
    }

    // Absence of an actual statement amount is not a zero allocation. These
    // checks bind source totals to their existing typed statement owners.
    pub(crate) fn required_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let missing = || ValidationIssue::new("PARTNERSHIP_SOURCE_STATEMENT_REQUIRED");
        let mismatch = || ValidationIssue::new("PARTNERSHIP_SOURCE_STATEMENT_MISMATCH");
        let form = &self.paper;
        let statements = &self.statements;
        if has_code(&form.information, "Z") && matches!(statements.qbi, PartnerQbi::NoStatement) {
            return Some(missing().at_field("qbi").in_field("statements"));
        }
        if form.k3_attached && statements.schedule_k3.is_none() {
            return Some(missing().at_field("schedule_k3").in_field("statements"));
        }
        for (index, detail) in statements.income.iter().enumerate() {
            if detail
                .amount
                .or_else(|| raw_code_amount(&form.income, income_code(detail.code)))
                .is_none()
            {
                return Some(
                    missing().at_field("amount").in_list("income", index).in_field("statements"),
                );
            }
        }
        for (index, detail) in statements.credits.iter().enumerate() {
            if detail.code != K1CreditCode::Research
                && detail.code.needs_line()
                && detail.line.is_none()
            {
                return Some(
                    missing().at_field("line").in_list("credits", index).in_field("statements"),
                );
            }
            if detail
                .amount
                .or_else(|| raw_code_amount(&form.credits, credit_code_name(&detail.code)))
                .is_none()
            {
                return Some(
                    missing().at_field("amount").in_list("credits", index).in_field("statements"),
                );
            }
        }
        for (field, rows) in [
            ("deductions", &form.deductions),
            ("self_employment", &form.self_employment),
            ("exempt", &form.exempt),
        ] {
            for (index, row) in rows.iter().enumerate() {
                let supported = if field == "deductions" {
                    matches!(row.code.as_str(), "A" | "M" | "R" | "O")
                        || deduction_code(&row.code).is_some()
                } else {
                    matches!(row.code.as_str(), "A" | "B" | "C")
                };
                if !supported {
                    return Some(
                        ValidationIssue::new("PARTNERSHIP_SOURCE_CODE_NOT_CALCULATED")
                            .at_field("code")
                            .in_list(field, index)
                            .in_field("form"),
                    );
                }
            }
        }
        let Some(property_totals) = self.investment_property_totals() else {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("investment_property")
                    .in_field("statements"),
            );
        };
        for (index, (rows, code)) in [
            (&form.credits, "E"),
            (&form.information, "D"),
            (&form.information, "E"),
            (&form.information, "AV"),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(printed) = raw_code_amount(rows, code)
                && printed != property_totals[index]
            {
                return Some(mismatch().at_field("investment_property").in_field("statements"));
            }
        }
        if form.services.unwrap_or(0).checked_add(form.capital.unwrap_or(0))
            != Some(form.total_guaranteed.unwrap_or(0))
        {
            return Some(mismatch().at_field("total_guaranteed").in_field("form"));
        }
        for (field, printed, parts) in [
            (
                "unrecaptured",
                form.unrecaptured.unwrap_or(0),
                vec![
                    statements.property.unrecaptured_partnership,
                    statements.property.unrecaptured_fund,
                    statements.property.unrecaptured_business,
                    statements.rentals.real_estate_unrecaptured,
                    statements.rentals.other_unrecaptured,
                ],
            ),
            (
                "section1231",
                form.section1231.unwrap_or(0),
                vec![
                    statements.property.section1231_business,
                    statements.rentals.real_estate_section1231,
                    statements.rentals.other_section1231,
                ],
            ),
        ] {
            if parts.into_iter().try_fold(0i64, |sum, amount| sum.checked_add(amount))
                != Some(printed)
            {
                return Some(mismatch().at_field(field).in_field("form"));
            }
        }
        for (field, rows) in [
            ("deductions", &form.deductions),
            ("self_employment", &form.self_employment),
            ("amt", &form.amt),
            ("exempt", &form.exempt),
            ("distributions", &form.distributions),
            ("information", &form.information),
        ] {
            for (index, row) in rows.iter().enumerate() {
                let requires_amount = match field {
                    "deductions" => {
                        matches!(row.code.as_str(), "A" | "M" | "R")
                            || deduction_code(&row.code).is_some()
                    }
                    "self_employment" => matches!(row.code.as_str(), "A" | "B" | "C"),
                    "amt" => matches!(row.code.as_str(), "A" | "B"),
                    "exempt" => matches!(row.code.as_str(), "A" | "B" | "C"),
                    "distributions" => matches!(row.code.as_str(), "A" | "F"),
                    _ => matches!(row.code.as_str(), "A" | "B" | "AI" | "AQ" | "AB" | "AC" | "AD"),
                };
                if requires_amount && row.amount.is_none() {
                    return Some(
                        missing().at_field("amount").in_list(field, index).in_field("form"),
                    );
                }
            }
        }
        for (index, row) in form.income.iter().enumerate() {
            let details: Vec<_> = statements
                .income
                .iter()
                .filter(|detail| income_code(detail.code) == row.code)
                .collect();
            if details.is_empty() {
                if matches!(row.code.as_str(), "A" | "C" | "E" | "J" | "R") && row.amount.is_none()
                {
                    return Some(
                        missing().at_field("amount").in_list("income", index).in_field("form"),
                    );
                }
                continue;
            }
            if details.len() > 1 && details.iter().any(|detail| detail.amount.is_none()) {
                return Some(missing().at_field("income").in_field("statements"));
            }
            let resolved: Option<Vec<i64>> = details
                .into_iter()
                .map(|detail| detail.amount.or_else(|| raw_code_amount(&form.income, &row.code)))
                .collect();
            let Some(resolved) = resolved else {
                return Some(missing().at_field("income").in_field("statements"));
            };
            if row.code != "K"
                && let Some(printed) = raw_code_amount(&form.income, &row.code)
                && resolved.into_iter().try_fold(0i64, |sum, amount| sum.checked_add(amount))
                    != Some(printed)
            {
                return Some(mismatch().at_field("income").in_field("statements"));
            }
        }
        for (index, row) in form.credits.iter().enumerate() {
            if row.code == "O" {
                if row.amount.is_none() {
                    return Some(
                        missing().at_field("amount").in_list("credits", index).in_field("form"),
                    );
                }
                continue;
            }
            if row.code == "E" {
                if !statements.investment_property.iter().any(|property| {
                    matches!(property, K1InvestmentProperty::RehabilitationRental { .. })
                }) {
                    return Some(missing().at_field("investment_property").in_field("statements"));
                }
                continue;
            }
            let Some(code) = credit_code(&row.code) else { continue };
            let details: Vec<_> =
                statements.credits.iter().filter(|detail| detail.code == code).collect();
            if details.is_empty() && code != K1CreditCode::Research && code.needs_line() {
                return Some(missing().at_field("credits").in_field("statements"));
            }
            if details.iter().any(|detail| {
                code != K1CreditCode::Research && code.needs_line() && detail.line.is_none()
            }) || (details.len() > 1 && details.iter().any(|detail| detail.amount.is_none()))
            {
                return Some(missing().at_field("credits").in_field("statements"));
            }
            let resolved: Option<Vec<i64>> = details
                .into_iter()
                .map(|detail| detail.amount.or_else(|| raw_code_amount(&form.credits, &row.code)))
                .collect();
            if row.amount.is_none() && resolved.as_ref().is_none_or(|values| values.is_empty()) {
                return Some(
                    missing().at_field("amount").in_list("credits", index).in_field("form"),
                );
            }
            if let (Some(printed), Some(resolved)) =
                (raw_code_amount(&form.credits, &row.code), resolved)
                && !resolved.is_empty()
                && resolved.into_iter().try_fold(0i64, |sum, amount| sum.checked_add(amount))
                    != Some(printed)
            {
                return Some(mismatch().at_field("credits").in_field("statements"));
            }
        }
        for row in &form.information {
            let found = match row.code.as_str() {
                "D" => statements.investment_property.iter().any(|property| {
                    matches!(property, K1InvestmentProperty::RehabilitationOther { .. })
                }),
                "E" => statements
                    .investment_property
                    .iter()
                    .any(|property| matches!(property, K1InvestmentProperty::Energy { .. })),
                "AV" => statements.investment_property.iter().any(|property| {
                    matches!(property, K1InvestmentProperty::CleanElectricity { .. })
                }),
                "L" => !statements.section179_sales.is_empty(),
                _ => true,
            };
            if !found {
                return Some(missing().at_field("information").in_field("form"));
            }
        }
        self.income_issue()
    }
}
