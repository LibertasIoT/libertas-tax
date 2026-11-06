use crate::common::Owner;
use crate::dates::valid_birth;
use crate::kernel::ratio::exact::{self, Exact};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Trustee payment directly to an eligible public charity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QcdPayment {
    /// Inherited traditional IRA income entry ID, if applicable
    /// Leave absent for personal traditional IRAs. Otherwise use the income entry ID for this beneficiary and deceased owner. The gift must be included in that entry's gross distributions; never use another filer's entry or another deceased owner's basis.
    #[libertas_number(min = 1)]
    pub inherited: Option<i64>,
    /// Date paid in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Cash amount the IRA trustee paid directly
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Paid to a split-interest entity under the one-time election
    /// Counts within the annual limit and within the $55,000 one-time limit.
    pub split_interest: bool,
}
/// Qualified charitable distributions for one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QualifiedCharity {
    /// Filer receiving the IRA distributions
    pub owner: Owner,
    /// All direct trustee payments for this filer, including inherited IRAs
    #[libertas_size(min = 1)]
    pub payments: alloc::vec::Vec<QcdPayment>,
    /// Unused post-age-70½ deductible contributions from the prior QCD adjustment worksheet
    /// Use the carryforward from the latest worksheet. For a first worksheet, use the aggregate deductions on prior filed returns beginning with the year you were 70½ at year-end, but not before 2020. Enter zero if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_deductions: i64,
}
impl QualifiedCharity {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.prior_deductions)
            && !self.payments.is_empty()
            && self.payments.len() <= 100
            && self.payments.iter().all(|p| {
                p.inherited.is_none_or(|id| id > 0)
                    && valid_birth(p.date)
                    && p.date / 10000 == 2026
                    && (0..=100000000000).contains(&p.amount)
            })
    }
}
/// Traditional IRA basis for one filer
/// All this filer's IRA distributions use the traditional IRA basis allocation. Conversions, eligible rollovers, the QCD gifts and prior-basis HSA funding are supported; inherited accounts have their own record. Early-distribution exceptions use the taxable amount after this allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraBasis {
    /// Account owner
    pub owner: Owner,
    /// Unused basis from the most recently filed Form 8606 line 14
    /// Use zero if no nondeductible contributions remain. For a form before 2001 use the Total Basis Chart of the instructions. Do not add after-tax employer-plan rollovers entered in current-year income; the App adds those automatically. The Form 8606 line 2 adjustment is entered with the Form 8606.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior: i64,
    /// Adjustment to that basis, Form 8606 line 2
    /// Line 6 of the last Line 15c Worksheet, basis received or transferred in a divorce, a nontaxable plan rollover never reported, a returned excess contribution. Signed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub adjustment: i64,
    /// December 31, 2026 value of all this filer's traditional IRAs
    /// Combine traditional, SEP and SIMPLE accounts, including accounts with no distributions. Do not include Roth or inherited accounts or a spouse's accounts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end: i64,
    /// Contributions for 2026 paid after December 31, 2026
    /// Included in the traditional contribution total entered below. Include all completed 2027 traditional deposits entered there and any planned timely payments by the filing deadline; none shelter a 2026 distribution.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub paid_next_year: i64,
    /// Qualified reservist repayments deposited in 2026
    /// Adds nondeductible basis outside regular contribution limits. Include the deposits in year-end value, but do not add them to prior basis or regular contributions. Leave empty when none.
    /// ----
    /// Original distribution and its repayments
    pub reservist_repayments: alloc::vec::Vec<crate::ReservistRepayment>,
    /// Repayments of special 2026 IRA distributions
    /// Birth/adoption, emergency, domestic-abuse and terminal-illness repayments use their separate worksheet. Leave empty when none. Keep the original distributions and actual year-end balance unchanged in their source entries.
    /// ----
    /// Original distribution and accepted repayments
    pub special_repayments: alloc::vec::Vec<crate::IraRepayment>,
}
impl IraBasis {
    // Form 8606 line 2: the last form's line 14 and the adjustment.
    pub(crate) fn opening_basis_exact(prior: i64, adjustment: i64) -> Exact {
        exact::whole(prior) + exact::whole(adjustment)
    }
    pub(crate) fn form_5329_value_exact(&self) -> Exact {
        exact::whole(self.year_end) + exact::whole(self.paid_next_year)
    }
    pub(crate) fn line_2_exact(&self) -> Exact {
        Self::opening_basis_exact(self.prior, self.adjustment)
    }
    pub(crate) fn reservist_basis_exact(&self) -> Exact {
        self.reservist_repayments.iter().map(|p| p.amount_exact()).sum()
    }
    pub(crate) fn valid(&self) -> bool {
        self.prior >= 0
            && self.line_2_exact() >= exact::whole(0)
            && self.year_end >= 0
            && self.paid_next_year >= 0
            && self.reservist_repayments.iter().all(|p| p.valid())
            && self.special_repayments.iter().all(|p| p.valid())
    }
    // Only confirmed repayments; an unconfirmed deposit stays in the
    // year-end value like any other.
    pub(crate) fn repaid_in_year_exact(&self) -> Exact {
        self.special_repayments
            .iter()
            .filter(|p| p.confirmed)
            .flat_map(|p| &p.payments)
            .filter(|p| p.date / 10000 == 2026)
            .map(|p| exact::whole(p.amount))
            .sum()
    }
}
