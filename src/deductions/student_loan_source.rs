use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 1098-E (2026), Copy B: https://www.irs.gov/pub/irs-pdf/f1098e.pdf
/// Form 1098-E Student Loan Interest Statement as printed
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the printed calendar year, the CORRECTED checkbox (true only when checked), and each printed party block into lender, borrower. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Copy the lender's telephone number and the account number exactly as printed; absent text is empty text. Never infer a recipient from household names. Read this one Form 1098-E (Student Loan Interest Statement) exactly. Compare the printed calendar year with the supplied year; report a mismatch. Read box 1 student loan interest received by the lender as printed; a blank, illegible or conflicting box 1 is an error, never zero or a guess. Read the printed box 2 checkbox: true when checked, false when clearly unchecked. The filer figures any additional interest; never infer that amount. Do not infer whether the filer is legally obligated to repay the loan or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.lender.federal_id,@.borrower.federal_id,@.calendar_year,@.box_1,@.account_number"
)]
pub struct Form1098EPaper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// RECIPIENT'S/LENDER'S name, street address, city/town, state/province, country, ZIP/foreign code, and TIN
    #[libertas_llm_doc_field]
    pub lender: crate::PrintedParty,
    /// RECIPIENT'S/LENDER'S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub lender_telephone: String,
    /// BORROWER'S name, street address, city/town, state/province, country, ZIP/foreign code, and TIN
    #[libertas_llm_doc_field]
    pub borrower: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Student loan interest received by lender
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_1: i64,
    /// Box 2 Check if box 1 does not include loan origination fees and/or capitalized interest, and the loan was made before September 1, 2004
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub box_2: bool,
}
/// Form 1098-E: the internal model the calculation reads
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1098E {
    /// RECIPIENT'S/LENDER'S name, TIN and address
    pub lender: crate::PrintedParty,
    /// BORROWER'S name, TIN and address
    pub borrower: crate::PrintedParty,
    /// Box 1 student loan interest received by lender
    pub interest: i64,
    /// Box 2 checked
    pub box_2: bool,
}
impl From<&Form1098EPaper> for Form1098E {
    fn from(paper: &Form1098EPaper) -> Self {
        let Form1098EPaper {
            // Printed identification and bookkeeping boxes; no figure reads them.
            calendar_year: _,
            corrected: _,
            lender,
            lender_telephone: _,
            borrower,
            account_number: _,
            box_1,
            box_2,
        } = paper;
        Self { lender: lender.clone(), borrower: borrower.clone(), interest: *box_1, box_2: *box_2 }
    }
}
/// Accepted Form 1098-E, its answer and recipient association
/// The printed form and the one answer its box 2 calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct StudentLoanSource {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098-E as printed
    pub document: Form1098EPaper,
    /// Additional qualified interest you figured (origination fees and capitalized interest paid in 2026)
    /// Asked only when box 2 is checked. Spread each loan origination fee charged for the use of money over the loan's term and include the part for the months you made payments in 2026. Include capitalized interest to the extent your 2026 payments of principal paid it off: a payment applies first to unpaid stated interest, then to the origination fee, then to capitalized interest, then to principal. See Publication 970, Student Loan Interest Deduction, for a worked example. Enter zero if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub additional_interest: i64,
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
impl StudentLoanSource {
    /// A newly entered form: no answer is given yet.
    pub(crate) fn printed(id: i64, document: Form1098EPaper) -> Self {
        Self {
            id,
            document,
            additional_interest: 0,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        }
    }
    /// Box 1 plus the qualified interest the filer figured for a checked box 2.
    pub(crate) fn qualified_interest(&self) -> i64 {
        let model = Form1098E::from(&self.document);
        model.interest + if model.box_2 { self.additional_interest } else { 0 }
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.document.box_1)
            && (0..=100000000000).contains(&self.additional_interest)
    }
}

/// Student-loan interest to evaluate
/// Zero amounts and no forms when there is none.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentLoanClaim {
    /// Qualified student-loan interest paid with no Form 1098-E
    /// Exclude ordinary reimbursements. If the interest here or on the forms includes health-service program-paid interest, report that overlap with the program assistance under Income; the App removes it once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub student_loan_interest: i64,
    /// Forms 1098-E
    #[libertas_unordered]
    /// ----
    /// Form 1098-E
    pub forms: Vec<crate::StudentLoanSource>,
    /// Interest your employer paid for you under an educational assistance program
    /// The tax-free part of the interest above that your employer paid, to you or to the lender, under its written educational assistance program. It cannot also be deducted, so the App subtracts it before the $2,500 limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employer_paid_interest: i64,
}
impl StudentLoanClaim {
    /// No student-loan interest.
    pub(crate) const NONE: Self =
        Self { student_loan_interest: 0, forms: Vec::new(), employer_paid_interest: 0 };
    /// Whether any interest or form is entered.
    pub(crate) fn any(&self) -> bool {
        *self != Self::NONE
    }
}
