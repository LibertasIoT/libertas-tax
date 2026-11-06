// Coverage: State/local source records (2026-09-30): W-2 b/c, 11, 13 sick pay,
// 14a/b and 15-20; W-2G 13-18; 1099-R 14-19; INT 15-17, DIV 14-16, OID
// 12-14 and supplemental issuing-state dollar/percentage allocations and
// government-obligation percentages; B 14-16, G 11a/b and 12, MISC 16-18,
// NEC 5-7, K 6-8. Payer identity, refund jurisdiction and railroad origin are
// retained without inferring state tax or changing federal calculations.
// Source coverage and IRS references: COVERAGE.md. No
// state/local calculation, SALT summation, 1099-Q intake or state K-1 intake.
// Evidence: src/tests/federal/state_return.rs saves all 16 affected document kinds,
// encodes persisted TaxAppData, reopens them, and covers edits, removal,
// source mismatch, absent facts, printed zero versus unknown, fractional
// percentages, percentage bounds and limits.
// Keep these facts beside the accepted income entry: federal projections do
// not retain printed state boxes, payer IDs, or supplemental state allocations.
pub use crate::printed_state::{
    BusinessStateDocument, BusinessStateForm, IncomeStateSource, InvestmentStateFields,
    LocalIncomeTax, PrintedParty, PrintedPartyNameAddress, RefundStateFields,
    RetirementStateFields, SocialSecurityStatement, StateIncomeFields, StateIncomeTax,
    StateLocalFields, StateTaxWithholding, StateWithholdingFields, TaxExemptAllocation,
    TaxExemptStateIncome, TaxPayerIdentity, TaxState, TippedOccupationCode, W2OtherItem,
    W2OtherValue, W2StateFields,
};

impl IncomeStateSource {
    pub(crate) fn from_document(document: &crate::IncomeSource) -> Option<Self> {
        macro_rules! source {
            ($value:expr, $variant:ident) => {
                ($value != &Default::default()).then(|| Self::$variant($value.clone()))
            };
        }
        match document {
            crate::IncomeSource::Wage { value: crate::WageDocument::W2(v) } => {
                source!(&v.state_details(), Wage)
            }
            crate::IncomeSource::Gambling(v) => source!(&v.state_details(), Gambling),
            crate::IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(v),
            } => {
                source!(&v.state_details, Investment)
            }
            crate::IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(v),
            } => {
                source!(&v.state_details, Investment)
            }
            crate::IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Oid1099(v),
            } => {
                source!(&v.state_details, Investment)
            }
            crate::IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form1099B(v),
            } => source!(&v.state_details, Withholding),
            crate::IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form1099DA(v),
            } => source!(&v.state_details, Withholding),
            crate::IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Sale1099B(v),
            } => {
                source!(&v.state_details, Withholding)
            }
            crate::IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(v),
            } => source!(&v.state_details, Withholding),
            crate::IncomeSource::GovernmentPayment { value: v, refund_state, refund_locality }
                if v.state_or_local_income_tax_refunds != 0 =>
            {
                source!(
                    &crate::Refund1099G::with_place(v, *refund_state, refund_locality)
                        .state_details,
                    Refund
                )
            }
            crate::IncomeSource::GovernmentPayment { value: v, .. } => {
                source!(&v.state_details, Withholding)
            }
            crate::IncomeSource::Retirement { value: crate::RetirementDocument::R1099(v) } => {
                source!(&v.state_details, Retirement)
            }
            crate::IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(v),
            } => {
                source!(&crate::Form1099Misc::from(v).state_details, Misc)
            }
            crate::IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099Nec(v),
            } => (v.state_details != Default::default()).then(|| Self::Business {
                documents: alloc::vec![BusinessStateDocument {
                    payer: v.payer.clone(),
                    form: BusinessStateForm::Nec { data: v.state_details.clone() },
                }],
            }),
            crate::IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099K(v),
            } => (!v.state_details.is_empty()).then(|| Self::Business {
                documents: alloc::vec![BusinessStateDocument {
                    payer: v.payer.clone(),
                    form: BusinessStateForm::K { data: crate::Form1099K::from(v).state_details },
                }],
            }),
            crate::IncomeSource::Retirement {
                value: crate::RetirementDocument::Ssa1099(_) | crate::RetirementDocument::Rrb1099(_),
            } => {
                // The typed paper and federal boxes retain statement origin;
                // neither statement supplies a separate state source.
                None
            }
            crate::IncomeSource::Record { value: crate::IncomeRecord::Farm(v) } => {
                Self::business(&v.state_documents)
            }
            crate::IncomeSource::Record { value: crate::IncomeRecord::FarmRental(v) } => {
                Self::business(&v.state_documents)
            }
            _ => None,
        }
    }

    fn business(documents: &[BusinessStateDocument]) -> Option<Self> {
        (!documents.is_empty()).then(|| Self::Business { documents: documents.to_vec() })
    }

    pub(crate) fn apply(&self, document: &mut crate::IncomeSource) -> bool {
        match (self, document) {
            (
                Self::Wage(source),
                crate::IncomeSource::Wage { value: crate::WageDocument::W2(v) },
            ) => v.set_state_details(source.clone()),
            (Self::Gambling(source), crate::IncomeSource::Gambling(v)) => {
                v.set_state_details(source.clone())
            }
            (
                Self::Investment(source),
                crate::IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Investment(source),
                crate::IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Investment(source),
                crate::IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Withholding(source),
                crate::IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form1099B(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Withholding(source),
                crate::IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form1099DA(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Withholding(source),
                crate::IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Withholding(source),
                crate::IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(v),
                },
            ) => v.state_details = source.clone(),
            (
                Self::Withholding(source),
                crate::IncomeSource::GovernmentPayment { value: v, .. },
            ) => v.state_details = source.clone(),
            (
                Self::Refund(source),
                crate::IncomeSource::GovernmentPayment { value: v, refund_state, refund_locality },
            ) => {
                *refund_state = source.refund_state;
                *refund_locality = source.refund_locality.clone();
                v.state_details = source.taxes.clone();
            }
            (
                Self::Retirement(source),
                crate::IncomeSource::Retirement { value: crate::RetirementDocument::R1099(v) },
            ) => v.state_details = source.clone(),
            (
                Self::Misc(source),
                crate::IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Form1099Misc(v),
                },
            ) => v.state_details = source.into(),
            (
                Self::Business { documents },
                crate::IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099Nec(v),
                },
            ) => match documents.first() {
                Some(BusinessStateDocument { payer, form: BusinessStateForm::Nec { data } }) => {
                    v.payer = payer.clone();
                    v.state_details = data.clone()
                }
                _ => return false,
            },
            (
                Self::Business { documents },
                crate::IncomeSource::SelfEmployment {
                    value: crate::SelfEmploymentDocument::Form1099K(v),
                },
            ) => match documents.first() {
                Some(BusinessStateDocument { payer, form: BusinessStateForm::K { data } }) => {
                    v.payer = payer.clone();
                    v.state_details = data.states.iter().map(Into::into).collect()
                }
                _ => return false,
            },
            (
                Self::SocialSecurity { statement: Some(SocialSecurityStatement::SocialSecurity) },
                crate::IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) },
            )
            | (
                Self::SocialSecurity { statement: Some(SocialSecurityStatement::Railroad) },
                crate::IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(_) },
            ) => {}
            (
                Self::Business { documents },
                crate::IncomeSource::Record { value: crate::IncomeRecord::Farm(v) },
            ) => v.state_documents = documents.clone(),
            (
                Self::Business { documents },
                crate::IncomeSource::Record { value: crate::IncomeRecord::FarmRental(v) },
            ) => v.state_documents = documents.clone(),
            _ => return false,
        }
        true
    }
}

pub(crate) fn document_rule(document: &crate::IncomeSource) -> Option<&'static str> {
    IncomeStateSource::from_document(document)
        .is_some_and(|source| !source.valid())
        .then_some("STATE_SOURCE_INVALID")
}
