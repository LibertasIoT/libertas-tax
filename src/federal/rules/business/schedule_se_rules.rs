// schedule_se items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Schedule SE ordinary nonfarm method; all returned amounts are whole dollars.
pub(crate) fn self_employment(
    profit_cents: i64,
    social_wages_cents: i64,
    church_wages_cents: i64,
) -> (i64, i64, i64) {
    let net = fraction(positive(dollars(profit_cents)), 9235, 10000);
    se_from_net(net, social_wages_cents, church_wages_cents)
}
// Schedule SE line 5b: church employee income (line 5a) times 92.35%,
// entering 0 when line 5b is under $100 (i1040sse 2026 draft).
pub(crate) fn church_employee_se(church_wages_cents: i64) -> i64 {
    let se_income = fraction(dollars(church_wages_cents), 9235, 10000);
    if se_income < 100 { 0 } else { se_income }
}
pub(crate) fn se_from_net(
    net: i64,
    social_wages_cents: i64,
    church_wages_cents: i64,
) -> (i64, i64, i64) {
    // Line 4c: "If less than $400, stop ... Exception: If less than $400 and you
    // had church employee income, enter -0- and continue" (2026 draft Schedule
    // SE). The small amount is then not part of line 6.
    let church_5b = church_employee_se(church_wages_cents);
    if net < 400 && church_5b == 0 {
        return (0, 0, 0);
    }
    let net = if net < 400 { 0 } else { net };
    // Line 6 = 4c + 5b. The $184,500 wage-base coordination (lines 7-9)
    // applies to the combined line 6 for the 12.4% portion only; the 2.9%
    // portion is never capped, for 4c or 5b alike.
    let net = net + church_5b;
    let tax = dollars(
        crate::kernel::ratio::trunc_i64(
            phase::cap(
                net,
                [positive(
                    crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_DOLLARS
                        - dollars(social_wages_cents),
                )],
            ) * 124,
            10,
        ) + crate::kernel::ratio::trunc_i64(net * 29, 10),
    );
    (net, tax, fraction(tax, 1, 2))
}

// One person's Schedule SE (each spouse files a separate one): line 3 is
// the combined net profit, 4a its 92.35%, 4c the net earnings before church
// employee income, 12 the tax and 13 the half deducted on Schedule 1 line
// 15. With a farm, line 1a is the Schedule F net profit, 1b the excluded
// Conservation Reserve Program payments and line 2 the nonfarm rest of line 3.
// Forward the same selected allocation and invocation-only HOME facts.

#[allow(clippy::too_many_arguments)]
pub(super) fn schedule_se_lines_with_claims(
    r: &mut FederalResult,
    d: &FederalDraft,
    owner: Owner,
    amounts: [i64; 4],
    optional: Option<crate::optional_se::OptionalLines>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<(), crate::Section179PreparedIssue> {
    let [profit, earnings, tax, half] = amounts;
    // An elected optional method: "skip lines 1a and 1b if you use the farm
    // optional method", "skip line 2 if you use the nonfarm optional method";
    // line 3 combines what is printed and line 4b adds lines 15 and 17.
    let (farm_elected, nonfarm_elected) = optional.map_or((false, false), |o| (o.farm, o.nonfarm));
    let profit = optional.map_or(profit, |o| o.regular_profit);
    let person = if owner == Owner::Spouse { "spouse" } else { "taxpayer" };
    let index = usize::from(owner == Owner::Spouse);
    let (mut farm, mut farm_profit, mut crp) = (false, 0, 0);
    for e in d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)) {
        let source = match &e.income {
            FederalIncome::Business { data }
                if matches!(data.activity, crate::BusinessActivity::Farming { .. }) =>
            {
                Some((
                    data.receipts()
                        + farm_routed_income_with_claims(d, index, e.id, source_claims)?
                        - crate::section179_return::expenses_with_claims(
                            d,
                            e.id,
                            data,
                            selected,
                            home,
                            source_claims,
                        )?,
                    data.se_exclusion(),
                ))
            }
            FederalIncome::Partnership { data }
                if matches!(data.se_gross, Some(crate::PartnerSeGross::Farm { .. })) =>
            {
                // Use the same regular, post-limit source that entered this
                // owner's line 3 and wage-base calculation. Gross code B
                // identifies farming; it is not itself taxable SE profit.
                partners
                    .iter()
                    .find(|(id, _)| *id == e.id)
                    .map(|(_, source)| (source.earnings * 100, 0))
            }
            FederalIncome::PartnershipActivities { data } => data
                .ordinary_self_employment()
                .ok()
                .flatten()
                .filter(|source| source.category == 0)
                .and_then(|source| {
                    partners
                        .iter()
                        .find(|(id, resolved)| *id == e.id && *resolved == source.net)
                        .map(|(_, resolved)| (resolved.earnings * 100, 0))
                }),
            _ => None,
        };
        if let Some((profit, payments)) = source {
            farm = true;
            farm_profit += profit;
            crp += payments;
        }
    }
    let farm = farm && !farm_elected;
    let (farm_profit, crp) = (dollars(farm_profit), dollars(crp));
    let source = "https://www.irs.gov/instructions/i1040sse";
    let clergy = crate::clergy::calculate_with_section179_with_claims(
        d,
        owner,
        selected,
        home,
        source_claims,
    )?;
    let exemption = crate::clergy::se_exemption_with_section179_with_claims(
        d,
        owner,
        earnings,
        selected,
        home,
        source_claims,
    )?;
    for (line, suffix, amount, explanation) in [
        ("1a", "se_farm_profit", farm.then_some(farm_profit), msg!("EXPLAIN_N_N_1A")),
        ("1b", "se_crp_payments", farm.then_some(-crp), msg!("EXPLAIN_N_N_1B")),
        (
            "2",
            "se_nonfarm_profit",
            (!nonfarm_elected).then_some(if farm { profit - farm_profit + crp } else { profit }),
            if clergy.se_line_2 != 0 {
                crate::Message::new(
                    "EXPLAIN_SE_NONFARM_WITH_CLERGY",
                    alloc::vec![
                        crate::MessageArg::usd_cents(clergy.se_line_2),
                        crate::MessageArg::Number(i64::from(exemption.line_a))
                    ],
                )
            } else {
                msg!("EXPLAIN_N_N_N_3")
            },
        ),
        (
            "3",
            "se_combined_profit",
            Some(profit),
            if exemption.notary_profit != 0 {
                crate::Message::new(
                    "EXPLAIN_SE_COMBINED_WITH_NOTARY",
                    alloc::vec![crate::MessageArg::usd_cents(exemption.notary_profit)],
                )
            } else {
                msg!("EXPLAIN_N_N_N_4")
            },
        ),
        (
            "4a",
            "se_net_earnings_ordinary",
            Some(if profit > 0 { fraction(profit, 9235, 10000) } else { profit }),
            msg!("EXPLAIN_N_N_4A"),
        ),
        (
            "4b",
            "se_optional_earnings",
            optional.map(|o| o.optional_earnings),
            msg!("EXPLAIN_N_N_4B"),
        ),
        ("4c", "se_net_earnings", Some(earnings), msg!("EXPLAIN_N_N_4C")),
        ("12", "self_employment_tax", Some(tax), msg!("EXPLAIN_N_N_N_5")),
        ("13", "half_self_employment_tax", Some(half), msg!("EXPLAIN_N_N_N_6")),
    ] {
        if let Some(amount) = amount {
            form_line(
                r,
                FormLine::new(TaxForm::ScheduleSe, line),
                &alloc::format!("{person}_{suffix}"),
                amount,
                &["income.business", "income.partnership", "income.social_wages"],
                source,
                explanation,
            );
        }
    }
    Ok(())
}

pub(super) struct ScheduleSeIn<'a> {
    pub(super) partners: &'a [(i64, crate::partnership::PartnerSeResult)],
    pub(super) foreign: &'a mut crate::extended_rules::ForeignResult,
}

pub(super) struct ScheduleSeOut {
    pub(super) se: [i64; 2],
    pub(super) se_profit: [i64; 2],
    pub(super) half: [i64; 2],
    pub(super) net: [i64; 2],
    pub(super) earned: [i64; 2],
    pub(super) compensation: [i64; 2],
    pub(super) compensation_earned_profit: [i64; 2],
    pub(super) optional_eic: [i64; 2],
    pub(super) optional_child: [i64; 2],
    pub(super) church_half: [i64; 2],
    pub(super) se_church_wages: [i64; 2],
    pub(super) business_se_tax: [i64; 2],
    pub(super) se_sources_complete: [bool; 2],
    pub(super) optional_weights: alloc::collections::BTreeMap<i64, i64>,
    pub(super) optional_weights_cents: alloc::collections::BTreeMap<i64, i64>,
    pub(super) business_half_se: alloc::collections::BTreeMap<i64, i64>,
    pub(super) business_half_se_cents: alloc::collections::BTreeMap<i64, i64>,
    pub(super) business_excluded: i64,
    pub(super) income_excluded: i64,
    pub(super) housing_deduction: i64,
    pub(super) excluded: i64,
    pub(super) business_exclusions: [crate::foreign_business::BusinessExclusion; 2],
}

pub(super) fn schedule_se_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: ScheduleSeIn<'_>,
) -> core::ops::ControlFlow<(), ScheduleSeOut> {
    let &Pass { d, context, allocated_entries, office_context, j, selected_regular, .. } = pass;
    let source_claims = context.source_claims;
    let ScheduleSeIn { partners, foreign } = i;
    let mut se = [0; 2];
    // Each person's Schedule SE net profit before the 92.35% reduction, for
    // Schedule AI column (d).
    let mut se_profit = [0; 2];
    let mut half = [0; 2];
    let mut net = [0; 2];
    let mut se_sources_complete = [true; 2];
    let mut earned = [0; 2];
    let mut compensation = [0; 2];
    let mut compensation_earned_profit = [0; 2];
    let mut optional_eic = [0; 2];
    let mut optional_child = [0; 2];
    let mut optional_weights = alloc::collections::BTreeMap::new();
    let mut optional_weights_cents = alloc::collections::BTreeMap::new();
    // The share of `half[i]` attributable to church employee income (line
    // 5a-5b), which is not a QBI trade or business: `half_se_by_gross`
    // allocates only against Business/Partnership gross-income sources, so
    // this share is excluded from that allocation below.
    let mut church_half = [0i64; 2];
    let mut se_church_wages = [0i64; 2];
    let mut business_se_tax = [0i64; 2];
    let mut business_half_se = alloc::collections::BTreeMap::new();
    let mut business_half_se_cents = alloc::collections::BTreeMap::new();
    let mut business_excluded = 0;
    let mut business_exclusions: [crate::foreign_business::BusinessExclusion; 2] =
        core::array::from_fn(|_| Default::default());
    if !j
        && d.screening
            .as_ref()
            .is_some_and(|s| s.optional_se.iter().any(|e| e.owner == Owner::Spouse))
    {
        note_once(r, msg!("WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
    }
    for i in 0..2 {
        let owner = if i == 0 { Owner::Taxpayer } else { Owner::Spouse };
        let clergy = match crate::clergy::calculate_with_section179_with_claims(
            d,
            owner,
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
        crate::clergy::lines(r, owner, &clergy);
        let clergy_extra = clergy.extra_se();
        let church_wages = if clergy.sect { 0 } else { t.church_wages[i] };
        se_church_wages[i] = church_wages;
        let combined = (!partners.is_empty())
            .then(|| {
                crate::partnership::owner_actual_se_with_section179_with_claims(
                    context.forms(d),
                    owner,
                    t.social_wages[i],
                    church_wages,
                    partners,
                    selected_regular,
                    Some(office_context),
                )
            })
            .transpose();
        let combined = match combined {
            Ok(value) => value,
            Err(crate::partnership::OwnerSeIssue::Prepared(issue)) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
            Err(crate::partnership::OwnerSeIssue::Rule((key, _message))) => {
                se_sources_complete[i] = false;
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                    "Partnership owner self-employment",
                    key,
                    _message,
                );
                None
            }
        };
        let earned_profit = t.business[i] - t.business_se_exclusion[i] + clergy_extra;
        let profit = combined.as_ref().map_or(dollars(earned_profit), |v| v.profit);
        se_profit[i] = profit;
        (net[i], se[i], half[i]) = combined.as_ref().map_or_else(
            || self_employment(earned_profit, t.social_wages[i], church_wages),
            |v| (v.net, v.tax, v.half),
        );
        let before_optional_issues = r.issues.len();
        let optional = if partners.is_empty() {
            match crate::optional_se::calculate_with_section179_with_claims(
                d,
                owner,
                selected_regular,
                r,
                Some(office_context),
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        } else {
            match crate::optional_se::calculate_with_partners_with_section179_with_claims(
                d,
                owner,
                Some(partners),
                selected_regular,
                r,
                Some(office_context),
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        };
        se_sources_complete[i] &= r.issues.len() == before_optional_issues;
        let optional_lines = optional.as_ref().map(|o| o.lines);
        let clergy_optional_net = optional.as_ref().map(|o| o.nonbusiness_net);
        let mut allocation = if let Some(optional) = optional {
            (net[i], se[i], half[i]) = se_from_net(optional.net, t.social_wages[i], church_wages);
            optional_eic[i] = optional.eic_adjustment;
            optional_child[i] = optional.child_adjustment;
            let allocation =
                crate::business_rules::half_se_allocation(half[i], optional.weights.clone());
            business_half_se_cents.extend(crate::business_rules::half_se_allocation(
                half[i] * 100,
                optional.weights_cents.clone(),
            ));
            optional_weights_cents.extend(optional.weights_cents);
            optional_weights.extend(optional.weights);
            allocation
        } else if let Some(combined) = combined {
            business_half_se_cents.extend(combined.attribution_cents);
            combined.attribution
        } else {
            match crate::business_rules::half_se_by_business_with_section179_with_claims(
                d,
                i,
                half[i],
                &optional_weights,
                selected_regular,
                Some(office_context),
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        };
        let church_5b = church_employee_se(church_wages);
        let nonbusiness_net = church_5b
            + clergy_optional_net
                .unwrap_or_else(|| fraction(positive(dollars(clergy_extra)), 9235, 10000));
        let business_se =
            crate::business_rules::business_se_totals(se[i], half[i], net[i], nonbusiness_net);
        business_se_tax[i] = business_se[0];
        church_half[i] = half[i] - business_se[1];
        if nonbusiness_net != 0 {
            allocation =
                crate::business_rules::half_se_allocation(half[i] - church_half[i], allocation);
            let cents: alloc::vec::Vec<_> = business_half_se_cents
                .iter()
                .filter(|(id, _)| {
                    allocated_entries
                        .iter()
                        .any(|e| e.id == **id && e.owner == crate::DocumentOwner::from(owner))
                })
                .map(|(id, value)| (*id, *value))
                .collect();
            business_half_se_cents.extend(crate::business_rules::half_se_allocation(
                (half[i] - church_half[i]) * 100,
                cents,
            ));
        }
        business_half_se.extend(allocation);
        if se_profit[i] != 0 || se[i] != 0 {
            match schedule_se_lines_with_claims(
                r,
                d,
                owner,
                [se_profit[i], net[i] - church_5b, se[i], half[i]],
                optional_lines,
                partners,
                selected_regular,
                Some(office_context),
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            };
        }
        let business_exclusion = match crate::foreign_business::exclude_with_section179_with_claims(
            d,
            i,
            &business_half_se,
            foreign.earned_wages[i],
            foreign.gross_excluded[i],
            selected_regular,
            r,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
        business_excluded += business_exclusion.business_net + business_exclusion.housing_deduction;
        foreign.gross_excluded[i] += business_exclusion.gross_adjustment;
        foreign.excluded[i] += business_exclusion.net_adjustment;
        foreign.housing_exclusion[i] += business_exclusion.housing_exclusion;
        foreign.elected[i] |= business_exclusion.elected;
        // Prior-service exclusion belongs only to line45, after current-year
        // business/line44 and housing-carryover computations.
        foreign.gross_excluded[i] += foreign.prior_excluded[i];
        foreign.excluded[i] += foreign.prior_excluded[i];
        let (clergy_compensation, totalization) =
            match crate::clergy::earned_adjustments_with_section179_with_claims(
                d,
                owner,
                selected_regular,
                Some(office_context),
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            };
        earned[i] = earned_before_special_adjustments(t, i, profit, half[i]) + foreign.wages[i]
            - foreign.excluded[i]
            + business_exclusion.statutory_net_excluded
            - business_exclusion.statutory_gross_excluded
            + dollars(
                totalization
                    + if !clergy.exempt && !clergy.sect {
                        -clergy.wages
                    } else {
                        clergy.housing_excess
                    },
            );
        optional_child[i] -=
            if !clergy.exempt && !clergy.sect { dollars(clergy.housing_tax_free) } else { 0 };
        // Pub. 590-A: a business loss does not reduce wages, but the half-SE
        // deduction still reduces compensation, including optional-method tax.
        // A disability pension reported on line 1h is earned income (Pub. 596)
        // but not compensation: "Pension or annuity income" is not (Pub. 590-A).
        compensation_earned_profit[i] = profit + dollars(clergy_compensation - clergy_extra)
            - (business_exclusion.business_net - business_exclusion.statutory_net_excluded);
        compensation[i] = dollars(
            t.wages[i]
                + t.inmate_pay[i]
                + t.iso_unreported[i]
                + t.moving_excess[i]
                + t.deferral_excess[i]
                + t.alimony_compensation[i]
                + t.graduate_compensation[i]
                + t.household_wages[i]
                + t.medicaid_taxable[i]
                + t.statutory_gross[i]
                + t.combat_pay[i]
                - t.medicaid_in_box1[i],
        ) + positive(compensation_earned_profit[i])
            - half[i]
            + dollars(clergy.housing_excess)
            + foreign.wages[i]
            - (foreign.excluded[i] - business_exclusion.business_net)
            - business_exclusion.statutory_gross_excluded;
        if i == 0 && t.community_earned != 0 {
            // Pub. 590-A uses this worker's own compensation for IRA
            // contributions; Pub. 555 uses the worker's own earnings for
            // EIC even though community wages enter AGI at their share.
            let personal_delta = dollars(t.community_earned);
            earned[i] += personal_delta;
            compensation[i] += personal_delta;
        }
        business_exclusions[i] = business_exclusion;
    }
    line(
        r,
        "foreign_housing_exclusion",
        foreign.housing_exclusion.iter().sum(),
        &["income.foreign_employment.housing", "income.business.source.housing"],
        "https://www.irs.gov/instructions/i2555",
        msg!("EXPLAIN_FOREIGN_HOUSING_EXCLUSION"),
    );
    let income_excluded = foreign.excluded.iter().sum::<i64>();
    let housing_deduction: i64 =
        business_exclusions.iter().map(|v| v.housing_deduction).sum::<i64>()
            + foreign.housing_deduction.iter().sum::<i64>();
    // AGI and section 911 addback/rate worksheets include both Form 2555
    // lines 45 and 50; compensation above uses only actual income exclusions.
    let excluded = income_excluded + housing_deduction;
    line(
        r,
        "foreign_housing_deduction",
        housing_deduction,
        &["income.business.source.housing", "income.foreign_employment.housing"],
        "https://www.irs.gov/instructions/i2555",
        msg!("EXPLAIN_FOREIGN_HOUSING_DEDUCTION"),
    );
    line(
        r,
        "foreign_earned_exclusion",
        foreign.gross_excluded.iter().sum(),
        &["income.foreign_employment", "income.business.source"],
        "https://www.irs.gov/instructions/i2555",
        msg!("EXPLAIN_FOREIGN_EARNED_EXCLUSION"),
    );
    core::ops::ControlFlow::Continue(ScheduleSeOut {
        se,
        se_profit,
        half,
        net,
        earned,
        compensation,
        compensation_earned_profit,
        optional_eic,
        optional_child,
        church_half,
        se_church_wages,
        business_se_tax,
        se_sources_complete,
        optional_weights,
        optional_weights_cents,
        business_half_se,
        business_half_se_cents,
        business_excluded,
        income_excluded,
        housing_deduction,
        excluded,
        business_exclusions,
    })
}

pub(super) struct PartnershipSeReportingIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) se_tax: i64,
    pub(super) half_se: i64,
    pub(super) health_deduction: i64,
    pub(super) sep_deduction: i64,
    pub(super) rental_magi_base: i64,
}

pub(super) fn partnership_se_reporting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    partner_pass: &PartnerPass,
    i: PartnershipSeReportingIn<'_>,
) -> core::ops::ControlFlow<()> {
    let &Pass { d, context, j, selected_regular, office_context, .. } = pass;
    let source_claims = context.source_claims;
    let PartnershipSeReportingIn {
        rental,
        se_tax,
        half_se,
        health_deduction,
        sep_deduction,
        rental_magi_base,
    } = i;
    if !partner_pass.inputs.partners.is_empty() && rental.inputs_complete {
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Partnership SE coordination stabilized: return-stage SE tax ${se_tax}; half-SE deduction ${half_se}; health deduction ${}; retirement plan deduction ${sep_deduction}; rental MAGI base ${rental_magi_base}. The regular passive result agrees with the source amounts used by the earlier earnings calculation. Additional partnership items require their own source records and review.", health_deduction,
        ), Message::new("NOTE_PARTNERSHIP_SE_COORDINATION", alloc::vec![
            MessageArg::usd_dollars(se_tax), MessageArg::usd_dollars(half_se),
            MessageArg::usd_dollars(health_deduction), MessageArg::usd_dollars(sep_deduction),
            MessageArg::usd_dollars(rental_magi_base),
        ])));
    }
    if rental.inputs_complete && !rental.partners.is_empty() {
        for (i, owner) in [Owner::Taxpayer, Owner::Spouse].into_iter().enumerate() {
            if owner == Owner::Spouse && !j {
                continue;
            }
            match crate::partnership::owner_actual_se_with_section179_with_claims(
                context.forms(d),
                owner,
                t.social_wages[i],
                t.church_wages[i],
                &rental.partner_se,
                selected_regular,
                Some(office_context),
            ) {
                Ok(actual) => {
                    #[cfg(test)]
                    let label = if i == 0 { "taxpayer" } else { "spouse" };
                    r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                        "Partnership-inclusive actual-method SE projection for {label}: combined profit ${}; net earnings ${}; SE tax ${}; half-SE deduction ${}. The wage base is shared across this owner's businesses and partnerships. This shows the actual-method alternative; an elected optional method replaces it in the coordinated calculation. Additional partnership items require their own source records and review.",
                        actual.profit, actual.net, actual.tax, actual.half,
                    ), Message::new(if i == 0 { "NOTE_PARTNERSHIP_SE_ACTUAL_TAXPAYER" } else { "NOTE_PARTNERSHIP_SE_ACTUAL_SPOUSE" }, alloc::vec![
                        MessageArg::usd_dollars(actual.profit), MessageArg::usd_dollars(actual.net),
                        MessageArg::usd_dollars(actual.tax), MessageArg::usd_dollars(actual.half),
                    ])));
                    for (id, amount) in actual.attribution {
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)] alloc::format!("Actual-method SE deduction attribution, document {id}: ${amount}. Do not deduct this again in Schedule E income."),
                            crate::Message::new("NOTE_SE_ACTUAL_ATTRIBUTION", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: amount }]),
                        ));
                    }
                    let before_optional = r.issues.len();
                    let optional = match crate::optional_se::calculate_with_partners_with_section179_with_claims(
                        d,
                        owner,
                        Some(&rental.partner_se),
                        selected_regular,
                        r,
                        Some(office_context),
                        source_claims,
                    ) {
                        Ok(value) => value,
                        Err(issue) => {
                            crate::section179_return::Issue::from(issue).report(r);
                            r.state = ResultState::Unsupported;
                            return core::ops::ControlFlow::Break(());
                        }
                    };
                    if let Some(optional) = optional
                        && r.issues.len() == before_optional
                    {
                        let (net, tax, half) =
                            se_from_net(optional.net, t.social_wages[i], t.church_wages[i]);
                        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                            "Partnership-inclusive elected optional-method SE projection for {label}: net earnings ${net}; SE tax ${tax}; half-SE deduction ${half}. EIC earned-income adjustment ${}; child-credit earned-income adjustment ${}. Actual Schedule E income is unchanged. Intermediate only; final return coordination remains incomplete.",
                            optional.eic_adjustment, optional.child_adjustment,
                        ), Message::new(if i == 0 { "NOTE_PARTNERSHIP_SE_OPTIONAL_TAXPAYER" } else { "NOTE_PARTNERSHIP_SE_OPTIONAL_SPOUSE" }, alloc::vec![
                            MessageArg::usd_dollars(net), MessageArg::usd_dollars(tax), MessageArg::usd_dollars(half),
                            MessageArg::usd_dollars(optional.eic_adjustment), MessageArg::usd_dollars(optional.child_adjustment),
                        ])));
                        for (id, amount) in
                            crate::business_rules::half_se_allocation(half, optional.weights)
                        {
                            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                #[cfg(test)] alloc::format!("Elected optional-method SE deduction attribution, document {id}: ${amount}. This replaces, rather than adds to, the actual-method attribution."),
                                crate::Message::new("NOTE_SE_OPTIONAL_ATTRIBUTION", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: amount }]),
                            ));
                        }
                    }
                }
                Err(crate::partnership::OwnerSeIssue::Prepared(issue)) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
                Err(crate::partnership::OwnerSeIssue::Rule((key, _message))) => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                        "Partnership owner self-employment",
                        key,
                        _message,
                    )
                }
            }
        }
    }
    core::ops::ControlFlow::Continue(())
}
