//! Personal SALT source election and Form 8960 allocation.
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// State/local income or general sales tax deduction
/// Choose one; the same return cannot deduct both. Exclude business/rental deductions and taxes paid for someone else.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PersonalTaxElection {
    /// Neither
    None,
    /// Income tax
    Income {
        /// Eligible state/local income taxes paid
        /// Include eligible withholding and payments, net of same-year refunds. This is an amount paid, not a federal deduction or credit. For a refund spanning a prior payment year and 2026, enter its 2026 allocated share here only through this net amount; the refund record separately retains gross payments for reconciliation.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid: i64,
        /// Investment-income allocation source
        allocation: TaxIncomeAllocation,
    },
    /// General sales tax
    Sales {
        /// Eligible general sales taxes paid
        /// Use documented deductible general sales taxes. This branch does not calculate optional state sales-tax tables.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid: i64,
    },
}

/// Income sources for allocating state/local income tax
/// The App allocates the allowed income-tax deduction by the investment-income share. Keep records supporting this reasonable method. This does not affect Form 8615's classification of income taxes as indirect expenses.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxIncomeAllocation {
    /// Current return's investment income divided by AGI
    /// Use when these income amounts reasonably represent the income underlying the tax payments. The App uses gross investment income before investment expenses and caps the ratio at one.
    CurrentReturn,
    /// Other documented source-income amounts
    /// Use source-year or other documented income amounts when current-return AGI is not representative. Enter income, not an allocated tax or percentage.
    SourceIncome {
        /// Investment income underlying these payments
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        investment: i64,
        /// Total income underlying these payments
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        total: i64,
    },
    // The paid tax and current AGI cannot recover a reasonable source-year
    // investment allocation when its supporting income records are unavailable
    // or current-year income is unrepresentative. niit_expense returns None;
    // the NIIT consumer omits the deduction with WARN_NIIT_STATE_TAX_NOT_ALLOCATED.
    /// Not resolved
    /// None of the tax is allocated to investment income, so the net investment income tax may be higher; a note says so where it matters.
    Unresolved,
}
impl PersonalTaxElection {
    pub(crate) fn paid(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Income { paid, .. } | Self::Sales { paid } => *paid,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        if !(0..=100000000000).contains(&self.paid()) {
            return false;
        }
        match self {
            Self::Income {
                allocation: TaxIncomeAllocation::SourceIncome { investment, total },
                ..
            } => {
                (0..=100000000000).contains(investment)
                    && (1..=100000000000).contains(total)
                    && investment <= total
            }
            _ => true,
        }
    }
    pub(crate) fn niit_expense(
        &self,
        allowed_salt: i64,
        total_salt: &crate::kernel::ratio::exact::Exact,
        investment: i64,
        agi: i64,
    ) -> Option<i64> {
        use crate::kernel::ratio::{dollars, fraction};
        let Self::Income { paid, allocation } = self else {
            return Some(0);
        };
        if *paid == 0 || allowed_salt == 0 {
            return Some(0);
        }
        // Apply SALT's cap proportionally to the elected income tax and personal
        // property taxes. Sales and personal-use property taxes have no NII share.
        let income_deduction = crate::kernel::ratio::exact::rounded(
            &(crate::kernel::ratio::exact::whole(allowed_salt)
                * crate::kernel::ratio::exact::whole(dollars(*paid))
                / phase::floor(total_salt.clone(), crate::kernel::ratio::exact::whole(1))),
        )
        .expect("allocated income tax is bounded by the native SALT deduction");
        let (investment, total) = match allocation {
            TaxIncomeAllocation::CurrentReturn => (phase::floor(investment, 0), agi),
            TaxIncomeAllocation::SourceIncome { investment, total } => (*investment, *total),
            TaxIncomeAllocation::Unresolved => return None,
        };
        if total <= 0 {
            return None;
        }
        Some(fraction(income_deduction, phase::cap(investment, [total]), total))
    }
}
