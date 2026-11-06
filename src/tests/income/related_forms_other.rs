use super::*;
use crate::related_forms::{catalog, catalog_for, claimed_elsewhere, effective_claimants};
use crate::tests::cross::interview::open_topic;
use crate::tests::support::builders::related_forms::other_related_return;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

#[test]
fn other_activity_components_keep_separate_claims_and_resolve_the_physical_form() {
    let mut draft = other_related_return();
    let center = catalog(&draft);
    let rent = center
        .iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099MiscRents)
        .unwrap();
    let royalty = center
        .iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099MiscRoyalties)
        .unwrap();
    let part = center
        .iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099KPart)
        .unwrap();
    assert_eq!(rent.form_id, royalty.form_id);
    assert_ne!(rent.id, royalty.id);
    let hobby_id = draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::HobbySales { .. }))
        .unwrap()
        .id;
    let equipment_id = draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::NonbusinessProperty { .. }))
        .unwrap()
        .id;
    let royalty_id = draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::Royalty { .. }))
        .unwrap()
        .id;
    for entry in &mut draft.income {
        match &mut entry.income {
            FederalIncome::HobbySales { data } if entry.id == hobby_id => {
                data.related.push(part.id)
            }
            FederalIncome::Royalty { data } => data.related.push(royalty.id),
            FederalIncome::NonbusinessProperty {
                kind: NonbusinessPropertyKind::PersonalRental { related, .. },
                ..
            } => related.push(rent.id),
            _ => {}
        }
    }
    let claims = effective_claimants(&draft);
    assert_eq!(claims.get(&part.id), Some(&hobby_id));
    assert_eq!(claims.get(&rent.id), Some(&equipment_id));
    assert_eq!(claims.get(&royalty.id), Some(&royalty_id));
    assert_eq!(
        RelatedForms::validate::<EquipmentRent>(
            &[royalty.id],
            &catalog_for(&draft, equipment_id),
            &claimed_elsewhere(&draft, equipment_id)
        ),
        Err(0)
    );
    let mut session = restart(draft);
    for key in [rent.id, royalty.id] {
        assert!(matches!(
            session.handle(P::ViewRelatedForm { user: TEST_USER, key }),
            Some(Reply::Response(P::FormViewed { form: RelatedForm::Form1099Misc { .. }, .. }))
        ));
    }
    let native = NativeInterviewCase("TAX_OTHER_RELATED");
    for (name, topic) in
        [("other-income", TaxSection::OtherIncome), ("royalty", TaxSection::Rentals)]
    {
        let request = native.editor(&mut session, topic, "EDITOR");
        let P::SaveIncomes { documents, .. } = &request else { panic!("expected activity editor") };
        for row in &documents.documents {
            let related = match &row.document {
                IncomeRecord::HobbySales(page) if row.id == hobby_id => &page.related,
                IncomeRecord::EquipmentRental(page) => &page.related,
                IncomeRecord::Royalties(page) => &page.related,
                _ => continue,
            };
            assert_eq!(related.forms.len(), 1);
            assert!(related.forms[0].summary.is_some());
        }
        native.write_file("CAPTURE_DIR", &alloc::format!("{name}-editor.avro"), &request);
        native.save(&mut session, request);
        native.restart(&mut session);
        let reopened = native.editor(&mut session, topic, "REOPENED");
        native.write_file("CAPTURE_DIR", &alloc::format!("{name}-reopened.avro"), &reopened);
    }
    let result = supported(&session.draft);
    assert_eq!(value(&result, "hobby_sales_income"), 3_350);
    assert_eq!(value(&result, "nonbusiness_personal_rent"), 2_500);
    assert_eq!(value(&result, "nonbusiness_royalty_income"), 5_000);
    assert_eq!(effective_claimants(&restart(session.draft).draft), claims);
}

#[test]
fn other_activity_provisional_choices_use_the_selected_owner_without_storing_claims() {
    let mut session = restart(other_related_return());
    let original = session.draft.clone();
    let Some(Reply::Response(P::HobbyRelatedChoices { candidates })) =
        session.handle(P::HobbyRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            activity_input: true,
        })
    else {
        panic!("expected hobby choices")
    };
    assert_eq!(candidates.len(), 1);
    let Some(Reply::Response(P::EquipmentRelatedChoices { candidates })) =
        session.handle(P::EquipmentRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            activity_input: 0,
        })
    else {
        panic!("expected equipment choices")
    };
    assert_eq!(candidates.len(), 1);
    let Some(Reply::Response(P::RoyaltyRelatedChoices { candidates })) =
        session.handle(P::RoyaltyRelatedOptions {
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            activity_input: false,
        })
    else {
        panic!("expected royalty choices")
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(session.draft, original);
}

#[test]
fn hobby_claim_conflicts_ignore_display_candidates_and_locate_the_key() {
    let mut session = restart(other_related_return());
    let key = catalog(&session.draft)
        .into_iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099KPart)
        .unwrap()
        .id;
    let mut request = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { documents, .. } = &mut request else { panic!("expected activity editor") };
    let mut first = None;
    for (index, row) in documents.documents.iter_mut().enumerate() {
        if let IncomeRecord::HobbySales(page) = &mut row.document {
            first.get_or_insert(index);
            page.related.forms.push(RelatedFormView { key, summary: None });
            page.related.candidates.clear();
        }
    }
    let before = session.draft.clone();
    let reply = send(&mut session, request);
    let Reply::Response(P::Problem { field, .. }) = reply else {
        panic!("expected conflicting claim refusal")
    };
    assert_eq!(
        field,
        Some(alloc::format!(
            "[documents/documents/{}/document/HobbySales/related/forms/0/key]",
            first.unwrap()
        ))
    );
    assert_eq!(session.draft, before);
}
