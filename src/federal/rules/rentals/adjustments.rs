use super::*;

// The prior unallowed loss of an activity with active participation this
// year that the special allowance does not admit.
pub(crate) fn ineligible_prior(allowance: bool, history: &crate::RentalHistory, amt: bool) -> i64 {
    if allowance && !history.active_in_loss_year { dollars(history.balance(amt)) } else { 0 }
}

// Reg. 1.469-2T(f)(3): this basis test applies to every rental type.
pub(crate) fn nondepreciable(v: &FederalRental) -> bool {
    let total: i128 = v.buildings.iter().map(|b| i128::from(b.cost)).sum::<i128>()
        + v.assets.iter().map(|a| i128::from(a.cost)).sum::<i128>();
    let depreciable: i128 = v.buildings.iter().map(|b| i128::from(b.cost - b.land)).sum::<i128>()
        + v.assets.iter().map(|a| i128::from(a.cost)).sum::<i128>();
    total > 0 && depreciable * 10 < total * 3
}

pub(crate) fn rental_risk(
    v: &FederalRental,
    ordinary: i64,
    sale: i64,
    amt: bool,
) -> Result<RentalRisk, crate::activity_risk::ProjectionIssue> {
    match &v.at_risk {
        Some(crate::RentalAtRisk::All) => Ok(RentalRisk { ordinary, sale, recapture: Ok(0) }),
        Some(crate::RentalAtRisk::Limited { record, .. }) => {
            let record = record.as_deref().ok_or(crate::activity_risk::ProjectionIssue::Source(
                "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
            ))?;
            let activity = crate::activity_risk::project_record(
                &[
                    (crate::activity_risk::Part1Line::Operating, ordinary),
                    (crate::activity_risk::Part1Line::Property, sale),
                ],
                record,
                &crate::RiskEffectiveCategory::HoldingRealProperty,
                amt,
            )?;
            Ok(RentalRisk::from_activity(&activity))
        }
        None => Err(crate::activity_risk::ProjectionIssue::Source(
            "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
        )),
    }
}

pub(crate) fn rental_source_risk(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    _property: &FederalRental,
    amt: bool,
    sale: i64,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Option<RentalRisk>, crate::activity_risk::ProjectionIssue> {
    let d = form_context.draft;
    let Some(whole) = d.income.iter().find(|entry| entry.id == id) else {
        return Ok(None);
    };
    let allocated = crate::community::record(d, whole);
    let FederalIncome::Rental { data: v } = allocated.as_ref() else {
        return Ok(None);
    };
    let projected = v.clone();
    let usage = rental_use_core(
        &projected,
        amt,
        crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref())),
        home_use.and_then(|context| context.itemizes).unwrap_or(true),
        home_use.and_then(|context| home_fixed_allowed(form_context, id, context.agi, amt)),
        &rental_expense_rows(form_context, whole, false),
        Some((d, id, false, home_use)),
        Some(form_context.macrs().as_ref()),
    )
    .map_err(|(key, _)| crate::activity_risk::ProjectionIssue::Source(key))?;
    let ordinary = if !usage.current_casualty_losses.is_empty() {
        let prior_casualty =
            usage.home_third_tier.and_then(|tiers| tiers.allowed_prior_casualty_cents).ok_or(
                crate::activity_risk::ProjectionIssue::Source(crate::rental_home_carry::CHARACTER),
            )?;
        usage.net.checked_add(prior_casualty).ok_or(
            crate::activity_risk::ProjectionIssue::Source(crate::rental_home_carry::OVERFLOW),
        )?
    } else {
        usage.net
    };
    Ok(Some(rental_risk(v, ordinary, sale, amt)?))
}

pub(crate) fn rental_at_risk(
    v: &FederalRental,
    id: i64,
    raw_sale: i64,
    amt: bool,
    usage: &mut RentalUse,
    r: &mut FederalResult,
) -> Result<Option<i64>, crate::activity_risk::ProjectionIssue> {
    let Some(crate::RentalAtRisk::Limited { record, .. }) = &v.at_risk else {
        // The source admission above distinguishes a missing answer from All.
        let projection = rental_risk(v, usage.net, raw_sale, amt)?;
        usage.net = projection.ordinary_income()?;
        projection.recapture?;
        return Ok(Some(0));
    };
    let record = record.as_deref().ok_or(crate::activity_risk::ProjectionIssue::Source(
        "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
    ))?;
    let activity = crate::activity_risk::project_record(
        &[
            (crate::activity_risk::Part1Line::Operating, usage.net),
            (crate::activity_risk::Part1Line::Property, raw_sale),
        ],
        record,
        &crate::RiskEffectiveCategory::HoldingRealProperty,
        amt,
    )?;
    let prior_allowed = crate::activity_risk::prior_operating_allowed(
        match &usage.current_operating_for_risk {
            Some(amount) => Some(exact::rounded(amount).ok_or(
                crate::activity_risk::ProjectionIssue::Source("RENTAL_EXPENSE_AMOUNT_UNAVAILABLE"),
            )?),
            None => None,
        },
        usage.prior_at_risk_cents,
        activity.components.suspended[0],
    )?;
    let projection = RentalRisk::from_activity(&activity);
    usage.net = projection.ordinary_income()?;
    projection.recapture?;
    report_risk_projection(id, amt, "income.rental.at_risk", activity, r)?;
    Ok(prior_allowed)
}

pub(crate) fn report_risk_projection(
    id: i64,
    amt: bool,
    input: &str,
    activity: crate::activity_risk::ActivityProjection,
    r: &mut FederalResult,
) -> Result<i64, crate::activity_risk::ProjectionIssue> {
    let recapture = activity.recapture.clone()?;
    let sum = |values: &[i64]| {
        crate::kernel::ratio::checked_sum(values.iter().copied()).ok_or(
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    };
    let business = input == "income.business.at_risk";
    let property_carry = if business {
        sum(&activity.components.suspended.iter().skip(2).step_by(2).copied().collect::<Vec<_>>())?
    } else {
        *activity.components.suspended.last().expect("activity component")
    };
    let property_allowed = if business {
        sum(&activity.components.allowed.iter().skip(2).step_by(2).copied().collect::<Vec<_>>())?
    } else {
        *activity.components.allowed.last().expect("activity component")
    };
    let kind = if business { "business" } else { "rental" };
    let total_carry = sum(&activity.components.suspended)?;
    let prefix = if amt { "amt_" } else { "" };
    emit_lines(
        r,
        &mut (),
        [
            ("at_risk_carryover", total_carry),
            ("at_risk_ordinary_carryover", activity.components.suspended[0]),
            ("at_risk_section1231_carryover", property_carry),
            ("at_risk_allowed_ordinary", activity.components.allowed[0]),
            ("at_risk_allowed_section1231", property_allowed),
        ],
        |(suffix, amount), _, _| {
            if activity.components.allowed.len() == 1 && suffix.contains("section1231") {
                return None;
            }
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                LineValue::MoneyCents(amount),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            ))
        },
    );
    if activity.components.allowed.len() == 3 || (business && activity.components.allowed.len() > 1)
    {
        let ordinary_property_carry = if business {
            sum(&activity
                .components
                .suspended
                .iter()
                .skip(1)
                .step_by(2)
                .copied()
                .collect::<Vec<_>>())?
        } else {
            activity.components.suspended[1]
        };
        let ordinary_property_allowed = if business {
            sum(&activity
                .components
                .allowed
                .iter()
                .skip(1)
                .step_by(2)
                .copied()
                .collect::<Vec<_>>())?
        } else {
            activity.components.allowed[1]
        };
        emit_lines(
            r,
            &mut (),
            [
                ("at_risk_ordinary_property_carryover", ordinary_property_carry),
                ("at_risk_allowed_ordinary_property", ordinary_property_allowed),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &[input],
                    "https://www.irs.gov/instructions/i6198",
                    msg!("EXPLAIN_RENTAL_AT_RISK"),
                ))
            },
        );
    }
    emit_lines(
        r,
        &mut (),
        ["at_risk_recapture", "at_risk_recapture_deduction_next_year"],
        |suffix, _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}{kind}_{id}_{suffix}"),
                LineValue::MoneyCents(recapture),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK_RECAPTURE"),
            ))
        },
    );
    if !amt {
        emit_lines(r, &mut (), activity.lines, |(printed, amount), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form6198, printed)),
                &alloc::format!("{kind}_{id}_form6198_line{printed}"),
                LineValue::MoneyCents(amount),
                &[input],
                "https://www.irs.gov/instructions/i6198",
                msg!("EXPLAIN_RENTAL_AT_RISK"),
            ))
        });
    }
    Ok(recapture)
}

// Canonical identities are durable source facts, never list positions or echoed catalogs.
pub(crate) fn rental_mortgage_roots(d: &FederalDraft) -> alloc::vec::Vec<i64> {
    let claims = crate::related_forms::effective_claimants(d);
    crate::related_forms::catalog(d)
        .into_iter()
        .filter(|source| {
            source.kind == crate::related_forms::DocumentKind::Mortgage1098
                && claims.contains_key(&source.id)
        })
        .map(|source| source.id)
        .collect()
}

// The common component owner supplies both explicit links and sole eligible defaults.
pub(crate) fn rental_mortgage_keys(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    activity: i64,
) -> Vec<i64> {
    let d = form_context.draft;
    let claims = form_context.claims;
    crate::related_forms::catalog(d)
        .into_iter()
        .filter(|source| {
            source.kind == crate::related_forms::DocumentKind::Mortgage1098
                && claims.get(&source.id) == Some(&activity)
        })
        .map(|source| source.id)
        .collect()
}

pub(crate) fn rental_mortgage_rule(d: &FederalDraft) -> Option<&'static str> {
    rental_mortgage_issue(d).map(|(_, issue)| issue.key)
}

pub(crate) fn rental_mortgage_issue(
    d: &FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    if let Some(failure) = crate::estate_trust::mortgage_graph_issue(d) {
        return Some(failure);
    }
    for e in &d.income {
        let FederalIncome::Rental { data } = &e.income else {
            continue;
        };
        if let Some(issue) = crate::related_forms::activity_issue(d, e) {
            return Some((e.id, issue));
        }
        if !matches!(data.personal_use, RentalPersonalUse::No)
            && data.expenses.iter().any(|v| {
                matches!(v.category, RentalExpenseCategory::MortgageInterest) && v.amount != 0
            })
        {
            return Some((
                e.id,
                ValidationIssue::new("RENTAL_MORTGAGE_SINGLE_SOURCE")
                    .at_field("mortgage_interest")
                    .in_field("expenses"),
            ));
        }
    }
    None
}

// The first-lender loan a current Form 8396 certificate names, and the
// credit on it in cents: section 163(g) reduces the deduction for interest
// on that loan, the rental share of it on Schedule E.
pub(crate) fn certified_loan(d: &FederalDraft) -> Option<(i64, i64)> {
    let c = d.credits.as_ref()?;
    let credit = c.mortgage_credit.as_ref()?;
    credit.current.as_ref()?;
    let loan = credit.loan?;
    d.deductions
        .as_ref()?
        .mortgage_loans()
        .unwrap_or_default()
        .iter()
        .any(|l| l.id == loan && matches!(l.details, crate::MortgageLoanDetails::FirstLender(_)))
        .then(|| {
            (
                loan,
                crate::federal_rules::mortgage_credit_line3(c, &mut FederalResult::empty(0)) * 100,
            )
        })
}

pub(crate) fn loan_credit(certified: Option<(i64, i64)>, canonical_id: i64) -> i64 {
    certified.filter(|(loan, _)| *loan == canonical_id).map_or(0, |(_, credit)| credit)
}

// The rental share of the certified loan's credit; the rest reduces Schedule A.
pub(crate) fn rental_credit_share(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    credit_cents: i64,
) -> i64 {
    let d = form_context.draft;
    let Some((loan, _)) = certified_loan(d) else {
        return 0;
    };
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data }
                if rental_mortgage_keys(form_context, e.id).contains(&loan) =>
            {
                Some(rental_fraction(data, credit_cents))
            }
            _ => None,
        })
        .sum()
}

// Section 280A(d)(1): the dwelling was used as a residence when personal
// use exceeded the greater of 14 days or 10% of the days rented at a fair
// rental; an overlap day is personal use and not a fair-rental day here.
pub(crate) fn used_as_home(v: &FederalRental) -> bool {
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, overlap_days, .. } =
        v.personal_use
    else {
        return false;
    };
    let personal = i64::from(personal_days) + i64::from(overlap_days);
    personal > 14 && personal * 10 > i64::from(fair_rental_days) - i64::from(overlap_days)
}

pub(crate) fn rental_fraction(v: &FederalRental, cents: i64) -> i64 {
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, .. } = v.personal_use else {
        return 0;
    };
    let fair = i64::from(fair_rental_days);
    let personal = i64::from(personal_days);
    if fair + personal == 0 || (used_as_home(v) && fair < 15) {
        0
    } else {
        fraction(cents, fair, fair + personal)
    }
}

// The part of a linked loan amount that leaves Schedule A: the rental share,
// or the whole amount when the loan's second home is not used as a residence
// under section 280A(d)(1), so it is not a qualified residence (section
// 163(h)(4)(A)(i)(II); Pub. 936 "Second home rented out") and its personal
// share is nondeductible personal interest.
pub(crate) fn linked_loan_share(
    d: &FederalDraft,
    data: &FederalRental,
    l: &crate::mortgage_limit::LoanAllowance,
    cents: i64,
) -> i64 {
    let second = d
        .deductions
        .as_ref()
        .map_or_else(Vec::new, |v| v.mortgage_loans().unwrap_or_default())
        .iter()
        .any(|loan| {
            loan.id == l.canonical_id
                && matches!(
                    loan.home(),
                    crate::MortgageHome::Second | crate::MortgageHome::SecondVehicle
                )
        });
    if second && !used_as_home(data) { cents } else { rental_fraction(data, cents) }
}

pub(crate) fn rental_loan_allowances(
    d: &FederalDraft,
) -> alloc::vec::Vec<crate::mortgage_limit::LoanAllowance> {
    crate::mortgage_limit::loan_allowances(
        &d.deductions.as_ref().map_or_else(Vec::new, |v| v.mortgage_loans().unwrap_or_default()),
        d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate),
    )
}

// Whether a linked loan's points reach this rental spread over the term.
pub(crate) fn straight_lined_points(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalRental,
) -> bool {
    let d = form_context.draft;
    let keys = rental_mortgage_keys(form_context, id);
    rental_fraction(v, 1_000_000) > 0
        && d.deductions
            .as_ref()
            .map_or_else(Vec::new, |ded| ded.mortgage_loans().unwrap_or_default())
            .iter()
            .any(|l| {
                keys.contains(&l.id)
                    && matches!(
                        l.points,
                        crate::MortgagePoints::Amortized { .. }
                            | crate::MortgagePoints::Improvement { .. }
                    )
            })
}

pub(crate) fn whole_rental_expense_rows(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    v: &FederalRental,
) -> Vec<crate::extended_model::RentalExpenseAmount> {
    let d = form_context.draft;
    let keys = rental_mortgage_keys(form_context, id);
    let mut rows = RentalExpense::exact_rows(&v.expenses);
    if keys.is_empty() {
        return rows;
    }
    rows.retain(|row| row.category != RentalExpenseCategory::MortgageInterest);
    let mut interest = exact::whole(0);
    let mut insurance = exact::whole(0);
    let certified = certified_loan(d);
    for loan in rental_loan_allowances(d).iter().filter(|loan| keys.contains(&loan.canonical_id)) {
        interest += exact::whole(positive(
            loan.qualified_interest_cents - loan_credit(certified, loan.canonical_id),
        )) + exact::whole(loan.rental_points_cents);
        insurance += exact::whole(loan.qualified_insurance_cents);
    }
    for (category, amount) in [
        (RentalExpenseCategory::MortgageInterest, interest),
        (RentalExpenseCategory::Insurance, insurance),
    ] {
        if let Some(row) = rows.iter_mut().find(|row| {
            row.category == category && row.treatment == crate::BusinessExpenseKind::Ordinary
        }) {
            row.amount += amount;
        } else {
            rows.push(crate::extended_model::RentalExpenseAmount {
                category,
                amount,
                treatment: crate::BusinessExpenseKind::Ordinary,
            });
        }
    }
    rows
}

pub(crate) fn rental_expense_rows(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    entry: &FederalIncomeEntry,
    property: bool,
) -> Vec<crate::extended_model::RentalExpenseAmount> {
    let d = form_context.draft;
    let FederalIncome::Rental { data } = &entry.income else { unreachable!("rental source") };
    let mut rows = whole_rental_expense_rows(form_context, entry.id, data);
    if let Some(share) = d.setup.as_ref().and_then(|setup| {
        crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
    }) {
        for row in &mut rows {
            row.amount = if property { share.for_gains() } else { share }.filer_exact(&row.amount);
        }
    }
    rows
}

pub(crate) fn whole_rental_net(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    entry: &FederalIncomeEntry,
    home_use: Option<&HomeUseContext>,
) -> Result<i64, (&'static str, &'static str)> {
    let d = form_context.draft;
    let FederalIncome::Rental { data } = &entry.income else { unreachable!("rental source") };
    rental_use_core(
        data,
        false,
        false,
        true,
        None,
        &whole_rental_expense_rows(form_context, entry.id, data),
        Some((d, entry.id, true, home_use)),
        Some(form_context.macrs().as_ref()),
    )
    .map(|usage| usage.net)
}

// Only taxes are entered on Rental. Linked loan facts already belong to Schedule A;
// its rental components are removed by home_mortgage_shares, never added again.
pub(crate) fn home_personal_shares(d: &FederalDraft) -> [Exact; 2] {
    let mut shares = core::array::from_fn(|_| exact::whole(0));
    for entry in &d.income {
        let FederalIncome::Rental { data } = &entry.income else {
            continue;
        };
        if matches!(data.personal_use, RentalPersonalUse::No) {
            continue;
        }
        for e in &data.expenses {
            if matches!(e.category, RentalExpenseCategory::TaxesAndLicenses) {
                shares[1] += exact::whole(e.amount) - exact::whole(rental_fraction(data, e.amount));
            }
        }
    }
    shares
}

// 8a interest,8b interest,8a points,8c points,pre-phaseout premiums,AMT vehicle adjustment.
pub(crate) fn home_mortgage_shares(
    form_context: crate::related_forms::RelatedFormContext<'_>,
) -> [i64; 6] {
    let d = form_context.draft;
    let mut shares = [0; 6];
    let allowances = rental_loan_allowances(d);
    for e in &d.income {
        let FederalIncome::Rental { data } = &e.income else {
            continue;
        };
        let keys = rental_mortgage_keys(form_context, e.id);
        for l in allowances.iter().filter(|l| keys.contains(&l.canonical_id)) {
            shares[usize::from(!l.reported_on_1098)] +=
                linked_loan_share(d, data, l, l.regular_interest_cents);
            shares[2 + usize::from(!l.points_reported_on_1098)] +=
                linked_loan_share(d, data, l, l.regular_points_cents);
            shares[4] += linked_loan_share(d, data, l, l.regular_insurance_cents);
            shares[5] += linked_loan_share(
                d,
                data,
                l,
                l.regular_interest_cents - l.amt_interest_cents + l.regular_points_cents
                    - l.amt_points_cents,
            );
        }
    }
    shares
}

pub(crate) fn home_vehicle_insurance_share(
    form_context: crate::related_forms::RelatedFormContext<'_>,
) -> i64 {
    let d = form_context.draft;
    let allowances = rental_loan_allowances(d);
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data } => Some((e.id, data)),
            _ => None,
        })
        .map(|(id, data)| {
            let keys = rental_mortgage_keys(form_context, id);
            allowances
                .iter()
                .filter(|l| keys.contains(&l.canonical_id))
                .map(|l| {
                    linked_loan_share(d, data, l, l.regular_insurance_cents - l.amt_insurance_cents)
                })
                .sum::<i64>()
        })
        .sum()
}

pub(crate) fn home_fixed_allowed(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    agi: [i64; 2],
    amt: bool,
) -> Option<[Exact; 3]> {
    let d = form_context.draft;
    let ded = d.deductions.as_ref()?;
    let mut total_rental_taxes = exact::whole(0);
    let mut target = core::array::from_fn(|_| exact::whole(0));
    let allowances = rental_loan_allowances(d);
    let certified = certified_loan(d);
    let separate = d.setup.as_ref()?.filing_status() == FederalStatus::Separate;
    for entry in &d.income {
        let FederalIncome::Rental { data } = &entry.income else {
            continue;
        };
        if matches!(data.personal_use, RentalPersonalUse::No) {
            continue;
        }
        for expense in &data.expenses {
            if matches!(expense.category, RentalExpenseCategory::TaxesAndLicenses) {
                let part = rental_fraction(data, expense.amount);
                total_rental_taxes += exact::whole(part);
                if entry.id == id {
                    target[1] += exact::whole(part);
                }
            }
        }
        if entry.id == id {
            let keys = rental_mortgage_keys(form_context, entry.id);
            for l in allowances.iter().filter(|l| keys.contains(&l.canonical_id)) {
                let personal_points = if amt { l.amt_points_cents } else { l.regular_points_cents };
                let eligible_points = phase::cap(l.rental_points_cents, [personal_points]);
                target[0] += exact::whole(rental_fraction(
                    data,
                    positive(
                        (if amt { l.amt_interest_cents } else { l.regular_interest_cents })
                            - loan_credit(certified, l.canonical_id),
                    ) + eligible_points,
                ));
                target[2] += exact::whole(rental_fraction(
                    data,
                    crate::mortgage_limit::insurance_phaseout_cents(
                        if amt { l.amt_insurance_cents } else { l.regular_insurance_cents },
                        agi[0],
                        separate,
                    ),
                ));
            }
        }
    }
    let cap = exact::whole(crate::federal_rules::salt_cap_current_year(
        d.setup.as_ref()?.filing_status(),
        agi[1],
    )) * exact::whole(100);
    let personal_taxes = exact::whole(ded.state_local_taxes()) + &home_personal_shares(d)[1];
    let allowed = phase::cap(
        phase::floor(cap - personal_taxes, exact::whole(0)),
        [total_rental_taxes.clone()],
    );
    // Section 56(b)(1)(A)(ii): AMT keeps the rental tax share under
    // Worksheet 5-1 line 3, rather than treating it as line 2b.
    target[1] = if total_rental_taxes > exact::whole(0) && !amt {
        exact::rounded_exact(&(&target[1] * allowed / total_rental_taxes))
    } else {
        exact::whole(0)
    };
    Some(target)
}

// A wholly separate spouse source contributes no activity to this return.
// Income and property can use different fractions; Share owns that distinction.
pub(crate) fn professional_group_member(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
    owner: Owner,
) -> bool {
    d.professional_group_owner(entry.owner.owned()) == d.professional_group_owner(owner)
        && !d
            .setup
            .as_ref()
            .and_then(|setup| {
                crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
            })
            .is_some_and(|share| share.none())
}

// Pure source projections use the same depreciation, mortgage, basis and
// at-risk owners as the emitting pass. The elected group is transient: each
// accepted rental/K-1 history remains attached to its own source identity.
// Reg. 1.469-9(e)(1), (4) applies the former-passive limit to this combined
// activity, including current income/loss on property forms.
pub(crate) fn professional_former_passive(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    amt: bool,
    home_use: &HomeUseContext,
) -> Result<ProfessionalGroups, crate::activity_risk::SourceIssue> {
    let d = form_context.draft;
    let mut result = ProfessionalGroups::default();
    if professional_election_rule(d).is_some() {
        return Ok(result);
    }
    let itemizing = home_use.itemizes.unwrap_or(true);
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref()));
    let allocated_entries = crate::community::entries(d);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        if d.professional_group_owner(owner) != owner {
            continue;
        }
        if !d.professional(owner).is_some_and(|status| status.single_activity) {
            continue;
        }
        let disposed = professional_group_disposed(d, owner);
        let mut rental_issue = None;
        let sources = (|| {
            let mut sources = alloc::vec::Vec::new();
            for entry in
                allocated_entries.iter().filter(|entry| professional_group_member(d, entry, owner))
            {
                let projected = match &entry.income {
                    FederalIncome::Rental { data }
                        if data.participation.professional()
                            && data.property_type.business_participation().is_none()
                            && !used_as_home(data) =>
                    {
                        let data = data.clone();
                        let whole_entry = d.income.iter().find(|source| source.id == entry.id)?;
                        let property_entry = crate::community::property_entry(d, whole_entry);
                        let FederalIncome::Rental { data: property_rental } =
                            &property_entry.income
                        else {
                            unreachable!("rental source")
                        };
                        let usage = match rental_use_core(
                            &data,
                            amt,
                            mid_quarter,
                            itemizing,
                            home_fixed_allowed(form_context, entry.id, home_use.agi, amt),
                            &rental_expense_rows(form_context, whole_entry, false),
                            Some((d, entry.id, false, Some(home_use))),
                            Some(form_context.macrs().as_ref()),
                        ) {
                            Ok(value) => value,
                            Err((key, _)) => {
                                rental_issue = Some(crate::activity_risk::SourceIssue::new(
                                    entry.id,
                                    crate::activity_risk::ProjectionIssue::Source(key),
                                ));
                                return None;
                            }
                        };
                        let projected = crate::asset_sale::rental_sale_amount(
                            form_context,
                            entry.id,
                            property_rental,
                            mid_quarter,
                            amt,
                            Some(home_use),
                        )
                        .and_then(|sale| rental_risk(&data, usage.net, sale, amt))
                        .and_then(|risk| risk.ordinary_income());
                        let ordinary = match projected {
                            Ok(value) => value,
                            Err(issue) => {
                                rental_issue =
                                    Some(crate::activity_risk::SourceIssue::new(entry.id, issue));
                                return None;
                            }
                        };
                        let prior = data.history.balance(amt);
                        let sale = match crate::asset_sale::rental_source(
                            form_context,
                            entry.id,
                            amt,
                            Some(home_use),
                        ) {
                            Ok(value) => value,
                            Err(issue) => {
                                rental_issue = Some(issue);
                                return None;
                            }
                        };
                        Some((
                            [
                                (dollars(ordinary) + sale.as_ref().map_or(0, |sale| sale.ordinary))
                                    * 100,
                                sale.as_ref().map_or(0, |sale| sale.section1231) * 100,
                            ],
                            [dollars(prior) * 100, 0],
                        ))
                    }
                    FederalIncome::Scorp { data }
                        if crate::ScorpK1::from(data)
                            .rental_facts
                            .participation
                            .is_some_and(crate::K1RentalParticipation::professional) =>
                    {
                        let data = &crate::ScorpK1::from(data);
                        let source: crate::scorp_amt::ScorpSource<'_> =
                            if amt { data.amt.as_ref()?.source(data).ok()? } else { data.into() };
                        let activity =
                            crate::community::scorp_pool(d, entry.id, &source, amt).ok()?;
                        source
                            .rental_activities(&activity.ledger, true, disposed)
                            .ok()?
                            .iter()
                            .find(|rental| rental.real_estate)
                            .map(|rental| rental.current_and_prior())
                    }
                    FederalIncome::Partnership { data }
                        if data
                            .rental_facts
                            .participation
                            .is_some_and(crate::K1RentalParticipation::professional) =>
                    {
                        let source = if amt { data.alternative.as_ref()? } else { &data.regular };
                        let activity =
                            crate::community::partner_pool(d, entry.id, data, source).ok()?;
                        data.rental_activities(source, &activity.activity.ledger, true, disposed)
                            .ok()?
                            .iter()
                            .find(|rental| rental.real_estate)
                            .map(|rental| rental.current_and_prior())
                    }
                    _ => None,
                };
                if let Some(projected) = projected {
                    sources.push((entry.id, projected));
                }
            }
            // Stable source identities, rather than labels or interview order,
            // determine the shared allocator's source-cent remainder order.
            sources.sort_by_key(|(id, _)| *id);
            Some(sources)
        })();
        if let Some(issue) = rental_issue {
            return Err(issue);
        }
        let Some(sources) = sources else { continue };
        let amounts: alloc::vec::Vec<_> = sources.iter().map(|(_, amounts)| *amounts).collect();
        let Ok(parts) =
            crate::scorp::ActivityCharacters::grouped_former_passive(&amounts, disposed)
        else {
            continue;
        };
        let Ok(former) = crate::scorp::ActivityCharacters::grouped_former_passive(&amounts, false)
        else {
            continue;
        };
        let net = parts.iter().flat_map(|parts| parts.nonpassive).sum::<i64>();
        result.magi += dollars(if disposed { net } else { positive(net) });
        result.sources.extend(sources.iter().zip(parts).zip(former).map(
            |(((id, _), parts), former)| ProfessionalSource {
                id: *id,
                parts,
                income_release: former.prior_released,
            },
        ));
    }
    Ok(result)
}

pub(crate) fn professional_group_disposed(d: &FederalDraft, owner: Owner) -> bool {
    let owner = d.professional_group_owner(owner);
    if !d.professional(owner).is_some_and(|p| p.single_activity) {
        return true;
    }
    d.income.iter().filter(|entry| professional_group_member(d, entry, owner)).all(|entry| {
        match &entry.income {
            FederalIncome::Rental { data }
                if data.property_type.business_participation().is_none() && !used_as_home(data) =>
            {
                match &data.sale {
                    crate::RentalSale::Sold { buyer, payment, .. } => {
                        *buyer == crate::Buyer::Unrelated && !payment.installment()
                    }
                    crate::RentalSale::Condemned { replacement_cost, .. } => *replacement_cost == 0,
                    crate::RentalSale::Exchanged(exchange) => exchange.releases_losses(),
                    crate::RentalSale::No => false,
                }
            }
            FederalIncome::Partnership { data } if data.regular.rentals.real_estate_present() => {
                data.disposition.as_ref().is_some_and(crate::K1Disposition::ended)
                    || data.rental_facts.disposed.contains(&crate::K1RentalActivity::RealEstate)
            }
            FederalIncome::Scorp { data }
                if crate::ScorpK1::from(data).rentals.real_estate_present() =>
            {
                let data = crate::ScorpK1::from(data);
                data.disposition.as_ref().is_some_and(crate::K1Disposition::ended)
                    || data.rental_facts.disposed.contains(&crate::K1RentalActivity::RealEstate)
            }
            _ => true,
        }
    })
}

pub(crate) fn professional_election_rule(d: &FederalDraft) -> Option<&'static str> {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        if !d.professional(owner).is_some_and(|p| p.single_activity) {
            continue;
        }
        let mut answers = alloc::vec::Vec::new();
        for entry in d.income.iter().filter(|e| professional_group_member(d, e, owner)) {
            match &entry.income {
                // A section 280A home year is outside section 469 (section
                // 469(j)(10)), so the election does not reach it.
                FederalIncome::Rental { data }
                    if data.property_type.business_participation().is_none()
                        && !used_as_home(data) =>
                {
                    answers.push(data.participation.professional())
                }
                FederalIncome::Partnership { data }
                    if data.regular.rentals.real_estate_present() =>
                {
                    answers.push(
                        data.rental_facts
                            .participation
                            .is_some_and(crate::K1RentalParticipation::professional),
                    )
                }
                FederalIncome::Scorp { data }
                    if crate::ScorpK1::from(data).rentals.real_estate_present() =>
                {
                    answers.push(
                        crate::ScorpK1::from(data)
                            .rental_facts
                            .participation
                            .is_some_and(crate::K1RentalParticipation::professional),
                    )
                }
                _ => {}
            }
        }
        if answers.iter().any(|v| *v) && answers.iter().any(|v| !*v) {
            return Some("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS");
        }
    }
    None
}

pub(crate) fn rental_expense_lines(
    v: &FederalRental,
    usage: &RentalUse,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
) -> Result<[i64; RentalExpenseCategory::COUNT], (&'static str, &'static str)> {
    let mut rows = RentalExpense::category_amounts(&usage.expense_rows);
    rows[14] += exact::whole(method_amounts(v).1) + exact::whole(usage.prior_at_risk_cents);
    let project = |rows: [Exact; RentalExpenseCategory::COUNT]| {
        let mut native = [0; RentalExpenseCategory::COUNT];
        for (output, amount) in native.iter_mut().zip(rows) {
            *output = exact::rounded(&amount).ok_or_else(rental_expense_amount_issue)?;
        }
        Ok(native)
    };
    let RentalPersonalUse::Yes { fair_rental_days, personal_days, direct_expenses, .. } =
        v.personal_use
    else {
        rows[14] += exact::whole(v.home_operating_carryover) - exact::whole(usage.carry[0]);
        return project(rows);
    };
    let days = i64::from(fair_rental_days + personal_days);
    if days > 0 {
        for amount in &mut rows {
            *amount = exact::rounded_exact(
                &(&*amount * exact::ratio(i64::from(fair_rental_days), days).expect("rental days")),
            );
        }
    }
    if !usage.home {
        rows[14] += exact::whole(direct_expenses) + exact::whole(v.home_operating_carryover)
            - exact::whole(usage.carry[0]);
        return project(rows);
    }
    let fixed = if itemizing {
        fixed_allowed.unwrap_or_else(|| [rows[7].clone(), rows[12].clone(), exact::whole(0)])
    } else {
        core::array::from_fn(|_| exact::whole(0))
    };
    rows[4] -= &fixed[2];
    rows[7] -= &fixed[0];
    rows[12] -= &fixed[1];
    rows[14] += exact::whole(v.home_operating_carryover);
    let mut room = phase::floor(
        exact::whole(v.rent)
            - fixed.iter().cloned().sum::<Exact>()
            - exact::whole(direct_expenses)
            - exact::whole(usage.otherwise_casualty_cents),
        exact::whole(0),
    );
    for amount in &mut rows {
        *amount = phase::cap(amount.clone(), [room.clone()]);
        room -= &*amount;
    }
    rows[4] += &fixed[2];
    rows[7] += &fixed[0];
    rows[12] += &fixed[1];
    rows[14] += exact::whole(direct_expenses);
    project(rows)
}
