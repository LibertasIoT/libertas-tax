use crate::tests::*;

pub(in crate::tests) fn timber_cost() -> crate::BusinessTimberCost {
    crate::BusinessTimberCost {
        description: "Synthetic timber planting".into(),
        cost: 2_400_000,
        incurred: 20260401,
        prior_allowed: crate::SourceCents::Reported { amount: 0 },
        facts: crate::ReforestationCost {
            property_key: "Synthetic timber property".into(),
            domestic_commercial_timber: true,
            government_reimbursement: 0,
            reimbursement_taxable: false,
            current_expense_elected: 1_000_000,
            other_current_expense: 0,
            amortizable_basis_elected: 1_400_000,
        },
        original_records_complete: true,
        ended: None,
    }
}

pub(in crate::tests) fn timber_business(cost: crate::BusinessTimberCost) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut business = cash_business(100_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.timber.push(cost);
    add(&mut draft, income(business));
    draft
}

pub(in crate::tests) fn timber_mut(draft: &mut FederalDraft) -> &mut crate::BusinessTimberCost {
    draft
        .income
        .iter_mut()
        .find_map(|entry| match &mut entry.income {
            FederalIncome::Business { data } => data.timber.first_mut(),
            _ => None,
        })
        .expect("synthetic timber business")
}

pub(in crate::tests) fn monthly_history_schedule()
-> crate::kernel::cost_recovery::monthly_history::ActualMonthlySchedule {
    crate::kernel::cost_recovery::monthly_history::ActualMonthlySchedule {
        started: crate::kernel::cost_recovery::Month { year: 2025, month: 1 },
        period_months: 180,
        tax_year: 2026,
        basis_cents: 100,
        prior_recovery_cents: Some(7),
        allowed_through: None,
        allowed_before: None,
    }
}

pub(in crate::tests) fn unrepresentable_timber_return() -> FederalDraft {
    let mut draft = timber_business(timber_cost());
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let mut cost = timber_cost();
    cost.cost = i64::MAX;
    cost.incurred = 20240401;
    cost.prior_allowed = crate::SourceCents::Reported { amount: 300_000 };
    cost.facts.current_expense_elected = 0;
    cost.facts.amortizable_basis_elected = i64::MAX;
    // Eight distinct actual properties are enough to exceed the native annual
    // total, while every original cohort and fraction is individually valid.
    data.timber = (0..8)
        .map(|index| {
            let mut row = cost.clone();
            row.facts.property_key = alloc::format!("Synthetic property {index}");
            row
        })
        .collect();
    draft
}

pub(in crate::tests) fn timber_invalid_editors() -> Vec<(Vec<Business>, &'static str, &'static str)>
{
    use crate::tests::cross::interview::business;
    let mut limit = business(10_000_000);
    limit.more.timber.push(timber_cost());
    let mut duplicate = limit.clone();
    duplicate.more.timber[0].facts.current_expense_elected = 0;
    let mut large = business(10_000_000);
    let draft = unrepresentable_timber_return();
    let FederalIncome::Business { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    large.more.timber = data.timber.clone();
    vec![
        (
            vec![limit],
            "TIMBER_PROPERTY_EXPENSE_LIMIT",
            "[documents/documents/0/document/more/timber/0/facts/current_expense_elected]",
        ),
        (
            vec![duplicate.clone(), duplicate],
            "TIMBER_COHORT_ONCE",
            "[documents/documents/1/document/more/timber/0/facts/property_key]",
        ),
        (
            vec![large],
            "TIMBER_TOTAL_UNREPRESENTABLE",
            "[documents/documents/0/document/more/timber]",
        ),
    ]
}

pub(in crate::tests) fn invalid_legacy_amortization_return(with_timber: bool) -> FederalDraft {
    let mut draft = timber_business(timber_cost());
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    if !with_timber {
        data.timber.clear();
    }
    data.intangibles.push(crate::BusinessIntangible {
        label: "Synthetic invalid legacy source".into(),
        cost: i64::MAX,
        acquired: 20240101,
        business_started: 20240101,
        additions: vec![crate::IntangibleCost { amount: i64::MAX, date: 20250101 }; 2],
        eligible: true,
    });
    draft
}

pub(in crate::tests) fn unrepresentable_timber_operating_return() -> FederalDraft {
    let mut draft = unrepresentable_timber_return();
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.timber.pop();
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Synthetic ordinary cost".into() },
        1,
    ));
    draft
}
