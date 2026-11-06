// Form 5498-SA as printed (Rev. December 2026). The stored paper; the internal `Form5498Sa` is derived from it.
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 5498-SA, HSA, Archer MSA, or Medicare Advantage MSA Information
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual Form5498-SA. Preserve the printed account kind, year, parties and boxes. This is not Form5498 or Form1099-SA. Store only the street in each party address; preserve room or apartment, city, state, country and postal code in their separate printed fields. Keep blank numeric boxes as zero. Do not infer contribution designations, payroll or IRA funding classifications from the totals. Copies of the same form are one document.",
    duplicate_identity = "@.trustee.federal_id,@.participant.federal_id,@.calendar_year,@.archer_msa_contributions,@.total_contributions,@.subsequent_year_contributions,@.rollover_contributions,@.fair_market_value,@.account_number"
)]
pub struct Form5498SaPaper {
    /// VOID
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub void: bool,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// TRUSTEE'S name, street address and TRUSTEE'S TIN
    #[libertas_llm_doc_field]
    pub trustee: crate::printed_state::PrintedParty,
    /// Trustee's room/suite no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_room_suite: String,
    /// Trustee's city/town
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_city_town: String,
    /// Trustee's state/province
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_state_province: String,
    /// Trustee's country
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_country: String,
    /// Trustee's ZIP/foreign code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_zip_foreign_code: String,
    /// Trustee's telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub trustee_telephone: String,
    /// PARTICIPANT'S name, street address and PARTICIPANT'S TIN
    #[libertas_llm_doc_field]
    pub participant: crate::printed_state::PrintedParty,
    /// Participant's apt. no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub participant_apt_no: String,
    /// Participant's city/town
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub participant_city_town: String,
    /// Participant's state/province
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub participant_state_province: String,
    /// Participant's country
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub participant_country: String,
    /// Participant's ZIP/foreign code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub participant_zip_foreign_code: String,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// 1 Employee's or self-employed person's Archer MSA contributions made in the calendar year and the subsequent year for the calendar year
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub archer_msa_contributions: i64,
    /// 2 Total contributions made in the calendar year
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub total_contributions: i64,
    /// 3 Total HSA or Archer MSA contributions made in the subsequent year for the calendar year
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub subsequent_year_contributions: i64,
    /// 4 Rollover contributions
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rollover_contributions: i64,
    /// 5 Fair market value of HSA, Archer MSA, or MA MSA
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fair_market_value: i64,
    /// 6 HSA, Archer MSA or MA MSA
    #[libertas_llm_doc_field(allow_empty = false)]
    pub account: crate::health_contribution_source::HealthContributionKind,
}

/// Accepted Form 5498-SA and its recipient association
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthStatementSource {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Accepted association with a person on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: crate::document_parties::DocumentRecipient,
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
    /// Form 5498-SA as printed
    #[libertas_llm_doc_field]
    pub form: Form5498SaPaper,
}

impl HealthStatementSource {
    pub(crate) fn retain_assignment(&mut self, accepted: Option<&Self>) {
        accepted
            .filter(|old| old.form.participant.name == self.form.participant.name)
            .map(|old| crate::document_parties::Assignment {
                owner: old.assigned,
                owner_source: old.owner_source,
                joint_share: old.joint_share,
            })
            .unwrap_or_default()
            .apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
    }
}
impl crate::document_parties::AcceptedPartySource for HealthStatementSource {
    fn accepted_parties(&self) -> Option<crate::document_parties::PrintedDocumentNames> {
        Some(crate::document_parties::PrintedDocumentNames {
            recipient: self.form.participant.clone(),
            issuer: Some(self.form.trustee.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        })
    }
    fn set_document_recipient(&mut self, recipient: crate::document_parties::Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}
