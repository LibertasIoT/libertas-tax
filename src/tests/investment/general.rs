// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
use crate::tests::federal::general::final_person;

#[test]
fn collectible_rate_is_a_marginal_cap_not_a_flat_tax() {
    use crate::federal_rules::preferential_tax_with_collectibles as tax;
    // With no other income, $10,000 stays in the ordinary 10% bracket.
    assert_eq!(tax(10000, 10000, 10000, FederalStatus::Single), 1003);
    // At the 2026 24% ceiling, the next $1,000 of collectible gain is capped at 28%.
    let baseline = preferential_tax(201775, 0, FederalStatus::Single);
    assert_eq!(tax(202775, 1000, 1000, FederalStatus::Single), baseline + 280);
    // A $2,000 gain crosses the ceiling: $1,000 at 24%, $1,000 at 28%.
    let baseline = preferential_tax(200775, 0, FederalStatus::Single);
    assert_eq!(tax(202775, 2000, 2000, FederalStatus::Single), baseline + 520);
    // Remaining 0%-eligible ordinary capital gain does not make collectibles tax free.
    assert_eq!(tax(20000, 20000, 10000, FederalStatus::Single), 1003);
    // Deductions can consume the collectible portion before ordinary preferential gains.
    assert_eq!(tax(5000, 20000, 10000, FederalStatus::Single), 0);
}
#[test]
fn worthless_security_negative_answer_and_invalid_acquisition_dates() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::WorthlessSecurity {
            data: crate::WorthlessSecurity {
                basis: 100000,
                acquisition: SecurityAcquisition::Purchased { date: 20261231 },
                worthless: false,
                ordinary: true,
            },
        }),
    );
    let r = supported(&d);

    assert!(!r.lines.iter().any(|line| line.id.starts_with("worthless_security_")));
    {
        let date = 20270201;
        let FederalIncome::WorthlessSecurity { data: crate::WorthlessSecurity { acquisition, .. } } =
            &mut d.income[1].income
        else {
            panic!()
        };
        *acquisition = SecurityAcquisition::Purchased { date };
        assert!(!d.valid_stored());
    }
}

#[test]
fn family_schedule_d_obeys_low_income_skip_and_signed_line_42() {
    use crate::federal_rules::schedule_d_tax;
    // Line 1 equals line 16: skip special rates and use the Tax Table on $2,000.
    assert_eq!(schedule_d_tax(3000, 2000, 1000, 2000, true, FederalStatus::Single), 201);
    // Lines 21/39/42: $230,000 / $10,000 / -$2,000. Original line 19 can
    // exceed allocated line 11 in the family worksheet; line 42 has no floor.
    assert_eq!(schedule_d_tax(250000, 20000, 8000, 10000, true, FederalStatus::Single), 53796);
    // No original 28% gain means lines 41–43 are skipped instead.
    assert_eq!(schedule_d_tax(250000, 20000, 8000, 10000, false, FederalStatus::Single), 54356);
}

#[test]
fn seller_adjusted_sale_rejects_dates_before_adjustment_or_outside_tax_year() {
    for date in [20260614, 20270201] {
        let mut entry = price_reduction_debt();
        if let FederalIncome::PersonalDebt { data } = &mut entry.income {
            if let Some(CanceledDebtChoice::SellerReduction { sale, .. }) = &mut data.treatment {
                *sale = Some(DebtPropertySale {
                    date,
                    term: GainTerm::Short,
                    collectible: false,
                    proceeds: 1000000,
                    withholding: 0,
                });
            }
            assert!(!PersonalDebtCancellation::from(&*data).valid());
        }
    }
}

#[test]
fn documented_interest_premium_can_coexist_with_after_death_components() {
    // Documented box 11 premium belongs entirely to the final return; the later
    // recipient has zero premium. With the taxable-premium election, net retained interest: 1,000 − 400 − 100 = 500.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    d.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20260630));
    let mut interest = income(savings_income(1000, 0));
    if let FederalIncome::Interest { data } = &mut interest.income {
        let mut data = edit_interest(data);
        data.bond_premium = 10_000;
        data.after_death = Some(InterestAfterDeath {
            taxable: 40_000,
            us_obligations: 0,
            exempt: 0,

            private_activity: 0,
            foreign_tax: 0,
            bond_premium: 0,
            treasury_bond_premium: 0,
            tax_exempt_bond_premium: 0,
            accrued_interest: 0,
            market_discount: 0,
        });
    }
    add(&mut d, interest);
    let r = supported(&d);
    assert!(!noted(&r, "WARN_AFTER_DEATH_SPLIT_LEFT_OUT"));
    assert!(noted(&r, "NOTE_INCOME_IN_RESPECT_OF_A_DECEDENT"));
    assert_eq!(return_line(&r, "2b"), 50_000);
}
