// form_1099_int items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_interest(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    v: &crate::InterestSource,
) -> Result<(), CollectFailure> {
    let Source { e, owner, .. } = *src;
    let v = &crate::FederalInterest::from(v);
    let parts = crate::investment_statements::components(e).expect("investment source");
    // Box 11/12 bond premium and accrued interest paid on
    // purchase each reduce this payer's own taxable interest,
    // not below zero; a shortfall never borrows from another
    // payer's statement. Box 13 bond premium reduces this
    // payer's own tax-exempt interest the same way. Box 3
    // Treasury/savings-bond interest is ordinary
    // taxable interest with no subtraction of its own.
    t.interest += parts.interest;
    t.early_penalty += v.early_penalty;
    t.injured_early_penalty[owner] += v.early_penalty;
    t.exempt += parts.exempt_interest;
    t.private_activity += parts.private_activity;
    t.withholding_1099 += parts.withholding;
    Ok(())
}

pub(super) fn collect_oid(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    v: &crate::Form1099OidSource,
) -> Result<(), CollectFailure> {
    let Source { e, owner, .. } = *src;
    let v = &crate::FederalOid::from(v);
    let parts = crate::investment_statements::components(e).expect("investment source");
    t.interest += parts.interest;
    t.early_penalty += v.early_penalty;
    t.injured_early_penalty[owner] += v.early_penalty;
    t.exempt += parts.exempt_interest;
    t.private_activity += parts.private_activity;
    t.withholding_1099 += parts.withholding;
    Ok(())
}

pub(super) struct FinalBondInterestIn {
    pub(super) late_investment_sources: [[i64; 2]; 2],
}

pub(super) struct FinalBondInterestOut {
    pub(super) late_investment_sources: [[i64; 2]; 2],
    pub(super) amt_interest_cents: i64,
}

pub(super) fn final_bond_interest_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: FinalBondInterestIn,
) -> core::ops::ControlFlow<(), FinalBondInterestOut> {
    let &Pass { p, .. } = pass;
    let FinalBondInterestIn { mut late_investment_sources } = i;
    for (owner, person) in core::iter::once(&p.taxpayer).chain(p.spouse.iter()).enumerate() {
        if let Some(bonds) = person.died.as_ref().and_then(|v| v.bonds.as_ref()) {
            let deferred = matches!(bonds.reporting, crate::FinalBondReporting::Defer);
            let amount = if deferred { 0 } else { bonds.unreported() };
            if checked_native_add(&mut t.interest, i128::from(amount)).is_none() {
                CollectFailure::Broker(crate::IssueContext::Section(
                    crate::TaxSection::InvestmentSales,
                ))
                .report(r);
                withhold_required_source_result(r);
                return core::ops::ControlFlow::Break(());
            }
            late_investment_sources[0][owner] += amount;
            line_cents(
                r,
                &alloc::format!("final_bond_interest_{owner}"),
                amount,
                &["people.died.bonds"],
                "https://www.irs.gov/publications/p559",
                msg!("EXPLAIN_FINAL_BOND_INTEREST_N"),
            );
            line_cents(
                r,
                &alloc::format!("final_bond_recipient_interest_{owner}"),
                if deferred { bonds.unreported() } else { 0 },
                &["people.died.bonds"],
                "https://www.irs.gov/publications/p559",
                msg!("EXPLAIN_FINAL_BOND_RECIPIENT_INTEREST_N"),
            );
        }
    }

    // All ordinary, native K-1, sale-discount and late bond interest has
    // entered the annual category. Check its independent AMT endpoint before
    // rounding or publishing any dependent calculation (IRS i6251/i4952).
    let amt_interest_cents = match i64::try_from(i128::from(t.interest) + t.k1_amt_interest) {
        Ok(amount) => amount,
        Err(_) => {
            // Only a retained native difference can make this endpoint exceed
            // its regular i64 category; unrelated sources never replace its owner.
            let context = t
                .k1_amt_interest_source
                .as_ref()
                .expect("AMT interest differences retain their actual source");
            crate::issue_localized!(
                r,
                context,
                "K-1 interest",
                crate::k1_activity::ordinary::RANGE_KEY,
                "Review the document amounts; they must be within the supported input range.",
            );
            r.lines.clear();
            r.return_lines.clear();
            return core::ops::ControlFlow::Break(());
        }
    };
    core::ops::ControlFlow::Continue(FinalBondInterestOut {
        late_investment_sources,
        amt_interest_cents,
    })
}
