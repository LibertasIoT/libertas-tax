pub use crate::personal_refueling_source::FederalPersonalRefueling;

use crate::dates::source_valid_date;
use crate::kernel::phase;
use alloc::{collections::BTreeSet, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How a registered fuel vendor handled the federal tax in a qualifying sale
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalVendorTaxRelief {
    /// The selling price excluded the federal fuel tax
    TaxExcludedPrice,
    /// The vendor repaid the federal fuel tax to the buyer
    TaxRepaid,
    /// The buyer gave written consent for the vendor to claim the credit
    BuyerWrittenConsent,
    /// The vendor's tax handling records are unavailable
    // The buyer cannot reconstruct an unprovided vendor refund/consent record.
    // WARN_FUEL_SALE_LEFT_OUT omits this sale's credit conservatively.
    NotSure,
}

fn valid_government_tin(value: &str) -> bool {
    match value.as_bytes() {
        [a, b, c, d, e, f, g, h, i] => {
            [a, b, c, d, e, f, g, h, i].iter().all(|digit| digit.is_ascii_digit())
        }
        [a, b, b'-', c, d, e, f, g, h, i] => {
            [a, b, c, d, e, f, g, h, i].iter().all(|digit| digit.is_ascii_digit())
        }
        _ => false,
    }
}

/// Fuel sold on a registered credit-card issuer's 2026 Form 4136 line 13
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalCardIssuerFuel {
    /// Line 13a: undyed diesel originally taxed at 24.4 cents; credit 24.3 cents per gallon
    UndyedDiesel,
    /// Line 13b: undyed nonaviation kerosene originally taxed at 24.4 cents; credit 24.3 cents
    UndyedNonaviationKerosene,
    /// Line 13c: aviation kerosene originally taxed at 21.9 cents; credit 21.8 cents
    AviationKeroseneTaxed219,
    /// Line 13c write-in “Taxed at $.244”: aviation kerosene taxed at 24.4 cents; credit 24.3 cents
    AviationKeroseneTaxed244,
}

impl FederalCardIssuerFuel {
    fn line_index(self) -> usize {
        match self {
            Self::UndyedDiesel => 0,
            Self::UndyedNonaviationKerosene => 1,
            Self::AviationKeroseneTaxed219 => 2,
            Self::AviationKeroseneTaxed244 => 3,
        }
    }
}

/// How the card issuer provided the ultimate vendor with the federal fuel-tax amount
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalCardVendorSettlement {
    /// The issuer repaid or agreed to repay the tax to the fuel vendor
    RepaidOrAgreedToRepay,
    /// The vendor gave written consent for the issuer to claim the credit
    VendorWrittenConsent,
    /// Another documented arrangement directly or indirectly reimburses the vendor
    OtherDocumentedReimbursement,
    /// The card issuer/vendor settlement record is unavailable
    // This return cannot reconstruct an unprovided reimbursement or consent agreement.
    // WARN_FUEL_SALE_LEFT_OUT omits this sale's credit conservatively.
    NotSure,
}

/// One government fuel purchase on a registered issuer's credit card
/// Written buyer consent can support a claim with either a full buyer refund or a retained claimable amount. A retained claim has separate, narrow accrual- and cash-method accounting routes below.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCardIssuerSale {
    /// Unique card-transaction or vendor-invoice reference; these gallons cannot be claimed again
    #[libertas_size(min = 1)]
    pub sale_reference: String,
    /// Date the government buyer bought the fuel in 2026
    #[libertas_date_only]
    pub date: u32,
    /// State or local government buyer, including an Indian tribal government making essential government use
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Government buyer's taxpayer identification number from its signed Model Certificate R
    // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
    #[libertas_size(min = 9, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-?[0-9]{7}")]
    pub buyer_tin: String,
    /// Government buyer's address on its Model Certificate R
    #[libertas_size(min = 1)]
    pub buyer_address: String,
    /// Account or card reference covered by the buyer's Model Certificate R
    #[libertas_size(min = 1)]
    pub card_account_reference: String,
    /// Which separate Form 4136 line and original-tax rate applies
    pub fuel: FederalCardIssuerFuel,
    /// Qualified gallons
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual fuel cost from the card transaction and vendor records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// Federal tax was imposed at the original rate named by the selected fuel branch
    pub taxed_at_stated_rate: bool,
    /// The buyer purchased with a credit card this issuer issued to that government unit
    pub issuer_card_used_by_buyer: bool,
    /// The fuel was for exclusive state or local government use, or essential government use by an Indian tribal government
    pub exclusive_government_use: bool,
    /// The issuer did not collect the claimable federal fuel-tax amount from the government buyer
    pub tax_not_collected_from_buyer: bool,
    /// The government buyer gave written consent for this issuer to claim the tax collected on this sale
    /// Record this separately from the unexpired buyer certificate required below.
    #[libertas_default(false)]
    pub buyer_written_consent: bool,
    /// Date the issuer refunded or credited the collected claimable tax to the government buyer
    /// For the written-consent route this must be no earlier than the sale and no later than December 31, 2026. Keep the buyer-account credit or refund record.
    /// [NullText]
    /// No refund or credit was made to the buyer
    /// [SetText]
    /// The collected tax was refunded or credited on
    #[libertas_date_only]
    pub buyer_tax_refund_on: Option<u32>,
    /// All collected tax claimed on this line was refunded or credited to the government buyer
    /// The separate nonrefundable 0.1-cent LUST tax, if charged, is outside the Form 4136 line 13 claim.
    #[libertas_default(false)]
    pub claimable_tax_refunded_in_full: bool,
    /// Claimable tax collected from and retained with the buyer's written consent, excluding the nonrefundable LUST tax
    /// Enter the exact cents from the card ledger; all retained sales on this Form 4136 claim must reconcile to the rounded line 13 credit. Use zero if tax was not collected or was fully refunded.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub retained_claimable_tax: i64,
    /// The issuer's documented reimbursement or consent arrangement with the ultimate vendor
    pub vendor_settlement: FederalCardVendorSettlement,
    /// The issuer holds the buyer's signed Model Certificate R covering this card and sale, with no reason to doubt it
    /// It expires two years after its effective date or when a replacement is provided, whichever is earlier.
    pub buyer_certificate_current: bool,
    /// No Form 720, Form 8849, Form 4136 or other claimant has already claimed these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalCardIssuerSale {}

impl FederalCardIssuerSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && valid_government_tin(&self.buyer_tin)
            && !self.buyer_address.trim().is_empty()
            && !self.card_account_reference.trim().is_empty()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && (0..=self.actual_fuel_cost).contains(&self.retained_claimable_tax)
            && self.buyer_tax_refund_on.is_none_or(|date| {
                !self.tax_not_collected_from_buyer
                    && (self.date..=20261231).contains(&date)
                    && source_valid_date(date)
            })
    }

    pub(crate) fn qualified(&self) -> bool {
        self.taxed_at_stated_rate
            && self.issuer_card_used_by_buyer
            && self.exclusive_government_use
            && ((self.tax_not_collected_from_buyer
                && !self.buyer_written_consent
                && self.buyer_tax_refund_on.is_none()
                && !self.claimable_tax_refunded_in_full
                && self.retained_claimable_tax == 0)
                || (!self.tax_not_collected_from_buyer
                    && self.buyer_written_consent
                    && self.buyer_tax_refund_on.is_some()
                    && self.claimable_tax_refunded_in_full
                    && self.retained_claimable_tax == 0)
                || (!self.tax_not_collected_from_buyer
                    && self.buyer_written_consent
                    && self.buyer_tax_refund_on.is_none()
                    && !self.claimable_tax_refunded_in_full
                    && self.retained_claimable_tax > 0))
            && self.vendor_settlement != FederalCardVendorSettlement::NotSure
            && self.buyer_certificate_current
            && self.not_claimed_elsewhere
    }
}

/// Registered credit-card issuer's 2026 government-fuel claim on Form 4136 line 13
/// For a partnership K-1 box 20 code C statement, enter only the partner's allocated gallons and costs from the reporting partnership issuer. Do not also claim the partnership's full sale.
/// Claim sources: https://www.irs.gov/instructions/i4136 (line 13), https://www.irs.gov/pub/irs-dft/f4136--dft.pdf (2026 rates), and https://www.irs.gov/irb/2005-46_IRB#NOT-2005-80 (buyer/vendor consent and Model Certificate R).
/// Retained-tax income is an inference from the reconciled economics, not the IRS tax-benefit recovery rule: section 61 includes business gains (https://www.govinfo.gov/link/uscode/26/61), and Schedule C line 6 includes federal fuel credits and retained buyer-collected tax (https://www.irs.gov/instructions/i1040sc). An accrual issuer's fixed 2026 right follows the all-events rule, while a cash issuer recognizes the separate benefit when actually or constructively received or applied as a credit (https://www.irs.gov/publications/p334). Publication 510's filing-year rule expressly covers deducted-tax recoveries; it does not independently establish timing for this no-deduction route (https://www.irs.gov/publications/p510).
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCardIssuerClaim {
    /// IRS Form 637 credit-card-issuer registration number, including its CC prefix
    #[libertas_size(min = 3)]
    pub cc_registration_number: String,
    /// The CC registration was issued, is active for these sales, and belongs to this direct business or reporting partnership
    pub registration_active: bool,
    /// Government card purchases supporting lines 13a, 13b and 13c
    #[libertas_size(min = 1)]
    /// ----
    /// Keep each transaction, buyer certificate, original tax rate, cost and no-duplicate evidence.
    pub sales: Vec<FederalCardIssuerSale>,
    /// Card, tax-paid fuel, government-buyer, vendor reimbursement and Certificate R records are complete
    pub records_complete: bool,
    /// The buyer's card payment and vendor settlement are card-principal flows, not also included in business receipts or deducted expenses
    /// For a retained-tax sale the App separately accounts for the Form 4136 benefit in the proper income year. The vendor payment and buyer collection must not be entered as revenue or cost.
    pub settlement_outside_business_income_and_expenses: bool,
    /// Retained-tax accrual income facts are complete and the gain has not already been reported
    /// Choose Yes only for a direct accrual-method Schedule C issuer when buyer consent, current Certificate R, vendor arrangement and an unconditional right to retain the credit were fixed by December 31, 2026. The buyer reimbursed the claimable tax cost; buyer and vendor card-principal transfers are outside profit and loss; no tax-cost deduction, prior regular/AMT/NIIT benefit, duplicate income entry, passive treatment or known later adjustment applies. The App adds the retained line 13 benefit as Schedule C line 6 income once. Leave empty for cash accounting, partner sources, or when tax was not collected or was fully refunded.
    /// [NullText]
    /// This claim does not use the direct accrual-method retained-tax route
    /// [SetText]
    /// The direct accrual-method retained-tax facts are confirmed as
    pub retained_accrual_accounting_confirmed: Option<bool>,
    /// Direct cash-method retained-tax facts are complete; no claim benefit was received in 2026
    /// Choose Yes only when the buyer's claimable-tax payment and the matching vendor card-principal settlement were actually completed by December 31, 2026, with buyer written consent and a current Certificate R. Neither principal flow is a business receipt or expense; the claimable tax was never deducted; no 2026 Form 720/8849 refund, claim filing, payment, credit application, constructive receipt, earlier regular/AMT/NIIT benefit, duplicate income entry, passive treatment, or known later adjustment applies. The 2026 Form 4136 credit is a 2026 refundable payment; its separate business-income review belongs to the year the benefit is actually or constructively received or applied, often 2027. Leave empty for accrual accounting, partner sources, or when tax was not collected or was fully refunded. Do not confirm if a vendor arrangement remains merely an unpaid promise or receipt timing is uncertain.
    /// [NullText]
    /// This claim does not use the direct cash-method retained-tax route
    /// [SetText]
    /// The direct cash-method retained-tax facts are confirmed as
    pub retained_cash_accounting_confirmed: Option<bool>,
}

impl FederalCardIssuerClaim {
    pub(crate) fn has_retained_tax(&self) -> bool {
        self.sales.iter().any(|sale| sale.retained_claimable_tax > 0)
    }

    pub(crate) fn retained_income_cents(&self) -> i64 {
        if self.has_retained_tax() { self.lines().iter().sum() } else { 0 }
    }

    fn line_totals(&self) -> Option<[(i64, i64); 4]> {
        self.sales.iter().try_fold([(0_i64, 0_i64); 4], |mut totals, sale| {
            let line = &mut totals[sale.fuel.line_index()];
            line.0 = line.0.checked_add(crate::kernel::natural::thousandths(sale.gallons))?;
            line.1 = line.1.checked_add(sale.actual_fuel_cost)?;
            Some(totals)
        })
    }

    /// Rounded cents for lines 13a and 13b, the printed 13c rate, and its 24.4-cent-tax write-in rate
    pub(crate) fn lines(&self) -> [i64; 4] {
        self.line_totals().map_or([0; 4], |totals| {
            let rates = [243, 243, 218, 243];
            core::array::from_fn(|index| {
                crate::kernel::ratio::fraction(totals[index].0, rates[index], 10_000)
            })
        })
    }

    pub(crate) fn valid(&self) -> bool {
        let registration = self.cc_registration_number.trim();
        let mut references = BTreeSet::new();
        registration.starts_with("CC")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self
                .sales
                .iter()
                .all(|sale| sale.valid() && references.insert(sale.sale_reference.trim()))
            && self.line_totals().is_some_and(|totals| {
                let lines = self.lines();
                totals.iter().enumerate().all(|(index, (_, cost))| *cost >= lines[index])
            })
    }

    pub(crate) fn qualified(&self) -> bool {
        let retained = self.has_retained_tax();
        self.registration_active
            && self.records_complete
            && self.settlement_outside_business_income_and_expenses
            && self.sales.iter().all(FederalCardIssuerSale::qualified)
            && if retained {
                ((self.retained_accrual_accounting_confirmed == Some(true)
                    && self.retained_cash_accounting_confirmed.is_none())
                    || (self.retained_cash_accounting_confirmed == Some(true)
                        && self.retained_accrual_accounting_confirmed.is_none()
                        && self.sales.iter().all(|sale| {
                            sale.vendor_settlement
                                != FederalCardVendorSettlement::VendorWrittenConsent
                        })))
                    && self.sales.iter().all(|sale| sale.retained_claimable_tax > 0)
                    && self.sales.iter().try_fold([0_i64; 4], |mut retained, sale| {
                        let line = &mut retained[sale.fuel.line_index()];
                        *line = line.checked_add(sale.retained_claimable_tax)?;
                        Some(retained)
                    }) == Some(self.lines())
            } else {
                self.retained_accrual_accounting_confirmed.is_none()
                    && self.retained_cash_accounting_confirmed.is_none()
            }
    }
}

/// One 2026 government sale of undyed diesel or nonaviation kerosene by a registered ultimate vendor
/// Keep each sale distinct so its buyer, certificate, tax treatment and gallons can be checked. The government-buyer information is needed for the Form 4136 line 6a or 7a statement, although this App currently displays its results on screen rather than producing filing documents.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalGovernmentFuelSale {
    /// Date the undyed fuel was sold to the government buyer in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Name of the state or local government unit, or Indian tribal government making essential government use
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Government buyer's nine-digit taxpayer identification number
    /// Enter nine digits, with an optional hyphen after the first two digits.
    // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
    #[libertas_size(min = 9, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-?[0-9]{7}")]
    pub buyer_tin: String,
    /// Qualified undyed fuel sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual cost of this fuel from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// How the federal fuel tax was handled in this sale
    pub tax_relief: FederalVendorTaxRelief,
    /// The fuel was taxed at 24.4 cents per gallon and undyed, without visible evidence of dye
    /// For line 7a, this also confirms nonaviation kerosene; aviation kerosene belongs on a different line. A dye exception requires separate review and is not supported here.
    pub taxed_244_undyed_fuel: bool,
    /// The buyer's use was exclusively state or local government use, or essential government use by an Indian tribal government
    pub exclusive_government_use: bool,
    /// The buyer did not use a credit card issued to the government unit
    /// A state-issued credit-card purchase follows the credit-card-issuer claim rules, not this vendor path.
    pub no_government_credit_card: bool,
    /// A current Model Certificate P from this buyer is held, and nothing in it is known to be false
    /// The certificate must still be unexpired when this credit is claimed.
    pub buyer_certificate_current: bool,
    /// No Form 720, Form 8849 or other credit/refund claim was made for these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalGovernmentFuelSale {}

impl FederalGovernmentFuelSale {
    fn tin_valid(&self) -> bool {
        valid_government_tin(&self.buyer_tin)
    }

    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.buyer_name.trim().is_empty()
            && self.tin_valid()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.taxed_244_undyed_fuel
            && self.exclusive_government_use
            && self.no_government_credit_card
            && self.buyer_certificate_current
            && self.not_claimed_elsewhere
    }
}

/// Registered ultimate vendor's 2026 Form 4136 claim for government fuel sales
/// Use separately for line 6a undyed diesel and line 7a undyed nonaviation kerosene. Each line has its own qualified quantity, actual cost and cent rounding.
/// The registered seller is the direct sole proprietorship or the partnership named on a partner's separate K-1 box 20 code C statement. A partner enters only its allocated, nonduplicated share of the seller's qualified gallons and cost.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorStateClaim {
    /// IRS Form 637 ultimate-vendor registration number, including its UV prefix
    #[libertas_size(min = 3)]
    pub uv_registration_number: String,
    /// The seller's UV registration is active and belongs to this direct business or reporting partnership
    pub registration_active: bool,
    /// Government-buyer sales supporting this claim
    #[libertas_size(min = 1)]
    /// ----
    /// Each entry must have separate sale, cost, certificate and no-duplicate-claim evidence.
    pub sales: Vec<FederalGovernmentFuelSale>,
    /// Purchase, excise-tax, sale and buyer-certificate records are complete
    /// This includes the government-unit name, TIN and gallons required for the line 6a or 7a statement.
    pub records_complete: bool,
}

impl FederalVendorStateClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales.iter().try_fold(0_i64, |total, sale| {
            total.checked_add(crate::kernel::natural::thousandths(sale.gallons))
        })
    }

    fn total_cost(&self) -> Option<i64> {
        self.sales.iter().try_fold(0_i64, |total, sale| total.checked_add(sale.actual_fuel_cost))
    }

    /// Rounded cents for one printed Form 4136 line.
    /// Add that line's eligible gallon quantities before rounding. The caller owns its line-specific rate.
    pub(crate) fn line_cents(&self, rate_thousandths: i64) -> i64 {
        self.total_thousandths()
            .map_or(0, |gallons| crate::kernel::ratio::fraction(gallons, rate_thousandths, 10_000))
    }

    pub(crate) fn valid(&self, rate_thousandths: i64) -> bool {
        let registration = self.uv_registration_number.trim();
        registration.starts_with("UV")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self.sales.iter().all(FederalGovernmentFuelSale::valid)
            && self.total_thousandths().is_some()
            && self.total_cost().is_some_and(|cost| cost >= self.line_cents(rate_thousandths))
    }

    pub(crate) fn qualified(&self) -> bool {
        self.registration_active
            && self.records_complete
            && self.sales.iter().all(FederalGovernmentFuelSale::qualified)
    }
}

/// One 2026 sale of undyed diesel or nonaviation kerosene for public intercity or local bus service
/// Keep the seller's sale and buyer-waiver evidence distinct from an ultimate purchaser's lines 3d and 4c. A partner enters only the allocated, nonduplicated share of the partnership seller's gallons and cost.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorBusSale {
    /// Date of the sale to the bus operator in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Name of the bus operator that signed the waiver
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Address of the bus operator that signed the waiver
    #[libertas_size(min = 1)]
    pub buyer_address: String,
    /// Qualified undyed fuel sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual fuel cost from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// Tax-excluded price, repayment, or written buyer consent for the vendor's claim
    pub tax_relief: FederalVendorTaxRelief,
    /// The sale was undyed fuel taxed at 24.4 cents per gallon, without visible evidence of dye
    /// Kerosene must be nonaviation kerosene. A dye exception needs separate review and is unsupported here.
    pub taxed_244_undyed_nonaviation: bool,
    /// The bus furnished passenger land transportation for compensation
    pub compensated_passenger_service: bool,
    /// The bus was available for hire by the general public, not only a limited group
    pub available_to_general_public: bool,
    /// The bus operated scheduled transportation along regular routes
    /// When No (assumed nonscheduled, with a note), at least 20 adult seats excluding the driver are required for nonscheduled service.
    pub scheduled_regular_routes: bool,
    /// Adult passenger seats excluding the driver, required for nonscheduled service
    /// Leave empty for scheduled regular routes.
    /// [NullText]
    /// The bus ran scheduled transportation along regular routes
    /// [SetText]
    /// The number of adult passenger seats excluding the driver was
    #[libertas_number(min = 1)]
    pub adult_seats_excluding_driver: Option<u32>,
    /// The sale was not for a school bus or a qualified local-bus claim
    /// Those uses have different Form 4136 lines and rates.
    pub not_school_or_qualified_local_bus: bool,
    /// The buyer gave the vendor a signed Model Waiver N covering this sale
    /// It must still be unexpired when the credit is claimed; single-sale or account waivers are allowed.
    pub buyer_waiver_current: bool,
    /// No Form 720, Form 8849, purchaser credit, or other credit/refund was claimed for these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalVendorBusSale {}

impl FederalVendorBusSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && self.adult_seats_excluding_driver.is_none_or(|seats| (1..=1_000).contains(&seats))
            && (self.scheduled_regular_routes || self.adult_seats_excluding_driver.is_some())
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.taxed_244_undyed_nonaviation
            && self.compensated_passenger_service
            && self.available_to_general_public
            // No: nonscheduled service needs the 20-seat test.
            && (self.scheduled_regular_routes
                || self
                    .adult_seats_excluding_driver
                    .is_some_and(|seats| seats >= 20))
            && self.not_school_or_qualified_local_bus
            && self.buyer_waiver_current
            && self.not_claimed_elsewhere
    }
}

/// Registered ultimate vendor's Form 4136 line 6b diesel or line 7c nonaviation-kerosene bus sales
/// Use one claim per printed line. The direct seller or reporting partnership holds the UB registration and current buyer waivers; only its separate qualified gallons and costs are entered on each line.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorBusClaim {
    /// IRS Form 637 ultimate-vendor bus registration number, including its UB prefix
    #[libertas_size(min = 3)]
    pub ub_registration_number: String,
    /// The seller's UB registration is active and belongs to this direct business or partnership
    pub registration_active: bool,
    /// Bus-operator sales supporting this fuel's Form 4136 line
    #[libertas_size(min = 1)]
    /// ----
    /// Each entry retains its own buyer, sale, waiver, tax, bus-use, cost and no-duplicate-claim evidence.
    pub sales: Vec<FederalVendorBusSale>,
    /// Purchase, excise-tax, sale and buyer-waiver records are complete
    /// Include each sale date, buyer name and address, qualified gallons and the signed Model Waiver N.
    pub records_complete: bool,
}

impl FederalVendorBusClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales.iter().try_fold(0_i64, |total, sale| {
            total.checked_add(crate::kernel::natural::thousandths(sale.gallons))
        })
    }

    pub(crate) fn line_cents(&self) -> i64 {
        self.total_thousandths()
            .map_or(0, |gallons| crate::kernel::ratio::fraction(gallons, 170, 10_000))
    }

    pub(crate) fn valid(&self) -> bool {
        let registration = self.ub_registration_number.trim();
        registration.starts_with("UB")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self.sales.iter().all(FederalVendorBusSale::valid)
            && self.total_thousandths().is_some()
            && self
                .sales
                .iter()
                .try_fold(0_i64, |total, sale| total.checked_add(sale.actual_fuel_cost))
                .is_some_and(|cost| cost >= self.line_cents())
    }

    pub(crate) fn qualified(&self) -> bool {
        self.registration_active
            && self.records_complete
            && self.sales.iter().all(FederalVendorBusSale::qualified)
    }
}

/// Federal tax originally paid on aviation kerosene sold for a qualifying flight
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalVendorAviationTaxRate {
    /// Originally taxed at 21.9 cents per gallon — commercial line 8a returns 17.5 cents; nontaxable line 8e returns 21.8 cents
    TaxedAt219,
    /// Originally taxed at 24.4 cents per gallon — commercial line 8b returns 20.0 cents; nontaxable line 8d returns 24.3 cents
    TaxedAt244,
}

/// One 2026 aviation-kerosene sale by an IRS-registered ultimate vendor
/// Enter only the allocated share of a partnership seller's sale when this is reported on a partner's K-1 box 20 code C statement.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorAviationSale {
    /// Unique invoice or delivery-line reference for this sale within this claim
    /// Keep each sold gallon on one Form 4136 line and out of other credit or refund claims.
    #[libertas_size(min = 1)]
    pub sale_reference: String,
    /// Date the kerosene was sold to the aircraft operator in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Name of the aircraft operator that signed Model Waiver L
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Address of the aircraft operator that signed Model Waiver L
    #[libertas_size(min = 1)]
    pub buyer_address: String,
    /// Qualified kerosene sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// The federal tax rate originally imposed on these sold gallons
    pub original_tax_rate: FederalVendorAviationTaxRate,
    /// The vendor sold at a tax-excluded price without collecting the tax, repaid it, or obtained the buyer's written consent
    pub tax_relief: FederalVendorTaxRelief,
    /// The buyer will use all these gallons in commercial aviation other than foreign trade
    /// Excludes private or mixed aviation, foreign-trade flights, and state or local government sales.
    pub commercial_aviation_other_than_foreign_trade: bool,
    /// Federal tax was paid at the selected original rate on all these gallons
    pub taxed_at_stated_rate: bool,
    /// The buyer's signed Model Waiver L covers this sale and is unexpired when the credit is claimed
    /// Keep the signed waiver and its single-purchase reference or account terms with the sale records.
    pub buyer_waiver_current: bool,
    /// Neither the vendor, buyer, nor another claimant filed a Form 720, Form 8849, Form 4136 or other claim for these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalVendorAviationSale {}

impl FederalVendorAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.commercial_aviation_other_than_foreign_trade
            && self.taxed_at_stated_rate
            && self.buyer_waiver_current
            && self.not_claimed_elsewhere
    }

    fn line_index(&self) -> usize {
        usize::from(self.original_tax_rate == FederalVendorAviationTaxRate::TaxedAt244)
    }
}

/// Registered ultimate vendor's 2026 commercial-aviation kerosene sales on Form 4136 lines 8a and 8b
/// The seller is an ordinary Schedule C business or the partnership identified by a partner's K-1 box 20 code C statement. Keep each rate's qualified gallons and cost separate.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorAviationClaim {
    /// IRS Form 637 aviation ultimate-vendor registration number, including its UA prefix
    #[libertas_size(min = 3)]
    pub ua_registration_number: String,
    /// The seller's UA registration is active and belongs to this business or reporting partnership
    pub registration_active: bool,
    /// Commercial-aviation kerosene sales supported by signed buyer waivers
    #[libertas_size(min = 1)]
    /// ----
    /// Each entry retains the sale, buyer, original tax rate, cost and no-duplicate-claim evidence.
    pub sales: Vec<FederalVendorAviationSale>,
    /// The vendor retained the fuel purchase, excise-tax, sale and signed Model Waiver L records
    pub records_complete: bool,
}

impl FederalVendorAviationClaim {
    fn line_totals(&self) -> Option<[(i64, i64); 2]> {
        self.sales.iter().try_fold([(0_i64, 0_i64); 2], |mut totals, sale| {
            let line = &mut totals[sale.line_index()];
            line.0 = line.0.checked_add(crate::kernel::natural::thousandths(sale.gallons))?;
            line.1 = line.1.checked_add(sale.actual_fuel_cost)?;
            Some(totals)
        })
    }

    /// Credits for Form 4136 lines 8a and 8b, each rounded once after adding its sale quantities
    pub(crate) fn lines(&self) -> [i64; 2] {
        self.line_totals().map_or([0, 0], |totals| {
            [
                crate::kernel::ratio::fraction(totals[0].0, 175, 10_000),
                crate::kernel::ratio::fraction(totals[1].0, 200, 10_000),
            ]
        })
    }

    pub(crate) fn valid(&self) -> bool {
        let registration = self.ua_registration_number.trim();
        let mut references = BTreeSet::new();
        registration.starts_with("UA")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self
                .sales
                .iter()
                .all(|sale| sale.valid() && references.insert(sale.sale_reference.trim()))
            && self.line_totals().is_some_and(|totals| {
                let lines = self.lines();
                totals[0].1 >= lines[0] && totals[1].1 >= lines[1]
            })
    }

    pub(crate) fn qualified(&self) -> bool {
        self.registration_active
            && self.records_complete
            && self.sales.iter().all(FederalVendorAviationSale::qualified)
    }
}

/// One 2026 sale of taxed aviation kerosene for a nonexempt, noncommercial flight
/// The buyer's Model Certificate Q supports this vendor-only Form 4136 line 8c claim. Do not enter exempt, commercial, foreign-trade, government, or mixed-use gallons here.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorNoncommercialAviationSale {
    /// Unique invoice or delivery-line reference within this vendor's claim
    #[libertas_size(min = 1)]
    pub sale_reference: String,
    /// Date of the kerosene sale in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Name of the aircraft operator that signed Model Certificate Q
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Address of the aircraft operator that signed Model Certificate Q
    #[libertas_size(min = 1)]
    pub buyer_address: String,
    /// Qualified kerosene sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// The vendor sold at a tax-excluded price, repaid the tax, or obtained the buyer's written consent
    pub tax_relief: FederalVendorTaxRelief,
    /// These gallons were taxed at 24.4 cents per gallon, including the 2.5 cents claimed here
    /// The 21.9-cent nonexempt rate leaves no 2.5-cent difference to claim.
    pub taxed_at_244: bool,
    /// The entire sale was for nonexempt, noncommercial aviation
    /// Mixed commercial and noncommercial use requires a separate allocation and is unsupported here.
    pub nonexempt_noncommercial_use: bool,
    /// The buyer's signed Model Certificate Q covers this sale and is current when the claim is made
    pub buyer_certificate_current: bool,
    /// Neither the vendor, buyer, nor another claimant made a Form 720, 8849, 4136, or other claim for these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalVendorNoncommercialAviationSale {}

impl FederalVendorNoncommercialAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.taxed_at_244
            && self.nonexempt_noncommercial_use
            && self.buyer_certificate_current
            && self.not_claimed_elsewhere
    }
}

/// Registered ultimate vendor's 2026 Form 4136 line 8c claim
/// The direct ordinary Schedule C seller or partnership named on a partner's K-1 box 20 code C statement holds the UA registration and Model Certificate Q. A partner enters only its allocated share of qualified sale gallons and cost.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorNoncommercialAviationClaim {
    /// IRS Form 637 aviation ultimate-vendor registration number, including its UA prefix
    #[libertas_size(min = 3)]
    pub ua_registration_number: String,
    /// The seller's UA registration is active and belongs to this business or reporting partnership
    pub registration_active: bool,
    /// Nonexempt, noncommercial aviation sales supported by signed buyer certificates
    #[libertas_size(min = 1)]
    /// ----
    /// Keep each sale's buyer, gallons, original tax, cost and no-duplicate-claim evidence.
    pub sales: Vec<FederalVendorNoncommercialAviationSale>,
    /// The vendor retained purchase, excise-tax, sale and signed Model Certificate Q records
    pub records_complete: bool,
}

/// Form 4136 use types allowed for registered-vendor nontaxable noncommercial aviation sales
/// Lines 8d and 8e accept only these use types. Foreign-trade sales also generate the separate 8f LUST credit.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalAviationNontaxableUse {
    /// Type 1: on a farm for farming purposes
    Farming,
    /// Type 9: in foreign trade, including reciprocal treatment for a foreign-registered aircraft
    ForeignTrade,
    /// Type 10: qualifying helicopter or fixed-wing aircraft use
    HelicopterOrFixedWing,
    /// Type 11: exclusive use by a qualified blood collector organization
    BloodCollector,
    /// Type 13: exclusive use by a nonprofit educational organization
    NonprofitEducation,
    /// Type 14: exclusive state or local government use; requires UV registration and Model Certificate P
    StateOrLocalGovernment,
    /// Type 15: in an aircraft owned by an eligible aircraft museum
    AircraftMuseum,
    /// Type 16: in military aircraft
    MilitaryAircraft,
}

/// Seller's IRS Form 637 registration for an aviation-kerosene vendor claim
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAviationVendorRegistration {
    /// IRS-issued registration number, including its UA or UV activity prefix
    #[libertas_size(min = 3)]
    pub number: String,
    /// This registration is active and belongs to the direct business or reporting partnership
    pub active: bool,
}

impl FederalAviationVendorRegistration {
    fn valid(&self, prefix: &str) -> bool {
        let number = self.number.trim();
        number.starts_with(prefix) && number.len() >= 3
    }
}

/// One 2026 registered-vendor sale of taxed aviation kerosene for nontaxable noncommercial use
/// Keep a sale's qualifying gallons on only one of lines 8d/8e; foreign-trade gallons also contribute to line 8f. If an invoice includes commercial or nonexempt use, keep its allocation evidence and give each separately eligible delivery line a distinct reference.
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorNontaxableAviationSale {
    /// Unique invoice or delivery-line reference across this seller's aviation vendor claims
    #[libertas_size(min = 1)]
    pub sale_reference: String,
    /// Date the kerosene was sold in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Ultimate buyer's name
    #[libertas_size(min = 1)]
    pub buyer_name: String,
    /// Ultimate buyer's address
    #[libertas_size(min = 1)]
    pub buyer_address: String,
    /// Government buyer's nine-digit TIN for type 14; leave empty for other uses
    /// Enter nine digits, with an optional hyphen after the first two digits.
    /// [NullText]
    /// This sale was not government type 14
    /// [SetText]
    /// The government buyer's TIN is
    // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
    #[libertas_size(min = 9, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-?[0-9]{7}")]
    pub government_buyer_tin: Option<String>,
    /// Qualifying 2026 nontaxable aviation use
    pub use_type: FederalAviationNontaxableUse,
    /// Federal tax originally imposed: 24.4 cents for line 8d or 21.9 cents for line 8e
    pub original_tax_rate: FederalVendorAviationTaxRate,
    /// Qualified kerosene sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// The vendor excluded the tax from the price, repaid it, or obtained written buyer consent
    pub tax_relief: FederalVendorTaxRelief,
    /// The selected federal tax rate was imposed on all of these gallons
    pub taxed_at_stated_rate: bool,
    /// All these gallons qualify for the selected Form 4136 nontaxable aviation use
    /// For foreign trade by a foreign-registered aircraft, confirm reciprocal benefits for U.S.-registered aircraft.
    pub qualifying_nontaxable_use: bool,
    /// Current signed Model Waiver L, or Model Certificate P for government use, covers this sale
    pub buyer_document_current: bool,
    /// For type 14, the government did not buy with a government-issued credit card
    /// Leave empty for other use types.
    /// [NullText]
    /// This sale was not government type 14
    /// [SetText]
    /// No government-issued credit card was used, confirmed as
    pub no_government_credit_card: Option<bool>,
    /// No purchaser, vendor, credit-card issuer or other claimant already claimed these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalVendorNontaxableAviationSale {}

impl FederalVendorNontaxableAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && if self.use_type == FederalAviationNontaxableUse::StateOrLocalGovernment {
                self.government_buyer_tin.as_ref().is_some_and(|tin| valid_government_tin(tin))
                    && self.no_government_credit_card.is_some()
            } else {
                self.government_buyer_tin.is_none() && self.no_government_credit_card.is_none()
            }
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.taxed_at_stated_rate
            && self.qualifying_nontaxable_use
            && self.buyer_document_current
            && self.not_claimed_elsewhere
            && (self.use_type != FederalAviationNontaxableUse::StateOrLocalGovernment
                || self.no_government_credit_card == Some(true))
    }

    fn line_index(&self) -> usize {
        usize::from(self.original_tax_rate == FederalVendorAviationTaxRate::TaxedAt219)
    }
}

/// Registered vendor's 2026 nontaxable noncommercial aviation claims on Form 4136 lines 8d–8f
/// The direct Schedule C seller or K-1 box 20 code C reporting partnership keeps the sale, buyer, original tax, cost, and transfer-of-claim records. A partner enters only its allocated share. See the 2026 draft Form 4136 and Publication 510.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalVendorNontaxableAviationClaim {
    /// UA registration is required for all use types except government type 14
    /// [NullText]
    /// No sale in this claim requires UA registration
    /// [SetText]
    /// The vendor's UA registration is
    pub ua_registration: Option<FederalAviationVendorRegistration>,
    /// UV registration is required for government type 14
    /// [NullText]
    /// No sale in this claim is government type 14
    /// [SetText]
    /// The vendor's UV registration is
    pub uv_registration: Option<FederalAviationVendorRegistration>,
    /// Nontaxable noncommercial aviation sales for lines 8d and 8e; foreign trade also contributes to 8f
    #[libertas_size(min = 1)]
    /// ----
    /// Each entry holds one distinct sale, tax rate, use type, buyer document and actual cost.
    pub sales: Vec<FederalVendorNontaxableAviationSale>,
    /// The seller retained fuel purchase, excise-tax, sale, signed waiver or certificate and government buyer records
    pub records_complete: bool,
}

impl FederalVendorNontaxableAviationClaim {
    fn totals(&self) -> Option<([(i64, i64); 2], i64)> {
        self.sales.iter().try_fold(
            ([(0_i64, 0_i64); 2], 0_i64),
            |(mut lines, foreign_trade), sale| {
                let line = &mut lines[sale.line_index()];
                line.0 = line.0.checked_add(crate::kernel::natural::thousandths(sale.gallons))?;
                line.1 = line.1.checked_add(sale.actual_fuel_cost)?;
                let foreign_trade = if sale.use_type == FederalAviationNontaxableUse::ForeignTrade {
                    foreign_trade.checked_add(crate::kernel::natural::thousandths(sale.gallons))?
                } else {
                    foreign_trade
                };
                Some((lines, foreign_trade))
            },
        )
    }

    /// Lines 8d, 8e and 8f, each rounded once after summing that line's gallons
    pub(crate) fn lines(&self) -> [i64; 3] {
        self.totals().map_or([0; 3], |(totals, foreign_trade)| {
            [
                crate::kernel::ratio::fraction(totals[0].0, 243, 10_000),
                crate::kernel::ratio::fraction(totals[1].0, 218, 10_000),
                crate::kernel::ratio::fraction(foreign_trade, 1, 10_000),
            ]
        })
    }

    pub(crate) fn valid(&self) -> bool {
        let mut references = BTreeSet::new();
        !self.sales.is_empty()
            && self.ua_registration.as_ref().is_none_or(|registration| registration.valid("UA"))
            && self.uv_registration.as_ref().is_none_or(|registration| registration.valid("UV"))
            && self.sales.iter().all(|sale| {
                sale.valid()
                    && references.insert(sale.sale_reference.trim())
                    && if sale.use_type == FederalAviationNontaxableUse::StateOrLocalGovernment {
                        self.uv_registration.is_some()
                    } else {
                        self.ua_registration.is_some()
                    }
            })
            && self.totals().is_some_and(|(totals, _)| {
                let lines = self.lines();
                totals[0].1 >= lines[0]
                    && totals[1].1 >= lines[1]
                    && totals[0]
                        .1
                        .checked_add(totals[1].1)
                        .is_some_and(|cost| cost >= lines.into_iter().sum::<i64>())
            })
    }

    pub(crate) fn qualified(&self) -> bool {
        self.records_complete
            && self.sales.iter().all(FederalVendorNontaxableAviationSale::qualified)
            && self.sales.iter().all(|sale| {
                if sale.use_type == FederalAviationNontaxableUse::StateOrLocalGovernment {
                    self.uv_registration.as_ref().is_some_and(|r| r.active)
                } else {
                    self.ua_registration.as_ref().is_some_and(|r| r.active)
                }
            })
    }
}

impl FederalVendorNoncommercialAviationClaim {
    fn totals(&self) -> Option<(i64, i64)> {
        self.sales.iter().try_fold((0_i64, 0_i64), |totals, sale| {
            Some((
                totals.0.checked_add(crate::kernel::natural::thousandths(sale.gallons))?,
                totals.1.checked_add(sale.actual_fuel_cost)?,
            ))
        })
    }

    /// Form 4136 line 8c: 2.5 cents per gallon, rounded once after adding its sales
    pub(crate) fn line_cents(&self) -> i64 {
        self.totals().map_or(0, |(gallons, _)| crate::kernel::ratio::fraction(gallons, 25, 10_000))
    }

    pub(crate) fn valid(&self) -> bool {
        let registration = self.ua_registration_number.trim();
        let mut references = BTreeSet::new();
        registration.starts_with("UA")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self
                .sales
                .iter()
                .all(|sale| sale.valid() && references.insert(sale.sale_reference.trim()))
            && self.totals().is_some_and(|(_, cost)| cost >= self.line_cents())
    }

    pub(crate) fn qualified(&self) -> bool {
        self.registration_active
            && self.records_complete
            && self.sales.iter().all(FederalVendorNoncommercialAviationSale::qualified)
    }
}

/// How a fixed retail pump prevents undyed kerosene from fueling a highway vehicle or train
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalBlockedPumpAccess {
    /// The pump cannot reasonably fill a diesel-powered highway vehicle or train directly
    CannotFillHighwayVehicleOrTrain,
    /// The vendor locks the pump after each sale and unlocks it only for a qualifying request
    LockedBetweenSales,
    /// The vendor's pump-access records are unavailable
    // A buyer's invoice cannot reconstruct the vendor's access/locking arrangement.
    // WARN_FUEL_CLAIM_LEFT_OUT omits this claim conservatively.
    NotSure,
}

/// One 2026 sale of undyed nonaviation kerosene from a blocked retail pump
/// The invoice, the buyer's signed certificate or waiver and the sale records answer its questions. Any No that no other claim covers the gallons or that the buyer's use is as stated, leaves this sale out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBlockedPumpKeroseneSale {
    /// Date of this sale in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Buyer's name; required with an address for locked-pump sales over five gallons
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub buyer_name: String,
    /// Buyer's address; required with a name for locked-pump sales over five gallons
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub buyer_address: String,
    /// Undyed nonaviation kerosene sold
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual cost of this fuel from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_fuel_cost: i64,
    /// How the federal fuel tax was handled in this sale
    pub tax_relief: FederalVendorTaxRelief,
    /// The sold nonaviation kerosene was taxed at 24.4 cents per gallon and had no visible dye
    /// A dye exception requires separate review and is not supported here.
    pub taxed_244_undyed_nonaviation: bool,
    /// The buyer requested the kerosene for a nontaxable use, not propulsion of a diesel-powered highway vehicle or train
    pub buyer_nontaxable_use: bool,
    /// The vendor had no reason to believe the buyer would use the kerosene otherwise
    pub no_reason_to_doubt_use: bool,
    /// No Form 720, Form 8849 or other credit/refund claim was made for these gallons
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalBlockedPumpKeroseneSale {}

impl FederalBlockedPumpKeroseneSale {
    fn valid(&self, locked: bool) -> bool {
        (20260101..=20261231).contains(&self.date)
            && source_valid_date(self.date)
            && (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && self.buyer_issue(locked).is_none()
    }

    fn buyer_issue(&self, locked: bool) -> Option<crate::validation::ValidationIssue> {
        let required = locked && crate::kernel::natural::thousandths(self.gallons) > 5_000;
        let field = if (!self.buyer_name.is_empty() && self.buyer_name.trim().is_empty())
            || (self.buyer_name.is_empty() && (required || !self.buyer_address.is_empty()))
        {
            "buyer_name"
        } else if (!self.buyer_address.is_empty() && self.buyer_address.trim().is_empty())
            || (self.buyer_address.is_empty() && (required || !self.buyer_name.is_empty()))
        {
            "buyer_address"
        } else {
            return None;
        };
        Some(
            crate::validation::ValidationIssue::new("FUEL_BLOCKED_PUMP_BUYER_RECORD")
                .at_field(field),
        )
    }

    pub(crate) fn qualified(&self) -> bool {
        self.tax_relief != FederalVendorTaxRelief::NotSure
            && self.taxed_244_undyed_nonaviation
            && self.buyer_nontaxable_use
            && self.no_reason_to_doubt_use
            && self.not_claimed_elsewhere
    }
}

/// Registered ultimate vendor's 2026 Form 4136 line 7b blocked-pump kerosene claim
/// Use only for sales by the registered UP seller. Government sales on line 7a and buyer home use on line 4a are different claims; each gallon may be claimed once.
/// IRS Letter 637 and the vendor's own records answer the registration and record questions. Any No leaves this claim out of the fuel tax credit with a review note.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBlockedPumpKeroseneClaim {
    /// IRS Form 637 blocked-pump ultimate-vendor registration number, including its UP prefix
    #[libertas_size(min = 3)]
    pub up_registration_number: String,
    /// The seller's UP registration is active and belongs to this direct business or reporting partnership
    pub registration_active: bool,
    /// The retail pump is at a fixed location
    pub pump_fixed: bool,
    /// This fixed pump makes retail sales to ultimate buyers, rather than wholesale transfers
    pub retail_sales: bool,
    /// The pump displays a conspicuous and legible “UNDYED UNTAXED KEROSENE, NONTAXABLE USE ONLY” notice
    pub pump_notice_displayed: bool,
    /// The pump cannot fill a highway vehicle or train, or the vendor locks it after each sale
    pub pump_access: FederalBlockedPumpAccess,
    /// Qualifying kerosene sales from this blocked pump
    #[libertas_size(min = 1)]
    /// ----
    /// Each sale retains its own tax, use, quantity and no-duplicate-claim evidence.
    pub sales: Vec<FederalBlockedPumpKeroseneSale>,
    /// The vendor retained complete purchase, excise-tax, pump and sale records
    /// Locked-pump sales over five gallons also need the buyer's name and address.
    pub records_complete: bool,
}

impl FederalBlockedPumpKeroseneClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales.iter().try_fold(0_i64, |total, sale| {
            total.checked_add(crate::kernel::natural::thousandths(sale.gallons))
        })
    }

    pub(crate) fn line_cents(&self) -> i64 {
        self.total_thousandths()
            .map_or(0, |gallons| crate::kernel::ratio::fraction(gallons, 243, 10_000))
    }

    pub(crate) fn buyer_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let locked = self.pump_access == FederalBlockedPumpAccess::LockedBetweenSales;
        self.sales.iter().enumerate().find_map(|(index, sale)| {
            sale.buyer_issue(locked).map(|issue| issue.in_list("sales", index))
        })
    }

    pub(crate) fn valid(&self) -> bool {
        let registration = self.up_registration_number.trim();
        let locked = self.pump_access == FederalBlockedPumpAccess::LockedBetweenSales;
        registration.starts_with("UP")
            && registration.len() >= 3
            && !self.sales.is_empty()
            && self.sales.iter().all(|sale| sale.valid(locked))
            && self.total_thousandths().is_some()
            && self
                .sales
                .iter()
                .try_fold(0_i64, |total, sale| total.checked_add(sale.actual_fuel_cost))
                .is_some_and(|cost| cost >= self.line_cents())
    }

    pub(crate) fn qualified(&self) -> bool {
        self.registration_active
            && self.pump_fixed
            && self.retail_sales
            && self.pump_notice_displayed
            && self.pump_access != FederalBlockedPumpAccess::NotSure
            && self.records_complete
            && self.sales.iter().all(FederalBlockedPumpKeroseneSale::qualified)
    }
}

/// One Form 4136 fuel and use line for this business
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalFuelUse {
    /// Qualified fuel, including any fractional part
    /// Use the fuel purchase and equipment records, not vehicle odometer miles.
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual fuel cost from purchase records, including federal excise tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_cost: i64,
    /// The ultimate purchaser paid federal fuel tax and used these gallons for the stated qualifying activity
    /// Exclude personal use, commuting and registered highway vehicles. Form 4136 lines 3 and 4 require undyed fuel with no visible dye; a dye exception needs separate review. Retain invoices and equipment-use records. No leaves these gallons out of the fuel tax credit with a notice.
    /// Check fuel invoices, actual use records and previously filed fuel-tax claims.
    pub qualified: bool,
    /// This fuel tax was not claimed through Form 8849, Form 8864 or Form 720
    /// Your filed Forms 8849, 8864 and 720 and refund records show this. No leaves these gallons out of the fuel tax credit with a notice.
    /// Check fuel invoices, actual use records and previously filed fuel-tax claims.
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalFuelUse {}

/// Registered blender's Form 4136 line 15a credit
/// This credits undyed diesel used to produce the emulsion, not a purchaser's gallons of finished emulsion on line 14.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEmulsionBlenderClaim {
    /// Undyed diesel taxed at 24.4 cents and used in the emulsion, including any fractional part
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub diesel_gallons: f64,
    /// Actual cost of that diesel from business or partnership records
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_cost: i64,
    /// IRS Form 637 M registration number of the blender
    /// A partner enters the reporting partnership's registration number from its K-1 credit statement.
    pub m_registration_number: String,
    /// The blender's statement facts are confirmed
    /// Emulsion contains at least 14% water; its additive was registered by a U.S. manufacturer with the EPA under Clean Air Act section 211; undyed diesel taxed at 24.4 cents was used; and the blender produced and sold or used the emulsion in its trade or business. The blender's certification statement and the EPA additive registration show this. No leaves this blender claim out of the fuel tax credit with a notice.
    pub statement_facts_confirmed: bool,
    /// This diesel has not generated the same line 15a credit on Form 720 or 8849
    /// Your filed Forms 720 and 8849 show this. No leaves this blender claim out of the fuel tax credit with a notice.
    pub not_claimed_elsewhere: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalEmulsionBlenderClaim {}

impl FederalEmulsionBlenderClaim {
    pub(super) fn credit_cents(&self) -> i64 {
        crate::kernel::ratio::fraction(
            crate::kernel::natural::thousandths(self.diesel_gallons),
            46,
            10_000,
        )
    }

    pub(super) fn valid(&self) -> bool {
        (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.diesel_gallons))
            && (0..=100_000_000_000).contains(&self.actual_cost)
            && self.actual_cost >= self.credit_cents()
            && !self.m_registration_number.trim().is_empty()
    }

    pub(super) fn qualified(&self) -> bool {
        self.statement_facts_confirmed && self.not_claimed_elsewhere
    }
}

/// Source quantity for Form 4136 line 11 alternative fuel
/// LPG and CNG are credited in gasoline gallon equivalents (GGE); LNG uses diesel gallon equivalents (DGE). Select the invoice's equivalent unit when available, or its measured physical unit so the App converts using the published IRS factor.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalAlternativeFuelQuantity {
    /// Gasoline gallon equivalents printed on the fuel record
    Gge { thousandths: u32 },
    /// Diesel gallon equivalents printed on the fuel record
    Dge { thousandths: u32 },
    /// Measured liquid gallons on the fuel record
    LiquidGallons { thousandths: u32 },
    /// Measured pounds on the fuel record
    Pounds { thousandths: u32 },
    /// Measured cubic feet on the CNG fuel record
    CubicFeet { thousandths: u32 },
}

/// One tax-paid alternative-fuel purchase and qualifying business use
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAlternativeFuelUse {
    /// Quantity and unit shown in your purchase or dispenser records
    pub quantity: FederalAlternativeFuelQuantity,
    /// Actual fuel cost including federal excise tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_cost: i64,
    /// Your direct business or reporting partnership was the ultimate purchaser of taxed fuel and used it for the stated qualifying activity
    /// Do not include personal use, an unqualified highway vehicle, or another tax-exempt purchase. Enter off-highway and public-bus fuel in their separate fields. No leaves this fuel out of the fuel tax credit with a notice.
    /// Check fuel invoices, actual use records and previously filed fuel-tax claims.
    pub qualified: bool,
    /// No Form 8849, Form 8864 or Form 720 claim was made for this fuel
    /// Your filed Forms 8849, 8864 and 720 show this. No leaves this fuel out of the fuel tax credit with a notice.
    /// Check fuel invoices, actual use records and previously filed fuel-tax claims.
    pub not_claimed_elsewhere: bool,
}

#[derive(Clone, Copy)]
pub(super) enum AlternativeFuelKind {
    Lpg,
    Cng,
    Lng,
}

impl FederalAlternativeFuelUse {
    pub(super) fn line_cents(&self, kind: AlternativeFuelKind, rate: i64) -> Option<i64> {
        let (thousandths, denominator) = match (&self.quantity, kind) {
            (
                FederalAlternativeFuelQuantity::Gge { thousandths },
                AlternativeFuelKind::Lpg | AlternativeFuelKind::Cng,
            ) => (*thousandths, 10_000),
            (FederalAlternativeFuelQuantity::Dge { thousandths }, AlternativeFuelKind::Lng) => {
                (*thousandths, 10_000)
            }
            (
                FederalAlternativeFuelQuantity::LiquidGallons { thousandths },
                AlternativeFuelKind::Lpg,
            ) => (*thousandths, 13_530),
            (FederalAlternativeFuelQuantity::Pounds { thousandths }, AlternativeFuelKind::Lpg) => {
                (*thousandths, 57_500)
            }
            (FederalAlternativeFuelQuantity::Pounds { thousandths }, AlternativeFuelKind::Cng) => {
                (*thousandths, 56_600)
            }
            (
                FederalAlternativeFuelQuantity::CubicFeet { thousandths },
                AlternativeFuelKind::Cng,
            ) => (*thousandths, 1_235_700),
            (
                FederalAlternativeFuelQuantity::LiquidGallons { thousandths },
                AlternativeFuelKind::Lng,
            ) => (*thousandths, 17_100),
            (FederalAlternativeFuelQuantity::Pounds { thousandths }, AlternativeFuelKind::Lng) => {
                (*thousandths, 60_600)
            }
            _ => return None,
        };
        (1..=2_000_000_000)
            .contains(&thousandths)
            .then(|| crate::kernel::ratio::fraction(i64::from(thousandths), rate, denominator))
    }

    pub(super) fn valid(&self, kind: AlternativeFuelKind, rate: i64) -> bool {
        self.line_cents(kind, rate).is_some_and(|credit| {
            (0..=100_000_000_000).contains(&self.actual_cost) && self.actual_cost >= credit
        })
    }
}

/// Kerosene bought by the aircraft operator for commercial aviation other than foreign trade
/// The operator must retain the right to claim; vendor-waived, noncommercial, foreign-trade and mixed-use fuel need separate records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCommercialAviationKerosene {
    /// Kerosene originally taxed at 24.4 cents per gallon — Form 4136 line 5a
    /// The commercial-aviation difference is 20.0 cents per gallon.
    /// [NullText]
    /// No credit claimed for commercial-aviation kerosene taxed at 24.4 cents
    /// [SetText]
    /// Claim credit for commercial-aviation kerosene taxed at 24.4 cents using
    #[libertas_default($null)]
    pub taxed_at_244: Option<FederalFuelUse>,
    /// Kerosene originally taxed at 21.9 cents per gallon — Form 4136 line 5b
    /// The commercial-aviation difference is 17.5 cents per gallon.
    /// [NullText]
    /// No credit claimed for commercial-aviation kerosene taxed at 21.9 cents
    /// [SetText]
    /// Claim credit for commercial-aviation kerosene taxed at 21.9 cents using
    #[libertas_default($null)]
    pub taxed_at_219: Option<FederalFuelUse>,
    /// The aircraft operator did not waive either claim to a registered ultimate vendor
    /// Whether you gave a vendor a signed waiver (Model Waiver L) shows on your tax-excluded purchase invoices. No leaves these gallons out of the fuel tax credit with a notice.
    pub no_waiver: bool,
}

impl FederalCommercialAviationKerosene {
    pub(super) fn uses(&self) -> [Option<&FederalFuelUse>; 2] {
        [self.taxed_at_244.as_ref(), self.taxed_at_219.as_ref()]
    }

    pub(super) fn lines(&self) -> [i64; 2] {
        [
            FederalFuelLines::line_cents(self.taxed_at_244.as_ref(), 200),
            FederalFuelLines::line_cents(self.taxed_at_219.as_ref(), 175),
        ]
    }

    pub(super) fn valid(&self) -> bool {
        let uses = self.uses();
        uses.iter().any(Option::is_some)
            && uses.iter().zip(self.lines()).all(|(use_line, credit)| {
                use_line.is_none_or(|v| {
                    (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(v.gallons))
                        && (0..=100_000_000_000).contains(&v.actual_cost)
                        && v.actual_cost >= credit
                })
            })
    }

    pub(super) fn qualified(&self) -> bool {
        self.no_waiver
            && self.uses().iter().flatten().all(|v| v.qualified && v.not_claimed_elsewhere)
    }
}

/// Kerosene used by the ultimate purchaser in civil-aircraft foreign trade — Form 4136 lines 5c–5e
/// Lines 5c or 5d recover tax above the LUST rate, and line 5e recovers the remaining LUST tax on the same gallons. Do not enter these gallons on commercial-aviation lines 5a/5b.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForeignTradeAviationKerosene {
    /// Foreign-trade aviation kerosene originally taxed at 24.4 cents per gallon — line 5c plus 5e
    /// [NullText]
    /// No credit claimed for foreign-trade aviation kerosene taxed at 24.4 cents
    /// [SetText]
    /// Claim credit for foreign-trade aviation kerosene taxed at 24.4 cents using
    #[libertas_default($null)]
    pub taxed_at_244: Option<FederalFuelUse>,
    /// Foreign-trade aviation kerosene originally taxed at 21.9 cents per gallon — line 5d plus 5e
    /// [NullText]
    /// No credit claimed for foreign-trade aviation kerosene taxed at 21.9 cents
    /// [SetText]
    /// Claim credit for foreign-trade aviation kerosene taxed at 21.9 cents using
    #[libertas_default($null)]
    pub taxed_at_219: Option<FederalFuelUse>,
    /// The ultimate purchaser kept the claim right and did not waive it to a registered vendor
    /// Your signed waiver records and purchase invoices show this. No leaves these gallons out of the fuel tax credit with a notice.
    pub no_waiver: bool,
}

impl FederalForeignTradeAviationKerosene {
    pub(super) fn uses(&self) -> [Option<&FederalFuelUse>; 2] {
        [self.taxed_at_244.as_ref(), self.taxed_at_219.as_ref()]
    }

    pub(super) fn lines(&self) -> [i64; 3] {
        let gallons = self
            .uses()
            .into_iter()
            .flatten()
            .fold(0i64, |sum, value| sum + crate::kernel::natural::thousandths(value.gallons));
        [
            FederalFuelLines::line_cents(self.taxed_at_244.as_ref(), 243),
            FederalFuelLines::line_cents(self.taxed_at_219.as_ref(), 218),
            crate::kernel::ratio::fraction(gallons, 1, 10_000),
        ]
    }

    pub(super) fn valid(&self) -> bool {
        let uses = self.uses();
        uses.iter().any(Option::is_some)
            && uses.into_iter().zip([244, 219]).all(|(use_line, rate)| {
                use_line.is_none_or(|value| {
                    (1..=2_000_000_000)
                        .contains(&crate::kernel::natural::thousandths(value.gallons))
                        && (0..=100_000_000_000).contains(&value.actual_cost)
                        && value.actual_cost >= FederalFuelLines::line_cents(Some(value), rate)
                })
            })
            && self.uses().into_iter().flatten().map(|value| value.actual_cost).sum::<i64>()
                >= self.lines().into_iter().sum::<i64>()
    }

    pub(super) fn qualified(&self) -> bool {
        self.no_waiver
            && self
                .uses()
                .iter()
                .flatten()
                .all(|value| value.qualified && value.not_claimed_elsewhere)
    }
}

/// Tax-paid aviation kerosene used by an aerial applicator on a farm — Form 4136 lines 5c and 5d, type of use 1
/// The Schedule F farmer, Schedule C custom applicator, or reporting partnership must be the ultimate purchaser and keep its claim right. Unlike aviation gasoline, a purchaser can waive this kerosene claim to a registered vendor; waived gallons are excluded here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalFarmAviationKerosene {
    /// Fuel originally taxed at 24.4 cents used applying fertilizer, pesticides or other substances — line 5c
    /// [NullText]
    /// No credit claimed for farm aerial-application kerosene taxed at 24.4 cents
    /// [SetText]
    /// Claim credit for farm aerial-application kerosene taxed at 24.4 cents using
    #[libertas_default($null)]
    pub taxed_at_244_application: Option<FederalFuelUse>,
    /// Fuel originally taxed at 24.4 cents used on a direct airfield-to-farm flight — line 5c
    /// [NullText]
    /// No credit claimed for direct airfield-to-farm kerosene taxed at 24.4 cents
    /// [SetText]
    /// Claim credit for direct airfield-to-farm kerosene taxed at 24.4 cents using
    #[libertas_default($null)]
    pub taxed_at_244_direct_flight: Option<FederalFuelUse>,
    /// Fuel originally taxed at 21.9 cents used applying fertilizer, pesticides or other substances — line 5d
    /// [NullText]
    /// No credit claimed for farm aerial-application kerosene taxed at 21.9 cents
    /// [SetText]
    /// Claim credit for farm aerial-application kerosene taxed at 21.9 cents using
    #[libertas_default($null)]
    pub taxed_at_219_application: Option<FederalFuelUse>,
    /// Fuel originally taxed at 21.9 cents used on a direct airfield-to-farm flight — line 5d
    /// [NullText]
    /// No credit claimed for direct airfield-to-farm kerosene taxed at 21.9 cents
    /// [SetText]
    /// Claim credit for direct airfield-to-farm kerosene taxed at 21.9 cents using
    #[libertas_default($null)]
    pub taxed_at_219_direct_flight: Option<FederalFuelUse>,
    /// The aerial applicator was the ultimate purchaser of all entered fuel
    /// A farmer who hired another applicator cannot claim the applicator's fuel. The fuel purchase invoices show the buyer. No leaves these gallons out of the fuel tax credit with a notice.
    pub applicator_was_ultimate_purchaser: bool,
    /// The purchaser did not waive the claim to a registered ultimate vendor
    /// Your signed waiver records and invoices show this. No leaves these gallons out of the fuel tax credit with a notice.
    pub no_waiver: bool,
    /// Each rate and flight-use bucket contains different gallons, also distinct from every other credit or refund claim
    /// Your fuel log shows this. No leaves these gallons out of the fuel tax credit with a notice.
    pub distinct_gallons: bool,
}

impl FederalFarmAviationKerosene {
    pub(super) fn uses(&self) -> [Option<&FederalFuelUse>; 4] {
        [
            self.taxed_at_244_application.as_ref(),
            self.taxed_at_244_direct_flight.as_ref(),
            self.taxed_at_219_application.as_ref(),
            self.taxed_at_219_direct_flight.as_ref(),
        ]
    }

    pub(super) fn gallons_thousandths(&self, original_rate: i64) -> i64 {
        let uses = self.uses();
        let selected = if original_rate == 244 { &uses[..2] } else { &uses[2..] };
        selected
            .iter()
            .flatten()
            .map(|value| crate::kernel::natural::thousandths(value.gallons))
            .sum()
    }

    pub(super) fn cost_cents(&self) -> i64 {
        self.uses().into_iter().flatten().map(|value| value.actual_cost).sum()
    }

    pub(super) fn valid(&self) -> bool {
        let uses = self.uses();
        uses.iter().any(Option::is_some)
            && uses.into_iter().zip([243, 243, 218, 218]).all(|(fuel, rate)| {
                fuel.is_none_or(|value| {
                    (1..=2_000_000_000)
                        .contains(&crate::kernel::natural::thousandths(value.gallons))
                        && (0..=100_000_000_000).contains(&value.actual_cost)
                        && value.actual_cost >= FederalFuelLines::line_cents(Some(value), rate)
                })
            })
            && self.cost_cents()
                >= crate::kernel::ratio::fraction(self.gallons_thousandths(244), 243, 10_000)
                    + crate::kernel::ratio::fraction(self.gallons_thousandths(219), 218, 10_000)
    }

    pub(super) fn qualified(&self) -> bool {
        self.applicator_was_ultimate_purchaser
            && self.no_waiver
            && self.distinct_gallons
            && self
                .uses()
                .into_iter()
                .flatten()
                .all(|value| value.qualified && value.not_claimed_elsewhere)
    }
}

/// Treatment of the credited fuel tax in this business's recorded expenses
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalFuelExpenseTreatment {
    /// Full fuel cost, including the credited tax, was deducted here for this business's own fuel purchases or use
    /// An accrual-method business includes this credit in 2026 income. A cash-method business includes it in the year Form 4136 is filed, normally 2027 for this return. A separate card-issuer line 13 credit offsets its vendor settlement and is excluded from this purchase-cost recovery.
    FullCostDeducted,
    /// Credited tax was excluded from this business's expenses
    /// Do not enter the same credit again in gross receipts or another income item.
    TaxExcludedFromExpenses,
    /// A different or unresolved cost/basis treatment needs review
    Other,
}

/// Cash-method event that makes a fuel-tax credit or refund 2026 income
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalCashFuelRecoveryEvent {
    /// An original earlier-year return with Form 4136 was filed in 2026
    OriginalReturnFiled,
    /// A credit from an amended earlier-year return was received in 2026
    AmendedReturnCreditReceived,
    /// A Form 8849 or other fuel-tax refund was received in 2026
    RefundReceived,
}

/// Prior fuel-tax credit or refund included in a cash-method business's 2026 income
/// This is distinct from the Form 4136 payment claimed on the 2026 return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCashFuelRecovery {
    /// Credit or refund received under the selected event, in dollars and cents
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// What established 2026 cash-method income timing
    pub event: FederalCashFuelRecoveryEvent,
    /// The event actually occurred during 2026
    /// The date the prior return was filed, or the bank or IRS notice of the refund or credit, shows this. No leaves this recovery out of the estimate with a notice.
    pub occurred_in_year: bool,
    /// Full associated fuel cost including this tax was deducted and reduced income tax
    /// Choose Yes only if the entire recovery reduced prior Chapter 1 tax. For the narrow 2025 partial-benefit case below, choose No and enter its retained comparison worksheet. No without that worksheet includes the whole recovery as income with a notice.
    /// Check the retained prior return, deducted fuel costs and tax-benefit comparison worksheet.
    pub full_tax_benefit: bool,
    /// Documented partial 2025 regular-income-tax benefit from a deducted Schedule C fuel tax
    /// Choose the full-recovery answer when no partial-benefit comparison is supplied. This case uses the filed 2025 Form 1040 and a retained comparison, not a reconstructed historical return.
    /// [NullText]
    /// Use the full recovery; no partial-benefit comparison is supplied
    /// [SetText]
    /// The documented partial 2025 tax benefit is
    #[libertas_default($null)]
    pub partial_prior_year_benefit: Option<FederalPartialCashFuelTaxBenefit>,
    /// The associated fuel-cost deduction did not reduce net investment income tax
    /// If it did, the Form 8960 deduction-recovery worksheet must be completed separately. The prior-year Form 8960 shows it; a deduction appears there only for a nonactive activity. No includes the recovery for regular tax and leaves its net investment income recovery out with a notice.
    pub niit_recovery_excluded: bool,
    /// This amount is absent from recorded business receipts and other income
    /// Compare the business's books and entered receipts. No means it is already included, so it is not added again.
    pub not_reported_elsewhere: bool,
}

impl FederalCashFuelRecovery {
    pub(crate) fn valid(&self) -> bool {
        (1..=100_000_000_000).contains(&self.amount)
            && self
                .partial_prior_year_benefit
                .as_ref()
                .is_none_or(|partial| partial.valid(self.amount))
    }

    pub(crate) fn qualified(&self) -> bool {
        self.occurred_in_year
            && if let Some(partial) = &self.partial_prior_year_benefit {
                !self.full_tax_benefit && partial.qualified()
            } else {
                self.full_tax_benefit
            }
            && self.niit_recovery_excluded
            && self.not_reported_elsewhere
    }

    pub(crate) fn taxable_amount(&self) -> i64 {
        self.partial_prior_year_benefit
            .as_ref()
            .map_or(self.amount, |partial| partial.taxable_amount(self.amount))
    }

    pub(crate) fn excluded_amount(&self) -> i64 {
        self.amount - self.taxable_amount()
    }
}

/// Narrow section 111 exclusion for a partial 2025 active-business fuel-tax deduction recovery
/// Publication 525, Nonitemized Deduction Recoveries, excludes a prior deduction without an income-tax benefit. Treasury regulation 26 CFR 1.111-1(b)(2) measures that benefit by comparing taxable income with and without the recovered item; the retained refigure includes related half-SE and QBI changes. Original 2025 Form 1040 line 15 was zero, so the refigured positive taxable income is the tax-benefiting portion of this single item. See https://www.irs.gov/publications/p525, https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol2/pdf/CFR-2025-title26-vol2-sec1-111-1.pdf, and https://www.irs.gov/pub/irs-prior/f1040--2025.pdf.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPartialCashFuelTaxBenefit {
    /// Reference to the filed 2025 return, deducted-fuel-tax records and retained refigured-tax worksheet
    #[libertas_size(min = 1)]
    pub worksheet_reference: String,
    /// 2025 Form 1040 line 11b adjusted gross income as filed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_prior_year_line_11b: i64,
    /// 2025 Form 1040 line 14 total deductions as filed
    /// For this case, line 12e was the standard deduction and original lines 13a/13b were zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_prior_year_line_14: i64,
    /// Fuel excise tax included once in the original 2025 active-business deduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub originally_deducted_fuel_tax: i64,
    /// 2025 Form 1040 line 11b on the retained comparison without this recovered fuel-tax expense
    /// The supporting worksheet includes any change in the half-SE-tax deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub refigured_prior_year_line_11b: i64,
    /// 2025 Form 1040 line 14 on the retained comparison without this recovered fuel-tax expense
    /// The supporting worksheet includes any resulting QBI-deduction change.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub refigured_prior_year_line_14: i64,
    /// Refigured 2025 Form 1040 line 16 regular income tax after removing only the recovered expense
    /// The original line 16 was zero. This confirms a tax benefit; the amount included in 2026 income comes from the refigured taxable income, not from this tax amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub refigured_prior_year_line_16: i64,
    /// Original 2025 line 12e was the standard deduction, with lines 13a and 13b zero
    /// Itemized, QBI or Schedule 1-A amounts on the original return need a separate recovery worksheet. The original 2025 Form 1040 lines 12e, 13a and 13b show this. No includes the whole recovery as income with a notice.
    pub original_standard_only_deduction: bool,
    /// The retained original and refigured 2025 returns differ only in this fuel tax and its required half-SE/QBI calculations
    /// Confirm no other 2025 recovery, changing income, deduction, filing status, tax-rate election, preferential-rate income or dependent tax item affects the comparison. The retained refigured 2025 comparison worksheet shows this. No includes the whole recovery as income with a notice.
    pub comparison_complete: bool,
    /// The 2025 fuel deduction was currently allowed for a materially participating ordinary trade
    /// No passive, basis, at-risk or other deduction limit applied, and the recovered amount was not greater than the tax actually deducted. The 2025 business deduction records and any Form 8582 or 6198 show this. No includes the whole recovery as income with a notice.
    pub active_unlimited_deduction: bool,
    /// Neither the original nor the refigured 2025 comparison has alternative minimum tax or a distinct AMT benefit
    /// The 2025 Form 6251 and Schedule 2 line 1 on both returns show this. No includes the whole recovery as income with a notice.
    pub no_prior_amt: bool,
    /// No 2025 credit affecting Chapter 1 tax, operating loss, unused credit, QBI loss or other carryover changed because of this deduction
    /// The separate refundable Form 4136 payment remains on the 2025 return; it does not itself reduce line 16 income tax. Section 111 treats an unexpired increased carryover as a tax benefit; cases with such a carryover need separate calculation. Compare the credits and carryovers on the original and refigured 2025 returns. No includes the whole recovery as income with a notice.
    pub no_prior_credits_or_carryovers: bool,
}

impl FederalPartialCashFuelTaxBenefit {
    pub(crate) fn taxable_amount(&self, recovered: i64) -> i64 {
        // 26 CFR 1.111-1(b)(2) compares taxable income with and
        // without the recovered deduction. Original line 15 was zero;
        // the complete refigure includes half-SE and QBI changes.
        phase::bound(
            self.refigured_prior_year_line_11b - self.refigured_prior_year_line_14,
            0,
            recovered,
        )
    }

    pub(crate) fn valid(&self, recovered: i64) -> bool {
        !self.worksheet_reference.trim().is_empty()
            && (0..=100_000_000_000).contains(&self.original_prior_year_line_11b)
            && (0..=100_000_000_000).contains(&self.original_prior_year_line_14)
            && (1..=100_000_000_000).contains(&self.originally_deducted_fuel_tax)
            && (0..=100_000_000_000).contains(&self.refigured_prior_year_line_11b)
            && (0..=100_000_000_000).contains(&self.refigured_prior_year_line_14)
            && (1..=100_000_000_000).contains(&self.refigured_prior_year_line_16)
            && recovered <= self.originally_deducted_fuel_tax
            && (1..recovered)
                .contains(&(self.original_prior_year_line_14 - self.original_prior_year_line_11b))
            && self.refigured_prior_year_line_11b > self.refigured_prior_year_line_14
            && self.refigured_prior_year_line_11b >= self.original_prior_year_line_11b
            && self.refigured_prior_year_line_11b - self.original_prior_year_line_11b <= recovered
            && self.refigured_prior_year_line_14 >= self.original_prior_year_line_14
            && self.refigured_prior_year_line_14 - self.original_prior_year_line_14 <= recovered
            && self.refigured_prior_year_line_16
                <= self.refigured_prior_year_line_11b - self.refigured_prior_year_line_14
    }

    pub(crate) fn qualified(&self) -> bool {
        self.original_standard_only_deduction
            && self.comparison_complete
            && self.active_unlimited_deduction
            && self.no_prior_amt
            && self.no_prior_credits_or_carryovers
    }
}

/// Additional school-bus, public-bus and qualified local-bus Form 4136 uses
/// Keep each fuel and use's purchase cost separate. School-bus type 7, public intercity/local bus type 5 and qualified local-bus type 6 are distinct from off-highway type 2; confirm the stated bus service and tax-paid gallons in each selected use.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalBusFuelUses {
    /// Undyed diesel used in a qualifying school bus — line 3a, type of use 7
    /// [NullText]
    /// No credit claimed for undyed diesel used in a qualifying school bus
    /// [SetText]
    /// Claim credit for undyed diesel used in a qualifying school bus using
    #[libertas_default($null)]
    pub diesel_school_bus: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene used in a qualifying school bus — line 4a, type of use 7
    /// [NullText]
    /// No credit claimed for nonaviation kerosene used in a qualifying school bus
    /// [SetText]
    /// Claim credit for nonaviation kerosene used in a qualifying school bus using
    #[libertas_default($null)]
    pub kerosene_school_bus: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion used in a qualifying school bus — line 14a, type of use 7
    /// [NullText]
    /// No credit claimed for diesel-water emulsion used in a qualifying school bus
    /// [SetText]
    /// Claim credit for diesel-water emulsion used in a qualifying school bus using
    #[libertas_default($null)]
    pub emulsion_school_bus: Option<FederalFuelUse>,
    /// LPG used in a qualifying public bus — line 11a, type of use 5, 10.9 cents per GGE
    /// [NullText]
    /// No credit claimed for LPG used in a qualifying public bus
    /// [SetText]
    /// Claim credit for LPG used in a qualifying public bus using
    #[libertas_default($null)]
    pub lpg_public_bus: Option<FederalAlternativeFuelUse>,
    /// P Series fuel used in a qualifying public bus — line 11b, type of use 5, 11.0 cents per gallon
    /// [NullText]
    /// No credit claimed for P Series fuel used in a qualifying public bus
    /// [SetText]
    /// Claim credit for P Series fuel used in a qualifying public bus using
    #[libertas_default($null)]
    pub p_series_public_bus: Option<FederalFuelUse>,
    /// CNG used in a qualifying public bus — line 11c, type of use 5, 10.9 cents per GGE
    /// [NullText]
    /// No credit claimed for CNG used in a qualifying public bus
    /// [SetText]
    /// Claim credit for CNG used in a qualifying public bus using
    #[libertas_default($null)]
    pub cng_public_bus: Option<FederalAlternativeFuelUse>,
    /// Liquefied hydrogen used in a qualifying public bus — line 11d, type of use 5, 11.0 cents per gallon
    /// [NullText]
    /// No credit claimed for liquefied hydrogen used in a qualifying public bus
    /// [SetText]
    /// Claim credit for liquefied hydrogen used in a qualifying public bus using
    #[libertas_default($null)]
    pub hydrogen_public_bus: Option<FederalFuelUse>,
    /// Fischer-Tropsch coal- or peat-derived liquid used in a qualifying public bus — line 11e, type of use 5, 17.0 cents per gallon
    /// [NullText]
    /// No credit claimed for Fischer-Tropsch coal- or peat-derived fuel used in a qualifying public bus
    /// [SetText]
    /// Claim credit for Fischer-Tropsch coal- or peat-derived fuel used in a qualifying public bus using
    #[libertas_default($null)]
    pub fischer_tropsch_public_bus: Option<FederalFuelUse>,
    /// Biomass-derived liquid fuel used in a qualifying public bus — line 11f, type of use 5, 17.0 cents per gallon
    /// [NullText]
    /// No credit claimed for biomass-derived liquid fuel used in a qualifying public bus
    /// [SetText]
    /// Claim credit for biomass-derived liquid fuel used in a qualifying public bus using
    #[libertas_default($null)]
    pub biomass_liquid_public_bus: Option<FederalFuelUse>,
    /// LNG used in a qualifying public bus — line 11g, type of use 5, 16.9 cents per DGE
    /// [NullText]
    /// No credit claimed for LNG used in a qualifying public bus
    /// [SetText]
    /// Claim credit for LNG used in a qualifying public bus using
    #[libertas_default($null)]
    pub lng_public_bus: Option<FederalAlternativeFuelUse>,
    /// Liquefied biomass gas used in a qualifying public bus — line 11h, type of use 5, 11.0 cents per gallon
    /// [NullText]
    /// No credit claimed for liquefied biomass gas used in a qualifying public bus
    /// [SetText]
    /// Claim credit for liquefied biomass gas used in a qualifying public bus using
    #[libertas_default($null)]
    pub biomass_gas_public_bus: Option<FederalFuelUse>,
    /// Undyed diesel used in a qualified local bus — line 3a, type of use 6, 24.3 cents per gallon
    /// [NullText]
    /// No credit claimed for undyed diesel used in a qualified local bus
    /// [SetText]
    /// Claim credit for undyed diesel used in a qualified local bus using
    #[libertas_default($null)]
    pub diesel_local_bus: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene used in a qualified local bus — line 4a, type of use 6, 24.3 cents per gallon
    /// [NullText]
    /// No credit claimed for nonaviation kerosene used in a qualified local bus
    /// [SetText]
    /// Claim credit for nonaviation kerosene used in a qualified local bus using
    #[libertas_default($null)]
    pub kerosene_local_bus: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion used in a qualified local bus — line 14a, type of use 6, 19.7 cents per gallon
    /// [NullText]
    /// No credit claimed for diesel-water emulsion used in a qualified local bus
    /// [SetText]
    /// Claim credit for diesel-water emulsion used in a qualified local bus using
    #[libertas_default($null)]
    pub emulsion_local_bus: Option<FederalFuelUse>,
    /// The local-bus service qualifies for type of use 6
    /// It is compensated intracity service open to the public on scheduled regular routes, with at least 20 adult seats excluding the driver, under a bona fide state/local government contract or more-than-nominal subsidy. Confirm when entering any type-of-use 6 fuel. No leaves the type-of-use 6 fuel out of the fuel tax credit with a notice.
    /// Check the service contract, route and seat records, and subsidy statement.
    /// [NullText]
    /// No qualified-local-bus fuel credit claimed
    /// [SetText]
    /// For the claimed local-bus fuel, the service qualification is
    pub local_bus_qualified: Option<bool>,
    /// LPG used in a qualified local bus — line 11a, type of use 6, 18.3 cents per GGE
    /// [NullText]
    /// No credit claimed for LPG used in a qualified local bus
    /// [SetText]
    /// Claim credit for LPG used in a qualified local bus using
    #[libertas_default($null)]
    pub lpg_local_bus: Option<FederalAlternativeFuelUse>,
    /// P Series fuel used in a qualified local bus — line 11b, type of use 6, 18.3 cents per gallon
    /// [NullText]
    /// No credit claimed for P Series fuel used in a qualified local bus
    /// [SetText]
    /// Claim credit for P Series fuel used in a qualified local bus using
    #[libertas_default($null)]
    pub p_series_local_bus: Option<FederalFuelUse>,
    /// CNG used in a qualified local bus — line 11c, type of use 6, 18.3 cents per GGE
    /// [NullText]
    /// No credit claimed for CNG used in a qualified local bus
    /// [SetText]
    /// Claim credit for CNG used in a qualified local bus using
    #[libertas_default($null)]
    pub cng_local_bus: Option<FederalAlternativeFuelUse>,
    /// Liquefied hydrogen used in a qualified local bus — line 11d, type of use 6, 18.3 cents per gallon
    /// [NullText]
    /// No credit claimed for liquefied hydrogen used in a qualified local bus
    /// [SetText]
    /// Claim credit for liquefied hydrogen used in a qualified local bus using
    #[libertas_default($null)]
    pub hydrogen_local_bus: Option<FederalFuelUse>,
    /// Fischer-Tropsch coal- or peat-derived liquid used in a qualified local bus — line 11e, type of use 6, 24.3 cents per gallon
    /// [NullText]
    /// No credit claimed for Fischer-Tropsch coal- or peat-derived fuel used in a qualified local bus
    /// [SetText]
    /// Claim credit for Fischer-Tropsch coal- or peat-derived fuel used in a qualified local bus using
    #[libertas_default($null)]
    pub fischer_tropsch_local_bus: Option<FederalFuelUse>,
    /// Biomass-derived liquid fuel used in a qualified local bus — line 11f, type of use 6, 24.3 cents per gallon
    /// [NullText]
    /// No credit claimed for biomass-derived liquid fuel used in a qualified local bus
    /// [SetText]
    /// Claim credit for biomass-derived liquid fuel used in a qualified local bus using
    #[libertas_default($null)]
    pub biomass_liquid_local_bus: Option<FederalFuelUse>,
    /// LNG used in a qualified local bus — line 11g, type of use 6, 24.3 cents per DGE
    /// [NullText]
    /// No credit claimed for LNG used in a qualified local bus
    /// [SetText]
    /// Claim credit for LNG used in a qualified local bus using
    #[libertas_default($null)]
    pub lng_local_bus: Option<FederalAlternativeFuelUse>,
    /// Liquefied biomass gas used in a qualified local bus — line 11h, type of use 6, 18.3 cents per gallon
    /// [NullText]
    /// No credit claimed for liquefied biomass gas used in a qualified local bus
    /// [SetText]
    /// Claim credit for liquefied biomass gas used in a qualified local bus using
    #[libertas_default($null)]
    pub biomass_gas_local_bus: Option<FederalFuelUse>,
}

impl FederalBusFuelUses {
    pub(super) fn priced_uses(&self) -> [(Option<&FederalFuelUse>, i64); 16] {
        [
            (self.diesel_school_bus.as_ref(), 243),
            (self.kerosene_school_bus.as_ref(), 243),
            (self.emulsion_school_bus.as_ref(), 197),
            (self.p_series_public_bus.as_ref(), 110),
            (self.hydrogen_public_bus.as_ref(), 110),
            (self.fischer_tropsch_public_bus.as_ref(), 170),
            (self.biomass_liquid_public_bus.as_ref(), 170),
            (self.biomass_gas_public_bus.as_ref(), 110),
            (self.diesel_local_bus.as_ref(), 243),
            (self.kerosene_local_bus.as_ref(), 243),
            (self.emulsion_local_bus.as_ref(), 197),
            (self.p_series_local_bus.as_ref(), 183),
            (self.hydrogen_local_bus.as_ref(), 183),
            (self.fischer_tropsch_local_bus.as_ref(), 243),
            (self.biomass_liquid_local_bus.as_ref(), 243),
            (self.biomass_gas_local_bus.as_ref(), 183),
        ]
    }

    pub(super) fn alternative_uses(
        &self,
    ) -> [(Option<&FederalAlternativeFuelUse>, AlternativeFuelKind, i64); 6] {
        [
            (self.lpg_public_bus.as_ref(), AlternativeFuelKind::Lpg, 109),
            (self.cng_public_bus.as_ref(), AlternativeFuelKind::Cng, 109),
            (self.lng_public_bus.as_ref(), AlternativeFuelKind::Lng, 169),
            (self.lpg_local_bus.as_ref(), AlternativeFuelKind::Lpg, 183),
            (self.cng_local_bus.as_ref(), AlternativeFuelKind::Cng, 183),
            (self.lng_local_bus.as_ref(), AlternativeFuelKind::Lng, 243),
        ]
    }

    pub(super) fn lines(&self) -> [i64; 22] {
        let priced =
            self.priced_uses().map(|(use_line, rate)| FederalFuelLines::line_cents(use_line, rate));
        [
            priced[0],
            priced[1],
            self.lpg_public_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lpg, 109))
                .unwrap_or(0),
            priced[3],
            self.cng_public_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Cng, 109))
                .unwrap_or(0),
            priced[4],
            priced[5],
            priced[6],
            self.lng_public_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lng, 169))
                .unwrap_or(0),
            priced[7],
            priced[2],
            priced[8],
            priced[9],
            priced[10],
            self.lpg_local_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lpg, 183))
                .unwrap_or(0),
            priced[11],
            self.cng_local_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Cng, 183))
                .unwrap_or(0),
            priced[12],
            priced[13],
            priced[14],
            self.lng_local_bus
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lng, 243))
                .unwrap_or(0),
            priced[15],
        ]
    }

    pub(super) fn valid(&self) -> bool {
        (self.priced_uses().iter().any(|(value, _)| value.is_some())
            || self.alternative_uses().iter().any(|(value, _, _)| value.is_some()))
            && self.priced_uses().into_iter().all(|(use_line, rate)| {
                use_line.is_none_or(|value| {
                    (1..=2_000_000_000)
                        .contains(&crate::kernel::natural::thousandths(value.gallons))
                        && (0..=100_000_000_000).contains(&value.actual_cost)
                        && value.actual_cost >= FederalFuelLines::line_cents(use_line, rate)
                })
            })
            && self
                .alternative_uses()
                .into_iter()
                .all(|(use_line, kind, rate)| use_line.is_none_or(|value| value.valid(kind, rate)))
    }

    pub(super) fn qualified(&self) -> bool {
        ((!self.has_local_bus_use()) || self.local_bus_qualified == Some(true))
            && self
                .priced_uses()
                .iter()
                .filter_map(|(value, _)| *value)
                .all(|value| value.qualified && value.not_claimed_elsewhere)
            && self
                .alternative_uses()
                .iter()
                .filter_map(|(value, _, _)| *value)
                .all(|value| value.qualified && value.not_claimed_elsewhere)
    }

    pub(super) fn has_local_bus_use(&self) -> bool {
        self.diesel_local_bus.is_some()
            || self.kerosene_local_bus.is_some()
            || self.emulsion_local_bus.is_some()
            || self.lpg_local_bus.is_some()
            || self.p_series_local_bus.is_some()
            || self.cng_local_bus.is_some()
            || self.hydrogen_local_bus.is_some()
            || self.fischer_tropsch_local_bus.is_some()
            || self.biomass_liquid_local_bus.is_some()
            || self.lng_local_bus.is_some()
            || self.biomass_gas_local_bus.is_some()
    }
}

/// Other nontaxable business use of diesel, nonaviation kerosene or diesel-water emulsion — type of use 8
/// This includes qualifying business heating/process fuel and motorboat fuel, not fuel used to propel a train or diesel-powered highway vehicle. Personal fuel needs a separate exception.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalOtherFuelUses {
    /// Undyed diesel used in a qualifying business heater, process or motorboat — line 3a
    /// [NullText]
    /// No credit claimed for diesel used in a business heater, process or motorboat
    /// [SetText]
    /// Claim credit for diesel used in a business heater, process or motorboat using
    #[libertas_default($null)]
    pub diesel_type_8: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene used in a qualifying business heater, process or motorboat — line 4a
    /// [NullText]
    /// No credit claimed for nonaviation kerosene used in a business heater, process or motorboat
    /// [SetText]
    /// Claim credit for nonaviation kerosene used in a business heater, process or motorboat using
    #[libertas_default($null)]
    pub kerosene_type_8: Option<FederalFuelUse>,
    /// Tax-paid diesel-water emulsion used in a qualifying business heater, process or motorboat — line 14a
    /// [NullText]
    /// No credit claimed for diesel-water emulsion used in a business heater, process or motorboat
    /// [SetText]
    /// Claim credit for diesel-water emulsion used in a business heater, process or motorboat using
    #[libertas_default($null)]
    pub emulsion_type_8: Option<FederalFuelUse>,
}

impl FederalOtherFuelUses {
    pub(super) fn priced_uses(&self) -> [(Option<&FederalFuelUse>, i64); 3] {
        [
            (self.diesel_type_8.as_ref(), 243),
            (self.kerosene_type_8.as_ref(), 243),
            (self.emulsion_type_8.as_ref(), 197),
        ]
    }

    pub(super) fn lines(&self) -> [i64; 3] {
        self.priced_uses().map(|(value, rate)| FederalFuelLines::line_cents(value, rate))
    }

    pub(super) fn valid(&self) -> bool {
        self.priced_uses().iter().any(|(value, _)| value.is_some())
            && self.priced_uses().into_iter().all(|(value, rate)| {
                value.is_none_or(|use_line| {
                    (1..=2_000_000_000)
                        .contains(&crate::kernel::natural::thousandths(use_line.gallons))
                        && (0..=100_000_000_000).contains(&use_line.actual_cost)
                        && use_line.actual_cost >= FederalFuelLines::line_cents(value, rate)
                })
            })
    }

    pub(super) fn qualified(&self) -> bool {
        self.priced_uses()
            .into_iter()
            .filter_map(|(value, _)| value)
            .all(|value| value.qualified && value.not_claimed_elsewhere)
    }
}

/// Aviation gasoline used in certain helicopter and fixed-wing aircraft — Form 4136 line 2b, type of use 10
/// The ultimate purchaser claims 19.3 cents per gallon. Keep these gallons and purchase costs separate from commercial aviation on line 2a and every other fuel use. No vendor waiver or certificate transfers this aviation-gasoline claim.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAviationGasolineType10Uses {
    /// Helicopter transporting people, equipment or supplies to explore, develop or remove hard minerals, oil or gas
    /// For mineral work, apply the airport/service condition to each flight segment.
    /// [NullText]
    /// No credit claimed for aviation gasoline for helicopter mineral transport
    /// [SetText]
    /// Claim credit for aviation gasoline for helicopter mineral transport using
    #[libertas_default($null)]
    pub helicopter_mineral_transport: Option<FederalFuelUse>,
    /// Helicopter planting, cultivating, cutting, transporting or caring for trees, including logging
    /// [NullText]
    /// No credit claimed for aviation gasoline for helicopter tree work
    /// [SetText]
    /// Claim credit for aviation gasoline for helicopter tree work using
    #[libertas_default($null)]
    pub helicopter_tree_work: Option<FederalFuelUse>,
    /// Helicopter providing emergency medical transportation
    /// [NullText]
    /// No credit claimed for aviation gasoline for helicopter emergency medical transport
    /// [SetText]
    /// Claim credit for aviation gasoline for helicopter emergency medical transport using
    #[libertas_default($null)]
    pub helicopter_emergency_medical: Option<FederalFuelUse>,
    /// Fixed-wing aircraft planting, cultivating, cutting, transporting or caring for trees, including logging
    /// [NullText]
    /// No credit claimed for aviation gasoline for fixed-wing tree work
    /// [SetText]
    /// Claim credit for aviation gasoline for fixed-wing tree work using
    #[libertas_default($null)]
    pub fixed_wing_tree_work: Option<FederalFuelUse>,
    /// Fixed-wing aircraft providing emergency medical transportation
    /// [NullText]
    /// No credit claimed for aviation gasoline for fixed-wing emergency medical transport
    /// [SetText]
    /// Claim credit for aviation gasoline for fixed-wing emergency medical transport using
    #[libertas_default($null)]
    pub fixed_wing_emergency_medical: Option<FederalFuelUse>,
    /// Mineral and tree-work flights avoided federally assisted airports and covered airport services
    /// Confirm that every selected mineral or tree-work flight neither took off from nor landed at an eligible airport and did not use the specified federal airport services. This condition does not apply to emergency-medical flights. The flight logs of take-off and landing airports show this. No leaves these gallons out of the fuel tax credit with a notice.
    /// [NullText]
    /// No mineral-transport or tree-work aviation fuel credit claimed
    /// [SetText]
    /// For the claimed mineral or tree-work flights, the airport condition is
    #[libertas_default($null)]
    pub mineral_and_tree_airport_condition: Option<bool>,
    /// Every fixed-wing emergency-medical flight used an aircraft equipped and exclusively dedicated to acute-care emergency medical services
    /// The operator's aircraft equipment and dedication records show this. No leaves these gallons out of the fuel tax credit with a notice.
    /// [NullText]
    /// No fixed-wing emergency-medical aviation fuel credit claimed
    /// [SetText]
    /// For the claimed fixed-wing medical flights, acute-care equipment and dedication are confirmed
    #[libertas_default($null)]
    pub fixed_wing_acute_care_condition: Option<bool>,
    /// These type-10 gallons are distinct from one another and from all other Form 4136 and refund claims
    /// Your fuel log shows this. No leaves these gallons out of the fuel tax credit with a notice.
    pub distinct_gallons: bool,
}

impl FederalAviationGasolineType10Uses {
    pub(super) fn uses(&self) -> [Option<&FederalFuelUse>; 5] {
        [
            self.helicopter_mineral_transport.as_ref(),
            self.helicopter_tree_work.as_ref(),
            self.helicopter_emergency_medical.as_ref(),
            self.fixed_wing_tree_work.as_ref(),
            self.fixed_wing_emergency_medical.as_ref(),
        ]
    }

    pub(super) fn gallons_thousandths(&self) -> i64 {
        self.uses()
            .into_iter()
            .flatten()
            .map(|value| crate::kernel::natural::thousandths(value.gallons))
            .sum::<i64>()
    }

    pub(super) fn line_cents(&self) -> i64 {
        crate::kernel::ratio::fraction(self.gallons_thousandths(), 193, 10_000)
    }

    pub(super) fn valid(&self) -> bool {
        self.uses().iter().any(Option::is_some)
            && self.uses().into_iter().flatten().all(|value| {
                (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(value.gallons))
                    && (0..=100_000_000_000).contains(&value.actual_cost)
                    && value.actual_cost >= FederalFuelLines::line_cents(Some(value), 193)
            })
            && self.uses().into_iter().flatten().map(|value| value.actual_cost).sum::<i64>()
                >= self.line_cents()
    }

    pub(super) fn qualified(&self) -> bool {
        self.distinct_gallons
            && (self.helicopter_mineral_transport.is_none()
                && self.helicopter_tree_work.is_none()
                && self.fixed_wing_tree_work.is_none()
                || self.mineral_and_tree_airport_condition == Some(true))
            && (self.fixed_wing_emergency_medical.is_none()
                || self.fixed_wing_acute_care_condition == Some(true))
            && self
                .uses()
                .into_iter()
                .flatten()
                .all(|value| value.qualified && value.not_claimed_elsewhere)
    }
}

/// Aviation gasoline used by an aerial applicator on a farm for farming purposes — Form 4136 line 2b, type of use 1
/// A Schedule F farmer, Schedule C custom applicator, or reporting partnership may claim only when it was both the aerial applicator and ultimate purchaser. A farmer who merely hired another applicator does not claim that applicator's fuel. No aircraft-ownership condition or kerosene vendor waiver applies.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAviationGasolineFarmUse {
    /// Tax-paid aviation gasoline consumed applying fertilizer, pesticides or other substances over the farm
    /// [NullText]
    /// No credit claimed for aviation gasoline for aerial farm application
    /// [SetText]
    /// Claim credit for aviation gasoline for aerial farm application using
    #[libertas_default($null)]
    pub aerial_application: Option<FederalFuelUse>,
    /// Tax-paid aviation gasoline consumed on direct flight between the airfield and one or more serviced farms
    /// Exclude unrelated repositioning, transport or other off-farm flights.
    /// [NullText]
    /// No credit claimed for aviation gasoline for direct airfield-to-farm flights
    /// [SetText]
    /// Claim credit for aviation gasoline for direct airfield-to-farm flights using
    #[libertas_default($null)]
    pub direct_airfield_farm_flight: Option<FederalFuelUse>,
    /// The claiming Schedule F farmer, Schedule C custom applicator, or K-1 reporting partnership both bought the fuel and performed the aerial application
    /// The fuel invoices and application records show this. No leaves these gallons out of the fuel tax credit with a notice.
    pub applicator_was_ultimate_purchaser: bool,
    /// Application and direct-flight gallons are distinct from each other and every other Form 4136 or refund claim
    /// Your fuel log shows this. No leaves these gallons out of the fuel tax credit with a notice.
    pub distinct_gallons: bool,
}

impl FederalAviationGasolineFarmUse {
    pub(super) fn uses(&self) -> [Option<&FederalFuelUse>; 2] {
        [self.aerial_application.as_ref(), self.direct_airfield_farm_flight.as_ref()]
    }

    pub(super) fn gallons_thousandths(&self) -> i64 {
        self.uses()
            .into_iter()
            .flatten()
            .map(|value| crate::kernel::natural::thousandths(value.gallons))
            .sum()
    }

    pub(super) fn cost_cents(&self) -> i64 {
        self.uses().into_iter().flatten().map(|value| value.actual_cost).sum()
    }

    pub(super) fn valid(&self) -> bool {
        self.uses().iter().any(Option::is_some)
            && self.uses().into_iter().flatten().all(|value| {
                (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(value.gallons))
                    && (0..=100_000_000_000).contains(&value.actual_cost)
                    && value.actual_cost >= FederalFuelLines::line_cents(Some(value), 193)
            })
            && self.cost_cents()
                >= crate::kernel::ratio::fraction(self.gallons_thousandths(), 193, 10_000)
    }

    pub(super) fn qualified(&self) -> bool {
        self.applicator_was_ultimate_purchaser
            && self.distinct_gallons
            && self
                .uses()
                .into_iter()
                .flatten()
                .all(|value| value.qualified && value.not_claimed_elsewhere)
    }
}

/// Registration of a civil aircraft used in foreign trade
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalForeignTradeAircraftRegistration {
    /// Registered in the United States
    UnitedStates,
    /// Registered in another country
    Foreign {
        /// That country allows reciprocal foreign-trade privileges for aircraft registered in the United States
        /// No leaves this gasoline out of the fuel tax credit with a notice.
        /// Check the aircraft registration and the applicable foreign-government reciprocity records.
        reciprocal_privileges: bool,
    },
}

/// Tax-paid aviation gasoline used in civil-aircraft foreign trade — Form 4136 lines 2b and 2d, type of use 9
/// The same qualifying gallons recover 19.3 cents per gallon on line 2b and 0.1 cent per gallon of LUST tax on line 2d. Do not also enter them as commercial aviation, export, or another nontaxable use.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAviationGasolineForeignTrade {
    /// One set of taxed gallons, purchase cost and ultimate-purchaser records for both claim lines
    pub fuel: FederalFuelUse,
    /// The civil aircraft carried people or property for hire in foreign trade or trade with a U.S. territory, or made necessary preparations for that transportation
    /// Your flight operations and charter records show this. No leaves these gallons out of the fuel tax credit with a notice.
    pub qualifying_civil_aircraft_trade: bool,
    /// Country where the civil aircraft is registered
    pub registration: FederalForeignTradeAircraftRegistration,
    /// These gallons are distinct from every other Form 4136 line and any earlier refund or credit claim
    /// Your fuel log shows this. No leaves these gallons out of the fuel tax credit with a notice.
    pub distinct_gallons: bool,
}

impl FederalAviationGasolineForeignTrade {
    pub(super) fn valid(&self) -> bool {
        (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.fuel.gallons))
            && (0..=100_000_000_000).contains(&self.fuel.actual_cost)
            && self.fuel.actual_cost
                >= FederalFuelLines::line_cents(Some(&self.fuel), 193)
                    + FederalFuelLines::line_cents(Some(&self.fuel), 1)
    }

    pub(super) fn qualified(&self) -> bool {
        self.fuel.qualified
            && self.fuel.not_claimed_elsewhere
            && self.qualifying_civil_aircraft_trade
            && self.distinct_gallons
            && match self.registration {
                FederalForeignTradeAircraftRegistration::UnitedStates => true,
                FederalForeignTradeAircraftRegistration::Foreign { reciprocal_privileges } => {
                    reciprocal_privileges
                }
            }
    }
}

/// Supported Form 4136 fuel and use lines
/// A direct business enters its own annual quantities; a partner enters their distributive share from the K-1 box 20 code C statement. Other fuel uses, vendors and card issuers need separate records. Do not enter the same fuel twice.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalFuelLines {
    /// Gasoline for off-highway business use — Form 4136 line 1a, type of use 2
    /// [NullText]
    /// No credit claimed for off-highway business gasoline
    /// [SetText]
    /// Claim credit for off-highway business gasoline using
    #[libertas_default($null)]
    pub gasoline_off_highway: Option<FederalFuelUse>,
    /// Gasoline on a farm for farming purposes — line 1b, type of use 1
    /// [NullText]
    /// No credit claimed for farm-use gasoline
    /// [SetText]
    /// Claim credit for farm-use gasoline using
    #[libertas_default($null)]
    pub gasoline_farm: Option<FederalFuelUse>,
    /// Tax-paid gasoline used in a boat engaged in commercial fishing — line 1c, type of use 4
    /// This is a business fishing boat, not personal boating or another motorboat use. The ultimate purchaser claims 18.3 cents per gallon.
    /// [NullText]
    /// No credit claimed for commercial-fishing boat gasoline
    /// [SetText]
    /// Claim credit for commercial-fishing boat gasoline using
    #[libertas_default($null)]
    pub gasoline_commercial_fishing: Option<FederalFuelUse>,
    /// Tax-paid gasoline used in a bus transporting students or school employees — line 1c, type of use 7
    /// This is a qualifying school bus, not a personal vehicle or ordinary commute. The ultimate purchaser claims 18.3 cents per gallon; use a separate line 1c statement when also claiming another use type.
    /// [NullText]
    /// No credit claimed for school-bus gasoline
    /// [SetText]
    /// Claim credit for school-bus gasoline using
    #[libertas_default($null)]
    pub gasoline_school_bus: Option<FederalFuelUse>,
    /// Tax-paid gasoline used in a qualifying public intercity or local bus — line 1c, type of use 5
    /// The bus transports paying passengers on scheduled regular routes, or has at least 20 adult seats for nonscheduled service, and is available to the general public. Exclude taxis, van pools and school-bus gallons. The ultimate purchaser claims 18.3 cents per gallon.
    /// [NullText]
    /// No credit claimed for public intercity or local-bus gasoline
    /// [SetText]
    /// Claim credit for public intercity or local-bus gasoline using
    #[libertas_default($null)]
    pub gasoline_intercity_local_bus: Option<FederalFuelUse>,
    /// Tax-paid gasoline exported from the United States — line 1d, type of use 3
    /// The ultimate purchaser claims 18.4 cents per gallon; fuel merely in a vehicle's tank is not an export. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported gasoline
    /// [SetText]
    /// Claim credit for exported gasoline using
    #[libertas_default($null)]
    pub gasoline_export: Option<FederalFuelUse>,
    /// Tax-paid aviation gasoline used in commercial aviation other than foreign trade — line 2a
    /// The ultimate purchaser claims 15.0 cents per gallon. Other aviation uses and exports need separate records.
    /// [NullText]
    /// No credit claimed for commercial-aviation gasoline
    /// [SetText]
    /// Claim credit for commercial-aviation gasoline using
    #[libertas_default($null)]
    pub aviation_gasoline_commercial: Option<FederalFuelUse>,
    /// Tax-paid aviation gasoline used off highway in this business, outside an aircraft's propulsion system — line 2b, type of use 2
    /// The ultimate purchaser claims 19.3 cents per gallon. Do not include these gallons on line 2a.
    /// [NullText]
    /// No credit claimed for off-highway aviation gasoline used outside aircraft propulsion
    /// [SetText]
    /// Claim credit for off-highway aviation gasoline used outside aircraft propulsion using
    #[libertas_default($null)]
    pub aviation_gasoline_nonpropulsion_off_highway: Option<FederalFuelUse>,
    /// Tax-paid aviation gasoline used by this business as the aerial applicator — line 2b, type of use 1
    /// A farm operator or custom application service may claim its own fuel, including direct airfield-to-farm flights.
    /// [NullText]
    /// No credit claimed for aerial-applicator farm-use aviation gasoline
    /// [SetText]
    /// Claim credit for aerial-applicator farm-use aviation gasoline using
    #[libertas_default($null)]
    pub aviation_gasoline_farm: Option<FederalAviationGasolineFarmUse>,
    /// Tax-paid aviation gasoline in qualifying helicopter or fixed-wing aircraft — line 2b, type of use 10
    /// [NullText]
    /// No credit claimed for special helicopter or fixed-wing aviation gasoline
    /// [SetText]
    /// Claim credit for special helicopter or fixed-wing aviation gasoline using
    #[libertas_default($null)]
    pub aviation_gasoline_type_10: Option<FederalAviationGasolineType10Uses>,
    /// Tax-paid aviation gasoline exported from the United States — line 2c, type of use 3
    /// The ultimate purchaser claims 19.4 cents per gallon. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported aviation gasoline
    /// [SetText]
    /// Claim credit for exported aviation gasoline using
    #[libertas_default($null)]
    pub aviation_gasoline_export: Option<FederalFuelUse>,
    /// Tax-paid aviation gasoline used in qualifying civil-aircraft foreign trade — lines 2b and 2d, type of use 9
    /// [NullText]
    /// No credit claimed for civil-aircraft foreign-trade aviation gasoline
    /// [SetText]
    /// Claim credit for civil-aircraft foreign-trade aviation gasoline using
    #[libertas_default($null)]
    pub aviation_gasoline_foreign_trade: Option<FederalAviationGasolineForeignTrade>,
    /// Undyed diesel for off-highway business use — line 3a, type of use 2
    /// [NullText]
    /// No credit claimed for off-highway business diesel
    /// [SetText]
    /// Claim credit for off-highway business diesel using
    #[libertas_default($null)]
    pub diesel_off_highway: Option<FederalFuelUse>,
    /// Undyed diesel on a farm for farming purposes — line 3b, type of use 1
    /// [NullText]
    /// No credit claimed for farm-use diesel
    /// [SetText]
    /// Claim credit for farm-use diesel using
    #[libertas_default($null)]
    pub diesel_farm: Option<FederalFuelUse>,
    /// Undyed diesel used to propel a business train — line 3c
    /// Keep this separate from line 3a off-highway equipment fuel. Confirm the tax was included in the price paid for these train gallons.
    /// [NullText]
    /// No credit claimed for business train diesel
    /// [SetText]
    /// Claim credit for business train diesel using
    #[libertas_default($null)]
    pub diesel_trains: Option<FederalFuelUse>,
    /// Undyed diesel used in a qualifying public intercity or local bus — line 3d, type of use 5
    /// Paid public service must use scheduled routes or at least 20 adult seats for nonscheduled service. The reduced credit is 17.0 cents per gallon; do not enter these gallons as off-highway or school-bus use. Confirm no waiver below.
    /// [NullText]
    /// No credit claimed for public intercity or local-bus diesel
    /// [SetText]
    /// Claim credit for public intercity or local-bus diesel using
    #[libertas_default($null)]
    pub diesel_intercity_local_bus: Option<FederalFuelUse>,
    /// Tax-paid undyed diesel exported from the United States — line 3e, type of use 3
    /// The ultimate purchaser claims 24.4 cents per gallon. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported undyed diesel
    /// [SetText]
    /// Claim credit for exported undyed diesel using
    #[libertas_default($null)]
    pub diesel_export: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene taxed at 24.4 cents per gallon, for off-highway business use — line 4a, type of use 2
    /// [NullText]
    /// No credit claimed for off-highway nonaviation kerosene taxed at 24.4 cents
    /// [SetText]
    /// Claim credit for off-highway nonaviation kerosene taxed at 24.4 cents using
    #[libertas_default($null)]
    pub kerosene_off_highway: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene on a farm for farming purposes — line 4b, type of use 1
    /// Kerosene used in aviation needs its own Form 4136 line 5 worksheet.
    /// [NullText]
    /// No credit claimed for farm-use nonaviation kerosene
    /// [SetText]
    /// Claim credit for farm-use nonaviation kerosene using
    #[libertas_default($null)]
    pub kerosene_farm: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene used in a qualifying public intercity or local bus — line 4c, type of use 5
    /// Paid public service must use scheduled routes or at least 20 adult seats for nonscheduled service. The reduced credit is 17.0 cents per gallon; do not enter these gallons on another kerosene line. Confirm no waiver below.
    /// [NullText]
    /// No credit claimed for public intercity or local-bus nonaviation kerosene
    /// [SetText]
    /// Claim credit for public intercity or local-bus nonaviation kerosene using
    #[libertas_default($null)]
    pub kerosene_intercity_local_bus: Option<FederalFuelUse>,
    /// The ultimate purchaser did not waive the line 3d or 4c claim to a registered vendor
    /// Confirm this when either reduced-rate bus fuel is claimed; it is not needed for the gasoline line 1c claim. Your signed waiver records and invoices show this. No leaves these gallons out of the fuel tax credit with a notice.
    /// [NullText]
    /// No reduced-rate diesel or kerosene public-bus credit claimed
    /// [SetText]
    /// For the claimed public-bus fuel, the purchaser retained the right to claim
    #[libertas_default($null)]
    pub intercity_bus_no_waiver: Option<bool>,
    /// Additional school-bus, public-bus and qualified local-bus fuels
    /// These type-of-use 7, 5 and 6 claims retain separate gallons, physical units and actual costs, including when another use shares the same printed Form 4136 line.
    /// [NullText]
    /// No credit claimed for additional school-bus, public-bus or qualified local-bus fuel
    /// [SetText]
    /// Claim credit for additional school-bus, public-bus or qualified local-bus fuel using
    #[libertas_default($null)]
    pub bus_uses: Option<FederalBusFuelUses>,
    /// Other nontaxable business use of diesel, nonaviation kerosene or diesel-water emulsion — type of use 8
    /// Use distinct gallons and actual costs when other fuel uses share lines 3a, 4a or 14a.
    /// [NullText]
    /// No credit claimed for business heater, process or motorboat fuel
    /// [SetText]
    /// Claim credit for business heater, process or motorboat fuel using
    #[libertas_default($null)]
    pub other_uses: Option<FederalOtherFuelUses>,
    /// Tax-paid undyed nonaviation kerosene exported from the United States — line 4d, type of use 3
    /// The ultimate purchaser claims 24.4 cents per gallon. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported undyed nonaviation kerosene
    /// [SetText]
    /// Claim credit for exported undyed nonaviation kerosene using
    #[libertas_default($null)]
    pub kerosene_export: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene originally taxed at 4.4 cents and used off highway in this business — line 4e, type of use 2
    /// Claim 4.3 cents per gallon; do not include the same gallons on line 4a or 4f.
    /// [NullText]
    /// No credit claimed for off-highway nonaviation kerosene taxed at 4.4 cents
    /// [SetText]
    /// Claim credit for off-highway nonaviation kerosene taxed at 4.4 cents using
    #[libertas_default($null)]
    pub kerosene_off_highway_taxed_044: Option<FederalFuelUse>,
    /// Undyed nonaviation kerosene originally taxed at 21.9 cents and used off highway in this business — line 4f, type of use 2
    /// Claim 21.8 cents per gallon; do not include the same gallons on line 4a or 4e.
    /// [NullText]
    /// No credit claimed for off-highway nonaviation kerosene taxed at 21.9 cents
    /// [SetText]
    /// Claim credit for off-highway nonaviation kerosene taxed at 21.9 cents using
    #[libertas_default($null)]
    pub kerosene_off_highway_taxed_219: Option<FederalFuelUse>,
    /// Commercial-aviation kerosene purchased by the operator — lines 5a and 5b
    /// Excludes foreign trade, noncommercial and mixed aviation use. The operator must not have waived the claim to the vendor.
    /// [NullText]
    /// No credit claimed for operator-purchased commercial-aviation kerosene
    /// [SetText]
    /// Claim credit for operator-purchased commercial-aviation kerosene using
    #[libertas_default($null)]
    pub aviation_commercial: Option<FederalCommercialAviationKerosene>,
    /// Foreign-trade aviation kerosene purchased by the operator — lines 5c, 5d and 5e
    /// Civil-aircraft trade includes compensated carriage or preparation for it; foreign-registered aircraft require reciprocal treatment. The operator keeps the claim right.
    /// [NullText]
    /// No credit claimed for operator-purchased foreign-trade aviation kerosene
    /// [SetText]
    /// Claim credit for operator-purchased foreign-trade aviation kerosene using
    #[libertas_default($null)]
    pub aviation_kerosene_foreign_trade: Option<FederalForeignTradeAviationKerosene>,
    /// Farm-use aviation kerosene bought by its aerial applicator — lines 5c and 5d, type of use 1
    /// Enter original 24.4-cent and 21.9-cent purchases separately. Farm and foreign-trade gallons sharing a printed line are rounded together; only foreign-trade gallons generate line 5e.
    /// [NullText]
    /// No credit claimed for aerial-applicator farm-use aviation kerosene
    /// [SetText]
    /// Claim credit for aerial-applicator farm-use aviation kerosene using
    #[libertas_default($null)]
    pub aviation_kerosene_farm: Option<FederalFarmAviationKerosene>,
    /// Registered ultimate vendor's undyed-diesel sale to a government buyer — line 6a
    /// The seller must hold active IRS UV registration and sell at a tax-excluded price, repay the tax, or have the buyer's written consent. Keep government-buyer and certificate evidence for each sale; ordinary ultimate-purchaser fuel uses belong on other lines.
    /// [NullText]
    /// No credit claimed for registered-vendor government diesel sales
    /// [SetText]
    /// Claim credit for registered-vendor government diesel sales using
    #[libertas_default($null)]
    pub vendor_state_diesel: Option<crate::FederalVendorStateClaim>,
    /// Registered UB vendor's undyed-diesel sale for qualifying public intercity or local buses — line 6b
    /// Claim 17 cents per gallon only when a current Model Waiver N transfers the buyer's claim to the seller. Keep each bus-operator sale and its distinct 6b cost; an ultimate purchaser uses line 3d instead.
    /// [NullText]
    /// No credit claimed for registered UB-vendor public-bus diesel sales
    /// [SetText]
    /// Claim credit for registered UB-vendor public-bus diesel sales using
    #[libertas_default($null)]
    pub vendor_bus_diesel: Option<crate::FederalVendorBusClaim>,
    /// Registered ultimate vendor's undyed nonaviation-kerosene sale to a government buyer — line 7a
    /// Use the separate 24.3-cent line only for sales by the registered UV seller, with a current Model Certificate P and no government-issued credit card. Aviation kerosene, blocked-pump and bus sales belong on other lines.
    /// [NullText]
    /// No credit claimed for registered-vendor government nonaviation kerosene sales
    /// [SetText]
    /// Claim credit for registered-vendor government nonaviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_state_kerosene: Option<crate::FederalVendorStateClaim>,
    /// Registered ultimate vendor's undyed nonaviation-kerosene sales from a blocked retail pump — line 7b
    /// The seller needs active UP registration, a fixed and marked pump with qualifying access, and records for each sale. Locked-pump sales over five gallons need buyer name and address.
    /// [NullText]
    /// No credit claimed for registered-vendor blocked-pump nonaviation kerosene sales
    /// [SetText]
    /// Claim credit for registered-vendor blocked-pump nonaviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_blocked_pump_kerosene: Option<crate::FederalBlockedPumpKeroseneClaim>,
    /// Registered UB vendor's undyed nonaviation-kerosene sale for qualifying public buses — line 7c
    /// Claim 17 cents per gallon only with a current Model Waiver N; retain sale, bus-use and waiver evidence. The purchaser's own claim belongs on line 4c.
    /// [NullText]
    /// No credit claimed for registered UB-vendor public-bus nonaviation kerosene sales
    /// [SetText]
    /// Claim credit for registered UB-vendor public-bus nonaviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_bus_kerosene: Option<crate::FederalVendorBusClaim>,
    /// Registered UA vendor's kerosene sales for commercial aviation other than foreign trade — lines 8a and 8b
    /// The buyer's unexpired Model Waiver L transfers the claim. Separate the 21.9-cent original-tax sales on line 8a from 24.4-cent sales on line 8b; the corresponding credits are 17.5 and 20.0 cents per gallon.
    /// [NullText]
    /// No credit claimed for registered UA-vendor commercial-aviation kerosene sales
    /// [SetText]
    /// Claim credit for registered UA-vendor commercial-aviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_aviation_kerosene: Option<crate::FederalVendorAviationClaim>,
    /// Registered UA vendor's kerosene sales for nonexempt, noncommercial aviation — line 8c
    /// Claim 2.5 cents per gallon only for 24.4-cent-taxed fuel with the buyer's current Model Certificate Q. Keep commercial, exempt, foreign-trade, government and mixed-use sales separate.
    /// [NullText]
    /// No credit claimed for registered UA-vendor nonexempt noncommercial aviation kerosene sales
    /// [SetText]
    /// Claim credit for registered UA-vendor nonexempt noncommercial aviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_aviation_noncommercial: Option<crate::FederalVendorNoncommercialAviationClaim>,
    /// Registered vendor's nontaxable noncommercial aviation kerosene sales — lines 8d, 8e and, for foreign trade, 8f
    /// Use a current Model Waiver L with the seller's UA registration, or Model Certificate P and UV registration for government type 14. Original tax of 24.4 or 21.9 cents determines the first claim line; foreign trade also earns the separate 0.1-cent LUST line.
    /// [NullText]
    /// No credit claimed for registered-vendor nontaxable noncommercial aviation kerosene sales
    /// [SetText]
    /// Claim credit for registered-vendor nontaxable noncommercial aviation kerosene sales using
    #[libertas_default($null)]
    pub vendor_aviation_nontaxable: Option<crate::FederalVendorNontaxableAviationClaim>,
    /// Tax-paid liquefied petroleum gas (LPG) used off highway in this business — line 11a, type of use 2
    /// The 18.3-cent rate applies per GGE. One GGE is 1.353 liquid gallons or 5.75 pounds; bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway LPG
    /// [SetText]
    /// Claim credit for off-highway LPG using
    #[libertas_default($null)]
    pub lpg_off_highway: Option<FederalAlternativeFuelUse>,
    /// Tax-paid P Series fuel used off highway in this business — line 11b, type of use 2
    /// The 18.3-cent rate applies per gallon; certain bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway P Series fuel
    /// [SetText]
    /// Claim credit for off-highway P Series fuel using
    #[libertas_default($null)]
    pub p_series_off_highway: Option<FederalFuelUse>,
    /// Tax-paid compressed natural gas (CNG) used off highway in this business — line 11c, type of use 2
    /// The 18.3-cent rate applies per GGE. One GGE is 123.57 cubic feet or 5.66 pounds; bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway CNG
    /// [SetText]
    /// Claim credit for off-highway CNG using
    #[libertas_default($null)]
    pub cng_off_highway: Option<FederalAlternativeFuelUse>,
    /// Tax-paid liquefied hydrogen used off highway in this business — line 11d, type of use 2
    /// The 18.3-cent rate applies per gallon; certain bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway liquefied hydrogen
    /// [SetText]
    /// Claim credit for off-highway liquefied hydrogen using
    #[libertas_default($null)]
    pub hydrogen_off_highway: Option<FederalFuelUse>,
    /// Tax-paid Fischer-Tropsch liquid fuel from coal or peat used off highway in this business — line 11e, type of use 2
    /// The 24.3-cent rate applies per gallon; certain bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway Fischer-Tropsch coal- or peat-derived fuel
    /// [SetText]
    /// Claim credit for off-highway Fischer-Tropsch coal- or peat-derived fuel using
    #[libertas_default($null)]
    pub fischer_tropsch_off_highway: Option<FederalFuelUse>,
    /// Tax-paid liquid fuel derived from biomass used off highway in this business — line 11f, type of use 2
    /// The 24.3-cent rate applies per gallon; certain bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway biomass-derived liquid fuel
    /// [SetText]
    /// Claim credit for off-highway biomass-derived liquid fuel using
    #[libertas_default($null)]
    pub biomass_liquid_off_highway: Option<FederalFuelUse>,
    /// Tax-paid liquefied natural gas (LNG) used off highway in this business — line 11g, type of use 2
    /// The 24.3-cent rate applies per DGE. One DGE is 1.71 liquid gallons or 6.06 pounds; bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway LNG
    /// [SetText]
    /// Claim credit for off-highway LNG using
    #[libertas_default($null)]
    pub lng_off_highway: Option<FederalAlternativeFuelUse>,
    /// Tax-paid liquefied gas derived from biomass used off highway in this business — line 11h, type of use 2
    /// The 18.3-cent rate applies per gallon; certain bus use has a different rate.
    /// [NullText]
    /// No credit claimed for off-highway liquefied biomass gas
    /// [SetText]
    /// Claim credit for off-highway liquefied biomass gas using
    #[libertas_default($null)]
    pub biomass_gas_off_highway: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion used off-highway in this business — line 14a, type of use 2
    /// Enter the emulsion's gallons, not its undiluted diesel component. The blender's line 15 claim is separate and is not included here.
    /// [NullText]
    /// No credit claimed for off-highway diesel-water emulsion
    /// [SetText]
    /// Claim credit for off-highway diesel-water emulsion using
    #[libertas_default($null)]
    pub emulsion_off_highway: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion used on a farm for farming — line 14a, type of use 1
    /// Claim 19.7 cents per emulsion gallon, distinct from off-highway or exported emulsion. The direct activity or partner K-1 statement must establish farm use.
    /// [NullText]
    /// No credit claimed for farm-use diesel-water emulsion
    /// [SetText]
    /// Claim credit for farm-use diesel-water emulsion using
    #[libertas_default($null)]
    pub emulsion_farm: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion used in a qualifying public intercity or local bus — line 14a, type of use 5
    /// Claim the reduced 12.4 cents per emulsion gallon. Paid public service must use scheduled routes or at least 20 adult seats for nonscheduled service; keep it separate from the ordinary line 14a rate.
    /// [NullText]
    /// No credit claimed for public intercity or local-bus diesel-water emulsion
    /// [SetText]
    /// Claim credit for public intercity or local-bus diesel-water emulsion using
    #[libertas_default($null)]
    pub emulsion_intercity_local_bus: Option<FederalFuelUse>,
    /// Tax-paid diesel-water fuel emulsion exported from the United States — line 14b, type of use 3
    /// The ultimate purchaser claims 19.8 cents per emulsion gallon. Confirm export proof below; this is not the blender's separate line 15 claim.
    /// [NullText]
    /// No credit claimed for exported diesel-water emulsion
    /// [SetText]
    /// Claim credit for exported diesel-water emulsion using
    #[libertas_default($null)]
    pub emulsion_export: Option<FederalFuelUse>,
    /// Registered blender's undyed diesel used to produce diesel-water fuel emulsion — line 15a
    /// The 4.6-cent credit is per gallon of diesel input, distinct from line 14's finished-emulsion gallons. An IRS Form 637 M registration and complete certification are required.
    /// [NullText]
    /// No credit claimed for registered-blender diesel used in diesel-water emulsion
    /// [SetText]
    /// Claim credit for registered-blender diesel used in diesel-water emulsion using
    #[libertas_default($null)]
    pub emulsion_blender: Option<FederalEmulsionBlenderClaim>,
    /// Gasoline blendstocks exported after 0.1 cent per gallon LUST tax was paid — line 16a
    /// This is taxed blendstock, not finished gasoline taxed at 18.4 cents on line 1d. The exporter retains proof; line 16a combines these gallons with exported dyed diesel before cent rounding.
    /// [NullText]
    /// No credit claimed for exported gasoline blendstocks subject to LUST tax
    /// [SetText]
    /// Claim credit for exported gasoline blendstocks subject to LUST tax using
    #[libertas_default($null)]
    pub gasoline_blendstock_export: Option<FederalFuelUse>,
    /// Dyed diesel exported from the United States after 0.1 cent per gallon LUST tax was paid — line 16a
    /// The exporter claims only the 0.1-cent tax; do not enter undyed diesel here. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported dyed diesel subject to LUST tax
    /// [SetText]
    /// Claim credit for exported dyed diesel subject to LUST tax using
    #[libertas_default($null)]
    pub dyed_diesel_export: Option<FederalFuelUse>,
    /// Dyed kerosene exported from the United States after 0.1 cent per gallon LUST tax was paid — line 16b
    /// The exporter claims only the 0.1-cent tax; do not enter undyed kerosene here. Confirm export proof below.
    /// [NullText]
    /// No credit claimed for exported dyed kerosene subject to LUST tax
    /// [SetText]
    /// Claim credit for exported dyed kerosene subject to LUST tax using
    #[libertas_default($null)]
    pub dyed_kerosene_export: Option<FederalFuelUse>,
    /// The exported fuel left the United States to remain abroad, with required proof retained
    /// Confirm when claiming any export line. A fuel supply tank carried abroad is not an export. Keep the bill of lading, carrier/customs certificate or foreign consignee's receipt with the business records. A bill of lading, customs certificate or consignee receipt shows this. No leaves the exported fuel out of the fuel tax credit with a notice.
    /// [NullText]
    /// No exported-fuel credit claimed
    /// [SetText]
    /// For the claimed exported fuel, export and retained proof are confirmed
    #[libertas_default($null)]
    pub export_proof_complete: Option<bool>,
    /// Registered CC credit-card issuer's government fuel purchases — lines 13a, 13b and 13c
    /// This issuer claim is distinct from an ultimate purchaser's use or an ultimate vendor's sale. Buyer consent permits a fully refunded claim or a narrowly reconciled retained-tax claim. An accrual-method direct issuer includes the retained credit once as 2026 Schedule C other income; buyer/vendor card-principal transfers stay outside receipts and expenses. Cash-method retained tax needs a later-year income worksheet. The 21.9-cent and 24.4-cent original-tax rates on line 13c retain separate gallon and credit buckets.
    /// [NullText]
    /// No credit claimed for registered CC-issuer government fuel purchases
    /// [SetText]
    /// Claim credit for registered CC-issuer government fuel purchases using
    #[libertas_default($null)]
    pub card_issuer: Option<crate::FederalCardIssuerClaim>,
}

/// Form 4136 fuel tax credit for this sole-proprietor business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalFuelTaxCredit {
    /// Qualified fuel and use lines for this business
    // Boxed: inline, these lines would set the size of every income source and record value.
    pub fuels: alloc::boxed::Box<FederalFuelLines>,
    /// How the credited tax was handled in the expense records above
    pub expense_treatment: FederalFuelExpenseTreatment,
    /// Fuel purchase or issuer sale, payment of tax, equipment/use where applicable, and business records are complete
    /// Your fuel invoices and use records show this. No leaves this fuel tax credit out of the estimate with a notice.
    pub records_complete: bool,
}

// Answers and unknowns: what dropping the unconfirmed
// ultimate-purchaser fuel uses left out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FuelUseDrop {
    // A No answer left a use out.
    pub(crate) left_out: bool,
}

impl FuelUseDrop {
    pub(super) fn keep(&mut self, qualified: bool, others: bool) -> bool {
        if qualified && others {
            return true;
        }
        self.left_out = true;
        false
    }

    pub(super) fn fuel(&mut self, slot: &mut Option<FederalFuelUse>) {
        if let Some(value) = slot
            && !self.keep(value.qualified, value.not_claimed_elsewhere)
        {
            *slot = None;
        }
    }

    pub(super) fn alternative(&mut self, slot: &mut Option<FederalAlternativeFuelUse>) {
        if let Some(value) = slot
            && !self.keep(value.qualified, value.not_claimed_elsewhere)
        {
            *slot = None;
        }
    }

    // A group whose own answers do not hold leaves all its uses out.
    pub(super) fn group<T>(&mut self, slot: &mut Option<T>, holds: impl Fn(&T) -> bool) {
        if slot.as_ref().is_some_and(|value| !holds(value)) {
            self.left_out = true;
            *slot = None;
        }
    }

    // A group emptied by this call goes too; one entered empty stays for
    // its validity check.
    pub(super) fn emptied<T>(self, before: Self, slot: &mut Option<T>, has: impl Fn(&T) -> bool) {
        if self != before && slot.as_ref().is_some_and(|value| !has(value)) {
            *slot = None;
        }
    }
}

impl FederalFuelLines {
    /// Leaves out each ultimate-purchaser use whose answers do not qualify;
    /// the remaining lines are claimed. Vendor and card-issuer claims are
    /// `drop_unsupported`'s.
    pub(crate) fn drop_unconfirmed_uses(&mut self) -> FuelUseDrop {
        let mut drop = FuelUseDrop::default();
        if (self.diesel_intercity_local_bus.is_some()
            || self.kerosene_intercity_local_bus.is_some())
            && self.intercity_bus_no_waiver != Some(true)
        {
            drop.left_out = true;
            self.diesel_intercity_local_bus = None;
            self.kerosene_intercity_local_bus = None;
        }
        if self.has_export_use() && self.export_proof_complete != Some(true) {
            drop.left_out = true;
            for slot in [
                &mut self.gasoline_export,
                &mut self.aviation_gasoline_export,
                &mut self.diesel_export,
                &mut self.kerosene_export,
                &mut self.emulsion_export,
                &mut self.gasoline_blendstock_export,
                &mut self.dyed_diesel_export,
                &mut self.dyed_kerosene_export,
            ] {
                *slot = None;
            }
        }
        for slot in [
            &mut self.gasoline_off_highway,
            &mut self.gasoline_farm,
            &mut self.gasoline_commercial_fishing,
            &mut self.gasoline_school_bus,
            &mut self.gasoline_intercity_local_bus,
            &mut self.gasoline_export,
            &mut self.aviation_gasoline_commercial,
            &mut self.aviation_gasoline_nonpropulsion_off_highway,
            &mut self.aviation_gasoline_export,
            &mut self.diesel_off_highway,
            &mut self.diesel_farm,
            &mut self.diesel_trains,
            &mut self.diesel_intercity_local_bus,
            &mut self.diesel_export,
            &mut self.kerosene_off_highway,
            &mut self.kerosene_farm,
            &mut self.kerosene_intercity_local_bus,
            &mut self.kerosene_export,
            &mut self.kerosene_off_highway_taxed_044,
            &mut self.kerosene_off_highway_taxed_219,
            &mut self.p_series_off_highway,
            &mut self.hydrogen_off_highway,
            &mut self.fischer_tropsch_off_highway,
            &mut self.biomass_liquid_off_highway,
            &mut self.biomass_gas_off_highway,
            &mut self.emulsion_off_highway,
            &mut self.emulsion_farm,
            &mut self.emulsion_intercity_local_bus,
            &mut self.emulsion_export,
            &mut self.gasoline_blendstock_export,
            &mut self.dyed_diesel_export,
            &mut self.dyed_kerosene_export,
        ] {
            drop.fuel(slot);
        }
        for slot in
            [&mut self.lpg_off_highway, &mut self.cng_off_highway, &mut self.lng_off_highway]
        {
            drop.alternative(slot);
        }
        drop.group(&mut self.emulsion_blender, |claim| {
            claim.statement_facts_confirmed && claim.not_claimed_elsewhere
        });
        drop.group(&mut self.aviation_commercial, |claim| claim.no_waiver);
        let before = drop;
        if let Some(claim) = &mut self.aviation_commercial {
            drop.fuel(&mut claim.taxed_at_244);
            drop.fuel(&mut claim.taxed_at_219);
        }
        drop.emptied(before, &mut self.aviation_commercial, |claim| {
            claim.uses().iter().any(Option::is_some)
        });
        drop.group(&mut self.aviation_kerosene_foreign_trade, |claim| claim.no_waiver);
        let before = drop;
        if let Some(claim) = &mut self.aviation_kerosene_foreign_trade {
            drop.fuel(&mut claim.taxed_at_244);
            drop.fuel(&mut claim.taxed_at_219);
        }
        drop.emptied(before, &mut self.aviation_kerosene_foreign_trade, |claim| {
            claim.uses().iter().any(Option::is_some)
        });
        drop.group(&mut self.aviation_kerosene_farm, |claim| {
            claim.applicator_was_ultimate_purchaser && claim.no_waiver && claim.distinct_gallons
        });
        let before = drop;
        if let Some(claim) = &mut self.aviation_kerosene_farm {
            drop.fuel(&mut claim.taxed_at_244_application);
            drop.fuel(&mut claim.taxed_at_244_direct_flight);
            drop.fuel(&mut claim.taxed_at_219_application);
            drop.fuel(&mut claim.taxed_at_219_direct_flight);
        }
        drop.emptied(before, &mut self.aviation_kerosene_farm, |claim| {
            claim.uses().iter().any(Option::is_some)
        });
        drop.group(&mut self.aviation_gasoline_farm, |claim| {
            claim.applicator_was_ultimate_purchaser && claim.distinct_gallons
        });
        let before = drop;
        if let Some(claim) = &mut self.aviation_gasoline_farm {
            drop.fuel(&mut claim.aerial_application);
            drop.fuel(&mut claim.direct_airfield_farm_flight);
        }
        drop.emptied(before, &mut self.aviation_gasoline_farm, |claim| {
            claim.uses().iter().any(Option::is_some)
        });
        drop.group(&mut self.aviation_gasoline_type_10, |uses| uses.distinct_gallons);
        let before = drop;
        if let Some(uses) = &mut self.aviation_gasoline_type_10 {
            if uses.mineral_and_tree_airport_condition != Some(true) {
                for slot in [
                    &mut uses.helicopter_mineral_transport,
                    &mut uses.helicopter_tree_work,
                    &mut uses.fixed_wing_tree_work,
                ] {
                    if slot.take().is_some() {
                        drop.left_out = true;
                    }
                }
            }
            if uses.fixed_wing_acute_care_condition != Some(true)
                && uses.fixed_wing_emergency_medical.take().is_some()
            {
                drop.left_out = true;
            }
            drop.fuel(&mut uses.helicopter_mineral_transport);
            drop.fuel(&mut uses.helicopter_tree_work);
            drop.fuel(&mut uses.helicopter_emergency_medical);
            drop.fuel(&mut uses.fixed_wing_tree_work);
            drop.fuel(&mut uses.fixed_wing_emergency_medical);
        }
        drop.emptied(before, &mut self.aviation_gasoline_type_10, |uses| {
            uses.uses().iter().any(Option::is_some)
        });
        if let Some(trade) = &self.aviation_gasoline_foreign_trade {
            let reciprocity = match trade.registration {
                FederalForeignTradeAircraftRegistration::UnitedStates => true,
                FederalForeignTradeAircraftRegistration::Foreign { reciprocal_privileges } => {
                    reciprocal_privileges
                }
            };
            let fuel = drop.keep(trade.fuel.qualified, trade.fuel.not_claimed_elsewhere);
            let facts = trade.qualifying_civil_aircraft_trade && trade.distinct_gallons;
            if !fuel || !facts || !drop.keep(reciprocity, true) {
                if !facts {
                    drop.left_out = true;
                }
                self.aviation_gasoline_foreign_trade = None;
            }
        }
        let before = drop;
        if let Some(bus) = &mut self.bus_uses {
            if bus.has_local_bus_use() {
                let qualified = bus.local_bus_qualified.unwrap_or(false);
                if !drop.keep(qualified, true) {
                    bus.diesel_local_bus = None;
                    bus.kerosene_local_bus = None;
                    bus.emulsion_local_bus = None;
                    bus.lpg_local_bus = None;
                    bus.p_series_local_bus = None;
                    bus.cng_local_bus = None;
                    bus.hydrogen_local_bus = None;
                    bus.fischer_tropsch_local_bus = None;
                    bus.biomass_liquid_local_bus = None;
                    bus.lng_local_bus = None;
                    bus.biomass_gas_local_bus = None;
                }
            }
            for slot in [
                &mut bus.diesel_school_bus,
                &mut bus.kerosene_school_bus,
                &mut bus.emulsion_school_bus,
                &mut bus.p_series_public_bus,
                &mut bus.hydrogen_public_bus,
                &mut bus.fischer_tropsch_public_bus,
                &mut bus.biomass_liquid_public_bus,
                &mut bus.biomass_gas_public_bus,
                &mut bus.diesel_local_bus,
                &mut bus.kerosene_local_bus,
                &mut bus.emulsion_local_bus,
                &mut bus.p_series_local_bus,
                &mut bus.hydrogen_local_bus,
                &mut bus.fischer_tropsch_local_bus,
                &mut bus.biomass_liquid_local_bus,
                &mut bus.biomass_gas_local_bus,
            ] {
                drop.fuel(slot);
            }
            for slot in [
                &mut bus.lpg_public_bus,
                &mut bus.cng_public_bus,
                &mut bus.lng_public_bus,
                &mut bus.lpg_local_bus,
                &mut bus.cng_local_bus,
                &mut bus.lng_local_bus,
            ] {
                drop.alternative(slot);
            }
        }
        drop.emptied(before, &mut self.bus_uses, |bus| {
            bus.priced_uses().iter().any(|(value, _)| value.is_some())
                || bus.alternative_uses().iter().any(|(value, _, _)| value.is_some())
        });
        let before = drop;
        if let Some(other) = &mut self.other_uses {
            drop.fuel(&mut other.diesel_type_8);
            drop.fuel(&mut other.kerosene_type_8);
            drop.fuel(&mut other.emulsion_type_8);
        }
        drop.emptied(before, &mut self.other_uses, |other| {
            other.priced_uses().iter().any(|(value, _)| value.is_some())
        });
        drop
    }
}

pub(super) const FUEL_LINE_COUNT: usize = 52;
pub(super) const VENDOR_STATE_DIESEL_RATE: i64 = 243;
pub(super) const VENDOR_STATE_KEROSENE_RATE: i64 = 243;
pub(super) const FUEL_LINE_NAMES: [&str; FUEL_LINE_COUNT] = [
    "1a", "1b", "1c", "1d", "2a", "2b", "2c", "2d", "3a", "3b", "3c", "3d", "3e", "4a", "4b", "4c",
    "4d", "4e", "4f", "5a", "5b", "5c", "5d", "5e", "6a", "6b", "7a", "7b", "7c", "8a", "8b", "8c",
    "8d", "8e", "8f", "11a", "11b", "11c", "11d", "11e", "11f", "11g", "11h", "13a", "13b",
    "13c_219", "13c_244", "14a", "14b", "15a", "16a", "16b",
];

impl FederalFuelLines {
    pub(super) fn line_cents(value: Option<&FederalFuelUse>, rate_thousandths: i64) -> i64 {
        value.map_or(0, |v| {
            crate::kernel::ratio::fraction(
                crate::kernel::natural::thousandths(v.gallons),
                rate_thousandths,
                10_000,
            )
        })
    }

    pub(super) fn line_2b_gallons_thousandths(&self) -> i64 {
        self.aviation_gasoline_nonpropulsion_off_highway
            .as_ref()
            .map_or(0, |value| crate::kernel::natural::thousandths(value.gallons))
            + self
                .aviation_gasoline_farm
                .as_ref()
                .map_or(0, FederalAviationGasolineFarmUse::gallons_thousandths)
            + self
                .aviation_gasoline_type_10
                .as_ref()
                .map_or(0, FederalAviationGasolineType10Uses::gallons_thousandths)
            + self
                .aviation_gasoline_foreign_trade
                .as_ref()
                .map_or(0, |value| crate::kernel::natural::thousandths(value.fuel.gallons))
    }

    pub(super) fn line_2b_cents(&self) -> i64 {
        crate::kernel::ratio::fraction(self.line_2b_gallons_thousandths(), 193, 10_000)
    }

    pub(super) fn line_2b_cost_cents(&self) -> i64 {
        self.aviation_gasoline_nonpropulsion_off_highway
            .as_ref()
            .map_or(0, |value| value.actual_cost)
            + self
                .aviation_gasoline_farm
                .as_ref()
                .map_or(0, FederalAviationGasolineFarmUse::cost_cents)
            + self.aviation_gasoline_type_10.as_ref().map_or(0, |type_10| {
                type_10.uses().into_iter().flatten().map(|value| value.actual_cost).sum()
            })
            + self
                .aviation_gasoline_foreign_trade
                .as_ref()
                .map_or(0, |value| value.fuel.actual_cost)
    }

    pub(super) fn line_5c_or_5d_gallons_thousandths(&self, original_rate: i64) -> i64 {
        let trade = self.aviation_kerosene_foreign_trade.as_ref().map_or(0, |value| {
            let fuel = if original_rate == 244 { &value.taxed_at_244 } else { &value.taxed_at_219 };
            fuel.as_ref().map_or(0, |fuel| crate::kernel::natural::thousandths(fuel.gallons))
        });
        trade
            + self
                .aviation_kerosene_farm
                .as_ref()
                .map_or(0, |value| value.gallons_thousandths(original_rate))
    }

    pub(super) fn line_5c_or_5d_cents(&self, original_rate: i64) -> i64 {
        let credit_rate = if original_rate == 244 { 243 } else { 218 };
        crate::kernel::ratio::fraction(
            self.line_5c_or_5d_gallons_thousandths(original_rate),
            credit_rate,
            10_000,
        )
    }

    pub(super) fn line_5c_through_5e_cost_cents(&self) -> i64 {
        self.aviation_kerosene_foreign_trade.as_ref().map_or(0, |value| {
            value.uses().into_iter().flatten().map(|fuel| fuel.actual_cost).sum::<i64>()
        }) + self.aviation_kerosene_farm.as_ref().map_or(0, FederalFarmAviationKerosene::cost_cents)
    }

    pub(super) fn priced_uses(&self) -> [(Option<&FederalFuelUse>, i64); 33] {
        [
            (self.gasoline_off_highway.as_ref(), 183),
            (self.gasoline_farm.as_ref(), 183),
            (self.gasoline_commercial_fishing.as_ref(), 183),
            (self.gasoline_school_bus.as_ref(), 183),
            (self.aviation_gasoline_commercial.as_ref(), 150),
            (self.aviation_gasoline_nonpropulsion_off_highway.as_ref(), 193),
            (self.diesel_off_highway.as_ref(), 243),
            (self.diesel_farm.as_ref(), 243),
            (self.diesel_trains.as_ref(), 243),
            (self.kerosene_off_highway.as_ref(), 243),
            (self.kerosene_farm.as_ref(), 243),
            (self.kerosene_off_highway_taxed_044.as_ref(), 43),
            (self.kerosene_off_highway_taxed_219.as_ref(), 218),
            (self.p_series_off_highway.as_ref(), 183),
            (self.hydrogen_off_highway.as_ref(), 183),
            (self.fischer_tropsch_off_highway.as_ref(), 243),
            (self.biomass_liquid_off_highway.as_ref(), 243),
            (self.biomass_gas_off_highway.as_ref(), 183),
            (self.emulsion_off_highway.as_ref(), 197),
            (self.gasoline_intercity_local_bus.as_ref(), 183),
            (self.diesel_intercity_local_bus.as_ref(), 170),
            (self.kerosene_intercity_local_bus.as_ref(), 170),
            (self.gasoline_export.as_ref(), 184),
            (self.aviation_gasoline_export.as_ref(), 194),
            (self.diesel_export.as_ref(), 244),
            (self.kerosene_export.as_ref(), 244),
            (self.emulsion_export.as_ref(), 198),
            (self.emulsion_farm.as_ref(), 197),
            (self.emulsion_intercity_local_bus.as_ref(), 124),
            (self.gasoline_blendstock_export.as_ref(), 1),
            (self.dyed_diesel_export.as_ref(), 1),
            (self.dyed_kerosene_export.as_ref(), 1),
            (self.aviation_gasoline_foreign_trade.as_ref().map(|value| &value.fuel), 194),
        ]
    }

    pub(super) fn regular_lines(&self) -> [i64; 18] {
        let amount = self.priced_uses().map(|(use_line, rate)| Self::line_cents(use_line, rate));
        [
            amount[0],
            amount[1],
            amount[2] + amount[3] + amount[19],
            amount[4],
            self.line_2b_cents(),
            amount[6],
            amount[7],
            amount[8],
            amount[9],
            amount[10],
            amount[11],
            amount[12],
            amount[13],
            amount[14],
            amount[15],
            amount[16],
            amount[17],
            amount[18] + amount[27] + amount[28],
        ]
    }

    pub(super) fn line_16a_cents(&self) -> i64 {
        let gallons = [self.gasoline_blendstock_export.as_ref(), self.dyed_diesel_export.as_ref()]
            .into_iter()
            .flatten()
            .map(|value| crate::kernel::natural::thousandths(value.gallons))
            .sum::<i64>();
        crate::kernel::ratio::fraction(gallons, 1, 10_000)
    }

    pub(crate) fn lines(&self) -> [i64; FUEL_LINE_COUNT] {
        let regular = self.regular_lines();
        let aviation = self
            .aviation_commercial
            .as_ref()
            .map_or([0, 0], FederalCommercialAviationKerosene::lines);
        let foreign_trade = self
            .aviation_kerosene_foreign_trade
            .as_ref()
            .map_or([0, 0, 0], FederalForeignTradeAviationKerosene::lines);
        let vendor_aviation = self
            .vendor_aviation_kerosene
            .as_ref()
            .map_or([0, 0], FederalVendorAviationClaim::lines);
        let vendor_nontaxable = self
            .vendor_aviation_nontaxable
            .as_ref()
            .map_or([0; 3], FederalVendorNontaxableAviationClaim::lines);
        let card_issuer = self.card_issuer.as_ref().map_or([0; 4], FederalCardIssuerClaim::lines);
        let mut lines = [
            regular[0],
            regular[1],
            regular[2],
            Self::line_cents(self.gasoline_export.as_ref(), 184),
            regular[3],
            regular[4],
            Self::line_cents(self.aviation_gasoline_export.as_ref(), 194),
            Self::line_cents(
                self.aviation_gasoline_foreign_trade.as_ref().map(|value| &value.fuel),
                1,
            ),
            regular[5],
            regular[6],
            regular[7],
            Self::line_cents(self.diesel_intercity_local_bus.as_ref(), 170),
            Self::line_cents(self.diesel_export.as_ref(), 244),
            regular[8],
            regular[9],
            Self::line_cents(self.kerosene_intercity_local_bus.as_ref(), 170),
            Self::line_cents(self.kerosene_export.as_ref(), 244),
            regular[10],
            regular[11],
            aviation[0],
            aviation[1],
            self.line_5c_or_5d_cents(244),
            self.line_5c_or_5d_cents(219),
            foreign_trade[2],
            self.vendor_state_diesel
                .as_ref()
                .map_or(0, |claim| claim.line_cents(VENDOR_STATE_DIESEL_RATE)),
            self.vendor_bus_diesel.as_ref().map_or(0, FederalVendorBusClaim::line_cents),
            self.vendor_state_kerosene
                .as_ref()
                .map_or(0, |claim| claim.line_cents(VENDOR_STATE_KEROSENE_RATE)),
            self.vendor_blocked_pump_kerosene
                .as_ref()
                .map_or(0, FederalBlockedPumpKeroseneClaim::line_cents),
            self.vendor_bus_kerosene.as_ref().map_or(0, FederalVendorBusClaim::line_cents),
            vendor_aviation[0],
            vendor_aviation[1],
            self.vendor_aviation_noncommercial
                .as_ref()
                .map_or(0, FederalVendorNoncommercialAviationClaim::line_cents),
            vendor_nontaxable[0],
            vendor_nontaxable[1],
            vendor_nontaxable[2],
            self.lpg_off_highway
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lpg, 183))
                .unwrap_or(0),
            regular[12],
            self.cng_off_highway
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Cng, 183))
                .unwrap_or(0),
            regular[13],
            regular[14],
            regular[15],
            self.lng_off_highway
                .as_ref()
                .and_then(|v| v.line_cents(AlternativeFuelKind::Lng, 243))
                .unwrap_or(0),
            regular[16],
            card_issuer[0],
            card_issuer[1],
            card_issuer[2],
            card_issuer[3],
            regular[17],
            Self::line_cents(self.emulsion_export.as_ref(), 198),
            self.emulsion_blender.as_ref().map_or(0, FederalEmulsionBlenderClaim::credit_cents),
            self.line_16a_cents(),
            Self::line_cents(self.dyed_kerosene_export.as_ref(), 1),
        ];
        if let Some(bus) = &self.bus_uses {
            let additional = bus.lines();
            for (index, amount) in [
                8, 13, 35, 36, 37, 38, 39, 40, 41, 42, 47, 8, 13, 47, 35, 36, 37, 38, 39, 40, 41,
                42,
            ]
            .into_iter()
            .zip(additional)
            {
                lines[index] += amount;
            }
        }
        if let Some(other) = &self.other_uses {
            let [diesel, kerosene, emulsion] = other.lines();
            lines[8] += diesel;
            lines[13] += kerosene;
            lines[47] += emulsion;
        }
        lines
    }

    pub(crate) fn labeled_lines(&self) -> impl Iterator<Item = (&'static str, i64)> {
        FUEL_LINE_NAMES.into_iter().zip(self.lines())
    }

    pub(crate) fn credit_cents(&self) -> i64 {
        self.lines().iter().sum()
    }

    pub(crate) fn uses(&self) -> [Option<&FederalFuelUse>; 33] {
        self.priced_uses().map(|(use_line, _)| use_line)
    }

    pub(crate) fn alternative_uses(&self) -> [Option<&FederalAlternativeFuelUse>; 3] {
        [
            self.lpg_off_highway.as_ref(),
            self.cng_off_highway.as_ref(),
            self.lng_off_highway.as_ref(),
        ]
    }

    pub(crate) fn qualified(&self) -> bool {
        self.uses().iter().flatten().all(|v| v.qualified && v.not_claimed_elsewhere)
            && self
                .alternative_uses()
                .iter()
                .flatten()
                .all(|v| v.qualified && v.not_claimed_elsewhere)
            && self
                .aviation_commercial
                .as_ref()
                .is_none_or(FederalCommercialAviationKerosene::qualified)
            && self
                .aviation_kerosene_foreign_trade
                .as_ref()
                .is_none_or(FederalForeignTradeAviationKerosene::qualified)
            && self
                .aviation_kerosene_farm
                .as_ref()
                .is_none_or(FederalFarmAviationKerosene::qualified)
            && self
                .aviation_gasoline_farm
                .as_ref()
                .is_none_or(FederalAviationGasolineFarmUse::qualified)
            && self
                .aviation_gasoline_type_10
                .as_ref()
                .is_none_or(FederalAviationGasolineType10Uses::qualified)
            && self
                .aviation_gasoline_foreign_trade
                .as_ref()
                .is_none_or(FederalAviationGasolineForeignTrade::qualified)
            && self.bus_uses.as_ref().is_none_or(FederalBusFuelUses::qualified)
            && self.other_uses.as_ref().is_none_or(FederalOtherFuelUses::qualified)
            && self.emulsion_blender.as_ref().is_none_or(FederalEmulsionBlenderClaim::qualified)
            && self.vendor_state_diesel.as_ref().is_none_or(FederalVendorStateClaim::qualified)
            && self.vendor_bus_diesel.as_ref().is_none_or(FederalVendorBusClaim::qualified)
            && self.vendor_state_kerosene.as_ref().is_none_or(FederalVendorStateClaim::qualified)
            && self
                .vendor_blocked_pump_kerosene
                .as_ref()
                .is_none_or(FederalBlockedPumpKeroseneClaim::qualified)
            && self.vendor_bus_kerosene.as_ref().is_none_or(FederalVendorBusClaim::qualified)
            && self
                .vendor_aviation_kerosene
                .as_ref()
                .is_none_or(FederalVendorAviationClaim::qualified)
            && self
                .vendor_aviation_noncommercial
                .as_ref()
                .is_none_or(FederalVendorNoncommercialAviationClaim::qualified)
            && self
                .vendor_aviation_nontaxable
                .as_ref()
                .is_none_or(FederalVendorNontaxableAviationClaim::qualified)
            && self.card_issuer.as_ref().is_none_or(FederalCardIssuerClaim::qualified)
            && ((self.diesel_intercity_local_bus.is_none()
                && self.kerosene_intercity_local_bus.is_none())
                || self.intercity_bus_no_waiver == Some(true))
            && (!self.has_export_use() || self.export_proof_complete == Some(true))
    }

    pub(crate) fn has_farm_operator_use(&self) -> bool {
        self.gasoline_farm.is_some()
            || self.diesel_farm.is_some()
            || self.kerosene_farm.is_some()
            || self.emulsion_farm.is_some()
    }

    pub(super) fn has_export_use(&self) -> bool {
        self.gasoline_export.is_some()
            || self.aviation_gasoline_export.is_some()
            || self.diesel_export.is_some()
            || self.kerosene_export.is_some()
            || self.emulsion_export.is_some()
            || self.gasoline_blendstock_export.is_some()
            || self.dyed_diesel_export.is_some()
            || self.dyed_kerosene_export.is_some()
    }

    pub(crate) fn valid(&self) -> bool {
        let uses = self.uses();
        (uses.iter().any(Option::is_some)
            || self.alternative_uses().iter().any(Option::is_some)
            || self.aviation_commercial.is_some()
            || self.aviation_kerosene_foreign_trade.is_some()
            || self.aviation_kerosene_farm.is_some()
            || self.aviation_gasoline_farm.is_some()
            || self.aviation_gasoline_type_10.is_some()
            || self.bus_uses.is_some()
            || self.other_uses.is_some()
            || self.emulsion_blender.is_some()
            || self.vendor_state_diesel.is_some()
            || self.vendor_bus_diesel.is_some()
            || self.vendor_state_kerosene.is_some()
            || self.vendor_blocked_pump_kerosene.is_some()
            || self.vendor_bus_kerosene.is_some()
            || self.vendor_aviation_kerosene.is_some()
            || self.vendor_aviation_noncommercial.is_some()
            || self.vendor_aviation_nontaxable.is_some()
            || self.card_issuer.is_some())
            && uses.iter().flatten().all(|v| {
                (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(v.gallons))
                    && (0..=100_000_000_000).contains(&v.actual_cost)
            })
            && self.priced_uses().into_iter().all(|(use_line, rate)| {
                use_line.is_none_or(|v| v.actual_cost >= Self::line_cents(use_line, rate))
            })
            && self.line_2b_cost_cents() >= self.line_2b_cents()
            && self.line_5c_through_5e_cost_cents()
                >= self.line_5c_or_5d_cents(244)
                    + self.line_5c_or_5d_cents(219)
                    + self
                        .aviation_kerosene_foreign_trade
                        .as_ref()
                        .map_or(0, |value| value.lines()[2])
            && self.lpg_off_highway.as_ref().is_none_or(|v| v.valid(AlternativeFuelKind::Lpg, 183))
            && self.cng_off_highway.as_ref().is_none_or(|v| v.valid(AlternativeFuelKind::Cng, 183))
            && self.lng_off_highway.as_ref().is_none_or(|v| v.valid(AlternativeFuelKind::Lng, 243))
            && self
                .aviation_commercial
                .as_ref()
                .is_none_or(FederalCommercialAviationKerosene::valid)
            && self
                .aviation_kerosene_foreign_trade
                .as_ref()
                .is_none_or(FederalForeignTradeAviationKerosene::valid)
            && self.aviation_kerosene_farm.as_ref().is_none_or(FederalFarmAviationKerosene::valid)
            && self
                .aviation_gasoline_farm
                .as_ref()
                .is_none_or(FederalAviationGasolineFarmUse::valid)
            && self
                .aviation_gasoline_type_10
                .as_ref()
                .is_none_or(FederalAviationGasolineType10Uses::valid)
            && self
                .aviation_gasoline_foreign_trade
                .as_ref()
                .is_none_or(FederalAviationGasolineForeignTrade::valid)
            && self.bus_uses.as_ref().is_none_or(FederalBusFuelUses::valid)
            && self.other_uses.as_ref().is_none_or(FederalOtherFuelUses::valid)
            && self.emulsion_blender.as_ref().is_none_or(FederalEmulsionBlenderClaim::valid)
            && self
                .vendor_state_diesel
                .as_ref()
                .is_none_or(|claim| claim.valid(VENDOR_STATE_DIESEL_RATE))
            && self.vendor_bus_diesel.as_ref().is_none_or(FederalVendorBusClaim::valid)
            && self
                .vendor_state_kerosene
                .as_ref()
                .is_none_or(|claim| claim.valid(VENDOR_STATE_KEROSENE_RATE))
            && self
                .vendor_blocked_pump_kerosene
                .as_ref()
                .is_none_or(FederalBlockedPumpKeroseneClaim::valid)
            && self.vendor_bus_kerosene.as_ref().is_none_or(FederalVendorBusClaim::valid)
            && self.vendor_aviation_kerosene.as_ref().is_none_or(FederalVendorAviationClaim::valid)
            && self
                .vendor_aviation_noncommercial
                .as_ref()
                .is_none_or(FederalVendorNoncommercialAviationClaim::valid)
            && self
                .vendor_aviation_nontaxable
                .as_ref()
                .is_none_or(FederalVendorNontaxableAviationClaim::valid)
            && self.card_issuer.as_ref().is_none_or(FederalCardIssuerClaim::valid)
            && self.aviation_vendor_sale_references_unique()
    }

    pub(super) fn aviation_vendor_sale_references_unique(&self) -> bool {
        let mut references = BTreeSet::new();
        self.vendor_aviation_kerosene
            .iter()
            .flat_map(|claim| claim.sales.iter().map(|sale| sale.sale_reference.trim()))
            .chain(
                self.vendor_aviation_noncommercial
                    .iter()
                    .flat_map(|claim| claim.sales.iter().map(|sale| sale.sale_reference.trim())),
            )
            .chain(
                self.vendor_aviation_nontaxable
                    .iter()
                    .flat_map(|claim| claim.sales.iter().map(|sale| sale.sale_reference.trim())),
            )
            .chain(
                self.card_issuer
                    .iter()
                    .flat_map(|claim| claim.sales.iter().map(|sale| sale.sale_reference.trim())),
            )
            .all(|reference| references.insert(reference))
    }
}

impl FederalFuelTaxCredit {
    pub(crate) fn lines(&self) -> [i64; FUEL_LINE_COUNT] {
        self.fuels.lines()
    }

    pub(crate) fn labeled_lines(&self) -> impl Iterator<Item = (&'static str, i64)> {
        FUEL_LINE_NAMES.into_iter().zip(self.lines())
    }

    pub(crate) fn credit_cents(&self) -> i64 {
        self.fuels.credit_cents()
    }

    pub(crate) fn expense_recovery_cents(&self) -> i64 {
        // The issuer's credit is never a fuel-cost deduction recovery here.
        // Retained accrual issuer income has its own once-only path in receipts().
        self.credit_cents()
            - self.fuels.card_issuer.as_ref().map_or(0, |claim| claim.lines().iter().sum::<i64>())
    }

    pub(crate) fn valid(&self) -> bool {
        self.fuels.valid()
    }
}

/// Home-use undyed kerosene fuel tax credit — Form 4136 line 4a, type of use 8
/// This is the exception to Form 4136's ordinary business requirement. Only kerosene purchased away from a blocked pump and used in your home for heating, lighting or cooking qualifies here.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalHomeKerosene {
    /// Qualified fuel, including any fractional part
    #[libertas_number(min = 0.001, max = 2000000, step = 0.001)]
    #[libertas_physical_unit("gallon")]
    pub gallons: f64,
    /// Actual tax-paid kerosene cost from purchase records
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_cost: i64,
    /// You bought tax-paid undyed kerosene away from a blocked pump and used these gallons in your home for heating, lighting or cooking
    /// Your fuel receipts show it. Exclude any fuel used in a business, vehicle or another home. This is Form 4136 type of use 8, not an ordinary business-use claim.
    pub qualified: bool,
    /// No Form 8849, Form 8864, Form 720 or other claim covers these same gallons
    /// Check the Forms 8849 and 720 you filed.
    pub not_claimed_elsewhere: bool,
    /// Fuel purchase, payment of tax and home-use records are complete
    /// No leaves this credit out of this estimate.
    pub records_complete: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalHomeKerosene {}

impl FederalHomeKerosene {
    // No credit for fuel that does not qualify, is claimed elsewhere or has
    // incomplete records.
    pub(crate) fn allowed_cents(&self) -> i64 {
        if self.qualified && self.not_claimed_elsewhere && self.records_complete {
            self.credit_cents()
        } else {
            0
        }
    }
    pub(crate) fn credit_cents(&self) -> i64 {
        crate::kernel::ratio::fraction(
            crate::kernel::natural::thousandths(self.gallons),
            243,
            10_000,
        )
    }

    pub(crate) fn valid(&self) -> bool {
        (1..=2_000_000_000).contains(&crate::kernel::natural::thousandths(self.gallons))
            && (0..=100_000_000_000).contains(&self.actual_cost)
            && self.actual_cost >= self.credit_cents()
    }
}
