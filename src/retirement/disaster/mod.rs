pub use crate::disaster_carry_model::{
    DisasterCarry, EarlierForm8915, Form8915Repayments, Form8915Year,
};
use crate::disaster_source::{DisasterDistribution, DisasterSpread, LIMIT};
// PRINT projection (2026-10-13): disaster/projection.rs emits2026 draft
// PartI columns2-5c/limits/unused gross, conditional8-26/28-32 and actual
// traditional/Roth/payer21 components. Copies keep filer and disaster year;
// terminal and per-document PartIV components are supporting rows only.
// Financial amounts, source cents, repayments, timing and old source IDs stay.
// Tests: print_form_8915_*, all supported() corpus Form8915 footings.
// Unit6 inherited-IRA correction (2026-10-13): qualified traditional and
// Roth beneficiary payments consume their existing group taxable/designated
// owners and remain in the IRA category. Existing spread and cent rounding
// are unchanged. Tests: inherited_*_disaster_uses_* and combination controls.
// Coverage: Form 8915-F — Supported (unit T3-8915, 2026-09-27; audited Missing 2026-09-24).
//   Calculated, per owner and per disaster year (item B), one 2026 Form 8915-F each:
//   Part I from the 1099-R fold "This was a qualified disaster distribution" (FEMA DR number,
//   incident beginning and declaration dates, distribution date, designated amount, earlier
//   qualified distributions for the same disaster = line 1b, spread election, repayments by the
//   filing date). Line 1e is $22,000 per disaster less line 1b; each designated amount is
//   allocated to its own disaster, which is the instructions' "reasonable method" for lines 2-5b
//   column (b) and makes Worksheet 1B unnecessary. Line 6 (no Form 5329 additional tax: the
//   designated part's taxable amount leaves Form 5329 line 1 in retirement::early_tax), line 7
//   (the undesignated rest keeps its 1099-R treatment). Part II lines 8-15: line 9 cost is the
//   designated share of the entry's non-taxable amount (pro rata to the payer or Simplified Method
//   taxable amount); line 11 one third or all (election); line 12 Worksheet 2 = line 11 of the
//   2024 and 2025 Forms 8915-F for the same disaster year; line 14 Worksheet 3 = line 3c (2025
//   line 14 less line 13, less the part carried back) + line 4 (2026 repayments of 2024/2025
//   distributions and of this year's, taxable share only); line 15 to Form 1040 line 5b.
//   An owner who died in 2026 includes the rest of every spread on the final return (lines 13 and 24:
//   the 2026 distributions' whole taxable amount and the 2027 installment of a 2025 form's spread).
//   Part III lines 16-26 the same for IRAs (lines 18/20 are the designated share of the entry's
//   Form 8606 or payer taxable amount; line 26 to line 4b). Excess repayment (line 14 above line
//   13) is shown for next year or an amended earlier return; the 2027 and 2028 installments are
//   shown.
//   Unit B9-4 (2026-10-10, map retirement-special-codes R11): Part IV lines 28-32 (a hardship
//   distribution or IRA first-home distribution to buy or build a main home, DisasterDistribution::
//   HomePurchase with its own DR number and dates: line 32 = line 28 - cost - the taxable part
//   repaid stays in income and keeps the row's 10% path); the distribution period (Part I table)
//   and the Part IV window are rules on the 1099-R page and its server twin (DISASTER_OUTSIDE_PERIOD,
//   DISASTER_HOME_OUTSIDE_PERIOD, ...), not refusals; the designated part is a share of the entry's
//   taxable amount before any rollover after a gift to charity, so the undesignated rest keeps its
//   rollover, conversion, exception or QCD (DISASTER_AND_ROLLOVER_ABOVE_GROSS), and line 9 cost is
//   the designated share of the nontaxable amount whatever method produced it.
//   Refused by name: a disaster without a FEMA major-disaster (DR) number (item C); designated
//   amounts above line 1e; different spread elections on one form (lines 11/22);
//   a 2023-or-earlier
//   Form 8915-F (its spread has ended; 2026 repayments of those distributions are carried back by
//   amended returns). Ordinary and inherited Roth designated earnings feed line19 from
//   their existing Form8606 owners; qualified-only payments retain their zero taxable share. Forms
//   8915-D and 8915-E (2019/2020 disasters) have no 2026 income or repayment period and are not
//   routed to 2026 by the instructions.
//   Line audit F10 (2026-10-10): a repayment made during 2026 of a 2026 IRA distribution
//   (DisasterRepayment::Yes.in_return_year, of the repaid amount) is in the December 31
//   value and reduces Form 8606 line 6 (repaid_in_return_year here, IraBasis::adjusted_year_end); a 2025
//   Form 8915-F of any disaster year (item B) spreads over 2025-2027, so a filer who died in 2026
//   takes its 2027 remainder on the final return. Tests: tests/cross/line_audit_f10.rs.
//   Evidence: disaster.rs, retirement.rs early_tax, federal_rules.rs (line 4b/5b adjustment),
//   interview.rs (R1099 fold, Form8915 data answer); tests/retirement/disaster.rs. Sources: Form 8915-F
//   2026 draft (Rev. December 2026) and instructions (Rev. December 2025), irs.gov,
//   fetched2026-10-13; the draft adds5c after5a(total)/5b(nonqualified).

mod projection;

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::fraction;
use crate::line_emission::emit as emit_lines;
use crate::{FederalDraft, FederalIncome, FederalResult, Owner, RetirementKind, RetirementTaxable};
use alloc::collections::BTreeMap;

// Form 8606 line 6 (i8606, "Certain 2026 retirement plan distribution repayments treated as
// rollovers"): the part of this owner's 2026 IRA qualified disaster distributions that was
// repaid during 2026 is already in the December 31 value and comes off it.
pub(crate) fn repaid_in_return_year(d: &FederalDraft, owner: Owner) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data: source }
                if {
                    let data = &crate::FederalRetirement::from(source);
                    data.kind == RetirementKind::Ira
                } =>
            {
                let data = &crate::FederalRetirement::from(source);
                match data.disaster {
                    DisasterDistribution::Qualified { ref repaid, .. } => {
                        Some(repaid.in_return_year())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .sum()
}

// The incident beginning date of a designated entry, or the refusal that a disaster
// with no FEMA major-disaster (DR) number has no qualified disaster distributions
// (item C). The distribution period, the money rolled over or converted, an exception
// on the rest, a nonqualified annuity and a gift to charity out of the same 1099-R are
// page rules and ordinary treatments, not refusals: only the designated part spreads.
fn entry_refusal(v: &crate::FederalRetirement) -> Result<u32, (&'static str, &'static str)> {
    match v.disaster {
        DisasterDistribution::Qualified { began, .. } => Ok(began),
        _ => Err((
            "ISSUE_8915_MISSING_FEMA_DECLARATION",
            crate::fixture_english!(
                "Form 8915-F item C: only a disaster with a FEMA major disaster declaration (a DR number) can have qualified disaster distributions. Choose No, or enter the DR number."
            ),
        )),
    }
}

// The taxable part of a main home distribution that was repaid (Form 8915-F line 31), which
// leaves Form 5329 line 1 with the rest of line 32 kept there.
pub(crate) fn home_repaid(v: &crate::FederalRetirement, taxable: i64) -> i64 {
    match v.disaster {
        DisasterDistribution::HomePurchase { taxable_repaid, .. } => {
            phase::floor(phase::cap(taxable_repaid, [taxable]), 0)
        }
        _ => 0,
    }
}

// The entry's taxable amount before any rollover, less the gift to charity (Form 8915-F
// line 9's base): the amount the designated part is a share of.
fn taxable_before_rollover(
    e: &crate::FederalIncomeEntry,
    v: &crate::FederalRetirement,
    basis: &crate::retirement_return::BasisResult,
    qcd: i64,
) -> i64 {
    // A linked inherited payment's group supplies its share separately. With
    // no group, preserve the existing payer-amount fallback used by collect.
    let source_taxable = match v.taxable {
        Some(RetirementTaxable::Inherited { .. }) => v.box_2a,
        _ => basis.taxable(e.id, v),
    };
    let after = phase::floor(source_taxable - qcd, 0);
    if v.taxable == Some(RetirementTaxable::IraBasis) {
        // The Form 8606 ratio applies to every dollar that was not converted or given away.
        let funding = basis.hsa_funding.get(&e.id).copied().unwrap_or(0);
        let kept =
            phase::floor(v.gross - v.rolled_amount() - v.converted_amount() - qcd - funding, 0);
        if kept == 0 {
            return after;
        }
        return fraction(after, kept + v.rolled_amount(), kept);
    }
    // A rollover takes the taxable amount first (a Roth account's is taxed anyway).
    after + phase::floor(v.taxable_before_rollover() - v.taxable_income(), 0)
}

// The designated part's taxable amount of each 2026 qualified disaster distribution
// (lines 10 and 18/20 before the spread), by entry: the part of the distribution, after
// any gift to charity, that is designated, at the entry's taxable share. It leaves Form
// 5329 line 1 (retirement::early_tax); the rest keeps its 1099-R treatment.
pub(crate) fn designated(
    d: &FederalDraft,
    basis: &crate::retirement_return::BasisResult,
    qcd: &crate::retirement_return::QcdResult,
) -> BTreeMap<i64, i64> {
    let mut out = basis.inherited.designated.clone();
    for e in &d.income {
        let Some(v) = e.income.retirement() else {
            continue;
        };
        let DisasterDistribution::Qualified { amount, .. } = v.disaster else {
            continue;
        };
        if v.kind == RetirementKind::Inherited
            && matches!(
                crate::inherited_ira::link_of(d, e),
                Some(crate::inherited_ira::Link::Record(_))
            )
        {
            continue;
        }
        let gift = qcd.for_entry(e.id);
        let whole = v.gross - gift;
        if whole <= 0 || v.kind == RetirementKind::Nonqualified {
            continue;
        }
        let amount = phase::cap(amount, [whole]);
        let taxable = taxable_before_rollover(e, &v, basis, gift);
        out.insert(e.id, phase::cap(fraction(taxable, amount, whole), [amount]));
    }
    out
}

#[derive(Default)]
struct Form {
    projection: projection::Projection,
    // Part II and Part III: current designated amounts, their taxable
    // parts, carried income, repayments.
    plan_amount: i64,
    plan_taxable: i64,
    ira_amount: i64,
    ira_taxable: i64,
    plan_carried: i64,
    ira_carried: i64,
    plan_excess_in: i64,
    ira_excess_in: i64,
    plan_repaid: i64,
    ira_repaid: i64,
    // Line 11 and 22 amounts of 2025 forms: their third installment is 2027.
    plan_2025: i64,
    ira_2025: i64,
    spread: Option<DisasterSpread>,
    // Part I to III content: a form with only Part IV shows no spread lines.
    spread_parts: bool,
    // Part IV lines 28, 29 and 31 (the distributions for a main home, their cost, the
    // taxable part repaid).
    home_28: i64,
    home_29: i64,
    home_31: i64,
}

// Runtime attribution of the same final adjustment; columns are owner,
// rows are the unchanged IRA/pension categories. No second worksheet.
#[derive(Default)]
pub(crate) struct DisasterResult {
    pub(crate) categories: [i64; 2],
    pub(crate) owners: [[i64; 2]; 2],
}
// Form 8915-F for each owner and disaster year: the change to the IRA
// (index 0) and pension (index 1) taxable amounts, in cents, that replaces
// the designated parts' full taxable amounts with lines 26 and 15.
pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement_return::BasisResult,
    qcd: &crate::retirement_return::QcdResult,
) -> DisasterResult {
    let mut forms: BTreeMap<(usize, u32), Form> = BTreeMap::new();
    let mut limits: BTreeMap<(usize, u32), (i64, i64)> = BTreeMap::new();
    let mut adjustment = [0, 0];
    let mut owners = [[0; 2]; 2];
    // The Roth IRA rows with a designated disaster share (Form 8606 line 25b, Form 8915-F line
    // 19) are IRA rows for the form: their designated taxable amount is never in Form 1040
    // line 4b, so nothing is taken out of it here.
    let mut rows: alloc::vec::Vec<(
        &crate::FederalIncomeEntry,
        alloc::borrow::Cow<'_, crate::FederalRetirement>,
        bool,
    )> = alloc::vec::Vec::new();
    for e in &d.income {
        match &e.income {
            FederalIncome::Retirement { data } => {
                rows.push((
                    e,
                    alloc::borrow::Cow::Owned(crate::FederalRetirement::from(data)),
                    false,
                ));
            }
            FederalIncome::Roth { data: source }
                if matches!(
                    crate::RothWithdrawal::from(source).disaster,
                    DisasterDistribution::Qualified { .. }
                ) =>
            {
                let w = &crate::RothWithdrawal::from(source);
                rows.push((
                    e,
                    alloc::borrow::Cow::Owned(crate::FederalRetirement {
                        form_boxes: Default::default(),
                        trump: false,
                        box_2a: 0,
                        box_3: 0,
                        box_6: 0,
                        box_8a: 0,
                        box_8b_percent: None,
                        box_9a_percent: None,
                        ira: (RetirementKind::Ira).ira(),
                        disability: crate::DisabilityIncome::None,
                        kind: RetirementKind::Ira,
                        disability_wages: crate::DisabilityWages::None,
                        gross: w.gross - w.rolled_over,
                        taxable: Some(RetirementTaxable::Payer),
                        withholding: 0,
                        box7: 0,
                        ordinary_distribution: Some(crate::RetirementTreatment::Ordinary),
                        disaster: w.disaster.clone(),
                        after_death: None,
                        box_10: 0,
                        box_10_basis: None,
                        box_9b: None,
                    }),
                    true,
                ));
            }
            _ => {}
        }
    }
    for (e, v, roth) in &rows {
        let (e, roth) = (*e, *roth);
        let v: &crate::FederalRetirement = v;
        if v.disaster == DisasterDistribution::No {
            continue;
        }
        #[cfg(test)]
        let source = alloc::format!("Income: {}", e.label);
        let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
        // Part IV: a distribution for a main home. The taxable amount less the part repaid
        // stays in income (line 32) and keeps the row's 10% path (retirement::early_tax).
        if let DisasterDistribution::HomePurchase { began, taxable_repaid: repaid, .. } = v.disaster
        {
            let line_28 = phase::floor(v.gross - v.rolled_amount() - v.converted_amount(), 0);
            let form = forms.entry((owner, began / 10000)).or_default();
            form.home_28 += line_28;
            if basis.home_repaid.contains(&e.id) {
                // An IRA's cost or basis is figured on this year's Form 8606, which already
                // left the repaid amount out of line 7 (i8915-F lines 29 and 31): line 29 is
                // zero and the taxable amount is net of the repayment.
                form.projection.home_8606 = true;
                form.home_31 += phase::cap(repaid, [line_28]);
                continue;
            }
            let line_30 = phase::bound(basis.taxable(e.id, v), 0, line_28);
            form.home_29 += line_28 - line_30;
            let repaid = phase::cap(repaid, [line_30]);
            form.home_31 += repaid;
            form.projection.home(e, line_28, line_28 - line_30, repaid);
            adjustment[usize::from(v.kind != RetirementKind::Ira)] -= repaid;
            continue;
        }
        let began = match entry_refusal(v) {
            Ok(began) => began,
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    &source,
                    key,
                    message,
                );
                continue;
            }
        };
        let DisasterDistribution::Qualified { fema, amount, earlier, spread, ref repaid, .. } =
            v.disaster
        else {
            continue;
        };
        let amount = phase::floor(phase::cap(amount, [v.gross - qcd.for_entry(e.id)]), 0);
        let limit = limits.entry((owner, fema)).or_insert((0, earlier));
        limit.0 += amount;
        crate::require_localized!(
            r,
            limit.1 == earlier,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            &source,
            "ISSUE_FORM_8915_F_LINE_1B_ENTER_THE_SAME_EARLIER_QUALIFIED_DISASTER_DISTR_530EA91B",
            "Form 8915-F line 1b: enter the same earlier qualified disaster distributions on every 1099-R for this disaster.",
        );
        let taxable = if roth {
            basis.roth.designated.get(&e.id).copied().unwrap_or(0)
        } else {
            basis.disaster.get(&e.id).copied().unwrap_or(0)
        };
        let repaid_taxable =
            if amount == 0 { 0 } else { fraction(repaid.total(), taxable, amount) };
        let form = forms.entry((owner, began / 10000)).or_default();
        form.spread_parts = true;
        crate::require_localized!(
            r,
            form.spread.is_none_or(|s| s == spread),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            &source,
            "ISSUE_FORM_8915_F_LINES_11_AND_22_ONE_ELECTION_APPLIES_TO_EVERY_QUALIFIED_BC403371",
            "Form 8915-F lines 11 and 22: one election applies to every qualified disaster distribution of the same disaster year on this filer's form. Choose the same timing on each.",
        );
        form.spread = Some(spread);
        form.projection.record(d, r, e, amount, taxable);
        if v.kind.ira() {
            form.ira_amount += amount;
            form.ira_taxable += taxable;
            form.ira_repaid += repaid_taxable;
            if !roth {
                adjustment[0] -= taxable;
                owners[0][owner] -= taxable;
            }
        } else {
            form.plan_amount += amount;
            form.plan_taxable += taxable;
            form.plan_repaid += repaid_taxable;
            adjustment[1] -= taxable;
            owners[1][owner] -= taxable;
        }
    }
    for ((owner, fema), (amount, earlier)) in &limits {
        crate::federal_rules::require_message(
            r,
            amount + earlier <= LIMIT,
            &crate::IssueContext::owner(
                crate::TaxSection::Retirement,
                if *owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
            ),
            #[cfg(test)]
            "Form 8915-F",
            #[cfg(test)]
            alloc::format!(
                "Line 1e: qualified disaster distributions for disaster DR-{fema} of {} cannot exceed $22,000 less earlier years; treat less as qualified.",
                if *owner == 0 { "you" } else { "your spouse" }
            ),
            crate::Message::new(
                if *owner == 0 {
                    "ISSUE_DISASTER_LIMIT_TAXPAYER"
                } else {
                    "ISSUE_DISASTER_LIMIT_SPOUSE"
                },
                alloc::vec![crate::MessageArg::Text(alloc::format!("{fema}"))],
            ),
        );
    }
    let carries = d.screening.as_ref().map_or(&[][..], |s| &s.disaster_carries[..]);
    for (i, carry) in carries.iter().enumerate() {
        let f = &carry.form;
        let owner = usize::from(carry.owner == Owner::Spouse);
        let Some(year) = f.year() else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Form 8915-F",
                "ISSUE_A_FORM_8915_F_FROM_2023_OR_EARLIER_HAS_NO_2026_INCOME_A_2026_REPAYM_8F28E5B6",
                "A Form 8915-F from 2023 or earlier has no 2026 income; a 2026 repayment of those distributions is carried back to 2023-2025 on amended returns (Form 1040-X, Form 8915-F lines 14 and 25), which this App does not prepare.",
            );
            continue;
        };
        crate::require_localized!(
            r,
            f.valid()
                && !carries[..i].iter().any(|c| {
                    c.owner == carry.owner
                        && c.form.year() == Some(year)
                        && c.form.disaster_year == f.disaster_year
                }),
            &crate::IssueContext::owner(crate::TaxSection::Retirement, carry.owner)
                .item("disaster_carries", i),
            "Form 8915-F",
            "ISSUE_ENTER_EACH_EARLIER_FORM_8915_F_ONCE_WITH_A_DISASTER_YEAR_NO_LATER_T_087EE17B",
            "Enter each earlier Form 8915-F once, with a disaster year no later than its tax year and a carried-back amount no larger than its line 14 or 25 excess.",
        );
        let (plan_repaid, ira_repaid, plan_back, ira_back) = f.repaid();
        let (plan_excess, ira_excess) = f.excess_2025();
        let form = forms.entry((owner, f.disaster_year)).or_default();
        form.spread_parts = true;
        form.plan_carried += f.line_11;
        form.ira_carried += f.line_22;
        // Any 2025 Form 8915-F spreads over 2025-2027, whichever year the
        // disaster began (item B).
        if year == 2025 {
            form.plan_2025 += f.line_11;
            form.ira_2025 += f.line_22;
        }
        form.plan_excess_in += phase::floor(plan_excess - plan_back, 0);
        form.ira_excess_in += phase::floor(ira_excess - ira_back, 0);
        form.plan_repaid += plan_repaid;
        form.ira_repaid += ira_repaid;
    }
    let mut available = projection::available(d);
    for ((owner, year), form) in &forms {
        let person = d
            .people
            .as_ref()
            .and_then(|p| if *owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        let prefix = if *owner == 0 { "taxpayer" } else { "spouse" };
        // A filer who died in 2026 includes the rest of every spread on the
        // final return (Form 8915-F lines 13 and 24): the whole taxable amount
        // of the 2026 distributions on lines 11 and 22, and the third
        // installment of a 2025 spread that 2027 would have brought.
        let died = person.is_some_and(|p| p.died.is_some());
        let spread = form.spread != Some(DisasterSpread::AllInReturnYear) && !died;
        let third = |v: i64| if spread { fraction(v, 1, 3) } else { v };
        let (rest_plan, rest_ira) = if died { (form.plan_2025, form.ira_2025) } else { (0, 0) };
        let line_11 = third(form.plan_taxable);
        let line_13 = line_11 + form.plan_carried + rest_plan;
        let line_14 = form.plan_excess_in + form.plan_repaid;
        let line_15 = phase::floor(line_13 - line_14, 0);
        let line_22 = third(form.ira_taxable);
        let line_24 = line_22 + form.ira_carried + rest_ira;
        let line_25 = form.ira_excess_in + form.ira_repaid;
        let line_26 = phase::floor(line_24 - line_25, 0);
        adjustment[1] += line_15;
        owners[1][*owner] += line_15;
        adjustment[0] += line_26;
        owners[0][*owner] += line_26;
        let installment_following_year = if spread { line_11 + line_22 } else { 0 };
        let installment_second_following_year =
            form.plan_taxable + form.ira_taxable - line_11 - line_22 - installment_following_year;
        // Part IV: line 30 is the taxable part of the distributions for a main home and
        // line 32 what stays in income after the repayment.
        let (line_30, line_32) =
            (form.home_28 - form.home_29, form.home_28 - form.home_29 - form.home_31);
        let part_iv = if form.home_28 > 0 {
            alloc::vec![
                ("line_28", form.home_28, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_16")),
                ("line_29", form.home_29, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_17")),
                ("line_30", line_30, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_18")),
                ("line_31", form.home_31, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_19")),
                ("line_32", line_32, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_20")),
            ]
        } else {
            alloc::vec![]
        };
        emit_lines(r, &mut (), part_iv, |(suffix, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("form_8915_{prefix}_{year}_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.retirement.disaster"],
                "https://www.irs.gov/instructions/i8915f",
                note,
            ))
        });
        if !form.spread_parts {
            continue;
        }
        emit_lines(
            r,
            &mut (),
            [
                ("line_6", form.plan_amount + form.ira_amount, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N")),
                ("line_8", form.plan_amount, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_2")),
                (
                    "line_9",
                    form.plan_amount - form.plan_taxable,
                    msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_3"),
                ),
                ("line_10", form.plan_taxable, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_4")),
                ("line_11", line_11, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_5")),
                ("line_12", form.plan_carried, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_6")),
                ("line_13", line_13, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_7")),
                ("line_14", line_14, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_8")),
                ("line_15", line_15, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_9")),
                ("line_20", form.ira_taxable, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_10")),
                ("line_22", line_22, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_11")),
                ("line_23", form.ira_carried, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_12")),
                ("line_24", line_24, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_13")),
                ("line_25", line_25, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_14")),
                ("line_26", line_26, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_15")),
                (
                    "excess_repayment",
                    phase::floor(line_14 - line_13, 0) + phase::floor(line_25 - line_24, 0),
                    msg!("EXPLAIN_FORM_N_N_N_N_EXCESS_REPAYMENT"),
                ),
                (
                    "installment_following_year",
                    installment_following_year,
                    msg!("EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N"),
                ),
                (
                    "installment_second_following_year",
                    installment_second_following_year,
                    msg!("EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N_2"),
                ),
            ],
            |(suffix, amount, note), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("form_8915_{prefix}_{year}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.disaster", "screening.disaster_carries"],
                    "https://www.irs.gov/instructions/i8915f",
                    note,
                ))
            },
        );
    }
    for ((owner, year), form) in &forms {
        let later = forms.iter().any(|((next_owner, next_year), next)| {
            next_owner == owner && next_year > year && !next.projection.disasters.is_empty()
        });
        projection::emit(r, *owner, *year, form, &mut available[*owner], later);
    }
    DisasterResult { categories: adjustment, owners }
}
