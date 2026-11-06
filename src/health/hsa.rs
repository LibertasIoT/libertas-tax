pub use crate::hsa_source::{
    HsaAllocationMethod, HsaEligibilityLoss, HsaExcessAccounting, HsaFamilyShare,
    HsaFundingHistory, HsaLateExcessWithdrawal, HsaLossReason, HsaPriorEmployerExcessDeposit,
    HsaPriorExcess,
};
// Coverage: Form5329 PRINT PartVII (2026-10-13): accepted excess-accounting
//   sources project lines42–49 per filer. Full room/distributions, carry and
//   current excess retain source cents; tax retains its existing whole-dollar
//   rounding. Corpus footings preserve account-value caps and both owner taxes.
//   Source: IRS2026 draft f5329; tests/health/health_accounts.rs.
// Coverage: Form 8889 / 1099-SA — Supported (audited 2026-09-24).
//   Lines: Part I: monthly eligibility, self/family limits, age-55 catch-up, joint/separate family
//   allocation, last-month rule and 2026/2027 testing period, IRA-to-HSA funding and recapture,
//   Archer reduction, timely corrections; Part II: medical/nonmedical distributions, 20% tax with
//   age-65/disability/death exceptions, deemed distributions, rollovers, late prior-excess
//   withdrawals; Part III: prior last-month recapture; Form 5329 Part VII 6% excise. Form 8889
//   Current on-screen Parts I/II retain owner contribution traces and classified cents with separate rounded estimate
//   transfer rows; linked code2 gross and unlinked completed returns enter14a/b once.
//   Loan pledges remain direct income, distinct from terminated-account14a values.
//   Tests: hsa_distribution_detail_*, hsa_code2_actual_statement_links_signed_correction_once.
//   Historical last-month limits retain monthly fractions through recapture
//   subtraction; Pub969(2025) p6 Examples1/2 keep7837.50/3541.67 in work
//   income, with separate whole-dollar PartIII rows. Tests: interview
//   published_hsa_last_month_examples_keep_recapture_cents_through_public_page.
//   Ordinary/deemed distributions and beneficiary Part II sources are the canonical
//   1099-SA/trustee statement document (medical_account.rs), not duplicate HSA account fields. Refusals are explicit
//   issues (unknown facts, mismatched histories).
//   Every stored FederalHsa field has a page field. The HSA page (HsaPage)
//   gains, on each account, the timely corrections (Form 8889 lines 2, 9, 14a and the 1099-SA code
//   2), late withdrawals of an old excess, the prior-year employer excess deposited in 2026, the
//   excess accounting (Form 5329 Part VII), the last-month rule's first loss and its
//   2025 history (line 18, Notice 2008-52) and
//   the 2025 IRA-to-HSA transfers (line 19); the 2026 IRA-to-HSA transfers are listed once on the
//   page, each choosing its Form 1099-R through the catalog `$.ira_sources` (HsaFundingRow). The
//   former "something else" answer (special, fully_eligible) is gone. Rules both sides:
//   HSA_CORRECTION_*, HSA_LATE_EXCESS_NOT_LATE, HSA_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE,
//   HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER, HSA_ROLLOVER_*, HSA_FUNDING_*, and the relationships
//   across records, each with its own key (platform item 13/14): HSA_CORRECTED_PERSONAL_ and
//   _EMPLOYER_ABOVE_CONTRIBUTED and HSA_LATER_EMPLOYER_ and _PERSONAL_ABOVE_NET (sum over the
//   corrections), HSA_FUNDING_HISTORY_SAME_MONTH (is_unique_set on year and month) and
//   HSA_LAST_MONTH_DECEMBER_NEEDED. Server-only: the ranges and shapes FederalHsa::valid
//   repeats (HSA_RECORD_INVALID). Tests: tests/health/account_documents.rs hsa_*.
//   The agreed current-year family split requires a positive combined share on the
//   client and save; either spouse may have zero. Tests: interview
//   validation_audit_f06_hsa_family_parts_* (fresh rows, rejected state, reopen).
//   Lifetime funding: HsaFundingUseHistory records actual2007–2024 uses on the
//   existing account/page; existing funding_history alone owns2025 facts.
//   Both block another lifetime exclusion without creating current income or
//   contributions. Date/coverage/duplicate guards repeat on save and restore.
//   Tests: health_accounts/interview hsa_earlier_*; Notice2008-51 pp4–5.
//   The refusal-only confirmations are gone (the excess accounting, the
//   last-month history, the 2025 funding history, and the left-out flags on corrections, late
//   withdrawals, prior employer excess and 2026 funding): the printed dates and amounts decide.
//   Evidence: hsa.rs, hsa_funding.rs, hsa_last_month.rs, hsa_correction.rs, medical_account.rs,
//   federal_model.rs FederalHsa; tests:
//   hsa_monthly_eligibility_and_medical_withdrawals_after_medicare,
//   hsa_shared_family_limit_does_not_transfer_unused_catchup,
//   uncorrected_personal_hsa_excess_reduces_deduction_and_creates_f5329_tax,
//   prior_last_month_hsa_recapture_uses_historical_months_and_not_current_limits
//   (health_accounts.rs); i8889, draft i5329, Rev. Proc. 2025-19, Notice 2008-52
//   C-fixture (2026-09-25, tests/health/health_accounts.rs): hsa_return_matches_hand_computation (lines 1-17b by
//   hand), hsa_contributions_are_capped_at_the_self_only_limit (excess accounting required),
//   age_55_catch_up_adds_1000, nonmedical_distribution_is_taxed_20_percent_before_65.

// Form 8889; 2026 contribution limits: Rev. Proc. 2025-19.

// IRC 223(b)(3) and Form 8889 line 7: the $1,000 catch-up needs age 55 before
// the close of the tax year. The age is day-exact (Pub. 501): an owner born
// January 1, 1972 attained 55 on December 31, 2026.
pub(crate) fn catchup_age(birth: u32, year: u32) -> bool {
    crate::federal_model::age_on(birth, year * 10000 + 1231) >= 55
}
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
#[cfg(test)]
use crate::kernel::ratio::dollars;
use crate::kernel::ratio::exact::{self, Exact, whole};
use crate::kernel::ratio::positive;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line_cents;
use crate::{
    FederalHsa, FederalPeople, FederalResult, FederalStatus, HsaCorrection, HsaCoverage, Owner,
};

impl crate::HsaAccount {
    // The first rule an account breaks, in the order the rules are declared
    // (the client's rules on each type, repeated on save with the same keys).
    //
    // `employer` is the contribution total the page shows read-only: the
    // owner's W-2 box 12 code W, which the server derives on every save.
    pub(crate) fn issue(&self, employer: i64) -> Option<crate::validation::ValidationIssue> {
        self.corrections
            .iter()
            .enumerate()
            .find_map(|(index, row)| {
                row.record().issue().map(|issue| issue.in_list("corrections", index))
            })
            .or_else(|| {
                self.late_excess.iter().enumerate().find_map(|(index, row)| {
                    row.issue().map(|issue| issue.in_list("late_excess", index))
                })
            })
            .or_else(|| {
                self.prior_employer_excess.iter().enumerate().find_map(|(index, row)| {
                    row.issue().map(|issue| issue.in_list("prior_employer_excess", index))
                })
            })
            .or_else(|| {
                self.excess_accounting
                    .as_ref()
                    .and_then(|row| row.issue().map(|issue| issue.in_field("excess_accounting")))
            })
            .or_else(|| {
                self.funding_history.iter().enumerate().find_map(|(index, row)| {
                    row.issue().map(|issue| issue.in_list("funding_history", index))
                })
            })
            .or_else(|| {
                self.uses
                    .iter()
                    .enumerate()
                    .find_map(|(index, row)| row.issue().map(|issue| issue.in_list("uses", index)))
            })
            .or_else(|| {
                self.last_month_history
                    .as_ref()
                    .and_then(|row| row.issue().map(|issue| issue.in_field("last_month_history")))
            })
            .or_else(|| self.cross_record_issue(employer))
    }
    // The sums across the account's records retain their original native
    // order; local scalar fields and actual history rows locate each failure.
    fn cross_record_issue(&self, employer: i64) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let corrections: alloc::vec::Vec<_> =
            self.corrections.iter().map(crate::HsaCorrectionPage::record).collect();
        let personal_returned = HsaCorrection::manual_returned_for_return_year(&corrections, false);
        let employer_returned = HsaCorrection::manual_returned_for_return_year(&corrections, true);
        if personal_returned > whole(self.personal) {
            return Some(
                ValidationIssue::new("HSA_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED")
                    .at_field("personal"),
            );
        }
        if employer_returned > whole(employer) {
            return Some(
                ValidationIssue::new("HSA_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED")
                    .at_field("employer"),
            );
        }
        if let Some(v) = &self.excess_accounting {
            if whole(v.later_employer_deposits) > whole(employer) - employer_returned {
                return Some(
                    ValidationIssue::new("HSA_LATER_EMPLOYER_ABOVE_NET")
                        .at_field("later_employer_deposits")
                        .in_field("excess_accounting"),
                );
            }
            if whole(v.later_return_year_deposits) - whole(v.later_employer_deposits)
                > whole(self.personal) - personal_returned
            {
                return Some(
                    ValidationIssue::new("HSA_LATER_PERSONAL_ABOVE_NET")
                        .at_field("later_return_year_deposits")
                        .in_field("excess_accounting"),
                );
            }
        }
        HsaFundingHistory::same_month_issue(&self.funding_history)
            .or_else(|| crate::HsaFundingUseHistory::same_month_issue(&self.uses))
    }
}
struct ContributionLimitTrace {
    before_archer: i64,
    allocated: i64,
    catchup: i64,
}

struct ContributionAmounts {
    archer: i64,
    funding: i64,
    personal: i64,
    employer: i64,
    prior_deducted: i64,
}

// Form 8889 lines 2 to 13 print whole dollars figured from the printed lines; the
// returned line 13 is the deduction that crosses to Schedule 1 (D-2026-10-19-12).
fn contribution_lines(
    owner: usize,
    trace: &ContributionLimitTrace,
    amounts: ContributionAmounts,
    r: &mut FederalResult,
) -> Result<i64, crate::medical_account::OutputFailure> {
    let ContributionAmounts { archer, funding, personal, employer, prior_deducted } = amounts;
    let cents = |numerator| crate::kernel::ratio::fraction(numerator, 1, 12);
    let l = crate::forms::form_8889::contribution(&crate::forms::form_8889::ContributionCents {
        personal,
        before_archer: cents(trace.before_archer),
        archer,
        allocated: cents(trace.allocated),
        catchup: cents(trace.catchup),
        employer,
        funding,
        prior_deducted,
    });
    emit_lines(
        r,
        &mut (),
        [
            ("personal", "2", l.l2),
            ("limit", "3", l.l3),
            ("archer", "4", l.l4),
            ("after_archer", "5", l.l5),
            ("allocated", "6", l.l6),
            ("catchup", "7", l.l7),
            ("total_limit", "8", l.l8),
            ("employer", "9", l.l9),
            ("funding", "10", l.l10),
            ("other", "11", l.l11),
            ("personal_limit", "12", l.l12),
            ("deduction", "13", l.l13),
        ],
        |(suffix, printed, amount), _, _| {
            Some(calculation_row(
                Some(crate::FormLine::new(crate::TaxForm::Form8889, printed)),
                &alloc::format!("hsa_{}_contribution_{suffix}", owner + 1),
                LineValue::MoneyDollars(amount),
                &["adjustments.hsa", "adjustments.archer"],
                "https://www.irs.gov/pub/irs-dft/i8889--dft.pdf",
                msg!("EXPLAIN_HSA_CONTRIBUTION_CENTS"),
            ))
        },
    );
    Ok(l.l13)
}

// Dollar outputs except the personal deduction, retained in cents alongside
// other source adjustments until the existing return rounding boundary.
// Detailed on-screen cents come from the same classified owner totals as the
// whole-dollar estimate. Do not reconstruct penalty exceptions from rounded rows.
fn distribution_lines(
    sources: &[crate::medical_account::OwnedHsaAmounts; 2],
    taxable: &[Exact; 2],
    penalized: &[Exact; 2],
    r: &mut FederalResult,
) -> Result<(), crate::medical_account::OutputFailure> {
    for (owner, source) in sources.iter().enumerate() {
        if source.gross == whole(0) && source.pledged == whole(0) {
            continue;
        }
        let context = crate::IssueContext::owner(
            crate::TaxSection::Hsa,
            if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
        );
        let gross = &source.gross - &source.pledged;
        let excluded = &source.rollover + &source.returned_excess;
        let net = &gross - &excluded;
        let form_taxable = &taxable[owner] - &source.pledged;
        let form_penalized = &penalized[owner] - &source.pledged_penalized;
        for (suffix, printed, amount) in [
            ("distribution_gross", Some("14a"), gross),
            ("distribution_excluded", Some("14b"), excluded),
            ("distribution_net", Some("14c"), net),
            ("distribution_medical", Some("15"), source.medical.clone()),
            ("distribution_taxable", Some("16"), form_taxable.clone()),
            ("distribution_penalty_base", None, form_penalized.clone()),
            ("distribution_tax", Some("17b"), exact::rounded_exact(&(&form_penalized / whole(5)))),
            ("pledged_income", None, source.pledged.clone()),
            ("pledged_penalty_base", None, source.pledged_penalized.clone()),
        ] {
            let amount = if printed.is_some() {
                crate::medical_account::required_output(&amount, context.clone())?
            } else if let Some(amount) =
                crate::medical_account_return::explanation_output(&amount, context.clone(), r)
            {
                amount
            } else {
                continue;
            };
            emit_lines(
                r,
                &mut (),
                [(suffix, printed, amount)],
                |(suffix, printed, amount), _, _| {
                    Some(calculation_row(
                        printed.map(|line| crate::FormLine::new(crate::TaxForm::Form8889, line)),
                        &alloc::format!("hsa_{}_{suffix}", owner + 1),
                        LineValue::MoneyCents(amount),
                        &["income.medical_distribution", "adjustments.hsa"],
                        "https://www.irs.gov/pub/irs-dft/i8889--dft.pdf",
                        msg!("EXPLAIN_HSA_DISTRIBUTION_CENTS"),
                    ))
                },
            );
        }
        crate::line_emission::form_count(
            r,
            crate::FormLine::new(crate::TaxForm::Form8889, "17a"),
            &alloc::format!("hsa_{}_distribution_exception", owner + 1),
            i64::from(form_taxable > form_penalized),
            &["income.medical_distribution", "adjustments.hsa.late_excess"],
            "https://www.irs.gov/pub/irs-dft/i8889--dft.pdf",
            msg!("EXPLAIN_HSA_DISTRIBUTION_EXCEPTION"),
        );
        // These transfer explanations accompany the required form rows above;
        // the actual native income/tax outputs remain fallible below.
        for (suffix, amount) in [
            ("distribution_estimate_income", exact::dollars_exact(&taxable[owner])),
            (
                "distribution_estimate_tax",
                exact::rounded_exact(&(exact::dollars_exact(&penalized[owner]) / whole(5))),
            ),
        ] {
            let Some(cents) = crate::medical_account_return::explanation_output(
                &(amount * whole(100)),
                context.clone(),
                r,
            ) else {
                continue;
            };
            emit_lines(r, &mut (), [(suffix, cents)], |(suffix, cents), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("hsa_{}_{suffix}", owner + 1),
                    LineValue::MoneyCents(cents),
                    &["income.medical_distribution", "adjustments.hsa.late_excess"],
                    "https://www.irs.gov/instructions/i1040gi",
                    msg!("EXPLAIN_HSA_DISTRIBUTION_ESTIMATE"),
                ))
            });
        }
    }
    Ok(())
}

// Existing contribution-owner facts, separate from distribution source rows.
pub(crate) struct ContributionContext<'a> {
    pub msa: [Exact; 2],
    pub statements: &'a [crate::contribution_documents::StatementAmounts; 2],
    pub failed: &'a [crate::medical_rollover::Contribution; 2],
    pub married: bool,
    pub family_share: Option<&'a HsaFamilyShare>,
}

// Existing owner outputs plus its pre-filing income projection. Keeping this
// runtime trace avoids rebuilding contribution/distribution classes in PTC.
// Final native component ownership. Money is whole dollars except the three
// jointly rounded correction groups, which retain actual source cents.
#[derive(Default)]
pub(crate) struct OwnerAmounts {
    pub distribution_income: [i64; 2],
    pub distribution_tax: [i64; 2],
    pub funding_income: [i64; 2],
    pub last_month_income: [i64; 2],
    pub testing_tax: [i64; 2],
    pub excess_tax: [i64; 2],
    pub correction_income: [Exact; 2],
    pub correction_employer_income: [Exact; 2],
    pub excess_employer_income: [Exact; 2],
}

pub(crate) struct CalculationResult {
    pub sources: OwnerAmounts,
    pub deduction: i64,
    pub income: i64,
    pub tax: i64,
    pub recapture: i64,
    pub correction_income: i64,
    pub correction_employer_income: i64,
    pub excess_employer_income: i64,
    pub excess_tax: i64,
    pub last_month_income: i64,
    pub testing_tax: i64,
    pub work_income_cents: Exact,
    // Per-owner deductions for the runtime owner projection.
    pub deduction_by_owner: [i64; 2],
}

pub(crate) fn calculate(
    entries: &[FederalHsa],
    people: &FederalPeople,
    status: FederalStatus,
    contributions: &ContributionContext<'_>,
    distributions: &[crate::medical_account::OwnedHsaAmounts; 2],
    income: &[crate::FederalIncomeEntry],
    r: &mut FederalResult,
) -> Result<CalculationResult, crate::medical_account::OutputFailure> {
    let msa = &contributions.msa;
    let married = contributions.married;
    let family_share = contributions.family_share;
    if let Some(key) = crate::hsa_correction::source_rule(entries, income) {
        crate::require_localized!(
            r,
            false,
            &crate::IssueContext::Section(crate::TaxSection::Hsa)
                .alternatives(&[crate::TaxSection::Hsa, crate::TaxSection::OtherIncome]),
            "HSA corrective statement",
            key,
            "Reconcile the actual HSA corrective statement with its contribution-year, payer, principal and signed attributable income facts.",
        );
    }
    let mut accounts = [None, None];
    let mut form_distributions = distributions.clone();
    let mut printed_deduction = [0i64; 2];
    let mut taxable = distributions.each_ref().map(|v| v.taxable.clone());
    let mut penalized = distributions.each_ref().map(|v| v.penalized.clone());
    let mut personal = [whole(0), whole(0)];
    let mut employer = [whole(0), whole(0)];
    let mut funding_income = [whole(0), whole(0)];
    let mut sources = OwnerAmounts::default();
    let mut correction_income = whole(0);
    let mut correction_employer_principal_income = whole(0);
    let mut excess_tax = 0;
    let mut excess_employer_other_income = whole(0);
    let mut last_month_income = [whole(0), whole(0)];
    for h in entries {
        let owner = usize::from(h.owner == Owner::Spouse);
        let person = if owner == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
        crate::require_localized!(
            r,
            h.valid()
                && person.is_some()
                && (owner == 0 || status == FederalStatus::Joint)
                && accounts[owner].is_none(),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "HSA",
            "ISSUE_USE_ONE_COMBINED_HSA_ENTRY_PER_ELIGIBLE_FILER_AND_CONFIRM_MONTHLY_E_ECDA078E",
            "Use one combined HSA entry per eligible filer and confirm monthly eligibility and ordinary account treatment.",
        );
        crate::require_localized!(
            r,
            person.is_some_and(|p| {
                // Known claimability gives zero contribution capacity below;
                // it does not disqualify distributions from an existing HSA.
                // A qualified IRA funding transfer needs actual contribution eligibility.
                h.funding.is_empty() || !p.dependency.claimable()
            }),
            &crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner).alternatives(&[
                crate::TaxSection::Hsa,
                if h.owner == Owner::Spouse {
                    crate::TaxSection::Spouse
                } else {
                    crate::TaxSection::People
                },
            ]),
            "HSA",
            "HSA_DEPENDENCY_SOURCE_REQUIRED",
            "Confirm whether this HSA owner can be claimed as a dependent to calculate contribution income and excess tax. A qualified IRA-to-HSA funding transfer also requires an eligible owner; claimable owners can still receive ordinary HSA distributions.",
        );
        accounts[owner] = Some(h);
        personal[owner] = h.net_personal() + &contributions.failed[owner].personal;
        employer[owner] = h.net_employer();
        for correction in &h.corrections {
            crate::require_localized!(
                r,
                correction.eligible(),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_CONFIRM_A_COMPLETE_TIMELY_HSA_CORRECTION_AND_CONTRIBUTION_YEAR_REPO_E7E27A00",
                "An HSA excess-contribution return made after its correction deadline needs separate treatment.",
            );
            crate::require_localized!(
                r,
                person.is_none_or(|p| p.death_date().is_none_or(|d| correction.returned <= d)),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_POST_DEATH_HSA_CORRECTIONS_REQUIRE_BENEFICIARY_OR_ESTATE_TREATMENT_0EE8EEC3",
                "Post-death HSA corrections require beneficiary or estate treatment.",
            );
            // A linked actual statement already supplies its gross once.
            if correction.document.is_none() && correction.returned / 10000 == 2026 {
                let gross = whole(correction.principal) + whole(correction.earnings);
                form_distributions[owner].gross += &gross;
                form_distributions[owner].returned_excess += gross;
            }
            let earned = correction.income();
            let principal = correction.current_employer_principal_income();
            correction_income += whole(earned);
            correction_employer_principal_income += whole(principal);
            sources.correction_income[owner] += whole(earned);
            sources.correction_employer_income[owner] += whole(principal);
        }
        if !h.corrections.is_empty()
            && let Some(amount) = crate::medical_account_return::explanation_output(
                &(h.corrected(false) + h.corrected(true)),
                crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                r,
            )
        {
            line_cents(
                r,
                &alloc::format!("hsa_{}_corrected_contributions", owner + 1),
                amount,
                &["adjustments.hsa.corrections"],
                "https://www.irs.gov/instructions/i8889",
                msg!("EXPLAIN_HSA_N_CORRECTED_CONTRIBUTIONS"),
            );
        }
        // An unconfirmed deposit was left out before the rules ran.
        for deposit in &h.prior_employer_excess {
            if !deposit.in_wages {
                excess_employer_other_income += whole(deposit.amount);
                sources.excess_employer_income[owner] += whole(deposit.amount);
                line_cents(
                    r,
                    &alloc::format!("hsa_{}_prior_designated_employer_excess_income", owner + 1),
                    deposit.amount,
                    &["adjustments.hsa.prior_employer_excess"],
                    "https://www.irs.gov/irb/2008-29_IRB",
                    msg!("EXPLAIN_HSA_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS_INCOME"),
                );
            }
        }
        if let Some(prior) = h.excess_accounting.as_ref().and_then(|v| v.prior_year.as_ref()) {
            crate::require_localized!(
                r,
                h.prior_employer_excess.iter().map(|v| whole(v.amount)).sum::<Exact>()
                    <= whole(prior.line_48),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA prior employer excess",
                "ISSUE_EMPLOYER_EXCESS_DEPOSITED_IN_2026_FOR_2025_CANNOT_EXCEED_THE_RETAIN_11C1C5AC",
                "Employer excess deposited in 2026 for 2025 cannot exceed the retained 2025 Form 5329 total excess.",
            );
        }
        if let (Some(history), Some(person)) = (&h.last_month_history, person) {
            crate::require_localized!(
                r,
                h.funding_history.iter().all(|v| history.consistent_loss(v)),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_THE_PRIOR_ELIGIBILITY_HISTORIES_MUST_AGREE_ACROSS_OVERLAPPING_TESTING_PERIODS",
                "The prior last-month-rule history and each prior funding transfer must agree on the first loss of eligibility across overlapping testing periods.",
            );
            let funding = crate::medical_account::required_output(
                &h.funding_history.iter().map(|v| whole(v.amount)).sum::<Exact>(),
                crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
            )?;
            let (ordinary_limit, elected_limit) = history.limits_exact(person.birth_date);
            crate::require_localized!(
                r,
                person.birth_date <= 20251201
                    && (whole(history.contributions) + whole(funding)) <= elected_limit,
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_PRIOR_CONTRIBUTIONS_MUST_FIT_THE_2025_LIMIT_INCLUDING_THE_LAST_MONT_9539DED2",
                "Prior contributions must fit the 2025 limit including the last-month rule; remaining prior excess needs separate treatment.",
            );
            if let Some(loss) = &history.loss {
                crate::require_localized!(
                    r,
                    (loss.reason != HsaLossReason::Death || person.death_date() == Some(loss.date))
                        && person.death_date().is_none_or(|date| loss.date <= date),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                    "HSA",
                    "ISSUE_PRIOR_LAST_MONTH_RULE_ELIGIBILITY_LOSS_AND_ANY_DEATH_EXCEPTION_MUST_E4F84B65",
                    "Prior last-month-rule eligibility loss and any death exception must agree with the account owner's final-return date.",
                );
            }
            last_month_income[owner] = whole(history.recapture_cents(person.birth_date, funding));
            line_cents(
                r,
                &alloc::format!("hsa_{}_prior_monthly_limit", owner + 1),
                crate::kernel::ratio::exact::rounded(&ordinary_limit)
                    .expect("bounded HSA monthly limit"),
                &["adjustments.hsa.last_month_history"],
                "https://www.irs.gov/instructions/i8889",
                msg!("EXPLAIN_HSA_N_PRIOR_MONTHLY_LIMIT"),
            );
        }
        funding_income[owner] += h.funding.iter().map(|v| whole(v.recapture())).sum::<Exact>();
        crate::require_localized!(
            r,
            h.uses.iter().all(|transfer| person.is_none_or(|p| transfer.funded >= p.birth_date)),
            &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
            "HSA",
            "ISSUE_THE_PRIOR_FUNDING_TRANSFER_CANNOT_PRECEDE_THE_ACCOUNT_OWNER_S_BIRTH_B039F8EB",
            "The prior funding transfer cannot precede the account owner's birth.",
        );
        for transfer in &h.funding_history {
            crate::require_localized!(
                r,
                person.is_none_or(|p| transfer.funded >= p.birth_date),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_THE_PRIOR_FUNDING_TRANSFER_CANNOT_PRECEDE_THE_ACCOUNT_OWNER_S_BIRTH_B039F8EB",
                "The prior funding transfer cannot precede the account owner's birth.",
            );
            if let (Some(loss), Some(person)) = (&transfer.loss, person) {
                crate::require_localized!(
                    r,
                    loss.reason != HsaLossReason::Death || person.death_date() == Some(loss.date),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                    "HSA",
                    "ISSUE_A_DEATH_EXCEPTION_MUST_AGREE_WITH_THE_ACCOUNT_OWNER_S_FINAL_RETURN_F625155C",
                    "A death exception must agree with the account owner's final-return death date.",
                );
                crate::require_localized!(
                    r,
                    person.death_date().is_none_or(|date| loss.date <= date),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                    "HSA",
                    "ISSUE_ELIGIBILITY_LOSS_CANNOT_FIRST_OCCUR_AFTER_THE_ORIGINAL_ACCOUNT_OWNE_1C32D4D4",
                    "Eligibility loss cannot first occur after the original account owner's death.",
                );
            }
            funding_income[owner] += whole(transfer.recapture());
        }
        let late_excess: Exact = h.late_excess.iter().map(|v| whole(v.amount)).sum();
        form_distributions[owner].gross += &late_excess;
        if late_excess > whole(0)
            && let Some(amount) = crate::medical_account_return::explanation_output(
                &late_excess,
                crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                r,
            )
        {
            line_cents(
                r,
                &alloc::format!("hsa_{}_late_excess_withdrawals", owner + 1),
                amount,
                &["adjustments.hsa.late_excess"],
                "https://www.irs.gov/pub/irs-dft/i8889--dft.pdf",
                msg!("EXPLAIN_HSA_N_LATE_EXCESS_WITHDRAWALS"),
            );
        }
        for v in &h.late_excess {
            let (date, amount, disabled) = (v.date, v.amount, v.disabled);
            crate::require_localized!(
                r,
                person.is_none_or(|p| p.death_date().is_none_or(|d| date <= d)),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_POST_DEATH_HSA_WITHDRAWALS_REQUIRE_BENEFICIARY_ACCOUNT_TREATMENT_08CA97B0",
                "Post-death HSA withdrawals require beneficiary account treatment.",
            );
            taxable[owner] += whole(amount);
            let over65 = person
                .is_some_and(|p| crate::federal_model::after_65th_birthday(p.birth_date, date));
            if !over65 && !disabled {
                penalized[owner] += whole(amount);
            }
        }
    }
    for (owner, source) in distributions.iter().enumerate() {
        for (suffix, amount, explanation) in [
            ("rollover", &source.rollover, msg!("EXPLAIN_HSA_N_ROLLOVER")),
            ("deemed", &source.deemed, msg!("EXPLAIN_HSA_N_DEEMED")),
        ] {
            if *amount > whole(0)
                && let Some(amount) = crate::medical_account_return::explanation_output(
                    amount,
                    crate::IssueContext::owner(
                        crate::TaxSection::Hsa,
                        if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
                    ),
                    r,
                )
            {
                line_cents(
                    r,
                    &alloc::format!("hsa_{}_{}", owner + 1, suffix),
                    amount,
                    &["income.medical_distribution"],
                    "https://www.irs.gov/instructions/i8889",
                    explanation,
                );
            }
        }
    }
    if let (Some(a), Some(b)) = (
        accounts[0].and_then(|h| h.last_month_history.as_ref()),
        accounts[1].and_then(|h| h.last_month_history.as_ref()),
    ) {
        if let (Some(af), Some(bf)) = (&a.family, &b.family) {
            crate::require_localized!(
                r,
                af.spouse_months == b.months
                    && bf.spouse_months == a.months
                    && af.spouse_archer == b.archer
                    && bf.spouse_archer == a.archer
                    && af.reciprocal(bf),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_THE_TWO_HISTORICAL_FAMILY_ALLOCATIONS_MUST_USE_MATCHING_SPOUSE_COVE_928CA8C8",
                "The two historical family allocations must use matching spouse coverage, Archer deposits and reciprocal agreed shares.",
            );
        } else {
            crate::require_localized!(
                r,
                a.family.is_none() && b.family.is_none(),
                &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                "HSA",
                "ISSUE_BOTH_SPOUSES_HISTORICAL_RECORDS_MUST_IDENTIFY_THEIR_SHARED_FAMILY_A_786BF2A0",
                "Both spouses' historical records must identify their shared family allocation.",
            );
        }
    }
    let december_family = family_share.is_some_and(|v| v.method == HsaAllocationMethod::December);
    let other_coverage = family_share.and_then(|v| v.other_coverage.as_ref());
    let eligible = [
        !people.taxpayer.dependency.claimable(),
        people.spouse.as_ref().is_some_and(|p| !p.dependency.claimable()),
    ];
    let coverage_at = |month| {
        let mut coverage = [HsaCoverage::None; 2];
        for i in 0..2 {
            if let Some(h) = accounts[i].filter(|_| eligible[i]) {
                coverage[i] = h.month(month);
            } else if accounts[i].is_none() && accounts[1 - i].is_some() {
                // An external spouse on a separate return is represented by
                // direct eligibility context, not a second account or income.
                let claimable = if i == 0 {
                    people.taxpayer.dependency.claimable()
                } else {
                    people.spouse.as_ref().is_some_and(|p| p.dependency.claimable())
                };
                if !claimable {
                    coverage[i] = other_coverage.map_or(HsaCoverage::None, |v| v.month(month));
                }
            }
        }
        coverage
    };
    if married && accounts.iter().filter(|h| h.is_some()).count() == 1 {
        let i = usize::from(accounts[0].is_none());
        let needs_limit = accounts[i].is_some_and(|h| {
            eligible[i]
                && (&personal[i] + &employer[i] > whole(0) || !h.funding.is_empty())
                && (0..12).any(|m| h.month(m) != HsaCoverage::None)
        });
        crate::require_localized!(
            r,
            !needs_limit || other_coverage.is_some(),
            &crate::IssueContext::owner(
                crate::TaxSection::Hsa,
                if i == 0 { Owner::Taxpayer } else { Owner::Spouse },
            ),
            "HSA",
            "HSA_OTHER_OWNER_ELIGIBILITY_REQUIRED",
            "Enter the other filer's actual monthly HSA eligibility, including an ineligible full year. An absent HSA account does not establish whether spouses share the family contribution limit.",
        );
    }
    if let Some(share) = family_share {
        let count = accounts.iter().filter(|h| h.is_some()).count();
        let context = if status == FederalStatus::Joint {
            (other_coverage.is_none() && count == 2) || count == 1
        } else {
            accounts[0].is_some() && accounts[1].is_none() && (married || other_coverage.is_none())
        };
        crate::require_localized!(
            r,
            share.valid()
                && context
                && (!december_family || coverage_at(11).iter().any(|c| *c != HsaCoverage::None)),
            &crate::IssueContext::Section(crate::TaxSection::Hsa),
            "HSA",
            "ISSUE_FAMILY_ALLOCATION_REQUIRES_COMPLETE_SPOUSE_COVERAGE_AN_ELIGIBLE_FAM_CE8DC988",
            "Confirm the contribution-limit method and actual owner eligibility. December requires December1 eligibility; family capacity is divided only while both spouses are eligible. Use other-filer context only when that person has no HSA entry; separate returns retain only the taxpayer's account.",
        );
    }
    let mut limits = [0; 2];
    let mut catchup = [0; 2];
    let mut shared = 0;
    let mut family_limits = [0; 2];
    let mut family_shared = 0;
    let mut common_family = 0;
    let mut has_family = false;
    for month in 0..12 {
        let coverage = coverage_at(month);
        let family = coverage.contains(&HsaCoverage::Family);
        has_family |= family;
        crate::require_localized!(
            r,
            !married || status == FederalStatus::Joint || !family || family_share.is_some(),
            &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
            "HSA",
            "ISSUE_SEPARATE_RETURN_FAMILY_CONTRIBUTIONS_REQUIRE_THE_OTHER_SPOUSE_S_SHA_F78A5F4F",
            "Separate-return family contributions require the other spouse's shared-limit allocation.",
        );
        let mut combined = 0;
        for i in 0..2 {
            if coverage[i] == HsaCoverage::None {
                continue;
            }
            let base = if family {
                crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS
            } else {
                crate::year_constants::HSA_SELF_ONLY_LIMIT_2026_CENTS
            };
            limits[i] += base;
            if family {
                family_limits[i] += base;
            }
            combined += base;
            let person = if i == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
            if person.is_some_and(|p| crate::hsa::catchup_age(p.birth_date, 2026)) {
                catchup[i] += crate::year_constants::HSA_CATCH_UP_CENTS;
            }
        }
        shared +=
            if family { crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS } else { combined };
        if family {
            family_shared += crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS;
            if coverage.iter().all(|c| *c != HsaCoverage::None) {
                common_family += crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS;
            }
        }
    }
    // Form 8889 line 4 includes both spouses' annual Archer contributions
    // whenever either had family coverage, even if one is ineligible in December.
    let family_msa: Exact = msa.iter().sum();
    let archer_reduction = |i: usize| {
        if has_family && married { family_msa.clone() } else { msa[i].clone() }
    };
    let cash = [0, 1].map(|i| &personal[i] + &employer[i]);
    let funding =
        accounts.map(|h| h.map_or(0, |h| h.funding.iter().map(|v| v.amount).sum::<i64>()));
    let total = [0, 1].map(|i| &cash[i] + whole(funding[i]));
    let mut shared_used = whole(0);
    let mut shared_funding = 0;
    let mut shared_excess = whole(0);
    let mut december_self_capacity = 0;
    for i in 0..2 {
        let native_context = crate::IssueContext::owner(
            crate::TaxSection::Hsa,
            if i == 0 { Owner::Taxpayer } else { Owner::Spouse },
        );
        // Form8889 line6 allocates the family allowance only while both
        // spouses are eligible. Otherwise the eligible owner keeps that
        // owner's capacity. Equal shares are the statutory default, never
        // inferred from deposits. MSA reduction precedes the shared division.
        let ordinary = if has_family && married {
            let common = crate::medical_account::required_output(
                &phase::floor(whole(common_family) - &family_msa * whole(12), whole(0)),
                native_context.clone(),
            )?;
            let other = family_share.map_or_else(
                || crate::kernel::ratio::fraction(common, 1, 2),
                |share| share.amount(common, 1 - i),
            );
            limits[i] - family_limits[i]
                + crate::medical_account::required_output(
                    &phase::floor(
                        whole(family_limits[i]) - &family_msa * whole(12) - whole(other),
                        whole(0),
                    ),
                    native_context.clone(),
                )?
                + catchup[i]
        } else {
            crate::medical_account::required_output(
                &phase::floor(whole(limits[i] + catchup[i]) - &msa[i] * whole(12), whole(0)),
                native_context.clone(),
            )?
        };
        let separate_catchup = has_family && married;
        let ordinary_catchup = if separate_catchup { catchup[i] } else { 0 };
        let mut trace = ContributionLimitTrace {
            before_archer: limits[i] + catchup[i] - ordinary_catchup,
            allocated: ordinary - ordinary_catchup,
            catchup: ordinary_catchup,
        };
        let mut allowed = ordinary;
        if let Some(h) = accounts[i] {
            let person = if i == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
            if let Some(loss) = &h.last_month_loss {
                crate::require_localized!(
                    r,
                    coverage_at(11)[i] != HsaCoverage::None
                        && person.is_some_and(|p| {
                            (loss.reason != HsaLossReason::Death
                                || loss.date / 10000 == 2027
                                || p.death_date() == Some(loss.date))
                                && p.death_date().is_none_or(|d| loss.date <= d)
                        }),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                    "HSA",
                    "ISSUE_CURRENT_LAST_MONTH_TESTING_HISTORY_REQUIRES_DECEMBER_1_ELIGIBILITY_14EC49E7",
                    "Current last-month testing history requires December 1 eligibility and a death exception consistent with People.",
                );
            }
            if let Some(share) = family_share
                .filter(|_| december_family && coverage_at(11).contains(&HsaCoverage::Family))
            {
                crate::require_localized!(
                    r,
                    total[i] == whole(0)
                        || coverage_at(11)[i] != HsaCoverage::None
                        || h.excess_accounting.is_some(),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa),
                    "HSA",
                    "ISSUE_AN_OWNER_WITH_CONTRIBUTIONS_UNDER_THE_DECEMBER_FAMILY_METHOD_MUST_B_F7C5DF33",
                    "An owner with contributions under the December family method must be eligible on December 1.",
                );
                let extra = if coverage_at(11)[i] != HsaCoverage::None
                    && person.is_some_and(|p| crate::hsa::catchup_age(p.birth_date, 2026))
                {
                    crate::year_constants::HSA_CATCH_UP_CENTS
                } else {
                    0
                };
                // i8889 lines3/5 include an unmarried catch-up before the
                // Archer floor. Married family-anytime catch-up is separate
                // line7, added only after allocating the nonnegative line5.
                let inline_extra = if separate_catchup { 0 } else { extra };
                let separate_extra = extra - inline_extra;
                let after_archer = crate::medical_account::required_output(
                    &phase::floor(
                        whole(crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS + inline_extra)
                            - &family_msa,
                        whole(0),
                    ),
                    native_context.clone(),
                )?;
                let family = if coverage_at(11).iter().all(|c| *c != HsaCoverage::None) {
                    share.amount(after_archer, i)
                } else {
                    after_archer
                };
                allowed = (family + separate_extra) * 12;
                trace = ContributionLimitTrace {
                    before_archer: (crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS
                        + inline_extra)
                        * 12,
                    allocated: family * 12,
                    catchup: separate_extra * 12,
                };
            }
            if family_share
                .map_or(!(has_family && married), |s| s.method == HsaAllocationMethod::December)
                && coverage_at(11)[i] != HsaCoverage::None
            {
                let base = if coverage_at(11)[i] == HsaCoverage::Family {
                    crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS
                } else {
                    crate::year_constants::HSA_SELF_ONLY_LIMIT_2026_CENTS
                };
                let extra = if person.is_some_and(|p| crate::hsa::catchup_age(p.birth_date, 2026)) {
                    crate::year_constants::HSA_CATCH_UP_CENTS
                } else {
                    0
                };
                if family_share.is_none()
                    || !coverage_at(11).contains(&HsaCoverage::Family)
                    || !married
                {
                    let separate_extra = if separate_catchup { extra } else { 0 };
                    let december = (crate::medical_account::required_output(
                        &phase::floor(
                            whole(base + extra - separate_extra) - archer_reduction(i),
                            whole(0),
                        ),
                        native_context.clone(),
                    )? + separate_extra)
                        * 12;
                    if december > allowed {
                        allowed = phase::floor(allowed, december);
                        trace = ContributionLimitTrace {
                            before_archer: (base + extra - separate_extra) * 12,
                            allocated: positive(december - separate_extra * 12),
                            catchup: separate_extra * 12,
                        };
                    }
                }
                let accepted_cash = if h.excess_accounting.is_some() {
                    phase::cap(
                        cash[i].clone(),
                        [whole(positive(
                            crate::kernel::ratio::fraction(allowed, 1, 12) - funding[i],
                        ))],
                    )
                } else {
                    cash[i].clone()
                };
                let dependent_cents = crate::medical_account::required_output(
                    &phase::cap(
                        phase::floor(
                            &accepted_cash + whole(funding[i])
                                - crate::kernel::ratio::exact::ratio(ordinary, 12).unwrap(),
                            crate::kernel::ratio::exact::whole(0),
                        ),
                        [accepted_cash],
                    ),
                    native_context.clone(),
                )?;
                #[cfg(test)]
                let dependent = dollars(dependent_cents);
                if dependent_cents > 0 {
                    crate::require_localized!(
                        r,
                        person.is_some_and(|p| p.death_date().is_none_or(|d| d >= 20261201)),
                        &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                        "HSA",
                        "ISSUE_DECEMBER_BASED_ELIGIBILITY_CANNOT_BEGIN_AFTER_THE_ACCOUNT_OWNER_S_D_0DFC1873",
                        "December-based eligibility cannot begin after the account owner's death.",
                    );
                    line_cents(
                        r,
                        &alloc::format!("hsa_{}_last_month_contributions", i + 1),
                        dependent_cents,
                        &["adjustments.hsa"],
                        "https://www.irs.gov/irb/2008-25_IRB",
                        msg!("EXPLAIN_HSA_N_LAST_MONTH_CONTRIBUTIONS"),
                    );
                    if h.last_month_loss
                        .as_ref()
                        .is_some_and(|v| v.date / 10000 == 2026 && v.reason == HsaLossReason::Other)
                    {
                        last_month_income[i] += whole(dependent_cents);
                    }
                    let context = crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner);
                    r.review_notes.push(crate::federal_model::ReviewNote::scoped(
                        #[cfg(test)] alloc::format!(
                            "HSA owner {} uses the 2026 last-month rule. Retain the monthly coverage and contribution records through December 31, 2027. A nonexcepted first loss in 2026 is included here; a first loss in 2027 belongs on the 2027 return. The amount above the ordinary monthly limit is ${}.",
                            i + 1,
                            dependent,
                        ),
                        crate::Message::new(
                            "NOTE_HSA_LAST_MONTH_RULE",
                            alloc::vec![
                                crate::MessageArg::Resource(
                                    if i == 0 { "PERSON_YOU" } else { "PERSON_SPOUSE" }.into()
                                ),
                                crate::MessageArg::Currency {
                                    currency_code: "USD".into(), scale: 2, minor_units: dependent_cents,
                                },
                            ],
                        ),
                        context,
                    ));
                }
            }
        }
        // An election by the eligible spouse cannot give a claimable owner
        // contribution room, including through the December family method.
        if !eligible[i] && accounts[i].is_some() {
            allowed = 0;
            trace = ContributionLimitTrace { before_archer: 0, allocated: 0, catchup: 0 };
        }
        if let Some(h) = accounts[i] {
            let mut prior_deducted_cents = 0;
            if let Some(excess) = &h.excess_accounting {
                // Form 8889 line 10 reduces both the line 8 employer limit and
                // line 12 personal limit. A qualified transfer can exceed the
                // ordinary monthly limit when coverage at transfer permits it.
                // The monthly worksheet divides by 12; source deposits are
                // exact cents, so round that result once to the nearest cent.
                let capacity =
                    positive(crate::kernel::ratio::fraction(allowed, 1, 12) - funding[i]);
                let personal_room = phase::floor(whole(capacity) - &employer[i], whole(0));
                let personal_excess = phase::floor(&personal[i] - &personal_room, whole(0));
                let employer_excess = phase::floor(&employer[i] - whole(capacity), whole(0));
                let current_employer_excess = phase::floor(
                    &employer[i]
                        - whole(excess.later_employer_deposits)
                        - whole(contributions.statements[i].later_employer)
                        - whole(capacity),
                    whole(0),
                );
                let later_employer_excess = &employer_excess - &current_employer_excess;
                let employer_other_income =
                    &current_employer_excess - whole(excess.employer_excess_in_wages);
                let prior = excess.prior_year.as_ref().map_or(0, |v| v.line_48);
                let prior_used_by_distributions = crate::medical_account::required_output(
                    &phase::cap(whole(prior), [taxable[i].clone()]),
                    crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                )?;
                let prior_after_distributions = crate::kernel::carry::closing_values::<
                    crate::kernel::carry::Cents,
                >(
                    prior, prior_used_by_distributions
                )
                .expect("prior HSA excess use is capped by its opening")
                .get();
                let prior_deducted = phase::cap(
                    prior_after_distributions,
                    [crate::medical_account::required_output(
                        &phase::floor(&personal_room - &personal[i], whole(0)),
                        crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                    )?],
                );
                let prior_remaining = crate::kernel::carry::closing_values::<
                    crate::kernel::carry::Cents,
                >(prior_after_distributions, prior_deducted)
                .expect("prior HSA excess deduction is capped by remaining excess")
                .get();
                crate::require_localized!(
                    r,
                    whole(excess.employer_excess_in_wages) <= current_employer_excess
                        && (&personal_excess + &employer_excess > whole(0) || prior > 0),
                    &crate::IssueContext::Section(crate::TaxSection::Hsa).manual(),
                    "HSA excess",
                    "ISSUE_THE_SUPPORTED_FORM_5329_PATH_REQUIRES_A_CURRENT_PERSONAL_EMPLOYER_E_A421EC88",
                    "The supported Form 5329 path requires a current personal/employer excess or retained 2025 line 48 with positive line 49, reconciled W-2 wages for employer excess, and the statutory equal division or an actual agreement for shared eligible family months. Monthly limits are divided by 12 and rounded to cents before qualified IRA funding is subtracted.",
                );
                shared_excess += (&personal_excess + &employer_excess) * whole(12);
                prior_deducted_cents = prior_deducted;
                let employer_income = phase::floor(employer_other_income, whole(0));
                excess_employer_other_income += &employer_income;
                sources.excess_employer_income[i] += &employer_income;
                let account_value = whole(excess.year_end_value)
                    + whole(contributions.statements[i].value)
                    + whole(contributions.statements[i].later)
                    + whole(excess.later_return_year_deposits)
                    + &contributions.failed[i].later;
                let current_excess = &personal_excess + &employer_excess;
                // Form5329 PartVII uses the same accepted account facts and
                // exact-cent limit owner; only line49 is a whole-dollar tax.
                // Lines43/44 are the full source amounts, not the portions
                // capped by the prior carry that actually got absorbed.
                let unused_room = if prior > 0 {
                    crate::medical_account::required_output(
                        &phase::floor(&personal_room - &personal[i], whole(0)),
                        crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                    )?
                } else {
                    0
                };
                let distributions = if prior > 0 { taxable[i].clone() } else { whole(0) };
                let output_context = crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner);
                let distribution_row = crate::medical_account::required_output(
                    &distributions,
                    output_context.clone(),
                )?;
                let excess_row = crate::medical_account::required_output(
                    &current_excess,
                    output_context.clone(),
                )?;
                // Lines 45 to 49 are figured from the printed lines; line 49 is the
                // tax that crosses to Schedule 2 (D-2026-10-19-12).
                let part = crate::forms::form_8889::excess(
                    prior,
                    unused_room,
                    distribution_row,
                    excess_row,
                    crate::medical_account::required_output(
                        &phase::cap(
                            &current_excess + whole(prior_remaining),
                            [account_value.clone()],
                        ),
                        output_context.clone(),
                    )?,
                );
                excess_tax += part.l49;
                sources.excess_tax[i] += part.l49;
                emit_lines(
                    r,
                    &mut (),
                    [
                        (42, part.l42),
                        (43, part.l43),
                        (44, part.l44),
                        (45, part.l45),
                        (46, part.l46),
                        (47, part.l47),
                        (48, part.l48),
                        (49, part.l49),
                    ],
                    |(number, value), _, _| {
                        Some(calculation_row(
                            Some(crate::FormLine::new(
                                crate::TaxForm::Form5329,
                                &alloc::format!("{number}"),
                            )),
                            &alloc::format!("hsa_{}_excess_line{number}", i + 1),
                            LineValue::MoneyDollars(value),
                            &["adjustments.hsa", "adjustments.hsa.excess_accounting"],
                            "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                            msg!("EXPLAIN_HEALTH_ACCOUNT_EXCESS_FORM"),
                        ))
                    },
                );
                if employer_excess > whole(0) {
                    line_cents(
                        r,
                        &alloc::format!("hsa_{}_return_year_uncorrected_employer_excess", i + 1),
                        crate::medical_account::required_output(
                            &employer_excess,
                            output_context.clone(),
                        )?,
                        &["adjustments.hsa.employer", "adjustments.hsa.excess_accounting"],
                        "https://www.irs.gov/instructions/i8889",
                        msg!("EXPLAIN_HSA_N_N_UNCORRECTED_EMPLOYER_EXCESS"),
                    );
                    if later_employer_excess > whole(0) {
                        line_cents(
                            r,
                            &alloc::format!("hsa_{}_later_employer_excess_income", i + 1),
                            crate::medical_account::required_output(
                                &later_employer_excess,
                                output_context.clone(),
                            )?,
                            &["adjustments.hsa.excess_accounting"],
                            "https://www.irs.gov/irb/2008-29_IRB",
                            msg!("EXPLAIN_HSA_N_LATER_EMPLOYER_EXCESS_INCOME"),
                        );
                    }
                }
                if prior > 0 {
                    emit_lines(
                        r,
                        &mut (),
                        [
                            (
                                "used_by_distributions",
                                prior_used_by_distributions,
                                msg!("EXPLAIN_HSA_N_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS"),
                            ),
                            (
                                "deducted_return_year",
                                prior_deducted,
                                msg!("EXPLAIN_HSA_N_PRIOR_EXCESS_N_DEDUCTED_N"),
                            ),
                            (
                                "remaining_following_year",
                                prior_remaining,
                                msg!("EXPLAIN_HSA_N_PRIOR_EXCESS_N_REMAINING_N"),
                            ),
                        ],
                        |(key, amount, explanation), _, _| {
                            Some(calculation_row(
                                None,
                                &alloc::format!("hsa_{}_prior_excess_{}", i + 1, key),
                                LineValue::MoneyCents(amount),
                                &[
                                    "adjustments.hsa.excess_accounting",
                                    "income.medical_distribution",
                                ],
                                "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                                explanation,
                            ))
                        },
                    );
                }
                line_cents(
                    r,
                    &alloc::format!("hsa_{}_return_year_uncorrected_excess", i + 1),
                    crate::medical_account::required_output(
                        &personal_excess,
                        output_context.clone(),
                    )?,
                    &["adjustments.hsa.excess_accounting", "adjustments.hsa.personal"],
                    "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                    msg!("EXPLAIN_HSA_N_N_UNCORRECTED_EXCESS"),
                );
                if let Some(amount) = crate::medical_account_return::explanation_output(
                    &account_value,
                    output_context,
                    r,
                ) {
                    line_cents(
                        r,
                        &alloc::format!("hsa_{}_return_year_excess_account_value", i + 1),
                        amount,
                        &["adjustments.hsa.excess_accounting"],
                        "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                        msg!("EXPLAIN_HSA_N_N_EXCESS_ACCOUNT_VALUE"),
                    );
                }
            } else {
                crate::require_localized!(
                    r,
                    cash[i] == whole(0)
                        || total[i]
                            <= whole(positive(crate::kernel::ratio::fraction(allowed, 1, 12))),
                    &crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                    "HSA",
                    "HSA_EXCESS_ACCOUNTING_REQUIRED",
                    "Contributions exceed this owner's HSA contribution limit. Review eligibility, the family split and timely returns of excess contributions. If excess remains, open Excess contributions to this HSA and enter the year-end account value and any later deposits so the App can calculate Form 5329's 6% additional tax.",
                );
            }
            printed_deduction[i] = contribution_lines(
                i,
                &trace,
                ContributionAmounts {
                    archer: crate::medical_account::required_output(
                        &archer_reduction(i),
                        native_context,
                    )?,
                    funding: funding[i],
                    prior_deducted: prior_deducted_cents,
                    personal: crate::medical_account::required_output(
                        &personal[i],
                        crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                    )?,
                    employer: crate::medical_account::required_output(
                        &employer[i],
                        crate::IssueContext::owner(crate::TaxSection::Hsa, h.owner),
                    )?,
                },
                r,
            )?;
        }
        // Each catch-up belongs to its own owner; an unused spouse catch-up
        // cannot absorb the other owner's contributions.
        let owner_catchup = if december_family {
            if coverage_at(11)[i] != HsaCoverage::None {
                let person = if i == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
                if person.is_some_and(|p| crate::hsa::catchup_age(p.birth_date, 2026)) {
                    1200000
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            catchup[i]
        };
        if accounts[i].is_some() {
            december_self_capacity += positive(allowed - owner_catchup);
        }
        shared_used += phase::floor(&total[i] * whole(12) - whole(owner_catchup), whole(0));
        shared_funding += positive(funding[i] * 12 - owner_catchup);
    }
    crate::require_localized!(
        r,
        !(has_family && married)
            || shared_used - shared_excess
                <= whole(phase::floor(
                    crate::kernel::ratio::fraction(
                        if december_family && !coverage_at(11).contains(&HsaCoverage::Family) {
                            december_self_capacity
                        } else if december_family {
                            crate::medical_account::required_output(
                                &phase::floor(
                                    whole(crate::year_constants::HSA_FAMILY_LIMIT_2026_CENTS)
                                        - &family_msa,
                                    whole(0),
                                ),
                                crate::IssueContext::Section(crate::TaxSection::Hsa),
                            )? * 12
                        } else {
                            shared - family_shared
                                + crate::medical_account::required_output(
                                    &phase::floor(
                                        whole(family_shared) - &family_msa * whole(12),
                                        whole(0),
                                    ),
                                    crate::IssueContext::Section(crate::TaxSection::Hsa),
                                )?
                        },
                        1,
                        12,
                    ) * 12,
                    shared_funding,
                )),
        &crate::IssueContext::Section(crate::TaxSection::Hsa),
        "HSA",
        "ISSUE_COMBINED_CONTRIBUTIONS_EXCEED_THE_MONTHLY_SHARED_FAMILY_LIMIT_AFTER_52D649B7",
        "Combined contributions exceed the monthly shared family limit after Archer MSA contributions; excess-contribution treatment is required.",
    );
    distribution_lines(&form_distributions, &taxable, &penalized, r)?;
    let work_income_cents = taxable.iter().cloned().sum::<Exact>()
        + funding_income.iter().cloned().sum::<Exact>()
        + &correction_income
        + &correction_employer_principal_income
        + &excess_employer_other_income
        + last_month_income.iter().cloned().sum::<Exact>();
    let last_month_income = [
        crate::medical_account::required_output(
            &exact::dollars_exact(&last_month_income[0]),
            crate::IssueContext::owner(crate::TaxSection::Hsa, Owner::Taxpayer),
        )?,
        crate::medical_account::required_output(
            &exact::dollars_exact(&last_month_income[1]),
            crate::IssueContext::owner(crate::TaxSection::Hsa, Owner::Spouse),
        )?,
    ];
    let funding_income = [
        crate::medical_account::required_output(
            &exact::dollars_exact(&funding_income[0]),
            crate::IssueContext::owner(crate::TaxSection::Hsa, Owner::Taxpayer),
        )?,
        crate::medical_account::required_output(
            &exact::dollars_exact(&funding_income[1]),
            crate::IssueContext::owner(crate::TaxSection::Hsa, Owner::Spouse),
        )?,
    ];
    let mut testing_tax = [0; 2];
    for owner in 0..2 {
        let combined = funding_income[owner] + last_month_income[owner];
        let context = crate::IssueContext::owner(
            crate::TaxSection::Hsa,
            if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
        );
        crate::medical_account::required_output(&(whole(combined) * whole(100)), context)?;
        let tax = crate::kernel::ratio::fraction(combined, 1, 10);
        if combined > 0 {
            emit_lines(
                r,
                &mut (),
                [
                    ("testing_last_month_income", "18", last_month_income[owner]),
                    ("testing_funding_income", "19", funding_income[owner]),
                    ("testing_total_income", "20", combined),
                    ("testing_tax", "21", tax),
                ],
                |(suffix, printed, amount), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(crate::TaxForm::Form8889, printed)),
                        &alloc::format!("hsa_{}_{suffix}", owner + 1),
                        LineValue::MoneyDollars(amount),
                        &[
                            "adjustments.hsa.last_month_history",
                            "adjustments.hsa.funding_history",
                            "adjustments.hsa.funding",
                        ],
                        "https://www.irs.gov/pub/irs-dft/i8889--dft.pdf",
                        msg!("EXPLAIN_HSA_TESTING_OWNER_FORM"),
                    ))
                },
            );
        }
        testing_tax[owner] = tax;
    }
    let output_context = crate::IssueContext::Section(crate::TaxSection::Hsa);
    for owner in 0..2 {
        let context = crate::IssueContext::owner(
            crate::TaxSection::Hsa,
            if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
        );
        sources.distribution_income[owner] = crate::medical_account::required_output(
            &exact::dollars_exact(&taxable[owner]),
            context.clone(),
        )?;
        sources.distribution_tax[owner] = crate::medical_account::required_output(
            &exact::rounded_exact(&(exact::dollars_exact(&penalized[owner]) / whole(5))),
            context,
        )?;
    }
    sources.funding_income = funding_income;
    sources.last_month_income = last_month_income;
    sources.testing_tax = testing_tax;
    let required_dollars = |cents: &Exact| {
        let dollars = exact::dollars_exact(cents);
        crate::medical_account::required_output(
            &(dollars.clone() * whole(100)),
            output_context.clone(),
        )?;
        crate::medical_account::required_output(&dollars, output_context.clone())
    };
    let required_sum = |values: [i64; 2]| {
        let dollars = whole(values[0]) + whole(values[1]);
        crate::medical_account::required_output(
            &(dollars.clone() * whole(100)),
            output_context.clone(),
        )?;
        crate::medical_account::required_output(&dollars, output_context.clone())
    };
    let income = required_sum(sources.distribution_income)?;
    let tax = required_sum(sources.distribution_tax)?;
    // Schedule 1 line 13 is the sum of the printed Form 8889 line 13 amounts.
    let deduction_by_owner = printed_deduction.map(|dollars| dollars * 100);
    crate::medical_account::required_output(
        &(whole(excess_tax) * whole(100)),
        output_context.clone(),
    )?;
    Ok(CalculationResult {
        sources,
        deduction: deduction_by_owner[0] + deduction_by_owner[1],
        income,
        tax,
        recapture: required_sum(funding_income)?,
        correction_income: required_dollars(&correction_income)?,
        correction_employer_income: required_dollars(&correction_employer_principal_income)?,
        excess_employer_income: required_dollars(&excess_employer_other_income)?,
        excess_tax,
        last_month_income: required_sum(last_month_income)?,
        testing_tax: required_sum(testing_tax)?,
        work_income_cents,
        deduction_by_owner,
    })
}

impl HsaFundingHistory {
    fn recapture(&self) -> i64 {
        if self.loss.as_ref().is_some_and(|v| {
            v.reason == HsaLossReason::Other
                && v.date / 10000 == 2026
                && v.date / 100 <= self.funded / 100 + 100
        }) {
            self.amount
        } else {
            0
        }
    }
}
