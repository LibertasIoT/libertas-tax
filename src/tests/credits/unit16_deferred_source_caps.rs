use super::*;

pub(super) fn draft(filer: CarryFiler) -> FederalDraft {
    let mut d = r4_portfolio_draft();
    history(&mut d).carryforwards[0].filer = filer;
    for (owner, tax) in [(Owner::Taxpayer, 10_000), (Owner::Spouse, 30_000)] {
        let mut data = ordinary_partner_source(0, false);
        data.deferred_compensation_tax = tax;
        let mut entry = income(FederalIncome::Partnership { data });
        entry.owner = crate::DocumentOwner::from(owner);
        entry.label = "Synthetic same-name partnership".into();
        add(&mut d, entry);
    }
    d
}

#[test]
fn unit16_r4_native_deferred_tax_owners_drive_both_carry_caps_without_a_claim() {
    // Existing independent MFS Table figures:5023 and393. Actual box20AI
    // adds100/300 to those owners, not to income or the joint GBC limit.
    // 5123/5816 ->8808/10000, residual1192; joint Table tax5243 gives
    // rounded capacities4618/625. SameFilers uses the unprorated5243.
    for (filer, used) in
        [(CarryFiler::You, 4618), (CarryFiler::Spouse, 625), (CarryFiler::SameFilers, 5243)]
    {
        let mut d = draft(filer);
        assert!(d.injured_spouse.is_none());
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 80_000);
        assert_eq!(value(&r, "taxable_income"), 47_800);
        assert_eq!(value(&r, "form_3800_line_16"), 5243);
        assert_eq!(value(&r, "k1_deferred_compensation_tax"), 400);
        assert_eq!(return_line(&r, "23") / 100, 400);
        assert_eq!(value(&r, "general_business_credit"), used);
        assert_eq!(r.tax, Some((5243 - used + 400) * 100));
        let prefix = if filer == CarryFiler::SameFilers {
            alloc::string::String::from("business_credit_work_opportunity_2025")
        } else {
            alloc::format!(
                "business_credit_work_opportunity_source_none_filer_{}_2025",
                filer as i32
            )
        };
        assert_eq!(value(&r, &alloc::format!("{prefix}_to_next_year")), 10_000 - used);
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        let reordered = supported(&d);
        assert_eq!(reordered.tax, r.tax);
        assert_eq!(value(&reordered, "general_business_credit"), used);
        assert_eq!(value(&reordered, &alloc::format!("{prefix}_to_next_year")), 10_000 - used);
    }
}

#[test]
fn unit16_r4_actual_k1_owner_correction_changes_cap_and_preserves_joint_tax_source() {
    use crate::tests::cross::interview::{accepted, open_document_center};
    let d = draft(CarryFiler::You);
    let before = supported(&d);
    assert_eq!(value(&before, "general_business_credit"), 4618);
    let mut session = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { panic!() };
    let rows = &mut documents.common.k1.documents;
    let original_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    for row in rows {
        row.owner = if row.owner == crate::DocumentOwner::Yours {
            crate::DocumentOwner::Spouses
        } else {
            crate::DocumentOwner::Yours
        };
    }
    let request = fixture_owner_inputs(&session.draft, request);
    accepted(&send(&mut session, request));
    assert!(session.draft.injured_spouse.is_none());
    let after = supported(&session.draft);
    // Same accepted tax400, now actual owners300/100:5323/5816 ->9152,
    // with cap4798 (other owner's residual capacity445).
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "taxable_income"), value(&before, "taxable_income"));
    assert_eq!(value(&after, "form_3800_line_16"), 5243);
    assert_eq!(value(&after, "k1_deferred_compensation_tax"), 400);
    assert_eq!(value(&after, "general_business_credit"), 4798);
    let reopened = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &reopened else { panic!() };
    assert_eq!(
        documents.common.k1.documents.iter().map(|row| row.id).collect::<Vec<_>>(),
        original_ids
    );
    let saved = session.draft.clone();
    let reopened = fixture_owner_inputs(&session.draft, reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&restart(saved.clone()).draft), after);
    let mut spouse = saved.clone();
    history(&mut spouse).carryforwards[0].filer = CarryFiler::Spouse;
    assert_eq!(value(&supported(&spouse), "general_business_credit"), 445);
    let mut current = saved;
    own(&mut current, BusinessCredit::WorkOpportunity, 1000);
    let r = supported(&current);
    assert_eq!(value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_used"), 4798);
    assert_eq!(value(&r, "general_business_credit"), 5243);
    assert_eq!(return_line(&r, "23") / 100, 400);
}
