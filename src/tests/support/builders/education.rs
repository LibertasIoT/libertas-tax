use crate::tests::*;

// The American opportunity requirements are stated by choosing the credit.
pub(in crate::tests) fn education_choice(
    method: EducationMethod,
    _requirements: Answer,
) -> EducationChoice {
    match method {
        EducationMethod::AmericanOpportunity => EducationChoice::AmericanOpportunity,
        EducationMethod::LifetimeLearning => EducationChoice::LifetimeLearning,
    }
}
pub(in crate::tests) fn distribution_period(
    year: SaverDistributionYear,
    joint_with_current_spouse: Answer,
) -> SaverDistributionPeriod {
    match year {
        SaverDistributionYear::PriorTwo => SaverDistributionPeriod::PriorTwo {
            joint_with_current_spouse: joint_with_current_spouse == Answer::Yes,
        },
        SaverDistributionYear::PriorOne => SaverDistributionPeriod::PriorOne {
            joint_with_current_spouse: joint_with_current_spouse == Answer::Yes,
        },
        SaverDistributionYear::TaxYear => SaverDistributionPeriod::TaxYear,
        SaverDistributionYear::FollowingYear => {
            SaverDistributionPeriod::FollowingYear { joint_with_current_spouse }
        }
    }
}

pub(in crate::tests) fn education_expenses(tuition: i64) -> FederalEducationExpenses {
    FederalEducationExpenses {
        tuition,
        school_materials: 0,
        other_materials: 0,
        assistance: 0,
        refunds: 0,
        other_benefits: 0,
        scholarship_in_income: crate::ScholarshipInclusion::None,
    }
}

pub(in crate::tests) fn scholarship(
    received: i64,
    restricted: i64,
    qualified_paid: i64,
) -> FederalIncome {
    FederalIncome::Scholarship {
        data: Scholarship::Education {
            data: crate::EducationAward {
                received: received * 100,
                restricted: restricted * 100,
                qualified_paid: qualified_paid * 100,
                degree_candidate: true,
                services: ScholarshipService::None,
                ordinary: true,
            },
        },
    }
}

// A prize or award in Other income: cash, goods won, withholding (all cents).
pub(in crate::tests) fn prize_income(
    cash: i64,
    noncash: i64,
    withholding: i64,
    disposition: PrizeDisposition,
) -> FederalIncome {
    FederalIncome::OtherIncome {
        data: OtherIncomeItem {
            kind: OtherIncomeKind::Prize { noncash, disposition },
            amount: cash,
            withholding,
        },
    }
}

// Native student identity and expense cents, with the selected typed credit.
pub(in crate::tests) fn native_education(
    student: i64,
    expenses: FederalEducationExpenses,
    method: EducationChoice,
) -> FederalEducation {
    FederalEducation { student, expenses, method }
}
