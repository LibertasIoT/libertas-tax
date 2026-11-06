// Accepted education statements, annual source answers and intrinsic expense helpers.
use crate::kernel::phase;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Education credit to evaluate
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EducationMethod {
    /// American opportunity credit
    AmericanOpportunity,
    /// Lifetime learning credit
    LifetimeLearning,
}
/// Annual education answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationStudent {
    /// Student
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    #[libertas_read_only]
    pub student: i64,
    /// Credit to evaluate
    /// [NullText]
    /// Choose the credit for this student
    #[libertas_default($null)]
    pub method: Option<EducationChoice>,
    /// Annual expense and eligibility answers
    pub more: StudentMore,
}
// Form 1098-T (2026), Copy B: https://www.irs.gov/pub/irs-pdf/f1098t.pdf
/// Form 1098-T Tuition Statement as printed
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the printed calendar year, the CORRECTED checkbox (true only when checked), and each printed party block into filer, student. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Copy the filer's telephone number and the service provider/account number exactly as printed; absent text is empty text. Never infer a recipient from household names. Read this one Form 1098-T (Tuition Statement) exactly. Compare the printed calendar year with the supplied year; report a mismatch. Read box 1 payments received for qualified tuition and related expenses as printed; a clearly blank box 1 on a statement reporting a prior-year adjustment is zero; otherwise a blank, illegible or conflicting box 1 is an error, never a guess. Return zero for a clearly blank box 5 scholarships or grants; report an illegible box 5 as an error. Return true for box 8 (at least half-time student) and box 9 (graduate student) when the box is checked and false when it is clearly unchecked. Read box 4 Adjustments made for a prior year into box_4 and box 6 Adjustments to scholarships or grants for a prior year into box_6 as printed amounts. Return null for a clearly blank box; preserve a printed zero as zero. Report an illegible or conflicting amount as an error, never infer an amount. Return true for box 7 (the amount in box 1 includes amounts for an academic period beginning January-March 2027) when checked and false when clearly unchecked. Read box 10 insurance contract reimbursements or refunds as printed; return zero for a clearly blank box and report an illegible box as an error. Do not choose the student and do not infer books, refunds, 529 or Coverdell payments, earlier credits or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.filer.federal_id,@.student.federal_id,@.calendar_year,@.box_1,@.box_4,@.box_5,@.box_6,@.box_10,@.account_number"
)]
pub struct Form1098TPaper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// FILER'S name, street address, city/town, state/province, country, ZIP/foreign code, and employer identification no.
    #[libertas_llm_doc_field]
    pub filer: crate::PrintedParty,
    /// FILER'S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub filer_telephone: String,
    /// STUDENT'S name, street address, city/town, state/province, country, ZIP/foreign code, and TIN
    #[libertas_llm_doc_field]
    pub student: crate::PrintedParty,
    /// Service Provider/Acct. No. (see instr.)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Payments received for qualified tuition and related expenses
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_1: i64,
    /// Box 4 Adjustments made for a prior year
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// Box 4 has this printed amount
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_4: Option<i64>,
    /// Box 5 Scholarships or grants
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_5: i64,
    /// Box 6 Adjustments to scholarships or grants for a prior year
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// Box 6 has this printed amount
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_6: Option<i64>,
    /// Box 7 Checked if the amount in box 1 includes amounts for an academic period beginning January-March 2027
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub box_7: bool,
    /// Box 8 Checked if at least half-time student
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub box_8: bool,
    /// Box 9 Checked if a graduate student
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_9: bool,
    /// Box 10 Ins. contract reimb./refund
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_10: i64,
}
/// Form 1098-T in the Document center
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentCreditDocument {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Student
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub student_id: i64,
    /// Form 1098-T as printed
    #[libertas_llm_doc_field]
    pub form: Form1098TPaper,
}
/// Accepted Form 1098-T and its student
/// The printed form and the person on this return it belongs to, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationSource {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Student, by the identity of a person on the return
    #[libertas_read_only]
    #[libertas_default(0)]
    pub student: i64,
    /// Form 1098-T as printed
    pub form: Form1098TPaper,
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
impl EducationSource {
    /// A newly entered form; the App assigns its student after entry.
    pub(crate) fn printed(document: StudentCreditDocument) -> Self {
        Self {
            id: document.id,
            student: 0,
            form: document.form,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        }
    }
}
/// More about this student's expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentMore {
    /// Required books, supplies and equipment bought from the school
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub books_from_school: i64,
    /// Other required course books, supplies and equipment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_books: i64,
    /// Refunds of these 2026 expenses
    /// Enter other refunds allocated to the expenses used for this return. Exclude Form 1098-T box 10 insurance reimbursements, which have their own allocation below. Box 1 already subtracts the school's same-year refunds; count each refund once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refunds: i64,
    /// Part of the box 10 insurance reimbursements allocated to this return's qualified expenses
    /// Use the insurer's records and the original payments to establish the expense year. Include only the amount reducing the expenses used for this return, after excluding any reimbursement already reflected in box 1. Records establishing none is a known zero; an unavailable allocation leaves this credit unresolved. A remaining refund of earlier expenses needs its actual year and prior-credit history established for an earlier-return adjustment or recapture in More credits.
    #[libertas_default(Unavailable)]
    pub insurance_refund_current_expenses: crate::SourceCents,
    /// Expenses paid with tax-free 529 or Coverdell distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_free_529: i64,
    /// Part of the box 5 grant the student includes in income
    /// A grant that may, by its terms, pay living costs (a Pell grant, most scholarships) can be treated as paying room and board instead of tuition: the student reports that part as income and the credit uses more of the tuition (Pub. 970 chapter 2, "Coordination with Pell grants"). Not for a grant restricted to tuition, and only for a student who is a candidate for a degree at an eligible school.
    #[libertas_default(None)]
    pub scholarship_in_income: ScholarshipInclusion,
    /// Four years of college finished before 2026, the American opportunity credit claimed for four earlier years, or a felony drug conviction
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    pub american_opportunity_used_up: Option<bool>,
}
impl Default for StudentMore {
    fn default() -> Self {
        Self {
            books_from_school: 0,
            other_books: 0,
            refunds: 0,
            insurance_refund_current_expenses: crate::SourceCents::Unavailable,
            tax_free_529: 0,
            scholarship_in_income: ScholarshipInclusion::None,
            american_opportunity_used_up: None,
        }
    }
}
/// Part of the box 5 grant the student includes in income
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ScholarshipInclusion {
    /// None: the whole grant pays tuition and stays tax-free
    #[default]
    None,
    /// Let the App choose
    /// For a student who is you or your spouse, the App figures the credit, the tax and the earned income credit with each part worth trying and keeps the one with the lowest total tax on this return. For a dependent student it keeps the smallest part that gives the largest credit, and tells you what the student reports on their own return. Never more than the living costs below.
    Automatic {
        /// Room, board and other living costs the student paid in 2026
        /// What the student actually paid, not the school's cost-of-attendance estimate. The included part cannot exceed it.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        living_costs: i64,
    },
    /// This amount
    Amount {
        /// Part of the grant the student includes in income
        /// At most the box 5 grant and the living costs the student actually paid in 2026. A part included by you or your spouse goes on Schedule 1 line 8r of this return; a dependent student reports it on their own return.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        included: i64,
    },
}

/// Education expense worksheet
/// Enter payments and reductions for this student's selected credit once. Do not use room, board, insurance or transportation. A part of the grant the student includes in income is chosen on the Education page.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEducationExpenses {
    /// Tuition and required enrollment fees paid for eligible academic periods
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tuition: i64,
    /// Required books, supplies and equipment paid to the institution as a condition of enrollment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub school_materials: i64,
    /// Other required course books, supplies and equipment
    /// Counted for American Opportunity only; not counted for Lifetime Learning.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other_materials: i64,
    /// Tax-free scholarships, grants and employer assistance allocated to the expenses counted for this credit
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub assistance: i64,
    /// Refunds of those expenses for this return
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub refunds: i64,
    /// Those expenses allocated to tax-free 529/ESA distributions or another tax benefit
    /// Do not enter an amount already included in assistance or refunds.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other_benefits: i64,
    /// Part of the assistance the student includes in income
    /// The page's choice: none, the App's choice within the living costs, or an amount. The credit's assistance is reduced by the part included.
    #[libertas_default(None)]
    pub scholarship_in_income: ScholarshipInclusion,
}
impl FederalEducationExpenses {
    // The part of the box 5 grant the student includes in income (Pub. 970
    // chapter 2): the chosen amount, never more than the grant. An unresolved
    // Automatic (the estimate resolves it before the rules run) is none.
    pub(crate) fn included(&self) -> i64 {
        match self.scholarship_in_income {
            ScholarshipInclusion::Amount { included } => phase::cap(included, [self.assistance]),
            _ => 0,
        }
    }
    // The assistance that is excludable from income (IRC 25A(g)(2)(A)): the
    // grant less the part the student includes.
    pub(crate) fn tax_free_assistance(&self) -> i64 {
        self.assistance - self.included()
    }
    pub(crate) fn qualified(&self, method: EducationMethod) -> i64 {
        phase::floor(
            self.tuition
                + self.school_materials
                + if method == EducationMethod::AmericanOpportunity {
                    self.other_materials
                } else {
                    0
                }
                - self.tax_free_assistance()
                - self.refunds
                - self.other_benefits,
            0,
        )
    }
}

/// Education credit facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEducation {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Student, by the identity of a person on the return
    #[libertas_read_only]
    #[libertas_default(0)]
    pub student: i64,
    /// Form 1098-T box 8: at least half-time student
    pub half_time: bool,
    /// Form 1098-T box 9: graduate student
    pub graduate: bool,
    /// Form 1098-T box 1 payments
    pub tuition: i64,
    /// Form 1098-T box 5 scholarships
    pub scholarships: i64,
    /// Form 1098-T box 4 prior-year tuition adjustment
    pub prior_tuition_adjustment: crate::SourceCents,
    /// Form 1098-T box 6 prior-year scholarship adjustment
    pub prior_scholarship_adjustment: crate::SourceCents,
}
/// Education credit to evaluate
/// Credit-specific questions belong only to the selected credit.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EducationChoice {
    /// American opportunity credit
    /// First four years, fewer than four prior claims, at least half-time degree study, and no disqualifying drug conviction.
    AmericanOpportunity,
    /// Lifetime learning credit
    LifetimeLearning,
}
