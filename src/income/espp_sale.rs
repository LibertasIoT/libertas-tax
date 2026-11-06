// Coverage: Form 3922 records an ESPP acquisition independently of any sale.
// The printed 1099-B sale and its keyed Investment sales answers project into
// the existing section 423 compensation, holding-period and basis owner.
// Qualifying compensation is the lesser grant-date discount or disposition gain;
// a disqualifying disposition uses the exercise-date spread. Pub. 525, examples 10–11.
// https://www.irs.gov/pub/irs-pdf/f3922.pdf
// Tests: investment/investment_kinds.rs and federal/undeclared_forms.rs.
use crate::kernel::phase;
use crate::{FederalSale, GainTerm, SaleUse};
use alloc::string::String;
use libertas_macros::*;

pub use crate::espp_source::{
    EsppAcquisition, EsppAcquisitionChoice, EsppAcquisitionFacts, EsppPlanRecord, EsppSaleAnswers,
    EsppShareSaleMore, Form3922, Form3922Paper,
};

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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_market: i64,
    /// Exercise price per share as if exercised on the grant date
    /// Use Form 3922 box 8 for a variable-price option. If box 8 is blank because the price was fixed at grant, enter that fixed price from the grant record; a blank box does not mean zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_price: Option<i64>,
    /// Actual exercise price per share (Form 3922 box 5)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Fair market value per share on exercise (Form 3922 box 4)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Acquisition commissions allocated to these sold shares
    /// Enter zero if none. Exclude selling fees already subtracted from proceeds.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub acquisition_costs: i64,
    /// Net sale proceeds after selling fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Selling fees already subtracted from net proceeds
    /// The qualifying compensation limit uses proceeds before these fees; capital gain uses net proceeds. Enter zero if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub selling_fees: i64,
    /// This lot's calculated compensation is already included in this filer's W-2 box 1 wages
    /// Keep the original W-2 wages unchanged. Choose false if no W-2 includes the compensation; the App then reports it on Schedule 1 line 8k. Confirm the entire calculated amount is included, resolving incorrect or partial employer reporting first. Sales with no compensation use false.
    pub in_wages: bool,
    /// Disallowed regular-tax wash-sale loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wash_sale: i64,
    /// Federal income tax withheld on this sale
    /// Include withholding once, not again on another sale entry.
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
        qualifying_dates(self.granted, self.exercised, self.sold)
    }
    pub(crate) fn compensation(&self) -> i64 {
        if self.qualifying() {
            phase::cap(
                self.lot_amount(phase::floor(
                    self.grant_market
                        - self
                            .grant_price
                            .expect("qualifying ESPP lot has its actual grant-date price"),
                    0,
                )),
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
    pub(crate) fn wash_sale_issue(&self) -> Option<crate::validation::ValidationIssue> {
        // Admit the lot arithmetic independently of its loss adjustment before
        // reporting the existing specific wash-sale relationship failure.
        let mut admitted = self.clone();
        admitted.wash_sale = 0;
        (admitted.valid()
            && self.wash_sale > (admitted.sale().basis_amount() - admitted.proceeds).max(0))
        .then(|| {
            crate::validation::ValidationIssue::new("SALE_WASH_SALE_ABOVE_LOSS")
                .at_field("wash_sale")
                .in_field("more")
        })
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
            && self.grant_price.is_none_or(|price| (0..=max).contains(&price))
            && (!self.qualifying() || self.grant_price.is_some())
            && self.proceeds + self.selling_fees <= max
            && self.sale().basis_amount() <= max
            && self.wash_sale <= (self.sale().basis_amount() - self.proceeds).max(0)
    }
}

/// ESPP shares sold
/// Shares bought through an employee stock purchase plan and sold in 2026. Enter the sale here with its 1099-B, not again as a 1099-B sale; the App figures the compensation, holding period and basis from your Form 3922.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 1099-B sale of these employee stock purchase plan shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The number of shares, box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return the number of shares with up to six decimals. Return box 1e as reported_basis exactly as printed, or leave it empty when the box is blank, and return box 12 as reporting. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Return plan_record from the employer's stock plan record supplied with the 1099-B for these shares: the option grant date, the purchase date, the market value per share on each date, the price paid per share, and the purchase price per share as if bought on the grant date, or an empty string for that price when the record does not show it. Return null for plan_record when no such record is supplied. Do not return Form 3922 values even when a Form 3922 is supplied, and do not infer selling fees, wage inclusion, commissions, a wash sale, special treatment or other facts absent from the 1099-B. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the printed calendar year as an integer such as 2026, or zero when blank. Read the account number exactly as printed, or empty text when blank. Read box 1b date acquired as the printed date, VARIOUS, INHERITED or another notation, or null when blank. Read the Form 8949 box letter printed on the statement, or null when none is printed, and box 2 short-term, long-term and ordinary as printed checkboxes (true only when checked).",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.description,@.shares,@.sold,@.proceeds,@.reported_basis,@.withholding",
    form_date_order = "MonthDayYear"
)]
pub struct EsppShareSale {
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
    // Arithmetic safety: preserve the ESPP monetary owner's admitted lot and amount limit.
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
    /// Employer's stock plan record of these shares
    /// [NullText]
    /// No stock plan record came with this 1099-B
    /// [SetText]
    /// The employer's stock plan record shows
    #[libertas_llm_doc_field]
    pub plan_record: Option<EsppPlanRecord>,
}
// Share counts are validated finite quantities (share_units), so equality
// is reflexive; the income document union needs Eq.
impl Eq for EsppShareSale {}

impl EsppShareSale {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (!(20260101..=20261231).contains(&self.sold)).then(|| {
            crate::validation::ValidationIssue::new("ESPP_SALE_NOT_IN_RETURN_YEAR").at_field("sold")
        })
    }
    pub(crate) fn from_record(label: &str, value: &EsppSale) -> Option<Self> {
        Some(Self {
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
            shares: value.shares,
            sold: value.sold,
            proceeds: value.proceeds,
            withholding: value.withholding,
            reported_basis: value.reported_basis,
            reporting: value.reporting,
            plan_record: None,
        })
    }
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}

pub(crate) fn qualifying_dates(granted: u32, exercised: u32, sold: u32) -> bool {
    sold > crate::dates::add_years(granted, 2)
        && crate::dates::held_more_than_one_year(exercised, sold)
}
