// Coverage: Employee stock sales (B7a, 2026-10-09). EmployeeStockSale (a nonstatutory option, vested
//   stock award or section 83(b) restricted stock sold in 2026: compensation = value when taxed less
//   the amount paid, basis = paid + compensation + commissions, long-term when sold more than one
//   year after the taxing date; Pub. 525 Example 12) gains its page, EmployeeShareSale, on Investment
//   sales. The lot is reported on Form 8949 with code B (the broker's box 1e omits compensation of
//   options and awards granted after 2013; Instructions for Form 1099-B, Instructions for Form 8949
//   code B) in box A/D, so it totals on Schedule D line 1b (short) or 8b (long), not the summary lines
//   1a/8a. A sale whose page says compensation is not in a W-2, or another special treatment applies,
//   is left out with WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT. Forms: 8949, Schedule D (lines 1b, 8b). The
//   page imports the Form 1099-B boxes 1a, 1c, 1d and 4; the taxing date, value and amount paid are
//   from the stock plan records and W-2. Tests: tests/investment/investment_kinds.rs employee_share_sale_*.
use crate::kernel::phase;
use crate::{FederalSale, GainTerm, SaleAdjustment, SaleUse};
use alloc::string::String;
use libertas_macros::*;

/// Employee stock sale after compensation recognition
/// One sold lot from a nonstatutory option exercise, vested stock award or restricted stock covered by a valid section 83(b) election. Use transaction totals for the sold shares, including fractional shares. The App calculates compensation and basis; do not repeat this sale in another entry. ISO and employee stock purchase plan dispositions use different rules.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeStockSale {
    /// Date shares became taxable as compensation
    /// Exercise date for an ordinary fully vested nonstatutory option; delivery date for an unrestricted vested award; vesting date for restricted stock without an election. With a valid section 83(b) election, use the original stock-transfer date, not the later vesting date. Deferred settlement requires separate treatment.
    #[libertas_date_only]
    pub acquired: u32,
    /// Sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Amount paid for these shares, excluding acquisition fees
    /// Include the exercise cost and any option purchase price allocable to this lot. Enter zero for an award received without payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Fair market value of these shares when compensation was recognized
    /// Total value of this sold lot from the compensation-date record, not today's value or a computed taxable gain. For section 83(b), use transfer-date fair market value determined without lapse restrictions; later vesting does not reset basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_value: i64,
    /// Acquisition commissions allocated to this lot
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub fees: i64,
    /// Net sale proceeds after selling fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Disallowed wash-sale loss from replacement-share records
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wash_sale: i64,
    /// Federal income tax withheld on the sale, not already entered elsewhere
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Basis the Form 1099-B printed in box 1e, when it shows one
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// Whether the broker reported the basis to the IRS (box 12)
    pub reporting: crate::BasisReporting,
    /// Compensation not in a W-2, a section 83(i) or deferred settlement, a forfeiture, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}
impl EmployeeStockSale {
    pub(crate) fn compensation(&self) -> i64 {
        phase::floor(self.market_value - self.paid, 0)
    }
    pub(crate) fn sale(&self) -> FederalSale {
        FederalSale {
            term: if crate::dates::held_more_than_one_year(self.acquired, self.sold) {
                GainTerm::Long
            } else {
                GainTerm::Short
            },
            collectible: false,
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            proceeds: self.proceeds,
            basis: Some(self.paid + self.compensation() + self.fees),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: self.withholding,
            use_kind: SaleUse::Investment { wash_sale: self.wash_sale },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: sale_adjustment(
                self.reporting,
                self.reported_basis,
                self.paid + self.fees,
                self.paid + self.compensation() + self.fees,
            ),
            basis_reporting: self.reporting,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let max = 100000000000;
        crate::federal_rules::valid_birth(self.acquired)
            && crate::federal_rules::valid_birth(self.sold)
            && self.acquired <= self.sold
            && self.sold / 10000 == 2026
            && [
                self.paid,
                self.market_value,
                self.fees,
                self.proceeds,
                self.wash_sale,
                self.withholding,
            ]
            .iter()
            .all(|v| (0..=max).contains(v))
            && self.sale().basis_amount() <= max
            && self.wash_sale <= (self.sale().basis_amount() - self.proceeds).max(0)
    }
}

// Form 8949 code B for a stock-compensation sale: the broker's box 1e omits the compensation of an
// option or award granted after 2013 (Instructions for Form 1099-B; Instructions for Form 8949, code
// B), so a lot whose basis the broker reported to the IRS (box 12, box A/D, Schedule D line 1b/8b)
// shows the basis the broker printed and a column (g) correction to the actual basis. The printed
// box 1e (F3, 2026-10-09) is that figure; without one the broker's figure is the model's `modeled`
// (the cost without the compensation). No difference, no correction; a basis not reported to the
// IRS (box B/E) or no Form 1099-B (box C/F) needs no code B.
pub(crate) fn sale_adjustment(
    reporting: crate::BasisReporting,
    printed: Option<i64>,
    modeled: i64,
    basis: i64,
) -> SaleAdjustment {
    let reported = printed.unwrap_or(modeled);
    if reporting == crate::BasisReporting::ReportedToIrs && reported != basis {
        SaleAdjustment::BasisCorrection { reported_basis: reported }
    } else {
        SaleAdjustment::None
    }
}

// Shared exact monetary boundary for fractional stock quantities.
pub(crate) fn share_units(shares: f64) -> Option<i64> {
    if !shares.is_finite() || !(0.000001..=1000000.0).contains(&shares) {
        return None;
    }
    let scaled = shares * 1000000.0;
    let units = (scaled + 0.5) as i64;
    // Accept binary representation noise, not additional decimal precision.
    if (scaled - units as f64).abs() > scaled.abs() * f64::EPSILON * 4.0 {
        return None;
    }
    Some(units)
}

pub(crate) fn lot_amount(shares: f64, per_share: i64) -> i64 {
    crate::kernel::ratio::fraction(per_share, share_units(shares).unwrap_or(0), 1000000)
}

/// Employee stock shares sold
/// Shares from a nonqualified stock option, restricted stock units or another stock award from your employer, sold in 2026. Enter the sale here with its 1099-B, not again as a 1099-B sale; the App adds the compensation already taxed as wages to the basis.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 1099-B sale of these employee stock shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Return box 1e as reported_basis exactly as printed, even when a supplemental statement shows an adjusted basis, or leave it empty when the box is blank, and return box 12 as reporting. Read taxed, value and paid from the employer's stock plan record or supplemental statement supplied with the 1099-B for these shares: the date the shares vested or were otherwise taxed as wages, the value of the sold shares taxed as wages on that date, and the amount the employee paid for them, zero when the record shows nothing was paid. When no such record is supplied, report an error asking for the stock plan record that shows the date and value taxed as wages. Do not infer commissions, a wash sale, special treatment or other facts absent from these pages. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the printed calendar year as an integer such as 2026, or zero when blank. Read the account number exactly as printed, or empty text when blank. Read box 1b date acquired as the printed date, VARIOUS, INHERITED or another notation, or null when blank. Read the Form 8949 box letter printed on the statement, or null when none is printed, and box 2 short-term, long-term and ordinary as printed checkboxes (true only when checked).",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.description,@.sold,@.taxed,@.proceeds,@.reported_basis,@.withholding,@.value,@.paid",
    form_date_order = "MonthDayYear"
)]
pub struct EmployeeShareSale {
    /// Calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// 1099-B box 1a description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub description: String,
    /// 1b — Date acquired
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date or notation is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub box_1b_acquired: Option<crate::BrokerAcquisitionDate>,
    /// 1099-B box 1c date sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// 1099-B box 1d proceeds
    /// Net of selling fees, as the 1099-B reports them.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// 1099-B box 1e cost or other basis
    /// As printed, before any correction: the broker's figure leaves out the compensation of an option or award granted after 2013, which the App adds with Form 8949 code B. Leave empty when the box is blank.
    /// [NullText]
    /// No cost basis is printed in box 1e
    /// [SetText]
    /// The cost basis printed in box 1e is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// 1099-B box 12 basis reported to the IRS
    /// Box A or D when the broker reported the basis to the IRS, B or E when it did not, C or F when no Form 1099-B was received.
    #[libertas_llm_doc_field]
    #[libertas_default(ReportedToIrs)]
    pub reporting: crate::BasisReporting,
    /// Applicable checkbox on Form 8949
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed code is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub form_8949_checkbox: Option<crate::Form1099B8949Checkbox>,
    /// 2 — Short-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_short_term: bool,
    /// 2 — Long-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_long_term: bool,
    /// 2 — Ordinary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_ordinary: bool,
    /// 1099-B box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Date the shares were taxed as wages
    /// The exercise date of a nonqualified option, or the date restricted stock or units vested. With a section 83(b) election, the date you received the shares. Your employer's stock plan statement shows it.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub taxed: u32,
    /// Value of the shares sold on that date
    /// For only the shares sold: the market value your employer used for your W-2 wages (box 12 code V for an option), not today's value.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Amount you paid for the shares sold
    /// The option's exercise price for these shares; zero for restricted stock units or an award you did not pay for.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub paid: i64,
    /// About this sale
    pub more: EmployeeShareSaleMore,
}

/// About this sale
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct EmployeeShareSaleMore {
    /// Commissions paid when you exercised, for these shares
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fees: i64,
    /// Wash sale loss disallowed
    /// When you bought the same stock within 30 days before or after selling these shares at a loss, including shares that vested in that window. Box 1g shows the broker's figure for this account.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wash_sale: i64,
    /// Compensation not in a W-2, a section 83(i) or deferred settlement, a forfeiture, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl EmployeeShareSale {
    // The first relationship between this page's fields that fails, by its
    // rule's key, in the order of the client rules. The wash sale bound is
    // the stored sale's: the larger of the value and the amount paid, plus
    // commissions, less proceeds.

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.taxed > self.sold {
            return Some(
                crate::validation::ValidationIssue::new("EMPLOYEE_STOCK_SOLD_BEFORE_TAXED")
                    .at_field("sold"),
            );
        }
        if !(20260101..=20261231).contains(&self.sold) {
            return Some(
                crate::validation::ValidationIssue::new("EMPLOYEE_STOCK_SALE_NOT_IN_RETURN_YEAR")
                    .at_field("sold"),
            );
        }
        let more = &self.more;
        if more.wash_sale != 0
            && i128::from(more.wash_sale)
                > i128::from(self.value.max(self.paid)) + i128::from(more.fees)
                    - i128::from(self.proceeds)
        {
            return Some(
                crate::validation::ValidationIssue::new("SALE_WASH_SALE_ABOVE_LOSS")
                    .at_field("wash_sale")
                    .in_field("more"),
            );
        }
        None
    }
    // The stored sale this page writes.
    pub(crate) fn record(&self) -> EmployeeStockSale {
        EmployeeStockSale {
            acquired: self.taxed,
            sold: self.sold,
            paid: self.paid,
            market_value: self.value,
            fees: self.more.fees,
            proceeds: self.proceeds,
            wash_sale: self.more.wash_sale,
            withholding: self.withholding,
            reported_basis: self.reported_basis,
            reporting: self.reporting,
            special: self.more.special,
        }
    }
    // The reverse of record.
    pub(crate) fn from_record(label: &str, v: &EmployeeStockSale) -> Self {
        Self {
            calendar_year: Default::default(),
            account_number: Default::default(),
            box_1b_acquired: Default::default(),
            form_8949_checkbox: Default::default(),
            box_2_short_term: Default::default(),
            box_2_long_term: Default::default(),
            box_2_ordinary: Default::default(),
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            description: label.into(),
            sold: v.sold,
            proceeds: v.proceeds,
            withholding: v.withholding,
            reported_basis: v.reported_basis,
            reporting: v.reporting,
            taxed: v.acquired,
            value: v.market_value,
            paid: v.paid,
            more: EmployeeShareSaleMore {
                fees: v.fees,
                wash_sale: v.wash_sale,
                special: v.special,
            },
        }
    }
    // The entry label: the box 1a description as typed.
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}
