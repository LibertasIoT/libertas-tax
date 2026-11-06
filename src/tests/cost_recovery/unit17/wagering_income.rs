use super::*;
use crate::tests::cross::interview::{asset, business};
use crate::tests::support::assert::accepted;
use crate::tests::support::client::income_pages::open_topic;

#[test]
fn unit17_wagering_source_and_shared_income_limit_use_combined_expenses_once() {
    let draft = wagering179_session().draft;
    let source = draft.income.last().unwrap().id;
    for (losses, expected) in [(500_000, 460_000), (1_500_000, 0)] {
        let mut selected = draft.clone();
        let FederalIncome::Business { data } = &mut selected.income.last_mut().unwrap().income
        else {
            unreachable!()
        };
        let BusinessActivity::Gambling { losses: amount, .. } = &mut data.activity else {
            unreachable!()
        };
        *amount = losses;
        for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
            let sources = section179_direct_income_sources(&selected, system).unwrap();
            let wagering = sources.iter().find(|row| row.source == source).unwrap();
            assert_eq!(wagering.operating_net_cents, expected);
            assert_eq!(wagering.recognized_property_gain_dollars, 0);
            let prepared = crate::section179_return::prepare(&selected, system).unwrap();
            assert_eq!(prepared.allocation.income.business_income_limit_cents, expected);
            let mut result = FederalResult::empty(selected.revision);
            prepared.report(&mut result).unwrap();
            assert!(result.issues.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn unit17_wagering_selected_asset_cost_enters_ninety_percent_limit_in_each_system() {
    let mut draft = empty_return(FederalStatus::Single);
    let mut native = professional_gambling(50_000, 10_000, 1_000);
    let FederalIncome::Business { data } = &mut native else { unreachable!() };
    data.assets.push(separate_amt_equipment());
    add(&mut draft, income(native));
    let source = draft.income[0].id;
    let mut page = business(5_000_000);
    let mut property = asset(1_000_000, 20170601, AssetClass::Ten);
    property.acquired = Some(20160101);
    property.gds_mut().unwrap().history = AssetHistory::Continuing {
        prior_depreciation: 901_696,
        first_year_179: 0,
        first_year_bonus: 0,
        carryforward: 0,
        mid_quarter: false,
    };
    page.assets = vec![property];
    let draft = save_native_asset_identities(
        draft,
        source,
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
        TaxSection::SelfEmployment,
    );
    for (system, expected) in [
        (Section179IncomeSystem::Regular, 3_951_018),
        (Section179IncomeSystem::Alternative, 3_931_344),
    ] {
        let sources = section179_direct_income_sources(&draft, system).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source, source);
        assert_eq!(sources[0].system, system);
        assert_eq!(sources[0].operating_net_cents, expected);
    }
}

#[test]
fn unit17_current_active_conduct_is_independent_of_historical_and_deemed_participation() {
    let mut draft = wagering179_session().draft;
    let source = draft.income.last().unwrap().id;
    for (hours, test, active, material) in [
        (501, ParticipationTest::Over500, true, true),
        (1, ParticipationTest::SubstantiallyAll, true, true),
        (101, ParticipationTest::Over100NoOneMore { other_max_hours: 100 }, true, true),
        (200, ParticipationTest::SignificantActivities, true, false),
        (
            80,
            ParticipationTest::FactsCircumstances {
                regular_continuous_substantial: true,
                management_hours: 80,
                paid_other_manager: true,
                other_management_hours: 100,
            },
            true,
            false,
        ),
        (
            0,
            ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] },
            false,
            true,
        ),
        (
            200,
            ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] },
            false,
            true,
        ),
        (
            501,
            ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] },
            true,
            true,
        ),
    ] {
        let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.material_participation = ParticipationFacts { hours, test };
        let view = draft.direct_participation_view();
        assert_eq!(view.active_conduct_established(source), active);
        assert_eq!(view.get(source) == crate::participation::Participation::Yes, material);
    }
    let mut farm_draft = empty_return(FederalStatus::Single);
    add(&mut farm_draft, income(raised_farm(10_000, 0)));
    let farm_id = farm_draft.income[0].id;
    for (test, expected) in [
        (
            ParticipationTest::SurvivingFarmer {
                special_estate_valuation: true,
                active_management: true,
            },
            true,
        ),
        (
            ParticipationTest::RetiredDisabledFarmer {
                retired_or_disabled: 2026,
                qualifying_years: vec![2021, 2022, 2023, 2024, 2025],
            },
            false,
        ),
    ] {
        let FederalIncome::Business { data } = &mut farm_draft.income[0].income else {
            unreachable!()
        };
        data.material_participation = ParticipationFacts { hours: 0, test };
        let view = farm_draft.direct_participation_view();
        assert_eq!(view.get(farm_id), crate::participation::Participation::Yes);
        assert_eq!(view.active_conduct_established(farm_id), expected);
    }
    // Keep the same original accepted view through a transformed calculation
    // draft; rebuilding from the transformed facts would lose its evidence.
    let original = draft.direct_participation_view();
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.material_participation = ParticipationFacts { hours: 0, test: ParticipationTest::None };
    let mut home = crate::home_office::HomeContext::default();
    home.bind_participation(&original);
    assert!(!draft.direct_participation_view().active_conduct_established(source));
    let prepared =
        crate::section179_return::prepare_with_home(&draft, Section179IncomeSystem::Regular, &home)
            .unwrap();
    let mut result = FederalResult::empty(draft.revision);
    prepared.report(&mut result).unwrap();
    assert!(result.issues.is_empty(), "{result:?}");
}

#[test]
fn unit17_wagering_unsupported_sources_keep_located_boundaries() {
    for boundary in
        ["own179", "amtcarry", "amtunknown", "property", "historical", "separate", "home"]
    {
        let mut draft = wagering179_session().draft;
        let source = draft.income.last().unwrap().id;
        let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        let key = match boundary {
            "own179" => {
                let mut row = elected_asset(1_000, 100, BonusElection::Apply);
                row.id = draft.next_id;
                data.assets.push(row);
                "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION"
            }
            "amtcarry" | "amtunknown" => {
                let mut row = bonus_equipment(20_000, 100);
                row.id = draft.next_id;
                row.acquired = 20250101;
                row.placed = 20250101;
                row.history = EquipmentHistory::Continuing {
                    initial_section179: 1_000_000,
                    carryforward: 0,
                    prior_depreciation: 1_200_000,
                    amt: EquipmentAmtHistory::Same,
                    mid_quarter: false,
                    schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
                };
                row.amt_section179_income_carry =
                    if boundary == "amtunknown" { None } else { Some(30_000) };
                assert_eq!(row.available179(), 0);
                assert_eq!(
                    row.amt_section179_carry(),
                    if boundary == "amtunknown" { Err(()) } else { Ok(30_000) }
                );
                data.assets.push(row);
                "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION"
            }
            "property" => {
                let mut row = bonus_equipment(1_000, 100);
                row.id = draft.next_id;
                row.disposal =
                    AssetDisposal::Disposed { date: 20261201, how: DisposalKind::Scrapped };
                data.assets.push(row);
                "ISSUE_SECTION179_WAGERING_PROPERTY"
            }
            "historical" => {
                data.material_participation = ParticipationFacts {
                    hours: 0,
                    test: ParticipationTest::FiveOfTen {
                        years: vec![2021, 2022, 2023, 2024, 2025],
                    },
                };
                "ISSUE_SECTION179_WAGERING_CURRENT_WORK"
            }
            "separate" => {
                draft.setup = complete(FederalStatus::Separate).setup;
                "ISSUE_SECTION179_SEPARATE_OWN_WORK"
            }
            "home" => {
                add_simplified_office(&mut draft, source, 100);
                "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION"
            }
            _ => unreachable!(),
        };
        for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
            let issue = section179_direct_income_sources(&draft, system).unwrap_err();
            if boundary == "amtunknown" && system == Section179IncomeSystem::Alternative {
                assert!(matches!(issue,
                    Section179PreparedIssue::AlternativeAssetCarryUnavailable(id, _) if id == source));
                continue;
            }
            assert_eq!(
                issue,
                Section179PreparedIssue::SourceBoundaryUnavailable(source, key),
                "{boundary}"
            );
            let mut result = FederalResult::empty(draft.revision);
            crate::section179_return::Issue::Prepared(issue).report(&mut result);
            assert!(result.issues[0].display.key == key);
            assert!(matches!(result.issues[0].context.source(),
                crate::IssueContext::Income { id, .. } if *id == source));
        }
    }
}

#[test]
fn unit17_wagering_actual_save_correction_reopen_and_restart() {
    let mut session = wagering179_session();
    let original = session.draft.clone();
    let result = supported(&original);
    assert_eq!(line_amount(&result, "section179_business_income_limit"), 460_000);
    assert_eq!(line_amount(&result, "section179_deduction"), 460_000);
    let reopened = open_topic(&mut session, TaxSection::SelfEmployment);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, original);
    session = restart(session.draft);
    let mut correction = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let IncomeRecord::Business(page) = &mut documents.documents[1].document else { unreachable!() };
    page.more.gambling.as_mut().unwrap().losses = 900_000;
    accepted(&send(&mut session, correction));
    let corrected = supported(&session.draft);
    assert_eq!(line_amount(&corrected, "section179_business_income_limit"), 100_000);
    assert_eq!(line_amount(&corrected, "section179_deduction"), 100_000);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
    let reopen = open_topic(&mut session, TaxSection::SelfEmployment);
    accepted(&send(&mut session, reopen));
}

#[test]
fn unit17_direct_income_requires_current_conduct_and_native_pal_certification() {
    for farming in [false, true] {
        for (hours, test, key) in [
            (
                0,
                ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] },
                "ISSUE_SECTION179_DIRECT_CURRENT_WORK",
            ),
            (
                0,
                ParticipationTest::PersonalServiceThree {
                    years: vec![2023, 2024, 2025],
                    personal_services: true,
                },
                "ISSUE_SECTION179_DIRECT_CURRENT_WORK",
            ),
            (
                80,
                ParticipationTest::FactsCircumstances {
                    regular_continuous_substantial: true,
                    management_hours: 80,
                    paid_other_manager: true,
                    other_management_hours: 100,
                },
                "ISSUE_SECTION179_DIRECT_PAL_INCOME",
            ),
        ] {
            let mut draft = wagering179_session().draft;
            let source = draft.income[0].id;
            let FederalIncome::Business { data } = &mut draft.income[0].income else {
                unreachable!()
            };
            if farming {
                let FederalIncome::Business { data: farm } = raised_farm(0, 0) else {
                    unreachable!()
                };
                data.activity = farm.activity;
            }
            data.material_participation = ParticipationFacts { hours, test };
            for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
                // Raw operating operands remain available; they do not certify
                // current conduct or a passive source's allowed deductions.
                assert!(section179_direct_income_sources(&draft, system).is_ok());
                let issue = match crate::section179_return::prepare(&draft, system) {
                    Err(crate::section179_return::Issue::Prepared(issue)) => issue,
                    Err(issue) => panic!("{issue:?}"),
                    Ok(_) => panic!("uncertified income was accepted"),
                };
                assert_eq!(issue, Section179PreparedIssue::SourceBoundaryUnavailable(source, key));
                let mut located = FederalResult::empty(draft.revision);
                crate::section179_return::Issue::Prepared(issue).report(&mut located);
                assert!(matches!(located.issues[0].context.source(),
                    crate::IssueContext::Income { id, .. } if *id == source));
            }
            let mut report = FederalResult::empty(draft.revision);
            crate::section179::report(&draft, &mut report);
            assert!(report.issues.iter().any(|issue| issue.display.key == key
                && matches!(issue.context.source(), crate::IssueContext::Income { id, .. } if *id == source)));
        }
    }
}

#[test]
fn unit17_farm_deemed_participation_and_mfs_do_not_establish_current_conduct() {
    for separate in [false, true] {
        let mut draft = wagering179_session().draft;
        if separate {
            draft.income.pop();
        }
        let source = draft.income[0].id;
        let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
        if separate {
            draft.setup = complete(FederalStatus::Separate).setup;
        } else {
            let FederalIncome::Business { data: farm } = raised_farm(0, 0) else { unreachable!() };
            data.activity = farm.activity;
            data.material_participation = ParticipationFacts {
                hours: 0,
                test: ParticipationTest::RetiredDisabledFarmer {
                    retired_or_disabled: 2026,
                    qualifying_years: vec![2021, 2022, 2023, 2024, 2025],
                },
            };
        }
        let view = draft.direct_participation_view();
        assert_eq!(view.get(source), crate::participation::Participation::Yes);
        assert!(!view.active_conduct_established(source));
        let issue = match crate::section179_return::prepare(&draft, Section179IncomeSystem::Regular)
        {
            Err(crate::section179_return::Issue::Prepared(issue)) => issue,
            Err(issue) => panic!("{issue:?}"),
            Ok(_) => panic!("unestablished current conduct was accepted"),
        };
        assert_eq!(
            issue,
            Section179PreparedIssue::SourceBoundaryUnavailable(
                source,
                if separate {
                    "ISSUE_SECTION179_SEPARATE_OWN_WORK"
                } else {
                    "ISSUE_SECTION179_DIRECT_CURRENT_WORK"
                }
            )
        );
    }
}

#[test]
fn unit17_direct_current_conduct_native_correction_reopen_and_restart() {
    let mut session = wagering179_session();
    let source = session.draft.income[0].id;
    for (hours, test, supported_source) in [
        (0, ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] }, false),
        (501, ParticipationTest::Over500, true),
    ] {
        let mut correction = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
        let IncomeRecord::Business(page) = &mut documents.documents[0].document else {
            unreachable!()
        };
        page.more.material_participation = ParticipationFacts { hours, test };
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft.income[0].id, source);
        let saved = session.draft.clone();
        let result = estimate(&saved);
        if supported_source {
            assert_eq!(
                line_amount(&supported(&saved), "section179_business_income_limit"),
                460_000
            );
        } else {
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_SECTION179_DIRECT_CURRENT_WORK")
            );
        }
        session = restart(saved.clone());
        assert_eq!(estimate(&session.draft), result);
        let reopen = open_topic(&mut session, TaxSection::SelfEmployment);
        accepted(&send(&mut session, reopen));
        assert_eq!(session.draft, saved);
    }
}

#[test]
fn unit17_current_farm_management_keeps_selected_income_and_allowance() {
    let mut draft = wagering179_session().draft;
    let source = draft.income[0].id;
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let FederalIncome::Business { data: farm } = raised_farm(0, 0) else { unreachable!() };
    data.activity = farm.activity;
    data.material_participation = ParticipationFacts {
        hours: 0,
        test: ParticipationTest::SurvivingFarmer {
            special_estate_valuation: true,
            active_management: true,
        },
    };
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let prepared = crate::section179_return::prepare(&draft, system).unwrap();
        assert_eq!(prepared.allocation.income.business_income_limit_cents, 460_000);
        assert_eq!(prepared.allocation.allowed(source).unwrap(), 460_000);
        let mut report = FederalResult::empty(draft.revision);
        prepared.report(&mut report).unwrap();
        assert!(report.issues.is_empty(), "{report:?}");
    }
}
