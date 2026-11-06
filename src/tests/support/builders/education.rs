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

// A retained statement and its student's one annual answer row.
pub(in crate::tests) fn native_education(
    student: i64,
    expenses: FederalEducationExpenses,
    method: EducationChoice,
) -> (FederalEducation, EducationStudent) {
    (
        FederalEducation {
            id: 0,
            half_time: true,
            graduate: method == EducationChoice::LifetimeLearning,
            student,
            tuition: expenses.tuition,
            scholarships: expenses.assistance,
            prior_tuition_adjustment: crate::SourceCents::NotReported,
            prior_scholarship_adjustment: crate::SourceCents::NotReported,
        },
        EducationStudent {
            student,
            method: Some(method),
            more: StudentMore {
                books_from_school: expenses.school_materials,
                other_books: expenses.other_materials,
                refunds: expenses.refunds,
                insurance_refund_current_expenses: crate::SourceCents::NotReported,
                tax_free_529: expenses.other_benefits,
                scholarship_in_income: expenses.scholarship_in_income,
                american_opportunity_used_up: Some(false),
            },
        },
    )
}

pub(in crate::tests) fn add_education(
    credits: &mut FederalCredits,
    (source, student): (FederalEducation, EducationStudent),
) {
    credits.education.push(education_source(source));
    credits.students.push(student);
}

pub(in crate::tests) fn set_education(
    credits: &mut FederalCredits,
    pairs: Vec<(FederalEducation, EducationStudent)>,
) {
    let (sources, students) =
        pairs.into_iter().map(|(source, student)| (education_source(source), student)).unzip();
    credits.education = sources;
    credits.students = students;
}
