pub use crate::student_loan_source::{
    Form1098E, Form1098EPaper, StudentLoanClaim, StudentLoanSource,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Student loan interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentLoanInterest {
    /// Forms 1098-E
    /// Only loans you are legally obligated to repay.
    #[libertas_unordered]
    #[libertas_read_only]
    /// ----
    /// Form 1098-E
    pub forms: Vec<crate::StudentLoanDocumentView>,
    /// Interest paid with no Form 1098-E
    /// Qualified student loan interest no lender reported to you, such as less than $600 paid to one lender. Use the lender's statements. Only loans you are legally obligated to repay.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_interest: i64,
    /// Interest your employer paid for you under an educational assistance program
    /// The tax-free part of the interest above that your employer paid, to you or to the lender, under its written educational assistance program. Your employer can tell you; it is not in your W-2 box 1 wages. Leave out anything your employer added to your wages. It cannot also be deducted, so the App subtracts it before the $2,500 limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employer_paid_interest: i64,
}
/// Form 1098-E in the Document center
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentLoanDocument {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098-E as printed
    #[libertas_llm_doc_field]
    pub form: Form1098EPaper,
}
