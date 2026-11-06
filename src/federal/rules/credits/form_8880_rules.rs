// form_8880 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Notice 2025-67 supplies 2026 boundaries; Form 8880 supplies the worksheet.
// Surviving-spouse status uses the "all other" Saver's Credit band, not joint brackets.
pub(crate) fn saver_rate(agi: i64, status: FederalStatus) -> i64 {
    let limits = match status {
        FederalStatus::Joint => crate::year_constants::SAVER_AGI_LIMITS_JOINT_DOLLARS,
        FederalStatus::Head => crate::year_constants::SAVER_AGI_LIMITS_HEAD_DOLLARS,
        _ => crate::year_constants::SAVER_AGI_LIMITS_OTHER_DOLLARS,
    };
    if agi <= limits[0] {
        50
    } else if agi <= limits[1] {
        20
    } else if agi <= limits[2] {
        10
    } else {
        0
    }
}

pub(super) fn saver_potential_with_sources(
    facts: &FederalSaver,
    people: &FederalPeople,
    status: FederalStatus,
    agi: i64,
) -> Result<(i64, [i64; 2]), (&'static str, &'static str)> {
    let j = joint(status);
    if !j
        && (facts.spouse_able != 0
            || facts.spouse_contributions != 0
            || facts.distributions.iter().any(|d| d.owner == Owner::Spouse))
    {
        return Err((
            "ISSUE_SAVER_SPOUSE_REQUIRES_JOINT_RETURN",
            "Spouse contributions and distributions belong only on a joint return. Revisit Saver’s Credit after a filing-status change.",
        ));
    }
    let mut offsets = [0, 0];
    for d in &facts.distributions {
        let owner = if d.owner == Owner::Taxpayer { 0 } else { 1 };
        offsets[owner] += d.amount;
        // A recipient's own withdrawals always reduce their base. Cross-spouse reductions
        // depend on filing together in the distribution year, even if the recipient is ineligible.
        // An undecided 2027 status is assumed joint (the smaller credit).
        if j && (d.year() == SaverDistributionYear::TaxYear
            || d.joint_with_current_spouse() != Answer::No)
        {
            offsets[1 - owner] += d.amount;
        }
    }
    let contributions = [
        facts.taxpayer_contributions + facts.taxpayer_able,
        facts.spouse_contributions + facts.spouse_able,
    ];
    let persons = [Some(&people.taxpayer), if j { people.spouse.as_ref() } else { None }];
    let eligible: [i64; 2] = core::array::from_fn(|i| {
        // Form 8880 prints "born after January 1, 2009": the day-before rule.
        if persons[i].is_some_and(|p| {
            p.attained(18, true) && !p.full_time_student && !p.dependency.claimable()
        }) {
            // Aggregate source cents within each worksheet line before whole-dollar rounding.
            phase::cap(positive(dollars(contributions[i]) - dollars(offsets[i])), [2000])
        } else {
            0
        }
    });
    Ok((fraction(eligible.iter().sum(), saver_rate(agi, status), 100), eligible))
}
#[cfg(test)]
pub(crate) fn saver_potential(
    facts: &FederalSaver,
    people: &FederalPeople,
    status: FederalStatus,
    agi: i64,
) -> Result<i64, (&'static str, &'static str)> {
    saver_potential_with_sources(facts, people, status, agi).map(|(amount, _)| amount)
}

pub(super) struct Form8880In<'a> {
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) foreign_magi: i64,
}

pub(super) struct Form8880Out {
    pub(super) saver_available: i64,
    pub(super) saver_eligible_sources: Option<[i64; 2]>,
}

pub(super) fn form_8880_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form8880In<'_>,
) -> Form8880Out {
    let &Pass { c, s, p, pension, allocated_entries, .. } = pass;
    let Form8880In { ira, foreign_magi } = i;
    let mut saver_eligible_sources = None;
    let saver_available = match c.saver.as_ref() {
        Some(facts) => {
            let mut combined = facts.clone();
            // Section 25B(d)(2)(C) does not exclude section 414(w) withdrawals.
            // Reuse the source amount once, then let the shared lookback apply
            // recipient and joint-return offsets before the contribution ceiling.
            let mut automatic = [0, 0];
            for entry in allocated_entries {
                if let Some(data) = entry.income.retirement()
                    && matches!(
                        data.ordinary_distribution,
                        Some(RetirementTreatment::Exception {
                            facts: RetirementException::AutomaticEnrollment { .. },
                            ..
                        })
                    )
                {
                    automatic[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                        data.gross;
                }
            }
            for (index, amount) in automatic.into_iter().enumerate() {
                if amount > 0 {
                    combined.distributions.push(FederalSaverDistribution {
                        owner: if index == 0 { Owner::Taxpayer } else { Owner::Spouse },
                        year: SaverDistributionPeriod::TaxYear,
                        amount,
                    });
                    line_cents(
                        r,
                        if index == 0 {
                            "saver_automatic_withdrawals_taxpayer"
                        } else {
                            "saver_automatic_withdrawals_spouse"
                        },
                        amount,
                        &["income.retirement"],
                        "https://www.law.cornell.edu/uscode/text/26/25B",
                        msg!("EXPLAIN_SAVER_AUTOMATIC_WITHDRAWALS_TAXPAYER"),
                    );
                }
            }
            // The same $2,000 per-person base covers IRA, workplace and beneficiary-funded ABLE contributions.
            combined.taxpayer_contributions += ira.contributions[0] + pension.saver[0];
            combined.spouse_contributions += ira.contributions[1] + pension.saver[1];
            if joint(s.filing_status())
                && combined.distributions.iter().any(|v| {
                    v.amount > 0
                        && v.year() == SaverDistributionYear::FollowingYear
                        && v.joint_with_current_spouse() == Answer::NotSure
                })
            {
                note_once_scoped(
                    r,
                    msg!("WARN_SAVER_2027_JOINT_ASSUMED"),
                    crate::IssueContext::Section(crate::TaxSection::Saver),
                );
            }
            match saver_potential_with_sources(&combined, p, s.filing_status(), foreign_magi) {
                Ok((potential, sources)) => {
                    saver_eligible_sources = Some(sources);
                    potential
                }
                Err((key, _message)) => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::Saver),
                        "Saver’s Credit",
                        key,
                        _message,
                    );
                    0
                }
            }
        }
        None => 0,
    };
    let ira_saver_possible = saver_rate(foreign_magi, s.filing_status()) > 0
        && [Some(&p.taxpayer), p.spouse.as_ref()].iter().enumerate().any(|(i, person)| {
            (ira.contributions[i] > 0 || pension.saver[i] > 0)
                && person.is_some_and(|v| {
                    // Form 8880 prints "born after January 1, 2009": the
                    // day-before rule.
                    v.attained(18, true) && !v.full_time_student && !v.dependency.claimable()
                })
        });
    // Facts not entered: no Saver's Credit, with a warning.
    if ira_saver_possible && c.saver.is_none() {
        note_once_scoped(
            r,
            msg!("WARN_SAVER_CREDIT_NOT_CLAIMED"),
            crate::IssueContext::Section(crate::TaxSection::Saver),
        );
    }
    Form8880Out { saver_available, saver_eligible_sources }
}
