// Derived fixtures, not IRS published examples. Arithmetic follows the posted
// 2026 draft Form8801 (prior2025 rates) and draft i8801 pp1–5. Current2026
// Single60,000 W-2 less16,100 standard deduction: taxable43,900, table tax5,023;
// current TMT0. Prior2025 single exemption88,100, AMT26% below239,100.
// Standalone audit sources use the Document center; credit histories use More credits.
use super::*;
use crate::tests::cross::interview::{
    accepted, document, from_review, rejected_with, save_documents, w2,
};

fn save_audit_source(session: &mut Interview, mut audit: FederalBbaAuditInterest) {
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.credits.documents.push(crate::CreditDocument::PartnershipAudit(
        alloc::boxed::Box::new(crate::tests::support::builders::audit_row(audit.clone())),
    ));
    accepted(&send(session, request));
    // No printed partner name: the statement is pending until your section adds it.
    let id = session.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref().unwrap().id;
    crate::tests::support::builders::assign_documents(
        session,
        TaxSection::MoreCredits,
        -1,
        &[id],
        crate::AddedOwnership::Yours,
    );
    audit.id = id;
    let request = crate::tests::support::client::credit_documents::credit_document_answers_request(
        session,
        &[],
        Some(&audit),
    );
    accepted(&send(session, request));
}

pub(in crate::tests) fn source() -> MinimumTaxHistory {
    MinimumTaxHistory {
        return_form: MinimumTaxReturnForm::Form1040,
        status: FederalStatus::Single,
        base: 20_000_000,
        exclusions: vec![MinimumTaxExclusion {
            source: "Retained 2025 Form 6251 permanent adjustment".into(),
            amount: 2_000_000,
        }],
        regular: 2_000_000,
        amt: 2_000_000,
        carry: 1_000_000,
        regular_nol: 0,
        mtcnol: MinimumTaxLoss::None,
        foreign_earned: None,
        foreign_credit: MinimumTaxForeignCredit::None,
        gains: None,
        old_vehicle_credit: 0,
    }
}
pub(in crate::tests) fn session(status: FederalStatus) -> Interview {
    let mut s = Interview::new(TEST_USER, empty_return(status));
    let page = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(6_000_000))) },
        )],
    );
    accepted(&send(&mut s, page));
    s
}
pub(in crate::tests) fn request(s: &mut Interview, histories: MinimumTaxHistories) -> P {
    let Reply::Edit(mut page) = from_review(s, TaxSection::MoreCredits) else {
        panic!("expected More credits")
    };
    let P::SaveMoreCredits { value, .. } = &mut page else { panic!("expected SaveMoreCredits") };
    value.credits = vec![MoreCredit::MinimumTax(histories.into())];
    page
}
fn save(s: &mut Interview, histories: MinimumTaxHistories) -> FederalResult {
    let page = request(s, histories);
    accepted(&send(s, page));
    let r = supported(&s.draft);
    // Exercise the actual presentation labels, including typed form rows.
    for row in &r.lines {
        let label = row
            .form_line
            .as_ref()
            .map_or_else(|| crate::line_label(&row.id), |form| form.row_label(&row.id));
        assert!(!label.english().is_empty(), "{}", row.id);
    }
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
    r
}
fn expect(r: &FederalResult, index: usize, rows: &[(usize, i64)]) {
    for (number, expected) in rows {
        let id = alloc::format!("minimum_tax_history_{index}_line_{number}");
        assert_eq!(value(r, &id), *expected, "{id}");
        assert_eq!(printed_line(r, &id), (TaxForm::Form8801, alloc::format!("{number}")));
    }
}

#[test]
fn mtc_retained_loss_is_exclusion_only_without_regular_nol_percent_caps() {
    let mut h = source();
    h.regular_nol = 1_000_000;
    h.mtcnol = MinimumTaxLoss::Retained {
        records: MinimumTaxLossRecords {
            amounts: vec![MinimumTaxLossAmount { year: 2024, available: 3_000_000 }],
        },
    };
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // 210,000+20,000−30,000=200,000; −88,100=111,900;
    // *.26=29,094; −20,000=9,094 exclusion tax. 20,000−9,094+10,000=20,906.
    expect(
        &r,
        0,
        &[
            (1, 210000),
            (2, 20000),
            (3, -30000),
            (4, 200000),
            (5, 88100),
            (10, 111900),
            (11, 29094),
            (15, 9094),
            (18, 10906),
            (21, 20906),
        ],
    );
    assert_eq!(value(&r, "minimum_tax_credit"), 5023);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15883);
    assert_eq!(r.tax, Some(0));
}
#[test]
fn mtc_foreign_earned_stacking_uses_net_exclusion_once() {
    let mut h = source();
    h.base = 10_000_000;
    h.exclusions.clear();
    h.regular = 0;
    h.carry = 0;
    h.foreign_earned = Some(MinimumTaxForeignEarned {
        excluded: 10_000_000,
        related_deductions: 1_000_000,
        gain_source: MinimumTaxForeignGainSource::NoPreferentialIncome,
    });
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // 100,000−88,100=11,900; net excluded90,000; tax101,900*.26=26,494;
    // subtract tax90,000*.26=23,400 =>3,094, not ordinary tax on11,900 at other rates.
    expect(&r, 0, &[(4, 100000), (10, 11900), (11, 3094), (15, 3094), (18, 16906), (21, 16906)]);
    for (name, expected) in [
        ("net_excluded", 90000),
        ("stacked", 101900),
        ("stacked_tax", 26494),
        ("excluded_tax", 23400),
        ("tax", 3094),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("minimum_tax_history_0_foreign_earned_{name}")),
            expected
        );
    }
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 11883);
}
#[test]
fn mtc_retained_foreign_category_reuses_credit_limit_and_actual_prior_source_lines() {
    let mut h = source();
    h.foreign_credit = MinimumTaxForeignCredit::RetainedCategories {
        categories: vec![MinimumTaxForeignCategory {
            category: ForeignCreditCategory::Passive,
            income: 5_500_000,
            paid: 1_000_000,
            carry: 200_000,
            reduction: 100_000,
            reclassified: 0,
        }],
        worldwide: 22_000_000,
    };
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // TMT34,294;55,000/220,000=.25; limit8,573.50→8,574 <11,000 tax pool.
    // Exclusion tax34,294−8,574−20,000=5,720; available20,000−5,720+10,000=24,280.
    expect(&r, 0, &[(11, 34294), (12, 8574), (13, 25720), (15, 5720), (18, 14280), (21, 24280)]);
    for (number, expected) in [
        (9, 10000),
        (10, 2000),
        (11, 12000),
        (12, 1000),
        (14, 11000),
        (17, 55000),
        (18, 220000),
        (20, 34294),
        (21, 8574),
        (24, 8574),
    ] {
        let id = alloc::format!("minimum_tax_history_0_foreign_1_line_{number}");
        assert_eq!(value(&r, &id), expected);
        assert_eq!(printed_line(&r, &id), (TaxForm::Form1116, alloc::format!("{number}")));
    }
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 19257);
}
#[test]
fn mtc_qualified_dividends_use_prior_regular_band_and_round_printed_lines() {
    let mut h = source();
    h.gains = Some(MinimumTaxRateFacts::QualifiedDividends(MinimumTaxQualifiedRateFacts {
        preferential: 5_000_000,
        regular_ordinary: 4_000_000,
    }));
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // 131,900 less50,000=81,900 ordinary*.26=21,294. Zero band48,350−40,000=8,350.
    // Remaining41,650*.15=6,247.50→6,248; tax27,542; exclusion7,542; available22,458.
    expect(
        &r,
        0,
        &[
            (27, 131900),
            (28, 50000),
            (30, 50000),
            (32, 81900),
            (33, 21294),
            (34, 48350),
            (35, 40000),
            (36, 8350),
            (38, 8350),
            (45, 41650),
            (46, 6248),
            (53, 27542),
            (54, 34294),
            (55, 27542),
            (15, 7542),
            (21, 22458),
        ],
    );
    assert!(!r.lines.iter().any(|v| v.id == "minimum_tax_history_0_line_29"));
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 17435);
}
#[test]
fn mtc_two_prior_individual_returns_join_one_current_joint_limit() {
    let first = minimum_tax_history(); //10,000 prior AMT+5,000 carry; exclusion tax0.
    let mut second = first.clone();
    second.amt = 100_000;
    second.carry = 0;
    let histories = MinimumTaxHistories {
        returns: vec![
            OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner: Owner::Taxpayer },
                facts: first,
            },
            OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner: Owner::Spouse },
                facts: second,
            },
        ],
    };
    let r = save(&mut session(FederalStatus::Joint), histories);
    // Current joint taxable27,800; tax table midpoint27,825:2,480+3,025*.12=2,843.
    expect(&r, 0, &[(21, 15000)]);
    expect(&r, 1, &[(21, 1000)]);
    assert_eq!(value(&r, "minimum_tax_current_regular_liability"), 2843);
    assert_eq!(value(&r, "minimum_tax_credit_available"), 16000);
    assert_eq!(value(&r, "minimum_tax_credit"), 2843);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 13157);
    // The two actual prior-return worksheets must not interleave by printed
    // line number. Preserve each complete copy before the joint current limit.
    let review: Vec<_> = crate::draft_return(&r)
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form8801))
        .collect();
    let starts: Vec<_> = review
        .iter()
        .enumerate()
        .filter_map(|(index, row)| (row.form_line.as_ref().unwrap().line == "1").then_some(index))
        .collect();
    assert_eq!(starts.len(), 2);
    assert!(
        review[starts[0]..starts[1]]
            .iter()
            .any(|row| { row.form_line.as_ref().unwrap().line == "21" && row.amount == 1_500_000 })
    );
    assert!(
        review[starts[1]..]
            .iter()
            .any(|row| { row.form_line.as_ref().unwrap().line == "21" && row.amount == 100_000 })
    );
}
#[test]
fn mtc_unavailable_nonresident_and_former_joint_preserve_ordinary_tax() {
    for (unit, form, key) in [
        (
            MinimumTaxHistoryUnit::Individual { owner: Owner::Taxpayer },
            MinimumTaxReturnForm::Form1040Nr,
            "WARN_MTC_NONRESIDENT_LEFT_OUT",
        ),
        (
            MinimumTaxHistoryUnit::FormerJoint { claimant: Owner::Taxpayer },
            MinimumTaxReturnForm::Form1040,
            "WARN_MTC_OWNERSHIP_LEFT_OUT",
        ),
    ] {
        let mut h = source();
        h.return_form = form;
        if matches!(unit, MinimumTaxHistoryUnit::FormerJoint { .. }) {
            h.status = FederalStatus::Joint;
        }
        let r = save(
            &mut session(FederalStatus::Single),
            MinimumTaxHistories { returns: vec![OwnedMinimumTaxHistory { unit, facts: h }] },
        );
        assert!(noted(&r, key));
        assert_eq!(r.tax, Some(502300));
        assert!(!r.lines.iter().any(|v| v.id == "minimum_tax_credit"));
    }
}
#[test]
fn mtc_public_failed_save_rejects_same_year_loss_and_duplicate_source_without_mutation() {
    let mut s = session(FederalStatus::Single);
    let mut h = source();
    h.mtcnol = MinimumTaxLoss::Retained {
        records: MinimumTaxLossRecords {
            amounts: vec![MinimumTaxLossAmount { year: 2025, available: 100000 }],
        },
    };
    let page = request(&mut s, MinimumTaxHistories::from_current_unit(h));
    let before = s.draft.clone();
    let dirty = s.dirty;
    rejected_with(send(&mut s, page), "MTC_NOL_SAME_YEAR");
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    let first = OwnedMinimumTaxHistory {
        unit: MinimumTaxHistoryUnit::Individual { owner: Owner::Taxpayer },
        facts: source(),
    };
    let page = request(&mut s, MinimumTaxHistories { returns: vec![first.clone(), first] });
    let before = s.draft.clone();
    let dirty = s.dirty;
    rejected_with(send(&mut s, page), "MTC_PRIOR_RETURN_TWICE");
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
}

#[test]
fn mtc_negative_line_21_stops_typed_part_two_without_changing_current_tax() {
    let mut h = source();
    h.regular = 0;
    h.amt = 1_000_000;
    h.carry = 500_000;
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // Prior 220,000 less88,100 =131,900; *.26=34,294 exclusion tax.
    // 10,000−34,294=−24,294; +5,000 carry =−19,294 on signedline21.
    expect(&r, 0, &[(15, 34294), (18, -24294), (19, 5000), (21, -19294)]);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 0);
    assert_eq!(r.tax, Some(502300));
    assert!(!r.lines.iter().any(|row| {
        row.form_line.as_ref().is_some_and(|line| {
            line.form == TaxForm::Form8801
                && ["22", "23", "24", "25", "26"].contains(&line.line.as_str())
        })
    }));
}

#[test]
fn mtc_page_cannot_supply_a_missing_current_filer_through_an_echoed_owner_catalog() {
    let mut s = session(FederalStatus::Single);
    let mut page = request(
        &mut s,
        MinimumTaxHistories {
            returns: vec![OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner: Owner::Spouse },
                facts: source(),
            }],
        },
    );
    let P::SaveMoreCredits { allowed_owners, .. } = &mut page else {
        panic!("More credits");
    };
    *allowed_owners = vec![Owner::Taxpayer as i32, Owner::Spouse as i32];
    let before = s.draft.clone();
    let dirty = s.dirty;
    rejected_with(send(&mut s, page), "MTC_OWNER_NOT_ON_RETURN");
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
}

// Independent derived current-year coordination: the published negative 8978
// worksheet reserves S3 6l; draft8801 line22 then subtracts allowable credits.
// All source facts use ordinary Wages and More credits saves.
#[test]
fn mtc_negative_8978_and_energy_credits_use_one_capacity_without_duplicate_decrease() {
    let mut s = session(FederalStatus::Single);
    save_audit_source(&mut s, bba_interest_reduction());
    let mut page = request(&mut s, MinimumTaxHistories::from_current_unit(source()));
    let P::SaveMoreCredits { value: chosen, .. } = &mut page else { unreachable!() };
    chosen.credits.extend([MoreCredit::PriorCreditCarryovers {
        residential_energy_carry: Some(100_000),
        dc_homebuyer_carry: Some(50_000),
    }]);
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    // Current43,900 taxable -> table tax5,023; 8978 prior5,920−5,700=220.
    // DC500,energy1,000 leave8801 line22=5,023−220−500−1,000=3,303.
    // Prior balance15,706−3,303=12,403 carry. Current credits sum5,023.
    assert_eq!(value(&r, "schedule_3_line_6l"), 220);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 500);
    assert_eq!(value(&r, "residential_energy_credit"), 1000);
    assert_eq!(value(&r, "minimum_tax_credit"), 3303);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 12403);
    assert_eq!(value(&r, "minimum_tax_current_regular_liability"), 3303);
    assert_eq!(return_line(&r, "20"), 502_300);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn mtc_negative_8978_reduces_general_business_capacity_before_the_mtc_limit() {
    let mut s = session(FederalStatus::Single);
    save_audit_source(&mut s, bba_interest_reduction());
    let mut page = request(&mut s, MinimumTaxHistories::from_current_unit(source()));
    let P::SaveMoreCredits { value: chosen, .. } = &mut page else { unreachable!() };
    chosen.credits.extend([MoreCredit::EmployerTip(tip_fica_carryforward(5_000_000))]);
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    // 5,023−220=4,803 general-business capacity (TMT0; under25,000).
    // 50,000 carry uses4,803 ->45,197. No capacity remains for8801;
    // retained15,706 MTC is preserved, never used a second time.
    assert_eq!(value(&r, "schedule_3_line_6l"), 220);
    assert_eq!(value(&r, "general_business_credit_limit"), 4803);
    assert_eq!(value(&r, "general_business_credit"), 4803);
    assert_eq!(value(&r, "tip_fica_credit_carryforward_to_next_year"), 45197);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15706);
    assert_eq!(return_line(&r, "20"), 502_300);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn mtc_negative_8978_that_creates_amt_cannot_reuse_the_same_regular_capacity() {
    let mut s = session(FederalStatus::Single);
    let wages = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(13_000_000))) },
        )],
    );
    accepted(&send(&mut s, wages));
    let mut audit = bba_interest_reduction();
    audit.interest_adjustment = -9_000_000;
    audit.original_partnership_interest = 9_000_000;
    audit.prior_wages = 2_500_000;
    audit.prior_taxable_income = 9_925_000;
    audit.prior_income_tax = 1_675_500;
    save_audit_source(&mut s, audit);
    let page = request(&mut s, MinimumTaxHistories::from_current_unit(source()));
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    // Current113,900 taxable:1,240+4,560+12,166+1,968=19,934 regular.
    // AMTI130,000−90,100=39,900;*.26=10,374 TMT.
    // Negative8978=16,755−928=15,827 ->regular comparison4,107;
    // AMT=10,374−4,107=6,267. MTC cannot reduce this TMT floor.
    assert_eq!(value(&r, "regular_income_tax"), 19934);
    assert_eq!(value(&r, "schedule_3_line_6l"), 15827);
    assert_eq!(value(&r, "alternative_minimum_tax"), 6267);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15706);
    assert_eq!(r.tax, Some(1_037_400));
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn mtc_line_22_has_zero_floor_when_negative_8978_exceeds_regular_tax() {
    let mut s = session(FederalStatus::Single);
    let wages = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(10_000_000))) },
        )],
    );
    accepted(&send(&mut s, wages));
    let mut audit = bba_interest_reduction();
    audit.interest_adjustment = -9_000_000;
    audit.original_partnership_interest = 9_000_000;
    audit.prior_wages = 2_500_000;
    audit.prior_taxable_income = 9_925_000;
    audit.prior_income_tax = 1_675_500;
    save_audit_source(&mut s, audit);
    let page = request(&mut s, MinimumTaxHistories::from_current_unit(source()));
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    // Table midpoint83,925:1,240+4,560+33,525*.22=13,175.50 ->13,176.
    // TMT=(100,000−90,100)*.26=2,574; negative8978 is15,827.
    // Line18=13,176+2,574=15,750, so6l=15,750 and77 remains unused.
    // Draft i8801 pp4–5 floor22 at0, rather than reporting−2,574.
    assert_eq!(value(&r, "regular_income_tax"), 13176);
    assert_eq!(value(&r, "alternative_minimum_tax"), 2574);
    assert_eq!(value(&r, "schedule_3_line_6l"), 15750);
    assert_eq!(value(&r, "form_8978_unusable_decrease"), 77);
    assert_eq!(value(&r, "minimum_tax_current_regular_liability"), 0);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15706);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

#[test]
fn unit41_f3_form_8801_line11_uses_retained_foreign_earned_owner() {
    let mut h = source();
    h.base = 12_810_000;
    h.exclusions.clear();
    h.foreign_earned = Some(MinimumTaxForeignEarned {
        excluded: 11_000_000,
        related_deductions: 0,
        gain_source: MinimumTaxForeignGainSource::NoPreferentialIncome,
    });
    let r = save(&mut session(FederalStatus::Single), MinimumTaxHistories::from_current_unit(h));
    // Line10=128100−88100=40000; tax150000*.26−tax110000*.26=10400.
    expect(&r, 0, &[(10, 40000), (11, 10400)]);
    for (name, expected) in
        [("stacked", 150000), ("stacked_tax", 39000), ("excluded_tax", 28600), ("tax", 10400)]
    {
        assert_eq!(
            value(&r, &alloc::format!("minimum_tax_history_0_foreign_earned_{name}")),
            expected
        );
    }
}
