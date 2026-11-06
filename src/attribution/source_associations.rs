// Borrowed accepted-record adapter; the kernel owns identity lookup and ambiguity.
// The normal accepted Save path owns global IDs beyond these income/property rows.
// Parent identity is read from the actual containing source, never inferred from
// a position, label, amount or another property's result.
use crate::kernel::associations::{AssociationSource, AssociationView, Relation, SourceKind};
use crate::kernel::source_fact::SourceFact;
use crate::{BusinessAsset, FederalDraft, FederalIncome, FederalIncomeEntry, Owner};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceAssociationIssue {
    IdentityUnavailable(i64),
    UnknownSource(i64),
    AmbiguousSource(i64),
    WrongSourceKind(i64),
    ParentMismatch { source: i64, parent: i64 },
}

// Income-only consumers keep their native lookup domain. Nested assets and
// K-1 children are registered only by the full SourceAssociations adapter.
impl AssociationSource for FederalIncomeEntry {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> SourceKind {
        if matches!(self.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. }) {
            SourceKind::K1
        } else {
            SourceKind::Income
        }
    }
    fn owner(&self) -> SourceFact<Owner> {
        SourceFact::Reported(self.owner.owned())
    }
    fn parent(&self, _: Relation) -> SourceFact<i64> {
        SourceFact::NotReported
    }
}

fn unique_source<T: AssociationSource>(
    sources: &[T],
    id: i64,
) -> Result<&T, SourceAssociationIssue> {
    if id <= 0 {
        return Err(SourceAssociationIssue::IdentityUnavailable(id));
    }
    match AssociationView::new(sources).source(id) {
        SourceFact::Reported(source) => Ok(source),
        _ => Err(if sources.iter().any(|source| source.id() == id) {
            SourceAssociationIssue::AmbiguousSource(id)
        } else {
            SourceAssociationIssue::UnknownSource(id)
        }),
    }
}

pub(crate) fn income_source(
    draft: &FederalDraft,
    id: i64,
) -> Result<&FederalIncomeEntry, SourceAssociationIssue> {
    unique_source(&draft.income, id)
}

#[derive(Clone, Copy)]
enum Source<'a> {
    EsppAcquisition(&'a crate::EsppAcquisition),
    Income(&'a FederalIncomeEntry),
    ImprovementBuilding {
        parent: Option<&'a FederalIncomeEntry>,
        owner: Owner,
        facts: &'a crate::ImprovementBuildingFacts,
    },
    EmployeeAsset {
        owner: Owner,
        asset: &'a BusinessAsset,
    },
    RentalBuilding {
        parent: &'a FederalIncomeEntry,
        facts: &'a crate::RentalEnergyBuildingFacts,
    },
    RentalCost {
        parent: &'a FederalIncomeEntry,
        cost: &'a crate::RentalBuilding,
    },
    Mileage {
        parent: &'a FederalIncomeEntry,
        mileage: &'a crate::BusinessMileage,
    },
    Intangible {
        parent: &'a FederalIncomeEntry,
        intangible: &'a crate::BusinessIntangible,
    },
    Asset {
        parent: &'a FederalIncomeEntry,
        asset: &'a BusinessAsset,
    },
    Other {
        parent: &'a FederalIncomeEntry,
        id: i64,
        kind: SourceKind,
    },
    // A saved personal casualty event (no property) or one of its properties.
    Casualty {
        event: &'a crate::PersonalCasualty,
        property: Option<&'a crate::CasualtyProperty>,
    },
}
impl Source<'_> {
    fn income(&self) -> Option<&FederalIncomeEntry> {
        match self {
            Self::Income(parent)
            | Self::RentalBuilding { parent, .. }
            | Self::RentalCost { parent, .. }
            | Self::Intangible { parent, .. }
            | Self::Asset { parent, .. }
            | Self::Mileage { parent, .. }
            | Self::Other { parent, .. } => Some(parent),
            Self::ImprovementBuilding { parent, .. } => *parent,
            Self::EmployeeAsset { .. } | Self::Casualty { .. } | Self::EsppAcquisition(_) => None,
        }
    }
}
impl AssociationSource for Source<'_> {
    fn id(&self) -> i64 {
        match self {
            Self::Income(entry) => entry.id(),
            Self::EsppAcquisition(source) => source.id,
            Self::ImprovementBuilding { facts, .. } => facts.id,
            Self::EmployeeAsset { asset, .. } => asset.id,
            Self::RentalBuilding { facts, .. } => facts.id,
            Self::RentalCost { cost, .. } => cost.id,
            Self::Intangible { intangible, .. } => intangible.id,
            Self::Asset { asset, .. } => asset.id,
            Self::Mileage { mileage, .. } => mileage.id,
            Self::Other { id, .. } => *id,
            Self::Casualty { event, property } => property.map_or(event.id, |property| property.id),
        }
    }
    fn kind(&self) -> SourceKind {
        match self {
            Self::Income(entry) => entry.kind(),
            Self::EsppAcquisition(_) => SourceKind::Property,
            Self::Intangible { .. } => SourceKind::Property,
            Self::RentalBuilding { .. } | Self::ImprovementBuilding { .. } => SourceKind::Building,
            Self::EmployeeAsset { .. } => SourceKind::Equipment,
            Self::RentalCost { .. } => SourceKind::Component,
            Self::Asset { .. } | Self::Mileage { .. } => SourceKind::Equipment,
            Self::Other { kind, .. } => *kind,
            Self::Casualty { property: Some(_), .. } => SourceKind::Property,
            Self::Casualty { property: None, .. } => SourceKind::CasualtyEvent,
        }
    }
    fn owner(&self) -> SourceFact<Owner> {
        match self {
            Self::ImprovementBuilding { owner, .. } | Self::EmployeeAsset { owner, .. } => {
                SourceFact::Reported(*owner)
            }
            Self::EsppAcquisition(source) => SourceFact::Reported(source.owner),
            _ => self.income().map_or(SourceFact::Unavailable, |entry| entry.owner()),
        }
    }
    fn parent(&self, relation: Relation) -> SourceFact<i64> {
        match self {
            Self::Intangible { intangible, .. } if relation == Relation::Property => {
                match &intangible.source.obtained {
                    crate::IntangibleObtained::Renewal { previous_intangible_id: Some(id) } => {
                        SourceFact::Reported(*id)
                    }
                    _ => SourceFact::NotReported,
                }
            }
            Self::RentalCost { cost, .. } if relation == Relation::PhysicalBuilding => {
                cost.energy.as_ref().map_or(SourceFact::NotReported, |claim| {
                    SourceFact::Reported(claim.physical_building)
                })
            }
            Self::Asset { asset, .. } | Self::EmployeeAsset { asset, .. }
                if relation == Relation::PhysicalBuilding =>
            {
                asset
                    .macrs_source
                    .as_ref()
                    .and_then(|source| source.improvement.as_ref())
                    .map_or(SourceFact::NotReported, |improvement| {
                        SourceFact::Reported(improvement.building_id)
                    })
            }
            Self::Other { parent, kind: SourceKind::K1Child, .. }
                if relation == Relation::K1Parent =>
            {
                SourceFact::Reported(parent.id)
            }
            Self::Casualty { event, property: Some(_) } if relation == Relation::CasualtyEvent => {
                SourceFact::Reported(event.id)
            }
            _ => SourceFact::NotReported,
        }
    }
}

pub struct SourceAssociations<'a> {
    sources: Vec<Source<'a>>,
}
impl<'a> SourceAssociations<'a> {
    pub fn new(draft: &'a FederalDraft) -> Self {
        let mut sources = Vec::new();
        sources.extend(
            draft
                .screening
                .iter()
                .flat_map(|screening| &screening.espp_acquisitions)
                .map(Source::EsppAcquisition),
        );
        for parent in &draft.income {
            sources.push(Source::Income(parent));
            sources.extend(crate::macrs_records::improvement_buildings(&parent.income).iter().map(
                |facts| Source::ImprovementBuilding {
                    parent: Some(parent),
                    owner: parent.owner.owned(),
                    facts,
                },
            ));
            if let FederalIncome::Rental { data } = &parent.income {
                sources.extend(
                    data.energy_buildings
                        .iter()
                        .map(|facts| Source::RentalBuilding { parent, facts }),
                );
                sources.extend(
                    data.buildings
                        .iter()
                        .filter(|cost| cost.id > 0)
                        .map(|cost| Source::RentalCost { parent, cost }),
                );
            }
            sources.extend(
                crate::amortization_records::records(&parent.income)
                    .iter()
                    .map(|intangible| Source::Intangible { parent, intangible }),
            );
            let assets = match &parent.income {
                FederalIncome::Business { data } => data.assets.as_slice(),
                FederalIncome::FarmRental { data } => data.assets.as_slice(),
                FederalIncome::Rental { data } => data.assets.as_slice(),
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
                    ..
                } => assets.as_slice(),
                _ => &[],
            };
            sources.extend(assets.iter().map(|asset| Source::Asset { parent, asset }));
            sources.extend(
                crate::BusinessMileage::records(&parent.income)
                    .iter()
                    .map(|mileage| Source::Mileage { parent, mileage }),
            );
            let sales = match &parent.income {
                FederalIncome::Partnership { data } => data.section179_sales.as_slice(),
                FederalIncome::Scorp { data } => data.statements.section179_sales.as_slice(),
                _ => &[],
            };
            sources.extend(sales.iter().map(|sale| Source::Other {
                parent,
                id: sale.id,
                kind: SourceKind::K1Child,
            }));
            sources.extend(
                crate::investment_credit::k1_property_ids(&parent.income)
                    .into_iter()
                    .map(|id| Source::Other { parent, id, kind: SourceKind::K1Child }),
            );
        }
        for claim in draft.adjustments.iter().flat_map(|a| &a.employee_expenses) {
            sources.extend(claim.improvement_buildings.iter().map(|facts| {
                Source::ImprovementBuilding { parent: None, owner: claim.owner, facts }
            }));
            sources.extend(
                claim
                    .vehicle_assets
                    .iter()
                    .chain(&claim.equipment_assets)
                    .map(|asset| Source::EmployeeAsset { owner: claim.owner, asset }),
            );
        }
        // Saved personal casualty events and their properties share the
        // return-wide identities; a home-use rental names its dwelling by one.
        let events = draft.deductions.as_ref().map_or(&[][..], |d| &d.personal_casualties);
        for event in events {
            sources.push(Source::Casualty { event, property: None });
            sources.extend(
                event.properties.iter().map(|p| Source::Casualty { event, property: Some(p) }),
            );
        }
        Self { sources }
    }
    fn source(&self, id: i64) -> Result<Source<'a>, SourceAssociationIssue> {
        let source = unique_source(&self.sources, id)?;
        let Some(parent) = source.income().map(|entry| entry.id) else {
            return Ok(*source);
        };
        if parent <= 0 {
            return Err(SourceAssociationIssue::IdentityUnavailable(parent));
        }
        if !matches!(
            AssociationView::new(&self.sources).source(parent),
            SourceFact::Reported(Source::Income(_))
        ) {
            return Err(SourceAssociationIssue::AmbiguousSource(parent));
        }
        Ok(*source)
    }
    /// Select only unambiguous accepted physical-building identities.
    pub(crate) fn physical_building_choices(&self) -> Vec<crate::RecordChoice> {
        self.sources
            .iter()
            .filter_map(|source| {
                let (id, label) = match source {
                    Source::RentalBuilding { facts, .. } => (facts.id, &facts.building_source),
                    Source::ImprovementBuilding { facts, .. } => (facts.id, &facts.description),
                    _ => return None,
                };
                self.source(id).ok()?;
                Some(crate::RecordChoice {
                    id,
                    label: crate::Message::new(
                        "RENTAL_ENERGY_BUILDING_CHOICE",
                        alloc::vec![crate::MessageArg::Text(label.clone())],
                    )
                    .encode(),
                })
            })
            .collect()
    }
    // Recovery borrows the return-wide physical source. Section179D retains its
    // separate same-rental cost association and never supplies recovery facts.
    pub(crate) fn improvement_building_links(&self) -> Vec<i64> {
        self.sources
            .iter()
            .filter_map(|source| {
                let asset = match source {
                    Source::Asset { asset, .. } | Source::EmployeeAsset { asset, .. } => asset,
                    _ => return None,
                };
                if !asset.needs_building_source() {
                    return None;
                }
                match source.parent(Relation::PhysicalBuilding) {
                    SourceFact::Reported(id) if id > 0 => Some(id),
                    _ => None,
                }
            })
            .collect()
    }
    pub(crate) fn recovery_building(&self, id: i64) -> SourceFact<&'a crate::BuildingUseSource> {
        let Ok(source) = self.source(id) else { return SourceFact::Unavailable };
        match source {
            Source::ImprovementBuilding { facts, .. } => SourceFact::Reported(&facts.facts),
            Source::RentalBuilding { facts, .. } => {
                facts.recovery.as_ref().map_or(SourceFact::Unavailable, SourceFact::Reported)
            }
            _ => SourceFact::Unavailable,
        }
    }
    pub(crate) fn recovery_buildings(&self) -> Vec<(i64, &'a crate::BuildingUseSource)> {
        self.sources
            .iter()
            .filter_map(|source| {
                let SourceFact::Reported(facts) = self.recovery_building(source.id()) else {
                    return None;
                };
                Some((source.id(), facts))
            })
            .collect()
    }
    pub(crate) fn espp_acquisition(
        &self,
        id: i64,
        owner: Owner,
    ) -> Result<&'a crate::EsppAcquisition, SourceAssociationIssue> {
        match self.source(id)? {
            Source::EsppAcquisition(source) if source.owner == owner => Ok(source),
            _ => Err(SourceAssociationIssue::WrongSourceKind(id)),
        }
    }

    pub fn income(&self, id: i64) -> Result<&'a FederalIncomeEntry, SourceAssociationIssue> {
        match self.source(id)? {
            Source::Income(entry) => Ok(entry),
            _ => Err(SourceAssociationIssue::WrongSourceKind(id)),
        }
    }
    pub(crate) fn mileage(
        &self,
        parent: i64,
        id: i64,
    ) -> Result<&'a crate::BusinessMileage, SourceAssociationIssue> {
        self.income(parent)?;
        match self.source(id)? {
            Source::Mileage { parent: actual, mileage } if actual.id == parent => Ok(mileage),
            Source::Mileage { .. } => {
                Err(SourceAssociationIssue::ParentMismatch { source: id, parent })
            }
            _ => Err(SourceAssociationIssue::WrongSourceKind(id)),
        }
    }
    pub(crate) fn physical_building(
        &self,
        parent: i64,
        cost_id: i64,
    ) -> Result<&'a crate::RentalEnergyBuildingFacts, SourceAssociationIssue> {
        self.income(parent)?;
        let Source::RentalCost { parent: actual, .. } = self.source(cost_id)? else {
            return Err(SourceAssociationIssue::WrongSourceKind(cost_id));
        };
        if actual.id != parent {
            return Err(SourceAssociationIssue::ParentMismatch { source: cost_id, parent });
        }
        let SourceFact::Reported(id) =
            AssociationView::new(&self.sources).parent(cost_id, Relation::PhysicalBuilding)
        else {
            return Err(SourceAssociationIssue::UnknownSource(cost_id));
        };
        match self.source(id)? {
            Source::RentalBuilding { parent: actual, facts } if actual.id == parent => Ok(facts),
            _ => Err(SourceAssociationIssue::ParentMismatch { source: cost_id, parent }),
        }
    }
    // The saved personal casualty property that is this rental building or
    // improvement, with its event, so the dwelling's one loss and its event's
    // one Form 4684 Section A reduction are split between personal and rental
    // use once. The event's date and declaration must be the rental record's.
    // Ok(None) is a casualty that names no saved personal property.
    #[allow(dead_code)] // read by the dwelling's Worksheet 5-1 line 2c casualty
    pub(crate) fn rental_casualty_property(
        &self,
        parent: i64,
        cost_id: i64,
    ) -> Result<
        Option<(&'a crate::PersonalCasualty, &'a crate::CasualtyProperty)>,
        SourceAssociationIssue,
    > {
        self.income(parent)?;
        let Source::RentalCost { parent: actual, cost } = self.source(cost_id)? else {
            return Err(SourceAssociationIssue::WrongSourceKind(cost_id));
        };
        if actual.id != parent {
            return Err(SourceAssociationIssue::ParentMismatch { source: cost_id, parent });
        }
        let (Some(casualty), Some(id)) =
            (cost.casualty.as_ref().map(|records| &records.casualty), cost.casualty_property)
        else {
            return Ok(None);
        };
        // Saved identities are return-wide: the property ID names nothing else.
        let Source::Casualty { event, property: Some(property) } = self.source(id)? else {
            return Err(SourceAssociationIssue::WrongSourceKind(id));
        };
        if AssociationView::new(&self.sources).parent(id, Relation::CasualtyEvent)
            != SourceFact::Reported(event.id)
        {
            return Err(if event.id <= 0 {
                SourceAssociationIssue::IdentityUnavailable(event.id)
            } else {
                SourceAssociationIssue::AmbiguousSource(event.id)
            });
        }
        if event.date != casualty.date || event.declaration != casualty.declaration {
            return Err(SourceAssociationIssue::ParentMismatch {
                source: cost_id,
                parent: event.id,
            });
        }
        Ok(Some((event, property)))
    }
    pub(crate) fn intangible(
        &self,
        parent: i64,
        id: i64,
    ) -> Result<&'a crate::BusinessIntangible, SourceAssociationIssue> {
        self.income(parent)?;
        match self.source(id)? {
            Source::Intangible { parent: actual, intangible } if actual.id == parent => {
                Ok(intangible)
            }
            Source::Intangible { .. } => {
                Err(SourceAssociationIssue::ParentMismatch { source: id, parent })
            }
            _ => Err(SourceAssociationIssue::WrongSourceKind(id)),
        }
    }
    pub(crate) fn intangible_predecessor(
        &self,
        parent: i64,
        id: i64,
    ) -> Result<Option<&'a crate::BusinessIntangible>, SourceAssociationIssue> {
        self.intangible(parent, id)?;
        match AssociationView::new(&self.sources).parent(id, Relation::Property) {
            SourceFact::NotReported => Ok(None),
            SourceFact::Reported(previous) => self.intangible(parent, previous).map(Some),
            SourceFact::Unavailable => Err(SourceAssociationIssue::UnknownSource(id)),
        }
    }
    pub fn asset(&self, parent: i64, id: i64) -> Result<&'a BusinessAsset, SourceAssociationIssue> {
        self.income(parent)?;
        match self.source(id)? {
            Source::Asset { parent: actual, asset } if actual.id == parent => Ok(asset),
            Source::Asset { .. } => {
                Err(SourceAssociationIssue::ParentMismatch { source: id, parent })
            }
            _ => Err(SourceAssociationIssue::WrongSourceKind(id)),
        }
    }
}
