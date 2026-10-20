// Original R5/R6 monetary controls, exercised through ordinary new-row Save.
use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
extern crate std;

fn assert_profile(d: &FederalDraft, profile: &str) {
    let r = supported(d);
    if profile.starts_with("elective-") {
        let elected = if profile == "elective-full" { 50000 } else { 30000 };
        let net = if elected == 50000 { 42162 } else { 30000 };
        assert_eq!(value(&r, "agi"), 216100);
        assert_eq!(value(&r, "general_business_credit"), 7838);
        assert_eq!(column(&r, 3, "1b", 'h'), elected);
        assert_eq!(column(&r, 3, "1b", 'i'), 7838);
        assert_eq!(column(&r, 3, "1b", 'j'), net);
        assert_eq!(column(&r, 3, "6", 'j'), net);
        assert_eq!(value(&r, "schedule_3_elective_payment"), net);
        assert_eq!(
            value(
                &r,
                "business_credit_advanced_manufacturing_production_current_year_to_next_year"
            ),
            42162 - net
        );
    } else {
        assert_eq!(value(&r, "agi"), 116100);
        if profile == "sold" {
            assert_eq!(value(&r, "general_business_credit"), 3500);
            for (c, a) in [('d', 0), ('e', 5000), ('f', -1500), ('g', 3500), ('i', 3500)] {
                assert_eq!(column(&r, 3, "1b", c), a);
            }
        } else {
            assert_eq!(profile, "bought");
            assert_eq!(value(&r, "general_business_credit"), 0);
            assert_eq!(column(&r, 3, "1b", 'e'), 0);
            assert_eq!(column(&r, 3, "1b", 'f'), 5000);
            assert_eq!(
                value(
                    &r,
                    "business_credit_advanced_manufacturing_production_current_year_passive_to_next_year"
                ),
                5000
            );
        }
    }
}

#[test]
fn unit16_r5_r6_fresh_row_election_transfer_editor_save_reopen_restart() {
    use crate::tests::cross::interview::{accepted, navigate, rejected_with};
    let profiles = std::env::var("TAX_UNIT16_ELECTION_PROFILE").map_or_else(
        |_| vec!["elective-full".into(), "elective-part".into(), "sold".into(), "bought".into()],
        |p| vec![p],
    );
    for profile in profiles {
        let mut d = complete(FederalStatus::Single);
        let elective = profile.starts_with("elective-");
        d.income[0] = wage(1, Owner::Taxpayer, if elective { 21_610_000 } else { 11_610_000 }, 0);
        own(
            &mut d,
            BusinessCredit::AdvancedManufacturingProduction,
            if elective { 50000 } else { 5000 },
        );
        let mut item = history(&mut d).credits.pop().unwrap();
        item.election = match profile.as_str() {
            "elective-full" => CreditElection::ElectivePayment { amount: 5_000_000 },
            "elective-part" => CreditElection::ElectivePayment { amount: 3_000_000 },
            "sold" => CreditElection::Sold { amount: 150_000 },
            "bought" => {
                item.source = CreditSource::Bought;
                CreditElection::None
            }
            _ => panic!(),
        };
        let original_income = d.income.clone();
        let mut session = Interview::new(TEST_USER, d);
        let open = |s: &mut Interview| {
            let d = s.draft.clone();
            match send(s, navigate(&d, TaxSection::MoreCredits)) {
                Reply::Edit(p @ P::SaveMoreCredits { .. }) => p,
                other => panic!("{other:?}"),
            }
        };
        let native = NativeInterviewCase("TAX_UNIT16_ELECTION");
        if native.required_calculation(&mut session) {
            return;
        }
        let seed = open(&mut session);
        native.write_file("DIR", "seed.avro", &seed);
        let mut answers = seed;
        let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
        let MoreCredit::BusinessCredits(h) =
            value.credits.iter_mut().find(|v| matches!(v, MoreCredit::BusinessCredits(_))).unwrap()
        else {
            panic!()
        };
        assert!(h.credits.is_empty());
        h.credits.push(item.clone().into());
        native.write_file("DIR", "answers.avro", &answers);
        if std::env::var("TAX_UNIT16_ELECTION_SEED_ONLY").as_deref() == Ok("1") {
            return;
        }
        let payload = native.input("PAYLOAD").unwrap_or(answers);
        accepted(&send(&mut session, payload));
        assert_eq!(session.draft.income, original_income);
        assert_profile(&session.draft, &profile);
        let saved = session.draft.clone();
        session = restart(saved.clone());
        assert_eq!(session.draft, saved);
        let reopened = open(&mut session);
        native.write_file("DIR", "reopened.avro", &reopened);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        assert_profile(&session.draft, &profile);
        if let Ok(dir) = std::env::var("TAX_UNIT16_ELECTION_DIR") {
            write_native_draft(std::path::Path::new(&dir).join("persisted.avro"), &saved);
        }
        let mut invalid = open(&mut session);
        let P::SaveMoreCredits { value, .. } = &mut invalid else { panic!() };
        let MoreCredit::BusinessCredits(h) =
            value.credits.iter_mut().find(|v| matches!(v, MoreCredit::BusinessCredits(_))).unwrap()
        else {
            panic!()
        };
        h.credits[0].election = if profile == "bought" {
            CreditElection::Sold { amount: 100 }
        } else if elective {
            CreditElection::ElectivePayment { amount: item.amount + 100 }
        } else {
            CreditElection::Sold { amount: item.amount + 100 }
        };
        rejected_with(
            send(&mut session, invalid),
            if profile == "bought" {
                "BOUGHT_CREDIT_CANNOT_BE_SOLD"
            } else {
                "BUSINESS_CREDIT_ELECTION_ABOVE_CREDIT"
            },
        );
        assert_eq!(session.draft, saved);
    }
}
