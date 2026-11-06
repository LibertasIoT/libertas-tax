use crate::common::{Message, MessageArg, Owner};
use crate::navigation::TaxSection;
use crate::section_labels::section_message;
#[cfg(test)]
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// People on the original return
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RepaymentFilers {
    /// You only
    Taxpayer,
    /// Current spouse only
    Spouse,
    /// You and your current spouse together
    Both,
}

// Runtime issue ownership. Record identity is independent of its translated label.
// Every context names the page that holds the facts to add or fix, so each
// estimate issue on the review links there; Interview opens the first
// unfinished page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IssueContext {
    Section(TaxSection),
    Income {
        id: i64,
        label: Message,
    },
    Record {
        section: TaxSection,
        identity: NoteIdentity,
        label: Message,
    },
    Corrective {
        source: alloc::boxed::Box<IssueContext>,
        sections: Vec<TaxSection>,
    },
    Scoped {
        source: alloc::boxed::Box<IssueContext>,
        identity: NoteIdentity,
        label: Option<Message>,
    },
    Business,
    Section1231,
    ScorpAlternativeMinimum,
    Underpayment,
    Interview,
}
impl IssueContext {
    pub(crate) fn corrective(self, section: TaxSection) -> Self {
        self.alternatives(&[section])
    }
    pub(crate) fn alternatives(self, sections: &[TaxSection]) -> Self {
        Self::Corrective { source: alloc::boxed::Box::new(self), sections: sections.to_vec() }
    }
    pub(crate) fn manual(self) -> Self {
        self.alternatives(&[])
    }
    pub(crate) fn source(&self) -> &Self {
        match self {
            Self::Corrective { source, .. } | Self::Scoped { source, .. } => source.source(),
            _ => self,
        }
    }
    fn correction_sections(&self) -> Option<&[TaxSection]> {
        match self {
            Self::Corrective { sections, .. } => Some(sections),
            Self::Scoped { source, .. } => source.correction_sections(),
            _ => None,
        }
    }
    pub(crate) fn identity(&self) -> NoteIdentity {
        match self {
            Self::Corrective { source, .. } => source.identity(),
            Self::Scoped { identity, .. } => identity.clone(),
            Self::Income { id, .. } => NoteIdentity::Income(*id),
            Self::Record { identity, .. } => identity.clone(),
            Self::Section(section) => NoteIdentity::Section(*section),
            Self::Business => NoteIdentity::Context("business"),
            Self::Section1231 => NoteIdentity::Context("section1231"),
            Self::ScorpAlternativeMinimum => NoteIdentity::Context("scorp-amt"),
            Self::Underpayment => NoteIdentity::Context("underpayment"),
            Self::Interview => NoteIdentity::Context("interview"),
        }
    }
    pub(crate) fn income_id(id: i64) -> Self {
        Self::Income {
            id,
            label: Message::new("ISSUE_INCOME_DOCUMENT", alloc::vec![MessageArg::Number(id)]),
        }
    }
    pub(crate) fn label(&self) -> Message {
        match self {
            Self::Income { label, .. } => label.clone(),
            Self::Record { label, .. } => label.clone(),
            Self::Scoped { label: Some(label), .. } => label.clone(),
            Self::Scoped { source, identity: NoteIdentity::Item { position, .. }, .. } => {
                Message::new(
                    "REVIEW_RECORD_ITEM",
                    alloc::vec![
                        MessageArg::Nested(alloc::boxed::Box::new(source.label())),
                        MessageArg::Number(*position as i64 + 1)
                    ],
                )
            }
            Self::Corrective { source, .. } | Self::Scoped { source, .. } => source.label(),
            Self::Section(section) => section_message(*section),
            Self::Business => Message::new("ISSUE_BUSINESS_INCOME_AND_LOSSES", alloc::vec![]),
            Self::Section1231 => Message::new("ISSUE_SECTION_1231", alloc::vec![]),
            Self::ScorpAlternativeMinimum => section_message(TaxSection::PassThrough),
            Self::Underpayment => section_message(TaxSection::Payments),
            Self::Interview => section_message(TaxSection::Review),
        }
    }
}
// Identity belongs to the accepted source, never to its mutable display name.
// Nested positions are scoped to their parent within one accepted revision;
// they are not fabricated persistent record IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoteIdentity {
    Income(i64),
    Section(TaxSection),
    Owner { section: TaxSection, owner: Owner },
    FilingYear { section: TaxSection, year: i32, filers: RepaymentFilers },
    Item { parent: alloc::boxed::Box<NoteIdentity>, collection: &'static str, position: usize },
    Context(&'static str),
}
impl IssueContext {
    pub(crate) fn owner(section: TaxSection, owner: Owner) -> Self {
        Self::Record {
            section,
            identity: NoteIdentity::Owner { section, owner },
            label: Message::new(
                if owner == Owner::Taxpayer { "SAVING_OWNER_YOURS" } else { "SAVING_OWNER_SPOUSE" },
                alloc::vec![],
            ),
        }
    }
    pub(crate) fn item(&self, collection: &'static str, position: usize) -> Self {
        Self::Scoped {
            source: alloc::boxed::Box::new(self.clone()),
            label: None,
            identity: NoteIdentity::Item {
                parent: alloc::boxed::Box::new(self.identity()),
                collection,
                position,
            },
        }
    }
    pub(crate) fn named(self, name: &str) -> Self {
        if name.trim().is_empty() {
            return self;
        }
        self.labeled(Message::new("TEXT", alloc::vec![MessageArg::Text(name.into())]))
    }
    // The same, named by a stored FormattedText label.
    pub(crate) fn named_encoded(self, name: &[u8]) -> Self {
        self.labeled(Message::new("TEXT", alloc::vec![MessageArg::Encoded(name.to_vec())]))
    }
    fn labeled(mut self, name: Message) -> Self {
        match &mut self {
            Self::Scoped { label, .. } => *label = Some(name),
            Self::Corrective { source, .. } => {
                **source = (**source).clone().labeled(name);
            }
            _ => {}
        }
        self
    }
}
// Internal calculation issue. English fixture fields do not enter the App artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FederalIssue {
    pub context: IssueContext,
    #[cfg(test)]
    pub section: String,
    #[cfg(test)]
    pub message: String,
    pub display: Message,
}

/// Internal review note, encoded only at the response boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewNote {
    #[cfg(test)]
    pub english: String,
    pub display: Message,
    pub context: Option<IssueContext>,
}

impl ReviewNote {
    pub fn localized(#[cfg(test)] english: impl Into<String>, message: Message) -> Self {
        Self {
            #[cfg(test)]
            english: english.into(),
            display: message,
            context: None,
        }
    }
    pub(crate) fn scoped(
        #[cfg(test)] english: impl Into<String>,
        message: Message,
        context: IssueContext,
    ) -> Self {
        let mut note = Self::localized(
            #[cfg(test)]
            english,
            message,
        );
        note.context = Some(context);
        note
    }
    pub(crate) fn same_notice(&self, other: &Self) -> bool {
        self.display == other.display
            && self.context.as_ref().map(IssueContext::identity)
                == other.context.as_ref().map(IssueContext::identity)
            && self.context.as_ref().and_then(IssueContext::correction_sections)
                == other.context.as_ref().and_then(IssueContext::correction_sections)
    }
}

#[cfg(test)]
impl core::ops::Deref for ReviewNote {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.english
    }
}

#[cfg(test)]
impl core::fmt::Display for ReviewNote {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.english.fmt(f)
    }
}

#[cfg(test)]
impl PartialEq<str> for ReviewNote {
    fn eq(&self, other: &str) -> bool {
        self.english == other
    }
}

#[cfg(test)]
impl PartialEq<String> for ReviewNote {
    fn eq(&self, other: &String) -> bool {
        self.english == *other
    }
}

// Affected-record names are display data. The typed identity still owns dedup
// and routing when two records have the same name or a name changes.
// Whether the message already names the record its context labels.
pub(crate) fn message_names_label(message: &Message, label: &Message) -> bool {
    message.args.iter().any(|arg| *arg == MessageArg::Nested(alloc::boxed::Box::new(label.clone())))
        || (label.key == "TEXT"
            && label.args.first().is_some_and(|label| message.args.first() == Some(label)))
}
pub(crate) fn contextual_message(message: &Message, context: &IssueContext) -> Message {
    if message_names_label(message, &context.label()) {
        return message.clone();
    }
    if matches!(context.source(), IssueContext::Income { .. } | IssueContext::Record { .. })
        || matches!(context.identity(), NoteIdentity::Item { .. })
    {
        Message::new(
            "REVIEW_RECORD_STATUS",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(context.label())),
                MessageArg::Nested(alloc::boxed::Box::new(message.clone())),
            ],
        )
    } else {
        message.clone()
    }
}
