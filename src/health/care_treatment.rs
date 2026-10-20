// LTC receipt source facts outside Form8853 Section C. Reg1.105-1(d) supplies the
// insured employer-funding ratios; Reg1.105-2/3 supplies actual reimbursement and
// permanent-injury exclusions. Existing annuity/recovery owners retain their tax
// calculations. Source IDs name accepted records, never a second income ledger.
use crate::CareContractQualification;
use crate::kernel::ratio::exact::{Exact, whole};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How this stream of LTC benefits was paid
/// Use the policyholder’s records, insurer/provider classification statement and the state insurance department’s licensing records.
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
pub enum LtcPaymentOrigin {
    /// LTC insurance benefits, not a life-policy sale or death-benefit advance
    Insurance,
    /// Advance under the insured's life insurance contract
    AcceleratedLife {
        /// The policyholder has a business insurable interest in the insured
        /// Yes if the insured is the policyholder's director, officer or employee, or has a financial interest in its business. This prevents section101(g)'s exclusion, even for terminal illness.
        business_insurable_interest: bool,
    },
    /// Sale or assignment to a viatical settlement provider
    Viatical {
        /// Provider regularly buys life contracts on terminally or chronically ill insureds
        regular_business: bool,
        /// Provider qualification under the insured person’s state licensing requirements
        regulation: ViaticalRegulation,
        /// The policyholder has a business insurable interest in the insured
        /// Yes if the insured is its director, officer or employee, or has a financial interest in its business.
        business_insurable_interest: bool,
    },
}
impl LtcPaymentOrigin {
    pub(crate) fn life_eligible(&self) -> bool {
        match self {
            Self::Insurance => false,
            Self::AcceleratedLife { business_insurable_interest } => !*business_insurable_interest,
            Self::Viatical { regular_business, regulation, business_insurable_interest } => {
                *regular_business
                    && !*business_insurable_interest
                    && match regulation {
                        ViaticalRegulation::StateLicense { licensed } => *licensed,
                        ViaticalRegulation::NoStateLicense { naic_compliant } => *naic_compliant,
                    }
            }
        }
    }
}

/// Viatical provider’s applicable regulatory qualification
/// Check the insured person’s state insurance department requirements and the provider’s actual license or compliance statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ViaticalRegulation {
    /// The state requires a provider license
    StateLicense {
        /// Provider is licensed in that state
        /// Check the state insurance department’s provider-license record.
        licensed: bool,
    },
    /// The state does not require a provider license
    NoStateLicense {
        /// Provider meets the NAIC standards prescribed by section 101(g)(2)(B)
        /// Check the provider’s actual compliance statement.
        naic_compliant: bool,
    },
}

/// Premiums attributable to the actual accident/health coverage
/// Employer premiums here are only payments excluded from your income. Include employer payments already taxed to you and after-tax payroll deductions with employee premiums. Use the current policy year for an individual policy and the prescribed latest known years for a group policy. Do not include another benefit's premiums unless they cannot be separated under the plan's records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct CareFundingAmounts {
    /// Employee's after-tax net premiums
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub employee_premium: i64,
    /// Employer's tax-free net premiums
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub employer_premium: i64,
}
impl CareFundingAmounts {
    fn valid(&self) -> bool {
        [self.employee_premium, self.employer_premium].iter().all(|v| *v >= 0)
            && (self.employee_premium > 0 || self.employer_premium > 0)
    }
}

/// One group-policy year known at the start of the calendar year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareFundingYear {
    /// Actual ending policy year
    // Tax structure: historical policy funding cannot come from a later tax year.
    #[libertas_number(min = 1900, max = $HEALTH_TAX_YEAR)]
    pub year: u32,
    /// Actual net premiums for this coverage and employee class
    /// Keep dividends, credits and retroactive adjustments in these net amounts. Enter the latest three policy years known at the start of2026, or all known years if fewer than three; do not selectively omit an employer-funded year.
    pub premiums: CareFundingAmounts,
}

/// Group-policy ratio when no prior policy year is known at calendar-year start
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
pub enum CareFirstYearFunding {
    /// Reasonable estimate for the first policy year
    Estimated {
        /// Actual premium estimate for this coverage
        premiums: CareFundingAmounts,
    },
    /// Actual net premiums ascertained during this calendar year
    Actual {
        /// Actual net premiums for this coverage
        premiums: CareFundingAmounts,
    },
}

/// Who funded this accident/health insurance
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
pub enum CareBenefitFunding {
    /// Entirely after-tax employee premiums
    /// Include employer-paid premiums already included in your taxable wages. Benefits are not attributed to tax-free employer payments.
    AfterTax,
    /// Entirely tax-free employer payments
    EmployerOnly,
    /// Individual insurance policy funded by both
    Individual {
        /// Current policy year's actual net premiums
        premiums: CareFundingAmounts,
    },
    /// Group insurance policy funded by both
    Group {
        /// Latest policy years known at the start of2026
        /// Enter all available years up to the latest three. If none is known, leave this empty and choose First-year funding below.
        // Treas. Reg. 1.105-1(d): group-policy funding uses at most the latest three policy years.
        #[libertas_size(max = 3)]
        years: Vec<CareFundingYear>,
        /// Funding when no prior policy year is known
        /// [NullText]
        /// Use the policy-year funding entered above
        /// [SetText]
        /// The first-year policy funding is
        #[libertas_default($null)]
        first_year: Option<CareFirstYearFunding>,
    },
}
impl CareBenefitFunding {
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let zero = |p: &CareFundingAmounts| {
            whole(p.employee_premium) + whole(p.employer_premium) <= whole(0)
        };
        let zero_issue = || {
            ValidationIssue::new("LTC_FUNDING_ZERO")
                .at_field("employee_premium")
                .in_field("premiums")
        };
        match self {
            Self::Individual { premiums } if zero(premiums) => return Some(zero_issue()),
            Self::Group { years, first_year } => {
                if let Some((index, _)) = years.iter().enumerate().find(|(_, y)| zero(&y.premiums))
                {
                    return Some(zero_issue().in_list("years", index));
                }
                if first_year.as_ref().is_some_and(|f| match f {
                    CareFirstYearFunding::Estimated { premiums }
                    | CareFirstYearFunding::Actual { premiums } => zero(premiums),
                }) {
                    return Some(zero_issue().in_field("first_year"));
                }
            }
            _ => {}
        }
        let invalid_premiums = |p: &CareFundingAmounts| {
            ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID")
                .at_field(if p.employee_premium < 0 {
                    "employee_premium"
                } else {
                    "employer_premium"
                })
                .in_field("premiums")
        };
        match self {
            Self::AfterTax | Self::EmployerOnly => None,
            Self::Individual { premiums } => {
                (!premiums.valid()).then(|| invalid_premiums(premiums))
            }
            Self::Group { years, first_year } => {
                // Reg. 1.105-1(d): the group-policy ratio uses no more than three policy years.
                if years.len() > 3 {
                    return Some(
                        ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID").at_field("years"),
                    );
                }
                for (index, y) in years.iter().enumerate() {
                    if !(1900..=crate::HEALTH_TAX_YEAR).contains(&y.year) {
                        return Some(
                            ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID")
                                .at_field("year")
                                .in_list("years", index),
                        );
                    }
                    if !y.premiums.valid() {
                        return Some(invalid_premiums(&y.premiums).in_list("years", index));
                    }
                    if years[..index].iter().any(|v| v.year == y.year) {
                        return Some(
                            ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID")
                                .at_field("year")
                                .in_list("years", index),
                        );
                    }
                }
                match first_year {
                    Some(
                        CareFirstYearFunding::Estimated { premiums }
                        | CareFirstYearFunding::Actual { premiums },
                    ) => {
                        if !years.is_empty() {
                            Some(
                                ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID")
                                    .at_field("first_year"),
                            )
                        } else {
                            (!premiums.valid())
                                .then(|| invalid_premiums(premiums).in_field("first_year"))
                        }
                    }
                    None => years.is_empty().then(|| {
                        ValidationIssue::new("LTC_FUNDING_HISTORY_INVALID").at_field("years")
                    }),
                }
            }
        }
    }
    fn employer_fraction(&self) -> Option<(Exact, Exact)> {
        if !self.valid() {
            return None;
        }
        let (employer, total) = match self {
            Self::AfterTax => return Some((whole(0), whole(1))),
            Self::EmployerOnly => return Some((whole(1), whole(1))),
            Self::Individual { premiums } => (
                whole(premiums.employer_premium),
                whole(premiums.employee_premium) + whole(premiums.employer_premium),
            ),
            Self::Group { years, first_year: None } => {
                years.iter().fold((whole(0), whole(0)), |(e, t), y| {
                    (
                        e + whole(y.premiums.employer_premium),
                        t + whole(y.premiums.employee_premium) + whole(y.premiums.employer_premium),
                    )
                })
            }
            Self::Group {
                first_year:
                    Some(
                        CareFirstYearFunding::Estimated { premiums }
                        | CareFirstYearFunding::Actual { premiums },
                    ),
                ..
            } => (
                whole(premiums.employer_premium),
                whole(premiums.employee_premium) + whole(premiums.employer_premium),
            ),
        };
        Some((employer, total))
    }
    pub(crate) fn employer_share(&self, received: i64) -> Option<i64> {
        let (employer, total) = self.employer_fraction()?;
        crate::kernel::ratio::exact::rounded(&(whole(received) * employer / total))
    }
    fn permits_portions(&self, gross: i64, portions: &Exact) -> bool {
        self.employer_fraction().is_some_and(|(e, t)| portions * t <= whole(gross) * e)
    }
}

/// Basis of a permanent-injury payment
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
pub enum CareInjuryKind {
    /// Permanent loss of a body member
    PermanentMemberLoss,
    /// Permanent loss of use of a body function
    PermanentFunctionLoss,
    /// Permanent disfigurement
    PermanentDisfigurement,
    /// Another injury or disability
    /// This does not establish section105(c)'s permanent-injury exclusion.
    Other,
}

/// Actual employer-attributable permanent-injury payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CarePermanentInjury {
    /// Portion of the employer-attributable benefit paid for this injury
    /// Do not include medical reimbursement or another payment already listed as an excluded portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Injury on which the payment was based
    pub kind: CareInjuryKind,
    /// Amount was computed solely by the injury, independently of absence from work
    /// Check the benefit determination and the plan’s payment formula.
    pub computed_by_injury: bool,
}
impl CarePermanentInjury {
    fn valid(&self) -> bool {
        self.amount >= 0
    }
    pub(crate) fn excluded(&self) -> i64 {
        if self.computed_by_injury && !matches!(self.kind, CareInjuryKind::Other) {
            self.amount
        } else {
            0
        }
    }
}

/// Actual tax character of nonqualified LTC insurance benefits
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
pub enum NonqualifiedCareTreatment {
    /// Accident or health insurance paid for personal injury or sickness
    /// Use actual policy coverage. This is not a wage-continuation or disability pension statement; those keep their existing W-2/retirement source treatment.
    AccidentHealth {
        /// Actual premium funding
        funding: CareBenefitFunding,
        /// Employer-attributable payment specifically reimbursing eligible medical care
        /// Only actual eligible expenses after the plan began, not amounts received regardless of expenses. Exclude prior-year deductions recovered and permanent-injury payments entered separately. Enter unreimbursed medical costs net of this reimbursement on Medical.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        medical_reimbursement: i64,
        /// Separate permanent-injury payment from the employer-attributable benefit
        #[libertas_default($null)]
        permanent_injury: Option<CarePermanentInjury>,
        /// Recovery of medical expenses deducted on an earlier return
        /// Choose the actual Other income recovery that includes this payment; enter its complete prior-year worksheet there. Empty means none of this payment recovers a prior-year deduction. The existing recovery owner includes any required income once.
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        recovery: Option<i64>,
    },
    /// Payment already included in an actual nonqualified annuity source
    /// Choose the retirement entry whose actual gross distribution includes this very receipt. Its completed annuity calculation retains required income; this link alone does not establish a new LTC exclusion. A life settlement without such a completed source remains Unclassified.
    LifeAnnuity {
        /// Actual annuity source
        #[libertas_number(min = 1)]
        source: i64,
    },
    /// Actual receipt character has not been established
    /// Required taxable income is unresolved, so the complete estimate is unavailable.
    Unclassified,
}

/// LTC insurance contract qualification
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
pub enum LtcContract {
    /// Qualified long-term-care contract
    Qualified {
        /// Actual contract qualification
        /// A pre1997 policy must satisfy the shared issue-date, state-law and material-change facts. A printed box4 check establishes the Qualified branch, not invented policy history.
        qualification: CareContractQualification,
    },
    /// Nonqualified long-term-care contract
    Nonqualified {
        /// Actual payment tax treatment
        treatment: NonqualifiedCareTreatment,
    },
    // A printed 1099-LTC does not establish the actual legal receipt character.
    // Unavailable insurer/source records retain ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED.
    /// Contract qualification is not established
    Unknown,
}

/// Actual tax character of nonqualified LTC insurance benefits
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
pub enum NonqualifiedCareTreatmentPage {
    /// Accident or health insurance paid for personal injury or sickness
    /// Use actual policy coverage. This is not a wage-continuation or disability pension statement; those keep their existing W-2/retirement source treatment.
    AccidentHealth {
        /// Actual premium funding
        funding: CareBenefitFunding,
        /// Employer-attributable payment specifically reimbursing eligible medical care
        /// Only actual eligible expenses after the plan began, not amounts received regardless of expenses. Exclude prior-year deductions recovered and permanent-injury payments entered separately. Enter unreimbursed medical costs net of this reimbursement on Medical.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        medical_reimbursement: i64,
        /// Separate permanent-injury payment from the employer-attributable benefit
        /// [NullText]
        /// No separate permanent-injury payment was received
        /// [SetText]
        /// The separate permanent-injury payment is
        permanent_injury: Option<CarePermanentInjury>,
        /// Recovery of medical expenses deducted on an earlier return
        /// Choose the actual Other income recovery that includes this payment; enter its complete prior-year worksheet there. Empty means none of this payment recovers a prior-year deduction. The existing recovery owner includes any required income once.
        /// [NullText]
        /// None of this payment recovers an earlier medical deduction
        /// [SetText]
        /// The Other income recovery including this payment is
        #[libertas_enum_source("$.recovery_sources")]
        recovery: Option<i64>,
    },
    /// Payment already included in an actual nonqualified annuity source
    /// Choose the retirement entry whose actual gross distribution includes this very receipt. Its completed annuity calculation retains required income; this link alone does not establish a new LTC exclusion. A life settlement without such a completed source remains Unclassified.
    LifeAnnuity {
        /// Actual annuity source
        #[libertas_enum_source("$.annuity_sources")]
        source: i64,
    },
    /// Actual receipt character has not been established
    /// Required taxable income is unresolved, so the complete estimate is unavailable.
    Unclassified,
}

/// LTC insurance contract qualification
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
pub enum LtcContractPage {
    /// Qualified long-term-care contract
    Qualified {
        /// Actual contract qualification
        /// A pre1997 policy must satisfy the shared issue-date, state-law and material-change facts. A printed box4 check establishes the Qualified branch, not invented policy history.
        qualification: CareContractQualification,
    },
    /// Nonqualified long-term-care contract
    Nonqualified {
        /// Actual payment tax treatment
        treatment: NonqualifiedCareTreatmentPage,
    },
    // A printed 1099-LTC does not establish the actual legal receipt character.
    // Unavailable insurer/source records retain ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED.
    /// Contract qualification is not established
    Unknown,
}

impl NonqualifiedCareTreatment {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::AccidentHealth { funding, medical_reimbursement, permanent_injury, recovery } => {
                funding.valid()
                    && *medical_reimbursement >= 0
                    && permanent_injury.as_ref().is_none_or(CarePermanentInjury::valid)
                    && recovery.is_none_or(|id| id > 0)
            }
            Self::LifeAnnuity { source } => *source > 0,
            Self::Unclassified => true,
        }
    }
    pub(crate) fn accident_health(&self, gross: i64) -> Option<(i64, i64)> {
        let Self::AccidentHealth { funding, medical_reimbursement, permanent_injury, .. } = self
        else {
            return None;
        };
        let employer = funding.employer_share(gross)?;
        let injury = permanent_injury.as_ref().map_or(0, |i| i.amount);
        if !funding.permits_portions(gross, &(whole(*medical_reimbursement) + whole(injury))) {
            return None;
        }
        let excluded_injury = permanent_injury.as_ref().map_or(0, CarePermanentInjury::excluded);
        let taxable = employer.checked_sub(*medical_reimbursement)?.checked_sub(excluded_injury)?;
        Some((taxable, gross.checked_sub(taxable)?))
    }
}
impl LtcContract {
    pub(crate) fn qualified(&self) -> bool {
        matches!(self,Self::Qualified { qualification } if qualification.qualified())
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Qualified { qualification } => qualification.valid(),
            Self::Nonqualified { treatment } => treatment.valid(),
            Self::Unknown => true,
        }
    }
}
record_page_conversions!(LtcContract, LtcContractPage, [&], |value, Target, Source| {
    match value {
        Source::Qualified { qualification } => {
            Target::Qualified { qualification: qualification.clone() }
        }
        Source::Nonqualified { treatment } => {
            Target::Nonqualified { treatment: treatment.into() }
        }
        Source::Unknown => Target::Unknown,
    }
});

record_page_conversions!(NonqualifiedCareTreatment, NonqualifiedCareTreatmentPage, [&], |value, Target, Source| {
    match value {
        Source::AccidentHealth { funding, medical_reimbursement, permanent_injury, recovery } => {
            Target::AccidentHealth {
                funding: funding.clone(),
                medical_reimbursement: *medical_reimbursement,
                permanent_injury: permanent_injury.clone(),
                recovery: *recovery,
            }
        }
        Source::LifeAnnuity { source } => {
            Target::LifeAnnuity { source: *source }
        }
        Source::Unclassified => Target::Unclassified,
    }
});

/// Actual independently entered LTC-related source
/// A read-only view of the accepted income record, not another receipt or contract ledger.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct LtcSourceChoice {
    /// Accepted source identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Source label
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Actual source owner
    #[libertas_hidden]
    pub owner: crate::Owner,
    /// Actual source's gross amount
    #[libertas_money("USD", 2)]
    pub gross: i64,
}
