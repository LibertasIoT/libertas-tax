#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{allocate_remaining, dollars, fraction};
use crate::{FederalResult, ForeignCreditCategory};
use alloc::{format, vec::Vec};
use libertas_macros::*;

/// Retained foreign-credit loss accounts for one tax system
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignLossSystem {
    /// Overall foreign loss balance at the end of 2025
    /// Loss in this category previously used against domestic income, not an unused tax credit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub overall: i64,
    /// Separate limitation loss balance at the end of 2025
    /// Loss in this category previously used against the other category: general to passive, or passive to general. Use the unrecovered balance after all prior adjustments.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub separate: i64,
    /// Overall domestic loss balance for this foreign category at the end of 2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub domestic: i64,
    /// Additional overall foreign loss recapture elected for 2026
    /// Normally zero. Enter your elected extra amount above the App's statutory recapture calculation, not a total computed recapture. The App checks remaining account balance and same-category income. This annual election requires a statement with the return; the App does not file it.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub additional_recapture: i64,
}

/// Foreign-credit loss accounts for one income category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignLossAccount {
    /// Category that incurred the foreign loss or received the domestic-loss offset
    pub category: ForeignCreditCategory,
    /// Regular-tax account records
    pub regular: ForeignLossSystem,
    /// Separate AMT account records
    pub amt: ForeignLossSystem,
}

/// Prior foreign-credit loss accounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignLossHistory {
    /// No outstanding loss accounts in either tax system
    None,
    /// Retained loss accounts
    Retained {
        /// Closing 2025 account balances
        /// Enter each category once from the statement of loss-account balances attached to your 2025 Form 1116. An omitted category has no balance or additional election. The App applies the 50% domestic-loss recapture limit; the election to recapture more of a pre-2018 domestic loss is not offered.
        // Fixed three-variant ForeignCreditCategory catalog; valid requires unique categories.
        #[libertas_size(min = 1, max = 3)]
        accounts: Vec<ForeignLossAccount>,
        /// Every balance arose on your own returns, or on joint returns with the spouse on this joint return
        /// A balance from a joint return with someone you are not filing jointly with now needs an allocation between you, which this estimate does not figure; No leaves these loss accounts out with a note, which can make the tax higher or lower.
        same_filing_unit: bool,
    },
}
impl ForeignLossHistory {
    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Retained { .. })
    }
    pub(crate) fn has_amt_activity(&self) -> bool {
        match self {
            Self::None => false,
            Self::Retained { accounts, .. } => accounts.iter().any(|a| {
                let s = &a.amt;
                [s.overall, s.separate, s.domestic, s.additional_recapture]
                    .into_iter()
                    .any(|v| v != 0)
            }),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Retained { accounts, .. } => {
                !accounts.is_empty()
                    && accounts.iter().enumerate().all(|(i, a)| {
                        !accounts[..i].iter().any(|p| p.category == a.category)
                            && [&a.regular, &a.amt].into_iter().all(|s| {
                                [s.overall, s.separate, s.domestic, s.additional_recapture]
                                    .into_iter()
                                    .all(|v| (0..=100000000000).contains(&v))
                            })
                    })
                    && [false, true].into_iter().all(|amt| {
                        accounts
                            .iter()
                            .filter(|a| if amt { a.amt.separate } else { a.regular.separate } > 0)
                            .count()
                            <= 1
                    })
            }
        }
    }

    // Adjustments 2 through 6 of Form 1116 line 16 for the four category
    // slots (general, passive, foreign branch, section 901(j)), on the net
    // foreign income after the current-year loss allocation: separate
    // limitation loss netting and recharacterization, overall foreign loss
    // recapture capped at 50% of foreign income, overall domestic loss
    // recapture capped at 50% of U.S. income (Reg. 1.904(g)-3).
    pub(crate) fn apply(
        &self,
        net: &mut [i64; 4],
        current: &crate::foreign_losses::CurrentLoss,
        amt: bool,
        r: &mut FederalResult,
    ) {
        let Self::Retained { accounts, .. } = self else {
            return;
        };
        crate::require_localized!(
            r,
            self.valid(),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign loss history",
            "ISSUE_CONFIRM_COMPLETE_RETAINED_REGULAR_AND_AMT_LOSS_ACCOUNT_RECORDS_PRIO_C7499B09",
            "Enter each category's retained regular and AMT loss accounts once, with at most one separate limitation loss balance per tax system.",
        );
        if !self.valid() {
            return;
        }
        let mut overall = current.overall;
        let mut separate = current.separate;
        let mut domestic = current.domestic;
        let mut extra = [0; 4];
        for a in accounts {
            let i = a.category as usize;
            let s = if amt { &a.amt } else { &a.regular };
            overall[i] += dollars(s.overall);
            separate[i] += dollars(s.separate);
            domestic[i] += dollars(s.domestic);
            extra[i] = dollars(s.additional_recapture);
        }
        // Step3: opposite-direction current/prior separate loss accounts net.
        let mut netted = [0; 4];
        for i in 0..4 {
            for j in i + 1..4 {
                let offset = phase::cap(separate[i], [separate[j]]);
                separate[i] = crate::kernel::carry::native_balance(separate[i], offset);
                separate[j] = crate::kernel::carry::native_balance(separate[j], offset);
                netted[i] += offset;
                netted[j] += offset;
            }
        }
        // Step5: statutory OFL recapture uses a single50% total-income cap,
        // apportioned by each account's maximum potential recapture.
        let potential: [i64; 4] = core::array::from_fn(|i| phase::cap(overall[i], [net[i]]));
        let cap = phase::cap(potential.iter().sum::<i64>(), [fraction(net.iter().sum(), 1, 2)]);
        let mut ofl: [i64; 4] = allocate_remaining(cap, &potential).try_into().unwrap();
        for i in 0..4 {
            let remaining =
                crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                    potential[i],
                    ofl[i],
                )
                .get();
            crate::require_localized!(
                r,
                extra[i] <= remaining,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign loss recapture election",
                "ISSUE_ELECTED_ADDITIONAL_FOREIGN_LOSS_RECAPTURE_EXCEEDS_THE_REMAINING_ACC_D45469A8",
                "Elected additional foreign-loss recapture exceeds the remaining account balance or same-category income after statutory recapture.",
            );
            ofl[i] += phase::cap(extra[i], [remaining]);
            overall[i] = crate::kernel::carry::native_balance(overall[i], ofl[i]);
            net[i] = crate::kernel::carry::native_balance(net[i], ofl[i]);
        }
        // Step6: netting leaves at most one direction. Recharacterize only
        // income remaining after OFL recapture, without retesting high tax:
        // the income of a category with a separate loss balance becomes
        // income of the categories that absorbed the loss, pro rata by their
        // income (equally when they have none).
        let sll: [i64; 4] = core::array::from_fn(|i| phase::cap(separate[i], [net[i]]));
        for i in 0..4 {
            separate[i] = crate::kernel::carry::native_balance(separate[i], sll[i]);
            net[i] = crate::kernel::carry::native_balance(net[i], sll[i]);
            if sll[i] > 0 {
                let others: [i64; 4] = core::array::from_fn(|j| if j == i { 0 } else { net[j] });
                let weights: [i64; 4] = if others.iter().sum::<i64>() > 0 {
                    others
                } else {
                    // No income to follow: the passive category for a general
                    // loss, the general category for any other.
                    core::array::from_fn(|j| i64::from(j == usize::from(i == 0)))
                };
                for (j, share) in allocate_remaining(sll[i], &weights).into_iter().enumerate() {
                    net[j] += share;
                }
            }
        }
        // Step7: OFL-recharacterized U.S. income is excluded from this cap.
        let cap =
            phase::cap(domestic.iter().sum::<i64>(), [fraction(current.domestic_income, 1, 2)]);
        let odl = allocate_remaining(cap, &domestic);
        let context = if amt { "amt" } else { "regular" };
        for i in 0..4 {
            domestic[i] = crate::kernel::carry::native_balance(domestic[i], odl[i]);
            net[i] += odl[i];
        }
        for i in 0..3 {
            let category = crate::foreign_losses::CATEGORY_NAMES[i];
            emit_lines(
                r,
                &mut (),
                [
                    (
                        "separate_loss_netted",
                        netted[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_NETTED"),
                    ),
                    (
                        "overall_foreign_loss_recaptured",
                        ofl[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_OVERALL_FOREIGN_LOSS_RECAPTURED"),
                    ),
                    (
                        "separate_loss_recaptured",
                        sll[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_RECAPTURED"),
                    ),
                    (
                        "overall_domestic_loss_recaptured",
                        odl[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_OVERALL_DOMESTIC_LOSS_RECAPTURED"),
                    ),
                    (
                        "overall_foreign_loss_remaining",
                        overall[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_OVERALL_FOREIGN_LOSS_REMAINING"),
                    ),
                    (
                        "separate_loss_remaining",
                        separate[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_REMAINING"),
                    ),
                    (
                        "overall_domestic_loss_remaining",
                        domestic[i],
                        msg!("EXPLAIN_FOREIGN_N_N_N_OVERALL_DOMESTIC_LOSS_REMAINING"),
                    ),
                ],
                |(kind, amount, detail), _, _| {
                    Some(calculation_row(
                        None,
                        &format!("foreign_{context}_{category}_{kind}"),
                        LineValue::MoneyDollars(amount),
                        &["credits.foreign_tax.loss_history"],
                        "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol11/pdf/CFR-2025-title26-vol11-sec1-904g-3.pdf",
                        detail,
                    ))
                },
            );
        }
    }
}
