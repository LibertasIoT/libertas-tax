// Coverage: Form 6781 Part I — Supported (unit B2, 2026-10-09); Part II, Part III and boxes A to C
//   are refused or memo-only. Sources: the 2026 draft Form 6781 with its instructions on pages 2-4
//   (created 2026-09-14; there is no separate i6781), 26 U.S.C. 1256, 1212(c), 1092 and 1091,
//   Instructions for Form 1099-B (2026) boxes 8-11, Publication 550 (2025; no 2026 text exists),
//   the 2026 draft Schedule D lines 4 and 11 (created 2026-04-01). Both forms are drafts; step 4
//   of the parity plan rechecks them when the finals are posted.
//   Lines: box D (derived: checked exactly when a carryback is entered); line 1 one row for each
//   Investment sales "1099-B futures and section 1256 options" statement
//   (FederalIncome::Section1256, the broker's box 11 aggregate profit or loss, whole dollars, the
//   broker as column (a)); 2b losses and 2c gains; 3 and 5 the net (line 4, the 1099-B
//   adjustments, is zero: its three causes are refused through Straddles or left out through the
//   statement's special answer); 6 the entered net section 1256 contracts loss carried back
//   (Section1256Carryback, section 1212(c)); 7 = 5 + 6; 8 = 40% of line 7 to Schedule D line 4;
//   9 = line 7 less line 8 (60%) to Schedule D line 11. Rounding: 40% of a whole-dollar amount has
//   a fraction of .0, .2, .4, .6 or .8 and never .5, so line 9 as the remainder equals 60% rounded.
//   Lines 8 and 9 join the Form 6252 earlier-year amounts in Totals.schedule_d_line_4/11 (the one
//   Schedule D path), the short and long totals, net investment income and the Form 1116 line 3e
//   capital pass; AMT uses independently limited K-1 rows through the same owner. The carryback election is the filer's: the 2026
//   effect is inside (the amount leaves 2026 as 40/60 gain, reducing the capital loss and the 2027
//   carryover, section 1212(c)(6)(A)); the refund for 2023-2025 is claimed on Form 1045 or 1040-X
//   and is not in the estimate (NOTE_SECTION_1256_CARRYBACK). Two blocking checks, exact for
//   every amount the App can decide: a carryback needs a net loss on line 5
//   (ISSUE_SECTION_1256_CARRYBACK_WITHOUT_LOSS), and it cannot exceed
//   section_1256_net_loss = min(the 2027 carryover figured without the carryback, the line 5 loss
//   plus the section 1256 part of the 2025 carryover less the $3,000 ($1,500 separate) limit;
//   draft Form 6781 box D, Pub. 550 "Net section 1256 contracts loss": a carryover from section
//   1256 losses is treated as a 2026 section 1256 loss, section 1212(c)(6)(B)), exact because
//   PriorCapitalReturn.section_1256 carries that part
//   (ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS). Section 1256 character of the 2027 carryover
//   is not reported. Wash sales do not apply to a year-end mark
//   (section 1256(f)(5)).
//   Excluded: straddles (Part II lines 10-13b, boxes A to C, section 1092) are refused by name
//   through RareSituation::Straddles; a hedge, a section 475(f) or 988 election and a dealer's
//   contracts are left out with WARN_SECTION_1256_LEFT_OUT; Part III is a memo disclosure with no
//   calculation effect; the amended-return carryback row of line 1 is not an original-return
//   item; differing K-1 AMT allowances with a carryback election remain unavailable without
//   independent AMT carryback records; a QOF deferral stays refused (Form 8997). Printed
//   broker documents retain boxes 8–10 for reconciliation; the actual box 11
//   remains the authoritative aggregate for this existing calculation. Tests (hand-computed): src/tests/investment/investment.rs
//   form_6781_net_gain_is_taxed_60_40, form_6781_net_loss_carries_over_without_carryback,
//   form_6781_carryback_moves_gain_into_the_loss_year_and_checks_the_bound,
//   form_6781_open_contracts_and_rounding, form_6781_left_out_statement,
//   form_6781_straddles_are_refused_by_name, form_6781_pages_and_twice_rules; src/tests/investment/interview.rs
//   contracts_1099b_*, section_1256_carryback_*.
//   https://www.irs.gov/pub/irs-dft/f6781--dft.pdf
//   https://www.irs.gov/pub/irs-pdf/i1099b.pdf
//   https://www.irs.gov/publications/p550
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::form_line;
use crate::{FederalDraft, FederalIncome, FederalResult, FormLine, TaxForm};
use alloc::format;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 2026 section 1256 loss: carryback choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Section1256Carryback {
    /// Net section 1256 contracts loss you carry back (Form 6781 line 6)
    /// Figure it from your 2023–2025 returns with Publication 550: earliest year first, no more than each year's net gain from section 1256 contracts, and never creating a net operating loss there. The estimate removes it from 2026; the refund for those years is claimed on Form 1045 or Form 1040-X and is not part of this estimate.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Actual owners of the loss absorbed in the carryback years
    /// Only for a Form 8379 claim. Use the actual absorbed amounts from the retained carryback worksheets, not the surviving capital-loss shares or the owner of this election page. This is not a current-year refund or payment.
    /// [NullText]
    /// Owners of the absorbed carryback have not been established
    /// [SetText]
    /// The loss absorbed in the carryback years belongs to
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_retained_owners")]
    pub injured_owners: Option<crate::InjuredRetainedOwners>,
}

// Form 6781 Part I in whole dollars; each line 1 row is rounded alone and
// every later line is figured from the rounded rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Form6781 {
    pub losses: i64,
    pub gains: i64,
    pub net: i64,
    pub carryback: i64,
    pub total: i64,
    pub short: i64,
    pub long: i64,
    // Actual accepted line1 rows in dollars, before the carryback; runtime
    // attribution only, never another Form6781 calculation or stored answer.
    pub source_net: [i64; 2],
}

impl Form6781 {
    fn with_carryback(net: i64, carryback: i64, losses: i64, gains: i64) -> Self {
        let total = net + carryback;
        let short = fraction(total, 40, 100);
        Self {
            losses,
            gains,
            net,
            carryback,
            total,
            short,
            long: total - short,
            source_net: [0; 2],
        }
    }
}

// The supported working copy's statements and the entered carryback.
pub(crate) fn form_6781(d: &FederalDraft) -> Form6781 {
    form_6781_rows(d, false, false)
}

// The same without the K-1s that have a Schedule K-3 (Form 1116 line 3e).
pub(crate) fn form_6781_without_k3(d: &FederalDraft) -> Form6781 {
    form_6781_rows(d, true, false)
}

// Independent AMT K-1 allowances through the same Part I calculation.
pub(crate) fn form_6781_amt(d: &FederalDraft) -> Form6781 {
    form_6781_rows(d, false, true)
}

pub(crate) fn form_6781_amt_without_k3(d: &FederalDraft) -> Form6781 {
    form_6781_rows(d, true, true)
}

fn form_6781_rows(d: &FederalDraft, without_k3: bool, alternative: bool) -> Form6781 {
    checked_form_6781_rows(d, without_k3, alternative)
        .expect("section 1256 financial source admission")
}

// Accumulate the actual rounded rows widely before narrowing the existing
// Part I owner. Both normal projection and financial admission use this pass.
fn checked_form_6781_rows(
    d: &FederalDraft,
    without_k3: bool,
    alternative: bool,
) -> Option<Form6781> {
    let (mut losses, mut gains) = (0_i128, 0_i128);
    let mut source_net = [0_i128; 2];
    for e in &crate::community::entries(d) {
        if without_k3
            && match &e.income {
                FederalIncome::Partnership { data } => data.schedule_k3.is_some(),
                FederalIncome::Scorp { data } => {
                    let data = &crate::ScorpK1::from(data);
                    data.schedule_k3.is_some()
                }
                _ => false,
            }
        {
            continue;
        }
        // A K-1's net section 1256 amount (box 11 code C, box 10
        // code C) after its basis and at-risk limits is one more line 1 row,
        // named for the entity.
        let k1 = match &e.income {
            FederalIncome::Section1256 { .. } | FederalIncome::Broker { .. } => {
                crate::broker_calculation::contracts_amount(d, e)
            }
            FederalIncome::Partnership { data } => {
                let facts = crate::community::partner_facts(d, e, data).ok()?;
                Some(if alternative {
                    facts.amt_flows.section1256
                } else {
                    facts.flows.section1256
                })
            }
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                let facts = crate::community::scorp_facts(d, e, data);
                Some(if alternative {
                    facts.amt_flows.section1256
                } else {
                    facts.flows.section1256
                })
            }
            _ => None,
        };
        if let Some(amount) = k1 {
            let row = dollars(amount);
            let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
            source_net[owner] = source_net[owner].checked_add(i128::from(row))?;
            if row < 0 {
                losses = losses.checked_add(i128::from(row))?;
            } else {
                gains = gains.checked_add(i128::from(row))?;
            }
        }
    }
    let carryback = d.screening.as_ref().and_then(|s| s.section_1256_carryback).map_or(0, dollars);
    let losses = i64::try_from(losses).ok()?;
    let gains = i64::try_from(gains).ok()?;
    let net = losses.checked_add(gains)?;
    net.checked_add(carryback)?;
    let mut form = Form6781::with_carryback(net, carryback, losses, gains);
    form.source_net = [i64::try_from(source_net[0]).ok()?, i64::try_from(source_net[1]).ok()?];
    // Native Schedule D, AMT, NIIT and attribution consumers carry cents.
    for amount in [
        form.losses,
        form.gains,
        form.net,
        form.carryback,
        form.total,
        form.short,
        form.long,
        form.source_net[0],
        form.source_net[1],
    ] {
        amount.checked_mul(100)?;
    }
    Some(form)
}

pub(crate) fn financial_issue(draft: &FederalDraft) -> Option<(i64, &'static str)> {
    let id = draft.income.iter().find_map(|entry| {
        matches!(entry.income, FederalIncome::Broker { .. }).then_some(entry.id)
    })?;
    for (without_k3, alternative) in [(false, false), (true, false), (false, true), (true, true)] {
        if checked_form_6781_rows(draft, without_k3, alternative).is_none() {
            return Some((id, "BROKER_CONTRACTS_ARITHMETIC_UNAVAILABLE"));
        }
    }
    None
}

// Whether any Part I statement or carryback is on the return.
pub(crate) fn present(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| match &e.income {
        FederalIncome::Section1256 { .. } => true,
        FederalIncome::Broker { data } => {
            crate::broker_calculation::projection(data).contracts.is_some()
        }
        FederalIncome::Partnership { data } => {
            data.k1_items().has(crate::k1_items::K1ItemKind::Section1256)
        }
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            data.k1_items().has(crate::k1_items::K1ItemKind::Section1256)
        }
        _ => false,
    }) || d.screening.as_ref().is_some_and(|s| s.section_1256_carryback.is_some())
}

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f6781--dft.pdf";

fn row(r: &mut FederalResult, line: &str, id: &str, amount: i64, explanation: crate::Message) {
    form_line(
        r,
        FormLine::new(TaxForm::Form6781, line),
        id,
        amount,
        &["income.section_1256", "screening.section_1256_carryback"],
        SOURCE,
        explanation,
    );
}

// The inputs to the carryback bound, from the Schedule D netting after the
// 2025 carryover and Form 6781: the short- and long-term totals, the loss
// deducted, the unfloored taxable income, the 2025 carryover and the entered
// carryback.
pub(crate) struct Netting {
    pub short: i64,
    pub long: i64,
    pub deducted: i64,
    pub taxable: i64,
    pub carry_in: i64,
    // The part of the carry-in that came from section 1256 contracts, which
    // counts as a 2026 section 1256 loss (Pub. 550).
    pub carry_1256: i64,
    pub separate: bool,
}

// Form 6781's rows and the carryback checks.
pub(crate) fn report(d: &FederalDraft, netting: &Netting, r: &mut FederalResult) {
    if !present(d) {
        return;
    }
    let form = form_6781(d);
    for e in &d.income {
        if let Some(amount) = crate::broker_calculation::contracts_amount(d, e) {
            crate::line_emission::line(
                r,
                &format!("section_1256_{}_aggregate", e.id),
                dollars(amount),
                &["income.section_1256"],
                SOURCE,
                msg!("EXPLAIN_SECTION_1256_N_AGGREGATE"),
            );
        }
    }
    row(r, "2", "form_6781_losses", form.losses, msg!("EXPLAIN_FORM_6781_LOSSES"));
    row(r, "2", "form_6781_gains", form.gains, msg!("EXPLAIN_FORM_6781_GAINS"));
    row(r, "5", "form_6781_net", form.net, msg!("EXPLAIN_FORM_6781_NET"));
    let elected = d.screening.as_ref().and_then(|s| s.section_1256_carryback).is_some();
    if elected {
        row(r, "6", "form_6781_carryback", form.carryback, msg!("EXPLAIN_FORM_6781_CARRYBACK"));
    }
    row(r, "7", "form_6781_total", form.total, msg!("EXPLAIN_FORM_6781_TOTAL"));
    row(r, "8", "form_6781_short_term", form.short, msg!("EXPLAIN_FORM_6781_SHORT_TERM"));
    row(r, "9", "form_6781_long_term", form.long, msg!("EXPLAIN_FORM_6781_LONG_TERM"));
    let bound = net_loss_bound(&form, netting);
    if form.net < 0 {
        row(r, "D", "section_1256_net_loss", bound, msg!("EXPLAIN_SECTION_1256_NET_LOSS"));
    }
    if !elected {
        return;
    }
    let context = crate::IssueContext::Section(crate::TaxSection::InvestmentSales);
    let money = |v: i64| crate::MessageArg::Currency {
        currency_code: "USD".into(),
        scale: 0,
        minor_units: v,
    };
    let problem = if form.net >= 0 {
        Some(crate::Message::new("ISSUE_SECTION_1256_CARRYBACK_WITHOUT_LOSS", alloc::vec![]))
    } else if form.carryback > bound {
        Some(crate::Message::new(
            "ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS",
            alloc::vec![money(form.carryback), money(bound)],
        ))
    } else {
        None
    };
    if let Some(message) = problem {
        crate::federal_rules::issue_message(
            r,
            &context,
            #[cfg(test)]
            "Section 1256 carryback",
            #[cfg(test)]
            message.english(),
            message,
        );
    }
    let message =
        crate::Message::new("NOTE_SECTION_1256_CARRYBACK", alloc::vec![money(form.carryback)]);
    r.review_notes.push(crate::federal_model::ReviewNote::scoped(
        #[cfg(test)]
        message.english(),
        message,
        context,
    ));
}

// The most a carryback election can carry back (box D, second bullet): the
// 2027 carryover figured with line 6 zero, limited by the line 5 loss plus
// the 2025 carryover less the annual limit. The netting includes the entered
// carryback's 40/60 gain; it is taken out, and taxable income plus the loss
// deducted stays fixed, which is exact for every amount that passes the
// check because such an amount leaves at least the limit of net loss.
fn net_loss_bound(form: &Form6781, netting: &Netting) -> i64 {
    let limit = if netting.separate {
        crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS
    } else {
        crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS
    };
    let without = Form6781::with_carryback(form.net, 0, form.losses, form.gains);
    let short = netting.short - (form.short - without.short);
    let long = netting.long - (form.long - without.long);
    let deducted = phase::cap(limit, [positive(-(short + long))]);
    let carry = crate::extended_rules::capital_carryover(
        short,
        long,
        deducted,
        netting.taxable + netting.deducted - deducted,
    );
    phase::cap(
        carry[0] + carry[1],
        [positive(-form.net + phase::cap(netting.carry_1256, [netting.carry_in]) - limit)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forty_percent_never_lands_on_a_half_and_the_parts_sum() {
        // 40% of a whole-dollar amount has a fraction of .0, .2, .4, .6 or
        // .8, so the rounding is unambiguous and line 9 (the remainder)
        // equals 60% rounded.
        for total in -1_000..=1_000 {
            let form = Form6781::with_carryback(total, 0, 0, 0);
            assert_eq!(form.short + form.long, total);
            assert_eq!(form.long, fraction(total, 60, 100), "{total}");
            assert_eq!((total * 40).rem_euclid(100) % 10, 0);
            assert_ne!((total * 40).rem_euclid(100), 50);
        }
        let form = Form6781::with_carryback(-1_001, 0, 0, 0);
        assert_eq!((form.short, form.long), (-400, -601));
        let form = Form6781::with_carryback(1_501, 0, 0, 0);
        assert_eq!((form.short, form.long), (600, 901));
    }

    #[test]
    fn carryback_bound_takes_out_the_carryback_and_the_limit() {
        // Net loss 17,500 with a 2025 carryover of 0: short -7,000 and long
        // -6,500 before the carryback (see src/tests/investment/investment.rs F2).
        let without = Form6781::with_carryback(-17_500, 0, -20_000, 2_500);
        let netting = Netting {
            short: -7_000,
            long: -6_500,
            deducted: 3_000,
            taxable: 114_800,
            carry_in: 0,
            carry_1256: 0,
            separate: false,
        };
        assert_eq!(net_loss_bound(&without, &netting), 10_500);
        // With 8,000 carried back the netting holds 3,800 / 1,700 and the
        // bound is the same.
        let with = Form6781::with_carryback(-17_500, 8_000, -20_000, 2_500);
        let netting = Netting { short: -3_800, long: -1_700, ..netting };
        assert_eq!(net_loss_bound(&with, &netting), 10_500);
        // A 2025 carryover of 4,000 (the netting holds it: long -10,500)
        // raises the second term to 17,500 + 4,000 - 3,000 = 18,500, so the
        // 2027 carryover of 14,500 binds.
        let netting = Netting {
            short: -7_000,
            long: -10_500,
            deducted: 3_000,
            taxable: 114_800,
            carry_in: 4_000,
            carry_1256: 4_000,
            separate: false,
        };
        assert_eq!(net_loss_bound(&without, &netting), 14_500);
    }

    #[test]
    fn carryback_bound_counts_only_the_section_1256_part_of_the_carryover() {
        // A 2026 section 1256 loss of 5,000 (no gains) and a 20,000 carryover
        // from 2025: short -2,000 and long -3,000 - 20,000 = -23,000 in the
        // netting; 3,000 deducted; the 2027 carryover is 22,000. Box D's first
        // bullet: the section 1256 losses (5,000, plus the part of the
        // carryover with section 1256 character) less the 3,000 limit.
        let form = Form6781::with_carryback(-5_000, 0, -5_000, 0);
        let netting = |carry_1256| Netting {
            short: -2_000,
            long: -23_000,
            deducted: 3_000,
            taxable: 100_000,
            carry_in: 20_000,
            carry_1256,
            separate: false,
        };
        // None of the carryover is section 1256: 5,000 - 3,000 = 2,000.
        assert_eq!(net_loss_bound(&form, &netting(0)), 2_000);
        // 10,000 of it is: 5,000 + 10,000 - 3,000 = 12,000.
        assert_eq!(net_loss_bound(&form, &netting(10_000)), 12_000);
        // All of it: 25,000 - 3,000 = 22,000, the 2027 carryover binds.
        assert_eq!(net_loss_bound(&form, &netting(20_000)), 22_000);
    }
}
