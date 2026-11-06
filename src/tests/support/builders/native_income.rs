use crate::tests::*;

// 2026-10-16, Required answers / Person attribution: fresh rows remain unanswered.
// Reopened non-Texas and supporting rows explicitly derive known-no applicability
// in document_refund_management_needed; their expected metadata is Some(false).
pub(in crate::tests) fn document(id: i64, document: IncomeSource) -> IncomeDocumentItem {
    owned(Owner::Taxpayer, id, document)
}
pub(in crate::tests) fn owned(owner: Owner, id: i64, document: IncomeSource) -> IncomeDocumentItem {
    IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: vec![crate::community::index(CommunityShare::NotNeeded)],
        investment_caps: match &document {
            IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } => {
                Some(crate::extended_rules::capital_caps(&v.clone().into()))
            }
            _ => None,
        },
        id,
        owner: crate::DocumentOwner::from(owner),
        document,
        preview: None,
        community: CommunityShare::NotNeeded,
    }
}

pub(in crate::tests) fn empty_business_credit_history() -> BusinessCreditHistory {
    BusinessCreditHistory {
        credits: vec![],
        carryforwards: vec![],
        carrybacks: vec![],
        spouse_without_credits: false,
    }
}

pub(in crate::tests) fn credit_construction_return() -> FederalDraft {
    let mut draft = joint_employer_adoption_return();
    let income_source = draft.next_id;
    add(&mut draft, income(qualified_dividend(4_000)));
    let mut bond = direct_tax_credit_bond(
        "INITIAL123",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        12_300,
    );
    bond.answers.origin = Some(TaxCreditOrigin::Entity1097(TaxCreditEntity {
        entity: "Synthetic distribution fund".into(),
        designated_income: 250_000,
        income_already_reported: 150_000,
        income_kind: TaxCreditIncomeKind::RicDistribution,
        income_source: Some(income_source),
    }));
    let mut names = accepted_document_names(&draft, Owner::Taxpayer);
    bond.form.recipient = names.recipient.clone();
    names.issuer = Some(bond.form.issuer.clone());
    bond.printed = Some(names);
    let credits = draft.credits.as_mut().unwrap();
    credits.foreign_tax = ForeignTaxClaim::Statements { eligible: None };
    credits.tax_credit_bonds = vec![bond];
    credits.business_credits = Some(BusinessCreditHistory {
        credits: vec![EnteredBusinessCredit {
            credit: BusinessCredit::CarbonOxide,
            source: CreditSource::Bought,
            amount: 123_400,
            election: CreditElection::None,
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
        }],
        carryforwards: vec![BusinessCreditCarryforward {
            credit: BusinessCredit::ResearchSmallBusiness,
            origin_year: 2025,
            amount: 234_500,
            passive: None,
            filer: CarryFiler::SameFilers,
            research_source: None,
        }],
        carrybacks: vec![BusinessCreditCarryback {
            credit: BusinessCredit::ResearchSmallBusiness,
            year: 2025,
            amount: 345_600,
            research_source: None,
        }],
        spouse_without_credits: false,
    });
    draft
}

// Per-form modules split out by tools/split-hubs; the tool regenerates this block.
#[path = "form_1098_native.rs"]
mod form_1098_native;
#[path = "form_1099_div_native.rs"]
mod form_1099_div_native;
#[path = "form_1099_int_native.rs"]
mod form_1099_int_native;
#[path = "form_1099_oid_native.rs"]
mod form_1099_oid_native;
#[path = "w2_native.rs"]
mod w2_native;
#[allow(unused_imports)]
pub use form_1098_native::*;
#[allow(unused_imports)]
pub use form_1099_div_native::*;
#[allow(unused_imports)]
pub use form_1099_int_native::*;
#[allow(unused_imports)]
pub use form_1099_oid_native::*;
#[allow(unused_imports)]
pub use w2_native::*;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const HUB_SOURCE: &str = concat!(
    include_str!("native_income.rs"),
    include_str!("form_1098_native.rs"),
    include_str!("form_1099_div_native.rs"),
    include_str!("form_1099_int_native.rs"),
    include_str!("form_1099_oid_native.rs"),
    include_str!("w2_native.rs"),
);
// End of the tools/split-hubs block.
