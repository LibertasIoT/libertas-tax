// Coverage: direct Schedule C/F participation retains actual annual work,
// selected tests and qualifying histories. Reg. 1.469-5T and IRS Pub. 925:
// https://www.irs.gov/publications/p925
use crate::participation::{OrdinaryEvidence, Participation};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_number};

/// Qualifying work in this trade or business
/// Include the owner's and spouse's work even on separate returns. Exclude investor-only monitoring and work not customarily done by owners whose main purpose is avoiding passive-loss limits. Use time records or another reasonable record of services. This is not the rental real-estate-professional test.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ParticipationFacts {
    /// Combined qualifying time during the return year
    // Calendar capacity: two spouses can each work 8,784 hours in a leap year.
    #[libertas_number(min = 0, max = 17568)]
    #[libertas_physical_unit("hour")]
    pub hours: u32,
    /// Material-participation test supported by your records
    pub test: ParticipationTest,
}

/// Actual qualifying historical tax year
#[libertas_number(min = 1900, max = $RISK_PRIOR_YEAR)]
pub type ParticipationYear = u32;

/// Material-participation test
/// Select the test your records establish. Historical years must be actual qualifying years, not years inferred from business ownership.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ParticipationTest {
    /// No individual test applies
    /// Keep the actual hours; the App also checks the combined significant-participation test.
    None,
    /// More than 500 qualifying hours
    Over500,
    /// Your work was substantially all participation by everyone
    /// Include employees, contractors and nonowners when comparing work.
    SubstantiallyAll,
    /// More than 100 hours and no other individual worked more
    Over100NoOneMore {
        /// Time worked by the other individual who participated most
        #[libertas_number(min = 0, max = 17568)]
        #[libertas_physical_unit("hour")]
        other_max_hours: u32,
    },
    /// Significant participation in several trades or businesses
    /// Each qualifying activity needs more than 100 hours and must not meet another material-participation test. The App combines actual activity hours and requires more than 500 total.
    SignificantActivities,
    /// Material participation in at least five of the preceding ten years
    FiveOfTen {
        /// Actual qualifying years
        /// Do not count a year qualified solely by this same five-of-ten test.
        #[libertas_unordered]
        years: Vec<ParticipationYear>,
    },
    /// Personal-service activity with at least three earlier qualifying years
    PersonalServiceThree {
        /// Actual prior material-participation years, not limited to the past ten
        #[libertas_unordered]
        years: Vec<ParticipationYear>,
        /// This is a personal-service activity
        /// Health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting, or another business where capital is not a material income-producing factor.
        personal_services: bool,
    },
    /// Regular, continuous and substantial participation
    FactsCircumstances {
        /// Your participation was regular, continuous and substantial
        regular_continuous_substantial: bool,
        /// Management time included in the combined total
        #[libertas_number(min = 0, max = 17568)]
        #[libertas_physical_unit("hour")]
        management_hours: u32,
        /// Another person was paid to manage this activity
        paid_other_manager: bool,
        /// Management time of the other individual who managed most
        #[libertas_number(min = 0, max = 17568)]
        #[libertas_physical_unit("hour")]
        other_management_hours: u32,
    },
    /// Retired or disabled farmer with five qualifying years out of eight
    RetiredDisabledFarmer {
        /// Year of retirement or disability
        #[libertas_number(min = 1900, max = $RISK_TAX_YEAR)]
        retired_or_disabled: u32,
        /// Actual material-participation years in the eight preceding years
        #[libertas_unordered]
        qualifying_years: Vec<ParticipationYear>,
    },
    /// Surviving spouse of a farmer
    SurvivingFarmer {
        /// The farm real estate meets the estate-tax special-valuation rules
        special_estate_valuation: bool,
        /// You actively manage this farm
        active_management: bool,
    },
}

impl ParticipationFacts {
    pub(crate) fn valid(&self, year: u32, farming: bool) -> bool {
        use ParticipationTest::*;
        if self.hours > 17_568 {
            return false;
        }
        let years_valid = |years: &[u32], first: u32, last: u32, minimum: usize| {
            years.len() >= minimum
                && years.iter().all(|v| (first..=last).contains(v))
                && years.iter().enumerate().all(|(i, v)| !years[..i].contains(v))
        };
        match &self.test {
            None => self.hours <= 500,
            Over500 => self.hours > 500,
            SubstantiallyAll => self.hours > 0,
            Over100NoOneMore { other_max_hours } => {
                self.hours > 100 && self.hours >= *other_max_hours
            }
            SignificantActivities => self.hours > 100 && self.hours <= 500,
            FiveOfTen { years } => {
                years_valid(years, year.saturating_sub(10).max(1900), year.saturating_sub(1), 5)
            }
            PersonalServiceThree { years, personal_services } => {
                *personal_services && years_valid(years, 1900, year.saturating_sub(1), 3)
            }
            FactsCircumstances { management_hours, other_management_hours, .. } => {
                *management_hours <= self.hours && *other_management_hours <= 17_568
            }
            RetiredDisabledFarmer { retired_or_disabled, qualifying_years } => {
                farming
                    && (1900..=year).contains(retired_or_disabled)
                    && years_valid(
                        qualifying_years,
                        retired_or_disabled.saturating_sub(8).max(1900),
                        retired_or_disabled.saturating_sub(1),
                        5,
                    )
            }
            SurvivingFarmer { .. } => farming,
        }
    }

    // Reg. 1.179-2(c)(6) asks for meaningful current management or operations,
    // not the section 469 historical/deemed result. Qualifying current work
    // excludes investor-only monitoring (Reg. 1.469-5T(f)(2)). These existing
    // current-work answers provide sufficient evidence; false leaves the
    // current-conduct question unestablished, rather than classifying inactivity.
    fn establishes_current_active_conduct(&self, year: u32, farming: bool) -> bool {
        if !self.valid(year, farming) {
            return false;
        }
        if self.hours > 500 {
            return true;
        }
        match self.test {
            ParticipationTest::Over500
            | ParticipationTest::SubstantiallyAll
            | ParticipationTest::Over100NoOneMore { .. }
            | ParticipationTest::SignificantActivities => true,
            ParticipationTest::FactsCircumstances { regular_continuous_substantial, .. } => {
                regular_continuous_substantial && self.hours > 0
            }
            ParticipationTest::SurvivingFarmer { active_management, .. } => active_management,
            _ => false,
        }
    }

    pub(crate) fn individual(&self, year: u32, farming: bool) -> Participation {
        use ParticipationTest::*;
        if !self.valid(year, farming) {
            return Participation::Unavailable;
        }
        if matches!(
            &self.test,
            RetiredDisabledFarmer { .. }
                | SurvivingFarmer { special_estate_valuation: true, active_management: true }
        ) {
            return Participation::Yes;
        }
        let mut evidence = OrdinaryEvidence {
            minutes: self.hours * 60,
            substantially_all: false,
            at_least_as_much: false,
            prior_ten: 0,
            personal_service_years: 0,
            regular: false,
            management_minutes: 0,
            disqualified_management: false,
        };
        match &self.test {
            SubstantiallyAll => evidence.substantially_all = true,
            Over100NoOneMore { .. } => evidence.at_least_as_much = true,
            FiveOfTen { years } => evidence.prior_ten = years.len() as u32,
            PersonalServiceThree { years, .. } => {
                evidence.personal_service_years = years.len() as u32
            }
            FactsCircumstances {
                regular_continuous_substantial,
                management_hours,
                paid_other_manager,
                other_management_hours,
            } => {
                evidence.regular = *regular_continuous_substantial;
                evidence.management_minutes = *management_hours * 60;
                evidence.disqualified_management =
                    *paid_other_manager || *other_management_hours > *management_hours;
            }
            _ => {}
        }
        evidence.material()
    }
}

/// One accepted direct source's actual hours and derived classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DirectParticipationSource {
    pub id: i64,
    pub hours: u32,
    pub result: Participation,
    pub significant: bool,
}

impl crate::FederalDraft {
    pub(crate) fn direct_participation_sources(&self) -> Vec<DirectParticipationSource> {
        let year = u32::try_from(self.tax_year).unwrap_or(0);
        let mut sources: Vec<_> = self
            .income
            .iter()
            .filter_map(|entry| {
                let crate::FederalIncome::Business { data } = &entry.income else {
                    return None;
                };
                let facts = &data.material_participation;
                let result = facts.individual(
                    year,
                    matches!(data.activity, crate::BusinessActivity::Farming { .. }),
                );
                Some(DirectParticipationSource {
                    id: entry.id,
                    hours: facts.hours,
                    result,
                    // Reg. 1.469-5T(c)(1): casual gambling is not a trade or business.
                    significant: result == Participation::No
                        && facts.hours > 100
                        && !matches!(
                            data.activity,
                            crate::BusinessActivity::Gambling { eligible: false, .. }
                        ),
                })
            })
            .collect();
        let mut external_unavailable = false;
        let mut external_minutes = 0u64;
        for entry in &self.income {
            match crate::k1_activity::ordinary_participation_sources(&entry.income) {
                None => external_unavailable = true,
                Some(records) => {
                    for facts in records {
                        if !facts.ordinary_scope || !facts.valid() {
                            external_unavailable = true;
                        } else if facts.material() == Participation::No && facts.minutes > 6_000 {
                            external_minutes += u64::from(facts.minutes);
                        }
                    }
                }
            }
        }
        let invalid = sources.iter().any(|source| source.result == Participation::Unavailable);
        let minutes: u64 = external_minutes
            + sources
                .iter()
                .filter(|source| source.significant)
                .map(|source| u64::from(source.hours) * 60)
                .sum::<u64>();
        for source in &mut sources {
            if source.significant {
                source.result = if minutes > 30_000 {
                    Participation::Yes
                } else if invalid || external_unavailable {
                    Participation::Unavailable
                } else {
                    Participation::No
                };
            }
        }
        sources
    }
}

/// A collector's transient, source-bound classification; never persisted.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DirectParticipationView(alloc::collections::BTreeMap<i64, (Participation, bool)>);
impl DirectParticipationView {
    pub(crate) fn get(&self, id: i64) -> Participation {
        self.0.get(&id).map_or(Participation::Unavailable, |source| source.0)
    }

    pub(crate) fn active_conduct_established(&self, id: i64) -> bool {
        self.0.get(&id).is_some_and(|source| source.1)
    }
}
impl crate::FederalDraft {
    pub(crate) fn direct_participation_view(&self) -> alloc::sync::Arc<DirectParticipationView> {
        let year = u32::try_from(self.tax_year).unwrap_or(0);
        // Section 469 attributes spousal work even on separate returns; section
        // 179(c)(8) treats those taxpayers separately. Combined hours cannot
        // establish an individual spouse's current conduct on an MFS return.
        let attributable = self
            .setup
            .as_ref()
            .is_some_and(|setup| setup.filing_status() != crate::FederalStatus::Separate);
        alloc::sync::Arc::new(DirectParticipationView(
            self.direct_participation_sources()
                .into_iter()
                .map(|source| {
                    let active = attributable && self.income.iter().any(|entry| {
                        entry.id == source.id
                            && matches!(&entry.income,
                            crate::FederalIncome::Business { data }
                            if data.material_participation.establishes_current_active_conduct(
                                year,
                                matches!(data.activity, crate::BusinessActivity::Farming { .. }),
                            ))
                    });
                    (source.id, (source.result, active))
                })
                .collect(),
        ))
    }
}
