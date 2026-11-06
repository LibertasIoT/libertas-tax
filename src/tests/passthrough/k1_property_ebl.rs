use crate::tests::support::assert::k1_consolidated::k1_supported as supported;
// Section461(l) after the actual K1 activity limits; independent expected
// business nets, qualified deferral and deductible property history.
use super::*;
use crate::tests::passthrough::k1_sale_return::{draft, sources};

fn capacity(source: &mut FederalIncome, amount: i64) {
    let risk = ScorpRiskStart::Detailed { capacity: amount * 100, profit: 0, loss: 0 };
    match source {
        FederalIncome::Partnership { data } => {
            data.regular.basis.opening = Some(amount * 100);
            data.regular.risk.start = risk;
            data.alternative = Some(data.regular.clone());
        }
        FederalIncome::Scorp { data } => {
            let mut source_edit = edit_scorp(data);
            let data = &mut *source_edit;

            data.opening = Some(amount * 100);
            data.risk.start = risk;
            data.amt = Some(amt_records(data));
            for sale in &mut data.section179_sales {
                sale.alternative =
                    crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
            }
        }
        _ => unreachable!(),
    }
}
fn with_wages(source: FederalIncome) -> FederalDraft {
    let (mut d, _) = draft(source);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Taxpayer, 8_000_000, 0));
    d
}
fn ordinary_source(mut source: FederalIncome, ordinary: i64) -> FederalIncome {
    k1_qbi_statement(&mut source, ordinary, false);
    if let FederalIncome::Scorp { data } = &mut source {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.loss_qbi = sole_qbi_loss(-ordinary, 1_000_000, -ordinary).loss_qbi;
        data.qbi_statement.as_mut().unwrap().income = 0;
    }
    capacity(&mut source, 1_000_000);
    source
}

fn property_source(mut source: FederalIncome, amount: i64) -> FederalIncome {
    k1_qbi_statement(&mut source, 0, false);
    let sales = match &mut source {
        FederalIncome::Partnership { data } => &mut data.section179_sales,
        FederalIncome::Scorp { data } => &mut data.statements.section179_sales,
        _ => unreachable!(),
    };
    sales[0].regular.as_mut().unwrap().price = 10_000_000;
    sales[0].regular.as_mut().unwrap().cost = (100_000 + amount) * 100;
    sales[0].regular.as_mut().unwrap().depreciation = 0;
    sales[0].regular.as_mut().unwrap().section179 = 0;
    capacity(&mut source, 1_000_000);
    source
}
#[test]
fn k1_property_ebl_ordinary_sale_gain_offsets_operating_loss_in_each_tax_system() {
    for mut source in sources(-400_000, 0, 0, 30_000) {
        k1_qbi_statement(&mut source, -400_000, false);
        if let FederalIncome::Scorp { data } = &mut source {
            let mut source_edit = edit_scorp(data);
            let data = &mut *source_edit;
            data.loss_qbi = sole_qbi_loss(400_000, 1_000_000, 400_000).loss_qbi;
            data.qbi_statement.as_mut().unwrap().income = 0;
        }
        capacity(&mut source, 1_000_000);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000
            }
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;

                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000
            }
            _ => unreachable!(),
        }
        let d = with_wages(source);
        let r = supported(&d);
        // Current loss400 less ordinary sale8 = loss392; threshold256.
        // AMT ordinary sale6 => loss394. Wages do not absorb business losses.
        assert_eq!(value(&r, "excess_business_loss"), 136_000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 138_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 256_000);
        assert_eq!(value(&r, "agi"), -176_000);
        assert_eq!(value(&r, "current_net_operating_loss"), 176_000);
        // Form172's carryover worksheet adds the separately disallowed EBL.
        assert_eq!(value(&r, "nol_carryforward"), 312_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
#[test]
fn k1_property_ebl_partner_property_only_keeps_partial_qualification_and_independent_amt() {
    let mut p = ordinary_partner_source(0, true);
    p.qbi = Some(PartnerQbiStatement { section1231_qbi: -20_000_000, ..partner_qbi_statement(0) });
    p.regular.section1231 = Some(k1_section1231(-400_000, 0));
    let mut source = FederalIncome::Partnership { data: p };
    capacity(&mut source, 500_000);
    let FederalIncome::Partnership { data } = &mut source else { unreachable!() };
    data.alternative.as_mut().unwrap().section1231.as_mut().unwrap().net = -30_000_000;
    let d = with_wages(source);
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 144_000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256_000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 256_000);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_ebl_short_sale_loss_increases_disallowance_and_total_nol() {
    for source in sources(-400_000, 0, 0, 20_000) {
        let mut source = ordinary_source(source, -400_000);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].acquired = 20260301;
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;

                data.section179_sales[0].acquired = 20260301;
                drop(source_edit);
                capacity(&mut source, 1_000_000);
            }
            _ => unreachable!(),
        }
        let d = with_wages(source);
        let r = supported(&d);
        // Adjusted basis22 less proceeds20 produces ordinary loss2.
        assert_eq!(value(&r, "excess_business_loss"), 146_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 256_000);
        assert_eq!(value(&r, "current_net_operating_loss"), 176_000);
        assert_eq!(value(&r, "nol_carryforward"), 322_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn k1_property_ebl_sale_loss_uses_allowed_basis_and_risk_before_form461() {
    for source in sources(0, 0, 0, 30_000) {
        for limit in [250_000, 300_000, 500_000] {
            let mut source = property_source(source.clone(), 400_000);
            capacity(&mut source, limit);
            let d = with_wages(source);
            let r = supported(&d);
            let allowed = limit.min(400_000);
            assert_eq!(value(&r, "excess_business_loss"), (allowed - 256_000).max(0));
            assert_eq!(value(&r, "section1231_2026_next_lookback"), allowed.min(256_000));
            assert_eq!(value(&r, "qbi_loss_carryforward"), allowed.min(256_000));
            assert_eq!(value(&r, "nol_carryforward"), allowed - 80_000);
            assert_eq!(estimate(&restart(d).draft), r);
        }
    }
}

#[test]
fn k1_property_ebl_multiple_entity_property_losses_share_one_threshold() {
    let [partner, scorp] = sources(0, 0, 0, 30_000);
    let mut d = with_wages(property_source(partner, 200_000));
    add(&mut d, income(property_source(scorp, 200_000)));
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 144_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 256_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256_000);
    assert_eq!(value(&r, "nol_carryforward"), 320_000);
    d.income.reverse();
    let reversed = supported(&d);
    for key in [
        "excess_business_loss",
        "qbi_loss_carryforward",
        "section1231_2026_next_lookback",
        "nol_carryforward",
    ] {
        assert_eq!(value(&reversed, key), value(&r, key));
    }
    assert_eq!(estimate(&restart(d).draft), reversed);
}

#[test]
fn k1_property_ebl_passive_ordinary_sale_loss_is_not_an_allowed_business_loss() {
    for source in sources(-400_000, 0, 0, 20_000) {
        let mut source = ordinary_source(source, -400_000);
        match &mut source {
            FederalIncome::Partnership { data } => {
                let Some(PartnerParticipation::General { facts }) = &mut data.participation else {
                    unreachable!()
                };
                facts.minutes = 0;
                data.section179_sales[0].acquired = 20260301;
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;

                data.participation.as_mut().unwrap().minutes = 0;
                data.section179_sales[0].acquired = 20260301;
                drop(source_edit);
                capacity(&mut source, 1_000_000);
            }
            _ => unreachable!(),
        }
        let r = supported(&with_wages(source));
        assert_eq!(value(&r, "excess_business_loss"), 0);
        assert_eq!(value(&r, "agi"), 80_000);
        assert_eq!(value(&r, "nol_carryforward"), 0);
    }
}

#[test]
fn k1_property_ebl_shareholder_loan_gain_stays_nonbusiness() {
    let mut source = ordinary_source(sources(-400_000, 0, 0, 30_000)[1].clone(), -400_000);
    let FederalIncome::Scorp { data } = &mut source else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.loans.push(ScorpLoan {
            kind: ScorpLoanKind::OpenAccount,
            repayment_term: ScorpNoteHolding::NoRepayment,
            principal: 500_000,
            basis: 300_000,
            advances: 0,
            repayments: 500_000,
        });
    }
    capacity(&mut source, 1_000_000);
    let d = with_wages(source);
    let r = supported(&d);
    assert_eq!(value(&r, "scorp_ordinary_loan_gain"), 2_000);
    assert_eq!(value(&r, "excess_business_loss"), 136_000);
    assert_eq!(value(&r, "agi"), -174_000);
    // Nonbusiness loan gain is absorbed by the standard deduction on Form172.
    assert_eq!(value(&r, "current_net_operating_loss"), 176_000);
    assert_eq!(value(&r, "nol_carryforward"), 312_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_ebl_net_equality_cannot_prove_gross_loss_qualification() {
    use crate::business_rules::QbiActivity;
    let source = ordinary_source(sources(-400_000, 0, 0, 30_000)[0].clone(), -400_000);
    let mut d = with_wages(source);
    add(
        &mut d,
        income(FederalIncome::Partnership { data: ordinary_partner_source(100_000, true) }),
    );
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(&d, 0, &mut result, false);
    let mut activities = vec![QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(-292_000 * 100),
        owner: None,
        income: -292_000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let original = activities.clone();
    // Qualified net equals business net, but different gross source fractions
    // can produce that equality. No candidate mutation or inferred allocation.
    assert!(
        crate::k1_excess::qualified_deferral(
            &d,
            &pool,
            &mut activities,
            -292_000,
            36_000,
            false,
            (0, FederalStatus::Single)
        )
        .is_none()
    );
    assert!(activities == original);
}

#[test]
fn k1_property_ebl_incomplete_pool_cannot_change_qualified_loss() {
    use crate::business_rules::QbiActivity;
    let d = with_wages(property_source(sources(0, 0, 0, 30_000)[0].clone(), 400_000));
    let mut result = FederalResult::empty(d.revision);
    let mut pool = crate::extended_rules::rentals_for(&d, 0, &mut result, false);
    pool.inputs_complete = false;
    let mut activities = vec![QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(-400_000 * 100),
        owner: None,
        income: -400_000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let original = activities.clone();
    assert!(
        crate::k1_excess::qualified_deferral(
            &d,
            &pool,
            &mut activities,
            -400_000,
            144_000,
            true,
            (0, FederalStatus::Single)
        )
        .is_none()
    );
    assert!(activities == original);
}
