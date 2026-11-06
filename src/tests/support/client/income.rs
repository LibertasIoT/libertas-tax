use super::lifecycle::{TEST_USER, restart, send};
use crate::interview::Interview;
use crate::tests::cross::interview::{accepted, from_review, open_topic};
use crate::tests::support::assert::assert_native_cash_money;
use crate::tests::support::builders::complete;
use crate::tests::*;
use crate::{
    BasisStart, Farm, FederalDraft, FederalResult, FederalStatus, IncomeSource,
    TaxInterviewProtocol as P, TaxSection,
};
use alloc::vec::Vec;

// Bind Form8815 to the assigned ID from the accepted 1099-INT source.
pub(in crate::tests) fn education_bond_session(mut interest: Int1099, amount: i64) -> Interview {
    use crate::tests::cross::interview::{document, page_documents, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    interest.taxable = 0;
    interest.more = IntMore { us_obligations: 300_000, ..Default::default() };
    interest.adjustments =
        Some(InterestAdjustments { previously_reported: 100_000, ..Default::default() });
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
            },
        )],
    );
    accepted(&super::income_pages::save_source_fixture_request(&mut session, request));
    let sources = page_documents(&super::income_pages::open_document_center(&mut session));
    let mut interest_sources = sources.into_iter().filter(|source| {
        matches!(
            source.document,
            IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) }
        )
    });
    let source = interest_sources.next().expect("accepted 1099-INT");
    assert!(interest_sources.next().is_none());
    let bonds = IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
            savings_bonds_source(SavingsBonds {
                source_issuer: Default::default(),
                source_recipient: Default::default(),
                proceeds: 1_200_000,
                redemption_interest: amount,
                earliest_issue_date: 20100601,
                interest_statement: source.id,
                students: vec![SavingsBondStudent {
                    person: -1,
                    expenses: SavingsBondExpenses::Entered { tuition: 1_400_000, benefits: 0 },
                    contributions: 0,
                }],
            }),
        )),
    };
    let request = save_documents(&mut session, vec![source, document(0, bonds)]);
    accepted(&super::income_pages::save_source_fixture_request(&mut session, request));
    session
}

pub(in crate::tests) fn estimate_documents_from_save(request: &P) -> P {
    if matches!(request, P::SaveEducation { .. } | P::SaveHsa { .. } | P::SaveInjuredSpouse { .. })
    {
        return super::source_controls::source_calculation(request);
    }
    if matches!(request, P::SaveDocuments { .. }) {
        return request.clone();
    }
    if let P::SaveWagesAnswers {
        wages,
        followups,
        records,
        businesses,
        wage_scorporations,
        allowed_wage_scorporations,
        allowed_owners,
        allowed_community,
        allowed_refund_management,
        ..
    } = request
    {
        return P::EstimateWagesAnswers {
            user: TEST_USER,
            wages: wages.clone(),
            followups: followups.clone(),
            records: records.clone(),
            questions: Vec::new(),
            situations: crate::DocumentSituations::NoneOfThese,
            allowed_situations: Vec::new(),
            situation_answers: Vec::new(),
            situation_documents: Vec::new(),
            disaster_choices: Vec::new(),
            people: Vec::new(),
            inherited_iras: Vec::new(),
            inherited_roth_iras: Vec::new(),
            businesses: businesses.clone(),
            wage_scorporations: wage_scorporations.clone(),
            allowed_wage_scorporations: allowed_wage_scorporations.clone(),
            allowed_owners: allowed_owners.clone(),
            allowed_community: allowed_community.clone(),
            allowed_refund_management: allowed_refund_management.clone(),
        };
    }
    let P::SaveIncomes {
        cookie,
        revision,
        followups,
        empty_k1_activity_choices,
        allowed_injured_source_facts,
        allowed_injured_retained_owners,
        ltc_payee_sources,
        wage_scorporations,
        allowed_wage_scorporations,
        initial_k1_investment_choices,
        research_businesses,
        retained_research_choices,
        k1_credit_lines,
        research_credit_lines,
        renewable_credit_lines,
        transferable_credit_lines,
        allowed_continuing_risk,
        topic,
        source_choices,
        allowed_documents,
        allowed_owners,
        allowed_community,
        allowed_refund_management,
        foreign_countries,
        interest_statements,
        people,
        farms,
        cooperative_activities,

        cooperative_properties,
        businesses,
        qbi_businesses,
        rentals,
        mortgage_loans,
        rental_energy_buildings,
        casualty_properties,
        royalty_properties,
        equipment_rentals,
        hobbies,
        ccc_lots,
        farm_codes,
        farm_limitations,
        farm_sources,
        basis_assets,
        farm_method_changes,
        inherited_iras,
        inherited_roth_iras,
        ltc_insured,
        annuity_sources,
        recovery_sources,
        allowed_business_source,
        allowed_farm_source,
        allowed_partner_source,
        allowed_home_spouse,
        allowed_other_parent,
        children,
        documents,
        ..
    } = request
    else {
        unreachable!()
    };
    P::EstimateDocuments {
        cookie: cookie.clone(),
        revision: *revision,
        followups: followups.clone(),
        questions: Vec::new(),
        situations: crate::DocumentSituations::NoneOfThese,
        allowed_situations: Vec::new(),
        situation_answers: Vec::new(),
        situation_documents: Vec::new(),
        disaster_choices: Vec::new(),
        initial_asset_amt_carry_choices: Vec::new(),
        initial_interest_limit_choices: Vec::new(),
        initial_rental_recovery_choices: Vec::new(),
        empty_k1_activity_choices: empty_k1_activity_choices.clone(),
        user: TEST_USER,
        allowed_injured_source_facts: allowed_injured_source_facts.clone(),
        allowed_injured_retained_owners: allowed_injured_retained_owners.clone(),
        ltc_payee_sources: ltc_payee_sources.clone(),
        wage_scorporations: wage_scorporations.clone(),
        allowed_wage_scorporations: allowed_wage_scorporations.clone(),
        initial_k1_investment_choices: initial_k1_investment_choices.clone(),
        research_businesses: research_businesses.clone(),
        retained_research_choices: retained_research_choices.clone(),
        k1_credit_lines: k1_credit_lines.clone(),
        research_credit_lines: research_credit_lines.clone(),
        renewable_credit_lines: renewable_credit_lines.clone(),
        transferable_credit_lines: transferable_credit_lines.clone(),
        allowed_continuing_risk: allowed_continuing_risk.clone(),
        topic: *topic,
        source_choices: source_choices.clone(),
        allowed_documents: allowed_documents.clone(),
        allowed_owners: allowed_owners.clone(),
        allowed_community: allowed_community.clone(),
        allowed_refund_management: allowed_refund_management.clone(),
        foreign_countries: foreign_countries.clone(),
        interest_statements: interest_statements.clone(),
        people: people.clone(),
        farms: farms.clone(),
        cooperative_activities: cooperative_activities.clone(),

        cooperative_properties: cooperative_properties.clone(),
        businesses: businesses.clone(),
        qbi_businesses: qbi_businesses.clone(),
        rentals: rentals.clone(),
        mortgage_loans: mortgage_loans.clone(),
        rental_energy_buildings: rental_energy_buildings.clone(),
        casualty_properties: casualty_properties.clone(),
        royalty_properties: royalty_properties.clone(),
        equipment_rentals: equipment_rentals.clone(),
        hobbies: hobbies.clone(),
        ccc_lots: ccc_lots.clone(),
        farm_codes: farm_codes.clone(),
        farm_limitations: farm_limitations.clone(),
        farm_sources: farm_sources.clone(),
        basis_assets: basis_assets.clone(),
        farm_method_changes: farm_method_changes.clone(),
        inherited_iras: inherited_iras.clone(),
        inherited_roth_iras: inherited_roth_iras.clone(),
        ltc_insured: ltc_insured.clone(),
        annuity_sources: annuity_sources.clone(),
        recovery_sources: recovery_sources.clone(),
        allowed_business_source: allowed_business_source.clone(),
        allowed_farm_source: allowed_farm_source.clone(),
        allowed_partner_source: allowed_partner_source.clone(),
        allowed_home_spouse: allowed_home_spouse.clone(),
        allowed_other_parent: allowed_other_parent.clone(),
        children: children.clone(),
        documents: documents.clone().into(),
    }
}

// Admit the entered record and topic facts of a synthetic native fixture through
// the actual checklist before exercising its source-owner lifecycle.
pub(in crate::tests) fn admit_native_fixture_facts(session: &mut Interview) {
    session.draft =
        crate::tests::support::builders::known_fact_partnership_draft(session.draft.clone());
    // Assembled accepted fixtures need the same source identities as a saved
    // inventory before ownership rows can refer to their distinct documents.
    let accepted_sources = session.draft.clone();
    crate::interview::assign_native_document_ids(&mut session.draft, &accepted_sources)
        .expect("distinct accepted native fixture documents");
    let value = crate::interview::with_ticks(session.draft.clone()).checklist;
    let reply = crate::tests::cross::interview::from_review(session, TaxSection::Checklist);
    let request = crate::tests::cross::interview::tick(reply, value);
    accepted(&send(session, request));
}

pub(in crate::tests) fn save_native_cash_sources(
    d: FederalDraft,
    topic: TaxSection,
    categories: [i64; 3],
    agi: i64,
) -> FederalResult {
    let before = assert_native_cash_money(&d, categories, agi);
    let mut session = restart(d);
    admit_native_fixture_facts(&mut session);
    let page = super::income_pages::open_document_center(&mut session);
    crate::tests::cross::interview::accepted(&send(&mut session, page));
    save_native_topic_records(&mut session, topic);
    let persisted = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &persisted);
    assert_eq!(native_draft_records(&draft), persisted);
    assert_eq!(draft, session.draft);
    let mut session = restart(draft);
    let page = super::income_pages::open_document_center(&mut session);
    crate::tests::cross::interview::accepted(&send(&mut session, page));
    save_native_topic_records(&mut session, topic);
    let after = assert_native_cash_money(&session.draft, categories, agi);
    assert_eq!(before.return_lines, after.return_lines);
    assert_eq!(before.withholding_sources, after.withholding_sources);
    after
}

pub(crate) fn save_farms(session: &mut Interview, farms: Vec<Farm>) {
    let request = crate::tests::cross::interview::save_topic(
        session,
        TaxSection::Farming,
        farms
            .into_iter()
            .map(|farm| {
                crate::tests::cross::interview::document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) },
                )
            })
            .collect(),
    );
    crate::tests::cross::interview::accepted(&send(session, request));
    assert!(session.draft.valid_stored());
}

pub(in crate::tests) fn d6_ptp_source_session(start: BasisStart) -> Interview {
    use crate::tests::cross::interview::accepted;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&super::income_pages::save_k1_sources_and_answers(
        &mut session,
        super::builders::d6_ptp_pages(start),
    ));
    session
}

// The existing source fixtures use public source editors and complete native
// requests. Care/MoreCredits open from Review; income pages use their normal
// topic route. No Form8379 answer or hidden value is fabricated here.
pub(in crate::tests) fn save_reopen_native_sources(
    draft: FederalDraft,
    saves: &[TaxSection],
    reopens: &[TaxSection],
) -> FederalDraft {
    save_reopen_checked_sources(draft, saves, reopens, supported)
}

pub(in crate::tests) fn save_reopen_checked_sources(
    draft: FederalDraft,
    saves: &[TaxSection],
    reopens: &[TaxSection],
    checked: impl Fn(&FederalDraft) -> FederalResult,
) -> FederalDraft {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    let draft = crate::tests::support::builders::known_fact_scorp_draft(draft);
    let original = checked(&draft);
    let draft = crate::tests::support::builders::known_fact_partnership_draft(draft);
    let mut session = restart(draft);
    admit_native_fixture_facts(&mut session);
    let open = |session: &mut Interview, topic| match topic {
        TaxSection::Documents => super::income_pages::open_document_center(session),
        TaxSection::OtherItemized => {
            crate::tests::cross::interview::open_itemized_topic(session, topic)
        }
        TaxSection::InjuredSpouse => complete_niit_allocation(session),
        TaxSection::Care => match from_review(session, topic) {
            reply @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
                crate::tests::cross::interview::care_editor(reply)
            }
            _ => panic!("expected the actual Care editor"),
        },
        TaxSection::MoreCredits => match from_review(session, topic) {
            Reply::Edit(page @ P::SaveMoreCredits { .. }) => page,
            _ => panic!("expected adoption credit editor"),
        },
        _ => open_topic(session, topic),
    };
    for &topic in saves {
        let request = open(&mut session, topic);
        accepted(&send(&mut session, roundtrip_native_request(request)));
    }
    checked(&session.draft);
    let saved = session.draft.clone();
    let draft = roundtrip_native_draft(saved.clone());
    let mut again = restart(draft);
    assert_eq!(again.draft, saved);
    assert_eq!(estimate(&again.draft).return_lines, original.return_lines);
    for &topic in reopens {
        let request = open(&mut again, topic);
        accepted(&send(&mut again, roundtrip_native_request(request)));
    }
    assert_eq!(again.draft, saved);
    let result = checked(&again.draft);
    assert_native_money_unchanged(&result, &original);
    again.draft
}

fn save_native_topic_records(session: &mut Interview, topic: TaxSection) {
    let request = open_topic(session, topic);
    let save = match &request {
        P::SaveWagesAnswers { .. } => true,
        P::SaveIncomes { documents, .. } => !documents.documents.is_empty(),
        _ => unreachable!(),
    };
    if save {
        accepted(&send(session, request));
    }
}

pub(super) fn save_native_income_topics(session: &mut Interview, topics: &[TaxSection]) {
    for &topic in topics {
        let request = if topic == TaxSection::Documents {
            super::income_pages::open_document_center(session)
        } else {
            open_topic(session, topic)
        };
        accepted(&send(session, request));
    }
}

pub(in crate::tests) fn save_native_credit_sources(session: &mut Interview, topics: &[TaxSection]) {
    admit_native_fixture_facts(session);
    save_native_income_topics(session, topics);
    let Reply::Edit(request @ P::SaveMoreCredits { .. }) =
        from_review(session, TaxSection::MoreCredits)
    else {
        panic!("expected native MoreCredits editor")
    };
    accepted(&send(session, request));
}

pub(super) fn save_native_review_topics(session: &mut Interview, topics: &[TaxSection]) {
    for &topic in topics {
        let Reply::Edit(request) = from_review(session, topic) else {
            panic!("expected native editor")
        };
        accepted(&send(session, request));
    }
}

pub(in crate::tests) fn check_wage_classification_source(
    case: WageClassificationCase,
    owner: Owner,
) {
    let original = wage_classification_fixture(case, owner);
    let (wages, others) = expected_wage_classification_columns(case, owner);
    let mut observed = vec![];
    let mut expected_columns = vec![];
    for reversed in [false, true] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = original.clone();
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reversed {
                draft.income.reverse();
            }
            let result = supported(&draft);
            assert_wage_classification_money(&result, case);
            let mut plain = draft.clone();
            plain.injured_spouse = None;
            let ordinary = supported(&plain);
            assert_native_money_unchanged(&result, &ordinary);
            assert_eq!(
                result
                    .lines
                    .iter()
                    .filter(|row| !row.id.starts_with("form_8379_"))
                    .collect::<Vec<_>>(),
                ordinary
                    .lines
                    .iter()
                    .filter(|row| !row.id.starts_with("form_8379_"))
                    .collect::<Vec<_>>()
            );
            let mut session = Interview::new(TEST_USER, draft.clone());
            admit_native_fixture_facts(&mut session);
            let request = super::income_pages::open_document_center(&mut session);
            accepted(&send(&mut session, request));
            assert_eq!(session.draft.income, draft.income);
            assert_wage_classification_money(&supported(&session.draft), case);
            let draft = session.draft.clone();
            let accepted_result = estimate(&draft);
            assert_native_money_unchanged(&accepted_result, &result);
            assert_eq!(accepted_result.return_lines, result.return_lines);
            let result = accepted_result;
            let request = super::income_pages::open_document_center(&mut session);
            accepted(&send(&mut session, request));
            assert_eq!(session.draft, draft);
            assert_eq!(estimate(&session.draft), result);
            let saved = roundtrip_native_draft(session.draft.clone());
            assert_eq!(saved, draft);
            let mut restarted = restart(saved);
            assert_eq!(estimate(&restarted.draft), result);
            let reopened = super::income_pages::open_document_center(&mut restarted);
            accepted(&send(&mut restarted, reopened));
            assert_eq!(restarted.draft, draft);
            assert_eq!(estimate(&restarted.draft), result);
            let i = usize::from(injured == Owner::Spouse);
            for (line, owners) in [("13a", wages), ("13b", others)] {
                observed.push((
                    reversed,
                    injured,
                    line,
                    ["a", "b", "c"].map(|col| niit_optional_column(&result, line, col)),
                ));
                expected_columns.push((
                    reversed,
                    injured,
                    line,
                    [Some(owners[0] + owners[1]), Some(owners[i]), Some(owners[1 - i])],
                ));
            }
        }
    }
    // Every ordinary native/asserted Save/restart case precedes owner checks.
    assert_eq!(observed, expected_columns, "{case:?}, source owner {owner:?}");
}

pub(in crate::tests) fn check_non_code_l_wage_source(owner: Owner) {
    // Independent pure-source control, not a revision of original Form2106F3:
    // no W-2 codeL; actual line2 expenses1900 and other reimbursements2000
    // produce line8A excess100. Native1a stays80100; the known separate
    // reimbursement source belongs to8379other100, not W-2 source80000.
    let (original, employer) = non_code_l_wage_fixture(owner);
    let assert_money = |result: &FederalResult| {
        assert_eq!(return_line(result, "1a"), 8_010_000);
        assert_eq!(return_line(result, "1d"), 0);
        assert_eq!(return_line(result, "8"), 0);
        assert_eq!(return_line(result, "9"), 8_010_000);
        assert_eq!(value(result, "agi"), 80_100);
        assert_eq!(value(result, "wages"), 80_100);
        assert_eq!(value(result, "taxable_income"), 47_900);
        assert_eq!(value(result, "employee_expenses_1_excess_reimbursement"), 100);
        assert_eq!(value(result, "self_employment_tax"), 0);
        // Tax Table47900–47950:2480+12%×(47925−24800)=5255.
        assert_eq!(return_line(result, "16"), 525_500);
        assert_eq!(result.tax, Some(525_500));
        assert_eq!(result.refund, Some(124_500));
    };
    let mut observed = vec![];
    let mut expected_columns = vec![];
    for reversed in [false, true] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = original.clone();
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reversed {
                draft.income.reverse();
            }
            let claim = &draft.adjustments.as_ref().unwrap().employee_expenses[0];
            assert_eq!(claim.owner, owner);
            assert_eq!(claim.employers, [employer]);
            let lines = claim.lines(&draft, false);
            assert_eq!(
                (lines.line2, lines.line7, lines.line8a, lines.excess, lines.line10),
                (190_000, 200_000, 0, 10_000, 0)
            );
            let result = supported(&draft);
            assert_money(&result);
            let mut plain = draft.clone();
            plain.injured_spouse = None;
            let ordinary = supported(&plain);
            assert_native_money_unchanged(&result, &ordinary);
            let mut session = Interview::new(TEST_USER, draft.clone());
            for topic in [TaxSection::Wages, TaxSection::EmployeeExpenses] {
                let request = if topic == TaxSection::Wages {
                    open_topic(&mut session, topic)
                } else {
                    match crate::tests::cross::interview::from_review(&mut session, topic) {
                        Reply::Edit(page @ P::SaveEmployeeExpenses { .. }) => page,
                        other => panic!("expected the Form2106 editor, got {other:?}"),
                    }
                };
                accepted(&send(&mut session, request));
                // The first save of an applying topic records the visit, and
                // nothing else changes.
                let mut expected = draft.clone();
                expected.finished_sections = session.draft.finished_sections.clone();
                expected.revision = session.draft.revision;
                expected.checklist = session.draft.checklist;
                assert_eq!(session.draft, expected);
                let mut visited = result.clone();
                visited.revision = session.draft.revision;
                assert_eq!(estimate(&session.draft), visited);
            }
            let saved = roundtrip_native_draft(session.draft.clone());
            assert_eq!(saved, session.draft);
            let mut restarted = restart(saved.clone());
            let mut visited = result.clone();
            visited.revision = saved.revision;
            assert_eq!(estimate(&restarted.draft), visited);
            for topic in [TaxSection::Wages, TaxSection::EmployeeExpenses] {
                let request = if topic == TaxSection::Wages {
                    open_topic(&mut restarted, topic)
                } else {
                    match crate::tests::cross::interview::from_review(&mut restarted, topic) {
                        Reply::Edit(page @ P::SaveEmployeeExpenses { .. }) => page,
                        other => panic!("expected the Form2106 editor, got {other:?}"),
                    }
                };
                accepted(&send(&mut restarted, request));
                // A visited topic's unchanged save is a no-op.
                assert_eq!(restarted.draft, saved);
                assert_eq!(estimate(&restarted.draft), visited);
            }
            let i = usize::from(injured == Owner::Spouse);
            let mut other = [0; 2];
            other[usize::from(owner == Owner::Spouse)] = 100;
            for (line, owners) in [("13a", [60_000, 20_000]), ("13b", other)] {
                observed.push((
                    reversed,
                    injured,
                    line,
                    ["a", "b", "c"].map(|col| niit_optional_column(&result, line, col)),
                ));
                expected_columns.push((
                    reversed,
                    injured,
                    line,
                    [Some(owners[0] + owners[1]), Some(owners[i]), Some(owners[1 - i])],
                ));
            }
        }
    }
    // Every native financial/publicSave/restart assertion precedes attribution.
    assert_eq!(observed, expected_columns, "non-code-L source owner {owner:?}");
}

pub(in crate::tests) fn save_noninvestment_election_source(d: FederalDraft) -> FederalDraft {
    use crate::tests::cross::interview::{accepted, open_topic};
    let mut session = Interview::new(TEST_USER, d);
    let request = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { documents, .. } = &request else { panic!() };
    assert!(
        documents
            .documents
            .iter()
            .any(|row| matches!(row.document, crate::IncomeRecord::InstallmentPayments(_))
                && row.owner == Owner::Taxpayer)
    );
    accepted(&send(&mut session, roundtrip_native_request(request)));
    assert_noninvestment_election_money(&session.draft);
    let draft = roundtrip_native_draft(session.draft.clone());
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let again = open_topic(&mut reopened, TaxSection::InvestmentSales);
    let before = reopened.draft.clone();
    accepted(&send(&mut reopened, again));
    assert_eq!(reopened.draft, before);
    assert_noninvestment_election_money(&reopened.draft);
    reopened.draft
}

pub(in crate::tests) fn d6_mixed_direct_sources(mut session: Interview) -> (Interview, i64, i64) {
    use crate::tests::cross::interview::{
        asset, business, cash_farm, save_businesses, stored_business,
    };
    use crate::tests::support::builders::{
        add, cooperative_routed_income, elected_asset, farm_land_sale, government_unemployment,
        income, section1231_history, set_land_sales,
    };
    use crate::{
        AssetClass, BonusElection, BusinessActivity, FederalIncome, FirstYearDeduction,
        NoteInterest, SalePayment,
    };
    use alloc::vec;
    let mut page = business(4_000_000);
    let mut property = asset(1_000_000, 20260601, AssetClass::Five);
    property.acquired = Some(20160928);
    property.gds_mut().unwrap().history = crate::AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic selected direct source".into(),
            elected: 300_000,
        },
    };
    page.assets.push(property);
    crate::tests::cross::interview::accepted(&save_businesses(&mut session, vec![page]));
    let (business, record) = stored_business(&session);
    let asset_id = record.assets[0].id;
    crate::tests::support::builders::add_simplified_office(&mut session.draft, business, 300);
    save_farms(&mut session, vec![cash_farm(1_000_000, 300_000)]);
    let farm = session
        .draft
        .income
        .iter()
        .find(|entry| {
            matches!(&entry.income, FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Farming { .. }))
        })
        .unwrap()
        .id;
    // Reuse Unit17's native source recipe. Normal Save assigned the C/F and
    // asset identities; these remaining native financial facts are not a UI
    // roundtrip claim (the page does not preserve this ElectOut combination).
    let FederalIncome::Business { data } =
        &mut session.draft.income.iter_mut().find(|entry| entry.id == business).unwrap().income
    else {
        unreachable!()
    };
    let mut native = elected_asset(10_000, 3_000, BonusElection::ElectOut);
    native.id = asset_id;
    native.acquired = 20160928;
    data.assets[0] = native;
    for id in [business, farm] {
        add(&mut session.draft, income(cooperative_routed_income(200_000, 50_000)));
        claim_last_cooperative(&mut session.draft, id);
    }
    let mut grant = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut grant.income else { unreachable!() };
    data.form.taxable_grants = Some(250_000);
    data.grant_use = None;
    data.form.agriculture_payments = Some(400_000);

    add(&mut session.draft, grant);
    // The grant belongs to the business; both it and the farm are eligible receivers.
    let FederalIncome::GovernmentPayment { data } = &session.draft.income.last().unwrap().income
    else {
        unreachable!()
    };
    let grant_component = data.component_ids.grant;
    crate::tests::support::builders::claim_related_component(
        &mut session.draft,
        business,
        grant_component,
    );
    let component = crate::tests::support::builders::farm_government_component(&session.draft);
    crate::tests::support::builders::relate_farm_component(&mut session.draft, farm, component);
    let FederalIncome::Business { data } =
        &mut session.draft.income.iter_mut().find(|entry| entry.id == farm).unwrap().income
    else {
        unreachable!()
    };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let mut sale = farm_land_sale(20210615, 20260616);
    sale.payment = SalePayment::Installment {
        assumed_debt: 0,
        payments: 7_500_000,
        interest: NoteInterest::Adequate,
        special: false,
    };
    set_land_sales(data, vec![sale], &mut session.draft.next_id);
    session.draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    assert!(session.draft.valid_stored());
    (restart(session.draft), business, farm)
}

pub(in crate::tests) fn d6_business_boundary_source(
    mut session: Interview,
    profit: i64,
    fully_at_risk: bool,
    depletion: Option<crate::BusinessDepletion>,
) -> (Interview, i64) {
    use crate::tests::cross::interview::{accepted, business, save_businesses, stored_business};
    use alloc::vec;
    let mut page = business(profit.max(0) * 100);
    if profit < 0 {
        page.expenses.other.push(crate::OtherExpense {
            description: "Synthetic operating costs".into(),
            amount: -profit * 100,
        });
    }
    page.more.at_risk = crate::RentalAtRisk::from_full_risk(fully_at_risk);
    page.more.depletion = depletion;
    accepted(&save_businesses(&mut session, vec![page]));
    let (id, _) = stored_business(&session);
    (session, id)
}

pub(in crate::tests) fn injured_income_source_roundtrip(
    d: FederalDraft,
    source_topic: TaxSection,
) -> FederalDraft {
    let mut session = restart(d);
    admit_native_fixture_facts(&mut session);
    save_native_income_topics(&mut session, &[TaxSection::Documents]);
    save_native_topic_records(&mut session, source_topic);
    let bytes = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &bytes);
    assert_eq!(native_draft_records(&draft), bytes);
    assert_eq!(draft, session.draft);
    let mut session = restart(draft);
    let page = super::income_pages::open_document_center(&mut session);
    accepted(&send(&mut session, page));
    save_native_topic_records(&mut session, source_topic);
    session.draft
}

pub(in crate::tests) fn save_ira_people(
    d: &FederalDraft,
    allowed_owners: Vec<i32>,
    people: Vec<IraContributor>,
) -> P {
    P::SaveIraContributions {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        previous: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        allowed_owners,
        value: IraContributions { people },
    }
}

pub(in crate::tests) fn save_educator(
    d: &FederalDraft,
    allowed_owners: Vec<i32>,
    value: EducatorExpenses,
) -> P {
    P::SaveEducator {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        previous: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        allowed_owners,
        value,
    }
}

pub(in crate::tests) fn save_student_loan(d: &FederalDraft, value: StudentLoanInterest) -> P {
    P::SaveStudentLoan {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        previous: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}

// Accept source facts and their native property classifier through ordinary topic Save.
pub(in crate::tests) fn current_installment_session(
    sale: crate::InvestmentInstallmentSale,
) -> Interview {
    current_installment_session_for(
        complete(FederalStatus::Single),
        sale,
        crate::Owner::Taxpayer,
        crate::CommunityShare::NotNeeded,
    )
}
pub(in crate::tests) fn current_installment_session_for(
    draft: FederalDraft,
    sale: crate::InvestmentInstallmentSale,
    owner: crate::Owner,
    community: crate::CommunityShare,
) -> Interview {
    use crate::tests::cross::interview::{document, save_topic};
    let mut session = Interview::new(TEST_USER, draft);
    let mut row =
        document(0, IncomeSource::Record { value: crate::IncomeRecord::InstallmentSale(sale) });
    row.owner = crate::DocumentOwner::from(owner);
    row.community = community;
    let request = save_topic(&mut session, TaxSection::InvestmentSales, vec![row]);
    accepted(&send(&mut session, roundtrip_native_request(request)));
    session
}

// Exercise source identity through the activity's actual native editor.
pub(in crate::tests) fn reject_unavailable_related_components(
    case: &str,
    source: IncomeSource,
    kinds: &[crate::related_forms::DocumentKind],
    cross_owner: bool,
) {
    use super::income_pages::save_documents;
    use crate::tests::cross::interview::{business, document, save_businesses};
    let miscellaneous = matches!(
        &source,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) }
    );
    let mut draft = empty_return(FederalStatus::Joint);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.first_name = "Alex".into();
    people.spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    accepted(&save_businesses(&mut session, vec![business(0)]));
    let mut row = document(0, source);
    if cross_owner {
        row.owner = crate::DocumentOwner::Spouses;
    }
    row.document.set_printed_names(&accepted_document_names(&session.draft, row.owner.owned()));
    let request = save_documents(&mut session, vec![row]);
    accepted(&send(&mut session, request));
    let keys = crate::related_forms::catalog(&session.draft)
        .into_iter()
        .filter(|component| kinds.contains(&component.kind))
        .map(|component| component.id)
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), kinds.len());
    if !cross_owner {
        let mut request = super::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        if miscellaneous {
            documents.other.other_income.documents.clear();
        } else {
            documents.common.government_payments.documents.clear();
        }
        accepted(&send(&mut session, request));
        let remaining = crate::related_forms::catalog(&session.draft);
        assert!(keys.iter().all(|key| remaining.iter().all(|source| source.id != *key)));
    }
    let before = session.draft.clone();
    let figures = estimate(&before);
    for key in keys {
        let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let business = documents
            .documents
            .iter_mut()
            .find_map(|row| match &mut row.document {
                crate::IncomeRecord::Business(business) => Some(business),
                _ => None,
            })
            .unwrap();
        business.related.forms.push(crate::RelatedFormView { key, summary: None });
        let scenario = alloc::format!("{case}-{key}");
        super::survey::survey_native_refusal(
            &scenario,
            &mut session,
            request,
            "PROBLEM_RELATED_FORM_UNAVAILABLE",
            "[documents/documents/0/document/Business/related/forms/0/key]",
        );
        assert_eq!(estimate(&session.draft), figures);
    }
    let page = open_topic(&mut session, TaxSection::SelfEmployment);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, before);
}

// Supply the fixture's independent disposition through the ordinary topic editor.
pub(in crate::tests) fn answer_only_retirement_treatment(
    session: &mut Interview,
    treatment: DistributionCode,
) -> Reply {
    let mut request = super::income_pages::open_topic(session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let mut answers = followups.iter_mut().filter_map(|row| match &mut row.answer {
        crate::DocumentAnswer::RetirementTreatment { code } => Some(code),
        _ => None,
    });
    let answer = answers.next().expect("one retirement source in this fixture");
    assert!(answers.next().is_none());
    *answer = Some(treatment.into());
    send(session, request)
}

// Preserve a retirement fixture's supplied answers through the real two editors.
// A failed topic Save is preflighted in a cold session so the caller stays atomic.
pub(in crate::tests) fn save_retirement_fixture(
    session: &mut Interview,
    rows: Vec<IncomeDocumentItem>,
) -> Reply {
    fn attempt(session: &mut Interview, rows: &[IncomeDocumentItem]) -> Reply {
        let (records, sources): (Vec<_>, Vec<_>) = rows
            .iter()
            .cloned()
            .partition(|row| matches!(row.document, IncomeSource::Record { .. }));
        if !sources.is_empty() {
            let request = super::income_pages::save_documents(session, sources.clone());
            let reply = send(session, request);
            if matches!(reply, Reply::Response(P::Problem { .. })) {
                return reply;
            }
        }
        let mut request = if sources.is_empty() {
            super::income_pages::open_topic(session, TaxSection::Retirement)
        } else {
            super::income_pages::information_return_answers_for_topic(
                session,
                &sources,
                TaxSection::Retirement,
            )
            .unwrap_or_else(|| super::income_pages::open_topic(session, TaxSection::Retirement))
        };
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        if !records.is_empty() {
            let supplied: Vec<_> = records
                .into_iter()
                .map(|row| crate::IncomeRecordItem::from_income(row).unwrap())
                .collect();
            documents.documents.retain(|accepted| {
                !supplied.iter().any(|row| {
                    if row.id > 0 {
                        row.id == accepted.id
                    } else {
                        row.owner == accepted.owner
                            && core::mem::discriminant(&row.document)
                                == core::mem::discriminant(&accepted.document)
                    }
                })
            });
            documents.documents.extend(supplied);
        }
        send(session, request)
    }
    {
        let mut candidate = restart(session.draft.clone());
        let reply = attempt(&mut candidate, &rows);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
    }
    attempt(session, &rows)
}

// An accepted pension election remains independently answerable after paper reclassification.
pub(in crate::tests) fn retirement_pso_ira_refusal(
    draft: &FederalDraft,
    desired: R1099Answered,
) -> Reply {
    let mut session = restart(draft.clone());
    let mut eligible = desired.clone();
    eligible.source_mut().form.ira = false;
    eligible.source_mut().form.trump = false;
    eligible.source_mut().form.box7 = crate::DistributionCodes::CODE_7.0;
    eligible.source_mut().code =
        Some(DistributionCode::Normal { what_happened: crate::Disposition::Kept });
    eligible.source_mut().taxable_method = Some(crate::TaxableMethod::BoxTwoA);
    let row = |id, source| {
        crate::tests::cross::interview::document(
            id,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(source)),
            },
        )
    };
    accepted(&save_retirement_fixture(&mut session, vec![row(0, eligible)]));
    let id = session.draft.screening.as_ref().unwrap().public_safety.last().unwrap().income_id;
    let request = super::income_pages::save_documents(&mut session, vec![row(id, desired.clone())]);
    accepted(&send(&mut session, request));
    assert!(session.draft.valid_stored());
    assert!(!session.draft.screening.as_ref().unwrap().public_safety.is_empty());
    let before = session.draft.clone();
    let reply = save_retirement_fixture(&mut session, vec![row(id, desired)]);
    assert_eq!(session.draft, before);
    reply
}
