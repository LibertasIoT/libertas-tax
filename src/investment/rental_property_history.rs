// Coverage: direct-rental Form4797 mixed-use business basis/depreciation and
// original Form6198 property-loss QBI vintages. Native recognition, at-risk,
// passive, lookback and QBI owners retain their respective calculations.
// Sources: Pub544, property partly business/personal; i8995 QBI loss tracking;
// Reg1.199A-3(b)(1)(iv). Source cents are not annual dollar worksheets.
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::loss_limits::{LossLimitError, vintages};
use crate::kernel::source_fact::SourceFact;
use crate::validation::ValidationIssue;
use crate::{FederalRental, SourceCents};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// This building's actual business portion at the sale date
/// Keep records for this building or improvement alone, excluding land and any
/// component already disposed of separately. Amounts include the current year.
/// Do not estimate earlier business use from this year's personal-use percentage.
/// These sale records do not replace the current rental cost, service dates or
/// actual current use. Those facts must independently support this year's
/// deduction; a different current depreciable basis is not established here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalBuildingSaleHistory {
    /// Business portion of cost or other gain basis before depreciation
    /// Exclude land, separately disposed components and sale expenses. Include
    /// credit basis adjustments in depreciation below, as Form 4797 requires.
    #[libertas_default(Unavailable)]
    pub basis: SourceCents,
    /// Business depreciation allowed or allowable through this sale, regular tax
    /// Include the net credit basis reduction, after any recapture restores basis.
    #[libertas_default(Unavailable)]
    pub depreciation: SourceCents,
    /// Business depreciation allowed or allowable through this sale, AMT
    /// Use separately tracked AMT records even when the amount equals regular tax.
    #[libertas_default(Unavailable)]
    pub amt_depreciation: SourceCents,
    /// Regular additional depreciation subject to section 1250 recapture
    /// Needed only when a credit basis reduction creates additional depreciation;
    /// use the sale's Form 4797 records, not a full-use straight-line estimate.
    #[libertas_default(Unavailable)]
    pub additional: SourceCents,
    /// Separately tracked AMT additional depreciation for section 1250
    #[libertas_default(Unavailable)]
    pub amt_additional: SourceCents,
}

/// Original QBI loss-tracking worksheet for this rental's at-risk property loss
/// Use one row for each actual year of disallowance under section 465. Keep
/// pre-2018 years separately with documented zero QBI. These are regular-tax
/// QBI records; the separate AMT property-loss carry remains on Form 6198.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalPropertyLossYear {
    /// Original year this property-sale loss was disallowed under section 465
    #[libertas_number(min = 1)]
    pub year: u32,
    /// Original property loss disallowed in that year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub original: i64,
    /// Qualified part of the original disallowed loss
    /// Apply the original year's business qualification, not today's rental status.
    #[libertas_default(Unavailable)]
    pub qualified: SourceCents,
    /// Property loss still suspended at the start of this return year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining: i64,
    /// Qualified part still suspended at the start of this return year
    #[libertas_default(Unavailable)]
    pub remaining_qualified: SourceCents,
}

fn known(value: &SourceCents) -> Option<i64> {
    match value.source_fact().ok()? {
        SourceFact::Reported(value) => Some(value),
        SourceFact::NotReported => Some(0),
        SourceFact::Unavailable => None,
    }
}

impl RentalBuildingSaleHistory {
    pub(crate) fn valid(&self, cost: i64) -> bool {
        cost >= 0
            && [
                &self.basis,
                &self.depreciation,
                &self.amt_depreciation,
                &self.additional,
                &self.amt_additional,
            ]
            .iter()
            .all(|source| {
                source.source_fact().is_ok() && known(source).is_none_or(|amount| amount <= cost)
            })
            && [
                (&self.additional, &self.depreciation),
                (&self.amt_additional, &self.amt_depreciation),
            ]
            .iter()
            .all(|(additional, depreciation)| {
                known(additional).is_none_or(|additional| {
                    known(depreciation).is_none_or(|depreciation| additional <= depreciation)
                })
            })
            && known(&self.basis).is_none_or(|basis| {
                (0..=cost).contains(&basis)
                    && [&self.depreciation, &self.amt_depreciation]
                        .iter()
                        .all(|source| known(source).is_none_or(|amount| amount <= basis))
            })
    }

    pub(crate) fn amounts(&self, cost: i64, amt: bool, credit: bool) -> Option<(i64, i64, i64)> {
        self.valid(cost).then_some(())?;
        let additional = if amt { &self.amt_additional } else { &self.additional };
        if !credit && known(additional).is_some_and(|amount| amount != 0) {
            return None;
        }
        Some((
            known(&self.basis)?,
            known(if amt { &self.amt_depreciation } else { &self.depreciation })?,
            if credit { known(additional)? } else { 0 },
        ))
    }
}

pub(crate) fn history_issue(
    history: &[RentalPropertyLossYear],
    expected: i64,
) -> Option<ValidationIssue> {
    let bad = || ValidationIssue::new("ACTIVITY_QBI_HISTORY_UNAVAILABLE");
    for (index, row) in history.iter().enumerate() {
        let invalid = if row.year == 0
            || row.year >= crate::year_constants::TAX_YEAR
            || history[..index].iter().any(|previous| previous.year == row.year)
        {
            Some("year")
        } else if row.original <= 0 {
            Some("original")
        } else if !(0..=row.original).contains(&row.remaining) {
            Some("remaining")
        } else if row.qualified.source_fact().is_err()
            || known(&row.qualified)
                .is_some_and(|q| q > row.original || (row.year < 2018 && q != 0))
        {
            Some("qualified")
        } else if row.remaining_qualified.source_fact().is_err()
            || known(&row.remaining_qualified).is_some_and(|q| {
                q > row.remaining
                    || (row.year < 2018 && q != 0)
                    || known(&row.qualified).is_some_and(|original| {
                        q > original || row.remaining - q > row.original - original
                    })
            })
        {
            Some("remaining_qualified")
        } else {
            None
        };
        if let Some(field) = invalid {
            return Some(bad().at_field(field).in_list("property_qbi_history", index));
        }
    }
    // Empty is an unanswered worksheet, allowed to save but never assumed zero.
    if !history.is_empty()
        && crate::kernel::ratio::checked_sum(history.iter().map(|row| row.remaining))
            != Some(expected)
    {
        return Some(bad().at_field("property_qbi_history"));
    }
    None
}

fn amount(value: i64) -> Result<Amount<Cents>, LossLimitError> {
    Amount::new(value).map_err(|_| LossLimitError::InvalidAmount)
}

pub(crate) fn qualified_release(
    source: &FederalRental,
    released: i64,
    after_passive: i64,
) -> Result<i64, LossLimitError> {
    let expected = source.at_risk.as_ref().ok_or(LossLimitError::Footing)?.prior_sale(false);
    if released == 0 || after_passive == 0 {
        return Ok(0);
    }
    if source.property_qbi_history.is_empty()
        || history_issue(&source.property_qbi_history, expected).is_some()
    {
        return Err(LossLimitError::Footing);
    }
    let rows = source
        .property_qbi_history
        .iter()
        .map(|row| {
            Ok(vintages::LossVintage::<_, Cents> {
                key: row.year,
                original: amount(row.original)?,
                original_qualified: amount(known(&row.qualified).ok_or(LossLimitError::Footing)?)?,
                remaining: amount(row.remaining)?,
                remaining_qualified: amount(
                    known(&row.remaining_qualified).ok_or(LossLimitError::Footing)?,
                )?,
            })
        })
        .collect::<Result<Vec<_>, LossLimitError>>()?;
    let mut order: Vec<_> = (0..rows.len()).collect();
    order.sort_by_key(|&index| rows[index].key);
    let SourceFact::Reported(release) =
        vintages::release_vintages(SourceFact::Reported(vintages::VintageInput {
            vintages: &rows,
            amount: amount(released)?,
            year: crate::year_constants::TAX_YEAR,
            order: &order,
        }))?
    else {
        return Err(LossLimitError::Footing);
    };
    Ok(release
        .after_next_limit(amount(after_passive)?, crate::year_constants::TAX_YEAR)?
        .allowed_qualified
        .get())
}

// Embedded source records stay bound to the actual rental/building. Community
// projection divides money through its existing owner, preserving source facts.
impl crate::community::Divide for SourceCents {
    fn divided(&self, share: crate::community::Share) -> Self {
        match self {
            Self::Reported { amount } => Self::Reported { amount: amount.divided(share) },
            other => other.clone(),
        }
    }
}
crate::community::divide_record!(RentalBuildingSaleHistory {
    amounts: [basis, depreciation, amt_depreciation, additional, amt_additional],
    gains: [],
    facts: [],
});

// Project actual source cells once. Remaining losses have the Form6198
// aggregate boundary; already consumed portions follow. Chronological order
// and qualified-before-nonqualified source cells declare the cent residual
// order. Share::filer keeps the physical owner's rounding and the other
// spouse's exact residual for every source cell, without separate ratios.
pub(crate) fn divided_history(
    history: &[RentalPropertyLossYear],
    share: crate::community::Share,
) -> Vec<RentalPropertyLossYear> {
    let Some(total) = crate::kernel::ratio::checked_sum(history.iter().map(|row| row.remaining))
    else {
        return history.to_vec(); // Invalid source remains invalid, never a zero history.
    };
    if history_issue(history, total).is_some() {
        return history.to_vec();
    }
    let mut rows = history.to_vec();
    rows.sort_by_key(|row| row.year);
    let original_fits =
        crate::kernel::ratio::checked_sum(rows.iter().map(|row| row.original)).is_some();
    let qualified = rows
        .iter()
        .map(|row| Some((known(&row.qualified)?, known(&row.remaining_qualified)?)))
        .collect::<Option<Vec<_>>>()
        .filter(|_| original_fits);
    let mut prefix = crate::kernel::ratio::Prefix::default();
    let mut cells = alloc::vec![[0_i64; 4]; rows.len()];
    for (index, row) in rows.iter().enumerate() {
        if let Some(qualified) = &qualified {
            cells[index][0] = prefix.next(qualified[index].1, |amount| share.filer(amount));
            cells[index][1] =
                prefix.next(row.remaining - qualified[index].1, |amount| share.filer(amount));
        } else {
            cells[index][1] = prefix.next(row.remaining, |amount| share.filer(amount));
        }
    }
    for (index, row) in rows.iter().enumerate() {
        if let Some(qualified) = &qualified {
            let used_qualified = qualified[index].0 - qualified[index].1;
            cells[index][2] = prefix.next(used_qualified, |amount| share.filer(amount));
            cells[index][3] = prefix
                .next(row.original - row.remaining - used_qualified, |amount| share.filer(amount));
        } else {
            // A cumulative original too wide for Prefix cannot establish a
            // qualified share. Retain the ordinary carry and per-cell spouse
            // residuals; positive qualified amounts remain named unavailable.
            cells[index][3] = if original_fits {
                prefix.next(row.original - row.remaining, |amount| share.filer(amount))
            } else {
                share.filer(row.original - row.remaining)
            };
        }
    }
    rows.into_iter()
        .zip(cells)
        .filter_map(|(row, cells)| {
            let remaining = cells[0] + cells[1];
            let original = remaining + cells[2] + cells[3];
            if original == 0 {
                return None;
            }
            Some(RentalPropertyLossYear {
                year: row.year,
                original,
                remaining,
                qualified: if qualified.is_some() {
                    projected_fact(&row.qualified, cells[0] + cells[2])
                } else {
                    unknown_share(&row.qualified)
                },
                remaining_qualified: if qualified.is_some() {
                    projected_fact(&row.remaining_qualified, cells[0])
                } else {
                    unknown_share(&row.remaining_qualified)
                },
            })
        })
        .collect()
}

fn projected_fact(source: &SourceCents, amount: i64) -> SourceCents {
    match source {
        SourceCents::Reported { .. } => SourceCents::Reported { amount },
        other => other.clone(),
    }
}

fn unknown_share(source: &SourceCents) -> SourceCents {
    if known(source) == Some(0) { source.clone() } else { SourceCents::Unavailable }
}
