// Every must-answer canonical row added by D-2026-11-04-06 part D: an
// unanswered row is a strict question asked unanswered (no Default), and an
// answered one round-trips through its strict twin to the same stored value.
use crate::document_followup_page::strict_answer;
use crate::{DocumentAnswer as A, DocumentStrictAnswer as T};

fn carried() -> crate::PartnerCarried {
    let start =
        crate::tests::cross::interview::k1_1065_continuing(0, crate::ScorpRiskStart::Unknown);
    let crate::BasisStart::Continuing { carried, .. } = start else { unreachable!() };
    carried
}

fn round_trip(unanswered: A, answered: A) {
    assert_eq!(strict_answer(&unanswered), Some(None), "{unanswered:?}");
    assert!(T::kind_of(&unanswered).is_some());
    let strict = strict_answer(&answered).unwrap().unwrap();
    assert_eq!(T::kind_of(&answered), Some(strict.index()));
    assert_eq!(strict.into_canonical(), Some(answered));
}

#[test]
fn part_d_strict_twins_round_trip_to_their_canonical_rows() {
    round_trip(
        A::K1RentalParticipation { value: None },
        A::K1RentalParticipation { value: Some(crate::K1RentalParticipation::Professional) },
    );
    round_trip(
        A::K1PartnerBasisStart {
            value: Some(crate::BasisStart::Continuing {
                opening: None,
                risk_history: crate::ScorpRiskStart::Unknown,
                carried: carried(),
            }),
        },
        A::K1PartnerBasisStart {
            value: Some(crate::BasisStart::Continuing {
                opening: Some(12_300),
                risk_history: crate::ScorpRiskStart::Unknown,
                carried: carried(),
            }),
        },
    );
    round_trip(
        A::K1ScorpBasisStart { value: None },
        A::K1ScorpBasisStart { value: Some(crate::K1ScorpStart::New) },
    );
    for line in [crate::BusinessCredit::Research, crate::BusinessCredit::ResearchSmallBusiness] {
        round_trip(
            A::K1ResearchCreditLine { value: None },
            A::K1ResearchCreditLine { value: Some(line) },
        );
        round_trip(
            A::EstateResearchCreditLine { value: None },
            A::EstateResearchCreditLine { value: Some(line) },
        );
    }
    round_trip(
        A::DebtAcquisition { value: None },
        A::DebtAcquisition { value: Some(crate::AcquiredPropertyView::CccCrop { loan: 7 }) },
    );
    round_trip(
        A::EstateFinalSuccessor { value: None },
        A::EstateFinalSuccessor { value: Some(false) },
    );
    round_trip(
        A::EsppAcquisition { value: None },
        A::EsppAcquisition { value: Some(crate::EsppAcquisitionPage::Printed { document_id: 3 }) },
    );
    round_trip(
        A::SavingsBondsRecords { value: None },
        A::SavingsBondsRecords {
            value: Some(crate::SavingsBondAnswersView {
                interest_statement: 4,
                students: alloc::vec![crate::SavingsBondStudent {
                    person: -2,
                    expenses: crate::SavingsBondExpenses::EducationCredit,
                    contributions: 0,
                }],
            }),
        },
    );
    round_trip(
        A::K1Section179Amt { value: None },
        A::K1Section179Amt {
            value: Some(crate::Section179IncomeCarryHistory {
                gross: 0,
                sold: 0,
                vintages: alloc::vec::Vec::new(),
            }),
        },
    );
    round_trip(
        A::MiscellaneousUse { value: None },
        A::MiscellaneousUse { value: Some(crate::IncomeUse::FromActivity) },
    );
    round_trip(
        A::MiscellaneousUse { value: None },
        A::MiscellaneousUse { value: Some(crate::IncomeUse::NotForProfit) },
    );
    round_trip(
        A::SocialPremiums { value: None },
        A::SocialPremiums { value: Some(crate::SocialPremiumUsePage::Medical) },
    );
    round_trip(
        A::SocialReimbursedPremiums { reimbursed: None },
        A::SocialReimbursedPremiums { reimbursed: Some(0) },
    );
    round_trip(
        A::SocialSharedPremiums { value: None, reimbursed: Some(5) },
        A::SocialSharedPremiums {
            value: Some(crate::SocialPremiumUsePage::AlreadyInSelfEmployedPlan {
                income_id: 2,
                amount: 4_000,
            }),
            reimbursed: Some(0),
        },
    );
}

// The basis start is unanswered only while a continuing interest's opening
// basis is unknown; a stored answer converts both ways unchanged.
#[test]
fn strict_basis_start_reads_and_writes_the_stored_start() {
    let carried = carried();
    let open = crate::BasisStart::Continuing {
        opening: None,
        risk_history: crate::ScorpRiskStart::Unknown,
        carried: carried.clone(),
    };
    assert_eq!(crate::StrictPartnerStart::from_stored(&open), None);
    let known = crate::BasisStart::Continuing {
        opening: Some(0),
        risk_history: crate::ScorpRiskStart::Unknown,
        carried,
    };
    let strict = crate::StrictPartnerStart::from_stored(&known).unwrap();
    assert_eq!(strict.stored(), known);
    assert_eq!(
        crate::StrictPartnerStart::from_stored(&crate::BasisStart::New).unwrap().stored(),
        crate::BasisStart::New
    );
}
