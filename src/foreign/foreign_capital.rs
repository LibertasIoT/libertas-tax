// Coverage: Form 1116 section 904(b) adjustments — Supported (unit b9-6, 2026-10-10; 2026 draft
//   i1116 "Foreign Qualified Dividends and Capital Gains (Losses)" of 9/28/26, Worksheet A,
//   Worksheet B with its Line 2 Worksheet and Line 15 Worksheet, the Worksheet for Line 18;
//   2025 final i6251 line 8 Steps 2 and 4; Pub. 514 "Capital Gains and Losses"; IRC 904(b)(2)-(3);
//   26 CFR 1.904(b)-1). One owner of the qualified dividend rate adjustment (line 1a), the capital
//   gain and loss adjustments (lines 1a and 5, regular tax and AMT) and the line 18 reduction.
//   The rate bands are read from the worksheet that figured the return's tax (the Qualified
//   Dividends and Capital Gain Tax Worksheet or the Schedule D Tax Worksheet, and Form 6251
//   Part III), not rebuilt from the sources. Worksheet A runs when the adjustment exception is not
//   elected and the return has no benefit from the lower rates or no net capital gain; Worksheet B
//   runs for at most two categories when no 25%, 28%, 0% or 20% gain is in the return; any other
//   mix needs Pub. 514's rate-group method, which is not calculated (the credit is left out with
//   WARN_FTC_CAPITAL_RATE_GROUPS_LEFT_OUT). The adjustment exception (ordinary taxable income within
//   the 24% bracket and foreign net capital gain plus foreign qualified dividends below $20,000)
//   is presumed elected unless the person chose to adjust, and then neither the dividends nor the
//   gains and losses are adjusted.
//   Evidence: tests/foreign.rs foreign_capital_*, form_1116_f3_*; tests/form_1116.rs.
#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalStatus, income_tax};
use alloc::vec::Vec;

// Per 10,000: line 1a keeps 15% and 20% qualified dividends at 0.4054 and
// 0.5405 (AMT 0.5357 and 0.7143); the line 18 worksheet subtracts all 0%
// income and 0.5946, 0.4595, 0.3243 and 0.2432 of the 15%, 20%, 25% and 28%
// groups (AMT 0.4643, 0.2857, 0.1071 and nothing).
pub(crate) struct RateFactors {
    pub line_1a: [i64; 2],
    pub line_18: [i64; 4],
}
pub(crate) const REGULAR_FACTORS: RateFactors =
    RateFactors { line_1a: [4054, 5405], line_18: [5946, 4595, 3243, 2432] };
pub(crate) const AMT_FACTORS: RateFactors =
    RateFactors { line_1a: [5357, 7143], line_18: [4643, 2857, 1071, 0] };

// Line 1a rate adjustment and the Worksheet for Line 18 (i1116 "Foreign
// Qualified Dividends and Capital Gains", "Schedule D Filers" and line 18;
// i6251 line 8 steps 2 and 4; 26 CFR 1.904(b)-1(b)(3), (e) and (f)(4)(ii)).
// `bands` are the amounts taxed at 0%, 15%, 20%, 25% and 28%: Schedule D Tax
// Worksheet lines 22, 30, 33, 39 and 42 (the Qualified Dividends and Capital
// Gain Tax Worksheet lines 9, 17 and 20 when there is no 25% or 28% gain), or
// Form 6251 lines 23, 30, 33 and 36 (AMT taxes 28% gain at its ordinary rates,
// so that group has no AMT band).
pub(crate) struct RateBands {
    pub bands: [i64; 5],
    // The lower rates give a benefit: the Qualified Dividends and Capital Gain
    // Tax Worksheet line 5 is above zero and line 23 is below line 24 (Schedule
    // D Tax Worksheet lines 18 and 45/46; Form 6251 lines 17 and 38/39).
    pub adjust: bool,
}

impl RateBands {
    // Schedule D Tax Worksheet without a Form 4952 line 4g election (lines 3
    // to 5 and 8 are zero). Returns the bands and lines 18, 14 and 21.
    pub(crate) fn regular(
        taxable: i64,
        preference: [i64; 3],
        status: FederalStatus,
    ) -> (Self, [i64; 3]) {
        let (zero, fifteen) = income_tax::Rates::capital_limits(2026, status).unwrap();
        let top_24 = income_tax::Rates::for_year(2026, status).unwrap().ordinary[3];
        let line1 = positive(taxable);
        let [line10, collectibles, unrecaptured] = preference.map(positive);
        let line13 = positive(line10 - collectibles - unrecaptured);
        let line14 = positive(line1 - line13);
        let line16 = phase::cap(line1, [zero]);
        let line17 = phase::cap(line14, [line16]);
        let line18 = positive(line1 - line10);
        let line21 = phase::floor(line18, phase::cap(line14, [phase::cap(line1, [top_24])]));
        let line22 = line16 - line17;
        let line23 = phase::cap(line1, [line13]);
        let line30 = phase::cap(
            positive(line23 - line22),
            [positive(phase::cap(line1, [fifteen]) - line21 - line22)],
        );
        let line33 = positive(line23 - line22 - line30);
        let line39 = positive(unrecaptured - positive(line10 + line21 - line1));
        let line42 = if collectibles > 0 {
            positive(line1 - line21 - line22 - line30 - line33 - line39)
        } else {
            0
        };
        // Adjust only when line 18 is above zero and line 45 is below line 46.
        let required = line10 > 0
            && line18 > 0
            && crate::federal_rules::preferential_tax_with_special_gains(
                line1,
                line10,
                collectibles,
                unrecaptured,
                status,
            ) < crate::federal_rules::preferential_tax(line1, 0, status);
        (
            Self { bands: [line22, line30, line33, line39, line42], adjust: required },
            [line18, line14, line21],
        )
    }
    // Form 6251 Part III; `regular` is Schedule D Tax Worksheet lines 14 and
    // 21 (Qualified Dividends and Capital Gain Tax Worksheet line 5 for both).
    pub(crate) fn amt(
        base: i64,
        preference: [i64; 3],
        regular: [i64; 2],
        status: FederalStatus,
    ) -> (Self, i64) {
        let rates = income_tax::AmtRates::for_year(2026, status).unwrap();
        let (zero, fifteen) = income_tax::Rates::capital_limits(2026, status).unwrap();
        let [total, collectibles, unrecaptured] = preference.map(positive);
        let line12 = rates.taxable(base);
        let line13 = positive(total - collectibles - unrecaptured);
        let line16 = phase::cap(line12, [line13 + unrecaptured]);
        let line17 = line12 - line16;
        let line21 = positive(zero - regular[0]);
        let line22 = phase::cap(line12, [line13]);
        let line23 = phase::cap(line21, [line22]);
        let line30 = phase::cap(line22 - line23, [positive(fifteen - line21 - regular[1])]);
        let line33 = line22 - line23 - line30;
        let line36 = line16 - line22;
        let line38 = rates.ordinary_cents(line17) + line30 * 15 + line33 * 20 + line36 * 25;
        let required =
            line16 > 0 && line17 > 0 && dollars(line38) < dollars(rates.ordinary_cents(line12));
        (Self { bands: [line23, line30, line33, line36, 0], adjust: required }, line17)
    }
    // Line 1a: foreign qualified dividends split ratably across the 0%, 15%
    // and 20% bands, the only groups qualified dividends occupy.
    pub(crate) fn line_1a_reduction(&self, qualified: i64, factors: &RateFactors) -> i64 {
        let total = self.bands[..3].iter().sum::<i64>();
        if !self.adjust || total == 0 {
            return 0;
        }
        let fifteen = fraction(qualified, self.bands[1], total);
        let twenty = fraction(qualified, self.bands[2], total);
        qualified
            - fraction(fifteen, factors.line_1a[0], 10000)
            - fraction(twenty, factors.line_1a[1], 10000)
    }
    // Worksheet for Line 18 lines 2-11.
    pub(crate) fn line_18_reduction(&self, factors: &RateFactors) -> i64 {
        if !self.adjust {
            return 0;
        }
        self.bands[0]
            + (0..4).map(|i| fraction(self.bands[i + 1], factors.line_18[i], 10000)).sum::<i64>()
    }
    // Whether Worksheet B can be used: none of the gain is taxed at 0%, 20%,
    // 25% or 28% (the return completes no 28% or unrecaptured section 1250
    // worksheet and the Qualified Dividends and Capital Gain Tax Worksheet
    // lines 9 and 20 are zero).
    pub(crate) fn worksheet_b_allowed(&self) -> bool {
        self.bands[0] == 0 && self.bands[2] == 0 && self.bands[3] == 0 && self.bands[4] == 0
    }
}

// The return's Schedule D amounts the worksheets read, whole dollars (AMT:
// the AMT Schedule D): line 7 (net short-term), line 15 (net long-term) and
// line 16 (net capital gain).
#[derive(Clone, Copy, Default)]
pub(crate) struct ScheduleD {
    pub short: i64,
    pub long: i64,
    pub net_gain: i64,
}

pub(crate) struct CapitalInput {
    // Foreign source net short-term and long-term gain (loss), whole dollars,
    // by category slot (general, passive, foreign branch, section 901(j)).
    pub net: [[i64; 2]; 4],
    pub schedule_d: ScheduleD,
    // The net capital gain chosen on Form 4952 line 4g, assumed to come from
    // U.S. items first.
    pub elected_gain: i64,
    // The adjustment exception is elected: nothing is adjusted.
    pub excepted: bool,
    // 4054 for the regular tax, 5357 for the AMT (per 10,000).
    pub long_term_factor: i64,
}

#[derive(Default)]
pub(crate) struct Adjusted {
    // By category slot, whole dollars: gains for line 1a and losses (positive)
    // for line 5, after the adjustments.
    pub gains: [i64; 4],
    pub losses: [i64; 4],
    // Which worksheet ran, and its printed lines for the review.
    pub worksheet: Option<Worksheet>,
    pub lines: Vec<(&'static str, i64)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Worksheet {
    A,
    B,
}

pub(crate) enum Outcome {
    Adjusted(Adjusted),
    // The mix needs Pub. 514's rate-group method.
    LeftOut,
}

// Foreign capital gains and losses to lines 1a and 5. `bands` are the return's
// rate bands (regular or AMT).
pub(crate) fn adjust(input: &CapitalInput, bands: &RateBands) -> Outcome {
    let nets: [i64; 4] = core::array::from_fn(|c| input.net[c][0] + input.net[c][1]);
    let active: Vec<usize> =
        (0..4).filter(|&c| input.net[c][0] != 0 || input.net[c][1] != 0).collect();
    let mut out = Adjusted::default();
    if active.is_empty() {
        return Outcome::Adjusted(out);
    }
    let unadjusted = |out: &mut Adjusted| {
        for &c in &active {
            if nets[c] > 0 {
                out.gains[c] = nets[c];
            } else {
                out.losses[c] = -nets[c];
            }
        }
    };
    if input.excepted {
        unadjusted(&mut out);
        return Outcome::Adjusted(out);
    }
    let line16 = positive(input.schedule_d.net_gain - input.elected_gain);
    let worksheet_a = !bands.adjust || input.schedule_d.net_gain <= 0;
    if worksheet_a {
        if active.len() > 2 {
            return Outcome::LeftOut;
        }
        out.worksheet = Some(Worksheet::A);
        // Worksheet A.
        let line2: i64 = active.iter().map(|&c| nets[c]).sum();
        out.lines.push(("worksheet_a_line_2", phase::floor(line2, 0)));
        out.lines.push(("worksheet_a_line_3", line16));
        if line2 <= 0 {
            unadjusted(&mut out);
            return Outcome::Adjusted(out);
        }
        let line4 = line2 - line16;
        out.lines.push(("worksheet_a_line_4", phase::floor(line4, 0)));
        if line4 <= 0 {
            unadjusted(&mut out);
            return Outcome::Adjusted(out);
        }
        let positives: Vec<usize> = active.iter().copied().filter(|&c| nets[c] > 0).collect();
        for &c in &active {
            if nets[c] <= 0 {
                out.losses[c] = -nets[c];
            }
        }
        if positives.len() == 1 {
            let c = positives[0];
            out.gains[c] = positive(nets[c] - line4);
        } else {
            for &c in &positives {
                // Line 6 factor rounded to four places, as the form directs.
                let factor = fraction(10000, nets[c], line2);
                let line7 = fraction(line4, factor, 10000);
                out.gains[c] = nets[c] - line7;
            }
        }
        return Outcome::Adjusted(out);
    }
    // Worksheet B: at most two categories and no 0%, 20%, 25% or 28% gain.
    if active.len() > 2 || !bands.worksheet_b_allowed() {
        return Outcome::LeftOut;
    }
    out.worksheet = Some(Worksheet::B);
    worksheet_b(input, &active, line16, &mut out);
    Outcome::Adjusted(out)
}

// Worksheet B with the Line 2 Worksheet and the Line 15 Worksheet.
fn worksheet_b(input: &CapitalInput, active: &[usize], line16: i64, out: &mut Adjusted) {
    let factor = input.long_term_factor;
    // Line 1: [category][short, long].
    let line1: Vec<[i64; 2]> = active.iter().map(|&c| input.net[c]).collect();
    let n = active.len();
    // Line 2 Worksheet.
    let line2: Vec<i64> = line1.iter().map(|v| v[0] + v[1]).collect();
    let l2_line3: i64 = line2.iter().sum();
    let mut adjustment = alloc::vec![[0_i64; 2]; n];
    if l2_line3 > 0 && line16 > 0 {
        let l2_line5 = l2_line3 - line16;
        if l2_line5 > 0 {
            let positive_categories = line2.iter().filter(|&&v| v > 0).count();
            for k in 0..n {
                if line2[k] <= 0 {
                    continue;
                }
                let line6 = if positive_categories == 1 {
                    l2_line5
                } else {
                    fraction(l2_line5, line2[k], l2_line3)
                };
                let gains: Vec<usize> = (0..2).filter(|&t| line1[k][t] > 0).collect();
                if gains.len() == 2 {
                    for &t in &gains {
                        adjustment[k][t] = fraction(line6, line1[k][t], line2[k]);
                    }
                } else if let Some(&t) = gains.first() {
                    adjustment[k][t] = line6;
                }
            }
        }
    }
    out.lines.push(("worksheet_b_line_2", adjustment.iter().map(|v| v[0] + v[1]).sum()));
    // Line 3: gains less their line 2 amounts.
    let line3: Vec<[i64; 2]> =
        (0..n)
            .map(|k| {
                core::array::from_fn(|t| {
                    if line1[k][t] > 0 { line1[k][t] - adjustment[k][t] } else { 0 }
                })
            })
            .collect();
    // Line 4: the net U.S. long-term capital loss.
    let foreign_long: i64 = input.net.iter().map(|v| v[1]).sum();
    let foreign_short: i64 = input.net.iter().map(|v| v[0]).sum();
    let us_long = input.schedule_d.long - foreign_long;
    let us_short = input.schedule_d.short - foreign_short;
    let line4 = positive(-us_long);
    let mut long_handled = alloc::vec![false; n];
    let mut gains = alloc::vec![0_i64; n];
    let mut losses = alloc::vec![0_i64; n];
    if line4 > 0 {
        let line5: i64 = adjustment.iter().map(|v| v[1]).sum();
        let line6 = line4 - line5;
        out.lines.push(("worksheet_b_line_6", positive(line6)));
        if line6 > 0 {
            let columns: Vec<usize> = (0..n).filter(|&k| line3[k][1] > 0).collect();
            if columns.len() == 1 {
                let k = columns[0];
                let line7 = positive(line3[k][1] - line6);
                let line13 = fraction(line7, factor, 10000);
                gains[k] += phase::cap(line6, [line3[k][1]]) + line13;
                long_handled[k] = true;
            } else if columns.len() == 2 {
                let line7: i64 = columns.iter().map(|&k| line3[k][1]).sum();
                for &k in &columns {
                    let line9 = fraction(line6, line3[k][1], line7);
                    let line10 = line3[k][1] - line9;
                    gains[k] += fraction(line10, factor, 10000) + line9;
                    long_handled[k] = true;
                }
            }
        }
    }
    // Line 15.
    for k in 0..n {
        gains[k] += line3[k][0];
        if !long_handled[k] {
            gains[k] += fraction(line3[k][1], factor, 10000);
        }
        if line1[k][1] < 0 {
            losses[k] += fraction(-line1[k][1], factor, 10000);
        }
    }
    // Short-term losses: the Line 15 Worksheet.
    let st_losses: Vec<(usize, i64)> =
        (0..n).filter(|&k| line1[k][0] < 0).map(|k| (k, -line1[k][0])).collect();
    if !st_losses.is_empty() {
        let line1_w = positive(us_short);
        let st_gain: i64 = (0..n).map(|k| line3[k][0]).sum();
        let line3_w = line1_w + st_gain;
        let adjusted: Vec<(usize, i64)> = if st_losses.len() == 1 {
            let (k, loss) = st_losses[0];
            let line6 = positive(line3_w);
            let line7 = positive(loss - line6);
            let line10 = fraction(line7, factor, 10000) + phase::cap(loss, [line6]);
            alloc::vec![(k, line10)]
        } else if line1_w == 0 {
            st_losses.iter().map(|&(k, loss)| (k, fraction(loss, factor, 10000))).collect()
        } else {
            let line14: i64 = st_losses.iter().map(|v| v.1).sum();
            let line16_w = line14 - line1_w;
            if line16_w <= 0 {
                st_losses.clone()
            } else {
                let line18 = line1_w + fraction(line16_w, factor, 10000);
                st_losses.iter().map(|&(k, loss)| (k, fraction(line18, loss, line14))).collect()
            }
        };
        for (k, amount) in adjusted {
            losses[k] += amount;
        }
    }
    for (k, &c) in active.iter().enumerate() {
        out.gains[c] = gains[k];
        out.losses[c] = losses[k];
    }
}
