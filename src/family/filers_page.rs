// filers items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl Interview {
    // Sent only on a joint return whose filer page is saved; Back returns to it.
    // The editor belongs to the filers section.
    pub(super) fn spouse_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let (cookie, revision, page, previous) =
            (d.cookie.clone(), d.revision, PEOPLE_PAGE, PEOPLE_PAGE);
        let progress = self.progress(SPOUSE_PAGE);
        let estimate = self.estimate_now(TaxSection::Spouse);
        match d.people.as_ref().and_then(|p| p.spouse.as_ref()) {
            Some(spouse) => Reply::Edit(P::SaveSpouse {
                professional_election: professional_election(d, Owner::Spouse),
                allowed_professional_elections: d.professional_elections(Owner::Spouse),
                user: self.user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: Vec::new(),
                progress,
                spouse: professional_filer(d, Owner::Spouse, spouse).into(),
            }),
            None => Reply::Response(P::BeginSpouse {
                professional_election: professional_election(d, Owner::Spouse),
                allowed_professional_elections: d.professional_elections(Owner::Spouse),
                cookie,
                revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                progress,
            }),
        }
    }
    // Community property: a step of the filers section after About you, on a
    // separate return only. It asks unanswered until saved and reopens with
    // the saved answer; Back returns to About you.
    pub(super) fn community_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let (cookie, revision, page, previous) =
            (d.cookie.clone(), d.revision, PEOPLE_PAGE, PEOPLE_PAGE);
        let progress = self.progress(PEOPLE_PAGE);
        let estimate = self.estimate_now(TaxSection::People);
        match d.setup.as_ref().and_then(|s| s.community_property.answer()) {
            Some(value) => Reply::Edit(P::SaveCommunityProperty {
                user: self.user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: Vec::new(),
                progress,
                value,
            }),
            None => Reply::Response(P::BeginCommunityProperty {
                cookie,
                revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                progress,
            }),
        }
    }
    pub(super) fn community_candidate(
        &self,
        value: CommunityAnswer,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let mut draft = self.draft.clone();
        match draft.setup.as_mut() {
            Some(setup) if setup.filing_status() == FederalStatus::Separate => {
                setup.community_property = value.into();
                Ok(draft)
            }
            _ => Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into()),
        }
    }
    // Each page's answers merged into the accepted return, exactly as its
    // save builds them; a refusal is the message the save would send.
    pub(super) fn setup_candidate(
        &self,
        value: &ReturnSetup,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        // The page as it opened: navigation, whatever the stored setup holds.
        if self.draft.setup.as_ref().is_some_and(|s| return_setup(s) == *value) {
            return Ok(self.draft.clone());
        }
        if let Some(issue) = setup_issue(value) {
            return Err(issue.in_field("value").into());
        }
        let setup =
            federal_setup(value, self.draft.setup.as_ref()).map_err(|(key, _)| status(key))?;
        let mut draft = self.draft.clone();
        draft.setup = Some(setup);
        Ok(draft)
    }
    pub(super) fn you_candidate(
        &self,
        you: &Filer,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        if let Some(issue) = filer_issue(you) {
            return Err(issue.in_field("you").into());
        }
        let d = &self.draft;
        let spouse_situations = d.whole_return() & !YOUR_SITUATIONS;
        if let Some(issue) = situations_issue(you.situations, YOUR_SITUATIONS, spouse_situations) {
            return Err(issue.at_field("situations").in_field("you").into());
        }
        let old = d.people.as_ref();
        let taxpayer = federal_person(you);
        let mut draft = d.clone();
        draft.people = Some(FederalPeople {
            parent_tax: old.and_then(|p| p.parent_tax.clone()),
            taxpayer,
            spouse: old.map_or(SpouseFiler::No, |p| p.spouse.clone()),
        });
        draft.reconcile_professional_election(&self.draft, Owner::Taxpayer);
        if let Some(key) = crate::extended_rules::professional_election_rule(&draft) {
            return Err(libertas_formatted_text(key, &[]).into());
        }
        Ok(draft)
    }
    pub(super) fn spouse_candidate(
        &self,
        spouse: &SpouseAnswers,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let spouse = &Filer::from(spouse.clone());
        if let Some(issue) = filer_issue(spouse) {
            return Err(issue.in_field("spouse").into());
        }
        let yours = self.draft.whole_return() & YOUR_SITUATIONS;
        if let Some(issue) = situations_issue(spouse.situations, SPOUSE_SITUATIONS, yours) {
            return Err(issue.at_field("situations").in_field("spouse").into());
        }
        // The spouse page follows the saved filer page; there is no
        // filers record to add a spouse to before it.
        if self.draft.people.is_none() {
            return Err(crate::interview::status(
                "MESSAGE_SAVE_YOUR_OWN_FILER_PAGE_BEFORE_YOUR_SPOUSE_S",
            )
            .into());
        }
        let spouse = federal_person(spouse);
        let mut draft = self.draft.clone();
        draft.people.as_mut().unwrap().spouse = SpouseFiler::Yes { facts: spouse };
        draft.reconcile_professional_election(&self.draft, Owner::Spouse);
        if let Some(key) = crate::extended_rules::professional_election_rule(&draft) {
            return Err(libertas_formatted_text(key, &[]).into());
        }
        Ok(draft)
    }
}
pub(super) fn filing_key(status: FederalStatus) -> &'static str {
    match status {
        FederalStatus::Single => "REVIEW_FILING_NAME_SINGLE",
        FederalStatus::Joint => "REVIEW_FILING_NAME_JOINT",
        FederalStatus::Separate => "REVIEW_FILING_NAME_SEPARATE",
        FederalStatus::Head => "REVIEW_FILING_NAME_HEAD",
        FederalStatus::Surviving => "REVIEW_FILING_NAME_SURVIVING",
    }
}
// The M1 setup and filer pages are projections of the stored draft. Facts
// the pages no longer ask keep their saved value, or their "nothing applies"
// value on a new return, until a later milestone adds them to a page.
// The return's own name, built from the tax year.
pub(super) fn return_label() -> Vec<u8> {
    crate::Message::new("LABEL_RETURN", alloc::vec![crate::MessageArg::Number(2026)]).encode()
}
// The answers a filer nobody can claim gives to the dependency tests.
pub(super) const NOBODY_CLAIMS: Claimable = Claimable::Possibly {
    another_eitc_child: false,
    dependency: FilerDependency::Independent,
    us_home_over_half_year: true,
    full_time_student: false,
    disabled: false,
    incapable_self_care: false,
    support_share: SupportShare::MoreThanHalf,
    parent_alive: false,
};
// Household source and home-cost guards, shared by normal Save and Estimate.
pub(super) fn setup_issue(value: &ReturnSetup) -> Option<ValidationIssue> {
    match value.filing {
        Filing::HeadOfHousehold { paid_over_half_home: false, .. } => Some(
            ValidationIssue::new("HOH_NEEDS_HOME_COST")
                .at_field("paid_over_half_home")
                .in_field("filing"),
        ),
        Filing::QualifyingSurvivingSpouse { paid_over_half_home: false, .. } => Some(
            ValidationIssue::new("QSS_NEEDS_HOME_COST")
                .at_field("paid_over_half_home")
                .in_field("filing"),
        ),
        _ => None,
    }
}
// One native death-date owner for the Filer and losslessly projected spouse input.
pub(super) fn filer_issue(value: &Filer) -> Option<ValidationIssue> {
    let MoreAboutYou::Died { date, .. } = &value.details.more else {
        return None;
    };
    if !crate::federal_rules::valid_date(*date) || !(20260101..=20261231).contains(date) {
        Some(
            ValidationIssue::new("DEATH_DATE_OUTSIDE_2026")
                .at_field("date")
                .in_field("more")
                .in_field("details"),
        )
    } else if *date < value.birth_date {
        Some(
            ValidationIssue::new("DEATH_BEFORE_BIRTH")
                .at_field("date")
                .in_field("more")
                .in_field("details"),
        )
    } else {
        None
    }
}
pub(crate) fn federal_setup(
    value: &ReturnSetup,
    old: Option<&FederalSetup>,
) -> Result<FederalSetup, (&'static str, &'static str)> {
    let married = || match old.map(|o| o.marital_state) {
        Some(state @ MaritalState::Married { .. }) => state,
        _ => MaritalState::Married { living: SpouseLiving::Together },
    };
    // The statuses that do not ask about the home keep its saved answer.
    let kept_home = old.is_some_and(|o| o.paid_over_half_home);
    // A separate return keeps the facts its own pages ask; every other status
    // stores them unanswered.
    let kept_separate = old.filter(|o| {
        matches!(o.status, FilingChoice::Separate { .. })
            && value.filing == Filing::MarriedSeparately
    });
    let (status, marital_state, paid_over_half_home) = match value.filing {
        Filing::Single => (
            FilingChoice::Single,
            match old.map(|o| o.marital_state) {
                Some(MaritalState::Married { .. } | MaritalState::MarriedNonresidentSpouse)
                | None => MaritalState::Unmarried,
                Some(state) => state,
            },
            kept_home,
        ),
        Filing::MarriedJointly => (FilingChoice::Joint, married(), kept_home),
        Filing::MarriedSeparately => {
            // A return that becomes separate holds no spouse records until its
            // business page saves them; the rules read that as records not
            // given, never as an answer.
            let section179 = match kept_separate.map(|o| o.status) {
                Some(FilingChoice::Separate { section179 }) => section179,
                _ => crate::Section179Spouse::NotSupplied,
            };
            (FilingChoice::Separate { section179 }, married(), kept_home)
        }
        Filing::HeadOfHousehold { paid_over_half_home, marital } => (
            FilingChoice::Head,
            match marital {
                HeadMarital::Unmarried => MaritalState::Unmarried,
                HeadMarital::LivingApart => {
                    MaritalState::Married { living: SpouseLiving::LastSixMonths }
                }
                HeadMarital::NonresidentSpouse => MaritalState::MarriedNonresidentSpouse,
            },
            paid_over_half_home,
        ),
        Filing::QualifyingSurvivingSpouse {
            spouse_died,
            paid_over_half_home,
            could_file_joint,
        } => (
            FilingChoice::Surviving,
            match spouse_died {
                DeathYear::DiedPriorYear => MaritalState::WidowedPriorYear { could_file_joint },
                DeathYear::DiedTwoYearsPrior => {
                    MaritalState::WidowedTwoYearsPrior { could_file_joint }
                }
            },
            paid_over_half_home,
        ),
    };
    Ok(FederalSetup {
        label: return_label(),
        basis: old.map_or(AmountBasis::Actual, |o| o.basis),
        status,
        marital_state,
        paid_over_half_home,
        separate_spouse: kept_separate
            .map_or(crate::SeparateSpouseBoxes::NotAnswered, |o| o.separate_spouse),
        community_property: kept_separate
            .map_or(CommunityProperty::NotAnswered, |o| o.community_property),
    })
}
pub(crate) fn return_setup(setup: &FederalSetup) -> ReturnSetup {
    ReturnSetup {
        filing: match setup.status {
            FilingChoice::Single => Filing::Single,
            FilingChoice::Joint => Filing::MarriedJointly,
            FilingChoice::Separate { .. } => Filing::MarriedSeparately,
            FilingChoice::Head => Filing::HeadOfHousehold {
                paid_over_half_home: setup.paid_over_half_home,
                marital: match setup.marital_state {
                    MaritalState::Married { .. } => HeadMarital::LivingApart,
                    MaritalState::MarriedNonresidentSpouse => HeadMarital::NonresidentSpouse,
                    _ => HeadMarital::Unmarried,
                },
            },
            FilingChoice::Surviving => Filing::QualifyingSurvivingSpouse {
                spouse_died: match setup.marital_state {
                    MaritalState::WidowedTwoYearsPrior { .. } => DeathYear::DiedTwoYearsPrior,
                    _ => DeathYear::DiedPriorYear,
                },
                paid_over_half_home: setup.paid_over_half_home,
                could_file_joint: setup.could_file_joint_death_year(),
            },
        },
    }
}
// The whole-return situations each filer page offers.
pub(super) const YOUR_SITUATIONS: u32 =
    YourSituations::DUAL_STATUS.0 | YourSituations::COMMUNITY_PROPERTY.0;
pub(super) const SPOUSE_SITUATIONS: u32 = SpouseSituations::NONRESIDENT_SPOUSE.0
    | SpouseSituations::INJURED_SPOUSE.0
    | SpouseSituations::INNOCENT_SPOUSE.0
    | SpouseSituations::NIIT_JOINT_ELECTION.0
    | SpouseSituations::YOU_WERE_THE_NONRESIDENT.0;
// A page's whole-return answer against the other page's: only the page's own
// items; the resident choice for a nonresident spouse (IRC 6013(g), (h))
// makes both spouses residents for the whole year, so it excludes a
// dual-status return; the net investment income tax items need that choice.
pub(super) fn situations_issue(value: u32, offered: u32, other: u32) -> Option<ValidationIssue> {
    let all = value | other;
    if value & !offered != 0 {
        Some(ValidationIssue::new("MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS"))
    } else if all & YourSituations::DUAL_STATUS.0 != 0
        && all & SpouseSituations::NONRESIDENT_SPOUSE.0 != 0
    {
        Some(ValidationIssue::new("WHOLE_RETURN_RESIDENT_CHOICE_OR_DUAL_STATUS"))
    } else if value
        & (SpouseSituations::NIIT_JOINT_ELECTION.0 | SpouseSituations::YOU_WERE_THE_NONRESIDENT.0)
        != 0
        && value & SpouseSituations::NONRESIDENT_SPOUSE.0 == 0
    {
        Some(ValidationIssue::new("WHOLE_RETURN_NIIT_NEEDS_THE_RESIDENT_CHOICE"))
    } else {
        None
    }
}
pub(crate) fn federal_person(value: &Filer) -> FederalPerson {
    let tests = match &value.details.claimable {
        Claimable::Nobody => NOBODY_CLAIMS,
        other => other.clone(),
    };
    let Claimable::Possibly {
        another_eitc_child,
        dependency,
        us_home_over_half_year,
        full_time_student,
        disabled,
        incapable_self_care,
        support_share,
        parent_alive,
    } = tests
    else {
        unreachable!("Nobody is replaced by its answers")
    };
    FederalPerson {
        birth_date: value.birth_date,
        blind: value.details.blind,
        died: match &value.details.more {
            MoreAboutYou::Nothing => None,
            MoreAboutYou::Died { date, estate_medical, bonds, ordinary } => {
                Some(crate::FinalReturn {
                    bonds: bonds.clone(),
                    estate_medical: *estate_medical,
                    date: *date,
                    ordinary: *ordinary,
                })
            }
        },
        another_eitc_child,
        dependency,
        valid_ssn: value.identification == Identification::SocialSecurityNumber,
        valid_tin: value.identification != Identification::NoneByDueDate,
        us_home_over_half_year,
        full_time_student,
        disabled,
        incapable_self_care,
        support_share,
        parent_alive,
        first_name: value.first_name.clone(),
        last_name: value.last_name.clone(),
        real_estate_professional: value.details.real_estate_professional.clone().map(Into::into),
        whole_return: value.situations,
    }
}
pub(super) fn professional_election(
    d: &FederalDraft,
    owner: Owner,
) -> crate::ProfessionalRentalElection {
    if d.professional(owner).is_some_and(|p| p.single_activity) {
        crate::ProfessionalRentalElection::SingleActivity
    } else {
        crate::ProfessionalRentalElection::SeparateActivities
    }
}
pub(super) fn professional_filer(d: &FederalDraft, owner: Owner, person: &FederalPerson) -> Filer {
    let mut value = filer(person);
    if d.professional_joint()
        && let Some(status) = value.details.real_estate_professional.as_mut()
    {
        status.single_activity = if d.professional(owner).is_some_and(|p| p.single_activity) {
            crate::federal_model::ProfessionalRentalElection::SingleActivity
        } else {
            crate::federal_model::ProfessionalRentalElection::SeparateActivities
        };
    }
    value
}
pub(crate) fn filer(person: &FederalPerson) -> Filer {
    let tests = Claimable::Possibly {
        another_eitc_child: person.another_eitc_child,
        dependency: person.dependency.clone(),
        us_home_over_half_year: person.us_home_over_half_year,
        full_time_student: person.full_time_student,
        disabled: person.disabled,
        incapable_self_care: person.incapable_self_care,
        support_share: person.support_share,
        parent_alive: person.parent_alive,
    };
    Filer {
        first_name: person.first_name.clone(),
        last_name: person.last_name.clone(),
        birth_date: person.birth_date,
        identification: if person.valid_ssn {
            Identification::SocialSecurityNumber
        } else if !person.valid_tin {
            Identification::NoneByDueDate
        } else {
            Identification::Itin
        },
        details: crate::FilerDetails {
            blind: person.blind,
            claimable: if tests == NOBODY_CLAIMS { Claimable::Nobody } else { tests },
            more: match &person.died {
                None => MoreAboutYou::Nothing,
                Some(died) => MoreAboutYou::Died {
                    date: died.date,
                    estate_medical: died.estate_medical,
                    bonds: died.bonds.clone(),
                    ordinary: died.ordinary,
                },
            },
            real_estate_professional: person.real_estate_professional.clone().map(Into::into),
        },
        situations: person.whole_return,
    }
}

impl Interview {
    pub(super) fn save_setup(
        &self,
        value: &ReturnSetup,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != SETUP_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.setup_candidate(value)
    }
}
