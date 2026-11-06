use crate::kernel::cost_recovery::{AmortizationRegime, Month, MonthlySchedule};
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Business start-up costs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessStartup {
    /// No start-up costs or remaining start-up basis
    None,
    /// Costs incurred before the active business began
    Costs {
        /// Total qualifying start-up costs for this business
        /// Enter original qualifying costs, not this year's deduction or remaining balance. Include eligible investigation, advertising, training and other pre-opening costs that would have been deductible in an existing business. Exclude equipment, inventory, acquired-business transaction costs, section 197 intangibles, research, interest, taxes and costs deducted elsewhere. Do not split one business's costs to multiply the initial deduction.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        costs: i64,
        /// Date the active business began
        /// Amortization begins in this month, including a business that started in an earlier year. Investigating an idea or paying start-up costs alone does not establish this date.
        #[libertas_date_only]
        started: u32,
        /// Elect to capitalize instead of deducting and amortizing
        /// No uses the initial deduction and 180-month amortization. Yes preserves the costs as capital basis with no current deduction. For an existing business, use its original irrevocable election; it cannot be changed each year. An affirmative capitalization election belongs on the timely original return for the year the business began, including extensions.
        capitalize: bool,
        /// Complete eligible costs and original election confirmed
        /// No takes no start-up deduction or amortization for this business. Confirm all costs were paid or incurred after October 22, 2004, and qualify under section 195; this business began by the end of 2026 and remained active through year-end. No abandonment, acquisition/disposal, partnership/corporate organizational costs, changed original election, omitted costs, missed prior deduction or accounting-method change other than one entered under Accounting-method changes. For an earlier start, records must agree with the normal initial deduction and amortization since opening. This is one business, entered once.
        /// Check original cost records, the opening date, election and earlier returns.
        eligible: bool,
    },
}
impl BusinessStartup {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Costs { costs, started, .. } => {
                (0..=100000000000).contains(costs) && crate::federal_rules::valid_birth(*started)
            }
        }
    }
    // Costs whose eligibility is not confirmed are carried as capital with no
    // deduction, the same as an election to capitalize.
    pub(crate) fn deducted(&self) -> bool {
        matches!(self, Self::Costs { capitalize: false, eligible: true, .. })
    }
    pub(crate) fn initial(&self) -> i64 {
        match *self {
            Self::Costs { costs, started, .. } if self.deducted() => {
                // Section 195(b)(3) temporarily enlarged the deduction in 2010.
                let (limit, phaseout) =
                    if started / 10000 == 2010 { (1000000, 6000000) } else { (500000, 5000000) };
                phase::cap(costs, [phase::phase_out(limit, phase::floor(costs - phaseout, 0))])
            }
            _ => 0,
        }
    }
    pub(crate) fn initial_current(&self) -> i64 {
        match *self {
            Self::Costs { started, .. } if started / 10000 == 2026 => self.initial(),
            _ => 0,
        }
    }
    pub(crate) fn amortization(&self) -> i64 {
        match *self {
            Self::Costs { costs, started, .. } if self.deducted() => {
                MonthlySchedule::original_180(
                    AmortizationRegime::Section195,
                    Month { year: started / 10000, month: started / 100 % 100 },
                    2026,
                    costs - self.initial(),
                    self.initial_current(),
                )
                .current_allowable_cents
            }
            _ => 0,
        }
    }
    pub(crate) fn deduction(&self) -> i64 {
        self.initial_current() + self.amortization()
    }
    // The month the 180-month amortization begins, when costs are being
    // amortized (Form 4562-B line 1 column (b); an earlier start is line 2).
    pub(crate) fn amortization_begins(&self) -> Option<u32> {
        match *self {
            Self::Costs { started, .. } if self.deducted() => Some(started),
            _ => None,
        }
    }
    pub(crate) fn remaining(&self) -> i64 {
        match *self {
            Self::None => 0,
            Self::Costs { costs, .. } if !self.deducted() => costs,
            Self::Costs { costs, started, .. } => {
                MonthlySchedule::original_180(
                    AmortizationRegime::Section195,
                    Month { year: started / 10000, month: started / 100 % 100 },
                    2026,
                    costs - self.initial(),
                    self.initial_current(),
                )
                .statutory_remaining_cents
            }
        }
    }
}
