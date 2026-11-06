pub(crate) use crate::document_parties::{AcceptedPartySource, Assignment};
pub use crate::document_parties::{
    DocumentOwner, DocumentRecipient, OwnerSource, PrintedDocumentNames,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A person's documents in this section
/// Derived from each document's stored owner on every visit; never stored.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("DocumentOwnershipOptions")]
pub struct PersonDocuments {
    /// Person on this return
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub person: i64,
    /// Person
    #[libertas_hidden]
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: alloc::vec::Vec<u8>,
    /// Documents
    /// The documents with this person's printed name stay theirs. Add a document without an owner; only a document you added can be removed, and it returns to the documents without an owner.
    /// ----
    /// Document
    #[libertas_dynamic(r#"Removable = [owner_source] is Added;"#)]
    pub documents: alloc::vec::Vec<SectionDocument>,
    /// Document summaries
    /// Every accepted document in this section, including documents already assigned to another person.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub choices: alloc::vec::Vec<crate::RecordChoice>,
    /// Documents available to this person
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub available_documents: alloc::vec::Vec<i64>,
    /// Income share from these documents
    /// The complete preview of this person's assigned income documents; absent while any preview is unavailable.
    /// [NullText]
    /// Not available until every document's income can be previewed
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_default($null)]
    pub income_share: Option<i64>,
}

impl PersonDocuments {
    pub(crate) fn needs_joint_share(
        draft: &crate::FederalDraft,
        section: crate::TaxSection,
    ) -> bool {
        // Joint-capable income statements live here. Mortgage interest follows
        // its own actual-paid-interest answer, not an ownership percentage.
        if section != crate::TaxSection::InterestDividends {
            return false;
        }
        let situations = draft.whole_return();
        let separate_niit = draft.setup.as_ref().is_some_and(|setup| {
            setup.filing_status() == crate::FederalStatus::Joint
                && situations & crate::SpouseSituations::NONRESIDENT_SPOUSE.0 != 0
                && situations & crate::SpouseSituations::NIIT_JOINT_ELECTION.0 == 0
        });
        draft
            .setup
            .as_ref()
            .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Separate)
            || crate::injured_spouse_sources::requested(draft)
            || separate_niit
    }
}

/// Document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SectionDocument {
    /// Document
    /// Choose one of the documents without an owner.
    #[libertas_ask_on_add]
    #[libertas_calculation_request("DocumentOwnershipOptions")]
    #[libertas_enum_source("^<PersonDocuments>.choices")]
    #[libertas_constrained_by("^<PersonDocuments>.available_documents")]
    #[libertas_unique(1)]
    #[libertas_dynamic(r#"Editable = [^/owner_source] is Added;"#)]
    pub document: i64,
    /// Owned jointly?
    #[libertas_constrained_by("^<SectionDocument>.allowed_ownership")]
    #[libertas_dynamic(
        r#"Editable = [^/owner_source] is Added || ([] is Joint && [^/joint_share_needed]);"#
    )]
    #[libertas_default(Yours)]
    pub ownership: AddedOwnership,
    /// Edit the original document
    #[libertas_read_only]
    #[libertas_output_only]
    pub edit_document: alloc::vec::Vec<DocumentEditTarget>,
    /// Ownership choices for this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_document_ownership")]
    pub allowed_ownership: alloc::vec::Vec<i32>,
    /// Whether a joint share affects this return's figures
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_default($null)]
    pub joint_share_needed: Option<bool>,
    /// How the App assigned the owner
    /// Matched: the printed name is this person's, so it stays. Added: you added it here.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Added)]
    pub owner_source: OwnerSource,
}

/// Selected document for this person's ownership choices
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DocumentOwnerSelection {
    /// Document
    pub document: i64,
}

/// Ownership choices for one selected document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct DocumentOwnershipDerived {
    /// Ownership choices for this document
    #[libertas_hidden]
    pub allowed_ownership: alloc::vec::Vec<i32>,
    /// Whether a joint share affects this return's figures
    #[libertas_hidden]
    pub joint_share_needed: Option<bool>,
}

/// Owned jointly?
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
pub enum AddedOwnership {
    /// Not joint
    Yours,
    /// Joint
    /// You and your spouse own it together.
    Joint {
        /// This spouse's share
        #[libertas_number(min = 0, max = 100)]
        #[libertas_physical_unit("percent")]
        #[libertas_default(50)]
        #[libertas_dynamic(r#"Editable = [^^/joint_share_needed];"#)]
        share: i32,
    },
}

impl AcceptedPartySource for crate::Form1098CDocument {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.form.donor.clone(),
            issuer: Some(self.form.donee.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::Mortgage1098Source {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.form.borrower.clone(),
            issuer: Some(self.form.lender.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::MortgageLoan {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        self.printed.clone()
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        let Some(parties) = &mut self.printed else {
            return false;
        };
        parties.set_assignment(recipient);
        true
    }
}

impl AcceptedPartySource for crate::EducationSource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.form.student.clone(),
            issuer: Some(self.form.filer.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::MarketplacePolicyRecord {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        self.printed.clone()
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        let Some(parties) = &mut self.printed else {
            return false;
        };
        parties.set_assignment(recipient);
        true
    }
}

impl AcceptedPartySource for crate::FederalTaxCreditBond1097 {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        self.printed.clone()
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        let Some(parties) = &mut self.printed else {
            return false;
        };
        parties.set_assignment(recipient);
        true
    }
}

impl AcceptedPartySource for crate::IsoExerciseSource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.form.employee.clone(),
            issuer: Some(self.form.transferor.clone()),
            transfer_corporation: Some(self.form.corporation.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::StudentLoanSource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.document.borrower.clone(),
            issuer: Some(self.document.lender.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::VehicleLoanSource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.document.payer_of_record.clone(),
            issuer: Some(self.document.lender.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::BbaAuditSource {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.document.partner.clone(),
            issuer: Some(self.document.partnership.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl AcceptedPartySource for crate::EsppAcquisition {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(PrintedDocumentNames {
            recipient: self.form.employee.clone(),
            issuer: Some(self.form.corporation.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

/// Open the original document without submitting this page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct DocumentEditTarget {
    /// Edit document
    #[libertas_formatted_text]
    pub action: alloc::vec::Vec<u8>,
    /// Original document in this editor
    #[libertas_hidden]
    #[libertas_reveal_path]
    pub field: Option<alloc::string::String>,
}
