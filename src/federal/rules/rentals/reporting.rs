use super::*;

pub(crate) struct ReportingInput<'input, 'draft, 'source> {
    pub(crate) allocated_entries: &'input [alloc::borrow::Cow<'draft, crate::FederalIncomeEntry>],
    pub(crate) personal: &'input [(i64, i64, i64)],
    pub(crate) care: &'input crate::care::CareDeductions,
    pub(crate) section179: Option<&'input crate::Section179PreparedAllocation>,
    pub(crate) ptp_sources:
        &'input alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>,
    pub(crate) farm_rental_passive_property: &'input alloc::collections::BTreeSet<i64>,
    pub(crate) farm_rental_passive_sources:
        &'input alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertySource>,
    pub(crate) values: Vec<PassiveEntry<'source>>,
    pub(crate) personal_inputs: alloc::collections::BTreeMap<i64, PersonalRentalInputs>,
    pub(crate) magi: i64,
    pub(crate) personal_profit: i64,
    pub(crate) nonpassive_income: i64,
    pub(crate) health_work_income_residual_cents: i64,
    pub(crate) part_one: i64,
    pub(crate) profits: i64,
    pub(crate) rental_niit_adjustment: i64,
    pub(crate) ptp_income: i64,
    pub(crate) ptp_suspended: i64,
    pub(crate) estate_trust_business: i64,
    pub(crate) estate_trust_nonpassive: i64,
    pub(crate) prior_home_casualty_by_owner: [i64; 2],
    pub(crate) prior_home_casualty_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) income_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    pub(crate) foreign_sources:
        alloc::collections::BTreeMap<i64, crate::rental_foreign::RentalForeignSource>,
    pub(crate) operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    pub(crate) home_activities: Vec<(i64, UniformBusinessIncome)>,
    pub(crate) farm_rental_property: alloc::collections::BTreeMap<i64, crate::farm_land::LandGain>,
    pub(crate) farm_rental_property_rows:
        alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertyRows>,
    pub(crate) k1_ledgers: Vec<(i64, crate::k1_loss::LossLedger)>,
    pub(crate) has_scorp: bool,
    pub(crate) section179_allocations: core::cell::OnceCell<alloc::collections::BTreeMap<i64, i64>>,
}

pub(crate) struct ReportingOutput {
    pub(crate) result: RentalResult,
    pub(crate) post_pal: PostPalSources,
    pub(crate) personal_display_balances: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) professional_pass_through_rentals: i64,
}

pub(crate) fn report(
    context: &rental_setup::RentalContext<'_, '_>,
    input: ReportingInput<'_, '_, '_>,
    settlement: rental_settlement::SettlementOutput<'_>,
    source_issue: &mut Option<crate::activity_risk::SourceIssue>,
    r: &mut FederalResult,
) -> ReportingOutput {
    let form_context = context.form;
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let amt = context.amt;
    let home_use = context.home_use;
    let prefix = if amt { "amt_" } else { "" };
    let ReportingInput {
        allocated_entries,
        personal,
        care,
        section179,
        ptp_sources,
        farm_rental_passive_property,
        farm_rental_passive_sources,
        values,
        personal_inputs,
        magi,
        personal_profit,
        nonpassive_income,
        health_work_income_residual_cents,
        part_one,
        profits,
        rental_niit_adjustment,
        ptp_income,
        ptp_suspended,
        estate_trust_business,
        estate_trust_nonpassive,
        prior_home_casualty_by_owner,
        prior_home_casualty_by_id,
        income_by_id,
        nii_adjustment_by_id,
        operating_sources,
        foreign_sources,
        operating_prior_allowed,
        home_activities,
        farm_rental_property,
        farm_rental_property_rows,
        k1_ledgers,
        has_scorp,
        section179_allocations,
    } = input;
    let rental_settlement::SettlementOutput {
        inputs_complete,
        pool_complete,
        together,
        line5,
        line6,
        line7,
        allowance,
        net_cents,
        prior_cents,
        ineligible_cents,
        losses,
        suspended,
        allocation,
        special_allowance,
        combined_cents,
        health_ptc,
    } = settlement;
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

    if !amt
        && (values
            .iter()
            .enumerate()
            .any(|(index, entry)| !entry.released || net_cents[index] != 0)
            || personal_profit != 0)
    {
        let rows = |r: &mut FederalResult, printed: &str, id: &str, amount: i64, explanation| {
            form_line(
                r,
                FormLine::new(TaxForm::Form8582, printed),
                id,
                amount,
                &[
                    "income.nonbusiness_property",
                    "income.business",
                    "income.partnership",
                    "income.scorp",
                    "adjustments",
                ],
                PASSIVE,
                explanation,
            );
        };
        // Parts IV and V, column by column: (a) net income, (b) net loss and
        // (c) earlier years' unallowed losses (negative). A prior loss from a
        // year without active participation goes to Part V, not Part IV.
        let (mut part_four, mut part_five) = ([0i64; 3], [0i64; 3]);
        let column = |sums: &mut [i64; 3], current: i64, prior: i64| {
            sums[0] += positive(current);
            sums[1] -= positive(-current);
            sums[2] -= prior;
        };
        for (index, entry) in values.iter().enumerate() {
            if entry.released && net_cents[index] == 0 {
                continue;
            }
            let current = net_cents[index] + prior_cents[index];
            if entry.direct_business {
                let source =
                    &section179.expect("direct source is bound").passive_ordinary[&entry.id];
                column(&mut part_five, source.before_pal_cents, 0);
                for part in section179.unwrap().direct_property.get(&entry.id).into_iter().flatten()
                {
                    for amount in part.risk_allowed_cents {
                        column(&mut part_five, amount, 0);
                    }
                }
                continue;
            }
            // Income and loss reported on different forms stay in their own
            // columns even when they belong to the same activity (i8582 IV/V).
            let property = if entry.rental_partial {
                rental_source!(crate::asset_sale::kept_rental_partials(
                    form_context,
                    entry.id,
                    amt,
                    Some(home_use)
                ))
            } else {
                None
            };
            let property_net = property.as_ref().map_or(0, |v| v.gain * 100);
            let current = current - property_net;
            if let Some(property) = property {
                let sums = if entry.rental_allowance && !together {
                    &mut part_four
                } else {
                    &mut part_five
                };
                column(sums, property.ordinary * 100, 0);
                column(sums, property.section1231 * 100, 0);
            }
            if entry.rental_allowance && !together {
                column(&mut part_four, current, prior_cents[index] - ineligible_cents[index]);
                column(&mut part_five, 0, ineligible_cents[index]);
            } else {
                column(&mut part_five, current, prior_cents[index]);
            }
        }
        for (_, net, prior) in personal {
            if *net >= 0 {
                column(&mut part_five, *net + *prior, *prior);
            }
        }
        let part_four = part_four.map(dollars);
        let part_five = part_five.map(dollars);
        // These entered annual columns own the printed form's subtotals.
        // The actual cent pool and its source packet retain their separate
        // amounts; do not substitute a rounded raw net for this footer.
        let active: i64 = part_four.iter().sum();
        let combined = active + part_five.iter().sum::<i64>();
        for (printed, id, amount, explanation) in [
            ("1a", "form8582_line_1a", part_four[0], msg!("EXPLAIN_FORM8582_LINE_1A")),
            ("1b", "form8582_line_1b", part_four[1], msg!("EXPLAIN_FORM8582_LINE_1B")),
            ("1c", "form8582_line_1c", part_four[2], msg!("EXPLAIN_FORM8582_LINE_1C")),
            ("1d", "form8582_line_1d", active, msg!("EXPLAIN_FORM8582_LINE_1D")),
            ("2a", "form8582_line_2a", part_five[0], msg!("EXPLAIN_FORM8582_LINE_2A")),
            ("2b", "form8582_line_2b", part_five[1], msg!("EXPLAIN_FORM8582_LINE_2B")),
            ("2c", "form8582_line_2c", part_five[2], msg!("EXPLAIN_FORM8582_LINE_2C")),
            ("2d", "form8582_line_2d", combined - active, msg!("EXPLAIN_FORM8582_LINE_2D")),
        ] {
            // Married filing separately and lived with the spouse: Part IV
            // is not used (lines 1a-1d are blank).
            if together && printed.starts_with('1') {
                continue;
            }
            rows(r, printed, id, amount, explanation);
        }
        rows(r, "3", "form8582_line_3", combined, msg!("EXPLAIN_FORM8582_LINE_3"));
        // A net loss: Part II applies when the rental real estate with active
        // participation also shows a loss; lines 10 and 11 total the losses
        // allowed (the passive income offsets, then the special allowance).
        if combined < 0 {
            if active < 0 {
                for (printed, id, amount, explanation) in [
                    (
                        "4",
                        "form8582_line_4",
                        phase::cap(-active, [-combined]),
                        msg!("EXPLAIN_FORM8582_LINE_4"),
                    ),
                    ("5", "form8582_line_5", line5, msg!("EXPLAIN_FORM8582_LINE_5")),
                    ("6", "form8582_line_6", line6, msg!("EXPLAIN_FORM8582_LINE_6")),
                    ("7", "form8582_line_7", line7, msg!("EXPLAIN_FORM8582_LINE_7")),
                    ("8", "form8582_line_8", allowance, msg!("EXPLAIN_FORM8582_LINE_8")),
                    ("9", "form8582_line_9", special_allowance, msg!("EXPLAIN_FORM8582_LINE_9")),
                ] {
                    rows(r, printed, id, amount, explanation);
                }
            }
            // Line 10 is the income on lines 1a and 2a (Part IV and V column (a)).
            let line_10 = part_four[0] + part_five[0];
            rows(r, "10", "form8582_line_10", line_10, msg!("EXPLAIN_FORM8582_LINE_10"));
            rows(
                r,
                "11",
                "form8582_line_11",
                line_10 + special_allowance,
                msg!("EXPLAIN_FORM8582_LINE_11"),
            );
        }
    }
    let source_owners: alloc::collections::BTreeMap<_, _> =
        allocated_entries.iter().map(|entry| (entry.id, entry.owner)).collect();
    let mut post_pal = PostPalSources {
        alternative: amt,
        total_loss_cents: allocation.total_loss(),
        suspended_cents: allocation.held_loss(),
        allowed_loss_cents: allocation.allowed_loss(),
        special_allowance_cents: allocation.special_allowance(),
        sources: alloc::vec::Vec::new(),
    };
    let source_indices: alloc::vec::Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            // A PTP's own limit is not this common invocation. Do not return
            // a common-pool zero as its final allowed or suspended result.
            if (entry.partner || entry.pass_through.is_some())
                && allocated_entries.iter().any(|source| {
                    source.id == entry.id
                        && matches!(&source.income, FederalIncome::Partnership { data } if data.publicly_traded)
                })
            {
                return None;
            }
            let source_index = post_pal.sources.len();
            let net = net_cents[index];
            let held = allocation.suspended()[index];
            post_pal.sources.push(PostPalSourceAmounts {
                income: entry.id,
                owner: (source_owners[&entry.id]).owned(),
                component: entry.component(),
                k1_amt: None,
                common_pool: true,
                net_cents: net,
                prior_cents: prior_cents[index],
                suspended_cents: held,
                allowed_net_cents: net + held,
                characters: alloc::vec![post_pal_character(
                    if entry.direct_business {
                        PostPalCharacter::ScheduleC
                    } else if entry.personal {
                        PostPalCharacter::PersonalExpense
                    } else {
                        PostPalCharacter::ScheduleE
                    },
                    net,
                    held,
                )],
            });
            Some(source_index)
        })
        .collect();
    // Positive and zero personal sources have a genuine complete zero hold,
    // including recharacterized income excluded from the common profit.
    for entry in allocated_entries {
        let Some(input) = personal_inputs.get(&entry.id) else {
            continue;
        };
        let net = input.receipts_cents - input.expenses_cents - input.prior_cents;
        if net >= 0 {
            post_pal.sources.push(PostPalSourceAmounts {
                income: entry.id,
                owner: input.owner,
                component: PostPalComponent::PersonalProperty,
                k1_amt: None,
                common_pool: !input.investment_recharacterization || net == 0,
                net_cents: net,
                prior_cents: input.prior_cents,
                suspended_cents: 0,
                allowed_net_cents: net,
                characters: alloc::vec![post_pal_character(
                    PostPalCharacter::PersonalExpense,
                    net,
                    0,
                )],
            });
        }
    }
    let mut result = RentalResult {
        source_projection: None,
        macrs: Some(form_context.macrs()),
        source_issue: source_issue.clone(),
        pool_issue: None,
        section179_allocations: section179_allocations.into_inner(),
        business_losses_by_id: alloc::collections::BTreeMap::new(),
        ordinary_business_by_id: alloc::collections::BTreeMap::new(),
        qbi_source_ids: alloc::vec::Vec::new(),
        health_ptc,
        net_passive_income: dollars(combined_cents),
        special_allowance,
        modified_agi: positive(magi),
        inputs_complete,
        income_by_id,
        prior_home_casualty_by_owner,
        prior_home_casualty_by_id,
        settled_home_casualty_by_id: alloc::collections::BTreeMap::new(),
        home_use: home_use.clone(),
        nii_adjustment_by_id,
        personal_inputs,
        operating_sources,
        foreign_sources,
        farm_rental_property,
        direct_business_property: section179
            .map_or_else(Default::default, |selected| selected.direct_property.clone()),
        farm_rental_property_rows,
        operating_prior_allowed,
        post_pal: None,
        // Personal deductions belong to their Schedule1 owner. Sum actual
        // ScheduleE source allowances, rather than subtracting a common loss
        // then adding a separately rounded Personal residual back.
        income: profits
            - values
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.personal && !entry.direct_business)
                .map(|(index, _)| losses[index] - suspended[index])
                .sum::<i64>()
            + nonpassive_income
            + ptp_income,
        health_work_income_residual_cents,
        ptp_income,
        passive_loss: dollars(allocation.held_loss()) + ptp_suspended,
        scorp_niit_adjustment: 0,
        partner_niit_adjustment: 0,
        rental_niit_adjustment,
        estate_activities: alloc::vec::Vec::new(),
        estate_columns: [0; 4],
        estate_trust_business,
        estate_trust_nonpassive,
        estate_trust_niit_adjustment: -estate_trust_nonpassive,
        scorp_business: 0,
        partners: alloc::vec::Vec::new(),
        k1_amounts: alloc::vec::Vec::new(),
        k1_ledgers,
        partner_se: alloc::vec::Vec::new(),
        counterpart_partner_se: alloc::vec::Vec::new(),
        scorp_nonpassive: 0,
        pass_through_rental_nonpassive: 0,
        qbi: 0,
        // Filled from complete native allowed costs after source validation.
        personal_expense_adjustment: 0,
        part_one,
        farm_rental: 0,
        activities: alloc::vec::Vec::new(),
        qbi_loss_sources: alloc::vec::Vec::new(),
        scorp_suspensions: alloc::vec::Vec::new(),
        k1_section1231: alloc::vec::Vec::new(),
        scorp_qbi_ordinary_suspensions: alloc::vec::Vec::new(),
        k1_cuts: K1Cuts::default(),
        rental_partial_cuts: alloc::vec::Vec::new(),
    };
    // Preserve the existing annual display residuals; these are never raw
    // carry operands or a replacement for the same-invocation cent packet.
    let personal_display_balances: alloc::collections::BTreeMap<_, _> = values
        .iter()
        .zip(&suspended)
        .filter_map(|(entry, balance)| entry.personal.then_some((entry.id, *balance)))
        .collect();
    let mut professional_pass_through_rentals = 0;
    for (i, entry) in values.into_iter().enumerate() {
        let PassiveEntry {
            id,
            net,
            source_cents,
            amt_source,
            personal,
            direct_business,
            scorp,
            partner,
            source_group,
            characters,
            mut nonpassive,
            mut prior_released,
            uniform_business,
            disposition_gain,
            rental_partial,
            estate_trust,
            estate_activity,
            farm_rental,
            pass_through,
            prior,
            ptp,
            items,
            ..
        } = entry;
        if direct_business {
            let source = &section179.expect("direct source is bound").passive_ordinary[&id];
            let property =
                result.direct_business_property.get_mut(&id).expect("prepared property packet");
            let mut nets = alloc::vec![(None, source.before_pal_cents)];
            for part in property.iter() {
                for (class, amount) in part.risk_allowed_cents.iter().enumerate() {
                    nets.push((Some((part.original.id, class)), *amount));
                }
            }
            let held = allocation.suspended()[i];
            let suspended = rental_amount!(
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(held).map_err(
                    |_| crate::activity_risk::ProjectionIssue::Arithmetic(
                        crate::kernel::loss_limits::LossLimitError::InvalidAmount
                    )
                )
            );
            let settled = rental_amount!(
                id,
                crate::kernel::loss_limits::passive_characters(SourceFact::Reported(
                    crate::kernel::loss_limits::CharacterInput { nets: &nets, suspended }
                ))
                .map_err(crate::activity_risk::ProjectionIssue::from)
            );
            let SourceFact::Reported(settled) = settled else {
                unreachable!("reported source remains reported")
            };
            let operating = &settled[0];
            for component in settled.iter().skip(1) {
                let (key, class) = component.key.expect("property component identity");
                let part = property
                    .iter_mut()
                    .find(|part| part.original.id == key)
                    .expect("same invocation property identity");
                part.allowed_cents[class] = component.allowed_net();
            }
            rental_amount!(
                id,
                crate::business_risk::project_direct_business_property_rows(property)
            );
            let source_index = source_indices[i].expect("common direct source");
            post_pal.sources[source_index].characters = settled
                .iter()
                .map(|part| {
                    post_pal_character(
                        match part.key {
                            None => PostPalCharacter::ScheduleC,
                            Some((_, 0)) => PostPalCharacter::OrdinaryProperty,
                            Some(_) => PostPalCharacter::Section1231,
                        },
                        part.net,
                        part.held(),
                    )
                })
                .collect();
            for (suffix, amount, message) in [
                ("passive_current", net_cents[i], "EXPLAIN_BUSINESS_PASSIVE_CURRENT"),
                ("passive_allowed", operating.allowed_net(), "EXPLAIN_BUSINESS_PASSIVE_ALLOWED"),
                ("passive_suspended", held, "EXPLAIN_BUSINESS_PASSIVE_SUSPENDED"),
            ] {
                push_line(
                    r,
                    None,
                    &alloc::format!("{prefix}business_{id}_{suffix}"),
                    amount,
                    &["income.business", "rental_magi"],
                    PASSIVE,
                    Message::new(message, alloc::vec![]),
                );
            }
            continue;
        }
        // A publicly traded partnership's income and its still-suspended
        // loss, apart from the pool (section 469(k)).
        let bound_ptp = ptp_sources.get(&id);
        let ptp_character =
            pass_through.as_ref().map_or(crate::ptp_passive::PtpPalCharacter::Ordinary, |rental| {
                if rental.real_estate {
                    crate::ptp_passive::PtpPalCharacter::RentalRealEstate
                } else {
                    crate::ptp_passive::PtpPalCharacter::OtherRental
                }
            });
        let ptp_component = bound_ptp.and_then(|source| {
            source.components.iter().find(|part| part.original.character == ptp_character)
        });
        let ptp_gain = match bound_ptp {
            Some(_) if ptp_component.is_none() => None,
            Some(_) if pass_through.is_some() => Some((0, 0)),
            Some(source) => source.ordinary_net_cents().zip(source.ordinary_reported_net()),
            None => {
                let cents = source_cents
                    .as_ref()
                    .map_or(positive(ptp) * 100, |source| positive(source.ptp));
                Some((cents, dollars(cents)))
            }
        };
        let Some((ptp_gain_cents, ptp_gain)) = ptp_gain else {
            crate::issue_localized!(
                r,
                &crate::IssueContext::income_id(id),
                "K-1 source endpoint",
                "ISSUE_K1_RENTAL_PTP_SOURCE",
                "The rental activities of this publicly traded partnership need the same selected regular or AMT passive-loss source, including its checked section 179 allocation."
            );
            result.inputs_complete = false;
            result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                id,
                crate::activity_risk::ProjectionIssue::Source("ISSUE_K1_RENTAL_PTP_SOURCE"),
            ));
            continue;
        };
        let ptp_ordinary_held_cents = bound_ptp.map_or_else(
            || source_cents.as_ref().map_or(positive(-ptp) * 100, |source| positive(-source.ptp)),
            |_| ptp_component.map_or(0, |part| part.suspended_cents),
        );
        let ptp_ordinary_held = dollars(ptp_ordinary_held_cents);
        let mut carried = suspended[i] + ptp_ordinary_held;
        // The part of it the ordinary (Schedule E) character keeps once the
        // activity items take their share.
        let mut ordinary_suspended = suspended[i];
        let mut source_slot_held_cents = allocation.suspended()[i] + ptp_ordinary_held_cents;
        let mut exact_allowed_net_cents = if pass_through.is_some() {
            ptp_component.map_or(net_cents[i] + allocation.suspended()[i], |part| {
                part.allowed_passive_net_cents
            })
        } else {
            net_cents[i] + allocation.suspended()[i]
        };
        let reported_nonpassive = |cents| {
            let separately_stated =
                source_cents.as_ref().map_or([0; 4], |source| source.separate_nonpassive);
            dollars(cents - separately_stated.iter().sum::<i64>())
                + separately_stated.into_iter().map(dollars).sum::<i64>()
        };
        let mut exact_nonpassive_cents =
            source_cents.as_ref().map_or(nonpassive * 100, |source| source.nonpassive);
        let exact_disposition_cents =
            source_cents.as_ref().map_or(disposition_gain * 100, |source| source.disposition_gain);
        exact_allowed_net_cents -= exact_disposition_cents;
        let mut allowed_net = if source_cents.is_some() {
            dollars(exact_allowed_net_cents)
        } else if personal {
            dollars(net_cents[i] + allocation.suspended()[i])
        } else {
            positive(net) - (losses[i] - suspended[i])
        };
        // The sale gain is Form 4797 income; Schedule E keeps the rest.
        if source_cents.is_none() {
            allowed_net -= disposition_gain;
        }
        result.income -= disposition_gain;
        if disposition_gain != 0
            && let Some(source) = source_indices[i]
        {
            let character_nets = [net_cents[i] - exact_disposition_cents, exact_disposition_cents]
                .into_iter()
                .enumerate()
                .collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                    nets: &character_nets,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        allocation.suspended()[i],
                    )
                    .expect("native suspension is nonnegative"),
                }),
            )
            .expect("native disposition characters retain their common cent allowance") else {
                unreachable!("reported characters remain reported")
            };
            source_slot_held_cents = exact[0].held();
            post_pal.sources[source].characters =
                [PostPalCharacter::ScheduleE, PostPalCharacter::PropertyDisposition]
                    .into_iter()
                    .zip(&exact)
                    .map(|(character, amount)| post_pal_class_amounts(character, amount))
                    .collect();
        }
        if farm_rental && farm_rental_passive_property.contains(&id) {
            let property = result.farm_rental_property[&id];
            let ordinary = rental_source!(
                annual_dollars_to_working_cents(property.ordinary)
                    .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))
            );
            let section1231 = rental_source!(
                annual_dollars_to_working_cents(property.section1231)
                    .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))
            );
            let nets =
                [(0, net_cents[i] - exact_disposition_cents), (1, ordinary), (2, section1231)];
            let split = rental_source!(
                crate::kernel::loss_limits::passive_characters(SourceFact::Reported(
                    crate::kernel::loss_limits::CharacterInput {
                        nets: &nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                allocation.suspended()[i]
                            )
                            .expect("native suspension is nonnegative"),
                    }
                ))
                .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))
            );
            let SourceFact::Reported(split) = split else {
                unreachable!("reported characters remain reported");
            };
            source_slot_held_cents = split[0].held();
            ordinary_suspended = dollars(split[0].held());
            exact_allowed_net_cents = split[0].allowed_net();
            allowed_net = dollars(exact_allowed_net_cents);
            let mut allowed_property = property;
            allowed_property.ordinary = dollars(split[1].allowed_net());
            allowed_property.section1231 = dollars(split[2].allowed_net());
            allowed_property.gain = allowed_property.ordinary + allowed_property.section1231;
            result.income -= allowed_property.gain - disposition_gain;
            let source = &farm_rental_passive_sources[&id];
            let rows = rental_source!(
                source
                    .allowed_rows(&allowed_property, 0)
                    .map_err(|issue| crate::activity_risk::SourceIssue::new(id, issue))
            );
            result.farm_rental_property.insert(id, allowed_property);
            result.farm_rental_property_rows.insert(id, rows);
            if let Some(source) = source_indices[i] {
                post_pal.sources[source].characters = [
                    PostPalCharacter::ScheduleE,
                    PostPalCharacter::OrdinaryProperty,
                    PostPalCharacter::Section1231,
                ]
                .into_iter()
                .zip(&split)
                .map(|(character, amount)| post_pal_class_amounts(character, amount))
                .collect();
            }
        }
        if rental_partial
            && let Some(property) = rental_source!(crate::asset_sale::kept_rental_partials(
                form_context,
                id,
                amt,
                Some(home_use)
            ))
        {
            // Reuse Part IX's character allocator. Schedule E, Part II and
            // Part I of Form 4797 retain their own allowed loss; subtracting
            // the raw property loss from the pool would allow it twice.
            let nets = [net - disposition_gain, property.ordinary, property.section1231];
            let character_nets = nets.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(split) = crate::kernel::loss_limits::passive_characters(
                SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                    nets: &character_nets,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Dollars>::new(
                        suspended[i],
                    )
                    .expect("native suspension is nonnegative"),
                }),
            )
            .expect("rental partial characters reconcile with their passive activity") else {
                unreachable!("reported characters remain reported")
            };
            let split: alloc::vec::Vec<_> = split.iter().map(|amount| amount.held()).collect();
            if let Some(source) = source_indices[i] {
                let character_nets = nets
                    .map(|net| net * 100)
                    .into_iter()
                    .enumerate()
                    .collect::<alloc::vec::Vec<_>>();
                let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                    SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                allocation.suspended()[i],
                            )
                            .expect("native suspension is nonnegative"),
                    }),
                )
                .expect("native rental characters retain their common cent allowance") else {
                    unreachable!("reported characters remain reported")
                };
                source_slot_held_cents = exact[0].held();
                post_pal.sources[source].characters = [
                    PostPalCharacter::ScheduleE,
                    PostPalCharacter::OrdinaryProperty,
                    PostPalCharacter::Section1231,
                ]
                .into_iter()
                .zip(&exact)
                .map(|(character, amount)| post_pal_class_amounts(character, amount))
                .collect();
            }
            ordinary_suspended = split[0];
            let property_cut = split[1] + split[2];
            allowed_net -= property_cut;
            result.income -= property_cut;
            result.rental_partial_cuts.push((id, [split[1], split[2]]));
            if !amt {
                for (index, kind, unallowed_message, allowed_message) in [
                    (
                        0,
                        "ordinary",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        1,
                        "short_property",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        2,
                        "section1231",
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_UNALLOWED"),
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_ALLOWED"),
                    ),
                ] {
                    if nets[index] < 0 {
                        emit_lines(
                            r,
                            &mut (),
                            [
                                ("unallowed", "IX (d)", split[index], unallowed_message),
                                ("allowed", "IX (e)", -nets[index] - split[index], allowed_message),
                            ],
                            |(suffix, column, amount, explanation), _, _| {
                                Some(calculation_row(
                                    Some(FormLine::new(TaxForm::Form8582, column)),
                                    &alloc::format!("form8582_part_ix_{id}_{kind}_{suffix}"),
                                    LineValue::MoneyDollars(amount),
                                    &["income.rental", "rental_magi"],
                                    PASSIVE,
                                    explanation,
                                ))
                            },
                        );
                    }
                }
            }
        }
        if let Some(parts) = characters {
            let source_parts = if source_cents.is_some() {
                parts
            } else {
                crate::scorp::ActivityCharacters {
                    passive: parts.passive.map(|value| value * 100),
                    nonpassive: parts.nonpassive.map(|value| value * 100),
                    prior_released: parts.prior_released.map(|value| value * 100),
                }
            };
            let character_nets =
                source_parts.passive.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
            let SourceFact::Reported(class_amounts) =
                crate::kernel::loss_limits::passive_characters(SourceFact::Reported(
                    crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                allocation.suspended()[i],
                            )
                            .expect("native suspension is nonnegative"),
                    },
                ))
                .expect("validated character nets reconcile with the shared activity pool")
            else {
                unreachable!("reported characters remain reported")
            };
            let split: alloc::vec::Vec<_> =
                class_amounts.iter().map(|amount| dollars(amount.held())).collect();
            if let Some(source) = source_indices[i] {
                source_slot_held_cents = class_amounts[0].held();
                post_pal.sources[source].characters =
                    [PostPalCharacter::ScheduleE, PostPalCharacter::Section1231]
                        .into_iter()
                        .zip(&class_amounts)
                        .map(|(character, amount)| post_pal_class_amounts(character, amount))
                        .collect();
            }
            let parts = crate::scorp::ActivityCharacters {
                passive: source_parts.passive.map(dollars),
                nonpassive: source_parts.nonpassive.map(dollars),
                prior_released: source_parts.prior_released.map(dollars),
            };
            carried = split[0];
            ordinary_suspended = split[0];
            if partner {
                prior_released = parts.prior_released[0];
            }
            if scorp.is_some() {
                result.scorp_qbi_ordinary_suspensions.push((
                    id,
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        class_amounts[0].held(),
                    )
                    .expect("nonnegative ordinary source hold"),
                ));
            }
            if !amt
                && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(key, _)| *key == id)
            {
                let class = crate::k1_loss::K1LossClass::Section1231;
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + class_amounts[1].held(),
                )
                .expect("nonnegative source suspension");
            }
            // Part IX: an activity whose losses go to two forms or schedules
            // (ordinary on Schedule E, section 1231 on Form 4797): column (d)
            // the unallowed and column (e) the allowed part of each loss.
            if !amt {
                for (suffix, column, loss, unallowed, explanation) in [
                    (
                        "ordinary_unallowed",
                        "IX (d)",
                        parts.passive[0],
                        split[0],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED"),
                    ),
                    (
                        "ordinary_allowed",
                        "IX (e)",
                        parts.passive[0],
                        split[0],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED"),
                    ),
                    (
                        "section1231_unallowed",
                        "IX (d)",
                        parts.passive[1],
                        split[1],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_UNALLOWED"),
                    ),
                    (
                        "section1231_allowed",
                        "IX (e)",
                        parts.passive[1],
                        split[1],
                        msg!("EXPLAIN_FORM8582_PART_IX_N_SECTION1231_ALLOWED"),
                    ),
                ] {
                    if loss >= 0 {
                        continue;
                    }
                    form_line(
                        r,
                        FormLine::new(TaxForm::Form8582, column),
                        &match &pass_through {
                            Some(rental) => alloc::format!(
                                "form8582_part_ix_{id}_{}_{suffix}",
                                if rental.real_estate {
                                    "rental_real_estate"
                                } else {
                                    "other_rental"
                                }
                            ),
                            None => alloc::format!("form8582_part_ix_{id}_{suffix}"),
                        },
                        if suffix.ends_with("_unallowed") { unallowed } else { -loss - unallowed },
                        &["income.partnership", "income.scorp"],
                        PASSIVE,
                        explanation,
                    );
                }
            }
            let property_net = dollars(class_amounts[1].allowed_net() + source_parts.nonpassive[1]);
            exact_allowed_net_cents = class_amounts[0].allowed_net();
            allowed_net = dollars(exact_allowed_net_cents);
            let all_nonpassive_cents =
                source_cents.as_ref().map_or(nonpassive * 100, |source| source.nonpassive);
            let other_nonpassive =
                all_nonpassive_cents - source_parts.nonpassive.iter().sum::<i64>();
            exact_nonpassive_cents = source_parts.nonpassive[0] + other_nonpassive;
            nonpassive = reported_nonpassive(exact_nonpassive_cents);
            // Form 4797 character is determined later. Never feed property
            // amounts into Schedule E ordinary income, QBI or ordinary NIIT.
            result.income -= property_net - disposition_gain;
            result.add_property(
                id,
                class_amounts[1].allowed_net() + source_parts.nonpassive[1],
                class_amounts[1].held(),
                source_parts.nonpassive[1],
                pass_through.as_ref().map(|rental| K1RentalPropertyResult {
                    real_estate: rental.real_estate,
                    net_cents: class_amounts[1].allowed_net() + source_parts.nonpassive[1],
                    suspended_cents:
                        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                            class_amounts[1].held(),
                        )
                        .expect("nonnegative rental property source hold"),
                    nonpassive_cents: source_parts.nonpassive[1],
                    net: property_net,
                    suspended: split[1],
                    nonpassive: parts.nonpassive[1],
                    exclude_niit: rental.exclude_niit,
                    niit_disposition_loss: rental.niit_disposition_loss,
                    qbi_index: None,
                }),
            );
            if !amt
                && inputs_complete
                && let Some(source) = scorp
                && source.property_tracking()
                && source.qbi_left_out().is_none()
                && let Ok(basis) = source.basis_result()
            {
                let source = source.qbi_projection();
                source.property_qbi.report(
                    &source,
                    &basis,
                    id,
                    Some(
                        crate::kernel::carry::Amount::new(class_amounts[1].held())
                            .expect("nonnegative property source hold"),
                    ),
                    r,
                );
            }
            if !amt
                && inputs_complete
                && let Some(source) = scorp
                && let Some(statement) = &source.qbi_statement
                && source.qbi_left_out().is_none()
            {
                // This source-stage preview precedes current SSTB eligibility.
                // Do not route it into final QBI until property income is in AGI.
                match statement.activities(
                    source,
                    crate::kernel::carry::Amount::new(class_amounts[0].held())
                        .expect("nonnegative ordinary source hold"),
                    dollars(result.section179_allowed_cents(d, id)),
                ) {
                    Ok(items) => {
                        let current = items.first().map_or(0, |v| v.income);
                        let historical = -items.iter().skip(1).map(|v| v.income).sum::<i64>();
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)] alloc::format!("S-corporation document {id}, ordinary QBI source after shared activity limits: current ${current}; released historical qualified loss ${historical}. Property QBI is separate. Current amounts precede SSTB eligibility and remaining return limits; this is not a final QBI deduction."),
                            Message::new("NOTE_SCORP_ORDINARY_QBI_SOURCE", alloc::vec![
                                MessageArg::Number(id), MessageArg::usd_dollars(current), MessageArg::usd_dollars(historical),
                            ]),
                        ));
                    }
                    Err((key, _message)) => {
                        #[cfg(test)]
                        let message = _message;
                        crate::issue_localized!(
                            r,
                            &crate::IssueContext::income_id(id),
                            "S-corporation ordinary QBI source",
                            key,
                            message,
                        )
                    }
                }
            }
            #[cfg(test)]
            let tax_label = if amt { "AMT" } else { "regular" };
            #[cfg(test)]
            let entity = if partner { "Partnership" } else { "S-corporation" };
            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                "{entity} document {id}, {tax_label} passive character worksheet: ordinary net ${}, ordinary suspended loss ${}; section 1231 net ${property_net}, section 1231 suspended loss ${}. Nonpassive ordinary net ${nonpassive}; nonpassive section 1231 net ${}; prior loss absorbed by own income: ordinary ${}, section 1231 ${}. Intermediate allocation using resolved pool inputs; unresolved inputs may change these amounts. Section 1231 character and return-level limits are not final.", allowed_net + nonpassive, split[0], split[1], parts.nonpassive[1], parts.prior_released[0], parts.prior_released[1],
            ), Message::new(if partner { if amt { "NOTE_PARTNER_AMT_PASSIVE_CHARACTER" } else { "NOTE_PARTNER_REGULAR_PASSIVE_CHARACTER" } } else if amt { "NOTE_SCORP_AMT_PASSIVE_CHARACTER" } else { "NOTE_SCORP_REGULAR_PASSIVE_CHARACTER" }, alloc::vec![
                MessageArg::Number(id), MessageArg::usd_dollars(allowed_net + nonpassive),
                MessageArg::usd_dollars(split[0]), MessageArg::usd_dollars(property_net),
                MessageArg::usd_dollars(split[1]), MessageArg::usd_dollars(nonpassive),
                MessageArg::usd_dollars(parts.nonpassive[1]), MessageArg::usd_dollars(parts.prior_released[0]),
                MessageArg::usd_dollars(parts.prior_released[1]),
            ])));
        }
        // The activity items (code E, R and S) are part of the
        // activity's net. When the net is a loss the passive rules hold back,
        // the unallowed part is shared among the characters that have a net
        // loss in proportion to it (Form 8582 Parts VII to IX), and the
        // allowed part of each loss reaches its own form: code R Form 4797,
        // code S Schedule D. Nothing of them is Schedule E income.
        if let Some((parts, in_pool)) = items {
            let mut exact_item_cuts = None;
            if in_pool && let Some(source) = source_indices[i] {
                let slot = characters.map_or(net_cents[i], |parts| parts.passive[0]);
                let character_nets =
                    [slot - parts.net(), parts.ordinary, parts.short, parts.long, parts.debt]
                        .into_iter()
                        .enumerate()
                        .collect::<alloc::vec::Vec<_>>();
                let SourceFact::Reported(exact) = crate::kernel::loss_limits::passive_characters(
                    SourceFact::Reported(crate::kernel::loss_limits::CharacterInput {
                        nets: &character_nets,
                        suspended:
                            crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                source_slot_held_cents,
                            )
                            .expect("native suspension is nonnegative"),
                    }),
                )
                .expect("native activity items retain their common cent allowance") else {
                    unreachable!("reported characters remain reported")
                };
                exact_item_cuts =
                    Some([exact[0].held(), exact[1].held(), exact[2].held(), exact[3].held()]);
                source_slot_held_cents = exact[0].held();
                let mut classes: alloc::vec::Vec<_> = [
                    PostPalCharacter::ScheduleE,
                    PostPalCharacter::OrdinaryProperty,
                    PostPalCharacter::ShortCapital,
                    PostPalCharacter::LongCapital,
                    PostPalCharacter::DebtCancellation,
                ]
                .into_iter()
                .zip(&exact)
                .map(|(character, amount)| post_pal_class_amounts(character, amount))
                .collect();
                classes.extend(
                    post_pal.sources[source]
                        .characters
                        .iter()
                        .filter(|amount| amount.character != PostPalCharacter::ScheduleE)
                        .cloned(),
                );
                post_pal.sources[source].characters = classes;
            }
            let exact_cuts = if !in_pool {
                [0; 4]
            } else if let Some(cuts) = exact_item_cuts {
                cuts
            } else {
                // A PTP is intentionally absent from the common source packet.
                // Its own owner result remains the source of its character cut.
                let slot = characters.map_or(
                    net_cents[i] + source_cents.as_ref().map_or(ptp * 100, |source| source.ptp),
                    |c| c.passive[0],
                );
                parts
                    .cuts(slot - parts.net(), allocation.suspended()[i] + ptp_ordinary_held_cents)
                    .expect("the activity items reconcile with their owner passive result")
            };
            let exact_cut_sum = exact_cuts[1] + exact_cuts[2] + exact_cuts[3];
            let exact_items_allowed = parts.net() + exact_cut_sum;
            if in_pool {
                exact_allowed_net_cents -= exact_items_allowed;
                source_slot_held_cents = exact_cuts[0];
            } else {
                exact_nonpassive_cents -= parts.net();
            }
            let cuts = exact_cuts.map(dollars);
            let cut_sum = cuts[1] + cuts[2] + cuts[3];
            let items_allowed = dollars(exact_items_allowed);
            result.k1_cuts.ordinary =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.ordinary.get() + exact_cuts[1],
                )
                .expect("nonnegative source cut");
            result.k1_cuts.ordinary_by_id.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(exact_cuts[1])
                    .expect("nonnegative source cut"),
            ));
            result.k1_cuts.income_by_id.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(exact_cut_sum)
                    .expect("nonnegative source cut"),
            ));
            result.k1_cuts.short =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.short.get() + exact_cuts[2],
                )
                .expect("nonnegative source cut");
            result.k1_cuts.long = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                result.k1_cuts.long.get() + exact_cuts[3],
            )
            .expect("nonnegative source cut");
            result.k1_cuts.capital_by_id.push((
                id,
                [exact_cuts[2], exact_cuts[3]].map(|value| {
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(value)
                        .expect("nonnegative source cut")
                }),
            ));
            result.k1_cuts.passive =
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.k1_cuts.passive.get() + exact_cut_sum,
                )
                .expect("nonnegative source cut");
            result.income -= items_allowed;
            if in_pool {
                allowed_net = dollars(exact_allowed_net_cents);
                carried = cuts[0];
                ordinary_suspended =
                    dollars(phase::floor(exact_cuts[0] - ptp_ordinary_held_cents, 0));
            } else {
                nonpassive = reported_nonpassive(exact_nonpassive_cents);
            }
            if characters.is_some()
                && in_pool
                && let Some(last) =
                    result.scorp_qbi_ordinary_suspensions.iter_mut().rfind(|(k, _)| *k == id)
            {
                last.1 = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    source_slot_held_cents,
                )
                .expect("nonnegative ordinary source hold");
            }
            if !amt
                && exact_cut_sum != 0
                && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(k, _)| *k == id)
            {
                let class = crate::k1_loss::K1LossClass::OtherLoss;
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + exact_cut_sum,
                )
                .expect("nonnegative source suspension");
            }
            if !amt && in_pool && inputs_complete {
                // Part IX: the unallowed and allowed part of each loss of an
                // activity whose losses go to more than one form.
                let loss = positive(-(parts.ordinary + parts.short + parts.long));
                for (suffix, column, cut, explanation) in [
                    (
                        "items_unallowed",
                        "IX (d)",
                        cut_sum,
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ITEMS_UNALLOWED"),
                    ),
                    (
                        "items_allowed",
                        "IX (e)",
                        loss - cut_sum,
                        msg!("EXPLAIN_FORM8582_PART_IX_N_ITEMS_ALLOWED"),
                    ),
                ] {
                    if loss > 0 {
                        form_line(
                            r,
                            FormLine::new(TaxForm::Form8582, column),
                            &alloc::format!("form8582_part_ix_{id}_{suffix}"),
                            cut,
                            &["income.partnership", "income.scorp"],
                            PASSIVE,
                            explanation,
                        );
                    }
                }
            }
        }
        // Schedule K-1 (1065) box 12 / (1120-S) box 11 section 179 allowed on
        // Form 4562 line 12 (T2-29b): Schedule E line 28 column (j) of a
        // nonpassive activity, using this invocation's selected tax system.
        // A passive activity's deduction is already in its Form 8582 net:
        // what the pool allows reaches Schedule E with that net.
        let pooled179 = if (partner || scorp.is_some())
            && source_group.is_none()
            && bound_ptp.is_none()
            && crate::section179::passive_pass_through(d, id)
        {
            result.section179_allowed_cents(d, id)
        } else {
            0
        };
        // i8582 Worksheets 5 and 6: the unallowed loss is shared by the
        // activity's losses in proportion; the section 179 part keeps its
        // character and is not part of the ordinary (SE, QBI) hold.
        let held179 =
            section179_held(pooled179, net_cents[i], prior_cents[i], source_slot_held_cents);
        source_slot_held_cents -= held179;
        // The held section 179 part is its own carryover entry, apart from the
        // activity's ordinary unallowed loss; the allowed part is Schedule E
        // line 28 column (j).
        carried -= dollars(held179);
        if pooled179 != 0 && inputs_complete {
            let kind = if partner { "partnership" } else { "scorp" };
            let input =
                if partner { "income.partnership.section179" } else { "income.scorp.section179" };
            let allowed_id = alloc::format!("{prefix}{kind}_{id}_section179_passive_allowed");
            let allowed = dollars(pooled179 - held179);
            if amt {
                line(
                    r,
                    &allowed_id,
                    allowed,
                    &[input],
                    PASSIVE,
                    msg!("EXPLAIN_SECTION179_PASSIVE_ALLOWED"),
                );
            } else {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleE, "28 (j)"),
                    &allowed_id,
                    allowed,
                    &[input],
                    PASSIVE,
                    msg!("EXPLAIN_SECTION179_PASSIVE_ALLOWED"),
                );
            }
            line(
                r,
                &alloc::format!("{prefix}{kind}_{id}_section179_passive_unallowed"),
                dollars(held179),
                &[input],
                PASSIVE,
                msg!("EXPLAIN_SECTION179_PASSIVE_UNALLOWED"),
            );
        }
        if (partner || scorp.is_some())
            && source_group.is_none()
            && bound_ptp.is_none()
            && pooled179 == 0
        {
            let section179 = dollars(result.section179_allowed_cents(d, id));
            nonpassive -= section179;
            exact_nonpassive_cents -= result.section179_allowed_cents(d, id);
            result.income -= section179;
        }
        if partner && source_group.is_none() {
            // Form 2441 line24 is a partner-level nonpassive Schedule E
            // deduction, not a second change to the partnership's box1 or basis facts.
            let deductible = care.source(id);
            nonpassive = crate::kernel::carry::native_balance(nonpassive, deductible);
            exact_nonpassive_cents =
                crate::kernel::carry::native_balance(exact_nonpassive_cents, deductible * 100);
            result.income = crate::kernel::carry::native_balance(result.income, deductible);
            if deductible != 0 && !amt {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleE, "28 (j)"),
                    &alloc::format!("partnership_{id}_care_deduction"),
                    deductible,
                    &["credits.care.own_business", "income.partnership"],
                    "https://www.irs.gov/instructions/i2441",
                    msg!("EXPLAIN_CARE_SOURCE_DEDUCTION"),
                );
            }
        }
        if !personal {
            // Direct rentals' nonpassive result was retained at its original
            // part_one emission above. Only their remaining passive net is here.
            let amount = allowed_net
                + if partner
                    || scorp.is_some()
                    || source_group.is_some()
                    || pass_through.is_some()
                    || estate_trust
                {
                    nonpassive + ptp_gain
                } else {
                    0
                };
            *result.income_by_id.entry(id).or_default() += amount;
        }
        if partner || scorp.is_some() || source_group.is_some() || pass_through.is_some() {
            let amount = allowed_net + nonpassive + ptp_gain;
            let source_net_cents = source_cents.as_ref().and_then(|_| {
                exact_allowed_net_cents
                    .checked_add(exact_nonpassive_cents)
                    .and_then(|net| net.checked_add(ptp_gain_cents))
            });
            if source_cents.is_some() && source_net_cents.is_none() {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::income_id(id),
                    "K-1 source endpoint",
                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    "Review the document amounts; they must be within the supported input range."
                );
                result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                    id,
                    crate::activity_risk::ProjectionIssue::Source(
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    ),
                ));
                result.inputs_complete = false;
            }

            if let Some(source) = source_indices[i]
                && let (Some(classification), Some(cents)) = (amt_source, source_net_cents)
            {
                post_pal.sources[source].k1_amt = Some(K1AmtActivityAmounts {
                    source: classification,
                    schedule_e_cents: cents,
                    schedule_e_dollars: amount,
                });
            }

            match result.k1_amounts.iter_mut().find(|v| v.id == id) {
                Some(existing) => {
                    existing.net += amount;
                    existing.source_net_cents = match (existing.source_net_cents, source_net_cents)
                    {
                        (Some(left), Some(right)) => match left.checked_add(right) {
                            Some(total) => Some(total),
                            None => {
                                crate::issue_localized!(
                                    r,
                                    &crate::IssueContext::income_id(id),
                                    "K-1 source endpoint",
                                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                                    "Review the document amounts; they must be within the supported input range."
                                );
                                result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                                    id,
                                    crate::activity_risk::ProjectionIssue::Source(
                                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                                    ),
                                ));
                                result.inputs_complete = false;
                                None
                            }
                        },
                        _ => None,
                    };
                }
                None => result.k1_amounts.push(K1Amount { id, net: amount, source_net_cents }),
            }
            // The passive stage of the K-1's ledger: what the
            // passive rules hold back stays with the class that lost it,
            // including a publicly traded partnership's own suspended box 1
            // loss (section 469(k), `carried` adds it to the pool's share).
            if !amt && let Some((_, ledger)) = result.k1_ledgers.iter_mut().find(|(k, _)| *k == id)
            {
                let class = match &pass_through {
                    Some(rental) if rental.real_estate => {
                        crate::k1_loss::K1LossClass::RentalRealEstate
                    }
                    Some(_) => crate::k1_loss::K1LossClass::OtherRental,
                    None => crate::k1_loss::K1LossClass::Ordinary,
                };
                ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[class].get() + source_slot_held_cents,
                )
                .expect("nonnegative source suspension");
                let section179_class = crate::k1_loss::K1LossClass::Section179;
                ledger.passive_suspended[section179_class] = crate::kernel::carry::Amount::new(
                    ledger.passive_suspended[section179_class].get() + held179,
                )
                .expect("nonnegative source suspension");
                if let Some(source) = bound_ptp.filter(|_| partner) {
                    let class = crate::k1_loss::K1LossClass::Section179;
                    let held_cents = source
                        .components
                        .iter()
                        .filter(|part| {
                            part.original.character
                                == crate::ptp_passive::PtpPalCharacter::Section179
                        })
                        .map(|part| part.suspended_cents)
                        .sum::<i64>();
                    ledger.passive_suspended[class] = crate::kernel::carry::Amount::new(
                        ledger.passive_suspended[class].get() + held_cents,
                    )
                    .expect("nonnegative source suspension");
                }
            }
        }
        if partner || scorp.is_some() || source_group.is_some() || estate_trust {
            *result.business_losses_by_id.entry(id).or_default() +=
                positive(-(allowed_net + nonpassive + ptp_gain));
        }
        if partner {
            *result.ordinary_business_by_id.entry(id).or_default() +=
                allowed_net + nonpassive + ptp_gain;
            if let Some(value) = result.partners.iter_mut().find(|value| value.id == id) {
                value.net += allowed_net + nonpassive + ptp_gain;
                value.nonpassive += nonpassive;
                value.suspended = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    value.suspended.get() + source_slot_held_cents,
                )
                .expect("nonnegative source hold");
                value.prior_released += prior_released;
            } else {
                result.partners.push(PartnerPassiveResult {
                    id,
                    net: allowed_net + nonpassive + ptp_gain,
                    nonpassive,
                    suspended: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        source_slot_held_cents,
                    )
                    .expect("nonnegative source hold"),
                    prior_released,
                });
            }
            if source_group.is_some() && !amt {
                result.partner_niit_adjustment -= nonpassive;
                *result.nii_adjustment_by_id.entry(id).or_default() -= nonpassive;
            }
            if inputs_complete {
                let source_id = source_group.map_or_else(
                    || alloc::format!("partnership_{id}"),
                    |group| alloc::format!("partnership_{id}_group_{group}"),
                );
                line(
                    r,
                    &alloc::format!("{prefix}{source_id}_passive_suspended"),
                    carried,
                    &["income.partnership"],
                    PASSIVE,
                    msg!("EXPLAIN_0PARTNERSHIP_N_PASSIVE_SUSPENDED"),
                );
                line(
                    r,
                    &alloc::format!("{prefix}{source_id}_activity_net"),
                    allowed_net + nonpassive + ptp_gain,
                    &["income.partnership"],
                    PASSIVE,
                    msg!("EXPLAIN_0PARTNERSHIP_N_ACTIVITY_NET"),
                );
            }
        }
        if !partner && let Some(group) = source_group {
            if inputs_complete {
                for (stage, amount, explanation) in [
                    (
                        "passive_suspended",
                        carried,
                        msg!("EXPLAIN_SCORP_ACTIVITY_PASSIVE_SUSPENDED"),
                    ),
                    ("activity_net", allowed_net + nonpassive, msg!("EXPLAIN_SCORP_ACTIVITY_NET")),
                ] {
                    line(
                        r,
                        &alloc::format!("{prefix}scorp_{id}_group_{group}_{stage}"),
                        amount,
                        &["income.scorp.activities"],
                        PASSIVE,
                        explanation,
                    );
                }
            }

            result.scorp_business += allowed_net + nonpassive;
            result.scorp_nonpassive += nonpassive;
            *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            if !amt {
                result.scorp_niit_adjustment -= nonpassive;
                *result.nii_adjustment_by_id.entry(id).or_default() -= nonpassive;
            }
        }
        if let Some(source) = scorp {
            result.scorp_business += allowed_net + nonpassive;
            *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            result.scorp_nonpassive += nonpassive;
            if !amt {
                // Form 8960 line 4b reverses nonpassive ordinary net, including
                // the own-income release of a former passive loss. Losses released
                // against OTHER passive income remain in allowed_net.
                let adjustment = match source.trade_kind {
                    crate::ScorpTradeKind::Ordinary => -nonpassive,
                    crate::ScorpTradeKind::FinancialTrading => 0,
                };
                result.scorp_niit_adjustment += adjustment;
                *result.nii_adjustment_by_id.entry(id).or_default() += adjustment;
                if pool_complete {
                    let nii = allowed_net + nonpassive + adjustment;
                    r.review_notes.push(crate::federal_model::ReviewNote::localized(
                        #[cfg(test)] alloc::format!("S-corporation document {id}, investment income after activity limits: ${nii}; Form 8960 adjustment: ${adjustment}. Ordinary nonpassive business net is excluded; financial-trading net remains included. Additional return-level limits remain separate."),
                        Message::new("NOTE_SCORP_INVESTMENT_INCOME", alloc::vec![
                            MessageArg::Number(id), MessageArg::usd_dollars(nii), MessageArg::usd_dollars(adjustment),
                        ]),
                    ));
                }
            }
        }
        // Carry resolved source inputs independently of unrelated return issues.
        if inputs_complete
            && characters.is_none()
            && scorp.is_some_and(|source| source.loss_tracking())
        {
            result.scorp_suspensions.push((
                id,
                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    source_slot_held_cents,
                )
                .expect("nonnegative ordinary source hold"),
            ));
        }
        if has_scorp && pool_complete && !personal && characters.is_none() && pass_through.is_none()
        {
            if let Some(source) = scorp {
                #[cfg(test)]
                let tax_label = if amt { "AMT S-corporation" } else { "S-corporation" };
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)] alloc::format!("{tax_label} document {id}, nonpassive net after own prior passive loss: ${nonpassive}; prior passive loss absorbed by own income: ${prior_released}. Nonpassive amounts do not offset other passive losses; the allowance uses regular-tax MAGI."),
                    Message::new(if amt { "NOTE_SCORP_AMT_NONPASSIVE_NET" } else { "NOTE_SCORP_REGULAR_NONPASSIVE_NET" }, alloc::vec![
                        MessageArg::Number(id), MessageArg::usd_dollars(nonpassive), MessageArg::usd_dollars(prior_released),
                    ]),
                ));
                if source.loss_reported_with_basis() {
                    let source = source.qbi_projection();
                    source.loss_qbi.report_passive(
                        &source,
                        id,
                        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                            source_slot_held_cents,
                        )
                        .expect("nonnegative ordinary source hold"),
                        r,
                    );
                }
            }
            #[cfg(test)]
            let tax_label = if amt { "AMT passive activity" } else { "Passive activity" };
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("{tax_label} {id}, net after passive limits: ${allowed_net}; remaining passive loss: ${}. Amount after activity limits; return-level limits apply separately.", suspended[i]),
                Message::new(if amt { "NOTE_AMT_PASSIVE_ACTIVITY_NET" } else { "NOTE_REGULAR_PASSIVE_ACTIVITY_NET" }, alloc::vec![
                    MessageArg::Number(id), MessageArg::usd_dollars(allowed_net), MessageArg::usd_dollars(suspended[i]),
                ]),
            ));
        }
        if let Some(business) = uniform_business {
            // Cooperative distributions above their box 8 qualified items are
            // not QBI, on a farm rental as on a farm (Form 1099-PATR box 8).
            let nonqualified = if farm_rental {
                match crate::farm_rental::nonqualified_cooperative_with_claims(d, id, source_claims)
                {
                    Ok(amount) => amount,
                    Err(issue) => {
                        let key = match &issue {
                            crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key)
                            | crate::Section179PreparedIssue::InvalidSource(_, key) => *key,
                            _ => "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE",
                        };
                        crate::section179_return::Issue::Prepared(issue).report(r);
                        source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                            id,
                            crate::activity_risk::ProjectionIssue::Source(key),
                        ));
                        continue;
                    }
                }
            } else {
                0
            };
            let income = allowed_net - nonqualified;
            result.qbi += income;
            *result.business_losses_by_id.entry(id).or_default() += positive(-income);
            if income < 0 {
                result.qbi_loss_sources.push(id);
            }
            if let Some(mut activity) = business.regular_qbi {
                activity.income = income;
                activity.owner = d.income.iter().find(|e| e.id == id).map(|e| e.owner.owned());
                result.qbi_source_ids.push(id);
                result.activities.push(activity);
            }
        }
        if let Some((parent, activity)) = estate_activity {
            let current = dollars(activity.net(amt));
            let source_prior = dollars(activity.prior(amt));
            let gross_income = positive(dollars(activity.income));
            // Box 9 deductions are separate from the fiduciary's signed
            // source income. Keep both Schedule E columns visible; the
            // net already used by the common owner fixes rounded footing.
            let current_deductions = gross_income - current;
            let allowed_loss = current_deductions + source_prior - suspended[i];
            if activity.kind == crate::EstateActivityKind::Business {
                result.estate_trust_business += allowed_net + nonpassive;
                *result.ordinary_business_by_id.entry(id).or_default() += allowed_net + nonpassive;
            }
            result.estate_activities.push(crate::estate_trust::EstateActivityResult {
                source: parent,
                activity: activity.id,
                current,
                prior: source_prior,
                passive: allowed_net,
                nonpassive,
                suspended: suspended[i],
                prior_released,
            });
            if !amt && inputs_complete {
                let material = activity.material();
                let nonpassive_loss =
                    if material { current_deductions + prior_released } else { 0 };
                result.estate_columns[0] += allowed_loss - nonpassive_loss;
                result.estate_columns[1] += if material { 0 } else { gross_income };
                result.estate_columns[2] += nonpassive_loss;
                result.estate_columns[3] += if material { gross_income } else { 0 };
                // Former-passive losses released against this activity's own
                // nonpassive profit are nonpassive under section 469(f).
                // Any further loss allowed by the common pool stays passive.
                for (column, amount) in [
                    (if material { "33f" } else { "33d" }, gross_income),
                    ("33e", nonpassive_loss),
                    ("33c", allowed_loss - nonpassive_loss),
                ] {
                    if amount != 0 {
                        form_line(
                            r,
                            FormLine::new(TaxForm::ScheduleE, column),
                            &alloc::format!(
                                "estate_trust_{parent}_activity_{}_{}",
                                activity.id,
                                column
                            ),
                            amount,
                            &["income.estate_trust_portfolio.activities"],
                            PASSIVE,
                            crate::Message::new(
                                "EXPLAIN_ESTATE_ACTIVITY_COLUMN",
                                alloc::vec![
                                    crate::MessageArg::Number(parent),
                                    crate::MessageArg::Text(activity.name.clone()),
                                    crate::MessageArg::Text(column.into()),
                                ],
                            ),
                        );
                    }
                }
            }
            line(
                r,
                &alloc::format!("{prefix}estate_trust_{parent}_activity_{}_suspended", activity.id),
                suspended[i],
                &["income.estate_trust_portfolio.activities"],
                PASSIVE,
                crate::Message::new(
                    "EXPLAIN_ESTATE_ACTIVITY_CARRY",
                    alloc::vec![
                        crate::MessageArg::Number(parent),
                        crate::MessageArg::Text(activity.name.clone())
                    ],
                ),
            );
        }
        if let Some(rental) = pass_through {
            // Schedule E line 28 of the K-1: column (g)/(h) for the passive
            // part, (i)/(k) for a professional's nonpassive part; the
            // suspended loss carries to 2027 (Form 8582 Parts IV/V).
            let amount = allowed_net + nonpassive;
            result.pass_through_rental_nonpassive += nonpassive;
            if rental.real_estate && rental.material {
                professional_pass_through_rentals += amount
                    + result
                        .k1_section1231
                        .iter()
                        .find(|property| property.id == id)
                        .and_then(|property| {
                            property.rentals.iter().find(|property| property.real_estate)
                        })
                        .map_or(0, |property| property.net);
            }
            let (activity, explanation) = match (rental.real_estate, amt) {
                (true, false) => (
                    "rental_real_estate",
                    [
                        msg!("EXPLAIN_K1_RENTAL_REAL_ESTATE_NET"),
                        msg!("EXPLAIN_K1_RENTAL_REAL_ESTATE_SUSPENDED"),
                    ],
                ),
                (true, true) => (
                    "rental_real_estate",
                    [
                        msg!("EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_NET"),
                        msg!("EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_SUSPENDED"),
                    ],
                ),
                (false, false) => (
                    "other_rental",
                    [
                        msg!("EXPLAIN_K1_OTHER_RENTAL_NET"),
                        msg!("EXPLAIN_K1_OTHER_RENTAL_SUSPENDED"),
                    ],
                ),
                (false, true) => (
                    "other_rental",
                    [
                        msg!("EXPLAIN_AMT_K1_OTHER_RENTAL_NET"),
                        msg!("EXPLAIN_AMT_K1_OTHER_RENTAL_SUSPENDED"),
                    ],
                ),
            };
            let [net_explanation, suspended_explanation] = explanation;
            let input = if rental.kind == "scorp" { "income.scorp" } else { "income.partnership" };
            line(
                r,
                &alloc::format!("{prefix}{}_{id}_{activity}_net", rental.kind),
                amount,
                &[input, "rental_magi"],
                PASSIVE,
                net_explanation,
            );
            line(
                r,
                &alloc::format!("{prefix}{}_{id}_{activity}_suspended", rental.kind),
                carried,
                &[input, "rental_magi"],
                PASSIVE,
                suspended_explanation,
            );
            if let Some(property) = result
                .k1_section1231
                .iter()
                .find(|v| v.id == id)
                .and_then(|v| v.rentals.iter().find(|v| v.real_estate == rental.real_estate))
            {
                line(
                    r,
                    &alloc::format!(
                        "{prefix}{}_{id}_{activity}_section1231_suspended",
                        rental.kind
                    ),
                    property.suspended,
                    &[input, "rental_magi"],
                    PASSIVE,
                    msg!("EXPLAIN_K1_RENTAL_SECTION1231_SUSPENDED"),
                );
            }
            // Section 199A: the allowed amount of a rental activity the
            // statement lists as a trade or business (i8995; the
            // convention of Schedule E and Form 4835 rentals).
            if let Some(qbi) = rental.qbi {
                result.qbi += amount;
                *result.business_losses_by_id.entry(id).or_default() += positive(-amount);
                if let Some(property) =
                    result.k1_section1231.iter_mut().find(|v| v.id == id).and_then(|v| {
                        v.rentals.iter_mut().find(|v| v.real_estate == rental.real_estate)
                    })
                {
                    property.qbi_index = Some(result.activities.len());
                }
                if amount < 0 {
                    result.qbi_loss_sources.push(id);
                }
                result.qbi_source_ids.push(id);
                let source_income_cents =
                    exact_allowed_net_cents.checked_add(exact_nonpassive_cents);
                if source_income_cents.is_none() {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::income_id(id),
                        "K-1 rental source endpoint",
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        "Review the document amounts; they must be within the supported input range."
                    );
                    result.source_issue = Some(crate::activity_risk::SourceIssue::new(
                        id,
                        crate::activity_risk::ProjectionIssue::Source(
                            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        ),
                    ));
                    result.inputs_complete = false;
                }
                result.activities.push(crate::business_rules::QbiActivity {
                    source_wages_cents: Some(qbi.wages),
                    source_property_cents: Some(qbi.property),
                    source_income_cents,
                    owner: d.income.iter().find(|e| e.id == id).map(|e| e.owner.owned()),
                    income: amount,
                    wages: dollars(qbi.wages),
                    property: dollars(qbi.property),
                    active: rental.material,
                    service: Some(false),
                    group: None,
                });
            }
        } else if farm_rental {
            result.farm_rental += allowed_net;
            if !amt && inputs_complete {
                crate::farm_rental::report_pool_with_claims(
                    form_context,
                    r,
                    id,
                    if farm_rental_passive_property.contains(&id) {
                        net - disposition_gain
                    } else {
                        net
                    },
                    if farm_rental_passive_property.contains(&id) {
                        positive(-allowed_net)
                    } else {
                        losses[i] - suspended[i]
                    },
                    if farm_rental_passive_property.contains(&id) {
                        ordinary_suspended
                    } else {
                        suspended[i]
                    },
                    d.any_professional(),
                );
            }
            line(
                r,
                &alloc::format!("{prefix}farm_rental_{id}_suspended_loss"),
                suspended[i],
                &["income.farm_rental", "rental_magi"],
                PASSIVE,
                if amt {
                    msg!("EXPLAIN_0FARM_RENTAL_N_SUSPENDED_LOSS")
                } else {
                    msg!("EXPLAIN_FARM_RENTAL_N_SUSPENDED_LOSS")
                },
            );
        } else if !personal
            && scorp.is_none()
            && !partner
            && source_group.is_none()
            && !estate_trust
        {
            result.part_one += allowed_net;
            // Form 8582 Part VIII column (c): the loss allowed is the net loss
            // plus the prior-year unallowed loss, less what stays suspended.
            // The part of it that the activity's own profit absorbs is on
            // line 22 too ("will include the loss allowed to the extent of the
            // net profit", 2025 i8582, How To Report Allowed Losses). A sale
            // gain is Form 4797 income, not part of the Schedule E net.
            let current = if nonpassive != 0 || prior_released > 0 {
                nonpassive + prior_released
            } else {
                net - disposition_gain + prior
            };
            let allowed_loss = positive(-current) + prior - ordinary_suspended;
            if let Some(source) = result.foreign_sources.get_mut(&id) {
                source.settle_pal(current, prior, allowed_loss);
            }
            if !amt && (net < 0 || prior > 0 || (disposition_gain != 0 && allowed_net < 0)) {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleE, "22"),
                    &alloc::format!("rental_{id}_line22_allowed_loss"),
                    -allowed_loss,
                    &["income.rental", "rental_magi"],
                    PASSIVE,
                    msg!("EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS_2"),
                );
            }
            line(
                r,
                &alloc::format!("{prefix}rental_{id}_suspended_loss"),
                suspended[i],
                &["income.rental", "rental_magi"],
                PASSIVE,
                msg!("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS_2"),
            );
        }
    }
    if !amt {
        crate::asset_sale::report_rental_partial_limits(form_context, &result, r);
    }
    for (id, business) in home_activities {
        *result.business_losses_by_id.entry(id).or_default() += positive(-business.income);
        if business.income < 0 {
            result.qbi_loss_sources.push(id);
        }
        result.qbi += business.income;
        if let Some(activity) = business.regular_qbi {
            result.qbi_source_ids.push(id);
            result.activities.push(activity);
        }
    }
    ReportingOutput {
        result,
        post_pal,
        personal_display_balances,
        professional_pass_through_rentals,
    }
}

// i8582 Worksheet 5 ratio: the held loss times the section 179 deduction's
// share of the activity's current and prior losses, never above the deduction.
fn section179_held(section179: i64, net: i64, prior: i64, held: i64) -> i64 {
    if section179 <= 0 || held <= 0 {
        return 0;
    }
    // `net` already nets the prior unallowed loss (Form 8582 column (c)).
    let ordinary_loss = positive(-(net + prior + section179));
    let losses = i128::from(ordinary_loss) + i128::from(prior.max(0)) + i128::from(section179);
    let share = i128::from(held) * i128::from(section179) / losses;
    i64::try_from(share).unwrap_or(section179).min(section179)
}
