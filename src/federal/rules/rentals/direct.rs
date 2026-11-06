use super::*;

pub(crate) struct DirectInputs<'input, 'source> {
    pub(crate) allocated_entries:
        &'input [alloc::borrow::Cow<'source, crate::FederalIncomeEntry>],
    pub(crate) personal: &'input [(i64, i64, i64)],
    pub(crate) mid_quarter: bool,
    pub(crate) professional_groups: &'input ProfessionalGroups,
    pub(crate) regular_groups: &'input ProfessionalGroups,
    pub(crate) magi: i64,
}

pub(crate) struct DirectOutput<'scorp> {
    pub(crate) magi: i64,
    pub(crate) prior_home_casualty_by_owner: [i64; 2],
    pub(crate) prior_home_casualty_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) source_issue: Option<crate::activity_risk::SourceIssue>,
    pub(crate) nonpassive_income: i64,
    pub(crate) health_work_income_residual_cents: i64,
    pub(crate) part_one: i64,
    pub(crate) profits: i64,
    pub(crate) rental_niit_adjustment: i64,
    pub(crate) income_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    pub(crate) foreign_sources:
        alloc::collections::BTreeMap<i64, crate::rental_foreign::RentalForeignSource>,
    pub(crate) operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    pub(crate) values: alloc::vec::Vec<PassiveEntry<'scorp>>,
    pub(crate) home_activities: alloc::vec::Vec<(i64, UniformBusinessIncome)>,
    pub(crate) released_rental_losses: bool,
    pub(crate) nonpassive_rental_losses: bool,
}

pub(crate) fn project<'scorp>(
    context: &rental_setup::RentalContext<'_, '_>,
    inputs: DirectInputs<'_, '_>,
    r: &mut FederalResult,
) -> DirectOutput<'scorp> {
    let form_context = context.form;
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let amt = context.amt;
    let home_use = context.home_use;
    let DirectInputs {
        allocated_entries,
        personal,
        mid_quarter,
        professional_groups,
        regular_groups,
        mut magi,
    } = inputs;
    let mut prior_home_casualty_by_owner = [0i64; 2];
    let mut prior_home_casualty_by_id = alloc::collections::BTreeMap::new();
    let mut source_issue = None;
    macro_rules! rental_source {
        ($value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    issue.report(r);
                    source_issue.get_or_insert(issue);
                    continue;
                }
            }
        };
    }
    macro_rules! rental_amount {
        ($id:expr, $value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    issue.report($id, r);
                    source_issue.get_or_insert(crate::activity_risk::SourceIssue::new($id, issue));
                    continue;
                }
            }
        };
    }

    let prefix = if amt { "amt_" } else { "" };
    let mut nonpassive_income = 0;
    let mut health_work_income_residual_cents = 0;
    let mut part_one = 0;
    let mut income_by_id: alloc::collections::BTreeMap<i64, i64> = allocated_entries
        .iter()
        .filter(|e| {
            matches!(
                e.income,
                FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Partnership { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::EstateTrustPortfolio { .. }
            )
        })
        .map(|e| (e.id, 0))
        .collect();
    let mut nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64> =
        income_by_id.keys().map(|id| (*id, 0)).collect();
    let mut operating_sources = alloc::collections::BTreeMap::new();
    let mut foreign_sources = alloc::collections::BTreeMap::new();
    let mut operating_prior_allowed = alloc::collections::BTreeMap::new();
    let mut profits = 0;
    let mut values: alloc::vec::Vec<_> = personal
        .iter()
        .filter(|(_, net, _)| *net < 0)
        .map(|(id, net, prior)| PassiveEntry {
            amt_source: None,
            source_cents: None,
            id: *id,
            net: dollars(*net),
            rental_allowance: false,
            ineligible_prior: 0,
            prior: dollars(*prior),
            personal: true,
            direct_business: false,
            scorp: None,
            partner: false,
            source_group: None,
            characters: None,
            items: None,
            nonpassive: 0,
            prior_released: 0,
            uniform_business: None,
            disposition_gain: 0,
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: false,
            ptp: 0,
            pass_through: None,
            released: false,
        })
        .collect();
    let mut home_activities = alloc::vec::Vec::new();
    let mut released_rental_losses = false;
    let mut nonpassive_rental_losses = false;
    // Sec. 469(c)(7): a qualified real-estate professional (the People
    // page's status) who materially participates in a rental removes that
    // rental from the passive rules entirely (no Form 8582 allowance; the loss
    // offsets other income in full, subject to the at-risk and
    // excess-business-loss rules computed elsewhere). Each rental, and each
    // K-1 box 2 activity, records the material participation (or the
    // aggregation election) itself: RentalParticipation::Professional.
    // Reg. 1.1411-4(g)(7): a qualified professional with more than 500 hours
    // in each rental activity this year (or in 5 of the last 10 years) earns
    // the rent in a trade or business; it leaves net investment income.
    let mut rental_niit_adjustment = 0;
    for e in allocated_entries {
        if let FederalIncome::Rental { data: original } = &e.income {
            if let Some((index, issue)) = crate::interest_limit::rental_building_issue(original) {
                let source = crate::interview::income_issue_context(e);
                let context = crate::IssueContext::Scoped {
                    label: Some(source.item("buildings", index).label()),
                    identity: source.identity(),
                    source: alloc::boxed::Box::new(source),
                }
                .manual();
                if !r.issues.iter().any(|old| {
                    old.context.identity() == context.identity() && old.display.key == issue.key
                }) {
                    crate::issue_localized!(
                        r,
                        &context,
                        "Rental building recovery",
                        issue.key,
                        "Retain the actual building recovery and original ADS history.",
                    );
                }
                let issue = crate::activity_risk::SourceIssue::new(
                    e.id,
                    crate::activity_risk::ProjectionIssue::Source(issue.key),
                );
                issue.report(r);
                source_issue.get_or_insert(issue);
                continue;
            }
            let source_before = part_one;
            let source_niit_before = rental_niit_adjustment;
            let projected = original.clone();
            let whole_entry = d.income.iter().find(|entry| entry.id == e.id).expect("source ID");
            let expense_rows = rental_expense_rows(form_context, whole_entry, false);
            let property_entry = crate::community::property_entry(d, whole_entry);
            let FederalIncome::Rental { data: property } = &property_entry.income else {
                unreachable!("rental source")
            };
            let v = &projected;
            if let Some(source) = crate::rental_foreign::RentalForeignSource::new(
                e.id,
                e.owner.owned(),
                amt,
                v,
                &expense_rows,
            ) {
                foreign_sources.insert(e.id, source);
            }
            let label = alloc::format!("Rental: {}", e.label);
            if !amt && straight_lined_points(form_context, e.id, original) {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_RENTAL_POINTS_STRAIGHT_LINE",
                        alloc::vec![crate::MessageArg::Text(label.clone())],
                    ),
                    crate::interview::income_issue_context(e),
                );
            }
            let status = d.professional(crate::community::professional_owner(
                d.setup.as_ref(),
                e.owner.owned(),
            ));
            let re_professional = v.participation.professional()
                && status.is_some()
                && v.property_type.business_participation().is_none();
            // The recorded status is the filer's, not a rental's: a rental
            // that says Professional without it is not nonpassive. A
            // short-stay business variant is not rental real estate, so the
            // participation answer does not apply to it (Reg. 1.469-1T(e)(3)(ii)).
            if !amt {
                crate::require_localized!(
                    r,
                    !v.participation.professional()
                        || re_professional
                        || v.property_type.business_participation().is_some(),
                    &status_context(d, e),
                    &label,
                    "ISSUE_RENTAL_PROFESSIONAL_NEEDS_STATUS",
                    "A rental marked as one in which a real estate professional materially participated needs that owner's Real estate professional answer on About you or About your spouse.",
                );
            }
            let niit_safe_harbor = re_professional && status.is_some_and(|p| p.over_500_hours);
            let business_participation = v.property_type.business_participation();
            let allowance = business_participation.is_none() && v.participation.active();
            if !amt {
                crate::asset_sale::report_rental(d, r, e.id, &e.label, property);
            }
            if v.property_type.substantial_services() {
                if !amt {
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e),
                        &label,
                        "ISSUE_SHORT_TERM_RENTAL_IS_A_BUSINESS",
                        "Substantial guest services belong on Schedule C. Enter the property in Business and remove this rental.",
                    );
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e)
                            .corrective(crate::TaxSection::SelfEmployment),
                        &label,
                        "ISSUE_SHORT_TERM_RENTAL_IS_A_BUSINESS",
                        "Substantial guest services belong on Schedule C. Enter the property in Business and remove this rental.",
                    );
                }
                continue;
            }
            // How the property left, read once: the exchange (if any) whose
            // property received continues this rental.
            let exchange = v.sale.exchange();
            #[cfg(test)]
            if !amt && home_use.dwelling_agi_cents(e.id).is_some() {
                HOME_USE_SEEN.with(|seen| seen.borrow_mut().push((e.id, home_use.clone())));
            }
            let itemizing = home_use.itemizes.unwrap_or(true);
            match rental_use_core(
                v,
                amt,
                mid_quarter,
                itemizing,
                home_fixed_allowed(form_context, e.id, home_use.agi, amt),
                &expense_rows,
                Some((d, e.id, false, Some(home_use))),
                Some(form_context.macrs().as_ref()),
            ) {
                Ok(mut usage) => {
                    if let Some(source) = foreign_sources.get_mut(&e.id) {
                        source.expenses(v, &usage.expense_rows);
                        source.depreciation_cents =
                            crate::kernel::source_fact::SourceFact::Reported(usage.depreciation);
                        source.current_before_risk_cents =
                            usage.current_operating_for_risk.clone().map_or(
                                crate::kernel::source_fact::SourceFact::Unavailable,
                                crate::kernel::source_fact::SourceFact::Reported,
                            );
                    }
                    if let Some(tiers) = usage.home_third_tier {
                        if tiers.otherwise_allowed_casualty_cents > 0
                            || tiers.excess_casualty_cents > 0
                            || crate::rental_home_carry::total(v, amt)
                                .is_some_and(|amount| amount > 0)
                        {
                            for (suffix, amount, explanation) in [
                                (
                                    "home_third_tier_allowed",
                                    Some(tiers.total_allowed_cents),
                                    msg!("EXPLAIN_RENTAL_HOME_THIRD_TIER_ALLOWED"),
                                ),
                                (
                                    "home_casualty_otherwise",
                                    Some(tiers.otherwise_allowed_casualty_cents),
                                    msg!("EXPLAIN_RENTAL_HOME_CASUALTY_OTHERWISE"),
                                ),
                                (
                                    "home_casualty_allowed",
                                    tiers.allowed_casualty_cents(),
                                    msg!("EXPLAIN_RENTAL_HOME_CASUALTY_ALLOWED"),
                                ),
                                (
                                    "home_casualty_carryover",
                                    tiers.carry.casualty_cents,
                                    msg!("EXPLAIN_RENTAL_HOME_CASUALTY_CARRY"),
                                ),
                            ] {
                                if let Some(amount) = amount {
                                    push_line(
                                        r,
                                        None,
                                        &alloc::format!("{prefix}rental_{}_{suffix}", e.id),
                                        amount,
                                        &["income.rental.home_depreciation_carryover"],
                                        RENTAL,
                                        explanation,
                                    );
                                }
                            }
                        }
                        let casualty = tiers.allowed_prior_casualty_cents;
                        // The combined balance remains known without its character;
                        // withhold only the dependent separated deduction/return.
                        let boundary = if casualty.is_none() {
                            Some(crate::rental_home_carry::CHARACTER)
                        } else if !usage.home && casualty.is_some_and(|amount| amount != 0) {
                            // The original casualty's recurring/nonrecurring passive
                            // character is not established by a combined prior carry.
                            Some("RENTAL_HOME_PRIOR_CASUALTY_ACTIVITY_UNAVAILABLE")
                        } else if matches!(v.qbi_business, RentalBusiness::Yes { .. })
                            && casualty.is_some_and(|amount| amount != 0)
                        {
                            Some("RENTAL_HOME_PRIOR_CASUALTY_QBI_UNAVAILABLE")
                        } else {
                            None
                        };
                        if let Some(key) = boundary {
                            carryover_rows(r, prefix, e.id, usage.carry);
                            crate::issue_localized!(
                                r,
                                &crate::interview::income_issue_context(e),
                                &label,
                                key,
                                "Supply the actual character of the prior home-use casualty carry before separated deductions are available."
                            );
                            source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                                e.id,
                                crate::activity_risk::ProjectionIssue::Source(key),
                            ));
                            continue;
                        }
                        if let Some(casualty) = casualty.filter(|amount| *amount != 0) {
                            // HOME's casualty allowance goes to SectionB/Form4797;
                            // it is excluded from ScheduleE depreciation and net.
                            let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
                            let projected = usage
                                .net
                                .checked_add(casualty)
                                .zip(prior_home_casualty_by_owner[owner].checked_sub(casualty));
                            let Some((net, owner_casualty)) = projected else {
                                let issue = crate::activity_risk::ProjectionIssue::Arithmetic(
                                    crate::kernel::loss_limits::LossLimitError::Overflow,
                                );
                                issue.report(e.id, r);
                                source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                                    e.id, issue,
                                ));
                                continue;
                            };
                            usage.net = net;
                            prior_home_casualty_by_owner[owner] = owner_casualty;
                            prior_home_casualty_by_id.insert(e.id, -casualty);
                        }
                    }
                    let prior_passive = [v.history.balance(false), v.history.balance(true)];
                    let ineligible = if allowance && !v.history.active_in_loss_year {
                        v.history.balance(amt)
                    } else {
                        0
                    };
                    // The regular-only QBI projection requires its original GDS
                    // period; AMT business-income bookkeeping needs no UBIA.
                    let ubia = if !amt
                        && !v.location.foreign()
                        && matches!(v.qbi_business, RentalBusiness::Yes { .. })
                    {
                        match (|| -> Result<i64, (&'static str, &'static str)> {
                            Ok(match &v.sale {
                                // Sold property is not held at year-end.
                                crate::RentalSale::Sold { .. } => 0,
                                _ => match exchange {
                                    // The property received continues the
                                    // buildings given up (Reg. 1.199A-2(c)(3)).
                                    Some(exchange) => exchange.ubia(v)?,
                                    None => v.buildings.iter().try_fold(0, |basis, b| {
                                        let recovery = crate::historical_recovery::rental_building_recovery(
                                            &v.property_type, &v.location, b.in_service, b.commercial_1993_transition.as_ref(), false,
                                        )?;
                                        Ok::<_, (&'static str, &'static str)>(basis + if b.casualty.as_ref().is_none_or(|source| matches!(source.casualty.kind, crate::CasualtyKind::Damaged { retired: None, .. })) && crate::kernel::cost_recovery::historical::rental_elapsed_half_months(b.in_service, 2026) < recovery.half_months() {
                                            b.cost - b.land
                                        } else { 0 })
                                    })?,
                                },
                            })
                        })() {
                            Ok(value) => Some(value),
                            Err((key, _message)) => {
                                #[cfg(test)]
                                let message = _message;
                                crate::issue_localized!(
                                    r,
                                    &crate::interview::income_issue_context(e),
                                    &label,
                                    key,
                                    message
                                );
                                continue;
                            }
                        }
                    } else {
                        None
                    };
                    let qbi_activity = |income: i64| {
                        (!v.location.foreign()
                            && matches!(v.qbi_business, RentalBusiness::Yes { .. }))
                        .then(|| UniformBusinessIncome {
                            income,
                            regular_qbi: (!amt).then(|| crate::business_rules::QbiActivity {
                                source_wages_cents: None,
                                source_property_cents: None,
                                source_income_cents: None,
                                owner: Some(e.owner.owned()),
                                income,
                                wages: dollars(v.payroll.qbi_wages()),
                                property: dollars(
                                    ubia.expect(
                                        "regular QBI requires its original recovery source",
                                    ) + v
                                        .assets
                                        .iter()
                                        .map(|a| {
                                            if a.qbi_property_in_context(
                                                form_context.macrs().context(
                                                    e.owner.owned(),
                                                    Some(e.id),
                                                    a,
                                                ),
                                            ) {
                                                a.business_basis()
                                            } else {
                                                0
                                            }
                                        })
                                        .sum::<i64>(),
                                ),
                                active: matches!(
                                    v.qbi_business,
                                    RentalBusiness::Yes { material_participation: true, .. }
                                ),
                                service: Some(false),
                                group: v.qbi_aggregation_group.clone(),
                            }),
                        })
                    };
                    // The Form 4797 gain of a sold or exchanged rental, and whether
                    // its disposition is fully taxable to an unrelated person: only
                    // that releases the suspended losses (section 469(g)(1)(A)). A
                    // like-kind exchange's recognized gain is passive income of
                    // the activity, whose losses stay with the property received.
                    let source_gain = rental_source!(crate::asset_sale::rental_source(
                        form_context,
                        e.id,
                        amt,
                        Some(home_use)
                    ))
                    .map(|value| value.gain);
                    let releases_losses = exchange
                        .is_none_or(|exchange| exchange.releases_losses())
                        && professional_group_disposed(d, e.owner.owned());
                    // Form 6198 precedes section 469. Its prior disallowance is
                    // a current line 19 expense; the remaining shortfall carries.
                    let before_at_risk_cents = usage.net;
                    let raw_sale = rental_amount!(
                        e.id,
                        crate::asset_sale::rental_sale_amount(
                            form_context,
                            e.id,
                            property,
                            mid_quarter,
                            amt,
                            Some(home_use)
                        )
                    );
                    match rental_at_risk(v, e.id, raw_sale, amt, &mut usage, r) {
                        Ok(prior) => {
                            operating_prior_allowed.insert(e.id, prior);
                            if let Some(source) = foreign_sources.get_mut(&e.id) {
                                source.prior_at_risk_allowed_cents = prior.map_or(
                                    crate::kernel::source_fact::SourceFact::Unavailable,
                                    crate::kernel::source_fact::SourceFact::Reported,
                                );
                            }
                        }
                        Err(issue) => {
                            issue.report(e.id, r);
                            source_issue
                                .get_or_insert(crate::activity_risk::SourceIssue::new(e.id, issue));
                            continue;
                        }
                    }
                    let source = RentalOperatingSource {
                        source: e.id,
                        owner: e.owner.owned(),
                        alternative: amt,
                        before_at_risk_cents,
                        after_at_risk_cents: usage.net,
                        prior_at_risk_cents: usage.prior_at_risk_cents,
                        prior_passive_cents: prior_passive[usize::from(amt)],
                        prior_home_carry_cents: [
                            v.home_operating_carryover,
                            crate::rental_home_carry::total(v, amt)
                                .expect("completed rental source"),
                        ],
                        home_limited: usage.home,
                        excluded: usage.excluded,
                        property_disposition: !matches!(v.sale, crate::RentalSale::No),
                        filed_ordinary_dollars: None,
                    };
                    if let Some(foreign) = foreign_sources.get_mut(&e.id) {
                        foreign.operating =
                            crate::kernel::source_fact::SourceFact::Reported(source.clone());
                    }
                    let net_before_at_risk = source.before_at_risk_cents;
                    operating_sources.insert(e.id, source);
                    if usage.personal_fixed > exact::whole(0) {
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)] alloc::format!("{label}: eligible personal mortgage interest, points, insurance and property taxes flow to Schedule A automatically when you itemize. Do not enter them again."),
                            crate::Message::new("NOTE_RENTAL_PERSONAL_EXPENSE_SHARE", alloc::vec![crate::MessageArg::Text(label.clone())]),
                        ));
                    }
                    if !usage.home
                        && (v.home_operating_carryover != 0
                            || crate::rental_home_carry::total(v, amt) != Some(0))
                    {
                        // Worksheet 5-1 lines 7a and 7b: what the rent left
                        // unused still carries to 2027 in a year that is not a home.
                        carryover_rows(r, prefix, e.id, usage.carry);
                    }
                    if usage.home {
                        // Pub 527 ch. 5: renting a home is not a passive activity; the
                        // Worksheet 5-1 limit replaces the passive-loss rules.
                        let released = if !matches!(v.sale, crate::RentalSale::No)
                            && source_gain.is_some()
                            && releases_losses
                        {
                            dollars(prior_passive[usize::from(amt)])
                        } else {
                            0
                        };
                        let net = dollars(usage.net) - released;
                        if let Some(source) = foreign_sources.get_mut(&e.id) {
                            source.settle_pal(
                                dollars(usage.net),
                                released,
                                positive(-dollars(usage.net)) + released,
                            );
                        }
                        line(
                            r,
                            &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                            dollars(prior_passive[usize::from(amt)]) - released,
                            &["income.rental"],
                            PASSIVE,
                            msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                        );
                        if !amt && (net < 0 || released > 0) {
                            form_line(
                                r,
                                FormLine::new(TaxForm::ScheduleE, "22"),
                                &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                phase::cap(net, [0]) - if net >= 0 { released } else { 0 },
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_RENTAL_N_LINE22_NONPASSIVE_LOSS"),
                            );
                        }
                        // MAGI always uses the regular-tax amount.
                        let regular = if amt {
                            dollars(
                                rental_amount!(
                                    e.id,
                                    rental_use_core(
                                        v,
                                        false,
                                        mid_quarter,
                                        itemizing,
                                        home_fixed_allowed(
                                            crate::related_forms::RelatedFormContext::original(
                                                d,
                                                source_claims
                                            ),
                                            e.id,
                                            home_use.agi,
                                            false
                                        ),
                                        &expense_rows,
                                        Some((d, e.id, false, Some(home_use))),
                                        Some(form_context.macrs().as_ref()),
                                    )
                                    .map_err(|(key, _)| {
                                        crate::activity_risk::ProjectionIssue::Source(key)
                                    })
                                )
                                .net,
                            )
                        } else {
                            net
                        };
                        nonpassive_income += net;
                        if usage.net >= 0 && prior_passive[usize::from(amt)] == 0 {
                            health_work_income_residual_cents +=
                                usage.net - dollars(usage.net) * 100;
                        }
                        part_one += net;
                        magi += regular;
                        if let Some(activity) = qbi_activity(net) {
                            home_activities.push((e.id, activity));
                        }
                        carryover_rows(r, prefix, e.id, usage.carry);
                        if usage.excluded {
                            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!("{label}: used as a home and rented fewer than 15 days. The rent is not income and no rental expenses or depreciation are deducted."), crate::Message::new("NOTE_LABEL_USED_AS_A_HOME_AND_RENTED_FEWER_THAN_15_DAYS_THE_R_1B4E3107", alloc::vec![crate::MessageArg::Text(alloc::string::String::from(&label))])));
                        }
                    } else if re_professional
                        || business_participation == Some(true)
                        || (dollars(usage.net) > 0
                            && (v.property_type == RentalPropertyType::SelfRental
                                || nondepreciable(v)))
                    {
                        let net = dollars(usage.net);
                        let prior = dollars(prior_passive[usize::from(amt)]);
                        // The activity's prior unallowed loss offsets its own
                        // income first, for a former passive activity (section
                        // 469(f)(1)(A)) and for recharacterized self-rental or
                        // nondepreciable-property income alike: only "net
                        // passive income (including prior-year unallowed
                        // losses)" is nonpassive (Pub. 925; Reg. 1.469-2(f)(6),
                        // 1.469-2T(d)(1)). A fully taxable disposition releases
                        // the rest (section 469(g)).
                        let grouped = professional_groups.source(e.id);
                        let regular_grouped = regular_groups.source(e.id);
                        let released = if let Some(grouped) = grouped {
                            dollars(grouped.parts.prior_released.iter().sum())
                        } else if !matches!(v.sale, crate::RentalSale::No)
                            && source_gain.is_some()
                            && releases_losses
                        {
                            prior
                        } else {
                            phase::cap(prior, [positive(net)])
                        };
                        let nonpassive = net - released;
                        if prior <= released
                            && let Some(source) = foreign_sources.get_mut(&e.id)
                        {
                            source.settle_pal(net, prior, positive(-net) + released);
                        }
                        nonpassive_income += nonpassive;
                        if usage.net >= 0 && prior_passive[usize::from(amt)] == 0 {
                            health_work_income_residual_cents += usage.net - net * 100;
                        }
                        part_one += nonpassive;
                        nonpassive_rental_losses |= net < 0;
                        // MAGI always uses the regular-tax amount.
                        let regular = if amt {
                            dollars(
                                rental_amount!(
                                    e.id,
                                    rental_use_core(
                                        v,
                                        false,
                                        mid_quarter,
                                        itemizing,
                                        home_fixed_allowed(
                                            crate::related_forms::RelatedFormContext::original(
                                                d,
                                                source_claims
                                            ),
                                            e.id,
                                            home_use.agi,
                                            false
                                        ),
                                        &expense_rows,
                                        Some((d, e.id, false, Some(home_use))),
                                        Some(form_context.macrs().as_ref()),
                                    )
                                    .map_err(|(key, _)| {
                                        crate::activity_risk::ProjectionIssue::Source(key)
                                    })
                                )
                                .net,
                            )
                        } else {
                            net
                        };
                        magi += if regular_grouped.is_some() {
                            0
                        } else if business_participation.is_some() {
                            regular - phase::cap(dollars(prior_passive[0]), [positive(regular)])
                        } else {
                            positive(
                                regular
                                    - phase::cap(dollars(prior_passive[0]), [positive(regular)]),
                            )
                        };
                        // A prior loss the activity's own income or disposition
                        // absorbs in full never enters Form 8582 (i8582 "Former
                        // passive activities"; Pub. 925 "Recharacterization of
                        // Passive Income"): line 22 reports it here, and the
                        // pool reports only a remaining passive loss.
                        let own_release = if prior <= released { released } else { 0 };
                        if !amt && prior <= released && (net < 0 || own_release > 0) {
                            form_line(
                                r,
                                FormLine::new(TaxForm::ScheduleE, "22"),
                                &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                phase::cap(net, [0]) - own_release,
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_RENTAL_N_LINE22_NONPASSIVE_LOSS"),
                            );
                        }
                        if niit_safe_harbor
                            || v.property_type == RentalPropertyType::SelfRental
                            || (business_participation == Some(true)
                                && matches!(v.qbi_business, RentalBusiness::Yes { .. }))
                        {
                            rental_niit_adjustment -= regular
                                - regular_grouped.map_or(released, |grouped| {
                                    dollars(grouped.income_release.iter().sum())
                                });
                        }
                        if prior > 0 && prior <= released {
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                0,
                                &["income.rental"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS_2"),
                            );
                        } else if prior > 0 {
                            values.push(PassiveEntry {
                                amt_source: None,
                                source_cents: None,
                                id: e.id,
                                net: -(prior - released),
                                prior,
                                rental_allowance: business_participation.is_none() && allowance,
                                // The own-income release is already outside the pool.
                                // Only the remaining prior loss can be ineligible
                                // for the special allowance; the absorbed portion
                                // must not become apparent active profit.
                                ineligible_prior: phase::cap(
                                    dollars(ineligible),
                                    [prior - released],
                                ),
                                personal: false,
                                direct_business: false,
                                scorp: None,
                                partner: false,
                                source_group: None,
                                characters: None,
                                nonpassive,
                                prior_released: released,
                                uniform_business: qbi_activity(-(prior - released)),
                                disposition_gain: 0,
                                rental_partial: false,
                                estate_trust: false,
                                estate_activity: None,
                                farm_rental: false,
                                ptp: 0,
                                pass_through: None,
                                released: false,
                                items: None,
                            });
                        }
                        if let Some(activity) = qbi_activity(nonpassive) {
                            home_activities.push((e.id, activity));
                        }
                    } else if business_participation == Some(false) {
                        let prior = dollars(prior_passive[usize::from(amt)]);
                        let net = dollars(usage.net) - prior + source_gain.unwrap_or(0);
                        profits += positive(net);
                        values.push(PassiveEntry {
                            amt_source: None,
                            source_cents: None,
                            id: e.id,
                            net,
                            prior,
                            rental_allowance: false,
                            ineligible_prior: 0,
                            personal: false,
                            direct_business: false,
                            scorp: None,
                            partner: false,
                            source_group: None,
                            characters: None,
                            nonpassive: 0,
                            prior_released: 0,
                            uniform_business: qbi_activity(net),
                            disposition_gain: source_gain.unwrap_or(0),
                            rental_partial: rental_source!(
                                crate::asset_sale::kept_rental_partials(
                                    crate::related_forms::RelatedFormContext::original(
                                        d,
                                        source_claims
                                    ),
                                    e.id,
                                    amt,
                                    Some(home_use)
                                )
                            )
                            .is_some(),
                            estate_trust: false,
                            estate_activity: None,
                            farm_rental: false,
                            ptp: 0,
                            pass_through: None,
                            released: false,
                            items: None,
                        });
                    } else if let Some(gain) = source_gain
                        .filter(|_| releases_losses && !matches!(v.sale, crate::RentalSale::No))
                    {
                        // Section 469(g) (Pub 925, Form 8582 instructions): the
                        // whole property sold in a fully taxable sale. Its current
                        // and prior losses are combined with the sale gain; an
                        // overall loss is nonpassive and allowed in full, an
                        // overall gain is passive income that offsets other
                        // passive losses. Either way this activity's own losses
                        // are allowed.
                        // Section 469(g)(3), an installment sale of the entire
                        // interest: the overall loss (this year's net and the
                        // prior unallowed balance) is nonpassive in the ratio of
                        // the gain recognized this year to the total gain (i8582
                        // "Disposition of an Entire Interest"; Pub 925 "Installment
                        // sale of an entire interest"); the rest stays the
                        // property's unallowed loss, carried until the later
                        // payments recognize more gain. The recognized gain is
                        // passive income of the sale.
                        let installment_form = crate::asset_sale::rental_sale(v, amt)
                            .ok()
                            .flatten()
                            .and_then(|g| g.installment);
                        let own_net = dollars(usage.net);
                        let prior_balance = dollars(prior_passive[usize::from(amt)]);
                        if let Some(form) =
                            installment_form.as_ref().filter(|_| prior_balance > own_net)
                        {
                            let (released, remaining) =
                                installment_release(prior_balance - own_net, form);
                            // MAGI always uses the regular-tax amount.
                            let regular = if amt {
                                let regular_gain =
                                    rental_source!(crate::asset_sale::rental_source(
                                        crate::related_forms::RelatedFormContext::original(
                                            d,
                                            source_claims
                                        ),
                                        e.id,
                                        false,
                                        Some(home_use)
                                    ))
                                    .map_or(0, |value| value.gain);
                                let regular_net = dollars(
                                    rental_amount!(
                                        e.id,
                                        rental_use_core(
                                            v,
                                            false,
                                            mid_quarter,
                                            itemizing,
                                            home_fixed_allowed(
                                                crate::related_forms::RelatedFormContext::original(
                                                    d,
                                                    source_claims
                                                ),
                                                e.id,
                                                home_use.agi,
                                                false
                                            ),
                                            &expense_rows,
                                            Some((d, e.id, false, Some(home_use))),
                                            Some(form_context.macrs().as_ref()),
                                        )
                                        .map_err(
                                            |(key, _)| {
                                                crate::activity_risk::ProjectionIssue::Source(key)
                                            }
                                        )
                                    )
                                    .net,
                                );
                                let regular_prior = dollars(prior_passive[0]);
                                let regular_released = crate::asset_sale::rental_sale(v, false)
                                    .ok()
                                    .flatten()
                                    .and_then(|g| g.installment)
                                    .filter(|_| regular_prior > regular_net)
                                    .map_or(regular_prior - regular_net, |f| {
                                        installment_release(regular_prior - regular_net, &f).0
                                    });
                                regular_gain - regular_released
                            } else {
                                gain - released
                            };
                            nonpassive_income -= released;
                            part_one -= released;
                            released_rental_losses |= positive(-dollars(usage.net))
                                + dollars(prior_passive[usize::from(amt)])
                                > 0;
                            magi += regular;
                            if let Some(activity) = qbi_activity(-released) {
                                home_activities.push((e.id, activity));
                            }
                            if !amt && released > 0 {
                                form_line(
                                    r,
                                    FormLine::new(TaxForm::ScheduleE, "22"),
                                    &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                    -released,
                                    &["income.rental", "income.rental.sale"],
                                    PASSIVE,
                                    msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS"),
                                );
                            }
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                remaining,
                                &["income.rental", "income.rental.sale"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                            );
                            line(
                                r,
                                &alloc::format!("{prefix}rental_{}_sale_released_loss", e.id),
                                released,
                                &["income.rental", "income.rental.sale"],
                                PASSIVE,
                                msg!("EXPLAIN_0RENTAL_N_SALE_RELEASED_LOSS"),
                            );
                            profits += gain;
                            values.push(PassiveEntry {
                                amt_source: None,
                                source_cents: None,
                                id: e.id,
                                net: gain,
                                rental_allowance: allowance,
                                ineligible_prior: 0,
                                prior: 0,
                                personal: false,
                                direct_business: false,
                                scorp: None,
                                partner: false,
                                source_group: None,
                                characters: None,
                                items: None,
                                nonpassive: 0,
                                prior_released: 0,
                                uniform_business: qbi_activity(0),
                                disposition_gain: gain,
                                rental_partial: rental_source!(
                                    crate::asset_sale::kept_rental_partials(
                                        crate::related_forms::RelatedFormContext::original(
                                            d,
                                            source_claims
                                        ),
                                        e.id,
                                        amt,
                                        Some(home_use)
                                    )
                                )
                                .is_some(),
                                estate_trust: false,
                                estate_activity: None,
                                farm_rental: false,
                                ptp: 0,
                                pass_through: None,
                                released: false,
                            });
                        } else {
                            let net = dollars(usage.net) - dollars(prior_passive[usize::from(amt)]);
                            let combined = net + gain;
                            if combined < 0 {
                                let regular = if amt {
                                    let regular_gain =
                                        rental_source!(crate::asset_sale::rental_source(
                                            crate::related_forms::RelatedFormContext::original(
                                                d,
                                                source_claims
                                            ),
                                            e.id,
                                            false,
                                            Some(home_use)
                                        ))
                                        .map_or(0, |value| value.gain);
                                    dollars(
                                        rental_amount!(
                                            e.id,
                                            rental_use_core(
                                                v,
                                                false,
                                                mid_quarter,
                                                itemizing,
                                                home_fixed_allowed(
                                                    crate::related_forms::RelatedFormContext::original(
                                                        d,
                                                        source_claims
                                                    ),
                                                    e.id,
                                                    home_use.agi,
                                                    false
                                                ),
                                                &expense_rows,
                                                Some((d, e.id, false, Some(home_use))),
                                             Some(form_context.macrs().as_ref()),)
                                            .map_err(
                                                |(key, _)| {
                                                    crate::activity_risk::ProjectionIssue::Source(
                                                        key,
                                                    )
                                                }
                                            )
                                        )
                                        .net,
                                    ) - dollars(prior_passive[0])
                                        + regular_gain
                                } else {
                                    combined
                                };
                                nonpassive_income += net;
                                part_one += net;
                                magi += regular;
                                if let Some(activity) = qbi_activity(net) {
                                    home_activities.push((e.id, activity));
                                }
                                if !amt {
                                    form_line(
                                        r,
                                        FormLine::new(TaxForm::ScheduleE, "22"),
                                        &alloc::format!("rental_{}_line22_allowed_loss", e.id),
                                        // Part VIII (c): this year's loss and the
                                        // prior-year loss the sale releases, even
                                        // when the activity's own profit absorbs part.
                                        -(positive(-dollars(usage.net))
                                            + dollars(prior_passive[usize::from(amt)])),
                                        &["income.rental", "income.rental.sale"],
                                        PASSIVE,
                                        msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS"),
                                    );
                                }
                                line(
                                    r,
                                    &alloc::format!("{prefix}rental_{}_suspended_loss", e.id),
                                    0,
                                    &["income.rental", "income.rental.sale"],
                                    PASSIVE,
                                    msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS"),
                                );
                            } else {
                                profits += combined;
                                values.push(PassiveEntry {
                                    amt_source: None,
                                    source_cents: None,
                                    id: e.id,
                                    net: combined,
                                    rental_allowance: allowance,
                                    ineligible_prior: dollars(ineligible),
                                    prior: dollars(prior_passive[usize::from(amt)]),
                                    personal: false,
                                    direct_business: false,
                                    scorp: None,
                                    partner: false,
                                    source_group: None,
                                    characters: None,
                                    items: None,
                                    nonpassive: 0,
                                    prior_released: 0,
                                    uniform_business: qbi_activity(net),
                                    disposition_gain: gain,
                                    rental_partial: rental_source!(
                                        crate::asset_sale::kept_rental_partials(
                                            crate::related_forms::RelatedFormContext::original(
                                                d,
                                                source_claims
                                            ),
                                            e.id,
                                            amt,
                                            Some(home_use)
                                        )
                                    )
                                    .is_some(),
                                    released: false,
                                    estate_trust: false,
                                    estate_activity: None,
                                    farm_rental: false,
                                    ptp: 0,
                                    pass_through: None,
                                });
                            }
                        }
                    } else {
                        let gain = source_gain.unwrap_or(0);
                        let own = dollars(usage.net) - dollars(prior_passive[usize::from(amt)]);
                        let net = own + gain;
                        profits += positive(net);
                        values.push(PassiveEntry {
                            amt_source: None,
                            source_cents: None,
                            id: e.id,
                            net,
                            rental_allowance: allowance,
                            ineligible_prior: dollars(ineligible),
                            prior: dollars(prior_passive[usize::from(amt)]),
                            personal: false,
                            direct_business: false,
                            scorp: None,
                            partner: false,
                            source_group: None,
                            characters: None,
                            items: None,
                            nonpassive: 0,
                            prior_released: 0,
                            uniform_business: qbi_activity(own),
                            disposition_gain: gain,
                            rental_partial: rental_source!(
                                crate::asset_sale::kept_rental_partials(
                                    crate::related_forms::RelatedFormContext::original(
                                        d,
                                        source_claims
                                    ),
                                    e.id,
                                    amt,
                                    Some(home_use)
                                )
                            )
                            .is_some(),
                            estate_trust: false,
                            estate_activity: None,
                            farm_rental: false,
                            ptp: 0,
                            pass_through: None,
                            released: false,
                        });
                    }
                    if !amt {
                        if let Err(issue) = crate::federal_rules::form_4562_asset_class_lines(
                            d,
                            r,
                            e.id,
                            &v.assets,
                            "rental",
                            "income.rental.assets",
                            Some(form_context.macrs().as_ref()),
                        ) {
                            crate::section179_return::Issue::Prepared(issue).report(r);
                        }
                        let annual_classes = v.assets.iter().any(crate::BusinessAsset::nonmacrs);
                        let recovery_source = crate::nonmacrs::rental_recovery_source_with_macrs(
                            d,
                            e.id,
                            false,
                            Some(form_context.macrs().as_ref()),
                        );
                        for (index, asset) in v.assets.iter().enumerate() {
                            let Some([bonus, regular]) = crate::nonmacrs::rental_asset_parts(
                                asset,
                                index,
                                recovery_source.as_ref(),
                                mid_quarter,
                            ) else {
                                expense_output_unavailable(r, Some(e.id));
                                continue;
                            };
                            let qip = asset.qualified_improvement(form_context.macrs().context(
                                e.owner.owned(),
                                Some(e.id),
                                asset,
                            )) == Ok(true);
                            for (suffix, amount) in [("bonus", bonus), ("macrs", regular)] {
                                if amount == 0 && !qip {
                                    continue;
                                }
                                push_line(
                                    r,
                                    crate::depreciation::form_4562_line_in_context(
                                        asset,
                                        suffix,
                                        form_context.macrs().context(
                                            e.owner.owned(),
                                            Some(e.id),
                                            asset,
                                        ),
                                    )
                                    .map(|line| {
                                        if annual_classes {
                                            FormLine::subrow(TaxForm::Form4562, line)
                                        } else {
                                            FormLine::new(TaxForm::Form4562, line)
                                        }
                                    }),
                                    &alloc::format!("rental_{}_asset_{index}_{suffix}", e.id),
                                    amount,
                                    &["income.rental.assets"],
                                    RENTAL,
                                    msg!("EXPLAIN_RENTAL_ASSET_DEPRECIATION"),
                                );
                            }
                        }
                    }
                    if !amt && usage.energy_deduction > 0 {
                        push_line(
                            r,
                            Some(FormLine::subrow(TaxForm::ScheduleE, "19")),
                            &alloc::format!("rental_{}_energy_deduction", e.id),
                            usage.energy_deduction,
                            &["income.rental.energy_buildings", "income.rental.buildings.energy"],
                            "https://www.irs.gov/instructions/i7205",
                            msg!("EXPLAIN_RENTAL_ENERGY_DEDUCTION"),
                        );
                    }
                    let commercial = v.property_type.nonresidential();
                    // The entered expenses and depreciation are the rental
                    // share only without personal use (or with no personal
                    // days and no carried or rental-only amounts).
                    let no_personal_split = (v.home_operating_carryover == 0
                        && crate::rental_home_carry::total(v, amt) == Some(0))
                        && match v.personal_use {
                            crate::extended_model::RentalPersonalUse::No => true,
                            crate::extended_model::RentalPersonalUse::Yes {
                                personal_days,
                                direct_expenses,
                                ..
                            } => personal_days == 0 && direct_expenses == 0,
                        };
                    if amt {
                        line(
                            r,
                            &alloc::format!("{prefix}rental_{}_depreciation", e.id),
                            dollars(usage.depreciation),
                            &["income.rental.buildings"],
                            RENTAL,
                            msg!("EXPLAIN_0RENTAL_N_DEPRECIATION"),
                        );
                    } else {
                        // Form 4562 "Who Must File": a rental needs the form
                        // only for a building placed in service in 2026
                        // (it has no section 179, listed property or
                        // amortization); otherwise its depreciation is on
                        // Schedule E line 18 alone. Line 22 is the rental
                        // share Schedule E line 18 deducts.
                        // The excess basis of a like-kind exchange's property
                        // received is new property placed in service in 2026.
                        let replacement = exchange.map_or_else(Default::default, |exchange| {
                            exchange.replacement_depreciation(v, false).unwrap_or_default()
                        });
                        let form_4562 = v.buildings.iter().any(|b| b.in_service / 10000 == 2026)
                            || replacement.excess > 0;
                        push_line(
                            r,
                            form_4562.then(|| FormLine::new(TaxForm::Form4562, "22")),
                            &alloc::format!("rental_{}_depreciation", e.id),
                            usage.depreciation,
                            &["income.rental.buildings"],
                            RENTAL,
                            if commercial {
                                msg!("EXPLAIN_RENTAL_N_DEPRECIATION")
                            } else {
                                msg!("EXPLAIN_RENTAL_N_DEPRECIATION_2")
                            },
                        );
                        // Each building's GDS by service year: 2026 on Part III
                        // 19i (residential) or 19j (nonresidential), earlier
                        // years on line 17. With personal use the rental share
                        // is divided for the whole property, not per building,
                        // so no building row is listed.
                        if form_4562 && no_personal_split {
                            for (index, b) in v.buildings.iter().enumerate() {
                                let line = if b.in_service / 10000 < 2026 {
                                    "17"
                                } else if v.location.foreign() {
                                    if !commercial && b.in_service >= 20180101 {
                                        "20c"
                                    } else {
                                        "20d"
                                    }
                                } else if commercial {
                                    "19j"
                                } else {
                                    "19i"
                                };
                                push_line(
                                    r,
                                    Some(FormLine::new(TaxForm::Form4562, line)),
                                    &alloc::format!(
                                        "rental_{}_building_{index}_depreciation",
                                        e.id
                                    ),
                                    depreciation_for_year(
                                        b,
                                        false,
                                        (&v.property_type, &v.location),
                                        u32::try_from(d.tax_year).expect("validated return year"),
                                        v.sale.date(),
                                        crate::kernel::source_fact::SourceFact::Reported(
                                            crate::rental_energy::for_source(d, e.id, false)
                                                .reductions[index],
                                        ),
                                    )
                                    .unwrap_or(0),
                                    &["income.rental.buildings"],
                                    RENTAL,
                                    msg!("EXPLAIN_RENTAL_N_BUILDING_N_DEPRECIATION"),
                                );
                            }
                            // The property received: the exchanged basis
                            // continues the building given up (line 17), the
                            // excess basis is new (19i or 19j).
                            let replacement_commercial = exchange.is_some_and(|exchange| {
                                matches!(
                                    exchange.replacement.used,
                                    crate::ReplacementUse::Rental {
                                        kind: RentalPropertyType::Commercial,
                                        ..
                                    }
                                )
                            });
                            for (line, suffix, amount, explanation) in [
                                (
                                    "17",
                                    "carryover",
                                    replacement.carryover,
                                    msg!("EXPLAIN_RENTAL_REPLACEMENT_CARRYOVER_DEPRECIATION"),
                                ),
                                (
                                    if replacement_commercial { "19j" } else { "19i" },
                                    "excess",
                                    replacement.excess,
                                    msg!("EXPLAIN_RENTAL_REPLACEMENT_EXCESS_DEPRECIATION"),
                                ),
                            ] {
                                if amount != 0 {
                                    push_line(
                                        r,
                                        Some(FormLine::new(TaxForm::Form4562, line)),
                                        &alloc::format!(
                                            "rental_{}_replacement_{suffix}_depreciation",
                                            e.id
                                        ),
                                        amount,
                                        &["income.rental.sale"],
                                        RENTAL,
                                        explanation,
                                    );
                                }
                            }
                        }
                    }
                    // Schedule E lines 3, 18, 20 and 21 of a property: the
                    // rent and the rental share of depreciation, total
                    // expenses and income after any personal-use division
                    // and Worksheet 5-1 limit. A home rented fewer than 15
                    // days is not on Schedule E.
                    let schedule_e = !amt && !usage.excluded;
                    if schedule_e {
                        emit_lines(
                            r,
                            &mut (),
                            [
                                (
                                    "3",
                                    "line3_rents",
                                    v.rent + method_amounts(v).0 + usage.asset_recapture_cents,
                                    msg!("EXPLAIN_RENTAL_N_N_N"),
                                ),
                                (
                                    "18",
                                    "line18_depreciation",
                                    usage.depreciation,
                                    msg!("EXPLAIN_RENTAL_N_N_N_2"),
                                ),
                                (
                                    "20",
                                    "line20_total_expenses",
                                    v.rent + method_amounts(v).0 + usage.asset_recapture_cents
                                        - net_before_at_risk,
                                    msg!("EXPLAIN_RENTAL_N_N_N_3"),
                                ),
                                (
                                    "21",
                                    "line21_income",
                                    net_before_at_risk,
                                    msg!("EXPLAIN_RENTAL_N_N_N_4"),
                                ),
                            ],
                            |(line, suffix, amount, explanation), _, _| {
                                Some(calculation_row(
                                    Some(FormLine::new(TaxForm::ScheduleE, line)),
                                    &alloc::format!("rental_{}_{suffix}", e.id),
                                    LineValue::MoneyCents(amount),
                                    &["income.rental"],
                                    RENTAL,
                                    explanation,
                                ))
                            },
                        );
                    }
                    // Schedule E shows the allowed rental share after Worksheet 5-1.
                    // Allocate limited operating expenses in printed-line order.
                    if !amt && !usage.excluded {
                        let amounts = rental_expense_lines(
                            v,
                            &usage,
                            itemizing,
                            home_fixed_allowed(
                                crate::related_forms::RelatedFormContext::original(
                                    d,
                                    source_claims,
                                ),
                                e.id,
                                home_use.agi,
                                amt,
                            ),
                        );
                        let amounts = match amounts {
                            Ok(amounts) => amounts,
                            Err((key, _message)) => {
                                crate::issue_localized!(
                                    r,
                                    &crate::interview::income_issue_context(e),
                                    &label,
                                    key,
                                    _message,
                                );
                                crate::federal_rules::withhold_required_source_result(r);
                                continue;
                            }
                        };
                        for (index, amount) in amounts.into_iter().enumerate() {
                            let (suffix, explanation) = RentalExpenseCategory::LINES[index];
                            if amount != 0 {
                                push_line(
                                    r,
                                    Some(FormLine::new(TaxForm::ScheduleE, printed_line(suffix))),
                                    &alloc::format!("rental_{}_expense_{suffix}", e.id),
                                    amount,
                                    &["income.rental.expenses"],
                                    RENTAL,
                                    crate::Message::new(explanation, alloc::vec![]),
                                );
                            }
                        }
                        crate::method_change::report(
                            &v.method_changes,
                            r,
                            &alloc::format!("rental_{}", e.id),
                            "income.rental.method_changes",
                            &crate::interview::income_display_arg(e),
                            &crate::interview::income_issue_context(e),
                        );
                    }
                }
                Err((key, _message)) => {
                    if let Some(source) = foreign_sources.get_mut(&e.id) {
                        source.boundary = Some(key);
                    }
                    if [
                        crate::rental_home_carry::TOTAL,
                        crate::rental_home_carry::INVALID,
                        crate::rental_home_carry::OVERFLOW,
                    ]
                    .contains(&key)
                    {
                        source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                            e.id,
                            crate::activity_risk::ProjectionIssue::Source(key),
                        ));
                    }
                    let context = crate::activity_risk::ProjectionIssue::Source(key)
                        .context(crate::interview::income_issue_context(e));
                    // Regular and AMT evaluation share one correction per source,
                    // as do the downstream activity projections.
                    if !r.issues.iter().any(|issue| {
                        issue.context.identity() == context.identity() && issue.display.key == key
                    }) {
                        #[cfg(test)]
                        let message = _message;
                        crate::issue_localized!(r, &context, &label, key, message)
                    }
                }
            }
            *income_by_id.entry(e.id).or_default() += part_one - source_before;
            *nii_adjustment_by_id.entry(e.id).or_default() +=
                rental_niit_adjustment - source_niit_before;
        }
    }
    DirectOutput {
        magi,
        prior_home_casualty_by_owner,
        prior_home_casualty_by_id,
        source_issue,
        nonpassive_income,
        health_work_income_residual_cents,
        part_one,
        profits,
        rental_niit_adjustment,
        income_by_id,
        nii_adjustment_by_id,
        operating_sources,
        foreign_sources,
        operating_prior_allowed,
        values,
        home_activities,
        released_rental_losses,
        nonpassive_rental_losses,
    }
}
