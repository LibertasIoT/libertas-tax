use super::{FederalForm1099Nec, Form1099Nec, IncomeUse};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 1099-NEC, Rev. December 2026, Copy B:
// https://www.irs.gov/pub/irs-pdf/f1099nec.pdf#page=4
/// 1099-NEC
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-NEC exactly as printed, including the calendar year, corrected checkbox, payer and recipient party blocks, payer telephone number, account number, and boxes 1a through 7. Check the printed calendar year against the supplied tax year and report a mismatch as an error. Preserve identification numbers and account numbers as text, including leading zeros, masking and punctuation. Copy each printed TTOC as one code record, including 000; use an empty list when blank. Return null for clearly blank amount boxes and empty text for absent text fields, never numeric zero or the text 0 for a blank field. Box 3 is included in box 1a: never add it to compensation or calculate its tax. Read box 2 as a checkbox. Do not classify the income or choose its business; those are separate answers. Copy each populated state row once into state_details, preserving a blank state income as null and a printed zero as zero. Never infer state income from federal compensation or a state from an address. An illegible amount, conflicting value, or populated state row without a legible state is an error. Multiple copies of the same form represent one document and must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.compensation,@.cash_tips,@.overtime,@.golden_parachute,@.withholding,@.account_number"
)]
pub struct Form1099NecPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// Telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: alloc::string::String,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: alloc::string::String,
    /// Box 1a Nonemployee compensation
    /// [NullText]
    /// Box 1a Nonemployee compensation is blank
    /// [SetText]
    /// Box 1a Nonemployee compensation
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub compensation: Option<i64>,
    /// Box 1b Cash tips
    /// [NullText]
    /// Box 1b Cash tips is blank
    /// [SetText]
    /// Box 1b Cash tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub cash_tips: Option<i64>,
    /// Box 1c TTOC
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 1d Overtime compensation
    /// [NullText]
    /// Box 1d Overtime compensation is blank
    /// [SetText]
    /// Box 1d Overtime compensation
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub overtime: Option<i64>,
    /// Box 2 Payer made direct sales totaling $5,000 or more of consumer products to recipient for resale
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub direct_sales: bool,
    /// Box 3 Excess golden parachute payments
    /// [NullText]
    /// Box 3 Excess golden parachute payments is blank
    /// [SetText]
    /// Box 3 Excess golden parachute payments
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub golden_parachute: Option<i64>,
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
    /// Boxes 5 State tax withheld; 6 State/Payer's state no.; 7 State income
    #[libertas_llm_doc_field]
    pub state_details: crate::StateIncomeFields,
}

/// Form 1099-NEC and its income classification
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Nec1099Source {
    /// Form 1099-NEC as printed
    pub form: Form1099NecPaper,
    /// Where box 1a belongs
    /// [NullText]
    /// Relate this amount from its activity
    pub belongs: Option<IncomeUse>,
}

impl Nec1099Source {
    pub(crate) fn printed(form: Form1099NecPaper) -> Self {
        Self { form, belongs: None }
    }

    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.belongs = accepted.belongs.clone();
    }

    // The Form 8919 record this form is figured as when its pay is employee pay.
    pub(crate) fn employee_wages(&self) -> Option<crate::EmployeeWages> {
        let form = FederalForm1099Nec::from(self);
        self.belongs.as_ref()?.employee_wages(
            &self.form.payer,
            &self.form.recipient,
            form.compensation,
            form.withholding,
        )
    }
}

impl From<&Form1099NecPaper> for Form1099Nec {
    fn from(form: &Form1099NecPaper) -> Self {
        let Form1099NecPaper {
            // Identifying and informational fields do not change federal figures.
            calendar_year: _,
            corrected: _,
            payer_telephone: _,
            account_number: _,
            direct_sales: _,
            payer,
            recipient,
            compensation,
            cash_tips,
            tip_occupation_codes,
            overtime,
            golden_parachute,
            withholding,
            state_details,
        } = form;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            compensation: compensation.unwrap_or(0),
            cash_tips: cash_tips.unwrap_or(0),
            tip_occupation_codes: tip_occupation_codes.clone(),
            overtime: overtime.unwrap_or(0),
            golden_parachute: golden_parachute.unwrap_or(0),
            withholding: withholding.unwrap_or(0),
            belongs: None,
            state_details: state_details.clone(),
        }
    }
}

impl From<&Nec1099Source> for Form1099Nec {
    fn from(source: &Nec1099Source) -> Self {
        let Nec1099Source { form, belongs } = source;
        let mut natural = Self::from(form);
        natural.belongs = belongs.clone();
        natural
    }
}

impl From<&Nec1099Source> for FederalForm1099Nec {
    fn from(source: &Nec1099Source) -> Self {
        Form1099Nec::from(source).record()
    }
}

impl Form1099NecPaper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        super::nec_issue(&Form1099Nec::from(self))
    }
}
