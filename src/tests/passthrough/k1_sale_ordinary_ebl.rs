use crate::tests::support::assert::k1_consolidated::k1_supported as supported;
// Gross ordinary operating/SALE losses retain their own current and original
// qualification through section461(l); gains never change that fraction.
use super::*;

fn sources(ordinary: i64, qualified: i64) -> [FederalIncome; 2] {
    let mut rows = crate::tests::passthrough::k1_sale_return::sources(ordinary, 0, 0, 30_000);
    for row in &mut rows {
        k1_qbi_statement(row, 0, false);
        match row {
            FederalIncome::Partnership { data: p } => {
                p.regular.basis.opening = 100_000_000;
                p.regular.risk.start = capacity(100_000_000);
                p.qbi.as_mut().unwrap().income = -qualified * 100;
                p.alternative = Some(p.regular.clone());
            }
            FederalIncome::Scorp { data: s } => {
                s.opening = 100_000_000;
                s.risk.start = capacity(100_000_000);
                s.loss_qbi = ScorpLossQbi { current: qualified * 100, ..Default::default() };
                s.amt = Some(amt_records(s));
                for sale in &mut s.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
            }
            _ => unreachable!(),
        }
    }
    rows
}
fn capacity(value: i64) -> ScorpRiskStart {
    ScorpRiskStart::Detailed { capacity: value, profit: 0, loss: 0 }
}
fn history(row: &mut FederalIncome, loss: i64, qualified: i64) {
    let vintage = loss_year(2025, loss, qualified, loss, qualified);
    match row {
        FederalIncome::Partnership { data: p } => {
            p.regular.basis.prior_loss = loss * 100;
            p.qbi_records.history =
                Some(PartnerQbiHistory { basis: vec![vintage], risk: vec![], passive: vec![] });
            p.alternative = Some(p.regular.clone());
        }
        FederalIncome::Scorp { data: s } => {
            s.prior_loss = loss * 100;
            s.loss_qbi.history = vec![vintage];
            s.amt = Some(amt_records(s));
            for sale in &mut s.section179_sales {
                sale.alternative =
                    crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
            }
        }
        _ => unreachable!(),
    }
}
fn draft(row: FederalIncome, wages: i64) -> FederalDraft {
    let (mut d, _) = crate::tests::passthrough::k1_sale_return::draft(row);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Taxpayer, wages * 100, 0));
    d
}
fn check(d: FederalDraft, excess: i64, carry: i64, nol: i64) {
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), excess);
    assert_eq!(value(&r, "qbi_loss_carryforward"), carry);
    assert_eq!(value(&r, "nol_carryforward"), nol);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_sale_ordinary_ebl_partial_current_qualification_uses_gross_loss() {
    for row in sources(-400_000, 200_000) {
        // Gross loss400, gain8, EBL136. Qualified200 defers68;
        // qualified gain8 - loss200 + deferral68 leaves loss124.
        check(draft(row, 80_000), 136_000, 124_000, 312_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_retained_loss_keeps_original_qualification() {
    for mut row in sources(-300_000, 150_000) {
        history(&mut row, 100_000, 50_000);
        check(draft(row, 80_000), 136_000, 124_000, 312_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_no_statement_preserves_only_historical_qualification() {
    for mut row in sources(-300_000, 0) {
        history(&mut row, 100_000, 50_000);
        match &mut row {
            FederalIncome::Partnership { data } => data.qbi = None,
            FederalIncome::Scorp { data } => data.qbi_statement = None,
            _ => unreachable!(),
        }
        // No current statement qualifies either current loss or sale gain.
        // Original50 defers136*50/400=17, leaving33.
        check(draft(row, 80_000), 136_000, 33_000, 312_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_current_gain_and_historical_loss_are_separate() {
    for mut row in sources(0, 0) {
        history(&mut row, 400_000, 200_000);
        check(draft(row, 80_000), 136_000, 124_000, 312_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_multiple_gain_rows_do_not_change_loss_fraction() {
    for mut row in sources(-400_000, 200_000) {
        match &mut row {
            FederalIncome::Partnership { data: p } => {
                let second = p.section179_sales[0].clone();
                p.section179_sales.push(second);
                p.alternative = Some(p.regular.clone());
            }
            FederalIncome::Scorp { data: s } => {
                let second = s.section179_sales[0].clone();
                s.section179_sales.push(second);
                s.amt = Some(amt_records(s));
                for sale in &mut s.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
            }
            _ => unreachable!(),
        }
        // Gain16; EBL128; qualified deferral64; QBI loss200-16-64=120.
        check(draft(row, 80_000), 128_000, 120_000, 304_000);
    }
}

#[test]
fn k1_sale_ordinary_ebl_short_sale_loss_joins_gross_losses_without_netting_gain() {
    for mut row in sources(-400_000, 200_000) {
        match &mut row {
            FederalIncome::Partnership { data: p } => {
                let mut second = p.section179_sales[0].clone();
                second.acquired = 20260301;
                second.regular.as_mut().unwrap().price = 2_000_000; // basis22,000: short-held ordinary loss2,000.
                p.section179_sales.push(second);
                p.alternative = Some(p.regular.clone());
            }
            FederalIncome::Scorp { data: s } => {
                let mut second = s.section179_sales[0].clone();
                second.acquired = 20260301;
                second.regular.as_mut().unwrap().price = 2_000_000;
                s.section179_sales.push(second);
                s.amt = Some(amt_records(s));
                for sale in &mut s.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
            }
            _ => unreachable!(),
        }
        // Gross loss402, qualified202, gain8. EBL138 defers
        // round(138000*202000/402000)=69343: QBI carry202000-8000-69343.
        check(draft(row, 80_000), 138_000, 124_657, 314_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_basis_stage_fixes_the_qualified_fraction_first() {
    for mut row in sources(-400_000, 200_000) {
        match &mut row {
            FederalIncome::Partnership { data: p } => {
                p.regular.basis.opening = 30_000_000;
                p.alternative = Some(p.regular.clone());
            }
            FederalIncome::Scorp { data: s } => {
                s.opening = 30_000_000;
                s.amt = Some(amt_records(s));
                for sale in &mut s.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
            }
            _ => unreachable!(),
        }
        // Basis300+gain8 allows308 loss/154 qualified; EBL44 defers22.
        check(draft(row, 80_000), 44_000, 124_000, 220_000);
    }
}
#[test]
fn k1_sale_ordinary_ebl_amt_uses_its_own_allowed_sale_amount() {
    for mut row in sources(-400_000, 200_000) {
        match &mut row {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().price =
                    2_800_000;
            }
            FederalIncome::Scorp { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().price =
                    2_800_000;
            }
            _ => unreachable!(),
        }
        let d = draft(row, 80_000);
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss"), 136_000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 138_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 124_000);
        assert_eq!(value(&r, "nol_carryforward"), 312_000);
        assert_eq!(value(&r, "amt_nol_carryforward"), 314_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
#[test]
fn k1_sale_ordinary_ebl_current_sstb_phase_does_not_rephase_original_history() {
    for mut row in sources(-300_000, 150_000) {
        history(&mut row, 100_000, 50_000);
        match &mut row {
            FederalIncome::Partnership { data } => data.qbi.as_mut().unwrap().service = true,
            FederalIncome::Scorp { data } => data.qbi_statement.as_mut().unwrap().service = true,
            _ => unreachable!(),
        }
        let d = draft(row, 900_000);
        let r = supported(&d);
        // Current loss and gain are fully phased out. Original qualified50
        // defers17 of EBL136, leaving33; no current-year NOL beyond EBL.
        assert_eq!(value(&r, "qbi_loss_carryforward"), 33_000);
        assert_eq!(value(&r, "excess_business_loss"), 136_000);
        assert_eq!(value(&r, "nol_carryforward"), 136_000);
        assert_eq!(value(&r, "agi"), 644_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
