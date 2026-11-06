// form_1098 items of src/tests/support/builders/native_income.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(in crate::tests) fn first_lender_mortgage(
    interest: i64,
    principal: i64,
    average: AverageBalance,
) -> Mortgage1098 {
    Mortgage1098 {
        lender: Default::default(),
        borrower: Default::default(),
        id: 0,
        interest,
        principal,
        originated: 20200601,
        refund: 0,
        insurance: 0,
        points: 0,
        more: MortgageMore {
            details: MortgageDetails::FirstLender(MortgageFirstLender {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average,
            }),
            ..MortgageMore::default()
        },
    }
}

pub(in crate::tests) fn mortgage_1098_source() -> crate::federal_model::Mortgage1098Source {
    use crate::federal_model::{Form1098, Mortgage1098Source};
    let mut source = Mortgage1098Source::printed(Form1098 {
        calendar_year: 2026,
        corrected: true,
        lender: PrintedParty {
            name: "Fictional Mortgage Company".into(),
            federal_id: "12-3456789".into(),
            address: "108 Example Street".into(),
        },
        lender_telephone: "555-0108".into(),
        borrower: PrintedParty {
            name: "Synthetic Borrower".into(),
            federal_id: "987-65-4320".into(),
            address: "109 Example Street".into(),
        },
        account_number: "001-ABC".into(),
        box_1: 123_456,
        box_2: 10_000_000,
        box_3: 20200601,
        box_4: 1_234,
        box_5: 2_345,
        box_6: 3_456,
        box_7: true,
        box_8: "109 Example Street, parcel 001-XYZ".into(),
        box_9: 2,
        box_10: "Real estate taxes $456.78; escrow insurance $567.89".into(),
        box_11: Some(20260701),
    });
    source.id = 41;
    source.assigned = DocumentRecipient::Taxpayer;
    source.owner_source = Some(crate::OwnerSource::Added);
    source.property = "Synthetic home".into();
    source.insurance_contract = SourceDate::Reported { date: 20100601 };
    source.refund_origin = Some(MortgageRefundOrigin::NotDeducted);
    // A directly built loan carries its answers as Home mortgage interest saved them.
    source.answered = true;
    source
}

pub(in crate::tests) fn mortgage_source_return() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut source = mortgage_1098_source();
    source.form.borrower = accepted_document_names(&draft, Owner::Taxpayer).recipient;
    source.form.box_3 = 20260101;
    source.form.box_4 = 0;
    source.refund_origin = None;
    source.form.box_5 = 0;
    draft.next_id = source.id + 1;
    draft.deductions.as_mut().unwrap().mortgage = vec![source];
    draft
}

pub(in crate::tests) fn mortgage_source(form: Mortgage1098) -> Mortgage1098Source {
    let Mortgage1098 {
        lender,
        borrower,
        id,
        interest,
        principal,
        originated,
        refund,
        insurance,
        points,
        more,
    } = form;
    let MortgageMore {
        allowed_refund_origins: _,
        property,
        lender: loan_lender,
        details,
        refund_origin,
    } = more;
    let mut source = Mortgage1098Source::printed(Form1098 {
        calendar_year: 2026,
        lender,
        borrower,
        box_1: interest,
        box_2: principal,
        box_3: originated,
        box_4: refund,
        box_5: insurance,
        box_6: points,
        ..Default::default()
    });
    source.id = id;
    source.property = property;
    source.lender = loan_lender;
    source.details = details;
    source.refund_origin = refund_origin;
    // A directly built return: the borrower is you, as your section added it.
    source.assigned = crate::DocumentRecipient::Taxpayer;
    source.owner_source = Some(crate::OwnerSource::Added);
    // A directly built loan carries its answers as Home mortgage interest saved them.
    source.answered = true;
    source
}

pub(in crate::tests) fn mortgage_document(form: Mortgage1098) -> Form1098Document {
    let source = mortgage_source(form);
    Form1098Document { id: source.id, form: source.form, lender: source.lender }
}

pub(in crate::tests) fn mortgage_loan_source(loan: MortgageLoan) -> Mortgage1098Source {
    let MortgageLoan {
        printed,
        id,
        property,
        lender,
        interest,
        outstanding_principal,
        incurred,
        details,
        refund_of_interest,
        mortgage_insurance_premiums,
        points,
    } = loan;
    let names = printed.unwrap_or_default();
    let details = match details {
        MortgageLoanDetails::EarlierLender { loan } => MortgageDetails::EarlierLender { loan },
        MortgageLoanDetails::FirstLender(MortgageLedger { home, purpose, average }) => {
            MortgageDetails::FirstLender(MortgageFirstLender {
                home,
                purpose,
                average: match average {
                    MortgageAverageBalance::WithinLimit => AverageBalance::WithinLimit,
                    MortgageAverageBalance::FirstAndLast { last_balance } => {
                        AverageBalance::FirstAndLast { last_balance }
                    }
                    MortgageAverageBalance::InterestOverRate { rate } => {
                        AverageBalance::InterestOverRate { rate }
                    }
                    MortgageAverageBalance::Statements { average_balance } => {
                        AverageBalance::Statements { average: average_balance }
                    }
                    MortgageAverageBalance::Monthly { balances } => {
                        AverageBalance::Monthly { balances }
                    }
                },
            })
        }
    };
    let (refund, refund_origin) = match refund_of_interest {
        MortgageInterestRefund::No => (0, None),
        MortgageInterestRefund::Yes { amount, origin } => (amount, origin),
    };
    let (amount, point_treatment) = match points {
        MortgagePoints::No => (0, MortgagePointTreatment::None),
        MortgagePoints::Purchase { amount } => (amount, MortgagePointTreatment::Purchase),
        MortgagePoints::Amortized { amount, term_months, ended } => {
            (amount, MortgagePointTreatment::Amortized { term_months, ended })
        }
        MortgagePoints::Improvement { amount, term_months } => {
            (amount, MortgagePointTreatment::Improvement { term_months })
        }
    };
    let mut source = mortgage_source(Mortgage1098 {
        id,
        lender: names.issuer.unwrap_or_default(),
        borrower: names.recipient,
        interest,
        principal: outstanding_principal,
        originated: incurred,
        refund,
        insurance: mortgage_insurance_premiums,
        points: amount,
        more: MortgageMore { property, lender, details, refund_origin, ..Default::default() },
    });
    // Printed names carry their assignment; without one, mortgage_source's added
    // owner stands.
    if names.assigned.is_known() {
        crate::document_names::Assignment {
            owner: names.assigned,
            owner_source: names.owner_source,
            joint_share: names.joint_share,
        }
        .apply_to(&mut source.assigned, &mut source.owner_source, &mut source.joint_share);
    }
    source.points = point_treatment;
    source
}

pub(in crate::tests) fn mortgage_loan(source: &Mortgage1098Source) -> MortgageLoan {
    MortgageLoan::try_from(source)
        .expect("the supplied mortgage fixture has its required source answers")
}

pub(in crate::tests) fn mortgage_snapshot(source: &Mortgage1098Source) -> Mortgage1098 {
    let view = MortgageDocumentView::from(source);
    Mortgage1098 {
        id: view.id,
        lender: view.value.lender,
        borrower: view.value.borrower,
        interest: view.value.box_1,
        principal: view.value.box_2,
        originated: view.value.box_3,
        refund: view.value.box_4,
        insurance: view.value.box_5,
        points: view.value.box_6,
        more: MortgageMore {
            allowed_refund_origins: view.allowed_refund_origins,
            property: view.property,
            lender: view.lender,
            details: view.details.into(),
            refund_origin: view.refund_origin,
        },
    }
}
