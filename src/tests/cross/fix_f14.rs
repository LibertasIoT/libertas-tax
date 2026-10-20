// Fix unit F14 (2026-10-10): the merge review's defects D1-D5 (community-property withholding of a
// routed Form 1099, Form 8958 rows, a publicly traded partnership's prior loss on Form 8582,
// Schedule 1 lines 3 and 6, the Form 2210 waiver labels) and its suspected rows S1 and S2.
use super::*;

fn nec_form(compensation: i64, withholding: i64, belongs: IncomeUse) -> FederalIncome {
    FederalIncome::Nec1099 {
        data: FederalForm1099Nec {
            compensation: compensation * 100,
            withholding: withholding * 100,
            golden_parachute: 0,
            belongs,
            business: None,
        },
    }
}
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
fn community(d: &mut FederalDraft, rule: CommunityRule) {
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Arizona, rule };
    for e in &mut d.income {
        e.community = CommunityShare::Community;
    }
}
fn base(rule: CommunityRule) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.next_id = 2;
    community(&mut d, rule);
    d
}
// The filer's wages, Lee Design (5,000 of receipts, 3,000 of supplies) and a 1099-NEC of 20,000
// with 4,800 withheld that belongs to the business.
fn nec_return(rule: CommunityRule) -> FederalDraft {
    let mut d = base(rule);
    add(&mut d, named_business("Lee Design", 5_000, 3_000));
    add(&mut d, income(nec_form(20_000, 4_800, IncomeUse::Business)));
    community(&mut d, rule);
    d
}

#[test]
fn d1_the_withholding_on_a_routed_form_1099_follows_the_income_under_section_879() {
    // Living apart all year: the business (22,000 with the routed NEC) is the earner's whole, so is
    // the tax withheld from the NEC (Reg. 1.31-1(a)); the wages' withholding is still halved.
    for rule in [CommunityRule::LivingApartAllYear, CommunityRule::NonresidentSpouse] {
        let r = supported(&nec_return(rule));
        assert_eq!(value(&r, "business_2_net_profit"), 22_000, "{rule:?}");
        assert_eq!(value(&r, "agi"), 30_000 + 22_000 - 1_554, "{rule:?}");
        assert_eq!(return_line(&r, "25a"), 300_000, "{rule:?}");
        assert_eq!(return_line(&r, "25b"), 480_000, "{rule:?}");
    }
}

#[test]
fn d1_a_halves_return_still_divides_the_withholding_on_the_nec() {
    let r = supported(&nec_return(CommunityRule::Halves));
    assert_eq!(return_line(&r, "25b"), 240_000);
}

#[test]
fn d1_the_withholding_on_a_spouses_routed_nec_is_not_this_returns_under_section_879() {
    let mut d = base(CommunityRule::LivingApartAllYear);
    let mut business = named_business("Lee Design", 5_000, 3_000);
    business.owner = Owner::Spouse;
    add(&mut d, business);
    let mut nec = income(nec_form(20_000, 4_800, IncomeUse::Business));
    nec.owner = Owner::Spouse;
    add(&mut d, nec);
    community(&mut d, CommunityRule::LivingApartAllYear);
    let r = supported(&d);
    assert_eq!(return_line(&r, "25b"), 0);
    assert_eq!(value(&r, "agi"), 30_000);
}

#[test]
fn d1_the_withholding_on_a_grant_routed_to_a_business_follows_the_earner() {
    let mut d = base(CommunityRule::LivingApartAllYear);
    add(&mut d, named_business("Lee Design", 5_000, 3_000));
    let mut g = government_unemployment(0, &[], 1_000);
    if let FederalIncome::Unemployment { data } = &mut g.income {
        data.taxable_grants = 1_000_000;
        data.grant_use = Some(FederalGrantUse::Business { business: None });
    }
    add(&mut d, g);
    community(&mut d, CommunityRule::LivingApartAllYear);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 12_000);
    assert_eq!(return_line(&r, "25b"), 100_000);
}

#[test]
fn d2_form_8958_lists_a_routed_nec_once() {
    let r = supported(&nec_return(CommunityRule::Halves));
    let rows = |suffix: &str| -> i64 {
        r.lines
            .iter()
            .filter(|l| {
                l.id.starts_with("form8958_")
                    && l.id.ends_with(suffix)
                    && !l.id.starts_with("form8958_withholding_")
            })
            .map(|l| l.amount)
            .sum()
    };
    assert!(!r.lines.iter().any(|l| l.id == "form8958_3_total"));
    assert_eq!(rows("_total"), 5_200_000, "wages 30,000 + Schedule C 22,000");
    assert_eq!(rows("_this_return"), 2_600_000);
    // The withholding rows stay: Form 8958 splits the tax withheld too.
    assert!(r.lines.iter().any(|l| l.id == "form8958_withholding_3_total" && l.amount == 480_000));
}

fn partner_id(d: &FederalDraft) -> i64 {
    d.income.iter().find(|e| matches!(e.income, FederalIncome::Partnership { .. })).unwrap().id
}
fn profit_rental(profit: i64) -> FederalIncomeEntry {
    let mut v = rental(profit + 10_000, 0);
    v.participation = crate::RentalParticipation::NotActive;
    v.history = RentalHistory { regular: 0, amt: 0, active_in_loss_year: true };
    income(FederalIncome::Rental { data: v })
}
fn ptp(ordinary: i64, prior_passive: i64, opening_basis: i64) -> PartnerK1 {
    let mut data = ordinary_partner_source(ordinary, false);
    data.regular.passive_loss = prior_passive * 100;
    data.regular.basis.opening = opening_basis * 100;
    data.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 500_000_000, profit: 0, loss: 0 };
    data.publicly_traded = true;
    data.alternative = Some(data.regular.clone());
    data
}

#[test]
fn d3_a_ptps_prior_year_loss_stays_off_form_8582() {
    // Section 469(k); i8582 "Publicly traded partnerships": a PTP's prior-year unallowed loss
    // offsets only that PTP's income, so Part V holds the rental alone.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: ptp(-1_000, 4_000, 50_000) }));
    add(&mut d, profit_rental(5_000));
    let r = supported(&d);
    let id = partner_id(&d);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_passive_suspended")), 5_000);
    assert_eq!(value(&r, "form8582_line_2a"), 5_000);
    assert_eq!(value(&r, "form8582_line_2c"), 0);
    assert_eq!(value(&r, "form8582_line_2d"), 5_000);
    assert_eq!(value(&r, "form8582_line_3"), 5_000);
}

#[test]
fn d3_a_ptps_income_with_a_prior_loss_stays_off_form_8582() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: ptp(6_000, 4_000, 50_000) }));
    add(&mut d, profit_rental(5_000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 67_000);
    assert_eq!(value(&r, "form8582_line_2a"), 5_000);
    assert_eq!(value(&r, "form8582_line_2c"), 0);
}

#[test]
fn d3_lines_10_and_11_do_not_count_a_ptps_prior_loss_as_income() {
    // A rental loss of 3,000 and a PTP with a 4,000 prior loss: line 3 is the rental's -3,000, and
    // the income on lines 1a and 2a (line 10) is 0, not the PTP's loss added back.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: ptp(-1_000, 4_000, 50_000) }));
    add(&mut d, profit_rental(-3_000));
    let r = supported(&d);
    assert_eq!(value(&r, "form8582_line_3"), -3_000);
    assert_eq!(value(&r, "form8582_line_10"), 0);
}

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
fn d4_schedule_1_line_3_is_the_community_share_of_the_profit() {
    let mut d = community_base();
    add(&mut d, income(cash_business(20_000)));
    d.income[1].community = CommunityShare::Community;
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 2_826);
    assert_eq!(value(&r, "agi"), 15_000 + 10_000 - 1_413);
    assert_eq!(schedule_1_line(&r, "3"), 10_000);
    assert_eq!(schedule_1_line(&r, "6"), 0);
    assert_eq!(schedule_1_line(&r, "10"), 10_000);
}

#[test]
fn d4_a_spouses_business_is_this_returns_share_on_line_3() {
    let mut d = community_base();
    let mut business = income(cash_business(20_000));
    business.owner = Owner::Spouse;
    business.community = CommunityShare::Community;
    add(&mut d, business);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 25_000);
    assert_eq!(schedule_1_line(&r, "3"), 10_000);
    assert_eq!(schedule_1_line(&r, "6"), 0);
}

#[test]
fn d4_a_business_the_filer_alone_earns_under_section_879_is_whole_on_line_3() {
    let mut d = community_base();
    add(&mut d, income(cash_business(20_000)));
    community(&mut d, CommunityRule::LivingApartAllYear);
    let r = supported(&d);
    assert_eq!(schedule_1_line(&r, "3"), 20_000);
    assert_eq!(schedule_1_line(&r, "6"), 0);
}

fn waiver_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_110_000, 400_000)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 800_000, agi: Some(10_000_000), spouse_tax: 0 });
    pay.estimated_payments = vec![DatedPayment {
        date: 20260610,
        amount: 150_000,
        payer: PaymentPayer::Taxpayer,
        refund_property: None,
    }];
    pay.refresh_estimated_projection();
    pay.waiver = Some(PenaltyWaiver::Full {
        reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
        explanation: "A house fire destroyed our records in October 2026.".into(),
    });
    d
}

#[test]
fn d5_the_penalty_before_the_waiver_has_a_label_in_the_estimate_view() {
    // Debug builds assert that every calculation id has a label resource; building the estimate
    // view labels each row.
    let r = supported(&waiver_return());
    assert!(r.lines.iter().any(|l| l.id == "form_2210_penalty_before_waiver"));
    let view = FederalEstimate::from(r);
    let row = view
        .breakdown
        .iter()
        .map(|a| status_text(&a.label))
        .find(|label| label.contains("penalty before the waiver"))
        .expect("a labelled row");
    assert!(row.contains("2210"), "{row}");
}

#[test]
fn d5_the_farmers_penalty_before_the_waiver_has_its_label() {
    for id in ["form_2210_penalty_before_waiver", "form_2210_f_penalty_before_waiver"] {
        let keys = crate::federal_model::IdKeys::new(id);
        assert!(keys.registered(), "{id}");
        assert!(crate::line_label(id).english().contains("penalty before the waiver"));
    }
}

#[test]
fn s1_a_blank_basis_box_clears_the_nullable_basis_and_defaults_to_null() {
    // The four import markers declare `blank = $null`, the eight fields `$null` as their default.
    for file in [
        include_str!("../../income/employee_stock.rs"),
        include_str!("../../retirement/nua_sale.rs"),
        include_str!("../../income/iso_sale.rs"),
        include_str!("../../income/espp_sale.rs"),
    ] {
        let lines: Vec<&str> = file.lines().collect();
        let mut seen = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "pub reported_basis: Option<i64>," {
                seen += 1;
                let above = lines[i.saturating_sub(5)..i].join("\n");
                assert!(above.contains("#[libertas_default($null)]"), "{above}");
                if above.contains("libertas_llm_doc_field") {
                    assert!(above.contains("#[libertas_llm_doc_field(blank = $null)]"), "{above}");
                }
            }
        }
        assert_eq!(seen, 2);
    }
}

#[test]
fn s2_the_special_allowance_amounts_are_one_set_of_constants() {
    use crate::year_constants::*;
    assert_eq!(PASSIVE_ALLOWANCE_DOLLARS, 25_000);
    assert_eq!(PASSIVE_ALLOWANCE_SEPARATE_DOLLARS, 12_500);
    assert_eq!(PASSIVE_ALLOWANCE_PHASEOUT_END_DOLLARS, 150_000);
    assert_eq!(PASSIVE_ALLOWANCE_PHASEOUT_END_SEPARATE_DOLLARS, 75_000);
    let rules = include_str!("../../federal/extended_rules.rs");
    for literal in ["75_000", "150_000", "12_500", "25_000"] {
        assert!(!rules.contains(literal), "{literal} repeats a constant");
    }
    assert!(!include_str!("../../federal/federal_rules.rs").contains("12_500"));
}
