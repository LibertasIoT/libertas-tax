// Coverage: Form 5329 Part V — bounded implementation (2026-10-10).
// Lines 26–33: beneficiary-owned all-account excess/carry and value-limited 6%
// tax, donor MAGI limits, actual/deemed age, timely returned contributions
// and associated income once. Sources: IRS 2026 draft i5329 p. 7;
// https://www.irs.gov/pub/irs-pdf/p970.pdf (2025 ch. 6 Worksheets 6-1/6-2,
// including its actual 2026 carry example); IRC 530(d)(4)(C), 4973(e).
// Missing limits compute only when both tax and carry are determined by
// endpoints. Invalid age/account consequences or unreconciled correction
// sources remain named mandatory-tax limitations, with no assumed zero.
// Entry: OtherTax::CoverdellExcess Page/accepted pairs; src/tests/family/other_taxes.rs
// public arithmetic, restart, failed-save, source-link and owner lifecycle.
// Form8379 line18 receives the actual beneficiary's final line33 tax, not
// the contributor's share.
use crate::common::Owner;
use crate::coverdell_model::intrinsic_rule;
pub use crate::coverdell_model::{
    CoverdellContribution, CoverdellContributor, CoverdellContributorKind, CoverdellCorrection,
    CoverdellExcess,
};
use crate::filer_facts::FederalStatus;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use crate::year_constants::{
    COVERDELL_EXCESS_RATE_PERCENT, COVERDELL_JOINT_SPAN_DOLLARS, COVERDELL_JOINT_START_DOLLARS,
    COVERDELL_LIMIT_DOLLARS, COVERDELL_SINGLE_SPAN_DOLLARS, COVERDELL_SINGLE_START_DOLLARS,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Coverdell ESA excess contributions for one beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoverdellExcessPage {
    /// Beneficiary on this return
    #[libertas_constrained_by("$.allowed_owners")]
    pub beneficiary: Owner,
    /// Contributor limits and actual non-rollover deposits for all beneficiary accounts
    /// No deposit is invented for a year with only prior excess. Retain actual contributor limit facts when available.
    pub contributors: Vec<CoverdellContributorPage>,
    /// Timely returned current contributions and their net earnings
    pub corrections: Vec<CoverdellCorrectionPage>,
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
pub struct CoverdellContributorPage {
    /// Your contributor label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Use this return for an on-return contributor
    pub person: CoverdellContributorKindPage,
    /// Actual 2026-designated non-rollover deposits
    pub contributions: Vec<CoverdellContribution>,
}
/// Contributor source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoverdellContributorKindPage {
    /// A filer on this return
    Filer {
        #[libertas_constrained_by("$.allowed_owners")]
        owner: Owner,
    },
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
pub struct CoverdellCorrectionPage {
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
    /// [NullText]
    /// No separate 2026 Form 1099-Q reports these same earnings
    /// [SetText]
    /// The Form 1099-Q reporting these earnings is
    #[libertas_enum_source("$.coverdell_corrections")]
    pub income_document: Option<i64>,
}
impl From<CoverdellExcessPage> for CoverdellExcess {
    fn from(v: CoverdellExcessPage) -> Self {
        Self {
            beneficiary: v.beneficiary,
            contributors: v.contributors.into_iter().map(Into::into).collect(),
            corrections: v.corrections.into_iter().map(Into::into).collect(),
            prior_excess: v.prior_excess,
            prior_tax: v.prior_tax,
            distributions: v.distributions,
            value: v.value,
            special_needs: v.special_needs,
        }
    }
}
impl From<CoverdellContributorPage> for CoverdellContributor {
    fn from(v: CoverdellContributorPage) -> Self {
        Self { label: v.label, person: v.person.into(), contributions: v.contributions }
    }
}
impl From<CoverdellCorrectionPage> for CoverdellCorrection {
    fn from(v: CoverdellCorrectionPage) -> Self {
        Self {
            account: v.account,
            principal: v.principal,
            withdrawn: v.withdrawn,
            earnings: v.earnings,
            income_document: v.income_document,
        }
    }
}
impl From<CoverdellContributorKindPage> for CoverdellContributorKind {
    fn from(v: CoverdellContributorKindPage) -> Self {
        match v {
            CoverdellContributorKindPage::Filer { owner } => Self::Filer { owner },
            CoverdellContributorKindPage::OtherPerson { modified_agi, status } => {
                Self::OtherPerson { modified_agi, status }
            }
            CoverdellContributorKindPage::Organization => Self::Organization,
            CoverdellContributorKindPage::Unavailable => Self::Unavailable,
        }
    }
}
impl From<CoverdellExcess> for CoverdellExcessPage {
    fn from(v: CoverdellExcess) -> Self {
        Self {
            beneficiary: v.beneficiary,
            contributors: v.contributors.into_iter().map(Into::into).collect(),
            corrections: v.corrections.into_iter().map(Into::into).collect(),
            prior_excess: v.prior_excess,
            prior_tax: v.prior_tax,
            distributions: v.distributions,
            value: v.value,
            special_needs: v.special_needs,
        }
    }
}
impl From<CoverdellContributor> for CoverdellContributorPage {
    fn from(v: CoverdellContributor) -> Self {
        Self { label: v.label, person: v.person.into(), contributions: v.contributions }
    }
}
impl From<CoverdellCorrection> for CoverdellCorrectionPage {
    fn from(v: CoverdellCorrection) -> Self {
        Self {
            account: v.account,
            principal: v.principal,
            withdrawn: v.withdrawn,
            earnings: v.earnings,
            income_document: v.income_document,
        }
    }
}
impl From<CoverdellContributorKind> for CoverdellContributorKindPage {
    fn from(v: CoverdellContributorKind) -> Self {
        match v {
            CoverdellContributorKind::Filer { owner } => Self::Filer { owner },
            CoverdellContributorKind::OtherPerson { modified_agi, status } => {
                Self::OtherPerson { modified_agi, status }
            }
            CoverdellContributorKind::Organization => Self::Organization,
            CoverdellContributorKind::Unavailable => Self::Unavailable,
        }
    }
}

fn beneficiary(
    entry: &crate::FederalIncomeEntry,
    q: &crate::FederalEducationDistribution,
) -> Option<Owner> {
    match q.effective_beneficiary()? {
        crate::FederalEducationBeneficiary::Recipient => Some(entry.owner.owned()),
        crate::FederalEducationBeneficiary::Student { person: -1 } => Some(Owner::Taxpayer),
        crate::FederalEducationBeneficiary::Student { person: -2 } => Some(Owner::Spouse),
        _ => None,
    }
}

pub(crate) fn correction_source(entry: &crate::FederalIncomeEntry) -> bool {
    let crate::FederalIncome::EducationDistribution { data } = &entry.income else {
        return false;
    };
    let form = crate::Q1099::from(&data.form);
    matches!(form.transfer, crate::Q1099Transfer::ExcessThisYear)
        && matches!(
            form.account,
            crate::EducationAccount::Coverdell {
                amounts: crate::CoverdellAmounts::Reported { .. }
            }
        )
}

pub(crate) fn rule(values: &[CoverdellExcess], d: &crate::FederalDraft) -> Option<&'static str> {
    issue(values, d).map(|issue| issue.key)
}
pub(crate) fn issue(
    values: &[CoverdellExcess],
    d: &crate::FederalDraft,
) -> Option<ValidationIssue> {
    rule_impl(values, d, true)
}
// Accepted facts keep intrinsic consistency even when a referenced income source
// changes. Normal page validation reopens the affected editor; calculation then
// reports its mandatory-tax limitation until the actual source is reconciled.
pub(crate) fn accepted_rule(
    values: &[CoverdellExcess],
    d: &crate::FederalDraft,
) -> Option<&'static str> {
    rule_impl(values, d, false).map(|issue| issue.key)
}
fn rule_impl(
    values: &[CoverdellExcess],
    d: &crate::FederalDraft,
    check_links: bool,
) -> Option<ValidationIssue> {
    if let Some(issue) = intrinsic_rule(values) {
        return Some(issue);
    }
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint);
    let on_return = |owner| owner == Owner::Taxpayer || joint;
    let mut used = Vec::new();
    for (beneficiary_index, v) in values.iter().enumerate() {
        if !on_return(v.beneficiary)
            || v.contributors.iter().any(
                |c| matches!(c.person, CoverdellContributorKind::Filer{owner} if !on_return(owner)),
            )
        {
            return Some("COVERDELL_OWNER_NOT_ON_RETURN".into());
        }
        for (correction_index, correction) in v.corrections.iter().enumerate() {
            let available: i64 = v
                .contributors
                .iter()
                .flat_map(|c| &c.contributions)
                .filter(|c| c.account == correction.account && c.paid <= correction.withdrawn)
                .map(|c| c.amount)
                .sum();
            let corrected: i64 = v
                .corrections
                .iter()
                .filter(|c| c.account == correction.account && c.withdrawn <= correction.withdrawn)
                .map(|c| c.principal)
                .sum();
            if corrected > available {
                return Some(
                    ValidationIssue::new("COVERDELL_RETURN_ABOVE_CONTRIBUTIONS")
                        .at_field("principal")
                        .in_list("corrections", correction_index)
                        .in_list("beneficiaries", beneficiary_index),
                );
            }
            if let Some(id) = correction.income_document {
                if used.contains(&id) {
                    return Some(
                        ValidationIssue::new("COVERDELL_CORRECTION_SOURCE_TWICE")
                            .at_field("income_document")
                            .in_list("corrections", correction_index)
                            .in_list("beneficiaries", beneficiary_index),
                    );
                }
                if !check_links {
                    used.push(id);
                    continue;
                }
                let Some(entry) = d.income.iter().find(|e| e.id == id && correction_source(e))
                else {
                    return Some(
                        ValidationIssue::new("COVERDELL_CORRECTION_SOURCE_INVALID")
                            .at_field("income_document")
                            .in_list("corrections", correction_index)
                            .in_list("beneficiaries", beneficiary_index),
                    );
                };
                let crate::FederalIncome::EducationDistribution { data } = &entry.income else {
                    unreachable!()
                };
                let data = &crate::FederalEducationDistribution::from(data);
                let crate::EducationAccount::Coverdell {
                    amounts: crate::CoverdellAmounts::Reported { earnings, basis },
                } = data.account
                else {
                    unreachable!()
                };
                if beneficiary(entry, data) != Some(v.beneficiary)
                    || entry.owner != crate::DocumentOwner::from(v.beneficiary)
                    || earnings != correction.earnings
                    || basis != correction.principal
                    || data.gross != correction.principal + correction.earnings
                    || correction.withdrawn > crate::year_constants::TAX_YEAR_END_DATE
                {
                    return Some(
                        ValidationIssue::new("COVERDELL_CORRECTION_SOURCE_INVALID")
                            .at_field("income_document")
                            .in_list("corrections", correction_index)
                            .in_list("beneficiaries", beneficiary_index),
                    );
                }
                used.push(id);
            }
        }
    }
    None
}

pub(crate) fn unlinked_earnings(
    values: &[CoverdellExcess],
    r: &mut crate::FederalResult,
) -> [i64; 2] {
    let mut income = [0; 2];
    for (index, c) in values
        .iter()
        .flat_map(|v| v.corrections.iter().map(move |c| (v.beneficiary, c)))
        .enumerate()
    {
        let (owner, c) = c;
        if c.income_document.is_none() {
            let amount = phase::floor(c.earnings, 0);
            income[owner as usize] += amount;
            crate::line_emission::line_cents(
                r,
                &alloc::format!("coverdell_correction_{}_earnings", index + 1),
                amount,
                &["coverdell_excess.corrections"],
                "https://www.irs.gov/pub/irs-pdf/p970.pdf",
                msg!("EXPLAIN_COVERDELL_CORRECTION_EARNINGS"),
            );
        }
    }
    income
}

fn contributor_limit(c: &CoverdellContributor, magi: i64, status: FederalStatus) -> Option<i64> {
    use crate::kernel::ratio::dollars;
    let (magi, status) = match c.person {
        CoverdellContributorKind::Organization => return Some(COVERDELL_LIMIT_DOLLARS),
        CoverdellContributorKind::Unavailable => return None,
        CoverdellContributorKind::Filer { .. } => (magi, status),
        CoverdellContributorKind::OtherPerson { modified_agi, status } => {
            (dollars(modified_agi), status)
        }
    };
    let (start, span) = if status == FederalStatus::Joint {
        (COVERDELL_JOINT_START_DOLLARS, COVERDELL_JOINT_SPAN_DOLLARS)
    } else {
        (COVERDELL_SINGLE_START_DOLLARS, COVERDELL_SINGLE_SPAN_DOLLARS)
    };
    Some(phase::linear_remaining(COVERDELL_LIMIT_DOLLARS, magi, start, span))
}

fn distributions(v: &CoverdellExcess, d: &crate::FederalDraft) -> i64 {
    v.distributions
        + d.income
            .iter()
            .filter_map(|entry| {
                let crate::FederalIncome::EducationDistribution { data } = &entry.income else {
                    return None;
                };
                let data = &crate::FederalEducationDistribution::from(data);
                if !matches!(data.account, crate::EducationAccount::Coverdell { .. })
                    || data.answer_issue().is_some()
                    || beneficiary(entry, data) != Some(v.beneficiary)
                {
                    return None;
                }
                match data.transfer {
                    crate::Q1099Transfer::Withdrawal { .. } => Some(data.gross - data.redeposited),
                    crate::Q1099Transfer::TrusteeTransfer => Some(data.taxable_transfer_part),
                    _ => None,
                }
            })
            .sum::<i64>()
}

fn rows(prior: i64, limit: i64, net: i64, distributions: i64, value: i64) -> [i64; 8] {
    use crate::kernel::ratio::{fraction, positive};
    let room = positive(limit - net);
    let absorbed = room + distributions;
    let old =
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(prior, absorbed)
            .get();
    let current = positive(net - limit);
    let carry = old + current;
    let tax = fraction(phase::cap(carry, [value]), COVERDELL_EXCESS_RATE_PERCENT, 100);
    [prior, room, distributions, absorbed, old, current, carry, tax]
}

pub(crate) fn calculate(
    d: &crate::FederalDraft,
    magi: i64,
    status: FederalStatus,
    r: &mut crate::FederalResult,
) -> [i64; 2] {
    use crate::{kernel::ratio::dollars, line_emission::form_line};
    let reconciled = rule(&d.coverdell_excess, d).is_none();
    crate::require_localized!(
        r,
        reconciled,
        &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
        "Coverdell correction source",
        "ISSUE_COVERDELL_CORRECTION_REVIEW",
        "A recorded Coverdell correction no longer agrees with its linked income source. Reconcile the actual principal, earnings and beneficiary before its additional tax is computed.",
    );
    if !reconciled {
        return [0; 2];
    }
    let mut total = [0; 2];
    for v in &d.coverdell_excess {
        let context = crate::IssueContext::owner(crate::TaxSection::OtherTaxes, v.beneficiary);
        let person = d.people.as_ref().and_then(|p| {
            if v.beneficiary == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        });
        let eligible = person.is_some_and(|p| {
            v.contributors.iter().flat_map(|c| &c.contributions).all(|c| {
                let deemed = phase::cap(c.paid, [crate::year_constants::TAX_YEAR_END_DATE]);
                deemed >= p.birth_date
                    && (!crate::federal_model::attained_on(p.birth_date, 18, false, deemed)
                        || v.special_needs)
            })
        });
        crate::require_localized!(
            r,
            eligible,
            &context,
            "Coverdell contribution eligibility",
            "ISSUE_COVERDELL_ELIGIBILITY",
            "A contribution after age 18 without an established special-needs exception needs the actual account consequences under section 530; the App cannot assume a permitted account or a zero added tax.",
        );
        let known: i64 = phase::cap(
            v.contributors.iter().filter_map(|c| contributor_limit(c, magi, status)).sum::<i64>(),
            [COVERDELL_LIMIT_DOLLARS],
        );
        // A known contributor already reaching the global cap fixes the
        // aggregate limit even if another contributor's own limit is unknown.
        let uncertain = known < COVERDELL_LIMIT_DOLLARS
            && (v.contributors.is_empty()
                || v.contributors.iter().any(|c| contributor_limit(c, magi, status).is_none()));
        let gross_cents: i64 =
            v.contributors.iter().flat_map(|c| &c.contributions).map(|c| c.amount).sum();
        let returned_cents: i64 = v.corrections.iter().map(|c| c.principal).sum();
        let net = dollars(gross_cents - returned_cents);
        // IRC4973(e)(1)(B) carries the preceding excess, not the preceding
        // tax. Section4973(a)'s account-value cap can make that tax zero
        // without extinguishing excess. The Sep4,2026 draft line26 condition
        // conflicts in this case; retain actual source excess and disclose it.
        let prior = dollars(v.prior_excess);
        if prior > 0 && v.prior_tax == 0 {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_COVERDELL_PRIOR_ZERO_TAX"),
                context.clone(),
            );
        }
        let dist = dollars(distributions(v, d));
        let value = dollars(v.value);
        let low = rows(prior, known, net, dist, value);
        let high =
            rows(prior, if uncertain { COVERDELL_LIMIT_DOLLARS } else { known }, net, dist, value);
        let determined = low[6..] == high[6..] && (!uncertain || low[4..] == high[4..]);
        crate::require_localized!(
            r,
            determined,
            &context,
            "Coverdell contributor limit",
            "ISSUE_COVERDELL_LIMIT_UNAVAILABLE",
            "The actual contributor limit records do not establish the beneficiary's section 4973(e) tax and next-year excess. No donor or contribution limit is invented.",
        );
        // Section 530(d)(4)(C) covers a timely returned contribution;
        // section 4973(e)(2)(A) excludes it without an original-excess cap.
        // The source rule above bounds returns by actual dated contributions.
        if !eligible || !determined {
            continue;
        }
        if uncertain {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_COVERDELL_LIMIT_ENDPOINTS"),
                context.clone(),
            );
        }
        // Keep the final beneficiary-wide line33 amount after its native
        // limits and rounding; the contributor is not the tax's owner.
        total[usize::from(v.beneficiary == Owner::Spouse)] += low[7];
        let owner = if v.beneficiary == Owner::Taxpayer { "taxpayer" } else { "spouse" };
        for (i, amount) in low.into_iter().enumerate() {
            if uncertain && [1, 3].contains(&i) {
                continue;
            }
            form_line(
                r,
                crate::FormLine::new(crate::TaxForm::Form5329, &alloc::format!("{}", i + 26)),
                &alloc::format!("coverdell_{owner}_line_{}", i + 26),
                amount,
                &["coverdell_excess"],
                if i == 0 {
                    "https://www.govinfo.gov/content/pkg/USCODE-2024-title26/html/USCODE-2024-title26-subtitleD-chap43-sec4973.htm"
                } else {
                    "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf"
                },
                msg!("EXPLAIN_COVERDELL_PART_V"),
            );
        }
    }
    total
}
