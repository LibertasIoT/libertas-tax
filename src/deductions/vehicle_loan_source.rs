use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 1098-VLI (December 2026), Copy B: https://www.irs.gov/pub/irs-dft/f1098vli--dft.pdf
/// Form 1098-VLI Vehicle Loan Interest Statement, or your lender's statement of 2026 vehicle loan interest
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into lender, payer_of_record. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read this one Form 1098-VLI (Vehicle Loan Interest Statement), or the lender's statement of vehicle loan interest when no Form 1098-VLI was sent, exactly. Check its calendar year against the supplied year; report a mismatch. Return the lender's name as printed and the interest received in the year: Form 1098-VLI box 1, or the interest paid in the year on the lender's statement, not principal, fees or a payoff amount. A blank, illegible or conflicting name or interest amount is an error, never zero or a guess. Copy the CORRECTED checkbox exactly; unchecked or blank means false. Copy the lender's telephone number and the account number as printed; absent text is empty text, never the text 0. Read box 2a vehicle year as the printed number, box 2b make, box 2c model and box 2d VIN as printed text; leave them empty when blank. Read box 3a loan origination date and box 3b loan acquisition date as valid printed calendar dates, or null when clearly blank; never copy one into the other. Read box 4 outstanding principal as printed, null when blank. Read box 5 refunds of overpaid interest, returning null when blank; it is a refund of prior-year interest and must not be subtracted from box 1. Read box 6 original use and box 7 U.S. final assembly as printed checkboxes, returning true only when checked. Never infer either checkbox from the borrower or vehicle. Do not infer the business share, prior-year deduction, tax benefit or other facts absent from the document. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.lender.federal_id,@.payer_of_record.federal_id,@.calendar_year,@.box_2a,@.box_2b,@.box_2c,@.box_2d,@.box_1,@.box_4,@.box_5,@.account_number"
)]
pub struct Form1098VliPaper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// RECIPIENT'S/LENDER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub lender: crate::printed_state::PrintedParty,
    /// RECIPIENT'S/LENDER'S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub lender_telephone: String,
    /// PAYER OF RECORD'S name, street address (including apt. no.), city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub payer_of_record: crate::printed_state::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Vehicle loan interest received by lender
    /// Interest only, not principal or fees. If the lender sent no form, use its statement.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_1: i64,
    /// Box 2a Year
    /// [NullText]
    /// No vehicle year is printed in box 2a.
    /// [SetText]
    /// The vehicle year printed in box 2a is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_number(min = 1900, max = 2100)]
    #[libertas_default($null)]
    pub box_2a: Option<u32>,
    /// Box 2b Make
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_2b: String,
    /// Box 2c Model
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_2c: String,
    /// Box 2d VIN
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_2d: String,
    /// Box 3a Loan origination date
    /// [NullText]
    /// No loan origination date is printed in box 3a.
    /// [SetText]
    /// The loan origination date printed in box 3a is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_3a: Option<u32>,
    /// Box 3b Loan acquisition date
    /// [NullText]
    /// No loan acquisition date is printed in box 3b.
    /// [SetText]
    /// The loan acquisition date printed in box 3b is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_3b: Option<u32>,
    /// Box 4 Outstanding principal
    /// [NullText]
    /// Box 4 Outstanding principal is blank
    /// [SetText]
    /// Printed box 4 outstanding principal
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_4: Option<i64>,
    /// Box 5 Refund of overpaid interest
    /// [NullText]
    /// Box 5 Refund of overpaid interest is blank
    /// [SetText]
    /// Printed box 5 refund of overpaid interest
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_5: Option<i64>,
    /// Box 6 Check if original use of the vehicle began with the payer of record
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6: bool,
    /// Box 7 Check if final assembly of the vehicle occurred within the United States
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_7: bool,
}

/// Accepted Form 1098-VLI and the answers it calls for.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct VehicleLoanSource {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098-VLI as printed
    pub document: Form1098VliPaper,
    /// Accepted recipient association
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
    /// Part deducted on Schedule C, E or F
    /// The business or rental share of this interest, if you use the vehicle for work.
    #[libertas_hidden]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub business: i64,
}

impl VehicleLoanSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(id: i64, document: Form1098VliPaper) -> Self {
        Self {
            id,
            document,
            assigned: crate::document_parties::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
            business: 0,
        }
    }
}
