// Genuine native clergy/ScheduleH sources at the shared Form8379/Form3800 boundary.
use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

#[test]
fn unit16_r4_household_source_keys_use_actual_employer_tax_and_order() {
    let mut d = crate::tests::credits::form_3800::changed_filer_profile("household-source");
    for reverse in [false, true] {
        if reverse {
            d.screening.as_mut().unwrap().household_employment.reverse();
        }
        assert!(d.injured_spouse.is_none());
        let r = supported(&d);
        assert_eq!(value(&r, "household_0_tax"), 795);
        assert_eq!(value(&r, "household_1_tax"), 557);
        assert_eq!(value(&r, "agi"), 80_000);
        assert_eq!(value(&r, "form_3800_line_16"), 5243);
        // Native MFS5023/393 plus employer795/557 ->5818/950,
        // ratio8596/10000; actual joint section38 cap5243 ->4507.
        assert_eq!(value(&r, "general_business_credit"), 4507);
        assert_eq!(
            value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
            5493
        );
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
    }
}

#[test]
fn unit16_clergy_native_housing_source_reaches_both_injured_owner_views() {
    let mut d = crate::tests::credits::form_3800::changed_filer_profile("clergy-source");
    d.credits.as_mut().unwrap().business_credits = None;
    for entry in &mut d.income {
        if let FederalIncome::Wage { data } = &mut entry.income {
            data.withholding = 1_000_000;
        }
    }
    let joint = supported(&d);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut claim = allocation();
        claim.injured = owner;
        d.injured_spouse = Some(claim);
        d.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
        let r = supported(&d);
        let first = usize::from(owner == Owner::Spouse);
        assert_eq!(column(&r, "13a", "b"), [59500, 20000][first]);
        assert_eq!(column(&r, "13b", "b"), [2233, 0][first]);
        assert_eq!(column(&r, "14", "b"), [6542, 0][first]);
        assert_eq!(column(&r, "18", "b"), [13084, 0][first]);
        let tax = r.lines.iter().find(|row| row.id == "form_8379_injured_share_of_tax").unwrap();
        assert_eq!(tax.explanation.args[1], MessageArg::usd_dollars([4393, 393][first]));
        assert_eq!(tax.explanation.args[2], MessageArg::usd_dollars([4393, 393][1 - first]));
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        assert_eq!(supported(&restart(d.clone()).draft), r);
    }
}

#[test]
fn unit16_clergy_source_cents_reconcile_before_existing_selected_owner_rounding() {
    for amounts in [[49, 50], [50, 49], [49, 49], [0, 50], [50, 0]] {
        let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
        d.screening.as_mut().unwrap().clergy = [Owner::Taxpayer, Owner::Spouse]
            .into_iter()
            .zip(amounts)
            .map(|(owner, excess)| ClergyClaim {
                owner,
                role: ClergyRole::Minister {
                    housing: MinisterHousing::Parsonage {
                        fair_rental_value: 0,
                        utility_allowance: excess,
                        utilities_paid: 0,
                    },
                    form_4361: true,
                    employee_expenses: 0,
                },
            })
            .collect();
        for first in [0, 1] {
            d.injured_spouse.as_mut().unwrap().injured =
                if first == 0 { Owner::Taxpayer } else { Owner::Spouse };
            let r = supported(&d);
            // Native line1h rounds the combined50..99c to1. The existing
            // projection rounds only the selected owner's actual source and
            // gives the other owner the joint remainder. No per-source floors.
            let selected = i64::from(amounts[first] >= 50);
            assert_eq!(value(&r, "agi"), 80001);
            assert_eq!(column(&r, "13b", "b"), selected, "{amounts:?}, first{first}");
            assert_eq!(column(&r, "13b", "c"), 1 - selected);
            assert_eq!(column(&r, "13b", "a"), 1);
            let mut ordinary = d.clone();
            ordinary.injured_spouse = None;
            let joint = supported(&ordinary);
            assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
            assert_eq!(supported(&restart(d.clone()).draft), r);
        }
    }
}

#[test]
fn unit16_clergy_wages_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, document, open_topic};
    let mut d = crate::tests::credits::form_3800::changed_filer_profile("clergy-source");
    let claim = d.screening.as_mut().unwrap().clergy.pop().unwrap();
    let mut session = Interview::new(TEST_USER, d);
    let seed = open_topic(&mut session, TaxSection::Wages);
    let native = NativeInterviewCase("TAX_UNIT16_CLERGY");
    if native.required_calculation(&mut session) {
        return;
    }
    native.write_file("DIR", "seed.avro", &seed);
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
    let mut source = document(0, IncomeDocument::Clergy(ClergyRecord { role: claim.role.clone() }));
    source.owner = claim.owner;
    documents.documents.push(source);
    native.write_file("DIR", "answers.avro", &answers);
    if std::env::var("TAX_UNIT16_CLERGY_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let payload = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, payload));
    assert_eq!(session.draft.screening.as_ref().unwrap().clergy, vec![claim]);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "clergy_taxpayer_housing_excess"), 500);
    assert_eq!(value(&r, "agi"), 75191);
    assert_eq!(value(&r, "general_business_credit"), 4512);
    assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(session.draft.tax_year, &stored);
    assert_eq!(decoded, session.draft);
    assert_eq!(native_draft_records(&decoded), stored);
    session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::Wages);
    native.write_file("DIR", "reopened.avro", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(supported(&session.draft), r);
    if let Ok(dir) = std::env::var("TAX_UNIT16_CLERGY_DIR") {
        write_native_draft_records(std::path::Path::new(&dir).join("persisted.avro"), &stored);
    }
}
