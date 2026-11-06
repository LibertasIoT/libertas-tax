use super::*;

pub(crate) struct LinkedSourceInputs<'source, 'draft, 'prepared> {
    pub(crate) allocated_entries:
        &'source [alloc::borrow::Cow<'draft, crate::FederalIncomeEntry>],
    pub(crate) native_scorps: &'source alloc::collections::BTreeMap<i64, crate::ScorpK1>,
    pub(crate) amt_sources:
        &'source mut alloc::vec::Vec<(i64, crate::scorp_amt::ScorpSource<'static>)>,
    pub(crate) professional_groups: &'prepared ProfessionalGroups,
    pub(crate) regular_groups: &'prepared ProfessionalGroups,
    pub(crate) mid_quarter: bool,
    pub(crate) section179: Option<&'prepared crate::Section179PreparedAllocation>,
    pub(crate) regular_section179: Option<&'prepared crate::Section179PreparedAllocation>,
    pub(crate) regular_ptp:
        Option<&'prepared alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>>,
    pub(crate) section179_allowed: &'prepared dyn Fn(i64) -> i64,
}

pub(crate) struct LinkedInput<'source> {
    pub(crate) source_issue: Option<crate::activity_risk::SourceIssue>,
    pub(crate) ptp_sources: alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>,
    pub(crate) operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    pub(crate) operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    pub(crate) values: alloc::vec::Vec<PassiveEntry<'source>>,
    pub(crate) nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) profits: i64,
    pub(crate) nonpassive_income: i64,
    pub(crate) magi: i64,
    pub(crate) rental_niit_adjustment: i64,
    pub(crate) ptp_income: i64,
    pub(crate) ptp_suspended: i64,
}

pub(crate) struct LinkedOutput<'source> {
    pub(crate) source_issue: Option<crate::activity_risk::SourceIssue>,
    pub(crate) ptp_sources: alloc::collections::BTreeMap<i64, crate::ptp_passive::PtpPalResult>,
    pub(crate) operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    pub(crate) operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    pub(crate) values: alloc::vec::Vec<PassiveEntry<'source>>,
    pub(crate) nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub(crate) profits: i64,
    pub(crate) nonpassive_income: i64,
    pub(crate) magi: i64,
    pub(crate) rental_niit_adjustment: i64,
    pub(crate) ptp_income: i64,
    pub(crate) ptp_suspended: i64,
    pub(crate) farm_rental_passive_property: alloc::collections::BTreeSet<i64>,
    pub(crate) farm_rental_passive_sources:
        alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertySource>,
    pub(crate) farm_rental_property: alloc::collections::BTreeMap<i64, crate::farm_land::LandGain>,
    pub(crate) farm_rental_property_rows:
        alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertyRows>,
    pub(crate) k1_ledgers: alloc::vec::Vec<(i64, crate::k1_loss::LossLedger)>,
    pub(crate) has_scorp: bool,
    pub(crate) estate_trust_business: i64,
    pub(crate) estate_trust_nonpassive: i64,
    pub(crate) native_partner_se: alloc::vec::Vec<(i64, crate::partnership::PartnerSeResult)>,
}

pub(crate) fn collect<'draft: 'source, 'source>(
    context: &rental_setup::RentalContext<'draft, '_>,
    sources: LinkedSourceInputs<'source, 'draft, '_>,
    input: LinkedInput<'source>,
    r: &mut FederalResult,
) -> Result<LinkedOutput<'source>, crate::activity_risk::SourceIssue> {
    let form_context = context.form;
    let d = form_context.draft;
    let amt = context.amt;
    let LinkedSourceInputs {
        allocated_entries,
        native_scorps,
        amt_sources,
        professional_groups,
        regular_groups,
        mid_quarter,
        section179,
        regular_section179,
        regular_ptp,
        section179_allowed,
    } = sources;
    let LinkedInput {
        mut source_issue,
        mut ptp_sources,
        mut operating_sources,
        mut operating_prior_allowed,
        mut values,
        mut nii_adjustment_by_id,
        mut profits,
        mut nonpassive_income,
        mut magi,
        mut rental_niit_adjustment,
        mut ptp_income,
        mut ptp_suspended,
    } = input;
    let pooled_section179 = |id: i64| {
        if crate::section179::passive_pass_through(d, id) { section179_allowed(id) } else { 0 }
    };
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
    let loss_limited = |ledger: &crate::k1_loss::LossLedger| {
        let ordinary = crate::k1_loss::K1LossClass::Ordinary;
        ledger.basis_suspended[ordinary].get() != 0 || ledger.risk_suspended[ordinary].get() != 0
    };
    let mut native_partner_se = alloc::vec::Vec::new();
    let mut farm_rental_passive_property = alloc::collections::BTreeSet::new();
    let mut farm_rental_passive_sources = alloc::collections::BTreeMap::new();
    let mut farm_rental_property = alloc::collections::BTreeMap::new();
    let mut farm_rental_property_rows = alloc::collections::BTreeMap::new();
    for e in allocated_entries {
        let FederalIncome::FarmRental { data: v } = &e.income else {
            continue;
        };
        let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
        let projection = rental_source!(crate::farm_rental::risk_source_with_claims(
            form_context,
            e.id,
            v,
            owner,
            mid_quarter,
            amt,
            Some(r)
        ));
        let passive_property = rental_source!(crate::farm_rental::passive_property_gain(
            form_context,
            e.id,
            v,
            owner,
            mid_quarter,
            &projection,
            amt
        ));
        let property_gain = if passive_property {
            farm_rental_passive_property.insert(e.id);
            projection.property.as_ref().map_or(0, |value| value.gain)
        } else {
            if !amt && projection.property.as_ref().is_some_and(|value| value.gain != 0) {
                crate::farm_rental::property_boundary_notes(d, e.id, v, r);
            }
            0
        };
        let property_cents = rental_source!(
            annual_dollars_to_working_cents(property_gain)
                .map_err(|error| crate::activity_risk::SourceIssue::new(e.id, error.into()))
        );
        operating_sources.insert(
            e.id,
            RentalOperatingSource {
                source: e.id,
                owner: e.owner.owned(),
                alternative: amt,
                before_at_risk_cents: projection.before_risk,
                after_at_risk_cents: projection.operating,
                prior_at_risk_cents: projection.prior_risk,
                prior_passive_cents: v.history.balance(amt),
                prior_home_carry_cents: [0; 2],
                home_limited: false,
                excluded: false,
                property_disposition: !v.farm.land_sales.is_empty()
                    || v.assets.iter().any(|asset| {
                        !matches!(asset.disposal, crate::AssetDisposal::No)
                            || asset.partial.is_some()
                    }),
                filed_ordinary_dollars: None,
            },
        );
        operating_prior_allowed.insert(e.id, projection.prior_operating_allowed);
        if passive_property {
            farm_rental_passive_sources.insert(e.id, projection.property_source);
        }
        farm_rental_property_rows.insert(e.id, projection.property_rows);
        if let Some(property) = projection.property {
            farm_rental_property.insert(e.id, property);
        }
        let native_net = if passive_property {
            rental_source!(projection.operating.checked_add(property_cents).ok_or_else(|| {
                crate::activity_risk::SourceIssue::new(
                    e.id,
                    crate::kernel::loss_limits::LossLimitError::Overflow.into(),
                )
            }))
        } else {
            0
        };
        let net = if passive_property {
            dollars(native_net)
        } else {
            dollars(projection.operating) - dollars(v.history.balance(amt))
        };
        profits += positive(net);
        values.push(PassiveEntry {
            amt_source: None,
            source_cents: passive_property.then_some(PassiveSourceCents {
                separate_nonpassive: [0; 4],
                net: native_net,
                prior: 0,
                ineligible_prior: 0,
                nonpassive: 0,
                prior_released: 0,
                disposition_gain: property_cents,
                ptp: 0,
            }),
            id: e.id,
            net,
            rental_allowance: v.active_participation,
            ineligible_prior: ineligible_prior(v.active_participation, &v.history, amt),
            prior: dollars(v.history.balance(amt)),
            personal: false,
            direct_business: false,
            scorp: None,
            partner: false,
            source_group: None,
            characters: None,
            items: None,
            nonpassive: 0,
            prior_released: 0,
            uniform_business: crate::farm_rental::qbi_activity(
                v,
                if passive_property { dollars(projection.operating) } else { net },
                e.owner.owned(),
                e.id,
                form_context.macrs().as_ref(),
            )
            .map(|activity| UniformBusinessIncome {
                income: activity.income,
                regular_qbi: (!amt).then_some(activity),
            }),
            disposition_gain: property_gain,
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: true,
            ptp: 0,
            pass_through: None,
            released: false,
        });
    }
    // Each tax system uses its own retained source records and the same
    // basis -> at-risk -> passive engine. AMT's allowance still uses regular MAGI.
    let mut k1_ledgers = alloc::vec::Vec::new();
    if amt {
        for e in allocated_entries {
            if let FederalIncome::Scorp { .. } = &e.income {
                let data = &native_scorps[&e.id];
                match data.amt.as_ref().ok_or(("ISSUE_SCORP_AMT_BASIS_AND_LOSS_RECORDS", crate::fixture_english!("Complete separate AMT K-1 basis and loss records; absence is not a zero adjustment."))).and_then(|v| v.source(data)) {
                    Ok(source) => {
                        if let Ok(basis) = source.basis_result() {
                            for (_name, key, amount) in [
                                ("ending stock basis", "NOTE_AMT_SCORP_ENDING_STOCK_BASIS", basis.stock),
                                ("basis suspended loss", "NOTE_AMT_SCORP_BASIS_SUSPENDED_LOSS", basis.suspended[crate::k1_loss::K1LossClass::Ordinary]),
                            ] {
                                let message = Message::new(key, alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::usd_cents(amount.get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(),
                                    message,
                                ));
                            }
                            for (index, loan) in basis.loans.iter().enumerate() {
                                let message = Message::new("NOTE_AMT_SCORP_LOAN_ENDING_BASIS", alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::Number((index + 1) as i64), MessageArg::usd_cents(loan.basis.get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                            }
                            if let Ok(risk) = source.risk_result(&basis) {
                                let message = Message::new("NOTE_AMT_SCORP_RISK_SUSPENDED", alloc::vec![
                                    MessageArg::Number(e.id), MessageArg::usd_cents(risk.ledger.risk_suspended[crate::k1_loss::K1LossClass::Ordinary].get()),
                                ]);
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                                let message = match risk.year_end {
                                    Some(end) => Message::new("NOTE_AMT_SCORP_NONDEDUCTIBLE_WITH_END", alloc::vec![MessageArg::Number(e.id), MessageArg::usd_cents(risk.nondeductible_used.get()), MessageArg::usd_cents(risk.nondeductible_suspended.get()), MessageArg::usd_cents(end)]),
                                    None => Message::new("NOTE_AMT_SCORP_NONDEDUCTIBLE_NO_END", alloc::vec![MessageArg::Number(e.id), MessageArg::usd_cents(risk.nondeductible_used.get()), MessageArg::usd_cents(risk.nondeductible_suspended.get())]),
                                };
                                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                                    #[cfg(test)] message.english(), message,
                                ));
                            }
                        }
                        amt_sources.push((e.id, source));
                    },
                    Err((key, _message)) => {
#[cfg(test)] let message = _message;
crate::issue_localized!(r, &crate::IssueContext::ScorpAlternativeMinimum,"S-corporation AMT", key,message)},
                }
            }
        }
    }
    let mut has_scorp = false;
    for e in allocated_entries {
        if let FederalIncome::Scorp { .. } = &e.income {
            let data = &native_scorps[&e.id];
            has_scorp = true;
            // The later passive carrier borrows the stable source record for
            // identity/QBI metadata. All AMT basis/risk/outcome calculations
            // retain this producer's typed capacity view below.
            let (source_record, source): (&crate::ScorpK1, crate::scorp_amt::ScorpSource<'_>) =
                if amt {
                    let Some((_, source)) = amt_sources.iter().find(|(id, _)| *id == e.id) else {
                        continue;
                    };
                    (&**source, source.into())
                } else {
                    (data, data.into())
                };
            let computed =
                crate::community::scorp_pool(d, e.id, &source, amt).and_then(|activity| {
                    let regular = if amt {
                        Some(crate::community::scorp_pool(d, e.id, data, false)?)
                    } else {
                        None
                    };
                    let magi_net = regular
                        .as_ref()
                        .map_or(activity.combined.nonpassive, |v| v.combined.nonpassive);
                    let regular_ledger = regular.as_ref().map_or(activity.ledger, |v| v.ledger);
                    let status = d
                        .professional(crate::community::professional_owner(
                            d.setup.as_ref(),
                            e.owner.owned(),
                        ))
                        .is_some();
                    let mut rentals = source.rental_activities(
                        &activity.ledger,
                        status,
                        professional_group_disposed(d, e.owner.owned()),
                    )?;
                    let mut regular_rentals = data.rental_activities(
                        &regular_ledger,
                        status,
                        professional_group_disposed(d, e.owner.owned()),
                    )?;
                    professional_groups.apply(
                        e.id,
                        &mut rentals,
                        source.rental_facts.carryover_active,
                    );
                    regular_groups.apply(
                        e.id,
                        &mut regular_rentals,
                        data.rental_facts.carryover_active,
                    );
                    Ok((activity, magi_net, rentals, regular_rentals))
                });
            match computed {
                Ok((activity, magi_net, rentals, regular_rentals)) => {
                    let [rental_nonpassive, rental_magi, rental_profits, rental_niit] =
                        pass_through_rentals(
                            &mut values,
                            e.id,
                            "scorp",
                            rentals,
                            &regular_rentals,
                            d.professional(crate::community::professional_owner(
                                d.setup.as_ref(),
                                e.owner.owned(),
                            ))
                            .is_some_and(|p| p.over_500_hours),
                            regular_groups.source(e.id).is_some(),
                        );
                    nonpassive_income += rental_nonpassive;
                    magi += rental_magi;
                    profits += rental_profits;
                    rental_niit_adjustment += rental_niit;
                    *nii_adjustment_by_id.entry(e.id).or_default() += rental_niit;
                    k1_ledgers.push((e.id, activity.ledger));
                    // A passive activity's Form 4562 line 12 deduction is one
                    // of its Form 8582 losses; otherwise it is nonpassive
                    // (Schedule E line 28 column (j)) and reduces MAGI.
                    let pooled179 = pooled_section179(e.id);
                    let net = activity.combined.passive - pooled179;
                    nonpassive_income += dollars(activity.combined.nonpassive);
                    magi += dollars(magi_net)
                        - if pooled179 == 0 { dollars(section179_allowed(e.id)) } else { 0 };
                    profits += dollars(positive(net));
                    values.push(PassiveEntry {
                        amt_source: Some(K1AmtActivitySource {
                            passive: source.nonpassive()
                                != crate::participation::Participation::Yes
                                || activity.combined.prior_pool != 0
                                || activity.combined.prior_released != 0,
                            loss_limited: loss_limited(&activity.ledger),
                            depletion_cents: match &e.income {
                                FederalIncome::Scorp { data } => data.amt_depletion_adjustment(),
                                _ => unreachable!("same S corporation source"),
                            },
                        }),
                        source_cents: Some(PassiveSourceCents {
                            separate_nonpassive: [0; 4],
                            net,
                            prior: activity.combined.prior_pool,
                            ineligible_prior: 0,
                            nonpassive: activity.combined.nonpassive,
                            prior_released: activity.combined.prior_released,
                            disposition_gain: 0,
                            ptp: 0,
                        }),
                        id: e.id,
                        net: dollars(net),
                        rental_allowance: false,
                        ineligible_prior: 0,
                        prior: dollars(activity.combined.prior_pool),
                        personal: false,
                        direct_business: false,
                        scorp: Some(source_record),
                        partner: false,
                        source_group: None,
                        // The pooled section 179 is an ordinary-character loss of the entry.
                        characters: activity.combined.characters.map(|mut parts| {
                            parts.passive[0] -= pooled179;
                            parts
                        }),
                        items: activity
                            .items
                            .any()
                            .then_some((activity.items, source.items_in_pool())),
                        nonpassive: dollars(activity.combined.nonpassive),
                        prior_released: dollars(activity.combined.prior_released),
                        uniform_business: None,
                        disposition_gain: 0,
                        rental_partial: false,
                        estate_trust: false,
                        estate_activity: None,
                        farm_rental: false,
                        ptp: 0,
                        pass_through: None,
                        released: false,
                    });
                }
                Err((key, _message)) => {
                    #[cfg(test)]
                    let message = _message;
                    crate::issue_localized!(
                        r,
                        if amt {
                            &crate::IssueContext::ScorpAlternativeMinimum
                        } else {
                            &crate::IssueContext::Section(crate::TaxSection::PassThrough)
                        },
                        if amt { "S-corporation AMT" } else { "S-corporation passive pool" },
                        key,
                        message,
                    )
                }
            }
        }
    }
    for e in allocated_entries {
        let FederalIncome::Partnership { data } = &e.income else {
            continue;
        };
        let computed = (|| {
            let source = if amt {
                data.alternative.as_ref().ok_or(("ISSUE_EXTENDED_RULES_COMPLETE_SEPARATE_AMT_PARTNERSHIP_ACTIVITY_RECORDS_ABC340B8", crate::fixture_english!("Complete separate AMT partnership activity records; missing data is not a zero adjustment.")))?
            } else {
                &data.regular
            };
            let fuel_recovery = if let Some(prior) = &data.prior_fuel_credit {
                if data.trade_kind != crate::ScorpTradeKind::Ordinary
                    || data.material_for_losses() != crate::participation::Participation::Yes
                    || [&data.regular, data.alternative.as_ref().unwrap_or(&data.regular)]
                        .into_iter()
                        .any(|record| {
                            record.basis.prior_loss != 0
                                || record.risk.prior_loss != 0
                                || record.passive_loss != 0
                                || record.carried != crate::K1LossCarryforwards::default()
                                || record.capital_carried != crate::PartnerCapitalCarry::default()
                        })
                {
                    return Err((
                        "ISSUE_EXTENDED_RULES_CONFIRM_THE_CASH_METHOD_2025_PARTNER_FUEL_F5415DDF",
                        crate::fixture_english!(
                            "A prior-year partner fuel credit recovery requires a continuing nonpassive ordinary U.S. partnership with no retained basis, at-risk or passive losses; other histories need a separate recovery worksheet."
                        ),
                    ));
                }
                dollars(prior.taxable_amount())
            } else {
                0
            };
            let activity = crate::community::partner_pool(d, e.id, data, source)?;
            let magi_net = if amt {
                crate::community::partner_pool(d, e.id, data, &data.regular)?.combined.nonpassive
            } else {
                activity.combined.nonpassive
            };
            let status = d
                .professional(crate::community::professional_owner(
                    d.setup.as_ref(),
                    e.owner.owned(),
                ))
                .is_some();
            let regular_ledger = if amt {
                crate::community::partner_pool(d, e.id, data, &data.regular)?.activity.ledger
            } else {
                activity.activity.ledger
            };
            let mut rentals = data.rental_activities(
                source,
                &activity.activity.ledger,
                status,
                professional_group_disposed(d, e.owner.owned()),
            )?;
            let mut regular_rentals = data.rental_activities(
                &data.regular,
                &regular_ledger,
                status,
                professional_group_disposed(d, e.owner.owned()),
            )?;
            let mut ptp_magi = None;
            if data.publicly_traded && !rentals.is_empty() {
                if let alloc::collections::btree_map::Entry::Vacant(slot) = ptp_sources.entry(e.id)
                {
                    if data.section179 != 0 || data.section179_carryover != 0 {
                        return Err(crate::k1_rental::PTP_SOURCE);
                    }
                    let input = data.ptp_passive_activity(e.id, source, &activity)?;
                    let result = crate::ptp_passive::without_section179(e.id, amt, &[input])
                        .map_err(|_| crate::k1_rental::PTP_SOURCE)?;
                    slot.insert(result);
                }
                ptp_magi = Some(if amt {
                    if let Some(regular) = regular_ptp.and_then(|sources| sources.get(&e.id)) {
                        ptp_reported_magi(regular).ok_or(crate::k1_rental::PTP_SOURCE)?
                    } else {
                        if data.section179 != 0 || data.section179_carryover != 0 {
                            return Err(crate::k1_rental::PTP_SOURCE);
                        }
                        let pool = crate::community::partner_pool(d, e.id, data, &data.regular)?;
                        let input = data.ptp_passive_activity(e.id, &data.regular, &pool)?;
                        let regular = crate::ptp_passive::without_section179(e.id, false, &[input])
                            .map_err(|_| crate::k1_rental::PTP_SOURCE)?;
                        ptp_reported_magi(&regular).ok_or(crate::k1_rental::PTP_SOURCE)?
                    }
                } else {
                    ptp_reported_magi(&ptp_sources[&e.id]).ok_or(crate::k1_rental::PTP_SOURCE)?
                });
                // PTP components are settled by their own owner. Zero common
                // operands cannot admit their losses to another PTP or to the
                // rental allowance (i8582, Special Instructions for PTPs).
                for rental in &mut rentals {
                    rental.passive = 0;
                    rental.prior = 0;
                    rental.allowance = false;
                    rental.ineligible_prior = 0;
                }
            }
            professional_groups.apply(e.id, &mut rentals, data.rental_facts.carryover_active);
            regular_groups.apply(e.id, &mut regular_rentals, data.rental_facts.carryover_active);
            let ptp_printed = ptp_sources
                .get(&e.id)
                .map(|source| ptp_reported_net(source).ok_or(crate::k1_rental::PTP_SOURCE))
                .transpose()?;
            Ok((activity, magi_net, fuel_recovery, rentals, regular_rentals, ptp_magi, ptp_printed))
        })();
        match computed {
            Ok((
                activity,
                magi_net,
                fuel_recovery,
                rentals,
                regular_rentals,
                ptp_magi,
                ptp_printed,
            )) => {
                let [rental_nonpassive, rental_magi, rental_profits, rental_niit] =
                    pass_through_rentals(
                        &mut values,
                        e.id,
                        "partnership",
                        rentals,
                        &regular_rentals,
                        d.professional(crate::community::professional_owner(
                            d.setup.as_ref(),
                            e.owner.owned(),
                        ))
                        .is_some_and(|p| p.over_500_hours),
                        regular_groups.source(e.id).is_some(),
                    );
                nonpassive_income += rental_nonpassive;
                magi += rental_magi;
                profits += rental_profits;
                rental_niit_adjustment += rental_niit;
                *nii_adjustment_by_id.entry(e.id).or_default() += rental_niit;
                // Guaranteed payments are nonpassive but are not ordinary activity
                // income that can release its former passive losses or add basis.
                // The partner's prior Form 4136 recovery is also separate from
                // the current K-1 box 1 and box 14 amounts.
                let guarantees =
                    dollars(data.guaranteed_services) + dollars(data.guaranteed_capital);
                k1_ledgers.push((e.id, activity.activity.ledger));
                // The selected PAL owner already classifies the section179
                // component; consume its nonpassive part once, not a second
                // blanket deduction from ordinary/nonpassive income below.
                let pooled179 = pooled_section179(e.id);
                let selected_nonpassive = ptp_sources.get(&e.id).map_or(0, |source| {
                    source
                        .components
                        .iter()
                        .filter(|part| {
                            part.original.character
                                == crate::ptp_passive::PtpPalCharacter::Section179
                        })
                        .map(|part| part.original.nonpassive_cents)
                        .sum::<i64>()
                });
                nonpassive_income += dollars(activity.combined.nonpassive)
                    + guarantees
                    + fuel_recovery
                    + dollars(selected_nonpassive);
                magi += dollars(magi_net)
                    + guarantees
                    + fuel_recovery
                    + ptp_magi.unwrap_or_else(|| {
                        if pooled179 == 0 { -dollars(section179_allowed(e.id)) } else { 0 }
                    });
                // Section 469(k): a publicly traded partnership's passive
                // result is limited by itself and never joins the common
                // Form 8582 pool (2025 i8582, Special Instructions for PTPs).
                // Its net income is nonpassive income of the return; its net
                // loss stays suspended with the partnership.
                let ptp = if data.publicly_traded { activity.combined.passive } else { 0 };
                let pool_net = activity.combined.passive - ptp - pooled179;
                if data.publicly_traded {
                    let (gain, held) = ptp_sources.get(&e.id).map_or_else(
                        || {
                            crate::ptp_passive::single_activity_cents(e.id, ptp)
                                .expect("validated native PTP activity fits its own passive pool")
                        },
                        |source| (source.net_gain_cents, source.suspended_cents),
                    );
                    let printed_gain = ptp_printed.unwrap_or(dollars(gain));
                    ptp_income += printed_gain;
                    ptp_suspended += dollars(held);
                }
                profits += dollars(positive(pool_net));
                let separate_nonpassive = fuel_recovery.checked_mul(100).map(|fuel| {
                    [data.guaranteed_services, data.guaranteed_capital, fuel, selected_nonpassive]
                });
                let source_nonpassive = separate_nonpassive
                    .and_then(|parts| parts.into_iter().try_fold(0i64, i64::checked_add))
                    .and_then(|separate| activity.combined.nonpassive.checked_add(separate));
                let (Some(separate_nonpassive), Some(source_nonpassive)) =
                    (separate_nonpassive, source_nonpassive)
                else {
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(e),
                        "Partnership",
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        "Review the document amounts; they must be within the supported input range."
                    );
                    continue;
                };
                values.push(PassiveEntry {
                    amt_source: Some(K1AmtActivitySource {
                        passive: data.material_for_losses()
                            != crate::participation::Participation::Yes
                            || activity.combined.prior_pool != 0
                            || activity.combined.prior_released != 0,
                        loss_limited: loss_limited(&activity.activity.ledger),
                        depletion_cents: data
                            .source
                            .as_ref()
                            .map_or(Some(0), |source| source.amt_depletion_adjustment()),
                    }),
                    source_cents: Some(PassiveSourceCents {
                        separate_nonpassive,
                        net: pool_net,
                        prior: if data.publicly_traded { 0 } else { activity.combined.prior_pool },
                        ineligible_prior: 0,
                        nonpassive: source_nonpassive,
                        prior_released: activity.combined.prior_released,
                        disposition_gain: 0,
                        ptp,
                    }),
                    id: e.id,
                    net: dollars(pool_net),
                    rental_allowance: false,
                    ineligible_prior: 0,
                    // A PTP's prior-year unallowed loss offsets only that PTP's
                    // income (section 469(k)): none of it is a Form 8582 entry.
                    prior: if data.publicly_traded {
                        0
                    } else {
                        dollars(activity.combined.prior_pool)
                    },
                    personal: false,
                    direct_business: false,
                    scorp: None,
                    partner: true,
                    source_group: None,
                    // The pooled section 179 is an ordinary-character loss of the entry.
                    characters: activity.combined.characters.map(|mut parts| {
                        parts.passive[0] -= pooled179;
                        parts
                    }),
                    items: activity.items.any().then_some((activity.items, data.items_in_pool())),
                    nonpassive: dollars(activity.combined.nonpassive)
                        + guarantees
                        + fuel_recovery
                        + dollars(selected_nonpassive),
                    prior_released: dollars(activity.combined.prior_released),
                    uniform_business: None,
                    disposition_gain: 0,
                    rental_partial: false,
                    estate_trust: false,
                    estate_activity: None,
                    farm_rental: false,
                    ptp: dollars(ptp),
                    pass_through: None,
                    released: false,
                });
            }
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    r,
                    &if key == "ISSUE_K1_PTP_PROPERTY"
                        || [
                            crate::k1_rental::PTP_PROPERTY.0,
                            crate::k1_rental::PTP_PARTICIPATION.0,
                            crate::k1_rental::PTP_DISPOSITION.0,
                            crate::k1_rental::PTP_QBI.0,
                            crate::k1_rental::PTP_SOURCE.0,
                        ]
                        .contains(&key)
                    {
                        crate::interview::income_issue_context(e)
                    } else {
                        crate::IssueContext::Section(crate::TaxSection::PassThrough)
                    },
                    if amt { "Partnership AMT" } else { "Partnership passive pool" },
                    key,
                    message,
                )
            }
        }
    }
    // Actual partnership and S-corporation graphs share this return-level pool.
    // The parent identity owns return totals; the group identity owns PAL.
    for e in allocated_entries {
        let (preliminary, partner) = match &e.income {
            FederalIncome::PartnershipActivities { data } => (data.before_section179(amt), true),
            FederalIncome::ScorpActivities { data } => (data.before_section179(amt), false),
            _ => continue,
        };
        let admitted = preliminary.is_ok();
        let computed = preliminary.and_then(|output| {
            let output = output.apply_section179(section179)?;
            let regular = if amt {
                let source = match &e.income {
                    FederalIncome::PartnershipActivities { data } => data.before_section179(false),
                    FederalIncome::ScorpActivities { data } => data.before_section179(false),
                    _ => unreachable!(),
                };
                Some(source?.apply_section179(regular_section179)?)
            } else {
                None
            };
            Ok((output, regular))
        });
        // Only a failure after preliminary admission belongs here. The
        // source/community admission boundary remains owned by collect.
        if let Err((issue, _message)) = &computed {
            let context = crate::interview::income_issue_context(e);
            if admitted
                && !r.issues.iter().any(|existing| {
                    existing.display.key == issue.key
                        && existing.context.source() == context.source()
                })
            {
                crate::issue_localized!(r, &context, "K-1 activity", issue.key, *_message);
            }
            continue;
        }
        let computed = computed.and_then(|(output, regular)| {
            if d.setup.as_ref().is_some_and(crate::community::applies) {
                return Err(crate::k1_activity::ordinary::UNAVAILABLE);
            }
            Ok((output, regular))
        });
        // collect owns the single scoped source/community boundary.
        // Neither tax-system pass may duplicate it or admit graph values.
        if let Ok((output, regular)) = computed {
            if let Some(source) = output.self_employment {
                native_partner_se.push((e.id, source.net));
            }
            let regular_groups = regular.as_ref().map_or(&output.groups, |v| &v.groups);
            magi += dollars(regular_groups.iter().map(|g| g.nonpassive).sum::<i64>());
            nonpassive_income += dollars(output.groups.iter().map(|g| g.nonpassive).sum::<i64>());
            profits += dollars(output.groups.iter().map(|g| positive(g.passive)).sum::<i64>());
            if !partner {
                let message = Message::new(
                    if amt {
                        "NOTE_AMT_SCORP_ENDING_STOCK_BASIS"
                    } else {
                        "NOTE_SCORP_STOCK_BASIS"
                    },
                    alloc::vec![
                        MessageArg::Number(e.id),
                        MessageArg::usd_cents(output.basis.ending.get())
                    ],
                );
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)]
                    message.english(),
                    message,
                ));
            }
            if let Some(basis) = &output.basis.scorp {
                for (index, loan) in basis.loans.iter().enumerate() {
                    let loan_id = MessageArg::Number((index + 1) as i64);
                    for (key, value) in [
                        (
                            if amt {
                                "NOTE_AMT_SCORP_LOAN_ENDING_BASIS"
                            } else {
                                "NOTE_SCORP_LOAN_BASIS"
                            },
                            loan.basis,
                        ),
                        (
                            if amt {
                                "NOTE_AMT_SCORP_LOAN_RESTORATION"
                            } else {
                                "NOTE_SCORP_LOAN_RESTORATION"
                            },
                            loan.restoration,
                        ),
                    ] {
                        let money = MessageArg::usd_cents(value.get());
                        let args = if amt {
                            alloc::vec![MessageArg::Number(e.id), loan_id.clone(), money]
                        } else {
                            alloc::vec![MessageArg::Number(e.id), money, loan_id.clone()]
                        };
                        let message = Message::new(key, args);
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(
                            #[cfg(test)]
                            message.english(),
                            message,
                        ));
                    }
                }
            }
            if !amt {
                for component in &output.components {
                    if component.section179 != 0 || component.section179_basis_suspended != 0 {
                        for (suffix, cents, explanation) in [
                            (
                                "section179",
                                component.section179_allowed,
                                msg!("EXPLAIN_N_N_N_SECTION179"),
                            ),
                            (
                                "section179_carryforward",
                                component.section179_carry,
                                msg!("EXPLAIN_N_N_N_SECTION179_CARRYFORWARD"),
                            ),
                        ] {
                            crate::line_emission::line_cents(
                                r,
                                &alloc::format!(
                                    "{}_{}_{suffix}",
                                    if partner { "partnership" } else { "scorp" },
                                    component.id
                                ),
                                cents,
                                &["income.partnership", "income.scorp"],
                                "https://www.irs.gov/instructions/i4562",
                                explanation,
                            );
                        }
                    }
                }
            }
            for component in &output.components {
                for (term, capital) in ["short", "long"].into_iter().zip(&component.capital) {
                    if [
                        capital.income.get(),
                        capital.basis_allowed.get(),
                        capital.basis_suspended.get(),
                        capital.risk_allowed.get(),
                        capital.risk_suspended.get(),
                    ]
                    .into_iter()
                    .all(|value| value == 0)
                    {
                        continue;
                    }
                    for (stage, cents, key) in [
                        (
                            "net",
                            capital.income.get() - capital.risk_allowed.get(),
                            "EXPLAIN_K1_COMPONENT_CAPITAL_NET",
                        ),
                        (
                            "basis_allowed",
                            capital.basis_allowed.get(),
                            "EXPLAIN_K1_COMPONENT_CAPITAL_BASIS_ALLOWED",
                        ),
                        (
                            "basis_carry",
                            capital.basis_suspended.get(),
                            "EXPLAIN_K1_COMPONENT_CAPITAL_BASIS_CARRY",
                        ),
                        (
                            "risk_allowed",
                            capital.risk_allowed.get(),
                            "EXPLAIN_K1_COMPONENT_CAPITAL_RISK_ALLOWED",
                        ),
                        (
                            "risk_carry",
                            capital.risk_suspended.get(),
                            "EXPLAIN_K1_COMPONENT_CAPITAL_RISK_CARRY",
                        ),
                    ] {
                        crate::line_emission::line_cents(
                            r,
                            &alloc::format!(
                                "{}k1_{}_component_{}_{term}_{stage}",
                                if amt { "amt_" } else { "" },
                                e.id,
                                component.id
                            ),
                            cents,
                            &[if partner {
                                "income.partnership.activities"
                            } else {
                                "income.scorp.activities"
                            }],
                            "https://www.irs.gov/instructions/i1040sd",
                            Message::new(key, alloc::vec![]),
                        );
                    }
                }
            }
            // Retain owner-stage precision in worksheet details.
            let mut detail = |key, amount, source_id, explanation| {
                let message = Message::new(
                    key,
                    alloc::vec![
                        MessageArg::Number(e.id),
                        MessageArg::usd_cents(amount),
                        MessageArg::Number(i64::from(d.tax_year)),
                        MessageArg::Nested(alloc::boxed::Box::new(explanation)),
                        MessageArg::Number(source_id),
                    ],
                );
                r.review_notes.push(crate::federal_model::ReviewNote::localized(
                    #[cfg(test)]
                    message.english(),
                    message,
                ));
            };
            if partner {
                detail(
                    if amt {
                        "NOTE_AMT_PARTNERSHIP_ENTITY_BASIS_ENDING"
                    } else {
                        "NOTE_PARTNERSHIP_ENTITY_BASIS_ENDING"
                    },
                    output.basis.ending.get(),
                    e.id,
                    msg!("EXPLAIN_K1_ACTIVITY_ENTITY_BASIS"),
                );
            }
            if output.basis.nondeductible_used.get() != 0
                || output.basis.nondeductible_carry.get() != 0
            {
                detail(
                    if amt {
                        "NOTE_AMT_K1_ENTITY_NONDEDUCTIBLE_USED"
                    } else {
                        "NOTE_K1_ENTITY_NONDEDUCTIBLE_USED"
                    },
                    output.basis.nondeductible_used.get(),
                    e.id,
                    msg!("EXPLAIN_K1_ENTITY_NONDEDUCTIBLE_USED"),
                );
                detail(
                    if amt {
                        "NOTE_AMT_K1_ENTITY_NONDEDUCTIBLE_CARRY"
                    } else {
                        "NOTE_K1_ENTITY_NONDEDUCTIBLE_CARRY"
                    },
                    output.basis.nondeductible_carry.get(),
                    e.id,
                    if partner {
                        msg!("EXPLAIN_PARTNER_ENTITY_NONDEDUCTIBLE_CARRY")
                    } else {
                        msg!("EXPLAIN_SCORP_ENTITY_NONDEDUCTIBLE_CARRY")
                    },
                );
            }
            for component in &output.components {
                for (key, amount, explanation) in [
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_BASIS_ALLOWED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_BASIS_ALLOWED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_BASIS_ALLOWED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_BASIS_ALLOWED",
                        },
                        component.basis_allowed.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_BASIS_ALLOWED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_BASIS_SUSPENDED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_BASIS_SUSPENDED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_BASIS_SUSPENDED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_BASIS_SUSPENDED",
                        },
                        component.basis_suspended.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_BASIS_SUSPENDED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_RISK_ALLOWED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_RISK_ALLOWED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_RISK_ALLOWED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_RISK_ALLOWED",
                        },
                        component.risk_allowed.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_RISK_ALLOWED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_RISK_SUSPENDED",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_RISK_SUSPENDED",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_RISK_SUSPENDED",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_RISK_SUSPENDED",
                        },
                        component.risk_suspended.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_RISK_SUSPENDED"),
                    ),
                    (
                        match (partner, amt) {
                            (true, true) => "NOTE_AMT_PARTNERSHIP_COMPONENT_PASSIVE_OPENING",
                            (true, false) => "NOTE_PARTNERSHIP_COMPONENT_PASSIVE_OPENING",
                            (false, true) => "NOTE_AMT_SCORP_ACTIVITY_COMPONENT_PASSIVE_OPENING",
                            (false, false) => "NOTE_SCORP_ACTIVITY_COMPONENT_PASSIVE_OPENING",
                        },
                        component.prior_passive.get(),
                        msg!("EXPLAIN_K1_ACTIVITY_PASSIVE_OPENING"),
                    ),
                ] {
                    detail(key, amount, component.id, explanation);
                }
            }
            for group in output.groups {
                detail(
                    match (partner, amt) {
                        (true, true) => "NOTE_AMT_PARTNERSHIP_GROUP_BEFORE_PASSIVE",
                        (true, false) => "NOTE_PARTNERSHIP_GROUP_BEFORE_PASSIVE",
                        (false, true) => "NOTE_AMT_SCORP_ACTIVITY_GROUP_BEFORE_PASSIVE",
                        (false, false) => "NOTE_SCORP_ACTIVITY_GROUP_BEFORE_PASSIVE",
                    },
                    group.current,
                    group.id,
                    msg!("EXPLAIN_K1_ACTIVITY_BEFORE_PASSIVE"),
                );
                values.push(PassiveEntry {
                    amt_source: Some(K1AmtActivitySource {
                        passive: !group.material() || group.prior != 0 || group.prior_released != 0,
                        loss_limited: output.components.iter().any(|component| {
                            component.passive_group == group.id
                                && (component.basis_suspended.get() != 0
                                    || component.risk_suspended.get() != 0)
                        }),
                        depletion_cents: output
                            .components
                            .iter()
                            .filter(|component| component.passive_group == group.id)
                            .try_fold(0i64, |total, component| {
                                let cents = match &e.income {
                                    FederalIncome::PartnershipActivities { data } => {
                                        data.depletion_adjustment(component.id)
                                    }
                                    FederalIncome::ScorpActivities { data } => {
                                        data.depletion_adjustment(component.id)
                                    }
                                    _ => unreachable!("same component source"),
                                }?;
                                total.checked_add(cents)
                            }),
                    }),
                    source_cents: Some(PassiveSourceCents {
                        separate_nonpassive: [0; 4],
                        net: group.passive,
                        prior: group.prior,
                        ineligible_prior: 0,
                        nonpassive: group.nonpassive,
                        prior_released: group.prior_released,
                        disposition_gain: 0,
                        ptp: 0,
                    }),
                    id: e.id,
                    net: dollars(group.passive),
                    rental_allowance: false,
                    ineligible_prior: 0,
                    prior: dollars(group.prior),
                    personal: false,
                    direct_business: false,
                    scorp: None,
                    partner,
                    source_group: Some(group.id),
                    characters: None,
                    items: None,
                    nonpassive: dollars(group.nonpassive),
                    prior_released: dollars(group.prior_released),
                    uniform_business: None,
                    disposition_gain: 0,
                    rental_partial: false,
                    estate_trust: false,
                    estate_activity: None,
                    farm_rental: false,
                    pass_through: None,
                    released: false,
                    ptp: 0,
                });
            }
        }
    }
    // Each fiduciary activity retains its own regular/AMT passive history.
    // Section 469(i)'s rental allowance never applies to beneficiary losses.
    let estate_trust_business = 0;
    let mut estate_trust_nonpassive = 0;
    for e in allocated_entries {
        let FederalIncome::EstateTrustPortfolio { data } = &e.income else {
            continue;
        };
        for activity in &data.statements.activities {
            let current = dollars(activity.net(amt));
            let prior = dollars(activity.prior(amt));
            let material = activity.material();
            let parts = crate::scorp::ActivityCharacters::from_current_and_prior(
                [current, 0],
                [prior, 0],
                material,
                false,
            )
            .expect("validated beneficiary activity has one income character");
            let passive = parts.passive[0];
            let nonpassive = parts.nonpassive[0];
            nonpassive_income += nonpassive;
            magi += positive(nonpassive);
            estate_trust_nonpassive += nonpassive;
            *nii_adjustment_by_id.entry(e.id).or_default() -= nonpassive;
            profits += positive(passive);
            values.push(PassiveEntry {
                amt_source: None,
                source_cents: None,
                id: e.id,
                net: passive,
                rental_allowance: false,
                ineligible_prior: 0,
                prior: prior - parts.prior_released[0],
                personal: false,
                direct_business: false,
                scorp: None,
                partner: false,
                source_group: None,
                characters: None,
                nonpassive,
                prior_released: parts.prior_released[0],
                uniform_business: None,
                disposition_gain: 0,
                rental_partial: false,
                estate_trust: true,
                estate_activity: Some((e.id, activity)),
                items: None,
                farm_rental: false,
                pass_through: None,
                ptp: 0,
                released: false,
            });
        }
    }
    // Current ordinary Schedule C sources have already passed their selected
    // deductions and Form6198. The accepted scope excludes prior losses;
    // neither a printed form net nor annual property Dollars supplies cents.
    let participation = section179
        .map(|source| source.participation())
        .unwrap_or_else(|| form_context.participation());
    for entry in &d.income {
        let FederalIncome::Business { data } = &entry.income else { continue };
        if !data.current_passive_supported(participation.get(entry.id))
            || crate::statutory_wages::business(d, entry.id)
        {
            continue;
        }
        let Some(source) = section179.and_then(|source| source.passive_ordinary.get(&entry.id))
        else {
            crate::section179_return::Issue::Activity(entry.id,
                "BUSINESS_PASSIVE_OPERATING_SOURCE_REQUIRED",
                "The passive business needs its actual selected operating source before the loss limit.").report(r);
            source_issue.get_or_insert(crate::activity_risk::SourceIssue::new(
                entry.id,
                crate::activity_risk::ProjectionIssue::Source(
                    "BUSINESS_PASSIVE_OPERATING_SOURCE_REQUIRED",
                ),
            ));
            continue;
        };
        let property = section179.and_then(|selected| selected.direct_property.get(&entry.id));
        let net = rental_amount!(
            entry.id,
            crate::kernel::ratio::checked_sum(
                core::iter::once(source.before_pal_cents)
                    .chain(property.into_iter().flatten().flat_map(|part| part.risk_allowed_cents))
            )
            .ok_or(crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow
            ))
        );
        values.push(PassiveEntry {
            amt_source: None,
            id: entry.id,
            net: dollars(net),
            source_cents: Some(PassiveSourceCents {
                separate_nonpassive: [0; 4],
                net,
                prior: 0,
                ineligible_prior: 0,
                nonpassive: 0,
                prior_released: 0,
                disposition_gain: 0,
                ptp: 0,
            }),
            rental_allowance: false,
            ineligible_prior: 0,
            prior: 0,
            personal: false,
            direct_business: true,
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
            pass_through: None,
            released: false,
            ptp: 0,
        });
    }
    if let Some(missing) = values.iter().find(|entry| {
        (entry.partner
            || entry.scorp.is_some()
            || entry.source_group.is_some()
            || entry.pass_through.is_some())
            && entry.source_cents.is_none()
    }) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            "Partnership activity allocation",
            "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
            "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records."
        );
        return Err(crate::activity_risk::SourceIssue::new(
            missing.id,
            crate::activity_risk::ProjectionIssue::Source(
                "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
            ),
        ));
    }
    Ok(LinkedOutput {
        source_issue,
        ptp_sources,
        operating_sources,
        operating_prior_allowed,
        values,
        nii_adjustment_by_id,
        profits,
        nonpassive_income,
        magi,
        rental_niit_adjustment,
        ptp_income,
        ptp_suspended,
        farm_rental_passive_property,
        farm_rental_passive_sources,
        farm_rental_property,
        farm_rental_property_rows,
        k1_ledgers,
        has_scorp,
        estate_trust_business,
        estate_trust_nonpassive,
        native_partner_se,
    })
}
