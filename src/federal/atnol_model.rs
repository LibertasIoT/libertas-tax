use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior alternative-tax net operating losses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AtnolHistory {
    /// None
    None,
    /// Retained AMT loss carryforward records
    Records {
        /// Loss years
        /// One entry per calendar loss year. Use the remaining ATNOL after every required carryback and intervening return, not the original loss or the regular-tax NOL. Input order does not control use.
        #[libertas_size(min = 1)]
        years: Vec<AtnolYear>,
        /// Complete same-taxpayer history and unchanged filing unit
        /// Confirm the same filer, or the same married couple filing jointly, throughout all involved years; separate filers use only their own losses with unchanged filing status. No deceased-owner transfer, estate/trust allocation, foreign-source loss, farming carryback, unresolved earlier return, prior-year debt-discharge reduction or mixed ordinary/special-exception components in one loss year. Carryback elections and remaining balances are already reconciled. Regular NOL balances and deductions are entered separately. Prior Forms 6251 and their ATNOL worksheets show it. No leaves this AMT loss history out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// AMT loss from a retained return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct AtnolYear {
    /// Calendar loss year
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Remaining alternative-tax loss entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Documented exception to the ordinary AMT loss limit
    /// Yes only when the entire remaining loss is a qualified disaster, Gulf Opportunity Zone, Kansas recovery assistance, Midwestern disaster recovery assistance loss, or a 2008/2009 loss with the documented extended-carryback election described in the Form 6251 instructions. This is a source-return classification, not an election to claim a larger deduction now. Mixed components need their own allocation worksheet.
    pub special_exception: bool,
}

impl AtnolYear {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.special_exception && self.year >= 2018).then(|| {
            crate::validation::ValidationIssue::new("ATNOL_EXCEPTION_AFTER_2017")
                .at_field("special_exception")
        })
    }
}
impl AtnolHistory {
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
                            && y.issue().is_none()
                            && !years[..i].iter().any(|prior| prior.year == y.year)
                    })
            }
        }
    }
    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Records { .. })
    }
    pub(crate) fn confirmed(&self) -> bool {
        matches!(self, Self::None | Self::Records { confirmed: true, .. })
    }
}
