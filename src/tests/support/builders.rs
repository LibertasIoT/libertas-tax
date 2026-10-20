// Domain builders retain the existing test-support facade.
mod able;
mod adoption;
mod alimony;
mod assets;
mod bba;
mod business;
mod business_property;
mod care;
mod casualty;
mod charity;
mod children;
mod community;
mod cooperative;
mod debt;
mod deductions;
mod education;
mod estate;
mod farm;
mod foreign;
mod fuel;
mod gambling;
mod health;
mod home_sale;
mod injured;
mod investment;
mod ira;
mod losses;
mod marketplace;
mod mortgage;
mod office;
mod partnership;
mod payroll;
mod refund;
mod rental;
mod repayment;
mod retirement;
mod returns;
mod roth;
mod scorporation;
mod situations;
mod social_security;
pub(in crate::tests) mod timber;
mod vehicles;

pub(in crate::tests) use able::{
    able_contributions, able_correction, able_distribution, mixed_able,
};
pub(in crate::tests) use adoption::{
    adoption_child, employer_adoption, foreign_adoption_benefits, joint_adoption_history_mut,
    joint_adoption_payments_mut, joint_employer_adoption_return,
    joint_special_needs_below_code_t_cap, ordinary_and_special_joint_adoption_return,
    separated_adoption_return, set_joint_adoption_exclusion,
    set_joint_adoption_exclusion_for_child, set_joint_adoption_w2, shared_adoption_return,
    two_child_joint_adoption_return, two_special_joint_adoption_return,
};
pub(in crate::tests) use alimony::{alimony_record, paid_alimony, recapture_alimony};
pub(in crate::tests) use assets::{
    bonus_equipment, business_with_bonus, carried_asset, continuing_asset, continuing_equipment,
    de_minimis_business, depreciable_asset, direct_amt_carry_history, documented_auto,
    elected_asset, historical_original_auto, intangible_business, macrs_equipment,
    ordinary_original_auto, original_auto_editor_sources, separate179, startup_business,
    table_history,
};
pub(in crate::tests) use bba::{
    bba_interest_increase, bba_interest_reduction, bba_low_tax_return,
    bba_thousand_dollar_reduction,
};
pub(in crate::tests) use business::{
    capital_business, care_business, cash_business, current_risk_business, health_business,
    mixed_business, self_employed_insurance, sep_business,
};
pub(in crate::tests) use business_property::{
    collectible_investment_gain, combined_property_sstb_source, complete_property_gain_draft,
    complete_trading_property_interest, former_passive_property_scorp,
    historical_property_excess_draft, k1_capital, k1_collectible_capital, k1_portfolio_1250,
    k1_section1231, lookback_worksheet, multiple_fully_recaptured_gains, niit_loss_history,
    nol_capital, operating_loss_property_gain_draft, passive_property_pool, passive_property_scorp,
    property_excess_draft, property_loss_operating_profit_draft, property_niit_worksheet,
    property_qbi, property_qbi_character_draft, property_qbi_character_items, qualified_rate_gain,
    rate_ordered_recapture, section1231_history, trading_with_unreleased_histories,
};
pub(in crate::tests) use care::{
    benefit_return, care_aggregate, care_box10, care_costs, care_months, care_w2,
    clear_care_benefits, document_care, grace_and_current_benefits, grace_only_benefits,
    joint_employer_care_return, joint_employer_with_one_grace_return, prior_care_facts,
    set_care_months,
};
pub(in crate::tests) use casualty::{
    casualty_living_payment, home_casualty_return, major_disaster, personal_disaster,
};
pub(in crate::tests) use charity::{
    cash_carry_with_student_and_whaling, cash_carry_with_whaling, donated_personal_vehicle,
    gift_appraisal, paired_2025_noncash_carries, prior_household_carry, prior_public_stock_carry,
    public_stock_gift, set_gift_appraisal, used_goods, volunteer_travel, whaling_claim,
    whaling_expense, with_current_direct_cash,
};
pub(in crate::tests) use children::{
    child_dividends, child_election, child_election_mut, child_interest, child_interest_gains,
    death, died_more, ordinary_parent, set_fact, set_release, young_interest_return,
    young_special_return,
};
pub(in crate::tests) use community::{
    community_arizona, community_dividend_draft, community_interest_draft,
    community_joint_portfolio_draft, community_multi_w2_draft, community_portfolio_with_extra_w2s,
    community_portfolio_with_second_payers, community_w2_draft,
};
pub(in crate::tests) use cooperative::{
    continuing_purchase_refund_session, cooperative_asset_refunds, cooperative_asset_refunds_on,
    cooperative_community_mfs, cooperative_equipment_identity_return, cooperative_partial_building,
    cooperative_personal, cooperative_routed_income,
};
pub(in crate::tests) use debt::{
    bankruptcy_debt, canceled_personal_debt, farm_debt, health_loan_assistance, insolvency_debt,
    price_reduction_debt, property,
};
pub(in crate::tests) use deductions::itemized;
pub(in crate::tests) use education::{
    distribution_period, education_choice, education_expenses, native_education, prize_income,
    scholarship,
};
pub(in crate::tests) use estate::{estate_trust_code_c_draft, estate_trust_code_d_data};
pub(crate) use farm::farm_land;
pub(in crate::tests) use farm::{
    SeMethod, accrual_farm, add_supply_farm, averaging_foreign, averaging_return,
    averaging_statement_credit, cash_farm, ccc_fifty_thousand, ccc_mut, ccc_return, ccc_source,
    ccc_source_input, conservation_facts, conservation_final_return, conservation_method,
    conservation_return, crop_farm, crop_practice, crop_records, crop_return, crp_return,
    district_assessment, elect_se, farm_business, farm_header, farm_land_sale, farm_loss_return,
    farm_only_return, farm_rental_data, farm_specials, farm_supplies, farm_supply_fifty_thousand,
    farm_supply_mut, farm_supply_purchase_source, farm_supply_return, farmland_basis_mut,
    farmland_installment_low_tax_credit, farmland_installment_return,
    farmland_marketplace_repayment, farmland_resolved_sale, farmland_sales_mut,
    farmland_statement_credit, fixture_conservation_parcel, foreign_special_distribution,
    native_cash_fixture, payee_statement_credit, poultry_resale, production_poultry_carry,
    qualified_dividend, raise_payee_statement_tax, raised_farm, se_health, se_history,
    set_farmland_sales, set_land_sales, waive_farm_carryback,
};
pub(in crate::tests) use foreign::{
    ForeignCreditBalance, claim_mut, combined_foreign_wage_business, daily_basis, employee_housing,
    excluded_foreign_business, foreign_asset, foreign_asset_return, foreign_balance, foreign_claim,
    foreign_high_tax_return, foreign_history_of, foreign_loss_records, foreign_loss_system,
    foreign_medical_itemizing, foreign_mortgage, foreign_passive_loss_return, foreign_pay,
    foreign_service_business, foreign_state_payment, foreign_state_return, housing_claim,
    k3_every_field, prior_foreign_credit, self_employed_housing, set_claim,
    set_foreign_amt_election, set_foreign_carryback, set_foreign_history, set_foreign_state,
    wholly_foreign_credit_return,
};
pub(in crate::tests) use fuel::{
    alternative_fuel_use, aviation_gasoline_type_10_uses, aviation_vendor_claim,
    blocked_pump_vendor_claim, bus_vendor_claim, card_issuer_claim, card_issuer_sale,
    cash_fuel_recovery, cash_retained_card_issuer_business, consented_card_issuer_claim,
    farm_aviation_gasoline, farm_aviation_kerosene_claim, fuel_credit_sources, fuel_use,
    government_vendor_claim, noncommercial_aviation_vendor_claim, nontaxable_aviation_vendor_claim,
    partial_cash_fuel_recovery, partial_partner_fuel_recovery, retained_card_issuer_business,
    retained_card_issuer_claim,
};
pub(in crate::tests) use gambling::{
    child_gambling_return, gambling, prize, professional_gambling,
};
pub(in crate::tests) use health::{
    add_hsa_deemed, add_hsa_distribution, archer_contributor, archer_family, archer_msa,
    archer_owner_mut, archer_rollover, archer_self, current_hsa_funding, december_hsa,
    health_coverage_child, health_w2_year, historical_hsa_family, hsa_correction,
    hsa_distribution_mut, hsa_entry, hsa_funding, hsa_prior_basis_funding, hsa_rollover,
    inherited_hsa_funding, inherited_interest, joint_december_hsa, last_month_history, ltc_form,
    ltc_native_payee_cohort, ltc_one_period, ltc_period, ltc_record, medical_inheritance,
    medicare_msa, personal_care, separate_december_hsa, set_archer,
};
pub(in crate::tests) use home_sale::{
    duty_home_use, home_care, home_sale, home_use, official_duty_sale, reduced_home_periods,
};
pub(crate) use injured::{
    INJURED_ANNUAL_SOURCE, INJURED_CONTRACT_DAYS, INJURED_DAILY_SOURCE, INJURED_INSURED_COUNT,
    injured_daily_forms, injured_insured_documents, injured_period_payees,
};
pub(in crate::tests) use investment::{
    bad_debt, capital_history, capital_return, collectible_sale, contractor_bad_debt,
    direct_tax_credit_bond, disqualifying_iso_sale, elected_portfolio,
    former_home_mortgage_bad_debt, hobby_sales, investment_borrowing, iso_exercise,
    mixed_gain_distribution, niit_tax_source_return, ordinary_espp_lot, ordinary_investment_sale,
    personal_sale, purchased_bad_debt, qsb_income, qsb_source_sale, qualifying_iso_sale,
    reit_income, retained_estate_nol_periods, sale_with_amt, savings_income, section1244_loss,
    single_bond_with_same_total_credit, undistributed_gain,
};
pub(in crate::tests) use ira::{
    add_inherited_ira, beneficiary_rmd, inherited_basis_mut, inherited_estate, inherited_ira,
    inherited_payment, inherited_rmd_data, ira_facts, next_year_ira_deposits, own_rmd,
    qtp_roth_rollover, rmd_corrected, rmd_record, saver, set_inherited_estate, set_qtp_roth,
    two_qtp_roth_rollovers,
};
pub(in crate::tests) use losses::{
    amt_only_capital_history, cash_carry_with_appreciated_stock, cash_carry_with_current_property,
    cash_carry_with_hosted_student, cash_carry_with_vehicle,
    current_and_prior_cash_with_appreciated_stock, direct_cash_prior_cash_and_property, k1_amount,
    minimum_tax_exclusion_base, minimum_tax_gains, minimum_tax_history, next_loss_limit,
    nol_adoption, nol_cash_gift, nol_rental, nol_social, nol_work_deductions,
    ordinary_property_for_cash_carry, prior_atnol, prior_cash_charity, prior_nol,
    ptp_ebl_nol_draft, retained_loss_projection, sold_vehicle_for_cash_carry,
};
pub(in crate::tests) use marketplace::{
    MarketplaceAllocation, MarketplaceEligibility, MarketplaceMonth, MarketplacePolicy,
    marketplace_claim, marketplace_facts_mut, marketplace_month,
};
pub(in crate::tests) use mortgage::{
    fixture_mortgage_interest, identified_mortgages, mortgage_certificate_credit,
    mortgage_disposition, mortgage_ledger_mut, mortgage_recapture_facts, mortgage_recapture_qsml,
    mortgage_recapture_sale_id, ordinary_subsidy_facts, set_mortgage_subsidy,
};
pub(in crate::tests) use partnership::{
    d6_ptp_pages, d6_selected_source, ordinary_partner_source, partner_activity_record,
    partner_basis_record, partner_distribution_health_source, partner_excess_loss_source,
    partner_health_source, partner_paid_health_source, partner_qbi_statement, partner_risk_record,
    partner_service179_session, partner_uniform_profit_draft, ptp179_draft,
    section179_income_source, selected_ptp179,
};
pub(in crate::tests) use payroll::{
    allocated_employee_tips, beauty_tip_credit_draft, covered_wage, household_disability,
    household_employer, household_pay, omitted_employee_wages, raw_sick_payment, reported_sick_pay,
    restaurant_business, restaurant_business_mut, restaurant_tip_credit_draft, sick_plan_employer,
    tip_fica_carryforward, uncollected_wage, w2_uncollected, wage_documents,
    wage_with_employee_tips, with_household, work_deductions,
};
pub(in crate::tests) use refund::{
    current_year_refund_draft, historical_pease_payment, historical_refund_payment,
    multiple_prior_and_current_refund_draft, multiple_prior_and_current_refund_history,
    prior_and_current_refund_history, refund_history, three_year_refund_history,
    two_year_refund_history,
};
pub(in crate::tests) use rental::{
    community_rental_property_history, compare_rental_risk_record, form4835_risk_prior_return,
    form4835_risk_property_return, kept_rental_casualty, large_separate_amt_business,
    mixed_rental_with_actual_sale_records, personal_rent, personal_rent_with_assets,
    personal_rent_with_history, qualified_condemned_short_rental, qualified_rental_sale,
    qualified_rental_with_amt_recapture, qualified_rental_with_passive_asset_loss,
    qualified_rental_with_personal_asset_gain, qualified_rental_with_recapture,
    qualified_rental_with_section179_conversion, recurring_rental_casualty, rehabilitation, rental,
    rental_casualty_basis, rental_casualty_source, rental_equipment, rental_property_loss_year,
    rental_risk_without_method, rental_sale_history, rental_with_prior_property_history, royalty,
    separate_amt_equipment, simplified_rental_risk, sold_rental_with_property_loss_history,
    transition_binding, transition_construction, transition_source_page, transition_source_record,
};
pub(in crate::tests) use repayment::{
    add_repayment_inferred_capital, box_1, box_2, employer_paid, government_unemployment,
    historical_capital, historical_dividends, historical_sale, loan_claim,
    prior_unemployment_repayment, repayment_carry_chain, repayment_comparison_draft,
    repayment_deferral_capital, repayment_early_draft, repayment_eic_draft,
    repayment_exclusion_loss_chain, repayment_minimum_chain, repayment_minimum_source,
    repayment_restoration_draft, repayment_senior_draft, repayment_social_draft,
    repayment_years_mut,
};
pub(crate) use retirement::public_safety_pensions;
pub(in crate::tests) use retirement::{
    abuse_withdrawal, abuse_withdrawal_plan, automatic_enrollment_income, basis_distribution,
    birth_withdrawal, box7, box7_of, care_claim, care_return, child_annuity_draft,
    contract_discharge, conversion, deferred_withdrawal, deferred_withdrawal_group,
    disability_pension, education_withdrawal, emergency_withdrawal, fixed_annuity,
    gross_birth_claim, gross_home_claim, immediate_annuity, incoming_plan_rollover,
    ira_basis_exception_return, ira_contribution_distribution, joint_annuity,
    nonqualified_exception, old_general_annuity, old_general_entry, pension_basis,
    pension_contribution, pension_entry, plan_gross_exception, printed_as, prior_ira_basis,
    public_safety, qcd, qcd_with_basis, qualified_fixed_pension, qualified_withdrawal,
    railroad_pension, railroad_repayment_draft, reservist_basis_return, reservist_capacity_return,
    reservist_repayment, retirement_entry, retirement_exception, retirement_exception_entry,
    rollover, special_ira_repayment, special_ira_repayment_example,
    special_repayment_capacity_return, variable_annuity,
};
pub(crate) use returns::make_professional;
pub(in crate::tests) use returns::{
    add, add_child, all_ticked, child, complete, empty_return, filing_choice, income,
    link_home_fixture, lot_id, person, set_age, sid, w2, wage,
};
pub(in crate::tests) use roth::{
    add_inherited_roth, inherited_roth, inherited_roth_payment, roth_earlier_funding, roth_history,
    roth_hsa_funding, roth_withdrawal,
};
pub(in crate::tests) use scorporation::{
    amt_k1_note, amt_records, basis_blocked_section179_scorp, complete_ordinary_scorp,
    complete_scorp_recapture, historical_k1_excess, investment_sale_gain, investment_tax_draft,
    investment_tax_note, k1_eic_investment_note, k1_excess_note, k1_interest_note, k1_nol_note,
    loss_year, mixed_note_repayments, nonpassive_scorp, participation_facts, passive_note,
    passive_qbi_source, passive_scorp_document, qbi_record_issues, reconcile_qbi_tracking,
    repaid_written_note, scorp_current_loss_statement, scorp_current_qbi_source, scorp_document,
    scorp_large_loss, scorp_loss_qbi_source, scorp_note, scorp_portfolio,
    scorp_repaid_reduced_open_account, scorp_risk_history, sole_qbi_loss, trading_interest_draft,
};
pub(crate) use situations::{Chosen, every_refusal_item};
pub(in crate::tests) use social_security::{social_prior_year, social_security_net};
pub(in crate::tests) use vehicles::{
    clean_vehicle_business_return, clean_vehicle_claim, new_clean_vehicle, owned_actual_vehicle,
    personal_refueling, used_clean_vehicle, vehicle_loan,
};

// Retain existing topic-call paths for helpers now used within their domain.
#[allow(unused_imports)]
pub(in crate::tests) use self::{
    care::CareMonth,
    debt::personal_property,
    farm::farm_supply_advance,
    foreign::residence,
    fuel::{bus_vendor_sale, government_vendor_sale},
    ira::qtp_roth_form,
    retirement::disability_pension_with,
    vehicles::clean_vehicle_details,
};

pub(crate) mod k1res;
pub(in crate::tests) use k1res::empty_k1_activity_choices;

pub(in crate::tests) use injured::{
    RestRoundingSource, SpecializedWageFixture, actual_liability_totals, allocation,
    append_injured_two_period_ltc_sources, changed_filer_carry, injured_return,
    liability_portfolio_facts, liability_portfolio_native_facts, liability_specialized_wage_facts,
    rest_rounding_source, texas_interest_admission,
};

pub(in crate::tests) use payroll::{
    WageClassificationCase, non_code_l_wage_fixture, source_medicaid, source_statutory_wage,
    statutory_multiple_return, wage_classification_fixture,
};

pub(in crate::tests) use losses::source_owned_nol;

pub(in crate::tests) use business::{add_ebl_business, source_expenses_business};

pub(in crate::tests) use adoption::source_adoption_code_t;

pub(in crate::tests) use care::SettledCareCase;

pub(in crate::tests) use investment::{
    K1PortfolioNiitSource, NiitDeductionExpense, NiitExpenseSource, add_current_niit_property,
    current_nua_return, deduction_owner_fact, hobby_niit_return, k1_niit_portfolio_entry,
    mixed_niit_return, niit_binding_fixture, niit_deduction_fixture, niit_zero_fixture,
    noninvestment_election_fixture, retained_niit_return, retained_owner_source,
    unit13_election_fixture,
};

pub(in crate::tests) use children::child_election_owner_fixture;

pub(in crate::tests) use foreign::{add_foreign_owner_income, foreign_income_owner_fixture};

pub(in crate::tests) use rental::personal_rental_owner_fixture;

pub(in crate::tests) use debt::personal_debt_owner_fixture;

pub(in crate::tests) use payroll::{
    additional_medicare, home_employee, medicare_withholding, payroll_taxes,
};

pub(in crate::tests) mod injured_income;
pub(in crate::tests) use care::{care_own_plans, care_providers};

pub(in crate::tests) mod injured_credit;

pub(in crate::tests) mod native_income;

pub(in crate::tests) use fuel::empty_fuel_lines;

pub(in crate::tests) use estate::empty_estate_portfolio;

pub(in crate::tests) mod survey;

pub(in crate::tests) use children::noncitizen_child;

pub(in crate::tests) use situations::survivor_with_adult_son;

pub(in crate::tests) use marketplace::marketplace_nonmedical_child;

pub(in crate::tests) use marketplace::marketplace_wage_draft;

mod k1_consolidated;
pub(in crate::tests) use k1_consolidated::{
    k1_distribution_counterparts, k1_qbi_statement, k1_reported_sale,
};

pub(in crate::tests) use investment::native_sale;

pub(in crate::tests) use retirement::native_retirement;

pub(in crate::tests) use losses::{five_classes, four_classes};

pub(in crate::tests) mod unit50_retirement;

pub(in crate::tests) use business::ordinary_business_expense;
pub(in crate::tests) use investment::native_investment_loan;
pub(in crate::tests) use rental::purchased_rental_building;
pub(in crate::tests) use vehicles::native_business_mileage;

pub(in crate::tests) use refund::historical_refund_history;

pub(in crate::tests) use investment::{
    native_exempt_interest, native_interest, native_treasury_interest,
};

pub(in crate::tests) use investment::native_dividend;

pub(in crate::tests) use health::{
    archer_contributor_row, archer_excess_accounting, health_account_page, hsa_excess_accounting,
};
pub(in crate::tests) use marketplace::{
    annual_marketplace_businesses, assign_marketplace_businesses,
};
pub(in crate::tests) use native_income::{native_div1099, native_int1099, native_oid1099};

pub(in crate::tests) mod injured_original;
pub(in crate::tests) use office::{
    HomeFixture, actual_office, add_actual_office, add_office_claim, add_simplified_office,
    interview_office, itemizing_office, office_claim_mut, office_interview_return, office_itemize,
    office_line_id, office_return, owned_office_home, simplified_claim, standard_office,
};

pub(in crate::tests) mod cohorts;
pub(in crate::tests) use rental::{
    rental_energy_building, rental_energy_claim, rental_energy_return,
};
