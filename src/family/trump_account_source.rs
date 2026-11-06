// Trump-account contribution and corrective-return source facts.
use crate::common::Owner;
use crate::dates::source_valid_date as valid_date;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// One account beneficiary's 2026 Trump-account contributions
/// Use only on the beneficiary's own return, not on a parent's return for a dependent. The pilot payment, qualified general contributions and qualified rollovers are exempt from the $5,000 annual limit; eligible section 128 employer deposits are not. Contributions to another person's account do not create this return's Form 5329 tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpAccount {
    /// Account beneficiary on this return
    #[libertas_default(Taxpayer)]
    pub owner: Owner,
    /// Total contributions accepted in 2026
    /// Include original principal before the corrective returns below. Count a full trustee-to-trustee rollover accepted by a successor Trump account as another accepted contribution; its entire incoming amount also belongs in the exempt amount below. Contributions could not begin before July 4, 2026.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
    /// Contributions exempt from the $5,000 limit
    /// Include only section 6434 pilot payments, qualified general contributions and qualified rollover contributions. A qualified Trump-to-Trump rollover is a direct trustee transfer of the entire source balance for the same beneficiary; it is not a Form 1099-R distribution. Include the incoming rollover once here and once in total accepted contributions above, even if it contains earlier 2026 deposits. Section 128 employer contributions count toward the limit and stay in the total above, not here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Completed returns of excess principal and attributable income
    /// Enter a correction received in 2026 or later, including after the documented 2026-return filing deadline. Only principal returned by that deadline reduces 2026 excess; later distributions and their earnings belong to their receipt-year return.
    pub returns: Vec<TrumpExcessReturn>,
    /// Complete contribution and return records confirmed
    /// Confirm this beneficiary's eligible Trump account, all accepted contributions after July 3, 2026, the exempt-source classification, all corrective distributions and their Forms 1099-R. An employer's section 128 wage exclusion belongs on the contributing employee's return, which can be a different person's return. If the beneficiary died during 2026, enter the final-return death facts and a separate death-distribution income source on the acquirer's return, or the beneficiary's final return when the estate acquired it. A nonqualified account or another distribution requires separate treatment. The trustee's statements and Forms 1099-R show it. No stops the estimate: the Form 5329 additional taxes are not calculated without these records.
    pub confirmed: bool,
}

/// Returned excess Trump-account contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TrumpExcessReturn {
    /// Accepted Form 1099-R identity, when supplied by Document center
    #[libertas_hidden]
    #[libertas_default($null)]
    pub document_id: Option<i64>,
    /// Excess contribution principal, Form 1099-R box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Net income attributable to the returned excess
    /// A loss reduces cash returned but cannot exceed principal. Positive 2026 Form 1099-R box 7d income incurs the separate 100% additional tax in 2026; a 2027 return belongs to 2027 instead.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Date the completed distribution was received
    #[libertas_date_only]
    pub received: u32,
    /// Applicable 2026-return due date, including any documented extension or relief
    #[libertas_date_only]
    pub deadline: u32,
    /// The custodian's excess-contribution distribution and income allocation are confirmed
    /// The custodian's corrective-distribution statement shows it. No stops the estimate: the Form 5329 additional taxes are not calculated without it.
    pub confirmed: bool,
}

impl TrumpExcessReturn {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (i128::from(self.earnings) + i128::from(self.principal) < 0).then(|| {
            crate::validation::ValidationIssue::new("TRUMP_LOSS_ABOVE_PRINCIPAL")
                .at_field("earnings")
        })
    }
    fn valid(&self) -> bool {
        (1..=100_000_000_000).contains(&self.principal)
            && (-100_000_000_000..=100_000_000_000).contains(&self.earnings)
            && self.issue().is_none()
            && valid_date(self.received)
            && valid_date(self.deadline)
            && (20260704..=20281231).contains(&self.received)
            && (20270101..=20281231).contains(&self.deadline)
    }
}
impl TrumpAccount {
    fn contribution_issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.exempt > self.contributions).then(|| {
            crate::validation::ValidationIssue::new("TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS")
                .at_field("exempt")
        })
    }
    fn returns_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.returns.iter().enumerate().find_map(|(index, returned)| {
            returned.issue().map(|issue| issue.in_list("returns", index))
        })
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.contribution_issue().or_else(|| self.returns_issue())
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.contributions)
            && (0..=100_000_000_000).contains(&self.exempt)
            && self.issue().is_none()
            && self.returns.len() <= 100
            && self.returns.iter().all(TrumpExcessReturn::valid)
    }
}
// Preserve the normal Save's group-wide failure order, then keep the retained
// shape policy in the existing valid_accounts owner.
pub(crate) fn accounts_issue(
    values: &[TrumpAccount],
) -> Option<crate::validation::ValidationIssue> {
    values
        .iter()
        .enumerate()
        .find_map(|(index, account)| {
            account.contribution_issue().map(|issue| issue.in_list("accounts", index))
        })
        .or_else(|| {
            values.iter().enumerate().find_map(|(index, account)| {
                account.returns_issue().map(|issue| issue.in_list("accounts", index))
            })
        })
        .or_else(|| {
            values.iter().enumerate().find_map(|(index, account)| {
                values[..index].iter().any(|prior| prior.owner == account.owner).then(|| {
                    crate::validation::ValidationIssue::new("TRUMP_ACCOUNT_TWICE")
                        .at_field("owner")
                        .in_list("accounts", index)
                })
            })
        })
}

pub(crate) fn valid_accounts(values: &[TrumpAccount]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| v.valid() && !values[..i].iter().any(|old| old.owner == v.owner))
}
