use super::*;

pub(crate) struct RentalContext<'a, 'home> {
    pub(crate) form: crate::related_forms::RelatedFormContext<'a>,
    pub(crate) amt: bool,
    pub(crate) home_use: &'home HomeUseContext,
}

pub(crate) struct RentalSetup<'a> {
    pub(crate) allocated_entries:
        alloc::vec::Vec<alloc::borrow::Cow<'a, crate::FederalIncomeEntry>>,
    pub(crate) native_scorps: alloc::collections::BTreeMap<i64, crate::ScorpK1>,
    pub(crate) initial_issues: usize,
    pub(crate) mid_quarter: bool,
    pub(crate) professional_groups: ProfessionalGroups,
    pub(crate) regular_groups: ProfessionalGroups,
    pub(crate) personal_inputs: alloc::collections::BTreeMap<i64, PersonalRentalInputs>,
    pub(crate) personal: alloc::vec::Vec<(i64, i64, i64)>,
    pub(crate) personal_profit_cents: i64,
    pub(crate) personal_profit: i64,
    pub(crate) magi: i64,
}

pub(crate) fn prepare<'a>(
    context: &RentalContext<'a, '_>,
    magi: i64,
    r: &mut FederalResult,
) -> Result<RentalSetup<'a>, crate::activity_risk::SourceIssue> {
    let form_context = context.form;
    let d = form_context.draft;
    let amt = context.amt;
    let home_use = context.home_use;
    let allocated_entries = crate::community::entries(d);
    let native_scorps: alloc::collections::BTreeMap<_, _> = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Scorp { data } => Some((entry.id, crate::ScorpK1::from(data))),
            _ => None,
        })
        .collect();
    let initial_issues = r.issues.len();
    if let Some(key) = rental_mortgage_rule(d) {
        for section in [crate::TaxSection::Mortgage, crate::TaxSection::Rentals] {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(section),
                "Rental mortgage loans",
                key,
                "Correct the rental's canonical saved Mortgage loans, duplicate allocations or points schedule before estimating.",
            );
        }
    }

    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref()));
    // The election is annual taxpayer-level grouping, not a rewrite of any
    // accepted source history. AMT projects its own amounts; MAGI and NIIT
    // continue to read the regular-tax group.
    let professional_groups = match professional_former_passive(form_context, amt, home_use) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return Err(issue);
        }
    };
    let regular_groups = if amt {
        match professional_former_passive(form_context, false, home_use) {
            Ok(value) => value,
            Err(issue) => {
                issue.report(r);
                return Err(issue);
            }
        }
    } else {
        professional_groups.clone()
    };
    // Retain the actual system-specific cost/history before the existing
    // annual-dollar stages. Never recover these cents from a rounded net.
    let personal_inputs: alloc::collections::BTreeMap<_, _> = allocated_entries
        .iter()
        .filter_map(|entry| {
            let FederalIncome::NonbusinessProperty {
                kind: kind @ NonbusinessPropertyKind::PersonalRental { .. },
                receipts,
                expenses,
                ..
            } = &entry.income
            else {
                return None;
            };
            let regular_expenses_cents = *expenses
                + kind.rental_depreciation(
                    mid_quarter,
                    false,
                    entry.owner.owned(),
                    entry.id,
                    form_context.macrs().as_ref(),
                );
            let expenses_cents = if amt {
                *expenses
                    + kind.rental_depreciation(
                        mid_quarter,
                        true,
                        entry.owner.owned(),
                        entry.id,
                        form_context.macrs().as_ref(),
                    )
            } else {
                regular_expenses_cents
            };
            Some((
                entry.id,
                PersonalRentalInputs {
                    owner: entry.owner.owned(),
                    receipts_cents: *receipts,
                    regular_expenses_cents,
                    expenses_cents,
                    regular_prior_cents: kind.prior_rental_loss(false),
                    prior_cents: kind.prior_rental_loss(amt),
                    investment_recharacterization: kind.investment_rental(),
                },
            ))
        })
        .collect();
    // Schedule 1 supplies personal-property receipts, paid expenses and depreciation.
    // A nondepreciable-property loss remains passive: only positive income
    // is recharacterized. Current losses have no real-estate special allowance.
    let personal: alloc::vec::Vec<_> = allocated_entries
        .iter()
        .filter_map(|e| {
            let input = personal_inputs.get(&e.id)?;
            let prior = input.prior_cents;
            let net = input.receipts_cents - input.expenses_cents - prior;
            (net <= 0 || !input.investment_recharacterization).then_some((e.id, net, prior))
        })
        .collect();
    let personal_profit_cents = personal.iter().map(|(_, net, _)| positive(*net)).sum::<i64>();
    let personal_profit = dollars(personal_profit_cents);
    // MAGI always uses regular-tax history, even in the AMT pass. For a
    // passive activity remove its raw current net. For recharacterized profit
    // retain only the regular net after its own prior loss has been absorbed.
    let mut magi_adjustment_cents = 0;
    for e in &allocated_entries {
        if let FederalIncome::NonbusinessProperty {
            kind: kind @ NonbusinessPropertyKind::PersonalRental { .. },
            receipts,
            ..
        } = &e.income
        {
            let input = &personal_inputs[&e.id];
            let current = *receipts - input.regular_expenses_cents;
            let prior = input.regular_prior_cents;
            magi_adjustment_cents +=
                if kind.investment_rental() { phase::cap(current, [prior]) } else { current };
        }
    }
    let magi = magi - dollars(magi_adjustment_cents) + regular_groups.magi;
    Ok(RentalSetup {
        allocated_entries,
        native_scorps,
        initial_issues,
        mid_quarter,
        professional_groups,
        regular_groups,
        personal_inputs,
        personal,
        personal_profit_cents,
        personal_profit,
        magi,
    })
}
