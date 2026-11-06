use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Foreign income included in a state income-tax base
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStateIncome {
    /// Existing ordinary foreign-income record or saved fiduciary allocation
    /// For an ordinary record, use its exact unique country and payer label. For a fiduciary allocation, select the saved K-1 and its actual country and withholding class. The link retains the existing category before federal high-tax reclassification.
    pub source: crate::ForeignAllocationSource,
    /// Income attributed to this source by the state's tax law
    /// Use the state return and supporting income records underlying this payment, not a computed tax share. Exclude income exempted by that state; federal and state income amounts may differ.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
}

/// State or local income-tax payment and its source income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStatePayment {
    /// Taxing jurisdiction and source year
    #[libertas_size(min = 1)]
    pub label: String,
    /// Eligible income tax paid, already entered in itemized expenses
    /// Exclude sales and property taxes. Split payments only where their underlying state income differs; do not enter a federal deduction or duplicate the payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub paid: i64,
    /// U.S.-source income in this payment's state tax base
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub domestic: i64,
    /// Foreign-source income in that state tax base
    /// Leave empty only when the tax relates entirely to U.S. income, including when the state exempts all foreign income. Identify each source once; all domestic and foreign amounts must reconcile to the income underlying this payment.
    pub foreign: Vec<ForeignStateIncome>,
}

/// Source allocation of itemized state and local income taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignStateTaxes {
    // This carries unavailable state-source-income workpapers, not an unknown
    // confirmation. Federal AGI cannot establish another year's state tax base.
    // When needed, allocate stops at the source-fact boundary
    // ISSUE_SUPPLY_THE_SOURCE_INCOME_UNDERLYING_ITEMIZED_STATE_LOCAL_INCOME_TAX_5BB69CE3;
    // it never invents source income or treats unavailable records as zero.
    /// Not needed or not yet reviewed
    /// Required only when an itemized state/local income-tax deduction is used. Standard deduction and general sales taxes do not require these records.
    Unreviewed,
    /// Allocate from the state income records
    Records {
        /// Payments included in the itemized income-tax amount
        #[libertas_size(min = 1)]
        payments: Vec<ForeignStatePayment>,
        /// Complete source attribution under each taxing jurisdiction's law
        /// Check each state return's source-income worksheet. No apportions the tax with your other deductions instead, with a notice. All payments reconcile to the itemized income-tax entry. Source income is nonnegative and reflects the tax base underlying each payment, including exemptions and resolved differences from federal law. No unresolved state apportionment, special election or loss allocation. Do not substitute current federal AGI when the payment relates to other income.
        confirmed: bool,
    },
}
impl ForeignStateTaxes {
    pub(crate) fn valid(&self) -> bool {
        let money = |v| (0..=100000000000).contains(&v);
        let label = |s: &str| !s.trim().is_empty();
        match self {
            Self::Unreviewed => true,
            Self::Records { payments, .. } => {
                !payments.is_empty()
                    // Temporary arithmetic exclusion: allocate sums native payment
                    // amounts before reconciliation and accumulates native weights.
                    && payments.len() <= 100
                    && payments.iter().all(|p| {
                        label(&p.label)
                            && p.paid > 0
                            && money(p.paid)
                            && money(p.domestic)
                            // Temporary arithmetic exclusion: allocate builds and
                            // totals each payment's native income-group vector.
                            && p.foreign.len() <= 100
                            && p.foreign.iter().enumerate().all(|(i, f)| {
                                f.source.valid()
                                    && money(f.income)
                                    && !p.foreign[..i].iter().any(|v| v.source.same_as(&f.source))
                            })
                    })
            }
        }
    }
}
