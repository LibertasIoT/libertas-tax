// Section 1301; retained prior-return facts.
// https://www.irs.gov/instructions/i1040sj
// https://www.irs.gov/instructions/i6251 (line 10)
use crate::filer_facts::FederalStatus;
use crate::kernel::ratio::{dollars, positive};
use crate::validation::ValidationIssue;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Farm income averaging election
/// Leave absent to use ordinary current-year tax. This election changes income tax, not AGI, self-employment tax, QBI, carryovers or AMT. Compare the displayed taxes and adjust your election; the App does not automatically choose an optimal amount or file Schedule J.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmAveraging {
    /// Ordinary farm income you elect to average
    /// Choose all or part of eligible ordinary income. The App calculates the limit after farm expenses, attributable owner deductions and QBI, and caps it at current taxable income. This amount is your election, not a precomputed tax or deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub elected: i64,
    /// Retained 2023, 2024 and 2025 return records
    /// Include each year exactly once, using corrected/amended records when applicable. You need not have farmed in those years. Use the most recent prior Schedule J that includes each base year; do not reuse an earlier unadjusted return instead.
    // Tax structure: Schedule J averages over the three preceding base years.
    #[libertas_size(min = 3, max = 3)]
    pub years: Vec<AveragingBaseYear>,
    /// Selected current-source components and retained base-year scope
    pub selection: crate::FarmAveragingSelection,
}

/// Income-averaging base year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingBaseYear {
    /// Year
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2023, max = 2025)]
    pub year: u32,
    /// Filing status on that year's return
    pub status: FederalStatus,
    /// Retained taxable-income and tax source
    pub source: AveragingSource,
    /// Qualified dividends from that base year's retained tax worksheet
    /// Use retained Form 1040 line 3a or the corresponding prior Schedule J source amount BEFORE foreign-income capital-gain-excess modifications. This is a retained source amount, not a new averaging calculation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub dividends: i64,
    /// Retained capital-gain worksheet facts
    pub capital: AveragingCapital,
    /// Retained foreign-earned-income and housing exclusion records
    /// Leave absent when no Form 2555 applied in this base year. Earlier exclusions affect the rate on income allocated to that year; they do not become current-year exclusions.
    pub foreign: Option<AveragingForeign>,
    /// Retained investment-interest election
    /// Supply the original Form 4952 election records when present. The App adjusts preferential income without making a new base-year election or changing current-year deductions.
    pub investment: Option<AveragingInvestment>,
}

/// Retained Form 4952 election facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingInvestment {
    /// Form 4952 line 4g
    /// Total retained election to include qualified dividends and net investment capital gains in investment income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub elected: i64,
    /// Form 4952 line 4e or its smaller dotted-line election
    /// Copy line 4e unless the retained return elected a smaller amount on the dotted line beside it; then copy that smaller amount. Do not recompute the election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital: i64,
}

/// Base-year foreign-income worksheet facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingForeign {
    /// Retained Forms 2555
    /// One form per filer who used Form 2555 in this base year. Include both filers when applicable on a joint return.
    // Tax structure: this retained joint-return worksheet has at most one Form 2555 per spouse.
    #[libertas_size(min = 1, max = 2)]
    pub forms: Vec<AveragingForeignForm>,
    /// Retained Foreign Earned Income Tax Worksheet line 2b
    /// Related itemized deductions or exclusions that could not be claimed. Copy the retained amount; the App subtracts it from the Forms 2555 amounts and floors the stacking amount at zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub disallowed: i64,
}

/// Base-year Form 2555
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingForeignForm {
    /// Filer
    pub owner: crate::Owner,
    /// Form 2555 line 45
    /// Retained foreign earned income and housing exclusion amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exclusion: i64,
    /// Form 2555 line 50
    /// Retained housing deduction amount; do not repeat line 45 here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub housing: i64,
}
impl AveragingForeign {
    fn valid(&self) -> bool {
        (1..=2).contains(&self.forms.len())
            && amount(self.disallowed)
            && self.forms.iter().all(|v| amount(v.exclusion) && amount(v.housing))
    }
    pub(crate) fn stacking(&self) -> i64 {
        positive(
            dollars(self.forms.iter().map(|v| v.exclusion + v.housing).sum())
                - dollars(self.disallowed),
        )
    }
}

/// Base-year source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AveragingSource {
    /// Original or corrected base-year Form 1040
    /// Use this only when no Schedule J for this or a later year through 2025 adjusts this base year. Supply retained return amounts; the App derives negative taxable income instead of treating Form 1040's zero floor as the true amount.
    Return {
        /// Form 1040 line 11 adjusted gross income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        agi: i64,
        /// Form 1040 line 14 deductions
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductions: i64,
        /// Form 1040 line 16 tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tax: i64,
        /// Non-section-1 taxes included in line 16
        /// Enter each retained component separately, such as Form 8814, Form 4972 or education-credit recapture. Never enter total tax, self-employment tax or amounts already outside line 16.
        other_tax: Vec<AveragingOtherTax>,
        /// Negative taxable-income source records
        /// Required when AGI minus deductions is negative, including when the original Form 1040 showed zero. Leave absent otherwise.
        #[libertas_default($null)]
        loss: Option<AveragingLoss>,
    },
    /// Most recent prior Schedule J
    /// For base year 2023: use 2025 Schedule J lines 11/12, otherwise 2024 lines 15/16, otherwise 2023 lines 3/4. For base year 2024: use 2025 lines 15/16, otherwise 2024 lines 3/4. For base year 2025: use 2025 lines 3/4. Copy the paired taxable income and tax; do not enter Schedule J line 23 or a newly calculated adjustment.
    ScheduleJ {
        /// Year of the retained Schedule J
        // Tax structure: this retained source belongs to a year before the 2026 return year.
        #[libertas_number(min = 2023, max = 2025)]
        year: u32,
        /// Taxable income from the specified retained line
        /// Preserve a negative amount on line 11 or 15.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        taxable: i64,
        /// Tax from the paired retained line
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tax: i64,
    },
}

/// Other tax included in base-year Form 1040 line 16
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingOtherTax {
    /// Retained non-section-1 tax amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Negative base-year taxable-income records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingLoss {
    /// Schedule D line 21 capital loss deduction, as a positive amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital_deduction: i64,
    /// Schedule D line 16 net capital loss, as a positive amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital_loss: i64,
    /// Total short- and long-term capital loss carried to the following year
    /// From the retained capital-loss carryover worksheet, including any corrected NOL carryback treatment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital_carry: i64,
    /// Net operating loss history
    pub nol: AveragingNol,
}

/// Base-year NOL source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AveragingNol {
    /// No NOL or unused NOL carryover/carryback
    None,
    /// NOL generated in this base year
    Current {
        /// Retained Form 1045 Schedule A or Form 172 current-year NOL
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// NOL deduction from other years already included in AGI
        /// The App adds this back before deriving negative taxable income when this year itself generated an NOL.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_deduction: i64,
    },
    /// No current NOL; earlier/later-year NOL remained unused
    Unused {
        /// Retained NOL worksheet amount not used in this base year and carried forward
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

/// Base-year capital-gain source
/// From retained Schedule D or equivalent capital-distribution records; leave every amount at zero when that year had no positive net capital gain. Use retained amounts after any earlier Schedule J allocation, BEFORE foreign-income capital-gain-excess modifications and without applying the new election. For direct capital distributions without Schedule D, enter that reported amount in both gain fields and zero special-rate gains. Enter any retained Form 4952 election separately; do not subtract it from these source amounts.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AveragingCapital {
    /// Schedule D line 15 net long-term gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub long_term: i64,
    /// Schedule D line 16 net gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub net: i64,
    /// Schedule D line 18 collectibles rate gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Schedule D line 19 unrecaptured section 1250 gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
}
impl AveragingCapital {
    /// No positive net capital gain.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self { long_term: 0, net: 0, collectibles: 0, unrecaptured: 0 };
}

fn amount(v: i64) -> bool {
    (0..=100000000000).contains(&v)
}
impl FarmAveraging {
    pub(crate) fn valid(&self) -> bool {
        self.elected > 0
            && amount(self.elected)
            && self.selection.valid(self.elected)
            && self.years.len() == 3
            && self.years.iter().all(|b| {
                (2023..=2025).contains(&b.year)
                    && amount(b.dividends)
                    && b.investment.as_ref().is_none_or(|v| amount(v.elected) && amount(v.capital))
                    && b.foreign.as_ref().is_none_or(AveragingForeign::valid)
                    && [
                        b.capital.long_term,
                        b.capital.net,
                        b.capital.collectibles,
                        b.capital.unrecaptured,
                    ]
                    .iter()
                    .all(|v| amount(*v))
                    && match &b.source {
                        AveragingSource::ScheduleJ { year, taxable, tax } => {
                            (2023..=2025).contains(year)
                                && (-100000000000..=100000000000).contains(taxable)
                                && amount(*tax)
                        }
                        AveragingSource::Return { agi, deductions, tax, other_tax, loss } => {
                            (-100000000000..=100000000000).contains(agi)
                                && amount(*deductions)
                                && amount(*tax)
                                && other_tax.len() <= 20
                                && other_tax.iter().all(|v| amount(v.amount))
                                && loss.as_ref().is_none_or(|l| {
                                    [l.capital_deduction, l.capital_loss, l.capital_carry]
                                        .into_iter()
                                        .all(amount)
                                        && match l.nol {
                                            AveragingNol::None => true,
                                            AveragingNol::Current {
                                                amount: v,
                                                prior_deduction,
                                            } => amount(v) && amount(prior_deduction),
                                            AveragingNol::Unused { amount: v } => amount(v),
                                        }
                                })
                        }
                    }
            })
    }
}

/// Exact election delta on existing FarmAveraging, retaining all base-year types
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmAveragingSelection {
    /// Schedule J line 2 b net capital gain selected within total elected amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital_elected: i64,
    /// Line 2 c farm unrecaptured section 1250 gain within selected capital
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured_elected: i64,
    /// Distinct additional eligible sources shown in current accepted-record catalog
    #[libertas_size(min = 0)]
    pub sources: Vec<i64>,
    /// Actual retained base-year scope; known limitations affect averaging only
    pub history_scope: FarmAveragingHistoryScope,
}

impl FarmAveragingSelection {
    pub(crate) fn issue(&self, elected: i64) -> Option<ValidationIssue> {
        if !(0..=elected).contains(&self.capital_elected) {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SELECTION_INVALID")
                    .at_field("capital_elected"),
            );
        }
        if !(0..=self.capital_elected).contains(&self.unrecaptured_elected) {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SELECTION_INVALID")
                    .at_field("unrecaptured_elected"),
            );
        }
        if self.sources.len() > 100 {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SOURCES_INVALID").at_field("sources"),
            );
        }
        for (i, source) in self.sources.iter().enumerate() {
            if !(1..=100_000_000_000).contains(source) || self.sources[..i].contains(source) {
                return Some(
                    ValidationIssue::new("FARM_AVERAGING_SOURCES_INVALID")
                        .at_list_item("sources", i),
                );
            }
        }
        None
    }
    pub(crate) fn rule(&self, elected: i64) -> Option<&'static str> {
        self.issue(elected).map(|issue| issue.key)
    }
    pub(crate) fn valid(&self, elected: i64) -> bool {
        self.rule(elected).is_none()
    }
}

/// Remaining original history boundaries
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmAveragingHistoryScope {
    /// Full-year resident returns with retained return/J/foreign/investment source worksheets
    /// Use the actual worksheets. This choice does not establish missing current income, source attribution or spouse allocation.
    CompleteSupported,
    /// Two farming spouses with separate incomplete base-year ownership history
    SeparateSpouseHistories,
    /// Prior NOL components need HIST owner
    PriorNol,
    /// Prior/current MSA attribution not supplied
    ArcherMsa,
    /// Minor-filer base tax needs its owner
    MinorTax,
    /// Unmodeled short-year/nonresident tax worksheet
    ShortYearOrForeign,
}
