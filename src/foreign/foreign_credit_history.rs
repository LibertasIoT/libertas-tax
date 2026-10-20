use crate::FederalResult;
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use alloc::vec::Vec;
use libertas_macros::*;

/// Retained foreign-credit category
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignCreditCategory {
    /// General category
    General,
    /// Passive category
    Passive,
    /// Foreign branch category
    ForeignBranch,
}

impl ForeignCreditCategory {
    // The words of the calculation ids (`foreign_{name}_...`).
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Passive => "passive",
            Self::ForeignBranch => "branch",
        }
    }
}

/// Opening carryover of one origin year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignCarryoverOpening {
    /// Line 1 — this year's column of line 8 of your 2025 Schedule B (Form 1116)
    /// Enter zero if none.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: adjusted() still adds four i64 source terms unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carryover: i64,
    /// Line 2a — carryback adjustment
    /// Only in the 2025 column: the estimated carryback on your 2025 Schedule B line 7 less the actual carryback to 2024. A negative amount when the actual carryback was larger.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: adjusted() still adds four i64 source terms unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub carryback_adjustment: i64,
    /// Line 2b — section 905(c) redeterminations
    /// The change in this column from a foreign tax refund or other redetermination reported this year on Schedule C (Form 1116). A negative amount reduces the carryover.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: adjusted() still adds four i64 source terms unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub redetermination: i64,
    /// Lines 2c through 2g — other adjustments
    /// Domestic audit adjustments and any other change needed for the carryover available this year. A negative amount reduces it.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: adjusted() still adds four i64 source terms unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_adjustments: i64,
}

impl ForeignCarryoverOpening {
    // Line 3: lines 1 and 2a through 2g.
    pub(crate) fn adjusted(&self) -> i64 {
        self.carryover + self.carryback_adjustment + self.redetermination + self.other_adjustments
    }

    fn valid(&self) -> bool {
        const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;
        const SIGNED: core::ops::RangeInclusive<i64> = -100_000_000_000..=100_000_000_000;
        MONEY.contains(&self.carryover)
            && SIGNED.contains(&self.carryback_adjustment)
            && SIGNED.contains(&self.redetermination)
            && SIGNED.contains(&self.other_adjustments)
            && self.adjusted() >= 0
    }
}

/// Origin year column
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ForeignCarryoverColumn {
    /// Year the unused taxes originated
    /// 2016 is the tenth preceding year, column (i); 2025 the first preceding year, column (xii).
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = 2016, max = 2025)]
    pub year: u32,
    /// Regular tax
    pub regular: ForeignCarryoverOpening,
    /// Alternative minimum tax
    /// From your separate AMT Schedule B; never copy the regular amounts unless the AMT records show them.
    pub amt: ForeignCarryoverOpening,
}

impl ForeignCarryoverColumn {
    pub(crate) fn opening(&self, amt: bool) -> &ForeignCarryoverOpening {
        if amt { &self.amt } else { &self.regular }
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.year != 2025 {
            for (name, opening) in [("regular", &self.regular), ("amt", &self.amt)] {
                if opening.carryback_adjustment != 0 {
                    return Some(
                        ValidationIssue::new("FOREIGN_CARRYBACK_ADJUSTMENT_PRIOR_YEAR_ONLY")
                            .at_field("carryback_adjustment")
                            .in_field(name),
                    );
                }
            }
        }
        for (name, opening) in [("regular", &self.regular), ("amt", &self.amt)] {
            if opening.adjusted() < 0 {
                return Some(ValidationIssue::new("FOREIGN_CARRYOVER_BELOW_ZERO").at_field(name));
            }
        }
        None
    }
}

/// Schedule B (Form 1116) for one category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ForeignCarryoverSchedule {
    /// Category — the box checked on the schedule
    /// Pre-2018 general category balances are general category.
    pub category: ForeignCreditCategory,
    /// Origin years
    /// ----
    /// Year
    // Form 1116 carryovers span ten origin years (2016–2025), unique in issue.
    #[libertas_size(min = 1, max = 10)]
    pub columns: Vec<ForeignCarryoverColumn>,
}

/// Unused foreign taxes from prior returns
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ForeignCarryoverRecords {
    /// Your 2025 Schedules B (Form 1116)
    /// One for each category with a carryover. The App uses each year oldest first, expires the 2016 remainder and figures the 2026 carryover.
    /// ----
    /// Schedule B
    // Fixed three-variant ForeignCreditCategory catalog; native validation requires unique categories.
    #[libertas_size(min = 1, max = 3)]
    pub schedules: Vec<ForeignCarryoverSchedule>,
    /// Every balance arose on your own returns, or on joint returns with the spouse on this joint return
    /// Spouses' balances from separate returns combine on a joint return. A balance from a joint return with someone you are not filing jointly with now needs an allocation between you, which this estimate does not figure; No leaves these carryovers out with a note.
    pub same_filing_unit: bool,
}

/// Foreign-credit carryovers entering 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum ForeignCreditHistory {
    /// No unused foreign taxes from earlier years
    None,
    /// Use your prior Schedules B (Form 1116)
    Retained(crate::ForeignCarryoverRecords),
}

impl ForeignCreditHistory {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let Self::Retained(records) = self else {
            return None;
        };
        if let Some((i, _)) = records.schedules.iter().enumerate().find(|(i, s)| {
            records.schedules[..*i].iter().any(|earlier| earlier.category == s.category)
        }) {
            return Some(
                ValidationIssue::new("FOREIGN_CARRYOVER_CATEGORY_TWICE")
                    .at_field("category")
                    .in_list("schedules", i),
            );
        }
        for (i, schedule) in records.schedules.iter().enumerate() {
            if let Some((j, _)) =
                schedule.columns.iter().enumerate().find(|(j, c)| {
                    schedule.columns[..*j].iter().any(|earlier| earlier.year == c.year)
                })
            {
                return Some(
                    ValidationIssue::new("FOREIGN_CARRYOVER_YEAR_TWICE")
                        .at_field("year")
                        .in_list("columns", j)
                        .in_list("schedules", i),
                );
            }
            if let Some(issue) = schedule.columns.iter().enumerate().find_map(|(j, c)| {
                c.issue().map(|issue| issue.in_list("columns", j).in_list("schedules", i))
            }) {
                return Some(issue);
            }
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Retained(records) => {
                self.rule().is_none()
                    && !records.schedules.is_empty()
                    && records.schedules.iter().all(|s| {
                        !s.columns.is_empty()
                            && s.columns.iter().all(|c| {
                                (2016..=2025).contains(&c.year)
                                    && c.regular.valid()
                                    && c.amt.valid()
                            })
                    })
            }
        }
    }

    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Retained(_))
    }

    // The Schedule B of one category, when the person entered it.
    pub(crate) fn schedule(
        &self,
        kind: ForeignCreditCategory,
    ) -> Option<&ForeignCarryoverSchedule> {
        match self {
            Self::None => None,
            Self::Retained(records) => records.schedules.iter().find(|s| s.category == kind),
        }
    }

    pub(crate) fn check(&self, result: &mut FederalResult) {
        if let Self::Retained(_) = self {
            crate::require_localized!(
                result,
                self.valid(),
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign-credit history",
                "ISSUE_PROVIDE_UNIQUE_YEAR_CATEGORY_BALANCES_FROM_COMPLETE_RECONCILED_REGU_96463AA7",
                "Provide unique year/category balances from the regular and AMT records through 2025.",
            );
        }
    }
}

/// Retained 2025 foreign-credit limitation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignPriorLimit {
    /// Form 1116 line 23, credit limitation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub limit: i64,
    /// Form 1116 line 14, total taxes available before this carryback
    /// Include older carryovers already considered on the 2025 return. Exclude the proposed 2026 carryback.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub taxes: i64,
}

/// Retained 2025 category worksheet
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignPriorCredit {
    /// Category on the 2025 worksheet
    pub category: ForeignCreditCategory,
    /// Regular Form 1116
    pub regular: ForeignPriorLimit,
    /// Separate AMT Form 1116
    /// Use independently retained AMT figures, even if no AMT was payable.
    pub amt: ForeignPriorLimit,
}

/// Review 2026 excess foreign taxes for carryback to 2025
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignCarryback {
    // This is unavailable prior-return worksheet data, not an unknown Yes/No
    // confirmation. A current foreign-tax source cannot establish hypothetical
    // 2025 credit capacity, older carry absorption or later redeterminations.
    // capacity preserves 2026 tax and provisional carries with the existing
    // NOTE_NAME_FOREIGN_CREDIT_EXCESS_REVIEW_THE_PRIOR_YEAR_CARRYBACK_BEF_257AE351 notice.
    /// Prior-year review is unfinished
    /// Current-year credit can be calculated, but excess taxes do not yet establish a final carryforward. Use this when the prior-year credit or hypothetical deduction-year worksheets are unavailable or unresolved.
    Unreviewed,
    /// Valid small-credit election without Form 1116 in 2025
    /// That election prohibits both regular and AMT carrybacks into 2025; all otherwise eligible 2026 excess remains for later years.
    SmallCreditElection,
    /// Foreign taxes were deducted in 2025
    DeductionYear {
        /// Retained hypothetical 2025 credit worksheets
        /// Use reconciled worksheets computed as if credit had been chosen instead of the foreign-tax deduction, separately for regular tax and AMT. Include older carryover absorption. Include each category with remaining capacity; an omitted category has none. Do not use the actual deduction-year tax limitation or enter a computed 2026 absorption amount.
        // Fixed three-variant ForeignCreditCategory catalog; native validation requires unique categories.
        #[libertas_size(min = 0, max = 3)]
        categories: Vec<ForeignPriorCredit>,
        /// Complete hypothetical credit records with the 2025 deduction choice unchanged
        /// Same filing unit and calendar year, with no unresolved ownership changes, redeterminations, loss adjustments or amendments. The App computes 2026 excess absorbed without a tax benefit; changing the 2025 deduction choice requires a separate amended-return calculation. Check the retained 2025 return, worksheets and subsequent amendments or IRS changes to answer. No treats the carryback as unreviewed: 2026 tax is unchanged and the new carryforward is provisional.
        confirmed: bool,
    },
    /// Use retained 2025 credit worksheets
    PriorReturn {
        /// Categories with a retained 2025 credit worksheet
        /// Include every general, passive or foreign branch category with remaining capacity. An omitted category has zero capacity; omit it only if there was no eligible foreign income or its limit was exhausted. These are prior-return records, not a user-computed carryback.
        // Fixed three-variant ForeignCreditCategory catalog; native validation requires unique categories.
        #[libertas_size(min = 0, max = 3)]
        categories: Vec<ForeignPriorCredit>,
        /// Complete reconciled credit-year records for the same filing unit
        /// Calendar-year 2025 credit treatment, not a foreign-tax deduction or small-credit election. All older carryovers are included, with no unresolved ownership changes, loss recapture, tax redeterminations or other amendments. Figures precede this 2026 carryback. Allocation is not a refund calculation; claiming a refund requires recomputing the prior return and dependent credits. Check the retained 2025 return, worksheets and subsequent amendments or IRS changes to answer. No treats the carryback as unreviewed: 2026 tax is unchanged and the new carryforward is provisional.
        confirmed: bool,
    },
}

impl ForeignCarryback {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::PriorReturn { categories, .. } | Self::DeductionYear { categories, .. } => {
                categories.iter().enumerate().all(|(i, c)| {
                    !categories[..i].iter().any(|p| p.category == c.category)
                        && [&c.regular, &c.amt].iter().all(|v| {
                            (0..=100000000000).contains(&v.limit)
                                && (0..=100000000000).contains(&v.taxes)
                        })
                })
            }
            _ => true,
        }
    }

    // Whether the 2026 excess of this category goes to the 2025 return as a
    // deduction-year absorption (Schedule B line 7 under the id
    // `deduction_year_absorbed_prior_year`).
    pub(crate) fn deduction_year(&self) -> bool {
        matches!(self, Self::DeductionYear { .. })
    }

    // The 2025 capacity of one category in the regular tax and the AMT, for
    // the Schedule B line 7 carryback. An unreviewed carryback, a small-credit
    // election year and a category without a retained worksheet have none; the
    // excess then stays a provisional carryforward, with a note when the
    // review is unfinished.
    pub(crate) fn capacity(
        &self,
        kind: ForeignCreditCategory,
        excess: [i64; 2],
        history: &ForeignCreditHistory,
        result: &mut FederalResult,
    ) -> [i64; 2] {
        let name = kind.name();
        if matches!(self, Self::DeductionYear { .. } | Self::SmallCreditElection) {
            let consistent = history.schedule(kind).is_none_or(|schedule| {
                schedule
                    .columns
                    .iter()
                    .all(|c| c.year != 2025 || (c.regular.adjusted() == 0 && c.amt.adjusted() == 0))
            });
            crate::require_localized!(
                result,
                consistent,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign-credit origin year",
                "ISSUE_A_PRIOR_YEAR_FOREIGN_TAX_DEDUCTION_OR_NO_FORM_1116_SMALL_CREDIT_ELECTION_30E20D12",
                "A 2025 foreign-tax deduction or no-Form-1116 small-credit election cannot generate unused 2025 foreign taxes. Reconcile the origin year and prior election; older surviving balances retain their original years.",
            );
            if !consistent {
                return [0; 2];
            }
        }
        let mut capacity = [0; 2];
        match self {
            Self::Unreviewed if excess != [0; 2] => {
                crate::federal_rules::note_once_scoped(
                    result,
                    crate::Message::new(
                        "NOTE_NAME_FOREIGN_CREDIT_EXCESS_REVIEW_THE_PRIOR_YEAR_CARRYBACK_BEF_257AE351",
                        alloc::vec![crate::MessageArg::Text(alloc::string::String::from(name))],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                        .item("foreign-credit-category", kind as usize),
                );
                return [0; 2];
            }
            Self::PriorReturn { categories, .. } | Self::DeductionYear { categories, .. } => {
                let valid = self.valid();
                crate::require_localized!(
                    result,
                    valid,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    "Foreign-credit carryback",
                    "ISSUE_CONFIRM_COMPLETE_PRIOR_YEAR_CREDIT_OR_HYPOTHETICAL_DEDUCTION_YEAR_WORKSHE_9E12DBA6",
                    "Enter each category of the 2025 credit or hypothetical deduction-year worksheets once before allocating excess taxes.",
                );
                if !valid {
                    return [0; 2];
                }
                if let Some(prior) = categories.iter().find(|c| c.category == kind) {
                    capacity = [
                        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                            dollars(prior.regular.limit),
                            dollars(prior.regular.taxes),
                        )
                        .get(),
                        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                            dollars(prior.amt.limit),
                            dollars(prior.amt.taxes),
                        )
                        .get(),
                    ];
                }
                if let Some(schedule) = history.schedule(kind) {
                    let consistent = schedule.columns.iter().all(|c| {
                        (capacity[0] == 0 || dollars(c.regular.carryover) == 0)
                            && (capacity[1] == 0 || dollars(c.amt.carryover) == 0)
                    });
                    crate::require_localized!(
                        result,
                        consistent,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        "Foreign-credit carryback history",
                        "ISSUE_UNUSED_BALANCES_ENTERING_RETURN_YEAR_CONFLICT_WITH_AVAILABLE_PRIOR_YEAR_CAPACITY_2C145BA2",
                        "Unused balances entering 2026 conflict with available 2025 capacity in the same category and tax system. Reconcile older usage before allocating new taxes.",
                    );
                    if !consistent {
                        return [0; 2];
                    }
                }
            }
            _ => (),
        }
        [phase::cap(excess[0], [capacity[0]]), phase::cap(excess[1], [capacity[1]])]
    }
}
