// Form 1099-Q source facts, education account expenses and intrinsic checks.
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-Q
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer_trustee, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-Q (Payments From Qualified Education Programs) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs; return a box 2 loss shown in parentheses as a negative number. Never infer a value that is not printed. Read box 1 gross distribution; a blank box 1 is an error. For box 5, return Qtp when box 5a (private QTP) or 5b (state QTP) is checked and Coverdell when box 5c is checked; report an error when none or more than one is checked. For Qtp, read box 2 earnings and box 3 basis; a blank box 2 or box 3 is an error. For Coverdell, return Reported with box 2 and box 3 when they are printed, and ValueOnly with the box 7 fair market value when boxes 2 and 3 are blank and box 7 shows an FMV; leave the opening basis, contributions and late rollovers to the filer. For box 4, return TrusteeTransfer when box 4a is checked, RothIra when box 4b is checked, and otherwise read the box 7 distribution code: ExcessThisYear for code 2, ExcessLastYear for code 3, ProhibitedTransaction for code 6, and Withdrawal for code 1, 4, 5 or no code, with reason Disability for code 4, Death for code 5 and Withdrawal otherwise. Read box 6 into recipient_not_beneficiary exactly as checked; do not choose the personal beneficiary. Do not read the payer or recipient address, TIN or account number, and do not choose the student, report what the money paid for, a rollover or redeposit, or any 529-to-Roth history; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Q1099 {
    /// PAYER'S/TRUSTEE'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer_trustee: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 gross distribution
    /// Enter only a form addressed to you or your spouse. A form in a dependent student's name goes on the student's own return.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub gross: i64,
    /// Account (box 5)
    #[libertas_llm_doc_field]
    pub account: EducationAccount,
    /// What happened to the money (box 4 and box 7 code)
    #[libertas_llm_doc_field]
    #[libertas_default(Withdrawal)]
    pub transfer: Q1099Transfer,
    /// Recipient is not the designated beneficiary (box 6)
    #[libertas_llm_doc_field]
    pub recipient_not_beneficiary: bool,
}

/// Accepted Form 1099-Q
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationDistributionSource {
    /// Form 1099-Q as printed
    pub form: crate::Form1099QPaper,
    /// Student identified from your records
    /// [NullText]
    /// The student has not been identified yet
    /// [SetText]
    /// The student is
    #[libertas_default($null)]
    pub beneficiary: Option<FederalEducationBeneficiary>,
    /// This Coverdell account's basis at December 31, 2025
    /// Contributions before 2026 minus the tax-free part of earlier withdrawals: line 15 of your 2025 Worksheet 6-3, or contributions if this is the first withdrawal.
    /// [NullText]
    /// Opening Coverdell basis is unavailable
    /// [SetText]
    /// Opening Coverdell basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub coverdell_basis_start: Option<i64>,
    /// Contributions to this Coverdell account for 2026
    /// Include contributions for 2026 made through April 15, 2027. Not rollovers or returned excess contributions. Zero when none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub coverdell_contributions: i64,
    /// Part put back within 60 days
    /// 529: rolled into another 529 or an ABLE account for this student or a family member (one same-student 529 rollover in 12 months; ABLE within its yearly limit), or a refund of college costs recontributed to a 529. Coverdell: rolled into another Coverdell for a family member under 30, one rollover in 12 months. Zero when none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeposited: i64,
    /// Part of the direct trustee transfer that does not qualify
    /// A second same-student 529 transfer within 12 months, a transfer for someone outside the student's family (or 30 or older, for a Coverdell), or the part of an ABLE transfer above that account's 2026 contribution limit. Zero when it all qualifies.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable_transfer_part: i64,
    /// 529-to-Roth personal transfer records
    /// [NullText]
    /// The transfer records are unavailable
    /// [SetText]
    /// The transfer records are
    #[libertas_default($null)]
    pub roth_rollover: Option<crate::QtpRothRollover>,
    /// Late-year withdrawals rolled into this Coverdell account next year
    /// Amount withdrawn in the last 60 days of 2026 and rolled into this account in 2027, added to printed year-end FMV for Worksheet 6-3. Enter actual zero when none.
    /// [NullText]
    /// The late-rollover adjustment is unavailable
    /// [SetText]
    /// The established late-rollover adjustment is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub late_rollovers: Option<i64>,
}
impl EducationDistributionSource {
    // Box 6 establishes the recipient as beneficiary unless it is checked.
    // https://www.irs.gov/instructions/i1099q
    pub(crate) fn printed(form: crate::Form1099QPaper) -> Self {
        let beneficiary =
            (!form.recipient_not_beneficiary).then_some(FederalEducationBeneficiary::Recipient);
        Self {
            form,
            beneficiary,
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: None,
            late_rollovers: None,
        }
    }
    /// Retain personal answers only while the paper leaves the same question open.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.beneficiary = if !self.form.recipient_not_beneficiary {
            Some(FederalEducationBeneficiary::Recipient)
        } else if accepted.form.recipient_not_beneficiary {
            accepted.beneficiary.clone()
        } else {
            None
        };
        self.coverdell_basis_start = accepted.coverdell_basis_start;
        self.coverdell_contributions = accepted.coverdell_contributions;
        self.redeposited = accepted.redeposited;
        self.taxable_transfer_part = accepted.taxable_transfer_part;
        self.roth_rollover = accepted.roth_rollover.clone();
        self.late_rollovers = accepted.late_rollovers;
    }
}
// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&EducationDistributionSource> for FederalEducationDistribution {
    fn from(source: &EducationDistributionSource) -> Self {
        let EducationDistributionSource {
            form,
            beneficiary,
            coverdell_basis_start,
            coverdell_contributions,
            redeposited,
            taxable_transfer_part,
            roth_rollover,
            late_rollovers,
        } = source;
        let Q1099 {
            // The parties name and assign the document; no figure reads them.
            payer_trustee: _,
            recipient: _,
            gross,
            account,
            transfer,
            recipient_not_beneficiary,
        } = &Q1099::from(form);
        Self {
            gross: *gross,
            account: account.clone(),
            transfer: transfer.clone(),
            recipient_not_beneficiary: *recipient_not_beneficiary,
            coverdell_basis_start: *coverdell_basis_start,
            coverdell_contributions: *coverdell_contributions,
            redeposited: *redeposited,
            taxable_transfer_part: *taxable_transfer_part,
            roth_rollover: roth_rollover.clone(),
            late_rollovers: *late_rollovers,
            beneficiary: beneficiary.clone(),
        }
    }
}

/// Account (box 5)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EducationAccount {
    /// 529 plan (box 5a or 5b)
    Qtp {
        /// Box 2 earnings
        /// A loss in parentheses is shown only for the account's final year; it is not deductible.
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        earnings: i64,
        /// Box 3 basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
    },
    /// Coverdell ESA (box 5c)
    Coverdell {
        /// Earnings and basis
        #[libertas_llm_doc_field]
        #[libertas_inline_choices]
        amounts: CoverdellAmounts,
    },
}

/// Earnings and basis
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoverdellAmounts {
    /// Boxes 2 and 3 are filled in
    Reported {
        /// Box 2 earnings
        /// A loss in parentheses is shown only for the account's final year; it is not deductible.
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        earnings: i64,
        /// Box 3 basis
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
    },
    /// Only a year-end value in box 7
    /// The App figures the earnings from the account's basis, as Publication 970 Worksheet 6-3 does.
    ValueOnly {
        /// Box 7 fair market value at December 31, 2026
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
    },
}

/// What happened to the money (box 4 and box 7 code)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Q1099Transfer {
    /// Paid to you or the school
    /// Neither box 4a nor box 4b is checked. A Coverdell balance must be paid out by 30 days after the student turns 30 (unless the student has special needs); a balance not paid out, or moved to a new student outside the family or 30 or older, is still a withdrawal: enter it from the trustee's statement.
    Withdrawal {
        /// Reason
        #[libertas_llm_doc_field]
        #[libertas_default(Withdrawal)]
        #[libertas_inline_choices]
        reason: DistributionReason,
    },
    /// Direct transfer to another 529, Coverdell or ABLE account (box 4a)
    TrusteeTransfer,
    /// Direct transfer from a 529 to the student's Roth IRA (box 4b)
    RothIra,
    /// Return of an excess Coverdell contribution, taxable this year (box 7 code 2)
    ExcessThisYear,
    /// Return of an excess Coverdell contribution, taxable last year (box 7 code 3)
    /// Its earnings belong on your 2025 return, not this one.
    ExcessLastYear,
    /// Prohibited transaction (box 7 code 6)
    /// The account stopped being a Coverdell ESA on January 1: box 1 is its value that day, taxed as a withdrawal with no education expenses applied, plus the 10% additional tax.
    ProhibitedTransaction,
}

/// Reason
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DistributionReason {
    /// Education or other withdrawal
    Withdrawal,
    /// The student died (box 7 code 5)
    /// Paid to a beneficiary or the estate on or after the student's death.
    Death,
    /// The student is disabled (box 7 code 4)
    /// Unable to do any substantial gainful activity because of a condition expected to last indefinitely or end in death, as a physician determines.
    Disability,
}

/// College tuition and required books
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CollegeCosts {
    /// From this student's education credit entry
    /// The App uses the tuition, books, refunds and scholarships entered on Education credits, after the expenses that credit used.
    EducationCredit,
    /// Entered here
    /// For a student with no education credit entry on this return.
    Entered {
        /// Tuition, fees and required books, supplies and equipment for 2026
        /// At a college, university, vocational school or other school eligible for federal student aid.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        tuition_books: i64,
        /// Tax-free scholarships, grants and other tax-free educational assistance
        /// Pell grants, veterans' and employer assistance count; gifts and loans do not.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        assistance: i64,
        /// Costs another person's education credit used for this student
        /// For example, the expenses on which a parent claimed the American opportunity credit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        credit_elsewhere: i64,
    },
}

/// Payment from a 529 plan or Coverdell ESA — Form 1099-Q
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEducationDistribution {
    /// Box 1 gross distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub gross: i64,
    /// Account (box 5)
    pub account: EducationAccount,
    /// What happened to the money (box 4 and box 7 code)
    pub transfer: Q1099Transfer,
    /// Recipient is not the designated beneficiary (box 6)
    pub recipient_not_beneficiary: bool,
    /// Student identified from your records
    /// [NullText]
    /// The student has not been identified yet
    /// [SetText]
    /// The student is
    pub beneficiary: Option<FederalEducationBeneficiary>,
    /// This Coverdell account's basis at December 31, 2025
    /// Contributions before 2026 minus the tax-free part of earlier withdrawals: line 15 of your 2025 Worksheet 6-3, or contributions if this is the first withdrawal.
    /// [NullText]
    /// Opening Coverdell basis is unavailable
    /// [SetText]
    /// Opening Coverdell basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub coverdell_basis_start: Option<i64>,
    /// Contributions to this Coverdell account for 2026
    /// Include contributions for 2026 made through April 15, 2027. Not rollovers or returned excess contributions. Zero when none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub coverdell_contributions: i64,
    /// Part put back within 60 days
    /// 529: rolled into another 529 or an ABLE account for this student or a family member (one same-student 529 rollover in 12 months; ABLE within its yearly limit), or a refund of college costs recontributed to a 529. Coverdell: rolled into another Coverdell for a family member under 30, one rollover in 12 months. Zero when none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeposited: i64,
    /// Part of the direct trustee transfer that does not qualify
    /// A second same-student 529 transfer within 12 months, a transfer for someone outside the student's family (or 30 or older, for a Coverdell), or the part of an ABLE transfer above that account's 2026 contribution limit. Zero when it all qualifies.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable_transfer_part: i64,
    /// 529-to-Roth personal transfer records
    /// [NullText]
    /// The transfer records are unavailable
    /// [SetText]
    /// The transfer records are
    #[libertas_default($null)]
    pub roth_rollover: Option<crate::QtpRothRollover>,
    /// Late-year withdrawals rolled into this Coverdell account next year
    /// Amount withdrawn in the last 60 days of 2026 and rolled into this account in 2027, added to printed year-end FMV for Worksheet 6-3. Enter actual zero when none.
    /// [NullText]
    /// The late-rollover adjustment is unavailable
    /// [SetText]
    /// The established late-rollover adjustment is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub late_rollovers: Option<i64>,
}

/// Student (box 6)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalEducationBeneficiary {
    /// The person this form is addressed to is the student
    Recipient,
    /// Someone else on this return is the student
    Student {
        /// Person identity
        /// −1 you, −2 your spouse, otherwise the dependent's accepted record identity.
        #[libertas_hidden]
        person: i64,
    },
    /// The student is not on this return
    OtherStudent {
        /// The student's qualified education expenses assigned to this distribution
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
        /// Tax-free educational assistance toward those expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assistance: i64,
        /// Those expenses used for an education credit
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        credit_expenses: i64,
    },
}

/// 529 and Coverdell expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEducationAccountExpenses {
    /// Students
    #[libertas_size(min = 1)]
    /// ----
    /// Student
    pub students: Vec<FederalEducationAccountStudent>,
}

/// Student
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEducationAccountStudent {
    /// Person identity
    /// −1 you, −2 your spouse, otherwise the dependent's accepted record identity.
    #[libertas_hidden]
    pub person: i64,
    /// College tuition and required books
    pub college: CollegeCosts,
    /// Room, board, computer and other college costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other_college: i64,
    /// K–12 costs on the 529 list
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub k12: i64,
    /// Other K–12 costs only a Coverdell ESA can pay
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub k12_coverdell: i64,
    /// Registered apprenticeship costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub apprenticeship: i64,
    /// Payments on this student's own qualified student loans
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub loan_payments: i64,
    /// 529 money used for this student's loans before 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub loan_payments_before: i64,
    /// U.S. military academy costs of advanced education
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub academy_costs: i64,
    /// Costs already used elsewhere
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used_elsewhere: i64,
}

fn money(v: i64) -> bool {
    (0..=100_000_000_000).contains(&v)
}

impl EducationAccount {
    // Box 2 and box 3 of the form when it prints them.
    pub(crate) fn boxes(&self) -> Option<(i64, i64)> {
        match self {
            Self::Qtp { earnings, basis }
            | Self::Coverdell { amounts: CoverdellAmounts::Reported { earnings, basis, .. } } => {
                Some((*earnings, *basis))
            }
            Self::Coverdell { .. } => None,
        }
    }
    pub(crate) fn is_qtp(&self) -> bool {
        matches!(self, Self::Qtp { .. })
    }
    fn valid(&self, gross: i64) -> bool {
        match self {
            Self::Qtp { .. } | Self::Coverdell { amounts: CoverdellAmounts::Reported { .. } } => {
                self.boxes().is_some_and(|(earnings, basis)| {
                    (-100_000_000_000..=100_000_000_000).contains(&earnings)
                        && money(basis)
                        && earnings + basis == gross
                })
            }
            Self::Coverdell { amounts: CoverdellAmounts::ValueOnly { value } } => money(*value),
        }
    }
}

impl FederalEducationDistribution {
    // Original source facts and independently valid retained answers may be
    // saved before their reconciliation is established on the topic page.
    pub(crate) fn valid(&self) -> bool {
        let gross = self.gross;
        (1..=100_000_000_000).contains(&gross)
            && self.account.valid(gross)
            && self.coverdell_basis_start.is_none_or(money)
            && money(self.coverdell_contributions)
            && money(self.redeposited)
            && money(self.taxable_transfer_part)
            && self.late_rollovers.is_none_or(money)
            && self.roth_rollover.as_ref().is_none_or(crate::QtpRothRollover::well_formed)
            && match self.transfer {
                Q1099Transfer::RothIra => {
                    self.account.is_qtp()
                        && self.account.boxes().is_some_and(|(earnings, _)| earnings >= 0)
                }
                Q1099Transfer::ExcessThisYear | Q1099Transfer::ExcessLastYear => matches!(
                    self.account,
                    EducationAccount::Coverdell { amounts: CoverdellAmounts::Reported { .. } }
                ),
                Q1099Transfer::ProhibitedTransaction => !self.account.is_qtp(),
                _ => true,
            }
            && match &self.beneficiary {
                Some(FederalEducationBeneficiary::OtherStudent {
                    expenses,
                    assistance,
                    credit_expenses,
                }) => money(*expenses) && money(*assistance) && money(*credit_expenses),
                _ => true,
            }
    }

    pub(crate) fn effective_beneficiary(&self) -> Option<&FederalEducationBeneficiary> {
        if self.recipient_not_beneficiary {
            self.beneficiary.as_ref()
        } else {
            Some(&FederalEducationBeneficiary::Recipient)
        }
    }

    fn coverdell_adjusted_value(&self, value: i64) -> Option<i64> {
        value.checked_add(self.late_rollovers?).filter(|value| money(*value))
    }

    // These predicates are shared by strict topic Save, pending rows and the
    // calculation's availability guard, never by printed-source admission.
    pub(crate) fn answer_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.answer_issue_with_required(true)
    }
    pub(crate) fn answer_issue_with_required(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if require_answers && self.recipient_not_beneficiary && self.beneficiary.is_none() {
            return Some(ValidationIssue::new("Q1099_BENEFICIARY_NEEDED").at_field("beneficiary"));
        }
        if self.beneficiary.as_ref().is_some_and(|beneficiary| {
            self.recipient_not_beneficiary
                == matches!(beneficiary, FederalEducationBeneficiary::Recipient)
        }) {
            return Some(
                ValidationIssue::new("Q1099_BENEFICIARY_BOX6_CONFLICT").at_field("beneficiary"),
            );
        }
        if let EducationAccount::Coverdell { amounts: CoverdellAmounts::ValueOnly { value } } =
            self.account
        {
            if require_answers && self.coverdell_basis_start.is_none() {
                return Some(
                    ValidationIssue::new("Q1099_COVERDELL_BASIS_NEEDED")
                        .at_field("coverdell_basis_start"),
                );
            }
            if require_answers && self.late_rollovers.is_none() {
                return Some(
                    ValidationIssue::new("Q1099_LATE_ROLLOVERS_NEEDED").at_field("late_rollovers"),
                );
            }
            if self.late_rollovers.is_some() && self.coverdell_adjusted_value(value).is_none() {
                return Some(
                    ValidationIssue::new(
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    )
                    .at_field("late_rollovers"),
                );
            }
        }
        match self.transfer {
            Q1099Transfer::Withdrawal { .. } if self.redeposited > self.gross => {
                Some(ValidationIssue::new("Q1099_PART_ABOVE_GROSS").at_field("redeposited"))
            }
            Q1099Transfer::TrusteeTransfer if self.taxable_transfer_part > self.gross => Some(
                ValidationIssue::new("Q1099_PART_ABOVE_GROSS").at_field("taxable_transfer_part"),
            ),
            Q1099Transfer::RothIra if require_answers && self.roth_rollover.is_none() => {
                Some(ValidationIssue::new("Q1099_ROTH_RECORDS_NEEDED").at_field("roth_rollover"))
            }
            Q1099Transfer::RothIra
                if self.effective_beneficiary().is_some_and(|beneficiary| {
                    beneficiary != &FederalEducationBeneficiary::Recipient
                }) =>
            {
                Some(
                    ValidationIssue::new("Q1099_ROTH_ROLLOVER_NOT_STUDENT").at_field("beneficiary"),
                )
            }
            _ => None,
        }
    }
}

impl FederalEducationAccountExpenses {
    pub(crate) fn valid(&self) -> bool {
        (1..=10).contains(&self.students.len())
            && self.students.iter().enumerate().all(|(i, s)| {
                s.valid() && !self.students[..i].iter().any(|earlier| earlier.person == s.person)
            })
    }
}

impl FederalEducationAccountStudent {
    fn valid(&self) -> bool {
        (match &self.college {
            CollegeCosts::EducationCredit => true,
            CollegeCosts::Entered { tuition_books, assistance, credit_elsewhere } => {
                money(*tuition_books) && money(*assistance) && money(*credit_elsewhere)
            }
        }) && [
            self.other_college,
            self.k12,
            self.k12_coverdell,
            self.apprenticeship,
            self.loan_payments,
            self.academy_costs,
            self.used_elsewhere,
        ]
        .iter()
        .all(|v| money(*v))
            && (0..=1_000_000).contains(&self.loan_payments_before)
    }
}

impl FederalEducationDistribution {
    // The amount a document preview can state without the return's expenses
    // and credits: the earnings of a part no expense can cover, a timely
    // returned excess, and nothing for a transfer to a Roth IRA or a code 3
    // excess; None where the expense record decides.
    pub(crate) fn preview(&self) -> Option<i64> {
        if self.answer_issue().is_some() {
            return None;
        }
        match &self.transfer {
            Q1099Transfer::Withdrawal { .. } | Q1099Transfer::ProhibitedTransaction => None,
            Q1099Transfer::TrusteeTransfer => {
                Some(positive(earnings_in(self, self.gross, self.taxable_transfer_part)?.0))
            }
            Q1099Transfer::ExcessThisYear => {
                Some(positive(self.account.boxes().map_or(0, |(earnings, _)| earnings)))
            }
            Q1099Transfer::RothIra if !self.roth_rollover.as_ref()?.confirmed => None,
            Q1099Transfer::RothIra | Q1099Transfer::ExcessLastYear => Some(0),
        }
    }
}

// Earnings in a distribution and, for a Coverdell with only a year-end value,
// the basis left (Pub 970 Worksheet 6-3 lines 3 to 15).
pub(crate) fn earnings_in(
    data: &FederalEducationDistribution,
    gross: i64,
    amount: i64,
) -> Option<(i64, Option<i64>)> {
    Some(match &data.account {
        EducationAccount::Coverdell { amounts: CoverdellAmounts::ValueOnly { value } } => {
            let basis_start = data.coverdell_basis_start?;
            let contributions = data.coverdell_contributions;
            let value = data.coverdell_adjusted_value(*value)?;
            let basis = basis_start + contributions;
            let after = amount + value;
            let basis_part =
                if after > 0 { fraction(amount, phase::cap(basis, [after]), after) } else { 0 };
            (amount - basis_part, Some(basis - basis_part))
        }
        _ => {
            let earnings = data.account.boxes().map_or(0, |(earnings, _)| earnings);
            (if gross > 0 { fraction(earnings, amount, gross) } else { 0 }, None)
        }
    })
}

// IRC 529(c)(3)(E), 408A(c)(3)(E), (c)(5)(B), (e)(1)(C):
// https://uscode.house.gov/view.xhtml?req=(title:26%20section:529%20edition:prelim)
// https://uscode.house.gov/view.xhtml?req=(title:26%20section:408a%20edition:prelim)
/// Direct 2026 transfer from a long-held section 529 plan to its beneficiary's Roth IRA
/// Form 1099-Q box 4b is checked: the 529 trustee moved this money directly to the beneficiary's Roth IRA. The form's earnings (box 2) and basis (box 3) are the transfer's earnings and contribution portions. A qualified rollover, reported by the Roth custodian in 2026 Form 5498 box 10. It shares the annual IRA dollar and compensation room with regular contributions but is not a regular Roth contribution or a Saver's Credit contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QtpRothRollover {
    /// Source 529 account reference
    /// Use a short local reference or the Form 1099-Q account identifier; do not enter a full account number. Use the same reference for multiple Forms 1099-Q from the same 529 account, and different references for different accounts.
    #[libertas_size(min = 1)]
    pub account_reference: String,
    /// State sponsoring the source 529 plan
    /// Use the plan's records, not your home state, school location or the payer's address. Leave undefined when the plan does not identify a sponsoring state.
    /// [NullText]
    /// The plan records do not identify a sponsoring state
    /// [SetText]
    /// The source 529 plan's sponsoring state is
    #[libertas_default($null)]
    pub plan_state: Option<crate::TaxState>,
    /// Date the source 529 account was opened for this beneficiary
    /// The account must have been maintained for this same beneficiary throughout the 15-year period ending on the direct transfer date. A beneficiary change or uncertain account history needs separate review.
    #[libertas_date_only]
    pub account_opened_on: u32,
    /// Date the 529 trustee transferred the money directly to the Roth IRA trustee
    /// This entry covers a transfer completed in 2026 and designated for 2026. A transfer in 2027 designated for 2026 needs a separate timing worksheet.
    #[libertas_date_only]
    pub transferred_on: u32,
    /// Amount of the source account available from deposits made at least five years earlier
    /// Use this source account's administrator-reconciled balance of contributions made before the five-year period ending on this transfer date plus earnings attributable to them, immediately before this transfer and after all earlier distributions and transfers, including earlier listed transfers on the same date. It must cover this entire transfer. Do not infer it from the account opening date or another account's balance.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub seasoned_available: i64,
    /// Transfer and source records confirmed
    /// Check Form 1099-Q, the 529 plan statements and Form 5498: this account's history for the same beneficiary throughout the 15 years, the direct trustee-to-trustee transfer into that person's own Roth IRA, this account's five-year seasoned balance after earlier distributions, and that the earlier transfers on the IRA contributions page include every earlier qualifying transfer. No means the transfer is not a qualified rollover: the form is taxed as an ordinary 529 withdrawal and the amount deposited is a regular Roth contribution you enter on the IRA contributions page.
    pub confirmed: bool,
}

impl QtpRothRollover {
    // The shape of the stored facts, whatever their tax meaning.
    pub(crate) fn well_formed(&self) -> bool {
        let date = crate::dates::source_valid_date;
        !self.account_reference.trim().is_empty()
            && date(self.account_opened_on)
            && date(self.transferred_on)
            && (0..=100_000_000_000).contains(&self.seasoned_available)
    }

    // The facts of box 4b against the transfer's amount (the 1099-Q's boxes 2
    // and 3, in cents).
    pub(crate) fn valid(&self, amount: i64) -> bool {
        self.well_formed()
            && (20260101..=20261231).contains(&self.transferred_on)
            && self.account_opened_on / 10000 + crate::year_constants::QTP_ROTH_ACCOUNT_YEARS
                <= self.transferred_on / 10000
            && self.account_opened_on + crate::year_constants::QTP_ROTH_ACCOUNT_YEARS * 10000
                <= self.transferred_on
            && (1..=crate::year_constants::QTP_ROTH_LIFETIME_LIMIT_CENTS).contains(&amount)
            && amount <= self.seasoned_available
    }
}

/// 1099-Q
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-Q (Payments From Qualified Education Programs), Copy B, exactly as printed. Check its calendar year against the supplied year and report a mismatch as an error. Read corrected, both printed party blocks, payer telephone and account number, preserving masking, punctuation and leading zeros. Use empty text for absent identity fields. Never infer the recipient from household names. Read boxes 1 gross distribution, 2 earnings and 3 basis as decimal numbers without commas or currency signs; return zero for a clearly blank amount and preserve a printed box 2 loss as negative. Read boxes 4a and 4b independently as checked. For box 5 return the one printed selection: PrivateQtp for 5a, StateQtp for 5b, or CoverdellEsa for 5c; report an error when none or more than one is checked. Read box 6 exactly as checked. Read an FMV printed in box 7 into fair_market_value, including a printed zero; return null when no FMV is printed. Separately read the optional distribution code printed in box 7 into distribution_code; return null when no code is printed. Do not infer a code, FMV, earnings or basis. Report illegible or conflicting values as an error. Use a corrected form in place of the original. Do not choose a beneficiary, classify the transfer for tax purposes, calculate earnings, or supply personal account, education, rollover or Roth history. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer_trustee.federal_id,@.recipient.federal_id,@.calendar_year,@.fair_market_value.value,@.gross_distribution,@.earnings,@.basis,@.account_number"
)]
pub struct Form1099QPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: i32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S/TRUSTEE'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer_trustee: crate::PrintedParty,
    /// PAYER'S/TRUSTEE'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN and address
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
    pub gross_distribution: i64,
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
    /// Box 4a Trustee-to-trustee
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub trustee_to_trustee: bool,
    /// Box 4b QTP to Roth IRA
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub qtp_to_roth_ira: bool,
    /// Box 5 Distribution is from:
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub distribution_from: Form1099QDistributionFrom,
    /// Box 6 Check if the recipient is not the designated beneficiary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub recipient_not_beneficiary: bool,
    /// Box 7 If the fair market value (FMV) is shown below, see Pub. 970, Tax Benefits for Education, for how to figure earnings.
    /// [NullText]
    /// No fair market value is printed in box 7
    /// [SetText]
    /// The fair market value printed in box 7 is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub fair_market_value: Option<Form1099QFairMarketValue>,
    /// Box 7 Distribution code
    /// [NullText]
    /// No distribution code is printed in box 7
    /// [SetText]
    /// The distribution code printed in box 7 is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub distribution_code: Option<Form1099QDistributionCode>,
}

/// Box 5 Distribution is from:
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form1099QDistributionFrom {
    /// a Private QTP
    PrivateQtp,
    /// b State QTP
    StateQtp,
    /// c Coverdell ESA
    CoverdellEsa,
}

/// Fair market value printed in box 7
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1099QFairMarketValue {
    /// Fair market value
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub value: i64,
}

/// Distribution code printed in box 7
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Form1099QDistributionCode {
    /// 1 — Distributions (including transfers)
    #[libertas_form_code("1")]
    Distribution,
    /// 2 — Excess contributions plus earnings taxable in the calendar year
    #[libertas_form_code("2")]
    ExcessThisYear,
    /// 3 — Excess contributions plus earnings taxable in the prior calendar year
    #[libertas_form_code("3")]
    ExcessLastYear,
    /// 4 — Disability
    #[libertas_form_code("4")]
    Disability,
    /// 5 — Death
    #[libertas_form_code("5")]
    Death,
    /// 6 — Prohibited transaction
    #[libertas_form_code("6")]
    ProhibitedTransaction,
}

impl From<&Form1099QPaper> for crate::Q1099 {
    fn from(paper: &Form1099QPaper) -> Self {
        let Form1099QPaper {
            // Retained printed identifiers; no existing figure reads these boxes.
            calendar_year: _,
            corrected: _,
            payer_trustee,
            payer_telephone: _,
            recipient,
            account_number: _,
            gross_distribution,
            earnings,
            basis,
            trustee_to_trustee,
            qtp_to_roth_ira,
            distribution_from,
            recipient_not_beneficiary,
            fair_market_value,
            distribution_code,
        } = paper;
        let account = match distribution_from {
            Form1099QDistributionFrom::PrivateQtp | Form1099QDistributionFrom::StateQtp => {
                crate::EducationAccount::Qtp { earnings: *earnings, basis: *basis }
            }
            Form1099QDistributionFrom::CoverdellEsa => crate::EducationAccount::Coverdell {
                amounts: match fair_market_value {
                    Some(Form1099QFairMarketValue { value }) if *earnings == 0 && *basis == 0 => {
                        crate::CoverdellAmounts::ValueOnly { value: *value }
                    }
                    _ => crate::CoverdellAmounts::Reported { earnings: *earnings, basis: *basis },
                },
            },
        };
        let transfer = if *trustee_to_trustee {
            crate::Q1099Transfer::TrusteeTransfer
        } else if *qtp_to_roth_ira {
            crate::Q1099Transfer::RothIra
        } else {
            use Form1099QDistributionCode as Code;
            match distribution_code {
                Some(Code::ExcessThisYear) => crate::Q1099Transfer::ExcessThisYear,
                Some(Code::ExcessLastYear) => crate::Q1099Transfer::ExcessLastYear,
                Some(Code::ProhibitedTransaction) => crate::Q1099Transfer::ProhibitedTransaction,
                Some(Code::Disability) => crate::Q1099Transfer::Withdrawal {
                    reason: crate::DistributionReason::Disability,
                },
                Some(Code::Death) => {
                    crate::Q1099Transfer::Withdrawal { reason: crate::DistributionReason::Death }
                }
                Some(Code::Distribution) | None => crate::Q1099Transfer::Withdrawal {
                    reason: crate::DistributionReason::Withdrawal,
                },
            }
        };
        Self {
            payer_trustee: payer_trustee.clone(),
            recipient: recipient.clone(),
            gross: *gross_distribution,
            account,
            transfer,
            recipient_not_beneficiary: *recipient_not_beneficiary,
        }
    }
}
