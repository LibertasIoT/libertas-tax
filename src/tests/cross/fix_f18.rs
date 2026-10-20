// F18 closes merge-review D1–D7. Derived fixtures retain the review's exact
// source facts and independent figures; i1116 lines 3d/3e define gross gains
// before losses: https://www.irs.gov/instructions/i1116
use super::*;
use crate::tests::cross::interview::{accepted, document, from_review};

fn property_return(proceeds: i64, partial_cost: i64) -> FederalDraft {
    let mut d = crate::tests::cross::fix_f17::k3_base(0);
    let mut building = crate::tests::business::business::shop_building(AssetDisposal::No);
    building.partial =
        Some(PartialDisposition { date: 20260610, cost_of_part: partial_cost, proceeds: 0 });
    let mut business = cash_business(150_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets = vec![
        crate::tests::business::business::press(crate::tests::business::business::sold(
            20260715, proceeds, 0,
        )),
        building,
    ];
    add(&mut d, income(business));
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    d
}
#[test]
fn d1_property_losses_do_not_reduce_gross_before_losses_in_either_system() {
    // Independent review oracles: $8,160/$16,160 press gains. The building
    // losses $28,582/$2,858 remain deductible on Form4797, not line3e.
    for (proceeds, cost, expected) in
        [(1_200_000, 3_900_000, 320_160), (2_000_000, 390_000, 328_160)]
    {
        let d = property_return(proceeds, cost);
        let r = supported(&d);
        assert_eq!(crate::tests::cross::fix_f17::line_3e(&r), expected);
        assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), expected);
        assert_eq!(
            value(&r, "business_3_asset_1_partial_gain"),
            if cost == 3_900_000 { -28_582 } else { -2858 }
        );
        if cost == 3_900_000 {
            assert_eq!(value(&r, "agi"), 277_152);
            assert_eq!(value(&r, "form4797_line8"), -26_582);
        }
        let _ = FederalEstimate::from(r.clone());
        assert_eq!(supported(&restart(d).draft), r);
    }
}
#[test]
fn d2_farm_conservation_amt_rows_render_in_the_actual_public_save() {
    crate::tests::farm::farm_specials::farm_special_conservation_plan_work_and_election_are_public_and_item_specific();
}
#[test]
fn d3_both_disaster_owners_render_without_changing_their_adjustments() {
    crate::tests::retirement::disaster::injured_spouse_disaster_projection_keeps_two_owner_adjustments();
}
fn leave_joint(s: &mut Interview) {
    let Reply::Edit(mut page) = from_review(s, TaxSection::Setup) else { panic!("Setup") };
    let P::SaveSetup { value, .. } = &mut page else { panic!("Setup request") };
    value.filing = Filing::Single;
    accepted(&send(s, page));
    assert!(s.draft.valid_stored());
}
#[test]
fn d4_minimum_tax_histories_follow_their_actual_return_unit() {
    // Exact D4 reproducer: only the spouse's identified prior return.
    let mut original = crate::tests::federal::minimum_tax::session(FederalStatus::Joint);
    let page = crate::tests::federal::minimum_tax::request(
        &mut original,
        MinimumTaxHistories {
            returns: vec![OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner: Owner::Spouse },
                facts: crate::tests::federal::minimum_tax::source(),
            }],
        },
    );
    accepted(&send(&mut original, page));
    crate::tests::core_tests::storage::f18_cold(&original.draft);
    leave_joint(&mut original);
    assert!(original.draft.credits.as_ref().unwrap().minimum_tax.is_none());
    crate::tests::core_tests::storage::f18_cold(&original.draft);
    let mut s = crate::tests::federal::minimum_tax::session(FederalStatus::Joint);
    let histories = MinimumTaxHistories {
        returns: [Owner::Taxpayer, Owner::Spouse]
            .into_iter()
            .map(|owner| OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner },
                facts: crate::tests::federal::minimum_tax::source(),
            })
            .collect(),
    };
    let page = crate::tests::federal::minimum_tax::request(&mut s, histories);
    accepted(&send(&mut s, page));
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    leave_joint(&mut s);
    let histories = s.draft.credits.as_ref().unwrap().minimum_tax.as_ref().unwrap();
    assert_eq!(histories.returns.len(), 1);
    assert_eq!(
        histories.returns[0].unit,
        MinimumTaxHistoryUnit::Individual { owner: Owner::Taxpayer }
    );
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    // A former joint return remains its named unsupported attribution case;
    // a same-couple joint return never becomes an individual history.
    for unit in [
        MinimumTaxHistoryUnit::SameJointCouple,
        MinimumTaxHistoryUnit::FormerJoint { claimant: Owner::Taxpayer },
        MinimumTaxHistoryUnit::FormerJoint { claimant: Owner::Spouse },
    ] {
        let mut s = crate::tests::federal::minimum_tax::session(FederalStatus::Joint);
        let mut facts = crate::tests::federal::minimum_tax::source();
        facts.status = FederalStatus::Joint;
        let page = crate::tests::federal::minimum_tax::request(
            &mut s,
            MinimumTaxHistories {
                returns: vec![OwnedMinimumTaxHistory { unit: unit.clone(), facts }],
            },
        );
        accepted(&send(&mut s, page));
        leave_joint(&mut s);
        let kept = s.draft.credits.as_ref().unwrap().minimum_tax.as_ref();
        if matches!(unit, MinimumTaxHistoryUnit::FormerJoint { claimant: Owner::Taxpayer }) {
            assert_eq!(kept.unwrap().returns[0].unit, unit);
        } else {
            assert!(kept.is_none());
        }
        crate::tests::core_tests::storage::f18_cold(&s.draft);
    }
}
#[test]
fn d5_both_owners_returned_ira_contributions_save_render_and_prune() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let Reply::Edit(mut page) = from_review(&mut s, TaxSection::IraContributions) else {
        panic!("IRA")
    };
    let corrections: Vec<_> = [IraKind::Traditional, IraKind::Roth]
        .into_iter()
        .flat_map(|kind| {
            [LateCorrection::FilingExtension, LateCorrection::TimelyFiledReturn].into_iter().map(
                move |late| ReturnedContribution {
                    kind,
                    principal: 100_000,
                    earnings: 5_000,
                    returned_on: 20270601,
                    late: Some(late),
                },
            )
        })
        .collect();
    let P::SaveIraContributions { value, .. } = &mut page else { panic!() };
    value.people = [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| IraContributor {
            owner,
            traditional: 0,
            roth: 0,
            covered: false,
            form_8606_value: None,
            more: IraMore { returned_in_following_year: corrections.clone(), ..Default::default() },
        })
        .collect();
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 40_000);
    let _ = FederalEstimate::from(r);
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    leave_joint(&mut s);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().iras.taxpayer.returned_next_year, corrections);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().iras.spouse, IraSpouse::NotApplicable);
    assert_eq!(return_line(&supported(&s.draft), "4b"), 20_000);
    crate::tests::core_tests::storage::f18_cold(&s.draft);
}
#[test]
fn d7_inherited_spouse_deposit_and_529_labels_render_exact_source_amounts() {
    let mut d = complete(FederalStatus::Joint);
    let mut iras = ira_facts(0, 0, Answer::No);
    iras.spouse = IraSpouse::Joint { facts: iras.taxpayer.clone() };
    d.adjustments.as_mut().unwrap().iras = iras;
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Edit(mut page) = from_review(&mut s, TaxSection::IraContributions) else {
        panic!("IRA")
    };
    let P::SaveIraContributions { value, .. } = &mut page else { panic!() };
    let mut spouse = crate::tests::cross::interview::ira_row(Owner::Spouse, 200_000, None);
    spouse.roth = 100_000;
    spouse.more.paid_next_year = Some(next_year_ira_deposits(200_000, 100_000, 20270331));
    value.people.push(spouse);
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    for (id, amount) in [
        ("spouse_2026_form_5498_box1_2027_deposit", 2000),
        ("spouse_2026_form_5498_box10_2027_deposit", 1000),
    ] {
        assert_eq!(value_of(&r, id), amount);
    }
    let _ = FederalEstimate::from(r);
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    // Original review used a raw valid stored transfer, not a public1099-Q
    // Save. Keep that distinction and the same6000/4000/6000 oracle.
    let mut d = complete(FederalStatus::Joint);
    let mut iras = ira_facts(0, 0, Answer::No);
    iras.spouse = IraSpouse::Joint { facts: iras.taxpayer.clone() };
    d.adjustments.as_mut().unwrap().iras = iras;
    set_qtp_roth(&mut d, Owner::Spouse, &[qtp_roth_rollover()]);
    let r = supported(&d);
    for (id, amount) in [
        ("spouse_529_roth_form_5498_box10", 6000),
        ("spouse_529_roth_source_contribution_basis", 4000),
        ("spouse_529_roth_transfer_1_amount", 6000),
    ] {
        assert_eq!(value_of(&r, id), amount);
    }
    let _ = FederalEstimate::from(r);
    crate::tests::core_tests::storage::f18_cold(&d);
}
fn value_of(r: &FederalResult, id: &str) -> i64 {
    super::value(r, id)
}

#[test]
fn d6_catalog_selection_rejects_forged_sources_and_follows_real_source_deletion() {
    use crate::farm_specials::{FarmExternalSourceInput, FarmSourceCharacter};
    let mut s = Interview::new(TEST_USER, crate::tests::cross::interview::farm_only_return());
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents
        .documents
        .push(document(0, IncomeDocument::W2(crate::tests::cross::interview::w2(2_000_000))));
    accepted(&send(&mut s, page));
    let source =
        s.draft.income.iter().find(|e| matches!(e.income, FederalIncome::Wage { .. })).unwrap().id;
    let input = |source, character| FarmExternalSourceInput {
        source,
        character,
        gross_sales: 100_000,
        product_cost: 0,
        agreement: None,
        shareholder_employee: false,
        regularly_used: false,
        ceased: None,
    };
    // Every original absent99 character remains a negative selection test;
    // character does not manufacture an accepted document identity.
    for (case, character) in [
        ("income-source-640", FarmSourceCharacter::FarmK1),
        ("income-source-645", FarmSourceCharacter::FishingK1),
        ("income-source-647", FarmSourceCharacter::ScorpFarm),
        ("income-source-651", FarmSourceCharacter::ScorpWages),
        ("income-source-657", FarmSourceCharacter::FishingCrewShare),
        ("income-source-660", FarmSourceCharacter::FishingCrewWages),
        ("income-source-664", FarmSourceCharacter::VesselShare),
        ("income-source-666", FarmSourceCharacter::FarmProperty),
        ("income-source-667", FarmSourceCharacter::Other),
    ] {
        let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Farming);
        let P::SaveIncomes { documents, farm_sources, .. } = &mut page else { panic!() };
        assert!(farm_sources.iter().any(|choice| choice.id == source));
        assert!(farm_sources.iter().all(|choice| choice.id != 99));
        // A forged echoed picker catalog cannot authorize its invented ID.
        let mut forged_choice = farm_sources[0].clone();
        forged_choice.id = 99;
        farm_sources.push(forged_choice);
        let mut farm = crate::tests::cross::interview::cash_farm(20_000_000, 0);
        let mut missing = input(99, character);
        // Preserve the original nine-branch review's exact statement facts.
        missing.product_cost = 20_000;
        missing.agreement = Some(crate::farm_specials::FarmShareAgreement {
            signed: 20250101,
            significant_activity: 20250201,
            production_based: true,
        });
        missing.shareholder_employee = true;
        missing.regularly_used = true;
        farm.more.specials.external_sources.push(missing);
        documents.documents.push(document(0, IncomeDocument::Farm(farm)));
        let before = s.draft.clone();
        crate::tests::cross::interview::rejected_with(
            current_income_refusal(
                case,
                &mut s,
                page,
                "FARM_EXTERNAL_SOURCE_UNAVAILABLE",
                "[documents/documents/0/document/more/specials/external_sources/0/source]",
            ),
            "FARM_EXTERNAL_SOURCE_UNAVAILABLE",
        );
        assert_eq!(s.draft, before);
    }
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    let mut farm = crate::tests::cross::interview::cash_farm(20_000_000, 0);
    farm.more.specials.external_sources.push(input(source, FarmSourceCharacter::FishingCrewWages));
    documents.documents.push(document(0, IncomeDocument::Farm(farm)));
    accepted(&send(&mut s, page));
    assert_eq!(
        crate::tests::cross::interview::stored_farm(&s.draft).1.specials.external_sources[0].source,
        source
    );
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    let mut forged = s.draft.clone();
    if let FederalIncome::Business { data } = &mut forged.income.last_mut().unwrap().income
        && let BusinessActivity::Farming { data } = &mut data.activity
    {
        data.specials.external_sources[0].source = 99;
    }
    assert!(!forged.valid_stored());
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.retain(|item| item.id != source);
    accepted(&send(&mut s, page));
    assert!(
        crate::tests::cross::interview::stored_farm(&s.draft)
            .1
            .specials
            .external_sources
            .is_empty()
    );
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    let mut walked = s.draft.clone();
    let _ = crate::interview::prune(&mut walked);
    assert_eq!(walked, s.draft);
    let page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Farming);
    let P::SaveIncomes { documents, farm_sources, .. } = page else { panic!() };
    assert!(farm_sources.iter().all(|choice| choice.id != source));
    let IncomeDocument::Farm(farm) = &documents.documents[0].document else { panic!() };
    assert!(farm.more.specials.external_sources.is_empty());
}

#[test]
fn d1_amt_uses_its_own_recapture_gross_without_reusing_regular_ordinary() {
    let mut d = property_return(1_200_000, 3_900_000);
    let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let asset = &mut data.assets[0];
    asset.acquired = 20170101; // Before this source owner's2017-09-28 bonus-eligibility cutoff.
    let EquipmentHistory::Continuing { amt, .. } = &mut asset.history else { panic!() };
    *amt = EquipmentAmtHistory::Separate {
        prior_depreciation: 405_000,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    //150% five-year half-year table:15%+25.5% prior=$4050;
    // disposal-year17.85%/2=$892.50 ->$893. Basis5057, gain6943.
    let r = supported(&d);
    assert_eq!(crate::tests::cross::fix_f17::line_3e(&r), 320_160);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 318_943);
}
#[test]
fn d1_rental_property_restores_only_losses_that_reduced_the_allowed_net() {
    // Unit40 roof facts:7100 property loss,9717 operating depreciation.
    // At140000 MAGI the5000 allowance splits2111property/2889operating.
    // At162000 none is allowed. Gross excludes both kinds of deduction,
    // without adding the suspended7100 back into income.
    for (wages, gross, property_allowed) in [(128_000, 140_000, -2111), (150_000, 162_000, 0)] {
        let mut d = crate::tests::cross::fix_f17::k3_base(0);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        add(
            &mut d,
            income(FederalIncome::Rental {
                data: crate::tests::investment::investment::unit40_roof(0, 0, 0),
            }),
        );
        d.screening.as_mut().unwrap().section1231 =
            Some(Section1231History { regular: vec![], amt: vec![] });
        let r = supported(&d);
        assert_eq!(value(&r, "form4797_line8"), property_allowed);
        assert_eq!(crate::tests::cross::fix_f17::line_3e(&r), gross);
        assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), gross);
    }
}

#[test]
fn d6_valid_farm_share_agreement_survives_cold_read_and_follows_source_deletion() {
    use crate::farm_specials::{FarmExternalSourceInput, FarmShareAgreement, FarmSourceCharacter};
    let mut s = Interview::new(TEST_USER, crate::tests::cross::interview::farm_only_return());
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents
        .documents
        .push(document(0, IncomeDocument::W2(crate::tests::cross::interview::w2(2_000_000))));
    accepted(&send(&mut s, page));
    let source = s
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap()
        .id;
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Farming);
    let P::SaveIncomes { documents, farm_sources, .. } = &mut page else { panic!() };
    assert!(farm_sources.iter().any(|choice| choice.id == source));
    // Preserve the bounded original storage supplement's statement facts.
    // Catalog membership proves source identity, not legal character entitlement.
    let agreement = FarmShareAgreement {
        signed: 20250101,
        significant_activity: 20250201,
        production_based: true,
    };
    let mut farm = crate::tests::cross::interview::cash_farm(20_000_000, 0);
    farm.more.specials.external_sources.push(FarmExternalSourceInput {
        source,
        character: FarmSourceCharacter::FishingCrewShare,
        gross_sales: 100_000,
        product_cost: 0,
        agreement: Some(agreement.clone()),
        shareholder_employee: false,
        regularly_used: false,
        ceased: None,
    });
    documents.documents.push(document(0, IncomeDocument::Farm(farm)));
    accepted(&send(&mut s, page));
    assert_eq!(
        crate::tests::cross::interview::stored_farm(&s.draft).1.specials.external_sources[0]
            .agreement,
        Some(agreement)
    );
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    let mut page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.retain(|item| item.id != source);
    accepted(&send(&mut s, page));
    assert!(
        crate::tests::cross::interview::stored_farm(&s.draft)
            .1
            .specials
            .external_sources
            .is_empty()
    );
    crate::tests::core_tests::storage::f18_cold(&s.draft);
    let mut walked = s.draft.clone();
    let _ = crate::interview::prune(&mut walked);
    assert_eq!(walked, s.draft);
    let page = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Farming);
    let P::SaveIncomes { documents, farm_sources, .. } = page else { panic!() };
    assert!(farm_sources.iter().all(|choice| choice.id != source));
    let IncomeDocument::Farm(farm) = &documents.documents[0].document else { panic!() };
    assert!(farm.more.specials.external_sources.is_empty());
}
