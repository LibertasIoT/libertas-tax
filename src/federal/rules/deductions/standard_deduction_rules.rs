// standard_deduction items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Publication 501 Table 8, using Rev. Proc. 2025-32 amounts for 2026.
// Earned income here includes taxable scholarships;
// it is not the EIC definition for that source.
pub(super) fn standard_base(status: FederalStatus) -> i64 {
    match status {
        FederalStatus::Single | FederalStatus::Separate => {
            crate::year_constants::STANDARD_DEDUCTION_SINGLE_OR_SEPARATE_DOLLARS
        }
        FederalStatus::Head => crate::year_constants::STANDARD_DEDUCTION_HEAD_DOLLARS,
        _ => crate::year_constants::STANDARD_DEDUCTION_JOINT_DOLLARS,
    }
}
// The Standard Deduction Worksheet for Dependents applies "only if someone
// can claim you, or your spouse if filing jointly, as a dependent" (Form
// 1040 instructions, line 12e); the spouse is present only on a joint
// return. Its earned income counts Schedule C and F profit whatever
// produced it (Schedule 1 lines 3 and 6), so only a partnership's income
// source enters it.
pub(crate) fn dependent_standard(p: &FederalPeople) -> bool {
    core::iter::once(&p.taxpayer).chain(p.spouse.iter()).any(|person| person.dependency.claimable())
}
// A separate return's spouse boxes (Form 1040 line 12d) matter only while
// the standard deduction can apply: not when the spouse itemizes (the boxes
// need a spouse who does not file) and not when the return itemizes.
pub(crate) fn spouse_boxes_apply(d: &FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
        && d.deductions.as_ref().is_some_and(|ded| ded.spouse_itemizes == Some(false))
        && d.deductions.as_ref().is_none_or(|v| v.itemize_override.is_none())
}
pub(crate) fn standard_deduction(
    s: &FederalSetup,
    p: &FederalPeople,
    earned: i64,
    spouse_itemizes: bool,
) -> i64 {
    let mut standard = standard_base(s.filing_status());
    if dependent_standard(p) {
        standard = phase::cap(
            standard,
            [phase::floor(
                crate::year_constants::STANDARD_DEDUCTION_DEPENDENT_MINIMUM_DOLLARS,
                positive(earned)
                    + crate::year_constants::STANDARD_DEDUCTION_DEPENDENT_EARNED_ADDITION_DOLLARS,
            )],
        );
    }
    let additional = if matches!(s.filing_status(), FederalStatus::Single | FederalStatus::Head) {
        crate::year_constants::STANDARD_DEDUCTION_AGE_BLIND_UNMARRIED_DOLLARS
    } else {
        crate::year_constants::STANDARD_DEDUCTION_AGE_BLIND_MARRIED_DOLLARS
    };
    for person in core::iter::once(&p.taxpayer).chain(p.spouse.iter()) {
        standard += additional * (i64::from(person.senior()) + i64::from(person.blind));
    }
    if s.filing_status() == FederalStatus::Separate {
        standard += additional * s.separate_spouse.count();
    }
    if spouse_itemizes {
        standard = 0;
    }
    standard
}

pub(super) struct StandardDeductionIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) earned: [i64; 2],
    pub(super) dependent_standard: bool,
    pub(super) partner_standard_complete: bool,
    pub(super) modify_income: bool,
    pub(super) strike_benefits: i64,
    pub(super) deferred_pension: i64,
    pub(super) inmate_pay: i64,
    pub(super) scholarship: i64,
    pub(super) partner_standard_earned: i64,
    pub(super) foreign_magi: i64,
}

pub(super) struct StandardDeductionOut {
    pub(super) standard: i64,
    pub(super) senior_ded: i64,
}

pub(super) fn standard_deduction_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: StandardDeductionIn<'_>,
) -> StandardDeductionOut {
    let &Pass { d, s, p, .. } = pass;
    let StandardDeductionIn {
        rental,
        earned,
        dependent_standard,
        partner_standard_complete,
        modify_income,
        strike_benefits,
        deferred_pension,
        inmate_pay,
        scholarship,
        partner_standard_earned,
        foreign_magi,
    } = input;
    // The Standard Deduction Worksheet for Dependents takes line 1z plus Schedule 1 lines 3,
    // 6, 8r, 8t and 8u less line 15 (2025 Instructions for Form 1040, footnote); section
    // 63(c)(5)(B) earned income is section 32(c)'s, whose self-employment earnings include a
    // loss, so a business loss reduces it.
    let standard_earned = earned.iter().sum::<i64>() - strike_benefits
        + deferred_pension
        + inmate_pay
        + scholarship
        + if dependent_standard {
            partner_standard_earned
                - rental.partner_se.iter().map(|(_, se)| se.earnings).sum::<i64>()
        } else {
            0
        };
    crate::require_localized!(
        r,
        !(s.filing_status() == FederalStatus::Separate
            && s.separate_spouse.count() > 0
            && d.spouse_itemizes()),
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Separate spouse standard deduction boxes",
        "ISSUE_SEPARATE_SPOUSE_BOXES_WITH_SPOUSE_ITEMIZING",
        "Spouse standard deduction boxes require a spouse who is not filing, so cannot be combined with a spouse who itemizes.",
    );
    crate::require_localized!(
        r,
        !spouse_boxes_apply(d) || s.separate_spouse != crate::SeparateSpouseBoxes::NotAnswered,
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Separate spouse standard deduction boxes",
        "ISSUE_ANSWER_YOUR_SPOUSE_STANDARD_DEDUCTION_BOXES",
        "Answer your spouse's standard deduction boxes: choose None of these unless your spouse had no gross income, is not filing and cannot be claimed as a dependent.",
    );
    crate::require_localized!(
        r,
        s.filing_status() != FederalStatus::Separate
            || d.deductions.as_ref().is_some_and(|ded| ded.spouse_itemizes.is_some()),
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Separate spouse itemizing",
        "ISSUE_ANSWER_SPOUSE_ITEMIZES",
        "Answer whether your spouse itemizes on their separate return.",
    );
    let standard = standard_deduction(s, p, standard_earned, d.spouse_itemizes());
    if dependent_standard
        && partner_standard_complete
        && !rental.partners.is_empty()
        && rental.inputs_complete
    {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("Dependent standard-deduction earned income from partnership sources: ${partner_standard_earned}; total earned income after the shared half-SE deduction: ${standard_earned}; standard deduction: ${standard}. The source amount follows personal services rather than K-1 box 14 classification."),
            Message::new("NOTE_DEPENDENT_PARTNERSHIP_STANDARD_DEDUCTION", alloc::vec![
                MessageArg::usd_dollars(partner_standard_earned), MessageArg::usd_dollars(standard_earned),
                MessageArg::usd_dollars(standard),
            ]),
        ));
    }

    let senior_count = core::iter::once(&p.taxpayer)
        .chain(p.spouse.iter())
        .filter(|v| v.senior() && v.valid_ssn)
        .count() as i64;
    let senior_ded = if modify_income {
        0
    } else {
        crate::extended_rules::senior_deduction(s.filing_status(), senior_count, foreign_magi)
    };
    StandardDeductionOut { standard, senior_ded }
}
