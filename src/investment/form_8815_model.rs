// Savings-bond source records and Form 8815 document entry.
#[allow(unused_imports)]
use super::*;

/// Savings-bond interest reporting at death
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FinalBondReporting {
    /// Interest was reported annually; include this year's remaining accrual
    Annual,
    /// Interest was deferred; elect all unreported pre-death interest on this return
    Elect,
    /// Interest was deferred; leave unreported interest for the estate or beneficiary
    Defer,
}

/// Series EE and I bonds held at death
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalBonds {
    /// Total purchase cost of these bonds
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Total redemption value at the date of death
    /// Use Treasury redemption values, not face value or later beneficiary proceeds.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Interest already included in earlier income tax returns
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub reported: i64,
    pub reporting: FinalBondReporting,
    /// Complete solely owned, unredeemed and unmatured EE/I bond holdings
    /// The decedent purchased these bonds; no joint ownership, inherited basis, HH exchange, prior redemption or maturity, education exclusion or duplicated income entry. All these holdings used the same reporting method. The election includes all their unreported pre-death interest, not selected bonds. Retain the final-return interest record for the recipient to avoid taxing it again. The bond registrations, TreasuryDirect or paper bond records and earlier returns show this. No leaves these bonds out of the estimate with a notice.
    pub eligible: bool,
}
impl FinalBonds {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.cost)
            && (self.cost..=100000000000).contains(&self.value)
            && (0..=self.value - self.cost).contains(&self.reported)
    }
    pub(crate) fn unreported(&self) -> i64 {
        self.value - self.cost - self.reported
    }
}

/// Savings bonds cashed for education — Form 8815
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSavingsBonds {
    /// Total proceeds from series EE and I bonds issued after 1989 cashed in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub proceeds: i64,
    /// Interest included in the proceeds
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Earliest issue date of the bonds cashed
    #[libertas_date_only]
    pub earliest_issue_date: u32,
    /// Accepted interest statement or annual instrument for these bonds
    pub interest_statement: i64,
    /// People whose education was paid for
    #[libertas_size(min = 1)]
    /// ----
    /// Person
    pub students: Vec<FederalSavingsBondStudent>,
}
impl FederalSavingsBonds {
    pub(crate) fn valid(&self) -> bool {
        (1..=100_000_000_000).contains(&self.proceeds)
            && self.interest_statement > 0
            && (0..=self.proceeds).contains(&self.interest)
            && crate::federal_rules::valid_date(self.earliest_issue_date)
            && (crate::year_constants::SAVINGS_BOND_FIRST_ISSUE_DATE..=20261231)
                .contains(&self.earliest_issue_date)
            && (1..=10).contains(&self.students.len())
            && self.students.iter().all(FederalSavingsBondStudent::valid)
            && self
                .students
                .iter()
                .enumerate()
                .all(|(i, s)| !self.students[..i].iter().any(|prior| prior.person == s.person))
    }
}

/// Person whose education was paid for
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSavingsBondStudent {
    /// Person identity
    /// −1 you, −2 your spouse, otherwise the dependent's accepted record identity.
    #[libertas_hidden]
    pub person: i64,
    /// Tuition and fees
    pub expenses: FederalSavingsBondExpenses,
    /// Contributions in 2026 to a 529 plan or Coverdell ESA for this person
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
}
impl FederalSavingsBondStudent {
    pub(super) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.contributions)
            && match self.expenses {
                FederalSavingsBondExpenses::EducationCredit => true,
                FederalSavingsBondExpenses::Entered { tuition, benefits } => {
                    (0..=100_000_000_000).contains(&tuition)
                        && (0..=100_000_000_000).contains(&benefits)
                }
            }
    }
}

/// Tuition and fees
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalSavingsBondExpenses {
    /// From this person's education credit entry
    EducationCredit,
    /// Entered on the savings bond record
    Entered {
        /// Tuition and fees required for enrollment or attendance
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tuition: i64,
        /// Tax-free educational benefits covering those expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        benefits: i64,
    },
}

/// Savings bonds cashed for education (Form 8815)
/// Series EE or I bonds issued after 1989 and cashed in 2026, whose interest is in box 3 of a 1099-INT entered on this page. Enter it once for the whole return; keep Form 8818 or your redemption records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 8818 (Optional Form To Record Redemption of Series EE and I U.S. Savings Bonds Issued After 1989) or an equivalent redemption record exactly. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read line 6 total redemption proceeds into proceeds and line 8 total interest into redemption_interest; a blank or illegible line 6 or line 8 is an error, never zero. From every issue date in column (b) of lines 1 and 4 return the earliest one as earliest_issue_date; report an error when no issue date is legible. Do not read serial numbers or face values, and do not decide who owns the bonds, the canonical source link, or any education expense; the filer supplies those. Previously reported interest comes from the linked source, never this import. The file may hold a cashing receipt as well; use the Form 8818 totals when both are present.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct SavingsBonds {
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Total proceeds from the bonds cashed in 2026
    /// Form 8818 line 6: the principal and interest the bank or TreasuryDirect paid you.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub proceeds: i64,
    /// Interest included in those proceeds
    /// Form 8818 line 8. The same interest is in box 3 of the 1099-INT entered on this page.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub redemption_interest: i64,
    /// Earliest issue date of the bonds cashed
    /// Form 8818 column (b). The bonds must be issued after 1989 and after the owner turned 24; the App checks this from the owner's birth date.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub earliest_issue_date: u32,
    /// Accepted interest statement or annual instrument for these bonds
    /// Enter the interest statement first, then select its accepted source. Previously reported interest comes from that source. Use a separate annual instrument when one payer statement combines different bonds.
    #[libertas_enum_source("$.interest_statements")]
    pub interest_statement: i64,
    /// People whose education you paid for in 2026
    /// You, your spouse, or a dependent on this return.
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    /// ----
    /// Person
    pub students: Vec<SavingsBondStudent>,
}

/// Person whose education you paid for
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SavingsBondStudent {
    /// Person
    #[libertas_enum_source("$.people")]
    #[libertas_unique]
    pub person: i64,
    /// Tuition and fees
    #[libertas_default(EducationCredit)]
    pub expenses: SavingsBondExpenses,
    /// Contributions in 2026 to a 529 plan or Coverdell ESA for this person
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contributions: i64,
}

/// Where this person's tuition and fees come from
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SavingsBondExpenses {
    /// From this person's education credit entry
    /// The App uses the tuition, scholarships, refunds and 529 amounts entered on Education credits, after the expenses that credit used.
    EducationCredit,
    /// Entered here
    /// For a person with no education credit entry.
    Entered {
        /// Tuition and fees required for enrollment or attendance in 2026
        /// At an accredited college, university or vocational school eligible for federal student aid. Not room and board, books, or sports, game or hobby courses outside a degree or certificate program.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tuition: i64,
        /// Tax-free scholarships, grants, veterans' or employer educational assistance and 529 or Coverdell payments covering those expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        benefits: i64,
    },
}

impl From<&SavingsBondExpenses> for FederalSavingsBondExpenses {
    fn from(expenses: &SavingsBondExpenses) -> Self {
        match expenses {
            SavingsBondExpenses::EducationCredit => Self::EducationCredit,
            SavingsBondExpenses::Entered { tuition, benefits } => {
                Self::Entered { tuition: *tuition, benefits: *benefits }
            }
        }
    }
}

impl From<&FederalSavingsBondExpenses> for SavingsBondExpenses {
    fn from(expenses: &FederalSavingsBondExpenses) -> Self {
        match expenses {
            FederalSavingsBondExpenses::EducationCredit => Self::EducationCredit,
            FederalSavingsBondExpenses::Entered { tuition, benefits } => {
                Self::Entered { tuition: *tuition, benefits: *benefits }
            }
        }
    }
}

impl From<&SavingsBondStudent> for FederalSavingsBondStudent {
    fn from(student: &SavingsBondStudent) -> Self {
        let SavingsBondStudent { person, expenses, contributions } = student;
        Self { person: *person, expenses: expenses.into(), contributions: *contributions }
    }
}

impl From<&FederalSavingsBondStudent> for SavingsBondStudent {
    fn from(student: &FederalSavingsBondStudent) -> Self {
        let FederalSavingsBondStudent { person, expenses, contributions } = student;
        Self { person: *person, expenses: expenses.into(), contributions: *contributions }
    }
}

// One-way: every printed fact either reaches the internal model or is named as
// not read by any calculation, so a new one cannot be forgotten.
impl From<&SavingsBonds> for FederalSavingsBonds {
    fn from(source: &SavingsBonds) -> Self {
        let SavingsBonds {
            // The source statement's parties name the document; no figure reads them.
            source_issuer: _,
            source_recipient: _,
            proceeds,
            redemption_interest,
            earliest_issue_date,
            interest_statement,
            students,
        } = source;
        FederalSavingsBonds {
            proceeds: *proceeds,
            interest: *redemption_interest,
            earliest_issue_date: *earliest_issue_date,
            interest_statement: *interest_statement,
            students: students
                .iter()
                .map(|s| {
                    let SavingsBondStudent { person, expenses, contributions } = s;
                    FederalSavingsBondStudent {
                        person: *person,
                        expenses: match expenses {
                            SavingsBondExpenses::EducationCredit => {
                                FederalSavingsBondExpenses::EducationCredit
                            }
                            SavingsBondExpenses::Entered { tuition, benefits } => {
                                FederalSavingsBondExpenses::Entered {
                                    tuition: *tuition,
                                    benefits: *benefits,
                                }
                            }
                        },
                        contributions: *contributions,
                    }
                })
                .collect(),
        }
    }
}

/// Exclusion of Interest From Series EE and I U.S. Savings Bonds Issued After 1989
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 8815 exactly in printed order: the name and social security number shown on the return, line 1 columns (a) and (b), and lines 2 through 14. Preserve printed names, identification numbers and institution addresses. Blank numeric lines mean zero. Read lines 7 and 12 as decimals, not percentages. Report illegible or conflicting values as errors. Never infer bond issue dates, gross redemption interest, source links or education-record answers from this form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.taxpayer.federal_id,@.line2,@.line3,@.line4,@.line5,@.line6,@.line8,@.line9,@.line10,@.line11,@.line13,@.line14"
)]
pub struct Form8815Paper {
    /// Name(s) shown on return — Your social security number
    #[libertas_llm_doc_field]
    pub taxpayer: crate::PrintedParty,
    /// 1 (a) Name of person (you, your spouse, or your dependent) who was enrolled at or attended an eligible educational institution — (b) Name and address of eligible educational institution
    #[libertas_llm_doc_field]
    pub line1: Vec<Form8815EducationRow>,
    /// 2 Enter the total qualified higher education expenses you paid in 2026 for the person(s) listed in column (a) of line 1. See the instructions to find out which expenses qualify
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line2: i64,
    /// 3 Enter the total of any nontaxable educational benefits (such as nontaxable scholarship or fellowship grants) received for 2026 for the person(s) listed in column (a) of line 1. See instructions
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line3: i64,
    /// 4 Subtract line 3 from line 2. If zero or less, stop. You cannot take the exclusion
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub line4: i64,
    /// 5 Enter the total proceeds (principal and interest) from all series EE and I U.S. savings bonds issued after 1989 that you cashed during 2026
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line5: i64,
    /// 6 Enter the interest included on line 5. See instructions
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line6: i64,
    /// 7 If line 4 is equal to or more than line 5, enter “1.000.” If line 4 is less than line 5, divide line 4 by line 5. Enter the result as a decimal (rounded to at least three places)
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub line7: f64,
    /// 8 Multiply line 6 by line 7
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line8: i64,
    /// 9 Enter your modified adjusted gross income. See instructions
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub line9: i64,
    /// 10 Enter $101,800 if single, head of household, or qualifying surviving spouse; or $152,650 if married filing jointly
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line10: i64,
    /// 11 Subtract line 10 from line 9. If zero or less, skip line 12, enter -0- on line 13, and go to line 14
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub line11: i64,
    /// 12 Divide line 11 by $15,000 if single, head of household, or qualifying surviving spouse; or $30,000 if married filing jointly. Enter the result as a decimal (rounded to at least three places)
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub line12: f64,
    /// 13 Multiply line 8 by line 12
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line13: i64,
    /// 14 Excludable savings bond interest. Subtract line 13 from line 8. Enter the result here and on Schedule B (Form 1040), line 3
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line14: i64,
}

/// Form 8815 line 1
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8815EducationRow {
    /// (a) Name of person (you, your spouse, or your dependent) who was enrolled at or attended an eligible educational institution
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub person: String,
    /// (b) Name and address of eligible educational institution
    #[libertas_llm_doc_field]
    pub institution: crate::PrintedPartyNameAddress,
}

/// Form 8815 and its supporting redemption records
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 8815 into form exactly as printed. Read supporting Form 8818 or equivalent redemption records into redemption_interest and earliest_issue_date. For redemption_interest use Form 8818 line 8 or the record's total interest before previously reported interest, never Form 8815 line 6: line 6 may already subtract previously reported interest. For earliest_issue_date use the earliest issue date in Form 8818 column (b) or the redemption records. Leave either supporting fact null when the supplied documents do not establish it. Never infer a zero amount or issue date. Preserve every printed Form 8815 line, including a line 6 that differs from gross redemption interest. Do not decide ownership, source links, education expenses or previously reported interest.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.form.taxpayer.federal_id,@.form.line2,@.form.line3,@.form.line4,@.form.line5,@.form.line6,@.form.line8,@.form.line9,@.form.line10,@.form.line11,@.form.line13,@.form.line14",
    form_date_order = "MonthDayYear"
)]
pub struct SavingsBondsEntry {
    /// Form 8815 as printed
    #[libertas_llm_doc_field]
    pub form: Form8815Paper,
    /// Interest included in the redemption proceeds before previously reported interest
    /// Form 8818 line 8 or the redemption record. Form 8815 line 6 may already exclude previously reported interest; the linked source supplies that adjustment once.
    /// [NullText]
    /// The supplied records do not show gross redemption interest
    /// [SetText]
    /// Gross redemption interest shown by the supplied records
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub redemption_interest: Option<i64>,
    /// Earliest issue date of the bonds cashed
    /// Form 8818 column (b) or the redemption records.
    /// [NullText]
    /// The supplied records do not show an issue date
    /// [SetText]
    /// Earliest issue date shown by the supplied records
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    #[libertas_date_only]
    pub earliest_issue_date: Option<u32>,
}

/// Personal source links and education records supporting Form 8815
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SavingsBondAnswers {
    /// Accepted interest statement or annual instrument for these bonds
    pub interest_statement: i64,
    /// People whose education you paid for in 2026
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    /// ----
    /// Person
    pub students: Vec<FederalSavingsBondStudent>,
}

/// Personal source links and education records supporting Form 8815
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SavingsBondAnswersView {
    /// Accepted interest statement or annual instrument for these bonds
    #[libertas_enum_source("$.interest_statements")]
    pub interest_statement: i64,
    /// People whose education you paid for in 2026
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    /// ----
    /// Person
    pub students: Vec<SavingsBondStudent>,
}

impl From<&SavingsBondAnswers> for SavingsBondAnswersView {
    fn from(value: &SavingsBondAnswers) -> Self {
        Self {
            interest_statement: value.interest_statement,
            students: value.students.iter().map(Into::into).collect(),
        }
    }
}
impl From<SavingsBondAnswersView> for SavingsBondAnswers {
    fn from(value: SavingsBondAnswersView) -> Self {
        Self {
            interest_statement: value.interest_statement,
            students: value.students.iter().map(Into::into).collect(),
        }
    }
}

/// Form 8815 and its supporting bond and education answers
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct SavingsBondsSource {
    pub form: Form8815Paper,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub redemption_interest: Option<i64>,
    #[libertas_date_only]
    pub earliest_issue_date: Option<u32>,
    #[libertas_default($null)]
    pub answers: Option<SavingsBondAnswers>,
}
impl SavingsBondsSource {
    pub(crate) fn printed(entry: SavingsBondsEntry) -> Self {
        let SavingsBondsEntry { form, redemption_interest, earliest_issue_date } = entry;
        Self { form, redemption_interest, earliest_issue_date, answers: None }
    }
    pub(crate) fn entry(&self) -> SavingsBondsEntry {
        SavingsBondsEntry {
            form: self.form.clone(),
            redemption_interest: self.redemption_interest,
            earliest_issue_date: self.earliest_issue_date,
        }
    }
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.answers = accepted.answers.clone();
    }
    pub(crate) fn natural(&self) -> Option<SavingsBonds> {
        let SavingsBondsSource { form, redemption_interest, earliest_issue_date, answers } = self;
        let Form8815Paper {
            taxpayer,
            // Printed institution names and calculated lines remain source evidence;
            // the education and phaseout owners compute the exclusion.
            line1: _,
            line2: _,
            line3: _,
            line4: _,
            line5,
            line6: _,
            line7: _,
            line8: _,
            line9: _,
            line10: _,
            line11: _,
            line12: _,
            line13: _,
            line14: _,
        } = form;
        let SavingsBondAnswers { interest_statement, students } = answers.as_ref()?;
        Some(SavingsBonds {
            source_issuer: crate::PrintedParty::default(),
            source_recipient: taxpayer.clone(),
            proceeds: *line5,
            redemption_interest: (*redemption_interest)?,
            earliest_issue_date: (*earliest_issue_date)?,
            interest_statement: *interest_statement,
            students: students.iter().map(Into::into).collect(),
        })
    }
    pub(crate) fn record(&self) -> Option<FederalSavingsBonds> {
        self.natural().as_ref().map(FederalSavingsBonds::from)
    }
}

/// Optional Form To Record Redemption of Series EE and I U.S. Savings Bonds Issued After 1989
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 8818 exactly as printed: Name, Date cashed, every row of lines 1 and 4, and lines 2, 3, 5, 6, 7 and 8. Preserve serial numbers, leading zeros and the printed issue dates. Blank numeric boxes mean zero. Leave an absent or illegible date null; never infer it. Do not require a Form 8815, invent education expenses, choose ownership or supply an internal source identity. Do not recompute or replace printed totals. Report conflicting printed values as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.name,@.date_cashed,@.line2,@.line3,@.line5,@.line6,@.line7,@.line8",
    form_date_order = "MonthDayYear"
)]
pub struct Form8818Paper {
    /// Name
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub name: String,
    /// Date cashed
    /// [NullText]
    /// The paper does not show a legible cashing date
    /// [SetText]
    /// Cashing date printed on the paper
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    #[libertas_date_only]
    pub date_cashed: Option<u32>,
    /// 1 Paper Series EE Bonds — (a) Serial number, (b) Issue date, (c) Face value
    #[libertas_llm_doc_field]
    pub line1: Vec<Form8818BondRow>,
    /// 2 Add the amounts in column (c) of line 1
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line2: i64,
    /// 3 Multiply line 2 by 50% (.50). This is your cost of the paper series EE bonds cashed
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line3: i64,
    /// 4 Series I Bonds and Electronic Series EE Bonds — (a) Serial number, (b) Issue date, (c) Face value
    #[libertas_llm_doc_field]
    pub line4: Vec<Form8818BondRow>,
    /// 5 Add the amounts in column (c) of line 4. This is your cost of the series I bonds and electronic series EE bonds cashed
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line5: i64,
    /// 6 Total redemption proceeds from the bonds listed in Parts I and II
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line6: i64,
    /// 7 Add lines 3 and 5. This is your total cost of the bonds listed in Parts I and II
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line7: i64,
    /// 8 Subtract line 7 from line 6. This is the total interest on the bonds listed in Parts I and II
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line8: i64,
}

/// Form 8818 bond as printed
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8818BondRow {
    /// (a) Serial number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub serial_number: String,
    /// (b) Issue date (must be after 1989)
    /// [NullText]
    /// The paper does not show a legible issue date
    /// [SetText]
    /// Issue date printed on the paper
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    #[libertas_date_only]
    pub issue_date: Option<u32>,
    /// (c) Face value
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub face_value: i64,
}

impl Form8818Paper {
    pub(crate) fn earliest_issue_date(&self) -> Option<u32> {
        self.line1.iter().chain(&self.line4).try_fold(None, |earliest, row| {
            let date = row.issue_date?;
            Some(Some(earliest.map_or(date, |prior: u32| prior.min(date))))
        })?
    }
}

/// Form 8818 and its separately answered topic association
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct BondRedemptionSource {
    pub form: Form8818Paper,
    pub form_8815: Option<i64>,
}

/// Form 8815 supported by this redemption record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BondRedemptionLink {
    /// Form 8815 supported by these bonds
    #[libertas_enum_source("^<DocumentQuestion>.savings_bond_exclusions")]
    pub form_8815: i64,
}
