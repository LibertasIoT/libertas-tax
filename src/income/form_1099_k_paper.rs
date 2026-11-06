use super::{FederalForm1099K, Form1099K, KPart};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 1099-K, Rev. December 2026, Copy B:
// https://www.irs.gov/pub/irs-pdf/f1099k.pdf#page=4
/// 1099-K
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-K exactly as printed, including its calendar year, corrected checkbox, filer and payee party blocks, filer telephone number, filer and transaction checkboxes, PSE name and telephone number, account number, and boxes 1a through 8. Check the printed calendar year against the supplied tax year and report a mismatch as an error. Preserve identification numbers, merchant category code and account number as text, including leading zeros, masking and punctuation. Copy each printed TTOC as one code record, including 000; use an empty list when blank. Use empty text for absent text, false for unchecked checkboxes and null for clearly blank number and money boxes; never substitute numeric zero or the text 0 for a blank field. Copy each populated state row once in printed order; never infer a state from an address. Multiple copies of the same form are one document. Never infer a value, split box 1a, classify payments or choose their activity. An illegible amount, conflicting value or populated state row without a legible state is an error.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.gross,@.card_not_present,@.cash_tips,@.withholding,@.january,@.february,@.march,@.april,@.may,@.june,@.july,@.august,@.september,@.october,@.november,@.december,@.account_number"
)]
pub struct Form1099KPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// FILER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// Telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub filer_telephone: String,
    /// Payment settlement entity (PSE)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub filer_is_pse: bool,
    /// Electronic payment facilitator (EPF)/Other third party
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub filer_is_epf_or_other: bool,
    /// Payment card
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub payment_card: bool,
    /// Third party network
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub third_party_network: bool,
    /// PAYEE'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// PSE'S name and telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub pse_name_and_telephone: String,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1a Gross amount of payment card/third party network transactions
    /// [NullText]
    /// Box 1a Gross amount of payment card/third party network transactions is blank
    /// [SetText]
    /// Box 1a Gross amount of payment card/third party network transactions
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub gross: Option<i64>,
    /// Box 1b Card Not Present transactions
    /// [NullText]
    /// Box 1b Card Not Present transactions is blank
    /// [SetText]
    /// Box 1b Card Not Present transactions
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub card_not_present: Option<i64>,
    /// Box 1c Cash tips
    /// [NullText]
    /// Box 1c Cash tips is blank
    /// [SetText]
    /// Box 1c Cash tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub cash_tips: Option<i64>,
    /// Box 1d TTOC
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: Vec<crate::TippedOccupationCode>,
    /// Box 2 Merchant category code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub merchant_category_code: String,
    /// Box 3 Number of payment transactions
    /// [NullText]
    /// Box 3 Number of payment transactions is blank
    /// [SetText]
    /// Box 3 Number of payment transactions
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub payment_transactions: Option<u32>,
    /// Box 4 Federal income tax withheld
    /// [NullText]
    /// Box 4 Federal income tax withheld is blank
    /// [SetText]
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 5a January
    /// [NullText]
    /// Box 5a January is blank
    /// [SetText]
    /// Box 5a January
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub january: Option<i64>,
    /// Box 5b February
    /// [NullText]
    /// Box 5b February is blank
    /// [SetText]
    /// Box 5b February
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub february: Option<i64>,
    /// Box 5c March
    /// [NullText]
    /// Box 5c March is blank
    /// [SetText]
    /// Box 5c March
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub march: Option<i64>,
    /// Box 5d April
    /// [NullText]
    /// Box 5d April is blank
    /// [SetText]
    /// Box 5d April
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub april: Option<i64>,
    /// Box 5e May
    /// [NullText]
    /// Box 5e May is blank
    /// [SetText]
    /// Box 5e May
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub may: Option<i64>,
    /// Box 5f June
    /// [NullText]
    /// Box 5f June is blank
    /// [SetText]
    /// Box 5f June
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub june: Option<i64>,
    /// Box 5g July
    /// [NullText]
    /// Box 5g July is blank
    /// [SetText]
    /// Box 5g July
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub july: Option<i64>,
    /// Box 5h August
    /// [NullText]
    /// Box 5h August is blank
    /// [SetText]
    /// Box 5h August
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub august: Option<i64>,
    /// Box 5i September
    /// [NullText]
    /// Box 5i September is blank
    /// [SetText]
    /// Box 5i September
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub september: Option<i64>,
    /// Box 5j October
    /// [NullText]
    /// Box 5j October is blank
    /// [SetText]
    /// Box 5j October
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub october: Option<i64>,
    /// Box 5k November
    /// [NullText]
    /// Box 5k November is blank
    /// [SetText]
    /// Box 5k November
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub november: Option<i64>,
    /// Box 5l December
    /// [NullText]
    /// Box 5l December is blank
    /// [SetText]
    /// Box 5l December
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub december: Option<i64>,
    /// Boxes 6 State income tax withheld; 7 State identification no.; 8 State
    #[libertas_llm_doc_field]
    pub state_details: Vec<Form1099KStateRow>,
}

/// State row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1099KStateRow {
    /// Box 6 State income tax withheld
    /// [NullText]
    /// Box 8 State income tax withheld is blank
    /// [SetText]
    /// Box 8 State income tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 7 State identification no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_state_id: String,
    /// Box 8 State
    #[libertas_llm_doc_field]
    pub state: crate::TaxState,
}

/// Form 1099-K and its payment-part answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1099Source {
    /// Form 1099-K as printed
    pub form: Form1099KPaper,
    /// Where the payments in box 1a belong
    pub parts: Vec<KPart>,
}

impl From<&Form1099KPaper> for Form1099K {
    fn from(paper: &Form1099KPaper) -> Self {
        let Form1099KPaper {
            calendar_year: _,
            corrected: _,
            payer,
            filer_telephone: _,
            filer_is_pse: _,
            filer_is_epf_or_other: _,
            payment_card: _,
            third_party_network: _,
            recipient,
            pse_name_and_telephone: _,
            account_number: _,
            gross,
            card_not_present: _,
            cash_tips,
            tip_occupation_codes,
            merchant_category_code: _,
            payment_transactions: _,
            withholding,
            january: _,
            february: _,
            march: _,
            april: _,
            may: _,
            june: _,
            july: _,
            august: _,
            september: _,
            october: _,
            november: _,
            december: _,
            state_details,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            gross: gross.unwrap_or(0),
            cash_tips: cash_tips.unwrap_or(0),
            tip_occupation_codes: tip_occupation_codes.clone(),
            withholding: withholding.unwrap_or(0),
            parts: Vec::new(),
            state_details: crate::StateWithholdingFields {
                states: state_details
                    .iter()
                    .map(|row| {
                        let Form1099KStateRow { withholding, payer_state_id, state } = row;
                        crate::StateTaxWithholding {
                            state: *state,
                            payer_state_id: payer_state_id.clone(),
                            withholding: *withholding,
                        }
                    })
                    .collect(),
            },
        }
    }
}

impl From<&K1099Source> for Form1099K {
    fn from(source: &K1099Source) -> Self {
        let K1099Source { form, parts } = source;
        let mut natural = Self::from(form);
        natural.parts = parts.clone();
        natural
    }
}

impl From<&K1099Source> for FederalForm1099K {
    fn from(source: &K1099Source) -> Self {
        Form1099K::from(source).record()
    }
}

impl K1099Source {
    pub(crate) fn printed(form: Form1099KPaper) -> Self {
        Self { form, parts: Vec::new() }
    }
}

impl Form1099KPaper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        super::k_input_issue(&Form1099K::from(self), false)
    }
}

impl From<&crate::StateTaxWithholding> for Form1099KStateRow {
    fn from(row: &crate::StateTaxWithholding) -> Self {
        let crate::StateTaxWithholding { state, payer_state_id, withholding } = row;
        Self { withholding: *withholding, payer_state_id: payer_state_id.clone(), state: *state }
    }
}
