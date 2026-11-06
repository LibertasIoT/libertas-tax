use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

fn draft(method: &str) -> (FederalDraft, K1CreditCode, i64) {
    let (mut d, data, code, amount) = if method == "child" {
        // Exact original F3b composition, before entering its printed credit.
        let mut d = young_interest_return();
        d.income[0].income = savings_income(81_350, 0);
        let mut data = ordinary_partner_source(10_000, false);
        data.income_source = Some(crate::kiddie::BusinessIncomeSource::Capital);
        (d, data, K1CreditCode::DifferentialWage, 3000)
    } else {
        assert_eq!(method, "averaging");
        (
            averaging_return(60_000),
            crate::tests::credits::housing_credit::housing_partner(&[]),
            K1CreditCode::LowIncomeHousingOther,
            4500,
        )
    };
    add(&mut d, income(FederalIncome::Partnership { data }));
    (
        crate::interview::with_ticks(
            crate::tests::support::builders::known_fact_partnership_draft(d),
        ),
        code,
        amount,
    )
}

fn assert_return(d: &FederalDraft, method: &str, amount: i64) {
    let r = supported(d);
    let (taxable, regular, capacity, line, credit) = if method == "child" {
        (90_000, 20_184, 2212, "6", "differential_wage")
    } else {
        (118_248, 14_623, 3783, "35", "low_income_housing")
    };
    assert_eq!(value(&r, "taxable_income"), taxable);
    assert_eq!(value(&r, "regular_income_tax"), regular);
    // Original independent method oracles: child20184−17972=2212 with the
    // same TI150000 parent; ScheduleJ14623−10840=3783 with its same60000
    // election and7101 base-year delta. Do not substitute1744 for F3b.
    assert_eq!(value(&r, &alloc::format!("form8582cr_line_{line}")), capacity);
    // Line37 is total passive credit allowed, also bounded by credit entered.
    assert_eq!(value(&r, "form8582cr_line_37"), amount.min(capacity));
    assert_eq!(value(&r, "general_business_credit"), amount.min(capacity));
    assert_eq!(
        value(&r, &alloc::format!("business_credit_{credit}_current_year_passive_to_next_year")),
        (amount - capacity).max(0)
    );
    assert!(!r.issues.iter().any(
        |issue| issue.display.key == "ISSUE_FORM_8582_CR_NEEDS_THE_FORM_8615_OR_SCHEDULE_J_TAX"
    ));
}

#[test]
fn unit16_passive_methods_actual_k1_editor_save_reopen_restart_correction() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let method = std::env::var("TAX_UNIT16_PASSIVE_METHOD").ok();
    let methods =
        method.as_deref().map_or_else(|| vec!["child", "averaging"], |method| vec![method]);
    for method in methods {
        let (d, code, original_amount) = draft(method);
        let original_people = d.people.clone();
        let original_averaging = d.screening.as_ref().unwrap().farm_averaging.clone();
        let mut session = Interview::new(TEST_USER, d);
        let native = NativeInterviewCase("TAX_UNIT16_PASSIVE");
        let correction = std::env::var("TAX_UNIT16_PASSIVE_CORRECTION").as_deref() == Ok("1");
        if correction && let Some(initial) = native.input("INITIAL_PAYLOAD") {
            accepted(&send(&mut session, initial));
            assert_return(&session.draft, method, original_amount);
        }
        if native.calculation(&mut session) {
            return;
        }
        let seed = crate::tests::support::client::income_pages::open_document_center(&mut session);
        native.write("SEED", &seed);
        let mut answers = seed;
        let P::SaveDocuments { documents, .. } = &mut answers else { panic!() };
        assert_eq!(documents.common.k1.documents.len(), 1);
        let source = documents.common.k1.documents[0].id;
        let crate::K1Document::K1Partnership(data) = &mut documents.common.k1.documents[0].document
        else {
            panic!()
        };
        let amount = if correction { 2000 } else { original_amount };
        crate::tests::support::builders::set_partnership_credits(
            data,
            &[K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                code,
                line: None,
                amount: amount * 100,
            }],
        );
        data.personal.as_mut().unwrap().rental_participation =
            code.rental().then_some(crate::K1RentalParticipation::NotActive);
        native.write("ANSWERS", &answers);
        if std::env::var("TAX_UNIT16_PASSIVE_SEED_ONLY").as_deref() == Ok("1") {
            return;
        }
        let client_payload = std::env::var("TAX_UNIT16_PASSIVE_PAYLOAD").ok();
        let request = native.input("PAYLOAD").unwrap_or(answers);
        accepted(&send(&mut session, request));
        assert_return(&session.draft, method, amount);
        assert_eq!(session.draft.people, original_people);
        assert_eq!(session.draft.screening.as_ref().unwrap().farm_averaging, original_averaging);
        assert_eq!(session.draft.income.iter().filter(|row| row.id == source).count(), 1);
        let saved = session.draft.clone();
        session = restart(saved.clone());
        assert_eq!(session.draft, saved);
        let reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        native.write("REOPENED", &reopened);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        assert_return(&session.draft, method, amount);
        if client_payload.is_none() && !correction {
            // Standalone native acceptance follows both methods through a
            // second actual Save without changing the parent/election facts.
            let mut changed =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let P::SaveDocuments { documents, .. } = &mut changed else { panic!() };
            let crate::K1Document::K1Partnership(data) =
                &mut documents.common.k1.documents[0].document
            else {
                panic!()
            };
            data.form.credits[0].amount = Some(200_000);
            accepted(&send(&mut session, changed));
            assert_return(&session.draft, method, 2000);
            assert_eq!(session.draft.people, original_people);
            assert_eq!(
                session.draft.screening.as_ref().unwrap().farm_averaging,
                original_averaging
            );
            assert_return(&restart(session.draft.clone()).draft, method, 2000);
        }
    }
}
