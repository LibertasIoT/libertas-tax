#[allow(unused_imports)]
use super::*;

/// Form 8986 partnership portfolio-interest adjustment
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form8986Paper {
    /// 1 Original (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub original: bool,
    /// 2 Corrected (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// Tracking Number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub tracking_number: String,
    /// Audit Control Number (if applicable)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub audit_control_number: String,
    /// Part I A Entity submitting this form
    /// [NullText]
    /// No box is checked
    /// [SetText]
    /// The checked box is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub submitting_entity: Option<Form8986SubmittingEntity>,
    /// Part I B Type of return filed by the entity that submitted this form
    /// [NullText]
    /// No box is checked
    /// [SetText]
    /// The checked box is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub entity_return: Option<Form8986EntityReturn>,
    /// Part II A and C Partnership's name, address and TIN
    #[libertas_llm_doc_field]
    pub partnership: crate::PrintedParty,
    /// Part II B Partnership representative or designated individual
    #[libertas_llm_doc_field]
    pub representative: Form8986Representative,
    /// Part II D Review year of the partnership is for tax year ended
    /// [NullText]
    /// The date is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub review_year_end: Option<u32>,
    /// Part II E Adjustment year of the partnership is for tax year ended
    /// [NullText]
    /// The date is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub adjustment_year_end: Option<u32>,
    /// Part II F Extended due date of the partnership's adjustment year return
    /// [NullText]
    /// The date is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub extended_due_date: Option<u32>,
    /// Part II G Date the partnership furnished the Form 8986 statements to its partners
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub furnished_on: u32,
    /// Part III Pass-through partner
    #[libertas_llm_doc_field]
    pub pass_through: Form8986PassThrough,
    /// Part IV A and B Partner's name, address and TIN
    #[libertas_llm_doc_field]
    pub partner: crate::PrintedParty,
    /// Part IV C1 General partner or LLC member manager (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub general_partner: bool,
    /// Part IV C2 Limited partner or other LLC member (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub limited_partner: bool,
    /// Part IV C3 Domestic partner (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub domestic_partner: bool,
    /// Part IV C4 Foreign partner (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub foreign_partner: bool,
    /// Part IV D What type of entity is this partner?
    /// [NullText]
    /// No box is checked
    /// [SetText]
    /// The checked box is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub partner_entity: Option<Form8986PartnerEntity>,
    /// Part IV D Retirement plan or other tax-exempt entity (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub tax_exempt: bool,
    /// Part IV E Partner's share of profit
    #[libertas_llm_doc_field]
    pub profit: Form8986PercentRow,
    /// Part IV E Partner's share of loss
    #[libertas_llm_doc_field]
    pub loss: Form8986PercentRow,
    /// Part IV E Partner's share of capital
    #[libertas_llm_doc_field]
    pub capital: Form8986PercentRow,
    /// Part IV F Nonrecourse liabilities
    #[libertas_llm_doc_field]
    pub nonrecourse: Form8986AmountRow,
    /// Part IV F Qualified nonrecourse financing
    #[libertas_llm_doc_field]
    pub qualified_nonrecourse: Form8986AmountRow,
    /// Part IV F Recourse liabilities
    #[libertas_llm_doc_field]
    pub recourse: Form8986AmountRow,
    /// Part IV G Beginning capital account balance
    #[libertas_llm_doc_field]
    pub beginning_capital: Form8986AmountRow,
    /// Part IV G Capital contributed during the year
    #[libertas_llm_doc_field]
    pub capital_contributed: Form8986AmountRow,
    /// Part IV G Current year increase (decrease)
    #[libertas_llm_doc_field]
    pub current_year_change: Form8986AmountRow,
    /// Part IV G Withdrawals and distributions
    #[libertas_llm_doc_field]
    pub withdrawals: Form8986AmountRow,
    /// Part IV G Section 736 payments received
    #[libertas_llm_doc_field]
    pub section_736_payments: Form8986AmountRow,
    /// Part IV G Ending capital account balance
    #[libertas_llm_doc_field]
    pub ending_capital: Form8986AmountRow,
    /// Part V Partner's total reviewed year items
    #[libertas_llm_doc_field]
    /// ----
    /// Part V line
    pub items: Vec<Form8986Item>,
    /// Part V Totals
    #[libertas_llm_doc_field]
    pub totals: Form8986Totals,
    /// Part V Applicable penalties
    #[libertas_llm_doc_field]
    /// ----
    /// Penalty
    pub penalties: Vec<Form8986Penalty>,
    /// Part VI Statements
    #[libertas_llm_doc_field]
    /// ----
    /// Statement
    pub statements: Vec<Form8986Statement>,
}

impl Form8986Paper {
    /// The printed identity: the tracking number, else the audit control number,
    /// else the partnership's TIN.
    pub(crate) fn source_id(&self) -> String {
        [&self.tracking_number, &self.audit_control_number, &self.partnership.federal_id]
            .into_iter()
            .find(|text| !text.trim().is_empty())
            .cloned()
            .unwrap_or_default()
    }
    /// The one Part V line for Schedule K-1 line 5 interest income, when that is
    /// the only line.
    pub(crate) fn interest_item(&self) -> Option<&Form8986Item> {
        match self.items.as_slice() {
            [item] if item.line_number.trim() == "5" && item.code.trim().is_empty() => Some(item),
            _ => None,
        }
    }
    /// Part V column (g) of the interest line.
    pub(crate) fn interest_adjustment(&self) -> i64 {
        self.interest_item().and_then(|item| item.adjustment).unwrap_or(0)
    }
    /// Part V column (d) of the interest line.
    pub(crate) fn original_partnership_interest(&self) -> i64 {
        self.interest_item().and_then(|item| item.as_reported).unwrap_or(0)
    }
}

/// Form 8986 Part I A
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form8986SubmittingEntity {
    /// 1 Audited BBA partnership
    AuditedPartnership,
    /// 2 Pass-through partner of an audited BBA partnership
    AuditPassThrough,
    /// 3 BBA partnership that filed an administrative adjustment request (AAR)
    AarPartnership,
    /// 4 Pass-through partner of a BBA partnership that filed an AAR
    AarPassThrough,
}

/// Form 8986 Part I B
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form8986EntityReturn {
    /// 1 Form 1065
    Form1065,
    /// 2 Form 1120-S
    Form1120S,
    /// 3 Form 1041
    Form1041,
    /// 4 Other
    Other {
        /// Form number as printed
        #[libertas_llm_doc_field]
        #[libertas_size(min = 1)]
        form: String,
    },
}

/// Form 8986 Part II B
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986Representative {
    /// PR (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub representative: bool,
    /// DI (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub designated_individual: bool,
    /// 1 First name
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub first_name: String,
    /// 2 Last name
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub last_name: String,
    /// 3 to 6 Street address, city or town, state and ZIP code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub address: String,
    /// 7 Area code and phone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub phone: String,
}

/// Form 8986 Part III
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986PassThrough {
    /// A and B Pass-through entity's name, address and TIN
    #[libertas_llm_doc_field]
    pub entity: crate::PrintedParty,
    /// C Pass-through partner's tax year end to which the adjustments relate
    /// [NullText]
    /// The date is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub tax_year_end: Option<u32>,
    /// D Name of the entity that issued the statement to the pass-through partner
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub issuer_name: String,
    /// E TIN of the entity that issued the statement to the pass-through partner
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub issuer_tin: String,
}

/// Form 8986 Part IV D
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form8986PartnerEntity {
    /// 1 Individual
    Individual,
    /// 2 S Corporation
    SCorporation,
    /// 3 Corporation
    Corporation,
    /// 4 Partnership
    Partnership,
    /// 5 Other
    Other,
}

/// One Form 8986 Part IV E row: As Reported, Change, Corrected
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986PercentRow {
    /// As Reported
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = -100, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub as_reported: Option<f64>,
    /// Change
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = -100, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub change: Option<f64>,
    /// Corrected
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = -100, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub corrected: Option<f64>,
}

/// One Form 8986 Part IV F or G row: As Reported, Change, Corrected
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986AmountRow {
    /// As Reported
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub as_reported: Option<i64>,
    /// Change
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub change: Option<i64>,
    /// Corrected
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub corrected: Option<i64>,
}

/// One Form 8986 Part V line
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986Item {
    /// (a) Schedule K-1/K-3 line number
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub line_number: String,
    /// (b) Line title
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub line_title: String,
    /// (c) Code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub code: String,
    /// (d) As reported
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub as_reported: Option<i64>,
    /// (e) Check if statement in Part VI
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub statement: bool,
    /// (f) Approved modifications (if applicable)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub modifications: Option<i64>,
    /// (g) Reviewed year adjustments net of approved modifications
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub adjustment: Option<i64>,
    /// (h) As corrected (column (d) combined with column (g))
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub corrected: Option<i64>,
}

/// Form 8986 Part V totals of columns (f), (g) and (h)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986Totals {
    /// Total of column (f)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub modifications: Option<i64>,
    /// Total of column (g)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub adjustment: Option<i64>,
    /// Total of column (h)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub corrected: Option<i64>,
}

/// One Form 8986 applicable penalty
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986Penalty {
    /// (a) Internal Revenue Code section
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub code_section: String,
    /// (b) Penalty description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub description: String,
    /// (c) Penalty rate
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub rate: Option<f64>,
    /// (d) Applicable line numbers above
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub line_numbers: String,
    /// (e) Total adjustments subject to the penalty
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub total: Option<i64>,
}

/// One Form 8986 Part VI statement
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8986Statement {
    /// (a) Line no./code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub line: String,
    /// (b) Statement
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub statement: String,
}

/// Your records for the Form 8986 adjustment
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct BbaAuditAnswers {
    /// Total 2025 Form 1040 line 1z wages on the filed or IRS-adjusted return
    /// [NullText]
    /// The prior-return wages have not been supplied
    /// [SetText]
    /// The prior-return wages are
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_wages: Option<i64>,
    /// Taxable income on the filed or IRS-adjusted 2025 Form 1040 line 15
    /// This calculation supports original and corrected taxable income below $100,000.
    /// [NullText]
    /// The prior taxable income has not been supplied
    /// [SetText]
    /// The prior taxable income is
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Tax structure: this calculation uses the IRS Tax Table below $100,000 taxable income.
    #[libertas_number(min = 0, max = 9999900)]
    pub prior_taxable_income: Option<i64>,
    /// Income tax on the filed or IRS-adjusted 2025 Form 1040 line 16
    /// For this case, lines 16, 18 and 24 must be equal; there were no other income taxes or credits.
    /// [NullText]
    /// The prior income tax has not been supplied
    /// [SetText]
    /// The prior income tax is
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_income_tax: Option<i64>,
    /// 2025 filing status used on the return and its Tax Table
    /// [NullText]
    /// The prior-return filing status has not been supplied
    /// [SetText]
    /// The prior-return filing status is
    #[libertas_default($null)]
    pub prior_status: Option<FederalStatus>,
    /// This is the only Form 8986 or Form 8978 adjustment for 2026, from a BBA audit rather than an AAR
    /// [NullText]
    /// The adjustment's scope has not been confirmed
    /// [SetText]
    /// Only one BBA-audit adjustment applies
    #[libertas_default($null)]
    pub single_audit_adjustment: Option<bool>,
    /// The filed 2025 return used ordinary Tax Table income tax, with only wages, this K-1 interest and the basic standard deduction
    /// Neither filer was age 65 or blind, no one could claim either filer as a dependent, no dependents were claimed, and no other adjustment or deduction applied. There was no Social Security, preferential income, QBI, NIIT, FTC, EIC or other credit, AMT, other tax, AGI-based deduction or phaseout, or prior correction, before or after the adjustment. Filed Forms 1040 lines 16, 18 and 24 were equal.
    /// [NullText]
    /// The prior return's scope has not been confirmed
    /// [SetText]
    /// The prior return meets this scope
    #[libertas_default($null)]
    pub simple_prior_return: Option<bool>,
    /// The interest change has no other deduction, credit, carryover, tax-attribute or 2026-income effect
    /// No inconsistent K-1 position, penalty, section 6603 deposit, prior payment toward this adjustment or other Form 8986 amount applies.
    /// [NullText]
    /// The adjustment's other effects have not been confirmed
    /// [SetText]
    /// No other effects apply
    #[libertas_default($null)]
    pub no_other_effects: Option<bool>,
    /// Form 8978 line 17 interest from the separate supporting computation
    /// Needed for an increase in prior-year income tax. Compute from the original 2025 return due date through the planned payment date using the applicable quarterly short-term rate plus five percentage points. A documented zero is valid. This interest is outside Form 1040 tax and balance.
    /// [NullText]
    /// The line 17 interest calculation has not been supplied
    /// [SetText]
    /// The documented line 17 interest is
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub line_17_interest: Option<i64>,
    /// The separate Form 8978 line 17 interest calculation is complete and documented
    /// Needed only for an increase in prior-year income tax.
    /// [NullText]
    /// The interest calculation has not been confirmed
    /// [SetText]
    /// The interest calculation is complete and documented
    #[libertas_default($null)]
    pub interest_calculation_prepared: Option<bool>,
}

impl FederalBbaAuditInterest {
    pub(crate) fn into_parts(self) -> (Form8986Paper, BbaAuditAnswers) {
        (
            Form8986Paper {
                original: true,
                corrected: false,
                tracking_number: self.source_id,
                audit_control_number: alloc::string::String::new(),
                submitting_entity: Some(Form8986SubmittingEntity::AuditedPartnership),
                entity_return: Some(Form8986EntityReturn::Form1065),
                partnership: self.partnership,
                representative: Default::default(),
                review_year_end: Some(crate::year_constants::PRIOR_TAX_YEAR * 10_000 + 1231),
                adjustment_year_end: Some(crate::year_constants::TAX_YEAR * 10_000 + 1231),
                extended_due_date: None,
                furnished_on: self.furnished_on,
                pass_through: Default::default(),
                partner: self.reviewed_year_partner,
                general_partner: false,
                limited_partner: true,
                domestic_partner: true,
                foreign_partner: false,
                partner_entity: Some(Form8986PartnerEntity::Individual),
                tax_exempt: false,
                profit: Default::default(),
                loss: Default::default(),
                capital: Default::default(),
                nonrecourse: Default::default(),
                qualified_nonrecourse: Default::default(),
                recourse: Default::default(),
                beginning_capital: Default::default(),
                capital_contributed: Default::default(),
                current_year_change: Default::default(),
                withdrawals: Default::default(),
                section_736_payments: Default::default(),
                ending_capital: Default::default(),
                items: alloc::vec![Form8986Item {
                    line_number: "5".into(),
                    line_title: "Interest income".into(),
                    code: alloc::string::String::new(),
                    as_reported: Some(self.original_partnership_interest),
                    statement: false,
                    modifications: None,
                    adjustment: Some(self.interest_adjustment),
                    corrected: Some(self.original_partnership_interest + self.interest_adjustment),
                }],
                totals: Form8986Totals {
                    modifications: None,
                    adjustment: Some(self.interest_adjustment),
                    corrected: Some(self.original_partnership_interest + self.interest_adjustment),
                },
                penalties: alloc::vec::Vec::new(),
                statements: alloc::vec::Vec::new(),
            },
            BbaAuditAnswers {
                prior_wages: Some(self.prior_wages),
                prior_taxable_income: Some(self.prior_taxable_income),
                prior_income_tax: Some(self.prior_income_tax),
                prior_status: Some(self.prior_status),
                single_audit_adjustment: Some(self.single_audit_adjustment),
                simple_prior_return: Some(self.simple_prior_return),
                no_other_effects: Some(self.no_other_effects),
                line_17_interest: Some(self.line_17_interest),
                interest_calculation_prepared: Some(self.interest_calculation_prepared),
            },
        )
    }
}

/// Complete source facts used by the Form 8978 calculation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FederalBbaAuditInterest {
    /// Partnership name, TIN and address
    pub partnership: crate::PrintedParty,
    /// Reviewed-year partner name, TIN and address
    pub reviewed_year_partner: crate::PrintedParty,
    /// Accepted document identity
    pub id: i64,
    /// Form 8986 tracking number; use its audit control number or issuing entity tax ID only when the tracking number is blank
    pub source_id: String,
    /// Date the partnership furnished Form 8986 to you in 2026
    pub furnished_on: u32,
    /// Signed Form 8986 Part V column (g) change to 2025 Schedule K-1 box 5 portfolio interest
    /// Enter the adjustment net of approved modifications, not column (h)'s as-corrected total. Check that column (h) equals column (d) plus column (g). It must be nonzero and in whole dollars.
    pub interest_adjustment: i64,
    /// 2025 interest originally reported from this partnership's Schedule K-1 box 5
    pub original_partnership_interest: i64,
    /// Total 2025 Form 1040 line 1z wages on the filed or IRS-adjusted return
    pub prior_wages: i64,
    /// Taxable income on the filed or IRS-adjusted 2025 Form 1040 line 15
    pub prior_taxable_income: i64,
    /// Income tax on the filed or IRS-adjusted 2025 Form 1040 line 16
    /// For this narrow case, lines 16, 18, and 24 must be equal; there were no other income taxes or credits.
    pub prior_income_tax: i64,
    /// 2025 filing status used on the return and its Tax Table
    pub prior_status: FederalStatus,
    /// This is the only Form 8986 or Form 8978 adjustment for the 2026 reporting year, from a BBA audit rather than an AAR
    pub single_audit_adjustment: bool,
    /// The filed 2025 return used ordinary Tax Table income tax, with only wages and this K-1 interest and the basic standard deduction
    /// Neither filer was age 65 or blind, no one could claim either filer as a dependent, no dependents were claimed, and no other adjustment or deduction applied.
    /// There was no Social Security, preferential income, QBI, NIIT, FTC, EIC or other credit, AMT, other tax, AGI-based deduction or phaseout, or prior correction, before or after the adjustment. Filed Forms 1040 lines 16, 18 and 24 were equal.
    pub simple_prior_return: bool,
    /// The interest change has no other deduction, credit, carryover, tax-attribute, or 2026-income effect; no inconsistent K-1 position, penalty, section 6603 deposit, prior payment toward this adjustment, or other Form 8986 amount applies
    pub no_other_effects: bool,
    /// Form 8978 line 17 interest from the separate supporting computation, excluded from Form 1040 tax and balance
    /// For an increase in prior-year income tax, compute interest from the original 2025 return due date through the planned payment date using the applicable quarterly short-term rate plus five percentage points. Confirm the supporting calculation. Zero is valid only when the calculation supports zero.
    pub line_17_interest: i64,
    /// The separate Form 8978 line 17 interest calculation is complete and documented
    pub interest_calculation_prepared: bool,
}
