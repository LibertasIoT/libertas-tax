// form_8396 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form 8396 line 3: the current-year credit the certificate supports, with
// its reissue worksheet rows. Section 163(g) then reduces the deduction for
// interest on the certified loan by this amount.
pub(crate) fn mortgage_credit_line3(c: &crate::FederalCredits, r: &mut FederalResult) -> i64 {
    c.mortgage_credit
        .as_ref()
        .and_then(|claim| claim.current.as_ref())
        .map_or(0, |certificate| {
            let original_period_interest = match &certificate.history {
                crate::MortgageCertificateHistory::ReissuedDuring2026SameRate { original_period_interest, .. }
                | crate::MortgageCertificateHistory::ReissuedDuring2026DifferentRates { original_period_interest, .. } => original_period_interest.certified_interest(),
                _ => 0,
            };
            let current_interest = certificate.interest.certified_interest();
            line(
                r,
                "mortgage_certified_interest",
                dollars(current_interest + original_period_interest),
                &["credits.mortgage_credit.current.interest", "credits.mortgage_credit.current.history"],
                "https://www.irs.gov/pub/irs-dft/f8396--dft.pdf",
                msg!("EXPLAIN_MORTGAGE_CERTIFIED_INTEREST"),
            );
            let gross = if let crate::MortgageCertificateHistory::ReissuedDuring2026DifferentRates { facts, .. } = &certificate.history {
                let original = fraction(dollars(original_period_interest), crate::kernel::natural::units(facts.original_rate, 100), 10000);
                let replacement = fraction(dollars(current_interest), crate::kernel::natural::units(certificate.rate, 100), 10000);
                emit_lines(r, &mut (), [
                    ("mortgage_original_period_credit", original, msg!("EXPLAIN_MORTGAGE_ORIGINAL_PERIOD_CREDIT")),
                    ("mortgage_reissued_period_credit", replacement, msg!("EXPLAIN_MORTGAGE_REISSUED_PERIOD_CREDIT"))], |(id, amount, description), _, _| Some(calculation_row(None,
id,
LineValue::MoneyDollars(amount),
&["credits.mortgage_credit.current.history"],
"https://www.irs.gov/pub/irs-dft/f8396--dft.pdf",
description)));
                original + replacement
            } else {
                fraction(
                    dollars(current_interest + original_period_interest),
                    crate::kernel::natural::units(certificate.rate, 100),
                    10000,
                )
            };
            let reissued_credit = if crate::kernel::natural::units(certificate.rate, 100) > 2000 {
                phase::cap(gross, [fraction(2000, crate::kernel::natural::units(certificate.ownership_share, 100), 10000)])
            } else {
                gross
            };
            match &certificate.history {
                crate::MortgageCertificateHistory::Original => reissued_credit,
                crate::MortgageCertificateHistory::ReissuedBefore2026 { facts }
                | crate::MortgageCertificateHistory::ReissuedDuring2026SameRate { facts, .. }
                | crate::MortgageCertificateHistory::ReissuedDuring2026DifferentRates { facts, .. } => {
                    crate::require_localized!(
                        r,
                        facts.confirmed
                            && facts.original_interest_method.supported()
                            && facts.first_reissue,
                        &(crate::IssueContext::Section(crate::TaxSection::MoreCredits)).manual(),"Mortgage interest credit",
                        "ISSUE_CONFIRM_A_FIRST_REISSUE_THE_ORIGINAL_CERTIFICATE_AND_ITS_2026_LIMIT_71828D94","Confirm a first reissue, the original certificate and its 2026 limit interest, and that the reissue went to the same holders for the same home, replaced the original entirely, and retained no old certified debt. A variable-rate original needs a retained scheduled- or hypothetical-interest worksheet with confirmed consistent use of that method; multiple refinancings need separate history.",
                    );
                    let original_gross = fraction(
                        dollars(facts.original_scheduled_interest),
                        crate::kernel::natural::units(facts.original_rate, 100),
                        10000,
                    );
                    let original_credit = if crate::kernel::natural::units(facts.original_rate, 100) > 2000 {
                        phase::cap(original_gross, [fraction(2000, crate::kernel::natural::units(certificate.ownership_share, 100), 10000)])
                    } else {
                        original_gross
                    };
                    line(
                        r,
                        "mortgage_original_schedule_cap",
                        original_credit,
                        &["credits.mortgage_credit.current.history"],
                        "https://www.irs.gov/pub/irs-dft/f8396--dft.pdf",
                        msg!("EXPLAIN_MORTGAGE_ORIGINAL_SCHEDULE_CAP"),
                    );
                    phase::cap(reissued_credit, [original_credit])
                }
                crate::MortgageCertificateHistory::Unreviewed => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),"Mortgage interest credit",
                        "ISSUE_AN_UNREVIEWED_CERTIFICATE_NEEDS_ITS_ORIGINAL_SCHEDULE_REFINANCE_DAT_81C9C3D6","An unreviewed certificate needs its original schedule, refinance date, and reissue facts before the Form 8396 credit can be calculated.",
                    );
                    0
                }
            }
        })
}

pub(super) struct Form8396In<'a> {
    pub(super) schedule_8812: &'a crate::forms::schedule_8812::Schedule8812,
    pub(super) homeowner_potentials: [i64; 4],
    pub(super) mortgage_credit_current: i64,
    pub(super) creditable_tax: i64,
    pub(super) preceding_mortgage_credits: i64,
}

pub(super) struct Form8396Out {
    pub(super) homeowner_credits: crate::forms::schedule_8812::HomeownerCredits,
    pub(super) mortgage_credit_used: i64,
}

pub(super) fn form_8396_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form8396In<'_>,
) -> Form8396Out {
    let &Pass { c, .. } = pass;
    let Form8396In {
        schedule_8812,
        homeowner_potentials,
        mortgage_credit_current,
        creditable_tax,
        preceding_mortgage_credits,
    } = i;
    let mortgage_carry = c.mortgage_credit.as_ref().map_or([0; 3], |claim| {
        [
            dollars(claim.carry_three_years_prior),
            dollars(claim.carry_two_years_prior),
            dollars(claim.carry_prior_year),
        ]
    });
    let homeowner_credits = schedule_8812.homeowner_credits(
        creditable_tax,
        preceding_mortgage_credits,
        homeowner_potentials,
    );
    let mortgage_credit_used = homeowner_credits.mortgage;
    // Form 8396 applies current-year credit first, then the oldest surviving
    // carryforward. Unused 2023 credit expires; 2024–2026 balances reach 2027.
    let mortgage_entries = [
        (2023, mortgage_carry[0]),
        (2024, mortgage_carry[1]),
        (2025, mortgage_carry[2]),
        (2026, mortgage_credit_current),
    ]
    .map(|(year, amount)| crate::kernel::carry::Entry {
        key: year,
        amount: crate::kernel::carry::Amount::<crate::kernel::carry::Dollars>::new(amount)
            .expect("nonnegative mortgage credit"),
        last_use_year: Some(year + 3),
    });
    let mortgage_settlements = crate::kernel::carry::consume_by_key(
        &mortgage_entries,
        crate::kernel::carry::Amount::new(mortgage_credit_used).expect("nonnegative credit use"),
        2026,
        |year| (u8::from(*year != 2026), *year),
    )
    .expect("mortgage origins and priority are established");
    emit_lines(
        r,
        &mut (),
        [
            (
                "mortgage_interest_credit",
                mortgage_credit_used,
                msg!("EXPLAIN_MORTGAGE_INTEREST_CREDIT"),
            ),
            (
                "mortgage_credit_expired_2023",
                mortgage_settlements[0].expired.get(),
                msg!("EXPLAIN_MORTGAGE_CREDIT_EXPIRED_N"),
            ),
            (
                "mortgage_credit_carry_2024_to_2027",
                mortgage_settlements[1].closing.get(),
                msg!("EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N"),
            ),
            (
                "mortgage_credit_carry_2025_to_2027",
                mortgage_settlements[2].closing.get(),
                msg!("EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N_2"),
            ),
            (
                "mortgage_credit_carry_2026_to_2027",
                mortgage_settlements[3].closing.get(),
                msg!("EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N_3"),
            ),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["credits.mortgage_credit", "income_tax", "child_credit"],
                "https://www.irs.gov/pub/irs-dft/f8396--dft.pdf",
                explanation,
            ))
        },
    );
    Form8396Out { homeowner_credits, mortgage_credit_used }
}
