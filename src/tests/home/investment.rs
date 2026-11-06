use super::*;
use crate::tests::investment::investment::{add_vacation_home, cents};
#[test]
fn home_exclusion_election_and_personal_loss_do_not_create_a_deduction() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 10000000;
    home.exclusion = HomeExclusion::Decline;
    add(&mut d, income(FederalIncome::HomeSale { data: main_home_source(home) }));
    assert_eq!(value(&supported(&d), "capital_gain"), 0);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_home(data);
    data.price = 80000000;
    drop(data);
    assert_eq!(value(&supported(&d), "capital_gain"), 550000);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_home(data);
    // Special treatment (No) leaves the sale out with a notice.
    data.ordinary = false;
    drop(data);
    let r = supported(&d);
    // The stored sale states the special situation in its answers: that notice names it.
    assert!(noted(&r, "WARN_HOME_SALE_OTHER_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "capital_gain"), 0);
}
#[test]
fn reduced_home_exclusion_caps_at_gain_and_requires_confirmed_qualifying_event() {
    for reason in [
        HomeSaleTrigger::Unemployment,
        HomeSaleTrigger::MultipleBirth,
        HomeSaleTrigger::Divorce,
        HomeSaleTrigger::Death,
        HomeSaleTrigger::Destruction,
        HomeSaleTrigger::Casualty,
    ] {
        let mut home = home_sale(false);
        home.price = 30000000;
        home.exclusion =
            HomeExclusion::Reduced { reason, taxpayer: reduced_home_periods(365), spouse: None };
        assert_eq!(home.excluded(FederalStatus::Single), Ok(5000000));
        let HomeExclusion::Reduced { reason, .. } = &mut home.exclusion else { panic!() };
        *reason = HomeSaleTrigger::Other;
        assert!(home.excluded(FederalStatus::Single).is_err());
    }
}

#[test]
fn ordinary_subsidized_home_sale_calculates_form_8828_recapture() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 40_000_000;
    home.expenses = 2_000_000;
    home.basis = 30_000_000;
    add(
        &mut d,
        income(FederalIncome::HomeSale { data: sold_on(main_home_source(home), 20260701) }),
    );
    let without_recapture = supported(&d);
    assert_eq!(value(&without_recapture, "home_2_taxable_gain"), 0);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 2,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), 80000);
    assert_eq!(value(&r, "mortgage_subsidy_1_modified_income"), 60000);
    assert_eq!(value(&r, "mortgage_subsidy_1_adjusted_qualifying_income"), 50000);
    assert_eq!(value(&r, "mortgage_subsidy_1_amount"), 6250);
    assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 3750);
    assert_eq!(value(&r, "total_tax"), value(&without_recapture, "total_tax") + 3750);
    assert_eq!(r.tax, without_recapture.tax.map(|tax| tax + 375_000));
    assert_eq!(supported(&restart(d.clone()).draft), r);

    if let MortgageSubsidyDisposition::Ordinary { subsidy, .. } = mortgage_disposition(&mut d) {
        *subsidy = MortgageSubsidyKind::Qmb;
    }
    let qmb = supported(&d);
    assert_eq!(value(&qmb, "mortgage_subsidy_recapture_tax"), 3750);
    assert_eq!(qmb.tax, r.tax);
    assert_eq!(supported(&restart(d.clone()).draft), qmb);

    mortgage_recapture_facts(&mut d).payoff =
        MortgageSubsidyPayoff::Earlier { repaid_on: 20220115 };
    let early_payoff = supported(&d);
    // Form 8828 line 20 worksheet: 2 years -> 40%, then 5 rounded-up
    // post-payoff years -> 20%; 40% × 20% = 8% of the $6,250 subsidy.
    assert_eq!(value(&early_payoff, "mortgage_subsidy_recapture_tax"), 500);
    assert_eq!(supported(&restart(d.clone()).draft), early_payoff);
    mortgage_recapture_facts(&mut d).payoff =
        MortgageSubsidyPayoff::Earlier { repaid_on: 20240116 };
    let after_four_years = supported(&d);
    assert_eq!(value(&after_four_years, "mortgage_subsidy_recapture_tax"), 3750);
    mortgage_recapture_facts(&mut d).payoff = MortgageSubsidyPayoff::AtDispositionOrReissued;

    mortgage_recapture_facts(&mut d).adjusted_qualifying_income = 6_500_000;
    let below_income = supported(&d);
    assert_eq!(value(&below_income, "mortgage_subsidy_recapture_tax"), 0);
    mortgage_recapture_facts(&mut d).adjusted_qualifying_income = 5_750_000;
    let partial_income = supported(&d);
    assert_eq!(value(&partial_income, "mortgage_subsidy_recapture_tax"), 1875);
    mortgage_recapture_facts(&mut d).adjusted_qualifying_income = 5_000_000;
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_home(data);
    data.price = 32_500_000;
    drop(data);
    let gain_limited = supported(&d);
    assert_eq!(value(&gain_limited, "mortgage_subsidy_1_disposition_gain"), 5000);
    assert_eq!(value(&gain_limited, "mortgage_subsidy_recapture_tax"), 2500);
    mortgage_recapture_facts(&mut d).original_closed_on = 20170115;
    let after_nine_years = supported(&d);
    assert_eq!(value(&after_nine_years, "mortgage_subsidy_recapture_tax"), 0);
    *mortgage_recapture_sale_id(&mut d) = 999;
    unsupported(&d, "Mortgage subsidy recapture");
    *mortgage_recapture_sale_id(&mut d) = 2;
    mortgage_recapture_facts(&mut d).original_closed_on = 20200115;
    mortgage_recapture_facts(&mut d).payoff = MortgageSubsidyPayoff::Unreviewed;
    unsupported(&d, "Mortgage subsidy recapture");
    mortgage_recapture_facts(&mut d).payoff = MortgageSubsidyPayoff::AtDispositionOrReissued;
    *mortgage_recapture_sale_id(&mut d) = 999;
    unsupported(&d, "Mortgage subsidy recapture");
    *mortgage_recapture_sale_id(&mut d) = 2;
    mortgage_recapture_facts(&mut d).payoff =
        MortgageSubsidyPayoff::Earlier { repaid_on: 20270101 };
    assert!(!d.valid_stored());
    mortgage_recapture_facts(&mut d).payoff = MortgageSubsidyPayoff::AtDispositionOrReissued;
}

#[test]
fn two_subsidized_home_sales_keep_distinct_form_8828_results() {
    let mut d = complete(FederalStatus::Single);
    let mut first_home = home_sale(false);
    first_home.price = 40_000_000;
    first_home.expenses = 2_000_000;
    first_home.basis = 30_000_000;
    add(
        &mut d,
        income(FederalIncome::HomeSale { data: sold_on(main_home_source(first_home), 20260701) }),
    );
    let mut second_home = home_sale(false);
    second_home.price = 40_000_000;
    second_home.expenses = 2_000_000;
    second_home.basis = 30_000_000;
    second_home.exclusion = HomeExclusion::Decline;
    add(
        &mut d,
        income(FederalIncome::HomeSale { data: sold_on(main_home_source(second_home), 20260701) }),
    );
    let without_recapture = supported(&d);
    d.credits.as_mut().unwrap().mortgage_subsidies = vec![
        MortgageSubsidyEntry {
            label: "First home".into(),
            disposition: MortgageSubsidyDisposition::Ordinary {
                subsidy: MortgageSubsidyKind::Mcc,
                transfer: MortgageSubsidyTransfer::Sale {
                    home_sale_income_id: 2,
                    qsml_gain_share: None,
                },
                facts: ordinary_subsidy_facts(),
            },
        },
        MortgageSubsidyEntry {
            label: "Second home".into(),
            disposition: MortgageSubsidyDisposition::Ordinary {
                subsidy: MortgageSubsidyKind::Qmb,
                transfer: MortgageSubsidyTransfer::Sale {
                    home_sale_income_id: 3,
                    qsml_gain_share: None,
                },
                facts: ordinary_subsidy_facts(),
            },
        },
    ];
    let result = supported(&d);
    assert_eq!(value(&result, "mortgage_subsidy_1_recapture_tax"), 3750);
    assert_eq!(value(&result, "mortgage_subsidy_2_recapture_tax"), 3750);
    assert_eq!(value(&result, "mortgage_subsidy_recapture_tax"), 7500);
    assert_eq!(value(&result, "mortgage_subsidy_1_disposition_gain"), 80000);
    assert_eq!(value(&result, "mortgage_subsidy_2_disposition_gain"), 80000);
    assert!(
        breakdown(&result).iter().any(|line| { line.label == "Mortgage subsidy 1 recapture tax" })
    );
    assert!(
        breakdown(&result).iter().any(|line| { line.label == "Mortgage subsidy 2 recapture tax" })
    );
    assert!(
        result.lines.iter().any(|line| { line.rule == "US-2026.mortgage_subsidy_2_recapture_tax" })
    );
    let combined_line =
        result.lines.iter().find(|line| line.id == "mortgage_subsidy_recapture_tax").unwrap();
    assert_eq!(
        combined_line.inputs.iter().map(|input| input.as_str()).collect::<Vec<_>>(),
        vec!["mortgage_subsidy_1_recapture_tax", "mortgage_subsidy_2_recapture_tax"]
    );
    assert_eq!(value(&result, "total_tax"), value(&without_recapture, "total_tax") + 7500);
    assert_eq!(supported(&restart(d.clone()).draft), result);

    d.credits.as_mut().unwrap().mortgage_subsidies[1].label = "First home".into();
    assert!(!d.valid_stored());
    d.credits.as_mut().unwrap().mortgage_subsidies[1].label = "Second home".into();
    if let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::Sale { home_sale_income_id, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().mortgage_subsidies[1].disposition
    {
        *home_sale_income_id = 2;
    }
    assert!(!d.valid_stored());
}

#[test]
fn qualified_subordinate_mortgage_lender_share_limits_form_8828_gain() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 40_000_000;
    home.expenses = 2_000_000;
    home.basis = 30_000_000;
    add(
        &mut d,
        income(FederalIncome::HomeSale { data: sold_on(main_home_source(home), 20260701) }),
    );
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Qmb,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 2,
                qsml_gain_share: Some(MortgageSubsidyQsmlGainShare {
                    agreement_reference: "Synthetic local housing-agency agreement".into(),
                    paid_on: 20260701,
                    paid_gain_share: 7_800_000,
                    qualified_and_distinct: true,
                }),
            },
            facts: ordinary_subsidy_facts(),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_subsidy_1_qsml_paid_gain_share"), 78000);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), 2000);
    assert_eq!(value(&r, "mortgage_subsidy_1_modified_income"), 60000);
    assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 1000);
    assert_eq!(value(&r, "home_2_taxable_gain"), 0);
    assert_eq!(r.tax, ordinary_return.tax.map(|tax| tax + 100_000));
    assert_eq!(supported(&restart(d.clone()).draft), r);

    mortgage_recapture_qsml(&mut d).paid_gain_share = 8_100_000;
    unsupported(&d, "Mortgage subsidy recapture");
    mortgage_recapture_qsml(&mut d).paid_gain_share = 7_800_000;
    // No or Not sure: the share is not subtracted, so the larger gain is
    // recaptured; Not sure adds a note.
    {
        let answer = false;
        mortgage_recapture_qsml(&mut d).qualified_and_distinct = answer;
        let unshared = supported(&d);
        assert!(
            !unshared.lines.iter().any(|line| line.id == "mortgage_subsidy_1_qsml_paid_gain_share")
        );
        assert!(unshared.tax > r.tax);
        // The note is labeled with its entry (NOTE_WITH_LABEL).
    }
    mortgage_recapture_qsml(&mut d).qualified_and_distinct = true;
    mortgage_recapture_qsml(&mut d).paid_on = 20260630;
    unsupported(&d, "Mortgage subsidy recapture");
}

#[test]
fn subsidized_home_sale_at_a_loss_has_no_recapture_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 25_000_000;
    home.expenses = 2_000_000;
    home.basis = 30_000_000;
    add(
        &mut d,
        income(FederalIncome::HomeSale { data: sold_on(main_home_source(home), 20260701) }),
    );
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 2,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), -70000);
    assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

#[test]
fn personal_casualty_applies_per_event_and_shared_agi_limits() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.personal_casualties.push(personal_disaster("Storm A", 3_000_000));
    let first = supported(&d);
    assert_eq!(value(&first, "form_4684_personal_loss_after_events"), 29_900);
    assert_eq!(value(&first, "personal_casualty_deduction"), 23_900);
    assert_eq!(return_line(&first, "12e"), 2_390_000);

    d.deductions
        .as_mut()
        .unwrap()
        .personal_casualties
        .push(personal_disaster("Storm B", 1_000_000));
    let two_events = supported(&d);
    assert_eq!(value(&two_events, "form_4684_personal_loss_after_events"), 39_800);
    assert_eq!(value(&two_events, "personal_casualty_deduction"), 33_800);
    assert_eq!(estimate(&restart(d.clone()).draft), two_events);

    let ded = d.deductions.as_mut().unwrap();
    let second = ded.personal_casualties.pop().unwrap().properties.pop().unwrap();
    ded.personal_casualties[0].properties.push(second);
    let one_event = supported(&d);
    assert_eq!(value(&one_event, "form_4684_personal_loss_after_events"), 39_900);
    assert_eq!(value(&one_event, "personal_casualty_deduction"), 33_900);
}

#[test]
fn personal_casualty_standard_route_and_source_guards() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.personal_casualties.push(personal_disaster("Storm", 3_000_000));
    let standard = supported(&d);
    assert_eq!(value(&standard, "form_4684_personal_loss_after_agi"), 23_900);
    assert_eq!(value(&standard, "personal_casualty_deduction"), 23900);

    // No leaves the event's loss out with its notice; the estimate proceeds.
    {
        let (answer, key) = (false, "WARN_CASUALTY_LOSS_LEFT_OUT");
        d.deductions.as_mut().unwrap().personal_casualties[0].source_complete = answer;
        let r = supported(&d);
        assert!(noted(&r, key));
        assert_eq!(value(&r, "form_4684_personal_loss_after_events"), 0);
        assert_eq!(value(&r, "personal_casualty_deduction"), 0);
    }
    d.deductions.as_mut().unwrap().personal_casualties[0].source_complete = true;
    d.deductions.as_mut().unwrap().personal_casualties[0].properties[0].after = 3_000_001;
    let invalid = estimate(&d);
    assert_eq!(invalid.state, ResultState::Incomplete);
    assert!(invalid.issues.iter().any(|issue| issue.section == "Saved return"));
    d.deductions.as_mut().unwrap().personal_casualties[0].properties[0].after = 0;
    d.deductions.as_mut().unwrap().personal_casualties.push(personal_disaster("Storm", 100_000));
    // 2026-10-17: Source identity and association contract: event labels
    // are descriptions, so two distinct events may have the same label.
    let same_label_events = supported(&d);
    assert_eq!(value(&same_label_events, "form_4684_personal_loss_after_events"), 30_800);
}

#[test]
fn personal_casualty_gains_offset_disaster_losses_before_agi_limit() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.personal_casualties.push(personal_disaster("Storm", 3_000_000));
    let gain = CasualtyProperty {
        id: 0,
        home_site: None,
        business_damage: None,
        label: "Damaged property with gain".into(),
        location: "Synthetic City, CA".into(),
        acquisition: CasualtyAcquisition::Purchased { date: 20180101 },
        basis: 100_000,
        before: 200_000,
        after: 0,
        reimbursement: 400_000,
        received: 400_000,
    };
    ded.personal_casualties[0].properties.push(gain);
    let result = supported(&d);
    assert_eq!(value(&result, "form_4684_personal_loss_after_events"), 29_900);
    assert_eq!(value(&result, "form_4684_personal_gains"), 3_000);
    assert_eq!(value(&result, "form_4684_personal_net_after_gains"), 26_900);
    assert_eq!(value(&result, "personal_casualty_deduction"), 20_900);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().personal_casualties[0].properties[1].reimbursement = 3_500_000;

    d.deductions.as_mut().unwrap().personal_casualties[0].properties[1].received = 3_500_000;
    let excess = supported(&d);
    assert_eq!(value(&excess, "form_4684_personal_long_net"), 4_100);
    assert_eq!(value(&excess, "personal_casualty_deduction"), 0);
    assert_eq!(return_line(&excess, "7a"), 410_000);
    assert_eq!(estimate(&restart(d.clone()).draft), excess);
}

#[test]
fn personal_casualty_excess_gain_uses_holding_period_and_not_investment_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 650_000);
    let baseline = supported(&d);
    let mut short = personal_disaster("Short holding", 100_000);
    short.properties[0].before = 200_000;
    short.properties[0].reimbursement = 400_000;
    short.properties[0].received = 400_000;
    short.properties[0].acquisition = CasualtyAcquisition::Purchased { date: 20250801 }; // Exactly one year: short term.
    let mut long = personal_disaster("Long holding", 100_000);
    long.properties[0].before = 200_000;
    long.properties[0].reimbursement = 300_000;
    long.properties[0].received = 300_000;
    long.properties[0].acquisition = CasualtyAcquisition::Purchased { date: 20250731 }; // One year and one day: long term.
    d.deductions.as_mut().unwrap().personal_casualties.extend([short, long]);
    let result = supported(&d);
    assert_eq!(value(&result, "form_4684_personal_short_net"), 3_000);
    assert_eq!(value(&result, "form_4684_personal_long_net"), 2_000);
    assert_eq!(value(&result, "capital_gain"), 5_000);
    assert_eq!(return_line(&result, "7a"), 500_000);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 5_000);
    assert_eq!(
        value(&result, "net_investment_income_tax"),
        value(&baseline, "net_investment_income_tax") + 190
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    // A short-term loss offsets the short-term pool before cross-term Schedule D netting.
    let mut mixed = personal_disaster("Mixed holding loss", 100_000);
    mixed.properties[0].acquisition = CasualtyAcquisition::Purchased { date: 20260701 };
    d.deductions.as_mut().unwrap().personal_casualties.push(mixed);
    let with_short_loss = supported(&d);
    assert_eq!(value(&with_short_loss, "form_4684_personal_short_net"), 2_100);
    assert_eq!(value(&with_short_loss, "form_4684_personal_long_net"), 2_000);
    assert_eq!(return_line(&with_short_loss, "7a"), 410_000);
    assert_eq!(estimate(&restart(d.clone()).draft), with_short_loss);

    let short_loss_property =
        &mut d.deductions.as_mut().unwrap().personal_casualties[2].properties[0];
    short_loss_property.basis = 400_000;
    short_loss_property.before = 400_000;
    let signed = supported(&d);
    assert_eq!(value(&signed, "form_4684_personal_short_net"), -900);
    assert_eq!(value(&signed, "form_4684_personal_long_net"), 2_000);
    assert_eq!(return_line(&signed, "7a"), 110_000);
    assert_eq!(estimate(&restart(d.clone()).draft), signed);

    // One event with losses of both terms still needs its $100 allocation.
    d.deductions.as_mut().unwrap().personal_casualties.last_mut().unwrap().properties.push(
        CasualtyProperty {
            id: 0,
            home_site: None,
            business_damage: None,
            label: "Long-term lost property".into(),
            location: "Synthetic City, CA".into(),
            acquisition: CasualtyAcquisition::Purchased { date: 20200101 },
            basis: 100_000,
            before: 100_000,
            after: 0,
            reimbursement: 0,
            received: 0,
        },
    );
    unsupported(&d, "Personal casualty gain");
    assert!(estimate(&d).issues.iter().any(|issue| {
        issue.section == "Personal casualty gain"
            && Some({
                let text = &issue.display;
                text.key.as_str()
            }) == Some("ISSUE_CASUALTY_MIXED_TERM_EVENT_REDUCTION")
    }));
}

#[test]
fn undeclared_personal_casualty_loss_only_offsets_recognized_gains() {
    let mut d = complete(FederalStatus::Single);
    let mut loss = personal_disaster("Undeclared theft", 300_000);
    loss.declaration = CasualtyDeclaration::Undeclared;
    loss.zip.clear();
    d.deductions.as_mut().unwrap().personal_casualties.push(loss);
    let alone = supported(&d);
    assert_eq!(value(&alone, "form_4684_undeclared_loss_after_events"), 2_900);
    assert_eq!(value(&alone, "form_4684_personal_loss_after_events"), 0);
    assert_eq!(value(&alone, "personal_casualty_deduction"), 0);

    let mut gain = personal_disaster("Undeclared gain", 100_000);
    gain.declaration = CasualtyDeclaration::Undeclared;
    gain.properties[0].before = 200_000;
    gain.properties[0].reimbursement = 400_000;
    gain.properties[0].received = 400_000;
    d.deductions.as_mut().unwrap().personal_casualties.push(gain);
    let net_gain = supported(&d);
    assert_eq!(value(&net_gain, "form_4684_personal_loss_after_events"), 2_900);
    assert_eq!(value(&net_gain, "form_4684_personal_long_net"), 100);
    assert_eq!(return_line(&net_gain, "7a"), 10_000);
    assert_eq!(estimate(&restart(d.clone()).draft), net_gain);

    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    ded.personal_casualties.push(personal_disaster("Declared storm", 1_000_000));
    let combined = supported(&d);
    assert_eq!(value(&combined, "form_4684_personal_loss_after_events"), 12_800);
    assert_eq!(value(&combined, "form_4684_personal_gains"), 3_000);
    assert_eq!(value(&combined, "form_4684_personal_net_after_gains"), 9_800);
    assert_eq!(value(&combined, "personal_casualty_deduction"), 3_800);
    assert_eq!(return_line(&combined, "7a"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), combined);
    d.deductions.as_mut().unwrap().personal_casualties.last_mut().unwrap().zip.clear();
    assert!(!d.valid_stored());
}

#[test]
fn government_casualty_access_order_can_support_living_expense_insurance() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::CasualtyLivingInsurance {
            data: FederalCasualtyLivingInsurance {
                event_reference: "SYN-ACCESS-2026".into(),
                loss_of_use: FederalHomeLossOfUse::GovernmentDeniedAccess {
                    authority: "Synthetic county emergency office".into(),
                    order_reference: "SYN-ORDER-1, threat of nearby fire".into(),
                },
                lost_use_on: 20260601,
                regained_use_on: Some(20260701),
                payments: vec![casualty_living_payment("SYN-ACCESS-CLAIM", 20260615, 220_000, 0)],
                actual_living_expenses: 385_000,
                normal_living_expenses: 190_000,
                prior_taxable_reported: 0,
                federal_disaster_area: false,
                federal_declaration_reference: "".into(),
                confirmed: true,
            },
        }),
    );
    let result = supported(&d);
    assert_eq!(value(&result, "taxable_casualty_living_insurance"), 250);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 250);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.loss_of_use = FederalHomeLossOfUse::GovernmentDeniedAccess {
            authority: "Synthetic county emergency office".into(),
            order_reference: "".into(),
        };
    }
    assert!(!d.valid_stored());
}

#[test]
fn personal_use_home_lists_its_schedule_e_rental_share() {
    // The Pub 527 home below (60 fair rental, 30 personal days): Schedule
    // E line 3 rent $6,000; line 18 the depreciation Worksheet 5-1 allows,
    // $1,600; line 20 = interest and taxes $2,800 + operating $1,600 +
    // depreciation $1,600 = $6,000; line 21 = 0 (not listed). The entered
    // expense categories are whole-property amounts, not the rental share,
    // so lines 9-17 are not listed. A royalty's $5,000 gross is line 4.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 60, 30, [0, 0]);
    add(&mut d, income(royalty(5000, 1000)));
    let rows = crate::draft_return(&supported(&d));
    let schedule_e: Vec<(alloc::string::String, i64)> = rows
        .iter()
        .filter(|r| r.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleE))
        .map(|r| (status_text(&r.label), r.amount))
        .collect();
    for (label, amount) in [
        ("Schedule E 4 — Royalty 3 line 4 royalties", 500_000),
        ("Schedule E 10 — Royalty 3 line 10 legal and professional", 100_000),
        ("Schedule E 20 — Royalty 3 line 20 total expenses", 100_000),
        ("Schedule E 21 — Royalty 3 line 21 income", 400_000),
        ("Schedule E 23b — Schedule E line 23b royalties received", 500_000),
        ("Schedule E 3 — Rental 2 line 3 rents", 600_000),
        ("Schedule E 18 — Rental 2 line 18 depreciation", 160_000),
        ("Schedule E 20 — Rental 2 line 20 total expenses", 600_000),
        ("Schedule E 26 — Schedule E rental royalty total", 400_000),
    ] {
        assert!(
            schedule_e.iter().any(|(shown, actual)| shown == label && *actual == amount),
            "{label}"
        );
    }
}

#[test]
fn vacation_home_expense_allocation_and_limit_match_publication_527_example() {
    // 60 fair rental days, 30 personal days: 30 > max(14, 10% x 60 = 6), so
    // the unit is a home (Pub 527 ch. 5). Rental share 60/90 = 2/3.
    // Worksheet 5-1: line 1 rent 6,000; line 2 interest 2,000 + taxes 800
    // = 2,800; line 3 = 3,200; line 4e operating 1,600; line 4f = 1,600;
    // line 5 = 1,600; line 6d depreciation 2,666.67; line 6e = 1,600;
    // line 7a = 0; line 7b = 1,066.67. Schedule E net = 6,000 - 2,800
    // - 1,600 - 1,600 = 0, outside the passive rules; AGI stays $60,000.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 60, 30, [0, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "rental_2_depreciation"), 1600);
    assert_eq!(cents(&r, "rental_2_home_operating_carryover"), 0);
    assert_eq!(cents(&r, "rental_2_home_depreciation_carryover"), 106_667);
    assert_eq!(value(&r, "agi"), 60_000);
    // Eligible personal components are allocated automatically, never entered twice.
    assert!(
        r.review_notes.iter().any(|n| n.display.key == "NOTE_RENTAL_PERSONAL_EXPENSE_SHARE"),
        "{r:?}"
    );

    // Next year's carryovers enter lines 4d and 6c: operating 1,600 + 500
    // = 2,100 fits line 3 (3,200); line 5 = 1,100; depreciation
    // 2,666.67 + 1,000 = 3,666.67 is limited to 1,100; line 7b = 2,566.67.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 60, 30, [500, 1000]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_2_depreciation"), 1100);
    assert_eq!(cents(&r, "rental_2_home_operating_carryover"), 0);
    assert_eq!(cents(&r, "rental_2_home_depreciation_carryover"), 256_667);

    // 14-day boundary: 14 personal days is not more than 14, so the unit
    // is not a home. Expenses are still divided 60/74 and the loss enters
    // the passive rules: 6,000 - 4,200 x 60/74 (3,405.41) - 2,400 x 60/74
    // (1,945.95) - 4,000 x 60/74 (3,243.24) = -2,594.60, rounded -2,595,
    // fully allowed under the $25,000 active-participation allowance.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 60, 14, [0, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -2595);
    assert_eq!(value(&r, "rental_2_depreciation"), 3243);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("rental_2_home_")));

    // 10% boundary: 200 fair rental days make 20 personal days not more
    // than 10%: 6,000 - 10,600 x 200/220 (9,636.36) = -3,636. With 21
    // personal days the unit is a home and the limit leaves zero:
    // 200/221 shares are fixed 3,800.90, operating 2,171.95, depreciation
    // 3,619.91; line 3 = 2,199.10; line 5 = 27.15; line 7b = 3,592.76.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 200, 20, [0, 0]);
    assert_eq!(value(&supported(&d), "rental_income"), -3636);
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 200, 21, [0, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(cents(&r, "rental_2_home_depreciation_carryover"), 359_276);

    // The standard route limits all interest and taxes with operating costs.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 2000, 60, 30, [0, 0]);
    assert_eq!(value(&supported(&d), "rental_income"), 0);
}
