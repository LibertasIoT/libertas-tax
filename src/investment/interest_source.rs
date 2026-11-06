use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Interest
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalInterest {
    /// Interest entered from private-loan records instead of a Form 1099-INT
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(false)]
    pub private_loan: bool,
    /// Buyer facts when the interest came from financing a home sold to its buyer
    #[libertas_default($null)]
    pub seller_mortgage: Option<crate::SellerMortgageParty>,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    #[libertas_default($null)]
    pub nominee: Option<crate::InterestNominee>,
    /// Personal annual reporting adjustments
    #[libertas_default($null)]
    pub adjustments: Option<crate::InterestAdjustments>,

    /// Box 5 investment expenses; accepted without an investment-expense deduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Box 10 current-election market discount, separate from boxes 1 and 3
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Foreign country or U.S. territory printed in box 7
    #[libertas_default("")]
    pub foreign_country: String,
    /// Ordinary taxable bank interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: i64,
    /// Interest on U.S. savings bonds and Treasury obligations (box 3)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Early withdrawal penalty on savings
    /// Copy Form 1099-INT box 2 or Form 1099-OID box 3. Report the full interest above, without subtracting this penalty. The deduction may exceed this year's interest. This is not an IRA withdrawal penalty or an account maintenance fee.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_penalty: i64,
    /// Total tax-exempt interest, including private-activity bond interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Specified private-activity bond interest included in the tax-exempt amount
    /// Copy Form 1099-INT box 9; do not enter it twice as income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// A treatment this page does not calculate
    /// Your treaty and ownership records show it. Any choice but None of these leaves this statement out of the estimate with a notice; those treatments are not supported.
    pub special: InterestSpecial,
    /// Bond premium (box 11), reducing the taxable interest above
    /// Copy Form 1099-INT box 11. Reduces this payer's own taxable interest above, not below zero; it does not carry over to another payer's statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Bond premium on Treasury obligations (box 12), reducing the taxable interest above
    /// Copy Form 1099-INT box 12. Reduces this payer's own taxable interest above, not below zero; it does not carry over to another payer's statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Bond premium on tax-exempt bonds (box 13), reducing the tax-exempt interest above
    /// Copy Form 1099-INT box 13. Reduces this payer's own tax-exempt interest above, not below zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Accrued interest paid to the seller when this bond was purchased
    /// Subtracted from this payer's taxable interest above, not below zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub accrued_interest: i64,
    /// Foreign tax paid (box 6)
    /// Read by the foreign tax credit: the section 904(j) payee-statement election takes it; a Form 1116 source records it again by country.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
    /// The part paid after the owner's death
    /// Only for a person who died in 2026: the time after the death belongs to the estate or another recipient and is left off the final return.
    #[libertas_default($null)]
    pub after_death: Option<crate::InterestAfterDeath>,
    /// Printed facts of the Form 1099-INT that no figure reads
    pub printed_facts: InterestPrintedFacts,
}

/// Stored interest source
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InterestSource {
    /// Form 1099-INT as printed and its personal answers
    Int1099(Form1099IntSource),
    /// Interest entered from private-loan or seller-financing records
    Record(FederalInterest),
}

impl InterestSource {
    pub(crate) fn adjustments(&self) -> Option<&crate::InterestAdjustments> {
        match self {
            Self::Int1099(source) => source.adjustments.as_ref(),
            Self::Record(data) => data.adjustments.as_ref(),
        }
    }
    pub(crate) fn adjustments_mut(&mut self) -> Option<&mut crate::InterestAdjustments> {
        match self {
            Self::Int1099(source) => source.adjustments.as_mut(),
            Self::Record(data) => data.adjustments.as_mut(),
        }
    }
}

impl From<&InterestSource> for FederalInterest {
    fn from(source: &InterestSource) -> Self {
        match source {
            InterestSource::Int1099(source) => Self::from(source),
            InterestSource::Record(data) => data.clone(),
        }
    }
}

impl FederalInterest {
    // This payer's taxable interest: box 1 and box 3, less the box 11 and box
    // 12 bond premium and accrued interest paid on purchase, each reducing
    // this payer's own interest and never below zero (a shortfall never
    // borrows from another payer's statement). Schedule B, the Form 8814 line
    // 1a and the penalty-interest base all read this one netting.
    pub(crate) fn net_taxable(&self) -> i64 {
        if let Some(records) = self.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref())
        {
            return records.taxable();
        }
        crate::kernel::ratio::positive(
            self.taxable + self.us_obligations
                - self.bond_premium
                - self.treasury_bond_premium
                - self.accrued_interest,
        ) + self.market_discount
    }
    // This payer's tax-exempt interest after the box 13 premium, not below zero.
    pub(crate) fn net_exempt(&self) -> i64 {
        if let Some(records) = self.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref())
        {
            return records.exempt();
        }
        crate::kernel::ratio::positive(self.exempt - self.tax_exempt_bond_premium)
    }
}

// Form 1099-INT (Rev. January 2024), Copy B: https://www.irs.gov/pub/irs-pdf/f1099int.pdf
/// Form 1099-INT Interest Income
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-INT exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Return the payer's name as printed, without truncation, or an empty string when it is absent. Read the CORRECTED checkbox into corrected, the payer's telephone number into payer_telephone and the payer's RTN into payer_rtn, returning false or empty text when absent. Read box 1 interest_income; a clearly blank box 1 means the payer reports no taxable interest, so return zero. In the other boxes read box 2 early_withdrawal_penalty, box 3 us_savings_bond_and_treasury_interest, box 4 federal_income_tax_withheld, box 5 investment_expenses, box 6 foreign_tax_paid, box 7 foreign_country_or_us_territory, box 8 tax_exempt_interest, box 9 specified_private_activity_bond_interest, box 10 market_discount, box 11 bond_premium, box 12 bond_premium_on_treasury_obligations, and box 13 bond_premium_on_tax_exempt_bond, returning zero for each clearly blank amount and empty text for a blank country. Read the FATCA filing requirement checkbox into fatca_filing_requirement, the account number into account_number, and the box 14 tax-exempt and tax credit bond CUSIP no. into tax_exempt_and_tax_credit_bond_cusip_no. Do not report accrued interest paid to a seller or whether a special treatment applies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer federal ID and address and state boxes 15 through 17. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. From an included supplemental statement, read the identified issuing-state allocations into exempt_sources, selecting Amount for printed dollar amounts or Percentage for printed percentages. Return Percentage.value as a decimal number from 0 to 100 without a percent sign; for example, 27.43% is 27.43, not 0.2743. Read the fund's U.S. government obligation percentage on the same scale, returning a decimal number without a percent sign for a reported value, or null when absent; an explicit zero is zero. Return an empty allocation list when no allocation is supplied. Do not convert a percentage to dollars or assume the withholding state is the bond-issuing state. Do not infer a personal election, nominee ownership, prior inclusion or gross/net presentation. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.interest_income,@.early_withdrawal_penalty,@.us_savings_bond_and_treasury_interest,@.federal_income_tax_withheld,@.investment_expenses,@.foreign_tax_paid,@.tax_exempt_interest,@.specified_private_activity_bond_interest,@.market_discount,@.bond_premium,@.bond_premium_on_treasury_obligations,@.bond_premium_on_tax_exempt_bond,@.account_number"
)]
pub struct Form1099IntPaper {
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
    /// Payer's RTN (optional)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_rtn: String,
    /// RECIPIENT'S name, street address (including apt. no.), city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 Interest income
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub interest_income: i64,
    /// Box 2 Early withdrawal penalty
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_withdrawal_penalty: i64,
    /// Box 3 Interest on U.S. Savings Bonds and Treasury obligations
    /// Savings bond interest you exclude for college costs is entered once more as Savings bonds cashed for education (Form 8815).
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_savings_bond_and_treasury_interest: i64,
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub federal_income_tax_withheld: i64,
    /// Box 5 Investment expenses
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Box 6 Foreign tax paid
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax_paid: i64,
    /// Box 7 Foreign country or U.S. territory
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub foreign_country_or_us_territory: String,
    /// Box 8 Tax-exempt interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_interest: i64,
    /// Box 9 Specified private activity bond interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub specified_private_activity_bond_interest: i64,
    /// Box 10 Market discount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Box 11 Bond premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Box 12 Bond premium on Treasury obligations
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium_on_treasury_obligations: i64,
    /// Box 13 Bond premium on tax-exempt bond
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium_on_tax_exempt_bond: i64,
    /// FATCA filing requirement
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 14 Tax-exempt and tax credit bond CUSIP no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub tax_exempt_and_tax_credit_bond_cusip_no: String,
    /// Boxes 15–17 State, State identification no., State tax withheld, and supplemental information
    #[libertas_llm_doc_field]
    pub state_details: crate::InvestmentStateFields,
}

/// Printed Form 1099-INT facts that no figure reads
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestPrintedFacts {
    /// CORRECTED (if checked)
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S telephone no.
    #[libertas_default("")]
    pub payer_telephone: String,
    /// Payer's RTN (optional)
    #[libertas_default("")]
    pub payer_rtn: String,
    /// FATCA filing requirement
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Account number (see instructions)
    #[libertas_default("")]
    pub account_number: String,
    /// Box 14 Tax-exempt and tax credit bond CUSIP no.
    #[libertas_default("")]
    pub tax_exempt_and_tax_credit_bond_cusip_no: String,
}

/// Accepted Form 1099-INT
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1099IntSource {
    /// Form 1099-INT as printed
    pub form: Form1099IntPaper,
    /// Applicable personal follow-up choices
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: IntCalls,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    /// [NullText]
    /// All reported interest components belong to this return
    /// [SetText]
    /// These interest components belong to someone outside this return
    #[libertas_hidden]
    pub nominee: Option<IntNomineeAnswer>,
    /// Personal annual reporting adjustments
    /// [NullText]
    /// Use the reported interest without personal annual adjustments
    /// [SetText]
    /// My personal annual interest adjustments are
    #[libertas_default($null)]
    #[libertas_hidden]
    pub adjustments: Option<crate::InterestAdjustments>,
    /// Paid after the death
    /// Only for a person who died in 2026 whose statement also includes interest for the time after the death: that part belongs to the estate or another recipient and is left off this final return (Pub. 559). Choose “No reported interest belongs to the period after death” otherwise. Your account statements show the dates.
    /// [NullText]
    /// No reported interest belongs to the period after death
    /// [SetText]
    /// The reported interest paid after death is
    #[libertas_hidden]
    pub after_death: Option<IntAfterDeathAnswer>,
    /// Accrued interest you paid the seller when you bought a bond
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub accrued_interest: i64,
    /// A treatment this page does not calculate
    /// Treaty interest or another unentered interest treatment. Nominee components and previously reported savings-bond interest have their own personal records below. Any choice but None of these leaves this statement out of the estimate with a notice.
    #[libertas_default(NoneOfThese)]
    #[libertas_hidden]
    pub special: InterestSpecial,
}

impl Form1099IntSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099IntPaper) -> Self {
        Self {
            form,
            calls: IntCalls::default(),
            nominee: None,
            adjustments: None,
            after_death: None,
            accrued_interest: 0,
            special: InterestSpecial::NoneOfThese,
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.nominee = accepted.nominee.clone();
        self.adjustments = accepted.adjustments.clone();
        self.after_death = accepted.after_death.clone();
        self.accrued_interest = accepted.accrued_interest;
        self.special = accepted.special;
    }
}

// One-way: every printed box and answer either reaches the internal model or
// is named as not read by any calculation, so a new one cannot be forgotten.
impl From<&Form1099IntSource> for FederalInterest {
    fn from(source: &Form1099IntSource) -> Self {
        let Form1099IntSource {
            form,
            // Presentation only: which follow-ups the page offers.
            calls: _,
            nominee,
            adjustments,
            after_death,
            accrued_interest,
            special,
        } = source;
        let Form1099IntPaper {
            calendar_year: _,
            corrected,
            // The parties name and assign the document; no figure reads them.
            payer: _,
            payer_telephone,
            payer_rtn,
            recipient: _,
            interest_income,
            early_withdrawal_penalty,
            us_savings_bond_and_treasury_interest,
            federal_income_tax_withheld,
            investment_expenses,
            foreign_tax_paid,
            foreign_country_or_us_territory,
            tax_exempt_interest,
            specified_private_activity_bond_interest,
            market_discount,
            bond_premium,
            bond_premium_on_treasury_obligations,
            bond_premium_on_tax_exempt_bond,
            fatca_filing_requirement,
            account_number,
            tax_exempt_and_tax_credit_bond_cusip_no,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = form;
        Self {
            private_loan: false,
            seller_mortgage: None,
            nominee: nominee.as_ref().map(|answer| answer.record().clone()),
            adjustments: adjustments.clone(),
            investment_expenses: *investment_expenses,
            market_discount: *market_discount,
            foreign_country: foreign_country_or_us_territory.clone(),
            taxable: *interest_income,
            us_obligations: *us_savings_bond_and_treasury_interest,
            early_penalty: *early_withdrawal_penalty,
            exempt: *tax_exempt_interest,
            private_activity: *specified_private_activity_bond_interest,
            withholding: *federal_income_tax_withheld,
            special: *special,
            bond_premium: *bond_premium,
            treasury_bond_premium: *bond_premium_on_treasury_obligations,
            tax_exempt_bond_premium: *bond_premium_on_tax_exempt_bond,
            accrued_interest: *accrued_interest,
            foreign_tax: *foreign_tax_paid,
            after_death: after_death.as_ref().map(|answer| answer.record().clone()),
            // Printed facts no figure reads; kept so a child's statement reopens whole.
            printed_facts: InterestPrintedFacts {
                corrected: *corrected,
                payer_telephone: payer_telephone.clone(),
                payer_rtn: payer_rtn.clone(),
                fatca_filing_requirement: *fatca_filing_requirement,
                account_number: account_number.clone(),
                tax_exempt_and_tax_credit_bond_cusip_no: tax_exempt_and_tax_credit_bond_cusip_no
                    .clone(),
            },
        }
    }
}

/// A treatment of interest this page does not calculate
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
pub enum InterestSpecial {
    /// None of these
    #[default]
    NoneOfThese,
    /// Interest taxed under a tax treaty
    Treaty,
}
impl InterestSpecial {
    // Answers and unknowns: the notice of the treatment that leaves the
    // statement out of the estimate; None for no treatment.
    pub(crate) fn notice(self) -> Option<&'static str> {
        match self {
            Self::NoneOfThese => None,
            Self::Treaty => Some("WARN_INTEREST_TREATY_LEFT_OUT"),
        }
    }
}

/// Applicable personal answers for Form 1099-INT
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IntCalls {
    /// Allowed choices for this answer
    pub nominee: Vec<i32>,
    /// Allowed choices for this answer
    pub after_death: Vec<i32>,
}

/// Reported personal allocation for this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntNomineeAnswer {
    /// Enter the allocation
    Reported {
        /// Allocation
        value: crate::InterestNominee,
    },
}
impl From<crate::InterestNominee> for IntNomineeAnswer {
    fn from(value: crate::InterestNominee) -> Self {
        Self::Reported { value }
    }
}
impl IntNomineeAnswer {
    pub(crate) fn record(&self) -> &crate::InterestNominee {
        let Self::Reported { value } = self;
        value
    }
}

/// Reported personal allocation for this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IntAfterDeathAnswer {
    /// Enter the allocation
    Reported {
        /// Allocation
        value: crate::InterestAfterDeath,
    },
}
impl From<crate::InterestAfterDeath> for IntAfterDeathAnswer {
    fn from(value: crate::InterestAfterDeath) -> Self {
        Self::Reported { value }
    }
}
impl IntAfterDeathAnswer {
    pub(crate) fn record(&self) -> &crate::InterestAfterDeath {
        let Self::Reported { value } = self;
        value
    }
}

impl core::ops::Deref for IntNomineeAnswer {
    type Target = crate::InterestNominee;
    fn deref(&self) -> &Self::Target {
        self.record()
    }
}
impl core::ops::DerefMut for IntNomineeAnswer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Reported { value } = self;
        value
    }
}

impl core::ops::Deref for IntAfterDeathAnswer {
    type Target = crate::InterestAfterDeath;
    fn deref(&self) -> &Self::Target {
        self.record()
    }
}
impl core::ops::DerefMut for IntAfterDeathAnswer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Reported { value } = self;
        value
    }
}
