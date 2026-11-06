pub use crate::hsa_source::{HsaCorrection, HsaCorrectionPayer};
use crate::validation::ValidationIssue;
// Form 8889 lines 2, 9, 14a/14b and the timely excess-contribution rules.
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, whole};
use crate::kernel::ratio::positive;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

impl HsaCorrection {
    pub(crate) fn current_employer_principal_income(&self) -> i64 {
        if self.contributed_on / 10000 == 2026
            && matches!(self.payer, HsaCorrectionPayer::Employer { in_wages: false })
        {
            self.principal
        } else {
            0
        }
    }
    pub(crate) fn income(&self) -> i64 {
        if self.returned / 10000 == 2026 { positive(self.earnings) } else { 0 }
    }
}

/// Timely HSA excess-contribution return
/// Use the custodian's completed corrective distribution (Form 1099-SA code 2), not an ordinary withdrawal, and enter it once. Split by contribution year and payer. Current-year contribution totals above include this original principal; the App subtracts it once. Exclude the corrective payment from ordinary medical/nonmedical withdrawals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaCorrectionPage {
    /// Actual returned-excess statement, if entered on Other income
    /// The existing accepted source identity; never a list position or account number. Unlinked manual corrections retain their actual custodian facts.
    /// [NullText]
    /// This correction has no separate Other income statement
    /// [SetText]
    /// The returned-excess statement is
    #[libertas_enum_source("$.hsa_return_documents")]
    pub document: Option<i64>,
    /// Original failed indirect-rollover deposit, if this return corrects one
    /// Choose the original Other-income statement, separately from the returned-excess statement above. The contribution date and year must match its actual failed cash deposit. Leave empty only for other manually reported contributions.
    /// [NullText]
    /// This correction is not for a failed indirect rollover
    /// [SetText]
    /// The original failed-rollover deposit is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.hsa_failed_deposits")]
    pub failed_deposit: Option<i64>,
    /// Year for which the contribution was designated
    // Tax structure: these corrections concern the prior or current return year (2025 or 2026).
    #[libertas_number(min = 2025, max = 2026)]
    pub year: u32,
    /// Date the employer or individual actually deposited the principal
    /// A deposit designated for a prior tax year may have been made the following year. Its Form 8889 limit uses the designated year, while employer wage and other-income timing follows the actual deposit year.
    #[libertas_date_only]
    // Tax structure: a 2025 or 2026 designated contribution may be deposited in the following calendar year.
    #[libertas_number(min = 20250101, max = 20271231)]
    pub contributed_on: u32,
    /// Principal contribution returned
    /// Original excess principal, before the custodian's gain or loss adjustment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub principal: i64,
    /// Attributable net income returned
    /// Use the custodian's allocation, including a negative amount for loss. A loss reduces the cash returned, not the principal removed from contributions. Positive earnings enter income in the withdrawal year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Date principal and all attributable net income were returned
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub returned: u32,
    /// Applicable correction deadline
    /// Use the actual return due date including a valid extension, or the documented deadline under applicable IRS relief. Confirm entitlement to any later deadline, including timely filing and amendment requirements for six-month relief.
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub deadline: u32,
    /// Original contribution source
    pub payer: HsaCorrectionPayer,
}

record_page_conversions!(HsaCorrection, HsaCorrectionPage, [&], |value, Target| {
    Target {
        document: value.document,
        failed_deposit: value.failed_deposit,
        year: value.year,
        contributed_on: value.contributed_on,
        principal: value.principal,
        earnings: value.earnings,
        returned: value.returned,
        deadline: value.deadline,
        payer: value.payer.clone(),
    }
});

impl HsaCorrectionPage {
    pub(crate) fn record(&self) -> HsaCorrection {
        self.into()
    }
}
impl HsaCorrection {
    pub(crate) fn page(&self) -> HsaCorrectionPage {
        self.into()
    }
}

/// Actual health-account returned-excess statement choice
/// These are current accepted source facts, supplied read-only by the App.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthReturnDocumentChoice {
    /// Accepted income identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Actual source and printed amounts
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: alloc::vec::Vec<u8>,
    /// Source owner
    #[libertas_hidden]
    #[libertas_read_only]
    pub owner: crate::Owner,
    /// Printed gross cash returned
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub gross: i64,
    /// Printed earnings
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub earnings: Option<i64>,
}

#[derive(Clone, Debug)]
pub(crate) enum CorrectionSource {
    Hsa { owner: crate::Owner, correction: crate::HsaCorrection },
    Archer { owner: crate::Owner, correction: crate::ArcherCorrection },
    Document { owner: crate::Owner, id: i64 },
}
pub(crate) type CorrectionReferenceIssue = (CorrectionSource, ValidationIssue);
// One transient graph check for the accepted HSA and Archer source adapters.
#[derive(Clone)]
struct CorrectionLink {
    location: CorrectionSource,
    owner: crate::Owner,
    document: Option<i64>,
    principal: i64,
    earnings: i64,
    returned: u32,
}
fn source_is(data: &crate::SaSource, hsa: bool) -> bool {
    let data = &crate::FederalMedicalDistribution::from(data);
    matches!(
        (&data.account, hsa),
        (crate::MedicalSaAccount::Hsa, true) | (crate::MedicalSaAccount::Archer, false)
    ) && matches!(data.effective_payment(), Some(crate::MedicalSaPayment::ExcessReturned))
}
fn hsa_links(entries: &[crate::FederalHsa]) -> alloc::vec::Vec<CorrectionLink> {
    entries
        .iter()
        .flat_map(|h| {
            h.corrections.iter().map(|c| CorrectionLink {
                location: CorrectionSource::Hsa { owner: h.owner, correction: c.clone() },
                owner: h.owner,
                document: c.document,
                principal: c.principal,
                earnings: c.earnings,
                returned: c.returned,
            })
        })
        .collect()
}
fn archer_links(entries: &crate::ArcherContributions) -> alloc::vec::Vec<CorrectionLink> {
    let crate::ArcherContributions::Records { owners, .. } = entries else {
        return alloc::vec::Vec::new();
    };
    owners
        .iter()
        .flat_map(|h| {
            h.corrections.iter().map(|c| CorrectionLink {
                location: CorrectionSource::Archer { owner: h.owner, correction: c.clone() },
                owner: h.owner,
                document: c.document,
                principal: c.principal,
                earnings: c.earnings,
                returned: c.returned,
            })
        })
        .collect()
}
fn references_issue(
    links: &[CorrectionLink],
    income: &[crate::FederalIncomeEntry],
    hsa: bool,
) -> Option<CorrectionReferenceIssue> {
    links.iter().find_map(|c| {
        let id=c.document?;
        (!income.iter().any(|e| e.id==id && e.owner==crate::DocumentOwner::from(c.owner) && matches!(&e.income,crate::FederalIncome::MedicalDistribution {data} if source_is(data,hsa))))
            .then(|| (c.location.clone(),ValidationIssue::new("HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE").at_field("document")))
    })
}
fn sources_issue(
    links: &[CorrectionLink],
    income: &[crate::FederalIncomeEntry],
    hsa: bool,
) -> Option<CorrectionReferenceIssue> {
    if let Some(issue) = references_issue(links, income, hsa) {
        return Some(issue);
    }
    for e in income {
        let crate::FederalIncome::MedicalDistribution { data } = &e.income else {
            continue;
        };
        if !source_is(data, hsa) {
            continue;
        }
        let form = data.internal();
        let earnings = &form.box2_earnings;
        let corrections: alloc::vec::Vec<_> = links
            .iter()
            .filter(|c| c.owner == e.owner.owned() && c.document == Some(e.id))
            .collect();
        if corrections.is_empty() {
            if hsa || earnings.is_none() {
                return Some((
                    CorrectionSource::Document { owner: e.owner.owned(), id: e.id },
                    ValidationIssue::new("HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED").at_record(),
                ));
            }
            continue;
        }
        if let Some(c) = corrections.iter().find(|c| c.returned / 10000 != crate::HEALTH_TAX_YEAR) {
            return Some((
                c.location.clone(),
                ValidationIssue::new("HEALTH_CORRECTION_DOCUMENT_YEAR").at_field("returned"),
            ));
        }
        if corrections.iter().map(|c| whole(c.principal) + whole(c.earnings)).sum::<Exact>()
            != whole(form.gross)
        {
            return Some((
                CorrectionSource::Document { owner: e.owner.owned(), id: e.id },
                ValidationIssue::new("HEALTH_CORRECTION_DOCUMENT_GROSS").at_field("gross"),
            ));
        }
        if earnings.is_some_and(|printed| {
            corrections.iter().map(|c| whole(phase::floor(c.earnings, 0))).sum::<Exact>()
                != whole(printed)
        }) {
            return Some((
                CorrectionSource::Document { owner: e.owner.owned(), id: e.id },
                ValidationIssue::new("HEALTH_CORRECTION_DOCUMENT_EARNINGS")
                    .at_field("earnings")
                    .in_field("payment"),
            ));
        }
    }
    None
}
pub(crate) fn reference_issue(
    entries: &[crate::FederalHsa],
    income: &[crate::FederalIncomeEntry],
) -> Option<CorrectionReferenceIssue> {
    references_issue(&hsa_links(entries), income, true)
}
pub(crate) fn source_issue(
    entries: &[crate::FederalHsa],
    income: &[crate::FederalIncomeEntry],
) -> Option<CorrectionReferenceIssue> {
    sources_issue(&hsa_links(entries), income, true)
}
pub(crate) fn source_rule(
    entries: &[crate::FederalHsa],
    income: &[crate::FederalIncomeEntry],
) -> Option<&'static str> {
    source_issue(entries, income).map(|(_, issue)| issue.key)
}
pub(crate) fn archer_reference_issue(
    entries: &crate::ArcherContributions,
    income: &[crate::FederalIncomeEntry],
) -> Option<CorrectionReferenceIssue> {
    references_issue(&archer_links(entries), income, false)
}
pub(crate) fn archer_source_issue(
    entries: &crate::ArcherContributions,
    income: &[crate::FederalIncomeEntry],
) -> Option<CorrectionReferenceIssue> {
    sources_issue(&archer_links(entries), income, false)
}
pub(crate) fn archer_source_rule(
    entries: &crate::ArcherContributions,
    income: &[crate::FederalIncomeEntry],
) -> Option<&'static str> {
    archer_source_issue(entries, income).map(|(_, issue)| issue.key)
}
pub(crate) fn archer_linked(
    entries: &crate::ArcherContributions,
    id: i64,
    owner: crate::Owner,
) -> bool {
    let crate::ArcherContributions::Records { owners, .. } = entries else {
        return false;
    };
    owners.iter().any(|h| h.owner == owner && h.corrections.iter().any(|c| c.document == Some(id)))
}
