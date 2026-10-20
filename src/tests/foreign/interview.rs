// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::builders::native_income::credit_construction_return;
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, owned};
use super::support::client::income_pages::{
    open_topic, page_documents, save_documents, save_topic,
};
use crate::tests::cross::interview::{
    back, from_review, k1_1041, more_credits_value, open_credits_topic, save_deductions, save_k1s,
    save_more_credits,
};

#[test]
fn unanswered_foreign_tax_preserves_adoption_and_business_credit_construction() {
    let draft = credit_construction_return();
    let mut session = Interview::new(TEST_USER, draft.clone());
    for _ in 0..2 {
        let Reply::Response(P::BeginMoreCredits { previous, value, .. }) =
            from_review(&mut session, TaxSection::MoreCredits)
        else {
            panic!("required foreign-tax construction");
        };
        let adoption = value
            .credits
            .iter()
            .find_map(|credit| match credit {
                MoreCreditInitial::Adoption { adoptions, adoption_carry } => {
                    Some((adoptions, adoption_carry))
                }
                _ => None,
            })
            .unwrap();
        let child = &draft.credits.as_ref().unwrap().adoptions[0];
        // The response and request counterparts retain every source field and
        // catalog in the same order for ordinary structural CopyFrom.
        assert!(!adoption.0[0].wages.is_empty());
        assert_eq!(adoption.0[0].to_avro(), child.page(&adoption.0[0].wages).to_avro());
        assert_eq!(adoption.1, &draft.credits.as_ref().unwrap().adoption_carry);
        let business = value
            .credits
            .iter()
            .find_map(|credit| match credit {
                MoreCreditInitial::BusinessCredits(value) => Some(value),
                _ => None,
            })
            .unwrap();
        let expected = BusinessCreditHistoryPage::from(
            draft.credits.as_ref().unwrap().business_credits.clone().unwrap(),
        );
        assert_eq!(business.to_avro(), expected.to_avro());
        assert!(!business.credits[0].allowed_carbon_oxide_origins.is_empty());
        assert!(!business.carryforwards[0].allowed_research_choices.is_empty());
        assert!(!business.carrybacks[0].allowed_research_choices.is_empty());
        let bonds = value
            .credits
            .iter()
            .find_map(|credit| match credit {
                MoreCreditInitial::TaxCreditBonds { tax_credit_bonds } => Some(tax_credit_bonds),
                _ => None,
            })
            .unwrap();
        assert_eq!(bonds, &draft.credits.as_ref().unwrap().tax_credit_bonds);
        let entity = bonds[0].origin.entity().unwrap();
        assert_eq!(entity.designated_income, 250_000);
        assert_eq!(entity.income_already_reported, 150_000);
        assert!(
            value.credit_income_sources.iter().any(|source| {
                Some(source.id) == entity.income_source && source.amount == 400_000
            })
        );
        let request = back(&session.draft, previous);
        send(&mut session, request);
        assert_eq!(session.draft, draft);
        session = restart(session.draft);
    }
}

// Unit M3-P2: 1099-INT box 6 and 1099-DIV box 7 fold into the return-level
// small foreign-tax election instead of a per-document stored field.
#[test]
fn small_foreign_tax_becomes_the_statement_election() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let int = Int1099 {
        more: IntMore {
            foreign_tax: 20_000,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Synthetic Bank".into(), 50_000)
    };
    let div = Div1099 {
        more: DivMore {
            foreign_tax: 10_000,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_div1099("".into(), 30_000, 0)
    };
    send(
        &mut session,
        save_documents(
            &original,
            vec![
                document(0, IncomeDocument::Int1099(int.clone())),
                document(0, IncomeDocument::Div1099(div.clone())),
            ],
        ),
    );
    // Exactly $300, the single-filer limit: the amounts stay on the two
    // documents (box 6 and box 7 are read from there) and the claim holds
    // only the election. The 1099 boxes do not show that all foreign income
    // is passive and on statements (section 904(j)(2)), so the election starts
    // unanswered: no credit and one notice pointing to More
    // credits, where the person answers.
    assert_eq!(
        session.draft.credits.as_ref().unwrap().foreign_tax,
        ForeignTaxClaim::Statements { eligible: None }
    );
    // A required eligibility answer is constructed without supplying No or Yes.
    // Other accepted credit values survive both the first opening and restart.
    session.draft.credits.as_mut().unwrap().decline_earned_income_credit = true;
    let before = session.draft.clone();
    let Reply::Response(P::BeginMoreCredits { previous, value: initial, .. }) =
        from_review(&mut session, TaxSection::MoreCredits)
    else {
        panic!("required foreign-tax construction");
    };
    assert!(initial.credits.iter().any(|credit| matches!(
        credit,
        crate::federal_model::MoreCreditInitial::ForeignTax(
            crate::extended_model::ForeignTaxInitial::Statements { eligible: None }
        )
    )));
    assert!(initial.credits.iter().any(|credit| matches!(
        credit,
        crate::federal_model::MoreCreditInitial::DeclineEarnedIncomeCredit
    )));
    let request = back(&session.draft, previous);
    send(&mut session, request);
    assert_eq!(session.draft, before);
    let mut session = restart(session.draft);
    assert!(matches!(
        from_review(&mut session, TaxSection::MoreCredits),
        Reply::Response(P::BeginMoreCredits { .. })
    ));
    assert_eq!(crate::extended_rules::payee_statement_taxes(&session.draft), 30_000);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert!(noted(&r, "WARN_FOREIGN_TAX_CREDIT_NOT_CLAIMED"));
    // More credits owns the answer: Yes claims the $300, and a later save of
    // Interest and dividends keeps the answer.
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(ForeignTaxPage::Statements { eligible: true })],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut session, request);
    let request = save_documents(
        &session.draft,
        vec![
            document(0, IncomeDocument::Int1099(int.clone())),
            document(0, IncomeDocument::Div1099(div.clone())),
        ],
    );
    send(&mut session, request);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().foreign_tax,
        ForeignTaxClaim::Statements { eligible: Some(true) }
    );
    let r = supported(&session.draft);
    assert_eq!(value(&r, "foreign_tax_credit"), 300);
    // Reopening restores each document's box 6/7 amount from the document.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::InterestDividends)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::Int1099(int));
    assert_eq!(documents[1].document, IncomeDocument::Div1099(div.clone()));
    // Boundary: one more cent needs Form 1116. Still stored, not rejected.
    let mut int_over = documents[0].document.clone();
    let IncomeDocument::Int1099(Int1099 { more: IntMore { foreign_tax, .. }, .. }) = &mut int_over
    else {
        unreachable!()
    };
    *foreign_tax += 1;
    let mut over_session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut over_session,
        save_documents(
            &original,
            vec![document(0, int_over), document(0, IncomeDocument::Div1099(div))],
        ),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // Unconfirmed, the claim is not made; confirmed, $300.01 is refused.
    assert_eq!(estimate(&over_session.draft).state, ResultState::EstimatedForSupportedScope);
    let request = save_more_credits(
        &over_session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(ForeignTaxPage::Statements { eligible: true })],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut over_session, request);
    let r = estimate(&over_session.draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.message.contains("$300")), "{r:?}");
}

#[test]
fn foreign_tax_above_the_election_points_to_form_1116() {
    // The $300 single-filer election (extended_rules.rs::foreign_credit):
    // box 6/7 foreign tax of $300.01 already saves (small_foreign_tax_becomes_
    // the_statement_election, tests/interview.rs) but the calculation refuses
    // it by name, pointing to Form 1116. The More credits page is where a
    // claim above the election is routed: its foreign-tax item carries the
    // whole existing ForeignTaxClaim election unchanged, so choosing the
    // Form 1116 (Ordinary) branch there replaces the over-the-cap Statements
    // claim.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let int = Int1099 {
        more: IntMore {
            foreign_tax: 30_001,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Synthetic Bank".into(), 50_000)
    };
    send(&mut session, save_documents(&original, vec![document(0, IncomeDocument::Int1099(int))]));
    assert_eq!(
        session.draft.credits.as_ref().unwrap().foreign_tax,
        ForeignTaxClaim::Statements { eligible: None }
    );
    // Confirmed on More credits, the claim above the election is refused by
    // name.
    let confirmed = ForeignTaxClaim::Statements { eligible: Some(true) };
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(confirmed.try_into().unwrap())],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut session, request);
    let r = estimate(&session.draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.message.contains("Form 1116")), "{r:?}");
    // The More credits page lists the over-the-cap Statements claim; routing
    // it to Form 1116 (Ordinary) instead replaces the stored claim.
    let page = open_credits_topic(&mut session, TaxSection::MoreCredits);
    assert_eq!(
        more_credits_value(&page).credits,
        vec![MoreCredit::ForeignTax(ForeignTaxPage::Statements { eligible: true })]
    );
    let ordinary = ForeignTaxClaim::Ordinary {
        sources: Vec::new(),
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(ordinary.clone().try_into().unwrap())],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut session, request);
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, ordinary);
    // Removing the item from the list clears the field back to No.
    let request = save_more_credits(
        &session.draft,
        MoreCredits { credits: Vec::new(), credit_income_sources: Vec::new() },
    );
    send(&mut session, request);
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, ForeignTaxClaim::No);
}

#[test]
fn foreign_account_kind_answers_schedule_b_part_iii() {
    // Schedule B Part III: line 7a (a foreign financial account), 7b (its
    // country) and 8 (a foreign trust) are Yes/No answers; a Yes adds the
    // FinCEN 114 / Form 3520 review note and changes no amount.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let baseline = supported(&s.draft);
    let account = ForeignAccountOrTrust {
        account: true,
        fbar_required: Some(true),
        fbar_countries: vec![0x4341],
        trust: false,
        highest_value: Some(4_000_000),
    };
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InterestDividends,
        vec![owned(Owner::Taxpayer, 0, IncomeDocument::ForeignAccountOrTrust(account.clone()))],
    );
    accepted(&send(&mut s, request));
    let screening = s.draft.screening.clone().unwrap();
    assert!(screening.foreign_account);
    assert_eq!(screening.fbar_countries, vec![0x4341]);
    assert!(!screening.foreign_trust);
    let result = supported(&s.draft);
    assert_eq!(result.tax, baseline.tax);
    assert!(result.review_notes.iter().any(|n| n.contains("FinCEN 114 / Form 3520")));
    // It is not an income entry, and it reopens as the kind.
    assert_eq!(s.draft.income.len(), complete(FederalStatus::Single).income.len());
    let reopened = page_documents(&open_topic(&mut s, TaxSection::InterestDividends)).to_vec();
    assert_eq!(
        reopened,
        vec![IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..owned(Owner::Taxpayer, 0, IncomeDocument::ForeignAccountOrTrust(account.clone()))
        }]
    );
    // A trust alone: line 7a No, no country, line 8 Yes.
    let trust = ForeignAccountOrTrust {
        account: false,
        fbar_required: None,
        fbar_countries: vec![],
        trust: true,
        highest_value: Some(4_000_000),
    };
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InterestDividends,
        vec![document(0, IncomeDocument::ForeignAccountOrTrust(trust))],
    );
    accepted(&send(&mut s, request));
    let screening = s.draft.screening.clone().unwrap();
    assert_eq!(
        (screening.foreign_account, screening.fbar_required, screening.foreign_trust),
        (false, None, true)
    );
    // Twice is refused by name; none clears the answers.
    let before = s.draft.clone();
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InterestDividends,
                vec![
                    document(0, IncomeDocument::ForeignAccountOrTrust(account.clone())),
                    document(0, IncomeDocument::ForeignAccountOrTrust(account)),
                ],
            ),
        ),
        "FOREIGN_ACCOUNT_TWICE",
    );
    accepted(&send(&mut s, save_topic(&before, TaxSection::InterestDividends, vec![])));
    let screening = s.draft.screening.clone().unwrap();
    assert_eq!((screening.foreign_account, screening.foreign_trust), (false, false));
}

// T2-26: ForeignOrdinaryTax's client rule (qualified dividends within the
// source's income) is checked again when More credits is saved.
#[test]
fn foreign_qualified_above_source_income_rejects_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let claim = |qualified: i64| ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "RIC international fund".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::Below15,
                qualified,
            },
            income: 100_000,
            tax: 10_000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(claim(100_001).try_into().unwrap())],

            credit_income_sources: Vec::new(),
        },
    );
    let reply = send(&mut session, request);
    rejected_with(reply, "FOREIGN_QUALIFIED_ABOVE_INCOME");
    assert_eq!(session.draft, original);
    assert!(!crate::extended_rules::valid_credit(&claim(100_001)));
    // Equal to the income is accepted.
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credits: vec![MoreCredit::ForeignTax(claim(100_000).try_into().unwrap())],

            credit_income_sources: Vec::new(),
        },
    );
    send(&mut session, request);
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, claim(100_000));
}

#[test]
fn foreign_account_kind_asks_the_highest_value() {
    // A foreign account needs the highest value during 2026 (both sides,
    // FOREIGN_ASSETS_VALUE_NEEDED); it is stored for the Form 8938 check,
    // reopens with the kind, and $80,000 for a single filer in the United
    // States is above the $75,000 any-time threshold.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut account = ForeignAccountOrTrust {
        account: true,
        fbar_required: Some(true),
        fbar_countries: vec![0x4341],
        trust: false,
        highest_value: None,
    };
    let save = |s: &mut Interview, value: &ForeignAccountOrTrust| {
        let request = save_topic(
            &s.draft.clone(),
            TaxSection::InterestDividends,
            vec![document(0, IncomeDocument::ForeignAccountOrTrust(value.clone()))],
        );
        send(s, request)
    };
    rejected_with(save(&mut s, &account), "FOREIGN_ASSETS_VALUE_NEEDED");
    account.highest_value = Some(8_000_000);
    accepted(&save(&mut s, &account));
    assert_eq!(s.draft.screening.as_ref().unwrap().foreign_asset_max, Some(8_000_000));
    let note =
        supported(&s.draft).review_notes.into_iter().find(|n| n.contains("Form 8938")).unwrap();
    assert!(note.starts_with("Form 8938 is likely required"), "{note}");
    let reopened = page_documents(&open_topic(&mut s, TaxSection::InterestDividends)).to_vec();
    assert_eq!(
        reopened,
        vec![IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::ForeignAccountOrTrust(account.clone()))
        }]
    );
    // With neither answer the value is not needed and not stored.
    let none = ForeignAccountOrTrust {
        account: false,
        fbar_required: None,
        fbar_countries: vec![],
        trust: false,
        highest_value: Some(8_000_000),
    };
    accepted(&save(&mut s, &none));
    assert_eq!(s.draft.screening.as_ref().unwrap().foreign_asset_max, None);
}

fn foreign_deduction_page(owner: Owner) -> ForeignTaxPage {
    ForeignTaxPage::Deduction {
        taxes: vec![ForeignTaxDeductedPage {
            owners: None,
            label: "Synthetic foreign employer tax".into(),
            income: ForeignTaxIncomePage::Earned { owner },
            tax: 300_000,
        }],
    }
}

#[test]
fn foreign_deduction_public_page_estimates_saves_reopens_and_preserves_choice() {
    let d = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d);
    let mut page = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &mut page else { unreachable!() };
    value.credits = vec![MoreCredit::ForeignTax(foreign_deduction_page(Owner::Taxpayer))];
    let edited = value.clone();
    let before = session.draft.clone();
    let reply = send(
        &mut session,
        P::EstimateMoreCredits {
            initial_carbon_oxide_origins: vec![],
            initial_renewable_facility_origins: Vec::new(),
            businesses: vec![],
            farms: vec![],
            clean_vehicle_properties: vec![],
            rentals: vec![],
            research_businesses: vec![],
            entered_credits: vec![],
            carryback_credits: vec![],
            allowed_carry_filers: vec![],
            initial_credit_elections: vec![],
            initial_research_choices: vec![],
            user: TEST_USER,
            allowed_owners: vec![0],
            mortgage_loans: vec![],
            foreign_businesses: vec![],
            capital_sales: vec![],
            foreign_situations: vec![],
            value: edited.clone(),
        },
    );
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(session.draft, before);
    send(&mut session, page);
    let saved = session.draft.credits.as_ref().unwrap().foreign_tax.clone();
    assert!(matches!(saved, ForeignTaxClaim::Deduction { .. }));
    assert!(noted(&supported(&session.draft), "WARN_FOREIGN_TAX_DEDUCTION_UNUSED"));
    let page = open_credits_topic(&mut session, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page), &edited);
    let mut restarted = restart(session.draft.clone());
    let page = open_credits_topic(&mut restarted, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page), &edited);
    // A later investment save must not replace the explicit deduction with the
    // automatically discovered, unconfirmed payee-statement election.
    let original = restarted.draft.clone();
    let int = Int1099 {
        more: IntMore { foreign_tax: 12_000, ..IntMore::default() },
        ..native_int1099("Synthetic bank".into(), 100_000)
    };
    send(
        &mut restarted,
        save_documents(&original, vec![document(0, IncomeDocument::Int1099(int))]),
    );
    assert_eq!(restarted.draft.credits.as_ref().unwrap().foreign_tax, saved);
    // Selecting another deduction method leaves More credits' own record intact.
    for route in [DeductionRoute::Itemize, DeductionRoute::Standard] {
        let request = save_deductions(&restarted.draft, route, 0);
        send(&mut restarted, request);
        assert_eq!(restarted.draft.credits.as_ref().unwrap().foreign_tax, saved);
    }
    let page = open_credits_topic(&mut restarted, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page), &edited);
}

#[test]
fn foreign_deduction_earned_owner_rejects_forged_catalog_in_save_and_estimate() {
    let d = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d);
    let mut page = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, allowed_owners, .. } = &mut page else { unreachable!() };
    *allowed_owners = vec![Owner::Taxpayer as i32, Owner::Spouse as i32];
    value.credits = vec![MoreCredit::ForeignTax(foreign_deduction_page(Owner::Spouse))];
    let value = value.clone();
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "FOREIGN_DEDUCTION_OWNER_NOT_ALLOWED");
    assert_eq!(session.draft, before);
    rejected_with(
        send(
            &mut session,
            P::EstimateMoreCredits {
                initial_carbon_oxide_origins: vec![],
                initial_renewable_facility_origins: Vec::new(),
                businesses: vec![],
                farms: vec![],
                clean_vehicle_properties: vec![],
                rentals: vec![],
                research_businesses: vec![],
                entered_credits: vec![],
                carryback_credits: vec![],
                allowed_carry_filers: vec![],
                initial_credit_elections: vec![],
                initial_research_choices: vec![],
                user: TEST_USER,
                allowed_owners: vec![0, 1],
                mortgage_loans: vec![],
                foreign_businesses: vec![],
                capital_sales: vec![],
                foreign_situations: vec![],
                value,
            },
        ),
        "FOREIGN_DEDUCTION_OWNER_NOT_ALLOWED",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn foreign_deduction_row_ranges_and_collection_limit_have_server_twins() {
    for (count, amount, label) in [
        (51, 100, "Synthetic tax"),
        (1, 0, "Synthetic tax"),
        (1, 100_000_000_001, "Synthetic tax"),
        (1, 100, ""),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction {
            taxes: (0..count)
                .map(|_| ForeignTaxDeducted {
                    owners: None,
                    label: label.into(),
                    income: ForeignTaxIncome::Other,
                    tax: amount,
                })
                .collect(),
        };
        assert!(!d.valid_stored());
        let request = save_more_credits(
            &complete(FederalStatus::Single),
            MoreCredits {
                credit_income_sources: vec![],
                credits: vec![MoreCredit::ForeignTax(
                    d.credits.unwrap().foreign_tax.try_into().unwrap(),
                )],
            },
        );
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let before = session.draft.clone();
        let reply = send(&mut session, request);
        assert_eq!(session.draft, before, "{reply:?}");
    }
}

#[test]
fn foreign_deduction_reuses_current_estate_payee_statement_owner_once() {
    let mut page = k1_1041("Synthetic fiduciary foreign-tax statement");
    page.interest = 100_000;
    page.foreign_statement = Some(crate::EstateForeignStatement {
        tax: 20_000,
        treatment: crate::EstateForeignTreatment::QualifiedPassivePayeeStatement,
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credit_income_sources: vec![],
            credits: vec![MoreCredit::ForeignTax(ForeignTaxPage::Deduction { taxes: vec![] })],
        },
    );
    accepted(&send(&mut session, request));
    let request = save_deductions(&session.draft, DeductionRoute::Itemize, 0);
    accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    assert_eq!(value(&r, "other_taxes_deduction"), 200);
    assert_eq!(value(&r, "foreign_investment_tax_deduction"), 200);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(supported(&restart(session.draft).draft), r);
}

#[test]
fn foreign_deduction_removing_choice_releases_only_its_normal_page_record() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().foreign_tax = foreign_deduction_page(Owner::Taxpayer).into();
    let mut session = restart(d);
    let mut request = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &mut request else { unreachable!() };
    value.credits.retain(|item| !matches!(item, MoreCredit::ForeignTax(_)));
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.credits.as_ref().unwrap().foreign_tax, ForeignTaxClaim::No);
    assert!(!noted(&supported(&session.draft), "WARN_FOREIGN_TAX_DEDUCTION_UNUSED"));
    let saved = session.draft.clone();
    let mut restarted = restart(saved.clone());
    let request = open_credits_topic(&mut restarted, TaxSection::MoreCredits);
    assert!(
        !more_credits_value(&request)
            .credits
            .iter()
            .any(|item| matches!(item, MoreCredit::ForeignTax(_)))
    );
    assert_eq!(restarted.draft, saved);
}
