use super::*;

// A building's depreciation for one calendar year; `sold` limits the year of
// sale to the half-months before the middle of the month sold (mid-month
// convention, Pub 946 ch. 4). Earlier years reproduce the amounts allowed or
// allowable for the Form 4797 line 24 total.
pub(crate) fn rental_building_source_rule(
    b: &RentalBuilding,
    location: &crate::RentalLocation,
) -> Result<(), (&'static str, &'static str)> {
    if b.casualty.as_ref().is_some_and(|source| source.issue(b.cost.saturating_sub(b.land)).is_some())
        || b.land < 0
        || b.cost < b.land
        || b.cost > 100_000_000_000
        || b.sale_history.as_ref().is_some_and(|history| !history.valid(b.cost - b.land))
        || !(0..=b.cost - b.land).contains(&b.credit_reduction)
        || matches!(b.acquired, crate::RentalAcquisition::Converted { fair_market_value } if !(0..=100_000_000_000).contains(&fair_market_value))
        || !valid_service_date(b.in_service)
        || (b.in_service < 19870101 && !location.foreign())
        // A part replaced or removed in 2026 leaves a building placed in
        // service before 2026 (Reg. 1.168(i)-8(d)): its cost is a part of the
        // building's, and it follows the date placed in service.
        || b.partial.as_ref().is_some_and(|part| {
            !(20260101..=20261231).contains(&part.date)
                || !crate::federal_rules::valid_date(part.date)
                || part.date <= b.in_service
                || b.in_service / 10000 >= 2026
                || !(1..b.cost - b.land).contains(&part.cost_of_part)
                || !(0..=100_000_000_000).contains(&part.proceeds)
        })
    {
        return Err((
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
            crate::fixture_english!(
                "Review building cost, land allocation and service date; this path uses post-1986 residential or nonresidential GDS."
            ),
        ));
    }
    Ok(())
}

pub(crate) fn depreciation_for_year(
    b: &RentalBuilding,
    amt: bool,
    property: (&RentalPropertyType, &crate::RentalLocation),
    year: u32,
    sold: Option<u32>,
    energy_basis_reduction: crate::kernel::source_fact::SourceFact<i64>,
) -> Result<i64, (&'static str, &'static str)> {
    let (property_type, location) = property;
    rental_building_source_rule(b, location)?;
    let unavailable = (
        crate::rental_energy::BASIS,
        "The energy property needs its actual prepared section 179D basis adjustment.",
    );
    let energy_basis_reduction = match energy_basis_reduction {
        crate::kernel::source_fact::SourceFact::Reported(value)
            if (0..=b.cost - b.land - b.rehabilitation_reduction()).contains(&value) =>
        {
            value
        }
        crate::kernel::source_fact::SourceFact::NotReported if b.energy.is_none() => 0,
        _ => return Err(unavailable),
    };
    // Section 179D(e) has its own basis input; it is never a credit or
    // depreciation recapture source. Original acquisition cost remains UBIA
    // under Treas.Reg.1.199A-2(c)(3)(i).
    // Section 50(c): a rehabilitation credit reduces the depreciable basis by
    // the whole credit determined.
    let basis = match b.acquired {
        crate::RentalAcquisition::Converted { fair_market_value } => {
            phase::cap(b.cost - b.land, [fair_market_value])
        }
        _ => b.cost - b.land,
    } - b.rehabilitation_reduction()
        - energy_basis_reduction;
    let reduction =
        crate::casualty::rental::continuing_reduction(b, amt, property_type, year, basis).map_err(
            |key| (key, "Reconcile the property's actual casualty basis and depreciation records."),
        )?;
    // Half-months of the year of sale before the mid-month disposition.
    let window = sold.map(|date| 2 * (date / 100 % 100).clamp(1, 12) as i32 - 1);
    // A part of the building replaced or removed in 2026 takes its cost out of
    // the depreciable basis from the month of the disposition (Reg.
    // 1.168(i)-8(f)(3)): the year is the mid-month share before the part
    // leaves and the building's share of the cost after it.
    let part = b.partial.as_ref().filter(|_| year == 2026);
    let basis_after =
        part.map(|p| fraction(basis, b.cost - b.land - p.cost_of_part, b.cost - b.land));
    type AmountResult = Result<i64, (&'static str, &'static str)>;
    let split = |current: i32, amount: &dyn Fn(i64, i32) -> AmountResult| -> AmountResult {
        match (part, basis_after) {
            (Some(p), Some(after)) => {
                let w = (2 * (p.date / 100 % 100).clamp(1, 12) as i32 - 1).min(current);
                Ok(amount(basis, w)? + amount(after, current - w)?)
            }
            _ => amount(basis, current),
        }
    };
    // The shared historical owner selects the original GDS/AMT clock and
    // computes its exact amount. Keep this rental owner's existing rounding
    // and partial-disposition allocation boundaries.
    let recovery = crate::historical_recovery::rental_building_recovery(
        property_type,
        location,
        b.in_service,
        b.commercial_1993_transition.as_ref(),
        amt,
    )?;
    let current = if reduction.is_some() {
        // The dated owner caps the actual calendar window to the original
        // remaining life, including an already fully recovered zero basis.
        24
    } else {
        crate::historical_recovery::rental_recovery_window(recovery, b.in_service, year) as i32
    };
    split(window.map_or(current, |w| current.min(w)), &|base, hm| {
        let amount = |window| {
            crate::historical_recovery::rental_building_amount(
                recovery,
                base,
                b.in_service,
                year,
                window,
                reduction,
            )
            .map(|amount| amount.rounded_cents())
            .map_err(|_| {
                (
                    "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
                    "Review the original building recovery records.",
                )
            })
        };
        if reduction.is_none()
            && recovery == crate::historical_recovery::RentalBuildingRecovery::Commercial39
        {
            // Preserve the rental form's existing annual-table rounding
            // before applying its disposition/component fraction.
            Ok(fraction(amount(24)?, i64::from(hm), 24))
        } else {
            amount(hm as u32)
        }
    })
}

// Section 469(g)(3): the part of an overall loss that an installment sale of the
// entire interest makes nonpassive this year, and the part that stays
// suspended. The ratio is the gain recognized this year (the installment
// income and the recapture, which is taxed in full) to the total gain (the
// gross profit and the recapture). Whole dollars.
pub(crate) fn installment_release(
    overall: i64,
    form: &crate::kernel::installment::Schedule,
) -> (i64, i64) {
    let total = form.gross_profit + form.recapture;
    let recognized = phase::bound(form.income + form.recapture, 0, total);
    let released = if total > 0 {
        phase::bound(fraction(overall, recognized, total), 0, overall)
    } else {
        overall
    };
    (released, overall - released)
}

pub(crate) fn rental_net(v: &FederalRental) -> Result<i64, (&'static str, &'static str)> {
    if let Some(key) = v.payroll.rule() {
        return Err((
            key,
            crate::fixture_english!(
                "Reconcile documented qualified wages with the original payroll statements."
            ),
        ));
    }
    rental_net_for(v, false)
}

// Same de minimis safe-harbor election check as the shared Schedule C
// `expense_elections`, applied to `RentalExpense` instead of
// `FederalBusinessExpense`: the two record shapes differ only in their
// category enum, and this check reads only `amount` and `treatment`.
pub(crate) fn rental_expense_elections(
    r: &mut FederalResult,
    expenses: &[crate::extended_model::RentalExpense],
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
    prefix: &str,
) {
    for (index, expense) in expenses.iter().enumerate() {
        if let crate::BusinessExpenseKind::DeMinimis {
            largest_cost,
            financial_statement,
            eligible,
        } = expense.treatment
        {
            let ceiling = if financial_statement { 500_000 } else { 250_000 };
            // An ineligible expense is left out before the rules run.
            crate::require_localized!(
                r,
                eligible && largest_cost <= ceiling,
                context,
                source,
                "ISSUE_DE_MINIMIS_ELECTION_REQUIRES_CONFIRMED_ELIGIBLE_BOOK_EXPENSING_THE_20E65A9C",
                "De minimis election requires confirmed eligible book expensing, the annual election for all qualifying purchases, and invoice/item costs at most $2,500 without an applicable financial statement or $5,000 with one. Reclassify ineligible purchases under their ordinary rules; an over-limit item is not a partially deductible safe-harbor purchase.",
            );
            crate::require_localized!(
                r,
                expenses.iter().all(|other| match other.treatment {
                    crate::BusinessExpenseKind::DeMinimis {
                        financial_statement: other_statement,
                        ..
                    } => other_statement == financial_statement,
                    _ => true,
                }),
                context,
                source,
                "ISSUE_USE_CONSISTENT_APPLICABLE_FINANCIAL_STATEMENT_FACTS_FOR_THIS_ACTIVI_5F5D61F3",
                "Use consistent applicable-financial-statement facts for this activity's de minimis purchases.",
            );
            line_cents(
                r,
                &alloc::format!("{prefix}_expense_{index}_de_minimis"),
                expense.amount,
                &["income.expenses"],
                "https://www.irs.gov/businesses/small-businesses-self-employed/tangible-property-final-regulations",
                msg!("EXPLAIN_N_EXPENSE_N_DE_MINIMIS"),
            );
        }
    }
}

pub(crate) fn rental_net_for(
    v: &FederalRental,
    amt: bool,
) -> Result<i64, (&'static str, &'static str)> {
    rental_use_for(v, amt).map(|u| u.net)
}

// The 2026 section 481(a) increase and decrease of a rental, in cents. A
// rental with personal use keeps no record (unknowns::supported).
pub(crate) fn method_amounts(v: &FederalRental) -> (i64, i64) {
    (
        crate::method_change::increase(&v.method_changes),
        crate::method_change::decrease(&v.method_changes),
    )
}

// Worksheet 5-1 lines 7a and 7b: the operating expenses and depreciation that
// carry to the next year.
pub(crate) fn carryover_rows(r: &mut FederalResult, prefix: &str, id: i64, carry: [i64; 2]) {
    emit_lines(
        r,
        &mut (),
        [
            (
                "home_operating_carryover",
                carry[0],
                msg!("EXPLAIN_0RENTAL_N_N_HOME_OPERATING_CARRYOVER"),
            ),
            (
                "home_depreciation_carryover",
                carry[1],
                msg!("EXPLAIN_0RENTAL_N_N_HOME_DEPRECIATION_CARRYOVER"),
            ),
        ],
        |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}rental_{id}_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.rental.personal_use"],
                RENTAL,
                explanation,
            ))
        },
    );
}

pub(crate) fn rental_use_for(
    v: &FederalRental,
    amt: bool,
) -> Result<RentalUse, (&'static str, &'static str)> {
    rental_use_with(v, amt, false, true, None)
}

pub(crate) fn rental_use_with(
    v: &FederalRental,
    amt: bool,
    mid_quarter: bool,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
) -> Result<RentalUse, (&'static str, &'static str)> {
    rental_use_core(
        v,
        amt,
        mid_quarter,
        itemizing,
        fixed_allowed,
        &RentalExpense::exact_rows(&v.expenses),
        None,
        None,
    )
}

pub(crate) fn rental_source_valid(v: &FederalRental) -> bool {
    v.payroll.rule().is_none() && rental_source_rule(v, None).is_ok()
}

pub(crate) fn rental_source_rule(
    v: &FederalRental,
    source: Option<&crate::nonmacrs::RentalRecoverySource<'_>>,
) -> Result<(), (&'static str, &'static str)> {
    if let Some(issue) = crate::rental_foreign::location_issue(&v.location) {
        return Err((issue.key, "Review the rental property's actual country."));
    }
    if v.property_type.business_participation().is_some_and(|material| matches!(v.qbi_business, RentalBusiness::Yes { material_participation } if material_participation != material)) {
        return Err(("SHORT_TERM_PARTICIPATION_AGREES", crate::fixture_english!("Use the same material-participation answer for the short-stay business and its QBI treatment.")));
    }
    let invalid_input = (
        "ISSUE_RENTAL_BUILDINGS_OR_EXPENSES_OUT_OF_LIMITS",
        crate::fixture_english!(
            "Enter the rental's building records and ordinary expenses within the supported limits."
        ),
    );
    if crate::rental_energy::source_issue(v).is_some() || !v.payroll.projected_shape_valid()
        || crate::activity_risk::source_issue(&v.at_risk, &v.history).is_some()
        || v.buildings.is_empty()
        || v.buildings.len() > 100
        || v.assets.len() > 100
        || v.assets.iter().enumerate().any(|(index, a)| {
            let a = source.map_or(a, |source| source.asset(index, a));
            !a.valid() || a.available179() > 0
        })
        || !crate::method_change::valid(&v.method_changes)
        || v.rent < 0
        || v.rent > 100_000_000_000
        || v.expenses
            .iter()
            .any(|e| e.amount < 0 || matches!(e.treatment, crate::BusinessExpenseKind::DeMinimis { largest_cost, .. } if !(1..=100_000_000_000).contains(&largest_cost)))
        || match v.personal_use {
            crate::extended_model::RentalPersonalUse::No => false,
            crate::extended_model::RentalPersonalUse::Yes {
                fair_rental_days,
                personal_days,
                overlap_days,
                direct_expenses,
            } => fair_rental_days + personal_days > 365 || overlap_days > fair_rental_days || !(0..=100_000_000_000).contains(&direct_expenses),
        }
        || !(0..=100_000_000_000).contains(&v.home_operating_carryover)
        || crate::rental_home_carry::issue(v).is_some()
    {
        return Err(invalid_input);
    }
    let risk = v.at_risk.as_ref().ok_or(invalid_input)?;
    if crate::rental_property_history::history_issue(
        &v.property_qbi_history,
        risk.prior_sale(false),
    )
    .is_some()
    {
        return Err(invalid_input);
    }
    for building in &v.buildings {
        rental_building_source_rule(building, &v.location)?;
        if !v.location.foreign()
            && let Some(issue) = crate::rental_recovery::source_rule(
                &v.property_type,
                building.in_service,
                building.commercial_1993_transition.as_ref(),
            )
        {
            return Err((issue.key, "Review the actual 1993 commercial recovery source."));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rental_use_core(
    v: &FederalRental,
    amt: bool,
    mid_quarter: bool,
    itemizing: bool,
    fixed_allowed: Option<[Exact; 3]>,
    expense_rows: &[crate::extended_model::RentalExpenseAmount],
    energy_context: Option<(&FederalDraft, i64, bool, Option<&HomeUseContext>)>,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<RentalUse, (&'static str, &'static str)> {
    let home_use = energy_context.and_then(|(_, _, _, home_use)| home_use);
    let recovery_source = energy_context.and_then(|(draft, id, _, _)| {
        crate::nonmacrs::rental_recovery_source_with_macrs(draft, id, false, macrs)
    });
    let source = recovery_source.as_ref();
    if let Some((_, issue)) = crate::interest_limit::rental_building_issue(v) {
        return Err((issue.key, "Retain the actual building recovery and original ADS history."));
    }
    rental_source_rule(v, source)?;
    let asset_amounts = crate::nonmacrs::rental_asset_amounts(&v.assets, source, mid_quarter, amt)
        .map_err(|key| (key, "Retain the original property schedule and actual source records."))?;
    crate::rental_foreign::source_admission(v, source)?;
    let unavailable = rental_expense_amount_issue();
    let energy = if let Some((draft, source, whole, _)) = energy_context {
        crate::rental_energy::for_source(draft, source, whole)
    } else {
        if v.buildings.iter().any(|cost| cost.energy.is_some()) {
            return Err((
                crate::rental_energy::YEAR,
                "The section 179D claim needs the bound return year.",
            ));
        }
        crate::rental_energy::Prepared {
            reductions: alloc::vec![0; v.buildings.len()],
            boundaries: Vec::new(),
        }
    };
    if let Some(key) = energy.boundaries.first() {
        return Err((
            *key,
            "The energy property's source-dependent deduction and basis are unavailable.",
        ));
    }
    let depreciation_year =
        u32::try_from(energy_context.map_or(2026, |(draft, _, _, _)| draft.tax_year))
            .map_err(|_| (crate::rental_energy::YEAR, "The bound return year is unavailable."))?;
    let mut expense_rows = expense_rows.to_vec();
    let energy_total =
        crate::kernel::ratio::checked_sum(energy.reductions.iter().copied()).ok_or(unavailable)?;
    if energy_total > 0 {
        expense_rows.push(crate::extended_model::RentalExpenseAmount {
            category: RentalExpenseCategory::Other { label: "Section 179D".into() },
            amount: exact::whole(energy_total),
            treatment: BusinessExpenseKind::Ordinary,
        });
    }
    let expense_rows = expense_rows.as_slice();
    let prior = v.at_risk.as_ref().expect("validated source").prior(amt);
    let current_expenses = RentalExpense::total_amounts(expense_rows);
    let expenses = &current_expenses + exact::whole(prior);
    let mut depreciation_total = asset_amounts.values().cloned().sum::<Exact>();
    let mut asset_recapture = exact::whole(0);
    for (index, projected) in v.assets.iter().enumerate() {
        let asset = source.map_or(projected, |source| source.asset(index, projected));
        let alternative;
        let source_asset = if amt {
            alternative = asset.amt_projection_or_self();
            &alternative
        } else {
            asset
        };
        if let Some(key) = source.and_then(|source| source.history_issue(source_asset)) {
            return Err((
                key,
                "The property's actual recovery history is needed for disposition or recapture.",
            ));
        }
        if let Some(recapture) =
            source_asset.recapture().or_else(|| source_asset.section179_recapture())
        {
            let amount = exact::whole(recapture.excess());
            asset_recapture += if asset.nonmacrs() {
                source.map_or_else(|| amount.clone(), |source| source.project(&amount))
            } else {
                amount
            };
        }
    }
    for (index, building) in v.buildings.iter().enumerate() {
        if building.energy.is_some() && energy.boundaries.contains(&crate::rental_energy::BASIS) {
            return Err((
                crate::rental_energy::BASIS,
                "This energy property needs its actual section 179D basis source.",
            ));
        }
        let energy_basis =
            crate::kernel::source_fact::SourceFact::Reported(energy.reductions[index]);
        let depreciation = if building.casualty.is_some() {
            crate::casualty::rental::current_depreciation(
                building, v, amt, depreciation_year, energy_basis,
            ).map_err(|key| (key, "Resolve this casualty's actual basis and continuing-depreciation records before estimating."))?
        } else {
            depreciation_for_year(
                building,
                amt,
                (&v.property_type, &v.location),
                depreciation_year,
                v.sale.date(),
                energy_basis,
            )?
        };
        depreciation_total += exact::whole(depreciation);
    }
    if let Some(exchange) = v.sale.exchange() {
        depreciation_total += exact::whole(exchange.replacement_depreciation(v, amt)?.total());
    }
    let prior_operating = exact::whole(v.home_operating_carryover);
    let prior_total = crate::rental_home_carry::total(v, amt).ok_or((
        crate::rental_home_carry::TOTAL,
        "Supply the actual prior home-use carry for this tax system.",
    ))?;
    let prior_depreciation = exact::whole(prior_total);
    // Project only the completed Worksheet 5-1/native operating outputs,
    // after wide expenses have passed through all actual limits and offsets.
    let finish = |net: Exact,
                  current_operating_for_risk: Option<Exact>,
                  depreciation: Exact,
                  home: bool,
                  excluded: bool,
                  carry: [Exact; 2],
                  personal_fixed: Exact| {
        Ok(RentalUse {
            energy_deduction: energy_total,
            expense_rows: expense_rows.to_vec(),
            // The recapture owner already supplies the business-only amount.
            // Add after rental/home prorating, before the shared risk/PAL stage;
            // it neither receives another personal share nor becomes home rent.
            net: exact::rounded(&(net + &asset_recapture)).ok_or(unavailable)?,
            current_operating_for_risk: current_operating_for_risk
                .map(|net| net + &asset_recapture),
            asset_recapture_cents: exact::rounded(&asset_recapture).ok_or(unavailable)?,
            prior_at_risk_cents: prior,
            depreciation: exact::rounded(&depreciation).ok_or(unavailable)?,
            home_third_tier: None,
            current_casualty_losses: Vec::new(),
            otherwise_casualty_cents: 0,
            home,
            excluded,
            carry: [
                exact::rounded(&carry[0]).ok_or(unavailable)?,
                exact::rounded(&carry[1]).ok_or(unavailable)?,
            ],
            personal_fixed,
        })
    };
    let crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days,
        personal_days,
        overlap_days,
        direct_expenses,
    } = v.personal_use
    else {
        let (increase, decrease) = method_amounts(v);
        let income = exact::whole(v.rent) + exact::whole(increase);
        let current = income - &current_expenses - &depreciation_total - exact::whole(decrease);
        let net = &current - exact::whole(prior);
        // Pub527, "Renting Part of Property" and "Carryover of Unallowed
        // Expenses": prior home-use expenses remain limited to rental income
        // after current deductions, including the separately reported casualty.
        // Keep that casualty out of Schedule E; its property activity owns it.
        let casualty = if v.home_operating_carryover > 0 || prior_total > 0 {
            let losses = crate::casualty::rental::property_losses(v, amt)
                .map_err(|key| (key, "Review this rental's current casualty source."))?;
            crate::kernel::ratio::checked_sum(losses.into_iter().map(|(_, loss)| loss))
                .ok_or(unavailable)?
        } else {
            0
        };
        let room = phase::floor(&net - exact::whole(casualty), exact::whole(0));
        let operating_carried = phase::cap(room.clone(), [prior_operating.clone()]);
        let tiers = crate::rental_home_carry::tiers(v, amt, 0, 0, 0, &(&room - &operating_carried))
            .map_err(|key| (key, "Review the prior home-use carry sources."))?;
        let depreciation_carried = exact::whole(tiers.total_allowed_cents);
        let mut result = finish(
            net - &operating_carried - &depreciation_carried,
            Some(current - &operating_carried - &depreciation_carried),
            depreciation_total + &depreciation_carried,
            false,
            false,
            [prior_operating - operating_carried, prior_depreciation - depreciation_carried],
            exact::whole(0),
        )?;
        result.depreciation =
            tiers.allowed_depreciation_cents().map_or(result.depreciation, |allowed| {
                result.depreciation - tiers.total_allowed_cents + allowed
            });
        result.home_third_tier = Some(tiers);
        return Ok(result);
    };
    let (fair, personal) = (i64::from(fair_rental_days), i64::from(personal_days));
    let rental_part = |amount: &Exact| {
        if personal == 0 {
            amount.clone()
        } else {
            exact::rounded_exact(
                &(amount * exact::ratio(fair, fair + personal).expect("rental days")),
            )
        }
    };
    let personal_use_days = personal + i64::from(overlap_days);
    let fair_days_for_test = fair - i64::from(overlap_days);
    let home = personal_use_days > 14 && personal_use_days * 10 > fair_days_for_test;
    let fixed_expenses: alloc::vec::Vec<_> = expense_rows
        .iter()
        .filter(|expense| {
            matches!(
                expense.category,
                RentalExpenseCategory::MortgageInterest | RentalExpenseCategory::TaxesAndLicenses
            )
        })
        .cloned()
        .collect();
    let fixed_total = RentalExpense::total_amounts(&fixed_expenses);
    let personal_fixed = &fixed_total - rental_part(&fixed_total);
    if home && fair < 15 {
        if v.buildings
            .iter()
            .filter_map(|building| building.casualty.as_ref())
            .any(|source| !crate::casualty::rental::neutral_event(&source.casualty))
        {
            return Err((
                "RENTAL_CASUALTY_HOME_ALLOWANCE_UNAVAILABLE",
                "A home rented fewer than15days needs its actual personal casualty worksheet.",
            ));
        }
        return finish(
            exact::whole(0),
            None,
            exact::whole(0),
            home,
            true,
            [prior_operating, prior_depreciation],
            fixed_total,
        );
    }
    let fixed = rental_part(&fixed_total) + exact::whole(direct_expenses);
    if !home {
        let operating = rental_part(&(expenses - &fixed_total));
        let depreciation = rental_part(&depreciation_total);
        let net = exact::whole(v.rent) - fixed - operating - &depreciation;
        let room = phase::floor(net.clone(), exact::whole(0));
        let operating_carried = phase::cap(room.clone(), [prior_operating.clone()]);
        let tiers = crate::rental_home_carry::tiers(v, amt, 0, 0, 0, &(&room - &operating_carried))
            .map_err(|key| (key, "Review the prior home-use carry sources."))?;
        let depreciation_carried = exact::whole(tiers.total_allowed_cents);
        let mut result = finish(
            net - &operating_carried - &depreciation_carried,
            None,
            depreciation + &depreciation_carried,
            home,
            false,
            [prior_operating - operating_carried, prior_depreciation - depreciation_carried],
            personal_fixed,
        )?;
        result.depreciation =
            tiers.allowed_depreciation_cents().map_or(result.depreciation, |allowed| {
                result.depreciation - tiers.total_allowed_cents + allowed
            });
        result.home_third_tier = Some(tiers);
        return Ok(result);
    }
    let casualty_losses = crate::casualty::rental::property_losses(v, amt)
        .map_err(|key| (key, "Review this rental's current casualty source."))?;
    let current_casualty =
        crate::kernel::ratio::checked_sum(casualty_losses.iter().map(|(_, loss)| *loss))
            .ok_or(unavailable)?;
    let itemizing = home_use.and_then(|context| context.itemizes).unwrap_or(itemizing);
    let otherwise_by_cost = if casualty_losses.is_empty() {
        Vec::new()
    } else {
        let (draft, id, _, _) = energy_context.ok_or((
            "RENTAL_CASUALTY_DEDUCTION_CONTEXT_UNAVAILABLE",
            "This home casualty needs the return's actual deduction choice.",
        ))?;
        let route = home_use
            .and_then(|context| context.itemizes)
            .or_else(|| HomeUseContext::route(draft, None));
        // An ordinary loss cannot increase the standard deduction. Preserve
        // that established zero without requiring an unused source association.
        let ordinary_standard = route == Some(false)
            && v.buildings.iter().filter_map(|cost| cost.casualty.as_ref()).all(|records| {
                !matches!(
                    records.casualty.declaration,
                    crate::CasualtyDeclaration::MajorDisaster { .. }
                )
            })
            && draft.deductions.as_ref().is_none_or(|ded| {
                ded.personal_casualties.iter().all(|event| !event.qualified() && event.gain() == 0)
            });
        if ordinary_standard {
            alloc::vec![0; casualty_losses.len()]
        } else {
            let context = home_use.ok_or((
                "RENTAL_CASUALTY_DEDUCTION_CONTEXT_UNAVAILABLE",
                "This home casualty needs its bound deduction choice and AGI.",
            ))?;
            let itemizes = route.ok_or((
                "RENTAL_CASUALTY_DEDUCTION_CONTEXT_UNAVAILABLE",
                "Compare the final deduction routes before settling this home casualty.",
            ))?;
            let agi = context.dwelling_agi_cents(id).ok_or((
                "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE",
                "The dwelling's actual income contribution is unavailable.",
            ))?;
            crate::casualty::rental_home::allowances(
                draft,
                id,
                v,
                &casualty_losses,
                amt,
                itemizes,
                agi,
            )
            .map_err(|key| {
                (key, "Resolve the saved whole-dwelling casualty source before estimating.")
            })?
        }
    };
    let otherwise =
        crate::kernel::ratio::checked_sum(otherwise_by_cost.iter().copied()).ok_or(unavailable)?;
    let eligible = if itemizing {
        fixed_allowed
            .as_ref()
            .map_or_else(|| rental_part(&fixed_total), |allowed| allowed.iter().cloned().sum())
    } else {
        exact::whole(0)
    };
    let fixed = &eligible + exact::whole(direct_expenses) + exact::whole(otherwise);
    let operating = rental_part(&expenses) - eligible + prior_operating;
    let depreciation = rental_part(&depreciation_total);
    let line3 = phase::floor(exact::whole(v.rent) - &fixed, exact::whole(0));
    let operating_allowed = phase::cap(line3.clone(), [operating.clone()]);
    let line5 = line3 - &operating_allowed;
    let current_depreciation = exact::rounded(&depreciation).ok_or(unavailable)?;
    let tiers = crate::rental_home_carry::tiers(
        v,
        amt,
        otherwise,
        current_casualty - otherwise,
        current_depreciation,
        &line5,
    )
    .map_err(|key| (key, "Review the prior home-use carry sources."))?;
    let depreciation_allowed = exact::whole(tiers.total_allowed_cents);
    let mut result = finish(
        exact::whole(v.rent) - fixed - &operating_allowed - &depreciation_allowed,
        None,
        depreciation_allowed.clone(),
        home,
        false,
        [operating - operating_allowed, exact::whole(tiers.carry.total_cents)],
        personal_fixed,
    )?;
    if let Some(allowed) = tiers.allowed_depreciation_cents() {
        result.depreciation = allowed;
    }
    // SectionB owns casualty, while ScheduleE includes only depreciation.
    result.net = result
        .net
        .checked_add(otherwise)
        .and_then(|amount| amount.checked_add(tiers.allowed_excess_casualty_cents))
        .ok_or(unavailable)?;
    result.otherwise_casualty_cents = otherwise;
    let weights: Vec<_> = casualty_losses.iter().map(|(_, loss)| *loss).collect();
    if !casualty_losses.is_empty() {
        let excess_weights: Vec<_> = weights
            .iter()
            .zip(&otherwise_by_cost)
            .map(|(loss, allowance)| loss - allowance)
            .collect();
        let source_cents: Vec<_> = crate::kernel::ratio::allocate_remaining(
            tiers.allowed_excess_casualty_cents,
            &excess_weights,
        )
        .into_iter()
        .zip(&otherwise_by_cost)
        .map(|(excess, allowance)| excess + allowance)
        .collect();
        // Use the same rounded allowed operating categories as ScheduleE before
        // depreciation/prior third-tier use. The printed casualty cannot exceed
        // this completed Worksheet5-1 room, even when source cents fit in full.
        let operating_rows = rental_expense_lines(v, &result, itemizing, fixed_allowed)?;
        let operating_dollars =
            crate::kernel::ratio::checked_sum(operating_rows.into_iter().map(dollars))
                .ok_or(unavailable)?;
        let printed_otherwise = dollars(otherwise);
        let printed_remainder = dollars(v.rent) - operating_dollars - printed_otherwise;
        let printed_room = phase::floor(printed_remainder, 0);
        let source_dollars = casualty_losses
            .iter()
            .map(|(index, _)| {
                crate::casualty::rental::home_source_form(&v.buildings[*index], v, amt)
                    .map(|form| form.loss)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|key| (key, "Review this casualty's printed source worksheet."))?;
        let printed_total =
            crate::kernel::ratio::checked_sum(source_dollars.iter().copied()).ok_or(unavailable)?;
        if printed_otherwise > printed_total {
            return Err((
                "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE",
                "The printed property loss cannot support its otherwise deductible allowance.",
            ));
        }
        let whole_cents = |amount: i64| amount.checked_mul(100).ok_or(unavailable);
        let prior = crate::rental_home_carry::prior(v, amt)
            .map_err(|key| (key, "Review the prior home-use carry sources."))?;
        let printed_prior = dollars(prior.total_cents);
        let printed_prior_casualty = prior
            .casualty_cents
            .map(|casualty| {
                crate::kernel::ratio::project_rounded_pair(
                    printed_prior,
                    [casualty, prior.total_cents - casualty],
                    0,
                )
                .ok_or(unavailable)
                .and_then(|parts| whole_cents(parts[0]))
            })
            .transpose()?;
        // Form4684 and Worksheet5-1 share one final whole-dollar tier. The
        // source-cent pass above describes the original inputs; its rounded
        // amounts do not compete with this form's limits or closing carries.
        let financial = crate::casualty::home::third_tier(
            whole_cents(printed_otherwise)?,
            whole_cents(printed_total - printed_otherwise)?,
            whole_cents(dollars(current_depreciation))?,
            crate::casualty::home::ThirdTierCarry {
                total_cents: whole_cents(printed_prior)?,
                casualty_cents: printed_prior_casualty,
            },
            whole_cents(printed_room)?,
        )
        .map_err(|_| {
            (
                crate::rental_home_carry::OVERFLOW,
                "The printed home-use casualty and carry amounts cannot be represented.",
            )
        })?;
        let reported_otherwise = crate::kernel::ratio::allocate_remaining(
            dollars(financial.otherwise_allowed_casualty_cents),
            &otherwise_by_cost,
        );
        if reported_otherwise.iter().zip(&source_dollars).any(|(allowance, loss)| allowance > loss)
        {
            return Err((
                "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE",
                "The casualty's printed property capacities cannot establish its allowance allocation.",
            ));
        }
        let printed_excess: Vec<_> = source_dollars
            .iter()
            .zip(&reported_otherwise)
            .map(|(loss, allowance)| loss - allowance)
            .collect();
        let reported: Vec<_> = crate::kernel::ratio::allocate_remaining(
            dollars(financial.allowed_excess_casualty_cents),
            &printed_excess,
        )
        .into_iter()
        .zip(reported_otherwise)
        .map(|(excess, allowance)| excess + allowance)
        .collect();
        result.current_casualty_losses = casualty_losses
            .into_iter()
            .zip(source_cents)
            .zip(reported)
            .map(|(((building, _), cents), dollars)| HomeCasualtyAdmission {
                building,
                cents,
                dollars,
            })
            .collect();
        result.net = whole_cents(printed_remainder)?
            .checked_sub(financial.total_allowed_cents)
            .and_then(|amount| amount.checked_add(financial.allowed_excess_casualty_cents))
            .and_then(|amount| amount.checked_add(financial.otherwise_allowed_casualty_cents))
            .and_then(|amount| amount.checked_add(result.asset_recapture_cents))
            .ok_or(unavailable)?;
        result.depreciation = financial
            .allowed_depreciation_cents()
            .unwrap_or(financial.total_allowed_cents - financial.allowed_excess_casualty_cents);
        result.carry[1] = financial.carry.total_cents;
        result.home_third_tier = Some(financial);
        return Ok(result);
    }
    result.home_third_tier = Some(tiers);
    Ok(result)
}

// Reuse the completed home worksheet for the property packet.
// Raw loss and continuing depreciation are upstream; neither calls this adapter.
pub(crate) fn rental_home_casualty_losses(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    source: &FederalRental,
    amt: bool,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Result<Vec<HomeCasualtyAdmission>, (&'static str, &'static str)> {
    let draft = form_context.draft;
    let entry = draft.income.iter().find(|entry| entry.id == id).ok_or((
        "DIRECT_ACTIVITY_SOURCE_UNAVAILABLE",
        "This casualty needs its accepted rental source.",
    ))?;
    rental_use_core(
        source,
        amt,
        crate::depreciation::mid_quarter_with_macrs(draft, Some(form_context.macrs().as_ref())),
        home_use.and_then(|context| context.itemizes).unwrap_or(false),
        home_use.and_then(|context| home_fixed_allowed(form_context, id, context.agi, amt)),
        &rental_expense_rows(form_context, entry, true),
        Some((draft, id, true, home_use)),
        Some(form_context.macrs().as_ref()),
    )
    .map(|usage| usage.current_casualty_losses)
}

pub(crate) fn expense_output_unavailable(result: &mut FederalResult, source: Option<i64>) {
    let context = source.map_or(
        crate::IssueContext::Section(crate::TaxSection::Rentals),
        crate::IssueContext::income_id,
    );
    crate::issue_localized!(
        result,
        &context,
        "Rental or royalty expenses",
        "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE",
        "A required rental or royalty expense output cannot be represented in its native amount. No final tax can be reported.",
    );
    crate::federal_rules::withhold_required_source_result(result);
}

pub(crate) fn rental_expense_amount_issue() -> (&'static str, &'static str) {
    (
        "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE",
        crate::fixture_english!(
            "The completed rental expense, carryover or operating amount cannot be represented in its required native output. No final tax can be reported."
        ),
    )
}

// The annual native owner already recognized this exact whole-Dollar amount.
// This changes working units only; it never reconstructs source-cent precision.
pub(crate) fn annual_dollars_to_working_cents(
    amount: i64,
) -> Result<i64, crate::kernel::loss_limits::LossLimitError> {
    amount.checked_mul(100).ok_or(crate::kernel::loss_limits::LossLimitError::Overflow)
}

pub(crate) fn post_pal_character(
    character: PostPalCharacter,
    net_cents: i64,
    suspended_cents: i64,
) -> PostPalCharacterAmounts {
    PostPalCharacterAmounts {
        character,
        net_cents,
        suspended_cents,
        allowed_net_cents: net_cents + suspended_cents,
    }
}

pub(crate) fn post_pal_class_amounts(
    character: PostPalCharacter,
    amount: &crate::kernel::loss_limits::CharacterSettlement<
        '_,
        usize,
        crate::kernel::carry::Cents,
    >,
) -> PostPalCharacterAmounts {
    PostPalCharacterAmounts {
        character,
        net_cents: amount.net,
        suspended_cents: amount.held(),
        allowed_net_cents: amount.allowed_net(),
    }
}

// K-1 box 2 and 3 activities of one K-1 in this pass,
// with the regular-tax activities MAGI and the NIIT safe harbor read in both
// passes. Returns [nonpassive income, MAGI, passive profit, NIIT
// adjustment]. MAGI includes a professional's nonpassive income but not
// its allowed loss (i8582 line 6); passive amounts never enter it.
pub(crate) fn pass_through_rentals<'a>(
    values: &mut alloc::vec::Vec<PassiveEntry<'a>>,
    id: i64,
    kind: &'static str,
    current: alloc::vec::Vec<crate::k1_rental::RentalActivity>,
    regular: &[crate::k1_rental::RentalActivity],
    niit_safe_harbor: bool,
    grouped: bool,
) -> [i64; 4] {
    let mut sums = [0; 4];
    for activity in regular {
        if !grouped || !activity.real_estate {
            sums[1] +=
                if activity.released { activity.nonpassive } else { positive(activity.nonpassive) };
        }
        if niit_safe_harbor && activity.material {
            sums[3] -= activity.characters.map_or(activity.nonpassive, |parts| parts.nonpassive[0])
                + activity.niit_disposition_loss[0];
        }
    }
    for activity in current {
        sums[0] += activity.nonpassive;
        sums[2] += positive(activity.passive);
        values.push(PassiveEntry::pass_through(id, kind, activity, niit_safe_harbor));
    }
    sums.map(dollars)
}

// Each printed Schedule E component retains its own rounding boundary.
// Form 4952 and MAGI transfer those final printed amounts, once per PTP.
pub(crate) fn ptp_reported_net(source: &crate::ptp_passive::PtpPalResult) -> Option<i64> {
    use crate::ptp_passive::PtpPalCharacter::{OtherRental, RentalRealEstate};
    let ordinary = source.ordinary_reported_net()?;
    crate::kernel::ratio::checked_sum(
        core::iter::once(ordinary).chain(
            source
                .components
                .iter()
                .filter(|part| matches!(part.original.character, RentalRealEstate | OtherRental))
                .map(|part| dollars(part.allowed_passive_net_cents)),
        ),
    )
}

pub(crate) fn ptp_reported_magi(source: &crate::ptp_passive::PtpPalResult) -> Option<i64> {
    let section179_nonpassive = crate::kernel::ratio::checked_sum(
        source
            .components
            .iter()
            .filter(|part| {
                part.original.character == crate::ptp_passive::PtpPalCharacter::Section179
            })
            .map(|part| part.original.nonpassive_cents),
    )?;
    ptp_reported_net(source)?.checked_add(dollars(section179_nonpassive))
}

// The page that records a filer's real estate professional status: the
// owner's own, as a correction of the rental or K-1 that needs it.
pub(crate) fn status_context(
    d: &FederalDraft,
    e: &crate::FederalIncomeEntry,
) -> crate::IssueContext {
    crate::interview::income_issue_context(e).corrective(
        if crate::community::professional_owner(d.setup.as_ref(), e.owner.owned()) == Owner::Spouse
        {
            crate::TaxSection::Spouse
        } else {
            crate::TaxSection::People
        },
    )
}

// All supported rentals are passive; dispositions and changes of historical character are screened.
// Special allowance applies only to active-participation loss activities. Other rental
// profit offsets passive losses before the allowance; portfolio income never does.
#[cfg(test)]
pub(crate) fn rentals(d: &FederalDraft, magi: i64, r: &mut FederalResult) -> RentalResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    rentals_with_claims(d, magi, r, &source_claims)
}

#[cfg(test)]
pub(crate) fn rentals_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    rentals_for_with_claims(d, magi, r, false, source_claims)
}

// 26 CFR 1.55-1(b): use regular-tax MAGI for the allowance in both engines.
// Only the activity amounts and suspended-loss balances differ for AMT.
#[cfg(test)]
pub(crate) fn rentals_for(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
) -> RentalResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    rentals_for_with_claims(d, magi, r, amt, &source_claims)
}

#[cfg(test)]
pub(crate) fn rentals_for_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    rentals_for_home_with_claims(d, magi, r, amt, [magi, magi], source_claims)
}

#[cfg(test)]
pub(crate) fn rentals_for_home_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    rentals_for_home_with_care_with_claims(
        d,
        magi,
        r,
        amt,
        home_agi,
        &crate::care::CareDeductions::default(),
        source_claims,
    )
}

#[cfg(test)]
pub(crate) fn rentals_for_home_with_care(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    care: &crate::care::CareDeductions,
) -> RentalResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    rentals_for_home_with_care_with_claims(d, magi, r, amt, home_agi, care, &source_claims)
}

#[cfg(test)]
pub(crate) fn rentals_for_home_with_care_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    care: &crate::care::CareDeductions,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    rentals_for_home_with_section179_with_claims(
        d,
        magi,
        r,
        amt,
        home_agi,
        (care, None),
        source_claims,
    )
}

// The selected allocator owns amounts and availability; this adapter only
// forwards that same result. The legacy entry keeps its original lazy regular
// owner calls, including its existing unresolved PTP/AMT routing boundaries.
#[cfg(test)]
pub(crate) fn rentals_for_home_with_section179(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    sources: (&crate::care::CareDeductions, Option<&crate::Section179PreparedAllocation>),
) -> RentalResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    rentals_for_home_with_section179_with_claims(d, magi, r, amt, home_agi, sources, &source_claims)
}

#[cfg(test)]
pub(crate) fn rentals_for_home_with_section179_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_agi: [i64; 2],
    sources: (&crate::care::CareDeductions, Option<&crate::Section179PreparedAllocation>),
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    let home = HomeUseContext::provisional(d, home_agi, None);
    rentals_with_prepared_sources_with_claims(
        crate::related_forms::RelatedFormContext::original(d, source_claims),
        magi,
        r,
        amt,
        &home,
        sources,
        None,
        None,
        None,
        None,
    )
}

// Forward the existing invocation operands and its borrowed home context together.

#[allow(clippy::too_many_arguments)]
pub(crate) fn rentals_for_home_with_selected_section179_with_claims(
    d: &FederalDraft,
    magi: i64,
    r: &mut FederalResult,
    amt: bool,
    home_use: &HomeUseContext,
    care: &crate::care::CareDeductions,
    selected: Option<&[crate::section179_return::Selected; 2]>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> RentalResult {
    let regular_section179 = selected.map(|sources| &sources[0].allocation);
    let regular_ptp = selected.map(|sources| &sources[0].ptp);
    let selected = selected.map(|sources| &sources[usize::from(amt)]);
    let native = if selected.is_none() && !d.home_sites.is_empty() {
        let source = home
            .ok_or(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                0,
                "HOME_CONTEXT_REQUIRED",
            ))
            .and_then(|home| {
                crate::section179::calculate_with_home_with_claims(d, home, source_claims)
            });
        match source {
            Ok(source) => Some(source.pass_through),
            Err(issue) => {
                let (id, key) = match &issue {
                    crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key)
                    | crate::Section179PreparedIssue::InvalidSource(id, key) => (*id, *key),
                    _ => (0, "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"),
                };
                crate::section179_return::Issue::Prepared(issue).report(r);
                return RentalResult {
                    source_issue: Some(crate::activity_risk::SourceIssue::new(
                        id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    )),
                    ..Default::default()
                };
            }
        }
    } else {
        None
    };
    rentals_with_prepared_sources_with_claims(
        home.map_or_else(
            || crate::related_forms::RelatedFormContext::original(d, source_claims),
            |home| home.source_context(d, source_claims),
        ),
        magi,
        r,
        amt,
        home_use,
        (care, selected.map(|source| &source.allocation)),
        selected.map(|source| &source.ptp),
        native,
        regular_section179,
        regular_ptp,
    )
}
