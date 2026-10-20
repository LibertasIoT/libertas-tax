use crate::validation::ValidationIssue;
// Coverage: Form 2441 input facts and computations, tax year 2026.
//   Part I providers, Part III employee/own-business benefits, deemed monthly
//   income and prior-year Worksheet A are owned here; forms/form_2441.rs emits
//   the form. Sources: IRS 2026 draft instructions, Pub.503 (2025 final).
//   Provider name/address limits use UTF-8 bytes like the general String Size
//   constraint. Tests: care_completion validation_audit_f05_* exercise actual
//   saves, reopen and rejected state at ASCII/accent/supplementary boundaries.
use crate::Owner;
use crate::kernel::lines::{LineValue, emit, row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) fn considered_unmarried(setup: &crate::FederalSetup, people: i64) -> bool {
    setup.filing_status() == crate::FederalStatus::Separate
        && setup.lived_apart_last_six_months()
        && setup.paid_over_half_home
        && people > 0
}

/// Filing status for the care worksheet
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        earned_in_months: Option<i64>,
        /// Lived with their spouse for more than half of 2026
        /// Required for this filer to count as a qualifying person and for the deemed income.
        same_home_over_half_year: bool,
        /// Care paid in 2026 for this filer so the other could work
        /// Include care an employer plan paid or reimbursed; the App removes excluded benefits. Enter zero if none.
        #[libertas_money("USD", 2)]
        // Temporary Unit 47 exclusion: submitted filer_expenses() sums care_paid before the aggregate guard; widen that receiver before removing this bound.
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
        #[libertas_number(min = 1)]
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
    #[libertas_number(min = 1)]
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
    #[libertas_number(min = 0)]
    /// [NullText]
    /// No box 10 benefits are also included in box 1 wages
    /// [SetText]
    /// Box 10 benefits also included in box 1 wages were
    pub in_box_1: Option<i64>,
    /// Forfeited or carried forward to 2027
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
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
    #[libertas_number(min = 0)]
    pub prior_earned_smaller: i64,
    /// 2025 Form 2441 line 6 (line 9)
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 0, max = $CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS)]
    pub prior_line_6: i64,
    /// 2025 adjusted gross income, 2025 Form 1040 line 11a (line 11)
    #[libertas_money("USD", 2)]
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
    #[libertas_number(min = 1)]
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
    #[libertas_number(min = 0)]
    pub prior_earned_smaller: i64,
    /// 2025 Form 2441 line 6 (line 9)
    #[libertas_money("USD", 2)]
    // 2025 Form 2441 statutory reported-line limit; checked by native prior-year/grace validation.
    #[libertas_number(min = 0, max = $CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_CENTS)]
    pub prior_line_6: i64,
    /// 2025 adjusted gross income, 2025 Form 1040 line 11a (line 11)
    #[libertas_money("USD", 2)]
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
    fn earnings(&self) -> i64 {
        match self {
            Self::Student { earned_in_months, .. } | Self::Incapable { earned_in_months, .. } => {
                earned_in_months.unwrap_or(0)
            }
        }
    }
    fn eligible(&self) -> bool {
        !matches!(self, Self::Incapable { same_home_over_half_year: false, .. })
    }
}
impl CareProvider {
    fn omission_key(&self) -> Option<&'static str> {
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
    pub(crate) fn has_benefits(&self, d: &crate::FederalDraft) -> bool {
        !self.grace.is_empty()
            || !self.own_business.is_empty()
            || d.income.iter().any(
                |e| matches!(&e.income,crate::FederalIncome::Wage {data} if data.care_benefits>0),
            )
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
    pub(crate) fn left_out(
        &self,
        d: &crate::FederalDraft,
        r: &mut crate::FederalResult,
    ) -> Option<i64> {
        for (index, f) in self.filers.iter().enumerate() {
            if matches!(f.situation,CareFilerSituation::Incapable {same_home_over_half_year:false,care_paid,..} if care_paid>0)
                && let Some(p) = d.people.as_ref().and_then(|p| {
                    if f.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
                })
            {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_CARE_COSTS_LEFT_OUT",
                        alloc::vec![crate::MessageArg::Encoded(crate::interview::person_name(
                            &p.first_name,
                            &p.last_name
                        ))],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Care).item("filers", index),
                );
            }
        }
        self.providers.iter().enumerate().for_each(|(i, p)| {
            let context =
                crate::IssueContext::Section(crate::TaxSection::Care).item("providers", i);
            let marker = match p.identification {
                ProviderIdentification::Individual => "CARE_PROVIDER_INDIVIDUAL",
                ProviderIdentification::Organization => "CARE_PROVIDER_ORGANIZATION",
                ProviderIdentification::TaxExempt => "CARE_PROVIDER_TAX_EXEMPT",
                ProviderIdentification::EmployerPlan => "CARE_PROVIDER_SEE_W2",
                ProviderIdentification::LivingAbroad => "CARE_PROVIDER_LAFCP",
                ProviderIdentification::Refused => "CARE_PROVIDER_REFUSED",
                ProviderIdentification::NotObtained => "CARE_PROVIDER_NOT_OBTAINED",
            };
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(
                    "NOTE_CARE_PROVIDER_DETAILS",
                    alloc::vec![
                        crate::MessageArg::Text(p.name.clone()),
                        crate::MessageArg::Text(p.address.clone()),
                        crate::MessageArg::Nested(alloc::boxed::Box::new(crate::Message::new(
                            marker,
                            Vec::new()
                        )))
                    ],
                ),
                context.clone(),
            );
            if p.household_employee
                && d.screening
                    .as_ref()
                    .is_none_or(|screening| screening.household_employment.is_empty())
            {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("NOTE_CARE_HOUSEHOLD_EMPLOYEE_SCHEDULE_H"),
                    context.clone(),
                );
            }
            let key = p.omission_key();
            if let Some(key) = key {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(key, alloc::vec![crate::MessageArg::Text(p.name.clone())]),
                    context.clone(),
                );
            } else {
                if p.identification == ProviderIdentification::Refused {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new(
                            "NOTE_CARE_PROVIDER_STATEMENT_ATTACH",
                            alloc::vec![crate::MessageArg::Text(p.name.clone())],
                        ),
                        context.clone(),
                    );
                }
            }
        });
        self.omitted_amount()
    }
    pub(crate) fn earned(
        &self,
        actual: [i64; 2],
        people: i64,
        r: &mut crate::FederalResult,
    ) -> [i64; 2] {
        let unit = if people >= 2 {
            500
        } else if people == 1 {
            250
        } else {
            0
        };
        let mut months: [Vec<i64>; 2] = [Vec::new(), Vec::new()];
        let mut wages = [0; 2];
        for f in &self.filers {
            let i = f.owner as usize;
            if f.situation.eligible() {
                months[i] = f.situation.months().to_vec();
                wages[i] = dollars(f.situation.earnings());
            }
        }
        let shared = months[0].iter().filter(|m| months[1].contains(m)).count() as i64;
        let counts = [months[0].len() as i64, months[1].len() as i64];
        let calculate = |winner: usize| {
            core::array::from_fn(|i| {
                let n = counts[i] - if i != winner { shared } else { 0 };
                actual[i] - wages[i] + phase::floor(wages[i], n * unit)
            })
        };
        let first: [i64; 2] = calculate(0);
        let second: [i64; 2] = calculate(1);
        let out = if phase::cap(first[0], [first[1]]) >= phase::cap(second[0], [second[1]]) {
            first
        } else {
            second
        };
        for i in 0..2 {
            if (wages[i] > 0 && wages[i] < counts[i] * unit) || shared > 0 {
                crate::federal_rules::note_once(r, msg!("WARN_CARE_DEEMED_INCOME_MONTHLY"));
            }
        }
        out
    }
    pub(crate) fn rule(&self, d: &crate::FederalDraft) -> Option<&'static str> {
        self.issue(d).map(|issue| issue.key)
    }
    pub(crate) fn valid(&self, d: &crate::FederalDraft) -> bool {
        self.validate_issue(d, false).is_none()
    }
    pub(crate) fn issue(&self, d: &crate::FederalDraft) -> Option<ValidationIssue> {
        self.validate_issue(d, true)
    }
    fn validate_issue(&self, d: &crate::FederalDraft, page: bool) -> Option<ValidationIssue> {
        let family = d.people.as_ref().map(|p| {
            crate::federal_rules::family(d, &mut crate::FederalResult::empty(d.revision), p)
        });
        let care_cost = family.as_ref().map_or(0, |f| f.care_expenses);
        let money = |v: i64| (0..=100000000000).contains(&v);
        // A return has at most two filers; these rows are unique by filer.
        for (field, len, max) in [("grace", self.grace.len(), 2), ("filers", self.filers.len(), 2)]
        {
            if len > max {
                return Some(ValidationIssue::new("CARE_INVALID_RECORD").at_field(field));
            }
        }
        for (i, p) in self.providers.iter().enumerate() {
            let field = if p.name.trim().is_empty() {
                Some("name")
            } else if p.address.trim().is_empty() {
                Some("address")
            } else if p.paid < 0 {
                Some("paid")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field(field)
                        .in_list("providers", i),
                );
            }
        }
        if page
            && !family.as_ref().is_some_and(|f| f.care_under_13_child)
            && let Some((i, _)) = self
                .providers
                .iter()
                .enumerate()
                .find(|(_, p)| p.relationship == ProviderRelationship::ParentOfChild)
        {
            return Some(
                ValidationIssue::new("CARE_PARENT_PROVIDER_NEEDS_A_CHILD")
                    .at_field("relationship")
                    .in_list("providers", i),
            );
        }
        if page && care_cost + self.filer_expenses() > 0 && self.providers.is_empty() {
            return Some(ValidationIssue::new("CARE_COSTS_NEED_A_PROVIDER").at_field("providers"));
        }
        match self.plan_maximum {
            CarePlanMaximum::Plans { maximum } if maximum <= 0 => {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field("maximum")
                        .in_field("plan_maximum"),
                );
            }
            _ => {}
        }
        if page
            && self.has_benefits(d) != matches!(self.plan_maximum, CarePlanMaximum::Plans { .. })
        {
            return Some(
                ValidationIssue::new("CARE_PLAN_MAXIMUM_REQUIRED").at_field("plan_maximum"),
            );
        }
        let joint =
            d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
        let allowed = |o: Owner| o == Owner::Taxpayer || joint;
        for (i, b) in self.benefits.iter().enumerate() {
            let box10 =
                d.income.iter().find_map(|e| (e.id == b.wage).then_some(&e.income)).and_then(|v| {
                    if let crate::FederalIncome::Wage { data } = v {
                        Some(data.care_benefits)
                    } else {
                        None
                    }
                });
            let Some(box10) = box10.filter(|v| *v > 0) else {
                return Some(
                    ValidationIssue::new("CARE_WAGE_SOURCE_INVALID")
                        .at_field("wage")
                        .in_list("benefits", i),
                );
            };
            if self.benefits[..i].iter().any(|other| other.wage == b.wage) {
                return Some(
                    ValidationIssue::new("CARE_WAGE_SOURCE_INVALID")
                        .at_field("wage")
                        .in_list("benefits", i),
                );
            }
            if b.in_box_1.is_some_and(|amount| amount < 0)
                || b.forfeited.is_some_and(|amount| amount < 0)
                || b.in_box_1
                    .unwrap_or(0)
                    .checked_add(b.forfeited.unwrap_or(0))
                    .is_none_or(|total| total > box10)
            {
                return Some(
                    ValidationIssue::new("CARE_BENEFIT_PARTS_ABOVE_BOX_10")
                        .at_field(
                            if b.in_box_1.is_some_and(|amount| amount < 0)
                                || b.in_box_1.unwrap_or(0) > box10
                            {
                                "in_box_1"
                            } else {
                                "forfeited"
                            },
                        )
                        .in_list("benefits", i),
                );
            }
        }
        for (i, f) in self.filers.iter().enumerate() {
            if !joint || self.filers[..i].iter().any(|other| other.owner == f.owner) {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("owner")
                        .in_list("filers", i),
                );
            }
            let Some(p) = d.people.as_ref().and_then(|p| {
                if f.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
            }) else {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("owner")
                        .in_list("filers", i),
                );
            };
            if match f.situation {
                CareFilerSituation::Student { .. } => !p.full_time_student,
                CareFilerSituation::Incapable { .. } => !p.incapable_self_care,
            } {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("situation")
                        .in_list("filers", i),
                );
            }
            let m = f.situation.months();
            if m.iter().enumerate().any(|(i, x)| !(1..=12).contains(x) || m[..i].contains(x)) {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if matches!(f.situation, CareFilerSituation::Student { .. }) && m.len() < 5 {
                return Some(
                    ValidationIssue::new("CARE_STUDENT_NEEDS_FIVE_MONTHS")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if !money(f.situation.earnings())
                || matches!(f.situation,CareFilerSituation::Incapable {care_paid,..} if !money(care_paid))
            {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field(if !money(f.situation.earnings()) {
                            "earned_in_months"
                        } else {
                            "care_paid"
                        })
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if p.death_date()
                .is_some_and(|date| m.iter().any(|month| *month > i64::from(date / 100 % 100)))
            {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
        }
        if let Some((i, _)) = self.grace.iter().enumerate().find(|(i, g)| {
            !allowed(g.owner)
                || g.used_in_return_year <= 0
                || g.used_in_return_year
                    > crate::year_constants::DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS
                || self.grace[..*i].iter().any(|x| x.owner == g.owner)
        }) {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD").at_field("owner").in_list("grace", i),
            );
        }
        if let Some((i, _)) = self.own_business.iter().enumerate().find(|(i,b)|self.own_business[..*i].iter().any(|other|other.owner==b.owner&&other.source==b.source)||!allowed(b.owner)||b.received<=0||match b.source { CareBenefitSource::Partnership {partnership}=>!d.income.iter().any(|e|e.id==partnership&&e.owner==b.owner&&matches!(e.income,crate::FederalIncome::Partnership {..})),CareBenefitSource::Business {business}|CareBenefitSource::Farm {farm:business}=>!d.income.iter().any(|e|e.id==business&&e.owner==b.owner&&matches!(&e.income,crate::FederalIncome::Business {data} if matches!(data.activity,crate::BusinessActivity::Farming {..})==matches!(b.source,CareBenefitSource::Farm {..}))) }) {return Some(ValidationIssue::new("CARE_BUSINESS_SOURCE_INVALID").at_field("source").in_list("own_business", i));}
        if matches!(self.filing,CareFilingStatus::Separate {spouse_earned_income:Some(v)} if !money(v))
        {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD")
                    .at_field("spouse_earned_income")
                    .in_field("filing"),
            );
        }
        if matches!(self.unpaid_return_year_care, Some(amount) if amount <= 0 || !money(amount)) {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD").at_field("unpaid_return_year_care"),
            );
        }
        if self.prior_year.as_ref().is_some_and(|p| {
            p.injured_claimants.as_ref().is_some_and(|facts| {
                facts.people().iter().map(|p| p.expense).sum::<i64>() != p.paid_in_return_year
            })
        }) {
            return Some(
                ValidationIssue::new("INJURED_PRIOR_CARE_EXPENSE_TOTAL")
                    .at_field("injured_claimants")
                    .in_field("prior_year"),
            );
        }
        if self.prior_year.as_ref().is_some_and(|p| !p.valid()) {
            return Some(
                ValidationIssue::new("PRIOR_CARE_LINE_6_ABOVE_LINE_3")
                    .at_field("prior_line_6")
                    .in_field("prior_year"),
            );
        }
        let separate =
            d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Separate);
        if self.active() && separate != matches!(self.filing, CareFilingStatus::Separate { .. }) {
            return Some(ValidationIssue::new("CARE_FILING_STATUS_INVALID").at_field("filing"));
        }
        for (field, available) in [
            ("benefits", crate::forms::form_2441::benefit_sources(d, self).is_some()),
            ("providers", self.omitted_amount().is_some()),
        ] {
            if !available {
                return Some(
                    ValidationIssue::new("CARE_SOURCE_AMOUNTS_UNAVAILABLE").at_field(field),
                );
            }
        }
        None
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
    pub(crate) fn credit(&self, r: &mut crate::FederalResult) -> i64 {
        use crate::year_constants::*;
        let line1 = dollars(self.prior_line_3);
        let line2 = dollars(self.paid_in_return_year);
        let line3 = line1 + line2;
        let line4 = if self.two_or_more {
            CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_DOLLARS
        } else {
            CARE_EXPENSE_LIMIT_ONE_PERSON_2025_DOLLARS
        };
        let line5 = dollars(self.prior_benefits);
        let line6 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line4, line5)
                .get();
        let line7 = dollars(self.prior_earned_smaller);
        let line8 = phase::cap(line3, [line6, line7]);
        let line9 = dollars(self.prior_line_6);
        let line10 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line8, line9)
                .get();
        let line11 = dollars(self.prior_agi);
        let rate = phase::floor(
            CARE_CREDIT_MAXIMUM_2025_PERCENT
                - phase::steps(
                    line11 - CARE_CREDIT_PHASEOUT_START_2025_DOLLARS,
                    CARE_CREDIT_PHASEOUT_STEP_2025_DOLLARS,
                    phase::StepRounding::Up,
                ),
            CARE_CREDIT_MINIMUM_2025_PERCENT,
        );
        let credit = fraction(line10, rate, 100);
        emit(
            r,
            &mut (),
            [
                (1, line1),
                (2, line2),
                (3, line3),
                (4, line4),
                (5, line5),
                (6, line6),
                (7, line7),
                (8, line8),
                (9, line9),
                (10, line10),
                (11, line11),
                (13, credit),
            ],
            |(line, amount), _, _| {
                Some(row(
                    Some(crate::FormLine::new(
                        crate::TaxForm::Form2441,
                        &alloc::format!("Worksheet A {line}"),
                    )),
                    &alloc::format!("care_prior_worksheet_a_line{line}"),
                    LineValue::MoneyDollars(amount),
                    &["credits.care.prior_year"],
                    "https://www.irs.gov/instructions/i2441",
                    msg!("EXPLAIN_CARE_WORKSHEET_A"),
                ))
            },
        );
        crate::kernel::lines::form_value(
            r,
            crate::FormLine::new(crate::TaxForm::Form2441, "Worksheet A 12"),
            "care_prior_worksheet_a_line12",
            LineValue::PercentageHundredths(rate * 100),
            &["credits.care.prior_year"],
            "https://www.irs.gov/instructions/i2441",
            msg!("EXPLAIN_CARE_WORKSHEET_A"),
        );
        crate::federal_rules::note_once_scoped(
            r,
            msg!("NOTE_CARE_WORKSHEET_A_ATTACH"),
            crate::IssueContext::Section(crate::TaxSection::Care),
        );
        credit
    }
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
    #[libertas_number(min = 1)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        earned_in_months: Option<i64>,
        /// Lived with their spouse for more than half of 2026
        /// Required for this filer to count as a qualifying person and for the deemed income.
        same_home_over_half_year: bool,
        /// Care paid in 2026 for this filer so the other could work
        /// Include care an employer plan paid or reimbursed; the App removes excluded benefits. Enter zero if none.
        #[libertas_money("USD", 2)]
        // Temporary Unit 47 exclusion: submitted filer_expenses() sums care_paid before the aggregate guard; widen that receiver before removing this bound.
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
    pub(crate) fn owners(&self, d: &crate::FederalDraft) -> [i64; 2] {
        let mut out = [0; 2];
        for e in &d.income {
            out[e.owner as usize] += self.source(e.id);
        }
        out
    }
    pub(crate) fn business_projection(&self, d: &crate::FederalDraft) -> crate::FederalDraft {
        let mut out = d.clone();
        for e in &mut out.income {
            let amount = self.source(e.id);
            if amount != 0
                && let crate::FederalIncome::Business { data } = &mut e.income
            {
                data.expenses.push(crate::FederalBusinessExpense {
                    category: crate::BusinessExpenseCategory::EmployeeBenefits,
                    amount: amount * 100,
                    treatment: crate::BusinessExpenseKind::Ordinary,
                });
            }
        }
        out
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
