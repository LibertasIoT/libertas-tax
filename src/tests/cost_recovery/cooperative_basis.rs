use super::*;
use crate::kernel::{ratio::exact, source_fact::SourceFact};

#[test]
fn published_brown_purchase_refund_preserves_historical_year_and_schedule() {
    // Pub225(2025), ch3: original2024/2025 example, not a2026 return.
    let mut asset = continuing_equipment(20240701, 138_200, false);
    asset.cost = 1_290_000;
    asset.acquired = 20240701;
    asset.class = EquipmentClass::Seven;
    asset.method = EquipmentMethod::Declining150;
    asset.history = table_history(138_200, 0, false);
    let (mut draft, activity, property) = cooperative_asset_refunds(asset, &[[30_000, 0, 0, 0]]);
    draft.tax_year = 2025;
    let before = draft.clone();
    let SourceFact::Reported(purchase) =
        crate::cooperative_basis::purchases(&draft, Owner::Taxpayer, activity, property).unwrap()
    else {
        panic!("actual accepted purchase source")
    };
    assert_eq!(purchase.reduction(false), Some(30_000));
    assert_eq!(purchase.excess(false), exact::whole(0));
    let view = crate::cooperative_basis::AssetView { asset: purchase.asset, purchase };
    let amounts = view.amounts(false, false).unwrap();
    assert_eq!(amounts.opening, 1_121_800);
    assert_eq!(crate::kernel::ratio::dollars(amounts.deduction().unwrap()), 2588);
    assert_eq!(draft, before);
}

#[test]
fn service_year_refund_reduces_bonus_basis_and_keeps_excess_election_unavailable() {
    let (mut draft, activity, _) =
        cooperative_asset_refunds(bonus_equipment(1000, 100), &[[20_000, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let amounts =
        crate::cooperative_basis::source_amounts(&draft, activity, &data.assets[0], false, false)
            .unwrap();
    assert_eq!((amounts.bonus, amounts.macrs, amounts.qualified179_cost), (80_000, 0, 80_000));
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let EquipmentHistory::New { section179, .. } = &mut data.assets[0].history else {
        unreachable!()
    };
    *section179 = Section179::Eligible { elected: 90_000 };
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    assert_eq!(
        crate::cooperative_basis::source_amounts(&draft, activity, &data.assets[0], false, false),
        Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            activity,
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"
        )),
    );
}

#[test]
fn shared_refunds_keep_fractional_source_weights_and_issuer_qualified_excess() {
    let asset = bonus_equipment(1, 33);
    let (mut draft, activity, property) =
        cooperative_asset_refunds(asset, &[[101, 101, 101, 0], [100, 100, 0, 100]]);
    let SourceFact::Reported(first) =
        crate::cooperative_basis::purchases(&draft, Owner::Taxpayer, activity, property).unwrap()
    else {
        panic!("purchase pool")
    };
    assert_eq!(first.reduction(false), Some(33));
    // Income-producing refund201×33%=66.33; capacity33; excess33.33.
    assert_eq!(first.excess(false), exact::ratio(3333, 100).unwrap());
    let SourceFact::Reported(first_income) =
        crate::cooperative_basis::income(&draft, Owner::Taxpayer, activity, false).unwrap()
    else {
        panic!("source income")
    };
    assert_eq!(first_income.qualification[0], first_income.ordinary);
    assert_eq!(
        first_income.qualification[1] + first_income.qualification[2],
        first_income.ordinary
    );
    draft.income.swap(1, 2);
    let SourceFact::Reported(reordered) =
        crate::cooperative_basis::income(&draft, Owner::Taxpayer, activity, false).unwrap()
    else {
        panic!("reordered source income")
    };
    assert_eq!(reordered, first_income);
}

#[test]
fn prior_section179_reserve_is_not_available_purchase_refund_basis() {
    let mut asset = continuing_equipment(20240601, 60_000, false);
    if let EquipmentHistory::Continuing { initial_section179, carryforward, .. } =
        &mut asset.history
    {
        *initial_section179 = 50_000;
        *carryforward = 30_000;
    }
    let (draft, activity, property) = cooperative_asset_refunds(asset, &[[20_000, 0, 0, 0]]);
    let SourceFact::Reported(purchase) =
        crate::cooperative_basis::purchases(&draft, Owner::Taxpayer, activity, property).unwrap()
    else {
        panic!("reserved179 source")
    };
    assert_eq!(purchase.opening_basis, [10_000; 2]);
    assert_eq!(purchase.reduction(false), Some(10_000));
    assert_eq!(purchase.excess(false), exact::whole(10_000));
}

#[test]
fn current_sale_refund_changes_cost_and_personal_basis_without_deemed_depreciation() {
    let mut asset = bonus_equipment(1000, 60);
    asset.disposal = AssetDisposal::Disposed {
        date: 20261201,
        how: DisposalKind::Sold {
            proceeds: 120_000,
            selling_expenses: 0,
            livestock: Livestock::No,
            buyer: Buyer::Unrelated,
            payment: SalePayment::InFull,
        },
    };
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[20_000, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let sale = crate::cooperative_basis::sale(&draft, activity, &data.assets[0], false, false)
        .unwrap()
        .unwrap();
    assert_eq!((sale.basis, sale.depreciation, sale.recapture), (480, 0, 0));
    assert_eq!((sale.personal_basis, sale.personal), (320, 160));
}

#[test]
fn fractional_cent_refund_excess_projects_once_at_the_receipt_owner_boundary() {
    let mut asset = continuing_equipment(20200601, 3, false);
    asset.cost = 100;
    asset.listed = ListedProperty::No { business_percent: 3 };
    asset.method = EquipmentMethod::StraightLine;
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[22, 0, 0, 0]]);
    let receipts =
        crate::cooperative::receipt_totals(&draft, Owner::Taxpayer, activity, false).unwrap();
    // D-2026-10-19-01: exact inside the source owner; once half-up to cents
    // when crossing into the receiving owner. .66c becomes1c, then49c+1c=$1.
    assert_eq!(receipts.taxable, 1);
    assert_eq!(crate::kernel::ratio::dollars(49 + receipts.taxable), 1);
}

#[test]
fn service_year_refund_reduces_full_cost_before_the_native_business_share() {
    let mut asset = bonus_equipment(1, 33);
    asset.cost = 102;
    let original = asset.clone();
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[1, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let amounts =
        crate::cooperative_basis::source_amounts(&draft, activity, &data.assets[0], false, false)
            .unwrap();
    // D02: (102c−1c)×33%, projected once by the native basis owner, is33c.
    assert_eq!(amounts.opening, 33);
    assert_eq!(data.assets[0].cost, original.cost);
    assert_eq!(data.assets[0].history, original.history);
}

#[test]
fn continuing_refund_rounds_the_adjusted_balance_and_not_the_refund_operand() {
    let mut asset = continuing_equipment(20240601, 47, false);
    asset.cost = 100;
    asset.listed = ListedProperty::No { business_percent: 50 };
    asset.method = EquipmentMethod::StraightLine;
    let (draft, activity, property) = cooperative_asset_refunds(asset, &[[1, 0, 0, 0]]);
    let SourceFact::Reported(purchase) =
        crate::cooperative_basis::purchases(&draft, Owner::Taxpayer, activity, property).unwrap()
    else {
        panic!("actual purchase balance")
    };
    // Opening3c−exactrefund.5c=2.5c; owner-boundary half-up leaves3c.
    // Rounding the refund to1c first would incorrectly leave2c.
    assert_eq!(purchase.opening_basis[0], 3);
    assert_eq!(purchase.reduction(false), Some(0));
    let view = crate::cooperative_basis::AssetView { asset: purchase.asset, purchase };
    // The actual event still selects remaining-life continuation:3c over
    // the original remaining3.5years yields1c, rather than the old3c cap.
    assert_eq!(view.amounts(false, false).unwrap().macrs, 1);
}

#[test]
fn equipment_purchase_refund_native_save_rejects_collision_wrong_kind_and_bad_parent() {
    use crate::tests::cross::interview::{
        accepted, asset, business, open_topic, rejected_with, save_businesses,
    };
    let draft = cooperative_equipment_identity_return();
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    let farm = documents
        .documents
        .iter_mut()
        .find_map(|row| match &mut row.document {
            IncomeDocument::Farm(farm) => Some(farm),
            _ => None,
        })
        .unwrap();
    farm.assets = vec![asset(100_000, 20260601, AssetClass::Five)];
    accepted(&send(&mut session, page));
    let mut other = business(100_000);
    other.assets = vec![asset(20_000, 20260601, AssetClass::Five)];
    accepted(&save_businesses(&mut session, vec![other]));
    let foreign_property = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Ordinary { .. }) =>
            {
                Some(data.assets[0].id)
            }
            _ => None,
        })
        .unwrap();
    let original = session.draft.clone();
    let baseline = supported(&original);
    let activity = session.draft.income[1].id;
    let FederalIncome::Business { data } = &session.draft.income[1].income else { unreachable!() };
    let property = data.assets[0].id;
    let mut page = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, cooperative_properties, .. } = &mut page else {
        unreachable!()
    };
    assert!(cooperative_properties.iter().any(|choice| choice.id == property));
    let mut patronage = crate::tests::farm::cooperative::patr_page(120_000, 0, Some(activity));
    patronage.withholding = 0;
    patronage.more.allocations[0].properties.push(CoopPropertyRefundInput {
        distribution: CoopDistributionBox::Patronage,
        property: Some(property),
        source: "Native actual equipment purchase invoice".into(),
        amount: 120_000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    documents
        .documents
        .push(crate::tests::cross::interview::document(0, IncomeDocument::Patr1099(patronage)));
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("farm_{activity}_cooperative_taxable")), 200);
    assert_eq!(
        value(&result, &alloc::format!("farm_{activity}_net_profit")),
        value(&baseline, &alloc::format!("farm_{activity}_net_profit")) + 1200
    );
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let reopened = supported(&session.draft);
    assert_eq!(value(&reopened, "agi"), value(&result, "agi"));
    let parcel = farmland_basis_mut(&mut session.draft).id;
    let page = open_topic(&mut session, TaxSection::Farming);
    for case in 0..3 {
        let mut invalid = page.clone();
        let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
        let farm = documents
            .documents
            .iter_mut()
            .find_map(|row| match &mut row.document {
                IncomeDocument::Farm(farm) => Some(farm),
                _ => None,
            })
            .unwrap();
        if case == 2 {
            farm.assets.push(farm.assets[0].clone());
        } else {
            farm.assets[0].id = [parcel, foreign_property][case];
        }
        rejected_with(
            send(&mut session, invalid),
            if case == 2 {
                "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425"
            } else {
                "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C"
            },
        );
        assert_eq!(session.draft, saved);
    }
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);
}

#[test]
fn mixed_use_refund_personal_basis_excludes_business_credit_reduction() {
    let mut asset = bonus_equipment(1000, 60);
    asset.credit_reduction = 20_000;
    asset.disposal = AssetDisposal::Disposed {
        date: 20261201,
        how: DisposalKind::Sold {
            proceeds: 120_000,
            selling_expenses: 0,
            livestock: Livestock::No,
            buyer: Buyer::Unrelated,
            payment: SalePayment::InFull,
        },
    };
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[20_000, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let sale = crate::cooperative_basis::sale(&draft, activity, &data.assets[0], false, false)
        .unwrap()
        .unwrap();
    assert_eq!(sale.personal_basis, 320);
    assert_eq!(sale.personal, 160);
}

#[test]
fn disjoint_qualified_refund_excess_keeps_one_cent_target_in_form_order() {
    let mut asset = continuing_equipment(20200601, 50, false);
    asset.cost = 100;
    asset.listed = ListedProperty::No { business_percent: 50 };
    asset.method = EquipmentMethod::StraightLine;
    let (mut draft, activity, _) = cooperative_asset_refunds(asset, &[[1, 1, 1, 0], [1, 1, 0, 1]]);
    let first = crate::cooperative_basis::income(&draft, Owner::Taxpayer, activity, false).unwrap();
    let SourceFact::Reported(ref income) = first else { panic!("known source") };
    assert_eq!((income.ordinary, income.qualification), (1, [1, 1, 0]));
    draft.income.swap(1, 2);
    assert_eq!(
        crate::cooperative_basis::income(&draft, Owner::Taxpayer, activity, false).unwrap(),
        first
    );
}

#[test]
fn purchase_refund_excess_reaches_native_c_f_and_4835_receipts_and_amt_once() {
    for kind in 0..3 {
        let mut asset = continuing_equipment(20240601, 9_000, false);
        asset.cost = 10_000;
        asset.acquired = 20170927;
        asset.method = EquipmentMethod::Regular;
        if let EquipmentHistory::Continuing { amt, .. } = &mut asset.history {
            *amt = EquipmentAmtHistory::Separate {
                prior_depreciation: 8_000,
                schedule: EquipmentSchedule::Manual,
            };
        }
        let source = match kind {
            0 => cash_business(1000),
            1 => raised_farm(1000, 0),
            _ => FederalIncome::FarmRental { data: farm_rental_data(1000, 0) },
        };
        let (draft, activity, _) =
            cooperative_asset_refunds_on(asset, &[[3000, 3000, 3000, 0]], source);
        let result = supported(&draft);
        let prefix = match kind {
            0 => "business",
            1 => "farm",
            _ => "farm_rental",
        };
        let tail = if kind == 2 { "net_income" } else { "net_profit" };
        assert_eq!(value(&result, &alloc::format!("{prefix}_{activity}_{tail}")), 1020);
        assert_eq!(
            crate::cooperative::receipt_totals(&draft, Owner::Taxpayer, activity, false)
                .unwrap()
                .qualification,
            [2000, 2000, 0]
        );
        assert_eq!(
            crate::cooperative::receipt_totals(&draft, Owner::Taxpayer, activity, true)
                .unwrap()
                .taxable,
            1000
        );
        if kind != 2 {
            assert_eq!(value(&result, "amt_business_depreciation_adjustment"), -10);
        }
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn fell_use_refund_keeps_current_personal_and_sale_cost_basis() {
    let mut asset = continuing_equipment(20240601, 60_000, false);
    asset.method = EquipmentMethod::StraightLine;
    asset.listed = ListedProperty::No { business_percent: 40 };
    asset.earlier_business_percent = Some(80);
    asset.disposal = AssetDisposal::Disposed {
        date: 20261201,
        how: DisposalKind::Sold {
            proceeds: 120_000,
            selling_expenses: 0,
            livestock: Livestock::No,
            buyer: Buyer::Unrelated,
            payment: SalePayment::InFull,
        },
    };
    let original = asset.sale_for(false, false).unwrap();
    assert_eq!(original.personal_basis, 600);
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[125, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let sale = crate::cooperative_basis::sale(&draft, activity, &data.assets[0], false, false)
        .unwrap()
        .unwrap();
    // Actual current40% share: business cost40000c−50c rounds to$400;
    // personal60000c−75c rounds to$599. Historical80% must not become
    // a new personal-use source or subtract100c from current business cost.
    assert_eq!((sale.basis, sale.personal_basis), (400, 599));
}

#[test]
fn purchase_refund_numeric_overflow_is_located_and_not_missing_history() {
    let (draft, activity, property) =
        cooperative_asset_refunds(bonus_equipment(1000, 100), &[[i64::MAX, 0, 0, 0], [1, 0, 0, 0]]);
    assert!(
        matches!(crate::cooperative_basis::purchases(&draft, Owner::Taxpayer, activity, property),
        Err(crate::Section179PreparedIssue::AmountOverflow(id)) if id == activity)
    );
}

#[test]
fn building_partial_uses_contextual_recovery_and_existing_component_cost_ratio() {
    let asset = cooperative_partial_building();
    let native = asset.partial_for().unwrap();
    let original = asset.clone();
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[3_900_000, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let part = crate::cooperative_basis::partial(&draft, activity, &data.assets[0], false, false)
        .unwrap()
        .unwrap();
    assert_eq!(part.cost, 35_100);
    assert!(part.depreciation < native.depreciation);
    assert_eq!(data.assets[0].cost, original.cost);
    assert_eq!(data.assets[0].history, original.history);
}

#[test]
fn qualified_sstb_refund_excess_keeps_actual_related_expense_and_final_qbi() {
    let mut asset = continuing_equipment(20240601, 100_000, false);
    asset.method = EquipmentMethod::StraightLine;
    let mut farm = raised_farm(10000, 1000);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    data.expenses[0].category = BusinessExpenseCategory::Feed;
    let (mut draft, activity, _) = cooperative_asset_refunds_on(asset, &[[5000, 0, 0, 5000]], farm);
    let FederalIncome::Cooperative { data } = &mut draft.income[1].income else { unreachable!() };
    data.allocations[0].sstb_expenses.push(CoopSstbExpense {
        category: BusinessExpenseCategory::Feed,
        amount: 100_000,
        source: "Actual related feed invoice already deducted on this farm".into(),
    });
    let receipts =
        crate::cooperative::receipt_totals(&draft, Owner::Taxpayer, activity, false).unwrap();
    assert_eq!(receipts.qualification, [0, 0, 5000]);
    assert_eq!(receipts.related_sstb_expenses, 5000);
    let mut ordinary = draft.clone();
    let FederalIncome::Cooperative { data } = &mut ordinary.income[1].income else {
        unreachable!()
    };
    data.allocations[0].properties.clear();
    // An exhausted-basis refund has the same taxable and qualified classes
    // as this ordinary distribution, and no second expense deduction.
    let actual = supported(&draft);
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&actual, id), value(&expected, id), "{id}");
    }
    assert_eq!(actual.tax, expected.tax);
}

#[test]
fn current_refund_keeps_native_section179_income_capacity_and_excess_election_issue() {
    let mut asset = elected_asset(1000, 800, BonusElection::Apply);
    let (mut draft, activity, _) =
        cooperative_asset_refunds_on(asset.clone(), &[[20_000, 0, 0, 0]], cash_business(100));
    let result = supported(&draft);
    assert_eq!(value(&result, "section179_deduction"), 100);
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    if let EquipmentHistory::New { section179, .. } = &mut asset.history {
        *section179 = Section179::Eligible { elected: 90_000 };
    }
    data.assets[0].history = asset.history;
    let refused = estimate(&draft);
    assert_eq!(refused.state, ResultState::Unsupported);
    assert!(
        refused
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_COOP_PROPERTY_HISTORY_NEEDED")
    );
    assert_eq!(draft.income[0].id, activity);

    // Native legacy179 must use the same routed receipts as ScheduleC.
    // With no purchase adjustment: C100+ordinaryPATR100 funds200 of the
    // 1000 election. The old data.receipts-only path incorrectly funded100.
    let (mut ordinary, _, _) = cooperative_asset_refunds_on(
        elected_asset(1000, 1000, BonusElection::Apply),
        &[[10_000, 0, 0, 0]],
        cash_business(100),
    );
    let FederalIncome::Cooperative { data } = &mut ordinary.income[1].income else {
        unreachable!()
    };
    data.allocations[0].properties.clear();
    assert_eq!(value(&supported(&ordinary), "section179_deduction"), 200);
}

#[test]
fn fractional_building_component_refund_projects_its_complete_remaining_cost_once() {
    let mut asset = continuing_equipment(20240601, 0, false);
    asset.cost = 5;
    asset.class = EquipmentClass::ThirtyNine;
    asset.method = EquipmentMethod::StraightLine;
    asset.listed = ListedProperty::No { business_percent: 70 };
    asset.partial = Some(PartialDisposition { date: 20260610, cost_of_part: 2, proceeds: 0 });
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[2, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let amounts =
        crate::cooperative_basis::source_amounts(&draft, activity, &data.assets[0], false, false)
            .unwrap();
    // Original component business cost1c − exact refund1.4c×2/5=.56c
    // leaves.44c, projected once to0c. Rounding the whole refund first
    // would subtract only.4c and incorrectly leave1c.
    assert_eq!(amounts.partial.unwrap().cost, 0);
}

#[test]
fn purchase_refund_caps_documented_auto_recovery_at_actual_remaining_basis() {
    let asset = ordinary_original_auto(HistoricalAutoKind::Car);
    let refund = asset.opening_basis() - 100;
    let original = asset.clone();
    let (draft, activity, _) = cooperative_asset_refunds(asset, &[[refund, 0, 0, 0]]);
    let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
    let amounts =
        crate::cooperative_basis::source_amounts(&draft, activity, &data.assets[0], false, false)
            .unwrap();
    assert_eq!((amounts.opening, amounts.macrs), (100, 100));
    assert_eq!(data.assets[0].history_details, original.history_details);
    assert_eq!(data.assets[0].history, original.history);
}

#[test]
fn community_mfs_refund_keeps_whole_c_f_election_and_projects_4835_once() {
    for kind in 0..3 {
        let asset = if kind == 2 {
            bonus_equipment(1000, 100)
        } else {
            elected_asset(1000, 400, BonusElection::Apply)
        };
        let source = match kind {
            0 => cash_business(1000),
            1 => raised_farm(1000, 0),
            _ => FederalIncome::FarmRental { data: farm_rental_data(1000, 0) },
        };
        let (draft, activity, _) =
            cooperative_asset_refunds_on(asset, &[[20_000, 0, 0, 0]], source);
        let mut draft = cooperative_community_mfs(draft);
        let result = supported(&draft);
        // Actual C/F sources stay whole for operator SE; native Totals then
        // shares the200 profit. Form4835 owns its already-divided400 cost.
        assert_eq!(value(&result, "agi"), 100);
        if kind == 2 {
            assert_eq!(value(&result, &alloc::format!("farm_rental_{activity}_net_income")), 100);
            assert_eq!(
                value(&result, &alloc::format!("business_{activity}_depreciation_total")),
                400
            );
        } else {
            assert_eq!(value(&result, "section179_eligible_cost"), 800);
            assert_eq!(value(&result, "section179_deduction"), 400);
            assert_eq!(
                value(&result, &alloc::format!("business_{activity}_depreciation_total")),
                800
            );
            let FederalIncome::Business { data } = &mut draft.income[0].income else {
                unreachable!()
            };
            let EquipmentHistory::New { section179, .. } = &mut data.assets[0].history else {
                unreachable!()
            };
            *section179 = Section179::Eligible { elected: 90_000 };
            assert!(
                estimate(&draft)
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_COOP_PROPERTY_HISTORY_NEEDED")
            );
        }
    }
}

#[test]
fn community_4835_sale_uses_whole_refund_before_existing_property_projection() {
    let mut asset = bonus_equipment(1000, 100);
    asset.disposal = AssetDisposal::Disposed {
        date: 20261201,
        how: DisposalKind::Sold {
            proceeds: 120_000,
            selling_expenses: 0,
            livestock: Livestock::No,
            buyer: Buyer::Unrelated,
            payment: SalePayment::InFull,
        },
    };
    let (draft, activity, _) = cooperative_asset_refunds_on(
        asset,
        &[[20_000, 0, 0, 0]],
        FederalIncome::FarmRental { data: farm_rental_data(1000, 0) },
    );
    let mut draft = cooperative_community_mfs(draft);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let entries = crate::community::property_entries(&draft);
    let entry = entries.iter().find(|entry| entry.id == activity).unwrap();
    let FederalIncome::FarmRental { data } = &entry.income else { unreachable!() };
    assert_eq!(data.assets[0].cost, 50_000);
    let sale = crate::cooperative_basis::sale(&draft, activity, &data.assets[0], false, false)
        .unwrap()
        .unwrap();
    assert_eq!((sale.basis, sale.gain), (400, 200));
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{activity}_asset_0_disposal_cost")), 400);
    let (draft, activity, _) = cooperative_asset_refunds_on(
        cooperative_partial_building(),
        &[[3_900_000, 0, 0, 0]],
        FederalIncome::FarmRental { data: farm_rental_data(1000, 0) },
    );
    let draft = cooperative_community_mfs(draft);
    let entries = crate::community::property_entries(&draft);
    let entry = entries.iter().find(|entry| entry.id == activity).unwrap();
    let FederalIncome::FarmRental { data } = &entry.income else { unreachable!() };
    let partial =
        crate::cooperative_basis::partial(&draft, activity, &data.assets[0], false, false)
            .unwrap()
            .unwrap();
    assert_eq!(partial.cost, 17_550); // Whole35100, existing property half once.
}

#[test]
fn continuing_purchase_refund_page_save_reopen_resave_restart_uses_adjusted_gds_basis() {
    use crate::tests::cross::interview::open_topic;
    let (mut session, activity, property) = continuing_purchase_refund_session();
    assert_eq!(session.draft.tax_year, 2026);
    let saved = session.draft.clone();
    let result = supported(&saved);
    let SourceFact::Reported(purchase) =
        crate::cooperative_basis::purchases(&saved, Owner::Taxpayer, activity, property).unwrap()
    else {
        panic!("accepted refund source")
    };
    let view = crate::cooperative_basis::AssetView { asset: purchase.asset, purchase };
    let amounts = view.amounts(false, false).unwrap();
    // Pub946 remaining original clock: $1000−$520−$300=$180.
    // Pub225/Pub946 without tables, existing GDS continuation owner:
    // remaining3.5years reciprocal .2857×2=.5714; $180×.5714=$102.852.
    // This is neither original cost×third-year19.2%=$192 nor a restart.
    assert_eq!((amounts.opening, amounts.macrs), (18_000, 10_285));
    assert_eq!(line_amount(&result, &alloc::format!("business_{activity}_asset_0_macrs")), 10_285);
    assert_eq!(line_amount(&result, &alloc::format!("business_{activity}_net_profit")), 989_715);
    for _ in 0..2 {
        session = restart(session.draft);
        let request = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &request else { unreachable!() };
        assert!(documents.documents.iter().any(|row| matches!(&row.document,
            IncomeDocument::Business(page) if page.assets[0].id == property)));
        accepted(&send(&mut session, request));
        assert_eq!(session.draft, saved);
        assert_eq!(estimate(&session.draft), result);
    }
}
