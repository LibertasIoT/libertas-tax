use crate::cooperative_source::{CoopAllocation, CoopCreditFacts};
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// 1099-PATR
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-PATR exactly. Check its tax year against the supplied year and report a mismatch, illegible amount or conflicting copies as an error. Return printed amounts as decimal numbers without commas or currency signs; a clearly blank numeric box means zero. Read boxes 1 through 12 as printed amounts, including box 12 Other credits and deductions, the printed box 13 specified-cooperative checkbox, and the payer EIN. Retain separately stated box 12 item types and amounts in credit_facts.other only when the issuer actually identifies them; the aggregate does not establish an item type or entitlement. Read only these printed facts. Do not infer recipient identities, personal purchases, property refunds, qualifications, credit breakdowns, basis, participation, notice history or tax elections; preserve the filer's existing unmarked source records. Copies of the same form are one form. Do not read state boxes",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Patr1099 {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1 patronage dividends
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub patronage: i64,
    /// Box 2 nonpatronage distributions
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonpatronage: i64,
    /// Box 3 per-unit retain allocations
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub per_unit_retains: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 redeemed nonqualified notices and retain allocations
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeemed: i64,
    /// Other boxes
    #[libertas_llm_doc_field]
    pub more: PatrMore,
}

/// Other boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PatrMore {
    /// Box 6 section 199A(g) deduction
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a_g: i64,
    /// Box 7 qualified payments
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 qualified non-SSTB items
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 qualified SSTB items
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 13 specified-cooperative checkbox
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub specified_cooperative: bool,
    /// Actual recipient, personal-purchase, notice and property facts
    /// An empty list uses the only eligible same-owner activity. With several activities, enter the actual source allocations. Save a new activity before selecting its accepted identity.
    #[libertas_hidden]
    pub allocations: Vec<crate::CoopAllocationInput>,
    /// Boxes 10 and 11 and separately identified credit facts
    #[libertas_llm_doc_field]
    pub credit_facts: crate::CoopCreditFactsInput,
    /// Box 12 other credits and deductions
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_credits_and_deductions: i64,
}

// One-way: every printed box and answer either reaches the internal model or is
// named as not read by any calculation, so a new box cannot be forgotten.
impl From<&Patr1099> for crate::FederalCooperative {
    fn from(paper: &Patr1099) -> Self {
        let Patr1099 {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            patronage,
            nonpatronage,
            per_unit_retains,
            withholding,
            redeemed,
            more:
                PatrMore {
                    section_199a_g,
                    qualified_payments,
                    qualified_items,
                    sstb_items,
                    specified_cooperative,
                    allocations,
                    credit_facts,
                    other_credits_and_deductions,
                },
        } = paper;
        Self {
            patronage: *patronage,
            nonpatronage: *nonpatronage,
            per_unit_retains: *per_unit_retains,
            withholding: *withholding,
            redeemed: *redeemed,
            section_199a_g: *section_199a_g,
            qualified_payments: *qualified_payments,
            qualified_items: *qualified_items,
            sstb_items: *sstb_items,
            specified_cooperative: *specified_cooperative,
            allocations: allocations.iter().cloned().map(Into::into).collect(),
            credit_facts: credit_facts.clone().into(),
            other_credits_and_deductions: *other_credits_and_deductions,
        }
    }
}

/// Form 1099-PATR — Taxable Distributions Received From Cooperatives
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-PATR exactly as printed. Check its calendar year against the supplied tax year and report a mismatch, illegible amount or conflicting copies as an error. Include its calendar year, corrected checkbox, payer and recipient names, TINs and addresses, payer telephone number, account number, and boxes 1 through 13. Preserve masking, punctuation and leading zeros in identifiers. Clearly blank amount boxes mean zero. Do not infer personal purchases, activity classifications, allocations, credit qualifications, notice history, participation or elections. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.patronage,@.nonpatronage,@.per_unit_retains,@.withholding,@.redeemed,@.section_199a_g,@.qualified_payments,@.qualified_items,@.sstb_items,@.investment,@.work_opportunity,@.other_credits_and_deductions,@.account_number"
)]
pub struct Patr1099Paper {
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// For calendar year
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// PAYER’S name, street address, city or town, state or province, country, ZIP or foreign postal code, and PAYER’S TIN
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER’S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT’S name, TIN, street address (including apt. no.), city or town, state or province, country, and ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Patronage dividends
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub patronage: i64,
    /// Box 2 Nonpatronage distributions
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonpatronage: i64,
    /// Box 3 Per-unit retain allocations
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub per_unit_retains: i64,
    /// Box 4 Federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 Redeemed nonqualified notices
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeemed: i64,
    /// Box 6 Section 199A(g) deduction
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a_g: i64,
    /// Box 7 Qualified payments (Section 199A(b)(7))
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 Section 199A(a) qual. items
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 Section 199A(a) SSTB items
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 10 Investment credit
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment: i64,
    /// Box 11 Work opportunity credit
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub work_opportunity: i64,
    /// Box 12 Other credits and deductions
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_credits_and_deductions: i64,
    /// Box 13 Specified Coop
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub specified_cooperative: bool,
}

impl From<&Patr1099Paper> for Patr1099 {
    fn from(paper: &Patr1099Paper) -> Self {
        let Patr1099Paper {
            corrected: _,
            calendar_year: _,
            payer,
            payer_telephone: _,
            recipient,
            account_number: _,
            patronage,
            nonpatronage,
            per_unit_retains,
            withholding,
            redeemed,
            section_199a_g,
            qualified_payments,
            qualified_items,
            sstb_items,
            investment,
            work_opportunity,
            other_credits_and_deductions,
            specified_cooperative,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            patronage: *patronage,
            nonpatronage: *nonpatronage,
            per_unit_retains: *per_unit_retains,
            withholding: *withholding,
            redeemed: *redeemed,
            more: PatrMore {
                section_199a_g: *section_199a_g,
                qualified_payments: *qualified_payments,
                qualified_items: *qualified_items,
                sstb_items: *sstb_items,
                specified_cooperative: *specified_cooperative,
                allocations: Vec::new(),
                credit_facts: crate::CoopCreditFactsInput {
                    payer_ein: payer.federal_id.clone(),
                    investment: *investment,
                    work_opportunity: *work_opportunity,
                    investment_shares: Vec::new(),
                    other: Vec::new(),
                    participation: None,
                    activity: None,
                    notice: None,
                },
                other_credits_and_deductions: *other_credits_and_deductions,
            },
        }
    }
}

/// Form 1099-PATR and its cooperative topic answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Patr1099Source {
    /// Form 1099-PATR as printed
    pub form: Patr1099Paper,
    /// Actual cooperative source records
    pub more: Patr1099Answers,
}

/// Cooperative records not printed in the numbered form boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Patr1099Answers {
    /// Actual recipient, personal-purchase, notice and property facts
    pub allocations: Vec<crate::CoopAllocation>,
    /// Credit breakdown and source treatment
    pub credit_facts: Patr1099CreditAnswers,
}

/// Cooperative credit breakdown and personal treatment
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Patr1099CreditAnswers {
    /// Issuer breakdown of box 10
    pub investment_shares: Vec<crate::CoopInvestmentShare>,
    /// Separately identified box 12 credits and deductions
    pub other: Vec<crate::CoopOtherItem>,
    /// Activity credit participation
    pub participation: Option<bool>,
    /// Activity receiving box 10 and 11 credits
    pub activity: Option<i64>,
    /// Credit allocation notice
    pub notice: Option<crate::CoopNoticeStatus>,
}

impl Patr1099Source {
    pub(crate) fn printed(form: Patr1099Paper) -> Self {
        Self { form, more: Patr1099Answers::default() }
    }

    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.more = accepted.more.clone();
    }
}

impl From<&Patr1099Source> for Patr1099 {
    fn from(source: &Patr1099Source) -> Self {
        let Patr1099Source { form, more: Patr1099Answers { allocations, credit_facts } } = source;
        let Patr1099CreditAnswers { investment_shares, other, participation, activity, notice } =
            credit_facts;
        let mut value = Self::from(form);
        value.more.allocations = allocations.iter().cloned().map(Into::into).collect();
        value.more.credit_facts.investment_shares = investment_shares.clone();
        value.more.credit_facts.other = other.iter().cloned().map(Into::into).collect();
        value.more.credit_facts.participation = *participation;
        value.more.credit_facts.activity = *activity;
        value.more.credit_facts.notice = notice.clone();
        value
    }
}

impl From<&Patr1099Source> for crate::FederalCooperative {
    fn from(source: &Patr1099Source) -> Self {
        Self::from(&Patr1099::from(source))
    }
}

pub(crate) fn input_issue(v: &crate::Patr1099) -> Option<ValidationIssue> {
    let a: Vec<CoopAllocation> = v.more.allocations.iter().cloned().map(Into::into).collect();
    for (index, allocation) in a.iter().enumerate() {
        if let Some(issue) = allocation.issue() {
            return Some(issue.in_list("allocations", index).in_field("more"));
        }
    }
    if !a.is_empty() {
        let mut sums = [0; 8];
        for a in &a {
            for (to, n) in sums[..4].iter_mut().zip(a.distributions.values()) {
                *to += n;
            }
            for (to, n) in sums[4..].iter_mut().zip([
                a.qualified_payments,
                a.qualified_items,
                a.sstb_items,
                a.section_199a_g,
            ]) {
                *to += n;
            }
        }
        if sums
            != [
                v.patronage,
                v.nonpatronage,
                v.per_unit_retains,
                v.redeemed,
                v.more.qualified_payments,
                v.more.qualified_items,
                v.more.sstb_items,
                v.more.section_199a_g,
            ]
        {
            return Some(
                ValidationIssue::new("COOP_ALLOCATIONS_DO_NOT_RECONCILE")
                    .at_field("allocations")
                    .in_field("more"),
            );
        }
    }
    if let Some(issue) = CoopCreditFacts::from(v.more.credit_facts.clone()).issue() {
        return Some(issue.in_field("credit_facts").in_field("more"));
    }
    None
}

pub(crate) fn patr_input_issue(value: &crate::Patr1099, personal: bool) -> Option<ValidationIssue> {
    if personal && let Some(issue) = input_issue(value) {
        return Some(issue);
    }
    if !personal && let Some(issue) = CoopCreditFacts::from(value.more.credit_facts.clone()).issue()
    {
        return Some(issue.in_field("credit_facts").in_field("more"));
    }
    patr_distributions_issue(
        value.patronage,
        value.nonpatronage,
        value.per_unit_retains,
        value.redeemed,
        value.withholding,
        value.more.qualified_items,
    )
}

fn patr_distributions_issue(
    patronage: i64,
    nonpatronage: i64,
    per_unit_retains: i64,
    redeemed: i64,
    withholding: i64,
    qualified_items: i64,
) -> Option<ValidationIssue> {
    if i128::from(withholding)
        > i128::from(patronage)
            + i128::from(nonpatronage)
            + i128::from(per_unit_retains)
            + i128::from(redeemed)
    {
        return Some(
            ValidationIssue::new("PATR_WITHHOLDING_ABOVE_DISTRIBUTIONS").at_field("withholding"),
        );
    }
    if i128::from(qualified_items)
        > i128::from(patronage)
            + i128::from(nonpatronage)
            + i128::from(per_unit_retains)
            + i128::from(redeemed)
    {
        return Some(
            ValidationIssue::new("PATR_QUALIFIED_ITEMS_ABOVE_DISTRIBUTIONS")
                .at_field("qualified_items")
                .in_field("more"),
        );
    }
    None
}

pub(crate) fn patr_paper_input_issue(value: &crate::Patr1099Paper) -> Option<ValidationIssue> {
    if let Some(issue) = CoopCreditFacts::printed_issue(value.investment, value.work_opportunity) {
        return Some(issue);
    }
    patr_distributions_issue(
        value.patronage,
        value.nonpatronage,
        value.per_unit_retains,
        value.redeemed,
        value.withholding,
        value.qualified_items,
    )
    .map(|issue| issue.map_field_prefix(&["more", "qualified_items"], &["qualified_items"]))
}
