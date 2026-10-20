// Coverage: Form 1099-SA with Form 8853 Sections A-B and Form 8889 Part II for a beneficiary
//   (unit B6a, 2026-10-09; the Archer MSA, Medicare Advantage MSA and inherited-account kinds
//   merged into this one document).
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
//   Entry: IncomeDocument::Sa1099 (Form 1099-SA, importable; box 3 code 1/3 Own, 2 ExcessReturned,
//   4/6 Inherited, 5 Prohibited) on the Other income page. Own HSA distributions use this
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
//   Unit20 checkpoint: actual receipt/deposit parts, same-owner HSA/Archer
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
use crate::federal_rules::{valid_birth, valid_date};
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact, whole};
use crate::{FederalPerson, FederalResult, MedicalRollover};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-SA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-SA (Distributions From an HSA, Archer MSA, or Medicare Advantage MSA) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read box 1 gross distribution; a blank box 1 is an error. For box 5 return Hsa, Archer or MedicareAdvantage for the checked box; report an error when none or more than one is checked. For box 3 return Own for code 1 or 3, ExcessReturned with the printed box 2 amount for code 2 (null when genuinely blank; never infer the signed custodian net-income adjustment), Inherited with the box 4 amount as value for code 4 or 6, and Prohibited for code 5; a blank box 3 is an error. Do not read the payer or recipient name, address, TIN or account number, and do not report payment dates, what the money paid for, a rollover, a disability date, the death date, who the beneficiary is or the account's prior-year value; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Sa1099 {
    /// Actual source document
    /// Choose a real Form1099-SA or the trustee's statement. Unreported account events use a trustee statement; do not fabricate numbered form boxes.
    #[libertas_default(Form1099Sa)]
    pub origin: HealthDocumentOrigin,
    /// Box 1 gross distribution
    /// Use one real statement. Report contributions and corrections on Health accounts; ordinary distributions are entered only here.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Account (box 5)
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub account: MedicalSaAccount,
    /// What the payment was (box 3 code)
    #[libertas_llm_doc_field]
    #[libertas_default(Own)]
    pub payment: MedicalSaPayment,
}

/// Actual medical-account source document
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthDocumentOrigin {
    /// Form1099-SA
    Form1099Sa,
    /// Trustee statement or documented account event
    TrusteeStatement,
}

/// Account (box 5)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalSaAccount {
    /// HSA (box 5 HSA)
    /// Includes your own account and an HSA inherited from your spouse. Other beneficiaries choose the inherited payment treatment.
    Hsa,
    /// Archer MSA (box 5 Archer MSA)
    Archer,
    /// Medicare Advantage MSA (box 5 MA MSA)
    MedicareAdvantage {
        /// Account at the end of 2025
        /// Only your own account; ask the trustee for the December 31, 2025 value.
        prior: MedicareMsaPrior,
    },
    /// Medicare Advantage MSA you inherited from your spouse (box 5 MA MSA)
    /// It is now your own and is taxed like an Archer MSA.
    MedicareFromSpouse,
}

/// What the payment was (box 3 code)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
        #[libertas_number(min = 0)]
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
    ExcessReturned {
        /// Printed box 2 earnings, if present
        /// Leave empty for a genuinely blank box. Actual HSA principal and signed custodian gains or losses belong in the linked Health-account correction; a blank printed box never means that signed adjustment was zero. An Archer statement without printed earnings needs the linked actual custodian correction.
        /// [NullText]
        /// Printed box 2 is blank
        /// [SetText]
        /// The printed box 2 earnings are
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        earnings: Option<i64>,
    },
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
        /// Box 4 value on the date of death
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        value: i64,
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
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Account (box 5)
    pub account: MedicalSaAccount,
    /// What the payment was (box 3 code)
    pub payment: MedicalSaPayment,
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
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        balance: i64,
        /// Annual HDHP deductible on January 1, 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    fn relief(&self) -> Exact {
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

// Worksheet operands without a required form destination may omit only their
// explanation, with this owning notice. Required form amounts use the fallible
// projection above and never this optional path.
pub(crate) fn explanation_output(
    value: &Exact,
    context: crate::IssueContext,
    r: &mut FederalResult,
) -> Option<i64> {
    let amount = exact::rounded(value);
    if amount.is_none() {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("MEDICAL_EXPLANATION_UNAVAILABLE", alloc::vec![]),
            context,
        );
    }
    amount
}

impl OutputFailure {
    pub(crate) fn report(self, r: &mut FederalResult) {
        crate::issue_localized!(
            r,
            &self.context,
            "Medical account output",
            "MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE",
            "An actual required medical-account output exceeds its native amount representation.",
        );
        r.state = crate::ResultState::Unsupported;
        crate::federal_rules::withhold_required_source_result(r);
    }
}

impl Sa1099 {
    pub(crate) fn record(&self) -> FederalMedicalDistribution {
        FederalMedicalDistribution {
            origin: self.origin,
            gross: self.gross,
            account: self.account.clone(),
            payment: self.payment.clone(),
        }
    }
}
impl FederalMedicalDistribution {
    pub(crate) fn page(&self) -> Sa1099 {
        Sa1099 {
            origin: self.origin,
            gross: self.gross,
            account: self.account.clone(),
            payment: self.payment.clone(),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.gross >= 0
            && match &self.account {
                MedicalSaAccount::MedicareAdvantage { prior } => prior.valid(),
                _ => true,
            }
            && match &self.payment {
                MedicalSaPayment::Own { nonmedical, medical, rollover, disabled_since } => {
                    nonmedical.iter().all(|w| {
                        valid_date(w.date)
                            && (crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE)
                                .contains(&w.date)
                            && (0..=self.gross).contains(&w.amount)
                    }) && (0..=self.gross).contains(medical)
                        && rollover.as_ref().is_none_or(MedicalRollover::valid)
                        && disabled_since.is_none_or(valid_birth)
                }
                MedicalSaPayment::ExcessReturned { earnings } => {
                    earnings.is_none_or(|v| (0..=self.gross).contains(&v))
                }
                MedicalSaPayment::Prohibited { occurred } => {
                    valid_date(*occurred) && occurred / 10000 == 2026
                }
                MedicalSaPayment::Unreported { event, disabled_since } => {
                    event.valid() && disabled_since.is_none_or(valid_birth)
                }
                MedicalSaPayment::Inherited { died, value, beneficiary } => {
                    valid_date(*died)
                        && (19970101..=20261231).contains(died)
                        && *value >= 0
                        && match beneficiary {
                            MedicalBeneficiary::Nonspouse { payments } => {
                                payments.iter().all(|p| p.valid())
                            }
                            MedicalBeneficiary::Estate => true,
                        }
                }
            }
    }
    // The document's validation rules, checked again on save with the same keys.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let own = matches!(self.payment, MedicalSaPayment::Own { .. });
        let archer = matches!(
            self.account,
            MedicalSaAccount::Hsa | MedicalSaAccount::Archer | MedicalSaAccount::MedicareFromSpouse
        );
        match &self.payment {
            MedicalSaPayment::Unreported { event, .. } => {
                if self.origin != HealthDocumentOrigin::TrusteeStatement
                    || !matches!(
                        self.account,
                        MedicalSaAccount::Hsa
                            | MedicalSaAccount::Archer
                            | MedicalSaAccount::MedicareFromSpouse
                    )
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
                if rollover.is_some() && !archer {
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
            MedicalSaPayment::ExcessReturned { earnings } => {
                if !matches!(self.account, MedicalSaAccount::Hsa | MedicalSaAccount::Archer) {
                    Some(ValidationIssue::new("SA1099_EXCESS_NOT_ARCHER").at_field("account"))
                } else if earnings.is_some_and(|v| v > self.gross) {
                    Some(
                        ValidationIssue::new("SA1099_EARNINGS_ABOVE_GROSS")
                            .at_field("earnings")
                            .in_field("payment"),
                    )
                } else {
                    None
                }
            }
            MedicalSaPayment::Inherited { .. }
                if matches!(self.account, MedicalSaAccount::MedicareFromSpouse) =>
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
        let medicare = matches!(self.account, MedicalSaAccount::MedicareAdvantage { .. });
        match &self.payment {
            MedicalSaPayment::Own { nonmedical, medical, rollover, disabled_since } => {
                if rollover.as_ref().is_some_and(|v| !v.eligible(&self.account)) {
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
                            !crate::federal_model::after_65th_birthday(person.birth_date, w.date)
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
                        !crate::federal_model::after_65th_birthday(person.birth_date, date)
                            && disabled_since.is_none_or(|d| date <= d)
                    }),
                    ..SaAmounts::default()
                })
            }
            MedicalSaPayment::ExcessReturned { earnings } => Ok(SaAmounts {
                // HSA income comes from the linked actual corrective NIA once.
                earnings: if matches!(self.account, MedicalSaAccount::Hsa) {
                    whole(0)
                } else {
                    whole(earnings.unwrap_or(0))
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
                    !crate::federal_model::after_65th_birthday(person.birth_date, 20260101)
                };
                Ok(SaAmounts {
                    taxable: whole(self.gross),
                    penalized: whole(if penalized { self.gross } else { 0 }),
                    ..SaAmounts::default()
                })
            }
            MedicalSaPayment::Inherited { died, value, beneficiary } => {
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
        match &self.payment {
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
            MedicalSaPayment::ExcessReturned { earnings } => {
                if matches!(self.account, MedicalSaAccount::Hsa) { None } else { *earnings }
            }
            MedicalSaPayment::Inherited { died, value, beneficiary } => {
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
        let MedicalSaAccount::MedicareAdvantage { prior } = &self.account else {
            return None;
        };
        matches!(self.payment, MedicalSaPayment::Own { .. } | MedicalSaPayment::Prohibited { .. })
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

// Section B per spouse (whole dollars): the taxable distributions, the part
// without the additional tax, the prior-balance relief and the 50% tax. The
// relief is the spouse's, taken once from the first form.
pub(crate) fn medicare_totals(
    forms: &[MedicareForm],
    r: &mut FederalResult,
) -> Result<([i64; 2], [i64; 2]), OutputFailure> {
    let mut total = [0; 2];
    let mut tax = [0; 2];
    for owner in 0..2 {
        let mine: Vec<&MedicareForm> = forms.iter().filter(|f| f.owner == owner).collect();
        let Some(first) = mine.first() else {
            continue;
        };
        crate::require_localized!(
            r,
            mine.iter().all(|f| f.prior == first.prior),
            &first.context,
            "1099-SA",
            "ISSUE_MEDICARE_MSA_PRIOR_DIFFERS",
            "Two Medicare Advantage MSA forms for the same person show different December 31, 2025 balances. Use the same account value on each.",
        );
        let taxable = exact::dollars_exact(&mine.iter().map(|f| &f.taxable).sum::<Exact>());
        let penalized = phase::cap(
            exact::dollars_exact(&mine.iter().map(|f| &f.penalized).sum::<Exact>()),
            [taxable.clone()],
        );
        let relief = first.prior.relief();
        let owner_tax =
            exact::rounded_exact(&(phase::floor(&penalized - &relief, whole(0)) / whole(2)));
        total[owner] = required_output(&taxable, first.context.clone())?;
        tax[owner] = required_output(&owner_tax, first.context.clone())?;
        let tax_cents = required_output(&(&owner_tax * whole(100)), first.context.clone())?;
        let mut explanation_unavailable = false;
        emit_lines(
            r,
            &mut explanation_unavailable,
            [
                ("exception", taxable - penalized),
                ("balance_relief", relief),
                ("additional_tax", owner_tax),
            ],
            |(suffix, amount), unavailable, _| {
                // These two worksheet explanations are not required form
                // amounts. Preserve the known income/tax if a displayed
                // explanation cannot fit; the actual tax row remains required.
                let cents = if suffix == "additional_tax" {
                    tax_cents
                } else if let Some(cents) = exact::rounded(&(amount * whole(100))) {
                    cents
                } else {
                    *unavailable = true;
                    return None;
                };
                Some(calculation_row(
                    None,
                    &alloc::format!("medicare_msa_{}_{suffix}", first.id),
                    LineValue::MoneyCents(cents),
                    &["income.medical_distribution"],
                    "https://www.irs.gov/instructions/i8853",
                    msg!("EXPLAIN_MEDICARE_MSA_N_N"),
                ))
            },
        );
        if explanation_unavailable {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("MEDICAL_EXPLANATION_UNAVAILABLE", alloc::vec![]),
                first.context.clone(),
            );
        }
    }
    Ok((total, tax))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archer(gross: i64, payment: MedicalSaPayment) -> FederalMedicalDistribution {
        FederalMedicalDistribution {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross,
            account: MedicalSaAccount::Archer,
            payment,
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
        hsa.payment = MedicalSaPayment::ExcessReturned { earnings: Some(0) };
        assert_eq!(hsa.rule(), None);
    }
}
