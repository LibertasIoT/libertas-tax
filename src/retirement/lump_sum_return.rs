const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f4972--dft.pdf";
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::push_line;
use crate::lump_sum::{Form4972Election, Form4972Participant, LumpSumMethod, hundredths};
use crate::year_constants::{
    LUMP_SUM_MINIMUM_ALLOWANCE_CAP_DOLLARS, LUMP_SUM_MINIMUM_ALLOWANCE_PHASEOUT_START_DOLLARS,
    LUMP_SUM_RATE_SCHEDULE_CENTS,
};
use crate::{
    FederalDraft, FederalIncomeEntry, FederalResult, FormLine, RetirementKind, RetirementTaxable,
    RetirementTreatment, TaxForm,
};
use alloc::vec::Vec;

// The 1986 single-individual tax on one tenth of `x`, times ten: for a bracket with base b, rate r
// and lower bound L, 10 * tax(x / 10) = 10 * b + r * (x - 10 * L). Exact; no intermediate rounding.
pub(crate) fn ten_times_tax(x: i64) -> i64 {
    if x <= 0 {
        return 0;
    }
    let (over, base, rate) = LUMP_SUM_RATE_SCHEDULE_CENTS
        .iter()
        .rev()
        .find(|(over, ..)| x >= over * 10)
        .copied()
        .unwrap_or(LUMP_SUM_RATE_SCHEDULE_CENTS[0]);
    base * 10 + fraction(x - over * 10, rate, 100)
}

// Lines 12 to 16: the minimum distribution allowance of an adjusted total taxable amount, in cents.
fn minimum_allowance(line12: i64) -> i64 {
    let line13 = phase::cap(fraction(line12, 1, 2), [LUMP_SUM_MINIMUM_ALLOWANCE_CAP_DOLLARS * 100]);
    let line14 = positive(line12 - LUMP_SUM_MINIMUM_ALLOWANCE_PHASEOUT_START_DOLLARS * 100);
    phase::phase_out(line13, fraction(line14, 20, 100))
}

// One participant's records, summed as the form sums them. Amounts in cents.
#[derive(Default)]
struct Group {
    method: Option<LumpSumMethod>,
    // Box 2a, box 3, box 6 (included or not), the NUA Worksheet line E, box 8a.
    taxable: i64,
    capital_gain: i64,
    nua: i64,
    nua_capital: i64,
    annuity: i64,
    estate_tax: i64,
    death_benefit: i64,
    // Hundredths of a percent; a share needs one statement.
    percent: Option<u32>,
    annuity_percent: Option<u32>,
    include_nua: bool,
}

// Every line of one Form 4972, in cents.
#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct Form4972 {
    pub line6: i64,
    pub line7: i64,
    pub line8: i64,
    pub line9: i64,
    pub line10: i64,
    pub line11: i64,
    pub line12: i64,
    pub line16: i64,
    pub line17: i64,
    pub line18: i64,
    pub line19: i64,
    pub line21: i64,
    pub line22: i64,
    pub line25: i64,
    pub line28: i64,
    pub line29: i64,
    pub tax_cents: i64,
}

impl Form4972 {
    pub(crate) fn line30(&self) -> i64 {
        dollars(self.tax_cents)
    }
}

impl Group {
    fn figure(&self) -> Form4972 {
        let method = self.method.unwrap_or(LumpSumMethod::TenYear);
        let mut f = Form4972::default();
        // Step 2 of the multiple-recipient worksheet restates a recipient's share as the whole
        // distribution (box 9a); the form is silent on lines 9 and 18, so the recipient's share
        // of the death benefit exclusion and of the estate tax is divided the same way.
        let whole_of = |value: i64| match self.percent {
            Some(p) => fraction(value, 10_000, i64::from(p)),
            None => value,
        };
        let nua = if self.include_nua { self.nua } else { 0 };
        let mut estate_capital = 0;
        let mut death_capital = 0;
        if method.capital_gain() {
            // NUA Worksheet line G, then the Death Benefit Worksheet's capital-gain share
            // (line E = line D x the capital-gain share of the distribution) and the estate
            // tax's.
            let capital = self.capital_gain + self.nua_capital;
            let whole = self.taxable + nua;
            if whole > 0 {
                estate_capital = fraction(self.estate_tax, capital, whole);
                death_capital = fraction(self.death_benefit, capital, whole);
            }
            f.line6 = positive(capital - estate_capital - death_capital);
            f.line7 = fraction(f.line6, 20, 100);
        }
        if method.ten_year() {
            // Line 8: box 2a, less box 3 when Part II is completed, plus the net unrealized
            // appreciation (all of box 6 without Part II; the non-capital part with it).
            let ordinary = if method.capital_gain() {
                self.taxable - self.capital_gain + nua - self.nua_capital
            } else {
                self.taxable + nua
            };
            f.line8 = whole_of(ordinary);
            // Line 9: the death benefit exclusion of a participant who died before 1996-08-21,
            // line D less the capital-gain share E when Part II is completed.
            f.line9 = phase::cap(whole_of(self.death_benefit - death_capital), [f.line8]);
            f.line10 = f.line8 - f.line9;
            f.line11 = match self.annuity_percent {
                Some(p) => fraction(self.annuity, 10_000, i64::from(p)),
                None => self.annuity,
            };
            f.line12 = f.line10 + f.line11;
            f.line16 = minimum_allowance(f.line12);
            f.line17 = f.line12 - f.line16;
            f.line18 = positive(whole_of(self.estate_tax - estate_capital));
            f.line19 = positive(f.line17 - f.line18);
            f.line25 = ten_times_tax(f.line19);
            if f.line11 > 0 {
                f.line21 = fraction(f.line16, f.line11, f.line12);
                f.line22 = f.line11 - f.line21;
                f.line28 = ten_times_tax(f.line22);
            }
            f.line29 = f.line25 - f.line28;
            if let Some(p) = self.percent {
                f.line29 = fraction(f.line29, i64::from(p), 10_000);
            }
        }
        f.tax_cents = f.line7 + f.line29;
        f
    }
}

// Who a record's participant is: the filer's own plan, or a decedent by birth and death date.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Participant {
    owner: u8,
    born: u32,
    died: u32,
}

fn participant(entry: &FederalIncomeEntry, data: &Form4972Election) -> Participant {
    let owner = u8::from(entry.owner == crate::DocumentOwner::Spouses);
    match data.participant {
        Form4972Participant::Own { .. } => Participant { owner, born: 0, died: 0 },
        Form4972Participant::Decedent { born, died, .. } => Participant { owner, born, died },
    }
}

// A code A record that is not elected: its owner and date of death identify the participant as
// far as the record states it (a 7A is the owner's own plan; a 4A names only the date of death).
fn left_with_other_income(entry: &FederalIncomeEntry) -> Option<(u8, u32)> {
    let v = entry.income.retirement()?;
    let a = crate::DistributionCodes::CODE_A.0;
    if v.box7 & a == 0 || v.kind != RetirementKind::Pension {
        return None;
    }
    let died = match &v.ordinary_distribution {
        Some(RetirementTreatment::LumpSum { .. }) => return None,
        Some(RetirementTreatment::Exception {
            facts: crate::RetirementException::Death { died },
            ..
        }) => *died,
        _ => 0,
    };
    Some((u8::from(entry.owner == crate::DocumentOwner::Spouses), died))
}

// Every Form 4972 of the return: the issues about records of one participant, the form lines, and
// Form 1040 line 16 box 2 in whole dollars (the sum of the line 30 amounts). The records left out
// were already replaced by unknowns::supported.
pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut groups: Vec<(Participant, Vec<&FederalIncomeEntry>, Group)> = Vec::new();
    for entry in &d.income {
        let Some(v) = entry.income.retirement() else {
            continue;
        };
        let Some(RetirementTreatment::LumpSum { data }) = &v.ordinary_distribution else {
            continue;
        };
        let Some(RetirementTaxable::Payer) = v.taxable else {
            continue;
        };
        let taxable = v.box_2a;
        let key = participant(entry, data);
        let at = groups.iter().position(|(k, ..)| *k == key).unwrap_or_else(|| {
            groups.push((key, Vec::new(), Group::default()));
            groups.len() - 1
        });
        let (_, entries, group) = &mut groups[at];
        let first = entries.is_empty();
        let differs =
            !first && (group.method != Some(data.method) || group.include_nua != data.include_nua);
        entries.push(entry);
        if first {
            group.method = Some(data.method);
            group.include_nua = data.include_nua;
        }
        let shared = v.box_9a_percent.is_some() || v.box_8b_percent.is_some();
        if entries.len() > 1 {
            let context = crate::interview::income_issue_context(entry);
            if differs {
                crate::require_localized!(
                    r,
                    false,
                    &context,
                    "Form 4972",
                    "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY",
                    "All of a participant's lump-sum distributions in one year must be taxed the same way. Choose the same Form 4972 method for each 1099-R of this participant, or tax none of them separately.",
                );
            }
            if shared || group.percent.is_some() || group.annuity_percent.is_some() {
                crate::require_localized!(
                    r,
                    false,
                    &context,
                    "Form 4972",
                    "ISSUE_LUMP_SUM_SHARED_AND_COMBINED",
                    "A lump sum shared with other recipients (box 9a) or with an annuity share (box 8b) is figured from its own statement. It cannot be combined with another 1099-R of the same participant's lump sum.",
                );
            }
        }
        group.taxable += taxable;
        group.capital_gain += v.box_3;
        group.nua += v.box_6;
        group.nua_capital += data.nua_capital_share(taxable, &v);
        group.annuity += v.box_8a;
        group.estate_tax += data.participant.estate_tax();
        group.death_benefit += data.participant.death_benefit();
        group.percent = group.percent.or(hundredths(v.box_9a_percent));
        group.annuity_percent = group.annuity_percent.or(hundredths(v.box_8b_percent));
    }
    // A code A record of the same participant taxed with the other income.
    for (key, ..) in &groups {
        let other = d.income.iter().find(|e| {
            left_with_other_income(e)
                .is_some_and(|(owner, died)| owner == key.owner && died == key.died)
        });
        if let Some(other) = other {
            crate::require_localized!(
                r,
                false,
                &crate::interview::income_issue_context(other),
                "Form 4972",
                "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY",
                "All of a participant's lump-sum distributions in one year must be taxed the same way. Choose the same Form 4972 method for each 1099-R of this participant, or tax none of them separately.",
            );
        }
    }
    let mut total = 0;
    for (_, entries, group) in &groups {
        let form = group.figure();
        let id = entries[0].id;
        let method = group.method.unwrap_or(LumpSumMethod::TenYear);
        let input = "income.retirement.ordinary_distribution";
        let mut row = |line: &str, amount: i64| {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4972, line)),
                &alloc::format!("lump_sum_{id}_line_{line}"),
                amount,
                &[input, "income.retirement.taxable"],
                SOURCE,
                msg!("EXPLAIN_LUMP_SUM_N_LINE_N"),
            );
        };
        if method.capital_gain() {
            row("6", form.line6);
            row("7", form.line7);
        }
        if method.ten_year() {
            row("8", form.line8);
            if form.line9 > 0 {
                row("9", form.line9);
                row("10", form.line10);
            }
            if form.line11 > 0 {
                row("11", form.line11);
            }
            row("12", form.line12);
            row("16", form.line16);
            row("17", form.line17);
            row("18", form.line18);
            row("19", form.line19);
            if form.line11 > 0 {
                row("21", form.line21);
                row("22", form.line22);
            }
            row("25", form.line25);
            if form.line11 > 0 {
                row("28", form.line28);
            }
            row("29", form.line29);
        }
        row("30", form.line30() * 100);
        total += form.line30();
    }
    if !groups.is_empty() {
        crate::line_emission::line(
            r,
            "lump_sum_tax",
            total,
            &["income.retirement.ordinary_distribution"],
            SOURCE,
            msg!("EXPLAIN_LUMP_SUM_TAX"),
        );
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(taxable: i64) -> Group {
        Group { method: Some(LumpSumMethod::TenYear), taxable: taxable * 100, ..Group::default() }
    }
    // Form 4972 line 30 in whole dollars for a plain 10-year distribution of `taxable` dollars.
    fn tax(taxable: i64) -> i64 {
        group(taxable).figure().line30()
    }

    // Form 4972 page 4 Tax Rate Schedule, hand-computed on the printed (one tenth) amount:
    // line 23 = line 19 / 10, line 24 = base + rate x (line 23 - over), line 25 = 10 x line 24.
    #[test]
    fn the_schedule_matches_the_printed_form_at_every_bracket_boundary() {
        // (line 19 dollars, expected line 25 dollars). A bracket's lower bound and one dollar above.
        let cases: &[(i64, i64)] = &[
            (0, 0),
            // 11% to 11,900: 11% x 11,900 = 1,309.00 (the next bracket's base).
            (11_900, 1_309),
            // + 1 dollar at 12%: 1,309 + 0.12 = 1,309.12 -> rounds to 1,309 in whole dollars.
            (11_901, 1_309),
            // 22,700: 2,605.
            (22_700, 2_605),
            (22_701, 2_605),
            // 45,300: 5,769.
            (45_300, 5_769),
            (45_301, 5_769),
            // 66,900: 9,009.
            (66_900, 9_009),
            (66_901, 9_009),
            // 91,700: 12,977.
            (91_700, 12_977),
            (91_701, 12_977),
            // 114,400: 17,063.
            (114_400, 17_063),
            (114_401, 17_063),
            // 137,100: 21,603.
            (137_100, 21_603),
            (137_101, 21_603),
            // 171,600: 29,538.
            (171_600, 29_538),
            (171_601, 29_538),
            // 228,800: 44,410.
            (228_800, 44_410),
            (228_801, 44_410),
            // 286,000: 61,570.
            (286_000, 61_570),
            (286_001, 61_570),
            // 343,200: 81,018.
            (343_200, 81_018),
            (343_201, 81_018),
            // 423,000: 111,342.
            (423_000, 111_342),
            (423_001, 111_342),
            // 571,900: 173,880.
            (571_900, 173_880),
            (571_901, 173_880),
            // 857,900: 311,160; one dollar above adds 50 cents, which rounds up.
            (857_900, 311_160),
            (857_901, 311_161),
            // 1,000,000: 311,160 + 50% x 142,100 = 382,210.
            (1_000_000, 382_210),
        ];
        for &(line19, expected) in cases {
            assert_eq!(dollars(ten_times_tax(line19 * 100)), expected, "line 19 {line19}");
        }
    }

    // Lines 13 to 16 on a plain distribution (no annuity, no estate tax): line 12 = 2a.
    #[test]
    fn the_minimum_distribution_allowance_boundaries() {
        // 2a 20,000: line 13 = min(10,000, 10,000) = 10,000; line 14 = 0; line 16 = 10,000;
        // line 19 = 10,000; line 25 = 10 x 11% x 1,000 = 1,100.
        assert_eq!(minimum_allowance(2_000_000), 1_000_000);
        assert_eq!(tax(20_000), 1_100);
        // 2a 50,000: line 13 = 10,000; line 14 = 30,000; line 15 = 6,000; line 16 = 4,000;
        // line 19 = 46,000; line 25 = 5,769 + 15% x 700 = 5,874.
        assert_eq!(minimum_allowance(5_000_000), 400_000);
        assert_eq!(tax(50_000), 5_874);
        // 2a 70,000: line 14 = 50,000; line 15 = 10,000; line 16 = 0; line 25 = 9,009 + 16% x
        // 3,100 = 9,505.
        assert_eq!(minimum_allowance(7_000_000), 0);
        assert_eq!(tax(70_000), 9_505);
        // Below 20,000 the allowance is half of line 12: 15,000 -> 7,500; line 19 = 7,500; line 25
        // = 10 x 11% x 750 = 825.
        assert_eq!(minimum_allowance(1_500_000), 750_000);
        assert_eq!(tax(15_000), 825);
        // One dollar under 70,000: line 14 = 49,999; line 15 = 9,999.80; line 16 = 0.20.
        assert_eq!(minimum_allowance(6_999_900), 20);
    }

    // Pub 575 (2024) Example 2, lines 11 to 28 on the annuity contract.
    #[test]
    fn the_annuity_contract_share_removes_its_own_tax() {
        let mut g = group(160_000);
        g.annuity = 10_000 * 100;
        let f = g.figure();
        assert_eq!(f.line12, 17_000_000);
        assert_eq!(f.line16, 0);
        assert_eq!(f.line21, 0);
        assert_eq!(f.line22, 1_000_000);
        assert_eq!(f.line25, 2_917_000);
        assert_eq!(f.line28, 110_000);
        assert_eq!(f.line30(), 28_070);
    }
}
