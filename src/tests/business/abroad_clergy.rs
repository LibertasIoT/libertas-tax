// Independent approved Unit41 map fixtures: Pub517, ScheduleSE and Form8801.
use super::*;

#[test]
fn unit41_form4029_exempts_only_the_approved_business_owner() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(cash_business(20000)));
    let mut spouse = income(cash_business(10000));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    d.screening.as_mut().unwrap().clergy =
        vec![ClergyClaim { owner: Owner::Taxpayer, role: ClergyRole::ReligiousSect }];
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert!(!r.lines.iter().any(|line| line.id == "taxpayer_se_net_earnings"));
    assert_eq!(value(&r, "spouse_se_net_earnings"), 9235);
    assert_eq!(value(&r, "agi"), 29293);
}

#[test]
fn unit41_optional_clergy_wages_and_housing_share_one_nonfarm_method() {
    // Pub.517 nonfarm gross=3,000 wages+3,000 housing; employee expenses2,000.
    // Actual profit4,000 <72.189%of6,000; optional net=2/3of6,000=4,000.
    // All half-SE belongs to employee ministerial work, never a business.
    let mut d = empty_return(FederalStatus::Single);
    let mut pay = wage(1, Owner::Taxpayer, 300_000, 0);
    if let FederalIncome::Wage { data } = &mut pay.income {
        let mut data = edit_wage(data);
        data.coverage = W2Coverage::Ministerial;
        data.social_wages = 0;
        data.social_tax = 0;
        data.medicare_wages = 0;
        data.medicare_tax = 0;
    }
    add(&mut d, pay);
    d.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Allowance {
                designated: 300_000,
                spent: 300_000,
                fair_rental_value: 300_000,
            },
            form_4361: false,
            employee_expenses: 200_000,
        },
    }];
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(se_history()) }];
    let mut probe = FederalResult::empty(0);
    let optional = crate::optional_se::calculate(&d, Owner::Taxpayer, &mut probe).unwrap();
    assert_eq!(optional.net, 4000);
    assert_eq!(optional.nonbusiness_net, 4000);
    assert!(optional.weights.is_empty());
    assert!(optional.weights_cents.is_empty());
    assert_eq!(optional.eic_adjustment, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_nonfarm"), 4000);
    assert_eq!(value(&r, "self_employment_tax"), 612);
    assert_eq!(value(&r, "taxpayer_half_self_employment_tax"), 306);
    assert_eq!(value(&r, "agi"), 2694);
}

#[test]
fn unit41_nonfarm_optional_record_preserves_farm_half_and_rejects_invalid_history() {
    use crate::tests::cross::interview::{
        accepted, cash_farm, document, farm_save, from_review, rejected_with,
    };
    let mut d = empty_return(FederalStatus::Single);
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: true, nonfarm: None }];
    let mut session = Interview::new(TEST_USER, d);
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::SelfEmployment) else {
        panic!("SE editor");
    };
    let P::SaveIncomes { documents, .. } = &mut page else {
        panic!("SE save");
    };
    documents.documents = vec![document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(se_history()) },
    )]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
    accepted(&send(&mut session, page));
    let election = &session.draft.screening.as_ref().unwrap().optional_se[0];
    assert!(election.farm);
    assert_eq!(election.nonfarm, Some(se_history()));
    assert!(session.draft.income.is_empty(), "supporting record is not income");
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::SelfEmployment) else {
        panic!("SE reopen");
    };
    let P::SaveIncomes { documents, .. } = &mut page else {
        panic!("SE save");
    };
    assert!(documents.documents.iter().any(|row| matches!(&row.document, crate::IncomeRecord::NonfarmOptionalMethod(h) if *h == se_history())));
    let crate::IncomeRecord::NonfarmOptionalMethod(h) = &mut documents.documents[0].document else {
        panic!("history record");
    };
    h.prior_years = vec![2022, 2022];
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "NONFARM_OPTIONAL_HISTORY_INVALID");
    assert_eq!(session.draft, before);
    // Farming writes only its half, including removal; the nonfarm record survives.
    accepted(&{
        let request = farm_save(
            &mut session,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Farm(cash_farm(900_000, 400_000)),
                },
            )],
        );
        send(&mut session, request)
    });
    let election = &session.draft.screening.as_ref().unwrap().optional_se[0];
    assert!(!election.farm);
    assert_eq!(election.nonfarm, Some(se_history()));
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::SelfEmployment) else {
        panic!("SE reopen");
    };
    let P::SaveIncomes { documents, .. } = &mut page else {
        panic!("SE save");
    };
    documents.documents.clear();
    accepted(&send(&mut session, page));
    assert!(session.draft.screening.as_ref().unwrap().optional_se.is_empty());
}

#[test]
fn unit41_optional_ineligible_election_uses_regular_method_with_one_note() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(7000);
    if let FederalIncome::Business { data } = &mut entry {
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 900_000,
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Costs".into() },
            200_000,
        )];
    }
    add(&mut d, income(entry));
    let ordinary = supported(&d);
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(se_history()) }];
    let result = supported(&d);
    assert_eq!(result.tax, ordinary.tax);
    assert_eq!(value(&result, "self_employment_tax"), value(&ordinary, "self_employment_tax"));
    assert!(noted(&result, "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
    assert_eq!(
        result
            .review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_OPTIONAL_METHOD_NOT_AVAILABLE")
            .count(),
        1
    );
}

#[test]
fn unit41_form4029_removes_farm_and_partner_earnings_from_se() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(20000, 0)));
    let mut partner = ordinary_partner_source(10000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(1_000_000);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    d.screening.as_mut().unwrap().clergy =
        vec![ClergyClaim { owner: Owner::Taxpayer, role: ClergyRole::ReligiousSect }];
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert!(
        !r.lines
            .iter()
            .any(|line| { line.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleSe) })
    );
    assert_eq!(value(&r, "agi"), 30000);
}

#[test]
fn unit41_optional_method_cannot_generate_se_from_totalization_exempt_source() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = cash_business(5000);
    if let FederalIncome::Business { data } = &mut entry {
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 900_000,
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Expenses".into() },
            400_000,
        )];
        data.source =
            BusinessSource::ForeignServices { confirmed: true, totalization_exempt: true };
    }
    add(&mut d, income(entry));
    d.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: false, nonfarm: Some(se_history()) }];
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 5000);
    assert!(noted(&r, "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
    assert!(!r.lines.iter().any(|line| line.id == "optional_se_taxpayer_net"));
}
