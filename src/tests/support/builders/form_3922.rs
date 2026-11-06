use crate::tests::*;

fn espp_acquisition_document_native(form: Form3922) -> Form3922Paper {
    let Form3922 {
        corporation,
        employee,
        account_number,
        corrected,
        granted,
        exercised,
        grant_market,
        market_price,
        exercise_price,
        shares,
        transferred,
        grant_price,
    } = form;
    Form3922Paper {
        corporation,
        employee,
        account_number,
        corrected,
        granted,
        exercised,
        grant_market,
        market_price,
        exercise_price,
        shares,
        transferred,
        grant_price,
    }
}

paper_adapter!(
    espp_acquisition_document,
    espp_acquisition_document_native,
    EsppBoxFixture,
    Form3922,
    Form3922Paper
);
