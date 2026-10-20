use crate::tests::support::client::credit_origin::origin_credit_scenarios;

origin_credit_scenarios! {
    credit: BusinessCredit::CarbonOxide, code: CarbonOxide,
    native_line: None,
    editor_line: (|profile| (profile == "other").then_some(BusinessCredit::CarbonOxide)), native_profile: "aw",
    origin: CarbonOxideCreditOrigin, component: CarbonOxideEquipmentCredit,
    components: equipment, field: carbon_oxide_origin,
    other_field: renewable_facility_origin, allowed: allowed_carbon_oxide_origins,
    dates: (20180601, 20230601),
    identities: ("Facility A / train1", "Facility A / train2"),
    projected: ("Post2022 equipment", "Pre2023 equipment", "Suspended old equipment"),
    env: "TAX_UNIT16_45Q", line_prefix: "business_credit_carbon_oxide",
    profile_error: "unknown 45Q source profile",
    issues: ("CARBON_OXIDE_MIXED_CARRYBACK_ATTRIBUTION_REQUIRED",
        "WARN_CARBON_OXIDE_EXTENDED_CARRYBACK_NEEDS_DATE",
        "CARBON_OXIDE_SOURCE_STATEMENT_INVALID"),
    tests: (
        unit16_45q_actual_original_date_resolves_the_preserved_same_fact_witness,
        unit16_45q_source_components_reconcile_without_inferred_dates_or_origins,
        unit16_45q_community_source_cents_conserve_parent_and_each_actual_equipment,
        unit16_45q_source_editor_save_reopen_restart_and_correction,
        unit16_45q_actual_k1_statement_editor_save_reopen_restart,
        unit16_45q_actual_projected_whole_credit_controls_the_date_rule,
        unit16_45q_suspended_old_source_does_not_refuse_actual_post2022_unused_credit
    ),
    choices: {
    assert_eq!(
        crate::business_credit_page::carbon_oxide_origin_choices(BusinessCredit::CarbonOxide),
        vec![0]
    );
    assert!(
        crate::business_credit_page::carbon_oxide_origin_choices(BusinessCredit::DisabledAccess)
            .is_empty()
    );
    assert_eq!(
        crate::business_credit_page::k1_carbon_oxide_origin_choices(
            crate::business_credit_page::K1CreditCodePage::Other {
                line: BusinessCredit::CarbonOxide
            }
        ),
        vec![0]
    );
    },
}
