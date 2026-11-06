pub use crate::hsa_source::{HsaFamilyAllocation, HsaLastMonthHistory};
// Form 8889 line 18; Notice 2008-52. Historical 2025 limits, not 2026 limits.
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, ratio, whole};
use crate::{HsaCoverage, HsaLossReason};

impl HsaLastMonthHistory {
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

impl HsaFamilyAllocation {
    fn share_exact(&self, amount: Exact, divisor: i64) -> Exact {
        amount
            * ratio(
                i64::from(self.owner_parts),
                divisor * (i64::from(self.owner_parts) + i64::from(self.spouse_parts)).max(1),
            )
            .unwrap()
    }
}
