// Archer MSA contribution source facts and intrinsic validation.
use crate::common::Owner;
use crate::dates::source_valid_date as valid_date;
use crate::kernel::ratio::exact::{Exact, whole};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Archer MSA contribution records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ArcherContributions {
    /// No regular Archer MSA contributions or retained excess for either spouse
    None,
    /// Personal or employer contributions for 2026
    Records {
        /// Owners and coverage
        /// Include the spouse's coverage and contributions even if that spouse has no funded MSA or files separately. Use a zero-amount row for coverage-only information.
        // Tax structure: one contribution owner for each of the two spouses.
        #[libertas_size(min = 1, max = 2)]
        owners: Vec<ArcherContributor>,
        /// Family-limit allocation
        sharing: ArcherSharing,
    },
}

/// Annual Archer MSA owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ArcherContributor {
    /// Owner
    pub owner: Owner,
    /// Personal contributions
    /// Enter other original2026-designated principal outside the accepted Forms5498-SA before timely corrections. Exclude failed indirect-rollover deposits entered on Other income; the App adds those linked ordinary cash deposits once. Corrected principal is removed once.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub personal: i64,
    /// Personal principal outside the accepted Forms5498-SA
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub personal_outside: i64,
    /// Employer contributions, W-2 box 12 code R
    /// Enter original 2026-designated principal before the timely corrections listed below. Excluded amounts are not deducted again; taxable excess and corrected principal need W-2 reconciliation.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employer principal designated for 2026 outside entered W-2 sources
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer_outside: i64,
    /// Timely returned excess contributions
    /// Record custodian-completed corrective distributions here, not as ordinary Archer MSA withdrawals. The 2026 contribution totals above include original principal designated for 2026, which the App removes once. A 2025-designated principal belongs on the retained 2025 Form 8853 instead.
    pub corrections: Vec<ArcherCorrection>,
    /// Uncorrected 2025-designated employer excess actually deposited in 2026
    /// Keep it out of the 2026 personal/employer contribution totals above. Its retained 2025 Form 8853 establishes the excess; actual 2026 W-2 wage reporting determines whether it enters 2026 other income.
    pub prior_employer_excess: Vec<ArcherPriorEmployerExcessDeposit>,
    /// Current uncorrected contribution excess for this owner
    /// Leave absent when net deposits after timely corrections fit the rules and no retained prior excess remains. This worksheet computes the deductible amount, taxable employer excess and Form 5329 tax.
    pub excess_accounting: Option<ArcherExcessAccounting>,
    /// Eligibility and coverage on each month's first day
    pub coverage: ArcherEligibility,
    /// Income record IDs for the employers or businesses maintaining the HDHP
    /// Link this owner's W-2 or Form 8919 earned compensation or Schedule C or Schedule F businesses; no pension, deferred compensation or unrelated employer. The selected W-2 box 1 amounts must equal qualifying earned compensation. Business net income and its share of deductible SE tax are calculated by the App. Leave empty only for a coverage-only row or a spouse filing outside this return.
    pub income: Vec<i64>,
}

/// Retained 2025 Form 5329 Archer MSA excess
/// Use the actual prior return after any amendment. The 2026 form imports its excess only when the prior tax was positive.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ArcherPriorExcess {
    /// Retained 2025 Form 5329 line 40 excess contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub line_40: i64,
    /// Retained 2025 Form 5329 line 41 tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub line_41: i64,
}
impl ArcherPriorExcess {
    pub(crate) fn valid(&self) -> bool {
        self.line_40 >= 1 && self.line_41 >= 1
    }
}

/// Timely Archer MSA excess-contribution return
/// Use the custodian's completed corrective distribution (Form 1099-SA code 2), not an ordinary withdrawal, and enter it once. Current contribution totals include 2026-designated original principal; 2025-designated principal belongs only on the retained prior return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ArcherCorrection {
    /// Actual returned-excess statement, if entered on Other income
    /// Link the actual accepted source to avoid counting its earnings again. Leave empty for a correction whose statement is not entered separately.
    #[libertas_default($null)]
    #[libertas_number(min = 1)]
    pub document: Option<i64>,
    /// Original failed indirect-rollover deposit, if this return corrects one
    /// Choose the original Other-income statement, separately from the returned-excess statement above. The contribution date and year must match its actual failed cash deposit. Leave empty only for other manually reported contributions.
    #[libertas_default($null)]
    #[libertas_number(min = 1)]
    pub failed_deposit: Option<i64>,
    /// Tax year for which the original contribution was designated
    // Tax structure: these corrections concern the prior or current return year (2025 or 2026).
    #[libertas_number(min = 2025, max = 2026)]
    pub year: u32,
    /// Date the original principal was actually deposited
    #[libertas_date_only]
    // Tax structure: a 2025 or 2026 designated contribution may be deposited in the following calendar year.
    #[libertas_number(min = 20250101, max = 20271231)]
    pub contributed_on: u32,
    /// Original excess principal returned
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub principal: i64,
    /// Attributable net income returned
    /// A negative amount is a loss reducing cash returned, not the principal removed from contributions. Positive earnings enter income in the withdrawal year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Date the principal and all attributable net income were returned
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub returned: u32,
    /// Applicable return or IRS-relief correction deadline
    /// Use the documented due date including a valid extension or relief, with any timely-filing/amendment conditions already satisfied.
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub deadline: u32,
    /// Source of the corrected contribution
    pub payer: ArcherCorrectionPayer,
}

/// Source of a corrected Archer MSA deposit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ArcherCorrectionPayer {
    /// Personal or other after-tax contribution
    Personal,
    /// Employer or payroll-excluded contribution
    Employer {
        /// Corrected principal already included in deposit-year W-2 wages
        /// Yes requires those wages in Income. No adds principal to other income in the actual deposit year; a 2027 deposit does not add 2026 income. Check the deposit-year W-2 and employer payroll statement.
        in_wages: bool,
    },
}
impl ArcherCorrection {
    // The rules the page checks, checked again on save with the same keys.

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.failed_deposit.is_some() && !matches!(self.payer, ArcherCorrectionPayer::Personal) {
            Some(
                crate::validation::ValidationIssue::new("HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED")
                    .at_field("payer"),
            )
        } else if self.returned < self.contributed_on {
            Some(
                crate::validation::ValidationIssue::new(
                    "ARCHER_CORRECTION_RETURNED_BEFORE_DEPOSIT",
                )
                .at_field("returned"),
            )
        } else if self.deadline < self.contributed_on {
            Some(
                crate::validation::ValidationIssue::new(
                    "ARCHER_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
                )
                .at_field("deadline"),
            )
        } else if whole(self.earnings) + whole(self.principal) < whole(0) {
            Some(
                crate::validation::ValidationIssue::new("ARCHER_CORRECTION_LOSS_ABOVE_PRINCIPAL")
                    .at_field("earnings"),
            )
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.document.is_none_or(|id| id > 0)
            && self
                .failed_deposit
                .is_none_or(|id| id > 0 && matches!(self.payer, ArcherCorrectionPayer::Personal))
            && (2025..=2026).contains(&self.year)
            && valid_date(self.contributed_on)
            && (self.year..=self.year + 1).contains(&(self.contributed_on / 10000))
            && self.principal >= 1
            && self.earnings >= -self.principal
            && valid_date(self.returned)
            && valid_date(self.deadline)
            && self.contributed_on <= self.returned
            && self.contributed_on <= self.deadline
            && (self.year..=2028).contains(&(self.returned / 10000))
    }
    pub(crate) fn eligible(&self) -> bool {
        self.returned <= self.deadline
    }
}
/// Uncorrected prior-year Archer employer excess deposited in 2026
/// This deposit was designated for 2025 and belongs on that year's Form 8853, not the current contribution worksheet. Its taxable principal follows the actual deposit year's W-2 or other income. The employer and custodian designated it for 2025 and the retained Form 8853 shows it above the allowable limit; if the retained 2025 Form 5329 line 41 tax was positive, enter its line 40 excess separately.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ArcherPriorEmployerExcessDeposit {
    /// Actual 2026 Archer MSA deposit date
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub contributed_on: u32,
    /// Last permitted date for a 2025-designated contribution
    /// Use April 15, 2026 or a documented later deadline under applicable relief; an ordinary filing extension alone does not extend the contribution deadline.
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub deadline: u32,
    /// Principal established as uncorrected employer excess on retained 2025 Form 8853
    /// This is the taxable excess part, not the entire code R contribution. Do not also enter it as a 2026-designated contribution or timely correction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Taxable principal already included in 2026 W-2 box 1 wages
    /// Yes requires those wages in Income. No sends principal to Schedule 1 other income. Check the deposit-year W-2 and employer payroll statement.
    pub in_wages: bool,
}
impl ArcherPriorEmployerExcessDeposit {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.contributed_on > self.deadline).then_some(
            crate::validation::ValidationIssue::new("ARCHER_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE")
                .at_field("deadline"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        valid_date(self.contributed_on)
            && self.contributed_on / 10000 == 2026
            && valid_date(self.deadline)
            && self.deadline / 10000 == 2026
            && self.contributed_on <= self.deadline
            && self.amount >= 1
    }
}

/// Owner-specific Archer MSA excess-accounting facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ArcherExcessAccounting {
    /// 2025 excess and tax on the retained Form 5329, if any
    /// [NullText]
    /// No 2025 Archer MSA excess carries into 2026
    /// [SetText]
    /// The retained 2025 excess and tax are
    pub prior_year: Option<ArcherPriorExcess>,
    /// Fair market value on December 31, 2026 of Archer MSAs OUTSIDE the accepted Forms5498-SA
    /// Supply actual other-account values or zero when all accounts are represented by the statements. Box5 is added once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end_value: i64,
    /// Further deposits made in 2027 and designated for 2026
    /// Exclude deposits on accepted Forms5498-SA and source-linked failed indirect-rollover deposits here; the App adds them once. Include other deposits in the personal contribution total above. Form 5329 adds them to year-end value when limiting the 6% tax.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub later_return_year_deposits: i64,
    /// Employer deposits in 2027 designated for 2026
    /// Only the part outside accepted Forms5498-SA; it is part of later deposits and the employer total above. Taxable excess from these deposits belongs to the 2027 return, although it affects the 2026 Form 5329 limit and account value.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub later_employer_deposits: i64,
    /// Current-year employer excess already included in 2026 W-2 wages
    /// Supply only taxable excess from employer deposits actually made in 2026. The rest enters 2026 Schedule 1 other income; later deposits belong to 2027.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer_excess_in_wages: i64,
}
impl ArcherExcessAccounting {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.later_employer_deposits > self.later_return_year_deposits).then_some(
            crate::validation::ValidationIssue::new("ARCHER_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER")
                .at_field("later_employer_deposits"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        self.year_end_value >= 0
            && self.later_return_year_deposits >= 0
            && self.later_employer_deposits >= 0
            && self.employer_excess_in_wages >= 0
            && self.prior_year.as_ref().is_none_or(ArcherPriorExcess::valid)
    }
}

/// Archer MSA monthly eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ArcherEligibility {
    /// Same eligibility and plan all year
    FullYear { coverage: ArcherCoverage },
    /// Eligibility or plan changes during the year
    Monthly {
        /// January through December, exactly twelve entries
        // Tax structure: at most one eligibility or coverage entry per calendar month.
        #[libertas_size(min = 12, max = 12)]
        months: Vec<ArcherCoverage>,
    },
}
/// Archer MSA eligible coverage
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ArcherCoverage {
    /// Not eligible this month
    /// Includes Medicare, disqualifying coverage, dependent status or no eligible HDHP.
    None,
    /// Eligible self-only HDHP
    SelfOnly {
        /// Annual deductible
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductible: i64,
        /// Annual out-of-pocket maximum excluding premiums
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        out_of_pocket: i64,
    },
    /// Eligible family HDHP
    Family {
        /// Annual deductible
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductible: i64,
        /// Annual out-of-pocket maximum excluding premiums
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        out_of_pocket: i64,
    },
}
/// Shared family limit
#[derive(Clone, Copy, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ArcherSharing {
    /// Equal shares when both spouses are eligible
    Equal,
    /// Spouses agree to a different division
    Agreed {
        /// Your share
        /// The spouse receives the remainder. Applies to each month with two eligible spouses and family coverage. Separate filers must agree on complementary shares.
        // Tax structure: the taxpayer share is between zero and 100%.
        #[libertas_number(min = 0, max = 100, step = 0.01)]
        #[libertas_physical_unit("percent")]
        taxpayer: f64,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for ArcherSharing {}
impl ArcherEligibility {
    pub(crate) fn month(&self, index: usize) -> ArcherCoverage {
        match self {
            Self::FullYear { coverage } => *coverage,
            Self::Monthly { months } => months.get(index).copied().unwrap_or(ArcherCoverage::None),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (match self {
            Self::FullYear { .. } => true,
            Self::Monthly { months } => months.len() == 12,
        }) && (0..12).all(|m| match self.month(m) {
            ArcherCoverage::None => true,
            ArcherCoverage::SelfOnly { deductible, out_of_pocket }
            | ArcherCoverage::Family { deductible, out_of_pocket } => {
                [deductible, out_of_pocket].iter().all(|v| *v >= 0)
            }
        })
    }
}
impl ArcherCorrection {
    // The principal returned for 2026 by one payer: the sum the page rules
    // ARCHER_CORRECTED_*_ABOVE_CONTRIBUTED and ARCHER_LATER_*_ABOVE_NET take.
    pub(crate) fn returned_for_return_year(
        corrections: &[ArcherCorrection],
        employer: bool,
    ) -> Exact {
        corrections
            .iter()
            .filter(|v| {
                v.year == 2026
                    && matches!(v.payer, ArcherCorrectionPayer::Employer { .. }) == employer
            })
            .map(|v| whole(v.principal))
            .sum()
    }
}
impl ArcherCorrection {
    pub(crate) fn manual_returned_for_return_year(
        corrections: &[ArcherCorrection],
        employer: bool,
    ) -> Exact {
        corrections
            .iter()
            .filter(|c| {
                c.failed_deposit.is_none()
                    && c.year == 2026
                    && matches!(c.payer, ArcherCorrectionPayer::Employer { .. }) == employer
            })
            .map(|c| whole(c.principal))
            .sum()
    }
}
impl ArcherContributor {
    pub(crate) fn corrected(&self, employer: bool) -> Exact {
        ArcherCorrection::returned_for_return_year(&self.corrections, employer)
    }
    pub(crate) fn net_personal(&self) -> Exact {
        whole(self.personal) - self.corrected(false)
    }
    pub(crate) fn net_employer(&self) -> Exact {
        whole(self.employer) - self.corrected(true)
    }
}

impl ArcherCoverage {
    pub(crate) fn family(self) -> Option<i64> {
        match self {
            Self::Family { deductible, .. } => Some(deductible),
            _ => None,
        }
    }
    pub(crate) fn qualified(self) -> bool {
        match self {
            Self::None => true,
            Self::SelfOnly { deductible, out_of_pocket } => {
                (crate::year_constants::ARCHER_SELF_ONLY_DEDUCTIBLE_MINIMUM_CENTS
                    ..=crate::year_constants::ARCHER_SELF_ONLY_DEDUCTIBLE_MAXIMUM_CENTS)
                    .contains(&deductible)
                    && (deductible
                        ..=crate::year_constants::ARCHER_SELF_ONLY_OUT_OF_POCKET_MAXIMUM_CENTS)
                        .contains(&out_of_pocket)
            }
            Self::Family { deductible, out_of_pocket } => {
                (crate::year_constants::ARCHER_FAMILY_DEDUCTIBLE_MINIMUM_CENTS
                    ..=crate::year_constants::ARCHER_FAMILY_DEDUCTIBLE_MAXIMUM_CENTS)
                    .contains(&deductible)
                    && (deductible
                        ..=crate::year_constants::ARCHER_FAMILY_OUT_OF_POCKET_MAXIMUM_CENTS)
                        .contains(&out_of_pocket)
            }
        }
    }
}
impl ArcherContributions {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Records { owners, sharing, .. } => {
                !owners.is_empty()
                    // Tax structure: at most the taxpayer and spouse.
                    && owners.len() <= 2
                    && !matches!(sharing, ArcherSharing::Agreed { taxpayer } if !crate::kernel::natural::exact(*taxpayer, 100).is_some_and(|share| (0..=10000).contains(&share)))
                    && owners.iter().all(|v| {
                        v.personal >= 0
                            && v.employer >= 0
                            && v.employer_outside >= 0
                            && v.corrections.iter().all(ArcherCorrection::valid)
                            && ArcherCorrection::manual_returned_for_return_year(&v.corrections, false)
                                <= whole(v.personal)
                            && ArcherCorrection::manual_returned_for_return_year(&v.corrections, true)
                                <= whole(v.employer)
                            && v.prior_employer_excess
                                .iter()
                                .all(ArcherPriorEmployerExcessDeposit::valid)
                            && v.excess_accounting.as_ref().is_none_or(|e| {
                                e.valid()
                                    && whole(e.later_return_year_deposits)
                                        <= (whole(v.personal)
                                            - ArcherCorrection::manual_returned_for_return_year(
                                                &v.corrections,
                                                false,
                                            ))
                                            + v.net_employer()
                                    && e.later_employer_deposits <= e.later_return_year_deposits
                                    && whole(e.later_employer_deposits) <= v.net_employer()
                                    && whole(e.later_return_year_deposits)
                                        - whole(e.later_employer_deposits)
                                        <= (whole(v.personal)
                                            - ArcherCorrection::manual_returned_for_return_year(
                                                &v.corrections,
                                                false,
                                            ))
                            })
                            && v.coverage.valid()
                            && v.income.iter().all(|id| *id > 0)
                    })
            }
        }
    }
    pub(crate) fn deposits(&self) -> [Exact; 2] {
        let mut totals = [whole(0), whole(0)];
        if let Self::Records { owners, .. } = self {
            for v in owners {
                totals[usize::from(v.owner == Owner::Spouse)] +=
                    v.net_personal() + v.net_employer();
            }
        }
        totals
    }
}
