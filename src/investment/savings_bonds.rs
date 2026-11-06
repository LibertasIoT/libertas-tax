// Coverage: Form 8815 and Schedule B lines 2-4 (series EE and I savings bond interest excluded
//   for higher education, IRC 135) — Supported (audited 2026-10-09, unit B1 Form 8815).
//   Lines: 2 (tuition and fees plus 529 or Coverdell contributions, less the expenses an education
//   credit used), 3 (tax-free educational benefits: scholarships and refunds, and for a person with
//   a Form 1099-Q the expenses the 529 or Coverdell exclusion used, education_account.rs, never the
//   credit entry's own 529 allocation too), 4, 5 and 6 (Form 8818 lines 6 and 8), 7 and 8
//   (the proportional rule, exact), 9 (modified AGI: AGI before the student loan deduction and
//   after every other adjustment, plus the foreign earned income and adoption exclusions), 10-13
//   (the phaseout, exact), 14 (to Schedule B line 3); Schedule B line 2 (gross interest) and line 4
//   (interest less the exclusion, shown whenever the exclusion is positive). The entry is one
//   savings-bond record on the Interest and dividends page (InterestDividendDocument::SavingsBonds, imported
//   from Form 8818: proceeds, interest, earliest issue date) with one row per person whose
//   education was paid for. Exclusions: married filing separately and an owner under 24 when the
//   bond was issued take no exclusion (a note, the estimate proceeds); a bond issued before 1990
//   is refused on save; prior inclusion comes from the canonical linked statement or annual instrument
//   and reduces line 6 once, without reducing proceeds; a mixed source with no qualifying-bond
//   prior-inclusion allocation omits only the education exclusion with a named notice; the credit's consumed expenses fall on the course materials first, so line 2
//   loses only the tuition the credit needed (EducationUse::used_tuition; i8815 line 2 counts
//   tuition and fees only, Form 1099-Q reads the whole EducationUse::used); no other cross-benefit
//   optimization. Form 8815 line 1(b) (the institution's name and address) is filing-package text.
//   Shared owner: FederalCredits::education_expenses_used (federal_model.rs) is the one definition
//   of the expenses an education credit takes into account; Form 8863 figures its credits from it
//   and the exclusion reads it. Constants: year_constants.rs (Rev. Proc. 2025-32 section 3.17).
//   Sources: https://www.irs.gov/pub/irs-dft/f8815--dft.pdf (2026 draft), i8815 (Jul 8, 2026
//   draft), https://www.irs.gov/pub/irs-dft/f1040sb--dft.pdf, 26 U.S.C. 135, Form 8818.
//   Tests: src/tests/investment/investment.rs savings_bond_*, src/tests/credits/credits.rs savings_bond_tuition_*,
//   src/tests/family/education_accounts.rs form_8815_counts_the_expenses_the_529_exclusion_used_once,
//   src/tests/investment/interview.rs savings_bonds_*; unit tests below.
use crate::federal_model::EducationUse;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::year_constants::{
    SAVINGS_BOND_OWNER_MINIMUM_AGE, SAVINGS_BOND_PHASEOUT_RANGE_JOINT_DOLLARS,
    SAVINGS_BOND_PHASEOUT_RANGE_OTHER_DOLLARS, SAVINGS_BOND_PHASEOUT_START_JOINT_DOLLARS,
    SAVINGS_BOND_PHASEOUT_START_OTHER_DOLLARS,
};
use crate::{
    FederalCredits, FederalIncome, FederalIncomeEntry, FederalPeople, FederalResult,
    FederalSavingsBondExpenses, FederalSavingsBonds, FederalStatus, FormLine, Owner, TaxForm,
    TaxSection,
};
use crate::{federal_rules::note_once_scoped, line_emission::form_line};
use alloc::vec::Vec;

// Form 8815 line 6 worksheet, 2026 DRAFT instructions printed July 8, 2026,
// p.2: subtract prior inclusion from redemption interest, never from proceeds.
// Parent statement identity is used only without annual records. An annual
// instrument has its own accepted identity, so separate bonds never share relief.
#[derive(Clone)]
pub(crate) struct BondSource {
    pub id: i64,
    pub owner: Owner,
    pub label: alloc::string::String,
    pub gross: i64,
    pub prior: i64,
}
pub(crate) fn sources(income: &[FederalIncomeEntry]) -> Vec<BondSource> {
    let mut result = Vec::new();
    for e in income {
        let FederalIncome::Interest { data } = &e.income else {
            continue;
        };
        let data = &crate::FederalInterest::from(data);
        if let Some(records) = data.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref())
        {
            for r in &records.records {
                if r.id > 0
                    && r.amounts.us_obligations >= 0
                    && r.treatment == crate::AnnualDebtTreatment::Regular
                {
                    result.push(BondSource {
                        id: r.id,
                        owner: e.owner.owned(),
                        label: alloc::format!("{}: {}", e.label, r.instrument),
                        gross: r.retained().us_obligations + r.previously_reported,
                        prior: r.previously_reported,
                    });
                }
            }
        } else if e.id > 0 && data.us_obligations >= 0 {
            result.push(BondSource {
                id: e.id,
                owner: e.owner.owned(),
                label: e.label.clone(),
                gross: data.us_obligations
                    - data.nominee.as_ref().map_or(0, |n| n.us_obligations)
                    - data.after_death.as_ref().map_or(0, |a| a.us_obligations),
                prior: data.adjustments.as_ref().map_or(0, |a| a.previously_reported),
            });
        }
    }
    result
}
pub(crate) fn source_choices(income: &[FederalIncomeEntry]) -> Vec<crate::RecordChoice> {
    sources(income)
        .into_iter()
        .filter(|s| s.gross > 0)
        .map(|s| crate::RecordChoice {
            id: s.id,
            label: crate::Message::new(
                "BOND_PREMIUM_INSTRUMENT_LABEL",
                alloc::vec![
                    crate::MessageArg::Text(s.label),
                    crate::MessageArg::Text(alloc::string::String::from("Form 8815"))
                ],
            )
            .encode(),
        })
        .collect()
}
pub(crate) fn source_rule(income: &[FederalIncomeEntry]) -> Option<&'static str> {
    source_issue(income).map(|(_, issue)| issue.key)
}
pub(crate) fn source_issue(
    income: &[FederalIncomeEntry],
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let sources = sources(income);
    for e in income {
        if let FederalIncome::SavingsBonds { data } = &e.income
            && let Some(answers) = &data.answers
            && !sources.iter().any(|s| {
                s.id == answers.interest_statement
                    && s.owner == e.owner.owned()
                    && data.redemption_interest.is_none_or(|interest| interest <= s.gross)
            })
        {
            return Some((
                e.id,
                crate::validation::ValidationIssue::new("SAVINGS_BONDS_SOURCE_INVALID")
                    .at_field("interest_statement"),
            ));
        }
    }
    None
}
// Called once in the central calculation projection, after capturing the
// canonical prior-inclusion facts and before education's MAGI iteration.
pub(crate) fn normalize(
    income: &mut Vec<FederalIncomeEntry>,
    sources: &[BondSource],
    notes: &mut Vec<crate::federal_model::ReviewNote>,
) -> bool {
    let mut financial_omission = false;
    income.retain_mut(|entry| {
        let context = crate::interview::income_issue_context(entry);
        let FederalIncome::SavingsBonds { data } = &mut entry.income else {
            return true;
        };
        let Some(boxes) = data.record() else { return true };
        let source = sources
            .iter()
            .find(|s| s.id == boxes.interest_statement && s.owner == entry.owner.owned());
        let Some(source) = source else { return true };
        if !boxes.valid() || boxes.interest > source.gross {
            return true;
        }
        if source.prior == 0 || boxes.interest == source.gross {
            *data.redemption_interest.as_mut().unwrap() -= source.prior;
            return true;
        }
        financial_omission = true;
        let message = crate::Message::new("WARN_SAVINGS_BONDS_PRIOR_ALLOCATION", alloc::vec![]);
        notes.push(crate::federal_model::ReviewNote::scoped(
            #[cfg(test)]
            message.english(),
            message,
            context,
        ));
        false
    });
    financial_omission
}

// A redemption paper supplies evidence, never another interest-income entry.
// Form 8818 instructions require one record for each cashing; Form 8815 line 5
// combines them. Resolve only an explicitly linked complete set of proceeds.
struct RedemptionFacts {
    target: i64,
    owner: Owner,
    proceeds: i64,
    interest: i64,
    earliest: Option<u32>,
}
fn redemption_facts(income: &[FederalIncomeEntry]) -> Vec<RedemptionFacts> {
    income
        .iter()
        .filter_map(|entry| {
            let FederalIncome::BondRedemption { data } = &entry.income else { return None };
            Some(RedemptionFacts {
                target: data.form_8815?,
                owner: entry.owner.tax_owner()?,
                proceeds: data.form.line6,
                interest: data.form.line8,
                earliest: data.form.earliest_issue_date(),
            })
        })
        .collect()
}
fn resolve_redemptions(
    data: &mut crate::SavingsBondsSource,
    id: i64,
    owner: crate::DocumentOwner,
    redemptions: &[RedemptionFacts],
) {
    let mut linked = redemptions
        .iter()
        .filter(|row| row.target == id && Some(row.owner) == owner.tax_owner())
        .peekable();
    if linked.peek().is_none() {
        return;
    }
    let totals =
        linked.try_fold((0i64, 0i64, Some(u32::MAX)), |(proceeds, interest, earliest), row| {
            Some((
                proceeds.checked_add(row.proceeds)?,
                interest.checked_add(row.interest)?,
                match (earliest, row.earliest) {
                    (Some(prior), Some(date)) => Some(prior.min(date)),
                    _ => None,
                },
            ))
        });
    if let Some((proceeds, interest, earliest)) = totals
        && proceeds == data.form.line5
    {
        data.redemption_interest = Some(interest);
        data.earliest_issue_date = earliest;
    } else {
        data.redemption_interest = None;
        data.earliest_issue_date = None;
    }
}
pub(crate) fn apply_redemptions(income: &mut [FederalIncomeEntry]) {
    let redemptions = redemption_facts(income);
    for entry in income {
        if let FederalIncome::SavingsBonds { data } = &mut entry.income {
            resolve_redemptions(data, entry.id, entry.owner, &redemptions);
        }
    }
}
pub(crate) fn resolved_source(
    income: &[FederalIncomeEntry],
    id: i64,
) -> Option<crate::SavingsBondsSource> {
    let entry = income.iter().find(|entry| entry.id == id)?;
    let FederalIncome::SavingsBonds { data } = &entry.income else { return None };
    let mut resolved = data.clone();
    resolve_redemptions(&mut resolved, id, entry.owner, &redemption_facts(income));
    Some(resolved)
}

pub(crate) fn redemption_link_issue(
    income: &[FederalIncomeEntry],
) -> Option<(i64, crate::validation::ValidationIssue)> {
    income.iter().find_map(|entry| {
        let FederalIncome::BondRedemption { data } = &entry.income else { return None };
        let target = data.form_8815?;
        (!income.iter().any(|candidate| {
            candidate.id == target
                && entry.owner != crate::DocumentOwner::Unknown
                && candidate.owner == entry.owner
                && matches!(candidate.income, FederalIncome::SavingsBonds { .. })
        }))
        .then(|| {
            (
                entry.id,
                crate::validation::ValidationIssue::new("BOND_REDEMPTION_FORM_8815_INVALID")
                    .at_field("form_8815"),
            )
        })
    })
}

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f8815--dft.pdf";

// What the Form 8815 calculation reads from the rest of the return.
pub(crate) struct SavingsBondContext<'a> {
    // The supported working copy's income entries.
    pub(crate) income: &'a [FederalIncomeEntry],
    pub(crate) credits: &'a FederalCredits,
    // The people on the return, the students an education entry can name.
    pub(crate) students: &'a [crate::PersonChoice],
    pub(crate) people: &'a FederalPeople,
    pub(crate) status: FederalStatus,
    // The dependents claimed on this return, by person identity.
    pub(crate) dependent_ids: &'a [i64],
    // The App allows an education credit (not married filing separately, not
    // another taxpayer's dependent).
    pub(crate) credit_allowed: bool,
    // The 529 and Coverdell distributions of the same return: the expenses
    // their tax-free earnings used are line 3 benefits, in place of the amount
    // an education credit entry allocates to them (never both).
    pub(crate) education: &'a crate::education_account::EducationTotals,
    // Form 8815 line 9, whole dollars.
    pub(crate) magi: i64,
}

// The savings-bond entry of the working copy, if it has one.
pub(crate) fn record(
    income: &[FederalIncomeEntry],
) -> Option<(&FederalIncomeEntry, FederalSavingsBonds)> {
    income.iter().find_map(|entry| match &entry.income {
        FederalIncome::SavingsBonds { data } => data.record().map(|record| (entry, record)),
        _ => None,
    })
}

// Form 8815 line 8: the interest in the proceeds, in the proportion of the
// qualified expenses to the proceeds (line 7, kept exact instead of rounded
// to three places).
pub(crate) fn eligible_interest(net_expenses: i64, proceeds: i64, interest: i64) -> i64 {
    if net_expenses >= proceeds { interest } else { fraction(interest, net_expenses, proceeds) }
}

// Form 8815 line 13: the part of line 8 the modified AGI phases out
// (lines 10 to 12, exact).
pub(crate) fn phaseout_reduction(eligible: i64, magi: i64, joint: bool) -> i64 {
    let (start, range) = if joint {
        (SAVINGS_BOND_PHASEOUT_START_JOINT_DOLLARS, SAVINGS_BOND_PHASEOUT_RANGE_JOINT_DOLLARS)
    } else {
        (SAVINGS_BOND_PHASEOUT_START_OTHER_DOLLARS, SAVINGS_BOND_PHASEOUT_RANGE_OTHER_DOLLARS)
    };
    phase::linear_reduction(eligible, magi, start, range)
}

// The Form 8863 entry of a person, if the person has one.
pub(crate) fn education_entry(credits: &FederalCredits, person: i64) -> Option<usize> {
    credits.education_inputs().iter().position(|e| e.student == person)
}

// Form 8815 and its Schedule B line 3 amount (whole dollars); 0 without a
// savings-bond record. The lines are emitted here; Schedule B lines 2 and 4
// follow from the caller's gross interest.
pub(crate) fn calculate(cx: &SavingsBondContext, r: &mut FederalResult) -> i64 {
    let Some((entry, data)) = record(cx.income) else {
        return 0;
    };
    let context = crate::interview::income_issue_context(entry);
    let joint = cx.status == FederalStatus::Joint;
    let used = cx.credits.education_expenses_used(cx.students, cx.credit_allowed);
    let mut expenses = 0;
    let mut benefits = 0;
    for student in &data.students {
        crate::require_localized!(
            r,
            student.person == -1
                || (student.person == -2 && joint)
                || cx.dependent_ids.contains(&student.person),
            &context.clone().alternatives(&[TaxSection::InterestDividends, TaxSection::People]),
            "Savings bonds",
            "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D",
            "The selected student must be a filer or an eligible dependent on this return.",
        );
        let credit_entry = education_entry(cx.credits, student.person);
        let (tuition, received, credit_used) = match (&student.expenses, credit_entry) {
            (FederalSavingsBondExpenses::EducationCredit, Some(index)) => {
                let e = &cx.credits.education_inputs()[index].expenses;
                // The expenses the 529 or Coverdell distributions used come
                // from the Form 1099-Q calculation when the person has one;
                // the credit entry's own allocation otherwise.
                let tax_free_529 = cx.education.used_by(student.person).unwrap_or(e.other_benefits);
                (
                    e.tuition,
                    e.tax_free_assistance() + e.refunds + tax_free_529,
                    used[index].used_tuition,
                )
            }
            (FederalSavingsBondExpenses::EducationCredit, None) => {
                crate::require_localized!(
                    r,
                    false,
                    &context
                        .clone()
                        .alternatives(&[TaxSection::InterestDividends, TaxSection::Education]),
                    "Savings bonds",
                    "ISSUE_SAVINGS_BOND_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY",
                    "A savings bond student's expenses come from an education credit entry, but this person has none. Add the entry on Education credits or enter the expenses on the savings bond record.",
                );
                (0, 0, 0)
            }
            (FederalSavingsBondExpenses::Entered { tuition, benefits }, entry) => {
                crate::require_localized!(
                    r,
                    entry.is_none(),
                    &context
                        .clone()
                        .alternatives(&[TaxSection::InterestDividends, TaxSection::Education]),
                    "Savings bonds",
                    "ISSUE_SAVINGS_BOND_STUDENT_EXPENSES_ENTERED_TWICE",
                    "This person's tuition is entered on Education credits and again on the savings bond record. Enter it once: choose the education credit entry here, or remove the credit entry.",
                );
                (*tuition, *benefits, entry.map_or(0, |index| used[index].used_tuition))
            }
        };
        expenses += positive(dollars(tuition + student.contributions) - credit_used);
        benefits += dollars(received);
    }
    let linked = sources(cx.income)
        .into_iter()
        .find(|s| s.id == data.interest_statement && s.owner == entry.owner.owned());
    crate::require_localized!(
        r,
        linked.is_some_and(|s| data.interest <= s.gross),
        &context,
        "Savings bonds",
        "SAVINGS_BONDS_SOURCE_INVALID",
        "Select the accepted interest statement or annual instrument for these bonds, with the same owner and enough retained redemption interest. A linked source cannot be removed while this record remains.",
    );
    let net_expenses = positive(expenses - benefits);
    let proceeds = dollars(data.proceeds);
    let interest = phase::cap(dollars(data.interest), [proceeds]);
    let eligible = eligible_interest(net_expenses, proceeds, interest);
    let inputs: &[&str] = &["income.savings_bonds", "credits.education"];
    emit_lines(
        r,
        &mut (),
        [
            (
                "2",
                "form_8815_qualified_expenses",
                expenses,
                msg!("EXPLAIN_FORM_8815_QUALIFIED_EXPENSES"),
            ),
            (
                "3",
                "form_8815_educational_benefits",
                benefits,
                msg!("EXPLAIN_FORM_8815_EDUCATIONAL_BENEFITS"),
            ),
            ("4", "form_8815_net_expenses", net_expenses, msg!("EXPLAIN_FORM_8815_NET_EXPENSES")),
            ("5", "form_8815_proceeds", proceeds, msg!("EXPLAIN_FORM_8815_PROCEEDS")),
            ("6", "form_8815_interest", interest, msg!("EXPLAIN_FORM_8815_INTEREST")),
            (
                "8",
                "form_8815_eligible_interest",
                eligible,
                msg!("EXPLAIN_FORM_8815_ELIGIBLE_INTEREST"),
            ),
        ],
        |(line, id, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form8815, line)),
                id,
                LineValue::MoneyDollars(amount),
                inputs,
                SOURCE,
                explanation,
            ))
        },
    );
    // IRC 135(d)(3): no exclusion on a separate return. The record stays and
    // the interest stays taxable.
    let separate = cx.status == FederalStatus::Separate;
    if separate {
        note_once_scoped(
            r,
            msg!("NOTE_SAVINGS_BONDS_EXCLUSION_MFS"),
            context.clone().alternatives(&[TaxSection::Setup, TaxSection::InterestDividends]),
        );
    }
    // IRC 135(c)(1)(B): the owner had attained age 24 before the bonds were
    // issued, tested on the earliest issue date.
    let owner = cx.people.person(entry.owner.owned());
    let owner_section = match entry.owner.owned() {
        crate::Owner::Spouse => TaxSection::Spouse,
        crate::Owner::Taxpayer => TaxSection::People,
    };
    let owner_old_enough = owner.is_none_or(|person| {
        crate::federal_model::attained_on(
            person.birth_date,
            SAVINGS_BOND_OWNER_MINIMUM_AGE,
            false,
            data.earliest_issue_date,
        )
    });
    if !separate && !owner_old_enough {
        note_once_scoped(
            r,
            msg!("NOTE_SAVINGS_BONDS_OWNER_UNDER_24"),
            context.clone().alternatives(&[TaxSection::InterestDividends, owner_section]),
        );
    }
    let (reduction, exclusion, explanation) = if separate || !owner_old_enough {
        (0, 0, msg!("EXPLAIN_SAVINGS_BOND_EXCLUSION_NOT_ALLOWED"))
    } else {
        let reduction = phaseout_reduction(eligible, cx.magi, joint);
        form_line(
            r,
            FormLine::new(TaxForm::Form8815, "9"),
            "form_8815_modified_agi",
            cx.magi,
            &["agi", "adjustments", "income"],
            SOURCE,
            msg!("EXPLAIN_FORM_8815_MODIFIED_AGI"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::Form8815, "13"),
            "form_8815_phaseout_reduction",
            reduction,
            &["form_8815_eligible_interest", "form_8815_modified_agi"],
            SOURCE,
            msg!("EXPLAIN_FORM_8815_PHASEOUT_REDUCTION"),
        );
        (
            reduction,
            eligible - reduction,
            if reduction == eligible && eligible > 0 {
                msg!("EXPLAIN_SAVINGS_BOND_EXCLUSION_PHASED_OUT")
            } else {
                msg!("EXPLAIN_SAVINGS_BOND_EXCLUSION")
            },
        )
    };
    debug_assert!(reduction <= eligible);
    form_line(
        r,
        FormLine::new(TaxForm::Form8815, "14"),
        "savings_bond_exclusion",
        exclusion,
        &["form_8815_eligible_interest", "form_8815_phaseout_reduction"],
        SOURCE,
        explanation,
    );
    exclusion
}

// The note for a person whose education credit computes to zero but still
// holds expenses the credit would use: the exclusion cannot reuse them. Called
// once the credits are final; `aotc` and `llc` are the credits after the
// Form 8863 income phaseout.
pub(crate) fn note_credit_expenses(
    cx: &SavingsBondContext,
    aotc: i64,
    llc: i64,
    r: &mut FederalResult,
) {
    let Some((entry, data)) = record(cx.income) else {
        return;
    };
    let used: Vec<EducationUse> =
        cx.credits.education_expenses_used(cx.students, cx.credit_allowed);
    let wasted = data.students.iter().any(|student| {
        matches!(student.expenses, FederalSavingsBondExpenses::EducationCredit)
            && education_entry(cx.credits, student.person).is_some_and(|index| {
                let credit = match used[index].method {
                    crate::EducationMethod::AmericanOpportunity => aotc,
                    crate::EducationMethod::LifetimeLearning => llc,
                };
                used[index].used > 0 && credit == 0
            })
    });
    if wasted {
        note_once_scoped(
            r,
            msg!("NOTE_SAVINGS_BONDS_CREDIT_EXPENSES"),
            crate::interview::income_issue_context(entry)
                .alternatives(&[TaxSection::InterestDividends, TaxSection::Education]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Form 8815 line 7 is the exact fraction of line 4 to line 5.
    #[test]
    fn eligible_interest_is_exact_and_capped() {
        // Line 4 at or above line 5: line 7 is 1.000.
        assert_eq!(eligible_interest(12_000, 12_000, 3_000), 3_000);
        assert_eq!(eligible_interest(50_000, 12_000, 3_000), 3_000);
        // 4,000 x 6,000 / 10,000 = 2,400.
        assert_eq!(eligible_interest(6_000, 10_000, 4_000), 2_400);
        // 1,000 x 1 / 3 = 333.33, rounded to the dollar.
        assert_eq!(eligible_interest(1, 3, 1_000), 333);
        assert_eq!(eligible_interest(2, 3, 1_000), 667);
        assert_eq!(eligible_interest(0, 3, 1_000), 0);
    }

    // Lines 10 to 13: nothing below the start, the exact fraction inside the
    // range (rounded half up to the dollar), everything at the top.
    #[test]
    fn phaseout_reduction_is_exact_at_both_ends_of_both_columns() {
        for (joint, start, range) in [(false, 101_800, 15_000), (true, 152_650, 30_000)] {
            assert_eq!(phaseout_reduction(2_000, start, joint), 0);
            assert_eq!(phaseout_reduction(2_000, start - 5_000, joint), 0);
            assert_eq!(phaseout_reduction(2_000, start + range, joint), 2_000);
            assert_eq!(phaseout_reduction(2_000, start + range + 9_000, joint), 2_000);
            // Half way through the range: half of the interest.
            assert_eq!(phaseout_reduction(2_000, start + range / 2, joint), 1_000);
        }
        // 2,000 x 4,200 / 15,000 = 560.0 exactly.
        assert_eq!(phaseout_reduction(2_000, 106_000, false), 560);
        // 2,000 x 14,996 / 15,000 = 1,999.47 stays 1,999; 14,997 gives
        // 1,999.6 -> 2,000 (half up on the exact quotient 1,999.6).
        assert_eq!(phaseout_reduction(2_000, 101_800 + 14_996, false), 1_999);
        assert_eq!(phaseout_reduction(2_000, 101_800 + 14_997, false), 2_000);
    }
}
