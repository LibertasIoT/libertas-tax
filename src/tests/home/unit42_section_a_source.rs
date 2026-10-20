use super::*;
use crate::casualty::section_a;
use crate::tests::business::business::cents;

fn event(
    declaration: CasualtyDeclaration,
    basis: i64,
    before: i64,
    reimbursement: i64,
) -> PersonalCasualty {
    let declared = !matches!(declaration, CasualtyDeclaration::Undeclared);
    PersonalCasualty {
        id: 0,
        label: "Synthetic source storm".into(),
        date: 20260615,
        declaration,
        zip: if declared { "10001".into() } else { "".into() },
        source_complete: true,
        properties: vec![CasualtyProperty {
            id: 0,
            home_site: None,
            business_damage: None,
            label: "Synthetic source home".into(),
            location: "Synthetic City, NY".into(),
            acquisition: CasualtyAcquisition::Purchased { date: 20200101 },
            basis,
            before,
            after: 0,
            reimbursement,
            received: reimbursement,
        }],
    }
}

#[test]
fn home_f14_actual_personal_and_all_personal_use_the_same_section_a_owner() {
    let whole = event(
        CasualtyDeclaration::MajorDisaster { reference: "DR-4999".into() },
        30_000_000,
        5_000_000,
        3_000_000,
    );
    let personal = event(
        CasualtyDeclaration::MajorDisaster { reference: "DR-4999".into() },
        27_000_000,
        4_500_000,
        2_700_000,
    );
    assert!(whole.valid());
    assert!(personal.valid());
    // Approved F14: actual personal loss18,000 less500; all-personal
    // worksheet loss20,000 less500. No 10%-AGI floor for this qualified loss.
    let hypothetical = section_a(&[whole], 50_000);
    let actual = section_a(&[personal], 100_000);
    assert_eq!(hypothetical.qualified_net, 19_500);
    assert_eq!(hypothetical.deduction, 19_500);
    assert_eq!(actual.qualified_net, 17_500);
    assert_eq!(actual.deduction, 17_500);
}

#[test]
fn home_hypothetical_agi_excludes_business_income_and_deductions() {
    let whole = event(
        CasualtyDeclaration::Federal { reference: "EM-4999".into() },
        30_000_000,
        5_000_000,
        3_000_000,
    );
    let personal = event(
        CasualtyDeclaration::Federal { reference: "EM-4999".into() },
        27_000_000,
        4_500_000,
        2_700_000,
    );
    // Whole20,000 less100 less5,000; actualpersonal18,000 less100
    // less10,000. HOME supplies the genuine modified AGI, not a second
    // deduction answer or an assumed business-profit adjustment here.
    let hypothetical = section_a(&[whole], 50_000);
    let actual = section_a(&[personal], 100_000);
    assert_eq!(hypothetical.ordinary_deduction, 14_900);
    assert_eq!(hypothetical.qualified_net, 0);
    assert_eq!(actual.ordinary_deduction, 7_900);
    assert_eq!(actual.qualified_net, 0);
}

#[test]
fn section_a_source_retains_gain_offsets_and_qualified_character() {
    let ordinary =
        event(CasualtyDeclaration::Federal { reference: "EM-4999".into() }, 1_000_000, 500_000, 0);
    let qualified = event(
        CasualtyDeclaration::MajorDisaster { reference: "DR-4999".into() },
        1_000_000,
        1_000_000,
        0,
    );
    let undeclared = event(CasualtyDeclaration::Undeclared, 1_000_000, 1_000_000, 0);
    let gain = event(CasualtyDeclaration::Undeclared, 100_000, 0, 400_000);
    let result = section_a(&[ordinary, qualified, undeclared, gain], 10_000);
    assert_eq!(result.gains, 3_000);
    assert_eq!(result.declared_after_events, 14_400);
    assert_eq!(result.undeclared_after_events, 9_900);
    assert_eq!(result.qualified_after_events, 9_500);
    assert_eq!(result.after_events, 17_400);
    assert_eq!(result.net, 14_400);
    assert_eq!(result.qualified_net, 9_500);
    assert_eq!(result.ordinary_deduction, 3_900);
    assert_eq!(result.deduction, 13_400);
}

#[test]
fn section_a_source_rounds_once_per_event_after_its_single_floor() {
    let mut source =
        event(CasualtyDeclaration::Federal { reference: "EM-4999".into() }, 100_000, 5_025, 0);
    source.properties.push(source.properties[0].clone());
    source.properties[1].label = "Second synthetic source item".into();
    // Two50.25 losses, one100 eventfloor, leaves50c, rounded once to1.
    let result = section_a(&[source], -500);
    assert_eq!(result.after_events, 1);
    assert_eq!(result.ordinary_deduction, 1);
}

#[test]
fn section_a_incomplete_sources_keep_gains_without_assuming_losses() {
    {
        let answer = false;
        let mut source = event(
            CasualtyDeclaration::MajorDisaster { reference: "DR-4999".into() },
            1_000_000,
            1_000_000,
            0,
        );
        let gain = event(CasualtyDeclaration::Undeclared, 100_000, 0, 400_000);
        source.source_complete = answer;
        let result = section_a(&[source, gain], 10_000);
        assert_eq!(result.gains, 3_000);
        assert_eq!(result.after_events, 0);
        assert_eq!(result.deduction, 0);
    }
}

#[test]
fn home_shared_reductions_follow_d04_input_order_and_keep_cents() {
    let mut source = major_disaster("Shared source", 2_000_000);
    source.id = 11;
    source.properties[0].id = 12;
    source.properties[0].home_site = Some(9);
    let mut other = source.properties[0].clone();
    other.id = 13;
    other.home_site = None;
    other.basis = 1_000_000;
    other.before = 1_000_000;
    source.properties.push(other);
    let rows = crate::casualty::all_personal_allowances(&[source], 50_000_000).unwrap();
    assert_eq!(rows[0].qualified_cents, 1_966_667);
    assert_eq!(rows[1].qualified_cents, 983_333);
    assert_eq!(rows.iter().map(|row| row.qualified_cents).sum::<i64>(), 2_950_000);
}

#[test]
fn casualty_native_validation_has_no_twenty_property_cap() {
    let mut source = major_disaster("Many properties", 100);
    source.properties.resize(21, source.properties[0].clone());
    assert!(source.valid());
}

#[test]
fn hypothetical_property_allowance_preserves_one_odd_cent_across_owners() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(cash_business(100_000)));
    let source = d.income.last().unwrap().id;
    add_actual_office(&mut d, source, actual_office(1000, 2000, HomeFixture::Rented { rent: 0 }));
    add(&mut d, income(cash_business(100_000)));
    let spouse_entry = d.income.last_mut().unwrap();
    spouse_entry.owner = Owner::Spouse;
    let spouse_source = spouse_entry.id;
    let site = &mut d.home_sites[0];
    let mut spouse = site.claims[0].clone();
    spouse.owner = Owner::Spouse;
    spouse.activities[0].activity = spouse_source;
    site.claims.push(spouse);
    // D04 property allowance 1,966,667c, split 50/50 through saved claim
    // order: first rounds 983,333.5c up; the other keeps the exact residual.
    assert_eq!(
        crate::home_sites::casualty_allowance(site, Owner::Taxpayer, 1_966_667),
        Ok(983_334)
    );
    assert_eq!(crate::home_sites::casualty_allowance(site, Owner::Spouse, 1_966_667), Ok(983_333));
}

#[test]
fn canonical_casualty_ids_preserve_same_labels_and_survive_source_reordering() {
    let mut draft = home_casualty_return(&[2_000_000, 1_000_000]);
    assert!(draft.valid_stored());
    let personal = crate::home_sites::personal_casualties(&draft).unwrap();
    assert_eq!(
        personal.iter().map(|event| event.properties[0].basis).collect::<Vec<_>>(),
        vec![1_000_000, 500_000]
    );
    let first_id = personal[0].id;
    draft.deductions.as_mut().unwrap().personal_casualties.reverse();
    draft.home_sites[0].address = "Renamed synthetic home".into();
    let reordered = crate::home_sites::personal_casualties(&draft).unwrap();
    assert_eq!(reordered.iter().find(|event| event.id == first_id).unwrap(), &personal[0]);
    assert_eq!(restart(draft.clone()).draft, draft);
    let events = &mut draft.deductions.as_mut().unwrap().personal_casualties;
    events[0].id = events[1].id;
    assert!(!draft.valid_stored());
}

#[test]
fn linked_property_identity_requires_positive_unambiguous_ids() {
    let baseline = home_casualty_return(&[2_000_000]);
    for ambiguous in [false, true] {
        let mut draft = baseline.clone();
        let event = &mut draft.deductions.as_mut().unwrap().personal_casualties[0];
        if ambiguous {
            let mut duplicate = event.properties[0].clone();
            duplicate.home_site = None;
            duplicate.business_damage = None;
            event.properties.push(duplicate);
        } else {
            event.properties[0].id = 0;
        }
        assert!(!draft.valid_stored());
    }
    let mut draft = baseline;
    draft.home_sites[0].casualties[0] = 0;
    draft.deductions.as_mut().unwrap().personal_casualties[0].id = 0;
    assert!(!draft.valid_stored());
}

#[test]
fn linked_home_casualty_uses_explicit_damage_kind_and_leaves_business_expenses_once() {
    for (kind, allowed) in [
        (HomeCasualtyDamage::Damaged, 2_500),
        (HomeCasualtyDamage::Destroyed, 10_000),
        (HomeCasualtyDamage::Stolen, 10_000),
    ] {
        let mut draft = home_casualty_return(&[2_000_000]);
        let property = &mut draft.deductions.as_mut().unwrap().personal_casualties[0].properties[0];
        property.before = 500_000;
        property.business_damage = Some(kind);
        // Basis20,000, FMV decrease5,000, no reimbursement, 50% business.
        // Damaged uses the lesser; destroyed/stolen uses business basis.
        let result = supported(&draft);
        assert_eq!(value(&result, "home_office_casualty_ordinary"), -allowed);
        assert_eq!(value(&result, "business_2_home_office"), 0);
        assert_eq!(value(&result, "business_2_net_profit"), 50_000);
    }
}

#[test]
fn deliberately_unavailable_linked_damage_keeps_accepted_source_without_a_zero_loss() {
    let mut draft = home_casualty_return(&[2_000_000]);
    draft.deductions.as_mut().unwrap().personal_casualties[0].properties[0].business_damage =
        Some(HomeCasualtyDamage::Unavailable);
    assert!(draft.valid_stored());
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.tax.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "HOME_CASUALTY_DAMAGE_SOURCE_UNAVAILABLE")
    );
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn home_interest_and_points_join_before_the_personal_whole_dollar_rounding() {
    let mut draft = office_return(
        50_000,
        actual_office(1000, 2000, owned_office_home(99, 0, 0, 0, 0, 20200101)),
    );
    office_itemize(&mut draft, 0, 0, 99, 0);
    let loan = &mut draft.deductions.as_mut().unwrap().mortgage[0];
    loan.incurred = 20260101;
    loan.points = MortgagePoints::Purchase { amount: 1 };
    // 2026-10-17 approved HOME3.3/Money precision: 50% of99c interest
    // plus1c points leaves exactly50c personal, reported once as1dollar.
    // Rounding the separate business operands first would leave49c and0.
    assert_eq!(value(&supported(&draft), "mortgage_interest_8a"), 1);
}

#[test]
fn partial_home_gross_uses_schedule_c_income_after_inventory_cost() {
    let mut draft =
        office_return(1_000, actual_office(1000, 2000, HomeFixture::Rented { rent: 100_000 }));
    let source = draft.income[0].id;
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.inventory = BusinessInventory::Cost {
        method: InventoryMethod::Cost,
        opening: 0,
        purchases: 60_000,
        personal: 0,
        labor: 0,
        materials: 0,
        other: 0,
        closing: 0,
        eligible: true,
    };
    let inputs = crate::home_office::HomeIncome::from_source(
        &draft,
        source,
        match &draft.income[0].income {
            FederalIncome::Business { data } => data,
            _ => unreachable!(),
        },
        70_000,
    )
    .unwrap();
    assert_eq!((inputs.gross_cents, inputs.operating_cents), (40_000, 10_000));
    let activity = &mut draft.home_sites[0].claims[0].activities[0];
    activity.gross_share = 5000;
    activity.attribution_reason = "Half of the income arose from work in this home".into();
    // 2026-10-17 HOME3.2: 50% of line7 ($400) is $200. COGS has
    // already reduced line7 and must not also reduce the line28 operand.
    let result = supported(&draft);
    assert_eq!(cents(&result, &office_line_id(&result, "income")), 20_000);
    assert_eq!(value(&result, &alloc::format!("business_{source}_home_office")), 200);
}

#[test]
fn hypothetical_personal_allowance_cannot_create_a_business_loss_cent() {
    let mut draft = home_casualty_return(&[10]);
    let event = &mut draft.deductions.as_mut().unwrap().personal_casualties[0];
    event.properties[0].before = 10;
    event.properties[0].reimbursement = 9;
    event.properties[0].received = 9;
    let mut other = event.properties[0].clone();
    other.id = draft.next_id;
    draft.next_id += 1;
    other.home_site = None;
    other.business_damage = None;
    other.basis = 100_000;
    other.before = 100_000;
    other.reimbursement = 0;
    other.received = 0;
    event.properties.push(other);
    // The original-order floor allocation leaves one personal loss cent on
    // the home. At 50% business, basis5c minus reimbursement5c is zero.
    // That hypothetical cent cannot become an actual business deduction.
    let context = crate::home_office::HomeContext::default();
    assert_eq!(crate::home_office::casualty_ordinary(&draft, None, &context).unwrap(), 0);
}

#[test]
fn home_salt_iteration_keeps_the_preceding_cap_at_one_dollar_magi_tolerance() {
    let draft = empty_return(FederalStatus::Single);
    let initial =
        crate::home_office::HomeContext { salt_cap_cents: 1_000_000, ..Default::default() };
    let first =
        crate::home_office::next_context(&draft, 500_000, 505_000, 4_040_000, None, &initial)
            .unwrap();
    assert!(!initial.stable_with(&first));
    assert_eq!(first.salt_cap_cents, 4_040_000);
    let second =
        crate::home_office::next_context(&draft, 500_000, 505_001, 4_039_970, None, &first)
            .unwrap();
    // Approved map3.3: stop at a $1 successive SALT-MAGI difference,
    // retaining this return pass's preceding cap instead of the new cap.
    assert!(first.stable_with(&second));
    assert_eq!(second.salt_cap_cents, 4_040_000);
}

#[test]
fn home_limit_withholds_unattributed_sale_gain_without_dropping_the_source() {
    let baseline =
        office_return(1_000, actual_office(1000, 2000, HomeFixture::Rented { rent: 20_000 }));
    for (goodwill, unavailable) in [(0, false), (10_000, true)] {
        let mut draft = baseline.clone();
        let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
        data.sale = Some(BusinessSale {
            date: 20260615,
            inventory: 0,
            goodwill,
            covenant: 0,
            special: false,
        });
        assert!(draft.valid_stored());
        let result = estimate(&draft);
        if unavailable {
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(result.tax.is_none());
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE")
            );
            assert_eq!(restart(draft.clone()).draft, draft);
        } else {
            assert_eq!(value(&supported(&draft), "business_2_home_office"), 100);
        }
    }
}

#[test]
fn casualty_page_groups_home_damage_without_inventing_missing_source_facts() {
    let draft = home_casualty_return(&[2_000_000]);
    let mut source = draft.deductions.as_ref().unwrap().personal_casualties[0].clone();
    for damage in [HomeCasualtyDamage::Damaged, HomeCasualtyDamage::Unavailable] {
        source.properties[0].business_damage = Some(damage);
        let page = PersonalCasualtyPage::try_from(source.clone()).unwrap();
        let decoded = PersonalCasualtyPage::from_avro(&page.to_avro()).unwrap();
        assert_eq!(PersonalCasualty::from(decoded), source);
    }
    source.properties[0].business_damage = None;
    assert_eq!(
        PersonalCasualtyPage::try_from(source.clone()),
        Err(("HOME_CASUALTY_DAMAGE_REQUIRED", 0)),
    );
    source.properties[0].home_site = None;
    let page = PersonalCasualtyPage::try_from(source.clone()).unwrap();
    assert!(page.properties[0].home_link.is_none());
    assert_eq!(PersonalCasualty::from(page), source);
}

#[test]
fn simplified_owned_home_needs_no_building_recovery_records() {
    let mut draft = office_return(
        50_000,
        actual_office(200, 2000, owned_office_home(0, 0, 0, 1_000_000, 0, 20200101)),
    );
    let HomeTenure::Owned { building, improvements } = &mut draft.home_sites[0].tenure else {
        unreachable!()
    };
    // Retain the independently entered improvement facts when the original
    // building record is unavailable; simplified use does not consume either.
    improvements.push(*building.take().unwrap());
    let actual = draft.home_sites[0].claims[0].method.clone();
    draft.home_sites[0].claims[0].method = simplified_claim(200).method;
    supported(&draft);
    assert_eq!(restart(draft.clone()).draft, draft);
    draft.home_sites[0].claims[0].method = actual;
    assert!(draft.valid_stored());
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.tax.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "HOME_DEPRECIATION_HISTORY_UNAVAILABLE")
    );
}

#[test]
fn home_hypothetical_agi_preserves_actual_nonhome_expenses() {
    // 2026-10-17 D06; 2026 draft i8829 printed p3 line9 Step1/p4 MIP
    // Step1. Home gross50,000/home-use deductions5,000 are excluded;
    // nonhome costs10,000 remain in observed actual AGI145,000. The prior
    // limit-net reuse incorrectly produced110,000 instead of100,000.
    let mut draft = office_return(
        50_000,
        actual_office(1000, 2000, HomeFixture::Rented { rent: 1_000_000 }),
    );
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Nonhome business costs".into() },
        1_000_000,
    )];
    let observed = crate::home_office::HomeContext {
        agi_dollars: 145_000,
        salt_cap_cents: 1_000_000,
        ..Default::default()
    };
    let next = crate::home_office::next_context(
        &draft, 145_000, 145_000, 1_000_000, None, &observed,
    ).unwrap();
    let modified = next.modified_agi_cents[&draft.home_sites[0].id];
    assert_eq!(modified, 10_000_000);
    assert_eq!(next.agi_dollars, 145_000);
    // Independent worksheet consumers of the modified AGI; the rented
    // fixture does not itself claim mortgage insurance.
    let agi = crate::kernel::ratio::dollars(modified);
    assert_eq!(crate::mortgage_limit::insurance_phaseout_cents(100_000, 110_000, false), 0);
    assert_eq!(crate::mortgage_limit::insurance_phaseout_cents(100_000, agi, false), 100_000);
    let casualty = event(
        CasualtyDeclaration::Federal { reference: "EM-4999".into() },
        2_000_000, 2_000_000, 0,
    );
    assert_eq!(section_a(core::slice::from_ref(&casualty), 110_000).ordinary_deduction, 8_900);
    assert_eq!(section_a(core::slice::from_ref(&casualty), agi).ordinary_deduction, 9_900);
}

#[test]
fn site_mortgage_premium_phaseout_rounds_modified_agi_half_up() {
    // Modified AGI $100,000.60 is $100,001 like every other premium phase-out caller, one
    // step over the $100,000 start; truncating to $100,000 would take no reduction. A $300
    // profit is below the $500 business premium, so the reduction shows in the deduction.
    let mut draft = office_return(
        300,
        actual_office(1000, 2000, owned_office_home(99, 0, 0, 0, 0, 20200101)),
    );
    office_itemize(&mut draft, 0, 0, 99, 100_000);
    draft.deductions.as_mut().unwrap().mortgage[0].incurred = 20260101;
    let site = draft.home_sites[0].id;
    let modified = |cents: i64| {
        let observed = crate::home_office::HomeContext {
            agi_dollars: 100_001,
            salt_cap_cents: 1_000_000,
            modified_agi_cents: [(site, cents)].into(),
            ..Default::default()
        };
        crate::home_office::next_context(&draft, 100_001, 100_001, 1_000_000, None, &observed)
            .unwrap()
            .modified_agi_cents[&site]
    };
    assert_eq!(modified(10_000_060), modified(10_000_100));
    assert_ne!(modified(10_000_000), modified(10_000_100));
}
