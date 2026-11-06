use crate::federal_rules::valid_birth;
use crate::kernel::phase;
use alloc::vec::Vec;
use libertas_macros::*;

/// Temporary civil-service child survivor annuity
/// For one nondisabled child with no surviving-spouse annuity. Use the temporary-annuity cost and period from the common CSRS/FERS records. A child permanently disabled at the starting date uses the pension Simplified Method with a lifetime duration instead. Multiple temporary children and changed recovery periods need separate allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildAnnuity {
    /// Last entitled payment date if this annuity ended early in 2026
    /// Use the plan's last entitled payment month, not the date a late check arrived. Leave absent when payments continue through the normal period. Early termination does not shorten the original cost-recovery divisor or automatically create a deduction; enter any final lump sum separately.
    /// [NullText]
    /// Payments continue through the normal period
    /// [SetText]
    /// Payments ended early on
    #[libertas_date_only]
    pub ended: Option<u32>,
    /// Temporary annuity starting date
    #[libertas_date_only]
    pub started: u32,
    /// Payment period from the starting date until this child reaches age 22
    /// Use the original count from the plan's benefit dates, not what remains now.
    // Tax structure: this temporary child-survivor annuity ends at age 22 (22 × 12 months).
    #[libertas_number(min = 1, max = 264)]
    #[libertas_physical_unit("month")]
    pub term: u32,
    /// Cost allocated to this temporary annuity at its starting date
    /// Employee contributions available for recovery; for a retiree's survivor, exclude cost already recovered before this temporary annuity began.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Cost actually recovered before 2026 under this temporary annuity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub recovered: i64,
    /// Unused exclusion carried into 2026
    /// Retain from the prior worksheet. This is unrecovered cost, not an extra investment or an amount already excluded.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred: i64,
    /// Monthly 2026 payments, in order
    /// Include every entitled month of 2026, including zero payments; begin with January or the starting month for a new annuity. Stop at the documented early ending month when applicable. Their sum must equal the retirement gross. Do not combine several months into a single payment entry.
    // Tax structure: one payment row for each entitled month of this tax year.
    #[libertas_size(min = 1, max = 12)]
    /// ----
    /// Monthly payment
    pub payments: Vec<ChildAnnuityPayment>,
    /// Complete temporary-annuity records and eligibility confirmed
    /// CSRS/FERS temporary survivor annuity for this one nondisabled child, without a surviving-spouse annuity or other annuitants. The original recovery period ends at age 22; any earlier last entitled payment date is separately documented. All entitled months and actual payments are present, with no changed recovery divisor, foreign/treaty rule, estate-tax deduction, unentered final lump sum or other adjustment. Use the death exception for the retirement distribution. No taxes the whole payment, with no cost recovered.
    pub confirmed: bool,
}

/// Temporary child annuity monthly payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildAnnuityPayment {
    /// Gross payment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
impl ChildAnnuity {
    fn elapsed(&self) -> u32 {
        (2026_u32 * 12).saturating_sub(self.started / 10000 * 12 + self.started / 100 % 100 - 1)
    }
    fn first_month(&self) -> u32 {
        if self.started / 10000 == 2026 { self.started / 100 % 100 } else { 1 }
    }
    fn normal_last_month(&self) -> u32 {
        self.first_month() + (self.term - self.elapsed()).min(13 - self.first_month()) - 1
    }
    fn last_month(&self) -> u32 {
        self.ended.map_or_else(|| self.normal_last_month(), |ended| ended / 100 % 100)
    }
    #[cfg(test)]
    pub(crate) fn valid(&self, gross: i64) -> bool {
        self.valid_with_source_alignment(gross, true)
    }
    pub(crate) fn valid_with_source_alignment(&self, gross: i64, source_alignment: bool) -> bool {
        valid_birth(self.started)
            && (20040101..=20261231).contains(&self.started)
            && (1..=264).contains(&self.term)
            && (0..=100000000000).contains(&self.cost)
            && (0..=self.cost).contains(&self.recovered)
            && (0..=self.cost - self.recovered).contains(&self.deferred)
            && self.elapsed() < self.term
            && self.ended.is_none_or(|ended| {
                valid_birth(ended)
                    && (20260101..=20261231).contains(&ended)
                    && ended >= self.started
                    && (self.first_month()..=self.normal_last_month())
                        .contains(&(ended / 100 % 100))
            })
            && self.payments.len()
                == self.last_month().saturating_sub(self.first_month()) as usize + 1
            && self.payments.iter().all(|p| (0..=100000000000).contains(&p.amount))
            && (!source_alignment || self.payments.iter().map(|p| p.amount).sum::<i64>() == gross)
            && (self.started < 20260101 || (self.recovered == 0 && self.deferred == 0))
            && self.recovered + self.deferred
                == (self.monthly() * i64::from(self.elapsed())).min(self.cost)
    }
    pub(crate) fn matches_birth(&self, birth: u32) -> bool {
        valid_birth(birth)
            && birth <= self.started
            && ((birth / 10000 + 22) * 12 + birth / 100 % 100 - 1)
                .checked_sub(self.started / 10000 * 12 + self.started / 100 % 100 - 1)
                == Some(self.term)
    }
    pub(crate) fn monthly(&self) -> i64 {
        let months = i64::from(self.term.max(1));
        crate::kernel::ratio::half_offset_i64(self.cost, months)
    }
    pub(crate) fn result(&self) -> (i64, i64, i64) {
        let mut remaining = phase::floor(self.cost - self.recovered, 0);
        let mut deferred = self.deferred;
        let mut excluded = 0;
        for payment in &self.payments {
            let available = phase::cap(deferred + self.monthly(), [remaining]);
            let recovered = phase::floor(phase::cap(available, [payment.amount]), 0);
            excluded += recovered;
            remaining = crate::kernel::carry::native_balance(remaining, recovered);
            deferred = crate::kernel::carry::native_balance(available, recovered);
        }
        (excluded, deferred, remaining)
    }
}
