use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn iso_exercise_source(data: IsoExercise) -> IsoExerciseSource {
    let IsoExercise {
        id,
        printed,
        owner,
        label,
        exercised,
        recognition,
        exercise_price,
        market_price,
        option_cost,
        shares,
        granted,
        corporation,
        sold,
    } = data;
    let names = printed.unwrap_or_default();
    // Without a printed assignment the exercise is the stated owner's, as added.
    let assignment = if names.assigned.is_known() {
        names.assignment()
    } else {
        crate::document_names::Assignment::added(
            if owner == Owner::Spouse {
                crate::DocumentRecipient::Spouse
            } else {
                crate::DocumentRecipient::Taxpayer
            },
            None,
        )
    };
    IsoExerciseSource {
        id,
        assigned: assignment.owner,
        owner_source: assignment.owner_source,
        joint_share: assignment.joint_share,
        owner,
        form: Form3921Paper {
            transferor: names.issuer.unwrap_or_else(|| PrintedParty::from_name(label)),
            employee: names.recipient,
            granted,
            exercised,
            exercise_price,
            market_price,
            shares,
            corporation: names
                .transfer_corporation
                .unwrap_or_else(|| PrintedParty::from_name(corporation.unwrap_or_default())),
            corrected: false,
            account_number: String::new(),
        },
        more: Some(Form3921More { sold, recognition, option_cost }),
    }
}

pub(in crate::tests) fn iso_exercise(
    label: &str,
    shares: f64,
    exercise_price: i64,
    market_price: i64,
) -> IsoExerciseSource {
    iso_exercise_source(native_iso_exercise(label, shares, exercise_price, market_price))
}

pub(in crate::tests) fn iso_document(source: IsoExerciseSource) -> Form3921Paper {
    let form = source.form;
    remember(
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form3921(form.clone()) },
        vec![crate::DocumentAnswer::IsoExerciseFacts { value: source.more }],
    );
    form
}
