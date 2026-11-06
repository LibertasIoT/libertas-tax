pub use crate::care_source::{
    CareBenefitAnswers, CareBenefitSource, CareFiler, CareFilerSituation, CareFilingStatus,
    CareGraceBalance, CareOwnBusinessBenefit, CarePlanMaximum, CareProvider, FederalCare,
    PriorYearCare, ProviderIdentification, ProviderRelationship,
};
// Coverage: Form 2441 input facts, tax year 2026.
//   care_source.rs holds Part I providers and Part III benefit source facts.
//   care_return.rs computes deemed monthly income and prior-year Worksheet A;
//   forms/form_2441.rs emits the form. Sources: IRS 2026 draft instructions, Pub.503 (2025 final).
//   Provider name/address limits use UTF-8 bytes like the general String Size
//   constraint.
use crate::Owner;
use crate::kernel::ratio::dollars;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) fn considered_unmarried(setup: &crate::FederalSetup, people: i64) -> bool {
    setup.filing_status() == crate::FederalStatus::Separate
        && setup.lived_apart_last_six_months()
        && setup.paid_over_half_home
        && people > 0
}

/// 2025 care you paid in 2026
/// Copy the amounts from your 2025 Form 2441 and Form 1040 (Worksheet A in the 2026 instructions).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PriorYearCareInput {
    /// Two or more qualifying persons in 2025
    /// Worksheet A line 4 uses $6,000 instead of $3,000.
    pub two_or_more: bool,
    /// 2025 Form 2441 line 3 (Worksheet A line 1)
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 0, max = $CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS)]
    pub prior_line_3: i64,
    /// 2025 qualified expenses you paid in 2026 (line 2)
    /// Care given in 2025 and first paid in 2026, from payment and service records. Not reimbursed amounts, and not 2026 care.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub paid_in_return_year: i64,
    /// 2025 benefits deducted or excluded, 2025 Form 2441 lines 24 and 25 (line 5)
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 0, max = $DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS)]
    #[libertas_default(0)]
    pub prior_benefits: i64,
    /// Smaller of your and your spouse's 2025 earned income, 2025 Form 2441 lines 4 and 5 (line 7)
    /// On a 2025 return that was not joint, line 5 repeats line 4. Include any deemed income shown on those lines.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_earned_smaller: i64,
    /// 2025 Form 2441 line 6 (line 9)
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 0, max = $CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS)]
    pub prior_line_6: i64,
    /// 2025 adjusted gross income, 2025 Form 1040 line 11a (line 11)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub prior_agi: i64,
    /// Qualifying people and separate-return claimants for this prior-year care
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The prior-year qualifying people and claimants have not been established
    /// [SetText]
    /// The qualifying people and their separate-return claimants are
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_source_facts")]
    pub injured_claimants: Option<crate::InjuredPriorCareClaimantsInput>,
}

/// Saved business or partnership
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareSourceChoice {
    /// Saved income identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Owner of this saved source
    #[libertas_read_only]
    pub owner: Owner,
    /// Source name
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

/// Employee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareGraceBalanceInput {
    /// Employee
    #[libertas_unique]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// 2025 balance used for 2026 care in the grace period (line 13)
    /// The plan's statement shows the carried-forward amount and the reimbursements it paid; not a 2026 W-2 box 10 amount.
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 1, max = $DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS)]
    pub used_in_return_year: i64,
}

/// Benefit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareOwnBusinessBenefitInput {
    /// Owner
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Source
    pub source: CareBenefitSourceInput,
    /// Benefits received in 2026 under the program (line 22)
    /// From the plan's statement or Schedule K-1 box 13 code O. Do not also enter it as the business's line 14 employee benefit expense; the App deducts the allowed amount there.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub received: i64,
}

/// Source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CareBenefitSourceInput {
    /// Your sole proprietorship (Schedule C line 14)
    Business {
        /// Business
        #[libertas_enum_source("$.businesses")]
        business: i64,
    },
    /// Your farm (Schedule F line 15)
    Farm {
        /// Farm
        #[libertas_enum_source("$.farms")]
        farm: i64,
    },
    /// A partnership (Schedule K-1 box 13 code O; Schedule E line 28)
    Partnership {
        /// Partnership K-1 on this return
        #[libertas_enum_source("$.partnerships")]
        partnership: i64,
    },
}

/// Filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareFilerInput {
    /// Filer
    /// Offered only on a joint return, for each of you whose own page says full-time student or unable to care for yourself.
    #[libertas_unique]
    #[libertas_constrained_by("$.care_filers")]
    pub owner: Owner,
    /// Situation
    pub situation: CareFilerSituationInput,
}

/// Situation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CareFilerSituationInput {
    /// Full-time student
    /// Enrolled full time at a school for some part of each ticked month; at least five months. Not an on-the-job, correspondence or internet-only school.
    Student {
        /// Months
        #[libertas_enum_source("$.care_months")]
        // Calendar structure: each month is unique in Save.
        #[libertas_size(max = 12)]
        #[libertas_unordered]
        months: Vec<i64>,
        /// Earned income in those months
        /// The part of this filer's 2026 wages and self-employment earnings earned in the ticked months. Enter zero if none. The App counts at least $250 ($500 with two or more qualifying persons) for each ticked month.
        /// [NullText]
        /// No income was earned in the selected months
        /// [SetText]
        /// Earned income in the selected months was
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        earned_in_months: Option<i64>,
    },
    /// Not able to care for themselves
    /// Could not dress, clean or feed themselves, or needed constant attention, for some part of each ticked month. Choose this when both apply; tick every month of either.
    Incapable {
        /// Months
        #[libertas_enum_source("$.care_months")]
        // Calendar structure: each month is unique in Save.
        #[libertas_size(max = 12)]
        #[libertas_unordered]
        months: Vec<i64>,
        /// Earned income in those months
        /// The part of this filer's 2026 wages and self-employment earnings earned in the ticked months. Enter zero if none.
        /// [NullText]
        /// No income was earned in the selected months
        /// [SetText]
        /// Earned income in the selected months was
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        earned_in_months: Option<i64>,
        /// Lived with their spouse for more than half of 2026
        /// Required for this filer to count as a qualifying person and for the deemed income.
        same_home_over_half_year: bool,
        /// Care paid in 2026 for this filer so the other could work
        /// Include care an employer plan paid or reimbursed; the App removes excluded benefits. Enter zero if none.
        #[libertas_money("USD", 2)]
        // Filer expenses are summed before the aggregate guard; preserve arithmetic headroom.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        care_paid: i64,
    },
}

record_page_conversions!(CareBenefitSource, CareBenefitSourceInput, [], |v, Target, Source| {
    match v {
        Source::Business { business } => Target::Business { business },
        Source::Farm { farm } => Target::Farm { farm },
        Source::Partnership { partnership } => Target::Partnership { partnership },
    }
});

record_page_conversions!(CareFilerSituation, CareFilerSituationInput, [], |v, Target, Source| {
    match v {
        Source::Student { months, earned_in_months } => {
            Target::Student { months, earned_in_months }
        }
        Source::Incapable { months, earned_in_months, same_home_over_half_year, care_paid } => {
            Target::Incapable { months, earned_in_months, same_home_over_half_year, care_paid }
        }
    }
});

record_page_conversions!(CareGraceBalance, CareGraceBalanceInput, [], |v, Target| {
    Target { owner: v.owner, used_in_return_year: v.used_in_return_year }
});
record_page_conversions!(CareOwnBusinessBenefit, CareOwnBusinessBenefitInput, [], |v, Target| {
    Target { owner: v.owner, received: v.received, source: v.source.into() }
});
record_page_conversions!(CareFiler, CareFilerInput, [], |v, Target| {
    Target { owner: v.owner, situation: v.situation.into() }
});

// Runtime-only Form 2441 line 24 allocation. Source facts stay in FederalCare;
// normal business expense, partnership, Schedule SE and QBI owners consume it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CareDeductions {
    pub(crate) by_source: Vec<(i64, i64)>,
}
impl CareDeductions {
    pub(crate) fn calculate(care: &FederalCare, total: i64) -> Self {
        let shares = crate::kernel::ratio::allocate_remaining(
            total,
            &care.own_business.iter().map(|b| dollars(b.received)).collect::<Vec<_>>(),
        );
        Self {
            by_source: care
                .own_business
                .iter()
                .zip(shares)
                .filter_map(|(b, v)| {
                    (v != 0).then_some((
                        match b.source {
                            CareBenefitSource::Business { business } => business,
                            CareBenefitSource::Farm { farm } => farm,
                            CareBenefitSource::Partnership { partnership } => partnership,
                        },
                        v,
                    ))
                })
                .collect(),
        }
    }
    pub(crate) fn source(&self, id: i64) -> i64 {
        self.by_source.iter().filter(|(source, _)| *source == id).map(|(_, amount)| *amount).sum()
    }
}

record_page_conversions!(PriorYearCare, PriorYearCareInput, [], |v, Target| {
    Target {
        two_or_more: v.two_or_more,
        prior_line_3: v.prior_line_3,
        paid_in_return_year: v.paid_in_return_year,
        prior_benefits: v.prior_benefits,
        prior_earned_smaller: v.prior_earned_smaller,
        prior_line_6: v.prior_line_6,
        prior_agi: v.prior_agi,
        injured_claimants: v.injured_claimants.map(Into::into),
    }
});
