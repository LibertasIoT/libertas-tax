// 2026-10-14 independent R3/R4 control: a known noninvestment Form4797
// component cannot erase the unrelated, actually eligible Form4952 source.
// Native amounts below are annual Dollars; stored monetary inputs are Cents.
use super::*;

#[test]
fn unit13_noninvestment_property_preserves_native_money_and_public_save() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let d = noninvestment_election_fixture(injured, reverse);
            let r = assert_noninvestment_election_money(&d);
            let mut ordinary = d.clone();
            ordinary.injured_spouse = None;
            let plain = assert_noninvestment_election_money(&ordinary);
            assert_eq!((r.tax, r.refund), (plain.tax, plain.refund));
            let accepted = save_noninvestment_election_source(d);
            assert_eq!(
                accepted.screening.as_ref().unwrap().installment_payments[0].owner,
                Owner::Taxpayer
            );
        }
    }
}

#[test]
fn unit13_noninvestment_property_does_not_hide_actual_elected_capital_owner() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let accepted = save_noninvestment_election_source(noninvestment_election_fixture(
                injured, reverse,
            ));
            assert_noninvestment_election_taxes(
                &assert_noninvestment_election_money(&accepted),
                injured,
            );
        }
    }
}

#[test]
fn unit13_noninvestment_property_election_reaches_genuine_pre_gbc_without_claim() {
    for (filer, allowed) in [(CarryFiler::You, 6102), (CarryFiler::Spouse, 341)] {
        for reverse in [false, true] {
            let mut d = save_noninvestment_election_source(noninvestment_election_fixture(
                Owner::Taxpayer,
                reverse,
            ));
            d.injured_spouse = None;
            d.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
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
            // Real F caller invokes project_sources before calculate(3800).
            // 7026/7419 rounds9470/530 basis points; joint6443 cap gives
            // 6102/341 after its existing single native allowance rounding.
            assert_eq!(value(&r, "agi"), 93_000);
            assert_eq!(value(&r, "taxable_income"), 60_800);
            assert_eq!(return_line(&r, "16"), 644_300);
            assert_eq!(value(&r, "general_business_credit"), allowed);
            assert_eq!(r.tax, Some((6443 - allowed) * 100));
            assert_eq!(r.refund, Some((30_000 - 6443 + allowed) * 100));
            assert_eq!(estimate(&restart(d).draft), r);
        }
    }
}
