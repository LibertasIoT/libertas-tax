// Optional explicit condition controls and observed native Save evidence.
// Explicit callers first assert key, path and atomic Save state; observations
// retain their own narrower qualification for later source/schema review.
extern crate std;
use super::*;

pub(crate) fn export_unit47_rule_case(case: &str, request: &P, problem: &P) {
    let Some(directory) = std::env::var_os("TAX_UNIT47_RULE_CASE_DIR") else {
        return;
    };
    assert!(!case.is_empty());
    assert!(case.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
    let P::Problem { field: Some(field), .. } = problem else {
        panic!("a former rule must identify its field")
    };
    let request = request.to_avro();
    let problem = problem.to_avro();
    let _ = write_avro_case(
        std::path::Path::new(&directory),
        case,
        &request,
        "problem.avro",
        &problem,
        serde_json::json!({
            "case": case,
            "field": field,
            "request_bytes": request.len(),
            "problem_bytes": problem.len(),
        }),
    );
}

fn write_avro_case(
    root: &std::path::Path,
    case: &str,
    request: &[u8],
    response_file: &str,
    response: &[u8],
    metadata: serde_json::Value,
) -> std::path::PathBuf {
    std::fs::create_dir_all(root).unwrap();
    let directory = root.join(case);
    // Both explicit proofs and observations refuse to overwrite earlier bytes.
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("request.avro"), request).unwrap();
    std::fs::write(directory.join(response_file), response).unwrap();
    std::fs::write(directory.join("case.json"), serde_json::to_vec_pretty(&metadata).unwrap())
        .unwrap();
    directory
}

// Observations supplement authored condition controls. A matching resource key
// alone never establishes that a particular former declaration was exercised.
pub(crate) struct ObservedSave {
    directory: std::path::PathBuf,
    request_kind: &'static str,
    request: Vec<u8>,
    draft_before: Vec<u8>,
    dirty_before: bool,
}

fn observed_save_kind(request: &P) -> Option<&'static str> {
    Some(match request {
        P::SaveSetup { .. } => "SaveSetup",
        P::SavePeople { .. } => "SavePeople",
        P::SaveSpouse { .. } => "SaveSpouse",
        P::SaveDeductions { .. } => "SaveDeductions",
        P::SaveOftenMissed { .. } => "SaveOftenMissed",
        P::SaveIncomeWithoutForm { .. } => "SaveIncomeWithoutForm",
        P::SaveChecklist { .. } => "SaveChecklist",
        P::SaveChildren { .. } => "SaveChildren",
        P::SaveIncomes { .. } => "SaveIncomes",
        P::SaveHsa { .. } => "SaveHsa",
        P::SaveCare { .. } => "SaveCare",
        P::SaveSelfEmployedBenefits { .. } => "SaveSelfEmployedBenefits",
        P::SaveIraContributions { .. } => "SaveIraContributions",
        P::SaveEducator { .. } => "SaveEducator",
        P::SaveStudentLoan { .. } => "SaveStudentLoan",
        P::SaveEmployeeExpenses { .. } => "SaveEmployeeExpenses",
        P::SaveMedical { .. } => "SaveMedical",
        P::SaveTaxes { .. } => "SaveTaxes",
        P::SaveOtherItemized { .. } => "SaveOtherItemized",
        P::SaveMortgages { .. } => "SaveMortgages",
        P::SaveGiftCarryovers { .. } => "SaveGiftCarryovers",
        P::SaveEducation { .. } => "SaveEducation",
        P::SaveSaver { .. } => "SaveSaver",
        P::SaveMarketplace { .. } => "SaveMarketplace",
        P::SaveMoreCredits { .. } => "SaveMoreCredits",
        P::SaveOtherTaxes { .. } => "SaveOtherTaxes",
        P::SaveMoreAdjustments { .. } => "SaveMoreAdjustments",
        P::SavePayments { .. } => "SavePayments",
        P::SaveIncomeByPeriod { .. } => "SaveIncomeByPeriod",
        P::SaveInjuredSpouse { .. } => "SaveInjuredSpouse",
        P::SaveParentTax { .. } => "SaveParentTax",
        // Finish is also a native mutation; its actual refusals remain useful
        // observations without pretending it is a Save-prefixed Request.
        P::Finish { .. } => "Finish",
        _ => return None,
    })
}

// Keep serialization and capture bookkeeping outside the existing send frame.
// Inputs are borrowed; only encoded byte vectors live across the native call.
#[inline(never)]
pub(crate) fn begin_observed_save(session: &Interview, request: &P) -> Option<ObservedSave> {
    let directory = std::env::var_os("TAX_UNIT47_OBSERVED_CASE_DIR")?;
    let request_kind = observed_save_kind(request)?;
    Some(ObservedSave {
        directory: directory.into(),
        request_kind,
        request: request.to_avro(),
        draft_before: session.draft.to_avro(),
        dirty_before: session.dirty,
    })
}

fn observed_slug(text: &str, limit: usize) -> alloc::string::String {
    text.bytes()
        .take(limit)
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect()
}

#[inline(never)]
pub(crate) fn finish_observed_save(observed: ObservedSave, session: &Interview, reply: &Reply) {
    use libertas::AvroDecode;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(0);

    let (reply_kind, response) = match reply {
        Reply::Response(value) => ("Response", value),
        Reply::Edit(value) => ("Edit", value),
    };
    let (key, field) = match response {
        P::Problem { message, field } => {
            let mut offset = 0;
            let decoded = crate::common::FormattedMessage::avro_decode(message, &mut offset)
                .expect("an observed Problem contains a FormattedText tuple");
            assert_eq!(offset, message.len());
            (Some(decoded.0), field.as_ref())
        }
        _ => (None, None),
    };
    let rejected = key.is_some();
    let after = rejected.then(|| session.draft.to_avro());
    let draft_unchanged = after.as_ref().map(|after| *after == observed.draft_before);
    let dirty_unchanged = rejected.then_some(session.dirty == observed.dirty_before);
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let sequence = NEXT_OBSERVATION.fetch_add(1, Ordering::Relaxed);
    let case = alloc::format!(
        "observed-{}-{sequence:08}-{}-{}-{}",
        std::process::id(),
        observed_slug(thread_name, 64),
        observed.request_kind,
        observed_slug(key.as_deref().unwrap_or("accepted"), 64),
    );
    let encoded_response = response.to_avro();
    let directory = write_avro_case(
        &observed.directory,
        &case,
        &observed.request,
        if rejected { "problem.avro" } else { "response.avro" },
        &encoded_response,
        serde_json::json!({
            "case": case,
            "process_id": std::process::id(),
            "sequence": sequence,
            "thread_name": thread_name,
            "request_kind": observed.request_kind,
            "reply_kind": reply_kind,
            "outcome": if rejected { "native_problem" } else { "native_non_problem_response" },
            "key": key,
            "field": field,
            "locator_status": if !rejected { "not_applicable" } else if field.is_some() { "located" } else { "unlocated" },
            "request_bytes": observed.request.len(),
            "response_bytes": encoded_response.len(),
            "request_roundtrip_verified_by_send": true,
            "response_roundtrip_verified_by_send": true,
            "draft_encoding": "FederalDraft Avro",
            "atomic": {
                "applicable": rejected,
                "verified": rejected.then_some(draft_unchanged == Some(true) && dirty_unchanged == Some(true)),
                "draft_bytes_unchanged": draft_unchanged,
                "dirty_unchanged": dirty_unchanged,
                "dirty_before": observed.dirty_before,
                "dirty_after": session.dirty,
                "before_draft_bytes": observed.draft_before.len(),
                "after_draft_bytes": after.as_ref().map(Vec::len),
            },
            "qualification": "Actual native send observation; condition identity, former-rule coverage, locator resolution and positive financial assertions require separate evidence. A fieldless Problem remains explicitly unlocated.",
        }),
    );
    // Accepted request seeds keep their actual pre-call context as well. No
    // positive financial assertion or rejected-Save atomicity is inferred.
    std::fs::write(directory.join("draft-before.avro"), &observed.draft_before).unwrap();
    if let Some(after) = after {
        std::fs::write(directory.join("draft-after.avro"), &after).unwrap();
        // Preserve both actual states and metadata before surfacing any real
        // rejected-Save mutation; observation must never silently waive it.
        assert_eq!(draft_unchanged, Some(true), "{case}: rejected Save changed accepted Draft");
        assert_eq!(dirty_unchanged, Some(true), "{case}: rejected Save changed dirty state");
    }
}
