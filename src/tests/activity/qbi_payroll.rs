use crate::tests::support::client::estimate_documents_from_save;
use crate::tests::support::client::{read_native_draft, write_native_draft};
extern crate std;
use super::*;
use crate::community::Divide;

fn attribution(amount: i64, confirmations: [bool; 4]) -> PayrollQbiAttribution {
    PayrollQbiAttribution::Allocated {
        qualified_wages: amount,
        records_establish_attribution: confirmations[0],
        timely_correctly_reported: confirmations[1],
        employer_entitled: confirmations[2],
        ordinary_domestic_no_special_case: confirmations[3],
    }
}
fn payroll(boxes: &[(i64, i64)], amount: i64) -> BusinessPayroll {
    BusinessPayroll::Yes {
        statements: boxes
            .iter()
            .enumerate()
            .map(|(i, &(wages, medicare))| PayrollW2 {
                label: alloc::format!("Employee {}", i + 1),
                wages,
                medicare,
            })
            .collect(),
        attribution: attribution(amount, [true; 4]),
    }
}

#[test]
fn qbi_partial_payroll_uses_documented_quantity_and_aggregate_boxes() {
    // Crossed boxes: min(10001+20001,20001+10001)=30002;
    // the invalid per-form minimum method would cap at20002.
    let p = payroll(&[(10001, 20001), (20001, 10001)], 25001);
    assert!(p.valid());
    assert_eq!(p.qbi_wages(), 25001);
    assert!(!p.qbi_unavailable());
    for lowered in [vec![(10000, 20001), (15000, 10001)], vec![(10001, 10000), (20001, 15000)]] {
        let p = payroll(&lowered, 25001);
        assert_eq!(p.rule(), Some("QBI_WAGE_PART_SOURCE"));
    }
    assert!(payroll(&[(10001, 20001), (20001, 10001)], 30002).valid());
    assert!(!payroll(&[(10001, 20001), (20001, 10001)], 30003).valid());
}

#[test]
fn qbi_partial_payroll_required_confirmations_preserve_source_bounds_and_confirmed_zero() {
    for i in 0..4 {
        {
            let answer = false;
            let mut facts = [true; 4];
            facts[i] = answer;
            let mut p = payroll(&[(30099, 60099)], 30098);
            let BusinessPayroll::Yes { attribution: a, .. } = &mut p else { panic!() };
            *a = attribution(30098, facts);
            assert!(p.valid());
            assert!(p.qbi_unavailable());
            assert_eq!(p.qbi_wages(), 0);
            let BusinessPayroll::Yes { attribution: a, .. } = &mut p else { panic!() };
            *a = attribution(30100, facts);
            assert!(!p.valid(), "an unavailable confirmation cannot authorize excess source wages");
        }
    }
    let p = payroll(&[(30099, 60099)], 0);
    assert!(p.valid());
    assert!(!p.qbi_unavailable());
    assert_eq!(p.qbi_wages(), 0);
    let BusinessPayroll::Yes { statements, .. } = p else { panic!() };
    let p = BusinessPayroll::Yes { statements, attribution: PayrollQbiAttribution::Unavailable };
    assert!(p.valid());
    assert!(p.qbi_unavailable());
    assert_eq!(p.qbi_wages(), 0);
}

#[test]
fn qbi_partial_payroll_odd_cent_source_divides_quantity_once() {
    let p = payroll(&[(10033, 20033); 3], 30098);
    assert!(p.valid());
    let mut setup = complete(FederalStatus::Separate).setup.unwrap();
    setup.community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    let entry = cash_business(1000);
    let projected: Vec<_> = [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| {
            let share =
                crate::community::share(&setup, owner, CommunityShare::Community, &entry).unwrap();
            p.divided(share)
        })
        .collect();
    assert_eq!(projected[0].qbi_wages(), 15049);
    assert_eq!(projected[1].qbi_wages(), 15049);
    for p in &projected {
        assert!(p.projected_shape_valid());
    }
    // Three independently rounded box1 values give15051 to source owner and
    //15048 to counterpart. They must not cap the valid15049 quantity again.
    assert!(!projected[1].valid());
    let mut entire = p.clone();
    let BusinessPayroll::Yes { attribution, .. } = &mut entire else { panic!() };
    *attribution = PayrollQbiAttribution::EntireBusiness;
    let actual: Vec<_> = [Owner::Taxpayer, Owner::Spouse]
        .into_iter()
        .map(|owner| {
            let share =
                crate::community::share(&setup, owner, CommunityShare::Community, &entry).unwrap();
            entire.divided(share).qbi_wages()
        })
        .collect();
    assert_eq!(actual, vec![15051, 15048]);
    assert_eq!(actual.iter().sum::<i64>(), 30099);
}

#[test]
fn qbi_partial_payroll_long_utf8_labels_preserve_amount_source_integrity() {
    // D-2026-10-15-13 removes only the descriptive label maximum.
    for label in ["a".repeat(400), "é".repeat(200), "😀".repeat(100), "a\u{301}".repeat(133) + "a"]
    {
        assert_eq!(label.len(), 400);
        let mut p = payroll(&[(10033, 20033); 3], 30098);
        let BusinessPayroll::Yes { statements, .. } = &mut p else { panic!() };
        statements[0].label = label;
        assert!(p.valid());
        let BusinessPayroll::Yes { statements, .. } = &mut p else { panic!() };
        statements[0].label.push('a');
        assert!(p.valid());
        assert_eq!(p.qbi_wages(), 30098);
    }
    let mut p = payroll(&[(i64::MAX, i64::MAX); 3], 1);
    assert!(!p.valid());
    let BusinessPayroll::Yes { statements, .. } = &mut p else { panic!() };
    statements[0].wages = -1;
    assert!(!p.valid());
    assert!(!payroll(&[(10033, 20033)], -1).valid());
    assert!(!payroll(&[(100_000_000_000, 100_000_000_000)], 100_000_000_001).valid());
}

#[test]
fn qbi_partial_payroll_keeps_expenses_and_employer_credit_sources() {
    let mut d = restaurant_tip_credit_draft();
    let before = supported(&d);
    let business = restaurant_business_mut(&mut d);
    let BusinessPayroll::Yes { attribution: a, .. } = &mut business.payroll else { panic!() };
    *a = attribution(0, [true; 4]);
    let after = supported(&d);
    assert_eq!(value(&before, "agi"), value(&after, "agi"));
    assert_eq!(
        value(&before, "business_2_form8846_line1"),
        value(&after, "business_2_form8846_line1")
    );
    assert_eq!(
        value(&before, "business_2_form8846_line4"),
        value(&after, "business_2_form8846_line4")
    );
    assert_eq!(restart(d.clone()).draft, d);
}

fn business_source(case: &str) -> Business {
    let mut b = crate::tests::cross::interview::business(13_000_000);
    b.name = "Actual payroll source".into();
    b.expenses.wages = 3_000_000;
    b.more.payroll = payroll(&[(1_000_100, 2_000_100), (2_000_100, 1_000_100)], 2_500_000);
    let BusinessPayroll::Yes { attribution: a, .. } = &mut b.more.payroll else { panic!() };
    *a = match case {
        "zero" => attribution(0, [true; 4]),
        "unavailable" => PayrollQbiAttribution::Unavailable,
        case if case.starts_with("no-") => {
            let (_, index) = case.split_once('-').unwrap();
            let mut facts = [true; 4];
            facts[index.parse::<usize>().unwrap()] = false;
            attribution(2_500_000, facts)
        }
        "eligible" => attribution(2_500_000, [true; 4]),
        _ => panic!("unknown payroll fixture case"),
    };
    b
}
fn base() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 40_000_000, 0);
    d
}
fn assert_result(d: &FederalDraft, case: &str) {
    let r = supported(d);
    assert_eq!(value(&r, "qbi_deduction"), if case == "eligible" { 12_500 } else { 400 });
    assert_eq!(
        r.review_notes.iter().any(|n| n.display.key == "WARN_QBI_WAGES_LEFT_OUT"),
        case != "eligible" && case != "zero"
    );
}

#[test]
fn qbi_partial_payroll_normal_save_correction_reopen_restart_is_atomic() {
    use crate::tests::cross::interview::{
        accepted, open_topic, page_documents, save_businesses, save_topic,
    };
    let mut s = Interview::new(TEST_USER, base());
    accepted(&save_businesses(&mut s, vec![business_source("eligible")]));
    assert_result(&s.draft, "eligible");
    let saved = s.draft.clone();
    let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
    let mut rows = page_documents(&reopened).to_vec();
    let IncomeDocument::Business(b) = &mut rows[0].document else { panic!() };
    let BusinessPayroll::Yes { statements, .. } = &mut b.more.payroll else { panic!() };
    statements[0].wages = 499_899; //2499999 total; documented2500000 exceeds byonecent.
    let request = save_topic(&saved, TaxSection::SelfEmployment, rows.clone());
    let Reply::Response(P::Problem { message, field }) = send(&mut s, request) else {
        panic!("expectedsource refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text("QBI_WAGE_PART_SOURCE", &[]));
    assert_eq!(
        field.as_deref(),
        Some("[documents/documents/0/document/more/payroll/attribution/qualified_wages]")
    );
    assert_eq!(s.draft, saved);
    let IncomeDocument::Business(b) = &mut rows[0].document else { panic!() };
    let BusinessPayroll::Yes { attribution, .. } = &mut b.more.payroll else { panic!() };
    *attribution = PayrollQbiAttribution::Allocated {
        qualified_wages: 2_499_998,
        records_establish_attribution: true,
        timely_correctly_reported: true,
        employer_entitled: true,
        ordinary_domestic_no_special_case: true,
    };
    let request = save_topic(&saved, TaxSection::SelfEmployment, rows);
    accepted(&send(&mut s, request));
    let accepted_draft = s.draft.clone();
    let mut restarted = restart(accepted_draft.clone());
    assert_eq!(restarted.draft, accepted_draft);
    let reopened = open_topic(&mut restarted, TaxSection::SelfEmployment);
    accepted(&send(&mut restarted, reopened));
    assert_eq!(restarted.draft.income, accepted_draft.income);
    assert_eq!(value(&supported(&restarted.draft), "agi"), value(&supported(&saved), "agi"));
    assert_result(&restarted.draft, "eligible");
}

fn payroll_cases() -> [&'static str; 7] {
    ["eligible", "zero", "unavailable", "no-0", "no-1", "no-2", "no-3"]
}

#[test]
fn qbi_partial_payroll_all_confirmations_normal_save_preserves_actual_sources() {
    use crate::tests::cross::interview::{accepted, open_topic, page_documents, save_businesses};
    for case in payroll_cases() {
        let source = business_source(case);
        let expected = source.more.payroll.clone();
        let mut s = Interview::new(TEST_USER, base());
        accepted(&save_businesses(&mut s, vec![source]));
        assert_result(&s.draft, case);
        let saved = s.draft.clone();
        let financial = supported(&saved);
        let mut s = restart(saved.clone());
        assert_eq!(s.draft, saved);
        let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
        let IncomeDocument::Business(b) = &page_documents(&reopened)[0].document else { panic!() };
        assert_eq!(b.more.payroll, expected, "{case}: preserve actual boxes and attribution");
        assert_eq!(b.expenses.wages, 3_000_000);
        accepted(&send(&mut s, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(s.draft, saved);
        assert_eq!(supported(&s.draft), financial);
    }
}

#[test]
fn qbi_partial_payroll_each_box_cap_precedes_unknowns_save_and_estimate() {
    use crate::tests::cross::interview::{
        accepted, open_topic, page_documents, save_businesses, save_topic,
    };
    for case in payroll_cases().into_iter().filter(|c| *c != "zero" && *c != "unavailable") {
        for box_five in [false, true] {
            let mut s = Interview::new(TEST_USER, base());
            accepted(&save_businesses(&mut s, vec![business_source(case)]));
            let saved = s.draft.clone();
            let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
            let mut rows = page_documents(&reopened).to_vec();
            let IncomeDocument::Business(b) = &mut rows[0].document else { panic!() };
            let BusinessPayroll::Yes { statements, .. } = &mut b.more.payroll else { panic!() };
            if box_five {
                statements[1].medicare = 499_899;
            } else {
                statements[0].wages = 499_899;
            }
            let request = save_topic(&saved, TaxSection::SelfEmployment, rows.clone());
            for (stage, request, path) in [
                (
                    "save",
                    request.clone(),
                    "[documents/documents/0/document/more/payroll/attribution/qualified_wages]",
                ),
                (
                    "estimate",
                    estimate_documents_from_save(&request),
                    "[documents/documents/0/document/more/payroll/attribution/qualified_wages]",
                ),
            ] {
                let request = P::from_avro(&request.to_avro()).unwrap();
                let dirty = s.dirty;
                let Reply::Response(problem @ P::Problem { .. }) = send(&mut s, request.clone())
                else {
                    panic!("{case}/{stage}: source cap must refuse")
                };
                let P::Problem { message, field } = &problem else { unreachable!() };
                assert_eq!(
                    *message,
                    libertas::libertas_formatted_text("QBI_WAGE_PART_SOURCE", &[])
                );
                assert_eq!(field.as_deref(), Some(path));
                assert_eq!(s.draft, saved);
                assert_eq!(s.dirty, dirty);
                crate::tests::unit47_rule_cases::export_unit47_rule_case(
                    &alloc::format!("inv84-{case}-box{}-{stage}", if box_five { 5 } else { 1 }),
                    &request,
                    &problem,
                );
            }
            let IncomeDocument::Business(b) = &mut rows[0].document else { panic!() };
            let BusinessPayroll::Yes {
                attribution: PayrollQbiAttribution::Allocated { qualified_wages, .. },
                ..
            } = &mut b.more.payroll
            else {
                panic!()
            };
            *qualified_wages = 2_499_998;
            let corrected = save_topic(&saved, TaxSection::SelfEmployment, rows);
            let before = s.draft.clone();
            accepted(&send(&mut s, estimate_documents_from_save(&corrected)));
            assert_eq!(s.draft, before);
            accepted(&send(&mut s, corrected));
            assert_result(&s.draft, case);
            let saved = s.draft.clone();
            let mut s = restart(saved.clone());
            let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
            accepted(&send(&mut s, reopened));
            assert_eq!(s.draft, saved);
        }
    }
}

#[test]
fn qbi_partial_payroll_whole_sources_survive_normal_share_save_and_correction() {
    use crate::tests::cross::interview::{
        accepted, document, open_topic, page_documents, save_topic,
    };
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = complete(FederalStatus::Separate);
        d.income[0] = wage(1, Owner::Taxpayer, 40_000_000, 0);
        d.income[0].community = CommunityShare::Community;
        d.setup.as_mut().unwrap().community_property =
            CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
        let mut b = business_source("eligible");
        b.more.payroll = payroll(&[(10033, 20033); 3], 30098);
        let source = IncomeDocument::Business(b.clone());
        let mut row = document(0, source.clone());
        row.owner = owner;
        row.community = CommunityShare::Community;
        let mut s = Interview::new(TEST_USER, d);
        let request = save_topic(&s.draft, TaxSection::SelfEmployment, vec![row]);
        accepted(&send(&mut s, request));
        for answer in [
            CommunityShare::Community,
            CommunityShare::Partly { percent: 40 },
            CommunityShare::Separate,
        ] {
            let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
            let mut rows = page_documents(&reopened).to_vec();
            assert_eq!(rows[0].document, source);
            assert_eq!(rows[0].owner, owner);
            rows[0].community = answer;
            let request = save_topic(&s.draft, TaxSection::SelfEmployment, rows);
            accepted(&send(&mut s, request));
            let saved = s.draft.clone();
            let financial = supported(&saved);
            let entry = saved
                .income
                .iter()
                .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
                .unwrap();
            let FederalIncome::Business { data } = &entry.income else { unreachable!() };
            assert_eq!(data.payroll, b.more.payroll);
            assert_eq!(
                data.expenses
                    .iter()
                    .filter(|e| e.category == BusinessExpenseCategory::Wages)
                    .map(|e| e.amount)
                    .sum::<i64>(),
                3_000_000
            );
            let mut s2 = restart(saved.clone());
            let reopened = open_topic(&mut s2, TaxSection::SelfEmployment);
            assert_eq!(page_documents(&reopened)[0].community, answer);
            assert_eq!(page_documents(&reopened)[0].document, source);
            accepted(&send(&mut s2, reopened));
            assert_eq!(s2.draft, saved);
            assert_eq!(supported(&s2.draft), financial);
        }
    }
}

fn write_artifact(variable: &str, bytes: &[u8]) {
    if let Ok(path) = std::env::var(variable) {
        std::fs::write(path, bytes).unwrap();
    }
}

#[test]
fn qbi_partial_payroll_actual_client_artifact_bridge() {
    use crate::tests::cross::interview::{accepted, document, open_topic, page_documents};
    let Ok(case) = std::env::var("TAX_QBI_PAYROLL_CASE") else {
        return;
    };
    assert!(payroll_cases().contains(&case.as_str()));
    let mut s = Interview::new(TEST_USER, base());
    let seed = open_topic(&mut s, TaxSection::SelfEmployment);
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
    documents.documents = vec![document(0, IncomeDocument::Business(business_source(&case)))];
    write_artifact("TAX_QBI_PAYROLL_SEED", &seed.to_avro());
    write_artifact("TAX_QBI_PAYROLL_ANSWERS", &answers.to_avro());
    if let Ok(state) = std::env::var("TAX_QBI_PAYROLL_STATE") {
        let Ok(input) = std::env::var("TAX_QBI_PAYROLL_TRANSPORT_INPUT") else {
            accepted(&crate::tests::cross::interview::save_businesses(
                &mut s,
                vec![business_source(&case)],
            ));
            let accepted_page = open_topic(&mut s, TaxSection::SelfEmployment);
            write_artifact("TAX_QBI_PAYROLL_SEED", &accepted_page.to_avro());
            write_artifact("TAX_QBI_PAYROLL_ANSWERS", &accepted_page.to_avro());
            write_native_draft(state, &s.draft);
            return;
        };
        let draft = read_native_draft(&state);
        let mut s = restart(draft);
        let before = s.draft.clone();
        let dirty = s.dirty;
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let save = matches!(request, P::SaveIncomes { .. });
        let reply = send(&mut s, request);
        if let Ok(key) = std::env::var("TAX_QBI_PAYROLL_EXPECT_KEY") {
            let Reply::Response(P::Problem { message, field }) = &reply else {
                panic!("expected native refusal: {reply:?}")
            };
            assert_eq!(*message, libertas::libertas_formatted_text(&key, &[]));
            assert_eq!(
                field.as_deref(),
                Some(std::env::var("TAX_QBI_PAYROLL_EXPECT_FIELD").unwrap().as_str())
            );
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        } else if save {
            accepted(&reply);
            assert_result(&s.draft, &case);
            let saved = s.draft.clone();
            let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
            write_artifact("TAX_QBI_PAYROLL_REOPEN", &reopened.to_avro());
            accepted(&send(&mut s, reopened));
            assert_eq!(s.draft, saved);
            assert_eq!(restart(saved.clone()).draft, saved);
        } else {
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        }
        let (kind, response) = match reply {
            Reply::Edit(p) => ("edit", p),
            Reply::Response(p) => ("response", p),
        };
        write_artifact("TAX_QBI_PAYROLL_TRANSPORT_OUTPUT", &response.to_avro());
        std::fs::write(std::env::var("TAX_QBI_PAYROLL_TRANSPORT_KIND").unwrap(), kind).unwrap();
        write_native_draft(state, &s.draft);
        return;
    }
    if let Ok(input) = std::env::var("TAX_QBI_PAYROLL_CALC_INPUT") {
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let before = s.draft.clone();
        let Reply::Response(response) = send(&mut s, request) else { panic!() };
        assert!(!matches!(response, P::Problem { .. }), "{response:?}");
        assert_eq!(s.draft, before);
        write_artifact("TAX_QBI_PAYROLL_CALC_OUTPUT", &response.to_avro());
        return;
    }
    let Ok(payload) = std::env::var("TAX_QBI_PAYROLL_PAYLOAD") else {
        return;
    };
    if !std::path::Path::new(&payload).exists() {
        return;
    }
    let request = P::from_avro(&std::fs::read(payload).unwrap()).unwrap();
    accepted(&send(&mut s, request));
    assert_result(&s.draft, &case);
    let expected = s.draft.income.clone();
    let reopened = open_topic(&mut s, TaxSection::SelfEmployment);
    let rows = page_documents(&reopened);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].document, IncomeDocument::Business(business_source(&case)));
    write_artifact("TAX_QBI_PAYROLL_REOPEN", &reopened.to_avro());
    accepted(&send(&mut s, reopened));
    assert_eq!(s.draft.income, expected);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    if let Ok(payload) = std::env::var("TAX_QBI_PAYROLL_RESAVE") {
        let request = P::from_avro(&std::fs::read(payload).unwrap()).unwrap();
        accepted(&send(&mut s, request));
        assert_eq!(s.draft.income, expected);
    }
}
