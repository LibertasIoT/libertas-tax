// Section 1 ordinary and capital-gain tax, with verified 2023–2026 filing
// Tax Table lookups below $100,000. Current-year ordinary returns use the table;
// special worksheets retain their separately selected calculation paths.
// All inputs are whole dollars; rate-schedule arithmetic retains cents.
// Base-year parameters for Schedule J must use that year's filing status.
// 2023: https://www.irs.gov/pub/irs-drop/rp-22-38.pdf sections 3.01, 3.03
// 2024: https://www.irs.gov/pub/irs-drop/rp-23-34.pdf sections 3.01, 3.03
// 2025: https://www.irs.gov/pub/irs-drop/rp-24-40.pdf sections 3.01, 3.03
// 2026: https://www.irs.gov/irb/2025-45_IRB sections 4.01, 4.03
// 2026 filing table: https://www.irs.gov/pub/irs-dft/p1040--dft.pdf (Sep. 16 draft)
use crate::FederalStatus;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};

pub(crate) struct Rates {
    pub(crate) ordinary: [i64; 6],
    capital: (i64, i64),
    use_filing_table: bool,
}
impl Rates {
    pub(crate) fn for_year(year: u32, status: FederalStatus) -> Option<Self> {
        Self::with_method(year, status, false)
    }

    pub(crate) fn capital_limits(year: u32, status: FederalStatus) -> Option<(i64, i64)> {
        if year == 2022 {
            //2023 Form8801 PartIII refigures the preceding2022 return.
            return Some(match status {
                FederalStatus::Single => (41675, 459750),
                FederalStatus::Head => (55800, 488500),
                FederalStatus::Joint | FederalStatus::Surviving => (83350, 517200),
                FederalStatus::Separate => (41675, 258600),
            });
        }
        Some(Self::for_year(year, status)?.capital)
    }

    // Filing tables and Schedule J base-year recomputation are distinct methods.
    // Schedule J instructions require rate schedules even below $100,000.
    pub(crate) fn with_method(
        year: u32,
        status: FederalStatus,
        use_filing_table: bool,
    ) -> Option<Self> {
        if use_filing_table && year > 2026 {
            return None;
        }
        use FederalStatus::*;
        let (ordinary, capital) = match (year, status) {
            (2023, Single) => ([11000, 44725, 95375, 182100, 231250, 578125], (44625, 492300)),
            (2023, Joint | Surviving) => {
                ([22000, 89450, 190750, 364200, 462500, 693750], (89250, 553850))
            }
            (2023, Head) => ([15700, 59850, 95350, 182100, 231250, 578100], (59750, 523050)),
            (2023, Separate) => ([11000, 44725, 95375, 182100, 231250, 346875], (44625, 276900)),
            (2024, Single) => ([11600, 47150, 100525, 191950, 243725, 609350], (47025, 518900)),
            (2024, Joint | Surviving) => {
                ([23200, 94300, 201050, 383900, 487450, 731200], (94050, 583750))
            }
            (2024, Head) => ([16550, 63100, 100500, 191950, 243700, 609350], (63000, 551350)),
            (2024, Separate) => ([11600, 47150, 100525, 191950, 243725, 365600], (47025, 291850)),
            (2025, Single) => ([11925, 48475, 103350, 197300, 250525, 626350], (48350, 533400)),
            (2025, Joint | Surviving) => {
                ([23850, 96950, 206700, 394600, 501050, 751600], (96700, 600050))
            }
            (2025, Head) => ([17000, 64850, 103350, 197300, 250500, 626350], (64750, 566700)),
            (2025, Separate) => ([11925, 48475, 103350, 197300, 250525, 375800], (48350, 300000)),
            (2026, Single) => ([12400, 50400, 105700, 201775, 256225, 640600], (49450, 545500)),
            (2026, Joint | Surviving) => {
                ([24800, 100800, 211400, 403550, 512450, 768700], (98900, 613700))
            }
            (2026, Head) => ([17700, 67450, 105700, 201750, 256200, 640600], (66200, 579600)),
            (2026, Separate) => ([12400, 50400, 105700, 201775, 256225, 384350], (49450, 306850)),
            _ => return None,
        };
        Some(Self { ordinary, capital, use_filing_table })
    }

    // Return cents so ordinary and preferential components round once, together.
    pub(crate) fn ordinary_cents(&self, income: i64) -> i64 {
        let mut previous = 0;
        let mut result = 0;
        for (limit, rate) in
            self.ordinary.into_iter().chain([i64::MAX]).zip([10, 12, 22, 24, 32, 35, 37])
        {
            result += positive(phase::cap(income, [limit]) - previous) * rate;
            if income <= limit {
                break;
            }
            previous = limit;
        }
        result
    }

    // The IRS tables use a midpoint per income interval. Verified against every
    // row of the published 2023–2026 tables (tests/fixtures). Below $3,000 all
    // statuses remain in the 10% band, so truncating a half-dollar midpoint
    // cannot change the resulting whole-dollar table tax.
    pub(crate) fn worksheet_ordinary_cents(&self, income: i64) -> i64 {
        if !self.use_filing_table || income >= 100000 {
            return self.ordinary_cents(income);
        }
        let midpoint = match income {
            ..5 => 0,
            5..15 => 10,
            15..25 => 20,
            25..3000 => crate::kernel::ratio::trunc_i64(income, 25) * 25 + 12,
            _ => crate::kernel::ratio::trunc_i64(income, 50) * 50 + 25,
        };
        dollars(self.ordinary_cents(midpoint)) * 100
    }

    // Project Form6251 lines13/15/20/27 into next year's Form8801.
    // The first pair uses AMT gains; the band occupants use regular tax.
    pub(crate) fn minimum_tax_gains(
        &self,
        taxable: i64,
        regular: [i64; 3],
        alternative: [i64; 3],
    ) -> Option<crate::MinimumTaxRateFacts> {
        if regular[0] == 0 && alternative[0] == 0 {
            return None;
        }
        let [total, collectibles, unrecaptured] = regular;
        let zero = positive(taxable - total + collectibles + unrecaptured);
        let fifteen = phase::cap(zero, [phase::floor(positive(taxable - total), self.ordinary[3])]);
        let [total, collectibles, unrecaptured] = alternative;
        Some(
            crate::MinimumTaxGains {
                preferential: (total - collectibles - unrecaptured) * 100,
                combined: (total - collectibles) * 100,
                regular_zero: zero * 100,
                regular_fifteen: fifteen * 100,
            }
            .into(),
        )
    }

    // Preserve the distinct Schedule D line 12/35 amounts used by Form 8615.
    pub(crate) fn schedule_d(
        &self,
        income: i64,
        preferential: i64,
        special: i64,
        original_unrecaptured: i64,
        has_collectibles: bool,
    ) -> i64 {
        let gain = phase::cap(positive(preferential - special), [positive(income)]);
        let ordinary = income - gain;
        let (zero, fifteen) = self.capital;
        let free = phase::cap(gain, [positive(zero - ordinary)]);
        let ordinary_base = positive(income - preferential);
        // Fill ordinary brackets through 24%, then apply the 25% and 28% ceilings.
        // Use the 24% bracket ceiling, not min(total incremental tax, 28% of the gain):
        // the latter would overtax a gain spanning the 24% and 32% brackets.
        let below_cap = phase::cap(ordinary, [phase::floor(ordinary_base, self.ordinary[3])]);
        let middle =
            phase::cap(gain - free, [positive(phase::cap(income, [fifteen]) - below_cap - free)]);
        // Worksheet lines 35–43 allocate the remaining 25% component first.
        let skip_special = income <= zero || income == free + middle;
        let at_25 = if skip_special {
            0
        } else {
            positive(original_unrecaptured - positive(preferential + below_cap - income))
        };
        // Line 42 is signed (unlike line 39), and lines 41–43 are skipped when
        // Schedule D line 18 is absent. Family worksheets can make this negative.
        let at_28 =
            if skip_special || !has_collectibles { 0 } else { ordinary - below_cap - at_25 };
        let ordinary_and_collectible =
            self.worksheet_ordinary_cents(below_cap) + at_25 * 25 + at_28 * 28;
        dollars(phase::cap(
            self.worksheet_ordinary_cents(income),
            [ordinary_and_collectible + middle * 15 + (gain - free - middle) * 20],
        ))
    }
}

// Schedule D source pools after an investment election, before net-gain caps.
// Foreign worksheets add their excess to the 1250 worksheet's loss line;
// first consume the remaining positive 28% pool, then reduce the 25% pool.
#[derive(Clone, Copy)]
pub(crate) struct Preference {
    pub dividends: i64,
    pub gain: i64,
    pub line18: i64,
    pub line19: i64,
}
impl Preference {
    pub(crate) fn foreign(mut self, taxable: i64) -> Self {
        let excess = positive(self.dividends + self.gain - positive(taxable));
        self.dividends = positive(self.dividends - positive(excess - self.gain));
        self.gain = positive(self.gain - excess);
        self.line19 = positive(self.line19 - positive(excess - self.line18));
        self.line18 = positive(self.line18 - excess);
        self
    }
    pub(crate) fn capped(self) -> [i64; 3] {
        let unrecaptured = phase::cap(self.line19, [self.gain]);
        let collectibles = phase::cap(self.line18, [self.gain - unrecaptured]);
        [self.dividends + self.gain, collectibles, unrecaptured]
    }
}

// Reusable regular-income-tax worksheet boundary. Callers supply taxable
// income and resolved gain character, not a mutated return or recomputed AGI.
// Credits, AMT, Schedule J and Form 8615 remain their existing separate stages.
#[derive(Clone, Copy)]
pub(crate) struct RegularTaxWorksheet {
    pub taxable: i64,
    pub preference: Preference,
    pub excluded: i64,
    pub foreign_election: bool,
    pub status: FederalStatus,
}
impl RegularTaxWorksheet {
    // The same worksheet on taxable income reduced by `amount` (not below
    // zero). The preferential income is not itself reduced; `Preference::capped`
    // holds it to the remaining taxable income.
    pub(crate) fn less(self, amount: i64) -> Self {
        Self { taxable: positive(self.taxable - amount), ..self }
    }
    pub(crate) fn tax(self) -> i64 {
        // Form 1040 foreign-earned-income worksheet line 2c floors the
        // stacking amount, not the signed Form 2555 adjustment to income.
        let excluded = positive(self.excluded);
        let preference = if self.foreign_election {
            self.preference.foreign(self.taxable)
        } else {
            self.preference
        };
        let [preferential, collectibles, unrecaptured] = preference.capped();
        // The current-return ordinary, qualified-dividend, Schedule D and
        // Form 2555 worksheets all use the filing table below $100,000 for
        // their ordinary-tax lookups. Only these lookups change: their gain
        // allocation and foreign stacking remain distinct.
        positive(
            crate::federal_rules::preferential_tax_with_special_gains(
                positive(self.taxable) + excluded,
                preferential,
                collectibles,
                unrecaptured,
                self.status,
            ) - crate::federal_rules::preferential_tax(excluded, 0, self.status),
        )
    }
}

// Preserve the established Form 8863/8880/8812/8839 credit order while
// allowing a worksheet to reuse unchanged eligibility/phaseout amounts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PersonalCreditPotentials {
    pub care: i64,
    pub elderly: i64,
    pub education: i64,
    pub saver: i64,
    pub adoption: i64,
    pub child: i64,
    pub actc_limit: i64,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LimitedPersonalCredits {
    pub care: i64,
    pub elderly: i64,
    pub education: i64,
    pub saver: i64,
    pub adoption: i64,
    pub child: i64,
    pub actc: i64,
}
impl LimitedPersonalCredits {
    pub(crate) fn nonrefundable_total(&self) -> i64 {
        self.care + self.elderly + self.education + self.saver + self.adoption + self.child
    }
}
impl PersonalCreditPotentials {
    pub(crate) fn limit(self, tax: i64) -> LimitedPersonalCredits {
        let mut remaining = positive(tax);
        let mut take = |potential| {
            let used = phase::cap(remaining, [positive(potential)]);
            remaining -= used;
            used
        };
        let care = take(self.care);
        let elderly = take(self.elderly);
        let education = take(self.education);
        let saver = take(self.saver);
        let reserved_child = positive(self.child - self.actc_limit);
        let adoption = phase::cap(self.adoption, [positive(remaining - reserved_child)]);
        let child = phase::cap(remaining - adoption, [self.child]);
        let actc = phase::cap(positive(self.child - child), [self.actc_limit]);
        LimitedPersonalCredits { care, elderly, education, saver, adoption, child, actc }
    }
}

// Form 6251 Parts I/II parameters. 2023–2025 published forms; 2026 follows
// section 55 as amended by P.L.119-21 and Rev. Proc.2025-32.
pub(crate) struct AmtRates {
    exemption: i64,
    phaseout: i64,
    divisor: i64,
    separate: bool,
    pub(crate) bracket: i64,
}
impl AmtRates {
    pub(crate) fn for_year(year: u32, status: FederalStatus) -> Option<Self> {
        let separate = status == FederalStatus::Separate;
        let joint = matches!(status, FederalStatus::Joint | FederalStatus::Surviving);
        let (single_exemption, joint_exemption, single_phaseout, bracket, divisor) = match year {
            2022 => (75900, 118100, 539900, 206100, 4),
            2023 => (81300, 126500, 578150, 220700, 4),
            2024 => (85700, 133300, 609350, 232600, 4),
            2025 => (88100, 137000, 626350, 239100, 4),
            2026 => (90100, 140200, 500000, 244500, 2),
            _ => return None,
        };
        Some(Self {
            exemption: if joint {
                joint_exemption
            } else if separate {
                crate::kernel::ratio::trunc_i64(joint_exemption, 2)
            } else {
                single_exemption
            },
            phaseout: single_phaseout * if joint { 2 } else { 1 },
            divisor,
            separate,
            bracket: crate::kernel::ratio::trunc_i64(bracket, if separate { 2 } else { 1 }),
        })
    }

    pub(crate) fn income(&self, base: i64) -> i64 {
        use crate::kernel::ratio::fraction;
        if self.separate {
            base + phase::cap(
                self.exemption,
                [fraction(
                    positive(base - self.phaseout - self.exemption * self.divisor),
                    1,
                    self.divisor,
                )],
            )
        } else {
            base
        }
    }

    // Runtime Form6251/8801 projection of this same year's exemption owner:
    // income, starting exemption, phaseout threshold, excess, reduction,
    // remaining exemption and taxable AMT base. No stored amount override.
    pub(crate) fn exemption_lines(&self, base: i64) -> [i64; 7] {
        use crate::kernel::ratio::fraction;
        let base = self.income(base);
        let excess = positive(base - self.phaseout);
        let reduction = fraction(excess, 1, self.divisor);
        let exemption = phase::phase_out(self.exemption, reduction);
        [
            base,
            self.exemption,
            self.phaseout,
            excess,
            reduction,
            exemption,
            positive(base - exemption),
        ]
    }

    pub(crate) fn taxable(&self, base: i64) -> i64 {
        self.exemption_lines(base)[6]
    }

    pub(crate) fn ordinary_tax(&self, base: i64) -> i64 {
        let taxable = self.taxable(base);
        dollars(self.ordinary_cents(taxable))
    }

    pub(crate) fn ordinary_cents(&self, taxable: i64) -> i64 {
        phase::cap(taxable, [self.bracket]) * 26 + positive(taxable - self.bracket) * 28
    }
}

// Form 6251 preferential-rate worksheet, shared across supported tax years.
// Inputs are resolved source pools and whole-dollar regular/AMT taxable bases.
pub(crate) struct AmtTaxWorksheet {
    pub year: u32,
    pub status: FederalStatus,
    pub base: i64,
    pub preferential: i64,
    pub regular_ordinary: i64,
    pub excluded: i64,
    pub special: [i64; 2],
    pub foreign: Option<Preference>,
}
// Runtime-only source trace. Income and rate components retain their original
// cents; the final tax still rounds once before foreign stacking is removed.
pub(crate) struct AmtTaxResult {
    pub tax: i64,
    pub part_three: alloc::vec::Vec<(u8, i64)>,
    pub foreign: Option<[i64; 6]>,
}
pub(crate) mod projection;
impl AmtTaxWorksheet {
    pub(crate) fn tax(self) -> i64 {
        self.calculate().tax
    }

    pub(crate) fn calculate(self) -> AmtTaxResult {
        let Self { year, status, base, preferential, regular_ordinary, excluded, special, foreign } =
            self;
        // Form 6251 foreign-earned-income worksheet line 2c uses the same floor.
        let excluded = positive(excluded);
        let [mut collectibles, mut unrecaptured] = special;
        let mut preferential = preferential;
        let amt_rates = AmtRates::for_year(year, status).unwrap();
        if preferential == 0 && special == [0, 0] && excluded == 0 && foreign.is_none() {
            return AmtTaxResult {
                tax: amt_rates.ordinary_tax(base),
                part_three: alloc::vec::Vec::new(),
                foreign: None,
            };
        }
        // Section 55(d)(4)(A)(iii) disapplies the section 59(j) child cap.
        // P.L. 119-21 section 70107 removes the 2026 sunset from all of (A),
        // including (iii); Rev. Proc. 2025-32 section 4.11 conflicts with it.
        let taxable = amt_rates.taxable(base);
        if taxable == 0 {
            return AmtTaxResult { tax: 0, part_three: alloc::vec::Vec::new(), foreign: None };
        }
        let income = taxable + excluded;
        if let Some(source) = foreign {
            let [total, at28, at25] = source.foreign(taxable).capped();
            collectibles = at28;
            unrecaptured = at25;
            preferential = total - at28 - at25;
        } else if excluded > 0 && collectibles == 0 && unrecaptured == 0 {
            preferential = phase::cap(preferential, [taxable]);
        }
        let gain = phase::cap(income, [positive(preferential)]);
        let rate = |value| amt_rates.ordinary_cents(value);
        let rates = Rates::for_year(year, status).unwrap();
        let (zero, fifteen) = rates.capital;
        // Form 6251 Part III uses REGULAR ordinary taxable income for the capital-gain
        // band occupancy, not the AMT ordinary base.
        let zero_room = positive(zero - regular_ordinary);
        let free = phase::cap(gain, [zero_room]);
        let regular_fifteen = phase::cap(
            regular_ordinary,
            [phase::floor(regular_ordinary - collectibles - unrecaptured, rates.ordinary[3])],
        );
        let fifteen_room = positive(fifteen - regular_fifteen - zero_room);
        let middle = phase::cap(gain - free, [fifteen_room]);
        // Form 6251 lines 15–18 exclude both normal gains and the 25% pool
        // from ordinary AMT rates. Lines 35–37 tax the remaining 1250 pool at 25%.
        let all_preferential = phase::cap(income, [positive(preferential) + unrecaptured]);
        let ordinary = income - all_preferential;
        let ordinary_tax = rate(ordinary);
        let fifteen_tax = middle * 15;
        let twenty_income = gain - free - middle;
        let twenty_tax = twenty_income * 20;
        let twenty_five_income = all_preferential - gain;
        let twenty_five_tax = twenty_five_income * 25;
        // D-2026-10-19-12: Part III prints whole dollars; line 38 adds the printed tax lines,
        // line 39 is the printed regular-rate tax and line 40 the smaller of the two.
        let (ordinary_tax, fifteen_tax, twenty_tax, twenty_five_tax) = (
            dollars(ordinary_tax),
            dollars(fifteen_tax),
            dollars(twenty_tax),
            dollars(twenty_five_tax),
        );
        let preferential_tax = ordinary_tax + fifteen_tax + twenty_tax + twenty_five_tax;
        let all_ordinary_tax = dollars(rate(income));
        let rounded = preferential_tax.min(all_ordinary_tax);
        let excluded_tax = dollars(rate(excluded));
        let tax = positive(rounded - excluded_tax);
        let mut part_three = alloc::vec::Vec::new();
        if preferential != 0 || collectibles != 0 || unrecaptured != 0 {
            part_three.extend([
                (12, income * 100),
                (13, preferential * 100),
                (14, unrecaptured * 100),
                (15, (positive(preferential) + unrecaptured) * 100),
                (16, all_preferential * 100),
                (17, ordinary * 100),
                (18, ordinary_tax * 100),
                (19, zero * 100),
                (20, regular_ordinary * 100),
                (21, zero_room * 100),
                (22, gain * 100),
                (23, free * 100),
                (24, (gain - free) * 100),
                (25, fifteen * 100),
                (26, zero_room * 100),
                (27, regular_fifteen * 100),
                (28, (zero_room + regular_fifteen) * 100),
                (29, fifteen_room * 100),
                (30, middle * 100),
                (31, fifteen_tax * 100),
                (32, (free + middle) * 100),
            ]);
            if free + middle != income {
                part_three.extend([(33, twenty_income * 100), (34, twenty_tax * 100)]);
                if unrecaptured != 0 {
                    part_three.extend([
                        (35, (ordinary + free + middle + twenty_income) * 100),
                        (36, twenty_five_income * 100),
                        (37, twenty_five_tax * 100),
                    ]);
                }
            }
            part_three.extend([
                (38, preferential_tax * 100),
                (39, all_ordinary_tax * 100),
                (40, rounded * 100),
            ]);
        }
        AmtTaxResult {
            tax,
            part_three,
            foreign: (excluded > 0).then_some([
                taxable * 100,
                excluded * 100,
                income * 100,
                rounded * 100,
                excluded_tax * 100,
                tax * 100,
            ]),
        }
    }
}
