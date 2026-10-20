// Coverage: ESPP sales (B7a, 2026-10-09). EsppSale (an employee stock purchase plan lot sold in
//   2026: qualifying when sold more than 2 years after the grant and more than 1 year after the
//   exercise, compensation = the lesser of the grant-date discount and the gain, otherwise the
//   exercise-date spread; basis = price paid + commissions + compensation; Pub. 525 Examples 10 and
//   11, section 423) gains its page, EsppShareSale, on Investment sales, with the Form 3922 boxes 1
//   to 5 and 8 entered and the Form 1099-B boxes 1a, 1c, 1d and 4 imported. Compensation is in the
//   W-2 box 1 (the default) or, if not, Schedule 1 line 8k. The lot is reported on Form 8949 with
//   code B (the broker's box 1e omits the compensation) in box A/D, Schedule D line 1b or 8b. The
//   stored divestiture certificate is removed (the page leaves it to its special answer). A sale
//   with a special treatment is left out with WARN_ESPP_SALE_LEFT_OUT. Forms: 8949, Schedule D,
//   3922, Schedule 1 line 8k. Tests: tests/investment/investment_kinds.rs espp_*.
use crate::kernel::phase;
use crate::{FederalSale, GainTerm, SaleUse};
use alloc::string::String;
use libertas_macros::*;

/// Employee stock purchase plan sale
/// Use Form 3922 and sale records for one sold lot. The App derives compensation, holding period and stock basis. Share quantities support six decimal places; special histories still require review. Do not repeat this lot as another capital sale or as shares still held at year-end.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppSale {
    /// Option grant date (Form 3922 box 1)
    #[libertas_date_only]
    pub granted: u32,
    /// Exercise and acquisition date (Form 3922 box 2)
    /// The employee acquired the shares on exercise. Box 7 can show a later transfer of legal title by the employee to a broker; do not substitute that date for acquisition.
    #[libertas_date_only]
    pub exercised: u32,
    /// Sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Shares sold from this lot
    /// Use only the sold shares, not the entire exercise if some shares were retained. Enter up to six decimal places. The App normalizes the quantity to millionths before monetary calculations.
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Fair market value per share on the grant date (Form 3922 box 3)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub grant_market: i64,
    /// Exercise price per share as if exercised on the grant date
    /// Use Form 3922 box 8 for a variable-price option. If box 8 is blank because the price was fixed at grant, enter that fixed price from the grant record; a blank box does not mean zero.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub grant_price: i64,
    /// Actual exercise price per share (Form 3922 box 5)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exercise_price: i64,
    /// Fair market value per share on exercise (Form 3922 box 4)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub market_price: i64,
    /// Acquisition commissions allocated to these sold shares
    /// Enter zero if none. Exclude selling fees already subtracted from proceeds.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub acquisition_costs: i64,
    /// Net sale proceeds after selling fees
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// Selling fees already subtracted from net proceeds
    /// The qualifying compensation limit uses proceeds before these fees; capital gain uses net proceeds. Enter zero if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub selling_fees: i64,
    /// This lot's calculated compensation is already included in this filer's W-2 box 1 wages
    /// Keep the original W-2 wages unchanged. Choose false if no W-2 includes the compensation; the App then reports it on Schedule 1 line 8k. Confirm the entire calculated amount is included, resolving incorrect or partial employer reporting first. Sales with no compensation use false.
    pub in_wages: bool,
    /// Disallowed regular-tax wash-sale loss
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wash_sale: i64,
    /// Federal income tax withheld on this sale
    /// Include withholding once, not again on another sale entry.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub withholding: i64,
    /// Basis the Form 1099-B printed in box 1e, when it shows one
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// Whether the broker reported the basis to the IRS (box 12)
    pub reporting: crate::BasisReporting,
    /// A certificate of divestiture, a gift, the owner's death, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl EsppSale {
    fn lot_amount(&self, per_share: i64) -> i64 {
        crate::employee_stock::lot_amount(self.shares, per_share)
    }

    pub(crate) fn qualifying(&self) -> bool {
        self.sold > crate::dates::add_years(self.granted, 2)
            && crate::dates::held_more_than_one_year(self.exercised, self.sold)
    }
    pub(crate) fn compensation(&self) -> i64 {
        if self.qualifying() {
            phase::cap(
                self.lot_amount(phase::floor(self.grant_market - self.grant_price, 0)),
                [phase::floor(
                    self.proceeds + self.selling_fees - self.lot_amount(self.exercise_price),
                    0,
                )],
            )
        } else {
            self.lot_amount(phase::floor(self.market_price - self.exercise_price, 0))
        }
    }
    pub(crate) fn sale(&self) -> FederalSale {
        FederalSale {
            term: if crate::dates::held_more_than_one_year(self.exercised, self.sold) {
                GainTerm::Long
            } else {
                GainTerm::Short
            },
            collectible: false,
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            proceeds: self.proceeds,
            basis: Some(
                self.lot_amount(self.exercise_price) + self.acquisition_costs + self.compensation(),
            ),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: self.withholding,
            use_kind: SaleUse::Investment { wash_sale: self.wash_sale },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: crate::employee_stock::sale_adjustment(
                self.reporting,
                self.reported_basis,
                self.lot_amount(self.exercise_price) + self.acquisition_costs,
                self.lot_amount(self.exercise_price) + self.acquisition_costs + self.compensation(),
            ),
            basis_reporting: self.reporting,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let max = 100000000000;
        crate::employee_stock::share_units(self.shares).is_some()
            && [self.granted, self.exercised, self.sold]
                .iter()
                .all(|v| crate::federal_rules::valid_birth(*v))
            && self.granted <= self.exercised
            && self.exercised <= self.sold
            && self.sold / 10000 == 2026
            && [
                self.grant_market,
                self.grant_price,
                self.exercise_price,
                self.market_price,
                self.acquisition_costs,
                self.proceeds,
                self.selling_fees,
                self.wash_sale,
                self.withholding,
            ]
            .iter()
            .all(|v| (0..=max).contains(v))
            && self.proceeds + self.selling_fees <= max
            && self.sale().basis_amount() <= max
            && self.wash_sale <= (self.sale().basis_amount() - self.proceeds).max(0)
    }
}

/// ESPP shares sold
/// Shares bought through an employee stock purchase plan and sold in 2026. Enter the sale here with its 1099-B, not again as a 1099-B sale; the App figures the compensation, holding period and basis from your Form 3922.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the 1099-B sale of these employee stock purchase plan shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The number of shares, box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return the number of shares with up to six decimals. Return box 1e as reported_basis exactly as printed, or leave it empty when the box is blank, and return box 12 as reporting. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Do not return Form 3922 values even when a Form 3922 is supplied, and do not infer selling fees, wage inclusion, commissions, a wash sale, special treatment or other facts absent from the 1099-B. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct EsppShareSale {
    /// 1099-B box 1a description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub description: String,
    /// 1099-B box 1a number of shares sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// 1099-B box 1c date sold
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// 1099-B box 1d proceeds
    /// Net of selling fees, as the 1099-B reports them.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// 1099-B box 1e cost or other basis
    /// As printed, before any correction: the broker's figure leaves out the compensation or net unrealized appreciation, which the App adds with Form 8949 code B. Leave empty when the box is blank.
    /// [NullText]
    /// No cost basis is printed in box 1e
    /// [SetText]
    /// The cost basis printed in box 1e is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub reported_basis: Option<i64>,
    /// 1099-B box 12 basis reported to the IRS
    /// Box A or D when the broker reported the basis to the IRS, B or E when it did not, C or F when no Form 1099-B was received.
    #[libertas_llm_doc_field]
    #[libertas_default(ReportedToIrs)]
    pub reporting: crate::BasisReporting,
    /// 1099-B box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Form 3922 box 1 date option granted
    /// From the Form 3922 for the purchase these shares came from, which may be from an earlier year.
    #[libertas_date_only]
    pub granted: u32,
    /// Form 3922 box 2 date option exercised
    /// The purchase date. Box 7 can show a later transfer to a broker; do not use that date.
    #[libertas_date_only]
    pub exercised: u32,
    /// Form 3922 box 3 fair market value per share on grant date
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_market: i64,
    /// Form 3922 box 4 fair market value per share on exercise date
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Form 3922 box 5 price paid per share
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Form 3922 box 8 price per share as if exercised on the grant date
    /// Box 8 is blank when the price was fixed on the grant date; then enter that fixed price from your plan records. A blank box does not mean zero.
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_price: i64,
    /// About this sale
    pub more: EsppShareSaleMore,
}
// Share counts are validated finite quantities (share_units), so equality
// is reflexive; the income document union needs Eq.
impl Eq for EsppShareSale {}

/// About this sale
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppShareSaleMore {
    /// Your W-2 box 1 includes the compensation from this sale
    /// Your employer may include it; check the W-2 or its statement. Choose false if no W-2 includes it; the App then reports it as other income.
    #[libertas_default(true)]
    pub in_wages: bool,
    /// Selling fees already subtracted from box 1d
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub selling_fees: i64,
    /// Commissions paid when you bought, for these shares
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_costs: i64,
    /// Wash sale loss disallowed
    /// When you bought the same stock within 30 days before or after selling these shares at a loss, including a plan purchase in that window.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub wash_sale: i64,
    /// A certificate of divestiture, a gift, the owner's death, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}
impl Default for EsppShareSaleMore {
    fn default() -> Self {
        Self {
            in_wages: true,
            selling_fees: Default::default(),
            acquisition_costs: Default::default(),
            wash_sale: Default::default(),
            special: Default::default(),
        }
    }
}
impl Eq for EsppShareSaleMore {}

impl EsppShareSale {
    // The first relationship between this page's fields that fails, by its
    // rule's key, in the order of the client rules, then the server-only
    // wash sale bound (the basis needs the per-share products and the
    // qualifying compensation limit, which the rule language cannot
    // express).

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.granted > self.exercised {
            return Some(
                crate::validation::ValidationIssue::new("ESPP_EXERCISED_BEFORE_GRANTED")
                    .at_field("exercised"),
            );
        }
        if self.exercised > self.sold {
            return Some(
                crate::validation::ValidationIssue::new("ESPP_SOLD_BEFORE_EXERCISED")
                    .at_field("sold"),
            );
        }
        if !(20260101..=20261231).contains(&self.sold) {
            return Some(
                crate::validation::ValidationIssue::new("ESPP_SALE_NOT_IN_RETURN_YEAR")
                    .at_field("sold"),
            );
        }
        let record = self.record();
        if record.wash_sale > (record.sale().basis_amount() - record.proceeds).max(0) {
            return Some(
                crate::validation::ValidationIssue::new("SALE_WASH_SALE_ABOVE_LOSS")
                    .at_field("wash_sale")
                    .in_field("more"),
            );
        }
        None
    }
    // The stored sale this page writes. Compensation counts as in the W-2
    // only when the sale has some.
    pub(crate) fn record(&self) -> EsppSale {
        let more = &self.more;
        let mut sale = EsppSale {
            granted: self.granted,
            exercised: self.exercised,
            sold: self.sold,
            shares: self.shares,
            grant_market: self.grant_market,
            grant_price: self.grant_price,
            exercise_price: self.exercise_price,
            market_price: self.market_price,
            acquisition_costs: more.acquisition_costs,
            proceeds: self.proceeds,
            selling_fees: more.selling_fees,
            in_wages: false,
            wash_sale: more.wash_sale,
            withholding: self.withholding,
            reported_basis: self.reported_basis,
            reporting: self.reporting,
            special: more.special,
        };
        sale.in_wages = more.in_wages && sale.compensation() > 0;
        sale
    }
    // The reverse of record.
    pub(crate) fn from_record(label: &str, v: &EsppSale) -> Option<Self> {
        let page = Self {
            description: label.into(),
            shares: v.shares,
            sold: v.sold,
            proceeds: v.proceeds,
            withholding: v.withholding,
            reported_basis: v.reported_basis,
            reporting: v.reporting,
            granted: v.granted,
            exercised: v.exercised,
            grant_market: v.grant_market,
            market_price: v.market_price,
            exercise_price: v.exercise_price,
            grant_price: v.grant_price,
            more: EsppShareSaleMore {
                in_wages: v.in_wages || v.compensation() == 0,
                selling_fees: v.selling_fees,
                acquisition_costs: v.acquisition_costs,
                wash_sale: v.wash_sale,
                special: v.special,
            },
        };
        (page.record() == *v).then_some(page)
    }
    // The entry label: the box 1a description as typed.
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}
