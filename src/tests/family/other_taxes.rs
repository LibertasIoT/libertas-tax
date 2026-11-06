// u5-credits: the stored data no page asked before (Schedule H, Form 8828,
// Form 5329 Parts X-XI Trump accounts and the Form 4136 home kerosene
// credit), each reached through its page: the Other taxes list or the More
// credits list. Every expected amount is computed by hand in the comment
// beside it from the 2026 draft forms (f1040sh--dft, f8828, f5329--dft
// Parts X-XI, f4136--dft/i4136--dft line 4a).
use super::*;
use crate::tests::cross::interview::{
    open_other_taxes, other_taxes_request, refusal, save_other_taxes,
};
use crate::tests::federal::other_taxes::{line23, ok, refused_key};

// ---------------------------------------------------------------------------
// Form 5329 Parts X-XI (Trump accounts) through the Other taxes page.

fn young() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100301;
    d
}
fn account(contributions: i64, exempt: i64, returns: Vec<TrumpExcessReturn>) -> TrumpAccount {
    TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: contributions * 100,
        exempt: exempt * 100,
        returns,
        confirmed: true,
    }
}
fn returned(principal: i64, earnings: i64) -> TrumpExcessReturn {
    TrumpExcessReturn {
        document_id: None,
        principal: principal * 100,
        earnings: earnings * 100,
        received: 20261201,
        deadline: 20270415,
        confirmed: true,
    }
}

#[test]
fn trump_account_excess_through_the_other_taxes_page() {
    let d = young();
    let before = line23(&d);
    let mut s = Interview::new(TEST_USER, d);
    // $8,000 accepted, $1,000 pilot (exempt): line 57 8,000 - 1,000 - 5,000
    // = 2,000; $500 principal returned by the due date: line 59 500,
    // line 60 1,500, line 61 6% = 90; $20 earnings: lines 62-63 20.
    // Schedule 2 lines 18 and 5 add 110.
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts {
            accounts: vec![(&account(8_000, 1_000, vec![returned(500, 20)])).into()],
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "trump_account_taxpayer_line57_excess"), 2_000);
    assert_eq!(value(&r, "trump_account_taxpayer_line60_remaining"), 1_500);
    assert_eq!(value(&r, "trump_account_taxpayer_line61_tax"), 90);
    assert_eq!(value(&r, "trump_account_taxpayer_line63_tax"), 20);
    assert_eq!(line23(&s.draft) - before, 11_000);
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = open_other_taxes(&mut again);
    let P::SaveOtherTaxes { value: shown, .. } = &page else { unreachable!() };
    let request = other_taxes_request(&page, &again.draft.clone(), shown.taxes.clone());
    ok(&send(&mut again, request));
    assert_eq!(again.draft, stored);
}

#[test]
fn late_trump_account_excess_note_keeps_exact_usd_amount() {
    let mut s = Interview::new(TEST_USER, young());
    let mut late = returned(500, 20);
    late.received = 20270501;
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts {
            accounts: vec![(&account(8_000, 1_000, vec![late])).into()],
        }],
    ));
    assert!(supported(&s.draft).review_notes.iter().any(|note| {
        note.display
            == Message::new(
                "NOTE_TRUMP_ACCOUNT_LATE_EXCESS_TAXPAYER",
                vec![MessageArg::usd_dollars(500)],
            )
    }));
}

#[test]
fn trump_account_five_thousand_dollar_limit_boundary() {
    // Nonexempt exactly $5,000: no excess. $5,100: 100 x 6% = 6.
    let mut s = Interview::new(TEST_USER, young());
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts { accounts: vec![(&account(6_000, 1_000, vec![])).into()] }],
    ));
    assert_eq!(value(&supported(&s.draft), "trump_account_taxpayer_line61_tax"), 0);
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts { accounts: vec![(&account(6_100, 1_000, vec![])).into()] }],
    ));
    assert_eq!(value(&supported(&s.draft), "trump_account_taxpayer_line61_tax"), 6);
}

// The account beneficiary must be under 18 at year end, day-exact (Pub. 501):
// born 2009-01-01 the 18th birthday is 2027-01-01, attained 2026-12-31, so the
// 2026 return does not apply; born 2009-01-02 is 17. $6,100 of nonexempt
// contributions less the $1,000 exempt = $5,100, $100 over the $5,000 limit,
// x 6% = $6.
#[test]
fn trump_account_beneficiary_under_18_is_day_exact() {
    let mut d = young();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090102;
    let mut s = Interview::new(TEST_USER, d);
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts { accounts: vec![(&account(6_100, 1_000, vec![])).into()] }],
    ));
    assert_eq!(value(&supported(&s.draft), "trump_account_taxpayer_line61_tax"), 6);
    s.draft.people.as_mut().unwrap().taxpayer.birth_date = 20090101;
    unsupported(&s.draft, "Trump account");
}

#[test]
fn trump_account_page_rules_are_checked_on_save() {
    let mut s = Interview::new(TEST_USER, young());
    let before = s.draft.clone();
    refused_key(
        save_other_taxes(
            &mut s,
            vec![OtherTax::TrumpAccounts {
                accounts: vec![(&account(1_000, 1_001, vec![])).into()],
            }],
        ),
        "TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS",
    );
    refused_key(
        save_other_taxes(
            &mut s,
            vec![OtherTax::TrumpAccounts {
                accounts: vec![(&account(8_000, 0, vec![returned(100, -101)])).into()],
            }],
        ),
        "TRUMP_LOSS_ABOVE_PRINCIPAL",
    );
    let mut spouse = account(8_000, 0, vec![]);
    spouse.owner = Owner::Spouse;
    assert!(
        refusal(save_other_taxes(
            &mut s,
            vec![OtherTax::TrumpAccounts { accounts: vec![(&spouse).into()] }],
        ))
        .contains("joint return")
    );
    assert_eq!(s.draft, before);
    // A loss equal to the principal is accepted.
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::TrumpAccounts {
            accounts: vec![(&account(8_000, 0, vec![returned(100, -100)])).into()],
        }],
    ));
}

// Form 5329 Part V: public OtherTaxes input, actual 2026 IRS draft p. 7
// and final Pub 970 chapter 6 (including its expressly dated 2026 example).
fn coverdell_session(status: FederalStatus, birth: u32) -> Interview {
    let mut s = Interview::new(TEST_USER, complete(status));
    let mut you = crate::interview::filer(&s.draft.people.as_ref().unwrap().taxpayer);
    you.birth_date = birth;
    let request = crate::tests::cross::interview::save_you(&s.draft.clone(), you);
    ok(&send(&mut s, request));
    s
}
fn coverdell_topic(v: CoverdellExcessPage) -> OtherTax {
    OtherTax::CoverdellExcess { beneficiaries: vec![v] }
}
fn coverdell_record(contribution: i64) -> CoverdellExcessPage {
    CoverdellExcessPage {
        beneficiary: Owner::Taxpayer,
        contributors: vec![CoverdellContributorPage {
            label: "Synthetic donor".into(),
            person: CoverdellContributorKindPage::Organization,
            contributions: if contribution == 0 {
                vec![]
            } else {
                vec![CoverdellContribution {
                    account: "Synthetic ESA".into(),
                    paid: 20261201,
                    amount: contribution * 100,
                }]
            },
        }],
        corrections: vec![],
        prior_excess: 0,
        prior_tax: 0,
        distributions: 0,
        value: 1_000_000,
        special_needs: false,
    }
}
fn coverdell_return(principal: i64, earnings: i64, withdrawn: u32) -> CoverdellCorrectionPage {
    CoverdellCorrectionPage {
        account: "Synthetic ESA".into(),
        principal: principal * 100,
        earnings: earnings * 100,
        withdrawn,
        income_document: None,
    }
}
#[test]
fn coverdell_published_2026_prior_excess_is_absorbed_by_current_room() {
    // Pub970 2025 ch6 actual 2026 example: 2025 excess550/tax33,
    // 2026 permitted2000/contributed1450: room550, old/current excess0.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(1450);
    v.prior_excess = 55_000;
    v.prior_tax = 3300;
    let topic = coverdell_topic(v);
    ok(&save_other_taxes(&mut s, vec![topic.clone()]));
    let r = supported(&s.draft);
    for (line, expected) in
        [(26, 550), (27, 550), (28, 0), (29, 550), (30, 0), (31, 0), (32, 0), (33, 0)]
    {
        assert_eq!(value(&r, &alloc::format!("coverdell_taxpayer_line_{line}")), expected);
    }
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let P::SaveOtherTaxes { value: shown, .. } = open_other_taxes(&mut again) else {
        unreachable!()
    };
    assert_eq!(shown.taxes, vec![topic]);
    ok(&save_other_taxes(&mut again, shown.taxes));
    assert_eq!(again.draft, stored);
}
#[test]
fn coverdell_donor_phaseout_and_value_cap_are_beneficiary_wide() {
    // Worksheet6-2 donorMAGI96500: 2000*(1-1500/15000)=1800.
    // Actual2300 across two accounts => excess500, tax30; value200 =>12.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(2300);
    v.contributors[0].person = CoverdellContributorKindPage::OtherPerson {
        modified_agi: 9_650_000,
        status: FederalStatus::Single,
    };
    v.contributors[0].contributions[0].amount = 100_000;
    v.contributors[0].contributions.push(CoverdellContribution {
        account: "Second ESA".into(),
        paid: 20261215,
        amount: 130_000,
    });
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v.clone())]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_31"), 500);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 30);
    v.value = 20_000;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 500);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 12);
}
#[test]
fn coverdell_timely_2027_return_adds_original_year_earnings_once_without_early_tax() {
    // 2500 deposited, 500+50 earnings returnedMay31,2027: net2000,
    // PartVtax0; 2026Schedule1income50, PartIIexception50, earlytax0.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let before = supported(&s.draft);
    let mut v = coverdell_record(2500);
    v.corrections.push(coverdell_return(500, 50, 20270531));
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_31"), 0);
    assert_eq!(value(&r, "coverdell_correction_1_earnings"), 50);
    assert_eq!(value(&r, "education_account_taxable_earnings"), 50);
    assert_eq!(return_line(&r, "9") - return_line(&before, "9"), 5000);
    assert_eq!(return_line(&r, "23") - return_line(&before, "23"), 0);
}
#[test]
fn coverdell_returned_nonexcess_contribution_is_excluded_under_actual_code() {
    // §§530(d)(4)(C),4973(e)(2)(A): qualifying returned contributions
    // need not be confined to original excess. 1500 deposited,500+20
    // returned in time => net1000, room1000 absorbs retained800,tax0.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(1500);
    v.prior_excess = 80_000;
    v.prior_tax = 4800;
    v.corrections.push(coverdell_return(500, 20, 20270515));
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_27"), 1000);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 0);
    assert_eq!(value(&r, "education_account_taxable_earnings"), 20);
}
#[test]
fn coverdell_prior_only_unknown_limit_uses_only_proved_tax_and_carry() {
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(0);
    v.contributors.clear();
    v.prior_excess = 50_000;
    v.prior_tax = 3000;
    v.distributions = 50_000;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v.clone())]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 0);
    assert!(!r.lines.iter().any(|l| l.id == "coverdell_taxpayer_line_27"));
    v.distributions = 0;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COVERDELL_LIMIT_UNAVAILABLE"));
}
#[test]
fn coverdell_actual_deemed_age_and_special_needs_are_not_guessed() {
    let mut s = coverdell_session(FederalStatus::Single, 20081231);
    let mut v = coverdell_record(2500);
    v.contributors[0].contributions[0].paid = 20261230;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v.clone())]));
    assert_eq!(value(&supported(&s.draft), "coverdell_taxpayer_line_33"), 30);
    // January2027 designated2026 is deemedDecember31,2026, the18thbirthday.
    v.contributors[0].contributions[0].paid = 20270115;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v.clone())]));
    assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
    v.special_needs = true;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    assert_eq!(value(&supported(&s.draft), "coverdell_taxpayer_line_33"), 30);
}
#[test]
fn coverdell_failed_correction_save_preserves_accepted_draft_and_dirty_state() {
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(coverdell_record(2500))]));
    let accepted = s.draft.clone();
    let dirty = s.dirty;
    let mut v = coverdell_record(2500);
    v.corrections.push(coverdell_return(2501, 0, 20270501));
    refused_key(
        save_other_taxes(&mut s, vec![coverdell_topic(v)]),
        "COVERDELL_RETURN_ABOVE_CONTRIBUTIONS",
    );
    assert_eq!(s.draft, accepted);
    assert_eq!(s.dirty, dirty);
    let mut v = coverdell_record(2500);
    v.corrections.push(coverdell_return(500, 0, 20270501));
    v.corrections[0].income_document = Some(9999);
    let page = open_other_taxes(&mut s);
    let mut request = other_taxes_request(&page, &s.draft.clone(), vec![coverdell_topic(v)]);
    let P::SaveOtherTaxes { coverdell_corrections, .. } = &mut request else { unreachable!() };
    coverdell_corrections.push(RecordChoice {
        id: 9999,
        label: crate::Message::new(
            "COVERDELL_SOURCE_LABEL",
            vec![MessageArg::Text("Synthetic forged source".into())],
        )
        .encode(),
    });
    refused_key(send(&mut s, request), "COVERDELL_CORRECTION_SOURCE_INVALID");
    assert_eq!(s.draft, accepted);
    assert_eq!(s.dirty, dirty);
}
#[test]
fn coverdell_current_code2_source_is_linked_once_and_deletion_keeps_actual_correction() {
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut rows = crate::tests::support::client::income_pages::all_documents(&mut s);
    rows.push(crate::tests::cross::interview::document(
        0,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Q1099(q1099_paper(Q1099 {
                payer_trustee: Default::default(),
                recipient: Default::default(),
                gross: 55_000,
                account: EducationAccount::Coverdell {
                    amounts: CoverdellAmounts::Reported { earnings: 5000, basis: 50_000 },
                },
                transfer: Q1099Transfer::ExcessThisYear,
                recipient_not_beneficiary: false,
            })),
        },
    ));
    let page = crate::tests::support::client::income_pages::save_documents(&mut s, rows);
    ok(&send(&mut s, page));
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::EducationDistribution { .. }))
        .unwrap()
        .id;
    let mut v = coverdell_record(2500);
    let mut correction = coverdell_return(500, 50, 20261215);
    correction.income_document = Some(id);
    v.corrections.push(correction);
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "education_account_taxable_earnings"), 50);
    assert!(!r.lines.iter().any(|l| l.id == "coverdell_correction_1_earnings"));
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 0);
    // Editing the linked source is accepted normally, reopens this owner
    // page, and names the unresolved mandatory tax instead of blocking the
    // income Save or counting the old correction earnings again.
    let mut changed = restart(s.draft.clone());
    let mut page = crate::tests::support::client::income_pages::open_document_center(&mut changed);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    let crate::OtherIncomeDocument::Q1099(q) = &mut documents
        .other
        .other_income
        .documents
        .iter_mut()
        .find(|e| e.id == id)
        .unwrap()
        .document
    else {
        unreachable!()
    };
    q.gross_distribution = 56_000;
    q.distribution_from = crate::Form1099QDistributionFrom::CoverdellEsa;
    q.earnings = 6000;
    q.basis = 50_000;
    q.fair_market_value = None;
    ok(&send(&mut changed, page));
    assert!(!changed.draft.finished_sections.contains(&TaxSection::OtherTaxes));
    let r = estimate(&changed.draft);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COVERDELL_CORRECTION_REVIEW"));
    assert_eq!(r.tax, None);
    let mut corrected = coverdell_record(2500);
    let mut correction = coverdell_return(500, 60, 20261215);
    correction.income_document = Some(id);
    corrected.corrections.push(correction);
    ok(&save_other_taxes(&mut changed, vec![coverdell_topic(corrected)]));
    assert_eq!(value(&supported(&changed.draft), "education_account_taxable_earnings"), 60);
    let mut again = restart(s.draft.clone());
    let mut page = crate::tests::support::client::income_pages::open_document_center(&mut again);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.other.other_income.documents.retain(|d| d.id != id);
    ok(&send(&mut again, page));
    assert_eq!(again.draft.coverdell_excess[0].corrections[0].income_document, None);
    let r = supported(&again.draft);
    assert_eq!(value(&r, "education_account_taxable_earnings"), 50);
    assert_eq!(value(&r, "coverdell_correction_1_earnings"), 50);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 0);
}
#[test]
fn coverdell_leaving_joint_keeps_deposits_without_inventing_former_filer_limits() {
    let mut s = coverdell_session(FederalStatus::Joint, 20100301);
    let mut v = coverdell_record(2500);
    v.contributors[0].person = CoverdellContributorKindPage::Filer { owner: Owner::Spouse };
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    assert_eq!(value(&supported(&s.draft), "coverdell_taxpayer_line_33"), 30);
    let request = crate::tests::cross::interview::go_to(
        &s.draft.clone(),
        crate::tests::cross::interview::PAGE_INCOME,
        TaxSection::Setup,
    );
    let Reply::Edit(P::SaveSetup { mut value, .. }) = send(&mut s, request) else { unreachable!() };
    value.filing = Filing::Single;
    let request = crate::tests::cross::interview::save_setup(&s.draft.clone(), value);
    ok(&send(&mut s, request));
    assert_eq!(s.draft.coverdell_excess[0].contributors[0].contributions[0].amount, 250_000);
    assert!(matches!(
        s.draft.coverdell_excess[0].contributors[0].person,
        CoverdellContributorKind::Unavailable
    ));
    assert!(
        estimate(&s.draft)
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_COVERDELL_LIMIT_UNAVAILABLE")
    );
}

#[test]
fn coverdell_net_contributions_round_after_the_actual_returned_cents() {
    // $2000.49 less $0.51 = $1999.98, rounding to2000.
    // There is no1dollar absorption of the retained500 excess.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(2000);
    v.contributors[0].contributions[0].amount = 200_049;
    v.prior_excess = 50_000;
    v.prior_tax = 3000;
    let mut c = coverdell_return(1, 0, 20261215);
    c.principal = 51;
    v.corrections.push(c);
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_27"), 0);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 500);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 30);
}
#[test]
fn coverdell_two_beneficiaries_keep_separate_limits_and_reconcile_one_return_tax() {
    let mut s = coverdell_session(FederalStatus::Joint, 20100301);
    let mut spouse =
        crate::interview::filer(s.draft.people.as_ref().unwrap().spouse.as_ref().unwrap());
    spouse.birth_date = 20100301;
    let request = crate::tests::cross::interview::save_spouse(&s.draft.clone(), spouse);
    ok(&send(&mut s, request));
    let a = coverdell_record(2500);
    let mut b = coverdell_record(3000);
    b.beneficiary = Owner::Spouse;
    let before = return_line(&supported(&s.draft), "23");
    ok(&save_other_taxes(&mut s, vec![OtherTax::CoverdellExcess { beneficiaries: vec![a, b] }]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 30);
    assert_eq!(value(&r, "coverdell_spouse_line_33"), 60);
    assert_eq!(return_line(&r, "23") - before, 9000);
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn coverdell_known_global_cap_keeps_all_determined_form_lines() {
    // The organization already establishes the2000 global limit. An
    // unknown second donor cannot raise it: total3000 =>excess1000,tax60.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let mut v = coverdell_record(2500);
    v.contributors.push(CoverdellContributorPage {
        label: "Another actual donor".into(),
        person: CoverdellContributorKindPage::Unavailable,
        contributions: vec![CoverdellContribution {
            account: "Synthetic ESA".into(),
            paid: 20261201,
            amount: 50_000,
        }],
    });
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_27"), 0);
    assert_eq!(value(&r, "coverdell_taxpayer_line_29"), 0);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 1000);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 60);
    assert!(!r.review_notes.iter().any(|n| n.display.key == "NOTE_COVERDELL_LIMIT_ENDPOINTS"));
}

#[test]
fn coverdell_zero_prior_value_cap_does_not_erase_statutory_excess() {
    // Independent Code fixture, not an IRS published example: retained2025
    // excess500 and tax0 from value0. In2026 the organization contributes2000
    // (limit2000), distributions0, value2000. §4973(e)(1)(B) carries500;
    // §4973(a) taxes6% of min(500,2000)=30. Sep4,2026 draft i5329 p7's
    // line26 prior-positive-tax gate instead gives0tax/0carry; flag discrepancy.
    let mut s = coverdell_session(FederalStatus::Single, 20100301);
    let before = supported(&s.draft);
    let mut v = coverdell_record(2000);
    v.prior_excess = 50_000;
    v.prior_tax = 0;
    v.value = 200_000;
    ok(&save_other_taxes(&mut s, vec![coverdell_topic(v)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "coverdell_taxpayer_line_26"), 500);
    assert_eq!(value(&r, "coverdell_taxpayer_line_27"), 0);
    assert_eq!(value(&r, "coverdell_taxpayer_line_32"), 500);
    assert_eq!(value(&r, "coverdell_taxpayer_line_33"), 30);
    assert_eq!(return_line(&r, "23") - return_line(&before, "23"), 3000);
    assert!(noted(&r, "NOTE_COVERDELL_PRIOR_ZERO_TAX"));
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
}
