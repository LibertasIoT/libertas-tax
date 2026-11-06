// schedule_1a items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Schedule1AWorkSourcesIn<'a> {
    pub(super) tips: [i64; 2],
    pub(super) overtime: [i64; 2],
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
}

pub(super) struct Schedule1AWorkSourcesOut {
    pub(super) qualified_tip_owners: [i64; 2],
    pub(super) qualified_overtime_owners: [i64; 2],
    pub(super) qualified_tips: i64,
    pub(super) qualified_overtime: i64,
    pub(super) qualified_business_tips: i64,
    pub(super) business_tips: Vec<(i64, i64)>,
}

pub(super) fn schedule_1a_work_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: Schedule1AWorkSourcesIn<'_>,
) -> core::ops::ControlFlow<(), Schedule1AWorkSourcesOut> {
    let Schedule1AWorkSourcesIn { tips, overtime, business_half_se, health, sep } = input;
    let d = pass.d;
    let p = pass.p;
    let allocated_entries = pass.allocated_entries;
    let selected_regular = pass.selected_regular;
    let office_context = pass.office_context;
    let source_claims = pass.context.source_claims;
    let mut qualified_tip_owners = [0i64; 2];
    let mut qualified_overtime_owners = [0i64; 2];
    let mut qualified_tips = 0;
    let mut qualified_overtime = 0;
    for (i, person) in [Some(&p.taxpayer), p.spouse.as_ref()].into_iter().enumerate() {
        if person.is_some_and(|p| p.valid_ssn) {
            qualified_tips += tips[i];
            qualified_tip_owners[i] += tips[i];
            qualified_overtime += overtime[i];
            qualified_overtime_owners[i] += overtime[i];
        }
    }
    let mut qualified_business_tips = 0;
    // The allowed qualified tips of each business, in whole dollars.
    let mut business_tips: Vec<(i64, i64)> = Vec::new();
    for entry in allocated_entries {
        let FederalIncome::Business { data: business } = &entry.income else {
            continue;
        };
        let ordinary_receipts = match business.activity {
            crate::BusinessActivity::Ordinary { receipts, .. } => Some(receipts),
            _ => None,
        };
        let tip_receipts = match &business.activity {
            crate::BusinessActivity::Ordinary { receipts, .. } => Some(*receipts),
            crate::BusinessActivity::Farming { data } => match &data.accounting {
                crate::FarmAccounting::Cash { receipts } => Some(
                    receipts
                        .iter()
                        .filter_map(|receipt| match receipt {
                            crate::FarmReceipt::Other { amount } => Some(*amount),
                            _ => None,
                        })
                        .sum(),
                ),
                crate::FarmAccounting::Accrual { other, .. } => Some(*other),
            },
            crate::BusinessActivity::Gambling { .. } => None,
        };
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let valid_ssn =
            [Some(&p.taxpayer), p.spouse.as_ref()][owner].is_some_and(|person| person.valid_ssn);
        let (printed_tips, printed_overtime) =
            crate::information_returns::reported_work(d, entry.id, source_claims);
        crate::require_localized!(
            r,
            printed_tips.is_none_or(|amount| amount == 0 || business.tips.is_some()),
            &crate::IssueContext::income_id(entry.id),
            "Self-employed tips",
            "DOCUMENT_REPORTED_TIPS_FACTS",
            "Review this activity's reported cash tips, occupation and eligibility, excluding amounts duplicated on other information returns.",
        );
        crate::require_localized!(
            r,
            printed_overtime.is_none_or(|amount| amount == 0 || business.overtime.is_some()),
            &crate::IssueContext::income_id(entry.id),
            "1099 overtime",
            "DOCUMENT_REPORTED_OVERTIME_FACTS",
            "Review this activity's separately reported overtime, FLSA eligibility and any duplicate W-2 or information-return reporting.",
        );
        if let Some(tips) = &business.tips {
            let mut tips = tips.clone();
            if let Some(reported) = printed_tips {
                tips.amount = tips.amount.min(reported);
            }
            crate::require_localized!(
                r,
                tips.amount == 0
                    || (business.source == crate::BusinessSource::UnitedStates
                        && tip_receipts.is_some_and(|receipts| tips.amount <= receipts)
                        && tips.eligible),
                &crate::IssueContext::income_id(entry.id),
                "Self-employed tips",
                "ISSUE_CONFIRM_QUALIFIED_CUSTOMER_TIPS_SEPARATELY_STATED_ON_2026_INFORMATI_23ADE78E",
                "Confirm qualified customer tips separately stated on 2026 information returns furnished to this individual, included once in U.S. Schedule C ordinary receipts or Schedule F other farm operating receipts. Partnership and unresolved source cases require their own worksheet.",
            );
            crate::require_localized!(
                r,
                tips.amount == 0 || valid_ssn,
                &crate::IssueContext::income_id(entry.id),
                "Self-employed tips",
                "ISSUE_THE_PERSON_WHO_RECEIVED_QUALIFIED_TIPS_NEEDS_A_SOCIAL_SECURITY_NUMB_36F061FF",
                "The person who received qualified tips needs a Social Security number valid for employment.",
            );
            let net = dollars(
                business.receipts()
                    - match crate::section179_return::expenses_with_claims(
                        d,
                        entry.id,
                        business,
                        selected_regular,
                        Some(office_context),
                        source_claims,
                    )
                    .map(|expense| {
                        expense
                            - selected_regular.map_or(0, |allocation| {
                                allocation.ordinary_limit_adjustment(entry.id)
                            })
                    }) {
                        Ok(value) => value,
                        Err(issue) => {
                            crate::section179_return::Issue::from(issue).report(r);
                            r.state = ResultState::Unsupported;
                            return core::ops::ControlFlow::Break(());
                        }
                    },
            ) - business_half_se.get(&entry.id).copied().unwrap_or(0)
                - sep.by_business.get(&entry.id).copied().unwrap_or(0)
                - health.by_business.get(&entry.id).copied().unwrap_or(0);
            let allowed = if valid_ssn && tip_receipts.is_some() {
                phase::cap(dollars(tips.amount), [positive(net)])
            } else {
                0
            };
            qualified_business_tips += allowed;
            qualified_tip_owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                allowed * 100;
            business_tips.push((entry.id, allowed));
            line(
                r,
                &alloc::format!("business_{}_qualified_tips", entry.id),
                allowed,
                &["income.business.tips", "income.business"],
                "https://www.irs.gov/pub/irs-dft/f1040s1a--dft.pdf",
                msg!("EXPLAIN_BUSINESS_N_QUALIFIED_TIPS"),
            );
        }
        if let Some(overtime) = &business.overtime {
            let mut overtime = overtime.clone();
            if let Some(reported) = printed_overtime {
                overtime.amount = overtime.amount.min(reported);
            }
            crate::require_localized!(
                r,
                overtime.amount == 0
                    || (business.source == crate::BusinessSource::UnitedStates
                        && ordinary_receipts.is_some_and(|receipts| overtime.amount <= receipts)
                        && overtime.eligible),
                &crate::IssueContext::income_id(entry.id),
                "1099 overtime",
                "ISSUE_CONFIRM_FLSA_REQUIRED_OVERTIME_ABOVE_THE_REGULAR_RATE_SEPARATELY_RE_DE584C59",
                "Confirm FLSA-required overtime above the regular rate, separately reported on a 2026 Form 1099-NEC or 1099-MISC and included once in this U.S. Schedule C business's ordinary receipts. Other payment and source cases require their own worksheet.",
            );
            crate::require_localized!(
                r,
                overtime.amount == 0 || valid_ssn,
                &crate::IssueContext::income_id(entry.id),
                "1099 overtime",
                "ISSUE_THE_PERSON_WHO_RECEIVED_QUALIFIED_OVERTIME_NEEDS_A_SOCIAL_SECURITY_A0D851C0",
                "The person who received qualified overtime needs a Social Security number valid for employment.",
            );
            if valid_ssn && ordinary_receipts.is_some() {
                qualified_overtime += overtime.amount;
                qualified_overtime_owners
                    [usize::from(entry.owner == crate::DocumentOwner::Spouses)] += overtime.amount;
            }
            line_cents(
                r,
                &alloc::format!("business_{}_qualified_overtime", entry.id),
                overtime.amount,
                &["income.business.overtime", "income.business"],
                "https://www.irs.gov/pub/irs-dft/f1040s1a--dft.pdf",
                msg!("EXPLAIN_BUSINESS_N_QUALIFIED_OVERTIME"),
            );
        }
    }
    core::ops::ControlFlow::Continue(Schedule1AWorkSourcesOut {
        qualified_tip_owners,
        qualified_overtime_owners,
        qualified_tips,
        qualified_overtime,
        qualified_business_tips,
        business_tips,
    })
}

pub(super) struct Schedule1ADeductionsIn<'a> {
    pub(super) foreign_magi: i64,
    pub(super) qualified_tips: i64,
    pub(super) qualified_overtime: i64,
    pub(super) qualified_business_tips: i64,
    pub(super) business_tips: &'a [(i64, i64)],
}

pub(super) struct Schedule1ADeductionsOut {
    pub(super) additional: [i64; 3],
    pub(super) tips_deducted: alloc::collections::BTreeMap<i64, i64>,
}

pub(super) fn schedule_1a_deductions_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: Schedule1ADeductionsIn<'_>,
) -> Schedule1ADeductionsOut {
    let Schedule1ADeductionsIn {
        foreign_magi,
        qualified_tips,
        qualified_overtime,
        qualified_business_tips,
        business_tips,
    } = input;
    let ded = pass.ded;
    let s = pass.s;
    for loan in &ded.vehicle_loans {
        crate::require_localized!(
            r,
            loan.business <= loan.model().interest,
            &crate::IssueContext::Section(crate::TaxSection::Deductions),
            "Vehicle loan interest",
            "VEHICLE_BUSINESS_ABOVE_INTEREST",
            "The part deducted on Schedule C, E or F cannot be more than the interest you paid on this loan.",
        );
    }
    // Schedule 1-A line 28: a loan counts only when both questions are
    // answered Yes; a No means the vehicle does not qualify (no deduction).
    // Column (iii) is the interest less the part deducted on Schedule C, E
    // or F (column (ii)).
    let vehicle_interest: i64 = ded
        .vehicle_loans
        .iter()
        .filter(|v| v.model().original_use && v.model().assembled_in_us)
        .map(|v| positive(v.model().interest - v.business))
        .sum();
    let additional = additional_deductions(
        foreign_magi,
        s.filing_status(),
        dollars(qualified_tips) + qualified_business_tips,
        dollars(qualified_overtime),
        dollars(vehicle_interest),
    );
    // The part of the Schedule 1-A tips deduction that falls on each business's
    // tips: amounts deductible under section 224(a) are not qualified business
    // income (2026 i8995 What's New and line 1). The cap and phaseout reduce
    // every qualified tip alike.
    let tips_weights: Vec<i64> = core::iter::once(dollars(qualified_tips))
        .chain(business_tips.iter().map(|(_, allowed)| *allowed))
        .collect();
    let tips_shares = crate::kernel::ratio::allocate_remaining(additional[0], &tips_weights);
    let tips_deducted: alloc::collections::BTreeMap<i64, i64> =
        business_tips.iter().map(|(id, _)| *id).zip(tips_shares.into_iter().skip(1)).collect();
    emit_lines(
        r,
        &mut (),
        ["tips_deduction", "overtime_deduction", "vehicle_interest_deduction"]
            .into_iter()
            .zip(additional),
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income", "deductions.vehicle_loans", "agi", "people"],
                "https://www.irs.gov/pub/irs-pdf/f1040s1a.pdf",
                msg!("EXPLAIN_FEDERAL_RULES_6"),
            ))
        },
    );
    Schedule1ADeductionsOut { additional, tips_deducted }
}

// Schedule 1-A deliberately floors tips/overtime phaseout steps but CEILS vehicle steps.
pub(crate) fn additional_deductions(
    agi: i64,
    status: FederalStatus,
    tips: i64,
    overtime: i64,
    vehicle: i64,
) -> [i64; 3] {
    let j = joint(status);
    let work_reduction = phase::steps(
        agi - if j {
            crate::year_constants::SCHEDULE_1A_WORK_PHASEOUT_START_JOINT_DOLLARS
        } else {
            crate::year_constants::SCHEDULE_1A_WORK_PHASEOUT_START_SINGLE_DOLLARS
        },
        1000,
        phase::StepRounding::Down,
    ) * crate::year_constants::SCHEDULE_1A_WORK_REDUCTION_PER_THOUSAND_DOLLARS;
    [
        if status == FederalStatus::Separate {
            0
        } else {
            positive(
                phase::cap(tips, [crate::year_constants::SCHEDULE_1A_TIPS_CAP_DOLLARS])
                    - work_reduction,
            )
        },
        if status == FederalStatus::Separate {
            0
        } else {
            positive(
                phase::cap(
                    overtime,
                    [if j {
                        crate::year_constants::SCHEDULE_1A_OVERTIME_CAP_JOINT_DOLLARS
                    } else {
                        crate::year_constants::SCHEDULE_1A_OVERTIME_CAP_SINGLE_DOLLARS
                    }],
                ) - work_reduction,
            )
        },
        positive(
            phase::cap(vehicle, [crate::year_constants::SCHEDULE_1A_VEHICLE_INTEREST_CAP_DOLLARS])
                - phase::steps(
                    agi - if j {
                        crate::year_constants::SCHEDULE_1A_VEHICLE_PHASEOUT_START_JOINT_DOLLARS
                    } else {
                        crate::year_constants::SCHEDULE_1A_VEHICLE_PHASEOUT_START_SINGLE_DOLLARS
                    },
                    1000,
                    phase::StepRounding::Up,
                ) * crate::year_constants::SCHEDULE_1A_VEHICLE_REDUCTION_PER_THOUSAND_DOLLARS,
        ),
    ]
}
