// Unit16 Form3800 map controls. The standard/QBI F6/F7 composition is
// approved in f08f67e; property-tax alternatives remain rejected as fixtures.
use super::*;

#[path = "unit16_mortgage_gift_caps.rs"]
mod unit16_mortgage_gift_caps;

#[path = "unit16_property_recapture_caps.rs"]
mod unit16_property_recapture_caps;

#[path = "unit16_allowance_client.rs"]
mod unit16_allowance_client;

#[path = "unit16_applicable_carryback.rs"]
mod unit16_applicable_carryback;

#[path = "unit16_research_limited_interest.rs"]
mod unit16_research_limited_interest;

#[path = "unit16_passive_tax_method_client.rs"]
mod unit16_passive_tax_method_client;

fn history(d: &mut FederalDraft) -> &mut BusinessCreditHistory {
    d.credits.as_mut().unwrap().business_credits.get_or_insert(BusinessCreditHistory {
        credits: vec![],
        carryforwards: vec![],
        carrybacks: vec![],
        spouse_without_credits: false,
    })
}
fn own(d: &mut FederalDraft, credit: BusinessCredit, amount: i64) {
    add(d, income(cash_business(0)));
    let id = d.income.last().unwrap().id;
    history(d).credits.push(EnteredBusinessCredit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        credit,
        amount: amount * 100,
        source: CreditSource::Business { business: Some(id), reduced: true },
        election: CreditElection::None,
    });
}
fn carry(credit: BusinessCredit, year: u32, amount: i64) -> BusinessCreditCarryforward {
    BusinessCreditCarryforward {
        credit,
        origin_year: year,
        amount: amount * 100,
        passive: None,
        filer: CarryFiler::SameFilers,
        research_source: None,
    }
}
fn column(r: &FederalResult, part: u8, line: &str, col: char) -> i64 {
    value(r, &alloc::format!("form_3800_part_{part}_line_{line}({col})"))
}
#[test]
fn unit16_f1_own_and_k1_disabled_access_share_one_cap_before_fifo() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21_610_000, 0);
    own(&mut d, BusinessCredit::DisabledAccess, 4000);
    let mut k1 = ordinary_partner_source(0, true);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::DisabledAccess,
        line: None,
        amount: 150_000,
    });
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    history(&mut d).carryforwards.push(carry(BusinessCredit::DisabledAccess, 2024, 4000));
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 200000),
        ("regular_income_tax", 40598),
        ("form_3800_line_13", 3900),
        ("form_3800_line_14", 32760),
        ("form_3800_line_16", 7838),
        ("form_3800_line_38", 7838),
        ("business_credit_disabled_access_2024_used", 4000),
        ("business_credit_disabled_access_current_year_used", 3838),
        ("business_credit_disabled_access_current_year_to_next_year", 1162),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    assert_eq!(column(&r, 3, "1e", 'e'), 5000);
    assert_eq!(column(&r, 3, "1e", 'i'), 3838);
    assert_eq!(column(&r, 4, "1e", 'g'), 4000);
    assert_eq!(return_line(&r, "22"), 3_276_000);
}
#[test]
fn unit16_f2_scorporation_box13_ah_uses_form3800() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 21_220_000, 0);
    let mut k1 = complete_ordinary_scorp(0, true);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::EmployerChildcare,
        line: None,
        amount: 1_200_000,
    });
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 180000),
        ("regular_income_tax", 29024),
        ("form_3800_line_13", 1006),
        ("form_3800_line_14", 18720),
        ("general_business_credit", 10304),
        ("business_credit_employer_childcare_current_year_to_next_year", 1696),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    assert_eq!(column(&r, 3, "1k", 'e'), 12000);
}
#[test]
fn unit16_f3_passive_differential_wage_uses_tax_difference() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9_610_000, 0);
    let mut k1 = ordinary_partner_source(10000, false);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::DifferentialWage,
        line: None,
        amount: 300_000,
    });
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 90000),
        ("regular_income_tax", 14518),
        ("form8582cr_line_6", 2200),
        ("form8582cr_line_37", 2200),
        ("general_business_credit", 2200),
        ("business_credit_differential_wage_current_year_passive_to_next_year", 800),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    assert_eq!(column(&r, 3, "1w", 'd'), 3000);
    assert_eq!(column(&r, 3, "1w", 'g'), 2200);
}
#[test]
fn unit16_f4_current_carryback_conserves_credit_and_respects_first_year() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 5_716_000, 0);
    own(&mut d, BusinessCredit::DifferentialWage, 6000);
    history(&mut d).carrybacks.push(BusinessCreditCarryback {
        credit: BusinessCredit::DifferentialWage,
        year: 2025,
        amount: 300_000,
        research_source: None,
    });
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 24960),
        ("regular_income_tax", 2501),
        ("general_business_credit", 2501),
        ("business_credit_differential_wage_current_year_carryback_2025", 3000),
        ("business_credit_differential_wage_current_year_to_next_year", 499),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    history(&mut d).carrybacks[0].amount = 350_000;
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_CREDIT_CARRYBACK_LIMIT"));
    for (credit, year, valid) in [
        (BusinessCredit::DifferentialWage, 2024, false),
        (BusinessCredit::MarginalWells, 2022, true),
        // Section39(a)(4) exception. Historical rejected2024 proof remains
        // archived; ordinary DifferentialWage2024 above still staysfalse.
        (BusinessCredit::CleanElectricity, 2024, true),
    ] {
        let h = BusinessCreditHistory {
            credits: vec![],
            carryforwards: vec![],
            spouse_without_credits: false,
            carrybacks: vec![BusinessCreditCarryback {
                credit,
                year,
                amount: 100,
                research_source: None,
            }],
        };
        assert_eq!(h.rule().is_none(), valid, "{credit:?}/{year}");
    }
}
#[test]
fn unit16_f5_elective_payment_flows_through_schedule3_and_payments() {
    for (gross, payment, carry) in [(50000, 42162, 0), (30000, 30000, 12162)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 21_610_000, 0);
        own(&mut d, BusinessCredit::AdvancedManufacturingProduction, 50000);
        history(&mut d).credits[0].election =
            CreditElection::ElectivePayment { amount: gross * 100 };
        let r = supported(&d);
        assert_eq!(value(&r, "general_business_credit"), 7838);
        assert_eq!(column(&r, 3, "1b", 'h'), gross);
        assert_eq!(column(&r, 3, "1b", 'i'), 7838);
        assert_eq!(column(&r, 3, "1b", 'j'), payment);
        assert_eq!(column(&r, 3, "6", 'j'), payment);
        assert_eq!(value(&r, "schedule_3_elective_payment"), payment);
        assert_eq!(
            value(
                &r,
                "business_credit_advanced_manufacturing_production_current_year_to_next_year"
            ),
            carry
        );
    }
}
#[test]
fn unit16_proposed_f6_itemized_variant_changes_original_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    // Provisional alternative only: this preserves regular AGI/TI but adds
    // back property tax for AMT, contrary to the map's original assumption.
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 2_000_000;
    let rent = rental(10_000, 0);
    add(&mut d, income(FederalIncome::Rental { data: rent }));
    let id = d.income.last().unwrap().id;
    history(&mut d).credits.push(EnteredBusinessCredit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        credit: BusinessCredit::DisabledAccess,
        amount: 200_000,
        source: CreditSource::Rental { rental: Some(id), reduced: true },
        election: CreditElection::None,
    });
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 100000),
        ("regular_income_tax", 16712),
        ("form_3800_line_14", 7774),
        ("form_3800_line_16", 8938),
        ("form8582cr_line_8", 2000),
        ("form8582cr_line_10", 120000),
        ("form8582cr_line_14", 15000),
        ("form8582cr_line_15", 3294),
        ("form8582cr_line_16", 2000),
        ("general_business_credit", 2000),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_16"), 0);
    assert_eq!(value(&r, "general_business_credit"), 0);
}
#[test]
fn unit16_proposed_f7_itemized_variant_changes_original_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21_000_000, 0);
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 2_000_000;
    let mut k1 = ordinary_partner_source(0, false);
    k1.housing_confirmed = Some(false);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::LowIncomeHousingOther,
        line: None,
        amount: 300_000,
    });
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    let r = supported(&d);
    for (key, expected) in [
        ("taxable_income", 190000),
        ("regular_income_tax", 38198),
        ("form_3800_line_14", 31174),
        ("form8582cr_line_2a", 3000),
        ("form8582cr_line_22", 210000),
        ("form8582cr_line_26", 20000),
        ("form8582cr_line_27", 4800),
        ("form8582cr_line_30", 3000),
        ("general_business_credit", 3000),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "form8582cr_line_30"), 0);
    assert_eq!(value(&r, "general_business_credit"), 0);
}

fn capacity() -> crate::general_business_credit::TaxCapacity {
    use crate::general_business_credit::*;
    TaxCapacity {
        status: FederalStatus::Joint,
        spouse_without_credits: false,
        regular_tax_before_credits: 9000,
        alternative_minimum_tax: 0,
        tentative_minimum_tax: 0,
        foreign_tax_credit: 0,
        other_nonrefundable_credits: 0,
        taxable_income: 90000,
        research_numerators: vec![],
        separate_ratios: Some([3000, 7000]),
        passive: PassiveCreditCapacity {
            net_passive_income_tax: 0,
            remaining_allowance: 0,
            remaining_allowance_tax: 0,
            rehabilitation: Default::default(),
            active: Default::default(),
        },
    }
}

#[test]
fn unit16_standard_qbi_inputs_preserve_original_f6_f7_rows() {
    extern crate std;
    for pre1990 in [false, true] {
        let agi = if pre1990 { 210000 } else { 120000 };
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, (agi - 19500) * 100, 0);
        add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(19500, true) }));
        if pre1990 {
            let mut data = ordinary_partner_source(0, false);
            data.housing_confirmed = Some(false);
            data.credits.push(K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                code: K1CreditCode::LowIncomeHousingOther,
                line: None,
                amount: 300_000,
            });
            add(&mut d, income(FederalIncome::Partnership { data }));
        } else {
            add(&mut d, income(FederalIncome::Rental { data: rental(10000, 0) }));
            let source = d.income.last().unwrap().id;
            history(&mut d).credits.push(EnteredBusinessCredit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                credit: BusinessCredit::DisabledAccess,
                source: CreditSource::Rental { rental: Some(source), reduced: true },
                amount: 200_000,
                election: CreditElection::None,
            });
        }
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "qbi_deduction"), 3900);
        assert_eq!(value(&r, "taxable_income"), agi - 20000);
        let rows = if pre1990 {
            vec![
                ("regular_income_tax", 38198),
                ("form_3800_line_14", 30160),
                ("form8582cr_line_2a", 3000),
                ("form8582cr_line_2c", 3000),
                ("form8582cr_line_5", 3000),
                ("form8582cr_line_6", 0),
                ("form8582cr_line_7", 3000),
                ("form8582cr_line_17", 3000),
                ("form8582cr_line_19", 3000),
                ("form8582cr_line_20", 3000),
                ("form8582cr_line_21", 250000),
                ("form8582cr_line_22", 210000),
                ("form8582cr_line_23", 40000),
                ("form8582cr_line_24", 20000),
                ("form8582cr_line_25", 0),
                ("form8582cr_line_26", 20000),
                ("form8582cr_line_27", 4800),
                ("form8582cr_line_29", 4800),
                ("form8582cr_line_30", 3000),
                ("form8582cr_line_37", 3000),
                ("form_3800_line_13", 3300),
                ("form_3800_line_27", 34898),
                ("form_3800_line_29", 34898),
                ("form_3800_line_32", 3000),
                ("form_3800_line_33", 3000),
                ("form_3800_line_36", 3000),
                ("form_3800_line_37", 3000),
                ("form_3800_line_38", 3000),
            ]
        } else {
            vec![
                ("regular_income_tax", 16712),
                ("form8582cr_line_1a", 2000),
                ("form8582cr_line_1c", 2000),
                ("form8582cr_line_5", 2000),
                ("form8582cr_line_6", 0),
                ("form8582cr_line_7", 2000),
                ("form8582cr_line_8", 2000),
                ("form8582cr_line_9", 150000),
                ("form8582cr_line_10", 120000),
                ("form8582cr_line_11", 30000),
                ("form8582cr_line_12", 15000),
                ("form8582cr_line_13", 0),
                ("form8582cr_line_14", 15000),
                ("form8582cr_line_15", 3294),
                ("form8582cr_line_16", 2000),
                ("form8582cr_line_37", 2000),
                ("form_3800_line_2", 2000),
                ("form_3800_line_3", 2000),
                ("form_3800_line_6", 2000),
                ("form_3800_line_14", 6760),
                ("form_3800_line_16", 9952),
                ("form_3800_line_17", 2000),
                ("form_3800_line_38", 2000),
            ]
        };
        for (id, expected) in &rows {
            assert_eq!(value(&r, id), *expected, "pre1990={pre1990}, {id}");
        }
        std::println!(
            "Unit16 approved standard+QBI fixture pre1990={pre1990}: AGI={agi}, QBI3900, TI{}, rows={rows:?}",
            agi - 20000
        );
        let boundary = if pre1990 { 250000 } else { 150000 };
        d.income[0] = wage(1, Owner::Taxpayer, (boundary - 19500) * 100, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), boundary);
        assert_eq!(
            value(&r, if pre1990 { "form8582cr_line_22" } else { "form8582cr_line_10" }),
            boundary
        );
        assert_eq!(value(&r, if pre1990 { "form8582cr_line_30" } else { "form8582cr_line_16" }), 0);
        assert_eq!(value(&r, "form8582cr_line_37"), 0);
        assert_eq!(value(&r, "general_business_credit"), 0);
        assert_eq!(
            value(
                &r,
                if pre1990 {
                    "business_credit_low_income_housing_current_year_passive_to_next_year"
                } else {
                    "business_credit_disabled_access_current_year_passive_to_next_year"
                }
            ),
            if pre1990 { 3000 } else { 2000 }
        );
    }
}
fn entry(
    credit: BusinessCredit,
    year: u32,
    amount: i64,
) -> crate::general_business_credit::CreditEntry {
    crate::general_business_credit::CreditEntry {
        source_owner: None,
        conditional_origin: None,
        credit,
        year,
        amount,
        passive: None,
        activity: None,
        filer: CarryFiler::SameFilers,
        transfer: 0,
        elective: 0,
    }
}
#[test]
fn unit16_f8_changed_filer_uses_existing_owner_ratio_without_guessing_it() {
    for (filer, ratio, expected) in [
        (CarryFiler::You, Some([3000, 7000]), 2700),
        (CarryFiler::SameFilers, Some([3000, 7000]), 5000),
        (CarryFiler::You, None, 0),
    ] {
        let mut input = entry(BusinessCredit::PensionStartup, 2023, 5000);
        input.filer = filer;
        let mut cap = capacity();
        cap.separate_ratios = ratio;
        let mut r = FederalResult::empty(0);
        let (credit, payment) =
            crate::general_business_credit::calculate(None, vec![input], &[], cap, &mut r);
        assert_eq!((credit, payment), (expected, 0));
        assert_eq!(column(&r, 4, "1j", 'g'), expected);
        assert_eq!(column(&r, 4, "1j", 'i'), 5000 - expected);
        assert_eq!(
            r.review_notes
                .iter()
                .any(|n| n.display.key == "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"),
            ratio.is_none()
        );
    }
}
#[test]
fn unit16_research_caps_keep_sources_vintages_and_both_lines_distinct() {
    let mut a = entry(BusinessCredit::Research, 2024, 2000);
    a.activity = Some(11);
    let mut b = entry(BusinessCredit::Research, 2024, 2000);
    b.activity = Some(12);
    let mut specified = entry(BusinessCredit::ResearchSmallBusiness, 2026, 1000);
    specified.activity = Some(11);
    let mut cap = capacity();
    cap.regular_tax_before_credits = 10000;
    cap.taxable_income = 100000;
    cap.research_numerators = vec![(11, 10000), (12, 0)];
    let mut r = FederalResult::empty(0);
    let (credit, payment) =
        crate::general_business_credit::calculate(None, vec![a, b, specified], &[], cap, &mut r);
    assert_eq!((credit, payment), (1000, 0));
    assert_eq!(value(&r, "business_credit_research_source_11_filer_0_2024_used"), 1000);
    assert_eq!(value(&r, "business_credit_research_source_12_filer_0_2024_used"), 0);
    assert_eq!(
        value(
            &r,
            "business_credit_research_small_business_source_11_filer_0_current_year_to_next_year"
        ),
        1000
    );
    assert_eq!(column(&r, 4, "1c", 'i'), 3000);
}

#[test]
fn unit16_retained_investment_interest_origin_changes_research_cap_without_rerouting() {
    extern crate std;
    use crate::investment_interest::{
        InvestmentBorrowing, InvestmentElection, InvestmentInterestUse, InvestmentLimitedAllocation,
    };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    add(&mut d, income(savings_income(10000, 0)));
    let mut entity = complete_ordinary_scorp(0, true);
    entity.interest = 2_000_000;
    add(&mut d, income(FederalIncome::Scorp { data: entity }));
    let source = d.income.last().unwrap().id;
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 1_000_000;
    // Hold the approved witness's itemized route fixed. Automatic otherwise
    // correctly compares the new research limit and can prefer standard.
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    d.deductions.as_mut().unwrap().investment_interest = Some(InvestmentBorrowing {
        loans: vec![native_investment_loan(
            "Retained investment borrowing".into(),
            0,
            1_000_000,
            1_000_000,
        )],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    let mut credit = carry(BusinessCredit::Research, 2024, 10000);
    credit.research_source = Some(source);
    history(&mut d).carryforwards.push(credit);
    let mut world_entity = d.clone();
    world_entity.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0]
        .originating_k1 = Some(source);
    let world_other = d.clone();
    assert_ne!(world_entity, world_other);
    assert_eq!(restart(world_entity.clone()).draft, world_entity);
    assert_eq!(
        crate::investment_interest::sources(&d).iter().map(|v| v.id).collect::<Vec<_>>(),
        vec![0]
    );
    let a = supported(&world_entity);
    let b = supported(&world_other);
    assert_eq!(value(&a, "general_business_credit"), 1583);
    assert_eq!(value(&b, "general_business_credit"), 3166);
    for (id, expected) in [
        ("agi", 130000),
        ("taxable_income", 110000),
        ("regular_income_tax", 18998),
        ("investment_interest_deduction", 10000),
    ] {
        assert_eq!(value(&a, id), expected, "{id}");
    }
    let interest = d.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap();
    let routed = interest.route(interest.aggregate().calculate(30000, 0, 0, 0)).unwrap();
    assert_eq!(routed.forms, [[10000, 0], [10000, 0]]);
    assert_eq!(routed.sources.get(&0), Some(&[[10000, 0], [10000, 0]]));
    let numerator = |net| net - crate::kernel::ratio::fraction(10000, net, 120000);
    let attributed = numerator(10000);
    let unrelated = numerator(20000);
    let cap = |n| crate::kernel::ratio::fraction(18998, n, 110000);
    assert_eq!((attributed, unrelated), (9167, 18333));
    assert_eq!((cap(attributed), cap(unrelated)), (1583, 3166));
    std::println!(
        "Unit16 actual retained4952 origin witness: entity={source}, AGI130000/TI110000/line11=18998; same-entity carry deduction10000 => numerator9167/cap1583; unrelated source => numerator18333/cap3166; both Form4952 allowed10000/carry0"
    );
}

#[test]
fn unit16_retained_origin_projection_preserves_form4952_allowance_and_carry() {
    use crate::investment_interest::{
        InvestmentBorrowing, InvestmentElection, InvestmentInterestUse,
        InvestmentLimitedAllocation, InvestmentLoan,
    };
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(0, true) }));
    let source = d.income.last().unwrap().id;
    let mut borrowing = InvestmentBorrowing {
        loans: vec![
            InvestmentLoan {
                originating_k1: Some(source),
                ..native_investment_loan("Prior entity interest".into(), 0, 2_000_000, 1_000_000)
            },
            native_investment_loan("Other prior interest".into(), 0, 2_000_000, 3_000_000),
        ],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    };
    assert!(borrowing.valid() && borrowing.sources_valid(&d));
    let forms = borrowing.aggregate().calculate(30000, 0, 0, 0);
    assert_eq!(forms, [[30000, 10000], [30000, 10000]]);
    let routed = borrowing.route(forms).unwrap();
    assert_eq!(routed.sources.get(&0), Some(&forms));
    let current = alloc::collections::BTreeMap::new();
    assert_eq!(
        crate::investment_interest::research_parts(Some(&borrowing), &current, 30000).get(&source),
        Some(&15000)
    );
    assert_eq!(
        crate::investment_interest::research_parts(Some(&borrowing), &current, 40000).get(&source),
        Some(&20000)
    );
    borrowing.loans[0].originating_k1 = None;
    let other = borrowing.route(borrowing.aggregate().calculate(30000, 0, 0, 0)).unwrap();
    assert_eq!(other.forms, routed.forms);
    assert_eq!(other.sources, routed.sources);
    assert!(
        crate::investment_interest::research_parts(Some(&borrowing), &current, 30000).is_empty()
    );
    borrowing.loans[0].originating_k1 = Some(999999);
    assert!(!borrowing.sources_valid(&d));
    borrowing.loans[0].originating_k1 = Some(source);
    borrowing.loans[0].source = source;
    assert!(!borrowing.sources_valid(&d)); // K1 origin is never a royalty destination.
}

#[test]
fn unit16_f9_all_nonresearch_entered_lines_share_the_income_tax_credit_base() {
    let offered = BusinessCredit::entered_choices();
    for credit in
        BusinessCredit::ALL.into_iter().filter(|c| offered.contains(&(*c as i32)) && !c.research())
    {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 11_610_000, 0);
        own(&mut d, credit, 5000);
        let r = supported(&d);
        assert_eq!(value(&r, "general_business_credit"), 5000, "{credit:?}");
        assert_eq!(column(&r, 3, credit.line(2026), 'e'), 5000, "{credit:?}");
        assert_eq!(column(&r, 3, credit.line(2026), 'i'), 5000, "{credit:?}");
        // supported() checks every recorded credit base against1040 line18.
    }
}
#[test]
fn unit16_transfer_preserves_gross_column_and_never_treats_cash_as_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 11_610_000, 0);
    own(&mut d, BusinessCredit::AdvancedManufacturingProduction, 5000);
    history(&mut d).credits[0].election = CreditElection::Sold { amount: 150_000 };
    let sold = supported(&d);
    assert_eq!(value(&sold, "agi"), 116100);
    assert_eq!(value(&sold, "general_business_credit"), 3500);
    for (col, amount) in [('d', 0), ('e', 5000), ('f', -1500), ('g', 3500), ('i', 3500)] {
        assert_eq!(column(&sold, 3, "1b", col), amount);
    }
    history(&mut d).credits[0].source = CreditSource::Bought;
    history(&mut d).credits[0].election = CreditElection::None;
    let bought = supported(&d);
    assert_eq!(value(&bought, "agi"), 116100);
    assert_eq!(value(&bought, "general_business_credit"), 0);
    assert_eq!(column(&bought, 3, "1b", 'e'), 0);
    assert_eq!(column(&bought, 3, "1b", 'f'), 5000);
    assert_eq!(
        value(
            &bought,
            "business_credit_advanced_manufacturing_production_current_year_passive_to_next_year"
        ),
        5000
    );
    history(&mut d).credits[0].election = CreditElection::Sold { amount: 100 };
    assert_eq!(history(&mut d).rule(), Some("BOUGHT_CREDIT_CANNOT_BE_SOLD"));
}
#[test]
fn unit16_section_b_uses_its_own_capacity_and_fifo() {
    let mut cap = capacity();
    cap.regular_tax_before_credits = 30000;
    cap.tentative_minimum_tax = 20000;
    let ordinary = entry(BusinessCredit::DifferentialWage, 2026, 20000);
    let old = entry(BusinessCredit::EmpowermentZone, 2024, 3000);
    let current = entry(BusinessCredit::EmpowermentZone, 2026, 9000);
    let mut r = FederalResult::empty(0);
    let (credit, payment) = crate::general_business_credit::calculate(
        None,
        vec![ordinary, old, current],
        &[],
        cap,
        &mut r,
    );
    // A:30000−20000=10000. B:30000−15000−10000=5000.
    assert_eq!((credit, payment), (15000, 0));
    for (line, amount) in [("17", 10000), ("18", 15000), ("21", 5000), ("22", 12000), ("26", 5000)]
    {
        assert_eq!(value(&r, &alloc::format!("form_3800_line_{line}")), amount);
    }
    assert_eq!(value(&r, "business_credit_empowerment_zone_2024_used"), 3000);
    assert_eq!(value(&r, "business_credit_empowerment_zone_current_year_used"), 2000);
    assert_eq!(value(&r, "business_credit_empowerment_zone_current_year_to_next_year"), 7000);
}

#[test]
fn unit16_r21_childcare_cap_allocates_before_use_and_retains_the_approved_notice() {
    let mut entries = vec![
        entry(BusinessCredit::EmployerChildcare, 2026, 300000),
        entry(BusinessCredit::EmployerChildcare, 2026, 400000),
    ];
    let mut result = FederalResult::empty(0);
    crate::general_business_credit::cap_entries(&mut entries, true, &mut result);
    assert_eq!((entries[0].amount, entries[1].amount), (214286, 285714));
    assert!(
        result
            .review_notes
            .iter()
            .any(|n| { n.display.key == "WARN_EMPLOYER_CHILDCARE_LIMIT_ASSUMED" })
    );
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 101_610_000, 0);
    own(&mut d, BusinessCredit::EmployerChildcare, 300000);
    let mut k1 = ordinary_partner_source(0, true);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::EmployerChildcare,
        line: None,
        amount: 40_000_000,
    });
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    let r = supported(&d);
    assert_eq!(column(&r, 3, "1k", 'e'), 500000);
    assert_eq!(
        value(&r, "business_credit_employer_childcare_current_year_to_next_year"),
        500000 - value(&r, "general_business_credit")
    );
}

#[test]
fn unit16_r19_clean_vehicle_uses_either_year_magi_but_commercial_has_no_test() {
    for (current, prior, expected) in [
        (150000, None, 7500),
        (150001, None, 0),
        (150001, Some(150000), 7500),
        (150001, Some(150001), 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, current * 100, 0);
        own(&mut d, BusinessCredit::CleanVehicle, 7500);
        d.credits.as_mut().unwrap().clean_vehicles = Some(FederalCleanVehicleClaim {
            prior_year: prior.map(|magi| FederalCleanVehiclePrior {
                status: FederalStatus::Single,
                magi: magi * 100,
            }),
            current_territory_addbacks: 0,
            vehicles: vec![],
        });
        let r = supported(&d);
        assert_eq!(
            r.lines
                .iter()
                .find(|line| line.id == "general_business_credit")
                .map_or(0, |line| crate::kernel::ratio::dollars(line.amount)),
            expected
        );
        history(&mut d).credits[0].credit = BusinessCredit::CommercialCleanVehicle;
        let r = supported(&d);
        assert_eq!(value(&r, "general_business_credit"), 7500);
        assert_eq!(column(&r, 3, "1aa", 'e'), 7500);
    }
}

#[test]
fn unit16_r17_nonresearch_k1_catalog_reaches_its_statement_line_for_both_entities() {
    for code in K1CreditCode::ALL {
        if code == K1CreditCode::Research {
            // Research has separate actual-source return controls for both
            // lines, allowed deductions and retained-source history.
            continue;
        }
        let line = match code {
            K1CreditCode::RenewableElectricity => {
                Some(BusinessCredit::RenewableElectricitySpecified)
            }
            K1CreditCode::Transferred => Some(BusinessCredit::AdvancedManufacturingProduction),
            c if c.needs_line() => Some(BusinessCredit::DifferentialWage),
            _ => None,
        };
        let item = K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            code,
            line,
            amount: 500_000,
        };
        assert!(K1Credit::valid_list(core::slice::from_ref(&item)));
        let page: crate::K1CreditPage = item.clone().into();
        assert_eq!(K1Credit::from(page), item);
        let target = item.credit();
        for corporation in [false, true] {
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(1, Owner::Taxpayer, 11_610_000, 0);
            let source = if corporation {
                let mut data = complete_ordinary_scorp(0, true);
                data.credits.push(item.clone());
                data.housing_confirmed = Some(true);
                FederalIncome::Scorp { data }
            } else {
                let mut data = ordinary_partner_source(0, true);
                data.credits.push(item.clone());
                data.housing_confirmed = Some(true);
                FederalIncome::Partnership { data }
            };
            add(&mut d, income(source));
            let r = supported(&d);
            assert_eq!(
                column(&r, 3, target.line(2026), 'd')
                    + column(&r, 3, target.line(2026), 'e')
                    + column(&r, 3, target.line(2026), 'f'),
                5000,
                "{code:?}, S corporation={corporation}"
            );
            assert_eq!(supported(&restart(d).draft), r);
        }
        let mut bad = item;
        bad.line = if code.needs_line() { None } else { Some(BusinessCredit::Other) };
        assert!(!K1Credit::valid_list(&[bad]));
    }
}

// Diagnostic for the original R14 own-form route: the existing K-1 facts
// distinguish the activities, but no accepted field attaches an own-form
// amount to either property. This is not a production substitute route.
#[test]
fn unit16_r14_own_form_amount_needs_its_actual_k1_property_relationship() {
    use crate::general_business_credit::{calculate, resolve_source};
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 31_610_000, 0);
    let mut k1 = ordinary_partner_source(0, true);
    k1.investment_property = vec![
        K1InvestmentProperty::RehabilitationRental {
            id: 0,
            own_form: None,
            expenditures: 2_000_000,
            placed_year: 2025,
        },
        K1InvestmentProperty::RehabilitationOther {
            id: 0,
            own_form: None,
            expenditures: 2_000_000,
            placed_year: 2025,
        },
    ];
    k1.investment_confirmed = Some(false);
    assert!(k1.investment_property[0].rental_real_estate());
    assert!(!k1.investment_property[1].rental_real_estate());
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    let id = d.income.last().unwrap().id;
    let accepted_a = d.clone();
    let accepted_b = d.clone();
    assert_eq!(accepted_a, accepted_b);
    assert_eq!(restart(accepted_a.clone()).draft, accepted_b);
    let r = supported(&accepted_a);
    assert_eq!(value(&r, "taxable_income"), 300000);
    assert!(r.lines.iter().all(|v| v.id != "general_business_credit" || v.amount == 0));
    assert!(r.review_notes.iter().any(|v| v.display.key == "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT"));
    for source in [
        CreditSource::Business { business: Some(id), reduced: true },
        CreditSource::Farm { farm: Some(id), reduced: true },
        CreditSource::Rental { rental: Some(id), reduced: true },
    ] {
        assert!(resolve_source(&d, &source).is_none());
    }
    let mut own_form = entry(BusinessCredit::Rehabilitation, 2026, 800);
    own_form.activity = Some(id);
    let mut other = r.clone();
    assert_eq!(calculate(None, vec![own_form.clone()], &[], capacity(), &mut other).0, 800);
    own_form.passive = Some(PassiveWorksheet::Rehabilitation);
    let mut rental = r;
    assert_eq!(calculate(None, vec![own_form], &[], capacity(), &mut rental).0, 0);
    assert_eq!(
        value(&rental, "business_credit_rehabilitation_current_year_passive_to_next_year"),
        800
    );
}

#[test]
fn unit16_f3b_form8615_uses_same_parent_for_both_passive_tax_comparisons() {
    let mut d = young_interest_return();
    d.income[0].income = savings_income(81350, 0);
    let mut k1 = ordinary_partner_source(10000, false);
    k1.income_source = Some(crate::kiddie::BusinessIncomeSource::Capital);
    k1.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::DifferentialWage,
        line: None,
        amount: 300_000,
    });
    add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 90000);
    // Same joint parent's TI150000/tax22424. Original child's net unearned
    // 88650 gives parent tax42472, share20048 plus child's1350 row136.
    // Reducing child TI to80000 caps net unearned at80000: parent40396,
    // share17972 and child's remaining0. Difference20184−17972=2212.
    assert_eq!(value(&r, "regular_income_tax"), 20184);
    assert_eq!(value(&r, "form8582cr_line_6"), 2212);
    assert_eq!(value(&r, "general_business_credit"), 2212);
    assert_eq!(
        value(&r, "business_credit_differential_wage_current_year_passive_to_next_year"),
        788
    );
}

#[test]
fn unit16_actual_k1_own_form_uses_property_classification_and_one_basis_reduction() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 31_610_000, 0);
    for rental in [false, true] {
        let own_form = Some(OwnInvestmentCredit { amount: 80_000, reduced: true });
        let property = if rental {
            K1InvestmentProperty::RehabilitationRental {
                id: 0,
                expenditures: 2_000_000,
                placed_year: 2025,
                own_form,
            }
        } else {
            K1InvestmentProperty::RehabilitationOther {
                id: 0,
                expenditures: 2_000_000,
                placed_year: 2025,
                own_form,
            }
        };
        let mut k1 = ordinary_partner_source(0, true);
        k1.investment_property.push(property);
        k1.investment_confirmed = None;
        let mut draft = d.clone();
        add(&mut draft, income(FederalIncome::Partnership { data: k1 }));
        let r = supported(&draft);
        assert_eq!(value(&r, "general_business_credit"), if rental { 0 } else { 800 });
        assert!(!noted(&r, "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT"));
        assert_eq!(supported(&restart(draft.clone()).draft), r);
        if rental {
            assert_eq!(
                value(&r, "business_credit_rehabilitation_current_year_passive_to_next_year"),
                800
            );
        }
    }
    let energy = K1InvestmentProperty::Energy {
        id: 0,
        basis: 1_000_000,
        rate: EnergyRate {
            increased_rate: false,
            domestic_content: false,
            energy_community: false,
        },
        own_form: Some(OwnInvestmentCredit { amount: 200_000, reduced: true }),
    };
    assert_eq!(energy.current_year_credit(), 2000);
    assert_eq!(
        crate::investment_credit::k1_basis_decrease(core::slice::from_ref(&energy)),
        100_000
    );
    let mut scorp = complete_ordinary_scorp(0, true);
    let baseline = scorp.basis_result().unwrap();
    scorp.investment_property.push(energy);
    let with_credit = scorp.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(baseline.stock.get() - with_credit.stock.get()), 1000);
    let rehabilitation = K1InvestmentProperty::RehabilitationOther {
        id: 0,
        expenditures: 2_000_000,
        placed_year: 2026,
        own_form: Some(OwnInvestmentCredit { amount: 80_000, reduced: true }),
    };
    assert_eq!(crate::investment_credit::k1_basis_decrease(&[rehabilitation]), 400_000);
}

#[test]
fn unit16_research_k1_current_both_lines_and_history_share_actual_source_cap() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_050_000, 0);
    let mut k1 = complete_ordinary_scorp(19500, true);
    for line in [BusinessCredit::Research, BusinessCredit::ResearchSmallBusiness] {
        k1.credits.push(K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            code: K1CreditCode::Research,
            line: Some(line),
            amount: 500_000,
        });
    }
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let source = d.income.last().unwrap().id;
    let mut prior = carry(BusinessCredit::Research, 2024, 1000);
    prior.research_source = Some(source);
    history(&mut d).carryforwards.push(prior);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 100000);
    assert_eq!(value(&r, "qbi_deduction"), 3900);
    // Source19500−QBI3900=15600; common16100/denominator116100 gives13437.
    // Shared1c/4i/all-vintage cap=round(16712×13437/100000)=2246.
    assert_eq!(value(&r, "general_business_credit"), 2246);
    assert_eq!(
        column(&r, 3, "1c", 'i') + column(&r, 3, "4i", 'i') + column(&r, 4, "1c", 'f'),
        2246
    );
}

#[test]
fn unit16_research_source_excludes_form461_disallowed_loss() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 50_000_000, 0);
    let mut k1 = sole_qbi_loss(500_000, 1_000_000, 0);
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(id);
    let mut credit = carry(BusinessCredit::Research, 2024, 10_000);
    credit.research_source = Some(id);
    history(&mut d).carryforwards.push(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 244_000);
    assert_eq!(value(&r, "agi"), 244_000);
    assert_eq!(value(&r, "form_3800_line_11"), 49_384);
    assert_eq!(value(&r, "form_3800_line_16"), 9370);
    assert_eq!(value(&r, "general_business_credit"), 9370);
}

#[test]
fn unit16_research_source_uses_only_schedule_d_allowed_loss() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
    let mut k1 = sole_qbi_loss(0, 1_000_000, 0);
    k1.capital = Some(k1_capital(-100_000, 0));
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(id);
    let mut credit = carry(BusinessCredit::Research, 2024, 10_000);
    credit.research_source = Some(id);
    history(&mut d).carryforwards.push(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 147_000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 97_000);
    assert_eq!(value(&r, "form_3800_line_11"), 24_014);
    assert_eq!(value(&r, "form_3800_line_16"), 9220);
    assert_eq!(value(&r, "general_business_credit"), 9220);
}

#[test]
fn unit16_research_source_counts_portfolio_1250_component_once() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_610_000, 0);
    let mut k1 = sole_qbi_loss(0, 1_000_000, 0);
    k1.capital = Some(k1_portfolio_1250(10_000, 0, 6000));
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let mut credit = carry(BusinessCredit::Research, 2024, 10_000);
    credit.research_source = Some(id);
    history(&mut d).carryforwards.push(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 116_100);
    assert_eq!(value(&r, "taxable_income"), 100_000);
    assert_eq!(value(&r, "form_3800_line_11"), 16_438);
    // The ordinary90000 TaxTable midpoint gives14518, plus1320/600 capital tax.
    // Source10000 less its16100*10000/116100 common share1387 =8613.
    assert_eq!(value(&r, "general_business_credit"), 1416);
}

#[test]
fn unit16_research_nonpassive_rental_income_and_qbi_keep_actual_source() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_050_000, 0);
    let mut source = rental(19_500, 0);
    source.buildings[0].cost = source.buildings[0].land;
    source.property_type = RentalPropertyType::VacationShortTerm {
        stays: ShortTermStays::SevenDaysOrLess { material_participation: true },
    };
    source.qbi_business = RentalBusiness::Yes { material_participation: true };
    add(&mut d, income(FederalIncome::Rental { data: source }));
    let id = d.income.last().unwrap().id;
    let mut credit = carry(BusinessCredit::Research, 2024, 10_000);
    credit.research_source = Some(id);
    history(&mut d).carryforwards.push(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 120_000);
    assert_eq!(value(&r, "qbi_deduction"), 3900);
    assert_eq!(value(&r, "taxable_income"), 100_000);
    assert_eq!(value(&r, "general_business_credit"), 2246);
}

#[test]
fn unit16_retained_capital_original_missing_source_witness_keeps_financial_controls() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 18_000_000, 0));
    let mut k1 = sole_qbi_loss(0, 1_000_000, 0);
    k1.interest = 1_000_000;
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(id);
    let mut old = capital_history();
    old.agi = 10_000_000;
    old.deductions = 1_500_000;
    old.short = -1_300_000;
    old.long = 0;
    old.deducted_loss = 300_000;
    old.investment_parts = Some(InvestmentCapitalParts { short: 1_000_000, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![old];
    let mut credit = carry(BusinessCredit::Research, 2024, 10_000);
    credit.research_source = Some(id);
    history(&mut d).carryforwards.push(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 207_000);
    assert_eq!(value(&r, "taxable_income"), 190_900);
    assert_eq!(value(&r, "capital_loss_carryforward"), 7000);
    assert_eq!(value(&r, "form_3800_line_11"), 38_414);
    assert_eq!(value(&r, "form_3800_line_16"), 8020);
    // Historical pre-approval witness: the legacy fields alone do not
    // identify the permitted3000 loss source. Keep those financial controls
    // beside the actual approved-source adapter proof below.
    let same = d.clone();
    assert_eq!(d, same);
    assert_eq!(supported(&same), r);
    for (source_net, expected) in [(27_000, 5011), (30_000, 5567)] {
        let numerator = source_net - crate::kernel::ratio::fraction(16_100, source_net, 207_000);
        let mut cap = capacity();
        cap.status = FederalStatus::Single;
        cap.regular_tax_before_credits = 38_414;
        cap.tentative_minimum_tax = 30_394;
        cap.taxable_income = 190_900;
        cap.research_numerators = vec![(id, numerator)];
        let mut input = entry(BusinessCredit::Research, 2024, 10_000);
        input.activity = Some(id);
        let mut result = FederalResult::empty(0);
        let (credit, payment) =
            crate::general_business_credit::calculate(None, vec![input], &[], cap, &mut result);
        assert_eq!((credit, payment), (expected, 0));
    }
}

#[test]
fn unit16_current_capital_limit_prorates_entity_and_unrelated_loss_items() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 18_000_000, 0));
    let mut k1 = sole_qbi_loss(0, 1_000_000, 0);
    k1.capital = Some(k1_capital(-40_000, 0));
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(id);
    add(&mut d, income(investment_sale_gain(GainTerm::Short, -60_000, -60_000)));
    let mut old = carry(BusinessCredit::Research, 2024, 10_000);
    old.research_source = Some(id);
    history(&mut d).carryforwards.push(old);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 197_000);
    assert_eq!(value(&r, "taxable_income"), 180_900);
    assert_eq!(value(&r, "capital_loss_carryforward"), 97_000);
    // 3000 allowed loss: this actual K1's40000/100000 share is1200.
    let net = 20_000 - 1200;
    let numerator = net - crate::kernel::ratio::fraction(16_100, net, 197_000);
    let expected = crate::kernel::ratio::fraction(36_014, numerator, 180_900);
    assert_eq!(value(&r, "general_business_credit"), expected);
}

#[test]
fn unit16_form461_loss_allocation_uses_actual_loss_sources_before_salary() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 50_000_000, 0));
    let mut ids = vec![];
    for loss in [400_000, 100_000] {
        let mut k1 = sole_qbi_loss(loss, 1_000_000, 0);
        k1.amt = Some(amt_records(&k1));
        add(&mut d, income(FederalIncome::Scorp { data: k1 }));
        ids.push(d.income.last().unwrap().id);
    }
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(ids[0]);
    for (id, amount) in ids.iter().zip([50_000, 1000]) {
        let mut old = carry(BusinessCredit::Research, 2024, amount);
        old.research_source = Some(*id);
        history(&mut d).carryforwards.push(old);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 244_000);
    assert_eq!(value(&r, "agi"), 494_000);
    assert_eq!(value(&r, "taxable_income"), 477_900);
    // EntityA has400/500 of the ordinary losses: withheld195200, source
    // net250000−400000+195200=45200. EntityB remains negative51200.
    let numerator = 45_200 - crate::kernel::ratio::fraction(16_100, 45_200, 494_000);
    let expected =
        crate::kernel::ratio::fraction(value(&r, "form_3800_line_11"), numerator, 477_900);
    assert_eq!(value(&r, "general_business_credit"), expected);
    assert_eq!(
        value(&r, &alloc::format!("business_credit_research_source_{}_filer_0_2024_used", ids[1])),
        0
    );
}

pub(in crate::tests) fn retained_research_draft() -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 18_000_000, 0));
    let mut k1 = sole_qbi_loss(0, 1_000_000, 0);
    k1.interest = 1_000_000;
    k1.amt = Some(amt_records(&k1));
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let id = d.income.last().unwrap().id;
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.paying_scorporation = Some(id);
    let mut old = carry(BusinessCredit::Research, 2024, 10_000);
    old.research_source = Some(id);
    history(&mut d).carryforwards.push(old);
    (d, id)
}

#[test]
fn unit16_retained_nol_entity_source_is_not_recorded_by_loss_year() {
    let (mut d, id) = retained_research_draft();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 10_000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 200_000);
    assert_eq!(value(&r, "taxable_income"), 183_900);
    assert_eq!(value(&r, "form_3800_line_11"), 36_734);
    assert_eq!(value(&r, "form_3800_line_14"), 31_174);
    assert_eq!(value(&r, "form_3800_line_16"), 5560);
    let same = d.clone();
    assert_eq!(same, d);
    assert_eq!(supported(&same), r);
    for (source_net, expected) in [(20_000, 3673), (30_000, 5510)] {
        let numerator = source_net - crate::kernel::ratio::fraction(16_100, source_net, 200_000);
        let mut cap = capacity();
        cap.status = FederalStatus::Single;
        cap.regular_tax_before_credits = 36_734;
        cap.tentative_minimum_tax = value(&r, "form_3800_line_14");
        cap.taxable_income = 183_900;
        cap.research_numerators = vec![(id, numerator)];
        let mut input = entry(BusinessCredit::Research, 2024, 10_000);
        input.activity = Some(id);
        let mut result = FederalResult::empty(0);
        let (credit, payment) =
            crate::general_business_credit::calculate(None, vec![input], &[], cap, &mut result);
        assert_eq!((credit, payment), (expected, 0));
    }
}

#[test]
fn unit16_retained_nol_origin_uses_exact_owner_deduction_and_preserves_tax() {
    let (mut d, id) = retained_research_draft();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 10_000)]);
    let unrelated = supported(&d);
    assert_eq!(value(&unrelated, "general_business_credit"), 5510);
    let NolHistory::Records { years, .. } = &mut d.screening.as_mut().unwrap().nol else {
        panic!()
    };
    years[0].research_parts = Some(ResearchLossParts::Parts {
        sources: vec![ResearchLossPart { source: id, amount: 1_000_000 }],
    });
    assert!(d.valid_stored());
    let attributed = supported(&d);
    assert_eq!(value(&attributed, "general_business_credit"), 3673);
    for row in [
        "agi",
        "taxable_income",
        "form_3800_line_11",
        "form_3800_line_14",
        "form_3800_line_16",
        "prior_nol_deduction",
        "nol_2024_carryforward",
    ] {
        assert_eq!(value(&attributed, row), value(&unrelated, row), "{row}");
    }
    let accepted = d.clone();
    let NolHistory::Records { years, .. } = &mut d.screening.as_mut().unwrap().nol else {
        panic!()
    };
    let Some(ResearchLossParts::Parts { sources }) = &mut years[0].research_parts else { panic!() };
    sources[0].source = 999999;
    assert!(!d.valid_stored());
    d = accepted;
    let NolHistory::Records { years, .. } = &mut d.screening.as_mut().unwrap().nol else {
        panic!()
    };
    let Some(ResearchLossParts::Parts { sources }) = &mut years[0].research_parts else { panic!() };
    sources[0].amount = 1_000_100;
    assert!(!d.valid_stored());
}

#[test]
fn unit16_retained_nol_parts_prorate_deduction_without_using_carry_absorption() {
    for (year, deduction, carry, attributed) in
        [(2016, 100_000, 16_100, 20_000), (2024, 67_120, 32_880, 13_424)]
    {
        let (mut d, id) = retained_research_draft();
        let loss = NolYear {
            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
            injured_owners: None,
            year,
            balance: 10_000_000,
            research_parts: Some(ResearchLossParts::Parts {
                sources: vec![ResearchLossPart { source: id, amount: 2_000_000 }],
            }),
        };
        d.screening.as_mut().unwrap().nol =
            NolHistory::Records { years: vec![loss], confirmed: true };
        let vintages = crate::estate_trust::losses(&d, false);
        let allocation = NolHistory::allocate(83_900, &vintages.vintages, |_| 83_900).unwrap();
        assert_eq!(allocation.deduction, deduction);
        assert_eq!(allocation.carryforward, crate::kernel::ratio::exact::whole(carry));
        // Pre-2018 deduction100000 exceeds absorption83900; attribution must
        // use the former. The limited2024 deduction gives20000/100000×67120.
        assert_eq!(allocation.research_deductions(&d).get(&id), Some(&attributed));
    }
}

#[test]
fn unit16_retained_nol_source_rounding_preserves_identity_when_rows_move() {
    let (mut d, first) = retained_research_draft();
    add(&mut d, income(cash_business(0)));
    let second = d.income.last().unwrap().id;
    d.screening.as_mut().unwrap().nol = NolHistory::Records {
        years: vec![NolYear {
            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
            injured_owners: None,
            year: 2024,
            balance: 200,
            research_parts: Some(ResearchLossParts::Parts {
                sources: vec![
                    ResearchLossPart { source: first, amount: 100 },
                    ResearchLossPart { source: second, amount: 100 },
                ],
            }),
        }],
        confirmed: true,
    };
    assert!(d.valid_stored());
    let vintages = crate::estate_trust::losses(&d, false);
    let allocation = NolHistory::allocate(1, &vintages.vintages, |_| 1).unwrap();
    assert_eq!(allocation.deduction, 1);
    let before = allocation.research_deductions(&d);
    assert_eq!(before.values().sum::<i64>(), 1);
    let NolHistory::Records { years, .. } = &mut d.screening.as_mut().unwrap().nol else {
        panic!()
    };
    let Some(ResearchLossParts::Parts { sources }) = &mut years[0].research_parts else { panic!() };
    sources.reverse();
    // Moving the same actual facts cannot move the owner's rounded dollar
    // to a different business's research numerator.
    assert_eq!(allocation.research_deductions(&d), before);
}

pub(in crate::tests) fn retained_charity_draft() -> (FederalDraft, i64) {
    let (mut d, id) = retained_research_draft();
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).real_estate_taxes = 1_000_000;
    ded.charity_carryovers.push(CharityCarryover {
        research_parts: None,
        year: 2024,
        class: CharityClass::Cash60,
        balance: 1_000_000,
        appreciation: None,
        same_filers: true,
    });
    let facts = itemized(ded).clone();
    ded.choice = DeductionSelection::Itemized { facts };
    (d, id)
}

#[test]
fn unit16_retained_charitable_gift_entity_source_is_not_in_its_class_year() {
    extern crate std;
    let (d, id) = retained_charity_draft();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 210_000);
    let allowed = value(&r, "charity_carry_2024_cash_sixty_deduction");
    let taxable = value(&r, "taxable_income");
    assert_eq!(taxable, 210_000 - 10_000 - allowed);
    let same = d.clone();
    assert_eq!(same, d);
    assert_eq!(supported(&same), r);
    let mut outcomes = vec![];
    for origin in [0, 1, 2] {
        let (net, denominator, common) = match origin {
            0 => (30_000 - allowed, 210_000 - allowed, 10_000),
            1 => (30_000, 210_000, 10_000 + allowed),
            _ => (30_000, 210_000 - allowed, 10_000),
        };
        let numerator = net - crate::kernel::ratio::fraction(common, net, denominator);
        let mut cap = capacity();
        cap.status = FederalStatus::Single;
        cap.regular_tax_before_credits = value(&r, "form_3800_line_11");
        cap.tentative_minimum_tax = value(&r, "form_3800_line_14");
        cap.taxable_income = taxable;
        cap.research_numerators = vec![(id, numerator)];
        let mut input = entry(BusinessCredit::Research, 2024, 10_000);
        input.activity = Some(id);
        let mut result = FederalResult::empty(0);
        let (credit, payment) =
            crate::general_business_credit::calculate(None, vec![input], &[], cap, &mut result);
        assert_eq!(payment, 0);
        outcomes.push(credit);
        std::eprintln!(
            "retained charity origin={origin}, allowed={allowed}, TI={taxable}, L11={}, TMT={}, net={net}, denominator={denominator}, common={common}, numerator={numerator}, used={credit}",
            value(&r, "form_3800_line_11"),
            value(&r, "form_3800_line_14")
        );
    }
    assert_eq!(allowed, 8950);
    assert_eq!(outcomes, vec![4026, 5493, 5737]);
}

pub(in crate::tests) fn rental_own_form_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    let mut property = rental(24_000, 0);
    property.buildings = vec![RentalBuilding {
        rehabilitation: Some(RehabilitationCredit {
            period_start: 20240201,
            phased: false,
            building_basis: 1_000_000,
            period_expenditures: 2_000_000,
            earlier: vec![],
            freed: 0,
            confirmed: false,
        }),
        ..purchased_rental_building(2_000_000, 0, 20260115)
    }];
    add(&mut d, income(FederalIncome::Rental { data: property }));
    let id = d.income.last().unwrap().id;
    history(&mut d).credits.push(EnteredBusinessCredit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        credit: BusinessCredit::Rehabilitation,
        amount: 80_000,
        source: CreditSource::Rental { rental: Some(id), reduced: false },
        election: CreditElection::None,
    });
    d
}

// Finite R14 route diagnostic only. The generic entered source's reduction
// confirmation cannot change a purchased RentalBuilding's original cost.
#[test]
fn unit16_r14_rental_own_form_route_has_no_separate_basis_reduction_field() {
    let mut d = rental_own_form_draft();
    let truthful = supported(&d);
    assert!(truthful.lines.iter().all(|line| line.id != "general_business_credit"));
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&truthful, "rental_2_depreciation"), 69_697);
    assert_eq!(displayed_line_amount(&truthful, "rental_2_depreciation"), 69_700);
    // Turning the confirmation to Yes without changing any property fact
    // admits800 but still depreciates unreduced20000. This is not a truthful
    // repair; purchased cost cannot be replaced with adjusted16000.
    history(&mut d).credits[0].source =
        CreditSource::Rental { rental: Some(d.income.last().unwrap().id), reduced: true };
    let asserted = supported(&d);
    assert_eq!(value(&asserted, "general_business_credit"), 800);
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&asserted, "rental_2_depreciation"), 69_697);
    assert_eq!(displayed_line_amount(&asserted, "rental_2_depreciation"), 69_700);
    assert_eq!(value(&asserted, "agi"), 123_303);
    // The existing automatic owner demonstrates the required single4000
    // section50(c) reduction on these property facts, without altering cost.
    history(&mut d).credits.clear();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[0].rehabilitation.as_mut().unwrap().confirmed = true;
    let automatic = supported(&d);
    assert_eq!(value(&automatic, "general_business_credit"), 800);
    assert_eq!(value(&automatic, "rental_2_building_0_basis_reduction"), 4000);
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&automatic, "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&automatic, "rental_2_depreciation"), 55_800);
    assert_eq!(value(&automatic, "agi"), 123_442);
}

#[test]
fn unit16_r14_multiple_rental_improvements_need_entered_basis_allocation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    let mut property = rental(24_000, 0);
    property.buildings = [(20260115, 20240201), (20261215, 20250101)]
        .into_iter()
        .map(|(in_service, period_start)| RentalBuilding {
            rehabilitation: Some(RehabilitationCredit {
                period_start,
                phased: false,
                building_basis: 500_000,
                period_expenditures: 1_000_000,
                earlier: vec![],
                freed: 0,
                confirmed: false,
            }),
            ..purchased_rental_building(1_000_000, 0, in_service)
        })
        .collect();
    add(&mut d, income(FederalIncome::Rental { data: property }));
    let id = d.income.last().unwrap().id;
    history(&mut d).credits.push(EnteredBusinessCredit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        credit: BusinessCredit::Rehabilitation,
        amount: 40_000,
        source: CreditSource::Rental { rental: Some(id), reduced: true },
        election: CreditElection::None,
    });
    // This asserted reduction still has no per-building source value. It is
    // not a truthful completed route: the actual400 credit identifies only
    // the rental, not which improvement has the2000 basis reduction.
    let asserted = supported(&d);
    assert_eq!(value(&asserted, "general_business_credit"), 400);
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&asserted, "rental_2_depreciation"), 36_363);
    assert_eq!(displayed_line_amount(&asserted, "rental_2_depreciation"), 36_400);
    history(&mut d).credits.clear();
    for (index, depreciation, exact_cents, agi) in
        [(0, 294, 29_394, 123_706), (1, 361, 36_060, 123_639)]
    {
        let mut component = d.clone();
        let FederalIncome::Rental { data } = &mut component.income.last_mut().unwrap().income
        else {
            panic!()
        };
        data.buildings[index].rehabilitation.as_mut().unwrap().confirmed = true;
        assert_eq!(
            data.buildings.iter().map(|b| b.cost).collect::<Vec<_>>(),
            vec![1_000_000, 1_000_000]
        );
        let r = supported(&component);
        // These existing automatic-owner controls establish the financial
        // consequences; they do not pretend to answer the missing manual
        // source allocation or replace declined eligibility confirmations.
        assert_eq!(value(&r, "general_business_credit"), 400);
        assert_eq!(value(&r, &alloc::format!("rental_2_building_{index}_basis_reduction")), 2000);
        // 2026-10-19, D-2026-10-19-01: Jan23/660 plus Dec1/660;
        // reduced Jan$8000 gives29394c; reduced Dec$8000 gives36060c.
        assert_eq!(line_amount(&r, "rental_2_depreciation"), exact_cents);
        assert_eq!(displayed_line_amount(&r, "rental_2_depreciation"), depreciation * 100);
        assert_eq!(value(&r, "agi"), agi);
    }
}

pub(in crate::tests) fn retained_capital_draft() -> (FederalDraft, i64) {
    let (mut d, id) = retained_research_draft();
    let mut old = capital_history();
    old.agi = 10_000_000;
    old.deductions = 1_500_000;
    old.short = -1_300_000;
    old.long = 0;
    old.deducted_loss = 300_000;
    old.investment_parts = Some(InvestmentCapitalParts { short: 1_000_000, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![old];
    (d, id)
}

#[test]
fn unit16_retained_capital_actual_origin_preserves_schedule_d_amt_and_niit() {
    let (mut d, id) = retained_capital_draft();
    let unrelated = supported(&d);
    assert_eq!(value(&unrelated, "general_business_credit"), 5567);
    d.screening.as_mut().unwrap().capital_history[0].research_parts =
        Some(ResearchCapitalParts::Parts {
            sources: vec![ResearchCapitalPart { source: id, short: 1_000_000, long: 0 }],
        });
    assert!(d.valid_stored());
    let actual = supported(&d);
    assert_eq!(value(&actual, "general_business_credit"), 5011);
    for row in [
        "agi",
        "taxable_income",
        "capital_loss_carryforward",
        "form_3800_line_11",
        "form_3800_line_14",
        "form_3800_line_16",
    ] {
        assert_eq!(value(&actual, row), value(&unrelated, row), "{row}");
    }
    let accepted = d.clone();
    let Some(ResearchCapitalParts::Parts { sources }) =
        &mut d.screening.as_mut().unwrap().capital_history[0].research_parts
    else {
        panic!()
    };
    sources[0].short += 100;
    assert!(!d.valid_stored());
    d = accepted.clone();
    let Some(ResearchCapitalParts::Parts { sources }) =
        &mut d.screening.as_mut().unwrap().capital_history[0].research_parts
    else {
        panic!()
    };
    sources[0].long = 100;
    assert!(!d.valid_stored());
    d = accepted;
    let Some(ResearchCapitalParts::Parts { sources }) =
        &mut d.screening.as_mut().unwrap().capital_history[0].research_parts
    else {
        panic!()
    };
    sources[0].source = 999999;
    assert!(!d.valid_stored());
}

#[test]
fn unit16_retained_capital_source_parts_conserve_owner_dollars_independent_of_row_order() {
    let (mut d, id) = retained_capital_draft();
    add(&mut d, income(cash_business(0)));
    let other = d.income.last().unwrap().id;
    let old = &mut d.screening.as_mut().unwrap().capital_history[0];
    old.short = -300_100;
    old.investment_parts = Some(InvestmentCapitalParts { short: 100, long: 0, amt: None });
    old.research_parts = Some(ResearchCapitalParts::Parts {
        sources: vec![
            ResearchCapitalPart { source: id, short: 50, long: 0 },
            ResearchCapitalPart { source: other, short: 50, long: 0 },
        ],
    });
    assert!(d.valid_stored());
    let original = crate::retained_research::capital_losses(&d);
    assert_eq!(original.values().sum::<i64>(), 1);
    let Some(ResearchCapitalParts::Parts { sources }) =
        &mut d.screening.as_mut().unwrap().capital_history[0].research_parts
    else {
        panic!()
    };
    sources.reverse();
    assert_eq!(crate::retained_research::capital_losses(&d), original);
}

// Approved e39afe8: an actual reduction belongs to its property, independently
// of the entered credit amount or the declined automatic claim.
#[test]
fn unit16_r14_rental_own_form_actual_basis_reduction_preserves_original_cost() {
    let mut d = rental_own_form_draft();
    history(&mut d).credits[0].source = CreditSource::Rental { rental: Some(2), reduced: true };
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[0].credit_reduction = 400_000;
    assert_eq!(data.buildings[0].cost, 2_000_000);
    assert!(!data.buildings[0].rehabilitation.as_ref().unwrap().confirmed);
    assert!(d.valid_stored());
    let manual = supported(&d);
    assert_eq!(value(&manual, "general_business_credit"), 800);
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&manual, "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&manual, "rental_2_depreciation"), 55_800);
    assert_eq!(value(&manual, "agi"), 123_442);
    // Normal automatic precedence ignores the separate amount, rather than
    // applying both reductions to the same original cost.
    history(&mut d).credits.clear();
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[0].credit_reduction = 1_000_000;
    data.buildings[0].rehabilitation.as_mut().unwrap().confirmed = true;
    let automatic = supported(&d);
    assert_eq!(value(&automatic, "general_business_credit"), 800);
    assert_eq!(value(&automatic, "rental_2_building_0_basis_reduction"), 4000);
    // 2026-10-19, D-2026-10-19-01: exact source cents; dollars on the form.
    assert_eq!(line_amount(&automatic, "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&automatic, "rental_2_depreciation"), 55_800);
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[0].credit_reduction = 2_000_001;
    assert!(!d.valid_stored());
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else { panic!() };
    data.buildings[0].land = 100_000;
    data.buildings[0].credit_reduction = 1_900_001;
    assert!(!d.valid_stored());
}

#[test]
fn unit16_r14_rental_manual_reduction_identifies_actual_improvement() {
    for (index, depreciation, exact_cents, agi) in
        [(0, 294, 29_394, 123_706), (1, 361, 36_060, 123_639)]
    {
        let mut d = rental_own_form_draft();
        history(&mut d).credits[0].amount = 40_000;
        history(&mut d).credits[0].source = CreditSource::Rental { rental: Some(2), reduced: true };
        let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
            panic!()
        };
        let mut first = data.buildings[0].clone();
        first.cost = 1_000_000;
        let claim = first.rehabilitation.as_mut().unwrap();
        claim.building_basis = 500_000;
        claim.period_expenditures = 1_000_000;
        let mut second = first.clone();
        second.in_service = 20261215;
        second.rehabilitation.as_mut().unwrap().period_start = 20250101;
        data.buildings = vec![first, second];
        data.buildings[index].credit_reduction = 200_000;
        assert!(
            data.buildings
                .iter()
                .all(|b| b.cost == 1_000_000 && !b.rehabilitation.as_ref().unwrap().confirmed)
        );
        assert!(d.valid_stored());
        let result = supported(&d);
        assert_eq!(value(&result, "general_business_credit"), 400);
        // 2026-10-19, D-2026-10-19-01: Jan23/660 plus Dec1/660;
        // reduced Jan$8000 gives29394c; reduced Dec$8000 gives36060c.
        assert_eq!(line_amount(&result, "rental_2_depreciation"), exact_cents);
        assert_eq!(displayed_line_amount(&result, "rental_2_depreciation"), depreciation * 100);
        assert_eq!(value(&result, "agi"), agi);
    }
}

#[test]
fn unit16_retained_charity_actual_k1_personal_and_other_entity_keep_owner_deduction() {
    for (origin, expected) in [(0, 4026), (1, 5493), (2, 5737)] {
        let (mut d, source) = retained_charity_draft();
        d.deductions.as_mut().unwrap().charity_carryovers[0].research_parts = match origin {
            1 => None,
            _ => Some(ResearchCharityParts::Parts {
                sources: vec![ResearchCharityPart {
                    origin: if origin == 0 {
                        ResearchCharityOrigin::CurrentK1 { source }
                    } else {
                        ResearchCharityOrigin::OtherEntity(OtherCharityEntity {
                            name: "Synthetic retained donor entity".into(),
                            ein: "12-3456789".into(),
                        })
                    },
                    amount: 1_000_000,
                    original_appreciation: None,
                }],
            }),
        };
        assert!(d.valid_stored());
        let result = supported(&d);
        assert_eq!(value(&result, "general_business_credit"), expected);
        assert_eq!(value(&result, "agi"), 210_000);
        assert_eq!(value(&result, "taxable_income"), 191_050);
        assert_eq!(value(&result, "charity_carry_2024_cash_sixty_deduction"), 8950);
        assert_eq!(value(&result, "form_3800_line_11"), 38_450);
        assert_eq!(value(&result, "form_3800_line_14"), 28_847);
        let accepted = d.clone();
        if let Some(ResearchCharityParts::Parts { sources }) =
            &mut d.deductions.as_mut().unwrap().charity_carryovers[0].research_parts
        {
            sources[0].amount += 100;
            assert!(!d.valid_stored());
            d = accepted;
            let Some(ResearchCharityParts::Parts { sources }) =
                &mut d.deductions.as_mut().unwrap().charity_carryovers[0].research_parts
            else {
                unreachable!()
            };
            match &mut sources[0].origin {
                ResearchCharityOrigin::CurrentK1 { source } => *source = 987_654_321,
                ResearchCharityOrigin::OtherEntity(entity) => entity.ein.clear(),
            }
            assert!(!d.valid_stored());
        }
    }
}

#[test]
fn unit16_retained_charity_limited_deduction_conserves_owner_amount_and_source_order() {
    let (mut d, first) = retained_charity_draft();
    add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(0, true) }));
    let second = d.income.last().unwrap().id;
    let mut sources = vec![
        ResearchCharityPart {
            origin: ResearchCharityOrigin::CurrentK1 { source: first },
            amount: 500_000,
            original_appreciation: None,
        },
        ResearchCharityPart {
            origin: ResearchCharityOrigin::CurrentK1 { source: second },
            amount: 500_000,
            original_appreciation: None,
        },
    ];
    let mut results = vec![];
    for _ in 0..2 {
        d.deductions.as_mut().unwrap().charity_carryovers[0].research_parts =
            Some(ResearchCharityParts::Parts { sources: sources.clone() });
        assert!(d.valid_stored());
        results.push(supported(&d));
        sources.reverse();
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(value(&results[0], "charity_carry_2024_cash_sixty_deduction"), 8950);
    // 1050 of the original10000 is removed by the owner's statutory floor;
    // the remaining8950, not10000, is shared by the two equal source gifts.
    assert_eq!(value(&results[0], "agi"), 210_000);
    assert_eq!(value(&results[0], "taxable_income"), 191_050);
}

// Finite source-allocation diagnostic, not an approved original-map oracle.
// Both worlds have the same accepted carryover balance, aggregate appreciation
// and entity/personal original portions. The unrecorded appreciation source
// changes the refigured otherwise-deductible items before limited proration.
#[test]
fn unit16_retained_charity_basis_election_missing_appreciation_source_witness() {
    extern crate std;
    let (mut d, id) = retained_charity_draft();
    let ded = d.deductions.as_mut().unwrap();
    ded.basis_election = true;
    let record = &mut ded.charity_carryovers[0];
    record.class = CharityClass::CapitalGain30;
    record.appreciation = Some(400_000);
    record.research_parts = Some(ResearchCharityParts::Parts {
        sources: vec![ResearchCharityPart {
            origin: ResearchCharityOrigin::CurrentK1 { source: id },
            amount: 500_000,
            original_appreciation: None,
        }],
    });
    // Historical597-byte accepted witness remains archived before84a8ef8.
    // Its missing source fact is now rejected instead of silently selecting
    // the old provisional allocation.
    assert!(!d.valid_stored());
    assert_eq!(
        crate::retained_charity::rule(&d),
        Some("RETAINED_CHARITY_SOURCE_APPRECIATION_NEEDED")
    );
    let bytes = d.to_avro();
    let same = FederalDraft::from_avro(&bytes).unwrap();
    assert_eq!(same, d);
    assert_eq!(same.to_avro(), bytes);
    assert!(!same.valid_stored());
    let mut financial = d.clone();
    financial.deductions.as_mut().unwrap().charity_carryovers[0].research_parts = None;
    let r = supported(&financial);
    assert_eq!(value(&r, "agi"), 210_000);
    assert_eq!(value(&r, "taxable_income"), 195_050);
    assert_eq!(value(&r, "charity_carry_2024_capital_gain_thirty_deduction"), 4950);
    let mut caps = vec![];
    // Each original gift was10000 and previously used5000: the unchanged
    // entity/personal carry portions are5000 each. Aggregate appreciation4000
    // is3000/1000 for entity/personal in A,1000/3000 in B.
    // Both original gifts have positive long-term appreciation. Refigured
    // balances are2000/4000 or4000/2000. Both total6000; the owner's
    // 1050 floor leaves4950, allocated1650/3300 or3300/1650 respectively.
    for source_deduction in [1650, 3300] {
        let net = 30_000 - source_deduction;
        let denominator = 210_000 - source_deduction;
        let common = denominator - 195_050;
        let numerator = net - crate::kernel::ratio::fraction(common, net, denominator);
        let mut cap = capacity();
        cap.status = FederalStatus::Single;
        cap.regular_tax_before_credits = value(&r, "form_3800_line_11");
        cap.tentative_minimum_tax = value(&r, "form_3800_line_14");
        cap.taxable_income = 195_050;
        cap.research_numerators = vec![(id, numerator)];
        let mut input = entry(BusinessCredit::Research, 2024, 10_000);
        input.activity = Some(id);
        let mut result = FederalResult::empty(0);
        let (credit, payment) =
            crate::general_business_credit::calculate(None, vec![input], &[], cap, &mut result);
        assert_eq!(payment, 0);
        caps.push(credit);
    }
    assert_ne!(caps[0], caps[1]);
    std::eprintln!(
        "historical charity basis source now rejected without approved fact: identical missing bytes={}, AGI={}, TI={}, allowed=4950, L11={}, TMT={}, financial_all_personal_control={}, actual_source_A={}, actual_source_B={}",
        bytes.len(),
        value(&r, "agi"),
        value(&r, "taxable_income"),
        value(&r, "form_3800_line_11"),
        value(&r, "form_3800_line_14"),
        value(&r, "general_business_credit"),
        caps[0],
        caps[1]
    );
    if let Ok(folder) = std::env::var("TAX_UNIT16_CHARITY_BASIS_WITNESS_DIR") {
        let folder = std::path::Path::new(&folder);
        std::fs::create_dir_all(folder).unwrap();
        std::fs::write(folder.join("missing-source-draft.avro"), &bytes).unwrap();
        std::fs::write(folder.join("identical-other-world-draft.avro"), same.to_avro()).unwrap();
    }
}

#[test]
fn unit16_retained_charity_basis_source_appreciation_resolves_both_worlds() {
    for (appreciation, source_deduction, credit) in
        [(300_000, 1650, 5362), (100_000, 3300, 5091), (0, 4125, 4953)]
    {
        let (mut d, source) = retained_charity_draft();
        let ded = d.deductions.as_mut().unwrap();
        ded.basis_election = true;
        let record = &mut ded.charity_carryovers[0];
        record.class = CharityClass::CapitalGain30;
        record.appreciation = Some(400_000);
        record.research_parts = Some(ResearchCharityParts::Parts {
            sources: vec![ResearchCharityPart {
                origin: ResearchCharityOrigin::CurrentK1 { source },
                amount: 500_000,
                original_appreciation: Some(appreciation),
            }],
        });
        assert!(d.valid_stored());
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 210000);
        assert_eq!(value(&r, "taxable_income"), 195050);
        assert_eq!(value(&r, "charity_carry_2024_capital_gain_thirty_deduction"), 4950);
        let record = &d.deductions.as_ref().unwrap().charity_carryovers[0];
        let worksheet = crate::charity::Worksheet::calculate(
            210000,
            crate::charity::ClassClaims::default(),
            core::slice::from_ref(record),
            true,
        );
        assert_eq!(
            crate::retained_charity::deductions(core::slice::from_ref(record), &worksheet, true)
                .get(&(true, Some(source))),
            Some(&source_deduction)
        );
        assert_eq!(value(&r, "general_business_credit"), credit);
        assert_eq!(FederalDraft::from_avro(&d.to_avro()).unwrap(), d);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let mut off = d.clone();
        off.deductions.as_mut().unwrap().basis_election = false;
        assert_eq!(
            value(&supported(&off), "charity_carry_2024_capital_gain_thirty_deduction"),
            8950
        );
        off.deductions.as_mut().unwrap().basis_election = true;
        assert_eq!(supported(&off), r);
        let Some(ResearchCharityParts::Parts { sources }) =
            &mut off.deductions.as_mut().unwrap().charity_carryovers[0].research_parts
        else {
            panic!()
        };
        sources[0].original_appreciation = Some(400_001);
        assert!(!off.valid_stored());
    }
}
#[test]
fn unit16_retained_charity_original_appreciation_is_not_capped_by_unused_portion() {
    let (mut d, source) = retained_charity_draft();
    let ded = d.deductions.as_mut().unwrap();
    ded.basis_election = true;
    let record = &mut ded.charity_carryovers[0];
    record.class = CharityClass::CapitalGain30;
    record.appreciation = Some(400_000);
    record.research_parts = Some(ResearchCharityParts::Parts {
        sources: vec![ResearchCharityPart {
            origin: ResearchCharityOrigin::CurrentK1 { source },
            amount: 100_000,
            original_appreciation: Some(300_000),
        }],
    });
    assert!(d.valid_stored());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 210000);
    assert_eq!(value(&r, "taxable_income"), 195050);
    assert_eq!(value(&r, "charity_carry_2024_capital_gain_thirty_deduction"), 4950);
    let record = &d.deductions.as_ref().unwrap().charity_carryovers[0];
    let worksheet = crate::charity::Worksheet::calculate(
        210000,
        crate::charity::ClassClaims::default(),
        core::slice::from_ref(record),
        true,
    );
    let parts =
        crate::retained_charity::deductions(core::slice::from_ref(record), &worksheet, true);
    assert_eq!(parts.values().sum::<i64>(), 4950);
    assert_eq!(parts.get(&(true, Some(source))), Some(&0));
}

#[test]
fn unit16_retained_charity_basis_source_rounding_is_independent_of_row_order() {
    let (mut d, first) = retained_charity_draft();
    add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(0, true) }));
    let second = d.income.last().unwrap().id;
    let ded = d.deductions.as_mut().unwrap();
    ded.basis_election = true;
    let record = &mut ded.charity_carryovers[0];
    record.class = CharityClass::CapitalGain30;
    record.appreciation = Some(400_000);
    record.research_parts = Some(ResearchCharityParts::Parts {
        sources: vec![
            ResearchCharityPart {
                origin: ResearchCharityOrigin::CurrentK1 { source: first },
                amount: 500_000,
                original_appreciation: Some(300_001),
            },
            ResearchCharityPart {
                origin: ResearchCharityOrigin::CurrentK1 { source: second },
                amount: 500_000,
                original_appreciation: Some(99_999),
            },
        ],
    });
    assert!(d.valid_stored());
    let before = supported(&d);
    let Some(ResearchCharityParts::Parts { sources }) =
        &mut d.deductions.as_mut().unwrap().charity_carryovers[0].research_parts
    else {
        unreachable!()
    };
    sources.reverse();
    assert_eq!(supported(&d), before);
    let record = &d.deductions.as_ref().unwrap().charity_carryovers[0];
    let worksheet = crate::charity::Worksheet::calculate(
        210000,
        crate::charity::ClassClaims::default(),
        core::slice::from_ref(record),
        true,
    );
    let parts =
        crate::retained_charity::deductions(core::slice::from_ref(record), &worksheet, true);
    assert_eq!(parts.values().sum::<i64>(), 4950);
    assert_eq!(parts.get(&(true, Some(first))), Some(&1650));
    assert_eq!(parts.get(&(true, Some(second))), Some(&3300));
}

#[test]
fn unit16_repayment_zero_original_capital_carry_has_no_source_for_refigured_loss() {
    extern crate std;
    let (mut d, source) = retained_research_draft();
    add(&mut d, prior_unemployment_repayment(500_000, Owner::Taxpayer));
    d.deductions.as_mut().unwrap().repayment =
        repayment_comparison_draft(RepaymentMethod::Credit).deductions.unwrap().repayment;
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        unreachable!()
    };
    years[0].wages = 0;
    years[0].benefits = 1_875_000;
    years[0].income_tax = 0;
    // Actual incoming ordinary investment losses; no original-year sales or
    // business property. The original return uses all3000, leaving zero.
    years[0].capital = historical_capital(vec![], 3000, 0);
    let mut prior = capital_history();
    prior.agi = 1_575_000;
    prior.deductions = 1_575_000;
    prior.short = -300_000;
    prior.long = 0;
    prior.deducted_loss = 300_000;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 0, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![prior.clone()];
    assert_eq!(crate::extended_rules::capital_caps(&prior).short, 0);
    assert_eq!(prior.research_parts, None);
    assert!(d.valid_stored());
    let bytes = d.to_avro();
    let same = FederalDraft::from_avro(&bytes).unwrap();
    assert_eq!(same, d);
    assert_eq!(same.to_avro(), bytes);
    assert!(same.valid_stored());
    // The inherited narrow combined-tax owner guard still refuses this
    // method. This is not a supported source-information witness or authority
    // to introduce another fact; the counterfactuals below are qualified.
    let actual = estimate(&d);
    assert_eq!(actual.state, ResultState::Unsupported);
    assert!(actual.issues.iter().any(|i| i.display.key
        == "ISSUE_A_FORM_3800_CURRENT_OR_CARRIED_CREDIT_WITH_A_SECTION_1341_REPAYMENT_A267A4E8"));
    assert_eq!(estimate(&same), actual);
    let mut counterfactual = d.clone();
    counterfactual.deductions.as_mut().unwrap().repayment = RepaymentClaim::None;
    for entry in &mut counterfactual.income {
        if let FederalIncome::Unemployment { data } = &mut entry.income {
            data.repayments.clear();
        }
    }
    let before = supported(&counterfactual);
    // The unchanged capital owner refigures original AGI15750 to10750.
    // Its otherwise-unused3000 now enters2026; the original zero balance
    // could not record whether that incoming loss belonged to this K-1.
    prior.agi = 1_075_000;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 300_000, long: 0, amt: None });
    assert_eq!(crate::extended_rules::capital_caps(&prior).short, 300_000);
    let mut worlds = vec![];
    for entity in [false, true] {
        let mut world = counterfactual.clone();
        let mut revised = prior.clone();
        revised.research_parts = entity.then(|| ResearchCapitalParts::Parts {
            sources: vec![ResearchCapitalPart { source, short: 300_000, long: 0 }],
        });
        world.screening.as_mut().unwrap().capital_history = vec![revised];
        assert!(world.valid_stored());
        let result = supported(&world);
        worlds.push(result);
    }
    assert_eq!(value(&worlds[0], "agi"), value(&worlds[1], "agi"));
    assert_eq!(value(&worlds[0], "taxable_income"), value(&worlds[1], "taxable_income"));
    assert_eq!(value(&worlds[0], "capital_gain"), -3000);
    assert_ne!(
        value(&worlds[0], "general_business_credit"),
        value(&worlds[1], "general_business_credit")
    );
    std::eprintln!(
        "qualified repayment consumer audit (combined method unsupported): identical stored bytes={}, method_state=Unsupported, before_AGI={}, before_credit={}, before_total_tax={}, refigured AGI={}, TI={}, personal_cap={}, entity_cap={}, personal_total_tax={}, entity_total_tax={}",
        bytes.len(),
        value(&before, "agi"),
        value(&before, "general_business_credit"),
        before.tax.unwrap() / 100,
        value(&worlds[0], "agi"),
        value(&worlds[0], "taxable_income"),
        value(&worlds[0], "general_business_credit"),
        value(&worlds[1], "general_business_credit"),
        worlds[0].tax.unwrap() / 100,
        worlds[1].tax.unwrap() / 100
    );
    if let Ok(folder) = std::env::var("TAX_UNIT16_REPAYMENT_CAPITAL_WITNESS_DIR") {
        let folder = std::path::Path::new(&folder);
        std::fs::create_dir_all(folder).unwrap();
        std::fs::write(folder.join("stored-draft.avro"), &bytes).unwrap();
        std::fs::write(folder.join("identical-other-world-stored-draft.avro"), same.to_avro())
            .unwrap();
    }
}

// Genuine R4 return sources, independent of a Form8379 claim or offset debt.
pub(in crate::tests) fn changed_filer_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0), wage(2, Owner::Spouse, 2_000_000, 0)];
    d.next_id = 3;
    let mut interest = income(savings_income(1000, 0));
    interest.owner = Owner::Spouse;
    add(&mut d, interest);
    let mut retained = carry(BusinessCredit::WorkOpportunity, 2025, 5000);
    retained.filer = CarryFiler::You;
    history(&mut d).carryforwards.push(retained);
    d
}

#[test]
fn unit16_r4_genuine_return_uses_owner_liabilities_without_an_injured_claim() {
    // TP MFS TI43900 -> table5023; SP TI4900 -> table493.
    // Ratio5023/5516 rounds to9106/10000, not an income ratio or half.
    // Joint TI48800 ->5363; each spouse's capacity uses that same joint line.
    for (filer, used, unused) in [
        (CarryFiler::You, 4884, 116),
        (CarryFiler::Spouse, 479, 4521),
        (CarryFiler::SameFilers, 5000, 0),
    ] {
        let mut d = changed_filer_draft();
        assert!(d.injured_spouse.is_none());
        history(&mut d).carryforwards[0].filer = filer;
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 81000);
        assert_eq!(value(&r, "taxable_income"), 48800);
        assert_eq!(value(&r, "form_3800_line_16"), 5363);
        assert_eq!(value(&r, "general_business_credit"), used);
        let prefix = if filer == CarryFiler::SameFilers {
            alloc::string::String::from("business_credit_work_opportunity_2025")
        } else {
            alloc::format!(
                "business_credit_work_opportunity_source_none_filer_{}_2025",
                filer as i32
            )
        };
        assert_eq!(value(&r, &alloc::format!("{prefix}_used")), used);
        assert_eq!(value(&r, &alloc::format!("{prefix}_to_next_year")), unused);
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    }
}

#[test]
fn unit16_r4_current_credit_does_not_feed_its_own_owner_ratio() {
    let mut d = changed_filer_draft();
    let before = supported(&d);
    own(&mut d, BusinessCredit::WorkOpportunity, 1000);
    let after = supported(&d);
    assert_eq!(
        value(&before, "business_credit_work_opportunity_source_none_filer_1_2025_used"),
        4884
    );
    assert_eq!(
        value(&after, "business_credit_work_opportunity_source_none_filer_1_2025_used"),
        4884
    );
    assert_eq!(value(&after, "business_credit_work_opportunity_current_year_used"), 479);
    assert_eq!(value(&after, "general_business_credit"), 5363);
    assert_eq!(value(&after, "form_3800_line_16"), value(&before, "form_3800_line_16"));
}

#[test]
fn unit16_r4_native_rmd_owner_tax_increases_only_that_owners_liability() {
    let mut d = changed_filer_draft();
    d.adjustments.as_mut().unwrap().required_distributions.push(rmd_record(1000, 0));
    let r = supported(&d);
    // Native5329 TP tax250/SP0; liabilities5273/493 give9145/10000.
    // The other-tax source changes the ratio, not joint section38 line16.
    assert_eq!(value(&r, "required_distribution_tax"), 250);
    assert_eq!(value(&r, "form_3800_line_16"), 5363);
    assert_eq!(value(&r, "general_business_credit"), 4904);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
        96
    );
}

#[test]
fn unit16_r4_unknown_joint_loss_owners_leave_only_changed_filer_credit_out() {
    let mut d = changed_filer_draft();
    let mut prior = capital_history();
    prior.owners = PriorReturnOwners::Joint;
    prior.status = FederalStatus::Joint;
    prior.agi = 10_000_000;
    prior.deductions = 3_000_000;
    prior.short = -600_000;
    prior.long = 0;
    prior.deducted_loss = 300_000;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 300_000, long: 0, amt: None });
    assert!(prior.injured_owners.is_none());
    d.screening.as_mut().unwrap().capital_history.push(prior);
    history(&mut d).carryforwards.push(carry(BusinessCredit::WorkOpportunity, 2024, 1000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 78000);
    assert_eq!(value(&r, "form_3800_line_16"), 5003);
    assert_eq!(value(&r, "general_business_credit"), 1000);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
        5000
    );
    assert!(noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
}

#[test]
fn unit16_r4_zero_liability_denominator_remains_unavailable() {
    let mut d = changed_filer_draft();
    d.income = vec![wage(1, Owner::Taxpayer, 0, 0), wage(2, Owner::Spouse, 0, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "form_3800_line_16"), 0);
    assert_eq!(value(&r, "general_business_credit"), 0);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
        5000
    );
    assert!(noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
}

pub(in crate::tests) fn changed_filer_profile(profile: &str) -> FederalDraft {
    let mut d = changed_filer_draft();
    match profile {
        "you" => {}
        "deferred-you" => d = unit16_deferred_source_caps::draft(CarryFiler::You),
        "deferred-spouse" => d = unit16_deferred_source_caps::draft(CarryFiler::Spouse),
        "hsa-you" | "hsa-spouse" => {
            d = r4_portfolio_draft();
            let mut entry = income(FederalIncome::MedicalDistribution {
                data: FederalMedicalDistribution {
                    origin: HealthDocumentOrigin::Form1099Sa,
                    gross: 100_000,
                    account: MedicalSaAccount::Hsa,
                    payment: MedicalSaPayment::Own {
                        nonmedical: vec![MedicalAccountWithdrawal {
                            date: 20260601,
                            amount: 100_000,
                        }],
                        medical: 0,
                        rollover: None,
                        disabled_since: None,
                    },
                },
            });
            entry.owner = if profile == "hsa-you" { Owner::Taxpayer } else { Owner::Spouse };
            add(&mut d, entry);
        }
        "clergy-source" => d = r4_clergy_source_draft(),
        "household-source" => {
            d = r4_portfolio_draft();
            let mut spouse = household_employer(3500);
            spouse.owner = Owner::Spouse;
            d.screening.as_mut().unwrap().household_employment =
                vec![spouse, household_employer(5000)];
        }
        "cf-source" => {
            d = r4_portfolio_draft();
            let mut nonfarm = cash_business(100);
            let FederalIncome::Business { data } = &mut nonfarm else { unreachable!() };
            let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else {
                unreachable!()
            };
            *receipts += 49;
            add(&mut d, income(nonfarm));
            add(&mut d, income(raised_farm(100, 0)));
            let farm_id = d.income.last().unwrap().id;
            // The existing linked source contributes49c to this actual farm.
            let FederalIncome::Unemployment { mut data } =
                government_unemployment(0, &[], 0).income
            else {
                unreachable!()
            };
            data.agriculture_payment = 49;
            data.agriculture_farm = Some(farm_id);
            add(&mut d, income(FederalIncome::Unemployment { data }));
        }
        "spouse" => history(&mut d).carryforwards[0].filer = CarryFiler::Spouse,
        "same" => history(&mut d).carryforwards[0].filer = CarryFiler::SameFilers,
        "current" => own(&mut d, BusinessCredit::WorkOpportunity, 1000),
        "rmd" => d.adjustments.as_mut().unwrap().required_distributions.push(rmd_record(1000, 0)),
        "short" | "long" | "qualified-election" | "capital-election" | "clean-28"
        | "specified-28" | "shared-election" | "qsbs" => {
            d = r4_portfolio_draft();
            match profile {
                "qsbs" => add_r4_qsbs_sales(&mut d),
                "short" | "long" => add(
                    &mut d,
                    income(ordinary_investment_sale(
                        10_000,
                        0,
                        if profile == "short" { GainTerm::Short } else { GainTerm::Long },
                    )),
                ),
                "qualified-election" | "capital-election" => {
                    let qualified = if profile == "qualified-election" { 10_000 } else { 0 };
                    let capital = 10_000 - qualified;
                    add(
                        &mut d,
                        income(FederalIncome::Dividend {
                            data: FederalDividend {
                                ordinary: qualified * 100,
                                qualified: qualified * 100,
                                capital_distributions: capital * 100,
                                ..Default::default()
                            },
                        }),
                    );
                    investment_borrowing(&mut d, 1, 0, 0);
                    let ded = d.deductions.as_mut().unwrap();
                    ded.choice = DeductionSelection::Standard;
                    ded.investment_interest.as_mut().unwrap().election =
                        InvestmentElection::Allocated {
                            capital: capital * 100,
                            dividends: qualified * 100,
                        };
                }
                "clean-28" | "specified-28" => {
                    d.income =
                        vec![wage(1, Owner::Taxpayer, 0, 0), wage(2, Owner::Spouse, 6_000_000, 0)];
                    add(&mut d, income(savings_income(250_000, 0)));
                    add(
                        &mut d,
                        income(FederalIncome::Dividend {
                            data: FederalDividend {
                                capital_distributions: 1_000_000,
                                collectible_distributions: 1_000_000,
                                ..Default::default()
                            },
                        }),
                    );
                    let record = &mut history(&mut d).carryforwards[0];
                    if profile == "clean-28" {
                        record.credit = BusinessCredit::CleanElectricity;
                    } else {
                        record.amount = 5_000_000;
                    }
                }
                "shared-election" => {
                    for owner in [Owner::Taxpayer, Owner::Spouse] {
                        let mut entry = income(FederalIncome::Dividend {
                            data: FederalDividend {
                                ordinary: 500_000,
                                qualified: 500_000,
                                ..Default::default()
                            },
                        });
                        entry.owner = owner;
                        add(&mut d, entry);
                    }
                    investment_borrowing(&mut d, 1, 0, 0);
                    let ded = d.deductions.as_mut().unwrap();
                    ded.choice = DeductionSelection::Standard;
                    ded.investment_interest.as_mut().unwrap().election =
                        InvestmentElection::Allocated { capital: 0, dividends: 500_000 };
                    history(&mut d).carryforwards.push(carry(
                        BusinessCredit::WorkOpportunity,
                        2024,
                        1000,
                    ));
                }
                _ => unreachable!(),
            }
        }
        "zero" => d.income = vec![wage(1, Owner::Taxpayer, 0, 0), wage(2, Owner::Spouse, 0, 0)],
        "missing" => {
            let mut prior = capital_history();
            prior.owners = PriorReturnOwners::Joint;
            prior.status = FederalStatus::Joint;
            prior.agi = 10_000_000;
            prior.deductions = 3_000_000;
            prior.short = -600_000;
            prior.long = 0;
            prior.deducted_loss = 300_000;
            prior.investment_parts =
                Some(InvestmentCapitalParts { short: 300_000, long: 0, amt: None });
            d.screening.as_mut().unwrap().capital_history.push(prior);
            history(&mut d).carryforwards.push(carry(BusinessCredit::WorkOpportunity, 2024, 1000));
        }
        _ => panic!("unknown actual changed-filer source profile"),
    }
    d
}

// New consumer controls for the delivered native source-owner corrections.
// Their original MFS Table/QDCGT figures are independent of the GBC engine.
fn r4_portfolio_draft() -> FederalDraft {
    let mut d = changed_filer_draft();
    d.income.retain(|e| matches!(e.income, FederalIncome::Wage { .. }));
    history(&mut d).carryforwards[0].amount = 1_000_000;
    d
}
// Reuse the established Unit41 source composition: ministerial W2 wages,
// parsonage and fee expenses restore133 to the actual ScheduleC source.
// No ScheduleF activity exists and no claim is supplied.
fn r4_clergy_source_draft() -> FederalDraft {
    let mut d = r4_portfolio_draft();
    let mut pay = wage(1, Owner::Taxpayer, 5_950_000, 0);
    if let FederalIncome::Wage { data } = &mut pay.income {
        data.coverage = W2Coverage::Ministerial;
        data.social_wages = 0;
        data.social_tax = 0;
        data.medicare_wages = 0;
        data.medicare_tax = 0;
    }
    d.income[0] = pay;
    let mut fees = cash_business(1600);
    if let FederalIncome::Business { data } = &mut fees {
        data.activity = BusinessActivity::Ordinary {
            returns: 0,
            receipts: 200_000,
            accounting: BusinessAccounting::Cash,
        };
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Ministerial costs".into() },
            40_000,
        )];
        data.ministerial = true;
    }
    add(&mut d, income(fees));
    d.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Parsonage {
                fair_rental_value: 2_400_000,
                utility_allowance: 750_000,
                utilities_paid: 700_000,
            },
            form_4361: false,
            employee_expenses: 0,
        },
    }];
    d
}

#[test]
fn unit16_r4_delivered_hsa_final_owner_income_and_tax_drive_caps() {
    for (profile, used) in [("hsa-you", 4996), ("hsa-spouse", 4713)] {
        let d = changed_filer_profile(profile);
        assert!(d.injured_spouse.is_none());
        let r = supported(&d);
        // Nonmedical1000 adds final income1000 and native HSA tax200.
        // TP income-tax5143+200 vsSP393 gives9315/10000. Moving the
        // actual source toSP givesTP5023 vsSP493+200, or8788/10000.
        assert_eq!(value(&r, "agi"), 81_000);
        assert_eq!(value(&r, "taxable_income"), 48_800);
        assert_eq!(value(&r, "form_3800_line_16"), 5363);
        assert_eq!(value(&r, "hsa_additional_tax"), 200);
        assert_eq!(value(&r, "general_business_credit"), used, "{profile}");
        assert_eq!(
            value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
            10_000 - used
        );
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
    }
}

#[test]
fn unit16_r4_actual_clergy_owner_preserves_native_cf_and_changed_carry() {
    let d = changed_filer_profile("clergy-source");
    assert!(d.injured_spouse.is_none());
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_business_income"), 1733);
    assert!(!r.lines.iter().any(|line| line.id == "schedule_1_farm_income"));
    assert_eq!(value(&r, "agi"), 75_191);
    assert_eq!(value(&r, "self_employment_tax"), 13_084);
    assert_eq!(value(&r, "qbi_deduction"), 400);
    // Native MFS TP4393+SE13084 vsSP393 gives9780/10000;
    // joint TI42591 table4613 gives4512 of the changed-filer carry.
    assert_eq!(value(&r, "taxable_income"), 42_591);
    assert_eq!(value(&r, "form_3800_line_16"), 4613);
    // The previously failed proposed4512 now receives the genuine taxable
    // housing source. Its original oracle and failed evidence are preserved.
    assert_eq!(value(&r, "general_business_credit"), 4512);
    assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut same = d.clone();
    history(&mut same).carryforwards[0].filer = CarryFiler::SameFilers;
    assert_eq!(value(&supported(&same), "general_business_credit"), 4613);
}

#[test]
fn unit16_r4_delivered_cf_source_rounding_conserves_actual_forms() {
    let d = changed_filer_profile("cf-source");
    assert!(d.injured_spouse.is_none());
    let r = supported(&d);
    // Two actual source groups10049c each round jointly to201. Existing
    // C-first projection retains C100/F101; no phantom or duplicate income.
    assert_eq!(value(&r, "schedule_1_business_income"), 100);
    assert_eq!(value(&r, "schedule_1_farm_income"), 101);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 80_201);
    assert_eq!(value(&r, "qbi_deduction"), 40);
    assert_eq!(value(&r, "taxable_income"), 47_961);
    assert_eq!(value(&r, "form_3800_line_16"), 5261);
    // TP MFS table5041/SP393 ->9277/10000; native joint cap5261 ->4881.
    assert_eq!(value(&r, "general_business_credit"), 4881);
    assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

#[test]
fn unit16_r4_delivered_short_long_owners_drive_actual_changed_filer_caps() {
    for (short, long, joint_limit, used) in [
        (10_000, 0, 6443, 6080),
        (0, 10_000, 5243, 4904),
        (10_000, -3000, 6083, 5704),
        (-3000, 10_000, 5243, 4877),
    ] {
        let mut d = r4_portfolio_draft();
        for (term, gain) in [(GainTerm::Short, short), (GainTerm::Long, long)] {
            if gain != 0 {
                add(&mut d, income(ordinary_investment_sale(gain.max(0), (-gain).max(0), term)));
            }
        }
        // TP native MFS tax6576/5691/5916/5241; SP393. Four-decimal
        // ratios9436/9354/9377/9302 apply to the actual joint section38 cap.
        assert!(d.injured_spouse.is_none());
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 80_000 + short + long);
        assert_eq!(value(&r, "form_3800_line_16"), joint_limit);
        assert_eq!(value(&r, "general_business_credit"), used);
        assert_eq!(
            value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
            10_000 - used
        );
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
    }
}
#[test]
fn unit16_r4_delivered_single_owner_elections_keep_the_actual_class_and_no_claim() {
    for (qualified, capital) in [(10_000, 0), (0, 10_000)] {
        let mut d = r4_portfolio_draft();
        add(
            &mut d,
            income(FederalIncome::Dividend {
                data: FederalDividend {
                    ordinary: qualified * 100,
                    qualified: qualified * 100,
                    capital_distributions: capital * 100,
                    ..Default::default()
                },
            }),
        );
        investment_borrowing(&mut d, 1, 0, 0);
        let ded = d.deductions.as_mut().unwrap();
        ded.choice = DeductionSelection::Standard;
        ded.investment_interest.as_mut().unwrap().election =
            InvestmentElection::Allocated { capital: capital * 100, dividends: qualified * 100 };
        assert!(d.injured_spouse.is_none());
        let r = supported(&d);
        // The actual sole eligible owner receives the elected10000. Both
        // classes now use ordinary MFS6576/SP393, ratio9436, joint6443.
        assert_eq!(value(&r, "agi"), 90_000);
        assert_eq!(value(&r, "taxable_income"), 57_800);
        assert_eq!(return_line(&r, "3a"), qualified * 100);
        assert_eq!(return_line(&r, "7a"), capital * 100);
        assert_eq!(value(&r, "form_3800_line_16"), 6443);
        assert_eq!(value(&r, "general_business_credit"), 6080);
        assert_eq!(
            value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
            3920
        );
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    }
}
#[test]
fn unit16_r4_delivered_special_pools_and_native_niit_preserve_source_cap() {
    for (collectible, unrecaptured, joint_limit, specified_limit, used, specified_used) in [
        (0, 0, 7720, 46276, 7079, 42435),
        (10_000, 0, 7520, 46951, 6909, 43134),
        (0, 10_000, 7620, 46951, 6997, 43115),
        (4000, 6000, 7580, 46951, 6962, 43124),
    ] {
        let mut d = r4_portfolio_draft();
        d.income = vec![wage(1, Owner::Taxpayer, 0, 0), wage(2, Owner::Spouse, 6_000_000, 0)];
        add(&mut d, income(savings_income(250_000, 0)));
        add(
            &mut d,
            income(FederalIncome::Dividend {
                data: FederalDividend {
                    capital_distributions: 1_000_000,
                    collectible_distributions: collectible * 100,
                    unrecaptured_distributions: unrecaptured * 100,
                    ..Default::default()
                },
            }),
        );
        // TP income tax52804/54104/53804/53924 plus actual NIIT2660;
        // SP5023. Ratios9170/9187/9183/9185 apply to the category's own
        // section38 limit: line16 for CleanElectricity, line29 for specified WOTC.
        // Each world uses its actual retained credit, not a changed oracle.
        for (credit, amount, expected) in [
            (BusinessCredit::CleanElectricity, 10_000, used),
            (BusinessCredit::WorkOpportunity, 50_000, specified_used),
        ] {
            let mut world = d.clone();
            let carry = &mut history(&mut world).carryforwards[0];
            carry.credit = credit;
            carry.amount = amount * 100;
            let r = supported(&world);
            assert!(world.injured_spouse.is_none());
            assert_eq!(value(&r, "agi"), 320_000);
            assert_eq!(value(&r, "net_investment_income_tax"), 2660);
            assert_eq!(value(&r, "form_3800_line_16"), joint_limit);
            assert_eq!(
                value(&r, "form_3800_line_29"),
                if credit == BusinessCredit::CleanElectricity {
                    specified_limit - used
                } else {
                    specified_limit
                }
            );
            assert_eq!(value(&r, "general_business_credit"), expected);
            assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        }
    }
}

#[test]
fn unit16_r4_shared_election_without_an_actual_split_retains_only_narrow_carry_warning() {
    let mut d = r4_portfolio_draft();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(FederalIncome::Dividend {
            data: FederalDividend { ordinary: 500_000, qualified: 500_000, ..Default::default() },
        });
        entry.owner = owner;
        add(&mut d, entry);
    }
    investment_borrowing(&mut d, 1, 0, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Standard;
    ded.investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 0, dividends: 500_000 };
    history(&mut d).carryforwards.push(carry(BusinessCredit::WorkOpportunity, 2024, 1000));
    let r = supported(&d);
    assert!(d.injured_spouse.is_none());
    assert_eq!(value(&r, "agi"), 90_000);
    assert_eq!(value(&r, "form_3800_line_16"), 5843);
    assert_eq!(value(&r, "general_business_credit"), 1000);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
        10_000
    );
    assert!(noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
}

// Mixed issuance cohorts retain the actual source-owner 28% classes delivered
// by the QSBS calculator; the fully excluded spouse lot contributes no tax.
fn add_r4_qsbs_sales(d: &mut FederalDraft) {
    for (owner, issued, gain) in [
        (Owner::Taxpayer, 20080602, 1_000_000),
        (Owner::Spouse, 20150302, 1_000_000),
        (Owner::Spouse, 20100927, 800_000),
    ] {
        let mut entry = income(FederalIncome::QualifiedSmallBusinessSale {
            data: QsbIssuerSale {
                issuer: "Synthetic R4 Source, Inc.".into(),
                prior_eligible_gain: 0,
                special: false,
                issuer_qualified: true,
                active_business: true,
                no_disqualification: true,
                zone_60_exclusion: Some(false),
                complete_history: true,
                lots: vec![QsbStockLot {
                    issued,
                    sold: 20260501,
                    basis: 1_000_000,
                    proceeds: 1_000_000 + gain,
                    zone_gain_through_2018: None,
                }],
                reporting: BasisReporting::ReportedToIrs,
                withholding: 0,
            },
        });
        entry.owner = owner;
        add(d, entry);
    }
}
#[test]
fn unit16_r4_delivered_qsbs_cohorts_use_actual_taxable_owner_sources() {
    let d = changed_filer_profile("qsbs");
    assert!(d.injured_spouse.is_none());
    let r = supported(&d);
    // Exclusion: 5000 +10000 +6000 =21000. Taxable owner pools5000/2000.
    // MFS TI48900/5900 gives native Table tax5623/593 (ordinary rate is
    // lower than the28% maximum); ratio9046 applies to joint line16=6083.
    assert_eq!(value(&r, "agi"), 87_000);
    assert_eq!(value(&r, "taxable_income"), 54_800);
    assert_eq!(value(&r, "rate_28_gain"), 7000);
    assert_eq!(value(&r, "form_3800_line_16"), 6083);
    assert_eq!(value(&r, "general_business_credit"), 5503);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_to_next_year"),
        4497
    );
    assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

#[path = "unit16_research_aggregation.rs"]
mod unit16_research_aggregation;

#[path = "unit16_deferred_source_caps.rs"]
mod unit16_deferred_source_caps;

#[path = "unit16_45q_equipment_origin.rs"]
mod unit16_45q_equipment_origin;
#[path = "unit16_election_transfer_client.rs"]
mod unit16_election_transfer_client;
#[path = "unit16_renewable_facility_origin.rs"]
mod unit16_renewable_facility_origin;

#[test]
fn unit16_original_printed_columns_conserve_mixed_vintages_passive_transfer_and_payment() {
    use crate::general_business_credit::{
        PassiveCreditCapacity, RehabilitationAllowance, TaxCapacity,
    };
    let mut prior = entry(BusinessCredit::DisabledAccess, 2024, 1000);
    prior.passive = None;
    let mut current_passive = entry(BusinessCredit::DifferentialWage, 2026, 400);
    current_passive.passive = Some(PassiveWorksheet::Other);
    let mut old_passive = entry(BusinessCredit::WorkOpportunity, 2006, 600);
    old_passive.passive = Some(PassiveWorksheet::Other);
    // Two completed sources: gross500 less sold200 leaves300, while a
    // distinct gross500 source elects payment500. The native merge owns
    // their common printed credit line, without treating sold cash as income.
    let mut transferred = entry(BusinessCredit::AdvancedManufacturingProduction, 2026, 300);
    transferred.transfer = -200;
    let mut paid = entry(BusinessCredit::AdvancedManufacturingProduction, 2026, 500);
    paid.elective = 500;
    let current = entry(BusinessCredit::DisabledAccess, 2026, 1000);
    let mut r = FederalResult::empty(0);
    let used = crate::general_business_credit::calculate(
        None,
        vec![prior, current_passive, old_passive, transferred, paid, current],
        &[],
        TaxCapacity {
            status: FederalStatus::Single,
            spouse_without_credits: false,
            regular_tax_before_credits: 500,
            alternative_minimum_tax: 0,
            tentative_minimum_tax: 0,
            foreign_tax_credit: 0,
            other_nonrefundable_credits: 0,
            passive: PassiveCreditCapacity {
                net_passive_income_tax: 200,
                remaining_allowance: 0,
                remaining_allowance_tax: 0,
                rehabilitation: RehabilitationAllowance::default(),
                active: RehabilitationAllowance::default(),
            },
            research_numerators: vec![],
            taxable_income: 100000,
            separate_ratios: None,
        },
        &mut r,
    );
    // One200 passive pool permits80 current and120 old. FIFO uses the
    // oldest allowed120 before380 of the2024 credit, exhausting tax capacity500.
    assert_eq!(used, (500, 500));
    for (line, expected) in [
        ("1e", [0, 1000, 0, 1000, 0, 0, 0]),
        ("1w", [400, 0, 0, 80, 0, 0, 0]),
        ("1b", [0, 1000, -200, 800, 500, 0, 500]),
        ("2", [400, 2000, -200, 1880, 500, 0, 500]),
        ("5", [0; 7]),
        ("6", [400, 2000, -200, 1880, 500, 0, 500]),
    ] {
        for (c, expected) in ['d', 'e', 'f', 'g', 'h', 'i', 'j'].into_iter().zip(expected) {
            assert_eq!(column(&r, 3, line, c), expected, "PartIII {line}({c})");
        }
    }
    for (line, expected) in [
        ("1e", [0, 0, 1000, 380, 0, 620]),
        ("2j", [600, 120, 0, 120, 0, 0]),
        ("5", [0, 0, 1000, 380, 0, 620]),
        ("6", [600, 120, 0, 120, 0, 0]),
        ("7", [600, 120, 1000, 500, 0, 620]),
    ] {
        for (c, expected) in ['d', 'e', 'f', 'g', 'h', 'i'].into_iter().zip(expected) {
            assert_eq!(column(&r, 4, line, c), expected, "PartIV {line}({c})");
        }
    }
    assert_eq!(value(&r, "form_3800_line_17"), 500);
    assert_eq!(value(&r, "form_3800_line_26"), 0);
    for line in 18..=25 {
        assert!(!r.lines.iter().any(|row| row.id == alloc::format!("form_3800_line_{line}")));
    }
    for (key, expected) in [
        // Only the120 allowed into Form3800 is used. The480 still
        // suspended by Form8582CR is separately retained, not column(i).
        ("business_credit_work_opportunity_2006_expired", 0),
        ("business_credit_work_opportunity_2006_passive_to_next_year", 480),
        ("business_credit_disabled_access_2024_to_next_year", 620),
        ("business_credit_disabled_access_current_year_to_next_year", 1000),
        ("business_credit_differential_wage_current_year_passive_to_next_year", 320),
        ("business_credit_differential_wage_current_year_to_next_year", 80),
        ("business_credit_advanced_manufacturing_production_current_year_to_next_year", 300),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
}
