// Accepted employer medical-account contribution-year allocations.
use crate::common::Owner;
use crate::validation::ValidationIssue;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

type Rule = &'static str;

/// Employer medical-account contribution kind
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthEmployerKind {
    /// HSA, Form W-2 box 12 code W
    Hsa,
    /// Archer MSA, Form W-2 box 12 code R
    Archer,
}

/// Contribution year of the entered W-2 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HealthEmployerYear {
    // The printed W-2 code does not identify the designated contribution year;
    // a source with no answer is asked first (a required question), never
    // stored as an unknown year.
    /// All designated for 2026
    CurrentYear,
    /// All designated for 2025
    PriorYear,
    /// Split between 2026 and 2025
    Split {
        /// Amount designated for 2026
        /// The rest of the displayed W-2 code amount belongs to 2025; do not enter that source amount again as an outside-W-2 employer contribution.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        current: i64,
    },
}
impl HealthEmployerYear {
    pub(crate) fn checked_current_part(&self, reported: i64) -> Result<i64, ValidationIssue> {
        match self {
            Self::CurrentYear => Ok(reported),
            Self::PriorYear => Ok(0),
            Self::Split { current } if (0..=reported).contains(current) => Ok(*current),
            Self::Split { .. } => {
                Err(ValidationIssue::new("HEALTH_EMPLOYER_SPLIT_ABOVE_REPORTED")
                    .at_field("current"))
            }
        }
    }
    pub(crate) fn current_part(&self, reported: i64) -> Result<i64, Rule> {
        self.checked_current_part(reported).map_err(|issue| issue.key)
    }
}

/// Accepted W-2 medical-account year allocation
/// The source amount is an accepted binding, not another editable contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthEmployerBinding {
    /// Actual accepted income document identity
    pub document: i64,
    /// Actual accepted income owner
    pub owner: Owner,
    /// Actual source code kind
    pub kind: HealthEmployerKind,
    /// Source code amount when this allocation was accepted
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub accepted_amount: i64,
    /// Designated contribution year
    pub allocation: HealthEmployerYear,
}
