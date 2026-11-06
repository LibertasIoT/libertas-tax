use super::*;
mod broker_documents;
mod followup_page;
mod oid_workflow;

pub(in crate::tests) mod business;
pub(in crate::tests) mod exchange;
pub(in crate::tests) mod family;
pub(in crate::tests) mod form_2439;
pub(in crate::tests) mod general;
pub(in crate::tests) mod interview;
#[allow(clippy::module_inception)]
pub(in crate::tests) mod investment;
pub(in crate::tests) mod investment_kinds;
pub(in crate::tests) mod investment_statements;
pub(in crate::tests) mod passthrough;
pub(in crate::tests) mod repayment;
pub(in crate::tests) mod retirement;
pub(in crate::tests) mod sale_specials;

mod qbi_direct_rental;
