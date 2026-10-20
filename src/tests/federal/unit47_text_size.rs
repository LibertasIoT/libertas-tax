// Ordinary String Size UTF-8 bytes; no normalization, source truncation or tax math.
use super::*;
use crate::tests::support::builders::survey::survey_ordinary_foreign_credit;

fn supplementary(bytes: usize) -> alloc::string::String {
    "😀".repeat(bytes / 4) + &"a".repeat(bytes % 4)
}

fn multibyte(bytes: usize) -> alloc::string::String {
    "界".repeat(bytes / 3) + &"a".repeat(bytes % 3)
}

fn first_sale(page: &mut P) -> &mut crate::K1Section179Sale {
    let P::SaveIncomes { documents, .. } = page else { panic!("normal PassThrough editor") };
    match &mut documents.documents[0].document {
        IncomeDocument::K1Partnership(source) => &mut source.more.section179_sales[0],
        IncomeDocument::K1Scorp(source) => &mut source.more.section179_sales[0],
        _ => panic!("actual entity source"),
    }
}

#[test]
fn unit47_k1_sale_long_text_save_reopen_restart_preserves_source() {
    for corporation in [false, true] {
        let mut sale = k1_sale_input_statement();
        sale.description = supplementary(400);
        let source = if corporation {
            let mut page = k1_1120s("Actual corporation source");
            page.more.section179_sales = vec![sale.clone()];
            page.more.section179_carryover = 200_000;
            page.more.section179_carryover_sold = 200_000;
            IncomeDocument::K1Scorp(page)
        } else {
            let mut page = k1_1065("Actual partnership source");
            page.more.section179_sales = vec![sale.clone()];
            page.more.section179_carryover = 200_000;
            page.more.section179_carryover_sold = 200_000;
            IncomeDocument::K1Partnership(page)
        };
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let request = save_topic(&original, TaxSection::PassThrough, vec![document(0, source)]);
        accepted(&send(&mut session, request));
        let baseline_tax = supported(&session.draft).tax;
        let mut page = open_topic(&mut session, TaxSection::PassThrough);
        let stored_sale = first_sale(&mut page).clone();
        assert!(stored_sale.id > 0);
        sale.id = stored_sale.id;
        assert_eq!(stored_sale, sale);
        let saved = session.draft.clone();
        accepted(&send(&mut session, page));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        assert_eq!(restored.draft, saved);
        let mut page = open_topic(&mut restored, TaxSection::PassThrough);
        assert_eq!(first_sale(&mut page), &sale);
        // Longer descriptions preserve the complete source and its tax result.
        first_sale(&mut page).description.push('a');
        sale.description.push('a');
        accepted(&send(&mut restored, page));
        assert_eq!(supported(&restored.draft).tax, baseline_tax);
        let changed = restored.draft.clone();
        let mut page = open_topic(&mut restored, TaxSection::PassThrough);
        assert_eq!(first_sale(&mut page), &sale);
        // Required nonempty text still refuses atomically at the same source.
        first_sale(&mut page).description.clear();
        let dirty = restored.dirty;
        rejected_with(
            send(&mut restored, page),
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        );
        assert_eq!(restored.draft, changed);
        assert_eq!(restored.dirty, dirty);
        assert_eq!(supported(&restored.draft).tax, baseline_tax);
        let mut again = open_topic(&mut restored, TaxSection::PassThrough);
        assert_eq!(first_sale(&mut again), &sale);
        accepted(&send(&mut restored, again));
        assert_eq!(restored.draft, changed);
        let mut cold = restart(changed.clone());
        let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(first_sale(&mut reopened), &sale);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, changed);
        assert_eq!(supported(&cold.draft).tax, baseline_tax);
    }
}

#[test]
fn unit47_business_and_payroll_long_text_preserves_save_and_restart() {
    let mut page = business(15_000_000);
    page.principal_business = supplementary(400);
    page.address = supplementary(400);
    page.more.payroll = BusinessPayroll::Yes {
        statements: vec![crate::PayrollW2 {
            label: supplementary(400),
            wages: 1_000_000,
            medicare: 1_000_000,
        }],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    assert_round_trip(&mut session, &[page]);
    let saved = session.draft.clone();
    let baseline_tax = supported(&saved).tax;
    for field in 0..3 {
        let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let IncomeDocument::Business(candidate) = &mut documents.documents[0].document else {
            unreachable!()
        };
        match field {
            0 => candidate.principal_business.push('a'),
            1 => candidate.address.push('a'),
            _ => {
                let BusinessPayroll::Yes { statements, .. } = &mut candidate.more.payroll else {
                    unreachable!()
                };
                statements[0].label.push('a');
            }
        }
        accepted(&send(&mut session, request));
        assert_eq!(supported(&session.draft).tax, baseline_tax);
    }
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft).tax, baseline_tax);
    let reopened = open_topic(&mut restored, TaxSection::SelfEmployment);
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_state_source_long_text_keeps_required_and_optional_semantics() {
    let mut investment = crate::InvestmentStateFields {
        payer_federal_id: supplementary(400),
        payer_address: supplementary(400),
        ..Default::default()
    };
    assert!(crate::IncomeStateSource::Investment(investment.clone()).valid());
    investment.payer_address.push('a');
    assert!(crate::IncomeStateSource::Investment(investment.clone()).valid());
    investment.payer_address.clear();
    investment.payer_federal_id.push('a');
    assert!(crate::IncomeStateSource::Investment(investment.clone()).valid());
    investment.payer_federal_id.clear();
    assert!(crate::IncomeStateSource::Investment(investment).valid());
    let mut wage = crate::W2StateFields {
        box14a: vec![crate::W2OtherItem {
            label: supplementary(400),
            value: crate::W2OtherValue::Text { text: supplementary(400) },
        }],
        ..Default::default()
    };
    assert!(crate::IncomeStateSource::Wage(wage.clone()).valid());
    let crate::W2OtherValue::Text { text } = &mut wage.box14a[0].value else { unreachable!() };
    text.push('a');
    assert!(crate::IncomeStateSource::Wage(wage.clone()).valid());
    wage.box14a[0].value = crate::W2OtherValue::Text { text: supplementary(400) };
    wage.box14a[0].label.push('a');
    assert!(crate::IncomeStateSource::Wage(wage.clone()).valid());
    wage.box14a[0].label.clear();
    assert!(!crate::IncomeStateSource::Wage(wage.clone()).valid());
    wage.box14a[0].label = supplementary(400);
    wage.box14a[0].value = crate::W2OtherValue::Text { text: "".into() };
    assert!(!crate::IncomeStateSource::Wage(wage).valid());
}

#[test]
fn unit47_optional_farm_reference_preserves_absence_and_long_text() {
    let mut facts = crate::FarmConservationFacts {
        plan: crate::FarmConservationPlan::None,
        work: crate::FarmConservationWork::QualifyingLandWork,
        land_farmed: false,
        plan_reference: None,
    };
    assert!(facts.valid());
    facts.plan_reference = Some(supplementary(4000));
    assert!(facts.valid());
    facts.plan_reference.as_mut().unwrap().push('a');
    assert!(facts.valid());
    facts.plan_reference = Some("".into());
    assert!(!facts.valid());
    facts.plan_reference = None;
    assert!(facts.valid());
}

#[test]
fn unit47_parent_source_long_utf8_labels_preserve_money_and_codec() {
    let mut parent = crate::ParentTax {
        status: FederalStatus::Joint,
        taxable: 10_000_000,
        gains: None,
        others: vec![crate::OtherChildTax { label: supplementary(400), net: 100_000, gains: None }],
        complete: true,
        ordinary: true,
        foreign_excluded: None,
    };
    assert_eq!(parent.others[0].label.len(), 400);
    assert!(parent.valid());
    assert_eq!(crate::ParentTax::from_avro(&parent.to_avro()).unwrap(), parent);
    parent.others[0].label.push('a');
    assert!(parent.valid());
    parent.others[0].label = multibyte(400);
    assert!(parent.valid());
    parent.others[0].label.push('a');
    assert!(parent.valid());
    parent.others[0].label = " ".into();
    assert!(!parent.valid());
}

#[test]
fn unit47_text_size_byte_guard_iso_keeps_nullable_corporation_and_source_limits() {
    let mut source = iso_exercise(&supplementary(80), 1.0, 1000, 11000);
    assert!(crate::extended_rules::valid_exercises(&[source.clone()]));
    source.corporation = Some(supplementary(80));
    assert!(crate::extended_rules::valid_exercises(&[source.clone()]));
    // 2026-10-15 D-13: arbitrary ISO text caps are removed; nullable and
    // meaningful names, source amounts, quantities and timing still apply.
    source.corporation.as_mut().unwrap().push('a');
    assert!(crate::extended_rules::valid_exercises(&[source.clone()]));
    source.corporation = None;
    source.label.push('a');
    assert!(crate::extended_rules::valid_exercises(&[source.clone()]));
    source.label = multibyte(80);
    assert!(crate::extended_rules::valid_exercises(&[source.clone()]));
    for boundary in 0..5 {
        let mut invalid = source.clone();
        match boundary {
            0 => invalid.label = " ".into(),
            1 => invalid.corporation = Some(" ".into()),
            2 => invalid.exercise_price = -1,
            3 => invalid.granted = 20260102,
            4 => invalid.sold = invalid.shares + 1.0,
            _ => unreachable!(),
        }
        assert!(!crate::extended_rules::valid_exercises(&[invalid]));
    }
}

#[test]
fn unit47_casualty_long_source_text_keeps_required_fields_and_codec() {
    let mut event = crate::PersonalCasualty {
        id: 0,
        label: supplementary(400),
        date: 20260703,
        declaration: crate::CasualtyDeclaration::Undeclared,
        zip: "".into(),
        properties: vec![crate::CasualtyProperty {
            id: 0,
            home_site: None,
            business_damage: None,
            label: supplementary(400),
            location: supplementary(400),
            acquisition: crate::CasualtyAcquisition::Purchased { date: 20200101 },
            basis: 1_000_000,
            before: 1_000_000,
            after: 0,
            reimbursement: 0,
            received: 0,
        }],
        source_complete: true,
    };
    assert!(event.valid());
    assert_eq!(crate::PersonalCasualty::from_avro(&event.to_avro()).unwrap(), event);
    for field in 0..3 {
        let mut candidate = event.clone();
        match field {
            0 => candidate.label.push('a'),
            1 => candidate.properties[0].label.push('a'),
            _ => candidate.properties[0].location.push('a'),
        }
        assert!(candidate.valid());
        match field {
            0 => candidate.label.clear(),
            1 => candidate.properties[0].label.clear(),
            _ => candidate.properties[0].location.clear(),
        }
        assert!(!candidate.valid());
    }
    event.label = multibyte(400);
    assert!(event.valid());
}

#[test]
fn unit47_income_labels_preserve_long_utf8_names_through_save_and_restart() {
    let mut source = business(15_000_000);
    source.name = supplementary(80);
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![source.clone()]));
    assert_round_trip(&mut session, &[source]);
    let saved = session.draft.clone();
    assert_eq!(
        saved
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
            .unwrap()
            .label,
        supplementary(80)
    );
    let tax = supported(&saved).tax;
    let mut restored = restart(saved.clone());
    let mut request = open_topic(&mut restored, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Business(candidate) = &mut documents.documents[0].document else {
        unreachable!()
    };
    // D-2026-10-15-13: source labels have no independent byte maximum.
    let label = supplementary(400);
    candidate.name = label.clone();
    accepted(&send(&mut restored, request));
    let source = saved
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    source_label_change(&saved, &restored.draft, source, &label);
    assert_eq!(supported(&restored.draft).tax, tax);
    let changed = restored.draft.clone();
    let mut cold = restart(changed.clone());
    let reopened = open_topic(&mut cold, TaxSection::SelfEmployment);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, changed);
    assert_eq!(supported(&cold.draft).tax, tax);
}

#[test]
fn unit47_foreign_source_labels_preserve_long_text_and_required_links() {
    let mut assets = crate::ForeignInvestmentAssets::TaxBook {
        assets: vec![crate::ForeignInvestmentAsset {
            label: supplementary(400),
            source: crate::InvestmentAssetSource::Foreign { source: supplementary(400) },
            basis: crate::InvestmentAssetBasis::Average {
                beginning: 100_000,
                ending: 100_000,
                amt_beginning: 100_000,
                amt_ending: 100_000,
            },
        }],
    };
    assert!(assets.valid());
    let crate::ForeignInvestmentAssets::TaxBook { assets: rows } = &mut assets else {
        unreachable!()
    };
    let crate::InvestmentAssetSource::Foreign { source } = &mut rows[0].source else {
        unreachable!()
    };
    source.push('a');
    assert!(assets.valid());
    // D13 removes arbitrary length limits without changing trim/nonempty.
    let crate::ForeignInvestmentAssets::TaxBook { assets: rows } = &mut assets else {
        unreachable!()
    };
    rows[0].label = " ".into();
    assert!(!assets.valid());
    let mut state = crate::ForeignStateTaxes::Records {
        payments: vec![crate::ForeignStatePayment {
            label: supplementary(400),
            paid: 100_000,
            domestic: 100_000,
            foreign: vec![crate::ForeignStateIncome {
                source: supplementary(400),
                income: 100_000,
            }],
        }],
        confirmed: true,
    };
    assert!(state.valid());
    let crate::ForeignStateTaxes::Records { payments, .. } = &mut state else { unreachable!() };
    payments[0].foreign[0].source.push('a');
    assert!(state.valid());
    let crate::ForeignStateTaxes::Records { payments, .. } = &mut state else { unreachable!() };
    payments[0].foreign[0].source = " ".into();
    assert!(!state.valid());
}

#[test]
fn unit47_farm_land_long_label_keeps_native_sale_resolution_and_lifecycle() {
    let mut draft = farmland_installment_return();
    let financial = supported(&draft);
    let label = supplementary(400);
    farmland_sales_mut(&mut draft)[0].label = label.clone();
    assert!(farmland_sales_mut(&mut draft)[0].valid());
    let mut resolved = farmland_resolved_sale(&draft, 0);
    assert_eq!(resolved.label, label);
    assert!(resolved.valid());
    assert_eq!(
        supported(&draft).tax.expect("supported farm tax"),
        financial.tax.expect("supported farm tax")
    );
    let mut session = restart(draft);
    let request = open_topic(&mut session, TaxSection::Farming);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let result = supported(&saved);
    let mut cold = restart(saved.clone());
    let request = open_topic(&mut cold, TaxSection::Farming);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), result);
    // Keep the distinct original minima: the submitted source trims, while
    // the resolved financial carrier only requires a nonempty label.
    farmland_sales_mut(&mut cold.draft)[0].label = " ".into();
    assert!(!farmland_sales_mut(&mut cold.draft)[0].valid());
    resolved.label = " ".into();
    assert!(resolved.valid());
    resolved.label.clear();
    assert!(!resolved.valid());
}

#[test]
fn unit47_prior_property_and_audit_long_source_text_preserves_tax() {
    let mut installments = crate::InstallmentPayments {
        property: supplementary(400),
        sold: 20250101,
        gross_profit: 100_000,
        contract_price: 200_000,
        earlier_payments: 0,
        payments: 50_000,
        kind: crate::PropertyCharacter::LongTermCapital,
        special: false,
        qbi: crate::InstallmentQbi::NotQualified,
    };
    assert!(installments.valid());
    installments.property.push('a');
    assert!(installments.valid());
    let mut missing = installments.clone();
    missing.property.clear();
    assert!(!missing.valid());
    let mut exchange = crate::RelatedExchange {
        property: supplementary(400),
        last_transfer: 20250101,
        disposed: 20260101,
        exception: crate::RelatedException::None,
        deferred: 100_000,
        kind: crate::PropertyCharacter::LongTermCapital,
        special: false,
    };
    assert!(exchange.valid());
    exchange.property.push('a');
    assert!(exchange.valid());
    let mut missing = exchange.clone();
    missing.property.clear();
    assert!(!missing.valid());
    let mut audit = bba_interest_increase();
    audit.source_id = supplementary(400);
    let accepted = audit.refigure().unwrap();
    audit.source_id.push('a');
    let longer = audit.refigure().unwrap();
    assert_eq!(longer.prior_tax, accepted.prior_tax);
    assert_eq!(longer.corrected_tax, accepted.corrected_tax);
    audit.source_id = " ".into();
    assert_eq!(audit.refigure().err().unwrap().0, "ISSUE_BBA_AUDIT_SOURCE_ID");
    audit.source_id = multibyte(400);
    let same = audit.refigure().unwrap();
    assert_eq!(same.prior_tax, accepted.prior_tax);
    assert_eq!(same.corrected_tax, accepted.corrected_tax);
}

#[test]
fn unit47_cooperative_source_text_preserves_long_identifiers_and_optional_notice() {
    // 2026-10-15, D-2026-10-15-13: the same 31-byte printed identifier and
    // 2,001-byte notice source remain valid after redundant native maxima removal.
    let mut facts = crate::CoopCreditFacts {
        payer_ein: supplementary(30),
        notice: Some(crate::CoopNoticeStatus::Received { source: supplementary(2000) }),
        ..Default::default()
    };
    assert!(facts.valid());
    facts.payer_ein.push('a');
    assert!(facts.valid());
    facts.payer_ein.clear();
    assert!(facts.valid());
    let Some(crate::CoopNoticeStatus::Received { source }) = &mut facts.notice else {
        unreachable!()
    };
    source.push('a');
    assert!(facts.valid());
    facts.notice = None;
    assert!(facts.valid());
}

#[test]
fn unit47_fuel_long_source_text_keeps_prefix_minimum_and_credit_rules() {
    let mut issuer = card_issuer_claim("UTF8", 1000);
    let lines = issuer.lines();
    // D13 removes length ceilings, preserving exact source references and
    // required text. Each sale keeps its original distinct reference prefix.
    for sale in &mut issuer.sales {
        sale.sale_reference.push_str(&supplementary(400));
        sale.buyer_name = supplementary(400);
        sale.buyer_address = supplementary(400);
        sale.card_account_reference = supplementary(400);
    }
    issuer.cc_registration_number = "CC".into();
    issuer.cc_registration_number.push_str(&supplementary(398));
    assert!(issuer.valid());
    issuer.cc_registration_number.push('a');
    assert!(issuer.valid());
    issuer.cc_registration_number = "XX".into();
    issuer.cc_registration_number.push_str(&supplementary(398));
    assert!(!issuer.valid());
    issuer.cc_registration_number = "CC".into();
    issuer.cc_registration_number.push_str(&multibyte(398));
    assert!(issuer.valid());
    assert_eq!(issuer.lines(), lines);
    issuer.cc_registration_number = "CC".into();
    assert!(!issuer.valid());
    let mut vendor = government_vendor_claim(1000);
    let credit = vendor.line_cents(244);
    vendor.sales[0].buyer_name = supplementary(400);
    vendor.uv_registration_number = "UV".into();
    vendor.uv_registration_number.push_str(&supplementary(398));
    assert!(vendor.valid(244));
    vendor.uv_registration_number.push('a');
    assert!(vendor.valid(244));
    assert_eq!(vendor.line_cents(244), credit);
    vendor.sales[0].buyer_name = " ".into();
    assert!(!vendor.valid(244));
}

fn country_claim(countries: &[&str]) -> ForeignTaxClaim {
    survey_ordinary_foreign_credit(
        countries
            .iter()
            .enumerate()
            .map(|(index, country)| ForeignOrdinaryTax {
                country: (*country).into(),
                owner: Owner::Taxpayer,
                label: alloc::format!("Actual country source {index}"),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::Below15,
                    qualified: 0,
                },
                income: 100_000,
                tax: 10_000,
            })
            .collect(),
    )
}

#[test]
fn unit47_text_size_country_size_keeps_the_independent_codepoint_format() {
    // These are intrinsic scalar/format controls, not an ISO-country oracle.
    for country in ["CA", "RIC", "éA"] {
        assert!(crate::form_1116::country_size_valid(country));
        assert_eq!(crate::form_1116::claim_rule(&country_claim(&[country]), &[]), None);
        assert!(crate::extended_rules::valid_credit(&country_claim(&[country])));
    }
    for country in ["界界", "😀a", "😀😀"] {
        assert!(!crate::form_1116::country_size_valid(country));
        assert!(!crate::extended_rules::valid_credit(&country_claim(&[country])));
    }
    // Two encoded bytes alone do not satisfy the separate two-codepoint format.
    assert!(crate::form_1116::country_size_valid("é"));
    assert_eq!(
        crate::form_1116::claim_rule(&country_claim(&["é"]), &[]).unwrap().key,
        "FOREIGN_COUNTRY_CODE"
    );
    assert_eq!("😀😀".chars().count(), 2);
    assert!(!crate::form_1116::country_size_valid("😀😀"));
    let claim = country_claim(&["😀😀"]);
    let issue = crate::form_1116::claim_rule(&claim, &[]).unwrap();
    assert_eq!(issue.key, "FOREIGN_COUNTRY_CODE");
    assert_eq!(issue.field.unwrap().square(), "[sources/0/country]");
    assert!(!crate::extended_rules::valid_credit(&claim));
    // Three ASCII bytes pass Size but fail the separate format unless RIC.
    assert!(crate::form_1116::country_size_valid("ABC"));
    assert_eq!(
        crate::form_1116::claim_rule(&country_claim(&["ABC"]), &[]).unwrap().key,
        "FOREIGN_COUNTRY_CODE"
    );
}

fn more_credit_estimate(page: &P) -> P {
    let P::SaveMoreCredits {
        value,
        allowed_owners,
        foreign_businesses,
        capital_sales,
        foreign_situations,
        mortgage_loans,
        businesses,
        farms,
        clean_vehicle_properties,
        rentals,
        research_businesses,
        entered_credits,
        carryback_credits,
        allowed_carry_filers,
        initial_carbon_oxide_origins,
        initial_renewable_facility_origins,
        initial_credit_elections,
        initial_research_choices,
        ..
    } = page
    else {
        unreachable!()
    };
    P::EstimateMoreCredits {
        user: TEST_USER,
        value: value.clone(),
        allowed_owners: allowed_owners.clone(),
        foreign_businesses: foreign_businesses.clone(),
        capital_sales: capital_sales.clone(),
        foreign_situations: foreign_situations.clone(),
        mortgage_loans: mortgage_loans.clone(),
        businesses: businesses.clone(),
        farms: farms.clone(),
        clean_vehicle_properties: clean_vehicle_properties.clone(),
        rentals: rentals.clone(),
        research_businesses: research_businesses.clone(),
        entered_credits: entered_credits.clone(),
        carryback_credits: carryback_credits.clone(),
        allowed_carry_filers: allowed_carry_filers.clone(),
        initial_carbon_oxide_origins: initial_carbon_oxide_origins.clone(),
        initial_renewable_facility_origins: initial_renewable_facility_origins.clone(),
        initial_credit_elections: initial_credit_elections.clone(),
        initial_research_choices: initial_research_choices.clone(),
    }
}

#[test]
fn unit47_foreign_sale_uniqueness_uses_native_ids_and_locates_the_duplicate_source() {
    let mut draft = complete(FederalStatus::Single);
    let mut sale_ids = vec![];
    for proceeds in [50_000, 60_000] {
        add(&mut draft, income(ordinary_investment_sale(proceeds, 30_000, GainTerm::Long)));
        sale_ids.push(draft.income.last().unwrap().id);
    }
    let set_sales = |claim: &mut ForeignTaxClaim, lists: &[Vec<i64>]| {
        let ForeignTaxClaim::Ordinary { sources, .. } = claim else { unreachable!() };
        for (source, values) in sources.iter_mut().zip(lists) {
            let ForeignTaxCategory::Passive { sales, .. } = &mut source.category else {
                unreachable!()
            };
            *sales = values.clone();
        }
    };
    let request = |draft: &FederalDraft, claim: ForeignTaxClaim| {
        save_more_credits(
            draft,
            MoreCredits {
                credits: vec![
                    MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed: Some(0) },
                    MoreCredit::ForeignTax(claim.try_into().unwrap()),
                ],
                credit_income_sources: vec![],
            },
        )
    };
    let mut session = Interview::new(TEST_USER, draft);
    let mut valid = country_claim(&["CA", "RIC"]);
    set_sales(&mut valid, &[vec![sale_ids[0]], vec![sale_ids[1]]]);
    let initial_request = request(&session.draft, valid.clone());
    accepted(&send(&mut session, initial_request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    for (lists, source_index, sale_index) in [
        (vec![vec![sale_ids[0], sale_ids[0]], vec![]], 0, 1),
        (vec![vec![sale_ids[0]], vec![sale_ids[0]]], 1, 0),
    ] {
        let mut invalid = valid.clone();
        set_sales(&mut invalid, &lists);
        let issue = crate::form_1116::claim_rule(&invalid, &[]).unwrap();
        assert_eq!(issue.key, "FOREIGN_SALE_TWICE");
        assert_eq!(
            issue.field.unwrap().square(),
            alloc::format!("[sources/{source_index}/category/sales/{sale_index}]")
        );
        let submitted = P::from_avro(&request(&saved, invalid.clone()).to_avro()).unwrap();
        for submitted in [more_credit_estimate(&submitted), submitted] {
            let dirty = session.dirty;
            let Reply::Response(P::Problem { message, field }) = send(&mut session, submitted)
            else {
                panic!("duplicate foreign sale must be refused by the native owner")
            };
            assert_eq!(message, libertas::libertas_formatted_text("FOREIGN_SALE_TWICE", &[]));
            assert_eq!(
                field,
                Some(alloc::format!(
                    "[value/credits/1/sources/{source_index}/category/sales/{sale_index}]"
                ))
            );
            assert_eq!(session.draft, saved);
            assert_eq!(session.dirty, dirty);
        }
    }
    // Empty sale selections in separate passive-income rows are valid; they
    // share no sale ID. Comparing the two List values was the removed client
    // clause's different condition.
    set_sales(&mut valid, &[vec![], vec![]]);
    assert_eq!(crate::form_1116::claim_rule(&valid, &[]), None);
    accepted(&send(&mut session, request(&saved, valid.clone())));
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_credits_topic(&mut restored, TaxSection::MoreCredits);
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
    assert_eq!(restored.draft.credits.as_ref().unwrap().foreign_tax, valid);
}

#[test]
fn unit47_text_size_country_forged_avro_save_and_estimate_locate_actual_source_atomically() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let claim = country_claim(&["CA", "RIC"]);
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(claim.clone().try_into().unwrap())],
            credit_income_sources: vec![],
        },
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, claim);
    assert!(session.draft.valid_stored());
    let saved = session.draft.clone();
    let bad = country_claim(&["CA", "😀😀"]);
    let request = save_more_credits(
        &saved,
        MoreCredits {
            credits: vec![
                MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed: Some(0) },
                MoreCredit::ForeignTax(bad.clone().try_into().unwrap()),
            ],
            credit_income_sources: vec![],
        },
    );
    // Ordinary static Size blocks this in a client; a complete forged Avro
    // request must independently fail the native source owner at the real row.
    let forged = P::from_avro(&request.to_avro()).unwrap();
    for request in [more_credit_estimate(&forged), forged] {
        let dirty = session.dirty;
        let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
            panic!("native country refusal")
        };
        assert_eq!(message, libertas::libertas_formatted_text("FOREIGN_COUNTRY_CODE", &[]));
        assert_eq!(field.as_deref(), Some("[value/credits/1/sources/1/country]"));
        assert_eq!(session.draft, saved);
        assert_eq!(session.dirty, dirty);
    }
    let mut malformed = saved.clone();
    malformed.credits.as_mut().unwrap().foreign_tax = bad.clone();
    assert!(!malformed.valid_stored());
    let result = estimate(&malformed);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.tax.is_none());
    // Reopen/resave/restart keeps the valid original source and exact strings.
    let mut restored = restart(saved.clone());
    let reopened = open_credits_topic(&mut restored, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &reopened else { unreachable!() };
    assert!(value.credits.iter().any(|credit|
        matches!(credit, MoreCredit::ForeignTax(page) if ForeignTaxClaim::from(page.clone()) == claim)));
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
    assert_eq!(restored.draft.credits.as_ref().unwrap().foreign_tax, claim);
}

// Reuse the existing immutable native client bridge and its ordinary endpoint.
pub(crate) fn country_fixture() -> (Interview, P, P) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let mut guide = seed.clone();
    let P::SaveMoreCredits { value, .. } = &mut guide else { unreachable!() };
    value.credits = vec![
        MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed: Some(0) },
        MoreCredit::ForeignTax(country_claim(&["CA", "RIC"]).try_into().unwrap()),
    ];
    (session, seed, guide)
}
pub(crate) fn country_reopened(session: &mut Interview) -> P {
    open_credits_topic(session, TaxSection::MoreCredits)
}
pub(crate) fn assert_country_source(session: &Interview) {
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.foreign_tax, country_claim(&["CA", "RIC"]));
    assert_eq!(credits.legacy_ev_passive_allowed, Some(0));
}

#[test]
fn unit47_rental_labels_preserve_long_utf8_addresses_through_save_and_restart() {
    for bytes in [81, 120] {
        let mut source = rental_page(1_200_000);
        source.address = supplementary(bytes);
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save_rentals(&mut session, vec![source.clone()]));
        let saved = session.draft.clone();
        let tax = supported(&saved).tax;
        assert_eq!(stored_rentals(&session)[0].1, source.address);
        let mut reopened = open_topic(&mut session, TaxSection::Rentals);
        let P::SaveIncomes { documents, .. } = &mut reopened else { unreachable!() };
        let IncomeDocument::Rental(actual) = &documents.documents[0].document else {
            unreachable!()
        };
        assert_eq!(actual.address, source.address);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let mut request = open_topic(&mut restored, TaxSection::Rentals);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let IncomeDocument::Rental(actual) = &mut documents.documents[0].document else {
            unreachable!()
        };
        assert_eq!(actual.address, source.address);
        // D-2026-10-15-13: the source label carries the full address.
        let label = supplementary(400);
        actual.address = label.clone();
        accepted(&send(&mut restored, request));
        let id = saved
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
            .unwrap()
            .id;
        source_label_change(&saved, &restored.draft, id, &label);
        assert_eq!(stored_rentals(&restored)[0].1, label);
        assert_eq!(supported(&restored.draft).tax, tax);
        let changed = restored.draft.clone();
        let mut cold = restart(changed.clone());
        let reopened = open_topic(&mut cold, TaxSection::Rentals);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, changed);
        assert_eq!(supported(&cold.draft).tax, tax);
    }
}
