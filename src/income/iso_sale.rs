// Coverage: ISO sales (T3-small, 2026-09-27). IsoSale (the calculation: compensation on a
//   disqualifying sale, regular basis, AMT basis = box 4 value for shares
//   exercised in an earlier year, Form 6251 line 2k through the AMT capital gain, the AMT reversal
//   of the compensation) gains its page, IsoShareSale on Investment sales. Qualifying = sold more
//   than 2 years after the grant and more than 1 year after the exercise (Pub. 525). A sale whose
//   page says another special treatment applies (restricted shares, a wash sale, a divestiture
//   certificate) is left out (the stored IsoSale holds none of those facts: row 252) with the notice WARN_ISO_SALE_LEFT_OUT (Form 8949 and Form 6251
//   line 2k). Code-health 2 (2026-10-10): the stored sale no longer carries the five fields its
//   only page wrote as constants (restriction, divestiture, wash disposition and the two wash
//   losses): a restricted, wash-sale or divestiture sale is the page's special answer. Tests:
//   see src/tests/income/interview.rs T3-small block.
use crate::kernel::phase;
use crate::{CapitalSaleAmt, FederalSale, GainTerm, SaleUse};
use alloc::string::String;
use libertas_macros::*;

/// Incentive stock option sale
/// Use Form 3921 and sale records for one sold lot. The App derives compensation, holding period, regular basis and AMT basis. Share quantities support six decimal places; special histories still require review. Do not repeat this lot as another capital sale or as shares still held at year-end.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IsoSale {
    /// Option grant date (Form 3921 box 1)
    #[libertas_date_only]
    pub granted: u32,
    /// Exercise and stock-transfer date (Form 3921 box 2)
    #[libertas_date_only]
    pub exercised: u32,
    /// Sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Shares sold from this lot
    /// Use only the sold shares, not the entire exercise if some shares were retained. Enter up to six decimal places. The App normalizes the quantity to millionths before monetary calculations.
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Exercise price per share (Form 3921 box 3)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exercise_price: i64,
    /// Fair market value per share on exercise (Form 3921 box 4)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub market_price: i64,
    /// Amount paid for the option itself, allocated to these shares
    /// Total option purchase cost for this lot, separate from the exercise price and commissions. Enter zero if nothing was paid for the option. Exclude interest and amounts assigned to other lots.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub option_cost: i64,
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
    /// The compensation limit uses proceeds before these fees; capital gain uses net proceeds. Enter zero if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub selling_fees: i64,
    /// This lot's calculated compensation is already included in this filer's W-2 box 1 wages
    /// Keep the original W-2 wages unchanged. Choose false if no W-2 includes the compensation; the App then reports it on Schedule 1 line 8k. Confirm the entire calculated amount is included, resolving incorrect or partial employer reporting first. Qualifying sales and sales with no compensation use false.
    pub in_wages: bool,
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
    /// Ordinary employee ISO history confirmed
    /// This lot qualified as an ISO under the employment and plan requirements, was paid for in cash and unrestricted on exercise. Any prior AMT recognition spread was accounted for even if no AMT was payable. No related-party sale, gift, inheritance, stock split, return of capital, unentered election, foreign treatment or other unrecorded basis adjustment applies. For a same-year sale, do not add a separate held-share exercise adjustment. Check Form 3921 and the plan records. No leaves this sale out of the estimate with a notice; that situation is not supported.
    pub confirmed: bool,
}

impl IsoSale {
    fn lot_amount(&self, per_share: i64) -> i64 {
        crate::employee_stock::lot_amount(self.shares, per_share)
    }

    pub(crate) fn qualifying(&self) -> bool {
        // Holding periods start after grant/transfer: the anniversary itself
        // is still within the period. Calendar dates handle leap years.
        self.sold > crate::dates::add_years(self.granted, 2)
            && crate::dates::held_more_than_one_year(self.exercised, self.sold)
    }
    fn cost(&self) -> i64 {
        self.lot_amount(self.exercise_price) + self.option_cost + self.acquisition_costs
    }
    pub(crate) fn compensation(&self) -> i64 {
        if self.qualifying() {
            return 0;
        }
        let spread = phase::floor(
            self.lot_amount(phase::floor(self.market_price - self.exercise_price, 0))
                - self.option_cost,
            0,
        );
        // 1.422-1(b)(2): disposition fees do not reduce compensation.
        phase::cap(spread, [phase::floor(self.proceeds + self.selling_fees - self.cost(), 0)])
    }
    pub(crate) fn amt_compensation_adjustment(&self) -> i64 {
        if self.exercised / 10000 < 2026 { -self.compensation() } else { 0 }
    }
    pub(crate) fn sale(&self) -> FederalSale {
        let basis = self.cost() + self.compensation();
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
            basis: Some(basis),
            amt: crate::SaleAmt::Different {
                record: CapitalSaleAmt {
                    term: if self.exercised / 10000 == 2026 {
                        None
                    } else {
                        Some(if crate::dates::held_more_than_one_year(self.exercised, self.sold) {
                            GainTerm::Long
                        } else {
                            GainTerm::Short
                        })
                    },
                    basis: if self.exercised / 10000 == 2026 {
                        basis
                    } else {
                        phase::floor(
                            self.lot_amount(self.market_price),
                            self.lot_amount(self.exercise_price) + self.option_cost,
                        ) + self.acquisition_costs
                    },
                    wash_sale: 0,
                },
            },
            withholding: self.withholding,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            special: crate::SaleSpecial::NoneOfThese,
            // The ISO convention is no box 1e: only a printed figure that
            // differs from the actual basis is a code B correction.
            adjustment: crate::employee_stock::sale_adjustment(
                self.reporting,
                self.reported_basis,
                basis,
                basis,
            ),
            basis_reporting: self.reporting,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let max = 100000000000;
        if crate::employee_stock::share_units(self.shares).is_none()
            || ![self.granted, self.exercised, self.sold]
                .iter()
                .all(|v| crate::federal_rules::valid_birth(*v))
            || self.granted > self.exercised
            || self.exercised > self.sold
            || self.sold / 10000 != 2026
            || ![
                self.option_cost,
                self.exercise_price,
                self.market_price,
                self.acquisition_costs,
                self.proceeds,
                self.selling_fees,
                self.withholding,
            ]
            .iter()
            .all(|v| (0..=max).contains(v))
        {
            return false;
        }
        if self.proceeds + self.selling_fees > max || self.in_wages && self.compensation() == 0 {
            return false;
        }
        let sale = self.sale();
        let amt = sale.amt.different().unwrap();
        sale.basis_amount() <= max && amt.basis <= max
    }
}

pub(crate) fn reconcile_wages(d: &crate::FederalDraft, r: &mut crate::FederalResult) {
    for owner in [crate::Owner::Taxpayer, crate::Owner::Spouse] {
        let mut reported = 0;
        let mut wages = 0;
        for entry in d.income.iter().filter(|entry| entry.owner == owner) {
            match &entry.income {
                crate::FederalIncome::EsppSale { data } if data.in_wages => {
                    reported += data.compensation()
                }
                crate::FederalIncome::IsoSale { data } if data.in_wages => {
                    reported += data.compensation()
                }
                crate::FederalIncome::EmployeeStockSale { data }
                    if data.acquired / 10000 == 2026 =>
                {
                    reported += data.compensation()
                }
                crate::FederalIncome::Wage { data } => wages += data.wages,
                _ => {}
            }
        }
        if reported > wages {
            for entry in d.income.iter().filter(|entry| entry.owner == owner) {
                let context = crate::interview::income_issue_context(entry);
                let context = match &entry.income {
                    crate::FederalIncome::IsoSale { data }
                        if data.in_wages && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    // Each kind has its Investment sales page, and the W-2 is
                    // the Wages page's.
                    crate::FederalIncome::EsppSale { data }
                        if data.in_wages && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    crate::FederalIncome::EmployeeStockSale { data }
                        if data.acquired / 10000 == 2026 && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    _ => continue,
                };
                crate::issue_localized!(
                    r,
                    &context,
                    "ISO compensation",
                    "ISSUE_COMBINED_STOCK_COMPENSATION_MARKED_AS_ALREADY_REPORTED_CANNOT_EXCEE_0E5F6C00",
                    "Combined stock compensation marked as already reported cannot exceed this filer's entered W-2 box 1 wages. Reconcile employer reporting before calculating.",
                );
            }
        }
    }
}

/// ISO shares sold
/// Shares from an incentive stock option, sold in 2026. Enter the sale here with its 1099-B, not again as a 1099-B sale; the App figures the regular and AMT basis from your Form 3921.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the 1099-B sale of these incentive stock option shares exactly. Check its tax year against the supplied year; report a mismatch. The statement must show exactly one sale of these shares; if it shows several sales, report an error asking for only this sale. Return the box 1a description, or an empty string when it is blank. The number of shares, box 1c date sold and box 1d proceeds must show values; report a blank one as an error. Return the number of shares with up to six decimals. Return box 1e as reported_basis exactly as printed, or leave it empty when the box is blank, and return box 12 as reporting. Return zero for a clearly blank box 4. Report illegible or conflicting values as errors. Do not return Form 3921 values even when a Form 3921 is supplied, and do not infer selling fees, wage inclusion, option cost, special treatment or other facts absent from the 1099-B. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct IsoShareSale {
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
    /// Form 3921 box 1 date option granted
    /// From the Form 3921 for the exercise these shares came from, which may be from an earlier year.
    #[libertas_date_only]
    pub granted: u32,
    /// Form 3921 box 2 date option exercised
    #[libertas_date_only]
    pub exercised: u32,
    /// Form 3921 box 3 exercise price per share
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Form 3921 box 4 fair market value per share on exercise date
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// About this sale
    pub more: IsoShareSaleMore,
}
// Share counts are validated finite quantities (share_units), so equality
// is reflexive; the income document union needs Eq.
impl Eq for IsoShareSale {}

/// About this sale
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IsoShareSaleMore {
    /// Your W-2 box 1 includes the compensation from this sale
    /// Applies only when the sale is within 2 years of the grant or 1 year of the exercise. Choose false if no W-2 includes it; the App then reports it as other income.
    #[libertas_default(true)]
    pub in_wages: bool,
    /// Selling fees already subtracted from box 1d
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub selling_fees: i64,
    /// Commissions paid when you exercised, for these shares
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_costs: i64,
    /// Amount paid for the option itself, for these shares
    /// Separate from the exercise price. Enter zero if you paid nothing for the option.
    #[libertas_money("USD", 2)]
    // Removal deferred: page construction computes compensation and lot basis through unchecked lot_amount and cost arithmetic before native admission; widening needs the monetary owner.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub option_cost: i64,
    /// Restricted shares, a wash sale, a divestiture certificate or another special treatment
    #[libertas_default(false)]
    pub special: bool,
}
impl Default for IsoShareSaleMore {
    fn default() -> Self {
        Self {
            in_wages: true,
            selling_fees: Default::default(),
            acquisition_costs: Default::default(),
            option_cost: Default::default(),
            special: Default::default(),
        }
    }
}
impl Eq for IsoShareSaleMore {}

impl IsoShareSale {
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the 2026 sale date is a DateOnly-literal rule too).

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.granted > self.exercised {
            return Some(
                crate::validation::ValidationIssue::new("ISO_EXERCISED_BEFORE_GRANTED")
                    .at_field("exercised"),
            );
        }
        if self.exercised > self.sold {
            return Some(
                crate::validation::ValidationIssue::new("ISO_SOLD_BEFORE_EXERCISED")
                    .at_field("sold"),
            );
        }
        if !(20260101..=20261231).contains(&self.sold) {
            return Some(
                crate::validation::ValidationIssue::new("ISO_SALE_NOT_IN_RETURN_YEAR")
                    .at_field("sold"),
            );
        }
        None
    }
    // The stored sale this page writes. Compensation counts as in the W-2
    // only when the sale has some (a qualifying sale has none).
    pub(crate) fn record(&self) -> IsoSale {
        let IsoShareSaleMore { in_wages, selling_fees, acquisition_costs, option_cost, special } =
            self.more;
        let mut sale = IsoSale {
            granted: self.granted,
            exercised: self.exercised,
            sold: self.sold,
            shares: self.shares,
            exercise_price: self.exercise_price,
            market_price: self.market_price,
            option_cost,
            acquisition_costs,
            proceeds: self.proceeds,
            selling_fees,
            in_wages: false,
            withholding: self.withholding,
            reported_basis: self.reported_basis,
            reporting: self.reporting,
            confirmed: !special,
        };
        sale.in_wages = in_wages && sale.compensation() > 0;
        sale
    }
    // The reverse of record; None for a stored sale this page could not
    // have written.
    pub(crate) fn from_record(label: &str, v: &IsoSale) -> Option<Self> {
        let special = !v.confirmed;
        let compensation = v.compensation();
        let more = IsoShareSaleMore {
            in_wages: v.in_wages || compensation == 0,
            selling_fees: v.selling_fees,
            acquisition_costs: v.acquisition_costs,
            option_cost: v.option_cost,
            special,
        };
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
            exercise_price: v.exercise_price,
            market_price: v.market_price,
            more,
        };
        (page.record() == *v).then_some(page)
    }
    // The entry label: the box 1a description as typed.
    pub(crate) fn label(&self) -> String {
        self.description.clone()
    }
}
