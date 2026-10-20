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

pub(in crate::tests) fn ok(reply: &Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}
pub(in crate::tests) fn refused_key(reply: Reply, key: &str) {
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected {key}, got {reply:?}")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
}
pub(in crate::tests) fn line23(d: &FederalDraft) -> i64 {
    return_line(&supported(d), "23")
}

// ---------------------------------------------------------------------------
// Schedule H through the Other taxes page.

fn nanny(cash_dollars: i64) -> HouseholdEmployer {
    household_employer(cash_dollars)
}

#[test]
fn schedule_h_through_the_other_taxes_page() {
    let d = complete(FederalStatus::Single);
    let before = line23(&d);
    let mut s = Interview::new(TEST_USER, d);
    let page = open_other_taxes(&mut s);
    let P::SaveOtherTaxes { allowed_owners, value: shown, .. } = &page else { unreachable!() };
    assert_eq!(allowed_owners, &vec![Owner::Taxpayer as i32]);
    assert!(shown.taxes.is_empty());
    // One nanny paid $3,500 in March, Texas, state rate 0 and no credit
    // reduction: line A yes ($3,500 >= $3,000).
    //   line 2: 3,500 x 12.4% = 434.00; line 4: 3,500 x 2.9% = 101.50 -> 102;
    //   line 16 (Section A, one state, all paid, all covered): 3,500 x 0.6% = 21;
    //   Schedule H total 434 + 102 + 21 = 557 on Schedule 2 line 17a.
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_500)] }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "household_0_social_tax"), 434);
    assert_eq!(value(&r, "household_0_medicare_tax"), 102);
    assert_eq!(value(&r, "household_0_futa_tax"), 21);
    assert_eq!(value(&r, "household_0_tax"), 557);
    assert_eq!(line23(&s.draft) - before, 55_700);
    // The checklist's household tick follows the entry.
    assert_ne!(s.draft.checklist.other_taxes & OtherTaxTopics::HOUSEHOLD_EMPLOYEE.0, 0);
    // Round trip: saved, restarted, reopened unchanged.
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let P::SaveOtherTaxes { value: shown, .. } = open_other_taxes(&mut again) else {
        unreachable!()
    };
    assert_eq!(shown.taxes, vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_500)] }]);
    let page = open_other_taxes(&mut again);
    let request = other_taxes_request(&page, &again.draft.clone(), shown.taxes.clone());
    ok(&send(&mut again, request));
    assert_eq!(again.draft, stored);
}

#[test]
fn schedule_h_line_a_threshold_is_three_thousand_dollars() {
    // $2,999.99: line A no; line C yes (a quarter of $1,000 or more), so
    // only FUTA: 2,999.99 -> 3,000 x 0.6% = 18.
    let mut e = nanny(0);
    e.workers[0].pay[0].cash = 299_999;
    e.states[0].periods[0].wages = 299_999;
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    ok(&save_other_taxes(&mut s, vec![OtherTax::HouseholdEmployees { employers: vec![e] }]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "household_0_social_tax"), 0);
    assert_eq!(value(&r, "household_0_tax"), 18);
    // $3,000.00 exactly: 372 + 87 + 18 = 477.
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_000)] }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "household_0_social_tax"), 372);
    assert_eq!(value(&r, "household_0_medicare_tax"), 87);
    assert_eq!(value(&r, "household_0_tax"), 477);
}

#[test]
fn schedule_h_futa_quarter_test_is_one_thousand_dollars() {
    // $999.99 in the only quarter and no 2025 quarter of $1,000: line C no,
    // nothing is due. $1,000.00: line C yes, FUTA 1,000 x 0.6% = 6.
    let mut e = nanny(0);
    e.workers[0].pay[0].cash = 99_999;
    e.states[0].periods[0].wages = 99_999;
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    ok(&save_other_taxes(&mut s, vec![OtherTax::HouseholdEmployees { employers: vec![e] }]));
    assert_eq!(value(&supported(&s.draft), "household_0_tax"), 0);
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::HouseholdEmployees { employers: vec![nanny(1_000)] }],
    ));
    assert_eq!(value(&supported(&s.draft), "household_0_tax"), 6);
}

#[test]
fn schedule_h_owner_rules_are_checked_on_save() {
    // A spouse's employer only on a joint return (the client's allowed
    // owners); one employer per filer.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    let mut spouse = nanny(3_000);
    spouse.owner = Owner::Spouse;
    assert!(
        refusal(save_other_taxes(
            &mut s,
            vec![OtherTax::HouseholdEmployees { employers: vec![spouse.clone()] }],
        ))
        .contains("joint return")
    );
    assert_eq!(s.draft, before);
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    refused_key(
        save_other_taxes(
            &mut s,
            vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_000), nanny(4_000)] }],
        ),
        "HOUSEHOLD_EMPLOYER_TWICE",
    );
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_000), spouse] }],
    ));
    // Leaving the joint return erases the spouse's employer (walk-owned
    // data: the row's owner leaves allowed_owners).
    let mut d = s.draft.clone();
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    d.setup.as_mut().unwrap().marital_state = MaritalState::Unmarried;
    d.people.as_mut().unwrap().spouse = None.into();
    let mut pruned = d.clone();
    crate::interview::prune(&mut pruned);
    assert_eq!(pruned.screening.as_ref().unwrap().household_employment, vec![nanny(3_000)]);
}

// ---------------------------------------------------------------------------
// Form 8828 through the Other taxes page.

pub(in crate::tests) fn gift(
    original: u32,
    fmv: i64,
    basis: i64,
    aqi: i64,
    holding: u32,
) -> MortgageSubsidyEntry {
    MortgageSubsidyEntry {
        label: "Maple Street".into(),
        disposition: MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::GiftToOther {
                owners: None,
                fair_market_value: fmv * 100,
                adjusted_basis: basis * 100,
                no_consideration_or_debt_relief: true,
            },
            facts: MortgageSubsidyFacts {
                original_closed_on: original,
                disposed_on: 20260501,
                notice_reference: "Issuer notice 2020-17".into(),
                family_size_at_disposition: 3,
                adjusted_qualifying_income: aqi * 100,
                highest_subsidized_principal: 20_000_000,
                holding_percentage: holding,
                payoff: MortgageSubsidyPayoff::AtDispositionOrReissued,
                confirmed: true,
            },
        },
    }
}

// ---------------------------------------------------------------------------
// The page's list, refusal kinds and review row.

#[test]
fn other_taxes_list_keeps_its_refusal_kinds() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    ok(&save_other_taxes(
        &mut s,
        vec![OtherTax::Section965, OtherTax::HouseholdEmployees { employers: vec![nanny(3_000)] }],
    ));
    assert_eq!(s.draft.tax_refusals, vec![OtherTaxRefusal::Section965]);
    let P::SaveOtherTaxes { value, .. } = open_other_taxes(&mut s) else { unreachable!() };
    // Records first, then the refusal kinds.
    assert_eq!(
        value.taxes,
        vec![OtherTax::HouseholdEmployees { employers: vec![nanny(3_000)] }, OtherTax::Section965,]
    );
    // An empty list clears both.
    ok(&save_other_taxes(&mut s, vec![]));
    assert!(s.draft.situations.is_empty());
    assert!(s.draft.screening.as_ref().unwrap().household_employment.is_empty());
}

// Named additional-tax limits are accepted facts, rather than a manual total.
// Independent source expectations are the 2026 draft Schedule 2 lines 7 and
// 13h, with Code sections 453(l)(3) and 409A(a)(1)(B), respectively.
#[test]
fn named_additional_taxes_survive_pages_and_restart_and_clear_without_zero_tax() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(supported(&s.draft).tax, Some(502_300));
    let installment =
        OtherTax::OtherAdditionalTaxes { kind: RareAdditionalTax::InstallmentLotsTimeshares };
    let deferred =
        OtherTax::OtherAdditionalTaxes { kind: RareAdditionalTax::DeferredCompensation409A };
    ok(&save_other_taxes(&mut s, vec![installment.clone(), deferred.clone()]));
    assert_eq!(
        s.draft.tax_refusals,
        vec![installment.refusal().unwrap(), deferred.refusal().unwrap()]
    );
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    for (key, source) in [
        ("ISSUE_RARE_INSTALLMENT_LOTS_TIMESHARES", "Section 453(l)(3); Schedule 2 line 7"),
        ("ISSUE_RARE_DEFERRED_COMPENSATION409_A", "Section 409A(a)(1)(B); Schedule 2 line 13h"),
    ] {
        assert!(
            r.issues.iter().any(|i| i.display.key == key && i.message.contains(source)),
            "{r:?}"
        );
    }
    let mut s = restart(s.draft.clone());
    let P::SaveOtherTaxes { value, .. } = open_other_taxes(&mut s) else { unreachable!() };
    assert_eq!(value.taxes, vec![installment.clone(), deferred.clone()]);
    let accepted = s.draft.clone();
    let dirty = s.dirty;
    refused_key(
        save_other_taxes(&mut s, vec![installment.clone(), installment]),
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
    );
    assert_eq!(s.draft, accepted);
    assert_eq!(s.dirty, dirty);
    ok(&save_other_taxes(&mut s, vec![]));
    assert!(s.draft.tax_refusals.is_empty());
    // Clearing an actual indication restores the ordinary computation; no
    // uncomputed tax has been silently assigned zero while the fact remains.
    assert_eq!(supported(&s.draft).tax, Some(502_300));
}

#[test]
fn other_taxes_server_rejects_duplicate_kinds_without_mutation() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    let dirty = s.dirty;
    // 2026-10-15 D-13 removes the former 24-item cap; actual kind uniqueness remains.
    refused_key(
        save_other_taxes(
            &mut s,
            vec![
                OtherTax::OtherAdditionalTaxes {
                    kind: RareAdditionalTax::AnotherIdentifiedAddedTax,
                };
                25
            ],
        ),
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
    );
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
}

// More than the old twelve-item retained bound must remain accepted through
// restart; each indicated source still yields its named Unsupported result.
#[test]
fn all_sixteen_named_additional_taxes_fit_the_accepted_return_bound() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let items: Vec<_> = RareAdditionalTax::ALL
        .into_iter()
        .map(|kind| OtherTax::OtherAdditionalTaxes { kind })
        .collect();
    ok(&save_other_taxes(&mut s, items.clone()));
    assert_eq!(s.draft.tax_refusals.len(), 16);
    assert!(s.draft.valid_stored());
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert_eq!(r.issues.len(), 16);
    assert_eq!(r.tax, None); // An uncomputed mandatory tax never becomes zero.
    // Approved Unit18 map §3A/§7 retains these exact G15 source-history limits.
    // Numbering is the 2026 draft, rather than the historical ledger labels.
    for (key, source) in [
        ("ISSUE_RARE_FORM4255_NET_EPE", "Schedule 2 lines 1d and 10"),
        ("ISSUE_RARE_FORM4255_EXCESSIVE_PAYMENT", "Schedule 2 lines 1e and 1f"),
        ("ISSUE_RARE_CREDIT_RECAPTURE_OTHER", "Schedule 2 lines 1y/13a"),
        ("ISSUE_RARE_EXCESS_BENEFITS72M5", "Section 72(m)(5); Schedule 2 line 13j"),
        ("ISSUE_RARE_PFIC8621_INTEREST", "Schedule 2 lines 19a–19c"),
    ] {
        assert!(
            r.issues.iter().any(|issue| issue.display.key == key && issue.message.contains(source)),
            "{key}: {r:?}"
        );
    }
    let mut s = restart(s.draft.clone());
    let P::SaveOtherTaxes { value, .. } = open_other_taxes(&mut s) else { unreachable!() };
    assert_eq!(value.taxes, items);
    assert!(s.draft.valid_stored());
    assert_eq!(estimate(&s.draft), r);
}

#[test]
fn unit13_mortgage_gift_owner_save_checks_actual_joint_positive_tax() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let before = session.draft.clone();
    let home = gift(20200601, 300_000, 200_000, 0, 60);
    refused_key(
        save_other_taxes(
            &mut session,
            vec![OtherTax::MortgageSubsidyRecapture { homes: vec![home.clone()] }],
        ),
        "MORTGAGE_GIFT_OWNER_REQUIRED",
    );
    assert_eq!(session.draft, before);
    // This source fact is required even without a Form 8379 claim.
    let mut owned = home.clone();
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
        ..
    } = &mut owned.disposition
    else {
        unreachable!()
    };
    *owners = Some(MortgageSubsidyGiftOwners::Spouse);
    ok(&save_other_taxes(
        &mut session,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![owned.clone()] }],
    ));
    assert_eq!(value(&supported(&session.draft), "mortgage_subsidy_recapture_tax"), 7_500);
    let stored = session.draft.clone();
    let mut again = restart(stored.clone());
    let P::SaveOtherTaxes { value: shown, mortgage_gift_owners, .. } = open_other_taxes(&mut again)
    else {
        unreachable!()
    };
    assert_eq!(mortgage_gift_owners, vec![0, 1, 2]);
    assert_eq!(shown.taxes, vec![OtherTax::MortgageSubsidyRecapture { homes: vec![owned] }]);
    ok(&save_other_taxes(&mut again, shown.taxes));
    assert_eq!(again.draft, stored);
    // Actual zero recapture permits the explicit null answer on a joint return.
    ok(&save_other_taxes(
        &mut again,
        vec![OtherTax::MortgageSubsidyRecapture {
            homes: vec![gift(20200601, 200_000, 200_000, 0, 60)],
        }],
    ));
    assert_eq!(value(&supported(&again.draft), "mortgage_subsidy_recapture_tax"), 0);
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let P::SaveOtherTaxes { mortgage_gift_owners, .. } = open_other_taxes(&mut single) else {
        unreachable!()
    };
    assert!(mortgage_gift_owners.is_empty());
    ok(&save_other_taxes(
        &mut single,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![home] }],
    ));
    assert_eq!(value(&supported(&single.draft), "mortgage_subsidy_recapture_tax"), 7_500);
    let before = single.draft.clone();
    let mut invalid = before.credits.as_ref().unwrap().mortgage_subsidies[0].clone();
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
        ..
    } = &mut invalid.disposition
    else {
        unreachable!()
    };
    *owners = Some(MortgageSubsidyGiftOwners::Spouse);
    refused_key(
        save_other_taxes(
            &mut single,
            vec![OtherTax::MortgageSubsidyRecapture { homes: vec![invalid] }],
        ),
        "MORTGAGE_GIFT_OWNER_NOT_JOINT",
    );
    assert_eq!(single.draft, before);
}

// 2026 W-2 instructions and draft Schedule 2: A/B/M/N report tax.
// Combine both supported common destinations so their separate Schedule 2
// subtotals survive ordinary native editors and persisted-record restart.
#[test]
fn schedule_2_common_w2_codes_and_8828_native_save_reopen_restart() {
    use crate::tests::cross::interview::open_topic;
    use crate::tests::support::client::exchange;
    use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

    let wages = NativeInterviewCase("TAX_UNIT18_COMMON_W2");
    let taxes = NativeInterviewCase("TAX_UNIT18_COMMON_TAXES");
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut request = open_topic(&mut session, TaxSection::Wages);
    wages.write("SEED", &request);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::W2(w2) = &mut documents.documents[0].document else { unreachable!() };
    // $60,000 covered wages: ordinary SS $3,720 and Medicare $870.
    // Reported uncollected tax is A40+B10+M20+N5=$75; reduce only the
    // respective payroll withholding boxes, leaving box2=$6,500 unchanged.
    w2.social_tax -= 6_000;
    w2.medicare_tax -= 1_500;
    w2.more.box12 = [(W2Code::A, 4_000), (W2Code::B, 1_000), (W2Code::M, 2_000), (W2Code::N, 500)]
        .into_iter()
        .map(|(code, amount)| Box12Entry { code, amount })
        .collect();
    wages.save(&mut session, request);

    let mut request = open_other_taxes(&mut session);
    taxes.write("SEED", &request);
    let P::SaveOtherTaxes { value: answers, .. } = &mut request else { unreachable!() };
    // Gift gain $100,000: 50% gain cap=$50,000. Principal $200,000
    // x6.25% x60% holding x100% income factor=$7,500, the smaller cap
    // (Form 8828 lines 14 and 19-23).
    answers.taxes = vec![OtherTax::MortgageSubsidyRecapture {
        homes: vec![gift(20200601, 300_000, 200_000, 0, 60)],
    }];
    taxes.save(&mut session, request);
    let result = supported(&session.draft);
    for (id, line, dollars) in [
        ("schedule_2_mortgage_subsidy_recapture", "13b", 7_500),
        ("schedule_2_other_taxes_total", "14", 7_500),
        ("schedule_2_income_and_other_taxes_subtotal", "15", 7_500),
        ("schedule_2_uncollected_tip_tax", "17c", 75),
        ("schedule_2_employment_tax_subtotal", "17d", 75),
        ("schedule_2_employment_and_other_total", "20", 75),
        ("schedule_2_total_additional_taxes", "21", 7_575),
    ] {
        assert_eq!(printed_line(&result, id), (TaxForm::Schedule2, line.into()));
        assert_eq!(value(&result, id), dollars);
    }
    assert_eq!(return_line(&result, "25a"), 650_000);
    assert_eq!(return_line(&result, "25c"), 0);
    assert_eq!(result.tax, Some(1_259_800)); // ordinary $5,023 + $7,575

    let mut invalid = open_topic(&mut session, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    let IncomeDocument::W2(w2) = &mut documents.documents[0].document else { unreachable!() };
    w2.withholding = w2.wages + 1;
    let before_invalid = session.draft.clone();
    refused_key(
        exchange(&mut session, invalid, "common Schedule 2 invalid W-2"),
        "W2_WITHHOLDING_ABOVE_WAGES",
    );
    assert_eq!(session.draft, before_invalid);
    // Correct the first invalid submitted home without replacing accepted
    // source records. Both failures are schema-valid native request values.
    for field in ["label", "disposition"] {
        let mut invalid = open_other_taxes(&mut session);
        let P::SaveOtherTaxes { value: answers, .. } = &mut invalid else { unreachable!() };
        let OtherTax::MortgageSubsidyRecapture { homes } = &mut answers.taxes[0] else {
            unreachable!()
        };
        if field == "label" {
            homes[0].label = " ".into();
        } else {
            let MortgageSubsidyDisposition::Ordinary { facts, .. } = &mut homes[0].disposition
            else {
                unreachable!()
            };
            facts.original_closed_on = 20260601; // After the recorded disposition.
        }
        let dirty = session.dirty;
        let Reply::Response(P::Problem { message, field: path }) =
            exchange(&mut session, invalid, "common Schedule 2 invalid home")
        else {
            panic!("expected located Form 8828 source refusal")
        };
        assert_eq!(
            Message::decode(&message).key,
            "MESSAGE_FORM_8828_ENTER_A_2026_DISPOSITION_DATE_AN_ORIGINAL_LOAN_CLOSED_AFTER_1990_A"
        );
        assert_eq!(
            path.as_deref(),
            Some(alloc::format!("[value/taxes/0/homes/0/{field}]").as_str())
        );
        assert_eq!(session.draft, before_invalid);
        assert_eq!(session.dirty, dirty);
    }
    let saved = taxes.restart(&mut session);
    assert_eq!(supported(&session.draft), result);
    let reopened = open_topic(&mut session, TaxSection::Wages);
    wages.write("REOPENED", &reopened);
    ok(&send(&mut session, reopened));
    let reopened = open_other_taxes(&mut session);
    taxes.write("REOPENED", &reopened);
    ok(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
}
