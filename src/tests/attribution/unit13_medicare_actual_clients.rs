use super::*;
use crate::tests::cross::interview::{accepted, open_documents, open_topic};
use crate::tests::support::assert::injured_income::medicare_actual_clients::checked;
use crate::tests::support::builders::additional_medicare::{Source, fixture};
use crate::tests::support::builders::injured_income::medicare_actual_clients::is_primary;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};
// Native Form8959 contributor controls: the source operands must follow the
// actual settled Parts I/II/SE, not raw ScheduleC profit or only W2 wages.

#[test]
fn medicare_actual_intermediate_source_save_has_independent_native_money() {
    for source in [Source::PartnerSe, Source::SmallBusiness] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let d = fixture(source, Owner::Taxpayer, injured, reverse);
                checked(&d, source, Owner::Taxpayer, Owner::Spouse, injured);
                let mut s = restart(d);
                crate::tests::support::client::admit_native_fixture_facts(&mut s);
                let id = s.draft.income.iter().find(|e| is_primary(source, e)).unwrap().id;
                let mut p = match source {
                    Source::PartnerSe => open_documents(&mut s),
                    Source::SmallBusiness => open_topic(&mut s, source.topic()),
                    Source::Railroad => unreachable!(),
                };
                match &mut p {
                    P::SaveDocuments { documents, .. } => {
                        documents
                            .common
                            .k1
                            .documents
                            .iter_mut()
                            .find(|row| row.id == id)
                            .unwrap()
                            .owner = crate::DocumentOwner::Spouses
                    }
                    P::SaveIncomes { documents, .. } => {
                        documents.documents.iter_mut().find(|row| row.id == id).unwrap().owner =
                            Owner::Spouse
                    }
                    _ => unreachable!(),
                }
                let p = fixture_owner_inputs(&s.draft, p);
                accepted(&send(&mut s, p));
                checked(&s.draft, source, Owner::Spouse, Owner::Spouse, injured);
                let tax_year = s.draft.tax_year;
                let bytes = native_draft_records(&s.draft);
                let draft = draft_from_native_records(tax_year, &bytes);
                assert_eq!(native_draft_records(&draft), bytes);
                let mut s = restart(draft);
                checked(&s.draft, source, Owner::Spouse, Owner::Spouse, injured);
                let mut p = open_documents(&mut s);
                let P::SaveDocuments { documents, .. } = &mut p else { panic!() };
                for old in &s.draft.income {
                    if let FederalIncome::Wage { data } = &old.income {
                        documents
                            .common
                            .wages
                            .documents
                            .iter_mut()
                            .find(|r| r.id == old.id)
                            .unwrap()
                            .owner = if FederalWage::from(data).wages > 0 {
                            crate::DocumentOwner::Yours
                        } else {
                            crate::DocumentOwner::Spouses
                        };
                    }
                }
                let p = fixture_owner_inputs(&s.draft, p);
                accepted(&send(&mut s, p));
                checked(&s.draft, source, Owner::Spouse, Owner::Taxpayer, injured);
            }
        }
    }
}
