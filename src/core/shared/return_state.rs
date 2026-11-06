use crate::common::{CalculationLine, ResultState};
use crate::diagnostics::{FederalIssue, ReviewNote};
use crate::form_lines::FormLine;
use crate::return_carriers::{
    AllocationPlan, ExpenseRoleDollars, FarmTaxInstallments, HomeUseContext, NativeElectiveSources,
    NativeWithholdingSources, Withheld,
};
use alloc::{boxed::Box, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A review note shown beside the estimate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalReviewNoteView {
    /// Explanation
    #[libertas_read_only]
    #[libertas_formatted_text]
    pub message: Vec<u8>,
}

/// 2026 federal estimate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEstimate {
    /// Coverage status
    pub state: ResultState,
    /// Accepted revision
    pub revision: i64,
    /// Estimate notes
    /// These notes explain assumptions, items left out, and limitations affecting this estimate. Review them before relying on the result.
    /// ----
    /// Review note
    pub review_notes: Vec<FederalReviewNoteView>,
    /// Estimate method and limitations
    #[libertas_read_only]
    #[libertas_formatted_text]
    pub method: Vec<u8>,
    /// Calculated draft return
    /// Provisional 2026 Form 1040 line references, then on the review each other form's lines with an amount, using the same calculation as this estimate. This is a review view, not a completed IRS form. The proposed Schedule 3-A reduction is not applied; read Estimate notes.
    #[libertas_suppress_label]
    /// ----
    /// Return line
    pub return_lines: Vec<FederalAmount>,
    /// Calculation breakdown
    #[libertas_suppress_label]
    #[libertas_foldable(initially_open = false)]
    /// ----
    /// Lines entry
    pub breakdown: Vec<FederalAmount>,
    /// Total estimated federal tax
    #[libertas_money("USD", 2)]
    pub tax: Option<i64>,
    /// Optional farmland tax-payment schedule
    /// The full tax above remains the liability. This schedule separates current payment from the remaining deferred balance.
    pub farmland_installments: Option<crate::FarmTaxInstallments>,
    /// Refundable credits
    #[libertas_money("USD", 2)]
    pub refundable_credits: Option<i64>,
    /// Withholding and payments
    #[libertas_money("USD", 2)]
    pub payments: Option<i64>,
    /// Estimated refund before interest and offsets
    /// Less the estimated tax penalty when your 2025 tax is entered and the reported planned credit on line 36. Calculated without the proposed Schedule 3-A reduction. Check Estimate notes; this is not a filing-ready refund.
    #[libertas_money("USD", 2)]
    pub refund: Option<i64>,
    /// Estimated balance before interest
    /// Includes the estimated tax penalty when your 2025 tax is entered.
    #[libertas_money("USD", 2)]
    pub balance: Option<i64>,
}

// Certified independent payroll results from one completed native pass.
// Transient cleanup policy only: never stored or exported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PayrollKnownRows {
    pub(crate) agi: bool,
    pub(crate) social: [bool; 2],
    pub(crate) coverage_parts: [[bool; 2]; 2],
    pub(crate) form8919: [bool; 2],
    pub(crate) medicare_wages: bool,
    pub(crate) railroad_compensation: bool,
    pub(crate) railroad_withholding: bool,
    pub(crate) additional_parts: [bool; 3],
    pub(crate) se_net: bool,
}

// The calculation's full result: what the review shows (FederalEstimate)
// and the calculation provenance (lines) the App reads for its own rules,
// tests and diagnostics. Provenance never travels in a reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FederalResult {
    pub(crate) rental_home_context: Option<HomeUseContext>,
    pub(crate) payroll_known_rows: Option<PayrollKnownRows>,
    // The sole native payment-category projection, independent of whether
    // unrelated tax sections permit a complete return. Never persisted.
    pub(crate) payment_withholding: Option<Withheld>,
    // Finalized settlement capacity, before the planned credit; never persisted.
    pub(crate) credit_capacity: Option<i64>,
    // Internal choice before a caller fixes the route; home-use Worksheet 5-1
    // uses this to detect a deduction-method cycle without exposing state.
    pub(crate) automatic_itemizes: bool,
    // Actual outer omission scope and final native financial availability.
    // Transient metadata only; informational review notes do not change it.
    pub(crate) financial_source_scope_complete: bool,
    // Same-invocation native cash footprint; never persisted or exported.
    pub(crate) withholding_sources: Option<Box<NativeWithholdingSources>>,
    pub(crate) expense_roles: Option<Box<ExpenseRoleDollars>>,
    // Same-invocation Form3800 source ownership; never stored or exported.
    pub(crate) elective_sources: Option<Box<NativeElectiveSources>>,
    // Each field as FederalEstimate documents it, plus the provenance.
    pub state: ResultState,
    pub revision: i64,
    pub issues: Vec<FederalIssue>,
    pub review_notes: Vec<ReviewNote>,
    pub lines: Vec<CalculationLine>,
    pub return_lines: Vec<FederalAmount>,
    pub tax: Option<i64>,
    pub farmland_installments: Option<FarmTaxInstallments>,
    pub refundable_credits: Option<i64>,
    pub payments: Option<i64>,
    pub refund: Option<i64>,
    pub balance: Option<i64>,
    // Test footing of Form 1116 line 3e's Schedule D part, in cents: the gains before losses
    // the totals hold, and the net plus the losses netted less the gains kept out on purpose.
    #[cfg(test)]
    pub(crate) capital_footing: Option<[i64; 2]>,
    pub(crate) injured_spouse: Option<AllocationPlan>,
}

impl FederalResult {
    pub(crate) fn empty(revision: i64) -> Self {
        Self {
            rental_home_context: None,
            payroll_known_rows: None,
            payment_withholding: None,
            credit_capacity: None,
            automatic_itemizes: false,
            financial_source_scope_complete: false,
            withholding_sources: None,
            expense_roles: None,
            elective_sources: None,
            state: ResultState::Incomplete,
            revision,
            issues: Vec::new(),
            review_notes: Vec::new(),
            lines: Vec::new(),
            return_lines: Vec::new(),
            tax: None,
            farmland_installments: None,
            refundable_credits: None,
            payments: None,
            refund: None,
            balance: None,
            #[cfg(test)]
            capital_footing: None,
            injured_spouse: None,
        }
    }
}

/// Calculation amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_foldable(initially_open = false)]
pub struct FederalAmount {
    /// Calculation
    #[libertas_hidden]
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Amount
    /// Zero for a count, whose number is in the label; the shown Amount is the next field.
    #[libertas_hidden]
    #[libertas_money("USD", 2)]
    pub amount: i64,
    /// How this amount was calculated
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub explanation: Vec<u8>,
    /// Form line
    #[libertas_hidden]
    pub form_line: Option<FormLine>,
    /// Amount
    /// Empty for a line that counts people instead of dollars.
    #[libertas_money("USD", 2)]
    pub money: Option<i64>,
}
