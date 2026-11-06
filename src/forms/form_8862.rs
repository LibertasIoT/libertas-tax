#[allow(unused_imports)]
pub(crate) use crate::disallowance_source::{
    Bans, NoFormReason, Requirement, TAX_YEAR, bans, requirement,
};
pub use crate::disallowance_source::{
    CreditDisallowance, CreditsAfterDisallowance, DisallowanceFacts, DisallowanceFinding,
    EicDisallowance, EicDisallowanceCause,
};
// Form 8862: whether a previously reduced or disallowed EIC, CTC/ACTC/ODC or
// AOTC may be claimed again for 2026, and the form's Parts I–IV filled from
// the return's own people, children and students.
use crate::line_emission::form_line;
use crate::{
    DependentResidency, EducationMethod, FederalDependent, FederalPeople, FederalResult, FormLine,
    Message, MessageArg, PersonChoice, TaxForm,
};
use alloc::{format, vec::Vec};

const SOURCE: &str = "https://www.irs.gov/instructions/i8862";
const INPUTS: &[&str] = &["credits"];
#[cfg(test)]
const SECTION: &str = "More credits";

// What the return supplies to Form 8862.
pub(crate) struct Form8862Context<'a> {
    pub(crate) claim: &'a CreditsAfterDisallowance,
    pub(crate) people: &'a FederalPeople,
    pub(crate) dependents: &'a [FederalDependent],
    // The Schedule EIC qualifying children, the CTC children and the ODC
    // people, by dependent id in dependents order (family()).
    pub(crate) eic_children: &'a [i64],
    pub(crate) ctc_children: &'a [i64],
    pub(crate) other_dependents: &'a [i64],
    pub(crate) education: &'a [crate::federal_model::EducationAnnualProjection],
    pub(crate) students: &'a [PersonChoice],
    pub(crate) joint: bool,
    // Whether the return claims each credit group at all (line 27a, Schedule
    // 8812 line 14 or 27 potential, Form 8863 Part I).
    pub(crate) eic_claimed: bool,
    pub(crate) child_credits_claimed: bool,
    pub(crate) aotc_claimed: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum CreditGroup {
    EarnedIncome,
    Child,
    Education,
}

impl NoFormReason {
    #[cfg(test)]
    fn english(self) -> &'static str {
        match self {
            Self::MathError => "the earlier reduction was a math or clerical error only",
            Self::Reinstated => {
                "the credit was allowed again after Form 8862 and has not been disallowed since"
            }
            Self::BeforeRequirement => "the disallowed year is before the requirement began",
            Self::NotClaimed => "the return does not claim it",
            Self::ChildlessException => {
                "it is claimed without a qualifying child and the only earlier finding was that a listed child was not a qualifying child"
            }
        }
    }

    fn key(self, credit: CreditGroup) -> &'static str {
        match (credit, self) {
            (CreditGroup::EarnedIncome, Self::MathError) => "NOTE_8862_EIC_MATH_ERROR",
            (CreditGroup::EarnedIncome, Self::Reinstated) => "NOTE_8862_EIC_REINSTATED",
            (CreditGroup::EarnedIncome, Self::BeforeRequirement) => {
                "NOTE_8862_EIC_BEFORE_REQUIREMENT"
            }
            (CreditGroup::EarnedIncome, Self::NotClaimed) => "NOTE_8862_EIC_NOT_CLAIMED",
            (CreditGroup::EarnedIncome, Self::ChildlessException) => {
                "NOTE_8862_EIC_CHILDLESS_EXCEPTION"
            }
            (CreditGroup::Child, Self::MathError) => "NOTE_8862_CHILD_MATH_ERROR",
            (CreditGroup::Child, Self::Reinstated) => "NOTE_8862_CHILD_REINSTATED",
            (CreditGroup::Child, Self::BeforeRequirement) => "NOTE_8862_CHILD_BEFORE_REQUIREMENT",
            (CreditGroup::Child, Self::NotClaimed) => "NOTE_8862_CHILD_NOT_CLAIMED",
            (CreditGroup::Education, Self::MathError) => "NOTE_8862_EDUCATION_MATH_ERROR",
            (CreditGroup::Education, Self::Reinstated) => "NOTE_8862_EDUCATION_REINSTATED",
            (CreditGroup::Education, Self::BeforeRequirement) => {
                "NOTE_8862_EDUCATION_BEFORE_REQUIREMENT"
            }
            (CreditGroup::Education, Self::NotClaimed) => "NOTE_8862_EDUCATION_NOT_CLAIMED",
            (_, Self::ChildlessException) => {
                unreachable!("only the earned income credit has a childless exception")
            }
        }
    }
}

impl CreditGroup {
    #[cfg(test)]
    fn name(self) -> &'static str {
        match self {
            Self::EarnedIncome => "earned income credit",
            Self::Child => {
                "child tax credit, additional child tax credit or credit for other dependents"
            }
            Self::Education => "American opportunity tax credit",
        }
    }

    pub(crate) fn banned_key(self) -> &'static str {
        match self {
            Self::EarnedIncome => "NOTE_8862_EIC_BANNED",
            Self::Child => "NOTE_8862_CHILD_CREDITS_BANNED",
            Self::Education => "NOTE_8862_AOTC_BANNED",
        }
    }
}

// The note a ban leaves in the estimate when the return would have claimed
// the credit: the credit is left out, and the appeal is a mailed return.
pub(crate) fn ban_message(credit: CreditGroup, ban: (u32, u32)) -> Message {
    let (years, last) = ban;
    Message::new(
        credit.banned_key(),
        alloc::vec![
            MessageArg::Number(i64::from(years)),
            MessageArg::Number(i64::from(last - years)),
            MessageArg::Number(i64::from(last)),
        ],
    )
}

pub(crate) fn note_ban(r: &mut FederalResult, credit: CreditGroup, ban: (u32, u32)) {
    crate::federal_rules::note_once_scoped(
        r,
        ban_message(credit, ban),
        crate::IssueContext::Section(crate::TaxSection::MoreCredits),
    );
}

fn no_form(r: &mut FederalResult, credit: CreditGroup, reason: NoFormReason) {
    r.review_notes.push(crate::federal_model::ReviewNote::localized(
        #[cfg(test)]
        format!("Form 8862 is not needed for the {}: {}.", credit.name(), reason.english()),
        crate::Message::new(reason.key(credit), alloc::vec![]),
    ));
}

fn emit(r: &mut FederalResult, line: &str, id: &str, amount: u32, explanation: crate::Message) {
    form_line(
        r,
        FormLine::new(TaxForm::Form8862, line),
        id,
        i64::from(amount),
        INPUTS,
        SOURCE,
        explanation,
    );
}

// One credit group's gate: Some(true) when the form is needed and the
// credit is claimed, Some(false) when nothing is emitted, None when the
// claim is refused.
fn gate(
    r: &mut FederalResult,
    facts: &DisallowanceFacts,
    first_year: u32,
    credit: CreditGroup,
    claimed: bool,
) -> bool {
    match requirement(facts, first_year) {
        // A ban zeroes the group's credit; the caller states it (`note_ban`)
        // and no form is filled for it.
        Requirement::Banned(..) => false,
        Requirement::NotNeeded(reason) => {
            no_form(r, credit, reason);
            false
        }
        Requirement::Required => {
            if !claimed {
                no_form(r, credit, NoFormReason::NotClaimed);
            }
            claimed
        }
    }
}

fn dependent<'a>(ctx: &Form8862Context<'a>, id: i64) -> Option<&'a FederalDependent> {
    ctx.dependents.iter().find(|c| c.id == id)
}

pub(crate) fn compute(ctx: &Form8862Context<'_>, r: &mut FederalResult) {
    let claim = ctx.claim;
    let eic = match &claim.earned_income {
        EicDisallowance::Never => None,
        EicDisallowance::Disallowed { facts, cause, your_us_home_days, spouse_us_home_days } => {
            // The childless exception: no form when the only earlier finding
            // was that a listed child was not a qualifying child and the
            // credit is now claimed without one.
            if *cause == EicDisallowanceCause::ChildNotQualifying
                && ctx.eic_children.is_empty()
                && !matches!(
                    requirement(facts, 1996),
                    Requirement::Banned(..) | Requirement::NotNeeded(_)
                )
            {
                no_form(r, CreditGroup::EarnedIncome, NoFormReason::ChildlessException);
                None
            } else {
                gate(r, facts, 1996, CreditGroup::EarnedIncome, ctx.eic_claimed).then_some((
                    *cause,
                    *your_us_home_days,
                    *spouse_us_home_days,
                ))
            }
        }
    };
    let child_credits = match &claim.child_credits {
        CreditDisallowance::Never => false,
        CreditDisallowance::Disallowed(facts) => {
            gate(r, facts, 2015, CreditGroup::Child, ctx.child_credits_claimed)
        }
    };
    let aotc = match &claim.education_credit {
        CreditDisallowance::Never => false,
        CreditDisallowance::Disallowed(facts) => {
            gate(r, facts, 2015, CreditGroup::Education, ctx.aotc_claimed)
        }
    };
    if eic.is_none() && !child_credits && !aotc {
        return;
    }
    // Part I.
    emit(r, "1", "form_8862_line_1", TAX_YEAR, msg!("EXPLAIN_FORM_N_LINE_N_3"));
    for (needed, line_id, explanation_key) in [
        (eic.is_some(), "form_8862_line_2_eic", "EXPLAIN_8862_CHECKED_EIC"),
        (child_credits, "form_8862_line_2_child_credits", "EXPLAIN_8862_CHECKED_CHILD_CREDITS"),
        (aotc, "form_8862_line_2_aotc", "EXPLAIN_8862_CHECKED_AOTC"),
    ] {
        if needed {
            emit(r, "2", line_id, 1, Message::new(explanation_key, alloc::vec![]));
        }
    }
    if let Some((cause, your_days, spouse_days)) = eic {
        part_ii(ctx, r, cause, your_days, spouse_days);
    }
    if child_credits {
        part_iii(ctx, r);
    }
    if aotc {
        part_iv(ctx, r);
    }
}

fn part_ii(
    ctx: &Form8862Context<'_>,
    r: &mut FederalResult,
    cause: EicDisallowanceCause,
    your_days: Option<u32>,
    spouse_days: Option<u32>,
) {
    let p = ctx.people;
    let income_only = cause == EicDisallowanceCause::IncomeReporting;
    let amount = u32::from(income_only);
    emit(
        r,
        "3",
        "form_8862_line_3",
        amount,
        Message::new(
            if income_only {
                "EXPLAIN_8862_INCOME_REPORTING_ONLY"
            } else {
                "EXPLAIN_8862_FINDING_NOT_ONLY_INCOME"
            },
            alloc::vec![],
        ),
    );
    if income_only {
        return;
    }
    let another =
        p.taxpayer.another_eitc_child || p.spouse.as_ref().is_some_and(|v| v.another_eitc_child);
    let amount = u32::from(another);
    emit(
        r,
        "4",
        "form_8862_line_4",
        amount,
        Message::new(
            if another {
                "EXPLAIN_8862_ANOTHER_QUALIFYING_CHILD_YES"
            } else {
                "EXPLAIN_8862_ANOTHER_QUALIFYING_CHILD_NO"
            },
            alloc::vec![],
        ),
    );
    if ctx.eic_children.is_empty() {
        section_b(ctx, r, your_days, spouse_days);
        return;
    }
    // Section A. The form lists three children, as Schedule EIC does.
    if ctx.eic_children.len() > 3 {
        r.review_notes.push(
            crate::federal_model::ReviewNote::localized(#[cfg(test)] "Form 8862 Section A lists the three Schedule EIC children; the other qualifying children are not listed.", crate::Message::new("NOTE_FORM_8862_SECTION_A_LISTS_THE_THREE_SCHEDULE_EIC_CHILDREN_THE_OT_3C959748", alloc::vec![])),
        );
    }
    let letters = ["a", "b", "c"];
    for (index, id) in ctx.eic_children.iter().take(3).enumerate() {
        let Some(child) = dependent(ctx, *id) else {
            continue;
        };
        let number = index + 1;
        emit(
            r,
            &format!("5{}", letters[index]),
            &format!("form_8862_line_5{}", letters[index]),
            1,
            Message::new(
                "EXPLAIN_FORM_N_LINE_N_6",
                alloc::vec![MessageArg::Number(number as i64), MessageArg::Encoded(child.name()),],
            ),
        );
    }
    emit(r, "6", "form_8862_line_6", 1, msg!("EXPLAIN_FORM_N_LINE_N_7"));
    for (index, id) in ctx.eic_children.iter().take(3).enumerate() {
        let Some(child) = dependent(ctx, *id) else {
            continue;
        };
        let number = index + 1;
        let whole_year_exception =
            matches!(child.residency, DependentResidency::WholeYearException);
        let born_return_year = child.birth_date / 10000 == TAX_YEAR;
        emit(
            r,
            "7",
            &format!("form_8862_line_7_child_{number}"),
            child.days_in_home(),
            Message::new(
                if whole_year_exception {
                    "EXPLAIN_8862_CHILD_HOME_DAYS_EXCEPTION"
                } else {
                    "EXPLAIN_8862_CHILD_HOME_DAYS"
                },
                alloc::vec![MessageArg::Encoded(child.name())],
            ),
        );
        if born_return_year {
            emit(
                r,
                "8",
                &format!("form_8862_line_8_child_{number}_birth"),
                child.birth_date % 10000,
                Message::new(
                    "EXPLAIN_FORM_N_LINE_N_CHILD_N_BIRTH",
                    alloc::vec![MessageArg::Encoded(child.name())],
                ),
            );
        }
        if let Some(death) = child.death_date {
            emit(
                r,
                "8",
                &format!("form_8862_line_8_child_{number}_death"),
                death % 10000,
                Message::new(
                    "EXPLAIN_FORM_N_LINE_N_CHILD_N_DEATH",
                    alloc::vec![MessageArg::Encoded(child.name())],
                ),
            );
        }
    }
}

fn section_b(
    ctx: &Form8862Context<'_>,
    r: &mut FederalResult,
    your_days: Option<u32>,
    spouse_days: Option<u32>,
) {
    let p = ctx.people;
    let filers: Vec<(&str, &str, &crate::FederalPerson, Option<u32>)> =
        core::iter::once(("a", crate::fixture_english!("your"), &p.taxpayer, your_days))
            .chain(
                ctx.joint
                    .then(|| {
                        p.spouse.as_ref().map(|s| {
                            ("b", crate::fixture_english!("your spouse’s"), s, spouse_days)
                        })
                    })
                    .flatten(),
            )
            .collect();
    for (letter, _whose, person, days) in filers {
        match days {
            // Days not entered: the estimate does not depend on them; the
            // form still needs them, with a note.
            None => crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(
                    if letter == "a" {
                        "WARN_8862_HOME_DAYS_TAXPAYER"
                    } else {
                        "WARN_8862_HOME_DAYS_SPOUSE"
                    },
                    alloc::vec![],
                ),
                crate::IssueContext::owner(
                    crate::TaxSection::MoreCredits,
                    if letter == "a" { crate::Owner::Taxpayer } else { crate::Owner::Spouse },
                ),
            ),
            Some(days) => {
                crate::federal_rules::require_message(
                    r,
                    (days >= 183) == person.us_home_over_half_year,
                    &crate::IssueContext::owner(
                        crate::TaxSection::MoreCredits,
                        if letter == "a" { crate::Owner::Taxpayer } else { crate::Owner::Spouse },
                    )
                    .alternatives(&[
                        crate::TaxSection::MoreCredits,
                        if letter == "a" {
                            crate::TaxSection::People
                        } else {
                            crate::TaxSection::Spouse
                        },
                    ]),
                    #[cfg(test)]
                    SECTION,
                    #[cfg(test)]
                    format!(
                        "Form 8862 line 9{letter}: {days} days contradicts the answer that {_whose} main home was in the United States for more than half of 2026; correct one of them."
                    ),
                    crate::Message::new(
                        if letter == "a" {
                            "ISSUE_8862_HOME_DAYS_CONTRADICTION_TAXPAYER"
                        } else {
                            "ISSUE_8862_HOME_DAYS_CONTRADICTION_SPOUSE"
                        },
                        alloc::vec![crate::MessageArg::Number(i64::from(days))],
                    ),
                );
                emit(
                    r,
                    &format!("9{letter}"),
                    &format!("form_8862_line_9{letter}"),
                    days,
                    Message::new(
                        if letter == "a" {
                            "EXPLAIN_8862_US_HOME_DAYS_TAXPAYER"
                        } else {
                            "EXPLAIN_8862_US_HOME_DAYS_SPOUSE"
                        },
                        alloc::vec![],
                    ),
                );
            }
        }
        emit(
            r,
            &format!("10{letter}"),
            &format!("form_8862_line_10{letter}"),
            person.eic_age(),
            Message::new(
                if letter == "a" { "EXPLAIN_8862_AGE_TAXPAYER" } else { "EXPLAIN_8862_AGE_SPOUSE" },
                alloc::vec![],
            ),
        );
        let amount = u32::from(person.dependency.claimable());
        emit(
            r,
            &format!("11{letter}"),
            &format!("form_8862_line_11{letter}"),
            amount,
            Message::new(
                match (letter == "a", amount != 0) {
                    (true, true) => "EXPLAIN_8862_CLAIMABLE_TAXPAYER_YES",
                    (true, false) => "EXPLAIN_8862_CLAIMABLE_TAXPAYER_NO",
                    (false, true) => "EXPLAIN_8862_CLAIMABLE_SPOUSE_YES",
                    (false, false) => "EXPLAIN_8862_CLAIMABLE_SPOUSE_NO",
                },
                alloc::vec![],
            ),
        );
    }
}

fn part_iii(ctx: &Form8862Context<'_>, r: &mut FederalResult) {
    if ctx.ctc_children.len() > 4 || ctx.other_dependents.len() > 4 {
        r.review_notes.push(
            crate::federal_model::ReviewNote::localized(#[cfg(test)] "Form 8862 Part III lists four children and four other dependents; the rest need an attached statement answering the same questions.", crate::Message::new("NOTE_FORM_8862_PART_III_LISTS_FOUR_CHILDREN_AND_FOUR_OTHER_DEPENDENTS_7C3CBA57", alloc::vec![])),
        );
    }
    let letters = ["a", "b", "c", "d"];
    // (index within its line, the person, a CTC child rather than an ODC person)
    let people: Vec<(usize, &FederalDependent, bool)> = ctx
        .ctc_children
        .iter()
        .take(4)
        .enumerate()
        .filter_map(|(i, id)| dependent(ctx, *id).map(|c| (i, c, true)))
        .chain(
            ctx.other_dependents
                .iter()
                .take(4)
                .enumerate()
                .filter_map(|(i, id)| dependent(ctx, *id).map(|c| (i, c, false))),
        )
        .collect();
    for (index, person, is_child) in &people {
        let (line, explanation_key) = if *is_child {
            ("12", "EXPLAIN_8862_LISTED_CHILD")
        } else {
            ("13", "EXPLAIN_8862_LISTED_OTHER_DEPENDENT")
        };
        emit(
            r,
            &format!("{line}{}", letters[*index]),
            &format!("form_8862_line_{line}{}", letters[*index]),
            1,
            Message::new(
                explanation_key,
                alloc::vec![
                    MessageArg::Number((index + 1) as i64),
                    MessageArg::Encoded(person.name()),
                ],
            ),
        );
    }
    for (index, person, is_child) in &people {
        let slot = if *is_child {
            format!("child_{}", index + 1)
        } else {
            format!("other_dependent_{}", index + 1)
        };
        if *is_child {
            emit(
                r,
                "14",
                &format!("form_8862_line_14_{slot}"),
                1,
                Message::new(
                    "EXPLAIN_FORM_N_LINE_N_N_2",
                    alloc::vec![
                        MessageArg::Encoded(person.name()),
                        MessageArg::Number(i64::from(person.days_in_home())),
                    ],
                ),
            );
            emit(
                r,
                "15",
                &format!("form_8862_line_15_{slot}"),
                1,
                Message::new(
                    "EXPLAIN_FORM_N_LINE_N_N_3",
                    alloc::vec![MessageArg::Encoded(person.name())],
                ),
            );
        }
        emit(
            r,
            "16",
            &format!("form_8862_line_16_{slot}"),
            1,
            Message::new(
                "EXPLAIN_FORM_N_LINE_N_N_4",
                alloc::vec![MessageArg::Encoded(person.name())],
            ),
        );
        emit(
            r,
            "17",
            &format!("form_8862_line_17_{slot}"),
            1,
            Message::new(
                "EXPLAIN_FORM_N_LINE_N_N_5",
                alloc::vec![MessageArg::Encoded(person.name())],
            ),
        );
    }
}

fn part_iv(ctx: &Form8862Context<'_>, r: &mut FederalResult) {
    let students: Vec<(&crate::federal_model::EducationAnnualProjection, &PersonChoice)> = ctx
        .education
        .iter()
        .filter(|e| e.method() == EducationMethod::AmericanOpportunity)
        .filter_map(|e| ctx.students.iter().find(|s| s.id == e.student).map(|s| (e, s)))
        .collect();
    if students.len() > 3 {
        r.review_notes.push(
            crate::federal_model::ReviewNote::localized(#[cfg(test)] "Form 8862 Part IV lists three students; the rest need an attached statement answering the same questions.", crate::Message::new("NOTE_FORM_8862_PART_IV_LISTS_THREE_STUDENTS_THE_REST_NEED_AN_ATTACHED_B8DCB35A", alloc::vec![])),
        );
    }
    let letters = ["a", "b", "c"];
    for (index, (_, student)) in students.iter().take(3).enumerate() {
        emit(
            r,
            &format!("18{}", letters[index]),
            &format!("form_8862_line_18{}", letters[index]),
            1,
            Message::new(
                "EXPLAIN_FORM_N_LINE_N_12",
                alloc::vec![
                    MessageArg::Number((index + 1) as i64),
                    MessageArg::Encoded(student.label.clone()),
                ],
            ),
        );
    }
    for (index, (entry, student)) in students.iter().take(3).enumerate() {
        let number = index + 1;
        let eligible = entry.method() == crate::EducationMethod::AmericanOpportunity;
        let amount = u32::from(eligible);
        emit(
            r,
            "19a",
            &format!("form_8862_line_19a_student_{number}"),
            amount,
            Message::new(
                if eligible {
                    "EXPLAIN_8862_STUDENT_ELIGIBLE_YES"
                } else {
                    "EXPLAIN_8862_STUDENT_ELIGIBLE_NO"
                },
                alloc::vec![MessageArg::Encoded(student.label.clone())],
            ),
        );
        let amount = u32::from(!eligible);
        emit(
            r,
            "19b",
            &format!("form_8862_line_19b_student_{number}"),
            amount,
            Message::new(
                if !eligible {
                    "EXPLAIN_8862_FOUR_PRIOR_YEARS_YES"
                } else {
                    "EXPLAIN_8862_FOUR_PRIOR_YEARS_NO"
                },
                alloc::vec![MessageArg::Encoded(student.label.clone())],
            ),
        );
    }
}
