use crate::Owner;
use crate::kernel::associations::{AssociationSource, AssociationView, Relation, SourceKind};
use crate::kernel::source_fact::SourceFact::{self, NotReported, Reported, Unavailable};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Source {
    id: i64,
    kind: SourceKind,
    owner: SourceFact<Owner>,
    link: SourceFact<i64>,
    label: &'static str,
}
impl AssociationSource for Source {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> SourceKind {
        self.kind
    }
    fn owner(&self) -> SourceFact<Owner> {
        self.owner.clone()
    }
    fn parent(&self, _: Relation) -> SourceFact<i64> {
        self.link.clone()
    }
}
fn source(id: i64, kind: SourceKind, link: SourceFact<i64>) -> Source {
    Source { id, kind, owner: Reported(Owner::Taxpayer), link, label: "Same label" }
}

#[test]
fn association_identity_borrows_records_and_checks_each_actual_relation() {
    for (relation, child, parent) in [
        (Relation::PhysicalBuilding, SourceKind::Component, SourceKind::Building),
        (Relation::Property, SourceKind::Sale, SourceKind::Property),
        (Relation::K1Parent, SourceKind::K1Child, SourceKind::K1),
        (Relation::CreditOrigin, SourceKind::Credit, SourceKind::Facility),
        (Relation::CreditOrigin, SourceKind::Credit, SourceKind::Equipment),
    ] {
        let mut records = [source(31, child, Reported(47)), source(47, parent, NotReported)];
        records[0].owner = Reported(Owner::Spouse);
        let view = AssociationView::new(&records);
        let Reported(actual) = view.source(31) else { panic!("actual source missing") };
        assert!(core::ptr::eq(actual, &records[0]));
        assert_eq!(view.parent(31, relation), Reported(47));
        assert_eq!(view.owner(31), Reported(Owner::Spouse));
        records.swap(0, 1);
        records[1].label = "Corrected label";
        assert_eq!(AssociationView::new(&records).parent(31, relation), Reported(47));
    }
}

#[test]
fn association_known_absence_is_distinct_from_unknown_and_names_only_needed_boundary() {
    let mut records = [source(31, SourceKind::Credit, NotReported)];
    let view = AssociationView::new(&records);
    assert_eq!(view.parent(31, Relation::CreditOrigin).require(|| "origin missing"), Ok(None));
    assert_eq!(view.source(99), Unavailable);
    records[0].link = Unavailable;
    records[0].owner = Unavailable;
    let view = AssociationView::new(&records);
    let mut notices = 0;
    assert_eq!(
        view.parent(31, Relation::CreditOrigin).require(|| {
            notices += 1;
            (31, Relation::CreditOrigin)
        }),
        Err((31, Relation::CreditOrigin))
    );
    assert_eq!(notices, 1);
    assert_eq!(view.owner(31), Unavailable);
}

#[test]
fn association_invalid_or_ambiguous_ids_and_targets_never_supply_an_owner_or_parent() {
    let mut records =
        [source(31, SourceKind::Sale, Reported(47)), source(47, SourceKind::Property, NotReported)];
    for id in [0, -1, 99] {
        assert_eq!(AssociationView::new(&records).source(id), Unavailable);
    }
    for link in [Reported(0), Reported(-1), Reported(31), Reported(99)] {
        records[0].link = link;
        assert_eq!(AssociationView::new(&records).parent(31, Relation::Property), Unavailable);
    }
    records[0].link = Reported(47);
    records[1].kind = SourceKind::Income;
    assert_eq!(AssociationView::new(&records).parent(31, Relation::Property), Unavailable);
    records[1].id = 31;
    let view = AssociationView::new(&records);
    assert_eq!(view.source(31), Unavailable);
    assert_eq!(view.owner(31), Unavailable);
    assert_eq!(view.parent(31, Relation::Property), Unavailable);
}
