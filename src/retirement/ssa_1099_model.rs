// ssa_1099 items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

/// A line printed in the description of SSA-1099 benefits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SocialBenefitDescription {
    /// Printed description
    #[libertas_llm_doc_field]
    pub description: SocialBenefitDescriptionKind,
    /// Printed amount
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
}
/// Printed benefit-description category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SocialBenefitDescriptionKind {
    /// Medicare Part B premiums
    MedicarePartB,
    /// Medicare premiums reported as one total
    MedicareTotal,
    /// Medicare Part C premiums
    MedicarePartC,
    /// Medicare Part D premiums
    MedicarePartD,
    /// Benefits for an earlier year
    PriorYear {
        /// Year printed with the payment
        #[libertas_llm_doc_field(allow_empty = false)]
        year: u32,
    },
    /// Another printed description
    Other {
        /// Description exactly as printed
        /// Retain punctuation belonging to the label, including a trailing colon.
        #[libertas_llm_doc_field(allow_empty = false)]
        label: String,
    },
}
/// Printed gross benefits, repayments and their description lines
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct SocialPrintedBoxes {
    /// Box 3 benefits paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_paid: i64,
    /// Description of amounts in box 3
    pub paid_description: Vec<SocialBenefitDescription>,
    /// Box 4 benefits repaid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_repaid: i64,
    /// Description of amounts in box 4
    pub repaid_description: Vec<SocialBenefitDescription>,
}
/// Social Security or equivalent railroad benefits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSocial {
    /// Retained printed source lines
    pub form_boxes: SocialPrintedBoxes,
    /// SSA-1099 or RRB-1099 box 5 net benefits
    /// Enter a parenthesized box 5 amount as negative; repayments already included in box 5 must not be deducted again. Combine SSA and RRB benefits for this return; joint returns also combine both spouses. Do not subtract Medicare premiums or workers' compensation offsets from box 5.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub benefits: i64,
    /// Federal income tax withheld — SSA-1099 box 6 or RRB-1099 box 10
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}
/// Accepted Social Security benefit statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum SocialBenefitStatement {
    /// SSA-1099
    Ssa1099(Ssa1099Paper),
    /// RRB-1099
    Rrb1099(crate::rrb_document::Rrb1099Paper),
}

impl From<&SocialBenefitStatement> for FederalSocial {
    fn from(statement: &SocialBenefitStatement) -> Self {
        match statement {
            SocialBenefitStatement::Ssa1099(form) => form.into(),
            SocialBenefitStatement::Rrb1099(form) => crate::Rrb1099::from(form).record(),
        }
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&Ssa1099> for FederalSocial {
    fn from(form: &Ssa1099) -> Self {
        let Ssa1099 {
            // The parties and the claim number identify the statement; no figure reads them.
            issuer: _,
            recipient: _,
            benefits_paid,
            paid_description,
            benefits_repaid,
            repaid_description,
            benefits,
            withholding,
            claim_number: _,
        } = form;
        FederalSocial {
            form_boxes: SocialPrintedBoxes {
                benefits_paid: *benefits_paid,
                paid_description: paid_description.clone(),
                benefits_repaid: *benefits_repaid,
                repaid_description: repaid_description.clone(),
            },
            benefits: *benefits,
            withholding: *withholding,
        }
    }
}
/// SSA-1099
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into issuer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the SSA-1099 exactly. Check its tax year against the supplied year; report a mismatch. Box 5 must show an amount; report a blank box 5 as an error. Return box 5 net benefits, with a leading minus sign when the amount is shown in parentheses. Return zero for a clearly blank box 6. Copy box 8 claim number exactly as printed; use empty text when it is absent. Report illegible or conflicting values as errors. Read box 3 benefits paid and box 4 benefits repaid, returning zero when blank. Read each printed line in the descriptions of boxes 3 and 4 once, preserving its amount and description. Classify a line as Medicare Part B, C or D only when the printed description names that coverage, and as PriorYear only when it expressly states the year of the benefits; preserve any other description as Other with its exact label. Never infer an earlier-year payment or a premium absent from the statement. This import is for SSA-1099, not RRB-1099.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Ssa1099 {
    /// Statement issuer as printed
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// Boxes 1 and 2 recipient name, identifying number and mailing address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 3 benefits paid
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_paid: i64,
    /// Description of amounts in box 3
    #[libertas_llm_doc_field]
    pub paid_description: Vec<SocialBenefitDescription>,
    /// Box 4 benefits repaid
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_repaid: i64,
    /// Description of amounts in box 4
    #[libertas_llm_doc_field]
    pub repaid_description: Vec<SocialBenefitDescription>,
    /// Box 5 net benefits
    /// Enter an amount in parentheses as negative.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub benefits: i64,
    /// Box 6 voluntary federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 8 claim number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub claim_number: String,
}

/// FORM SSA-1099 — SOCIAL SECURITY BENEFIT STATEMENT
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the SSA-1099 exactly as printed, not an RRB-1099. Copy the statement issuer and the recipient's box 1 name, box 2 Social Security number and box 7 address into their printed party blocks; preserve masking, punctuation and leading zeros. Leave absent identity text empty. Copy the printed tax year and check it against the supplied year; report a mismatch. Read boxes 3 and 4 separately, returning null when clearly blank. Box 5 must show an amount; report a blank box 5 as an error. Read box 5 as printed, using a negative number for parentheses; do not recompute it, deduct box 4 again, or subtract Medicare premiums or workers' compensation offsets. Read box 6, returning null when clearly blank. Copy box 8 exactly, or empty text when absent. Read each printed description line under boxes 3 and 4 once, preserving its amount. Use a Medicare coverage category only when the line names that coverage, and PriorYear only when the printed line expressly identifies the year for which benefits were paid; preserve other wording as Other. Never infer earlier-year benefits or premiums. Report illegible or conflicting values as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.issuer.federal_id,@.recipient.federal_id,@.tax_year,@.claim_number,@.benefits_paid,@.benefits_repaid_to_ssa,@.net_benefits,@.voluntary_federal_income_tax_withheld"
)]
pub struct Ssa1099Paper {
    /// Statement issuer as printed
    /// Use only a printed issuer block; the form title does not supply an issuer name.
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// Year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub tax_year: u32,
    /// Box 1. Name; Box 2. Beneficiary's Social Security Number; Box 7. Address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 3. Benefits Paid
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The amount printed in box 3 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub benefits_paid: Option<i64>,
    /// Box 4. Benefits Repaid to SSA
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The amount printed in box 4 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub benefits_repaid_to_ssa: Option<i64>,
    /// Box 5. Net Benefits (Box 3 minus Box 4)
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub net_benefits: i64,
    /// Box 6. Voluntary Federal Income Tax Withheld
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The amount printed in box 6 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub voluntary_federal_income_tax_withheld: Option<i64>,
    /// Box 8. Claim Number (Use this number if you need to contact SSA.)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub claim_number: String,
    /// DESCRIPTION OF AMOUNT IN BOX 3
    #[libertas_llm_doc_field]
    pub description_of_amount_in_box_3: Vec<SocialBenefitDescription>,
    /// DESCRIPTION OF AMOUNT IN BOX 4
    #[libertas_llm_doc_field]
    pub description_of_amount_in_box_4: Vec<SocialBenefitDescription>,
}

impl From<&Ssa1099Paper> for Ssa1099 {
    fn from(paper: &Ssa1099Paper) -> Self {
        let Ssa1099Paper {
            issuer,
            // The statement year identifies the document and does not change its amounts.
            tax_year: _,
            recipient,
            benefits_paid,
            benefits_repaid_to_ssa,
            net_benefits,
            voluntary_federal_income_tax_withheld,
            claim_number,
            description_of_amount_in_box_3,
            description_of_amount_in_box_4,
        } = paper;
        Self {
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            benefits_paid: benefits_paid.unwrap_or(0),
            paid_description: description_of_amount_in_box_3.clone(),
            benefits_repaid: benefits_repaid_to_ssa.unwrap_or(0),
            repaid_description: description_of_amount_in_box_4.clone(),
            benefits: *net_benefits,
            withholding: voluntary_federal_income_tax_withheld.unwrap_or(0),
            claim_number: claim_number.clone(),
        }
    }
}

impl From<&Ssa1099Paper> for FederalSocial {
    fn from(paper: &Ssa1099Paper) -> Self {
        Self::from(&Ssa1099::from(paper))
    }
}

/// RRB-1099-R
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into issuer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the RRB-1099-R (Annuities or Pensions by the Railroad Retirement Board) exactly. Check its tax year against the supplied year; report a mismatch. Box 7 must show an amount; report a blank box 7 as an error. Return box 3 employee contributions when printed, or null when blank. Read box 7 total gross paid as printed. Return box 4 contributory amount paid, box 5 vested dual benefit, box 6 supplemental annuity, box 8 repayments and box 9 federal income tax withheld, returning null for a clearly blank box. Box 7 is the sum of boxes 4, 5 and 6; report a mismatch as an error. Report an entry in box 10 (rate of tax) or box 11 (country) as an error: that statement is for a nonresident alien, which this App does not calculate. Read box 12 Medicare premium total, returning null when blank. Copy box 1 claim number and payee code exactly, or empty text when absent; never use the text 0 for a blank. Copy box 2 (recipient's identification number) with the recipient's other printed party details. Report illegible or conflicting values as errors. Do not infer whose annuity it is, the recipient's age, the Simplified Method facts or the years of the box 8 repayments. The file may hold several copies of this form, such as Copy B, Copy C and Copy 2; copies of the same form are one form. This import is for RRB-1099-R, not Form 1099-R, RRB-1099 or RRB-1042S.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.issuer.federal_id,@.recipient.federal_id,@.claim_number_and_payee_code,@.box_3,@.box_4,@.box_5,@.box_6,@.gross,@.box_8,@.withholding,@.medicare_premiums"
)]
pub struct FormRrb1099RPaper {
    /// Statement issuer as printed
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// Recipient name, identifying number and mailing address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 claim number and payee code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub claim_number_and_payee_code: String,
    /// Box 3 employee contributions
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// Box 3 employee contributions are
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_3: Option<i64>,
    /// Box 4 contributory amount paid
    /// Tier 2 and the non-Social Security part of tier 1. Box 3 employee contributions are recovered from this amount only.
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The amount printed in box 4 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_4: Option<i64>,
    /// Box 5 vested dual benefit
    /// Fully taxable.
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The amount printed in box 5 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_5: Option<i64>,
    /// Box 6 supplemental annuity
    /// Fully taxable.
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The amount printed in box 6 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_6: Option<i64>,
    /// Box 7 total gross paid
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 8 repayments
    /// Benefits taxed in an earlier year that you repaid in 2026; not subtracted from boxes 4–6.
    /// [NullText]
    /// Box 8 is blank
    /// [SetText]
    /// The amount printed in box 8 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_8: Option<i64>,
    /// Box 9 federal income tax withheld
    /// [NullText]
    /// Box 9 is blank
    /// [SetText]
    /// The amount printed in box 9 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 12 Medicare premium total
    /// [NullText]
    /// Box 12 is blank
    /// [SetText]
    /// The amount printed in box 12 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub medicare_premiums: Option<i64>,
}

/// RRB-1099-R as the calculations read it
/// Derived from the stored paper, `FormRrb1099RPaper`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Rrb1099R {
    /// Statement issuer as printed
    pub issuer: crate::PrintedParty,
    /// Recipient name, identifying number and mailing address
    pub recipient: crate::PrintedParty,
    /// Box 1 claim number and payee code
    #[libertas_default("")]
    pub claim_number_and_payee_code: String,
    /// Box 3 employee contributions
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// Box 3 employee contributions are
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_3: Option<i64>,
    /// Box 4 contributory amount paid
    /// Tier 2 and the non-Social Security part of tier 1. Box 3 employee contributions are recovered from this amount only.
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The amount printed in box 4 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_4: Option<i64>,
    /// Box 5 vested dual benefit
    /// Fully taxable.
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The amount printed in box 5 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_5: Option<i64>,
    /// Box 6 supplemental annuity
    /// Fully taxable.
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The amount printed in box 6 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_6: Option<i64>,
    /// Box 7 total gross paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 8 repayments
    /// Benefits taxed in an earlier year that you repaid in 2026; not subtracted from boxes 4–6.
    /// [NullText]
    /// Box 8 is blank
    /// [SetText]
    /// The amount printed in box 8 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_8: Option<i64>,
    /// Box 9 federal income tax withheld
    /// [NullText]
    /// Box 9 is blank
    /// [SetText]
    /// The amount printed in box 9 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 12 Medicare premium total
    /// [NullText]
    /// Box 12 is blank
    /// [SetText]
    /// The amount printed in box 12 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub medicare_premiums: Option<i64>,
}

// One-way: every printed box reaches the internal record, so a new box cannot be forgotten.
impl From<&FormRrb1099RPaper> for Rrb1099R {
    fn from(paper: &FormRrb1099RPaper) -> Self {
        let FormRrb1099RPaper {
            issuer,
            recipient,
            claim_number_and_payee_code,
            box_3,
            box_4,
            box_5,
            box_6,
            gross,
            box_8,
            withholding,
            medicare_premiums,
        } = paper;
        Self {
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            claim_number_and_payee_code: claim_number_and_payee_code.clone(),
            box_3: *box_3,
            box_4: *box_4,
            box_5: *box_5,
            box_6: *box_6,
            gross: *gross,
            box_8: *box_8,
            withholding: *withholding,
            medicare_premiums: *medicare_premiums,
        }
    }
}
// Preserve the printed absence and text when reopening the stored source.
impl From<&Rrb1099R> for FormRrb1099RPaper {
    fn from(form: &Rrb1099R) -> Self {
        let Rrb1099R {
            issuer,
            recipient,
            claim_number_and_payee_code,
            box_3,
            box_4,
            box_5,
            box_6,
            gross,
            box_8,
            withholding,
            medicare_premiums,
        } = form;
        Self {
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            claim_number_and_payee_code: claim_number_and_payee_code.clone(),
            box_3: *box_3,
            box_4: *box_4,
            box_5: *box_5,
            box_6: *box_6,
            gross: *gross,
            box_8: *box_8,
            withholding: *withholding,
            medicare_premiums: *medicare_premiums,
        }
    }
}

/// Accepted RRB-1099-R
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RailroadSource {
    /// RRB-1099-R as printed
    pub form: Rrb1099R,
    /// Whose annuity this is
    #[libertas_default($null)]
    pub recipient_kind: Option<RailroadRecipient>,
    /// Box 8 repayments by the year the benefits were paid
    /// Leave empty when box 8 is blank. Your RRB records show each year.
    /// ----
    /// Repayment
    pub repayments: Vec<crate::RailroadRepayment>,
}
impl core::ops::Deref for RailroadSource {
    type Target = Rrb1099R;
    fn deref(&self) -> &Rrb1099R {
        &self.form
    }
}
impl RailroadSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Rrb1099R) -> Self {
        Self { form, recipient_kind: None, repayments: Vec::new() }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.recipient_kind = accepted.recipient_kind.clone();
        self.repayments = accepted.repayments.clone();
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&RailroadSource> for FederalRetirement {
    fn from(source: &RailroadSource) -> Self {
        use crate::{RailroadRecipient as W, RailroadRecovery as R};
        let RailroadSource { form, recipient_kind, repayments } = source;
        let Rrb1099R {
            // The parties identify the statement; no figure reads them.
            issuer: _,
            recipient: _,
            claim_number_and_payee_code: _,
            box_3,
            box_4,
            box_5,
            box_6,
            gross,
            box_8,
            withholding,
            medicare_premiums,
        } = form;
        // An empty cost takes box 3 for an annuity that started in 2026 (rule
        // RRB_RECOVERY_COST_NEEDED), as an empty 1099-R cost takes box 9b.
        let basis = |recovery: &R| match recovery {
            R::None => None,
            R::SimplifiedMethod { data } => {
                let mut data = data.clone();
                if data.started / 10000 == 2026 {
                    data.cost = data.cost.or(*box_3);
                }
                Some(data)
            }
        };
        let (holder, basis, ordinary_distribution) = recipient_kind
            .as_ref()
            .map(|recipient| match recipient {
                W::Employee { recovery } => (
                    crate::RailroadHolder::Employee,
                    basis(recovery),
                    crate::RetirementTreatment::Ordinary,
                ),
                W::Spouse => {
                    (crate::RailroadHolder::Spouse, None, crate::RetirementTreatment::Ordinary)
                }
                W::Survivor { died, distributed, recovery } => (
                    crate::RailroadHolder::Survivor { died: *died },
                    basis(recovery),
                    crate::RetirementTreatment::Exception {
                        distributed: *distributed,
                        facts: crate::RetirementException::Death { died: *died },
                        second: None,
                        early_simple: false,
                    },
                ),
            })
            .map(|(holder, basis, treatment)| (Some(holder), basis, Some(treatment)))
            .unwrap_or((None, None, None));
        FederalRetirement {
            form_boxes: crate::RetirementPrintedBoxes {
                medicare_premiums: medicare_premiums.unwrap_or(0),
                ..Default::default()
            },
            trump: false,
            box_2a: 0,
            box_3: 0,
            box_6: 0,
            box_8a: 0,
            box_8b_percent: None,
            box_9a_percent: None,
            ira: false,
            disability: crate::DisabilityIncome::None,
            kind: crate::RetirementKind::Pension,
            disability_wages: crate::DisabilityWages::None,
            gross: *gross,
            taxable: Some(crate::RetirementTaxable::Railroad {
                repaid: box_8.unwrap_or(0),
                repayments: repayments.clone(),
                contributory: box_4.unwrap_or(0),
                box_5: box_5.unwrap_or(0),
                box_6: box_6.unwrap_or(0),
                holder,
                basis,
            }),
            withholding: withholding.unwrap_or(0),
            box7: crate::DistributionCodes::CODE_7.0,
            ordinary_distribution,
            disaster: crate::DisasterDistribution::No,
            after_death: None,
            box_10: 0,
            box_10_basis: None,
            box_9b: *box_3,
        }
    }
}

/// Whose annuity this is
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RailroadRecipient {
    /// Your own employee annuity
    /// You retired at 60 or older. A disability annuity before then is not calculated yet.
    Employee {
        /// Recover the box 3 employee contributions
        recovery: RailroadRecovery,
    },
    /// A spouse or divorced-spouse annuity
    /// Fully taxable: it does not recover the employee's contributions.
    Spouse,
    /// A survivor annuity after the employee's death
    Survivor {
        /// Date of death
        #[libertas_date_only]
        died: u32,
        /// Date the annuity payment was received
        /// Use your payment records; this date is not printed on Form RRB-1099-R.
        #[libertas_date_only]
        distributed: u32,
        /// Recover the box 3 employee contributions
        recovery: RailroadRecovery,
    },
}

/// Recover the box 3 employee contributions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RailroadRecovery {
    /// No
    /// Box 3 is blank, or the contributions were already recovered: box 4 is fully taxable.
    None,
    /// Simplified Method
    /// Enter box 3 as the cost (worksheet line 2). If an earlier annuity ended, ask the RRB to confirm the amount.
    SimplifiedMethod {
        /// Pension Simplified Method facts
        data: crate::PensionBasis,
    },
}

pub(crate) fn net_benefits_issue(
    boxes: &crate::SocialPrintedBoxes,
    net: i64,
) -> Option<crate::validation::ValidationIssue> {
    net_benefits_values_issue(boxes.benefits_paid, boxes.benefits_repaid, net)
}

pub(crate) fn net_benefits_values_issue(
    paid: i64,
    repaid: i64,
    net: i64,
) -> Option<crate::validation::ValidationIssue> {
    (i128::from(paid) - i128::from(repaid) != i128::from(net)).then(|| {
        crate::validation::ValidationIssue::new("SSA_PRINTED_BOXES_DIFFER").at_field("net_benefits")
    })
}
