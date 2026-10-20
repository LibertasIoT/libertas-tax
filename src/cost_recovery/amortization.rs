use crate::kernel::cost_recovery::{AmortizationRegime, Month, MonthlySchedule};
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::lines::{form_count, line_cents, push_line};
use crate::{FederalResult, FormLine, TaxForm};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Later capitalized acquisition cost
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IntangibleCost {
    /// Additional allocated business acquisition cost
    /// Use purchase/allocation records. Exclude interest, original issue discount, amounts already included in original cost, deductible royalties, renewal costs and transfer or disposition adjustments. A renewal is a separate intangible. Include only amounts properly added to basis under your accounting method.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
    /// Date this cost became includible in basis
    /// Use the date supported by the acquisition obligation and accounting records, not automatically the cash-payment date. Unresolved contingent-debt interest allocation or timing needs separate review.
    #[libertas_date_only]
    pub date: u32,
}

/// Section 197 business intangible
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessIntangible {
    /// Asset or right acquired
    /// Examples include acquired goodwill, going-concern value, workforce, business records, customer/supplier relationships, a qualifying government license or a franchise. Eligibility depends on the acquisition and statutory exclusions, not the label alone.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Original capitalized business cost allocated to this intangible
    /// Use the documented purchase-price allocation and original capitalized acquisition costs. Enter zero only when all cost became includible later and is entered below. Exclude personal basis. Do not deduct this cost again as an ordinary expense, start-up cost or equipment depreciation. Enter each eligible renewal separately.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Acquisition, qualifying creation or renewal date
    #[libertas_date_only]
    pub acquired: u32,
    /// Later capitalized acquisition costs
    /// Leave empty if none. Include all later costs through 2026 exactly once, including earlier-year additions. Each addition uses the original schedule's remaining months; an addition after its end is deductible immediately. These additions do not start another 15-year period.
    pub additions: Vec<IntangibleCost>,
    /// Date this trade or business began
    /// Amortization starts in the later of the acquisition month and the month this business began. An existing business may begin before it acquires this asset.
    #[libertas_date_only]
    pub business_started: u32,
    /// Ordinary eligible section 197 asset and complete records
    /// Confirm acquisition after August 10, 1993, continued business ownership/use through 2026, and correct prior 180-month deductions when applicable. Exclude anti-churning/related-party restrictions, carryover-basis transfers, basis decreases, transfer/disposition basis adjustments, unresolved contingent-payment timing/interest allocation, special leases, dispositions, missed prior deductions and changed use. Self-created goodwill and most other self-created intangibles do not qualify; eligible government rights, franchises/trademarks/trade names and creation in a qualifying business acquisition follow their statutory exceptions. Standalone software, patents/copyrights, interests in entities, land, loans and other excluded rights need their own treatment. All original and added acquisition costs and their inclusion dates are supported by records and entered once. No leaves this amortization out of this estimate.
    /// Check the acquisition agreement, basis records and earlier amortization returns.
    pub eligible: bool,
}
impl BusinessIntangible {
    pub(crate) fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && (0..=100000000000).contains(&self.cost)
            && (self.cost > 0 || !self.additions.is_empty())
            && self.additions.len() <= 100
            && self.additions.iter().all(|a| {
                (1..=100000000000).contains(&a.amount)
                    && crate::federal_rules::valid_birth(a.date)
                    && a.date >= self.acquired
            })
            && crate::federal_rules::valid_birth(self.acquired)
            && crate::federal_rules::valid_birth(self.business_started)
    }
    pub(crate) fn started(&self) -> u32 {
        self.acquired.max(self.business_started)
    }
    pub(crate) fn deduction(&self) -> i64 {
        let started = self.started();
        let month = Month { year: started / 10000, month: started / 100 % 100 };
        MonthlySchedule::original_180(AmortizationRegime::Section197, month, 2026, self.cost, 0)
            .current_allowable_cents
            + self
                .additions
                .iter()
                .map(|a| {
                    let date = a.date.max(started);
                    MonthlySchedule::section197_addition_180(
                        month,
                        Month { year: date / 10000, month: date / 100 % 100 },
                        2026,
                        a.amount,
                    )
                    .current_allowable_cents
                })
                .sum::<i64>()
    }
    pub(crate) fn remaining(&self) -> i64 {
        let started = self.started();
        let month = Month { year: started / 10000, month: started / 100 % 100 };
        MonthlySchedule::original_180(AmortizationRegime::Section197, month, 2026, self.cost, 0)
            .statutory_remaining_cents
            + self
                .additions
                .iter()
                .map(|a| {
                    let date = a.date.max(started);
                    MonthlySchedule::section197_addition_180(
                        month,
                        Month { year: date / 10000, month: date / 100 % 100 },
                        2026,
                        a.amount,
                    )
                    .statutory_remaining_cents
                })
                .sum::<i64>()
    }
    pub(crate) fn report(
        &self,
        r: &mut FederalResult,
        id: i64,
        index: usize,
        #[cfg(test)] source: &str,
    ) {
        crate::require_localized!(
            r,
            self.acquired >= 19930811
                && self.started() <= 20261231
                && self.additions.iter().all(|a| a.date <= 20261231),
            &crate::IssueContext::income_id(id),
            source,
            "ISSUE_CONFIRM_ELIGIBLE_SECTION_197_ACQUISITION_AFTER_AUGUST_10_1993_ACTIV_2860391A",
            "Confirm eligible section 197 acquisition after August 10, 1993, active business by 2026 and continued ordinary ownership/use with correct prior deductions. All later acquisition costs must be includible in basis by 2026 with documented allocation and timing. Excluded intangibles, transfers, basis decreases or dispositions need separate rules.",
        );
        // Amortization that begins in 2026 is Form 4562-B line 1; the earlier
        // starts are only the line 2 total (report_form_4562b).
        let begins_current_year = self.started() / 10000 == 2026;
        let tag = |line: &str| begins_current_year.then(|| FormLine::new(TaxForm::Form4562B, line));
        let inputs = ["income.business.intangibles"];
        let source = "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol4/pdf/CFR-2025-title26-vol4-sec1-197-2.pdf";
        if begins_current_year {
            push_line(
                r,
                tag("1(c)"),
                &format!("business_{id}_intangible_{index}_amortizable"),
                self.cost + self.additions.iter().map(|a| a.amount).sum::<i64>(),
                &inputs,
                source,
                crate::Message::new(
                    "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZABLE",
                    alloc::vec![
                        crate::MessageArg::Text(self.label.clone()),
                        crate::MessageArg::DateOnly(i64::from(self.started())),
                    ],
                ),
            );
            form_count(
                r,
                FormLine::new(TaxForm::Form4562B, "1(e)"),
                &format!("business_{id}_intangible_{index}_period"),
                180,
                &inputs,
                source,
                msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_PERIOD"),
            );
        }
        push_line(
            r,
            tag("1(f)"),
            &format!("business_{id}_intangible_{index}_amortization"),
            self.deduction(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZATION"),
        );
        line_cents(
            r,
            &format!("business_{id}_intangible_{index}_remaining_basis"),
            self.remaining(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_REMAINING_BASIS"),
        );
    }
}
// Form 4562-B lines 2 and 3 of one business: line 2 is the 2026 amortization of
// costs that began in an earlier year, line 3 adds line 1 column (f). Printed
// whenever the business amortizes anything.
pub(crate) fn report_form_4562b(
    r: &mut FederalResult,
    id: i64,
    startup: &crate::BusinessStartup,
    intangibles: &[BusinessIntangible],
    timber: &[crate::BusinessTimberCost],
    other: &[crate::OtherAmortizableCost],
    assets: &[crate::BusinessAsset],
) {
    if startup.amortization_begins().is_none()
        && intangibles.is_empty()
        && timber.is_empty()
        && !crate::cohorts::cost_cohorts(other, 2026, assets)
            .parts
            .iter()
            .any(|part| part.as_ref().is_ok_and(|part| part.months > 0 || part.amortization != 0))
    {
        return;
    }
    let earlier_start = |started: u32| started / 10000 < 2026;
    let earlier = startup
        .amortization_begins()
        .filter(|s| earlier_start(*s))
        .map_or(0, |_| startup.amortization())
        + intangibles
            .iter()
            .filter(|i| earlier_start(i.started()))
            .map(BusinessIntangible::deduction)
            .sum::<i64>();
    let timber = crate::timber::totals(timber).expect("validated timber form totals");
    let other = crate::cohorts::totals(other, assets).expect("validated cost cohort totals");
    let earlier = earlier.checked_add(other[1]).expect("validated amortization total");
    let current_other = other[2];
    let earlier = earlier.checked_add(timber[1]).expect("validated amortization total");
    let current = startup
        .amortization_begins()
        .filter(|s| !earlier_start(*s))
        .map_or(0, |_| startup.amortization())
        + intangibles
            .iter()
            .filter(|i| !earlier_start(i.started()))
            .map(BusinessIntangible::deduction)
            .sum::<i64>();
    let current = current.checked_add(current_other).expect("validated amortization total");
    let current = current.checked_add(timber[2]).expect("validated amortization total");
    let total = earlier.checked_add(current).expect("validated amortization total");
    emit_lines(
        r,
        &mut (),
        [
            ("2", "earlier", earlier, msg!("EXPLAIN_BUSINESS_N_FORM_4562B_EARLIER")),
            ("3", "total", total, msg!("EXPLAIN_BUSINESS_N_FORM_4562B_TOTAL")),
        ],
        |(line, suffix, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4562B, line)),
                &format!("business_{id}_form_4562b_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.business.startup", "income.business.intangibles"],
                "https://www.irs.gov/instructions/i4562",
                explanation,
            ))
        },
    );
}
