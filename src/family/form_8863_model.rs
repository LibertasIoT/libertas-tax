pub use crate::education_source::{
    EducationChoice, EducationMethod, EducationSource, EducationStudent, FederalEducation,
    FederalEducationExpenses, Form1098TPaper, ScholarshipInclusion, StudentCreditDocument,
    StudentMore,
};
// Education editor adapters and annual calculation inputs.
#[allow(unused_imports)]
use super::*;

/// Education credits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1098-T tuition statement, keeping its supporting pages together. Do not infer the student selection, eligible expenses, grants outside the statement or prior education credits. Skip blank templates and count duplicate copies of the same document once. Use only printed evidence; do not infer personal-record answers."
)]
pub struct EducationCredits {
    /// Students
    #[libertas_unordered]
    /// ----
    /// Student
    pub students: Vec<StudentCreditDocument>,
    /// People on this return
    #[libertas_hidden]
    #[libertas_read_only]
    pub people: Vec<PersonChoice>,
}
/// Accepted education documents
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationDocumentView {
    /// Accepted tuition statements
    #[libertas_read_only]
    #[libertas_output_only]
    pub documents: Vec<crate::StudentCreditView>,
    /// Annual answers, once for each student with a statement
    #[libertas_read_only]
    pub students: Vec<EducationStudentPage>,
    /// People named by accepted education documents
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub people: Vec<PersonChoice>,
}

/// Annual education answers for an accepted student's statements
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationStudentPage {
    /// Student
    #[libertas_enum_source("^<EducationDocumentView>.people")]
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
    pub more: StudentMorePage,
    /// Available insurance refund allocation answers
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_insurance_refund_allocations: Vec<i32>,
}
impl From<&EducationStudent> for EducationStudentPage {
    fn from(value: &EducationStudent) -> Self {
        let EducationStudent { student, method, more } = value;
        Self {
            student: *student,
            method: *method,
            more: StudentMorePage::from(more),
            // SourceCents schema order: Unavailable, NotReported, Reported.
            allowed_insurance_refund_allocations: alloc::vec![0, 1, 2],
        }
    }
}
impl From<EducationStudent> for EducationStudentPage {
    fn from(value: EducationStudent) -> Self {
        Self::from(&value)
    }
}
impl From<&EducationStudentPage> for EducationStudent {
    fn from(value: &EducationStudentPage) -> Self {
        let EducationStudentPage { student, method, more, allowed_insurance_refund_allocations: _ } =
            value;
        Self { student: *student, method: *method, more: StudentMore::from(more) }
    }
}

/// Student
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into filer, student. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read this one Form 1098-T (Tuition Statement) exactly. Check its calendar year against the supplied year; report a mismatch. Read box 1 payments received for qualified tuition and related expenses as printed; a clearly blank box 1 on a statement reporting a prior-year adjustment is zero; otherwise a blank, illegible or conflicting box 1 is an error, never a guess. Return an empty string for a clearly blank box 5 scholarships or grants; report an illegible box 5 as an error. Return true for box 8 (at least half-time student) and box 9 (graduate student) when the box is checked and false when it is clearly unchecked. Read box 4 prior-year tuition adjustments into prior_tuition_adjustment and box 6 prior-year scholarship adjustments into prior_scholarship_adjustment: NotReported for a clearly blank box, Reported for a legible amount, Unavailable for an illegible amount. Do not read boxes 7 or 10. Do not choose the student and do not infer books, refunds, 529 or Coverdell payments, earlier credits or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct StudentCredit {
    /// FILER'S name, employer identification number and address
    pub filer: crate::PrintedParty,
    /// STUDENT'S name, TIN and address
    pub student: crate::PrintedParty,
    /// 1098-T box 1 payments for tuition
    /// What you paid in 2026 for 2026 and early-2027 terms.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tuition: i64,
    /// 1098-T box 5 scholarships or grants
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub scholarships: i64,
    /// 1098-T box 8 at least half-time
    #[libertas_default(true)]
    pub half_time: bool,
    /// 1098-T box 9 graduate student
    #[libertas_default(false)]
    pub graduate: bool,
    /// Box 4 adjustments to tuition for a prior year
    #[libertas_default(NotReported)]
    pub prior_tuition_adjustment: crate::SourceCents,
    /// Box 6 adjustments to scholarships for a prior year
    #[libertas_default(NotReported)]
    pub prior_scholarship_adjustment: crate::SourceCents,
}
// Single-statement facts; the annual projection separately reads retained
// box 10 amounts together with the student's insurance refund allocation.
impl From<&EducationSource> for FederalEducation {
    fn from(source: &EducationSource) -> Self {
        let EducationSource {
            id,
            student,
            owner_source: _,
            joint_share: _,
            form:
                Form1098TPaper {
                    // Identification and bookkeeping boxes; no figure reads them.
                    calendar_year: _,
                    corrected: _,
                    filer: _,
                    filer_telephone: _,
                    student: _,
                    account_number: _,
                    box_1: tuition,
                    box_4: prior_tuition_adjustment,
                    box_5: scholarships,
                    box_6: prior_scholarship_adjustment,
                    // Pub. 970 chapters 2–3: prepaid January–March tuition
                    // belongs to the payment year already represented by box 1.
                    box_7: _,
                    box_8: half_time,
                    box_9: graduate,
                    // The insurance refund needs its expense-year allocation.
                    box_10: _,
                },
            // The matcher's result; no figure reads it.
            assigned: _,
        } = source;
        FederalEducation {
            id: *id,
            student: *student,
            half_time: *half_time,
            graduate: *graduate,
            tuition: *tuition,
            scholarships: *scholarships,
            prior_tuition_adjustment: match prior_tuition_adjustment {
                Some(amount) => crate::SourceCents::Reported { amount: *amount },
                None => crate::SourceCents::NotReported,
            },
            prior_scholarship_adjustment: match prior_scholarship_adjustment {
                Some(amount) => crate::SourceCents::Reported { amount: *amount },
                None => crate::SourceCents::NotReported,
            },
        }
    }
}
/// Accepted student statement and its answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct StudentCreditView {
    /// Accepted statement identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub id: i64,
    /// Student
    /// [NullText]
    /// Student not assigned
    #[libertas_enum_source("^<EducationDocumentView>.people")]
    #[libertas_default($null)]
    pub student: Option<i64>,
    /// Tuition reported on the statement
    #[libertas_money("USD", 2)]
    pub tuition: i64,
    /// Scholarships reported on the statement
    #[libertas_money("USD", 2)]
    pub scholarships: i64,
    /// Box 7: payments include an academic period beginning January–March of the following year
    /// These payments count in the year paid. For example, tuition paid in 2026 for a term beginning January–March 2027 counts toward the 2026 credit.
    pub following_year_academic_period: bool,
    /// Box 10: insurance contract reimbursements or refunds
    #[libertas_money("USD", 2)]
    pub insurance_reimbursements: i64,
    /// At least half-time
    pub half_time: bool,
    /// Graduate student
    pub graduate: bool,
    /// Prior-year tuition adjustments
    pub prior_tuition_adjustment: crate::SourceCents,
    /// Prior-year scholarship adjustments
    pub prior_scholarship_adjustment: crate::SourceCents,
}
impl From<&EducationSource> for StudentCreditView {
    fn from(value: &EducationSource) -> Self {
        let facts = FederalEducation::from(value);
        Self {
            id: value.id,
            student: value.assigned.is_known().then_some(value.student),
            tuition: value.form.box_1,
            scholarships: value.form.box_5,
            following_year_academic_period: value.form.box_7,
            insurance_reimbursements: value.form.box_10,
            half_time: value.form.box_8,
            graduate: value.form.box_9,
            prior_tuition_adjustment: facts.prior_tuition_adjustment,
            prior_scholarship_adjustment: facts.prior_scholarship_adjustment,
        }
    }
}

/// More about this student's expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StudentMorePage {
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
    #[libertas_constrained_by("^<EducationStudentPage>.allowed_insurance_refund_allocations")]
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
impl Default for StudentMorePage {
    fn default() -> Self {
        Self::from(&StudentMore::default())
    }
}
impl From<&StudentMore> for StudentMorePage {
    fn from(value: &StudentMore) -> Self {
        let StudentMore {
            books_from_school,
            other_books,
            refunds,
            insurance_refund_current_expenses,
            tax_free_529,
            scholarship_in_income,
            american_opportunity_used_up,
        } = value;
        Self {
            books_from_school: *books_from_school,
            other_books: *other_books,
            refunds: *refunds,
            insurance_refund_current_expenses: insurance_refund_current_expenses.clone(),
            tax_free_529: *tax_free_529,
            scholarship_in_income: scholarship_in_income.clone(),
            american_opportunity_used_up: *american_opportunity_used_up,
        }
    }
}
impl From<&StudentMorePage> for StudentMore {
    fn from(value: &StudentMorePage) -> Self {
        let StudentMorePage {
            books_from_school,
            other_books,
            refunds,
            insurance_refund_current_expenses,
            tax_free_529,
            scholarship_in_income,
            american_opportunity_used_up,
        } = value;
        Self {
            books_from_school: *books_from_school,
            other_books: *other_books,
            refunds: *refunds,
            insurance_refund_current_expenses: insurance_refund_current_expenses.clone(),
            tax_free_529: *tax_free_529,
            scholarship_in_income: scholarship_in_income.clone(),
            american_opportunity_used_up: *american_opportunity_used_up,
        }
    }
}

// Annual calculation inputs are projected from retained statements and the
// student's one annual answer row. They have no storage representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EducationAnnualProjection {
    pub student: i64,
    pub method: EducationChoice,
    pub expenses: FederalEducationExpenses,
    pub sources: Vec<i64>,
}

// What one education entry's credit takes into account, in whole dollars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EducationUse {
    // The student's person identity; None for an entry naming no catalog student.
    pub(crate) person: Option<i64>,
    pub(crate) method: EducationMethod,
    // The credit's qualified expenses before the credit's own caps.
    pub(crate) qualified: i64,
    // The part the credit consumes: at most $4,000 for the American
    // opportunity credit, and the return's $10,000 for the lifetime learning
    // credit shared in list order; zero when no credit is allowed. Form 1099-Q
    // reads this whole figure (its college costs include course materials).
    pub(crate) used: i64,
    // The part of `used` that falls on tuition and fees: `used` less the
    // course materials (the credit takes the materials first). Form 8815
    // line 2 reads this figure, because that line counts tuition and fees only.
    pub(crate) used_tuition: i64,
}

impl EducationAnnualProjection {
    pub(crate) fn method(&self) -> EducationMethod {
        match self.method {
            EducationChoice::AmericanOpportunity => EducationMethod::AmericanOpportunity,
            EducationChoice::LifetimeLearning => EducationMethod::LifetimeLearning,
        }
    }
}
