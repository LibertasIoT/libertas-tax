use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, read_native_draft, write_native_draft_records,
};
extern crate std;
use super::*;
use crate::tests::cross::interview::accepted;
use alloc::string::String;

fn fixture(joint: bool) -> FederalDraft {
    let mut d = complete(if joint { FederalStatus::Joint } else { FederalStatus::Single });
    d.income.clear();
    d.next_id = 1;
    d.checklist.income |= IncomeTopics::INTEREST_DIVIDENDS.0;
    d
}
fn checked(d: &FederalDraft, expected: Option<Owner>) {
    assert_eq!(d.income.len(), usize::from(expected.is_some()));
    if let Some(owner) = expected {
        let entry = &d.income[0];
        assert!(entry.id > 0);
        assert_eq!(entry.owner, owner);
        let FederalIncome::Interest { data } = &entry.income else { panic!() };
        assert_eq!(data.taxable, 10_000);
    }
    let r = supported(d);
    let total = if expected.is_some() { 100 } else { 0 };
    assert_eq!(return_line(&r, "2b"), total * 100);
    assert_eq!(value(&r, "agi"), total);
    assert_eq!(value(&r, "taxable_income"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 0);
    assert_eq!(r.tax, Some(0));
}
#[inline(never)]
fn review(session: &mut Interview) -> (alloc::boxed::Box<P>, alloc::boxed::Box<P>) {
    let entry = alloc::boxed::Box::new(P::ReviewAccepted { user: TEST_USER });
    let Reply::Response(answer @ P::Review { .. }) = send(session, (*entry).clone()) else {
        panic!()
    };
    (entry, alloc::boxed::Box::new(answer))
}

#[test]
fn unit13_document_owner_actual_add_required_singleton_save_restart() {
    use alloc::format;
    use std::{fs, path::PathBuf};
    for profile in std::env::var("TAX_UNIT13_OWNER_PROFILE")
        .map(|v| vec![v])
        .unwrap_or_else(|_| vec!["joint".into(), "single".into()])
    {
        assert!(matches!(profile.as_str(), "joint" | "single"));
        let joint = profile == "joint";
        let phase = std::env::var("TAX_UNIT13_OWNER_PHASE").unwrap_or_else(|_| "fresh".into());
        assert!(matches!(
            phase.as_str(),
            "untouched" | "fresh" | "resave" | "correction" | "final-resave"
        ));
        let directory = std::env::var("TAX_UNIT13_OWNER_DIR").ok().map(PathBuf::from);
        let load_current = std::env::var_os("TAX_UNIT13_OWNER_LOAD_CURRENT").is_some();
        let current_owner = if joint && phase != "correction" && phase != "final-resave" {
            Owner::Spouse
        } else {
            Owner::Taxpayer
        };
        let prior = match phase.as_str() {
            "resave" => "fresh",
            "correction" => "resave",
            "final-resave" => "correction",
            _ => "fresh",
        };
        let draft = if load_current || !matches!(phase.as_str(), "fresh" | "untouched") {
            let path = directory
                .as_ref()
                .unwrap()
                .join(format!("{}.persisted.avro", if load_current { &phase } else { prior }));
            read_native_draft(path)
        } else {
            fixture(joint)
        };
        let expected_before = if load_current {
            Some(current_owner)
        } else if matches!(phase.as_str(), "fresh" | "untouched") {
            None
        } else if joint && phase != "final-resave" {
            Some(Owner::Spouse)
        } else {
            Some(Owner::Taxpayer)
        };
        checked(&draft, expected_before);
        let mut session = restart(draft);
        let before = session.draft.clone();
        let persisted_before = native_draft_records(&before);
        let (entry, shown) = review(&mut session);
        assert_eq!(session.draft, before);
        assert!(!session.dirty);
        if std::env::var_os("TAX_UNIT13_OWNER_SEED_ONLY").is_some() {
            let directory = directory.as_ref().unwrap();
            fs::create_dir_all(directory).unwrap();
            fs::write(directory.join("entry.avro"), entry.to_avro()).unwrap();
            fs::write(directory.join("review.avro"), shown.to_avro()).unwrap();
            fs::write(directory.join("identity.json"), serde_json::to_vec_pretty(&serde_json::json!({"joint":joint,"income_section":TaxSection::Income as i32,"topic_section":TaxSection::InterestDividends as i32})).unwrap()).unwrap();
            return;
        }
        let Some(directory) = directory else { continue };
        let chain: Vec<String> = serde_json::from_slice(
            &fs::read(std::env::var("TAX_UNIT13_OWNER_OPEN_CHAIN").unwrap_or_else(|_| {
                directory.join(format!("{phase}.open-chain.json")).to_string_lossy().into_owned()
            }))
            .unwrap(),
        )
        .unwrap();
        let mut last = None;
        for (index, path) in chain.iter().enumerate() {
            let bytes = fs::read(path).unwrap();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            assert!(
                matches!(request, P::OpenSection { section, .. } if section == if index == 0 { TaxSection::Income } else { TaxSection::InterestDividends })
            );
            last = Some(send(&mut session, request));
            assert_eq!(session.draft, before);
            assert!(!session.dirty);
        }
        if let Ok(output) = std::env::var("TAX_UNIT13_OWNER_OPEN_OUTPUT") {
            let (value, operation) = match last.unwrap() {
                Reply::Response(v) => (v, "Response"),
                Reply::Edit(v) => (v, "EditRequest"),
            };
            fs::write(output, value.to_avro()).unwrap();
            fs::write(std::env::var("TAX_UNIT13_OWNER_OPERATION_OUTPUT").unwrap(), operation)
                .unwrap();
            return;
        }
        assert!(matches!(last, Some(Reply::Edit(P::SaveIncomes { .. }))));
        if let Ok(input) = std::env::var("TAX_UNIT13_OWNER_CALC_INPUT") {
            let bytes = fs::read(input).unwrap();
            let request = P::from_avro(&bytes).unwrap();
            assert_eq!(request.to_avro(), bytes);
            assert!(matches!(
                request,
                P::PreviewDocument { .. }
                    | P::EstimateDocuments { .. }
                    | P::DocumentPropertyOptions { .. }
            ));
            let Reply::Response(value) = send(&mut session, request) else { panic!() };
            assert_eq!(session.draft, before);
            fs::write(std::env::var("TAX_UNIT13_OWNER_CALC_OUTPUT").unwrap(), value.to_avro())
                .unwrap();
            return;
        }
        if phase == "untouched" {
            assert!(joint);
            assert!(!directory.join("untouched.payload.avro").exists());
            assert_eq!(native_draft_records(&session.draft), persisted_before);
            fs::write(
                directory.join("untouched.observed.json"),
                br#"{"save_sent":false,"draft_unchanged":true,"dirty_unchanged":true}"#,
            )
            .unwrap();
            return;
        }
        let bytes = fs::read(directory.join(format!("{phase}.payload.avro"))).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        let P::SaveIncomes { documents, .. } = &request else { panic!() };
        assert_eq!(documents.documents.len(), 1);
        let item = &documents.documents[0];
        assert_eq!(item.owner, current_owner);
        let IncomeDocument::Int1099(data) = &item.document else { panic!() };
        assert_eq!(data.taxable, 10_000);
        let before_id = session.draft.income.first().map(|e| e.id);
        accepted(&send(&mut session, request));
        checked(&session.draft, Some(current_owner));
        if let Some(id) = before_id {
            assert_eq!(session.draft.income[0].id, id)
        }
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        assert_eq!(native_draft_records(&draft), stored);
        let mut reopened = restart(draft);
        checked(&reopened.draft, Some(current_owner));
        let (entry, shown) = review(&mut reopened);
        write_native_draft_records(directory.join(format!("{phase}.persisted.avro")), &stored);
        fs::write(directory.join(format!("{phase}.entry.avro")), entry.to_avro()).unwrap();
        fs::write(directory.join(format!("{phase}.review.avro")), shown.to_avro()).unwrap();
        fs::write(directory.join(format!("{phase}.result.json")), serde_json::to_vec_pretty(&serde_json::json!({"phase":phase,"owner":current_owner as i32,"source":reopened.draft.income[0].id,"agi":100,"tax":0,"identity_preserved":true})).unwrap()).unwrap();
    }
}
