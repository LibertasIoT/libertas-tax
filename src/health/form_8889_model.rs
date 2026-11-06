#[allow(unused_imports)]
use super::*;
pub use crate::hsa_source::{FederalHsa, HsaCoverage};

// Each account on the HSA page is a
// projection of one `FederalHsa`; the split is `hsa_family_share`. The task
// derives the accepted employer total from actual R/W year allocations and
// the page's separate outside-source principal.
/// Health and medical savings accounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaPage {
    /// Actual employer statements available for source attribution
    #[libertas_read_only]
    #[libertas_output_only]
    pub employer_statements: Vec<crate::RecordChoice>,
    /// Forms 5498-SA and the actual contribution classifications they do not print
    /// Read the source first. Supply only the missing classifications from custodian, employer and transfer records.
    #[libertas_read_only]
    pub contribution_sources: Vec<crate::HealthContributionSourceRow>,
    /// Employer contributions on entered W-2s
    /// The source and amount come from each accepted W-2. Choose the contribution year shown by employer and custodian records. A prior-year part is not automatically taxable prior excess. Not yet determined retains an incomplete source and prevents a complete estimate.
    #[libertas_read_only]
    /// ----
    /// W-2 contribution
    pub employer_sources: Vec<crate::HealthEmployerSourceRow>,
    /// Failed indirect-rollover cash deposits already entered on Other income
    /// These actual source deposits join the matching account's contribution limits once. Do not include them in manual personal or later-deposit amounts. Corrections below can choose their original statement; this display does not change accepted source facts.
    #[libertas_read_only]
    #[libertas_output_only]
    pub linked_deposits: Vec<crate::HealthFailedDepositChoice>,
    /// Accounts
    // Tax structure: one consolidated HSA contribution account per filer.
    #[libertas_size(max = 2)]
    #[libertas_unordered]
    #[libertas_calculation_request("HsaOptions")]
    /// ----
    /// Account
    pub accounts: Vec<HsaAccount>,
    /// Other filer's HSA eligibility when no account is listed
    /// A married filer making eligible contributions needs the other filer's actual eligibility, even if that person owns no HSA. Choose full-year ineligible when appropriate. It is asked on its own page when needed and not needed when both owners' accounts are listed; a separate return supplies the spouse's context without reporting the spouse's account or contributions here.
    /// [NullText]
    /// Not entered here: it is asked after the accounts are saved, when this return needs it
    /// [SetText]
    /// The other filer's HSA eligibility is
    #[libertas_default($null)]
    #[libertas_calculation_request("HsaOptions")]
    pub other_coverage: Option<HsaEligibilityPage>,
    /// Contribution-limit method
    /// Actual monthly eligibility is the default. December uses the last-month rule and its testing period; it does not turn an ineligible owner into an eligible one.
    #[libertas_default(Monthly)]
    #[libertas_calculation_request("HsaOptions")]
    pub family_method: crate::HsaAllocationMethod,
    /// Family contribution split
    /// Equal shares are the statutory default when both spouses are eligible and either has family coverage. Choose Agreed only for an actual different division; deposits alone do not establish an agreement.
    #[libertas_default(Equal)]
    #[libertas_constrained_by("$.allowed_family_split")]
    pub family_split: FamilySplit,
    /// IRA-to-HSA transfers
    /// A qualified direct transfer from an IRA to an HSA in 2026, once in a lifetime (a second one only after changing from self-only to family coverage). Choose the Form 1099-R of the transfer.
    // Tax structure: each of the two filers has at most two qualified lifetime IRA-to-HSA transfers.
    #[libertas_size(max = 4)]
    #[libertas_unordered]
    /// ----
    /// Transfer
    pub funding: Vec<crate::HsaFundingRow>,
    /// Archer MSA contributions
    /// Form 8853 Part I: personal and employer contributions, the family limit shared by spouses, timely corrections and any excess.
    #[libertas_default(None)]
    pub archer: crate::ArcherContributionsPage,
}

/// Account
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaAccount {
    /// Whose account
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Coverage in 2026
    pub coverage: HsaCoverageYear,
    /// Contributions you made yourself
    /// Original other after-tax2026-designated deposits OUTSIDE the Forms5498-SA above, before timely returns. Exclude payroll and every failed indirect-rollover deposit entered on Other income: the App adds those source-linked cash deposits once. Form8889 line2 includes both.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub personal: i64,
    /// Other employer contributions designated for 2026
    /// Only principal outside BOTH the entered W-2 sources and Forms5498-SA above, including actual 2027 deposits designated for 2026. Include original principal before timely corrections; do not add the entered W-2 amount again.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employer: i64,
    /// Excess contributions returned in time
    /// Check Form 1099-SA (distribution code 2). Split by contribution year and payer.
    /// ----
    /// Correction
    pub corrections: Vec<crate::HsaCorrectionPage>,
    /// Prior-year excess withdrawn late
    /// Even medical use is taxable when the excess is returned late. Enter the full distribution here, not again as a medical or nonmedical withdrawal.
    /// ----
    /// Withdrawal
    pub late_excess: Vec<crate::HsaLateExcessWithdrawal>,
    /// Employer excess for 2025 deposited in 2026
    /// Uncorrected employer deposits made in 2026 that were designated for 2025.
    /// ----
    /// Deposit
    pub prior_employer_excess: Vec<crate::HsaPriorEmployerExcessDeposit>,
    /// Excess contributions to this HSA
    /// Choose “No excess contribution or retained 2025 HSA excess remains” when contributions fit the 2026 limit and no 2025 excess remains.
    /// [NullText]
    /// No excess contribution or retained 2025 HSA excess remains
    /// [SetText]
    /// The HSA excess-contribution accounting is
    pub excess_accounting: Option<crate::HsaExcessAccounting>,
    /// First loss of eligibility since December 2026
    /// [NullText]
    /// No loss of HSA eligibility since December 2026 is reported
    /// [SetText]
    /// HSA eligibility was first lost on
    pub last_month_loss: Option<crate::HsaEligibilityLoss>,
    /// Last-month rule used in 2025
    /// Fill in only if you contributed for all of 2025 as eligible only in December.
    /// [NullText]
    /// The last-month rule was not used for 2025
    /// [SetText]
    /// The 2025 last-month-rule history is
    pub last_month_history: Option<crate::HsaLastMonthHistory>,
    /// IRA-to-HSA transfers made in 2025
    /// Transfers whose testing period runs into 2026.
    // Tax structure: at most two qualified IRA-to-HSA transfers per account owner.
    #[libertas_size(max = 2)]
    /// ----
    /// Transfer
    pub funding_history: Vec<crate::HsaFundingHistory>,
    /// Qualified IRA-to-HSA funding before 2025
    /// Lifetime uses from 2007 through 2024 only. Keep 2025 transfers in the testing-period history above; neither historical source is a current contribution. A prior qualified transfer consumes lifetime use even if its testing period later failed.
    pub uses: Vec<crate::HsaFundingUseHistory>,
}

/// HSA contribution eligibility in 2026
/// Use actual eligibility on each month's first day, including retroactive Medicare enrollment and the qualifying coverage, other-coverage, telehealth and direct-primary-care rules described by the monthly choices. Full-year choices assert eligibility on every first day; they do not mean merely owning an HSA or having any health insurance.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaCoverageYear {
    /// Self-only coverage all year
    SelfOnly,
    /// Family coverage all year
    Family,
    /// Coverage changed during the year
    Changed {
        /// Coverage on the first day of each month
        months: HsaCoverageMonths,
    },
}

/// Other filer's HSA contribution eligibility in 2026
/// Use actual eligibility on each month's first day, including retroactive Medicare enrollment and the qualifying coverage rules described by each coverage choice. This context creates no account or contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaEligibilityPage {
    /// Same eligibility all year
    FullYear {
        /// Eligibility on every month's first day
        coverage: crate::HsaCoverage,
    },
    /// Eligibility changes during the year
    Monthly {
        /// Coverage on the first day of each month
        months: HsaCoverageMonths,
    },
}
impl HsaEligibilityPage {
    pub(crate) fn record(&self) -> crate::HsaEligibility {
        match self {
            Self::FullYear { coverage } => crate::HsaEligibility::FullYear { coverage: *coverage },
            Self::Monthly { months } => {
                crate::HsaEligibility::Monthly { months: months.ordered().to_vec() }
            }
        }
    }
    pub(crate) fn from_record(record: &crate::HsaEligibility) -> Self {
        match record {
            crate::HsaEligibility::FullYear { coverage } => Self::FullYear { coverage: *coverage },
            crate::HsaEligibility::Monthly { months } => {
                Self::Monthly { months: HsaCoverageMonths::from_ordered(months) }
            }
        }
    }
}

/// HSA eligibility on the first day of each month
/// Use eligibility on the first day of each month. Medicare enrollment, including retroactive enrollment, makes that month ineligible. Every month needs its own choice.
// Tax structure: one eligibility entry for each of the twelve calendar months, so the request
// carries a named member per month; the stored record keeps the ordered list of twelve.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct HsaCoverageMonths {
    /// January
    pub january: crate::HsaCoverage,
    /// February
    pub february: crate::HsaCoverage,
    /// March
    pub march: crate::HsaCoverage,
    /// April
    pub april: crate::HsaCoverage,
    /// May
    pub may: crate::HsaCoverage,
    /// June
    pub june: crate::HsaCoverage,
    /// July
    pub july: crate::HsaCoverage,
    /// August
    pub august: crate::HsaCoverage,
    /// September
    pub september: crate::HsaCoverage,
    /// October
    pub october: crate::HsaCoverage,
    /// November
    pub november: crate::HsaCoverage,
    /// December
    pub december: crate::HsaCoverage,
}
impl HsaCoverageMonths {
    /// The twelve months in calendar order, as the stored record keeps them.
    pub(crate) fn ordered(self) -> [crate::HsaCoverage; 12] {
        [
            self.january,
            self.february,
            self.march,
            self.april,
            self.may,
            self.june,
            self.july,
            self.august,
            self.september,
            self.october,
            self.november,
            self.december,
        ]
    }
    /// A stored record's months; a missing month is not eligible (stored records hold twelve).
    pub(crate) fn from_ordered(months: &[crate::HsaCoverage]) -> Self {
        let month = |index: usize| months.get(index).copied().unwrap_or(crate::HsaCoverage::None);
        Self {
            january: month(0),
            february: month(1),
            march: month(2),
            april: month(3),
            may: month(4),
            june: month(5),
            july: month(6),
            august: month(7),
            september: month(8),
            october: month(9),
            november: month(10),
            december: month(11),
        }
    }
}
impl core::iter::FromIterator<crate::HsaCoverage> for HsaCoverageMonths {
    fn from_iter<I: IntoIterator<Item = crate::HsaCoverage>>(months: I) -> Self {
        Self::from_ordered(&months.into_iter().collect::<Vec<_>>())
    }
}
impl From<Vec<crate::HsaCoverage>> for HsaCoverageMonths {
    fn from(months: Vec<crate::HsaCoverage>) -> Self {
        Self::from_ordered(&months)
    }
}
impl core::ops::Index<usize> for HsaCoverageMonths {
    type Output = crate::HsaCoverage;
    fn index(&self, month: usize) -> &crate::HsaCoverage {
        match month {
            0 => &self.january,
            1 => &self.february,
            2 => &self.march,
            3 => &self.april,
            4 => &self.may,
            5 => &self.june,
            6 => &self.july,
            7 => &self.august,
            8 => &self.september,
            9 => &self.october,
            10 => &self.november,
            _ => &self.december,
        }
    }
}
impl core::ops::IndexMut<usize> for HsaCoverageMonths {
    fn index_mut(&mut self, month: usize) -> &mut crate::HsaCoverage {
        match month {
            0 => &mut self.january,
            1 => &mut self.february,
            2 => &mut self.march,
            3 => &mut self.april,
            4 => &mut self.may,
            5 => &mut self.june,
            6 => &mut self.july,
            7 => &mut self.august,
            8 => &mut self.september,
            9 => &mut self.october,
            10 => &mut self.november,
            _ => &mut self.december,
        }
    }
}

/// Family contribution split
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum FamilySplit {
    /// Not applicable
    NotApplicable,
    /// Equally
    Equal,
    /// As you agreed
    Agreed {
        /// Your parts
        #[libertas_number(min = 0)]
        #[libertas_default(1)]
        you: u32,
        /// Your spouse's parts
        #[libertas_number(min = 0)]
        #[libertas_default(1)]
        spouse: u32,
    },
}
