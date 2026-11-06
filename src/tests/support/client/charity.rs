use crate::tests::cross::interview::{open_itemized_topic, past_less_common};
use crate::tests::support::client::income_pages::open_document_center;
use crate::tests::*;

pub(in crate::tests) fn gift_document_fixture_request(
    session: &mut Interview,
    value: &GiftCarryovers,
) -> P {
    // The fixture's donor is you, as printed on the form.
    let you = fixture_document_names(&session.draft, -1).map(|names| names.recipient);
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.vehicle_gifts.documents = value
        .vehicles
        .iter()
        .map(|page| crate::Form1098CDocument {
            id: 0,
            form: {
                let mut form = page.document.form.clone();
                if form.donor.name.is_empty()
                    && let Some(you) = &you
                {
                    form.donor = you.clone();
                }
                form
            },
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        })
        .collect();
    request
}

fn refused(reply: &Reply) -> bool {
    matches!(reply, Reply::Response(P::Problem { .. }))
}

// Printed forms and personal answers use their own editors. Preflight the
// complete fixture through both real Saves before changing accepted answers.
pub(in crate::tests) fn save_gift_source_fixtures(
    session: &mut Interview,
    value: &mut GiftCarryovers,
) -> Reply {
    fn attempt(session: &mut Interview, value: &mut GiftCarryovers) -> Reply {
        if !value.vehicles.is_empty() {
            let request = gift_document_fixture_request(session, value);
            let reply = send(session, request);
            if refused(&reply) {
                return reply;
            }
        }
        let mut request = open_itemized_topic(session, TaxSection::GiftCarryovers);
        let P::SaveGiftCarryovers { value: opened, .. } = &mut request else { unreachable!() };
        assert_eq!(opened.vehicles.len(), value.vehicles.len());
        for (supplied, accepted) in value.vehicles.iter_mut().zip(&opened.vehicles) {
            supplied.document = accepted.document.clone();
            if let Some(gift) = &mut supplied.gift {
                gift.id = accepted.document.id;
            }
        }
        value.appraisal_choices = opened.appraisal_choices.clone();
        *opened = value.clone();
        let reply = send(session, request);
        if refused(&reply) { reply } else { past_less_common(session, reply) }
    }
    let mut candidate = restart(session.draft.clone());
    let mut supplied = value.clone();
    let reply = attempt(&mut candidate, &mut supplied);
    if refused(&reply) {
        return reply;
    }
    let mut supplied = value.clone();
    let reply = attempt(session, &mut supplied);
    if !refused(&reply) {
        *value = supplied;
    }
    reply
}
