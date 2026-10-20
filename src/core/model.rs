use crate::*;
use libertas::LibertasUser;
use libertas::data::{KeyCodec, KeyEncodeError};
use libertas_macros::{
    LibertasAvroDecode, LibertasAvroEncode, LibertasExport, LibertasKeyCodec, LibertasKeyDecode,
};

/// Stored-shape release marker. The release owner advances this before deployment.
/// The stable single-u32 legacy marker stays readable for the isolated format-124
/// rehearsal. Normal keyed loads use the working header's layout generation and
/// never reset a corrupt or unknown keyed return.
pub const STORAGE_FORMAT: u32 = 139;

/// Saved tax return
/// Complete accepted pages; unfinished editor values are never stored.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum TaxAppData {
    // Must remain the first variant with a single `u32` payload so every
    // build can decode the marker. Never declare anything before it.
    /// Stored data format
    StorageFormat {
        /// Format number
        format: u32,
    },
    /// Accepted 2026 return
    Draft {
        /// Retained format-124 answers
        draft: crate::migration124::Draft124,
    },
    /// Household structure and parent-return facts
    Household {
        /// Household facts
        record: HouseholdRecord,
    },
    /// Filer facts under an App person role
    Filer {
        /// App filer role
        role: Owner,
        /// Accepted person facts
        facts: FederalPerson,
    },
    /// Accepted dependent with its original identity and order
    Dependent {
        /// Zero-based accepted order
        position: i64,
        /// Accepted dependent facts and native identity
        facts: FederalDependent,
    },
    /// Accepted return setup section
    SetupSection {
        /// Absent until setup has been accepted
        setup: Option<crate::FederalSetup>,
    },
    /// Accepted topic selection
    SectionSelection {
        /// Selected return topics
        checklist: crate::Checklist,
    },
    /// One accepted income document or semantic brokerage sale
    IncomeDocument {
        /// Zero-based accepted position, independent of native identity
        position: i64,
        /// Entire native accepted source, including its identity and source facts
        entry: FederalIncomeEntry,
    },
    /// Accepted adjustments section
    AdjustmentsSection {
        /// Complete accepted section, including absence
        adjustments: Option<FederalAdjustments>,
    },
    /// Accepted deductions section
    DeductionsSection {
        /// Complete accepted section, including absence
        deductions: Option<FederalDeductions>,
    },
    /// Accepted credits section
    CreditsSection {
        /// Complete accepted section, including absence
        credits: Option<FederalCredits>,
    },
    /// Accepted screening section
    ScreeningSection {
        /// Complete accepted section, including absence
        screening: Option<FederalScreening>,
    },
    /// Accepted injured spouse section
    InjuredSpouseSection {
        /// Complete accepted section, including absence
        injured_spouse: Option<InjuredSpouseAllocation>,
    },
    /// Accepted less-common review selections
    ReviewSelections {
        /// Income and deduction situations
        situations: alloc::vec::Vec<crate::RareSituation>,
        /// Adjustment review selections
        adjustment_refusals: alloc::vec::Vec<crate::MoreAdjustment>,
        /// Credit review selections
        credit_refusals: alloc::vec::Vec<crate::MoreCreditRefusal>,
        /// Other-tax review selections
        tax_refusals: alloc::vec::Vec<crate::OtherTaxRefusal>,
    },
    /// Beneficiary-owned Coverdell excess records
    CoverdellExcessSection {
        /// Complete accepted account records in their original order
        coverdell_excess: alloc::vec::Vec<crate::CoverdellExcess>,
    },
    /// Accepted payment facts
    PaymentsSection {
        /// Complete accepted section, including absence
        payments: Option<crate::FederalPayments>,
    },
    /// Complete working return metadata
    WorkingHeader {
        /// Storage layout and accepted revision
        header: WorkingHeader,
    },
    /// Accepted business-use home sources
    HomeOfficesSection {
        /// Complete accepted home sources and claims in their original order
        home_sites: alloc::vec::Vec<crate::HomeOfficeSite>,
    },
}

/// Household record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdRecord {
    /// Accepted filer-page presence and household facts
    pub people: Option<HouseholdPeople>,
    // Derived storage completeness: detects a missing final row as well as gaps.
    // Not a source fact, financial count limit, or working-header state.
    /// Number of accepted dependent records
    pub dependent_count: i64,
}

/// Household facts shared by the filers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdPeople {
    /// Parent return for Form 8615
    pub parent_tax: Option<ParentTax>,
    /// Spouse facts are present
    pub spouse_present: bool,
}

/// Working return header; copy namespaces are reserved for a future filing owner.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorkingHeader {
    /// Semantic record layout
    pub layout: u32,
    /// Actual return year
    pub tax_year: i32,
    /// Non-reusing native identity allocator
    pub next_id: i64,
    /// Return identity
    pub cookie: alloc::string::String,
    /// Accepted revision
    pub revision: i64,
    /// Completed sections in accepted order
    pub finished_sections: alloc::vec::Vec<crate::TaxSection>,
    /// Review completed
    pub finished: bool,
    /// Presence of the eight optional accepted sections, in draft declaration order
    pub section_presence: u16,
    /// Accepted income row count
    pub income_count: i64,
    /// Next positive immutable filed-copy identity, separate from native facts
    pub next_filed_copy: i64,
}

/// Saved return keys
/// Working returns use the user/year namespace. The original marker and draft
/// keys remain reserved for the isolated legacy migration.
#[derive(Clone, Debug, PartialEq, Eq, LibertasExport, LibertasKeyCodec, LibertasKeyDecode)]
pub enum TaxDataKey {
    /// Stored format marker
    StorageFormat {
        /// Return owner
        user: LibertasUser,
    },
    /// Latest accepted return
    Draft {
        /// Return owner
        user: LibertasUser,
    },
    // Fixed kind assignments: 1 header; 2 household; 3 filer (id 1 taxpayer,
    // 2 spouse, distinct from Hub users); 4 dependent; 5 setup; 6 checklist;
    // 7 income document/semantic sale; 8 adjustments; 9 deductions; 10 credits;
    // 11 screening; 12 injured spouse; 13 review selections; 14 Coverdell;
    // 15 payments; 16 business-use homes. All scalar sections use id 0;
    // dependents/income use native IDs.
    // Never reuse a published assignment.
    /// Semantic return record
    ReturnRecord {
        /// Hub-verified return owner
        user: LibertasUser,
        /// Server-authorized return year
        tax_year: i32,
        /// Working return (zero) or immutable filed-copy identity (positive)
        copy: i64,
        /// Fixed record-kind selector
        kind: u16,
        /// Native source/person identity or fixed section selector
        id: i64,
    },
}

impl TaxDataKey {
    /// SDK-encoded prefix covering all working and filed records for one user/year.
    /// Field boundaries are 2 (root), 10 (user), 14 (year), 22 (copy),
    /// 24 (kind), and 32 (identity) bytes. It cannot include either user-only key.
    pub fn return_record_prefix(
        user: LibertasUser,
        tax_year: i32,
    ) -> Result<[u8; 14], KeyEncodeError> {
        let key = Self::ReturnRecord { user, tax_year, copy: 0, kind: 0, id: 0 };
        let mut encoded = [0; 32];
        key.encode_key(&mut encoded)?;
        let mut prefix = [0; 14];
        prefix.copy_from_slice(&encoded[..14]);
        Ok(prefix)
    }
}
