// Coverage: Schedule C header and expense specials; approved 2026 scope.
// Sources: i1040sc (2026 draft), Pub. 463/538/560 (2025), IRC 611-613A/274.
// Validation audit F03 (2026-10-13): the de minimis limit tests full invoice/item
// cost; the deductible amount is its business share, which can be smaller than
// that full cost. Reg. 1.263(a)-1(f)(1)(i)(D)/(ii)(D), Notice 2015-82 and Pub334:
// https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/section-1.263(a)-1
// https://www.irs.gov/pub/irs-drop/n-15-82.pdf
// https://www.irs.gov/publications/p334 (personal and business expenses).
// Evidence: f03_de_minimis_business_share_saves_reopens_and_retains_full_cost_limit.
use crate::*;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum InformationReturnFiling {
    /// No payment required a Form 1099
    NoneRequired,
    /// Required, and filed or will be filed
    Filed,
    /// Required, but not filed
    NotFiled,
}

/// Other income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherBusinessIncome {
    /// Description
    #[libertas_size(min = 1)]
    pub description: String,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

/// Meals
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MealExpense {
    /// Kind of meals
    pub kind: MealKind,
    /// Cost before the limit
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

/// Kind of meals
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum MealKind {
    /// Business meals, 50% deductible
    /// Meals with a customer, client or business contact, and your own meals while traveling away from home overnight for business.
    Business,
    /// Meals during Department of Transportation hours-of-service duty, 80% deductible
    /// Meals away from home during or incident to a period of duty under the DOT hours-of-service limits: airline pilots, crews, dispatchers, mechanics and control tower operators under FAA rules; interstate truck and bus drivers under DOT rules; railroad engineers, conductors and crews under FRA rules; merchant mariners under Coast Guard rules.
    Transportation,
    /// Fully deductible meals
    /// Meals whose value you included in an employee's wages; meals a client reimbursed you for with adequate records; recreational meals, such as a holiday party, mainly for employees who are not highly compensated; meals you sold to customers, including food bought to prepare their meals and your employees' meals at the worksite; meals whose value is reported on a nonemployee's Form 1099-NEC; crew meals on a commercial fishing vessel.
    FullyDeductible,
}

/// De minimis safe-harbor election (Regulation 1.263(a)-1(f))
/// Deducts low-cost tangible property bought in 2026 as other expenses instead of depreciating it. The election is yours for the whole year, covers every business and rental you have, and is made by a statement with your timely filed return. Enter the purchases once, here; not as assets, supplies or inventory.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct DeMinimisElection {
    /// Purchases deducted under the election
    /// The business share of every 2026 purchase that qualifies, with the delivery and installation charges on the same invoice. Schedule C Part V line 48.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub purchases: i64,
    /// Largest invoice or item
    /// The full cost of the most expensive item, or of the invoice when items are not priced separately, before any personal-use share. Do not split one item across invoices. At most $2,500, or $5,000 with an applicable financial statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub largest_item: i64,
    /// Applicable financial statement
    /// Yes means a statement filed with the SEC, a certified audited statement used for credit, shareholders or another substantial non-tax purpose, or one required by a federal or state agency other than the IRS or SEC, and written expensing procedures in place at the start of 2026. Ordinary bookkeeping or an unaudited statement is No. Answer the same for every business and rental on this return.
    pub financial_statement: bool,
    /// Every purchase was expensed on your books under a policy in place at the start of 2026, and the election covers all your qualifying purchases of the year
    /// Not land, inventory or rotable spare parts; no item above the limit. No leaves these purchases out of the estimate with a notice; enter such an item under its ordinary treatment or as an asset instead.
    pub eligible: bool,
}

/// Depletion of an operating interest (Schedule C line 12)
/// This business is one property: an operating (working) interest in a domestic oil, gas or mineral deposit you develop as an independent producer. A second property is a second business. Timber and Form T are not calculated.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessDepletion {
    /// Kind of deposit
    pub kind: crate::RoyaltyKind,
    /// Cost or percentage depletion
    /// Percentage depletion is 15% of this business's gross income (22% to 5% by mineral), limited to the business's income before depletion (100% for oil and gas, 50% for other minerals) and to 65% of your taxable income; the excess carries to 2027.
    pub depletion: crate::RoyaltyDepletion,
}

impl DeMinimisElection {
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let limit = if self.financial_statement {
            crate::year_constants::DE_MINIMIS_AFS_LIMIT_CENTS
        } else {
            crate::year_constants::DE_MINIMIS_LIMIT_CENTS
        };
        if self.largest_item > limit {
            Some(
                crate::validation::ValidationIssue::new("DE_MINIMIS_ITEM_ABOVE_LIMIT")
                    .at_field("largest_item"),
            )
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.rule().is_none()
            && (1..=100000000000).contains(&self.purchases)
            && (1..=100000000000).contains(&self.largest_item)
    }
}
impl BusinessDepletion {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.kind, RoyaltyKind::Intangible) {
            return Some(
                crate::validation::ValidationIssue::new("BUSINESS_DEPLETION_NEEDS_A_DEPOSIT")
                    .at_field("kind"),
            );
        }
        let record = FederalRoyalty {
            related: alloc::vec![],
            kind: self.kind.clone(),
            depletion: self.depletion.clone(),
            receipts: 0,
            advance: 0,
            expenses: alloc::vec![],
            ordinary: true,
        };
        record.issue()
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn valid(&self) -> bool {
        let record = FederalRoyalty {
            related: alloc::vec![],
            kind: self.kind.clone(),
            depletion: self.depletion.clone(),
            receipts: 0,
            advance: 0,
            expenses: alloc::vec![],
            ordinary: true,
        };
        self.rule().is_none() && record.valid()
    }
}

// Materialize one calculation's business depletion as cost depletion in its
// supported working copy. The accepted source and its original carry facts stay
// unchanged. Every ordinary business consumer then reads one profit/SE/QBI value.
pub(crate) fn resolved(d: &FederalDraft, rows: &[crate::royalties::Depletion]) -> FederalDraft {
    let mut draft = d.clone();
    for entry in &mut draft.income {
        if let FederalIncome::Business { data } = &mut entry.income
            && let Some(depletion) = data.depletion.as_mut()
            && let Some(row) = rows.iter().find(|v| v.id == entry.id)
        {
            depletion.depletion = RoyaltyDepletion::Cost {
                adjusted_basis: row.allowance,
                recoverable_units: 1,
                units_sold: 1,
            };
        }
    }
    draft
}
