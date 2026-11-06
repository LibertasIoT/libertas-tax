// Coverage: Form 8801 — Partial (retained-source implementation, 2026-10-10).
//   Part I, II and III numeric lines from identified prior 6251/exclusion facts,
//   exclusion-only MTCNOL carryovers/carrybacks, retained modified Form2555 rate
//   worksheets and MTFTCE1116 categories (general/passive/foreign branch).
//   Separate prior returns keep their actual status, then share one current limit.
//   Unavailable historical worksheets, former-joint attribution, nonresident
//   real-property floors and other foreign categories leave only this credit out.
//   Repayment history keeps its own prior source year; no historical reconstruction.
//   Evidence: src/tests/federal/minimum_tax.rs public-page derived arithmetic and failed-save
//   fixtures; src/tests/credits/credits.rs, repayment.rs; IRS2026 draft f8801/i8801,
//   retained2025 Form1116. Source/schema/final-unit acceptance remains separate.

//   F18: minimum-tax histories follow their actual return unit in normal
//   walk pruning. A departed spouse or same joint couple is removed, while
//   taxpayer Individual/FormerJoint facts retain their original unit and
//   existing computation/omission. Evidence: src/tests/cross/fix_f18.rs D4 public
//   Setup change, independent taxpayer retention and storage-owner cold loads.

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};
use crate::line_emission::emit as emit_lines;
pub use crate::minimum_tax_model::{
    MinimumTaxExclusion, MinimumTaxForeignCategory, MinimumTaxForeignCredit,
    MinimumTaxForeignEarned, MinimumTaxForeignGainSource, MinimumTaxGains, MinimumTaxHistories,
    MinimumTaxHistory, MinimumTaxHistoryUnit, MinimumTaxLoss, MinimumTaxLossAmount,
    MinimumTaxLossRecords, MinimumTaxQualifiedRateFacts, MinimumTaxRateFacts, MinimumTaxReturnForm,
    MinimumTaxScheduleDRateFacts, OwnedMinimumTaxHistory,
};
use crate::{FederalResult, FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

impl MinimumTaxGains {
    fn lines(&self, year: u32, taxable: i64, status: FederalStatus) -> Option<[i64; 29]> {
        use crate::kernel::ratio::fraction;
        let (zero, fifteen) = crate::income_tax::Rates::capital_limits(year, status)?;
        let rates = crate::income_tax::AmtRates::for_year(year, status)?;
        // Form8801 PartIII27..55. Each tax multiplication is rounded at its
        // printed line, then line53 adds those printed subtotals.
        let mut a = [0; 29];
        a[0] = taxable;
        a[1] = dollars(self.preferential);
        a[2] = dollars(self.combined - self.preferential);
        a[3] = dollars(self.combined);
        a[4] = phase::cap(a[0], [a[3]]);
        a[5] = a[0] - a[4];
        a[6] = dollars(rates.ordinary_cents(a[5]));
        a[7] = zero;
        a[8] = dollars(self.regular_zero);
        a[9] = positive(a[7] - a[8]);
        a[10] = phase::cap(a[0], [a[1]]);
        a[11] = phase::cap(a[9], [a[10]]);
        a[12] = a[10] - a[11];
        a[13] = fifteen;
        a[14] = a[9];
        a[15] = dollars(self.regular_fifteen);
        a[16] = a[14] + a[15];
        a[17] = positive(a[13] - a[16]);
        a[18] = phase::cap(a[12], [a[17]]);
        a[19] = fraction(a[18], 15, 100);
        a[20] = a[11] + a[18];
        if a[20] != a[0] {
            a[21] = a[10] - a[20];
            a[22] = fraction(a[21], 20, 100);
            if a[2] != 0 {
                a[23] = a[5] + a[20] + a[21];
                a[24] = a[0] - a[23];
                a[25] = fraction(a[24], 25, 100);
            }
        }
        a[26] = a[6] + a[19] + a[22] + a[25];
        a[27] = dollars(rates.ordinary_cents(a[0]));
        a[28] = phase::cap(a[26], [a[27]]);
        Some(a)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxHistoriesPage {
    /// Retained 2025 returns; one joint return or up to two separate returns
    // A retained history contains one joint return or at most the two current filers’ separate returns.
    #[libertas_size(min = 1, max = 2)]
    pub returns: Vec<OwnedMinimumTaxHistoryPage>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OwnedMinimumTaxHistoryPage {
    /// Actual prior return unit, not a reconstruction from current status
    pub unit: MinimumTaxHistoryUnitPage,
    pub facts: MinimumTaxHistory,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxHistoryUnitPage {
    /// Prior return filed separately by this current filer
    Individual {
        #[libertas_constrained_by("$.allowed_owners")]
        owner: Owner,
    },
    /// Prior joint return of the two people filing this current joint return
    SameJointCouple,
    /// Former joint return: credit attribution remains unsupported, named omission
    FormerJoint {
        #[libertas_constrained_by("$.allowed_owners")]
        claimant: Owner,
    },
}

impl From<MinimumTaxHistories> for MinimumTaxHistoriesPage {
    fn from(value: MinimumTaxHistories) -> Self {
        Self {
            returns: value
                .returns
                .into_iter()
                .map(|source| OwnedMinimumTaxHistoryPage {
                    unit: match source.unit {
                        MinimumTaxHistoryUnit::Individual { owner } => {
                            MinimumTaxHistoryUnitPage::Individual { owner }
                        }
                        MinimumTaxHistoryUnit::SameJointCouple => {
                            MinimumTaxHistoryUnitPage::SameJointCouple
                        }
                        MinimumTaxHistoryUnit::FormerJoint { claimant } => {
                            MinimumTaxHistoryUnitPage::FormerJoint { claimant }
                        }
                    },
                    facts: source.facts,
                })
                .collect(),
        }
    }
}
impl From<MinimumTaxHistoriesPage> for MinimumTaxHistories {
    fn from(value: MinimumTaxHistoriesPage) -> Self {
        Self {
            returns: value
                .returns
                .into_iter()
                .map(|source| OwnedMinimumTaxHistory {
                    unit: match source.unit {
                        MinimumTaxHistoryUnitPage::Individual { owner } => {
                            MinimumTaxHistoryUnit::Individual { owner }
                        }
                        MinimumTaxHistoryUnitPage::SameJointCouple => {
                            MinimumTaxHistoryUnit::SameJointCouple
                        }
                        MinimumTaxHistoryUnitPage::FormerJoint { claimant } => {
                            MinimumTaxHistoryUnit::FormerJoint { claimant }
                        }
                    },
                    facts: source.facts,
                })
                .collect(),
        }
    }
}

impl MinimumTaxHistory {
    // Resolve prior-year facts once. A hypothetical current-year tax limit must
    // never change this source balance or the actual return's carryforward.
    #[cfg(test)]
    pub(crate) fn balance(&self, status: FederalStatus) -> Option<CreditBalance> {
        self.balance_for_year(crate::year_constants::PRIOR_TAX_YEAR, status)
    }

    pub(crate) fn balance_for_year(
        &self,
        year: u32,
        status: FederalStatus,
    ) -> Option<CreditBalance> {
        let ready = self.rule_for_year(year).is_none()
            && !matches!(self.foreign_credit, MinimumTaxForeignCredit::Unavailable)
            && !self
                .foreign_earned
                .as_ref()
                .is_some_and(|v| matches!(v.gain_source, MinimumTaxForeignGainSource::Unavailable))
            && self.return_form == MinimumTaxReturnForm::Form1040
            && (self.status == FederalStatus::Joint) == (status == FederalStatus::Joint);
        if !ready {
            return None;
        }
        let excluded = self.exclusion_tax(year)?;
        // Form 8801 line 18 is signed: negative generation reduces old credit.
        let generated = dollars(self.amt) - excluded;
        Some(CreditBalance {
            excluded,
            generated,
            available: positive(generated + dollars(self.carry) + dollars(self.old_vehicle_credit)),
        })
    }

    fn worksheet(&self, year: u32) -> Option<ExclusionWorksheet> {
        if self.rule_for_year(year).is_some() || self.return_form != MinimumTaxReturnForm::Form1040
        {
            return None;
        }
        let rates = crate::income_tax::AmtRates::for_year(year, self.status)?;
        let mut a = [0; 15];
        a[0] = dollars(self.base + self.regular_nol);
        a[1] = dollars(self.exclusions.iter().map(|v| v.amount).sum());
        a[2] = -self.mtcnol.deduction(year)?;
        let [income, exemption, phaseout, excess, reduction, remaining, taxable] =
            rates.exemption_lines(positive(a[0] + a[1] + a[2]));
        a[3] = income;
        a[4] = exemption;
        a[5] = phaseout;
        a[6] = excess;
        a[7] = reduction;
        a[8] = remaining;
        a[9] = taxable;
        let mut gain_lines = None;
        let mut earned_lines = None;
        let mut forms = Vec::new();
        if taxable > 0 {
            let excluded = match &self.foreign_earned {
                Some(v) => {
                    if matches!(v.gain_source, MinimumTaxForeignGainSource::Unavailable) {
                        return None;
                    }
                    positive(dollars(v.excluded) - dollars(v.related_deductions))
                }
                None => 0,
            };
            let stacked = taxable + excluded;
            let tax = match &self.gains {
                Some(gains) => {
                    let mut lines = gains.normalized().lines(year, stacked, self.status)?;
                    if let MinimumTaxRateFacts::ScheduleD(v) = gains {
                        lines[2] = dollars(v.unrecaptured);
                        if lines[2] != 0 && lines[20] != lines[0] {
                            lines[23] = lines[5] + lines[20] + lines[21];
                            lines[24] = lines[0] - lines[23];
                            lines[25] = crate::kernel::ratio::fraction(lines[24], 25, 100);
                        }
                    }
                    let tax = lines[28];
                    gain_lines = Some(lines);
                    tax
                }
                None => dollars(rates.ordinary_cents(stacked)),
            };
            let excluded_tax = dollars(rates.ordinary_cents(excluded));
            a[10] = tax - excluded_tax;
            if let Some(v) = &self.foreign_earned {
                earned_lines = Some([
                    taxable,
                    dollars(v.excluded),
                    dollars(v.related_deductions),
                    excluded,
                    stacked,
                    tax,
                    excluded_tax,
                    a[10],
                ]);
            }
            forms = self.foreign_credit.forms(a[10])?;
            a[11] = self.foreign_credit.credit(a[10])?;
            a[12] = a[10] - a[11];
            a[13] = dollars(self.regular);
            a[14] = positive(a[12] - a[13]);
        }
        Some(ExclusionWorksheet {
            part_one: a,
            gains: gain_lines,
            foreign_earned: earned_lines,
            forms,
        })
    }

    fn exclusion_tax(&self, year: u32) -> Option<i64> {
        Some(self.worksheet(year)?.part_one[14])
    }
}

struct ExclusionWorksheet {
    part_one: [i64; 15],
    gains: Option<[i64; 29]>,
    foreign_earned: Option<[i64; 8]>,
    forms: Vec<crate::form_1116::Form1116>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CreditBalance {
    pub excluded: i64,
    pub generated: i64,
    pub available: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CreditUse {
    pub limit: i64,
    pub used: i64,
    pub carried: i64,
}

impl CreditBalance {
    // Both current-year tax inputs are explicit. Changing only regular tax is
    // not a substitute for deciding a worksheet's tentative-minimum-tax basis.
    pub(crate) fn limit(
        self,
        regular_after_credits: i64,
        tentative_after_credit: i64,
    ) -> CreditUse {
        let limit = positive(regular_after_credits - tentative_after_credit);
        let used = phase::cap(self.available, [limit]);
        CreditUse {
            limit,
            used,
            carried: crate::kernel::carry::closing_values::<crate::kernel::carry::Dollars>(
                self.available,
                used,
            )
            .expect("minimum-tax credit use is capped by available credit")
            .get(),
        }
    }
}

pub(crate) fn calculate(
    histories: Option<&MinimumTaxHistories>,
    status: FederalStatus,
    regular_after_credits: i64,
    tentative_after_credit: i64,
    r: &mut FederalResult,
) -> Option<CreditUse> {
    let histories = histories?;
    if histories.rule(status).is_some() {
        return None;
    }
    let mut balance = CreditBalance { excluded: 0, generated: 0, available: 0 };
    let mut computed = false;
    for (index, source) in histories.returns.iter().enumerate() {
        let unavailable = if source.facts.return_form == MinimumTaxReturnForm::Form1040Nr {
            Some("WARN_MTC_NONRESIDENT_LEFT_OUT")
        } else if matches!(source.unit, MinimumTaxHistoryUnit::FormerJoint { .. }) {
            Some("WARN_MTC_OWNERSHIP_LEFT_OUT")
        } else if matches!(source.facts.mtcnol, MinimumTaxLoss::Unavailable) {
            Some("WARN_MTC_NOL_LEFT_OUT")
        } else if source
            .facts
            .foreign_earned
            .as_ref()
            .is_some_and(|v| matches!(v.gain_source, MinimumTaxForeignGainSource::Unavailable))
        {
            Some("WARN_MTC_FOREIGN_EARNED_LEFT_OUT")
        } else if matches!(source.facts.foreign_credit, MinimumTaxForeignCredit::Unavailable) {
            Some("WARN_MTC_FOREIGN_CREDIT_LEFT_OUT")
        } else {
            None
        };
        if let Some(key) = unavailable {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(key, alloc::vec![]),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            );
            continue;
        }
        let worksheet = source.facts.worksheet(crate::year_constants::PRIOR_TAX_YEAR)?;
        worksheet.report(&source.facts, index, r);
        computed = true;
        let next = source
            .facts
            .balance_for_year(crate::year_constants::PRIOR_TAX_YEAR, source.facts.status)?;
        balance.excluded += next.excluded;
        balance.generated += next.generated;
        balance.available += next.available;
    }
    if !computed {
        return None;
    }
    if balance.available > 0 {
        emit_lines(
            r,
            &mut (),
            [
                (22, "minimum_tax_current_regular_liability", regular_after_credits),
                (23, "minimum_tax_current_tentative", tentative_after_credit),
            ],
            |(number, id, amount), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(
                        crate::TaxForm::Form8801,
                        &alloc::format!("{number}"),
                    )),
                    id,
                    LineValue::MoneyDollars(amount),
                    &["regular_income_tax", "alternative_minimum_tax"],
                    "https://www.irs.gov/pub/irs-dft/f8801--dft.pdf",
                    crate::Message::new("EXPLAIN_MTC_CURRENT_LIMIT", alloc::vec![]),
                ))
            },
        );
    }
    let credit = balance.limit(regular_after_credits, tentative_after_credit);
    emit_lines(
        r,
        &mut (),
        [
            (
                "minimum_tax_exclusion_tax",
                balance.excluded,
                msg!("EXPLAIN_MINIMUM_TAX_EXCLUSION_TAX"),
            ),
            (
                "minimum_tax_credit_generated",
                balance.generated,
                msg!("EXPLAIN_MINIMUM_TAX_CREDIT_GENERATED"),
            ),
            (
                "minimum_tax_credit_available",
                balance.available,
                msg!("EXPLAIN_MINIMUM_TAX_CREDIT_AVAILABLE"),
            ),
            ("minimum_tax_credit_limit", credit.limit, msg!("EXPLAIN_MINIMUM_TAX_CREDIT_LIMIT")),
            ("minimum_tax_credit", credit.used, msg!("EXPLAIN_MINIMUM_TAX_CREDIT")),
            (
                "minimum_tax_credit_carryforward",
                credit.carried,
                msg!("EXPLAIN_MINIMUM_TAX_CREDIT_CARRYFORWARD"),
            ),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["credits.minimum_tax", "regular_income_tax", "alternative_minimum_tax"],
                "https://www.irs.gov/instructions/i8801",
                explanation,
            ))
        },
    );
    if balance.available > 0 {
        for (id, number) in [
            ("minimum_tax_credit_limit", "24"),
            ("minimum_tax_credit", "25"),
            ("minimum_tax_credit_carryforward", "26"),
        ] {
            if let Some(row) = r.lines.iter_mut().find(|row| row.id == id) {
                row.form_line = Some(crate::FormLine::new(crate::TaxForm::Form8801, number));
            }
        }
    }
    Some(credit)
}

impl MinimumTaxLoss {
    fn deduction(&self, year: u32) -> Option<i64> {
        match self {
            Self::None => Some(0),
            Self::Retained { records }
                if self.valid() && records.amounts.iter().all(|v| v.year != year) =>
            {
                // i8801 line3: retained exclusion-only carryovers/carrybacks to
                // this year, without regular-NOL80% or ATNOLD90% substitution.
                Some(dollars(records.amounts.iter().map(|v| v.available).sum()))
            }
            _ => None,
        }
    }
}

impl MinimumTaxForeignCredit {
    fn forms(&self, tentative: i64) -> Option<Vec<crate::form_1116::Form1116>> {
        match self {
            Self::None | Self::PriorStatementElection { .. } => Some(Vec::new()),
            Self::Unavailable => None,
            Self::RetainedCategories { categories, worldwide } => Some(
                categories
                    .iter()
                    .map(|v| {
                        // Retained2025 MTFTCE line17 and18 already include source
                        // allocation, loss accounts and required capital adjustments.
                        // Reuse the existing1116 exact-ratio/credit engine below.
                        crate::form_1116::Form1116 {
                            slot: match v.category {
                                crate::ForeignCreditCategory::General => 0,
                                crate::ForeignCreditCategory::Passive => 1,
                                crate::ForeignCreditCategory::ForeignBranch => 2,
                            },
                            amt: true,
                            columns: alloc::vec![crate::form_1116::Column {
                                line_1a: dollars(v.income),
                                ..Default::default()
                            }],
                            taxes: alloc::vec![("Retained MTFTCE".into(), dollars(v.paid))],
                            line_10: dollars(v.carry),
                            line_12: -dollars(v.reduction),
                            line_13: dollars(v.reclassified),
                            line_16: 0,
                            line_18: dollars(*worldwide),
                            line_20: tentative,
                            line_22: 0,
                        }
                    })
                    .collect(),
            ),
        }
    }
    fn credit(&self, tentative: i64) -> Option<i64> {
        match self {
            Self::PriorStatementElection { credit } => Some(dollars(*credit)),
            _ => Some(phase::cap(
                self.forms(tentative)?.iter().map(|f| f.line_24()).sum::<i64>(),
                [tentative],
            )),
        }
    }
}

impl ExclusionWorksheet {
    fn report(&self, source: &MinimumTaxHistory, index: usize, r: &mut FederalResult) {
        use crate::line_emission::{form_line, form_ratio};
        use crate::{FormLine, TaxForm};
        let explain = || crate::Message::new("EXPLAIN_MTC_RETAINED_WORKSHEET", alloc::vec![]);
        let a = self.part_one;
        for (offset, value) in a.into_iter().enumerate() {
            let number = offset + 1;
            if (5..=14).contains(&number) && a[3] == 0 || (11..=14).contains(&number) && a[9] == 0 {
                continue;
            }
            form_line(
                r,
                FormLine::new(TaxForm::Form8801, &alloc::format!("{number}")),
                &alloc::format!("minimum_tax_history_{index}_line_{number}"),
                value,
                &["credits.minimum_tax"],
                "https://www.irs.gov/pub/irs-dft/f8801--dft.pdf",
                explain(),
            );
        }
        let generated = dollars(source.amt) - a[14];
        emit_lines(
            r,
            &mut (),
            [
                (16, dollars(source.amt)),
                (17, a[14]),
                (18, generated),
                (19, dollars(source.carry)),
                (20, dollars(source.old_vehicle_credit)),
                (21, generated + dollars(source.carry) + dollars(source.old_vehicle_credit)),
            ],
            |(number, value), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form8801, &alloc::format!("{number}"))),
                    &alloc::format!("minimum_tax_history_{index}_line_{number}"),
                    LineValue::MoneyDollars(value),
                    &["credits.minimum_tax"],
                    "https://www.irs.gov/pub/irs-dft/f8801--dft.pdf",
                    explain(),
                ))
            },
        );
        if let Some(lines) = &self.gains {
            for (offset, value) in lines.iter().enumerate() {
                let number = offset + 27;
                if number == 29
                    && matches!(source.gains, Some(MinimumTaxRateFacts::QualifiedDividends(_)))
                    || (48..=52).contains(&number) && lines[20] == lines[0]
                    || (50..=52).contains(&number) && lines[2] == 0
                {
                    continue;
                }
                form_line(
                    r,
                    FormLine::new(TaxForm::Form8801, &alloc::format!("{number}")),
                    &alloc::format!("minimum_tax_history_{index}_line_{number}"),
                    *value,
                    &["credits.minimum_tax"],
                    "https://www.irs.gov/pub/irs-dft/f8801--dft.pdf",
                    explain(),
                );
            }
        }
        if let Some(lines) = &self.foreign_earned {
            emit_lines(
                r,
                &mut (),
                [
                    "taxable",
                    "excluded",
                    "related",
                    "net_excluded",
                    "stacked",
                    "stacked_tax",
                    "excluded_tax",
                    "tax",
                ]
                .into_iter()
                .zip(lines),
                |(name, value), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("minimum_tax_history_{index}_foreign_earned_{name}"),
                        LineValue::MoneyDollars(*value),
                        &["credits.minimum_tax.foreign_earned"],
                        "https://www.irs.gov/pub/irs-dft/i8801--dft.pdf",
                        explain(),
                    ))
                },
            );
        }
        for form in &self.forms {
            for (number, value) in [
                (9, form.line_8()),
                (10, form.line_10),
                (11, form.line_11()),
                (12, -form.line_12),
                (13, form.line_13),
                (14, form.line_14()),
                (17, form.line_17()),
                (18, form.line_18),
                (19, form.line_19()),
                (20, form.line_20),
                (21, form.line_21()),
                (22, form.line_22),
                (23, form.line_23()),
                (24, form.line_24()),
            ] {
                let id = alloc::format!(
                    "minimum_tax_history_{index}_foreign_{}_line_{number}",
                    form.slot
                );
                let printed = FormLine::new(TaxForm::Form1116, &alloc::format!("{number}"));
                if number == 19 {
                    form_ratio(
                        r,
                        printed,
                        &id,
                        value,
                        &["credits.minimum_tax.foreign_credit"],
                        "https://www.irs.gov/pub/irs-prior/f1116--2025.pdf",
                        explain(),
                    );
                } else {
                    form_line(
                        r,
                        printed,
                        &id,
                        value,
                        &["credits.minimum_tax.foreign_credit"],
                        "https://www.irs.gov/pub/irs-prior/f1116--2025.pdf",
                        explain(),
                    );
                }
            }
        }
        if !self.forms.is_empty() {
            let category = |slot| {
                self.forms.iter().filter(|f| f.slot == slot).map(|f| f.line_24()).sum::<i64>()
            };
            let total = self.forms.iter().map(|f| f.line_24()).sum::<i64>();
            emit_lines(
                r,
                &mut (),
                [
                    (26, category(2)),
                    (27, category(1)),
                    (28, category(0)),
                    (29, 0),
                    (30, 0),
                    (31, 0),
                    (32, total),
                    (33, phase::cap(total, [a[10]])),
                    (34, 0),
                    (35, phase::cap(total, [a[10]])),
                ],
                |(number, value), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(TaxForm::Form1116, &alloc::format!("{number}"))),
                        &alloc::format!("minimum_tax_history_{index}_foreign_total_line_{number}"),
                        LineValue::MoneyDollars(value),
                        &["credits.minimum_tax.foreign_credit"],
                        "https://www.irs.gov/pub/irs-prior/f1116--2025.pdf",
                        explain(),
                    ))
                },
            );
        }
    }
}
