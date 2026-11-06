use super::*;
use crate::federal_model::NoteIdentity;

#[test]
fn no_statement_missing_history_warns_for_each_actual_suspension_stage() {
    for property in [false, true] {
        for stage in 0..3 {
            let mut source = complete_ordinary_scorp(0, true);
            source.qbi_statement = None;
            if property {
                let mut records = k1_section1231(0, 0);
                match stage {
                    0 => records.basis_loss = 200_000,
                    1 => records.risk_loss = 200_000,
                    _ => records.passive_loss = 200_000,
                }
                source.section1231 = Some(records);
            } else {
                match stage {
                    0 => source.prior_loss = 200_000,
                    1 => source.risk.prior_loss = 200_000,
                    _ => source.passive.loss = 200_000,
                }
            }
            source.amt = Some(amt_records(&source));
            assert!(source.valid());
            assert!(!source.loss_tracking() && !source.property_tracking());
            let (mut draft, id) = passive_property_pool(source, 0);
            draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
            let result = supported(&draft);
            let notes: Vec<_> = result
                .review_notes
                .iter()
                .filter(|note| note.display.key == "WARN_SCORP_QBI_HISTORY_LEFT_OUT")
                .collect();
            assert_eq!(notes.len(), 1, "property={property}, stage={stage}");
            assert_eq!(notes[0].context.as_ref().unwrap().identity(), NoteIdentity::Income(id));
            let mut session = Interview::new(TEST_USER, draft);
            let Reply::Response(P::Review { correction_notes, .. }) =
                send(&mut session, P::ReviewAccepted { user: TEST_USER })
            else {
                panic!("expected review")
            };
            assert!(correction_notes.iter().any(|note| note.section == TaxSection::Documents));
        }
    }
}

#[test]
fn no_statement_partial_history_omits_only_its_character_and_rejects_supplied_mismatch() {
    for missing_property in [false, true] {
        let mut source = complete_ordinary_scorp(0, true);
        source.qbi_statement = None;
        source.prior_loss = 200_000;
        source.loss_qbi.history = vec![loss_year(2020, 2_000, 1_000, 2_000, 1_000)];
        source.section1231 = Some(K1Section1231 { basis_loss: 200_000, ..k1_section1231(0, 0) });
        source.property_qbi.losses.history = source.loss_qbi.history.clone();
        if missing_property {
            source.section1231.as_mut().unwrap().risk_loss = 200_000;
        } else {
            source.risk.prior_loss = 200_000;
        }
        source.amt = Some(amt_records(&source));
        assert!(source.valid());
        assert_eq!(source.loss_tracking(), missing_property);
        assert_eq!(source.property_tracking(), !missing_property);
        let (mut draft, _) = passive_property_pool(source.clone(), 0);
        draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let mut rental = rental(12_000, 0);
        rental.qbi_business = RentalBusiness::Yes { material_participation: false };
        add(&mut draft, income(FederalIncome::Rental { data: rental }));
        let result = supported(&draft);
        assert_eq!(value(&result, "qbi_deduction"), 200);
        assert!(noted(&result, "WARN_SCORP_QBI_HISTORY_LEFT_OUT"));
        let records =
            if missing_property { &mut source.property_qbi.losses } else { &mut source.loss_qbi };
        records.history[0].remaining = 100_000;
        assert!(
            !source.valid(),
            "supplied mismatch must remain invalid despite another missing stage"
        );
    }
}

#[test]
fn no_statement_negative_current_loss_still_counts_released_qualified_history() {
    let mut source = complete_ordinary_scorp(-5_000, true);
    source.qbi_statement = None;
    source.passive.loss = 200_000;
    source.loss_qbi.passive = vec![loss_year(2020, 2_000, 1_000, 2_000, 1_000)];
    source.amt = Some(amt_records(&source));
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source.clone()),
        }),
    );
    let mut rental = rental(12_000, 0);
    rental.qbi_business = RentalBusiness::Yes { material_participation: false };
    add(&mut draft, income(FederalIncome::Rental { data: rental }));
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 200);
    assert!(noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
    assert!(
        result
            .review_notes
            .iter()
            .any(|note| note.contains("QBI loss passing passive limits: $1000."))
    );
    assert_eq!(source.loss_qbi.current, 0);
    assert!(source.loss_reported_with_basis());
    // Qualification comes from 2020, independently of the missing current statement.
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.loss_qbi.passive[0].qualified = 0;
        data.loss_qbi.passive[0].remaining_qualified = 0;
    } else {
        unreachable!()
    }
    assert_eq!(value(&supported(&draft), "qbi_deduction"), 400);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn no_statement_unrelated_histories_remain_inactive_and_retained() {
    let mut source = complete_ordinary_scorp(0, true);
    source.qbi_statement = None;
    source.loss_qbi.history = vec![loss_year(2020, 2_000, 1_000, 2_000, 1_000)];
    source.property_qbi.losses.history = source.loss_qbi.history.clone();
    source.property_qbi.complete = false;
    assert!(source.valid());
    assert!(!source.loss_tracking() && !source.property_tracking());
    let projection = source.qbi_projection();
    assert!(projection.loss_qbi.is_empty() && projection.property_qbi.is_empty());
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source.clone()),
        }),
    );
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    assert_eq!(restart(draft.clone()).draft, draft);
    assert_eq!(source.loss_qbi.history.len(), 1);
    assert_eq!(source.property_qbi.losses.history.len(), 1);
}

#[test]
fn no_statement_counts_released_property_history_without_current_confirmation() {
    let mut source = complete_ordinary_scorp(0, true);
    source.qbi_statement = None;
    source.opening = Some(200_000);
    source.loans.clear();
    source.section1231 = Some(K1Section1231 { basis_loss: 200_000, ..k1_section1231(0, 0) });
    source.property_qbi.losses.history = vec![loss_year(2020, 2_000, 1_000, 2_000, 1_000)];
    source.property_qbi.complete = false;
    source.amt = Some(amt_records(&source));
    let (mut draft, _) = passive_property_pool(source, 0);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut rental = rental(12_000, 0);
    rental.qbi_business = RentalBusiness::Yes { material_participation: false };
    add(&mut draft, income(FederalIncome::Rental { data: rental }));
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 200);
    let items = property_qbi_character_items(&draft).unwrap();
    assert_eq!((items[0].loss, items[0].current_loss), (1_000, 0));
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    assert_eq!(restart(draft.clone()).draft, draft);
}

fn destinations(context: &IssueContext) -> &[TaxSection] {
    match context {
        IssueContext::Corrective { sections, .. } => sections,
        IssueContext::Scoped { source, .. } => destinations(source),
        _ => &[],
    }
}

#[test]
fn missing_partner_gross_leaves_qbi_out_with_distinct_editable_source_notes() {
    let mut draft = complete(FederalStatus::Single);
    let mut ids = Vec::new();
    for _ in 0..2 {
        let mut partner = ordinary_partner_source(8_000, true);
        partner.limited_for_se = Some(false);
        partner.reported_se = Some(800_000);
        partner.qbi = Some(partner_qbi_statement(4_000));
        let mut entry = income(FederalIncome::Partnership { data: partner });
        entry.label = "Same partnership".into();
        add(&mut draft, entry);
        ids.push(draft.income.last().unwrap().id);
    }
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert!(!result.lines.iter().any(|line| line.id == "qbi_loss_carryforward"));
    let notes: Vec<_> = result
        .review_notes
        .iter()
        .filter(|note| note.display.key == "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT")
        .collect();
    assert_eq!(notes.len(), 2);
    assert_eq!(
        notes.iter().map(|note| note.context.as_ref().unwrap().identity()).collect::<Vec<_>>(),
        ids.iter().copied().map(NoteIdentity::Income).collect::<Vec<_>>()
    );

    let mut draft = known_fact_partnership_draft(draft);
    let mut session = Interview::new(TEST_USER, draft.clone());
    let Reply::Response(P::Review {
        correction_notes,
        remaining_items,
        cookie,
        revision,
        page,
        ..
    }) = send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!()
    };
    assert!(remaining_items.is_empty());
    let notes: Vec<_> = correction_notes
        .iter()
        .filter(|note| status_message(&note.status).key == "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT")
        .collect();
    assert_eq!(notes.len(), 2);
    assert!(notes.iter().all(|note| note.section == TaxSection::Documents));
    assert!(matches!(
        send(
            &mut session,
            P::OpenSection {
                user: TEST_USER,
                cookie,
                revision,
                from: page,
                section: TaxSection::Documents,
                reveal_path: None,
            }
        ),
        Reply::Edit(P::SaveDocuments { .. })
    ));

    for entry in &mut draft.income {
        if let FederalIncome::Partnership { data } = &mut entry.income {
            data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 1_000_000 });
            data.qbi_records.qualified_se_gross = 500_000;
        }
    }
    let resolved = supported(&draft);
    assert!(!noted(&resolved, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
    assert!(value(&resolved, "qbi_deduction") > 0);
}

#[test]
fn inconsistent_partner_gross_keeps_its_actual_source_identity() {
    let mut draft = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(8_000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(800_000);
    partner.qbi = Some(partner_qbi_statement(4_000));
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 1_000_000 });
    partner.qbi_records.qualified_se_gross = 1_000_001;
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    let crate::partnership_qbi::QbiGrossError::Inconsistent(context) =
        crate::partnership_qbi::half_se_by_gross(&draft, [500, 0]).unwrap_err()
    else {
        panic!()
    };
    assert_eq!(context.identity(), NoteIdentity::Income(draft.income.last().unwrap().id));
}

#[test]
fn annuity_recovery_errors_route_to_cost_records_and_raw_death_dates_stay_manual() {
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            0,
            RetirementTreatment::Ordinary,
        ),
    );
    draft.people.as_mut().unwrap().taxpayer.died = death(20260615);
    draft.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20200101,
        cost: 5_000_000,
        recovered: 1_000_000,
        refunds: 4_100_000,
        eligible: true,
    });
    let mut result = estimate(&complete(FederalStatus::Single));
    result.issues.clear();
    crate::federal_rules::final_annuity_losses(&draft, &mut result);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].context.identity(), NoteIdentity::Income(2));
    assert_eq!(destinations(&result.issues[0].context), &[TaxSection::OtherItemized]);

    draft.deductions.as_mut().unwrap().annuity_losses[0].refunds = 0;
    draft.people.as_mut().unwrap().taxpayer.died = None;
    result.issues.clear();
    crate::federal_rules::final_annuity_losses(&draft, &mut result);
    assert_eq!(result.issues.len(), 1);
    assert!(destinations(&result.issues[0].context).is_empty());
    assert_eq!(result.issues[0].context.identity(), NoteIdentity::Income(2));
}

#[test]
fn annuity_worksheet_reconciliation_offers_both_editable_cost_sources() {
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.died = death(20261215);
    let basis = pension_basis();
    add(&mut draft, pension_entry(basis.clone(), 10_000));
    draft.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: basis.started,
        cost: basis.cost.unwrap() + 10_000,
        recovered: 0,
        refunds: 0,
        eligible: true,
    });
    let mut result = estimate(&complete(FederalStatus::Single));
    result.issues.clear();
    crate::federal_rules::final_annuity_losses(&draft, &mut result);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(
        destinations(&result.issues[0].context),
        &[TaxSection::OtherItemized, TaxSection::Retirement]
    );
    assert_eq!(result.issues[0].context.identity(), NoteIdentity::Income(2));
}

#[test]
fn care_plan_reconciliation_retains_both_payments_even_with_the_same_name() {
    for (balance, key) in [
        (
            true,
            "ISSUE_REPEAT_ONE_PLAN_S_ANNUAL_VALUATION_CONSISTENTLY_ACROSS_ITS_PAYMENTS_FB3D8571",
        ),
        (
            false,
            "ISSUE_USE_ONE_CONSISTENT_ANNUAL_CERTIFIED_PREMIUM_TOTAL_FOR_EACH_COVERED_255AFF79",
        ),
    ] {
        let mut draft = care_return(care_claim(3000, 100000, false), 20000);
        draft.income[1].label = "Same plan".into();
        let mut second = draft.income[1].clone();
        second.id = 3;
        let FederalIncome::Retirement { data } = &mut second.income else { panic!() };
        {
            let mut data = edit_retirement(data);
            let Some(RetirementTreatment::Exception {
                facts: RetirementException::LongTermCare { data },
                ..
            }) = &mut data.ordinary_distribution
            else {
                panic!()
            };
            if balance {
                data.balance += 100;
            } else {
                data.premiums += 100;
            }
        }
        add(&mut draft, second);
        let result = estimate(&draft);
        let issues: Vec<_> =
            result.issues.iter().filter(|issue| issue.display.key == key).collect();
        assert_eq!(issues.len(), 2);
        assert_eq!(
            issues.iter().map(|issue| issue.context.identity()).collect::<Vec<_>>(),
            vec![NoteIdentity::Income(2), NoteIdentity::Income(3)]
        );
        let rows = crate::interview::remaining_items(&draft, &result, TaxSection::Review);
        assert_eq!(rows.iter().filter(|row| row.section == TaxSection::Documents).count(), 2);
    }
}
