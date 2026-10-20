use super::*;

use crate::tests::support::builders::injured_credit::pre_gbc_credit_sources::fixture;

#[test]
fn unit13_exclusive_saver_native_money_without_business_carry() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let d = fixture(owner, reverse);
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 60_000);
            assert_eq!(value(&r, "taxable_income"), 27_800);
            assert_eq!(value(&r, "saver_credit"), 200);
            assert_eq!(return_line(&r, "16"), 284_300);
            assert_eq!(r.tax, Some(264_300));
            assert_eq!(r.refund, Some(1_735_700));
            assert!(d.injured_spouse.is_none());
            assert_eq!(estimate(&restart(d).draft), r);
        }
    }
}

#[test]
fn unit13_exclusive_saver_reaches_actual_pre_gbc_without_claim() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for filer in [CarryFiler::You, CarryFiler::Spouse] {
            for reverse in [false, true] {
                let mut d = fixture(owner, reverse);
                d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
                    carryforwards: vec![BusinessCreditCarryforward {
                        credit: BusinessCredit::WorkOpportunity,
                        origin_year: 2025,
                        amount: 1_000_000,
                        passive: None,
                        filer,
                        research_source: None,
                    }],
                    carrybacks: vec![],
                    spouse_without_credits: false,
                    credits: vec![],
                });
                let r = supported(&d);
                assert_eq!(value(&r, "agi"), 60_000);
                assert_eq!(value(&r, "taxable_income"), 27_800);
                assert_eq!(value(&r, "saver_credit"), 200);
                assert_eq!(return_line(&r, "16"), 284_300);
                let used = value(&r, "general_business_credit");
                assert!(used > 0 && used <= 2643);
                assert_eq!(r.tax, Some((2643 - used) * 100));
                assert_eq!(r.refund, Some((17_357 + used) * 100));
                assert!(d.injured_spouse.is_none());
                assert_eq!(estimate(&restart(d).draft), r);
            }
        }
    }
}
