// form_1099_div items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Both a standalone distribution and a populated DIV use this existing owner.
pub(super) fn capital_return(
    t: &mut Totals,
    r: &mut FederalResult,
    e: &FederalIncomeEntry,
    data: &crate::CapitalReturn1099Div,
    prefix: &str,
) {
    let context = crate::interview::income_issue_context(e);
    #[cfg(test)]
    let source = alloc::format!("Income: {}", e.label);
    if data.needs_liquidation_status() {
        crate::require_localized!(
            r,
            false,
            &context,
            &source,
            "DIVIDEND_LIQUIDATION_STAGE_NEEDED",
            "Identify whether the liquidation is interim or final from the issuer's statement.",
        );
        return;
    }
    let kind = &data.kind;
    let amount = &data.amount;
    let property = &data.property;
    let basis = &data.basis;
    let term = &data.term;

    crate::require_localized!(
        r,
        property.is_empty()
            || matches!(
                kind,
                CapitalDistributionKind::InterimLiquidation
                    | CapitalDistributionKind::FinalLiquidation
            ),
        &context,
        &source,
        "ISSUE_PROPERTY_DISTRIBUTIONS_REQUIRE_THE_TAXABLE_LIQUIDATION_PATH_ORDINAR_AAB4FBFE",
        "Property distributions require the taxable liquidation path; ordinary noncash corporate distributions need separate treatment.",
    );
    let amount = *amount + property.iter().map(|p| p.value).sum::<i64>();
    emit_lines(r, &mut (), property.iter().enumerate(), |(index, asset), _, _| {
        Some(calculation_row(
            None,
            &alloc::format!("{prefix}_property_{index}_basis"),
            LineValue::MoneyCents(asset.value),
            &["income.return_of_capital.property"],
            "https://www.irs.gov/publications/p550",
            Message::new(
                "EXPLAIN_DISTRIBUTION_N_PROPERTY_N_BASIS",
                alloc::vec![MessageArg::Text(asset.label.clone())],
            ),
        ))
    });
    let gain = kind.gain(amount, *basis);
    t.investment_gains[usize::from(*term == GainTerm::Long)] += gain;
    // Form 8949 box C or F: only a gain, or a final liquidation's loss, is a row.
    match term {
        GainTerm::Short => t.add_short(gain),
        GainTerm::Long => t.add_long(gain),
    }
    if gain != 0 || *kind == CapitalDistributionKind::FinalLiquidation {
        let (line, box_letter) = match term {
            GainTerm::Short => (3, 'C'),
            GainTerm::Long => (7, 'F'),
        };
        t.row(r, e, line, box_letter, [amount, *basis, gain - (amount - *basis)], "");
    }
    emit_lines(
        r,
        &mut (),
        [
            (
                "basis_reduction",
                phase::cap(amount, [*basis]),
                msg!("EXPLAIN_DISTRIBUTION_N_N_BASIS_REDUCTION"),
            ),
            (
                "remaining_basis",
                if *kind == CapitalDistributionKind::FinalLiquidation {
                    0
                } else {
                    positive(basis - amount)
                },
                msg!("EXPLAIN_DISTRIBUTION_N_N_REMAINING_BASIS"),
            ),
            ("capital_gain", gain, msg!("EXPLAIN_DISTRIBUTION_N_N_CAPITAL_GAIN")),
        ],
        |(suffix, value, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}_{suffix}"),
                LineValue::MoneyCents(value),
                &["income.return_of_capital"],
                "https://www.irs.gov/publications/p550",
                explanation,
            ))
        },
    );
}

// A standalone return-of-capital distribution: collect_with_claims arm.
pub(super) fn collect_return_of_capital(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    (kind, amount, property, basis, term): (
        &CapitalDistributionKind,
        &i64,
        &Vec<LiquidationProperty>,
        &i64,
        &GainTerm,
    ),
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    capital_return(
        t,
        r,
        e,
        &crate::CapitalReturn1099Div {
            kind: *kind,
            amount: *amount,
            property: property.clone(),
            basis: *basis,
            term: *term,
            special: false,
        },
        &alloc::format!("distribution_{}", e.id),
    );
    Ok(())
}

// A Form 2439 statement: collect_with_claims arm.
pub(super) fn collect_undistributed_gain(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::UndistributedGainSource,
    pending: Option<&mut PendingShare>,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, .. } = *src;
    let v = &crate::UndistributedGain::from(data);
    // A statement with box 1c was left out before the rules ran
    // (unknowns.rs). Box 1a is Schedule D line 11.
    t.add_long(v.gain);
    t.schedule_d_line_11 += v.gain;
    t.investment_gains[1] += v.gain;
    t.collectible_long += v.collectibles;
    t.unrecaptured_long += v.unrecaptured;
    if let Some(frame) = pending {
        frame.fund = Some((v.small_business, v.section_1202_exclusion));
    } else {
        fund_section_1202(
            t,
            r,
            e,
            v.small_business,
            v.section_1202_exclusion,
            None,
            separate_return(d),
        );
    }
    t.fund_tax_paid += v.tax_paid;
    line_cents(
        r,
        &alloc::format!("fund_{}_basis_increase", e.id),
        v.gain - v.tax_paid,
        &["income.undistributed_gain"],
        "https://www.irs.gov/publications/p550",
        msg!("EXPLAIN_FUND_N_BASIS_INCREASE"),
    );
    Ok(())
}

// A Form 1099-DIV: collect_with_claims arm.
pub(super) fn collect_dividend(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    v: &crate::DividendSource,
    pending: Option<&mut PendingShare>,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    for (index, lot) in v.capital_returns.iter().enumerate() {
        if lot.special {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_RETURN_OF_CAPITAL_LEFT_OUT",
                    alloc::vec![crate::interview::income_display_arg(e)],
                ),
                context.clone(),
            );
        } else {
            capital_return(t, r, e, lot, &alloc::format!("distribution_{}_lot_{}", e.id, index));
        }
    }

    let parts = crate::investment_statements::components(e).expect("investment source");
    t.dividends += parts.ordinary_dividends;
    t.reit_dividends += parts.section199a;
    t.retain_reit_owner(e, parts.section199a);
    t.qualified += parts.qualified_dividends;
    t.add_long(parts.capital_distributions);
    t.schedule_d_distributions += parts.capital_distributions;
    t.investment_gains[1] += parts.capital_distributions;
    t.collectible_long += parts.collectible_distributions;
    t.unrecaptured_long += parts.unrecaptured_distributions;
    if let Some(frame) = pending {
        frame.fund = Some((parts.section_1202_gain, v.section_1202_exclusion));
    } else {
        fund_section_1202(
            t,
            r,
            e,
            parts.section_1202_gain,
            v.section_1202_exclusion,
            None,
            separate_return(d),
        );
    }
    t.withholding_1099 += parts.withholding;
    // Box 12 exempt-interest dividends join line 2a like
    // 1099-INT box 8; the box 13 private-activity subset joins
    // the AMT preference like 1099-INT box 9.
    t.exempt += parts.exempt_interest;
    t.private_activity += parts.private_activity;
    Ok(())
}

impl Totals {
    pub(crate) fn retain_reit_owner(&mut self, entry: &FederalIncomeEntry, cents: i64) {
        if let Some((weights, denominator)) = entry.accepted_assignment().owner_weights() {
            for (owner, weight) in weights.into_iter().enumerate() {
                self.injured_reit_dividends[owner] += exact::whole(cents)
                    * exact::ratio(weight, denominator).expect("ownership denominator");
            }
        } else if cents != 0 {
            self.injured_reit_owner_unavailable = true;
        }
    }
}
