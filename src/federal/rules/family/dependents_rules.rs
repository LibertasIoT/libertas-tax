// dependents items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(crate) fn validate_dependent(d: &FederalDependent) -> Result<(), (&'static str, &'static str)> {
    if !valid_birth(d.birth_date) {
        return Err((
            "ISSUE_FEDERAL_RULES_ENTER_A_VALID_BIRTH_DATE_ON_E3FCDFD3",
            crate::fixture_english!("Enter a valid birth date on or before December 31, 2026."),
        ));
    }
    if d.days_in_home() > 365 || !amounts(&[d.care_expenses(), d.gross_income()]) {
        return Err((
            "DEPENDENT_NAME_DAYS_OR_AMOUNTS",
            crate::fixture_english!("Review this person's name, residency days and amounts."),
        ));
    }
    if let DependentResidency::Days { days } = d.residency
        && d.birth_date / 10000 == 2026
        && days as i32 > crate::dates::days_between(d.birth_date, 20261231) + 1
    {
        return Err((
            "ISSUE_FEDERAL_RULES_RESIDENCE_DAYS_CANNOT_EXCEED_DAYS_ALIVE_1A7BB934",
            crate::fixture_english!(
                "Residence days cannot exceed days alive in 2026. Use the qualifying residency exception when the newborn rule applies."
            ),
        ));
    }
    if d.has(crate::PersonFacts::ADOPTED_US_CITIZEN_PARENT)
        && d.relationship != crate::Relationship::Child
    {
        return Err((
            "MEDICAL_ADOPTED_CHILD_RELATIONSHIP",
            crate::fixture_english!("Review the adopted-child medical exception and relationship."),
        ));
    }
    if d.has(crate::PersonFacts::MEDICAL_SEPARATED_PARENTS)
        && (d.relationship != crate::Relationship::Child
            || d.own_support_over_half()
            || d.multiple_support())
    {
        return Err((
            "MEDICAL_PARENT_SUPPORT_FACTS_CONFLICT",
            crate::fixture_english!(
                "Review the separated-parent medical relationship and support facts."
            ),
        ));
    }
    if d.own_support_over_half() && d.your_support_over_half() {
        return Err((
            "ISSUE_FEDERAL_RULES_BOTH_YOU_AND_THIS_PERSON_CANNOT_56C06D31",
            crate::fixture_english!(
                "Both you and this person cannot each provide more than half the same total support."
            ),
        ));
    }
    Ok(())
}

pub(crate) struct Family {
    pub(super) dependent_ids: Vec<i64>,
    // Form 8862 Parts II and III: the Schedule EIC qualifying children, the
    // CTC children and the ODC people, by id in dependents order.
    pub(crate) eic_child_ids: Vec<i64>,
    pub(crate) ctc_ids: Vec<i64>,
    pub(crate) odc_ids: Vec<i64>,
    pub(super) ctc: i64,
    pub(super) odc: i64,
    pub(super) eic_children: usize,
    // Children who meet every Schedule EIC qualifying-child test except the
    // valid SSN: they make Step 3 question 1 Yes (Schedule EIC is attached)
    // but raise no credit amount (Pub. 596 Rule 2, self-only EIC).
    pub(super) eic_ssnless_ids: Vec<i64>,
    pub(crate) care_people: i64,
    pub(crate) care_expenses: i64,
    pub(crate) care_person_ids: Vec<i64>,
    pub(crate) care_under_13_child: bool,
    pub(crate) qualifying_head: bool,
    pub(super) qualifying_married_head: bool,
    pub(super) qualifying_survivor: bool,
}
impl Family {
    pub(crate) fn dependent_ids(&self) -> &[i64] {
        &self.dependent_ids
    }
}
// The days the filer was alive in 2026 when the residence tests run over a
// shorter year than 365 days: the final return of a person who died, that is
// not joint (a joint return's years end with the survivor's, section 6013(c)).
pub(crate) fn short_year_days(d: &FederalDraft) -> Option<u32> {
    if d.setup.as_ref().is_none_or(|s| joint(s.filing_status())) {
        return None;
    }
    let date = d.people.as_ref()?.taxpayer.death_date()?;
    Some((crate::dates::days_between(20260101, date) + 1).clamp(1, 365) as u32)
}
pub(crate) fn family(d: &FederalDraft, r: &mut FederalResult, people: &FederalPeople) -> Family {
    family_for(d, r, people, FamilyPurpose::TaxReturn)
}

// IRC213(a), (d)(5): use the same relationship/residence/support owner, with
// only the statutory medical exceptions. No medical projection claims a
// dependent credit, EIC, care or filing-status benefit.
pub(crate) fn medical_dependent_ids(d: &FederalDraft) -> Vec<i64> {
    let Some(people) = &d.people else {
        return Vec::new();
    };
    family_for(d, &mut FederalResult::empty(d.revision), people, FamilyPurpose::Medical)
        .dependent_ids
}

// IRC162(l)(1)(C) uses ordinary section152 dependents; unlike213(a), it
// does not waive the dependent-taxpayer, joint-return or gross-income tests.
// Its separate under27 child rule is applied by the health owner.
pub(crate) fn self_employed_dependent_ids(d: &FederalDraft) -> Vec<i64> {
    let Some(people) = &d.people else {
        return Vec::new();
    };
    family_for(d, &mut FederalResult::empty(d.revision), people, FamilyPurpose::SelfEmployed)
        .dependent_ids
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FamilyPurpose {
    TaxReturn,
    Medical,
    SelfEmployed,
}

pub(super) fn family_for(
    d: &FederalDraft,
    r: &mut FederalResult,
    people: &FederalPeople,
    purpose: FamilyPurpose,
) -> Family {
    let medical = purpose == FamilyPurpose::Medical;
    let health = purpose != FamilyPurpose::TaxReturn;
    let mut f = Family {
        dependent_ids: Vec::new(),
        eic_child_ids: Vec::new(),
        ctc_ids: Vec::new(),
        odc_ids: Vec::new(),
        ctc: 0,
        odc: 0,
        eic_children: 0,
        eic_ssnless_ids: Vec::new(),
        care_people: 0,
        care_expenses: 0,
        care_person_ids: Vec::new(),
        care_under_13_child: false,
        qualifying_head: false,
        qualifying_married_head: false,
        qualifying_survivor: false,
    };
    let younger_than = people
        .spouse
        .as_ref()
        .map_or(people.taxpayer.birth_date, |p| min(p.birth_date, people.taxpayer.birth_date));
    let tin = people.taxpayer.valid_tin && people.spouse.as_ref().is_none_or(|p| p.valid_tin);
    let ssn =
        (people.taxpayer.valid_ssn || people.spouse.as_ref().is_some_and(|p| p.valid_ssn)) && tin;
    // The residence tests run over "such taxable year" (IRC sections 152(c)(1)(B),
    // 2(b)(1)(A), 32(c)(3)(A)(ii) and 21(b)(1)(B)), which ends at death on the
    // final return of a person who died; a joint return's years end with the
    // survivor's (section 6013(c)), so its tests run over the whole year.
    let year_days = short_year_days(d).unwrap_or(365);
    // A filer someone can claim is treated as having no dependents (IRC section
    // 152(b)(1); Pub. 501 "Dependent Taxpayer Test"): no dependency, child tax
    // credit, credit for other dependents, dependent care or head of household
    // through a dependent. Head of household through an unmarried qualifying
    // child and the earned income credit through a qualifying child follow
    // their own tests, which do not need a dependent. Not sure counts as
    // claimable.
    let dependent_filer = !medical
        && core::iter::once(&people.taxpayer)
            .chain(people.spouse.iter())
            .any(|person| person.dependency.claimable());
    for (position, c) in d.dependents().iter().enumerate() {
        if !health && c.use_for == crate::PersonUse::HealthCoverageOnly {
            continue;
        }
        // Someone else who could also claim this qualifying child (section
        // 152(c)(4)): the other person's claim gives this person none of the
        // five benefits; the tiebreaker not met by the filer is the same.
        let record = crate::IssueContext::Section(crate::TaxSection::Children)
            .item("dependents", position)
            .named_encoded(&c.name());
        let other_claims = match c.competing {
            CompetingClaim::NoOne => false,
            CompetingClaim::You { basis: ClaimBasis::NoneOfThese } => {
                note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_TIEBREAKER_NOT_MET",
                        alloc::vec![crate::MessageArg::Encoded(c.name())],
                    ),
                    record.clone(),
                );
                true
            }
            CompetingClaim::You { .. } => false,
            CompetingClaim::Other => true,
        };
        let medical_parent = medical
            && c.relationship == Relationship::Child
            && !c.multiple_support()
            && !c.own_support_over_half()
            && (c.has(crate::PersonFacts::MEDICAL_SEPARATED_PARENTS)
                || c.custody_release() != CustodyRelease::None);
        if other_claims && !medical_parent {
            note_once_scoped(
                r,
                crate::Message::new(
                    "NOTE_COMPETING_CLAIM_OTHER_PERSON_CLAIMS",
                    alloc::vec![crate::MessageArg::Encoded(c.name())],
                ),
                record,
            );
            continue;
        }
        let related = matches!(
            c.relationship,
            Relationship::Child
                | Relationship::Foster
                | Relationship::Grandchild
                | Relationship::Sibling
                | Relationship::NieceNephew
        );
        // A child born in 2026 who lived with the filer for more than half the days alive meets
        // the residence test (Pub. 501, "a child who was born or died during the year").
        // The days alive in 2026 run from the birth (or January 1) to the
        // death (or December 31); the residence tests run over them.
        let alive_days = {
            let first = if c.birth_date / 10000 == 2026 { c.birth_date } else { 20260101 };
            (crate::dates::days_between(first, c.death_date.unwrap_or(20261231)) + 1) as u32
        };
        let residence_days = year_days.min(alive_days);
        let lived = c.days_in_home() * 2 > residence_days;
        let whole = c.days_in_home() >= residence_days;
        // IRC 152(c)(3)(A)-(B): under 19, or under 24 and a student, "at the
        // close of the calendar year". A child attains an age on the birthday
        // (Rev. Rul. 2003-72, sections 24, 32 and 151): a child born January 1,
        // 2008 is 18 on December 31, 2026 and attains 19 on January 1, 2027.
        let year_end_age =
            crate::federal_model::birthday_age_at_close_or_death(c.birth_date, c.death_date);
        let age_ok = c.disabled()
            || ((year_end_age < 19 || (year_end_age < 24 && c.student()))
                && c.birth_date > younger_than);
        let child = related && lived && age_ok && (medical || !c.joint_return_nonrefund());
        // Forms 8332 and 2120. IRC 152(e)(1)-(2): a custodial parent's written
        // release (or a qualified pre-1985 instrument) lets the other parent
        // claim the child although the child lived with the custodial parent
        // for more nights; it overrides only the residency, tie-breaker and
        // support tests of 152(c)(1)(B), (c)(4) and (d)(1)(C), not the other
        // dependency tests. IRC 152(d)(3): a multiple support agreement
        // replaces the over-half support test of a qualifying relative.
        let received = c.custody_release() == CustodyRelease::ReleasedToYou;
        let given = c.custody_release() == CustodyRelease::YouReleased;
        // The qualifying child of 152(c) without regard to 152(e).
        let qualifying_child = child && !c.own_support_over_half();
        let released_child = received
            && c.relationship == Relationship::Child
            && age_ok
            && (medical || !c.joint_return_nonrefund())
            && !c.own_support_over_half();
        let dependency_child =
            (qualifying_child && (!given || medical)) || released_child || medical_parent;
        // IRC 152(d)(1)(D): a person who is another taxpayer's qualifying
        // child is no qualifying relative of this filer (Pub. 501 "Child of
        // person not required to file a return" is the unticked case).
        let anothers_child = c.has(PersonFacts::ANOTHERS_QUALIFYING_CHILD) && !child;
        if anothers_child {
            note_once_scoped(
                r,
                crate::Message::new(
                    "NOTE_ANOTHERS_QUALIFYING_CHILD",
                    alloc::vec![crate::MessageArg::Encoded(c.name())],
                ),
                crate::IssueContext::Section(crate::TaxSection::Children)
                    .item("dependents", position)
                    .named_encoded(&c.name()),
            );
        }
        let relative = !child
            && !released_child
            && (!given || medical)
            && !anothers_child
            && (c.your_support_over_half()
                || c.multiple_support()
                || (received && !c.own_support_over_half()))
            && (medical || c.gross_income() < 530_000)
            && (c.relationship != Relationship::Unrelated || whole)
            && (medical || !c.joint_return_nonrefund());
        let citizenship = c.us_citizen_resident()
            || (health
                && (c.has(crate::PersonFacts::RESIDENT_CANADA_MEXICO)
                    || (c.has(crate::PersonFacts::ADOPTED_US_CITIZEN_PARENT)
                        && c.relationship == Relationship::Child
                        && whole)));
        let dependent = (dependency_child || relative) && citizenship && !dependent_filer;
        if dependent {
            f.dependent_ids.push(c.id);
            // The declarations themselves are the person's to sign, keep and
            // attach; this App does not prepare them.
            let record = crate::IssueContext::Section(crate::TaxSection::Children)
                .item("dependents", position)
                .named_encoded(&c.name());
            if received {
                note_once_scoped(
                    r,
                    crate::Message::new(
                        "NOTE_FORM_8332_ATTACH",
                        alloc::vec![crate::MessageArg::Encoded(c.name())],
                    ),
                    record.clone(),
                );
            }
            if c.multiple_support() {
                note_once_scoped(
                    r,
                    crate::Message::new(
                        "NOTE_FORM_2120_ATTACH",
                        alloc::vec![crate::MessageArg::Encoded(c.name())],
                    ),
                    record,
                );
            }
        }
        if health {
            continue;
        }
        // A child born and died in 2026 with no SSN still counts for the child
        // tax credit and the earned income credit (2026 i1040s8 "Death or
        // birth of child"; Schedule EIC line 2 "Died").
        let child_ssn = c.valid_ssn || c.born_and_died();
        // IRC 24(c)(1): a qualifying child for the credit is under 17 at the
        // close of the year; the child attains 17 on the birthday (Rev. Rul.
        // 2003-72), so a child born January 1, 2010 is 16 on December 31, 2026.
        if dependent && year_end_age < 17 && dependency_child && child_ssn && ssn {
            f.ctc += 1;
            f.ctc_ids.push(c.id);
        } else if dependent && c.valid_tin && tin {
            f.odc += 1;
            f.odc_ids.push(c.id);
        }
        // EITC has no support test: a qualifying child need not be claimed as a dependent.
        // 2025 i1040 "Married child" (IRC 32(c)(3)(B)): a child married at the
        // end of the year is a qualifying child only if the filer can claim the
        // child as a dependent, or could have but for the section 152(e)
        // release the filer gave to the child's other parent.
        let married_child_counts = !c.married_at_year_end()
            || dependent
            || (given && qualifying_child && c.us_citizen_resident());
        if child && married_child_counts && c.us_home_over_half_year {
            if child_ssn {
                f.eic_children += 1;
                f.eic_child_ids.push(c.id);
            } else {
                f.eic_ssnless_ids.push(c.id);
            }
        }
        // Head of household (IRC section 2(b)(1)(A)(i)-(ii), Pub. 501 (2025)
        // Table 4): a qualifying child who lived with the filer qualifies the
        // filer, determined without regard to section 152(e) (so the
        // custodial parent who released the child keeps it, and the parent
        // holding the release never has it), whether or not the child is a
        // citizen or resident and whether or not the filer may claim the
        // child; but not a married child who is not a dependent by reason of
        // section 152(b)(2) or (b)(3), i.e. a married child who is not a U.S.
        // citizen, national or resident (a child's joint return already fails
        // the child test). Any other qualifying person must be the filer's
        // dependent, other than under a release or a section 152(d)(3)
        // multiple support agreement (section 2(b)(3)(B)(ii)).
        let head_child =
            qualifying_child && !received && (!c.married_at_year_end() || c.us_citizen_resident());
        let head_relative = relative
            && !dependent_filer
            && c.us_citizen_resident()
            && !received
            && !c.multiple_support()
            && c.relationship != Relationship::Unrelated
            && (lived || c.relationship == Relationship::Parent);
        f.qualifying_head |= head_child || head_relative;
        // Married filer considered unmarried (IRC section 7703(b)(1)): a
        // son, stepson, daughter, stepdaughter or foster child for whom the
        // filer is entitled to the section 151 deduction, or would be but
        // for section 152(e). That deduction needs the section 152(b)(3)
        // citizen, national or resident test.
        f.qualifying_married_head |= qualifying_child
            && !received
            && c.us_citizen_resident()
            && matches!(c.relationship, Relationship::Child | Relationship::Foster);
        // Qualifying surviving spouse (IRC section 2(a)(1)(B), Pub. 501 (2025)
        // "Qualifying Surviving Spouse" test 3): a son, daughter, stepson or
        // stepdaughter (never a foster child) who lived in the home all year
        // and is the filer's dependent as a qualifying child or as a
        // qualifying relative, determined without regard to section
        // 152(b)(1), (b)(2) and (d)(1)(B): the joint-return, gross-income and
        // being-claimed tests are waived, the citizen or resident test and
        // the support test are not. The relative path is open once the
        // person is not a qualifying child. A child the filer released is not
        // the filer's dependent.
        f.qualifying_survivor |= whole
            && !dependent_filer
            && c.relationship == Relationship::Child
            && c.us_citizen_resident()
            && (dependency_child
                || (!child && !given && !anothers_child && c.your_support_over_half()));
        // Form 2441 waives the dependency gross-income and joint-return tests
        // for a person incapable of self-care; support and household tests remain.
        let care_dependent = dependent
            || (!dependent_filer
                && c.incapable_self_care
                && c.us_citizen_resident()
                && ((related && lived && age_ok && !c.own_support_over_half())
                    || (!anothers_child
                        && (c.your_support_over_half() || c.multiple_support())
                        && (c.relationship != Relationship::Unrelated || whole))));
        // The two-person expense ceiling applies even if expenses were paid for only one person.
        // IRC 21(b)(1)(A): a dependent who has not attained 13, tested on
        // January 1 so care before the 13th birthday counts for the part of the
        // year. A child attains 13 on the birthday (Rev. Rul. 2003-72; the 2026
        // i2441 "turned 13 during the year"): a child born January 1, 2013 is
        // 13 on January 1, 2026, one born January 2, 2013 is still 12.
        let under_13_in_year =
            !crate::federal_model::attained_on(c.birth_date, 13, false, 20260101);
        // IRC 21(e)(5): the custodial parent's child is the parent's
        // qualifying individual although the other parent claims the child,
        // and the parent holding the release has none.
        let care_qualifying = lived
            && !received
            && ((qualifying_child
                && !dependent_filer
                && c.us_citizen_resident()
                && under_13_in_year)
                || (care_dependent && c.incapable_self_care));
        // IRC 21(e)(10): no credit for a qualifying individual whose taxpayer
        // identification number (SSN, ITIN or ATIN) is not on the return; the
        // 2026 i2441 line 2 column (b) excepts only a child born and died in 2026.
        let care_person = care_qualifying && (c.valid_tin || c.born_and_died());
        if care_person {
            f.care_people += 1;
            f.care_person_ids.push(c.id);
            f.care_under_13_child |= under_13_in_year
                && matches!(c.relationship, Relationship::Child | Relationship::Foster);
        }
        // Form 2441 qualifying person (IRC section 21(b)(1), 2026 Form 2441
        // instructions draft "Qualifying Person(s)"): costs entered for a
        // person who is not one are left out with one notice and the
        // estimate proceeds (contract decision 2).
        if c.care_expenses() > 0 {
            if care_person {
                f.care_expenses += c.care_expenses();
            } else if care_qualifying {
                note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_CARE_COSTS_NEED_TIN",
                        alloc::vec![crate::MessageArg::Encoded(c.name())],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Care)
                        .item("qualifying_people", position),
                );
            } else {
                note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_CARE_COSTS_LEFT_OUT",
                        alloc::vec![crate::MessageArg::Encoded(c.name())],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Care)
                        .item("qualifying_people", position),
                );
            }
        }
    }
    f
}
pub(super) fn household(s: &FederalSetup, p: &FederalPeople, f: &Family, r: &mut FederalResult) {
    for (index, person) in core::iter::once(&p.taxpayer).chain(p.spouse.iter()).enumerate() {
        // The final-return facts are the person's own answers; the death-related
        // adjustments the instructions do not let the App compute are left out
        // with one notice on that person's page.
        if let Some(final_return) = &person.died
            && !final_return.ordinary
        {
            let (section, owner) = if index == 0 {
                (crate::TaxSection::People, Owner::Taxpayer)
            } else {
                (crate::TaxSection::Spouse, Owner::Spouse)
            };
            note_once_scoped(
                r,
                msg!("WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"),
                crate::IssueContext::owner(section, owner),
            );
        }
    }
    // The person who qualifies a household status is answered on the
    // people page, after page 1: this end-of-walk check names that page.
    let qualifying = match s.filing_status() {
        FederalStatus::Head => f.qualifying_head || f.qualifying_married_head,
        FederalStatus::Surviving => f.qualifying_survivor,
        _ => true,
    };
    crate::require_localized!(
        r,
        qualifying,
        &crate::IssueContext::Section(crate::TaxSection::Children),
        "Children and people you support",
        "ISSUE_YOUR_FILING_STATUS_NEEDS_A_PERSON_WHO_QUALIFIES_YOU_FOR_IT_ADD_THEM_5994559A",
        "Your filing status needs a person who qualifies you for it: add them on this page, or choose another filing status on Your 2026 return.",
    );
    let married = s.marital_state.married();
    let valid = match s.filing_status() {
        FederalStatus::Single => !married,
        FederalStatus::Joint | FederalStatus::Separate => married,
        FederalStatus::Head => {
            // Section 2(b)(2)(B): a spouse who was a nonresident alien at any
            // time in 2026 leaves the filer unmarried for head of household.
            (!married
                || matches!(s.marital_state, MaritalState::MarriedNonresidentSpouse)
                || (matches!(s.marital_state, MaritalState::Married { .. })
                    && s.lived_apart_last_six_months()
                    && f.qualifying_married_head))
                && s.paid_over_half_home
                && f.qualifying_head
        }
        FederalStatus::Surviving => {
            matches!(
                s.marital_state,
                MaritalState::WidowedPriorYear { .. } | MaritalState::WidowedTwoYearsPrior { .. }
            ) && s.could_file_joint_death_year()
                && s.paid_over_half_home
                && f.qualifying_survivor
        }
    };
    crate::require_localized!(
        r,
        valid,
        &crate::IssueContext::Section(crate::TaxSection::Setup),
        "Filing status",
        "ISSUE_THE_HOUSEHOLD_FACTS_DO_NOT_ESTABLISH_ELIGIBILITY_FOR_THE_SELECTED_F_D1351812",
        "The household facts do not establish eligibility for the selected filing status; special dependency exceptions are not yet supported.",
    );
    crate::require_localized!(
        r,
        joint(s.filing_status()) == p.spouse.is_some(),
        &crate::IssueContext::Section(if joint(s.filing_status()) {
            crate::TaxSection::Spouse
        } else {
            crate::TaxSection::Setup
        }),
        "Filers",
        "ISSUE_PROVIDE_SPOUSE_FACTS_FOR_A_JOINT_RETURN_ONLY_2AB656E0",
        "Provide spouse facts for a joint return only.",
    );
}
// Whether nobody actually claims either filer, for the education benefits.
pub(crate) fn unclaimed_filers(p: &FederalPeople) -> bool {
    core::iter::once(&p.taxpayer).chain(p.spouse.iter()).all(|person| !person.dependency.claimed())
}
