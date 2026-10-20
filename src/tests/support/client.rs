// Preserve the shared client helper API while each domain owns its implementation.
use super::builders;

mod captures;
mod income;
mod injured;
mod lifecycle;
mod native_record_files;
mod property;

pub(in crate::tests) use captures::{
    captured_rental_request_before_nullable_risk, captured_request_before_k1_activity_context,
};
pub(crate) use income::save_farms;
pub(in crate::tests) use income::{
    d6_business_boundary_source, d6_mixed_direct_sources, d6_ptp_source_session,
    education_bond_session, estimate_documents_from_save, save_educator, save_ira_people,
    save_native_cash_sources, save_student_loan,
};
pub(crate) use injured::{
    injured_capacity_accepted_session, injured_capacity_answer_page_is_pure,
    injured_capacity_calculation, injured_capacity_editor, injured_capacity_refusal,
    injured_capacity_source_plan, native_editor,
};
pub(crate) use lifecycle::{TEST_USER, exchange};
pub(in crate::tests) use lifecycle::{error, restart, reviewed, send, status_message, status_text};
pub(in crate::tests) use property::{
    bind_saved_mileage_ids, bind_saved_property_ids, save_native_asset_identities,
};

// Retain the existing topic-call path alongside internal injured-editor use.
#[allow(unused_imports)]
pub(crate) use injured::injured_capacity_editor_reply;

pub(crate) mod k1res;

pub(in crate::tests) use income::{
    check_non_code_l_wage_source, check_wage_classification_source,
    save_noninvestment_election_source, save_reopen_checked_sources, save_reopen_native_sources,
};

pub(in crate::tests) use lifecycle::{
    NativeDraftRecords, draft_from_native_records, native_draft_records, roundtrip_native_draft,
    roundtrip_native_request,
};
pub(in crate::tests) use native_record_files::{
    read_native_draft, read_native_draft_records, write_native_draft, write_native_draft_records,
};

pub(in crate::tests) use injured::{
    complete_niit_allocation, niit_allocation_with_answers, niit_person_editor,
    persist_current_nua_sources, persist_ebl_sources, persist_hobby_niit_sources,
    persist_niit_binding, persist_niit_deduction, persist_niit_zero, restart_stored_niit_draft,
};

pub(in crate::tests) use income::injured_income_source_roundtrip;

mod care;
pub(in crate::tests) mod injured_income;
pub(in crate::tests) use care::care_w2_sources;

pub(in crate::tests) mod injured_credit;

pub(in crate::tests) mod credit_origin;
pub(in crate::tests) mod income_pages;

pub(in crate::tests) mod survey;

pub(in crate::tests) mod qbi_consolidated;

pub(in crate::tests) mod unit50_interview_native;

pub(in crate::tests) use income::save_native_credit_sources;
mod source_controls;
pub(in crate::tests) use source_controls::{
    SecondaryCorrection, SourceScope, current_document_refusal, current_income_key,
    current_income_refusal, current_source_acceptance, current_source_refusal,
    income_editor_resave, source_calculation,
};

mod home;
pub(in crate::tests) mod injured_original;
pub(in crate::tests) use home::save_home_offices;

pub(in crate::tests) mod cohorts;
