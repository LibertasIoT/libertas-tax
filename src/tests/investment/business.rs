use super::*;
use crate::tests::business::business::{
    asset_sale_return, cow, has_line, has_note, press, refused_with, shop_building, sold,
};

#[test]
fn reit_dividends_deduct_without_business_income_or_wage_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reit_income(10000, 6000, true)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "qbi_deduction"), 1200);
    d.income[0] = wage(1, Owner::Taxpayer, 40000000, 0);
    assert_eq!(value(&supported(&d), "qbi_deduction"), 1200);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income = reit_income(10000, 6000, false);
    assert_eq!(value(&supported(&d), "qbi_deduction"), 0);
}
#[test]
fn section1244_investment_loss_reduces_niit_and_can_create_business_nol() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(section1244_loss(50000, 0, 20200101)));
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 50000);
    assert_eq!(value(&r, "agi"), -50000);
    add(&mut d, wage(0, Owner::Taxpayer, 30000000, 0));
    add(&mut d, income(FederalIncome::Interest { data: native_interest(10000000, 0) }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 350000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
}

#[test]
fn form_4797_business_asset_sale_matches_hand_computation() {
    // Single filer, $60,000 W-2, a $150,000 Schedule C business that sold
    // two assets in 2026 (Form 4797 instructions, Pub 544, Pub 946 ch. 4).
    let d = asset_sale_return(vec![
        press(sold(20260715, 600_000, 20_000)),
        shop_building(sold(20260620, 42_000_000, 2_000_000)),
    ]);
    let r = supported(&d);
    // Press, 2026 depreciation: table year 3 is 19.2% ($1,920), half of it
    // under the half-year convention for the year of sale: $960.
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 960);
    // Part III: line 22 $6,000 (the gross price); line 23 $10,000 + $200 of
    // selling expenses = $10,200; line 24 $5,200 + $960 = $6,160; line 25 $4,040; line 26 $1,960; section 1245
    // line 27b the smaller of $1,960 and $6,160 = $1,960, all ordinary.
    for (suffix, amount) in [
        ("sale_price", 6_000),
        ("cost", 10_200),
        ("depreciation", 6_160),
        ("adjusted_basis", 4_040),
        ("gain", 1_960),
        ("1245_recapture", 1_960),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    // Building, 2026 depreciation by the mid-month convention through June:
    // $9,999.60 x 11/24 = $4,583.15.
    assert_eq!(line_amount(&r, "business_2_asset_1_macrs"), 458_315);
    // Line 22 $420,000; line 23 $390,000 + $20,000 = $410,000; depreciation
    // $99,594.30 + $4,583.15 = $104,177; line 23 $285,823; line 24 $114,177.
    // Straight-line real property after 1986: no additional depreciation.
    for (suffix, amount) in [
        ("sale_price", 420_000),
        ("cost", 410_000),
        ("depreciation", 104_177),
        ("adjusted_basis", 305_823),
        ("gain", 114_177),
        ("1250_recapture", 0),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_1_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    // Printed lines follow the 2026 draft Form 4797 (f4797--dft.pdf, created
    // 10/1/26): Part III lines 22-26, 1250 recapture 28g, Part I lines 8-10.
    for (id, line) in [
        ("sale_price", "22"),
        ("cost", "23"),
        ("depreciation", "24"),
        ("adjusted_basis", "25"),
        ("gain", "26"),
        ("1250_recapture", "28g"),
    ] {
        assert_eq!(
            printed_line(&r, &alloc::format!("business_2_asset_1_disposal_{id}")),
            (TaxForm::Form4797, line.into()),
            "{id}"
        );
    }
    for (id, line) in [("form4797_line8", "8"), ("form4797_line9", "9"), ("form4797_line10", "10")]
    {
        assert_eq!(printed_line(&r, id), (TaxForm::Form4797, line.into()));
    }
    // Part I: line 8 $114,177 (the building's Part III line 34 gain), no prior
    // losses so line 9 is $0 and line 10 $114,177 goes to Schedule D line 11.
    assert_eq!(value(&r, "form4797_line8"), 114_177);
    assert_eq!(value(&r, "form4797_line9"), 0);
    assert_eq!(value(&r, "form4797_line10"), 114_177);
    // Unrecaptured section 1250 gain: the smaller of line 22 ($104,177)
    // and line 24 ($114,177).
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 104_177);
    assert_eq!(return_line(&r, "7a"), 11_417_700);
    // Schedule C line 13 $960 + $4,583.15 = $5,543.15; line 31
    // $150,000 - $5,543.15 = $144,456.85. Schedule 1 line 4 (Form 4797
    // line 18b) is the $1,960 recapture; 1040 line 8 is $144,457 + $1,960.
    assert_eq!(line_amount(&r, "business_2_net_profit"), 14_445_685);
    assert_eq!(return_line(&r, "8"), 14_641_700);
}

#[test]
fn prior_section_1231_losses_make_part_of_the_asset_gain_ordinary() {
    // The building's $114,177 gain with a $3,000 nonrecaptured 2023 loss:
    // line 8 $3,000 ordinary (Schedule 1 line 4), line 9 $111,177; the
    // unrecaptured section 1250 gain falls by the recaptured $3,000.
    let mut d = asset_sale_return(vec![
        press(sold(20260715, 600_000, 20_000)),
        shop_building(sold(20260620, 42_000_000, 2_000_000)),
    ]);
    let year = Section1231LossYear { year: 2023, original: 300_000, remaining: 300_000 };
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![year.clone()], amt: vec![year] });
    let r = supported(&d);
    assert_eq!(value(&r, "form4797_line9"), 3_000);
    assert_eq!(value(&r, "form4797_line10"), 111_177);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 101_177);
    assert_eq!(return_line(&r, "7a"), 11_117_700);
    assert_eq!(return_line(&r, "8"), 14_941_700);
}

// ---- T3-6252 Form 6252 installment sales of business assets ----

fn sold_on_installment(
    date: u32,
    proceeds: i64,
    selling_expenses: i64,
    assumed_debt: i64,
    payments: i64,
    interest: crate::NoteInterest,
) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::Livestock::No,
            proceeds,
            selling_expenses,
            payment: crate::SalePayment::Installment {
                assumed_debt,
                payments,
                interest,
                special: false,
            },
            buyer: crate::Buyer::Unrelated,
        },
    }
}

#[test]
fn form_6252_equipment_installment_taxes_recapture_now_and_gain_by_payments() {
    // The press ($10,000 cost, $6,160 depreciation through July 15, 2026)
    // sold for a $20,000 note, no selling expenses, $4,000 principal
    // received in 2026 at an adequate rate. Form 4797 Part III: line 22
    // $20,000, line 25 $3,840, line 26 $16,160, line 27b $6,160 recapture
    // taxed in 2026 (Form 6252 line 12). Form 6252: line 5 $20,000; 6 $0;
    // 7 $20,000; 8 $10,000; 9 $6,160; 10 $3,840; 11 $0; 12 $6,160; 13
    // $10,000; 16 $10,000; 17 $0; 18 $20,000 (gross profit 50%); 21 and 22
    // $4,000; 24 $2,000; 25 $0; 26 $2,000 to Form 4797 line 4.
    let d = asset_sale_return(vec![press(sold_on_installment(
        20260715,
        2_000_000,
        0,
        0,
        400_000,
        crate::NoteInterest::Adequate,
    ))]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_disposal_gain"), 16_160);
    assert_eq!(value(&r, "business_2_asset_0_disposal_1245_recapture"), 6_160);
    for (suffix, amount) in [
        ("price", 20_000),
        ("assumed_debt", 0),
        ("net_price", 20_000),
        ("cost", 10_000),
        ("depreciation", 6_160),
        ("adjusted_basis", 3_840),
        ("expenses", 0),
        ("recapture", 6_160),
        ("total", 10_000),
        ("gross_profit", 10_000),
        ("excess_debt", 0),
        ("contract_price", 20_000),
        ("payments", 4_000),
        ("total_payments", 4_000),
        ("income", 2_000),
        ("ordinary", 0),
        ("gain", 2_000),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_form6252_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    assert_eq!(value(&r, "business_2_asset_0_disposal_installment_gain"), 2_000);
    // Part I line 7 $2,000, no prior losses: line 9 $2,000 long-term.
    assert_eq!(value(&r, "form4797_line8"), 2_000);
    assert_eq!(value(&r, "form4797_line10"), 2_000);
    assert_eq!(return_line(&r, "7a"), 200_000);
    // Schedule C $150,000 - $960 = $149,040 plus the $6,160 recapture:
    // 1040 line 8 $155,200.
    assert_eq!(return_line(&r, "8"), 15_520_000);
}

#[test]
fn form_6252_building_installment_recognizes_unrecaptured_1250_first() {
    // The shop building (adjusted basis $285,823, depreciation $104,177)
    // sold June 20, 2026 for $420,000 with $20,000 selling expenses; the
    // buyer took over a $100,000 mortgage and paid $60,000 of principal in
    // 2026. Form 6252: line 7 $320,000; 13 $305,823; 16 $114,177; 17 $0;
    // 18 $320,000; 24 $60,000 x 114,177 / 320,000 = $21,408.19, $21,408;
    // 26 $21,408, all unrecaptured section 1250 gain (the smaller of the
    // $104,177 still unreported and line 26).
    let d = asset_sale_return(vec![shop_building(sold_on_installment(
        20260620,
        42_000_000,
        2_000_000,
        10_000_000,
        6_000_000,
        crate::NoteInterest::Adequate,
    ))]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_net_price"), 320_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_total"), 305_823);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gross_profit"), 114_177);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_contract_price"), 320_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gain"), 21_408);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_unrecaptured_1250"), 21_408);
    assert_eq!(value(&r, "form4797_line8"), 21_408);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 21_408);
    assert_eq!(return_line(&r, "7a"), 2_140_800);
}

#[test]
fn form_6252_loans_above_basis_are_a_payment_in_the_year_of_sale() {
    // The press sold for $20,000: the buyer took over a $15,000 loan and
    // gave a $5,000 note, nothing paid in 2026. Line 13 $10,000, so line 17
    // is $5,000 (the loan above line 13), line 18 $5,000 + $5,000 =
    // $10,000, gross profit 100%; line 22 $5,000; line 24 and 26 $5,000.
    let d = asset_sale_return(vec![press(sold_on_installment(
        20260715,
        2_000_000,
        0,
        1_500_000,
        0,
        crate::NoteInterest::Adequate,
    ))]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_excess_debt"), 5_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_contract_price"), 10_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gain"), 5_000);
    assert_eq!(value(&r, "form4797_line8"), 5_000);
}

#[test]
fn form_6252_loss_sale_is_reported_in_full_and_unstated_interest_starts_above_3000() {
    // The press sold for a $3,000 note at no interest: a $840 loss
    // ($3,000 - $3,840). No Form 6252 for a loss: Part I line 2 in full.
    // Section 483(d)(2): no unstated interest on a sale of $3,000 or less.
    let d = asset_sale_return(vec![press(sold_on_installment(
        20260715,
        300_000,
        0,
        0,
        50_000,
        crate::NoteInterest::Low,
    ))]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_disposal_section1231"), -840);
    assert!(!has_line(&r, "business_2_asset_0_disposal_form6252_income"));
    // $3,000.01 at a low rate: unstated interest, refused by name.
    let d = asset_sale_return(vec![press(sold_on_installment(
        20260715,
        300_001,
        0,
        0,
        50_000,
        crate::NoteInterest::Low,
    ))]);
    refused_with(&d, "unstated interest");
}

#[test]
fn form_6252_notes_above_5_million_owe_section_453a_interest() {
    // The shop building sold for a note, nothing received in 2026: the
    // note outstanding at year-end is the whole price. Exactly $5,000,000
    // is not "more than $5 million".
    let at = |price| {
        asset_sale_return(vec![shop_building(sold_on_installment(
            20260620,
            price,
            0,
            0,
            0,
            crate::NoteInterest::Adequate,
        ))])
    };
    let r = supported(&at(500_000_000));
    // No payment in 2026: line 24 is zero, nothing recognized.
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gain"), 0);
    refused_with(&at(500_000_100), "section 453A");
}

#[test]
fn form_6252_situations_outside_the_calculation_are_refused_by_name() {
    // Related party, dealer, contingent price or pledged note.
    let mut disposal =
        sold_on_installment(20260715, 2_000_000, 0, 0, 400_000, crate::NoteInterest::Adequate);
    if let crate::AssetDisposal::Disposed {
        how:
            crate::DisposalKind::Sold {
                payment: crate::SalePayment::Installment { special, .. }, ..
            },
        ..
    } = &mut disposal
    {
        *special = true;
    }
    refused_with(&asset_sale_return(vec![press(disposal)]), "by a dealer");
    // Property held one year or less and property also used personally are
    // calculated now (unit B11 4797); a related buyer on an installment sale is
    // refused by name (sections 453(e) and 453(g)).
    let mut related =
        press(sold_to(20260715, 2_000_000, crate::Buyer::Relative, installment(400_000)));
    related.listed = ListedProperty::No { business_percent: 100 };
    let r = estimate(&asset_sale_return(vec![related]));
    assert_eq!(r.state, ResultState::Incomplete);
    // Payments above the price.
    let d = asset_sale_return(vec![shop_building(sold_on_installment(
        20260620,
        42_000_000,
        0,
        40_000_000,
        2_000_001,
        crate::NoteInterest::Adequate,
    ))]);
    // Loans taken over $400,000 plus $20,000.01 received exceed the
    // $420,000 price: invalid stored data (the page rejects it on both
    // sides with INSTALLMENT_PAID_ABOVE_PRICE).
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Incomplete);
    assert!(
        r.issues.iter().any(|i| i.message.contains("Accepted data is invalid")),
        "{:?}",
        r.issues
    );
}

// Line audit fix F13 (2026-10-10, Forms 4797 and 4684; audit
// qbi-schedule-f-4797-4684.md section 3). Every expected amount is the audit's
// hand-worked figure from the 2026 draft Form 4797 with the 2025 final
// instructions (Part III lines 22-28, Part I lines 8-10, the "Where To Make
// First Entry" chart), Pub. 225 chapter 9 and sections 1231(b)(3), 1245 and
// 1250, never the App's own output.

// The amounts of the Form 4797 rows printed on one line, whole dollars.
fn line_4797(r: &FederalResult, line: &str) -> i64 {
    let wanted = FormLine::new(TaxForm::Form4797, line);
    r.lines.iter().filter(|l| l.form_line.as_ref() == Some(&wanted)).map(|l| l.amount).sum::<i64>()
        / 100
}

// The lines foot: Part I lines 2-7 add to line 8, line 10 is line 8 less line
// 9, and the Part II rows (lines 11-18) are the whole ordinary gain on
// Schedule 1 line 4. One return with an S corporation's box 9 gain (line 2), a
// business asset sold at a section 1231 loss (line 2), an asset held under a
// year (line 11) and a breeding cow held under 24 months (line 18).
#[test]
fn f13_form_4797_lines_foot_to_their_source_rows() {
    let mut d = complete_property_gain_draft(true, 0);
    let scorp_id = d.income[1].id;
    let loss = continuing_asset(
        1_000_000,
        20240301,
        EquipmentClass::Five,
        520_000,
        sold(20260715, 100_000, 0),
    );
    let short = continuing_asset(
        1_000_000,
        20250715,
        EquipmentClass::Five,
        200_000,
        sold(20260715, 900_000, 0),
    );
    let mut business = cash_business(150_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets = vec![loss, short, cow(20250110, crate::Livestock::CattleOrHorse, 20260615)];
    add(&mut d, income(business));
    let r = supported(&d);
    // The K-1 box 9 gain is its own line 2 row.
    assert_eq!(value(&r, &alloc::format!("scorp_{scorp_id}_section1231")), 20_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("scorp_{scorp_id}_section1231")),
        (TaxForm::Form4797, "2".into())
    );
    let part_one: i64 = ["2", "3", "4", "5", "6", "7"].iter().map(|line| line_4797(&r, line)).sum();
    assert_eq!(part_one, value(&r, "form4797_line8"));
    // Loss: $10,000 less $5,200 and 2026's $960 leaves $3,840 basis against
    // $1,000: a $2,840 loss; so line 8 is $20,000 - $2,840.
    assert_eq!(part_one, 17_160);
    let line_8 = value(&r, "form4797_line8");
    assert_eq!(value(&r, "form4797_line10"), (line_8 - value(&r, "form4797_line9")).max(0));
    // Part II: the short asset ($9,000 less $10,000 - $2,000 - $1,600 =
    // $6,400, a $2,600 gain) on line 11 and the cow's $1,320 on line 18.
    assert_eq!(line_4797(&r, "11"), 2_600);
    assert_eq!(line_4797(&r, "18"), 1_320);
    assert_eq!(line_4797(&r, "19"), 3_920);
    assert_eq!(line_4797(&r, "20b"), 3_920);
    assert_eq!(line_4797(&r, "33"), 0);
    assert_eq!(line_4797(&r, "14"), 0);
    assert_eq!(
        value(&r, "schedule_1_other_gains"),
        (11..=18).map(|line| line_4797(&r, &alloc::format!("{line}"))).sum::<i64>()
    );
}
fn sold_to(
    date: u32,
    proceeds: i64,
    buyer: crate::Buyer,
    payment: crate::SalePayment,
) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::Livestock::No,
            proceeds,
            selling_expenses: 0,
            payment,
            buyer,
        },
    }
}
fn installment(payments: i64) -> crate::SalePayment {
    crate::SalePayment::Installment {
        assumed_debt: 0,
        payments,
        interest: crate::NoteInterest::Adequate,
        special: false,
    }
}

#[test]
fn installment_sales_of_short_held_and_mixed_use_property_are_calculated() {
    // Held a year or less ($10,000 5-year asset placed 2025-07-15, $3,600 allowed
    // with the 2026 half-year share, sold 2026-07-15 for $15,000 with $5,000 down):
    // gain $8,600, recapture $3,600 taxed now; gross profit $15,000 - ($6,400 + $3,600)
    // = $5,000, income $5,000 x 5,000 / 15,000 = $1,667 to Form 4797 line 11.
    let short = continuing_asset(
        1_000_000,
        20250715,
        EquipmentClass::Five,
        200_000,
        sold_to(20260715, 1_500_000, crate::Buyer::Unrelated, installment(500_000)),
    );
    let r = supported(&asset_sale_return(vec![short]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), 3_600);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gross_profit"), 5_000);
    assert_eq!(value(&r, "business_2_asset_0_disposal_installment_ordinary"), 1_667);
    assert_eq!(
        printed_line(&r, "business_2_asset_0_disposal_installment_ordinary"),
        (TaxForm::Form4797, "11".into())
    );
    // 60% business, cost $10,000, $3,696 allowed (3,120 + the 576 half-year share),
    // sold 2026-07-15 for $12,000 with $3,000 down: the business part is a Form 6252
    // on $7,200 (gross profit $7,200 - ($6,000 - $3,696 + $3,696) = $1,200, payments
    // $1,800, income $1,800 x 1,200 / 7,200 = $300); the personal part's $800 gain is
    // counted in full on Form 8949.
    let mut mixed = continuing_asset(
        1_000_000,
        20240301,
        EquipmentClass::Five,
        312_000,
        sold_to(20260715, 1_200_000, crate::Buyer::Unrelated, installment(300_000)),
    );
    mixed.listed = ListedProperty::No { business_percent: 60 };
    let r = supported(&asset_sale_return(vec![mixed]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_gross_profit"), 1_200);
    assert_eq!(value(&r, "business_2_asset_0_disposal_form6252_income"), 300);
    assert_eq!(value(&r, "form_8949_2_1_gain"), 800);
    assert!(has_note(&r, "NOTE_ASSET_PERSONAL_PART_IN_FULL"));
}

#[test]
fn f9_sales_to_related_buyers_follow_sections_267_and_1239() {
    // The press sold for $12,000: gain $8,160 (depreciation $6,160). To a stranger
    // $6,160 is section 1245 ordinary and $2,000 section 1231; to a corporation the
    // person controls (section 1239) the whole $8,160 is Part II line 11 ordinary.
    let at = |buyer| press(sold_to(20260715, 1_200_000, buyer, crate::SalePayment::InFull));
    let r = supported(&asset_sale_return(vec![at(crate::Buyer::ControlledEntity)]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), 8_160);
    assert!(!has_line(&r, "business_2_asset_0_disposal_1245_recapture"));
    assert_eq!(return_line(&r, "8"), 15_720_000);
    let r = supported(&asset_sale_return(vec![at(crate::Buyer::Unrelated)]));
    assert_eq!(return_line(&r, "8"), 15_520_000);
    // A relative pays $3,000 for it: an $840 loss that is not deductible, so the
    // return is Schedule C alone, with one note.
    let r = supported(&asset_sale_return(vec![press(sold_to(
        20260715,
        300_000,
        crate::Buyer::Relative,
        crate::SalePayment::InFull,
    ))]));
    assert!(has_note(&r, "NOTE_SALE_TO_RELATIVE_LOSS_DISALLOWED"));
    assert!(!has_line(&r, "business_2_asset_0_disposal_section1231"));
    assert_eq!(return_line(&r, "8"), 14_904_000);
}
