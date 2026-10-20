use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Forms 1099-NEC, 1099-MISC and 1099-K entered once and sent to the page that owns
// each box (unit b9-2, 2026-10-10), Form 1099-G boxes 5, 6 and 8, and the left-out
// and refused cases. Every amount is worked by hand with the 2026 amounts the App
// uses (`complete(Single)`: one W-2 of $60,000, standard deduction $16,100, taxable
// 43,900, tax 5,023; single 10% to $12,400 and 12% to $50,400, 22% above; joint
// 10% to $24,800 and 12% to $100,800; Tax Table rows $50 wide taxed at their
// midpoint). Schedule SE: 92.35% of the profit, 12.4% to the wage base and 2.9%.
use super::*;
use crate::tests::cross::interview::{
    document, from_review, open_topic, save_documents, save_topic, w2,
};

fn nec_form(
    compensation: i64,
    withholding: i64,
    belongs: IncomeUse,
    business: Option<i64>,
) -> FederalIncome {
    FederalIncome::Nec1099 {
        data: FederalForm1099Nec {
            compensation: compensation * 100,
            withholding: withholding * 100,
            golden_parachute: 0,
            belongs,
            business,
        },
    }
}
fn misc_form() -> FederalForm1099Misc {
    FederalForm1099Misc {
        rents: 0,
        rents_use: None,
        royalties: 0,
        royalties_use: None,
        other: 0,
        other_use: None,
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
        business: None,
    }
}
fn misc(form: FederalForm1099Misc) -> FederalIncome {
    FederalIncome::Misc1099 { data: form }
}
fn k_form(gross: i64, withholding: i64, parts: Vec<(i64, FederalKUse, i64)>) -> FederalIncome {
    FederalIncome::K1099 {
        data: FederalForm1099K {
            gross: gross * 100,
            withholding: withholding * 100,
            parts: parts
                .into_iter()
                .map(|(amount, belongs, refunded)| FederalKPart {
                    amount: amount * 100,
                    belongs,
                    refunded: refunded * 100,
                })
                .collect(),
        },
    }
}
// "Lee Design": a cash business with typed receipts not on forms and supplies.
fn named_business(name: &str, receipts: i64, supplies: i64) -> FederalIncomeEntry {
    let mut entry = income(cash_business(receipts));
    entry.label = name.into();
    if supplies != 0 {
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        data.expenses =
            vec![ordinary_business_expense(BusinessExpenseCategory::Supplies, supplies * 100)];
    }
    entry
}
fn tax(r: &FederalResult) -> i64 {
    r.tax.unwrap() / 100
}
fn issue_keys(r: &FederalResult) -> Vec<&str> {
    r.issues.iter().map(|i| i.display.key.as_str()).collect()
}

// ---- A: 1099-NEC routed to Schedule C, with withholding ----------------------

fn fixture_a() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Lee Design", 5_000, 3_000));
    add(&mut d, income(nec_form(20_000, 4_800, IncomeUse::Business, None)));
    d
}

#[test]
fn a_nec_box_1a_joins_schedule_c_line_1_and_box_4_reaches_line_25b() {
    // Line 1 = 5,000 + 20,000 = 25,000; line 31 = 25,000 - 3,000 = 22,000.
    // Schedule SE: 22,000 x 0.9235 = 20,317.00; line 10 = 12.4% = 2,519.31 -> 2,519 and
    // line 11 = 2.9% = 589.19 -> 589 (the App rounds each line, as the form does);
    // line 12 = 3,108; half 1,554. AGI = 60,000 + 22,000 - 1,554 = 80,446; less 16,100
    // = 64,346. QBI deduction 20% x (22,000 - 1,554) = 4,089 (limit 20% x 64,346);
    // taxable 60,257; row 60,250-60,300, midpoint 60,275: 5,800 + 22% x 9,875 =
    // 7,972.50 -> 7,973; total tax 7,973 + 3,108 = 11,081; line 25b +4,800. The unit's
    // map summed the unrounded lines (3,108.50 -> 3,109, total 11,082): one dollar from
    // the form's whole-dollar lines. The old page gave these figures with 25,000 and
    // 4,800 typed.
    let d = fixture_a();
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 22_000);
    assert_eq!(value(&r, "self_employment_tax"), 3_108);
    assert_eq!(value(&r, "agi"), 80_446);
    assert_eq!(value(&r, "qbi_deduction"), 4_089);
    assert_eq!(value(&r, "taxable_income"), 60_257);
    assert_eq!(tax(&r), 11_081);
    assert_eq!(return_line(&r, "25b"), 480_000);
    // The same business with the 25,000 and 4,800 typed on the old page agrees.
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn a_destination_boundaries_name_the_correction() {
    // A second business and no choice: the form names none of them.
    let mut d = fixture_a();
    add(&mut d, named_business("Lee Photo", 1_000, 0));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(issue_keys(&r).contains(&"ISSUE_FORM_1099_DESTINATION_NOT_CHOSEN"));
    // Choosing one of them is enough.
    let FederalIncome::Nec1099 { data } = &mut d.income[2].income else { panic!() };
    data.business = Some(2);
    assert_eq!(
        supported(&d).lines.iter().find(|l| l.id == "business_2_net_profit").unwrap().amount,
        2_200_000
    );
    // No business: the correction link opens Self-employment.
    let mut none = complete(FederalStatus::Single);
    add(&mut none, income(nec_form(20_000, 0, IncomeUse::Business, None)));
    let r = estimate(&none);
    let issue = r
        .issues
        .iter()
        .find(|i| i.display.key == "ISSUE_FORM_1099_DESTINATION_MISSING")
        .expect("missing destination");
    assert!(
        matches!(&issue.context, IssueContext::Corrective { sections, .. } if sections == &vec![TaxSection::SelfEmployment]),
        "{:?}",
        issue.context
    );
    // The spouse's business cannot receive the filer's form (self-employment tax is
    // per person).
    let mut joint = complete(FederalStatus::Joint);
    let mut spouse_business = named_business("Spouse Shop", 5_000, 0);
    spouse_business.owner = Owner::Spouse;
    add(&mut joint, spouse_business);
    add(&mut joint, income(nec_form(20_000, 0, IncomeUse::Business, Some(2))));
    let r = estimate(&joint);
    assert!(issue_keys(&r).contains(&"ISSUE_FORM_1099_DESTINATION_NOT_CHOSEN"));
}

#[test]
fn a_not_business_income_goes_to_its_own_line() {
    // Other income: 20,000 on line 8z and only the typed 2,000 on Schedule C.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Lee Design", 5_000, 3_000));
    add(
        &mut d,
        income(nec_form(
            20_000,
            0,
            IncomeUse::Income {
                kind: OtherIncomeKind::Other { description: "one-time consulting".into() },
            },
            None,
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "other_income_3_other"), 20_000);
    assert_eq!(value(&r, "business_2_net_profit"), 2_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 20_000);
    // A hobby's income goes to line 8j with no self-employment tax.
    let mut hobby = complete(FederalStatus::Single);
    add(&mut hobby, income(nec_form(5_000, 0, IncomeUse::NotForProfit, None)));
    let r = supported(&hobby);
    assert_eq!(value(&r, "hobby_sales_income"), 5_000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 65_000);
}

#[test]
fn a_box_4_cannot_exceed_the_payment() {
    let FederalIncome::Nec1099 { data } = nec_form(25_000, 25_001, IncomeUse::Business, None)
    else {
        unreachable!()
    };
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099Nec(data.page())),
        Some("NEC_WITHHOLDING_ABOVE_PAY")
    );
}

// IRS i1099mec (Rev.December2026), p.11: box1a includes the parachute
// payment; box3 is its excess part, not another payment to add to box1a.
#[test]
fn a_nec_excess_subset_cannot_expand_withholding_or_exceed_compensation() {
    for (excess, withholding, key) in [
        (1_000_000, 1_500_000, "NEC_WITHHOLDING_ABOVE_PAY"),
        (1_000_001, 0, "NEC_EXCESS_ABOVE_TOTAL_PAY"),
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let page = Form1099Nec {
            compensation: 1_000_000,
            withholding,
            golden_parachute: excess,
            belongs: IncomeUse::NotForProfit,
            business: None,
            state_details: Default::default(),
        };
        assert!(!page.record().valid(), "invalid restored source: {key}");
        let mut restored = original.clone();
        add(&mut restored, income(FederalIncome::Nec1099 { data: page.record() }));
        assert_eq!(estimate(&restored).state, ResultState::Incomplete);
        let reply = send(
            &mut session,
            save_topic(
                &original,
                TaxSection::SelfEmployment,
                vec![unanswered_document(Owner::Taxpayer, 0, IncomeDocument::Form1099Nec(page))],
            ),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected invalid NEC source to be rejected: {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
        assert!(!session.dirty);
    }
}

#[test]
fn a_nec_compensation_allows_equal_excess_and_withholding_without_adding_boxes() {
    let page = Form1099Nec {
        compensation: 1_000_000,
        withholding: 1_000_000,
        golden_parachute: 1_000_000,
        belongs: IncomeUse::NotForProfit,
        business: None,
        state_details: Default::default(),
    };
    assert!(page.record().valid());
    assert_eq!(crate::interview::document_rule(&IncomeDocument::Form1099Nec(page)), None,);
}

// ---- B: 1099-MISC ---------------------------------------------------------------

#[test]
fn b_a_box_3_prize_reaches_line_8i_and_box_4_line_25b() {
    // Box 3 5,000, box 4 1,200, a prize received: line 8i 5,000; AGI 65,000; taxable
    // 48,900; midpoint 48,925: 1,240 + 12% x 36,525 = 5,623 (+600 = 12% x 5,000);
    // line 25b +1,200.
    let mut d = complete(FederalStatus::Single);
    let mut form = misc_form();
    form.other = 500_000;
    form.withholding = 120_000;
    form.other_use = Some(IncomeUse::Income {
        kind: OtherIncomeKind::Prize { noncash: 0, disposition: PrizeDisposition::Received },
    });
    add(&mut d, income(misc(form.clone())));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_prizes"), 5_000);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(tax(&r), 5_623);
    assert_eq!(return_line(&r, "25b"), 120_000);
    // A refused prize is not income, and its box 4 still reaches line 25b.
    let FederalIncome::Misc1099 { data } = &mut d.income[1].income else { panic!() };
    data.other_use = Some(IncomeUse::Income {
        kind: OtherIncomeKind::Prize { noncash: 0, disposition: PrizeDisposition::Refused },
    });
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_prizes"), 0);
    assert_eq!(return_line(&r, "25b"), 120_000);
}

#[test]
fn b_one_form_sends_each_box_to_its_own_page() {
    // Box 1 2,400 to the rental "12 Oak St" (its own rent 12,000 -> Schedule E line 3
    // 14,400); box 2 3,000 to a royalty entry (2,000 typed -> 5,000); box 3 to Other
    // income; box 4 1,200 once.
    let mut d = complete(FederalStatus::Single);
    let mut property = rental(12_000, 0);
    add(
        &mut d,
        income(FederalIncome::Rental {
            data: {
                property.rent = 1_200_000;
                property
            },
        }),
    );
    add(&mut d, income(royalty(2_000, 0)));
    let mut form = misc_form();
    form.rents = 240_000;
    form.rents_use = Some(FederalRentUse::RealEstate { rental: None });
    form.royalties = 300_000;
    form.royalties_use = Some(FederalRoyaltyUse::Property { property: None });
    form.withholding = 120_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_2_line3_rents"), 14_400);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 5_000);
    assert_eq!(return_line(&r, "25b"), 120_000);
}

#[test]
fn b_box_use_is_required_and_box_15_is_refused_by_name() {
    let mut form = misc_form();
    form.other = 500_000;
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099Misc(form.page())),
        Some("MISC_OTHER_NEEDS_A_USE")
    );
    form.other_use = Some(IncomeUse::NotForProfit);
    assert_eq!(crate::interview::document_rule(&IncomeDocument::Form1099Misc(form.page())), None);
    form.rents = 1;
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099Misc(form.page())),
        Some("MISC_RENTS_NEED_A_USE")
    );
    let mut d = complete(FederalStatus::Single);
    let mut form = misc_form();
    form.deferred = 50_000;
    add(&mut d, income(misc(form)));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(issue_keys(&r).contains(&"ISSUE_FORM_1099_MISC_BOX_15_SECTION_409A"));
}

#[test]
fn b_attorney_proceeds_are_the_practice_receipts_less_client_funds() {
    // Box 10 50,000 with 35,000 paid on to clients: the business's line 1 +15,000.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Law Office", 0, 0));
    let mut form = misc_form();
    form.attorney = 5_000_000;
    form.client_funds = 3_500_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 15_000);
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099Misc(
            FederalForm1099Misc { attorney: 1, client_funds: 2, ..misc_form() }.page()
        )),
        Some("MISC_CLIENT_FUNDS_ABOVE_PROCEEDS")
    );
}

#[test]
fn b_fishing_medical_and_fish_sales_always_belong_to_the_business() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Boat", 0, 0));
    let mut form = misc_form();
    form.fishing = 100_000;
    form.medical = 200_000;
    form.fish_sales = 300_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 6_000);
}

#[test]
fn b_substitute_payments_are_line_8z_and_net_investment_income() {
    let mut d = complete(FederalStatus::Single);
    let mut form = misc_form();
    form.substitute = 400_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "other_income_2_substitute_payments"), 4_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 4_000);
    assert_eq!(value(&r, "agi"), 64_000);
}

// ---- C: 1099-K reconciled against Schedule C receipts -----------------------------

fn fixture_c() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut entry = named_business("Sam's Crafts", 3_000, 6_000);
    let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
    data.expenses.push(ordinary_business_expense(BusinessExpenseCategory::Commissions, 85_200));
    // 2026-10-16: the native business page emits expenses in category order.
    // Retain the same $6,000 supplies and $852 commissions in its stored shape.
    data.expenses.sort_by_key(|record| record.category.index());
    add(&mut d, entry);
    add(
        &mut d,
        income(k_form(
            30_000,
            0,
            // D-2026-10-15-13 (2026-10-16): eleven actual parts retain the
            // same gross, refunds and routed income as the former two parts.
            (0..10)
                .map(|_| (2_840, FederalKUse::Business { business: None }, 90))
                .chain([(1_600, FederalKUse::NotIncome, 0)])
                .collect(),
        )),
    );
    d
}

#[test]
fn c_a_1099_k_business_part_less_refunds_joins_line_1_and_the_rest_is_not_taxed() {
    // Line 1 = 3,000 + 28,400 - 900 = 30,500; expenses 6,852; line 31 = 23,648.
    // Schedule SE: 23,648 x 0.9235 = 21,838.93 -> 21,839; 12.4% -> 2,708 and 2.9% ->
    // 633, line 12 = 3,341; half 1,670.50 -> 1,671. AGI = 60,000 + 23,648 - 1,671 =
    // 81,977; less 16,100 = 65,877; QBI 20% x (23,648 - 1,671) = 4,395; taxable 61,482;
    // midpoint 61,475: 5,800 + 22% x 11,075 = 8,236.50 -> 8,237; total tax 8,237 +
    // 3,341 = 11,578. The top of Schedule 1 shows 1,600, not taxed.
    let d = fixture_c();
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 23_648);
    assert_eq!(value(&r, "self_employment_tax"), 3_341);
    assert_eq!(value(&r, "agi"), 81_977);
    assert_eq!(value(&r, "qbi_deduction"), 4_395);
    assert_eq!(value(&r, "taxable_income"), 61_482);
    assert_eq!(tax(&r), 11_578);
    assert_eq!(value(&r, "form_1099k_not_income"), 1_600);
    let mut session = restart(d.clone());
    let request = open_topic(&mut session, TaxSection::SelfEmployment);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let source = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::K1099 { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!((source.gross, source.parts.len()), (3_000_000, 11));
    let reopened = open_topic(&mut session, TaxSection::SelfEmployment);
    assert!(matches!(reopened, P::SaveIncomes { .. }));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn c_the_parts_must_add_up_to_box_1a() {
    let FederalIncome::K1099 { data } = k_form(
        30_000,
        0,
        vec![
            (28_400, FederalKUse::Business { business: None }, 900),
            (600, FederalKUse::NotIncome, 0),
        ],
    ) else {
        unreachable!()
    };
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099K(data.page())),
        Some("K_PARTS_NOT_GROSS")
    );
    // The same relation protects malformed stored records before routing.
    assert!(!data.valid());
    let mut overflow = data.clone();
    overflow.parts[0].amount = i64::MAX;
    overflow.parts[1].amount = i64::MAX;
    assert_eq!(
        crate::information_returns::k_issue(&overflow.page()).unwrap().field.unwrap().square(),
        "[parts]"
    );
    assert!(!overflow.valid());
    let mut negative = data.clone();
    negative.parts[0].amount = -1;
    assert!(!negative.valid());
    let FederalIncome::K1099 { data } = k_form(1_000, 0, vec![(1_000, FederalKUse::NotIncome, 5)])
    else {
        unreachable!()
    };
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099K(data.page())),
        Some("K_REFUNDS_ONLY_ON_INCOME")
    );
}

#[test]
fn c_a_hobby_part_adds_to_the_hobby_receipts_with_no_self_employment_tax() {
    // No business: the 28,400 part is the "Sam's Crafts" hobby (receipts 3,000, cost of
    // goods sold 6,000) with refunds 900: 8j = 3,000 + 28,400 - 900 - 6,000 = 24,500,
    // no self-employment tax, and the processor fees are not deductible.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(hobby_sales(3_000, 6_000, 0)));
    add(
        &mut d,
        income(k_form(
            30_000,
            0,
            vec![
                (28_400, FederalKUse::Hobby { hobby: None }, 900),
                (1_600, FederalKUse::NotIncome, 0),
            ],
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hobby_sales_income"), 24_500);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "form_1099k_not_income"), 1_600);
}

#[test]
fn c_a_rental_part_adds_to_schedule_e_line_3() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Rental { data: rental(12_000, 0) }));
    add(&mut d, income(k_form(4_000, 0, vec![(4_000, FederalKUse::Rental { rental: None }, 500)])));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_2_line3_rents"), 12_000 + 4_000 - 500);
}

#[test]
fn c_a_personal_gain_part_adds_nothing_and_asks_for_the_sale() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(k_form(400, 0, vec![(400, FederalKUse::PersonalGain, 0)])));
    let r = supported(&d);
    assert!(noted(&r, "NOTE_1099K_GAIN_NEEDS_SALE"));
    assert_eq!(value(&r, "agi"), 60_000);
    // With a sale of personal items entered (no 1099-B) the note goes.
    add(&mut d, income(personal_sale(1_000, 600, GainTerm::Short)));
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_1099K_GAIN_NEEDS_SALE"));
}

// ---- G: Form 1099-G boxes 5, 6 and 8 --------------------------------------------

fn unemployment_with(
    rtaa: i64,
    grants: i64,
    grant_use: Option<FederalGrantUse>,
    agriculture: i64,
) -> FederalIncomeEntry {
    let mut entry = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut entry.income else { unreachable!() };
    data.rtaa = rtaa * 100;
    data.taxable_grants = grants * 100;
    data.grant_use = grant_use;
    data.agriculture_payment = agriculture * 100;
    entry
}

#[test]
fn g_rtaa_payments_are_line_8z_other_income() {
    // Box 5 2,000 -> 8z 2,000 (was refused). AGI 62,000; taxable 45,900; midpoint
    // 45,925: 1,240 + 12% x 33,525 = 5,263.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, unemployment_with(2_000, 0, None, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "other_income_2_rtaa"), 2_000);
    assert_eq!(value(&r, "agi"), 62_000);
    assert_eq!(tax(&r), 5_263);
}

#[test]
fn g_a_box_6_grant_to_a_business_is_schedule_c_income() {
    // Box 6 3,000 to the only business (no other activity): Schedule C 3,000; SE tax
    // 3,000 x 0.9235 = 2,770.50 x 15.3% = 423.89 -> 424.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Grant Shop", 0, 0));
    add(&mut d, unemployment_with(0, 3_000, Some(FederalGrantUse::Business { business: None }), 0));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 3_000);
    assert_eq!(value(&r, "self_employment_tax"), 424);
    assert_eq!(value(&r, "taxable_grants_1099g"), 0);
    // A grant to you is line 8z.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, unemployment_with(0, 3_000, Some(FederalGrantUse::You), 0));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_grants_1099g"), 3_000);
    // A box 6 amount with no use is a page error.
    let mut form = Form1099G {
        state_details: Default::default(),
        amount: 0,
        withholding: 0,
        more: Form1099GMore::default(),
    };
    form.more.taxable_grants = 300_000;
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099G(form)),
        Some("FORM_1099G_GRANT_NEEDS_A_USE")
    );
}

#[test]
fn g_a_farm_1099g_keeps_its_box_7_payments() {
    // A farmer's 1099-G with box 7 4,000: Schedule F line 4a +4,000 (before this unit the
    // box 8 checkbox left it out with a warning).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(crate::interview::document_income(&IncomeDocument::Farm(
            crate::tests::cross::interview::cash_farm(1_000_000, 0),
        ))),
    );
    let before = supported(&d);
    add(&mut d, unemployment_with(0, 0, None, 4_000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 4_000 - r_se_half(&before, &r));
}

fn r_se_half(before: &FederalResult, after: &FederalResult) -> i64 {
    value(after, "adjustments") - value(before, "adjustments")
}

#[test]
fn g_a_business_tax_refund_is_the_business_income_not_schedule_1_line_1() {
    // A 1099-G refund of 400 marked as deducted on a business schedule: Schedule C +400,
    // Schedule 1 line 1 unchanged.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Shop", 1_000, 0));
    add(
        &mut d,
        income(FederalIncome::TaxRefund {
            amount: 40_000,
            history: TaxRefundHistory::BusinessExpense { business: None },
            withholding: 0,
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 1_400);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 0);
}

// ---- H: left out and refused --------------------------------------------------------

#[test]
fn h_a_form_goes_with_a_destination_that_is_left_out() {
    // Fixture A with material_participation No: the business is left out
    // (WARN_BUSINESS_LEFT_OUT), the 1099-NEC with it with its 4,800 withholding.
    let mut d = fixture_a();
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.material_participation = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    assert!(noted(&r, "WARN_FORM_1099_DESTINATION_LEFT_OUT"));
    assert_eq!(return_line(&r, "25b"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
}

// Derived statutory fixtures, not published IRS worked examples. 2026 W-2
// instructions p.21 code K reports tax already withheld in box 2; i1099mec
// (Rev.Dec.2026) p.11 puts the determined excess subset in NEC box 3 and the
// whole compensation in box 1a. Section 4999 imposes 20% on that excess.
#[test]
fn determined_golden_parachute_inputs_add_only_schedule_2_excise_tax() {
    for (code_k, excess, expected_tax, income_tax) in [
        (0, 0, 0, 5_023),
        (100_000, 0, 1_000, 5_023),
        (0, 50_000, 100, 8_776),
        (100_000, 50_000, 1_100, 8_776),
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut wage = w2(6_000_000);
        if code_k != 0 {
            wage.more.box12.push(Box12Entry { code: W2Code::K, amount: code_k });
        }
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::W2(wage))]),
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        if excess != 0 {
            let request = save_topic(
                &session.draft.clone(),
                TaxSection::SelfEmployment,
                vec![document(
                    0,
                    IncomeDocument::Form1099Nec(Form1099Nec {
                        compensation: 2_000_000,
                        withholding: 480_000,
                        golden_parachute: excess,
                        belongs: IncomeUse::NotForProfit,
                        business: None,
                        state_details: Default::default(),
                    }),
                )],
            );
            let reply = send(&mut session, request);
            assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        }
        let result = estimate(&session.draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{result:?}");
        assert_eq!(value(&result, "agi"), if excess == 0 { 60_000 } else { 80_000 });
        assert_eq!(return_line(&result, "16"), income_tax * 100);
        assert_eq!(return_line(&result, "23"), expected_tax * 100);
        assert_eq!(value(&result, "total_tax"), income_tax + expected_tax);
        assert_eq!(return_line(&result, "25a"), 600_000);
        assert_eq!(return_line(&result, "25b"), if excess == 0 { 0 } else { 480_000 });
        if expected_tax != 0 {
            for id in [
                "schedule_2_golden_parachute_tax",
                "schedule_2_other_taxes_total",
                "schedule_2_income_and_other_taxes_subtotal",
                "schedule_2_total_additional_taxes",
            ] {
                assert_eq!(value(&result, id), expected_tax, "{id}");
            }
            assert_eq!(
                printed_line(&result, "schedule_2_golden_parachute_tax"),
                (TaxForm::Schedule2, "13k".into())
            );
        }
    }
}

#[test]
fn code_k_tax_rounds_after_aggregating_employers_and_retains_box_2_only() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let documents = [149, 49]
        .into_iter()
        .map(|tax| {
            let mut wage = w2(3_000_000);
            wage.more.box12.push(Box12Entry { code: W2Code::K, amount: tax });
            document(0, IncomeDocument::W2(wage))
        })
        .collect();
    let reply = send(&mut session, save_documents(&original, documents));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{result:?}");
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "schedule_2_golden_parachute_tax"), 2);
    assert_eq!(return_line(&result, "23"), 200);
    assert_eq!(value(&result, "total_tax"), 5_025);
    assert_eq!(return_line(&result, "25a"), 600_000);
}

#[test]
fn h_no_fixture_still_refuses_the_four_removed_situations() {
    // The four situations became enterable kinds or per-form refusals; no test source
    // names them any more (the names are spelled in pieces so this file does not).
    let removed = [
        ["Unusual", "Misc"].concat(),
        ["Golden", "Parachute"].concat(),
        ["Digital", "Asset", "Pay"].concat(),
        ["RareSituation::", "Other", "Income"].concat(),
    ];
    for (name, source) in [
        ("general", include_str!("../federal/general.rs")),
        ("general", include_str!("../activity/general.rs")),
        ("general", include_str!("../cost_recovery/general.rs")),
        ("general", include_str!("../home/general.rs")),
        ("general", include_str!("../investment/general.rs")),
        ("general", include_str!("../retirement/general.rs")),
        ("taxes", include_str!("../federal/taxes.rs")),
        ("interview", include_str!("../cross/interview.rs")),
        ("interview", include_str!("../activity/interview.rs")),
        ("interview", include_str!("../attribution/interview.rs")),
        ("interview", include_str!("../business/interview.rs")),
        ("interview", include_str!("../cost_recovery/interview.rs")),
        ("interview", include_str!("../credits/interview.rs")),
        ("interview", include_str!("../deductions/interview.rs")),
        ("interview", include_str!("../family/interview.rs")),
        ("interview", include_str!("../farm/interview.rs")),
        ("interview", include_str!("../foreign/interview.rs")),
        ("interview", include_str!("../health/interview.rs")),
        ("interview", include_str!("../home/interview.rs")),
        ("interview", include_str!("../income/interview.rs")),
        ("interview", include_str!("../investment/interview.rs")),
        ("interview", include_str!("../passthrough/interview.rs")),
        ("interview", include_str!("../payments/interview.rs")),
        ("interview", include_str!("../retirement/interview.rs")),
        ("relief_income", include_str!("relief_income.rs")),
        ("repayment", include_str!("../deductions/repayment.rs")),
        ("repayment", include_str!("../activity/repayment.rs")),
        ("repayment", include_str!("../credits/repayment.rs")),
        ("repayment", include_str!("../income/repayment.rs")),
        ("repayment", include_str!("../investment/repayment.rs")),
        ("alimony_received", include_str!("../family/alimony_received.rs")),
    ] {
        for needle in &removed {
            assert!(!source.contains(needle.as_str()), "{name} still names {needle}");
        }
    }
    assert_eq!(RareSituation::ALL.len(), 12);
}

// ---- the routing of every box ---------------------------------------------------------

#[test]
fn every_box_of_every_form_has_a_destination() {
    // The flows a form sends to other records, by slot: each nonzero box with a route
    // appears once; a local box (box 4, 8, 15, 3 as not-for-profit) never does.
    use crate::information_returns::{Kind, Slot, flows};
    let mut form = misc_form();
    form.rents = 1;
    form.rents_use = Some(FederalRentUse::Equipment { rental: None });
    form.royalties = 2;
    form.royalties_use = Some(FederalRoyaltyUse::Business);
    form.other = 3;
    form.other_use = Some(IncomeUse::Business);
    form.withholding = 4;
    form.fishing = 5;
    form.medical = 6;
    form.substitute = 8;
    form.attorney = 10;
    form.client_funds = 4;
    form.fish_sales = 11;
    form.deferred = 15;
    let found: Vec<(Slot, Kind, i64)> =
        flows(&misc(form)).into_iter().map(|f| (f.slot, f.kind, f.amount)).collect();
    assert_eq!(
        found,
        vec![
            (Slot::Rents, Kind::Equipment, 1),
            (Slot::Royalties, Kind::Business, 2),
            (Slot::Other, Kind::Business, 3),
            (Slot::Fishing, Kind::Business, 5),
            (Slot::Medical, Kind::Business, 6),
            (Slot::Attorney, Kind::Business, 6),
            (Slot::FishSales, Kind::Business, 11),
        ]
    );
    let nec = flows(&nec_form(7, 1, IncomeUse::Business, None));
    assert_eq!(nec.len(), 1);
    assert_eq!(flows(&nec_form(7, 1, IncomeUse::NotForProfit, None)).len(), 0);
}

#[test]
fn equipment_and_royalty_destinations_receive_their_boxes() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(1_500, 0)));
    let mut form = misc_form();
    form.rents = 100_000;
    form.rents_use = Some(FederalRentUse::Equipment { rental: None });
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_personal_rent"), 2_500);
}

#[test]
fn the_pages_round_trip_through_save_and_reopen() {
    // Entered on the Self-employment page, the form saves, restarts and reopens as the
    // same document with its state boxes.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let mut nec = Form1099Nec {
        compensation: 2_000_000,
        withholding: 480_000,
        golden_parachute: 0,
        belongs: IncomeUse::NotForProfit,
        business: None,
        state_details: Default::default(),
    };
    nec.state_details.payer.name = "Payer".into();
    let request = save_topic(
        &original,
        TaxSection::SelfEmployment,
        vec![unanswered_document(Owner::Taxpayer, 0, IncomeDocument::Form1099Nec(nec.clone()))],
    );
    let reply = send(&mut s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let stored = &s.draft.income[1];
    assert!(matches!(stored.income, FederalIncome::Nec1099 { .. }));
    assert!(stored.state_source.is_some());
    let mut again = restart(s.draft.clone());
    let Reply::Edit(P::SaveIncomes { documents, .. }) =
        from_review(&mut again, TaxSection::SelfEmployment)
    else {
        panic!("expected the Self-employment topic")
    };
    assert!(
        documents
            .documents
            .iter()
            .any(|item| item.document == IncomeDocument::Form1099Nec(nec.clone())),
        "{documents:?}"
    );
}

// The import declarations read the printed boxes and never the person's answers (a
// document import cannot clear them).
#[test]
fn imports_read_the_printed_boxes_and_leave_the_routing_answers_to_the_person() {
    let source = include_str!("../../income/information_returns.rs");
    let page_of = |name: &str| {
        let start = source.find(&alloc::format!("pub struct {name} {{")).unwrap();
        let body = &source[start..];
        &body[..body.find("\n}").unwrap()]
    };
    let field_is_imported = |page: &str, field: &str| {
        let at = page.find(&alloc::format!("pub {field}:")).unwrap();
        let before = &page[..at];
        let attributes = &before[before.rfind("\n\n").map_or(0, |i| i)..];
        // The attributes just above the field: back to the previous field's end.
        let last = before.rsplit_once(",\n").map_or(before, |(_, tail)| tail);
        let _ = attributes;
        last.contains("libertas_llm_doc_field")
    };
    let nec = page_of("Form1099Nec");
    for field in ["compensation", "golden_parachute", "withholding", "state_details"] {
        assert!(field_is_imported(nec, field), "NEC {field}");
    }
    for field in ["belongs", "business"] {
        assert!(!field_is_imported(nec, field), "NEC {field}");
    }
    let misc = page_of("Form1099Misc");
    for field in [
        "rents",
        "royalties",
        "other",
        "withholding",
        "fishing",
        "medical",
        "substitute",
        "attorney",
        "fish_sales",
        "deferred",
        "state_details",
    ] {
        assert!(field_is_imported(misc, field), "MISC {field}");
    }
    for field in ["rents_use", "royalties_use", "other_use", "client_funds", "business"] {
        assert!(!field_is_imported(misc, field), "MISC {field}");
    }
    let k = page_of("Form1099K");
    for field in ["gross", "withholding", "state_details"] {
        assert!(field_is_imported(k, field), "K {field}");
    }
    assert!(!field_is_imported(k, "parts"));
    // The instructions name the boxes they read and the ones they leave.
    assert!(source.contains("Do not read boxes 1b, 1c or 1d or the box 2 checkbox."));
    assert!(
        source.contains("Do not read box 9 crop insurance proceeds, box 12, boxes 13a, 13b or 14")
    );
    assert!(source.contains("Do not read boxes 1b, 1c, 1d, 2, 3 or 5a through 5l."));
    // The removed declarations: the former 1099-MISC kinds and the 1099-G box 8.
    let model = include_str!("../../federal/federal_model.rs");
    assert!(!model.contains("pub struct Prize1099Misc"));
    assert!(!model.contains("pub struct Royalty1099Misc"));
    assert!(model.contains("Do not read boxes 2, 3 or 8; a state or local refund in box 2"));
    assert!(!model.contains("return true for box 8 only when its checkbox is checked"));
}

#[test]
fn the_removed_fields_have_no_readers_left() {
    // Typed withholding and per-form state lists on Business, Rental, hobby and
    // property are gone: the forms carry them. (Farm keeps its own.)
    let model = include_str!("../../federal/federal_model.rs");
    let page = |name: &str| {
        let start = model.find(&alloc::format!("pub struct {name} {{")).unwrap();
        let body = &model[start..];
        &body[..body.find("\n}").unwrap()]
    };
    assert!(!page("Business").contains("withholding"));
    assert!(!page("Business").contains("state_documents"));
    assert!(!page("Rental").contains("state_documents"));
    assert!(!page("EquipmentRent").contains("withholding"));
    assert!(!include_str!("../../income/hobby.rs").contains("pub withholding"));
}

#[test]
fn other_income_kinds_round_trip_through_the_page() {
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let kinds = [
        OtherIncomeKind::AlaskaDividend,
        OtherIncomeKind::OlympicPrize,
        OtherIncomeKind::DigitalAssets,
        OtherIncomeKind::InmatePay,
        OtherIncomeKind::CardInsurance { premiums: 100 },
        OtherIncomeKind::UnemploymentFund { contributions: 0 },
        OtherIncomeKind::Other { description: "Reward".into() },
    ];
    let items: Vec<_> = kinds
        .iter()
        .map(|kind| {
            unanswered_document(
                Owner::Taxpayer,
                0,
                IncomeDocument::OtherIncome(OtherIncomeItem {
                    kind: kind.clone(),
                    amount: 100_000,
                    withholding: 0,
                }),
            )
        })
        .collect();
    let reply = send(&mut s, save_topic(&original, TaxSection::OtherIncome, items.clone()));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(
        s.draft
            .income
            .iter()
            .filter(|e| matches!(e.income, FederalIncome::OtherIncome { .. }))
            .count(),
        kinds.len()
    );
    // A named item reads as its description.
    assert!(s.draft.income.iter().any(|e| e.label == "Reward"));
    let mut again = restart(s.draft.clone());
    let Reply::Edit(P::SaveIncomes { documents, .. }) =
        from_review(&mut again, TaxSection::OtherIncome)
    else {
        panic!("expected the Other income topic")
    };
    for item in &items {
        assert!(
            documents.documents.iter().any(|shown| shown.document == item.document),
            "{:?}",
            item.document
        );
    }
}

#[test]
fn the_catalogs_list_every_record_a_form_can_name() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, named_business("Lee Design", 1_000, 0));
    add(&mut d, income(FederalIncome::Rental { data: rental(1_000, 0) }));
    add(&mut d, income(royalty(100, 0)));
    add(&mut d, income(personal_rent(100, 0)));
    add(&mut d, income(hobby_sales(100, 0, 0)));
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Edit(P::SaveIncomes {
        businesses,
        rentals,
        royalty_properties,
        equipment_rentals,
        hobbies,
        ..
    }) = from_review(&mut s, TaxSection::SelfEmployment)
    else {
        panic!("expected the topic")
    };
    assert_eq!(
        [
            businesses.len(),
            rentals.len(),
            royalty_properties.len(),
            equipment_rentals.len(),
            hobbies.len()
        ],
        [1, 1, 1, 1, 1]
    );
    assert_eq!(businesses[0].id, 2);
}

// ---- Community property: the amounts the forms route are divided where they enter ----

// A community-property separate return (Arizona, halves) with one $30,000 W-2 of the filer.
fn community_base() -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.income[0].community = CommunityShare::Community;
    d.next_id = 2;
    d
}

#[test]
fn a_community_nec_divides_the_business_it_joins_and_keeps_the_owners_self_employment_tax() {
    // Lee Design (receipts 5,000, supplies 3,000) plus a 1099-NEC of 20,000 with
    // 4,800 withheld, all Community: profit 22,000, half 11,000 in income; the
    // owner's Schedule SE stays on all 22,000 (3,108, half 1,554); the withholding
    // is divided like the form: 2,400. AGI = 15,000 + 11,000 - 1,554.
    let mut d = community_base();
    add(&mut d, named_business("Lee Design", 5_000, 3_000));
    add(&mut d, income(nec_form(20_000, 4_800, IncomeUse::Business, None)));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 22_000);
    assert_eq!(value(&r, "self_employment_tax"), 3_108);
    assert_eq!(value(&r, "agi"), 15_000 + 11_000 - 1_554);
    assert_eq!(return_line(&r, "25b"), 240_000);
    // The same documents owned by the spouse: half the profit, none of the tax.
    let mut theirs = community_base();
    let mut business = named_business("Lee Design", 5_000, 3_000);
    business.owner = Owner::Spouse;
    add(&mut theirs, business);
    let mut form = income(nec_form(20_000, 4_800, IncomeUse::Business, None));
    form.owner = Owner::Spouse;
    add(&mut theirs, form);
    let r = supported(&theirs);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 15_000 + 11_000);
    assert_eq!(return_line(&r, "25b"), 240_000);
    // The spouse's own (Separate) business and form add nothing to this return.
    let mut apart = theirs.clone();
    for entry in &mut apart.income[1..] {
        entry.community = CommunityShare::Separate;
    }
    let r = supported(&apart);
    assert_eq!(value(&r, "agi"), 15_000);
    assert_eq!(return_line(&r, "25b"), 0);
}

#[test]
fn a_community_1099_misc_and_1099_k_divide_what_they_route() {
    // Substitute payments of 4,000 (line 8z): half is this return's.
    let mut d = community_base();
    let mut form = misc_form();
    form.substitute = 400_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 15_000 + 2_000);
    // A 1099-K hobby part: the hobby's 24,500 is half this return's.
    let mut k = community_base();
    add(&mut k, income(hobby_sales(3_000, 6_000, 0)));
    add(
        &mut k,
        income(k_form(
            30_000,
            0,
            vec![
                (28_400, FederalKUse::Hobby { hobby: None }, 900),
                (1_600, FederalKUse::NotIncome, 0),
            ],
        )),
    );
    let r = supported(&k);
    assert_eq!(value(&r, "hobby_sales_income"), 12_250);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 15_000 + 12_250);
    // A spouse's Separate substitute payments are not on this return.
    let mut apart = community_base();
    let mut form = misc_form();
    form.substitute = 400_000;
    let mut entry = income(misc(form));
    entry.owner = Owner::Spouse;
    entry.community = CommunityShare::Separate;
    add(&mut apart, entry);
    assert_eq!(value(&supported(&apart), "agi"), 15_000);
}

#[test]
fn a_community_other_income_item_is_divided_like_any_community_item() {
    // An Alaska Permanent Fund dividend of 1,000 and a digital-asset receipt of
    // 2,000 are community income: half of each is this return's.
    for (kind, amount) in
        [(OtherIncomeKind::AlaskaDividend, 100_000), (OtherIncomeKind::DigitalAssets, 200_000)]
    {
        let mut d = community_base();
        add(
            &mut d,
            income(FederalIncome::OtherIncome {
                data: OtherIncomeItem { kind, amount, withholding: 0 },
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 15_000 + amount / 200);
    }
}

#[test]
fn wage_specials_medicaid_nec_and_strike_misc_use_their_owned_line1_kinds() {
    for (home, agi) in [(true, 0), (false, 18_000)] {
        let mut d = empty_return(FederalStatus::Head);
        add_child(&mut d);
        add(
            &mut d,
            income(nec_form(
                18_000,
                0,
                IncomeUse::Income { kind: OtherIncomeKind::MedicaidWaiver { in_your_home: home } },
                None,
            )),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(return_line(&r, "1d"), 1_800_000);
        assert_eq!(value(&r, "earned_income_credit"), 4427);
    }
    let mut d = complete(FederalStatus::Single);
    let mut form = misc_form();
    form.other = 300_000;
    form.other_use = Some(IncomeUse::Income { kind: OtherIncomeKind::StrikeBenefits });
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(return_line(&r, "1h"), 300_000);
}

// Schedule C line 2 cannot exceed line 1, including each linked source once.
// Dollars below are independent source-box sums, not result snapshots.
#[test]
fn schedule_c_receipt_bound_uses_every_existing_business_source_once() {
    let mut d = complete(FederalStatus::Single);
    let mut shop = named_business("Shop", 1_000, 0);
    let FederalIncome::Business { data } = &mut shop.income else { panic!() };
    let BusinessActivity::Ordinary { returns, .. } = &mut data.activity else { panic!() };
    *returns = 3_600_000;
    add(&mut d, shop);
    add(&mut d, income(nec_form(20_000, 0, IncomeUse::Business, None)));
    add(
        &mut d,
        income(k_form(8_000, 0, vec![(8_000, FederalKUse::Business { business: None }, 1_000)])),
    );
    let mut m = misc_form();
    m.rents = 100_000;
    m.rents_use = Some(FederalRentUse::Business);
    m.royalties = 200_000;
    m.royalties_use = Some(FederalRoyaltyUse::Business);
    m.other = 300_000;
    m.other_use = Some(IncomeUse::Business);
    m.fishing = 40_000;
    m.medical = 50_000;
    m.attorney = 90_000;
    m.client_funds = 20_000;
    m.fish_sales = 60_000;
    add(&mut d, income(misc(m)));
    add(&mut d, unemployment_with(0, 300, Some(FederalGrantUse::Business { business: None }), 0));
    add(
        &mut d,
        income(FederalIncome::TaxRefund {
            amount: 40_000,
            history: TaxRefundHistory::BusinessExpense { business: None },
            withholding: 0,
        }),
    );
    // 20,000 + (8,000-1,000) + (1,000+2,000+3,000+400+500+900-200+600)
    // + 300 + 400 = 35,900 linked; +1,000 manual -36,000 returns =900 profit.
    assert_eq!(crate::information_returns::routed_to(&d, 2), 3_590_000);
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    assert_eq!(value(&supported(&d), "business_2_net_profit"), 900);
    let context = crate::information_returns::receipt_context(&d, &[2, 3, 4]);
    assert!(context.businesses.is_empty());
    assert_eq!(context.sources.iter().map(|s| s.amount).sum::<i64>(), 890_000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessActivity::Ordinary { returns, .. } = &mut data.activity else { panic!() };
    *returns = 3_690_001;
    assert_eq!(
        crate::information_returns::receipt_rule(&d),
        Some("BUSINESS_RETURNS_ABOVE_RECEIPTS")
    );
    assert!(issue_keys(&estimate(&d)).contains(&"BUSINESS_RETURNS_ABOVE_RECEIPTS"));
}

#[test]
fn schedule_c_receipt_bound_automatic_route_tracks_current_same_owner_candidates() {
    let mut d = fixture_a();
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessActivity::Ordinary { returns, .. } = &mut data.activity else { panic!() };
    *returns = 600_000; // More than manual5,000; covered by linked20,000.
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    let mut spouse = named_business("Spouse shop", 0, 0);
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    add(&mut d, named_business("Second shop", 0, 0));
    assert_eq!(crate::information_returns::routed_to(&d, 2), 0);
    assert_eq!(
        crate::information_returns::receipt_rule(&d),
        Some("BUSINESS_RETURNS_ABOVE_RECEIPTS")
    );
    let FederalIncome::Nec1099 { data } = &mut d.income[2].income else { panic!() };
    data.business = Some(2);
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    // A destination ID owned by the spouse cannot supply this business's receipts.
    d.income[2].owner = Owner::Spouse;
    assert_eq!(
        crate::information_returns::receipt_rule(&d),
        Some("BUSINESS_RETURNS_ABOVE_RECEIPTS")
    );
}

#[test]
fn schedule_c_receipt_bound_save_accepts_linked_receipts_and_ignores_forged_context() {
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let mut shop = crate::tests::cross::interview::business(100_000);
    shop.returns = 600_000;
    let nec = Form1099Nec {
        compensation: 1_000_000,
        withholding: 0,
        golden_parachute: 0,
        belongs: IncomeUse::Business,
        business: None,
        state_details: Default::default(),
    };
    let request = save_topic(
        &original,
        TaxSection::SelfEmployment,
        vec![
            document(0, IncomeDocument::Business(shop.clone())),
            document(0, IncomeDocument::Form1099Nec(nec)),
        ],
    );
    let reply = send(&mut s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let saved = s.draft.clone();
    let mut restarted = restart(saved.clone());
    let Reply::Edit(P::SaveIncomes { documents, .. }) =
        from_review(&mut restarted, TaxSection::SelfEmployment)
    else {
        panic!()
    };
    assert!(documents.documents.iter().any(|i| matches!(&i.document, IncomeDocument::Business(b) if b.returns == 600_000 && b.receipts == 100_000)));
    let mut request = save_topic(
        &saved,
        TaxSection::SelfEmployment,
        vec![document(saved.income[1].id, IncomeDocument::Business(shop.clone()))],
    );
    let P::SaveIncomes { documents, .. } = &mut request else { panic!() };
    documents.receipt_context.sources.push(crate::BusinessReceiptSource {
        owner: Owner::Taxpayer,
        business: None,
        amount: 99_000_000,
    });
    let reply = send(&mut s, request);
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(s.draft, saved);
}
