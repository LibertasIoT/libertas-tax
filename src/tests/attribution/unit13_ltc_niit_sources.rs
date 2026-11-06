use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};
use alloc::string::String;

fn fixture() -> FederalDraft {
    let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    append_injured_two_period_ltc_sources(&mut draft);
    draft
}

#[test]
fn unit13_ltc_qualified_benefits_do_not_erase_known_niit_owner() {
    let mut draft = fixture();
    let mut interest = income(savings_income(25_000, 0));
    interest.owner = crate::DocumentOwner::Spouses;
    add(&mut draft, interest);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        draft.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&draft);
        assert_eq!(value(&result, "taxable_ltc_benefits"), 3770);
        assert_eq!(value(&result, "agi"), 278_770);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 25_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 950);
        assert_eq!(
            [column(&result, "13b", "b"), column(&result, "13b", "c")],
            if injured == Owner::Taxpayer { [2100, 26_670] } else { [26_670, 2100] }
        );
        assert_injured_ordinary_unchanged(&draft, &result, supported);
        assert_eq!(
            [column(&result, "18", "b"), column(&result, "18", "c")],
            if injured == Owner::Taxpayer { [0, 950] } else { [950, 0] }
        );
        assert!(!noted(&result, "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"));
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
}

#[test]
fn unit13_ltc_interest_keeps_actual_payees_separate_from_care_benefits() {
    let mut draft = fixture();
    for (index, interest) in [(3, 500_000), (4, 1_500_000)] {
        let FederalIncome::LtcBenefits { data } = &mut draft.income[index].income else { panic!() };
        data.interest = Some(interest);
    }
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        draft.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&draft);
        assert_eq!(value(&result, "taxable_ltc_benefits"), 3770);
        assert_eq!(value(&result, "agi"), 273_770);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        assert_eq!(
            [column(&result, "13b", "b"), column(&result, "13b", "c")],
            if injured == Owner::Taxpayer { [7100, 16_670] } else { [16_670, 7100] }
        );
        assert_injured_ordinary_unchanged(&draft, &result, supported);
        assert_eq!(
            [column(&result, "18", "b"), column(&result, "18", "c")],
            if injured == Owner::Taxpayer { [190, 570] } else { [570, 190] }
        );
        assert!(!noted(&result, "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"));
        draft.income.reverse();
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
}

#[test]
fn unit13_ltc_interest_uses_the_native_aggregate_annual_boundary() {
    let mut draft = fixture();
    for row in &mut draft.income {
        if let FederalIncome::LtcBenefits { data } = &mut row.income {
            data.interest = Some(100_049);
        }
    }
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 255_771);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 2001);
    assert_eq!(value(&result, "net_investment_income_tax"), 76);
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [38, 38]);
    assert!(!noted(&result, "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"));
    assert_injured_ordinary_unchanged(&draft, &result, supported);
}

#[test]
fn unit13_ltc_accident_health_character_does_not_turn_benefits_into_interest() {
    for (funding, taxable) in
        [(CareBenefitFunding::EmployerOnly, 10_000), (CareBenefitFunding::AfterTax, 0)]
    {
        let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
        let FederalIncome::LtcBenefits { mut data } = ltc_form(10_000, None) else { panic!() };
        data.interest = Some(500_000);
        data.contract = LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth {
                funding,
                medical_reimbursement: 0,
                permanent_injury: None,
                recovery: None,
            },
        };
        add(&mut draft, income(FederalIncome::LtcBenefits { data }));
        let mut interest = income(savings_income(15_000, 0));
        interest.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, interest);
        let result = supported(&draft);
        assert_eq!(value(&result, "taxable_ltc_benefits"), taxable);
        assert_eq!(value(&result, "agi"), 270_000 + taxable);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        assert_eq!(
            [column(&result, "13b", "b"), column(&result, "13b", "c")],
            [5000 + taxable, 15_000]
        );
        assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [190, 570]);
        assert_injured_ordinary_unchanged(&draft, &result, supported);
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
}

#[test]
fn unit13_ltc_linked_actual_annuity_retains_income_and_niit_once() {
    use crate::tests::cross::interview::{accepted, r1099};
    let item = |owner, id, document| {
        crate::tests::support::builders::survey::survey_uncatalogued_document(owner, id, document)
    };
    let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19600101;
    let mut session = Interview::new(TEST_USER, draft);
    let mut annuity = r1099(1_000_000, DistributionCode::Nonqualified, false);
    annuity.source_mut().form.taxable = Some(600_000);
    accepted(&{
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![item(
                Owner::Spouse,
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(annuity)),
                },
            )],
        );
        let request = fixture_owner_inputs(&session.draft, request);
        let desired = crate::tests::support::client::income_pages::page_documents(&request);
        let reply = send(&mut session, request);
        accepted(&reply);
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut session,
            &desired,
        );
        reply
    });
    let source = session.draft.income.last().unwrap().id;
    let form = ltc_document(crate::LtcSource {
        interest: Some(100_000),
        form: Ltc1099Paper {
            payer: Default::default(),
            recipient: Default::default(),
            insured_party: Default::default(),
            date_certified: None,
            benefits: Some(1_000_000),
            accelerated: Some(0),
            paid: LtcPaidBasis::NotChecked,
            box4_qualified: false,
            illness: Some(LtcIllness::Chronic),
            calendar_year: 2026,
            corrected: false,
            payer_phone: String::new(),
            account_number: String::new(),
        },
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        contract: LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
        },
        insured: None,
    });
    accepted(&{
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![item(
                Owner::Spouse,
                0,
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(form) },
            )],
        );
        let request = fixture_owner_inputs(&session.draft, request);
        let desired = crate::tests::support::client::income_pages::page_documents(&request);
        let reply = send(&mut session, request);
        accepted(&reply);
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut session,
            &desired,
        );
        reply
    });
    // The interest is not a box of the paper: it is answered beside it.
    let ltc = session.draft.income.last().unwrap().id;
    accepted(&crate::tests::support::client::income_pages::answer_ltc_interest(
        &mut session,
        ltc,
        100_000,
    ));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&result, "agi"), 257_000);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 7000);
    assert_eq!(value(&result, "net_investment_income_tax"), 266);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [0, 7000]);
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [0, 266]);
    assert!(noted(&result, "WARN_LTC_ANNUITY_EXCLUSION_UNAVAILABLE"));
    assert!(!noted(&result, "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"));
    assert_injured_ordinary_unchanged(&session.draft, &result, supported);
    let mut reopened = restart(session.draft.clone());
    let page = crate::tests::support::client::income_pages::open_document_center(&mut reopened);
    let page = fixture_owner_inputs(&reopened.draft, page);
    accepted(&send(&mut reopened, page));
    assert_eq!(estimate(&reopened.draft), result);
}

#[test]
fn unit13_ltc_unknown_character_is_not_a_known_zero_investment_source() {
    // Isolate mandatory receipt character from a separate periods/payments mismatch.
    let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    let FederalIncome::LtcBenefits { mut data } = ltc_form(15_000, None) else { panic!() };
    data.contract = LtcContract::Unknown;
    data.interest = Some(500_000);
    add(&mut draft, income(FederalIncome::LtcBenefits { data }));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
    );
    assert!(result.lines.is_empty());
    assert!(result.tax.is_none());
}

#[test]
fn unit13_ltc_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let selected = std::env::var("TAX_UNIT13_LTC_NII_PROFILE").ok();
    for profile in ["taxpayer", "spouse"] {
        if selected.as_deref().is_some_and(|value| value != profile) {
            continue;
        }
        let mut draft = fixture();
        for (index, interest) in [(3, 500_000), (4, 1_500_000)] {
            let FederalIncome::LtcBenefits { data } = &mut draft.income[index].income else {
                panic!()
            };
            data.interest = Some(interest);
        }
        let mut session = Interview::new(TEST_USER, draft);
        let mut page =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let native = NativeInterviewCase("TAX_UNIT13_LTC_NII");
        if native.required_checked_calculation(&mut session) {
            return;
        }
        if native.seed(&page) {
            return;
        }
        let request = if let Some(request) = native.checked_input("PAYLOAD") {
            request
        } else {
            let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
            let id = if profile == "taxpayer" { 5 } else { 4 };
            let row =
                documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
            row.owner = if profile == "taxpayer" {
                crate::DocumentOwner::Yours
            } else {
                crate::DocumentOwner::Spouses
            };
            fixture_owner_inputs(&session.draft, page)
        };
        accepted(&send(&mut session, request));
        let correction =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        accepted(&send(&mut session, correction));
        let id = if profile == "taxpayer" { 5 } else { 4 };
        accepted(&crate::tests::support::client::income_pages::answer_ltc_interest(
            &mut session,
            id,
            if profile == "taxpayer" { 1_600_000 } else { 600_000 },
        ));
        let check = |draft: &FederalDraft| {
            let result = supported(draft);
            assert_eq!(value(&result, "taxable_ltc_benefits"), 3770);
            assert_eq!(value(&result, "agi"), 274_770);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 21_000);
            assert_eq!(value(&result, "net_investment_income_tax"), 798);
            assert_eq!(
                [column(&result, "13b", "b"), column(&result, "13b", "c")],
                if profile == "taxpayer" { [24_770, 0] } else { [0, 24_770] }
            );
            assert_eq!(
                [column(&result, "18", "b"), column(&result, "18", "c")],
                if profile == "taxpayer" { [798, 0] } else { [0, 798] }
            );
            assert!(!noted(&result, "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"));
            assert_injured_ordinary_unchanged(draft, &result, supported);
        };
        check(&session.draft);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let shown =
            crate::tests::support::client::income_pages::open_document_center(&mut reopened);
        native.output(&shown, &stored);
        let accepted_draft = reopened.draft.clone();
        accepted(&send(&mut reopened, shown));
        assert_eq!(reopened.draft, accepted_draft);
        check(&reopened.draft);
    }
}
