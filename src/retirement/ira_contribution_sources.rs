use crate::dates::source_valid_date as valid_date;
use crate::kernel::phase;
use crate::kernel::ratio::exact::Exact;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// 2026 Form 5498 boxes 1/10: https://www.irs.gov/instructions/i1099r
// 2026 Saver's Credit versus 2027 Saver's Match: https://www.irs.gov/irb/2026-35_IRB
/// Completed next-year IRA deposits designated for this tax year
/// These are a subset of the retained regular contributions below, not additional contributions. Use one entry per filer after the last ordinary deposit; the date is the latest actual deposit for either account type. Planned but unpaid contributions remain only in the regular totals and, if distributions require it, the separate IRA basis timing worksheet.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraNextYearDeposits {
    /// Last actual deposit date
    /// Every included traditional or Roth deposit was received between January 1 and April 15, 2027. Use the latest date when there were multiple deposits or custodians.
    #[libertas_date_only]
    pub last_deposited_on: u32,
    /// Traditional IRA principal deposited in 2027 for 2026
    /// Include this amount in retained regular traditional contributions below exactly once. Do not include returned contributions, recharacterization transfers, rollovers, conversions, SEP or SIMPLE employer deposits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub traditional: i64,
    /// Roth IRA principal deposited in 2027 for 2026
    /// Include this amount in retained regular Roth contributions below exactly once. Do not include returned contributions, recharacterization transfers, rollovers, conversions, SEP or SIMPLE employer deposits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub roth: i64,
    /// Custodian received the 2026 designation
    /// Confirm each 2027 deposit was actually received by its custodian and explicitly designated for 2026. The custodian reports it on the 2026 Form 5498; a deposit designated for 2027 belongs to the 2027 return instead, so No takes it out of the 2026 totals.
    pub designation_confirmed: bool,
}

/// Annual IRA contributions for one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalIraContribution {
    /// Regular traditional IRA contributions designated for 2026
    /// Aggregate retained contributions across all accounts after any recharacterization. Include planned contributions made by the filing deadline without extensions. Exclude principal moved to Roth and principal returned to you; a returned contribution is treated as never contributed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub traditional: i64,
    /// Regular Roth IRA contributions designated for 2026
    /// Aggregate retained contributions across all accounts after any recharacterization. Exclude principal moved to traditional and principal returned to you; a returned contribution is treated as never contributed and does not create Roth contribution basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub roth: i64,
    /// Traditional contributions voluntarily designated nondeductible
    /// Leave zero to claim the maximum allowed deduction. Enter the full traditional contribution to claim no deduction, or designate part of it. Income limits may make more nondeductible than you elect. Nondeductible contributions still use the contribution limit and must be reported on Form 8606; this choice is not a Roth conversion or a promise of lower tax.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub nondeductible_election: i64,
    /// Covered by a retirement plan at work during 2026
    /// Check the "Retirement plan" box 13 on each W-2 and your own SEP, SIMPLE or solo 401(k) records, even if no IRA contribution was made.
    pub covered_at_work: bool,
    /// Actual 2027 deposits designated for 2026, if any
    /// Enter only completed regular deposits, as a subset of the traditional and Roth totals above. The 2026 deduction, nondeductible basis, Roth contribution basis, and Saver's Credit use those totals once. A 2026-designated deposit cannot also support the 2027 Saver's Match.
    pub next_year_deposits: Option<IraNextYearDeposits>,
    /// Earlier qualifying 529-to-Roth transfers for this beneficiary
    /// Combine all prior tax years and every 529 account for this person; this year's transfers are their Forms 1099-Q on the Income page, never repeated here. The App applies the $35,000 lifetime ceiling once across this person's 529 sources.
    #[libertas_money("USD", 2)]
    // Tax structure: Section 529-to-Roth IRA transfers have a $35,000 lifetime limit.
    #[libertas_number(min = 0, max = $QTP_ROTH_LIFETIME_LIMIT_CENTS)]
    #[libertas_default(0)]
    pub qtp_roth_lifetime_prior: i64,
    /// 2026 contributions returned to you in 2027, with their earnings or loss
    /// Each one is excluded from the totals above; its earnings are 2026 income. A contribution returned in 2026 is its Form 1099-R (code 8) on the Retirement page.
    pub returned_next_year: Vec<crate::ReturnedContribution>,
    /// Excess contributions and year-end values, if any
    /// Needed when 2026 contributions exceed the limit the App computes, or a 2025 Form 5329 carried a traditional or Roth IRA excess. Leave absent when there is no excess.
    pub excess: Option<crate::IraExcess>,
}

/// IRA contributions
/// Compensation comes from entered wages, supported business earnings and qualifying decree-related taxable alimony, including payer recapture. Report rollovers and conversions in Income, never as regular contributions. Traditional contributions combined with IRA distributions require the owner-level basis worksheet above. Each filer may designate part or all of their retained traditional contributions as nondeductible; leave the election at zero to claim the maximum allowed deduction. Ordinary deposits made in 2027 for 2026 require the actual date and custodian designation above. Contributions above the limit and a 2025 excess carried in use each filer's excess record. Recharacterizations of contributions deposited outside 2026 or transfers after the supported deadline, repayments and special compensation need separate treatment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalIras {
    /// Your contributions and workplace coverage
    pub taxpayer: FederalIraContribution,
    /// Spouse situation for the IRA worksheet
    pub spouse: IraSpouse,
}

impl FederalIraContribution {
    /// One filer with nothing entered; `covered_at_work` is re-derived on
    /// every save of the IRA page.
    pub(crate) const NONE: Self = Self {
        traditional: 0,
        roth: 0,
        nondeductible_election: 0,
        covered_at_work: false,
        next_year_deposits: None,
        qtp_roth_lifetime_prior: 0,
        returned_next_year: Vec::new(),
        excess: None,
    };
    /// Whether the filer has any IRA fact beyond the derived workplace
    /// coverage.
    pub(crate) fn entered(&self) -> bool {
        Self { covered_at_work: false, ..self.clone() } != Self::NONE
    }
}
impl FederalIras {
    /// No IRA contributions.
    pub(crate) const NONE: Self =
        Self { taxpayer: FederalIraContribution::NONE, spouse: IraSpouse::NotApplicable };
    /// Whether either filer has an IRA fact; the spouse variant and the
    /// coverage answers alone follow the return and decide nothing.
    pub(crate) fn entered(&self) -> bool {
        self.taxpayer.entered() || self.spouse.as_ref().is_some_and(|f| f.entered())
    }
}

/// Spouse situation for IRA contributions and workplace coverage
// The schema needs the concrete joint contribution payload in this union variant.
#[allow(clippy::large_enum_variant)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum IraSpouse {
    /// Not filing jointly or separately while living together
    NotApplicable,
    /// Joint return
    Joint {
        /// Spouse’s contributions and workplace coverage, including when contributions are zero
        facts: FederalIraContribution,
    },
    /// Separate return; lived together during the year
    SeparateTogether {
        /// Spouse covered by a workplace retirement plan
        /// Check the "Retirement plan" box 13 on the spouse's W-2s.
        covered_at_work: bool,
    },
}
impl IraSpouse {
    pub fn as_ref(&self) -> Option<&FederalIraContribution> {
        match self {
            Self::Joint { facts } => Some(facts),
            _ => None,
        }
    }
    pub fn as_mut(&mut self) -> Option<&mut FederalIraContribution> {
        match self {
            Self::Joint { facts } => Some(facts),
            _ => None,
        }
    }
    pub fn is_some(&self) -> bool {
        self.as_ref().is_some()
    }
    pub fn iter(&self) -> core::option::IntoIter<&FederalIraContribution> {
        self.as_ref().into_iter()
    }
}
/// Returned contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReturnedContribution {
    /// Kind of IRA
    #[libertas_default(Traditional)]
    pub kind: IraKind,
    /// Contribution principal returned
    /// The original 2026 contribution, even when a loss reduced the cash paid back.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Earnings paid with it, or the loss
    /// The custodian's net income allocable to the contribution; a loss is negative. Positive earnings are 2026 income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub earnings: i64,
    /// Date the custodian paid it back
    #[libertas_date_only]
    // Treas. Reg. 301.9100-2: the six-month corrective-return extension ends October 15, 2027.
    #[libertas_number(min = 20270101, max = 20271015)]
    pub returned_on: u32,
    /// How a return after April 15, 2027 was still timely
    /// Leave empty for a return by April 15, 2027.
    /// [NullText]
    /// The contribution was returned by April 15, 2027
    /// [SetText]
    /// The later return was timely because
    #[libertas_default($null)]
    pub late: Option<LateCorrection>,
}
/// Kind of IRA
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum IraKind {
    /// Traditional IRA
    Traditional,
    /// Roth IRA
    Roth,
}
/// How a return after April 15, 2027 was still timely
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum LateCorrection {
    /// A filing extension to October 15, 2027
    /// Form 4868 was filed by April 15, 2027.
    FilingExtension,
    /// The return was filed on time and the contribution returned within six months after April 15, 2027
    /// Treas. Reg. 301.9100-2: an amended 2026 return reports the earnings, which this App does not file.
    TimelyFiledReturn,
}
impl ReturnedContribution {
    pub(crate) fn valid(&self) -> bool {
        valid_date(self.returned_on)
            && (20270101..=20271015).contains(&self.returned_on)
            && (1..=100_000_000_000).contains(&self.principal)
            && (-self.principal..=100_000_000_000 - self.principal).contains(&self.earnings)
    }
    // Relationships checked when accepting the source record.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |key, field| crate::validation::ValidationIssue::new(key).at_field(field);
        if self.returned_on > 20270415 && self.late.is_none() {
            Some(issue("IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE", "late"))
        } else if self.returned_on <= 20270415 && self.late.is_some() {
            Some(issue("IRA_RETURN_ROUTE_WITHOUT_LATE_DATE", "late"))
        } else if i128::from(self.principal) + i128::from(self.earnings) < 0 {
            Some(issue("IRA_RETURN_LOSS_ABOVE_PRINCIPAL", "earnings"))
        } else {
            None
        }
    }
    // Form 1040 line 4a: the cash the custodian paid back.
    pub(crate) fn gross(&self) -> i64 {
        self.principal + self.earnings
    }
    // Form 1040 line 4b: positive earnings; a loss is nothing.
    pub(crate) fn taxable(&self) -> i64 {
        phase::floor(self.earnings, 0)
    }
}

impl IraNextYearDeposits {
    pub(crate) fn valid(&self) -> bool {
        (20270101..=20271231).contains(&self.last_deposited_on)
            && valid_date(self.last_deposited_on)
            && (0..=100_000_000_000).contains(&self.traditional)
            && (0..=100_000_000_000).contains(&self.roth)
            && (self.traditional > 0 || self.roth > 0)
    }

    pub(crate) fn eligible(&self, facts: &FederalIraContribution) -> bool {
        self.valid()
            && self.last_deposited_on <= 20270415
            && self.designation_confirmed
            && self.traditional <= facts.traditional
            && self.roth <= facts.roth
    }
}

/// Excess IRA contributions and year-end values
/// The App computes the 2026 traditional and Roth IRA limits and any excess itself. These facts complete Form 5329 Parts III and IV: the 6% tax on an uncorrected excess is capped by the account values at year end, and a 2025 excess carried in is reduced by 2026 withdrawals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraExcess {
    /// Value of all your traditional IRAs on December 31, 2026
    /// Include SEP and SIMPLE IRAs and any 2026 contributions made in 2027. Use the custodians' year-end statements.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// Use the Form 8606 values
    pub traditional_value: Option<i64>,
    /// Value of all your Roth IRAs on December 31, 2026
    /// Include any 2026 contributions made in 2027.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub roth_value: i64,
    /// Traditional IRA excess carried from line 16 of your 2025 Form 5329
    /// Enter it only when line 17 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_traditional: i64,
    /// Roth IRA excess carried from line 24 of your 2025 Form 5329
    /// Enter it only when line 25 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_roth: i64,
    /// Part of your 2026 withdrawals that returned an excess from a year before 2026
    /// Form 5329 line 12: a traditional IRA excess contribution for 2025 or earlier paid back to you in 2026 after that year's return due date, when you took no deduction for it, none was allowable, and that year's total contributions were within its limit. It is not taxable and is not an IRA withdrawal on Form 8606. Leave 0 otherwise; a withdrawal that fails a condition is ordinary income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub returned_prior_excess: i64,
}

impl IraExcess {
    pub(crate) fn valid(&self, form_8606: Option<Exact>) -> bool {
        // A wide known duplicate projection is None only when the original
        // Form8606 operands establish it. Without that source, a value is required.
        // Form8606 remains authoritative over the readonly duplicate, including
        // an older representable duplicate that has not yet followed an edit.
        let consistent = form_8606.is_some() || self.traditional_value.is_some();
        consistent
            && self.traditional_value.is_none_or(|value| value >= 0)
            && [
                self.roth_value,
                self.prior_traditional,
                self.prior_roth,
                self.returned_prior_excess,
            ]
            .iter()
            .all(|v| (0..=100_000_000_000).contains(v))
            && self.returned_prior_excess <= self.prior_traditional
    }
}
