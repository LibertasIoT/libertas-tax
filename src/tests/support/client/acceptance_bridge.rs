extern crate std;

use super::lifecycle::{TEST_USER, send};
use super::native_record_files::{read_native_draft, write_native_draft};
use crate::interview::{Interview, Reply};
use crate::{FederalDraft, TaxInterviewProtocol as P};
use std::{fs, path::Path};

// TAX_ACCEPTANCE_BRIDGE_MODE selects initialize or exchange. AFTER is the
// output record directory. Exchange reads DRAFT and REQUEST; initialize starts
// an empty return with COOKIE (default acceptance-native), optionally sending
// REQUEST too. A request writes RESPONSE Avro and REPLY_KIND (edit or response).
#[test]
fn acceptance_native_bridge() {
    let Ok(mode) = std::env::var("TAX_ACCEPTANCE_BRIDGE_MODE") else {
        preserves_normal_requests_and_unknown_owner();
        return;
    };
    let variable = |name| std::env::var(alloc::format!("TAX_ACCEPTANCE_BRIDGE_{name}"));
    let draft = match mode.as_str() {
        "initialize" => {
            FederalDraft::empty(variable("COOKIE").unwrap_or_else(|_| "acceptance-native".into()))
        }
        "exchange" => read_native_draft(variable("DRAFT").expect("DRAFT is required")),
        _ => panic!("MODE must be initialize or exchange"),
    };
    let request = variable("REQUEST").ok();
    assert!(mode == "initialize" || request.is_some(), "exchange requires REQUEST");
    let response = request.as_ref().map(|_| variable("RESPONSE").expect("RESPONSE is required"));
    let kind = request.as_ref().map(|_| variable("REPLY_KIND").expect("REPLY_KIND is required"));
    run(
        draft,
        request.as_deref().map(Path::new),
        Path::new(&variable("AFTER").expect("AFTER is required")),
        response.as_deref().map(Path::new),
        kind.as_deref().map(Path::new),
    );
}

fn run(
    draft: FederalDraft,
    request: Option<&Path>,
    after: &Path,
    response: Option<&Path>,
    kind: Option<&Path>,
) {
    let Some(request) = request else {
        write_native_draft(after, &draft);
        return;
    };
    let mut session = Interview::new(TEST_USER, draft);
    let request = P::from_avro(&fs::read(request).unwrap()).unwrap();
    let reply = send(&mut session, request);
    let (value, reply_kind) = match reply {
        Reply::Response(value) => (value, "response"),
        Reply::Edit(value) => (value, "edit"),
    };
    fs::write(response.expect("response output"), value.to_avro()).unwrap();
    fs::write(kind.expect("reply kind output"), reply_kind).unwrap();
    write_native_draft(after, &session.draft);
}

fn preserves_normal_requests_and_unknown_owner() {
    use crate::tests::cross::interview::answer;
    use crate::tests::support::builders::person;
    use crate::{DocumentOwner, Filing, WageDocument};

    let directory = std::env::temp_dir()
        .join(alloc::format!("tax-acceptance-native-bridge-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let records = directory.join("records");
    let request_file = directory.join("request.avro");
    let response_file = directory.join("response.avro");
    let kind_file = directory.join("kind");
    let initial = FederalDraft::empty("acceptance-native".into());
    run(initial.clone(), None, &records, None, None);
    assert_eq!(read_native_draft(&records), initial);

    let mut request = P::OpenInterview { user: TEST_USER };
    loop {
        let saved_document = matches!(request, P::SaveDocuments { .. });
        let bytes = request.to_avro();
        fs::write(&request_file, &bytes).unwrap();
        run(
            read_native_draft(&records),
            Some(&request_file),
            &records,
            Some(&response_file),
            Some(&kind_file),
        );
        assert_eq!(fs::read(&request_file).unwrap(), bytes);
        let value = P::from_avro(&fs::read(&response_file).unwrap()).unwrap();
        assert!(!matches!(value, P::Problem { .. }), "{value:?}");
        let reply = match fs::read_to_string(&kind_file).unwrap().as_str() {
            "edit" => Reply::Edit(value),
            "response" => Reply::Response(value),
            other => panic!("unknown reply kind {other}"),
        };
        if saved_document {
            break;
        }
        request = answer(reply, Filing::Single);
        if let P::SaveDocuments { documents, .. } = &mut request {
            let row = &mut documents.common.wages.documents[0];
            row.owner = DocumentOwner::Unknown;
            let WageDocument::W2(form) = &mut row.document else { unreachable!() };
            let person = person();
            form.employee.name = alloc::format!("{} {}", person.first_name, person.last_name);
        }
    }
    let accepted = read_native_draft(&records);
    assert_eq!(accepted.income.len(), 1);
    assert_eq!(accepted.income[0].owner, DocumentOwner::Unknown);
    fs::remove_dir_all(directory).unwrap();
}
