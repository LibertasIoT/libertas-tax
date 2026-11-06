// Preserve the shared client helper API while each domain owns its implementation.
use super::builders;

mod acceptance_bridge;
mod broker;
pub(in crate::tests) mod credit_documents;
mod income;
pub(in crate::tests) use broker::{
    admit_broker_paper, broker_answers_page, broker_b_editor, broker_da_editor, broker_source,
    ordinary_broker_answers,
};
pub(in crate::tests) mod deductions;
mod injured;
mod lifecycle;
mod native_record_files;
pub(in crate::tests) mod own_questions;
mod property;

pub(crate) use income::save_farms;
pub(in crate::tests) use income::{
    answer_only_retirement_treatment, current_installment_session, current_installment_session_for,
    d6_business_boundary_source, d6_mixed_direct_sources, d6_ptp_source_session,
    education_bond_session, estimate_documents_from_save, reject_unavailable_related_components,
    retirement_pso_ira_refusal, save_educator, save_ira_people, save_native_cash_sources,
    save_retirement_fixture, save_student_loan,
};
pub(crate) use injured::{
    injured_capacity_accepted_session, injured_capacity_answer_page_is_pure,
    injured_capacity_calculation, injured_capacity_editor, injured_capacity_refusal,
    injured_capacity_source_plan, native_editor,
};
pub(crate) use lifecycle::{TEST_USER, exchange};
pub(in crate::tests) use lifecycle::{error, restart, reviewed, send, status_message, status_text};
pub(in crate::tests) use property::{
    bind_saved_mileage_ids, bind_saved_property_ids, save_elected_business_asset,
    save_native_asset_identities,
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
pub(in crate::tests) mod charity;
pub(in crate::tests) mod injured_income;
pub(in crate::tests) use care::care_w2_sources;

pub(in crate::tests) mod injured_credit;

pub(in crate::tests) mod credit_origin;
pub(in crate::tests) mod income_pages;

pub(in crate::tests) mod survey;

pub(in crate::tests) mod qbi_consolidated;

pub(in crate::tests) mod unit50_interview_native;

pub(in crate::tests) use income::{admit_native_fixture_facts, save_native_credit_sources};
mod source_controls;
pub(in crate::tests) use source_controls::{
    SecondaryCorrection, SourceScope, current_document_refusal, current_income_key,
    current_income_refusal, current_source_acceptance, current_source_refusal,
    information_return_refusal, railroad_answer_case, retained_marketplace_conflict,
    source_calculation,
};

mod home;
pub(in crate::tests) mod injured_original;
pub(in crate::tests) use home::save_home_offices;

pub(in crate::tests) mod cohorts;
pub(in crate::tests) mod nonmacrs;

pub(in crate::tests) use lifecycle::{answer_no_estimates, answer_payment_method};

mod often_missed;
pub(in crate::tests) use often_missed::{
    answer_often_missed, save_often_missed_marketplace_source,
};

pub(in crate::tests) mod contribution_documents;
pub(in crate::tests) mod undeclared_forms;
