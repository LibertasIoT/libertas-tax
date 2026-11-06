use crate::common::Owner;
use crate::filer_facts::FederalStatus;
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Coverdell ESA excess contributions for one beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoverdellExcess {
    /// Beneficiary on this return
    pub beneficiary: Owner,
    /// Contributor limits and actual non-rollover deposits for all beneficiary accounts
    /// No deposit is invented for a year with only prior excess. Retain actual contributor limit facts when available.
    pub contributors: Vec<CoverdellContributor>,
    /// Timely returned current contributions and their net earnings
    pub corrections: Vec<CoverdellCorrection>,
    /// Retained 2025 Form 5329 line 32
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_excess: i64,
    /// Retained 2025 Form 5329 line 33; only a positive value activates prior excess
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_tax: i64,
    /// Additional 2026 non-rollover distributions not already entered on a 1099-Q; exclude returned excess
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub distributions: i64,
    /// Combined December 31 account value, including 2026 contributions made in 2027
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Beneficiary has special needs under the Coverdell age exception
    /// Relevant only if a contribution was made after the beneficiary reached 18.
    /// Check the beneficiary's disability and account-eligibility records.
    pub special_needs: bool,
}
/// One contributor, with no taxpayer identification number
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoverdellContributor {
    /// Your contributor label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Use this return for an on-return contributor
    pub person: CoverdellContributorKind,
    /// Actual 2026-designated non-rollover deposits
    pub contributions: Vec<CoverdellContribution>,
}
/// Contributor source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoverdellContributorKind {
    /// A filer on this return
    Filer { owner: Owner },
    /// Another individual
    OtherPerson {
        /// Actual Coverdell modified adjusted gross income, not regular taxable income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        modified_agi: i64,
        /// Contributor filing status
        status: FederalStatus,
    },
    /// An organization; no individual MAGI phaseout
    Organization,
    /// Required contributor source records are unavailable
    Unavailable,
}
/// Timely returned current contributions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoverdellCorrection {
    /// Account/source label, matching its contribution records
    #[libertas_size(min = 1)]
    pub account: String,
    /// Current contribution principal returned
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Actual withdrawal date; before June 1, 2027
    #[libertas_date_only]
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = $crate::year_constants::TAX_YEAR_START_DATE, max = $crate::year_constants::COVERDELL_CORRECTION_LAST_DAY)]
    pub withdrawn: u32,
    /// Associated net income or loss withdrawn with the principal
    /// A loss is not a deduction. Positive earnings belong to the original contribution year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Existing 2026 Form 1099-Q code 2 for these same earnings, when furnished
    /// A 2027 correction without a 2026 source is recorded directly here.
    #[libertas_default($null)]
    pub income_document: Option<i64>,
}
/// A dated 2026-designated contribution, excluding rollovers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoverdellContribution {
    /// Account/source label
    #[libertas_size(min = 1)]
    pub account: String,
    /// Actual paid date
    /// A 2027 deposit designated for 2026 is deemed made December 31, 2026 for the contribution-year age rule.
    #[libertas_date_only]
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = $crate::year_constants::TAX_YEAR_START_DATE, max = $crate::year_constants::COVERDELL_CONTRIBUTION_LAST_DAY)]
    pub paid: u32,
    /// Actual non-rollover contribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

fn money(v: i64) -> bool {
    (0..=100_000_000_000).contains(&v)
}
fn label(v: &str) -> bool {
    !v.trim().is_empty()
}

// One intrinsic source owner supplies accepted validity and the normal Save issue.
// List/amount/date shape is checked before sums; these bounded values cannot overflow i64.
pub(crate) fn intrinsic_rule(values: &[CoverdellExcess]) -> Option<ValidationIssue> {
    for (i, v) in values.iter().enumerate() {
        if values[..i].iter().any(|old| old.beneficiary == v.beneficiary) {
            return Some(
                ValidationIssue::new("COVERDELL_BENEFICIARY_TWICE")
                    .at_field("beneficiary")
                    .in_list("beneficiaries", i),
            );
        }
        if v.contributors.len() > 100
            || v.corrections.len() > 100
            || ![v.prior_excess, v.prior_tax, v.distributions, v.value].into_iter().all(money)
        {
            return Some("COVERDELL_INVALID_SOURCE".into());
        }
        for (j, c) in v.contributors.iter().enumerate() {
            if !label(&c.label)
                || c.contributions.len() > 100
                || matches!(c.person, CoverdellContributorKind::OtherPerson { modified_agi, .. }
                    if !(-100_000_000_000..=100_000_000_000).contains(&modified_agi))
                || !c.contributions.iter().all(|deposit| {
                    label(&deposit.account)
                        && deposit.amount > 0
                        && money(deposit.amount)
                        && (crate::year_constants::TAX_YEAR_START_DATE
                            ..=crate::year_constants::COVERDELL_CONTRIBUTION_LAST_DAY)
                            .contains(&deposit.paid)
                        && crate::dates::source_valid_date(deposit.paid)
                })
            {
                return Some("COVERDELL_INVALID_SOURCE".into());
            }
            if v.contributors[..j].iter().any(|old| old.label == c.label) {
                return Some(
                    ValidationIssue::new("COVERDELL_DONOR_TWICE")
                        .at_field("label")
                        .in_list("contributors", j)
                        .in_list("beneficiaries", i),
                );
            }
            if matches!(c.person, CoverdellContributorKind::Filer { owner }
                if v.contributors[..j].iter().any(|old| matches!(old.person,
                    CoverdellContributorKind::Filer { owner: previous } if previous == owner)))
            {
                return Some(
                    ValidationIssue::new("COVERDELL_DONOR_TWICE")
                        .at_field("owner")
                        .in_field("person")
                        .in_list("contributors", j)
                        .in_list("beneficiaries", i),
                );
            }
        }
        for (correction_index, c) in v.corrections.iter().enumerate() {
            if !label(&c.account)
                || c.principal <= 0
                || !money(c.principal)
                || !(-100_000_000_000..=100_000_000_000).contains(&c.earnings)
                || !(crate::year_constants::TAX_YEAR_START_DATE
                    ..=crate::year_constants::COVERDELL_CORRECTION_LAST_DAY)
                    .contains(&c.withdrawn)
                || !crate::dates::source_valid_date(c.withdrawn)
            {
                return Some("COVERDELL_INVALID_SOURCE".into());
            }
            if c.principal + c.earnings < 0 {
                return Some(
                    ValidationIssue::new("COVERDELL_LOSS_ABOVE_PRINCIPAL")
                        .at_field("earnings")
                        .in_list("corrections", correction_index)
                        .in_list("beneficiaries", i),
                );
            }
        }
    }
    None
}
