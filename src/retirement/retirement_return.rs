// Retirement distribution calculations and their reported form amounts.
use crate::dates::valid_birth;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::fraction;
use crate::line_emission::emit as emit_lines;
use crate::reservist_sources::ReserveTerm;
use crate::retirement::{
    ContractDischarge, PlanDistribution, Reemployment, RetirementException, RetirementTaxable,
    UnemploymentQualification,
};
use crate::retirement_adjustment_sources::IraBasis;
use crate::{
    DistributionCodes, FederalDraft, FederalIncome, FederalIncomeEntry, FederalPerson,
    FederalResult, Owner, RetirementKind, RetirementTreatment,
};

// The rows the early-distribution stage works on (Form 5329 Part I): every Form 1099-R record,
// and each Roth IRA row whose Form 5329 line 1 amount has an exception of its own, as the IRA
// distribution it is for that stage (Form 8606 line 25c earnings plus the conversion recapture
// are its taxable amount; the early-distribution exceptions apply to a Roth IRA as to any IRA).
pub(crate) fn retirement_rows<'a>(
    d: &'a FederalDraft,
    basis: &BasisResult,
) -> alloc::vec::Vec<(
    &'a crate::FederalIncomeEntry,
    alloc::borrow::Cow<'a, crate::FederalRetirement>,
)> {
    let mut rows = alloc::vec::Vec::new();
    for e in &d.income {
        match &e.income {
            FederalIncome::Retirement { data: source } => {
                let data = crate::FederalRetirement::from(source);
                if data.taxable.is_some() && data.ordinary_distribution.is_some() {
                    rows.push((e, alloc::borrow::Cow::Owned(data)));
                }
            }
            FederalIncome::Roth { data: source } => {
                let w = &crate::RothWithdrawal::from(source);
                let (Some(staged), Some(exception)) = (basis.roth.staged.get(&e.id), &w.exception)
                else {
                    continue;
                };
                // The funding of an HSA is subtracted again by BasisResult::taxable.
                let funding = basis.hsa_funding.get(&e.id).copied().unwrap_or(0);
                let record = crate::FederalRetirement {
                    form_boxes: Default::default(),
                    trump: false,
                    box_2a: staged + funding,
                    box_3: 0,
                    box_6: 0,
                    box_8a: 0,
                    box_8b_percent: None,
                    box_9a_percent: None,
                    ira: (RetirementKind::Ira).ira(),
                    disability: crate::DisabilityIncome::None,
                    kind: RetirementKind::Ira,
                    disability_wages: crate::DisabilityWages::None,
                    gross: phase::floor(w.gross - w.rolled_over - funding, 0),
                    taxable: Some(RetirementTaxable::Payer),
                    withholding: 0,
                    box7: 0,
                    ordinary_distribution: Some(RetirementTreatment::Exception {
                        distributed: w.distributed,
                        facts: exception.clone(),
                        second: None,
                        early_simple: w.simple_early(),
                    }),
                    disaster: crate::DisasterDistribution::No,
                    after_death: None,
                    box_10: 0,
                    box_10_basis: None,
                    box_9b: None,
                };
                rows.push((e, alloc::borrow::Cow::Owned(record)));
            }
            _ => {}
        }
    }
    rows
}

#[derive(Default)]
pub(crate) struct GrossExceptions {
    pub amounts: alloc::collections::BTreeMap<i64, (i64, i64)>,
}

// Limits on qualified distributions consume gross dollars, while Form 5329
// relief covers only their taxable part. Expense-based exceptions are different.
#[derive(Clone)]
struct GrossExceptionClaim {
    id: i64,
    gross: i64,
    taxable: i64,
    eligible: i64,
    simple: bool,
    proportional: bool,
}
impl GrossExceptionClaim {
    fn relief(&self, amount: i64, r: &mut FederalResult, out: &mut GrossExceptions) -> i64 {
        let amount = phase::floor(phase::cap(amount, [self.gross]), 0);
        crate::require_localized!(
            r,
            amount == 0
                || amount == self.gross
                || self.taxable == 0
                || self.taxable == self.gross
                || self.proportional,
            &crate::IssueContext::income_id(self.id).manual(),
            "Retirement exception allocation",
            "ISSUE_A_PARTIAL_EXCEPTION_NEEDS_A_TRADITIONAL_IRA_ALLOCATION_THE_QUALIFIE_C205A7E3",
            "A partial exception needs a traditional IRA allocation, the qualified pre-annuity withdrawal worksheet, or a complete qualified employer-plan discharge. Other non-IRA basis methods and overlapping public-safety premium exclusions need source-specific allocation.",
        );
        let taxable = if self.gross == 0 { 0 } else { fraction(self.taxable, amount, self.gross) };
        out.amounts.insert(self.id, (amount, taxable));
        emit_lines(
            r,
            &mut (),
            [
                ("gross", amount, msg!("EXPLAIN_RETIREMENT_N_QUALIFIED_N_GROSS")),
                ("taxable", taxable, msg!("EXPLAIN_RETIREMENT_N_QUALIFIED_N_TAXABLE")),
            ],
            |(suffix, value, note), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("retirement_{}_qualified_{suffix}", self.id),
                    LineValue::MoneyCents(value),
                    &["income.retirement"],
                    "https://www.irs.gov/instructions/i5329",
                    note,
                ))
            },
        );
        taxable
    }
}
// Preserve the existing choice to apply a shared limit where it saves the most
// additional tax. Compare exact tax relief per gross dollar; stable IDs break ties.
fn gross_claim_order(a: &GrossExceptionClaim, b: &GrossExceptionClaim) -> core::cmp::Ordering {
    let benefit = |v: &GrossExceptionClaim| i128::from(v.taxable) * if v.simple { 25 } else { 10 };
    (benefit(b) * i128::from(a.gross.max(1)))
        .cmp(&(benefit(a) * i128::from(b.gross.max(1))))
        .then_with(|| a.id.cmp(&b.id))
}

fn gross_exception_limit(
    mut claims: alloc::vec::Vec<GrossExceptionClaim>,
    limit: i64,
    r: &mut FederalResult,
    out: &mut GrossExceptions,
) -> i64 {
    claims.sort_by(gross_claim_order);
    let mut remaining = limit;
    for claim in claims {
        let amount = crate::kernel::carry::native_consume_step(claim.eligible, &mut remaining);
        claim.relief(amount, r, out);
    }
    crate::kernel::carry::native_balance(limit, remaining)
}

fn domestic_abuse_limit(
    mut claims: alloc::vec::Vec<(alloc::string::String, GrossExceptionClaim)>,
    plans: &alloc::collections::BTreeMap<alloc::string::String, (i64, i64)>,
    limit: i64,
    r: &mut FederalResult,
    out: &mut GrossExceptions,
) -> i64 {
    claims.sort_by(|a, b| gross_claim_order(&a.1, &b.1));
    let mut remaining = limit;
    let mut plan_remaining: alloc::collections::BTreeMap<_, _> = plans
        .iter()
        .map(|(name, (balance, prior))| {
            (name.as_str(), phase::floor(crate::kernel::ratio::trunc_i64(*balance, 2) - prior, 0))
        })
        .collect();
    for (plan, claim) in claims {
        let available = plan_remaining.get_mut(plan.as_str()).expect("recorded plan");
        let amount = phase::cap(phase::cap(remaining, [*available]), [claim.eligible]);
        remaining = crate::kernel::carry::native_balance(remaining, amount);
        *available = crate::kernel::carry::native_balance(*available, amount);
        claim.relief(amount, r, out);
    }
    crate::kernel::carry::native_balance(limit, remaining)
}

pub(crate) fn gross_exceptions(
    d: &FederalDraft,
    basis: &BasisResult,
    qcd: &QcdResult,
    r: &mut FederalResult,
) -> GrossExceptions {
    let mut out = GrossExceptions::default();
    let mut birth_claims: alloc::collections::BTreeMap<
        (usize, alloc::string::String),
        (u32, i64, alloc::vec::Vec<GrossExceptionClaim>),
    > = alloc::collections::BTreeMap::new();
    let mut abuse_prior = [None, None];
    let mut abuse_plans: [alloc::collections::BTreeMap<alloc::string::String, (i64, i64)>; 2] =
        Default::default();
    let mut abuse_claims: [alloc::vec::Vec<(alloc::string::String, GrossExceptionClaim)>; 2] =
        Default::default();
    let mut home_prior = [None, None];
    let mut home_claims: [alloc::vec::Vec<GrossExceptionClaim>; 2] = Default::default();
    let rows = retirement_rows(d, basis);
    for (e, v) in &rows {
        let v: &crate::FederalRetirement = v;
        let Some(RetirementTreatment::Exception { facts, second, early_simple, .. }) =
            &v.ordinary_distribution
        else {
            continue;
        };
        #[cfg(test)]
        let source = alloc::format!("Income: {}", e.label);
        let taxable = basis.taxable(e.id, v) - basis.converted(e.id, v) - qcd.for_entry(e.id);
        let claim = GrossExceptionClaim {
            id: e.id,
            gross: v.gross,
            taxable,
            eligible: v.gross,
            simple: *early_simple,
            proportional: v.kind == RetirementKind::Ira
                || ((matches!(v.taxable, Some(RetirementTaxable::QualifiedWithdrawal { .. }))
                    || (v.kind == RetirementKind::Pension
                        && matches!(v.taxable, Some(RetirementTaxable::FullDischarge { .. }))))
                    && basis.public_safety.excluded(e.id) == 0),
        };
        for fact in core::iter::once(facts).chain(second.iter()) {
            match fact {
                RetirementException::DomesticAbuse {
                    plan,
                    balance,
                    prior_used,
                    prior_plan_used,
                    ..
                } => {
                    let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
                    crate::require_localized!(
                        r,
                        abuse_prior[owner].is_none_or(|v| v == *prior_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FOR_ONE_OWNER_MUST_USE_THE_SAME_PRE_2026_QUAL_77D1C43F",
                        "Domestic-abuse claims for one owner must use the same pre-2026 qualified gross total.",
                    );
                    abuse_prior[owner] = Some(*prior_used);
                    let opening = abuse_plans[owner]
                        .entry(plan.clone())
                        .or_insert((*balance, *prior_plan_used));
                    crate::require_localized!(
                        r,
                        *opening == (*balance, *prior_plan_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FROM_ONE_PLAN_MUST_USE_THE_SAME_OPENING_BALAN_956F1ABF",
                        "Domestic-abuse claims from one plan must use the same opening balance and pre-2026 plan usage.",
                    );
                    abuse_claims[owner].push((plan.clone(), claim.clone()));
                }
                RetirementException::Emergency { balance, .. } => {
                    claim.relief(phase::bound(balance - 100000, 0, 100000), r, &mut out);
                }
                RetirementException::Levy { amount } => {
                    claim.relief(*amount, r, &mut out);
                }
                RetirementException::TerminalIllness { .. } => {
                    claim.relief(v.gross, r, &mut out);
                }
                RetirementException::BirthAdoption { child, event, prior_used, .. } => {
                    let entry = birth_claims
                        .entry((
                            usize::from(e.owner == crate::DocumentOwner::Spouses),
                            child.clone(),
                        ))
                        .or_insert((*event, *prior_used, alloc::vec::Vec::new()));
                    crate::require_localized!(
                        r,
                        entry.0 == *event && entry.1 == *prior_used,
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_CLAIMS_FOR_ONE_PARENT_AND_CHILD_MUST_USE_THE_SAME_BIRTH_ADOPTION_DA_F05B51E0",
                        "Claims for one parent and child must use the same birth/adoption date and opening prior-distribution total.",
                    );
                    entry.2.push(claim.clone());
                }
                RetirementException::FirstHome { costs, prior_used, .. } => {
                    let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
                    crate::require_localized!(
                        r,
                        home_prior[owner].is_none_or(|v| v == *prior_used),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_FIRST_HOME_CLAIMS_FOR_ONE_IRA_OWNER_MUST_USE_THE_SAME_PRE_2026_LIFE_BBE3640F",
                        "First-home claims for one IRA owner must use the same pre-2026 lifetime total.",
                    );
                    home_prior[owner] = Some(*prior_used);
                    home_claims[owner].push(GrossExceptionClaim {
                        eligible: phase::cap(*costs, [v.gross]),
                        ..claim.clone()
                    });
                }
                _ => (),
            }
        }
    }
    for ((owner, child), (_, prior, claims)) in birth_claims {
        let used = gross_exception_limit(claims, 500000 - prior, r, &mut out);
        crate::line_emission::line_cents(
            r,
            &alloc::format!("birth_adoption_used_{owner}_{child}"),
            prior + used,
            &["income.retirement"],
            "https://www.irs.gov/irb/2020-38_IRB",
            msg!("EXPLAIN_BIRTH_ADOPTION_USED_N_N"),
        );
    }
    for owner in 0..2 {
        if let Some(prior) = abuse_prior[owner] {
            crate::require_localized!(
                r,
                abuse_plans[owner].values().map(|(_, used)| used).sum::<i64>() <= prior,
                &crate::IssueContext::owner(
                    crate::TaxSection::Retirement,
                    if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
                ),
                "Domestic-abuse claim history",
                "ISSUE_EARLIER_QUALIFIED_AMOUNTS_ASSIGNED_TO_CURRENT_PLANS_CANNOT_EXCEED_T_7D6D3EC7",
                "Earlier qualified amounts assigned to current plans cannot exceed the owner's total earlier qualified amount.",
            );
            let used = domestic_abuse_limit(
                core::mem::take(&mut abuse_claims[owner]),
                &abuse_plans[owner],
                crate::year_constants::DOMESTIC_ABUSE_DISTRIBUTION_LIMIT_CENTS - prior,
                r,
                &mut out,
            );
            crate::line_emission::line_cents(
                r,
                if owner == 0 {
                    "domestic_abuse_used_taxpayer"
                } else {
                    "domestic_abuse_used_spouse"
                },
                prior + used,
                &["income.retirement"],
                "https://www.irs.gov/irb/2024-28_IRB",
                msg!("EXPLAIN_DOMESTIC_ABUSE_USED_TAXPAYER"),
            );
        }
    }
    for owner in 0..2 {
        if let Some(prior) = home_prior[owner] {
            let used = gross_exception_limit(
                core::mem::take(&mut home_claims[owner]),
                phase::floor(
                    crate::year_constants::FIRST_HOME_DISTRIBUTION_LIMIT_CENTS
                        - prior
                        - basis.roth.home_used[owner],
                    0,
                ),
                r,
                &mut out,
            );

            crate::line_emission::line_cents(
                r,
                if owner == 0 { "first_home_used_taxpayer" } else { "first_home_used_spouse" },
                prior + used,
                &["income.retirement"],
                "https://www.irs.gov/publications/p590b",
                msg!("EXPLAIN_FIRST_HOME_USED_TAXPAYER"),
            );
        }
    }
    out
}

// Native PartI total keeps its existing cents API; owner form taxes are dollars.
pub(crate) struct EarlyDistributionTaxes {
    pub total_cents: i64,
    pub owners: [i64; 2],
}

pub(crate) fn early_tax(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &BasisResult,
    qcd: &QcdResult,
    agi: i64,
) -> EarlyDistributionTaxes {
    let care = crate::care_distribution::calculate(d, r, basis);
    // Roth rows: Form 5329 line 1 takes the Form 8606 line 25c earnings, the recapture of
    // 2022-2026 conversions and the line 20 first-home slice (which line 2 excepts as 09); the
    // 25% rate is a Roth SIMPLE IRA's inside its first two years. A row with an exception of its
    // own goes through the stage below (retirement_rows).
    let mut taxable_before = [0, 0];
    let mut after_exception = [[0, 0], [0, 0]];
    for owner in 0..2 {
        taxable_before[owner] =
            basis.roth.penalized[owner] + basis.roth.simple[owner] + basis.roth.slice[owner];
        after_exception[owner] = [basis.roth.penalized[owner], basis.roth.simple[owner]];
    }
    let mut emergency_seen = [false, false];
    let mut medical_expenses = 0;
    let mut medical_taxable = [[0, 0]; 2];
    let mut insurance_exception = false;
    let rows = retirement_rows(d, basis);
    for (e, v) in &rows {
        let v: &crate::FederalRetirement = v;
        // A qualified disaster distribution's taxable part is not an early
        // distribution on Form 5329 line 1 (Form 8915-F line 6).
        let taxable = basis.taxable(e.id, v)
            - basis.converted(e.id, v)
            - qcd.for_entry(e.id)
            - basis.disaster.get(&e.id).copied().unwrap_or(0)
            - if basis.home_repaid.contains(&e.id) {
                // Form 8606 line 7 already left the repayment out of this row's taxable amount.
                0
            } else {
                crate::disaster::home_repaid(v, basis.taxable(e.id, v))
            };
        let taxable = if let Some(RetirementTaxable::LegacyAnnuity { ref data }) = v.taxable {
            // Section 72(q)(2)(F): earnings allocated to pre-1982 investment.
            phase::floor(taxable - data.parts(v.gross)[1], 0)
        } else {
            taxable
        };
        // Pretax money rolled into a Roth account is taxed whole, but only the taxable amount
        // kept is an early distribution (sections 402A(c)(4)(B), 408A(d)(3)(A)(ii)); a
        // designated Roth account row already takes the rollover out of its earnings first.
        let taxable = match v.ordinary_distribution {
            Some(RetirementTreatment::Rollover {
                destination: crate::RolloverDestination::Roth,
                amount,
                ..
            }) if v.box7 & DistributionCodes::CODE_B.0 == 0 => phase::floor(taxable - amount, 0),
            _ => taxable,
        };
        // Form 5329 line 1 adds the recapture amount of an in-plan Roth rollover within five
        // years to a designated Roth account distribution: the part of box 10 allocated to the
        // Taxable column of the Recapture Allocation Chart, not the part that was not taxable
        // when rolled in (Pub. 575, Figuring your recapture amount; i5329 line 1).
        let taxable = if v.box7 & DistributionCodes::CODE_B.0 != 0 && v.box_10 > 0 {
            #[cfg(test)]
            let source = alloc::format!("Income: {}", e.label);
            crate::require_localized!(
                r,
                v.box_10_basis.is_some(),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                &source,
                "ISSUE_BOX_10_BASIS_UNKNOWN",
                "The part of box 10 that was not taxable when rolled in decides the recapture amount for the 10% additional tax; without it that tax is not calculated.",
            );
            taxable + phase::cap(v.box_10 - v.box_10_basis.unwrap_or(0), [v.gross])
        } else {
            taxable
        };
        let mut exempt = 0;
        let rate = match &v.ordinary_distribution {
            // A code S distribution kept, in a SIMPLE IRA's first two years, is taxed at 25%
            // (section 72(t)(6)).
            Some(RetirementTreatment::Rollover { early: true, .. })
                if v.box7 & DistributionCodes::CODE_S.0 != 0 =>
            {
                25
            }
            Some(RetirementTreatment::Early)
            | Some(RetirementTreatment::Rollover { early: true, .. })
            | Some(RetirementTreatment::Conversion { early: true, .. }) => 10,
            Some(RetirementTreatment::EarlySimple) => 25,
            Some(RetirementTreatment::Exception { distributed, facts, second, early_simple }) => {
                // A prohibited transaction (code 5) is deemed a distribution on January 1 (IRC
                // 408(e)(2)(B)): every date-tested exception is tested on that date, whatever
                // date the page holds.
                let deemed = 20260101;
                let distributed =
                    if v.box7 & DistributionCodes::CODE_5.0 != 0 { &deemed } else { distributed };
                let person: Option<&FederalPerson> = d.people.as_ref().and_then(|p| {
                    if e.owner == crate::DocumentOwner::Yours {
                        Some(&p.taxpayer)
                    } else {
                        p.spouse.as_ref()
                    }
                });
                #[cfg(test)]
                let source = alloc::format!("Income: {}", e.label);
                crate::require_localized!(
                    r,
                    !early_simple || v.kind == RetirementKind::Ira,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    &source,
                    "ISSUE_CONFIRM_THE_ELIGIBLE_ACCOUNT_OR_ANNUITY_SOURCE_DATES_AND_EXCEPTION_1865F21C",
                    "Confirm the eligible account or annuity, source dates and exception requirements; Roth recapture and unentered repayment adjustments need another worksheet.",
                );
                // No on the exception's own confirmation: the
                // exception is not applied, so the additional tax applies.
                // The exception, and a second one on the same distribution (Form 5329 line 2
                // enters 99 for two or more): each is tested and counted once.
                for fact in core::iter::once(facts).chain(second.iter()) {
                    let answer = fact.answer();
                    let applied = answer;
                    let valid = !applied
                        || match fact {
                            RetirementException::LongTermCare { .. } => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::Governmental457 { rollover_taxable, .. } => {
                                v.kind == RetirementKind::Pension && *rollover_taxable <= taxable
                            }
                            RetirementException::EsopDividends
                            | RetirementException::FederalPhasedRetirement => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::AutomaticEnrollment {
                                first_contribution,
                                elected,
                                window_days,
                                ..
                            } => {
                                *first_contribution <= *elected
                                    && *elected <= *distributed
                                    && crate::dates::days_between(*first_contribution, *elected)
                                        <= *window_days as i32
                            }
                            RetirementException::DomesticAbuse { event, .. } => {
                                v.kind != RetirementKind::Nonqualified
                                    && *event <= *distributed
                                    && *distributed < *event + 10000
                            }
                            RetirementException::Emergency { history, .. } => {
                                let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
                                crate::require_localized!(
                                    r,
                                    !emergency_seen[owner],
                                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                                    &source,
                                    "ISSUE_ONLY_ONE_ANNUAL_EMERGENCY_DISTRIBUTION_PER_PERSON_IS_PERMITTED_ENTE_4A862D65",
                                    "Only one annual emergency distribution per person is permitted; enter traditional IRA repayments in the owner's basis record.",
                                );
                                emergency_seen[owner] = true;
                                v.kind != RetirementKind::Nonqualified && history.permits()
                            }
                            RetirementException::BirthAdoption { event, .. } => {
                                v.kind != RetirementKind::Nonqualified
                                    && *event <= *distributed
                                    && *distributed < *event + 10000
                            }
                            RetirementException::DomesticRelations => {
                                v.kind == RetirementKind::Pension
                            }
                            RetirementException::Reservist { ordered, through, term, .. } => {
                                *ordered > 20010911
                                    && *ordered <= *distributed
                                    && *distributed <= *through
                                    && match term {
                                        ReserveTerm::Indefinite => true,
                                        ReserveTerm::Days { days } => *days >= 180,
                                    }
                            }
                            RetirementException::FirstHome { paid_on, .. } => {
                                v.kind == RetirementKind::Ira
                                    && *paid_on >= *distributed
                                    && crate::dates::days_between(*distributed, *paid_on) <= 120
                            }
                            RetirementException::Medical { .. } => true,
                            RetirementException::UnemploymentInsurance {
                                benefit_year,
                                weeks,
                                reemployment,
                                ..
                            } => {
                                v.kind == RetirementKind::Ira
                                    && *weeks >= 12
                                    && (*benefit_year == distributed / 10000
                                        || *benefit_year + 1 == distributed / 10000)
                                    && match reemployment {
                                        Reemployment::None => true,
                                        Reemployment::Started { date } => {
                                            crate::dates::days_between(*date, *distributed) <= 60
                                        }
                                    }
                            }
                            RetirementException::Education { .. } => v.kind == RetirementKind::Ira,
                            RetirementException::Disability { began, .. } => *began <= *distributed,
                            RetirementException::Death { died } => *died <= *distributed,
                            RetirementException::TerminalIllness { certified } => {
                                *certified <= *distributed
                            }
                            RetirementException::ImmediateAnnuity {
                                purchased, started, ..
                            } => {
                                v.kind == RetirementKind::Nonqualified
                                    && *purchased <= *started
                                    && *started <= *purchased + 10000
                                    && *started <= *distributed
                                    && match &v.taxable {
                                        Some(RetirementTaxable::GeneralAnnuity { data }) => {
                                            data.started == *started
                                        }
                                        Some(RetirementTaxable::VariableAnnuity { data }) => {
                                            data.started == *started
                                        }
                                        Some(RetirementTaxable::DeferredWithdrawal { .. })
                                        | Some(RetirementTaxable::LegacyAnnuity { .. }) => false,
                                        _ => true,
                                    }
                            }
                            RetirementException::Separation { separated, work } => {
                                v.kind == RetirementKind::Pension
                                    && *separated <= *distributed
                                    && person
                                        .is_some_and(|p| work.qualifies(p.birth_date, *separated))
                            }
                            RetirementException::Levy { .. } => {
                                v.kind != RetirementKind::Nonqualified
                            }
                            RetirementException::PeriodicPayments { first_payment, .. } => {
                                *first_payment <= *distributed
                            }
                            // The payer's code 1 was wrong: the owner had turned 59½ by the payment
                            // (Form 5329 line 2, exception 12).
                            // Code 5 is deemed paid on January 1 whatever the page holds: an owner
                            // not yet 59½ that day simply gets no exception.
                            RetirementException::ReachedAge => {
                                v.box7 & DistributionCodes::CODE_5.0 != 0
                                    || person.is_some_and(|p| {
                                        *distributed >= crate::roth_source::age_half(p.birth_date)
                                    })
                            }
                        };
                    // Pub. 590-B, Recapture tax for changes in distribution method: a change in a
                    // series of substantially equal periodic payments adds the earlier years' 10%
                    // with interest, which is not calculated.
                    crate::require_localized!(
                        r,
                        !(applied
                            && matches!(
                                fact,
                                RetirementException::PeriodicPayments { modified: true, .. }
                            )),
                        &crate::interview::income_issue_context(e),
                        &source,
                        "ISSUE_SEPP_RECAPTURE",
                        "A change in a series of substantially equal periodic payments before the later of five years after the first payment and age 59½ adds the earlier years' 10% additional tax with interest, which is not calculated.",
                    );
                    crate::require_localized!(
                        r,
                        valid,
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        &source,
                        "ISSUE_THE_EXCEPTION_S_DATE_ACCOUNT_TYPE_EMPLOYMENT_ELIGIBILITY_OR_SEPARAT_9E1325F3",
                        "The exception's date, account type, employment eligibility or separation age/service does not qualify for this distribution.",
                    );
                    let part = if !applied {
                        0
                    } else {
                        match fact {
                            RetirementException::LongTermCare { .. } => {
                                care.get(&e.id).copied().unwrap_or(0)
                            }
                            RetirementException::Governmental457 { rollover_taxable, .. } => {
                                phase::floor(taxable - rollover_taxable, 0)
                            }
                            RetirementException::DomesticAbuse { .. }
                            | RetirementException::Emergency { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::BirthAdoption { .. }
                            | RetirementException::FirstHome { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::Medical { expenses } => {
                                medical_expenses += expenses;
                                0
                            }
                            RetirementException::UnemploymentInsurance {
                                premiums,
                                qualification,
                                ..
                            } => {
                                let amount = phase::cap(*premiums, [taxable]);
                                crate::line_emission::line_cents(
                                    r,
                                    &alloc::format!("insurance_exception_{}", e.id),
                                    amount,
                                    &["income.retirement.ordinary_distribution"],
                                    "https://www.irs.gov/irb/2026-06_IRB",
                                    match qualification {
                                        UnemploymentQualification::Received { .. } => {
                                            msg!("EXPLAIN_INSURANCE_EXCEPTION_N")
                                        }
                                        UnemploymentQualification::SelfEmployed { .. } => {
                                            msg!("EXPLAIN_INSURANCE_EXCEPTION_N_2")
                                        }
                                    },
                                );
                                amount
                            }
                            RetirementException::Education {
                                education,
                                room_board,
                                allowance,
                                school_housing,
                                half_time,
                                assistance,
                                ..
                            } => {
                                let housing = if *half_time {
                                    phase::cap(
                                        *room_board,
                                        [phase::floor(*allowance, *school_housing)],
                                    )
                                } else {
                                    0
                                };
                                phase::cap(
                                    phase::floor(education + housing - assistance, 0),
                                    [taxable],
                                )
                            }
                            RetirementException::Levy { .. } => {
                                basis.gross.amounts.get(&e.id).map_or(0, |v| v.1)
                            }
                            RetirementException::ReachedAge
                                if !person.is_some_and(|p| {
                                    *distributed >= crate::roth_source::age_half(p.birth_date)
                                }) =>
                            {
                                0
                            }
                            _ => taxable,
                        }
                    };
                    exempt += part;
                }
                exempt = phase::cap(exempt, [taxable]);
                if *early_simple { 25 } else { 10 }
            }
            _ => 0,
        };
        // Exception 12 (i5329 line 2; IRC 72(t)(2)(A)(i)): a distribution a payer coded 1 or S (or
        // J) that the owner received at 59½ or older is never early, whatever other exception
        // was entered. The page asks no date for a plain early distribution, so it applies
        // when the owner was 59½ before 2026 began; an entered exception carries its own date
        // (January 1 for code 5).
        let received = match &v.ordinary_distribution {
            Some(RetirementTreatment::Exception { distributed, .. }) => {
                if v.box7 & DistributionCodes::CODE_5.0 != 0 { 20260101 } else { *distributed }
            }
            _ => 20260101,
        };
        let age_exempt = rate > 0
            && d.people.as_ref().is_some_and(|p| {
                let person = if e.owner == crate::DocumentOwner::Yours {
                    Some(&p.taxpayer)
                } else {
                    p.spouse.as_ref()
                };
                person.is_some_and(|p| crate::roth_source::age_half(p.birth_date) <= received)
            });
        if age_exempt {
            exempt = taxable;
        }
        let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
        if rate > 0 {
            taxable_before[owner] += taxable;
        }
        // Medical expenses above 7.5% of AGI are allocated once over the return (below); what a
        // second exception on the same distribution already covers is not theirs.
        let medical = match &v.ordinary_distribution {
            Some(RetirementTreatment::Exception { facts, second, .. }) => core::iter::once(facts)
                .chain(second.iter())
                .any(|f| matches!(f, RetirementException::Medical { .. })),
            _ => false,
        };
        if !age_exempt && medical {
            if rate > 0 {
                medical_taxable[owner][usize::from(rate == 25)] += taxable - exempt;
            }
        } else {
            insurance_exception |= match &v.ordinary_distribution {
                Some(RetirementTreatment::Exception { facts, second, .. }) => {
                    core::iter::once(facts)
                        .chain(second.iter())
                        .any(|f| matches!(f, RetirementException::UnemploymentInsurance { .. }))
                }
                _ => false,
            };
            if rate > 0 {
                after_exception[owner][usize::from(rate == 25)] += taxable - exempt;
            }
        }
    }
    crate::require_localized!(
        r,
        medical_expenses == 0 || !insurance_exception,
        &crate::IssueContext::Section(crate::TaxSection::Retirement).manual(),
        "Retirement medical expenses",
        "ISSUE_COMBINED_MEDICAL_EXPENSE_AND_UNEMPLOYMENT_INSURANCE_EXCEPTIONS_NEED_05A37B66",
        "Combined medical-expense and unemployment-insurance exceptions need allocation of overlapping premiums before calculation.",
    );
    let medical_remaining = medical_exception_allocation(medical_taxable, medical_expenses, agi);
    for owner in 0..2 {
        for rate in 0..2 {
            after_exception[owner][rate] += medical_remaining[owner][rate];
        }
    }
    for (owner, amount) in basis.repayment_excluded.iter().enumerate() {
        taxable_before[owner] = phase::floor(taxable_before[owner] - amount, 0);
    }
    // Each spouse files a separate Form 5329. Aggregate source cents before
    // rounding each form line, then add the separately rounded taxes.
    let mut total = 0;
    let mut owners = [0; 2];
    for owner in 0..2 {
        if taxable_before[owner] == 0 {
            continue;
        }
        let [taxable, exceptions, subject, tax] =
            early_tax_lines(taxable_before[owner], after_exception[owner]);
        total += tax;
        owners[owner] = tax;
        emit_lines(
            r,
            &mut (),
            [("taxable", taxable), ("exceptions", exceptions), ("subject", subject), ("tax", tax)],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "early_{}_{suffix}",
                        if owner == 0 { "taxpayer" } else { "spouse" }
                    ),
                    LineValue::MoneyDollars(amount),
                    &["income.retirement", "income.roth"],
                    "https://www.irs.gov/instructions/i5329",
                    msg!("EXPLAIN_EARLY_FILER"),
                ))
            },
        );
    }
    EarlyDistributionTaxes { total_cents: total * 100, owners }
}

fn age_seventy_half(birth: u32) -> u32 {
    let mut year = birth / 10000 + 70;
    let mut month = birth / 100 % 100 + 6;
    if month > 12 {
        month -= 12;
        year += 1;
    }
    let mut day = birth % 100;
    while day > 28 && !valid_birth(year * 10000 + month * 100 + day) {
        day -= 1;
    }
    year * 10000 + month * 100 + day
}
// Returns excluded IRA distributions and nonexcluded cash gifts, in source cents.
#[derive(Default)]
pub(crate) struct QcdResult {
    pub excluded: i64,
    pub excluded_by_owner: [i64; 2],
    pub cash_gifts: i64,
    qualified: [i64; 2],
    by_entry: alloc::collections::BTreeMap<i64, i64>,
    exclusions: alloc::collections::BTreeMap<i64, Option<i64>>,
}
impl QcdResult {
    pub(crate) fn exclusion_for_entry(&self, id: i64) -> Option<i64> {
        self.exclusions.get(&id).copied().unwrap_or(Some(0))
    }
    pub(crate) fn for_entry(&self, id: i64) -> i64 {
        self.by_entry.get(&id).copied().unwrap_or(0)
    }
}
pub(crate) fn qualified_charity(
    d: &FederalDraft,
    r: &mut FederalResult,
    contributions: Option<&crate::ira::IraResult>,
    funding: &crate::hsa_funding::FundingResult,
) -> QcdResult {
    use crate::kernel::ratio::{dollars, positive};
    use crate::line_emission::line;
    let Some(a) = &d.adjustments else {
        return QcdResult::default();
    };
    let mut excluded = 0;
    let mut cash = 0;
    let mut carry = 0;
    let mut result = QcdResult::default();
    for (index, q) in a.qualified_charity.iter().enumerate() {
        let owner = usize::from(q.owner == Owner::Spouse);
        let person = d
            .people
            .as_ref()
            .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        let mut ira = exact::whole(0);
        let mut pretax = true;
        for e in &d.income {
            if e.owner == crate::DocumentOwner::from(q.owner)
                && let Some(v) = e.income.retirement()
                && v.kind == crate::RetirementKind::Ira
                && !v.outside_form_8606()
            {
                ira += (exact::whole(v.taxable_income())
                    - exact::whole(v.converted_amount())
                    - exact::whole(funding.for_entry(e.id)))
                .max(exact::whole(0));
                pretax &= (v.taxable == Some(crate::RetirementTaxable::Payer)
                    && v.box_2a == v.gross)
                    || matches!(v.ordinary_distribution, Some(RetirementTreatment::Rollover { amount, .. }) if amount == v.gross && (v.taxable == Some(crate::RetirementTaxable::Payer) && v.box_2a == 0));
            }
        }
        let current_contribution = a.entered_iras().map_or(0, |f| {
            if owner == 0 {
                f.taxpayer.traditional
            } else {
                f.spouse.iter().map(|v| v.traditional).sum()
            }
        });
        // Start with full deductibility; a second candidate may use the IRA
        // worksheet result. The caller independently verifies either candidate.
        let nondeductible = contributions.map_or(0, |v| v.nondeductible[owner]);
        let current_deduction = positive(dollars(current_contribution) - nondeductible) * 100;
        let payments: i64 = q.payments.iter().map(|p| p.amount).sum();
        let basis = a.ira_basis.iter().find(|b| b.owner == q.owner);
        // Section 408(d)(8)(D): charitable transfers use the aggregate pretax
        // portion first, before applying ordinary section 72 basis allocation.
        let pretax_capacity = basis.map_or_else(
            || ira.clone(),
            |b| {
                (b.adjusted_year_end_exact(d) - b.line_2_exact()
                    + exact::whole(funding.basis_used[owner])
                    - incoming_plan_basis(d, q.owner)
                    - b.reservist_basis_exact()
                    - exact::whole(available_contribution_basis(
                        b,
                        current_contribution,
                        contributions.map_or(0, |v| v.absorbed[owner]),
                        nondeductible,
                    )) * exact::whole(100)
                    + d.income
                        .iter()
                        .filter_map(|e| {
                            if e.owner == crate::DocumentOwner::from(q.owner)
                                && let Some(data) = e.income.retirement()
                                && data.kind == crate::RetirementKind::Ira
                            {
                                Some(
                                    exact::whole(data.gross) - exact::whole(data.rolled_amount())
                                        + exact::whole(data.outstanding_rollover())
                                        - exact::whole(funding.for_entry(e.id)),
                                )
                            } else {
                                None
                            }
                        })
                        .sum::<Exact>())
                .max(exact::whole(0))
            },
        );
        let personal_payments: i64 =
            q.payments.iter().filter(|p| p.inherited.is_none()).map(|p| p.amount).sum();
        let qualified = exact::rounded(&exact::whole(personal_payments).min(pretax_capacity))
            .expect("QCD exclusion bounded by actual native payments");
        result.qualified[owner] += qualified;
        let mut entries: alloc::vec::Vec<_> =
            d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(q.owner)).collect();
        entries.sort_by_key(|e| e.id);
        let mut remaining_qualified = qualified;
        for e in entries {
            if let Some(data) = e.income.retirement()
                && data.kind == crate::RetirementKind::Ira
            {
                let assigned = crate::kernel::carry::native_consume_step(
                    positive(
                        data.taxable_income() - data.converted_amount() - funding.for_entry(e.id),
                    ),
                    &mut remaining_qualified,
                );
                result.by_entry.insert(e.id, assigned);
            }
        }
        crate::require_localized!(
            r,
            (pretax || basis.is_some())
                && exact::whole(personal_payments) <= ira
                && person.is_some()
                && !a.qualified_charity[..index].iter().any(|v| v.owner == q.owner),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "QCD",
            "ISSUE_USE_ONE_COMPLETE_QCD_RECORD_PER_FILER_RECONCILE_GIFTS_TO_ENTERED_IR_167E808F",
            "Use one QCD record per filer and reconcile gifts to entered IRA distributions. Personal after-tax sources need the owner-level basis worksheet; inherited basis stays in its linked income entry.",
        );
        crate::require_localized!(
            r,
            person.is_some_and(|p| {
                q.payments.iter().all(|v| v.date >= age_seventy_half(p.birth_date))
            }),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "QCD",
            "ISSUE_EACH_TRUSTEE_PAYMENT_MUST_BE_MADE_ON_OR_AFTER_THE_RECEIVING_FILER_S_17F2FD62",
            "Each trustee payment must be made on or after the receiving filer's actual age-70½ date, including gifts from inherited IRAs.",
        );
        // Inherited basis never joins personal IRAs or another decedent's group.
        // Notice 2007-7 Q&A-37 permits QCDs at the beneficiary's age 70½.
        let mut inherited_payments = alloc::collections::BTreeMap::<i64, i64>::new();
        for payment in &q.payments {
            if let Some(id) = payment.inherited {
                *inherited_payments.entry(id).or_default() += payment.amount;
            }
        }
        let mut inherited_qualified = 0;
        for (id, amount) in inherited_payments {
            let source = d
                .income
                .iter()
                .find(|e| e.id == id && e.owner == crate::DocumentOwner::from(q.owner));
            if let Some(crate::FederalIncomeEntry {
                income: FederalIncome::InheritedIra { data },
                ..
            }) = source
                && !data.roth()
            {
                let group = crate::inherited_ira::gross(d, id);
                crate::require_localized!(
                    r,
                    amount <= phase::floor(group - funding.for_inherited(id), 0)
                        && q.payments
                            .iter()
                            .filter(|p| p.inherited == Some(id))
                            .all(|p| p.date >= data.died),
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    "QCD",
                    "ISSUE_INHERITED_GIFTS_MUST_FOLLOW_THE_ORIGINAL_OWNER_S_DEATH_AND_BE_INCLU_434B63F8",
                    "Inherited gifts must follow the original owner's death and be included in the linked group's gross beneficiary distributions.",
                );
                let available = phase::floor(group - funding.for_inherited(id), 0);
                let capacity = data.basis().map_or(available, |b| {
                    positive(
                        available + b.year_end - b.prior
                            + funding.inherited_basis_used.get(&id).copied().unwrap_or(0),
                    )
                });
                let assigned = phase::cap(phase::cap(amount, [available]), [capacity]);
                result.by_entry.insert(id, assigned);
                inherited_qualified += assigned;
            } else {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    "QCD",
                    "ISSUE_LINK_EACH_INHERITED_GIFT_TO_AN_EXISTING_INHERITED_TRADITIONAL_IRA_I_4140777A",
                    "Link each inherited gift to an existing inherited traditional IRA income entry belonging to this filer.",
                );
            }
        }
        let capped =
            phase::cap(qualified + inherited_qualified, [crate::year_constants::QCD_LIMIT_CENTS]);
        let adjustment = q.prior_deductions + current_deduction;
        let allowed = positive(capped - adjustment);
        let sources: alloc::vec::Vec<_> = d
            .income
            .iter()
            .filter(|e| {
                e.owner == crate::DocumentOwner::from(q.owner) && result.for_entry(e.id) > 0
            })
            .collect();
        for entry in &sources {
            let source_exclusion = if allowed == 0 {
                Some(0)
            } else if allowed == qualified + inherited_qualified {
                Some(result.for_entry(entry.id))
            } else if sources.len() == 1 {
                Some(allowed)
            } else {
                None
            };
            result.exclusions.insert(entry.id, source_exclusion);
        }
        let remaining = crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            adjustment, capped,
        )
        .get();
        excluded += allowed;
        result.excluded_by_owner[owner] += allowed;
        cash += payments - allowed;
        carry += remaining;
        line(
            r,
            if owner == 0 { "qcd_taxpayer_exclusion" } else { "qcd_spouse_exclusion" },
            dollars(allowed),
            &["adjustments.qualified_charity", "income.retirement"],
            "https://www.irs.gov/publications/p590b",
            msg!("EXPLAIN_QCD_TAXPAYER_EXCLUSION"),
        );
        line(
            r,
            if owner == 0 {
                "qcd_taxpayer_adjustment_carry"
            } else {
                "qcd_spouse_adjustment_carry"
            },
            dollars(remaining),
            &["adjustments.qualified_charity"],
            "https://www.irs.gov/publications/p590b",
            msg!("EXPLAIN_QCD_TAXPAYER_ADJUSTMENT_CARRY"),
        );
    }
    emit_lines(
        r,
        &mut (),
        [
            ("qcd_exclusion", excluded, msg!("EXPLAIN_QCD_EXCLUSION")),
            ("qcd_cash_gifts", cash, msg!("EXPLAIN_QCD_CASH_GIFTS")),
            ("qcd_adjustment_carryforward", carry, msg!("EXPLAIN_QCD_ADJUSTMENT_CARRYFORWARD")),
        ],
        |(id, value, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(dollars(value)),
                &["adjustments.qualified_charity"],
                "https://www.irs.gov/publications/p590b",
                explanation,
            ))
        },
    );
    result.excluded = excluded;
    result.cash_gifts = cash;
    result
}

// The calculation and form owners report Dollars as native Cents. A failure
// belongs to the actual reported amount, not an exact source subtotal.
#[derive(Clone, Copy)]
pub(crate) struct IraAmountIssue;
impl IraAmountIssue {
    pub(crate) fn reported(dollars: i64) -> Result<i64, Self> {
        dollars.checked_mul(100).map(|_| dollars).ok_or(Self)
    }
    pub(crate) fn report(self, r: &mut FederalResult) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "IRA_REPAYMENT_AMOUNT_UNREPRESENTABLE",
            "An IRA basis or repayment amount cannot be represented in the native reported result. Review the original distribution and repayment sources.",
        );
        crate::federal_rules::withhold_required_source_result(r);
    }
}
#[derive(Default)]
pub(crate) struct BasisResult {
    pub gross: GrossExceptions,
    pub regular_gross: [i64; 2],
    pub regular_taxable: [i64; 2],
    pub repayment_excluded: [i64; 2],
    pub inherited_basis_used: alloc::collections::BTreeMap<i64, i64>,
    pub inherited: crate::inherited_ira::InheritedResult,
    pub public_safety: crate::public_safety::PublicSafetyResult,
    pub hsa_funding: alloc::collections::BTreeMap<i64, i64>,
    pub inherited_hsa_funding: alloc::collections::BTreeMap<i64, i64>,
    pub roth: crate::roth::RothResult,
    // Taxable part of each 2026 qualified disaster distribution (Form 8915-F).
    pub disaster: alloc::collections::BTreeMap<i64, i64>,
    // The Part IV home-purchase rows whose repaid amount Form 8606 line 7 left out: their
    // allocated taxable amount is already net of the repayment.
    pub home_repaid: alloc::collections::BTreeSet<i64>,
    taxable: alloc::collections::BTreeMap<i64, i64>,
    converted: alloc::collections::BTreeMap<i64, i64>,
}

// The funding source is an actual accepted payment, while its inherited
// basis belongs to the existing beneficiary/decedent group. Never substitute
// an annual group for the payment or create a second IRA pool identity.
pub(crate) struct IraFundingSource<'a> {
    pub entry: &'a FederalIncomeEntry,
    pub inherited: Option<(i64, &'a crate::FederalInheritedIra)>,
}

pub(crate) fn ira_funding_source(d: &FederalDraft, id: i64) -> Option<IraFundingSource<'_>> {
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    if !entry
        .income
        .retirement()
        .is_some_and(|data| matches!(data.kind, RetirementKind::Ira | RetirementKind::Inherited))
        && !matches!(entry.income, FederalIncome::Roth { .. })
    {
        return None;
    }
    let inherited = match crate::inherited_ira::link_of(d, entry) {
        None => None,
        Some(crate::inherited_ira::Link::Record(group)) => {
            let group_entry = d.income.iter().find(|entry| entry.id == group)?;
            let FederalIncome::InheritedIra { data } = &group_entry.income else {
                return None;
            };
            Some((group, data))
        }
        Some(_) => return None,
    };
    // An inherited payment without its canonical inherited link is not an
    // own-account source merely because it has a retirement gross amount.
    if entry
        .income
        .retirement()
        .is_some_and(|data| data.kind == RetirementKind::Inherited && inherited.is_none())
    {
        return None;
    }
    Some(IraFundingSource { entry, inherited })
}
impl BasisResult {
    pub(crate) fn converted(&self, id: i64, v: &crate::FederalRetirement) -> i64 {
        self.converted.get(&id).copied().unwrap_or(v.converted_amount())
    }
    pub(crate) fn taxable(&self, id: i64, v: &crate::FederalRetirement) -> i64 {
        self.taxable.get(&id).copied().unwrap_or_else(|| v.taxable_income())
            - self.public_safety.excluded(id)
            - self.hsa_funding.get(&id).copied().unwrap_or(0)
    }
}
// After-tax employer-plan rollovers deposited into a traditional IRA this year
// increase Form 8606 basis without being regular annual contributions.
fn incoming_plan_basis(d: &FederalDraft, owner: Owner) -> Exact {
    d.income
        .iter()
        .filter_map(|entry| {
            if entry.owner != crate::DocumentOwner::from(owner) {
                return None;
            }
            let data = entry.income.retirement()?;
            if data.kind == RetirementKind::Pension
                && matches!(
                    data.taxable,
                    Some(RetirementTaxable::QualifiedWithdrawal {
                        data: PlanDistribution { eligible: true, .. }
                    }) | Some(RetirementTaxable::FullDischarge {
                        data: ContractDischarge { eligible: true, .. }
                    })
                )
                && matches!(
                    data.ordinary_distribution,
                    Some(RetirementTreatment::Rollover {
                        destination: crate::RolloverDestination::Ira,
                        ..
                    })
                )
            {
                Some(exact::whole(data.rolled_basis()))
            } else {
                None
            }
        })
        .sum()
}

// Current-year basis excludes nondeductible contributions paid the following year.
// Both the QCD pretax-first rule and ordinary allocation use this same amount.
fn available_contribution_basis(
    b: &IraBasis,
    contribution: i64,
    absorbed: i64,
    nondeductible: i64,
) -> i64 {
    use crate::kernel::ratio::dollars;
    let later = dollars(b.paid_next_year);
    // Form5329 line10 applies an existing prior-year excess to this year. It
    // is already in the IRA, not a payment made next year. IraResult includes
    // it in permitted contributions and Form8606 line1; retain its source cents
    // until the same whole-dollar rounding as that owner.
    let earlier = exact::dollars(&(exact::whole(contribution) + exact::whole(absorbed)))
        .expect("contributions and absorbed excess retain their native source bounds")
        - later;
    // Favorable order: nondeductible basis uses amounts already in the IRA first.
    let nondeductible_later = phase::floor(nondeductible - earlier, 0);
    nondeductible - nondeductible_later
}

pub(crate) fn basis(
    d: &FederalDraft,
    r: &mut FederalResult,
    contribution_result: Option<&crate::ira::IraResult>,
    qcd: &QcdResult,
    funding: &crate::hsa_funding::FundingResult,
) -> Result<BasisResult, IraAmountIssue> {
    use crate::kernel::ratio::dollars;
    let mut result = BasisResult::default();
    let Some(a) = &d.adjustments else {
        return Ok(result);
    };
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        crate::require_localized!(
            r,
            incoming_plan_basis(d, owner) == exact::whole(0)
                || a.ira_basis.iter().any(|b| b.owner == owner),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "ISSUE_PROVIDE_THIS_RECIPIENT_S_IRA_BASIS_RECORD_AFTER_AN_EMPLOYER_PLAN_RO_E8156654",
            "Provide this recipient's IRA basis record after an employer-plan rollover of after-tax contributions. The App adds the incoming basis automatically; do not include it in prior basis.",
        );
    }
    for (index, b) in a.ira_basis.iter().enumerate() {
        let entries: alloc::vec::Vec<_> = d
            .income
            .iter()
            .filter_map(|e| {
                if e.owner != crate::DocumentOwner::from(b.owner) {
                    return None;
                }
                match &e.income {
                    FederalIncome::Retirement { data: source } => {
                        let data = crate::FederalRetirement::from(source);
                        (data.kind == crate::RetirementKind::Ira && !data.outside_form_8606())
                            .then_some((e, data))
                    }
                    _ => None,
                }
            })
            .collect();
        let contribution_fact = a.entered_iras().and_then(|f| {
            if b.owner == Owner::Taxpayer { Some(&f.taxpayer) } else { f.spouse.as_ref() }
        });
        let contribution = contribution_fact.map_or(0, |v| v.traditional);
        let actually_deposited_next_year = contribution_fact
            .and_then(|f| f.next_year_deposits.as_ref())
            .map_or(0, |d| d.traditional);
        let valid = !a.ira_basis[..index].iter().any(|v| v.owner == b.owner)
            && (b.owner == Owner::Taxpayer
                || d.people.as_ref().is_some_and(|p| p.spouse.is_some()))
            && b.paid_next_year <= contribution
            && b.paid_next_year >= actually_deposited_next_year
            && entries.iter().all(|(_, v)| {
                v.taxable == Some(RetirementTaxable::IraBasis)
                    && match &v.ordinary_distribution {
                        Some(RetirementTreatment::Ordinary)
                        | Some(RetirementTreatment::Early)
                        | Some(RetirementTreatment::EarlySimple)
                        | Some(RetirementTreatment::Conversion { .. })
                        | Some(RetirementTreatment::Rollover { .. }) => true,
                        // Form 5329 uses the allocated taxable amount. Its existing
                        // exception stage owns eligibility and gross-limit checks.
                        // A deceased owner's basis never joins personal IRA basis.
                        Some(RetirementTreatment::Exception { facts, .. }) => {
                            !matches!(facts, RetirementException::Death { .. })
                        }
                        _ => false,
                    }
            });
        crate::require_localized!(
            r,
            valid,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "IRA basis",
            "ISSUE_USE_ONE_COMPLETE_BASIS_RECORD_PER_FILER_AND_SELECT_THE_IRA_WORKSHEE_F3EC6BF8",
            "Use one complete basis record per filer and select the IRA worksheet for all their traditional IRA distributions. Contributions paid after 2026 must include every actual 2027 traditional deposit designated for 2026 and cannot exceed retained traditional contributions. Inherited accounts use their separate deceased-owner record; changed basis requires additional coordination. Exception eligibility and any gross-distribution limits are checked separately.",
        );
        if !valid {
            continue;
        }
        if b.adjustment != 0 {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_FORM_8606_DIVORCE_STATEMENT"),
                crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
            );
        }
        let owner_name = if b.owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        for (i, payment) in b.reservist_repayments.iter().enumerate() {
            crate::require_localized!(
                r,
                !b.reservist_repayments[..i].iter().any(|p| p.distributed == payment.distributed),
                &crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
                "Reservist IRA repayment",
                "ISSUE_USE_ONE_RECORD_PER_ORIGINAL_DISTRIBUTION_AND_COMBINE_ITS_2026_PAYME_8B66531C",
                "Use one record per original distribution and combine its 2026 payments; do not duplicate the repayment ceiling.",
            );
            payment.report(r, owner_name, i + 1);
        }
        let gross_cents = entries
            .iter()
            .map(|(e, v)| {
                exact::whole(v.gross)
                    - exact::whole(v.rolled_amount())
                    - exact::whole(funding.for_entry(e.id))
                    - exact::whole(v.home_repaid())
            })
            .sum::<Exact>();
        let outstanding =
            entries.iter().map(|(_, v)| exact::whole(v.outstanding_rollover())).sum::<Exact>();
        let converted_cents =
            entries.iter().map(|(_, v)| exact::whole(v.converted_amount())).sum::<Exact>();
        let qcd_qualified = qcd.qualified[usize::from(b.owner == Owner::Spouse)];
        let regular_cents =
            (gross_cents - &converted_cents - exact::whole(qcd_qualified)).max(exact::whole(0));
        let converted_gross = exact::dollars(&converted_cents).ok_or(IraAmountIssue)?;
        let regular_gross = exact::dollars(&regular_cents).ok_or(IraAmountIssue)?;
        let gross = regular_gross.checked_add(converted_gross).ok_or(IraAmountIssue)?;
        let incoming = incoming_plan_basis(d, b.owner);
        // These are independently reported categories: preserve both existing
        // whole-Dollar boundaries before adding their reported amounts.
        let opening = exact::dollars(
            &(b.line_2_exact()
                - exact::whole(funding.basis_used[usize::from(b.owner == Owner::Spouse)])
                + &incoming),
        )
        .ok_or(IraAmountIssue)?;
        let reservist = exact::dollars(&b.reservist_basis_exact()).ok_or(IraAmountIssue)?;
        let prior = opening.checked_add(reservist).ok_or(IraAmountIssue)?;
        let year_end =
            exact::dollars(&(b.adjusted_year_end_exact(d) + &outstanding)).ok_or(IraAmountIssue)?;
        let denominator = year_end.checked_add(gross).ok_or(IraAmountIssue)?;
        let limited = a
            .entered_iras()
            .zip(d.setup.as_ref())
            .is_some_and(|(f, s)| crate::ira::income_limited(f, s, b.owner));
        let owner = usize::from(b.owner == Owner::Spouse);
        let nondeductible = contribution_result.map_or(0, |v| v.nondeductible[owner]);
        let total_basis = prior.checked_add(nondeductible).ok_or(IraAmountIssue)?;
        let available = prior
            .checked_add(available_contribution_basis(
                b,
                contribution,
                contribution_result.map_or(0, |v| v.absorbed[owner]),
                nondeductible,
            ))
            .ok_or(IraAmountIssue)?;
        IraAmountIssue::reported(reservist)?;
        let current = nondeductible.checked_add(reservist).ok_or(IraAmountIssue)?;
        IraAmountIssue::reported(current)?;
        let to_plan = entries.iter().any(|(_, v)| matches!(v.ordinary_distribution,
            Some(RetirementTreatment::Rollover { destination: crate::RolloverDestination::EmployerPlan, amount, .. }) if amount > 0));
        if to_plan && (contribution == 0 || contribution_result.is_some()) {
            crate::require_localized!(
                r,
                available <= denominator,
                &crate::IssueContext::owner(crate::TaxSection::Retirement, b.owner),
                "IRA rollover",
                "ISSUE_AN_IRA_TO_EMPLOYER_PLAN_ROLLOVER_CAN_CONTAIN_ONLY_PRETAX_FUNDS_THE_EA576D4E",
                "An IRA-to-employer-plan rollover can contain only pretax funds. The retained IRA balances and amounts not rolled into that plan must cover this owner's available after-tax basis.",
            );
        }
        let pro_rata = |basis: i64| {
            if denominator == 0 {
                0
            } else {
                phase::cap(
                    phase::cap(
                        fraction(gross, phase::cap(basis, [denominator]), denominator),
                        [basis],
                    ),
                    [gross],
                )
            }
        };
        // Pub. 590-B Worksheet 1-1 first treats current contributions as basis
        // to determine deduction MAGI. After fixing that deduction, its reporting
        // instructions either retain the preliminary exclusion or use ordinary
        // Form 8606 allocation when available basis is insufficient. No iteration.
        let preliminary = pro_rata(
            prior
                .checked_add(if limited { dollars(contribution) } else { 0 })
                .ok_or(IraAmountIssue)?,
        );
        let worksheet = limited
            && contribution > 0
            && (contribution_result.is_none() || available >= preliminary);
        let (regular_taxable, converted_taxable) = if worksheet {
            let taxable = gross - preliminary;
            let converted = if gross == 0 { 0 } else { fraction(taxable, converted_gross, gross) };
            (taxable - converted, converted)
        } else {
            let basis = if contribution_result.is_none() { prior } else { available };
            let fraction_for = |amount| {
                if denominator == 0 {
                    0
                } else {
                    phase::cap(
                        fraction(amount, phase::cap(basis, [denominator]), denominator),
                        [amount],
                    )
                }
            };
            // Form 8606 lines 11 and 12 separately round conversions and withdrawals.
            let converted_exempt = phase::cap(fraction_for(converted_gross), [basis]);
            let regular_exempt = phase::cap(
                fraction_for(regular_gross),
                [phase::floor(basis - converted_exempt, 0)],
            );
            (regular_gross - regular_exempt, converted_gross - converted_exempt)
        };
        result.regular_gross[owner] = regular_gross;
        result.regular_taxable[owner] = regular_taxable;
        let taxable = regular_taxable + converted_taxable;
        IraAmountIssue::reported(taxable)?;
        let exempt = gross - taxable;
        let carried = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            total_basis,
            exempt,
        )
        .get();
        // Stable IDs keep cent allocation independent of the editor's row order.
        let mut entries = entries;
        entries.sort_by_key(|(e, _)| e.id);
        let mut prefixes = [exact::ExactPrefix::default(), exact::ExactPrefix::default()];
        for (e, v) in entries {
            let amounts = [
                phase::floor(
                    v.gross
                        - v.rolled_amount()
                        - v.converted_amount()
                        - qcd.for_entry(e.id)
                        - funding.for_entry(e.id)
                        - v.home_repaid(),
                    0,
                ),
                v.converted_amount(),
            ];
            if v.home_repaid() > 0 {
                result.home_repaid.insert(e.id);
            }
            let mut portion = [0, 0];
            for (i, (total, gross)) in
                [(regular_taxable, &regular_cents), (converted_taxable, &converted_cents)]
                    .into_iter()
                    .enumerate()
            {
                portion[i] = prefixes[i].next(&exact::whole(amounts[i]), |cumulative| {
                    if gross == &exact::whole(0) {
                        0
                    } else {
                        // The total's actual reported Cents were checked above;
                        // each cumulative share is bounded by that native total.
                        exact::rounded(
                            &(exact::whole(total) * exact::whole(100) * cumulative / gross),
                        )
                        .expect("bounded IRA taxable share")
                    }
                });
            }
            result.taxable.insert(
                e.id,
                portion[0] + portion[1] + qcd.for_entry(e.id) + funding.for_entry(e.id),
            );
            result.converted.insert(e.id, portion[1]);
        }
        let prefix = if b.owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        let calculation_rows = [
            (
                "ira_current_nondeductible_basis",
                current,
                msg!("EXPLAIN_N_N_IRA_CURRENT_NONDEDUCTIBLE_BASIS"),
            ),
            ("ira_reservist_basis", reservist, msg!("EXPLAIN_N_N_IRA_RESERVIST_BASIS")),
            (
                "ira_incoming_plan_basis",
                exact::dollars(&incoming).ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_N_IRA_INCOMING_PLAN_BASIS"),
            ),
            (
                "ira_outstanding_rollover",
                exact::dollars(&outstanding).ok_or(IraAmountIssue)?,
                msg!("EXPLAIN_N_N_IRA_OUTSTANDING_ROLLOVER"),
            ),
            ("ira_basis_recovered", exempt, msg!("EXPLAIN_N_N_IRA_BASIS_RECOVERED")),
            ("ira_basis_carryforward", carried, msg!("EXPLAIN_N_N_IRA_BASIS_CARRYFORWARD")),
            ("ira_basis_taxable", taxable, msg!("EXPLAIN_N_N_IRA_BASIS_TAXABLE")),
        ];
        for (_, amount, _) in &calculation_rows {
            IraAmountIssue::reported(*amount)?;
        }
        emit_lines(r, &mut (), calculation_rows, |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("{prefix}_{suffix}"),
                LineValue::MoneyDollars(amount),
                &["adjustments.ira_basis", "income.retirement"],
                "https://www.irs.gov/instructions/i8606",
                explanation,
            ))
        });
        // A carried balance alone does not require this year's form. Nor do
        // special distributions all repaid in their original year. Current
        // nondeductible contributions and divorce transfers still require it.
        let fully_repaid = regular_cents > exact::whole(0)
            && converted_cents == exact::whole(0)
            && b.repaid_in_year_exact() >= regular_cents;
        let part_i = current > 0
            || b.adjustment != 0
            || (!fully_repaid
                && opening > 0
                && (regular_gross > 0 || (converted_gross > 0 && year_end > 0)));
        if part_i {
            for value in [current, opening, total_basis, carried] {
                IraAmountIssue::reported(value)?;
            }
            if gross > 0 {
                for value in [total_basis - available, available, regular_taxable] {
                    IraAmountIssue::reported(value)?;
                }
                if !worksheet {
                    for value in [year_end, denominator] {
                        IraAmountIssue::reported(value)?;
                    }
                }
            }
            emit_lines(
                r,
                &mut (),
                [
                    ("1", LineValue::MoneyDollars(current)),
                    ("2", LineValue::MoneyDollars(opening)),
                    ("3", LineValue::MoneyDollars(total_basis)),
                    ("4", LineValue::MoneyDollars(total_basis - available)),
                    ("5", LineValue::MoneyDollars(available)),
                    ("6", LineValue::MoneyDollars(year_end)),
                    ("7", LineValue::MoneyDollars(regular_gross)),
                    ("8", LineValue::MoneyDollars(converted_gross)),
                    ("9", LineValue::MoneyDollars(denominator)),
                    (
                        "10",
                        LineValue::RatioTenThousandths(if denominator == 0 {
                            0
                        } else {
                            fraction(10_000, phase::cap(available, [denominator]), denominator)
                        }),
                    ),
                    ("11", LineValue::MoneyDollars(converted_gross - converted_taxable)),
                    ("12", LineValue::MoneyDollars(regular_gross - regular_taxable)),
                    ("13", LineValue::MoneyDollars(exempt)),
                    ("14", LineValue::MoneyDollars(carried)),
                    ("15a", LineValue::MoneyDollars(regular_taxable)),
                ],
                |(number, value), _, _| {
                    let included = match number {
                        "1" | "2" | "3" | "14" => true,
                        "6" | "7" | "8" | "9" | "10" | "11" | "12" => gross > 0 && !worksheet,
                        _ => gross > 0,
                    };
                    included.then(|| {
                        calculation_row(
                            Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                            &alloc::format!("{prefix}_ira_form_line{number}"),
                            value,
                            &["adjustments.ira_basis", "adjustments.iras", "income.retirement"],
                            "https://www.irs.gov/pub/irs-dft/f8606--dft.pdf",
                            if worksheet {
                                msg!("EXPLAIN_IRA_FORM_RETAINED_WORKSHEET")
                            } else {
                                msg!("EXPLAIN_IRA_FORM_PART_I")
                            },
                        )
                    })
                },
            );
        }
        if converted_gross > 0 {
            // Without PartI, line17 is the available basis itself. A line18
            // at or below zero contributes no taxable income or deductible loss.
            let line17 = if part_i { converted_gross - converted_taxable } else { available };
            IraAmountIssue::reported(line17)?;
            IraAmountIssue::reported(converted_gross - line17)?;
            emit_lines(
                r,
                &mut (),
                [("16", converted_gross), ("17", line17), ("18", converted_gross - line17)],
                |(number, amount), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                        &alloc::format!("{prefix}_ira_form_line{number}"),
                        LineValue::MoneyDollars(amount),
                        &["adjustments.ira_basis", "adjustments.iras", "income.retirement"],
                        "https://www.irs.gov/pub/irs-dft/f8606--dft.pdf",
                        msg!("EXPLAIN_IRA_FORM_PART_II"),
                    ))
                },
            );
        }
    }
    for e in &d.income {
        if let Some(v) = e.income.retirement()
            && v.taxable == Some(RetirementTaxable::IraBasis)
        {
            crate::require_localized!(
                r,
                v.kind == crate::RetirementKind::Ira && result.taxable.contains_key(&e.id),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "IRA basis",
                "ISSUE_EVERY_IRA_BASIS_SELECTION_NEEDS_A_COMPLETE_SUPPORTED_OWNER_LEVEL_WO_C967CBA5",
                "Every IRA-basis selection needs a complete supported owner-level worksheet in Adjustments; it cannot determine pension or annuity taxation.",
            );
        }
    }
    Ok(result)
}

// Complete PartI from the existing final disaster/repayment owners. This
// presentation step does not calculate another taxable distribution or spread.
pub(crate) fn project_basis_relief(d: &FederalDraft, r: &mut FederalResult, basis: &BasisResult) {
    for (owner, who) in [(Owner::Taxpayer, "taxpayer"), (Owner::Spouse, "spouse")] {
        let Some(line15a) = r
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("{who}_ira_form_line15a"))
            .map(|row| row.amount)
        else {
            continue;
        };
        let designated: i64 = d
            .income
            .iter()
            .filter(|entry| {
                entry.owner == crate::DocumentOwner::from(owner)
                    && entry.income.retirement().is_some_and(|data| {
                        data.kind == RetirementKind::Ira
                            && data.taxable == Some(RetirementTaxable::IraBasis)
                    })
            })
            .map(|entry| basis.disaster.get(&entry.id).copied().unwrap_or(0))
            .sum();
        let repaid = basis.repayment_excluded[usize::from(owner == Owner::Spouse)];
        emit_lines(
            r,
            &mut (),
            [("15b", designated), ("15c", line15a - designated - repaid)],
            |(number, amount), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                    &alloc::format!("{who}_ira_form_line{number}"),
                    LineValue::MoneyCents(amount),
                    &["adjustments.ira_basis", "income.retirement"],
                    "https://www.irs.gov/pub/irs-dft/i8606--dft.pdf",
                    msg!("EXPLAIN_IRA_FORM_RELIEF"),
                ))
            },
        );
    }
}

// Shared by current and historical Forms 5329. Source amounts are cents;
// AGI is whole dollars. Apply the return-wide medical floor once, allocating
// relief to 25% SIMPLE amounts first, then 10% amounts.
pub(crate) fn medical_exception_allocation(
    mut taxable: [[i64; 2]; 2],
    expenses: i64,
    agi: i64,
) -> [[i64; 2]; 2] {
    let mut allowance = phase::floor(
        crate::kernel::ratio::dollars(expenses) - fraction(phase::floor(agi, 0), 75, 1000),
        0,
    ) * 100;
    for rate in [1, 0] {
        for owner in &mut taxable {
            let exempt = crate::kernel::carry::native_consume_step(owner[rate], &mut allowance);
            owner[rate] = crate::kernel::carry::native_balance(owner[rate], exempt);
        }
    }
    taxable
}

// Separate spouse forms aggregate source cents before rounding each line.
// Return taxable distributions, exceptions, subject amount and tax in dollars.
pub(crate) fn early_tax_lines(before: i64, after: [i64; 2]) -> [i64; 4] {
    use crate::kernel::ratio::dollars;
    let taxable = dollars(before);
    let exceptions = dollars(before - after.iter().sum::<i64>());
    let subject = phase::floor(taxable - exceptions, 0);
    let simple = phase::cap(dollars(after[1]), [subject]);
    let tax = dollars((subject - simple) * 10 + simple * 25);
    [taxable, exceptions, subject, tax]
}

// Form 5329 Part II, one form for each filer: line 5 the ABLE and education
// account distributions included in income, line 6 those that are not subject
// to the additional tax, line 7 the difference and line 8 its 10%
// (Schedule 2 line 5). The ABLE part keeps the cents its calculation made;
// the education part is education_account::calculate's. Source cents are
// aggregated before each line is rounded, as Part I does. Returns the total
// tax in whole dollars.
pub(crate) struct AccountDistributionTaxes {
    pub total_dollars: i64,
    pub owners: [i64; 2],
}

pub(crate) fn account_additional_tax(
    r: &mut crate::FederalResult,
    able: [i64; 2],
    able_penalized: [i64; 2],
    education: &crate::education_account::EducationTotals,
) -> AccountDistributionTaxes {
    use crate::kernel::ratio::{dollars, fraction, positive};
    use crate::line_emission::line;
    let mut total = 0;
    let mut owners = [0; 2];
    for owner in 0..2 {
        let included = able[owner] + education.income[owner];
        if included == 0 {
            continue;
        }
        let exempt = able[owner] - able_penalized[owner] + education.exceptions[owner];
        let (line_5, line_6) = (dollars(included), dollars(exempt));
        let line_7 = positive(line_5 - line_6);
        let tax =
            fraction(line_7, crate::year_constants::EDUCATION_ACCOUNT_ADDITIONAL_TAX_PERCENT, 100);
        total += tax;
        owners[owner] = tax;
        emit_lines(
            r,
            &mut (),
            [
                ("taxable", line_5, msg!("EXPLAIN_ACCOUNT_TAXABLE")),
                ("exceptions", line_6, msg!("EXPLAIN_ACCOUNT_EXCEPTIONS")),
                ("subject", line_7, msg!("EXPLAIN_ACCOUNT_SUBJECT")),
                ("tax", tax, msg!("EXPLAIN_ACCOUNT_TAX")),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "account_{}_{suffix}",
                        if owner == 0 { "taxpayer" } else { "spouse" }
                    ),
                    LineValue::MoneyDollars(amount),
                    &["income.able", "income.education_distribution"],
                    "https://www.irs.gov/instructions/i5329",
                    explanation,
                ))
            },
        );
    }
    line(
        r,
        "account_additional_tax",
        total,
        &["income.able", "income.education_distribution"],
        "https://www.irs.gov/instructions/i5329",
        msg!("EXPLAIN_ACCOUNT_ADDITIONAL_TAX"),
    );
    AccountDistributionTaxes { total_dollars: total, owners }
}
