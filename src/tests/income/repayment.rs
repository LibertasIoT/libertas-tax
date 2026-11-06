// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn unemployment_same_year_repayments_net_once_on_complete_return() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    add(&mut d, government_unemployment(10000, &[500, 1500], 1000));
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_gross"), 10000);
    assert_eq!(value(&r, "unemployment_2_repaid"), 2000);
    assert_eq!(value(&r, "unemployment_2_taxable"), 8000);
    assert_eq!(value(&r, "agi"), 68000);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(return_line(&r, "8"), 800000);
    assert_eq!(r.tax, Some(613600));
    assert_eq!(r.payments, Some(100000));
    assert_eq!(r.balance, Some(513600));
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn unemployment_preserves_exact_repayment_cents_until_tax_line_rounding() {
    let mut d = complete(FederalStatus::Single);
    let mut e = government_unemployment(10000, &[3000, 1000], 0);
    let FederalIncome::GovernmentPayment { data } = &mut e.income else { panic!() };
    data.form.unemployment_compensation += 1;
    data.repayments[0].amount -= 1;
    data.repayments[1].amount += 1;
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(r.lines.iter().find(|v| v.id == "unemployment_2_taxable").unwrap().amount, 600001);
    assert_eq!(value(&r, "agi"), 66000);
    assert_eq!(r.tax, Some(574300));
}

#[test]
fn unemployment_prior_year_excess_and_unknown_treatment_remain_explicit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, government_unemployment(10000, &[2000], 0));
    for mode in 0..2 {
        let mut bad = d.clone();
        let FederalIncome::GovernmentPayment { data } = &mut bad.income[1].income else { panic!() };
        match mode {
            0 => {
                data.repayments[0].received_year = 2025;
                data.repayments[0].amount = 300001;
            }
            _ => data.repayments[0].amount = 1000001,
        }
        unsupported(&bad, if mode == 0 { "Earlier-income repayments" } else { "Income" });
    }
    {
        let mut bad = d.clone();
        let FederalIncome::GovernmentPayment { data } = &mut bad.income[1].income else { panic!() };
        data.repayments = vec![data.repayments[0].clone(); 101];
        assert!(!bad.valid_stored());
    }
}

#[test]
fn prior_unemployment_small_repayments_leave_income_tax_and_payments_unchanged() {
    for amount in [1, 299999, 300000] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
        add(&mut d, government_unemployment(10000, &[2000], 1000));
        add(&mut d, prior_unemployment_repayment(amount, Owner::Taxpayer));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 68000);
        assert_eq!(value(&r, "adjustments"), 0);
        assert_eq!(r.tax, Some(613600));
        assert_eq!(r.payments, Some(100000));
        assert_eq!(r.balance, Some(513600));
        assert_eq!(
            r.lines.iter().find(|v| v.id == "unemployment_prior_year_repayments").unwrap().amount,
            amount
        );
        assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
        d.often_missed.unprepared = vec![RareSituation::TaxTreaty.into()];
        unsupported(&d, "Situations people often miss");
    }
}

#[test]
fn prior_unemployment_review_combines_net_social_security_repayments() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(social_security_net(-100000, 0)));
    let mut spouse = income(social_security_net(50000, 0));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    add(&mut d, prior_unemployment_repayment(250000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "social_security_net_repayments"), 500);
    assert_eq!(value(&r, "prior_income_repayment_review_total"), 3000);
    assert_eq!(value(&r, "agi"), 100000);
    assert_eq!(r.tax, Some(764300));
    let FederalIncome::GovernmentPayment { data } = &mut d.income[3].income else { panic!() };
    data.repayments[0].amount += 1;
    unsupported(&d, "Earlier-income repayments");
}

#[test]
fn prior_unemployment_threshold_excludes_current_year_netting() {
    let mut d = complete(FederalStatus::Single);
    let mut e = government_unemployment(10000, &[10000], 0);
    let FederalIncome::GovernmentPayment { data } = &mut e.income else { panic!() };
    data.repayments.push(UnemploymentRepayment { received_year: 2025, amount: 300000 });
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_repaid"), 10000);
    assert_eq!(value(&r, "prior_income_repayment_review_total"), 3000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(r.tax, Some(502300));
}

#[test]
fn same_year_unemployment_repayment_nets_on_schedule_1_line_7() {
    // The default single wage is $60,000 with $6,500 withheld (`complete`).
    // Add a 1099-G with $6,000 gross unemployment, a $1,000 repayment made
    // in 2026 (the same year received) and $600 withheld. Schedule 1 line 7
    // nets the same-year repayment directly (gross - repaid = $6,000 -
    // $1,000 = $5,000); there is no separate deduction line for it, so the
    // aggregate `adjustments` (Schedule 1 line 26) stays at $0 and Form 1040
    // line 8 (Schedule 1 line 10) shows only the net $5,000, not the $6,000
    // gross with a matching $1,000 deduction elsewhere.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, government_unemployment(6000, &[1000], 600));
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_gross"), 6000);
    assert_eq!(value(&r, "unemployment_2_repaid"), 1000);
    assert_eq!(value(&r, "unemployment_2_taxable"), 5000);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(return_line(&r, "8"), 500_000);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "25b"), 60_000);
    assert_eq!(return_line(&r, "25d"), 710_000);
}

#[test]
fn unemployment_refund_and_grant_return_matches_hand_computation() {
    // Matrix row 23, unit C23a's typical Form 1099-G case: a single filer
    // born 1985 with one W-2 (box 1 $41,000, box 2 $3,600; boxes 3-6 are
    // `wage()`'s own consistent derivation), a 1099-G reporting $7,200
    // unemployment compensation (box 1) with $720 federal withholding
    // (box 4) and a $600 same-year repayment, a $1,500 taxable grant on
    // that same document (box 6), and a second 1099-G state income-tax
    // refund of $900 (box 2, for tax year 2025) from a year the filer used
    // the standard deduction, so none of it is taxable (Pub 525).
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19_850_615;
    d.income[0] = wage(1, Owner::Taxpayer, 4_100_000, 360_000); // box1 $41,000.00, box2 $3,600.00
    let mut benefits = government_unemployment(7_200, &[600], 720);
    let FederalIncome::GovernmentPayment { data } = &mut benefits.income else { panic!() };
    data.form.taxable_grants = Some(150_000); // $1,500.00 (box 6)
    // 2026-10-23: the stated Schedule 1 grant is personal; activity grants require their activity.
    data.grant_use = Some(GrantUse::You);
    add(&mut d, benefits);
    add(&mut d, income(refund_income(90_000, 0, TaxRefundHistory::NotDeducted)));
    let r = supported(&d);

    // Schedule 1, hand-computed from Pub 525 / i1040gi and the App's own
    // Schedule 1 line 7/8z fields (`FederalUnemployment`):
    // Line 1 (taxable state/local refund): $0. The filer used the standard
    // deduction the year the tax was paid, so under the Pub 525 recovery
    // rule no part of the $900 refund is a tax-benefit recovery.
    assert_eq!(value(&r, "taxable_state_tax_refund"), 0);
    // Line 7 (unemployment compensation): $7,200 gross box 1 minus the
    // $600 same-year repayment = $6,600 net.
    assert_eq!(value(&r, "unemployment_2_taxable"), 6_600);
    // Line 8z (other income - here, only the box 6 taxable grant): $1,500.
    assert_eq!(value(&r, "taxable_grants_1099g"), 1_500);
    // Line 9 (total other income, sum of lines 1-8z) and line 10 (combined
    // with lines 2-7, entered on Form 1040 line 8): $0 + $6,600 + $1,500 =
    // $8,100; there is no other Schedule 1 income item on this return.
    // Line 26 (total adjustments to income, entered on Form 1040 line 10):
    // $0 — no adjustment is entered on this return.

    // Form 1040, hand-computed:
    // Line 1a: W-2 box 1 wages, $41,000.
    assert_eq!(return_line(&r, "1a"), 41_000 * 100);
    // Line 8: additional income = Schedule 1 line 10 above = $8,100.
    assert_eq!(return_line(&r, "8"), 8_100 * 100);
    // Line 9: total income = $41,000 wages + $8,100 additional income =
    // $49,100.
    assert_eq!(return_line(&r, "9"), 49_100 * 100);
    // Line 11: AGI = $49,100 total income - $0 adjustments = $49,100.
    assert_eq!(return_line(&r, "11a / 11b"), 49_100 * 100);
    assert_eq!(value(&r, "agi"), 49_100);
    // Line 12e: 2026 single standard deduction, $16,100 (not a senior
    // (born 1985), not blind, not a dependent).
    assert_eq!(return_line(&r, "12e"), 16_100 * 100);
    // Line 15: taxable income = $49,100 - $16,100 = $33,000.
    assert_eq!(return_line(&r, "15"), 33_000 * 100);
    // Line 16: regular tax from the September 2026 draft Tax Table, Single.
    // $33,000 falls in the $33,000-$33,050 row, midpoint $33,025: 10% of
    // the first $12,400 ($1,240.00) plus 12% of the next $20,625
    // ($33,025 - $12,400 = $20,625, tax $2,475.00) = $3,715.00.
    assert_eq!(return_line(&r, "16"), 3_715 * 100);
    // Line 24a: total tax is unchanged; no AMT, credits or additional
    // taxes apply to this return.
    assert_eq!(return_line(&r, "24a"), 3_715 * 100);
    assert_eq!(r.tax, Some(3_715 * 100));
    // Line 25a: W-2 withholding, $3,600.
    assert_eq!(return_line(&r, "25a"), 3_600 * 100);
    // Line 25b: 1099 withholding — only the unemployment document's $720
    // box 4 (the refund document withheld nothing).
    assert_eq!(return_line(&r, "25b"), 720 * 100);
    // Line 25d: $3,600 + $720 = $4,320.
    assert_eq!(return_line(&r, "25d"), 4_320 * 100);
    // Line 33: total payments and refundable credits, $4,320 (no
    // refundable credits apply to this return).
    assert_eq!(return_line(&r, "33"), 4_320 * 100);
    assert_eq!(r.payments, Some(4_320 * 100));
    // Line 34: overpayment = $4,320 payments - $3,715 tax = $605 refund.
    assert_eq!(return_line(&r, "34"), 605 * 100);
    assert_eq!(r.refund, Some(605 * 100));
    // Line 37: no balance due.
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn later_year_unemployment_repayment_follows_the_claim_of_right_path() {
    // Out of this card's scope (box 5 RTAA, box 8, box 9 and the box 10
    // medical-leave exclusion are separately reported, not implemented
    // here): a prior-year unemployment repayment is not a Form 1099-G box
    // item at all, and the App already has a claim-of-right path for it
    // (Pub 525; `src/claim_repayment.rs`) that predates this unit. A
    // repayment above the $3,000 combined earlier-year review threshold
    // (unemployment, railroad and net Social Security repayments) is
    // refused until `FederalDeductions.repayment` supplies the
    // Automatic/Deduction/Credit comparison; this test only confirms that
    // existing behavior for a $4,000 (above $3,000) later-year repayment.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        prior_unemployment_repayment(400_000, Owner::Taxpayer), // $4,000.00, above the $3,000 threshold
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        r.issues.iter().any(|i| i.message.contains("Combined earlier-year unemployment")),
        "{r:?}"
    );
}
