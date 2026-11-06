pub use crate::archer_source::{
    ArcherContributions, ArcherContributor, ArcherCorrection, ArcherCorrectionPayer,
    ArcherCoverage, ArcherEligibility, ArcherExcessAccounting, ArcherPriorEmployerExcessDeposit,
    ArcherPriorExcess, ArcherSharing,
};
use crate::validation::ValidationIssue;
// Coverage: Form5329 PRINT PartVI (2026-10-13): existing accepted excess
//   sources project lines34–41 per filer, including full room/distribution
//   subtotal37 and total40. Source cents, carry and rounded taxes are unchanged.
//   Evidence: tests/health/health_accounts.rs and corpus footings; IRS2026 draft f5329.
// Coverage: Form 8853 Part I (Archer MSA contributions) — Supported.
//   The calculation was audited 2026-09-24.
//   Lines: 1-5 the monthly limits of Rev. Proc. 2025-32 section 4.28 (65% of the self-only and 75%
//   of the family deductible, shared by spouses in the agreed shares), the compensation cap, line 3
//   the employer-funded deposits, Schedule 1 line 23 the deduction; the personal and employer
//   excess, its timely corrections, the prior-year employer excess deposited in 2026 and the
//   retained 2025 excess (Form 5329 Part VI lines 39-41, 6%).
//   Entry: the HSA page (Health and medical savings accounts, HsaPage.archer, ArcherContributionsPage): one row
//   for each owner with the coverage by month, the corrections, the excess accounting and the
//   compensation the plan covers chosen by identity through the catalog `$.archer_income`
//   (a List host of identities); the spouse's row stays on a separate return as the family-limit facts.
//   Rules both sides: ARCHER_OWNER_TWICE, ARCHER_CORRECTION_*, ARCHER_PRIOR_EXCESS_DEPOSIT_AFTER_
//   DEADLINE, ARCHER_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER, and the sums across the records,
//   ARCHER_CORRECTED_PERSONAL_ and _EMPLOYER_ABOVE_CONTRIBUTED and ARCHER_LATER_EMPLOYER_ and
//   _PERSONAL_ABOVE_NET; the compensation List host is checked on save (ARCHER_INCOME_NOT_OFFERED,
//   ARCHER_INCOME_TWICE) and the ranges Archer valid() repeats stay server-only
//   (ARCHER_RECORD_INVALID). Not calculated: a Medicare Advantage MSA's contributions (none are
//   allowed) and the distributions side (medical_account.rs).
//   The records, correction, prior employer excess and excess
//   accounting confirmations are gone; the printed dates and amounts decide.
//   Retained excess also follows Form 5329 line 35 when current employer deposits
//   exist: unused capacity subtracts both current personal and employer deposits.
//   The prior-year deduction remains distinct from barred current personal deposits.
//   Tests: tests/health/account_documents.rs archer_contributions_save_through_the_hsa_page_and_reopen,
//   archer_corrections_and_excess_accounting_save_and_reopen; tests/health/health_accounts.rs archer_*,
//   tests/health/retirement.rs archer_*. Sources: https://www.irs.gov/pub/irs-dft/f8853--dft.pdf and
//   i8853--dft.pdf (2026 drafts), https://www.irs.gov/pub/irs-dft/f5329--dft.pdf, Rev. Proc. 2025-32.
// Form 8853 Part I; 2026 plan limits: Rev. Proc. 2025-32 section 4.28.
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact, whole};
use crate::kernel::ratio::positive;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{line, line_cents};
use crate::medical_account::required_output;
use crate::medical_account_return::explanation_output;
use crate::{FederalDraft, FederalIncome, FederalResult, FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Timely Archer MSA excess-contribution return
/// Use the custodian's completed corrective distribution (Form 1099-SA code 2), not an ordinary withdrawal, and enter it once. Current contribution totals include 2026-designated original principal; 2025-designated principal belongs only on the retained prior return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ArcherCorrectionPage {
    /// Actual returned-excess statement, if entered on Other income
    /// Link the actual accepted source to avoid counting its earnings again. Leave empty for a correction whose statement is not entered separately.
    /// [NullText]
    /// This correction has no separate Other income statement
    /// [SetText]
    /// The returned-excess statement is
    #[libertas_enum_source("$.archer_return_documents")]
    pub document: Option<i64>,
    /// Original failed indirect-rollover deposit, if this return corrects one
    /// Choose the original Other-income statement, separately from the returned-excess statement above. The contribution date and year must match its actual failed cash deposit. Leave empty only for other manually reported contributions.
    /// [NullText]
    /// This correction is not for a failed indirect rollover
    /// [SetText]
    /// The original failed-rollover deposit is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.archer_failed_deposits")]
    pub failed_deposit: Option<i64>,
    /// Tax year for which the original contribution was designated
    // Tax structure: these corrections concern the prior or current return year (2025 or 2026).
    #[libertas_number(min = 2025, max = 2026)]
    pub year: u32,
    /// Date the original principal was actually deposited
    #[libertas_date_only]
    // Tax structure: a 2025 or 2026 designated contribution may be deposited in the following calendar year.
    #[libertas_number(min = 20250101, max = 20271231)]
    pub contributed_on: u32,
    /// Original excess principal returned
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Attributable net income returned
    /// A negative amount is a loss reducing cash returned, not the principal removed from contributions. Positive earnings enter income in the withdrawal year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Date the principal and all attributable net income were returned
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub returned: u32,
    /// Applicable return or IRS-relief correction deadline
    /// Use the documented due date including a valid extension or relief, with any timely-filing/amendment conditions already satisfied.
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub deadline: u32,
    /// Source of the corrected contribution
    pub payer: ArcherCorrectionPayer,
}

record_page_conversions!(ArcherCorrection, ArcherCorrectionPage, [&], |value, Target| {
    Target {
        document: value.document,
        failed_deposit: value.failed_deposit,
        year: value.year,
        contributed_on: value.contributed_on,
        principal: value.principal,
        earnings: value.earnings,
        returned: value.returned,
        deadline: value.deadline,
        payer: value.payer.clone(),
    }
});

impl ArcherCorrectionPage {
    pub(crate) fn record(&self) -> ArcherCorrection {
        self.into()
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.record().issue()
    }
}
impl ArcherCorrection {
    pub(crate) fn page(&self) -> ArcherCorrectionPage {
        self.into()
    }
}
impl ArcherCorrection {
    fn current_earnings_income(&self) -> i64 {
        if self.returned / 10000 == 2026 { positive(self.earnings) } else { 0 }
    }
    fn current_employer_principal_income(&self) -> i64 {
        if self.contributed_on / 10000 == 2026
            && matches!(self.payer, ArcherCorrectionPayer::Employer { in_wages: false })
        {
            self.principal
        } else {
            0
        }
    }
}

impl ArcherContributorRow {
    // The sums across the owner's records, each with the key of its page rule.
    fn cross_record_issue(&self) -> Option<ValidationIssue> {
        let corrections: Vec<_> =
            self.corrections.iter().map(ArcherCorrectionPage::record).collect();
        let personal_returned =
            ArcherCorrection::manual_returned_for_return_year(&corrections, false);
        let employer_returned =
            ArcherCorrection::manual_returned_for_return_year(&corrections, true);
        if personal_returned > whole(self.personal) {
            return Some(
                ValidationIssue::new("ARCHER_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED")
                    .at_field("personal"),
            );
        }
        if employer_returned > whole(self.employer) {
            return Some(
                ValidationIssue::new("ARCHER_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED")
                    .at_field("employer"),
            );
        }
        let e = self.excess_accounting.as_ref()?;
        if whole(e.later_employer_deposits) > whole(self.employer) - employer_returned {
            return Some(
                ValidationIssue::new("ARCHER_LATER_EMPLOYER_ABOVE_NET")
                    .at_field("later_employer_deposits")
                    .in_field("excess_accounting"),
            );
        }
        (whole(e.later_return_year_deposits) - whole(e.later_employer_deposits)
            > whole(self.personal) - personal_returned)
            .then(|| {
                ValidationIssue::new("ARCHER_LATER_PERSONAL_ABOVE_NET")
                    .at_field("later_return_year_deposits")
                    .in_field("excess_accounting")
            })
    }
}

/// Archer MSA contributions (Form 8853 Part I)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ArcherContributionsPage {
    /// No regular Archer MSA contributions or retained excess for either spouse
    None,
    /// Personal or employer contributions for 2026
    Records {
        /// Owners and coverage
        /// Include the spouse's coverage and contributions even if that spouse has no funded MSA or files separately. Use a zero-amount row for coverage-only information.
        // Tax structure: one contribution owner for each of the two spouses.
        #[libertas_size(min = 1, max = 2)]
        #[libertas_unordered]
        /// ----
        /// Owner
        owners: Vec<ArcherContributorRow>,
        /// Family-limit allocation
        #[libertas_default(Equal)]
        sharing: ArcherSharing,
    },
}

/// Annual Archer MSA owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ArcherContributorRow {
    /// Owner
    #[libertas_default(Taxpayer)]
    pub owner: Owner,
    /// Personal contributions
    /// Enter other original2026-designated principal outside the accepted Forms5498-SA before timely corrections. Exclude failed indirect-rollover deposits entered on Other income; the App adds those linked ordinary cash deposits once. Corrected principal is removed once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub personal: i64,
    /// Other employer contributions designated for 2026
    /// Only original principal outside both the entered W-2 sources and Forms5498-SA above, before timely corrections. Do not add allocated code R again. Actual next-calendar-year deposits designated for this return belong here; prior-year designations do not.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employer: i64,
    /// Timely returned excess contributions
    /// Record custodian-completed corrective distributions here, not as ordinary Archer MSA withdrawals. The 2026 contribution totals above include original principal designated for 2026, which the App removes once. A 2025-designated principal belongs on the retained 2025 Form 8853 instead.
    /// ----
    /// Correction
    pub corrections: Vec<ArcherCorrectionPage>,
    /// Uncorrected 2025-designated employer excess actually deposited in 2026
    /// Keep it out of the 2026 personal/employer contribution totals above. Its retained 2025 Form 8853 establishes the excess; actual 2026 W-2 wage reporting determines whether it enters 2026 other income.
    /// ----
    /// Deposit
    pub prior_employer_excess: Vec<ArcherPriorEmployerExcessDeposit>,
    /// Current uncorrected contribution excess for this owner
    /// Leave empty when net deposits after timely corrections fit the rules and no retained prior excess remains. This worksheet computes the deductible amount, taxable employer excess and Form 5329 tax.
    /// [NullText]
    /// No uncorrected Archer MSA excess remains
    /// [SetText]
    /// The uncorrected contribution excess is
    pub excess_accounting: Option<ArcherExcessAccounting>,
    /// Eligibility and coverage on each month's first day
    pub coverage: ArcherEligibility,
    /// Compensation the plan covers
    /// Choose this owner's W-2 or Form 8919 earned compensation or Schedule C or Schedule F business whose employer or business maintains the high deductible plan; no pension, deferred compensation or unrelated employer. The chosen W-2 box 1 amounts must equal qualifying earned compensation. Business net income and its share of deductible SE tax are calculated by the App. Leave empty only for a coverage-only row or a spouse filing outside this return.
    #[libertas_enum_source("$.archer_income")]
    pub income: Vec<i64>,
}
impl ArcherContributions {
    pub(crate) fn page(&self) -> ArcherContributionsPage {
        match self {
            Self::None => ArcherContributionsPage::None,
            Self::Records { owners, sharing } => ArcherContributionsPage::Records {
                owners: owners
                    .iter()
                    .map(|v| ArcherContributorRow {
                        owner: v.owner,
                        personal: v.personal,
                        employer: v.employer_outside,
                        corrections: v.corrections.iter().map(ArcherCorrection::page).collect(),
                        prior_employer_excess: v.prior_employer_excess.clone(),
                        excess_accounting: v.excess_accounting.clone(),
                        coverage: v.coverage.clone(),
                        income: v.income.clone(),
                    })
                    .collect(),
                sharing: *sharing,
            },
        }
    }
}
impl ArcherContributionsPage {
    pub(crate) fn record(&self) -> ArcherContributions {
        match self {
            Self::None => ArcherContributions::None,
            Self::Records { owners, sharing } => ArcherContributions::Records {
                owners: owners
                    .iter()
                    .map(|v| ArcherContributor {
                        owner: v.owner,
                        personal: v.personal,
                        personal_outside: v.personal,
                        employer: v.employer,
                        employer_outside: v.employer,
                        corrections: v
                            .corrections
                            .iter()
                            .map(ArcherCorrectionPage::record)
                            .collect(),
                        prior_employer_excess: v.prior_employer_excess.clone(),
                        excess_accounting: v.excess_accounting.clone(),
                        coverage: v.coverage.clone(),
                        income: v.income.clone(),
                    })
                    .collect(),
                sharing: *sharing,
            },
        }
    }
    // The first page rule a record breaks, in the order the rules are declared
    // (the client's rules on each type, repeated on save with the same keys).
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let Self::Records { owners, .. } = self else {
            return None;
        };
        if let Some((i, _)) =
            owners.iter().enumerate().find(|(i, v)| owners[..*i].iter().any(|o| o.owner == v.owner))
        {
            return Some(
                ValidationIssue::new("ARCHER_OWNER_TWICE").at_field("owner").in_list("owners", i),
            );
        }
        for (i, v) in owners.iter().enumerate() {
            if let Some(issue) = v
                .corrections
                .iter()
                .enumerate()
                .find_map(|(j, c)| c.issue().map(|issue| issue.in_list("corrections", j)))
            {
                return Some(issue.in_list("owners", i));
            }
            if let Some(issue) =
                v.prior_employer_excess.iter().enumerate().find_map(|(j, c)| {
                    c.issue().map(|issue| issue.in_list("prior_employer_excess", j))
                })
            {
                return Some(issue.in_list("owners", i));
            }
            if let Some(issue) = v.excess_accounting.as_ref().and_then(|e| e.issue()) {
                return Some(issue.in_field("excess_accounting").in_list("owners", i));
            }
            if let Some(issue) = v.cross_record_issue() {
                return Some(issue.in_list("owners", i));
            }
        }
        None
    }
}

// Source cents in/out. Limits are not rounded upward to permit excess deposits.
#[derive(Default)]
pub(crate) struct OwnerAmounts {
    pub deduction: [Exact; 2],   // native cents, before the one joint rounding
    pub excess_tax: [i64; 2],    // native final dollars
    pub income: [[Exact; 2]; 4], // employer, corrective earnings/principal, prior employer; cents
}

pub(crate) type Calculation = (Exact, i64, Exact, Exact, Exact, Exact, OwnerAmounts);

#[derive(Debug)]
pub(crate) enum CalculationIssue {
    Section179(crate::Section179PreparedIssue),
    Output(crate::medical_account::OutputFailure),
}
impl From<crate::Section179PreparedIssue> for CalculationIssue {
    fn from(value: crate::Section179PreparedIssue) -> Self {
        Self::Section179(value)
    }
}
impl From<crate::medical_account::OutputFailure> for CalculationIssue {
    fn from(value: crate::medical_account::OutputFailure) -> Self {
        Self::Output(value)
    }
}

// Forward the existing invocation operands and its borrowed home context together.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_with_section179(
    d: &FederalDraft,
    half: [i64; 2],
    optional_weights: &alloc::collections::BTreeMap<i64, i64>,
    taxable_withdrawals: [Exact; 2],
    selected: Option<&crate::Section179PreparedAllocation>,
    failed: &[crate::medical_rollover::Contribution; 2],
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<Calculation, CalculationIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_with_section179_with_claims(
        d,
        half,
        optional_weights,
        taxable_withdrawals,
        selected,
        failed,
        r,
        home,
        &source_claims,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_with_section179_with_claims(
    d: &FederalDraft,
    half: [i64; 2],
    optional_weights: &alloc::collections::BTreeMap<i64, i64>,
    taxable_withdrawals: [Exact; 2],
    selected: Option<&crate::Section179PreparedAllocation>,
    failed: &[crate::medical_rollover::Contribution; 2],
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Calculation, CalculationIssue> {
    let statement_totals = crate::contribution_documents::project(d).unwrap_or_default();
    let corrections = d.adjustments.as_ref().map_or(&ArcherContributions::None, |v| &v.archer);
    if let Some(key) = crate::hsa_correction::archer_source_rule(corrections, &d.income) {
        crate::require_localized!(
            r,
            false,
            &crate::IssueContext::Section(crate::TaxSection::Hsa)
                .alternatives(&[crate::TaxSection::Hsa, crate::TaxSection::OtherIncome]),
            "Archer corrective statement",
            key,
            "Reconcile this Archer returned-excess statement with its actual correction facts or printed earnings.",
        );
    }
    let Some(ArcherContributions::Records { owners, sharing }) =
        d.adjustments.as_ref().map(|v| &v.archer)
    else {
        return Ok((whole(0), 0, whole(0), whole(0), whole(0), whole(0), OwnerAmounts::default()));
    };
    let status = d.setup.as_ref().unwrap().filing_status();
    let married = d.setup.as_ref().unwrap().marital_state.married();
    let accounts =
        [Owner::Taxpayer, Owner::Spouse].map(|owner| owners.iter().find(|v| v.owner == owner));
    crate::require_localized!(
        r,
        owners.iter().all(|v| owners.iter().filter(|o| o.owner == v.owner).count() == 1)
            && (married || accounts[1].is_none()),
        &crate::IssueContext::Section(crate::TaxSection::Hsa),
        "Archer MSA",
        "ISSUE_USE_ONE_ARCHER_CONTRIBUTION_RECORD_PER_OWNER_WITH_A_SPOUSE_ROW_ONLY_89036B4A",
        "Use one Archer contribution record per owner, with a spouse row only when married.",
    );
    let share = match sharing {
        ArcherSharing::Equal => 5000,
        ArcherSharing::Agreed { taxpayer } => crate::kernel::natural::units(*taxpayer, 100),
    };
    let mut numerators = [0_i128; 2];
    let mut shared_family = false;
    for m in 0..12 {
        let coverage = accounts.map(|v| v.map_or(ArcherCoverage::None, |a| a.coverage.month(m)));
        crate::require_localized!(
            r,
            coverage.iter().all(|v| v.qualified()),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "Archer MSA",
            "ISSUE_AN_ELIGIBLE_2026_ARCHER_HDHP_MUST_MEET_THE_IRS_DEDUCTIBLE_AND_OUT_O_8BE1BDA9",
            "An eligible 2026 Archer HDHP must meet the IRS deductible and out-of-pocket limits; otherwise mark the month ineligible.",
        );
        let family = coverage.iter().filter_map(|v| v.family()).min();
        let both = coverage.iter().all(|v| *v != ArcherCoverage::None);
        shared_family |= both && family.is_some();
        crate::require_localized!(
            r,
            !married || family.is_none() || accounts.iter().all(|v| v.is_some()),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "Archer MSA",
            "ISSUE_SUPPLY_BOTH_SPOUSES_COVERAGE_RECORDS_INCLUDING_A_ZERO_CONTRIBUTION_B30F0C5A",
            "Supply both spouses' coverage records, including a zero-contribution row, to determine the family limit.",
        );
        for i in 0..2 {
            if coverage[i] == ArcherCoverage::None {
                continue;
            }
            let person = d
                .people
                .as_ref()
                .and_then(|p| if i == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
            crate::require_localized!(
                r,
                person.is_none_or(|p| {
                    !p.dependency.claimable()
                        && p.death_date().is_none_or(|date| 20260101 + m as u32 * 100 <= date)
                }),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA",
                "ISSUE_ELIGIBLE_ARCHER_MSA_MONTHS_CANNOT_FOLLOW_DEATH_OR_BELONG_TO_A_DEPEN_1C818CF3",
                "Eligible Archer MSA months cannot follow death or belong to a dependent.",
            );
            numerators[i] += if let Some(deductible) = family {
                i128::from(deductible)
                    * 75
                    * if both || status == FederalStatus::Separate {
                        // Section 220(b)(5): a separate filer with family
                        // coverage takes half the limit (or the agreed share)
                        // whether or not the spouse is also eligible.
                        if i == 0 { i128::from(share) } else { i128::from(10000 - share) }
                    } else {
                        10000
                    }
            } else if let ArcherCoverage::SelfOnly { deductible, .. } = coverage[i] {
                i128::from(deductible) * 65 * 10000
            } else {
                0
            };
        }
    }
    let total_employer: Exact = owners.iter().map(ArcherContributor::net_employer).sum();
    let mut sources = OwnerAmounts::default();
    let mut deduction = whole(0);
    let mut excess_tax = 0;
    let mut employer_other_income = whole(0);
    let mut correction_earnings = whole(0);
    let mut correction_employer_principal = whole(0);
    let mut prior_employer_income = whole(0);
    let mut accounted_owners = 0;
    let mut prior_owners = 0;
    let mut prior_used_total = whole(0);
    let mut prior_deducted_total = whole(0);
    let mut prior_remaining_total = whole(0);
    let mut personal_excess_total = whole(0);
    let mut employer_excess_total = whole(0);
    let mut excess_account_value_total = whole(0);
    for (i, record) in accounts.iter().enumerate() {
        let Some(v) = record else { continue };
        let returned = i == 0 || status == FederalStatus::Joint;
        let net_personal = v.net_personal() + &failed[i].personal;
        let output_context = crate::IssueContext::owner(crate::TaxSection::Hsa, v.owner);
        let net_employer = v.net_employer();
        let prior_designated_total: Exact =
            v.prior_employer_excess.iter().map(|e| whole(e.amount)).sum();
        if let Some(prior) = v.excess_accounting.as_ref().and_then(|e| e.prior_year.as_ref()) {
            crate::require_localized!(
                r,
                prior_designated_total <= whole(prior.line_40),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer prior employer excess",
                "ISSUE_UNCORRECTED_2025_DESIGNATED_EMPLOYER_EXCESS_DEPOSITED_IN_2026_CANNO_CD748793",
                "Uncorrected 2025-designated employer excess deposited in 2026 cannot exceed retained 2025 Form 5329 line 40 excess.",
            );
        }
        for deposit in &v.prior_employer_excess {
            crate::require_localized!(
                r,
                d.people
                    .as_ref()
                    .and_then(|p| if i == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() })
                    .is_none_or(|p| {
                        p.death_date().is_none_or(|date| deposit.contributed_on <= date)
                    }),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer prior employer excess",
                "ISSUE_POST_DEATH_EMPLOYER_DEPOSITS_NEED_BENEFICIARY_OR_ESTATE_TREATMENT_559CB33D",
                "Post-death employer deposits need beneficiary or estate treatment.",
            );

            if returned && !deposit.in_wages {
                prior_employer_income += whole(deposit.amount);
                sources.income[3][i] += whole(deposit.amount);
            }
        }
        if returned && prior_designated_total > whole(0) {
            exact_explanation(
                r,
                &alloc::format!("archer_{}_prior_designated_employer_excess", i + 1),
                prior_designated_total.clone(),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS"),
            );
        }
        for correction in &v.corrections {
            crate::require_localized!(
                r,
                correction.eligible(),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA correction",
                "ISSUE_CONFIRM_A_COMPLETE_TIMELY_ARCHER_MSA_CORRECTION_ITS_CONTRIBUTION_YE_68B7F8FA",
                "Confirm a complete timely Archer MSA correction, its contribution year, deadline and known employer wage reporting. Late or incomplete corrections need separate treatment.",
            );
            crate::require_localized!(
                r,
                d.people
                    .as_ref()
                    .and_then(|p| if i == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() })
                    .is_none_or(|p| p.death_date().is_none_or(|date| correction.returned <= date)),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA correction",
                "ISSUE_POST_DEATH_CORRECTIVE_DISTRIBUTIONS_REQUIRE_THE_BENEFICIARY_S_ACCOU_54757DF0",
                "Post-death corrective distributions require the beneficiary's account or inheritance treatment.",
            );
            if returned {
                let earnings = correction.current_earnings_income();
                correction_earnings += whole(earnings);
                sources.income[1][i] += whole(earnings);
                let principal = correction.current_employer_principal_income();
                correction_employer_principal += whole(principal);
                sources.income[2][i] += whole(principal);
            }
        }
        if returned && v.corrected(false) + v.corrected(true) > whole(0) {
            exact_explanation(
                r,
                &alloc::format!("archer_{}_corrected_contributions", i + 1),
                v.corrected(false) + v.corrected(true),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_CORRECTED_CONTRIBUTIONS"),
            );
        }
        if returned {
            let prior_corrected: Exact =
                v.corrections.iter().filter(|c| c.year == 2025).map(|c| whole(c.principal)).sum();
            if prior_corrected > whole(0) {
                exact_explanation(
                    r,
                    &alloc::format!("archer_{}_prior_designated_corrected_contributions", i + 1),
                    prior_corrected.clone(),
                    &["adjustments.archer"],
                    "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
                    msg!("EXPLAIN_ARCHER_N_PRIOR_DESIGNATED_CORRECTED_CONTRIBUTIONS"),
                );
            }
        }
        let mut compensation = whole(0);
        let mut business = whole(0);
        let mut business_positive = whole(0);
        let mut has_business = false;
        // Allocate the actual half-SE deduction by attributable SE earnings,
        // while the MSA compensation base remains actual business profit.
        let mut all_profit = whole(0);
        for e in d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(v.owner)) {
            if let FederalIncome::Business { data } = &e.income {
                all_profit += match optional_weights.get(&e.id) {
                    Some(amount) => whole(*amount) * whole(100),
                    None => phase::floor(
                        whole(data.earned_receipts())
                            - whole(
                                crate::section179_return::expenses_with_claims(
                                    d,
                                    e.id,
                                    data,
                                    selected,
                                    home,
                                    source_claims,
                                )
                                .map(|expense| {
                                    expense
                                        - selected.map_or(0, |allocation| {
                                            allocation.ordinary_limit_adjustment(e.id)
                                        })
                                })?,
                            ),
                        whole(0),
                    ),
                };
            }
        }
        if returned {
            crate::require_localized!(
                r,
                &net_personal + &net_employer == whole(0) || !v.income.is_empty(),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA",
                "ISSUE_LINK_THE_EMPLOYER_OR_BUSINESS_MAINTAINING_THE_HDHP_TO_CALCULATE_THE_9968DAAC",
                "Link the employer or business maintaining the HDHP to calculate the compensation limit.",
            );
            for (n, id) in v.income.iter().enumerate() {
                crate::require_localized!(
                    r,
                    !v.income[..n].contains(id),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa),
                    "Archer MSA",
                    "ISSUE_DO_NOT_COUNT_ONE_COMPENSATION_SOURCE_TWICE_D840887D",
                    "Do not count one compensation source twice.",
                );
                match d
                    .income
                    .iter()
                    .find(|e| e.id == *id && e.owner == crate::DocumentOwner::from(v.owner))
                    .map(|e| (&e.income, e.id))
                {
                    Some((FederalIncome::Wage { data }, _)) => {
                        match crate::FederalWage::from(data).total_income() {
                            Ok(amount) => compensation += whole(amount),
                            Err(_message) => {
                                #[cfg(test)]
                                let message = _message;
                                crate::require_localized!(
                                    r,
                                    false,
                                    &crate::IssueContext::Section(crate::TaxSection::Hsa),
                                    "Archer MSA compensation",
                                    "ISSUE_EMPLOYEE_TIPS_RECORDS",
                                    message,
                                )
                            }
                        }
                    }
                    Some((FederalIncome::EmployeeWages { data }, _)) => match data.income() {
                        Ok(amount) => compensation += whole(amount),
                        Err(_message) => {
                            #[cfg(test)]
                            let message = _message;
                            crate::require_localized!(
                                r,
                                false,
                                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                                "Archer MSA compensation",
                                "ISSUE_EMPLOYEE_WAGES_RECORDS",
                                message,
                            )
                        }
                    },
                    Some((FederalIncome::Business { data }, id)) => {
                        has_business = true;
                        let profit = whole(data.earned_receipts())
                            - whole(
                                crate::section179_return::expenses_with_claims(
                                    d,
                                    id,
                                    data,
                                    selected,
                                    home,
                                    source_claims,
                                )
                                .map(|expense| {
                                    expense
                                        - selected.map_or(0, |allocation| {
                                            allocation.ordinary_limit_adjustment(id)
                                        })
                                })?,
                            );
                        business += &profit;
                        business_positive += optional_weights.get(&id).map_or_else(
                            || phase::floor(profit, whole(0)),
                            |v| whole(*v) * whole(100),
                        );
                    }
                    _ => {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::Hsa),
                            "Archer MSA",
                            "ISSUE_ARCHER_COMPENSATION_MUST_LINK_THIS_OWNER_S_W_2_FORM_8919_WAGES_OR_A_9C3E430C",
                            "Archer compensation must link this owner's W-2, Form 8919 wages or a Schedule C or Schedule F business maintaining the HDHP.",
                        );
                    }
                }
            }
            crate::require_localized!(
                r,
                net_employer == whole(0) || !has_business,
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA",
                "ISSUE_EMPLOYER_DEPOSITS_MUST_BE_SUPPORTED_BY_COMPENSATION_FROM_THE_CONTRI_D2405C7C",
                "Employer deposits must be supported by compensation from the contributing employer, not by an unrelated self-employed business.",
            );
            compensation += phase::floor(
                business
                    - if all_profit > whole(0) {
                        // Preserve the existing cent-rounded proportional half-SE stage.
                        exact::rounded_exact(
                            &(whole(half[i]) * whole(100) * &business_positive / &all_profit),
                        )
                    } else {
                        whole(0)
                    },
                whole(0),
            );
        }
        // Twelve months × i64 source deductible × 75 × u32 share fits i128.
        // Preserve the owner's existing truncation of the monthly coverage numerator.
        let cap = Exact::from_integer((numerators[i] / 12000000).into());
        let barred = net_employer > whole(0) || (shared_family && total_employer > whole(0));
        let deductible = phase::floor(phase::cap(cap.clone(), [compensation.clone()]), whole(0));
        crate::require_localized!(
            r,
            !returned
                || (v.corrected(false)
                    <= (if barred {
                        whole(v.personal) + &failed[i].personal
                    } else {
                        phase::floor(
                            whole(v.personal) + &failed[i].personal - &deductible,
                            whole(0),
                        )
                    })
                    && v.corrected(true)
                        <= phase::floor(whole(v.employer) - &deductible, whole(0))),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "Archer MSA correction",
            "ISSUE_TIMELY_RETURNED_PERSONAL_AND_EMPLOYER_PRINCIPAL_CANNOT_EXCEED_THEIR_5DBC74C3",
            "Timely returned personal and employer principal cannot exceed their respective original excess above the Form 8853 deduction or employer limit.",
        );
        let personal_excess = if barred {
            net_personal.clone()
        } else {
            phase::floor(&net_personal - &deductible, whole(0))
        };
        let employer_excess = phase::floor(&net_employer - &deductible, whole(0));
        let mut prior_deducted = whole(0);
        if let Some(excess) = &v.excess_accounting {
            accounted_owners += 1;
            let prior = excess.prior_year.as_ref().map_or(0, |v| v.line_40);
            let mut prior_remaining = whole(prior);
            let prior_used_by_distributions =
                phase::cap(taxable_withdrawals[i].clone(), [prior_remaining.clone()]);
            prior_remaining -= &prior_used_by_distributions;
            prior_deducted = phase::cap(
                phase::floor(&deductible - &net_personal - &net_employer, whole(0)),
                [prior_remaining.clone()],
            );
            prior_remaining -= &prior_deducted;
            let current_employer_excess = phase::floor(
                &net_employer
                    - whole(excess.later_employer_deposits)
                    - whole(statement_totals.archer[i].later_employer)
                    - &deductible,
                whole(0),
            );
            crate::require_localized!(
                r,
                returned
                    && (&personal_excess + &employer_excess > whole(0) || prior > 0)
                    && whole(excess.employer_excess_in_wages) <= current_employer_excess,
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA excess",
                "ISSUE_THE_SUPPORTED_OWNER_SPECIFIC_FORM_5329_PART_VI_PATH_REQUIRES_CURREN_2A3F1B47",
                "The owner-specific Form 5329 Part VI path requires current uncorrected excess or retained positive-tax 2025 excess, reconciled W-2 wages and complete year-end account value.",
            );
            let account_value = whole(excess.year_end_value)
                + whole(statement_totals.archer[i].value)
                + whole(statement_totals.archer[i].later)
                + whole(excess.later_return_year_deposits)
                + &failed[i].later;
            let capped_excess = phase::cap(
                &personal_excess + &employer_excess + &prior_remaining,
                [account_value.clone()],
            );
            let owner_tax = required_output(
                &exact::rounded_exact(
                    &(exact::dollars_exact(&capped_excess) * whole(6) / whole(100)),
                ),
                output_context.clone(),
            )?;
            required_output(&(whole(owner_tax) * whole(100)), output_context.clone())?;
            excess_tax += owner_tax;
            sources.excess_tax[i] += owner_tax;
            let employer_income = &current_employer_excess - whole(excess.employer_excess_in_wages);
            employer_other_income += &employer_income;
            sources.income[0][i] += &employer_income;
            // Form5329 PartVI projects the same source-cents reduction owner.
            // A carry consumes only the needed portion; the printed subtotal
            // still shows the full unused room and taxable distributions.
            let unused_room = if prior > 0 {
                phase::floor(&deductible - &net_personal - &net_employer, whole(0))
            } else {
                whole(0)
            };
            let distributions = if prior > 0 { taxable_withdrawals[i].clone() } else { whole(0) };
            emit_lines(
                r,
                &mut (),
                [
                    (34, LineValue::MoneyCents(prior)),
                    (
                        35,
                        LineValue::MoneyCents(required_output(
                            &unused_room,
                            output_context.clone(),
                        )?),
                    ),
                    (
                        36,
                        LineValue::MoneyCents(required_output(
                            &distributions,
                            output_context.clone(),
                        )?),
                    ),
                    (
                        37,
                        LineValue::MoneyCents(required_output(
                            &(&unused_room + &distributions),
                            output_context.clone(),
                        )?),
                    ),
                    (
                        38,
                        LineValue::MoneyCents(required_output(
                            &prior_remaining,
                            output_context.clone(),
                        )?),
                    ),
                    (
                        39,
                        LineValue::MoneyCents(required_output(
                            &(&personal_excess + &employer_excess),
                            output_context.clone(),
                        )?),
                    ),
                    (
                        40,
                        LineValue::MoneyCents(required_output(
                            &(&prior_remaining + &personal_excess + &employer_excess),
                            output_context.clone(),
                        )?),
                    ),
                    (41, LineValue::MoneyDollars(owner_tax)),
                ],
                |(number, value), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(
                            crate::TaxForm::Form5329,
                            &alloc::format!("{number}"),
                        )),
                        &alloc::format!("archer_{}_excess_line{number}", i + 1),
                        value,
                        &["adjustments.archer", "income.archer_msa"],
                        "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                        msg!("EXPLAIN_HEALTH_ACCOUNT_EXCESS_FORM"),
                    ))
                },
            );
            personal_excess_total += &personal_excess;
            employer_excess_total += &employer_excess;
            excess_account_value_total += &account_value;
            line(
                r,
                &alloc::format!("archer_{}_excess_contribution_tax", i + 1),
                owner_tax,
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_EXCESS_CONTRIBUTION_TAX"),
            );
            if prior > 0 {
                prior_owners += 1;
                prior_used_total += &prior_used_by_distributions;
                prior_deducted_total += &prior_deducted;
                prior_remaining_total += &prior_remaining;
                let rows = [
                    (
                        "used_by_distributions",
                        prior_used_by_distributions.clone(),
                        msg!("EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS"),
                    ),
                    (
                        "deducted_return_year",
                        prior_deducted.clone(),
                        msg!("EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_DEDUCTED_N"),
                    ),
                    (
                        "remaining_following_year",
                        prior_remaining.clone(),
                        msg!("EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_REMAINING_N"),
                    ),
                ]
                .into_iter()
                .filter_map(|(suffix, amount, explanation)| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("archer_{}_prior_excess_{suffix}", i + 1),
                        LineValue::MoneyCents(explanation_output(
                            &amount,
                            crate::IssueContext::Section(crate::TaxSection::Hsa),
                            r,
                        )?),
                        &["adjustments.archer", "income.archer_msa"],
                        "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                        explanation,
                    ))
                })
                .collect::<Vec<_>>();
                emit_lines(r, &mut (), rows, |row, _, _| Some(row));
            }
            exact_explanation(
                r,
                &alloc::format!("archer_{}_return_year_uncorrected_personal_excess", i + 1),
                personal_excess.clone(),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_N_UNCORRECTED_PERSONAL_EXCESS"),
            );
            exact_explanation(
                r,
                &alloc::format!("archer_{}_return_year_uncorrected_employer_excess", i + 1),
                employer_excess.clone(),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS"),
            );
            exact_explanation(
                r,
                &alloc::format!(
                    "archer_{}_return_year_uncorrected_employer_excess_other_income",
                    i + 1
                ),
                &current_employer_excess - whole(excess.employer_excess_in_wages),
                &["adjustments.archer", "income"],
                "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME"),
            );
            exact_explanation(
                r,
                &alloc::format!("archer_{}_return_year_excess_account_value", i + 1),
                account_value.clone(),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                msg!("EXPLAIN_ARCHER_N_N_EXCESS_ACCOUNT_VALUE"),
            );
        } else {
            crate::require_localized!(
                r,
                (!barred || net_personal == whole(0))
                    && &net_personal + &net_employer <= cap
                    && (!returned || &net_personal + &net_employer <= compensation),
                &crate::IssueContext::Section(crate::TaxSection::Hsa),
                "Archer MSA",
                "ISSUE_ARCHER_MSA_CONTRIBUTIONS_EXCEED_COVERAGE_ALLOCATION_OR_LINKED_COMPE_C7A07770",
                "Archer MSA contributions exceed coverage/allocation or linked compensation limits, or employer deposits bar personal deductions; excess treatment is required.",
            );
        }
        if returned {
            let allowed_personal = if barred {
                whole(0)
            } else {
                net_personal.clone()
                    - if v.excess_accounting.is_some() { personal_excess.clone() } else { whole(0) }
            } + &prior_deducted;
            deduction += &allowed_personal;
            sources.deduction[i] += &allowed_personal;
            let rows = [
                ("coverage_limit", cap.clone()),
                ("compensation", compensation.clone()),
                ("deduction", allowed_personal.clone()),
            ]
            .into_iter()
            .filter_map(|(suffix, amount)| {
                Some(calculation_row(
                    None,
                    &alloc::format!("archer_contribution_{}_{suffix}", i + 1),
                    LineValue::MoneyCents(explanation_output(
                        &amount,
                        crate::IssueContext::Section(crate::TaxSection::Hsa),
                        r,
                    )?),
                    &["adjustments.archer", "income", "self_employment_tax"],
                    "https://www.irs.gov/instructions/i8853",
                    msg!("EXPLAIN_ARCHER_CONTRIBUTION_N_N"),
                ))
            })
            .collect::<Vec<_>>();
            emit_lines(r, &mut (), rows, |row, _, _| Some(row));
        }
    }
    if prior_owners > 0 {
        let rows = [
            (
                "used_by_distributions",
                prior_used_total.clone(),
                msg!("EXPLAIN_ARCHER_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS"),
            ),
            (
                "deducted_return_year",
                prior_deducted_total.clone(),
                msg!("EXPLAIN_ARCHER_PRIOR_EXCESS_N_DEDUCTED_N"),
            ),
            (
                "remaining_following_year",
                prior_remaining_total.clone(),
                msg!("EXPLAIN_ARCHER_PRIOR_EXCESS_N_REMAINING_N"),
            ),
        ]
        .into_iter()
        .filter_map(|(suffix, amount, explanation)| {
            Some(calculation_row(
                None,
                &alloc::format!("archer_prior_excess_{suffix}"),
                LineValue::MoneyCents(explanation_output(
                    &amount,
                    crate::IssueContext::Section(crate::TaxSection::Hsa),
                    r,
                )?),
                &["adjustments.archer", "income.archer_msa"],
                "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                explanation,
            ))
        })
        .collect::<Vec<_>>();
        emit_lines(r, &mut (), rows, |row, _, _| Some(row));
    }
    if accounted_owners > 0 {
        let rows = [
            (
                "archer_return_year_uncorrected_personal_excess",
                personal_excess_total.clone(),
                msg!("EXPLAIN_ARCHER_N_UNCORRECTED_PERSONAL_EXCESS"),
            ),
            (
                "archer_return_year_uncorrected_employer_excess",
                employer_excess_total.clone(),
                msg!("EXPLAIN_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS"),
            ),
            (
                "archer_return_year_uncorrected_employer_excess_other_income",
                employer_other_income.clone(),
                msg!("EXPLAIN_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME"),
            ),
            (
                "archer_return_year_excess_account_value",
                excess_account_value_total.clone(),
                msg!("EXPLAIN_ARCHER_N_EXCESS_ACCOUNT_VALUE"),
            ),
        ]
        .into_iter()
        .filter_map(|(id, amount, explanation)| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyCents(explanation_output(
                    &amount,
                    crate::IssueContext::Section(crate::TaxSection::Hsa),
                    r,
                )?),
                &["adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                explanation,
            ))
        })
        .collect::<Vec<_>>();
        emit_lines(r, &mut (), rows, |row, _, _| Some(row));
    }
    exact_explanation(
        r,
        "archer_msa_deduction",
        deduction.clone(),
        &["adjustments.archer"],
        "https://www.irs.gov/instructions/i8853",
        msg!("EXPLAIN_ARCHER_MSA_DEDUCTION"),
    );
    required_output(
        &(whole(excess_tax) * whole(100)),
        crate::IssueContext::Section(crate::TaxSection::Hsa),
    )?;
    line(
        r,
        "archer_return_year_excess_contribution_tax",
        excess_tax,
        &["adjustments.archer"],
        "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
        msg!("EXPLAIN_ARCHER_N_EXCESS_CONTRIBUTION_TAX_2"),
    );
    exact_explanation(
        r,
        "archer_correction_earnings",
        correction_earnings.clone(),
        &["adjustments.archer"],
        "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
        msg!("EXPLAIN_ARCHER_CORRECTION_EARNINGS"),
    );
    exact_explanation(
        r,
        "archer_correction_employer_principal_other_income",
        correction_employer_principal.clone(),
        &["adjustments.archer"],
        "https://www.irs.gov/pub/irs-dft/i8853--dft.pdf",
        msg!("EXPLAIN_ARCHER_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME"),
    );
    exact_explanation(
        r,
        "archer_prior_designated_employer_excess_other_income",
        prior_employer_income.clone(),
        &["adjustments.archer"],
        "https://www.irs.gov/instructions/iw2w3",
        msg!("EXPLAIN_ARCHER_PRIOR_DESIGNATED_EMPLOYER_EXCESS_OTHER_INCOME"),
    );
    Ok((
        deduction.clone(),
        excess_tax,
        employer_other_income.clone(),
        correction_earnings.clone(),
        correction_employer_principal.clone(),
        prior_employer_income.clone(),
        sources,
    ))
}

fn exact_explanation(
    r: &mut FederalResult,
    id: &str,
    value: Exact,
    contexts: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    if let Some(amount) =
        explanation_output(&value, crate::IssueContext::Section(crate::TaxSection::Hsa), r)
    {
        line_cents(r, id, amount, contexts, source, explanation);
    }
}
