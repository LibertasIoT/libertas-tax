// k1 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form 8941 line 15 (code BA) and Form 8994 line 2 (code BB): the owner's
// K-1 (1065) box 15 and K-1 (1120-S) box 13 credits of one code, whole
// dollars. Each K-1 amount also reaches Form 3800 Part III on its own entry.
pub(super) fn k1_credit_code(
    d: &FederalDraft,
    owner: crate::Owner,
    code: crate::K1CreditCode,
) -> i64 {
    dollars(
        d.income
            .iter()
            .filter(|entry| entry.owner == crate::DocumentOwner::from(owner))
            .filter_map(|entry| match &entry.income {
                FederalIncome::Partnership { data } => Some(data.credits.clone()),
                FederalIncome::Scorp { data } => Some(crate::ScorpK1::from(data).credits),
                _ => None,
            })
            .flatten()
            .filter(|credit| credit.code == code)
            .map(|credit| credit.amount)
            .sum(),
    )
}

pub(super) fn collect_k1_activities(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    k1_components: &impl Fn(&Totals) -> ([i64; 13], Exact),
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let output = match &e.income {
        FederalIncome::PartnershipActivities { data } => data
            .before_section179(false)
            .and_then(|regular| data.before_section179(true).map(|amt| (regular, amt))),
        FederalIncome::ScorpActivities { data } => data
            .before_section179(false)
            .and_then(|regular| data.before_section179(true).map(|amt| (regular, amt))),
        _ => unreachable!("activity graph source"),
    };
    let output = output.and_then(|output| {
        // The return owner reports this boundary once; the graph
        // consumer still withholds its unprojected community values.
        if d.setup.as_ref().is_some_and(crate::community::applies) {
            Err(crate::k1_activity::ordinary::UNAVAILABLE)
        } else {
            Ok(output)
        }
    });
    let output =
        output.and_then(|(regular, alternative)| {
            t.native_entity_portfolio_fits(
            &regular,
            &alternative,
        ).map_err(|issue| (issue, crate::fixture_english!(
            "Review the document amounts; they must be within the supported input range."
        )))?;
            Ok((regular, alternative))
        });
    match output {
        Ok((regular, alternative)) => {
            // Only available native outputs establish a final source,
            // including a genuine known-zero contribution.
            cx.previous_k1 = Some((
                e.id,
                usize::from(e.owner == crate::DocumentOwner::Spouses),
                k1_components(t),
            ));
            // IRS i6251 refigures AMT income and Form 4952 from
            // the actual sources, independently of regular income.
            t.k1_amt_interest +=
                i128::from(alternative.entity_interest) - i128::from(regular.entity_interest);
            if alternative.entity_interest != regular.entity_interest {
                t.k1_amt_interest_source = Some(context.clone());
            }
            if regular.entity_interest != 0 {
                // The existing per-K1 retainer attributes this interest
                // to its source owner for NIIT and injured-spouse totals.
                t.interest += regular.entity_interest;
                t.research_items.entry(e.id).or_default().income += regular.entity_interest;
                let (kind, path, citation) = match &e.income {
                    FederalIncome::PartnershipActivities { .. } => (
                        "partnership",
                        "income.partnership.activities.entity_only.portfolio.interest",
                        "https://www.irs.gov/instructions/i1065sk1",
                    ),
                    FederalIncome::ScorpActivities { .. } => (
                        "scorp",
                        "income.scorp.activities.entity_only.portfolio.interest",
                        "https://www.irs.gov/instructions/i1120ssk",
                    ),
                    _ => unreachable!("activity graph source"),
                };
                line_cents(
                    r,
                    &alloc::format!("{kind}_{}_interest", e.id),
                    regular.entity_interest,
                    &[path],
                    citation,
                    msg!("EXPLAIN_1040_2B"),
                );
            }
            // Portfolio character stays outside ordinary PAL, SE and QBI.
            // Source cents join the existing annual category and owner packets.
            for ((values, income), (amt_values, amt_income)) in regular
                .capital_sources()
                .zip(regular.capital_income())
                .zip(alternative.capital_sources().zip(alternative.capital_income()))
            {
                // Form 1116 lines 3d/3e retain gains before released losses,
                // even when both belong to the same component and term.
                t.add_short(income[0]);
                t.add_short(values[0] - income[0]);
                t.add_long(income[1]);
                t.add_long(values[1] - income[1]);
                for term in 0..2 {
                    t.schedule_d_k1[term] += values[term];
                    t.investment_gains[term] += values[term];
                    let adjustment = amt_values[term] - values[term];
                    t.amt_sale_adjustment[term] += adjustment;
                    t.amt_investment_gain_adjustment[term] += adjustment;
                    t.amt_k1_capital_net_adjustment += adjustment;
                    t.amt_k1_capital_gross_adjustment += amt_income[term] - income[term];
                }
            }
            let (kind, capital_path) =
                if matches!(&e.income, FederalIncome::PartnershipActivities { .. }) {
                    ("partnership", "income.partnership.activities.entity_only.portfolio")
                } else {
                    ("scorp", "income.scorp.activities.entity_only.portfolio")
                };
            for (suffix, amount, explanation) in [
                ("short_gain", regular.entity_capital[0], msg!("EXPLAIN_K1_ACTIVITY_SHORT_GAIN")),
                ("long_gain", regular.entity_capital[1], msg!("EXPLAIN_K1_ACTIVITY_LONG_GAIN")),
            ] {
                if amount != 0 {
                    line_cents(
                        r,
                        &alloc::format!("{kind}_{}_{suffix}", e.id),
                        amount,
                        &[capital_path],
                        "https://www.irs.gov/instructions/i1040sd",
                        explanation,
                    );
                }
            }
            if regular.entity_exempt_interest != 0 {
                // Transfer the regular source once, after both systems
                // are available. Existing return owners round line 2a
                // and use exempt interest in their dependent worksheets.
                t.exempt += regular.entity_exempt_interest;
                let (kind, path, citation) = match &e.income {
                    FederalIncome::PartnershipActivities { .. } => (
                        "partnership",
                        "income.partnership.activities.entity_only.exempt_interest",
                        "https://www.irs.gov/instructions/i1065sk1",
                    ),
                    FederalIncome::ScorpActivities { .. } => (
                        "scorp",
                        "income.scorp.activities.entity_only.exempt_interest",
                        "https://www.irs.gov/instructions/i1120ssk",
                    ),
                    _ => unreachable!("activity graph source"),
                };
                line_cents(
                    r,
                    &alloc::format!("{kind}_{}_exempt_interest", e.id),
                    regular.entity_exempt_interest,
                    &[path],
                    citation,
                    msg!("EXPLAIN_1040_2A"),
                );
            }
            if regular.entity_dividends != 0 {
                t.dividends += regular.entity_dividends;
                t.qualified += regular.entity_qualified_dividends;
                t.research_items.entry(e.id).or_default().income += regular.entity_dividends;
            }
            t.k1_amt_dividends += alternative.entity_dividends - regular.entity_dividends;
            t.k1_amt_qualified +=
                alternative.entity_qualified_dividends - regular.entity_qualified_dividends;
            let (prefix, ordinary_path, qualified_path, alternative_path, citation) =
                match &e.income {
                    FederalIncome::PartnershipActivities { .. } => (
                        "partnership",
                        "income.partnership.activities.entity_only.portfolio.dividends",
                        "income.partnership.activities.entity_only.portfolio.qualified_dividends",
                        "income.partnership.activities.alternative_entity_only",
                        "https://www.irs.gov/instructions/i1065sk1",
                    ),
                    _ => (
                        "scorp",
                        "income.scorp.activities.entity_only.portfolio.dividends",
                        "income.scorp.activities.entity_only.portfolio.qualified_dividends",
                        "income.scorp.activities.alternative_entity_only",
                        "https://www.irs.gov/instructions/i1120ssk",
                    ),
                };
            for (amt, amounts) in [(false, &regular), (true, &alternative)] {
                for (name, amount, path, note) in [
                    ("dividends", amounts.entity_dividends, ordinary_path, msg!("EXPLAIN_1040_3B")),
                    (
                        "qualified_dividends",
                        amounts.entity_qualified_dividends,
                        qualified_path,
                        msg!("EXPLAIN_1040_3A"),
                    ),
                ] {
                    if amount != 0 {
                        line_cents(
                            r,
                            &alloc::format!(
                                "{}{prefix}_{}_{name}",
                                if amt { "amt_" } else { "" },
                                e.id
                            ),
                            amount,
                            &[if amt { alternative_path } else { path }],
                            citation,
                            note,
                        );
                    }
                }
            }
        }
        Err((key, _message)) => {
            t.injured_other_sources.insert(e.id, None);
            #[cfg(test)]
            let message = _message;
            crate::issue_localized!(r, &context, &source, key.key, message);
        }
    }
    Ok(())
}

pub(super) fn collect_partnership(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::PartnerK1,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    if let Some(issue) =
        data.source.as_deref().and_then(crate::PartnershipSource::required_issue).or_else(|| {
            crate::k1_page::K1Partnership::from_record(&e.label, data).and_then(|page| {
                data.source
                    .as_deref()
                    .and_then(|source| source.amt_records_issue(page.amt.as_ref()))
                    .or_else(|| page.issue())
            })
        })
        && !matches!(
            issue.key,
            "K1_INVESTMENT_NEEDS_CONFIRMATION" | "K1_HOUSING_NEEDS_CONFIRMATION"
        )
    {
        crate::issue_localized!(
            r,
            &context,
            &source,
            issue.key,
            "Review this K-1's answers against its corrected printed amounts."
        );
    }
    crate::require_localized!(
        r,
        data.investment_property.iter().all(|property| property.own_form().is_some())
            || data.investment_confirmed.is_some(),
        &context,
        &source,
        "K1_INVESTMENT_NEEDS_CONFIRMATION",
        "Answer whether this K-1's investment credit property is confirmed."
    );
    crate::require_localized!(
        r,
        !data.credits.iter().any(|credit| credit.code.housing())
            || data.housing_confirmed.is_some(),
        &context,
        &source,
        "K1_HOUSING_NEEDS_CONFIRMATION",
        "Answer whether you acquired your interest in this K-1's low-income housing credit after 1989."
    );
    if let Some(claim) = &data.fuel_tax_credit {
        crate::require_localized!(
            r,
            claim.fuels.card_issuer.as_ref().is_none_or(|issuer| !issuer.has_retained_tax())
                && claim.fuels.qualified(),
            &crate::IssueContext::Section(crate::TaxSection::PassThrough),
            "Partnership fuel tax credit",
            "ISSUE_CONFIRM_THE_K_1_BOX_20_CODE_C_FUEL_QUANTITIES_USE_APPLICABLE_RATE_P_8AB09713",
            "Confirm the K-1 box 20 code C fuel quantities, use, applicable rate, partner-share cost and no earlier Form 8849/8864/720 claim. Farm-operator fuel lines require an established farm activity; a custom aerial applicator may claim its own farm-use aviation gasoline or kerosene without operating the farm, with an unwaived kerosene claim; public-bus diesel/kerosene and foreign-trade aviation kerosene need unwaived claims; foreign-trade aviation gasoline needs civil-aircraft trade and reciprocity for foreign registry; exports need retained proof, and a blender claim needs the partnership's M registration and certification. Registered-vendor government diesel line 6a and nonaviation-kerosene line 7a require distinct seller UV registration, buyer certificates, sale records, tax relief and no government-issued credit card. Blocked-pump kerosene line 7b requires the partnership seller's UP registration, qualifying fixed and marked pump, nontaxable buyer use, and buyer records for locked-pump sales over five gallons. Registered-vendor bus lines 6b and 7c require the partnership seller's UB registration, current buyer waivers, qualifying public-bus service and distinct sale records. Commercial-aviation kerosene vendor lines 8a and 8b require the partnership seller's UA registration, current signed Model Waiver L, separate original-tax-rate sales, and buyer and cost records. Nonexempt noncommercial aviation line 8c requires its separate UA vendor claim, 24.4-cent original tax, current signed Model Certificate Q, distinct sale and cost records, and no duplicated gallons. Nontaxable noncommercial aviation lines 8d/8e need the original tax rate, eligible use, current Model Waiver L with UA registration or Model Certificate P with UV registration for government sales, and distinct sale and cost records; foreign-trade gallons also generate line 8f LUST credit. A registered card issuer on line 13 needs its own active CC registration, government-issued credit-card purchase, current signed Model Certificate R, buyer tax relief, vendor settlement, actual cost, and no duplicate Form 720/8849 claim; only the partner share from box 20 code C belongs here. Retained buyer tax requires partnership-level income and K-1 accounting and is not supported on this partner path. A prior-year credit producing current Schedule E income belongs in the separate prior-credit field.",
        );
        emit_lines(r, &mut (), claim.fuels.labeled_lines(), |(form_line, amount), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("partnership_{}_fuel_{}", e.id, form_line),
                LineValue::MoneyCents(crate::community::income_amount(d, e.id, amount)),
                &["income.partnership.fuel_tax_credit"],
                "https://www.irs.gov/instructions/i1065sk1",
                msg!("EXPLAIN_PARTNERSHIP_N_FUEL_N"),
            ))
        });
    }
    if let Some(prior) = &data.prior_fuel_credit {
        // A qualified retained comparison limits the recovery;
        // otherwise keep the conservative full-income treatment.
        if !prior.full_prior_tax_benefit && !prior.partial_benefit_applies() {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT",
                    alloc::vec![crate::interview::income_display_arg(e)],
                ),
                context.clone(),
            );
        }
        if !prior.prior_niit_recovery_excluded {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_PARTNER_FUEL_NIIT_RECOVERY_LEFT_OUT",
                    alloc::vec![crate::interview::income_display_arg(e)],
                ),
                context.clone(),
            );
        }
        line_cents(
            r,
            &alloc::format!("partnership_{}_prior_fuel_credit_recovery", e.id),
            prior.taxable_amount(),
            &["income.partnership.prior_fuel_credit"],
            "https://www.irs.gov/instructions/i1040se",
            msg!("EXPLAIN_PARTNERSHIP_N_PRIOR_FUEL_CREDIT_RECOVERY"),
        );
    }
    crate::require_localized!(
        r,
        data.reported_se.is_some(),
        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
        "Partnership self-employment source",
        "ISSUE_ESTABLISH_K_1_BOX_14_CODE_A_FROM_THE_REPORTED_RECORDS_MISSING_EARNI_6EA55DAB",
        "Establish K-1 box 14 code A from the reported records. Missing earnings are not zero and cannot be inferred from box 1 or participation.",
    );
    for source in core::iter::once(&data.regular).chain(data.alternative.iter()) {
        if let Err((key, _message)) = crate::community::partner_pool(d, e.id, data, source) {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                "Partnership activity",
                key,
                _message,
            );
        }
    }
    // The ledger of each tax system carries what the basis
    // and at-risk limits hold back to 2027, one note per class.
    let regular_ledger = crate::community::partner_ledger(d, e.id, data, &data.regular).ok();
    let amt_ledger = data
        .alternative
        .as_ref()
        .and_then(|record| crate::community::partner_ledger(d, e.id, data, record).ok());
    if let Some(ledger) = &regular_ledger {
        crate::k1_loss::carry_notes(
            e,
            true,
            false,
            ledger,
            None,
            data.disposition.as_ref().is_some_and(crate::K1Disposition::ended),
            r,
        );
    }
    if let Some(ledger) = &amt_ledger {
        crate::k1_loss::carry_notes(
            e,
            true,
            true,
            ledger,
            regular_ledger.as_ref(),
            data.disposition.as_ref().is_some_and(crate::K1Disposition::ended),
            r,
        );
    }
    if data.long_term_parts_affected() {
        crate::issue_localized!(
            r,
            &context,
            &source,
            "ISSUE_PARTNER_LONG_TERM_PARTS_LIMITED",
            PartnerK1::LONG_TERM_PARTS_LIMITED,
        );
    }
    if let (Some(regular), Some(alternative), Some(amt_items)) = (
        &regular_ledger,
        &amt_ledger,
        data.alternative.as_ref().and_then(|source| source.source_k1_items(&data.amt_items()).ok()),
    ) && crate::k1_loss::amt_limit_differs(
        regular,
        alternative,
        &data.k1_items(),
        &amt_items,
        d.screening.as_ref().is_none_or(|s| s.section_1256_carryback.is_none()),
    ) {
        crate::issue_localized!(
            r,
            &context,
            &source,
            "ISSUE_K1_AMT_LIMIT_DIFFERS",
            crate::k1_loss::AMT_LIMIT_DIFFERS,
        );
    }
    for item in &data.unentered {
        crate::issue_localized!(r, &context, &source, item.refusal_key(), item.refusal(),);
    }
    for (key, _message) in data.schedule_k3.iter().flat_map(|k3| k3.refusals(false)) {
        crate::issue_localized!(r, &context, &source, key, _message,);
    }
    // The box 11 and box 13 codes, box 19 cash above the
    // basis and the sale or liquidation of the interest.
    let facts = match crate::community::partner_facts(d, e, data) {
        Ok(facts) => facts,
        Err((key, _message)) => {
            crate::issue_localized!(r, &context, &source, key, _message);
            return Ok(());
        }
    };
    crate::k1_totals::apply(&facts, t, r);
    crate::k1_totals::refusals(
        e,
        &data.k1_items(),
        &data.disposition,
        [regular_ledger.as_ref(), amt_ledger.as_ref()],
        r,
    );
    // Boxes 8 and 9a less the losses each tax system's basis and
    // at-risk limits hold back, plus the carries they release
    // (partner basis worksheet 15e and 15f). The AMT
    // difference joins line 2k as for an S corporation.
    let raw_capital = data.separate.as_ref().map_or([0, 0], |items| [items.short, items.long]);
    let regular_capital =
        crate::community::partner_capital(d, e.id, data, &data.regular).unwrap_or(raw_capital);
    let amt_capital = data
        .alternative
        .as_ref()
        .and_then(|record| crate::community::partner_capital(d, e.id, data, record))
        .unwrap_or(regular_capital);
    let capital_gross = data.capital_gains_by_term();
    if !t.restore_k1_capital_gross(
        [regular_capital, amt_capital],
        [capital_gross; 2],
        data.schedule_k3.is_some(),
    ) {
        crate::issue_localized!(
            r,
            &context,
            &source,
            crate::k1_activity::ordinary::RANGE_KEY,
            "Review the document amounts; they must be within the supported input range.",
        );
        return Ok(());
    }
    // A K-1 with a Schedule K-3 has these gains inside the K-3's gross income
    // (Part II line 24), which Form 1116 line 3e takes instead.
    if data.schedule_k3.is_some() {
        t.add_short_outside_line_3e(regular_capital[0]);
        t.add_long_outside_line_3e(regular_capital[1]);
    } else {
        t.add_short(regular_capital[0]);
        t.add_long(regular_capital[1]);
    }
    for term in 0..2 {
        t.schedule_d_k1[term] += regular_capital[term];
        t.investment_gains[term] += regular_capital[term];
        let adjustment = amt_capital[term] - regular_capital[term];
        t.amt_sale_adjustment[term] += adjustment;
        t.amt_investment_gain_adjustment[term] += adjustment;
    }
    if data.separate.is_none() {
        for (name, amount, note) in [
            ("short_gain", regular_capital[0], msg!("EXPLAIN_PARTNERSHIP_N_N_SHORT_GAIN")),
            ("long_gain", regular_capital[1], msg!("EXPLAIN_PARTNERSHIP_N_N_LONG_GAIN")),
        ] {
            if amount != 0 {
                line_cents(
                    r,
                    &alloc::format!("partnership_{}_{name}", e.id),
                    amount,
                    &["income.partnership.separate"],
                    "https://www.irs.gov/instructions/i1040sd",
                    note,
                );
            }
        }
    }
    {
        let source = t.research_items.entry(e.id).or_default();
        source.income += regular_capital.iter().sum::<i64>();
        source.capital_loss += regular_capital.iter().map(|amount| positive(-*amount)).sum::<i64>();
        source.charity[1] += data.allowed_cash_charity();
        if let Some(portfolio) = &data.separate {
            source.income += portfolio.interest + portfolio.dividends;
        }
    }
    if let Some(items) = &data.separate {
        // Portfolio items are not passive and are the same in both
        // tax systems; box 9a already includes 9b and 9c.
        t.interest += items.interest;
        t.dividends += items.dividends;
        t.qualified += items.qualified_dividends;
        t.exempt += items.exempt_interest;
        t.collectible_long += items.collectibles;
        t.unrecaptured_long += items.unrecaptured_partnership + items.unrecaptured_fund;
        t.withholding_other += items.backup_withholding;
        for (name, amount, form, note) in [
            (
                "interest",
                items.interest,
                "https://www.irs.gov/instructions/i1065sk1",
                msg!("EXPLAIN_PARTNERSHIP_N_N_INTEREST"),
            ),
            (
                "dividends",
                items.dividends,
                "https://www.irs.gov/instructions/i1065sk1",
                msg!("EXPLAIN_PARTNERSHIP_N_N_DIVIDENDS"),
            ),
            (
                "short_gain",
                regular_capital[0],
                "https://www.irs.gov/instructions/i1040sd",
                msg!("EXPLAIN_PARTNERSHIP_N_N_SHORT_GAIN"),
            ),
            (
                "long_gain",
                regular_capital[1],
                "https://www.irs.gov/instructions/i1040sd",
                msg!("EXPLAIN_PARTNERSHIP_N_N_LONG_GAIN"),
            ),
            (
                "cash_charity",
                data.allowed_cash_charity(),
                "https://www.irs.gov/instructions/i1040sca",
                msg!("EXPLAIN_PARTNERSHIP_N_N_CASH_CHARITY"),
            ),
            (
                "backup_withholding",
                items.backup_withholding,
                "https://www.irs.gov/instructions/i1065sk1",
                msg!("EXPLAIN_PARTNERSHIP_N_N_BACKUP_WITHHOLDING"),
            ),
            (
                "exempt_interest",
                items.exempt_interest,
                "https://www.irs.gov/instructions/i1065sk1",
                msg!("EXPLAIN_PARTNERSHIP_N_N_EXEMPT_INTEREST"),
            ),
        ] {
            if amount != 0 {
                line_cents(
                    r,
                    &alloc::format!("partnership_{}_{name}", e.id),
                    amount,
                    &["income.partnership.separate"],
                    form,
                    note,
                );
            }
        }
    }
    Ok(())
}

pub(super) fn collect_scorp(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    stored: &crate::ScorpSource,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let data = &crate::ScorpK1::from(stored);
    if let Some(issue) = stored
        .required_issue()
        .or_else(|| stored.page().issue())
        .or_else(|| stored.personal_issue())
        && !matches!(
            issue.key,
            "K1_INVESTMENT_NEEDS_CONFIRMATION" | "K1_HOUSING_NEEDS_CONFIRMATION"
        )
    {
        crate::issue_localized!(
            r,
            &context,
            &source,
            issue.key,
            "Review this K-1's answers against its corrected printed amounts."
        );
    }
    crate::require_localized!(
        r,
        data.investment_property.iter().all(|property| property.own_form().is_some())
            || data.investment_confirmed.is_some(),
        &context,
        &source,
        "K1_INVESTMENT_NEEDS_CONFIRMATION",
        "Answer whether this K-1's investment credit property is confirmed."
    );
    crate::require_localized!(
        r,
        !data.credits.iter().any(|credit| credit.code.housing())
            || data.housing_confirmed.is_some(),
        &context,
        &source,
        "K1_HOUSING_NEEDS_CONFIRMATION",
        "Answer whether you acquired your interest in this K-1's low-income housing credit after 1989."
    );
    data.report(e.id, r);
    for item in &data.unentered {
        crate::issue_localized!(r, &context, &source, item.refusal_key(), item.refusal(),);
    }
    for (key, _message) in data.schedule_k3.iter().flat_map(|k3| k3.refusals(true)) {
        crate::issue_localized!(r, &context, &source, key, _message,);
    }
    let mut gains = [[0; 3]; 2];
    let mut capital = [[0; 4]; 2];
    let mut capital_gross = [[0; 2]; 2];
    let amt_source = data.amt.as_ref().and_then(|amt| amt.source(data).ok());
    let regular_source = crate::scorp_amt::ScorpSource::from(data);
    for (system, source) in [Some(&regular_source), amt_source.as_ref()].into_iter().enumerate() {
        if let Some(source) = source
            && let Ok(basis) = source.basis_result()
        {
            source.report_section1231(&basis, e.id, system == 1, r);
            match source.capital_result_with_events(
                &basis,
                e.id,
                system == 1,
                r,
                crate::community::scorp_events(d, e.id, system == 1).ok().flatten().as_ref(),
            ) {
                Ok(values) => {
                    capital[system] = values;
                    capital_gross[system] = source.capital_gains_by_term();
                }
                Err((key, _message)) => crate::issue_localized!(
                    r,
                    &crate::interview::income_issue_context(e),
                    "S-corporation capital",
                    key,
                    _message,
                ),
            }
            match crate::community::scorp_events(d, e.id, system == 1).and_then(|events| {
                events.map_or_else(|| source.loan_gains(&basis), |value| Ok(value.loans))
            }) {
                Ok(values) => gains[system] = values,
                Err((key, _message)) => crate::issue_localized!(
                    r,
                    &if key == "ISSUE_SCORP_NOTE_ADVANCES_WITH_DATED_REPAYMENTS" {
                        crate::interview::income_issue_context(e).manual()
                    } else {
                        crate::interview::income_issue_context(e)
                    },
                    "S-corporation loan holding period",
                    key,
                    _message,
                ),
            }
        }
    }
    // What the stock and debt basis and the at-risk limit
    // hold back of each class carries to 2027, one note per class.
    let regular_ledger = crate::community::scorp_ledger(d, e.id, data, false).ok();
    let amt_ledger = amt_source
        .as_ref()
        .and_then(|source| crate::community::scorp_ledger(d, e.id, source, true).ok());
    if let Some(ledger) = &regular_ledger {
        crate::k1_loss::carry_notes(
            e,
            false,
            false,
            ledger,
            None,
            data.disposition.as_ref().is_some_and(crate::K1Disposition::ended),
            r,
        );
    }
    if let Some(ledger) = &amt_ledger {
        crate::k1_loss::carry_notes(
            e,
            false,
            true,
            ledger,
            regular_ledger.as_ref(),
            data.disposition.as_ref().is_some_and(crate::K1Disposition::ended),
            r,
        );
    }
    if let (Some(regular), Some(alternative), Some(amt_source)) =
        (&regular_ledger, &amt_ledger, &amt_source)
        && crate::k1_loss::amt_limit_differs(
            regular,
            alternative,
            &data.k1_items(),
            &amt_source.k1_items(),
            d.screening.as_ref().is_none_or(|s| s.section_1256_carryback.is_none()),
        )
    {
        crate::issue_localized!(
            r,
            &context,
            &source,
            "ISSUE_K1_AMT_LIMIT_DIFFERS",
            crate::k1_loss::AMT_LIMIT_DIFFERS,
        );
    }
    // The box 10 and box 12 codes, box 16 code D above
    // stock basis and the sale of the shares (k1_totals.rs).
    crate::k1_totals::apply(&crate::community::scorp_facts(d, e, data), t, r);
    crate::k1_totals::refusals(
        e,
        &data.k1_items(),
        &data.disposition,
        [regular_ledger.as_ref(), amt_ledger.as_ref()],
        r,
    );
    {
        let source = t.research_items.entry(e.id).or_default();
        source.income += data.interest
            + data.dividends
            + capital[0][..3].iter().sum::<i64>()
            + gains[0].iter().sum::<i64>();
        source.capital_loss +=
            [capital[0][0], capital[0][1] + capital[0][2], gains[0][1], gains[0][2]]
                .iter()
                .map(|amount| positive(-*amount))
                .sum::<i64>();
        source.charity[1] += data.allowed_cash_charity();
    }
    if data.allowed_cash_charity() > 0 {
        line_cents(
            r,
            &alloc::format!("scorp_{}_cash_charity", e.id),
            data.allowed_cash_charity(),
            &["income.scorp.cash_charity"],
            "https://www.irs.gov/instructions/i1040sca",
            msg!("EXPLAIN_SCORP_N_CASH_CHARITY"),
        );
    }
    for (system, values) in gains.iter().enumerate() {
        t.scorp_loan_gain[system] += values[0];
    }
    // Form 1116 line 3e counts the loan-repayment gain whole (a loss restored).
    t.ordinary_gain_gross += phase::floor(gains[0][0], 0);
    t.add_short(gains[0][1]);
    t.add_long(gains[0][2]);
    for term in 0..2 {
        let adjustment = gains[1][term + 1] - gains[0][term + 1];
        t.investment_gains[term] += gains[0][term + 1];
        t.amt_sale_adjustment[term] += adjustment;
        t.amt_investment_gain_adjustment[term] += adjustment;
        emit_lines(
            r,
            &mut (),
            [("gain", gains[0][term + 1]), ("amt_gain", gains[1][term + 1])],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "scorp_{}_loan_{}_{}",
                        e.id,
                        if term == 0 { "short" } else { "long" },
                        suffix
                    ),
                    LineValue::MoneyCents(amount),
                    &["income.scorp.loans", "income.scorp.amt.loans"],
                    "https://www.irs.gov/instructions/i7203",
                    msg!("EXPLAIN_SCORP_N_LOAN_N_N"),
                ))
            },
        );
    }
    // The gain already entered short/long above. This is its
    // missing source row only; add_printed_row updates columns,
    // never the financial income or gain pools a second time.
    if let Ok(rows) = crate::community::scorp_loan_rows(d, e.id, data) {
        for note in rows {
            let (line, box_letter) = if note.long { (7, 'F') } else { (3, 'C') };
            let row = crate::capital::Form8949Row {
                entry: e.id,
                row: t.next_row(e.id),
                line,
                box_letter,
                label: MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                    "SOURCE_SCORP_NOTE_REPAYMENT",
                    alloc::vec![
                        crate::interview::income_display_arg(e),
                        MessageArg::Number(note.note as i64)
                    ],
                ))),
                proceeds: note.proceeds,
                basis: note.basis,
                adjustment: 0,
                codes: String::new(),
                summary: false,
            };
            row.emit(r);
            t.add_printed_row(&row);
        }
    }
    t.unrecaptured_long += capital[0][3];
    t.amt_unrecaptured_adjustment += capital[1][3] - capital[0][3];
    t.collectible_long += capital[0][2];
    t.amt_collectible_adjustment += capital[1][2] - capital[0][2];
    // Collectibles are a component of total long-term capital,
    // never an additional gain on top of the K-1 box 8a total.
    let capital = capital.map(|v| [v[0], v[1] + v[2]]);
    if !t.restore_k1_capital_gross(capital, capital_gross, data.schedule_k3.is_some()) {
        crate::issue_localized!(
            r,
            &context,
            &source,
            crate::k1_activity::ordinary::RANGE_KEY,
            "Review the document amounts; they must be within the supported input range.",
        );
        return Ok(());
    }
    if data.schedule_k3.is_some() {
        t.add_short_outside_line_3e(capital[0][0]);
        t.add_long_outside_line_3e(capital[0][1]);
    } else {
        t.add_short(capital[0][0]);
        t.add_long(capital[0][1]);
    }
    for (term, (regular, amt)) in capital[0].iter().zip(&capital[1]).enumerate() {
        t.schedule_d_k1[term] += regular;
        let adjustment = amt - regular;
        t.investment_gains[term] += regular;
        t.amt_sale_adjustment[term] += adjustment;
        t.amt_investment_gain_adjustment[term] += adjustment;
    }
    t.withholding_other += data.withholding;
    t.interest += data.interest;
    t.dividends += data.dividends;
    t.qualified += data.qualified_dividends;
    t.exempt += data.exempt_interest;
    t.private_activity += data.private_activity;
    if data.withholding > 0 {
        line_cents(
            r,
            &alloc::format!("scorp_{}_backup_withholding", e.id),
            data.withholding,
            &["income.scorp.withholding"],
            "https://www.irs.gov/instructions/i1120ssk",
            msg!("EXPLAIN_SCORP_N_BACKUP_WITHHOLDING"),
        );
        r.review_notes.push(note_number(#[cfg(test)] alloc::format!("S-corporation document {}, backup withholding: retain the K-1 for attachment to a filed return, or provide the corporation's name, address, EIN and withholding amount in the required statement. This App does not submit the return.", e.id), "NOTE_SCORP_BACKUP_WITHHOLDING", e.id));
    }
    Ok(())
}

pub(super) fn k1_source_consistency_phase(pass: &Pass<'_>, r: &mut FederalResult) {
    let &Pass { p, allocated_entries, .. } = pass;
    // Preserve independently resolved intermediate worksheets, then verify
    // the source facts required for a complete current return and carryforwards.
    for entry in allocated_entries {
        if let FederalIncome::Scorp { data } = &entry.income {
            let data = &crate::ScorpK1::from(data);
            data.note_no_qbi_statement(entry, r);
            if let Some(key) = data.qbi_left_out() {
                note_once_scoped(
                    r,
                    Message::new(key, alloc::vec![crate::interview::income_display_arg(entry)]),
                    crate::interview::income_issue_context(entry),
                );
            }
            let owner = if entry.owner == crate::DocumentOwner::Yours {
                Some(&p.taxpayer)
            } else {
                p.spouse.as_ref()
            };
            // A K-1 is issued for the decedent's period (Pub. 559 "S Corporation
            // Income"). Only suspended losses need the disposition the
            // confirmation "no other death-related adjustment" excludes: a
            // person who says none applies cannot also have a K-1 carrying
            // them (unknowns::supported drops them when the answer is No).
            crate::require_localized!(
                r,
                owner.is_none_or(|v| {
                    v.died.as_ref().is_none_or(|died| !died.ordinary) || !data.suspended_history()
                }),
                &crate::interview::income_issue_context(entry),
                "Final-return K-1",
                "ISSUE_A_DECEASED_SHAREHOLDER_WITH_SUSPENDED_LOSSES_NEEDS_THE_DEATH_ADJUSTMENT_ANSWER",
                "A deceased shareholder's K-1 with suspended losses cannot also have no other death-related adjustment: say that an adjustment applies on the person's page, and the suspended losses are left out of this final return.",
            );
        }
    }
}
