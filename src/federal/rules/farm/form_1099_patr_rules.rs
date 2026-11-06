// form_1099_patr items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_cooperative(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    v: &crate::Patr1099Source,
) -> Result<(), CollectFailure> {
    let Collect { d, source_claims, .. } = *cx;
    let Source { e, .. } = *src;
    let v = &crate::FederalCooperative::from(v);
    // Accepted PATR receipts/property adjustments already enter
    // their linked C/F/4835 owners. This physical statement is a
    // known annotation, never a second copy of their income.
    t.injured_other_sources.insert(
        e.id,
        crate::cooperative::graph_rule_with_claims(d, source_claims)
            .is_none()
            .then(|| exact::whole(0)),
    );
    t.withholding_1099 += v.withholding;
    crate::cooperative::report_sources_with_claims(d, e, r, source_claims);
    for (suffix, amount, note) in [
        ("distributions", v.distributions(), msg!("EXPLAIN_COOPERATIVE_N_N_DISTRIBUTIONS")),
        ("taxable", v.taxable(), msg!("EXPLAIN_COOPERATIVE_N_N_TAXABLE")),
        ("nonqualified", v.nonqualified(), msg!("EXPLAIN_COOPERATIVE_N_N_NONQUALIFIED")),
        ("patron_reduction", 0, msg!("EXPLAIN_COOPERATIVE_N_N_PATRON_REDUCTION")),
    ] {
        if suffix == "patron_reduction" {
            let has_wages = crate::cooperative::sources_with_claims(d, source_claims).iter().filter(|s|s.parent==e.id)
                .any(|s| d.income.iter().any(|entry| Some(entry.id)==s.activity
                    && matches!(&entry.income,FederalIncome::Business{data} if data.payroll.qbi_wages()>0)));
            if v.qualified_payments == 0 || has_wages {
                continue;
            }
        }
        line_cents(
            r,
            &alloc::format!("cooperative_{}_{suffix}", e.id),
            amount,
            &[&alloc::format!("income.{}", e.id)],
            "https://www.irs.gov/instructions/i1040sf",
            note,
        );
    }
    Ok(())
}
