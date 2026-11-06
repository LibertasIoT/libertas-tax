use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior regular-tax net operating losses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NolHistory {
    /// None
    None,
    /// Retained regular-tax loss records
    Records {
        /// Loss years
        /// One entry per calendar loss year, after all required carrybacks and intervening returns. Enter the remaining regular NOL, not the original loss or AMT balance. The App uses the oldest eligible year first.
        #[libertas_size(min = 1)]
        years: Vec<NolYear>,
        /// Complete same-filer operating-loss history
        /// Confirm the same taxpayer or unchanged joint-filing couple throughout the loss and intervening years. Separate filers use only their own reconciled balances. These losses come entirely from U.S.-source nonpassive operating businesses, with no investment-income component or ordinary-business section 461 excess-business-loss component. Identify any PTP excess-business-loss component separately for each year; the business origin alone never establishes its qualified portion. No unresolved carryback, farming, foreign allocation, death, ownership change, estate/trust allocation or prior-year debt-discharge reduction. Other histories need further allocation worksheets; AMT losses are entered separately. Prior Forms 172 and returns show it. No leaves this loss history out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// Regular loss from a retained return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct NolYear {
    /// Calendar loss year
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Unused regular-tax loss entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Each spouse's actual surviving regular loss
    /// Only for a Form 8379 claim. Use the retained source records for this year after all earlier use. This does not change the joint deduction or AGI.
    /// [NullText]
    /// The surviving spouse shares have not been established
    /// [SetText]
    /// The actual surviving spouse shares are
    #[libertas_default($null)]
    pub injured_owners: Option<crate::InjuredNolOwners>,
    /// Actual current research-business portions of this retained loss
    pub research_parts: Option<crate::ResearchLossParts>,
    /// Actual PTP excess-business-loss character in this NOL
    /// From the retained section 461(l) and section 199A loss-tracking records, independently of the ordinary business origin.
    #[libertas_default(Unavailable)]
    pub ptp_ebl: PtpEblNol,
}

/// PTP excess-business-loss character retained inside a regular NOL
/// These are tax-suspended losses, not the tax-allowed REIT/PTP loss carry on Form 8995. Original qualified amounts already reflect the original loss year's eligibility, including SSTB restrictions; pre-2018 losses have no qualified portion. Never reapply current-year SSTB restrictions.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PtpEblNol {
    /// The PTP excess-business-loss records are unavailable
    Unavailable,
    /// The records establish no PTP excess-business-loss component
    NoPtpEbl,
    /// Actual original and surviving PTP excess-business-loss amounts
    Reported {
        /// Original PTP loss suspended by section 461(l)
        /// Use the amount that reached this limit, after basis, at-risk and PTP passive limits, rather than the earlier gross loss.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        original: i64,
        /// Original qualified portion of that PTP excess business loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        qualified: i64,
        /// PTP excess business loss still in this year's opening NOL balance
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        remaining: i64,
        /// Qualified portion still suspended in that opening PTP amount
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        remaining_qualified: i64,
        /// Each spouse's actual surviving qualified PTP portion
        /// Only for Form 8379. Financial NOL ownership alone does not establish ownership of its qualified subset.
        /// [NullText]
        /// The qualified spouse shares have not been established
        /// [SetText]
        /// The actual qualified spouse shares are
        #[libertas_default($null)]
        injured_qualified: Option<crate::InjuredNolOwners>,
    },
}

impl PtpEblNol {
    pub(crate) fn issue(
        &self,
        year: u32,
        balance: i64,
        financial_owners: Option<&crate::InjuredNolOwners>,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let Self::Reported {
            original,
            qualified,
            remaining,
            remaining_qualified,
            injured_qualified,
        } = self
        else {
            return None;
        };
        let field = if year < 2018 || *original <= 0 {
            "original"
        } else if *qualified < 0 || qualified > original {
            "qualified"
        } else if *remaining < 0 || remaining > original || *remaining > balance {
            "remaining"
        } else if *remaining_qualified < 0
            || remaining_qualified > remaining
            || remaining_qualified > qualified
            || remaining - remaining_qualified > original - qualified
        {
            "remaining_qualified"
        } else if injured_qualified.as_ref().is_some_and(|owners| {
            let [a, b] = owners.amounts();
            a < 0
                || b < 0
                || a.checked_add(b) != Some(*remaining_qualified)
                || financial_owners.is_some_and(|financial| {
                    owners
                        .amounts()
                        .into_iter()
                        .zip(financial.amounts())
                        .any(|(qualified, capacity)| qualified > capacity)
                })
        }) {
            "injured_qualified"
        } else {
            return None;
        };
        Some(ValidationIssue::new("NOL_PTP_EBL_RECONCILE").at_field(field))
    }
}

impl NolHistory {
    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Records { .. })
    }
    pub(crate) fn confirmed(&self) -> bool {
        matches!(self, Self::None | Self::Records { confirmed: true, .. })
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Records { years, .. } => {
                !years.is_empty()
                    && years.iter().enumerate().all(|(i, y)| {
                        (1900..=2025).contains(&y.year)
                            // Arithmetic safety: unique 1900..2025 source years
                            // keep the unchecked vintage sums within native i64.
                            && (0..=100000000000).contains(&y.balance)
                            && y.injured_owners.as_ref().is_none_or(|v| v.valid(y.balance))
                            && y.ptp_ebl.issue(y.year, y.balance, y.injured_owners.as_ref()).is_none()
                            && !years[..i].iter().any(|p| p.year == y.year)
                    })
            }
        }
    }
}
