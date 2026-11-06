#[allow(unused_imports)]
use super::*;
use crate::validation::ValidationIssue;

/// Government unemployment repayment made in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct UnemploymentRepayment {
    /// Year these benefits were received
    /// Use the original payment year, not the year an overpayment notice arrived. Earlier-year repayments do not reduce 2026 benefits. Supported return-wide repayments of $3,000 or less give no deduction; larger amounts need prior-return deduction-versus-credit evaluation.
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2026)]
    pub received_year: u32,
    /// Benefit principal actually repaid in 2026
    /// Exclude interest, penalties and amounts already netted out of the gross benefits entered above. Include each repayment once, including documented amounts recovered from later benefit payments.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Unemployment compensation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalUnemployment {
    /// Accepted identities of the independently claimable boxes
    #[libertas_hidden]
    #[libertas_read_only]
    pub component_ids: crate::GovernmentComponentIds,

    /// Government unemployment benefits actually paid in 2026, before listed repayments
    /// Reconcile Form 1099-G box 1 with the agency payment records. Enter gross benefits once; do not subtract the repayments listed below first. Do not include an incorrect identity-theft payment you never received.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Federal income tax withheld
    /// Use the final actual withholding record; the App does not reduce this amount when benefit principal is repaid.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Repayments made in 2026 for this benefit source
    /// Leave empty if none. Separate original receipt years. The App nets same-year benefits and repayments. Earlier-year amounts are combined across this return for the $3,000 review threshold, including net Social Security repayments.
    /// ----
    /// Repayment
    pub repayments: Vec<UnemploymentRepayment>,
    /// Ordinary taxable governmental unemployment and reconciled payment records
    /// No unrecovered employee contributions to the program, private or union-fund benefits, employer-paid supplemental wages, treaty exclusion or other unentered special treatment. Every repayment is recorded once above. The Form 1099-G payer and the repayment records show this. No leaves this statement out of the estimate with a notice; that treatment is not supported.
    pub ordinary: bool,
    /// Taxable grants received in 2026 — Form 1099-G box 6
    /// Reconcile Form 1099-G box 6 with the granting agency's records. Enter zero when this document has no box 6 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable_grants: i64,
    /// Agriculture program payments received in 2026 — Form 1099-G box 7
    /// Reconcile Form 1099-G box 7 with the paying agency's records. A farm or farm rental must already be entered elsewhere on this return; the App adds this amount to that activity's own fully taxable agricultural-program income (Schedule F line 4a/4b, Form 4835 line 3). Enter zero when this document has no box 7 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub agriculture_payment: i64,
    /// Who received the box 6 grant
    /// A grant to you is Schedule 1 line 8z; a grant to a business you run is that business's Schedule C income.
    #[libertas_default($null)]
    pub grant_use: Option<crate::FederalGrantUse>,
    /// Reemployment trade adjustment assistance (RTAA) payments received in 2026 — Form 1099-G box 5
    /// Other income on Schedule 1 line 8z. Enter zero when this document has no box 5 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rtaa: i64,
    /// Market gain received in 2026 — Form 1099-G box 9
    /// The App computes Commodity Credit Corporation market gain independently from this return's own loan records and cannot reconcile it to this box. Enter zero when this document has no box 9 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_gain: i64,
    /// Paid family leave benefits received in 2026 — Form 1099-G box 10
    /// Enter the full box 10 amount only when none of it is your own paid-medical-leave contributions returned to you; the App cannot separate an excludable medical-leave share. Enter zero when this document has no box 10 amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub family_leave: i64,
    /// This box 10 amount includes a paid medical leave portion
    /// Choose No only when the entire box 10 amount is family leave. The excludable medical-leave share needs facts the App does not calculate: Check the payer’s family/medical-leave statement. Yes taxes the whole box 10 amount, with a note.
    pub family_leave_includes_medical: bool,
}

/// 1099-G Certain Government Payments
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-G (Certain Government Payments) exactly as printed, every box in the form's own order. Read each printed party block into payer and recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the CORRECTED checkbox into corrected, and the payer telephone number and the account number as printed. box only when it is marked. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. The supplied year is the year the form was issued for; box 3 is a different, earlier year. Read box 1 unemployment compensation, box 2 state or local income tax refunds, credits, or offsets, box 4 federal income tax withheld and box 10 family leave benefits, returning zero for each of these boxes that is clearly blank, and box 5 RTAA payments, box 6 taxable grants, box 7 agriculture payments and box 9 market gain, returning null for each of these boxes that is clearly blank, never zero. Report an error when box 1, box 2 and box 7 are all blank. Read box 3 as the actual four-digit year printed, returning null when box 3 is clearly blank; report an illegible box 3 year as an error. Check box 8 only when the 'Check if box 2 is trade or business income' box is marked. Read all printed boxes into one form, including when the same paper reports both unemployment compensation in box 1 and a state or local refund in box 2. Each paper is one entry. Do not report repayments, who received the box 6 grant, whether box 10 includes paid medical leave, how state taxes were deducted that year, or whether a special treatment applies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read state boxes 11a, 11b and 12. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. These refund jurisdiction fields do not supply a missing state in state_details. Never infer a state withholding row from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.unemployment_compensation,@.state_or_local_income_tax_refunds,@.federal_income_tax_withheld,@.rtaa_payments,@.taxable_grants,@.agriculture_payments,@.market_gain,@.family_leave_benefits,@.account_number"
)]
pub struct Form1099GPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, street address (including apt. no.), city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Unemployment compensation
    /// Leave zero on a form that shows only a box 2 refund.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unemployment_compensation: i64,
    /// Box 2 State or local income tax refunds, credits, or offsets
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub state_or_local_income_tax_refunds: i64,
    /// Box 3 Box 2 amount is for tax year
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// Box 3 tax year is
    #[libertas_llm_doc_field(blank = $null)]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 0, max = 2025)]
    #[libertas_format("0")]
    #[libertas_default($null)]
    pub box_2_tax_year: Option<i32>,
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub federal_income_tax_withheld: i64,
    /// Box 5 RTAA payments
    /// Reemployment trade adjustment assistance; other income on Schedule 1 line 8z.
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub rtaa_payments: Option<i64>,
    /// Box 6 Taxable grants
    /// Leave out a qualified disaster relief grant; enter it under Other income.
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub taxable_grants: Option<i64>,
    /// Box 7 Agriculture payments
    /// The App adds it to the farm or farm rental related to this payment (Schedule F line 4, Form 4835 line 3). Leave out CRP and crop disaster payments you enter on the farm.
    /// [NullText]
    /// Box 7 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub agriculture_payments: Option<i64>,
    /// Box 8 Check if box 2 is trade or business income
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub trade_or_business_income: bool,
    /// Box 9 Market gain
    /// [NullText]
    /// Box 9 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub market_gain: Option<i64>,
    /// Box 10 Family leave benefits
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub family_leave_benefits: i64,
    /// Boxes 11a State, 11b State identification no. and 12 State income tax withheld
    #[libertas_llm_doc_field]
    pub state_details: crate::StateWithholdingFields,
}

impl Form1099GPaper {
    pub(crate) fn payments_admission_issue(&self) -> Option<ValidationIssue> {
        let payments = i128::from(self.unemployment_compensation)
            + i128::from(self.rtaa_payments.unwrap_or(0))
            + i128::from(self.taxable_grants.unwrap_or(0))
            + i128::from(self.agriculture_payments.unwrap_or(0))
            + i128::from(self.market_gain.unwrap_or(0))
            + i128::from(self.family_leave_benefits);
        if i128::from(self.federal_income_tax_withheld) > payments {
            Some(
                ValidationIssue::new("FORM_1099G_WITHHOLDING_ABOVE_BENEFITS")
                    .at_field("federal_income_tax_withheld"),
            )
        } else {
            None
        }
    }
    pub(crate) fn refund_admission_issue(&self) -> Option<ValidationIssue> {
        if self.refund_withholding() > self.state_or_local_income_tax_refunds {
            Some(
                ValidationIssue::new("REFUND_WITHHOLDING_ABOVE_REFUND")
                    .at_field("federal_income_tax_withheld"),
            )
        } else {
            None
        }
    }
    // Box 4 belongs to the payments part when this paper also reports payments.
    fn refund_withholding(&self) -> i64 {
        if self.has_payments() { 0 } else { self.federal_income_tax_withheld }
    }
    /// The refund part's tax year: a blank box 3 reads as the prior year.
    pub(crate) fn refund_year(&self) -> i32 {
        let year = self.box_2_tax_year.unwrap_or(0);
        if year == 0 { 2025 } else { year }
    }
    /// Whether any box other than the box 2 refund shows a payment.
    pub(crate) fn has_payments(&self) -> bool {
        let Self {
            unemployment_compensation,
            rtaa_payments,
            taxable_grants,
            agriculture_payments,
            market_gain,
            family_leave_benefits,
            ..
        } = self;
        [
            *unemployment_compensation,
            rtaa_payments.unwrap_or(0),
            taxable_grants.unwrap_or(0),
            agriculture_payments.unwrap_or(0),
            market_gain.unwrap_or(0),
            *family_leave_benefits,
        ]
        .iter()
        .any(|amount| *amount != 0)
    }
}

// One-way: the paper fills the unemployment part the calculations read; every
// printed box either reaches it or is named as not read, so a new box cannot
// be forgotten.
impl From<&Form1099GPaper> for Form1099G {
    fn from(paper: &Form1099GPaper) -> Self {
        let Form1099GPaper {
            calendar_year: _,
            corrected: _,
            payer,
            payer_telephone: _,
            recipient,
            account_number: _,
            unemployment_compensation,
            // Box 2, 3 and 8 are the refund part (`Refund1099G`).
            state_or_local_income_tax_refunds: _,
            box_2_tax_year: _,
            federal_income_tax_withheld,
            rtaa_payments,
            taxable_grants,
            agriculture_payments,
            trade_or_business_income: _,
            market_gain,
            family_leave_benefits,
            state_details,
        } = paper;
        Form1099G {
            payer: payer.clone(),
            recipient: recipient.clone(),
            amount: *unemployment_compensation,
            withholding: *federal_income_tax_withheld,
            rtaa: rtaa_payments.unwrap_or(0),
            taxable_grants: taxable_grants.unwrap_or(0),
            agriculture_payment: agriculture_payments.unwrap_or(0),
            market_gain: market_gain.unwrap_or(0),
            family_leave: *family_leave_benefits,
            state_details: state_details.clone(),
        }
    }
}

// One-way: the paper fills the refund part. Box 4 withholding is one amount of
// the form: when the paper also reports payments, that part carries it.
impl From<&Form1099GPaper> for Refund1099G {
    fn from(paper: &Form1099GPaper) -> Self {
        Self::with_place(paper, None, "")
    }
}

impl Refund1099G {
    /// The refund part from the printed paper and the refund place its wrapper records
    /// (the paper does not print which state or locality paid the refund).
    pub(crate) fn with_place(
        paper: &Form1099GPaper,
        refund_state: Option<crate::TaxState>,
        refund_locality: &str,
    ) -> Self {
        let Form1099GPaper {
            calendar_year: _,
            corrected: _,
            payer,
            payer_telephone: _,
            recipient,
            account_number: _,
            // Boxes 1, 5, 6, 7, 9 and 10 are the unemployment part (`Form1099G`).
            unemployment_compensation: _,
            state_or_local_income_tax_refunds,
            box_2_tax_year: _,
            federal_income_tax_withheld: _,
            rtaa_payments: _,
            taxable_grants: _,
            agriculture_payments: _,
            // The business-income mark does not move a figure until the
            // deduction answer is made (RefundDeduction::BusinessExpense).
            trade_or_business_income: _,
            market_gain: _,
            family_leave_benefits: _,
            state_details,
        } = paper;
        Refund1099G {
            payer: payer.clone(),
            recipient: recipient.clone(),
            amount: *state_or_local_income_tax_refunds,
            year: paper.refund_year(),
            withholding: paper.refund_withholding(),
            state_details: crate::RefundStateFields {
                refund_state,
                refund_locality: refund_locality.into(),
                taxes: state_details.clone(),
            },
        }
    }
}

/// Unemployment or agriculture part of a Form 1099-G
/// The internal model filled from the paper `Form1099GPaper`.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1099G {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Box 1 unemployment compensation
    /// Leave zero on a form that shows only box 7 agriculture payments.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 RTAA payments
    /// Reemployment trade adjustment assistance; other income on Schedule 1 line 8z.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rtaa: i64,
    /// Box 6 taxable grants
    /// Leave out a qualified disaster relief grant; enter it under Other income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable_grants: i64,
    /// Box 7 agriculture payments
    /// The App adds it to the farm or farm rental related to this payment (Schedule F line 4, Form 4835 line 3). Leave out CRP and crop disaster payments you enter on the farm.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub agriculture_payment: i64,
    /// Box 9 market gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_gain: i64,
    /// Box 10 family leave benefits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub family_leave: i64,
    /// Boxes 11a, 11b and 12 state
    pub state_details: crate::StateWithholdingFields,
}

/// Accepted Form 1099-G government payments
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct GovernmentPaymentSource {
    /// Form 1099-G as printed
    pub form: Form1099GPaper,
    /// State that paid the box 2 refund
    /// Saving fills it once from the paying agency's printed address when that shows a state; a state already recorded is kept. Otherwise identify the refund-paying agency's state, not your current home state.
    /// [NullText]
    /// The paying agency's printed address shows no state
    #[libertas_default($null)]
    pub refund_state: Option<crate::TaxState>,
    /// Locality that paid a local box 2 refund
    /// The local refund agency or locality; leave empty when none.
    #[libertas_default("")]
    pub refund_locality: String,
    /// Accepted identities of the independently claimable boxes
    #[libertas_hidden]
    #[libertas_read_only]
    pub component_ids: crate::GovernmentComponentIds,
    /// Who received the box 6 grant
    #[libertas_default($null)]
    pub grant_use: Option<crate::GrantUse>,
    /// Box 10 includes paid medical leave
    #[libertas_default(false)]
    pub family_leave_medical: bool,
    /// Benefits you repaid in 2026
    /// ----
    /// Repayment
    pub repayments: Vec<UnemploymentRepayment>,
    /// Union or private-fund benefits, employee contributions, or a treaty exclusion
    #[libertas_default(false)]
    pub special: bool,
    /// Federal deduction treatment of the refunded tax
    pub history: crate::TaxRefundHistory,
}

impl GovernmentPaymentSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099GPaper) -> Self {
        Self {
            form,
            component_ids: Default::default(),
            grant_use: None,
            family_leave_medical: false,
            repayments: Vec::new(),
            special: false,
            history: crate::TaxRefundHistory::Unanswered,
            refund_state: None,
            refund_locality: alloc::string::String::new(),
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.grant_use = accepted.grant_use.clone();
        self.family_leave_medical = accepted.family_leave_medical;
        self.repayments = accepted.repayments.clone();
        self.special = accepted.special;
        self.history = accepted.history.clone();
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&GovernmentPaymentSource> for FederalUnemployment {
    fn from(source: &GovernmentPaymentSource) -> Self {
        let GovernmentPaymentSource {
            form,
            component_ids,
            grant_use,
            family_leave_medical,
            repayments,
            special,
            history: _,
            // The refund place belongs to the refund part only.
            refund_state: _,
            refund_locality: _,
        } = source;
        let Form1099G {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            amount,
            withholding,
            rtaa,
            taxable_grants,
            agriculture_payment,
            market_gain,
            family_leave,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = &Form1099G::from(form);
        FederalUnemployment {
            component_ids: component_ids.clone(),
            amount: *amount,
            withholding: *withholding,
            repayments: repayments.clone(),
            ordinary: !*special,
            taxable_grants: *taxable_grants,
            agriculture_payment: *agriculture_payment,
            grant_use: grant_use.as_ref().map(crate::GrantUse::record),
            rtaa: *rtaa,
            market_gain: *market_gain,
            family_leave: *family_leave,
            family_leave_includes_medical: *family_leave_medical,
        }
    }
}

/// Who received the box 6 grant
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum GrantUse {
    /// You, not for an activity
    /// Other income on Schedule 1 line 8z.
    You,
}

/// 1099-G state or local tax refund
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Refund1099G {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Box 2 state or local income tax refund
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Box 3 tax year
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 2010, max = 2025)]
    #[libertas_default(2025)]
    pub year: i32,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Refund jurisdiction and state withholding
    pub state_details: crate::RefundStateFields,
}

/// The state or local refund as the calculations read it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FederalTaxRefund {
    /// Refund, credit or offset received in 2026
    pub amount: i64,
    /// Prior federal deduction treatment
    pub history: crate::TaxRefundHistory,
    /// Federal income tax withheld
    pub withholding: i64,
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&GovernmentPaymentSource> for FederalTaxRefund {
    fn from(source: &GovernmentPaymentSource) -> Self {
        let GovernmentPaymentSource {
            form,
            component_ids: _,
            grant_use: _,
            family_leave_medical: _,
            repayments: _,
            special: _,
            history,
            refund_state,
            refund_locality,
        } = source;
        let Refund1099G {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            amount,
            year,
            withholding,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = &Refund1099G::with_place(form, *refund_state, refund_locality);
        let mut history = history.clone();
        // Box 3 is the year the refunded tax was deducted.
        if let crate::TaxRefundHistory::IncomeTax { deduction_year, .. } = &mut history {
            *deduction_year = *year;
        }
        FederalTaxRefund { amount: *amount, history, withholding: *withholding }
    }
}

/// How you deducted state taxes that year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RefundDeduction {
    /// Standard deduction
    StandardDeduction,
    /// Itemized, deducting sales tax
    SalesTax,
    /// Itemized, deducting state income tax
    IncomeTax {
        /// That year's Schedule A line 5a
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        income_paid: i64,
        /// Line 5d
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salt_paid: i64,
        /// Line 5e
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salt_deducted: i64,
        /// Line 17 total itemized deductions
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        itemized: i64,
        /// The standard deduction you could have taken that year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        standard: i64,
        /// Sales tax you could have deducted instead
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        sales_available: i64,
        /// Real estate and personal property taxes refunded to you in 2026 for that year
        /// Publication 525 Worksheet 2a line 2. They are figured with the income tax refund against the same Schedule A taxes and the same limit, and any taxable part goes on Schedule 1 line 8z. A refund of a tax you deducted for a business goes on that business.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        property_refunded: i64,
    },
    /// Deducted as a business expense (box 8 is checked)
    /// A refund of a state or local tax that applies only to business income, deducted on a business schedule. The App adds it to that business's Schedule C income instead of Schedule 1 line 1.
    BusinessExpense {
        /// Business
        /// The null answer uses your only business on Self-employment. A farm's refund goes on that farm's other income.
        /// [NullText]
        /// Use my only saved business
        /// [SetText]
        /// The refunded deduction belongs to this saved business
        #[libertas_enum_source("$.businesses")]
        #[libertas_default($null)]
        business: Option<i64>,
    },
    /// Payments made over several years
    SeveralYears,
}
