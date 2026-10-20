use crate::federal_rules::estimate;
use crate::interview::{Interview, Reply};
use crate::tests::unit47_rule_cases;
use crate::{FederalDraft, FederalResult, TaxAppData, TaxDataKey, TaxInterviewProtocol as P};
use alloc::vec::Vec;
use libertas::data::{KeyCodec, KeyDecode};

pub(in crate::tests) fn send(s: &mut Interview, p: P) -> Reply {
    assert_eq!(P::from_avro(&p.to_avro()).unwrap(), p);
    let observed = unit47_rule_cases::begin_observed_save(s, &p);
    let capture = crate::tests::cross::oracle::begin_send(s, &p);
    let r = s.handle(p).unwrap();
    let data = match &r {
        Reply::Response(v) | Reply::Edit(v) => v,
    };
    assert_eq!(P::from_avro(&data.to_avro()).unwrap(), *data);
    crate::tests::cross::oracle::finish_send(capture, s, &r);
    if let Some(observed) = observed {
        unit47_rule_cases::finish_observed_save(observed, s, &r);
    }
    r
}
// A row status (FormattedText) in English, as a client renders it from
// APP_STRINGS: positional arguments, resource-text arguments and one/other
// plurals, the forms the App's statuses use.
pub(in crate::tests) fn status_text(value: &[u8]) -> alloc::string::String {
    status_message(value).english()
}
pub(in crate::tests) fn status_message(value: &[u8]) -> crate::Message {
    crate::Message::decode(value)
}
pub(in crate::tests) fn restart(d: FederalDraft) -> Interview {
    Interview::new(TEST_USER, roundtrip_native_draft(d))
}
pub(in crate::tests) fn error(r: Reply) {
    assert!(matches!(r, Reply::Response(P::Problem { .. })), "{r:?}");
}
// The result the review showed, with its provenance: the same calculation
// over the accepted return (a reply carries no provenance).
pub(in crate::tests) fn reviewed(s: &Interview) -> FederalResult {
    let mut r = estimate(&s.draft);
    r.return_lines = crate::draft_return(&r);
    r
}

/// The verified user every test request carries.
pub(crate) const TEST_USER: libertas::LibertasUser = 7;

pub(crate) fn exchange(session: &mut Interview, request: P, case: &str) -> Reply {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, request);
    if matches!(&reply, Reply::Response(P::Problem { .. })) {
        assert_eq!(session.draft, before, "{case}: refusal is atomic");
        assert_eq!(session.dirty, dirty, "{case}: refusal keeps dirty state");
    }
    reply
}

pub(in crate::tests) fn roundtrip_native_request(request: P) -> P {
    let bytes = request.to_avro();
    let decoded = P::from_avro(&bytes).unwrap();
    assert_eq!(decoded, request);
    assert_eq!(decoded.to_avro(), bytes);
    decoded
}

pub(in crate::tests) fn roundtrip_native_draft(draft: FederalDraft) -> FederalDraft {
    let records = native_draft_records(&draft);
    let decoded = draft_from_native_records(draft.tax_year, &records);
    assert_eq!(decoded, draft);
    assert_eq!(native_draft_records(&decoded), records);
    decoded
}

// Each pair is one real Data key and value, without a whole-draft envelope.
pub(in crate::tests) type NativeDraftRecords = Vec<(Vec<u8>, Vec<u8>)>;

pub(in crate::tests) fn native_draft_records(draft: &FederalDraft) -> NativeDraftRecords {
    crate::return_records::project(TEST_USER, draft, 1)
        .unwrap()
        .into_iter()
        .map(|(key, value)| {
            let key_bytes = key.to_key_bytes().unwrap();
            let value_bytes = value.to_avro();
            assert_eq!(TaxDataKey::decode_key(&key_bytes).unwrap(), key);
            assert_eq!(TaxAppData::from_avro(&value_bytes).unwrap(), value);
            (key_bytes, value_bytes)
        })
        .collect()
}

pub(in crate::tests) fn draft_from_native_records(
    tax_year: i32,
    records: &NativeDraftRecords,
) -> FederalDraft {
    let decoded = records
        .iter()
        .map(|(key_bytes, value_bytes)| {
            let key = TaxDataKey::decode_key(key_bytes).unwrap();
            let value = TaxAppData::from_avro(value_bytes).unwrap();
            assert_eq!(key.to_key_bytes().unwrap(), *key_bytes);
            assert_eq!(value.to_avro(), *value_bytes);
            (key, value)
        })
        .collect();
    let (draft, _) = crate::return_records::assemble(TEST_USER, tax_year, decoded).unwrap();
    draft
}
