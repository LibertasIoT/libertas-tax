#[allow(unused_imports)]
use super::*;
use crate::{CashTipRecords, DisabilityIncome, EmployeeTips, TipCoverage, W2UncollectedTax};

/// W-2 wages
/// Enter annual totals once per employer. Combine multiple W-2s from that employer; use corrected amounts instead of duplicating the original.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalWage {
    /// Cash-tip records entered separately from a W-2
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(false)]
    pub tips_without_w2: bool,
    /// Box 1 wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Disability retirement wages for Schedule R
    pub disability: crate::DisabilityIncome,
    /// Box 1 pay for work abroad
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub abroad: i64,
    /// Box 1 pension from a nonqualified deferred compensation or nongovernmental 457 plan
    /// Schedule 1 line 8t instead of Form 1040 line 1a.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred_pension: i64,
    /// Box 1 pay for work while an inmate
    /// Schedule 1 line 8u instead of Form 1040 line 1a.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub inmate_pay: i64,
    /// Box 2 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Box 3 Social Security wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub social_wages: i64,
    /// Box 7 Social Security tips — do not add these to box 1 again
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social_tips: i64,
    /// Unreported or allocated employee tips
    /// Supply actual tip records or allocated tips separately from W-2 box 1. Leave absent when all tips are already included.
    pub employee_tips: Option<crate::EmployeeTips>,
    /// Qualified tips and overtime deductions
    pub work_deductions: FederalWorkDeductions,
    /// Box 4 Social Security tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub social_tax: i64,
    /// Box 5 Medicare wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medicare_wages: i64,
    /// Box 6 Medicare tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medicare_tax: i64,
    /// W-2 box 12 uncollected payroll taxes
    /// Codes A/B for reported tips or M/N for former-employee group-term life insurance. Leave absent when none.
    pub uncollected_tax: Option<crate::W2UncollectedTax>,
    /// Box 1 Medicaid waiver payments excluded on Schedule 1 line 8s
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medicaid_waiver: i64,
    /// Saved Schedule C for this statutory employee pay
    pub statutory_business: Option<i64>,
    /// S-corporation K-1 whose corporation paid this W-2
    pub paying_scorporation: Option<i64>,
    /// Social Security or railroad retirement
    pub payroll: W2Payroll,
    /// This W-2 is from a church or church-controlled organization that elected out of Social Security and Medicare taxes
    #[libertas_default(Ordinary)]
    pub coverage: crate::W2Coverage,
    /// Box 10 dependent care benefits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub care_benefits: i64,
    /// Box 13 statutory employee
    #[libertas_default(false)]
    pub statutory_employee: bool,
    /// Box 13 retirement plan
    #[libertas_default(false)]
    pub retirement_plan: bool,
    /// Box 12 codes not otherwise modeled
    /// A, B, M, N and TT have their own fields. TP is kept here only when its printed amount is zero, so its presence is not lost. Q, II and TA are read from here.
    pub other_box12: Vec<W2CodeAmount>,
}

/// W-2 box 12 code
/// Codes in the form's order, as the Instructions for Employee describe them.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum W2Code {
    /// A — uncollected social security or RRTA tax on tips
    #[libertas_form_code("A")]
    A,
    /// B — uncollected Medicare tax on tips
    #[libertas_form_code("B")]
    B,
    /// C — taxable cost of group-term life insurance over $50,000
    #[libertas_form_code("C")]
    C,
    /// D — 401(k) elective deferrals
    #[libertas_form_code("D")]
    D,
    /// E — 403(b) elective deferrals
    #[libertas_form_code("E")]
    E,
    /// F — SEP elective deferrals
    #[libertas_form_code("F")]
    F,
    /// G — 457(b) deferrals and employer contributions
    #[libertas_form_code("G")]
    G,
    /// H — 501(c)(18)(D) elective deferrals
    #[libertas_form_code("H")]
    H,
    /// J — nontaxable sick pay
    #[libertas_form_code("J")]
    J,
    /// K — 20% excise tax on excess golden parachute payments
    #[libertas_form_code("K")]
    K,
    /// L — substantiated employee business expense reimbursements
    #[libertas_form_code("L")]
    L,
    /// M — uncollected social security or RRTA tax on group-term life insurance (former employees)
    #[libertas_form_code("M")]
    M,
    /// N — uncollected Medicare tax on group-term life insurance (former employees)
    #[libertas_form_code("N")]
    N,
    /// P — excludable moving expense reimbursements (Armed Forces or intelligence community)
    #[libertas_form_code("P")]
    P,
    /// Q — nontaxable combat pay
    #[libertas_form_code("Q")]
    Q,
    /// R — employer Archer MSA contributions
    #[libertas_form_code("R")]
    R,
    /// S — SIMPLE elective deferrals
    #[libertas_form_code("S")]
    S,
    /// T — adoption benefits
    #[libertas_form_code("T")]
    T,
    /// V — income from exercise of nonstatutory stock options
    /// Already included in the wage boxes. Keep the amount with your stock records when checking basis after a sale.
    #[libertas_form_code("V")]
    V,
    /// W — employer contributions to your HSA
    #[libertas_form_code("W")]
    W,
    /// Y — deferrals under a section 409A nonqualified deferred compensation plan
    #[libertas_form_code("Y")]
    Y,
    /// Z — income under a nonqualified deferred compensation plan that fails section 409A
    #[libertas_form_code("Z")]
    Z,
    /// AA — designated Roth contributions under a 401(k) plan
    #[libertas_form_code("AA")]
    AA,
    /// BB — designated Roth contributions under a 403(b) plan
    #[libertas_form_code("BB")]
    BB,
    /// DD — cost of employer-sponsored health coverage
    #[libertas_form_code("DD")]
    DD,
    /// EE — designated Roth contributions under a governmental 457(b) plan
    #[libertas_form_code("EE")]
    EE,
    /// FF — permitted benefits under a qualified small employer health reimbursement arrangement
    #[libertas_form_code("FF")]
    FF,
    /// GG — income from qualified equity grants under section 83(i)
    #[libertas_form_code("GG")]
    GG,
    /// HH — aggregate deferrals under section 83(i) elections
    #[libertas_form_code("HH")]
    HH,
    /// II — Medicaid waiver payments excluded from gross income
    #[libertas_form_code("II")]
    II,
    /// TA — employer contributions to a Trump account
    #[libertas_form_code("TA")]
    TA,
    /// TP — total cash tips reported to the employer
    #[libertas_form_code("TP")]
    TP,
    /// TT — total qualified overtime compensation
    #[libertas_form_code("TT")]
    TT,
}

/// W-2 box 12 code and amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2CodeAmount {
    /// Code
    pub code: W2Code,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Wage document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum WageDocument {
    /// W-2
    W2(FormW2Paper),
    /// Employee pay without a W-2 (actual payroll coverage; Form 8919 when applicable)
    /// Pay from a firm that treated you as an independent contractor although you were its employee, with no Social Security or Medicare tax withheld. One for each firm.
    Form8919(crate::EmployeeWages),
}

/// W-2 Wage and Tax Statement
/// The printed boxes of Form W-2 in the form's order, exactly as the employer printed them. Box 9 is not used. If your employer gave you no W-2, enter your Form 4852 instead.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read employer boxes b and c into employer and employee boxes a, e and f into employee. Copy the printed names, identification numbers and addresses exactly, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer the employee from household names. Read box d exactly into control_number, preserving leading zeros and punctuation; return an empty string when blank. Read the W-2 exactly. Check its tax year against the supplied year; report a mismatch. Return null for a clearly blank amount in boxes 1 through 8, 10 or 11, preserving a printed zero as zero; report illegible amounts as errors. For Box 12, return each code actually entered in a code cell and supported by the result schema with its exact amount, including informational codes C, J, V, Y, DD, GG and HH. The preprinted word Code, including vertically stacked C/o/d/e, row markers 12a through 12d, headings and instructions are labels, never code entries. Return an empty list when all code cells are blank. Preserve an actually entered supported code when its amount is printed as zero or clearly blank; return null for that blank amount and zero only for printed zero. Omit absent codes and codes absent from the schema. The imported list replaces the existing Box 12 list. Informational amounts do not create additional wages, stock sales, stock basis, health contributions or retirement contributions. Code K is the reported excise tax itself; code Z is deferred compensation income. Do not multiply, divide or add either printed code amount to box 1 or box 2. Read the three Box 13 checkboxes (statutory employee, retirement plan, third-party sick pay), false when unchecked. Do not infer church employment, disability, or other facts absent from the W-2. The file may hold several copies of this form, such as Copy B, Copy C and Copy 2; copies of the same form are one form. Read box 14a items as their exact labels and amounts or other printed text, including railroad retirement (RRTA) compensation, Tier 1 tax, Tier 2 tax, Medicare tax and Additional Medicare Tax lines. Read up to two box 14b occupation codes as numbers from 0 through 999; their display retains three digits, including 000. Do not use box 14b occupation codes to decide a tax treatment. Read state boxes 15 through 17 and local boxes 18 through 20: copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return null for a blank state or local income or withholding box; preserve a printed zero as zero. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. If the file is a Form 4852, read line 7a as box 1, 7e as box 2, 7b as box 3, 7h as box 4, 7c as box 5, 7i as box 6 and 7d as box 7, and report the employer name from line 5; read no other line. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.employer.federal_id,@.employee.federal_id,@.wages,@.withholding,@.social_wages,@.social_tax,@.medicare_wages,@.medicare_tax,@.social_tips,@.allocated_tips,@.care_benefits,@.nonqualified_plans"
)]
pub struct FormW2Paper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Boxes b and c employer
    /// b Employer identification number (EIN); c Employer's name, address, and ZIP code
    #[libertas_llm_doc_field]
    pub employer: crate::PrintedParty,
    /// Box d Control number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub control_number: String,
    /// Boxes a, e and f employee
    /// a Employee's social security number; e Employee's first name and initial, last name, suffix; f Employee's address and ZIP code
    #[libertas_llm_doc_field]
    pub employee: crate::PrintedParty,
    /// Box 1 Wages, tips, other compensation
    /// [NullText]
    /// Box 1 Wages, tips, other compensation is blank
    /// [SetText]
    /// Box 1 Wages, tips, other compensation
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub wages: Option<i64>,
    /// Box 2 Federal income tax withheld
    /// [NullText]
    /// Box 2 Federal income tax withheld is blank
    /// [SetText]
    /// Box 2 Federal income tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 3 Social security wages
    /// [NullText]
    /// Box 3 Social security wages is blank
    /// [SetText]
    /// Box 3 Social security wages
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Tax structure: Form W-2 box 3 excludes Social Security wages above the annual wage base.
    #[libertas_number(min = 0, max = $SOCIAL_SECURITY_WAGE_BASE_CENTS)]
    #[libertas_default($null)]
    pub social_wages: Option<i64>,
    /// Box 4 Social security tax withheld
    /// [NullText]
    /// Box 4 Social security tax withheld is blank
    /// [SetText]
    /// Box 4 Social security tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub social_tax: Option<i64>,
    /// Box 5 Medicare wages and tips
    /// [NullText]
    /// Box 5 Medicare wages and tips is blank
    /// [SetText]
    /// Box 5 Medicare wages and tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub medicare_wages: Option<i64>,
    /// Box 6 Medicare tax withheld
    /// [NullText]
    /// Box 6 Medicare tax withheld is blank
    /// [SetText]
    /// Box 6 Medicare tax withheld
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub medicare_tax: Option<i64>,
    /// Box 7 Social security tips
    /// [NullText]
    /// Box 7 Social security tips is blank
    /// [SetText]
    /// Box 7 Social security tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub social_tips: Option<i64>,
    /// Box 8 Allocated tips
    /// [NullText]
    /// Box 8 Allocated tips is blank
    /// [SetText]
    /// Box 8 Allocated tips
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub allocated_tips: Option<i64>,
    /// Box 10 Dependent care benefits
    /// [NullText]
    /// Box 10 Dependent care benefits is blank
    /// [SetText]
    /// Box 10 Dependent care benefits
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub care_benefits: Option<i64>,
    /// Box 11 Nonqualified plans
    /// Already included in the printed federal wages; do not add it to wages again.
    /// [NullText]
    /// Box 11 is blank
    /// [SetText]
    /// Box 11 Nonqualified plans
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub nonqualified_plans: Option<i64>,
    /// Box 12 (12a to 12d)
    /// Copy each code actually entered in a code cell and its amount, retaining a printed zero and preserving a clearly blank amount as null.
    /// The preprinted Code label (including stacked C/o/d/e), 12a through 12d row markers, headings and instructions are not entries. Leave the list empty when all code cells are blank.
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Box 12 code
    pub box12: Vec<PrintedBox12Entry>,
    /// Box 13 Statutory employee
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub statutory: bool,
    /// Box 13 Retirement plan
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub retirement_plan: bool,
    /// Box 13 Third-party sick pay
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub third_party_sick_pay: bool,
    /// Box 14a Other
    #[libertas_llm_doc_field]
    /// ----
    /// Box 14a item
    pub box14a: Vec<crate::W2OtherItem>,
    /// Box 14b Treasury Tipped Occupation Code(s)
    /// Copy each three-digit code, including 000. These records do not decide whether tips qualify for a deduction.
    #[libertas_llm_doc_field]
    /// ----
    /// Tipped-occupation code
    pub tipped_occupation_codes: Vec<crate::TippedOccupationCode>,
    /// Boxes 15 to 17 State, employer's state ID number, state wages and income tax
    #[libertas_llm_doc_field]
    /// ----
    /// State wage row
    pub states: Vec<crate::StateIncomeTax>,
    /// Boxes 18 to 20 Local wages, local income tax and locality name
    #[libertas_llm_doc_field]
    /// ----
    /// Local wage row
    pub localities: Vec<crate::LocalIncomeTax>,
}

impl FormW2Paper {
    /// The internal W-2 model of the printed boxes alone: Social Security payroll.
    /// A railroad W-2's payroll is its wrapper's answer (`W2Source::native`).
    pub(crate) fn native(&self) -> W2 {
        let Self {
            calendar_year: _,
            employer,
            control_number,
            employee,
            wages,
            withholding,
            social_wages,
            social_tax,
            medicare_wages,
            medicare_tax,
            social_tips,
            allocated_tips,
            care_benefits,
            nonqualified_plans,
            box12,
            statutory,
            retirement_plan,
            third_party_sick_pay,
            box14a,
            tipped_occupation_codes,
            states,
            localities,
        } = self;
        W2 {
            employer: employer.clone(),
            employee: employee.clone(),
            wages: wages.unwrap_or(0),
            withholding: withholding.unwrap_or(0),
            social_wages: social_wages.unwrap_or(0),
            social_tax: social_tax.unwrap_or(0),
            medicare_wages: medicare_wages.unwrap_or(0),
            medicare_tax: medicare_tax.unwrap_or(0),
            more: W2More {
                social_tips: social_tips.unwrap_or(0),
                allocated_tips: allocated_tips.unwrap_or(0),
                care_benefits: care_benefits.unwrap_or(0),
                box12: box12.iter().map(Into::into).collect(),
                statutory: *statutory,
                retirement_plan: *retirement_plan,
                payroll: crate::W2Payroll::SocialSecurity,
            },
            state_details: crate::W2StateFields {
                control_number: control_number.clone(),
                nonqualified_plans: nonqualified_plans.unwrap_or(0),
                third_party_sick_pay: *third_party_sick_pay,
                states: states.clone(),
                localities: localities.clone(),
                box14a: box14a.clone(),
                tipped_occupation_codes: tipped_occupation_codes.clone(),
            },
        }
    }
    /// The state and local boxes the state-return owner reads.
    pub(crate) fn state_details(&self) -> crate::W2StateFields {
        self.native().state_details
    }
    /// Replace the state and local boxes from the state-return owner.
    pub(crate) fn set_state_details(&mut self, details: crate::W2StateFields) {
        let crate::W2StateFields {
            control_number,
            nonqualified_plans,
            third_party_sick_pay,
            states,
            localities,
            box14a,
            tipped_occupation_codes,
        } = details;
        self.control_number = control_number;
        if self.nonqualified_plans.unwrap_or(0) != nonqualified_plans {
            self.nonqualified_plans = Some(nonqualified_plans);
        }
        self.third_party_sick_pay = third_party_sick_pay;
        self.states = states;
        self.localities = localities;
        self.box14a = box14a;
        self.tipped_occupation_codes = tipped_occupation_codes;
    }
}
impl From<&FormW2Paper> for W2 {
    fn from(paper: &FormW2Paper) -> Self {
        paper.native()
    }
}

/// W-2
/// If your employer gave you no W-2, enter the amounts from your Form 4852 (line 7a is box 1, 7e box 2, 7b box 3, 7h box 4, 7c box 5, 7i box 6).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2 {
    /// Boxes b and c employer
    #[libertas_llm_doc_field]
    pub employer: crate::PrintedParty,
    /// Boxes a, e and f employee
    #[libertas_llm_doc_field]
    pub employee: crate::PrintedParty,
    /// Box 1 wages
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    /// Box 2 federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 3 Social Security wages
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Tax structure: Form W-2 box 3 excludes Social Security wages above the annual wage base.
    #[libertas_number(min = 0, max = $SOCIAL_SECURITY_WAGE_BASE_CENTS)]
    #[libertas_default(0)]
    pub social_wages: i64,
    /// Box 4 Social Security tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social_tax: i64,
    /// Box 5 Medicare wages
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_wages: i64,
    /// Box 6 Medicare tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_tax: i64,
    /// Boxes 7–14
    #[libertas_llm_doc_field]
    pub more: W2More,
    /// Additional wage, state and local boxes
    #[libertas_llm_doc_field]
    pub state_details: crate::W2StateFields,
}

/// Boxes 7–14
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct W2More {
    /// Box 7 Social Security tips
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social_tips: i64,
    /// Box 8 allocated tips
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub allocated_tips: i64,
    /// Box 10 dependent care benefits
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub care_benefits: i64,
    /// Box 12
    /// Copy each printed code and amount. An absent code needs no row.
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Box 12 code
    pub box12: Vec<Box12Entry>,
    /// Box 13 statutory employee
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub statutory: bool,
    /// Box 13 retirement plan
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub retirement_plan: bool,
    /// Social Security or railroad retirement
    /// Railroad employers leave boxes 3 to 7 blank and list RRTA amounts in box 14a.
    #[libertas_llm_doc_field(blank = SocialSecurity)]
    #[libertas_default(SocialSecurity)]
    pub payroll: W2Payroll,
}

/// Box 12 code and printed amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PrintedBox12Entry {
    /// Code
    #[libertas_llm_doc_field(allow_empty = false)]
    pub code: W2Code,
    /// Amount
    /// [NullText]
    /// Amount is blank
    /// [SetText]
    /// Amount
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub amount: Option<i64>,
}
impl From<&PrintedBox12Entry> for Box12Entry {
    fn from(value: &PrintedBox12Entry) -> Self {
        Self { code: value.code, amount: value.amount.unwrap_or(0) }
    }
}
impl From<&Box12Entry> for PrintedBox12Entry {
    fn from(value: &Box12Entry) -> Self {
        Self { code: value.code, amount: Some(value.amount) }
    }
}
/// Social Security or railroad retirement
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum W2Payroll {
    /// Social Security and Medicare (boxes 3 to 7)
    #[default]
    SocialSecurity,
    /// Railroad retirement (box 14a)
    /// The App figures Additional Medicare Tax separately on this compensation and credits the Additional Medicare Tax withheld. Tier 1 tax counts with Social Security tax for the excess-withholding credit.
    Railroad {
        /// RRTA compensation, including reported tips
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
        /// Tier 1 tax
        #[libertas_llm_doc_field(blank = 0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        tier_1: i64,
        /// Tier 2 tax, claimed separately on Form 843 for an overpayment
        #[libertas_llm_doc_field(blank = 0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        tier_2: i64,
        /// Medicare tax, without Additional Medicare Tax
        #[libertas_llm_doc_field(blank = 0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        medicare: i64,
        /// Additional Medicare Tax withheld
        #[libertas_llm_doc_field(blank = 0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        additional_medicare: i64,
    },
}

impl W2Payroll {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::SocialSecurity => true,
            Self::Railroad { compensation, tier_1, tier_2, medicare, additional_medicare } => {
                [*compensation, *tier_1, *tier_2, *medicare, *additional_medicare]
                    .iter()
                    .all(|v| (0..=100000000000).contains(v))
            }
        }
    }
}

/// Eligibility of this W-2's tips or overtime for the corresponding deduction
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum WageDeductionEligibility {
    /// No
    Ineligible,
    /// Yes
    Eligible,
}

/// An amount this W-2 reports
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum W2Amount {
    /// Enter the amount
    Amount {
        /// Amount
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
impl W2Amount {
    #[cfg(test)]
    pub(crate) fn of(amount: i64) -> Option<Self> {
        (amount != 0).then_some(Self::Amount { amount })
    }
    pub(crate) fn cents(this: Option<Self>) -> i64 {
        this.map_or(0, |Self::Amount { amount }| amount)
    }
}

/// Tips recorded day by day
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DailyTipRecord {
    /// Enter the months
    Months {
        /// Months
        #[libertas_unordered]
        /// ----
        /// Month
        months: Vec<crate::CashTipMonth>,
    },
}

/// The follow-up answers this W-2's boxes and your return call for
/// Each list holds the one choice that opens its answer, or is empty while nothing calls for it.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct W2Calls {
    /// Tip records and noncash tips
    pub tips: Vec<i32>,
    /// Tipped occupation
    pub occupation: Vec<i32>,
    /// Overtime
    pub overtime: Vec<i32>,
    /// Church employee or minister
    pub coverage: Vec<i32>,
    /// Disability retirement
    pub disability: Vec<i32>,
    /// Pay for work abroad
    pub abroad: Vec<i32>,
    /// Nonqualified plan pension
    pub deferred_pension: Vec<i32>,
    /// Statutory employee Schedule C
    pub statutory_business: Vec<i32>,
}

/// The W-2 boxes that decide which follow-up answers show
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2Boxes {
    /// Box 1 wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    /// Box 3 Social Security wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social_wages: i64,
    /// Box 5 Medicare wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_wages: i64,
    /// Boxes 7, 8, 12 and 13
    pub more: W2MoreBoxes,
    /// Box 11
    pub state_details: W2StateBoxes,
}

/// Boxes 7, 8, 12 and 13
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2MoreBoxes {
    /// Box 7 Social Security tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social_tips: i64,
    /// Box 8 allocated tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub allocated_tips: i64,
    /// Box 12 codes
    /// ----
    /// Box 12 code
    pub box12: Vec<Box12Entry>,
    /// Box 13 statutory employee
    #[libertas_default(false)]
    pub statutory: bool,
    /// Social Security or railroad retirement
    #[libertas_default(SocialSecurity)]
    pub payroll: W2Payroll,
}

/// Box 11
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2StateBoxes {
    /// Box 11 nonqualified plans
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonqualified_plans: i64,
}
impl From<&W2> for W2Boxes {
    fn from(w: &W2) -> Self {
        Self {
            wages: w.wages,
            social_wages: w.social_wages,
            medicare_wages: w.medicare_wages,
            more: W2MoreBoxes {
                social_tips: w.more.social_tips,
                allocated_tips: w.more.allocated_tips,
                box12: w.more.box12.clone(),
                statutory: w.more.statutory,
                payroll: w.more.payroll.clone(),
            },
            state_details: W2StateBoxes { nonqualified_plans: w.state_details.nonqualified_plans },
        }
    }
}

// What only the person knows about this W-2, beside the imported boxes and
// never an import target: a document import cannot clear it. A row shows only
// when the boxes or the return call for it (W2Calls); the rest stay out of the
// editor.
/// Your answers about this W-2
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct W2Answers {
    /// Follow-up answers the boxes and the return call for
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: W2Calls,
    /// Tips from your daily record
    /// Only if you kept a daily tip record for this employer: it replaces the box 8 allocation as your unreported tips. Shown when box 7, box 8 or a box 12 TP code reports tips.
    /// [NullText]
    /// I kept no daily tip record for this employer
    /// [SetText]
    /// I kept a daily tip record for this employer
    #[libertas_default($null)]
    pub tip_records: Option<DailyTipRecord>,
    /// Your tips are from an occupation on the IRS list of tipped occupations
    /// Answer when box 12 shows code TP, box 8 shows allocated tips or you entered daily tip records. Box 14b shows the occupation code when your employer reported tips. No takes no qualified-tips deduction.
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    pub tips_listed_occupation: Option<WageDeductionEligibility>,
    /// Box 12 code TT: this is overtime pay the Fair Labor Standards Act requires
    /// Answer when box 12 shows code TT. No takes no qualified-overtime deduction.
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    pub overtime_flsa_required: Option<WageDeductionEligibility>,
    /// Church employee who elected out of Social Security and Medicare
    /// Shown when box 1 has pay and boxes 3 and 5 are zero.
    /// [NullText]
    /// Ordinary pay
    #[libertas_default($null)]
    pub coverage: Option<crate::W2Coverage>,
    /// Disability retirement wages for Schedule R
    /// Shown when you or your spouse is marked disabled on About you or About your spouse.
    /// [NullText]
    /// None
    #[libertas_default($null)]
    pub disability: Option<crate::DisabilityIncome>,
    /// Box 1 pay for work you did abroad
    /// For Form 2555, shown when you have a Form 2555 record on Work abroad: the part of box 1 paid for services you performed in a foreign country while your tax home was there, as your employer's statement or your workday record shows. Include employer housing value, 2025-service pay and the complete awards listed on Form 2555; the App removes their nonqualifying portions once. Do not include U.S. Government employee pay, pensions, advances for later services or other pay for U.S. work. For statutory employee pay (box 13), leave this empty and select a saved Schedule C, even if it has no expenses; mark its services source there instead.
    /// [NullText]
    /// None of box 1 was paid for work abroad
    /// [SetText]
    /// Part of box 1 was paid for work abroad
    #[libertas_default($null)]
    pub abroad: Option<W2Amount>,
    /// Box 1 pension from a nonqualified deferred compensation or nongovernmental 457 plan
    /// Shown when box 11 has an amount. A pension or annuity from such a plan, usually also in box 11. The App moves it from Form 1040 line 1a to Schedule 1 line 8t.
    /// [NullText]
    /// Box 1 holds no such pension
    /// [SetText]
    /// Box 1 holds a pension from such a plan
    #[libertas_default($null)]
    pub deferred_pension: Option<W2Amount>,
    /// Box 1 pay for work while an inmate
    /// Pay for work you did while an inmate in a penal institution. The App moves it from Form 1040 line 1a to Schedule 1 line 8u; it is not earned income for the earned income credit. Leave zero otherwise.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub inmate_pay: i64,
    /// Noncash tips
    /// Value of goods, passes and other noncash tips from customers in this job. Shown when box 7, box 8 or a box 12 TP code reports tips. Income on Form 1040 line 1c, with no tip payroll tax or tips deduction.
    /// [NullText]
    /// No noncash tips
    /// [SetText]
    /// Noncash tips
    #[libertas_default($null)]
    pub noncash_tips: Option<W2Amount>,
    /// Box 1 Medicaid waiver payments you can exclude
    /// State Medicaid waiver payments for caring for someone who lived in your home with you (Notice 2014-7) that your employer left in box 1. The App subtracts them on Schedule 1 line 8s. Payments already excluded from box 1 use code II. Leave zero otherwise.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicaid_waiver: i64,
    /// Schedule C for this statutory employee pay
    /// Optional override of the computed zero-expense Schedule C. Shown when box 13 statutory employee is checked: select the saved business holding only this job's expenses. The null answer keeps the computed Schedule C using box 1 with no expenses.
    /// [NullText]
    /// Use the computed zero-expense Schedule C for statutory employee pay
    /// [SetText]
    /// Use this saved Schedule C for statutory employee pay
    #[libertas_default($null)]
    pub statutory_business: Option<StatutoryBusiness>,
    /// S corporation that paid this W-2
    /// Select its actual Schedule K-1 on this return. Never match employers by a similar name or amount.
    /// [NullText]
    /// Not paid by one of my S corporations
    /// [SetText]
    /// Paid by this S corporation
    #[libertas_default($null)]
    pub paying_scorporation: Option<crate::business_credit_page::WageScorpEmployer>,
    /// What each printed box 14 item is, by the item's index in box 14a
    /// Filled once when the W-2 is saved from the printed label and state; an answer already recorded is kept.
    #[libertas_hidden]
    #[libertas_read_only]
    pub box14: Vec<W2Box14Meaning>,
}

/// One printed box 14 item's meaning
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2Box14Meaning {
    /// Index of the printed item in box 14a
    pub item: u32,
    /// Its state, when the item belongs to one
    /// [NullText]
    /// No state
    pub state: Option<crate::TaxState>,
    /// What the item is; empty until classified or answered
    /// [NullText]
    /// Not answered yet
    pub kind: Option<W2Box14Kind>,
}

/// What a printed box 14 item is
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum W2Box14Kind {
    /// State disability or family-leave contribution
    /// A mandatory state fund contribution (CA SDI or VPDI, NJ DI or FLI, NY SDI or PFL, RI TDI, WA or MA PFML). It is state income tax on Schedule A when you itemize.
    StateDisabilityOrLeave,
    /// Public-employee pension pickup
    /// An amount excluded from box 1 that the state still taxes (NY 414(h), STPICKUP).
    StatePickup,
    /// Railroad retirement
    /// Already the church-or-railroad answer; no federal figure here.
    Railroad,
    /// Union dues
    UnionDues,
    /// Pre-tax health, dental or cafeteria-plan amount
    /// Already out of box 1.
    PretaxBenefit,
    /// Information only
    /// For example a vehicle, tuition or uniform.
    Informational,
    /// None of these
    Other,
}

impl W2Box14Kind {
    /// Whether the kind belongs to one state.
    pub(crate) fn needs_state(self) -> bool {
        matches!(self, Self::StateDisabilityOrLeave | Self::StatePickup)
    }
    // The label's words, state removed and joined, against the known printings.
    fn of_label(core: &str) -> Option<Self> {
        Some(match core {
            "sdi" | "vpdi" | "di" | "fli" | "tdi" | "pfl" | "pfml" | "uiwfswf" | "uiwf" => {
                Self::StateDisabilityOrLeave
            }
            "414h" | "stpickup" | "pickup" | "statepickup" => Self::StatePickup,
            core if core.starts_with("rrta") || matches!(core, "tier1" | "tier2" | "railroad") => {
                Self::Railroad
            }
            "uniondues" | "union" | "dues" => Self::UnionDues,
            "dental" | "health" | "medical" | "cafeteria" | "section125" | "sec125" | "125"
            | "healthinsurance" => Self::PretaxBenefit,
            "uniform" | "tuition" | "vehicle" => Self::Informational,
            _ => return None,
        })
    }
}

impl W2Box14Meaning {
    /// A row is asked while its kind is open, or its state is missing where the kind needs one.
    pub(crate) fn asks(&self) -> bool {
        self.kind.is_none_or(|kind| kind.needs_state() && self.state.is_none())
    }
    // The meaning of one printed label; `printed_state` is the W-2's single box 15 state.
    fn detect(item: u32, label: &str, printed_state: Option<crate::TaxState>) -> Self {
        let (mut state, core) = crate::us_address::split_state(label);
        let mut kind = W2Box14Kind::of_label(&core);
        if kind.is_none()
            && state.is_none()
            && let Some((coded, rest)) = crate::us_address::code_prefix(&core)
            && let Some(found) = W2Box14Kind::of_label(rest)
        {
            (state, kind) = (Some(coded), Some(found));
        }
        let state = kind.filter(|kind| kind.needs_state()).and_then(|_| state.or(printed_state));
        Self { item, state, kind }
    }
}

impl W2Source {
    /// Fills the meaning of each box 14a item without a recorded kind. A recorded kind stays
    /// through later saves and paper edits; entries follow the printed items by index.
    pub(crate) fn classify_box14(&mut self) {
        let printed_state = match self.form.states.as_slice() {
            [first, rest @ ..] if rest.iter().all(|row| row.state == first.state) => {
                Some(first.state)
            }
            _ => None,
        };
        let kept = core::mem::take(&mut self.answers.box14);
        self.answers.box14 = self
            .form
            .box14a
            .iter()
            .zip(0..)
            .map(|(printed, item)| {
                kept.iter()
                    .find(|meaning| meaning.item == item && meaning.kind.is_some())
                    .cloned()
                    .unwrap_or_else(|| W2Box14Meaning::detect(item, &printed.label, printed_state))
            })
            .collect();
    }
    /// The questions still open, each with its printed label.
    pub(crate) fn box14_asks(&self) -> Vec<W2Box14Ask> {
        self.answers
            .box14
            .iter()
            .filter(|meaning| meaning.asks())
            .filter_map(|meaning| {
                let printed = self.form.box14a.get(meaning.item as usize)?;
                Some(W2Box14Ask {
                    item: meaning.item,
                    label: printed.label.clone(),
                    state: meaning.state,
                    kind: meaning.kind,
                })
            })
            .collect()
    }
    /// State disability and family-leave contributions printed in box 14a, in cents.
    pub(crate) fn state_disability_cents(&self) -> i64 {
        self.answers
            .box14
            .iter()
            .filter(|meaning| meaning.kind == Some(W2Box14Kind::StateDisabilityOrLeave))
            .filter_map(|meaning| self.form.box14a.get(meaning.item as usize))
            .map(|printed| match printed.value {
                crate::W2OtherValue::Money { amount } => amount,
                crate::W2OtherValue::Text { .. } => 0,
            })
            .sum()
    }
}

/// One open box 14 question
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2Box14Ask {
    /// Index of the printed item in box 14a
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_key]
    #[libertas_unique]
    pub item: u32,
    /// Box 14 item as printed
    #[libertas_read_only]
    #[libertas_default("")]
    pub label: String,
    /// State of the item
    /// Needed for a state disability, family-leave or pickup amount when the label and box 15 show none.
    /// [NullText]
    /// Not answered yet
    pub state: Option<crate::TaxState>,
    /// What is this box 14 item?
    /// [NullText]
    /// Not answered yet
    pub kind: Option<W2Box14Kind>,
}

/// Your answers about this W-2
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct W2AnswersView {
    /// Follow-up answers the boxes and the return call for
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: W2Calls,
    /// Tips from your daily record
    /// Only if you kept a daily tip record for this employer: it replaces the box 8 allocation as your unreported tips. Shown when box 7, box 8 or a box 12 TP code reports tips.
    /// [NullText]
    /// I kept no daily tip record for this employer
    /// [SetText]
    /// I kept a daily tip record for this employer
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.tips")]
    pub tip_records: Option<DailyTipRecord>,
    /// Your tips are from an occupation on the IRS list of tipped occupations
    /// Answer when box 12 shows code TP, box 8 shows allocated tips or you entered daily tip records. Box 14b shows the occupation code when your employer reported tips. No takes no qualified-tips deduction.
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.occupation")]
    pub tips_listed_occupation: Option<WageDeductionEligibility>,
    /// Box 12 code TT: this is overtime pay the Fair Labor Standards Act requires
    /// Answer when box 12 shows code TT. No takes no qualified-overtime deduction.
    /// [NullText]
    /// Not answered yet
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.overtime")]
    pub overtime_flsa_required: Option<WageDeductionEligibility>,
    /// Church employee who elected out of Social Security and Medicare
    /// Shown when box 1 has pay and boxes 3 and 5 are zero.
    /// [NullText]
    /// Ordinary pay
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.coverage")]
    pub coverage: Option<crate::W2Coverage>,
    /// Disability retirement wages for Schedule R
    /// Shown when you or your spouse is marked disabled on About you or About your spouse.
    /// [NullText]
    /// None
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.disability")]
    pub disability: Option<crate::DisabilityIncome>,
    /// Box 1 pay for work you did abroad
    /// For Form 2555, shown when you have a Form 2555 record on Work abroad: the part of box 1 paid for services you performed in a foreign country while your tax home was there, as your employer's statement or your workday record shows. Include employer housing value, 2025-service pay and the complete awards listed on Form 2555; the App removes their nonqualifying portions once. Do not include U.S. Government employee pay, pensions, advances for later services or other pay for U.S. work. For statutory employee pay (box 13), leave this empty and select a saved Schedule C, even if it has no expenses; mark its services source there instead.
    /// [NullText]
    /// None of box 1 was paid for work abroad
    /// [SetText]
    /// Part of box 1 was paid for work abroad
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.abroad")]
    pub abroad: Option<W2Amount>,
    /// Box 1 pension from a nonqualified deferred compensation or nongovernmental 457 plan
    /// Shown when box 11 has an amount. A pension or annuity from such a plan, usually also in box 11. The App moves it from Form 1040 line 1a to Schedule 1 line 8t.
    /// [NullText]
    /// Box 1 holds no such pension
    /// [SetText]
    /// Box 1 holds a pension from such a plan
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.deferred_pension")]
    pub deferred_pension: Option<W2Amount>,
    /// Box 1 pay for work while an inmate
    /// Pay for work you did while an inmate in a penal institution. The App moves it from Form 1040 line 1a to Schedule 1 line 8u; it is not earned income for the earned income credit. Leave zero otherwise.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub inmate_pay: i64,
    /// Noncash tips
    /// Value of goods, passes and other noncash tips from customers in this job. Shown when box 7, box 8 or a box 12 TP code reports tips. Income on Form 1040 line 1c, with no tip payroll tax or tips deduction.
    /// [NullText]
    /// No noncash tips
    /// [SetText]
    /// Noncash tips
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.tips")]
    pub noncash_tips: Option<W2Amount>,
    /// Box 1 Medicaid waiver payments you can exclude
    /// State Medicaid waiver payments for caring for someone who lived in your home with you (Notice 2014-7) that your employer left in box 1. The App subtracts them on Schedule 1 line 8s. Payments already excluded from box 1 use code II. Leave zero otherwise.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicaid_waiver: i64,
    /// Schedule C for this statutory employee pay
    /// Optional override of the computed zero-expense Schedule C. Shown when box 13 statutory employee is checked: select the saved business holding only this job's expenses. The null answer keeps the computed Schedule C using box 1 with no expenses.
    /// [NullText]
    /// Use the computed zero-expense Schedule C for statutory employee pay
    /// [SetText]
    /// Use this saved Schedule C for statutory employee pay
    #[libertas_default($null)]
    #[libertas_constrained_by("^<W2AnswersView>.calls.statutory_business")]
    pub statutory_business: Option<StatutoryBusinessView>,
    /// S corporation that paid this W-2
    /// Select its actual Schedule K-1 on this return. Never match employers by a similar name or amount.
    /// [NullText]
    /// Not paid by one of my S corporations
    /// [SetText]
    /// Paid by this S corporation
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_wage_scorporations")]
    pub paying_scorporation: Option<crate::business_credit_page::WageScorpEmployerView>,
    /// Box 14 items still to classify
    /// One row for each printed item whose meaning the label and state could not settle.
    pub box14: Vec<W2Box14Ask>,
}

/// Answers about an accepted W-2
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2AnswerRow {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// W-2 employer and printed details
    #[libertas_hidden]
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub document: Vec<u8>,
    /// Answers
    pub answers: W2AnswersView,
}

impl From<&W2Answers> for W2AnswersView {
    fn from(value: &W2Answers) -> Self {
        let W2Answers {
            calls,
            tip_records,
            tips_listed_occupation,
            overtime_flsa_required,
            coverage,
            disability,
            abroad,
            deferred_pension,
            inmate_pay,
            noncash_tips,
            medicaid_waiver,
            statutory_business,
            paying_scorporation,
            box14: _,
        } = value;
        Self {
            calls: calls.clone(),
            tip_records: tip_records.clone(),
            tips_listed_occupation: *tips_listed_occupation,
            overtime_flsa_required: *overtime_flsa_required,
            coverage: coverage.clone(),
            disability: disability.clone(),
            abroad: *abroad,
            deferred_pension: *deferred_pension,
            inmate_pay: *inmate_pay,
            noncash_tips: *noncash_tips,
            medicaid_waiver: *medicaid_waiver,
            statutory_business: statutory_business.as_ref().map(Into::into),
            paying_scorporation: paying_scorporation.as_ref().map(Into::into),
            box14: Vec::new(),
        }
    }
}

impl From<&W2AnswersView> for W2Answers {
    fn from(value: &W2AnswersView) -> Self {
        let W2AnswersView {
            calls,
            tip_records,
            tips_listed_occupation,
            overtime_flsa_required,
            coverage,
            disability,
            abroad,
            deferred_pension,
            inmate_pay,
            noncash_tips,
            medicaid_waiver,
            statutory_business,
            paying_scorporation,
            box14,
        } = value;
        Self {
            calls: calls.clone(),
            tip_records: tip_records.clone(),
            tips_listed_occupation: *tips_listed_occupation,
            overtime_flsa_required: *overtime_flsa_required,
            coverage: coverage.clone(),
            disability: disability.clone(),
            abroad: *abroad,
            deferred_pension: *deferred_pension,
            inmate_pay: *inmate_pay,
            noncash_tips: *noncash_tips,
            medicaid_waiver: *medicaid_waiver,
            statutory_business: statutory_business.as_ref().map(Into::into),
            paying_scorporation: paying_scorporation.as_ref().map(Into::into),
            // The answered rows only; the save merges them into the recorded list.
            box14: box14
                .iter()
                .map(|ask| W2Box14Meaning { item: ask.item, state: ask.state, kind: ask.kind })
                .collect(),
        }
    }
}
impl W2Answers {
    pub(crate) fn tip_months(&self) -> &[crate::CashTipMonth] {
        self.tip_records.as_ref().map_or(&[], |DailyTipRecord::Months { months }| months)
    }
}

/// The saved Schedule C holding this statutory employee's expenses
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum StatutoryBusiness {
    /// A saved Schedule C
    Business {
        /// Business
        source: i64,
    },
}

/// The saved Schedule C holding this statutory employee's expenses
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum StatutoryBusinessView {
    /// A saved Schedule C
    Business {
        /// Business
        #[libertas_enum_source("$.businesses")]
        source: i64,
    },
}

impl From<&StatutoryBusiness> for StatutoryBusinessView {
    fn from(value: &StatutoryBusiness) -> Self {
        match value {
            StatutoryBusiness::Business { source } => Self::Business { source: *source },
        }
    }
}
impl From<&StatutoryBusinessView> for StatutoryBusiness {
    fn from(value: &StatutoryBusinessView) -> Self {
        match value {
            StatutoryBusinessView::Business { source } => Self::Business { source: *source },
        }
    }
}

/// Box 12 code and amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Box12Entry {
    /// Code
    #[libertas_llm_doc_field]
    pub code: W2Code,
    /// Amount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
}

/// Accepted wage source
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum WageSource {
    /// W-2 and the answers supplied on its topic page
    W2(W2Source),
    /// Cash tips recorded without a W-2
    CashTipsWithoutW2(crate::CashTipsWithoutW2),
}
/// Accepted W-2 and personal answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2Source {
    /// Printed W-2
    pub form: FormW2Paper,
    /// Personal answers
    pub answers: W2Answers,
}
impl W2Source {
    /// The internal W-2 model for this printed form and its collected payroll.
    pub(crate) fn native(&self) -> W2 {
        let mut w: W2 = (&self.form).into();
        // The railroad answer counts only while boxes 3 and 5 are blank, as when it is asked.
        if w.social_wages == 0
            && w.medicare_wages == 0
            && let Some(coverage) = &self.answers.coverage
        {
            w.more.payroll = coverage.into();
        }
        w
    }
    pub(crate) fn printed(form: FormW2Paper) -> Self {
        Self { form, answers: Default::default() }
    }
}
impl WageSource {
    pub(crate) fn is_cash_tips(&self) -> bool {
        matches!(self, Self::CashTipsWithoutW2(_))
    }
    pub(crate) fn answers(&self) -> Option<&W2Answers> {
        match self {
            Self::W2(source) => Some(&source.answers),
            Self::CashTipsWithoutW2(_) => None,
        }
    }
    pub(crate) fn answers_mut(&mut self) -> Option<&mut W2Answers> {
        match self {
            Self::W2(source) => Some(&mut source.answers),
            Self::CashTipsWithoutW2(_) => None,
        }
    }
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        if let (Self::W2(source), Self::W2(old)) = (self, accepted) {
            source.answers = old.answers.clone();
        }
    }
}
impl From<&WageSource> for FederalWage {
    fn from(source: &WageSource) -> Self {
        match source {
            WageSource::W2(source) => source.into(),
            WageSource::CashTipsWithoutW2(value) => cash_tips_record(value),
        }
    }
}
pub(crate) fn unreported_tips_amount(allocated: i64, records: &[crate::CashTipMonth]) -> i64 {
    if records.is_empty() {
        allocated
    } else {
        records.iter().map(|m| m.received - m.reported).sum()
    }
}
pub(crate) fn unreported_tips_record(
    allocated: i64,
    records: &[crate::CashTipMonth],
    social_wages: i64,
    medicare_wages: i64,
    noncash: i64,
    payroll: &crate::W2Payroll,
) -> Option<EmployeeTips> {
    let cash = if !records.is_empty() {
        CashTipRecords::Actual { months: records.to_vec() }
    } else if allocated != 0 {
        CashTipRecords::Allocated { amount: allocated }
    } else if noncash != 0 {
        CashTipRecords::Actual { months: Vec::new() }
    } else {
        return None;
    };
    Some(EmployeeTips {
        cash,
        noncash,
        coverage: if matches!(payroll, crate::W2Payroll::Railroad { .. }) {
            TipCoverage::Railroad
        } else if social_wages == 0 && medicare_wages != 0 {
            TipCoverage::MedicareOnlyGovernment
        } else {
            TipCoverage::SocialAndMedicare
        },
    })
}
// Box 12: A/B/M/N fill uncollected_tax, TP/TT fill work_deductions, and every
// other code is stored, unread by the calculation, in other_box12.
pub(crate) fn box12_records(
    box12: &[Box12Entry],
    answers: &crate::W2Answers,
    unreported_tips: i64,
) -> (FederalWorkDeductions, Option<W2UncollectedTax>, Vec<W2CodeAmount>) {
    let mut uncollected =
        W2UncollectedTax { tip_social: 0, tip_medicare: 0, life_social: 0, life_medicare: 0 };
    let mut has_uncollected = false;
    let mut tips_claim = if unreported_tips > 0 {
        TipsClaim::Yes {
            w2_tips: 0,
            form4137_tips: unreported_tips,
            tips_eligible: answers
                .tips_listed_occupation
                .map(|answer| answer == crate::WageDeductionEligibility::Eligible),
        }
    } else {
        TipsClaim::No
    };
    let mut overtime_claim = OvertimeClaim::No;
    let mut other = Vec::new();
    for entry in box12 {
        let amount = entry.amount;
        match entry.code {
            W2Code::A => {
                uncollected.tip_social = amount;
                has_uncollected = true;
            }
            W2Code::B => {
                uncollected.tip_medicare = amount;
                has_uncollected = true;
            }
            W2Code::M => {
                uncollected.life_social = amount;
                has_uncollected = true;
            }
            W2Code::N => {
                uncollected.life_medicare = amount;
                has_uncollected = true;
            }
            // The save holds each answer present with its code
            // (W2_TP_OCCUPATION_NEEDED, W2_TT_FLSA_NEEDED).
            W2Code::TP => {
                // Preserve a printed zero code without inventing one for tips-only claims.
                if amount == 0 {
                    other.push(W2CodeAmount { code: W2Code::TP, amount });
                }
                tips_claim = TipsClaim::Yes {
                    w2_tips: amount,
                    // Form 4137 is filed only for unreported tips; its column
                    // 1(c) then holds the reported and unreported tips. With
                    // every tip reported there is no Form 4137 amount.
                    form4137_tips: if unreported_tips > 0 { amount + unreported_tips } else { 0 },
                    tips_eligible: answers
                        .tips_listed_occupation
                        .map(|answer| answer == crate::WageDeductionEligibility::Eligible),
                };
            }
            W2Code::TT => {
                overtime_claim = OvertimeClaim::Yes {
                    overtime: amount,
                    overtime_eligible: answers
                        .overtime_flsa_required
                        .map(|answer| answer == crate::WageDeductionEligibility::Eligible),
                };
            }
            code => other.push(W2CodeAmount { code, amount }),
        }
    }
    let work_deductions = FederalWorkDeductions { tips_claim, overtime_claim };
    (work_deductions, has_uncollected.then_some(uncollected), other)
}
pub(crate) fn cash_tips_record(v: &crate::CashTipsWithoutW2) -> FederalWage {
    let crate::CashTipsWithoutW2 { employer: _, months, coverage: _, qualified_occupation } = v;
    let mut data = FederalWage {
        tips_without_w2: true,
        wages: 0,
        disability: DisabilityIncome::None,
        abroad: 0,
        deferred_pension: 0,
        inmate_pay: 0,
        withholding: 0,
        social_wages: 0,
        social_tips: 0,
        employee_tips: None,
        work_deductions: FederalWorkDeductions::NONE,
        social_tax: 0,
        medicare_wages: 0,
        medicare_tax: 0,
        uncollected_tax: None,
        medicaid_waiver: 0,
        statutory_business: None,
        paying_scorporation: None,
        payroll: crate::W2Payroll::SocialSecurity,
        coverage: crate::W2Coverage::Ordinary,
        care_benefits: 0,
        statutory_employee: false,
        retirement_plan: false,
        other_box12: Vec::new(),
    };
    data.employee_tips = Some(v.tips());
    data.work_deductions.tips_claim = TipsClaim::Yes {
        w2_tips: 0,
        form4137_tips: unreported_tips_amount(0, months),
        tips_eligible: Some(*qualified_occupation),
    };
    data
}
impl From<&W2Source> for FederalWage {
    fn from(source: &W2Source) -> Self {
        let W2Source { form: _, answers } = source;
        let w = &source.native();
        let W2 {
            employer: _,
            employee: _,
            wages: _,
            withholding: _,
            social_wages: _,
            social_tax: _,
            medicare_wages: _,
            medicare_tax: _,
            more: _,
            state_details: _,
        } = w;
        let W2Answers {
            calls: _,
            tip_records: _,
            tips_listed_occupation: _,
            overtime_flsa_required: _,
            coverage: _,
            disability: _,
            abroad: _,
            deferred_pension: _,
            inmate_pay: _,
            noncash_tips: _,
            medicaid_waiver: _,
            statutory_business: _,
            paying_scorporation: _,
            box14: _,
        } = answers;
        let mut data = FederalWage {
            tips_without_w2: false,
            wages: w.wages,
            disability: DisabilityIncome::None,
            abroad: W2Amount::cents(answers.abroad),
            deferred_pension: W2Amount::cents(answers.deferred_pension),
            inmate_pay: answers.inmate_pay,
            withholding: w.withholding,
            social_wages: w.social_wages,
            social_tips: 0,
            employee_tips: None,
            work_deductions: FederalWorkDeductions::NONE,
            social_tax: w.social_tax,
            medicare_wages: w.medicare_wages,
            medicare_tax: w.medicare_tax,
            uncollected_tax: None,
            medicaid_waiver: 0,
            statutory_business: None,
            paying_scorporation: None,
            payroll: crate::W2Payroll::SocialSecurity,
            coverage: crate::W2Coverage::Ordinary,
            care_benefits: 0,
            statutory_employee: false,
            retirement_plan: false,
            other_box12: Vec::new(),
        };
        let W2More {
            social_tips,
            allocated_tips,
            care_benefits,
            box12,
            statutory,
            retirement_plan,
            payroll,
        } = &w.more;
        data.social_tips = *social_tips;
        data.care_benefits = *care_benefits;
        data.statutory_employee = *statutory;
        data.retirement_plan = *retirement_plan;
        data.payroll = payroll.clone();
        data.medicaid_waiver = answers.medicaid_waiver;
        data.statutory_business = answers
            .statutory_business
            .filter(|_| *statutory)
            .map(|crate::StatutoryBusiness::Business { source }| source);
        data.paying_scorporation = answers
            .paying_scorporation
            .as_ref()
            .map(crate::business_credit_page::WageScorpEmployer::id);
        data.coverage = answers.coverage.clone().unwrap_or(crate::W2Coverage::Ordinary);
        data.disability = answers.disability.clone().unwrap_or(DisabilityIncome::None);
        let months = answers.tip_months();
        data.employee_tips = unreported_tips_record(
            *allocated_tips,
            months,
            w.social_wages,
            w.medicare_wages,
            W2Amount::cents(answers.noncash_tips),
            payroll,
        );
        let (work_deductions, uncollected_tax, other_box12) =
            box12_records(box12, answers, unreported_tips_amount(*allocated_tips, months));
        data.work_deductions = work_deductions;
        data.uncollected_tax = uncollected_tax;
        data.other_box12 = other_box12;
        data
    }
}
