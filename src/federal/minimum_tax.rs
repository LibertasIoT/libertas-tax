use crate::validation::ValidationIssue;
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

use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};
use crate::{FederalResult, FederalStatus, Owner};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior-year dividend and capital-gain rate facts
/// Copy these four amounts from retained prior Form 6251 Part III and its supporting worksheets, without Form 2555. They are prior-return facts, not a newly calculated credit. Retain the separate regular and AMT worksheets when basis or gain treatment differed. If a required prior worksheet was never completed or its amounts are unresolved, do not confirm the history as complete.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxGains {
    /// Prior Form 6251 line 13: dividends and gains eligible for 0%, 15% or 20%
    /// From the prior Qualified Dividends and Capital Gain Tax Worksheet line 4 or Schedule D Tax Worksheet line 13, using the AMT worksheet when required.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub preferential: i64,
    /// Prior Form 6251 line 15: combined preferential and eligible 25% gain pool
    /// This equals line 13 for the simple qualified-dividend worksheet. For Schedule D, use the retained line 15 after its line 10 cap; do not simply add all unrecaptured section 1250 gain.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub combined: i64,
    /// Prior Form 6251 line 20: regular-tax income occupying the zero-rate band
    /// From the REGULAR Qualified Dividends and Capital Gain Tax Worksheet line 5 or Schedule D Tax Worksheet line 14. Use the form's zero-or-less taxable-income rule when applicable, never the AMT ordinary-income amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular_zero: i64,
    /// Prior Form 6251 line 27: regular-tax income occupying the 15% band
    /// From the REGULAR Qualified Dividends and Capital Gain Tax Worksheet line 5 or Schedule D Tax Worksheet line 21. Special-rate gains can make this differ from line 20.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular_fifteen: i64,
}

impl MinimumTaxGains {
    fn valid(&self) -> bool {
        [self.preferential, self.combined, self.regular_zero, self.regular_fifteen]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
            && self.combined >= self.preferential
            && self.regular_fifteen <= self.regular_zero
    }

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

/// Prior-year permanent AMT adjustment
/// Copy an exclusion item from the retained prior return: Form 6251 lines 2a, 2b, 2c, 2d, 2g or 2h, Schedule K-1 (Form 1041) box 12 code J, or an identified exclusion component on another line. Include the standard deduction when applicable. Do not include ISO, depreciation or other deferral adjustments, or the entire estate/trust adjustment from line 2j.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxExclusion {
    /// Source form, line and description
    #[libertas_size(min = 1)]
    pub source: String,
    /// Signed adjustment
    /// Use a negative amount for Form 6251 line 2b tax refunds and other subtractive adjustments. Enter each source once.
    #[libertas_money("USD", 2)]
    pub amount: i64,
}

impl MinimumTaxExclusion {
    pub(crate) fn valid(&self) -> bool {
        !self.source.trim().is_empty() && (-100000000000..=100000000000).contains(&self.amount)
    }
}

/// Minimum-tax credit history
/// Supply retained prior-return facts, not a credit you calculated. For the current 2026 return use 2025 records; within a historical repayment return use records from the immediately preceding year. The App separates permanent exclusions from deferrals, applies any retained dividend/capital-gain rate facts, limits the current credit and reports the unused balance. Use identified exclusion-only loss and foreign worksheets when applicable; an unavailable source leaves only this credit unclaimed.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxHistory {
    /// Form filed for this retained return
    pub return_form: MinimumTaxReturnForm,
    /// Prior filing status
    pub status: FederalStatus,
    /// Prior Form 6251 line 1 (line 1b for 2025)
    /// Preserve a negative value. This is the starting income before AMT adjustments, not line 4 AMTI or the amount after the AMT exemption.
    #[libertas_money("USD", 2)]
    pub base: i64,
    /// Permanent exclusion adjustments from the prior return
    pub exclusions: Vec<MinimumTaxExclusion>,
    /// Prior Form 6251 line 10 regular-tax comparison
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// Prior Form 6251 line 11 AMT
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt: i64,
    /// Prior Form 8801 line 26 credit carried into the return being calculated
    /// Enter zero if none; this is separate from AMT paid for that prior year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub carry: i64,
    /// Regular NOL deducted from prior Form6251 line1 (2025 line1b)
    /// Copy prior6251 line2e; this is added back once, never substituted for the exclusion-only loss below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub regular_nol: i64,
    /// Exclusion-only minimum-tax-credit loss carryovers/carrybacks to the retained year
    pub mtcnol: MinimumTaxLoss,
    /// Foreign-earned-income treatment in the retained year
    /// Use the retained year's Form 2555 and related minimum-tax worksheet facts.
    /// [NullText]
    /// The retained year has no Form 2555 foreign-earned-income treatment
    /// [SetText]
    /// The retained year's foreign-earned-income facts are
    pub foreign_earned: Option<MinimumTaxForeignEarned>,
    /// Prior foreign credit on exclusion items, separate from regular/AMT FTC
    pub foreign_credit: MinimumTaxForeignCredit,
    /// Retained prior preferential-rate worksheets; modified worksheets when Form2555 applies
    /// [NullText]
    /// The retained year needs no preferential-rate worksheet
    /// [SetText]
    /// The retained preferential-rate worksheet facts are
    pub gains: Option<MinimumTaxRateFacts>,
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    /// Prior unallowed pre-ARRA qualified electric vehicle credit (old section30)
    pub old_vehicle_credit: i64,
}

/// Retained prior return form
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MinimumTaxReturnForm {
    /// Form 1040 or 1040-SR
    Form1040,
    /// Form 1040-NR
    /// The App does not calculate the prior nonresident real-property-gain floor. This leaves out only the minimum-tax credit; current income and taxes remain calculated.
    Form1040Nr,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxHistories {
    /// Retained 2025 returns; one joint return or up to two separate returns
    // A retained history contains one joint return or at most the two current filers’ separate returns.
    #[libertas_size(min = 1, max = 2)]
    pub returns: Vec<OwnedMinimumTaxHistory>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OwnedMinimumTaxHistory {
    /// Actual prior return unit, not a reconstruction from current status
    pub unit: MinimumTaxHistoryUnit,
    pub facts: MinimumTaxHistory,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxHistoryUnit {
    /// Prior return filed separately by this current filer
    Individual { owner: Owner },
    /// Prior joint return of the two people filing this current joint return
    SameJointCouple,
    /// Former joint return: credit attribution remains unsupported, named omission
    FormerJoint { claimant: Owner },
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

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxLoss {
    None,
    /// Amounts from the separate retained exclusion-only MTCNOL worksheets
    Retained {
        records: MinimumTaxLossRecords,
    },
    /// A source history exists but its exclusion-only worksheet is unavailable
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxLossRecords {
    #[libertas_size(min = 1)]
    pub amounts: Vec<MinimumTaxLossAmount>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxLossAmount {
    /// Original loss year; a carryback must be a different year from 2025
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Reconciled carryover/carryback available to deduct on 2025 8801 line 3
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub available: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxForeignEarned {
    /// Combined retained 2025 Form 2555 lines 45 and 50
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub excluded: i64,
    /// Related deductions not allowed, exclusion-only worksheet line 2b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub related_deductions: i64,
    /// Gain facts come from the modified retained worksheets described below
    pub gain_source: MinimumTaxForeignGainSource,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxForeignGainSource {
    /// No gains/dividends requiring Part III
    NoPreferentialIncome,
    /// Required modified regular and AMT rate worksheets are retained
    RetainedModifiedWorksheets,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxRateFacts {
    QualifiedDividends(MinimumTaxQualifiedRateFacts),
    ScheduleD(MinimumTaxScheduleDRateFacts),
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxQualifiedRateFacts {
    /// Retained AMT qualified-dividend/gain worksheet line 4
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub preferential: i64,
    /// Retained regular worksheet line 5; used for both 8801 35 and 42
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular_ordinary: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxScheduleDRateFacts {
    /// Retained AMT rate worksheet: 8801 28 after required modifications
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub preferential: i64,
    /// Retained Schedule D line 19, refigured as instructed: 8801 29
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Retained AMT Schedule D Tax Worksheet line 10; caps line 30
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gain_cap: i64,
    /// Regular worksheet amount used by 8801 35, not AMT ordinary income
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular_zero: i64,
    /// Regular worksheet amount used by 8801 42
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular_fifteen: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinimumTaxForeignCredit {
    None,
    /// Actual prior-year statement-only election without Form 1116
    PriorStatementElection {
        #[libertas_money("USD", 2)]
        // Tax structure: The prior statement-only foreign-tax election permits at most $600 jointly, $300 separately.
        #[libertas_number(min = 0, max = 60000)]
        credit: i64,
    },
    /// Refigured exclusion-only 2025 MTFTCE Forms 1116 are retained
    RetainedCategories {
        // These are the three supported, unique foreign-tax categories.
        #[libertas_size(min = 1, max = 3)]
        categories: Vec<MinimumTaxForeignCategory>,
        /// Worldwide exclusion-only taxable income after required line-18 worksheet
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        worldwide: i64,
    },
    /// Other category, section 960(c), boycott or missing refigured source worksheet
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxForeignCategory {
    /// Existing General, Passive or ForeignBranch category
    pub category: crate::ForeignCreditCategory,
    /// Retained exclusion-only Form 1116 line 17; simplified election uses prior AMT 17
    #[libertas_money("USD", 2)]
    pub income: i64,
    /// Same 2025 Form 1116 line 9 as regular tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Exclusion-only MTFTCE carryover on line 10, not regular/AMT FTC carry
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub carry: i64,
    /// Retained line 12 reductions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub reduction: i64,
    /// Retained line 13 tax reclassification; signed, linked categories must reconcile
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub reclassified: i64,
}

impl MinimumTaxHistory {
    pub(crate) fn valid(&self) -> bool {
        let signed = |v| (-100000000000..=100000000000).contains(&v);
        signed(self.base)
            && self.gains.as_ref().is_none_or(MinimumTaxRateFacts::valid)
            && (self.foreign_earned.is_some()
                || self
                    .gains
                    .as_ref()
                    .is_none_or(|v| v.normalized().regular_zero <= positive(self.base)))
            && amount(self.regular_nol)
            && amount(self.old_vehicle_credit)
            && self.mtcnol.valid()
            && self.foreign_credit.valid(self.status)
            && self.foreign_earned.as_ref().is_none_or(|v| {
                amount(v.excluded)
                    && amount(v.related_deductions)
                    && match v.gain_source {
                        MinimumTaxForeignGainSource::NoPreferentialIncome => self.gains.is_none(),
                        MinimumTaxForeignGainSource::RetainedModifiedWorksheets => {
                            self.gains.is_some()
                        }
                        MinimumTaxForeignGainSource::Unavailable => true,
                    }
            })
            && [self.regular, self.amt, self.carry].iter().all(|v| (0..=100000000000).contains(v))
            && self.exclusions.len() <= 100
            && self.exclusions.iter().all(MinimumTaxExclusion::valid)
    }

    pub(crate) fn issue_for_year(&self, year: u32) -> Option<ValidationIssue> {
        if !self.foreign_credit.bounds_valid() {
            return Some(ValidationIssue::new("MTC_HISTORY_INVALID").at_field("foreign_credit"));
        }
        if self.gains.as_ref().is_some_and(|v| !v.bounds_valid()) {
            return Some(ValidationIssue::new("MTC_HISTORY_INVALID").at_field("gains"));
        }
        if let MinimumTaxLoss::Retained { records } = &self.mtcnol
            && let Some((i, _)) = records
                .amounts
                .iter()
                .enumerate()
                .find(|(i, v)| records.amounts[..*i].iter().any(|p| p.year == v.year))
        {
            return Some(
                ValidationIssue::new("MTC_NOL_YEAR_TWICE")
                    .at_field("year")
                    .in_list("amounts", i)
                    .in_field("records")
                    .in_field("mtcnol"),
            );
        }
        match &self.foreign_credit {
            MinimumTaxForeignCredit::PriorStatementElection { credit }
                if *credit > if self.status == FederalStatus::Joint { 60000 } else { 30000 } =>
            {
                return Some(
                    ValidationIssue::new("MTC_FTC_STATEMENT_LIMIT")
                        .at_field("credit")
                        .in_field("foreign_credit"),
                );
            }
            MinimumTaxForeignCredit::RetainedCategories { categories, .. } => {
                if let Some((i, _)) = categories
                    .iter()
                    .enumerate()
                    .find(|(i, v)| categories[..*i].iter().any(|p| p.category == v.category))
                {
                    return Some(
                        ValidationIssue::new("MTC_FTC_CATEGORY_TWICE")
                            .at_field("category")
                            .in_list("categories", i)
                            .in_field("foreign_credit"),
                    );
                }
                if categories.iter().map(|v| v.reclassified).sum::<i64>() != 0 {
                    return Some(
                        ValidationIssue::new("MTC_FTC_RECLASSIFICATION_UNBALANCED")
                            .at_field("categories")
                            .in_field("foreign_credit"),
                    );
                }
                if let Some((i, _)) =
                    categories.iter().enumerate().find(|(_, v)| v.reduction > v.paid + v.carry)
                {
                    return Some(
                        ValidationIssue::new("MTC_FTC_REDUCTION_ABOVE_TAX")
                            .at_field("reduction")
                            .in_list("categories", i)
                            .in_field("foreign_credit"),
                    );
                }
                if let Some((i, _)) = categories
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.paid + v.carry - v.reduction + v.reclassified < 0)
                {
                    return Some(
                        ValidationIssue::new("MTC_FTC_NEGATIVE_POOL")
                            .at_field("reclassified")
                            .in_list("categories", i)
                            .in_field("foreign_credit"),
                    );
                }
            }
            _ => {}
        }
        if let Some(v) = &self.foreign_earned
            && match v.gain_source {
                MinimumTaxForeignGainSource::NoPreferentialIncome => self.gains.is_some(),
                MinimumTaxForeignGainSource::RetainedModifiedWorksheets => self.gains.is_none(),
                MinimumTaxForeignGainSource::Unavailable => false,
            }
        {
            return Some(
                ValidationIssue::new("MTC_FOREIGN_GAIN_SOURCE")
                    .at_field("gain_source")
                    .in_field("foreign_earned"),
            );
        }
        if let Some(MinimumTaxRateFacts::ScheduleD(v)) = &self.gains {
            if v.regular_fifteen > v.regular_zero {
                return Some(
                    ValidationIssue::new("MTC_REGULAR_GAIN_BANDS")
                        .at_field("regular_fifteen")
                        .in_field("gains"),
                );
            }
            if v.gain_cap < v.preferential {
                return Some(
                    ValidationIssue::new("MTC_GAIN_CAP_BELOW_PREFERENTIAL")
                        .at_field("gain_cap")
                        .in_field("gains"),
                );
            }
        }
        if self.foreign_earned.is_none()
            && self
                .gains
                .as_ref()
                .is_some_and(|v| v.normalized().regular_zero > positive(self.base))
        {
            return Some(ValidationIssue::new("MTC_REGULAR_GAIN_BASE").at_field("gains"));
        }
        if !self.valid() {
            return Some(ValidationIssue::new("MTC_HISTORY_INVALID").at_record());
        }
        if let MinimumTaxLoss::Retained { records } = &self.mtcnol
            && let Some((i, _)) = records.amounts.iter().enumerate().find(|(_, v)| v.year == year)
        {
            return Some(
                ValidationIssue::new("MTC_NOL_SAME_YEAR")
                    .at_field("year")
                    .in_list("amounts", i)
                    .in_field("records")
                    .in_field("mtcnol"),
            );
        }
        None
    }
    pub(crate) fn rule_for_year(&self, year: u32) -> Option<&'static str> {
        self.issue_for_year(year).map(|issue| issue.key)
    }

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

fn amount(v: i64) -> bool {
    (0..=100000000000).contains(&v)
}
fn signed_amount(v: i64) -> bool {
    (-100000000000..=100000000000).contains(&v)
}

impl MinimumTaxRateFacts {
    pub(crate) fn normalized(&self) -> MinimumTaxGains {
        match self {
            Self::QualifiedDividends(v) => MinimumTaxGains {
                preferential: v.preferential,
                combined: v.preferential,
                regular_zero: v.regular_ordinary,
                regular_fifteen: v.regular_ordinary,
            },
            Self::ScheduleD(v) => MinimumTaxGains {
                preferential: v.preferential,
                combined: phase::cap(v.preferential + v.unrecaptured, [v.gain_cap]),
                regular_zero: v.regular_zero,
                regular_fifteen: v.regular_fifteen,
            },
        }
    }
    fn bounds_valid(&self) -> bool {
        let values = match self {
            Self::QualifiedDividends(v) => alloc::vec![v.preferential, v.regular_ordinary],
            Self::ScheduleD(v) => alloc::vec![
                v.preferential,
                v.unrecaptured,
                v.gain_cap,
                v.regular_zero,
                v.regular_fifteen
            ],
        };
        values.into_iter().all(amount)
    }
    fn valid(&self) -> bool {
        self.bounds_valid() && self.normalized().valid()
    }
}

// This adapts an already-computed historical rate worksheet for §1341 source
// propagation. It does not derive retained source facts from a credit amount.
impl From<MinimumTaxGains> for MinimumTaxRateFacts {
    fn from(v: MinimumTaxGains) -> Self {
        Self::ScheduleD(MinimumTaxScheduleDRateFacts {
            preferential: v.preferential,
            unrecaptured: v.combined - v.preferential,
            gain_cap: v.combined,
            regular_zero: v.regular_zero,
            regular_fifteen: v.regular_fifteen,
        })
    }
}

impl MinimumTaxLoss {
    fn valid(&self) -> bool {
        match self {
            Self::Retained { records } => {
                !records.amounts.is_empty()
                    && records.amounts.iter().enumerate().all(|(i, v)| {
                        (1900..=crate::year_constants::TAX_YEAR).contains(&v.year)
                            && amount(v.available)
                            && !records.amounts[..i].iter().any(|p| p.year == v.year)
                    })
            }
            Self::None | Self::Unavailable => true,
        }
    }
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
    fn bounds_valid(&self) -> bool {
        match self {
            Self::None | Self::Unavailable => true,
            Self::PriorStatementElection { credit } => amount(*credit),
            Self::RetainedCategories { categories, worldwide } => {
                amount(*worldwide)
                    && (1..=3).contains(&categories.len())
                    && categories.iter().all(|v| {
                        signed_amount(v.income)
                            && amount(v.paid)
                            && amount(v.carry)
                            && amount(v.reduction)
                            && signed_amount(v.reclassified)
                    })
            }
        }
    }
    fn valid(&self, status: FederalStatus) -> bool {
        if !self.bounds_valid() {
            return false;
        }
        match self {
            Self::None | Self::Unavailable => true,
            Self::PriorStatementElection { credit } => {
                (0..=if status == FederalStatus::Joint { 60000 } else { 30000 }).contains(credit)
            }
            Self::RetainedCategories { categories, worldwide } => {
                amount(*worldwide)
                    && !categories.is_empty()
                    && categories.len() <= 3
                    && categories.iter().map(|v| v.reclassified).sum::<i64>() == 0
                    && categories.iter().enumerate().all(|(i, v)| {
                        signed_amount(v.income)
                            && amount(v.paid)
                            && amount(v.carry)
                            && amount(v.reduction)
                            && signed_amount(v.reclassified)
                            && v.reduction <= v.paid + v.carry
                            && v.paid + v.carry - v.reduction + v.reclassified >= 0
                            && !categories[..i].iter().any(|p| p.category == v.category)
                    })
            }
        }
    }
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

impl MinimumTaxHistories {
    pub(crate) fn issue(&self, status: FederalStatus) -> Option<ValidationIssue> {
        if self.returns.is_empty() {
            return Some(ValidationIssue::new("MTC_PRIOR_RETURN_COUNT").at_field("returns"));
        }
        for (i, source) in self.returns.iter().enumerate() {
            if self.returns[..i].iter().any(|v| v.unit == source.unit) {
                return Some(
                    ValidationIssue::new("MTC_PRIOR_RETURN_TWICE")
                        .at_field("unit")
                        .in_list("returns", i),
                );
            }
            match source.unit {
                MinimumTaxHistoryUnit::SameJointCouple
                | MinimumTaxHistoryUnit::FormerJoint { .. }
                    if self.returns.len() != 1 =>
                {
                    return Some(
                        ValidationIssue::new("MTC_JOINT_AND_SEPARATE_HISTORY")
                            .at_field("unit")
                            .in_list("returns", i),
                    );
                }
                _ => {}
            }
            let source_joint = source.facts.status == FederalStatus::Joint;
            let unit_joint = !matches!(source.unit, MinimumTaxHistoryUnit::Individual { .. });
            if source_joint != unit_joint {
                return Some(
                    ValidationIssue::new("MTC_PRIOR_STATUS_UNIT")
                        .at_field("status")
                        .in_field("facts")
                        .in_list("returns", i),
                );
            }
            if status != FederalStatus::Joint
                && match source.unit {
                    MinimumTaxHistoryUnit::Individual { owner } => owner != Owner::Taxpayer,
                    MinimumTaxHistoryUnit::SameJointCouple => true,
                    MinimumTaxHistoryUnit::FormerJoint { claimant } => claimant != Owner::Taxpayer,
                }
            {
                return Some(
                    ValidationIssue::new("MTC_OWNER_NOT_ON_RETURN")
                        .at_field("unit")
                        .in_list("returns", i),
                );
            }
            if let Some(issue) = source.facts.issue_for_year(crate::year_constants::PRIOR_TAX_YEAR)
            {
                return Some(issue.in_field("facts").in_list("returns", i));
            }
        }
        None
    }
    pub(crate) fn rule(&self, status: FederalStatus) -> Option<&'static str> {
        self.issue(status).map(|issue| issue.key)
    }
    pub(crate) fn valid(&self) -> bool {
        // Full current-unit validation additionally uses actual return status.
        self.rule(FederalStatus::Joint).is_none()
    }
    pub(crate) fn single_current_unit(&self, status: FederalStatus) -> Option<&MinimumTaxHistory> {
        if self.rule(status).is_some() || self.returns.len() != 1 {
            return None;
        }
        let source = &self.returns[0];
        let same = match source.unit {
            MinimumTaxHistoryUnit::Individual { owner } => {
                owner == Owner::Taxpayer && status != FederalStatus::Joint
            }
            MinimumTaxHistoryUnit::SameJointCouple => status == FederalStatus::Joint,
            MinimumTaxHistoryUnit::FormerJoint { .. } => false,
        };
        same.then_some(&source.facts)
    }
    pub(crate) fn from_current_unit(facts: MinimumTaxHistory) -> Self {
        let unit = if facts.status == FederalStatus::Joint {
            MinimumTaxHistoryUnit::SameJointCouple
        } else {
            MinimumTaxHistoryUnit::Individual { owner: Owner::Taxpayer }
        };
        Self { returns: alloc::vec![OwnedMinimumTaxHistory { unit, facts }] }
    }
}

impl ExclusionWorksheet {
    fn report(&self, source: &MinimumTaxHistory, index: usize, r: &mut FederalResult) {
        use crate::kernel::lines::{form_line, form_ratio};
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
