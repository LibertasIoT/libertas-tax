// Unit u5-losses: the pages for the earlier-year operating losses (Form 172,
// Schedule 1 line 8a; Form 6251 line 2f) and Form 8615's parent facts,
// reached through the interview. Every expected amount is worked by hand
// from the 2026 draft instructions (i172, i6251, i8615) in its comment.
use super::*;
use crate::tests::cross::interview::{
    accepted, document, from_review, open_topic, page_documents, rejected_with, save_topic,
};

// ---------------------------------------------------------------------------
// Earlier-year operating losses: a kind of Self-employment and Farming.

fn nol(year: u32, dollars: i64) -> NolYear {
    NolYear {
        ptp_ebl: crate::PtpEblNol::NoPtpEbl,
        year,
        balance: dollars * 100,
        injured_owners: None,
        research_parts: None,
    }
}
fn atnol(year: u32, dollars: i64, special_exception: bool) -> AtnolYear {
    AtnolYear { year, balance: dollars * 100, special_exception }
}
fn losses(regular: Vec<NolYear>, amt: Vec<AtnolYear>) -> LossCarryforward {
    LossCarryforward {
        regular: regular.into_iter().map(Into::into).collect(),
        amt,
        source: LossSource::Business,
    }
}
// A single filer with one W-2 of `wages` dollars.
fn single_with_wages(wages: i64) -> Interview {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    Interview::new(TEST_USER, d)
}
fn save_losses(s: &mut Interview, topic: TaxSection, value: LossCarryforward) -> Reply {
    let request = save_topic(
        &s.draft.clone(),
        topic,
        vec![document(0, IncomeDocument::NetOperatingLoss(value))],
    );
    send(s, request)
}

#[test]
fn operating_loss_kind_matches_hand_computed_form_172_and_6251() {
    // Single, W-2 $100,000, standard deduction $16,100 (2026).
    // Loss years left entering 2026: 2016 $4,000 (pre-2018), 2021 $60,000
    // and 2023 $20,000 (post-2017); AMT loss 2021 $30,000.
    //
    // i172 "NOL deduction limitation": pre-2018 losses + the lesser of the
    // post-2017 losses or 80% of (taxable income without the NOL and QBI
    // deductions - pre-2018 losses).
    //   Taxable income before the NOL: 100,000 - 16,100 = 83,900.
    //   80% x (83,900 - 4,000) = 63,920 < 80,000, so post-2017 = 63,920.
    //   Schedule 1 line 8a = -(4,000 + 63,920) = -67,920.
    //   AGI 100,000 - 67,920 = 32,080; taxable 32,080 - 16,100 = 15,980.
    //   Tax Table 15,950-16,000 (midpoint 15,975): 1,240 + 12% x 3,575 =
    //   1,669.
    // Carryforward (oldest first): 2016 uses 4,000; 2021 uses 60,000; 2023
    // uses 67,920 - 64,000 = 3,920 and carries 16,080 to 2027.
    // Form 6251 line 2f: AMTI without the ATNOLD adds back the standard
    // deduction and the regular NOL: 15,980 + 16,100 + 67,920 = 100,000;
    // the ATNOLD is the smaller of 30,000 and 90% x 100,000 = 30,000. AMTI
    // 70,000 is under the $90,100 exemption: no AMT.
    let mut s = single_with_wages(100_000);
    let mut entered = losses(
        vec![nol(2016, 4_000), nol(2021, 60_000), nol(2023, 20_000)],
        vec![atnol(2021, 30_000, false)],
    );
    // D-2026-10-15-13: distinct retained years own cardinality. These
    // already-expired $1 sources preserve every current financial oracle.
    entered.regular.extend((1900..1998).map(|year| nol(year, 1).into()));
    entered.amt.extend((1900..2000).map(|year| atnol(year, 1, false)));
    let before = s.draft.clone();
    let dirty = s.dirty;
    for alternative in [false, true] {
        let mut bad = entered.clone();
        let field = if alternative {
            bad.amt[100].year = bad.amt[99].year;
            "amt"
        } else {
            bad.regular[100].year = bad.regular[99].year;
            "regular"
        };
        let request = save_topic(
            &s.draft,
            TaxSection::SelfEmployment,
            vec![document(0, IncomeDocument::NetOperatingLoss(bad))],
        );
        let estimate = super::support::client::estimate_documents_from_save(&request);
        for request in [estimate, request] {
            let Reply::Response(P::Problem { message, field: actual }) = send(&mut s, request)
            else {
                panic!("duplicate retained loss year")
            };
            assert_eq!(message, libertas::libertas_formatted_text("NOL_YEAR_TWICE", &[]));
            assert_eq!(
                actual.as_deref(),
                Some(alloc::format!("[documents/documents/0/document/{field}/100/year]").as_str())
            );
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        }
    }
    let request = save_topic(
        &s.draft,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeDocument::NetOperatingLoss(entered.clone()))],
    );
    assert!(matches!(
        send(&mut s, super::support::client::estimate_documents_from_save(&request)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    accepted(&save_losses(&mut s, TaxSection::SelfEmployment, entered.clone()));
    let screening = s.draft.screening.as_ref().unwrap();
    assert!(matches!(screening.nol, NolHistory::Records { confirmed: true, .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_nol_deduction"), 67_920);
    assert_eq!(value(&r, "agi"), 32_080);
    assert_eq!(value(&r, "taxable_income"), 15_980);
    assert_eq!(value(&r, "regular_income_tax"), 1_669);
    assert_eq!(value(&r, "nol_2016_used"), 4_000);
    assert_eq!(value(&r, "nol_2021_used"), 60_000);
    assert_eq!(value(&r, "nol_2023_used"), 3_920);
    assert_eq!(value(&r, "nol_2023_carryforward"), 16_080);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 16_080);
    assert_eq!(value(&r, "prior_atnol_deduction"), 30_000);
    assert_eq!(value(&r, "atnol_2021_carryforward"), 0);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    // 2026-10-16, Person attribution / document_refund_management_needed:
    // supporting history derives known-no; fresh document() still starts None.
    // Round trip: the page reopens the same item, after a restart too.
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::SelfEmployment)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::NetOperatingLoss(entered.clone()))
        }]
    );
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    assert_eq!(
        page_documents(&open_topic(&mut again, TaxSection::SelfEmployment)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::NetOperatingLoss(entered))
        }]
    );
    let saved = again.draft.clone();
    let reopened = open_topic(&mut again, TaxSection::SelfEmployment);
    accepted(&send(&mut again, reopened));
    assert_eq!(again.draft, saved);
    assert_eq!(supported(&again.draft), r);
    // Leaving it out is no history.
    let before = s.draft.clone();
    accepted(&send(&mut s, save_topic(&before, TaxSection::SelfEmployment, vec![])));
    let screening = s.draft.screening.as_ref().unwrap();
    assert_eq!(screening.nol, NolHistory::None);
    assert_eq!(screening.atnol, AtnolHistory::None);
}

#[test]
fn post_2017_losses_below_the_80_percent_limit_are_deducted_in_full() {
    // W-2 $100,000: taxable before the NOL 83,900; 80% = 67,120 > 10,000,
    // so the whole 2022 loss is deducted: AGI 90,000, nothing carried.
    let mut s = single_with_wages(100_000);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![nol(2022, 10_000)], vec![]),
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_nol_deduction"), 10_000);
    assert_eq!(value(&r, "agi"), 90_000);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 0);
}

#[test]
fn post_2017_losses_stop_at_80_percent_of_taxable_income() {
    // W-2 $60,000: taxable before the NOL 43,900; 80% = 35,120 < 50,000.
    // Deduction 35,120; AGI 24,880; 14,880 carries to 2027.
    let mut s = single_with_wages(60_000);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![nol(2021, 50_000)], vec![]),
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_nol_deduction"), 35_120);
    assert_eq!(value(&r, "agi"), 24_880);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 14_880);
}

#[test]
fn pre_2018_losses_are_not_limited_to_taxable_income() {
    // W-2 $60,000, a 2017 loss of $50,000: the pre-2018 deduction is the
    // whole 50,000 (AGI 10,000, taxable 0), but it absorbs only the 43,900
    // of taxable income; 6,100 carries to 2027.
    let mut s = single_with_wages(60_000);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![nol(2017, 50_000)], vec![]),
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_nol_deduction"), 50_000);
    assert_eq!(value(&r, "agi"), 10_000);
    assert_eq!(value(&r, "taxable_income"), 0);
    assert_eq!(value(&r, "nol_2017_used"), 43_900);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 6_100);
}

#[test]
fn a_2006_loss_is_used_in_the_year_and_its_rest_expires() {
    // Pre-2018 losses carry 20 years: a 2006 loss is deductible in 2026
    // and its unabsorbed 6,100 expires; a 2005 loss already expired.
    let mut s = single_with_wages(60_000);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![nol(2005, 5_000), nol(2006, 50_000)], vec![]),
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_nol_deduction"), 50_000);
    assert_eq!(value(&r, "nol_2005_used"), 0);
    assert_eq!(value(&r, "nol_2005_expired"), 5_000);
    assert_eq!(value(&r, "nol_2006_used"), 43_900);
    assert_eq!(value(&r, "nol_2006_expired"), 6_100);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 0);
}

#[test]
fn amt_loss_stops_at_90_percent_of_amti() {
    // W-2 $100,000 with no regular loss: AMTI without the ATNOLD is
    // 83,900 + 16,100 = 100,000; a 2021 AMT loss of 200,000 deducts
    // 90% = 90,000 and carries 110,000.
    let mut s = single_with_wages(100_000);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![], vec![atnol(2021, 200_000, false)]),
    ));
    assert_eq!(s.draft.screening.as_ref().unwrap().nol, NolHistory::None);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "prior_atnol_deduction"), 90_000);
    assert_eq!(value(&r, "atnol_2021_carryforward"), 110_000);
}

#[test]
fn operating_loss_rules_are_checked_on_both_sides() {
    let mut s = single_with_wages(60_000);
    let before = s.draft.clone();
    // NOL_NOTHING_ENTERED: neither list has a year.
    rejected_with(
        save_losses(&mut s, TaxSection::SelfEmployment, losses(vec![], vec![])),
        "NOL_NOTHING_ENTERED",
    );
    // NOL_YEAR_TWICE: a year twice in one list.
    rejected_with(
        save_losses(
            &mut s,
            TaxSection::SelfEmployment,
            losses(vec![nol(2021, 1_000), nol(2021, 2_000)], vec![]),
        ),
        "NOL_YEAR_TWICE",
    );
    // ATNOL_EXCEPTION_AFTER_2017: the documented exception is pre-2018 only;
    // 2017 is accepted.
    rejected_with(
        save_losses(
            &mut s,
            TaxSection::SelfEmployment,
            losses(vec![], vec![atnol(2018, 1_000, true)]),
        ),
        "ATNOL_EXCEPTION_AFTER_2017",
    );
    assert_eq!(s.draft, before);
    accepted(&save_losses(
        &mut s,
        TaxSection::SelfEmployment,
        losses(vec![], vec![atnol(2017, 1_000, true)]),
    ));
    // NOL_TWICE: the kind once for the return.
    let current = s.draft.clone();
    let entered = losses(vec![nol(2021, 1_000)], vec![]);
    rejected_with(
        send(
            &mut s,
            save_topic(
                &current,
                TaxSection::SelfEmployment,
                vec![document(0, IncomeDocument::NetOperatingLoss(entered.clone())); 2],
            ),
        ),
        "NOL_TWICE",
    );
}

#[test]
fn something_else_leaves_out_both_loss_histories_with_notices() {
    let mut s = single_with_wages(60_000);
    let mut entered = losses(vec![nol(2021, 10_000)], vec![atnol(2021, 10_000, false)]);
    entered.source = LossSource::SomethingElse;
    accepted(&save_losses(&mut s, TaxSection::SelfEmployment, entered));
    // The regular and AMT loss histories are both left out with a notice
    // (Answers and unknowns); the estimate proceeds.
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_NOL_LEFT_OUT"), "{r:?}");
    assert!(noted(&r, "WARN_ATNOL_LEFT_OUT"), "{r:?}");
}

#[test]
fn the_farming_topic_shares_the_operating_loss_kind() {
    let mut s = single_with_wages(60_000);
    let entered = losses(vec![nol(2022, 10_000)], vec![]);
    accepted(&save_losses(&mut s, TaxSection::Farming, entered.clone()));
    assert!(s.draft.screening.as_ref().unwrap().nol.present());
    assert!(page_documents(&open_topic(&mut s, TaxSection::SelfEmployment)).contains(
        &IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::NetOperatingLoss(entered))
        }
    ));
}
pub(in crate::tests) fn parent_request(s: &Interview, value: ParentReturn) -> P {
    P::SaveParentTax {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: TaxSection::ParentTax as i32,
        previous: TaxSection::OtherTaxes as i32,
        review: true,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        value,
    }
}
pub(in crate::tests) fn save_parent(s: &mut Interview, value: ParentReturn) -> Reply {
    let request = parent_request(s, value);
    send(s, request)
}
pub(in crate::tests) fn parent(status: FederalStatus, taxable: i64) -> ParentReturn {
    ParentReturn {
        foreign_excluded: None,
        status,
        taxable: taxable * 100,
        gains: None,
        others: vec![],
        special: ParentSituation::Nothing,
    }
}
// A 16-year-old dependent with `interest` dollars of interest and no
// parent facts yet.
pub(in crate::tests) fn young_with_interest(interest: i64) -> FederalDraft {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = None;
    d.income[0].income = savings_income(interest, 0);
    d
}

#[test]
fn more_than_2700_of_interest_opens_the_parent_page_after_other_taxes() {
    // Form 8615 applies above $2,700 of unearned income (i8615 "Who Must
    // File"): a completed return with $2,000 does not ask the page; raising
    // the 1099-INT to $10,000 opens it next, before the payments page.
    let d = young_with_interest(2_000);
    let mut s = Interview::new(TEST_USER, d);
    assert!(!crate::interview::parent_tax_in_walk(&s.draft));
    let Reply::Edit(mut request) = from_review(&mut s, TaxSection::InterestDividends) else {
        panic!("expected the interest page")
    };
    let P::SaveIncomes { documents, .. } = &mut request else {
        panic!("expected the interest page")
    };
    let IncomeDocument::Int1099(int) = &mut documents.documents[0].document else {
        panic!("expected a 1099-INT")
    };
    int.taxable = 1_000_000;
    let reply = send(&mut s, request);
    assert!(matches!(&reply, Reply::Edit(P::SaveParentTax { .. })), "{reply:?}");
    assert!(crate::interview::parent_tax_in_walk(&s.draft));
    let r = estimate(&s.draft);
    assert!(r.issues.iter().any(|i| i.message.contains("Form 8615 lines 6 to 8")), "{r:?}");
    let review = match from_review(&mut s, TaxSection::Review) {
        Reply::Response(P::Review { remaining_items, .. }) => remaining_items,
        _ => vec![],
    };
    assert!(
        review.iter().any(|i| i.section == TaxSection::ParentTax
            && status_text(&i.item) == crate::kiddie::SECTION),
        "{review:?}"
    );
}

#[test]
fn a_brother_on_the_parent_page_shares_the_parent_rate() {
    // Parent single, taxable $50,000; a brother's Form 8615 line 5 $3,000.
    //  6 50,000  7 3,000  8 50,000 + 7,300 + 3,000 = 60,300
    //  9 Tax Table 60,300-60,350 (midpoint 60,325): 1,240 + 4,560 +
    //    22% x 9,925 (2,183.50) = 7,983.50, 7,984
    // 10 Tax Table 50,000-50,050 (midpoint 50,025): 1,240 + 12% x 37,625
    //    (4,515) = 5,755
    // 11 2,229   12a 7,300  12b 10,300  12c 0.7087
    // 13 2,229 x 0.7087 = 1,579.69, 1,580   14 136   15 1,716
    // 16 868   17 1,716
    let mut s = Interview::new(TEST_USER, young_with_interest(10_000));
    let mut entered = parent(FederalStatus::Single, 50_000);
    entered.others.push(OtherChildTax { label: "Brother".into(), net: 300_000, gains: None });
    accepted(&save_parent(&mut s, entered));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1_580);
    assert_eq!(value(&r, "regular_income_tax"), 1_716);
}

#[test]
fn unit47_parent_gain_admission_rejects_oversized_source_without_overflow() {
    let mut source = ordinary_parent();
    source.gains = Some(ParentGains {
        qualified: i64::MAX,
        long_term: 1,
        combined: 1,
        collectibles: 0,
        unrecaptured: 0,
        investment_election: 0,
    });
    assert!(!source.valid());
}

#[test]
fn ptp_ebl_nol_native_later_deduction_and_same_source_correction() {
    // Reg.1.199A-3(b)(1)(iv),(v),(c)(3)(ii): the 2024 EBL retained
    // 144000 principal /108000 original qualified after the earlier limits.
    // Entering 2026 it is still tax-suspended, outside the 8995 carry.
    // 130000 wages +20000 REIT −16100 standard =133900; section172
    // deducts107120, of which80340 is the separate PTP loss. Financial
    // carry36880 and suspended qualification27660 do not enter line7 again.
    let mut history = prior_nol(&[(2024, 144_000)]);
    let NolHistory::Records { years, .. } = &mut history else { unreachable!() };
    years[0].ptp_ebl = PtpEblNol::Reported {
        original: 14_400_000,
        qualified: 10_800_000,
        remaining: 14_400_000,
        remaining_qualified: 10_800_000,
        injured_qualified: None,
    };
    let page = LossCarryforward::from_histories(&history, &AtnolHistory::None).unwrap().unwrap();
    let mut session = Interview::new(
        TEST_USER,
        ptp_ebl_nol_draft(FederalStatus::Single, 13_000_000, 2_000_000, NolHistory::None),
    );
    accepted(&save_losses(&mut session, TaxSection::SelfEmployment, page));
    let r = supported(&session.draft);
    for (key, expected) in [
        ("prior_nol_deduction", 107_120),
        ("agi", 42_880),
        ("nol_2024_ptp_qualified_deducted", 80_340),
        ("nol_2024_carryforward", 36_880),
        ("qbi_reit_ptp", -60_340),
        ("nol_2024_ptp_qualified_pre_debt_closing", 27_660),
        ("nol_2024_ptp_qualified_lost", 0),
        ("reit_ptp_loss_carryforward", 60_340),
        ("qbi_loss_carryforward", 0),
        ("qbi_deduction", 0),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    let accepted_income_ids: Vec<_> = session.draft.income.iter().map(|e| e.id).collect();
    let mut cold = restart(session.draft.clone()); // native keyed records + Avro
    assert_eq!(supported(&cold.draft), r);
    let mut correction = open_topic(&mut cold, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let id = documents.documents[0].id;
    let IncomeDocument::NetOperatingLoss(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let PtpEblNol::Reported { qualified, remaining_qualified, .. } = &mut page.regular[0].ptp_ebl
    else {
        unreachable!()
    };
    *qualified = 7_200_000;
    *remaining_qualified = 7_200_000;
    accepted(&send(&mut cold, correction));
    let corrected = supported(&cold.draft);
    assert_eq!(value(&corrected, "prior_nol_deduction"), 107_120);
    assert_eq!(value(&corrected, "agi"), 42_880);
    assert_eq!(value(&corrected, "nol_2024_ptp_qualified_deducted"), 53_560);
    assert_eq!(value(&corrected, "reit_ptp_loss_carryforward"), 33_560);
    let mut cold = restart(cold.draft);
    let reopened = open_topic(&mut cold, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(documents.documents[0].id, id);
    accepted(&send(&mut cold, reopened));
    assert_eq!(supported(&cold.draft), corrected);
    assert_eq!(cold.draft.income.iter().map(|e| e.id).collect::<Vec<_>>(), accepted_income_ids);
    let mut invalid = open_topic(&mut cold, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    let IncomeDocument::NetOperatingLoss(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let PtpEblNol::Reported { qualified, .. } = &mut page.regular[0].ptp_ebl else {
        unreachable!()
    };
    *qualified = 14_400_001;
    let before = cold.draft.clone();
    let Reply::Response(P::Problem { message, field }) = send(&mut cold, invalid) else {
        panic!("invalid original qualified portion")
    };
    assert_eq!(message, libertas::libertas_formatted_text("NOL_PTP_EBL_RECONCILE", &[]));
    assert_eq!(
        field.as_deref(),
        Some("[documents/documents/0/document/regular/0/ptp_ebl/qualified]")
    );
    assert_eq!(cold.draft, before);
}

#[test]
fn ptp_ebl_nol_uses_actual_deduction_not_absorption_or_current_sstb() {
    let mut history = prior_nol(&[(2024, 100_000)]);
    let NolHistory::Records { years, .. } = &mut history else { unreachable!() };
    years[0].ptp_ebl = PtpEblNol::Reported {
        original: 10_000_000,
        qualified: 7_500_000,
        remaining: 10_000_000,
        remaining_qualified: 7_500_000,
        injured_qualified: None,
    };
    let draft = ptp_ebl_nol_draft(FederalStatus::Single, 50_000_000, 0, history);
    let projection = crate::estate_trust::losses(&draft, false);
    // Independent financial inputs deliberately distinguish section172(a)'s
    // actual deduction40000 from section172(b)(2) absorption70000.
    let allocation = NolHistory::allocate(50_000, &projection.vintages, |_| 80_000).unwrap();
    assert_eq!(allocation.deduction, 40_000);
    assert_eq!(allocation.carryforward, crate::kernel::ratio::exact::whole(30_000));
    let parts = allocation.ptp_deductions(&draft);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].qualified, Some(3_000_000)); // not absorption52500
    assert_eq!(parts[0].owners, Some([3_000_000, 0]));
    assert_eq!(parts[0].closing, Some(2_250_000));
    assert_eq!(parts[0].lost, Some(2_250_000));
    // The original qualified fraction remains75% above the current SSTB
    // phaseout; no statement/current business eligibility is consulted.
    let result = supported(&draft);
    assert_eq!(value(&result, "nol_2024_ptp_qualified_deducted"), 75_000);
    assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 75_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
}

#[test]
fn ptp_ebl_nol_unknown_character_preserves_financial_nol_and_explicit_none() {
    let mut history = prior_nol(&[(2024, 10_000)]);
    let NolHistory::Records { years, .. } = &mut history else { unreachable!() };
    years[0].ptp_ebl = PtpEblNol::Unavailable;
    let mut draft = ptp_ebl_nol_draft(FederalStatus::Single, 10_000_000, 1_000_000, history);
    let unknown = supported(&draft);
    assert_eq!(value(&unknown, "prior_nol_deduction"), 10_000);
    assert_eq!(value(&unknown, "agi"), 100_000);
    assert_eq!(value(&unknown, "qbi_deduction"), 0);
    assert!(
        unknown.review_notes.iter().any(|note| note.display.key == "WARN_NOL_PTP_EBL_UNAVAILABLE")
    );
    let NolHistory::Records { years, .. } = &mut draft.screening.as_mut().unwrap().nol else {
        unreachable!()
    };
    years[0].ptp_ebl = PtpEblNol::NoPtpEbl;
    let none = supported(&draft);
    assert_eq!(value(&none, "prior_nol_deduction"), 10_000);
    assert_eq!(value(&none, "agi"), 100_000);
    assert_eq!(value(&none, "qbi_deduction"), 2_000);
    assert!(
        !none.review_notes.iter().any(|note| note.display.key == "WARN_NOL_PTP_EBL_UNAVAILABLE")
    );
}

#[test]
fn ptp_ebl_nol_original_ratio_mixed_principal_and_subdollar_conservation() {
    // All amounts below are native cents except taxable/D/used financial
    // inputs. These independent controls distinguish original33 from a
    // mistaken remaining-ratio34, and exercise the owner's rounded financial
    // boundary without inventing source cents from its dollar output.
    for (balance, original, qualified, remaining, remaining_qualified, taxable, expected) in [
        (200, 300, 100, 200, 67, 1, [1, 33, 34, 0]),
        (200, 300, 100, 200, 67, 3, [2, 67, 0, 0]),
        (400, 300, 100, 200, 67, 3, [2, 33, 34, 0]), // 200 ordinary +200 PTP
        (99, 99, 99, 99, 99, 1, [1, 99, 0, 0]),
        (149, 149, 149, 149, 149, 1, [1, 100, 0, 49]), // D equals rounded opening, not149c capacity.
        (17, 17, 17, 17, 17, 1, [0, 0, 0, 17]),
        // Both complementary halves round up individually. One consumed
        // projection plus its residual conserves the original qualified cent.
        (200, 200, 1, 200, 1, 1, [1, 1, 0, 0]),
    ] {
        let history = NolHistory::Records {
            confirmed: true,
            years: vec![NolYear {
                year: 2024,
                balance,
                injured_owners: None,
                research_parts: None,
                ptp_ebl: PtpEblNol::Reported {
                    original,
                    qualified,
                    remaining,
                    remaining_qualified,
                    injured_qualified: None,
                },
            }],
        };
        let draft = ptp_ebl_nol_draft(FederalStatus::Single, 10_000_000, 0, history);
        let projection = crate::estate_trust::losses(&draft, false);
        let allocation =
            NolHistory::allocate(taxable, &projection.vintages, |_| expected[0]).unwrap();
        assert_eq!(allocation.deduction, expected[0]);
        let parts = allocation.ptp_deductions(&draft);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].qualified, Some(expected[1]));
        assert_eq!(parts[0].closing, Some(expected[2]));
        assert_eq!(parts[0].lost, Some(expected[3]));
        assert_eq!(expected[1] + expected[2] + expected[3], remaining_qualified);
    }
}

#[test]
fn ptp_ebl_nol_later_supplied_remaining_and_8995_carry_stay_separate() {
    // A synthetic reenactment of actual intervening-year source records,
    // NOT an automatic next-year return or newly invented filing route.
    // The 144000/108000 original cohort now has36880/27660 left. Its
    // earlier60340 tax-allowed8995 carry is a different input. With20000
    // currentREIT, currentPTPloss27660 +prior8995loss60340 =>68000 carry.
    let history = NolHistory::Records {
        confirmed: true,
        years: vec![NolYear {
            year: 2024,
            balance: 3_688_000,
            injured_owners: None,
            research_parts: None,
            ptp_ebl: PtpEblNol::Reported {
                original: 14_400_000,
                qualified: 10_800_000,
                remaining: 3_688_000,
                remaining_qualified: 2_766_000,
                injured_qualified: None,
            },
        }],
    };
    let mut draft = ptp_ebl_nol_draft(FederalStatus::Single, 10_000_000, 2_000_000, history);
    draft.screening.as_mut().unwrap().qbi_history.reit_ptp = 6_034_000;
    let result = supported(&draft);
    assert_eq!(value(&result, "prior_nol_deduction"), 36_880);
    assert_eq!(value(&result, "nol_2024_ptp_qualified_deducted"), 27_660);
    assert_eq!(value(&result, "nol_2024_ptp_qualified_pre_debt_closing"), 0);
    assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 68_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
}

#[test]
fn ptp_ebl_nol_pooled_estate_first_origin_and_qualified_owner_are_independent() {
    let history = NolHistory::Records {
        confirmed: true,
        years: vec![NolYear {
            year: 2024,
            balance: 200,
            research_parts: None,
            injured_owners: Some(InjuredNolOwners::Shares { taxpayer: 100, spouse: 100 }),
            ptp_ebl: PtpEblNol::Reported {
                original: 300,
                qualified: 100,
                remaining: 200,
                remaining_qualified: 67,
                injured_qualified: None,
            },
        }],
    };
    let mut draft = ptp_ebl_nol_draft(FederalStatus::Joint, 10_000_000, 0, history);
    let mut estate = empty_estate_portfolio();
    estate.final_nol = Some(FinalEstateTrustNol {
        terminated_on: Some(20261231),
        fiduciary_history_complete: true,
        years: vec![FinalEstateTrustNolYear {
            year: 2024,
            original_period: Some(FinalEstateTrustLossPeriod { began: 20240101, ended: 20241231 }),
            remaining_years: None,
            regular_balance: 200,
            alternative_balance: 0,
        }],
        taxable_periods: (2024..=2026)
            .map(|year| FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod {
                    began: year * 10000 + 101,
                    ended: year * 10000 + 1231,
                },
                accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
            })
            .collect(),
    });
    add(&mut draft, income(FederalIncome::EstateTrustPortfolio { data: estate }));
    let mut sources = crate::estate_trust::loss_vintages(&draft, false);
    sources.reverse(); // Pool keeps estate first; that is not personal ownership.
    let projection = crate::estate_trust::pooled_loss_vintages(sources);
    assert_eq!(projection.vintages.len(), 1);
    assert!(projection.vintages[0].origin.is_some());
    let allocation = NolHistory::allocate(3, &projection.vintages, |_| 2).unwrap();
    let parts = allocation.ptp_deductions(&draft);
    // Joint deduction200c; actual PTP principal200/total400 gives100c;
    // original qualification100/300 gives33c, never totalcohort denominator.
    assert_eq!(parts[0].qualified, Some(33));
    assert_eq!(parts[0].closing, Some(34));
    assert_eq!(parts[0].owners, None); // Financial100/100 cannot identify q subset.
    let NolHistory::Records { years, .. } = &mut draft.screening.as_mut().unwrap().nol else {
        unreachable!()
    };
    let PtpEblNol::Reported { injured_qualified, .. } = &mut years[0].ptp_ebl else {
        unreachable!()
    };
    *injured_qualified = Some(InjuredNolOwners::Shares { taxpayer: 40, spouse: 27 });
    let parts = allocation.ptp_deductions(&draft);
    assert_eq!(parts[0].qualified, Some(33));
    assert_eq!(parts[0].owners, None); // Unequal surviving q cannot establish original owner ratios.
    let full = NolHistory::allocate(5, &projection.vintages, |_| 4).unwrap();
    assert_eq!(full.ptp_deductions(&draft)[0].owners, Some([40, 27]));
    let NolHistory::Records { years, .. } = &mut draft.screening.as_mut().unwrap().nol else {
        unreachable!()
    };
    let PtpEblNol::Reported { injured_qualified, .. } = &mut years[0].ptp_ebl else {
        unreachable!()
    };
    *injured_qualified = Some(InjuredNolOwners::Shares { taxpayer: 0, spouse: 67 });
    let parts = allocation.ptp_deductions(&draft);
    assert_eq!(parts[0].qualified, Some(33));
    assert_eq!(parts[0].owners, Some([0, 33]));
    // At the native-cent boundary,148.6c exports149c and releases the
    // whole149c qualified stock. Its actual70/79 owners are then known.
    let NolHistory::Records { years, .. } = &mut draft.screening.as_mut().unwrap().nol else {
        unreachable!()
    };
    years[0].balance = 201;
    years[0].injured_owners = Some(InjuredNolOwners::Shares { taxpayer: 100, spouse: 101 });
    years[0].ptp_ebl = PtpEblNol::Reported {
        original: 1000,
        qualified: 743,
        remaining: 201,
        remaining_qualified: 149,
        injured_qualified: Some(InjuredNolOwners::Shares { taxpayer: 70, spouse: 79 }),
    };
    draft
        .income
        .retain(|entry| !matches!(&entry.income, FederalIncome::EstateTrustPortfolio { .. }));
    let projection = crate::estate_trust::losses(&draft, false);
    let allocation = NolHistory::allocate(3, &projection.vintages, |_| 3).unwrap();
    let parts = allocation.ptp_deductions(&draft);
    assert_eq!(parts[0].qualified, Some(149));
    assert_eq!(parts[0].owners, Some([70, 79]));
}

#[test]
fn ptp_ebl_nol_unknown_component_preserves_independent_ordinary_qbi() {
    for (profit, deduction, carry) in [(10_000, 1_859, 0), (-1_000, 0, 1_000)] {
        let mut history = prior_nol(&[(2024, 10_000)]);
        let NolHistory::Records { years, .. } = &mut history else { unreachable!() };
        years[0].ptp_ebl = PtpEblNol::Unavailable;
        let mut draft = ptp_ebl_nol_draft(FederalStatus::Single, 10_000_000, 1_000_000, history);
        add(&mut draft, income(cash_business(profit)));
        let result = supported(&draft);
        // Positive business10000 −halfSE707 =9293 QBI;20%=1859.
        // The negative business's1000 remains its ordinary QBI carry.
        assert_eq!(value(&result, "qbi_deduction"), deduction);
        assert_eq!(value(&result, "qbi_loss_carryforward"), carry);
        assert!(!result.lines.iter().any(|line| line.id == "reit_ptp_loss_carryforward"));
        assert!(!result.lines.iter().any(|line| {
            line.form_line
                .as_ref()
                .is_some_and(|f| matches!(f.form, TaxForm::Form8995 | TaxForm::Form8995A))
        }));
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.id == "qbi_loss_carryforward" && line.form_line.is_none())
        );
        assert!(
            result
                .review_notes
                .iter()
                .any(|note| note.display.key == "WARN_NOL_PTP_EBL_UNAVAILABLE")
        );
    }
}

#[test]
fn ptp_ebl_nol_pre_2018_unavailable_character_does_not_gate_reit() {
    let mut history = prior_nol(&[(2017, 10_000)]);
    let NolHistory::Records { years, .. } = &mut history else { unreachable!() };
    years[0].ptp_ebl = PtpEblNol::Unavailable;
    let source_bytes = history.to_avro();
    let draft = ptp_ebl_nol_draft(FederalStatus::Single, 10_000_000, 1_000_000, history);
    let mut session = restart(draft);
    let reopened = open_topic(&mut session, TaxSection::SelfEmployment);
    accepted(&send(&mut session, reopened));
    let result = supported(&session.draft);
    // Pre-2018 NOL10000 is deducted; the known REIT10000 still earns2000.
    assert_eq!(value(&result, "prior_nol_deduction"), 10_000);
    assert_eq!(value(&result, "qbi_reit_ptp"), 10_000);
    assert_eq!(value(&result, "qbi_deduction"), 2_000);
    assert!(
        !result.review_notes.iter().any(|note| note.display.key == "WARN_NOL_PTP_EBL_UNAVAILABLE")
    );
    assert_eq!(session.draft.screening.as_ref().unwrap().nol.to_avro(), source_bytes);
}
