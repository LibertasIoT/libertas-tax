use crate::tests::*;

pub(in crate::tests) fn fuel_use(thousandths: u32, actual_cost: i64) -> FederalFuelUse {
    FederalFuelUse {
        gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
        actual_cost,
        qualified: true,
        not_claimed_elsewhere: true,
    }
}

pub(in crate::tests) fn cash_fuel_recovery(amount: i64) -> FederalCashFuelRecovery {
    FederalCashFuelRecovery {
        amount,
        event: FederalCashFuelRecoveryEvent::OriginalReturnFiled,
        occurred_in_year: true,
        full_tax_benefit: true,
        partial_prior_year_benefit: None,
        niit_recovery_excluded: true,
        not_reported_elsewhere: true,
    }
}

pub(in crate::tests) fn partial_cash_fuel_recovery() -> FederalCashFuelRecovery {
    let mut recovery = cash_fuel_recovery(500_000);
    recovery.full_tax_benefit = false;
    recovery.partial_prior_year_benefit = Some(FederalPartialCashFuelTaxBenefit {
        worksheet_reference:
            "Synthetic filed 2025 Form 1040, Schedule C, fuel ledger and regular-tax comparison"
                .into(),
        original_prior_year_line_11b: 1_200_000,
        original_prior_year_line_14: 1_575_000,
        originally_deducted_fuel_tax: 500_000,
        refigured_prior_year_line_11b: 1_664_700,
        refigured_prior_year_line_14: 1_592_900,
        refigured_prior_year_line_16: 7_300,
        original_standard_only_deduction: true,
        comparison_complete: true,
        active_unlimited_deduction: true,
        no_prior_amt: true,
        no_prior_credits_or_carryovers: true,
    });
    recovery
}

pub(in crate::tests) fn partial_partner_fuel_recovery() -> PartnerPriorFuelCredit {
    let cash = partial_cash_fuel_recovery();
    PartnerPriorFuelCredit {
        claimed_credit: cash.amount,
        full_prior_tax_benefit: false,
        partial_prior_year_benefit: cash.partial_prior_year_benefit,
        ..super::k1res::prior_fuel_source()
    }
}

pub(in crate::tests) fn alternative_fuel_use(
    quantity: FederalAlternativeFuelQuantity,
) -> FederalAlternativeFuelUse {
    FederalAlternativeFuelUse {
        quantity,
        actual_cost: 30_000,
        qualified: true,
        not_claimed_elsewhere: true,
    }
}

pub(in crate::tests) fn farm_aviation_kerosene_claim() -> FederalFarmAviationKerosene {
    FederalFarmAviationKerosene {
        taxed_at_244_application: Some(fuel_use(100_000, 20_000)),
        taxed_at_244_direct_flight: None,
        taxed_at_219_application: None,
        taxed_at_219_direct_flight: Some(fuel_use(100_000, 20_000)),
        applicator_was_ultimate_purchaser: true,
        no_waiver: true,
        distinct_gallons: true,
    }
}

pub(in crate::tests) fn aviation_gasoline_type_10_uses() -> FederalAviationGasolineType10Uses {
    FederalAviationGasolineType10Uses {
        helicopter_mineral_transport: Some(fuel_use(100_000, 20_000)),
        helicopter_tree_work: Some(fuel_use(100_000, 20_000)),
        helicopter_emergency_medical: Some(fuel_use(100_000, 20_000)),
        fixed_wing_tree_work: Some(fuel_use(100_000, 20_000)),
        fixed_wing_emergency_medical: Some(fuel_use(100_000, 20_000)),
        mineral_and_tree_airport_condition: Some(true),
        fixed_wing_acute_care_condition: Some(true),
        distinct_gallons: true,
    }
}

pub(in crate::tests) fn farm_aviation_gasoline(
    application: u32,
    direct_flight: Option<u32>,
) -> FederalAviationGasolineFarmUse {
    FederalAviationGasolineFarmUse {
        aerial_application: Some(fuel_use(application, 20_000)),
        direct_airfield_farm_flight: direct_flight.map(|gallons| fuel_use(gallons, 20_000)),
        applicator_was_ultimate_purchaser: true,
        distinct_gallons: true,
    }
}

pub(in crate::tests) fn government_vendor_sale(thousandths: u32) -> FederalGovernmentFuelSale {
    FederalGovernmentFuelSale {
        date: 20260615,
        buyer_name: "Synthetic State Roads Agency".into(),
        buyer_tin: "12-3456789".into(),
        gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
        actual_fuel_cost: 30_000,
        tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
        taxed_244_undyed_fuel: true,
        exclusive_government_use: true,
        no_government_credit_card: true,
        buyer_certificate_current: true,
        not_claimed_elsewhere: true,
    }
}

pub(in crate::tests) fn government_vendor_claim(thousandths: u32) -> FederalVendorStateClaim {
    FederalVendorStateClaim {
        uv_registration_number: "UV-SYNTHETIC-637".into(),
        registration_active: true,
        sales: vec![government_vendor_sale(thousandths)],
        records_complete: true,
    }
}

pub(in crate::tests) fn blocked_pump_vendor_claim(
    thousandths: u32,
    access: FederalBlockedPumpAccess,
) -> FederalBlockedPumpKeroseneClaim {
    let locked = access == FederalBlockedPumpAccess::LockedBetweenSales;
    FederalBlockedPumpKeroseneClaim {
        up_registration_number: "UP-SYNTHETIC-637".into(),
        registration_active: true,
        pump_fixed: true,
        retail_sales: true,
        pump_notice_displayed: true,
        pump_access: access,
        sales: vec![FederalBlockedPumpKeroseneSale {
            date: 20260615,
            buyer_name: (if locked { "Synthetic Buyer" } else { "" }).into(),
            buyer_address: (if locked { "100 Example Street" } else { "" }).into(),
            gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
            actual_fuel_cost: 30_000,
            tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
            taxed_244_undyed_nonaviation: true,
            buyer_nontaxable_use: true,
            no_reason_to_doubt_use: true,
            not_claimed_elsewhere: true,
        }],
        records_complete: true,
    }
}

pub(in crate::tests) fn bus_vendor_sale(thousandths: u32) -> FederalVendorBusSale {
    FederalVendorBusSale {
        date: 20260615,
        buyer_name: "Synthetic Bus Operator".into(),
        buyer_address: "100 Sample Avenue".into(),
        gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
        actual_fuel_cost: 30_000,
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

pub(in crate::tests) fn card_issuer_sale(
    reference: &str,
    fuel: FederalCardIssuerFuel,
    thousandths: u32,
) -> FederalCardIssuerSale {
    FederalCardIssuerSale {
        sale_reference: reference.into(),
        date: 20260615,
        buyer_name: "Synthetic City Government".into(),
        buyer_tin: "12-3456789".into(),
        buyer_address: "1 Test Plaza".into(),
        card_account_reference: "TEST-GOV-CARD-1".into(),
        fuel,
        gallons: crate::kernel::natural::natural(i64::from(thousandths), 1_000),
        actual_fuel_cost: 100_000,
        taxed_at_stated_rate: true,
        issuer_card_used_by_buyer: true,
        exclusive_government_use: true,
        tax_not_collected_from_buyer: true,
        buyer_written_consent: false,
        buyer_tax_refund_on: None,
        claimable_tax_refunded_in_full: false,
        retained_claimable_tax: 0,
        vendor_settlement: FederalCardVendorSettlement::RepaidOrAgreedToRepay,
        buyer_certificate_current: true,
        not_claimed_elsewhere: true,
    }
}

pub(in crate::tests) fn card_issuer_claim(prefix: &str, gallons: u32) -> FederalCardIssuerClaim {
    FederalCardIssuerClaim {
        cc_registration_number: "CC-SYNTHETIC-637".into(),
        registration_active: true,
        sales: vec![
            card_issuer_sale(
                &alloc::format!("{prefix}-13a"),
                FederalCardIssuerFuel::UndyedDiesel,
                gallons,
            ),
            card_issuer_sale(
                &alloc::format!("{prefix}-13b"),
                FederalCardIssuerFuel::UndyedNonaviationKerosene,
                gallons,
            ),
            card_issuer_sale(
                &alloc::format!("{prefix}-13c219"),
                FederalCardIssuerFuel::AviationKeroseneTaxed219,
                gallons,
            ),
            card_issuer_sale(
                &alloc::format!("{prefix}-13c244"),
                FederalCardIssuerFuel::AviationKeroseneTaxed244,
                gallons,
            ),
        ],
        records_complete: true,
        settlement_outside_business_income_and_expenses: true,
        retained_accrual_accounting_confirmed: None,
        retained_cash_accounting_confirmed: None,
    }
}

pub(in crate::tests) fn consented_card_issuer_claim(
    prefix: &str,
    gallons: u32,
) -> FederalCardIssuerClaim {
    let mut claim = card_issuer_claim(prefix, gallons);
    for sale in &mut claim.sales {
        sale.tax_not_collected_from_buyer = false;
        sale.buyer_written_consent = true;
        sale.buyer_tax_refund_on = Some(20261231);
        sale.claimable_tax_refunded_in_full = true;
    }
    claim
}

pub(in crate::tests) fn retained_card_issuer_claim() -> FederalCardIssuerClaim {
    let mut claim = card_issuer_claim("retained", 500_000);
    claim.sales.truncate(2);
    claim.sales[1].fuel = FederalCardIssuerFuel::UndyedDiesel;
    for sale in &mut claim.sales {
        sale.tax_not_collected_from_buyer = false;
        sale.buyer_written_consent = true;
        sale.retained_claimable_tax = 12_150;
    }
    claim.retained_accrual_accounting_confirmed = Some(true);
    claim
}

pub(in crate::tests) fn retained_card_issuer_business() -> FederalBusiness {
    let FederalIncome::Business { data: mut business } = cash_business(40_000) else {
        unreachable!()
    };
    business.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 4_000_000,
        accounting: BusinessAccounting::Accrual,
    };
    business.income_source = BusinessIncomeSource::Capital;
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            card_issuer: Some(retained_card_issuer_claim()),
            ..FederalFuelLines::default()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    business
}

pub(in crate::tests) fn cash_retained_card_issuer_business() -> FederalBusiness {
    let mut business = retained_card_issuer_business();
    business.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 4_000_000,
        accounting: BusinessAccounting::Cash,
    };
    let issuer = business.fuel_tax_credit.as_mut().unwrap().fuels.card_issuer.as_mut().unwrap();
    *issuer = card_issuer_claim("cash-retained", 100_000);
    for (sale, cents) in issuer.sales.iter_mut().zip([2_430, 2_430, 2_180, 2_430]) {
        sale.tax_not_collected_from_buyer = false;
        sale.buyer_written_consent = true;
        sale.retained_claimable_tax = cents;
    }
    issuer.retained_cash_accounting_confirmed = Some(true);
    business
}

pub(in crate::tests) fn bus_vendor_claim(thousandths: u32) -> FederalVendorBusClaim {
    FederalVendorBusClaim {
        ub_registration_number: "UB-SYNTHETIC-637".into(),
        registration_active: true,
        sales: vec![bus_vendor_sale(thousandths)],
        records_complete: true,
    }
}

pub(in crate::tests) fn aviation_vendor_claim(thousandths: u32) -> FederalVendorAviationClaim {
    let sale = FederalVendorAviationSale {
        sale_reference: "synthetic-sale-8a".into(),
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
    };
    FederalVendorAviationClaim {
        ua_registration_number: "UA-SYNTHETIC-637".into(),
        registration_active: true,
        sales: vec![
            sale.clone(),
            FederalVendorAviationSale {
                sale_reference: "synthetic-sale-8b".into(),
                original_tax_rate: FederalVendorAviationTaxRate::TaxedAt244,
                tax_relief: FederalVendorTaxRelief::TaxRepaid,
                ..sale
            },
        ],
        records_complete: true,
    }
}

pub(in crate::tests) fn noncommercial_aviation_vendor_claim(
    thousandths: u32,
) -> FederalVendorNoncommercialAviationClaim {
    FederalVendorNoncommercialAviationClaim {
        ua_registration_number: "UA-SYNTHETIC-637".into(),
        registration_active: true,
        sales: vec![FederalVendorNoncommercialAviationSale {
            sale_reference: "synthetic-sale-8c".into(),
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
        }],
        records_complete: true,
    }
}

pub(in crate::tests) fn nontaxable_aviation_vendor_claim(
    gallons: u32,
) -> FederalVendorNontaxableAviationClaim {
    let sale = FederalVendorNontaxableAviationSale {
        sale_reference: "synthetic-sale-8d".into(),
        date: 20260615,
        buyer_name: "Synthetic Foreign Trade Operator".into(),
        buyer_address: "300 Airport Road".into(),
        government_buyer_tin: None,
        use_type: FederalAviationNontaxableUse::ForeignTrade,
        original_tax_rate: FederalVendorAviationTaxRate::TaxedAt244,
        gallons: crate::kernel::natural::natural(i64::from(gallons), 1_000),
        actual_fuel_cost: 30_000,
        tax_relief: FederalVendorTaxRelief::TaxExcludedPrice,
        taxed_at_stated_rate: true,
        qualifying_nontaxable_use: true,
        buyer_document_current: true,
        no_government_credit_card: None,
        not_claimed_elsewhere: true,
    };
    FederalVendorNontaxableAviationClaim {
        ua_registration: Some(FederalAviationVendorRegistration {
            number: "UA-SYNTHETIC-637".into(),
            active: true,
        }),
        uv_registration: None,
        sales: vec![
            sale.clone(),
            FederalVendorNontaxableAviationSale {
                sale_reference: "synthetic-sale-8e".into(),
                buyer_name: "Synthetic Air Ambulance".into(),
                use_type: FederalAviationNontaxableUse::HelicopterOrFixedWing,
                original_tax_rate: FederalVendorAviationTaxRate::TaxedAt219,
                ..sale
            },
        ],
        records_complete: true,
    }
}

pub(in crate::tests) fn empty_fuel_lines() -> FederalFuelLines {
    FederalFuelLines {
        vendor_state_diesel: None,
        vendor_bus_diesel: None,
        vendor_state_kerosene: None,
        vendor_blocked_pump_kerosene: None,
        vendor_bus_kerosene: None,
        vendor_aviation_kerosene: None,
        vendor_aviation_noncommercial: None,
        vendor_aviation_nontaxable: None,
        card_issuer: None,
        gasoline_off_highway: None,
        gasoline_farm: None,
        gasoline_commercial_fishing: None,
        gasoline_school_bus: None,
        gasoline_intercity_local_bus: None,
        gasoline_export: None,
        aviation_gasoline_commercial: None,
        aviation_gasoline_nonpropulsion_off_highway: None,
        aviation_gasoline_farm: None,
        aviation_gasoline_type_10: None,
        aviation_gasoline_export: None,
        aviation_gasoline_foreign_trade: None,
        diesel_off_highway: None,
        diesel_farm: None,
        diesel_trains: None,
        diesel_intercity_local_bus: None,
        diesel_export: None,
        kerosene_off_highway: None,
        kerosene_farm: None,
        kerosene_intercity_local_bus: None,
        intercity_bus_no_waiver: None,
        bus_uses: None,
        other_uses: None,
        kerosene_export: None,
        kerosene_off_highway_taxed_044: None,
        kerosene_off_highway_taxed_219: None,
        aviation_commercial: None,
        aviation_kerosene_foreign_trade: None,
        aviation_kerosene_farm: None,
        emulsion_off_highway: None,
        emulsion_farm: None,
        emulsion_intercity_local_bus: None,
        emulsion_export: None,
        emulsion_blender: None,
        gasoline_blendstock_export: None,
        dyed_diesel_export: None,
        dyed_kerosene_export: None,
        export_proof_complete: None,
        lpg_off_highway: None,
        p_series_off_highway: None,
        cng_off_highway: None,
        hydrogen_off_highway: None,
        fischer_tropsch_off_highway: None,
        biomass_liquid_off_highway: None,
        lng_off_highway: None,
        biomass_gas_off_highway: None,
    }
}

// Preserve the supplied Schedule C/F source and the two independent gallon ledgers.
// The ordinary partner's QBI statement and source order are shared by these fixtures.
pub(in crate::tests) fn fuel_credit_sources(
    mut business: FederalIncome,
    business_fuels: FederalFuelLines,
    partner_fuels: FederalFuelLines,
    farm_activity: bool,
) -> (FederalDraft, FederalResult) {
    let mut draft = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    let mut without_credit = draft.clone();
    add(&mut without_credit, income(business.clone()));
    add(&mut without_credit, income(FederalIncome::Partnership { data: partner.clone() }));
    let baseline = supported(&without_credit);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: business_fuels.into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: partner_fuels.into(),
        statement_complete: true,
        farm_activity,
    });
    add(&mut draft, income(business));
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    (draft, baseline)
}
