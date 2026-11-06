// Coverage: Form 1099-QA with Form 5329 Part II (ABLE distributions) — Supported (2026-10-09; the stored ABLE kind reshaped into the per-form document).
//   Lines: every box of the Rev. April 2025 form: 1 gross distribution (it includes amounts the
//   beneficiary meant to roll over and the value of a non-qualifying beneficiary change, never a
//   program-to-program transfer), 2 earnings, 3 basis, 4 program-to-program transfer, 5 account
//   terminated, 6 other than the designated beneficiary. Boxes 4 and 5 change no figure (a form
//   showing only the transfer needs no treatment). Box 6 never decides the treatment; an annual
//   withdrawal answered while box 6 is checked is ISSUE_ABLE_PAYMENT_TO_NON_BENEFICIARY: no figure,
//   because the additional taxes on that recipient (Form 5329) are not supported.
//   Publication 907 Table 1 / section 529A(c)(1): earnings times the qualified disability expenses
//   (with the first 60 days of 2027 elected for 2026) over the distribution are excluded; a rollover
//   to an ABLE account within 60 days is removed with its earnings; a payment after the
//   beneficiary's death is not taxed; a timely returned excess contribution is taxed to the
//   contributor as its earnings with no 10% tax. Taxable earnings are Schedule 1 line 8q; the 10%
//   additional tax is Form 5329 Part II lines 5-8 per filer (retirement.rs account_additional_tax).
//   Entry: OtherIncomeDocument::Qa1099 (Form 1099-QA, importable) on the Other income page; the
//   treatment is Annual (the beneficiary's own withdrawal), Death or Corrective (the filer's answer).
//   Exclusions: a beneficiary change to someone who is not an eligible brother or sister is already
//   in box 1; the Saver's Credit lookback reads these forms.
//   Contributions (Form 5329 Part VIII, the 2026 draft's ABLE excess: the $20,000 limit, the
//   working-beneficiary allowance, 6% of the lesser of the excess and the year-end value, to
//   Schedule 2): IncomeRecord::AbleContributions (AbleContributionsPage) on the same
//   page, one for each beneficiary by owner (ABLE_CONTRIBUTIONS_TWICE), stored on
//   screening.able_contributions. The records-complete confirmation is gone: the form's own
//   figures decide; a spouse's record needs a joint return (ISSUE_ABLE_BENEFICIARY_NOT_ON_RETURN).
//   Tests: src/tests/health/account_documents.rs able_contributions_*, src/tests/family/health_accounts.rs able_excess_*.
//   Tests: src/tests/family/health_accounts.rs (fixtures F11-F13), src/tests/family/education_accounts.rs, src/tests/family/interview.rs
//   qa1099_*. Sources: https://www.irs.gov/pub/irs-pdf/f1099qa.pdf and i1099qa.pdf (Rev. April
//   2025), https://www.irs.gov/pub/irs-pdf/p907.pdf (2025), https://www.irs.gov/pub/irs-dft/f5329--dft.pdf
//   (2026 draft), 26 U.S.C. 529A.
// Native source projection controls: src/tests/attribution/unit13_account_sources.rs (recipient,
// source cents, penalty character, correction coordination and Save/restart).
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::fraction;
use crate::line_emission::emit as emit_lines;
use crate::year_constants::ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::able_distribution_source::{
    AbleDistribution, AbleRollover, AbleSource, AbleTreatment, Qa1099,
};

pub(crate) use crate::able_contribution_source::valid_contributions;
pub use crate::able_contribution_source::{AbleContributions, AbleWork};

/// ABLE contributions
/// Contributions to the ABLE accounts of one beneficiary on this return for 2026, to figure the 6% tax on contributions above the limit (Form 5329 Part VIII). One for each beneficiary; choose whose accounts they are by the owner. Contributions do not reduce taxable income, and qualified ABLE-to-ABLE transfers are not counted.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleContributionsPage {
    /// Beneficiary's own cash contributions for 2026
    /// Exclude timely returned principal; retain program records. Do not subtract associated earnings from contributions. Do not include a section 529 tuition-plan rollover here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub own: i64,
    /// Other contributions and section 529 tuition-plan rollovers for 2026
    /// Include contributions from every other person, and all tuition-plan rollovers regardless of contributor. Exclude timely returned principal, qualified ABLE-to-ABLE rollovers or transfers, and full direct Trump-account-to-ABLE rollovers for a 17-year-old beneficiary. Other contributors cannot use the beneficiary's working-person allowance.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// Working-beneficiary additional allowance
    #[libertas_default(No)]
    pub work: AbleWork,
    /// Account value on December 31, 2026
    /// The 6% tax cannot exceed 6% of this value. Sum year-end values of every 2026 ABLE account for this beneficiary, including an additional account still open; do not enter just current contributions.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end: i64,
    /// Additional 2026 account still open
    /// Select Yes when this beneficiary has a second ABLE account that has not yet been fully corrected or transferred into the preexisting account. Include both accounts' 2026 contributions and year-end values once; enter each account's distributions on its own Form 1099-QA. The additional account remains an ABLE account through 2026; if neither complete correction nor consolidation occurs by the beneficiary's return deadline, including extensions, it ceases to be an ABLE account immediately after that deadline. This later change is not a 2026 deemed distribution.
    #[libertas_default(false)]
    pub additional_open: bool,
}

impl AbleContributionsPage {
    pub(crate) fn record(&self, owner: crate::Owner) -> AbleContributions {
        AbleContributions {
            owner,
            own: self.own,
            other: self.other,
            work: self.work.clone(),
            year_end: self.year_end,
            additional_open: self.additional_open,
        }
    }
}
impl AbleContributions {
    pub(crate) fn page(&self) -> AbleContributionsPage {
        AbleContributionsPage {
            own: self.own,
            other: self.other,
            work: self.work.clone(),
            year_end: self.year_end,
            additional_open: self.additional_open,
        }
    }
}

pub(crate) fn excess_tax(
    values: &[AbleContributions],
    status: crate::FederalStatus,
    r: &mut crate::FederalResult,
) -> [i64; 2] {
    use crate::kernel::ratio::{dollars, positive};
    let mut tax = [0; 2];
    for v in values {
        if v.additional_open {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Additional ABLE account: 2026 contributions, account values and distributions must include both accounts once. Unless the additional account is fully corrected or consolidated by the beneficiary's return deadline, including extensions, it ceases to be an ABLE account immediately after that deadline; review the following year separately.", crate::Message::new("NOTE_ADDITIONAL_ABLE_ACCOUNT_RETURN_YEAR_CONTRIBUTIONS_ACCOUNT_VALUES_AND_DI_24653906", alloc::vec![])));
        }
        crate::require_localized!(
            r,
            v.owner == crate::Owner::Taxpayer || status == crate::FederalStatus::Joint,
            &crate::IssueContext::owner(crate::TaxSection::OtherIncome, v.owner),
            "ABLE contributions",
            "ISSUE_ABLE_BENEFICIARY_NOT_ON_RETURN",
            "An ABLE beneficiary other than you is on this return only when you file jointly.",
        );
        let total = dollars(v.own + v.other);
        let additional = match v.work {
            AbleWork::No => 0,
            AbleWork::Working { compensation, residence, no_plan } => {
                let potential = phase::cap(
                    phase::cap(dollars(v.own), [dollars(compensation)]),
                    [crate::marketplace::poverty(residence, 1)],
                );
                if no_plan { potential } else { 0 }
            }
        };
        let excess = positive(total - ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS - additional);
        let charge = fraction(phase::cap(excess, [dollars(v.year_end)]), 6, 100);
        tax[usize::from(v.owner == crate::Owner::Spouse)] += charge;
        emit_lines(
            r,
            &mut (),
            [
                ("contribution_limit", ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS + additional),
                ("excess_contributions", excess),
                ("excess_tax", charge),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "able_{}_{}",
                        if v.owner == crate::Owner::Taxpayer { "taxpayer" } else { "spouse" },
                        suffix
                    ),
                    LineValue::MoneyDollars(amount),
                    &["screening.able_contributions"],
                    "https://www.irs.gov/instructions/i5329",
                    msg!("EXPLAIN_ABLE_N_N"),
                ))
            },
        );
    }
    tax
}
