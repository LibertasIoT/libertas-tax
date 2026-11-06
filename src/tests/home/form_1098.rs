use super::*;
use crate::federal_model::{
    Form1098, Mortgage1098ConversionError as E, Mortgage1098Source, MortgageDebtDate,
    MortgagePaidShare,
};
use crate::tests::support::builders::native_income::mortgage_1098_source;

#[test]
fn paper_converter_preserves_every_loan_input_and_the_original_paper() {
    let mut source = mortgage_1098_source();
    source.paid_share = MortgagePaidShare::Amount(SourceCents::Reported { amount: 98_765 });
    source.debt_date = MortgageDebtDate::Draw { date: SourceDate::Reported { date: 20220302 } };
    source.points = MortgagePointTreatment::Amortized { term_months: 240, ended: true };
    let paper = source.form.clone();
    assert_eq!(
        MortgageLoan::try_from(&source),
        Ok(MortgageLoan {
            printed: Some(PrintedDocumentNames {
                recipient: paper.borrower.clone(),
                issuer: Some(paper.lender.clone()),
                insured_party: None,
                fiduciary: None,
                transfer_corporation: None,
                assigned: DocumentRecipient::Taxpayer,
                owner_source: Some(crate::OwnerSource::Added),
                joint_share: None,
            }),
            id: 41,
            property: "Synthetic home".into(),
            lender: MortgageLender::Institution,
            interest: 98_765,
            outstanding_principal: 10_000_000,
            incurred: 20220302,
            details: MortgageLoanDetails::FirstLender(MortgageLedger {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average: MortgageAverageBalance::WithinLimit,
            }),
            refund_of_interest: MortgageInterestRefund::Yes {
                amount: 1_234,
                origin: Some(MortgageRefundOrigin::NotDeducted),
            },
            mortgage_insurance_premiums: 2_345,
            points: MortgagePoints::Amortized { amount: 3_456, term_months: 240, ended: true },
        }),
    );
    assert_eq!(source.form, paper);
}

#[test]
fn paper_converter_keeps_zero_paid_share_distinct_from_unavailable() {
    let mut source = mortgage_1098_source();
    source.paid_share = MortgagePaidShare::Amount(SourceCents::NotReported);
    assert_eq!(MortgageLoan::try_from(&source).unwrap().interest, 0);
    source.paid_share = MortgagePaidShare::Amount(SourceCents::Unavailable);
    assert_eq!(MortgageLoan::try_from(&source), Err(E::PaidInterestUnavailable));
    source.paid_share = MortgagePaidShare::Amount(SourceCents::Reported { amount: 123_456 });
    assert_eq!(MortgageLoan::try_from(&source).unwrap().interest, 123_456);
    source.paid_share = MortgagePaidShare::Amount(SourceCents::Reported { amount: 123_457 });
    assert_eq!(MortgageLoan::try_from(&source), Err(E::PaidInterestAboveBox1));
}

#[test]
fn paper_converter_requires_an_actual_contract_date_only_for_reported_premiums() {
    let mut source = mortgage_1098_source();
    for missing in [SourceDate::Unavailable, SourceDate::NotReported] {
        source.insurance_contract = missing;
        // The converter keeps the earlier behavior (dispatcher decision of 2026-10-26 11:45):
        // an unanswered contract date counts the printed premiums.
        assert_eq!(MortgageLoan::try_from(&source).unwrap().mortgage_insurance_premiums, 2_345);
    }
    source.form.box_5 = 0;
    assert_eq!(MortgageLoan::try_from(&source).unwrap().mortgage_insurance_premiums, 0);
}

#[test]
fn paper_converter_uses_the_existing_insurance_contract_cutoff() {
    let mut source = mortgage_1098_source();
    source.insurance_contract = SourceDate::Reported { date: 20061231 };
    assert_eq!(MortgageLoan::try_from(&source).unwrap().mortgage_insurance_premiums, 0);
    source.insurance_contract = SourceDate::Reported { date: 20070101 };
    assert_eq!(MortgageLoan::try_from(&source).unwrap().mortgage_insurance_premiums, 2_345);
    assert_eq!(source.form.box_5, 2_345);
}

#[test]
fn paper_converter_never_substitutes_origination_for_a_missing_draw_date() {
    let mut source = mortgage_1098_source();
    for missing in [SourceDate::Unavailable, SourceDate::NotReported] {
        source.debt_date = MortgageDebtDate::Draw { date: missing };
        assert_eq!(MortgageLoan::try_from(&source), Err(E::DrawDateNeeded));
    }
    source.debt_date = MortgageDebtDate::Origination;
    assert_eq!(MortgageLoan::try_from(&source).unwrap().incurred, 20200601);
}

#[test]
fn paper_converter_requires_only_the_points_treatment_called_for_by_box_6() {
    let mut source = mortgage_1098_source();
    assert_eq!(
        MortgageLoan::try_from(&source).unwrap().points,
        MortgagePoints::Purchase { amount: 3_456 },
    );
    source.points = MortgagePointTreatment::Improvement { term_months: 180 };
    assert_eq!(
        MortgageLoan::try_from(&source).unwrap().points,
        MortgagePoints::Improvement { amount: 3_456, term_months: 180 },
    );
    source.points = MortgagePointTreatment::None;
    assert_eq!(MortgageLoan::try_from(&source), Err(E::PointsTreatmentNeeded));
    source.form.box_6 = 0;
    assert_eq!(MortgageLoan::try_from(&source).unwrap().points, MortgagePoints::No);
}

#[test]
fn paper_converter_retains_the_canonical_lender_link_and_unresolved_refund() {
    let mut source = mortgage_1098_source();
    source.details = MortgageDetails::EarlierLender { loan: 39 };
    source.refund_origin = None;
    let loan = MortgageLoan::try_from(&source).unwrap();
    assert_eq!(loan.details, MortgageLoanDetails::EarlierLender { loan: 39 });
    assert_eq!(
        loan.refund_of_interest,
        MortgageInterestRefund::Yes { amount: 1_234, origin: None }
    );
}

#[test]
fn paper_replacement_retains_answers_only_for_the_same_durable_source() {
    let mut accepted = mortgage_1098_source();
    accepted.paid_share = MortgagePaidShare::Amount(SourceCents::Reported { amount: 50_000 });
    accepted.points = MortgagePointTreatment::Amortized { term_months: 180, ended: false };
    let mut edited = Mortgage1098Source::printed(Form1098 {
        box_1: 222_222,
        box_5: 0,
        box_6: 0,
        ..accepted.form.clone()
    });
    edited.id = accepted.id;
    let paper = edited.form.clone();
    edited.keep_answers(&accepted);
    let mut expected = accepted.clone();
    expected.form = paper;
    assert_eq!(edited, expected);
    edited.form.borrower.name = "Another synthetic borrower".into();
    edited.keep_answers(&accepted);
    assert_eq!(edited.assigned, crate::DocumentRecipient::Unknown);
    edited.id = 42;
    let other = edited.clone();
    edited.keep_answers(&accepted);
    assert_eq!(edited, other);
}

#[test]
fn printed_paper_has_visible_usual_choices_and_no_fabricated_contract_date() {
    let source = Mortgage1098Source::printed(Form1098 {
        calendar_year: 2026,
        box_3: 20200601,
        ..Default::default()
    });
    assert_eq!(source.paid_share, MortgagePaidShare::All);
    assert_eq!(source.insurance_contract, SourceDate::Unavailable);
    assert_eq!(source.points, MortgagePointTreatment::None);
    assert_eq!(source.debt_date, MortgageDebtDate::Origination);
}
