use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A physical building not already represented by an accepted building source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ImprovementBuildingFacts {
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    pub description: String,
    pub facts: BuildingUseSource,
}

/// Building character and independently established first-service evidence
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BuildingUseSource {
    /// Building first-service evidence, independent of this improvement's date
    /// [NullText]
    /// Records unavailable
    /// [SetText]
    /// Service evidence
    pub first_service: Option<BuildingFirstService>,
    pub kind: RecoveryBuildingKind,
}

/// Actual building use for recovery classification
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RecoveryBuildingKind {
    Nonresidential,
    ResidentialRental,
    PersonalResidence,
}

/// Documentary first-service date or a strictly-before bound
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BuildingFirstService {
    ExactDate {
        #[libertas_date_only]
        date: u32,
    },
    BeforeDate {
        #[libertas_date_only]
        date: u32,
    },
}

impl BuildingUseSource {
    pub(crate) fn valid(&self) -> bool {
        self.first_service.is_none_or(|evidence| {
            let (BuildingFirstService::ExactDate { date }
            | BuildingFirstService::BeforeDate { date }) = evidence;
            crate::dates::source_valid_date(date)
        })
    }
}

pub(crate) fn building_facts_issue(
    rows: &[ImprovementBuildingFacts],
) -> Option<crate::validation::ValidationIssue> {
    rows.iter().enumerate().find_map(|(index, row)| {
        let field = if row.id < 0 {
            Some("id")
        } else if row.description.trim().is_empty() {
            Some("description")
        } else if !row.facts.valid() {
            Some("facts")
        } else {
            None
        }?;
        Some(
            crate::validation::ValidationIssue::new("ASSET_MACRS_SOURCE_FACTS")
                .at_field(field)
                .in_list("improvement_buildings", index),
        )
    })
}
