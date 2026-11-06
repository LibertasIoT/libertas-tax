use super::*;
fn provider(paid: i64) -> CareProvider {
    CareProvider {
        id: 0,
        name: "Synthetic daycare".into(),
        address: "1 Example Way".into(),
        identification: ProviderIdentification::Organization,
        household_employee: false,
        paid: paid * 100,
        relationship: ProviderRelationship::NotRelated,
    }
}
fn care_return(status: FederalStatus, wages: [i64; 2], costs: &[i64]) -> FederalDraft {
    let mut d = complete(status);
    d.income = vec![wage(1, Owner::Taxpayer, wages[0] * 100, 0)];
    if status == FederalStatus::Joint {
        d.income.push(wage(2, Owner::Spouse, wages[1] * 100, 0));
    }
    d.next_id = 10;
    for cost in costs {
        add_child(&mut d);
        d.dependents.last_mut().unwrap().set_care_expenses(cost * 100);
    }
    let total = costs.iter().sum();
    if total > 0 {
        d.credits.as_mut().unwrap().care.providers.push(provider(total));
        d.checklist.paid |= PaidTopics::CARE.0;
    }
    if status == FederalStatus::Separate {
        d.credits.as_mut().unwrap().care.filing =
            CareFilingStatus::Separate { spouse_earned_income: Some(wages[1] * 100) };
    }
    d
}
fn box10(d: &mut FederalDraft, amount: i64, already: i64, forfeit: i64, limit: i64) {
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        let mut data = edit_wage(data);
        data.care_benefits = amount * 100;
    }
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: limit * 100 };
    d.credits.as_mut().unwrap().care.benefits = vec![CareBenefitAnswers {
        wage: 1,
        in_box_1: Some(already * 100),
        forfeited: Some(forfeit * 100),
    }];
}
fn student(d: &mut FederalDraft, owner: Owner, months: Vec<i64>, earned: i64) {
    let p = d.people.as_mut().unwrap();
    let p = if owner == Owner::Taxpayer { &mut p.taxpayer } else { p.spouse.as_mut().unwrap() };
    p.full_time_student = true;
    d.credits.as_mut().unwrap().care.filers.push(CareFiler {
        owner,
        situation: CareFilerSituation::Student { months, earned_in_months: Some(earned * 100) },
    });
}
fn prior() -> PriorYearCare {
    PriorYearCare {
        two_or_more: false,
        prior_line_3: 200_000,
        paid_in_return_year: 60_000,
        prior_benefits: 0,
        prior_earned_smaller: 3_800_000,
        prior_line_6: 200_000,
        prior_agi: 3_800_000,
        injured_claimants: None,
    }
}
#[test]
fn care_completion_f1_w2_benefits_reduce_credit_once() {
    for (amount, already, plan, expected_exclusion, credit) in
        [(5000, 0, 7500, 5000, 350), (5000, 0, 5000, 5000, 350), (7500, 500, 7000, 7000, 0)]
    {
        let mut d = care_return(FederalStatus::Joint, [60000, 40000], &[5000, 3000]);
        box10(&mut d, amount, already, 0, plan);
        let r = supported(&d);
        assert_eq!(value(&r, "care_benefits_excluded"), expected_exclusion);
        assert_eq!(value(&r, "care_benefits_taxable"), 0);
        assert_eq!(value(&r, "care_credit"), credit);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn care_completion_f2_student_months_and_five_month_source() {
    for (children, summer, spouse_earned, base, credit) in
        [(1, 0, 2250, 2250, 788), (1, 3000, 5250, 3000, 1050), (2, 0, 4500, 4000, 1400)]
    {
        let mut d = care_return(
            FederalStatus::Joint,
            [50000, summer],
            if children == 1 { &[4000] } else { &[4000, 0] },
        );
        student(&mut d, Owner::Spouse, vec![1, 2, 3, 4, 5, 9, 10, 11, 12], 0);
        let r = supported(&d);
        assert_eq!(value(&r, "care_earned_spouse"), spouse_earned);
        assert_eq!(value(&r, "care_expenses_smallest"), base);
        assert_eq!(value(&r, "care_credit"), credit);
    }
    let mut d = care_return(FederalStatus::Joint, [0, 0], &[5000]);
    student(&mut d, Owner::Taxpayer, (1..=12).collect(), 0);
    student(&mut d, Owner::Spouse, (1..=12).collect(), 0);
    let r = supported(&d);
    assert_eq!(value(&r, "care_expenses_smallest"), 0);
    assert_eq!(value(&r, "care_credit_before_prior"), 0);
    let mut d = care_return(FederalStatus::Joint, [50000, 0], &[4000]);
    student(&mut d, Owner::Spouse, vec![1, 2, 3, 4], 0);
    assert_eq!(d.credits.as_ref().unwrap().care.rule(&d), Some("CARE_STUDENT_NEEDS_FIVE_MONTHS"));
}
#[test]
fn care_completion_f3_prior_year_worksheet_and_line_a() {
    for status in [FederalStatus::Single, FederalStatus::Separate] {
        let mut d = care_return(status, [40000, 0], &[3500]);
        d.credits.as_mut().unwrap().care.prior_year = Some(prior());
        let r = supported(&d);
        assert_eq!(value(&r, "prior_year_care_credit"), 138);
        assert_eq!(
            value(&r, "care_credit_total"),
            if status == FederalStatus::Single { 1248 } else { 138 }
        );
        d.credits.as_mut().unwrap().care.prior_year.as_mut().unwrap().prior_line_6 = 260_000;
        assert!(!d.valid_stored()); // Historical inconsistent line3=2000/line6=2600.
        d.credits.as_mut().unwrap().care.prior_year.as_mut().unwrap().prior_line_3 = 260_000;
        assert_eq!(value(&supported(&d), "prior_year_care_credit"), 92);
        let p = d.credits.as_mut().unwrap().care.prior_year.as_mut().unwrap();
        p.prior_line_3 = 300_000;
        p.prior_line_6 = 300_000;
        assert_eq!(value(&supported(&d), "prior_year_care_credit"), 0);
    }
}
#[test]
fn care_completion_f4_forfeiture_and_carry_forward() {
    for (forfeit, taxable) in [(400, 0), (0, 400)] {
        let mut d = care_return(FederalStatus::Head, [45000, 0], &[2600]);
        box10(&mut d, 3000, 0, forfeit, 7500);
        let r = supported(&d);
        assert_eq!(value(&r, "care_form2441_line15"), 3000 - forfeit);
        assert_eq!(value(&r, "care_benefits_excluded"), 2600);
        assert_eq!(value(&r, "care_benefits_taxable"), taxable);
        assert_eq!(value(&r, "care_form2441_line30"), 0);
        assert_eq!(value(&r, "care_form2441_line31"), 0);
        assert_eq!(value(&r, "care_credit"), 0);
    }
}
#[test]
fn care_completion_f5_incapable_filer_is_qualifying_person() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for same_home in [true, false] {
            let wages = if owner == Owner::Spouse { [29000, 0] } else { [0, 29000] };
            let mut d = care_return(FederalStatus::Joint, wages, &[]);
            let p = d.people.as_mut().unwrap();
            if owner == Owner::Taxpayer {
                p.taxpayer.incapable_self_care = true;
            } else {
                p.spouse.as_mut().unwrap().incapable_self_care = true;
            }
            let c = &mut d.credits.as_mut().unwrap().care;
            c.providers = vec![provider(3300)];
            c.filers = vec![CareFiler {
                owner,
                situation: CareFilerSituation::Incapable {
                    months: (2..=12).collect(),
                    earned_in_months: Some(0),
                    same_home_over_half_year: same_home,
                    care_paid: 330_000,
                },
            }];
            let r = supported(&d);
            assert_eq!(value(&r, "care_credit_before_prior"), if same_home { 1183 } else { 0 });
        }
    }
}
#[test]
fn care_completion_f6_provider_relationship_and_due_diligence() {
    let mut d = care_return(FederalStatus::Single, [30000, 0], &[3500]);
    let c = &mut d.credits.as_mut().unwrap().care;
    let mut mother = provider(2000);
    mother.name = "Synthetic parent".into();
    mother.relationship = ProviderRelationship::Dependent;
    c.providers = vec![mother, provider(1500)];
    let r = supported(&d);
    assert_eq!(value(&r, "care_credit_before_prior"), 630);
    assert!(noted(&r, "WARN_CARE_RELATED_PROVIDER_LEFT_OUT"));
    d.credits.as_mut().unwrap().care.providers[1].identification =
        ProviderIdentification::NotObtained;
    let r = supported(&d);
    assert_eq!(value(&r, "care_credit_before_prior"), 0);
    assert!(noted(&r, "WARN_CARE_PROVIDER_NOT_IDENTIFIED_LEFT_OUT"));
    d.credits.as_mut().unwrap().care.providers[1].identification = ProviderIdentification::Refused;
    let r = supported(&d);
    assert_eq!(value(&r, "care_credit_before_prior"), 630);
    assert!(noted(&r, "NOTE_CARE_PROVIDER_STATEMENT_ATTACH"));
    d.credits.as_mut().unwrap().care.providers[0].relationship = ProviderRelationship::NotRelated;
    assert_eq!(value(&supported(&d), "care_credit_before_prior"), 1260);
}
#[test]
fn care_completion_f7_separate_spouse_earned_and_half_limit() {
    for (spouse, excluded, taxable) in [(Some(2_000_000), 3750, 1250), (None, 0, 5000)] {
        let mut d = care_return(FederalStatus::Separate, [38000, 0], &[5000]);
        box10(&mut d, 5000, 0, 0, 7500);
        d.credits.as_mut().unwrap().care.filing =
            CareFilingStatus::Separate { spouse_earned_income: spouse };
        let r = supported(&d);
        assert_eq!(value(&r, "care_benefits_excluded"), excluded);
        assert_eq!(value(&r, "care_benefits_taxable"), taxable);
        assert_eq!(value(&r, "care_credit_before_prior"), 0);
        if spouse.is_none() {
            assert!(noted(&r, "WARN_CARE_SPOUSE_EARNINGS_ASSUMED_ZERO"));
        }
    }
}
#[test]
fn care_completion_f8_benefits_without_qualifying_people_taxed() {
    let mut d = care_return(FederalStatus::Single, [40000, 0], &[]);
    box10(&mut d, 2000, 0, 0, 5000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line16"), 0);
    assert_eq!(value(&r, "care_benefits_excluded"), 0);
    assert_eq!(value(&r, "care_benefits_taxable"), 2000);
    assert_eq!(value(&r, "care_credit"), 0);
}

// Independently worked source deduction: $50,000 business receipts less
// $3,000 Form2441line24 leaves $47,000 ScheduleC/F profit. ScheduleSE4a
// round(47000*.9235)=43405; tax round(43405*.153)=6641; half=3321.
#[test]
fn care_completion_own_business_deduction_reaches_se_once_without_saved_mutation() {
    for farm in [false, true] {
        let mut d = care_return(FederalStatus::Single, [0, 0], &[4000]);
        let id = d.next_id;
        add(&mut d, income(if farm { raised_farm(50000, 0) } else { cash_business(50000) }));
        d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        d.credits.as_mut().unwrap().care.own_business = vec![CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: if farm {
                CareBenefitSource::Farm { farm: id }
            } else {
                CareBenefitSource::Business { business: id }
            },
            received: 300_000,
        }];
        let original = d.clone();
        let r = supported(&d);
        assert_eq!(value(&r, "care_form2441_line24"), 3000);
        assert_eq!(
            value(
                &r,
                &alloc::format!("{}_{id}_net_profit", if farm { "farm" } else { "business" })
            ),
            47000
        );
        assert_eq!(value(&r, "self_employment_tax"), 6641);
        assert_eq!(value(&r, "agi"), 43679);
        assert_eq!(value(&r, "care_benefits_taxable"), 0);
        assert_eq!(value(&r, "care_credit_before_prior"), 0);
        assert_eq!(d, original);
        assert_eq!(restart(d.clone()).draft, original);
    }
}
#[test]
fn care_completion_partnership_selected_source_owns_nonpassive_deduction() {
    let mut d = care_return(FederalStatus::Single, [40000, 0], &[4000]);
    let first = d.next_id;
    let mut partner = ordinary_partner_source(50000, true);
    partner.qbi = Some(partner_qbi_statement(50000));
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 5_000_000 });
    partner.qbi_records.qualified_se_gross = 5_000_000;
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(5_000_000);
    add(&mut d, income(FederalIncome::Partnership { data: partner.clone() }));
    let second = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    d.credits.as_mut().unwrap().care.own_business = vec![CareOwnBusinessBenefit {
        owner: Owner::Taxpayer,
        source: CareBenefitSource::Partnership { partnership: second },
        received: 300_000,
    }];
    let original = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line24"), 3000);
    assert_eq!(value(&r, &alloc::format!("partnership_{second}_care_deduction")), 3000);
    assert!(!r.lines.iter().any(|l| l.id == alloc::format!("partnership_{first}_care_deduction")));
    // Two K1s retain their own income; the return has one $3,000 source debit.
    assert_eq!(value(&r, "rental_income"), 97000);
    // Independent Schedule SE oracle: (97000 * .9235) * .153 rounds
    // to13706; half deduction6853; AGI40000 +97000 -6853=130147.
    // QBI97000 -6853=90147 and its20% deduction rounds18029.
    assert_eq!(value(&r, "self_employment_tax"), 13706);
    assert_eq!(value(&r, "agi"), 130147);
    assert_eq!(value(&r, "qbi_deduction"), 18029);
    assert_eq!(d, original);
    assert_eq!(restart(d.clone()).draft, original);
    let care = &mut d.credits.as_mut().unwrap().care;
    care.own_business[0].source = CareBenefitSource::Partnership { partnership: 99999 };
    assert_eq!(care.rule(&original), Some("CARE_BUSINESS_SOURCE_INVALID"));
}

fn open_care(s: &mut Interview) -> P {
    let request = P::Navigate {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        from: TaxSection::Review as i32,
        section: TaxSection::Care.into(),
        navigation: NavigationAvailability::default(),
    };
    match send(s, request) {
        reply @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
            crate::tests::cross::interview::care_editor(reply)
        }
        other => panic!("Care editor: {other:?}"),
    }
}
fn care_saved(reply: &Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

fn validation_audit_utf8_text(unit: &str, length: usize) -> alloc::string::String {
    let width = unit.len();
    unit.repeat(length / width) + &"x".repeat(length % width)
}

#[test]
fn validation_audit_f05_care_provider_utf8_boundaries_save_and_reopen() {
    let names = ["N", "é", "𐐀", "e\u{301}"]
        .into_iter()
        .flat_map(|unit| [1, 79, 80].map(|length| validation_audit_utf8_text(unit, length)));
    for name in names {
        let addresses = ["A", "é", "𐐀", "e\u{301}"]
            .into_iter()
            .flat_map(|unit| [1, 159, 160].map(|length| validation_audit_utf8_text(unit, length)));
        for address in addresses {
            assert!((1..=80).contains(&name.len()));
            assert!((1..=160).contains(&address.len()));
            let mut session =
                Interview::new(TEST_USER, care_return(FederalStatus::Single, [60_000, 0], &[]));
            let mut page = open_care(&mut session);
            let P::SaveCare { value, .. } = &mut page else { panic!("expected the care page") };
            assert!(value.prior_year.is_none());
            assert!(value.unpaid_return_year_care.is_none());
            let mut row = provider(0);
            row.name = name.clone();
            row.address = address.clone();
            value.providers = vec![row];
            care_saved(&send(&mut session, page));
            let stored = session.draft.credits.as_ref().unwrap().care.providers[0].clone();
            assert!(stored.id > 0);
            assert_eq!((&stored.name, &stored.address), (&name, &address));
            supported(&session.draft);
            let reopened = open_care(&mut session);
            let P::SaveCare { value, .. } = &reopened else { unreachable!() };
            assert_eq!(value.providers[0], stored);
            let revision = session.draft.revision;
            care_saved(&send(&mut session, reopened));
            assert_eq!(session.draft.revision, revision);
        }
    }
}

#[test]
fn validation_audit_f05_care_provider_over_utf8_limits_rejects_unchanged() {
    let valid_name: alloc::string::String = "Synthetic daycare".into();
    let valid_address: alloc::string::String = "1 Example Way".into();
    for (name, address) in [
        ("".into(), valid_address.clone()),
        (valid_name.clone(), "".into()),
        ("N".repeat(81), valid_address.clone()),
        (validation_audit_utf8_text("é", 81), valid_address.clone()),
        (validation_audit_utf8_text("𐐀", 81), valid_address.clone()),
        (valid_name.clone(), "A".repeat(161)),
        (valid_name.clone(), validation_audit_utf8_text("é", 161)),
        (valid_name.clone(), validation_audit_utf8_text("𐐀", 161)),
    ] {
        let mut session =
            Interview::new(TEST_USER, care_return(FederalStatus::Single, [60_000, 0], &[]));
        let mut page = open_care(&mut session);
        let P::SaveCare { value, .. } = &mut page else { panic!("expected the care page") };
        value.providers = vec![provider(0)];
        care_saved(&send(&mut session, page));
        let mut reopened = open_care(&mut session);
        let P::SaveCare { value, .. } = &mut reopened else { unreachable!() };
        value.providers[0].name = name.clone();
        value.providers[0].address = address.clone();
        let before = session.draft.clone();
        let dirty = session.dirty;
        // 2026-10-15 D-13: required text still rejects empty answers; the
        // same six historical overlength inputs now save without truncation.
        if name.is_empty() || address.is_empty() {
            crate::tests::cross::interview::rejected_with(
                send(&mut session, reopened),
                "CARE_INVALID_RECORD",
            );
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
            let P::SaveCare { value, .. } = open_care(&mut session) else { unreachable!() };
            assert_eq!(value.providers[0].name, valid_name);
            assert_eq!(value.providers[0].address, valid_address);
        } else {
            care_saved(&send(&mut session, reopened));
            let saved = session.draft.clone();
            let reopened = open_care(&mut session);
            let P::SaveCare { value, .. } = &reopened else { unreachable!() };
            assert_eq!(value.providers[0].name, name);
            assert_eq!(value.providers[0].address, address);
            care_saved(&send(&mut session, reopened));
            assert_eq!(session.draft, saved);
        }
    }
}

#[test]
fn care_completion_public_editor_identity_reopen_stale_and_stored_codec() {
    let mut d = care_return(FederalStatus::Joint, [60000, 40000], &[5000, 3000]);
    box10(&mut d, 5000, 0, 0, 7500);
    let mut s = Interview::new(TEST_USER, d);
    let mut page = open_care(&mut s);
    let P::SaveCare { value, care_months, .. } = &mut page else { panic!() };
    assert_eq!(care_months.iter().map(|m| m.id).collect::<Vec<_>>(), (1..=12).collect::<Vec<_>>());
    assert_eq!(value.qualifying_people.len(), 2);
    assert_eq!(value.benefits.len(), 1);
    value.providers[0].name = "Accepted provider".into();
    let stale = page.clone();
    let reply = send(&mut s, page);
    care_saved(&reply);
    assert_eq!(s.draft.credits.as_ref().unwrap().care.providers[0].name, "Accepted provider");
    let id = s.draft.credits.as_ref().unwrap().care.providers[0].id;
    assert!(id > 0);
    let accepted = s.draft.clone();
    let reply = current_source_refusal(
        "health-context-02",
        &mut s,
        stale,
        "PROBLEM_ACCEPTED_ANSWERS_CHANGED",
        None,
        SourceScope::Health,
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })));
    assert_eq!(s.draft, accepted);
    let data = native_draft_records(&accepted);
    let decoded = draft_from_native_records(accepted.tax_year, &data);
    assert_eq!(decoded, accepted);
    assert_eq!(native_draft_records(&decoded), data);
    let mut reopened = restart(accepted.clone());
    let P::SaveCare { value, .. } = open_care(&mut reopened) else { panic!() };
    assert_eq!(value.providers[0].id, id);
    assert_eq!(value.providers[0].name, "Accepted provider");
    let mut forged = open_care(&mut reopened);
    if let P::SaveCare { value, .. } = &mut forged {
        value.benefits[0].owner = Owner::Spouse;
    }
    let before = reopened.draft.clone();
    let reply = current_source_refusal(
        "health-context-03",
        &mut reopened,
        forged,
        "CARE_WAGE_SOURCE_INVALID",
        Some("[value/benefits/0/owner]"),
        SourceScope::Health,
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })));
    assert_eq!(reopened.draft, before);
}

#[test]
fn care_completion_plan_maximum_counts_each_plan_once_and_covers_grace_only() {
    let mut same = care_return(FederalStatus::Single, [60_000, 0], &[9_000]);
    box10(&mut same, 5_000, 0, 0, 5_000);
    same.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Taxpayer, used_in_return_year: 200_000 }];
    let result = supported(&same);
    assert_eq!(value(&result, "care_form2441_line15"), 7_000);
    assert_eq!(value(&result, "care_form2441_line21"), 5_000);
    assert_eq!(value(&result, "care_benefits_excluded"), 5_000);
    assert_eq!(value(&result, "care_benefits_taxable"), 2_000);
    // Two distinct plans each allow $2,000: the aggregate is $4,000,
    // regardless of whether the second benefit is a grace-period payment.
    same.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 400_000 };
    box10(&mut same, 2_000, 0, 0, 4_000);
    let result = supported(&same);
    assert_eq!(value(&result, "care_form2441_line21"), 4_000);
    assert_eq!(value(&result, "care_benefits_excluded"), 4_000);
    assert_eq!(value(&result, "care_benefits_taxable"), 0);
    if let FederalIncome::Wage { data } = &mut same.income[0].income {
        let mut data = edit_wage(data);
        data.care_benefits = 0;
    }
    same.credits.as_mut().unwrap().care.benefits.clear();
    same.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 100_000 };
    let result = supported(&same);
    assert_eq!(value(&result, "care_form2441_line12"), 0);
    assert_eq!(value(&result, "care_form2441_line21"), 1_000);
    assert_eq!(value(&result, "care_benefits_excluded"), 1_000);
    assert_eq!(value(&result, "care_benefits_taxable"), 1_000);
}

#[test]
fn care_completion_plan_maximum_public_required_answer_and_no_plan_transition() {
    let d = care_return(FederalStatus::Single, [60_000, 0], &[3_000]);
    let mut session = Interview::new(TEST_USER, d);
    let page = open_care(&mut session);
    let P::SaveCare { value: facts, .. } = &page else { panic!() };
    assert_eq!(facts.plan_maximum, CarePlanMaximum::NoBenefits);
    care_saved(&send(&mut session, page));
    // Preserve an entered plan maximum when genuine W-2 box 10 benefits are added.
    if let FederalIncome::Wage { data } = &mut session.draft.income[0].income {
        let mut data = edit_wage(data);
        data.care_benefits = 200_000;
    }
    session.draft.credits.as_mut().unwrap().care.plan_maximum =
        CarePlanMaximum::Plans { maximum: 150_000 };
    assert!(session.draft.valid_stored());
    let mut page = open_care(&mut session);
    let before = session.draft.clone();
    if let P::SaveCare { value, .. } = &mut page {
        value.plan_maximum = CarePlanMaximum::NoBenefits;
    }
    assert!(matches!(send(&mut session, page.clone()), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
    if let P::SaveCare { value, .. } = &mut page {
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 150_000 };
    }
    care_saved(&send(&mut session, page));
    assert_eq!(value(&supported(&session.draft), "care_benefits_excluded"), 1_500);
    assert_eq!(value(&supported(&session.draft), "care_benefits_taxable"), 500);
    if let FederalIncome::Wage { data } = &mut session.draft.income[0].income {
        let mut data = edit_wage(data);
        data.care_benefits = 0;
    }
    crate::interview::prune(&mut session.draft);
    assert!(session.draft.credits.as_ref().unwrap().care.benefits.is_empty());
    assert_eq!(
        session.draft.credits.as_ref().unwrap().care.plan_maximum,
        CarePlanMaximum::NoBenefits
    );
    let mut page = open_care(&mut session);
    if let P::SaveCare { value, .. } = &mut page {
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 150_000 };
    }
    let before = session.draft.clone();
    assert!(matches!(send(&mut session, page), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
}

#[test]
fn care_completion_explicit_answers_roundtrip_and_estimate_do_not_commit() {
    let mut d = care_return(FederalStatus::Separate, [38000, 0], &[1000]);
    box10(&mut d, 2000, 0, 0, 7500);
    d.credits.as_mut().unwrap().care.filing =
        CareFilingStatus::Separate { spouse_earned_income: Some(2_000_000) };
    d.credits.as_mut().unwrap().care.unpaid_return_year_care = Some(100_000);
    d.credits.as_mut().unwrap().care.prior_year = Some(prior());
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line16"), 2000);
    assert_eq!(value(&r, "care_benefits_excluded"), 2000);
    assert_eq!(value(&r, "prior_year_care_credit"), 138);
    let stored = native_draft_records(&d);
    let decoded = draft_from_native_records(d.tax_year, &stored);
    assert_eq!(decoded, d);
    assert_eq!(native_draft_records(&decoded), stored);
    let mut s = Interview::new(TEST_USER, d);
    let page = open_care(&mut s);
    let P::SaveCare { mut value, .. } = page.clone() else { panic!() };
    let accepted = s.draft.clone();
    value.unpaid_return_year_care = None;
    let request = estimate_care(&page, value.clone());
    let reply = crate::tests::support::client::income_pages::candidate_save_review(&mut s, request);
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. })), "{reply:?}");
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
    value.unpaid_return_year_care = Some(0);
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        &mut s,
        estimate_care(&page, value),
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(s.draft, accepted);
}

#[test]
fn care_completion_shared_and_exclusive_months_preserve_one_shared_owner() {
    let mut d = care_return(FederalStatus::Joint, [0, 0], &[4000]);
    student(&mut d, Owner::Taxpayer, vec![1, 2, 3, 4, 5], 0);
    student(&mut d, Owner::Spouse, vec![4, 5, 6, 7, 8], 0);
    let mut result = FederalResult::empty(0);
    let care = &d.credits.as_ref().unwrap().care;
    // Three exclusive months each; two common months assigned once. Equal
    // smaller-income endpoints retain the taxpayer tie order: 1250 / 750.
    assert_eq!(care.earned([0, 0], 1, &mut result), [1250, 750]);
    assert!(noted(&result, "WARN_CARE_DEEMED_INCOME_MONTHLY"));
    // Other-month actual earnings can make the spouse the useful common-month
    // owner: taxpayer has $2000 outside the selected months, spouse none.
    assert_eq!(care.earned([2000, 0], 1, &mut result), [2750, 1250]);
    // Swapping actual source earnings swaps the useful allocation, preserving
    // source-owner identity rather than an arbitrary manual employee share.
    assert_eq!(care.earned([0, 2000], 1, &mut result), [1250, 2750]);
    let mut duplicate = d.clone();
    if let CareFilerSituation::Student { months, .. } =
        &mut duplicate.credits.as_mut().unwrap().care.filers[0].situation
    {
        months.push(1);
    }
    assert!(!duplicate.valid_stored());
    let mut unavailable = d.clone();
    unavailable.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = false;
    assert!(!unavailable.valid_stored());
}

#[test]
fn care_completion_normal_w2_correction_and_deletion_prune_only_source_answers() {
    let mut d = care_return(FederalStatus::Single, [60000, 0], &[4000]);
    box10(&mut d, 3000, 500, 400, 7500);
    d.credits.as_mut().unwrap().care.prior_year = Some(prior());
    let mut s = Interview::new(TEST_USER, d);
    let accepted_care = s.draft.credits.as_ref().unwrap().care.clone();
    let mut wages = crate::tests::support::client::income_pages::open_document_center(&mut s);
    if let P::SaveDocuments { documents, .. } = &mut wages {
        let crate::WageDocument::W2(w2) = &mut documents.common.wages.documents[0].document else {
            panic!()
        };
        w2.care_benefits = Some(50_000); // old $500 + $400 answers no longer fit.
    }
    care_saved(&send(&mut s, wages));
    let care = &s.draft.credits.as_ref().unwrap().care;
    assert!(care.benefits.is_empty());
    assert_eq!(care.providers, accepted_care.providers);
    assert_eq!(care.prior_year, accepted_care.prior_year);
    let P::SaveCare { value, .. } = open_care(&mut s) else { panic!() };
    assert_eq!(value.benefits.len(), 1);
    assert_eq!(value.benefits[0].box10, 50_000);
    assert_eq!(value.benefits[0].in_box_1, None);
    assert_eq!(value.benefits[0].forfeited, None);
    let mut wages = crate::tests::support::client::income_pages::open_document_center(&mut s);
    if let P::SaveDocuments { documents, .. } = &mut wages {
        documents.common.wages.documents.clear();
    }
    care_saved(&send(&mut s, wages));
    let care = &s.draft.credits.as_ref().unwrap().care;
    assert!(care.benefits.is_empty());
    assert_eq!(care.plan_maximum, CarePlanMaximum::NoBenefits);
    assert_eq!(care.providers, accepted_care.providers);
    assert_eq!(care.prior_year, accepted_care.prior_year);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

fn estimate_care(page: &P, value: CarePage) -> P {
    let P::SaveCare {
        allowed_owners,
        care_filers,
        care_months,
        businesses,
        farms,
        partnerships,
        ..
    } = page
    else {
        panic!()
    };
    P::EstimateCare {
        allowed_filing: Vec::new(),
        allowed_injured_source_facts: vec![],
        people: vec![],

        user: TEST_USER,
        allowed_owners: allowed_owners.clone(),
        care_filers: care_filers.clone(),
        care_months: care_months.clone(),
        businesses: businesses.clone(),
        farms: farms.clone(),
        partnerships: partnerships.clone(),
        value,
    }
}

#[test]
fn care_completion_source_catalog_owner_twins_and_normal_source_deletion() {
    for kind in 0..3 {
        let mut d = care_return(FederalStatus::Joint, [60000, 40000], &[4000]);
        let id = d.next_id;
        let source = match kind {
            0 => cash_business(50000),
            1 => raised_farm(50000, 0),
            _ => {
                let mut p = ordinary_partner_source(50000, true);
                p.limited_for_se = Some(false);
                p.reported_se = Some(5_000_000);
                FederalIncome::Partnership { data: p }
            }
        };
        add(&mut d, income(source));
        d.credits.as_mut().unwrap().care.prior_year = Some(prior());
        let d = known_fact_partnership_draft(d);
        let mut s = Interview::new(TEST_USER, d);
        let mut page = open_care(&mut s);
        let P::SaveCare { value, businesses, farms, partnerships, .. } = &mut page else {
            panic!()
        };
        let choices = match kind {
            0 => businesses,
            1 => farms,
            _ => partnerships,
        };
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].id, id);
        assert_eq!(choices[0].owner, Owner::Taxpayer);
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        value.own_business = vec![CareOwnBusinessBenefitInput {
            owner: Owner::Taxpayer,
            source: match kind {
                0 => CareBenefitSourceInput::Business { business: id },
                1 => CareBenefitSourceInput::Farm { farm: id },
                _ => CareBenefitSourceInput::Partnership { partnership: id },
            },
            received: 100_000,
        }];
        care_saved(&send(&mut s, page));
        let accepted = s.draft.clone();
        assert_eq!(accepted.credits.as_ref().unwrap().care.own_business[0].received, 100_000);
        let mut forged = open_care(&mut s);
        if let P::SaveCare { value, businesses, farms, partnerships, .. } = &mut forged {
            value.own_business[0].owner = Owner::Spouse;
            // A forged client catalog cannot change the saved source's owner.
            let choices = match kind {
                0 => businesses,
                1 => farms,
                _ => partnerships,
            };
            choices[0].owner = Owner::Spouse;
        }
        assert!(matches!(send(&mut s, forged), Reply::Response(P::Problem { .. })));
        assert_eq!(s.draft, accepted);
        let mut repeated = open_care(&mut s);
        if let P::SaveCare { value, .. } = &mut repeated {
            value.own_business.push(value.own_business[0].clone());
        }
        assert!(matches!(send(&mut s, repeated), Reply::Response(P::Problem { .. })));
        assert_eq!(s.draft, accepted);
        let mut missing = open_care(&mut s);
        if let P::SaveCare { value, .. } = &mut missing {
            value.own_business[0].source = match kind {
                0 => CareBenefitSourceInput::Business { business: 99999 },
                1 => CareBenefitSourceInput::Farm { farm: 99999 },
                _ => CareBenefitSourceInput::Partnership { partnership: 99999 },
            };
        }
        assert!(matches!(send(&mut s, missing), Reply::Response(P::Problem { .. })));
        assert_eq!(s.draft, accepted);
        let source_page = if kind == 2 {
            let mut request = crate::tests::cross::interview::open_document_center(&mut s);
            let P::SaveDocuments { documents, .. } = &mut request else { panic!() };
            assert_eq!(documents.common.k1.documents.len(), 1);
            assert_eq!(documents.common.k1.documents[0].id, id);
            documents.common.k1.documents.clear();
            request
        } else {
            let topic = if kind == 0 { TaxSection::SelfEmployment } else { TaxSection::Farming };
            let mut request = crate::tests::cross::interview::open_topic(&mut s, topic);
            let P::SaveIncomes { documents, .. } = &mut request else { panic!() };
            documents.documents.clear();
            request
        };
        care_saved(&send(&mut s, source_page));
        let care = &s.draft.credits.as_ref().unwrap().care;
        assert!(
            care.own_business.is_empty(),
            "kind {kind}, accepted source records: {:?}",
            s.draft.income
        );
        assert_eq!(care.plan_maximum, CarePlanMaximum::NoBenefits);
        let previous = &accepted.credits.as_ref().unwrap().care;
        assert_eq!(care.providers, previous.providers);
        assert_eq!(care.prior_year, previous.prior_year);
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
    }
}

#[test]
fn care_completion_source_expense_projection_coordinates_health_sep_and_optional_se() {
    // These equivalent source controls supplement the independent F1–F8 and
    // C/F arithmetic oracles. A full $1000 line24 deduction must have exactly
    // the same source-owner effect as that ordinary EmployeeBenefits expense.
    for optional in [false, true] {
        let mut d = care_return(FederalStatus::Single, [60000, 0], &[1000]);
        let source = d.next_id;
        add(
            &mut d,
            income(if optional { raised_farm(12000, 10000) } else { cash_business(50000) }),
        );
        if optional {
            elect_se(&mut d, SeMethod::Farm);
        }
        se_health(&mut d, 1, if optional { 6000 } else { 2000 });
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        data.retirement = BusinessRetirement::Sep {
            contribution: if optional { 8420 } else { 100_000 },
            rate: 25.0,
        };
        let care = &mut d.credits.as_mut().unwrap().care;
        care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        care.own_business = vec![CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: if optional {
                CareBenefitSource::Farm { farm: source }
            } else {
                CareBenefitSource::Business { business: source }
            },
            received: 100_000,
        }];
        let original = d.clone();
        let mut control = d.clone();
        control.dependents[0].set_care_expenses(0);
        control.credits.as_mut().unwrap().care = FederalCare::default();
        let FederalIncome::Business { data } = &mut control.income[1].income else { panic!() };
        data.expenses
            .push(ordinary_business_expense(BusinessExpenseCategory::EmployeeBenefits, 100_000));
        let expected = supported(&control);
        let actual = supported(&d);
        assert_eq!(value(&actual, "care_form2441_line24"), 1000);
        for key in ["self_employment_tax", "agi", "qbi_deduction", "taxable_income"] {
            assert_eq!(value(&actual, key), value(&expected, key), "optional={optional}, {key}");
        }
        assert_eq!(actual.tax, expected.tax);
        assert_eq!(d, original);
        assert_eq!(restart(d.clone()).draft, original);
    }
}

#[test]
fn care_completion_filer_correction_reopens_whole_month_aggregate() {
    let mut d = care_return(FederalStatus::Joint, [60000, 1000], &[4000]);
    student(&mut d, Owner::Spouse, (1..=12).collect(), 1000);
    d.credits.as_mut().unwrap().care.prior_year = Some(prior());
    let original = d.clone();
    let mut s = Interview::new(TEST_USER, d);
    let mut spouse =
        crate::interview::filer(s.draft.people.as_ref().unwrap().spouse.as_ref().unwrap());
    spouse.details.more =
        MoreAboutYou::Died { date: 20260815, estate_medical: 0, bonds: None, ordinary: true };
    let request = crate::tests::cross::interview::save_spouse(&s.draft, spouse);
    care_saved(&send(&mut s, request));
    let care = &s.draft.credits.as_ref().unwrap().care;
    assert!(care.filers.is_empty());
    assert_eq!(care.providers, original.credits.as_ref().unwrap().care.providers);
    assert_eq!(care.prior_year, original.credits.as_ref().unwrap().care.prior_year);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

#[test]
fn care_completion_selected_partnership_changes_only_its_qualified_deduction() {
    // Both general partners have gross50000. At this income the second SSTB
    // is excluded; the first non-SSTB has ample W2 wages for its199A limit.
    // Source care3000 leaves aggregate SE97000: Medicare SE2598, half1299,
    // allocated650/649 by the existing gross-source allocator. First QBI is
    // 50000-650 with care on the SSTB, or50000-650-3000 on the first source.
    let mut d = care_return(FederalStatus::Single, [700000, 0], &[4000]);
    let mut ids = Vec::new();
    for service in [false, true] {
        let mut p = ordinary_partner_source(50000, true);
        p.limited_for_se = Some(false);
        p.reported_se = Some(5_000_000);
        p.se_gross = Some(PartnerSeGross::Nonfarm { gross: 5_000_000 });
        p.qbi_records.qualified_se_gross = 5_000_000;
        let mut statement = partner_qbi_statement(50000);
        statement.service = service;
        statement.wages = 10_000_000;
        p.qbi = Some(statement);
        ids.push(d.next_id);
        add(&mut d, income(FederalIncome::Partnership { data: p }));
    }
    for (source, qbi) in [(ids[1], 9870), (ids[0], 9270)] {
        let care = &mut d.credits.as_mut().unwrap().care;
        care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        care.own_business = vec![CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: CareBenefitSource::Partnership { partnership: source },
            received: 300_000,
        }];
        let original = d.clone();
        let r = supported(&d);
        assert_eq!(value(&r, "self_employment_tax"), 2598);
        assert_eq!(value(&r, "agi"), 795701);
        assert_eq!(value(&r, "qbi_deduction"), qbi);
        assert_eq!(d, original);
    }
}

#[test]
fn care_completion_partnership_deduction_keeps_regular_amt_and_passive_owners() {
    for active in [false, true] {
        let mut d = care_return(FederalStatus::Single, [60000, 0], &[4000]);
        let mut partner = ordinary_partner_source(50000, active);
        partner.alternative.as_mut().unwrap().ordinary = 3_000_000;
        let id = d.next_id;
        add(&mut d, income(FederalIncome::Partnership { data: partner }));
        let original = d.clone();
        let deductions = crate::care::CareDeductions { by_source: vec![(id, 3000)] };
        for (amt, expected) in [(false, 47000), (true, 27000)] {
            let mut r = FederalResult::empty(0);
            let rental = crate::extended_rules::rentals_for_home_with_care(
                &d,
                100000,
                &mut r,
                amt,
                [100000, 100000],
                &deductions,
            );
            assert!(rental.inputs_complete, "{:?}", r.issues);
            assert_eq!(rental.income, expected, "active={active}, amt={amt}");
        }
        assert_eq!(d, original);
    }
}

#[test]
fn care_completion_low_profit_source_and_half_se_limit_stabilize_together() {
    // Independently settled equations: original profit2000; own benefits1000;
    // care deduction924 leaves profit1076. SE rounds1076*.9235*.153 to152,
    // half76. AGI/actual earned1076-76=1000; exclusion earned1000+924-1000
    // =924. Thus line24=924 is self-consistent with the source and SE owner.
    for farm in [false, true] {
        let mut d = care_return(FederalStatus::Single, [0, 0], &[1000]);
        let id = d.next_id;
        add(&mut d, income(if farm { raised_farm(2000, 0) } else { cash_business(2000) }));
        let care = &mut d.credits.as_mut().unwrap().care;
        care.plan_maximum = CarePlanMaximum::Plans { maximum: 100_000 };
        care.own_business = vec![CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: if farm {
                CareBenefitSource::Farm { farm: id }
            } else {
                CareBenefitSource::Business { business: id }
            },
            received: 100_000,
        }];
        let original = d.clone();
        let r = supported(&d);
        assert_eq!(value(&r, "care_form2441_line24"), 924);
        assert_eq!(value(&r, "self_employment_tax"), 152);
        assert_eq!(value(&r, "agi"), 1000);
        assert_eq!(d, original);
    }
}

#[test]
fn care_completion_multiple_business_benefits_share_single_statutory_limit() {
    // Two own-plan benefit sources6000 each share the single7500 limit;
    // source allocator deducts3750 each. Profit46250+26250=72500; SE rounds
    // 72500*.9235*.153 to10244, half5122, AGI60000+72500-5122=127378.
    let mut d = care_return(FederalStatus::Single, [60000, 0], &[8000]);
    let business = d.next_id;
    add(&mut d, income(cash_business(50000)));
    let farm = d.next_id;
    add(&mut d, income(raised_farm(30000, 0)));
    let care = &mut d.credits.as_mut().unwrap().care;
    care.plan_maximum = CarePlanMaximum::Plans { maximum: 1_200_000 };
    care.own_business = vec![
        CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: CareBenefitSource::Business { business },
            received: 600_000,
        },
        CareOwnBusinessBenefit {
            owner: Owner::Taxpayer,
            source: CareBenefitSource::Farm { farm },
            received: 600_000,
        },
    ];
    let original = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line24"), 7500);
    assert_eq!(value(&r, &alloc::format!("business_{business}_net_profit")), 46250);
    assert_eq!(value(&r, &alloc::format!("farm_{farm}_net_profit")), 26250);
    assert_eq!(value(&r, "self_employment_tax"), 10244);
    assert_eq!(value(&r, "agi"), 127378);
    assert_eq!(d, original);
}

#[test]
fn care_completion_limited_partner_service_pay_remains_a_se_source() {
    // IRC1402(a)(13) excludes the limited partner's distributive items;
    // guaranteed service remuneration remains separate. The selected code O
    // care deduction belongs to Schedule E, not that service payment.
    let mut d = care_return(FederalStatus::Single, [60000, 0], &[4000]);
    let id = d.next_id;
    let mut partner = ordinary_partner_source(50000, true);
    partner.guaranteed_services = 1_000_000;
    partner.reported_se = Some(1_000_000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 1_000_000 });
    partner.qbi = Some(partner_qbi_statement(50000));
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let care = &mut d.credits.as_mut().unwrap().care;
    care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    care.own_business = vec![CareOwnBusinessBenefit {
        owner: Owner::Taxpayer,
        source: CareBenefitSource::Partnership { partnership: id },
        received: 300_000,
    }];
    let original = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line24"), 3000);
    assert_eq!(value(&r, "rental_income"), 57000);
    // $10,000 × .9235 × .153 = $1,413 SE tax; half rounds $707.
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert_eq!(value(&r, "agi"), 116293);
    // Service guarantees are not QBI. Its qualified SE gross is zero, so
    // no half-SE deduction belongs to QBI: ($50,000 − $3,000) ×20%.
    assert_eq!(value(&r, "qbi_deduction"), 9400);
    assert_eq!(d, original);
    assert_eq!(restart(d.clone()).draft, original);
}

#[test]
fn care_completion_partner_qbi_expense_keeps_zero_loss_and_partial_sources() {
    let mut partner = ordinary_partner_source(50000, true);
    let mut statement = partner_qbi_statement(50000);
    // Full qualification is source-based, not capped at current profit.
    assert_eq!(statement.qualified_care_deduction(&partner, 3000), Ok(3000));
    for ordinary in [0, -5000] {
        partner.regular.ordinary = ordinary * 100;
        statement.income = ordinary * 100;
        assert_eq!(statement.qualified_care_deduction(&partner, 3000), Ok(3000));
        let result = statement
            .calculate(
                &partner,
                crate::tests::support::builders::k1_amount(0),
                3000,
                60000,
                FederalStatus::Single,
            )
            .unwrap();
        assert_eq!(result.current.income, ordinary - 3000);
    }
    partner = ordinary_partner_source(50000, true);
    statement.income = 2_500_000;
    // Limited-partner service SE gross says nothing about the qualified
    // fraction of ordinary distributive income: preserve the attribution error.
    partner.guaranteed_services = 1_000_000;
    partner.reported_se = Some(1_000_000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 1_000_000 });
    assert_eq!(
        statement.qualified_care_deduction(&partner, 3000).unwrap_err().0,
        "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5"
    );
    // Actual general-partner gross facts support the existing half-qualified
    // allocation. The health/retirement owner's allocation stays unchanged.
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(6_000_000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 6_000_000 });
    partner.qbi_records.qualified_se_gross = 3_000_000;
    assert_eq!(statement.qualified_care_deduction(&partner, 3000), Ok(1500));
    assert_eq!(statement.qualified_deduction(&partner, 1000), Ok(500));
}

#[test]
fn unit13_approved_prior_care_claimant_fact_saves_reopens_and_preserves_joint_credit() {
    let mut d = care_return(FederalStatus::Joint, [50000, 20000], &[]);
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(crate::tests::support::builders::allocation());
    d.credits.as_mut().unwrap().care.prior_year = Some(PriorYearCare {
        two_or_more: false,
        prior_line_3: 0,
        paid_in_return_year: 100_000,
        prior_benefits: 0,
        prior_earned_smaller: 5_000_000,
        prior_line_6: 0,
        prior_agi: 5_000_000,
        injured_claimants: None,
    });
    let before = supported(&d);
    assert_eq!(value(&before, "prior_year_care_credit"), 200);
    let mut s = Interview::new(TEST_USER, d);
    let mut page = open_care(&mut s);
    let P::SaveCare { value: facts, .. } = &mut page else { panic!() };
    let claimants = InjuredPriorCareClaimantsInput::People {
        people: vec![InjuredPriorCarePersonInput {
            person: InjuredPriorCareIdentityInput::Other {
                name: "Synthetic prior qualifying person 🧾".repeat(12),
            },
            claimant: Owner::Spouse,
            expense: 100_000,
        }],
    };
    facts.prior_year.as_mut().unwrap().injured_claimants = Some(claimants.clone());
    crate::tests::cross::interview::accepted(&send(&mut s, page));
    let after = supported(&s.draft);
    assert_eq!(value(&after, "prior_year_care_credit"), 200);
    assert_eq!(value(&after, "form_8379_line_16_b"), 0);
    assert_eq!(value(&after, "form_8379_line_16_c"), 200);
    assert!(
        !after
            .review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE")
    );
    assert_eq!(value(&before, "agi"), value(&after, "agi"));
    let mut reopened = restart(s.draft.clone());
    let P::SaveCare { value: shown, .. } = open_care(&mut reopened) else { panic!() };
    assert_eq!(shown.prior_year.unwrap().injured_claimants, Some(claimants));
    assert_eq!(reopened.draft, s.draft);
}

// The approved actual expenses now distinguish the two previously identical
// accepted histories without changing their joint worksheet or credit.
#[test]
fn unit13_approved_prior_care_expenses_resolve_mixed_claimants() {
    let mut d = care_return(FederalStatus::Joint, [50000, 20000], &[]);
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(crate::tests::support::builders::allocation());
    d.credits.as_mut().unwrap().care.prior_year = Some(PriorYearCare {
        two_or_more: true,
        prior_line_3: 0,
        paid_in_return_year: 100_000,
        prior_benefits: 0,
        prior_earned_smaller: 5_000_000,
        prior_line_6: 0,
        prior_agi: 5_000_000,
        injured_claimants: None,
    });
    let claimants = InjuredPriorCareClaimantsInput::People {
        people: vec![
            InjuredPriorCarePersonInput {
                person: InjuredPriorCareIdentityInput::Other { name: "Prior person P".into() },
                claimant: Owner::Taxpayer,
                expense: 90_000,
            },
            InjuredPriorCarePersonInput {
                person: InjuredPriorCareIdentityInput::Other { name: "Prior person Q".into() },
                claimant: Owner::Spouse,
                expense: 10_000,
            },
        ],
    };
    let mut encoded = vec![];
    for expenses in [[900_i64, 100_i64], [100, 900]] {
        let mut s = Interview::new(TEST_USER, d.clone());
        let mut page = open_care(&mut s);
        let P::SaveCare { value: facts, .. } = &mut page else { panic!() };
        let mut actual = claimants.clone();
        let InjuredPriorCareClaimantsInput::People { people } = &mut actual;
        for (person, expense) in people.iter_mut().zip(expenses) {
            person.expense = expense * 100;
        }
        facts.prior_year.as_mut().unwrap().injured_claimants = Some(actual.clone());
        let mut forged = page.clone();
        let P::SaveCare { value: forged_care, .. } = &mut forged else { panic!() };
        let InjuredPriorCareClaimantsInput::People { people } =
            forged_care.prior_year.as_mut().unwrap().injured_claimants.as_mut().unwrap();
        people[0].expense += 100;
        let unchanged = s.draft.clone();
        let response = send(&mut s, forged);
        crate::tests::cross::interview::rejected_with(response, "INJURED_PRIOR_CARE_EXPENSE_TOTAL");
        assert_eq!(s.draft, unchanged);
        crate::tests::cross::interview::accepted(&send(&mut s, page));
        let result = supported(&s.draft);
        assert_eq!(value(&result, "prior_year_care_credit"), 200);
        assert!(
            !result
                .review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE")
        );
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
        // Independent 2025 worksheet rate: AGI $50,000 => 20%; no
        // benefit, earned-income or $6,000 ceiling binds either payment.
        let owner_credit = expenses.map(|paid| paid * 20 / 100);
        assert_eq!(owner_credit.iter().sum::<i64>(), 200);
        assert_eq!(value(&result, "form_8379_line_16_b"), owner_credit[0]);
        assert_eq!(value(&result, "form_8379_line_16_c"), owner_credit[1]);
        let mut reopened = restart(s.draft.clone());
        let P::SaveCare { value: shown, .. } = open_care(&mut reopened) else { panic!() };
        assert_eq!(shown.prior_year.unwrap().injured_claimants, Some(actual));
        assert_eq!(owner_credit, if expenses[0] == 900 { [180, 20] } else { [20, 180] });
        encoded.push(s.draft);
    }
    assert_ne!(encoded[0], encoded[1]);
}

#[test]
fn unit13_prior_care_actual_client_rows_save_reopen_and_restart() {
    extern crate std;
    let mut d = care_return(FederalStatus::Joint, [50000, 20000], &[]);
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    d.injured_spouse = Some(crate::tests::support::builders::allocation());
    d.credits.as_mut().unwrap().care.prior_year = Some(PriorYearCare {
        two_or_more: true,
        prior_line_3: 0,
        paid_in_return_year: 100_000,
        prior_benefits: 0,
        prior_earned_smaller: 5_000_000,
        prior_line_6: 0,
        prior_agi: 5_000_000,
        injured_claimants: None,
    });
    let mut session = Interview::new(TEST_USER, d);
    let page = open_care(&mut session);
    if let Ok(input) = std::env::var("TAX_UNIT13_CLIENT_CALC_INPUT") {
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let Reply::Response(response) = send(&mut session, request) else {
            panic!("calculation reply")
        };
        std::fs::write(std::env::var("TAX_UNIT13_CLIENT_CALC_OUTPUT").unwrap(), response.to_avro())
            .unwrap();
        return;
    }
    if let Ok(dir) = std::env::var("TAX_UNIT13_CLIENT_SEED_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
        let mut answered = page.clone();
        let P::SaveCare { value: care, .. } = &mut answered else { panic!() };
        care.prior_year.as_mut().unwrap().injured_claimants =
            Some(InjuredPriorCareClaimantsInput::People {
                people: vec![
                    InjuredPriorCarePersonInput {
                        person: InjuredPriorCareIdentityInput::Other {
                            name: "Prior client P".into(),
                        },
                        claimant: Owner::Taxpayer,
                        expense: 90_000,
                    },
                    InjuredPriorCarePersonInput {
                        person: InjuredPriorCareIdentityInput::Other {
                            name: "Prior client Q".into(),
                        },
                        claimant: Owner::Spouse,
                        expense: 10_000,
                    },
                ],
            });
        std::fs::write(std::path::Path::new(&dir).join("answers.avro"), answered.to_avro())
            .unwrap();
        return;
    }
    // The editor's answers: two prior-care payers, each expense on one spouse.
    let mut request = page;
    let P::SaveCare { value: care, .. } = &mut request else { panic!() };
    care.prior_year.as_mut().unwrap().injured_claimants =
        Some(InjuredPriorCareClaimantsInput::People {
            people: vec![
                InjuredPriorCarePersonInput {
                    person: InjuredPriorCareIdentityInput::Other { name: "Prior client P".into() },
                    claimant: Owner::Taxpayer,
                    expense: 90_000,
                },
                InjuredPriorCarePersonInput {
                    person: InjuredPriorCareIdentityInput::Other { name: "Prior client Q".into() },
                    claimant: Owner::Spouse,
                    expense: 10_000,
                },
            ],
        });
    let before = supported(&session.draft);
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    let after = supported(&session.draft);
    assert_eq!(value(&after, "prior_year_care_credit"), 200);
    assert_eq!(value(&after, "form_8379_line_16_b"), 180);
    assert_eq!(value(&after, "form_8379_line_16_c"), 20);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(after.tax, before.tax);
    let data = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(session.draft.tax_year, &data);
    assert_eq!(decoded, session.draft);
    assert_eq!(native_draft_records(&decoded), data);
    let mut reopened = restart(session.draft.clone());
    let P::SaveCare { value: shown, .. } = open_care(&mut reopened) else { panic!() };
    let InjuredPriorCareClaimantsInput::People { people } =
        shown.prior_year.unwrap().injured_claimants.unwrap();
    assert_eq!(people.iter().map(|p| p.expense).collect::<Vec<_>>(), vec![90_000, 10_000]);
    assert_eq!(reopened.draft, session.draft);
}
