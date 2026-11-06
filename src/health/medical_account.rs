// Coverage: Form 1099-SA with Form 8853 Sections A-B and Form 8889 Part II for a beneficiary
//   (the Archer MSA, Medicare Advantage MSA and inherited-account kinds share this owner).
//   Lines: Form 8853 Section A Part II lines 6a-9b (6a gross of every Archer MSA form, 6b the
//   rollover and the returned excess, 7 spending on qualified medical expenses, 8 -> Schedule 1
//   line 8e, 9a/9b the 20% additional tax unless the payment came after the 65th birthday, the
//   disability onset or the death -> Schedule 2 line 13e; a prohibited transaction is a
//   distribution of the January 1 value); Section B lines 10-13b (a Medicare Advantage MSA: 50%
//   additional tax with only the disability and death exceptions, reduced by the December 31, 2025
//   balance over 60% of the January 1 deductible, once for each spouse); the death of the account
//   holder (a nonspouse beneficiary includes the fair market value at death, box 4, less the
//   holder's medical bills paid within one year, for the year of the death on Form 8889 lines 14a-16
//   or Form 8853 line 6a/10; the estate case is the holder's final return without that reduction;
//   no additional tax); earnings after the death (box 1 minus box 4) and a returned Archer excess
//   (box 2) are Schedule 1 line 8z other income, as the 1099-SA recipient instructions say.
//   Entry: OtherIncomeDocument::Sa1099 (Form 1099-SA, importable; box 3 code 1/3 Own, 2 ExcessReturned,
//   4/6 Inherited, 5 Prohibited), or a separate trustee statement for unreported events.
//   The intake kind identifies the actual source; only Form 1099-SA requires box 3. Own HSA distributions use this
//   same canonical source; contribution, correction and testing facts stay with Health accounts.
//   Exclusions: the Archer MSA Part I contributions are the Health and medical savings accounts page's (Form
//   8853 Part I, archer_contributions.rs). Trustee statements reuse MedicalDeemed for unreported
//   collateral/account events. A spouse succeeds to the owned HSA/Archer account path;
//   post-death earnings of an estate belong to the estate's separate return.
//   Tests: tests/health/health_accounts.rs (fixtures F1-F7 and the import mapping),
//   tests/health/interview.rs sa1099_*. Sources: https://www.irs.gov/pub/irs-pdf/f1099sa.pdf (Rev. April
//   2025), https://www.irs.gov/pub/irs-dft/i8853--dft.pdf and f8853--dft.pdf (2026 drafts),
//   https://www.irs.gov/pub/irs-dft/i8889--dft.pdf (2026 draft), Pub. 969, 26 U.S.C. 138, 220,
//   223(f)(8).
//   Actual receipt/deposit parts, same-owner HSA/Archer
//   destinations and successful-only source-type annual history are projected
//   once before HSA/retirement/deduction candidates. Failed2026 cash enters the
//   existing contribution owner; future-designated cash remains source income
//   without a current contribution. Tests hsa_failed_cash_rollover_*,
//   hsa_rollover_split_day_*, hsa_rollover_history_counts_only_*.
//   Actual failed-deposit corrections use the original statement ID separately
//   from returned-excess statement ID, bound source/owner/destination/date/year
//   principal, and preserve manual totals on reopen. Calculation-only gross/net
//   later composition clears only transient original-deposit links. Tests
//   hsa_failed_deposit_*, archer_failed_deposit_correction_*.
//   Remaining boundaries: prior-year designation/carry, failed property and
//   competing same-day priority.
//   Code223(f)(5)/220(f)(5) source-type history is an interpretation distinguished
//   from broader IRS destination wording; ordinary failed-cash contribution
//   treatment derives from4973(g)/(d), not a published IRS worked example.
// Native source projection controls: tests/attribution/unit13_account_sources.rs (recipient,
// source cents, penalty character, correction coordination and Save/restart).
use crate::dates::{source_valid_date as valid_date, valid_birth};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact, whole};
use crate::{FederalPerson, MedicalRollover};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Internal Form 1099-SA record derived from the stored paper `Form1099SaPaper`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sa1099 {
    /// TRUSTEE'S/PAYER'S name, TIN and address
    pub trustee_payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Actual source document
    pub origin: HealthDocumentOrigin,
    /// Box 1 gross distribution
    pub gross: i64,
    /// Printed box 2 earnings
    pub box2_earnings: Option<i64>,
    /// Printed box 4 value on the date of death
    pub box4_value: Option<i64>,
    /// Printed box 3 distribution code
    pub box3_code: Option<u32>,
    /// Account (box 5)
    pub account: MedicalSaAccount,
}

// One-way: every printed box either reaches the internal record or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&SaSource> for Sa1099 {
    fn from(source: &SaSource) -> Self {
        let SaSource {
            form,
            origin,
            // Topic answers; the internal Form 1099-SA record holds the printed boxes only.
            medicare_prior: _,
            medicare_from_spouse: _,
            payment: _,
        } = source;
        match form {
            MedicalSaPaper::Form1099Sa(form) => {
                let crate::Form1099SaPaper {
                    calendar_year: _,
                    corrected: _,
                    trustee_payer,
                    recipient,
                    // Identifies the account for the trustee; no figure reads it.
                    account_number: _,
                    gross_distribution,
                    earnings_on_excess_contributions,
                    distribution_code,
                    fmv_on_date_of_death,
                    account,
                } = form;
                Self {
                    trustee_payer: trustee_payer.clone(),
                    recipient: recipient.clone(),
                    origin: *origin,
                    gross: *gross_distribution,
                    box2_earnings: *earnings_on_excess_contributions,
                    box4_value: *fmv_on_date_of_death,
                    box3_code: distribution_code.map(|code| code.printed_code()),
                    account: account.clone(),
                }
            }
            MedicalSaPaper::TrusteeStatement(statement) => {
                let HsaTrusteeStatementPaper {
                    trustee_payer,
                    recipient,
                    // Identifies the account for the trustee; no figure reads it.
                    account_number: _,
                    amount,
                    account,
                } = statement;
                Self {
                    trustee_payer: trustee_payer.clone(),
                    recipient: recipient.clone(),
                    origin: *origin,
                    gross: *amount,
                    box2_earnings: None,
                    box4_value: None,
                    box3_code: None,
                    account: account.clone(),
                }
            }
        }
    }
}

/// Trustee statement for an HSA or Archer MSA account event
/// Enter the actual statement; payment treatment and event dates are answered separately.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the actual HSA or Archer MSA trustee statement. Copy trustee_payer and recipient names, identification numbers and addresses as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Read account_number as printed, or empty text when absent. Read amount as the documented value of the affected account or pledged assets, without deriving a total from other figures; report an absent, illegible or conflicting value as an error. Read account as Hsa or Archer only when the statement identifies it; do not infer the account type. Do not invent Form 1099-SA boxes, a distribution code, event dates, payment treatment, disability, rollover or medical-spending answers.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.trustee_payer.federal_id,@.recipient.federal_id,@.amount,@.account_number"
)]
pub struct HsaTrusteeStatementPaper {
    /// Trustee's name, identification number and address
    #[libertas_llm_doc_field]
    pub trustee_payer: crate::PrintedParty,
    /// Account holder's name, identification number and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: alloc::string::String,
    /// Documented value of the affected account or pledged assets
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Account type named on the statement
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub account: MedicalSaAccount,
}

/// Actual medical-account paper
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalSaPaper {
    /// Form 1099-SA
    Form1099Sa(crate::Form1099SaPaper),
    /// Trustee statement for an unreported account event
    TrusteeStatement(HsaTrusteeStatementPaper),
}

/// Accepted medical-account document
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SaSource {
    /// Actual form or trustee statement as printed
    pub form: MedicalSaPaper,
    /// Actual source document
    /// Choose a real Form1099-SA or the trustee's statement. Unreported account events use a trustee statement; do not fabricate numbered form boxes.
    #[libertas_default(Form1099Sa)]
    pub origin: HealthDocumentOrigin,
    /// Medicare MSA prior-year records
    #[libertas_default($null)]
    pub medicare_prior: Option<MedicareMsaPrior>,
    /// Medicare MSA inherited from your spouse
    #[libertas_default($null)]
    pub medicare_from_spouse: Option<bool>,
    /// Your payment records
    #[libertas_default($null)]
    pub payment: Option<MedicalSaPayment>,
}

/// Actual medical-account source document
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
pub enum HealthDocumentOrigin {
    #[default]
    /// Form1099-SA
    Form1099Sa,
    /// Trustee statement or documented account event
    TrusteeStatement,
}

/// Account (box 5)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MedicalSaAccount {
    #[default]
    /// HSA (box 5 HSA)
    /// Includes your own account and an HSA inherited from your spouse. Other beneficiaries choose the inherited payment treatment.
    Hsa,
    /// Archer MSA (box 5 Archer MSA)
    Archer,
    /// Medicare Advantage MSA (box 5 MA MSA)
    MedicareAdvantage,
}

/// What the payment was (box 3 code)
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
pub enum MedicalSaPayment {
    /// Paid from your own account (code 1 or 3)
    /// Including an Archer MSA you inherited from your spouse.
    Own {
        /// Nonmedical payments on this statement
        /// Enter each actual payment date and amount, including payments before and after an age or disability exception. Keep one real statement as one record. Together with medical spending and the rollover, these payments must equal box1 gross.
        /// ----
        /// Payment
        nonmedical: Vec<MedicalAccountWithdrawal>,
        /// Spent on qualified medical expenses
        /// Unreimbursed medical costs of you, your spouse or your dependents (only your own for a Medicare Advantage MSA) not also deducted on Schedule A. Most insurance premiums do not count. If anyone contributed to an Archer MSA for 2026, only costs of people covered by the high deductible plan then count.
        /// For an HSA, qualifying direct-primary-care fixed fees paid by you can count even above the $150/$300 contribution-eligibility limits. The arrangement must satisfy the qualifying primary-care practitioner/service and sole-fixed-fee terms; employer-paid or cafeteria-plan fees are not your expense. Use substantiated fees under the permitted timing method: each coverage month's first day pro rata, the coverage period's first day, or the payment date. Do not count the same expense again.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        medical: i64,
        /// Actual attempted indirect rollovers
        /// Enter actual source receipts and deposit parts, including unsuccessful attempts. The App tests 60 days, destination and successful-only annual history. Failed cash needs its actual ordinary contribution designation; leave it out of manual contributions on Health accounts. All destinations must belong to this same owner. Direct trustee transfers are not distributions on this form.
        /// [NullText]
        /// No indirect rollover was attempted from this statement
        /// [SetText]
        /// The attempted indirect rollovers were
        rollover: Option<MedicalRollover>,
        /// Date you became disabled
        /// Leave empty unless you cannot do any substantial gainful activity because of a condition expected to last indefinitely or end in death.
        /// [NullText]
        /// I did not meet the disability exception by the end of 2026
        /// [SetText]
        /// I became disabled on
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(max = $TAX_YEAR_END_DATE)]
        disabled_since: Option<u32>,
    },
    /// Excess contributions returned (code 2)
    /// Only the box 2 earnings are taxed this year.
    ExcessReturned,
    /// Prohibited transaction (code 5)
    /// The account ended on January 1; box 1 is its value then. Pledging the account as loan security is not calculated.
    Prohibited {
        /// Date of the transaction
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = 20260101, max = 20261231)]
        occurred: u32,
    },
    /// Unreported account event
    /// Use a trustee statement for an HSA or Archer MSA pledged as loan security or terminated by a prohibited transaction not reported on a Form1099-SA. Enter the affected account once; exclude its terminated-account cash withdrawals and the same pledged assets from other entries.
    Unreported {
        /// Actual terminated-account or pledged-assets facts
        event: crate::MedicalDeemed,
        /// Disability onset, if applicable
        /// [NullText]
        /// I did not meet the disability exception by the end of 2026
        /// [SetText]
        /// I became disabled on
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(max = $TAX_YEAR_END_DATE)]
        disabled_since: Option<u32>,
    },
    /// You inherited the account (code 4 or 6)
    /// From someone other than your spouse, or for the estate on the holder's final return. A spouse's HSA or Archer MSA is your own: enter it as your own.
    Inherited {
        /// Date the account holder died
        /// For a death before 2026, only the earnings after the death are taxed this year.
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = 19970101, max = 20261231)]
        died: u32,
        /// Who inherited it
        #[libertas_inline_choices]
        beneficiary: MedicalBeneficiary,
    },
}

/// Who inherited it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalBeneficiary {
    /// I did
    Nonspouse {
        /// Account holder's medical bills you paid
        /// Bills for care before the death that you paid within one year after it. Do not deduct them again on Schedule A.
        /// ----
        /// Bill
        payments: Vec<InheritedMedicalPayment>,
    },
    /// The estate did; this is the holder's final return
    /// The whole value is income on the final return of the person who died.
    Estate,
}

/// Medical-account distribution from one actual source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalMedicalDistribution {
    /// Actual source document
    #[libertas_default(Form1099Sa)]
    pub origin: HealthDocumentOrigin,
    /// Box 1 gross distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Printed box 2 earnings, absent only when blank
    pub box2_earnings: Option<i64>,
    /// Printed box 4 value on the date of death
    pub box4_value: Option<i64>,
    /// Printed box 3 code, absent for trustee-only events
    pub box3_code: Option<u32>,
    /// Account (box 5)
    pub account: MedicalSaAccount,
    /// Your Medicare MSA prior-year records
    pub medicare_prior: Option<MedicareMsaPrior>,
    /// Whether this Medicare MSA was inherited from your spouse
    pub medicare_from_spouse: Option<bool>,
    /// Your payment records
    pub payment: Option<MedicalSaPayment>,
}

/// Nonmedical health or medical savings account payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MedicalAccountWithdrawal {
    /// Payment date in 2026
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub date: u32,
    /// Nonmedical amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Prior-year Medicare MSA account
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicareMsaPrior {
    /// No Medicare MSA on December 31, 2025
    None,
    /// Medicare MSA existed on December 31, 2025
    Existing {
        /// Total account value on December 31, 2025
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        balance: i64,
        /// Annual HDHP deductible on January 1, 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductible: i64,
    },
}

/// Beneficiary-paid medical bill
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedMedicalPayment {
    /// Date the account holder incurred the qualified medical expense
    #[libertas_date_only]
    pub incurred: u32,
    /// Date you paid the bill
    /// Only payments after the death and within one year reduce the inherited income. Later payments and expenses incurred after the death do not.
    #[libertas_date_only]
    pub paid: u32,
    /// Qualified unreimbursed amount you paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

impl InheritedMedicalPayment {
    fn valid(&self) -> bool {
        valid_date(self.incurred)
            && valid_date(self.paid)
            && self.paid >= self.incurred
            && self.amount >= 0
    }
}

impl MedicareMsaPrior {
    fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Existing { balance, deductible } => {
                [balance, deductible].iter().all(|v| **v >= 0)
            }
        }
    }
    // Form 8853 line 13b worksheet: the December 31, 2025 balance over 60% of the
    // January 1 deductible (whole dollars).
    pub(crate) fn relief(&self) -> Exact {
        match self {
            Self::None => whole(0),
            Self::Existing { balance, deductible } => phase::floor(
                exact::dollars_exact(&whole(*balance))
                    - exact::rounded_exact(
                        &(exact::dollars_exact(&whole(*deductible)) * whole(60) / whole(100)),
                    ),
                whole(0),
            ),
        }
    }
}

// The Section B facts a Medicare Advantage MSA form hands to the spouse total.
#[derive(Clone)]
pub(crate) struct MedicareForm {
    pub id: i64,
    pub owner: usize,
    pub context: crate::IssueContext,
    pub taxable: Exact,
    pub penalized: Exact,
    pub prior: MedicareMsaPrior,
}

// What one Form 1099-SA contributes to the return (cents unless noted).
#[derive(Debug, Default)]
pub(crate) struct SaAmounts {
    // Archer MSA Part II line 6c less line 7 and the rollover (line 8), cents.
    pub taxable: Exact,
    // Line 9 base: the nonmedical amount dated on or before the 65th birthday and
    // the disability onset, cents.
    pub penalized: Exact,
    // Schedule 1 line 8z: returned excess earnings and earnings after the death.
    pub earnings: Exact,
    // The inherited death-year value included, whole dollars.
    pub inherited: Exact,
    // The holder's bills the beneficiary paid within a year, whole dollars.
    pub inherited_medical: Exact,
    // Set when box 1 is below box 4 (the account fell after the death).
    pub decline: bool,
}

// A completed Form1099-SA source, after the collector's actual linked-Archer
// correction suppression. Components stay separate to preserve native form
// rounding and the account-specific additional-tax character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MedicalSource {
    pub(crate) id: i64,
    pub(crate) owner: crate::Owner,
    pub(crate) account: MedicalSaAccount,
    pub(crate) taxable: Exact,
    pub(crate) penalized: Exact,
    pub(crate) earnings: Exact,
    // Cents representing the already-rounded native on-return whole-dollar
    // inheritance amount, not the original unrounded account value.
    pub(crate) inherited: Exact,
}

impl SaAmounts {
    // Call only after the collector has completed actual correction linking.
    // This projects the owning calculation; it never calculates the payment again.
    pub(crate) fn source(
        &self,
        id: i64,
        owner: crate::Owner,
        account: &MedicalSaAccount,
    ) -> MedicalSource {
        MedicalSource {
            id,
            owner,
            account: account.clone(),
            taxable: self.taxable.clone(),
            penalized: self.penalized.clone(),
            earnings: self.earnings.clone(),
            inherited: &self.inherited * whole(100),
        }
    }
}

// Transient Form8889 consumer totals from the canonical statement records.
// No second persisted distribution ledger or contribution eligibility is required.
#[derive(Clone, Debug, Default)]
pub(crate) struct OwnedHsaAmounts {
    pub gross: Exact,
    pub medical: Exact,
    pub rollover: Exact,
    pub returned_excess: Exact,
    pub deemed: Exact,
    pub pledged: Exact,
    pub pledged_penalized: Exact,
    pub taxable: Exact,
    pub penalized: Exact,
}

// Only an actual required native output may narrow an exact account amount.
// The caller supplies that output's owning account or statement context.
#[derive(Debug)]
pub(crate) struct OutputFailure {
    pub context: crate::IssueContext,
}

pub(crate) fn required_output(
    value: &Exact,
    context: crate::IssueContext,
) -> Result<i64, OutputFailure> {
    exact::rounded(value).ok_or(OutputFailure { context })
}

// A Medicare Advantage MSA inherited from the spouse is the spouse's Archer MSA.
fn calculation_account(
    account: &MedicalSaAccount,
    from_spouse: Option<bool>,
) -> Option<&MedicalSaAccount> {
    match account {
        MedicalSaAccount::MedicareAdvantage => {
            from_spouse.map(|spouse| if spouse { &MedicalSaAccount::Archer } else { account })
        }
        _ => Some(account),
    }
}

fn medical_sa_source_issue(
    origin: HealthDocumentOrigin,
    gross: i64,
    box2_earnings: Option<i64>,
    box3_code: Option<u32>,
    box4_value: Option<i64>,
    account: &MedicalSaAccount,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if gross < 0 {
        return Some(
            ValidationIssue::new(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            )
            .at_field("gross"),
        );
    }
    if box3_code == Some(2) && !matches!(account, MedicalSaAccount::Hsa | MedicalSaAccount::Archer)
    {
        return Some(ValidationIssue::new("SA1099_EXCESS_NOT_ARCHER").at_field("account"));
    }
    if box2_earnings.is_some_and(|amount| amount < 0 || (box3_code == Some(2) && amount > gross)) {
        return Some(ValidationIssue::new("SA1099_EARNINGS_ABOVE_GROSS").at_field("box2_earnings"));
    }
    if box4_value.is_some_and(|amount| amount < 0) {
        return Some(
            ValidationIssue::new(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            )
            .at_field("box4_value"),
        );
    }
    if (origin == HealthDocumentOrigin::Form1099Sa
        && !box3_code.is_some_and(|code| (1..=6).contains(&code)))
        || (origin == HealthDocumentOrigin::TrusteeStatement && box3_code.is_some())
    {
        return Some(ValidationIssue::new("SA1099_PRINTED_CODE_NEEDED").at_field("box3_code"));
    }
    if matches!(box3_code, Some(4 | 6)) && box4_value.is_none() {
        return Some(ValidationIssue::new("SA1099_DEATH_VALUE_NEEDED").at_field("box4_value"));
    }
    None
}

impl crate::Form1099SaPaper {
    pub(crate) fn input_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.source_issue(HealthDocumentOrigin::Form1099Sa)
    }

    fn source_issue(
        &self,
        origin: HealthDocumentOrigin,
    ) -> Option<crate::validation::ValidationIssue> {
        medical_sa_source_issue(
            origin,
            self.gross_distribution,
            self.earnings_on_excess_contributions,
            self.distribution_code.map(|code| code.printed_code()),
            self.fmv_on_date_of_death,
            &self.account,
        )
        .map(|issue| {
            issue
                .map_field_prefix(&["gross"], &["gross_distribution"])
                .map_field_prefix(&["box2_earnings"], &["earnings_on_excess_contributions"])
                .map_field_prefix(&["box3_code"], &["distribution_code"])
                .map_field_prefix(&["box4_value"], &["fmv_on_date_of_death"])
        })
    }
}

impl HsaTrusteeStatementPaper {
    pub(crate) fn input_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.source_issue(HealthDocumentOrigin::TrusteeStatement)
    }

    fn source_issue(
        &self,
        origin: HealthDocumentOrigin,
    ) -> Option<crate::validation::ValidationIssue> {
        medical_sa_source_issue(origin, self.amount, None, None, None, &self.account)
            .map(|issue| issue.map_field_prefix(&["gross"], &["amount"]))
    }
}

impl SaSource {
    pub(crate) fn calculation_account(&self) -> Option<&MedicalSaAccount> {
        calculation_account(self.account(), self.medicare_from_spouse)
    }
    pub(crate) fn account(&self) -> &MedicalSaAccount {
        match &self.form {
            MedicalSaPaper::Form1099Sa(form) => &form.account,
            MedicalSaPaper::TrusteeStatement(statement) => &statement.account,
        }
    }
    #[cfg(test)]
    pub(crate) fn gross_mut(&mut self) -> &mut i64 {
        match &mut self.form {
            MedicalSaPaper::Form1099Sa(form) => &mut form.gross_distribution,
            MedicalSaPaper::TrusteeStatement(statement) => &mut statement.amount,
        }
    }
    #[cfg(test)]
    pub(crate) fn account_mut(&mut self) -> &mut MedicalSaAccount {
        match &mut self.form {
            MedicalSaPaper::Form1099Sa(form) => &mut form.account,
            MedicalSaPaper::TrusteeStatement(statement) => &mut statement.account,
        }
    }
    #[cfg(test)]
    pub(crate) fn form1099sa(&self) -> Option<&crate::Form1099SaPaper> {
        match &self.form {
            MedicalSaPaper::Form1099Sa(form) => Some(form),
            MedicalSaPaper::TrusteeStatement(_) => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn form1099sa_mut(&mut self) -> Option<&mut crate::Form1099SaPaper> {
        match &mut self.form {
            MedicalSaPaper::Form1099Sa(form) => Some(form),
            MedicalSaPaper::TrusteeStatement(_) => None,
        }
    }
    pub(crate) fn trustee_statement_paper(&self) -> Option<&HsaTrusteeStatementPaper> {
        match &self.form {
            MedicalSaPaper::TrusteeStatement(statement) => Some(statement),
            MedicalSaPaper::Form1099Sa(_) => None,
        }
    }
    /// The internal medical-account record derived from the paper.
    pub(crate) fn internal(&self) -> Sa1099 {
        Sa1099::from(self)
    }
    /// The internal record the calculations read.
    #[cfg(test)]
    pub(crate) fn record(&self) -> FederalMedicalDistribution {
        self.into()
    }
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: crate::Form1099SaPaper) -> Self {
        Self {
            form: MedicalSaPaper::Form1099Sa(form),
            origin: HealthDocumentOrigin::Form1099Sa,
            medicare_prior: None,
            medicare_from_spouse: None,
            payment: None,
        }
    }
    /// A trustee statement enters without an answer about its account event.
    pub(crate) fn trustee_statement(statement: HsaTrusteeStatementPaper) -> Self {
        Self {
            form: MedicalSaPaper::TrusteeStatement(statement),
            origin: HealthDocumentOrigin::TrusteeStatement,
            medicare_prior: None,
            medicare_from_spouse: None,
            payment: None,
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.medicare_prior = accepted.medicare_prior.clone();
        self.medicare_from_spouse = accepted.medicare_from_spouse;
        self.payment = accepted.payment.clone();
    }
    pub(crate) fn input_issue(&self, personal: bool) -> Option<crate::validation::ValidationIssue> {
        match &self.form {
            MedicalSaPaper::Form1099Sa(form) => form.source_issue(self.origin),
            MedicalSaPaper::TrusteeStatement(statement) => statement.source_issue(self.origin),
        }
        .or_else(|| {
            personal.then(|| FederalMedicalDistribution::from(self).answer_issue()).flatten()
        })
    }
    pub(crate) fn topic_issue(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        let record = FederalMedicalDistribution::from(self);
        record.source_issue().or_else(|| record.answer_issue_with_required(require_answers))
    }
}
// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&SaSource> for FederalMedicalDistribution {
    fn from(source: &SaSource) -> Self {
        let SaSource { form: _, origin: _, medicare_prior, medicare_from_spouse, payment } = source;
        let form = Sa1099::from(source);
        let Sa1099 {
            // The parties name and assign the document; no figure reads them.
            trustee_payer: _,
            recipient: _,
            origin,
            gross,
            box2_earnings,
            box3_code,
            box4_value,
            account,
        } = &form;
        FederalMedicalDistribution {
            origin: *origin,
            gross: *gross,
            box2_earnings: *box2_earnings,
            box4_value: *box4_value,
            box3_code: *box3_code,
            account: account.clone(),
            medicare_prior: medicare_prior.clone(),
            medicare_from_spouse: *medicare_from_spouse,
            payment: payment.clone(),
        }
    }
}
impl FederalMedicalDistribution {
    pub(crate) fn valid(&self) -> bool {
        self.source_issue().is_none()
            && self.medicare_prior.as_ref().is_none_or(MedicareMsaPrior::valid)
            && match &self.payment {
                Some(MedicalSaPayment::Own { nonmedical, medical, rollover, disabled_since }) => {
                    nonmedical.iter().all(|w| {
                        valid_date(w.date)
                            && (crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE)
                                .contains(&w.date)
                            && w.amount >= 0
                    }) && *medical >= 0
                        && rollover.as_ref().is_none_or(MedicalRollover::valid)
                        && disabled_since.is_none_or(valid_birth)
                }
                Some(MedicalSaPayment::ExcessReturned) | None => true,
                Some(MedicalSaPayment::Prohibited { occurred }) => {
                    valid_date(*occurred) && occurred / 10000 == 2026
                }
                Some(MedicalSaPayment::Unreported { event, disabled_since }) => {
                    event.valid() && disabled_since.is_none_or(valid_birth)
                }
                Some(MedicalSaPayment::Inherited { died, beneficiary }) => {
                    valid_date(*died)
                        && (19970101..=20261231).contains(died)
                        && match beneficiary {
                            MedicalBeneficiary::Nonspouse { payments } => {
                                payments.iter().all(|p| p.valid())
                            }
                            MedicalBeneficiary::Estate => true,
                        }
                }
            }
    }
    pub(crate) fn effective_payment(&self) -> Option<&MedicalSaPayment> {
        self.payment.as_ref().or_else(|| {
            (self.origin == HealthDocumentOrigin::Form1099Sa && self.box3_code == Some(2))
                .then_some(&MedicalSaPayment::ExcessReturned)
        })
    }
    pub(crate) fn calculation_account(&self) -> Option<&MedicalSaAccount> {
        calculation_account(&self.account, self.medicare_from_spouse)
    }
    pub(crate) fn source_issue(&self) -> Option<crate::validation::ValidationIssue> {
        medical_sa_source_issue(
            self.origin,
            self.gross,
            self.box2_earnings,
            self.box3_code,
            self.box4_value,
            &self.account,
        )
    }
    pub(crate) fn answer_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.answer_issue_with_required(true)
    }
    pub(crate) fn answer_issue_with_required(
        &self,
        require_answers: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let Some(payment) = self.effective_payment() else {
            return require_answers.then(|| {
                ValidationIssue::new("SA1099_PAYMENT_RECORDS_NEEDED").at_field("payment")
            });
        };
        let matches_source = matches!(
            (self.origin, self.box3_code, payment),
            (HealthDocumentOrigin::TrusteeStatement, None, MedicalSaPayment::Unreported { .. })
                | (HealthDocumentOrigin::Form1099Sa, Some(1 | 3), MedicalSaPayment::Own { .. })
                | (HealthDocumentOrigin::Form1099Sa, Some(2), MedicalSaPayment::ExcessReturned)
                | (
                    HealthDocumentOrigin::Form1099Sa,
                    Some(4 | 6),
                    MedicalSaPayment::Inherited { .. }
                )
                | (HealthDocumentOrigin::Form1099Sa, Some(5), MedicalSaPayment::Prohibited { .. })
        );
        if !matches_source {
            return Some(ValidationIssue::new("SA1099_PAYMENT_NOT_PRINTED").at_field("payment"));
        }
        if let Some(issue) = self.personal_issue() {
            return Some(issue);
        }
        if require_answers && self.calculation_account().is_none() {
            return Some(
                ValidationIssue::new("SA1099_MEDICARE_SPOUSAL_RECORDS_NEEDED")
                    .at_field("medicare_from_spouse"),
            );
        }
        if require_answers
            && matches!(self.calculation_account(), Some(MedicalSaAccount::MedicareAdvantage))
            && matches!(payment, MedicalSaPayment::Own { .. } | MedicalSaPayment::Prohibited { .. })
            && self.medicare_prior.is_none()
        {
            return Some(
                ValidationIssue::new("SA1099_MEDICARE_PRIOR_RECORDS_NEEDED")
                    .at_field("medicare_prior"),
            );
        }
        None
    }
    // The document's validation rules, checked again on save with the same keys.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.source_issue().or_else(|| self.answer_issue())
    }
    fn personal_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let payment = self.effective_payment()?;
        let account = self.calculation_account();
        let own = matches!(payment, MedicalSaPayment::Own { .. });
        let archer = matches!(account, Some(MedicalSaAccount::Hsa | MedicalSaAccount::Archer));
        match payment {
            MedicalSaPayment::Unreported { event, .. } => {
                if self.origin != HealthDocumentOrigin::TrusteeStatement
                    || account.is_some_and(|account| {
                        !matches!(account, MedicalSaAccount::Hsa | MedicalSaAccount::Archer)
                    })
                {
                    Some(ValidationIssue::new("SA1099_UNREPORTED_SOURCE_INVALID").at_field(
                        if self.origin != HealthDocumentOrigin::TrusteeStatement {
                            "origin"
                        } else {
                            "account"
                        },
                    ))
                } else if whole(self.gross) != event.amount() {
                    Some(
                        ValidationIssue::new("SA1099_EVENT_VALUE_NOT_GROSS")
                            .at_field("event")
                            .in_field("payment"),
                    )
                } else {
                    None
                }
            }
            MedicalSaPayment::Own { nonmedical, medical, rollover, .. } if own => {
                if let Some(r) = rollover {
                    for (i, receipt) in r.receipts.iter().enumerate() {
                        for (j, deposit) in receipt.deposits.iter().enumerate() {
                            if let Some(issue) = deposit.issue() {
                                return Some(
                                    issue
                                        .in_list("deposits", j)
                                        .in_list("receipts", i)
                                        .in_field("rollover")
                                        .in_field("payment"),
                                );
                            }
                        }
                    }
                }
                let medical_and_rollover =
                    whole(*medical) + rollover.as_ref().map_or_else(|| whole(0), |v| v.amount());
                if medical_and_rollover > whole(self.gross) {
                    return Some(
                        ValidationIssue::new("SA1099_MEDICAL_ABOVE_GROSS")
                            .at_field("medical")
                            .in_field("payment"),
                    );
                }
                if rollover.is_some() && account.is_some() && !archer {
                    return Some(
                        ValidationIssue::new("SA1099_ROLLOVER_NOT_ARCHER")
                            .at_field("rollover")
                            .in_field("payment"),
                    );
                }
                if let Some(v) = rollover {
                    for (i, part) in v.receipts.iter().enumerate() {
                        if let Some((j, _)) = part
                            .deposits
                            .iter()
                            .enumerate()
                            .find(|(_, deposit)| deposit.deposited < part.received)
                        {
                            return Some(
                                ValidationIssue::new("SA1099_ROLLOVER_BEFORE_RECEIPT")
                                    .at_field("deposited")
                                    .in_list("deposits", j)
                                    .in_list("receipts", i)
                                    .in_field("rollover")
                                    .in_field("payment"),
                            );
                        }
                    }
                }
                if medical_and_rollover + nonmedical.iter().map(|w| whole(w.amount)).sum::<Exact>()
                    != whole(self.gross)
                {
                    return Some(ValidationIssue::new("SA1099_USES_NOT_GROSS").at_field("payment"));
                }
                None
            }
            MedicalSaPayment::ExcessReturned => None,
            MedicalSaPayment::Inherited { .. }
                if self.medicare_from_spouse == Some(true)
                    && matches!(self.account, MedicalSaAccount::MedicareAdvantage) =>
            {
                Some(ValidationIssue::new("SA1099_SPOUSE_ACCOUNT_IS_OWN").at_field("payment"))
            }
            _ => None,
        }
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    // The holder's medical bills the beneficiary paid within a year of the death,
    // capped at the value, whole dollars.
    fn bills(value: i64, died: u32, beneficiary: &MedicalBeneficiary) -> Exact {
        match beneficiary {
            MedicalBeneficiary::Nonspouse { payments } => exact::dollars_exact(&phase::cap(
                payments
                    .iter()
                    .filter(|p| p.incurred <= died && p.paid >= died && p.paid <= died + 10000)
                    .map(|p| whole(p.amount))
                    .sum::<Exact>(),
                [whole(value)],
            )),
            MedicalBeneficiary::Estate => whole(0),
        }
    }
    // The Section A, Section B and Form 8889 amounts of this form for its owner,
    // or the issue it needs before it can be figured.
    pub(crate) fn amounts(
        &self,
        person: &FederalPerson,
    ) -> Result<SaAmounts, (&'static str, &'static str)> {
        if let Some(key) = self.rule() {
            return Err((
                key,
                crate::fixture_english!(
                    "Correct the medical-account source facts before calculating this distribution."
                ),
            ));
        }
        let account = self.calculation_account().unwrap();
        let medicare = matches!(account, MedicalSaAccount::MedicareAdvantage);
        match self.effective_payment().unwrap() {
            MedicalSaPayment::Own { nonmedical, medical, rollover, disabled_since } => {
                if rollover.as_ref().is_some_and(|v| !v.eligible(account)) {
                    return Err((
                        "ISSUE_ARCHER_MSA_TREATMENT_UNCONFIRMED",
                        crate::fixture_english!(
                            "A rollover to an Archer MSA or HSA must be deposited within 60 days of the payment and be the only one from that account type in 12 months; an ineligible rollover needs excess-contribution treatment."
                        ),
                    ));
                }
                if disabled_since.is_some_and(|d| d < person.birth_date) {
                    return Err((
                        "ISSUE_ARCHER_MSA_DISABILITY_BEFORE_BIRTH",
                        crate::fixture_english!(
                            "The disability onset on a Form 1099-SA cannot precede the account holder's birth."
                        ),
                    ));
                }
                if person.death_date().is_some_and(|d| {
                    nonmedical.iter().any(|w| w.date > d)
                        || rollover.as_ref().is_some_and(|v| {
                            v.receipts.iter().any(|part| {
                                part.received > d
                                    || part.deposits.iter().any(|deposit| deposit.deposited > d)
                            })
                        })
                }) {
                    return Err((
                        "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY",
                        crate::fixture_english!(
                            "A payment after the account holder's death needs the beneficiary's account or inheritance treatment."
                        ),
                    ));
                }
                let taxable: Exact = nonmedical.iter().map(|w| whole(w.amount)).sum();
                if whole(*medical)
                    + rollover.as_ref().map_or_else(|| whole(0), |v| v.amount())
                    + &taxable
                    != whole(self.gross)
                {
                    return Err((
                        "SA1099_USES_NOT_GROSS",
                        crate::fixture_english!(
                            "Reconcile medical spending, the rollover and all dated nonmedical payments to this statement's gross distribution."
                        ),
                    ));
                }
                // Existing account-specific boundaries apply to each actual
                // payment, not the last payment reported on an annual form.
                let penalized = nonmedical
                    .iter()
                    .filter(|w| {
                        if medicare {
                            disabled_since.is_none_or(|d| w.date < d)
                                && person.death_date().is_none_or(|d| w.date < d)
                        } else {
                            !crate::accepted_filers::after_65th_birthday(person.birth_date, w.date)
                                && disabled_since.is_none_or(|d| w.date <= d)
                        }
                    })
                    .map(|w| whole(w.amount))
                    .sum();
                Ok(SaAmounts { taxable, penalized, ..SaAmounts::default() })
            }
            MedicalSaPayment::Unreported { event, disabled_since } => {
                if disabled_since.is_some_and(|d| d < person.birth_date) {
                    return Err((
                        "ISSUE_ARCHER_MSA_DISABILITY_BEFORE_BIRTH",
                        crate::fixture_english!(
                            "Disability onset cannot precede the account owner's birth."
                        ),
                    ));
                }
                if person.death_date().is_some_and(|d| event.last_event() > d) {
                    return Err((
                        "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY",
                        crate::fixture_english!(
                            "Post-death account events need beneficiary treatment."
                        ),
                    ));
                }
                Ok(SaAmounts {
                    taxable: event.amount(),
                    penalized: event.penalized(|date| {
                        !crate::accepted_filers::after_65th_birthday(person.birth_date, date)
                            && disabled_since.is_none_or(|d| date <= d)
                    }),
                    ..SaAmounts::default()
                })
            }
            MedicalSaPayment::ExcessReturned => Ok(SaAmounts {
                // HSA income comes from the linked actual corrective NIA once.
                earnings: if matches!(self.account, MedicalSaAccount::Hsa) {
                    whole(0)
                } else {
                    whole(self.box2_earnings.unwrap_or(0))
                },
                ..SaAmounts::default()
            }),
            MedicalSaPayment::Prohibited { occurred } => {
                if person.death_date().is_some_and(|d| *occurred > d) {
                    return Err((
                        "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY",
                        crate::fixture_english!(
                            "A payment after the account holder's death needs the beneficiary's account or inheritance treatment."
                        ),
                    ));
                }
                // The January 1 value is distributed on January 1.
                let penalized = if medicare {
                    person.death_date().is_none_or(|d| 20260101 < d)
                } else {
                    !crate::accepted_filers::after_65th_birthday(person.birth_date, 20260101)
                };
                Ok(SaAmounts {
                    taxable: whole(self.gross),
                    penalized: whole(if penalized { self.gross } else { 0 }),
                    ..SaAmounts::default()
                })
            }
            MedicalSaPayment::Inherited { died, beneficiary } => {
                let value = &self.box4_value.unwrap();
                if matches!(beneficiary, MedicalBeneficiary::Estate)
                    && person.death_date() != Some(*died)
                {
                    return Err((
                        "ISSUE_INHERITED_MEDICAL_ESTATE_DATE_MISMATCH",
                        crate::fixture_english!(
                            "An estate-beneficiary account belongs on the original holder's final return; its death date must match People."
                        ),
                    ));
                }
                if !matches!(beneficiary, MedicalBeneficiary::Estate)
                    && person.death_date().is_some_and(|date| date < *died)
                {
                    return Err((
                        "ISSUE_INHERITED_MEDICAL_BENEFICIARY_PREDECEASED",
                        crate::fixture_english!(
                            "An individual medical account beneficiary must survive the original holder; use estate treatment when the estate is the beneficiary."
                        ),
                    ));
                }
                let bills = if died / 10000 == 2026 {
                    Self::bills(*value, *died, beneficiary)
                } else {
                    whole(0)
                };
                let inherited = if died / 10000 == 2026 {
                    phase::floor(exact::dollars_exact(&whole(*value)) - &bills, whole(0))
                } else {
                    whole(0)
                };
                // Earnings after the death are the beneficiary's own income; an estate's
                // belong to the estate's return, not the holder's final return.
                let earnings = if matches!(beneficiary, MedicalBeneficiary::Estate) {
                    whole(0)
                } else {
                    phase::floor(whole(self.gross) - whole(*value), whole(0))
                };
                Ok(SaAmounts {
                    earnings,
                    inherited,
                    inherited_medical: bills,
                    decline: !matches!(beneficiary, MedicalBeneficiary::Estate)
                        && self.gross < *value,
                    ..SaAmounts::default()
                })
            }
        }
    }
    // The row preview in cents: what the form adds to the return's income before
    // the checks that need the people page. None for an ineligible rollover.
    pub(crate) fn preview(&self) -> Option<i64> {
        if self.rule().is_some() {
            return None;
        }
        match self.effective_payment()? {
            MedicalSaPayment::Own { medical, rollover, .. } => {
                if rollover.is_some() {
                    // Successful-only annual history belongs to the coordinated estimate.
                    return None;
                }
                exact::rounded(&(whole(self.gross) - whole(*medical)))
            }
            MedicalSaPayment::Prohibited { .. } | MedicalSaPayment::Unreported { .. } => {
                Some(self.gross)
            }
            MedicalSaPayment::ExcessReturned => {
                if matches!(self.account, MedicalSaAccount::Hsa) {
                    None
                } else {
                    self.box2_earnings
                }
            }
            MedicalSaPayment::Inherited { died, beneficiary } => {
                let value = &self.box4_value?;
                let death_year = if died / 10000 == 2026 {
                    phase::floor(
                        exact::dollars_exact(&whole(*value))
                            - Self::bills(*value, *died, beneficiary),
                        whole(0),
                    ) * whole(100)
                } else {
                    whole(0)
                };
                let earnings = if matches!(beneficiary, MedicalBeneficiary::Estate) {
                    whole(0)
                } else {
                    phase::floor(whole(self.gross) - whole(*value), whole(0))
                };
                exact::rounded(&(death_year + earnings))
            }
        }
    }
    // The Section B facts of an own or prohibited-transaction Medicare Advantage
    // MSA form, handed to its spouse's total.
    pub(crate) fn medicare_form(
        &self,
        id: i64,
        owner: usize,
        context: crate::IssueContext,
        amounts: &SaAmounts,
    ) -> Option<MedicareForm> {
        let Some(MedicalSaAccount::MedicareAdvantage) = self.calculation_account() else {
            return None;
        };
        let prior = self.medicare_prior.as_ref()?;
        matches!(
            self.effective_payment(),
            Some(MedicalSaPayment::Own { .. } | MedicalSaPayment::Prohibited { .. })
        )
        .then(|| MedicareForm {
            id,
            owner,
            context,
            taxable: amounts.taxable.clone(),
            penalized: amounts.penalized.clone(),
            prior: prior.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archer(gross: i64, payment: MedicalSaPayment) -> FederalMedicalDistribution {
        FederalMedicalDistribution {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross,
            box2_earnings: None,
            box4_value: None,
            box3_code: Some(1),
            account: MedicalSaAccount::Archer,
            medicare_prior: None,
            medicare_from_spouse: None,
            payment: Some(payment),
        }
    }

    #[test]
    fn an_hsa_form_accepts_owned_distributions_and_correction_sources() {
        let own = archer(
            100,
            MedicalSaPayment::Own {
                nonmedical: alloc::vec![MedicalAccountWithdrawal { date: 20260601, amount: 100 }],
                medical: 0,
                rollover: None,
                disabled_since: None,
            },
        );
        assert_eq!(own.rule(), None);
        let mut hsa = own.clone();
        hsa.account = MedicalSaAccount::Hsa;
        assert_eq!(hsa.rule(), None);
        hsa.box3_code = Some(2);
        hsa.box2_earnings = Some(0);
        hsa.payment = Some(MedicalSaPayment::ExcessReturned);
        assert_eq!(hsa.rule(), None);
    }
}
