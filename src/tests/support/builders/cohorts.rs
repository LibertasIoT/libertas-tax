use crate::tests::*;
pub(in crate::tests) fn cohort(
    cost: i64,
    incurred: u32,
    began: u32,
    kind: OtherAmortizationKind,
) -> OtherAmortizableCost {
    OtherAmortizableCost {
        description: "Synthetic cost cohort".into(),
        cost,
        incurred,
        began: if matches!(&kind, OtherAmortizationKind::Research { facts } if matches!(facts.treatment, ResearchTreatment::CurrentExpense))
        {
            None
        } else {
            Some(began)
        },
        prior_allowed: if matches!(&kind, OtherAmortizationKind::Research { facts } if matches!(facts.treatment, ResearchTreatment::CurrentExpense))
        {
            SourceCents::Unavailable
        } else {
            SourceCents::NotReported
        },
        kind,
        ended: None,
    }
}
pub(in crate::tests) fn domestic_location() -> ResearchLocation {
    ResearchLocation::Domestic { credit_adjustment: SourceCents::NotReported }
}
pub(in crate::tests) fn capitalization() -> ResearchTreatment {
    ResearchTreatment::Capitalize {
        election: ResearchCapitalization {
            months: 60,
            first_benefits: Some(20260701),
            transfer: None,
        },
    }
}
pub(in crate::tests) fn old_domestic(
    transition: DomesticResearchAcceleration,
) -> ResearchTreatment {
    ResearchTreatment::OldDomesticMidYear { transition }
}
pub(in crate::tests) fn research(
    location: ResearchLocation,
    treatment: ResearchTreatment,
) -> ResearchCost {
    ResearchCost { location, treatment, property: None }
}

pub(in crate::tests) fn cost(
    cost: i64,
    began: u32,
    kind: OtherAmortizationKind,
) -> OtherAmortizableCost {
    OtherAmortizableCost {
        description: "Synthetic cost cohort".into(),
        cost,
        incurred: began,
        began: if matches!(&kind, OtherAmortizationKind::Research { facts } if matches!(facts.treatment, ResearchTreatment::CurrentExpense))
        {
            None
        } else {
            Some(began)
        },
        prior_allowed: if matches!(&kind, OtherAmortizationKind::Research { facts } if matches!(facts.treatment, ResearchTreatment::CurrentExpense))
        {
            SourceCents::Unavailable
        } else {
            SourceCents::NotReported
        },
        kind,
        ended: None,
    }
}
pub(in crate::tests) fn creative(cost_cents: i64, began: u32) -> OtherAmortizableCost {
    cost(
        cost_cents,
        began,
        OtherAmortizationKind::Creative {
            facts: CreativePropertyCost {
                first_capitalized: 20200101,
                not_set_for_production_three_years: true,
                film_production_trade: true,
                book_writeoff: began,
                original_election: true,
            },
        },
    )
}

pub(in crate::tests) fn pollution() -> OtherAmortizableCost {
    cohort(
        1_200_000,
        20251201,
        20260101,
        OtherAmortizationKind::Pollution {
            facts: PollutionControlCost {
                state_certified: true,
                federal_certified: true,
                facility_qualifies: true,
                plant_first_operation: 19700101,
                coal_atmospheric: false,
                useful_life_months: 120,
                amortizable_allocation: 1_200_000,
                bonus: 0,
                start_choice: PollutionStart::NextMonth,
                terminated: None,
            },
        },
    )
}
pub(in crate::tests) fn lease() -> OtherAmortizableCost {
    cohort(
        1_200_000,
        20260701,
        20260701,
        OtherAmortizationKind::Lease {
            facts: LeaseAcquisitionCost {
                remaining_term_months: 60,
                renewal_months: 60,
                cost_attributed_to_remaining_term: 900_000,
            },
        },
    )
}
pub(in crate::tests) fn sale(proceeds: i64) -> AmortizationEnd {
    AmortizationEnd {
        date: 20260901,
        how: DisposalKind::Sold {
            proceeds,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    }
}
pub(in crate::tests) fn return_with_cohort(cost: OtherAmortizableCost) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut source = cash_business(100_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.other_amortization.push(cost);
    add(&mut draft, income(source));
    draft
}

pub(in crate::tests) fn linked_research_business() -> Business {
    let mut page = crate::tests::cross::interview::business(10_000_000);
    page.assets.push(crate::tests::cross::interview::asset(1_000_000, 20260301, AssetClass::Five));
    let facts = research(ResearchLocation::Foreign, ResearchTreatment::ForeignMidYear);
    page.more.other_amortization.push(
        cohort(1_200_000, 20260301, 20260701, OtherAmortizationKind::Research { facts }).into(),
    );
    page
}

pub(in crate::tests) fn converted_research_business() -> Business {
    let mut page = crate::tests::cross::interview::business(10_000_000);
    let mut asset = crate::tests::cross::interview::asset(840_000, 20260701, AssetClass::Five);
    asset.gds_mut().unwrap().history =
        AssetHistory::New { first_year: FirstYearDeduction::Regular };
    page.assets.push(asset);
    let mut cost = cohort(
        1_200_000,
        20250101,
        20250101,
        OtherAmortizationKind::Research {
            facts: research(
                domestic_location(),
                ResearchTreatment::Capitalize {
                    election: ResearchCapitalization {
                        months: 60,
                        first_benefits: Some(20250101),
                        transfer: None,
                    },
                },
            ),
        },
    );
    cost.prior_allowed = SourceCents::Reported { amount: 240_000 };
    page.more.other_amortization.push(cost.into());
    page
}

pub(in crate::tests) fn cohort_business(draft: &FederalDraft) -> (i64, &FederalBusiness) {
    draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Business { data } => Some((entry.id, data)),
            _ => None,
        })
        .expect("synthetic business")
}
