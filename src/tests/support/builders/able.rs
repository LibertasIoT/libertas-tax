use crate::tests::*;

pub(in crate::tests) fn able_distribution(
    gross: i64,
    earnings: i64,
    expenses: i64,
) -> FederalIncome {
    FederalIncome::Able {
        data: AbleDistribution {
            gross: gross * 100,
            earnings: earnings * 100,
            treatment: AbleTreatment::Annual {
                rollover: AbleRollover::None,
                following_expenses: 0,
                expenses: expenses * 100,
            },
        },
    }
}

pub(in crate::tests) fn able_correction(year: u32, earnings: i64) -> FederalIncome {
    FederalIncome::Able {
        data: AbleDistribution {
            gross: (2000 + earnings) * 100,
            earnings: earnings * 100,
            treatment: AbleTreatment::Corrective { year },
        },
    }
}

pub(in crate::tests) fn able_contributions(own: i64, other: i64, value: i64) -> AbleContributions {
    AbleContributions {
        owner: Owner::Taxpayer,
        own: own * 100,
        other: other * 100,
        work: AbleWork::No,
        year_end: value * 100,
        additional_open: false,
    }
}

pub(in crate::tests) fn mixed_able(
    rolled: i64,
    rolled_earnings: i64,
    expenses: i64,
) -> FederalIncome {
    let mut value = able_distribution(10000, 3000, expenses);
    if let FederalIncome::Able { data } = &mut value
        && let AbleTreatment::Annual { rollover, .. } = &mut data.treatment
    {
        *rollover =
            AbleRollover::Qualified { amount: rolled * 100, earnings: rolled_earnings * 100 };
    }
    value
}
