use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
// Form 2555 (unit B7b, tax-forms/form-2555.md): the map's fixtures F3 and F4
// (F1 and F2 are return fixtures in tests/fixtures/returns), the two-spouse
// case, the 330-day boundary and the 12-month window, the unknowns and the
// refusals by name.
use super::*;
use crate::tests::cross::interview::{business, from_review, owned, save_topic, w2};

fn dubai_consultant() -> FederalDraft {
    // F3: Schedule C receipts $220,000, expenses $20,000 for services wholly in
    // Dubai; Residence since 2023-01-15; the exclusion and a full-year Dubai
    // housing claim of $60,000.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(220000, 20000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20230115,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    claim.housing = housing_claim(60000, ForeignHousingLocation::UnitedArabEmiratesDubai, None);
    set_claim(&mut d, claim);
    d
}

#[test]
fn f3_self_employed_dubai_housing_deduction_and_line_44() {
    let d = dubai_consultant();
    let r = supported(&d);
    // Schedule C line 31 = 200,000; Schedule SE.
    assert_eq!(value(&r, "self_employment_tax"), 28234);
    // Form 2555: 29b 57,174; 33 = 57,174 - 21,264 = 35,910; 36 = 0.
    assert_eq!(value(&r, "foreign_housing_taxpayer_limit"), 57174);
    assert_eq!(value(&r, "foreign_housing_taxpayer_base"), 21264);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 220000);
    assert_eq!(value(&r, "foreign_earned_taxpayer_days"), 365);
    // Line 44 = (20,000 + 14,117) x 132,900 / 220,000; line 45 = 132,900 - 20,610.
    assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 20610);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 112290);
    // Part IX: 46 = 35,910; 47 = 220,000 - 132,900; 48 = 35,910; 50 = 35,910.
    assert_eq!(value(&r, "foreign_housing_deduction"), 35910);
    assert_eq!(value(&r, "agi"), 37683);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(return_line(&r, "15"), 2_158_300);
    // Foreign Earned Income Tax Worksheet line 6.
    assert_eq!(return_line(&r, "16"), 518_000);
    assert_eq!(r.tax, Some(3_341_400));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn f3_boundaries_carryover_vehicle_and_form_1116() {
    // A 2025 carryover (line 46 $10,000, line 48 $6,000): $4,000
    // joins line 49 and the worksheet tax falls to 4,220.
    let mut d = dubai_consultant();
    claim_mut(&mut d, Owner::Taxpayer).housing = housing_claim(
        60000,
        ForeignHousingLocation::UnitedArabEmiratesDubai,
        Some(HousingCarryover { amount: 1_000_000, deducted: 600_000 }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_housing_deduction"), 39910);
    assert_eq!(value(&r, "agi"), 33683);
    assert_eq!(return_line(&r, "16"), 422_000);
    // A fully used prior amount has no remaining carryover.
    if let ForeignHousing::Claim { prior: Some(prior), .. } =
        &mut claim_mut(&mut d, Owner::Taxpayer).housing
    {
        prior.deducted = prior.amount;
    }
    let r = supported(&d);
    assert!(!noted(&r, "WARN_HOUSING_CARRYOVER_LEFT_OUT"));
    assert_eq!(value(&r, "foreign_housing_deduction"), 35910);
    // Form 1116 with the business run abroad (its own books: foreign branch
    // category b): line 1a is Schedule C line 7 less the Form 2555 gross
    // exclusion, line 2 its expenses and half SE tax less line 44, and line 12
    // the tax on the excluded earnings.
    let mut credit = dubai_consultant();
    let business = credit.income[0].id;
    let mut claim = employment_credit(&[(Owner::Taxpayer, 0, 5000)]);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim {
        sources[0].country = "AE".into();
        sources[0].category = ForeignTaxCategory::Business { business, separate_books: true };
    }
    credit.credits.as_mut().unwrap().foreign_tax = claim;
    let r = supported(&credit);
    // Line 1a 220,000 - 132,900; line 2 20,000 + 14,117 - 20,610.
    assert_eq!(value(&r, "foreign_branch_AE_gross_income"), 87_100);
    assert_eq!(value(&r, "foreign_branch_AE_related_expenses"), 13_507);
    // Taxes 5,000 x 112,290 / (185,883 - 35,910).
    assert_eq!(value(&r, "foreign_branch_tax_reduction"), -3_744);
    // A business not confirmed is left out with its note, exclusion and all.
    let mut unconfirmed = dubai_consultant();
    if let FederalIncome::Business { data } = &mut unconfirmed.income[0].income {
        data.source =
            BusinessSource::ForeignServices { confirmed: false, totalization_exempt: false };
    }
    let r = estimate(&unconfirmed);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    assert!(noted(&r, "WARN_FORM_2555_NO_FOREIGN_INCOME"));
}

// A Form 1116 claim with employment sources only: (owner, income, tax) in whole dollars.
fn employment_credit(sources: &[(Owner, i64, i64)]) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: sources
            .iter()
            .map(|(owner, income, tax)| ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: *owner,
                label: "United Kingdom employer".into(),
                category: ForeignTaxCategory::Employment,
                income: income * 100,
                tax: tax * 100,
            })
            .collect(),
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    }
}

fn london_banker() -> FederalDraft {
    // F4: foreign pay $200,000, Residence since 2022-01-10, the exclusion, no
    // housing; Form 1116 with one Employment source, income $200,000, tax $60,000.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20220110,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    set_claim(&mut d, claim);
    d.credits.as_mut().unwrap().foreign_tax =
        employment_credit(&[(Owner::Taxpayer, 200000, 60000)]);
    d
}

#[test]
fn f4_london_employee_form_1116_reduces_tax_on_excluded_pay() {
    let d = london_banker();
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 20_000_000);
    assert_eq!(return_line(&r, "8"), -13_290_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_710_000);
    assert_eq!(return_line(&r, "15"), 5_100_000);
    // Form 1116: 1a = 200,000 - 132,900; 3e = AGI + 132,900; 12 reduction.
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 200000);
    assert_eq!(value(&r, "foreign_general_excluded_tax_reduction"), 39870);
    assert_eq!(value(&r, "foreign_general_amt_excluded_tax_reduction"), 39870);
    assert_eq!(value(&r, "foreign_tax_credit"), 12240);
    // 14 = 60,000 - 39,870 = 20,130; credit 12,240; carryforward 7,890.
    assert_eq!(value(&r, "foreign_general_unused_tax"), 7890);
    assert_eq!(value(&r, "income_tax"), 12240);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn f4_without_the_record_the_whole_tax_is_creditable() {
    let mut d = london_banker();
    d.screening.as_mut().unwrap().foreign_earned.clear();
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b"), 20_000_000);
    assert_eq!(return_line(&r, "15"), 18_390_000);
    assert_eq!(value(&r, "foreign_tax_credit"), 36734);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 23266);
    assert!(r.lines.iter().all(|l| l.id != "foreign_general_excluded_tax_reduction"));
}

#[test]
fn f4_source_entered_net_of_the_exclusion_does_not_reconcile() {
    let mut d = london_banker();
    d.credits.as_mut().unwrap().foreign_tax =
        employment_credit(&[(Owner::Taxpayer, 132900, 60000)]);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues
            .iter()
            .any(|i| { i.display.key.starts_with("ISSUE_FOREIGN_SOURCE_RECORDS_MUST_RECONCILE") })
    );
}

#[test]
fn two_spouses_exclude_separately_and_each_owner_reduces_its_own_foreign_tax() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(150000)));
    let mut spouse = income(foreign_pay(100000));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    let r = supported(&d);
    // 132,900 for the taxpayer, 100,000 for the spouse: separate limits.
    assert_eq!(value(&r, "foreign_earned_exclusion"), 232900);
    assert_eq!(value(&r, "foreign_earned_taxpayer_income"), 150000);
    assert_eq!(value(&r, "foreign_earned_spouse_income"), 100000);
    // Form 1116: taxpayer tax 30,000 on 150,000 (132,900/150,000 excluded),
    // spouse tax 10,000 on 100,000 (all excluded).
    d.credits.as_mut().unwrap().foreign_tax =
        employment_credit(&[(Owner::Taxpayer, 150000, 30000), (Owner::Spouse, 100000, 10000)]);
    let r = supported(&d);
    // 30,000 x 132,900 / 150,000 = 26,580; 10,000 x 100,000 / 100,000 = 10,000.
    assert_eq!(value(&r, "foreign_general_excluded_tax_reduction"), 36580);
    // The same sources with both owners moved to the taxpayer reduce the
    // taxpayer's fraction only, and no longer reconcile with the spouse's pay.
    let mut wrong = d.clone();
    wrong.credits.as_mut().unwrap().foreign_tax =
        employment_credit(&[(Owner::Taxpayer, 150000, 30000), (Owner::Taxpayer, 100000, 10000)]);
    let r = estimate(&wrong);
    assert_eq!(r.state, ResultState::Unsupported);
    // One spouse's record alone: the other's pay is taxable.
    let mut one = d.clone();
    one.screening.as_mut().unwrap().foreign_earned.pop();
    one.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    let r = supported(&one);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(return_line(&r, "1h"), 25_000_000);
}

// The physical presence test: 329 days fail with a note, 330 qualify, and the
// 12-month window ends starts_on + 364 days.
#[test]
fn the_330_day_boundary_and_the_12_month_window() {
    let presence = |days: i32| {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(110000)));
        let mut claim = foreign_claim(Owner::Taxpayer);
        claim.qualification =
            ForeignQualification::Presence { starts_on: 20260302, foreign_days: days };
        set_claim(&mut d, claim);
        d
    };
    let r = supported(&presence(330));
    assert_eq!(value(&r, "foreign_earned_taxpayer_days"), 305);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 110000);
    assert!(!noted(&r, "WARN_FOREIGN_PRESENCE_TEST_NOT_MET"));
    let r = supported(&presence(329));
    assert!(noted(&r, "WARN_FOREIGN_PRESENCE_TEST_NOT_MET"));
    assert_eq!(value(&r, "foreign_earned_exclusion"), 0);
    assert_eq!(value(&r, "agi"), 110000);
    // The note names the day count (the person's name, then the count).
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "WARN_FOREIGN_PRESENCE_TEST_NOT_MET")
        .unwrap();
    assert_eq!(note.display.args[1], MessageArg::Number(329));
    // The client rule and its server twin: more than 365 days is refused.
    let form = Form2555 {
        qualification: ForeignQualification::Presence { starts_on: 20260302, foreign_days: 366 },
        tax_home: true,
        revocation: crate::ExclusionRevocation::None,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        exclusion: true,
        housing: ForeignHousing::None,
    };
    assert_eq!(form.rule(), Some("FOREIGN_PRESENCE_DAYS_ABOVE_PERIOD"));
    // The window is starts_on + 364 days: a stay on the last day is inside,
    // the next day is not (2026-03-02 through 2027-03-01 spans into 2027, so
    // housing stays are bound by 2026 first; use a 2025 start for the edge).
    let mut early = Form2555 {
        qualification: ForeignQualification::Presence { starts_on: 20250302, foreign_days: 340 },
        tax_home: true,
        revocation: crate::ExclusionRevocation::None,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        exclusion: true,
        housing: housing_claim(10000, ForeignHousingLocation::General, None),
    };
    // 2025-03-02 + 364 = 2026-03-01: a stay through 2026-03-01 is inside.
    if let ForeignHousing::Claim { stays, .. } = &mut early.housing {
        stays[0].starts_on = 20260101;
        stays[0].ends_on = 20260301;
    }
    assert_eq!(early.rule(), None);
    if let ForeignHousing::Claim { stays, .. } = &mut early.housing {
        stays[0].ends_on = 20260302;
    }
    assert_eq!(early.rule(), Some("FOREIGN_HOUSING_STAY_OUTSIDE_PERIOD"));
    // Period start edges.
    for (start, ok) in [(20250101, false), (20250102, true), (20261231, true), (20270101, false)] {
        let form = Form2555 {
            qualification: ForeignQualification::Presence { starts_on: start, foreign_days: 330 },
            tax_home: true,
            revocation: crate::ExclusionRevocation::None,
            prior_year_pay: 0,
            prior_year_history: None,
            multi_year_pay: vec![],
            exclusion: true,
            housing: ForeignHousing::None,
        };
        assert_eq!(form.rule().is_none(), ok, "{start}");
    }
}

#[test]
fn residence_dates_need_a_whole_calendar_year() {
    let residence = |began, ended| Form2555 {
        qualification: ForeignQualification::Residence {
            began,
            ended,
            basis: crate::ResidenceBasis::Citizen,
            nonresident_statement: false,
        },
        tax_home: true,
        revocation: crate::ExclusionRevocation::None,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        exclusion: true,
        housing: ForeignHousing::None,
    };
    assert_eq!(residence(20240901, None).rule(), None);
    // Departure year: began 2023-06-01, ended 2026-06-30 includes all of 2024.
    assert_eq!(residence(20230601, Some(20260630)).rule(), None);
    assert_eq!(residence(20260301, Some(20261130)).rule(), Some("FOREIGN_RESIDENCE_NO_FULL_YEAR"));
    assert_eq!(residence(20270101, None).rule(), Some("FOREIGN_RESIDENCE_BEGAN_AFTER_RETURN_YEAR"));
    assert_eq!(
        residence(20230101, Some(20251231)).rule(),
        Some("FOREIGN_RESIDENCE_ENDED_BEFORE_RETURN_YEAR")
    );
    assert_eq!(
        residence(20250601, Some(20250101)).rule(),
        Some("FOREIGN_RESIDENCE_ENDED_BEFORE_RETURN_YEAR")
    );
    // The departure-year exclusion prorates by days: 181 x 132,900 / 365.
    let mut d = empty_return(FederalStatus::Single);
    let mut w = wage(0, Owner::Taxpayer, 8_000_000, 0);
    if let FederalIncome::Wage { data } = &mut w.income {
        data.abroad = 8_000_000;
    }
    add(&mut d, w);
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20230601,
        ended: Some(20260630),
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    set_claim(&mut d, claim);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_taxpayer_days"), 181);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 65904);
    assert_eq!(value(&r, "agi"), 14096);
}

#[test]
fn negative_answers_take_no_exclusion_and_a_record_without_income_claims_nothing() {
    let base = |change: fn(&mut ForeignEarnedClaim)| {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(100000)));
        let mut claim = foreign_claim(Owner::Taxpayer);
        change(&mut claim);
        set_claim(&mut d, claim);
        estimate(&d)
    };
    // Yes: the exclusion.
    let r = base(|_| {});
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100000);
    // No: no exclusion, silently.
    let r = base(|c| c.tax_home = false);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 0);
    assert_eq!(return_line(&r, "9"), 10_000_000);
    let r = base(|c| {
        c.qualification = ForeignQualification::Residence {
            began: 20200101,
            ended: None,
            basis: crate::ResidenceBasis::Citizen,
            nonresident_statement: true,
        }
    });
    assert!(noted(&r, "WARN_FOREIGN_RESIDENCE_STATEMENT"));
    // An unconfirmed housing claim is left out and the exclusion stays.
    let r = base(|c| {
        c.housing = housing_claim(50000, ForeignHousingLocation::General, None);
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Adverse { confirmed: false };
        }
    });
    assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"));
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100000);
    // Housing only, left out: nothing is claimed, the record goes.
    let r = base(|c| {
        c.exclusion = false;
        c.housing = housing_claim(50000, ForeignHousingLocation::General, None);
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Adverse { confirmed: false };
        }
    });
    assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"));
    assert_eq!(return_line(&r, "9"), 10_000_000);
    // No foreign earned income and no carryover: the record claims nothing,
    // so the earned income credit is not denied for it.
    let mut d = complete(FederalStatus::Single);
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert!(noted(&r, "WARN_FORM_2555_NO_FOREIGN_INCOME"));
    assert_eq!(r, {
        let mut none = d.clone();
        none.screening.as_mut().unwrap().foreign_earned.clear();
        let mut expected = supported(&none);
        expected.review_notes = r.review_notes.clone();
        expected
    });
}

#[test]
fn form_2106_community_split_and_other_refusals_are_named() {
    // A Form 2106 of the same person.
    let mut d = london_banker();
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    add(&mut d, wage(0, Owner::Taxpayer, 5_000_000, 0));
    let job = d.income.last().unwrap().id;
    d.adjustments.as_mut().unwrap().employee_expenses.push(EmployeeExpenseClaim {
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner: Owner::Taxpayer,
        occupation: "Musician".into(),
        capacity: EmployeeCapacity::PerformingArtist,
        employers: vec![job],
        local_transportation: 0,
        travel: 100_000,
        other: 0,
        meals: 0,
        reimbursements: 0,
        mileage: Vec::new(),
        vehicle_assets: Vec::new(),
    });
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_FORM_2555_LINE_44_EMPLOYEE_EXPENSES"),
        "{r:?}"
    );
    // Owner SEP or health insurance deductions with a business run abroad.
    let mut health = dubai_consultant();
    if let FederalIncome::Business { data } = &mut health.income[0].income {
        data.retirement = BusinessRetirement::Sep { contribution: 100000, rate: 2500 };
    }
    let r = estimate(&health);
    assert!(r.issues.iter().any(|i| i.section == "Foreign business exclusion"), "{r:?}");
}

// ---- The pages ------------------------------------------------------------

// A topic page as the review opens it.
fn page(s: &mut Interview, topic: TaxSection) -> P {
    match from_review(s, topic) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the {topic:?} topic, got {other:?}"),
    }
}
fn shown(s: &mut Interview, topic: TaxSection) -> Vec<IncomeDocumentItem> {
    let P::SaveIncomes { documents, .. } = page(s, topic) else { unreachable!() };
    documents.documents
}
// Saves the topic with only these (owner, document) rows.
fn save_rows(s: &mut Interview, topic: TaxSection, rows: Vec<(Owner, IncomeDocument)>) -> Reply {
    let items =
        rows.into_iter().map(|(owner, document)| unanswered_document(owner, 0, document)).collect();
    let request = save_topic(&s.draft.clone(), topic, items);
    send(s, request)
}
fn form_2555(qualification: ForeignQualification) -> Form2555 {
    Form2555 {
        qualification,
        tax_home: true,
        revocation: crate::ExclusionRevocation::None,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        exclusion: true,
        housing: housing_claim(30000, ForeignHousingLocation::General, None),
    }
}

// The Work abroad page: the kinds map onto the records and reopen unchanged.
#[test]
fn work_abroad_page_round_trips_and_checks_both_sides() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let presence = ForeignQualification::Presence { starts_on: 20260101, foreign_days: 340 };
    let residence = ForeignQualification::Residence {
        began: 20230101,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    let pay = IncomeDocument::ForeignPay(ForeignPay {
        employer: "Example Pte Ltd".into(),
        wages: 5_000_000,
    });
    accepted(&save_rows(
        &mut s,
        TaxSection::WorkAbroad,
        vec![
            (Owner::Taxpayer, IncomeDocument::Form2555(form_2555(presence.clone()))),
            (Owner::Spouse, IncomeDocument::Form2555(form_2555(residence.clone()))),
            (Owner::Taxpayer, pay),
        ],
    ));
    let stored = &s.draft.screening.as_ref().unwrap().foreign_earned;
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].owner, Owner::Taxpayer);
    assert_eq!(stored[1].qualification, residence);
    let foreign: Vec<_> = s
        .draft
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::ForeignEmployment { .. }))
        .collect();
    assert_eq!(foreign.len(), 1);
    assert_eq!(foreign[0].label, "Example Pte Ltd");
    assert_eq!(crate::interview::document_amount(&foreign[0].income, false), Some(5_000_000));
    // The page reopens with the same kinds and saving them unchanged keeps
    // the return.
    let rows = shown(&mut s, TaxSection::WorkAbroad);
    assert_eq!(rows.len(), 3);
    let reopened: Vec<_> = rows
        .iter()
        .filter_map(|item| match &item.document {
            IncomeDocument::Form2555(f) => Some((item.owner, f.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        reopened,
        vec![(Owner::Taxpayer, form_2555(presence.clone())), (Owner::Spouse, form_2555(residence)),]
    );
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let request = page(&mut again, TaxSection::WorkAbroad);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
    // The topic is ticked and always shown with entries.
    assert!(
        crate::interview::with_ticks(saved).checklist.income & IncomeTopics::WORK_ABROAD.0 != 0
    );
    // Both sides: a second Form 2555 for one person, a person with no choice,
    // dates outside the period, and a spouse off a joint return.
    rejected_with(
        save_rows(
            &mut s,
            TaxSection::WorkAbroad,
            vec![
                (Owner::Taxpayer, IncomeDocument::Form2555(form_2555(presence.clone()))),
                (Owner::Taxpayer, IncomeDocument::Form2555(form_2555(presence.clone()))),
            ],
        ),
        "FORM_2555_TWICE",
    );
    let mut nothing = form_2555(presence.clone());
    nothing.exclusion = false;
    nothing.housing = ForeignHousing::None;
    rejected_with(
        save_rows(
            &mut s,
            TaxSection::WorkAbroad,
            vec![(Owner::Taxpayer, IncomeDocument::Form2555(nothing))],
        ),
        "FORM_2555_NOTHING_CLAIMED",
    );
    let mut outside = form_2555(presence);
    if let ForeignHousing::Claim { stays, .. } = &mut outside.housing {
        stays[0].ends_on = 20270105;
    }
    rejected_with(
        save_rows(
            &mut s,
            TaxSection::WorkAbroad,
            vec![(Owner::Taxpayer, IncomeDocument::Form2555(outside))],
        ),
        "FOREIGN_HOUSING_STAY_OUTSIDE_RETURN_YEAR",
    );
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let reply = current_document_refusal(
        "income-source-10278",
        &mut single,
        vec![owned(Owner::Spouse, 0, IncomeDocument::Form2555(form_2555(residence_since_2023())))],
        "MESSAGE_FORM_2555_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "[documents/documents/0/owner]",
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}
fn residence_since_2023() -> ForeignQualification {
    ForeignQualification::Residence {
        began: 20230101,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    }
}

// The W-2 and business marks.
#[test]
fn w2_abroad_amount_and_business_source_round_trip() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut w = w2(6_000_000);
    w.answers.abroad = 4_000_000;
    accepted(&save_rows(
        &mut s,
        TaxSection::Wages,
        vec![(Owner::Taxpayer, IncomeDocument::W2(w.clone()))],
    ));
    let wage = s
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Wage { data } => Some(data.clone()),
            _ => None,
        })
        .expect("a W-2 record");
    assert_eq!(wage.abroad, 4_000_000);
    // The answer reopens with the boxes: it is not an import target.
    let rows = shown(&mut s, TaxSection::Wages);
    let IncomeDocument::W2(reopened) = &rows[0].document else { panic!("a W-2") };
    assert_eq!(reopened.answers.abroad, 4_000_000);
    // Above box 1 is refused on both sides (the server twin here).
    w.answers.abroad = 6_000_001;
    rejected_with(
        save_rows(&mut s, TaxSection::Wages, vec![(Owner::Taxpayer, IncomeDocument::W2(w))]),
        "W2_ABROAD_ABOVE_WAGES",
    );
    // A business run abroad reopens with its source.
    let mut b = business(100_000);
    b.more.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    accepted(&save_rows(
        &mut s,
        TaxSection::SelfEmployment,
        vec![(Owner::Taxpayer, IncomeDocument::Business(b))],
    ));
    let rows = shown(&mut s, TaxSection::SelfEmployment);
    let IncomeDocument::Business(reopened) = &rows[0].document else { panic!("a business") };
    assert_eq!(
        reopened.more.source,
        BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false }
    );
}

// Walk-owned data: the spouse's Form 2555 goes with the joint return, and
// the topic's tick owns the person's record and the foreign pay.
#[test]
fn leaving_a_joint_return_erases_the_spouses_form_2555() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(50000)));
    let mut spouse = income(foreign_pay(40000));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    let mut d = crate::interview::with_ticks(d);
    let mut stored = d.clone();
    assert!(crate::interview::prune(&mut stored).removed.is_empty());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    let result = crate::interview::prune(&mut d);
    let claims = &d.screening.as_ref().unwrap().foreign_earned;
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].owner, Owner::Taxpayer);
    assert!(result.removed.iter().any(|item| item.key() == "SAVING_KIND_FORM_2555"), "{result:?}");
}

// The 2026 housing amounts are 16% and 30% of the year constant.
#[test]
fn housing_amounts_are_derived_from_the_exclusion_amount() {
    use crate::year_constants::*;
    assert_eq!(FOREIGN_BASE_HOUSING_DOLLARS, 21264);
    assert_eq!(FOREIGN_BASE_HOUSING_DAILY_CENTS, 5826);
    assert_eq!(FOREIGN_HOUSING_LIMIT_DOLLARS, 39870);
    assert_eq!(FOREIGN_HOUSING_LIMIT_DAILY_CENTS, 10923);
}

// Pub. 54 mixed example: housing amount 18,000, foreign earned income
// 100,000 of which 50,000 wages: the employer share of line 35 excludes
// 9,000 (the rest is the self-employed deduction, limited on line 47).
#[test]
fn pub_54_mixed_housing_example_splits_by_the_employer_share() {
    assert_eq!(crate::foreign_housing::employer_exclusion(18000, 50000, 100000), 9000);
}

#[test]
fn unit47_foreign_pay_comparison_preserves_award_path_above_i64() {
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    let mut pay = foreign_pay(0);
    let FederalIncome::ForeignEmployment { data } = &mut pay else { unreachable!() };
    data.wages = i64::MAX;
    add(&mut draft, income(pay));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.prior_year_pay = i64::MAX;
    claim.multi_year_pay.push(MultiYearPay {
        description: "Award".into(),
        amount: 1,
        months_total: 1,
        months_in_return_year: 1,
        months_in_prior_year: 0,
    });
    set_claim(&mut draft, claim);
    let (owner, issue) = crate::foreign_housing::source_pay_issue(&draft).unwrap();
    assert_eq!(owner, Owner::Taxpayer);
    assert_eq!(issue.key, "ISSUE_FOREIGN_AWARDS_ABOVE_PAY");
    assert_eq!(issue.field.unwrap().square(), "[multi_year_pay]");
    // The source total itself can exceed i64 before source admission, too.
    add(&mut draft, income(foreign_pay(1)));
    assert!(crate::foreign_housing::source_pay_issue(&draft).is_none());
}

#[test]
fn unit47_foreign_housing_share_comparison_preserves_its_path_above_i64() {
    let mut form = form_2555(residence());
    let ForeignHousing::Claim { expenses, household, .. } = &mut form.housing else {
        unreachable!()
    };
    *expenses = i64::MAX;
    *household = ForeignHousingHousehold::SeparateReturnShared {
        total_expenses: i64::MAX,
        spouse_expenses: 1,
        confirmed: true,
    };
    let issue = form.issue().unwrap();
    assert_eq!(issue.key, "HOUSING_SHARES_ABOVE_TOTAL");
    assert_eq!(issue.field.unwrap().square(), "[housing/expenses]");
}
