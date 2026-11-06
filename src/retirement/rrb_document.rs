// Coverage: RRB-1099 Social Security-equivalent benefits, printed boxes 1–11.
// IRS Publication 915 (2025), pages 24–26. Net benefits and withholding use
// the Social Security owner; Medicare and earlier-year payments use its source projections.
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

pub type RrbRecipient = crate::PrintedPartyNameAddress;

/// RRB-1099 — payments by the Railroad Retirement Board
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Rrb1099 {
    /// Payer's name, address and federal identification number
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// Recipient's name and mailing address
    #[libertas_llm_doc_field]
    pub recipient: RrbRecipient,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// DUPLICATE
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub duplicate: bool,
    /// Statement year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// Box 1 claim number and payee code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub claim_number: String,
    /// Box 2 recipient's identification number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub recipient_id: String,
    /// Box 3 gross Social Security equivalent benefit portion of tier 1 paid
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_paid: i64,
    /// Box 4 Social Security equivalent benefits repaid to the RRB
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_repaid: i64,
    /// Box 5 net Social Security equivalent benefit portion of tier 1 paid
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits: i64,
    /// Box 6 workers compensation offset
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub workers_compensation: i64,
    /// Box 7 Social Security equivalent benefits paid for the previous year
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_year: i64,
    /// Box 8 Social Security equivalent benefits paid for two years earlier
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub two_years_prior: i64,
    /// Box 9 Social Security equivalent benefits paid for earlier years
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub earlier_years: i64,
    /// Box 10 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 11 Medicare premiums
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_premiums: i64,
    /// Attached printed breakdown of box 9 by benefit year
    #[libertas_llm_doc_field]
    pub earlier_year_details: Vec<RrbBenefitYear>,
}

/// RRB-1099 — payments by the Railroad Retirement Board
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read RRB-1099 exactly, not SSA-1099, RRB-1099-R or RRB-1042S. Preserve issuer and recipient name, address and identification number, claim/payee number, statement year and corrected/duplicate marks. Preserve leading zeros and masking. Read boxes 3 through 11 as printed, keeping a negative net benefit negative and returning zero for a clearly blank amount. Box 6 workers compensation offset is already included in box 3; do not add it. Boxes 7 and 8 are benefits for one and two years before the statement year. Box 9 is an aggregate for earlier years; do not assign it a fabricated year. Read earlier_year_details only from an actual attached printed year-by-year breakdown of box 9, otherwise return an empty list. Do not infer prior-return taxable income, an election, treaty facts, premium allocation, reimbursement or other personal answers. Copies of the same statement are one document. Report conflicting or illegible values as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.issuer.federal_id,@.year,@.recipient_id,@.claim_number,@.benefits_paid,@.benefits_repaid,@.benefits,@.workers_compensation,@.prior_year,@.two_years_prior,@.earlier_years,@.withholding,@.medicare_premiums"
)]
pub struct Rrb1099Paper {
    /// Payer’s name, street address, city, state, and ZIP code; payer’s federal identifying no.
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// DUPLICATE
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub duplicate: bool,
    /// Year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// 1. Claim Number and Payee Code
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub claim_number: String,
    /// 2. Recipient’s Identification Number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub recipient_id: String,
    /// Recipient’s Name, Street Address, City, State, and Zip Code
    #[libertas_llm_doc_field]
    pub recipient: RrbRecipient,
    /// 3. Gross Social Security Equivalent Benefit Portion of Tier 1 Paid in the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_paid: i64,
    /// 4. Social Security Equivalent Benefit Portion of Tier 1 Repaid to RRB in the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits_repaid: i64,
    /// 5. Net Social Security Equivalent Benefit Portion of Tier 1 Paid in the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits: i64,
    /// 6. Workers’ Compensation Offset in the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub workers_compensation: i64,
    /// 7. Social Security Equivalent Benefit Portion of Tier 1 Paid for One Year Prior to the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_year: i64,
    /// 8. Social Security Equivalent Benefit Portion of Tier 1 Paid for Two Years Prior to the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub two_years_prior: i64,
    /// 9. Social Security Equivalent Benefit Portion of Tier 1 Paid for Three or More Years Prior to the Year Shown Above
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub earlier_years: i64,
    /// 10. Federal Income Tax Withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// 11. Medicare Premium Total
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_premiums: i64,
    /// Attached printed breakdown of box 9 by benefit year
    #[libertas_llm_doc_field]
    pub earlier_year_details: Vec<RrbBenefitYear>,
}

// Each printed field reaches the existing source model without changing its figures.
impl From<&Rrb1099Paper> for Rrb1099 {
    fn from(paper: &Rrb1099Paper) -> Self {
        let Rrb1099Paper {
            issuer,
            corrected,
            duplicate,
            year,
            claim_number,
            recipient_id,
            recipient,
            benefits_paid,
            benefits_repaid,
            benefits,
            workers_compensation,
            prior_year,
            two_years_prior,
            earlier_years,
            withholding,
            medicare_premiums,
            earlier_year_details,
        } = paper;
        Self {
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            corrected: *corrected,
            duplicate: *duplicate,
            year: *year,
            claim_number: claim_number.clone(),
            recipient_id: recipient_id.clone(),
            benefits_paid: *benefits_paid,
            benefits_repaid: *benefits_repaid,
            benefits: *benefits,
            workers_compensation: *workers_compensation,
            prior_year: *prior_year,
            two_years_prior: *two_years_prior,
            earlier_years: *earlier_years,
            withholding: *withholding,
            medicare_premiums: *medicare_premiums,
            earlier_year_details: earlier_year_details.clone(),
        }
    }
}

impl Rrb1099Paper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        printed_issue(
            self.year,
            self.benefits_paid,
            self.benefits_repaid,
            self.benefits,
            self.earlier_years,
            &self.earlier_year_details,
        )
    }
}

/// A benefit year and amount printed on an RRB-1099 attachment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RrbBenefitYear {
    /// Printed benefit year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// Printed benefits for that year
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
}

fn printed_issue(
    year: u32,
    benefits_paid: i64,
    benefits_repaid: i64,
    benefits: i64,
    earlier_years: i64,
    earlier_year_details: &[RrbBenefitYear],
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if year != 2026 {
        return Some(ValidationIssue::new("RRB_STATEMENT_YEAR").at_field("year"));
    }
    if let Some(issue) =
        crate::ssa_1099_model::net_benefits_values_issue(benefits_paid, benefits_repaid, benefits)
    {
        return Some(issue.map_field_prefix(&["net_benefits"], &["benefits"]));
    }
    if !earlier_year_details.is_empty()
        && earlier_year_details.iter().map(|row| i128::from(row.amount)).sum::<i128>()
            != i128::from(earlier_years)
    {
        return Some(
            ValidationIssue::new("RRB_EARLIER_YEARS_DIFFER").at_field("earlier_year_details"),
        );
    }
    for (index, row) in earlier_year_details.iter().enumerate() {
        if row.year >= year.saturating_sub(2) {
            return Some(
                ValidationIssue::new("RRB_EARLIER_YEAR_NEEDED")
                    .at_field("year")
                    .in_list("earlier_year_details", index),
            );
        }
    }
    None
}

impl Rrb1099 {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        printed_issue(
            self.year,
            self.benefits_paid,
            self.benefits_repaid,
            self.benefits,
            self.earlier_years,
            &self.earlier_year_details,
        )
    }

    pub(crate) fn record(&self) -> crate::FederalSocial {
        use crate::{SocialBenefitDescription as Line, SocialBenefitDescriptionKind as Kind};
        let mut descriptions = Vec::new();
        if self.medicare_premiums != 0 {
            descriptions
                .push(Line { description: Kind::MedicareTotal, amount: self.medicare_premiums });
        }
        for (year, amount) in [
            (self.year.saturating_sub(1), self.prior_year),
            (self.year.saturating_sub(2), self.two_years_prior),
        ] {
            if amount != 0 {
                descriptions.push(Line { description: Kind::PriorYear { year }, amount });
            }
        }
        for row in &self.earlier_year_details {
            descriptions
                .push(Line { description: Kind::PriorYear { year: row.year }, amount: row.amount });
        }
        crate::FederalSocial {
            form_boxes: crate::SocialPrintedBoxes {
                benefits_paid: self.benefits_paid,
                paid_description: descriptions,
                benefits_repaid: self.benefits_repaid,
                repaid_description: Vec::new(),
            },
            benefits: self.benefits,
            withholding: self.withholding,
        }
    }
}
