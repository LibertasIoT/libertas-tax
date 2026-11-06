// dependents items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The people page is a projection of the stored dependent records: the stored
// record keeps the page's situation and the "Does any of these apply" fold
// (PersonFacts: citizenship, a joint return, a competing claim, the Form 8332
// release, the Form 2120 agreement, the marital status and the person's own
// support) as chosen, and family() reads both through FederalDependent's
// accessors (M3-P15).
// The page's client rules repeated on save and in the running estimate (the
// server twins); DEPENDENT_OLDER_THAN_YOU alone is server-only because it
// compares with the filers' birth dates, which this page does not hold:
// `filers` are the birth dates of you and, on a joint return, your spouse.
pub(super) fn dependent_issue(
    person: &Dependent,
    filers: &[u32],
    alive: Option<u32>,
) -> Option<ValidationIssue> {
    let born = person.birth_date;
    // The birthday rule (Rev. Rul. 2003-72, sections 24, 32 and 151).
    let year_end_age =
        crate::federal_model::birthday_age_at_close_or_death(born, person.death_date);
    let facts = person.facts;
    let fact = |flag: PersonFacts| facts & flag.0 != 0;
    let (you_released, released_to_you, multiple_support) = (
        fact(PersonFacts::YOU_RELEASED),
        fact(PersonFacts::RELEASED_TO_YOU),
        fact(PersonFacts::MULTIPLE_SUPPORT),
    );
    if fact(PersonFacts::ADOPTED_US_CITIZEN_PARENT) && person.relationship != Relationship::Child {
        return Some(
            ValidationIssue::new("MEDICAL_ADOPTED_CHILD_RELATIONSHIP").at_field("relationship"),
        );
    }
    if fact(PersonFacts::MEDICAL_SEPARATED_PARENTS)
        && (person.relationship != Relationship::Child
            || fact(PersonFacts::OWN_SUPPORT)
            || multiple_support)
    {
        return Some(
            ValidationIssue::new("MEDICAL_PARENT_SUPPORT_FACTS_CONFLICT").at_field("facts"),
        );
    }
    let supported_without_support = matches!(
        person.situation,
        DependentSituation::Supported { your_support_over_half: false, .. }
    );
    // IRC 152(d)(3), 152(e): another person you support needs your support,
    // the agreement, or the other parent's release (the support test is
    // waived by either).
    if person.use_for == crate::PersonUse::TaxDependent
        && supported_without_support
        && !multiple_support
        && !released_to_you
    {
        return Some(
            ValidationIssue::new("SUPPORTED_NEEDS_YOUR_SUPPORT")
                .at_field("your_support_over_half")
                .in_field("situation"),
        );
    }
    if you_released && released_to_you {
        return Some(ValidationIssue::new("CUSTODY_RELEASE_ONE_WAY").at_field("facts"));
    }
    if (you_released || released_to_you) && person.relationship != Relationship::Child {
        return Some(
            ValidationIssue::new("CUSTODY_RELEASE_NEEDS_YOUR_CHILD").at_field("relationship"),
        );
    }
    // IRC 152(e)(4): the release goes to the parent the child lived with for
    // fewer nights, so more than six months in your home cannot be released to you.
    if released_to_you
        && !matches!(person.lived, LivedWithYou::PartOfYear { months } if months <= 6)
        && !matches!(person.lived, LivedWithYou::Days { days } if days * 2 <= alive.unwrap_or(365))
    {
        return Some(
            ValidationIssue::new("CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS").at_field("lived"),
        );
    }
    if multiple_support && !supported_without_support {
        return Some(
            ValidationIssue::new("MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT").at_field("situation"),
        );
    }
    // IRC 152(c)(1)(D), (d)(1)(C), (e)(1)(A): a person who paid more than half
    // of their own support cannot also be supported mostly by you, by the
    // parents who release the claim, or by several supporters together.
    let own_support = fact(PersonFacts::OWN_SUPPORT);
    if own_support
        && matches!(
            person.situation,
            DependentSituation::Supported { your_support_over_half: true, .. }
        )
    {
        return Some(ValidationIssue::new("OWN_SUPPORT_NOT_WITH_YOUR_SUPPORT").at_field("facts"));
    }
    if own_support && (multiple_support || you_released || released_to_you) {
        return Some(ValidationIssue::new("OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT").at_field("facts"));
    }
    // IRC 152(e)(5): a multiple support agreement and a release exclude each other.
    if multiple_support && (you_released || released_to_you) {
        return Some(
            ValidationIssue::new("MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE").at_field("facts"),
        );
    }
    match person.situation {
        // IRC 152(c)(3): under 19, or under 24 and a student, at the close of
        // 2026; a child attains an age on the birthday (a child born January 1,
        // 2008 is 18 on December 31, 2026 and 19 on January 1, 2027).
        DependentSituation::ChildUnder19 if year_end_age >= 19 => {
            return Some(ValidationIssue::new("DEPENDENT_AGE_SITUATION").at_field("situation"));
        }
        DependentSituation::Student if year_end_age >= 24 => {
            return Some(ValidationIssue::new("DEPENDENT_AGE_SITUATION").at_field("situation"));
        }
        DependentSituation::ChildUnder19 | DependentSituation::Student
            if !filers.is_empty() && filers.iter().all(|filer| born <= *filer) =>
        {
            return Some(ValidationIssue::new("DEPENDENT_OLDER_THAN_YOU").at_field("birth_date"));
        }
        _ => {}
    }
    // The adopted-child exception is your own child living with you all year.
    if fact(PersonFacts::ADOPTED_ALL_YEAR)
        && !(person.relationship == Relationship::Child && person.lived == LivedWithYou::AllYear)
    {
        return Some(ValidationIssue::new("ADOPTED_NEEDS_CHILD_ALL_YEAR").at_field("facts"));
    }
    if person.death_date.is_some_and(|died| died < born) {
        return Some(ValidationIssue::new("DEATH_BEFORE_BIRTH").at_field("death_date"));
    }
    // IRC 152(d)(1)(D): only another person you support can be another
    // taxpayer's qualifying child instead of your qualifying relative.
    (fact(PersonFacts::ANOTHERS_QUALIFYING_CHILD)
        && !matches!(person.situation, DependentSituation::Supported { .. }))
    .then_some(ValidationIssue::new("ANOTHERS_CHILD_NEEDS_SUPPORTED").at_field("situation"))
}
pub(super) fn dependent_rule(
    person: &Dependent,
    filers: &[u32],
    alive: Option<u32>,
) -> Option<&'static str> {
    dependent_issue(person, filers, alive).map(|issue| issue.key)
}

// The filers' birth dates the people page checks its people against.
// The residence answers the Children page offers: the days through the death
// where the filer's year is shorter (the final return of a person who died,
// not joint), months otherwise. Whole-year stays an answer either way.
pub(super) fn lived_index(lived: &LivedWithYou) -> i32 {
    (match lived {
        LivedWithYou::AllYear => libertas_macros::variant_index!(LivedWithYou::AllYear),
        LivedWithYou::PartOfYear { .. } => {
            libertas_macros::variant_index!(LivedWithYou::PartOfYear)
        }
        LivedWithYou::Days { .. } => libertas_macros::variant_index!(LivedWithYou::Days),
    }) as i32
}
pub(crate) fn allowed_lived(alive: Option<u32>) -> Vec<i32> {
    alloc::vec![
        libertas_macros::variant_index!(LivedWithYou::AllYear) as i32,
        if alive.is_some() {
            libertas_macros::variant_index!(LivedWithYou::Days) as i32
        } else {
            libertas_macros::variant_index!(LivedWithYou::PartOfYear) as i32
        },
    ]
}
pub(super) fn filer_births(d: &FederalDraft) -> Vec<u32> {
    let people = d.people.as_ref();
    people
        .map(|p| p.taxpayer.birth_date)
        .into_iter()
        .chain(people.and_then(|p| p.spouse.as_ref()).filter(|_| joint(d)).map(|s| s.birth_date))
        .collect()
}
// The pages whose answers are their only mark (the people page and Form
// 8379's): a stored answer their own rules now refuse after an earlier
// change (a child no longer younger than you, an agreed part above the
// payments) cannot be reopened by a mark, so the review names the page with
// the rule's message (walk-owned data, level 2).
pub(super) fn unmarked_refusals(d: &FederalDraft) -> Vec<(TaxSection, &'static str)> {
    let filers = filer_births(d);
    let alive = crate::federal_rules::short_year_days(d);
    let people = d
        .dependents()
        .iter()
        .filter_map(|record| dependent_item(record, alive))
        .find_map(|person| dependent_rule(&person, &filers, alive))
        .map(|key| (TaxSection::Children, key));
    let joint_estimated = crate::injured_spouse::joint_estimated(d);
    let injured = d
        .injured_spouse
        .as_ref()
        .filter(|_| injured_chosen(d))
        .and_then(|a| crate::injured_spouse::rule(&a.page(d, &estimate(d)), &joint_estimated).err())
        .map(|key| (TaxSection::InjuredSpouse, key));
    people.into_iter().chain(injured).collect()
}
// The days a person who died was alive in 2026: from the birth (or January
// 1) through the date of death.
pub(super) fn days_alive_of(person: &Dependent) -> u32 {
    let first = if person.birth_date / 10000 == 2026 { person.birth_date } else { 20260101 };
    (crate::dates::days_between(first, person.death_date.unwrap_or(20261231)) + 1).max(1) as u32
}
// Each listed person's child tax credit, credit for other dependents and
// earned income credit, as the people page shows them: the estimate's own
// classification (federal_rules family(): IRC sections 24(c), 24(h)(4) and
// 32(c)(3)), with Schedule 8812's amount for each person before the income
// limit (2026 Schedule 8812 instructions, lines 5 and 7). The answers that
// override a result stay on the person (Someone else who could also claim
// this person, Use this person's facts for) or on More credits (choosing not
// to claim the earned income credit, Form 1040 line 27c).
pub(super) fn person_credits(d: &FederalDraft) -> Vec<PersonCredits> {
    let Some(people) = d.people.as_ref() else {
        return Vec::new();
    };
    let family =
        crate::federal_rules::family(d, &mut crate::FederalResult::empty(d.revision), people);
    let cents = |dollars: i64| MessageArg::usd_cents(dollars * 100);
    d.dependents()
        .iter()
        .map(|person| {
            let name = MessageArg::Encoded(person.name());
            let eic = family.eic_child_ids.contains(&person.id);
            let child = (
                family.ctc_ids.contains(&person.id),
                crate::year_constants::CHILD_TAX_CREDIT_PER_CHILD_DOLLARS,
            );
            let other = (
                family.odc_ids.contains(&person.id),
                crate::year_constants::OTHER_DEPENDENT_CREDIT_DOLLARS,
            );
            let (key, amount) = match (child, other, eic) {
                ((true, dollars), _, true) => ("PERSON_CREDITS_CHILD_EIC", Some(dollars)),
                ((true, dollars), _, false) => ("PERSON_CREDITS_CHILD", Some(dollars)),
                (_, (true, dollars), true) => ("PERSON_CREDITS_OTHER_EIC", Some(dollars)),
                (_, (true, dollars), false) => ("PERSON_CREDITS_OTHER", Some(dollars)),
                (_, _, true) => ("PERSON_CREDITS_EIC", None),
                (_, _, false) => ("PERSON_CREDITS_NONE", None),
            };
            let args = core::iter::once(name).chain(amount.map(cents)).collect();
            PersonCredits { credits: Message::new(key, args).encode() }
        })
        .collect()
}
// One page row onto its stored record.
pub(super) fn federal_dependent(person: &Dependent, alive: Option<u32>) -> FederalDependent {
    let (residency, us_home_over_half_year) = match person.lived {
        // A child born, or a person who died, in 2026 is in the home for the
        // days alive (Pub. 501 "Death or birth of child").
        LivedWithYou::AllYear
            if person.birth_date / 10000 == 2026 || person.death_date.is_some() =>
        {
            (DependentResidency::WholeYearException, true)
        }
        LivedWithYou::AllYear => (DependentResidency::Days { days: 365 }, true),
        // A person who died lived in the home for more than half of the days
        // alive, not of the year (Pub. 501 "Death or birth of child").
        LivedWithYou::PartOfYear { months } if person.death_date.is_some() => (
            DependentResidency::Days { days: months * 30 },
            months * 30 * 2 > days_alive_of(person),
        ),
        LivedWithYou::PartOfYear { months } => {
            (DependentResidency::Days { days: months * 30 }, months >= 7)
        }
        // More than half of the days the filer was alive.
        LivedWithYou::Days { days } => {
            (DependentResidency::Days { days }, days * 2 > alive.unwrap_or(365))
        }
    };
    // The care answers are the care page's; a save keeps them by identity.
    FederalDependent {
        use_for: person.use_for,
        id: person.id,
        first_name: person.first_name.clone(),
        last_name: person.last_name.clone(),
        birth_date: person.birth_date,
        death_date: person.death_date,
        relationship: person.relationship,
        residency,
        us_home_over_half_year,
        incapable_self_care: false,
        care: None,
        valid_ssn: person.identification == Identification::SocialSecurityNumber,
        valid_tin: person.identification != Identification::NoneByDueDate,
        situation: person.situation,
        facts: person.facts,
        competing: person.competing,
    }
}

// One stored record onto its page row; None for a record the page cannot
// show, i.e. one the page's answers would not reproduce. The names reopen as
// entered.
pub(super) fn dependent_item(record: &FederalDependent, alive: Option<u32>) -> Option<Dependent> {
    let lived = match record.residency {
        DependentResidency::WholeYearException => LivedWithYou::AllYear,
        DependentResidency::Days { days: 365 } => LivedWithYou::AllYear,
        // A shortened year asks days; any other year asks months.
        DependentResidency::Days { days } if alive.is_some() => LivedWithYou::Days { days },
        DependentResidency::Days { days } => LivedWithYou::PartOfYear { months: days / 30 },
    };
    let item = Dependent {
        use_for: record.use_for,
        id: record.id,
        first_name: record.first_name.clone(),
        last_name: record.last_name.clone(),
        birth_date: record.birth_date,
        death_date: record.death_date,
        relationship: record.relationship,
        lived,
        situation: record.situation,
        identification: if record.valid_ssn {
            Identification::SocialSecurityNumber
        } else if !record.valid_tin {
            Identification::NoneByDueDate
        } else {
            Identification::Itin
        },
        facts: record.facts,
        competing: record.competing,
    };
    let without_care =
        FederalDependent { incapable_self_care: false, care: None, ..record.clone() };
    (!item.first_name.is_empty()
        && !item.last_name.is_empty()
        && federal_dependent(&item, alive) == without_care)
        .then_some(item)
}

impl Interview {
    pub(super) fn dependents_candidate(
        &self,
        value: Dependents,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let d = &self.draft;
        let filers = filer_births(d);
        let alive = crate::federal_rules::short_year_days(d);
        // Actual return facts own both the residence choices and the day bound;
        // echoed read-only catalogs never authorize a source answer.
        let allowed = allowed_lived(alive);
        for (index, person) in value.people.iter().enumerate() {
            if !allowed.contains(&(lived_index(&person.lived))) {
                return Err(crate::validation::ValidationIssue::new(
                    "RESIDENCE_ANSWER_NOT_ALLOWED",
                )
                .at_field("lived")
                .in_list("people", index)
                .in_field("dependents")
                .into());
            }
            if matches!(person.lived, LivedWithYou::Days { days } if days > alive.unwrap_or(365)) {
                return Err(crate::validation::ValidationIssue::new(
                    "RESIDENCE_DAYS_ABOVE_DAYS_ALIVE",
                )
                .at_field("days")
                .in_field("lived")
                .in_list("people", index)
                .in_field("dependents")
                .into());
            }
        }
        if let Some((index, issue)) = value.people.iter().enumerate().find_map(|(index, person)| {
            dependent_issue(person, &filers, alive).map(|issue| (index, issue))
        }) {
            return Err(issue.in_list("people", index).in_field("dependents").into());
        }
        let mut records: Vec<FederalDependent> =
            value.people.iter().map(|p| federal_dependent(p, alive)).collect();
        for record in &records {
            validate_dependent(record).map_err(|(key, _)| status(key))?;
        }
        let mut draft = d.clone();
        assign_record_ids(
            records.iter_mut().map(|r| &mut r.id),
            d.dependents().iter().map(|r| r.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, _)| status(key))?;
        for record in &mut records {
            if let Some(before) = d.dependents().iter().find(|p| p.id == record.id) {
                record.care = before.care.clone();
                record.incapable_self_care = before.incapable_self_care;
            }
        }
        draft.dependents = records;
        finish_section(&mut draft, TaxSection::Children);
        Ok(draft)
    }
}
