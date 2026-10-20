use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

fn credit(profile: &str) -> BusinessCredit {
    match profile {
        "nuclear" => BusinessCredit::ZeroEmissionNuclear,
        "investment" => BusinessCredit::CleanElectricity,
        "production" => BusinessCredit::CleanElectricityProduction,
        "fuel" => BusinessCredit::CleanFuel,
        "generic-renewable" => BusinessCredit::RenewableElectricity,
        _ => panic!("unknown applicable-credit profile"),
    }
}
fn draft(credit: BusinessCredit) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 5_716_000, 0);
    own(&mut d, credit, 6000);
    d
}
fn assert_return(d: &FederalDraft, credit: BusinessCredit, year: u32) {
    let r = supported(d);
    // Original ordinary F4 monetary composition and conservation, unchanged.
    assert_eq!(value(&r, "taxable_income"), 24960);
    assert_eq!(value(&r, "regular_income_tax"), 2501);
    assert_eq!(value(&r, "general_business_credit"), 2501);
    assert_eq!(
        value(
            &r,
            &alloc::format!("business_credit_{}_current_year_carryback_{year}", credit.key())
        ),
        3000
    );
    assert_eq!(
        value(&r, &alloc::format!("business_credit_{}_current_year_to_next_year", credit.key())),
        499
    );
}
#[test]
fn unit16_applicable_four_classes_keep_three_year_window_and_original_origin_guards() {
    for profile in ["nuclear", "investment", "production", "fuel"] {
        let credit = credit(profile);
        for year in 2023..=2025 {
            let mut d = draft(credit);
            history(&mut d).carrybacks.push(BusinessCreditCarryback {
                credit,
                year,
                amount: 300_000,
                research_source: None,
            });
            assert!(history(&mut d).rule().is_none());
            assert_return(&d, credit, year);
            assert_eq!(supported(&restart(d.clone()).draft), supported(&d));
            history(&mut d).carrybacks[0].amount = 350_000;
            assert!(
                estimate(&d)
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_CREDIT_CARRYBACK_LIMIT")
            );
        }
        let mut h = BusinessCreditHistory {
            credits: vec![],
            carryforwards: vec![],
            carrybacks: vec![BusinessCreditCarryback {
                credit,
                year: 2022,
                amount: 100,
                research_source: None,
            }],
            spouse_without_credits: false,
        };
        assert!(h.rule().is_some());
        h.carrybacks.clear();
        h.carryforwards.push(carry(credit, credit.first_year() - 1, 1));
        assert!(h.rule().is_some());
        h.carryforwards[0].origin_year = credit.first_year();
        assert!(h.rule().is_none());
    }
    assert_eq!(BusinessCredit::DifferentialWage.carryback_first_year(), 2025);
    assert_eq!(BusinessCredit::MarginalWells.carryback_first_year(), 2021);
    assert_eq!(BusinessCredit::AdvancedManufacturingInvestment.carryback_first_year(), 2025);
}
#[test]
fn unit16_applicable_source_editor_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, navigate, rejected_with};
    let selected = std::env::var("TAX_UNIT16_APPLICABLE_PROFILE").ok();
    let profiles = selected
        .as_deref()
        .map_or_else(|| vec!["nuclear", "investment", "production", "fuel"], |p| vec![p]);
    let year = std::env::var("TAX_UNIT16_APPLICABLE_YEAR").map_or(2023, |v| v.parse().unwrap());
    for profile in profiles {
        let credit = credit(profile);
        let d = draft(credit);
        let mut session = Interview::new(TEST_USER, d);
        let native = NativeInterviewCase("TAX_UNIT16_APPLICABLE");
        if native.calculation(&mut session) {
            return;
        }
        let open = |session: &mut Interview| {
            let d = session.draft.clone();
            match send(session, navigate(&d, TaxSection::MoreCredits)) {
                Reply::Edit(p @ P::SaveMoreCredits { .. }) => p,
                other => panic!("expected More credits: {other:?}"),
            }
        };
        let seed = open(&mut session);
        native.write("SEED", &seed);
        let mut answers = seed;
        let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
        let MoreCredit::BusinessCredits(h) =
            value.credits.iter_mut().find(|v| matches!(v, MoreCredit::BusinessCredits(_))).unwrap()
        else {
            panic!()
        };
        h.carrybacks = vec![
            BusinessCreditCarryback { credit, year, amount: 300_000, research_source: None }.into(),
        ];
        native.write("ANSWERS", &answers);
        if std::env::var("TAX_UNIT16_APPLICABLE_SEED_ONLY").as_deref() == Ok("1") {
            return;
        }
        let requested = native.input("PAYLOAD").unwrap_or(answers);
        let accepted_year = if profile == "generic-renewable" && year < 2025 {
            let before = session.draft.clone();
            rejected_with(send(&mut session, requested), "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED");
            assert_eq!(session.draft, before);
            let mut corrected = open(&mut session);
            let P::SaveMoreCredits { value, .. } = &mut corrected else { panic!() };
            let MoreCredit::BusinessCredits(h) = value
                .credits
                .iter_mut()
                .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                .unwrap()
            else {
                panic!()
            };
            h.carrybacks = vec![
                BusinessCreditCarryback {
                    credit,
                    year: 2025,
                    amount: 300_000,
                    research_source: None,
                }
                .into(),
            ];
            accepted(&send(&mut session, corrected));
            2025
        } else {
            accepted(&send(&mut session, requested));
            year
        };
        assert_return(&session.draft, credit, accepted_year);
        let saved = session.draft.clone();
        session = restart(saved.clone());
        assert_eq!(session.draft, saved);
        let reopened = open(&mut session);
        native.write("REOPENED", &reopened);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        assert_return(&session.draft, credit, accepted_year);
        // The removed client clause does not remove native Save validation.
        let mut invalid = open(&mut session);
        let P::SaveMoreCredits { value, .. } = &mut invalid else { panic!() };
        let MoreCredit::BusinessCredits(h) =
            value.credits.iter_mut().find(|v| matches!(v, MoreCredit::BusinessCredits(_))).unwrap()
        else {
            panic!()
        };
        h.carrybacks[0].year = 2022;
        rejected_with(send(&mut session, invalid), "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED");
        assert_eq!(session.draft, saved);
        assert_eq!(restart(session.draft.clone()).draft, saved);
    }
}

// The dated diagnostic retains the original 2023/2024 admission and the
// independent red statutory witness. This approved correction changes only
// generic line1f; actual line4e origins and all four applicable classes keep
// their separate native paths.
#[test]
fn unit16_generic_renewable_ordinary_window_preserves_original_f4_figures() {
    let credit = BusinessCredit::RenewableElectricity;
    assert_eq!(credit.carryback_first_year(), 2025);
    for year in [2023, 2024, 2025] {
        let mut d = draft(credit);
        history(&mut d).carrybacks.push(BusinessCreditCarryback {
            credit,
            year,
            amount: 300_000,
            research_source: None,
        });
        if year == 2025 {
            assert_eq!(history(&mut d).rule(), None);
            assert_eq!(value(&supported(&d), "agi"), 57160);
            assert_return(&d, credit, year);
            assert_eq!(supported(&restart(d.clone()).draft), supported(&d));
        } else {
            assert_eq!(history(&mut d).rule(), Some("BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED"));
            // The estimate rejects a stored record that cannot pass Save;
            // the public Save test below checks the precise year error.
            assert!(!d.valid_stored());
            let r = estimate(&d);
            assert!(r.issues.iter().any(|issue| {
                issue.display.key
                    == "ISSUE_ACCEPTED_DATA_IS_INVALID_NO_CALCULATION_WAS_PERFORMED_7A345C87"
            }));
            assert!(r.lines.is_empty());
        }
    }
}
#[test]
fn unit16_generic_renewable_native_save_refuses_older_years_then_keeps_corrected_source() {
    use crate::tests::cross::interview::{accepted, navigate, rejected_with};
    let credit = BusinessCredit::RenewableElectricity;
    let mut d = draft(credit);
    history(&mut d).carrybacks.push(BusinessCreditCarryback {
        credit,
        year: 2025,
        amount: 300_000,
        research_source: None,
    });
    let mut session = Interview::new(TEST_USER, d);
    let open = |session: &mut Interview| {
        let d = session.draft.clone();
        match send(session, navigate(&d, TaxSection::MoreCredits)) {
            Reply::Edit(p @ P::SaveMoreCredits { .. }) => p,
            other => panic!("expected ordinary More credits editor: {other:?}"),
        }
    };
    let saved = session.draft.clone();
    for year in [2023, 2024] {
        let mut answers = open(&mut session);
        let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
        let MoreCredit::BusinessCredits(h) =
            value.credits.iter_mut().find(|v| matches!(v, MoreCredit::BusinessCredits(_))).unwrap()
        else {
            panic!()
        };
        h.carrybacks[0].year = year;
        rejected_with(send(&mut session, answers), "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED");
        assert_eq!(session.draft, saved);
        assert_eq!(restart(session.draft.clone()).draft, saved);
    }
    let corrected = open(&mut session);
    accepted(&send(&mut session, corrected));
    assert_eq!(session.draft, saved);
    assert_return(&session.draft, credit, 2025);
    session = restart(session.draft.clone());
    let reopened = open(&mut session);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_return(&session.draft, credit, 2025);
}
