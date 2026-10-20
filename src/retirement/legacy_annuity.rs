use crate::AnnuityBalance;
use crate::kernel::phase;
use libertas_macros::*;

/// Withdrawal from an annuity with pre-August 14, 1982 investment
/// Use the insurer's allocation immediately before one actual partial withdrawal. The App applies the old investment, old earnings, new earnings, new investment ordering.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LegacyAnnuity {
    /// Original contract issue date
    #[libertas_date_only]
    pub issued: u32,
    /// Portion attributable to investment before August 14, 1982
    pub before: AnnuityBalance,
    /// Portion attributable to investment after August 13, 1982
    /// Enter zero value and investment when no later investment exists.
    pub after: AnnuityBalance,
    /// Complete insurer allocation and ordinary partial cash withdrawal confirmed
    /// A commercial annuity issued before August 14, 1982, before annuity payments begin. Both portions' current value and remaining investment are reconciled after earlier withdrawals. No loan, pledge, full surrender, exchange, changed ownership or unresolved related-contract allocation. A portion with value below its remaining investment requires further allocation support. The insurer's allocation statement shows it. No taxes the whole withdrawal with no investment recovered, with a notice; that situation is not supported.
    pub confirmed: bool,
}
impl LegacyAnnuity {
    pub(crate) fn confirmed(&self) -> bool {
        self.confirmed
            && self.issued < 19820814
            && self.before.value >= self.before.investment
            && self.after.value >= self.after.investment
    }
    pub(crate) fn valid(&self, gross: i64) -> bool {
        crate::federal_rules::valid_birth(self.issued)
            && [self.before.value, self.before.investment, self.after.value, self.after.investment]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
            && gross <= self.before.value + self.after.value
    }
    // Pub. 575, nonqualified distributions before annuity starting date.
    pub(crate) fn parts(&self, gross: i64) -> [i64; 4] {
        let mut remaining = gross;
        let mut result = [0; 4];
        for (i, available) in [
            self.before.investment,
            phase::floor(self.before.value - self.before.investment, 0),
            phase::floor(self.after.value - self.after.investment, 0),
            self.after.investment,
        ]
        .into_iter()
        .enumerate()
        {
            result[i] = crate::kernel::carry::native_consume_step(available, &mut remaining);
        }
        result
    }
    pub(crate) fn taxable(&self, gross: i64) -> i64 {
        let p = self.parts(gross);
        p[1] + p[2]
    }
}
