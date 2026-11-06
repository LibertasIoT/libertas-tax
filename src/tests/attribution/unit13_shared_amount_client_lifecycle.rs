use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, read_native_draft, write_native_draft_records,
};
// Independent Social automatic-answer subset of the existing shared Workflow
// callback. The retained-NIIT families remain on the parked combined source.
extern crate std;
use super::*;
use crate::tests::cross::interview::accepted;

fn social(injured: Owner) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, claim);
    for (owner, amount) in [(Owner::Taxpayer, 2_000_000), (Owner::Spouse, 1_000_000)] {
        let mut entry = income(social_security_net(amount, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    d
}
fn checked(d: &FederalDraft) -> FederalResult {
    let injured = d.injured_spouse.as_ref().unwrap().injured;
    let i = usize::from(injured == Owner::Spouse);
    let r = supported(d);
    assert_eq!(value(&r, "taxable_social_security"), 25_500);
    assert_eq!(value(&r, "agi"), 105_500);
    assert_eq!(value(&r, "taxable_income"), 73_300);
    assert_eq!(r.tax, Some(830_300));
    assert_eq!(r.refund, Some(169_700));
    assert_eq!(
        [column(&r, "13b", "b"), column(&r, "13b", "c")],
        if i == 0 { [17_000, 8_500] } else { [8_500, 17_000] }
    );
    r
}

#[inline(never)]
fn review_context(session: &mut Interview) -> (alloc::boxed::Box<P>, alloc::boxed::Box<P>) {
    let entry = alloc::boxed::Box::new(P::ReviewAccepted { user: TEST_USER });
    let Reply::Response(review @ P::Review { .. }) = send(session, (*entry).clone()) else {
        panic!("real service review required")
    };
    (entry, alloc::boxed::Box::new(review))
}
#[inline(never)]
fn actual_open(session: &mut Interview, bytes: &[u8]) -> alloc::boxed::Box<P> {
    let request = P::from_avro(bytes).unwrap();
    assert_eq!(request.to_avro(), bytes);
    assert!(matches!(request, P::OpenSection { section: TaxSection::InjuredSpouse, .. }));
    let Reply::Response(response @ P::BeginInjuredSpouse { .. }) = send(session, request) else {
        panic!("bound actual review link must open workflow")
    };
    alloc::boxed::Box::new(response)
}
fn context_answers(response: &P, key: &str, expected: Option<i64>) {
    let P::BeginInjuredSpouse { value, .. } = response else { panic!() };
    assert_eq!(
        value.allocations.iter().filter(|row| row.component == key).count()
            + value.new_allocations.iter().filter(|row| row.component == key).count(),
        1
    );
    assert_eq!(
        value.allocations.iter().find(|row| row.component == key).map(|row| row.injured),
        expected
    );
    assert_eq!(value.new_allocations.iter().any(|row| row.component == key), expected.is_none());
}

// This bridge never fabricates a Save request from construction context. The
// Studio workflow runtime creates the complete payload after real input.
#[test]
fn unit13_shared_amount_actual_workflow_save_restart_reopen_correction() {
    use alloc::format;
    use std::{fs, path::PathBuf};
    let profiles = std::env::var("TAX_UNIT13_SHARED_PROFILE")
        .map(|p| vec![p])
        .unwrap_or_else(|_| vec!["social-taxpayer".into(), "social-spouse".into()]);
    for profile in profiles {
        assert!(matches!(profile.as_str(), "social-taxpayer" | "social-spouse"));
        let injured = if profile == "social-taxpayer" { Owner::Taxpayer } else { Owner::Spouse };
        let phase = std::env::var("TAX_UNIT13_SHARED_PHASE").unwrap_or_else(|_| "fresh".into());
        assert!(matches!(phase.as_str(), "fresh" | "resave" | "correction" | "final-resave"));
        let directory = std::env::var("TAX_UNIT13_SHARED_DIR").ok().map(PathBuf::from);
        let draft = if std::env::var_os("TAX_UNIT13_SHARED_LOAD_CURRENT").is_some() {
            read_native_draft(directory.as_ref().unwrap().join(format!("{phase}.persisted.avro")))
        } else if phase == "fresh" {
            social(injured)
        } else {
            let previous = match phase.as_str() {
                "resave" => "fresh",
                "correction" => "resave",
                "final-resave" => "correction",
                _ => unreachable!(),
            };
            read_native_draft(
                directory.as_ref().unwrap().join(format!("{previous}.persisted.avro")),
            )
        };
        let key = "taxable_social_security";
        checked(&draft);
        let stored_before = draft
            .injured_spouse
            .as_ref()
            .unwrap()
            .allocations
            .iter()
            .find(|row| row.component == key)
            .map(|row| row.injured);
        let original_history = draft.screening.as_ref().unwrap().capital_history.clone();
        let mut session = restart(draft);
        let before = session.draft.clone();
        let persisted_before = native_draft_records(&before);
        let (entry, review) = review_context(&mut session);
        assert_eq!(session.draft, before);
        assert!(!session.dirty);
        let automatic = if injured == Owner::Taxpayer { 1_700_000 } else { 850_000 };
        let accepted_initial = Some(automatic);
        if std::env::var_os("TAX_UNIT13_SHARED_SEED_ONLY").is_some() {
            let directory = directory.as_ref().unwrap();
            fs::create_dir_all(directory).unwrap();
            fs::write(directory.join("entry.avro"), entry.to_avro()).unwrap();
            fs::write(directory.join("review.avro"), review.to_avro()).unwrap();
            fs::write(directory.join("identity.json"),serde_json::to_vec_pretty(&serde_json::json!({"profile":profile,"injured":usize::from(injured==Owner::Spouse),"social":true,"stored_before":stored_before,"accepted_initial":accepted_initial,"key":key,"section":TaxSection::InjuredSpouse as i32})).unwrap()).unwrap();
            return;
        }
        let Some(directory) = &directory else {
            let P::Review { remaining_items, correction_notes, answers, .. } = &*review else {
                panic!()
            };
            assert!(
                remaining_items
                    .iter()
                    .chain(correction_notes.iter())
                    .chain(answers.iter())
                    .any(|row| row.section == TaxSection::InjuredSpouse)
            );
            continue;
        };
        let open_input = std::env::var("TAX_UNIT13_SHARED_OPEN_INPUT")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| directory.join(format!("{phase}.open.request.avro")));
        let response = actual_open(&mut session, &fs::read(open_input).unwrap());
        context_answers(&response, key, accepted_initial);
        assert_eq!(session.draft, before);
        if let Ok(output) = std::env::var("TAX_UNIT13_SHARED_OPEN_OUTPUT") {
            fs::write(output, response.to_avro()).unwrap();
            return;
        }
        if let Ok(input) = std::env::var("TAX_UNIT13_SHARED_CALC_INPUT") {
            let bytes = fs::read(input).unwrap();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            assert!(matches!(request, P::EstimateInjuredSpouse { .. }));
            let Reply::Response(answer) = send(&mut session, request) else { panic!() };
            assert_eq!(session.draft, before);
            fs::write(std::env::var("TAX_UNIT13_SHARED_CALC_OUTPUT").unwrap(), answer.to_avro())
                .unwrap();
            return;
        }

        if let Ok(input) = std::env::var("TAX_UNIT13_SHARED_DUPLICATE_INPUT") {
            let bytes = fs::read(input).unwrap();
            let mut request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
            let duplicate = value.allocations.first().expect("actual answered Social row").clone();
            let component = duplicate.component.clone();
            value.new_allocations.push(duplicate);
            let bytes = request.to_avro();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            let reply = send(&mut session, request);
            let Reply::Response(P::BeginInjuredSpouse { value, revision, .. }) = reply else {
                panic!("malformed duplicate must refresh without accepting: {reply:?}")
            };
            let mut refreshed = alloc::collections::BTreeSet::new();
            assert!(
                value
                    .allocations
                    .iter()
                    .map(|row| &row.component)
                    .chain(value.new_allocations.iter().map(|row| &row.component))
                    .all(|key| refreshed.insert(key.clone()))
            );
            assert_eq!(refreshed.len(), 1);
            assert_eq!(revision, before.revision);
            assert_eq!(session.draft, before);
            assert!(!session.dirty);
            assert_eq!(native_draft_records(&session.draft), persisted_before);
            fs::write(directory.join("duplicate.observed.json"),serde_json::to_vec_pretty(&serde_json::json!({"rejected":true,"reply":"BeginInjuredSpouse catalog refresh","draft_unchanged":true,"revision_unchanged":true,"dirty_unchanged":true,"duplicate_persisted":false,"returned_unique_components":refreshed,"component":component,"duplicate_array":"new_allocations"})).unwrap()).unwrap();
            return;
        }
        let bytes = fs::read(directory.join(format!("{phase}.payload.avro"))).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        let expected = automatic;
        let P::SaveInjuredSpouse { value, .. } = &request else { panic!() };
        assert_eq!(
            value
                .allocations
                .iter()
                .chain(value.new_allocations.iter())
                .filter(|row| row.component == key)
                .count(),
            1
        );
        assert_eq!(
            value
                .allocations
                .iter()
                .chain(value.new_allocations.iter())
                .find(|row| row.component == key)
                .unwrap()
                .injured,
            expected
        );

        let mut components = alloc::collections::BTreeSet::new();
        assert!(
            value
                .allocations
                .iter()
                .chain(value.new_allocations.iter())
                .all(|row| components.insert(row.component.clone()))
        );
        accepted(&send(&mut session, request));
        let result = checked(&session.draft);
        assert_eq!(session.draft.injured_spouse.as_ref().unwrap().allocations.len(), 1);
        assert_eq!(
            session
                .draft
                .injured_spouse
                .as_ref()
                .unwrap()
                .allocations
                .iter()
                .filter(|row| row.component == key)
                .count(),
            1
        );

        assert_eq!(session.draft.screening.as_ref().unwrap().capital_history, original_history);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        assert_eq!(native_draft_records(&draft), stored);
        let mut reopened = restart(draft);
        let (reopened_entry, shown) = review_context(&mut reopened);
        // Return the actual reopened Response to Studio. Its normal workflow
        // materializes the next complete resave payload; native code never
        // manufactures a missing answer or manually fills request context.
        let result_before = checked(&reopened.draft);
        assert_eq!(result.return_lines, result_before.return_lines);
        let tax_year = reopened.draft.tax_year;
        let stored = native_draft_records(&reopened.draft);
        let decoded = draft_from_native_records(tax_year, &stored);
        assert_eq!(decoded, reopened.draft);
        assert_eq!(native_draft_records(&decoded), stored);
        write_native_draft_records(directory.join(format!("{phase}.persisted.avro")), &stored);
        fs::write(directory.join(format!("{phase}.entry.avro")), reopened_entry.to_avro()).unwrap();
        fs::write(directory.join(format!("{phase}.reopened.avro")), shown.to_avro()).unwrap();
        fs::write(directory.join(format!("{phase}.result.json")),serde_json::to_vec_pretty(&serde_json::json!({"phase":phase,"social":true,"retained_injured":expected,"tax":result.tax,"refund":result.refund,"draft_history_unchanged":true})).unwrap()).unwrap();
    }
}
