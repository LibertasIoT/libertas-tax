// Accepted health contribution classifications.
use crate::kernel::source_fact::SourceCents;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Account reported in box 6
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthContributionKind {
    /// HSA
    #[libertas_form_code("HSA")]
    Hsa,
    /// Archer MSA
    Archer,
    /// Medicare Advantage MSA
    MedicareAdvantage,
}

/// Actual W-2 contribution included in this custodian statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementW2Part {
    /// Accepted W-2 identity
    pub income: i64,
    /// Current-year designated principal included in this statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
/// Qualified IRA funding included in this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementFunding {
    /// Actual Form1099-R payment
    pub income: i64,
    /// Actual funding date already entered below
    #[libertas_date_only]
    pub funded: u32,
}
/// Failed-rollover cash included in this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementDeposit {
    /// Original Form1099-SA statement
    pub income: i64,
    /// Actual deposit date already entered on that source
    #[libertas_date_only]
    pub deposited: u32,
}
/// Actual contribution classifications not printed on Form5498-SA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthContributionAllocation {
    /// Accepted Form5498-SA identity
    #[libertas_hidden]
    #[libertas_read_only]
    pub source: i64,
    /// Part of box2 designated for the preceding tax year
    pub calendar_prior_year: SourceCents,
    /// Employer and payroll-excluded current-year principal in box2
    pub employer_calendar: SourceCents,
    /// Employer and payroll-excluded current-year principal in box3
    pub employer_later: SourceCents,
    /// Personal principal from box3 already returned through the existing timely corrections
    /// This classifies existing correction principal by custodian statement; it is not another correction or deduction.
    pub later_correction_personal: SourceCents,
    /// Employer principal from box3 already returned through the existing timely corrections
    pub later_correction_employer: SourceCents,
    /// Current-year W-2 principal included in those employer portions
    /// Select actual accepted sources and their included parts. An empty list establishes that none of these employer deposits is already represented by an entered W-2.
    pub w2: Vec<HealthStatementW2Part>,
    /// Qualified IRA funding included in box2
    /// Choose the actual source and date already recorded on this page; an empty list establishes none. Amounts remain owned by those transfers.
    pub funding: Vec<HealthStatementFunding>,
    /// Failed-rollover ordinary deposits included in boxes2 or3
    /// Choose actual accepted deposits; an empty list establishes none. The existing rollover owner adds these deposits once.
    pub failed_deposits: Vec<HealthStatementDeposit>,
}

impl HealthContributionAllocation {
    pub(crate) fn new(source: i64) -> Self {
        Self {
            source,
            calendar_prior_year: SourceCents::Unavailable,
            employer_calendar: SourceCents::Unavailable,
            employer_later: SourceCents::Unavailable,
            later_correction_personal: SourceCents::Unavailable,
            later_correction_employer: SourceCents::Unavailable,
            w2: Vec::new(),
            funding: Vec::new(),
            failed_deposits: Vec::new(),
        }
    }
}
