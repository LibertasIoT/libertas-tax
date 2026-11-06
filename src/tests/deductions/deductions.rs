// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
use crate::tests::support::assert::line_amount as cents;

#[test]
fn educator_excess_is_itemized_once_and_only_when_itemizing() {
    let mut d = complete(FederalStatus::Single);
    let a = d.adjustments.as_mut().unwrap();
    a.set_educator_taxpayer(100_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 350);
    assert_eq!(value(&r, "deduction"), 16100);
    let v = d.deductions.as_mut().unwrap();
    itemized(v).personal_property_taxes = 1_000_000;
    fixture_mortgage_interest(v, 1_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 20650); // 20,000 plus only the remaining 650.
    assert_eq!(value(&r, "agi"), 59650);
    d.deductions.as_mut().unwrap().itemize_override = None;
    // 2026-11-02: automatic selection retains the larger deduction from these same facts.
    assert_eq!(value(&supported(&d), "deduction"), 20650);
}
#[test]
fn additional_deduction_facts_survive_restart_and_invalid_amounts_are_rejected() {
    let mut d = complete(FederalStatus::Single);
    let FederalIncome::Wage { data: w } = &mut d.income[0].income else { panic!() };
    let mut w = edit_wage(w);
    w.work_deductions = work_deductions(0, 500_000);
    drop(w);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![VehicleLoanSource {
        id: 0,
        assigned: crate::DocumentRecipient::Taxpayer,
        owner_source: Some(crate::OwnerSource::Added),
        joint_share: None,
        business: 0,
        document: vehicle_paper(FederalVehicleInterest {
            lender: crate::PrintedParty::from_name("Synthetic car".into()),
            borrower: Default::default(),
            interest: 100_000,
            refund: 0,
            original_use: true,
            assembled_in_us: true,
        }),
    }];
    assert_eq!(supported(&restart(d.clone()).draft), supported(&d));
}

#[test]
fn itemized_high_income_limitation_adds_back_deduction_when_testing_section_68() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 65_000_000, 0);
    let v = d.deductions.as_mut().unwrap();
    v.itemize_override = Some(ItemizeOverride::Itemize);
    fixture_mortgage_interest(v, 3_000_000);
    // Taxable income falls below 640,600, but the section 68 trigger adds itemized
    // deductions back. A result without this worksheet would overstate the deduction.
    let r = supported(&d);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 508);
    assert_eq!(return_line(&r, "12e"), 2_949_200);
    assert_eq!(value(&r, "taxable_income"), 620508);
}

#[test]
fn automatic_deductions_compare_amt_after_clearing_the_override() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises = vec![iso_exercise("ISO", 2000.0, 1000, 11000)];
    d.deductions.as_mut().unwrap().cash_charity = 1_000_000;
    d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    let elected = supported(&d);
    assert_eq!(return_line(&elected, "12e"), 970_000);
    assert_eq!(elected.tax, Some(4_165_200));
    d.deductions.as_mut().unwrap().itemize_override = None;
    let cleared = supported(&d);
    // 2026-11-02: no override reuses the existing lower-final-tax AMT comparison.
    assert_eq!(return_line(&cleared, "12e"), 970_000);
    assert_eq!(cleared.tax, Some(4_165_200));
}
#[test]
fn conversion_remains_in_traditional_deduction_magi() {
    let mut d = ira_contribution_distribution(80000, Answer::Yes);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    data.ordinary_distribution = Some(conversion(10000, true));
    drop(data);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1880);
    assert_eq!(value(&r, "roth_contribution_magi"), 80000);
    assert_eq!(value(&r, "roth_conversion_taxable"), 8500);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn personal_prizes_include_noncash_and_withholding_without_earned_income() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(prize(PrizeDisposition::Received)));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_prizes"), 3500);
    assert_eq!(value(&r, "agi"), 3500);
    assert_eq!(return_line(&r, "8"), 350000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(r.refund, Some(10000));
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(value(&supported(&d), "deduction"), 1350);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    unsupported(&d, "Investment tax");
}

#[test]
fn refused_and_direct_charity_prizes_are_excluded_without_a_second_deduction() {
    let mut d = complete(FederalStatus::Single);
    for disposition in
        [PrizeDisposition::Refused, PrizeDisposition::DirectCharity { eligible: true }]
    {
        d.income.truncate(1);
        add(&mut d, income(prize(disposition)));
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_prizes"), 0);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(value(&r, "deduction"), 16100);
    }
    // No is ordinary receipt: the exclusion is not applied.
    d.income[1].income = prize(PrizeDisposition::DirectCharity { eligible: false });
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_prizes"), 3500);
    assert_eq!(value(&r, "agi"), 63500);
}
#[test]
fn final_filer_standard_deduction_not_prorated_and_no_premature_senior_addition() {
    let mut d = complete(FederalStatus::Single);
    let p = d.people.as_mut().unwrap();
    p.taxpayer.birth_date = 19610615;
    p.taxpayer.died = death(20260110);
    assert_eq!(
        crate::federal_rules::standard_deduction(d.setup.as_ref().unwrap(), p, 60000, false),
        16100
    );
    p.taxpayer.died = death(20260614);
    assert_eq!(
        crate::federal_rules::standard_deduction(d.setup.as_ref().unwrap(), p, 60000, false),
        18150
    );
    assert_eq!(restart(d.clone()).draft, d);
    // Pub. 559: the full standard deduction, computed with a notice that the
    // special death adjustments are left out.
    assert!(noted(&supported(&d), "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"));
}

#[test]
fn bonus_equipment_deduction_flows_through_income_se_tax_and_qbi() {
    let d = business_with_bonus(100000, bonus_equipment(20000, 50));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 10000);
    let mut ordinary = d.clone();
    let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
    data.assets.clear();
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
        1000000,
    ));
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(r.tax, expected.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section197_later_costs_use_remaining_months_and_immediate_expired_deduction() {
    for (acquired, date, added, extra, remaining) in [
        (20160101, 20260731, 540000, 60000, 480000), // 54 months left, six in 2026.
        (20160101, 20250701, 660000, 120000, 480000), // Prior addition, 66-month schedule.
        (20110701, 20260630, 12345, 12345, 0),       // Original final month.
        (20110701, 20260701, 12345, 12345, 0),       // Immediately after original period.
        (20110101, 20261231, 12345, 12345, 0),       // Long-expired original period.
        (20100101, 20250101, 12345, 0, 0),           // Prior immediate deduction never repeats.
    ] {
        let mut d = intangible_business(acquired, 19900101);
        let baseline = supported(&d);
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        let asset = &mut data.intangibles[0];
        let deduction = asset.deduction();
        let basis = asset.remaining();
        asset.additions.push(IntangibleCost { amount: added, date });
        // Actual prior returns include earlier additions; those amounts are source facts.
        let earlier_addition = match date {
            20250701 => 60_000,
            20250101 => 12_345,
            _ => 0,
        };
        if earlier_addition != 0 {
            for source in [&mut asset.prior_claimed.regular, &mut asset.prior_claimed.amt] {
                let crate::SourceCents::Reported { amount } = source else { panic!() };
                *amount += earlier_addition;
            }
        }
        assert_eq!(asset.deduction(), deduction + extra);
        assert_eq!(asset.remaining(), basis + remaining);
        let r = supported(&d);
        assert!(value(&r, "agi") <= value(&baseline, "agi"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn automatic_charity_selection_keeps_caps_and_carryovers_for_the_larger_gift() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.set_cash_charity(10000000);
    let r = supported(&d);
    // 2026-11-02: automatic itemizing deducts 36,000 less the 300 floor;
    // the floor joins the 64,000 percentage-limit excess in the current carryover.
    assert_eq!(value(&r, "taxable_income"), 24300);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 64300);
    assert_eq!(value(&r, "charity_cash_sixty_floor"), 300);
    assert!(r.lines.iter().any(|line| line.id == "charity_floor_not_carried"));
    assert_eq!(value(&r, "charity_floor_not_carried"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    // The amount actually deductible, not the source gift, uses both caps.
    d.income = vec![wage(1, Owner::Taxpayer, 100000, 0)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "12f"), 60000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 99400);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    d.income.clear();
    let r = supported(&d);
    assert_eq!(return_line(&r, "12f"), 0);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 100000);
}

#[test]
fn cash_charity_percentage_excess_carries_floor_only_when_limit_exceeded() {
    for (gift, deduction, carry, lost_floor) in
        [(50000, 49500, 0, 500), (60000, 59500, 0, 500), (61000, 59500, 1500, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, 10000000, 0)];
        let ded = d.deductions.as_mut().unwrap();
        ded.itemize_override = Some(ItemizeOverride::Itemize);
        ded.set_cash_charity(gift * 100);
        let r = supported(&d);
        assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), deduction);
        assert_eq!(value(&r, "charity_return_year_carryforward"), carry);
        assert_eq!(value(&r, "charity_floor_not_carried"), lost_floor);
        assert_eq!(restart(d.clone()).draft, d);
        if carry > 0 {
            d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1000)]);
            supported(&d);
        }
    }
}

#[test]
fn mortgage_certificate_carryforwards_use_current_then_oldest_and_expire_2023() {
    let mut d = complete(FederalStatus::Single);
    let mut credit = mortgage_certificate_credit(2000, 10000, 1_000_000);
    credit.carry_three_years_prior = 200_000;
    credit.carry_two_years_prior = 200_000;
    credit.carry_prior_year = 200_000;
    d.credits.as_mut().unwrap().mortgage_credit = Some(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_credit"), 5023);
    assert_eq!(value(&r, "mortgage_credit_carry_2026_to_2027"), 0);
    assert_eq!(value(&r, "mortgage_credit_expired_2023"), 0);
    assert_eq!(value(&r, "mortgage_credit_carry_2024_to_2027"), 977);
    assert_eq!(value(&r, "mortgage_credit_carry_2025_to_2027"), 2000);

    d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current = None;
    d.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
    let low_tax = supported(&d);
    assert_eq!(value(&low_tax, "income_tax"), 793);
    assert_eq!(value(&low_tax, "mortgage_interest_credit"), 793);
    assert_eq!(value(&low_tax, "mortgage_credit_expired_2023"), 1207);
    assert_eq!(value(&low_tax, "mortgage_credit_carry_2024_to_2027"), 2000);
    assert_eq!(value(&low_tax, "mortgage_credit_carry_2025_to_2027"), 2000);
    set_mortgage_subsidy(&mut d, MortgageSubsidyDisposition::Unreviewed);
    unsupported(&d, "Mortgage subsidy recapture");
}

#[test]
fn final_guaranteed_life_deduction_keeps_net_cost_after_guarantee_expires() {
    let mut a = fixed_annuity();
    a.started = 20000101;
    a.cost = 2105300;
    a.periodic = 10000;
    a.received = 6;
    a.recovered = 1000000;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 2040000 };
    assert!(!a.ends_at_death(20161231));
    assert!(a.ends_at_death(20170101));
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        600,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: a.started,
        cost: a.cost,
        recovered: a.recovered,
        refunds: 0,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_loss_2"), 10598); // $21,053 - $10,000 - $454.80.
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::Retirement { data } = &mut d.income[1].income {
        let mut data = edit_retirement(data);
        if let Some(RetirementTaxable::GeneralAnnuity { data }) = &mut data.taxable {
            data.started = 20100101;
        }
    }
    d.deductions.as_mut().unwrap().annuity_losses[0].started = 20100101;
    unsupported(&d, "Annuity loss"); // Guaranteed installments continue beyond death.
}

#[test]
fn ordinary_used_goods_share_aggregate_2026_charity_floor_with_cash() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(used_goods(50_000));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let goods_only = supported(&d);
    assert_eq!(value(&goods_only, "charity_cash_sixty_current_deduction"), 0);
    assert_eq!(value(&goods_only, "charity_property_fifty_current_deduction"), 200);
    assert_eq!(value(&goods_only, "charity_floor_not_carried"), 300);
    assert_eq!(return_line(&goods_only, "12e"), 20_000);

    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    let combined = supported(&d);
    assert_eq!(value(&combined, "charity_cash_sixty_current_deduction"), 1_000);
    assert_eq!(value(&combined, "charity_property_fifty_current_deduction"), 200);
    assert_eq!(return_line(&combined, "12e"), 120_000);
    assert_eq!(estimate(&restart(d.clone()).draft), combined);

    d.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&d);
    d.deductions.as_mut().unwrap().property_gifts.clear();
    let without_goods = supported(&d);
    assert_eq!(standard.tax, without_goods.tax);
    assert_eq!(value(&standard, "taxable_income"), value(&without_goods, "taxable_income"));
}

#[test]
fn donated_car_and_used_goods_share_one_noncash_limit_and_floor() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut goods = used_goods(500_000);
    goods.basis = 1_000_000;
    ded.property_gifts.push(goods);
    push_gift(
        ded,
        donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 1_500_000 }),
    );
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let combined = supported(&d);
    assert_eq!(value(&combined, "charity_property_fifty_current_deduction"), 19_700);
    assert_eq!(value(&combined, "form_8283_section_a_value"), 20_000);
    assert_eq!(value(&combined, "charity_property_fifty_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), combined);
}

#[test]
fn substantial_away_service_and_required_uniform_are_cash_class_volunteer_costs() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    let mut away = volunteer_travel(VolunteerCarCost::Mileage { miles: 1 }, 0);
    away.service = "Full-day volunteer conference representative".into();
    away.kind =
        VolunteerExpenseKind::AwayTravel { transportation: 12_000, lodging: 18_000, meals: 4_000 };
    let mut uniform = volunteer_travel(VolunteerCarCost::Mileage { miles: 1 }, 0);
    uniform.service = "Volunteer hospital nurse aide".into();
    uniform.kind = VolunteerExpenseKind::Uniform { purchase: 8_000, upkeep: 2_000 };
    ded.volunteer_expenses.extend([away, uniform]);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&d);
    assert_eq!(value(&result, "charity_volunteer_1_expense"), 340);
    assert_eq!(value(&result, "charity_volunteer_2_expense"), 100);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 1_140);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = false;
    assert!(noted(&supported(&d), "WARN_VOLUNTEER_EXPENSE_LEFT_OUT"));
    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = true;
    d.deductions.as_mut().unwrap().volunteer_expenses[0].kind =
        VolunteerExpenseKind::AwayTravel { transportation: 0, lodging: 0, meals: 0 };
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().volunteer_expenses[0].kind =
        VolunteerExpenseKind::Uniform { purchase: 0, upkeep: 0 };
    assert!(!d.valid_stored());
}

#[test]
fn current_cash_uses_sixty_percent_room_before_older_cash_and_whaling() {
    let mut draft =
        with_current_direct_cash(cash_carry_with_whaling(10_000_000, 1_000_000, 30_000), 4_000_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 40_000);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_floor_not_carried"), 500);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 19_500);
    assert_eq!(value(&itemized, "charity_whaling_deduction"), 0);
    assert_eq!(value(&itemized, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(value(&itemized, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(return_line(&itemized, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&automatic, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn whaling_captain_expenses_use_itemized_thirty_percent_class_and_restart() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    let ded = draft.deductions.as_mut().unwrap();
    ded.whaling_captains.push(whaling_claim(vec![whaling_expense(
        WhalingExpenseKind::CrewAndProvisions,
        1_000_000,
    )]));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let itemized_result = supported(&draft);
    assert_eq!(value(&itemized_result, "charity_whaling_statutory_claim"), 10_000);
    assert_eq!(value(&itemized_result, "charity_whaling_deduction"), 5_900);
    assert_eq!(value(&itemized_result, "charity_whaling_return_year_carryforward"), 4_100);
    assert_eq!(value(&itemized_result, "charity_return_year_carryforward"), 4_100);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized_result);

    draft.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&draft);
    assert_eq!(value(&standard, "charity_whaling_deduction"), 0);
    assert_eq!(value(&standard, "charity_whaling_return_year_carryforward"), 4_000);
    assert_eq!(value(&standard, "charity_return_year_carryforward"), 4_000);
}

#[test]
fn whaling_source_must_be_valid_and_confirmed() {
    let mut draft = complete(FederalStatus::Single);
    draft
        .deductions
        .as_mut()
        .unwrap()
        .whaling_captains
        .push(whaling_claim(vec![whaling_expense(WhalingExpenseKind::WeaponsAndGear, 100_000)]));
    assert!(draft.valid_stored());
    draft.deductions.as_mut().unwrap().whaling_captains[0].eligibility_and_records_confirmed =
        false;
    assert!(noted(&supported(&draft), "WARN_WHALING_EXPENSES_LEFT_OUT"));
}

#[test]
fn schedule_a_other_lines_are_refused_by_line() {
    let mut d = complete(FederalStatus::Single);
    let v = d.deductions.as_mut().unwrap();
    itemized(v).personal_property_taxes = 1_000_000;
    fixture_mortgage_interest(v, 500_000);
    let baseline = supported(&d);

    // Unit A15 through the situations list.

    // An empty list keeps today's result.
    assert_eq!(estimate(&d), baseline);
}

// Matrix row 18, Schedule A (2026 draft renumbering per the form map: 1-4
// medical, 5a-5e and 7 taxes, 8a-10 interest, 11-15 gifts, 16 casualty,
// 17a-17z other, 18 total, 19 election; unit A15 recorded the renumbering).
//
// MFJ, spouses born 1972 and 1974, one W-2 (box 1 $160,000, box 2 $22,000
// withheld; boxes 3-6 follow `wage()`'s standard synthetic FICA amounts),
// medical expenses $20,000, state income tax paid (income-tax election)
// $9,500, real-estate tax $8,500, home mortgage interest $14,000 on
// acquisition debt within the limit, cash gifts to eligible public
// charities $7,000. Every line below is computed by hand from Pub 526/936,
// i1040sca and this App's own `salt_cap_current_year` and charity-floor constants
// in federal_rules.rs; none is copied from the App's own output.
//
// Schedule A (2026 draft numbering):
//   1   Medical expenses                                       20,000
//   2   AGI (wages only; no adjustments)                      160,000
//   3   7.5% AGI floor (Pub 502)                                12,000
//   4   Medical deduction = positive(1 - 3)                      8,000
//   5a  State income tax election, paid                         9,500
//   5b/5c Real-estate tax (one combined field for both)          8,500
//   5d  Sum of 5a + 5b/5c                                       18,000
//   5e  min(5d, salt_cap_current_year(Joint, magi=agi))                 18,000
//       salt_cap_current_year: full = max(10,000, 40,400 - 30% *
//       positive(160,000-505,000)) = max(10,000, 40,400-0) = 40,400
//       (AGI is below the $505,000 phase-down start); the $40,400 cap
//       is not binding here.
//   7   5e + 6 (line 6 is out of scope, always 0)                18,000
//   8e  mortgage_deductible_interest = mortgage interest less
//       any Form 8396 credit (none here)                        14,000
//   10  8e + 9 (line 9, investment interest, is 0)               14,000
//   11  Cash gifts to eligible public charities entered           7,000
//   13  Charitable Contribution Limitation Worksheet:
//       cash_charity_allowed = min(7,000, 60% * 160,000=96,000) = 7,000
//       charity_allowed = 7,000. The App applies its own 2026 itemizer
//       floor (P.L. 119-21, `charity_floor` in federal_rules.rs) of 0.5%
//       of AGI to the itemized charitable deduction:
//       charity_floor = min(7,000, 0.5% * 160,000=800) = 800.
//       No percentage-limit excess (7,000 <= the 60% cap), so the floor
//       is simply lost, not carried.
//       line 13 = charity_allowed - charity_floor = 6,200
//   14  Prior-year carryover                                         0
//   15  Charity deduction (13 + 14)                                6,200
//   16  Casualty                                                       0
//   17a-17z Other (gambling/repayment/annuity/educator)                0
//   18  Total itemized deductions (sum of 4, 7, 10, 15, 16, 17z),
//       reduced by the section-68-style 2026 high-income limitation
//       (0 here: taxable income is far below the top-bracket threshold
//       that test uses)                                          46,200
//       (8,000 + 18,000 + 14,000 + 6,200)
//
// Form 1040:
//   11 (11a/11b) AGI                                            160,000
//   12e Itemized deduction: 46,200 exceeds the $32,200 joint standard
//       deduction plus the $2,000 non-itemizer cash floor (34,200), so
//       the Automatic route itemizes                              46,200
//   15  Taxable income = 160,000 - 46,200                        113,800
//   16  Regular tax, 2026 MFJ rate schedule ([24,800; 100,800;
//       211,400; ...] at [10; 12; 22; ...]%; taxable income is above
//       $100,000 so the exact schedule applies, not the Tax Table):
//       10% * 24,800 = 2,480
//       + 12% * (100,800-24,800=76,000) = 9,120
//       + 22% * (113,800-100,800=13,000) = 2,860
//       = 14,460
//   24a Total tax: no AMT (AMTI ~= 113,800 + 18,000 SALT addback =
//       131,800, below the 2026 joint AMT exemption of $140,200), no
//       credits, no additional taxes                               14,460
//   25a W-2 federal income tax withheld                            22,000
//   33  Total payments and refundable credits                      22,000
//   34  Overpayment (refund) = 22,000 - 14,460                       7,540
#[test]
fn itemized_return_matches_hand_computation() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19720615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19740615;
    d.income = vec![wage(1, Owner::Taxpayer, 16_000_000, 2_200_000)];
    let v = d.deductions.as_mut().unwrap();
    itemized(v).medical = 2_000_000;
    itemized(v).personal_tax = PersonalTaxElection::Income {
        paid: 950_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    itemized(v).personal_property_taxes = 850_000;
    fixture_mortgage_interest(v, 1_400_000);
    v.set_cash_charity(700_000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 160_000);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 6_200);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&r, "charity_floor_not_carried"), 800);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 0);
    assert_eq!(value(&r, "deduction"), 46_200);
    assert_eq!(value(&r, "taxable_income"), 113_800);
    assert_eq!(value(&r, "income_tax"), 14_460);
    assert_eq!(return_line(&r, "11a / 11b"), 16_000_000);
    assert_eq!(return_line(&r, "12e"), 4_620_000);
    assert_eq!(return_line(&r, "15"), 11_380_000);
    assert_eq!(return_line(&r, "16"), 1_446_000);
    assert_eq!(return_line(&r, "24a"), 1_446_000);
    assert_eq!(return_line(&r, "25a"), 2_200_000);
    assert_eq!(return_line(&r, "33"), 2_200_000);
    assert_eq!(return_line(&r, "34"), 754_000);
    assert_eq!(r.tax, Some(1_446_000));
    assert_eq!(r.refund, Some(754_000));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary: Schedule A line 5e's `salt_cap_current_year` phase-down (federal_rules.rs).
// For Joint: full = max(10,000, 40,400 - 30% * positive(magi-505,000)); with
// no foreign income magi = AGI. The SALT election is forced Itemized (not
// Automatic) so the deduction line directly reflects the capped SALT amount,
// since every other itemized fact is left at zero.
#[test]
fn salt_cap_phases_down_with_income_to_a_10000_floor() {
    for (wages, agi, salt_paid, cap) in [
        // Mid phase-down: 40,400 - 30%*(550,000-505,000=45,000)=13,500
        // -> cap 26,900; the $35,000 paid exceeds it, so the cap binds.
        (55_000_000, 550_000, 3_500_000, 26_900),
        // Past the phase-down range: 40,400 - 30%*(700,000-505,000=
        // 195,000)=58,500 is negative, floored at the statutory $10,000
        // minimum; the $35,000 paid still exceeds the floor.
        (70_000_000, 700_000, 3_500_000, 10_000),
    ] {
        let mut d = complete(FederalStatus::Joint);
        d.income = vec![wage(1, Owner::Taxpayer, wages, 0)];
        let v = d.deductions.as_mut().unwrap();
        v.itemize_override = Some(ItemizeOverride::Itemize);
        itemized(v).personal_tax = PersonalTaxElection::Income {
            paid: salt_paid,
            allocation: TaxIncomeAllocation::CurrentReturn,
        };
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "itemized_deduction_reduction"), 0);
        // The forced-Itemized deduction is exactly the capped SALT amount:
        // every other itemized fact is zero.
        assert_eq!(value(&r, "deduction"), cap);
    }
}

// Boundary: Pub 526's 60%-of-AGI ceiling on cash gifts to public charities.
// AGI $160,000 (as in the main fixture); cap = 60% * 160,000 = $96,000.
// A $100,000 gift exceeds the cap by $4,000, which carries forward under
// section 170(b)(1)(G) (five years, through 2031), together with this
// year's 0.5%-of-AGI itemizer-floor share (the floor is not lost when a
// percentage-limit excess exists).
//   cash_charity_allowed = min(100,000, 96,000) = 96,000
//   charity_floor = min(96,000, 0.5% * 160,000 = 800) = 800
//   charity_cash_sixty_current_deduction (Schedule A line 13/15) = 96,000 - 800 = 95,200
//   excess above the 60% cap = 100,000 - 96,000 = 4,000
//   carryforward = excess + this year's floor share (800, since an excess
//   exists) = 4,800
#[test]
fn cash_gifts_above_60_percent_of_agi_carry_forward() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 16_000_000, 0)];
    let v = d.deductions.as_mut().unwrap();
    v.itemize_override = Some(ItemizeOverride::Itemize);
    v.set_cash_charity(10_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 160_000);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 95_200);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 4_800);
    assert_eq!(value(&r, "charity_floor_not_carried"), 0);
}

// Boundary: Schedule A line 19's election. Automatic takes the standard
// deduction when itemized expenses are smaller (2026 MFJ standard:
// $32,200); the explicit Itemized election keeps the smaller itemized
// amount instead.
#[test]
fn itemizing_below_the_standard_deduction_needs_the_election() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 0)];
    let v = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(v, 2_000_000);
    // Itemized total (mortgage interest only) = $20,000, below the
    // $32,200 joint standard deduction: Automatic takes the standard.
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 32_200);
    // The explicit line 19 election keeps the smaller itemized amount.
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 20_000);
}

// Pub 936 Table 1 (unit C20a). Helpers build one Form 1098 record; amounts in cents.
fn mortgage_loan(incurred: u32, interest: i64, average: MortgageAverageBalance) -> MortgageLoan {
    MortgageLoan {
        printed: None,
        id: 1,
        property: "Synthetic home".into(),
        lender: MortgageLender::Institution,
        interest,
        outstanding_principal: 0,
        incurred,
        refund_of_interest: MortgageInterestRefund::No,
        mortgage_insurance_premiums: 0,
        points: MortgagePoints::No,
        details: MortgageLoanDetails::FirstLender(MortgageLedger {
            home: MortgageHome::Main,
            purpose: MortgageUse::Home,
            average,
        }),
    }
}
fn statements(average_balance: i64) -> MortgageAverageBalance {
    MortgageAverageBalance::Statements { average_balance }
}
fn itemize_mortgages(d: &mut FederalDraft, loans: Vec<MortgageLoan>) {
    let v = d.deductions.as_mut().unwrap();
    let facts = itemized(v).clone();
    v.mortgage = mortgage_loan_sources(identified_mortgages(loans));
    {
        v.itemized = facts;
        v.itemize_override = Some(ItemizeOverride::Itemize);
    };
}

// Joint return, wages $300,000, real-estate taxes $10,000 (within the $40,400 SALT
// cap at this income), itemized explicitly.
//   Loan A: incurred 2015, average balance $600,000, interest $24,000.
//   Loan B: incurred 2022, average balance $500,000, interest $22,500.
// Pub 936 Table 1:
//   1  grandfathered (before 10/14/1987)                         0
//   2  10/14/1987 - 12/15/2017 acquisition debt            600,000
//   3  $1,000,000 (joint)                                1,000,000
//   4  larger of 1 or 3                                  1,000,000
//   5  1 + 2                                               600,000
//   6  smaller of 4 or 5                                   600,000
//      (later debt exists and 600,000 < 750,000, so continue)
//   7  acquisition debt after 12/15/2017                   500,000
//   8  $750,000 (joint)                                    750,000
//   9  larger of 6 or 8                                    750,000
//  10  6 + 7                                             1,100,000
//  11  smaller of 9 or 10: qualified loan limit            750,000
//  12  total average balance                             1,100,000
//  13  total interest 24,000 + 22,500                       46,500
//  14  750,000 / 1,100,000 = 0.6818... rounded to             0.682
//  15  46,500 x 0.682 = deductible interest                 31,713
//  16  46,500 - 31,713 = not deductible                     14,787
// Schedule A: 8a = 8e = 31,713; line 5 taxes 10,000; line 17 = 41,713.
// Form 1040 12e = 41,713; line 15 = 300,000 - 41,713 = 258,287.
// Line 16, 2026 MFJ schedule: 2,480 + 9,120 + 22% x 110,600 (24,332)
//   + 24% x (258,287 - 211,400 = 46,887) (11,252.88) = 47,184.88 -> 47,185.
#[test]
fn mortgage_interest_limit_uses_average_balance_worksheet_across_two_loans() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 30_000_000, 0)];
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 1_000_000;
    itemize_mortgages(
        &mut d,
        vec![
            mortgage_loan(20150601, 2_400_000, statements(60_000_000)),
            mortgage_loan(20220301, 2_250_000, statements(50_000_000)),
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_average_balance"), 1_100_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 31_713);
    assert_eq!(value(&r, "mortgage_interest_8b"), 0);
    assert_eq!(value(&r, "mortgage_interest_8e"), 31_713);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 14_787);
    assert_eq!(value(&r, "deduction"), 41_713);
    assert_eq!(return_line(&r, "12e"), 4_171_300);
    assert_eq!(return_line(&r, "15"), 25_828_700);
    assert_eq!(return_line(&r, "16"), 4_718_500);
    assert_eq!(restart(d.clone()).draft, d);
}

// The row-18 itemized fixture (itemized_return_matches_hand_computation) with its
// $14,000 of interest entered as one 1098 by the average of first and last
// balances: box 2 $300,000 on January 1, $290,000 on December 31, so the average
// is (300,000 + 290,000) / 2 = 295,000, below the $750,000 joint limit (Table 1
// line 11 = 750,000 >= line 12 = 295,000: stop, all interest deductible).
// Schedule A 8a = 14,000 and every total matches that fixture: 12e 46,200,
// line 15 113,800, line 16 14,460, refund 7,540.
#[test]
fn single_1098_within_the_limit_is_fully_deductible() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19720615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19740615;
    d.income = vec![wage(1, Owner::Taxpayer, 16_000_000, 2_200_000)];
    let v = d.deductions.as_mut().unwrap();
    itemized(v).medical = 2_000_000;
    itemized(v).personal_tax = PersonalTaxElection::Income {
        paid: 950_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    itemized(v).personal_property_taxes = 850_000;
    let mut loan = mortgage_loan(
        20190815,
        1_400_000,
        MortgageAverageBalance::FirstAndLast { last_balance: 29_000_000 },
    );
    loan.outstanding_principal = 30_000_000;
    v.mortgage = mortgage_loan_sources(identified_mortgages(vec![loan]));
    v.set_cash_charity(700_000);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_average_balance"), 295_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 14_000);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 0);
    assert_eq!(return_line(&r, "12e"), 4_620_000);
    assert_eq!(return_line(&r, "15"), 11_380_000);
    assert_eq!(return_line(&r, "16"), 1_446_000);
    assert_eq!(r.refund, Some(754_000));
}

// Single filer, both loans incurred after 12/15/2017:
//   Bank loan: interest $48,000 at a 6.000% lowest rate, interest-over-rate method:
//     48,000 / 0.06 = 800,000 average balance (line 8a interest).
//   Seller-financed note: interest $12,000, statement average 200,000 (line 8b).
// Table 1: line 6 = 0; line 7 = 1,000,000; line 8 = 750,000; line 9 = 750,000;
//   line 10 = 1,000,000; line 11 = 750,000; line 12 = 1,000,000;
//   line 14 = 0.750; line 15 = 60,000 x 0.75 = 45,000; line 16 = 15,000.
// Applied per Schedule A line: 8a = 48,000 x 0.75 = 36,000; 8b = 12,000 x 0.75
//   = 9,000; 8e = 45,000 = 12e (nothing else itemized).
#[test]
fn interest_above_the_limit_is_reduced_by_the_fraction() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 40_000_000, 0)];
    let bank =
        mortgage_loan(20210401, 4_800_000, MortgageAverageBalance::InterestOverRate { rate: 6.0 });
    let mut seller = mortgage_loan(20230101, 1_200_000, statements(20_000_000));
    seller.lender = MortgageLender::PrivateSeller {
        name: "Synthetic seller".into(),
        identifying_number: "000-00-0000".into(),
        address: "1 Example Way".into(),
    };
    itemize_mortgages(&mut d, vec![bank, seller]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_average_balance"), 1_000_000);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 36_000);
    assert_eq!(value(&r, "mortgage_interest_8b"), 9_000);
    assert_eq!(value(&r, "mortgage_interest_8e"), 45_000);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 15_000);
    assert_eq!(return_line(&r, "12e"), 4_500_000);
}

// Boundary, Table 1 line 8 for married filing separately: $375,000.
// Later debt average 500,000, interest 25,000: line 11 = 375,000, line 14 =
// 375,000 / 500,000 = 0.750, line 15 = 18,750.
#[test]
fn mortgage_limit_halves_for_married_filing_separately() {
    let mut d = complete(FederalStatus::Separate);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    itemize_mortgages(&mut d, vec![mortgage_loan(20200101, 2_500_000, statements(50_000_000))]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 375_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 18_750);
}

// Boundary, Table 1 line 12 equal to line 11: a single filer's later debt
// averaging exactly $750,000 is wholly deductible (37,500 of 37,500). One cent
// more leaves line 14 at 750,000 / 750,000.01 = 0.99999... rounded to 1.000,
// still all deductible.
#[test]
fn average_balance_at_the_limit_keeps_all_interest() {
    for balance in [75_000_000, 75_000_001] {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, 30_000_000, 0)];
        itemize_mortgages(&mut d, vec![mortgage_loan(20200101, 3_750_000, statements(balance))]);
        let r = supported(&d);
        assert_eq!(value(&r, "mortgage_interest_8a"), 37_500);
        assert_eq!(value(&r, "mortgage_nondeductible_interest"), 0);
    }
}

// Boundary, Table 1 lines 1-6 for grandfathered debt (before October 14, 1987),
// joint: loan from 1986 averaging 1,200,000 with 60,000 interest, and a 2019
// loan averaging 300,000 with 15,000 interest.
//   1 = 1,200,000; 2 = 0; 3 = 1,000,000; 4 = 1,200,000; 5 = 1,200,000;
//   6 = 1,200,000 (above 750,000); 7 = 300,000; 9 = 1,200,000;
//   10 = 1,500,000; 11 = 1,200,000; 12 = 1,500,000; 14 = 0.800;
//   15 = 75,000 x 0.8 = 60,000.
#[test]
fn grandfathered_debt_sets_the_limit_above_1000000() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 40_000_000, 0)];
    itemize_mortgages(
        &mut d,
        vec![
            mortgage_loan(19860301, 6_000_000, statements(120_000_000)),
            mortgage_loan(20190301, 1_500_000, statements(30_000_000)),
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 1_200_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 60_000);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 15_000);
}

// A loan that is not wholly acquisition debt (home equity, cash-out, mixed use)
// needs Pub 936's mixed-use worksheet: its interest is left out with a notice.
#[test]
fn mortgage_worksheet_leaves_out_mixed_use_debt() {
    let mut d = complete(FederalStatus::Single);
    let mut loan = mortgage_loan(20200101, 1_000_000, statements(20_000_000));
    mortgage_ledger_mut(&mut loan).purpose = MortgageUse::Other;
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_other_use_interest"), 10_000);
    assert!(!r.lines.iter().any(|line| line.id == "mortgage_interest_8a" && line.amount != 0));
}

// Pub 936 "Points" (unit C20b). $6,000 points on a home-acquisition purchase
// loan meeting every Pub 936 test are deductible in full in the year paid:
// line 8c = 6,000. The same $6,000 points on a 30-year (360-month) refinance
// that closed July 1, 2026 must instead be amortized (a refinance's points
// are never fully deductible in the year paid): by hand, July through
// December is 6 of the 360 months, so 6,000 x 6/360 = 100.
#[test]
fn points_are_deducted_in_full_or_amortized() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    let mut loan = mortgage_loan(20260301, 0, statements(10_000_000));
    loan.points = MortgagePoints::Purchase { amount: 600_000 };
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    // A main-home purchase loan's points are Form 1098 box 6: line 8a.
    assert_eq!(value(&r, "mortgage_interest_8a"), 6_000);
    assert_eq!(value(&r, "mortgage_points_8c"), 0);
    assert_eq!(value(&r, "mortgage_interest_8e"), 6_000);

    loan.incurred = 20260701;
    loan.points = MortgagePoints::Amortized { amount: 600_000, term_months: 360, ended: false };
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    // A refinance's points are not on a Form 1098: line 8c.
    assert_eq!(value(&r, "mortgage_interest_8a"), 0);
    assert_eq!(value(&r, "mortgage_points_8c"), 100);
}

// Pub 936 box 4 (unit C20b). A $10,000-interest loan, average balance
// $100,000 (well within every limit, so the whole amount is deductible
// before the refund). A $2,000 refund confirmed to repay interest paid in
// 2026 reduces that loan's own line 8a interest: 10,000 - 2,000 = 8,000, and
// is not Schedule 1 income. The same refund confirmed to repay a prior
// year's deducted interest leaves line 8a at the full 10,000 and is instead
// $2,000 of Schedule 1 line 8z income.
#[test]
fn refund_of_overpaid_interest_reduces_this_year_or_is_income() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    let mut loan = mortgage_loan(20200101, 1_000_000, statements(10_000_000));
    loan.refund_of_interest = MortgageInterestRefund::Yes {
        amount: 200_000,
        origin: Some(MortgageRefundOrigin::ThisYear),
    };
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 8_000);
    assert!(!r.lines.iter().any(|l| l.id == "recovery_2025_taxable"));

    loan.refund_of_interest = MortgageInterestRefund::Yes {
        amount: 200_000,
        origin: Some(MortgageRefundOrigin::Deducted(PriorYearRecovery {
            year: 2025,
            deducted: 1_000_000,
            itemized: 3_000_000,
            standard: 1_575_000,
            taxable_income: 5_000_000,
            ordinary: true,
        })),
    };
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 10_000);
    assert_eq!(value(&r, "recovery_2025_taxable"), 2_000);
}

// Matrix row 20 typical case: joint return, spouses born 1980 and 1982, one
// W-2 (box 1 $175,000, box 2 $24,000), $8,000 state income tax, $7,000 real
// estate tax (both within the $40,400 joint SALT cap at this income), $4,000
// eligible cash gifts, and two Forms 1098 on the main home, each using the
// first-and-last average-balance method:
//   Loan A (bank): incurred 2019 (post-12/15/2017 acquisition debt); box 1
//     $18,000; box 2 (Jan 1 balance) $520,000; year-end balance $508,000;
//     average = (520,000 + 508,000) / 2 = $514,000.
//   Loan B (bank refinance closed March 1, 2026, replacing a same-balance
//     2021 loan, entered with the 2021 origination date per the field's own
//     help text): box 1 $9,500; box 2 $310,000; year-end $305,000;
//     average = (310,000 + 305,000) / 2 = $307,500; box 6 points $3,000,
//     15-year (180-month) term, amortized because a refinance's points are
//     never fully deductible in the year paid.
// Pub 936 Table 1 (both loans are post-12/15/2017 acquisition debt, so both
// fall on line 7; joint):
//    1 grandfathered                                                    0
//    2 10/14/1987-12/15/2017 acquisition debt                           0
//    3 $1,000,000 (joint)                                       1,000,000
//    4 larger of 1 or 3                                         1,000,000
//    5 1 + 2                                                            0
//    6 smaller of 4 or 5                                                0
//    7 later acquisition debt: 514,000 + 307,500                821,500
//    8 $750,000 (joint)                                           750,000
//    9 larger of 6 or 8                                           750,000
//   10 6 + 7                                                      821,500
//   11 smaller of 9 or 10: qualified loan limit                   750,000
//   12 total average balance                                      821,500
//   13 total interest 18,000 + 9,500                                27,500
//   14 750,000 / 821,500 = 0.91302... rounded to                     0.913
//   15 27,500 x 0.913 = deductible interest (all one lender, line 8a)
//      27,500 x 0.913 = 25,107.50 -> 25,108
//   16 27,500 - 25,108 = not deductible                              2,392
// Points (unit C20b): loan B's refinance closed March 1, 2026; months
// through year end = 10 of 180; 3,000 x 10/180 = 166.666..., times Table 1 line
// 14 (0.913) = 152.17 -> 152 (a refinance's points are on Schedule A line 8c,
// not reported on a Form 1098). Line 8e = 8a + 8b + 8c = 25,108 + 0 + 152 =
// 25,260.
// Schedule A: 5e (SALT, $15,000 eligible against the $40,400 cap) = 15,000;
//   7 = 5e + 6(0) = 15,000; 8a = 25,108; 8c = 152; 8e = 25,260;
//   10 = 8e + 9(no investment interest) = 25,260; 11 (raw cash gift) =
//   4,000; the 2026 0.5%-of-AGI floor is 0.5% x 175,000 = 875, so 14 (no
//   prior carryover) = 0 and the current-year cash class after the floor
//   (`charity_cash_sixty_current_deduction`) = 4,000 - 875 = 3,125; 18 (high-income
//   itemized reduction) = 0, far below the top-bracket threshold.
// Total itemized = 15,000 (taxes) + 25,260 (mortgage) + 3,125 (charity) =
//   43,385, above the $32,200 joint standard deduction, so Automatic
//   comparison itemizes.
// Form 1040: 11a/11b (AGI, wages only) = 175,000; 12e (itemized) = 43,385;
//   15 (taxable income) = 175,000 - 43,385 = 131,615; 16 (2026 joint
//   schedule, 22% bracket): 2,480 + 9,120 + 22% x (131,615 - 100,800 =
//   30,815 -> 6,779.30) = 18,379.30 -> 18,379; 24a = 18,379 (no other taxes
//   or credits); 25a (W-2 withholding) = 24,000; 33 (total payments) = 24,000;
//   34 (overpayment, since payments exceed tax) = 24,000 - 18,379 = 5,621.
#[test]
fn two_1098s_return_matches_hand_computation() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19820301;
    d.income = vec![wage(1, Owner::Taxpayer, 17_500_000, 2_400_000)];
    let v = d.deductions.as_mut().unwrap();
    itemized(v).personal_tax = PersonalTaxElection::Income {
        paid: 800_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    itemized(v).personal_property_taxes = 700_000;
    let mut loan_a = mortgage_loan(
        20190815,
        1_800_000,
        MortgageAverageBalance::FirstAndLast { last_balance: 50_800_000 },
    );
    loan_a.property = "Main home".into();
    loan_a.outstanding_principal = 52_000_000;
    let mut loan_b = mortgage_loan(
        20210601,
        950_000,
        MortgageAverageBalance::FirstAndLast { last_balance: 30_500_000 },
    );
    loan_b.property = "Main home".into();
    loan_b.outstanding_principal = 31_000_000;
    loan_b.incurred = 20260301;
    // The lender reports the annual average; the March refinance date remains box 3.
    mortgage_ledger_mut(&mut loan_b).average =
        MortgageAverageBalance::Statements { average_balance: 30_750_000 };
    mortgage_ledger_mut(&mut loan_b).purpose = MortgageUse::Refinance(MortgageRefinance {
        refinanced: loan_b.outstanding_principal,
        earlier_incurred: 20210601,
        earlier_term_ends: None,
        improvement: 0,
        other: 0,
    });
    loan_b.points = MortgagePoints::Amortized { amount: 300_000, term_months: 180, ended: false };
    v.mortgage = mortgage_loan_sources(identified_mortgages(vec![loan_a, loan_b]));
    v.set_cash_charity(400_000);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_average_balance"), 821_500);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 25_108);
    assert_eq!(value(&r, "mortgage_interest_8b"), 0);
    assert_eq!(value(&r, "mortgage_points_8c"), 152);
    assert_eq!(value(&r, "mortgage_interest_8e"), 25_260);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 2_392);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 3_125);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&r, "charity_floor_not_carried"), 875);
    assert_eq!(value(&r, "agi"), 175_000);
    assert_eq!(value(&r, "deduction"), 43_385);
    assert_eq!(value(&r, "taxable_income"), 131_615);
    assert_eq!(value(&r, "income_tax"), 18_379);
    assert_eq!(return_line(&r, "11a / 11b"), 17_500_000);
    assert_eq!(return_line(&r, "12e"), 4_338_500);
    assert_eq!(return_line(&r, "15"), 13_161_500);
    assert_eq!(return_line(&r, "16"), 1_837_900);
    assert_eq!(return_line(&r, "24a"), 1_837_900);
    assert_eq!(return_line(&r, "25a"), 2_400_000);
    assert_eq!(return_line(&r, "33"), 2_400_000);
    assert_eq!(return_line(&r, "34"), 562_100);
    assert_eq!(r.tax, Some(1_837_900));
    assert_eq!(r.refund, Some(562_100));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary: a private-seller loan (no Form 1098) reports on Schedule A line
// 8b, limited by the same Pub 936 Table 1 fraction as an 8a loan. Single
// filer, later acquisition debt only, average balance $1,000,000 against the
// $750,000 single limit: line 11 = 750,000, line 12 = 1,000,000, line 14 =
// 750,000 / 1,000,000 = 0.750. Interest $40,000 x 0.750 = $30,000 on line
// 8b; line 8a stays 0 because no institution loan exists; nondeductible =
// 40,000 - 30,000 = 10,000.
#[test]
fn seller_financed_interest_uses_line_8b_with_the_fraction() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    let mut seller = mortgage_loan(20200101, 4_000_000, statements(100_000_000));
    seller.lender = MortgageLender::PrivateSeller {
        name: "Synthetic seller".into(),
        identifying_number: "000-00-0000".into(),
        address: "1 Example Way".into(),
    };
    itemize_mortgages(&mut d, vec![seller]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_average_balance"), 1_000_000);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 0);
    assert_eq!(value(&r, "mortgage_interest_8b"), 30_000);
    assert_eq!(value(&r, "mortgage_interest_8e"), 30_000);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 10_000);
}

// Boundary: Form 1098 box 4. A $10,000-interest loan with a $100,000
// average balance (well within every limit, so all of it is deductible)
// and a $2,000 refund confirmed to repay interest deducted on a prior
// year's return. That refund is Schedule 1 line 8z income, not a reduction
// of this year's box 1 interest: line 8a stays the full $10,000, and AGI
// rises by the $2,000 refund (wages $50,000 + refund $2,000 = $52,000).
#[test]
fn prior_year_refund_is_income_not_a_reduction() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 5_000_000, 0)];
    let mut loan = mortgage_loan(20200101, 1_000_000, statements(10_000_000));
    loan.refund_of_interest = MortgageInterestRefund::Yes {
        amount: 200_000,
        origin: Some(MortgageRefundOrigin::Deducted(PriorYearRecovery {
            year: 2025,
            deducted: 1_000_000,
            itemized: 3_000_000,
            standard: 1_575_000,
            taxable_income: 5_000_000,
            ordinary: true,
        })),
    };
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 10_000);
    assert_eq!(value(&r, "recovery_2025_taxable"), 2_000);
    assert_eq!(value(&r, "agi"), 52_000);
}

// Boundary: main-home purchase points deduct in full in the year paid. Second-home
// points spread over the term, and the server rejects a Purchase choice for that home.
#[test]
fn purchase_points_deduct_in_full() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    let mut loan = mortgage_loan(20260301, 0, statements(10_000_000));
    loan.points = MortgagePoints::Purchase { amount: 500_000 };
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 5_000);
    assert_eq!(value(&r, "mortgage_points_8c"), 0);

    // A second home's points must be spread over its term, not declared purchase points.
    mortgage_ledger_mut(&mut loan).home = MortgageHome::Second;
    loan.points = MortgagePoints::Amortized { amount: 500_000, term_months: 180, ended: false };
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 0);
    assert_eq!(value(&r, "mortgage_points_8c"), 278);
}

// M3-F2 fixture `m3-itemizing-homeowner` (2026-09-26). The JSON fixture
// loader in src/tests/returns.rs does not understand HSA, mortgage,
// itemized-deduction, education-credit or Form 2210 facts, so this builds
// the `FederalDraft` directly, the way the neighbouring tests above do.
// `dated`/`cents` mirror the private helpers already duplicated per test
// module in tests/penalty.rs, tests/business.rs and tests/investment/investment.rs.
fn dated(d: &mut FederalDraft, payments: &[(u32, i64)]) {
    d.payments.as_mut().unwrap().estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: payments
            .iter()
            .map(|&(date, amount)| DatedPayment {
                payer: crate::PaymentPayer::Joint,
                refund_property: None,
                date,
                amount,
            })
            .collect(),
    };
}

// The M3-F2 return (shared with the review test in tests/interview.rs).
pub(in crate::tests) fn m3_itemizing_homeowner_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19790520;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19820301;
    d.income = vec![wage(1, Owner::Taxpayer, 17_500_000, 1_200_000)];
    let FederalIncome::Wage { data: w } = &mut d.income[0].income else { panic!() };
    let mut w = edit_wage(w);
    w.other_box12 = vec![W2CodeAmount { code: W2Code::W, amount: 100_000 }];
    drop(w);

    let mut hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 300_000);
    health_w2_year(&mut d, HealthEmployerYear::CurrentYear);
    hsa.employer = 100_000;
    d.adjustments.as_mut().unwrap().hsa.push(hsa);
    // Explicit neutral completion of the original combined fixture: the
    // spouse is ineligible all year; one account alone does not establish it.
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::None }),
    });

    add_child(&mut d);
    d.dependents[0].birth_date = 20060815;
    d.dependents[0].situation = DependentSituation::Student;

    let v = d.deductions.as_mut().unwrap();
    itemized(v).personal_tax = PersonalTaxElection::Income {
        paid: 800_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    itemized(v).personal_property_taxes = 700_000;
    let mut loan_a = mortgage_loan(
        20190815,
        1_800_000,
        MortgageAverageBalance::FirstAndLast { last_balance: 50_800_000 },
    );
    loan_a.property = "Main home".into();
    loan_a.outstanding_principal = 52_000_000;
    let mut loan_b = mortgage_loan(
        20210601,
        950_000,
        MortgageAverageBalance::FirstAndLast { last_balance: 30_500_000 },
    );
    loan_b.property = "Main home".into();
    loan_b.outstanding_principal = 31_000_000;
    loan_b.incurred = 20260301;
    // The lender reports the annual average; the March refinance date remains box 3.
    mortgage_ledger_mut(&mut loan_b).average =
        MortgageAverageBalance::Statements { average_balance: 30_750_000 };
    mortgage_ledger_mut(&mut loan_b).purpose = MortgageUse::Refinance(MortgageRefinance {
        refinanced: loan_b.outstanding_principal,
        earlier_incurred: 20210601,
        earlier_term_ends: None,
        improvement: 0,
        other: 0,
    });
    loan_b.points = MortgagePoints::Amortized { amount: 300_000, term_months: 180, ended: false };
    v.mortgage = mortgage_loan_sources(identified_mortgages(vec![loan_a, loan_b]));
    v.set_cash_charity(400_000);

    let sid2_1406 = sid(&d, 2);
    set_education(
        d.credits.as_mut().unwrap(),
        vec![{
            let mut pair = native_education(
                sid2_1406,
                education_expenses(600_000),
                education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
            );
            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        }],
    );

    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 2_100_000,
        agi: Some(16_800_000),
        spouse_tax: 0,
    });
    dated(
        &mut d,
        &[(20260415, 150_000), (20260615, 150_000), (20260915, 150_000), (20270115, 150_000)],
    );
    d
}

// The m3-itemizing-homeowner fixture combines the independently audited
// two_1098s_return_matches_hand_computation (Schedule A / Pub 936 Table 1),
// hsa_return_matches_hand_computation (Form 8889),
// education_credits_return_matches_hand_computation (Form 8863) and
// underpayment_return_matches_hand_computation (Form 2210) fact patterns.
// Expected amounts below were worked by hand from the 2026 draft forms and
// instructions named by those tests, not copied from the App's output.
// Native acceptance limits: COVERAGE.md.
//
// Fixture facts: MFJ, taxpayer born 1979-05-20, spouse born
// 1982-03-01, one child born 2006-08-15, a full-time student who lived
// with them all year. One W-2 for the taxpayer: box 1 $175,000, box 2
// $12,000, box 3 $175,000, box 4 $10,850, box 5 $175,000, box 6 $2,537.50,
// box 12 code W $1,000 (wage() below derives boxes 3-6 from box 1 exactly
// as stated: 6.2% and 1.45% of $175,000, both under their 2026 wage
// bases). A family-coverage HSA all year, $3,000 of personal
// contributions. Itemized: cash gifts $4,000; state income tax $8,000 and
// property tax $7,000; two Forms 1098 on the main home -- $18,000 of
// interest on $520,000 originated 8/15/2019 with a $508,000 year-end
// balance, and $9,500 on $310,000 originated 6/1/2021 (refinanced
// 3/1/2026 with $3,000 of points over 180 months) with a $305,000
// year-end balance, both first-and-last-balance. A 1098-T for the child,
// box 1 $6,000. Estimated payments of $1,500 on 4/15, 6/15, 9/15/2026 and
// 1/15/2027; 2025 tax $21,000 on a $168,000 AGI, penalty figured on the
// return.
//
// Form 8889 (2026 family limit $8,750, Rev. Proc. 2025-19; taxpayer is 47,
// no age-55 catch-up; spouse actually ineligible, so no shared base):
//   Line 2  Personal contributions:                                $3,000
//   Line 3  Family limitation, no last-month rule:                 $8,750
//   Line 5  Line 3 - line 4 (no Archer MSA):                       $8,750
//   Line 6  Line 5 (other spouse ineligible):                                      $8,750
//   Line 8  Line 6 + line 7 (no catch-up):                         $8,750
//   Line 9  Employer contributions (W-2 box 12 code W):            $1,000
//   Line 11 Line 9 + line 10:                                      $1,000
//   Line 12 Line 8 - line 11:                                      $7,750
//   Line 13 HSA deduction, smaller of line 2 or line 12:           $3,000
// Schedule 1 line 13 (HSA deduction): $3,000. AGI = $175,000 - $3,000 =
// $172,000.
//
// Schedule A:
//   5a/5e SALT: $8,000 + $7,000 = $15,000, under the $40,400 joint cap
//     (MAGI $172,000 is far under the $505,000 phase-down threshold).
//   7 = 5e + 6(0) = $15,000.
//   Pub 936 Table 1 (both loans are post-12/15/2017 acquisition debt, so
//     both fall on line 7 of the worksheet; joint):
//       Loan A average: (520,000 + 508,000) / 2 =                 514,000
//       Loan B average: (310,000 + 305,000) / 2 =                 307,500
//       Line 7 (later acquisition debt): 514,000 + 307,500 =      821,500
//       Line 9 (larger of line 6(0) and line 8 $750,000):         750,000
//       Line 11 (qualified loan limit, smaller of 9, 10):         750,000
//       Line 13 (total interest, 18,000 + 9,500):                  27,500
//       Line 14 (750,000 / 821,500, rounded):                       0.913
//       Line 15 (27,500 x 0.913, one lender -> 8a): 25,107.50 ->   25,108
//       Line 16 (27,500 - 25,108, not deductible):                  2,392
//   Points (loan B refinanced 3/1/2026; 10 of 180 months through year
//     end): 3,000 x 10/180 = 166.666..., times line 14 (0.913) = 152.17
//     -> $152 (8c, a refinance's points no Form 1098 reports).
//   8e = 8a + 8b(0) + 8c = 25,108 + 0 + 152 =                      25,260
//   10 = 8e + 9(0) =                                               25,260
//   11 (cash gifts, raw) =                                          4,000
//   0.5% AGI floor: 0.5% x $172,000 =                                 860
//   13/charity_cash_sixty_current_deduction (11 - floor) = 4,000 - 860 =           3,140
//   15 (charity_deduction, no prior carryover) =                    3,140
//   12 (noncash gifts) and 17 (other itemized) are both 0: no such facts
//     exist on this return, so the total below is exactly 7 + 10 + 15.
//   Total itemized = 15,000 + 25,260 + 3,140 =                     43,400
// Above the $32,200 joint standard deduction, so Automatic itemizes.
//
// Form 8863 (the child is the only student; MAGI = AGI, no foreign
// exclusion). The model's `aotc` variable folds the MAGI ratio into what
// Part I calls line 7 directly (federal_rules.rs: `aotc =
// fraction(aotc_raw, education_factor, 20000)`), so the nonrefundable
// remainder is line 7 minus line 8 (the refundable 40%), not line 1 minus
// line 7:
//   Part III (AOTC, $6,000 tuition, no other qualified expenses):
//     Line 27 (adjusted expenses, capped at $4,000):                4,000
//     Line 28 (line 27 - $2,000):                                   2,000
//     Line 29 (line 28 x 25%):                                        500
//     Line 30 (100% of first $2,000 + line 29) -> Part I line 1:    2,500
//   Part I:
//     Line 2 (MFJ):                                               180,000
//     Line 3 (MAGI):                                               172,000
//     Line 4 (line 2 - line 3):                                      8,000
//     Line 5 (MFJ):                                                20,000
//     Line 6 (line 4 / line 5, inside the MAGI phase-out band):        0.400
//     Line 7 (line 1 x line 6; taxpayer is 47, no under-24 refund
//       restriction): 2,500 x 0.400 =                               1,000
//     Line 8/refundable (40% of line 7) -> 1040 line 29:              400
//     Line 9 (line 7 - line 8), nonrefundable remainder: 1,000-400 =    600
//   No LLC student, so Part II is $0. Line 19 (Credit Limit Worksheet:
//     line 9 + line 18($0), well under the remaining tax after the ODC
//     below) -> Schedule 3 line 3:                                    600
//
// The child is 20 at year end (born 8/15/2006): too old for the $2,000
// child tax credit, so the $500 credit for other dependents applies
// instead (1040 line 19), exactly as in
// education_credits_return_matches_hand_computation.
//
// Form 1040:
//   1a/9  Wages:                                                  175,000
//   10   Adjustments (Schedule 1 line 13, HSA):                     3,000
//   11a/11b AGI:                                                  172,000
//   12e  Itemized deduction:                                       43,400
//   15   Taxable income:                                          128,600
//   16   Tax (2026 MFJ schedule, 22% bracket over $100,800: $11,600 +
//        22% x (128,600-100,800=27,800) = 11,600+6,116.00=17,716.00 ->
//                                                                    17,716
//   19   Child/other-dependent credit (ODC only):                     500
//   20   Other nonrefundable credits (education):                     600
//   21   Total nonrefundable credits:                               1,100
//   22   Tax after credits: 17,716-1,100 =                         16,616
//   23   Other taxes (Schedule 2; no SE tax, no HSA additional tax; the
//        AMT tentative minimum tax on $143,585 AMTI, 26% x ($143,585 -
//        the $140,200 exemption) = $880, stays below the $17,716 regular
//        tax):                                                          0
//   24a  Total tax:                                                16,616
//   25a  W-2 withholding:                                           12,000
//   26   2026 estimated payments (4 x $1,500):                       6,000
//   29   Refundable American opportunity credit:                       400
//   33   Total payments and refundable credits:                    18,400
//
// Form 2210 Part I (the 110% prior-year rule: 2025 AGI $168,000 is over
// the $150,000 joint threshold):
//   Line 1 (1040 line 22):                                         16,616
//   Line 2 (Schedule 2 Part II):                                        0
//   Line 3 (refundable AOTC):                                          400
//   Line 4 (1 + 2 - 3):                                             16,216
//   Line 5 (90% of line 4):                                      14,594.40
//   Line 6 (withholding, box 2):                                    12,000
//   Line 7 (line 4 - line 6):                                        4,216
//   Line 8 (2025 tax $21,000 x 110%):                                23,100
//   Line 9 (smaller of 5 or 8):                                  14,594.40
// Part III: each required installment (line 10) is 25% of line 9, rounded
// once to $3,648.60 (the same rounded amount every column, per
// underpayment.rs's `fraction(l9, 1, 4)`). Withholding is spread evenly,
// $3,000/quarter; each period's own $1,500 estimated payment lands
// exactly on that period's due date, so every column receives $4,500 --
// comfortably more than its $3,648.60 installment -- because a payment
// dated exactly on the due date is credited to that period
// (`part_3`'s `stream[next].0 <= *due` test in underpayment.rs). No
// period ever falls short, so line 17 is $0 in every column and line 19
// (the penalty) is $0: Form 1040 line 38 is $0. Because
// `payments.prior_year` is present, the return still routes through the
// "amount you owe/refund after the estimated tax penalty" labels on
// lines 35a/37 even though the penalty itself is $0, leaving the same
// $1,784 refund either way.
#[test]
fn itemizing_homeowner_return_matches_hand_computation() {
    let d = m3_itemizing_homeowner_draft();

    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 3_000);
    assert_eq!(value(&r, "agi"), 172_000);
    assert_eq!(value(&r, "deduction"), 43_400);
    assert_eq!(value(&r, "taxable_income"), 128_600);
    assert_eq!(value(&r, "income_tax"), 17_716);
    assert_eq!(value(&r, "mortgage_average_balance"), 821_500);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 25_108);
    assert_eq!(value(&r, "mortgage_interest_8b"), 0);
    assert_eq!(value(&r, "mortgage_points_8c"), 152);
    assert_eq!(value(&r, "mortgage_interest_8e"), 25_260);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 2_392);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 3_140);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&r, "charity_floor_not_carried"), 860);
    assert_eq!(value(&r, "refundable_education_credit"), 400);
    // The nonrefundable AOTC remainder is line 7 ($1,000) minus the
    // refundable line 8 ($400) = $600, not line 1 ($2,500) minus line 7:
    // federal_rules.rs folds the MAGI ratio into `aotc` before computing
    // `education_refund`/`education_nonrefund`.
    assert_eq!(value(&r, "education_credit"), 600);

    assert_eq!(return_line(&r, "1a"), 17_500_000);
    assert_eq!(return_line(&r, "9"), 17_500_000);
    assert_eq!(return_line(&r, "10"), 300_000);
    assert_eq!(return_line(&r, "11a / 11b"), 17_200_000);
    assert_eq!(return_line(&r, "12e"), 4_340_000);
    assert_eq!(return_line(&r, "15"), 12_860_000);
    assert_eq!(return_line(&r, "16"), 1_771_600);
    assert_eq!(return_line(&r, "19"), 50_000);
    assert_eq!(return_line(&r, "20"), 60_000);
    assert_eq!(return_line(&r, "21"), 110_000);
    assert_eq!(return_line(&r, "22"), 1_661_600);
    assert_eq!(return_line(&r, "24a"), 1_661_600);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(return_line(&r, "26"), 600_000);
    assert_eq!(return_line(&r, "29"), 40_000);
    assert_eq!(return_line(&r, "33"), 1_840_000);
    assert_eq!(return_line(&r, "34"), 178_400);
    assert_eq!(return_line(&r, "35a"), 178_400);
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(return_line(&r, "38"), 0);

    for (line, expected) in [
        (1, 1_661_600),
        (2, 0),
        (3, 40_000),
        (4, 1_621_600),
        (5, 1_459_440),
        (6, 1_200_000),
        (7, 421_600),
        (8, 2_310_000),
        (9, 1_459_440),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_{line}")), expected);
    }
    for column in ["a", "b", "c", "d"] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_10_{column}")), 364_860);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_11_{column}")), 450_000);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), 0);
    }
    assert_eq!(cents(&r, "form_2210_line_19"), 0);

    assert_eq!(r.tax, Some(1_661_600));
    assert_eq!(r.refund, Some(178_400));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (M3-F2): the SALT cap phase-down (schedule-a.md line 5e).
// Single filer, MAGI $605,000 -- $100,000 over the $505,000 phase-down
// threshold -- reduces the flat $40,400 cap by 30% of the excess
// ($30,000), to $10,400, still above the $10,000 floor. $50,000 of state
// income tax paid is far above either cap, so forcing the itemized
// election (below the $16,100 single standard deduction) isolates the
// deduction to exactly the phased-down $10,400, not the flat $40,400 or
// the full $50,000 paid.
#[test]
fn salt_cap_phases_down_above_the_505000_magi_threshold() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 60_500_000, 0)];
    let v = d.deductions.as_mut().unwrap();
    itemized(v).personal_tax = PersonalTaxElection::Income {
        paid: 5_000_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    v.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 605_000);
    assert_eq!(value(&r, "deduction"), 10_400);
}

// Boundary (M3-F2): Pub 936 Table 1's fraction (schedule-a.md line 8a/8e)
// at exactly the $750,000 single/joint qualified loan limit. One
// institution loan, average balance exactly $750,000 (line 12), equal to
// the qualified loan limit (line 11): line 14 is 750,000 / 750,000 =
// 1.000 exactly, so the full $30,000 of interest is deductible on line 8a
// and nothing is nondeductible -- proving the boundary lands on "fully
// deductible," not just under it.
#[test]
fn pub_936_table_1_fraction_is_exactly_one_at_the_750000_limit() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    let loan = mortgage_loan(20200101, 3_000_000, statements(75_000_000));
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_average_balance"), 750_000);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 30_000);
    assert_eq!(value(&r, "mortgage_nondeductible_interest"), 0);
}

// Boundary (M3-F2): a Form 2210 estimated payment made exactly on its
// installment's due date (underpayment.rs `part_3`'s `stream[next].0 <=
// *due` test). Single filer, $80,000 wages, no withholding; 2025 tax
// $4,000 on a $100,000 AGI (at or under the $150,000 threshold, so the
// 100% safe harbor applies unmultiplied) makes the required annual
// payment (line 9) the smaller $4,000, not 90% of the $8,770 current-year
// tax ($7,893). Each required installment (line 10) is 25% of $4,000 =
// $1,000. The single $1,000 estimated payment lands on 4/15/2026, the
// first column's own due date: it is credited to that column (line 11 =
// $1,000) and exactly satisfies the $1,000 installment (line 17 = $0),
// rather than arriving "too late" for its own due date.
#[test]
fn estimated_payment_on_the_due_date_counts_for_its_own_period() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 400_000, agi: Some(10_000_000), spouse_tax: 0 });
    dated(&mut d, &[(20260415, 100_000)]);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_9"), 400_000);
    assert_eq!(cents(&r, "form_2210_line_10_a"), 100_000);
    assert_eq!(cents(&r, "form_2210_line_11_a"), 100_000);
    assert_eq!(cents(&r, "form_2210_line_17_a"), 0);
}

// T2-37: Form 4952 investment interest expense (matrix row 37). Boundary at
// the line 6 net investment income limit: current-year interest paid
// exceeds net investment income, with no prior carryover, so line 8's
// "smaller of" comparison caps the deduction below what was actually paid.
#[test]
fn investment_interest_above_net_investment_income_limits_the_line_6_deduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(5000, 0)));
    investment_borrowing(&mut d, 8000, 0, 0);
    let r = supported(&d);
    // Line 1-3: $8,000 interest paid in 2026, no 2025 carryover, so line 3
    // (total investment interest expense) is $8,000.
    // Lines 4a-4h/6: the $5,000 taxable interest is the only investment
    // income, with no line 5 investment expenses, so net investment income
    // (line 6) is $5,000.
    // Line 8: smaller of line 3 ($8,000) and line 6 ($5,000) = $5,000.
    // Line 7: line 3 minus line 8 = $3,000, carried to the 2027 Form 4952.
    assert_eq!(value(&r, "investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 3000);
    // The separate AMT Form 4952 uses the same $5,000 investment income and
    // has no AMT adjustment here, so its deduction and carryforward match.
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 3000);
}

// T2-37: Form 4952. Boundary: interest paid is exactly equal to net
// investment income, so line 8's limit binds with nothing left over -
// distinct from the case above it, where paid exceeds income.
#[test]
fn investment_interest_exactly_equal_to_net_investment_income_is_fully_deductible() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(6000, 0)));
    investment_borrowing(&mut d, 6000, 0, 0);
    let r = supported(&d);
    // Line 3 is $6,000 paid; line 6 net investment income is $6,000
    // taxable interest (no expenses, no prior carryover). Line 3 equals
    // line 6 exactly, so line 8 allows the full amount and line 7 (the
    // carryforward) is $0, not a partial disallowance.
    assert_eq!(value(&r, "investment_interest_deduction"), 6000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
}

// T2-37: Form 4952. Boundary: a 2025 Form 4952 line 7 carryover (line 2)
// adds to current-year interest (line 1) before line 3 is limited, so the
// carryforward mechanics on line 7 are visible even though the income
// limit itself is the same $5,000 as the first test above.
#[test]
fn prior_year_carryover_adds_to_line_3_before_the_line_7_carryforward() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(5000, 0)));
    investment_borrowing(&mut d, 8000, 2000, 0);
    let r = supported(&d);
    // Line 1: $8,000 paid in 2026. Line 2: $2,000 disallowed from the 2025
    // Form 4952 line 7. Line 3: $10,000. Net investment income (line 6)
    // is unchanged at $5,000 (taxable interest only, no expenses).
    // Line 8: smaller of line 3 ($10,000) and line 6 ($5,000) = $5,000.
    // Line 7: line 3 minus line 8 = $5,000 carried to 2027 - larger than
    // either the prior carryover or the current year's interest alone,
    // because both pools combine on line 3 before the line 6 limit applies.
    assert_eq!(value(&r, "investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 5000);
}

// T2-37: Form 4952. Boundary at line 5 investment expenses: expenses equal
// gross investment income exactly, so net investment income (line 6) is
// $0 and none of the interest is deductible. Investment-property taxes
// are an AMT addback rather than an AMT expense, so the separate AMT
// Form 4952 is unaffected and allows the full amount.
#[test]
fn investment_expenses_on_line_5_reduce_net_investment_income_but_not_the_amt_track() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(4000, 0)));
    investment_borrowing(&mut d, 1000, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 400_000;
    let r = supported(&d);
    // Lines 4a-4h: $4,000 taxable interest is the only investment income.
    // Line 5: $4,000 investment expenses (the investment property tax)
    // equal that income exactly, so line 6 net investment income is $0.
    // Line 8: smaller of line 3 ($1,000) and line 6 ($0) = $0.
    // Line 7: line 3 minus line 8 = the full $1,000, carried to 2027.
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_interest_carryforward"), 1000);
    // The AMT track does not subtract investment-property taxes, so its
    // investment income stays at $4,000: the full $1,000 is allowed and
    // nothing carries forward for AMT.
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 1000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 0);
}

// T2-37: Form 4952 line 4g election to include net capital gain (and,
// if needed, qualified dividends) in investment income, and its effect
// on the Qualified Dividends and Capital Gain Tax Worksheet. Single
// filer: $10,000 wages, $5,000 fully-qualified ordinary dividends and a
// $3,000 capital gain distribution (Form 4952's only investment income
// besides interest), $2,000 of interest paid, and a $2,000 capital-first
// election.
#[test]
fn line_4g_election_moves_capital_gain_into_investment_income_and_out_of_the_qdcg_worksheet() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(native_dividend(500_000, 500_000, 300_000)),
        }),
    );
    investment_borrowing(&mut d, 2000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 200_000 };
    let r = supported(&d);
    // Line 4d/4e: the $3,000 net gain from the capital gain distribution is
    // the only disposition of investment property, so it is all available
    // net investment capital gain. Capital-first electing $2,000 takes it
    // entirely from that $3,000 gain (line 4e), leaving none of the
    // $5,000 qualified dividends (line 4b) needed on line 4g.
    assert_eq!(value(&r, "investment_elected_capital_gain"), 2000);
    assert_eq!(value(&r, "investment_elected_qualified_dividends"), 0);
    // Line 4h/6: investment income is the $2,000 elected onto line 4g (the
    // ordinary dividends and gain otherwise net to $0 once the offsetting
    // qualified-dividend and net-capital-gain exclusions are backed out).
    // Line 3 ($2,000 paid, no carryover) equals line 6 exactly, so line 8
    // allows the full $2,000.
    assert_eq!(value(&r, "investment_interest_deduction"), 2000);
    // The election does not change what is reported as qualified dividends
    // on Form 1040 line 3a - it only moves the election's use out of the
    // preferential-rate worksheet.
    assert_eq!(return_line(&r, "3a"), 500_000);
    // Hand check of the QDCG worksheet: AGI is $18,000 (wages $10,000 plus
    // $5,000 ordinary dividends plus the $3,000 capital gain distribution).
    // The only itemized deduction is the $2,000 investment interest above,
    // so taxable income (line 15) is $16,000.
    // The worksheet's preferential income is now $5,000 qualified dividends
    // (line 4b, unelected) plus $1,000 remaining net capital gain ($3,000
    // line 4e amount minus the $2,000 elected onto Form 4952) = $6,000,
    // leaving $10,000 taxed at ordinary rates.
    // $16,000 total is far under the $49,450 single 0%-rate breakpoint, so
    // the entire $6,000 preferential amount is taxed at 0% and the tax is
    // simply the Tax Table amount on the $10,000 ordinary balance: midpoint
    // $10,025 falls in the 10% bracket, $10,025 x 10% = $1,002.50, which
    // the Tax Table rounds up to $1,003.
    assert_eq!(value(&r, "regular_income_tax"), 1003);
}

// T2-34 boundaries: Form 8283's $500 filing threshold, the 30% limit for
// appreciated public stock, the 50% limit for ordinary property and the
// more-than-one-year holding period, on the single $60,000-wage return
// (AGI $60,000, 0.5% floor $300) itemizing with nothing else.
fn itemizing_with_gifts(
    goods: Vec<PropertyGift>,
    stock: Vec<PropertyGift>,
    vehicles: Vec<PropertyGift>,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let v = d.deductions.as_mut().unwrap();
    v.itemize_override = Some(ItemizeOverride::Itemize);
    set_gifts(v, goods.into_iter().chain(stock).chain(vehicles).collect());
    d
}
fn goods_at(kind: GoodsKind, value: i64) -> PropertyGift {
    let mut g = used_goods(value);
    g.description = "Used items".into();
    g.charity = "Synthetic Thrift Charity".into();
    g.given = 20260321;
    g.acquired = 20210615;
    g.basis = 1_000_000;
    g.value_method = ValueMethod::ComparableSales;
    g.kind = GiftKind::Goods { kind, condition: GoodsCondition::Good };
    g
}
fn purchased_stock(acquired: u32, basis: i64, value: i64) -> PropertyGift {
    let mut g = public_stock_gift();
    g.description = "Example Corp. shares".into();
    g.charity = "Synthetic Public Charity".into();
    g.given = 20261115;
    g.acquired = acquired;
    g.basis = basis;
    g.value = value;
    g
}
fn sold_car(value: i64, proceeds: i64) -> PropertyGift {
    let mut g = donated_personal_vehicle(VehicleUse::Sold { sold: 20260710, proceeds });
    g.description = "2014 Example sedan".into();
    g.charity = "Synthetic Vehicle Charity".into();
    g.given = 20260601;
    g.acquired = 20140301;
    g.basis = 5_000_000;
    g.value = value;
    g
}

#[test]
fn form_8283_is_needed_only_above_500_of_noncash_gifts() {
    // i8283: file Form 8283 when all noncash gifts total more than $500.
    // $500.00 of clothing: no Section A line; $500.01: Section A $500.
    let at = |cents| {
        supported(&itemizing_with_gifts(vec![goods_at(GoodsKind::Clothing, cents)], vec![], vec![]))
    };
    let r = at(50_000);
    assert!(!r.lines.iter().any(|l| l.id == "form_8283_section_a_value"));
    // $500 allowed less the $300 floor.
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 200);
    let r = at(50_001);
    assert_eq!(value(&r, "form_8283_section_a_value"), 500);
}

#[test]
fn appreciated_stock_is_limited_to_30_percent_of_agi() {
    // 30% x $60,000 = $18,000. At $18,000 all of it is allowed and the floor
    // ($300, charged to this class first) leaves $17,700 with no carryover.
    // At $18,001 the $1 excess carries, and because a percentage limit now
    // creates a carryover the class's $300 floor share carries too: $301.
    let at = |dollars: i64| {
        supported(&itemizing_with_gifts(
            vec![],
            vec![purchased_stock(20200302, 100_000, dollars * 100)],
            vec![],
        ))
    };
    let r = at(18_000);
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&r, "charity_capital_gain_thirty_return_year_carryforward"), 0);
    let r = at(18_001);
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&r, "charity_capital_gain_thirty_return_year_carryforward"), 301);
}

#[test]
fn ordinary_property_is_limited_to_50_percent_of_agi() {
    // 50% x $60,000 = $30,000: clothing $5,000 plus a car sold for $25,000
    // (value $25,000) fits exactly; the deduction is $30,000 less the $300
    // floor. With proceeds of $25,001 the claim stays $25,000 (value is the
    // smaller); with value and proceeds of $25,001 the $1 excess carries with
    // the class's $300 floor share: $301.
    let at = |value_cents, proceeds_cents| {
        supported(&itemizing_with_gifts(
            vec![goods_at(GoodsKind::Clothing, 500_000)],
            vec![],
            vec![sold_car(value_cents, proceeds_cents)],
        ))
    };
    let r = at(2_500_000, 2_500_000);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 29_700);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 0);
    let r = at(2_500_000, 2_500_100);
    assert_eq!(value(&r, "charity_property_2_claim"), 25_000);
    let r = at(2_500_100, 2_500_100);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 29_700);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 301);
}

#[test]
fn stock_held_exactly_one_year_is_not_long_term() {
    // Pub 550: the holding period starts the day after purchase, so shares
    // bought November 15, 2025 and given November 15, 2026 are short-term
    // and claim the smaller of value ($3,000) and basis ($1,000) in the 50%
    // class; bought a day earlier, they are long-term appreciated stock at
    // value in the 30% class. The store refuses the other combinations.
    let short = purchased_stock(20251115, 100_000, 300_000);
    let r = supported(&itemizing_with_gifts(vec![], vec![short.clone()], vec![]));
    // $1,000 less the $300 floor.
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 700);
    let mut wrong = short;
    wrong.acquired = 20251114;
    assert_eq!(
        value(
            &supported(&itemizing_with_gifts(vec![], vec![wrong], vec![])),
            "charity_capital_gain_thirty_current_deduction"
        ),
        2_700
    );
    let long = purchased_stock(20251114, 100_000, 300_000);
    let r = supported(&itemizing_with_gifts(vec![], vec![long], vec![]));
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 2_700);
}

// ---- Unit F4 (2026-10-10, line audit schedules-a-b-d-8949 D1-D5) ----

fn single_with_wages(dollars: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, dollars * 100, 0)];
    d
}

// D1: Schedule A 8d, Pub. 936 (2026) Mortgage Insurance Premiums worksheet. Single, AGI $104,500,
// one Form 1098 inside the limit: box 1 $12,000, box 5 $1,800. AGI is $4,500 over $100,000, so five
// $1,000 steps (or part of one) cut the premiums by 50%: 8d $900, 8e $12,000 + $900 = $12,900.
#[test]
fn mortgage_insurance_premiums_phase_out_on_line_8d() {
    let mut d = single_with_wages(104_500);
    let mut loan = mortgage_loan(20200101, 1_200_000, statements(10_000_000));
    loan.mortgage_insurance_premiums = 180_000;
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 104_500);
    assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), 900);
    assert_eq!(
        printed_line(&r, "mortgage_insurance_premiums_8d"),
        (TaxForm::ScheduleA, "8d".into())
    );
    assert_eq!(value(&r, "mortgage_interest_8a"), 12_000);
    assert_eq!(value(&r, "mortgage_interest_8e"), 12_900);
    // Boundaries: nothing at or under $100,000; a part of a $1,000 step counts as a whole step;
    // $9,000 over leaves 10%; one dollar more leaves none.
    for (agi, expected) in [
        (100_000, 1_800),
        (100_001, 1_620),
        (101_000, 1_620),
        (101_001, 1_440),
        (109_000, 180),
        (109_001, 0),
    ] {
        let mut d = single_with_wages(agi);
        itemize_mortgages(&mut d, vec![loan.clone()]);
        let r = supported(&d);
        assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), expected, "{agi}");
        assert_eq!(value(&r, "mortgage_interest_8e"), 12_000 + expected, "{agi}");
    }
    // Married filing separately: $50,000 start and $500 steps.
    for (agi, expected) in [(50_000, 1_800), (50_001, 1_620), (54_500, 180), (54_501, 0)] {
        let mut d = complete(FederalStatus::Separate);
        d.income = vec![wage(1, Owner::Taxpayer, agi * 100, 0)];
        itemize_mortgages(&mut d, vec![loan.clone()]);
        let r = supported(&d);
        assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), expected, "{agi}");
    }
    // The premiums are not part of the Table 1 interest, and a loan with none emits a zero line.
    let mut d = single_with_wages(104_500);
    itemize_mortgages(&mut d, vec![mortgage_loan(20200101, 1_200_000, statements(10_000_000))]);
    assert_eq!(value(&supported(&d), "mortgage_insurance_premiums_8d"), 0);
}

// D2: Pub. 936 (2026) "Claiming your deductible points" multiplies the points by Table 1 line 14.
// Single, main home bought 2026-03-02, lender-statement average balance $1,000,000, box 1 $60,000,
// box 6 $10,000 with every full-deduction test met: line 11 $750,000, line 12 $1,000,000, line 14
// 0.750. Interest $45,000 + points $7,500 = $52,500 (8e).
#[test]
fn points_are_limited_by_the_table_1_fraction() {
    let mut d = single_with_wages(200_000);
    let mut loan = mortgage_loan(20260302, 6_000_000, statements(100_000_000));
    loan.points = MortgagePoints::Purchase { amount: 1_000_000 };
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_qualified_loan_limit"), 750_000);
    assert_eq!(value(&r, "mortgage_average_balance"), 1_000_000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 52_500);
    assert_eq!(value(&r, "mortgage_points_8c"), 0);
    assert_eq!(value(&r, "mortgage_interest_8e"), 52_500);
    // Points no Form 1098 reports (a refinance) are limited the same way on line 8c: the 2026
    // part of $10,000 over 360 months from March 2026 (10 months) is $277.78, times 0.750.
    loan.incurred = 20260302;
    loan.points = MortgagePoints::Amortized { amount: 1_000_000, term_months: 360, ended: false };
    itemize_mortgages(&mut d, vec![loan]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 45_000);
    assert_eq!(value(&r, "mortgage_points_8c"), 208);
    assert_eq!(value(&r, "mortgage_interest_8e"), 45_208);
}

// D3: Form 1098 box 6 points are reported on Schedule A line 8a ("Enter on line 8a mortgage
// interest and points reported to you on Form 1098"); line 8c holds only points no Form 1098
// reports. Box 1 $8,000, box 6 $2,000 (every test met, inside the limit): 8a $10,000, 8c $0.
#[test]
fn points_reported_on_form_1098_are_line_8a() {
    let mut d = single_with_wages(120_000);
    let mut loan = mortgage_loan(20260101, 800_000, statements(10_000_000));
    loan.points = MortgagePoints::Purchase { amount: 200_000 };
    itemize_mortgages(&mut d, vec![loan.clone()]);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 10_000);
    assert_eq!(value(&r, "mortgage_points_8c"), 0);
    assert_eq!(value(&r, "mortgage_interest_8e"), 10_000);
    // The Form 8396 credit reduces only the box 1 interest, never the points.
    loan.points = MortgagePoints::No;
    itemize_mortgages(&mut d, vec![loan]);
    assert_eq!(value(&supported(&d), "mortgage_interest_8a"), 8_000);
}

// D4: on the 2026 Schedule A line 16 is the disaster casualty loss and gambling losses are line 17a.
// Single itemizer, winnings $5,000, losses $4,000: 90% of the losses is $3,600, under the winnings.
#[test]
fn gambling_losses_are_schedule_a_line_17a() {
    let mut d = single_with_wages(60_000);
    add(&mut d, gambling(5_000, 4_000));
    let v = d.deductions.as_mut().unwrap();
    v.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 3_600);
    assert_eq!(printed_line(&r, "gambling_loss_deduction"), (TaxForm::ScheduleA, "17a".into()));
    assert_eq!(value(&r, "personal_casualty_deduction"), 0);
    assert_eq!(printed_line(&r, "personal_casualty_deduction"), (TaxForm::ScheduleA, "16".into()));
}

// D5: Schedule A line 18 and Form 1040 line 15. Section 68(a)(2) measures taxable income without the
// section 68 reduction and increased by the itemized deductions, not floored at zero first. Single,
// AGI $700,000 (all taxable interest), itemized $701,000 (Form 4952 interest $691,000 allowed in
// full against $700,000 of net investment income, real estate tax $10,000 at the floored $10,000
// SALT cap): 700,000 - 640,600 = 59,400, the smaller of that and $701,000, times 2/37 = $3,211.
// Line 18 is 701,000 - 3,211 = 697,789 and Form 1040 line 15 is 700,000 - 697,789 = 2,211.
#[test]
fn itemized_reduction_does_not_floor_taxable_income_at_zero() {
    let mut d = complete(FederalStatus::Single);
    d.income.clear();
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(70_000_000, 0)) }),
    );
    investment_borrowing(&mut d, 691_000, 0, 0);
    let v = d.deductions.as_mut().unwrap();
    let mut facts = itemized(v).clone();
    facts.personal_property_taxes = 1_000_000;
    {
        v.itemized = facts;
        v.itemize_override = Some(ItemizeOverride::Itemize);
    };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 700_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 691_000);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 3_211);
    assert_eq!(return_line(&r, "12e"), 69_778_900);
    assert_eq!(return_line(&r, "15"), 221_100);
    assert_eq!(value(&r, "taxable_income"), 2_211);
}

// D6: the 2026 Schedule A prints the Charitable Contribution Limitation Worksheet result on line 13
// and the carryover from the prior year on line 14, so the prior-year carryover rows name line 14.
#[test]
fn prior_year_charity_carryover_labels_name_schedule_a_line_14() {
    for key in [
        "EXPLAIN_CHARITY_PRIOR_CASH_DEDUCTION",
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_DEDUCTION",
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_DEDUCTION",
    ] {
        let text = crate::APP_STRINGS.iter().find(|(k, _)| *k == key).map(|(_, t)| *t).unwrap();
        assert!(text.contains("Schedule A line 14"), "{key}: {text}");
        assert!(!text.contains("line 13") && !text.contains("line 12"), "{key}");
    }
}
