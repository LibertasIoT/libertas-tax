#[path = "form_1099_k_paper.rs"]
mod form_1099_k_paper;
pub use form_1099_k_paper::{Form1099KPaper, Form1099KStateRow, K1099Source};

#[path = "form_1099_nec_paper.rs"]
mod form_1099_nec_paper;
pub use form_1099_nec_paper::{Form1099NecPaper, Nec1099Source};

use crate::validation::ValidationIssue;
// Coverage: Forms 1099-NEC, 1099-MISC and 1099-K, each entered once as its own document (Forms 1099-NEC, 1099-MISC and 1099-K Rev. December 2026 and their Instructions
//   for Recipient/Payee, i1099mec, i1099k, i1040sc (2026 draft) line 1, Schedule 1 (2026 draft)
//   top-of-form 1099-K entry, i1040gi (2025) "Form(s) 1099-K", Pub. 525).
//   Lines: business receipts follow activity-owned component claims; retained nonbusiness
//   classifications and source choices keep their native tax treatment. A business receives
//   (Schedule C line 1: 1099-NEC box 1a; 1099-MISC boxes 1, 2 and 3 on a business choice, and
//   boxes 5, 6, 10 less the part paid on to clients, and 11; a 1099-K part less its refunds), a
//   rental property (Schedule E line 3: 1099-MISC box 1, a 1099-K part), a royalty entry
//   (Schedule E line 4, box 2), equipment rented out (line 8l, box 1), a hobby (line 8j: a
//   1099-K part, and a 1099-NEC/MISC amount marked not for profit) or an Other income kind
//   (1099-NEC box 1a, 1099-MISC box 3). 1099-MISC box 8 substitute payments are line 8z and net
//   investment income. Box 4 of each form is Form 1040 line 25b. A 1099-K part of money that
//   was not income (gifts, shared costs, personal items sold at a loss) is the top-of-Schedule 1
//   entry; a personal item sold at a gain is a capital gain entered on Investment sales (a
//   review note when no sale is entered). The 1099-K box 1a parts must add to box 1a.
//   `route` runs on the supported working copy and adds the routed amounts to the destination
//   records; a form whose destination is left out follows it with
//   WARN_FORM_1099_DESTINATION_LEFT_OUT. NEC box 3 is included in box 1a and adds only
//   section 4999 tax (20%) to 2026 Schedule 2 line 13k. W-2 code K supplies tax itself;
//   see forms/w2.rs. Refused by name: MISC box 15 section 409A tax and premium interest.
//   Printed NEC 1b–1d, MISC 13a/13b/14 and K 1c/1d supply the tips and overtime
//   subsets to Schedule 1-A through the existing activity owner,
//   without adding receipts again. Not read: boxes 7 and 12 and the FATCA checkbox;
//   Box 9 crop insurance is retained as a printed source fact; its farm's payment records
//   own the income and any deferral. Farm 1099 amounts stay on the Farming lines.
//   Evidence: this module, other_income.rs, federal_rules.rs, unknowns.rs, state_return.rs;
//   tests/income/information_returns.rs and tests/federal/printed_form_boxes.rs.
//   Source validation: NEC box3 is included in box1a (i1099mec,
//   Rev.December2026, p.11); both the excess subset and box4 withholding
//   are bounded by total compensation, including on restored records.
use crate::{
    BusinessActivity, FederalDraft, FederalIncome, FederalIncomeEntry, OtherIncomeItem,
    OtherIncomeKind, Owner,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-NEC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-NEC exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1a nonemployee compensation; report an error when box 1a is blank. Read box 3 excess golden parachute payments and box 4 federal income tax withheld, returning zero for each when it is clearly blank. Box 3 is already included in box 1a; do not add it to compensation or compute its tax. Read box 1b cash tips and box 1d overtime compensation, returning zero when blank, and box 1c Treasury Tipped Occupation Code(s) as one code record per printed code, including 000, or an empty list when blank. Do not read the box 2 checkbox. Do not say where the income belongs or which business earned it; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer name, federal ID and address and state boxes 5 through 7. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099Nec {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1a nonemployee compensation
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub compensation: i64,
    /// Box 1b cash tips
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 1c Treasury Tipped Occupation Code (TTOC)
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 1d overtime compensation
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub overtime: i64,
    /// Box 3 excess golden parachute payments
    /// Already included in box 1a. The App calculates the 20% excise tax on this excess amount.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub golden_parachute: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Where box 1a belongs
    /// Self-employment income is taxed with self-employment tax and can be reduced by the costs of the business. Income from a hobby or a one-time job has no self-employment tax and no deduction of costs.
    /// [NullText]
    /// Relate this amount from its activity
    #[libertas_hidden]
    pub belongs: Option<IncomeUse>,
    /// Payer and state boxes 5–7
    #[libertas_llm_doc_field]
    pub state_details: crate::StateIncomeFields,
}

/// Where this income belongs
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum IncomeUse {
    /// An activity you don't run to make a profit
    /// A hobby: the income goes on Schedule 1 line 8j, with no self-employment tax and no deduction for its costs.
    NotForProfit,
    /// Income that is not from a business or a hobby
    /// It goes on its own Schedule 1 line, with no self-employment tax.
    Income {
        /// What it is
        kind: OtherIncomeKind,
    },
    /// A qualified disaster relief payment
    /// Excluded from income under section 139 when the whole payment qualifies; the form stays as printed.
    DisasterRelief {
        /// What the relief payment was
        relief: crate::DisasterReliefFacts,
    },
    /// A qualified wildfire relief payment
    /// Excluded from income under section 139M when the whole payment qualifies; the form stays as printed.
    WildfireRelief {
        /// What the relief payment was
        relief: crate::WildfireReliefFacts,
    },
    /// Pay for work you did as an employee
    /// Wages on which your employer collected no Social Security and Medicare tax, figured on Form 8919 with the form's withholding.
    EmployeePay {
        /// Actual payroll system and coverage for this pay
        payroll: crate::EmployeePayrollSource,
    },
    /// An amount I relate from its activity
    /// A business, rental, farm, royalty or other activity receives this amount; add it on that activity's Related forms.
    FromActivity,
}

impl IncomeUse {
    fn index(&self) -> i32 {
        use libertas_macros::variant_index;
        (match self {
            Self::NotForProfit => variant_index!(IncomeUse::NotForProfit),
            Self::Income { .. } => variant_index!(IncomeUse::Income),
            Self::DisasterRelief { .. } => variant_index!(IncomeUse::DisasterRelief),
            Self::WildfireRelief { .. } => variant_index!(IncomeUse::WildfireRelief),
            Self::EmployeePay { .. } => variant_index!(IncomeUse::EmployeePay),
            Self::FromActivity => variant_index!(IncomeUse::FromActivity),
        }) as i32
    }
    // Whether the form's question offers this route (the indexes it supplies).
    pub(crate) fn offered(&self, routes: &[i32]) -> bool {
        routes.contains(&self.index())
    }
    // The routes a Form 1099-NEC offers: the activity routes and Form 8919 pay.
    pub(crate) fn nonemployee_routes() -> alloc::vec::Vec<i32> {
        use libertas_macros::variant_index;
        alloc::vec![
            variant_index!(IncomeUse::NotForProfit) as i32,
            variant_index!(IncomeUse::Income) as i32,
            variant_index!(IncomeUse::EmployeePay) as i32,
        ]
    }
    // The routes a Form 1099-MISC box 3 offers: the activity routes and the two relief exclusions.
    pub(crate) fn miscellaneous_routes() -> alloc::vec::Vec<i32> {
        use libertas_macros::variant_index;
        alloc::vec![
            variant_index!(IncomeUse::NotForProfit) as i32,
            variant_index!(IncomeUse::Income) as i32,
            variant_index!(IncomeUse::DisasterRelief) as i32,
            variant_index!(IncomeUse::WildfireRelief) as i32,
            variant_index!(IncomeUse::FromActivity) as i32,
        ]
    }
    // A box 3 amount no local route keeps: an activity's Related forms place it.
    pub(crate) fn related_from_activity(route: &Option<Self>) -> bool {
        matches!(route, None | Some(Self::FromActivity))
    }
    // The part of a printed amount that stays out of income, for the routes
    // the relief calculations own.
    pub(crate) fn excluded(&self, amount: i64) -> i64 {
        match self {
            Self::DisasterRelief { relief } => amount - relief.relief(amount).on_return(),
            Self::WildfireRelief { relief } => amount - relief.relief(amount).on_return(),
            _ => 0,
        }
    }
    // The employee-pay record of a Form 1099-NEC or -MISC amount, which Form 8919 owns.
    pub(crate) fn employee_wages(
        &self,
        issuer: &crate::PrintedParty,
        recipient: &crate::PrintedParty,
        wages: i64,
        withholding: i64,
    ) -> Option<crate::EmployeeWages> {
        let Self::EmployeePay { payroll } = self else {
            return None;
        };
        Some(crate::EmployeeWages {
            source_issuer: issuer.clone(),
            source_recipient: recipient.clone(),
            wages,
            withholding,
            form1099: true,
            payroll: payroll.clone(),
        })
    }
}

/// Accepted identities of the claimable 1099-G boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct GovernmentComponentIds {
    /// Box 2 refund identity
    #[libertas_default(0)]
    pub refund: i64,
    /// Box 6 grant identity
    #[libertas_default(0)]
    pub grant: i64,
    /// Box 7 agriculture payment identity
    #[libertas_default(0)]
    pub agriculture: i64,
}

/// Accepted identities of the claimable 1099-MISC boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct MiscComponentIds {
    /// Crop-insurance box identity
    #[libertas_default(0)]
    pub crop_insurance: i64,
    /// Rents box identity
    #[libertas_default(0)]
    pub rents: i64,
    /// Royalties box identity
    #[libertas_default(0)]
    pub royalties: i64,
    /// Other box identity
    #[libertas_default(0)]
    pub other: i64,
    /// Fishing box identity
    #[libertas_default(0)]
    pub fishing: i64,
    /// Medical box identity
    #[libertas_default(0)]
    pub medical: i64,
    /// Attorney box identity
    #[libertas_default(0)]
    pub attorney: i64,
    /// Fish sales box identity
    #[libertas_default(0)]
    pub fish_sales: i64,
}

/// 1099-MISC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-MISC exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1 rents, box 2 royalties, box 3 other income, box 4 federal income tax withheld, box 5 fishing boat proceeds, box 6 medical and health care payments, box 8 substitute payments in lieu of dividends or interest, box 9 crop insurance proceeds, box 10 gross proceeds paid to an attorney, box 11 fish purchased for resale and box 15 nonqualified deferred compensation, returning zero for each box that is clearly blank; report an error when boxes 1, 2, 3, 5, 6, 8, 9, 10, 11 and 15 are all blank. Read box 13a cash tips and box 14 overtime compensation, returning zero when blank, and box 13b Treasury Tipped Occupation Code(s) as one code record per printed code, including 000, or an empty list when blank. Do not read box 12 or the box 7 and FATCA checkboxes. Do not say where an amount belongs, which business, rental or property received it, or what part of box 10 was paid on to clients; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer name, federal ID and address and state boxes 16 through 18. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099Misc {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Accepted identities of the independently claimable boxes
    #[libertas_hidden]
    #[libertas_read_only]
    pub component_ids: MiscComponentIds,

    /// Box 1 rents
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rents: i64,
    /// Box 2 royalties
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Box 3 other income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// Where box 3 belongs
    /// Prizes, damages, research-study pay and similar amounts are Other income; payments for your services or goods in a business are business income; a hobby's are not-for-profit income.
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// The box 3 other income belongs to
    #[libertas_hidden]
    pub other_use: Option<IncomeUse>,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 fishing boat proceeds
    /// Your share as a crew member; income of your fishing business.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fishing: i64,
    /// Box 6 medical and health care payments
    /// Paid to you as a provider of medical or health care services; income of your practice.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medical: i64,
    /// Box 8 substitute payments in lieu of dividends or interest
    /// Paid by your broker while your shares were on loan. Other income on Schedule 1 line 8z, not qualified dividends or tax-exempt interest.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub substitute: i64,
    /// Box 9 crop insurance proceeds
    /// Relate this payment to its farm or farm rental. The activity counts the printed proceeds and asks only for missing timing or physical-loss facts.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub crop_insurance: i64,
    /// Box 10 gross proceeds paid to an attorney
    /// Paid to you as an attorney; only your fee is your practice's income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub attorney: i64,
    /// Part of box 10 you paid on to clients or others
    /// Settlement money that was not your fee. The rest of box 10 is added to your practice's receipts.
    #[libertas_hidden]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub client_funds: i64,
    /// Box 11 fish purchased for resale
    /// Cash a buyer paid you for fish you caught in your fishing business.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fish_sales: i64,
    /// Box 13a cash tips
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 13b Treasury Tipped Occupation Code (TTOC)
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 14 overtime compensation
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub overtime: i64,
    /// Box 15 nonqualified deferred compensation
    /// The App does not figure the section 409A additional tax and interest on this income; a form with box 15 is named in the estimate as not calculated.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub deferred: i64,
    /// Payer and state boxes 16–18
    #[libertas_llm_doc_field]
    pub state_details: crate::StateIncomeFields,
}

/// 1099-K
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-K exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Read box 1a gross amount of payment card and third party network transactions; report an error when box 1a is blank. Read box 4 federal income tax withheld, returning zero when it is clearly blank. Read box 1c cash tips, returning zero when blank, and box 1d Treasury Tipped Occupation Code(s) as one code record per printed code, including 000, or an empty list when blank. Do not read boxes 1b, 2, 3 or 5a through 5l. Do not split box 1a or say where its payments belong; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the filer name, federal ID and address and state boxes 6 through 8. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099K {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Box 1a gross amount
    /// Before refunds, fees and shipping.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 1c cash tips
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 1d Treasury Tipped Occupation Code (TTOC)
    #[libertas_llm_doc_field]
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Where the payments in box 1a belong
    /// The parts add up to box 1a: one for each business, rental or hobby, and one for payments that were not income to you.
    #[libertas_hidden]
    /// ----
    /// Part
    pub parts: alloc::vec::Vec<KPart>,
    /// Filer and state boxes 6–8
    #[libertas_llm_doc_field]
    pub state_details: crate::StateWithholdingFields,
}

/// Part
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct KPart {
    /// Accepted identity of this part
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,

    /// Amount of box 1a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Non-activity payments
    /// Choose only for payments that are not activity income. Relate activity income from the activity page.
    /// [NullText]
    /// Activity income
    pub belongs: Option<KUse>,
    /// Refunds you gave back out of this part
    /// Money you returned to customers or guests for these payments. Platform fees are not refunds; enter them as that business's or rental's expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refunded: i64,
}

/// Where it belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum KUse {
    /// Not income to you
    /// Payments included in error, such as money from friends or family for shared costs, gifts or reimbursements, and personal items you sold for less than you paid. Shown at the top of Schedule 1; not taxed.
    NotIncome,
    /// Personal items you sold for more than you paid
    /// Each such item is a capital gain: enter it as a sale on Investment sales (no 1099-B). This part adds nothing by itself.
    PersonalGain,
}

// ---- The stored forms ----
//
// The page types above choose their records through `$.` catalogs of the page's
// request, so they are not stored: the stored forms name the same records by
// income identity.

/// Form 1099-NEC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099Nec {
    /// Box 1a nonemployee compensation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub compensation: i64,
    /// Box 1b cash tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 1c Treasury Tipped Occupation Code (TTOC)
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 1d overtime compensation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub overtime: i64,
    /// Box 3 excess golden parachute payments
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub golden_parachute: i64,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Where box 1a belongs
    pub belongs: Option<IncomeUse>,
}

/// Form 1099-MISC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099Misc {
    /// Accepted identities of the independently claimable boxes
    #[libertas_hidden]
    #[libertas_read_only]
    pub component_ids: MiscComponentIds,

    /// Box 1 rents
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rents: i64,
    /// Box 2 royalties
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub royalties: i64,
    /// Box 3 other income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other: i64,
    /// Where box 3 belongs
    #[libertas_default($null)]
    pub other_use: Option<IncomeUse>,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Box 5 fishing boat proceeds
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub fishing: i64,
    /// Box 6 medical and health care payments
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medical: i64,
    /// Box 8 substitute payments in lieu of dividends or interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub substitute: i64,
    /// Box 9 crop insurance proceeds
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub crop_insurance: i64,
    /// Box 10 gross proceeds paid to an attorney
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub attorney: i64,
    /// Part of box 10 paid on to clients
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub client_funds: i64,
    /// Box 11 fish purchased for resale
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub fish_sales: i64,
    /// Box 13a cash tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 13b Treasury Tipped Occupation Code (TTOC)
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 14 overtime compensation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub overtime: i64,
    /// Box 15 nonqualified deferred compensation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred: i64,
}

/// Form 1099-K
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalForm1099K {
    /// Box 1a gross amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 1c cash tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_tips: i64,
    /// Box 1d Treasury Tipped Occupation Code (TTOC)
    pub tip_occupation_codes: alloc::vec::Vec<crate::TippedOccupationCode>,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Where the payments in box 1a belong
    #[libertas_size(min = 1)]
    pub parts: alloc::vec::Vec<FederalKPart>,
}

/// Part of box 1a
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalKPart {
    /// Accepted identity of this part
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,

    /// Amount of box 1a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Where it belongs
    pub belongs: Option<FederalKUse>,
    /// Refunds given back out of this part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub refunded: i64,
}

/// Where a part of box 1a belongs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalKUse {
    /// Not income to the person
    NotIncome,
    /// Personal items sold for more than they cost
    PersonalGain,
}

/// Who received a Form 1099-G box 6 grant
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalGrantUse {
    /// The person, not for a business
    You,
}

impl Form1099Nec {
    fn record(&self) -> FederalForm1099Nec {
        FederalForm1099Nec {
            cash_tips: self.cash_tips,
            tip_occupation_codes: self.tip_occupation_codes.clone(),
            overtime: self.overtime,
            compensation: self.compensation,
            withholding: self.withholding,
            golden_parachute: self.golden_parachute,
            belongs: self.belongs.clone(),
        }
    }
}
impl FederalForm1099Nec {
    pub(crate) fn valid(&self) -> bool {
        [self.compensation, self.withholding, self.golden_parachute].iter().all(|v| money(*v))
            && self.cash_tips <= self.compensation
            && self.overtime <= self.compensation
            && self.golden_parachute <= self.compensation
            && self.withholding <= self.compensation
            && self.belongs.as_ref().is_none_or(|u| use_valid(u, self.compensation))
            && (self.golden_parachute == 0
                || !matches!(self.belongs, Some(IncomeUse::EmployeePay { .. })))
    }
}
impl Form1099Misc {
    fn record(&self) -> FederalForm1099Misc {
        FederalForm1099Misc {
            cash_tips: self.cash_tips,
            tip_occupation_codes: self.tip_occupation_codes.clone(),
            overtime: self.overtime,
            component_ids: self.component_ids.clone(),
            rents: self.rents,
            royalties: self.royalties,
            other: self.other,
            other_use: self.other_use.clone(),
            withholding: self.withholding,
            fishing: self.fishing,
            medical: self.medical,
            substitute: self.substitute,
            crop_insurance: self.crop_insurance,
            attorney: self.attorney,
            client_funds: self.client_funds,
            fish_sales: self.fish_sales,
            deferred: self.deferred,
        }
    }
}
impl FederalForm1099Misc {
    // The income the form adds to the return: every routed or local box at its
    // taxable amount (a prize under its disposition, box 10 less the part paid
    // on to clients, box 8 as other income). Box 15 is refused, not income. Cents.
    pub(crate) fn income(&self) -> i64 {
        let other = match &self.other_use {
            Some(IncomeUse::Income { kind }) => {
                OtherIncomeItem { kind: kind.clone(), amount: self.other, withholding: 0 }.preview()
            }
            Some(route) => self.other - route.excluded(self.other),
            None => self.other,
        };
        self.printed_income() - self.other + other - self.client_funds
    }
    fn printed_income(&self) -> i64 {
        self.rents
            + self.royalties
            + self.other
            + self.fishing
            + self.medical
            + self.substitute
            + self.crop_insurance
            + self.attorney
            + self.fish_sales
    }
    pub(crate) fn valid(&self) -> bool {
        [
            self.rents,
            self.royalties,
            self.other,
            self.withholding,
            self.fishing,
            self.medical,
            self.substitute,
            self.crop_insurance,
            self.attorney,
            self.client_funds,
            self.fish_sales,
            self.deferred,
        ]
        .iter()
        .all(|v| money(*v))
            && self.cash_tips <= self.printed_income()
            && self.overtime <= self.printed_income()
            && self.other_use.as_ref().is_none_or(|u| use_valid(u, self.other))
    }
}
impl Form1099K {
    fn record(&self) -> FederalForm1099K {
        FederalForm1099K {
            cash_tips: self.cash_tips,
            tip_occupation_codes: self.tip_occupation_codes.clone(),
            gross: self.gross,
            withholding: self.withholding,
            parts: self
                .parts
                .iter()
                .map(|part| FederalKPart {
                    id: part.id,
                    amount: part.amount,
                    belongs: part.belongs.as_ref().map(|belongs| match belongs {
                        KUse::NotIncome => FederalKUse::NotIncome,
                        KUse::PersonalGain => FederalKUse::PersonalGain,
                    }),
                    refunded: part.refunded,
                })
                .collect(),
        }
    }
}
impl FederalForm1099K {
    pub(crate) fn page(&self) -> Form1099K {
        Form1099K {
            payer: Default::default(),
            recipient: Default::default(),
            cash_tips: self.cash_tips,
            tip_occupation_codes: self.tip_occupation_codes.clone(),
            gross: self.gross,
            withholding: self.withholding,
            parts: self
                .parts
                .iter()
                .map(|part| KPart {
                    id: part.id,
                    amount: part.amount,
                    belongs: part.belongs.as_ref().map(|belongs| match belongs {
                        FederalKUse::NotIncome => KUse::NotIncome,
                        FederalKUse::PersonalGain => KUse::PersonalGain,
                    }),
                    refunded: part.refunded,
                })
                .collect(),
            state_details: Default::default(),
        }
    }
    // The income parts of box 1a less their refunds. Cents.
    pub(crate) fn income(&self) -> i64 {
        self.parts
            .iter()
            .filter(|part| {
                !matches!(part.belongs, Some(FederalKUse::NotIncome | FederalKUse::PersonalGain))
            })
            .map(|part| part.amount - part.refunded)
            .sum()
    }
    pub(crate) fn valid(&self) -> bool {
        self.cash_tips <= self.gross && money(self.gross)
            && money(self.withholding)
            && self.parts.iter().all(|part| {
                money(part.amount)
                    && money(part.refunded)
            })
            // Retained allocations keep the original monetary carrier bound
            // while a corrected printed gross awaits reconciliation.
            && self.parts.iter().try_fold(0i64, |sum, part| sum.checked_add(part.amount)).is_some_and(money)
            && k_input_issue(&self.page(), false).is_none()
    }
}

impl crate::GrantUse {
    pub(crate) fn record(&self) -> FederalGrantUse {
        match self {
            Self::You => FederalGrantUse::You,
        }
    }
}

fn money(value: i64) -> bool {
    (0..=100_000_000_000).contains(&value)
}

fn use_valid(belongs: &IncomeUse, amount: i64) -> bool {
    match belongs {
        IncomeUse::Income { kind } => {
            OtherIncomeItem { kind: kind.clone(), amount, withholding: 0 }.valid()
        }
        IncomeUse::DisasterRelief { relief } => relief.relief(amount).valid(),
        IncomeUse::WildfireRelief { relief } => relief.relief(amount).valid(),
        IncomeUse::EmployeePay { .. } => belongs
            .employee_wages(&Default::default(), &Default::default(), amount, 0)
            .is_some_and(|wages| wages.valid()),
        IncomeUse::NotForProfit | IncomeUse::FromActivity => true,
    }
}

pub(crate) fn nec_issue(v: &Form1099Nec) -> Option<ValidationIssue> {
    if v.golden_parachute > v.compensation {
        Some(ValidationIssue::new("NEC_EXCESS_ABOVE_TOTAL_PAY").at_field("golden_parachute"))
    } else if v.withholding > v.compensation {
        Some(ValidationIssue::new("NEC_WITHHOLDING_ABOVE_PAY").at_field("withholding"))
    } else if v.golden_parachute != 0 && matches!(v.belongs, Some(IncomeUse::EmployeePay { .. })) {
        Some(ValidationIssue::new("NEC_EMPLOYEE_PAY_HAS_GOLDEN_PARACHUTE").at_field("belongs"))
    } else {
        v.belongs
            .as_ref()
            .and_then(|belongs| use_issue(belongs, v.compensation))
            .map(|issue| issue.in_field("belongs"))
    }
}
// A form never reports a recovery of an earlier year's deduction; a route's own
// calculation states what its facts must satisfy.
fn use_issue(belongs: &IncomeUse, amount: i64) -> Option<ValidationIssue> {
    match belongs {
        IncomeUse::Income { kind: OtherIncomeKind::Recovery { .. } } => {
            Some(ValidationIssue::new("FORM_1099_USE_NOT_RECOVERY").at_field("kind"))
        }
        IncomeUse::DisasterRelief { relief } => {
            relief.relief(amount).issue().map(|issue| issue.in_field("relief"))
        }
        IncomeUse::WildfireRelief { relief } => {
            relief.relief(amount).issue().map(|issue| issue.in_field("relief"))
        }
        IncomeUse::EmployeePay { .. } => belongs
            .employee_wages(&Default::default(), &Default::default(), amount, 0)
            .and_then(|wages| wages.issue()),
        _ => None,
    }
}
pub(crate) fn misc_issue(v: &Form1099Misc) -> Option<ValidationIssue> {
    misc_input_issue(v, true)
}
pub(crate) fn misc_printed_issue(withholding: i64, income: [i64; 10]) -> Option<ValidationIssue> {
    (i128::from(withholding) > income.into_iter().map(i128::from).sum::<i128>())
        .then(|| ValidationIssue::new("MISC_WITHHOLDING_ABOVE_INCOME").at_field("withholding"))
}
pub(crate) fn misc_input_issue(v: &Form1099Misc, personal: bool) -> Option<ValidationIssue> {
    if let Some(issue) = misc_printed_issue(
        v.withholding,
        [
            v.rents,
            v.royalties,
            v.other,
            v.fishing,
            v.medical,
            v.substitute,
            v.crop_insurance,
            v.attorney,
            v.fish_sales,
            v.deferred,
        ],
    ) {
        Some(issue)
    } else if personal && v.client_funds > v.attorney {
        Some(ValidationIssue::new("MISC_CLIENT_FUNDS_ABOVE_PROCEEDS").at_field("client_funds"))
    } else {
        v.other_use
            .as_ref()
            .and_then(|belongs| use_issue(belongs, v.other))
            .map(|issue| issue.in_field("other_use"))
    }
}
pub(crate) fn k_issue(v: &Form1099K) -> Option<ValidationIssue> {
    k_input_issue(v, true)
}
pub(crate) fn k_input_issue(v: &Form1099K, personal: bool) -> Option<ValidationIssue> {
    if personal
        && (v.parts.is_empty()
            || v.parts.iter().map(|part| i128::from(part.amount)).sum::<i128>()
                != i128::from(v.gross))
    {
        return Some(ValidationIssue::new("K_PARTS_NOT_GROSS").at_field("parts"));
    }
    if v.withholding > v.gross {
        return Some(ValidationIssue::new("K_WITHHOLDING_ABOVE_GROSS").at_field("withholding"));
    }
    if let Some((i, _)) = v.parts.iter().enumerate().find(|(_, part)| part.refunded > part.amount) {
        return Some(
            ValidationIssue::new("K_REFUNDS_ABOVE_PART").at_field("refunded").in_list("parts", i),
        );
    }
    if let Some((i, _)) = v.parts.iter().enumerate().find(|(_, part)| {
        matches!(part.belongs, Some(KUse::NotIncome | KUse::PersonalGain)) && part.refunded != 0
    }) {
        return Some(
            ValidationIssue::new("K_REFUNDS_ONLY_ON_INCOME")
                .at_field("refunded")
                .in_list("parts", i),
        );
    }
    None
}

pub(crate) fn answer_issue(draft: &FederalDraft) -> Option<(i64, ValidationIssue)> {
    draft.income.iter().find_map(|entry| {
        let issue = match &entry.income {
            FederalIncome::Wage { .. } => crate::interview::wage_answer_issue(entry),
            FederalIncome::PersonalDebt { .. } | FederalIncome::PropertyAcquisition { .. } =>
                crate::interview::debt_answer_issue(entry, u32::try_from(draft.tax_year).unwrap_or(0), true),
            FederalIncome::Dividend { data } => crate::investment_statements::div_issue(&crate::FederalDividend::from(data))
                .or_else(|| {
                    matches!(crate::FederalDividend::from(data).section199a, crate::ReitDividends::Reported { amount, eligible: None } if amount > 0)
                        .then(|| ValidationIssue::new("DIV_REIT_ELIGIBILITY_NEEDED").at_field("reit_eligible"))
                }),
            FederalIncome::Interest { data } => crate::investment_statements::int_issue(&crate::FederalInterest::from(data)),
            FederalIncome::Oid { data } => crate::investment_statements::oid_issue(&crate::FederalOid::from(data)),
            FederalIncome::K1099 { data } => {
                k_issue(&Form1099K::from(data)).filter(|issue| issue.has_field_prefix(&["parts"]))
            }
            FederalIncome::Misc1099 { data } => {
                misc_issue(&Form1099Misc::from(data)).filter(|issue| issue.has_field_prefix(&["client_funds"]))
            }
            FederalIncome::LtcBenefits { data } => data.record().character_issue(draft, entry.owner.owned()),
            FederalIncome::Able { data } => crate::AbleDistribution::from(data).answer_issue(),
            FederalIncome::EducationDistribution { data } => {
                crate::FederalEducationDistribution::from(data).answer_issue()
            }
            FederalIncome::MedicalDistribution { data } => {
                crate::FederalMedicalDistribution::from(data).answer_issue()
            }
            FederalIncome::Roth { .. }
            | FederalIncome::TrumpDeath { .. }
            | FederalIncome::TrumpAbleRollover { .. } => {
                crate::interview::special_retirement_answer_issue(&entry.income)
            }
            FederalIncome::Retirement { data: source } => {
let data = &crate::FederalRetirement::from(source);
crate::interview::railroad_answer_issue(source)
                .or_else(|| {
                    data.taxable.is_none().then(|| {
                        ValidationIssue::new("R_TAXABLE_METHOD_NEEDED").at_field("taxable")
                    })
                })
                .or_else(|| {
                    data.ordinary_distribution.is_none().then(|| {
                        ValidationIssue::new("R_TREATMENT_NEEDED").at_field("ordinary_distribution")
                    })
                })
                .or_else(|| crate::interview::retirement_answer_issue(source))
},
            FederalIncome::Partnership { data } if data.regular.basis.opening.is_none() => Some(
                ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS")
                    .at_field("opening")
                    .in_field("basis"),
            ),
            FederalIncome::Partnership { data }
                if data
                    .alternative
                    .as_ref()
                    .is_some_and(|record| record.basis.opening.is_none()) =>
            {
                Some(
                    ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS")
                        .at_field("opening")
                        .in_field("basis")
                        .in_field("alternative"),
                )
            }
            FederalIncome::Scorp { data } if crate::ScorpK1::from(data).opening.is_none() => {
                Some(ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS").at_field("opening"))
            }
            _ => None,
        };
        issue.map(|issue| (entry.id, issue))
    }).or_else(|| crate::interview::estate_answer_issue(draft)
        .filter(|(_, issue)| issue.key != "ESTATE_PARTICIPATION_NEEDED"))
        .or_else(|| crate::interview::retirement_aggregate_answer_issue(draft))
}

// ---- Routing ----

// The page of a record kind, for a correction link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Business,
    Rental,
    Royalty,
    Equipment,
    Hobby,
}
impl Kind {
    pub(crate) fn section(self) -> crate::TaxSection {
        match self {
            Self::Business => crate::TaxSection::SelfEmployment,
            Self::Rental => crate::TaxSection::Rentals,
            Self::Royalty => crate::TaxSection::Rentals,
            Self::Equipment | Self::Hobby => crate::TaxSection::OtherIncome,
        }
    }
}

// Where on a form one routed amount sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Nec,
    Rents,
    Royalties,
    Other,
    Fishing,
    Medical,
    Attorney,
    FishSales,
    Part(i64),
    Grant,
    Refund,
}

// Resolve semantic source slots to accepted identities, never list positions.
pub(crate) fn component_key(entry: &FederalIncomeEntry, slot: Slot) -> Option<i64> {
    let key = match (&entry.income, slot) {
        (FederalIncome::Nec1099 { .. }, Slot::Nec) => entry.id,
        (FederalIncome::Misc1099 { data }, slot) => match slot {
            Slot::Rents => data.component_ids.rents,
            Slot::Royalties => data.component_ids.royalties,
            Slot::Other => data.component_ids.other,
            Slot::Fishing => data.component_ids.fishing,
            Slot::Medical => data.component_ids.medical,
            Slot::Attorney => data.component_ids.attorney,
            Slot::FishSales => data.component_ids.fish_sales,
            _ => return None,
        },
        (FederalIncome::K1099 { data }, Slot::Part(id))
            if data.parts.iter().any(|p| p.id == id) =>
        {
            id
        }
        (FederalIncome::GovernmentPayment { data }, Slot::Grant) => data.component_ids.grant,
        (FederalIncome::GovernmentPayment { data }, Slot::Refund) => data.component_ids.refund,
        _ => return None,
    };
    (key > 0).then_some(key)
}

// One amount (cents) a form sends to a record, and the record it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Flow {
    pub slot: Slot,
    pub kind: Option<Kind>,
    pub choice: Option<i64>,
    pub amount: i64,
}

// The amounts of one income record that belong to another record.
pub(crate) fn flows(income: &FederalIncome) -> alloc::vec::Vec<Flow> {
    let mut out = alloc::vec::Vec::new();
    let mut push = |slot, kind, choice: Option<i64>, amount: i64| {
        if amount != 0 {
            out.push(Flow { slot, kind, choice, amount });
        }
    };
    match income {
        FederalIncome::Nec1099 { data } if data.belongs.is_none() => {
            push(Slot::Nec, None, None, FederalForm1099Nec::from(data).compensation);
        }
        FederalIncome::Misc1099 { data } => {
            let data = &Form1099Misc::from(data);
            push(Slot::Rents, None, None, data.rents);
            push(Slot::Royalties, None, None, data.royalties);
            if IncomeUse::related_from_activity(&data.other_use) {
                push(Slot::Other, None, None, data.other);
            }
            push(Slot::Fishing, None, None, data.fishing);
            push(Slot::Medical, None, None, data.medical);
            push(Slot::Attorney, None, None, data.attorney - data.client_funds);
            push(Slot::FishSales, None, None, data.fish_sales);
        }
        FederalIncome::K1099 { data } => {
            for part in &data.parts {
                let amount = part.amount - part.refunded;
                if part.belongs.is_none() {
                    push(Slot::Part(part.id), None, None, amount);
                }
            }
        }
        FederalIncome::GovernmentPayment { data } => {
            if data.grant_use.is_none() {
                push(
                    Slot::Grant,
                    None,
                    None,
                    crate::FederalUnemployment::from(data).taxable_grants,
                );
            }
            if let crate::TaxRefundHistory::BusinessExpense { business } = &data.history {
                push(
                    Slot::Refund,
                    Some(Kind::Business),
                    *business,
                    data.form.state_or_local_income_tax_refunds,
                )
            }
        }
        _ => {}
    }
    out
}

// The record choice a slot names, for rewriting.
fn choice_mut(income: &mut FederalIncome, slot: Slot) -> Option<&mut Option<i64>> {
    match (income, slot) {
        (FederalIncome::Nec1099 { .. }, Slot::Nec) => None,
        (FederalIncome::Misc1099 { .. }, _) | (FederalIncome::K1099 { .. }, _) => None,
        (FederalIncome::GovernmentPayment { .. }, Slot::Grant) => None,
        (
            FederalIncome::GovernmentPayment {
                data:
                    crate::GovernmentPaymentSource {
                        history: crate::TaxRefundHistory::BusinessExpense { business },
                        ..
                    },
            },
            Slot::Refund,
        ) => Some(business),
        _ => None,
    }
}

// Name the record a form sends an amount to, so a record removed later takes
// the amount with it.
pub(crate) fn name_record(income: &mut FederalIncome, slot: Slot, id: i64) {
    if let Some(choice) = choice_mut(income, slot) {
        *choice = Some(id);
    }
}

// Take the amount out of the form (the record left the estimate).
pub(crate) fn clear(income: &mut FederalIncome, slot: Slot) {
    match (income, slot) {
        (FederalIncome::Nec1099 { data }, Slot::Nec) => data.form.compensation = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Rents) => data.form.rents = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Royalties) => data.form.royalties = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Other) => data.form.other = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Fishing) => data.form.fishing = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Medical) => data.form.medical = Some(0),
        (FederalIncome::Misc1099 { data }, Slot::Attorney) => {
            data.form.attorney = Some(0);
            data.client_funds = 0;
        }
        (FederalIncome::Misc1099 { data }, Slot::FishSales) => data.form.fish_sales = Some(0),
        (FederalIncome::K1099 { data }, Slot::Part(id)) => {
            if let Some(part) = data.parts.iter_mut().find(|part| part.id == id) {
                part.amount = 0;
                part.refunded = 0;
            }
        }
        (FederalIncome::GovernmentPayment { data }, Slot::Grant) => {
            data.form.taxable_grants = Some(0)
        }
        (
            FederalIncome::GovernmentPayment {
                data:
                    crate::GovernmentPaymentSource {
                        form,
                        history: crate::TaxRefundHistory::BusinessExpense { .. },
                        ..
                    },
            },
            Slot::Refund,
        ) => form.state_or_local_income_tax_refunds = 0,
        _ => {}
    }
}

// Whether a form still has an amount that does not go to another record, or
// nothing at all to send: a form emptied of its routed amounts is removed.
pub(crate) fn nothing_left(income: &FederalIncome) -> bool {
    match income {
        FederalIncome::Nec1099 { data } => {
            let data = FederalForm1099Nec::from(data);
            data.golden_parachute == 0 && (data.belongs.is_none() && data.compensation == 0)
        }
        FederalIncome::Misc1099 { data } => {
            let data = &Form1099Misc::from(data);
            data.rents == 0
                && data.royalties == 0
                && data.fishing == 0
                && data.medical == 0
                && data.substitute == 0
                && data.crop_insurance == 0
                && data.attorney == 0
                && data.fish_sales == 0
                && data.deferred == 0
                && data.other == 0
        }
        FederalIncome::K1099 { data } => {
            data.parts.iter().all(|part| part.amount == 0 && part.refunded == 0)
        }
        // Keep the physical source and its independently known boxes, repayments
        // and withholding when one routed component leaves the working copy.
        FederalIncome::GovernmentPayment { .. } => false,
        _ => false,
    }
}

pub(crate) fn owner_eligible(
    d: &FederalDraft,
    owner: Owner,
    activity_owner: Owner,
    business: bool,
) -> bool {
    activity_owner == owner
        || (!business
            && d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint))
}

fn mine(d: &FederalDraft, owner: Owner, entry: &FederalIncomeEntry, business: bool) -> bool {
    owner_eligible(d, owner, entry.owner.owned(), business)
}

// The records of one kind a form of `owner` may name: the owner's own
// businesses (self-employment tax is per person), and any rental, royalty,
// equipment or hobby record of the owner, or either spouse's on a joint return.
pub(crate) fn candidates(d: &FederalDraft, owner: Owner, kind: Kind) -> alloc::vec::Vec<i64> {
    d.income
        .iter()
        .filter(|entry| match (&entry.income, kind) {
            (FederalIncome::Business { data }, Kind::Business) => {
                matches!(data.activity, BusinessActivity::Ordinary { .. })
                    && mine(d, owner, entry, true)
            }
            (FederalIncome::Rental { .. }, Kind::Rental) => mine(d, owner, entry, false),
            (FederalIncome::Royalty { .. }, Kind::Royalty) => mine(d, owner, entry, false),
            (
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                    ..
                },
                Kind::Equipment,
            ) => mine(d, owner, entry, false),
            (FederalIncome::HobbySales { .. }, Kind::Hobby) => mine(d, owner, entry, false),
            _ => false,
        })
        .map(|entry| entry.id)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lookup {
    Found(i64),
    // The owner has no record of the kind.
    Missing,
    // Several, and none chosen; or the one named is not one the form may name.
    NotChosen,
}

pub(crate) fn lookup(d: &FederalDraft, owner: Owner, kind: Kind, choice: Option<i64>) -> Lookup {
    let found = candidates(d, owner, kind);
    match (choice, found.as_slice()) {
        (Some(id), _) if found.contains(&id) => Lookup::Found(id),
        (Some(_), _) => Lookup::NotChosen,
        (None, []) => Lookup::Missing,
        (None, [only]) => Lookup::Found(*only),
        (None, _) => Lookup::NotChosen,
    }
}

// Component ownership is resolved against the complete accepted activity cohort.
pub(crate) fn resolve_flow(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    flow: &Flow,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(Kind, i64)> {
    if let Some(kind) = flow.kind {
        return match lookup(d, entry.owner.owned(), kind, flow.choice) {
            Lookup::Found(id) => Some((kind, id)),
            _ => None,
        };
    }
    let id = *claims.get(&component_key(entry, flow.slot)?)?;
    let target = d.income.iter().find(|target| target.id == id)?;
    let kind = match &target.income {
        FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Ordinary { .. }) =>
        {
            Kind::Business
        }
        FederalIncome::Rental { .. } => Kind::Rental,
        FederalIncome::Royalty { .. } => Kind::Royalty,
        FederalIncome::HobbySales { .. } => Kind::Hobby,
        FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
            ..
        } => Kind::Equipment,
        _ => return None,
    };
    Some((kind, id))
}

// Schedule F and Form 4835 own farm grants. They are not ordinary business
// receipts and do not move the physical Form 1099-G's withholding.
pub(crate) fn farm_grants_with_claims(
    draft: &FederalDraft,
    activity: i64,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    let Some(owner) = crate::farm::sources(draft)
        .find(|source| source.entry.id == activity)
        .and_then(|source| source.entry.owner.tax_owner())
    else {
        return 0;
    };
    draft
        .income
        .iter()
        .filter_map(|entry| {
            let FederalIncome::GovernmentPayment { data } = &entry.income else {
                return None;
            };
            (entry.owner.tax_owner() == Some(owner)
                && data.grant_use.is_none()
                && claims.get(&data.component_ids.grant) == Some(&activity))
            .then_some(data.form.taxable_grants.unwrap_or(0))
        })
        .sum()
}

// Collect a retained local NEC/MISC amount at its actual selected destination.
// A business amount has already been routed to its native business record.
pub(crate) fn collect_local(
    ctx: &mut crate::other_income::Ctx<'_>,
    belongs: Option<&IncomeUse>,
    amount: i64,
) {
    match belongs {
        Some(IncomeUse::NotForProfit) => {
            ctx.totals.not_for_profit += amount;
            ctx.totals.injured_other_amount(ctx.id, Some(amount));
        }
        Some(IncomeUse::Income { kind }) => crate::other_income::collect(ctx, kind, amount),
        // The relief and employee-pay routes are figured by their own calculations.
        Some(
            IncomeUse::DisasterRelief { .. }
            | IncomeUse::WildfireRelief { .. }
            | IncomeUse::EmployeePay { .. }
            | IncomeUse::FromActivity,
        )
        | None => {}
    }
}

// Add every routed amount to the record it names (or the owner's only one).
// Runs on the supported working copy after unsupported records and their
// forms have been left out, so a unresolved or ineligible destination is
// left for the estimate's issue, and nothing is added to it.
pub(crate) fn route(d: &mut FederalDraft, claims: &alloc::collections::BTreeMap<i64, i64>) {
    project_work_values(d, true);
    let mut adds: alloc::vec::Vec<(Kind, i64, i64)> = alloc::vec::Vec::new();
    for entry in &d.income {
        for flow in flows(&entry.income) {
            if let Some((kind, id)) = resolve_flow(d, entry, &flow, claims) {
                adds.push((kind, id, flow.amount));
            }
        }
    }
    for (kind, id, amount) in adds {
        let Some(entry) = d.income.iter_mut().find(|entry| entry.id == id) else {
            continue;
        };
        match (&mut entry.income, kind) {
            (FederalIncome::Business { data }, Kind::Business) => {
                if let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity {
                    *receipts += amount;
                }
            }
            (FederalIncome::Rental { data }, Kind::Rental) => data.rent += amount,
            (FederalIncome::NonbusinessProperty { receipts, .. }, Kind::Equipment) => {
                *receipts += amount
            }
            (FederalIncome::Royalty { data }, Kind::Royalty) => data.receipts += amount,
            (FederalIncome::HobbySales { data }, Kind::Hobby) => data.receipts += amount,
            _ => {}
        }
    }
}

// Cents of one form that `route` added to the records it names.

pub(crate) fn routed_from_with_claims(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    let claims = source_claims;
    flows(&entry.income)
        .into_iter()
        .filter(|flow| resolve_flow(d, entry, flow, claims).is_some())
        .map(|flow| flow.amount)
        .sum()
}

// Cents the forms of the working copy add to one record, for its review row.
#[cfg(test)]
pub(crate) fn routed_to(d: &FederalDraft, id: i64) -> i64 {
    let source_claims = crate::related_forms::effective_claimants(d);
    routed_to_with_claims(d, id, &source_claims)
}

pub(crate) fn routed_to_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    let claims = source_claims;
    d.income
        .iter()
        .flat_map(|entry| {
            let claims = &claims;
            flows(&entry.income)
                .into_iter()
                .filter(move |flow| {
                    resolve_flow(d, entry, flow, claims).is_some_and(|(_, target)| target == id)
                })
                .map(|flow| flow.amount)
        })
        .sum()
}

// Recovered business expenses keep the refund record's explicit business choice.

pub(crate) fn refund_destination_issues_with_claims(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    result: &mut crate::FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    let claims = source_claims;
    for flow in flows(&entry.income).into_iter().filter(|flow| flow.slot == Slot::Refund) {
        if resolve_flow(d, entry, &flow, claims).is_some() {
            continue;
        }
        let Some(kind) = flow.kind else {
            continue;
        };
        let correction = context.clone().corrective(kind.section());
        let found = lookup(d, entry.owner.owned(), kind, flow.choice);
        match found {
            Lookup::Found(_) => {}
            Lookup::Missing => crate::issue_localized!(
                result,
                &correction,
                source,
                "ISSUE_FORM_1099_DESTINATION_MISSING",
                "Enter the business that recovered this previously deducted expense, then select it in the tax-refund record.",
            ),
            Lookup::NotChosen => crate::issue_localized!(
                result,
                &correction,
                source,
                "ISSUE_FORM_1099_DESTINATION_NOT_CHOSEN",
                "Select the business that recovered this previously deducted expense in the tax-refund record.",
            ),
        }
    }
}

// Schedule C line 2 is bounded by all line 1 receipts, including linked source
// forms once. These are invariant accepted rows outside the edited topic;
#[cfg(test)]
pub(crate) fn receipt_rule(d: &FederalDraft) -> Option<&'static str> {
    let source_claims = crate::related_forms::effective_claimants(d);
    receipt_rule_with_claims(d, &source_claims)
}

pub(crate) fn receipt_rule_with_claims(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<&'static str> {
    receipt_issue_with_claims(d, source_claims).map(|(_, issue)| issue.key)
}
pub(crate) fn receipt_issue(d: &FederalDraft) -> Option<(i64, crate::validation::ValidationIssue)> {
    let source_claims = crate::related_forms::effective_claimants(d);
    receipt_issue_with_claims(d, &source_claims)
}

pub(crate) fn receipt_issue_with_claims(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    d.income.iter().find_map(|entry| {
        if let FederalIncome::Business { data } = &entry.income
            && let BusinessActivity::Ordinary { receipts, returns, .. } = data.activity
            && returns > receipts + routed_to_with_claims(d, entry.id, source_claims)
        {
            Some((
                entry.id,
                crate::validation::ValidationIssue::new("BUSINESS_RETURNS_ABOVE_RECEIPTS")
                    .at_field("returns")
                    .in_field("activity"),
            ))
        } else {
            None
        }
    })
}

/// Printed work-compensation subsets associated with one activity.
/// Multiple information returns can describe the same receipts. The activity's
/// retained qualified amount identifies the nonduplicated part; printed boxes
/// establish its reporting ceiling and do not add any income a second time.
pub(crate) fn reported_work(
    draft: &FederalDraft,
    business: i64,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (Option<i64>, Option<i64>) {
    let mut tips = None;
    let mut overtime = None;
    for entry in &draft.income {
        let nec;
        let k;
        let misc = match &entry.income {
            FederalIncome::Misc1099 { data } => Some(Form1099Misc::from(data)),
            _ => None,
        };
        let (cash, codes, extra) = match &entry.income {
            FederalIncome::Nec1099 { data } => {
                nec = FederalForm1099Nec::from(data);
                (nec.cash_tips, nec.tip_occupation_codes.as_slice(), Some(nec.overtime))
            }
            FederalIncome::Misc1099 { .. } => {
                let data = misc.as_ref().unwrap();
                (data.cash_tips, data.tip_occupation_codes.as_slice(), Some(data.overtime))
            }
            FederalIncome::K1099 { data } => {
                k = FederalForm1099K::from(data);
                (k.cash_tips, k.tip_occupation_codes.as_slice(), None)
            }
            _ => continue,
        };
        if !flows(&entry.income).iter().any(|flow| {
            resolve_flow(draft, entry, flow, claims) == Some((Kind::Business, business))
        }) {
            continue;
        }
        // Instructions for Forms 1099-NEC/MISC and 1099-K: code 000
        // alone identifies tips that are not eligible for section 224.
        let qualified_occupation = codes.is_empty() || codes.iter().any(|code| code.code != 0);
        // A blank or zero subset on one form does not replace independent
        // compensation records for the activity. A corrected former subset
        // is pruned by the source lifecycle below.
        if cash > 0 {
            *tips.get_or_insert(0) += if qualified_occupation { cash } else { 0 };
        }
        if let Some(extra) = extra.filter(|amount| *amount > 0) {
            *overtime.get_or_insert(0) += extra;
        }
    }
    (tips, overtime)
}

// One printed source supplies its known compensation directly. Multiple
// sources retain the activity's independent nonduplicated qualified subset.
pub(crate) fn printed_work_amounts(
    draft: &FederalDraft,
    business: i64,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (Option<i64>, Option<i64>) {
    let mut shared = false;
    let mut mixed_tip_codes = false;
    let count = draft
        .income
        .iter()
        .filter(|source| {
            matches!(
                source.income,
                FederalIncome::Nec1099 { .. }
                    | FederalIncome::Misc1099 { .. }
                    | FederalIncome::K1099 { .. }
            ) && {
                let destinations: alloc::collections::BTreeSet<_> = flows(&source.income)
                    .iter()
                    .filter_map(|flow| {
                        resolve_flow(draft, source, flow, claims)
                            .and_then(|(kind, id)| (kind == Kind::Business).then_some(id))
                    })
                    .collect();
                if destinations.contains(&business) {
                    if destinations.len() > 1 {
                        shared = true;
                    }
                    let nec;
                    let k;
                    let misc = match &source.income {
                        FederalIncome::Misc1099 { data } => Some(Form1099Misc::from(data)),
                        _ => None,
                    };
                    let codes = match &source.income {
                        FederalIncome::Nec1099 { data } => {
                            nec = FederalForm1099Nec::from(data);
                            nec.tip_occupation_codes.as_slice()
                        }
                        FederalIncome::Misc1099 { .. } => {
                            misc.as_ref().unwrap().tip_occupation_codes.as_slice()
                        }
                        FederalIncome::K1099 { data } => {
                            k = FederalForm1099K::from(data);
                            k.tip_occupation_codes.as_slice()
                        }
                        _ => &[],
                    };
                    let has_unqualified = codes.iter().any(|code| code.code == 0);
                    let has_qualified = codes.iter().any(|code| code.code != 0);
                    mixed_tip_codes |= has_unqualified && has_qualified;
                }
                destinations.contains(&business)
            }
        })
        .count();
    if count != 1 || shared {
        return (None, None);
    }
    let (tips, overtime) = reported_work(draft, business, claims);
    (if mixed_tip_codes { None } else { tips }, overtime)
}

fn project_work_values(draft: &mut FederalDraft, preserve_excluded: bool) {
    let claims = crate::related_forms::effective_claimants(draft);
    let values: alloc::vec::Vec<_> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .map(|entry| (entry.id, printed_work_amounts(draft, entry.id, &claims)))
        .collect();
    for (id, (tips, overtime)) in values {
        let Some(FederalIncomeEntry { income: FederalIncome::Business { data }, .. }) =
            draft.income.iter_mut().find(|entry| entry.id == id)
        else {
            continue;
        };
        if let (Some(facts), Some(amount)) = (&mut data.tips, tips)
            && (facts.eligible || !preserve_excluded)
        {
            facts.amount = amount;
        }
        if let (Some(facts), Some(amount)) = (&mut data.overtime, overtime)
            && (facts.eligible || !preserve_excluded)
        {
            facts.amount = amount;
        }
    }
}

pub(crate) fn follow_work_sources(before: &FederalDraft, draft: &mut FederalDraft) {
    let old_claims = crate::related_forms::effective_claimants(before);
    let new_claims = crate::related_forms::effective_claimants(draft);
    let removed: alloc::vec::Vec<_> = before
        .income
        .iter()
        .filter_map(|entry| {
            if !matches!(entry.income, FederalIncome::Business { .. }) {
                return None;
            }
            let old = reported_work(before, entry.id, &old_claims);
            let new = reported_work(draft, entry.id, &new_claims);
            Some((entry.id, old.0.is_some() && new.0.is_none(), old.1.is_some() && new.1.is_none()))
        })
        .collect();
    for (id, tips, overtime) in removed {
        if let Some(FederalIncomeEntry { income: FederalIncome::Business { data }, .. }) =
            draft.income.iter_mut().find(|entry| entry.id == id)
        {
            if tips {
                data.tips = None;
            }
            if overtime {
                data.overtime = None;
            }
        }
    }
    project_work_values(draft, false);
}

// One-way: the stored paper derives the form the calculations read.
impl From<&Form1099Nec> for FederalForm1099Nec {
    fn from(paper: &Form1099Nec) -> Self {
        paper.record()
    }
}
impl From<&Form1099Misc> for FederalForm1099Misc {
    fn from(paper: &Form1099Misc) -> Self {
        paper.record()
    }
}
impl From<&Form1099K> for FederalForm1099K {
    fn from(paper: &Form1099K) -> Self {
        paper.record()
    }
}
