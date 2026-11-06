use crate::tests::cross::interview::{PAGE_ITEMIZED, back, itemized_hub, send_past_less_common};
use crate::tests::*;

pub(in crate::tests) fn submit_to_itemized_summary(session: &mut Interview, request: P) -> P {
    let reply = send_past_less_common(session, request);
    match reply {
        Reply::Edit(
            P::SaveTaxes { previous, .. }
            | P::SaveMedical { previous, .. }
            | P::SaveGiftCarryovers { previous, .. }
            | P::SaveOtherItemized { previous, .. },
        ) if previous == PAGE_ITEMIZED => {
            let request = back(&session.draft, previous);
            itemized_hub(send(session, request))
        }
        other => itemized_hub(other),
    }
}
