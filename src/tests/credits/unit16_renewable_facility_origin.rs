use crate::tests::support::client::credit_origin::origin_credit_scenarios;

origin_credit_scenarios! {
    credit: BusinessCredit::RenewableElectricitySpecified, code: RenewableElectricity,
    native_line: Some(BusinessCredit::RenewableElectricitySpecified),
    editor_line: (|_| Some(BusinessCredit::RenewableElectricitySpecified)), native_profile: "renewable",
    origin: RenewableFacilityCreditOrigin, component: RenewableFacilityCredit,
    components: facilities, field: renewable_facility_origin,
    other_field: carbon_oxide_origin, allowed: allowed_renewable_facility_origins,
    dates: (20221231, 20230101),
    identities: ("Facility A", "Facility B"),
    projected: ("Post2022 facilities", "Pre2023 facilities", "Suspended old facilities"),
    env: "TAX_UNIT16_RENEWABLE", line_prefix: "business_credit_renewable_electricity_specified",
    profile_error: "unknown renewable4e source profile",
    issues: ("RENEWABLE_FACILITY_MIXED_CARRYBACK_ATTRIBUTION_REQUIRED",
        "WARN_RENEWABLE_FACILITY_EXTENDED_CARRYBACK_NEEDS_DATE",
        "RENEWABLE_FACILITY_SOURCE_STATEMENT_INVALID"),
    tests: (
        unit16_renewable_actual_original_date_resolves_the_preserved_same_fact_witness,
        unit16_renewable_source_components_reconcile_without_inferred_dates_or_origins,
        unit16_renewable_community_source_cents_conserve_parent_and_each_actual_facilities,
        unit16_renewable_source_editor_save_reopen_restart_and_correction,
        unit16_renewable_actual_k1_statement_editor_save_reopen_restart,
        unit16_renewable_actual_projected_whole_credit_controls_the_date_rule,
        unit16_renewable_suspended_old_source_does_not_refuse_actual_post2022_unused_credit
    ),
    choices: {
    assert_eq!(
        crate::business_credit_page::renewable_facility_origin_choices(
            BusinessCredit::RenewableElectricitySpecified
        ),
        vec![0]
    );
    assert!(
        crate::business_credit_page::renewable_facility_origin_choices(
            BusinessCredit::DisabledAccess
        )
        .is_empty()
    );
    assert!(
        crate::business_credit_page::renewable_facility_origin_choices(
            BusinessCredit::RenewableElectricity
        )
        .is_empty()
    );
    assert!(
        crate::business_credit_page::k1_renewable_facility_origin_choices(
            crate::business_credit_page::K1CreditCodePage::RenewableElectricity {
                line: BusinessCredit::RenewableElectricity
            }
        )
        .is_empty()
    );
    assert_eq!(
        crate::business_credit_page::k1_renewable_facility_origin_choices(
            crate::business_credit_page::K1CreditCodePage::Other {
                line: BusinessCredit::RenewableElectricitySpecified
            }
        ),
        vec![0]
    );
    },
}
