// Coverage: unrecovered annuity investment at the final annuitant's death (Schedule A line 17i,
//   AnnuityLoss, federal_rules.rs final_annuity_losses) — Partial (T3-deduct, 2026-09-28). Asked on
//   the itemized hub's investment interest and other page ("Annuity cost not recovered at death"),
//   each row naming a non-IRA retirement entry of a filer who died (annuity_contracts); the walk
//   erases a row whose entry leaves that catalog or when the route stops itemizing. Client rules
//   with server twins: ANNUITY_RECOVERED_ABOVE_COST, ANNUITY_REFUNDS_ABOVE_COST,
//   ANNUITY_CONTRACT_TWICE. Tests: tests/deductions/deduction_pages.rs
//   unrecovered_annuity_cost_reaches_line_17i_through_the_other_itemized_page (35,000),
//   annuity_cost_rules_and_catalog; tests/retirement/retirement.rs final_annuity_loss_*.
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

fn term_end(started: u32, months: u32) -> Option<u32> {
    if !valid_date(started) || !(1..=18000).contains(&months) {
        return None;
    }
    let month = started / 10000 * 12 + started / 100 % 100 - 1 + months;
    let base = month / 12 * 10000 + (month % 12 + 1) * 100;
    (1..=started % 100).rev().map(|day| base + day).find(|date| valid_date(*date))
}

/// Expected pension payment duration
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PensionDuration {
    /// One lifetime
    /// For a CSRS/FERS child permanently disabled at the annuity starting date, with no surviving-spouse annuity, use that child's original age. Other temporary child beneficiaries share this lifetime exclusion; do not use their ages to create a joint-lifetime factor.
    Single {
        /// Annuitant's age at the annuity starting date
        #[libertas_number(min = 0)]
        age: u32,
    },
    /// Joint and survivor lifetimes
    Joint {
        /// Primary annuitant's age at the starting date
        #[libertas_number(min = 0)]
        age: u32,
        /// Survivor's age at the starting date
        /// With multiple survivors use the youngest survivor. With no primary annuitant use the oldest and youngest annuitants' ages.
        #[libertas_number(min = 0)]
        survivor_age: u32,
    },
    /// Fixed period, independent of anyone's life expectancy
    Fixed {
        /// Total monthly payments under the contract
        #[libertas_number(min = 1)]
        payments: u32,
    },
}
impl PensionDuration {
    pub(crate) fn payments(self, started: u32) -> i64 {
        // Pub. 575 Worksheet A: Table 2 applies only to starts after 1997.
        // Earlier joint-life contracts use the primary annuitant's Table 1 age.
        let duration = match self {
            Self::Joint { age, .. } if started < 19980101 => Self::Single { age },
            other => other,
        };
        match duration {
            Self::Single { age } if started < 19961119 => match age {
                0..=55 => 300,
                56..=60 => 260,
                61..=65 => 240,
                66..=70 => 170,
                _ => 120,
            },
            Self::Single { age } => match age {
                0..=55 => 360,
                56..=60 => 310,
                61..=65 => 260,
                66..=70 => 210,
                _ => 160,
            },
            Self::Joint { age, survivor_age } => match age.saturating_add(survivor_age) {
                0..=110 => 410,
                111..=120 => 360,
                121..=130 => 310,
                131..=140 => 260,
                _ => 210,
            },
            Self::Fixed { payments } => i64::from(payments.max(1)),
        }
    }
    fn valid(self) -> bool {
        match self {
            Self::Single { age } => age <= 120,
            Self::Joint { age, survivor_age } => age <= 120 && survivor_age <= 120,
            Self::Fixed { payments } => (1..=1500).contains(&payments),
        }
    }
}

/// Simplified Method worksheet history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PensionHistory {
    /// Annuity starts in the worksheet year
    New { duration: PensionDuration },
    /// Continue the previously used Simplified Method
    Prior {
        /// Monthly tax-free amount from the prior worksheet line 4
        /// Keep the original amount even if pension payments have changed. Do not calculate a new amount from your remaining basis.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        monthly: i64,
        /// Basis recovered before the worksheet year, prior worksheet line 10
        /// Include amounts that could have been recovered even if omitted on a prior return. For a pre-1987 starting date, enter zero; the lifetime cost cap does not apply.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        recovered: i64,
    },
}

/// Simultaneous annuitant payment share
/// Use the original payment amounts that established the allocation, not later cost-of-living increases. Use the initial share and list documented entitlement changes separately. This recipient must receive payments throughout the worksheet months.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PensionShare {
    /// Monthly payments to beneficiaries preceding this recipient
    /// Use one consistent beneficiary order in the common allocation record on every recipient's return. Enter zero for the first recipient; otherwise enter the sum of the preceding original monthly payments. This order only assigns rounding cents and must remain unchanged with the retained worksheet.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub preceding: i64,
    /// This recipient's monthly payment at allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub payment: i64,
    /// Total monthly payments to all simultaneous annuitants at allocation
    /// Includes this recipient and recipients filing other returns. Exclude ordinary railroad spouse annuities, which cannot recover employee contributions.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub total: i64,
}
impl PensionShare {
    fn valid(self) -> bool {
        (1..=100000000000).contains(&self.payment)
            && (self.payment..=100000000000).contains(&self.total)
            && (0..=self.total - self.payment).contains(&self.preceding)
    }
    fn portion(self, amount: i64) -> i64 {
        // Round cumulative boundaries, not recipients independently. Adjacent shares
        // telescope to the exact common amount, including the final shared cent.
        let denominator = self.total.max(1);
        crate::kernel::ratio::fraction(amount, self.preceding + self.payment, denominator)
            - crate::kernel::ratio::fraction(amount, self.preceding, denominator)
    }
}

/// Pension allocation change during the payment year
/// Use only a documented change in beneficiaries' entitlement, not a cost-of-living increase. The common monthly exclusion and original contract cost remain unchanged.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PensionChange {
    /// Payment month when the new allocation begins
    /// Count from the first payment month entered in this worksheet: 2 means the second payment month, not necessarily February.
    // Tax structure: payment months and scheduled annual installments cannot exceed twelve.
    #[libertas_number(min = 2, max = 12)]
    pub month: u32,
    /// New simultaneous annuitant share
    /// Leave absent when this recipient becomes the only remaining annuitant. Keep a common beneficiary order for recipients who remain.
    /// [NullText]
    /// This recipient becomes the only remaining annuitant
    /// [SetText]
    /// The new simultaneous annuitant share is
    pub share: Option<PensionShare>,
}

/// Pension Simplified Method facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PensionBasis {
    /// Earlier common payment months in this worksheet year
    /// Payments to other annuitants before this recipient's first payment. Use the same common monthly exclusion and include every earlier recovery month once. Do not include earlier years here or add these months to this recipient's payment months. Changed common exclusions or deferred recoveries need another reconciliation.
    // Tax structure: payment months and scheduled annual installments cannot exceed twelve.
    #[libertas_number(min = 0, max = 11)]
    #[libertas_default(0)]
    pub earlier_months: u32,
    /// Changes in allocation during these payment months
    /// Leave empty for unchanged beneficiaries. List each entitlement change once; dates determine the order. This recipient must receive payments throughout the entered months. Earlier common payment months are entered separately; excess-exclusion carryovers or a change in the common monthly exclusion require further reconciliation.
    /// ----
    /// Allocation change
    // Tax structure: at most one allocation change in each month after the first.
    #[libertas_size(max = 11)]
    pub changes: alloc::vec::Vec<PensionChange>,
    /// Share the exclusion with simultaneous annuitants
    /// Leave absent for one recipient. When present, cost and prior recovered amounts cover the whole contract and every recipient; the prior monthly amount is the common amount before allocation. Do not enter an already allocated monthly amount and allocate it again.
    /// [NullText]
    /// This contract has one recipient
    /// [SetText]
    /// The exclusion is shared with simultaneous annuitants as follows
    pub share: Option<PensionShare>,
    /// Annuity starting date from the contract
    /// First day of the first period for which an annuity payment is made, not necessarily the check date. Pre-July 2, 1986 annuities need another method.
    #[libertas_date_only]
    pub started: u32,
    /// Original after-tax cost in the contract, worksheet line 2
    /// Use documented contributions and basis adjustments at the starting date, not the current account balance. A prior worksheet may include an eligible historical death-benefit exclusion. Leave empty to use box 9b, only for an annuity that started in 2026.
    /// [NullText]
    /// Use box 9b for this annuity that started in 2026
    /// [SetText]
    /// The original after-tax cost in the contract was
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub cost: Option<i64>,
    /// Months for which payments were made in the worksheet year
    /// Use 2026 for current income; historical repayment worksheets use their selected original year.
    // Tax structure: payment months and scheduled annual installments cannot exceed twelve.
    #[libertas_number(min = 1, max = 12)]
    pub months: u32,
    /// New calculation or retained worksheet
    pub history: PensionHistory,
    /// This annuity qualifies for the Simplified Method with complete, unchanged records
    /// Qualified employee pension/annuity, 403(b), or eligible railroad contributory annuity; at the starting date the annuitant was under 75 or had fewer than five years of guaranteed payments. For a pre-November 19, 1996 start, this method was elected and used previously. All payments to this recipient for this contract are in this one entry; simultaneous recipients and entitlement changes use the payment-share facts above; no changed basis, rollover, conversion or other unentered special adjustment. Enter a final-annuitant unrecovered-cost deduction separately and link this income entry. A public-safety insurance election is entered separately in the coverage checks and applied after basis recovery. Nonqualified annuities require the General Rule. Railroad benefits use the Railroad taxable-amount choice so recovery cannot offset noncontributory benefits. No recovers no cost: the whole payment is taxed, with a notice.
    pub eligible: bool,
}
impl PensionBasis {
    // The worksheet cost; the server resolves an empty one from box 9b on save.
    pub(crate) fn cost(&self) -> i64 {
        self.cost.unwrap_or(0)
    }
    pub(crate) fn valid(&self) -> bool {
        self.valid_for_year(2026)
    }
    pub(crate) fn valid_for_year(&self, year: u32) -> bool {
        if !(1986..=2026).contains(&year) {
            return false;
        }
        let first = year * 10000 + 101;
        let last = year * 10000 + 1231;
        valid_date(self.started)
            && (19860702..=last).contains(&self.started)
            && self.cost.is_some_and(|c| (0..=100000000000).contains(&c))
            && (1..=12).contains(&self.months)
            && self.earlier_months <= 12 - self.months
            && match self.history {
                PensionHistory::New { duration } => {
                    self.started >= first
                        && self.months + self.earlier_months <= 13 - (self.started / 100 % 100)
                        && duration.valid()
                        && (self.started >= 19961119
                            || !matches!(duration, PensionDuration::Fixed { .. }))
                }
                PensionHistory::Prior { monthly, recovered } => {
                    self.started < first
                        && (0..=100000000000).contains(&monthly)
                        && (0..=self.cost()).contains(&recovered)
                        && (self.started >= 19870101 || recovered == 0)
                }
            }
            && self.share.is_none_or(|share| share.valid() && self.common_monthly() <= share.total)
            && self.changes.iter().enumerate().all(|(i, change)| {
                (2..=self.months).contains(&change.month)
                    && !self.changes[..i].iter().any(|earlier| earlier.month == change.month)
                    && change
                        .share
                        .is_none_or(|share| share.valid() && self.common_monthly() <= share.total)
            })
    }
    pub(crate) fn common_monthly(&self) -> i64 {
        match self.history {
            PensionHistory::New { duration } => {
                let n = duration.payments(self.started);
                crate::kernel::ratio::half_offset_i64(self.cost(), n)
            }
            PensionHistory::Prior { monthly, .. } => monthly,
        }
    }
    pub(crate) fn monthly(&self) -> i64 {
        let common = self.common_monthly();
        self.share_for_month(self.months).map_or(common, |share| share.portion(common))
    }
    pub(crate) fn earlier_recovery(&self) -> i64 {
        let amount = self.common_monthly() * i64::from(self.earlier_months);
        if self.started < 19870101 {
            amount
        } else {
            let prior = match self.history {
                PensionHistory::New { .. } => 0,
                PensionHistory::Prior { recovered, .. } => recovered,
            };
            phase::cap(
                amount,
                [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    self.cost(),
                    prior,
                )
                .get()],
            )
        }
    }
    pub(crate) fn remaining(&self) -> i64 {
        let recovered = match self.history {
            PensionHistory::New { .. } => 0,
            PensionHistory::Prior { recovered, .. } => recovered,
        };
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            crate::kernel::carry::native_balance(self.cost(), recovered),
            self.earlier_recovery(),
        )
        .get()
    }
    fn common_recovery(&self) -> i64 {
        let yearly = self.common_monthly() * i64::from(self.months);
        let limited =
            if self.started < 19870101 { yearly } else { phase::cap(yearly, [self.remaining()]) };
        phase::floor(limited, 0)
    }
    fn share_for_month(&self, month: u32) -> Option<PensionShare> {
        self.changes
            .iter()
            .filter(|change| change.month <= month)
            .max_by_key(|change| change.month)
            .map_or(self.share, |change| change.share)
    }
    pub(crate) fn recovery(&self) -> i64 {
        let monthly = self.common_monthly();
        let mut available = if self.started < 19870101 {
            monthly * i64::from(self.months)
        } else {
            self.remaining()
        };
        let mut recovered = 0;
        for month in 1..=self.months {
            let common = phase::floor(phase::cap(monthly, [available]), 0);
            recovered += self.share_for_month(month).map_or(common, |share| {
                PensionShare {
                    preceding: crate::kernel::ratio::fraction(
                        monthly,
                        share.preceding,
                        share.total.max(1),
                    ),
                    payment: share.portion(monthly),
                    total: monthly,
                }
                .portion(common)
            });
            available = crate::kernel::carry::native_balance(available, common);
        }
        recovered
    }
    pub(crate) fn remaining_after_payments(&self) -> i64 {
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            self.remaining(),
            self.common_recovery(),
        )
        .get()
    }
    pub(crate) fn excluded(&self, gross: i64) -> i64 {
        phase::floor(phase::cap(self.recovery(), [gross]), 0)
    }
}

/// General Rule payment duration
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AnnuityDuration {
    /// One life, ending no later than a fixed term
    /// Payments stop at death or the end of the term, whichever comes first. This is not a guaranteed period with payments continuing after death. Table VIII must apply to the entire investment.
    TemporaryLife {
        /// Nearest-birthday age at original annuitization
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
        /// Original maximum term in months
        /// From the contract, not remaining months. The App selects the IRS table using the nearest whole number of years. Terms over forty years need further treatment.
        // Tax structure: Pub. 939 Table VIII covers temporary terms through 40 years.
        #[libertas_number(min = 13, max = 480)]
        months: u32,
    },
    /// Joint life and last survivor, two annuitants
    JointLife {
        /// Whole months from original annuitization to the first primary payment
        /// Use original timing for the common payment frequency, even after the first death. Do not restart the timing calculation for the survivor.
        // Tax structure: Pub. 939 payment-frequency timing adjustments end at twelve months.
        #[libertas_number(min = 0, max = 12)]
        delay: u32,
        /// Primary annuitant's nearest-birthday age at the original starting date
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
        /// Survivor's nearest-birthday age at the original starting date
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        survivor_age: u32,
        /// Original periodic payment promised after the primary annuitant's death
        /// Enter the contract's survivor amount even while the primary annuitant is alive. It may differ from the primary amount entered above.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        survivor_payment: i64,
        /// All payments entered for 2026 were paid to the designated survivor
        /// No means all were paid to the primary annuitant. For a year with payments to both recipients, enter each recipient separately and include the first annuitant's earlier original-payment amounts on the survivor's worksheet. Enter any final-annuitant deduction separately and link this income entry. Original ages, payment terms and total contract cost remain unchanged after the first death; prior recovery includes both recipients' earlier exclusions.
        #[libertas_default(false)]
        survivor: bool,
    },
    /// Fixed period independent of anyone's lifespan
    Fixed {
        /// Total installments promised over a term longer than one year
        #[libertas_number(min = 2)]
        payments: u32,
    },
    /// One lifetime, optionally with guaranteed installments
    SingleLife {
        /// Original total payments guaranteed even after death
        /// Enter zero when payments stop at death without a guarantee. Otherwise enter the total dollars the contract guarantees to the annuitant and beneficiary together, not the remaining guarantee or a tax adjustment. For a guaranteed installment count, use the contract's total at the original payment rate. The guarantee may end with a fractional installment; all preceding installments must use the full regular amount. A fractional original first installment needs another worksheet. Payments continue for life if the annuitant survives it. The App derives duration and the Table VII refund adjustment. This covers continued installments, not a cash refund at death or beneficiary receipts. Guarantees over forty years or a rounded term below one year without the young-annuitant zero-value exception need further calculation.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        guaranteed: i64,
        /// Whole months from original annuitization to the first payment
        /// Used with payment frequency to select the IRS adjustment to the lifetime factor.
        // Tax structure: Pub. 939 payment-frequency timing adjustments end at twelve months.
        #[libertas_number(min = 0, max = 12)]
        delay: u32,
        /// Age at the birthday nearest the original annuity starting date
        /// Use the original annuitant's nearest-birthday age, not their current age or age at the last birthday. Table V and, when applicable, Table VII must apply to the entire investment.
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
    },
}
impl AnnuityDuration {
    pub(crate) fn life_multiple(age: u32) -> Option<i64> {
        // IRS Pub. 939 (12/2025), Table V. Ages 5..=115; multiples in tenths.
        // https://www.irs.gov/publications/p939#en_US_2025_publink1000137034
        [
            766, 756, 747, 737, 727, 717, 707, 697, 688, 678, 668, 658, 648, 639, 629, 619, 609,
            599, 590, 580, 570, 560, 551, 541, 531, 522, 512, 502, 493, 483, 473, 464, 454, 444,
            435, 425, 415, 406, 396, 387, 377, 368, 359, 349, 340, 331, 322, 313, 304, 295, 286,
            277, 268, 259, 250, 242, 233, 225, 216, 208, 200, 192, 184, 176, 168, 160, 153, 146,
            139, 132, 125, 119, 112, 106, 100, 95, 89, 84, 79, 74, 69, 65, 61, 57, 53, 50, 47, 44,
            41, 39, 37, 34, 32, 30, 28, 27, 25, 23, 21, 19, 18, 16, 14, 13, 11, 10, 9, 8, 7, 6, 5,
        ]
        .get(age.checked_sub(5)? as usize)
        .copied()
    }
    fn valid(self, frequency: AnnuityFrequency, periodic: i64) -> bool {
        match self {
            Self::TemporaryLife { age, months } => {
                (13..=480).contains(&months)
                    && crate::annuity_tables::temporary_multiple(age, (months + 6) / 12).is_some()
            }
            Self::Fixed { payments } => payments > frequency.count() && payments <= 1500,
            Self::SingleLife { age, delay, guaranteed } => {
                Self::life_multiple(age)
                    .zip(frequency.adjustment(delay))
                    .is_some_and(|(multiple, adjustment)| multiple + adjustment > 0)
                    && (0..=100000000000).contains(&guaranteed)
                    && guaranteed <= 40 * i64::from(frequency.count()) * periodic
                    && self.refund_percent(frequency, periodic).is_some()
            }
            Self::JointLife { age, survivor_age, survivor_payment, delay, .. } => {
                crate::annuity_tables::joint_multiple(age, survivor_age)
                    .zip(frequency.adjustment(delay))
                    .is_some_and(|(multiple, adjustment)| multiple + adjustment > 0)
                    && (1..=100000000000).contains(&survivor_payment)
            }
        }
    }
    fn refund_percent(self, frequency: AnnuityFrequency, periodic: i64) -> Option<i64> {
        match self {
            Self::SingleLife { age, guaranteed, .. } if guaranteed > 0 => {
                // Pub. 939's unisex-table exception: age <=57 and <2.5 years.
                let annual = periodic * i64::from(frequency.count());
                if annual <= 0 {
                    return None;
                }
                if age <= 57 && guaranteed * 2 < annual * 5 {
                    Some(0)
                } else {
                    let years = crate::kernel::ratio::half_offset_i64(guaranteed, annual);
                    crate::annuity_tables::refund_percent(age, u32::try_from(years).ok()?)
                }
            }
            _ => Some(0),
        }
    }
    fn expected_tenths(self, periodic: i64, frequency: AnnuityFrequency) -> i64 {
        let annual = periodic * i64::from(frequency.count());
        match self {
            Self::TemporaryLife { age, months } => {
                annual
                    * crate::annuity_tables::temporary_multiple(age, (months + 6) / 12).unwrap_or(0)
            }
            Self::Fixed { payments } => periodic * i64::from(payments) * 10,
            Self::SingleLife { age, delay, .. } => {
                annual
                    * (Self::life_multiple(age).unwrap_or(0)
                        + frequency.adjustment(delay).unwrap_or(0))
            }
            Self::JointLife { age, survivor_age, survivor_payment, delay, .. } => {
                let adjustment = frequency.adjustment(delay).unwrap_or(0);
                let primary = Self::life_multiple(age).unwrap_or(0) + adjustment;
                let both = crate::annuity_tables::joint_multiple(age, survivor_age).unwrap_or(0)
                    + adjustment;
                annual * primary
                    + survivor_payment * i64::from(frequency.count()) * (both - primary)
            }
        }
    }
}

/// General Rule for fixed-payment pensions and commercial annuities
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct GeneralAnnuity {
    /// Original contractual payment frequency
    /// Keep unchanged for all entered installments and the designated survivor. Irregular frequency changes need separate treatment.
    pub frequency: AnnuityFrequency,
    /// Original-payment amounts received by the first annuitant earlier in 2026
    /// On the designated survivor's entry, total the original-payment portions paid to the first annuitant before this survivor's payments, including delayed or fractional original installments but excluding cost-of-living increases. Use payment records, not a calculated tax-free amount. Those receipts belong on the first annuitant's income entry or return, not in this entry's gross income. Enter zero otherwise. Do not also count this year's recovery in the recovery-through-2025 field or other deduction payouts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub earlier_payments: i64,
    /// Annuity starting date
    #[libertas_date_only]
    pub started: u32,
    /// Original annuitant's age for a later qualified-plan General Rule claim
    /// Required only for a qualified pension starting on or after November 19, 1996. Use age actually attained on the original starting date, not current age or the nearest-birthday table age. The fixed-period or single-life contract must guarantee at least five years of payments, even after death. Leave absent for a commercial annuity or an earlier qualified-plan start. Other lifetime refund features need their separate expected-return calculation.
    /// [NullText]
    /// This is a commercial annuity or an earlier qualified-plan start
    /// [SetText]
    /// The original qualified-plan annuitant's age was
    #[libertas_number(min = 0)]
    pub qualified_age: Option<u32>,
    /// Original net cost of the contract
    /// After-tax premiums less tax-free pre-annuity returns and other documented basis adjustments, before any refund-feature adjustment. Not the current account value or remaining basis. The App computes the refund adjustment separately for the exclusion ratio; lifetime recovery limits retain this net cost.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Original regular periodic payment in the contract
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub periodic: i64,
    /// Payment duration in the original contract
    pub duration: AnnuityDuration,
    /// Full periodic installments represented by payments received in 2026
    /// Include delayed installments for earlier years, counted once when received, not the number of checks or transfers. For a monthly contract, twelve current-year installments plus three delayed installments means 15. Exclude installments already received or already included in earlier cost recovery. The original periodic amount and exclusion ratio must still apply to every entered installment.
    #[libertas_number(min = 0)]
    pub received: u32,
    /// Partial first-period payment received in 2026
    /// Enter only the fractional original first-period payment received this year, including a delayed first-period payment from an earlier year. Exclude it from the full-installment count and do not repeat amounts received or recovered earlier. Other fractional adjustments need separate treatment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub partial: i64,
    /// Total 2026 cost-of-living increases above the original payments
    /// Enter the actual additional dollars received during 2026 from payment statements, including any midyear increase. Keep the original periodic contract payment above unchanged. These increases are fully taxable and do not increase basis recovery. Enter zero if none; other contract changes require review.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub increase: i64,
    /// Cost recovered tax-free before 2026
    /// From retained returns/worksheets. Enter zero for a new annuity. Do not subtract this from original cost when calculating the exclusion ratio.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub recovered: i64,
    /// Fixed-payment General Rule pension or annuity with complete, unchanged records
    /// A qualified-plan pension must either have started before November 19, 1996 using the General Rule, or meet the age-75 and five-year guarantee rules using the entered original age and guaranteed installments. Prior payments must consistently use the General Rule, without an irrevocable Simplified Method election. Not variable, foreign, charitable or exchanged; inherited contracts are excluded except the designated survivor of the entered joint-life contract. For lifetime or temporary-life contracts, Tables V through VIII apply to the entire cost: contributions after June 30, 1986, or a documented election to use these tables. No allocation to older sex-specific tables. No refund feature except the entered single-life guaranteed installments, payment changes other than the entered cost-of-living increases, changed basis, surrender, loan, simultaneous shared-recipient allocation or unentered adjustment. All this contract's 2026 payments are entered once; prior recovery used the General Rule. Starting dates before July 2, 1986 still need separate treatment. For July–December 1986 starts, prior tax-free recovery can exceed original cost; the periodic exclusion continues without a lifetime cost cap. No recovers no cost: the whole payment is taxed, with a notice.
    pub eligible: bool,
}
impl GeneralAnnuity {
    pub(crate) fn qualified_method(self) -> bool {
        let five_years = match self.duration {
            AnnuityDuration::Fixed { payments } => payments >= 5 * self.frequency.count(),
            AnnuityDuration::SingleLife { guaranteed, .. } => {
                guaranteed >= self.periodic * i64::from(self.frequency.count()) * 5
            }
            _ => false,
        };
        self.started < 19961119 || (self.qualified_age.is_some_and(|age| age >= 75) && five_years)
    }
    pub(crate) fn valid(self, gross: i64) -> bool {
        valid_date(self.started)
            && (19860702..=20261231).contains(&self.started)
            && (0..=100000000000).contains(&self.cost)
            && (1..=100000000000).contains(&self.periodic)
            && self.duration.valid(self.frequency, self.periodic)
            && (!matches!(self.duration, AnnuityDuration::SingleLife { guaranteed, .. } if guaranteed > 0)
                || self.partial == 0)
            && match self.duration {
                AnnuityDuration::TemporaryLife { months, .. } => {
                    self.received.saturating_add(u32::from(self.partial > 0))
                        <= months.div_ceil(12 / self.frequency.count())
                }
                AnnuityDuration::Fixed { payments } => {
                    self.received.saturating_add(u32::from(self.partial > 0)) <= payments
                }
                _ => true,
            }
            && (0..=100000000000).contains(&self.earlier_payments)
            && (self.earlier_payments == 0
                || matches!(self.duration, AnnuityDuration::JointLife { survivor: true, .. }))
            && self.qualified_age.is_none_or(|age| age <= 120)
            && self.received <= 486
            && self.received + u32::from(self.partial > 0)
                <= ((2026 - self.started / 10000) * 12 + 13 - self.started / 100 % 100)
                    .div_ceil(12 / self.frequency.count())
            && (0..self.payment()).contains(&self.partial)
            && (0..=100000000000).contains(&self.recovered)
            && (self.started < 19870101 || self.recovered <= self.cost)
            && (0..=100000000000).contains(&self.increase)
            && (self.increase == 0 || self.received > 0 || self.partial > 0)
            && gross == self.payment() * i64::from(self.received) + self.partial + self.increase
            && (self.started < 20260101 || self.recovered == 0)
    }
    fn payment(self) -> i64 {
        match self.duration {
            AnnuityDuration::JointLife { survivor: true, survivor_payment, .. } => survivor_payment,
            _ => self.periodic,
        }
    }
    pub(crate) fn expected(self) -> i64 {
        crate::kernel::ratio::half_offset_i64(
            self.duration.expected_tenths(self.periodic, self.frequency),
            10,
        )
    }
    pub(crate) fn refund_adjustment(self) -> i64 {
        let guaranteed = match self.duration {
            AnnuityDuration::SingleLife { guaranteed, .. } => guaranteed,
            _ => 0,
        };
        // Pub. 939 rounds the adjustment to whole dollars. Keep net cost intact
        // for the post-1986 recovery cap and final-annuitant deduction.
        let percent = self.duration.refund_percent(self.frequency, self.periodic).unwrap_or(0);
        phase::cap(
            crate::kernel::ratio::half_offset_i64(
                phase::cap(self.cost, [guaranteed]) * percent,
                10000,
            ) * 100,
            [self.cost],
        )
    }
    pub(crate) fn ratio(self) -> i64 {
        let expected = self.duration.expected_tenths(self.periodic, self.frequency).max(1);
        (crate::kernel::ratio::half_offset_i64(
            (self.cost - self.refund_adjustment()) * 10000,
            expected,
        ))
        .min(1000)
    }
    pub(crate) fn ends_at_death(self, death: u32) -> bool {
        match self.duration {
            AnnuityDuration::SingleLife { guaranteed, delay, .. } if guaranteed > 0 => {
                let installments =
                    crate::kernel::ratio::trunc_i64(guaranteed + self.periodic - 1, self.periodic);
                let last = delay + (installments as u32 - 1) * (12 / self.frequency.count());
                let last_date =
                    if last == 0 { Some(self.started) } else { term_end(self.started, last) };
                last_date.is_some_and(|date| date <= death)
            }
            AnnuityDuration::Fixed { .. } => false,
            AnnuityDuration::TemporaryLife { months, .. } => {
                term_end(self.started, months).is_some_and(|end| death < end)
            }
            _ => true,
        }
    }
    pub(crate) fn earlier_recovery(self) -> i64 {
        let amount =
            crate::kernel::ratio::half_offset_i64(self.earlier_payments * self.ratio(), 1000);
        if self.started < 19870101 {
            amount
        } else {
            phase::cap(
                amount,
                [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    self.cost,
                    self.recovered,
                )
                .get()],
            )
        }
    }
    pub(crate) fn excluded(self) -> i64 {
        let ratio = self.ratio();
        // Preserve fractional cents per payment; Pub. 939's $125 x 3 x .631
        // example excludes $236.63, not $236.64 from rounding each payment.
        let original_payments = self.payment() * i64::from(self.received) + self.partial;
        let annual = crate::kernel::ratio::half_offset_i64(original_payments * ratio, 1000);
        if self.started < 19870101 {
            annual
        } else {
            phase::cap(
                annual,
                [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    crate::kernel::carry::native_balance(self.cost, self.recovered),
                    self.earlier_recovery(),
                )
                .get()],
            )
        }
    }
}

/// Contract payment frequency
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AnnuityFrequency {
    /// Annual
    Annual,
    /// Semiannual
    Semiannual,
    /// Quarterly
    Quarterly,
    /// Monthly
    Monthly,
}
impl AnnuityFrequency {
    fn count(self) -> u32 {
        match self {
            Self::Annual => 1,
            Self::Semiannual => 2,
            Self::Quarterly => 4,
            Self::Monthly => 12,
        }
    }
    fn adjustment(self, delay: u32) -> Option<i64> {
        let table: &[i64] = match self {
            Self::Annual => &[5, 5, 4, 3, 2, 1, 0, 0, -1, -2, -3, -4, -5],
            Self::Semiannual => &[2, 2, 1, 0, 0, -1, -2],
            Self::Quarterly => &[1, 1, 0, -1],
            Self::Monthly => &[0, 0],
        };
        table.get(delay as usize).copied()
    }
}

/// Expected variable-annuity installments
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum VariablePayments {
    /// One life ending no later than a fixed term
    /// Payments stop at death or term expiration, whichever comes first. Table VIII applies to the entire investment. Its factor does not use the ordinary-lifetime timing adjustment.
    TemporaryLife {
        /// Nearest-birthday age at the calculation date
        /// Original age for a new contract; current age when refiguring the remaining term.
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
        /// Maximum term in months from the calculation period
        /// Original term for a new contract, which must exceed twelve months; remaining term for a shortfall election. The App rounds to the nearest whole year for Table VIII. Less than six remaining months or terms over forty years need further treatment.
        // Tax structure: Pub. 939 Table VIII covers temporary terms through 40 years.
        #[libertas_number(min = 6, max = 480)]
        months: u32,
    },
    /// Two lives, unchanged benefit units through the last survivor
    /// The same number of annuity units continues after the first death; dollar payments may vary with investment performance. A survivor reduction or changed benefit formula needs another calculation.
    Joint {
        /// Primary annuitant's nearest-birthday age
        /// At original annuitization for a new contract; at the current first payment period when refiguring while both annuitants are alive.
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
        /// Second annuitant's nearest-birthday age at the same date
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        survivor_age: u32,
        /// Whole months from that period's start to payment
        // Tax structure: Pub. 939 payment-frequency timing adjustments end at twelve months.
        #[libertas_number(min = 0, max = 12)]
        delay: u32,
    },

    /// Fixed number of installments
    Fixed {
        /// Total installments promised by the contract
        /// For a new contract use its entire term. For a shortfall adjustment use installments remaining, including the current payment.
        #[libertas_number(min = 1)]
        payments: u32,
    },
    /// Payments for one lifetime without a refund or guaranteed period
    Life {
        /// Nearest-birthday age at the calculation date
        /// At the original starting date for a new contract; at the current payment period for a shortfall adjustment. Table V must apply to the entire investment.
        // Tax structure: Pub. 939 life-expectancy tables cover nearest-birthday ages through 115.
        #[libertas_number(min = 5, max = 115)]
        age: u32,
        /// Whole months from the period starting date to its payment
        /// At original annuitization for a new contract; from the current first period to payment when refiguring. The App applies the IRS timing adjustment for the selected payment frequency. Use the actual contractual timing, not a tax calculation.
        // Tax structure: Pub. 939 payment-frequency timing adjustments end at twelve months.
        #[libertas_number(min = 0, max = 12)]
        delay: u32,
    },
}
impl VariablePayments {
    fn tenths(self, frequency: AnnuityFrequency) -> Option<i64> {
        match self {
            Self::TemporaryLife { age, months } if (6..=480).contains(&months) => {
                // 26 CFR 1.72-5(a)(3): no (a)(2) timing adjustment for Table VIII.
                Some(
                    crate::annuity_tables::temporary_multiple(age, (months + 6) / 12)?
                        * i64::from(frequency.count()),
                )
            }
            Self::Fixed { payments } if (1..=1500).contains(&payments) => {
                Some(i64::from(payments) * 10)
            }
            Self::Joint { age, survivor_age, delay } => {
                let multiple = crate::annuity_tables::joint_multiple(age, survivor_age)?
                    + frequency.adjustment(delay)?;
                (multiple > 0).then_some(multiple * i64::from(frequency.count()))
            }
            Self::Life { age, delay } => {
                // Pub. 939 adjustment table; 26 CFR 1.72-5(a)(2).
                let adjustment = frequency.adjustment(delay)?;
                let multiple = AnnuityDuration::life_multiple(age)? + adjustment;
                (multiple > 0).then_some(multiple * i64::from(frequency.count()))
            }
            _ => None,
        }
    }
}

/// Election to refigure the annual tax-free amount after a shortfall
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct VariableRefigure {
    /// Number of full prior years with unelected shortfalls
    /// Each included year used the same retained annual exclusion and had total payments below that allowance. Exclude years whose shortfall was already used in an election. Partial-year allowances or changed exclusions require a separate history calculation.
    #[libertas_number(min = 1)]
    #[libertas_default(1)]
    pub years: u32,
    /// Total payments actually received in those shortfall years
    /// Add the payer's statement amounts, including zero-payment years. The App calculates the unused allowance; do not enter a tax shortfall.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Remaining payment facts at the current first payment period
    pub remaining: VariablePayments,
}

/// Variable-annuity calculation history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum VariableHistory {
    /// First payment received in 2026
    New { expected: VariablePayments },
    /// Continue a retained General Rule worksheet
    Prior {
        /// Annual tax-free amount from the preceding worksheet
        /// Retain any previously elected adjustments; do not substitute the taxable amount or remaining investment.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        annual: i64,
        /// Total cost recovered tax-free through 2025
        /// Include all earlier annuitants' recovery through 2025. Do not include the first annuitant's 2026 recovery supplied separately in the earlier-payment facts.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        recovered: i64,
        /// Elect to adjust future tax-free payments after the previous shortfall
        /// None keeps the existing tax-free amount. Choosing an adjustment requires the statement described in the result; the App does not submit a return or election.
        /// [NullText]
        /// Keep the existing annual tax-free payment amount
        /// [SetText]
        /// I elect to adjust future tax-free payments using
        #[libertas_default($null)]
        refigure: Option<VariableRefigure>,
    },
}

/// First annuitant's earlier current-year variable payments
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct VariableEarlier {
    /// Scheduled installments belonging to the first annuitant
    /// Include zero-dollar scheduled installments. These precede the survivor's entered installments and belong on the first annuitant's income entry or return.
    // Tax structure: payment months and scheduled annual installments cannot exceed twelve.
    #[libertas_number(min = 1, max = 12)]
    pub payments: u32,
    /// Gross receipts for those installments
    /// Source payment statements, not the calculated tax-free recovery. Do not include this amount in the survivor's gross receipts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
}

/// Retained contract term
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum VariableTerm {
    /// Fixed term independent of anyone's life
    Fixed {
        /// Contract expiration date
        /// First date outside the contractual term. Use contract records, not the date of a delayed transfer; no new payment period may start on or after this date.
        #[libertas_date_only]
        ends: u32,
    },
    /// One life ending no later than a fixed date
    TemporaryLife {
        /// Contract expiration date if the annuitant survives
        /// First date outside the contractual term. Payments end at death if earlier. Retain the original contract date; a shortfall election does not extend the term.
        #[libertas_date_only]
        ends: u32,
    },
}
impl VariableTerm {
    fn end(self) -> u32 {
        match self {
            Self::Fixed { ends } | Self::TemporaryLife { ends } => ends,
        }
    }
}

/// Variable commercial annuity
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct VariableAnnuity {
    /// Retained ending date for a continuing finite-term contract
    /// Supply when continuing a fixed-period or temporary-life contract without refiguring its term. Leave absent for lifetime-only contracts and new contracts whose term is already entered in expected-payment facts. If a shortfall election also supplies remaining-term facts, both must agree. Earlier-recipient transitions with a finite term need further treatment.
    /// [NullText]
    /// No retained ending date is needed for this contract
    /// [SetText]
    /// The continuing finite-term contract ends as follows
    pub term: Option<VariableTerm>,
    /// First annuitant's payments earlier in this year
    /// Use only on the designated joint-life survivor's entry in the year of the first death, with unchanged benefit units and the same annual exclusion for both recipients. The App calculates earlier recovery before limiting this survivor's recovery. Do not repeat current-year recovery in the prior-recovery field. A new shortfall election during this transition needs further calculation.
    /// [NullText]
    /// No first-annuitant payments precede mine in 2026
    /// [SetText]
    /// The first annuitant's earlier 2026 payments were
    pub earlier: Option<VariableEarlier>,
    /// Original annuity starting date
    #[libertas_date_only]
    pub started: u32,
    /// Annuitant's age on the original starting date
    /// Retain for the election statement. Lifetime expectation uses the nearest-birthday age specified separately.
    #[libertas_number(min = 0)]
    pub age: u32,
    /// First day of the first payment period paid in 2026
    /// This may be in late 2025 when payment is due in arrears. Enter the period start, not the receipt date. If death preceded every 2026 installment, enter the period start for the first installment that had been scheduled.
    #[libertas_date_only]
    pub period: u32,
    /// Original net investment in the contract
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Regular payment schedule in the contract
    pub frequency: AnnuityFrequency,
    /// Number of scheduled installments for 2026
    /// Include zero-dollar installments. All scheduled payments must be represented by the gross annual total. Enter the partial first-year count when applicable. On the owner's final return, include only that owner's scheduled installments through death, including zero-dollar installments; exclude later payments belonging to a beneficiary or estate. Enter zero only if death preceded every installment belonging to this owner, with zero gross receipts; a scheduled installment of zero dollars still counts as an installment. On a designated survivor's transition-year entry, the earlier and current counts together cover the contract's schedule. Continuing finite-term contracts include only periods before their retained expiration date; other continuing contracts require a full-year schedule.
    // Tax structure: payment months and scheduled annual installments cannot exceed twelve.
    #[libertas_number(min = 0, max = 12)]
    pub payments: u32,
    pub history: VariableHistory,
    /// Complete qualifying contract and payment records
    /// Nonqualified commercial annuity, post-1986 start, annual, semiannual, quarterly or monthly payment schedule for 2026, unchanged payment frequency, no refund feature, guaranteed lifetime period, exchange, shared recipient, surrender, loan, basis change or other adjustment. Lifetime or temporary-life calculations use Tables V/VI/VIII for the entire investment by contribution dates or a documented election. A joint-life contract keeps the same benefit units through the last survivor. Enter each recipient separately and supply earlier first-annuitant payment facts on the survivor entry when both received installments in 2026; prior recovery includes both recipients through 2025. Inheritance is excluded except the designated joint-life survivor. For a shortfall election after the first death use the surviving single life; while both are alive use joint ages. Earlier elections are reflected in the retained worksheet. Enter only previously unelected full-year shortfalls with the same annual allowance, each year below that allowance. The owner's entered death date permits a partial final-year schedule; all installments belonging to that owner must be included, without beneficiary or estate income. For a continuing finite term, retain its expiration date even when no new shortfall election is made. Other historical patterns, missed or delayed installments and other frequencies require review. No recovers no cost: the whole payment is taxed, with a notice.
    pub eligible: bool,
}
impl VariableAnnuity {
    pub(crate) fn valid(self) -> bool {
        valid_date(self.started)
            && (19870101..=20261231).contains(&self.started)
            && valid_date(self.period)
            && (20250101..=20261231).contains(&self.period)
            && (self.period >= 20260101
                || self.period / 100 % 100 >= 13 - 12 / self.frequency.count())
            && self.period >= self.started
            && self.age <= 120
            && self.payments <= self.frequency.count()
            && self.term.is_none_or(|term| {
                valid_date(term.end())
                    && term.end() > self.started
                    && self.earlier.is_none()
                    && match self.history {
                        VariableHistory::Prior { refigure: None, .. } => true,
                        VariableHistory::Prior { refigure: Some(r), .. } => {
                            match (term, r.remaining) {
                                (
                                    VariableTerm::TemporaryLife { ends },
                                    VariablePayments::TemporaryLife { months, .. },
                                ) => term_end(self.period, months) == Some(ends),
                                (
                                    VariableTerm::Fixed { ends },
                                    VariablePayments::Fixed { payments },
                                ) => {
                                    term_end(
                                        self.period,
                                        payments.saturating_mul(12 / self.frequency.count()),
                                    ) == Some(ends)
                                }
                                _ => false,
                            }
                        }
                        _ => false,
                    }
            })
            && self.earlier.is_none_or(|earlier| {
                (1..=self.frequency.count()).contains(&earlier.payments)
                    && self.payments + earlier.payments <= self.frequency.count()
                    && (0..=100000000000).contains(&earlier.gross)
                    && self.period / 10000 * 12 + self.period / 100 % 100
                        >= (if matches!(self.history, VariableHistory::New { .. }) {
                            self.started / 10000 * 12 + self.started / 100 % 100
                        } else {
                            2026 * 12 + 1 - 12 / self.frequency.count()
                        }) + earlier.payments * (12 / self.frequency.count())
            })
            && (0..=100000000000).contains(&self.cost)
            && match self.history {
                VariableHistory::New { expected } => {
                    self.started >= 20250101
                        && (self.period == self.started || self.earlier.is_some())
                        && self
                            .earlier
                            .is_none_or(|_| matches!(expected, VariablePayments::Joint { .. }))
                        && (self.started < 20260101
                            || self.payments + self.earlier.map_or(0, |e| e.payments)
                                <= (13 - self.started / 100 % 100)
                                    .div_ceil(12 / self.frequency.count()))
                        && expected.tenths(self.frequency).is_some()
                        && !matches!(expected, VariablePayments::Fixed { payments } if payments <= self.frequency.count())
                        && !matches!(expected, VariablePayments::TemporaryLife { months, .. } if months <= 12)
                        && match expected {
                            VariablePayments::Life { age, .. }
                            | VariablePayments::TemporaryLife { age, .. }
                            | VariablePayments::Joint { age, .. } => {
                                age == self.age || age == self.age + 1
                            }
                            _ => true,
                        }
                }
                VariableHistory::Prior { annual, recovered, refigure } => {
                    self.started < 20260101
                        && (self.earlier.is_some()
                            || self.period < 20260101
                            || self.period / 100 % 100 <= 12 / self.frequency.count())
                        && (0..=100000000000).contains(&annual)
                        && (0..=self.cost).contains(&recovered)
                        && refigure.is_none_or(|v| {
                            (1..=2026 - self.started / 10000).contains(&v.years)
                                && (0..annual * i64::from(v.years)).contains(&v.paid)
                                && recovered >= v.paid
                                && v.remaining.tenths(self.frequency).is_some()
                        })
                }
            }
    }
    fn known_term(self) -> Option<(u32, VariablePayments)> {
        match self.history {
            VariableHistory::New { expected } => Some((self.started, expected)),
            VariableHistory::Prior { refigure: Some(refigure), .. } => {
                Some((self.period, refigure.remaining))
            }
            _ => None,
        }
    }
    fn contract_end(self) -> Option<u32> {
        if let Some(term) = self.term {
            return Some(term.end());
        }
        match self.known_term() {
            Some((start, VariablePayments::TemporaryLife { months, .. })) => {
                term_end(start, months)
            }
            Some((start, VariablePayments::Fixed { payments })) => {
                term_end(start, payments.saturating_mul(12 / self.frequency.count()))
            }
            _ => None,
        }
    }
    pub(crate) fn ends_at_death(self, death: u32) -> bool {
        if let Some(term) = self.term {
            return matches!(term, VariableTerm::TemporaryLife { ends } if death < ends);
        }
        match self.known_term() {
            Some((_, VariablePayments::Fixed { .. })) => false,
            Some((start, VariablePayments::TemporaryLife { months, .. })) => {
                term_end(start, months).is_some_and(|end| death < end)
            }
            _ => true,
        }
    }
    pub(crate) fn schedule_valid(self, death: Option<u32>) -> bool {
        let last_period = if self.payments <= 1 {
            Some(self.period)
        } else {
            term_end(self.period, (self.payments - 1) * (12 / self.frequency.count()))
        };
        if self.payments > 0
            && last_period.is_none_or(|last| {
                last > 20261231 || self.contract_end().is_some_and(|end| last >= end)
            })
        {
            return false;
        }
        if let Some(death) = death {
            if self.payments == 0 {
                return self.started <= death;
            }
            // Period starts may precede receipt when installments are in arrears.
            // The source confirmation supplies the complete decedent-only schedule.
            last_period.is_some_and(|last| last <= death)
        } else {
            let term_count = self.contract_end().map(|end| {
                (0..self.frequency.count())
                    .take_while(|index| {
                        let period = if *index == 0 {
                            Some(self.period)
                        } else {
                            term_end(self.period, index * (12 / self.frequency.count()))
                        };
                        period.is_some_and(|date| date < end && date <= 20261231)
                    })
                    .count() as u32
            });
            let expected = if matches!(self.history, VariableHistory::New { .. })
                && self.started >= 20260101
            {
                (13 - self.started / 100 % 100).div_ceil(12 / self.frequency.count())
            } else {
                self.frequency.count()
            };
            self.payments > 0
                && if let Some(earlier) = self.earlier {
                    self.payments + earlier.payments == expected
                } else {
                    matches!(self.history, VariableHistory::New { .. })
                        || self.payments == term_count.unwrap_or(self.frequency.count())
                }
        }
    }
    pub(crate) fn recovered(self) -> i64 {
        match self.history {
            VariableHistory::New { .. } => 0,
            VariableHistory::Prior { recovered, .. } => recovered,
        }
    }
    pub(crate) fn annual(self) -> i64 {
        let divide = |amount: i64, expected: VariablePayments| {
            let n = expected.tenths(self.frequency).unwrap_or(1);
            crate::kernel::ratio::half_offset_i64(
                amount * 10 * i64::from(self.frequency.count()),
                n,
            )
        };
        match self.history {
            VariableHistory::New { expected } => divide(self.cost, expected),
            VariableHistory::Prior { annual, refigure, .. } => {
                annual
                    + refigure.map_or(0, |v| {
                        divide(
                            crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                                annual * i64::from(v.years),
                                v.paid,
                            )
                            .get(),
                            v.remaining,
                        )
                    })
            }
        }
    }
    fn allowance(self, payments: u32) -> i64 {
        let n = i64::from(self.frequency.count());
        crate::kernel::ratio::half_offset_i64(self.annual() * i64::from(payments), n)
    }
    pub(crate) fn earlier_recovery(self) -> i64 {
        self.earlier.map_or(0, |earlier| {
            phase::cap(
                phase::cap(self.allowance(earlier.payments), [earlier.gross]),
                [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    self.cost,
                    self.recovered(),
                )
                .get()],
            )
        })
    }
    pub(crate) fn excluded(self, gross: i64) -> i64 {
        let earlier = self.earlier.map_or(0, |e| e.payments);
        // Allocate cents by cumulative period count so splitting the same annual
        // allowance between recipients cannot create or lose a rounding cent.
        // Subtract the earlier allowance, not actual recovery: a short receipt
        // does not automatically transfer unused exclusion to the survivor.
        phase::cap(
            phase::cap(self.allowance(earlier + self.payments) - self.allowance(earlier), [gross]),
            [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                crate::kernel::carry::native_balance(self.cost, self.recovered()),
                self.earlier_recovery(),
            )
            .get()],
        )
    }
}

/// Contract cost unrecovered when the final annuitant dies
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnuityLoss {
    /// Retirement income entry for this contract
    /// Use the entry belonging to the deceased final annuitant. Include one zero-dollar entry if no payment was received during 2026. The App uses that entry's calculated tax-free recovery; do not enter the same contract twice.
    #[libertas_number(min = 1)]
    pub income_id: i64,
    /// Original annuity starting date
    #[libertas_date_only]
    pub started: u32,
    /// Net investment before recovery, from original contract records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Tax-free investment recovered through 2025
    /// Include recoverable amounts required by the retained worksheet and all earlier annuitants' recovery. Current-year recovery is calculated from the linked income entry. A linked Simplified Method worksheet includes all common recovery in its earlier and entered payment months. General Rule and variable-annuity survivor worksheets also include calculated recovery from the first annuitant's earlier current-year payments. Do not enter those amounts again as other payouts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub recovered: i64,
    /// Other tax-free contract payouts reducing unrecovered investment
    /// Include tax-free refund or death-settlement amounts not already counted as periodic recovery. Do not include a payout twice.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub refunds: i64,
    /// All annuity payments ceased at this final annuitant's death
    /// This is the final annuitant's 2026 return; no surviving annuitant, continuing guaranteed payments, transfer, surrender, IRA loss or duplicate deduction. Original investment and recoveries are complete and agree with the linked entry's basis calculation. Retirement income entered here belongs to the decedent, not the estate or a beneficiary. The payer's notice shows whether payments stopped. No claims no deduction.
    pub eligible: bool,
}
impl AnnuityLoss {
    pub(crate) fn valid(&self) -> bool {
        self.income_id > 0
            && valid_date(self.started)
            && self.started <= 20261231
            && (0..=100000000000).contains(&self.cost)
            && (0..=self.cost).contains(&self.recovered)
            && (0..=self.cost - self.recovered).contains(&self.refunds)
    }
    pub(crate) fn amount(&self, current_recovery: i64) -> i64 {
        phase::floor(self.cost - self.recovered - self.refunds - current_recovery, 0)
    }
}
