use crate::Owner;
pub use crate::educator_source::{EducatorClaim, EducatorRole};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Educator and coach expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducatorExpenses {
    /// Educators and coaches
    /// Early childhood through grade 12 teachers, instructors, counselors, principals and aides, and interscholastic sports administrators and coaches, who worked at least 900 hours in a school year.
    // Tax structure: one educator expense entry per filer.
    #[libertas_size(max = 2)]
    #[libertas_unordered]
    /// ----
    /// Educator
    pub people: Vec<Educator>,
}
/// Educator
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Educator {
    /// Whose expenses
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Role
    pub role: crate::EducatorRole,
}
