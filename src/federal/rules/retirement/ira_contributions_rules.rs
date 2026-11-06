// ira_contributions items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The originating helper determines its cause; current IRA fields have a
// correction owner while advanced retained worksheets remain readable only.
pub(super) fn ira_issue_context(key: &str) -> crate::IssueContext {
    use crate::TaxSection::{IraContributions, Setup};
    let context = crate::IssueContext::Section(IraContributions);
    match key {
        "ISSUE_IRA_SPOUSE_SITUATION" => context.corrective(Setup),
        // The transfer is a Form 1099-Q; its earlier-year total is on the
        // IRA contributions page.
        "ISSUE_IRA_QTP_ROLLOVERS" => context.corrective(crate::TaxSection::OtherIncome),
        "ISSUE_PENSION_IRA_SPOUSE_COVERAGE"
        | "ISSUE_PENSION_ROTH_COORDINATION"
        | "ISSUE_PENSION_IRA_PHASEOUT_COORDINATION" => context.manual(),
        _ => context,
    }
}

pub(super) struct IraExcessIn<'a, 'd> {
    pub(super) ira_facts: &'a Option<alloc::borrow::Cow<'d, FederalIras>>,
    pub(super) qtp_transfers: &'a crate::ira::QtpTransfers,
    pub(super) qtp_excess_by_entry: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) compensation: [i64; 2],
    pub(super) home_route: Option<bool>,
    pub(super) before_social: i64,
    pub(super) social: i64,
    pub(super) exempt: i64,
    pub(super) excluded: i64,
    pub(super) adoption_excluded: i64,
    pub(super) conversion: i64,
    pub(super) before_rental: i64,
    pub(super) half_se: i64,
    pub(super) adjusted_business: i64,
    pub(super) property_business_income: i64,
}

pub(super) fn ira_excess_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    partner_pass: &PartnerPass,
    nol_run: &crate::nol::NolRun,
    i: IraExcessIn<'_, '_>,
) -> i64 {
    let &Pass {
        d, allocated_entries, office_context, p, s, a, social_election, j, qcd, basis, ..
    } = pass;
    let source_claims = pass.context.source_claims;
    let IraExcessIn {
        ira_facts,
        qtp_transfers,
        qtp_excess_by_entry,
        compensation,
        home_route,
        before_social,
        social,
        exempt,
        excluded,
        adoption_excluded,
        conversion,
        before_rental,
        half_se,
        adjusted_business,
        property_business_income,
    } = i;
    let mut roth_limits = [0; 2];
    if let Some(f) = ira_facts.as_deref() {
        // Conversion income is excluded only from the Roth contribution MAGI.
        let mut roth_magi = before_social + social + excluded + adoption_excluded - conversion;
        let (start, width) = crate::ira::roth_band(s);
        if conversion > 0 && roth_magi > start + width {
            // Worksheet 2-1's optional refiguring uses conversion-free income
            // for AGI-sensitive rental losses and Appendix B Social Security.
            // These provisional calculations never replace tax-return values.
            let mut worksheet = r.clone();
            let modified_magi = before_rental - conversion + half_se + adoption_excluded;
            let rental =
                crate::extended_rules::rentals_for_home_with_selected_section179_with_claims(
                    d,
                    modified_magi,
                    &mut worksheet,
                    false,
                    &crate::extended_rules::HomeUseContext::provisional(
                        d,
                        [modified_magi; 2],
                        home_route,
                    ),
                    &crate::care::CareDeductions::default(),
                    partner_pass.section179.as_deref(),
                    Some(office_context),
                    source_claims,
                );
            let excess = excess_business_loss_amount(
                adjusted_business,
                &rental,
                property_business_income,
                j,
            )
            .1;
            let before_social = before_rental - conversion
                + rental.personal_expense_adjustment
                + rental.income
                + excess;
            let refigured = before_social
                + adoption_excluded
                + excluded
                + social_election.taxable(
                    dollars(t.social),
                    before_social,
                    exempt + excluded + adoption_excluded,
                    s.filing_status(),
                    s.lived_apart_all_year(),
                );
            if worksheet.issues.len() == r.issues.len() {
                roth_magi = phase::cap(roth_magi, [refigured]);
            }
        }
        line(
            r,
            "roth_contribution_magi",
            roth_magi,
            &["income", "adjustments.iras"],
            "https://www.irs.gov/publications/p590a",
            msg!("EXPLAIN_ROTH_CONTRIBUTION_MAGI"),
        );
        // A Form 172 carryover pass refigures income and deductions; its
        // hypothetical MAGI does not re-test the actual Roth contribution.
        match crate::ira::validate_roth(f, p, s, compensation, roth_magi, qtp_transfers) {
            Ok(limits) => roth_limits = limits,
            Err((key, _message)) => {
                if !nol_run.modify_income {
                    crate::issue_localized!(
                        r,
                        &ira_issue_context(key),
                        "IRA contributions",
                        key,
                        _message,
                    );
                }
            }
        }
    }
    // Form 5329 Parts III and IV: line 11 is each owner's 2026 traditional
    // IRA distributions included in income, line 20 each owner's gross Roth
    // IRA withdrawals; the 2026 excess comes from the App's own limits. The
    // Form 172 pass's hypothetical MAGI does not re-test the Roth excess.
    let mut rollover_excess = [0; 2];
    for (id, amount) in qtp_excess_by_entry {
        if let Some(e) = allocated_entries.iter().find(|e| e.id == *id) {
            rollover_excess[usize::from(e.owner == crate::DocumentOwner::Spouses)] += amount;
        }
    }
    let ira_excess_owners =
        ira_facts.as_deref().filter(|_| !nol_run.modify_income).map_or([0; 2], |f| {
            let mut ira_taxable = [0; 2];
            let mut roth_withdrawn = [0; 2];
            for e in allocated_entries {
                let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
                match &e.income {
                    FederalIncome::Retirement { data: stored }
                        if {
                            let data = &crate::FederalRetirement::from(stored);
                            data.kind == crate::RetirementKind::Ira
                        } =>
                    {
                        let data = &crate::FederalRetirement::from(stored);
                        ira_taxable[owner] += basis.taxable(e.id, data) - qcd.for_entry(e.id);
                    }
                    FederalIncome::Roth { data: stored }
                        if {
                            let data = &crate::RothWithdrawal::from(stored);
                            data.inherited == crate::InheritedLink::No
                        } =>
                    {
                        let data = &crate::RothWithdrawal::from(stored);
                        roth_withdrawn[owner] += data.gross
                    }
                    _ => {}
                }
            }
            crate::ira::excess_tax_with_difficulty_of_care(
                f,
                p,
                s,
                compensation,
                core::array::from_fn(|i| t.medicaid_excluded[i] + t.medicaid_in_box1[i]),
                roth_limits,
                ira_taxable.map(dollars),
                roth_withdrawn.map(dollars),
                [Owner::Taxpayer, Owner::Spouse].map(|owner| {
                    a.ira_basis
                        .iter()
                        .find(|b| b.owner == owner)
                        .map(crate::IraBasis::form_5329_value_exact)
                }),
                qtp_transfers,
                rollover_excess,
                r,
            )
        });
    let ira_excess_tax = ira_excess_owners.iter().sum::<i64>();
    t.injured_accounts.fixed_tax(ira_excess_owners);
    ira_excess_tax
}

pub(super) struct IraDeductionIn<'a> {
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) compensation: [i64; 2],
    pub(super) before_social: i64,
    pub(super) exempt: i64,
    pub(super) excluded: i64,
    pub(super) adoption_excluded: i64,
}

pub(super) struct IraDeductionOut<'d> {
    pub(super) ira_magi: i64,
    pub(super) qtp_transfers: crate::ira::QtpTransfers,
    pub(super) carry_distributions: [i64; 2],
    pub(super) ira_facts: Option<alloc::borrow::Cow<'d, crate::FederalIras>>,
    pub(super) covered_ira: bool,
    pub(super) pending_health_ira: bool,
    pub(super) ira: crate::ira::IraResult,
}

pub(super) fn ira_deduction_phase<'d>(
    pass: &Pass<'d>,
    r: &mut FederalResult,
    t: &Totals,
    partner_pass: &PartnerPass,
    input: IraDeductionIn<'_>,
) -> IraDeductionOut<'d> {
    let d = pass.d;
    let p = pass.p;
    let s = pass.s;
    let pension = pass.pension;
    let contribution_result = pass.contribution_result;
    let allocated_entries = pass.allocated_entries;
    let IraDeductionIn { health, compensation, before_social, exempt, excluded, adoption_excluded } =
        input;
    // Pub. 590-A Appendix B Worksheet 1 computes the taxable Social Security benefits in IRA
    // MAGI from all the benefits received under the regular method, before the IRA deduction;
    // the worksheet has no lump-sum election, which only lowers line 6b. The benefits are then
    // recomputed after the deduction. No fixed-point iteration.
    let ira_magi = before_social
        + adoption_excluded
        + excluded
        + social_taxable(
            dollars(t.social),
            before_social,
            exempt + excluded + adoption_excluded,
            s.filing_status(),
            s.lived_apart_all_year(),
        );
    let qtp_transfers = crate::ira::QtpTransfers::of(&d.income);
    // Form 5329 line 11 before the Form 8606 allocation, for the carried-excess room the
    // deduction takes in (line 10): the owner's traditional IRA distributions in income.
    let mut carry_distributions = [0; 2];
    for e in allocated_entries {
        if let Some(data) = e.income.retirement()
            && data.kind == crate::RetirementKind::Ira
            && !data.outside_form_8606()
        {
            carry_distributions[usize::from(e.owner == crate::DocumentOwner::Spouses)] +=
                phase::floor(data.taxable_income() - data.converted_amount(), 0);
        }
    }
    let ira_facts = crate::ira::facts(d, &qtp_transfers);
    let covered_ira = ira_facts.as_deref().is_some_and(|facts| {
        let traditional = facts.taxpayer.traditional > 0
            || facts.spouse.as_ref().is_some_and(|v| v.traditional > 0);
        let covered = facts.taxpayer.covered_at_work
            || facts.spouse.as_ref().is_some_and(|v| v.covered_at_work);
        traditional && covered
    });
    let pending_health_ira =
        !health.specified.is_empty() && covered_ira && partner_pass.specials.ira.is_none();
    // The selected Iterative Step 6 amount is a monetary worksheet input,
    // not its rounded Schedule 1 display. Pub. 974 Special Instructions item 2
    // feeds that same amount to the covered-IRA owner before its $10-up
    // phaseout limit. Keep the ordinary/noncoupled IRA path unchanged.
    let ira_magi_cents = if partner_pass.iterative && covered_ira && !health.specified.is_empty() {
        ira_magi * 100 + health.deduction * 100
            - crate::kernel::ratio::exact::rounded(&health.work_deduction())
                .expect("bounded selected health deduction")
    } else {
        ira_magi * 100
    };
    if let Some(f) = ira_facts.as_deref()
        && let Err((key, _message)) = crate::ira::validate_pension_contributions(
            f,
            p,
            s,
            compensation,
            ira_magi_cents,
            pension.saver,
            &qtp_transfers,
        )
    {
        crate::issue_localized!(r, &ira_issue_context(key), "IRA contributions", key, _message,);
    }
    let ira = if let Some(result) = &partner_pass.specials.ira {
        result.clone()
    } else if let Some(result) = contribution_result {
        result.clone()
    } else {
        match ira_facts.as_deref() {
            Some(f) => match crate::ira::calculate_with_difficulty_of_care(
                f,
                p,
                s,
                crate::ira::Compensation {
                    taxable: compensation,
                    difficulty_of_care: core::array::from_fn(|i| {
                        t.medicaid_excluded[i] + t.medicaid_in_box1[i]
                    }),
                },
                ira_magi_cents,
                &qtp_transfers,
                carry_distributions,
            ) {
                Ok(v) => v,
                Err((key, _message)) => {
                    crate::issue_localized!(
                        r,
                        &ira_issue_context(key),
                        "IRA contributions",
                        key,
                        _message,
                    );
                    crate::ira::IraResult::default()
                }
            },
            None => crate::ira::IraResult::default(),
        }
    };
    IraDeductionOut {
        ira_magi,
        qtp_transfers,
        carry_distributions,
        ira_facts,
        covered_ira,
        pending_health_ira,
        ira,
    }
}
