use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Original issue discount, Form 1099-OID
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalOid {
    /// Premium reducing exempt OID, from the identified annual bond worksheet
    /// Separate from taxable box 10 premium; never apply it to taxable OID.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,

    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    #[libertas_default($null)]
    pub nominee: Option<crate::OidNominee>,
    /// Personal annual reporting adjustments
    #[libertas_default($null)]
    pub adjustments: Option<crate::InterestAdjustments>,

    /// Box 9 investment expenses, not an additional deduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Original issue discount for the year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub oid: i64,
    /// Other periodic interest paid in addition to the discount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Early withdrawal penalty on savings
    /// Copy Form 1099-INT box 2 or Form 1099-OID box 3. Report the full discount above, without subtracting this penalty. The deduction may exceed this year's interest. This is not an IRA withdrawal penalty or an account maintenance fee.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_penalty: i64,
    /// Federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Market discount treated as interest, not as part of the discount above
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Acquisition premium that reduces the discount above
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Total tax-exempt original issue discount (box 11), including private-activity bond discount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Bond premium that reduces the discount above
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Specified private-activity bond discount included in the tax-exempt amount above
    /// From the payer's supplemental statement, not a numbered box on Form 1099-OID; do not enter it twice as income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// A treatment this page does not calculate
    /// Form 1099-OID box 7 and your ownership or instrument records show it. Any choice but None of these leaves this statement out of the estimate with a notice; those treatments are not supported.
    pub special: OidSpecial,
    /// Original issue discount on U.S. Treasury obligations (box 8)
    /// May be negative, such as a TIPS deflation adjustment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Printed facts of the Form 1099-OID that no figure reads
    pub printed_facts: OidPrintedFacts,
}
impl FederalOid {
    // This payer's taxable original issue discount: box 1, box 2 and box 8
    // less the box 6 acquisition premium and box 10 bond premium, not below
    // zero (box 8 may be negative), plus the box 5 market discount.
    pub(crate) fn net_taxable(&self) -> i64 {
        if let Some(records) = self.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref())
        {
            return records.taxable();
        }
        crate::kernel::ratio::positive(
            self.oid + self.other_periodic_interest + self.treasury_oid
                - self.acquisition_premium
                - self.bond_premium,
        ) + self.market_discount
    }
}

/// A treatment of original issue discount this page does not calculate
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum OidSpecial {
    /// None of these
    #[default]
    NoneOfThese,
    /// A stripped or contingent-payment obligation
    /// The box 7 description shows it.
    Obligation,
    /// Another treatment this page does not calculate
    Another,
}
impl OidSpecial {
    // Answers and unknowns: the notice of the treatment that leaves the
    // statement out of the estimate; None for no treatment.
    pub(crate) fn notice(self) -> Option<&'static str> {
        match self {
            Self::NoneOfThese => None,
            Self::Obligation => Some("WARN_OID_OBLIGATION_LEFT_OUT"),
            Self::Another => Some("WARN_OID_OTHER_LEFT_OUT"),
        }
    }
}

// Form 1099-OID (Rev. January 2024), Copy B: https://www.irs.gov/pub/irs-pdf/f1099oid.pdf
/// Form 1099-OID Original Issue Discount
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-OID exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Return the payer's name as printed, without truncation, or an empty string when it is absent. Read the CORRECTED (if checked) checkbox into corrected and the payer's telephone number into payer_telephone, returning false or empty text when absent. Read box 1 original_issue_discount_for_the_year; a clearly blank box 1 means none, so return zero. Read box 2 other_periodic_interest, returning zero when clearly blank. In the other boxes read box 3 early_withdrawal_penalty, box 4 federal_income_tax_withheld, box 5 market_discount, box 6 acquisition_premium, box 7 description as printed text (empty when blank), box 8 original_issue_discount_on_us_treasury_obligations (keep a printed minus sign), box 9 investment_expenses, box 10 bond_premium, and box 11 tax_exempt_oid, returning zero for each clearly blank amount. Read the FATCA filing requirement checkbox into fatca_filing_requirement and the account number into account_number. Return the printed boxes only; the private activity bond discount and any special treatment are the filer's topic answers. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer federal ID and address and state boxes 12 through 14. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. From an included supplemental statement, read the identified issuing-state allocations into exempt_sources, selecting Amount for printed dollar amounts or Percentage for printed percentages. Return Percentage.value as a decimal number from 0 to 100 without a percent sign; for example, 27.43% is 27.43, not 0.2743. Read the fund's U.S. government obligation percentage on the same scale, returning a decimal number without a percent sign for a reported value, or null when absent; an explicit zero is zero. Return an empty allocation list when no allocation is supplied. Do not convert a percentage to dollars or assume the withholding state is the bond-issuing state. Do not add box 9 to box 1. Do not infer yield, ownership, premium or contingent-payment adjustments. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.original_issue_discount_for_the_year,@.other_periodic_interest,@.early_withdrawal_penalty,@.federal_income_tax_withheld,@.market_discount,@.acquisition_premium,@.original_issue_discount_on_us_treasury_obligations,@.investment_expenses,@.bond_premium,@.tax_exempt_oid,@.account_number"
)]
pub struct Form1099OidPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, street address (including apt. no.), city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 Original issue discount for the year
    /// This may not be the correct figure to report on your income tax return.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub original_issue_discount_for_the_year: i64,
    /// Box 2 Other periodic interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Box 3 Early withdrawal penalty
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_withdrawal_penalty: i64,
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub federal_income_tax_withheld: i64,
    /// Box 5 Market discount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Box 6 Acquisition premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Box 7 Description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub description: String,
    /// Box 8 Original issue discount on U.S. Treasury obligations
    /// May be negative, such as a TIPS deflation adjustment.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub original_issue_discount_on_us_treasury_obligations: i64,
    /// Box 9 Investment expenses
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Box 10 Bond premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Box 11 Tax-exempt OID
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_oid: i64,
    /// FATCA filing requirement
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Boxes 12–14 State, State identification no., State tax withheld, and supplemental information
    #[libertas_llm_doc_field]
    pub state_details: crate::InvestmentStateFields,
}

/// Printed Form 1099-OID facts that no figure reads
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidPrintedFacts {
    /// CORRECTED (if checked)
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S telephone no.
    #[libertas_default("")]
    pub payer_telephone: String,
    /// Box 7 Description
    #[libertas_default("")]
    pub description: String,
    /// FATCA filing requirement
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Account number (see instructions)
    #[libertas_default("")]
    pub account_number: String,
}

/// Accepted Form 1099-OID
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1099OidSource {
    /// Form 1099-OID as printed
    pub form: Form1099OidPaper,
    /// Applicable personal follow-up choices
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: OidCalls,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    /// [NullText]
    /// All reported OID and interest components belong to this return
    /// [SetText]
    /// These OID and interest components belong to someone outside this return
    #[libertas_hidden]
    pub nominee: Option<OidNomineeAnswer>,
    /// Personal annual reporting adjustments
    /// [NullText]
    /// Use the reported OID and interest without personal annual adjustments
    /// [SetText]
    /// My personal annual OID and interest adjustments are
    #[libertas_default($null)]
    #[libertas_hidden]
    pub adjustments: Option<crate::InterestAdjustments>,
    /// Premium reducing exempt OID, from the identified annual bond worksheet
    /// Separate from taxable box 10 premium; never apply it to taxable OID.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub tax_exempt_bond_premium: i64,
    /// Private activity bond discount included in box 11
    /// From the payer's supplemental statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub private_activity: i64,
    /// A treatment this page does not calculate
    /// A box 7 description that is a stripped or contingent-payment obligation, or another unentered OID treatment. Nominee components have their own personal record below. Any choice but None of these leaves this statement out of the estimate with a notice.
    /// [NullText]
    /// No treatment answer yet
    /// [SetText]
    /// My treatment answer
    #[libertas_default($null)]
    #[libertas_hidden]
    pub special: Option<OidSpecial>,
}

impl Form1099OidSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099OidPaper) -> Self {
        Self {
            form,
            calls: OidCalls::default(),
            nominee: None,
            adjustments: None,
            tax_exempt_bond_premium: 0,
            private_activity: 0,
            special: None,
        }
    }
    /// Keep the topic answers, the treatment among them, through a paper correction.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.special = accepted.special;
        self.nominee = accepted.nominee.clone();
        self.adjustments = accepted.adjustments.clone();
        self.tax_exempt_bond_premium = accepted.tax_exempt_bond_premium;
        self.private_activity = accepted.private_activity;
    }
}

// One-way: every printed box and answer either reaches the internal model or
// is named as not read by any calculation, so a new one cannot be forgotten.
impl From<&Form1099OidSource> for FederalOid {
    fn from(source: &Form1099OidSource) -> Self {
        let Form1099OidSource {
            form,
            // Presentation only: which follow-ups the page offers.
            calls: _,
            nominee,
            adjustments,
            tax_exempt_bond_premium,
            private_activity,
            special,
        } = source;
        let Form1099OidPaper {
            calendar_year: _,
            corrected,
            // The parties name and assign the document; no figure reads them.
            payer: _,
            payer_telephone,
            recipient: _,
            original_issue_discount_for_the_year,
            other_periodic_interest,
            early_withdrawal_penalty,
            federal_income_tax_withheld,
            market_discount,
            acquisition_premium,
            description,
            original_issue_discount_on_us_treasury_obligations,
            investment_expenses,
            bond_premium,
            tax_exempt_oid,
            fatca_filing_requirement,
            account_number,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = form;
        Self {
            tax_exempt_bond_premium: *tax_exempt_bond_premium,
            nominee: nominee.as_ref().map(|answer| answer.record().clone()),
            adjustments: adjustments.clone(),
            investment_expenses: *investment_expenses,
            oid: *original_issue_discount_for_the_year,
            other_periodic_interest: *other_periodic_interest,
            early_penalty: *early_withdrawal_penalty,
            withholding: *federal_income_tax_withheld,
            market_discount: *market_discount,
            acquisition_premium: *acquisition_premium,
            exempt: *tax_exempt_oid,
            bond_premium: *bond_premium,
            private_activity: *private_activity,
            special: special.unwrap_or_default(),
            treasury_oid: *original_issue_discount_on_us_treasury_obligations,
            // Printed facts no figure reads; kept so a child's statement reopens whole.
            printed_facts: OidPrintedFacts {
                corrected: *corrected,
                payer_telephone: payer_telephone.clone(),
                description: description.clone(),
                fatca_filing_requirement: *fatca_filing_requirement,
                account_number: account_number.clone(),
            },
        }
    }
}

/// Applicable personal answers for Form 1099-OID
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidCalls {
    /// Allowed choices for this answer
    pub nominee: Vec<i32>,
}

/// Reported personal allocation for this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OidNomineeAnswer {
    /// Enter the allocation
    Reported {
        /// Allocation
        value: crate::OidNominee,
    },
}
impl From<crate::OidNominee> for OidNomineeAnswer {
    fn from(value: crate::OidNominee) -> Self {
        Self::Reported { value }
    }
}
impl OidNomineeAnswer {
    pub(crate) fn record(&self) -> &crate::OidNominee {
        let Self::Reported { value } = self;
        value
    }
}

impl core::ops::Deref for OidNomineeAnswer {
    type Target = crate::OidNominee;
    fn deref(&self) -> &Self::Target {
        self.record()
    }
}
impl core::ops::DerefMut for OidNomineeAnswer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Reported { value } = self;
        value
    }
}
