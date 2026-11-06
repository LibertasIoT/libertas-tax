#![allow(dead_code)]

use super::source_fact::SourceFact;
use crate::Owner;

pub(crate) type SourceId = i64;

// Borrowed facts from an accepted original return. The owning adapter supplies
// the authoritative original-return set and durable filer identity.
// Names, amounts and filing status do not establish that identity. Incomplete
// original-return history stays unavailable rather than becoming an empty set.
pub(crate) struct AcceptedOriginalReturn<'a, F: ?Sized> {
    pub(crate) id: SourceId,
    pub(crate) tax_year: SourceFact<i32>,
    pub(crate) filer: SourceFact<&'a F>,
}

pub(crate) fn operative_filed_return<F: Eq + ?Sized>(
    originals: SourceFact<&[AcceptedOriginalReturn<'_, F>]>,
    tax_year: SourceFact<i32>,
    filer: SourceFact<&F>,
) -> SourceFact<SourceId> {
    let (SourceFact::Reported(tax_year), SourceFact::Reported(filer)) = (tax_year, filer) else {
        return SourceFact::Unavailable;
    };
    match originals {
        SourceFact::NotReported | SourceFact::Reported([]) => SourceFact::NotReported,
        SourceFact::Reported([original]) if original.id > 0 => {
            match (&original.tax_year, &original.filer) {
                (SourceFact::Reported(year), SourceFact::Reported(identity))
                    if *year == tax_year && *identity == filer =>
                {
                    SourceFact::Reported(original.id)
                }
                _ => SourceFact::Unavailable,
            }
        }
        _ => SourceFact::Unavailable,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceKind {
    Income,
    Property,
    Building,
    Component,
    Sale,
    K1,
    K1Child,
    Credit,
    Facility,
    Equipment,
    CasualtyEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Relation {
    PhysicalBuilding,
    Property,
    K1Parent,
    CreditOrigin,
    CasualtyEvent,
}

impl Relation {
    fn accepts(self, target: SourceKind) -> bool {
        match self {
            Self::PhysicalBuilding => target == SourceKind::Building,
            Self::Property => matches!(
                target,
                SourceKind::Property | SourceKind::Building | SourceKind::Equipment
            ),
            Self::K1Parent => target == SourceKind::K1,
            Self::CreditOrigin => matches!(target, SourceKind::Facility | SourceKind::Equipment),
            Self::CasualtyEvent => target == SourceKind::CasualtyEvent,
        }
    }
}

// Implementations read the owning accepted records, including their actual
// identity references. No label, index, address, amount or share supplies a link.
pub(crate) trait AssociationSource {
    fn id(&self) -> SourceId;
    fn kind(&self) -> SourceKind;
    fn owner(&self) -> SourceFact<Owner>;
    fn parent(&self, relation: Relation) -> SourceFact<SourceId>;
}

// This borrows the owner's records, not a persisted registry or a second ID
// store. Reordering or correcting those records does not change their identity.
pub(crate) struct AssociationView<'a, T> {
    sources: &'a [T],
}

impl<'a, T: AssociationSource> AssociationView<'a, T> {
    pub(crate) fn new(sources: &'a [T]) -> Self {
        Self { sources }
    }

    pub(crate) fn source(&self, id: SourceId) -> SourceFact<&'a T> {
        if id <= 0 {
            return SourceFact::Unavailable;
        }
        let mut matches = self.sources.iter().filter(|source| source.id() == id);
        match (matches.next(), matches.next()) {
            (Some(source), None) => SourceFact::Reported(source),
            _ => SourceFact::Unavailable,
        }
    }

    pub(crate) fn owner(&self, id: SourceId) -> SourceFact<Owner> {
        match self.source(id) {
            SourceFact::Reported(source) => source.owner(),
            _ => SourceFact::Unavailable,
        }
    }

    pub(crate) fn parent(&self, id: SourceId, relation: Relation) -> SourceFact<SourceId> {
        let SourceFact::Reported(source) = self.source(id) else {
            return SourceFact::Unavailable;
        };
        match source.parent(relation) {
            SourceFact::Reported(parent) if parent != id => match self.source(parent) {
                SourceFact::Reported(target) if relation.accepts(target.kind()) => {
                    SourceFact::Reported(parent)
                }
                _ => SourceFact::Unavailable,
            },
            SourceFact::NotReported => SourceFact::NotReported,
            _ => SourceFact::Unavailable,
        }
    }
}

#[cfg(test)]
#[path = "associations_tests.rs"]
mod tests;
