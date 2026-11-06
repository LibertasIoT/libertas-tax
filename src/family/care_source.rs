// Accepted child and dependent-care source facts and intrinsic validation.
use crate::common::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Filing status for the care worksheet
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum CareFilingStatus {
    /// Not married filing separately
    Other,
    /// Married filing separately
    Separate {
        /// Your spouse's 2026 earned income (line 19)
        /// Needed unless you qualify as considered unmarried. Choose Unknown only when you cannot supply the amount; the App then counts zero with an estimate note.
        /// Enter what your spouse actually earned, not the income-tax share assigned to either spouse under community-property rules.
        /// [NullText]
        /// I do not know my spouse's 2026 earned income
        /// [SetText]
        /// My spouse's 2026 earned income was
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_earned_income: Option<i64>,
    },
}
/// Care provider
/// Form 2441 Part I, one column per person or organization you paid.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareProvider {
    /// Accepted record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Provider's name (line 1a)
    /// The person or organization as named on their Form W-10, invoice or statement. For care your employer's own plan furnished, your employer's name.
    #[libertas_size(min = 1)]
    pub name: String,
    /// Address (line 1b)
    /// Street, city, state and ZIP or foreign postal code. "See W-2" for your employer's own plan.
    #[libertas_size(min = 1)]
    pub address: String,
    /// Identifying number (line 1c)
    /// The number itself is never entered here; say what you hold.
    pub identification: ProviderIdentification,
    /// This provider was your household employee in 2026 (line 1d)
    /// Someone who came to your home, such as a nanny, whose work you controlled; not a daycare center. Yes may mean household employment taxes on Schedule H (Other taxes).
    pub household_employee: bool,
    /// Amount you paid in 2026 (line 1e)
    /// Everything actually paid to this provider in 2026, including amounts your employer paid to them for you, whenever the care was given. Do not subtract reimbursements.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Relationship to you
    /// No credit or benefit exclusion for amounts paid to these people (section 21(e)(6)). Other relatives count, even if they live with you.
    #[libertas_default(NotRelated)]
    pub relationship: ProviderRelationship,
}

/// Identifying number (line 1c)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ProviderIdentification {
    /// An individual's SSN or ITIN, from their Form W-10 or another record
    Individual,
    /// An organization's EIN, from its Form W-10, invoice or letterhead
    Organization,
    /// A tax-exempt organization, such as a church or school ("Tax-Exempt")
    TaxExempt,
    /// Your employer's own plan furnished the care ("See W-2")
    /// Your employer ran the daycare or hired the provider. Enter zero as the amount paid; the benefit is in W-2 box 10.
    EmployerPlan,
    /// A foreign provider with no U.S. number while you lived abroad ("LAFCP")
    LivingAbroad,
    /// You asked and the provider refused; you keep a record of asking
    /// The credit still applies; the return needs a statement that you requested the information (due diligence). The App adds the reminder.
    Refused,
    /// Not obtained
    /// Amounts paid to this provider earn no credit and no benefit exclusion, with a note (section 21(e)(9)).
    NotObtained,
}

/// Relationship to you
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ProviderRelationship {
    /// Not related, or a relative you cannot claim as a dependent
    NotRelated,
    /// Your spouse at any time in 2026
    Spouse,
    /// Your child, stepchild or foster child under 19 at the end of 2026
    ChildUnder19,
    /// A person you or your spouse can claim as a dependent
    Dependent,
    /// The other parent of a child under 13 you paid care for
    ParentOfChild,
}

/// Employee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareGraceBalance {
    /// Employee
    #[libertas_unique]
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
pub struct CareOwnBusinessBenefit {
    /// Owner
    pub owner: Owner,
    /// Source
    pub source: CareBenefitSource,
    /// Benefits received in 2026 under the program (line 22)
    /// From the plan's statement or Schedule K-1 box 13 code O. Do not also enter it as the business's line 14 employee benefit expense; the App deducts the allowed amount there.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub received: i64,
}

/// Source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CareBenefitSource {
    /// Your sole proprietorship (Schedule C line 14)
    Business {
        /// Business
        business: i64,
    },
    /// Your farm (Schedule F line 15)
    Farm {
        /// Farm
        farm: i64,
    },
    /// A partnership (Schedule K-1 box 13 code O; Schedule E line 28)
    Partnership {
        /// Partnership K-1 on this return
        partnership: i64,
    },
}

/// Filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareFiler {
    /// Filer
    /// Offered only on a joint return, for each of you whose own page says full-time student or unable to care for yourself.
    #[libertas_unique]
    pub owner: Owner,
    /// Situation
    pub situation: CareFilerSituation,
}

/// Situation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CareFilerSituation {
    /// Full-time student
    /// Enrolled full time at a school for some part of each ticked month; at least five months. Not an on-the-job, correspondence or internet-only school.
    Student {
        /// Months

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

/// Combined maximum of your dependent-care plans (Form 2441 line 21)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CarePlanMaximum {
    /// No dependent-care benefits or plan balance used in 2026
    NoBenefits,
    /// Dependent-care plans provided benefits in 2026
    Plans {
        /// Combined maximum the plans allow you to exclude for 2026
        /// From the plan documents or administrators, before the filing-status limit. Add the maxima of distinct plans, counting each plan once even if it appears on several W-2s or provided a 2025 grace-period balance. Include a plan of your business or partnership. Do not enter the benefits received: a $5,000 plan stays $5,000 when its grace period also reimburses $2,000 in 2026.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        maximum: i64,
    },
}

/// Child and dependent care (Form 2441)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCare {
    /// Care providers
    pub providers: Vec<CareProvider>,
    /// Answers for each W-2 with box 10 benefits
    pub benefits: Vec<CareBenefitAnswers>,
    /// Combined maximum of your dependent-care plans (line 21)
    pub plan_maximum: CarePlanMaximum,
    /// 2025 grace-period balances used in 2026
    // Tax structure: at most two filers, unique by owner in validate_issue.
    #[libertas_size(max = 2)]
    pub grace: Vec<CareGraceBalance>,
    /// Benefits from an own business or partnership
    pub own_business: Vec<CareOwnBusinessBenefit>,
    /// Joint filers who were students or unable to care for themselves
    // Tax structure: at most two filers, unique by owner in validate_issue.
    #[libertas_size(max = 2)]
    pub filers: Vec<CareFiler>,
    /// Filing-status facts for Form 2441 line 19
    pub filing: CareFilingStatus,
    /// 2026 care unpaid at year end
    /// [NullText]
    /// No unpaid 2026 care expenses are claimed
    /// [SetText]
    /// The unpaid 2026 care expenses claimed are
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    #[libertas_default($null)]
    pub unpaid_return_year_care: Option<i64>,
    /// 2025 expenses paid in 2026
    /// [NullText]
    /// No 2025 care expenses paid in 2026 are claimed
    /// [SetText]
    /// The 2025 care expenses paid in 2026 are
    #[libertas_default($null)]
    pub prior_year: Option<crate::PriorYearCare>,
}

/// Answers for one W-2's box 10
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareBenefitAnswers {
    /// Income record identity of the W-2
    pub wage: i64,
    /// Part of box 10 also in box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    /// [NullText]
    /// No box 10 benefits are also included in box 1 wages
    /// [SetText]
    /// Box 10 benefits also included in box 1 wages were
    pub in_box_1: Option<i64>,
    /// Forfeited or carried forward to 2027
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    /// [NullText]
    /// No box 10 benefits were forfeited or carried forward to 2027
    /// [SetText]
    /// Box 10 benefits forfeited or carried forward to 2027 were
    pub forfeited: Option<i64>,
}

/// 2025 care you paid in 2026
/// Copy the amounts from your 2025 Form 2441 and Form 1040 (Worksheet A in the 2026 instructions).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PriorYearCare {
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
    pub injured_claimants: Option<crate::InjuredPriorCareClaimants>,
}
impl Default for FederalCare {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            benefits: Vec::new(),
            plan_maximum: CarePlanMaximum::NoBenefits,
            grace: Vec::new(),
            own_business: Vec::new(),
            filers: Vec::new(),
            filing: CareFilingStatus::Other,
            unpaid_return_year_care: None,
            prior_year: None,
        }
    }
}
impl CareFilerSituation {
    pub(crate) fn months(&self) -> &[i64] {
        match self {
            Self::Student { months, .. } | Self::Incapable { months, .. } => months,
        }
    }
}
impl CareProvider {
    pub(crate) fn omission_key(&self) -> Option<&'static str> {
        if self.relationship != ProviderRelationship::NotRelated {
            Some("WARN_CARE_RELATED_PROVIDER_LEFT_OUT")
        } else if self.identification == ProviderIdentification::NotObtained {
            Some("WARN_CARE_PROVIDER_NOT_IDENTIFIED_LEFT_OUT")
        } else {
            None
        }
    }
}
impl FederalCare {
    pub(crate) fn filer_expenses(&self) -> i64 {
        self.filers
            .iter()
            .map(|f| match f.situation {
                CareFilerSituation::Incapable {
                    same_home_over_half_year: true, care_paid, ..
                } => care_paid,
                _ => 0,
            })
            .sum()
    }
    pub(crate) fn filer_people(&self) -> i64 {
        self.filers
            .iter()
            .filter(|f| {
                matches!(
                    f.situation,
                    CareFilerSituation::Incapable { same_home_over_half_year: true, .. }
                )
            })
            .count() as i64
    }
    pub(crate) fn active(&self) -> bool {
        !self.providers.is_empty()
            || !self.benefits.is_empty()
            || !self.grace.is_empty()
            || !self.own_business.is_empty()
            || !self.filers.is_empty()
            || self.prior_year.as_ref().is_some()
            || self.unpaid_return_year_care.unwrap_or(0) > 0
    }
    pub(crate) fn omitted_amount(&self) -> Option<i64> {
        crate::kernel::ratio::checked_sum(
            self.providers
                .iter()
                .filter(|provider| provider.omission_key().is_some())
                .map(|provider| provider.paid),
        )
    }
}
impl PriorYearCare {
    pub(crate) fn valid(&self) -> bool {
        self.prior_line_6 <= self.prior_line_3
            && self.prior_line_6 >= 0
            && self.prior_line_3 <= crate::year_constants::CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS
            && (1..=100000000000).contains(&self.paid_in_return_year)
            && (0..=crate::year_constants::DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS)
                .contains(&self.prior_benefits)
            && (0..=100000000000).contains(&self.prior_earned_smaller)
            && (-100000000000..=100000000000).contains(&self.prior_agi)
            && self.injured_claimants.as_ref().is_none_or(|facts| {
                facts.valid()
                    && facts.people().iter().map(|p| p.expense).sum::<i64>()
                        == self.paid_in_return_year
            })
    }
}

impl CareFilerSituation {
    pub(crate) fn earnings(&self) -> i64 {
        match self {
            Self::Student { earned_in_months, .. } | Self::Incapable { earned_in_months, .. } => {
                earned_in_months.unwrap_or(0)
            }
        }
    }
    pub(crate) fn eligible(&self) -> bool {
        !matches!(self, Self::Incapable { same_home_over_half_year: false, .. })
    }
}
