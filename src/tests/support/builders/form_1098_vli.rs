extern crate std;

use crate::tests::*;

fn vehicle_paper_native(form: FederalVehicleInterest) -> Form1098VliPaper {
    let FederalVehicleInterest {
        lender,
        borrower,
        interest,
        refund,
        original_use,
        assembled_in_us,
    } = form;
    Form1098VliPaper {
        lender,
        payer_of_record: borrower,
        box_1: interest,
        box_5: Some(refund),
        box_6: original_use,
        box_7: assembled_in_us,
        calendar_year: 2026,
        ..Default::default()
    }
}

paper_adapter!(
    vehicle_paper,
    vehicle_paper_native,
    VehicleLoanBoxFixture,
    FederalVehicleInterest,
    Form1098VliPaper
);

std::thread_local! {
    static VEHICLE_ANSWERS: core::cell::RefCell<Vec<(Form1098VliDocument, i64)>> = const { core::cell::RefCell::new(Vec::new()) };
}

pub(in crate::tests) fn vehicle_document(source: VehicleLoanSource) -> Form1098VliDocument {
    let form = Form1098VliDocument { id: source.id, form: source.document };
    VEHICLE_ANSWERS.with(|rows| rows.borrow_mut().push((form.clone(), source.business)));
    form
}

pub(in crate::tests) fn fixture_vehicle_business(form: &Form1098VliDocument) -> Option<i64> {
    VEHICLE_ANSWERS.with(|rows| {
        rows.borrow().iter().rev().find(|(given, _)| given == form).map(|(_, value)| *value)
    })
}
