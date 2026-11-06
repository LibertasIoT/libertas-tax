use crate::TaxInterviewProtocol as P;
use crate::interview::Reply;

// Compose both required answers through the workflow, as a client does after
// the person chooses situations or explicitly chooses None of these.
pub(in crate::tests) fn answer_often_missed(reply: Reply, answers: crate::OftenMissed) -> P {
    match reply {
        Reply::Response(P::BeginOftenMissed {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings,
            progress,
            all_topics,
            navigation,
            suggested_destination,
            ..
        }) => P::SaveOftenMissed {
            user: super::TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings,
            progress,
            value: answers.into(),
            all_topics,
            navigation,
            suggested_destination,
        },
        Reply::Edit(mut page @ P::SaveOftenMissed { .. }) => {
            let P::SaveOftenMissed { value, .. } = &mut page else { unreachable!() };
            *value = answers.into();
            page
        }
        other => panic!("expected the often-missed editor: {other:?}"),
    }
}

// Add or remove a printed statement through its owning source editor. Printed
// names go through the production matcher; this never assigns its recipient.
pub(in crate::tests) fn save_often_missed_marketplace_source(
    session: &mut crate::interview::Interview,
    present: bool,
) -> Reply {
    let mut page = super::income_pages::open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.common.marketplace.documents.clear();
    if present {
        let mut form = crate::tests::cross::interview::all_year(
            "Synthetic policy issuer",
            50_000,
            60_000,
            20_000,
        );
        form.value.recipient =
            (&crate::tests::support::builders::fixture_document_names(&session.draft, -1)
                .unwrap()
                .recipient)
                .into();
        documents
            .common
            .marketplace
            .documents
            .push(crate::Form1095ADocument { id: form.id, value: form.value });
    }
    super::send(session, page)
}
