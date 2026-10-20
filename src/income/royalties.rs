use crate::validation::ValidationIssue;
// Coverage: Schedule E royalty columns and Form 6251 line 2d (2026).
// Directly owned nonbusiness royalties: receipts and lines 5–19, cost/percentage
// depletion, mineral 50% and oil/gas 100% property limits, aggregate oil/gas
// 65% taxable-income limit, separate regular/AMT carryovers. K-1 royalty
// columns belong to k1_totals; federal_rules combines the Part I totals.
// Sources: IRC 611–613A, 57(a)(1); i1040se, i6251.
use crate::kernel::lines::push_line;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::{fraction, positive};
use crate::*;
use alloc::vec::Vec;
const SOURCE: &str = "https://www.irs.gov/instructions/i1040se";
impl FederalRoyalty {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        depletion_issue(&self.kind, &self.depletion).or_else(|| {
            (self.advance > self.receipts).then(|| {
                ValidationIssue::new("ROYALTY_ADVANCE_WITHIN_RECEIPTS").at_field("advance")
            })
        })
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn valid(&self) -> bool {
        let mut amounts = alloc::vec![self.receipts, self.advance];
        // Native unit counts feed the owner's i128 fractional product. The
        // retained sold <= recoverable relation bounds cost depletion by basis.
        let units = match self.depletion {
            RoyaltyDepletion::No => true,
            RoyaltyDepletion::Cost { adjusted_basis, recoverable_units, units_sold } => {
                amounts.push(adjusted_basis);
                recoverable_units >= 0 && (0..=recoverable_units).contains(&units_sold)
            }
            RoyaltyDepletion::Percentage {
                adjusted_basis,
                recoverable_units,
                units_sold,
                carryover,
                amt_carryover,
            } => {
                amounts.extend([adjusted_basis, carryover, amt_carryover]);
                recoverable_units >= 0 && (0..=recoverable_units).contains(&units_sold)
            }
        };
        amounts.iter().all(|v| (0..=100000000000).contains(v))
            && units
            && self.expenses.iter().all(|v| v.amount >= 0)
            && (self.rule().is_none() || self.rule() == Some("ROYALTY_ADVANCE_WITHIN_RECEIPTS"))
    }
}
fn depletion_issue(kind: &RoyaltyKind, depletion: &RoyaltyDepletion) -> Option<ValidationIssue> {
    if matches!(kind, RoyaltyKind::Intangible) && *depletion != RoyaltyDepletion::No {
        return Some(ValidationIssue::new("ROYALTY_INTANGIBLE_NO_DEPLETION").at_field("depletion"));
    }
    match depletion {
        RoyaltyDepletion::Cost { recoverable_units, units_sold, .. }
        | RoyaltyDepletion::Percentage { recoverable_units, units_sold, .. }
            if units_sold > recoverable_units =>
        {
            Some(
                ValidationIssue::new("ROYALTY_UNITS_SOLD_WITHIN_RECOVERABLE")
                    .at_field("units_sold")
                    .in_field("depletion"),
            )
        }
        _ => None,
    }
}
pub(crate) fn page_issue(page: &RoyaltyProperty) -> Option<ValidationIssue> {
    depletion_issue(&page.kind, &page.depletion)
}

#[derive(Clone)]
pub(crate) struct Depletion {
    pub id: i64,
    pub basis: i64,
    pub cost: i64,
    pub percentage: i64,
    pub allowance: i64,
    pub carryover: i64,
    pub oil: bool,
}
// The filer's part of a royalty's amounts on a community-property separate return (Pub. 555:
// each spouse reports half of the community income and of the deductions attributable to it;
// section 613A(c) percentage depletion is 15 percent of the taxpayer's own gross income from the
// property, and cost depletion follows the taxpayer's own basis). None when nothing is divided.
fn filer_share(d: &FederalDraft, entry: &FederalIncomeEntry) -> Option<crate::community::Share> {
    d.setup
        .as_ref()
        .and_then(|s| crate::community::share(s, entry.owner, entry.community, &entry.income))
}
// The filer's part of one royalty's amount.
pub(crate) fn filer_part(d: &FederalDraft, entry: &FederalIncomeEntry, amount: i64) -> i64 {
    mine(filer_share(d, entry), amount)
}
pub(crate) fn filer_part_exact(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    amount: &Exact,
) -> Exact {
    mine_exact(filer_share(d, entry), amount)
}
fn mine_exact(share: Option<crate::community::Share>, amount: &Exact) -> Exact {
    share.map_or_else(|| amount.clone(), |share| share.filer_exact(amount))
}
fn mine(share: Option<crate::community::Share>, amount: i64) -> i64 {
    share.map_or(amount, |share| share.filer(amount))
}
pub(crate) fn calculate(
    d: &FederalDraft,
    taxable_before: Option<i64>,
    amt: bool,
) -> Option<Vec<Depletion>> {
    // Percentage depletion needs this source's actual post-home expenses.
    // This prepass has no bound home invocation; preserve the unavailable
    // native output rather than substitute a zero home deduction. Cost
    // depletion remains independent of that income limit.
    if d.income.iter().any(|entry| matches!(&entry.income,
        FederalIncome::Business { data } if data.depletion.as_ref().is_some_and(|claim|
            matches!(claim.depletion, RoyaltyDepletion::Percentage { .. }))
            && (crate::home_office::uses_activity(d, entry.id)
                || (!d.home_sites.is_empty() && crate::section179_return::needed(d).unwrap_or(true))))) {
        return None;
    }
    let mut rows: Vec<_> = d
        .income
        .iter()
        .filter_map(|entry| {
            let (depletion, kind, receipts, advance, expenses, share) = match &entry.income {
                FederalIncome::Royalty { data } => {
                    let share = filer_share(d, entry);
                    (
                        &data.depletion,
                        &data.kind,
                        mine(share, data.receipts),
                        mine(share, data.advance),
                        Some(mine_exact(share, &RentalExpense::total_exact(&data.expenses))),
                        share,
                    )
                }
                FederalIncome::Business { data } => {
                    let depletion = data.depletion.as_ref()?;
                    let amt_adjustment = if amt {
                        crate::depreciation::business_amt_adjustment(
                            data,
                            crate::depreciation::mid_quarter(d),
                        ) + crate::method_change::amt_difference(&data.method_changes)
                    } else {
                        0
                    };
                    (
                        &depletion.depletion,
                        &depletion.kind,
                        data.receipts(),
                        0,
                        if matches!(depletion.depletion, RoyaltyDepletion::Percentage { .. }) {
                            Some(exact::whole(
                                data.expense_with_office(
                                    crate::section179::operating(d, entry.id, data),
                                    0,
                                ) - amt_adjustment,
                            ))
                        } else {
                            None
                        },
                        None,
                    )
                }
                _ => return None,
            };
            let (basis, units, sold, percentage, carry) = match *depletion {
                RoyaltyDepletion::No => (0, 0, 0, false, 0),
                RoyaltyDepletion::Cost { adjusted_basis, recoverable_units, units_sold } => {
                    (mine(share, adjusted_basis), recoverable_units, units_sold, false, 0)
                }
                RoyaltyDepletion::Percentage {
                    adjusted_basis,
                    recoverable_units,
                    units_sold,
                    carryover,
                    amt_carryover,
                } => (
                    mine(share, adjusted_basis),
                    recoverable_units,
                    units_sold,
                    true,
                    if amt { amt_carryover } else { carryover },
                ),
            };
            let cost =
                if units > 0 { phase::cap(fraction(basis, sold, units), [basis]) } else { 0 };
            let oil = matches!(kind, RoyaltyKind::OilGas);
            let percent = if percentage {
                let net = phase::floor(exact::whole(receipts) - expenses?, exact::whole(0));
                match *kind {
                    RoyaltyKind::OilGas => {
                        exact::rounded(&phase::cap(
                            exact::whole(fraction(positive(receipts - advance), 15, 100)),
                            [net.clone()],
                        ))
                        .expect("property percentage depletion is bounded by its native receipts")
                            + carry
                    }
                    RoyaltyKind::Mineral { rate } => {
                        let ceiling = fraction(receipts, rate.hundredths(), 10000);
                        // min(percentage, trunc(net / 2)): a wide net already
                        // above twice the native percentage ceiling needs no
                        // projection. Below it, the existing native quotient
                        // is safe (the actual mineral rates are at most 22%).
                        if net >= exact::whole(ceiling) * exact::whole(2) {
                            ceiling
                        } else {
                            crate::kernel::ratio::trunc_i64(
                                exact::rounded(&net)
                                    .expect("net below twice the native mineral ceiling"),
                                2,
                            )
                        }
                    }
                    RoyaltyKind::Intangible => 0,
                }
            } else {
                0
            };
            Some(Depletion {
                id: entry.id,
                basis,
                cost,
                percentage: percent,
                allowance: phase::floor(cost, percent),
                carryover: 0,
                oil,
            })
        })
        .collect();
    let total: Exact =
        rows.iter().filter(|row| row.oil).map(|row| exact::whole(row.percentage)).sum();
    // The taxable-income operand is whole dollars. Its 65% limit in cents
    // is exactly dollars × 65, with no native aggregate or intermediate product.
    let limit = taxable_before.map_or_else(
        || total.clone(),
        |base| phase::cap(exact::whole(positive(base)) * exact::whole(65), [total.clone()]),
    );
    // Reuse the return's exact-cent allocator for the aggregate 65% limit.
    let weights: Vec<_> = rows.iter().map(|r| if r.oil { r.percentage } else { 0 }).collect();
    let allocated = crate::kernel::ratio::allocate_remaining_exact(&limit, &weights)?;
    for (row, allowed) in rows.iter_mut().zip(allocated) {
        if row.oil {
            row.carryover = row.percentage - allowed;
            // None is the statutory projection before any oil/gas depletion,
            // including cost depletion. The actual pass applies the cost floor.
            row.allowance =
                if taxable_before.is_none() { 0 } else { phase::floor(row.cost, allowed) };
        } else if amt {
            row.allowance = phase::cap(row.allowance, [row.basis]);
        }
    }
    Some(rows)
}
pub(crate) fn total(rows: &[Depletion]) -> Exact {
    rows.iter().map(|r| exact::whole(r.allowance)).sum()
}
pub(crate) fn report(
    d: &FederalDraft,
    rows: &[Depletion],
    amt: bool,
    r: &mut FederalResult,
) -> bool {
    let prefix = if amt { "amt_" } else { "" };
    for row in rows {
        let Some(entry) = d.income.iter().find(|e| e.id == row.id) else {
            continue;
        };
        if let FederalIncome::Business { .. } = &entry.income {
            if !amt {
                push_line(
                    r,
                    Some(FormLine::new(TaxForm::ScheduleC, "12")),
                    &alloc::format!("business_{}_depletion", row.id),
                    row.allowance,
                    &["income.business.depletion"],
                    "https://www.irs.gov/instructions/i1040sc",
                    crate::Message::new("EXPLAIN_BUSINESS_DEPLETION", alloc::vec![]),
                );
            }
            line_business_depletion(r, row, prefix);
            if row.oil {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new("NOTE_DEPLETION_65_PERCENT_LIMIT", alloc::vec![]),
                    crate::interview::income_issue_context(entry),
                );
            }
            continue;
        }
        let FederalIncome::Royalty { data } = &entry.income else {
            continue;
        };
        let share = filer_share(d, entry);
        if !amt {
            crate::require_localized!(
                r,
                data.advance <= data.receipts,
                &crate::interview::income_issue_context(entry),
                &entry.label,
                "ROYALTY_ADVANCE_WITHIN_RECEIPTS",
                "Lease bonuses and advance royalties cannot exceed royalties including linked Forms 1099-MISC.",
            );
            let expenses = mine_exact(share, &RentalExpense::total_exact(&data.expenses));
            let receipts = mine(share, data.receipts);
            let totals = [
                exact::whole(receipts),
                exact::whole(row.allowance),
                &expenses + exact::whole(row.allowance),
                exact::whole(receipts) - &expenses - exact::whole(row.allowance),
            ];
            let categories = RentalExpense::category_totals_exact(&data.expenses)
                .map(|amount| mine_exact(share, &amount));
            let native = totals
                .iter()
                .chain(categories.iter())
                .map(exact::rounded)
                .collect::<Option<Vec<_>>>();
            let Some(native) = native else {
                crate::extended_rules::expense_output_unavailable(r, Some(row.id));
                return false;
            };
            emit_lines(
                r,
                &mut (),
                [
                    ("4", "line4_royalties", native[0]),
                    ("18", "line18_depletion", native[1]),
                    ("20", "line20_total_expenses", native[2]),
                    ("21", "line21_income", native[3]),
                ],
                |(line, suffix, amount), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(TaxForm::ScheduleE, line)),
                        &alloc::format!("royalty_{}_{suffix}", row.id),
                        LineValue::MoneyCents(amount),
                        &["income.royalty"],
                        SOURCE,
                        msg!("EXPLAIN_ROYALTY_COLUMN"),
                    ))
                },
            );
            for (index, &amount) in native[4..].iter().enumerate() {
                if amount == 0 {
                    continue;
                }
                let suffix = RentalExpenseCategory::LINES[index].0;
                push_line(
                    r,
                    Some(FormLine::new(
                        TaxForm::ScheduleE,
                        crate::federal_rules::printed_line(suffix),
                    )),
                    &alloc::format!("royalty_{}_expense_{suffix}", row.id),
                    amount,
                    &["income.royalty.expenses"],
                    SOURCE,
                    msg!("EXPLAIN_ROYALTY_COLUMN"),
                );
            }
        }
        emit_lines(
            r,
            &mut (),
            [
                ("depletion_carryover", row.carryover),
                ("basis_end", positive(row.basis - row.allowance)),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("{prefix}royalty_{}_{suffix}", row.id),
                    LineValue::MoneyCents(amount),
                    &["income.royalty.depletion"],
                    "https://www.irs.gov/instructions/i6251",
                    msg!("EXPLAIN_ROYALTY_RECORDS"),
                ))
            },
        );
    }
    true
}

// The loss columns of portfolio royalties go straight to Schedule E line 25,
// outside Form 8582. The return owner combines them with K-1 royalty losses.
pub(crate) fn losses(d: &FederalDraft, rows: &[Depletion]) -> Exact {
    rows.iter()
        .map(|row| {
            d.income
                .iter()
                .find_map(|entry| match &entry.income {
                    FederalIncome::Royalty { data } if entry.id == row.id => {
                        let share = filer_share(d, entry);
                        Some(phase::cap(
                            exact::whole(mine(share, data.receipts))
                                - mine_exact(share, &RentalExpense::total_exact(&data.expenses))
                                - exact::whole(row.allowance),
                            [exact::whole(0)],
                        ))
                    }
                    _ => None,
                })
                .unwrap_or_else(|| exact::whole(0))
        })
        .sum()
}

fn line_business_depletion(r: &mut FederalResult, row: &Depletion, prefix: &str) {
    crate::kernel::lines::line_cents(
        r,
        &alloc::format!("{prefix}business_{}_depletion_carryover", row.id),
        row.carryover,
        &["income.business.depletion"],
        "https://www.irs.gov/publications/p535",
        crate::Message::new("EXPLAIN_BUSINESS_DEPLETION_CARRY", alloc::vec![]),
    );
}
