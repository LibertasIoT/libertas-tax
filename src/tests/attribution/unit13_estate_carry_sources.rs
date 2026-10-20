use super::*;
use crate::tests::support::builders::injured_income::estate_carry_sources::fixture;
use crate::tests::support::client::injured_income_source_roundtrip;

// The fiduciary supplies two independent source axes: ScheduleD capital
// carry and the part excluded from Form8960. The beneficiary owns both.
#[test]
fn unit13_estate_carry_uses_beneficiary_on_regular_and_niit_axes_after_restart() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let draft = fixture(injured, &[(owner, 2_000_000, Some(1_000_000))]);
            let before = supported(&draft);
            let saved = injured_income_source_roundtrip(draft, TaxSection::PassThrough);
            let result = supported(&saved);
            assert_eq!(result.return_lines, before.return_lines);
            assert_eq!(value(&result, "capital_gain"), 40_000);
            assert_eq!(value(&result, "niit_disposition_adjustment"), 10_000);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 50_000);
            assert_eq!(value(&result, "net_investment_income_tax"), 1_900);
            let (income, tax) = if owner == injured { (10_000, 760) } else { (30_000, 1_140) };
            assert_eq!(column(&result, "13b", "b"), income);
            assert_eq!(column(&result, "13b", "c"), 40_000 - income);
            let payroll = if injured == Owner::Taxpayer { 1_350 } else { 0 };
            assert_eq!(column(&result, "18", "b"), tax + payroll);
            assert_eq!(column(&result, "18", "c"), 1_900 - tax + 1_350 - payroll);
            assert!(
                result
                    .review_notes
                    .iter()
                    .all(|n| n.display.key != "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
            );
            let mut plain = saved.clone();
            plain.injured_spouse = None;
            assert_eq!(estimate(&plain).tax, result.tax);
        }
    }
}

#[test]
fn unit13_estate_carries_pool_regular_cents_but_keep_niit_statement_boundaries() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for cents in [49, 50] {
            let mut draft = fixture(
                injured,
                &[(Owner::Taxpayer, cents, Some(cents)), (Owner::Spouse, cents, Some(cents))],
            );
            let before = supported(&draft);
            draft.income.reverse();
            let result = supported(&draft);
            assert_eq!(result.return_lines, before.return_lines);
            // Both regular49c and50c sources total one reported Dollar.
            // Each NIIT statement rounds independently:0+0 versus1+1.
            assert_eq!(value(&result, "capital_gain"), 59_999);
            assert_eq!(
                value(&result, "niit_disposition_adjustment"),
                if cents == 49 { 0 } else { 2 }
            );
            assert_eq!(
                value(&result, "form_8960_net_investment_income"),
                if cents == 49 { 59_999 } else { 60_001 }
            );
            let injured_carry = i64::from(cents == 50);
            assert_eq!(column(&result, "13b", "b"), 30_000 - injured_carry);
            assert_eq!(column(&result, "13b", "c"), 29_999 + injured_carry);
            assert_eq!(column(&result, "18", "a"), 2_280 + 1_350);
        }
    }
}

#[test]
fn unit13_estate_missing_niit_character_keeps_joint_return_and_known_capital() {
    let draft = fixture(Owner::Taxpayer, &[(Owner::Spouse, 2_000_000, None)]);
    let result = supported(&draft);
    assert_eq!(value(&result, "capital_gain"), 40_000);
    assert_eq!(column(&result, "13b", "b"), 30_000);
    assert_eq!(column(&result, "13b", "c"), 10_000);
    assert!(
        result.review_notes.iter().any(|n| n.display.key == "WARN_ESTATE_CAPITAL_NIIT_LEFT_OUT")
    );
    assert!(
        result
            .review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert!(result.lines.iter().all(|l| l.id != "form_8379_injured_refund_estimate"));
}

#[test]
fn unit13_estate_loss_floor_and_retained_difference_keep_their_actual_owner() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let draft = fixture(owner, &[(owner, 7_000_000, Some(4_000_000))]);
        let result = supported(&draft);
        assert_eq!(value(&result, "capital_gain"), -3_000);
        assert_eq!(value(&result, "niit_disposition_adjustment"), 33_000);
        assert_eq!(value(&result, "niit_capital_carry_adjustment"), 7_000);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 30_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 1_140);
        assert_eq!(column(&result, "13b", "b"), -33_000);
        assert_eq!(column(&result, "13b", "c"), 30_000);
        let payroll = if owner == Owner::Taxpayer { 1_350 } else { 0 };
        assert_eq!(column(&result, "18", "b"), payroll);
        assert_eq!(column(&result, "18", "c"), 1_140 + 1_350 - payroll);
        assert_eq!(draft.injured_spouse.as_ref().unwrap().allocations.len(), 1);
    }
}

#[test]
fn unit13_estate_long_carry_preserves_its_beneficiary_collectible_rate_pool() {
    use crate::tests::support::builders::injured_income::estate_carry_sources::long_collectible;
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let draft = long_collectible(injured);
        let result = supported(&draft);
        // The spouse's15000 collectible gain absorbs that beneficiary's
        //10000 inherited long carry. The taxpayer's10000 ordinary long gain
        //does not acquire another person's28% character.
        assert_eq!(value(&result, "capital_gain"), 75_000);
        assert_eq!(value(&result, "schedule_d_line_18"), 5_000);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 75_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 2_850);
        let income = if injured == Owner::Taxpayer { 40_000 } else { 35_000 };
        assert_eq!(column(&result, "13b", "b"), income);
        assert_eq!(column(&result, "13b", "c"), 75_000 - income);
        // 2026 MFS:313900 ordinary tax78634.25 plus10000 at20%=80634;
        //113900 ordinary tax19934 plus5000 at24%(below28%cap)=21134.
        let taxes = if injured == Owner::Taxpayer { [80_634, 21_134] } else { [21_134, 80_634] };
        let explanation = &result
            .lines
            .iter()
            .find(|line| line.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(taxes[0]));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(taxes[1]));
    }
}
