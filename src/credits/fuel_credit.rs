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

use crate::federal_model::form_4136_model::FederalFuelLines;
pub use crate::federal_model::form_4136_model::{
    FederalAviationNontaxableUse, FederalAviationVendorRegistration, FederalBlockedPumpAccess,
    FederalBlockedPumpKeroseneClaim, FederalBlockedPumpKeroseneSale, FederalCardIssuerClaim,
    FederalCardIssuerFuel, FederalCardIssuerSale, FederalCardVendorSettlement,
    FederalGovernmentFuelSale, FederalVendorAviationClaim, FederalVendorAviationSale,
    FederalVendorAviationTaxRate, FederalVendorBusClaim, FederalVendorBusSale,
    FederalVendorNoncommercialAviationClaim, FederalVendorNoncommercialAviationSale,
    FederalVendorNontaxableAviationClaim, FederalVendorNontaxableAviationSale,
    FederalVendorStateClaim, FederalVendorTaxRelief,
};
use alloc::vec::Vec;

// Answers and unknowns: a sale whose answers do not qualify
// leaves its claim, and a claim whose own answers do not hold leaves the
// return; unknowns::supported adds the notes. Remaining checks in
// `qualified` are record consistency and still refuse.
fn drop_sales<S>(sales: &mut Vec<S>, qualified: impl Fn(&S) -> bool) -> bool {
    let before = sales.len();
    sales.retain(|sale| qualified(sale));
    sales.len() != before
}

impl FederalFuelLines {
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
        claim.sales[0].gallons = 5.001;
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
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
