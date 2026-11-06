// IRC163(j)(7), Form8990 Special Rules: the actual activity election
// requires ADS for affected property and excludes its special allowance.
// https://www.irs.gov/instructions/i8990
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual election out of the business interest limit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InterestLimitElection {
    /// Election shown on the activity's original election statement
    pub kind: InterestLimitElectionKind,
    /// First tax year for which the election is effective
    pub effective_year: u32,
}
/// Statutory kind identified by the actual election statement
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InterestLimitElectionKind {
    /// Electing real property trade or business
    RealProperty,
    /// Electing farming business
    Farming,
}
/// Actual election record
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum InterestLimitElectionRecord {
    /// Election record
    Record { election: InterestLimitElection },
}
impl From<InterestLimitElection> for InterestLimitElectionRecord {
    fn from(election: InterestLimitElection) -> Self {
        Self::Record { election }
    }
}
impl From<InterestLimitElectionRecord> for InterestLimitElection {
    fn from(record: InterestLimitElectionRecord) -> Self {
        let InterestLimitElectionRecord::Record { election } = record;
        election
    }
}
impl InterestLimitElection {
    pub(crate) fn affects(&self, class: crate::EquipmentClass, year: u32) -> bool {
        self.effective_year <= year
            && match self.kind {
                InterestLimitElectionKind::RealProperty => {
                    class == crate::EquipmentClass::ThirtyNine
                }
                InterestLimitElectionKind::Farming => matches!(
                    class,
                    crate::EquipmentClass::Ten
                        | crate::EquipmentClass::Fifteen
                        | crate::EquipmentClass::Twenty
                        | crate::EquipmentClass::ThirtyNine
                ),
            }
    }
    pub(crate) fn reason(&self) -> crate::macrs_source::AdsReason {
        use crate::macrs_source::AdsReason;
        match self.kind {
            InterestLimitElectionKind::RealProperty => AdsReason::RealPropertyTrade,
            InterestLimitElectionKind::Farming => AdsReason::FarmingTrade,
        }
    }
}
pub(crate) fn source(income: &crate::FederalIncome) -> Option<&InterestLimitElection> {
    match income {
        crate::FederalIncome::Business { data } => data.interest_limit_election.as_ref(),
        crate::FederalIncome::Rental { data } => data.interest_limit_election.as_ref(),
        crate::FederalIncome::FarmRental { data } => data.interest_limit_election.as_ref(),
        _ => None,
    }
}
pub(crate) fn issue(
    income: &crate::FederalIncome,
    year: u32,
) -> Option<crate::validation::ValidationIssue> {
    let election = source(income)?;
    let kind_ok = match income {
        crate::FederalIncome::Business { data } => match data.activity {
            crate::BusinessActivity::Farming { .. } => {
                election.kind == InterestLimitElectionKind::Farming
            }
            crate::BusinessActivity::Ordinary { .. } => {
                election.kind == InterestLimitElectionKind::RealProperty
            }
            crate::BusinessActivity::Gambling { .. } => false,
        },
        crate::FederalIncome::Rental { .. } => {
            election.kind == InterestLimitElectionKind::RealProperty
        }
        crate::FederalIncome::FarmRental { .. } => true,
        _ => false,
    };
    // The statutory election first applied to tax years beginning after2017.
    (!(2018..=year).contains(&election.effective_year) || !kind_ok).then(|| {
        crate::validation::ValidationIssue::new("INTEREST_LIMIT_ELECTION_SOURCE")
            .at_field("interest_limit_election")
    })
}
pub(crate) fn choices(needed: bool) -> Vec<i32> {
    if needed {
        alloc::vec![libertas_macros::variant_index!(InterestLimitElectionRecord::Record) as i32]
    } else {
        Vec::new()
    }
}

fn real_property(
    class: crate::AssetClass,
    source: Option<&crate::MacrsRecoverySourcePage>,
) -> bool {
    class == crate::AssetClass::ThirtyNine
        || source.and_then(|source| source.improvement.as_ref()).is_some_and(|improvement| {
            matches!(improvement.kind, crate::RealImprovementKind::Interior1250 { .. })
        })
}
fn farm_property(asset: &crate::Asset) -> bool {
    matches!(&asset.recovery, crate::AssetRecoveryPage::Gds { facts }
        if matches!(facts.class, crate::AssetClass::Ten | crate::AssetClass::Fifteen |
            crate::AssetClass::Twenty | crate::AssetClass::ThirtyNine))
}
fn real_property_asset(asset: &crate::Asset) -> bool {
    matches!(&asset.recovery, crate::AssetRecoveryPage::Gds { facts }
        if real_property(facts.class, facts.source.as_ref()))
}
fn building_property(cost: i64, land: i64, in_service: u32) -> bool {
    cost > land && in_service >= 19870101
}
/// Opening and Save use the actual complete activity, never a client classification snapshot.
pub(crate) fn needed(page: &crate::IncomeRecord) -> bool {
    match page {
        crate::IncomeRecord::Business(page) => {
            page.more.gambling.is_none() && page.assets.iter().any(real_property_asset)
        }
        crate::IncomeRecord::Farm(page) => page.assets.iter().any(farm_property),
        crate::IncomeRecord::FarmRental(page) => {
            page.assets.iter().any(|asset| farm_property(asset) || real_property_asset(asset))
        }
        crate::IncomeRecord::Rental(page) => {
            (!matches!(page.kind, crate::RentalPropertyType::Land)
                && (building_property(
                    page.building.cost,
                    page.building.land,
                    page.building.in_service,
                ) || page
                    .more
                    .improvements
                    .iter()
                    .any(|building| building_property(building.cost, 0, building.in_service))))
                || page.assets.iter().any(|asset| match asset {
                    crate::CanonicalAssetPage::Gds { facts } => {
                        facts.financial.class == crate::GdsEquipmentClass::ThirtyNine
                            || facts
                                .financial
                                .source
                                .as_ref()
                                .and_then(|s| s.improvement.as_ref())
                                .is_some_and(|i| {
                                    matches!(
                                        i.kind,
                                        crate::RealImprovementKind::Interior1250 { .. }
                                    )
                                })
                    }
                    crate::CanonicalAssetPage::NonMacrs { .. } => false,
                })
        }
        _ => false,
    }
}
/// Refresh only derived presentation choices; no source answer is changed.
pub(crate) fn refresh(page: &mut crate::IncomeRecord, retained: bool) -> bool {
    let needed = needed(page) || retained;
    let (answer, allowed) = match page {
        crate::IncomeRecord::Business(p) => {
            (&p.interest_limit_election, &mut p.allowed_interest_limit_election)
        }
        crate::IncomeRecord::Farm(p) => {
            (&p.interest_limit_election, &mut p.allowed_interest_limit_election)
        }
        crate::IncomeRecord::Rental(p) => {
            (&p.interest_limit_election, &mut p.allowed_interest_limit_election)
        }
        crate::IncomeRecord::FarmRental(p) => {
            (&p.interest_limit_election, &mut p.allowed_interest_limit_election)
        }
        _ => return false,
    };
    let next = choices(needed || answer.is_some());
    let ask = answer.is_none() && next.iter().any(|value| !allowed.contains(value));
    *allowed = next;
    ask
}

// Foreign buildings already have their independently required ADS schedule.
// Domestic elected buildings need their actual ADS recovery and transition history.
pub(crate) fn rental_building_issue(
    property: &crate::FederalRental,
) -> Option<(usize, crate::validation::ValidationIssue)> {
    let source = property.interest_limit_election.as_ref()?;
    if source.kind != InterestLimitElectionKind::RealProperty
        || source.effective_year > crate::year_constants::TAX_YEAR
        || property.location.foreign()
        || matches!(property.property_type, crate::RentalPropertyType::Land)
    {
        return None;
    }
    property.buildings.iter().enumerate().find_map(|(index, building)| {
        (building.cost > building.land && building.in_service >= 19870101).then(|| {
            (
                index,
                crate::validation::ValidationIssue::new(
                    "RENTAL_INTEREST_ELECTION_ADS_HISTORY_UNAVAILABLE",
                )
                .at_field("in_service")
                .in_list("buildings", index),
            )
        })
    })
}
