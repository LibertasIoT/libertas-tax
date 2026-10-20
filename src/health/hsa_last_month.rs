// Form 8889 line 18; Notice 2008-52. Historical 2025 limits, not 2026 limits.
use crate::federal_rules::valid_birth;
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, ratio, whole};
use crate::{HsaCoverage, HsaEligibilityLoss, HsaFundingHistory, HsaLossReason};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior HSA last-month-rule history
/// Use the owner's 2025 source records. These historical contributions are not 2026 contributions or withdrawals. The App recomputes the ordinary monthly limit to determine the amount attributable to the last-month rule.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaLastMonthHistory {
    /// January through December 2025 eligibility
    /// Actual coverage on each month's first day, before the last-month rule. None includes Medicare, disqualifying coverage or dependent status. December must be eligible.
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 12, max = 12)]
    pub months: Vec<HsaCoverage>,
    /// Total 2025 HSA contributions retained
    /// Personal plus employer contributions deducted or excluded under the last-month rule, after timely corrections. Exclude rollovers and IRA funding. Use original contribution records, not a calculated recapture amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub contributions: i64,
    /// 2025 Archer MSA contributions
    /// Total personal and employer deposits for this owner. These reduce the ordinary HSA contribution limit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub archer: i64,
    /// Married family-limit allocation, if applicable
    /// Required when either spouse had family coverage in 2025. Actual monthly coverage controls the shared allowance; December self-only coverage uses the owner’s independent last-month allowance.
    /// [NullText]
    /// The married family-coverage allocation does not apply
    /// [SetText]
    /// The married family-limit allocation is
    pub family: Option<HsaFamilyAllocation>,
    /// First loss of eligibility during or after December 2025, if any
    /// Absent means eligibility continued through December 31, 2026. A later restoration does not erase the first loss. Death or permanent and total disability must cause the loss to qualify for the exception.
    /// [NullText]
    /// HSA eligibility continued through December 31, 2026
    /// [SetText]
    /// The first loss of HSA eligibility was
    pub loss: Option<HsaEligibilityLoss>,
}
impl HsaLastMonthHistory {
    // The page rule, checked again on save with the same key: the owner was
    // eligible in December, the month the last-month rule depends on.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.months.get(11) == Some(&HsaCoverage::None)).then(|| {
            crate::validation::ValidationIssue::new("HSA_LAST_MONTH_DECEMBER_NEEDED")
                .at_list_item("months", 11)
        })
    }
    pub(crate) fn valid(&self) -> bool {
        self.months.len() == 12
            && self.issue().is_none()
            // Arithmetic exclusion: recapture enters the federal before_rental,
            // unearned and Olympic AGI-to-Cents receiver chain before final AGI.
            && (0..=100000000000).contains(&self.contributions)
            && self.archer >= 0
            && self.family.as_ref().is_none_or(|v| {
                v.valid()
                    && (self.months.contains(&HsaCoverage::Family)
                        || v.spouse_months.contains(&HsaCoverage::Family))
            })
            && self.loss.as_ref().is_none_or(|v| valid_birth(v.date) && v.date >= 20251201)
    }
    fn annual(coverage: HsaCoverage, catchup: bool) -> i64 {
        match coverage {
            HsaCoverage::None => 0,
            HsaCoverage::SelfOnly => {
                crate::year_constants::HSA_SELF_ONLY_LIMIT_2025_DOLLARS
                    + if catchup { crate::year_constants::HSA_CATCH_UP_DOLLARS } else { 0 }
            }
            HsaCoverage::Family => {
                crate::year_constants::HSA_FAMILY_LIMIT_2025_DOLLARS
                    + if catchup { crate::year_constants::HSA_CATCH_UP_DOLLARS } else { 0 }
            }
        }
    }
    // Pub969(2025), p6 Examples1/2 retain monthly fractions through
    // subtraction: 8550/12=712.50 and 60100/12=5008.333... . These
    // transient cents are not another contribution or history record.
    pub(crate) fn limits_exact(&self, birth: u32) -> (Exact, Exact) {
        let catchup = crate::hsa::catchup_age(birth, 2025);
        if let Some(family) = &self.family {
            let mut eligible = 0;
            let mut family_months = 0;
            let mut self_months = 0;
            for (i, coverage) in self.months.iter().enumerate() {
                if *coverage == HsaCoverage::None {
                    continue;
                }
                eligible += 1;
                if *coverage == HsaCoverage::Family
                    || family.spouse_months.get(i) == Some(&HsaCoverage::Family)
                {
                    family_months += 1;
                } else {
                    self_months += 1;
                }
            }
            let archer = whole(self.archer) + whole(family.spouse_archer);
            let ordinary = whole(crate::year_constants::HSA_SELF_ONLY_LIMIT_2025_DOLLARS * 100)
                * ratio(self_months, 12).unwrap()
                + family.share_exact(
                    phase::floor(
                        whole(
                            crate::year_constants::HSA_FAMILY_LIMIT_2025_DOLLARS
                                * 100
                                * family_months,
                        ) - &archer * whole(12),
                        whole(0),
                    ),
                    12,
                )
                + if catchup {
                    whole(crate::year_constants::HSA_CATCH_UP_DOLLARS * 100)
                        * ratio(eligible, 12).unwrap()
                } else {
                    whole(0)
                };
            let december_family = self.months.last() == Some(&HsaCoverage::Family)
                || family.spouse_months.last() == Some(&HsaCoverage::Family);
            let december = if december_family {
                family.share_exact(
                    phase::floor(
                        whole(crate::year_constants::HSA_FAMILY_LIMIT_2025_DOLLARS * 100) - &archer,
                        whole(0),
                    ),
                    1,
                )
            } else {
                phase::floor(
                    whole(
                        crate::year_constants::HSA_SELF_ONLY_LIMIT_2025_DOLLARS * 100 - self.archer,
                    ),
                    whole(0),
                )
            } + if catchup {
                whole(crate::year_constants::HSA_CATCH_UP_DOLLARS * 100)
            } else {
                whole(0)
            };
            return (ordinary.clone(), phase::floor(ordinary, december));
        }
        let ordinary = whole(self.months.iter().map(|v| Self::annual(*v, catchup) * 100).sum())
            * ratio(1, 12).unwrap();
        let december = whole(self.months.last().map_or(0, |v| Self::annual(*v, catchup) * 100));
        (
            phase::floor(&ordinary - whole(self.archer), whole(0)),
            phase::floor(phase::floor(ordinary, december) - whole(self.archer), whole(0)),
        )
    }
    #[cfg(test)]
    pub(crate) fn limits(&self, birth: u32) -> (i64, i64) {
        let (ordinary, elected) = self.limits_exact(birth);
        (
            crate::kernel::ratio::exact::dollars(&ordinary).unwrap(),
            crate::kernel::ratio::exact::dollars(&elected).unwrap(),
        )
    }
    pub(crate) fn consistent_loss(&self, transfer: &HsaFundingHistory) -> bool {
        // Earlier loss followed by restored eligibility can precede the other
        // testing period. Otherwise both histories describe the same first loss
        // within their overlap. None confirms only through that period's end.
        let start = transfer.funded.max(20251201);
        if self.loss.as_ref().is_some_and(|v| v.date < start)
            || transfer.loss.as_ref().is_some_and(|v| v.date < start)
        {
            return true;
        }
        let end_month = transfer.funded / 100 + 100;
        self.loss.as_ref().filter(|v| v.date / 100 <= end_month)
            == transfer.loss.as_ref().filter(|v| v.date / 100 <= end_month)
    }
    pub(crate) fn recapture_cents(&self, birth: u32, funding: i64) -> i64 {
        if self
            .loss
            .as_ref()
            .is_some_and(|v| v.date / 10000 == 2026 && v.reason == HsaLossReason::Other)
        {
            let amount = phase::cap(
                phase::floor(
                    whole(self.contributions) + whole(funding) - self.limits_exact(birth).0,
                    whole(0),
                ),
                [whole(self.contributions)],
            );
            crate::kernel::ratio::exact::rounded(&amount).expect("bounded HSA recapture")
        } else {
            0
        }
    }
    #[cfg(test)]
    pub(crate) fn recapture(&self, birth: u32, funding: i64) -> i64 {
        crate::kernel::ratio::dollars(self.recapture_cents(birth, funding))
    }
}

/// Agreed share of the historical family HSA allowance
/// Describe the original agreement, not a computed contribution limit or recapture. Equal division is one part each; any agreed ratio, including zero for one spouse, is allowed. Each owner's age-55 catch-up remains separate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaFamilyAllocation {
    /// Spouse's actual January–December eligibility
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 12, max = 12)]
    pub spouse_months: Vec<HsaCoverage>,
    /// Spouse's 2025 Archer contributions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub spouse_archer: i64,
    /// This owner's agreed parts
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub owner_parts: u32,
    /// Spouse's agreed parts
    /// At least one spouse must have a nonzero share. For two histories on this return, enter reciprocal shares and matching coverage/Archer facts.
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub spouse_parts: u32,
}
impl HsaFamilyAllocation {
    fn valid(&self) -> bool {
        self.spouse_months.len() == 12
            && self.spouse_archer >= 0
            && u64::from(self.owner_parts) + u64::from(self.spouse_parts) > 0
    }
    fn share_exact(&self, amount: Exact, divisor: i64) -> Exact {
        amount
            * ratio(
                i64::from(self.owner_parts),
                divisor * (i64::from(self.owner_parts) + i64::from(self.spouse_parts)).max(1),
            )
            .unwrap()
    }
    pub(crate) fn reciprocal(&self, other: &Self) -> bool {
        u64::from(self.owner_parts) * u64::from(other.owner_parts)
            == u64::from(self.spouse_parts) * u64::from(other.spouse_parts)
    }
}
