use super::form_1040_es_model::{NextYearEstimate, NextYearEstimateRecord, NextYearInitial};
use super::form_2210_model::PenaltyChoice;
use super::form_4136_model::FederalHomeKerosene;
use crate::injured_spouse_payments::{EstimatedPaymentEntry, EstimatedPaymentEntryRecord};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// The refund choices, asked on the review as the return is finished: the
// stored record is FederalPayments' planned credit and its account.
/// Your refund
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RefundChoice {
    /// Amount of this return's overpayment to apply to 2027 estimated tax (Form 1040 line 36)
    /// Enter 0 to request no applied credit. The remaining available overpayment is your cash refund. This is a plan, not a payment or a filed election; once filed, the IRS election cannot be changed. The final 2025 Form 1040 instructions supply the settlement rule until final 2026 instructions are available.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_applied_next_year: i64,
    /// Account to receive the planned credit
    /// This is account intent, not ownership of a joint payment. A spouse-account election requires a separate filing statement with the spouse's SSN; this draft does not create it.
    /// [NullText]
    /// No account intent selected
    /// [SetText]
    /// Apply the credit to
    #[libertas_default($null)]
    pub credit_recipient: Option<OverpaymentRecipient>,
    /// Available overpayment after the calculated penalty
    /// When the IRS will figure the penalty, this is the blank-line-38 instruction amount, not a determination of zero penalty. Finish the return to establish an unavailable amount.
    /// [NullText]
    /// Not available from the current return
    /// [SetText]
    /// Available before the planned credit
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub credit_capacity: Option<i64>,
}

// The payments editor maps dated payments, the extension payment and Form 2210
// facts onto FederalPayments. Withholding is derived from Form 1040 lines 25a-25d.
/// Federal payments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Payments {
    /// Federal income tax withheld
    /// [NullText]
    /// Not available yet
    /// [SetText]
    /// From the actual withholding sources
    #[libertas_read_only]
    pub withheld: Option<Withheld>,
    /// Estimated tax payments for 2026
    /// Include only actual payments and prior-year credit applied. Changing methods replaces the source details.
    /// On a separate return, include only payments you paid separately and your part of joint payments. Use the spouses' agreed allocation; without agreement, use the separate-tax ratio described in Pub. 555. The App does not halve these entries automatically.
    pub estimated: EstimatedPaymentEntry,
    /// Paid with an extension request
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub extension: i64,
    /// Penalty, next-year estimate and payment source details
    pub more: PaymentsMore,
}

/// Penalty, next-year estimate and payment source details
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaymentsMore {
    /// Underpayment penalty
    #[libertas_default(IrsFigures)]
    pub penalty: PenaltyChoice,
    /// Your 2027 estimated tax (Form 1040-ES)
    #[libertas_default(NotNeeded)]
    pub next_year: NextYearEstimate,
    /// Payer of the extension payment
    /// [NullText]
    /// No extension payment was made
    /// [SetText]
    /// The extension payment was made by
    #[libertas_default($null)]
    pub extension_payer: Option<crate::PaymentPayer>,
    /// Source property of the extension payment
    /// [NullText]
    /// No separate community-property source detail is supplied for the extension payment
    /// [SetText]
    /// The extension payment came from
    #[libertas_default($null)]
    pub extension_refund_property: Option<crate::RefundSourceProperty>,
}

// Reduced current-editor calculation input: the derived withholding is
// absent.
/// Payment answers used for the running estimate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct PaymentsEstimateInput {
    pub estimated: EstimatedPaymentEntry,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub extension: i64,
    pub more: PaymentsMore,
}
impl From<PaymentsEstimateInput> for Payments {
    fn from(value: PaymentsEstimateInput) -> Self {
        Self {
            estimated: value.estimated,
            extension: value.extension,
            more: value.more,
            withheld: None,
        }
    }
}
#[cfg(test)]
impl From<Payments> for PaymentsEstimateInput {
    fn from(value: Payments) -> Self {
        Self { estimated: value.estimated, extension: value.extension, more: value.more }
    }
}

/// Account intended to receive the applied overpayment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum OverpaymentRecipient {
    /// Taxpayer's account
    TaxpayerAccount,
    /// Spouse's account (requires a separate statement when filing)
    SpouseAccount,
}
pub use crate::return_carriers::Withheld;

/// Federal payments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPayments {
    /// Planned amount of this return's overpayment applied to next year's estimated tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_applied_next_year: i64,
    /// Planned account recipient, not payment ownership
    pub credit_recipient: Option<OverpaymentRecipient>,
    /// Actual estimated payment sources and timing method
    pub estimated: EstimatedPaymentEntryRecord,
    /// Payment already made with your filing extension
    /// Enter the amount actually paid to the IRS with an extension for your 2026 federal return.
    /// Enter 0 if none. This is not the amount you expect to owe or a recommended payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub extension: i64,
    /// Federal fuel tax paid on undyed kerosene used at home
    /// Qualifying home use is the Form 4136 business-information exemption. The credit is a Schedule 3 line 12 payment, not a business deduction or taxable income recovery.
    pub home_kerosene: Option<FederalHomeKerosene>,
    /// Your 2025 tax, for the underpayment penalty
    /// Leave empty to let the IRS figure any penalty and bill you.
    pub prior_year: Option<crate::PriorYearTax>,
    /// Figure the penalty with withholding on the dates it was actually withheld
    /// Otherwise withholding counts as paid in four equal parts on the payment due dates.
    pub withholding_by_date: bool,
    /// Withholding by payment period
    pub withholding_periods: Option<crate::PeriodWithholding>,
    /// Figure the penalty with the annualized income installment method
    /// Choose this when your income arrived unevenly during the year, for example from a seasonal business or a large gain late in the year. It applies to every payment period.
    pub annualized: bool,
    /// Income and deductions by period
    pub annualized_income: Option<crate::AnnualizedIncome>,
    /// At least two-thirds of your 2025 gross income was from farming or fishing
    /// Compare the farming and fishing gross income on your 2025 Form 1040 schedules with its total gross income. The App figures 2026 from this return and ignores this answer when 2026 qualifies.
    pub farmer_fisherman_prior_year: bool,
    /// You filed your 2026 return and paid the entire tax shown on it by March 1, 2027
    /// Only used when two-thirds of your 2025 or 2026 gross income was from farming or fishing; when Yes, it removes the underpayment penalty entirely.
    pub farmer_paid_by_march: bool,
    /// Request a penalty waiver
    /// This does not change the estimated penalty; it adds a note reminding you to attach your explanation and any documentation when you file.
    pub waiver: Option<crate::PenaltyWaiver>,
    /// Your 2027 estimated tax (Form 1040-ES)
    pub next_year: NextYearEstimateRecord,
    /// Entered for a joint return
    /// Payments and 2025 facts are the household's; a change into or out of a joint return asks them again.
    pub joint: bool,
    /// Payer of the extension payment
    #[libertas_default($null)]
    pub extension_payer: Option<crate::PaymentPayer>,
    /// Source property of the extension payment
    #[libertas_default($null)]
    pub extension_refund_property: Option<crate::RefundSourcePropertyFacts>,
}

impl FederalPayments {
    // No payments, no dated payments and no 2025 facts: a new return.
    pub(crate) fn none() -> Self {
        Self {
            credit_applied_next_year: 0,
            credit_recipient: None,
            estimated: EstimatedPaymentEntryRecord::None,
            extension: 0,
            extension_payer: None,
            extension_refund_property: None,
            home_kerosene: None,
            prior_year: None,
            withholding_by_date: false,
            withholding_periods: None,
            annualized: false,
            annualized_income: None,
            farmer_fisherman_prior_year: false,
            farmer_paid_by_march: false,
            waiver: None,
            next_year: NextYearEstimateRecord::NotNeeded,
            joint: false,
        }
    }
    // Nothing entered on the payments page, for either household.
    pub(crate) fn nothing_entered(&self) -> bool {
        *self
            == Self { joint: self.joint, home_kerosene: self.home_kerosene.clone(), ..Self::none() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct PaymentsInitial {
    pub withheld: Option<Withheld>,
    pub estimated: Option<crate::EstimatedPaymentEntry>,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub extension: i64,
    pub more: PaymentsMoreInitial,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct PaymentsMoreInitial {
    pub penalty: PenaltyChoice,
    pub next_year: NextYearInitial,
    pub extension_payer: Option<crate::PaymentPayer>,
    pub extension_refund_property: Option<crate::RefundSourceProperty>,
}
