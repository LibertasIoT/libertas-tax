// form_3800 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct ResearchNumeratorsIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) research_business_income: alloc::collections::BTreeMap<i64, i64>,
    pub(super) tips_deducted: alloc::collections::BTreeMap<i64, i64>,
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) excess_business_loss: i64,
    pub(super) capital_in: [i64; 2],
    pub(super) capital: i64,
    pub(super) short: i64,
    pub(super) long: i64,
    pub(super) additional: [i64; 3],
    pub(super) business_tips: Vec<(i64, i64)>,
    pub(super) qualified_tips: i64,
    pub(super) qualified_business_tips: i64,
    pub(super) qualified_overtime: i64,
    pub(super) charity_claims: crate::charity::ClassClaims,
    pub(super) use_itemized: bool,
    pub(super) charity: &'a crate::charity::Worksheet,
    pub(super) nol_run: &'a crate::nol::NolRun,
    pub(super) cash_standard: i64,
    pub(super) investment_interest_deduction: i64,
    pub(super) itemized: i64,
    pub(super) itemized_reduction: i64,
    pub(super) research_qbi_deductions: alloc::collections::BTreeMap<i64, i64>,
    pub(super) hsa: i64,
    pub(super) msa_deduction: i64,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) loan: i64,
    pub(super) alimony_paid: i64,
    pub(super) agi: i64,
    pub(super) qbi_ded: i64,
    pub(super) taxable: i64,
    pub(super) credit_entries: &'a [crate::general_business_credit::CreditEntry],
}

pub(super) fn research_numerators_phase(
    pass: &Pass,
    t: &Totals,
    i: ResearchNumeratorsIn<'_>,
) -> Result<Vec<(i64, i64)>, crate::activity_risk::SourceIssue> {
    let &Pass { d, context, allocated_entries, p, ded, pension, .. } = pass;
    let ResearchNumeratorsIn {
        rental,
        research_business_income,
        tips_deducted,
        business_half_se,
        health,
        sep,
        source_claims,
        property_character,
        excess_business_loss,
        capital_in,
        capital,
        short,
        long,
        additional,
        business_tips,
        qualified_tips,
        qualified_business_tips,
        qualified_overtime,
        charity_claims,
        use_itemized,
        charity,
        nol_run,
        cash_standard,
        investment_interest_deduction,
        itemized,
        itemized_reduction,
        research_qbi_deductions,
        hsa,
        msa_deduction,
        ira,
        loan,
        alimony_paid,
        agi,
        qbi_ded,
        taxable,
        credit_entries,
    } = i;
    // Section41(g), Reg1.53-3(d): accepted source income, source deductions,
    // then a proportionate share of the deductions unrelated to an activity.
    // The selector supplies identity only; the source owners supply amounts.
    Ok({
        let mut business_losses = rental.business_losses_by_id.clone();
        for (&id, &net) in &research_business_income {
            *business_losses.entry(id).or_default() +=
                positive(-(net + tips_deducted.get(&id).copied().unwrap_or(0)));
        }
        let mut sources = research_business_income;
        for (&id, &income) in &rental.income_by_id {
            *sources.entry(id).or_default() += income;
        }
        for (&id, items) in &t.research_items {
            let cut_cents = rental
                .k1_cuts
                .income_by_id
                .iter()
                .filter(|(source, _)| *source == id)
                .map(|(_, amount)| amount.get())
                .sum::<i64>();
            *sources.entry(id).or_default() += dollars(items.income + cut_cents)
                + rental
                    .k1_section1231
                    .iter()
                    .filter(|v| v.id == id)
                    .map(|v| v.net + v.suspended)
                    .sum::<i64>()
                - business_half_se.get(&id).copied().unwrap_or(0)
                - health.by_business.get(&id).copied().unwrap_or(0)
                - sep.by_business.get(&id).copied().unwrap_or(0);
        }
        for (id, property) in (crate::asset_sale::allowed_direct_sources_with_claims(
            d,
            rental,
            false,
            source_claims,
        ))? {
            *sources.entry(id).or_default() += dollars(property.gain);
        }
        for (id, property) in
            (crate::asset_sale::allowed_rental_sources(context.forms(d), rental, false))?
        {
            *sources.entry(id).or_default() += property.gain;
        }
        for (&id, items) in &t.research_items {
            let passive_cut: i64 = rental
                .k1_cuts
                .ordinary_by_id
                .iter()
                .filter(|(source, _)| *source == id)
                .map(|(_, amount)| amount.get())
                .sum();
            *business_losses.entry(id).or_default() +=
                positive(dollars(items.business_loss - passive_cut));
        }
        if property_character.as_ref().is_some_and(|v| v.net < 0) {
            for property in &rental.k1_section1231 {
                *business_losses.entry(property.id).or_default() +=
                    positive(-(property.net + property.suspended));
            }
        }
        for (id, property) in (crate::asset_sale::allowed_direct_sources_with_claims(
            d,
            rental,
            false,
            source_claims,
        ))?
        .into_iter()
        .chain(
            (crate::asset_sale::allowed_rental_sources(context.forms(d), rental, false).and_then(
                |values| {
                    values
                        .into_iter()
                        .map(|(id, value)| {
                            crate::community::PropertyCents::annual(value)
                                .map(|value| (id, value))
                                .map_err(|error| {
                                    crate::activity_risk::SourceIssue::new(id, error.into())
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()
                },
            ))?,
        ) {
            *business_losses.entry(id).or_default() += positive(-dollars(property.ordinary))
                + if property_character.as_ref().is_some_and(|v| v.net < 0) {
                    positive(-dollars(property.section1231))
                } else {
                    0
                };
        }
        let weights: Vec<_> = business_losses.values().copied().collect();
        for (id, withheld) in business_losses
            .into_keys()
            .zip(crate::kernel::ratio::allocate_remaining(excess_business_loss, &weights))
        {
            *sources.entry(id).or_default() += withheld;
        }
        for entry in allocated_entries {
            if let FederalIncome::Wage { data } = &entry.income
                && let Some(source) = crate::FederalWage::from(data).paying_scorporation
            {
                let data = crate::FederalWage::from(data);
                *sources.entry(source).or_default() += dollars(data.wages - data.medicaid_waiver);
            }
        }
        let mut capital_losses = alloc::collections::BTreeMap::<Option<i64>, i64>::new();
        for (&id, items) in &t.research_items {
            let cuts: i64 = rental
                .k1_cuts
                .capital_by_id
                .iter()
                .filter(|(source, _)| *source == id)
                .map(|(_, amount)| amount[0].get() + amount[1].get())
                .sum();
            capital_losses.insert(Some(id), positive(dollars(items.capital_loss - cuts)));
        }
        for (id, opening) in crate::retained_research::capital_losses(d) {
            *sources.entry(id).or_default() -= opening;
            *capital_losses.entry(Some(id)).or_default() += opening;
        }
        let total = positive(dollars(
            t.netted_losses - rental.k1_cuts.short.get() - rental.k1_cuts.long.get(),
        )) + capital_in.iter().sum::<i64>();
        let known: i64 = capital_losses.values().sum();
        capital_losses.insert(None, positive(total - known));
        let weights: Vec<_> = capital_losses.values().copied().collect();
        let disallowed = positive(capital - short - long);
        for (id, withheld) in capital_losses
            .into_keys()
            .zip(crate::kernel::ratio::allocate_remaining(disallowed, &weights))
        {
            if let Some(id) = id {
                *sources.entry(id).or_default() += withheld;
            }
        }
        // Sections224/225 deductions are tied to the actual earning source.
        // Business tips were already removed alongside their QBI projection.
        for (allowed, tips) in [(additional[0], true), (additional[1], false)] {
            let mut claims = alloc::collections::BTreeMap::<Option<i64>, i64>::new();
            for entry in allocated_entries {
                let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                if ![Some(&p.taxpayer), p.spouse.as_ref()][owner].is_some_and(|v| v.valid_ssn) {
                    continue;
                }
                let (id, amount) = match &entry.income {
                    FederalIncome::Wage { data } => {
                        let data = crate::FederalWage::from(data);
                        (
                            data.paying_scorporation,
                            if tips && data.work_deductions.tips_eligible() {
                                data.work_deductions.tips()
                            } else if !tips && data.work_deductions.overtime_eligible() {
                                data.work_deductions.overtime()
                            } else {
                                0
                            },
                        )
                    }
                    FederalIncome::Business { data } if !tips => {
                        (Some(entry.id), data.overtime.as_ref().map_or(0, |v| v.amount))
                    }
                    _ => continue,
                };
                *claims.entry(id).or_default() += dollars(amount);
            }
            if tips {
                for &(id, amount) in &business_tips {
                    *claims.entry(Some(id)).or_default() += amount;
                }
            }
            let total = if tips {
                dollars(qualified_tips) + qualified_business_tips
            } else {
                dollars(qualified_overtime)
            };
            let known: i64 = claims.values().sum();
            *claims.entry(None).or_default() += positive(total - known);
            let weights: Vec<_> = claims.values().copied().collect();
            for (id, part) in
                claims.into_keys().zip(crate::kernel::ratio::allocate_remaining(allowed, &weights))
            {
                if let Some(id) = id
                    && !(tips && tips_deducted.contains_key(&id))
                {
                    *sources.entry(id).or_default() -= part;
                }
            }
        }
        // Reg1.53-3(d)(1) applies the owner's limited class deduction to all
        // constituent items. The None bucket retains personal/other gifts.
        let mut attributed = alloc::collections::BTreeMap::<(bool, Option<i64>), i64>::new();
        for class in 0..charity_claims.current.len() {
            let allowed = if use_itemized {
                charity.classes[class].current_allowed
                    - charity.classes[class].current_floor
                    - nol_run.charity_class_reduction[class]
            } else if class == 1 {
                cash_standard
            } else {
                0
            };
            let mut claims = alloc::collections::BTreeMap::<Option<i64>, i64>::new();
            for (&id, items) in &t.research_items {
                if items.charity[class] > 0 {
                    claims.insert(Some(id), items.charity[class]);
                }
            }
            let known: i64 = claims.values().sum();
            claims.insert(None, positive(charity_claims.current[class] * 100 - known));
            let weights: Vec<_> = claims.values().copied().collect();
            for (id, part) in
                claims.into_keys().zip(crate::kernel::ratio::allocate_remaining(allowed, &weights))
            {
                *attributed.entry((id.is_some(), id)).or_default() += part;
            }
        }
        if use_itemized {
            for (source, deduction) in crate::retained_charity::deductions(
                &ded.charity_carryovers,
                charity,
                ded.basis_election,
            ) {
                *attributed.entry(source).or_default() += deduction;
            }
        }
        if use_itemized {
            let current_interest = t
                .research_items
                .iter()
                .map(|(&id, items)| (id, items.investment_interest))
                .collect();
            let interest_parts = crate::investment_interest::research_parts(
                ded.investment_interest.as_ref(),
                &current_interest,
                investment_interest_deduction,
            );
            let known_interest: i64 = interest_parts.values().sum();
            for (id, part) in interest_parts {
                *attributed.entry((true, Some(id))).or_default() += part;
            }
            // Unrelated portfolio interest is still attributable to an
            // investment activity, rather than a personal common deduction.
            *attributed.entry((true, None)).or_default() +=
                investment_interest_deduction - known_interest;
            // Section68 applies after Form4952/charitable limits. Preserve the
            // complete allowed itemized amount while sharing its reduction.
            attributed.remove(&(false, None));
            let known: i64 = attributed.values().sum();
            attributed.insert((false, None), positive(itemized - known));
            let weights: Vec<_> = attributed.values().copied().collect();
            let parts =
                crate::kernel::ratio::allocate_remaining(itemized - itemized_reduction, &weights);
            attributed = attributed.into_keys().zip(parts).collect();
        }
        let mut attributed_deductions = 0;
        for ((activity, id), amount) in attributed {
            if let Some(id) = id {
                *sources.entry(id).or_default() -= amount;
            }
            if activity {
                attributed_deductions += amount;
            }
        }
        for (id, deduction) in research_qbi_deductions {
            *sources.entry(id).or_default() -= deduction;
        }
        if let Some(allocation) = &nol_run.allocation {
            for (id, amount) in allocation.research_deductions(d) {
                *sources.entry(id).or_default() -= amount;
            }
        }
        // Personal adjustments enter the common-deduction pool. Activity
        // adjustments and QBI have already reduced the actual source amounts.
        let personal_adjustments =
            dollars(hsa) + msa_deduction + ira.deduction + pension.deduction + loan + alimony_paid;
        let denominator = agi + personal_adjustments
            - qbi_ded
            - attributed_deductions
            - additional[0]
            - additional[1];
        let common = positive(denominator - taxable);
        sources
            .into_iter()
            .filter(|(id, _)| {
                credit_entries.iter().any(|e| e.credit.research() && e.activity == Some(*id))
            })
            .map(|(id, net)| {
                (
                    id,
                    if denominator > 0 {
                        positive(net - fraction(common, positive(net), denominator))
                    } else {
                        0
                    },
                )
            })
            .collect()
    })
}

pub(super) struct Form3800In<'a> {
    pub(super) credit_entries: Vec<crate::general_business_credit::CreditEntry>,
    pub(super) credit_carrybacks: &'a [(crate::BusinessCredit, u32, i64, Option<i64>)],
    pub(super) research_numerators: Vec<(i64, i64)>,
    pub(super) taxable: i64,
    pub(super) separate_ratios: Option<[i64; 2]>,
    pub(super) regular_liability: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) amt: i64,
    pub(super) tentative: i64,
    pub(super) amt_credit: i64,
    pub(super) foreign_credit: i64,
    pub(super) other_nonrefundable_before_business: i64,
    pub(super) passive_capacity: crate::general_business_credit::PassiveCreditCapacity,
}

pub(super) fn form_3800_phase(pass: &Pass, r: &mut FederalResult, i: Form3800In<'_>) -> (i64, i64) {
    let &Pass { c, s, .. } = pass;
    let Form3800In {
        credit_entries,
        credit_carrybacks,
        research_numerators,
        taxable,
        separate_ratios,
        regular_liability,
        education_recapture_tax,
        amt,
        tentative,
        amt_credit,
        foreign_credit,
        other_nonrefundable_before_business,
        passive_capacity,
    } = i;
    crate::general_business_credit::calculate(
        c.tip_fica_carryforward.as_ref(),
        credit_entries,
        credit_carrybacks,
        crate::general_business_credit::TaxCapacity {
            research_numerators,
            taxable_income: taxable,
            separate_ratios,
            status: s.filing_status(),
            spouse_without_credits: c
                .business_credits
                .as_ref()
                .is_some_and(|history| history.spouse_without_credits),
            regular_tax_before_credits: regular_liability - education_recapture_tax,
            alternative_minimum_tax: amt,
            tentative_minimum_tax: positive(tentative - amt_credit),
            foreign_tax_credit: foreign_credit,
            other_nonrefundable_credits: other_nonrefundable_before_business,
            passive: passive_capacity,
        },
        r,
    )
}

pub(super) struct Form3800SourcesIn<'a> {
    pub(super) clean_vehicle: &'a mut crate::clean_vehicle::PersonalCredits,
    pub(super) foreign_magi: i64,
}

pub(super) struct Form3800SourcesOut {
    pub(super) credit_entries: Vec<crate::general_business_credit::CreditEntry>,
    pub(super) credit_carrybacks: Vec<(crate::BusinessCredit, u32, i64, Option<i64>)>,
    pub(super) investment: crate::investment_credit::Claims,
    pub(super) investment_recapture_tax: i64,
    pub(super) has_business_credit: bool,
}

pub(super) fn form_3800_sources_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form3800SourcesIn<'_>,
) -> Form3800SourcesOut {
    let &Pass { d, context, allocated_entries, c, s, .. } = pass;
    let Form3800SourcesIn { clean_vehicle, foreign_magi } = i;
    // Form 3800 Part III and IV entries: each business's own credit forms,
    // each K-1's supported credit codes (nonpassive when the owner materially
    // participates) and the entered carryforwards.
    let own_clean_vehicle_business: i64 =
        clean_vehicle.business.iter().map(|entry| entry.amount).sum();
    let mut credit_entries: Vec<crate::general_business_credit::CreditEntry> =
        core::mem::take(&mut clean_vehicle.business);
    let mut credit_carrybacks: Vec<(crate::BusinessCredit, u32, i64, Option<i64>)> = Vec::new();
    // The investment credits of business, farm and rental property (Form
    // 3468) and their recapture (Form 4255).
    let investment = crate::investment_credit::claims_with_claims(context.forms(d), r);
    let investment_recapture_tax = investment.recapture_tax;
    credit_entries.extend(investment.entries.iter().cloned());
    credit_entries.extend(crate::cooperative::credit_entries(d, r));
    for entry in allocated_entries {
        // The ordinary activity adapter has already applied the canonical
        // reduction once to entity basis and established nonpassive bindings.
        let graph_context = !d.setup.as_ref().is_some_and(crate::community::applies);
        let activity_properties = match &entry.income {
            FederalIncome::PartnershipActivities { data }
                if graph_context
                    && data.ordinary_source(false).is_ok()
                    && data.ordinary_source(true).is_ok() =>
            {
                Some(&data.entity.investment_property)
            }
            FederalIncome::ScorpActivities { data }
                if graph_context
                    && data.ordinary_source(false).is_ok()
                    && data.ordinary_source(true).is_ok() =>
            {
                Some(&data.entity.investment_property)
            }
            _ => None,
        };
        if let Some(items) = activity_properties {
            if let Some(native) =
                crate::investment_credit::k1_property_credit_sources(&entry.income)
            {
                for (item, source) in items.iter().zip(native) {
                    crate::investment_credit::report_k1_item(
                        r,
                        &alloc::format!(
                            "k1_activity_{}_investment_{}",
                            entry.id,
                            source.property_id
                        ),
                        item,
                    );
                    credit_entries.push(crate::general_business_credit::CreditEntry {
                        source_owner: Some(entry.owner.owned()),
                        conditional_origin: None,
                        transfer: 0,
                        elective: 0,
                        activity: None,
                        filer: crate::CarryFiler::SameFilers,
                        credit: item.section(),
                        year: 2026,
                        amount: item.current_year_credit(),
                        passive: None,
                    });
                }
            }
            continue;
        }
        let (credits, material, participation) = match &entry.income {
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                for credit in &data.reported_credits {
                    if let (Some(identity), Some(passive)) = (
                        crate::estate_trust::credit_identity(&credit.code),
                        crate::estate_trust::credit_passive(
                            &data.activities.activities,
                            &credit.activities,
                        ),
                    ) {
                        // The K-1 is the credit's source, as for a partnership: the
                        // research credit's section 41(g) limit uses the tax on
                        // this K-1's business income, and a code J first-four-year
                        // credit without facility dates has no extended carryback.
                        credit_entries.push(crate::general_business_credit::CreditEntry {
                            source_owner: Some(entry.owner.owned()),
                            conditional_origin:
                                crate::general_business_credit::ConditionalCarrybackOrigin::from_source(
                                    identity, None, None,
                                ),
                            transfer: 0,
                            elective: 0,
                            activity: Some(entry.id),
                            filer: crate::CarryFiler::SameFilers,
                            credit: identity,
                            year: 2026,
                            amount: dollars(credit.amount),
                            passive: passive.then(|| identity.worksheet()),
                        });
                    }
                }
                for (position, item) in data.investment_property.iter().enumerate() {
                    if let Some(passive) = crate::estate_trust::credit_passive(
                        &data.activities.activities,
                        &item.activities,
                    ) {
                        crate::investment_credit::report_k1_item(
                            r,
                            &alloc::format!("estate_trust_{}_investment_{position}", entry.id),
                            &item.property,
                        );
                        credit_entries.push(crate::general_business_credit::CreditEntry {
                            source_owner: Some(entry.owner.owned()),
                            conditional_origin: None,
                            transfer: 0,
                            elective: 0,
                            activity: None,
                            filer: crate::CarryFiler::SameFilers,
                            credit: item.property.section(),
                            year: 2026,
                            amount: item.property.current_year_credit(),
                            // i1041sk1 printed p.4 expressly preserves the special
                            // 8582-CR treatment of rental rehabilitation expenditures.
                            passive: passive.then_some(if item.property.rental_real_estate() {
                                crate::PassiveWorksheet::Rehabilitation
                            } else {
                                crate::PassiveWorksheet::Other
                            }),
                        });
                    } else {
                        note_once_scoped(
                            r,
                            Message::new("WARN_ESTATE_INVESTMENT_ACTIVITY_LEFT_OUT", alloc::vec![]),
                            crate::interview::income_issue_context(entry),
                        );
                    }
                }
                continue;
            }
            FederalIncome::Business { data } => {
                let mut current = Vec::new();
                if let Some(claim) = &data.employer_tip_fica_credit {
                    current.push((crate::BusinessCredit::EmployerTipFica, claim.credit()));
                    if claim.used_prior_year_carryback > 0 {
                        credit_carrybacks.push((
                            crate::BusinessCredit::EmployerTipFica,
                            2025,
                            dollars(claim.used_prior_year_carryback),
                            None,
                        ));
                    }
                }
                if let Some(claim) = &data.work_opportunity_credit {
                    current.push((crate::BusinessCredit::WorkOpportunity, claim.credit()));
                }
                if let Some(claim) = &data.small_employer_health_credit {
                    current.push((
                        crate::BusinessCredit::SmallEmployerHealth,
                        claim.credit(&data.payroll),
                    ));
                }
                if let Some(claim) = &data.paid_leave_credit {
                    current.push((crate::BusinessCredit::PaidLeave, claim.credit()));
                }
                if let Some(claim) = &data.retirement_plan_credit {
                    current.push((crate::BusinessCredit::PensionStartup, claim.startup_credit()));
                    current.push((
                        crate::BusinessCredit::AutoEnrollment,
                        claim.auto_enrollment_credit(),
                    ));
                }
                for (credit, amount) in current {
                    credit_entries.push(crate::general_business_credit::CreditEntry {
                        source_owner: Some(entry.owner.owned()),
                        conditional_origin: None,
                        transfer: 0,
                        elective: 0,
                        activity: None,
                        filer: crate::CarryFiler::SameFilers,
                        credit,
                        year: 2026,
                        amount,
                        passive: None,
                    });
                }
                continue;
            }
            FederalIncome::Partnership { data } => {
                (data.credits.clone(), data.material_for_losses(), data.rental_facts.participation)
            }
            FederalIncome::Scorp { data } => {
                let data = crate::ScorpK1::from(data);
                (data.credits.clone(), data.nonpassive(), data.rental_facts.participation)
            }
            _ => continue,
        };
        let items = match &entry.income {
            FederalIncome::Partnership { data } => data.investment_property.clone(),
            FederalIncome::Scorp { data } => crate::ScorpK1::from(data).investment_property,
            _ => Vec::new(),
        };
        if credits.is_empty() && items.is_empty() {
            continue;
        }
        let rental_nonpassive = participation
            .is_some_and(crate::K1RentalParticipation::professional)
            && d.professional(entry.owner.owned()).is_some();
        let housing_before1990 = match &entry.income {
            FederalIncome::Partnership { data } => data.housing_confirmed == Some(false),
            FederalIncome::Scorp { data } => {
                crate::ScorpK1::from(data).housing_confirmed == Some(false)
            }
            _ => false,
        };
        for credit in &credits {
            let identity = credit.credit();
            let bought = credit.code == crate::K1CreditCode::Transferred;
            let other_rental = credit.code == crate::K1CreditCode::OtherRental;
            let passive = if bought || other_rental {
                true
            } else if credit.code.rental() {
                !rental_nonpassive
            } else {
                material != crate::participation::Participation::Yes
            };
            let worksheet = if bought || other_rental {
                crate::PassiveWorksheet::Other
            } else if matches!(
                credit.code,
                crate::K1CreditCode::LowIncomeHousingJ5
                    | crate::K1CreditCode::LowIncomeHousingOther
            ) && housing_before1990
            {
                crate::PassiveWorksheet::LowIncomeHousingBefore1990
            } else if credit.code == crate::K1CreditCode::OtherRentalRealEstate {
                if identity == crate::BusinessCredit::Rehabilitation {
                    crate::PassiveWorksheet::Rehabilitation
                } else if matches!(
                    identity,
                    crate::BusinessCredit::LowIncomeHousing
                        | crate::BusinessCredit::LowIncomeHousingBefore2008
                ) {
                    crate::PassiveWorksheet::LowIncomeHousing
                } else if participation.is_some_and(crate::K1RentalParticipation::active) {
                    crate::PassiveWorksheet::RentalActive
                } else {
                    crate::PassiveWorksheet::Other
                }
            } else {
                identity.worksheet()
            };
            let amount = dollars(credit.amount);
            credit_entries.push(crate::general_business_credit::CreditEntry {
                source_owner: Some(entry.owner.owned()),
                conditional_origin:
                    crate::general_business_credit::ConditionalCarrybackOrigin::from_source(
                        identity,
                        credit.carbon_oxide_origin.as_ref(),
                        credit.renewable_facility_origin.as_ref(),
                    ),
                credit: identity,
                year: 2026,
                amount,
                passive: passive.then_some(worksheet),
                transfer: if bought { amount } else { 0 },
                elective: 0,
                activity: Some(entry.id),
                filer: crate::CarryFiler::SameFilers,
            });
        }
        // Form 3468 items of the K-1: a rehabilitation credit of the
        // rental real estate activity is Worksheet 2, any other item
        // follows the owner's participation in the business (Worksheet 4).
        for (position, item) in items.iter().enumerate() {
            let rental = item.rental_real_estate();
            let passive = if rental {
                !rental_nonpassive
            } else {
                material != crate::participation::Participation::Yes
            };
            let amount = item.current_year_credit();
            crate::investment_credit::report_k1_item(
                r,
                &alloc::format!(
                    "{}_{}_investment_{position}",
                    if matches!(entry.income, FederalIncome::Partnership { .. }) {
                        "partner"
                    } else {
                        "scorp"
                    },
                    entry.id
                ),
                item,
            );
            if amount > 0 {
                credit_entries.push(crate::general_business_credit::CreditEntry {
                    source_owner: Some(entry.owner.owned()),
                    conditional_origin: None,
                    transfer: 0,
                    elective: 0,
                    activity: None,
                    filer: crate::CarryFiler::SameFilers,
                    credit: item.section(),
                    year: 2026,
                    amount,
                    passive: passive.then_some(if rental {
                        crate::PassiveWorksheet::Rehabilitation
                    } else {
                        crate::PassiveWorksheet::Other
                    }),
                });
            }
        }
    }
    if let Some(history) = &c.business_credits {
        credit_entries.extend(crate::general_business_credit::entered_entries(
            context.forms(d),
            history,
            r,
        ));
        for item in &history.carryforwards {
            credit_entries.push(crate::general_business_credit::CreditEntry {
                source_owner: None,
                conditional_origin: None,
                transfer: 0,
                elective: 0,
                activity: item.research_source,
                filer: item.filer,
                credit: item.credit,
                year: item.origin_year,
                amount: dollars(item.amount),
                passive: item.passive,
            });
        }
        for item in &history.carrybacks {
            credit_carrybacks.push((
                item.credit,
                item.year,
                dollars(item.amount),
                item.research_source,
            ));
        }
    }
    // The property's own earlier-year credit joins the carryforwards from the
    // property row, cut by a recapture (Form 4255 column (i)); it is entered
    // once, on the property.
    credit_entries.extend(investment.carryforward_entries());
    let clean_vehicle_magi = foreign_magi
        + c.clean_vehicles.as_ref().map_or(0, |claim| dollars(claim.current_territory_addbacks));
    let clean_vehicle_eligible =
        crate::clean_vehicle::magi_eligible(s.filing_status(), clean_vehicle_magi, false)
            || c.clean_vehicles.as_ref().and_then(|claim| claim.prior_year.as_ref()).is_some_and(
                |prior| {
                    crate::clean_vehicle::magi_eligible(prior.status, dollars(prior.magi), false)
                },
            );
    crate::general_business_credit::cap_entries(&mut credit_entries, clean_vehicle_eligible, r);
    let current_clean_vehicle: Vec<_> = credit_entries
        .iter()
        .filter(|entry| entry.credit == crate::BusinessCredit::CleanVehicle && entry.year == 2026)
        .collect();
    if c.clean_vehicles.as_ref().is_some_and(|claim| {
        claim.vehicles.iter().any(|vehicle| {
            matches!(vehicle, crate::FederalCleanVehicle::New { business_use: Some(_), .. })
        })
    }) {
        // Direct partnership/S-corporation K-1 entries retain their accepted
        // activity ID. Own property and other current sources are not line 7.
        let k1_amount = current_clean_vehicle
            .iter()
            .filter(|credit| {
                credit.activity.is_some_and(|id| {
                    allocated_entries.iter().any(|entry| {
                        entry.id == id
                            && matches!(
                                entry.income,
                                FederalIncome::Partnership { .. }
                                    | FederalIncome::Scorp { .. }
                                    | FederalIncome::PartnershipActivities { .. }
                                    | FederalIncome::ScorpActivities { .. }
                            )
                    })
                })
            })
            .map(|entry| entry.amount)
            .sum();
        for (number, id, amount, explanation) in [
            (
                "7",
                "form_8936_k1_business_part",
                k1_amount,
                msg!("EXPLAIN_CLEAN_VEHICLE_K1_BUSINESS_TOTAL"),
            ),
            (
                "8",
                "form_8936_business_part",
                own_clean_vehicle_business + k1_amount,
                msg!("EXPLAIN_CLEAN_VEHICLE_BUSINESS_TOTAL"),
            ),
        ] {
            crate::line_emission::form_line(
                r,
                crate::FormLine::new(crate::TaxForm::Form8936, number),
                id,
                amount,
                &["credits.clean_vehicles", "credits.business_credits", "income"],
                "https://www.irs.gov/instructions/i8936",
                explanation,
            );
        }
    }
    let has_business_credit = c.tip_fica_carryforward.is_some() || !credit_entries.is_empty();
    Form3800SourcesOut {
        credit_entries,
        credit_carrybacks,
        investment,
        investment_recapture_tax,
        has_business_credit,
    }
}
