use crate::dates::source_valid_date;
use crate::kernel::phase;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 8396 Mortgage Credit Certificate credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalMortgageCredit {
    /// Current-year qualified certificate, if interest was paid in 2026
    pub current: Option<FederalMortgageCertificate>,
    /// 2023 credit carryforward from 2025 Form 8396 line 16
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carry_three_years_prior: i64,
    /// 2024 credit carryforward from 2025 Form 8396 line 14
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carry_two_years_prior: i64,
    /// 2025 credit carryforward from 2025 Form 8396 line 17
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carry_prior_year: i64,
    /// Saved Home mortgage interest entry for the certified loan
    /// The More credits page selects it from the saved first-lender loans.
    pub loan: Option<i64>,
}

/// Source of mortgage interest on the debt covered by a Mortgage Credit Certificate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageInterestSource {
    /// Certified interest already identified on lender records
    Certified {
        /// Your share of paid interest on certified indebtedness
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Allocate your share of whole-loan interest to certified indebtedness
    WholeLoan {
        /// Your share of interest actually paid on the mortgage in this period
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid_interest: i64,
        /// Certified indebtedness shown on the certificate for this period
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        certified_debt: i64,
        /// Original amount of this period's mortgage loan, before repayments
        /// Use the original loan for its period and the refinancing loan for the replacement period. The IRS ratio does not change as the loan is paid down.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        original_loan_amount: i64,
    },
}
impl MortgageInterestSource {
    pub(crate) fn certified_debt(&self) -> Option<i64> {
        match self {
            Self::Certified { .. } => None,
            Self::WholeLoan { certified_debt, .. } => Some(*certified_debt),
        }
    }

    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Certified { amount } => (0..=100_000_000_000).contains(amount),
            Self::WholeLoan { paid_interest, certified_debt, original_loan_amount } => {
                (0..=100_000_000_000).contains(paid_interest)
                    && (1..=100_000_000_000).contains(certified_debt)
                    && (1..=100_000_000_000).contains(original_loan_amount)
            }
        }
    }

    pub(crate) fn certified_interest(&self) -> i64 {
        match self {
            Self::Certified { amount } => *amount,
            Self::WholeLoan { paid_interest, certified_debt, original_loan_amount } => {
                crate::kernel::ratio::fraction(
                    *paid_interest,
                    phase::cap(*certified_debt, [*original_loan_amount]),
                    (*original_loan_amount).max(1),
                )
            }
        }
    }
}

/// Retained original Mortgage Credit Certificate and replacement facts
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageReissueFacts {
    /// Date the original mortgage was refinanced
    /// Split paid interest before and after this date when refinancing during 2026.
    #[libertas_date_only]
    pub refinanced_on: u32,
    /// Method used to determine the original-loan interest ceiling
    pub original_interest_method: MortgageOriginalInterestMethod,
    /// This is the first refinance and first reissue of this certificate
    /// The reissued certificate and the loan's refinance history show it. A series of refinancings must trace the annual ceiling to the original loan and certificate, not only to the most recent loan, so No leaves the current-year credit out of this estimate.
    pub first_reissue: bool,
    /// Number on the original certificate retained with the mortgage records
    #[libertas_size(min = 1)]
    pub original_number: String,
    /// Original certificate credit rate
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 10, max = 50, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub original_rate: f64,
    /// Your share of interest treated as scheduled for 2026 on original certified debt
    /// Use the original loan's retained schedule or a consistently selected variable-rate hypothetical-loan worksheet, allocating to certified debt. The scheduled method applies old loan terms and required 2026 rate/principal changes as if the loan continued. The hypothetical method uses original certified balance, refinancing APR and replacement maturity. Do not use actual replacement-loan interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_scheduled_interest: i64,
    /// Outstanding certified debt on the original certificate when refinanced
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub original_certified_balance: i64,
    /// Certified debt on the reissued certificate
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub reissued_certified_debt: i64,
    /// Reissue conditions and original schedule confirmed
    /// The reissue was to the same holder(s) for the same main home, entirely replaced the old certificate, and did not retain old certified debt. Confirm the original schedule and both certificate rates. The reissued amount and rate cannot exceed the old certified balance and rate; the App caps the 2026 credit to the amount the original certificate would allow. No leaves the current-year credit out of this estimate.
    pub confirmed: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for MortgageReissueFacts {}

/// Source of the original-loan interest used for the reissued MCC annual ceiling
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageOriginalInterestMethod {
    /// Fixed-rate original mortgage: use its retained payment schedule
    FixedRateSchedule,
    /// Variable-rate original mortgage: use the scheduled-interest method
    VariableRateSchedule {
        /// Retained original-loan terms and 2026 schedule worksheet
        #[libertas_size(min = 1)]
        worksheet_reference: String,
        /// The entered interest applies original loan terms and required rate/principal changes as if refinancing had not occurred
        schedule_confirmed: bool,
        /// This scheduled-interest method was used from the first year of the reissued certificate and on every intervening return
        /// The Forms 8396 filed since the reissue year show it.
        method_consistent: bool,
    },
    /// Variable-rate original mortgage using the separate hypothetical-interest method
    VariableRateHypothetical {
        /// Retained level-payment amortization worksheet for the 2026 hypothetical loan
        #[libertas_size(min = 1)]
        worksheet_reference: String,
        /// Original certified balance when refinanced, used as hypothetical principal
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        principal_at_refinance: i64,
        /// Refinancing loan APR required by Truth in Lending
        #[libertas_number(min = 0, max = 100, step = 0.01)]
        #[libertas_physical_unit("percent")]
        refinance_apr: f64,
        /// Maturity date of the refinancing loan used in the level-payment worksheet
        #[libertas_date_only]
        refinancing_maturity: u32,
        /// The recorded 2026 original-limit interest comes from this level-payment worksheet
        worksheet_confirmed: bool,
        /// This hypothetical method was used from the first reissue year and on every intervening return
        /// The Forms 8396 filed since the reissue year show it.
        method_consistent: bool,
    },
    /// The original-loan interest method is unresolved
    Unreviewed,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for MortgageOriginalInterestMethod {}
impl MortgageOriginalInterestMethod {
    pub(super) fn valid(&self, refinanced_on: u32, original_certified_balance: i64) -> bool {
        match self {
            Self::VariableRateSchedule { worksheet_reference, .. } => {
                !worksheet_reference.trim().is_empty()
            }
            Self::VariableRateHypothetical {
                worksheet_reference,
                principal_at_refinance,
                refinance_apr,
                refinancing_maturity,
                ..
            } => {
                !worksheet_reference.trim().is_empty()
                    && *principal_at_refinance == original_certified_balance
                    && crate::kernel::natural::exact(*refinance_apr, 100)
                        .is_some_and(|apr| (0..=10000).contains(&apr))
                    && source_valid_date(*refinancing_maturity)
                    && *refinancing_maturity > refinanced_on
            }
            _ => true,
        }
    }

    pub(crate) fn supported(&self) -> bool {
        matches!(self, Self::FixedRateSchedule)
            || matches!(
                self,
                Self::VariableRateSchedule {
                    schedule_confirmed: true,
                    method_consistent: true,
                    ..
                }
            )
            || matches!(
                self,
                Self::VariableRateHypothetical {
                    worksheet_confirmed: true,
                    method_consistent: true,
                    ..
                }
            )
    }
}
impl MortgageReissueFacts {
    // Every reissue answer holds; the unreviewed method stays a refusal.
    pub(crate) fn answered(&self) -> bool {
        self.confirmed
            && self.first_reissue
            && (self.original_interest_method.supported()
                || matches!(
                    self.original_interest_method,
                    MortgageOriginalInterestMethod::Unreviewed
                ))
    }
    pub(crate) fn valid_for(&self, current_rate: f64, same_rate: bool) -> bool {
        source_valid_date(self.refinanced_on)
            && self.refinanced_on <= 20261231
            && self
                .original_interest_method
                .valid(self.refinanced_on, self.original_certified_balance)
            && !self.original_number.trim().is_empty()
            && crate::kernel::natural::exact(self.original_rate, 100)
                .is_some_and(|rate| (1000..=5000).contains(&rate))
            && (if same_rate {
                crate::kernel::natural::units(current_rate, 100)
                    == crate::kernel::natural::units(self.original_rate, 100)
            } else {
                crate::kernel::natural::units(current_rate, 100)
                    <= crate::kernel::natural::units(self.original_rate, 100)
            })
            && (0..=100_000_000_000).contains(&self.original_scheduled_interest)
            && (1..=100_000_000_000).contains(&self.original_certified_balance)
            && (1..=self.original_certified_balance).contains(&self.reissued_certified_debt)
    }
}

/// Current-year qualified Mortgage Credit Certificate
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalMortgageCertificate {
    /// Government issuer shown on the certificate
    pub issuer: String,
    /// Certificate number
    pub number: String,
    /// Certificate issue date
    #[libertas_date_only]
    pub issue_date: u32,
    /// Your 2026 mortgage interest and certified-debt allocation
    /// For a reissue during 2026, enter only replacement-period interest here and the earlier period in History. Otherwise enter the full year. Spouses filing jointly combine their share; exclude another owner's share.
    pub interest: MortgageInterestSource,
    /// Credit rate on the certificate
    /// This is the certificate credit rate, not the mortgage interest rate.
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 10, max = 50, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub rate: f64,
    /// Your share of the $2,000 high-rate annual limit
    /// Spouses filing jointly count as one holder. This matters only when the certificate rate exceeds 20%.
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0.01, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub ownership_share: f64,
    /// This is a qualified state/local Mortgage Credit Certificate for your main home in the issuer's jurisdiction, and no interest was paid to a related person
    /// The certificate issued by the state or local agency shows it. No leaves the current-year credit out of this estimate.
    pub eligible: bool,
    /// Original certificate or refinance history
    pub history: MortgageCertificateHistory,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalMortgageCertificate {}

// The 2026 in the variant names is the statutory reissue-rule date, not the return year.
/// Mortgage Credit Certificate history for the 2026 Form 8396 line 3 cap
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageCertificateHistory {
    /// Original certificate; no reissue
    Original,
    /// Reissued after a refinance before 2026
    ReissuedBefore2026 {
        /// Original and replacement certificate facts
        facts: MortgageReissueFacts,
    },
    /// Original and reissued certificates both applied during 2026 at the same credit rate
    ReissuedDuring2026SameRate {
        /// Original and replacement certificate facts
        facts: MortgageReissueFacts,
        /// Your share of interest actually paid before the 2026 refinance
        original_period_interest: MortgageInterestSource,
    },
    /// Original and reissued certificates had different credit rates during 2026
    ReissuedDuring2026DifferentRates {
        /// Original and replacement certificate facts
        facts: MortgageReissueFacts,
        /// Your share of interest actually paid before the 2026 refinance
        original_period_interest: MortgageInterestSource,
    },
    /// Certificate history needs review
    Unreviewed,
}
