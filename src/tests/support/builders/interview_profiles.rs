use crate::interview::Reply;
use crate::tests::cross::interview::{answer, row};
use crate::{Filing, TaxInterviewProtocol as P, WageDocument};
use alloc::vec;

// The two ordinary wage households share the normal client answer builder.
// Printed recipients, rather than an owner override, associate their documents.
pub(in crate::tests) fn wage_household_answer(reply: Reply, filing: Filing) -> P {
    let mut request = answer(reply, filing);
    if filing == Filing::MarriedJointly {
        match &mut request {
            P::SaveChildren { dependents, .. } => {
                dependents.people =
                    vec![row("Alex", "Sample", 20180615), row("Riley", "Sample", 20160615)];
            }
            P::SaveDocuments { documents, .. } => {
                let mut spouse = documents.common.wages.documents[0].clone();
                let WageDocument::W2(form) = &mut spouse.document else {
                    unreachable!("ordinary wages use a W-2")
                };
                form.employee.name = "Jordan Sample".into();
                documents.common.wages.documents.push(spouse);
            }
            _ => {}
        }
    }
    request
}
