extern crate std;

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
use crate::tests::support::client::survey::survey_refusal as reject_atomic;

pub(in crate::tests) fn nec_form(
    compensation: i64,
    withholding: i64,
    belongs: Option<IncomeUse>,
) -> FederalIncome {
    FederalIncome::Nec1099 {
        data: nec_source(FederalForm1099Nec {
            cash_tips: 0,
            tip_occupation_codes: Default::default(),
            overtime: 0,
            compensation: compensation * 100,
            withholding: withholding * 100,
            golden_parachute: 0,
            belongs,
        }),
    }
}
pub(in crate::tests) fn misc_form() -> FederalForm1099Misc {
    FederalForm1099Misc {
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        crop_insurance: 0,
        component_ids: Default::default(),
        rents: 0,

        royalties: 0,

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
    }
}
pub(in crate::tests) fn misc(form: FederalForm1099Misc) -> FederalIncome {
    FederalIncome::Misc1099 { data: misc_source(form) }
}
fn k_form(
    gross: i64,
    withholding: i64,
    parts: Vec<(i64, Option<FederalKUse>, i64)>,
) -> FederalIncome {
    FederalIncome::K1099 {
        data: k_source(FederalForm1099K {
            cash_tips: 0,
            tip_occupation_codes: Default::default(),
            gross: gross * 100,
            withholding: withholding * 100,
            parts: parts
                .into_iter()
                .map(|(amount, belongs, refunded)| FederalKPart {
                    id: 0,
                    amount: amount * 100,
                    belongs,
                    refunded: refunded * 100,
                })
                .collect(),
        }),
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
    add(&mut d, income(nec_form(20_000, 4_800, None)));
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
    let issue = r
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN")
        .expect("unclaimed destination");
    assert!(
        matches!(&issue.context, IssueContext::Corrective { sections, .. } if sections == &vec![TaxSection::SelfEmployment]),
        "{:?}",
        issue.context
    );
    // Choosing one of them is enough.
    let key = d.income[2].id;
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.related = vec![key];
    assert_eq!(
        supported(&d).lines.iter().find(|l| l.id == "business_2_net_profit").unwrap().amount,
        2_200_000
    );
    // No business: the correction link opens Self-employment.
    let mut none = complete(FederalStatus::Single);
    add(&mut none, income(nec_form(20_000, 0, None)));
    let r = estimate(&none);
    let issue = r
        .issues
        .iter()
        .find(|i| i.display.key == "ISSUE_RELATED_FORM_ACTIVITY_MISSING")
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
    spouse_business.owner = crate::DocumentOwner::Spouses;
    add(&mut joint, spouse_business);
    add(&mut joint, income(nec_form(20_000, 0, None)));
    let key = joint.income.last().unwrap().id;
    let FederalIncome::Business { data } = &mut joint.income[1].income else { panic!() };
    data.related = vec![key];
    let r = estimate(&joint);
    // 2026-10-23: accepted related keys must satisfy the recipient's existing eligibility before calculation.
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(issue_keys(&r).contains(&"PROBLEM_RELATED_FORM_UNAVAILABLE"));
    let (_, issue) =
        crate::related_forms::source_issue(&joint).expect("ineligible stored recipient");
    assert_eq!(issue.field.unwrap().square(), "[Business/related/forms/0/key]");
    // Shared components name every actual repair page. Without an activity,
    // their creation links follow the declared applicability of that component.
    for (slot, activities, actual_pages, creation_pages) in [
        (
            0,
            vec![
                cash_business(0),
                FederalIncome::Rental { data: rental(0, 0) },
                hobby_sales(0, 0, 0),
            ],
            vec![TaxSection::SelfEmployment, TaxSection::Rentals, TaxSection::OtherIncome],
            vec![TaxSection::SelfEmployment, TaxSection::Rentals, TaxSection::OtherIncome],
        ),
        (
            1,
            vec![FederalIncome::Rental { data: rental(0, 0) }, personal_rent(0, 0)],
            vec![TaxSection::Rentals, TaxSection::OtherIncome],
            vec![TaxSection::SelfEmployment, TaxSection::Rentals, TaxSection::OtherIncome],
        ),
        (
            2,
            vec![royalty(0, 0), royalty(0, 0)],
            vec![TaxSection::Rentals],
            vec![TaxSection::SelfEmployment, TaxSection::Rentals],
        ),
        (
            3,
            vec![hobby_sales(0, 0, 0), hobby_sales(0, 0, 0)],
            vec![TaxSection::OtherIncome],
            vec![TaxSection::SelfEmployment, TaxSection::OtherIncome],
        ),
    ] {
        for entered in [false, true] {
            let mut draft = complete(FederalStatus::Single);
            if entered {
                for activity in &activities {
                    add(&mut draft, income(activity.clone()));
                }
            }
            let source = if slot == 0 {
                k_form(1_000, 0, vec![(1_000, None, 0)])
            } else {
                let mut form = misc_form();
                match slot {
                    1 => form.rents = 100_000,
                    2 => form.royalties = 100_000,
                    _ => form.other = 100_000,
                }
                misc(form)
            };
            add(&mut draft, income(source));
            let result = estimate(&draft);
            let key = if entered {
                "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"
            } else {
                "ISSUE_RELATED_FORM_ACTIVITY_MISSING"
            };
            let issue = result.issues.iter().find(|issue| issue.display.key == key).unwrap();
            let expected = if entered { &actual_pages } else { &creation_pages };
            assert!(
                matches!(&issue.context, IssueContext::Corrective { sections, .. } if sections == expected),
                "component {slot}, entered {entered}: {:?}",
                issue.context
            );
        }
    }
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
            Some(IncomeUse::Income {
                kind: OtherIncomeKind::Other { description: "one-time consulting".into() },
            }),
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "other_income_3_other"), 20_000);
    assert_eq!(value(&r, "business_2_net_profit"), 2_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 20_000);
    // A hobby's income goes to line 8j with no self-employment tax.
    let mut hobby = complete(FederalStatus::Single);
    add(&mut hobby, income(nec_form(5_000, 0, Some(IncomeUse::NotForProfit))));
    let r = supported(&hobby);
    assert_eq!(value(&r, "hobby_sales_income"), 5_000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 65_000);
}

#[test]
fn a_box_4_cannot_exceed_the_payment() {
    let FederalIncome::Nec1099 { data } = nec_form(25_000, 25_001, None) else { unreachable!() };
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::SelfEmployment {
            value: crate::SelfEmploymentDocument::Form1099Nec(data.form.clone())
        }),
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
            payer: Default::default(),
            recipient: Default::default(),
            cash_tips: 0,
            tip_occupation_codes: Default::default(),
            overtime: 0,
            compensation: 1_000_000,
            withholding,
            golden_parachute: excess,
            belongs: Some(IncomeUse::NotForProfit),
            state_details: Default::default(),
        };
        assert!(!FederalForm1099Nec::from(&page).valid(), "invalid restored source: {key}");
        let mut restored = original.clone();
        add(
            &mut restored,
            income(FederalIncome::Nec1099 { data: nec_source(FederalForm1099Nec::from(&page)) }),
        );
        assert_eq!(estimate(&restored).state, ResultState::Incomplete);
        let reply = {
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                vec![unanswered_document(
                    Owner::Taxpayer,
                    0,
                    IncomeSource::SelfEmployment {
                        value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(page)),
                    },
                )],
            );
            send(&mut session, request)
        };
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
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        compensation: 1_000_000,
        withholding: 1_000_000,
        golden_parachute: 1_000_000,
        belongs: Some(IncomeUse::NotForProfit),
        state_details: Default::default(),
    };
    assert!(FederalForm1099Nec::from(&page).valid());
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::SelfEmployment {
            value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(page))
        }),
        None,
    );
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
    form.royalties = 300_000;
    form.withholding = 120_000;
    add(&mut d, income(misc(form)));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_2_line3_rents"), 14_400);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 5_000);
    assert_eq!(return_line(&r, "25b"), 120_000);
}

#[test]
fn b_unclaimed_boxes_need_activities_and_box_15_is_refused_by_name() {
    // 2026-10-23: activity-owned relations replace document-side business choices.
    // An absent nonactivity classification is valid input; an unclaimed amount
    // remains unavailable rather than becoming inferred income or zero.
    let mut form = misc_form();
    form.other = 500_000;
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Form1099Misc(misc_document(misc_source(
                form.clone()
            )))
        }),
        None
    );
    form.other_use = Some(IncomeUse::NotForProfit);
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Form1099Misc(misc_document(misc_source(
                form.clone()
            )))
        }),
        None
    );
    for (rents, royalties) in [(1, 0), (0, 1)] {
        let mut unclaimed = misc_form();
        unclaimed.rents = rents;
        unclaimed.royalties = royalties;
        assert_eq!(
            crate::interview::document_rule(&IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(misc_document(misc_source(
                    unclaimed.clone()
                )))
            }),
            None
        );
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(misc(unclaimed)));
        let result = estimate(&draft);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(issue_keys(&result).contains(&"ISSUE_RELATED_FORM_ACTIVITY_MISSING"));
    }
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
    // The client funds are a topic answer, not printed on the paper: the page rule reads them.
    assert_eq!(
        crate::information_returns::misc_issue(&crate::Form1099Misc::from(&misc_source(
            FederalForm1099Misc { attorney: 1, client_funds: 2, ..misc_form() }
        )))
        .map(|issue| issue.key),
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
                .map(|_| (2_840, None, 90))
                .chain([(1_600, Some(FederalKUse::NotIncome), 0)])
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
    assert_eq!((source.form.gross, source.parts.len()), (Some(3_000_000), 11));
    let reopened = open_topic(&mut session, TaxSection::SelfEmployment);
    assert!(matches!(reopened, P::SaveIncomes { .. }));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn c_the_parts_must_add_up_to_box_1a() {
    let FederalIncome::K1099 { data } =
        k_form(30_000, 0, vec![(28_400, None, 900), (600, Some(FederalKUse::NotIncome), 0)])
    else {
        unreachable!()
    };
    assert_eq!(
        crate::information_returns::k_issue(&FederalForm1099K::from(&data).page())
            .map(|issue| issue.key),
        Some("K_PARTS_NOT_GROSS")
    );
    // Printed corrections retain allocations in storage; the actual topic
    // requires reconciliation before the estimate is available.
    assert!(FederalForm1099K::from(&data).valid());
    let FederalIncome::K1099 { data } =
        k_form(1_000, 0, vec![(1_000, Some(FederalKUse::NotIncome), 5)])
    else {
        unreachable!()
    };
    assert_eq!(
        crate::information_returns::k_issue(&FederalForm1099K::from(&data).page())
            .map(|issue| issue.key),
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
            vec![(28_400, None, 900), (1_600, Some(FederalKUse::NotIncome), 0)],
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
    add(&mut d, income(k_form(4_000, 0, vec![(4_000, None, 500)])));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_2_line3_rents"), 12_000 + 4_000 - 500);
}

#[test]
fn c_a_personal_gain_part_adds_nothing_and_asks_for_the_sale() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(k_form(400, 0, vec![(400, Some(FederalKUse::PersonalGain), 0)])));
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
    grant_use: Option<GrantUse>,
    agriculture: i64,
) -> FederalIncomeEntry {
    let mut entry = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut entry.income else { unreachable!() };
    data.form.rtaa_payments = Some(rtaa * 100);
    data.form.taxable_grants = Some(grants * 100);
    data.grant_use = grant_use;
    data.form.agriculture_payments = Some(agriculture * 100);
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
    add(&mut d, unemployment_with(0, 3_000, None, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 3_000);
    assert_eq!(value(&r, "self_employment_tax"), 424);
    assert_eq!(value(&r, "taxable_grants_1099g"), 0);
    // A grant to you is line 8z.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, unemployment_with(0, 3_000, Some(GrantUse::You), 0));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_grants_1099g"), 3_000);
    // 2026-10-23: a grant without a personal classification is related from its
    // activity. The document accepts it, while no eligible activity remains unknown.
    let form = Form1099G { taxable_grants: 300_000, ..Default::default() };
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::GovernmentPayment {
            refund_state: None,
            refund_locality: Default::default(),
            value: government_document(form)
        }),
        None
    );
    let mut unclaimed = complete(FederalStatus::Single);
    add(&mut unclaimed, unemployment_with(0, 3_000, None, 0));
    let result = estimate(&unclaimed);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(issue_keys(&result).contains(&"ISSUE_RELATED_FORM_ACTIVITY_MISSING"));
}

#[test]
fn g_a_farm_1099g_keeps_its_box_7_payments() {
    // A farmer's 1099-G with box 7 4,000: Schedule F line 4a +4,000 (before this unit the
    // box 8 checkbox left it out with a warning).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(crate::interview::document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::Farm(crate::tests::cross::interview::cash_farm(
                1_000_000, 0,
            )),
        })),
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
        income(refund_income(40_000, 0, TaxRefundHistory::BusinessExpense { business: None })),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 1_400);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 0);
}

// ---- H: left out and refused --------------------------------------------------------

#[test]
fn h_a_form_goes_with_a_destination_that_is_left_out() {
    // Fixture A with unlisted special treatment: the business is left out
    // (WARN_BUSINESS_LEFT_OUT), the 1099-NEC with it with its 4,800 withholding.
    let mut d = fixture_a();
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    // 2026-10-24: current passive C is calculated; unlisted special
    // treatment remains an actual unsupported scope for omission controls.
    data.special_treatment = true;
    let r = supported(&d);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    assert!(noted(&r, "WARN_FORM_1099_DESTINATION_LEFT_OUT"));
    assert_eq!(return_line(&r, "25b"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn an_unclaimed_form_stays_unresolved_after_another_activity_is_left_out() {
    let mut draft = fixture_a();
    add(&mut draft, named_business("Retained business", 1_000, 0));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    // 2026-10-24: current passive C is calculated; unlisted special
    // treatment remains an actual unsupported scope for omission controls.
    data.special_treatment = true;
    let accepted = draft.clone();
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(issue_keys(&result).contains(&"ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"));
    let mut notes = Vec::new();
    let working = crate::unknowns::supported(&draft, &mut notes).expect("omitted activity");
    let FederalIncome::Business { data } =
        &working.income.iter().find(|entry| entry.id == 4).unwrap().income
    else {
        panic!()
    };
    assert_eq!(data.receipts(), 100_000);
    assert!(working.income.iter().any(|entry| entry.id == draft.income[2].id));
    assert!(!noted(&result, "WARN_FORM_1099_DESTINATION_LEFT_OUT"));
    assert_eq!(draft, accepted);
}

#[test]
fn an_explicit_form_claim_is_not_rerouted_when_its_activity_is_left_out() {
    let mut draft = fixture_a();
    add(&mut draft, named_business("Retained business", 1_000, 0));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    // 2026-10-24: current passive C is calculated; unlisted special
    // treatment remains an actual unsupported scope for omission controls.
    data.special_treatment = true;
    let source = draft.income[2].id;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.related.push(source);
    let accepted = draft.clone();
    let result = supported(&draft);
    assert!(noted(&result, "WARN_FORM_1099_DESTINATION_LEFT_OUT"));
    assert_eq!(value(&result, "business_4_net_profit"), 1_000);
    assert_eq!(return_line(&result, "25b"), 0);
    assert_eq!(draft, accepted);
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
        let mut original = complete(FederalStatus::Single);
        if excess != 0 {
            // The business that receives the NEC: the page refuses it without one.
            add(&mut original, income(cash_business(0)));
        }
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut wage = w2(6_000_000);
        if code_k != 0 {
            wage.more.box12.push(Box12Entry { code: W2Code::K, amount: code_k });
        }
        let reply = {
            let request = save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) },
                )],
            );
            send(&mut session, request)
        };
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        if excess != 0 {
            let sources = vec![document(
                0,
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(Form1099Nec {
                        payer: Default::default(),
                        recipient: Default::default(),
                        cash_tips: 0,
                        tip_occupation_codes: Default::default(),
                        overtime: 0,
                        compensation: 2_000_000,
                        withholding: 480_000,
                        golden_parachute: excess,
                        belongs: Some(IncomeUse::NotForProfit),
                        state_details: Default::default(),
                    })),
                },
            )];
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                sources.clone(),
            );
            let reply = send(&mut session, request);
            assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
            crate::tests::support::client::income_pages::save_information_return_answers(
                &mut session,
                &sources,
            );
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
            document(0, IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) })
        })
        .collect();
    let reply = {
        let request = save_documents(&mut session, documents);
        send(&mut session, request)
    };
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
    assert_eq!(RareSituation::ALL.len(), 10);
}

// ---- the routing of every box ---------------------------------------------------------

#[test]
fn every_box_of_every_form_has_a_destination() {
    // The flows a form sends to other records, by slot: each nonzero box with a route
    // appears once; a local box (box 4, 8, 15, 3 as not-for-profit) never does.
    use crate::information_returns::{Slot, flows};
    let mut form = misc_form();
    form.rents = 1;
    form.royalties = 2;
    form.other = 3;
    form.other_use = None;
    form.withholding = 4;
    form.fishing = 5;
    form.medical = 6;
    form.substitute = 8;
    form.attorney = 10;
    form.client_funds = 4;
    form.fish_sales = 11;
    form.deferred = 15;
    let found: Vec<(Slot, Option<crate::information_returns::Kind>, i64)> =
        flows(&misc(form)).into_iter().map(|f| (f.slot, f.kind, f.amount)).collect();
    assert_eq!(
        found,
        vec![
            (Slot::Rents, None, 1),
            (Slot::Royalties, None, 2),
            (Slot::Other, None, 3),
            (Slot::Fishing, None, 5),
            (Slot::Medical, None, 6),
            (Slot::Attorney, None, 6),
            (Slot::FishSales, None, 11),
        ]
    );
    let nec = flows(&nec_form(7, 1, None));
    assert_eq!(nec.len(), 1);
    assert_eq!(flows(&nec_form(7, 1, Some(IncomeUse::NotForProfit))).len(), 0);
}

#[test]
fn equipment_and_royalty_destinations_receive_their_boxes() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(1_500, 0)));
    let mut form = misc_form();
    form.rents = 100_000;
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
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        compensation: 2_000_000,
        withholding: 480_000,
        golden_parachute: 0,
        belongs: Some(IncomeUse::NotForProfit),
        state_details: Default::default(),
    };
    nec.payer.name = "Payer".into();
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut s,
        vec![unanswered_document(
            Owner::Taxpayer,
            0,
            IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(nec.clone())),
            },
        )],
    );
    let reply =
        crate::tests::support::client::income_pages::save_source_fixture_request(&mut s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let stored = &s.draft.income[1];
    assert!(matches!(stored.income, FederalIncome::Nec1099 { .. }));
    assert_eq!(stored.printed_names().unwrap().issuer.as_ref().unwrap().name, "Payer");
    assert!(stored.state_source.is_none());
    let mut again = restart(s.draft.clone());
    let documents = crate::tests::support::client::income_pages::all_documents(&mut again);
    assert!(
        documents.iter().any(|item| item.document
            == expected_named_source(
                &s.draft,
                Owner::Taxpayer,
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(nec.clone()))
                }
            )),
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
    for field in [
        "compensation",
        "cash_tips",
        "tip_occupation_codes",
        "overtime",
        "golden_parachute",
        "withholding",
        "state_details",
    ] {
        assert!(field_is_imported(nec, field), "NEC {field}");
    }
    assert!(!field_is_imported(nec, "belongs"), "NEC belongs");
    let misc = page_of("Form1099Misc");
    for field in [
        "rents",
        "royalties",
        "other",
        "withholding",
        "fishing",
        "medical",
        "substitute",
        "crop_insurance",
        "attorney",
        "fish_sales",
        "deferred",
        "cash_tips",
        "tip_occupation_codes",
        "overtime",
        "state_details",
    ] {
        assert!(field_is_imported(misc, field), "MISC {field}");
    }
    for field in ["other_use", "client_funds"] {
        assert!(!field_is_imported(misc, field), "MISC {field}");
    }
    let k = page_of("Form1099K");
    for field in ["gross", "cash_tips", "tip_occupation_codes", "withholding", "state_details"] {
        assert!(field_is_imported(k, field), "K {field}");
    }
    assert!(!field_is_imported(k, "parts"));
    // The instructions name the boxes they read and the ones they leave.
    assert!(source.contains("Read box 1b cash tips and box 1d overtime compensation"));
    assert!(source.contains("box 9 crop insurance proceeds"));
    assert!(source.contains("Read box 13a cash tips and box 14 overtime compensation"));
    assert!(source.contains("Read box 1c cash tips"));
    assert!(source.contains("Do not read boxes 1b, 2, 3 or 5a through 5l."));
    // The removed declarations: the former 1099-MISC kinds and the 1099-G box 8.
    let model = crate::federal_model::HUB_SOURCE;
    assert!(!model.contains("pub struct Prize1099Misc"));
    assert!(!model.contains("pub struct Royalty1099Misc"));
    // The 1099-G paper now reads every printed box; box 8 only when its checkbox is marked.
    assert!(model.contains(
        "Check box 8 only when the 'Check if box 2 is trade or business income' box is marked"
    ));
    assert!(!model.contains("return true for box 8 only when its checkbox is checked"));
}

#[test]
fn the_removed_fields_have_no_readers_left() {
    // Typed withholding and per-form state lists on Business, Rental, hobby and
    // property are gone: the forms carry them. (Farm keeps its own.)
    let model = crate::federal_model::HUB_SOURCE;
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
                IncomeSource::Record {
                    value: crate::IncomeRecord::OtherIncome(OtherIncomeItem {
                        kind: kind.clone(),
                        amount: 100_000,
                        withholding: 0,
                    }),
                },
            )
        })
        .collect();
    let reply = {
        let request = save_topic(&mut s, TaxSection::OtherIncome, items.clone());
        send(&mut s, request)
    };
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
            documents.documents.iter().any(|shown| shown.document
                == crate::IncomeRecordItem::from_income(item.clone())
                    .expect("other income record")
                    .document),
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
    d.deductions.as_mut().unwrap().itemize_override = None;
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
    add(&mut d, income(nec_form(20_000, 4_800, None)));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 22_000);
    assert_eq!(value(&r, "self_employment_tax"), 3_108);
    assert_eq!(value(&r, "agi"), 15_000 + 11_000 - 1_554);
    assert_eq!(return_line(&r, "25b"), 240_000);
    // The same documents owned by the spouse: half the profit, none of the tax.
    let mut theirs = community_base();
    let mut business = named_business("Lee Design", 5_000, 3_000);
    business.owner = crate::DocumentOwner::Spouses;
    add(&mut theirs, business);
    let mut form = income(nec_form(20_000, 4_800, None));
    form.owner = crate::DocumentOwner::Spouses;
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
            vec![(28_400, None, 900), (1_600, Some(FederalKUse::NotIncome), 0)],
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
    entry.owner = crate::DocumentOwner::Spouses;
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
                Some(IncomeUse::Income {
                    kind: OtherIncomeKind::MedicaidWaiver { in_your_home: home },
                }),
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
    add(&mut d, income(nec_form(20_000, 0, None)));
    add(&mut d, income(k_form(8_000, 0, vec![(8_000, None, 1_000)])));
    let mut m = misc_form();
    m.rents = 100_000;
    m.royalties = 200_000;
    m.other = 300_000;
    m.other_use = None;
    m.fishing = 40_000;
    m.medical = 50_000;
    m.attorney = 90_000;
    m.client_funds = 20_000;
    m.fish_sales = 60_000;
    add(&mut d, income(misc(m)));
    add(&mut d, unemployment_with(0, 300, None, 0));
    add(
        &mut d,
        income(refund_income(40_000, 0, TaxRefundHistory::BusinessExpense { business: None })),
    );
    // 20,000 + (8,000-1,000) + (1,000+2,000+3,000+400+500+900-200+600)
    // + 300 + 400 = 35,900 linked; +1,000 manual -36,000 returns =900 profit.
    assert_eq!(crate::information_returns::routed_to(&d, 2), 3_590_000);
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    assert_eq!(value(&supported(&d), "business_2_net_profit"), 900);
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
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    add(&mut d, named_business("Second shop", 0, 0));
    assert_eq!(crate::information_returns::routed_to(&d, 2), 0);
    assert_eq!(
        crate::information_returns::receipt_rule(&d),
        Some("BUSINESS_RETURNS_ABOVE_RECEIPTS")
    );
    let key = d.income[2].id;
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.related = vec![key];
    assert_eq!(crate::information_returns::receipt_rule(&d), None);
    // A destination ID owned by the spouse cannot supply this business's receipts.
    d.income[2].owner = crate::DocumentOwner::Spouses;
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
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        compensation: 1_000_000,
        withholding: 0,
        golden_parachute: 0,
        belongs: None,
        state_details: Default::default(),
    };
    crate::tests::support::client::income_pages::save_each_topic(
        &mut s,
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(shop.clone()) },
            ),
            document(
                0,
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099Nec(nec_document(nec)),
                },
            ),
        ],
    );
    let saved = s.draft.clone();
    let mut restarted = restart(saved.clone());
    let Reply::Edit(P::SaveIncomes { documents, .. }) =
        from_review(&mut restarted, TaxSection::SelfEmployment)
    else {
        panic!()
    };
    assert!(documents.documents.iter().any(|i| matches!(&i.document, crate::IncomeRecord::Business(b) if b.returns == 600_000 && b.receipts == 100_000)));
    let business = saved
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    shop.returns = 1_100_001;
    let mut request = save_topic(
        &mut restarted,
        TaxSection::SelfEmployment,
        vec![document(
            business,
            IncomeSource::Record { value: crate::IncomeRecord::Business(shop.clone()) },
        )],
    );
    let P::SaveIncomes { source_documents, .. } = &mut request else { panic!() };
    let Some(crate::TopicDocumentViews::SelfEmployment { documents, .. }) = source_documents else {
        panic!()
    };
    let crate::SelfEmploymentDocument::Form1099Nec(source) = &mut documents[0].value.document
    else {
        panic!()
    };
    source.compensation = Some(99_000_000);
    let reply = send(&mut restarted, request);
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(restarted.draft, saved);
}

#[test]
fn crop_insurance_printed_box_round_trips_and_supports_form_withholding() {
    let record =
        FederalForm1099Misc { crop_insurance: 125_000, withholding: 12_500, ..misc_form() };
    let source = misc_source(record.clone());
    assert_eq!(source.form.crop_insurance, Some(125_000));
    assert_eq!(FederalForm1099Misc::from(&source), record);
    let page = Form1099Misc::from(&source);
    assert!(record.valid());
    assert_eq!(crate::information_returns::misc_issue(&page), None);
    assert!(!crate::information_returns::nothing_left(&misc(record.clone())));
    let mut excessive = page;
    excessive.withholding = 125_001;
    let issue = crate::information_returns::misc_issue(&excessive).unwrap();
    assert_eq!(issue.key, "MISC_WITHHOLDING_ABOVE_INCOME");
}

#[test]
fn related_business_rejects_two_new_claimants_at_the_first_submitted_key() {
    use crate::tests::cross::interview::{business, save_businesses};
    let draft = fixture_a();
    let key = draft.income[2].id;
    let mut session = Interview::new(TEST_USER, draft.clone());
    let mut first = business(0);
    first.related.forms.push(RelatedFormView { key, summary: None });
    let second = first.clone();
    let reply = save_businesses(&mut session, vec![first, second]);
    let Reply::Response(P::Problem { field, .. }) = reply else { panic!("{reply:?}") };
    assert_eq!(
        field.as_deref(),
        Some("[documents/documents/0/document/Business/related/forms/0/key]")
    );
    assert_eq!(session.draft, draft);
}

#[test]
fn related_business_moves_claims_atomically_and_removal_preserves_the_document() {
    use crate::tests::cross::interview::{accepted, page_documents};
    let mut draft = fixture_a();
    add(&mut draft, named_business("Second shop", 0, 0));
    let key = draft.income[2].id;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.related = vec![key];
    let source = draft.income[2].clone();
    let second = draft.income[3].id;
    let mut session = Interview::new(TEST_USER, draft);
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::SelfEmployment));
    for row in &mut rows {
        let IncomeSource::Record { value: crate::IncomeRecord::Business(b) } = &mut row.document
        else {
            continue;
        };
        b.related.forms.clear();
        if row.id == second {
            b.related.forms.push(RelatedFormView { key, summary: None });
        }
    }
    let request = save_topic(&mut session, TaxSection::SelfEmployment, rows);
    accepted(&send(&mut session, request));
    assert_eq!(crate::related_forms::effective_claimants(&session.draft).get(&key), Some(&second));
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::SelfEmployment));
    for row in &mut rows {
        if let IncomeSource::Record { value: crate::IncomeRecord::Business(b) } = &mut row.document
        {
            b.related.forms.clear();
        }
    }
    // Unlinking the paper from every business is refused by the page that owns
    // the activity; the paper and its claim stay as the last save left them.
    let request = save_topic(&mut session, TaxSection::SelfEmployment, rows);
    let before = session.draft.clone();
    assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
    // The saves print the accepted names on the document; nothing else of it changes.
    let mut kept = session.draft.income.iter().find(|entry| entry.id == key).cloned().unwrap();
    kept.printed = source.printed.clone();
    assert_eq!(kept, source);
    assert_eq!(crate::related_forms::effective_claimants(&session.draft).get(&key), Some(&second));
}

#[test]
fn related_business_first_composition_gets_choices_and_refreshes_for_the_owner() {
    use crate::tests::cross::interview::{accepted, business, save_businesses};
    let mut draft = complete(FederalStatus::Joint);
    add(&mut draft, income(nec_form(20_000, 0, None)));
    let key = draft.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, draft);
    let cookie = session.draft.cookie.clone();
    let revision = session.draft.revision;
    let options = |owner| P::BusinessRelatedOptions {
        user: TEST_USER,
        cookie: cookie.clone(),
        revision,
        id: 0,
        owner,
        source: BusinessSource::UnitedStates,
        gambling: None,
    };
    let Reply::Response(P::BusinessRelatedChoices { candidates }) =
        send(&mut session, options(Owner::Taxpayer))
    else {
        panic!()
    };
    assert_eq!(candidates.iter().map(|choice| choice.id).collect::<Vec<_>>(), vec![key]);
    let Reply::Response(P::BusinessRelatedChoices { candidates: spouse_choices }) =
        send(&mut session, options(Owner::Spouse))
    else {
        panic!()
    };
    assert!(spouse_choices.is_empty());
    let mut page = business(0);
    page.related.candidates = candidates;
    page.related.forms.push(RelatedFormView { key, summary: None });
    accepted(&save_businesses(&mut session, vec![page]));
    assert_eq!(
        crate::information_returns::routed_to(
            &session.draft,
            session.draft.income.last().unwrap().id
        ),
        2_000_000
    );
    if let Some(path) = std::env::var_os("TAX_G59_HOST_AVRO") {
        let host = crate::tests::support::client::income_pages::open_documents(&mut session);
        assert!(matches!(host, P::SaveDocuments { .. }));
        std::fs::write(path, host.to_avro()).expect("write native document center");
    }
}

#[test]
fn farm_related_forms_offer_only_farm_components_and_enforce_exclusive_claims() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(cash_business(0)));
    add(&mut draft, income(raised_farm(0, 0)));
    let farm = draft.income.last().unwrap().id;
    add(&mut draft, income(raised_farm(0, 0)));
    let other = draft.income.last().unwrap().id;
    add(
        &mut draft,
        income(misc(FederalForm1099Misc { rents: 50_000, crop_insurance: 125_000, ..misc_form() })),
    );
    let center = crate::related_forms::catalog_for(&draft, farm);
    assert_eq!(center.len(), 1);
    assert_eq!(center[0].kind, crate::related_forms::DocumentKind::Form1099MiscCropInsurance);
    let key = center[0].id;
    assert_ne!(key, center[0].form_id);
    claim_related_component(&mut draft, farm, key);
    assert_eq!(crate::related_forms::effective_claimants(&draft).get(&key), Some(&farm));
    let other_center = crate::related_forms::catalog_for(&draft, other);
    let claimed = crate::related_forms::claimed_elsewhere(&draft, other);
    assert!(
        crate::RelatedForms::page::<crate::Farm>(&[], &other_center, &claimed)
            .candidates
            .is_empty()
    );
    assert_eq!(
        crate::RelatedForms::validate::<crate::Farm>(&[key], &other_center, &claimed),
        Err(0)
    );
    let original_claims = crate::related_forms::effective_claimants(&draft);
    draft.income.retain(|entry| entry.id != farm);
    assert_eq!(crate::farm_insurance_paper::income_delta(&draft, other, &original_claims), 0);
}

#[test]
fn related_crop_insurance_keeps_bookkeeping_deferral_and_withholding_owners() {
    use crate::farm_insurance_paper::{CropPaperClass, CropPaperFacts};
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(raised_farm(0, 0)));
    let activity = draft.income.last().unwrap().id;
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    install_crop_records(data, crop_records(vec![], 200, true, 0));
    add(
        &mut draft,
        income(misc(FederalForm1099Misc {
            crop_insurance: 100_000,
            withholding: 10_000,
            ..misc_form()
        })),
    );
    let key = crate::related_forms::catalog_for(&draft, activity)
        .into_iter()
        .find(|p| p.kind == crate::related_forms::DocumentKind::Form1099MiscCropInsurance)
        .unwrap()
        .id;
    claim_related_component(&mut draft, activity, key);
    crate::farm_insurance_paper::farm_mut(
        draft.income.iter_mut().find(|e| e.id == activity).unwrap(),
    )
    .unwrap()
    .insurance_papers
    .papers = vec![CropPaperFacts {
        source: key,
        classification: CropPaperClass::Mixed {
            portions: vec![CropLoss { crop: "Grain".into(), amount: 70_000, year: 2026 }],
        },
    }];
    let result = supported(&draft);
    assert_eq!(
        value(&result, &alloc::format!("farm_{activity}_crop_insurance_before_deferral")),
        1_000
    );
    assert_eq!(value(&result, &alloc::format!("farm_{activity}_crop_insurance_taxable")), 500);
    assert_eq!(return_line(&result, "25b"), 10_000);
    let before = draft.clone();
    let FederalIncome::Misc1099 { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.form.crop_insurance = Some(100_001);
    crate::farm_insurance_paper::follow_sources(&before, &mut draft);
    let claims = crate::related_forms::effective_claimants(&draft);
    let resolved = crate::farm_insurance_paper::resolved(&draft, activity, &claims).unwrap();
    assert_eq!(resolved.current_total(), 100_001);
    assert_eq!(resolved.income(true), 120_001);
    assert!(crate::farm_insurance_paper::missing(&draft, activity, &claims));
    assert_eq!(return_line(&supported(&draft), "25b"), 10_000);
}

#[test]
fn blank_printed_contact_and_account_text_survives_document_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let payment = crate::Form1099KPaper { calendar_year: 2026, ..Default::default() };
    let mut miscellaneous = crate::Form1099MiscPaper { calendar_year: 2026, ..Default::default() };
    miscellaneous.payer_telephone.clear();
    miscellaneous.account_number.clear();
    let request = save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099K(payment),
                },
            ),
            document(
                0,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Form1099Misc(miscellaneous),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert_eq!(papers.iter().filter(|row| matches!(&row.document,
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(form) }
        if form.filer_telephone.is_empty()
    )).count(), 1);
    assert_eq!(
        papers
            .iter()
            .filter(|row| matches!(&row.document,
                IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(form) }
                if form.payer_telephone.is_empty() && form.account_number.is_empty()
            ))
            .count(),
        1
    );
}

#[test]
fn printed_nec_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = crate::Form1099NecPaper { calendar_year: 2026, ..Default::default() };
    form.recipient = fixture_document_names(&session.draft, -1).unwrap().recipient;
    form.compensation = Some(0);
    form.withholding = None;
    form.cash_tips = Some(0);
    let expected = IncomeSource::SelfEmployment {
        value: crate::SelfEmploymentDocument::Form1099Nec(form.clone()),
    };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

#[test]
fn printed_misc_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = crate::Form1099MiscPaper { calendar_year: 2026, ..Default::default() };
    form.recipient = fixture_document_names(&session.draft, -1).unwrap().recipient;
    form.other = Some(0);
    form.withholding = None;
    form.payer_telephone.clear();
    form.account_number.clear();
    let expected =
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(form.clone()) };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

#[test]
fn printed_k_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = crate::Form1099KPaper { calendar_year: 2026, ..Default::default() };
    form.recipient = fixture_document_names(&session.draft, -1).unwrap().recipient;
    form.gross = Some(0);
    form.withholding = None;
    form.payment_transactions = Some(0);
    form.january = None;
    form.february = Some(0);
    let expected = IncomeSource::SelfEmployment {
        value: crate::SelfEmploymentDocument::Form1099K(form.clone()),
    };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

#[test]
fn printed_w2_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = wage_document(crate::tests::support::builders::native_income::native_w2(0));
    form.employee = fixture_document_names(&session.draft, -1).unwrap().recipient;
    form.wages = Some(0);
    form.withholding = None;
    form.social_wages = None;
    form.social_tax = Some(0);
    form.allocated_tips = None;
    form.care_benefits = Some(0);
    form.nonqualified_plans = None;
    form.box12 = vec![
        crate::PrintedBox12Entry { code: W2Code::D, amount: None },
        crate::PrintedBox12Entry { code: W2Code::DD, amount: Some(0) },
    ];
    let expected = IncomeSource::Wage { value: crate::WageDocument::W2(form.clone()) };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

#[test]
fn printed_w2g_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = w2g_document(crate::W2g {
        payer: Default::default(),
        winner: Default::default(),
        winnings: 100_000,
        withholding: 0,
        special: false,
        state_details: Default::default(),
    });
    form.winner = fixture_document_names(&session.draft, -1).unwrap().recipient;
    form.withholding = None;
    form.identical_wagers = Some(0);
    let expected = IncomeSource::Gambling(form.clone());
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

// ---- Routes: relief payments and employee pay answered on the form ---------------
// (2026-10-31, Smith round 3 G-48 and G-49) The form stays as printed; the answer sends its
// amount to the calculation that already owns the relief exclusions and Form 8919.

fn relief_misc(withholding: i64, route: IncomeUse) -> FederalIncome {
    misc(FederalForm1099Misc { other: 800_000, withholding, other_use: Some(route), ..misc_form() })
}
fn disaster_facts(qualified: bool) -> crate::DisasterReliefFacts {
    crate::DisasterReliefFacts {
        received: 20260612,
        qualification: DisasterQualification::FederalDeclaration { reference: "DR-4800".into() },
        purpose: DisasterReliefPurpose::GovernmentWelfare,
        qualified,
    }
}
fn wildfire_facts(qualified: bool) -> crate::WildfireReliefFacts {
    crate::WildfireReliefFacts {
        paid: 20260310,
        declared: 20250108,
        reference: "FEMA-FM-5000".into(),
        qualified,
    }
}

#[test]
fn a_box_3_relief_route_is_figured_exactly_as_the_relief_document() {
    for qualified in [true, false] {
        let mut routed = complete(FederalStatus::Single);
        add(
            &mut routed,
            income(relief_misc(
                50_000,
                IncomeUse::DisasterRelief { relief: disaster_facts(qualified) },
            )),
        );
        let mut own = complete(FederalStatus::Single);
        let mut data = disaster_facts(qualified).relief(800_000);
        data.withholding = 50_000;
        add(&mut own, income(FederalIncome::DisasterRelief { data }));
        let (a, b) = (supported(&routed), supported(&own));
        for line in ["11a / 11b", "16", "25b"] {
            assert_eq!(return_line(&a, line), return_line(&b, line), "disaster {qualified} {line}");
        }
        if !qualified {
            assert_eq!(value(&a, "schedule_1_other_income"), value(&b, "schedule_1_other_income"));
        }
        assert_eq!(return_line(&a, "11a / 11b") == 6_000_000, qualified);

        let mut routed = complete(FederalStatus::Single);
        add(
            &mut routed,
            income(relief_misc(
                50_000,
                IncomeUse::WildfireRelief { relief: wildfire_facts(qualified) },
            )),
        );
        let mut own = complete(FederalStatus::Single);
        let mut data = wildfire_facts(qualified).relief(800_000);
        data.withholding = 50_000;
        add(&mut own, income(FederalIncome::WildfireRelief { data }));
        let (a, b) = (supported(&routed), supported(&own));
        for line in ["11a / 11b", "16", "25b"] {
            assert_eq!(return_line(&a, line), return_line(&b, line), "wildfire {qualified} {line}");
        }
    }
    // The printed box 3 and the form's own preview follow the exclusion.
    let FederalIncome::Misc1099 { data } =
        relief_misc(0, IncomeUse::DisasterRelief { relief: disaster_facts(true) })
    else {
        unreachable!()
    };
    assert_eq!(FederalForm1099Misc::from(&data).income(), 0);
    // A date outside 2026 is refused by the relief calculation's own rule.
    let mut late = disaster_facts(true);
    late.received = 20250612;
    let FederalIncome::Misc1099 { data } =
        relief_misc(0, IncomeUse::DisasterRelief { relief: late })
    else {
        unreachable!()
    };
    assert_eq!(
        crate::information_returns::misc_issue(&Form1099Misc::from(&data)).map(|i| i.key),
        Some("RELIEF_RECEIVED_OUTSIDE_2026")
    );
}

#[test]
fn a_nec_employee_pay_route_is_figured_exactly_as_form_8919() {
    let payroll =
        crate::EmployeePayrollSource::Form8919 { reason: crate::EmployeeWageReason::SameFirmForms };
    let mut routed = complete(FederalStatus::Single);
    add(
        &mut routed,
        income(nec_form(1_200, 300, Some(IncomeUse::EmployeePay { payroll: payroll.clone() }))),
    );
    let mut own = complete(FederalStatus::Single);
    add(
        &mut own,
        income(FederalIncome::EmployeeWages {
            data: crate::EmployeeWages {
                source_issuer: Default::default(),
                source_recipient: Default::default(),
                wages: 120_000,
                withholding: 30_000,
                form1099: true,
                payroll,
            },
        }),
    );
    let (a, b) = (supported(&routed), supported(&own));
    for line in ["1g", "11a / 11b", "16", "25b"] {
        assert_eq!(return_line(&a, line), return_line(&b, line), "{line}");
    }
    assert_eq!(return_line(&a, "1g"), 120_000);
    assert_eq!(value(&a, "employee_wages_0_tax"), value(&b, "employee_wages_0_tax"));
    assert!(value(&a, "employee_wages_0_tax") > 0);
    // Excess golden parachute payments carry a tax Form 8919 does not figure.
    let mut form = crate::FederalForm1099Nec::from(&nec_source(FederalForm1099Nec {
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        compensation: 120_000,
        withholding: 0,
        golden_parachute: 1,
        belongs: Some(IncomeUse::EmployeePay {
            payroll: crate::EmployeePayrollSource::Unavailable,
        }),
    }));
    assert!(!form.valid());
    form.golden_parachute = 0;
    assert!(form.valid());
}

// R10-03: where box 3 belongs is a required strict question with no Default.
// Relating the amount from an activity is an explicit answer, not a null.
#[test]
fn box_3_use_is_a_required_strict_question_that_saves_only_when_answered() {
    use crate::tests::support::client::own_questions::{
        answer_questions, save_questions, wire_page,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let paper = misc_document(misc_source(FederalForm1099Misc { other: 800_000, ..misc_form() }));
    let desired = vec![document(
        0,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(paper) },
    )];
    let printed = save_documents(&mut session, desired);
    accepted(&send(&mut session, printed));
    let page = crate::tests::support::client::income_pages::open_topic(
        &mut session,
        TaxSection::OtherIncome,
    );
    let begin = match wire_page(&mut session, Reply::Edit(page)) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("the unanswered box 3 question opens first, got {other:?}"),
    };
    let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
    let asked = questions.iter().filter(|q| q.key.ends_with("/use")).count();
    assert_eq!(asked, 1, "box 3 is asked once, unanswered");
    let ask = questions.iter().find(|q| q.key.ends_with("/use")).unwrap();
    assert!(crate::IncomeUse::FromActivity.offered(&ask.allowed_values));
    let before = session.draft.clone();
    // Nothing answered: the page does not save.
    let reply = save_questions(&mut session, &begin, Vec::new());
    assert!(!matches!(reply, Reply::Edit(P::SaveIncomes { .. })), "{reply:?}");
    assert_eq!(session.draft, before);
    // Relating it from an activity is a real answer; it saves and reopens.
    let reply = answer_questions(
        &mut session,
        &begin,
        vec![crate::DocumentStrictAnswer::MiscellaneousUse { value: IncomeUse::FromActivity }],
    );
    assert!(matches!(reply, Reply::Edit(P::SaveIncomes { .. })), "{reply:?}");
    let mut session = restart(session.draft.clone());
    let reopened = crate::tests::support::client::income_pages::open_topic(
        &mut session,
        TaxSection::OtherIncome,
    );
    let reply = wire_page(&mut session, Reply::Edit(reopened));
    let Reply::Edit(P::SaveIncomes { questions, .. }) = reply else {
        panic!("answered, the page opens by EditRequest: {reply:?}")
    };
    assert!(questions.iter().any(|q| matches!(
        &q.answer,
        crate::DocumentStrictAnswer::MiscellaneousUse { value: IncomeUse::FromActivity }
    )));
}

#[test]
fn the_answer_offers_each_form_only_its_routes_and_keeps_the_paper() {
    use crate::DocumentAnswer as A;
    use alloc::string::ToString;
    use libertas::SchemaPath;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let misc_paper =
        misc_document(misc_source(FederalForm1099Misc { other: 800_000, ..misc_form() }));
    let nec_paper = nec_document(Form1099Nec {
        payer: Default::default(),
        recipient: Default::default(),
        compensation: 120_000,
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        golden_parachute: 0,
        withholding: 0,
        belongs: None,
        state_details: Default::default(),
    });
    let desired = vec![
        document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(misc_paper),
            },
        ),
        document(
            0,
            IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099Nec(nec_paper),
            },
        ),
    ];
    let printed = save_documents(&mut session, desired.clone());
    accepted(&send(&mut session, printed));
    let route_of = |session: &mut Interview,
                    nec: bool,
                    route: IncomeUse|
     -> (P, alloc::string::String) {
        let mut request =
            crate::tests::support::client::income_pages::information_return_answers_for_topic(
                session,
                &desired,
                if nec { TaxSection::SelfEmployment } else { TaxSection::OtherIncome },
            )
            .unwrap();
        let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
        let row = followups
            .iter_mut()
            .find(|row| {
                if nec {
                    matches!(row.answer, A::NonemployeeUse { .. })
                } else {
                    matches!(row.answer, A::MiscellaneousUse { .. })
                }
            })
            .unwrap();
        assert_eq!(
            row.allowed_values,
            if nec { IncomeUse::nonemployee_routes() } else { IncomeUse::miscellaneous_routes() }
        );
        row.answer = if nec {
            A::NonemployeeUse { value: Some(route) }
        } else {
            A::MiscellaneousUse { value: Some(route) }
        };
        let path = SchemaPath::root()
            .field("followups")
            .key(row.key.as_str())
            .field("answer")
            .field("value")
            .to_string();
        (request, path)
    };
    let employee = IncomeUse::EmployeePay {
        payroll: crate::EmployeePayrollSource::Form8919 {
            reason: crate::EmployeeWageReason::SameFirmForms,
        },
    };
    let relief = IncomeUse::DisasterRelief { relief: disaster_facts(true) };
    // Each form refuses the other form's route, and nothing is stored.
    let (request, path) = route_of(&mut session, false, employee.clone());
    reject_atomic("misc employee", &mut session, request, "DOCUMENT_QUESTIONS_CHANGED", &path);
    let (request, path) = route_of(&mut session, true, relief.clone());
    reject_atomic("nec relief", &mut session, request, "DOCUMENT_QUESTIONS_CHANGED", &path);
    // Its own routes save, and the stored paper is unchanged.
    let (request, _) = route_of(&mut session, false, relief.clone());
    accepted(&send(&mut session, request));
    let (request, _) = route_of(&mut session, true, employee.clone());
    accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    assert_eq!(return_line(&r, "1g"), 120_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_120_000);
    assert!(session.draft.income.iter().any(|e| matches!(&e.income,
        FederalIncome::Misc1099 { data } if data.other_use == Some(relief.clone())
            && data.form.other == Some(800_000))));
    assert!(session.draft.income.iter().any(|e| matches!(&e.income,
        FederalIncome::Nec1099 { data } if data.belongs == Some(employee.clone())
            && data.form.compensation == Some(120_000))));
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}

#[test]
fn printed_1099g_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = crate::Form1099GPaper {
        unemployment_compensation: 240_000,
        rtaa_payments: None,
        taxable_grants: Some(0),
        agriculture_payments: None,
        market_gain: Some(0),
        ..Default::default()
    };
    form.recipient.name = "Synthetic Filer".into();
    let expected = IncomeSource::GovernmentPayment {
        refund_state: None,
        refund_locality: Default::default(),
        value: form.clone(),
    };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}

#[test]
fn printed_1099b_withholding_blank_and_zero_survive_save_and_cold_reopen() {
    for withholding in [None, Some(0)] {
        let mut session = restart(complete(FederalStatus::Single));
        let mut form = broker_b_paper(&session.draft);
        form.box_4_withholding = withholding;
        let expected = IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Form1099B(form.clone()),
        };
        let request = save_documents(&mut session, vec![document(0, expected.clone())]);
        accepted(&send(&mut session, request));
        session = restart(session.draft);
        let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
        assert!(papers.iter().any(|row| row.document == expected), "{withholding:?}");
    }
}

#[test]
fn printed_1099a_and_1099c_year_corrected_and_box_3_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let acquisition = acquisition_document(crate::PropertyAcquisitionSource {
        form: crate::Form1099A {
            calendar_year: 2026,
            corrected: true,
            box3: "printed".into(),
            lender: crate::PrintedParty::from_name("Synthetic lender".into()),
            borrower: crate::PrintedParty::from_name("Synthetic Filer".into()),
            date: 20260801,
            balance: 800_000,
            value: 700_000,
            ..Default::default()
        },
        property: None,
    });
    let cancellation = debt_document(crate::PersonalDebtSource {
        form: crate::Form1099C {
            calendar_year: 2026,
            corrected: true,
            creditor: crate::PrintedParty::from_name("Synthetic creditor".into()),
            debtor: crate::PrintedParty::from_name("Synthetic Filer".into()),
            date: 20260801,
            amount: 100_000,
            ..Default::default()
        },
        treatment: None,
        property: None,
    });
    let expected = [
        IncomeSource::CanceledDebt {
            value: crate::CanceledDebtDocument::Acquisition1099A(acquisition),
        },
        IncomeSource::CanceledDebt {
            value: crate::CanceledDebtDocument::Cancel1099C(cancellation),
        },
    ];
    let request =
        save_documents(&mut session, expected.iter().cloned().map(|d| document(0, d)).collect());
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    for source in &expected {
        assert!(papers.iter().any(|row| &row.document == source), "{source:?}");
    }
}

#[test]
fn printed_1095a_recipient_keeps_boxes_13_to_15_apart_and_reads_as_one_party() {
    let recipient = crate::Form1095ARecipient {
        name: "Jane Smith".into(),
        ssn: "987-65-4321".into(),
        street_address: "100 Example Orchard Lane".into(),
        city: "Example Grove".into(),
        state: "VA".into(),
        country_zip: "00000".into(),
    };
    let party = crate::PrintedParty::from(&recipient);
    assert_eq!(party.name, "Jane Smith");
    assert_eq!(party.federal_id, "987-65-4321");
    assert_eq!(party.address, "100 Example Orchard Lane\nExample Grove VA 00000");
    // A blank box stays blank: no stray separators.
    let blank = crate::Form1095ARecipient { name: "Jane Smith".into(), ..Default::default() };
    assert_eq!(crate::PrintedParty::from(&blank).address, "");
    let city_only = crate::Form1095ARecipient { city: "Example Grove".into(), ..blank };
    assert_eq!(crate::PrintedParty::from(&city_only).address, "Example Grove");
}

#[test]
fn printed_1099div_blank_and_zero_survive_save_and_cold_reopen() {
    let mut session = restart(complete(FederalStatus::Single));
    let mut form = crate::Form1099DivPaper {
        calendar_year: Some(2026),
        ordinary: Some(10_000),
        qualified: Some(0),
        capital_distributions: None,
        nondividend_distributions: None,
        withholding: Some(0),
        section199a: None,
        foreign_tax: None,
        cash_liquidation: Some(0),
        noncash_liquidation: None,
        ..Default::default()
    };
    form.recipient.name = "Synthetic Filer".into();
    let expected = IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(form.clone()),
    };
    let request = save_documents(&mut session, vec![document(0, expected.clone())]);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let papers = crate::tests::support::client::income_pages::all_documents(&mut session);
    assert!(papers.iter().any(|row| row.document == expected));
}
