pub(in crate::tests) mod additional_medicare_contributors {
    use crate::tests::support::builders::additional_medicare::{Source, fixture};
    use crate::tests::support::client::injured_income::additional_medicare_contributors::persisted;
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, source: Source) -> FederalResult {
        crate::tests::support::assert::payroll::injured_additional_medicare_money(d, source, false)
    }

    pub(in crate::tests) fn check(source: Source) {
        let mut observed = vec![];
        let mut expected = vec![];
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    let d = persisted(fixture(source, owner, injured, reverse), source);
                    let r = money(&d, source);
                    let mut taxes = [380, 380];
                    let holder = usize::from(owner == Owner::Spouse);
                    let inj = usize::from(injured == Owner::Spouse);
                    match source {
                        Source::PartnerSe => taxes[holder] += 14130 + 381,
                        Source::SmallBusiness => taxes[1 - holder] += 450,
                        Source::Railroad => taxes[holder] += 450,
                    }
                    observed.push((
                        owner,
                        injured,
                        reverse,
                        ["a", "b", "c"].map(|col| {
                            r.lines
                                .iter()
                                .find(|l| l.id == alloc::format!("form_8379_line_18_{col}"))
                                .map(|l| l.amount / 100)
                        }),
                    ));
                    expected.push((
                        owner,
                        injured,
                        reverse,
                        [Some(taxes.iter().sum::<i64>()), Some(taxes[inj]), Some(taxes[1 - inj])],
                    ));
                }
            }
        }
        assert_eq!(
            observed, expected,
            "all independent native money/publicSave/records/restart cases passed first: {source:?}"
        );
    }
}

pub(in crate::tests) mod alimony_recapture_sources {
    use crate::tests::cross::interview::{accepted, from_review};
    use crate::tests::support::builders::injured_income::alimony_recapture_sources::Case;
    use crate::tests::*;
    pub(in crate::tests) fn check(case: Case) {
        for recipient in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let mut claim = allocation();
                    claim.injured = injured;
                    let mut d = injured_return((120_000, 25_000), (40_000, 5_000), 0, claim);
                    let mut received = recapture_alimony();
                    let (recapture, paid, deduction, agi, taxable, tax) = match case {
                        // Pub504 Worksheet1: second excess24000, first27500.
                        Case::Stopped => {
                            received.paid = 0;
                            received.alimony_due = 0;
                            (51_500, 0, 51_500, 108_500, 76_300, 8663)
                        }
                        // Original Pub504 example: current28000, recap1500;
                        // opposite spouse separately paid2000 under own decree.
                        Case::CurrentAndPaid => (1500, 200_000, 3500, 184_500, 152_300, 22_930),
                        // Native recapture rounds each worksheet input before
                        // computing its1501 annual Dollars. The category adds
                        // actual paid cents once, and rounds that joint sum once.
                        Case::Paid49Cents | Case::Paid50Cents => {
                            let AlimonyRecapture::ThirdYear { first, .. } = &mut received.recapture
                            else {
                                unreachable!()
                            };
                            *first += 50;
                            if matches!(case, Case::Paid49Cents) {
                                (1501, 49, 1501, 186_499, 154_299, 23_370)
                            } else {
                                (1501, 50, 1502, 186_498, 154_298, 23_370)
                            }
                        }
                    };
                    let mut entry = income(FederalIncome::Alimony { data: received });
                    entry.owner = crate::DocumentOwner::from(recipient);
                    add(&mut d, entry);
                    let payer =
                        if recipient == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
                    if paid > 0 {
                        d.adjustments.as_mut().unwrap().alimony_paid.push(AlimonyPaid {
                            owner: payer,
                            payment: AlimonyPayment {
                                paid,
                                alimony_due: paid,
                                child_support_due: 0,
                                ..alimony_record(20181231)
                            },
                        });
                    }
                    if reversed {
                        d.income.reverse();
                    }
                    let ordinary = supported(&d);
                    // Ordinary joint figures are the independent oracle; source
                    // attribution must leave every one of them unchanged.
                    assert_eq!(value(&ordinary, "alimony_recapture_deduction"), recapture);
                    assert_eq!(return_line(&ordinary, "10"), deduction * 100);
                    assert_eq!(value(&ordinary, "agi"), agi);
                    assert_eq!(value(&ordinary, "taxable_income"), taxable);
                    assert_eq!(ordinary.tax, Some(tax * 100));
                    assert_eq!(ordinary.refund, Some((30_000 - tax) * 100));
                    let mut session = Interview::new(TEST_USER, d);
                    let request = match from_review(&mut session, TaxSection::OtherIncome) {
                        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
                        _ => panic!("expected normal received-alimony editor"),
                    };
                    let bytes = request.to_avro();
                    let request = P::from_avro(&bytes).unwrap();
                    assert_eq!(request.to_avro(), bytes);
                    accepted(&send(&mut session, request));
                    if paid > 0 {
                        let request = match from_review(&mut session, TaxSection::MoreAdjustments) {
                            Reply::Edit(request @ P::SaveMoreAdjustments { .. }) => request,
                            _ => panic!("expected normal paid-alimony editor"),
                        };
                        accepted(&send(&mut session, request));
                    }
                    let saved = session.draft.clone();
                    let stored = native_draft_records(&saved);
                    let draft = draft_from_native_records(saved.tax_year, &stored);
                    assert_eq!(native_draft_records(&draft), stored);
                    assert_eq!(draft, saved);
                    let mut again = restart(draft);
                    let request = match from_review(&mut again, TaxSection::OtherIncome) {
                        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
                        _ => panic!("expected reopened received-alimony editor"),
                    };
                    accepted(&send(&mut again, request));
                    assert_eq!(again.draft, saved);
                    let r = supported(&again.draft);
                    assert_eq!(r.return_lines, ordinary.return_lines);
                    assert_eq!(r.tax, ordinary.tax);
                    assert_eq!(r.refund, ordinary.refund);
                    let mut owners = [0; 2];
                    let index = usize::from(recipient == Owner::Spouse);
                    owners[index] = recapture;
                    owners[1 - index] = deduction - recapture;
                    assert_eq!(column(&r, "14", "a"), deduction);
                    assert_eq!(
                        column(&r, "14", "b"),
                        owners[usize::from(injured == Owner::Spouse)]
                    );
                    assert_eq!(
                        column(&r, "14", "c"),
                        owners[usize::from(injured != Owner::Spouse)]
                    );
                    assert!(
                        !r.review_notes
                            .iter()
                            .any(|n| n.display.key == "WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE")
                    );
                }
            }
        }
    }
}

pub(in crate::tests) mod bonds_cooperative_sources {
    use crate::tests::investment::investment::{bond_record, bond_student};
    use crate::tests::support::assert::injured_ordinary_money_unchanged;
    use crate::tests::support::builders::injured_income::bonds_cooperative_sources::Route;
    use crate::tests::*;
    pub(in crate::tests) fn ordinary_unchanged(draft: &FederalDraft, result: &FederalResult) {
        let mut plain = draft.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(result.return_lines, ordinary.return_lines);
        injured_ordinary_money_unchanged(result, &ordinary);
    }

    pub(in crate::tests) fn observed_owners(result: &FederalResult) -> [Option<i64>; 3] {
        ["a", "b", "c"].map(|column| {
            result
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("form_8379_line_13b_{column}"))
                .map(|line| line.amount / 100)
        })
    }

    pub(in crate::tests) fn check_states(
        mut draft: FederalDraft,
        owners: [i64; 2],
        ordinary: impl Fn(&FederalDraft, &FederalResult),
    ) {
        let mut actual = vec![];
        let mut expected = vec![];
        for reversed in [false, true] {
            if reversed {
                draft.income.reverse();
            }
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                let result = supported(&draft);
                ordinary(&draft, &result);
                ordinary_unchanged(&draft, &result);
                let restored = restart(draft.clone());
                assert_eq!(restored.draft, draft);
                assert_eq!(estimate(&restored.draft), result);
                actual.push((reversed, injured, observed_owners(&result)));
                let pair = if injured == Owner::Taxpayer { owners } else { [owners[1], owners[0]] };
                expected.push((
                    reversed,
                    injured,
                    [Some(owners.iter().sum()), Some(pair[0]), Some(pair[1])],
                ));
            }
        }
        assert_eq!(
            actual, expected,
            "all ordinary-source/reorder/selected-spouse/native-restart controls passed first"
        );
    }

    pub(in crate::tests) fn bonds(
        gross: i64,
        prior: i64,
        proceeds: i64,
        tuition: i64,
        excluded: i64,
    ) {
        let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
        add(&mut draft, income(savings_income(1000, 0)));
        let mut linked = income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                us_obligations: gross * 100,
                adjustments: (prior != 0).then_some(InterestAdjustments {
                    previously_reported: prior * 100,
                    ..Default::default()
                }),
                special: InterestSpecial::NoneOfThese,
                ..Default::default()
            }),
        });
        linked.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, linked);
        let linked_id = draft.income.last().unwrap().id;
        // Actual source recipient is spouse; the education beneficiary is taxpayer.
        // The exclusion follows the linked bond interest, never the student's owner.
        let mut record = bond_record(
            Owner::Spouse,
            proceeds,
            gross,
            20100601,
            vec![bond_student(-1, tuition, 0)],
        );
        let FederalIncome::SavingsBonds { data } = &mut record.income else { unreachable!() };
        data.answers.as_mut().unwrap().interest_statement = linked_id;
        add(&mut draft, record);
        assert_eq!(crate::savings_bonds::source_rule(&draft.income), None);
        let sources = crate::savings_bonds::sources(&draft.income);
        let source = sources.iter().find(|s| s.id == linked_id).unwrap();
        assert_eq!(
            (source.owner, source.gross, source.prior),
            (Owner::Spouse, gross * 100, prior * 100)
        );
        let remaining = gross - prior - excluded;
        check_states(draft, [1000, remaining], |draft, result| {
            assert_eq!(value(result, "form_8815_proceeds"), proceeds);
            assert_eq!(value(result, "form_8815_interest"), gross - prior);
            assert_eq!(value(result, "savings_bond_exclusion"), excluded);
            assert_eq!(return_line(result, "2b"), (1000 + remaining) * 100);
            assert_eq!(value(result, "agi"), 81_000 + remaining);
            assert_eq!(crate::savings_bonds::source_rule(&draft.income), None);
            let (record, data) = crate::savings_bonds::record(&draft.income).unwrap();
            assert_eq!((record.owner.owned(), data.interest_statement), (Owner::Spouse, linked_id));
        });
    }

    pub(in crate::tests) fn cooperative(route: Route) {
        let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
        let activity = match route {
            Route::Business => cash_business(10_000),
            Route::Farm => raised_farm(11_000, 1000),
            Route::FarmRental => {
                let FederalIncome::Business { data } = raised_farm(11_000, 1000) else {
                    unreachable!()
                };
                let BusinessActivity::Farming { data: mut farm } = data.activity else {
                    unreachable!()
                };
                farm.header = None;
                FederalIncome::FarmRental {
                    data: FederalFarmRental {
                        improvement_buildings: Vec::new(),
                        interest_limit_election: None,
                        farm,
                        expenses: data.expenses,
                        assets: vec![],
                        vehicles: vec![],
                        active_participation: true,
                        at_risk: Some(crate::RentalAtRisk::All),
                        history: RentalHistory::NONE,
                        qbi_business: RentalBusiness::No,
                        qbi_aggregation_group: None,
                        payroll: BusinessPayroll::No,
                    },
                }
            }
        };
        let mut activity = income(activity);
        activity.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, activity);
        let activity_id = draft.income.last().unwrap().id;
        let mut data = FederalCooperative { patronage: 200_000, ..Default::default() };
        let mut allocation = crate::cooperative::automatic(&data);
        allocation.income_use = if matches!(route, Route::Business) {
            CoopIncomeUse::DeductiblePurchases
        } else {
            CoopIncomeUse::ProductionSales
        };
        data.allocations.push(allocation);
        let mut entry = income(FederalIncome::Cooperative { data: cooperative_source(data) });
        entry.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, entry);
        claim_last_cooperative(&mut draft, activity_id);
        let parent = draft.income.last().unwrap().id;
        assert_eq!(crate::cooperative::graph_rule(&draft), None);
        check_states(draft, [0, 12_000], |draft, result| {
            let sources = crate::cooperative::sources(draft);
            assert_eq!(sources.len(), 1);
            assert_eq!(
                (sources[0].parent, sources[0].owner, sources[0].activity),
                (parent, Owner::Spouse, Some(activity_id))
            );
            assert_eq!(value(result, &alloc::format!("cooperative_{parent}_taxable")), 2000);
            match route {
                Route::Business => {
                    assert_eq!(value(result, "schedule_1_business_income"), 12_000);
                }
                Route::Farm => {
                    assert_eq!(value(result, "schedule_1_farm_income"), 12_000);
                }
                Route::FarmRental => {
                    assert_eq!(value(result, "schedule_e_farm_rental_total"), 12_000);
                    assert_eq!(value(result, "self_employment_tax"), 0);
                }
            }
            // Native empty opposite C/F categories need not emit a row. Line8
            // verifies the entire Schedule1 income, not just the chosen activity.
            assert_eq!(return_line(result, "8"), 1_200_000);
            assert_eq!(return_line(result, "9"), 9_200_000);
            if matches!(route, Route::FarmRental) {
                assert_eq!(value(result, "agi"), 92_000);
            } else {
                assert_eq!(value(result, "self_employment_tax"), 1696);
                // The only adjustment is the native spouse half-SE deduction;
                // its aggregate is1040line10, not an unprefixed per-person row.
                assert_eq!(return_line(result, "10"), 84_800);
                assert_eq!(value(result, "agi"), 91_152);
            }
        });
    }
}

pub(in crate::tests) mod estate_estimated_payment_sources {
    use crate::tests::*;
    pub(in crate::tests) fn financial(d: &FederalDraft, dated: bool) -> FederalResult {
        let r = supported(d);
        assert_eq!(value(&r, "agi"), 50_000);
        assert_eq!(value(&r, "taxable_income"), 17_800);
        assert_eq!(return_line(&r, "16"), 178_300);
        assert_eq!(return_line(&r, "26"), if dated { 100_100 } else { 100_000 });
        assert_eq!(r.tax, Some(178_300));
        assert_eq!(r.refund, Some(if dated { 121_800 } else { 121_700 }));
        r
    }
}

pub(in crate::tests) mod home_employee_niit_sources {
    use crate::tests::support::assert::payroll::injured_home_employee_money as money;
    use crate::tests::support::builders::home_employee::Source;
    use crate::tests::support::builders::injured_income::home_employee_niit_sources::fixture;
    use crate::tests::support::client::injured_income::home_employee_niit_sources::persisted;
    use crate::tests::*;
    pub(in crate::tests) fn check(source: Source) {
        let mut observed = vec![];
        let mut expected = vec![];
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let d = persisted(fixture(source, owner, injured, reversed), source, owner);
                    let r = money(&d, source, owner);
                    if matches!(source, Source::UncoveredEmployee) {
                        assert_eq!(r.tax, None);
                        assert!(
                            !r.lines.iter().any(|line| line.id.starts_with("form_8379_line_18_"))
                        );
                        continue;
                    }
                    let mut other = [10000, 10000];
                    let owner_index = usize::from(owner == Owner::Spouse);
                    let inj = usize::from(injured == Owner::Spouse);
                    if matches!(source, Source::TaxableHome) {
                        other[owner_index] += 10000;
                    }
                    if !source.home() {
                        other[owner_index] +=
                            if matches!(source, Source::OppositeEmployee) { 160000 } else { 5000 };
                    }
                    let wages = if matches!(source, Source::OppositeEmployee) {
                        if owner == Owner::Taxpayer { [0, 100000] } else { [100000, 0] }
                    } else {
                        [200000, 50000]
                    };
                    assert_eq!(column(&r, "13a", "a"), wages.iter().sum::<i64>());
                    assert_eq!(column(&r, "13a", "b"), wages[inj]);
                    assert_eq!(column(&r, "13a", "c"), wages[1 - inj]);
                    assert_eq!(column(&r, "13b", "a"), other.iter().sum::<i64>());
                    assert_eq!(column(&r, "13b", "b"), other[inj]);
                    assert_eq!(column(&r, "13b", "c"), other[1 - inj]);
                    let mut taxes = [380, 380];
                    if matches!(source, Source::TaxableHome) {
                        taxes[owner_index] += 380;
                    }
                    if matches!(source, Source::CoveredEmployee | Source::OppositeEmployee) {
                        taxes[0] +=
                            if matches!(source, Source::OppositeEmployee) { 54 } else { 27 };
                        taxes[1] +=
                            if matches!(source, Source::OppositeEmployee) { 36 } else { 18 };
                        taxes[owner_index] += if matches!(source, Source::OppositeEmployee) {
                            12240
                        } else if owner == Owner::Taxpayer {
                            73
                        } else {
                            383
                        };
                    }
                    observed.push((
                        owner,
                        injured,
                        reversed,
                        ["a", "b", "c"].map(|col| {
                            r.lines
                                .iter()
                                .find(|line| line.id == alloc::format!("form_8379_line_18_{col}"))
                                .map(|line| line.amount / 100)
                        }),
                    ));
                    expected.push((
                        owner,
                        injured,
                        reversed,
                        [Some(taxes.iter().sum()), Some(taxes[inj]), Some(taxes[1 - inj])],
                    ));
                }
            }
        }
        assert_eq!(
            observed, expected,
            "all joint ordinary/native8919+8959/publicSave/restart/source-column controls passed first: {source:?}"
        );
    }
}

pub(in crate::tests) mod home_payroll_actual_clients {
    use crate::tests::support::assert::payroll::injured_home_employee_money as money;
    use crate::tests::support::builders::home_employee::Source;
    use crate::tests::*;
    pub(in crate::tests) fn selected_amount(source: Source, injured: Owner) -> Option<i64> {
        match (source, injured) {
            (Source::CoveredEmployee, Owner::Taxpayer) => Some(2700),
            (Source::CoveredEmployee, Owner::Spouse) => Some(1800),
            (Source::OppositeEmployee, Owner::Taxpayer) => Some(5400),
            (Source::OppositeEmployee, Owner::Spouse) => Some(3600),
            _ => None,
        }
    }

    pub(in crate::tests) fn checked(
        d: &FederalDraft,
        source: Source,
        owner: Owner,
        injured: Owner,
        answered: bool,
    ) -> FederalResult {
        let r = money(d, source, owner);
        if matches!(source, Source::UncoveredEmployee) {
            assert_eq!(r.tax, None);
            assert!(!r.lines.iter().any(|line| line.id.starts_with("form_8379_line_18_")));
            return r;
        }
        assert_eq!(d.injured_spouse.as_ref().unwrap().injured, injured);
        let index = usize::from(owner == Owner::Spouse);
        let inj = usize::from(injured == Owner::Spouse);
        let mut other = [10000, 10000];
        if matches!(source, Source::TaxableHome) {
            other[index] += 10000;
        }
        if !source.home() {
            other[index] += if matches!(source, Source::OppositeEmployee) { 160000 } else { 5000 };
        }
        let wages = if matches!(source, Source::OppositeEmployee) {
            if owner == Owner::Taxpayer { [0, 100000] } else { [100000, 0] }
        } else {
            [200000, 50000]
        };
        assert_eq!([column(&r, "13a", "b"), column(&r, "13a", "c")], [wages[inj], wages[1 - inj]]);
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [other[inj], other[1 - inj]]);
        let expected = selected_amount(source, injured);
        let rows = &d.injured_spouse.as_ref().unwrap().allocations;
        assert_eq!(
            rows.iter()
                .find(|row| row.component == "additional_medicare_tax")
                .map(|row| row.injured),
            if answered { expected } else { None }
        );
        let required = &r.injured_spouse.as_ref().unwrap().required;
        if let Some(amount) = expected {
            let row =
                required.iter().find(|row| row.component == "additional_medicare_tax").unwrap();
            let joint = if matches!(source, Source::OppositeEmployee) { 9000 } else { 4500 };
            assert_eq!(row.joint, joint);
            assert_eq!((row.minimum, row.maximum), (0, joint));
            assert!((row.minimum..=row.maximum).contains(&amount));
            if !answered {
                assert!(!r.lines.iter().any(|line| matches!(
                    line.id.as_str(),
                    "form_8379_line_18_b" | "form_8379_line_18_c"
                )));
                return r;
            }
        } else {
            assert!(required.is_empty());
        }
        let mut taxes = [380, 380];
        if matches!(source, Source::TaxableHome) {
            taxes[index] += 380;
        }
        if expected.is_some() {
            let pair = if matches!(source, Source::OppositeEmployee) { [54, 36] } else { [27, 18] };
            taxes[0] += pair[0];
            taxes[1] += pair[1];
            taxes[index] += if matches!(source, Source::OppositeEmployee) {
                12240
            } else if owner == Owner::Taxpayer {
                73
            } else {
                383
            };
        }
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [taxes[inj], taxes[1 - inj]]);
        r
    }
}

pub(in crate::tests) mod k1_activity_niit_sources {
    use crate::tests::cross::interview::{accepted, open_topic};
    use crate::tests::support::builders::injured_income::k1_activity_niit_sources::{
        Activity, Source, fixture,
    };
    use crate::tests::*;
    pub(in crate::tests) fn native_money(
        result: &FederalResult,
        source: Source,
        activity: Activity,
        id: i64,
    ) {
        let released = matches!(activity, Activity::RentalCarry);
        let ordinary = matches!(activity, Activity::Active | Activity::Passive);
        let net = if released { 7000 } else { 10_000 };
        let nii = if matches!(activity, Activity::Active) { 5000 } else { 5000 + net };
        let niit = match activity {
            Activity::Active => 190,
            Activity::RentalCarry => 456,
            _ => 570,
        };
        assert_eq!(return_line(result, "1a"), 25_000_000);
        assert_eq!(return_line(result, "2b"), 500_000);
        assert_eq!(return_line(result, "3b"), 0);
        assert_eq!(return_line(result, "7a"), 0);
        assert_eq!(return_line(result, "8"), net * 100);
        assert_eq!(return_line(result, "9"), (255_000 + net) * 100);
        assert_eq!(value(result, "agi"), 255_000 + net);
        assert_eq!(value(result, "form_8960_net_investment_income"), nii);
        assert_eq!(value(result, "net_investment_income_tax"), niit);
        assert_eq!(value(result, "self_employment_tax"), 0);
        assert_eq!(value(result, "additional_medicare_tax"), 0);
        let qbi = if ordinary { 2000 } else { 0 };
        assert_eq!(value(result, "qbi_deduction"), qbi);
        assert_eq!(value(result, "taxable_income"), 255_000 + net - 32_200 - qbi);
        // MFJ brackets:2480+9120+24332, then24% above211400.
        // Active/passive ordinary statement QBI2000 gives230800 taxable;
        // rental current232800, prior-release229800. All are above theTaxTable.
        let income_tax = if ordinary {
            40_588
        } else if released {
            40_348
        } else {
            41_068
        };
        assert_eq!(return_line(result, "16"), income_tax * 100);
        assert_eq!(result.tax, Some((income_tax + niit) * 100));
        assert_eq!(result.refund, Some((130_000 - income_tax - niit) * 100));
        if !ordinary && !matches!(source, Source::Rental) {
            let prefix = if matches!(source, Source::Scorp) { "scorp" } else { "partnership" };
            for system in ["", "amt_"] {
                assert_eq!(
                    value(result, &alloc::format!("{system}{prefix}_{id}_rental_real_estate_net")),
                    net
                );
                assert_eq!(
                    value(
                        result,
                        &alloc::format!("{system}{prefix}_{id}_rental_real_estate_suspended")
                    ),
                    0
                );
            }
        }
    }

    pub(in crate::tests) fn actual_column(
        result: &FederalResult,
        line: &str,
        column: &str,
    ) -> Option<i64> {
        result
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("form_8379_line_{line}_{column}"))
            .map(|row| row.amount / 100)
    }

    pub(in crate::tests) fn check(source: Source, activity: Activity, owner: Owner) {
        let (original, id) = fixture(source, activity, owner);
        let net = if matches!(activity, Activity::RentalCarry) { 7000 } else { 10_000 };
        let source_niit = match activity {
            Activity::Active => 0,
            Activity::RentalCarry => 266,
            _ => 380,
        };
        let mut actual = vec![];
        let mut expected = vec![];
        for reversed in [false, true] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                let mut draft = original.clone();
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                if reversed {
                    draft.income.reverse();
                }
                let result = supported(&draft);
                native_money(&result, source, activity, id);
                let mut plain = draft.clone();
                plain.injured_spouse = None;
                let ordinary = supported(&plain);
                assert_eq!(result.return_lines, ordinary.return_lines);
                assert_eq!(
                    (result.tax, result.refund, result.balance),
                    (ordinary.tax, ordinary.refund, ordinary.balance)
                );
                assert_eq!(
                    result
                        .lines
                        .iter()
                        .filter(|row| !row.id.starts_with("form_8379_"))
                        .collect::<Vec<_>>(),
                    ordinary
                        .lines
                        .iter()
                        .filter(|row| !row.id.starts_with("form_8379_"))
                        .collect::<Vec<_>>()
                );
                let open = |session: &mut Interview| {
                    if matches!(source, Source::Rental) {
                        open_topic(session, TaxSection::Rentals)
                    } else {
                        crate::tests::support::client::income_pages::open_document_center(session)
                    }
                };
                let mut session = Interview::new(TEST_USER, draft.clone());
                crate::tests::support::client::admit_native_fixture_facts(&mut session);
                let request =
                    crate::tests::support::client::income_pages::open_document_center(&mut session);
                accepted(&send(&mut session, request));
                let request = open(&mut session);
                accepted(&send(&mut session, request));
                assert_eq!(session.draft.income, draft.income);
                let draft = session.draft.clone();
                let accepted_result = supported(&draft);
                assert_native_money_unchanged(&accepted_result, &result);
                assert_eq!(accepted_result.return_lines, result.return_lines);
                let result = accepted_result;
                let request = open(&mut session);
                accepted(&send(&mut session, request));
                assert_eq!(session.draft, draft);
                assert_eq!(estimate(&session.draft), result);
                let stored = native_draft_records(&session.draft);
                let saved = draft_from_native_records(session.draft.tax_year, &stored);
                assert_eq!(native_draft_records(&saved), stored);
                assert_eq!(saved, draft);
                let mut restarted = restart(saved);
                assert_eq!(estimate(&restarted.draft), result);
                let reopened = open(&mut restarted);
                accepted(&send(&mut restarted, reopened));
                assert_eq!(restarted.draft, draft);
                assert_eq!(estimate(&restarted.draft), result);
                // The prior remains its actual accepted source fact after Save;
                // the current native net already consumes its allowed3000 once.
                assert_eq!(
                    restarted.draft.income.iter().find(|entry| entry.id == id).unwrap(),
                    draft.income.iter().find(|entry| entry.id == id).unwrap()
                );
                let held = injured == owner;
                for (line, shares) in [
                    ("13b", if held { [net, 5000] } else { [5000, net] }),
                    ("18", if held { [source_niit, 190] } else { [190, source_niit] }),
                ] {
                    actual.push((
                        reversed,
                        injured,
                        line,
                        ["a", "b", "c"].map(|col| actual_column(&result, line, col)),
                    ));
                    expected.push((
                        reversed,
                        injured,
                        line,
                        [Some(shares[0] + shares[1]), Some(shares[0]), Some(shares[1])],
                    ));
                }
            }
        }
        assert_eq!(actual, expected, "{source:?}/{activity:?}, actual owner {owner:?}");
    }
}

pub(in crate::tests) mod ltc_cent_sources {
    use crate::tests::*;
    pub(in crate::tests) fn ordinary(result: &FederalResult) {
        assert_eq!(value(result, "agi"), 30_001);
        assert_eq!(value(result, "taxable_income"), 0);
        assert_eq!((result.tax, result.refund, result.balance), (Some(0), Some(10_000), Some(0)));
        assert_eq!(value(result, "taxable_ltc_benefits"), 1);
        assert_eq!(column(result, "13b", "a"), 1);
        for period in [1, 2] {
            for (suffix, cents) in
                [("received", 43_049), ("limit", 43_000), ("excluded", 43_000), ("taxable", 49)]
            {
                let key = alloc::format!("ltc_benefit_3_{period}_{suffix}");
                assert_eq!(result.lines.iter().find(|line| line.id == key).unwrap().amount, cents);
            }
        }
    }
}

pub(in crate::tests) mod ltc_cent_actual_clients {
    use crate::tests::support::builders::injured_income::ltc_cent_actual_clients::{
        Family, source_rows,
    };
    use crate::tests::*;
    pub(in crate::tests) fn financial(d: &FederalDraft, family: Family) -> FederalResult {
        let r = supported(d);
        if family == Family::Zero {
            assert_eq!(value(&r, "agi"), 30_000);
            assert_eq!(value(&r, "taxable_income"), 0);
            assert_eq!(value(&r, "taxable_ltc_benefits"), 0);
            let (group, _) = source_rows(d);
            assert_eq!(
                r.lines
                    .iter()
                    .find(|row| row.id == alloc::format!("ltc_benefit_{group}_1_taxable"))
                    .unwrap()
                    .amount,
                49
            );
            assert_eq!((r.tax, r.refund, r.balance), (Some(0), Some(10_000), Some(0)));
        } else {
            crate::tests::support::assert::injured_income::ltc_cent_sources::ordinary(&r);
        }
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!(r.return_lines, plain.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (plain.tax, plain.payments, plain.refund, plain.balance)
        );
        r
    }
    pub(in crate::tests) fn checked(
        d: &FederalDraft,
        family: Family,
        injured: Owner,
        corrected: bool,
    ) -> FederalResult {
        let r = financial(d, family);
        let (group, rows) = source_rows(d);
        let plan = r.injured_spouse.as_ref().unwrap();
        let expected = if family.shared() {
            let answers = family.answers(injured, corrected);
            assert_eq!(plan.required.len(), 2);
            assert_eq!(plan.accepted.len(), 2);
            for (period, answer) in [1, 2].into_iter().zip(answers) {
                let key = alloc::format!("ltc_{group}_period_{period}");
                let row = plan.required.iter().find(|row| row.component == key).unwrap();
                assert_eq!((row.joint, row.minimum, row.maximum, row.injured), (49, 0, 49, answer));
                assert_eq!(
                    d.injured_spouse
                        .as_ref()
                        .unwrap()
                        .allocations
                        .iter()
                        .find(|row| row.component == key)
                        .unwrap()
                        .injured,
                    answer
                );
            }
            match family {
                Family::Interior => [0, 1],
                _ => {
                    let owns = answers == [49, 49];
                    if owns { [1, 0] } else { [0, 1] }
                }
            }
        } else {
            assert!(plan.required.is_empty());
            assert!(plan.accepted.is_empty());
            assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            if family == Family::Zero {
                [0, 0]
            } else {
                assert_eq!(rows.len(), 1);
                if rows[0].1 == injured { [1, 0] } else { [0, 1] }
            }
        };
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], expected);
        r
    }
}

pub(in crate::tests) mod medicare_actual_clients {
    use crate::tests::support::assert::payroll::injured_additional_medicare_money as money;
    use crate::tests::support::builders::additional_medicare::Source;
    use crate::tests::*;
    pub(in crate::tests) fn checked(
        d: &FederalDraft,
        source: Source,
        owner: Owner,
        w2_owner: Owner,
        injured: Owner,
    ) -> FederalResult {
        let intermediate = matches!(source, Source::PartnerSe) && owner == w2_owner;
        let r = money(d, source, intermediate);
        assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
        assert!(
            r.injured_spouse.as_ref().unwrap().required.is_empty(),
            "actual single contributor needs no shared answer"
        );
        let holder = usize::from(owner == Owner::Spouse);
        let wage_holder = usize::from(w2_owner == Owner::Spouse);
        let inj = usize::from(injured == Owner::Spouse);
        let mut taxes = [380, 380];
        match source {
            Source::PartnerSe => {
                taxes[holder] += if intermediate { 2678 + 381 } else { 14130 + 381 }
            }
            Source::SmallBusiness => taxes[wage_holder] += 450,
            Source::Railroad => taxes[holder] += 450,
        }
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [taxes[inj], taxes[1 - inj]]);
        r
    }
}

pub(in crate::tests) mod medicare_withholding_sources {
    use crate::tests::support::builders::medicare_withholding::{Source, fixture};
    use crate::tests::support::client::injured_income::medicare_withholding_sources::persist;
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, source: Source) -> FederalResult {
        let r = supported(d);
        let (agi, ti, regular, tax, credit, refund) = match source {
            Source::Railroad => (420000, 387800, 78268, 79478, 98, 20620),
            Source::HalfCategories => (420000, 387800, 78268, 79478, 2, 20524),
            Source::Uncollected => (270000, 237800, 42268, 43044, 1, 26957),
            Source::SignedOrdinary => (270000, 237800, 42268, 43028, 1, 26973),
            Source::ZeroUnderage => (270000, 237800, 42268, 43028, 0, 26972),
        };
        for (key, expected) in [
            ("agi", agi),
            ("taxable_income", ti),
            ("regular_income_tax", regular),
            ("additional_medicare_tax", if agi == 420000 { 450 } else { 0 }),
            ("net_investment_income_tax", 760),
            ("form_8960_net_investment_income", 20000),
            ("additional_medicare_total_withheld", credit),
        ] {
            if key == "additional_medicare_total_withheld" && matches!(source, Source::ZeroUnderage)
            {
                // The actual zero native credit has no emitted8959 row. Zero
                // here is established by native25c and all ordinary figures below.
                assert!(r.lines.iter().all(|line| line.id != key));
            } else {
                assert_eq!(value(&r, key), expected, "{source:?} {key}");
            }
        }
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(r.refund, Some(refund * 100));
        assert_eq!(return_line(&r, "25c"), credit * 100);
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(ordinary.return_lines, r.return_lines);
        assert_eq!((ordinary.tax, ordinary.refund, ordinary.balance), (r.tax, r.refund, r.balance));
        r
    }

    pub(in crate::tests) fn check(source: Source) {
        let (mut actual, mut expected) = (vec![], vec![]);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    let d = persist(fixture(source, owner, injured, reverse), source);
                    let r = money(&d, source);
                    let holder = usize::from(owner == Owner::Spouse);
                    let inj = usize::from(injured == Owner::Spouse);
                    let mut owners = match source {
                        Source::Railroad => [49; 2],
                        Source::HalfCategories => [1; 2],
                        _ => [60000, 10000],
                    };
                    if matches!(source, Source::Railroad | Source::HalfCategories) {
                        owners[1 - holder] += 100000;
                    } else if !matches!(source, Source::ZeroUnderage) {
                        owners[holder] += 1;
                    }
                    actual.push((
                        owner,
                        injured,
                        reverse,
                        ["a", "b", "c"].map(|col| {
                            r.lines
                                .iter()
                                .find(|line| line.id == alloc::format!("form_8379_line_19_{col}"))
                                .map(|line| line.amount / 100)
                        }),
                    ));
                    expected.push((
                        owner,
                        injured,
                        reverse,
                        [
                            Some(owners.iter().sum::<i64>()),
                            Some(owners[inj]),
                            Some(owners[1 - inj]),
                        ],
                    ));
                }
            }
        }
        assert_eq!(
            actual, expected,
            "all literal ordinary financial/publicSave/records/restart profiles passed first: {source:?}"
        );
    }
}

pub(in crate::tests) mod native_payroll_other_tax_sources {
    use crate::tests::support::builders::payroll_taxes::{Source, fixture};
    use crate::tests::support::client::injured_income::native_payroll_other_tax_sources::persist;
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, source: Source, owner: Owner) -> FederalResult {
        let r = supported(d);
        let (agi, taxable, regular, additional, tip, uncollected, golden, total, refund) =
            match source {
                Source::SocialTips if owner == Owner::Spouse => {
                    (275000, 242800, 43468, 45, 383, 0, 0, 44656, 25344)
                }
                Source::SocialTips | Source::GovernmentTips => {
                    (275000, 242800, 43468, 45, 73, 0, 0, 44346, 25654)
                }
                Source::RailroadTips => (275000, 242800, 43468, 0, 0, 0, 0, 44228, 25772),
                Source::Uncollected => (270000, 237800, 42268, 0, 0, 77, 0, 43105, 26895),
                Source::CentPair => (270000, 237800, 42268, 0, 0, 1, 0, 43029, 26971),
                Source::GoldenW2 => (270000, 237800, 42268, 0, 0, 0, 1000, 44028, 25972),
                Source::GoldenNec => (290000, 257800, 47068, 0, 0, 0, 1000, 48828, 21172),
                // Same100k W2+160k actualEmployee owner: SS remaining84500*6.2%=5239;
                // ordinaryMed2320;8919 total7559. Actual WH60000, not70000.
                Source::SameEmployee => (280000, 247800, 44668, 90, 0, 0, 0, 53077, 6923),
            };
        for (id, n) in [
            ("agi", agi),
            ("taxable_income", taxable),
            ("regular_income_tax", regular),
            ("additional_medicare_tax", additional),
            ("net_investment_income_tax", 760),
            ("form_8960_net_investment_income", 20000),
            ("w2_uncollected_payroll_tax", uncollected),
        ] {
            assert_eq!(value(&r, id), n, "{source:?} {id}");
        }
        assert_eq!(r.tax, Some(total * 100));
        assert_eq!(r.refund, Some(refund * 100));
        if source.tips() {
            let i = usize::from(owner == Owner::Spouse);
            assert_eq!(
                value(&r, &alloc::format!("employee_tips_{i}_social_tax"))
                    + value(&r, &alloc::format!("employee_tips_{i}_medicare_tax")),
                tip
            );
            assert_eq!(return_line(&r, "1c"), 500000);
        }
        if golden != 0 {
            assert_eq!(value(&r, "schedule_2_golden_parachute_tax"), golden);
        }
        if matches!(source, Source::SameEmployee) {
            let i = usize::from(owner == Owner::Spouse);
            assert_eq!(value(&r, &alloc::format!("employee_wages_{i}_social_tax")), 5239);
            assert_eq!(value(&r, &alloc::format!("employee_wages_{i}_medicare_tax")), 2320);
            assert!(
                r.injured_spouse
                    .as_ref()
                    .unwrap()
                    .required
                    .iter()
                    .all(|row| row.component != "additional_medicare_tax")
            );
        }
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(ordinary.return_lines, r.return_lines);
        assert_eq!((ordinary.tax, ordinary.refund, ordinary.balance), (r.tax, r.refund, r.balance));
        r
    }

    pub(in crate::tests) fn check(source: Source) {
        let (mut observed, mut expected) = (vec![], vec![]);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    let d = persist(fixture(source, owner, injured, reverse), source, owner);
                    let r = money(&d, source, owner);
                    let holder = usize::from(owner == Owner::Spouse);
                    let inj = usize::from(injured == Owner::Spouse);
                    let mut taxes = [380, 380];
                    match source {
                        Source::SocialTips | Source::GovernmentTips => {
                            taxes[0] += 27;
                            taxes[1] += 18;
                            taxes[holder] +=
                                if matches!(source, Source::SocialTips) && owner == Owner::Spouse {
                                    383
                                } else {
                                    73
                                };
                        }
                        Source::Uncollected => taxes[holder] += 77,
                        Source::CentPair => taxes[1 - inj] += 1,
                        Source::GoldenW2 | Source::GoldenNec => taxes[holder] += 1000,
                        Source::SameEmployee => taxes[holder] += 7559 + 90,
                        Source::RailroadTips => {}
                    }
                    observed.push((
                        owner,
                        injured,
                        reverse,
                        ["a", "b", "c"].map(|col| {
                            r.lines
                                .iter()
                                .find(|l| l.id == alloc::format!("form_8379_line_18_{col}"))
                                .map(|l| l.amount / 100)
                        }),
                    ));
                    expected.push((
                        owner,
                        injured,
                        reverse,
                        [Some(taxes.iter().sum::<i64>()), Some(taxes[inj]), Some(taxes[1 - inj])],
                    ));
                }
            }
        }
        assert_eq!(
            observed, expected,
            "all literal native financial/publicSave/Avro/restart profiles passed first: {source:?}"
        );
    }
}

pub(in crate::tests) mod niit_relief_completeness {
    use crate::tests::support::client::injured_income::niit_relief_completeness::accepted_case;
    use crate::tests::*;
    pub(in crate::tests) fn joint_money(draft: &FederalDraft, gross: i64) -> FederalResult {
        let result = supported(draft);
        assert_eq!(value(&result, "agi"), 270_000 + gross);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        // Every tested gross stays inside the same 24% joint bracket. There is
        // no SE/AdditionalMedicare/withholding or deduction change in these facts.
        assert_eq!(value(&result, "regular_income_tax"), 42_268 + gross * 24 / 100);
        let mut ordinary = draft.clone();
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!(
            (result.tax, result.refund, result.balance),
            (plain.tax, plain.refund, plain.balance)
        );
        assert_eq!(
            result
                .lines
                .iter()
                .filter(|line| !line.id.starts_with("form_8379_"))
                .collect::<Vec<_>>(),
            plain
                .lines
                .iter()
                .filter(|line| !line.id.starts_with("form_8379_"))
                .collect::<Vec<_>>()
        );
        result
    }

    pub(in crate::tests) fn owner_case(cases: &[usize]) {
        for &case in cases {
            for owner in [Owner::Taxpayer, Owner::Spouse] {
                for injured in [Owner::Taxpayer, Owner::Spouse] {
                    let (draft, gross) = accepted_case(case, owner, injured);
                    let result = joint_money(&draft, gross);
                    let expected = if injured == Owner::Taxpayer { [190, 570] } else { [570, 190] };
                    assert_eq!(
                        [column(&result, "18", "b"), column(&result, "18", "c")],
                        expected,
                        "case {case}"
                    );
                    let mut amounts = [5000, 15_000];
                    amounts[if owner == Owner::Taxpayer { 0 } else { 1 }] += gross;
                    if injured == Owner::Spouse {
                        amounts.reverse();
                    }
                    assert_eq!(
                        [column(&result, "13b", "b"), column(&result, "13b", "c")],
                        amounts,
                        "case {case}"
                    );
                }
            }
        }
    }
}

pub(in crate::tests) mod niit_supporting_sources {
    use crate::tests::support::builders::injured_income::niit_supporting_sources::{
        Supporting, fixture,
    };
    use crate::tests::support::client::injured_income::niit_supporting_sources::persisted;
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, kind: Supporting) -> FederalResult {
        let r = supported(d);
        let gain = matches!(kind, Supporting::InvestmentGain);
        assert_eq!(return_line(&r, "1a"), 25_000_000);
        assert_eq!(return_line(&r, "2b"), 2_000_000);
        assert_eq!(value(&r, "agi"), if gain { 280_000 } else { 270_000 });
        assert_eq!(value(&r, "taxable_income"), if gain { 247_800 } else { 237_800 });
        // Ordinary237800 uses the native rate worksheet:2480+9120+24332+6336.
        // The investment acquisition's10000 long-term gain adds1500 at15%.
        assert_eq!(value(&r, "regular_income_tax"), if gain { 43_768 } else { 42_268 });
        assert_eq!(
            value(&r, "form_8960_net_investment_income"),
            if gain { 30_000 } else { 20_000 }
        );
        assert_eq!(value(&r, "net_investment_income_tax"), if gain { 1140 } else { 760 });
        assert_eq!(value(&r, "additional_medicare_tax"), 0);
        assert_eq!(r.tax, Some(if gain { 4_490_800 } else { 4_302_800 }));
        assert_eq!(r.refund, Some(if gain { 2_509_200 } else { 2_697_200 }));
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!((r.tax, r.refund, r.balance), (ordinary.tax, ordinary.refund, ordinary.balance));
        r
    }

    pub(in crate::tests) fn check(kind: Supporting) {
        let mut observed = vec![];
        let mut expected = vec![];
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let d = persisted(fixture(kind, owner, injured, reversed), kind);
                    let r = money(&d, kind);
                    let niit = if matches!(kind, Supporting::InvestmentGain) {
                        if owner == injured { [1140, 760, 380] } else { [1140, 380, 760] }
                    } else {
                        [760, 380, 380]
                    };
                    observed.push((
                        owner,
                        injured,
                        reversed,
                        ["a", "b", "c"].map(|col| {
                            r.lines
                                .iter()
                                .find(|line| line.id == alloc::format!("form_8379_line_18_{col}"))
                                .map(|line| line.amount / 100)
                        }),
                    ));
                    expected.push((owner, injured, reversed, niit.map(Some)));
                    let other = if matches!(kind, Supporting::InvestmentGain) {
                        if owner == injured {
                            [30_000, 20_000, 10_000]
                        } else {
                            [30_000, 10_000, 20_000]
                        }
                    } else {
                        [20_000, 10_000, 10_000]
                    };
                    for (col, amount) in ["a", "b", "c"].into_iter().zip(other) {
                        assert_eq!(column(&r, "13b", col), amount);
                    }
                }
            }
        }
        assert_eq!(
            observed, expected,
            "all literal joint financial/public Save/restart controls passed first: {kind:?}"
        );
    }
}

pub(in crate::tests) mod reit_owner_footing {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    use crate::tests::support::builders::injured_income::reit_owner_footing::{Source, fixture};
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft) -> FederalResult {
        let r = supported(d);
        // Wages180000D + actual ordinary dividend10000D =190000D.
        // Native REIT deduction10000D*20%=2000D; no operating QBI/PTP/carry.
        // TI190000-32200-2000=155800; regular11600+22%*(155800-100800)=23700.
        assert_eq!(value(&r, "agi"), 190000);
        assert_eq!(return_line(&r, "3b"), 1000000);
        assert_eq!(value(&r, "qbi_deduction"), 2000);
        assert_eq!(value(&r, "taxable_income"), 155800);
        assert_eq!(value(&r, "regular_income_tax"), 23700);
        assert_eq!(return_line(&r, "23"), 0);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(2370000), Some(2800000), Some(430000), Some(0))
        );
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (ordinary.tax, ordinary.payments, ordinary.refund, ordinary.balance)
        );
        r
    }

    pub(in crate::tests) fn optional(r: &FederalResult, id: &str) -> Option<i64> {
        r.lines.iter().find(|l| l.id == id).map(|l| l.amount)
    }

    pub(in crate::tests) fn profiles(kinds: &[Source]) {
        let mut observations = vec![];
        for &kind in kinds {
            for owner in [Owner::Taxpayer, Owner::Spouse] {
                for injured in [Owner::Taxpayer, Owner::Spouse] {
                    for reverse in [false, true] {
                        let d = fixture(kind, owner, injured, reverse);
                        let actual = d
                            .income
                            .iter()
                            .find(|e| {
                                matches!(
                                    &e.income,
                                    FederalIncome::Partnership { .. }
                                        | FederalIncome::Scorp { .. }
                                        | FederalIncome::Dividend { .. }
                                        | FederalIncome::EstateTrustPortfolio { .. }
                                )
                            })
                            .unwrap()
                            .clone();
                        money(&d);
                        let mut s = restart(d);
                        for _ in 0..2 {
                            let p = open_topic(&mut s, kind.topic());
                            accepted(&send(&mut s, p));
                            let Reply::Edit(p) = crate::tests::support::client::native_editor(
                                from_review(&mut s, TaxSection::InjuredSpouse),
                                &[],
                            ) else {
                                panic!()
                            };
                            accepted(&send(&mut s, p));
                            assert_eq!(
                                s.draft.income.iter().find(|e| e.id == actual.id),
                                Some(&actual)
                            );
                            money(&s.draft);
                            let stored = s.draft;
                            let bytes = native_draft_records(&stored);
                            let draft = draft_from_native_records(stored.tax_year, &bytes);
                            assert_eq!(native_draft_records(&draft), bytes);
                            assert_eq!(draft, stored);
                            s = restart(draft);
                        }
                        let r = money(&s.draft);
                        let qbi = ["form_8379_supplemental_qbi_b", "form_8379_supplemental_qbi_c"]
                            .map(|id| optional(&r, id));
                        let warning = noted(&r, "WARN_8379_QBI_OWNER_UNAVAILABLE");
                        let benefit = optional(&r, crate::injured_spouse::ESTIMATE_LINE);
                        observations.push((kind, owner, injured, reverse, qbi, warning, benefit));
                    }
                }
            }
        }
        // The dispatcher assigned the actual K1 owner accumulation after the
        //825d ordinary/public lifecycle RED and6519 missing-feed protection passed.
        // Those exact sources/native/results remain sealed; no monetary oracle changes.
        for (kind, owner, injured, reverse, qbi, warning, benefit) in &observations {
            let expected =
                if owner == injured { [Some(200000), Some(0)] } else { [Some(0), Some(200000)] };
            assert_eq!(*qbi, expected, "{kind:?}/{owner:?}/{injured:?}/reverse{reverse}");
            assert!(!warning);
            assert!(benefit.is_some());
        }
    }
}

pub(in crate::tests) mod typed_information_return_sources {
    use crate::tests::*;
    pub(in crate::tests) fn check_states(
        mut draft: FederalDraft,
        owners: [i64; 2],
        ordinary: impl Fn(&FederalResult),
    ) {
        let mut actual = vec![];
        let mut expected = vec![];
        for reversed in [false, true] {
            if reversed {
                draft.income.reverse();
            }
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                let result = supported(&draft);
                ordinary(&result);
                // Both native taxable incomes are in the47800–47850 Tax Table
                // row:2480 +12% ×(47825−24800) =5243. Payments are6500.
                assert_eq!(result.tax, Some(524_300));
                assert_eq!(result.refund, Some(125_700));
                let mut plain = draft.clone();
                plain.injured_spouse = None;
                let without_allocation = supported(&plain);
                assert_eq!(result.return_lines, without_allocation.return_lines);
                assert_eq!(
                    (result.tax, result.refund, result.balance),
                    (without_allocation.tax, without_allocation.refund, without_allocation.balance)
                );
                let restored = restart(draft.clone());
                assert_eq!(restored.draft, draft);
                assert_eq!(estimate(&restored.draft), result);

                let observed = ["a", "b", "c"].map(|column| {
                    result
                        .lines
                        .iter()
                        .find(|line| line.id == alloc::format!("form_8379_line_13b_{column}"))
                        .map(|line| line.amount / 100)
                });
                actual.push((reversed, injured, observed));
                let pair = if injured == Owner::Taxpayer { owners } else { [owners[1], owners[0]] };
                expected.push((
                    reversed,
                    injured,
                    [Some(owners.iter().sum()), Some(pair[0]), Some(pair[1])],
                ));
            }
        }
        assert_eq!(actual, expected);
    }
}
