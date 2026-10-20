use crate::kernel::cost_recovery::{AmortizationRegime, Month, MonthlySchedule};
use crate::kernel::lines::{form_count, line_cents, push_line};
use crate::kernel::phase;
use crate::{FederalResult, FormLine, TaxForm};
use alloc::format;
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
        #[libertas_number(min = 0)]
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
    fn deducted(&self) -> bool {
        matches!(self, Self::Costs { capitalize: false, eligible: true, .. })
    }
    fn initial(&self) -> i64 {
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
    pub(crate) fn report(&self, r: &mut FederalResult, id: i64, #[cfg(test)] source: &str) {
        let Self::Costs { started, .. } = *self else {
            return;
        };

        crate::require_localized!(
            r,
            (20041023..=20261231).contains(&started),
            &crate::IssueContext::income_id(id),
            source,
            "ISSUE_CONFIRM_COMPLETE_SECTION_195_START_UP_COSTS_THE_ORIGINAL_DEDUCTION_D6F699EB",
            "Confirm complete section 195 start-up costs, the original deduction/capitalization election and an active business begun after October 22, 2004 and continuing through 2026. Disposition, missing prior deductions or other special treatment needs its own calculation.",
        );
        // Amortization that begins in 2026 is Form 4562-B line 1; the earlier
        // starts are only the line 2 total (amortization::report_form_4562b).
        let begins_current_year = self.amortization_begins().is_some_and(|s| s / 10000 == 2026);
        let tag = |line: &str| begins_current_year.then(|| FormLine::new(TaxForm::Form4562B, line));
        let amortizable = match *self {
            Self::Costs { costs, .. } if self.deducted() => costs - self.initial(),
            _ => 0,
        };
        let inputs = ["income.business.startup"];
        let source = "https://www.irs.gov/instructions/i4562";
        push_line(
            r,
            None,
            &format!("business_{id}_startup_initial"),
            self.initial_current(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_INITIAL"),
        );
        if begins_current_year {
            push_line(
                r,
                tag("1(c)"),
                &format!("business_{id}_startup_amortizable"),
                amortizable,
                &inputs,
                source,
                crate::Message::new(
                    "EXPLAIN_BUSINESS_N_STARTUP_AMORTIZABLE",
                    alloc::vec![crate::MessageArg::DateOnly(i64::from(started))],
                ),
            );
            form_count(
                r,
                FormLine::new(TaxForm::Form4562B, "1(e)"),
                &format!("business_{id}_startup_period"),
                180,
                &inputs,
                source,
                msg!("EXPLAIN_BUSINESS_N_STARTUP_PERIOD"),
            );
        }
        push_line(
            r,
            tag("1(f)"),
            &format!("business_{id}_startup_amortization"),
            self.amortization(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_AMORTIZATION"),
        );
        line_cents(
            r,
            &format!("business_{id}_startup_remaining_basis"),
            self.remaining(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_REMAINING_BASIS"),
        );
    }
}
