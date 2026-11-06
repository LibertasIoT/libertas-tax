use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Person catalogs reserve -1 for the taxpayer, -2 for the spouse and positive
// keys for dependents. -3 represents both spouses on a joint-capable document.
pub(crate) const JOINT_RECIPIENT: i64 = -3;

/// Printed source parties and their accepted association.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_read_only]
pub struct PrintedDocumentNames {
    /// Recipient exactly as printed
    pub recipient: crate::PrintedParty,
    /// Issuer exactly as printed
    #[libertas_default($null)]
    pub issuer: Option<crate::PrintedParty>,
    /// Insured party printed separately from the payment recipient
    #[libertas_default($null)]
    pub insured_party: Option<crate::PrintedParty>,
    /// Fiduciary printed separately from the estate or trust
    #[libertas_default($null)]
    pub fiduciary: Option<crate::PrintedPartyNameAddress>,
    /// Corporation named in box 6 of Form 3921
    #[libertas_default($null)]
    pub transfer_corporation: Option<crate::PrintedParty>,
    /// Accepted association with a person on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: DocumentRecipient,
    /// How the App assigned the owner
    /// Matched: the printed name is a person's on this return, so the owner is locked. Added: a person's section added the document.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub owner_source: Option<OwnerSource>,
    /// Your share of a joint document
    /// Percent of the document's amounts that are yours; your spouse's share is the rest.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub joint_share: Option<i32>,
}

// A document's stored owner, as the App assigned it: by the printed-name
// matcher (Matched, locked) or in a person's section (Added).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Assignment {
    pub owner: DocumentRecipient,
    pub owner_source: Option<OwnerSource>,
    pub joint_share: Option<i32>,
}
impl Assignment {
    // Taxpayer and spouse ownership weights, before any money calculation.
    pub(crate) fn owner_weights(&self) -> Option<([i64; 2], i64)> {
        let weights = match self.owner {
            DocumentRecipient::Taxpayer => [100, 0],
            DocumentRecipient::Spouse => [0, 100],
            DocumentRecipient::Joint => {
                let taxpayer = i64::from(self.joint_share.unwrap_or(50));
                [taxpayer, 100 - taxpayer]
            }
            DocumentRecipient::Dependent { .. } | DocumentRecipient::Unknown => return None,
        };
        Some((weights, 100))
    }
    pub(crate) fn matched(owner: DocumentRecipient) -> Self {
        Self {
            owner,
            owner_source: owner.is_known().then_some(OwnerSource::Matched),
            joint_share: (owner == DocumentRecipient::Joint).then_some(50),
        }
    }
    pub(crate) fn added(owner: DocumentRecipient, joint_share: Option<i32>) -> Self {
        Self {
            owner,
            owner_source: Some(OwnerSource::Added),
            joint_share: (owner == DocumentRecipient::Joint).then(|| joint_share.unwrap_or(50)),
        }
    }
    pub(crate) fn apply_to(
        self,
        assigned: &mut DocumentRecipient,
        owner_source: &mut Option<OwnerSource>,
        joint_share: &mut Option<i32>,
    ) {
        *assigned = self.owner;
        *owner_source = self.owner_source;
        *joint_share = self.joint_share;
    }
}

impl PrintedDocumentNames {
    pub(crate) fn assignment(&self) -> Assignment {
        Assignment {
            owner: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
        }
    }
    pub(crate) fn set_assignment(&mut self, assignment: Assignment) {
        assignment.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
    }
    // Only an unchanged printed name keeps the accepted assignment.
    pub(crate) fn retain_assignment_from(&mut self, accepted: Option<&Self>) {
        self.set_assignment(
            accepted
                .filter(|before| before.recipient.name == self.recipient.name)
                .map(Self::assignment)
                .unwrap_or_default(),
        );
    }
}

impl crate::PrintedParty {
    pub(crate) fn from_name(name: String) -> Self {
        Self { name, ..Self::default() }
    }
}

/// People named as recipients, independently of tax allocation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum DocumentRecipient {
    /// You
    Taxpayer,
    /// Your spouse
    Spouse,
    /// Both spouses
    Joint,
    /// A person supported on this return
    Dependent { person: i64 },
    /// Unknown
    /// No person on this return is assigned yet.
    #[default]
    Unknown,
}
impl DocumentRecipient {
    pub(crate) fn key(self) -> i64 {
        match self {
            Self::Taxpayer => -1,
            Self::Spouse => -2,
            Self::Joint => JOINT_RECIPIENT,
            Self::Dependent { person } => person,
            Self::Unknown => 0,
        }
    }
    pub(crate) fn from_key(key: i64) -> Option<Self> {
        match key {
            -1 => Some(Self::Taxpayer),
            -2 => Some(Self::Spouse),
            JOINT_RECIPIENT => Some(Self::Joint),
            person if person > 0 => Some(Self::Dependent { person }),
            _ => None,
        }
    }
    pub(crate) fn tax_owner(self) -> Option<crate::Owner> {
        match self {
            Self::Taxpayer => Some(crate::Owner::Taxpayer),
            Self::Spouse => Some(crate::Owner::Spouse),
            Self::Joint | Self::Dependent { .. } | Self::Unknown => None,
        }
    }
    pub(crate) fn is_known(self) -> bool {
        self != Self::Unknown
    }
    // The owner of the document entry. A joint or a supported person's
    // document is carried once on your return, as before; its shares are a
    // separate answer.
    pub(crate) fn document_owner(self) -> DocumentOwner {
        match self {
            Self::Spouse => DocumentOwner::Spouses,
            Self::Taxpayer | Self::Joint | Self::Dependent { .. } => DocumentOwner::Yours,
            Self::Unknown => DocumentOwner::Unknown,
        }
    }
}

/// Whose document
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
#[libertas_inline_choices]
pub enum DocumentOwner {
    /// Yours
    Yours,
    /// Your spouse's
    Spouses,
    /// Unknown
    /// No person on this return is assigned yet.
    #[default]
    Unknown,
}
impl DocumentOwner {
    // The owned value the calculation reads. The estimate refuses while any
    // document is Unknown, so an Unknown document never reaches it.
    pub(crate) fn owned(self) -> crate::Owner {
        match self {
            Self::Spouses => crate::Owner::Spouse,
            Self::Yours | Self::Unknown => crate::Owner::Taxpayer,
        }
    }
    pub(crate) fn tax_owner(self) -> Option<crate::Owner> {
        match self {
            Self::Yours => Some(crate::Owner::Taxpayer),
            Self::Spouses => Some(crate::Owner::Spouse),
            Self::Unknown => None,
        }
    }
}
impl From<crate::Owner> for DocumentOwner {
    fn from(owner: crate::Owner) -> Self {
        match owner {
            crate::Owner::Taxpayer => Self::Yours,
            crate::Owner::Spouse => Self::Spouses,
        }
    }
}

/// How the App assigned a document's owner
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum OwnerSource {
    /// The printed name matches the person
    Matched,
    /// Added in the person's section
    Added,
}

pub(crate) trait AcceptedPartySource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames>;
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool;
}
