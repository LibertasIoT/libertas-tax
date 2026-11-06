// Independent whole-return charity controls, IRC170 and Form8283.
use super::*;
use crate::tests::support::builders::vehicle_gift_source;

fn itemizer(agi: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, agi * 100, 0)];
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    d
}
fn property(kind: GiftKind, basis: i64, value: i64) -> PropertyGift {
    PropertyGift {
        kind,
        basis: basis * 100,
        value: value * 100,
        given: 20261115,
        acquired: 20200101,
        ..used_goods(value * 100)
    }
}
fn painting(basis: i64, value: i64) -> PropertyGift {
    property(
        GiftKind::Other {
            category: PropertyCategory::Art,
            similar_kind: SimilarPropertyKind::Paintings,
            condition: "Good".into(),
            unrelated_use: false,
        },
        basis,
        value,
    )
}
fn schedule_a(r: &FederalResult, expected: [i64; 5]) {
    for (id, expected) in [
        "charity_cash_gifts",
        "charity_noncash_gifts",
        "charity_current_allowed",
        "charity_carryover_allowed",
        "charity_deduction",
    ]
    .into_iter()
    .zip(expected)
    {
        assert_eq!(value(r, id), expected, "{id}");
    }
}
fn reopens(d: FederalDraft, r: FederalResult) {
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn unit15_f1_mixed_public_property_direct_cash_and_fund_cash() {
    let mut d = itemizer(100_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 5_000_000;
    ded.fund_cash_charity = 1_000_000;
    ded.property_gifts.push(property(
        GiftKind::Goods { kind: GoodsKind::Clothing, condition: GoodsCondition::Good },
        10_000,
        5_000,
    ));
    let r = supported(&d);
    // A5000 uses capacity before G60000; G allowed55000. Floor500 is A, with no Aexcess.
    schedule_a(&r, [55_000, 4_500, 59_500, 0, 59_500]);
    assert_eq!(value(&r, "charity_cash_sixty_return_year_carryforward"), 5_000);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 0);
    reopens(d.clone(), r);
    d.deductions.as_mut().unwrap().itemize_override = None;
    // 2026-11-02: the retained 59,500 itemized deduction remains larger under automatic selection.
    assert_eq!(return_line(&supported(&d), "12f"), 0);
}
#[test]
fn unit15_f2_private_foundation_stock_and_family_ten_percent() {
    let mut d = itemizer(100_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.other_cash_charity = 500_000;
    let mut stock =
        property(GiftKind::PublicStock { family_over_ten_percent: false }, 10_000, 30_000);
    stock.donee = DoneeKind::PrivateFoundation;
    ded.property_gifts.push(stock);
    let r = supported(&d);
    schedule_a(&r, [5_000, 19_500, 24_500, 0, 24_500]);
    assert_eq!(value(&r, "charity_capital_gain_twenty_return_year_carryforward"), 10_500);
    reopens(d.clone(), r);
    d.deductions.as_mut().unwrap().property_gifts[0].kind =
        GiftKind::PublicStock { family_over_ten_percent: true };
    schedule_a(&supported(&d), [4_833, 9_667, 14_500, 0, 14_500]);
}
#[test]
fn unit15_f3_art_related_use_basis_election_and_form8283_money() {
    let mut d = itemizer(200_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 5_000_000;
    ded.property_gifts.push(painting(8_000, 30_000));
    set_gift_appraisal(ded, 0, 2);
    d.next_id = 3;
    let r = supported(&d);
    schedule_a(&r, [50_000, 29_000, 79_000, 0, 79_000]);
    for (id, amount) in [
        ("form_8283_section_b_1_value", 30_000),
        ("form_8283_section_b_1_cost", 8_000),
        ("form_8283_section_b_1_claimed", 30_000),
    ] {
        assert_eq!(value(&r, id), amount);
    }
    reopens(d.clone(), r);
    d.deductions.as_mut().unwrap().basis_election = true;
    schedule_a(&supported(&d), [50_000, 7_000, 57_000, 0, 57_000]);
    d.deductions.as_mut().unwrap().basis_election = false;
    if let GiftKind::Other { unrelated_use, .. } =
        &mut d.deductions.as_mut().unwrap().property_gifts[0].kind
    {
        *unrelated_use = true;
    }
    schedule_a(&supported(&d), [50_000, 7_000, 57_000, 0, 57_000]);
}
#[test]
fn unit15_f4_sold_vehicle_proceeds_small_sale_and_return_benefit() {
    let mut d = itemizer(60_000);
    let mut g = donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 320_000 });
    g.value = 400_000;
    g.basis = 1_200_000;
    push_gift(d.deductions.as_mut().unwrap(), g);
    let r = supported(&d);
    schedule_a(&r, [0, 2_900, 2_900, 0, 2_900]);
    reopens(d.clone(), r);
    let form = &mut d.deductions.as_mut().unwrap().vehicle_gift_documents[0].form;
    (form.box_6a_yes, form.box_6a_no, form.box_6b) = (true, false, 20_000);
    schedule_a(&supported(&d), [0, 2_700, 2_700, 0, 2_700]);
    let form = &mut d.deductions.as_mut().unwrap().vehicle_gift_documents[0].form;
    (form.box_6a_yes, form.box_6a_no, form.box_6b) = (false, true, 0);
    form.box_4c = 45_000;
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_1_claim"), 500);
    assert_eq!(value(&r, "form_8283_needed"), 0);
}
#[test]
fn unit15_f5_class_current_first_prior_floor_and_basis_election() {
    let mut d = itemizer(80_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 3_000_000;
    ded.property_gifts.push(painting(5_000, 20_000));
    set_gift_appraisal(ded, 0, 2);
    ded.charity_carryovers = vec![
        prior_cash_charity(2024, 6_000),
        CharityCarryover {
            research_parts: None,
            year: 2025,
            class: CharityClass::CapitalGain30,
            balance: 900_000,
            appreciation: Some(400_000),
            same_filers: true,
        },
    ];
    d.next_id = 3;
    let r = supported(&d);
    schedule_a(&r, [30_000, 3_600, 33_600, 6_000, 39_600]);
    assert_eq!(value(&r, "charity_capital_gain_thirty_return_year_carryforward"), 16_400);
    assert_eq!(value(&r, "charity_carry_2025_capital_gain_thirty_remaining"), 9_000);
    reopens(d.clone(), r);
    d.deductions.as_mut().unwrap().basis_election = true;
    let r = supported(&d);
    schedule_a(&r, [30_000, 5_000, 35_000, 10_600, 45_600]);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
}
#[test]
fn unit15_f6_basis_allowed_k1_cash_current_limit_before_prior_cash() {
    let mut d = itemizer(50_000);
    let mut source = complete_ordinary_scorp(0, true);
    source.opening = Some(1_500_000);
    source.loans.clear();
    source.cash_charity = 2_000_000;
    source.amt = Some(amt_records(&source));
    // Only15000 of20000 source cash survives the actual Form7203 basis owner.
    assert_eq!(source.allowed_cash_charity(), 1_500_000);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 2_000_000;
    ded.charity_carryovers.push(prior_cash_charity(2025, 5_000));
    let r = supported(&d);
    schedule_a(&r, [29_750, 0, 29_750, 0, 29_750]);
    assert_eq!(value(&r, "charity_cash_sixty_return_year_carryforward"), 5_250);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 5_000);
    reopens(d, r);
}
#[test]
fn unit15_f7_poor_condition_single_household_item_has_required_appraisal() {
    let mut d = itemizer(40_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(property(
        GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::NotGood },
        2_000,
        600,
    ));
    assert!(!d.valid_stored());
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 0, 2);
    d.next_id = 3;
    let r = supported(&d);
    schedule_a(&r, [0, 400, 400, 0, 400]);
    assert_eq!(value(&r, "form_8283_section_b_value"), 600);
    reopens(d.clone(), r);
    d.deductions.as_mut().unwrap().appraisals[0].qualified = false;
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
}

#[test]
fn unit15_conservation_section_a_and_historic_credit_proportional_reduction() {
    // i8283 Which Sections To Complete; IRC170(f)(14). Approved5a489fb D3/D5.
    let mut d = itemizer(100_000);
    let kind = GiftKind::Conservation {
        purpose: ConservationPurpose::OpenSpace,
        historic_building: None,
        nps_number: None,
        farmer_rancher: false,
    };
    let mut gift = property(kind, 20_000, 4_000);
    gift.donee_acknowledged = false;
    d.deductions.as_mut().unwrap().property_gifts.push(gift);
    let r = supported(&d);
    schedule_a(&r, [0, 3_500, 3_500, 0, 3_500]);
    assert_eq!(value(&r, "form_8283_section_a_value"), 4_000);
    assert!(!r.lines.iter().any(|l| l.id == "form_8283_section_b_value"));
    reopens(d.clone(), r);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts[0].value = 3_000_000;
    ded.property_gifts[0].kind = GiftKind::Conservation {
        purpose: ConservationPurpose::HistoricStructure,
        historic_building: Some(HistoricBuildingGift {
            rehabilitation_credits: 2_000_000,
            building_value: 100_000_000,
        }),
        nps_number: None,
        farmer_rancher: false,
    };
    set_gift_appraisal(ded, 0, 2);
    d.next_id = 3;
    // 20000credits ×30000easement/1000000building =600;29400claim−500floor=28900.
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_1_claim"), 29_400);
    schedule_a(&r, [0, 28_900, 28_900, 0, 28_900]);
    reopens(d.clone(), r);
    if let GiftKind::Conservation { historic_building, .. } =
        &mut d.deductions.as_mut().unwrap().property_gifts[0].kind
    {
        *historic_building = None;
    }
    schedule_a(&supported(&d), [0, 29_500, 29_500, 0, 29_500]);
}
#[test]
fn unit15_similar_books_cross_reporting_kinds_and_donees() {
    // Reg1.170A-13(c)(7)(iii), Reg1.170A-16(f)(5)(ii), approved D2.
    let mut d = itemizer(60_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(property(
        GiftKind::Goods { kind: GoodsKind::Books, condition: GoodsCondition::Good },
        6_000,
        3_000,
    ));
    let mut rare = painting(6_000, 3_000);
    rare.charity = "Another public charity".into();
    rare.kind = GiftKind::Other {
        category: PropertyCategory::Art,
        similar_kind: SimilarPropertyKind::Books,
        condition: "Good".into(),
        unrelated_use: false,
    };
    ded.property_gifts.push(rare);
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 0, 2);
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 1, 2);
    d.next_id = 3;
    let r = supported(&d);
    assert_eq!(value(&r, "form_8283_section_b_value"), 6_000);
    schedule_a(&r, [0, 5_700, 5_700, 0, 5_700]);
    reopens(d, r);
}
#[test]
fn unit15_poor_condition_jewelry_has_no_clothing_household_guard() {
    // IRC170(f)(16) expressly restricts the special condition rule; approved D4.
    let mut d = itemizer(40_000);
    d.deductions.as_mut().unwrap().property_gifts.push(property(
        GiftKind::Goods { kind: GoodsKind::Jewelry, condition: GoodsCondition::NotGood },
        1_000,
        400,
    ));
    let r = supported(&d);
    schedule_a(&r, [0, 200, 200, 0, 200]);
    assert!(d.valid_stored());
    reopens(d, r);
}
#[test]
fn unit15_conservation_original_hundred_percent_vintage_expires_after_fifteen_years() {
    let mut d = itemizer(50_000);
    d.deductions.as_mut().unwrap().charity_carryovers.push(CharityCarryover {
        research_parts: None,
        year: 2011,
        class: CharityClass::Conservation100,
        balance: 6_000_000,
        appreciation: None,
        same_filers: true,
    });
    let r = supported(&d);
    schedule_a(&r, [0, 0, 0, 49_750, 49_750]);
    assert_eq!(value(&r, "charity_carry_2011_conservation_hundred_used"), 50_000);
    assert_eq!(value(&r, "charity_carry_2011_conservation_hundred_expired"), 10_000);
    assert_eq!(value(&r, "charity_carry_2011_conservation_hundred_remaining"), 0);
    reopens(d, r);
}
#[test]
fn unit15_appraisal_calendar_window_uses_report_effective_and_declaration_separately() {
    let mut d = itemizer(100_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(painting(8_000, 30_000));
    set_gift_appraisal(ded, 0, 2);
    d.next_id = 3;
    let app = &mut d.deductions.as_mut().unwrap().appraisals[0];
    app.report_date = 20270120;
    app.declaration_date = 20261201;
    app.effective_date = 20261115;
    assert!(d.valid_stored());
    let r = supported(&d);
    schedule_a(&r, [0, 29_500, 29_500, 0, 29_500]);
    reopens(d.clone(), r);
    let app = &mut d.deductions.as_mut().unwrap().appraisals[0];
    app.report_date = 20260916;
    app.effective_date = 20260916;
    assert!(d.valid_stored());
    d.deductions.as_mut().unwrap().appraisals[0].report_date = 20260915;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().appraisals[0].report_date = 20270120;
    d.deductions.as_mut().unwrap().appraisals[0].effective_date = 20261114;
    assert!(!d.valid_stored());
}
#[test]
fn unit15_nonitemizer_excludes_fund_other_cash_volunteer_and_prior_balances() {
    let mut d = itemizer(60_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.fund_cash_charity = 1_000_000;
    ded.other_cash_charity = 1_000_000;
    ded.volunteer_expenses
        .push(volunteer_travel(VolunteerCarCost::Mileage { miles: 1_000 }, 1_000));
    ded.charity_carryovers.push(prior_cash_charity(2025, 1_000));
    let r = supported(&d);
    assert_eq!(return_line(&r, "12f"), 0);
    reopens(d, r);
}

#[test]
fn unit15_attachment_thresholds_use_the_claimed_similar_group() {
    let mut d = itemizer(100_000);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts = vec![painting(1_000, 300_000), painting(1_000, 300_000)];
    set_gift_appraisal(ded, 0, 2);
    set_gift_appraisal(ded, 1, 2);
    d.next_id = 3;
    let r = supported(&d);
    assert!(noted(&r, "NOTE_CHARITY_LARGE_APPRAISAL"));
    assert!(noted(&r, "NOTE_CHARITY_ART_APPRAISAL"));
    assert!(!noted(&r, "NOTE_CHARITY_FORM1098C"));
    assert!(!noted(&r, "NOTE_CHARITY_HISTORIC_ATTACHMENTS"));
    // The basis election changes the actual deduction to $2,000; fair
    // market value alone does not create the art or $500,000 attachment.
    d.deductions.as_mut().unwrap().basis_election = true;
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_CHARITY_LARGE_APPRAISAL"));
    assert!(!noted(&r, "NOTE_CHARITY_ART_APPRAISAL"));
}

#[test]
fn unit15_nol_adjustment_conserves_public_and_other_cash_classes() {
    // Reg1.170A-10(d)(1): public-charity contributions are used first.
    // At60000 before NOL: G20000, B16000, floor300 charged to B.
    for (loss, reduction, public_carry, other_carry) in
        [(100_000, 35_700, 0, 4_300), (50_000, 25_700, 4_050, 4_300)]
    {
        let mut d = nol_cash_gift(&[(2017, loss)]);
        let ded = d.deductions.as_mut().unwrap();
        ded.cash_charity = 2_000_000;
        ded.other_cash_charity = 2_000_000;
        let r = supported(&d);
        assert_eq!(value(&r, "charity_return_year_nol_reduction"), reduction);
        assert_eq!(value(&r, "charity_cash_sixty_return_year_carryforward"), public_carry);
        assert_eq!(value(&r, "charity_other_thirty_return_year_carryforward"), other_carry);
        assert_eq!(value(&r, "charity_other_30_percent_return_year_carryforward"), other_carry);
        assert_eq!(value(&r, "charity_return_year_carryforward"), public_carry + other_carry);
        reopens(d, r);
    }
}

#[test]
fn form1098c_printed_sale_correction_feeds_the_existing_charity_claim() {
    use crate::tests::support::client::income_pages::open_document_center;
    let mut native =
        donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 320_000 });
    native.value = 400_000;
    native.basis = 1_200_000;
    let (mut document, source) = vehicle_gift_source(native);
    document.id = 0;
    document.form.donee.name = "Synthetic Care Vehicles".into();
    document.form.box_2b = Some(2018);
    document.form.box_2c = "Example".into();
    document.form.box_2d = "Sedan".into();
    let gift = PropertyGift::try_from((&source, &document.form)).unwrap();
    assert_eq!(gift.charity, "Synthetic Care Vehicles");
    assert_eq!(gift.description, "2018 Example Sedan");
    assert!(
        matches!(&gift.kind, GiftKind::Vehicle { vin, treatment: VehicleUse::Sold { sold: 20261001, proceeds: 320_000 } } if vin == &document.form.box_3)
    );
    assert_eq!(gift.claim(false).1, 320_000);
    document.form.treatment = None;
    let mut session = Interview::new(TEST_USER, itemizer(60_000));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.vehicle_gifts.documents.push(document);
    accepted(&send(&mut session, request));
    let ded = session.draft.deductions.as_ref().unwrap();
    let id = ded.vehicle_gift_documents[0].id;
    assert!(id > 0);
    assert!(ded.vehicle_gift_documents[0].assigned.is_known());
    assert!(ded.vehicle_gifts.is_empty(), "paper creates no personal facts");
    assert_eq!(ded.gifts().unwrap_err().key, "VEHICLE_GIFT_FACTS_NEEDED");
    assert!(session.draft.valid_stored(), "unresolved treatment saves as pending");
    let mut correction = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    documents.other.vehicle_gifts.documents[0].form.treatment = Some(Form1098CTreatment::Sold);
    accepted(&send(&mut session, correction));
    let mut request = crate::tests::cross::interview::open_itemized_topic(
        &mut session,
        TaxSection::GiftCarryovers,
    );
    let P::SaveGiftCarryovers { value: page, .. } = &mut request else { unreachable!() };
    assert_eq!(page.vehicles[0].document.id, id);
    assert!(page.vehicles[0].gift.is_none());
    page.vehicles[0].gift = Some(source);
    accepted(&send(&mut session, request));
    let personal = session.draft.deductions.as_ref().unwrap().vehicle_gifts[0].clone();
    assert_eq!(personal.id, id);
    let result = supported(&session.draft);
    schedule_a(&result, [0, 2_900, 2_900, 0, 2_900]);
    session = restart(session.draft.clone());
    assert_eq!(supported(&session.draft), result);
    let mut correction = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    documents.other.vehicle_gifts.documents[0].form.box_4c = 240_000;
    accepted(&send(&mut session, correction));
    assert_eq!(session.draft.deductions.as_ref().unwrap().vehicle_gifts[0], personal);
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "charity_property_1_claim"), 2_400);
    schedule_a(&corrected, [0, 2_100, 2_100, 0, 2_100]);
    session = restart(session.draft.clone());
    assert_eq!(supported(&session.draft), corrected);
    let mut benefit = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut benefit else { unreachable!() };
    let form = &mut documents.other.vehicle_gifts.documents[0].form;
    form.box_4c = 320_000;
    form.box_6a_yes = true;
    form.box_6a_no = false;
    form.box_6b = 20_000;
    accepted(&send(&mut session, benefit));
    schedule_a(&supported(&session.draft), [0, 2_700, 2_700, 0, 2_700]);
    let mut small_sale = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut small_sale else { unreachable!() };
    let form = &mut documents.other.vehicle_gifts.documents[0].form;
    form.box_4c = 45_000;
    form.box_6a_yes = false;
    form.box_6a_no = true;
    form.box_6b = 0;
    accepted(&send(&mut session, small_sale));
    let small_sale = supported(&session.draft);
    assert_eq!(value(&small_sale, "charity_property_1_claim"), 500);
    assert_eq!(value(&small_sale, "form_8283_needed"), 0);
    let mut removal = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut removal else { unreachable!() };
    documents.other.vehicle_gifts.documents.clear();
    accepted(&send(&mut session, removal));
    assert!(session.draft.deductions.as_ref().unwrap().vehicle_gifts.is_empty());
    assert!(restart(session.draft).draft.deductions.unwrap().vehicle_gift_documents.is_empty());
}

#[test]
fn form1098c_box5a_preserves_combined_certification_and_unresolved_facts() {
    let mut native = donated_personal_vehicle(VehicleUse::SignificantUse {
        details: "Major repairs followed by daily meal deliveries for one year".into(),
    });
    native.value = 400_000;
    let (mut document, source) = vehicle_gift_source(native);
    document.form.treatment =
        Some(crate::charity::Form1098CTreatment::SignificantUseOrMaterialImprovement);
    let gift = PropertyGift::try_from((&source, &document.form)).unwrap();
    assert!(matches!(&gift.kind, GiftKind::Vehicle {
        treatment: VehicleUse::SignificantUse { details }, ..
    } if details == "Major repairs followed by daily meal deliveries for one year"));
    assert_eq!(gift.claim(false).1, 400_000);
    document.form.treatment = Some(crate::charity::Form1098CTreatment::MaterialImprovement);
    assert_eq!(PropertyGift::try_from((&source, &document.form)).unwrap().claim(false).1, 400_000);
    document.form.treatment = None;
    let issue = PropertyGift::try_from((&source, &document.form)).unwrap_err().issue();
    assert_eq!(issue.key, "VEHICLE_TREATMENT_NEEDED");
    assert_eq!(issue.field.unwrap().square(), "[form/treatment]");
    let mut d = itemizer(60_000);
    d.deductions.as_mut().unwrap().vehicle_gift_documents.push(document);
    d.deductions.as_mut().unwrap().vehicle_gifts.push(source);
    let issue = d.deductions.as_ref().unwrap().gifts().unwrap_err();
    assert_eq!(issue.key, "VEHICLE_TREATMENT_NEEDED");
    assert_eq!(issue.field.unwrap().square(), "[vehicle_gifts/0/form/treatment]");
    let result = estimate(&d);
    let pending =
        result.issues.iter().find(|issue| issue.display.key == "VEHICLE_TREATMENT_NEEDED").unwrap();
    assert_eq!(pending.context, IssueContext::Section(TaxSection::Documents));
}

#[test]
fn form1098c_absent_vehicle_year_survives_document_save_and_restart() {
    use crate::tests::support::client::income_pages::open_document_center;
    let native = donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 320_000 });
    let (mut document, source) = vehicle_gift_source(native);
    document.id = 0;
    document.form.box_2b = None;
    document.form.box_2c = "Example".into();
    document.form.box_2d = "Sedan".into();
    let expected = PropertyGift::try_from((&source, &document.form)).unwrap();
    assert_eq!(expected.description, "Example Sedan");
    let mut session = Interview::new(TEST_USER, itemizer(60_000));
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.vehicle_gifts.documents.push(document);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
    let form = &documents.other.vehicle_gifts.documents[0].form;
    assert_eq!(form.box_2b, None);
    assert_eq!(PropertyGift::try_from((&source, form)).unwrap(), expected);
}
