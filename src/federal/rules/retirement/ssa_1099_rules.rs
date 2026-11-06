// ssa_1099 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(crate) fn social_taxable(
    benefits: i64,
    other: i64,
    exempt: i64,
    status: FederalStatus,
    apart: bool,
) -> i64 {
    // Pub. 915 Worksheet 1 excludes the student-loan deduction from adjustments here.
    let provisional = positive((other + exempt) * 100 + benefits * 50);
    if status == FederalStatus::Separate && !apart {
        return dollars(phase::cap(benefits * 85, [fraction(provisional, 85, 100)]));
    }
    let (base, band) = if joint(status) {
        (
            crate::year_constants::SOCIAL_SECURITY_BASE_JOINT_DOLLARS,
            crate::year_constants::SOCIAL_SECURITY_BAND_JOINT_DOLLARS,
        )
    } else {
        (
            crate::year_constants::SOCIAL_SECURITY_BASE_OTHER_DOLLARS,
            crate::year_constants::SOCIAL_SECURITY_BAND_OTHER_DOLLARS,
        )
    };
    let excess = positive(provisional - base * 100);
    dollars(phase::cap(
        benefits * 85,
        [phase::cap(
            benefits * 50,
            [crate::kernel::ratio::trunc_i64(phase::cap(excess, [band * 100]), 2)],
        ) + fraction(positive(excess - band * 100), 85, 100)],
    ))
}

pub(super) fn collect_social_security(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::SocialBenefitStatement,
) -> Result<(), CollectFailure> {
    let Source {
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    let v = crate::FederalSocial::from(data);
    crate::require_localized!(
        r,
        crate::ssa_1099_model::net_benefits_issue(&v.form_boxes, v.benefits).is_none(),
        &context,
        &source,
        "SSA_PRINTED_BOXES_DIFFER",
        "The SSA-1099's printed net benefits in box 5 must agree with benefits paid in box 3 less benefits repaid in box 4. Correct the printed entries without replacing box 5 with an inferred amount.",
    );
    t.social += v.benefits;
    t.withholding_1099 += v.withholding;
    Ok(())
}

pub(super) struct SocialIn<'a, 'd> {
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) ira_facts: &'a Option<alloc::borrow::Cow<'d, FederalIras>>,
    pub(super) qtp_transfers: &'a crate::ira::QtpTransfers,
    pub(super) compensation: [i64; 2],
    pub(super) carry_distributions: [i64; 2],
    pub(super) before_social: i64,
    pub(super) exempt: i64,
    pub(super) excluded: i64,
    pub(super) adoption_excluded: i64,
    pub(super) ira_magi: i64,
}

pub(super) struct SocialOut {
    pub(super) social: i64,
    pub(super) qsbs_with_prior_nol: bool,
    pub(super) modified_ira: Option<Result<crate::ira::IraResult, (&'static str, &'static str)>>,
}

pub(super) fn social_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &mut crate::nol::NolRun,
    i: SocialIn<'_, '_>,
) -> SocialOut {
    let &Pass { d, screen, p, s, pension, social_election, .. } = pass;
    let SocialIn {
        ira,
        ira_facts,
        qtp_transfers,
        compensation,
        carry_distributions,
        before_social,
        exempt,
        excluded,
        adoption_excluded,
        ira_magi,
    } = i;
    let social = social_election.taxable(
        dollars(t.social),
        before_social - ira.deduction - pension.deduction,
        exempt + excluded + adoption_excluded,
        s.filing_status(),
        s.lived_apart_all_year(),
    );
    let qsbs_with_prior_nol =
        t.qsbs_excluded > 0 && (screen.nol.present() || estate_final_nol(d).is_some());
    // Form 172 Part II line 6 uses the same Appendix B worksheet with the
    // excluded gain restored to modified income. This is separate from the
    // actual return's IRA deduction and Form 8606 basis.
    let modified_ira = if qsbs_with_prior_nol && nol_run.modify_income {
        ira_facts.as_deref().map(|iras| {
            let original_social_before_ira = social_taxable(
                dollars(t.social),
                before_social,
                exempt + excluded + adoption_excluded,
                s.filing_status(),
                s.lived_apart_all_year(),
            );
            let modified_social_before_ira = social_taxable(
                dollars(t.social),
                before_social + dollars(t.qsbs_excluded),
                exempt + excluded + adoption_excluded,
                s.filing_status(),
                s.lived_apart_all_year(),
            );
            crate::ira::calculate_with_difficulty_of_care(
                iras,
                p,
                s,
                crate::ira::Compensation {
                    taxable: compensation,
                    difficulty_of_care: core::array::from_fn(|i| {
                        t.medicaid_excluded[i] + t.medicaid_in_box1[i]
                    }),
                },
                (ira_magi + dollars(t.qsbs_excluded) + modified_social_before_ira
                    - original_social_before_ira)
                    * 100,
                qtp_transfers,
                carry_distributions,
            )
        })
    } else {
        None
    };
    nol_run.section1202_ira_adjustment = modified_ira
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map_or(0, |modified| ira.deduction - modified.deduction);
    nol_run.section1202_social_adjustment = if nol_run.modify_income && t.qsbs_excluded > 0 {
        social_election.taxable(
            dollars(t.social),
            before_social - ira.deduction + nol_run.section1202_ira_adjustment - pension.deduction
                + dollars(t.qsbs_excluded),
            exempt + excluded + adoption_excluded,
            s.filing_status(),
            s.lived_apart_all_year(),
        ) - social
    } else {
        0
    };
    line(
        r,
        "social_lump_sum_taxable_reduction",
        social_taxable(
            dollars(t.social),
            before_social - ira.deduction - pension.deduction,
            exempt + excluded + adoption_excluded,
            s.filing_status(),
            s.lived_apart_all_year(),
        ) - social,
        &["adjustments.social_election", "income.social_security"],
        "https://www.irs.gov/publications/p915",
        msg!("EXPLAIN_SOCIAL_LUMP_SUM_TAXABLE_REDUCTION"),
    );
    SocialOut { social, qsbs_with_prior_nol, modified_ira }
}
