use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-MISC
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-MISC exactly as printed, including its calendar year, corrected checkbox, party blocks, payer telephone number, account number, FATCA checkbox and boxes 1 through 18. Preserve identification numbers as text, including masking, punctuation and leading zeros. Copy each printed TTOC as one code record, including 000; use an empty list when blank. Box 13b holds only three-digit codes: the Copy B notice printed beside it is not part of the box, and a box 13b with no printed code is blank, never illegible. Use empty text for absent text, never the text 0 for a blank phone or account number; use false for an unchecked checkbox and null for a clearly blank money box. Preserve null for blank state income and zero for printed zero. Read each populated state row once; do not infer a state from an address or copy federal income into a blank state income box. Multiple copies of the same form are one form. Never infer a value, classification or personal answer. Check the printed year against the supplied year and report a mismatch or illegible or conflicting printed values as an error.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.rents,@.royalties,@.other,@.withholding,@.fishing,@.medical,@.substitute,@.crop_insurance,@.attorney,@.fish_sales,@.section_409a_deferrals,@.cash_tips,@.overtime,@.deferred,@.account_number"
)]
pub struct Form1099MiscPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: i32,
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
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// FATCA filing requirement
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Rents
    /// [NullText]
    /// Box 1 Rents is blank
    /// [SetText]
    /// Box 1 Rents
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub rents: Option<i64>,
    /// Box 2 Royalties
    /// [NullText]
    /// Box 2 Royalties is blank
    /// [SetText]
    /// Box 2 Royalties
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub royalties: Option<i64>,
    /// Box 3 Other income
    /// [NullText]
    /// Box 3 Other income is blank
    /// [SetText]
    /// Box 3 Other income
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub other: Option<i64>,
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
    /// Box 5 Fishing boat proceeds
    /// [NullText]
    /// Box 5 Fishing boat proceeds is blank
    /// [SetText]
    /// Box 5 Fishing boat proceeds
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub fishing: Option<i64>,
    /// Box 6 Medical and health care payments
    /// [NullText]
    /// Box 6 Medical and health care payments is blank
    /// [SetText]
    /// Box 6 Medical and health care payments
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub medical: Option<i64>,
    /// Box 7 Payer made direct sales totaling $5,000 or more of consumer products to recipient for resale
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub direct_sales: bool,
    /// Box 8 Substitute payments in lieu of dividends or interest
    /// [NullText]
    /// Box 8 Substitute payments in lieu of dividends or interest is blank
    /// [SetText]
    /// Box 8 Substitute payments in lieu of dividends or interest
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub substitute: Option<i64>,
    /// Box 9 Crop insurance proceeds
    /// [NullText]
    /// Box 9 Crop insurance proceeds is blank
    /// [SetText]
    /// Box 9 Crop insurance proceeds
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub crop_insurance: Option<i64>,
    /// Box 10 Gross proceeds paid to an attorney
    /// [NullText]
    /// Box 10 Gross proceeds paid to an attorney is blank
    /// [SetText]
    /// Box 10 Gross proceeds paid to an attorney
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub attorney: Option<i64>,
    /// Box 11 Fish purchased for resale
    /// [NullText]
    /// Box 11 Fish purchased for resale is blank
    /// [SetText]
    /// Box 11 Fish purchased for resale
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub fish_sales: Option<i64>,
    /// Box 12 Section 409A deferrals
    /// [NullText]
    /// Box 12 Section 409A deferrals is blank
    /// [SetText]
    /// Box 12 Section 409A deferrals
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section_409a_deferrals: Option<i64>,
    /// Box 13a Cash tips
    /// [NullText]
    /// Box 13a Cash tips is blank
    /// [SetText]
    /// Box 13a Cash tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub cash_tips: Option<i64>,
    /// Box 13b TTOC
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: Vec<crate::TippedOccupationCode>,
    /// Box 14 Overtime compensation
    /// [NullText]
    /// Box 14 Overtime compensation is blank
    /// [SetText]
    /// Box 14 Overtime compensation
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub overtime: Option<i64>,
    /// Box 15 Nonqualified deferred compensation
    /// [NullText]
    /// Box 15 Nonqualified deferred compensation is blank
    /// [SetText]
    /// Box 15 Nonqualified deferred compensation
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub deferred: Option<i64>,
    /// Boxes 16 State tax withheld, 17 State/Payer's state no., 18 State income
    #[libertas_llm_doc_field]
    pub state_details: Misc1099StateBoxes,
}

impl Form1099MiscPaper {
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let form = crate::Form1099Misc::from(self);
        crate::information_returns::misc_printed_issue(
            form.withholding,
            [
                form.rents,
                form.royalties,
                form.other,
                form.fishing,
                form.medical,
                form.substitute,
                form.crop_insurance,
                form.attorney,
                form.fish_sales,
                form.deferred,
            ],
        )
    }
}

// The printed form is the source; classifications and accepted component identities
// belong to the return's source record.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Misc1099Source {
    pub form: Form1099MiscPaper,
    pub component_ids: crate::MiscComponentIds,
    pub other_use: Option<crate::IncomeUse>,
    pub client_funds: i64,
}

impl From<&Form1099MiscPaper> for crate::Form1099Misc {
    fn from(paper: &Form1099MiscPaper) -> Self {
        let Form1099MiscPaper {
            calendar_year: _,
            corrected: _,
            payer,
            payer_telephone: _,
            recipient,
            fatca_filing_requirement: _,
            account_number: _,
            rents,
            royalties,
            other,
            withholding,
            fishing,
            medical,
            direct_sales: _,
            substitute,
            crop_insurance,
            attorney,
            fish_sales,
            section_409a_deferrals: _,
            cash_tips,
            tip_occupation_codes,
            overtime,
            deferred,
            state_details,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            component_ids: Default::default(),
            other_use: None,
            client_funds: 0,
            rents: rents.unwrap_or(0),
            royalties: royalties.unwrap_or(0),
            other: other.unwrap_or(0),
            withholding: withholding.unwrap_or(0),
            fishing: fishing.unwrap_or(0),
            medical: medical.unwrap_or(0),
            substitute: substitute.unwrap_or(0),
            crop_insurance: crop_insurance.unwrap_or(0),
            attorney: attorney.unwrap_or(0),
            fish_sales: fish_sales.unwrap_or(0),
            cash_tips: cash_tips.unwrap_or(0),
            tip_occupation_codes: tip_occupation_codes.clone(),
            overtime: overtime.unwrap_or(0),
            deferred: deferred.unwrap_or(0),
            state_details: state_details.into(),
        }
    }
}

impl From<&Misc1099Source> for crate::Form1099Misc {
    fn from(source: &Misc1099Source) -> Self {
        let Misc1099Source { form, component_ids, other_use, client_funds } = source;
        let mut value = Self::from(form);
        value.component_ids = component_ids.clone();
        value.other_use = other_use.clone();
        value.client_funds = *client_funds;
        value
    }
}

impl From<&Misc1099Source> for crate::FederalForm1099Misc {
    fn from(source: &Misc1099Source) -> Self {
        Self::from(&crate::Form1099Misc::from(source))
    }
}

impl Misc1099Source {
    pub(crate) fn printed(form: Form1099MiscPaper) -> Self {
        Self { form, component_ids: Default::default(), other_use: None, client_funds: 0 }
    }
}

/// State boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Misc1099StateBoxes {
    /// Boxes 16–18
    #[libertas_llm_doc_field]
    pub states: Vec<Misc1099StateRow>,
}

/// State row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Misc1099StateRow {
    /// Box 16 State tax withheld
    /// [NullText]
    /// Box 16 State tax withheld is blank
    /// [SetText]
    /// Box 16 State tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 17 State
    #[libertas_llm_doc_field]
    pub state: crate::TaxState,
    /// Box 17 Payer's state no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_state_id: String,
    /// Box 18 State income
    /// A blank box stays unreported; a printed zero stays zero.
    /// [NullText]
    /// Box 18 is blank
    /// [SetText]
    /// State income is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub income: Option<i64>,
}

impl From<&Misc1099StateBoxes> for crate::StateIncomeFields {
    fn from(boxes: &Misc1099StateBoxes) -> Self {
        let Misc1099StateBoxes { states } = boxes;
        Self {
            states: states
                .iter()
                .map(|row| {
                    let Misc1099StateRow { withholding, state, payer_state_id, income } = row;
                    crate::StateIncomeTax {
                        state: *state,
                        payer_state_id: payer_state_id.clone(),
                        income: *income,
                        withholding: *withholding,
                    }
                })
                .collect(),
        }
    }
}

impl From<&crate::StateIncomeFields> for Misc1099StateBoxes {
    fn from(fields: &crate::StateIncomeFields) -> Self {
        Self {
            states: fields
                .states
                .iter()
                .map(|row| Misc1099StateRow {
                    withholding: row.withholding,
                    state: row.state,
                    payer_state_id: row.payer_state_id.clone(),
                    income: row.income,
                })
                .collect(),
        }
    }
}
