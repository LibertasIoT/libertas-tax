use super::*;

#[test]
fn macrs_first_year_manual_rates_cover_every_equipment_class_and_method() {
    for (class, regular, slower, straight) in [
        (EquipmentClass::Three, 400000, 300000, 200000),
        (EquipmentClass::Five, 240000, 180000, 120000),
        (EquipmentClass::Seven, 171429, 128571, 85714),
        (EquipmentClass::Ten, 120000, 90000, 60000),
        (EquipmentClass::Fifteen, 60000, 60000, 40000),
        (EquipmentClass::Twenty, 45000, 45000, 30000),
    ] {
        for (method, expected) in [
            (EquipmentMethod::Regular, regular),
            (EquipmentMethod::Declining150, slower),
            (EquipmentMethod::StraightLine, straight),
        ] {
            let asset = macrs_equipment(12000, class, method, 20260601);
            assert_eq!(asset.deduction(false), expected, "{class:?} {method:?}");
            let d = business_with_bonus(100000, asset);
            let r = supported(&d);
            let actual = r.lines.iter().find(|v| v.id == "business_2_asset_0_macrs").unwrap();
            assert_eq!(actual.amount, expected);
            assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 0);
        }
    }
}

#[test]
fn phased_bonus_mid_quarter_and_class_election_out_are_separate() {
    let mut asset = bonus_equipment(10000, 100);
    asset.acquired = 20250119;
    asset.placed = 20261001;
    let mut d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 1);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 2000);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 400);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 500);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 9500);
    // Both acquisition cohorts can claim bonus in one recovery class.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::Apply,
        tables: false,
    };
    data.assets.push(bonus_equipment(2000, 100));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_1_bonus"), 2000);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 400);
}

#[test]
fn section179_joint_income_shares_limits_and_separate_needs_spouse_records() {
    let mut d = complete(FederalStatus::Joint);
    for (owner, cost) in [(Owner::Taxpayer, 120000), (Owner::Spouse, 80000)] {
        let mut e = income(cash_business(0));
        e.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::Business { data } = &mut e.income else { panic!() };
        data.assets.push(elected_asset(cost, cost, BonusElection::ElectOut));
        add(&mut d, e);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 100000);
    assert_eq!(value(&r, "section179_carryforward"), 100000);
    let mut separate = complete(FederalStatus::Separate);
    let mut e = income(cash_business(10000));
    let FederalIncome::Business { data } = &mut e.income else { panic!() };
    data.assets.push(elected_asset(1000, 1000, BonusElection::ElectOut));
    add(&mut separate, e);
    // Spouse records not entered: no section 179 on this separate return,
    // the election depreciated normally, with a warning.
    let r = supported(&separate);
    assert!(noted(&r, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"), "{r:?}");
}

#[test]
fn separate_section179_combines_purchases_before_sharing_limit() {
    for (share, expected) in [
        (Section179Share::Equal, 1175000),
        (Section179Share::Elected { yours: 95.0, spouse: 5.0 }, 2232500),
        (Section179Share::Elected { yours: 0.0, spouse: 100.0 }, 0),
        (Section179Share::Elected { yours: 100.0, spouse: 0.0 }, 2350000),
        (Section179Share::Elected { yours: 33.33, spouse: 66.67 }, 783255),
        (Section179Share::Elected { yours: 95.0, spouse: 10.0 }, 1175000),
    ] {
        let d = separate179(4000000, expected, 300000, share);
        let calc = crate::section179::calculate(&d);
        assert_eq!(calc.shared_limit, 235000000);
        assert_eq!(calc.limit, expected * 100);
        if expected > 0 {
            // 2026-10-23: Reg. 1.179-2(c)(6), (8): combined work does
            // not establish this separate filer's current conduct.
            crate::tests::support::assert::assert_mfs_section179_work_refused(
                &d,
                d.income[1].id,
                TaxSection::SelfEmployment,
            );
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn separate_section179_rejects_excess_election_and_invalid_shares() {
    unsupported(&separate179(4000000, 1175001, 300000, Section179Share::Equal), "Section 179");
    unsupported(&separate179(4000000, 1, 2650000, Section179Share::Equal), "Section 179");
}

#[test]
fn missing_separate_section179_records_depreciate_normally() {
    let missing = |d: &mut FederalDraft| {
        let FilingChoice::Separate { section179, .. } = &mut d.setup.as_mut().unwrap().status
        else {
            panic!()
        };
        *section179 = Section179Spouse::NotSupplied;
    };
    // Missing records take no section 179: the same asset is depreciated as if it
    // elected zero, with the notice.
    let mut d = separate179(1000, 1000, 0, Section179Share::Equal);
    missing(&mut d);
    let r = supported(&d);
    assert!(noted(&r, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"), "{r:?}");
    assert!(r.lines.iter().all(|line| line.id != "section179_deduction" || line.amount == 0));
    let expected = supported(&separate179(1000, 0, 0, Section179Share::Equal));
    assert_eq!(r.tax, expected.tax);
    assert!(!noted(&expected, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"));
    // An earlier carryforward is not deducted and waits for 2027.
    let mut d = separate179(1000, 0, 0, Section179Share::Equal);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(carried_asset(2025, 30000));
    missing(&mut d);
    // 2026-10-23: Reg. 1.179-2(c)(6), (8): retained carry does not
    // establish the separate filer's work from combined participation.
    crate::tests::support::assert::assert_mfs_section179_work_refused(
        &d,
        d.income[1].id,
        TaxSection::SelfEmployment,
    );
    let allocation = crate::section179::calculate(&d);
    assert_eq!(allocation.allowed, 0);
    assert_eq!(allocation.available - allocation.allowed, 3_000_000);
}

#[test]
fn separate_section179_zero_share_preserves_carry_without_spouse_income() {
    let mut d = separate179(1000, 0, 0, Section179Share::Elected { yours: 0.0, spouse: 100.0 });
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(carried_asset(2025, 30000));
    // 2026-10-23: Reg. 1.179-2(c)(6), (8): a zero elected share
    // supplies no missing individual-work evidence.
    crate::tests::support::assert::assert_mfs_section179_work_refused(
        &d,
        d.income[1].id,
        TaxSection::SelfEmployment,
    );
    let allocation = crate::section179::calculate(&d);
    assert_eq!(allocation.allowed, 0);
    assert_eq!(allocation.available - allocation.allowed, 3_000_000);
    let original_income = crate::section179::calculate(&d).income;
    let FilingChoice::Separate { section179, .. } = &mut d.setup.as_mut().unwrap().status else {
        panic!()
    };
    *section179 = Section179Spouse::Records { cost: 5000000, allocation: Section179Share::Equal };
    assert_eq!(crate::section179::calculate(&d).income, original_income);
}

// A return whose business, entered on the self-employment page, elects
// section 179 for a $4,000,000 asset (or elects none).
fn session_electing(status: FederalStatus, elected: i64) -> Interview {
    use crate::tests::cross::interview::{asset, business, save_businesses, with_first_year};
    let mut session = Interview::new(TEST_USER, complete(status));
    let mut b = business(8_800_000);
    b.assets = vec![with_first_year(
        asset(400_000_000, 20260310, AssetClass::Seven),
        FirstYearDeduction::Section179 { description: "Equipment".into(), elected },
    )];
    crate::tests::support::assert::accepted(&save_businesses(&mut session, vec![b]));
    session
}

// The spouse's section 179 records are offered on the self-employment page
// only for a separate return that elects section 179, and never seeded: the
// page holds no record until the person adds one, which starts unanswered.
#[test]
fn spouse_section179_records_are_asked_only_for_a_separate_return_that_elects() {
    use crate::tests::support::client::income_pages::open_topic;
    let kind = libertas_macros::variant_index!(crate::IncomeSourceKind::SpouseSection179) as i32;
    for (mut session, asked) in [
        (session_electing(FederalStatus::Separate, 0), false),
        (session_electing(FederalStatus::Separate, 100_000), true),
        (session_electing(FederalStatus::Single, 100_000), false),
    ] {
        let P::SaveIncomes { documents, allowed_documents, .. } =
            open_topic(&mut session, TaxSection::SelfEmployment)
        else {
            unreachable!()
        };
        assert_eq!(allowed_documents.contains(&kind), asked);
        assert!(
            !documents
                .documents
                .iter()
                .any(|item| matches!(item.document, crate::IncomeRecord::SpouseSection179(_)))
        );
        if asked {
            // Records not given: the rules leave the election out with a notice.
            assert!(matches!(
                session.draft.setup.as_ref().unwrap().status,
                crate::FilingChoice::Separate { section179: Section179Spouse::NotSupplied }
            ));
        }
    }
}

#[test]
fn separate_section179_sources_save_reopen_correct_and_restart() {
    use crate::tests::support::{assert::accepted, client::income_pages::open_topic};
    let mut session = session_electing(FederalStatus::Separate, 100_000);
    // Publication 946, Married Individuals: combine actual purchases before
    // sharing the reduced limit. An invalid total elects the equal allocation.
    // https://www.irs.gov/publications/p946
    for (source, expected_cost, expected_share, expected_limit) in [
        (
            Some(SpouseSection179 { cost: 0, allocation: Section179Share::Equal }),
            0,
            5000,
            128_000_000,
        ),
        (
            Some(SpouseSection179 {
                cost: 30_000_000,
                allocation: Section179Share::Elected { yours: 95.0, spouse: 5.0 },
            }),
            30_000_000,
            9500,
            223_250_000,
        ),
        (
            Some(SpouseSection179 {
                cost: 101,
                allocation: Section179Share::Elected { yours: 95.0, spouse: 10.0 },
            }),
            101,
            5000,
            128_000_000,
        ),
        // Removing the record leaves the records not given.
        (None, 0, 0, 0),
    ] {
        // The person adds or edits the spouse's record on the self-employment
        // page, as the return elects section 179.
        let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let template = documents.documents[0].clone();
        documents
            .documents
            .retain(|item| !matches!(item.document, crate::IncomeRecord::SpouseSection179(_)));
        if let Some(row) = &source {
            documents.documents.push(crate::IncomeRecordItem {
                id: 0,
                investment_caps: None,
                document: crate::IncomeRecord::SpouseSection179(row.clone()),
                ..template
            });
        }
        accepted(&send(&mut session, request));
        session = restart(session.draft.clone());
        let P::SaveIncomes { documents, .. } = open_topic(&mut session, TaxSection::SelfEmployment)
        else {
            unreachable!()
        };
        let reopened = documents.documents.iter().find_map(|item| match &item.document {
            crate::IncomeRecord::SpouseSection179(row) => Some(row.clone()),
            _ => None,
        });
        assert_eq!(reopened, source);
        let allocation = crate::section179::calculate(&session.draft);
        assert_eq!(allocation.spouse_cost, expected_cost);
        assert_eq!(allocation.share, expected_share);
        assert_eq!(allocation.limit, expected_limit);
    }
}
