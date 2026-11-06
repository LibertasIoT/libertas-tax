use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Wages
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each employer wage statement: Form W-2 or an actual Form 8919. Copies B, C and 2 of one W-2 are one document. Do not turn a 1099-NEC into Form 8919 without that form. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct WageDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<WageDocumentRow>,
}

/// Wages document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WageDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::WageDocument,
    /// Community or separate
    #[libertas_constrained_by("^<WageDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl WageDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::Wage { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Interest and dividends
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify interest, dividend and original-issue-discount statements, Form 2439 and an actual Form 8815. Distinguish ordinary 1099-DIV amounts from its printed return-of-capital or liquidation details. Do not infer share basis or education expenses. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct InterestDividendDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<InterestDividendDocumentRow>,
}

/// Interest and dividends document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InterestDividendDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::InterestDividendDocument,
    /// Community or separate
    #[libertas_constrained_by("^<InterestDividendDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl InterestDividendDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::InterestDividend { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Investment sales
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify investment-sale statements. A Form 1099-B together with the employer's or plan's stock-plan record for the same shares, such as a plan transaction record, sale lot record or supplemental statement showing ISO, employee stock award or option, ESPP or NUA shares, is one document of that specialized stock-sale kind: list the 1099-B pages and the record pages together as its parts, and never list that 1099-B again as Form1099B or the record as a document of its own. Classify a paying agent's or closing statement of a sale of one corporation's stock that is not a Form 1099-B, such as a stock sale cash closing statement, as QsbsSale whether or not it names qualified small business stock or section 1202: such a statement never states eligibility, so this classification records only the printed sale and leaves eligibility to the topic. A Form 1099-B with no such record is a plain Form1099B, even when its description names employee, option or plan shares. Identify every other actual Form 1099-B or Form 1099-DA as its own printed form, retaining all printed sale, section 1256 and receipt boxes on that one source. Do not choose a personal-record sale or contract entry for a printed broker form. Identify every actual Form 1099-S as Form1099S, whatever property it reports; the form does not state how the property was used, so never decide or infer its use. Recognize Form 3921 and standalone Form 3922 acquisition documents, including an earlier-year Form 3922 supplied for a current sale. A Form 3922 alone does not establish a disposition. Recognize the deferred investment on Form 8997. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct InvestmentSaleDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<InvestmentSaleDocumentRow>,
}

/// Investment sales document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentSaleDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::InvestmentSaleDocument,
    /// Community or separate
    #[libertas_constrained_by("^<InvestmentSaleDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl InvestmentSaleDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::InvestmentSale { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Retirement and Social Security
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1099-R, RRB-1099-R, RRB-1099 or SSA-1099 by its printed title. Keep RRB-1099 Social Security-equivalent benefits separate from RRB-1099-R pensions. Keep each payer and recipient statement separate; do not infer rollover or contribution history. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct RetirementDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<RetirementDocumentRow>,
}

/// Retirement and Social Security document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RetirementDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::RetirementDocument,
    /// Community or separate
    #[libertas_constrained_by("^<RetirementDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl RetirementDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::Retirement { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Self-employment
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1099-NEC or 1099-K. Do not infer a business, expense, payment allocation or employee classification from a payment statement. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct SelfEmploymentDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<SelfEmploymentDocumentRow>,
}

/// Self-employment document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SelfEmploymentDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::SelfEmploymentDocument,
    /// Community or separate
    #[libertas_constrained_by("^<SelfEmploymentDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl SelfEmploymentDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::SelfEmployment { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Unemployment and state refunds
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1099-G once, retaining every printed box on that one paper, including a box 2 refund together with unemployment or agricultural payments. Do not infer last year's deduction. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct GovernmentPaymentDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<GovernmentPaymentDocumentRow>,
}

/// Unemployment and state refunds document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct GovernmentPaymentDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    pub document: crate::Form1099GPaper,
    /// State that paid the box 2 refund
    /// Saving fills it once from the paying agency's printed address when that shows a state; a state already recorded is kept. Otherwise identify the refund-paying agency's state, not your current home state.
    /// [NullText]
    /// The paying agency's printed address shows no state
    #[libertas_default($null)]
    pub refund_state: Option<crate::TaxState>,
    /// Locality that paid a local box 2 refund
    /// The local refund agency or locality; leave empty when none.
    #[libertas_default("")]
    pub refund_locality: alloc::string::String,
    /// Community or separate
    #[libertas_constrained_by("^<GovernmentPaymentDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl GovernmentPaymentDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::GovernmentPayment {
                value: self.document,
                refund_state: self.refund_state,
                refund_locality: self.refund_locality,
            },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Gambling
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form W-2G gambling-winnings statement. Keep distinct winnings statements separate; do not create documents for unreported winnings or gambling losses. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct GamblingDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<GamblingDocumentRow>,
}

/// Gambling document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct GamblingDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    pub document: crate::FormW2GPaper,
    /// Community or separate
    #[libertas_constrained_by("^<GamblingDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl GamblingDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::Gambling(self.document),
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Partnerships, S corporations, estates and trusts
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Schedule K-1 by its parent form: 1065 partnership, 1120-S S corporation or 1041 estate or trust. Keep its attached statements with that K-1. Entity-total summaries, amounts in boxes 1, 2 and 3, and section 199A / QBI statements alone remain the ordinary K-1 kind. Choose a distinct-activity kind only when an attached statement separately identifies activities and allocates K-1 box amounts among them, such as two named businesses sharing box 1. A fiscal-year K-1 remains its printed parent-form kind, including a 2025 form for 10/01/2025 through 09/30/2026. A Schedule K-3 alone is not a K-1. Do not infer basis, participation or loss history. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct K1Documents {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<K1DocumentRow>,
}

/// Partnerships, S corporations, estates and trusts document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1DocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::K1Document,
    /// Community or separate
    #[libertas_constrained_by("^<K1DocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl K1DocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::K1 { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Farming
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1099-PATR cooperative-distribution statement. Do not infer its business or farm association or create farm bookkeeping records. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct FarmingDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<FarmingDocumentRow>,
}

/// Farming document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmingDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    pub document: crate::Patr1099Paper,
    /// Community or separate
    #[libertas_constrained_by("^<FarmingDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl FarmingDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::Farming(self.document),
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Other income
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify printed Forms 1099-MISC, 1099-Q, 1099-SA, 1099-LTC and 1099-QA and actual wildfire or disaster relief payment statements. Distinguish an insurer's 1099-LTC from its supporting care-period records; do not infer care costs, reimbursements or account history. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct OtherIncomeDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<OtherIncomeDocumentRow>,
}

/// Other income document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherIncomeDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::OtherIncomeDocument,
    /// Community or separate
    #[libertas_constrained_by("^<OtherIncomeDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl OtherIncomeDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::OtherIncome { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// Canceled debt
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1099-C cancellation-of-debt or Form 1099-A acquisition or abandonment statement by its printed title. Do not infer insolvency, debt exclusions or property basis. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct CanceledDebtDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<CanceledDebtDocumentRow>,
}

/// Canceled debt document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CanceledDebtDocumentRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose document
    /// The App assigns it from the printed name, or a person's section adds it.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Document
    #[libertas_llm_doc_field]
    #[libertas_ask_on_add]
    pub document: crate::CanceledDebtDocument,
    /// Community or separate
    #[libertas_constrained_by("^<CanceledDebtDocumentRow>.allowed_community")]
    #[libertas_hidden]
    #[libertas_default(NotNeeded)]
    pub community: crate::CommunityShare,
    /// Management of the withholding funds
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    #[libertas_hidden]
    pub refund_management: Option<crate::RefundManagement>,
    /// Available community-property answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.allowed_community")]
    pub allowed_community: Vec<i32>,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Calculated carryover bounds
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<crate::InvestmentCapitalCap>,
    /// Texas source-management answer is needed
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
}

impl CanceledDebtDocumentRow {
    pub(crate) fn into_income(self) -> crate::IncomeDocumentItem {
        crate::IncomeDocumentItem {
            id: self.id,
            owner: self.owner,
            document: crate::IncomeSource::CanceledDebt { value: self.document },
            community: self.community,
            refund_management: self.refund_management,
            allowed_community: self.allowed_community,
            investment_caps: self.investment_caps,
            preview: self.preview,
            refund_management_needed: self.refund_management_needed,
        }
    }
}

/// More credit documents
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CreditDocument {
    /// Form 1097-BTC: tax credit bond
    CreditBond(alloc::boxed::Box<crate::TaxCreditBondDocument>),
    /// Form 8986: partnership audit adjustment
    PartnershipAudit(alloc::boxed::Box<PartnershipAuditRow>),
}

/// Form 8986 and its accepted identity
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this one Form 8986 (Partner's Share of Adjustment(s) to Partnership-Related Item(s)) exactly as printed, every part in the form's own order. Read the Original and Corrected checkboxes, the Tracking Number and the Audit Control Number. Part I: return the one checked box of A (AuditedPartnership, AuditPassThrough, AarPartnership or AarPassThrough) and of B (Form1065, Form1120S, Form1041, or Other with its form number), or null when none is checked. Part II: read the partnership's name, address and TIN into partnership; the PR or DI checkbox, first and last name, address and phone into representative; dates D, E, F and G as dates, returning null for a blank D, E or F; report a blank or illegible G as an error. Part III: read the pass-through entity's name, address and TIN, item C as a date, and items D and E as printed; leave them empty when Part III is blank. Part IV: read the partner's name, address and TIN into partner; the C checkboxes; the one checked D entity type, or null; the tax-exempt checkbox; and every E percentage and F and G amount in its As Reported, Change and Corrected column, returning null for a blank box. Part V: return every filled line in printed order with (a) line number, (b) line title and (c) code as printed, (d), (f), (g) and (h) as signed amounts or null when blank, and (e) as checked; then the Totals and every Applicable Penalties row as printed. Part VI: return each statement with its line number or code and its text. Copy names, identification numbers and addresses exactly, preserving masking, punctuation and leading zeros; use empty text for absent text. Never infer a value that is not printed, and never infer the prior return's wages, taxable income, tax, filing status, the Form 8978 interest or any other fact absent from the form. Report illegible or conflicting values as errors.",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.document.tracking_number,@.document.audit_control_number,@.document.partnership.federal_id,@.document.partner.federal_id,@.document.pass_through.entity.federal_id,@.document.pass_through.issuer_tin,@.document.pass_through.tax_year_end,@.document.review_year_end,@.document.adjustment_year_end,@.document.furnished_on,@.document.items",
    form_date_order = "MonthDayYear"
)]
pub struct PartnershipAuditRow {
    /// Form 8986 as printed
    #[libertas_llm_doc_field]
    pub document: crate::Form8986Paper,
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
}

/// Mortgage interest
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1098 mortgage-interest statement. Collect the lender kind on the same entry for a person or AI Import: use Institution for a lender-issued Form 1098; retain PrivateSeller and NoForm only when the supplied records establish those routes. Do not infer personal use, loan proceeds use or refinancing history. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct MortgageDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<Form1098Document>,
}

/// Form 1098 document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Populate the nested form field with the printed Form 1098. The following instructions apply only within form: Read one Form 1098 Mortgage Interest Statement using its payer/borrower Copy B fields. Read the printed calendar year, not the form revision year, and compare it with the supplied document_facts year; a missing, illegible or conflicting calendar year or a year mismatch is an error. Copy the CORRECTED checkbox exactly; unchecked or blank means false. Copy the recipient/lender and payer/borrower names, TINs and addresses exactly as printed into lender and borrower, preserving masking, punctuation and leading zeros. Copy the lender's telephone number and account number exactly as printed; absent text is empty text. Never infer a borrower from household names or assign ownership. Read boxes 1 through 11 in their printed order: box 1 mortgage interest received, box 2 outstanding mortgage principal, box 3 mortgage origination date, box 4 refund of overpaid interest, box 5 mortgage insurance premiums, box 6 points paid on purchase of principal residence, box 7 same-address checkbox, box 8 address or description of property securing the mortgage, box 9 number of properties securing the mortgage, box 10 Other, and box 11 mortgage acquisition date. Copy monetary boxes 1, 2, 4, 5 and 6 exactly; a clearly blank monetary box means zero, but an illegible or conflicting amount is an error. Box 3 is a required valid calendar date: a blank, illegible, invalid or conflicting date is an error; never replace it with a refinance, draw or acquisition date. Box 7 is true only when checked; blank or unchecked means false. Copy box 8 as printed text, including parcel numbers or other descriptions; leave it empty when blank, and never fill it from the borrower's address. Read box 9 as the printed integer count; blank means zero for an unprinted count, never infer one property. Copy all of box 10 as printed text, preserving labels and amounts; do not classify it as property tax or another fixed amount. Read box 11 as the valid printed calendar date or null when clearly blank; an illegible, invalid or conflicting date is an error. Do not adjust any printed value for the taxpayer's share, prepayment, eligibility, deductible amount, loan use, refund treatment or points treatment. Do not infer personal facts, loan links or calculated values. Repeated copies of the same statement are one form; use Copy B when present and report conflicting copies rather than combining or guessing values. Copy-A-only VOID is not the Copy B CORRECTED checkbox. For this entry's top-level lender kind, use Institution for a lender-issued Form 1098; retain PrivateSeller and NoForm only when the supplied records establish those routes. Do not infer personal use, loan proceeds use or refinancing history.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.form.lender.federal_id,@.form.borrower.federal_id,@.form.calendar_year,@.form.box_3,@.form.box_8,@.form.box_1,@.form.box_2,@.form.box_4,@.form.box_5,@.form.box_6,@.form.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1098Document {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098 as printed
    #[libertas_llm_doc_field]
    pub form: crate::Form1098,
    /// Lender
    /// A private seller or a loan with no Form 1098 has no printed lender statement; enter the paper boxes you have.
    #[libertas_llm_doc_field]
    #[libertas_default(Institution)]
    pub lender: crate::MortgageLender,
}

/// Student loan interest
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1098-E student-loan-interest statement. Do not infer who claims the borrower or who paid an amount absent from the statement. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct StudentLoanDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<crate::StudentLoanDocument>,
}

/// Form 1098-VLI document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1098VliDocument {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098-VLI as printed
    #[libertas_llm_doc_field]
    pub form: crate::Form1098VliPaper,
}

/// Vehicle loan interest
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1098-VLI or lender statement of vehicle-loan interest. Do not infer vehicle eligibility, personal use or loan history. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct VehicleLoanDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<Form1098VliDocument>,
}

/// Marketplace
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1095-A Marketplace health-insurance statement; its monthly rows belong to one policy statement. Forms 1095-B and 1095-C are not this kind. Do not infer shared-policy allocations. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct MarketplaceDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<Form1095ADocument>,
}

/// One Form 1095-A document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1095ADocument {
    /// Accepted source identity
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1095-A as printed
    #[libertas_llm_doc_field]
    pub value: crate::Form1095APaper,
}
impl core::ops::Deref for Form1095ADocument {
    type Target = crate::Form1095APaper;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl From<crate::MarketplacePolicyRecord> for Form1095ADocument {
    fn from(value: crate::MarketplacePolicyRecord) -> Self {
        Self { id: value.id, value: value.form }
    }
}

/// More credits
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1097-BTC tax-credit-bond statement or Form 8986 partnership-audit adjustment statement. Keep supporting pages with their statement; do not infer credit elections or payment dates. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct CreditDocuments {
    /// Documents
    #[libertas_unordered]
    #[libertas_ask_on_add]
    /// ----
    /// Document
    pub documents: Vec<CreditDocument>,
}

/// Documents you may have
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Sort distinct printed documents into the offered common document categories, keeping supporting pages with their document. Leave out documents of other kinds and facts that require personal records. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct DocumentsYouMayHave {
    /// Health savings account contribution statements
    pub health_accounts: crate::HealthContributionDocuments,
    /// Wages
    pub wages: WageDocuments,
    /// Interest and dividends
    pub interest_dividends: InterestDividendDocuments,
    /// Investment sales
    pub investment_sales: InvestmentSaleDocuments,
    /// Retirement and Social Security
    pub retirement: RetirementDocuments,
    /// Self-employment
    pub self_employment: SelfEmploymentDocuments,
    /// Unemployment and state refunds
    pub government_payments: GovernmentPaymentDocuments,
    /// Gambling
    pub gambling: GamblingDocuments,
    /// Partnerships, S corporations, estates and trusts
    pub k1: K1Documents,
    /// Mortgage interest
    pub mortgages: MortgageDocuments,
    /// Student loan interest
    pub student_loans: StudentLoanDocuments,
    /// Education
    pub education: crate::EducationCredits,
    /// Marketplace
    pub marketplace: MarketplaceDocuments,
}

/// Other document types
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Sort distinct printed documents into the offered farming, other income, canceled debt, vehicle-loan interest, vehicle contributions and more-credit categories, keeping supporting pages with their document. Leave out other kinds and personal bookkeeping records. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct OtherDocumentTypes {
    /// Farming
    pub farming: FarmingDocuments,
    /// Other income
    pub other_income: OtherIncomeDocuments,
    /// Canceled debt
    pub canceled_debt: CanceledDebtDocuments,
    /// Vehicle loan interest
    pub vehicle_loans: VehicleLoanDocuments,
    /// Vehicle, boat and airplane contributions
    pub vehicle_gifts: crate::VehicleGiftDocuments,
    /// More credits
    pub credits: CreditDocuments,
}

/// Document center
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Sort each distinct printed tax document into its offered kind, keeping all its pages and supporting statements together. Leave out unrelated pages, unsupported forms and attached documents with no offered kind. A printed form does not establish facts absent from it, such as a property's use, share basis or personal elections. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct DocumentCenter {
    /// Documents you may have
    pub common: DocumentsYouMayHave,
    /// Other document types
    pub other: OtherDocumentTypes,
}

/// Your answers about an accepted document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("K1InvestmentOptions")]
pub struct DocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<DocumentQuestion>.allowed_answers")]
    pub answer: DocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_values: Vec<i32>,
    /// Available K-1 choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    pub k1: K1QuestionChoices,
    /// Existing self-employed health plans for this statement
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub health_plans: Vec<crate::RecordChoice>,
    /// Accepted Form 3922 acquisition documents for this employee
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub espp_acquisitions: Vec<crate::RecordChoice>,
    /// Accepted education exclusion forms for this document's owner
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub savings_bond_exclusions: Vec<crate::RecordChoice>,
}

/// Your answers about an accepted education-account document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationDocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<EducationDocumentQuestion>.allowed_answers")]
    pub answer: EducationDocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_values: Vec<i32>,
}

/// Answer called for by the accepted education-account document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum EducationDocumentAnswer {
    /// The student for this education-account distribution
    EducationBeneficiary {
        /// [NullText]
        /// The student has not been identified yet
        /// [SetText]
        /// The student is
        #[libertas_constrained_by("^<EducationDocumentQuestion>.allowed_values")]
        value: Option<crate::Q1099Beneficiary>,
    },
    /// Your Coverdell basis and late-rollover records
    CoverdellRecords {
        /// Opening basis
        /// [NullText]
        /// Opening basis is unavailable
        /// [SetText]
        /// Opening basis is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: Option<i64>,
        /// Contributions for this year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        contributions: i64,
        /// Late-year withdrawals rolled in next year
        /// [NullText]
        /// The late-rollover adjustment is unavailable
        /// [SetText]
        /// The established adjustment is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        late_rollovers: Option<i64>,
    },
    /// Education-account money put back within 60 days
    EducationRedeposited {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Nonqualifying part of the education-account transfer
    EducationTaxableTransfer {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Your 529-to-Roth transfer records
    EducationRothRecords {
        /// [NullText]
        /// The transfer records are unavailable
        /// [SetText]
        /// The transfer records are
        value: Option<crate::QtpRothRollover>,
    },
    /// Community or separate income
    Community {
        #[libertas_constrained_by("^<EducationDocumentQuestion>.allowed_values")]
        value: crate::CommunityShare,
    },
}

impl EducationDocumentAnswer {
    pub(crate) fn from_canonical(answer: DocumentAnswer) -> Option<Self> {
        Some(match answer {
            DocumentAnswer::EducationBeneficiary { value } => Self::EducationBeneficiary { value },
            DocumentAnswer::CoverdellRecords { opening, contributions, late_rollovers } => {
                Self::CoverdellRecords { opening, contributions, late_rollovers }
            }
            DocumentAnswer::EducationRedeposited { value } => Self::EducationRedeposited { value },
            DocumentAnswer::EducationTaxableTransfer { value } => {
                Self::EducationTaxableTransfer { value }
            }
            DocumentAnswer::EducationRothRecords { value } => Self::EducationRothRecords { value },
            DocumentAnswer::Community { value } => Self::Community { value },
            _ => return None,
        })
    }

    pub(crate) fn into_canonical(self) -> DocumentAnswer {
        match self {
            Self::EducationBeneficiary { value } => DocumentAnswer::EducationBeneficiary { value },
            Self::CoverdellRecords { opening, contributions, late_rollovers } => {
                DocumentAnswer::CoverdellRecords { opening, contributions, late_rollovers }
            }
            Self::EducationRedeposited { value } => DocumentAnswer::EducationRedeposited { value },
            Self::EducationTaxableTransfer { value } => {
                DocumentAnswer::EducationTaxableTransfer { value }
            }
            Self::EducationRothRecords { value } => DocumentAnswer::EducationRothRecords { value },
            Self::Community { value } => DocumentAnswer::Community { value },
        }
    }

    fn from_canonical_kind(kind: i32) -> Option<i32> {
        use libertas_macros::variant_index;
        let kinds = [
            (
                variant_index!(DocumentAnswer::EducationBeneficiary) as i32,
                variant_index!(Self::EducationBeneficiary) as i32,
            ),
            (
                variant_index!(DocumentAnswer::CoverdellRecords) as i32,
                variant_index!(Self::CoverdellRecords) as i32,
            ),
            (
                variant_index!(DocumentAnswer::EducationRedeposited) as i32,
                variant_index!(Self::EducationRedeposited) as i32,
            ),
            (
                variant_index!(DocumentAnswer::EducationTaxableTransfer) as i32,
                variant_index!(Self::EducationTaxableTransfer) as i32,
            ),
            (
                variant_index!(DocumentAnswer::EducationRothRecords) as i32,
                variant_index!(Self::EducationRothRecords) as i32,
            ),
            (
                variant_index!(DocumentAnswer::Community) as i32,
                variant_index!(Self::Community) as i32,
            ),
        ];
        kinds.into_iter().find_map(|(canonical, local)| (canonical == kind).then_some(local))
    }
}

impl EducationDocumentQuestion {
    pub(crate) fn from_canonical(row: DocumentQuestion) -> Option<Self> {
        Some(Self {
            key: row.key,
            prompt: row.prompt,
            answer: EducationDocumentAnswer::from_canonical(row.answer)?,
            allowed_answers: row
                .allowed_answers
                .into_iter()
                .map(EducationDocumentAnswer::from_canonical_kind)
                .collect::<Option<_>>()?,
            allowed_values: row.allowed_values,
        })
    }

    // Only the typed answer is accepted from the editor. The canonical walk
    // supplies the key, prompt and allowed choices again.
    pub(crate) fn into_canonical(self, mut expected: DocumentQuestion) -> Option<DocumentQuestion> {
        if self.key != expected.key {
            return None;
        }
        expected.answer = self.answer.into_canonical();
        Some(expected)
    }
}

/// Your answers about an accepted medical-account document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaDocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<HsaDocumentQuestion>.allowed_answers")]
    pub answer: HsaDocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_values: Vec<i32>,
}

/// Answer called for by the accepted medical-account document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum HsaDocumentAnswer {
    /// Your medical-account payment records
    MedicalPayment {
        /// [NullText]
        /// The payment records are not answered yet
        /// [SetText]
        /// The payment records are
        #[libertas_constrained_by("^<HsaDocumentQuestion>.allowed_values")]
        value: Option<crate::MedicalSaPayment>,
    },
    /// Whether your spouse's Medicare MSA became your own
    MedicareSpousal {
        /// [NullText]
        /// The account's spousal history is not answered yet
        /// [SetText]
        /// The Medicare MSA was inherited from my spouse
        value: Option<bool>,
    },
    /// Your Medicare MSA prior-year account records
    MedicarePrior {
        /// [NullText]
        /// The prior-year records are not answered yet
        /// [SetText]
        /// The prior-year account records are
        value: Option<crate::MedicareMsaPrior>,
    },
    /// Community or separate income
    Community {
        #[libertas_constrained_by("^<HsaDocumentQuestion>.allowed_values")]
        value: crate::CommunityShare,
    },
}

impl HsaDocumentAnswer {
    pub(crate) fn from_canonical(answer: DocumentAnswer) -> Option<Self> {
        Some(match answer {
            DocumentAnswer::MedicalPayment { value } => Self::MedicalPayment { value },
            DocumentAnswer::MedicareSpousal { value } => Self::MedicareSpousal { value },
            DocumentAnswer::MedicarePrior { value } => Self::MedicarePrior { value },
            DocumentAnswer::Community { value } => Self::Community { value },
            _ => return None,
        })
    }

    pub(crate) fn into_canonical(self) -> DocumentAnswer {
        match self {
            Self::MedicalPayment { value } => DocumentAnswer::MedicalPayment { value },
            Self::MedicareSpousal { value } => DocumentAnswer::MedicareSpousal { value },
            Self::MedicarePrior { value } => DocumentAnswer::MedicarePrior { value },
            Self::Community { value } => DocumentAnswer::Community { value },
        }
    }

    fn from_canonical_kind(kind: i32) -> Option<i32> {
        use libertas_macros::variant_index;
        let kinds = [
            (
                variant_index!(DocumentAnswer::MedicalPayment) as i32,
                variant_index!(Self::MedicalPayment) as i32,
            ),
            (
                variant_index!(DocumentAnswer::MedicareSpousal) as i32,
                variant_index!(Self::MedicareSpousal) as i32,
            ),
            (
                variant_index!(DocumentAnswer::MedicarePrior) as i32,
                variant_index!(Self::MedicarePrior) as i32,
            ),
            (
                variant_index!(DocumentAnswer::Community) as i32,
                variant_index!(Self::Community) as i32,
            ),
        ];
        kinds.into_iter().find_map(|(canonical, local)| (canonical == kind).then_some(local))
    }
}

impl HsaDocumentQuestion {
    pub(crate) fn from_canonical(row: DocumentQuestion) -> Option<Self> {
        Some(Self {
            key: row.key,
            prompt: row.prompt,
            answer: HsaDocumentAnswer::from_canonical(row.answer)?,
            allowed_answers: row
                .allowed_answers
                .into_iter()
                .map(HsaDocumentAnswer::from_canonical_kind)
                .collect::<Option<_>>()?,
            allowed_values: row.allowed_values,
        })
    }

    // Only the typed answer is accepted from the editor. The canonical walk
    // supplies the key, prompt and allowed choices again.
    pub(crate) fn into_canonical(self, mut expected: DocumentQuestion) -> Option<DocumentQuestion> {
        if self.key != expected.key {
            return None;
        }
        expected.answer = self.answer.into_canonical();
        Some(expected)
    }
}

/// Your answers about an accepted withholding document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredSpouseDocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<InjuredSpouseDocumentQuestion>.allowed_answers")]
    pub answer: InjuredSpouseDocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_values: Vec<i32>,
}

/// Answer called for by the accepted withholding document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum InjuredSpouseDocumentAnswer {
    /// Management of withholding funds
    Management {
        /// [NullText]
        /// No management of the withholding funds is recorded
        #[libertas_default($null)]
        #[libertas_constrained_by("^<InjuredSpouseDocumentQuestion>.allowed_values")]
        value: Option<crate::RefundManagement>,
    },
}

impl InjuredSpouseDocumentAnswer {
    pub(crate) fn from_canonical(answer: DocumentAnswer) -> Option<Self> {
        Some(match answer {
            DocumentAnswer::Management { value } => Self::Management { value },
            _ => return None,
        })
    }

    pub(crate) fn into_canonical(self) -> DocumentAnswer {
        match self {
            Self::Management { value } => DocumentAnswer::Management { value },
        }
    }

    fn from_canonical_kind(kind: i32) -> Option<i32> {
        use libertas_macros::variant_index;
        let kinds = [(
            variant_index!(DocumentAnswer::Management) as i32,
            variant_index!(Self::Management) as i32,
        )];
        kinds.into_iter().find_map(|(canonical, local)| (canonical == kind).then_some(local))
    }
}

impl InjuredSpouseDocumentQuestion {
    pub(crate) fn from_canonical(row: DocumentQuestion) -> Option<Self> {
        Some(Self {
            key: row.key,
            prompt: row.prompt,
            answer: InjuredSpouseDocumentAnswer::from_canonical(row.answer)?,
            allowed_answers: row
                .allowed_answers
                .into_iter()
                .map(InjuredSpouseDocumentAnswer::from_canonical_kind)
                .collect::<Option<_>>()?,
            allowed_values: row.allowed_values,
        })
    }

    // Only the typed answer is accepted from the editor. The canonical walk
    // supplies the key, prompt and allowed choices again.
    pub(crate) fn into_canonical(self, mut expected: DocumentQuestion) -> Option<DocumentQuestion> {
        if self.key != expected.key {
            return None;
        }
        expected.answer = self.answer.into_canonical();
        Some(expected)
    }
}

/// Your answers about an accepted wage document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WagesDocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<WagesDocumentQuestion>.allowed_answers")]
    pub answer: WagesDocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_values: Vec<i32>,
}

/// Answer called for by the accepted wage document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum WagesDocumentAnswer {
    /// Community or separate income
    Community {
        #[libertas_constrained_by("^<WagesDocumentQuestion>.allowed_values")]
        value: crate::CommunityShare,
    },
}

impl WagesDocumentAnswer {
    pub(crate) fn from_canonical(answer: DocumentAnswer) -> Option<Self> {
        Some(match answer {
            DocumentAnswer::Community { value } => Self::Community { value },
            _ => return None,
        })
    }

    pub(crate) fn into_canonical(self) -> DocumentAnswer {
        match self {
            Self::Community { value } => DocumentAnswer::Community { value },
        }
    }

    fn from_canonical_kind(kind: i32) -> Option<i32> {
        use libertas_macros::variant_index;
        let kinds = [(
            variant_index!(DocumentAnswer::Community) as i32,
            variant_index!(Self::Community) as i32,
        )];
        kinds.into_iter().find_map(|(canonical, local)| (canonical == kind).then_some(local))
    }
}

impl WagesDocumentQuestion {
    pub(crate) fn from_canonical(row: DocumentQuestion) -> Option<Self> {
        Some(Self {
            key: row.key,
            prompt: row.prompt,
            answer: WagesDocumentAnswer::from_canonical(row.answer)?,
            allowed_answers: row
                .allowed_answers
                .into_iter()
                .map(WagesDocumentAnswer::from_canonical_kind)
                .collect::<Option<_>>()?,
            allowed_values: row.allowed_values,
        })
    }

    // Only the typed answer is accepted from the editor. The canonical walk
    // supplies the key, prompt and allowed choices again.
    pub(crate) fn into_canonical(self, mut expected: DocumentQuestion) -> Option<DocumentQuestion> {
        if self.key != expected.key {
            return None;
        }
        expected.answer = self.answer.into_canonical();
        Some(expected)
    }
}

/// Choices derived from the accepted K-1 and the proposed topic answers
#[derive(
    Clone, Default, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_read_only]
pub struct K1QuestionChoices {
    pub estate_activities: Vec<crate::EstateSourceChoice>,
    pub estate_mortgages: Vec<crate::EstateSourceChoice>,
    pub source_choices: Vec<i32>,
    pub housing_choices: Vec<i32>,
    pub carbon_choices: Vec<i32>,
    pub renewable_choices: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityComponentRecords {
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub description: Vec<u8>,
    pub personal_services: crate::K1ActivityPersonalSource,
    pub se_history: crate::K1RetainedSeHistory,
    /// [NullText]
    /// Gross self-employment source is unavailable
    /// [SetText]
    /// Gross self-employment source
    pub se_gross: Option<crate::PartnerSeGross>,
    /// [NullText]
    /// Regular activity allocation is unavailable
    /// [SetText]
    /// Regular activity allocation
    pub regular: Option<crate::K1StatementAmounts>,
    pub alternative: crate::K1StatementAmt,
    /// [NullText]
    /// At-risk group is not established
    /// [SetText]
    /// At-risk group
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.risk")]
    pub risk_group: Option<i64>,
    /// [NullText]
    /// Passive group is not established
    /// [SetText]
    /// Passive group
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.passive")]
    pub passive_group: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityQbiRecords {
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub description: Vec<u8>,
    /// [NullText]
    /// Component qualification allocations are unavailable
    /// [SetText]
    /// Component qualification allocations
    pub allocations: Option<Vec<K1ActivityQbiAllocation>>,
    pub qbi_group: alloc::string::String,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityQbiAllocation {
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.components")]
    pub component: i64,
    pub qualified: crate::K1ComponentQualified,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityGraphRecords {
    #[libertas_hidden]
    #[libertas_read_only]
    pub choices: crate::K1ActivityChoiceContext,
    #[libertas_read_only]
    pub components: Vec<K1ActivityComponentRecords>,
    pub risk_groups: Vec<crate::K1RiskGroup>,
    pub passive_groups: Vec<crate::K1PassiveGroup>,
    #[libertas_read_only]
    pub qbi_blocks: Vec<K1ActivityQbiRecords>,
    /// [NullText]
    /// Sale associations are unavailable
    /// [SetText]
    /// Sale associations
    pub section179_sale_sources: Option<Vec<K1ActivitySaleAssociation>>,
    /// [NullText]
    /// Investment-property associations are unavailable
    /// [SetText]
    /// Investment-property associations
    pub investment_property_sources: Option<Vec<K1ActivityPropertyAssociation>>,
    /// [NullText]
    /// Entity-only allocation is unavailable
    /// [SetText]
    /// Entity-only allocation
    pub entity_only: Option<crate::K1EntityOnlyAmounts>,
    pub alternative_entity_only: crate::K1EntityOnlyAmt,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivitySaleAssociation {
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.sales")]
    pub sale: i64,
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.components")]
    pub component: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityPropertyAssociation {
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.properties")]
    pub property: i64,
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ActivityGraphRecords>.choices.components")]
    pub component: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1PartnerActivityPersonal {
    /// [NullText]
    /// Self-employment classification is unavailable
    /// [SetText]
    /// Limited-partner treatment applies
    pub limited_for_se: Option<bool>,
    pub se_history: crate::K1RetainedSeHistory,
    /// [NullText]
    /// Self-employment gross records are unavailable
    /// [SetText]
    /// Self-employment gross records
    pub se_gross: Option<Vec<crate::PartnerSeGross>>,
    /// [NullText]
    /// Prior fuel-credit records are unavailable
    /// [SetText]
    /// Prior fuel-credit records
    pub prior_fuel_credit: Option<crate::K1PriorFuelSource>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1PartnerActivityRecords {
    pub regular: crate::PartnerEntityBasis,
    /// [NullText]
    /// AMT basis records are unavailable
    /// [SetText]
    /// AMT basis records
    pub alternative: Option<crate::PartnerEntityBasis>,
    /// [NullText]
    /// AMT section 179 income-limit history is unavailable
    /// [SetText]
    /// AMT section 179 income-limit history
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// [NullText]
    /// Interest holding period is unavailable
    /// [SetText]
    /// Interest holding period
    pub interest_term: Option<crate::GainTerm>,
    /// [NullText]
    /// AMT current records are unavailable
    /// [SetText]
    /// AMT current records
    pub alternative_current: Option<crate::K1CurrentStatementTotals>,
    /// [NullText]
    /// Activity statement is unavailable
    /// [SetText]
    /// Activity records
    pub activities: Option<K1ActivityGraphRecords>,
    /// [NullText]
    /// Personal self-employment records are unavailable
    /// [SetText]
    /// Personal self-employment records
    pub person: Option<K1PartnerActivityPersonal>,
    pub special: Vec<crate::PartnerSpecial>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpActivityRecords {
    pub regular: crate::ScorpEntityBasis,
    /// [NullText]
    /// AMT basis records are unavailable
    /// [SetText]
    /// AMT basis records
    pub alternative: Option<crate::ScorpEntityBasis>,
    pub loans: Vec<crate::ScorpEntityLoan>,
    /// [NullText]
    /// AMT section 179 income-limit history is unavailable
    /// [SetText]
    /// AMT section 179 income-limit history
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// [NullText]
    /// Stock holding period is unavailable
    /// [SetText]
    /// Stock holding period
    pub stock_term: Option<crate::GainTerm>,
    /// [NullText]
    /// AMT current records are unavailable
    /// [SetText]
    /// AMT current records
    pub alternative_current: Option<crate::K1CurrentStatementTotals>,
    /// [NullText]
    /// Activity statement is unavailable
    /// [SetText]
    /// Activity records
    pub activities: Option<K1ActivityGraphRecords>,
    pub special: Vec<crate::ScorpSpecial>,
}

/// Answer called for by the accepted document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[expect(
    clippy::large_enum_variant,
    reason = "Typed protocol answers carry their complete editable records"
)]
pub enum DocumentAnswer {
    /// Bond redemption and education records supporting this Form 8815
    SavingsBondsRecords {
        /// [NullText]
        /// Supporting bond and education records not answered yet
        /// [SetText]
        /// Enter the bond redemption, issue date, source and education records
        value: Option<crate::SavingsBondAnswersView>,
    },
    /// Acquisition records for these actual ESPP shares sold
    EsppAcquisition {
        /// Acquisition source
        /// [NullText]
        /// Choose the actual acquisition source
        value: Option<crate::EsppAcquisitionPage>,
    },
    /// Actual fixed grant-date exercise price when Form 3922 box 8 is blank
    EsppFixedGrantPrice { value: crate::SourceCents },
    /// Fees, wash-sale adjustment and special treatment for this ESPP sale
    EsppAdjustments { value: crate::EsppShareSaleMore },
    /// Whether wages already include the compensation from this ESPP sale
    EsppWageInclusion {
        /// Your W-2 box 1 already includes this sale's full calculated compensation
        /// Check the employer's statement. If No, the compensation enters Schedule 1 once.
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Fees and special treatment for this ISO sale
    IsoAdjustments {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The fees and special treatment are
        value: Option<crate::IsoShareSaleMore>,
    },
    /// Whether wages already include the compensation from this ISO sale
    IsoWageInclusion {
        /// Your W-2 box 1 already includes this sale's compensation
        /// Applies because the sale is within 2 years of the grant or 1 year of the exercise. Check the employer's statement. If No, the compensation enters Schedule 1 once.
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Wash sale and special treatment for this employer-stock sale
    NuaAdjustments {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The wash sale and special treatment are
        value: Option<crate::NuaShareSaleMore>,
    },
    /// Whose employer shares these were
    NuaOwnership {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::NuaOwnership>,
    },
    /// Where the Medicare premiums on this statement are already included
    SocialPremiums {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::SocialPremiumUsePage>,
    },
    /// Medicare premiums on this statement reimbursed tax-free from an HSA
    SocialReimbursedPremiums {
        /// Actual part included in the qualified HSA spending already entered
        /// Enter zero if none. Do not repeat a premium used by a business health plan.
        /// [NullText]
        /// Reimbursed portion not answered yet
        /// [SetText]
        /// Enter the actual reimbursed portion, including zero
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reimbursed: Option<i64>,
    },
    /// Allocation between medical expenses, a business health plan and HSA reimbursement
    SocialSharedPremiums {
        /// Where these premiums are already included
        /// [NullText]
        /// Premium allocation not answered yet
        value: Option<crate::SocialPremiumUsePage>,
        /// Actual part reimbursed tax-free and included in existing qualified HSA spending
        /// Enter zero if none; this part must not also be included in the selected plan.
        /// [NullText]
        /// Reimbursed portion not answered yet
        /// [SetText]
        /// Enter the actual reimbursed portion, including zero
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reimbursed: Option<i64>,
    },
    /// Treatment of this canceled debt
    DebtTreatment {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The canceled debt is treated as
        value: Option<crate::CanceledDebtChoice>,
    },
    /// Property the creditor actually took
    DebtProperty {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The property was
        value: Option<crate::TransferredProperty>,
    },
    /// Classification of this acquisition or abandonment
    DebtAcquisition {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The property was
        value: Option<crate::AcquiredPropertyView>,
    },
    /// Interest paid in addition to the benefits
    LtcInterest {
        /// From the payer's actual statement, not part of box 1 or 2. Do not also enter it as a separate 1099-INT amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Actual source of the box 1 benefits
    LtcBenefitOrigin {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The box 1 benefits came from
        value: Option<crate::LtcPaymentOrigin>,
    },
    /// Actual source of the box 2 accelerated death benefits
    LtcAcceleratedOrigin {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The box 2 accelerated benefits came from
        value: Option<crate::LtcPaymentOrigin>,
    },
    /// Actual contract qualification and history
    LtcContract { value: crate::LtcContractPage },
    /// Long-term care periods for the insured person
    LtcInsured {
        /// [NullText]
        /// Use the only insured person's periods record
        #[libertas_enum_source("$.ltc_insured")]
        value: Option<i64>,
    },
    /// Whose railroad annuity this is
    RailroadRecipient {
        /// [NullText]
        /// Not answered yet
        #[libertas_default($null)]
        value: Option<crate::RailroadRecipient>,
    },
    /// The years of the railroad repayments
    RailroadRepayments {
        /// Repayments by the year the benefits were paid
        value: Vec<crate::RailroadRepayment>,
    },
    /// Your answers about this W-2
    Wage { answers: crate::W2AnswersView },
    /// What the distribution codes mean for you
    RetirementTreatment {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// Distribution treatment
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        code: Option<crate::RetirementTreatmentView>,
    },
    /// How to figure the taxable distribution
    RetirementMethod {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// Taxable amount method
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        method: Option<crate::TaxableMethod>,
    },
    /// Nontaxable part of the in-plan Roth rollover
    RetirementBasis {
        /// [NullText]
        /// Not answered yet
        #[libertas_default($null)]
        basis: Option<crate::InPlanRothBasis>,
    },
    /// Disability retirement income
    RetirementDisability {
        /// [NullText]
        /// No disability retirement income
        #[libertas_default($null)]
        disability: Option<crate::DisabilityIncome>,
    },
    /// Community or separate income
    Community {
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: crate::CommunityShare,
    },
    /// Management of withholding funds
    Management {
        /// [NullText]
        /// No management of the withholding funds is recorded
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::RefundManagement>,
    },
    /// Disaster distribution treatment
    RetirementDisaster {
        /// [NullText]
        /// No qualified disaster distribution
        #[libertas_default($null)]
        value: Option<crate::DisasterDistribution>,
    },
    /// Public safety pension premiums
    RetirementPublicSafety {
        /// [NullText]
        /// No public safety pension exclusion claimed
        #[libertas_default($null)]
        value: Option<crate::PublicSafetyAnswer>,
    },
    /// Payments made after death
    RetirementAfterDeath {
        /// [NullText]
        /// No payments made after death
        /// [SetText]
        /// Payments made after death
        #[libertas_default($null)]
        value: Option<crate::RetirementAfterDeath>,
    },
    /// Interest reported for someone else
    InterestNominee {
        /// [NullText]
        /// All interest belongs to this return
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::IntNomineeAnswer>,
    },
    /// Interest received after death
    InterestAfterDeath {
        /// [NullText]
        /// No interest received after death
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::IntAfterDeathAnswer>,
    },
    /// Personal interest reporting records
    InterestAdjustment {
        /// [NullText]
        /// No personal reporting adjustments
        /// [SetText]
        /// Personal reporting adjustments
        #[libertas_default($null)]
        value: Option<crate::InterestAdjustments>,
    },
    /// Accrued interest paid on purchase
    InterestAccrued {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Other interest circumstances
    InterestTreatment { value: crate::InterestSpecial },
    /// Original issue discount reported for someone else
    OidNominee {
        /// [NullText]
        /// All original issue discount belongs to this return
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::OidNomineeAnswer>,
    },
    /// Tax-exempt bond premium
    OidPremium {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Private activity bond interest
    OidPrivateActivity {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Other original issue discount circumstances
    OidTreatment {
        /// A box 7 description naming a stripped bond, stripped coupon or contingent-payment debt instrument is an identified obligation.
        /// [NullText]
        /// Not chosen yet; the estimate counts this statement
        #[libertas_default($null)]
        value: Option<crate::OidSpecial>,
    },
    /// Dividends reported for someone else
    DividendNominee {
        /// [NullText]
        /// All dividend components belong to this return
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::DivNomineeAnswer>,
    },
    /// Eligibility of section 199A dividends
    DividendEligibility {
        /// [NullText]
        /// Not answered yet
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::DividendEligibility>,
    },
    /// Dividends received after death
    DividendAfterDeath {
        /// [NullText]
        /// No dividends received after death
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::DivAfterDeathAnswer>,
    },
    /// Fund section 1202 exclusion
    DividendFundExclusion {
        /// [NullText]
        /// Not answered yet
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::FundQsbsExclusion>,
    },
    /// Share lots receiving nondividend or liquidation distributions
    DividendCapitalReturns { value: Vec<crate::CapitalReturn1099Div> },
    /// Other dividend circumstances
    DividendTreatment { value: crate::DividendSpecial },
    /// Adjustments to the reported sale
    SaleAdjustment { value: crate::SaleAdjustment },
    /// Personal sale circumstances
    SaleTreatment { value: crate::SaleSpecial },
    /// Other undistributed capital gain circumstances
    UndistributedGainSpecial {
        #[libertas_default(false)]
        value: bool,
    },
    /// Other futures and section 1256 option circumstances
    ContractsSpecial {
        #[libertas_default(false)]
        value: bool,
    },
    /// What happened to this opportunity fund investment in 2026
    OpportunityFundEvent {
        /// [NullText]
        /// I do not know what happened yet
        value: Option<crate::QofEvent>,
    },
    /// Other opportunity fund investment circumstances
    OpportunityFundSpecial {
        /// A partial sale, a distribution from the fund, a gift of the investment, a fund interest held through a partnership or S corporation, or another special treatment
        /// Yes leaves this investment out of the estimate with a notice; that treatment is not supported.
        #[libertas_default(false)]
        value: bool,
    },
    /// Shares from this incentive stock option exercise
    IsoExerciseFacts {
        /// [NullText]
        /// Share details not answered
        /// [SetText]
        /// Enter share details
        value: Option<crate::Form3921More>,
    },
    /// Use of the taxable grant
    GovernmentGrantUse {
        /// [NullText]
        /// Relate this grant from its activity
        #[libertas_default($null)]
        value: Option<crate::GrantUse>,
    },
    /// Paid medical leave included in family leave
    GovernmentMedicalLeave {
        #[libertas_default(false)]
        value: bool,
    },
    /// Benefits repaid during the year
    GovernmentRepayments { value: Vec<crate::UnemploymentRepayment> },
    /// Other benefit circumstances
    GovernmentSpecial {
        #[libertas_default(false)]
        value: bool,
    },
    /// Other gambling circumstances
    GamblingSpecial {
        #[libertas_default(false)]
        value: bool,
    },
    /// Deduction of the refunded state or local tax
    RefundDeduction {
        /// [NullText]
        /// Not answered
        value: Option<crate::RefundDeduction>,
    },
    /// What you sold
    HomeSale {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// I sold
        #[libertas_default($null)]
        value: Option<crate::RealEstateSaleUse>,
    },
    /// Recipient allocations and cooperative records
    CooperativeAllocations { value: Vec<crate::CoopAllocationInput> },
    /// Personal cooperative credit records
    CooperativeCreditRecords {
        /// Issuer breakdown of box 10
        investment_shares: Vec<crate::CoopInvestmentShare>,
        /// Separately identified box 12 credits and deductions
        other: Vec<crate::CoopOtherItemInput>,
        /// [NullText]
        /// I have no participation records; the credits are treated as passive
        /// [SetText]
        /// My activity credit participation is
        #[libertas_default($null)]
        participation: Option<bool>,
        /// [NullText]
        /// I am not claiming the box 10 or 11 credits
        /// [SetText]
        /// The credits belong to
        #[libertas_default($null)]
        #[libertas_enum_source("$.cooperative_activities")]
        activity: Option<i64>,
        /// [NullText]
        /// I have no credit allocation notice
        /// [SetText]
        /// The credit notice status is
        #[libertas_default($null)]
        notice: Option<crate::CoopNoticeStatus>,
    },
    /// Personal treatment of this printed box 12 item
    CooperativeOtherRecords {
        /// Printed credit or deduction kind
        #[libertas_read_only]
        kind: crate::CoopOtherKind,
        /// [NullText]
        /// The originating activity has not been linked
        /// [SetText]
        /// The originating activity is
        #[libertas_default($null)]
        #[libertas_enum_source("$.cooperative_activities")]
        activity: Option<i64>,
        #[libertas_default(Unresolved)]
        income: crate::CoopCreditIncome,
    },
    /// Where the nonemployee compensation belongs
    NonemployeeUse {
        /// [NullText]
        /// Relate this amount from its activity
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::IncomeUse>,
    },
    /// Where the other income belongs
    // Canonical row only: asked as a required strict question.
    MiscellaneousUse {
        /// [NullText]
        /// Not answered yet
        #[libertas_default($null)]
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::IncomeUse>,
    },
    /// Attorney proceeds paid on to clients or others
    AttorneyClientFunds {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Where the payment-card and network payments belong
    PaymentParts(PaymentPartsAnswer),
    /// The student for this education-account distribution
    EducationBeneficiary {
        /// [NullText]
        /// The student has not been identified yet
        /// [SetText]
        /// The student is
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::Q1099Beneficiary>,
    },
    /// Your Coverdell basis and late-rollover records
    CoverdellRecords {
        /// Opening basis
        /// [NullText]
        /// Opening basis is unavailable
        /// [SetText]
        /// Opening basis is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: Option<i64>,
        /// Contributions for this year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        contributions: i64,
        /// Late-year withdrawals rolled in next year
        /// [NullText]
        /// The late-rollover adjustment is unavailable
        /// [SetText]
        /// The established adjustment is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        late_rollovers: Option<i64>,
    },
    /// Education-account money put back within 60 days
    EducationRedeposited {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Nonqualifying part of the education-account transfer
    EducationTaxableTransfer {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        value: i64,
    },
    /// Your 529-to-Roth transfer records
    EducationRothRecords {
        /// [NullText]
        /// The transfer records are unavailable
        /// [SetText]
        /// The transfer records are
        value: Option<crate::QtpRothRollover>,
    },
    /// Your medical-account payment records
    MedicalPayment {
        /// [NullText]
        /// The payment records are not answered yet
        /// [SetText]
        /// The payment records are
        #[libertas_constrained_by("^<DocumentQuestion>.allowed_values")]
        value: Option<crate::MedicalSaPayment>,
    },
    /// Whether your spouse's Medicare MSA became your own
    MedicareSpousal {
        /// [NullText]
        /// The account's spousal history is not answered yet
        /// [SetText]
        /// The Medicare MSA was inherited from my spouse
        value: Option<bool>,
    },
    /// Your Medicare MSA prior-year account records
    MedicarePrior {
        /// [NullText]
        /// The prior-year records are not answered yet
        /// [SetText]
        /// The prior-year account records are
        value: Option<crate::MedicareMsaPrior>,
    },
    /// What the ABLE payment was
    AbleTreatment {
        /// [NullText]
        /// The payment treatment is not answered yet
        /// [SetText]
        /// The payment treatment is
        value: Option<crate::AbleTreatment>,
    },
    /// Your records for this fiduciary activity
    EstateActivityRecords {
        /// Accepted activity
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        description: Vec<u8>,
        participation: crate::EstateParticipation,
        carried: crate::EstatePassiveCarry,
    },
    /// Prior suspended qualified business losses for this source
    EstateQbiHistory {
        /// Accepted statement and activity
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        description: Vec<u8>,
        /// Activities identified by this statement
        #[libertas_unordered]
        #[libertas_enum_source("^<DocumentQuestion>.k1.estate_activities")]
        activities: Vec<i64>,
        passive_history: Vec<crate::ScorpLossYear>,
        allocations: Vec<EstateQbiAllocationHistory>,
    },
    /// Treatment of this fiduciary AMT statement amount
    EstateAmtTreatment {
        /// Accepted adjustment
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        description: Vec<u8>,
        treatment: EstateAmtTreatmentAnswer,
    },
    /// Your own investment-credit forms for these accepted properties
    EstateInvestmentRecords {
        #[libertas_read_only]
        properties: Vec<K1InvestmentAnswer>,
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_choices: Vec<i32>,
    },
    /// Source character of the box 5 royalties and related box 9 deductions
    EstateRoyaltyCharacter { royalty_only: bool, deduction: crate::EstateRoyaltyDeductionCharacter },
    /// Accounting election accompanying the current entity period
    EstateAccounting {
        /// [NullText]
        /// The accounting election is not established
        value: Option<crate::FinalEstateTrustAccountingYear>,
    },
    /// Fiduciary's breakdown of box 11 code B taxes
    EstateFinalTaxCharacter {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        personal_property_tax: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        real_estate_tax: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        state_income_tax: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        state_sales_tax: i64,
    },
    /// Original loss periods and beneficiary NOL carryover records
    EstateNolRecords {
        /// [NullText]
        /// The fiduciary's NOL records are unavailable
        /// [SetText]
        /// The fiduciary's NOL records are
        value: Option<crate::FinalEstateTrustNol>,
    },
    /// Modified adjusted gross income portion of box 14 code H
    EstateNiitAllocation {
        /// [NullText]
        /// The fiduciary's allocation is unavailable
        /// [SetText]
        /// The modified adjusted gross income portion is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        value: Option<i64>,
    },
    /// Beneficiary's Form 3800 line for box 13 code I
    EstateResearchCreditLine {
        /// Choose line 4i only when you and the estate or trust both meet the eligible-small-business test; otherwise line 1c.
        /// [NullText]
        /// The Form 3800 line is not established
        #[libertas_constrained_by("$.research_credit_lines")]
        value: Option<crate::BusinessCredit>,
    },
    /// Other source treatment applies
    EstateOtherSource { value: bool },
    /// Your entitlement and previously entered mortgage records
    EstateFinalRecords {
        #[libertas_unordered]
        #[libertas_enum_source("^<DocumentQuestion>.k1.estate_mortgages")]
        mortgages: Vec<i64>,
    },
    /// Holding-period eligibility of the printed section 199A dividends
    EstateReitEligibility {
        /// [NullText]
        /// Holding-period eligibility is not established
        /// [SetText]
        /// The reported dividends meet the holding-period rules
        value: Option<bool>,
    },
    /// Your partnership activity basis and retained records
    K1PartnerActivityRecords { value: K1PartnerActivityRecords },
    /// Your S corporation activity basis and retained records
    K1ScorpActivityRecords { value: K1ScorpActivityRecords },
    /// Your partnership records
    K1PartnerRecords {
        qbi_records: crate::PartnerQbiRecords,
        basis: crate::PartnerBasisRecords,
        #[libertas_constrained_by("^<DocumentQuestion>.k1.source_choices")]
        income_source: crate::K1IncomeSource,
        special: Vec<crate::PartnerSpecial>,
        qbi_group: alloc::string::String,
        /// [NullText]
        /// I am not claiming a box 20 code C fuel credit
        /// [SetText]
        /// The box 20 code C fuel credit information is
        fuel_credit: Option<crate::PartnerFuelTaxCredit>,
        /// [NullText]
        /// No 2025 fuel credit needs a current income adjustment
        /// [SetText]
        /// The 2025 fuel credit and prior deduction facts are
        prior_fuel_credit: Option<crate::PartnerPriorFuelCredit>,
        term: crate::K1StockTerm,
        trading: bool,
    },
    /// Your S-corporation records
    K1ScorpRecords {
        qbi_records: crate::K1ScorpQbiRecords,
        basis: crate::K1ScorpBasisRecords,
        special: Vec<crate::ScorpSpecial>,
        qbi_group: alloc::string::String,
        stock_term: crate::K1StockTerm,
        trading: bool,
    },
    /// What happened to your entity interest
    K1EntityDisposition {
        /// [NullText]
        /// What happened to this interest is not answered yet
        /// [SetText]
        /// What happened to this interest is
        value: Option<crate::K1Disposition>,
    },
    /// Your rental activity records
    K1RentalRecords { carryover_active: bool, disposed: Vec<crate::K1RentalActivity> },
    /// Whether you succeed to the terminated estate or trust
    // Canonical row only: asked as a required strict question.
    EstateFinalSuccessor {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// When you became a partner and your opening basis
    // Canonical row only: asked as a required strict question.
    K1PartnerBasisStart {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::BasisStart>,
    },
    /// When you became a shareholder and your opening basis
    // Canonical row only: asked as a required strict question.
    K1ScorpBasisStart {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::K1ScorpStart>,
    },
    /// How you took part in the rental real estate
    // Canonical row only: asked as a required strict question, and only
    // when the K-1 has rental real estate.
    K1RentalParticipation {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::K1RentalParticipation>,
    },
    /// How you took part in this partnership
    // Canonical row only: asked as a required strict question.
    K1PartnerParticipation {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::PartnerParticipation>,
    },
    /// Limited partner for self-employment tax
    // Canonical row only: asked as a required strict question.
    K1LimitedForSe {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Your partnership alternative minimum tax records
    // Canonical row only: asked as a required strict question.
    K1PartnerAmt {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::PartnerAmtRecords>,
    },
    /// What happened to your partnership interest
    // Canonical row only: asked as a required strict question.
    K1PartnerDisposition {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::PartnerDisposition>,
    },
    /// How you took part in this S corporation
    // Canonical row only: asked as a required strict question.
    K1ScorpParticipation {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::Participation>,
    },
    /// Your S-corporation alternative minimum tax records
    // Canonical row only: asked as a required strict question.
    K1ScorpAmt {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::ScorpAmtRecords>,
    },
    /// What happened to your S-corporation shares
    // Canonical row only: asked as a required strict question.
    K1ScorpDisposition {
        /// [NullText]
        /// Not answered yet
        value: Option<crate::ScorpDisposition>,
    },
    /// Your foreign withholding treatment for the K-3 passive countries
    K1K3Withholding { value: Vec<crate::K3Withholding> },
    /// Your section 179 carryover records
    K1Section179Records {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carryover: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        sold: i64,
    },
    /// Your AMT section 179 carryover history
    // Canonical row only: asked as a required strict question.
    K1Section179Amt {
        /// From your own prior AMT Form 4562 records, independently of the regular carryover. Enter actual zero amounts when the AMT carryover is zero.
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// Actual AMT carryover history, including an actual zero
        value: Option<crate::Section179IncomeCarryHistory>,
    },
    /// Depletion adjustment by activity on this K-1
    K1DepletionSources { value: crate::K1DepletionSourcesPage },
    /// Your credit eligibility confirmations
    K1CreditConfirmations {
        // Canonical row only: asked as a required strict question.
        /// [NullText]
        /// Not answered yet
        #[libertas_constrained_by("^<DocumentQuestion>.k1.housing_choices")]
        housing: Option<crate::K1HousingAcquisition>,
    },
    /// Your investment-credit records
    K1InvestmentRecords {
        #[libertas_read_only]
        properties: Vec<K1InvestmentAnswer>,
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_choices: Vec<i32>,
        /// [NullText]
        /// The automatic investment-credit property facts are not confirmed yet
        /// [SetText]
        /// The investment-credit property facts are confirmed as
        #[libertas_constrained_by("^<DocumentAnswer>.K1InvestmentRecords.allowed_choices")]
        investment: Option<crate::K1InvestmentConfirmation>,
    },
    /// Supporting origin details for this credit
    K1CreditOrigins {
        /// [NullText]
        /// I do not have the equipment origin details for the extended carryback
        /// [SetText]
        /// The equipment origin details are
        #[libertas_constrained_by("^<DocumentQuestion>.k1.carbon_choices")]
        carbon_oxide_origin: Option<crate::CarbonOxideCreditOrigin>,
        /// [NullText]
        /// I do not have the facility origin details for the extended carryback
        /// [SetText]
        /// The facility origin details are
        #[libertas_constrained_by("^<DocumentQuestion>.k1.renewable_choices")]
        renewable_facility_origin: Option<crate::RenewableFacilityCreditOrigin>,
    },
    /// Your eligibility for the K-1 research credit
    K1ResearchCreditLine {
        /// Choose line 4i only when you and the entity both meet the eligible-small-business test; otherwise line 1c.
        /// [NullText]
        /// My eligible-small-business treatment is not answered yet
        /// [SetText]
        /// My treatment is
        #[libertas_constrained_by("$.research_credit_lines")]
        #[libertas_default($null)]
        value: Option<crate::BusinessCredit>,
    },
    /// Your capital-sale records
    BrokerSaleRecords { value: crate::BrokerSaleAnswers },
    /// Section 1256 contract treatment
    BrokerContractsTreatment {
        /// A business or farm hedge, a section 475(f) or 988 election, or dealer contracts
        /// Yes retains the statement with the existing uncalculated-treatment notice.
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// This statement has special contract treatment
        value: Option<bool>,
    },
    /// Link this redemption record to its education exclusion form
    BondRedemptionForm8815 {
        /// [NullText]
        /// Not linked to a Form 8815
        /// [SetText]
        /// Link this redemption record to its Form 8815
        #[libertas_default($null)]
        value: Option<crate::BondRedemptionLink>,
    },
    /// Stock-sale qualification answer
    QsbsBoolean { value: crate::QsbsBooleanAnswer },
    /// Prior eligible gain for this stock issuer
    QsbsPriorGain { value: crate::QsbsGainAnswer },
    /// Additional empowerment-zone exclusion
    QsbsZone { value: crate::QsbsZoneAnswer },
    /// Historical gain for this empowerment-zone lot
    QsbsZoneGain { value: crate::QsbsValuationAnswer },
    /// Active conduct of this rental as a trade or business, for the section 179 income limit
    RentalSection179Business {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Last year's home-use worksheet carries for this rental
    // Canonical row only: asked as a required strict question.
    RentalHomeCarries {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The carries are
        value: Option<crate::RentalHomeCarries>,
    },
    /// 2025 Form 1040 line 1z wages
    // Canonical row only: asked as a required strict question (More credits).
    BbaPriorWages {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 taxable income
    // Canonical row only: asked as a required strict question (More credits).
    BbaPriorTaxableIncome {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 income tax
    // Canonical row only: asked as a required strict question (More credits).
    BbaPriorIncomeTax {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 filing status
    // Canonical row only: asked as a required strict question (More credits).
    BbaPriorStatus {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::FederalStatus>,
    },
    /// Only one BBA-audit adjustment this year
    // Canonical row only: asked as a required strict question (More credits).
    BbaSingleAdjustment {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// The 2025 return is the simple Tax Table case
    // Canonical row only: asked as a required strict question (More credits).
    BbaSimplePriorReturn {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// No other effects of the adjustment
    // Canonical row only: asked as a required strict question (More credits).
    BbaNoOtherEffects {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Form 8978 line 17 interest
    // Canonical row only: asked as a required strict question, and only
    // when the refigured 2025 tax rises.
    BbaLine17Interest {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        interest: Option<i64>,
        /// [NullText]
        /// Not answered yet
        prepared: Option<bool>,
    },
    /// Bond class
    // Canonical row only: asked as a required strict question (More credits).
    BondClass {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::FederalTaxCreditBondKind>,
    },
    /// Original bond issue date
    // Canonical row only: asked as a required strict question (More credits).
    BondIssueDate {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_date_only]
        value: Option<u32>,
    },
    /// How the bond credit reached you
    // Canonical row only: asked as a required strict question (More credits).
    BondOrigin {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::TaxCreditOrigin>,
    },
    /// Unused 2025 credit for this bond
    // Canonical row only: asked as a required strict question (More credits).
    BondPriorCarry {
        /// Unused 2025 credit in whole dollars
        dollars: i64,
        /// [NullText]
        /// Not answered yet
        confirmed: Option<bool>,
    },
    /// This bond's share of the allowed credit
    // Canonical row only: asked as a required strict question, and only for
    // a partly limited Form 8912 with several bonds.
    BondAllowedShare {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        dollars: Option<i64>,
        /// [NullText]
        /// Not answered yet
        confirmed: Option<bool>,
    },
}

/// Retained losses for one accepted fiduciary QBI allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateQbiAllocationHistory {
    /// Activity receiving this part of the statement
    #[libertas_key]
    #[libertas_unique]
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<DocumentQuestion>.k1.estate_activities")]
    pub activity: i64,
    /// Signed current qualified income allocated to this activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    pub passive_history: Vec<crate::ScorpLossYear>,
}

/// Topic answer projected onto the existing fiduciary AMT treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateAmtTreatmentAnswer {
    IncludedInA,
    Activity {
        included_in_a: bool,
        #[libertas_number(min = 1)]
        #[libertas_enum_source("^<DocumentQuestion>.k1.estate_activities")]
        activity: i64,
    },
    Independent {
        line: crate::EstateAmtLine,
        included_in_a: bool,
    },
    Unavailable,
}
impl From<crate::EstateAmtTreatment> for EstateAmtTreatmentAnswer {
    fn from(value: crate::EstateAmtTreatment) -> Self {
        match value {
            crate::EstateAmtTreatment::IncludedInA => Self::IncludedInA,
            crate::EstateAmtTreatment::Activity { included_in_a, activity } => {
                Self::Activity { included_in_a, activity }
            }
            crate::EstateAmtTreatment::Independent { line, included_in_a } => {
                Self::Independent { line, included_in_a }
            }
            crate::EstateAmtTreatment::Unavailable => Self::Unavailable,
        }
    }
}
impl From<EstateAmtTreatmentAnswer> for crate::EstateAmtTreatment {
    fn from(value: EstateAmtTreatmentAnswer) -> Self {
        match value {
            EstateAmtTreatmentAnswer::IncludedInA => Self::IncludedInA,
            EstateAmtTreatmentAnswer::Activity { included_in_a, activity } => {
                Self::Activity { included_in_a, activity }
            }
            EstateAmtTreatmentAnswer::Independent { line, included_in_a } => {
                Self::Independent { line, included_in_a }
            }
            EstateAmtTreatmentAnswer::Unavailable => Self::Unavailable,
        }
    }
}
/// Your own Form 3468 for an accepted property
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1InvestmentAnswer {
    /// Accepted property identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Printed property description
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub description: Vec<u8>,
    /// Your own completed Form 3468
    /// [NullText]
    /// Calculate the credit from this property's facts
    /// [SetText]
    /// My own Form 3468 credit is
    #[libertas_calculation_request("K1InvestmentOptions")]
    pub own_form: Option<crate::OwnInvestmentCredit>,
}

/// Investment-credit answers supplied to the catalog calculation
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1InvestmentAnswerInput {
    pub id: i64,
    /// [NullText]
    /// Calculate the credit from this property's facts
    /// [SetText]
    /// My own Form 3468 credit is
    pub own_form: Option<crate::OwnInvestmentCredit>,
}
impl From<K1InvestmentAnswerInput> for K1InvestmentAnswer {
    fn from(value: K1InvestmentAnswerInput) -> Self {
        Self { id: value.id, description: Vec::new(), own_form: value.own_form }
    }
}
impl From<K1InvestmentAnswer> for K1InvestmentAnswerInput {
    fn from(value: K1InvestmentAnswer) -> Self {
        Self { id: value.id, own_form: value.own_form }
    }
}

/// Accepted source document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageDocumentView {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Form 1098 as printed
    #[libertas_read_only]
    pub value: crate::Form1098,
    /// Property label or address
    pub property: alloc::string::String,
    /// Lender
    pub lender: crate::MortgageLender,
    /// Refund answers the printed box 4 calls for
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_refund_origins: Vec<i32>,
    /// What the box 4 refund repays
    /// Answered on the mortgage questions page; change it there.
    /// [SetText]
    /// The box 4 refund repays
    #[libertas_read_only]
    #[libertas_constrained_by("^.allowed_refund_origins")]
    pub refund_origin: Option<crate::MortgageRefundOrigin>,
    /// Loan facts or the earlier lender
    pub details: crate::MortgageDetailsView,
    /// Your share of the interest reported in box 1
    /// Include a spouse's payments on a joint return. Use the payment records when another borrower paid part of the interest.
    pub paid_share: crate::MortgagePaidShare,
    /// Date the mortgage insurance contract was issued
    /// Use the insurance contract, not the mortgage origination or lender acquisition date.
    pub insurance_contract: crate::SourceDate,
    /// Point treatments available for this printed amount
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_points: Vec<i32>,
    /// How the printed points are deducted
    #[libertas_constrained_by("^.allowed_points")]
    pub points: MortgagePointTreatment,
    /// Date the acquisition debt was incurred
    pub debt_date: crate::MortgageDebtDate,
}
pub use crate::mortgage_point_treatment::MortgagePointTreatment;

// The mortgage page as BeginMortgages carries it while a loan's facts are not
// answered yet: answered loans by the page's names, and each new loan with
// only what its paper settles (the lender that sent the 1098, the box 3 date
// and the printed amounts), so the editor built from it starts the rest
// unanswered.
/// Home mortgage interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgagesSeed {
    /// Forms 1098
    pub loans: Vec<MortgageLoanSeed>,
    /// Forms 1098 not answered yet
    pub new_loans: Vec<MortgageNewLoanSeed>,
}
/// Form 1098
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageLoanSeed {
    /// Accepted document identity
    pub id: i64,
    /// Form 1098 as printed
    pub value: crate::Form1098,
    /// Property label or address
    pub property: alloc::string::String,
    /// Lender
    pub lender: crate::MortgageLender,
    /// Refund answers the printed box 4 calls for
    pub allowed_refund_origins: Vec<i32>,
    /// What the box 4 refund repays
    /// Required when box 4 reports a refund: the page does not save without the answer.
    /// [SetText]
    /// The box 4 refund repays
    pub refund_origin: Option<crate::MortgageRefundOrigin>,
    /// Loan facts or the earlier lender
    pub details: crate::MortgageDetailsView,
    /// Your share of the interest reported in box 1
    pub paid_share: crate::MortgagePaidShare,
    /// Date the mortgage insurance contract was issued
    pub insurance_contract: crate::SourceDate,
    /// Point treatments available for this printed amount
    pub allowed_points: Vec<i32>,
    /// How the printed points are deducted
    pub points: MortgagePointTreatment,
    /// Date the acquisition debt was incurred
    pub debt_date: crate::MortgageDebtDate,
}
/// Form 1098
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageNewLoanSeed {
    /// Accepted document identity
    pub id: i64,
    /// Form 1098 as printed
    pub value: crate::Form1098,
    /// Property label or address
    pub property: alloc::string::String,
    /// Lender
    pub lender: crate::MortgageLender,
    /// Refund answers the printed box 4 calls for
    pub allowed_refund_origins: Vec<i32>,
    /// What the box 4 refund repays
    /// Answered on the mortgage questions page before this page opens.
    pub refund_origin: Option<crate::MortgageRefundOrigin>,
    /// Point treatments available for this printed amount
    pub allowed_points: Vec<i32>,
    /// Date the acquisition debt was incurred
    pub debt_date: crate::MortgageDebtDate,
}
impl From<MortgageDocumentView> for MortgageLoanSeed {
    fn from(v: MortgageDocumentView) -> Self {
        Self {
            id: v.id,
            value: v.value,
            property: v.property,
            lender: v.lender,
            allowed_refund_origins: v.allowed_refund_origins,
            refund_origin: v.refund_origin,
            details: v.details,
            paid_share: v.paid_share,
            insurance_contract: v.insurance_contract,
            allowed_points: v.allowed_points,
            points: v.points,
            debt_date: v.debt_date,
        }
    }
}
impl From<MortgageDocumentView> for MortgageNewLoanSeed {
    fn from(v: MortgageDocumentView) -> Self {
        Self {
            id: v.id,
            value: v.value,
            property: v.property,
            lender: v.lender,
            allowed_refund_origins: v.allowed_refund_origins,
            refund_origin: v.refund_origin,
            allowed_points: v.allowed_points,
            debt_date: v.debt_date,
        }
    }
}

/// Accepted source document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentLoanDocumentView {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: crate::Form1098EPaper,
    /// Answer required by the printed box 2 checkbox
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Additional qualified interest you figured
    #[libertas_constrained_by("^.allowed_answers")]
    pub additional_interest: StudentLoanAdditionalInterest,
}
/// Additional interest for the accepted Form 1098-E
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StudentLoanAdditionalInterest {
    /// Box 2 is not checked
    NotNeeded,
    /// Additional qualified interest for checked box 2
    Checked {
        /// Origination fees and capitalized interest paid in 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        additional_interest: i64,
    },
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleLoanDocumentView {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Form 1098-VLI as printed
    #[libertas_read_only]
    pub value: crate::Form1098VliPaper,
    /// Part deducted on Schedule C, E or F
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub business: i64,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceDocumentView {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Printed document
    #[libertas_read_only]
    pub value: crate::Form1095APaper,
    /// Actual premiums excluding unlawful enrollees
    #[libertas_unordered]
    pub lawful_premiums: Vec<crate::MarketplaceLawfulPremiumMonthPage>,
    /// Family that actually enrolled the policy
    pub enrolling: crate::MarketplaceFamilyRefPage,
    /// Actual covered people by month
    #[libertas_unordered]
    pub coverage: Vec<crate::MarketplacePolicyCoveragePage>,
    /// Shared-family allocation periods
    #[libertas_unordered]
    pub allocation: Vec<crate::MarketplaceAllocationPeriodPage>,
    /// Actual payment problems or appeal facts
    #[libertas_unordered]
    pub payments: Vec<crate::MarketplacePaymentMonthPage>,
    /// Actual extra premium for children who are not medical dependents
    /// Actual cost and funding evidence is independent of the silver-plan allocation; zero differs from an unknown cost. These facts are not printed on Form 1095-A.
    #[libertas_unordered]
    pub medical_costs: Vec<crate::MarketplaceMedicalCostMonth>,
}
/// Personal answers for an accepted credit document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Document views, like income documents, own their complete schema payload inline.
#[allow(clippy::large_enum_variant)]
pub enum CreditDocumentView {
    /// Form 1097-BTC: tax credit bond
    CreditBond {
        /// Accepted bond and your personal records
        value: crate::TaxCreditBondPage,
    },
    /// Form 8986: partnership audit adjustment
    PartnershipAudit {
        /// Accepted source identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        id: i64,
        /// Form 8986 as printed
        #[libertas_read_only]
        document: crate::Form8986Paper,
        /// Your records for the adjustment
        #[libertas_read_only]
        answers: crate::BbaAuditAnswers,
    },
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct WageDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: WageDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct InterestDividendDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: InterestDividendDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct InvestmentSaleDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: InvestmentSaleDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct RetirementDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: RetirementDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct SelfEmploymentDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: SelfEmploymentDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct GovernmentPaymentDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: GovernmentPaymentDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct GamblingDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: GamblingDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct K1DocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: K1DocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct FarmingDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: FarmingDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct OtherIncomeDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: OtherIncomeDocumentRow,
}
/// Accepted source document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct CanceledDebtDocumentView {
    /// Document and its accepted answers
    #[libertas_read_only]
    pub value: CanceledDebtDocumentRow,
}
/// Accepted documents for this topic
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub enum TopicDocumentViews {
    /// Accepted category documents
    Wages {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<WageDocumentView>,
    },
    /// Accepted category documents
    InterestDividends {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<InterestDividendDocumentView>,
    },
    /// Accepted category documents
    InvestmentSales {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<InvestmentSaleDocumentView>,
    },
    /// Accepted category documents
    Retirement {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<RetirementDocumentView>,
    },
    /// Accepted category documents
    SelfEmployment {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<SelfEmploymentDocumentView>,
    },
    /// Accepted category documents
    GovernmentPayments {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<GovernmentPaymentDocumentView>,
    },
    /// Accepted category documents
    Gambling {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<GamblingDocumentView>,
    },
    /// Accepted category documents
    K1 {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<K1DocumentView>,
    },
    /// Accepted category documents
    Farming {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<FarmingDocumentView>,
    },
    /// Accepted category documents
    OtherIncome {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<OtherIncomeDocumentView>,
    },
    /// Accepted category documents
    CanceledDebt {
        /// Documents
        #[libertas_read_only]
        #[libertas_output_only]
        /// ----
        /// Document
        documents: Vec<CanceledDebtDocumentView>,
    },
}
impl From<&crate::Mortgage1098Source> for MortgageDocumentView {
    fn from(source: &crate::Mortgage1098Source) -> Self {
        let crate::Mortgage1098Source {
            id,
            form,
            assigned: _,
            owner_source: _,
            joint_share: _,
            property,
            lender,
            details,
            paid_share,
            insurance_contract,
            points,
            debt_date,
            refund_origin,
            answered: _,
        } = source;
        Self {
            id: *id,
            value: form.clone(),
            property: property.clone(),
            lender: lender.clone(),
            // Form 1098 box 4: a positive refund opens every refund-origin choice
            // (IRS Form 1098 instructions; Pub. 936, Refunds of interest).
            allowed_refund_origins: if form.box_4 > 0 { alloc::vec![0, 1, 2] } else { Vec::new() },
            refund_origin: refund_origin.clone(),
            details: details.clone().into(),
            paid_share: paid_share.clone(),
            insurance_contract: insurance_contract.clone(),
            allowed_points: if form.box_6 == 0 { alloc::vec![0] } else { alloc::vec![1, 2, 3] },
            points: points.clone(),
            debt_date: debt_date.clone(),
        }
    }
}
impl core::ops::Deref for MortgageDocumentView {
    type Target = crate::Form1098;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl From<&crate::StudentLoanSource> for StudentLoanDocumentView {
    fn from(source: &crate::StudentLoanSource) -> Self {
        let (additional_interest, allowed_answers) = if source.document.box_2 {
            (
                StudentLoanAdditionalInterest::Checked {
                    additional_interest: source.additional_interest,
                },
                alloc::vec![1],
            )
        } else {
            (StudentLoanAdditionalInterest::NotNeeded, alloc::vec![0])
        };
        Self { id: source.id, additional_interest, allowed_answers, value: source.document.clone() }
    }
}
impl core::ops::Deref for StudentLoanDocumentView {
    type Target = crate::Form1098EPaper;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl From<&crate::VehicleLoanSource> for VehicleLoanDocumentView {
    fn from(source: &crate::VehicleLoanSource) -> Self {
        Self { id: source.id, business: source.business, value: source.document.clone() }
    }
}
impl core::ops::Deref for VehicleLoanDocumentView {
    type Target = crate::Form1098VliPaper;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl From<Form1095ADocument> for MarketplaceDocumentView {
    fn from(value: Form1095ADocument) -> Self {
        Self {
            id: value.id,
            value: value.value,
            lawful_premiums: Vec::new(),
            enrolling: crate::MarketplaceFamilyRefPage::ThisReturn,
            coverage: Vec::new(),
            allocation: Vec::new(),
            payments: Vec::new(),
            medical_costs: Vec::new(),
        }
    }
}
impl core::ops::Deref for MarketplaceDocumentView {
    type Target = crate::Form1095APaper;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl From<crate::MarketplacePolicyRecord> for MarketplaceDocumentView {
    fn from(value: crate::MarketplacePolicyRecord) -> Self {
        Self {
            id: value.id,
            lawful_premiums: value.lawful_premiums.iter().cloned().map(Into::into).collect(),
            enrolling: value.enrolling.clone().into(),
            coverage: value.coverage.iter().cloned().map(Into::into).collect(),
            allocation: value.allocation.iter().cloned().map(Into::into).collect(),
            payments: value.payments.iter().cloned().map(Into::into).collect(),
            medical_costs: value.medical_costs.clone(),
            value: value.form,
        }
    }
}
impl From<MarketplaceDocumentView> for crate::MarketplacePolicyRecord {
    fn from(value: MarketplaceDocumentView) -> Self {
        let mut record: Self = value.value.into();
        record.id = value.id;
        record.lawful_premiums = value.lawful_premiums.into_iter().map(Into::into).collect();
        record.enrolling = value.enrolling.into();
        record.coverage = value.coverage.into_iter().map(Into::into).collect();
        record.allocation = value.allocation.into_iter().map(Into::into).collect();
        record.payments = value.payments.into_iter().map(Into::into).collect();
        record.medical_costs = value.medical_costs;
        record
    }
}

/// Prior-return follow-up for a printed vehicle-interest refund
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleRefundDocumentView {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Printed document
    #[libertas_read_only]
    pub value: crate::Form1098VliPaper,
    /// Prior deduction of the refunded interest
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    pub origin: Option<crate::VehicleRefundOrigin>,
}

/// Accepted payment-card and network payment parts
/// Edit these source answers separately before relating their accepted identities.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
#[libertas_link("SavePaymentParts")]
pub struct PaymentPartsAnswer {
    /// Accepted parts of box 1a
    pub value: Vec<crate::KPart>,
}

/// A recorded answer about an accepted credit document
// The More credits page's narrow twin of the canonical row (as
// EducationDocumentQuestion): answered rows only; unanswered ones are asked
// on the questions page (credit_questions).
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditDocumentQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Your answer
    #[libertas_constrained_by("^<CreditDocumentQuestion>.allowed_answers")]
    pub answer: CreditDocumentAnswer,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Accepted sources that already include a bond's designated income
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub credit_income_sources: Vec<crate::RecordChoice>,
}

/// Answer called for by the accepted credit document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum CreditDocumentAnswer {
    /// 2025 Form 1040 line 1z wages
    BbaPriorWages {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 taxable income
    BbaPriorTaxableIncome {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 income tax
    BbaPriorIncomeTax {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        value: Option<i64>,
    },
    /// 2025 filing status
    BbaPriorStatus {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::FederalStatus>,
    },
    /// Only one BBA-audit adjustment this year
    BbaSingleAdjustment {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// The 2025 return is the simple Tax Table case
    BbaSimplePriorReturn {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// No other effects of the adjustment
    BbaNoOtherEffects {
        /// [NullText]
        /// Not answered yet
        value: Option<bool>,
    },
    /// Form 8978 line 17 interest
    BbaLine17Interest {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_money("USD", 2)]
        interest: Option<i64>,
        /// [NullText]
        /// Not answered yet
        prepared: Option<bool>,
    },
    /// Bond class
    BondClass {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::FederalTaxCreditBondKind>,
    },
    /// Original bond issue date
    BondIssueDate {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        #[libertas_date_only]
        value: Option<u32>,
    },
    /// How the bond credit reached you
    BondOrigin {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        value: Option<crate::CreditOriginAnswer>,
    },
    /// Unused 2025 credit for this bond
    BondPriorCarry {
        /// Unused 2025 credit in whole dollars
        dollars: i64,
        /// [NullText]
        /// Not answered yet
        confirmed: Option<bool>,
    },
    /// This bond's share of the allowed credit
    BondAllowedShare {
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// The answer is
        dollars: Option<i64>,
        /// [NullText]
        /// Not answered yet
        confirmed: Option<bool>,
    },
}

impl CreditDocumentAnswer {
    pub(crate) fn from_canonical(answer: DocumentAnswer) -> Option<Self> {
        Some(match answer {
            DocumentAnswer::BbaPriorWages { value } => Self::BbaPriorWages { value },
            DocumentAnswer::BbaPriorTaxableIncome { value } => {
                Self::BbaPriorTaxableIncome { value }
            }
            DocumentAnswer::BbaPriorIncomeTax { value } => Self::BbaPriorIncomeTax { value },
            DocumentAnswer::BbaPriorStatus { value } => Self::BbaPriorStatus { value },
            DocumentAnswer::BbaSingleAdjustment { value } => Self::BbaSingleAdjustment { value },
            DocumentAnswer::BbaSimplePriorReturn { value } => Self::BbaSimplePriorReturn { value },
            DocumentAnswer::BbaNoOtherEffects { value } => Self::BbaNoOtherEffects { value },
            DocumentAnswer::BbaLine17Interest { interest, prepared } => {
                Self::BbaLine17Interest { interest, prepared }
            }
            DocumentAnswer::BondClass { value } => Self::BondClass { value },
            DocumentAnswer::BondIssueDate { value } => Self::BondIssueDate { value },
            DocumentAnswer::BondOrigin { value } => {
                Self::BondOrigin { value: value.map(Into::into) }
            }
            DocumentAnswer::BondPriorCarry { dollars, confirmed } => {
                Self::BondPriorCarry { dollars, confirmed }
            }
            DocumentAnswer::BondAllowedShare { dollars, confirmed } => {
                Self::BondAllowedShare { dollars, confirmed }
            }
            _ => return None,
        })
    }

    pub(crate) fn into_canonical(self) -> DocumentAnswer {
        match self {
            Self::BbaPriorWages { value } => DocumentAnswer::BbaPriorWages { value },
            Self::BbaPriorTaxableIncome { value } => {
                DocumentAnswer::BbaPriorTaxableIncome { value }
            }
            Self::BbaPriorIncomeTax { value } => DocumentAnswer::BbaPriorIncomeTax { value },
            Self::BbaPriorStatus { value } => DocumentAnswer::BbaPriorStatus { value },
            Self::BbaSingleAdjustment { value } => DocumentAnswer::BbaSingleAdjustment { value },
            Self::BbaSimplePriorReturn { value } => DocumentAnswer::BbaSimplePriorReturn { value },
            Self::BbaNoOtherEffects { value } => DocumentAnswer::BbaNoOtherEffects { value },
            Self::BbaLine17Interest { interest, prepared } => {
                DocumentAnswer::BbaLine17Interest { interest, prepared }
            }
            Self::BondClass { value } => DocumentAnswer::BondClass { value },
            Self::BondIssueDate { value } => DocumentAnswer::BondIssueDate { value },
            Self::BondOrigin { value } => {
                DocumentAnswer::BondOrigin { value: value.map(Into::into) }
            }
            Self::BondPriorCarry { dollars, confirmed } => {
                DocumentAnswer::BondPriorCarry { dollars, confirmed }
            }
            Self::BondAllowedShare { dollars, confirmed } => {
                DocumentAnswer::BondAllowedShare { dollars, confirmed }
            }
        }
    }

    // The variant's position, the value `allowed_answers` lists.
    pub(crate) fn index(&self) -> i32 {
        use libertas_macros::variant_index as v;
        (match self {
            Self::BbaPriorWages { .. } => v!(Self::BbaPriorWages),
            Self::BbaPriorTaxableIncome { .. } => v!(Self::BbaPriorTaxableIncome),
            Self::BbaPriorIncomeTax { .. } => v!(Self::BbaPriorIncomeTax),
            Self::BbaPriorStatus { .. } => v!(Self::BbaPriorStatus),
            Self::BbaSingleAdjustment { .. } => v!(Self::BbaSingleAdjustment),
            Self::BbaSimplePriorReturn { .. } => v!(Self::BbaSimplePriorReturn),
            Self::BbaNoOtherEffects { .. } => v!(Self::BbaNoOtherEffects),
            Self::BbaLine17Interest { .. } => v!(Self::BbaLine17Interest),
            Self::BondClass { .. } => v!(Self::BondClass),
            Self::BondIssueDate { .. } => v!(Self::BondIssueDate),
            Self::BondOrigin { .. } => v!(Self::BondOrigin),
            Self::BondPriorCarry { .. } => v!(Self::BondPriorCarry),
            Self::BondAllowedShare { .. } => v!(Self::BondAllowedShare),
        }) as i32
    }
}
