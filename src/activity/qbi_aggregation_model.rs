use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Accepted election facts; source group labels alone own membership
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationFacts {
    /// Group name
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    pub name: String,
    /// The same person or group meets the 50% common-ownership test
    /// Check the businesses’ ownership, operating and tax-year records.
    pub common_ownership: bool,
    /// That ownership exists for most of the year including its last day
    /// Check the businesses’ ownership, operating and tax-year records.
    pub majority_year_last_day: bool,
    /// The businesses report the same tax year, ignoring short years
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_tax_year: bool,
    /// Products or services are the same or customarily offered together
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_or_customary_products: bool,
    /// The businesses share facilities or significant centralized elements
    /// Check the businesses’ ownership, operating and tax-year records.
    pub shared_facilities_or_elements: bool,
    /// The businesses operate in coordination or reliance on one another
    /// Check the businesses’ ownership, operating and tax-year records.
    pub coordinated_or_interdependent: bool,
    /// Election and prior-year facts
    pub election: QbiElectionHistory,
    /// Retained IRS restriction for the actual businesses, independent of group label
    pub irs_history: QbiIrsDisaggregationHistory,
    /// Each actual source in the group
    #[libertas_size(min = 0)]
    pub disclosure: Vec<QbiBusinessDisclosure>,
}
/// Actual election history; unavailable or unsupported histories are retained source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiElectionHistory {
    /// First election on a timely original return
    FirstTimelyOriginalReturn,
    /// A first election would be made on an amended return
    FirstOnAmendedReturn,
    /// Continue the prior election
    Continuing {
        /// First elected year
        // Tax structure: this retained history precedes the 2026 tax year.
        #[libertas_number(min = 2018, max = 2025)]
        original_year: u32,
    },
    /// Facts changed significantly; eligibility must be retested
    ChangedFacts {
        /// First elected year
        // Tax structure: this retained history precedes the 2026 tax year.
        #[libertas_number(min = 2018, max = 2025)]
        original_year: u32,
        /// What changed
        #[libertas_size(min = 1)]
        explanation: String,
    },
    /// Election history is unavailable
    Unavailable,
}
/// Retained IRS disaggregation source history, separate from this year's election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiIrsDisaggregationHistory {
    /// Source records establish no IRS disaggregation affecting these businesses
    None,
    /// Source records establish an IRS disaggregation
    Disaggregated {
        /// Tax year of disaggregation
        // Tax structure: the election history cannot postdate the 2026 return.
        #[libertas_number(min = 2018, max = 2026)]
        year: u32,
    },
    /// IRS disaggregation history for these businesses is unavailable
    Unavailable,
}
/// Source EIN facts without treating unknown as no EIN applying
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiEntityEin {
    /// No entity EIN applies to this directly operated business
    NotApplicable,
    /// Actual operating entity EIN
    Known {
        /// Nine digits of the operating entity EIN
        // Legal source identifier: EINs contain exactly nine digits.
        #[libertas_size(min = 9, max = 9)]
        #[libertas_pattern(r"[0-9]{9}")]
        number: String,
    },
    /// Required EIN information is unavailable
    Unavailable,
}
/// Actual business changes for the annual aggregation disclosure
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiBusinessChanges {
    /// None of these changes occurred
    None,
    /// Business formed
    Formed {
        /// Formation date
        #[libertas_date_only]
        // Tax structure: this event occurs in the 2026 tax year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
    },
    /// Business acquired
    Acquired {
        /// Acquisition date
        #[libertas_date_only]
        // Tax structure: this event occurs in the 2026 tax year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
    },
    /// Operations ceased
    Ceased {
        /// Date operations ceased
        #[libertas_date_only]
        // Tax structure: this event occurs in the 2026 tax year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
    },
    /// Business disposed of
    Disposed {
        /// Disposition date
        #[libertas_date_only]
        // Tax structure: this event occurs in the 2026 tax year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
    },
    /// More than one change occurred
    Multiple {
        /// Identify each actual change and its date
        #[libertas_size(min = 1)]
        explanation: String,
    },
}
/// Annual disclosure source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiBusinessDisclosure {
    /// Actual accepted business or reported-QBI-block reference
    pub source: QbiAggregationSource,
    /// Operating entity or business name
    #[libertas_size(min = 1)]
    pub entity_name: String,
    /// Actual EIN applicability or unavailable source information
    pub ein: QbiEntityEin,
    /// Business description
    #[libertas_size(min = 1)]
    pub business_description: String,
    /// Changes during 2026
    pub changes: QbiBusinessChanges,
}

/// Actual business or indivisible reported-QBI-block source reference
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiAggregationSource {
    /// One actual direct business, rental, farm or single-business K-1
    Income {
        /// Accepted parent income identity
        #[libertas_number(min = 1)]
        document: i64,
    },
    /// Actual uniquely named fiduciary section 199A statement
    EstateStatement {
        /// Accepted estate/trust K-1 identity
        #[libertas_number(min = 1)]
        document: i64,
        /// Existing source-owned statement name
        #[libertas_size(min = 1)]
        statement: String,
    },
    /// Actual approved K1RES reported block or business source
    K1Block {
        /// Accepted partnership or S-corporation K-1 identity
        #[libertas_number(min = 1)]
        document: i64,
        /// Actual accepted source-owned block identity; never invented from row position
        #[libertas_number(min = 1)]
        block: i64,
    },
}

impl QbiAggregationSource {
    // A transport key is never decoded into an accepted source. Save resolves
    // it against the candidate's genuine typed catalog instead.
    pub(crate) fn key(&self) -> String {
        match self {
            Self::Income { document } => alloc::format!("income:{document}"),
            Self::EstateStatement { document, statement } => {
                use core::fmt::Write;
                let mut key = alloc::format!("estate:{document}:");
                for byte in statement.as_bytes() {
                    // Writing to String cannot fail.
                    write!(key, "{byte:02x}").expect("String formatting");
                }
                key
            }
            Self::K1Block { document, block } => alloc::format!("k1:{document}:{block}"),
        }
    }
}

pub(crate) fn text_valid(value: &str) -> bool {
    !value.is_empty()
}

impl QbiElectionHistory {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Continuing { original_year } => (2018..=2025).contains(original_year),
            Self::ChangedFacts { original_year, explanation } => {
                (2018..=2025).contains(original_year) && text_valid(explanation)
            }
            _ => true,
        }
    }
}

impl QbiIrsDisaggregationHistory {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Disaggregated { year } => (2018..=2026).contains(year),
            _ => true,
        }
    }

    pub(crate) fn election_available(&self) -> bool {
        match self {
            Self::None => true,
            Self::Disaggregated { year } => {
                self.valid() && year.checked_add(3).is_some_and(|end| 2026 > end)
            }
            Self::Unavailable => false,
        }
    }
}

impl QbiEntityEin {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Known { number } => {
                number.len() == 9 && number.bytes().all(|b| b.is_ascii_digit())
            }
            _ => true,
        }
    }
}

impl QbiBusinessChanges {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Multiple { explanation } => text_valid(explanation),
            Self::Formed { date }
            | Self::Acquired { date }
            | Self::Ceased { date }
            | Self::Disposed { date } => {
                (20260101..=20261231).contains(date) && crate::dates::source_valid_date(*date)
            }
        }
    }
}
