use super::lifecycle::{TEST_USER, restart, send};
use crate::interview::Interview;
use crate::tests::cross::interview::{accepted, from_review, open_topic};
use crate::tests::support::assert::assert_native_cash_money;
use crate::tests::support::builders::complete;
use crate::tests::*;
use crate::{
    BasisStart, Farm, FederalDraft, FederalResult, FederalStatus, IncomeDocument,
    TaxInterviewProtocol as P, TaxSection,
};
use alloc::vec::Vec;

// Bind Form8815 to the assigned ID from the accepted 1099-INT source.
pub(in crate::tests) fn education_bond_session(mut interest: Int1099, amount: i64) -> Interview {
    use crate::tests::cross::interview::{document, page_documents, save_topic};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    interest.taxable = 0;
    interest.more = IntMore { us_obligations: 300_000, ..Default::default() };
    interest.adjustments =
        Some(InterestAdjustments { previously_reported: 100_000, ..Default::default() });
    let request = save_topic(
        &session.draft,
        TaxSection::InterestDividends,
        vec![document(0, IncomeDocument::Int1099(interest))],
    );
    accepted(&send(&mut session, request));
    let source =
        page_documents(&open_topic(&mut session, TaxSection::InterestDividends))[0].clone();
    let bonds = IncomeDocument::SavingsBonds(SavingsBonds {
        proceeds: 1_200_000,
        redemption_interest: amount,
        earliest_issue_date: 20100601,
        interest_statement: source.id,
        students: vec![SavingsBondStudent {
            person: -1,
            expenses: SavingsBondExpenses::Entered { tuition: 1_400_000, benefits: 0 },
            contributions: 0,
        }],
    });
    let request =
        save_topic(&session.draft, TaxSection::InterestDividends, vec![source, document(0, bonds)]);
    accepted(&send(&mut session, request));
    session
}

pub(in crate::tests) fn estimate_documents_from_save(request: &P) -> P {
    let P::SaveIncomes {
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

pub(in crate::tests) fn save_native_cash_sources(
    d: FederalDraft,
    topic: TaxSection,
    categories: [i64; 3],
    agi: i64,
) -> FederalResult {
    let before = assert_native_cash_money(&d, categories, agi);
    let mut session = restart(d);
    let page = crate::tests::cross::interview::open_topic(&mut session, topic);
    crate::tests::cross::interview::accepted(&send(&mut session, page));
    let persisted = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &persisted);
    assert_eq!(native_draft_records(&draft), persisted);
    assert_eq!(draft, session.draft);
    let mut session = restart(draft);
    let page = crate::tests::cross::interview::open_topic(&mut session, topic);
    crate::tests::cross::interview::accepted(&send(&mut session, page));
    let after = assert_native_cash_money(&session.draft, categories, agi);
    assert_eq!(before.return_lines, after.return_lines);
    assert_eq!(before.withholding_sources, after.withholding_sources);
    after
}

pub(crate) fn save_farms(session: &mut Interview, farms: Vec<Farm>) {
    let request = crate::tests::cross::interview::save_topic(
        &session.draft,
        TaxSection::Farming,
        farms
            .into_iter()
            .map(|farm| crate::tests::cross::interview::document(0, IncomeDocument::Farm(farm)))
            .collect(),
    );
    crate::tests::cross::interview::accepted(&send(session, request));
    assert!(session.draft.valid_stored());
}

pub(in crate::tests) fn d6_ptp_source_session(start: BasisStart) -> Interview {
    use crate::tests::cross::interview::{accepted, open_topic};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    documents.documents = super::builders::d6_ptp_pages(start);
    accepted(&send(&mut session, request));
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
    let original = checked(&draft);
    let mut session = restart(draft);
    let open = |session: &mut Interview, topic| match topic {
        TaxSection::InjuredSpouse => complete_niit_allocation(session),
        TaxSection::Care => match from_review(session, topic) {
            Reply::Edit(page @ P::SaveCare { .. }) => page,
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

pub(super) fn save_native_income_topics(session: &mut Interview, topics: &[TaxSection]) {
    for &topic in topics {
        let request = open_topic(session, topic);
        accepted(&send(session, request));
    }
}

pub(in crate::tests) fn save_native_credit_sources(session: &mut Interview, topics: &[TaxSection]) {
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
            let request = open_topic(&mut session, TaxSection::Wages);
            accepted(&send(&mut session, request));
            assert_eq!(session.draft, draft);
            assert_eq!(estimate(&session.draft), result);
            let saved = roundtrip_native_draft(session.draft.clone());
            assert_eq!(saved, draft);
            let mut restarted = restart(saved);
            assert_eq!(estimate(&restarted.draft), result);
            let reopened = open_topic(&mut restarted, TaxSection::Wages);
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
                assert_eq!(session.draft, draft);
                assert_eq!(estimate(&session.draft), result);
            }
            let saved = roundtrip_native_draft(session.draft.clone());
            assert_eq!(saved, draft);
            let mut restarted = restart(saved);
            assert_eq!(estimate(&restarted.draft), result);
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
                assert_eq!(restarted.draft, draft);
                assert_eq!(estimate(&restarted.draft), result);
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
            .any(|row| matches!(row.document, IncomeDocument::InstallmentPayments(_))
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
        AssetClass, BonusElection, BusinessActivity, FederalGrantUse, FederalIncome,
        FirstYearDeduction, NoteInterest, SalePayment,
    };
    use alloc::vec;
    let mut page = business(4_000_000);
    let mut property = asset(1_000_000, 20260601, AssetClass::Five);
    property.acquired = Some(20160928);
    property.history = crate::AssetHistory::New {
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
        add(&mut session.draft, income(cooperative_routed_income(id, 200_000, 50_000)));
    }
    let mut grant = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut grant.income else { unreachable!() };
    data.taxable_grants = 250_000;
    data.grant_use = Some(FederalGrantUse::Business { business: Some(business) });
    data.agriculture_payment = 400_000;
    data.agriculture_farm = Some(farm);
    add(&mut session.draft, grant);
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
    save_native_income_topics(
        &mut session,
        &[TaxSection::Wages, TaxSection::InterestDividends, source_topic],
    );
    let bytes = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &bytes);
    assert_eq!(native_draft_records(&draft), bytes);
    assert_eq!(draft, session.draft);
    let mut session = restart(draft);
    let page = open_topic(&mut session, source_topic);
    accepted(&send(&mut session, page));
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
        estimate: vec![],
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
        estimate: vec![],
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
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
