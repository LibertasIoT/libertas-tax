use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn history(gross: i64, sold: i64, years: &[(u32, i64)]) -> Section179IncomeCarryHistory {
    Section179IncomeCarryHistory {
        gross,
        sold,
        vintages: years.iter().map(|&(year, amount)| Asset179Carryover { year, amount }).collect(),
    }
}

fn page(partner: bool, actual: Option<Section179IncomeCarryHistory>) -> IncomeDocument {
    if partner {
        let mut page = k1_1065("Actual partnership carry history");
        page.ordinary = 2_000_000;
        page.interest = PartnerInterest::LlcMember {
            box1_in_se: false,
            participation: Participation::Over500Hours { hours: 501 },
        };
        page.basis.start = k1_1065_continuing(
            1_000_000,
            ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
        );
        page.more.section179_carryover = 100_000;
        page.more.amt_section179_income_carry = actual;
        IncomeDocument::K1Partnership(page)
    } else {
        let mut page = k1_1120s("Actual corporation carry history");
        page.ordinary = 2_000_000;
        page.participation = Participation::Over500Hours { hours: 501 };
        page.basis.start = k1_1120s_continuing(1_000_000, 1_000_000);
        page.more.section179_carryover = 100_000;
        page.more.amt_section179_income_carry = actual;
        IncomeDocument::K1Scorp(page)
    }
}

fn set_history(value: &mut IncomeDocument, actual: Option<Section179IncomeCarryHistory>) {
    match value {
        IncomeDocument::K1Partnership(page) => page.more.amt_section179_income_carry = actual,
        IncomeDocument::K1Scorp(page) => page.more.amt_section179_income_carry = actual,
        _ => unreachable!(),
    }
}

fn stored(entry: &FederalIncomeEntry) -> (i64, &Option<Section179IncomeCarryHistory>) {
    match &entry.income {
        FederalIncome::Partnership { data } => {
            (data.section179_carryover, &data.amt_section179_income_carry)
        }
        FederalIncome::Scorp { data } => {
            (data.section179_carryover, &data.amt_section179_income_carry)
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit19_amt_179_history_native_save_preserves_unknown_zero_and_actual_years() {
    for partner in [true, false] {
        for actual in [
            None,
            Some(history(0, 0, &[])),
            Some(history(30_000, 0, &[(2024, 30_000)])),
            Some(history(50_000, 20_000, &[(2023, 20_000), (2025, 10_000)])),
        ] {
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            let mut request = open_topic(&mut session, TaxSection::PassThrough);
            let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
            documents.documents = vec![document(0, page(partner, actual.clone()))];
            accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
            let entry = session
                .draft
                .income
                .iter()
                .find(|entry| {
                    matches!(
                        entry.income,
                        FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. }
                    )
                })
                .unwrap();
            assert_eq!(stored(entry), (100_000, &actual));
            let id = entry.id;
            let restored = restart(session.draft.clone());
            assert_eq!(restored.draft, session.draft);
            assert_eq!(
                stored(restored.draft.income.iter().find(|entry| entry.id == id).unwrap()),
                (100_000, &actual)
            );

            // Real corrections preserve identity and never copy regular carry
            // into either explicit zero or unavailable AMT history.
            for correction in [Some(history(0, 0, &[])), None] {
                let mut request = open_topic(&mut session, TaxSection::PassThrough);
                let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
                set_history(&mut documents.documents[0].document, correction.clone());
                accepted(&send(&mut session, request));
                assert_eq!(
                    stored(session.draft.income.iter().find(|entry| entry.id == id).unwrap()),
                    (100_000, &correction)
                );
                assert_eq!(restart(session.draft.clone()).draft, session.draft);
            }
        }
    }
}

#[test]
fn unit19_amt_179_history_invalid_save_keeps_the_accepted_source() {
    for partner in [true, false] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        documents.documents =
            vec![document(0, page(partner, Some(history(30_000, 0, &[(2024, 30_000)]))))];
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        for invalid in [
            history(30_000, 30_001, &[]),
            history(-1, 0, &[]),
            history(30_000, 0, &[(2026, 30_000)]),
            history(30_000, 0, &[(1957, 30_000)]),
            history(30_000, 0, &[(2024, 20_000), (2024, 10_000)]),
            history(30_000, 0, &[(2024, 29_999)]),
            history(30_000, 0, &[(2024, 30_000), (2025, 0)]),
        ] {
            assert!(invalid.rule().is_some());
            let mut request = open_topic(&mut session, TaxSection::PassThrough);
            let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
            set_history(&mut documents.documents[0].document, Some(invalid));
            error(send(&mut session, request));
            assert_eq!(session.draft, saved);
        }
    }
}

#[test]
fn unit19_amt_179_history_community_projection_conserves_each_actual_year() {
    use crate::community::Divide;
    let actual = history(7, 1, &[(2023, 1), (2024, 2), (2025, 3)]);
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    let income = FederalIncome::Partnership { data: ordinary_partner_source(0, true) };
    let setup = draft.setup.as_ref().unwrap();
    for (owner, counterpart) in [(Owner::Taxpayer, Owner::Spouse), (Owner::Spouse, Owner::Taxpayer)]
    {
        let share =
            crate::community::share(setup, owner, CommunityShare::Community, &income).unwrap();
        let other_share =
            crate::community::share(setup, counterpart, CommunityShare::Community, &income)
                .unwrap();
        let own = actual.divided(share);
        let other = actual.divided(other_share);
        assert_eq!(own.rule(), None);
        assert_eq!(other.rule(), None);
        assert_eq!(own.gross + other.gross, 7);
        assert_eq!(own.sold + other.sold, 1);
        for original in &actual.vintages {
            let amount = |value: &Section179IncomeCarryHistory| {
                value.vintages.iter().find(|v| v.year == original.year).map_or(0, |v| v.amount)
            };
            assert_eq!(amount(&own) + amount(&other), original.amount);
        }
    }
    assert_eq!(actual, history(7, 1, &[(2023, 1), (2024, 2), (2025, 3)]));
}

fn entered(value: &IncomeDocument) -> &Option<Section179IncomeCarryHistory> {
    match value {
        IncomeDocument::K1Partnership(page) => &page.more.amt_section179_income_carry,
        IncomeDocument::K1Scorp(page) => &page.more.amt_section179_income_carry,
        _ => unreachable!(),
    }
}

#[test]
fn unit19_amt_179_history_actual_client_save_reopen_restart_and_correction() {
    extern crate std;
    let partner = std::env::var("TAX_UNIT19_AMT179_KIND").as_deref() != Ok("scorp");
    let actual = [Some(history(30_000, 0, &[(2024, 30_000)])), None, Some(history(0, 0, &[]))];
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    let native = NativeInterviewCase("TAX_UNIT19_AMT179");
    if native.required_calculation(&mut session) {
        return;
    }
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = actual
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let mut value = page(partner, value.clone());
            match &mut value {
                IncomeDocument::K1Partnership(page) => {
                    page.name = alloc::format!("Actual AMT history source {}", index + 1)
                }
                IncomeDocument::K1Scorp(page) => {
                    page.name = alloc::format!("Actual AMT history source {}", index + 1)
                }
                _ => unreachable!(),
            }
            document(0, value)
        })
        .collect();
    let directory = std::env::var("TAX_UNIT19_AMT179_DIR").ok().map(std::path::PathBuf::from);
    let write = |name: &str, request: &P| {
        if let Some(directory) = &directory {
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(directory.join(name), request.to_avro()).unwrap();
        }
    };
    write("seed.avro", &seed);
    write("answers.avro", &answers);
    if std::env::var("TAX_UNIT19_AMT179_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let payload = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, P::from_avro(&payload.to_avro()).unwrap()));
    let saved = session.draft.clone();
    let sources: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| {
            matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
        })
        .collect();
    assert_eq!(sources.len(), 3);
    for (entry, expected) in sources.iter().zip(actual.iter()) {
        assert_eq!(stored(entry), (100_000, expected));
    }
    let ids: Vec<_> = sources.iter().map(|entry| entry.id).collect();
    assert_eq!(ids.iter().collect::<alloc::collections::BTreeSet<_>>().len(), 3);
    let result = supported(&saved);
    let mut without = saved.clone();
    for entry in &mut without.income {
        match &mut entry.income {
            FederalIncome::Partnership { data } => data.amt_section179_income_carry = None,
            FederalIncome::Scorp { data } => data.amt_section179_income_carry = None,
            _ => {}
        }
    }
    assert_eq!(supported(&without), result); // Adapter only; allocator remains its owner's work.
    let records = native_draft_records(&saved);
    let draft = draft_from_native_records(saved.tax_year, &records);
    assert_eq!(native_draft_records(&draft), records);
    assert_eq!(draft, saved);
    if let Some(directory) = &directory {
        write_native_draft_records(directory.join("saved-draft.avro"), &records);
    }
    let mut session = restart(draft);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(
        documents.documents.iter().map(|row| entered(&row.document)).collect::<Vec<_>>(),
        actual.iter().collect::<Vec<_>>()
    );
    write("reopened.avro", &reopened);
    let payload = native.input("REOPEN_PAYLOAD").unwrap_or(reopened);
    accepted(&send(&mut session, payload));
    assert_eq!(session.draft, saved);
    let mut correction = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let changed = Some(history(50_000, 20_000, &[(2023, 20_000), (2025, 10_000)]));
    set_history(&mut documents.documents[0].document, changed.clone());
    documents.documents.reverse();
    accepted(&send(&mut session, correction));
    for entry in session.draft.income.iter().filter(|entry| {
        matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
    }) {
        let expected = if entry.id == ids[0] {
            &changed
        } else if entry.id == ids[1] {
            &actual[1]
        } else {
            assert_eq!(entry.id, ids[2]);
            &actual[2]
        };
        assert_eq!(stored(entry), (100_000, expected));
    }
    let corrected = supported(&session.draft);
    // Three actual $20,000 ordinary sources, each releasing $1,000 regular
    // income-limit carry, plus the existing $60,000 wage. Changing the AMT
    // history and source order does not change that existing regular owner.
    for value in [&result, &corrected] {
        assert_eq!(super::super::value(value, "agi"), 117_000);
        assert_eq!(value.tax, Some(1_691_000));
        assert_eq!(value.payments, Some(650_000));
        assert_eq!(value.balance, Some(1_041_000));
    }
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
