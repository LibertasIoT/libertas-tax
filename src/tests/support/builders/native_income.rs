use crate::tests::*;

pub(in crate::tests) fn native_w2(wages: i64) -> W2 {
    W2 {
        answers: Default::default(),
        state_details: Default::default(),
        wages,
        withholding: wages / 10,
        social_wages: wages,
        social_tax: wages * 62 / 1000,
        medicare_wages: wages,
        medicare_tax: wages * 145 / 10000,
        more: W2More::default(),
    }
}
// Client document values use native cents and remain distinct from FederalIncome fixtures.
pub(in crate::tests) fn native_int1099(payer: alloc::string::String, taxable: i64) -> Int1099 {
    Int1099 {
        payer,
        taxable,
        more: IntMore::default(),
        state_details: Default::default(),
        after_death: None,
        nominee: None,
        adjustments: None,
    }
}

pub(in crate::tests) fn native_div1099(
    payer: alloc::string::String,
    ordinary: i64,
    qualified: i64,
) -> Div1099 {
    Div1099 {
        payer,
        ordinary,
        qualified,
        more: DivMore::default(),
        answers: DivAnswers::default(),
        state_details: Default::default(),
        nominee: None,
        capital_returns: Vec::new(),
    }
}

pub(in crate::tests) fn native_oid1099(
    payer: alloc::string::String,
    oid: i64,
    other_periodic_interest: i64,
) -> Oid1099 {
    Oid1099 {
        payer,
        oid,
        other_periodic_interest,
        more: OidMore::default(),
        state_details: Default::default(),
        nominee: None,
        adjustments: None,
    }
}

// 2026-10-16, Required answers / Person attribution: fresh rows remain unanswered.
// Reopened non-Texas and supporting rows explicitly derive known-no applicability
// in document_refund_management_needed; their expected metadata is Some(false).
pub(in crate::tests) fn document(id: i64, document: IncomeDocument) -> IncomeDocumentItem {
    owned(Owner::Taxpayer, id, document)
}
pub(in crate::tests) fn owned(
    owner: Owner,
    id: i64,
    document: IncomeDocument,
) -> IncomeDocumentItem {
    IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: vec![crate::community::index(CommunityShare::NotNeeded)],
        investment_caps: match &document {
            IncomeDocument::CapitalLossCarryover(v) => {
                Some(crate::extended_rules::capital_caps(&v.clone().into()))
            }
            _ => None,
        },
        id,
        owner,
        document,
        preview: None,
        community: CommunityShare::NotNeeded,
    }
}

// The source inputs of the W-2 refusal table remain typed for oracle recapture.
pub(in crate::tests) fn rejected_w2_cases() -> [(W2, &'static str); 4] {
    let mut duplicate = native_w2(1_000_000);
    duplicate.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        social_tips: 0,
        allocated_tips: 0,
        care_benefits: 0,
        box12: vec![
            Box12Entry { code: W2Code::D, amount: 10_000 },
            Box12Entry { code: W2Code::D, amount: 20_000 },
        ],
        statutory: false,
        retirement_plan: false,
    };
    // One cent above the 2026 $184,500 Social Security wage base.
    let mut above_base = native_w2(18_450_000);
    above_base.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        social_tips: 1,
        allocated_tips: 0,
        care_benefits: 0,
        box12: vec![],
        statutory: false,
        retirement_plan: false,
    };
    let mut withholding = native_w2(1_000_000);
    withholding.withholding = 1_000_001;
    let mut overflow = native_w2(1_010_000);
    overflow.social_wages = i64::MAX;
    overflow.more.social_tips = 1;
    [
        (duplicate, "W2_BOX12_CODE_TWICE"),
        (above_base, "W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE"),
        (withholding, "W2_WITHHOLDING_ABOVE_WAGES"),
        (overflow, "W2_SOCIAL_TAX_ABOVE_WAGES"),
    ]
}

pub(in crate::tests) fn first_lender_mortgage(
    interest: i64,
    principal: i64,
    average: AverageBalance,
) -> Mortgage1098 {
    Mortgage1098 {
        id: 0,
        interest,
        principal,
        originated: 20200601,
        refund: 0,
        insurance: 0,
        points: Points::None,
        more: MortgageMore {
            details: MortgageDetails::FirstLender(MortgageFirstLender {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average,
            }),
            ..MortgageMore::default()
        },
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
    bond.origin = TaxCreditOrigin::Entity1097(TaxCreditEntity {
        entity: "Synthetic distribution fund".into(),
        designated_income: 250_000,
        income_already_reported: 150_000,
        income_kind: TaxCreditIncomeKind::RicDistribution,
        income_source: Some(income_source),
    });
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
