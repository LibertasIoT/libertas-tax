// Coverage: Form 4136 — Supported (audited 2026-09-24).
//   Lines: Lines 1a–1d, 2a–2d, 3a–3e, 4a–4f (incl. home kerosene type 8), 5a–5e, vendor lines
//   6a–6b, 7a–7c, 8a–8f, 11a–11h alternative fuels (GGE/DGE), 13a–13c card issuer, 14a–14b
//   emulsion, 15a blender, 16a–16b LUST exports, 17; partner K-1 box 20 code C (the only page-written
//   fuel record before B7a). Direct business and farm fuel use (FederalBusiness.fuel_tax_credit) and
//   prior-year cash-method recovery (FederalBusiness.cash_fuel_tax_recovery) are the fuel records of
//   the Business and Farm pages' "More" folds (B7a); home kerosene is on More credits. Not entered:
//   noncommercial/mixed aviation kerosene ("separate records"); no lines for expired mixture credits.
//   Evidence: M/fuel_credit.rs, federal_model.rs FederalFuelLines, federal_rules.rs,
//   partnership.rs; src/tests/credits/credits.rs alternative_fuel_physical_units_convert_to_gge_and_dge_on_return;
//   src/tests/farm/farm.rs farm_kerosene_fuel_uses_reach_form_4136_lines_4a_and_4b; src/tests/business/business.rs
//   home_kerosene_is_a_fuel_payment_without_business_income; IRS i4136, draft f4136, Pub 510.
//   Home kerosene page (u5-credits, 2026-09-28): More credits' "Federal tax on kerosene used in
//   your home" item (MoreCredit::HomeKerosene, stored in payments.home_kerosene; the refusal
//   RareSituation::HomeKerosene removed); rate $.243 re-verified on the 2026 draft line 4a;
//   KEROSENE_COST_BELOW_CREDIT both sides. Tests: src/tests/credits/other_taxes.rs
//   home_kerosene_through_the_more_credits_list, home_kerosene_cost_boundary,
//   home_kerosene_gallon_limit_boundary.

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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified gallons, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual fuel cost from the card transaction and vendor records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub retained_claimable_tax: i64,
    /// The issuer's documented reimbursement or consent arrangement with the ultimate vendor
    pub vendor_settlement: FederalCardVendorSettlement,
    /// The issuer holds the buyer's signed Model Certificate R covering this card and sale, with no reason to doubt it
    /// It expires two years after its effective date or when a replacement is provided, whichever is earlier.
    pub buyer_certificate_current: bool,
    /// No Form 720, Form 8849, Form 4136 or other claimant has already claimed these gallons
    pub not_claimed_elsewhere: bool,
}

impl FederalCardIssuerSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && valid_government_tin(&self.buyer_tin)
            && !self.buyer_address.trim().is_empty()
            && !self.card_account_reference.trim().is_empty()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && (0..=self.actual_fuel_cost).contains(&self.retained_claimable_tax)
            && self.buyer_tax_refund_on.is_none_or(|date| {
                !self.tax_not_collected_from_buyer
                    && (self.date..=20261231).contains(&date)
                    && crate::federal_rules::valid_date(date)
            })
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Keep each transaction, buyer certificate, original tax rate, cost and no-duplicate evidence.
    #[libertas_size(min = 1)]
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
            line.0 = line.0.checked_add(i64::from(sale.thousandths_of_gallon))?;
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified undyed fuel sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual cost of this fuel from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalGovernmentFuelSale {
    fn tin_valid(&self) -> bool {
        valid_government_tin(&self.buyer_tin)
    }

    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.buyer_name.trim().is_empty()
            && self.tin_valid()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Each entry must have separate sale, cost, certificate and no-duplicate-claim evidence.
    #[libertas_size(min = 1)]
    pub sales: Vec<FederalGovernmentFuelSale>,
    /// Purchase, excise-tax, sale and buyer-certificate records are complete
    /// This includes the government-unit name, TIN and gallons required for the line 6a or 7a statement.
    pub records_complete: bool,
}

impl FederalVendorStateClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales
            .iter()
            .try_fold(0_i64, |total, sale| total.checked_add(i64::from(sale.thousandths_of_gallon)))
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified undyed fuel sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual fuel cost from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalVendorBusSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && self.adult_seats_excluding_driver.is_none_or(|seats| (1..=1_000).contains(&seats))
            && (self.scheduled_regular_routes || self.adult_seats_excluding_driver.is_some())
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Each entry retains its own buyer, sale, waiver, tax, bus-use, cost and no-duplicate-claim evidence.
    #[libertas_size(min = 1)]
    pub sales: Vec<FederalVendorBusSale>,
    /// Purchase, excise-tax, sale and buyer-waiver records are complete
    /// Include each sale date, buyer name and address, qualified gallons and the signed Model Waiver N.
    pub records_complete: bool,
}

impl FederalVendorBusClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales
            .iter()
            .try_fold(0_i64, |total, sale| total.checked_add(i64::from(sale.thousandths_of_gallon)))
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified kerosene sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalVendorAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Each entry retains the sale, buyer, original tax rate, cost and no-duplicate-claim evidence.
    #[libertas_size(min = 1)]
    pub sales: Vec<FederalVendorAviationSale>,
    /// The vendor retained the fuel purchase, excise-tax, sale and signed Model Waiver L records
    pub records_complete: bool,
}

impl FederalVendorAviationClaim {
    fn line_totals(&self) -> Option<[(i64, i64); 2]> {
        self.sales.iter().try_fold([(0_i64, 0_i64); 2], |mut totals, sale| {
            let line = &mut totals[sale.line_index()];
            line.0 = line.0.checked_add(i64::from(sale.thousandths_of_gallon))?;
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified kerosene sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalVendorNoncommercialAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Keep each sale's buyer, gallons, original tax, cost and no-duplicate-claim evidence.
    #[libertas_size(min = 1)]
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Qualified kerosene sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual cost of these gallons from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalVendorNontaxableAviationSale {
    fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && !self.sale_reference.trim().is_empty()
            && !self.buyer_name.trim().is_empty()
            && !self.buyer_address.trim().is_empty()
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && if self.use_type == FederalAviationNontaxableUse::StateOrLocalGovernment {
                self.government_buyer_tin.as_ref().is_some_and(|tin| valid_government_tin(tin))
                    && self.no_government_credit_card.is_some()
            } else {
                self.government_buyer_tin.is_none() && self.no_government_credit_card.is_none()
            }
    }

    fn qualified(&self) -> bool {
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
    /// ----
    /// Each entry holds one distinct sale, tax rate, use type, buyer document and actual cost.
    #[libertas_size(min = 1)]
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
                line.0 = line.0.checked_add(i64::from(sale.thousandths_of_gallon))?;
                line.1 = line.1.checked_add(sale.actual_fuel_cost)?;
                let foreign_trade = if sale.use_type == FederalAviationNontaxableUse::ForeignTrade {
                    foreign_trade.checked_add(i64::from(sale.thousandths_of_gallon))?
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
                totals.0.checked_add(i64::from(sale.thousandths_of_gallon))?,
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Undyed nonaviation kerosene sold, in thousandths of a gallon
    #[libertas_number(min = 1)]
    pub thousandths_of_gallon: u32,
    /// Actual cost of this fuel from the vendor's records, including federal tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

impl FederalBlockedPumpKeroseneSale {
    fn valid(&self, locked: bool) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && (1..=2_000_000_000).contains(&self.thousandths_of_gallon)
            && (0..=100_000_000_000).contains(&self.actual_fuel_cost)
            && self.buyer_issue(locked).is_none()
    }

    fn buyer_issue(&self, locked: bool) -> Option<crate::validation::ValidationIssue> {
        let required = locked && self.thousandths_of_gallon > 5_000;
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

    fn qualified(&self) -> bool {
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
    /// ----
    /// Each sale retains its own tax, use, quantity and no-duplicate-claim evidence.
    #[libertas_size(min = 1)]
    pub sales: Vec<FederalBlockedPumpKeroseneSale>,
    /// The vendor retained complete purchase, excise-tax, pump and sale records
    /// Locked-pump sales over five gallons also need the buyer's name and address.
    pub records_complete: bool,
}

impl FederalBlockedPumpKeroseneClaim {
    fn total_thousandths(&self) -> Option<i64> {
        self.sales
            .iter()
            .try_fold(0_i64, |total, sale| total.checked_add(i64::from(sale.thousandths_of_gallon)))
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

// Answers and unknowns: a sale whose answers do not qualify
// leaves its claim, and a claim whose own answers do not hold leaves the
// return; unknowns::supported adds the notes. Remaining checks in
// `qualified` are record consistency and still refuse.
fn drop_sales<S>(sales: &mut Vec<S>, qualified: impl Fn(&S) -> bool) -> bool {
    let before = sales.len();
    sales.retain(|sale| qualified(sale));
    sales.len() != before
}

impl crate::FederalFuelLines {
    /// (a sale was left out, a claim was left out)
    pub(crate) fn drop_unsupported(&mut self) -> (bool, bool) {
        let (mut sales, mut claims) = (false, false);
        macro_rules! claim {
            ($field:ident, $sale:expr, $answers:expr) => {
                if let Some(claim) = self.$field.as_mut() {
                    sales |= drop_sales(&mut claim.sales, $sale);
                    let answers = $answers(&*claim);
                    claims |= !claim.sales.is_empty() && !answers;
                    if claim.sales.is_empty() || !answers {
                        self.$field = None;
                    }
                }
            };
        }
        let vendor = |registration: bool, records: bool| registration && records;
        claim!(card_issuer, FederalCardIssuerSale::qualified, |c: &FederalCardIssuerClaim| {
            c.registration_active
                && c.records_complete
                && c.settlement_outside_business_income_and_expenses
                && (!c.has_retained_tax()
                    || c.retained_accrual_accounting_confirmed == Some(true)
                    || c.retained_cash_accounting_confirmed == Some(true))
        });
        claim!(
            vendor_state_diesel,
            FederalGovernmentFuelSale::qualified,
            |c: &FederalVendorStateClaim| vendor(c.registration_active, c.records_complete)
        );
        claim!(
            vendor_state_kerosene,
            FederalGovernmentFuelSale::qualified,
            |c: &FederalVendorStateClaim| vendor(c.registration_active, c.records_complete)
        );
        claim!(vendor_bus_diesel, FederalVendorBusSale::qualified, |c: &FederalVendorBusClaim| {
            vendor(c.registration_active, c.records_complete)
        });
        claim!(
            vendor_bus_kerosene,
            FederalVendorBusSale::qualified,
            |c: &FederalVendorBusClaim| vendor(c.registration_active, c.records_complete)
        );
        claim!(
            vendor_aviation_kerosene,
            FederalVendorAviationSale::qualified,
            |c: &FederalVendorAviationClaim| vendor(c.registration_active, c.records_complete)
        );
        claim!(
            vendor_aviation_noncommercial,
            FederalVendorNoncommercialAviationSale::qualified,
            |c: &FederalVendorNoncommercialAviationClaim| {
                vendor(c.registration_active, c.records_complete)
            }
        );
        if let Some(claim) = self.vendor_aviation_nontaxable.as_mut() {
            let (ua, uv) = (
                claim.ua_registration.as_ref().is_some_and(|r| r.active),
                claim.uv_registration.as_ref().is_some_and(|r| r.active),
            );
            sales |= drop_sales(&mut claim.sales, |sale| {
                sale.qualified()
                    && if sale.use_type == FederalAviationNontaxableUse::StateOrLocalGovernment {
                        uv
                    } else {
                        ua
                    }
            });
            claims |= !claim.sales.is_empty() && !claim.records_complete;
            if claim.sales.is_empty() || !claim.records_complete {
                self.vendor_aviation_nontaxable = None;
            }
        }
        claim!(
            vendor_blocked_pump_kerosene,
            FederalBlockedPumpKeroseneSale::qualified,
            FederalBlockedPumpKeroseneClaim::qualified
        );
        (sales, claims)
    }

    /// Some fuel line remains to claim.
    pub(crate) fn has_lines(&self) -> bool {
        self.uses().iter().any(Option::is_some)
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
            || self.card_issuer.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn sale(thousandths: u32, cost: i64) -> FederalGovernmentFuelSale {
        FederalGovernmentFuelSale {
            date: 20260615,
            buyer_name: "Synthetic State Roads Agency".into(),
            buyer_tin: "12-3456789".into(),
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: cost,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_244_undyed_fuel: true,
            exclusive_government_use: true,
            no_government_credit_card: true,
            buyer_certificate_current: true,
            not_claimed_elsewhere: true,
        }
    }

    fn sample_claim() -> FederalVendorStateClaim {
        FederalVendorStateClaim {
            uv_registration_number: "UV-SYNTHETIC-637".into(),
            registration_active: true,
            sales: vec![sale(100_000, 30_000)],
            records_complete: true,
        }
    }

    #[test]
    fn government_diesel_vendor_credit_uses_line_level_rounding() {
        let mut claim = sample_claim();
        assert!(claim.valid(243) && claim.qualified());
        assert_eq!(claim.line_cents(243), 2_430);

        claim.sales[0].tax_relief = FederalVendorTaxRelief::TaxRepaid;
        assert!(claim.qualified());
        claim.sales[0].tax_relief = FederalVendorTaxRelief::BuyerWrittenConsent;
        assert!(claim.qualified());

        claim.sales = vec![sale(21, 1), sale(21, 1)];
        assert!(claim.valid(243) && claim.qualified());
        assert_eq!(claim.line_cents(243), 1);
    }

    #[test]
    fn diesel_and_kerosene_vendor_lines_round_independently() {
        let mut claim = sample_claim();
        claim.sales = vec![sale(21, 1)];
        let lines = crate::FederalFuelLines {
            vendor_state_diesel: Some(claim.clone()),
            vendor_state_kerosene: Some(claim),
            ..crate::FederalFuelLines::default()
        };
        assert!(lines.valid() && lines.qualified());
        assert_eq!(lines.labeled_lines().find(|(name, _)| *name == "6a"), Some(("6a", 1)));
        assert_eq!(lines.labeled_lines().find(|(name, _)| *name == "7a"), Some(("7a", 1)));
        assert_eq!(lines.credit_cents(), 2);
    }

    #[test]
    fn vendor_claim_requires_registration_government_evidence_and_separate_cost() {
        let mut claim = sample_claim();
        claim.uv_registration_number = "UA-SYNTHETIC-637".into();
        assert!(!claim.valid(243));

        claim = sample_claim();
        claim.sales[0].buyer_tin = "invalid".into();
        assert!(!claim.valid(243));

        claim = sample_claim();
        claim.sales[0].date = 20270101;
        assert!(!claim.valid(243));

        claim = sample_claim();
        claim.sales[0].actual_fuel_cost = 2_429;
        assert!(!claim.valid(243));

        claim = sample_claim();
        claim.sales[0].no_government_credit_card = false;
        assert!(!claim.qualified());

        claim = sample_claim();
        claim.sales[0].buyer_certificate_current = false;
        assert!(!claim.qualified());

        claim = sample_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());

        claim = sample_claim();
        claim.sales[0].tax_relief = FederalVendorTaxRelief::NotSure;
        assert!(!claim.qualified());

        claim = sample_claim();
        claim.registration_active = false;
        assert!(!claim.qualified());
    }

    fn bus_sale(thousandths: u32, cost: i64) -> FederalVendorBusSale {
        FederalVendorBusSale {
            date: 20260615,
            buyer_name: "Synthetic Regional Bus Operator".into(),
            buyer_address: "100 Sample Avenue".into(),
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: cost,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_244_undyed_nonaviation: true,
            compensated_passenger_service: true,
            available_to_general_public: true,
            scheduled_regular_routes: true,
            adult_seats_excluding_driver: None,
            not_school_or_qualified_local_bus: true,
            buyer_waiver_current: true,
            not_claimed_elsewhere: true,
        }
    }

    fn bus_claim() -> FederalVendorBusClaim {
        FederalVendorBusClaim {
            ub_registration_number: "UB-SYNTHETIC-637".into(),
            registration_active: true,
            sales: vec![bus_sale(100_000, 30_000)],
            records_complete: true,
        }
    }

    #[test]
    fn ub_vendor_diesel_and_kerosene_lines_round_independently() {
        let mut claim = bus_claim();
        assert!(claim.valid() && claim.qualified());
        assert_eq!(claim.line_cents(), 1_700);
        claim.sales = vec![bus_sale(29, 1), bus_sale(29, 1)];
        assert!(claim.valid() && claim.qualified());
        assert_eq!(claim.line_cents(), 1);

        let lines = crate::FederalFuelLines {
            vendor_bus_diesel: Some(claim.clone()),
            vendor_bus_kerosene: Some(claim),
            ..crate::FederalFuelLines::default()
        };
        assert!(lines.valid() && lines.qualified());
        assert_eq!(lines.labeled_lines().find(|(name, _)| *name == "6b"), Some(("6b", 1)));
        assert_eq!(lines.labeled_lines().find(|(name, _)| *name == "7c"), Some(("7c", 1)));
        assert_eq!(lines.credit_cents(), 2);
    }

    #[test]
    fn ub_vendor_requires_registration_waiver_and_public_bus_facts() {
        let mut claim = bus_claim();
        claim.ub_registration_number = "UV-SYNTHETIC-637".into();
        assert!(!claim.valid());
        claim = bus_claim();
        claim.sales[0].buyer_address = " ".into();
        assert!(!claim.valid());
        claim = bus_claim();
        claim.sales[0].date = 20270101;
        assert!(!claim.valid());
        claim = bus_claim();
        claim.sales[0].actual_fuel_cost = 1_699;
        assert!(!claim.valid());
        claim = bus_claim();
        claim.sales[0].buyer_waiver_current = false;
        assert!(!claim.qualified());
        claim = bus_claim();
        claim.sales[0].available_to_general_public = false;
        assert!(!claim.qualified());
        claim = bus_claim();
        claim.sales[0].not_school_or_qualified_local_bus = false;
        assert!(!claim.qualified());
        claim = bus_claim();
        claim.sales[0].tax_relief = FederalVendorTaxRelief::NotSure;
        assert!(!claim.qualified());
        claim = bus_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());
        claim = bus_claim();
        claim.registration_active = false;
        assert!(!claim.qualified());
    }

    #[test]
    fn ub_vendor_nonscheduled_bus_needs_twenty_adult_seats() {
        let mut claim = bus_claim();
        claim.sales[0].scheduled_regular_routes = false;
        assert!(!claim.valid());
        claim.sales[0].adult_seats_excluding_driver = Some(19);
        assert!(claim.valid());
        assert!(!claim.qualified());
        claim.sales[0].adult_seats_excluding_driver = Some(20);
        assert!(claim.qualified());
        claim.sales[0].compensated_passenger_service = false;
        assert!(!claim.qualified());
    }

    fn blocked_pump_sale(thousandths: u32) -> FederalBlockedPumpKeroseneSale {
        FederalBlockedPumpKeroseneSale {
            date: 20260615,
            buyer_name: "".into(),
            buyer_address: "".into(),
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: 30_000,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_244_undyed_nonaviation: true,
            buyer_nontaxable_use: true,
            no_reason_to_doubt_use: true,
            not_claimed_elsewhere: true,
        }
    }

    fn blocked_pump_claim() -> FederalBlockedPumpKeroseneClaim {
        FederalBlockedPumpKeroseneClaim {
            up_registration_number: "UP-SYNTHETIC-637".into(),
            registration_active: true,
            pump_fixed: true,
            retail_sales: true,
            pump_notice_displayed: true,
            pump_access: FederalBlockedPumpAccess::CannotFillHighwayVehicleOrTrain,
            sales: vec![blocked_pump_sale(100_000)],
            records_complete: true,
        }
    }

    #[test]
    fn blocked_pump_line_rounds_combined_gallons_and_checks_cost() {
        let mut claim = blocked_pump_claim();
        assert!(claim.valid() && claim.qualified());
        assert_eq!(claim.line_cents(), 2_430);

        claim.sales = vec![blocked_pump_sale(21), blocked_pump_sale(21)];
        assert_eq!(claim.line_cents(), 1);
        assert!(claim.valid());

        claim.sales[0].actual_fuel_cost = 0;
        claim.sales[1].actual_fuel_cost = 0;
        assert!(!claim.valid());
    }

    #[test]
    fn locked_pump_over_five_gallons_needs_buyer_record() {
        let mut claim = blocked_pump_claim();
        claim.pump_access = FederalBlockedPumpAccess::LockedBetweenSales;
        claim.sales = vec![blocked_pump_sale(5_000)];
        assert!(claim.valid() && claim.qualified());
        claim.sales[0].thousandths_of_gallon = 5_001;
        assert!(!claim.valid());
        claim.sales[0].buyer_name = "Synthetic Buyer".into();
        assert!(!claim.valid());
        claim.sales[0].buyer_address = "100 Sample Street".into();
        assert!(claim.valid());
        claim.sales[0].buyer_address = " ".into();
        assert!(!claim.valid());
    }

    #[test]
    fn blocked_pump_claim_requires_registration_pump_and_sale_evidence() {
        let mut claim = blocked_pump_claim();
        claim.up_registration_number = "UV-SYNTHETIC-637".into();
        assert!(!claim.valid());
        claim = blocked_pump_claim();
        claim.retail_sales = false;
        assert!(!claim.qualified());
        claim = blocked_pump_claim();
        claim.pump_notice_displayed = false;
        assert!(!claim.qualified());
        claim = blocked_pump_claim();
        claim.pump_access = FederalBlockedPumpAccess::NotSure;
        assert!(!claim.qualified());
        claim = blocked_pump_claim();
        claim.sales[0].no_reason_to_doubt_use = false;
        assert!(!claim.qualified());
        claim = blocked_pump_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());
    }

    fn aviation_sale(reference: &str, thousandths: u32) -> FederalVendorAviationSale {
        FederalVendorAviationSale {
            sale_reference: reference.into(),
            date: 20260615,
            buyer_name: "Synthetic Commercial Air Carrier".into(),
            buyer_address: "100 Airport Road".into(),
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: 30_000,
            original_tax_rate: FederalVendorAviationTaxRate::TaxedAt219,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            commercial_aviation_other_than_foreign_trade: true,
            taxed_at_stated_rate: true,
            buyer_waiver_current: true,
            not_claimed_elsewhere: true,
        }
    }

    fn aviation_claim() -> FederalVendorAviationClaim {
        FederalVendorAviationClaim {
            ua_registration_number: "UA-SYNTHETIC-637".into(),
            registration_active: true,
            sales: vec![aviation_sale("sale-a", 100_000)],
            records_complete: true,
        }
    }

    #[test]
    fn aviation_vendor_8a_8b_round_each_tax_band_once() {
        let mut claim = aviation_claim();
        claim.sales = vec![
            aviation_sale("a-1", 29),
            aviation_sale("a-2", 29),
            FederalVendorAviationSale {
                original_tax_rate: FederalVendorAviationTaxRate::TaxedAt244,
                ..aviation_sale("b-1", 25)
            },
            FederalVendorAviationSale {
                original_tax_rate: FederalVendorAviationTaxRate::TaxedAt244,
                ..aviation_sale("b-2", 25)
            },
        ];
        assert_eq!(claim.lines(), [1, 1]);
        assert!(claim.valid() && claim.qualified());
        let fuels = crate::FederalFuelLines {
            vendor_aviation_kerosene: Some(claim),
            ..crate::FederalFuelLines::default()
        };
        assert_eq!(fuels.labeled_lines().find(|(name, _)| *name == "8a"), Some(("8a", 1)));
        assert_eq!(fuels.labeled_lines().find(|(name, _)| *name == "8b"), Some(("8b", 1)));
    }

    #[test]
    fn aviation_vendor_requires_ua_waiver_sale_records_and_separate_cost() {
        let mut claim = aviation_claim();
        assert_eq!(claim.lines(), [1_750, 0]);
        assert!(claim.valid() && claim.qualified());

        claim.ua_registration_number = "UV-SYNTHETIC-637".into();
        assert!(!claim.valid());
        claim = aviation_claim();
        claim.sales[0].actual_fuel_cost = 1_749;
        assert!(!claim.valid());
        claim = aviation_claim();
        claim.sales.push(aviation_sale("sale-a", 100_000));
        assert!(!claim.valid());
        claim = aviation_claim();
        claim.sales[0].buyer_address = " ".into();
        assert!(!claim.valid());
        claim = aviation_claim();
        claim.sales[0].buyer_waiver_current = false;
        assert!(!claim.qualified());
        claim = aviation_claim();
        claim.sales[0].taxed_at_stated_rate = false;
        assert!(!claim.qualified());
        claim = aviation_claim();
        claim.sales[0].commercial_aviation_other_than_foreign_trade = false;
        assert!(!claim.qualified());
        claim = aviation_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());
    }

    fn noncommercial_aviation_sale(
        reference: &str,
        thousandths: u32,
    ) -> FederalVendorNoncommercialAviationSale {
        FederalVendorNoncommercialAviationSale {
            sale_reference: reference.into(),
            date: 20260615,
            buyer_name: "Synthetic Private Aircraft Operator".into(),
            buyer_address: "200 Airport Road".into(),
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: 30_000,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_at_244: true,
            nonexempt_noncommercial_use: true,
            buyer_certificate_current: true,
            not_claimed_elsewhere: true,
        }
    }

    fn noncommercial_aviation_claim() -> FederalVendorNoncommercialAviationClaim {
        FederalVendorNoncommercialAviationClaim {
            ua_registration_number: "UA-SYNTHETIC-637".into(),
            registration_active: true,
            sales: vec![noncommercial_aviation_sale("sale-8c", 100_000)],
            records_complete: true,
        }
    }

    #[test]
    fn noncommercial_vendor_8c_rounds_its_own_gallons_once() {
        let mut claim = noncommercial_aviation_claim();
        assert_eq!(claim.line_cents(), 250);
        claim.sales =
            vec![noncommercial_aviation_sale("c-1", 101), noncommercial_aviation_sale("c-2", 101)];
        assert_eq!(claim.line_cents(), 1);
        assert!(claim.valid() && claim.qualified());
        let fuels = crate::FederalFuelLines {
            vendor_aviation_noncommercial: Some(claim),
            ..crate::FederalFuelLines::default()
        };
        assert_eq!(fuels.labeled_lines().find(|(name, _)| *name == "8c"), Some(("8c", 1)));
        assert_eq!(fuels.labeled_lines().find(|(name, _)| *name == "8a"), Some(("8a", 0)));
    }

    #[test]
    fn noncommercial_vendor_requires_ua_certificate_tax_and_distinct_sales() {
        let mut claim = noncommercial_aviation_claim();
        assert!(claim.valid() && claim.qualified());

        claim.ua_registration_number = "UV-SYNTHETIC-637".into();
        assert!(!claim.valid());
        claim = noncommercial_aviation_claim();
        claim.sales[0].actual_fuel_cost = 249;
        assert!(!claim.valid());
        claim = noncommercial_aviation_claim();
        claim.sales.push(noncommercial_aviation_sale("sale-8c", 50_000));
        assert!(!claim.valid());
        claim = noncommercial_aviation_claim();
        claim.sales[0].buyer_address = " ".into();
        assert!(!claim.valid());
        claim = noncommercial_aviation_claim();
        claim.sales[0].date = 20260229;
        assert!(!claim.valid());
        claim = noncommercial_aviation_claim();
        claim.sales[0].buyer_certificate_current = false;
        assert!(!claim.qualified());
        claim = noncommercial_aviation_claim();
        claim.sales[0].taxed_at_244 = false;
        assert!(!claim.qualified());
        claim = noncommercial_aviation_claim();
        claim.sales[0].nonexempt_noncommercial_use = false;
        assert!(!claim.qualified());
        claim = noncommercial_aviation_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());
        claim = noncommercial_aviation_claim();
        claim.registration_active = false;
        assert!(!claim.qualified());
    }

    #[test]
    fn commercial_and_noncommercial_vendor_lines_cannot_reuse_one_sale_reference() {
        let mut noncommercial = noncommercial_aviation_claim();
        noncommercial.sales[0].sale_reference = "sale-a".into();
        let mut fuels = crate::FederalFuelLines {
            vendor_aviation_kerosene: Some(aviation_claim()),
            vendor_aviation_noncommercial: Some(noncommercial),
            ..crate::FederalFuelLines::default()
        };
        assert!(!fuels.valid());
        fuels.vendor_aviation_noncommercial.as_mut().unwrap().sales[0].sale_reference =
            "sale-8c".into();
        assert!(fuels.valid());
    }

    fn nontaxable_aviation_sale(
        reference: &str,
        use_type: FederalAviationNontaxableUse,
        original_tax_rate: FederalVendorAviationTaxRate,
        thousandths: u32,
    ) -> FederalVendorNontaxableAviationSale {
        let government = use_type == FederalAviationNontaxableUse::StateOrLocalGovernment;
        FederalVendorNontaxableAviationSale {
            sale_reference: reference.into(),
            date: 20260615,
            buyer_name: "Synthetic Aviation Buyer".into(),
            buyer_address: "300 Airport Road".into(),
            government_buyer_tin: government.then(|| "12-3456789".into()),
            use_type,
            original_tax_rate,
            thousandths_of_gallon: thousandths,
            actual_fuel_cost: 30_000,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_at_stated_rate: true,
            qualifying_nontaxable_use: true,
            buyer_document_current: true,
            no_government_credit_card: government.then_some(true),
            not_claimed_elsewhere: true,
        }
    }

    fn nontaxable_aviation_claim() -> FederalVendorNontaxableAviationClaim {
        FederalVendorNontaxableAviationClaim {
            ua_registration: Some(FederalAviationVendorRegistration {
                number: "UA-SYNTHETIC-637".into(),
                active: true,
            }),
            uv_registration: None,
            sales: vec![nontaxable_aviation_sale(
                "sale-8d",
                FederalAviationNontaxableUse::Farming,
                FederalVendorAviationTaxRate::TaxedAt244,
                100_000,
            )],
            records_complete: true,
        }
    }

    #[test]
    fn nontaxable_aviation_vendor_rounds_8d_8e_and_8f_once_each() {
        let mut claim = nontaxable_aviation_claim();
        claim.sales = vec![
            nontaxable_aviation_sale(
                "d1",
                FederalAviationNontaxableUse::Farming,
                FederalVendorAviationTaxRate::TaxedAt244,
                11,
            ),
            nontaxable_aviation_sale(
                "d2",
                FederalAviationNontaxableUse::AircraftMuseum,
                FederalVendorAviationTaxRate::TaxedAt244,
                11,
            ),
            nontaxable_aviation_sale(
                "e1",
                FederalAviationNontaxableUse::BloodCollector,
                FederalVendorAviationTaxRate::TaxedAt219,
                12,
            ),
            nontaxable_aviation_sale(
                "e2",
                FederalAviationNontaxableUse::MilitaryAircraft,
                FederalVendorAviationTaxRate::TaxedAt219,
                12,
            ),
            nontaxable_aviation_sale(
                "foreign1",
                FederalAviationNontaxableUse::ForeignTrade,
                FederalVendorAviationTaxRate::TaxedAt244,
                2_500,
            ),
            nontaxable_aviation_sale(
                "foreign2",
                FederalAviationNontaxableUse::ForeignTrade,
                FederalVendorAviationTaxRate::TaxedAt244,
                2_500,
            ),
        ];
        assert_eq!(claim.lines(), [122, 1, 1]);
        assert!(claim.valid() && claim.qualified());
        let fuels = crate::FederalFuelLines {
            vendor_aviation_nontaxable: Some(claim),
            ..crate::FederalFuelLines::default()
        };
        for (name, cents) in [("8d", 122), ("8e", 1), ("8f", 1)] {
            assert_eq!(fuels.labeled_lines().find(|(line, _)| *line == name), Some((name, cents)));
        }
    }

    #[test]
    fn nontaxable_aviation_vendor_requires_correct_registration_documents_and_use() {
        let mut claim = nontaxable_aviation_claim();
        assert_eq!(claim.lines(), [2_430, 0, 0]);
        assert!(claim.valid() && claim.qualified());
        claim.ua_registration.as_mut().unwrap().number = "UV-SYNTHETIC-637".into();
        assert!(!claim.valid());
        claim = nontaxable_aviation_claim();
        claim.ua_registration.as_mut().unwrap().active = false;
        assert!(!claim.qualified());
        claim = nontaxable_aviation_claim();
        claim.sales[0].buyer_document_current = false;
        assert!(!claim.qualified());
        claim = nontaxable_aviation_claim();
        claim.sales[0].tax_relief = FederalVendorTaxRelief::NotSure;
        assert!(!claim.qualified());
        claim = nontaxable_aviation_claim();
        claim.sales[0].qualifying_nontaxable_use = false;
        assert!(!claim.qualified());
        claim = nontaxable_aviation_claim();
        claim.sales[0].not_claimed_elsewhere = false;
        assert!(!claim.qualified());
        claim = nontaxable_aviation_claim();
        claim.sales[0].actual_fuel_cost = 2_429;
        assert!(!claim.valid());
        claim = nontaxable_aviation_claim();
        claim.sales[0].date = 20260229;
        assert!(!claim.valid());
    }

    #[test]
    fn nontaxable_government_aviation_requires_uv_certificate_tin_and_no_card() {
        let mut claim = nontaxable_aviation_claim();
        claim.sales[0] = nontaxable_aviation_sale(
            "government",
            FederalAviationNontaxableUse::StateOrLocalGovernment,
            FederalVendorAviationTaxRate::TaxedAt219,
            100_000,
        );
        assert_eq!(claim.lines(), [0, 2_180, 0]);
        assert!(!claim.valid()); // UA registration cannot support government type 14.
        claim.uv_registration = Some(FederalAviationVendorRegistration {
            number: "UV-SYNTHETIC-637".into(),
            active: true,
        });
        assert!(claim.valid() && claim.qualified());
        claim.sales[0].government_buyer_tin = None;
        assert!(!claim.valid());
        claim.sales[0].government_buyer_tin = Some("not-a-tin".into());
        assert!(!claim.valid());
        claim.sales[0].government_buyer_tin = Some("12-3456789".into());
        claim.sales[0].no_government_credit_card = Some(false);
        assert!(!claim.qualified());
        claim.sales[0].no_government_credit_card = Some(true);
        claim.uv_registration.as_mut().unwrap().active = false;
        assert!(!claim.qualified());
    }

    #[test]
    fn aviation_vendor_claims_cannot_reuse_sale_reference_across_registration_numbers() {
        let mut nontaxable = nontaxable_aviation_claim();
        nontaxable.sales[0].sale_reference = "sale-a".into();
        let mut fuels = crate::FederalFuelLines {
            vendor_aviation_kerosene: Some(aviation_claim()),
            vendor_aviation_noncommercial: Some(noncommercial_aviation_claim()),
            vendor_aviation_nontaxable: Some(nontaxable),
            ..crate::FederalFuelLines::default()
        };
        assert!(!fuels.valid());
        fuels.vendor_aviation_nontaxable.as_mut().unwrap().sales[0].sale_reference =
            "sale-8d".into();
        assert!(fuels.valid());
        fuels.vendor_aviation_noncommercial.as_mut().unwrap().ua_registration_number =
            "UA-OTHER-637".into();
        fuels.vendor_aviation_noncommercial.as_mut().unwrap().sales[0].sale_reference =
            "sale-a".into();
        assert!(!fuels.valid()); // A different UA registration does not make the same sale unique.
    }
}
