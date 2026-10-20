// Source-complete current NUA and retained NIIT use share one native worksheet.
use super::*;
use crate::tests::cross::interview::{accepted, open_topic};

#[test]
fn unit13_niit_mixed_zero_adjustment_retains_real_prior_use_choice() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let r = assert_mixed_niit_money(&mixed_niit_return(Owner::Taxpayer, injured, false));
        let row = r.injured_spouse.as_ref().unwrap().required.iter().find(|row| row.component == "niit_retained_difference_used").expect("nonzero prior use must not disappear when net current-plus-prior adjustment is zero");
        assert_eq!(row.joint, 500_000);
        assert_eq!(
            (row.minimum, row.maximum),
            if injured == Owner::Taxpayer { (0, 400_000) } else { (100_000, 500_000) }
        );
    }
}

#[test]
fn unit13_niit_mixed_zero_adjustment_save_restart_and_owner_axes() {
    for nua_owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                for taxpayer_used in [0, 1000, 4000] {
                    let mut draft = mixed_niit_return(nua_owner, injured, reversed);
                    let before = assert_mixed_niit_money(&draft);
                    let used = if injured == Owner::Taxpayer {
                        taxpayer_used
                    } else {
                        5000 - taxpayer_used
                    };
                    draft.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                        component: "niit_retained_difference_used".into(),
                        injured: used * 100,
                    });
                    let mut session = restart(draft);
                    for topic in [
                        TaxSection::Wages,
                        TaxSection::InterestDividends,
                        TaxSection::InvestmentSales,
                    ] {
                        let request = open_topic(&mut session, topic);
                        accepted(&send(&mut session, request));
                    }
                    let request = complete_niit_allocation(&mut session);
                    accepted(&send(&mut session, request));
                    let draft = roundtrip_native_draft(session.draft);
                    let mut reopened = restart(draft);
                    let request = complete_niit_allocation(&mut reopened);
                    accepted(&send(&mut reopened, request));
                    let after = assert_mixed_niit_money(&reopened.draft);
                    assert_eq!(before.return_lines, after.return_lines);
                    // Annual capital ownership is still the independent -1500/-1500 answer.
                    assert_eq!(
                        [column(&after, "13b", "b"), column(&after, "13b", "c")],
                        [8500, 8500]
                    );
                    let nii = if nua_owner == Owner::Taxpayer {
                        [3500 + taxpayer_used, 13500 - taxpayer_used]
                    } else {
                        [8500 + taxpayer_used, 8500 - taxpayer_used]
                    };
                    let niit = nii.map(|amount| amount * 38 / 1000);
                    let i = usize::from(injured == Owner::Spouse);
                    assert_eq!(
                        [column(&after, "18", "b"), column(&after, "18", "c")],
                        [niit[i], niit[1 - i]]
                    );
                }
            }
        }
    }
}

#[test]
fn unit13_niit_mixed_bounds_require_separate_native_axes_and_exact_sources() {
    use crate::injured_spouse::niit_retained::RetainedNiitUse;
    let mut source = RetainedNiitUse {
        prior_dollars: 10_000,
        used_dollars: 0,
        next_dollars: 5000,
        current_components_known_zero: false,
    };
    assert_eq!(source.bounds([4000, 6000], 0), None);
    assert_eq!(source.bounds_with_current([4000, 6000], 0, 5000), Some((5000, 0, 4000)));
    assert_eq!(source.bounds_with_current([4000, 6000], 1, 5000), Some((5000, 1000, 5000)));
    assert_eq!(source.bounds_with_current([4000, 5999], 0, 5000), None);
    assert_eq!(source.bounds_with_current([-1, 10_001], 0, 5000), None);
    assert_eq!(source.bounds_with_current([4000, 6000], 0, 0), None);
    assert_eq!(source.bounds_with_current([4000, 6000], 0, -5000), None);
    source.next_dollars = 5001;
    assert_eq!(source.bounds_with_current([4000, 6000], 0, 5000), None);
    source.next_dollars = 10_001;
    assert_eq!(source.bounds_with_current([4000, 6000], 0, 5000), None);
    source.next_dollars = 0;
    source.used_dollars = 5000;
    assert_eq!(source.bounds_with_current([4000, 6000], 0, 5000), Some((10_000, 4000, 4000)));
}
