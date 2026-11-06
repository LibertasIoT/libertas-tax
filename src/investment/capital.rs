// Coverage: ScheduleD PRINT computed totals (2026-10-13): actual admitted
//   K-1 portfolio5/12, distributions13, prior carries6/14, regular net7/15/16,
//   conditional worksheet18/19 and loss21 project the same source owners.
//   Tax-rate-capped pools remain distinct; NOL internal refigures do not print.
//   Source: IRS2026 draft f1040sd; src/tests/investment/investment.rs and corpus footings.
// Coverage: Schedule D — Supported (audited 2026-09-24).
//   Lines: Short/long netting, 2025 line 7/15 carryovers with character preserved, $3,000/$1,500
//   loss limit, 28% collectibles and §1202 pool, unrecaptured 1250, Schedule D Tax Worksheet/QDCG
//   rates, Form 8615 gains, separate AMT and NIIT tracks, §1231/4797 feed, K-1 and 2439 components.
//   Exclusions: sales flagged with a SaleSpecial reason (stock compensation, business property, home,
//   another treatment) are left out with a notice unless entered through their own worksheets.
//   Refused explicitly (unit A13, from the Schedule D map): the Form 8824 amounts of lines 4 and
//   11 and the Form 6781 Part II straddle amounts through the LikeKindExchange and Straddles
//   situations (M1-P3; formerly the screening answer other_capital_gain_forms).
//   Unit B9-5 (2026-10-10, investment-sale-specials map): every lot that needs Form 8949 is a
//   Form8949Row (columns (a), (d)-(h), box A-L, codes) and Schedule D lines 1a-3 and 8a-10 total
//   columns (d), (e) and (g) as well as (h); the codes D (market discount, interest), M (several
//   sales), N (nominee, column (g) backs out the gain), E (option premium), Y and Z (qualified
//   opportunity funds, opportunity_fund.rs) and O (the section 1231 pair) are calculated, a
//   written option is always short-term (section 1234(b)) and an empty basis counts as zero.
//   Form 6252 (2026-09-27): lines 4 and 11 carry Form 6252 line
//   26 of earlier-year installment sales of investment property (installment::earlier_capital)
//   and are emitted as capital_gain_4 and capital_gain_11 (federal_rules.rs
//   Totals.schedule_d_line_4 and schedule_d_line_11). Current-year investment/personal installments
//   join those same cells through the current-sale owner; full recognition uses Form 8949.
//   The same totals take Form
//   6781 line 8 (40% short-term) and line 9 (60% long-term) of section 1256 contracts
//   (section1256.rs), with the net section 1256 contracts loss carryback election reducing the
//   2026 loss and the carryover (tests: form_6781_* in src/tests/investment/investment.rs). Unit F4 (2026-10-10,
//   line audit D8, D9): line 11 also carries the Form 4797 Part I gain (section1231.rs
//   long_term, added at the emission in federal_rules.rs) and lines 4 and 11 the Form 4684 line
//   15 casualty gains, so the lines foot to lines 7 and 15 (src/tests/investment/investment.rs
//   form_4684_gain_is_schedule_d_line_11, form_4797_part_i_gain_is_schedule_d_line_11,
//   src/tests/investment/investment_kinds.rs schedule_d_lines_foot_with_the_investment_kinds); basis-reported
//   collectibles (code C) and personal-use losses (code L) are lines 1b/8b, not 1a/8a (D7).
//   Form 2439 box 1a is also on line 11. Unit B2/8824
//   (2026-10-09): lines 4 and 11 also carry Form 8824 line 24 of an earlier-year related-party
//   exchange whose property was disposed of in 2026 within two years (exchange::related_capital,
//   Investment sales kind FederalScreening.related_exchanges); the LikeKindExchange situation
//   keeps only exchanges of business, farm or investment real property that is not a rental and
//   section 1043 sales.
//   Evidence: capital.rs, federal_rules.rs (preferential_tax*, schedule_d_tax, collect_sale),
//   extended_rules.rs (prior_capital, capital_carryover), section1231.rs, qsbs.rs; tests:
//   capital_loss_limit_and_carryforward,
//   capital_carryover_preserves_character_and_unused_below_zero_deduction,
//   section_1250_and_collectible_distributions_share_loss_netting_in_order (investment.rs),
//   preferential_rate_stacking_and_cross_netting (general.rs); i1040sd
//   C-fixture (2026-09-25, src/tests/investment/investment.rs): capital_gains_return_matches_hand_computation (lines 1b-22
//   and the QDCG worksheet by hand), net_capital_loss_deducts_3000_and_carries_forward,
//   collectibles_gain_uses_the_28_percent_worksheet, unrecaptured_1250_gain_is_capped_at_25_percent
//   (both through the Schedule D Tax Worksheet), prior_year_carryover_nets_on_lines_6_and_14.

// Shared Schedule D netting stage, normally in return dollars; Iterative
// health/PTC work reuses it in cents. Inputs include resolved
// capital-character property items only; ordinary section 1231 amounts belong
// outside Schedule D. Run with independent regular and AMT amounts/history.
// https://www.irs.gov/instructions/i1040sd
use crate::kernel::phase;

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::line_emission::emit as emit_lines;
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CapitalAmounts {
    pub short: i64,
    pub long: i64,
    // The Schedule D 28%-rate pool includes collectibles and unexcluded
    // eligible section 1202 gain from the 50%/75% acquisition cohorts.
    pub collectibles: i64,
    pub unrecaptured: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CapitalResult {
    pub short: i64,
    pub long: i64,
    pub income: i64,
    pub net_gain: i64,
    pub line18: i64,
    pub line19: i64,
    pub collectible_gain: i64,
    pub unrecaptured_gain: i64,
}

// Rate-character pools from already-net short/long amounts, in the caller's
// unchanged money unit. This stage neither applies nor recomputes an annual
// capital-loss limit.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CapitalRatePools {
    pub net_gain: i64,
    pub line18: i64,
    pub line19: i64,
    pub collectible_gain: i64,
    pub unrecaptured_gain: i64,
}

// Preserve existing whole-dollar cells. Their exact source cents are a
// typed supporting subrow when rounding would otherwise discard precision.
// Netting consumes the cents once; it never sums independently rounded cells.
pub(crate) fn schedule_d_source_cell(
    r: &mut crate::FederalResult,
    form: crate::FormLine,
    id: &str,
    cents: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    crate::line_emission::form_line(
        r,
        form.clone(),
        id,
        crate::kernel::ratio::dollars(cents),
        inputs,
        source,
        explanation.clone(),
    );
    if cents % 100 != 0 {
        crate::line_emission::form_value(
            r,
            crate::FormLine::subrow(form.form, &form.line),
            &alloc::format!("{id}_unrounded"),
            LineValue::MoneyCents(cents),
            inputs,
            source,
            explanation,
        );
    }
}

// PRINT consumes the settled regular netting owner. K-1 and fund source rows
// print whole dollars with admitted cents in supporting subrows; carryovers
// and net totals keep the owner's dollars.
// Shareholder loan repayments are separate disposition sources, never K-1
// portfolio gains. Neither the tax-rate-limited pools nor Form172's internal
// modified income may replace the actual ScheduleD worksheet amounts.
impl CapitalResult {
    pub(crate) fn emit_summary(
        &self,
        prior: [i64; 2],
        k1: [i64; 2],
        distributions: i64,
        r: &mut crate::FederalResult,
    ) {
        let gains = self.long > 0 && self.short + self.long > 0;
        for (number, value, applies) in [
            ("5", LineValue::MoneyCents(k1[0]), true),
            ("6", LineValue::MoneyDollars(-prior[0]), true),
            ("7", LineValue::MoneyDollars(self.short), true),
            ("12", LineValue::MoneyCents(k1[1]), true),
            ("13", LineValue::MoneyCents(distributions), true),
            ("14", LineValue::MoneyDollars(-prior[1]), true),
            ("15", LineValue::MoneyDollars(self.long), true),
            ("16", LineValue::MoneyDollars(self.short + self.long), true),
            ("18", LineValue::MoneyDollars(self.line18), gains),
            ("19", LineValue::MoneyDollars(self.line19), gains),
            ("21", LineValue::MoneyDollars(self.income), self.short + self.long < 0),
        ] {
            if !applies {
                continue;
            }
            let form = crate::FormLine::new(crate::TaxForm::ScheduleD, number);
            let id = alloc::format!("schedule_d_line_{number}");
            let inputs = &["income", "screening.capital_history"];
            let source = "https://www.irs.gov/pub/irs-dft/f1040sd--dft.pdf";
            let explanation = msg!("EXPLAIN_SCHEDULE_D_COMPUTED_TOTAL");
            if let LineValue::MoneyCents(cents) = value {
                schedule_d_source_cell(r, form, &id, cents, inputs, source, explanation);
            } else {
                crate::line_emission::form_value(r, form, &id, value, inputs, source, explanation);
            }
        }
    }
}

// Checkbox answers consume the actual projected summary and investment form.
// Zero source worksheets alone do not introduce a visible ScheduleD copy.
pub(crate) fn emit_summary_choices(r: &mut crate::FederalResult, qualified: i64) {
    let amount = |number: &str| {
        r.lines
            .iter()
            .find(|line| line.id == alloc::format!("schedule_d_line_{number}"))
            .map(|line| line.amount)
    };
    let Some(net) = amount("16") else {
        return;
    };
    let long = amount("15").unwrap();
    let special = [amount("18").unwrap_or(0), amount("19").unwrap_or(0)];
    let visible = r.lines.iter().any(|line| {
        line.amount != 0
            && line.form_line.as_ref().is_some_and(|form| form.form == crate::TaxForm::ScheduleD)
    });
    if !visible {
        return;
    }
    let investment = r.lines.iter().any(|line| {
        line.amount != 0
            && line.form_line.as_ref().is_some_and(|form| form.form == crate::TaxForm::Form4952)
    });
    let gains = net > 0 && long > 0;
    emit_lines(
        r,
        &mut (),
        [
            ("17", gains, net > 0),
            ("20", special == [0, 0] && !investment, gains),
            ("22", qualified > 0, !gains),
        ],
        |(number, yes, applies), _, _| {
            if !applies {
                return None;
            }
            Some(calculation_row(
                Some(crate::FormLine::new(crate::TaxForm::ScheduleD, number)),
                &alloc::format!("schedule_d_choice_{number}_{}", if yes { "yes" } else { "no" }),
                LineValue::Text,
                &["income", "deductions.investment_interest"],
                "https://www.irs.gov/pub/irs-dft/f1040sd--dft.pdf",
                msg!("EXPLAIN_SCHEDULE_D_SUMMARY_ANSWER"),
            ))
        },
    );
}

impl CapitalAmounts {
    pub(crate) fn calculate(
        self,
        prior: [i64; 2],
        separate: bool,
        nol_modification: bool,
    ) -> CapitalResult {
        self.calculate_with_limit(prior, separate, nol_modification, 1)
    }

    // Pub974's Iterative work keeps cents before filing-row rounding. Reuse
    // this same admitted-character netting owner and only scale its statutory
    // loss limit; this does not reclassify a sale or reconstruct prior returns.
    #[cfg(test)]
    pub(crate) fn calculate_cents(
        self,
        prior: [i64; 2],
        separate: bool,
        nol_modification: bool,
    ) -> CapitalResult {
        self.calculate_with_limit(prior, separate, nol_modification, 100)
    }

    fn calculate_with_limit(
        self,
        prior: [i64; 2],
        separate: bool,
        nol_modification: bool,
        units: i64,
    ) -> CapitalResult {
        self.checked_calculate_with_limit(prior, separate, nol_modification, units)
            .expect("admitted native capital netting")
    }

    pub(crate) fn checked_calculate_cents(
        self,
        prior: [i64; 2],
        separate: bool,
        nol_modification: bool,
    ) -> Option<CapitalResult> {
        self.checked_calculate_with_limit(prior, separate, nol_modification, 100)
    }

    fn checked_calculate_with_limit(
        self,
        prior: [i64; 2],
        separate: bool,
        nol_modification: bool,
        units: i64,
    ) -> Option<CapitalResult> {
        let short = i64::try_from(i128::from(self.short) - i128::from(prior[0])).ok()?;
        let long = i64::try_from(i128::from(self.long) - i128::from(prior[1])).ok()?;
        let net = i64::try_from(i128::from(short) + i128::from(long)).ok()?;
        let income = phase::floor(
            net,
            if separate {
                -crate::year_constants::CAPITAL_LOSS_LIMIT_SEPARATE_DOLLARS * units
            } else {
                -crate::year_constants::CAPITAL_LOSS_LIMIT_DOLLARS * units
            },
        );
        let income = if nol_modification { phase::floor(income, 0) } else { income };
        let rates = Self { short, long, ..self }.checked_rate_pools(prior[1])?;
        Some(CapitalResult {
            short,
            long,
            income,
            net_gain: rates.net_gain,
            line18: rates.line18,
            line19: rates.line19,
            collectible_gain: rates.collectible_gain,
            unrecaptured_gain: rates.unrecaptured_gain,
        })
    }

    // short/long have already absorbed their respective retained carryovers;
    // collectibles/unrecaptured are the admitted pools before loss netting.
    // prior_long is the same actual retained long loss already subtracted
    // from long; it also reduces the 28% pool in the worksheet's own order.
    pub(crate) fn rate_pools(self, prior_long: i64) -> CapitalRatePools {
        self.checked_rate_pools(prior_long).expect("admitted native capital rate pools")
    }

    fn checked_rate_pools(self, prior_long: i64) -> Option<CapitalRatePools> {
        let short = i128::from(self.short);
        let long = i128::from(self.long);
        let net_gain = long.max(0).min((short + long).max(0));
        // Long-term carryovers and net short-term losses first offset the
        // collectible pool; remaining losses then reduce the section 1250 pool.
        let collectible_after_losses =
            i128::from(self.collectibles) - i128::from(prior_long) + short.min(0);
        let line18 = collectible_after_losses.max(0);
        let line19 = (i128::from(self.unrecaptured) - (-collectible_after_losses).max(0)).max(0);
        // The tax worksheet retains the 25% pool before the 28% pool when
        // limiting both by the aggregate net capital gain.
        let unrecaptured_gain = net_gain.min(line19);
        let collectible_gain = (net_gain - unrecaptured_gain).min(line18);
        Some(CapitalRatePools {
            net_gain: i64::try_from(net_gain).ok()?,
            line18: i64::try_from(line18).ok()?,
            line19: i64::try_from(line19).ok()?,
            collectible_gain: i64::try_from(collectible_gain).ok()?,
            unrecaptured_gain: i64::try_from(unrecaptured_gain).ok()?,
        })
    }
}

// Form 8960 net-gains worksheet 2(h)–2(j). Exclusions are signed: reversing
// an excluded loss is positive, excluding a gain is negative. Returns the
// current adjustment and the positive difference retained for next year.
pub(crate) fn niit_adjustment(
    prior: i64,
    current_exclusions: i64,
    capital_net: i64,
    capital_income: i64,
) -> [i64; 2] {
    let combined = prior + current_exclusions;
    let next =
        phase::cap(phase::floor(combined, 0), [phase::floor(capital_income - capital_net, 0)]);
    [combined - next, next]
}

// The Schedule D lines that total Form 8949, in the order of `Totals.sale_columns`: line 1a (box
// A or G, Exception 1), 1b (A or G), 2 (B or H), 3 (C or I), then 8a, 8b, 9 and 10 for the
// long-term boxes.
pub(crate) const SALE_LINES: [&str; 8] = ["1a", "1b", "2", "3", "8a", "8b", "9", "10"];

// The index in `SALE_LINES` of a lot by its holding period, whether its basis was reported to the
// IRS and whether it is a summary line (Exception 1 of the Instructions for Form 8949).
pub(crate) fn sale_line(
    term: crate::GainTerm,
    reporting: crate::BasisReporting,
    summary: bool,
) -> usize {
    let base = match term {
        crate::GainTerm::Short => 0,
        crate::GainTerm::Long => 4,
    };
    base + match reporting {
        crate::BasisReporting::ReportedToIrs => usize::from(!summary),
        crate::BasisReporting::NotReported => 2,
        crate::BasisReporting::NoForm1099B => 3,
    }
}

// One Form 8949 row, in cents. Column (h) is (d) less (e) plus (g). A summary row (Exception 1)
// adds only to Schedule D line 1a or 8a and is never printed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Form8949Row {
    pub entry: i64,
    pub row: u32,
    pub line: usize,
    pub box_letter: char,
    // Column (a): the entry's display name.
    pub label: crate::MessageArg,
    pub proceeds: i64,
    pub basis: i64,
    pub adjustment: i64,
    // Column (f): the codes in alphabetical order.
    pub codes: alloc::string::String,
    pub summary: bool,
}

impl Form8949Row {
    pub(crate) fn gain(&self) -> i64 {
        self.checked_gain().expect("admitted Form 8949 row gain")
    }
    pub(crate) fn checked_gain(&self) -> Option<i64> {
        i64::try_from(
            i128::from(self.proceeds) - i128::from(self.basis) + i128::from(self.adjustment),
        )
        .ok()
    }

    // The four amount lines of a printed row. The explanation names the entry, the box and the
    // codes; a summary row has no line of its own.
    pub(crate) fn emit(&self, r: &mut crate::FederalResult) {
        if self.summary {
            return;
        }
        let box_text = alloc::string::String::from(self.box_letter);
        emit_lines(
            r,
            &mut (),
            [
                ("proceeds", "(d)", self.proceeds),
                ("basis", "(e)", self.basis),
                ("adjustment", "(g)", self.adjustment),
                ("gain", "(h)", self.gain()),
            ],
            |(suffix, column, amount), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(
                        crate::TaxForm::Form8949,
                        &alloc::format!("1{column}"),
                    )),
                    &alloc::format!("form_8949_{}_{}_{}", self.entry, self.row, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.sales"],
                    "https://www.irs.gov/instructions/i8949",
                    if self.codes.is_empty() {
                        crate::Message::new(
                            "EXPLAIN_FORM_8949_ROW_NO_CODES",
                            alloc::vec![
                                self.label.clone(),
                                crate::MessageArg::Text(box_text.clone()),
                            ],
                        )
                    } else {
                        crate::Message::new(
                            "EXPLAIN_FORM_8949_ROW",
                            alloc::vec![
                                self.label.clone(),
                                crate::MessageArg::Text(box_text.clone()),
                                crate::MessageArg::Text(self.codes.clone()),
                            ],
                        )
                    },
                ))
            },
        );
    }
}

#[cfg(test)]
mod work_precision_tests {
    use super::*;

    #[test]
    fn retained_losses_offset_rate_pools_before_the_aggregate_cap() {
        let net =
            CapitalAmounts { short: -2000, long: 17000, collectibles: 4000, unrecaptured: 7000 };
        let expected = CapitalRatePools {
            net_gain: 15000,
            line18: 0,
            line19: 6000,
            collectible_gain: 0,
            unrecaptured_gain: 6000,
        };
        assert_eq!(net.rate_pools(3000), expected);
        let raw = CapitalAmounts { short: -1000, long: 20000, ..net };
        let result = raw.calculate([1000, 3000], false, false);
        assert_eq!((result.short, result.long, result.income), (-2000, 17000, 15000));
        assert_eq!(
            (
                result.net_gain,
                result.line18,
                result.line19,
                result.collectible_gain,
                result.unrecaptured_gain
            ),
            (15000, 0, 6000, 0, 6000)
        );
    }

    #[test]
    fn rate_pools_preserve_long_character_and_twenty_five_percent_priority() {
        let net =
            CapitalAmounts { short: -5000, long: 14000, collectibles: 20000, unrecaptured: 5000 };
        assert_eq!(
            net.rate_pools(0),
            CapitalRatePools {
                net_gain: 9000,
                line18: 15000,
                line19: 5000,
                collectible_gain: 4000,
                unrecaptured_gain: 5000,
            }
        );
        let clipped = CapitalAmounts { unrecaptured: 9000, ..net }.rate_pools(0);
        assert_eq!((clipped.unrecaptured_gain, clipped.collectible_gain), (9000, 0));
        let short_gain = CapitalAmounts { short: 7000, long: -2000, ..Default::default() };
        assert_eq!(short_gain.rate_pools(0).net_gain, 0);
        assert_eq!(short_gain.calculate([0; 2], true, false).income, 5000);
    }

    #[test]
    fn rate_pools_keep_cents_and_do_not_apply_a_second_annual_loss_floor() {
        let net = CapitalAmounts {
            short: -200001,
            long: 1700001,
            collectibles: 400001,
            unrecaptured: 700001,
        };
        assert_eq!(
            net.rate_pools(300001),
            CapitalRatePools {
                net_gain: 1500000,
                line18: 0,
                line19: 600000,
                collectible_gain: 0,
                unrecaptured_gain: 600000,
            }
        );
        let raw = CapitalAmounts { short: -100001, long: 2000002, ..net };
        let result = raw.calculate_cents([100000, 300001], false, false);
        assert_eq!(
            (result.short, result.long, result.income, result.net_gain, result.unrecaptured_gain),
            (-200001, 1700001, 1500000, 1500000, 600000)
        );
        let loss = CapitalAmounts { short: -900001, ..Default::default() };
        assert_eq!(loss.rate_pools(0).net_gain, 0);
        assert_eq!(loss.calculate_cents([0; 2], false, false).income, -300000);
        assert_eq!(loss.calculate_cents([0; 2], true, false).income, -150000);
        assert_eq!(loss.calculate_cents([0; 2], false, true).income, 0);
    }

    #[test]
    fn admitted_capital_cents_net_before_the_same_statutory_limit() {
        // Derived Pub974 Iterative work endpoints, not published tax oracles.
        // Existing ScheduleD owner retains admitted short/long character and
        // applies the same $3000/$1500 loss cap in the selected money unit.
        let gain = CapitalAmounts { short: 49, long: 49, ..Default::default() };
        assert_eq!(gain.calculate_cents([0; 2], false, false).income, 98);
        let loss = CapitalAmounts { short: -300050, long: 49, ..Default::default() };
        assert_eq!(loss.calculate_cents([0; 2], false, false).income, -300000);
        assert_eq!(loss.calculate_cents([0; 2], true, false).income, -150000);
        assert_eq!(loss.calculate_cents([0; 2], false, true).income, 0);
        let near = CapitalAmounts { short: -300048, long: 49, ..Default::default() };
        assert_eq!(near.calculate_cents([0; 2], false, false).income, -299999);
        assert_eq!(gain.calculate_cents([51, 0], false, false).income, 47);
    }
}
