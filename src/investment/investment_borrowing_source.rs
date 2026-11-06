use crate::kernel::phase;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Investment borrowing costs and retained disallowed interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InvestmentBorrowing {
    /// Interest traced to each income source
    #[libertas_size(min = 1)]
    pub loans: Vec<InvestmentLoan>,
    /// Preferential-income election
    pub election: InvestmentElection,
    /// Income produced by the financed investments
    pub income_use: InvestmentInterestUse,
    /// Attribution when the combined deduction is limited
    pub limited_allocation: InvestmentLimitedAllocation,
}
/// Borrowing traced to one income source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentLoan {
    /// Your loan or retained borrowing source label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income source financed by this borrowing
    pub source: i64,
    /// Actual K-1 entity that originated this retained investment-interest balance
    /// This relationship does not change its Schedule A or royalty destination.
    pub originating_k1: Option<i64>,
    /// Eligible interest paid in 2026, excluding amounts deducted elsewhere
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Regular 2025 Form 4952 line 7 attributable to this source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub carry: i64,
    /// Separate retained AMT Form 4952 carryover; do not assume equality
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_carry: i64,
    /// Additional interest on debt financing private-activity bonds
    /// Excluded from regular paid interest above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Person who paid the current portfolio interest
    /// This is independent of who owns an earlier disallowed carryover.
    /// [NullText]
    /// The current payer is not established
    /// [SetText]
    /// The current interest was paid by
    #[libertas_default($null)]
    pub paid_owners: Option<crate::DeductionOwners>,
    /// Person retaining the regular Form 4952 carryover
    /// A K-1 entity that originated the balance does not identify the individual retaining it.
    /// [NullText]
    /// The retained individual owner is not established
    /// [SetText]
    /// The retained carryover belongs to
    #[libertas_default($null)]
    pub carry_owners: Option<crate::DeductionOwners>,
}
/// Attribution of a limited investment-interest deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentLimitedAllocation {
    /// Source facts determine each destination's allowance
    SourceDetermined,
    /// Mixed sources need further allocation records
    UnresolvedMixed,
}
impl InvestmentBorrowing {
    pub(crate) fn aggregate(&self) -> InvestmentInterest {
        InvestmentInterest {
            paid: self.loans.iter().map(|l| l.paid).sum(),
            carryover: self.loans.iter().map(|l| l.carry).sum(),
            amt_carryover: self.loans.iter().map(|l| l.amt_carry).sum(),
            private_activity: self.loans.iter().map(|l| l.private_activity).sum(),
            election: self.election.clone(),
            income_use: self.income_use.clone(),
        }
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.loans
            .iter()
            .enumerate()
            .find(|(i, l)| self.loans[..*i].iter().any(|p| p.label == l.label))
            .map(|(i, _)| {
                crate::validation::ValidationIssue::new("INVESTMENT_LOAN_LABEL_TWICE")
                    .at_field("label")
                    .in_list("loans", i)
            })
    }
    pub(crate) fn valid(&self) -> bool {
        self.admission_issue().is_none()
    }
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let key = "INVESTMENT_BORROWING_RECORDS";
        if self.loans.is_empty() || self.loans.len() > 100 {
            return Some(ValidationIssue::new(key).at_field("loans"));
        }
        if let Some(mut issue) = self.issue() {
            issue.key = key;
            return Some(issue);
        }
        if let Some((index, _)) = self.loans.iter().enumerate().find(|(_, loan)| {
            loan.label.is_empty()
                || ![loan.paid, loan.carry, loan.amt_carry, loan.private_activity]
                    .iter()
                    .all(|value| (0..=100000000000).contains(value))
        }) {
            return Some(ValidationIssue::new(key).at_list_item("loans", index));
        }
        (!self.aggregate().valid()).then(|| ValidationIssue::new(key).at_record())
    }
}

/// Income tax directly connected with investment income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentIncomeTax {
    /// No directly connected income tax paid
    None,
    /// Directly connected portion of the income tax already entered
    Records {
        /// Included part of the state and local income tax paid above
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid_part: i64,
        /// Attribution if the combined tax deduction is capped
        capped_allocation: InvestmentTaxCapAllocation,
    },
    /// Direct connection not established
    Unresolved,
}
/// Source attribution when the tax deduction is capped
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentTaxCapAllocation {
    /// Source facts determine the allowed portion
    SourceDetermined,
    /// The capped mixed components have no established allocation
    UnresolvedMixed,
}
impl InvestmentIncomeTax {
    pub(crate) fn valid(&self, income_paid: i64, income_elected: bool) -> bool {
        match self {
            Self::Records { paid_part, .. } => {
                income_elected && (0..=income_paid).contains(paid_part)
            }
            _ => true,
        }
    }
    // Whole-dollar endpoints for the directly connected allowed deduction.
    // No undocumented pro-rata assignment of a capped tax payment is made.
    pub(crate) fn allowed_bounds(
        &self,
        income_paid: i64,
        salt_paid: &crate::kernel::ratio::exact::Exact,
        salt_allowed: i64,
    ) -> [i64; 2] {
        use crate::kernel::ratio::dollars;
        use crate::kernel::ratio::exact;
        let connected = match self {
            Self::None => return [0; 2],
            Self::Records { paid_part, .. } => dollars(*paid_part),
            Self::Unresolved => {
                return [0, phase::cap(salt_allowed, [dollars(income_paid)])];
            }
        };
        [
            exact::rounded(&phase::floor(
                exact::whole(salt_allowed)
                    - phase::floor(salt_paid - exact::whole(connected), exact::whole(0)),
                exact::whole(0),
            ))
            .expect("SALT endpoint is bounded by the native allowed deduction"),
            phase::cap(salt_allowed, [connected]),
        ]
    }
}

/// Investment borrowing costs
/// The combined eligible interest traced on your borrowing records. Personal, mortgage, passive-activity, capitalized and tax-exempt investment interest is excluded. Source attribution determines whether an allowed amount belongs on Schedule A or Schedule E. Do not repeat interest deducted elsewhere; keep loan-use allocation records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentInterest {
    /// Eligible interest paid in 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Regular-tax disallowed interest from 2025 Form 4952 line 7
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub carryover: i64,
    /// AMT disallowed interest from the separate 2025 AMT Form 4952 line 7
    /// This can differ from the regular-tax carryforward. Use zero only when the AMT records establish none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt_carryover: i64,
    /// Additional interest attributable to private-activity bonds taxable for AMT
    /// Exclude this amount from eligible regular-tax interest above. Include only interest deductible if the bond income were taxable; ordinary tax-exempt bond interest does not qualify.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Preferential-income election
    pub election: InvestmentElection,
    /// Income produced by the financed investments
    /// Trace the use of loan proceeds and retained carryforward records. This identifies expenses for the child-tax worksheets; it does not change the Form 4952 income limit.
    pub income_use: InvestmentInterestUse,
}

impl InvestmentInterest {
    pub(crate) fn valid(&self) -> bool {
        [self.paid, self.carryover, self.amt_carryover, self.private_activity]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
            && self.election.valid()
            && self.income_use.valid(self.paid, self.carryover)
    }
}

/// Elect dividends or gains into investment income
/// Elected amounts lose preferential tax rates. Make the election on a timely return; revocation generally requires IRS consent. This App displays the result and does not submit an election. Form 8615 family coordination still requires additional worksheets.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentElection {
    /// No election
    None,
    /// Use net investment capital gains first, then qualified dividends
    CapitalFirst {
        /// Total amount elected
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Choose the allocation
    /// An explicit allocation may matter for foreign-tax-credit coordination. Such foreign-credit calculations still require further worksheets.
    Allocated {
        /// Net investment capital gain elected
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        capital: i64,
        /// Qualified dividends elected
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        dividends: i64,
    },
}
impl InvestmentElection {
    fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::CapitalFirst { amount } => (1..=100000000000).contains(amount),
            Self::Allocated { capital, dividends } => {
                (0..=100000000000).contains(capital)
                    && (0..=100000000000).contains(dividends)
                    && capital + dividends > 0
            }
        }
    }
}

/// Investment-interest income attribution
/// Identify the income connected with the borrowing expense, not the income used to absorb the Form 4952 deduction. Preserve attribution of prior disallowed interest from the loan records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentInterestUse {
    /// Entirely ordinary investment income
    /// None of the interest is directly connected with qualified dividends or net capital gain.
    Ordinary,
    /// Entirely qualified dividends or net capital gain
    Preferential,
    /// Both categories
    Mixed {
        /// Current interest connected with qualified dividends or net capital gain
        /// The remainder of eligible current interest relates to ordinary investment income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid: i64,
        /// Prior regular-tax carryforward connected with qualified dividends or net capital gain
        /// The remainder of the regular carryforward relates to ordinary investment income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carryover: i64,
    },
    /// Attribution not established
    /// Child-tax results may require these records when preferential income is present. Ordinary adult calculations do not require this child-tax allocation.
    Unresolved,
}
impl InvestmentInterestUse {
    fn valid(&self, current: i64, prior: i64) -> bool {
        match self {
            Self::Mixed { paid, carryover } => {
                (0..=current).contains(paid) && (0..=prior).contains(carryover)
            }
            _ => true,
        }
    }
}
