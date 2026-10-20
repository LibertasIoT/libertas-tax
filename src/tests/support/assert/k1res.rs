use crate::tests::cross::interview::rejected_with;
use crate::tests::*;
pub(crate) fn assert_atomic_rejection(session: &mut Interview, request: P, key: &str) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    rejected_with(send(session, request), key);
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

pub(crate) fn assert_located_rejection(session: &mut Interview, request: P, key: &str, path: &str) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let Reply::Response(P::Problem { message, field }) = send(session, request) else {
        panic!("expected located native refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(field.as_deref(), Some(path));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}
