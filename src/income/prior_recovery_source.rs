use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// That year's return
/// Publication 525 Worksheet 2, from the federal return of the year you deducted it. Enter every refund or reimbursement of the same year's itemized deductions together, as one item. A state or local income tax refund of the same year is entered as its 1099-G; the App figures both together.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PriorYearRecovery {
    /// Year you deducted it
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2010, max = 2025)]
    #[libertas_default(2025)]
    pub year: i32,
    /// What you deducted for the items refunded
    /// That year's deduction for them after its limits, such as medical costs only above the 7.5% floor. The taxable part is never more than this.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted: i64,
    /// Total itemized deductions that year
    /// Schedule A line 17 (line 29 for 2010–2017).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub itemized: i64,
    /// Standard deduction you could have taken that year
    /// With any age or blindness additions; zero if you had to itemize (married filing separately with a spouse who itemized).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub standard: i64,
    /// Taxable income that year
    /// Form 1040 line 15, or a negative amount when your deductions were more than your income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub taxable_income: i64,
    /// That year's return had no special tax-benefit situation
    /// Yes when, that year, you had no unused credits, no alternative minimum tax and no tax figured at the 0% capital gain rate, you could not be claimed as a dependent, nothing was refunded for it in an earlier year, and the refund is not of real estate, personal property or sales tax. That year's return shows it. No leaves this item out of the estimate with a notice; that refiguring is not calculated.
    pub ordinary: bool,
}

/// Tax-benefit facts for a prior Schedule 1-A vehicle-interest deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleRecoveryYear {
    /// Year the refunded interest was deducted
    // Section 163(h)(4) first applies to 2025; this return recovers prior years only.
    #[libertas_number(min = 2025, max = 2025)]
    pub year: i32,
    /// Filing status on that return
    pub status: crate::filer_facts::FederalStatus,
    /// Modified adjusted gross income used on Schedule 1-A
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub magi: i64,
    /// Total eligible personal vehicle interest before the cap and phaseout
    /// Include all qualifying vehicles on that prior return, after removing business interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub eligible_interest: i64,
    /// Taxable income before applying the zero floor
    /// Use that year's income less deductions; a negative amount preserves the part of a deduction that did not help.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub taxable_income: i64,
    /// The prior deduction's tax benefit can be established from these facts
    /// Confirm the prior interest was actually deducted, the refunded amount was included in eligible interest, and there are no unused credits, alternative minimum tax, zero-rate capital gains, earlier refunds or other dependent tax effects requiring a full prior-return recomputation. No leaves this recovery uncalculated, with a notice linked here.
    pub ordinary: bool,
}
/// How refunded vehicle interest was treated on the prior return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum VehicleRefundOrigin {
    /// No deduction was taken for the refunded interest
    NotDeducted,
    /// Deducted on Schedule 1-A
    Deducted { prior: VehicleRecoveryYear },
}
/// Prior-return answer for one vehicle-interest document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleInterestRecovery {
    /// Accepted vehicle-interest document identity
    pub id: i64,
    /// Prior deduction
    pub origin: VehicleRefundOrigin,
}
impl PriorYearRecovery {
    pub(crate) fn valid(&self) -> bool {
        let money = |v: i64| (0..=100_000_000_000).contains(&v);
        (2010..=2025).contains(&self.year)
            && money(self.deducted)
            && money(self.itemized)
            && money(self.standard)
            && (-100_000_000_000..=100_000_000_000).contains(&self.taxable_income)
            && self.deducted <= self.itemized
    }
}
