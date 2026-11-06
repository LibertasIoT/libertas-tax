use crate::InheritedRothBasis;
use crate::roth_history::{RothConversion, RothHistory, RothOrdering};
use crate::roth_source::{InheritedLink, RothCode, RothWithdrawal, age_half};
// Pub. 590-B chapter 2 and Form 8606 Part III.
// Coverage: Roth IRA distributions on the 1099-R (unit u6-roth, 2026-09-28).
//   Nonqualified inherited Roth QHFD coordinates actual dated ordinary/funding
//   payments within its beneficiary/decedent group, preserving regular and
//   conversion classes. Same-day funding uses unique actual source sequence1..100;
//   source IDs/list order never establish chronology. Missing or duplicate order
//   leaves funding unverified. Tests: roth_same_day_funding_*,
//   inherited_roth_same_day_funding_* and inherited_roth_earlier_funding_* in passthrough
//   and interview; Notice 2008-51 pp. 2–4 and Pub. 590-B beneficiary ordering.
//   The Retirement topic's 1099-R box 7 union has a "Q, J or T — Roth IRA" branch (RothIraCode)
//   stored as the Roth income kind (RothWithdrawal with its RothCode, exception and repayment), and the
//   topic's "Roth IRA history" kind (RothIraHistory onto FederalAdjustments.roth_history, one per
//   owner). Code Q is qualified as the payer reports it: line 4a gross, line 4b $0, no history
//   (Form 1040 lines 4a-4b Exception 2(b)). Codes J and T use Form 8606 Part III: line 19 the
//   nonqualified distributions, line 22 regular contributions (history plus 2026 contributions),
//   line 24 conversions oldest first (taxable part before nontaxable, plus 2026 conversions),
//   line 25c earnings on line 4b; a distribution on or after 59½ or with disability and a first
//   Roth year of 2021 or earlier is qualified (Exception 2(a) for T). Form 5329 line 1 adds the
//   earnings and the recapture of taxable conversions from 2022-2026 unless 59½ or disabled.
//   Unit B11 (2026-10-12) removed the refusals: a Roth SIMPLE IRA (box 7b, J printed with S), any
//   IRA exception, a first-home distribution, a rollover, a repayment and a qualified disaster share
//   are calculated (see the retirement.rs Coverage block); a history's other basis adjustments are
//   part of the two numbers the person keeps (RothOrdering). Still refused by name: an inherited Roth
//   IRA without its record and code T for age dated before 59½.
//   Both sides: ROTH_HISTORY_TWICE, ROTH_CONVERSION_YEAR_TWICE, ROTH_CONVERSION_BEFORE_FIRST_YEAR,
//   ROTH_DISABLED_AFTER_DISTRIBUTION, R_ROTH_OTHER_BOXES, R_ROTH_DISASTER_KIND, PSO_NOT_FROM_AN_IRA
//   (now also a Roth code). Verified against the 2026 draft Form 8606 and instructions, the
//   2026 draft Form 5329 instructions (line 1, recapture example), the 2026 Instructions for
//   Forms 1099-R (codes J, Q, T and their combinations; box 7b) and the 2025 Form 1040
//   instructions (no 2026 draft yet); no calculation error found. STORAGE_FORMAT 74.
//   Tests: tests/retirement/roth_pages.rs (fixtures code_q_is_qualified_with_no_history_by_hand,
//   code_j_covered_by_contribution_basis_by_hand,
//   code_j_into_conversions_inside_five_years_is_the_form_5329_example,
//   code_t_for_age_taxes_earnings_without_the_additional_tax_by_hand; boundaries
//   age_59_and_a_half_is_six_months_after_the_59th_birthday,
//   five_tax_year_period_for_qualification_starts_with_the_first_year,
//   conversion_five_year_clock_ends_with_its_fifth_tax_year,
//   contribution_basis_exactly_used_leaves_nothing_taxable; code_j_and_history_round_trip,
//   rare_roth_situations_are_refused_by_name, roth_page_rules_reject_on_the_server, walk tests).
//   Unit6 PRINT (2026-10-13): own-account PartIII19–25c rows retain annual
//   opening contribution/conversion basis, first-home/disaster/repayment cents
//   and actual form stop rules. Qualified-only payments/complete rollovers do
//   not create this part. Financial ordering and original oracles are unchanged.
//   Tests: print_roth_form_*, existing F6/F20 and corpus Form8606 footings.
//   Inherited PartIII uses separate decedent copies and the existing ordered
//   funding/basis owner; tests print_inherited_roth_* verify stop rules and cents.
use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line_cents;
use crate::{
    FederalDraft, FederalIncome, FederalResult, Owner, RetirementKind, RetirementTreatment,
};
use alloc::{collections::BTreeMap, vec::Vec};

#[derive(Default)]
pub(crate) struct RothResult {
    // Form 1040 line 4b (Form 8606 line 25c) of each Roth row, after the designated disaster
    // share and the 2026 Line 25c Worksheet.
    pub taxable: BTreeMap<i64, i64>,
    // Form 5329 line 1 at 10% and at 25% (a Roth SIMPLE IRA inside its first two years) with
    // no exception of its own.
    pub penalized: [i64; 2],
    pub simple: [i64; 2],
    // Form 8606 line 20: the first-home slice, on Form 5329 lines 1 and 2 (exception 09), and
    // the gross it uses of the owner's $10,000 first-home limit.
    pub slice: [i64; 2],
    pub home_used: [i64; 2],
    // The Form 5329 line 1 amount of a row with an exception of its own, which the early
    // distribution stage (retirement::early_tax) exempts.
    pub staged: BTreeMap<i64, i64>,
    // The designated share of Form 8606 line 25b by row, taxed on Form 8915-F.
    pub designated: BTreeMap<i64, i64>,
    // Line 5 of the 2026 Line 25c Worksheet: repayments above the taxable amount.
    pub carry: [i64; 2],
}
// Both ordinary withdrawals and the nontaxable portion of qualified HSA funding
// consume basis in the same order. Funding itself never incurs conversion tax.
fn take_basis(balance: &mut i64, left: &mut i64) -> i64 {
    let used = crate::kernel::carry::native_consume_step(*left, balance);
    crate::kernel::carry::native_consume_step(used, left);
    used
}

fn allocate(
    regular: &mut i64,
    conversions: &mut [RothConversion],
    mut left: i64,
) -> (i64, i64, i64) {
    let recovered = take_basis(regular, &mut left);
    let mut recent = 0;
    for c in conversions {
        let taxed = take_basis(&mut c.taxable, &mut left);
        if c.year >= 2022 {
            recent += taxed;
        }
        take_basis(&mut c.nontaxable, &mut left);
    }
    (left, recovered, recent)
}

// Notice 2008-51 pp. 2–3 uses otherwise-taxable money first. With no
// current-year incoming Roth basis, 1.408A-6 A-4/A-8/A-9 determines the
// remaining classes after each dated nonqualified ordinary withdrawal.
// This is not an ordinary traditional-IRA pro-rata calculation. The annual
// owner below still calculates ordinary income and conversion recapture.
// With regular contributions, opening basis plus all retained annual regular
// contributions bounds basis under either event-time or annual aggregation.
// If each event has earnings covering its transfer even at that upper bound,
// funding consumes no basis; the annual owner needs no ordering adjustment.
pub(crate) struct FundingOrdering {
    pub basis: BTreeMap<i64, i64>,
    pub consumed: RothOrdering,
}

// Same-day ordinary nonqualified withdrawals commute. A funding event uses
// otherwise-taxable money first, so its position among same-day payments can
// change the funding/ordinary basis split. Require actual unique source order
// for that whole date; an entry ID or list position never supplies the fact.
fn order_funding_payments(
    payments: &mut [(i64, RothWithdrawal)],
    is_funding: impl Fn(i64) -> bool,
) -> Option<()> {
    payments.sort_by_key(|(_, payment)| (payment.distributed, payment.sequence));
    let mut start = 0;
    while start < payments.len() {
        let end = start
            + payments[start..]
                .iter()
                .take_while(|(_, p)| p.distributed == payments[start].1.distributed)
                .count();
        let day = &payments[start..end];
        if day.len() > 1
            && day.iter().any(|(id, _)| is_funding(*id))
            && (day.iter().any(|(_, p)| p.sequence.is_none())
                || day.windows(2).any(|pair| pair[0].1.sequence == pair[1].1.sequence))
        {
            return None;
        }
        start = end;
    }
    Some(())
}

// A confirmation cannot contradict actual dated payments in the same pool.
// Qualified-code dates are not entered facts and cannot establish order.
// Fully funded earlier payments are not ordinary basis activity; an ordinary
// remainder on an earlier source still counts.
pub(crate) fn funding_first_conflict(d: &FederalDraft, source: i64) -> Option<i64> {
    let entry = d.income.iter().find(|e| e.id == source)?;
    let data = entry.income.roth()?;
    if data.code == RothCode::Qualified {
        return None;
    }
    let pool = crate::inherited_ira::link_of(d, entry);
    d.income
        .iter()
        .find(|e| {
            // The retained first-funding path can carry a later ordinary
            // remainder in this same source. Its existing confirmation supplies
            // that boundary; a sequence cannot order components inside one row.
            // Only distinct actual source events can contradict it here.
            if e.id == source
                || e.owner != entry.owner
                || crate::inherited_ira::link_of(d, e) != pool
            {
                return false;
            }
            let Some(ordinary) = e.income.roth() else {
                return false;
            };
            if ordinary.code == RothCode::Qualified
                || !valid_date(ordinary.distributed)
                || ordinary.distributed > data.distributed
                || (ordinary.distributed == data.distributed
                    && ordinary.sequence.zip(data.sequence).is_some_and(|(a, b)| a > b))
            {
                return false;
            }
            let funded = d.adjustments.as_ref().map_or(0, |a| {
                a.hsa
                    .iter()
                    .filter(|h| h.owner == e.owner.owned())
                    .flat_map(|h| &h.funding)
                    .filter(|t| t.income == e.id)
                    .map(|t| t.amount)
                    .sum::<i64>()
            });
            ordinary.gross > funded
        })
        .map(|entry| entry.id)
}

pub(crate) fn funding_first_consistent(d: &FederalDraft, source: i64) -> bool {
    funding_first_conflict(d, source).is_none()
}

pub(crate) fn earlier_funding(d: &FederalDraft, owner: Owner) -> Option<FundingOrdering> {
    let a = d.adjustments.as_ref()?;
    if !a.hsa.iter().filter(|h| h.owner == owner).flat_map(|h| &h.funding).any(|t| {
        matches!(t.basis, crate::HsaFundingBasis::Roth { before_other_activity: false, .. })
    }) {
        return None;
    }
    let history = a.roth_history.iter().find(|h| h.owner == owner)?;
    if a.roth_history.iter().filter(|h| h.owner == owner).count() != 1 {
        return None;
    }
    let opening = history.ordering.as_ref()?;
    let contributions =
        if owner == Owner::Taxpayer { Some(&a.iras.taxpayer) } else { a.iras.spouse.as_ref() };
    // The annual IRA owner may absorb retained prior Roth excess into this
    // year's room. Its existing provisional amount is an upper bound on the
    // final income-limited absorption, so it belongs in the same sufficient
    // basis bound; omitting it would claim unknown incoming basis is zero.
    let incoming_regular = contributions.map_or(0, |v| v.roth)
        + crate::ira::provisional_roth_absorbed(d)[usize::from(owner == Owner::Spouse)];
    if contributions
        .is_some_and(|v| v.next_year_deposits.as_ref().is_some_and(|next| next.roth > v.roth))
        || d.income.iter().any(|e| {
            e.owner == crate::DocumentOwner::from(owner)
                && matches!(&e.income, FederalIncome::EducationDistribution { data }
                if matches!(crate::Q1099::from(&data.form).transfer, crate::Q1099Transfer::RothIra))
        })
        || d.income.iter().any(|e| {
            e.owner == crate::DocumentOwner::from(owner)
                && e.income.retirement().is_some_and(|data| {
                    data.converted_amount() != 0
                        || (data.kind == RetirementKind::Ira
                            && data.ordinary_distribution
                                == Some(RetirementTreatment::EmployerRothContribution)
                            && data.gross != 0)
                })
        })
    {
        return None;
    }
    let people = d.people.as_ref()?;
    let person =
        if owner == Owner::Taxpayer { &people.taxpayer } else { people.spouse.as_ref()? };
    let mut payments: Vec<_> = d
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Roth { data: source }
                if e.owner == crate::DocumentOwner::from(owner)
                    && crate::RothWithdrawal::from(source).inherited
                        == crate::InheritedLink::No =>
            {
                Some((e.id, crate::RothWithdrawal::from(source)))
            }
            _ => None,
        })
        .collect();
    // A qualified code has no actual entered payment date. Missing material
    // within-day order is checked from the source facts below.
    if payments.iter().any(|(_, v)| {
        !v.valid()
            || v.rolled_over != 0
            || v.repaid.is_some()
            || v.disaster != crate::DisasterDistribution::No
            || (v.exception.is_some() && v.disabled_since().is_none())
            || v.code == RothCode::Qualified
            || ((v.distributed >= age_half(person.birth_date) || v.disabled_since().is_some())
                && history.first_year <= 2021)
    }) {
        return None;
    }
    let transfers: Vec<_> = a
        .hsa
        .iter()
        .filter(|h| h.owner == owner)
        .flat_map(|h| &h.funding)
        .filter(|t| payments.iter().any(|(id, _)| *id == t.income))
        .collect();
    if !transfers.iter().any(|t| {
        matches!(t.basis, crate::HsaFundingBasis::Roth { before_other_activity: false, .. })
    }) || transfers.iter().enumerate().any(|(i, t)| {
        !t.valid()
            || !matches!(t.basis, crate::HsaFundingBasis::Roth { value, .. } if t.amount <= value)
            || transfers[..i].iter().any(|p| p.income == t.income)
            || payments
                .iter()
                .find(|(id, _)| *id == t.income)
                .is_none_or(|(_, v)| v.distributed > t.funded || v.gross != t.amount)
    }) {
        return None;
    }
    order_funding_payments(&mut payments, |id| transfers.iter().any(|t| t.income == id))?;
    let first_ordinary = payments
        .iter()
        .filter(|(id, _)| !transfers.iter().any(|t| t.income == *id))
        .map(|(_, v)| (v.distributed, v.sequence))
        .min();
    let last_funding = payments
        .iter()
        .filter(|(id, _)| transfers.iter().any(|t| t.income == *id))
        .map(|(_, v)| (v.distributed, v.sequence))
        .max();
    // Preserve the existing all-funding confirmation: it cannot agree with
    // an ordinary payment before or between these transfers.
    if first_ordinary.zip(last_funding).is_some_and(|(ordinary, funding)| ordinary <= funding)
        && transfers.iter().any(|t| {
            matches!(t.basis, crate::HsaFundingBasis::Roth { before_other_activity: true, .. })
        })
    {
        return None;
    }
    let mut regular = opening.regular;
    let mut conversions = opening.conversions.clone();
    conversions.sort_by_key(|c| c.year);
    let mut result = FundingOrdering {
        basis: BTreeMap::new(),
        consumed: RothOrdering {
            regular: 0,
            conversions: conversions
                .iter()
                .map(|c| RothConversion { year: c.year, taxable: 0, nontaxable: 0 })
                .collect(),
        },
    };
    if incoming_regular != 0 {
        // Do not infer when the regular deposits arrived or allocate incoming
        // basis to funding. This sufficient bound includes 2027 deposits
        // designated for 2026 once, as part of the retained annual total.
        let maximum_basis = regular
            + conversions.iter().map(|c| c.taxable + c.nontaxable).sum::<i64>()
            + incoming_regular;
        for transfer in transfers {
            let crate::HsaFundingBasis::Roth { value, .. } = transfer.basis else {
                return None;
            };
            if value - maximum_basis < transfer.amount {
                return None;
            }
            result.basis.insert(transfer.income, 0);
        }
        return Some(result);
    }
    for (id, payment) in payments {
        if let Some(t) = transfers.iter().find(|t| t.income == id) {
            let crate::HsaFundingBasis::Roth { value, .. } = t.basis else {
                return None;
            };
            let remaining =
                regular + conversions.iter().map(|c| c.taxable + c.nontaxable).sum::<i64>();
            let consumed = phase::cap(
                phase::floor(t.amount - phase::floor(value - remaining, 0), 0),
                [remaining],
            );
            let before = conversions.clone();
            let (_, recovered, _) = allocate(&mut regular, &mut conversions, consumed);
            result.consumed.regular += recovered;
            for ((used, before), after) in
                result.consumed.conversions.iter_mut().zip(before).zip(&conversions)
            {
                used.taxable += before.taxable - after.taxable;
                used.nontaxable += before.nontaxable - after.nontaxable;
            }
            result.basis.insert(id, consumed);
        } else {
            let _ = allocate(&mut regular, &mut conversions, payment.gross);
        }
    }
    Some(result)
}

pub(crate) struct InheritedFundingOrdering {
    pub basis: BTreeMap<i64, i64>,
    pub consumed: InheritedRothBasis,
}

// Notice 2008-51 pp. 2–4 also covers beneficiary IRAs. Nonqualified inherited
// Roth payments use the same regular/conversion/earnings order (Pub. 590-B,
// Distributions After Owner's Death), without conversion recapture. Only the
// linked beneficiary/decedent pool participates; personal contributions and
// another decedent's accounts cannot replenish this opening allocation.
pub(crate) fn inherited_earlier_funding(
    d: &FederalDraft,
    group: i64,
) -> Option<InheritedFundingOrdering> {
    let entry = d.income.iter().find(|e| e.id == group)?;
    let FederalIncome::InheritedIra { data } = &entry.income else {
        return None;
    };
    if !data.valid() || data.roth_qualified() {
        return None;
    }
    let opening = data.roth_basis()?;
    let mut payments: Vec<_> = crate::inherited_ira::payments(d, group)
        .into_iter()
        .map(|(e, _, _)| match &e.income {
            FederalIncome::Roth { data } => Some((e.id, crate::RothWithdrawal::from(data))),
            _ => None,
        })
        .collect::<Option<_>>()?;
    if payments.iter().any(|(_, payment)| {
        !payment.valid()
            || payment.rolled_over != 0
            || payment.repaid.is_some()
            || payment.disaster != crate::DisasterDistribution::No
            || (payment.exception.is_some() && payment.disabled_since().is_none())
            || payment.code == RothCode::Qualified
            || payment.distributed < data.died
    }) {
        return None;
    }
    let transfers: Vec<_> = d
        .adjustments
        .as_ref()?
        .hsa
        .iter()
        .filter(|h| h.owner == entry.owner.owned())
        .flat_map(|h| &h.funding)
        .filter(|t| payments.iter().any(|(id, _)| *id == t.income))
        .collect();
    if !transfers.iter().any(|t| {
        matches!(t.basis, crate::HsaFundingBasis::Roth { before_other_activity: false, .. })
    }) || transfers.iter().enumerate().any(|(i, t)| {
        !t.valid()
            || !matches!(t.basis, crate::HsaFundingBasis::Roth { value, .. } if t.amount <= value)
            || transfers[..i].iter().any(|other| other.income == t.income)
            || payments.iter().find(|(id, _)| *id == t.income).is_none_or(|(_, payment)| {
                payment.gross != t.amount || payment.distributed > t.funded
            })
    }) {
        return None;
    }
    let is_funding = |id| transfers.iter().any(|t| t.income == id);
    order_funding_payments(&mut payments, is_funding)?;
    let first_ordinary = payments
        .iter()
        .filter(|(id, _)| !is_funding(*id))
        .map(|(_, payment)| (payment.distributed, payment.sequence))
        .min();
    let last_funding = payments
        .iter()
        .filter(|(id, _)| is_funding(*id))
        .map(|(_, payment)| (payment.distributed, payment.sequence))
        .max();
    if first_ordinary.zip(last_funding).is_some_and(|(ordinary, funding)| ordinary <= funding)
        && transfers.iter().any(|t| {
            matches!(t.basis, crate::HsaFundingBasis::Roth { before_other_activity: true, .. })
        })
    {
        return None;
    }
    let mut regular = opening.regular;
    let mut converted = opening.converted;
    let mut result = InheritedFundingOrdering {
        basis: BTreeMap::new(),
        consumed: InheritedRothBasis { regular: 0, converted: 0 },
    };
    for (id, payment) in payments {
        let mut amount = if let Some(transfer) = transfers.iter().find(|t| t.income == id) {
            let crate::HsaFundingBasis::Roth { value, .. } = transfer.basis else {
                return None;
            };
            let remaining = regular + converted;
            let consumed = phase::cap(
                phase::floor(transfer.amount - phase::floor(value - remaining, 0), 0),
                [remaining],
            );
            result.basis.insert(id, consumed);
            consumed
        } else {
            payment.gross
        };
        let regular_used = take_basis(&mut regular, &mut amount);
        let converted_used = take_basis(&mut converted, &mut amount);
        if is_funding(id) {
            result.consumed.regular += regular_used;
            result.consumed.converted += converted_used;
        }
    }
    Some(result)
}

pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement_return::BasisResult,
    funding: &crate::hsa_funding::FundingResult,
) -> RothResult {
    let mut result = RothResult::default();
    // An inherited Roth record (Inherited IRA, Roth kind): the payments that
    // name it are one group. An absent basis is none (the notice is
    // WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE); a qualified distribution needs
    // none.
    for e in &d.income {
        let FederalIncome::InheritedIra { data } = &e.income else {
            continue;
        };
        if !data.roth() {
            continue;
        }
        let qualified = data.roth_qualified();
        let consumed = funding.inherited_basis_used.get(&e.id).copied().unwrap_or(0);
        let opening_regular = data.roth_basis().map_or(0, |v| v.regular);
        let ordered = funding.inherited_roth_ordered_basis.get(&e.id);
        let regular = phase::floor(opening_regular - ordered.map_or(consumed, |v| v.regular), 0);
        let converted = phase::floor(
            data.roth_basis().map_or(0, |v| v.converted)
                - ordered.map_or(phase::floor(consumed - opening_regular, 0), |v| v.converted),
            0,
        );
        let gross =
            phase::floor(crate::inherited_ira::gross(d, e.id) - funding.for_inherited(e.id), 0);
        let recovered = if qualified { 0 } else { phase::cap(gross, [regular]) };
        let conversion = if qualified { 0 } else { phase::cap(gross - recovered, [converted]) };
        let mut taxable = if qualified { 0 } else { gross - recovered - conversion };
        // The printed Part III lines figure from each other; line 25a crosses (D-2026-10-19-12).
        let part = (!qualified && gross > 0)
            .then(|| crate::forms::form_8606::part_three(gross, 0, regular, converted, 0, 0));
        if let Some(part) = &part {
            taxable = part.l25a * 100;
        }
        let designated = crate::inherited_ira::allocate_designated(d, r, e.id, gross, taxable);
        let designated_taxable: i64 = designated.values().sum();
        result.designated.extend(designated);
        let direct_taxable = taxable - designated_taxable;
        result.taxable.insert(e.id, direct_taxable);
        // Form8606 PartIII retains the existing decedent-specific ordering
        // and source cents after qualified HSA funding. Qualified-only and
        // empty groups do not require a nonqualified-distribution form.
        if let Some(part) = &part {
            emit_lines(
                r,
                &mut (),
                [
                    ("19", part.l19),
                    ("20", part.l20),
                    ("21", part.l21),
                    ("22", part.l22),
                    ("23", part.l23),
                    ("24", part.l24),
                    ("25a", part.l25a),
                ],
                |(number, amount), _, _| {
                    let included = match number {
                        "24" | "25a" => part.l23 > 0,
                        _ => true,
                    };
                    included.then(|| {
                        calculation_row(
                            Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                            &alloc::format!("inherited_roth_{}_form_line{number}", e.id),
                            LineValue::MoneyDollars(amount),
                            &["income.inherited_roth"],
                            "https://www.irs.gov/pub/irs-dft/i8606--dft.pdf",
                            msg!("EXPLAIN_INHERITED_ROTH_FORM"),
                        )
                    })
                },
            );
        }
        emit_lines(
            r,
            &mut (),
            [
                ("taxable", direct_taxable, msg!("EXPLAIN_INHERITED_ROTH_N_N_TAXABLE")),
                (
                    "regular_recovered",
                    recovered,
                    msg!("EXPLAIN_INHERITED_ROTH_N_N_REGULAR_RECOVERED"),
                ),
                (
                    "conversion_recovered",
                    conversion,
                    msg!("EXPLAIN_INHERITED_ROTH_N_N_CONVERSION_RECOVERED"),
                ),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("inherited_roth_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.inherited_roth"],
                    "https://www.irs.gov/publications/p590b",
                    explanation,
                ))
            },
        );
        if !qualified && data.roth_basis().is_some() {
            emit_lines(
                r,
                &mut (),
                [
                    (
                        "regular_remaining",
                        crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
                            regular, recovered,
                        )
                        .expect("inherited Roth regular recovery is capped by basis")
                        .get(),
                    ),
                    (
                        "conversion_remaining",
                        crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
                            converted, conversion,
                        )
                        .expect("inherited Roth conversion recovery is capped by basis")
                        .get(),
                    ),
                ],
                |(suffix, amount), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("inherited_roth_{}_{suffix}", e.id),
                        LineValue::MoneyCents(amount),
                        &["income.inherited_roth"],
                        "https://www.irs.gov/instructions/i8606",
                        msg!("EXPLAIN_INHERITED_ROTH_N_N"),
                    ))
                },
            );
        }
    }

    let Some(a) = &d.adjustments else {
        return result;
    };
    let qtp_transfers = crate::ira::QtpTransfers::of(&d.income);
    let qtp_excess = crate::ira::qtp_excess_by_entry(d);
    let absorbed_roth = crate::ira::provisional_roth_absorbed(d);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let index = usize::from(owner == Owner::Spouse);
        let mut withdrawals: Vec<_> = d
            .income
            .iter()
            .filter_map(|e| match &e.income {
                // An inherited withdrawal is taxed with its record's group above.
                FederalIncome::Roth { data: source } => {
                    let data = crate::RothWithdrawal::from(source);
                    (e.owner == crate::DocumentOwner::from(owner)
                        && data.inherited == InheritedLink::No)
                        .then_some((e.id, data))
                }
                _ => None,
            })
            .collect();
        let records: Vec<_> = a.roth_history.iter().filter(|v| v.owner == owner).collect();
        // A code Q distribution is qualified as the payer reports it (Form
        // 1040 lines 4a-4b, Exception 2): it needs no history unless it
        // funded an HSA.
        let needs_history = withdrawals
            .iter()
            .any(|(id, v)| v.code != RothCode::Qualified || funding.for_entry(*id) > 0);
        crate::require_localized!(
            r,
            records.len() <= 1,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Roth IRA",
            "ISSUE_ENTER_ONE_ROTH_IRA_HISTORY_PER_OWNER_817D4C12",
            "Enter one Roth IRA history per owner (on the Retirement topic): the first Roth IRA year and the contributions and conversions not yet taken out (Form 8606 lines 22 and 24).",
        );
        let person = d
            .people
            .as_ref()
            .and_then(|p| if index == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        // A withdrawal that needs a history with none entered: assumed a
        // first Roth year of 2026 (no five-year period) and no earlier
        // contributions or conversions, so it is taxed as earnings, with a
        // warning.
        let assumed = RothHistory { owner, first_year: 2026, ordering: None };
        let history = match records.first() {
            Some(history) => *history,
            None if needs_history && person.is_some() => {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_ROTH_HISTORY_ASSUMED_NONE"),
                    crate::IssueContext::owner(crate::TaxSection::Retirement, owner),
                );
                &assumed
            }
            None => {
                for (id, _) in &withdrawals {
                    crate::require_localized!(
                        r,
                        person.is_some(),
                        &crate::IssueContext::Section(crate::TaxSection::Retirement),
                        "Roth IRA",
                        "ISSUE_CONFIRM_ORDINARY_OWN_ACCOUNT_WITHDRAWALS_FOR_AN_OWNER_PRESENT_ON_TH_44ECD1D8",
                        "Confirm ordinary own-account withdrawals for an owner present on this return.",
                    );
                    result.taxable.insert(*id, 0);
                }
                continue;
            }
        };
        let qtp_rollovers = qtp_transfers.owner(index);
        if !qtp_rollovers.is_empty() {
            crate::require_localized!(
                r,
                withdrawals.iter().all(|(id, _)| funding.for_entry(*id) == 0),
                &crate::IssueContext::Section(crate::TaxSection::Hsa)
                    .alternatives(&[crate::TaxSection::Hsa, crate::TaxSection::OtherIncome]),
                "529-to-Roth IRA rollover",
                "ISSUE_A_2026_QUALIFIED_ROTH_TO_HSA_FUNDING_TRANSFER_TOGETHER_WITH_529_TO_AFFC98A6",
                "A 2026 qualified Roth-to-HSA funding transfer together with 529-to-Roth transfers needs separate source-date and basis accounting.",
            );
        }
        crate::require_localized!(
            r,
            person.is_some_and(|p| history.first_year >= p.birth_date / 10000),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Roth IRA",
            "ISSUE_THE_FIRST_ROTH_IRA_YEAR_IN_THE_ROTH_IRA_HISTORY_CANNOT_BE_BEFORE_TH_8D09C080",
            "The first Roth IRA year in the Roth IRA history cannot be before the owner's birth year.",
        );
        // Every adjustment of lines 22 and 24 (a designated Roth rollover, a divorce transfer, a
        // recharacterization, the last Line 25c Worksheet) is part of the two numbers the
        // person keeps from their own Forms 8606 (R5).
        let mut regular = history.ordering.as_ref().map_or(0, |v| v.regular);
        let mut conversions =
            history.ordering.as_ref().map_or_else(Vec::new, |v| v.conversions.clone());
        conversions.sort_by_key(|v| v.year);
        if let Some(consumed) = &funding.roth_ordered_basis[index] {
            regular = crate::kernel::carry::native_balance(regular, consumed.regular);
            for c in &mut conversions {
                if let Some(used) = consumed.conversions.iter().find(|used| used.year == c.year) {
                    c.taxable = crate::kernel::carry::native_balance(c.taxable, used.taxable);
                    c.nontaxable =
                        crate::kernel::carry::native_balance(c.nontaxable, used.nontaxable);
                }
            }
        } else {
            let _ = allocate(&mut regular, &mut conversions, funding.roth_basis_used[index]);
        }
        regular += a
            .entered_iras()
            .and_then(|v| if index == 0 { Some(&v.taxpayer) } else { v.spouse.as_ref() })
            .map_or(0, |v| v.roth)
            + absorbed_roth[index];
        // Section 408A(e)(1) preserves QTP contribution/earnings character as in
        // a Roth-to-Roth rollover. Under Treas. Reg. 1.408A-6 A-8/A-9
        // (https://www.irs.gov/pub/irs-regs/td8816.pdf), annual Roth
        // distribution ordering uses the tax year's aggregated regular basis,
        // including source contributions transferred after an ordinary withdrawal.
        // Form 5498 box 10 does not turn source earnings into regular basis. The part of a
        // transfer above the year's room is not a rollover: all of it is a Roth contribution
        // (its earnings share was taxed as a 529 withdrawal).
        for rollover in qtp_rollovers {
            let excess = qtp_excess.get(&rollover.id).copied().unwrap_or(0);
            let amount = rollover.amount();
            let qualified = amount - excess;
            regular += if amount == 0 {
                0
            } else {
                crate::kernel::ratio::fraction(rollover.contributions, qualified, amount)
            } + excess;
        }
        let mut current = RothConversion { year: 2026, taxable: 0, nontaxable: 0 };
        for e in &d.income {
            if e.owner == crate::DocumentOwner::from(owner)
                && let Some(data) = e.income.retirement()
                && data.kind == RetirementKind::Ira
            {
                // An employer's Roth SEP or Roth SIMPLE contribution is reported as if converted:
                // taxable in full, a 2026 conversion for the ordering (Notice 2024-2 section L).
                if data.ordinary_distribution == Some(RetirementTreatment::EmployerRothContribution)
                {
                    current.taxable += data.gross;
                    continue;
                }
                let gross = data.converted_amount();
                let taxable = basis.converted(e.id, &data);
                current.taxable += taxable;
                current.nontaxable += gross - taxable;
            }
        }
        conversions.push(current);
        conversions.sort_by_key(|v| v.year);
        // Retain the actual annual ordering balances before withdrawals consume
        // them. Form8606 shows the available basis, not only the part recovered.
        let form_regular = regular;
        let form_conversions: i64 = conversions.iter().map(|v| v.taxable + v.nontaxable).sum();
        let last_funding = withdrawals
            .iter()
            .filter(|(id, _)| funding.for_entry(*id) > 0)
            .map(|(_, v)| v.distributed)
            .max();
        withdrawals.sort_by_key(|(id, v)| (v.distributed, v.sequence, *id));
        // Form 8606 Part III, row by row: the amount each row adds to line 19, the first-home
        // slice of line 20, then the ordering of lines 22 and 24.
        struct Row {
            id: i64,
            data: RothWithdrawal,
            left_in: i64,
            left: i64,
            recapture: i64,
            age_exempt: bool,
            slice: i64,
        }
        let mut rows: Vec<Row> = Vec::new();
        let mut home_used = 0;
        for (id, v) in withdrawals {
            crate::require_localized!(
                r,
                funding.roth_ordered_basis[index].is_some()
                    || last_funding.is_none_or(|date| {
                        v.distributed >= date || v.gross == funding.for_entry(id)
                    }),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Roth IRA",
                "ISSUE_ORDINARY_ROTH_WITHDRAWALS_BEFORE_THE_LAST_QUALIFIED_FUNDING_TRANSFE_E42B2BF2",
                "Ordinary Roth withdrawals before the last qualified funding transfer require supported own-account nonqualified, dated ordering. Current regular contributions need enough recorded earnings to cover every funding transfer without using basis; other earlier activity needs further support. Funding-only earlier entries are permitted.",
            );
            let payer_qualified = v.code == RothCode::Qualified;
            crate::require_localized!(
                r,
                person.is_some_and(|p| {
                    payer_qualified
                        || (v.distributed >= p.birth_date
                            && p.death_date().is_none_or(|date| v.distributed <= date)
                            && v.disabled_since().is_none_or(|date| date >= p.birth_date))
                }),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Roth IRA",
                "ISSUE_CONFIRM_ORDINARY_OWN_ACCOUNT_WITHDRAWALS_AND_SOURCE_DATES_INHERITED_68CEF704",
                "Confirm ordinary own-account withdrawals and source dates. Inherited, special repayment, rollover, unentered HSA funding and unlisted exception cases require further support.",
            );
            let age_exempt = person.is_some_and(|p| v.distributed >= age_half(p.birth_date));
            let exempt = age_exempt || v.disabled_since().is_some();
            // Code T for age: the payer's exception needs the owner to be
            // 59½ on the distribution date (Form 5329 line 2 otherwise).
            crate::require_localized!(
                r,
                v.code != RothCode::ExceptionApplies || exempt,
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Roth IRA",
                "ISSUE_A_1099_R_WITH_CODE_T_FOR_AGE_59_IS_DATED_BEFORE_THE_OWNER_TURNED_59_3DA81978",
                "A 1099-R with code T for age 59½ is dated before the owner turned 59½: check the distribution date and birth date, or choose disability as the reason.",
            );
            let qualified = payer_qualified || (exempt && history.first_year <= 2021);
            // Form 8606 line 19 excludes what was rolled over to another Roth IRA within 60 days
            // and the funding of an HSA.
            let base = phase::floor(v.gross - v.rolled_over - funding.for_entry(id), 0);
            // Line 20: a first-home distribution from a Roth IRA whose first contribution year is
            // 2021 or earlier, up to $10,000 less the earlier first-home distributions
            // (Pub. 590-B allocates it first). The App takes the Roth rows' share of the
            // owner's $10,000 before the traditional IRA rows (retirement::gross_exceptions).
            let mut slice = 0;
            if !qualified
                && history.first_year <= 2021
                && let Some(crate::RetirementException::FirstHome {
                    costs,
                    paid_on,
                    prior_used,
                    eligible,
                }) = &v.exception
                && *eligible
                && *paid_on >= v.distributed
                && crate::dates::days_between(v.distributed, *paid_on) <= 120
            {
                slice = phase::cap(
                    phase::cap(base, [*costs]),
                    [phase::floor(
                        crate::year_constants::FIRST_HOME_DISTRIBUTION_LIMIT_CENTS
                            - prior_used
                            - home_used,
                        0,
                    )],
                );
                home_used += slice;
            }
            let left_in = if qualified { 0 } else { base - slice };
            // Opening records not entered: no earlier contributions or
            // conversions are assumed, with a warning.
            if !qualified && history.ordering.is_none() && !records.is_empty() {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_ROTH_BASIS_ASSUMED_ZERO"),
                    crate::IssueContext::owner(crate::TaxSection::Retirement, owner),
                );
            }
            let (left, recovered, recent) = allocate(&mut regular, &mut conversions, left_in);
            rows.push(Row {
                id,
                data: v,
                left_in,
                left,
                recapture: if age_exempt { 0 } else { recent },
                age_exempt,
                slice,
            });
            result.slice[index] += slice;
            emit_lines(
                r,
                &mut (),
                [
                    ("regular_recovered", recovered, msg!("EXPLAIN_ROTH_N_N_REGULAR_RECOVERED")),
                    ("first_home", slice, msg!("EXPLAIN_ROTH_N_N_FIRST_HOME")),
                ],
                |(suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("roth_{id}_{suffix}"),
                        LineValue::MoneyCents(amount),
                        &["income.roth", "adjustments.roth_history", "adjustments.iras"],
                        "https://www.irs.gov/publications/p590b",
                        explanation,
                    ))
                },
            );
        }
        result.home_used[index] = home_used;
        // Form 8606 line 25b: the qualified disaster share of line 25a is the fraction of line
        // 21 designated; it goes to Form 8915-F (line 19) and its recapture leaves Form 5329
        // line 1. The fraction is the owner's, over all the Roth rows (i8606 line 25b).
        let designated_rows: Vec<(i64, i64)> = rows
            .iter()
            .filter_map(|row| match row.data.disaster {
                crate::DisasterDistribution::Qualified { amount, .. } => {
                    Some((row.id, phase::cap(amount, [row.left_in])))
                }
                _ => None,
            })
            .collect();
        let designated_total: i64 = designated_rows.iter().map(|(_, amount)| amount).sum();
        let line_21: i64 = rows.iter().map(|row| row.left_in).sum();
        let mut line_25b = 0;
        let mut left_final: Vec<i64> = Vec::new();
        let mut recapture_final: Vec<i64> = Vec::new();
        for row in &rows {
            let share = |amount: i64| {
                if designated_total == 0 || line_21 == 0 {
                    0
                } else {
                    crate::kernel::ratio::fraction(amount, designated_total, line_21)
                }
            };
            let taken = share(row.left);
            line_25b += taken;
            left_final.push(row.left - taken);
            recapture_final.push(row.recapture - share(row.recapture));
        }
        if designated_total > 0 {
            for (id, amount) in &designated_rows {
                result.designated.insert(
                    *id,
                    crate::kernel::ratio::fraction(line_25b, *amount, designated_total),
                );
            }
            line_cents(
                r,
                &alloc::format!(
                    "roth_{}_designated",
                    if index == 0 { "taxpayer" } else { "spouse" }
                ),
                line_25b,
                &["income.roth", "adjustments.roth_history"],
                "https://www.irs.gov/instructions/i8606",
                msg!("EXPLAIN_ROTH_N_DESIGNATED"),
            );
        }
        // The 2026 Line 25c Worksheet: repayments within three years of a birth or adoption,
        // emergency, domestic-abuse or terminal-illness distribution reduce line 25c; what
        // exceeds it is next year's line 22 (i8606 p. 16).
        let mut credited = 0;
        let mut repaid_ids: Vec<i64> = Vec::new();
        for row in &rows {
            let (id, v) = (row.id, &row.data);
            let Some(repaid) = &v.repaid else {
                continue;
            };
            if repaid.confirmed {
                use crate::kernel::ratio::exact;
                let paid: exact::Exact = repaid
                    .payments
                    .iter()
                    .filter(|p| p.date > v.distributed && p.date <= v.distributed + 30000)
                    .map(|p| exact::whole(p.amount))
                    .sum();
                // The existing actual-distribution cap precedes narrowing.
                credited += exact::rounded(&paid.min(exact::whole(v.gross)))
                    .expect("Roth repayment bounded by original distribution");
                repaid_ids.push(id);
            }
        }

        let left_before: i64 = left_final.iter().sum();
        let mut remaining = credited;
        // Repaid rows first, then the others.
        let mut order_of_reduction: Vec<usize> =
            (0..rows.len()).filter(|i| repaid_ids.contains(&rows[*i].id)).collect();
        order_of_reduction.extend((0..rows.len()).filter(|i| !repaid_ids.contains(&rows[*i].id)));
        for i in order_of_reduction {
            let reduction =
                crate::kernel::carry::native_consume_step(remaining, &mut left_final[i]);
            crate::kernel::carry::native_consume_step(reduction, &mut remaining);
        }
        if credited > 0 {
            result.carry[index] = remaining;
            if remaining > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "NOTE_ROTH_REPAYMENT_NEXT_YEAR_BASIS",
                        alloc::vec![
                            crate::MessageArg::Nested(alloc::boxed::Box::new(
                                crate::IssueContext::owner(crate::TaxSection::Retirement, owner)
                                    .label()
                            )),
                            crate::MessageArg::usd_cents(remaining)
                        ],
                    ),
                    crate::IssueContext::owner(crate::TaxSection::Retirement, owner),
                );
            }
        }
        // Project the existing PartIII calculation after disaster allocation and
        // repayment reductions. Qualified payments and complete rollovers never
        // create a Form8606 merely because a Roth income record was entered.
        // Form 8606 Part III foots on its printed whole-dollar lines, and the taxable amount that
        // crosses to the 1040 and Form 5329 is the printed line 25c (D-2026-10-19-12).
        let line_19 = line_21 + home_used;
        let part = crate::forms::form_8606::part_three(
            line_19,
            home_used,
            form_regular,
            form_conversions,
            line_25b,
            left_before - left_final.iter().sum::<i64>(),
        );
        if line_19 > 0 {
            crate::forms::form_8606::fit_total(&mut left_final, part.l25c * 100);
            let who = if index == 0 { "taxpayer" } else { "spouse" };
            emit_lines(
                r,
                &mut (),
                [
                    ("19", part.l19),
                    ("20", part.l20),
                    ("21", part.l21),
                    ("22", part.l22),
                    ("23", part.l23),
                    ("24", part.l24),
                    ("25a", part.l25a),
                    ("25b", part.l25b),
                    ("25c", part.l25c),
                ],
                |(number, amount), _, _| {
                    let included = match number {
                        "23" => part.l21 > 0,
                        "24" | "25a" => part.l23 > 0,
                        "25b" | "25c" => part.l25a > 0,
                        _ => true,
                    };
                    included.then(|| {
                        calculation_row(
                            Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                            &alloc::format!("{who}_roth_form_line{number}"),
                            LineValue::MoneyDollars(amount),
                            &["income.roth", "adjustments.roth_history", "adjustments.iras"],
                            "https://www.irs.gov/pub/irs-dft/f8606--dft.pdf",
                            msg!("EXPLAIN_ROTH_FORM_PART_III"),
                        )
                    })
                },
            );
        }
        // Form 5329 line 1 for each row.
        for (i, row) in rows.iter().enumerate() {
            let (id, v) = (row.id, &row.data);
            result.taxable.insert(id, left_final[i]);
            let pen = if row.age_exempt { 0 } else { left_final[i] + recapture_final[i] };
            if pen > 0 {
                if row.slice == 0 && v.exception.is_some() {
                    result.staged.insert(id, pen);
                } else if v.simple_early() {
                    result.simple[index] += pen;
                } else {
                    result.penalized[index] += pen;
                }
            }
            emit_lines(
                r,
                &mut (),
                [
                    ("taxable", left_final[i], msg!("EXPLAIN_ROTH_N_N_TAXABLE")),
                    (
                        "conversion_recapture",
                        recapture_final[i],
                        msg!("EXPLAIN_ROTH_N_N_CONVERSION_RECAPTURE"),
                    ),
                ],
                |(suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("roth_{id}_{suffix}"),
                        LineValue::MoneyCents(amount),
                        &["income.roth", "adjustments.roth_history", "adjustments.iras"],
                        "https://www.irs.gov/publications/p590b",
                        explanation,
                    ))
                },
            );
        }
    }
    result
}
