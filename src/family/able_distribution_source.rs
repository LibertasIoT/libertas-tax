// Form 1099-QA Copy B (Rev. April 2025): https://www.irs.gov/pub/irs-pdf/f1099qa.pdf
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-QA
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-QA (Distributions From ABLE Accounts) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs; return an amount shown in parentheses as a negative number. Never infer a value that is not printed. Read box 1 gross distribution, box 2 earnings and box 3 basis; a blank box 1, 2 or 3 is an error. Read the box 4 program-to-program transfer, box 5 ABLE account terminated and box 6 not-the-designated-beneficiary checkboxes into program_to_program_transfer, account_terminated and recipient_not_designated_beneficiary exactly as checked. Do not choose treatment. Do not read the account number, and do not report what the money paid for, a rollover, a death or the contribution year; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Qa1099 {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 gross distribution
    /// Enter only a form addressed to you or your spouse. A form in a dependent's name goes on the dependent's own return.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 2 earnings
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Box 3 basis
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Box 4 program-to-program transfer
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub program_to_program_transfer: bool,
    /// Box 5 ABLE account terminated in the calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub account_terminated: bool,
    /// Box 6 you are not the designated beneficiary of this ABLE account
    /// You and/or the designated beneficiary may be subject to additional taxes and/or penalties on the box 1 gross distribution. See Form 5329 and its separate instructions and the Instructions for Form 1040.
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub recipient_not_designated_beneficiary: bool,
}

/// Accepted Form 1099-QA
/// The printed form and the topic answer it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleSource {
    /// Form 1099-QA as printed
    pub form: crate::Form1099QaPaper,
    /// What the payment was
    /// [NullText]
    /// The payment treatment is not answered yet
    /// [SetText]
    /// The payment treatment is
    #[libertas_default($null)]
    pub treatment: Option<AbleTreatment>,
}
impl AbleSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: crate::Form1099QaPaper) -> Self {
        Self { form, treatment: None }
    }
    /// The topic answer of an accepted source survives a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.treatment = accepted.treatment.clone();
    }
    pub(crate) fn topic_issue(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        self.form
            .boxes_issue()
            .or_else(|| AbleDistribution::from(self).answer_issue())
            .filter(|issue| require_answers || issue.key != "QA1099_TREATMENT_NEEDED")
    }
}
// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&AbleSource> for AbleDistribution {
    fn from(source: &AbleSource) -> Self {
        let AbleSource { form, treatment } = source;
        let natural = Qa1099::from(form);
        let Qa1099 {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            gross,
            earnings,
            // Gross less earnings, checked at the paper; no figure reads it.
            basis: _,
            program_to_program_transfer,
            account_terminated,
            recipient_not_designated_beneficiary,
        } = &natural;
        Self {
            gross: *gross,
            earnings: *earnings,
            program_to_program_transfer: *program_to_program_transfer,
            // Box 5 changes no figure.
            account_terminated: *account_terminated,
            recipient_not_designated_beneficiary: *recipient_not_designated_beneficiary,
            treatment: treatment.clone(),
        }
    }
}

/// What the payment was
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum AbleTreatment {
    /// Paid to you as the account's beneficiary
    /// A change to a beneficiary who is not your eligible brother or sister is already in box 1. A payment to you while box 6 is checked is not figured.
    Annual {
        /// Put back into an ABLE account within 60 days
        #[libertas_default(None)]
        rollover: AbleRollover,
        /// Qualified disability expenses paid in 2026
        /// Costs related to your disability that maintain or improve your health, independence or quality of life: education, housing, transportation, employment training and support, assistive technology, health, legal and financial services, funeral costs and the like. With two ABLE accounts, count each expense for one account only.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        expenses: i64,
        /// Expenses paid in the first 60 days of 2027 you treat as 2026
        /// Optional. They cannot also count for 2027.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        following_expenses: i64,
    },
    /// Paid after the beneficiary's death
    /// To the estate, an heir, a state Medicaid claim or outstanding disability expenses. Not taxed.
    Death,
    /// Returned excess contribution
    /// Paid back to you as the person who contributed, by the beneficiary's return due date.
    Corrective {
        /// Year of the contribution returned
        /// Earnings of a contribution for 2025 belong on your 2025 return.
        // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
        #[libertas_number(min = 2015, max = 2026)]
        year: u32,
    },
}

/// Put back into an ABLE account within 60 days
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AbleRollover {
    /// None
    None,
    /// Part or all of it
    /// Into your own ABLE account or that of an eligible brother or sister; one rollover into your account in 12 months.
    Qualified {
        /// Amount put back
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Earnings in that amount
        /// Use the program's earnings share for the amount rolled over.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        earnings: i64,
    },
}

/// ABLE account distribution — Form 1099-QA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleDistribution {
    /// Box 1 gross distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 2 earnings
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Box 4 program-to-program transfer
    pub program_to_program_transfer: bool,
    /// Box 5 ABLE account terminated in the calendar year
    pub account_terminated: bool,
    /// Box 6 you are not the designated beneficiary of this ABLE account
    pub recipient_not_designated_beneficiary: bool,
    /// What the payment was
    pub treatment: Option<AbleTreatment>,
}

// Final annual/correction/death income and additional-tax base, in cents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AbleAmounts {
    pub(crate) taxable: i64,
    pub(crate) penalized: i64,
}

impl AbleRollover {
    fn intrinsically_valid(self) -> bool {
        match self {
            Self::None => true,
            Self::Qualified { amount, earnings } => {
                (0..=100000000000).contains(&amount) && (0..=amount).contains(&earnings)
            }
        }
    }
    fn valid(self, gross: i64, total_earnings: i64) -> bool {
        match self {
            Self::None => true,
            Self::Qualified { amount, earnings } => {
                (0..=gross).contains(&amount)
                    && (0..=amount.min(total_earnings.max(0))).contains(&earnings)
                    && total_earnings - earnings <= gross - amount
            }
        }
    }
    fn excluded(self) -> (i64, i64) {
        match self {
            Self::Qualified { amount, earnings } => (amount, earnings),
            Self::None => (0, 0),
        }
    }
}
impl Qa1099 {
    /// The boxes the paper itself must reconcile: earnings and basis make up the gross.
    pub(crate) fn boxes_issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.earnings.checked_add(self.basis) != Some(self.gross))
            .then(|| AbleDistribution::rule_issue("QA1099_BOXES_DO_NOT_ADD_UP"))
    }
}
impl AbleDistribution {
    // Field metadata for the existing native predicates, not a second check.
    pub(crate) fn rule_issue(key: &'static str) -> crate::validation::ValidationIssue {
        let issue = crate::validation::ValidationIssue::new(key);
        match key {
            "QA1099_BOXES_DO_NOT_ADD_UP" => issue.at_field("gross"),
            "QA1099_ROLLOVER_ABOVE_GROSS" => {
                issue.at_field("amount").in_field("rollover").in_field("treatment")
            }
            "QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT" => {
                issue.at_field("earnings").in_field("rollover").in_field("treatment")
            }
            _ => issue,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.gross)
            && (-100000000000..=self.gross).contains(&self.earnings)
            && match self.treatment {
                Some(AbleTreatment::Annual { expenses, following_expenses, rollover }) => {
                    [expenses, following_expenses].iter().all(|v| (0..=100000000000).contains(v))
                        && rollover.intrinsically_valid()
                }
                Some(AbleTreatment::Corrective { year }) => (2015..=2026).contains(&year),
                Some(AbleTreatment::Death) | None => true,
            }
    }
    pub(crate) fn answer_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = Self::rule_issue;
        if self.transfer_only() {
            return None;
        }
        let Some(treatment) = &self.treatment else {
            return Some(issue("QA1099_TREATMENT_NEEDED").at_field("treatment"));
        };
        if let AbleTreatment::Annual {
            rollover: rollover @ AbleRollover::Qualified { amount, earnings },
            ..
        } = treatment
        {
            if *amount > self.gross {
                return Some(issue("QA1099_ROLLOVER_ABOVE_GROSS"));
            }
            if earnings > amount {
                return Some(issue("QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT"));
            }
            if !rollover.valid(self.gross, self.earnings) {
                return Some(
                    issue("QA1099_ROLLOVER_EARNINGS_NOT_RECONCILED")
                        .at_field("rollover")
                        .in_field("treatment"),
                );
            }
        }
        None
    }
    // Box 1 never holds a program-to-program transfer (Instructions for Forms 1099-QA
    // and 5498-QA, box 1), so box 4 changes no figure; a form that shows only the
    // transfer distributed nothing and asks for no treatment.
    pub(crate) fn transfer_only(&self) -> bool {
        self.program_to_program_transfer && self.gross == 0 && self.earnings == 0
    }
    // Box 6 says the payment went to someone other than the designated beneficiary,
    // so an annual withdrawal by the beneficiary contradicts the form. The additional
    // taxes the form warns of (Form 5329) are not figured for that recipient.
    pub(crate) const NOT_BENEFICIARY_KEY: &'static str = "ISSUE_ABLE_PAYMENT_TO_NON_BENEFICIARY";
    pub(crate) fn annual_to_non_beneficiary(&self) -> bool {
        self.recipient_not_designated_beneficiary
            && matches!(self.treatment, Some(AbleTreatment::Annual { .. }))
    }
    // The rollover (amount, earnings) an annual payment put back.
    pub(crate) fn rollover(&self) -> (i64, i64) {
        match self.treatment {
            Some(AbleTreatment::Annual { rollover, .. }) => rollover.excluded(),
            _ => (0, 0),
        }
    }
    // Cents. Round the excluded earnings before subtraction, as in Publication
    // 907 Table 1; annual tax lines are rounded by the caller. A loss is not
    // deductible, so negative earnings tax nothing.
    pub(crate) fn amounts(&self) -> Result<AbleAmounts, (&'static str, &'static str)> {
        if let Some(issue) = self.answer_issue() {
            return Err((
                issue.key,
                crate::fixture_english!(
                    "Complete the ABLE distribution's treatment and reconcile its retained records on Other income."
                ),
            ));
        }
        if self.transfer_only() {
            return Ok(AbleAmounts::default());
        }
        if self.annual_to_non_beneficiary() {
            return Err((
                Self::NOT_BENEFICIARY_KEY,
                crate::fixture_english!(
                    "Box 6 says this ABLE payment went to someone other than the designated beneficiary; the estimate does not figure that payment's income or the additional taxes on it."
                ),
            ));
        }
        let taxable = match self.treatment.as_ref().unwrap() {
            AbleTreatment::Annual { expenses, following_expenses, .. } => {
                let (rolled, rolled_earnings) = self.rollover();
                let gross = self.gross - rolled;
                let earnings = self.earnings - rolled_earnings;
                if earnings <= 0 {
                    0
                } else {
                    let qualified = expenses + following_expenses;
                    earnings - fraction(earnings, phase::cap(qualified, [gross]), gross.max(1))
                }
            }
            AbleTreatment::Corrective { year } => {
                if *year == 2026 {
                    positive(self.earnings)
                } else {
                    0
                }
            }
            AbleTreatment::Death => 0,
        };
        Ok(AbleAmounts { taxable, penalized: if self.penalized() { taxable } else { 0 } })
    }
    pub(crate) fn taxable(&self) -> Option<i64> {
        self.amounts().ok().map(|amounts| amounts.taxable)
    }
    // The 10% additional tax applies to the taxable annual withdrawals, never to
    // post-death payments or corrections.
    pub(crate) fn penalized(&self) -> bool {
        matches!(self.treatment, Some(AbleTreatment::Annual { .. }))
    }
}

/// 1099-QA
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-QA (Distributions From ABLE Accounts), Copy B, exactly as printed. Read the printed calendar year as a number, or null when no year is printed; never substitute zero or the form revision year. Read the corrected checkbox. Check a printed calendar year against the supplied year and report a mismatch as an error. Read each printed party block into payer and recipient, preserving names, identification numbers, addresses, masking, punctuation and leading zeros. Read the payer telephone number and account number as text. Use empty text for absent identity or account text. Never infer a recipient from household names. Read boxes 1 gross distribution, 2 earnings and 3 basis as decimal amounts without commas or currency signs; clearly blank money boxes mean zero. Preserve a printed negative earnings amount. Read boxes 4 program-to-program transfer, 5 ABLE account terminated in the calendar year reported and 6 recipient is not the designated beneficiary exactly as checked. Never infer an unprinted value, choose payment treatment, or report expenses, rollovers, a death or a contribution year. Use a corrected form in place of the original. Report illegible or conflicting values as an error. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.gross,@.earnings,@.basis,@.account_number"
)]
pub struct Form1099QaPaper {
    /// For calendar year
    /// [NullText]
    /// Calendar year is not printed
    /// [SetText]
    /// Calendar year is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_format("0")]
    pub calendar_year: Option<u32>,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, TIN, street address, city or town, state or province, country, ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN, street address (including apt. no.), city or town, state or province, country, and ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Gross distribution
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub gross: i64,
    /// Box 2 Earnings
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub earnings: i64,
    /// Box 3 Basis
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis: i64,
    /// Box 4 Program-to-program transfer
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub program_to_program_transfer: bool,
    /// Box 5 Check if ABLE account terminated in the calendar year reported
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub account_terminated: bool,
    /// Box 6 If this box is checked, the recipient is not the designated beneficiary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub recipient_not_designated_beneficiary: bool,
}

impl From<&Form1099QaPaper> for crate::Qa1099 {
    fn from(paper: &Form1099QaPaper) -> Self {
        let Form1099QaPaper {
            // These printed identifiers do not feed an existing federal figure.
            calendar_year: _,
            corrected: _,
            payer,
            payer_telephone: _,
            recipient,
            account_number: _,
            gross,
            earnings,
            basis,
            program_to_program_transfer,
            account_terminated,
            recipient_not_designated_beneficiary,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            gross: *gross,
            earnings: *earnings,
            basis: *basis,
            program_to_program_transfer: *program_to_program_transfer,
            account_terminated: *account_terminated,
            recipient_not_designated_beneficiary: *recipient_not_designated_beneficiary,
        }
    }
}

impl Form1099QaPaper {
    pub(crate) fn boxes_issue(&self) -> Option<crate::validation::ValidationIssue> {
        crate::Qa1099::from(self).boxes_issue()
    }
}
