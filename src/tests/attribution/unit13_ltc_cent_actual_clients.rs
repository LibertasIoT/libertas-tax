use super::*;
use crate::tests::cross::interview::{accepted, open_documents};
use crate::tests::support::assert::injured_income::ltc_cent_actual_clients::{checked, financial};
use crate::tests::support::builders::injured_income::ltc_cent_actual_clients::{
    Family, fixture, opposite, source_rows,
};
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

#[test]
fn unit13_ltc_cent_actual_source_corrections_preserve_native_money() {
    for family in [Family::Low, Family::High, Family::Interior, Family::Exclusive, Family::Zero] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let mut s = restart(fixture(family, injured, reverse));
                let before = financial(&s.draft, family);
                let (_, ids) = source_rows(&s.draft);
                let mut page = open_documents(&mut s);
                let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
                for row in &mut documents.other.other_income.documents {
                    if ids.iter().any(|source| source.0 == row.id) {
                        row.owner = crate::DocumentOwner::from(opposite(row.owner.owned()));
                    }
                }
                let page = fixture_owner_inputs(&s.draft, page);
                accepted(&send(&mut s, page));
                assert_eq!(financial(&s.draft, family).return_lines, before.return_lines);
                let (_, after) = source_rows(&s.draft);
                assert_eq!(
                    after,
                    ids.iter().map(|(id, owner)| (*id, opposite(*owner))).collect::<Vec<_>>()
                );
                let tax_year = s.draft.tax_year;
                let stored = native_draft_records(&s.draft);
                let draft = draft_from_native_records(tax_year, &stored);
                assert_eq!(financial(&restart(draft).draft, family), financial(&s.draft, family));
                if !family.shared() {
                    checked(&s.draft, family, injured, true);
                }
            }
        }
    }
}
