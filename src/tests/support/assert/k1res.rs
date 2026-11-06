use crate::tests::cross::interview::rejected_with;
use crate::tests::*;
pub(crate) fn assert_atomic_rejection(session: &mut Interview, request: P, key: &str) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(session, request)
    } else {
        send(session, request)
    };
    rejected_with(reply, key);
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

pub(crate) fn assert_located_rejection(session: &mut Interview, request: P, key: &str, path: &str) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let control = crate::tests::support::client::income_pages::topic_fixture_control(
        session, &request, key, path,
    );
    let (reply, expected) = if let Some((mut candidate, actual, mapped)) = control {
        let before = candidate.draft.clone();
        let dirty = candidate.dirty;
        let reply = send(&mut candidate, actual);
        assert_eq!(candidate.draft, before);
        assert_eq!(candidate.dirty, dirty);
        (reply, mapped)
    } else {
        (send(session, request), path.into())
    };
    let Reply::Response(P::Problem { message, field }) = reply else {
        panic!("expected located native refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(field.as_deref(), Some(expected.as_str()));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}
