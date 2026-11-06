use super::form_1116::{claim, no_carryback};
use super::*;
use crate::tests::cross::interview::{accepted, open_credits_topic, save_k1s, save_more_credits};
use crate::tests::support::client::income_pages::open_document_center;

#[test]
fn estate_foreign_existing_form_1116_receives_actual_regular_and_amt_country_amounts_once() {
    let d = estate_foreign_draft();
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_passive_FR_gross_income"), 100);
    assert_eq!(value(&r, "foreign_passive_UK_gross_income"), 50);
    assert_eq!(value(&r, "foreign_amt_passive_FR_gross_income"), 40);
    assert_eq!(value(&r, "foreign_amt_passive_UK_gross_income"), 140);
    let mut without = d.clone();
    without.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    assert_eq!(value(&r, "agi"), value(&supported(&without), "agi"));
    // Each category's U.S. limitation exceeds the $10 source tax: all of
    // the fiduciary's $7.01 + $2.99 is creditable, rounded once as $10.
    assert_eq!(value(&r, "foreign_tax_credit"), 10);
    assert_eq!(crate::estate_foreign::sources(&d, false).iter().map(|s| s.tax).sum::<i64>(), 1_000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn estate_foreign_country_half_cents_reconcile_to_actual_parent_income() {
    let mut d = estate_foreign_draft();
    let mut source = estate_foreign_data(&mut d);
    source.interest = 100;
    source.ordinary_dividends = 0;
    source.qualified_dividends = 0;
    source.amt_box12a = 0;
    source.amt_box12b = 0;
    source.foreign_statement.as_mut().unwrap().tax = 2;
    drop(source);
    for row in estate_foreign_allocations(&mut d) {
        row.withholding = ForeignWithholding::None;
        row.tax = 1;
        row.regular = estate_foreign_amounts(50, 0, 0);
        row.alternative = estate_foreign_amounts(50, 0, 0);
    }
    for alternative in [false, true] {
        let sources = crate::estate_foreign::sources(&d, alternative);
        assert_eq!(sources.len(), 2);
        assert!(sources.iter().all(|source| source.income == 50 && source.tax == 1));
        assert_eq!(sources.iter().map(|source| source.income).sum::<i64>(), 100);
    }
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 60_001);
    estate_foreign_allocations(&mut d).reverse();
    assert_eq!(supported(&d), result);
}

#[test]
fn estate_foreign_unknown_expenses_amt_and_timing_are_distinct_located_boundaries() {
    for (case, warning) in [
        (0, "WARN_ESTATE_FOREIGN_FACTS_LEFT_OUT"),
        (1, "WARN_ESTATE_FOREIGN_EXPENSES_LEFT_OUT"),
        (2, "WARN_ESTATE_FOREIGN_AMT_LEFT_OUT"),
        (3, "WARN_ESTATE_FOREIGN_BASIS_LEFT_OUT"),
        (4, "WARN_ESTATE_FOREIGN_SOURCE_YEAR_LEFT_OUT"),
    ] {
        let mut d = estate_foreign_draft();
        match case {
            0 => {
                estate_foreign_allocations(&mut d)[0].alternative.interest =
                    SourceCents::Unavailable
            }
            1 => {
                estate_foreign_allocations(&mut d)[0].regular.direct_expenses =
                    SourceCents::Reported { amount: 1 }
            }
            2 => {
                estate_foreign_data(&mut d).amt_reconciliation = None;
                let mut qualified = d.income[1].clone();
                qualified.id = 3;
                if let FederalIncome::EstateTrustPortfolio { data } = &mut qualified.income {
                    let mut data = edit_estate(data);
                    data.foreign_statement = Some(EstateForeignStatement {
                        tax: 10,
                        treatment: EstateForeignTreatment::QualifiedPassivePayeeStatement,
                    });
                }
                d.income.push(qualified);
                d.next_id = 4;
            }
            3 => {
                if let ForeignTaxClaim::Ordinary { basis, .. } =
                    &mut d.credits.as_mut().unwrap().foreign_tax
                {
                    *basis = ForeignTaxBasis::Accrued;
                }
            }
            _ => {
                if let ForeignTaxClaim::Ordinary { sources, .. } =
                    &mut d.credits.as_mut().unwrap().foreign_tax
                {
                    sources.push(ForeignOrdinaryTax {
                        label: "Earlier passive income".into(),
                        country: "FR".into(),
                        owner: Owner::Taxpayer,
                        category: ForeignTaxCategory::PriorPassiveTax { income_year: 2025 },
                        income: 0,
                        tax: 100,
                    });
                }
            }
        }
        let saved = d.clone();
        let r = supported(&d);
        assert!(noted(&r, warning), "case {case}: {r:?}");
        assert!(crate::estate_foreign::sources(&d, false).is_empty());
        assert_eq!(d, saved);
        assert_eq!(restart(d).draft, saved);
    }
}

#[test]
fn estate_foreign_typed_membership_never_retargets_a_colliding_direct_label() {
    let d = estate_foreign_draft();
    let estate = crate::estate_foreign::sources(&d, false);
    let reference = ForeignAllocationSource::Estate {
        parent: 2,
        country: "FR".into(),
        withholding: ForeignWithholding::None,
    };
    let direct = crate::estate_foreign::ForeignCatalogSource::direct(&estate[0].value);
    let direct_reference = ForeignAllocationSource::Direct { label: estate[0].label.clone() };
    assert!(direct_reference.matches(&direct));
    assert!(!direct_reference.matches(&estate[0]));
    assert!(reference.matches(&estate[0]));
    assert!(!reference.matches(&direct));
    let mut removed = d.clone();
    removed.income.remove(1);
    assert!(crate::estate_foreign::sources(&removed, false).iter().all(|s| !reference.matches(s)));
    let mut wrong_class = reference;
    if let ForeignAllocationSource::Estate { withholding, .. } = &mut wrong_class {
        *withholding = ForeignWithholding::Below15;
    }
    assert!(estate.iter().all(|s| !wrong_class.matches(s)));
}

#[test]
fn estate_foreign_conservative_investment_interest_uses_each_systems_actual_groups() {
    let d = estate_foreign_draft();
    let reg = crate::estate_foreign::sources(&d, false);
    let amt = crate::estate_foreign::sources(&d, true);
    let mut weights = [[0; crate::form_1116::SLOTS]; 2];
    weights[0][4] = 100;
    weights[0][1] = 50;
    weights[1][4] = 40;
    weights[1][1] = 140;
    let mut r = FederalResult::empty(0);
    let allocated =
        ForeignInvestmentAssets::Unreviewed.allocate([90, 90], [&reg, &amt], 0, weights, &mut r);
    assert_eq!([allocated[0][4], allocated[0][1]], [60, 30]);
    assert_eq!([allocated[1][4], allocated[1][1]], [20, 70]);
    assert!(noted(&r, "WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED"));
    weights[1] = [0; crate::form_1116::SLOTS];
    weights[1][crate::form_1116::GENERAL] = 180;
    let mut only_regular = FederalResult::empty(0);
    let allocated = ForeignInvestmentAssets::Unreviewed.allocate(
        [90, 90],
        [&reg, &amt],
        0,
        weights,
        &mut only_regular,
    );
    assert_eq!([allocated[0][4], allocated[0][1]], [60, 30]);
    assert_eq!(allocated[1], [0; crate::form_1116::SLOTS]);
}

#[test]
fn estate_foreign_taxbook_and_state_worksheets_resolve_the_same_typed_source() {
    let d = estate_foreign_draft();
    let reg = crate::estate_foreign::sources(&d, false);
    let amt = crate::estate_foreign::sources(&d, true);
    let reference = ForeignAllocationSource::Estate {
        parent: 2,
        country: "FR".into(),
        withholding: ForeignWithholding::None,
    };
    let assets = ForeignInvestmentAssets::TaxBook {
        assets: vec![ForeignInvestmentAsset {
            label: "Synthetic fiduciary investment".into(),
            source: InvestmentAssetSource::Foreign { source: reference.clone() },
            basis: InvestmentAssetBasis::Average {
                beginning: 10_000,
                ending: 10_000,
                amt_beginning: 20_000,
                amt_ending: 20_000,
            },
        }],
    };
    let mut r = FederalResult::empty(0);
    let allocated =
        assets.allocate([11, 13], [&reg, &amt], 0, [[0; crate::form_1116::SLOTS]; 2], &mut r);
    assert_eq!([allocated[0][4], allocated[1][4]], [11, 13]);
    let state = ForeignStateTaxes::Records {
        confirmed: true,
        payments: vec![ForeignStatePayment {
            label: "Synthetic state allocation".into(),
            paid: 1_000,
            domestic: 300,
            foreign: vec![ForeignStateIncome { source: reference, income: 100 }],
        }],
    };
    assert_eq!(state.allocate(1_000, 100, &reg, &mut r)[4], 25);
    assert!(r.issues.is_empty(), "{r:?}");
}

#[test]
fn estate_foreign_native_parent_picker_correction_atomic_rejection_and_restart() {
    let raw = estate_foreign_source();
    let page = K1EstateTrust::from_record("Synthetic fiduciary", &raw).unwrap();
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::EstateTrustPortfolio { .. }))
        .unwrap()
        .id;
    let request = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { estate_foreign_sources, .. } = request else { unreachable!() };
    assert_eq!(estate_foreign_sources.len(), 1);
    assert_eq!(estate_foreign_sources[0].id, id);
    let reference = ForeignAllocationSource::Estate {
        parent: id,
        country: "FR".into(),
        withholding: ForeignWithholding::None,
    };
    let mut foreign_claim = claim(vec![], ForeignCreditHistory::None, no_carryback());
    if let ForeignTaxClaim::Ordinary { investment_assets, .. } = &mut foreign_claim {
        *investment_assets = ForeignInvestmentAssets::TaxBook {
            assets: vec![ForeignInvestmentAsset {
                label: "Synthetic fiduciary asset".into(),
                source: InvestmentAssetSource::Foreign { source: reference.clone() },
                basis: InvestmentAssetBasis::Average {
                    beginning: 10_000,
                    ending: 10_000,
                    amt_beginning: 10_000,
                    amt_ending: 10_000,
                },
            }],
        };
    }
    let saved_claim = foreign_claim.clone();
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            answers: Vec::new(),
            documents: vec![],
            credits: vec![MoreCredit::ForeignTax(ForeignTaxPage::try_from(foreign_claim).unwrap())],
        },
    );
    accepted(&send(&mut session, request));
    let _before = session.draft.clone();
    let reopened = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value: credits, .. } = reopened else { unreachable!() };
    let page = credits
        .credits
        .iter()
        .find_map(
            |item| if let MoreCredit::ForeignTax(page) = item { Some(page.clone()) } else { None },
        )
        .unwrap();
    assert_eq!(ForeignTaxClaim::from(page), saved_claim);
    // The fiduciary's attached statement is edited with its printed K-1.
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    for entry in &mut documents.common.k1.documents {
        if let K1Document::K1EstateTrust(document) = &mut entry.document {
            let statement = document.statements.foreign_statement.as_mut().unwrap();
            statement.tax += 1;
            let EstateForeignTreatment::PassivePortfolio { allocations, .. } =
                &mut statement.treatment
            else {
                unreachable!()
            };
            allocations[0].tax += 1;
            allocations.reverse();
        }
    }
    accepted(&send(&mut session, request));
    assert_eq!(
        crate::estate_foreign::sources(&session.draft, false).iter().map(|s| s.tax).sum::<i64>(),
        1_001
    );
    assert_eq!(crate::estate_foreign::choices(&session.draft)[0].id, id);
    assert!(
        crate::estate_foreign::sources(&session.draft, false)
            .iter()
            .any(|source| reference.matches(source))
    );
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, saved_claim);
    let after = session.draft.clone();
    let mut cold = restart(after.clone());
    open_document_center(&mut cold);
    assert_eq!(cold.draft, after);
    assert_eq!(supported(&cold.draft), supported(&session.draft));
}

#[test]
fn estate_foreign_community_projection_conserves_parent_cents_across_row_reorder() {
    let mut d = estate_foreign_draft();
    d.setup = complete(FederalStatus::Separate).setup;
    d.income[0].community = CommunityShare::Community;
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    d.income[1].community = CommunityShare::Community;
    let mut reordered = d.clone();
    estate_foreign_allocations(&mut reordered).reverse();
    for amt in [false, true] {
        let actual = crate::estate_foreign::sources(&d, amt);
        let reordered = crate::estate_foreign::sources(&reordered, amt);
        assert_eq!(actual.iter().map(|s| s.tax).sum::<i64>(), 500);
        assert_eq!(actual.iter().map(|s| s.income).sum::<i64>(), if amt { 9_002 } else { 7_502 });
        for source in &actual {
            let other = reordered.iter().find(|s| source.reference.matches(s)).unwrap();
            assert_eq!(source.value, other.value);
        }
    }
}

#[test]
fn estate_foreign_paid_deduction_and_accrued_credit_keep_actual_source_treatment() {
    let mut d = estate_foreign_draft();
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction { taxes: vec![] };
    assert_eq!(crate::extended_rules::payee_document_taxes(&d), 1_000);
    let mut incomplete_credit = d.clone();
    estate_foreign_data(&mut incomplete_credit).amt_reconciliation = None;
    estate_foreign_allocations(&mut incomplete_credit)[0].alternative.interest =
        SourceCents::Unavailable;
    assert_eq!(crate::extended_rules::payee_document_taxes(&incomplete_credit), 1_000);
    let paid = supported(&incomplete_credit);
    assert!(!noted(&paid, "WARN_ESTATE_FOREIGN_AMT_LEFT_OUT"));
    assert!(!noted(&paid, "WARN_ESTATE_FOREIGN_EXPENSES_LEFT_OUT"));

    let mut estate = estate_foreign_data(&mut d);
    let EstateForeignTreatment::PassivePortfolio { basis, .. } =
        &mut estate.foreign_statement.as_mut().unwrap().treatment
    else {
        unreachable!()
    };
    *basis = ForeignTaxBasis::Accrued;
    drop(estate);
    assert_eq!(crate::extended_rules::payee_document_taxes(&d), 0);
    assert!(noted(&supported(&d), "WARN_ESTATE_FOREIGN_BASIS_LEFT_OUT"));
    let mut ordinary = claim(vec![], ForeignCreditHistory::None, no_carryback());
    if let ForeignTaxClaim::Ordinary { basis, .. } = &mut ordinary {
        *basis = ForeignTaxBasis::Accrued;
    }
    d.credits.as_mut().unwrap().foreign_tax = ordinary;
    assert_eq!(crate::estate_foreign::sources(&d, false).iter().map(|s| s.tax).sum::<i64>(), 1_000);
    assert!(!noted(&supported(&d), "WARN_ESTATE_FOREIGN_BASIS_LEFT_OUT"));
}

#[test]
fn estate_foreign_two_parent_sources_keep_actual_taxpayer_and_spouse_ownership() {
    let mut d = estate_foreign_draft();
    d.setup = complete(FederalStatus::Joint).setup;
    d.people = complete(FederalStatus::Joint).people;
    d.income[1].owner = crate::DocumentOwner::Spouses;
    let mut second = d.income[1].clone();
    second.id = 3;
    second.owner = crate::DocumentOwner::Yours;
    d.income.push(second);
    d.next_id = 4;
    let before = crate::estate_foreign::sources(&d, false);
    let taxes = |sources: &[crate::estate_foreign::ForeignCatalogSource], owner| {
        sources.iter().filter(|s| s.owner == owner).map(|s| s.tax).sum::<i64>()
    };
    assert_eq!(taxes(&before, Owner::Taxpayer), 1_000);
    assert_eq!(taxes(&before, Owner::Spouse), 1_000);
    d.income.reverse();
    let after = crate::estate_foreign::sources(&d, false);
    for source in before {
        assert_eq!(source.value, after.iter().find(|s| source.reference.matches(s)).unwrap().value);
    }
}

#[test]
fn estate_foreign_reported_capacities_and_overflow_never_become_unknown_zero() {
    let mut d = estate_foreign_draft();
    estate_foreign_allocations(&mut d)[0].regular.interest =
        SourceCents::Reported { amount: 10_002 };
    assert_eq!(
        crate::estate_foreign::source_issue(&estate_foreign_data(&mut d)).unwrap().key,
        "ESTATE_FOREIGN_INCOME_TOTAL_INVALID"
    );
    let r = estimate(&d);
    assert!(r.tax.is_none());
    let mut d = estate_foreign_draft();
    estate_foreign_allocations(&mut d)[0].alternative.qualified =
        SourceCents::Reported { amount: 4_002 };
    assert_eq!(
        crate::estate_foreign::source_issue(&estate_foreign_data(&mut d)).unwrap().key,
        "ESTATE_FOREIGN_ALLOCATION_INVALID"
    );
}

#[test]
fn estate_foreign_original_one_cent_withholding_survives_zero_community_tax_projection() {
    let mut d = estate_foreign_draft();
    d.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut d);
    d.income[0].community = CommunityShare::Community;
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    d.income[1].community = CommunityShare::Community;
    estate_foreign_data(&mut d).foreign_statement.as_mut().unwrap().tax = 2;
    for row in estate_foreign_allocations(&mut d) {
        row.tax = 1;
    }
    assert!(crate::estate_foreign::source_issue(&estate_foreign_data(&mut d)).is_none());
    for amt in [false, true] {
        let rows = crate::estate_foreign::sources(&d, amt);
        assert_eq!(rows.len(), 2);
        let withheld = rows.iter().find(|s| s.country == "UK").unwrap();
        assert_eq!(withheld.owner, Owner::Taxpayer);
        assert_eq!(withheld.tax, 0);
        assert_eq!(withheld.group(), 1);
        assert_eq!(withheld.income, if amt { 7_001 } else { 2_501 });
        assert_eq!(rows.iter().map(|s| s.tax).sum::<i64>(), 1);
    }
    let result = supported(&d);
    assert_eq!(value(&result, "foreign_tax_credit"), 0);
    assert!(!noted(&result, "WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT"));
    assert!(!noted(&result, "WARN_ESTATE_FOREIGN_FACTS_LEFT_OUT"));
}
