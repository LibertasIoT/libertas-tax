use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Unused elected amount from an original election year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Asset179Carryover {
    /// Originating election year
    #[libertas_number(min = 1958)]
    pub year: u32,
    /// Unused elected cost after prior allowed amounts and dispositions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

impl Asset179Carryover {
    pub(crate) fn k1_issue(
        rows: &[Self],
        gross: i64,
        sold: i64,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some((index, row)) = rows.iter().enumerate().find(|(_, row)| {
            !(1958..=2025).contains(&row.year) || !(1..=100000000000).contains(&row.amount)
        }) {
            return Some(
                ValidationIssue::new("K1_179_VINTAGE_BOUNDS")
                    .at_field(if !(1958..=2025).contains(&row.year) { "year" } else { "amount" })
                    .in_list("vintages", index),
            );
        }
        if let Some((index, _)) = rows
            .iter()
            .enumerate()
            .find(|(i, row)| rows[..*i].iter().any(|prior| prior.year == row.year))
        {
            return Some(
                ValidationIssue::new("K1_179_VINTAGE_TWICE")
                    .at_field("year")
                    .in_list("vintages", index),
            );
        }
        (rows.iter().map(|row| row.amount).sum::<i64>()
            != crate::kernel::carry::native_balance(gross, sold))
        .then_some(ValidationIssue::new("K1_179_VINTAGE_TOTAL").at_field("vintages"))
    }
}

/// AMT section 179 business-income-limit carryover from this K-1
/// Use your own prior AMT Form 4562 records, independently of the regular carryover. An actual zero has zero amounts and no original-year rows.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Section179IncomeCarryHistory {
    /// AMT income-limit carryover before removing the sold-property part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Part belonging to property the entity sold
    /// This restores the sold property's basis and is no longer available as a deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub sold: i64,
    /// Remaining carryover by original election year
    /// Allocate the remaining amount after the sold-property part. Keep each actual original year; do not assign an unknown year.
    #[libertas_unordered]
    /// ----
    /// Original election
    pub vintages: Vec<Asset179Carryover>,
}

impl Section179IncomeCarryHistory {
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if !(0..=100000000000).contains(&self.gross) || !(0..=self.gross).contains(&self.sold) {
            return Some(
                crate::validation::ValidationIssue::new("K1_AMT_179_CARRY_BOUNDS").at_field(
                    if !(0..=100000000000).contains(&self.gross) { "gross" } else { "sold" },
                ),
            );
        }
        Asset179Carryover::k1_issue(&self.vintages, self.gross, self.sold)
    }
}

// Project the already-owned net carry after its sold component. Prefix
// projection conserves each original year across spouses at odd cents and
// keeps its sum equal to projected gross less projected sold.
pub(crate) fn divided_179_vintages(
    rows: &[Asset179Carryover],
    sold: i64,
    share: crate::community::Share,
) -> Vec<Asset179Carryover> {
    let mut prefix = crate::kernel::ratio::Prefix::default();
    rows.iter()
        .filter_map(|row| {
            let amount =
                prefix.next(row.amount, |through| share.filer(sold + through) - share.filer(sold));
            (amount > 0).then_some(Asset179Carryover { year: row.year, amount })
        })
        .collect()
}

impl crate::community::Divide for Section179IncomeCarryHistory {
    fn divided(&self, share: crate::community::Share) -> Self {
        Self {
            gross: share.filer(self.gross),
            sold: share.filer(self.sold),
            vintages: divided_179_vintages(&self.vintages, self.sold, share),
        }
    }
}

// Original direct-asset annual history retains year identity and its owned amount.
crate::community::divide_record!(Asset179Carryover { amounts: [amount], gains: [], facts: [year] });
