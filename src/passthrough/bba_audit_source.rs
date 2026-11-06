use core::ops::{Deref, DerefMut};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Accepted partnership-audit document and its recipient association.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct BbaAuditSource {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Printed document
    pub document: crate::Form8986Paper,
    /// Personal records not settled by Form 8986
    pub answers: crate::BbaAuditAnswers,
    /// Accepted recipient association
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: crate::DocumentRecipient,
    /// How the App assigned the owner
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub owner_source: Option<crate::OwnerSource>,
    /// Your share of a joint document
    /// Percent of the document's amounts that are yours; your spouse's share is the rest.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub joint_share: Option<i32>,
}

impl From<crate::Form8986Paper> for BbaAuditSource {
    fn from(document: crate::Form8986Paper) -> Self {
        Self {
            id: 0,
            document,
            answers: crate::BbaAuditAnswers::default(),
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        }
    }
}

impl Deref for BbaAuditSource {
    type Target = crate::Form8986Paper;

    fn deref(&self) -> &Self::Target {
        &self.document
    }
}

impl DerefMut for BbaAuditSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.document
    }
}
