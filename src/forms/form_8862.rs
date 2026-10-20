// Coverage: Form 8862 — Supported (unit T2-46, 2026-09-26).
//   Lines: Part I line 1 (2026) and line 2 (one box per credit group that needs the form);
//   Part II line 3 (income-reporting-only cause skips the rest), line 4 (another taxpayer's
//   qualifying child, from FederalPerson.another_eitc_child), Section A lines 5a–5c (the
//   Schedule EIC qualifying children, from family()), 6, 7 (days from DependentResidency) and 8
//   (a 2026 birth date), Section B lines 9a/9b (new days fields, checked against
//   FederalPerson.us_home_over_half_year), 10a/10b (eic_age: age 25 the day before the
//   birthday, age 65 on the birthday, i8862 line 10; unit family credits F8, 2026-10-10) and
//   11a/11b (FilerDependency);
//   Part III lines 12a–12d/13a–13d (the CTC and ODC people from family()) and 14–17 (their
//   classification facts); Part IV lines 18a–18c and 19a/19b (the American Opportunity
//   students, from FederalEducation.eligible_student and aotc_requirements). The credits
//   themselves compute in schedule_eic.rs, schedule_8812.rs and the Form 8863 arm of
//   federal_rules.rs; this module only decides whether the form is needed and fills it.
//   Not needed (a review note, no lines): a math or clerical error only; a reinstated credit
//   with no later disallowance; a CTC/ODC or AOTC disallowance for a year before 2016; the
//   childless-EIC exception when the only earlier finding was that a listed child was not a
//   qualifying child; a group whose credit the return does not claim.
//   A still-running 2-year (reckless or intentional disregard) or 10-year (fraud) ban covering
//   2026, counted from the disallowed tax year as the instructions' example does, is a zero
//   credit for that credit group with NOTE_8862_*_BANNED (unit B11 KIDS, 2026-10-11): `bans`
//   reads it before the credits, the earned income credit and the child credits (CTC, ACTC,
//   ODC; the people stay dependents) come out of Schedule EIC and Schedule 8812 and no Form
//   8862 is filled for the group, and a banned American opportunity student is figured with the
//   lifetime learning credit (section 25A(b)(4) names only the AOTC; unknowns.rs aotc_ban). The
//   appeal (a mailed return with Form 8862 during the ban) is the note's text, not a choice.
//   Line 8 prints the month and day of a child's birth (born in 2026) and of death
//   (FederalDependent.death_date). Refused by name: days on line 9a (or 9b) contradicting the
//   filer's "main home in the United States more than half the year" answer. Not modeled: the
//   tiebreaker note under lines 8 and 17 (a competing claim is decided by family()).
//   Batch 6 (2026-10-05): the finding is a required choice (Not sure removed; the IRS notice
//   states it); days on line 9a/9b not entered give a note (WARN_8862_HOME_DAYS_*), since the
//   estimate does not use them.
//   Evidence: forms/form_8862.rs (compute, requirement, bans); src/tests/family/kids.rs f4_*, f4b_*, f4c_*,
//   f6_*, f6b_*; federal_rules.rs (Family's
//   eic_child_ids/ctc_ids/odc_ids, the call after Schedule 8812); federal_model.rs
//   (FederalCredits.after_disallowance, MoreCredit::AfterDisallowance); interview.rs
//   (more_credit_items, apply_more_credits, more_credits_candidate); src/tests/family/family.rs
//   eic_after_disallowance_return_matches_hand_computation,
//   two_year_ban_runs_through_the_second_year_after_the_disallowed_year,
//   ten_year_ban_runs_through_the_tenth_year_after_the_disallowed_year,
//   childless_eic_after_disallowance_needs_line_9_days,
//   form_8862_is_not_needed_for_a_math_error_reinstatement_or_pre_2016_child_credit,
//   childless_eic_exception_and_income_reporting_finding, child_credit_and_aotc_parts_list_each_person;
//   src/tests/family/interview.rs each_more_credit_variant_reaches_its_field, refusals_name_their_topic.
//   T2-cx (2026-09-27, question C19): each credit group defaults to Never and reinstated to false.
//   IRS sources: Form 8862 (Rev. December 2025, Created 1/8/26) and Instructions for Form 8862
//   (Rev. December 2025, Nov 26, 2025), irs.gov/pub/irs-pdf/f8862.pdf and i8862.pdf, fetched
//   2026-09-26; the irs-dft copies (f8862--dft.pdf, i8862--dft.pdf) are drafts of the same
//   December 2025 revision, and no 2026 revision exists at that retrieval date.

// Form 8862: whether a previously reduced or disallowed EIC, CTC/ACTC/ODC or
// AOTC may be claimed again for 2026, and the form's Parts I–IV filled from
// the return's own people, children and students.
use crate::kernel::lines::form_line;
use crate::{
    DependentResidency, EducationMethod, FederalDependent, FederalEducation, FederalPeople,
    FederalResult, FormLine, Message, MessageArg, PersonChoice, TaxForm,
};
use alloc::{format, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const SOURCE: &str = "https://www.irs.gov/instructions/i8862";
const INPUTS: &[&str] = &["credits"];
#[cfg(test)]
const SECTION: &str = "More credits";
const TAX_YEAR: u32 = 2026;

/// Credits claimed after an earlier disallowance (Form 8862)
/// Answer for each credit the IRS reduced or disallowed for an earlier year. The App fills Form 8862 from the people, children and students already on this return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditsAfterDisallowance {
    /// Earned income credit
    #[libertas_default(Never)]
    pub earned_income: EicDisallowance,
    /// Child tax credit, additional child tax credit or credit for other dependents
    #[libertas_default(Never)]
    pub child_credits: CreditDisallowance,
    /// American opportunity tax credit
    #[libertas_default(Never)]
    pub education_credit: CreditDisallowance,
}

/// Earlier disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CreditDisallowance {
    /// Never reduced or disallowed
    Never,
    /// Reduced or disallowed for an earlier year
    Disallowed(DisallowanceFacts),
}

/// Earlier disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum EicDisallowance {
    /// Never reduced or disallowed
    Never,
    /// Reduced or disallowed for an earlier year
    Disallowed {
        /// The disallowance
        facts: DisallowanceFacts,
        /// What the IRS found
        cause: EicDisallowanceCause,
        /// Days in 2026 your main home was in the United States
        /// Needed only when you claim the credit without a qualifying child. Count temporary absences and extended active military duty as days at home.
        /// [NullText]
        /// I am claiming EIC with a qualifying child; these days do not apply
        /// [SetText]
        /// Without a qualifying child, my U.S. home days were
        // Calendar structure: tax year 2026 has 365 days.
        #[libertas_number(min = 0, max = 365)]
        your_us_home_days: Option<u32>,
        /// Days in 2026 your spouse’s main home was in the United States
        /// Needed only on a joint return claiming the credit without a qualifying child.
        /// [NullText]
        /// Spouse home days do not apply to this EIC claim
        /// [SetText]
        /// For our joint claim without a qualifying child, my spouse's U.S. home days were
        // Calendar structure: tax year 2026 has 365 days.
        #[libertas_number(min = 0, max = 365)]
        spouse_us_home_days: Option<u32>,
    },
}

/// The disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DisallowanceFacts {
    /// Most recent tax year for which the credit was reduced or disallowed
    /// The year of the return the IRS adjusted, not the year the notice arrived.
    // Form 8862 prior-finding window: 1997 onward, before this 2026 return.
    #[libertas_number(min = 1997, max = 2025)]
    pub year: u32,
    /// The IRS finding
    /// The IRS notice or letter that reduced or disallowed the credit states it; an IRS account transcript also shows it.
    #[libertas_inline_choices]
    pub finding: DisallowanceFinding,
    /// Form 8862 was filed after that year, the credit was allowed again, and it has not been reduced or disallowed since except for a math or clerical error
    #[libertas_default(false)]
    pub reinstated: bool,
}

/// The IRS finding
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisallowanceFinding {
    /// A math or clerical error only
    MathOrClericalError,
    /// Not eligible, with no finding of reckless disregard or fraud
    Ineligible,
    /// Reckless or intentional disregard of the rules (a 2-year ban)
    RecklessDisregard,
    /// Fraud (a 10-year ban)
    Fraud,
}

/// What the IRS found
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EicDisallowanceCause {
    /// Only incorrectly reported earned income or investment income
    IncomeReporting,
    /// Only that a child listed on Schedule EIC was not a qualifying child
    ChildNotQualifying,
    /// Another reason, or more than one
    Other,
}

impl CreditsAfterDisallowance {
    // The server twin of the page's range attributes.
    pub(crate) fn valid(&self) -> bool {
        let year_ok = |facts: &DisallowanceFacts| (1997..=2025).contains(&facts.year);
        let days_ok = |days: Option<u32>| days.is_none_or(|d| d <= 365);
        let group_ok = |group: &CreditDisallowance| match group {
            CreditDisallowance::Never => true,
            CreditDisallowance::Disallowed(facts) => year_ok(facts),
        };
        let eic_ok = match &self.earned_income {
            EicDisallowance::Never => true,
            EicDisallowance::Disallowed {
                facts, your_us_home_days, spouse_us_home_days, ..
            } => year_ok(facts) && days_ok(*your_us_home_days) && days_ok(*spouse_us_home_days),
        };
        eic_ok && group_ok(&self.child_credits) && group_ok(&self.education_credit)
    }
}

// What the disallowance facts mean for the 2026 claim of one credit group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Requirement {
    // The form is not needed for this credit group; the reason is noted.
    NotNeeded(NoFormReason),
    // The form is needed and can be filled.
    Required,
    // A ban still covers 2026: the credit cannot be claimed (years, last year).
    Banned(u32, u32),
}

// The instructions' Who Must File table and its exceptions: `first_year` is
// the last tax year the requirement does not reach (1996 for the EIC, 2015
// for the child credits and the AOTC). A ban runs for the 2 or 10 tax years
// after the disallowed year, as in the instructions' example (a disallowed
// 2024 claim bars 2025 and 2026).
pub(crate) fn requirement(facts: &DisallowanceFacts, first_year: u32) -> Requirement {
    if facts.finding == DisallowanceFinding::MathOrClericalError {
        return Requirement::NotNeeded(NoFormReason::MathError);
    }
    if facts.reinstated {
        return Requirement::NotNeeded(NoFormReason::Reinstated);
    }
    if facts.year <= first_year {
        return Requirement::NotNeeded(NoFormReason::BeforeRequirement);
    }
    let years = match facts.finding {
        DisallowanceFinding::RecklessDisregard => 2,
        DisallowanceFinding::Fraud => 10,
        _ => return Requirement::Required,
    };
    if TAX_YEAR <= facts.year + years {
        Requirement::Banned(years, facts.year + years)
    } else {
        Requirement::Required
    }
}

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
    pub(crate) education: &'a [FederalEducation],
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NoFormReason {
    MathError,
    Reinstated,
    BeforeRequirement,
    NotClaimed,
    ChildlessException,
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

// The credit groups a still-running ban covers for 2026, each as (years of
// the ban, last tax year it covers). A banned group's credit is zero and
// Form 8862 is not filled for it; the other groups are untouched.
#[derive(Clone, Copy, Default)]
pub(crate) struct Bans {
    pub(crate) eic: Option<(u32, u32)>,
    pub(crate) child: Option<(u32, u32)>,
    pub(crate) aotc: Option<(u32, u32)>,
}

pub(crate) fn bans(claim: Option<&CreditsAfterDisallowance>) -> Bans {
    let ban = |facts: &DisallowanceFacts, first_year: u32| match requirement(facts, first_year) {
        Requirement::Banned(years, last) => Some((years, last)),
        _ => None,
    };
    let Some(claim) = claim else {
        return Bans::default();
    };
    Bans {
        eic: match &claim.earned_income {
            EicDisallowance::Never => None,
            EicDisallowance::Disallowed { facts, .. } => ban(facts, 1996),
        },
        child: match &claim.child_credits {
            CreditDisallowance::Never => None,
            CreditDisallowance::Disallowed(facts) => ban(facts, 2015),
        },
        aotc: match &claim.education_credit {
            CreditDisallowance::Never => None,
            CreditDisallowance::Disallowed(facts) => ban(facts, 2015),
        },
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
    let students: Vec<(&FederalEducation, &PersonChoice)> = ctx
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
