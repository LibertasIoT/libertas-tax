// form_4684 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_wildfire_relief(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalWildfireRelief,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    // No: no section 139M exclusion; the payment is
    // other income.
    if data.qualified {
        line_cents(
            r,
            &alloc::format!("wildfire_relief_{}_excluded", e.id),
            data.amount,
            &["income.wildfire_relief"],
            "https://uscode.house.gov/view.xhtml?edition=prelim&num=0&req=granuleid%3AUSC-prelim-title26-section139M",
            msg!("EXPLAIN_WILDFIRE_RELIEF_N_EXCLUDED"),
        );
    } else {
        t.taxed_relief += data.amount;
    }
    t.withholding_1099 += data.withholding;
    Ok(())
}

pub(super) fn collect_disaster_relief(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalDisasterRelief,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    // No: no section 139 exclusion; the payment is
    // other income.
    if data.qualified {
        line_cents(
            r,
            &alloc::format!("disaster_relief_{}_excluded", e.id),
            data.amount,
            &["income.disaster_relief"],
            "https://uscode.house.gov/view.xhtml?edition=prelim&num=0&req=granuleid%3AUSC-prelim-title26-section139",
            msg!("EXPLAIN_DISASTER_RELIEF_N_EXCLUDED"),
        );
    } else {
        t.taxed_relief += data.amount;
    }
    t.withholding_1099 += data.withholding;
    if data.withholding > 0 {
        line_cents(
            r,
            &alloc::format!("disaster_relief_{}_withholding", e.id),
            data.withholding,
            &["income.disaster_relief.withholding"],
            "https://www.irs.gov/instructions/i1040gi",
            msg!("EXPLAIN_DISASTER_RELIEF_N_WITHHOLDING"),
        );
    }
    Ok(())
}

pub(super) fn collect_casualty_living_insurance(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalCasualtyLivingInsurance,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    // No: every 2026 insurer payment is taxable.
    match data.confirmed {
        true => {}
        false => note_once_scoped(
            r,
            Message::new(
                "WARN_CASUALTY_LIVING_PAYMENTS_TAXED",
                alloc::vec![crate::interview::income_display_arg(e)],
            ),
            context.clone(),
        ),
    }
    crate::require_localized!(
        r,
        data.prior_taxable_reported == data.expected_prior_taxable(),
        context,
        source,
        "ISSUE_RECONCILE_THE_TAXABLE_PART_OF_PRE_2026_INSURER_PAYMENTS_AGAINST_RET_197CBF4B",
        "Reconcile the taxable part of pre-2026 insurer payments against retained prior returns. No amount is taxable before the main home becomes usable again; after that, prior payments above the whole temporary expense increase must already have been reported. Resolve an inconsistent prior return before computing 2026 inclusion.",
    );
    let increase = data.temporary_increase();
    let taxable = data.included_in_return_year();
    t.casualty_living_insurance += taxable;
    t.withholding_1099 += data.current_withholding();
    let id = alloc::format!("casualty_living_insurance_{}", e.id);
    line_cents(
        r,
        &alloc::format!("{id}_temporary_increase"),
        increase,
        &[
            "income.casualty_living_insurance.actual_living_expenses",
            "income.casualty_living_insurance.normal_living_expenses",
        ],
        "https://www.irs.gov/publications/p547",
        msg!("EXPLAIN_N_TEMPORARY_INCREASE"),
    );
    line_cents(
        r,
        &alloc::format!("{id}_taxable"),
        taxable,
        &[
            "income.casualty_living_insurance.payments",
            "income.casualty_living_insurance.loss_of_use",
            "income.casualty_living_insurance.federal_disaster_area",
        ],
        "https://www.irs.gov/publications/p547",
        msg!("EXPLAIN_N_TAXABLE"),
    );
    if data.current_withholding() > 0 {
        line_cents(
            r,
            &alloc::format!("{id}_withholding"),
            data.current_withholding(),
            &["income.casualty_living_insurance.payments.withholding"],
            "https://www.irs.gov/instructions/i1040gi",
            msg!("EXPLAIN_N_WITHHOLDING"),
        );
    }
    Ok(())
}

pub(super) fn collect_disaster_mitigation(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    _t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalDisasterMitigation,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    // A payment not confirmed under section 139(g) was left out
    // before the rules ran.
    line_cents(
        r,
        &alloc::format!("disaster_mitigation_{}_excluded", e.id),
        data.amount,
        &["income.disaster_mitigation"],
        "https://uscode.house.gov/view.xhtml?edition=prelim&num=0&req=granuleid%3AUSC-prelim-title26-section139",
        msg!("EXPLAIN_DISASTER_MITIGATION_N_EXCLUDED"),
    );
    Ok(())
}
