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
        SourceFact::Reported(self.owner)
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
    Income(&'a FederalIncomeEntry),
    RentalBuilding { parent: &'a FederalIncomeEntry, facts: &'a crate::RentalEnergyBuildingFacts },
    RentalCost { parent: &'a FederalIncomeEntry, cost: &'a crate::RentalBuilding },
    Mileage { parent: &'a FederalIncomeEntry, mileage: &'a crate::BusinessMileage },
    Asset { parent: &'a FederalIncomeEntry, asset: &'a BusinessAsset },
    Other { parent: &'a FederalIncomeEntry, id: i64, kind: SourceKind },
}
impl Source<'_> {
    fn income(&self) -> &FederalIncomeEntry {
        match self {
            Self::Income(parent)
            | Self::RentalBuilding { parent, .. }
            | Self::RentalCost { parent, .. }
            | Self::Asset { parent, .. }
            | Self::Mileage { parent, .. }
            | Self::Other { parent, .. } => parent,
        }
    }
}
impl AssociationSource for Source<'_> {
    fn id(&self) -> i64 {
        match self {
            Self::Income(entry) => entry.id(),
            Self::RentalBuilding { facts, .. } => facts.id,
            Self::RentalCost { cost, .. } => cost.id,
            Self::Asset { asset, .. } => asset.id,
            Self::Mileage { mileage, .. } => mileage.id,
            Self::Other { id, .. } => *id,
        }
    }
    fn kind(&self) -> SourceKind {
        match self {
            Self::Income(entry) => entry.kind(),
            Self::RentalBuilding { .. } => SourceKind::Building,
            Self::RentalCost { .. } => SourceKind::Component,
            Self::Asset { .. } | Self::Mileage { .. } => SourceKind::Equipment,
            Self::Other { kind, .. } => *kind,
        }
    }
    fn owner(&self) -> SourceFact<Owner> {
        self.income().owner()
    }
    fn parent(&self, relation: Relation) -> SourceFact<i64> {
        match self {
            Self::RentalCost { cost, .. } if relation == Relation::PhysicalBuilding => {
                cost.energy.as_ref().map_or(SourceFact::NotReported, |claim| {
                    SourceFact::Reported(claim.physical_building)
                })
            }
            Self::Other { parent, kind: SourceKind::K1Child, .. }
                if relation == Relation::K1Parent =>
            {
                SourceFact::Reported(parent.id)
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
        for parent in &draft.income {
            sources.push(Source::Income(parent));
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
                FederalIncome::Scorp { data } => data.section179_sales.as_slice(),
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
        Self { sources }
    }
    fn source(&self, id: i64) -> Result<Source<'a>, SourceAssociationIssue> {
        let source = unique_source(&self.sources, id)?;
        let parent = source.income().id;
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
