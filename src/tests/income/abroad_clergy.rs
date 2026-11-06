use crate::tests::support::client::write_native_draft;
// Independent fixtures: Pub517, ScheduleSE and Form8801.
use super::*;
use crate::tests::support::client::income_pages::open_documents;
use alloc::format;

fn minister() -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    let mut pay = wage(1, Owner::Taxpayer, 5_950_000, 600_000);
    if let FederalIncome::Wage { data } = &mut pay.income {
        let mut data = edit_wage(data);
        data.coverage = W2Coverage::Ministerial;
        data.social_wages = 0;
        data.social_tax = 0;
        data.medicare_wages = 0;
        data.medicare_tax = 0;
    }
    add(&mut d, pay);
    let mut fees = cash_business(1600);
    if let FederalIncome::Business { data } = &mut fees {
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 200_000,
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Ministerial costs".into() },
            40_000,
        )];
        data.ministerial = true;
    }
    add(&mut d, income(fees));
    d.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Parsonage {
                fair_rental_value: 2_400_000,
                utility_allowance: 750_000,
                utilities_paid: 700_000,
            },
            form_4361: false,
            employee_expenses: 0,
        },
    }];
    d
}

#[test]
fn unit41_f1_minister_parsonage_and_fees_independent_oracles() {
    let d = minister();
    let r = supported(&d);
    for (id, expected) in [
        ("clergy_taxpayer_housing_excluded", 31000),
        ("clergy_taxpayer_housing_excess", 500),
        ("clergy_taxpayer_nondeductible_expenses", 133),
        ("clergy_taxpayer_se_earnings", 92600),
        ("self_employment_tax", 13084),
        ("taxpayer_half_self_employment_tax", 6542),
        ("agi", 55191),
        ("qbi_deduction", 400),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert_eq!(return_line(&r, "1h"), 50_000);
    assert_eq!(return_line(&r, "1z"), 6_000_000);
    assert_eq!(return_line(&r, "15"), 3_869_100);
    assert_eq!(return_line(&r, "16"), 439_300);
    assert_eq!(r.tax, Some(1_747_700));
}

#[test]
fn unit41_f1_approved_4361_preserves_income_tax_and_removes_ministerial_se() {
    let mut d = minister();
    if let ClergyRole::Minister { form_4361, .. } =
        &mut d.screening.as_mut().unwrap().clergy[0].role
    {
        *form_4361 = true;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 61733);
    assert_eq!(value(&r, "qbi_deduction"), 400);
    assert_eq!(return_line(&r, "15"), 4_523_300);
    assert_eq!(return_line(&r, "16"), 517_900);
}

#[test]
fn unit41_f5_notary_and_totalization_independent_oracles() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(20000);
    if let FederalIncome::Business { data } = &mut entry {
        data.notary_profit = 300_000;
    }
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 2402);
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(50000);
    if let FederalIncome::Business { data } = &mut entry {
        data.source =
            BusinessSource::ForeignServices { confirmed: true, totalization_exempt: true };
    }
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert!(!r.lines.iter().any(|row| row.id == "taxpayer_half_self_employment_tax"));
    assert_eq!(value(&r, "agi"), 50000);
}

#[test]
fn unit41_f4_nonfarm_optional_method_independent_oracles_and_fallback() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(5000);
    if let FederalIncome::Business { data } = &mut entry {
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 900_000,
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Operating expenses".into() },
            400_000,
        )];
    }
    add(&mut d, income(entry));
    let history = NonfarmSeHistory {
        earnings_third_prior_year: 300_000,
        earnings_second_prior_year: 0,
        earnings_prior_year: 250_000,
        prior_years: vec![2022, 2024],
    };
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(history) }];
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 6000);
    assert_eq!(value(&r, "self_employment_tax"), 918);
    let mut probe = FederalResult::empty(0);
    let method = crate::optional_se::calculate(&d, Owner::Taxpayer, &mut probe).unwrap();
    // WorksheetB starts with line3 profit5000, not line4a earnings4618.
    assert_eq!(method.eic_adjustment, 1000);
}

#[test]
fn unit41_wages_clergy_record_roundtrips_and_enforces_owner_uniqueness() {
    use crate::tests::cross::interview::{accepted, document, from_review, rejected_with};
    let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Joint));
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::Wages) else {
        panic!("Wages editor");
    };
    let P::SaveWagesAnswers { records, .. } = &mut page else {
        panic!("Wages save");
    };
    let record = document(
        0,
        IncomeSource::Record {
            value: crate::IncomeRecord::Clergy(ClergyRecord { role: ClergyRole::ReligiousSect }),
        },
    );
    records.documents = vec![
        crate::IncomeRecordItem::from_income(record.clone())
            .and_then(crate::WagesRecordItem::from_income)
            .unwrap(),
    ];
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let Reply::Edit(page) = from_review(&mut session, TaxSection::Wages) else {
        panic!("Wages reopen");
    };
    let P::SaveWagesAnswers { records, .. } = &page else {
        panic!("Wages reopen save");
    };
    assert_eq!(records.documents.len(), 1);
    assert_eq!(records.documents[0].clone().into_income().into_income().document, record.document);
    accepted(&send(&mut session, page));
    assert_eq!(
        session.draft.screening.as_ref().unwrap().clergy,
        saved.screening.as_ref().unwrap().clergy
    );
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::Wages) else {
        panic!("Wages reopen");
    };
    let P::SaveWagesAnswers { records, .. } = &mut page else {
        panic!("Wages save");
    };
    records.documents.push(
        crate::IncomeRecordItem::from_income(record)
            .and_then(crate::WagesRecordItem::from_income)
            .unwrap(),
    );
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "CLERGY_TWICE");
    assert_eq!(session.draft, before);
}

#[test]
fn unit41_revocation_blocks_only_selected_election_and_irs_consent_restores_it() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let base = supported(&d);
    assert_eq!(value(&base, "foreign_earned_exclusion"), 100000);
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::Revoked {
        year: 2023,
        elections: RevokedElections::EarnedIncome,
        consent: false,
    };
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_ELECTION_BARRED"));
    assert_eq!(value(&r, "foreign_earned_exclusion"), 0);
    assert_eq!(value(&r, "agi"), 100000);
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::Revoked {
        year: 2023,
        elections: RevokedElections::EarnedIncome,
        consent: true,
    };
    let r = supported(&d);
    assert!(!noted(&r, "WARN_FOREIGN_ELECTION_BARRED"));
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100000);
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::Revoked {
        year: 2023,
        elections: RevokedElections::Housing,
        consent: false,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100000);
    assert!(!noted(&r, "WARN_FOREIGN_ELECTION_BARRED"));
}

#[test]
fn unit41_clergy_housing_and_employee_expense_boundaries_use_full_se_expenses() {
    let mut d = minister();
    if let ClergyRole::Minister { housing, employee_expenses, .. } =
        &mut d.screening.as_mut().unwrap().clergy[0].role
    {
        *housing = MinisterHousing::Allowance {
            designated: 1_000_000,
            spent: 700_000,
            fair_rental_value: 800_000,
        };
        *employee_expenses = 350_000;
    }
    let facts = crate::clergy::calculate(&d, Owner::Taxpayer);
    assert_eq!(facts.housing_tax_free, 700_000);
    assert_eq!(facts.housing_excess, 300_000);
    //59,500salary+10,000fullallowance+1,600fees−3,500employeeexpenses.
    assert_eq!(facts.se_line_2, 6_760_000);
    //7,000/(59,500+2,000+10,000)*400=39.160839... dollars.
    assert_eq!(facts.nondeductible_by_business.values().copied().collect::<Vec<_>>(), vec![3916]);
    d.screening.as_mut().unwrap().clergy[0].role =
        ClergyRole::ChristianSciencePractitioner { form_4361: false, employee_expenses: 350_000 };
    let facts = crate::clergy::calculate(&d, Owner::Taxpayer);
    assert_eq!(facts.housing_tax_free, 0);
    assert_eq!(facts.housing_excess, 0);
    assert_eq!(facts.se_line_2, 5_760_000);
    assert_eq!(facts.nondeductible_by_business.values().sum::<i64>(), 0);
}

#[test]
fn unit41_farm_optional_eligibility_subtracts_crp_once() {
    //Gross25,000 exceeds gross-test ceiling. ScheduleFprofit15,000 less
    //exemptCRP10,000=5,000 qualifies via profit<8,186, retaining7,560optional.
    let mut d = crp_return(5000, &[(10000, true)]);
    elect_se(&mut d, SeMethod::Farm);
    let mut probe = FederalResult::empty(0);
    let optional = crate::optional_se::calculate(&d, Owner::Taxpayer, &mut probe).unwrap();
    assert_eq!(optional.net, 7560);
    assert_eq!(optional.eic_adjustment, 2560);
    assert_eq!(optional.child_adjustment, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_farm"), 7560);
    assert!(!noted(&r, "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
}

#[test]
fn unit41_exemption_arguments_retain_boxes_codes_and_schedule_se_line_a() {
    let mut d = minister();
    if let ClergyRole::Minister { form_4361, .. } =
        &mut d.screening.as_mut().unwrap().clergy[0].role
    {
        *form_4361 = true;
    }
    let r = supported(&d);
    let row = r.lines.iter().find(|line| line.id == "self_employment_tax").unwrap();
    assert_eq!(row.explanation.key, "EXPLAIN_SE_EXEMPTION_BOXES");
    assert_eq!(row.explanation.args[0], MessageArg::Number(1));
    assert_eq!(row.explanation.args[2], MessageArg::Number(0));
    assert!(
        !r.lines
            .iter()
            .any(|line| { line.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleSe) })
    );
    add(&mut d, income(cash_business(10000)));
    let r = supported(&d);
    let row = r.lines.iter().find(|line| line.id == "self_employment_tax").unwrap();
    assert_eq!(row.explanation.args[0], MessageArg::Number(0));
    assert_eq!(row.explanation.args[2], MessageArg::Number(1));
    assert_eq!(value(&r, "taxpayer_se_nonfarm_profit"), 10000);
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(3000);
    if let FederalIncome::Business { data } = &mut entry {
        data.notary_profit = 300_000;
    }
    add(&mut d, income(entry));
    let r = supported(&d);
    let row = r.lines.iter().find(|line| line.id == "self_employment_tax").unwrap();
    assert_eq!(row.explanation.args[0], MessageArg::Number(3));
    assert_eq!(row.explanation.args[1], MessageArg::Text("EN".into()));
    assert_eq!(value(&r, "agi"), 3000);
}

#[test]
fn unit41_minister_health_limit_uses_business_profit_and_its_own_half_se_share() {
    let mut d = minister();
    // Actual business net1,600 + housing expense addback133 −businesshalf113.
    // Church wages/housing cannot supply this trade's Form7206 limit.
    se_health(&mut d, 1, 10000);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 13084);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 1620);
}

#[test]
fn unit41_minister_sep_uses_taxable_trade_profit_and_only_trade_half_se() {
    let mut d = minister();
    let business_id = d.income[1].id;
    let FederalIncome::Business { data } = &mut d.income[1].income else {
        panic!("ministerial fees");
    };
    data.retirement = BusinessRetirement::Sep { contribution: 32_400, rate: 25.0 };
    // Pub560 starts with ScheduleC line31:1,600+133 housing-expense addback.
    // Only this business's113 half-SE deduction reduces its1,620 capacity;
    // the25% plan rate becomes20%, giving an independent324 contribution.
    let r = supported(&d);
    assert_eq!(value(&r, &format!("business_{business_id}_sep_limit")), 324);
    assert_eq!(value(&r, &format!("business_{business_id}_sep_deduction")), 324);
    assert_eq!(value(&r, "self_employment_tax"), 13084);
}

#[test]
fn unit41_removing_spouse_prunes_both_personal_records_through_normal_setup() {
    use crate::tests::cross::interview::{accepted, save_setup};
    let mut d = empty_return(FederalStatus::Joint);
    d.checklist = crate::tests::cross::interview::activities(ActivityTopics::BUSINESS.0);
    d.screening.as_mut().unwrap().clergy = vec![
        ClergyClaim { owner: Owner::Taxpayer, role: ClergyRole::ReligiousSect },
        ClergyClaim { owner: Owner::Spouse, role: ClergyRole::ReligiousSect },
    ];
    d.screening.as_mut().unwrap().optional_se = [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| SeElection { owner, farm: false, nonfarm: Some(se_history()) })
        .collect();
    let mut session = Interview::new(TEST_USER, d.clone());
    accepted(&send(&mut session, save_setup(&d, ReturnSetup { filing: Filing::Single })));
    let screening = session.draft.screening.as_ref().unwrap();
    assert_eq!(screening.clergy.len(), 1);
    assert_eq!(screening.clergy[0].owner, Owner::Taxpayer);
    assert_eq!(screening.optional_se.len(), 1);
    assert_eq!(screening.optional_se[0].owner, Owner::Taxpayer);
    assert!(session.draft.income.is_empty());
}

#[test]
fn unit41_removing_a_business_clears_its_optional_method_and_keeps_clergy_records() {
    use crate::tests::cross::interview::{accepted, activities, save_businesses};
    let mut d = empty_return(FederalStatus::Single);
    d.checklist = activities(ActivityTopics::BUSINESS.0);
    add(&mut d, income(cash_business(0)));
    d.screening.as_mut().unwrap().clergy =
        vec![ClergyClaim { owner: Owner::Taxpayer, role: ClergyRole::ReligiousSect }];
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(se_history()) }];
    let mut session = Interview::new(TEST_USER, d);
    accepted(&save_businesses(&mut session, vec![]));
    assert!(session.draft.income.is_empty());
    let screening = session.draft.screening.as_ref().unwrap();
    assert_eq!(screening.clergy.len(), 1);
    assert!(screening.optional_se.is_empty());
}

#[test]
fn unit41_housing_revocation_keeps_self_employed_deduction_and_never_reassigns_employee_share() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    add(&mut d, income(foreign_service_business(100000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.housing = housing_claim(40000, ForeignHousingLocation::General, None);
    set_claim(&mut d, claim);
    let ordinary = supported(&d);
    //39870ceiling−21264base=18606; employee and business eachfundhalf.
    assert_eq!(value(&ordinary, "foreign_housing_exclusion"), 9303);
    assert_eq!(value(&ordinary, "foreign_housing_deduction"), 9303);
    claim_mut(&mut d, Owner::Taxpayer).revocation = ExclusionRevocation::Revoked {
        year: 2025,
        elections: RevokedElections::Housing,
        consent: false,
    };
    let barred = supported(&d);
    assert_eq!(value(&barred, "foreign_housing_exclusion"), 0);
    assert_eq!(value(&barred, "foreign_housing_deduction"), 9303);
    assert_eq!(value(&barred, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&barred, "agi") - value(&ordinary, "agi"), 9303);
    assert!(noted(&barred, "WARN_FOREIGN_HOUSING_ELECTION_BARRED"));
    assert!(!noted(&barred, "WARN_FOREIGN_ELECTION_BARRED"));
    if let ExclusionRevocation::Revoked { consent, .. } =
        &mut claim_mut(&mut d, Owner::Taxpayer).revocation
    {
        *consent = true;
    }
    let restored = supported(&d);
    assert_eq!(restored.tax, ordinary.tax);
    assert_eq!(value(&restored, "foreign_housing_exclusion"), 9303);
    assert!(!noted(&restored, "WARN_FOREIGN_HOUSING_ELECTION_BARRED"));
}

#[test]
fn unit41_joint_housing_excess_aggregates_cents_before_form1040_rounding() {
    let mut d = empty_return(FederalStatus::Joint);
    d.screening.as_mut().unwrap().clergy = [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| ClergyClaim {
            owner,
            role: ClergyRole::Minister {
                housing: MinisterHousing::Allowance {
                    designated: 100,
                    spent: 51,
                    fair_rental_value: 100,
                },
                form_4361: false,
                employee_expenses: 0,
            },
        })
        .collect();
    //Two49-cent excess allowances become98cents, reportedonceas$1.
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 100);
    assert_eq!(return_line(&r, "1z"), 100);
    assert_eq!(value(&r, "wages"), 1);
}

#[test]
fn unit41_clergy_credit_earned_income_distinguishes_housing_and_approved_4361() {
    let mut d = empty_return(FederalStatus::Single);
    let mut pay = wage(1, Owner::Taxpayer, 300_000, 0);
    let FederalIncome::Wage { data } = &mut pay.income else {
        panic!("ministerial wage");
    };
    let mut data = edit_wage(data);
    data.coverage = W2Coverage::Ministerial;
    data.social_wages = 0;
    data.social_tax = 0;
    data.medicare_wages = 0;
    data.medicare_tax = 0;
    drop(data);
    add(&mut d, pay);
    d.dependents.push(child(d.next_id));
    d.next_id += 1;
    d.dependents[0].set_care_expenses(300000);
    document_care(&mut d);
    d.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Allowance {
                designated: 300_000,
                spent: 300_000,
                fair_rental_value: 300_000,
            },
            form_4361: false,
            employee_expenses: 0,
        },
    }];
    // SE:6,000×.9235=5,541;×.153=848; half424. EIC earned5,576;
    // the5,550–5,600 table's midpoint5,575×34%=1,896 for one child.
    // ACTC excludes3,000 tax-free housing: (2,576−2,500)×15%=11.
    let regular = supported(&d);
    assert_eq!(value(&regular, "self_employment_tax"), 848);
    assert_eq!(value(&regular, "agi"), 2576);
    assert_eq!(value(&regular, "earned_income_credit"), 1896);
    // Form2441 line4: wage3000 moved to SE, housing3000 added, halfSE424
    // deducted once =>5576. This is before its nonrefundable tax limit.
    assert_eq!(value(&regular, "care_earned_taxpayer"), 5576);
    assert_eq!(value(&regular, "additional_child_credit_earned_income"), 2576);
    assert_eq!(value(&regular, "additional_child_credit"), 11);

    let ClergyRole::Minister { form_4361, .. } = &mut d.screening.as_mut().unwrap().clergy[0].role
    else {
        panic!("minister");
    };
    *form_4361 = true;
    // Approved4361 removes housing and ministerial self-employment from EIC,
    // retaining3,000 employee wages: table3,025×34%=1,029; ACTC500×15%=75.
    let exempt = supported(&d);
    assert_eq!(value(&exempt, "self_employment_tax"), 0);
    assert_eq!(value(&exempt, "agi"), 3000);
    assert_eq!(value(&exempt, "earned_income_credit"), 1029);
    // Approved4361: taxable employee wage3000 remains; exempt housing
    // and ministerial SE do not supply Form2441 earned income.
    assert_eq!(value(&exempt, "care_earned_taxpayer"), 3000);
    assert_eq!(value(&exempt, "additional_child_credit_earned_income"), 3000);
    assert_eq!(value(&exempt, "additional_child_credit"), 75);
}

pub(in crate::tests) fn bonus_return() -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    let mut pay = wage(1, Owner::Taxpayer, 15_000_000, 900_000);
    let FederalIncome::Wage { data } = &mut pay.income else { unreachable!() };
    let mut data = edit_wage(data);
    data.abroad = 15_000_000;
    drop(data);
    add(&mut d, pay);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20250601,
        ended: None,
        basis: ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    claim.prior_year_history =
        Some(PriorYearHistory { prior_year_days: 214, prior_year_excluded: 7_000_000 });
    claim.multi_year_pay = vec![MultiYearPay {
        description: "Bonus March2024-February2026".into(),
        amount: 3_000_000,
        months_total: 24,
        months_in_return_year: 2,
        months_in_prior_year: 7,
    }];
    set_claim(&mut d, claim);
    d
}

#[test]
fn unit41_prior_only_uses_actual_2025_history_without_2026_qualification() {
    let mut d = empty_return(FederalStatus::Single);
    let mut pay = wage(1, Owner::Taxpayer, 1_000_000, 0);
    let FederalIncome::Wage { data } = &mut pay.income else { unreachable!() };
    let mut data = edit_wage(data);
    data.abroad = 1_000_000;
    drop(data);
    add(&mut d, pay);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::PriorYearOnly;
    claim.tax_home = false;
    claim.prior_year_pay = 1_000_000;
    claim.prior_year_history =
        Some(PriorYearHistory { prior_year_days: 365, prior_year_excluded: 12_500_000 });
    assert_eq!(Form2555::from_record(&claim).rule(), None);
    set_claim(&mut d, claim);
    // Prior capacity130000-125000=5000; no current income/exclusion/housing.
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 0);
    assert_eq!(value(&r, "foreign_earned_taxpayer_days"), 0);
    assert_eq!(value(&r, "foreign_prior_year_taxpayer_exclusion"), 5000);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 5000);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
    assert_eq!(value(&r, "agi"), 5000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    claim_mut(&mut d, Owner::Taxpayer).prior_year_history = None;
    assert_eq!(
        Form2555::from_record(claim_mut(&mut d, Owner::Taxpayer)).rule(),
        Some("FOREIGN_PRIOR_ONLY_NEEDS_HISTORY")
    );
}

fn optional_history() -> NonfarmSeHistory {
    NonfarmSeHistory {
        earnings_third_prior_year: 40000,
        earnings_second_prior_year: 40000,
        earnings_prior_year: 0,
        prior_years: vec![],
    }
}

#[test]
fn unit41_notary_optional_method_uses_actual_gross_and_regular_fallback() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(5000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 900000,
        accounting: BusinessAccounting::Cash,
    };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Costs".into() },
        400000,
    )];
    data.notary_profit = 500000;
    data.notary_gross = Some(700000);
    add(&mut d, income(entry));
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(optional_history()) }];
    // Ordinary gross2000 and profit0 =>2/3gross1333; tax204, half102.
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 1333);
    assert_eq!(value(&r, "self_employment_tax"), 204);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.notary_gross = Some(900000);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.notary_gross = None;
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert!(noted(&r, "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
}

#[test]
fn unit41_prior_and_award_pages_roundtrip_and_reject_cross_page_source_loss() {
    use crate::tests::cross::interview::{accepted, document, from_review, rejected_with};
    let mut session = Interview::new(TEST_USER, bonus_return());
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::WorkAbroad) else {
        panic!("Work abroad")
    };
    let P::SaveIncomes { documents, .. } = &mut page else { panic!("SaveIncomes") };
    let crate::IncomeRecord::Form2555(form) = &mut documents
        .documents
        .iter_mut()
        .find(|d| matches!(d.document, crate::IncomeRecord::Form2555(_)))
        .unwrap()
        .document
    else {
        unreachable!()
    };
    form.prior_year_pay = 100000;
    // Same candidate adds the actual source pay and its disjoint prior split.
    documents.documents.push(
        crate::IncomeRecordItem::from_income(document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::ForeignPay(ForeignPay {
                    employer: "Late salary".into(),
                    wages: 100000,
                }),
            },
        ))
        .unwrap(),
    );
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let Reply::Edit(page) = from_review(&mut session, TaxSection::WorkAbroad) else {
        panic!("Reopen")
    };
    let P::SaveIncomes { documents, .. } = &page else { panic!("SaveIncomes") };
    let crate::IncomeRecord::Form2555(form) = &documents
        .documents
        .iter()
        .find(|d| matches!(d.document, crate::IncomeRecord::Form2555(_)))
        .unwrap()
        .document
    else {
        unreachable!()
    };
    assert_eq!(form, &Form2555::from_record(&saved.screening.as_ref().unwrap().foreign_earned[0]));
    assert_eq!(value(&supported(&saved), "foreign_earned_taxpayer_income"), 122500);
    let mut wage_page = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut wage_page else { panic!("SaveDocuments") };
    let crate::WageDocument::W2(pay) = &mut documents.common.wages.documents[0].document else {
        panic!("W2")
    };
    // The refused answer is registered with a paper of its own (its control number), so the
    // harness's restoration of the original W-2 does not replay it.
    pay.control_number = "refused-awards".into();
    let mut answers = stored_w2_answers(&session.draft, 0);
    answers.abroad = W2Amount::of(2_000_000);
    wage_document(W2Source { form: pay.clone(), answers });
    rejected_with(
        current_income_refusal(
            "income-source-76",
            &mut session,
            wage_page,
            "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
            "[documents/common/wages/documents/0/document/answers/abroad]",
        ),
        "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
    );
    assert_eq!(session.draft, saved);
    let mut bad = page;
    let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
    documents.documents.retain(|d| !matches!(d.document, crate::IncomeRecord::ForeignPay(_)));
    // Direct1000 +award30000 still fits W2gross150000; raise actual direct to
    //120001 so its disjoint source total exceeds150000 by exactly one dollar.
    let crate::IncomeRecord::Form2555(form) = &mut documents
        .documents
        .iter_mut()
        .find(|d| matches!(d.document, crate::IncomeRecord::Form2555(_)))
        .unwrap()
        .document
    else {
        unreachable!()
    };
    form.prior_year_pay = 12_000_100;
    rejected_with(
        current_income_refusal(
            "income-source-77",
            &mut session,
            bad,
            "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
            "[documents/documents/0/document/multi_year_pay]",
        ),
        "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
    );
    assert_eq!(session.draft, saved);
}

#[test]
fn unit41_notary_gross_page_roundtrips_and_rejects_gross_below_net() {
    use crate::tests::cross::interview::{accepted, from_review, rejected_with};
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(5000)));
    let mut session = Interview::new(TEST_USER, d);
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::SelfEmployment) else {
        panic!("Self employment")
    };
    let P::SaveIncomes { documents, .. } = &mut page else { panic!("SaveIncomes") };
    let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
        panic!("Business")
    };
    business.more.notary_profit = 500000;
    business.more.notary_gross = Some(500000);
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::SelfEmployment) else {
        panic!("Reopen")
    };
    let P::SaveIncomes { documents, .. } = &mut page else { panic!("SaveIncomes") };
    let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
        panic!("Business")
    };
    assert_eq!(business.more.notary_gross, Some(500000));
    business.more.notary_gross = Some(499999);
    rejected_with(send(&mut session, page), "NOTARY_GROSS_BELOW_NET");
    assert_eq!(session.draft, saved);
    assert_eq!(value(&supported(&restart(saved).draft), "self_employment_tax"), 0);
}

#[test]
fn unit19_retained_source_witness_schedule_one_clergy_business() {
    extern crate std;
    let d = minister();
    assert!(!d.income.iter().any(|entry|matches!(&entry.income,
        FederalIncome::Business { data } if matches!(data.activity,BusinessActivity::Farming { .. }))));
    let r = supported(&d);
    let business = value(&r, "schedule_1_business_income");
    let farm = r
        .lines
        .iter()
        .find(|row| row.id == "schedule_1_farm_income")
        .map_or(0, |row| row.amount / 100);
    std::println!(
        "Actual no-farm minister source: Schedule1 business{business}, farm{farm}; AGI{}; tax{:?}",
        value(&r, "agi"),
        r.tax
    );
    assert_eq!(value(&r, "agi"), 55191);
    assert_eq!(r.tax, Some(1747700));
    assert_eq!(business, 1733);
    assert_eq!(farm, 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    use crate::tests::cross::interview::{accepted, open_topic};
    let record = |name: &str, bytes: &[u8]| {
        if let Ok(dir) = std::env::var("TAX_UNIT19_CF_DIR") {
            let dir = std::path::Path::new(&dir);
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join(name), bytes).unwrap();
        }
    };
    let mut session = Interview::new(TEST_USER, d);
    let page = open_topic(&mut session, TaxSection::SelfEmployment);
    record("initial-editor.avro", &page.to_avro());
    accepted(&send(&mut session, P::from_avro(&page.to_avro()).unwrap()));
    if let Ok(dir) = std::env::var("TAX_UNIT19_CF_DIR") {
        write_native_draft(
            std::path::Path::new(&dir).join("saved-tax-app-data.avro"),
            &session.draft,
        );
    }
    session = restart(session.draft);
    let saved = supported(&session.draft);
    assert_eq!(value(&saved, "schedule_1_business_income"), 1733);
    assert!(!saved.lines.iter().any(|row| row.id == "schedule_1_farm_income"));
    assert_eq!((value(&saved, "agi"), saved.tax), (55191, Some(1747700)));
    let mut page = open_topic(&mut session, TaxSection::SelfEmployment);
    record("reopened-editor.avro", &page.to_avro());
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
        unreachable!()
    };
    business.expenses.other[0].amount = 30000;
    record("expense-correction.avro", &page.to_avro());
    accepted(&send(&mut session, P::from_avro(&page.to_avro()).unwrap()));
    if let Ok(dir) = std::env::var("TAX_UNIT19_CF_DIR") {
        write_native_draft(
            std::path::Path::new(&dir).join("corrected-tax-app-data.avro"),
            &session.draft,
        );
    }
    session = restart(session.draft);
    let corrected = supported(&session.draft);
    // Costs300 give a100 ministerial restoration, C1800 and no Farm.
    // SE13098/half6549 gives AGI60000+1800-6549=55251.
    assert_eq!(value(&corrected, "schedule_1_business_income"), 1800);
    assert!(!corrected.lines.iter().any(|row| row.id == "schedule_1_farm_income"));
    assert_eq!((value(&corrected, "agi"), corrected.tax), (55251, Some(1750300)));
    let page = open_topic(&mut session, TaxSection::SelfEmployment);
    record("corrected-reopened-editor.avro", &page.to_avro());
    accepted(&send(&mut session, P::from_avro(&page.to_avro()).unwrap()));
    assert_eq!(supported(&restart(session.draft).draft).tax, corrected.tax);
}
