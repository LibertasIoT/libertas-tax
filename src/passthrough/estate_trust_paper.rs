// Schedule K-1 (Form 1041), official 2026 draft created May 22, 2026.
// https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf
use crate::*;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Code and amount as printed on Schedule K-1 (Form 1041)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct EstateTrustCodeRow {
    /// Code
    #[libertas_llm_doc_field]
    // IRS K-1 codes have one or two letters; an attached statement adds * or uses * alone.
    // https://www.irs.gov/instructions/i1041
    #[libertas_pattern(r"[A-Z]{1,2}\*?|\*")]
    pub code: String,
    /// Amount
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
}

/// Schedule K-1 (Form 1041) — Beneficiary's Share of Income, Deductions, Credits, etc.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1EstateTrustPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub tax_year: u32,
    /// Tax year beginning
    /// [NullText]
    /// No beginning date printed
    /// [SetText]
    /// The printed beginning date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub began: Option<u32>,
    /// Tax year ending
    /// [NullText]
    /// No ending date printed
    /// [SetText]
    /// The printed ending date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub ended: Option<u32>,
    /// Final K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_k1: bool,
    /// Amended K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amended_k1: bool,
    /// A and B — Estate's or trust's employer identification number and name
    #[libertas_llm_doc_field]
    pub estate_trust: PrintedParty,
    /// C — Fiduciary's name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub fiduciary: PrintedPartyNameAddress,
    /// D — Check if Form 1041-T was filed
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub filed_1041t: bool,
    /// D — Date it was filed
    /// [NullText]
    /// No filing date printed
    /// [SetText]
    /// The printed filing date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub filed_1041t_on: Option<u32>,
    /// E — Check if this is the final Form 1041 for the estate or trust
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_form1041: bool,
    /// F and G — Beneficiary's identifying number, name, address, city, state, and ZIP code
    #[libertas_llm_doc_field]
    pub beneficiary: PrintedParty,
    /// H — Domestic beneficiary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub domestic_beneficiary: bool,
    /// H — Foreign beneficiary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub foreign_beneficiary: bool,
    /// 1 — Interest income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// 2a — Ordinary dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub ordinary_dividends: i64,
    /// 2b — Qualified dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified_dividends: i64,
    /// 3 — Net short-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub short_gain: i64,
    /// 4a — Net long-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub long_gain: i64,
    /// 4b — 28% rate gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rate_28_gain: i64,
    /// 4c — Unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured_gain: i64,
    /// 5 — Other portfolio and nonbusiness income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub portfolio_income: i64,
    /// 6 — Ordinary business income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub business_income: i64,
    /// 7 — Net rental real estate income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_income: i64,
    /// 8 — Other rental income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_income: i64,
    /// 9 — Directly apportioned deductions
    #[libertas_llm_doc_field]
    pub deductions: Vec<EstateTrustCodeRow>,
    /// 10 — Estate tax deduction
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub estate_tax_deduction: i64,
    /// 11 — Final year deductions
    #[libertas_llm_doc_field]
    pub final_deductions: Vec<EstateTrustCodeRow>,
    /// 12 — Alternative minimum tax adjustment
    #[libertas_llm_doc_field]
    pub amt: Vec<EstateTrustCodeRow>,
    /// 13 — Credits and credit recapture
    #[libertas_llm_doc_field]
    pub credits: Vec<EstateTrustCodeRow>,
    /// 14 — Other information
    #[libertas_llm_doc_field]
    pub information: Vec<EstateTrustCodeRow>,
}

pub(crate) fn estate_code_amount(rows: &[EstateTrustCodeRow], code: &str) -> i64 {
    checked_estate_code_amount(rows, code).expect("admitted estate K-1 coded amount")
}

fn checked_estate_code_amount(
    rows: &[EstateTrustCodeRow],
    code: &str,
) -> Result<i64, crate::validation::ValidationIssue> {
    crate::kernel::ratio::checked_sum(
        rows.iter().filter(|row| row.code == code).map(|row| row.amount),
    )
    .ok_or_else(|| {
        estate_amount_issue().at_field("amount").in_list(
            "rows",
            rows.iter().position(|row| row.code == code).expect("coded amount source"),
        )
    })
}

pub(crate) fn estate_amount_issue() -> crate::validation::ValidationIssue {
    crate::validation::ValidationIssue::new(
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
    )
    .at_record()
}

pub(crate) fn estate_amount_residual(
    total: i64,
    amounts: impl IntoIterator<Item = i64>,
) -> Result<i64, crate::validation::ValidationIssue> {
    use crate::kernel::ratio::exact;
    let residual =
        exact::whole(total) - amounts.into_iter().map(exact::whole).sum::<exact::Exact>();
    exact::rounded(&residual).ok_or_else(estate_amount_issue)
}

impl K1EstateTrustPaper {
    fn calendar_period(&self) -> Option<(u32, u32)> {
        Some((
            self.tax_year.checked_mul(10000)?.checked_add(101)?,
            self.tax_year.checked_mul(10000)?.checked_add(1231)?,
        ))
    }
    /// The printed calendar-year header or actual fiscal/short-year dates.
    pub(crate) fn reported_period(&self) -> Option<FinalEstateTrustLossPeriod> {
        match (self.began, self.ended) {
            (Some(began), Some(ended)) => Some(FinalEstateTrustLossPeriod { began, ended }),
            (None, None) if self.tax_year != 0 => self
                .calendar_period()
                .map(|(began, ended)| FinalEstateTrustLossPeriod { began, ended }),
            _ => None,
        }
    }

    pub(crate) fn checked_record(&self) -> Result<Form1041K1, crate::validation::ValidationIssue> {
        let paper = self;
        let K1EstateTrustPaper {
            tax_year: _,
            began: _,
            ended: _,
            final_k1: _,
            amended_k1: _,
            estate_trust,
            fiduciary: _,
            filed_1041t: _,
            filed_1041t_on: _,
            final_form1041,
            beneficiary: _,
            domestic_beneficiary: _,
            foreign_beneficiary: _,
            interest,
            ordinary_dividends,
            qualified_dividends,
            short_gain,
            long_gain,
            rate_28_gain,
            unrecaptured_gain,
            portfolio_income,
            business_income: _,
            rental_real_estate_income: _,
            other_rental_income: _,
            deductions,
            estate_tax_deduction,
            final_deductions,
            amt,
            credits,
            information,
        } = paper;
        Ok(Form1041K1 {
            ein: estate_trust.federal_id.clone(),
            interest: *interest,
            ordinary_dividends: *ordinary_dividends,
            qualified_dividends: *qualified_dividends,
            short_gain: *short_gain,
            long_gain: *long_gain,
            rate_28_gain: *rate_28_gain,
            unrecaptured_gain: *unrecaptured_gain,
            royalty_income: *portfolio_income,
            box_9_royalty_depreciation: checked_estate_code_amount(deductions, "A")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["deductions"]))?,
            box_9_royalty_depletion: checked_estate_code_amount(deductions, "B")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["deductions"]))?,
            box_9_royalty_amortization: checked_estate_code_amount(deductions, "C")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["deductions"]))?,
            gross_farming_fishing_income: checked_estate_code_amount(information, "F")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["information"]))?,
            box_10_interest_ird_estate_tax: *estate_tax_deduction,
            final_67e_deduction: checked_estate_code_amount(final_deductions, "A")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["final_deductions"]))?,
            final_short_carry: checked_estate_code_amount(final_deductions, "C")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["final_deductions"]))?,
            final_long_carry: checked_estate_code_amount(final_deductions, "D")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["final_deductions"]))?,
            final_k1: *final_form1041,
            final_nol: None,
            amt_box12a: checked_estate_code_amount(amt, "A")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            amt_box12b: checked_estate_code_amount(amt, "B")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            amt_box12c: checked_estate_code_amount(amt, "C")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            amt_box12d: checked_estate_code_amount(amt, "D")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            amt_box12e: checked_estate_code_amount(amt, "E")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            amt_box12f: checked_estate_code_amount(amt, "F")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
            backup_withholding: checked_estate_code_amount(credits, "B")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["credits"]))?,
            allocated_estimated_tax: checked_estate_code_amount(credits, "A")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["credits"]))?,
            tax_exempt_interest: checked_estate_code_amount(information, "A")
                .map_err(|issue| issue.map_field_prefix(&["rows"], &["information"]))?,
            niit_adjustment: None,
            unentered: Vec::new(),
            entity_period: self.reported_period().map(|period| FinalEstateTrustTaxPeriod {
                period,
                accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
            }),
            minimum_tax_exclusion: if amt.iter().any(|row| row.code == "J") {
                Some(
                    checked_estate_code_amount(amt, "J")
                        .map_err(|issue| issue.map_field_prefix(&["rows"], &["amt"]))?,
                )
            } else {
                None
            },
        })
    }
    pub(crate) fn printed_admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.began.is_some() != self.ended.is_some() {
            return Some(
                crate::validation::ValidationIssue::new("ESTATE_SOURCE_PERIOD_INVALID")
                    .at_field(if self.began.is_none() { "began" } else { "ended" }),
            );
        }
        if self.tax_year != 0
            && self.began.is_none()
            && self.ended.is_none()
            && self.calendar_period().is_none()
        {
            return Some(
                crate::validation::ValidationIssue::new("ESTATE_SOURCE_PERIOD_INVALID")
                    .at_field("tax_year"),
            );
        }
        None
    }
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.printed_admission_issue() {
            return Some(issue);
        }
        self.checked_record().err().or_else(|| {
            checked_estate_code_amount(&self.information, "H")
                .err()
                .map(|issue| issue.map_field_prefix(&["rows"], &["information"]))
        })
    }
}
impl From<&K1EstateTrustPaper> for Form1041K1 {
    fn from(paper: &K1EstateTrustPaper) -> Self {
        paper.checked_record().expect("admitted estate K-1 printed amounts")
    }
}

impl From<&FederalEstateTrustPortfolioK1> for K1EstateTrustPaper {
    fn from(record: &FederalEstateTrustPortfolioK1) -> Self {
        let mut paper = Self::default();
        paper.estate_trust.federal_id = record.ein.clone();
        paper.final_form1041 = record.final_k1;
        paper.final_k1 = record.final_k1;
        if let Some(period) = &record.entity_period {
            paper.began = Some(period.period.began);
            paper.ended = Some(period.period.ended);
        }
        paper.interest = record.interest;
        paper.ordinary_dividends = record.ordinary_dividends;
        paper.qualified_dividends = record.qualified_dividends;
        paper.short_gain = record.short_gain;
        paper.long_gain = record.long_gain;
        paper.rate_28_gain = record.rate_28_gain;
        paper.unrecaptured_gain = record.unrecaptured_gain;
        paper.portfolio_income = record.royalty_income;
        paper.business_income = record.activities.box6;
        paper.rental_real_estate_income = record.activities.box7;
        paper.other_rental_income = record.activities.box8;
        paper.estate_tax_deduction = record.box_10_interest_ird_estate_tax;
        paper.filed_1041t = record.form_1041t_timely;
        let mut add = |box_number, code: &str, amount| {
            let rows = match box_number {
                9 => &mut paper.deductions,
                11 => &mut paper.final_deductions,
                12 => &mut paper.amt,
                13 => &mut paper.credits,
                _ => &mut paper.information,
            };
            if amount != 0 {
                rows.push(EstateTrustCodeRow { code: code.into(), amount });
            }
        };
        add(
            9,
            "A",
            crate::kernel::ratio::checked_sum(
                core::iter::once(record.box_9_royalty_depreciation)
                    .chain(record.activities.activities.iter().map(|a| a.depreciation)),
            )
            .expect("admitted estate deduction total"),
        );
        add(
            9,
            "B",
            crate::kernel::ratio::checked_sum(
                core::iter::once(record.box_9_royalty_depletion)
                    .chain(record.activities.activities.iter().map(|a| a.depletion)),
            )
            .expect("admitted estate deduction total"),
        );
        add(
            9,
            "C",
            crate::kernel::ratio::checked_sum(
                core::iter::once(record.box_9_royalty_amortization)
                    .chain(record.activities.activities.iter().map(|a| a.amortization)),
            )
            .expect("admitted estate deduction total"),
        );
        add(11, "A", record.final_67e_deduction);
        add(
            11,
            "B",
            crate::kernel::ratio::checked_sum([
                record.final_personal_property_tax,
                record.final_real_estate_tax,
                record.final_state_income_tax,
                record.final_state_sales_tax,
            ])
            .expect("admitted final-year tax total"),
        );
        add(11, "C", record.final_short_carry);
        add(11, "D", record.final_long_carry);
        if let Some(nol) = &record.final_nol {
            add(
                11,
                "E",
                crate::kernel::ratio::checked_sum(
                    nol.years.iter().map(|year| year.regular_balance),
                )
                .expect("admitted inherited regular NOL total"),
            );
            add(
                11,
                "F",
                crate::kernel::ratio::checked_sum(
                    nol.years.iter().map(|year| year.alternative_balance),
                )
                .expect("admitted inherited AMT NOL total"),
            );
        }
        add(12, "A", record.amt_box12a);
        add(12, "B", record.amt_box12b);
        add(12, "C", record.amt_box12c);
        add(12, "D", record.amt_box12d);
        add(12, "E", record.amt_box12e);
        add(12, "F", record.amt_box12f);
        add(13, "A", record.allocated_estimated_tax);
        add(13, "B", record.backup_withholding);
        add(14, "A", record.tax_exempt_interest);
        add(14, "F", record.gross_farming_fishing_income);
        if let Some(niit) = &record.niit_adjustment {
            paper.information.push(EstateTrustCodeRow { code: "H".into(), amount: niit.line_7 });
        }
        if let Some(amount) = record.minimum_tax_exclusion {
            paper.amt.push(EstateTrustCodeRow { code: "J".into(), amount });
        }
        paper
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read only the invoked beneficiary activity from the fiduciary attached Schedule K-1 (Form 1041) schedule. Read its source name, business/rental character, signed allocated income, separate codes 9A depreciation, 9B depletion, 9C amortization and signed AMT net difference. Clearly blank amounts are zero; missing required statements or illegible values are errors. Duplicate copies of the same activity count once. Do not supply participation, prior passive balances or identity. A corrected statement replaces current source facts in this row and retains personal records.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.name,@.kind,@.income,@.depreciation,@.depletion,@.amortization,@.amt_difference"
)]
pub struct EstateActivityPaper {
    /// Activity reference
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub name: String,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub kind: EstateActivityKind,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciation: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depletion: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amortization: i64,
    /// Signed difference to activity net from fiduciary's AMT activity statement
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amt_difference: i64,
}
impl From<&EstateActivityPaper> for EstateActivityPage {
    fn from(value: &EstateActivityPaper) -> Self {
        let EstateActivityPaper {
            id,
            name,
            kind,
            income,
            depreciation,
            depletion,
            amortization,
            amt_difference,
        } = value;
        Self {
            id: *id,
            name: name.clone(),
            kind: kind.clone(),
            income: *income,
            depreciation: *depreciation,
            depletion: *depletion,
            amortization: *amortization,
            participation: EstateParticipation::Unresolved,
            carried: EstatePassiveCarry { regular: 0, alternative: 0 },
            amt_difference: *amt_difference,
        }
    }
}
impl From<&EstateActivityPage> for EstateActivityPaper {
    fn from(value: &EstateActivityPage) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            kind: value.kind.clone(),
            income: value.income,
            depreciation: value.depreciation,
            depletion: value.depletion,
            amortization: value.amortization,
            amt_difference: value.amt_difference,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the invoked beneficiary section 199A statement attached to Schedule K-1 (Form 1041), with the fiduciary apportioned signed QBI, W-2 wages, UBIA, SSTB and business/PTP/cooperative character. Use beneficiary allocations, never total trust amounts. Duplicate copies count once. Do not infer source links or prior-loss histories; preserve them on a corrected statement. Missing or illegible source amounts are errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.name,@.kind,@.income,@.wages,@.property"
)]
pub struct EstateQbiStatementPaper {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub name: String,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub property: i64,
    /// The statement identifies a specified service trade or business
    #[libertas_llm_doc_field(allow_empty = false)]
    pub service: bool,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub kind: EstateQbiKind,
}
impl From<&EstateQbiStatementPaper> for EstateQbiStatementPage {
    fn from(value: &EstateQbiStatementPaper) -> Self {
        let EstateQbiStatementPaper { id, name, income, wages, property, service, kind } = value;
        Self {
            id: *id,
            name: name.clone(),
            income: *income,
            wages: *wages,
            property: *property,
            service: *service,
            kind: kind.clone(),
            activities: Vec::new(),
            passive_history: Vec::new(),
            allocations: Vec::new(),
        }
    }
}
impl From<&EstateQbiStatementPage> for EstateQbiStatementPaper {
    fn from(value: &EstateQbiStatementPage) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            income: value.income,
            wages: value.wages,
            property: value.property,
            service: value.service,
            kind: value.kind.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read only this invoked beneficiary AMT source statement attached to Schedule K-1 (Form 1041). Copy its actual box 12 code G, H or I and signed allocated amount. Do not use trust-wide totals or infer a Form 6251 line, activity link, or whether the amount is already in code A. Preserve all unmarked treatment facts. Duplicate copies count once; missing, illegible or conflicting source amounts are errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.code,@.amount"
)]
pub struct EstateAmtAdjustmentPaper {
    /// Accepted source-statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    #[libertas_llm_doc_field(allow_empty = false)]
    pub code: EstateAmtCode,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
}
impl From<&EstateAmtAdjustmentPaper> for EstateAmtAdjustmentPage {
    fn from(value: &EstateAmtAdjustmentPaper) -> Self {
        let EstateAmtAdjustmentPaper { id, code, amount } = value;
        Self {
            id: *id,
            code: code.clone(),
            amount: *amount,
            treatment: EstateAmtTreatmentPage::Unavailable,
        }
    }
}
impl From<&EstateAmtAdjustmentPage> for EstateAmtAdjustmentPaper {
    fn from(value: &EstateAmtAdjustmentPage) -> Self {
        Self { id: value.id, code: value.code.clone(), amount: value.amount }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateInvestmentPropertyPaper {
    /// Rehabilitation expenditures, rental real estate — K-1 (1065) box 15 code E or K-1 (1120-S) box 13 code E
    RehabilitationRental {
        /// Your share of qualified rehabilitation expenditures
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        expenditures: i64,
        /// Year the building was placed in service
        /// From the entity's statement. The credit is 4% of the expenditures in each of the five years starting that year.
        #[libertas_llm_doc_field(allow_empty = false)]
        // Tax-year source window: only the stated current or prior tax years apply.
        #[libertas_number(min = 2022, max = 2026)]
        placed_year: u32,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
    },
    /// Rehabilitation expenditures, other activities — K-1 (1065) box 20 code D or K-1 (1120-S) box 17 code C
    RehabilitationOther {
        /// Your share of qualified rehabilitation expenditures
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        expenditures: i64,
        /// Year the building was placed in service
        #[libertas_llm_doc_field(allow_empty = false)]
        // Tax-year source window: only the stated current or prior tax years apply.
        #[libertas_number(min = 2022, max = 2026)]
        placed_year: u32,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
    },
    /// Basis of energy property placed in service in 2026 — K-1 (1065) box 20 code E or K-1 (1120-S) box 17 code D
    Energy {
        /// Your share of the basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        basis: i64,
        /// Rates from the entity's Form 3468 Part I
        #[libertas_llm_doc_field]
        rate: EnergyRate,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
    },
    /// Clean electricity investment property placed in service in 2026 — K-1 (1065) box 20 code AV or K-1 (1120-S) box 17 code AV
    CleanElectricity {
        /// Your share of the basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        basis: i64,
        /// Energy storage technology
        /// Form 3468 Part V Section B instead of Section A.
        #[libertas_llm_doc_field]
        #[libertas_default(false)]
        storage: bool,
        /// Rates from the entity's Form 3468 Part I
        #[libertas_llm_doc_field]
        rate: EnergyRate,
        /// Accepted property identity, assigned only by normal Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        #[libertas_number(min = 0)]
        id: i64,
    },
}
impl From<&EstateInvestmentPropertyPaper> for K1InvestmentProperty {
    fn from(value: &EstateInvestmentPropertyPaper) -> Self {
        match value {
            EstateInvestmentPropertyPaper::RehabilitationRental {
                expenditures,
                placed_year,
                id,
            } => Self::RehabilitationRental {
                expenditures: *expenditures,
                placed_year: *placed_year,
                id: *id,
                own_form: None,
            },
            EstateInvestmentPropertyPaper::RehabilitationOther {
                expenditures,
                placed_year,
                id,
            } => Self::RehabilitationOther {
                expenditures: *expenditures,
                placed_year: *placed_year,
                id: *id,
                own_form: None,
            },
            EstateInvestmentPropertyPaper::Energy { basis, rate, id } => {
                Self::Energy { basis: *basis, rate: rate.clone(), id: *id, own_form: None }
            }
            EstateInvestmentPropertyPaper::CleanElectricity { basis, storage, rate, id } => {
                Self::CleanElectricity {
                    basis: *basis,
                    storage: *storage,
                    rate: rate.clone(),
                    id: *id,
                    own_form: None,
                }
            }
        }
    }
}
impl From<&K1InvestmentProperty> for EstateInvestmentPropertyPaper {
    fn from(value: &K1InvestmentProperty) -> Self {
        match value {
            K1InvestmentProperty::RehabilitationRental {
                expenditures,
                placed_year,
                id,
                own_form: _,
            } => Self::RehabilitationRental {
                expenditures: *expenditures,
                placed_year: *placed_year,
                id: *id,
            },
            K1InvestmentProperty::RehabilitationOther {
                expenditures,
                placed_year,
                id,
                own_form: _,
            } => Self::RehabilitationOther {
                expenditures: *expenditures,
                placed_year: *placed_year,
                id: *id,
            },
            K1InvestmentProperty::Energy { basis, rate, id, own_form: _ } => {
                Self::Energy { basis: *basis, rate: rate.clone(), id: *id }
            }
            K1InvestmentProperty::CleanElectricity { basis, storage, rate, id, own_form: _ } => {
                Self::CleanElectricity {
                    basis: *basis,
                    storage: *storage,
                    rate: rate.clone(),
                    id: *id,
                }
            }
        }
    }
}

/// Fiduciary's investment property statement and its source activity links
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstatePropertyStatementPaper {
    #[libertas_llm_doc_field]
    pub property: EstateInvestmentPropertyPaper,
    #[libertas_llm_doc_field]
    /// ----
    #[libertas_llm_doc_field(allow_empty = false)]
    pub activities: Vec<i64>,
}

/// Fiduciary statements accompanying Schedule K-1 (Form 1041)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct EstateTrustStatementsPaper {
    /// [NullText]
    /// The fiduciary's statement gives no final-year source character
    /// [SetText]
    /// The final-year source character and entitlement are
    #[libertas_llm_doc_field]
    pub final_character: Option<EstateFinalCharacterPage>,
    /// [NullText]
    /// No other box 12 code A amount is supplied
    /// [SetText]
    /// The other box 12 code A amount is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub amt_other_in_a: Option<i64>,
    #[libertas_llm_doc_field]
    pub amt_adjustments: Vec<EstateAmtAdjustmentPaper>,
    /// [NullText]
    /// No box 14 code E investment-income statement is available
    /// [SetText]
    /// The box 14 code E investment-income statement is
    #[libertas_llm_doc_field]
    pub investment_income: Option<EstateInvestmentIncome>,
    #[libertas_llm_doc_field]
    pub reported_credits: Vec<EstateCreditStatementPaper>,
    /// [NullText]
    /// No box 14 code B foreign-tax statement is available
    /// [SetText]
    /// The box 14 code B foreign-tax statement is
    #[libertas_llm_doc_field]
    pub foreign_statement: Option<EstateForeignStatement>,
    #[libertas_llm_doc_field]
    pub investment_property: Vec<EstatePropertyStatementPaper>,
    #[libertas_llm_doc_field]
    pub activities: Vec<EstateActivityPaper>,
    #[libertas_llm_doc_field]
    pub qbi: Vec<EstateQbiStatementPaper>,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub private_activity_interest: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub section199a_dividends: i64,
    /// The entire box 5 amount is ordinary domestic nonbusiness royalties
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_5_royalty_only: bool,
    /// Box 9 code A is the supported sound-recording deduction described by the fiduciary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_9_royalty_sound_recording_only: bool,
    /// Box 9 code B is cost depletion from the same royalty activity
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_9_royalty_cost_depletion_only: bool,
    /// Box 9 code C is the supported section 197 royalty amortization
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_9_royalty_amortization_only: bool,
    /// Box 10 is entirely estate tax attributable to this K-1's taxable IRD interest
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_10_interest_ird_only: bool,
    /// No box 11 code A amount is allocable to net investment income
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_67e_no_niit_allocation: bool,
    /// Box 11 code B beneficiary allocation of personal property tax
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub final_personal_property_tax: i64,
    /// Box 11 code B beneficiary allocation of noninvestment real-estate tax
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub final_real_estate_tax: i64,
    /// Box 11 code B beneficiary allocation of state or local income tax
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub final_state_income_tax: i64,
    /// Box 11 code B beneficiary allocation of state or local sales tax
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field]
    pub final_state_sales_tax: i64,
    /// The fiduciary's code B statement contains only these identified taxes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_taxes_only: bool,
    /// The carryovers have the same regular, AMT, investment and NIIT character
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_carry_same_tax_character: bool,
    /// Box 12 code A has no components or related AMT items
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amt_box12a_standalone: bool,
    /// The fiduciary identified every code A component in codes B, C and D
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amt_box12_components_confirmed: bool,
    /// Code E is a portion of code D
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amt_box12e_included_in_d: bool,
    /// Code F is a disjoint portion of code D
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amt_box12f_included_in_d: bool,
    /// The code I statement contains only section 199A dividends
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub code_i_dividends_only: bool,
    /// Original loss periods and beneficiary carryovers from the fiduciary's successor statement
    /// [NullText]
    /// No successor loss statement is available
    /// [SetText]
    /// The successor loss statement is
    #[libertas_llm_doc_field]
    pub final_nol: Option<FinalEstateTrustNol>,
    /// Fiduciary's allocation of the box 14 code H adjustment to modified AGI
    /// [NullText]
    /// No modified AGI adjustment is supplied
    /// [SetText]
    /// The fiduciary's modified AGI adjustment is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub niit_adjustment: Option<i64>,
}

/// Schedule K-1 (Form 1041) and the fiduciary's printed statements
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read this Schedule K-1 (Form 1041) and its actual fiduciary attachments. Fill the printed form fields exactly as shown, including parties, year, actual beginning/ending dates, checkboxes and every code/amount row. In statements collect the beneficiary's reported source character and allocated amounts that the attachments establish. Both sections are the same fields used for manual entry. Never substitute entity totals, fabricate an absent statement, infer a 52/53-week election from an ending date, or supply beneficiary participation, passive history, entitlement or personal elections. Report illegible or conflicting facts as errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.form.estate_trust.federal_id,@.form.beneficiary.federal_id,@.form.tax_year,@.form.began,@.form.ended,@.form.interest,@.form.ordinary_dividends,@.form.qualified_dividends,@.form.short_gain,@.form.long_gain,@.form.rate_28_gain,@.form.unrecaptured_gain,@.form.portfolio_income,@.form.business_income,@.form.rental_real_estate_income,@.form.other_rental_income,@.form.estate_tax_deduction",
    form_date_order = "MonthDayYear"
)]
pub struct K1EstateTrustDocument {
    #[libertas_llm_doc_field]
    pub form: K1EstateTrustPaper,
    #[libertas_llm_doc_field]
    pub statements: EstateTrustStatementsPaper,
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub estate_activities: Vec<EstateSourceChoice>,
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub estate_mortgages: Vec<EstateSourceChoice>,
}

impl EstateTrustSource {
    pub(crate) fn printed(form: K1EstateTrustPaper) -> Self {
        let mut source = Self {
            form,
            statements: EstateTrustStatements::default(),
            answers: EstateTrustAnswers::default(),
        };
        let a = &mut source.statements;
        let p = &source.form;
        source.answers.complete_portfolio_only = true;
        a.unentered = printed_unentered(p);
        source
    }
    pub(crate) fn page(&self) -> K1EstateTrust {
        let mut page = K1EstateTrust::from_source_record(
            &self.form.estate_trust.name,
            &FederalEstateTrustPortfolioK1::from(self),
        );
        page.estate_trust = self.form.estate_trust.clone();
        page.fiduciary = self.form.fiduciary.clone();
        page.beneficiary = self.form.beneficiary.clone();
        page
    }
    pub(crate) fn replace_personal_page(&mut self, page: &K1EstateTrust) {
        let projected = page.record();
        for credit in &mut self.statements.reported_credits {
            if credit.code_name() == "I" {
                credit.research_line =
                    projected.reported_credits.iter().find_map(|row| match row.code {
                        EstateCreditCode::I { line } => Some(line),
                        _ => None,
                    });
            }
        }
        self.answers.final_year_successor = projected.final_year_successor;
        self.answers.final_mortgages = projected.final_mortgages;
        if let ReitDividends::Reported { eligible, .. } = projected.section199a_dividends
            && let ReitDividends::Reported { eligible: old, .. } =
                &mut self.statements.section199a_dividends
        {
            *old = eligible;
        }
        for source in &mut self.statements.activities {
            if let Some(changed) =
                projected.activities.activities.iter().find(|r| r.id == source.id)
            {
                source.participation = changed.participation.clone();
                source.carried = changed.carried.clone();
            }
        }
        for source in &mut self.statements.qbi.statements {
            if let Some(changed) = projected.qbi.statements.iter().find(|r| r.id == source.id) {
                source.activities = changed.activities.clone();
                source.passive_history = changed.passive_history.clone();
                source.allocations = changed.allocations.clone();
            }
        }
        if let (Some(source), Some(changed)) =
            (&mut self.statements.amt_reconciliation, projected.amt_reconciliation)
        {
            for source in &mut source.adjustments {
                if let Some(changed) = changed.adjustments.iter().find(|r| r.id == source.id) {
                    source.treatment = changed.treatment.clone();
                }
            }
        }
        for source in &mut self.statements.investment_property {
            if let Some(changed) = projected
                .investment_property
                .iter()
                .find(|r| r.property.id() == source.property.id())
            {
                *source.property.own_form_mut() = changed.property.own_form().cloned();
            }
        }
    }
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.answers.entity_period = accepted.answers.entity_period.clone();
        self.answers.complete_portfolio_only = accepted.answers.complete_portfolio_only;
        for credit in &mut self.statements.reported_credits {
            credit.research_line = accepted
                .statements
                .reported_credits
                .iter()
                .find(|old| old.code_name() == credit.code_name())
                .and_then(|old| old.research_line);
        }

        crate::interview::estate_retain(&mut self.statements, &accepted.statements);
        self.answers.final_year_successor = accepted.answers.final_year_successor;
        self.answers.final_mortgages = accepted.answers.final_mortgages.clone();
    }
}

impl From<&K1EstateTrustPaper> for K1EstateTrust {
    fn from(paper: &K1EstateTrustPaper) -> Self {
        // The printed-to-natural conversion above exhaustively owns every printed field.
        EstateTrustSource::printed(paper.clone()).page()
    }
}

impl K1EstateTrustDocument {
    pub(crate) fn source(&self) -> EstateTrustSource {
        let mut source = EstateTrustSource::printed(self.form.clone());
        let s = &self.statements;
        let a = &mut source.statements;
        a.final_character = s.final_character.as_ref().map(EstateFinalCharacterPage::record);
        a.amt_reconciliation =
            (s.amt_other_in_a.is_some() || !s.amt_adjustments.is_empty()).then(|| {
                EstateAmtReconciliation {
                    other_in_a: s.amt_other_in_a.unwrap_or(0),
                    adjustments: s
                        .amt_adjustments
                        .iter()
                        .map(|r| EstateAmtAdjustment {
                            id: r.id,
                            code: r.code.clone(),
                            amount: r.amount,
                            treatment: EstateAmtTreatment::Unavailable,
                        })
                        .collect(),
                }
            });
        a.investment_income = s.investment_income.clone();
        a.reported_credits = s
            .reported_credits
            .iter()
            .map(|paper| EstateCreditSource { paper: paper.clone(), research_line: None })
            .collect();
        a.foreign_statement = s.foreign_statement.clone();
        a.investment_property = s
            .investment_property
            .iter()
            .map(|p| EstateInvestmentProperty {
                property: (&p.property).into(),
                activities: p.activities.clone(),
            })
            .collect();
        a.activities = s.activities.iter().map(|p| EstateActivityPage::from(p).record()).collect();
        a.qbi.statements = s.qbi.iter().map(|p| EstateQbiStatementPage::from(p).record()).collect();
        a.private_activity_interest = s.private_activity_interest;
        a.section199a_dividends = if s.section199a_dividends != 0 {
            ReitDividends::Reported { amount: s.section199a_dividends, eligible: None }
        } else {
            ReitDividends::None
        };
        a.box_5_royalty_only = s.box_5_royalty_only;
        a.box_9_royalty_sound_recording_only = s.box_9_royalty_sound_recording_only;
        a.box_9_royalty_cost_depletion_only = s.box_9_royalty_cost_depletion_only;
        a.box_9_royalty_amortization_only = s.box_9_royalty_amortization_only;
        a.box_10_interest_ird_only = s.box_10_interest_ird_only;
        a.final_67e_no_niit_allocation = s.final_67e_no_niit_allocation;
        a.final_personal_property_tax = s.final_personal_property_tax;
        a.final_real_estate_tax = s.final_real_estate_tax;
        a.final_state_income_tax = s.final_state_income_tax;
        a.final_state_sales_tax = s.final_state_sales_tax;
        a.final_taxes_only = s.final_taxes_only;
        a.final_carry_same_tax_character = s.final_carry_same_tax_character;
        a.amt_box12a_standalone = s.amt_box12a_standalone;
        a.amt_box12_components_confirmed = s.amt_box12_components_confirmed;
        a.amt_box12e_included_in_d = s.amt_box12e_included_in_d;
        a.amt_box12f_included_in_d = s.amt_box12f_included_in_d;
        a.code_i_dividends_only = s.code_i_dividends_only;
        a.final_nol = s.final_nol.clone();
        a.niit_adjustment = s.niit_adjustment;

        // A printed code still needs its actual supporting statement.
        a.unentered.retain(|item| match item {
            EstateTrustK1Item::AmtItems => a.amt_reconciliation.is_none(),
            EstateTrustK1Item::Credits => self.form.credits.iter().any(|row| {
                !matches!(row.code.as_str(), "A" | "B" | "R")
                    && !a.reported_credits.iter().any(|credit| credit.code_name() == row.code)
            }),
            EstateTrustK1Item::ForeignTaxes => a.foreign_statement.is_none(),
            EstateTrustK1Item::InvestmentIncome => a.investment_income.is_none(),
            EstateTrustK1Item::Rehabilitation => !a.investment_property.iter().any(|p| {
                matches!(
                    p.property,
                    K1InvestmentProperty::RehabilitationRental { .. }
                        | K1InvestmentProperty::RehabilitationOther { .. }
                )
            }),
            EstateTrustK1Item::EnergyProperty => !a
                .investment_property
                .iter()
                .any(|p| matches!(p.property, K1InvestmentProperty::Energy { .. })),
            EstateTrustK1Item::CleanElectricity => !a
                .investment_property
                .iter()
                .any(|p| matches!(p.property, K1InvestmentProperty::CleanElectricity { .. })),
            _ => true,
        });
        source
    }
    pub(crate) fn record(&self) -> EstateTrustSource {
        self.source()
    }
    pub(crate) fn printed_admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.form.printed_admission_issue().map(|issue| issue.in_field("form"))
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(
        &self,
        required: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.form.admission_issue() {
            return Some(issue.in_field("form"));
        }
        let source = self.source();
        source
            .admission_issue()
            .or_else(|| source.page().issue_with_mode(required))
            .map(|issue| self.map_issue(issue))
    }
    pub(crate) fn from_source(source: &EstateTrustSource) -> Self {
        let a = &source.statements;
        Self {
            form: source.form.clone(),
            estate_activities: crate::estate_trust::activity_choices(&EstateActivities {
                box6: source.form.business_income,
                box7: source.form.rental_real_estate_income,
                box8: source.form.other_rental_income,
                activities: a.activities.clone(),
            }),
            estate_mortgages: Vec::new(),
            statements: EstateTrustStatementsPaper {
                final_character: a
                    .final_character
                    .as_ref()
                    .map(EstateFinalCharacterPage::from_record),
                amt_other_in_a: a.amt_reconciliation.as_ref().map(|a| a.other_in_a),
                amt_adjustments: a.amt_reconciliation.as_ref().map_or_else(Vec::new, |a| {
                    a.adjustments
                        .iter()
                        .map(|r| EstateAmtAdjustmentPaper {
                            id: r.id,
                            code: r.code.clone(),
                            amount: r.amount,
                        })
                        .collect()
                }),
                investment_income: a.investment_income.clone(),
                reported_credits: a.reported_credits.iter().map(|r| r.paper.clone()).collect(),
                foreign_statement: a.foreign_statement.clone(),
                investment_property: a
                    .investment_property
                    .iter()
                    .map(|p| EstatePropertyStatementPaper {
                        property: (&p.property).into(),
                        activities: p.activities.clone(),
                    })
                    .collect(),
                activities: a
                    .activities
                    .iter()
                    .map(|r| EstateActivityPaper::from(&EstateActivityPage::from_record(r)))
                    .collect(),
                qbi: a
                    .qbi
                    .statements
                    .iter()
                    .map(|r| EstateQbiStatementPaper::from(&EstateQbiStatementPage::from_record(r)))
                    .collect(),
                private_activity_interest: a.private_activity_interest,
                section199a_dividends: match a.section199a_dividends {
                    ReitDividends::Reported { amount, .. } => amount,
                    ReitDividends::None => 0,
                },
                box_5_royalty_only: a.box_5_royalty_only,
                box_9_royalty_sound_recording_only: a.box_9_royalty_sound_recording_only,
                box_9_royalty_cost_depletion_only: a.box_9_royalty_cost_depletion_only,
                box_9_royalty_amortization_only: a.box_9_royalty_amortization_only,
                box_10_interest_ird_only: a.box_10_interest_ird_only,
                final_67e_no_niit_allocation: a.final_67e_no_niit_allocation,
                final_personal_property_tax: a.final_personal_property_tax,
                final_real_estate_tax: a.final_real_estate_tax,
                final_state_income_tax: a.final_state_income_tax,
                final_state_sales_tax: a.final_state_sales_tax,
                final_taxes_only: a.final_taxes_only,
                final_carry_same_tax_character: a.final_carry_same_tax_character,
                amt_box12a_standalone: a.amt_box12a_standalone,
                amt_box12_components_confirmed: a.amt_box12_components_confirmed,
                amt_box12e_included_in_d: a.amt_box12e_included_in_d,
                amt_box12f_included_in_d: a.amt_box12f_included_in_d,
                code_i_dividends_only: a.code_i_dividends_only,
                final_nol: a.final_nol.clone(),
                niit_adjustment: a.niit_adjustment,
            },
        }
    }
}

fn printed_unentered(paper: &K1EstateTrustPaper) -> Vec<EstateTrustK1Item> {
    let mut items = Vec::new();
    for code in &paper.amt {
        if matches!(code.code.as_str(), "G" | "H" | "I")
            && !items.contains(&EstateTrustK1Item::AmtItems)
        {
            items.push(EstateTrustK1Item::AmtItems);
        }
    }
    for code in &paper.credits {
        let item = match code.code.as_str() {
            "A" | "B" => continue,
            "R" => EstateTrustK1Item::CreditRecapture,
            _ => EstateTrustK1Item::Credits,
        };
        if !items.contains(&item) {
            items.push(item);
        }
    }
    for code in &paper.information {
        let item = match code.code.as_str() {
            "A" | "F" | "H" | "I" => continue,
            "B" => EstateTrustK1Item::ForeignTaxes,
            "C" => EstateTrustK1Item::Rehabilitation,
            "D" => EstateTrustK1Item::EnergyProperty,
            "E" => EstateTrustK1Item::InvestmentIncome,
            "G" => EstateTrustK1Item::ForeignTradingGrossReceipts,
            "J" => EstateTrustK1Item::AdvancedCoalGasification,
            "K" => EstateTrustK1Item::AdvancedEnergy,
            "L" => EstateTrustK1Item::AdvancedManufacturing,
            "M" => EstateTrustK1Item::CleanElectricity,
            _ => EstateTrustK1Item::OtherInformation,
        };
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

impl EstateTrustSource {
    pub(crate) fn moved_questions(&self) -> Vec<(String, &'static str, DocumentAnswer, Vec<i32>)> {
        use DocumentAnswer as A;
        let a = &self.answers;
        let p = &self.form;
        let mut rows = Vec::new();
        let mut add =
            |field: &str, question, answer| rows.push((field.into(), question, answer, Vec::new()));
        if p.reported_period().is_some() || a.entity_period.is_some() {
            add(
                "estate/accounting",
                "DOCUMENT_ESTATE_ACCOUNTING",
                A::EstateAccounting { value: a.entity_period.clone() },
            );
        }
        if let Some(credit) =
            self.statements.reported_credits.iter().find(|credit| credit.code_name() == "I")
        {
            add(
                "estate/research_credit",
                "DOCUMENT_K1_RESEARCH_CREDIT_LINE",
                A::EstateResearchCreditLine { value: credit.research_line },
            );
        }
        add(
            "estate/other_source",
            "DOCUMENT_ESTATE_OTHER_SOURCE",
            A::EstateOtherSource { value: !a.complete_portfolio_only },
        );
        for row in &mut rows {
            if matches!(row.2, A::EstateResearchCreditLine { .. }) {
                row.3 = crate::business_credit_page::research_credit_lines();
            }
        }
        rows
    }
    pub(crate) fn apply_moved_answer(
        &mut self,
        suffix: &str,
        answer: DocumentAnswer,
    ) -> Result<(), crate::validation::ValidationIssue> {
        use DocumentAnswer as A;
        let a = &mut self.answers;
        match (suffix, answer) {
            ("estate/accounting", A::EstateAccounting { value }) => a.entity_period = value,
            ("estate/research_credit", A::EstateResearchCreditLine { value }) => {
                for credit in &mut self.statements.reported_credits {
                    if credit.code_name() == "I" {
                        credit.research_line = value;
                    }
                }
            }
            ("estate/other_source", A::EstateOtherSource { value }) => {
                a.complete_portfolio_only = !value
            }
            _ => {
                return Err(crate::validation::ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("value"));
            }
        }
        Ok(())
    }
}

/// Printed box 13 credit code; the beneficiary's research-credit election is separate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateCreditCodePaper {
    #[libertas_form_code("C")]
    C,
    #[libertas_form_code("D")]
    D,
    #[libertas_form_code("E")]
    E,
    #[libertas_form_code("F")]
    F,
    #[libertas_form_code("G")]
    G,
    #[libertas_form_code("H")]
    H,
    #[libertas_form_code("I")]
    I,
    #[libertas_form_code("J")]
    J {
        #[libertas_llm_doc_field]
        line: BusinessCredit,
    },
    #[libertas_form_code("K")]
    K,
    #[libertas_form_code("L")]
    L,
    #[libertas_form_code("M")]
    M,
    #[libertas_form_code("N")]
    N,
    #[libertas_form_code("O")]
    O,
    #[libertas_form_code("P")]
    P,
    #[libertas_form_code("Q")]
    Q,
    #[libertas_form_code("S")]
    S,
    #[libertas_form_code("T")]
    T,
    #[libertas_form_code("ZZ")]
    ZZ {
        #[libertas_llm_doc_field]
        line: BusinessCredit,
    },
}

/// Fiduciary's credit statement and source activity links
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the fiduciary's actual box 13 credit statement: code, allocated amount and activity links. Do not infer the beneficiary's eligible-small-business test or a Form 3800 line for code I. Codes J and ZZ keep only a line actually identified by the fiduciary's statement. Preserve source identities and report illegible or conflicting facts as errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.code,@.amount"
)]
pub struct EstateCreditStatementPaper {
    #[libertas_llm_doc_field]
    pub code: EstateCreditCodePaper,
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    #[libertas_llm_doc_field]
    /// ----
    #[libertas_llm_doc_field(allow_empty = false)]
    pub activities: Vec<i64>,
}

/// Accepted credit statement and its beneficiary research-credit answer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateCreditSource {
    pub paper: EstateCreditStatementPaper,
    pub research_line: Option<BusinessCredit>,
}
impl EstateCreditSource {
    pub(crate) fn native(&self) -> Option<EstateReportedCredit> {
        let EstateCreditStatementPaper { code, amount, activities } = &self.paper;
        let code = match code {
            EstateCreditCodePaper::C => EstateCreditCode::C,
            EstateCreditCodePaper::D => EstateCreditCode::D,
            EstateCreditCodePaper::E => EstateCreditCode::E,
            EstateCreditCodePaper::F => EstateCreditCode::F,
            EstateCreditCodePaper::G => EstateCreditCode::G,
            EstateCreditCodePaper::H => EstateCreditCode::H,
            EstateCreditCodePaper::I => EstateCreditCode::I { line: self.research_line? },
            EstateCreditCodePaper::J { line } => EstateCreditCode::J { line: *line },
            EstateCreditCodePaper::K => EstateCreditCode::K,
            EstateCreditCodePaper::L => EstateCreditCode::L,
            EstateCreditCodePaper::M => EstateCreditCode::M,
            EstateCreditCodePaper::N => EstateCreditCode::N,
            EstateCreditCodePaper::O => EstateCreditCode::O,
            EstateCreditCodePaper::P => EstateCreditCode::P,
            EstateCreditCodePaper::Q => EstateCreditCode::Q,
            EstateCreditCodePaper::S => EstateCreditCode::S,
            EstateCreditCodePaper::T => EstateCreditCode::T,
            EstateCreditCodePaper::ZZ { line } => EstateCreditCode::ZZ { line: *line },
        };
        Some(EstateReportedCredit { code, amount: *amount, activities: activities.clone() })
    }
    pub(crate) fn code_name(&self) -> &'static str {
        match self.paper.code {
            EstateCreditCodePaper::C => "C",
            EstateCreditCodePaper::D => "D",
            EstateCreditCodePaper::E => "E",
            EstateCreditCodePaper::F => "F",
            EstateCreditCodePaper::G => "G",
            EstateCreditCodePaper::H => "H",
            EstateCreditCodePaper::I => "I",
            EstateCreditCodePaper::J { .. } => "J",
            EstateCreditCodePaper::K => "K",
            EstateCreditCodePaper::L => "L",
            EstateCreditCodePaper::M => "M",
            EstateCreditCodePaper::N => "N",
            EstateCreditCodePaper::O => "O",
            EstateCreditCodePaper::P => "P",
            EstateCreditCodePaper::Q => "Q",
            EstateCreditCodePaper::S => "S",
            EstateCreditCodePaper::T => "T",
            EstateCreditCodePaper::ZZ { .. } => "ZZ",
        }
    }
}
impl From<&EstateReportedCredit> for EstateCreditSource {
    fn from(value: &EstateReportedCredit) -> Self {
        let EstateReportedCredit { code, amount, activities } = value;
        let (code, research_line) = match code {
            EstateCreditCode::C => (EstateCreditCodePaper::C, None),
            EstateCreditCode::D => (EstateCreditCodePaper::D, None),
            EstateCreditCode::E => (EstateCreditCodePaper::E, None),
            EstateCreditCode::F => (EstateCreditCodePaper::F, None),
            EstateCreditCode::G => (EstateCreditCodePaper::G, None),
            EstateCreditCode::H => (EstateCreditCodePaper::H, None),
            EstateCreditCode::I { line } => (EstateCreditCodePaper::I, Some(*line)),
            EstateCreditCode::J { line } => (EstateCreditCodePaper::J { line: *line }, None),
            EstateCreditCode::K => (EstateCreditCodePaper::K, None),
            EstateCreditCode::L => (EstateCreditCodePaper::L, None),
            EstateCreditCode::M => (EstateCreditCodePaper::M, None),
            EstateCreditCode::N => (EstateCreditCodePaper::N, None),
            EstateCreditCode::O => (EstateCreditCodePaper::O, None),
            EstateCreditCode::P => (EstateCreditCodePaper::P, None),
            EstateCreditCode::Q => (EstateCreditCodePaper::Q, None),
            EstateCreditCode::S => (EstateCreditCodePaper::S, None),
            EstateCreditCode::T => (EstateCreditCodePaper::T, None),
            EstateCreditCode::ZZ { line } => (EstateCreditCodePaper::ZZ { line: *line }, None),
        };
        Self {
            paper: EstateCreditStatementPaper {
                code,
                amount: *amount,
                activities: activities.clone(),
            },
            research_line,
        }
    }
}

/// Character of the deduction directly apportioned to the royalty activity
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EstateRoyaltyDeductionCharacter {
    None,
    SoundRecording,
    CostDepletion,
    Amortization,
}

impl K1EstateTrustDocument {
    fn map_issue(
        &self,
        mut issue: crate::validation::ValidationIssue,
    ) -> crate::validation::ValidationIssue {
        let direct: &[(&[&'static str], &[&'static str])] = &[
            (&["estate_trust"], &["form", "estate_trust"]),
            (&["interest"], &["form", "interest"]),
            (&["ordinary_dividends"], &["form", "ordinary_dividends"]),
            (&["qualified_dividends"], &["form", "qualified_dividends"]),
            (&["short_gain"], &["form", "short_gain"]),
            (&["long_gain"], &["form", "long_gain"]),
            (&["rate_28_gain"], &["form", "rate_28_gain"]),
            (&["unrecaptured_gain"], &["form", "unrecaptured_gain"]),
            (&["business", "box6"], &["form", "business_income"]),
            (&["business", "box7"], &["form", "rental_real_estate_income"]),
            (&["business", "box8"], &["form", "other_rental_income"]),
            (&["royalties", "amount"], &["form", "portfolio_income"]),
            (&["business", "activities"], &["statements", "activities"]),
            (&["qbi", "statements"], &["statements", "qbi"]),
            (&["amt_reconciliation", "adjustments"], &["statements", "amt_adjustments"]),
            (&["amt_reconciliation", "other_in_a"], &["statements", "amt_other_in_a"]),
            (&["final_nol"], &["statements", "final_nol"]),
            (&["more", "final_year", "nol"], &["statements", "final_nol"]),
            (
                &["more", "final_year", "personal_property_tax"],
                &["statements", "final_personal_property_tax"],
            ),
            (&["more", "final_year", "real_estate_tax"], &["statements", "final_real_estate_tax"]),
            (
                &["more", "final_year", "state_income_tax"],
                &["statements", "final_state_income_tax"],
            ),
            (&["more", "final_year", "state_sales_tax"], &["statements", "final_state_sales_tax"]),
            (&["more", "niit", "modified_agi"], &["statements", "niit_adjustment"]),
            (&["entity_period", "period", "began"], &["form", "began"]),
            (&["entity_period", "period", "ended"], &["form", "ended"]),
            (&["entity_period", "began"], &["form", "began"]),
            (&["entity_period", "ended"], &["form", "ended"]),
            (&["final_character"], &["statements", "final_character"]),
            (&["reported_credits"], &["statements", "reported_credits"]),
            (&["foreign_statement"], &["statements", "foreign_statement"]),
            (&["investment_income"], &["statements", "investment_income"]),
            (&["investment_property"], &["statements", "investment_property"]),
            (&["more", "private_activity_interest"], &["statements", "private_activity_interest"]),
            (&["more", "section199a_dividends"], &["statements", "section199a_dividends"]),
            (&["more", "ird_estate_tax"], &["form", "estate_tax_deduction"]),
        ];
        if let Some((from, to)) = direct.iter().find(|(from, _)| issue.has_field_prefix(from)) {
            return issue.map_field_prefix(from, to);
        }
        let codes: &[(&[&str], &str, &str)] = &[
            (&["more", "amt", "a"], "amt", "A"),
            (&["more", "amt", "b"], "amt", "B"),
            (&["more", "amt", "c"], "amt", "C"),
            (&["more", "amt", "d"], "amt", "D"),
            (&["more", "amt", "e"], "amt", "E"),
            (&["more", "amt", "f"], "amt", "F"),
            (&["more", "minimum_tax_exclusion"], "amt", "J"),
            (&["more", "tax_exempt_interest"], "information", "A"),
            (&["gross_farming_fishing_income"], "information", "F"),
            (&["more", "backup_withholding"], "credits", "B"),
        ];
        for (prefix, list, code) in codes {
            if issue.has_field_prefix(prefix) {
                let rows = match *list {
                    "amt" => &self.form.amt,
                    "information" => &self.form.information,
                    _ => &self.form.credits,
                };
                if let Some(index) = rows.iter().position(|row| row.code == *code) {
                    return crate::validation::ValidationIssue::new(issue.key)
                        .at_field("amount")
                        .in_list(list, index)
                        .in_field("form");
                }
                return crate::validation::ValidationIssue::new(issue.key)
                    .at_field(list)
                    .in_field("form");
            }
        }
        if issue.has_field_prefix(&["royalties", "deduction"]) {
            issue = crate::validation::ValidationIssue::new(issue.key)
                .at_field("deductions")
                .in_field("form");
        }
        issue
    }
}

impl Default for EstateTrustStatements {
    fn default() -> Self {
        Self {
            final_character: None,
            amt_reconciliation: None,
            investment_income: None,
            reported_credits: Vec::new(),
            foreign_statement: None,
            investment_property: Vec::new(),
            section199a_dividends: ReitDividends::None,
            code_i_dividends_only: false,
            box_5_royalty_only: false,
            box_9_royalty_sound_recording_only: false,
            box_9_royalty_cost_depletion_only: false,
            box_9_royalty_amortization_only: false,
            activities: Vec::new(),
            qbi: EstateQbiStatements { statements: Vec::new() },
            box_10_interest_ird_only: false,
            final_67e_no_niit_allocation: false,
            final_personal_property_tax: 0,
            final_real_estate_tax: 0,
            final_state_income_tax: 0,
            final_state_sales_tax: 0,
            final_taxes_only: false,
            final_carry_same_tax_character: false,
            amt_box12a_standalone: false,
            amt_box12_components_confirmed: false,
            amt_box12e_included_in_d: false,
            amt_box12f_included_in_d: false,
            private_activity_interest: 0,
            final_nol: None,
            niit_adjustment: None,
            unentered: Vec::new(),
        }
    }
}
