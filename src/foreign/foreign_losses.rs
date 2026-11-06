#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::FederalResult;
use crate::kernel::phase;
use crate::kernel::ratio::{allocate_remaining, positive};
use crate::line_emission::line;
use alloc::format;

// Section1.904-4(c)(2)(ii): passive loss groups reduce other passive
// groups before high-tax classification, not after category aggregation.
// Index0 is employment/general income and never absorbs losses here.
pub(crate) fn passive_groups(net: &mut [i64; 5], context: &str, r: &mut FederalResult) -> i64 {
    let loss: i64 = net[1..].iter().map(|v| positive(-v)).sum();
    let weights: [i64; 4] = core::array::from_fn(|i| positive(net[i + 1]));
    let available: i64 = weights.iter().sum();
    let absorbed = phase::cap(loss, [available]);
    let reductions = allocate_remaining(absorbed, &weights);
    for i in 0..4 {
        net[i + 1] = weights[i] - reductions[i];
    }
    if loss > 0 {
        line(
            r,
            &format!("foreign_{context}_passive_loss_reallocated"),
            absorbed,
            &["credits.foreign_tax", "deductions"],
            "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol11/pdf/CFR-2025-title26-vol11-sec1-904-4.pdf",
            msg!("EXPLAIN_FOREIGN_N_PASSIVE_LOSS_REALLOCATED"),
        );
    }
    positive(loss - available)
}

// The Form 1116 loss-allocation categories, in slot order: general, passive,
// foreign branch and section 901(j). Only the first three can hold a retained
// account (no credit is allowed for a section 901(j) country), but a loss in
// any of the four is allocated.
pub(crate) const CATEGORY_NAMES: [&str; 4] = ["general", "passive", "branch", "sanctioned"];

pub(crate) struct CurrentLoss {
    pub separate: [i64; 4],
    pub overall: [i64; 4],
    pub domestic: [i64; 4],
    pub domestic_income: i64,
    pub created: bool,
}

// Current-year allocation precedes retained-account recapture (Reg.
// 1.904(f)-1, (f)-2): a separate limitation loss offsets the income of the
// other categories pro rata, an overall foreign loss offsets U.S. income, and
// a U.S. loss offsets the foreign income pro rata.
pub(crate) fn current_categories(
    net: &mut [i64; 4],
    worldwide: i64,
    context: &str,
    r: &mut FederalResult,
) -> CurrentLoss {
    let domestic = worldwide - net.iter().sum::<i64>();
    let mut loss = net.map(|v| positive(-v));
    *net = net.map(positive);
    let mut separate = [0; 4];
    for i in 0..4 {
        if loss[i] == 0 {
            continue;
        }
        let others: [i64; 4] = core::array::from_fn(|j| if j == i { 0 } else { net[j] });
        let available: i64 = others.iter().sum();
        separate[i] = phase::cap(loss[i], [available]);
        for (j, share) in allocate_remaining(separate[i], &others).into_iter().enumerate() {
            net[j] = crate::kernel::carry::native_balance(net[j], share);
        }
        loss[i] = crate::kernel::carry::native_balance(loss[i], separate[i]);
    }
    let loss_total: i64 = loss.iter().sum();
    let overall = allocate_remaining(phase::cap(loss_total, [positive(domestic)]), &loss);
    let domestic_loss =
        allocate_remaining(phase::cap(positive(-domestic), [net.iter().sum()]), net);
    let mut created = false;
    for i in 0..4 {
        net[i] = crate::kernel::carry::native_balance(net[i], domestic_loss[i]);
        let category = CATEGORY_NAMES[i];
        for (kind, amount, detail) in [
            ("separate_loss", separate[i], msg!("EXPLAIN_FOREIGN_N_N_N_NEW_SEPARATE_LOSS")),
            (
                "overall_foreign_loss",
                overall[i],
                msg!("EXPLAIN_FOREIGN_N_N_N_NEW_OVERALL_FOREIGN_LOSS"),
            ),
            (
                "overall_domestic_loss",
                domestic_loss[i],
                msg!("EXPLAIN_FOREIGN_N_N_N_NEW_OVERALL_DOMESTIC_LOSS"),
            ),
        ] {
            if amount > 0 {
                created = true;
                line(
                    r,
                    &format!("foreign_{context}_{category}_{kind}_new"),
                    amount,
                    &["credits.foreign_tax", "deductions"],
                    "https://www.irs.gov/publications/p514",
                    detail,
                );
            }
        }
    }
    CurrentLoss {
        separate,
        overall: [overall[0], overall[1], overall[2], overall[3]],
        domestic: [domestic_loss[0], domestic_loss[1], domestic_loss[2], domestic_loss[3]],
        domestic_income: positive(domestic - overall.iter().sum::<i64>()),
        created,
    }
}
