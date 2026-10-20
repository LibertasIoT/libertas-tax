// IRC 219(b)(3), (g)(5)(B), 402(g); IRS Pub. 525; Notice 2025-67.
use crate::kernel::lines::line;
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::{FederalResult, FederalStatus, Owner};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Employee-funded section 501(c)(18)(D) contributions
/// Enter one annual record per filer, combining all qualifying trusts. These contributions remain in W-2 box 1 wages; the App deducts them separately and includes them in the Saver's Credit contribution base. They also establish workplace-plan coverage for IRA deduction limits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PensionContributions {
    /// Employee
    pub owner: Owner,
    /// Total 2026 contributions reported with W-2 box 12 code H
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub contributions: i64,
    /// Employee's annual section 415(c)(3) compensation included in gross income
    /// Use the employer's compensation records. Do not enter household income, a spouse's compensation or a calculated 25% limit. Wages remain entered in Income without subtracting code H.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub compensation: i64,
    /// Other regular elective deferrals sharing the annual section 402(g) limit
    /// Combine this employee's 401(k), TSP, 403(b), SARSEP and SIMPLE deferrals, including designated Roth amounts. Exclude employer contributions, governmental 457 deferrals and separately established permitted catch-up amounts. Do not repeat code H contributions here. This input checks the shared limit; it does not automatically claim a Saver's Credit for those other deferrals.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other_deferrals: i64,
    /// Source amounts and ordinary contribution treatment confirmed
    /// Employee-funded qualified section 501(c)(18)(D) trusts only, with all annual contributions and compensation entered. No unresolved excess contributions, corrective distributions, prior-year allocation or unverified catch-up classification. The App calculates the $7,000, 25% compensation and shared annual limits. W-2 box 12 code H and the plan's documents show it; No leaves the deduction out of this estimate.
    pub confirmed: bool,
}
impl PensionContributions {
    pub(crate) fn valid(&self) -> bool {
        [self.contributions, self.compensation, self.other_deferrals]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
    }
}

#[derive(Default)]
pub(crate) struct Contributions {
    pub deduction: i64,
    pub saver: [i64; 2], // cents; join the existing Form 8880 base once
}

pub(crate) fn calculate(
    facts: &[PensionContributions],
    status: FederalStatus,
    r: &mut FederalResult,
) -> Contributions {
    let mut out = Contributions::default();
    let mut seen = [false; 2];
    for f in facts {
        let i = usize::from(f.owner == Owner::Spouse);
        let usable = !seen[i] && (i == 0 || status == FederalStatus::Joint);
        crate::require_localized!(
            r,
            usable,
            &crate::IssueContext::Review.manual(),
            "Employee-funded pension contributions",
            "ISSUE_ENTER_ONE_CONFIRMED_ANNUAL_SECTION_501_C_18_D_RECORD_PER_FILER_SPOU_D34001ED",
            "Enter one confirmed annual section 501(c)(18)(D) record per filer; spouse contributions require a joint return.",
        );
        seen[i] = true;
        // An unconfirmed record takes no deduction; the contributions still
        // show workplace-plan coverage.
        if !f.confirmed {
            crate::federal_rules::note_once(r, msg!("WARN_PENSION_CONTRIBUTIONS_LEFT_OUT"));
        }
        if !usable || !f.confirmed {
            continue;
        }
        let limit = phase::cap(
            phase::cap(
                crate::year_constants::SECTION_501C18D_CONTRIBUTION_CAP_CENTS,
                [crate::kernel::ratio::trunc_i64(f.compensation, 4)],
            ),
            [phase::floor(
                crate::year_constants::ELECTIVE_DEFERRAL_LIMIT_2026_CENTS - f.other_deferrals,
                0,
            )],
        );
        crate::require_localized!(
            r,
            f.other_deferrals <= crate::year_constants::ELECTIVE_DEFERRAL_LIMIT_2026_CENTS
                && f.contributions <= limit,
            &crate::IssueContext::Review.manual(),
            "Employee-funded pension contributions",
            "ISSUE_CONTRIBUTIONS_EXCEED_THE_7_000_25_COMPENSATION_OR_2026_SHARED_24_50_BB098953",
            "Contributions exceed the $7,000, 25% compensation or 2026 shared $24,500 regular-deferral limit. Excess-contribution and catch-up corrections need resolution before this return can be calculated.",
        );
        line(
            r,
            if i == 0 {
                "pension_contribution_limit_taxpayer"
            } else {
                "pension_contribution_limit_spouse"
            },
            dollars(limit),
            &["adjustments.pension_contributions"],
            "https://www.irs.gov/publications/p525",
            msg!("EXPLAIN_PENSION_CONTRIBUTION_LIMIT_TAXPAYER"),
        );
        if f.contributions <= limit
            && f.other_deferrals <= crate::year_constants::ELECTIVE_DEFERRAL_LIMIT_2026_CENTS
        {
            out.deduction += dollars(f.contributions);
            out.saver[i] = f.contributions;
        }
    }
    line(
        r,
        "pension_contribution_deduction",
        out.deduction,
        &["adjustments.pension_contributions"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_PENSION_CONTRIBUTION_DEDUCTION"),
    );
    out
}
