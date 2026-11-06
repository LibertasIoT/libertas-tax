macro_rules! paper_adapter {
    ($function:ident, $native_function:ident, $fixture:ident, $native:ty, $paper:ty) => {
        pub(in crate::tests) trait $fixture {
            fn paper(self) -> $paper;
        }
        impl $fixture for $native {
            fn paper(self) -> $paper {
                $native_function(self)
            }
        }
        impl $fixture for $paper {
            fn paper(self) -> $paper {
                self
            }
        }
        pub(in crate::tests) fn $function(form: impl $fixture) -> $paper {
            form.paper()
        }
    };
}

mod form_1041_k1;
pub(in crate::tests) use form_1041_k1::*;
mod form_1097_btc;
pub(in crate::tests) use form_1097_btc::*;
mod form_1099_a;
pub(in crate::tests) use form_1099_a::*;
mod form_1099_b;
pub(in crate::tests) use form_1099_b::*;
mod form_1099_c;
pub(in crate::tests) use form_1099_c::*;
mod form_1099_g;
pub(in crate::tests) use form_1099_g::*;
mod form_1099_k;
pub(in crate::tests) use form_1099_k::*;
mod form_1099_ltc;
pub(in crate::tests) use form_1099_ltc::*;
mod form_1099_misc;
pub(in crate::tests) use form_1099_misc::*;
mod form_1099_nec;
pub(in crate::tests) use form_1099_nec::*;
mod form_1099_patr;
pub(in crate::tests) use form_1099_patr::*;
mod form_1099_q;
pub(in crate::tests) use form_1099_q::*;
mod form_1099_qa;
pub(in crate::tests) use form_1099_qa::*;
mod form_1099_r;
pub(in crate::tests) use form_1099_r::*;
mod form_1099_s;
pub(in crate::tests) use form_1099_s::*;
mod form_1099_sa;
pub(in crate::tests) use form_1099_sa::*;
mod form_1120_s_k1;
pub(in crate::tests) use form_1120_s_k1::*;
mod form_2439;
pub(in crate::tests) use form_2439::*;
mod form_3921;
pub(in crate::tests) use form_3921::*;
mod form_3922;
pub(in crate::tests) use form_3922::*;
mod form_5498_sa;
pub(in crate::tests) use form_5498_sa::*;
mod form_8815;
pub(in crate::tests) use form_8815::*;
mod form_rrb_1099_r;
pub(in crate::tests) use form_rrb_1099_r::*;
mod form_ssa_1099;
pub(in crate::tests) use form_ssa_1099::*;
mod form_w2_g;
pub(in crate::tests) use form_w2_g::*;
pub(in crate::tests) use native_income::{
    InterestPaperFixture, OidPaperFixture, WagePaperFixture, capital_return_document,
    div1099_source, dividend_document, dividend_source, interest_document, interest_source,
    mortgage_document, mortgage_loan, mortgage_loan_source, mortgage_snapshot, mortgage_source,
    oid_document, oid_source, wage_document, wage_source,
};
mod form_1098_e;
pub(in crate::tests) use form_1098_e::*;
mod form_1098_t;
pub(in crate::tests) use form_1098_t::*;
mod form_1098_vli;
pub(in crate::tests) use form_1098_vli::*;
mod fixture_edits;
pub(in crate::tests) use fixture_edits::*;
mod paper_documents;
pub(in crate::tests) use paper_documents::*;
pub(in crate::tests) mod ads;
pub(in crate::tests) mod related_forms;
// Domain builders retain the existing test-support facade.
mod able;
mod adoption;
mod alimony;
mod assets;
mod intangible;
pub(in crate::tests) use intangible::ordinary_intangible;
mod bba;
mod broker;
mod business;
pub(in crate::tests) use broker::{
    broker_b_paper, broker_community_return, broker_da_paper, broker_manual_control,
};
mod business_property;
mod care;
mod casualty;
mod charity;
mod children;
mod community;
mod cooperative;
mod debt;
mod deductions;
mod document_names;
mod education;
mod estate;
mod farm;
mod foreign;
mod fuel;
mod gambling;
mod health;
mod home_sale;
mod injured;
pub(in crate::tests) mod interview_profiles;
mod investment;
mod undeclared_forms;
pub(in crate::tests) use undeclared_forms::{espp_acquisition_paper, rrb_benefit_paper};
mod ira;
mod k3_paper;
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
    actual_vehicle, auto_with_retained_179_release, bonus_auto_179, bonus_equipment,
    business_with_bonus, carried_asset, claimed_regular_asset_with_amt_carry, community_auto_179,
    continuing_asset, continuing_equipment, current_property_business_return, de_minimis_business,
    depreciable_asset, direct_amt_carry_history, documented_auto, elected_asset,
    historical_original_auto, intangible_business, macrs_equipment, mixed_auto_179,
    ordinary_original_auto, original_auto_editor_sources, partial_bonus_auto,
    same_year_property_sale, separate179, startup_business, suv_asset, table_history,
    uncapped_auto_179,
};
pub(in crate::tests) use bba::{
    added_bba, bba_interest_increase, bba_interest_reduction, bba_low_tax_return,
    bba_thousand_dollar_reduction,
};
pub(in crate::tests) use business::{
    capital_business, care_business, cash_business, current_risk_business,
    direct_participation_with_omitted_k1, health_business, ministerial_housing_business_return,
    mixed_business, passive_business, self_employed_insurance, sep_business,
};
pub(in crate::tests) use business_property::{
    collectible_investment_gain, combined_property_sstb_source, complete_property_gain_draft,
    complete_trading_property_interest, former_passive_property_scorp,
    historical_property_excess_draft, k1_capital, k1_collectible_capital, k1_portfolio_1250,
    k1_section1231, legacy_k1_capital_sources, lookback_worksheet,
    mixed_native_legacy_capital_draft, multiple_fully_recaptured_gains, niit_loss_history,
    nol_capital, operating_loss_property_gain_draft, passive_property_pool, passive_property_scorp,
    property_excess_draft, property_loss_operating_profit_draft, property_niit_worksheet,
    property_qbi, property_qbi_character_draft, property_qbi_character_items, qualified_rate_gain,
    rate_ordered_recapture, section1231_history, trading_with_unreleased_histories,
};
pub(in crate::tests) use care::{
    benefit_return, care_aggregate, care_box10, care_costs, care_months, care_w2,
    clear_care_benefits, document_care, grace_and_current_benefits, grace_only_benefits,
    joint_employer_care_return, joint_employer_with_one_grace_return, prior_care_facts,
    section179_care_employee_reimbursements, section179_care_return, set_care_months,
};
pub(in crate::tests) use casualty::{
    casualty_living_payment, home_casualty_return, major_disaster, personal_disaster,
};
pub(in crate::tests) use charity::{
    cash_carry_with_student_and_whaling, cash_carry_with_whaling, donated_personal_vehicle,
    gift_appraisal, paired_2025_noncash_carries, prior_household_carry, prior_public_stock_carry,
    public_stock_gift, push_gift, set_gift_appraisal, set_gifts, set_vehicle_appraisal, used_goods,
    vehicle_gift_page, vehicle_gift_source, volunteer_travel, whaling_claim, whaling_expense,
    with_current_direct_cash,
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
    claim_last_cooperative, claim_related_component, continuing_purchase_refund_session,
    cooperative_asset_refunds, cooperative_asset_refunds_on, cooperative_box12_session,
    cooperative_community_mfs, cooperative_equipment_identity_return, cooperative_partial_building,
    cooperative_personal, cooperative_routed_income,
};
pub(in crate::tests) use debt::{
    bankruptcy_debt, canceled_personal_debt, farm_debt, health_loan_assistance, insolvency_debt,
    price_reduction_debt, property,
};
pub(in crate::tests) use deductions::{
    elect_itemized, empty_facts, itemized, spouse_itemizes_request,
};
pub(in crate::tests) use document_names::{
    accepted_document_names, add_documents, added_documents_request, after_death_owner_document,
    assign_documents, dependent_recipient_with_unavailable_financial_owner,
    document_owner_household, document_owners_mut, document_recipient_household,
    education_recipient_name_collision, estimate_issue_household, expected_named_source,
    expected_named_sources, fixture_blank_owner_inputs, fixture_document_names, fixture_owned_rows,
    fixture_owner_inputs, information_return_answers_and_owners, joint_financial_session,
    name_accepted_income_documents, named_document, prepare_income_fixture, save_mfs_owner_fixture,
    set_fixture_recipient, set_joint_financial_share, unknown_documents,
};
pub(in crate::tests) use education::{
    add_education, distribution_period, education_choice, education_expenses, native_education,
    prize_income, scholarship, set_education,
};
pub(in crate::tests) use estate::{estate_trust_code_c_draft, estate_trust_code_d_data};
pub(crate) use farm::farm_land;
pub(in crate::tests) use farm::{
    SeMethod, accrual_farm, add_supply_farm, answer_crop_practice, answer_crop_timing,
    averaging_foreign, averaging_return, averaging_statement_credit, cash_farm, ccc_fifty_thousand,
    ccc_mut, ccc_return, ccc_source, ccc_source_input, conservation_facts,
    conservation_final_return, conservation_livestock_return, conservation_method,
    conservation_return, crop_farm, crop_practice, crop_records, crop_return, crp_return,
    district_assessment, elect_se, farm_business, farm_government_component, farm_header,
    farm_land_sale, farm_loss_return, farm_only_return, farm_rental_data,
    farm_rental_passive_asset_return, farm_rental_passive_property_editor_return,
    farm_rental_passive_property_return, farm_specials, farm_supplies, farm_supply_fifty_thousand,
    farm_supply_mut, farm_supply_purchase_source, farm_supply_return, farmer_gross_asset,
    farmer_gross_earlier_exchange, farmer_gross_earlier_property, farmer_gross_penalty_return,
    farmer_gross_qof_source, farmer_gross_rental_royalty_return, farmer_gross_return,
    farmland_basis_mut, farmland_installment_low_tax_credit, farmland_installment_return,
    farmland_marketplace_repayment, farmland_omitted_penalty_return, farmland_resolved_sale,
    farmland_sales_mut, farmland_statement_credit, farmland_underpayment_inputs,
    fixture_conservation_parcel, foreign_special_distribution, install_crop_records,
    native_cash_fixture, partial_land_allocation, partial_land_parcel, payee_statement_credit,
    poultry_resale, production_poultry_carry, qualified_dividend, raise_payee_statement_tax,
    raised_farm, relate_farm_component, se_health, se_history, set_farmland_sales, set_land_sales,
    waive_farm_carryback,
};
pub(in crate::tests) use foreign::{
    ForeignCreditBalance, claim_mut, combined_foreign_wage_business, daily_basis, employee_housing,
    excluded_foreign_business, foreign_asset, foreign_asset_return, foreign_balance, foreign_claim,
    foreign_employee179_return, foreign_high_tax_return, foreign_history_of, foreign_loss_records,
    foreign_loss_system, foreign_medical_itemizing, foreign_mortgage, foreign_passive_loss_return,
    foreign_pay, foreign_service_business, foreign_state_payment, foreign_state_return,
    housing_claim, k3_every_field, prior_foreign_credit, self_employed_housing, set_claim,
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
    child_gambling_return, gambling, prize, professional_gambling, wagering179_session,
};
pub(in crate::tests) use health::{
    add_hsa_deemed, add_hsa_distribution, archer_contributor, archer_family, archer_msa,
    archer_owner_mut, archer_rollover, archer_self, current_hsa_funding, december_hsa,
    health_coverage_child, health_w2_year, historical_hsa_family, hsa_correction,
    hsa_distribution_mut, hsa_entry, hsa_funding, hsa_prior_basis_funding, hsa_rollover,
    inherited_hsa_funding, inherited_interest, joint_december_hsa, last_month_history, ltc_form,
    ltc_identity_fixture, ltc_native_payee_cohort, ltc_one_period, ltc_period, ltc_record,
    medical_inheritance, medicare_msa, personal_care, separate_december_hsa, set_archer,
};
pub(in crate::tests) use home_sale::{
    duty_home_use, home_care, home_sale, home_sale_source, home_use, official_duty_sale,
    official_duty_sale_source, reduced_home_periods, sold_on,
};
pub(crate) use injured::{
    INJURED_ALLOCATION_COUNT, INJURED_ANNUAL_SOURCE, INJURED_CONTRACT_DAYS, INJURED_DAILY_SOURCE,
    INJURED_INSURED_COUNT, INJURED_TAXABLE_PER_PERIOD, injured_daily_forms,
    injured_insured_documents, injured_period_payees,
};
pub(in crate::tests) use investment::{
    bad_debt, capital_history, capital_return, collectible_sale, contractor_bad_debt,
    current_installment_joint_niit_draft, current_installment_land, current_installment_niit_draft,
    direct_tax_credit_bond, disqualifying_iso_sale, elected_portfolio, espp_source_answers,
    espp_source_paper, former_home_mortgage_bad_debt, hobby_sales, investment_borrowing,
    mixed_gain_distribution, native_iso_exercise, niit_tax_source_return, ordinary_espp_lot,
    ordinary_investment_sale, personal_sale, purchased_bad_debt, qsb_income, qsb_source_sale,
    qualifying_iso_sale, reit_income, retained_estate_nol_periods, sale_with_amt, savings_income,
    section1244_loss, single_bond_with_same_total_credit, undistributed_gain,
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
    complete_partnership_paper, current_ptp_care_return, current_ptp_ordinary_histories,
    current_ptp_page, current_ptp_passive_history, d6_ptp_pages, d6_selected_source,
    known_fact_partnership_draft, ordinary_partner_source, partner_activity_record,
    partner_basis_record, partner_distribution_health_source, partner_excess_loss_source,
    partner_health_source, partner_paid_health_source, partner_qbi_statement, partner_risk_record,
    partner_service179_session, partner_uniform_profit_draft, partnership_orphan_income_statement,
    partnership_property_total_papers, partnership_uncalculated_code_papers, ptp_rental_draft,
    ptp_rental_source, ptp179_draft, push_partnership_credit, push_partnership_property,
    section179_income_source, selected_ptp179, set_partnership_credits,
};
pub(in crate::tests) use payroll::{
    allocated_employee_tips, beauty_tip_credit_draft, covered_wage, employee_railroad_source,
    household_disability, household_employer, household_pay, income_record_boundary_documents,
    informational_wage_document, omitted_employee_wages, raw_sick_payment, reported_sick_pay,
    restaurant_business, restaurant_business_mut, restaurant_tip_credit_draft, sick_plan_employer,
    statutory_storage_return, tip_fica_carryforward, uncollected_wage, w2_uncollected,
    wage_documents, wage_with_employee_tips, with_household, work_deductions,
};
pub(in crate::tests) use refund::{
    current_year_refund_draft, historical_pease_payment, historical_refund_payment,
    multiple_prior_and_current_refund_draft, multiple_prior_and_current_refund_history,
    prior_and_current_refund_history, refund_history, three_year_refund_history,
    two_year_refund_history,
};
pub(in crate::tests) use rental::{
    accepted_rental_casualty_items, bound_home_casualty, community_rental_property_history,
    compare_rental_risk_record, duplicate_bound_home_casualty_cost, form4835_risk_prior_return,
    form4835_risk_property_return, kept_rental_casualty, large_separate_amt_business,
    mixed_rental_with_actual_sale_records, personal_rent, personal_rent_with_assets,
    personal_rent_with_history, qualified_condemned_short_rental, qualified_rental_sale,
    qualified_rental_with_amt_recapture, qualified_rental_with_passive_asset_loss,
    qualified_rental_with_personal_asset_gain, qualified_rental_with_recapture,
    qualified_rental_with_section179_conversion, recurring_rental_casualty, rehabilitation, rental,
    rental_casualty_basis, rental_casualty_source, rental_equipment, rental_property_loss_year,
    rental_risk_without_method, rental_sale_history, rental_with_prior_property_history, royalty,
    separate_amt_equipment, shared_casualty_return, simplified_rental_risk,
    sold_rental_with_property_loss_history, split_bound_home_casualty_cost, standard_home_casualty,
    transition_binding, transition_construction, transition_source_page, transition_source_record,
};
pub(in crate::tests) use repayment::{
    add_repayment_inferred_capital, government_unemployment, historical_capital,
    historical_dividends, historical_sale, prior_unemployment_repayment, repayment_carry_chain,
    repayment_comparison_draft, repayment_deferral_capital, repayment_early_draft,
    repayment_eic_draft, repayment_exclusion_loss_chain, repayment_minimum_chain,
    repayment_minimum_source, repayment_restoration_draft, repayment_senior_draft,
    repayment_social_draft, repayment_years_mut,
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
    retirement_followup_path, retirement_question_catalog, rollover, special_ira_repayment,
    special_ira_repayment_example, special_repayment_capacity_return, variable_annuity,
};
pub(crate) use returns::make_professional;
pub(in crate::tests) use returns::{
    FilerSituations, add, add_child, all_ticked, answer_separate_pages, child, complete,
    empty_return, filing_choice, income, link_home_fixture, lot_id, person, set_age, sid, w2, wage,
};
pub(in crate::tests) use roth::{
    add_inherited_roth, inherited_roth, inherited_roth_payment, roth_earlier_funding, roth_history,
    roth_hsa_funding, roth_withdrawal,
};
pub(in crate::tests) use scorporation::{
    amt_k1_note, amt_records, basis_blocked_section179_scorp, complete_ordinary_scorp,
    complete_scorp_recapture, historical_k1_excess, investment_sale_gain, investment_tax_draft,
    investment_tax_note, k1_boundary_continuing, k1_eic_investment_note, k1_excess_note,
    k1_interest_note, k1_nol_note, loss_year, mixed_note_repayments, nonpassive_scorp,
    participation_facts, passive_note, passive_qbi_source, passive_scorp_document,
    qbi_record_issues, reconcile_qbi_tracking, repaid_written_note,
    scorp_binding_current_expense_page, scorp_current_loss_statement, scorp_current_qbi_source,
    scorp_document, scorp_fully_excluded_repayment_page, scorp_large_loss, scorp_loss_qbi_source,
    scorp_loss_statement, scorp_no_loss_repayment_page, scorp_note, scorp_portfolio,
    scorp_repaid_reduced_open_account, scorp_retained_expense_page, scorp_risk_history,
    sole_qbi_loss, trading_interest_draft,
};
pub(crate) use situations::{Chosen, every_refusal_item};
pub(in crate::tests) use social_security::{social_prior_year, social_security_net, ssa_1099_net};
pub(in crate::tests) use vehicles::{
    clean_vehicle_business_return, clean_vehicle_claim, new_clean_vehicle, owned_actual_vehicle,
    owned_section179_vehicle, personal_refueling, used_clean_vehicle, vehicle_loan,
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

pub(in crate::tests) use estate::{
    empty_estate_portfolio, estate_foreign_allocations, estate_foreign_amounts,
    estate_foreign_data, estate_foreign_draft, estate_foreign_source,
};

pub(in crate::tests) mod survey;

pub(in crate::tests) use children::noncitizen_child;

pub(in crate::tests) use situations::survivor_with_adult_son;

pub(in crate::tests) use marketplace::marketplace_nonmedical_child;

pub(in crate::tests) use marketplace::marketplace_wage_draft;

mod k1_consolidated;
pub(in crate::tests) use k1_consolidated::{
    k1_distribution_counterparts, k1_page_credit_calls, k1_qbi_statement, k1_reported_sale,
    k1_section1256_sources,
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
    interview_office, itemizing_office, office_claim_mut, office_estate_tax_return,
    office_interview_return, office_itemize, office_line_id, office_return, owned_office_home,
    simplified_claim, standard_office,
};

pub(in crate::tests) mod cohorts;
pub(in crate::tests) use rental::{
    rental_energy_building, rental_energy_claim, rental_energy_return,
};
pub(in crate::tests) mod nonmacrs;

pub(in crate::tests) use rental::{foreign_rental, rental_source_view};

pub(in crate::tests) use returns::{
    dated_payments_mut, page_estimated_sources_mut, unknown_estimated,
};

pub(in crate::tests) use scorporation::{
    complete_scorp_paper, known_fact_scorp_draft, set_scorp_credits,
};
pub(in crate::tests) mod contribution_documents;

mod qsbs;
pub(in crate::tests) use qsbs::{qsbs_document, qsbs_zone_topic_request, remember_qsbs_answers};

/// The Document Center row of a Form 8986 fixture, with its accepted identity.
pub(in crate::tests) fn audit_row(
    audit: crate::FederalBbaAuditInterest,
) -> crate::PartnershipAuditRow {
    crate::PartnershipAuditRow { id: audit.id, document: audit.into_parts().0 }
}

/// Reprints a Form 8986 fixture's Part V interest line and totals.
pub(in crate::tests) fn set_8986_interest(
    paper: &mut crate::Form8986Paper,
    original: i64,
    adjustment: i64,
) {
    let corrected = original + adjustment;
    let item = &mut paper.items[0];
    (item.as_reported, item.adjustment, item.corrected) =
        (Some(original), Some(adjustment), Some(corrected));
    (paper.totals.adjustment, paper.totals.corrected) = (Some(adjustment), Some(corrected));
}
