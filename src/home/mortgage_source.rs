use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 1098 (Rev. April 2025), Copy B: https://www.irs.gov/pub/irs-pdf/f1098.pdf
/// Form 1098 Mortgage Interest Statement
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1098 Mortgage Interest Statement using its payer/borrower Copy B fields. Read the printed calendar year, not the form revision year, and compare it with the supplied document_facts year; a missing, illegible or conflicting calendar year or a year mismatch is an error. Copy the CORRECTED checkbox exactly; unchecked or blank means false. Copy the recipient/lender and payer/borrower names, TINs and addresses exactly as printed into lender and borrower, preserving masking, punctuation and leading zeros. Copy the lender's telephone number and account number exactly as printed; absent text is empty text. Never infer a borrower from household names or assign ownership. Read boxes 1 through 11 in their printed order: box 1 mortgage interest received, box 2 outstanding mortgage principal, box 3 mortgage origination date, box 4 refund of overpaid interest, box 5 mortgage insurance premiums, box 6 points paid on purchase of principal residence, box 7 same-address checkbox, box 8 address or description of property securing the mortgage, box 9 number of properties securing the mortgage, box 10 Other, and box 11 mortgage acquisition date. Copy monetary boxes 1, 2, 4, 5 and 6 exactly; a clearly blank monetary box means zero, but an illegible or conflicting amount is an error. Box 3 is a required valid calendar date: a blank, illegible, invalid or conflicting date is an error; never replace it with a refinance, draw or acquisition date. Box 7 is true only when checked; blank or unchecked means false. Copy box 8 as printed text, including parcel numbers or other descriptions; leave it empty when blank, and never fill it from the borrower's address. Read box 9 as the printed integer count; blank means zero for an unprinted count, never infer one property. Copy all of box 10 as printed text, preserving labels and amounts; do not classify it as property tax or another fixed amount. Read box 11 as the valid printed calendar date or null when clearly blank; an illegible, invalid or conflicting date is an error. Do not adjust any printed value for the taxpayer's share, prepayment, eligibility, deductible amount, loan use, refund treatment or points treatment. Do not infer personal facts, loan links or calculated values. Repeated copies of the same statement are one form; use Copy B when present and report conflicting copies rather than combining or guessing values. Copy-A-only VOID is not the Copy B CORRECTED checkbox.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.lender.federal_id,@.borrower.federal_id,@.calendar_year,@.box_3,@.box_8,@.box_1,@.box_2,@.box_4,@.box_5,@.box_6,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1098 {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// RECIPIENT'S/LENDER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub lender: crate::printed_state::PrintedParty,
    /// RECIPIENT'S/LENDER'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub lender_telephone: String,
    /// PAYER'S/BORROWER'S name, street address (including apt. no.), city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub borrower: crate::printed_state::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Mortgage interest received from payer(s)/borrower(s)
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_1: i64,
    /// Box 2 Outstanding mortgage principal
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_2: i64,
    /// Box 3 Mortgage origination date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub box_3: u32,
    /// Box 4 Refund of overpaid interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_4: i64,
    /// Box 5 Mortgage insurance premiums
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_5: i64,
    /// Box 6 Points paid on purchase of principal residence
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_6: i64,
    /// Box 7 If address of property securing mortgage is the same as PAYER'S/BORROWER'S address, the box is checked, or the address or description is entered in box 8.
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub box_7: bool,
    /// Box 8 Address or description of property securing mortgage
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_8: String,
    /// Box 9 Number of properties securing the mortgage
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_default(0)]
    pub box_9: u32,
    /// Box 10 Other
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_10: String,
    /// Box 11 Mortgage acquisition date
    /// [NullText]
    /// No mortgage acquisition date is printed in box 11.
    /// [SetText]
    /// The mortgage acquisition date printed in box 11 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_11: Option<u32>,
}

/// Form 1098 and the loan facts its printed boxes do not establish
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Mortgage1098Source {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Printed Form 1098
    pub form: Form1098,
    /// Person named as the printed borrower
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: crate::document_parties::DocumentRecipient,
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
    /// Property label or address
    #[libertas_default("")]
    pub property: String,
    /// Lender
    #[libertas_default(Institution)]
    pub lender: MortgageLender,
    /// Loan facts or the earlier lender
    #[libertas_default(FirstLender)]
    pub details: MortgageDetails,
    /// Your share of the interest reported in box 1
    /// Include a spouse's payments on a joint return. Use the payment records when another borrower paid part of the interest.
    #[libertas_default(All)]
    pub paid_share: MortgagePaidShare,
    /// Date the mortgage insurance contract was issued
    /// Needed only when box 5 reports premiums. Use the insurance contract, not the mortgage origination or lender acquisition date.
    #[libertas_default(Unavailable)]
    pub insurance_contract: crate::kernel::source_fact::SourceDate,
    /// How the points reported in box 6 are deducted
    /// Needed only when box 6 reports points.
    #[libertas_default(Purchase)]
    pub points: crate::mortgage_point_treatment::MortgagePointTreatment,
    /// Date the acquisition debt was incurred
    /// A credit-line draw has its own borrowing date; the printed mortgage origination date stays unchanged.
    #[libertas_default(Origination)]
    pub debt_date: MortgageDebtDate,
    /// What the box 4 refund repays
    /// [NullText]
    /// The source of the box 4 refund has not been established.
    /// [SetText]
    /// The box 4 refund repays
    #[libertas_default($null)]
    pub refund_origin: Option<crate::MortgageRefundOrigin>,
    /// Answered on Home mortgage interest
    /// The loan's own facts (its details, the share you paid and the insurance contract date) are asked there until saved.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(false)]
    pub answered: bool,
}

/// Your payments of the interest reported in box 1
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum MortgagePaidShare {
    /// I paid all of the box 1 interest
    /// Include your spouse's payments when filing jointly.
    All,
    /// My payment records show my share
    Amount(crate::kernel::source_fact::SourceCents),
}

/// Date the debt was incurred
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum MortgageDebtDate {
    /// The mortgage originated on the date printed in box 3
    Origination,
    /// The acquisition debt came from a credit-line draw
    Draw {
        /// Date the credit-line money was borrowed
        /// Use the draw records. An unavailable or unreported date cannot be replaced by the mortgage origination date.
        date: crate::kernel::source_fact::SourceDate,
    },
}

/// Average balance
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum AverageBalance {
    /// All my mortgages together are within the limit
    /// $750,000, or $1,000,000 for loans before December 16, 2017; half that on a separate return. Not for a line of credit you drew on after January 1, 2026.
    WithinLimit,
    /// Average of the first and last balances
    /// Level payments, with no additional borrowing after the beginning balance.
    FirstAndLast {
        /// Balance on December 31, 2026
        /// Or on the day the loan was paid off.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        last_balance: i64,
    },
    /// Interest divided by the interest rate
    /// Secured all year and paid at least monthly. Not for a loan used partly for other things.
    InterestOverRate {
        /// Lowest rate in 2026
        #[libertas_number(min = 0.001, max = 30, step = 0.001)]
        #[libertas_physical_unit("percent")]
        rate: f64,
    },
    /// From the lender's statements
    /// Not for a loan used partly for other things.
    Statements {
        /// Average balance
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        average: i64,
    },
    /// Month-end balances from the lender's statements
    /// Use this for a loan used partly for other things when the first and last balances do not fit, such as a line of credit you drew on in 2026. Leave out months with no balance.
    Monthly {
        /// Month-end balances
        // Tax structure: one balance for each calendar month.
        #[libertas_size(min = 1, max = 12)]
        #[libertas_unordered]
        /// ----
        /// Month
        balances: Vec<crate::MortgageMonthBalance>,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for AverageBalance {}
/// Loan facts or the earlier lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageDetails {
    /// First lender's entry for this loan
    /// Enter the balance for the whole year, including time with later lenders.
    FirstLender(MortgageFirstLender),
    /// A later lender acquired this same loan in 2026
    EarlierLender {
        /// Earlier lender's loan
        /// Save the first lender's entry before choosing it here. Its home, use and year-wide balances apply to every lender's interest; they are entered only once.
        loan: i64,
    },
}
/// The loan's year-wide facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageFirstLender {
    /// Home securing the loan
    #[libertas_default(Main)]
    pub home: MortgageHome,
    /// How the loan was used
    #[libertas_default(Home)]
    pub purpose: MortgageUse,
    /// Average balance for 2026
    #[libertas_default(WithinLimit)]
    pub average: AverageBalance,
}
impl Default for MortgageFirstLender {
    fn default() -> Self {
        Self {
            home: MortgageHome::Main,
            purpose: MortgageUse::Home,
            average: AverageBalance::WithinLimit,
        }
    }
}
impl Mortgage1098Source {
    pub fn printed(form: Form1098) -> Self {
        let points = if form.box_6 == 0 {
            crate::mortgage_point_treatment::MortgagePointTreatment::None
        } else {
            crate::mortgage_point_treatment::MortgagePointTreatment::Purchase
        };
        Self {
            id: 0,
            form,
            assigned: crate::document_parties::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
            property: String::new(),
            lender: MortgageLender::Institution,
            details: MortgageDetails::FirstLender(MortgageFirstLender::default()),
            paid_share: MortgagePaidShare::All,
            insurance_contract: crate::kernel::source_fact::SourceDate::Unavailable,
            points,
            debt_date: MortgageDebtDate::Origination,
            refund_origin: None,
            answered: false,
        }
    }

    pub fn keep_answers(&mut self, accepted: &Self) {
        if self.id == 0 || self.id != accepted.id {
            return;
        }
        let kept = if self.form.borrower.name == accepted.form.borrower.name {
            crate::document_parties::Assignment {
                owner: accepted.assigned,
                owner_source: accepted.owner_source,
                joint_share: accepted.joint_share,
            }
        } else {
            crate::document_parties::Assignment::default()
        };
        kept.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        self.property.clone_from(&accepted.property);
        self.lender.clone_from(&accepted.lender);
        self.details.clone_from(&accepted.details);
        self.paid_share.clone_from(&accepted.paid_share);
        self.insurance_contract.clone_from(&accepted.insurance_contract);
        self.points.clone_from(&accepted.points);
        self.debt_date.clone_from(&accepted.debt_date);
        self.refund_origin.clone_from(&accepted.refund_origin);
        self.answered = accepted.answered;
    }
}

/// How the loan was used
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageUse {
    /// To buy, build or substantially improve this home
    /// Every dollar went to buying, building or substantially improving the home that secures it. A line of credit opened before December 16, 2017 and drawn later dates from the draw: enter that date in box 3.
    Home,
    /// To refinance an earlier loan on this home
    /// Paid off a loan that bought, built or improved this home, with or without more money borrowed.
    Refinance(MortgageRefinance),
    /// Partly for this home, partly for other things
    /// A home equity loan or line of credit, or a loan secured by more than one property (Form 1098 box 9), used partly to buy, build or substantially improve this home.
    Partly(MortgageSplit),
    /// For other things only
    /// A home equity loan or line of credit used for a car, tuition, debts or anything other than this home. Its interest is not home mortgage interest; interest on money used in your business, rental or investments is entered there.
    Other,
}

/// Refinance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageRefinance {
    /// Balance of the earlier loan that this loan paid off
    /// Only the part that was still borrowed to buy, build or improve this home; any part of the earlier loan that was not belongs in the amount for other things below.
    #[libertas_money("USD", 2)]
    // Arithmetic safety: original_amount() sums these i64 loan parts.
    #[libertas_number(min = 1, max = 100000000000)]
    pub refinanced: i64,
    /// Date of the earlier loan
    /// For a chain of refinances, the date of the original loan that bought, built or improved the home.
    #[libertas_date_only]
    // Tax-year source window: this date cannot be later than the applicable 2026 cutoff.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub earlier_incurred: u32,
    /// Date the original loan's term ended or ends
    /// Its scheduled final payment. When the original loan paid no principal until the end (a balloon note), the end of the first refinance's term, but not more than 30 years after that refinance. Needed when the original loan is from before October 14, 1987, or from before December 16, 2017 and this loan is later.
    /// [NullText]
    /// The original term is not needed for grandfathered-debt treatment
    /// [SetText]
    /// The original loan term ended or ends on
    #[libertas_date_only]
    pub earlier_term_ends: Option<u32>,
    /// More borrowed to buy, build or substantially improve this home
    #[libertas_money("USD", 2)]
    // Arithmetic safety: original_amount() sums these i64 loan parts.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub improvement: i64,
    /// More borrowed for other things
    /// Cash out for anything else, including closing costs added to the loan.
    #[libertas_money("USD", 2)]
    // Arithmetic safety: original_amount() sums these i64 loan parts.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
}

/// Partly for this home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSplit {
    /// Amount used to buy, build or substantially improve this home
    /// For a line of credit, enter the total drawn for this use. The calculation treats the totals for each use as borrowed at the start. If you repaid principal between draws for different uses, this totals method can differ from tracing each draw.
    #[libertas_money("USD", 2)]
    // Arithmetic safety: original_amount() sums these i64 loan parts.
    #[libertas_number(min = 1, max = 100000000000)]
    pub improvement: i64,
    /// Date you borrowed that amount
    /// The loan's date, or for a line of credit the date of the draw (the latest one when you drew several times).
    #[libertas_date_only]
    // Tax-year source window: this date cannot be later than the applicable 2026 cutoff.
    #[libertas_number(min = $MORTGAGE_ACQUISITION_FIRST_DATE, max = $TAX_YEAR_END_DATE)]
    pub improvement_incurred: u32,
    /// Amount used for other things
    #[libertas_money("USD", 2)]
    // Arithmetic safety: original_amount() sums these i64 loan parts.
    #[libertas_number(min = 1, max = 100000000000)]
    pub other: i64,
}

/// Home securing the loan
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MortgageHome {
    /// Main home
    Main,
    /// Second home
    /// The one other home you choose for 2026. If you also rent it out, select this loan on the rental: the App applies the residence test from the rental's days, and when your personal use was not more than 14 days or 10% of the days rented, whichever is longer, the personal share of the interest is not deductible.
    Second,
    /// Second home that is a boat or recreational vehicle
    /// With sleeping, cooking and toilet facilities. Its interest is deductible, but not for the alternative minimum tax.
    SecondVehicle,
    /// Another home
    /// A third home, or a second home you do not choose for 2026. Its interest is not personally deductible home mortgage interest. For a home you also rent, keep the loan here and select it on the rental; the rental uses its acquisition-qualified interest without deducting it twice.
    NotQualified,
}

/// Lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageLender {
    /// Bank or other lender that sent a Form 1098
    Institution,
    /// Individual who sold you the home
    PrivateSeller {
        /// Seller's name
        #[libertas_size(min = 1)]
        name: String,
        /// Seller's SSN or EIN
        // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
        #[libertas_size(min = 9, max = 11)]
        identifying_number: String,
        /// Seller's address
        #[libertas_size(min = 1)]
        address: String,
    },
    /// A lender that sent no Form 1098, or your share of someone else's Form 1098
    /// Schedule A line 8b. When you and someone other than your spouse owe the loan and the Form 1098 is in that person's name, enter only the interest you paid.
    NoForm {
        /// Lender, or the person whose name is on the Form 1098
        #[libertas_size(min = 1)]
        name: String,
    },
}

/// Month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageMonthBalance {
    /// Month of 2026
    /// 1 is January.
    // Tax structure: a calendar year has twelve months.
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Balance at the end of the month
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
}

/// What the box 4 refund repays
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageRefundOrigin {
    /// Interest you paid in 2026
    /// The lender's letter says so. It reduces this year's interest.
    ThisYear,
    /// Interest of an earlier year that you deducted
    /// Only the part that lowered that year's tax is income (Schedule 1 line 8z). Enter that year's figures; a refund and other recoveries of the same year are figured together.
    Deducted(crate::prior_recovery_source::PriorYearRecovery),
    /// Interest of an earlier year that gave no tax benefit
    /// You took the standard deduction that year or did not deduct this interest. None of the refund is taxable.
    NotDeducted,
}
