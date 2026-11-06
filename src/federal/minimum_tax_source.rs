use crate::common::Owner;
use crate::filer_facts::FederalStatus;
use crate::foreign_credit_source::ForeignCreditCategory;
use crate::kernel::phase;
use crate::kernel::ratio::positive;
use crate::validation::ValidationIssue;
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub base: i64,
    /// Permanent exclusion adjustments from the prior return
    pub exclusions: Vec<MinimumTaxExclusion>,
    /// Prior Form 6251 line 10 regular-tax comparison
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// Prior Form 6251 line 11 AMT
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt: i64,
    /// Prior Form 8801 line 26 credit carried into the return being calculated
    /// Enter zero if none; this is separate from AMT paid for that prior year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carry: i64,
    /// Regular NOL deducted from prior Form6251 line1 (2025 line1b)
    /// Copy prior6251 line2e; this is added back once, never substituted for the exclusion-only loss below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub available: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MinimumTaxForeignEarned {
    /// Combined retained 2025 Form 2555 lines 45 and 50
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub excluded: i64,
    /// Related deductions not allowed, exclusion-only worksheet line 2b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub preferential: i64,
    /// Retained regular worksheet line 5; used for both 8801 35 and 42
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular_ordinary: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxScheduleDRateFacts {
    /// Retained AMT rate worksheet: 8801 28 after required modifications
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub preferential: i64,
    /// Retained Schedule D line 19, refigured as instructed: 8801 29
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Retained AMT Schedule D Tax Worksheet line 10; caps line 30
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gain_cap: i64,
    /// Regular worksheet amount used by 8801 35, not AMT ordinary income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular_zero: i64,
    /// Regular worksheet amount used by 8801 42
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        worldwide: i64,
    },
    /// Other category, section 960(c), boycott or missing refigured source worksheet
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MinimumTaxForeignCategory {
    /// Existing General, Passive or ForeignBranch category
    pub category: ForeignCreditCategory,
    /// Retained exclusion-only Form 1116 line 17; simplified election uses prior AMT 17
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    /// Same 2025 Form 1116 line 9 as regular tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Exclusion-only MTFTCE carryover on line 10, not regular/AMT FTC carry
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub carry: i64,
    /// Retained line 12 reductions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub reduction: i64,
    /// Retained line 13 tax reclassification; signed, linked categories must reconcile
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
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
impl MinimumTaxLoss {
    pub(crate) fn valid(&self) -> bool {
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
