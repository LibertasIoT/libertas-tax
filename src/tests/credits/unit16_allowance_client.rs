use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

fn draft(pre1990: bool) -> (FederalDraft, i64) {
    // Exact f08f67e original F6/F7 composition: standard16100 + QBI3900.
    let agi = if pre1990 { 210_000 } else { 120_000 };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, (agi - 19_500) * 100, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(19_500, true)) }),
    );
    if pre1990 {
        add(&mut d, income(FederalIncome::Partnership { data: ordinary_partner_source(0, false) }));
    } else {
        let mut data = rental(10_000, 0);
        // Genuine absence replaces only the helper's synthetic zero expense.
        // Gross rent, building/history, depreciation and every oracle stay.
        data.expenses.clear();
        add(&mut d, income(FederalIncome::Rental { data }));
        history(&mut d);
    }
    let source = d.income.last().unwrap().id;
    name_accepted_income_documents(&mut d);
    (crate::tests::support::builders::known_fact_partnership_draft(d), source)
}

fn open(session: &mut Interview, pre1990: bool, boundary: bool) -> P {
    use crate::tests::cross::interview::navigate;
    if boundary {
        return crate::tests::support::client::income_pages::open_document_center(session);
    }
    if pre1990 {
        return crate::tests::support::client::income_pages::open_document_center(session);
    }
    let current = session.draft.clone();
    match send(session, navigate(&current, TaxSection::MoreCredits)) {
        Reply::Edit(request @ P::SaveMoreCredits { .. }) => request,
        other => panic!("expected More credits: {other:?}"),
    }
}

fn answer(request: &mut P, pre1990: bool, boundary: bool, source: i64) {
    if boundary {
        let P::SaveDocuments { documents, .. } = request else { panic!() };
        assert_eq!(documents.common.wages.documents.len(), 1);
        let crate::WageDocument::W2(w2) = &mut documents.common.wages.documents[0].document else {
            panic!()
        };
        // Same original F6/F7 phase-out boundaries, only printed box1 changes.
        w2.wages = Some((if pre1990 { 250_000 - 19_500 } else { 150_000 - 19_500 }) * 100);
    } else if pre1990 {
        let P::SaveDocuments { documents, .. } = request else { panic!() };
        let row = documents.common.k1.documents.iter_mut().find(|row| row.id == source).unwrap();
        let crate::K1Document::K1Partnership(data) = &mut row.document else { panic!() };
        crate::tests::support::builders::set_partnership_credits(
            data,
            &[K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                code: K1CreditCode::LowIncomeHousingOther,
                line: None,
                amount: 300_000,
            }],
        );
        data.personal.as_mut().unwrap().housing_confirmed =
            Some(crate::K1HousingAcquisition::Before1990);
        data.personal.as_mut().unwrap().rental_participation =
            Some(crate::K1RentalParticipation::NotActive);
    } else {
        let P::SaveMoreCredits { value, .. } = request else { panic!() };
        let MoreCredit::BusinessCredits(history) = value
            .credits
            .iter_mut()
            .find(|credit| matches!(credit, MoreCredit::BusinessCredits(_)))
            .unwrap()
        else {
            panic!()
        };
        history.credits = vec![
            EnteredBusinessCredit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                credit: BusinessCredit::DisabledAccess,
                source: CreditSource::Rental { rental: Some(source), reduced: true },
                amount: 200_000,
                election: CreditElection::None,
            }
            .into(),
        ];
    }
}

fn assert_return(d: &FederalDraft, pre1990: bool, boundary: bool) {
    let r = supported(d);
    if boundary {
        assert_eq!(value(&r, "agi"), if pre1990 { 250_000 } else { 150_000 });
        assert_eq!(
            value(&r, if pre1990 { "form8582cr_line_22" } else { "form8582cr_line_10" }),
            if pre1990 { 250_000 } else { 150_000 }
        );
        assert_eq!(value(&r, if pre1990 { "form8582cr_line_30" } else { "form8582cr_line_16" }), 0);
        assert_eq!(value(&r, "form8582cr_line_37"), 0);
        assert_eq!(value(&r, "general_business_credit"), 0);
        assert_eq!(
            value(
                &r,
                if pre1990 {
                    "business_credit_low_income_housing_current_year_passive_to_next_year"
                } else {
                    "business_credit_disabled_access_current_year_passive_to_next_year"
                }
            ),
            if pre1990 { 3000 } else { 2000 }
        );
        return;
    }
    assert_eq!(value(&r, "agi"), if pre1990 { 210_000 } else { 120_000 });
    assert_eq!(value(&r, "qbi_deduction"), 3900);
    assert_eq!(value(&r, "taxable_income"), if pre1990 { 190_000 } else { 100_000 });
    let rows = if pre1990 {
        vec![
            ("regular_income_tax", 38198),
            ("form_3800_line_14", 30160),
            ("form8582cr_line_2a", 3000),
            ("form8582cr_line_21", 250000),
            ("form8582cr_line_22", 210000),
            ("form8582cr_line_26", 20000),
            ("form8582cr_line_27", 4800),
            ("form8582cr_line_30", 3000),
            ("form8582cr_line_37", 3000),
            ("general_business_credit", 3000),
        ]
    } else {
        vec![
            ("regular_income_tax", 16712),
            ("form_3800_line_14", 6760),
            ("form_3800_line_16", 9952),
            ("form8582cr_line_1a", 2000),
            ("form8582cr_line_10", 120000),
            ("form8582cr_line_14", 15000),
            ("form8582cr_line_15", 3294),
            ("form8582cr_line_16", 2000),
            ("form8582cr_line_37", 2000),
            ("general_business_credit", 2000),
        ]
    };
    for (key, amount) in rows {
        assert_eq!(value(&r, key), amount, "{key}");
    }
}

#[test]
fn unit16_f6_f7_actual_allowance_source_save_reopen_restart_boundary() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let profile = std::env::var("TAX_UNIT16_ALLOWANCE_PROFILE").ok();
    let profiles = profile.as_deref().map_or_else(|| vec!["rental", "pre1990"], |p| vec![p]);
    for profile in profiles {
        assert!(matches!(profile, "rental" | "pre1990"));
        let pre1990 = profile == "pre1990";
        let boundary = std::env::var("TAX_UNIT16_ALLOWANCE_BOUNDARY").as_deref() == Ok("1");
        let (d, source) = draft(pre1990);
        let people = d.people.clone();
        let mut expected_sources = d.income[1..].to_vec();
        if pre1990 {
            let entry = expected_sources.last_mut().unwrap();
            let FederalIncome::Partnership { data } = &mut entry.income else { panic!() };
            let kept = data.source.as_ref().unwrap();
            let assigned = (kept.assigned, kept.owner_source, kept.joint_share);
            let mut expected = crate::K1PartnershipDocument::from_record(&entry.label, data)
                .expect("complete original partnership fixture");
            expected.personal.as_mut().unwrap().housing_confirmed =
                Some(crate::K1HousingAcquisition::Before1990);
            crate::tests::support::builders::set_partnership_credits(
                &mut expected,
                &[K1Credit {
                    carbon_oxide_origin: None,
                    renewable_facility_origin: None,
                    code: K1CreditCode::LowIncomeHousingOther,
                    line: None,
                    amount: 300_000,
                }],
            );
            expected.personal.as_mut().unwrap().rental_participation =
                Some(crate::K1RentalParticipation::NotActive);
            *data = expected.record();
            let source = data.source.as_mut().unwrap();
            (source.assigned, source.owner_source, source.joint_share) = assigned;
        }
        let mut session = Interview::new(TEST_USER, d);
        let native = NativeInterviewCase("TAX_UNIT16_ALLOWANCE");
        if boundary {
            let initial = if let Some(initial) = native.input("INITIAL_PAYLOAD") {
                initial
            } else {
                let mut request = open(&mut session, pre1990, false);
                answer(&mut request, pre1990, false, source);
                request
            };
            accepted(&if matches!(initial, P::SaveDocuments { .. }) {
                crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    initial,
                )
            } else {
                send(&mut session, initial)
            });
            assert_return(&session.draft, pre1990, false);
        }
        if native.calculation(&mut session) {
            return;
        }
        let seed = open(&mut session, pre1990, boundary);
        native.write("SEED", &seed);
        let mut answers = seed;
        answer(&mut answers, pre1990, boundary, source);
        native.write("ANSWERS", &answers);
        if std::env::var("TAX_UNIT16_ALLOWANCE_SEED_ONLY").as_deref() == Ok("1") {
            return;
        }
        let payload = std::env::var("TAX_UNIT16_ALLOWANCE_PAYLOAD").ok();
        let request = native.input("PAYLOAD").unwrap_or(answers);
        accepted(&if matches!(request, P::SaveDocuments { .. }) {
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        } else {
            send(&mut session, request)
        });
        assert_return(&session.draft, pre1990, boundary);
        assert_eq!(session.draft.people, people);
        assert_eq!(session.draft.income[1..], expected_sources);
        assert_eq!(session.draft.income.iter().filter(|row| row.id == source).count(), 1);
        let saved = session.draft.clone();
        session = restart(saved.clone());
        assert_eq!(session.draft, saved);
        let reopened = open(&mut session, pre1990, boundary);
        native.write("REOPENED", &reopened);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        assert_return(&session.draft, pre1990, boundary);
        if payload.is_none() && !boundary {
            let mut request = open(&mut session, pre1990, true);
            answer(&mut request, pre1990, true, source);
            accepted(&if matches!(request, P::SaveDocuments { .. }) {
                crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    request,
                )
            } else {
                send(&mut session, request)
            });
            assert_return(&session.draft, pre1990, true);
            assert_eq!(session.draft.people, people);
            assert_eq!(session.draft.income[1..], expected_sources);
            assert_return(&restart(session.draft.clone()).draft, pre1990, true);
        }
    }
}
