// Coverage: Form 1099-Q — Supported (audited 2026-10-09, unit B3 Form 1099-Q).
//   Lines: every box of the Rev. April 2025 form: 1 gross, 2 earnings, 3 basis, 4a trustee transfer
//   (with the part that does not qualify), 4b 529 to Roth IRA (ira.rs), 5a/5b private and state 529
//   plan, 5c Coverdell ESA, 6 recipient is not the beneficiary, 7 fair market value (Coverdell
//   only) and codes 1-6 (code 6, a prohibited transaction, is a withdrawal of the January 1 value
//   that no education expenses cover, with the 10% additional tax: unit B11 KIDS, 2026-10-11,
//   IRC 530(e), 408(e)(2)). Publication 970
//   Worksheet 6-3 (Coverdell taxable distributions and basis, lines A-16, including the year-end
//   value only path), the chapter 7 QTP steps (earnings times adjusted qualified expenses over
//   distributions), the K-12 list with its $20,000 aggregate cap for a 529 plan, apprenticeships,
//   the $10,000 lifetime student loan limit, and the allocation of one student's expenses between
//   529 and Coverdell distributions (Pub 970 ch. 6 Example 2 and ch. 7 Example 3). Expenses are
//   reduced first by tax-free assistance, then by the expenses an education credit used (credit
//   first, FederalCredits::education_expenses_used, shared with Form 8815) and expenses used
//   elsewhere. Taxable earnings go to Schedule 1 line 8z; Form 5329 Part II lines 5-8 per filer
//   (retirement.rs account_additional_tax) take the 10% additional tax with the exceptions for
//   death, disability, an education credit, scholarships and military academy costs, and a timely
//   returned Coverdell excess. The expenses the exclusion used feed Form 8815 line 3, the excluded
//   earnings reduce the educator expenses (IRC 62(d)(2)) and the earnings paid to the student's own
//   loans reduce the student loan interest deduction (IRC 221(e)(1)).
//   Entry: OtherIncomeDocument::Q1099 (Form 1099-Q, importable) and IncomeRecord::EducationAccountExpenses
//   (one record per return, a row per student) on the Prizes, scholarships, relief payments, account payouts and other income page.
//   Coverdell contribution excess and carry are owned by coverdell.rs (Form 5329 Part V).
//   Exclusions: a prohibited transaction (code 6) is refused; the Education credits page figures its own credit; a
//   sibling's student loans and the earlier-year return for a code 3 excess are not computed;
//   a loss in the account's final year is a miscellaneous itemized deduction that IRC 67 disallows,
//   so it is never deducted.
//   Tests: src/tests/family/education_accounts.rs (fixtures F1-F7 and F9, the Form 8815 no-double-count case,
//   the credit-first note, transfers and codes, the issues), src/tests/family/interview.rs
//   education_account_*, ira_page_asks_earlier_529_transfers_*, q1099_import_*; unit tests below.
//   Sources: https://www.irs.gov/pub/irs-pdf/f1099q.pdf, https://www.irs.gov/pub/irs-pdf/i1099q.pdf,
//   https://www.irs.gov/pub/irs-pdf/p970.pdf (2025), https://www.irs.gov/pub/irs-dft/f5329--dft.pdf
//   (2026 draft), 26 U.S.C. 529, 530, 25A(g)(2), 62(d)(2), 221(e)(1), P.L. 119-21 sections 70413
//   and 70414; constants in year_constants.rs.
// Native source projection controls: src/tests/attribution/unit13_account_sources.rs (recipient,
// source cents, penalty character, correction coordination and Save/restart).
use crate::education_account_source::earnings_in;
pub use crate::education_account_source::{
    CollegeCosts, CoverdellAmounts, DistributionReason, EducationAccount,
    EducationDistributionSource, FederalEducationAccountExpenses, FederalEducationAccountStudent,
    FederalEducationBeneficiary, FederalEducationDistribution, Q1099, Q1099Transfer,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::year_constants::{QTP_K12_ANNUAL_LIMIT_CENTS, QTP_LOAN_LIFETIME_LIMIT_CENTS};
use crate::{
    FederalCredits, FederalIncome, FederalIncomeEntry, FederalResult, FederalStatus, Message,
    MessageArg, Owner, TaxSection,
};
use crate::{federal_rules::note_once_scoped, line_emission::line_cents};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const SOURCE: &str = "https://www.irs.gov/pub/irs-pdf/p970.pdf";

/// Student (box 6)
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
pub enum Q1099Beneficiary {
    /// The person this form is addressed to is the student
    /// Box 6 is not checked.
    Recipient,
    /// Someone else on this return is the student
    /// Box 6 is checked: you own the account and the money was paid to you.
    Student {
        /// Student
        #[libertas_enum_source("$.people")]
        person: i64,
    },
    /// The student is not on this return
    /// Box 6 is checked and the student is not you, your spouse or your dependent. Assign this form its share of the student's 2026 costs. A military academy exception is figured only for a student on this return.
    OtherStudent {
        /// The student's qualified education expenses you assign to this distribution
        /// College costs, room and board for a half-time student, and other costs a 529 or Coverdell can pay (see 529 and Coverdell expenses). Assign each expense to one distribution only.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
        /// Tax-free scholarships, grants and other tax-free educational assistance toward those expenses
        /// Pell grants, veterans' and employer assistance count; gifts and loans do not.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        assistance: i64,
        /// Those expenses used for anyone's American opportunity or lifetime learning credit
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        credit_expenses: i64,
    },
}

/// 529 and Coverdell expenses
/// Enter it once for the whole return: one row for each student on this return whose 529 plan or Coverdell ESA paid out in 2026. Keep bills, receipts and Form 1098-T.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationAccountExpenses {
    /// Students
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    /// ----
    /// Student
    pub students: Vec<EducationAccountStudent>,
}

/// Student
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationAccountStudent {
    /// Student
    #[libertas_enum_source("$.people")]
    #[libertas_unique]
    pub person: i64,
    /// College tuition and required books
    #[libertas_inline_choices]
    pub college: CollegeCosts,
    /// Room, board, computer and other college costs
    /// Room and board while enrolled at least half time, up to the school's allowance or its own housing charge; a computer, software or internet used mainly by the student while enrolled; special-needs services; recognized credential programs, their tests and required continuing education.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_college: i64,
    /// K–12 costs on the 529 list
    /// Tuition, curriculum and instructional materials, books, online materials, outside tutoring by a qualified unrelated tutor, standardized, AP and admission test fees, dual-enrollment fees and educational therapies. A 529 counts at most $20,000 a year; for a Coverdell, therapies count only for a student with special needs.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub k12: i64,
    /// Other K–12 costs only a Coverdell ESA can pay
    /// Room and board, uniforms, transportation and extended-day programs the school requires or provides, other fees, supplies and academic tutoring, and computer equipment the family uses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub k12_coverdell: i64,
    /// Registered apprenticeship fees, books, supplies and equipment
    /// 529 only.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub apprenticeship: i64,
    /// Payments on this student's own qualified student loans
    /// 529 only: principal and interest. A sibling's loans are not counted here. Interest paid with tax-free 529 earnings can't also be deducted as student loan interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub loan_payments: i64,
    /// 529 money used for this student's loans before 2026
    /// The lifetime limit is $10,000 per student.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub loan_payments_before: i64,
    /// U.S. military academy costs of advanced education
    /// For a cadet or midshipman: the academy's figure. It only removes the 10% additional tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub academy_costs: i64,
    /// Costs already used elsewhere
    /// Costs counted for a 529 or Coverdell withdrawal reported on someone else's return, such as the student's own, or deducted as a business expense.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub used_elsewhere: i64,
}

impl From<&Q1099Beneficiary> for FederalEducationBeneficiary {
    fn from(value: &Q1099Beneficiary) -> Self {
        match value {
            Q1099Beneficiary::Recipient => Self::Recipient,
            Q1099Beneficiary::Student { person } => Self::Student { person: *person },
            Q1099Beneficiary::OtherStudent { expenses, assistance, credit_expenses } => {
                Self::OtherStudent {
                    expenses: *expenses,
                    assistance: *assistance,
                    credit_expenses: *credit_expenses,
                }
            }
        }
    }
}
impl From<&FederalEducationBeneficiary> for Q1099Beneficiary {
    fn from(value: &FederalEducationBeneficiary) -> Self {
        match value {
            FederalEducationBeneficiary::Recipient => Self::Recipient,
            FederalEducationBeneficiary::Student { person } => Self::Student { person: *person },
            FederalEducationBeneficiary::OtherStudent { expenses, assistance, credit_expenses } => {
                Self::OtherStudent {
                    expenses: *expenses,
                    assistance: *assistance,
                    credit_expenses: *credit_expenses,
                }
            }
        }
    }
}

// What the calculation reads from the rest of the return.
pub(crate) struct EducationContext<'a> {
    // The supported working copy's income entries.
    pub(crate) income: &'a [FederalIncomeEntry],
    pub(crate) coverdell: &'a [crate::CoverdellExcess],
    pub(crate) credits: &'a FederalCredits,
    // The people on the return, the students an education entry can name.
    pub(crate) students: &'a [crate::PersonChoice],
    pub(crate) status: FederalStatus,
    // The dependents claimed on this return, by person identity.
    pub(crate) dependent_ids: &'a [i64],
    // The App allows an education credit (not married filing separately, not
    // another taxpayer's dependent).
    pub(crate) credit_allowed: bool,
    // The part of each 529-to-Roth transfer above the year's room, by Form 1099-Q entry.
    pub(crate) qtp_excess: &'a alloc::collections::BTreeMap<i64, i64>,
}

// Final native amounts of one actual 1099-Q recipient, not its student.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EducationSource {
    pub(crate) id: i64,
    pub(crate) owner: Owner,
    pub(crate) taxable: i64,
    pub(crate) exceptions: i64,
}

// What the distributions report on the return, in cents; index 0 is the
// taxpayer and 1 the spouse (the distribution's recipient).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EducationTotals {
    // Actual distribution sources, in cents, captured after shared expense coordination.
    pub(crate) sources: Vec<EducationSource>,
    // Actual unlinked contribution corrections, already attributed by their owner.
    // These have no income-entry ID and are not distribution sources.
    pub(crate) correction_income: [i64; 2],
    // Schedule 1 line 8z and Form 5329 Part II line 5.
    pub(crate) income: [i64; 2],
    // Form 5329 Part II line 6.
    pub(crate) exceptions: [i64; 2],
    // Earnings left out of income, which cut the educator expenses
    // (IRC 62(d)(2)).
    pub(crate) excluded_earnings: [i64; 2],
    // Excluded earnings paid to a filer's own student loans, which cut the
    // student loan interest deduction (IRC 221(e)(1)).
    pub(crate) loan_excluded: i64,
    // The expenses the tax-free earnings used, by person identity.
    pub(crate) expenses_used: Vec<(i64, i64)>,
}

impl EducationTotals {
    // Cents of one person's expenses used by tax-free education account
    // earnings (IRC 135(d)(2)(B)); None when no distribution names the person.
    pub(crate) fn used_by(&self, person: i64) -> Option<i64> {
        self.expenses_used.iter().find(|(id, _)| *id == person).map(|(_, cents)| *cents)
    }
}

// The distributions and expenses of the working copy.
fn expenses_record(
    income: &[FederalIncomeEntry],
) -> Option<(&FederalIncomeEntry, &FederalEducationAccountExpenses)> {
    income.iter().find_map(|entry| match &entry.income {
        FederalIncome::EducationAccountExpenses { data } => Some((entry, data)),
        _ => None,
    })
}

// What one student's expenses are, in cents.
#[derive(Clone, Debug, Default)]
struct Pools {
    // Tuition, fees and required books and supplies, before any reduction.
    college: i64,
    // Room, board, computer and the other college costs.
    other_college: i64,
    // K-12 costs on the 529 list.
    k12: i64,
    // K-12 costs only a Coverdell can pay.
    k12_coverdell: i64,
    // Apprenticeship costs and the allowed student loan payments (529 only).
    apprenticeship: i64,
    loans: i64,
    // Tax-free assistance, the expenses an education credit used and the
    // expenses used elsewhere (Pub 970 Worksheet 6-3 lines B to D).
    assistance: i64,
    credit: i64,
    elsewhere: i64,
    academy: i64,
}

// The expenses that cover each kind of distribution of one student.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Covered {
    // Adjusted qualified expenses for the 529 distributions and the part of
    // them that pays student loans.
    qtp: i64,
    qtp_loans: i64,
    // Adjusted qualified expenses for the Coverdell distributions.
    coverdell: i64,
}

impl Pools {
    // Pub 970 ch. 6 Example 2 and ch. 7 Example 3 ("any reasonable method"):
    // expenses only one kind can pay go to that kind first; the shared
    // remainder is divided in proportion to the distributions still
    // uncovered, and a 529's K-12 part stops at the annual limit.
    fn cover(&self, with_credit: bool, qtp: i64, coverdell: i64) -> Covered {
        let mut reduction =
            self.assistance + self.elsewhere + if with_credit { self.credit } else { 0 };
        let mut left = |pool: i64| {
            let used = crate::kernel::carry::native_consume_step(pool, &mut reduction);
            pool - used
        };
        let college = left(self.college);
        let other = left(self.other_college);
        let k12 = left(self.k12);
        let own = self.apprenticeship + self.loans;
        let qtp_own = phase::cap(own, [qtp]);
        let qtp_loans = if own > 0 { fraction(qtp_own, self.loans, own) } else { 0 };
        let coverdell_own = phase::cap(self.k12_coverdell, [coverdell]);
        let (qtp_rest, coverdell_rest) = (qtp - qtp_own, coverdell - coverdell_own);
        let shared = college + other + k12;
        let qtp_room = college + other + phase::cap(k12, [QTP_K12_ANNUAL_LIMIT_CENTS]);
        let wanted = qtp_rest + coverdell_rest;
        let by_share = if wanted > 0 { fraction(shared, qtp_rest, wanted) } else { 0 };
        let mut qtp_shared = phase::cap(phase::cap(by_share, [qtp_room]), [qtp_rest]);
        let coverdell_shared = phase::cap(shared - qtp_shared, [coverdell_rest]);
        // What the Coverdell could not use goes back to the 529.
        qtp_shared = phase::floor(
            qtp_shared,
            phase::cap(phase::cap(qtp_room, [qtp_rest]), [shared - coverdell_shared]),
        );
        Covered {
            qtp: qtp_own + qtp_shared,
            qtp_loans,
            coverdell: coverdell_own + coverdell_shared,
        }
    }
}

// One distribution that counts expenses.
struct Item {
    entry: usize,
    owner: usize,
    qtp: bool,
    // Distribution in cents (box 1 less the part put back).
    amount: i64,
    // Earnings in that distribution, in cents.
    earnings: i64,
    reason: DistributionReason,
}

// The expenses for one student ("group") against that student's
// distributions: the adjusted qualified expenses each one is matched with
// and the loan part of them.
fn spread(items: &[&Item], covered: Covered) -> Vec<(i64, i64)> {
    let qtp: i64 = items.iter().filter(|i| i.qtp).map(|i| i.amount).sum();
    let coverdell: i64 = items.iter().filter(|i| !i.qtp).map(|i| i.amount).sum();
    items
        .iter()
        .map(|item| {
            let (total, whole) =
                if item.qtp { (covered.qtp, qtp) } else { (covered.coverdell, coverdell) };
            let share = phase::cap(fraction(total, item.amount, whole), [item.amount]);
            let loans = if item.qtp && covered.qtp > 0 {
                fraction(covered.qtp_loans, item.amount, whole)
            } else {
                0
            };
            (share, phase::cap(loans, [share]))
        })
        .collect()
}

// Earnings the tax-free expenses leave taxable (a loss is never taxable).
fn taxable(earnings: i64, amount: i64, expenses: i64) -> i64 {
    if earnings <= 0 || amount <= 0 {
        return 0;
    }
    earnings - fraction(earnings, phase::cap(expenses, [amount]), amount)
}

fn person_of(owner: usize) -> i64 {
    if owner == 0 { -1 } else { -2 }
}

// The education entry of a person, if the person has one.
fn education_entry(credits: &FederalCredits, person: i64) -> Option<usize> {
    crate::savings_bonds::education_entry(credits, person)
}

// The taxable earnings of every 1099-Q on the return and the facts other
// forms read from them. Lines, issues and notes are emitted here.
pub(crate) fn calculate(cx: &EducationContext, r: &mut FederalResult) -> EducationTotals {
    let mut totals = EducationTotals::default();
    let corrections = crate::coverdell::unlinked_earnings(cx.coverdell, r);
    totals.correction_income = corrections;
    totals.income = corrections;
    totals.exceptions = corrections;
    let used = cx.credits.education_expenses_used(cx.students, cx.credit_allowed);
    let record = expenses_record(cx.income);
    let joint = cx.status == FederalStatus::Joint;
    let on_return =
        |person: i64| person == -1 || (person == -2 && joint) || cx.dependent_ids.contains(&person);
    // Each student's pools, once, and the issues of the expense record.
    let mut pools: Vec<(i64, Pools)> = Vec::new();
    if let Some((entry, data)) = record {
        let context = crate::interview::income_issue_context(entry);
        for student in &data.students {
            let alternatives =
                context.clone().alternatives(&[TaxSection::OtherIncome, TaxSection::Education]);
            crate::require_localized!(
                r,
                on_return(student.person),
                &context.clone().alternatives(&[TaxSection::OtherIncome, TaxSection::People]),
                "529 and Coverdell expenses",
                "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D",
                "The selected student must be a filer or an eligible dependent on this return.",
            );
            let credit_entry = education_entry(cx.credits, student.person);
            let mut pool = Pools {
                other_college: student.other_college,
                k12: student.k12,
                k12_coverdell: student.k12_coverdell,
                apprenticeship: student.apprenticeship,
                elsewhere: student.used_elsewhere,
                academy: student.academy_costs,
                loans: phase::cap(
                    student.loan_payments,
                    [positive(QTP_LOAN_LIFETIME_LIMIT_CENTS - student.loan_payments_before)],
                ),
                ..Pools::default()
            };
            match (&student.college, credit_entry) {
                (CollegeCosts::EducationCredit, Some(index)) => {
                    let e = &cx.credits.education_inputs()[index].expenses;
                    pool.college = e.tuition + e.school_materials + e.other_materials;
                    pool.assistance = e.tax_free_assistance() + e.refunds;
                    pool.credit = used[index].used * 100;
                }
                (CollegeCosts::EducationCredit, None) => {
                    crate::require_localized!(
                        r,
                        false,
                        &alternatives,
                        "529 and Coverdell expenses",
                        "ISSUE_EDUCATION_ACCOUNT_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY",
                        "A 529 or Coverdell student's college costs come from an education credit entry, but this person has none. Add the entry on Education credits or enter the costs on the 529 and Coverdell expenses record.",
                    );
                }
                (CollegeCosts::Entered { tuition_books, assistance, credit_elsewhere }, entry) => {
                    crate::require_localized!(
                        r,
                        entry.is_none(),
                        &alternatives,
                        "529 and Coverdell expenses",
                        "ISSUE_EDUCATION_ACCOUNT_EXPENSES_ENTERED_TWICE",
                        "This person's college costs are entered on Education credits and again on the 529 and Coverdell expenses record. Enter them once: choose the education credit entry here, or remove the credit entry.",
                    );
                    pool.college = *tuition_books;
                    pool.assistance = *assistance;
                    pool.credit =
                        credit_elsewhere + entry.map_or(0, |index| used[index].used * 100);
                }
            }
            pools.push((student.person, pool));
        }
    }
    // The distributions, grouped by the student whose costs they share.
    let mut items: Vec<Item> = Vec::new();
    let mut groups: Vec<(Option<i64>, Pools)> = Vec::new();
    let mut group_of: Vec<usize> = Vec::new();
    let mut basis_left: Vec<(usize, i64)> = Vec::new();
    for (entry_index, entry) in cx.income.iter().enumerate() {
        let FederalIncome::EducationDistribution { data } = &entry.income else {
            continue;
        };
        let data = &FederalEducationDistribution::from(data);
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let context = crate::interview::income_issue_context(entry);
        let alternatives =
            context.clone().alternatives(&[TaxSection::OtherIncome, TaxSection::People]);
        if let Some(issue) = data.answer_issue() {
            crate::issue_localized!(
                r,
                &context,
                "1099-Q",
                issue.key,
                &crate::Message::new(issue.key, alloc::vec![]).english()
            );
            continue;
        }
        let beneficiary = data.effective_beneficiary().expect("established 1099-Q beneficiary");
        let person = match beneficiary {
            FederalEducationBeneficiary::Recipient => Some(person_of(owner)),
            FederalEducationBeneficiary::Student { person } => Some(*person),
            FederalEducationBeneficiary::OtherStudent { .. } => None,
        };
        if let Some(person) = person {
            crate::require_localized!(
                r,
                on_return(person),
                &alternatives,
                "1099-Q",
                "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D",
                "The selected student must be a filer or an eligible dependent on this return.",
            );
            if matches!(beneficiary, FederalEducationBeneficiary::Student { .. }) {
                crate::require_localized!(
                    r,
                    person != person_of(owner),
                    &alternatives,
                    "1099-Q",
                    "ISSUE_Q1099_STUDENT_IS_THE_RECIPIENT",
                    "This 1099-Q is addressed to the person chosen as its student. Choose that the form is addressed to the student, or choose the person who owns the account.",
                );
            }
        }
        let gross = data.gross;
        let (amount, reason) = match &data.transfer {
            Q1099Transfer::Withdrawal { reason } => (gross - data.redeposited, *reason),
            Q1099Transfer::TrusteeTransfer => {
                (data.taxable_transfer_part, DistributionReason::Withdrawal)
            }
            Q1099Transfer::ExcessThisYear => {
                // Earnings of a timely returned excess contribution are taxed
                // this year, without the additional tax (IRC 530(d)(4)(C)).
                let earnings = positive(data.account.boxes().map_or(0, |(e, _)| e));
                totals.income[owner] += earnings;
                totals.exceptions[owner] += earnings;
                totals.sources.push(EducationSource {
                    id: entry.id,
                    owner: entry.owner.owned(),
                    taxable: earnings,
                    exceptions: earnings,
                });
                line_cents(
                    r,
                    &alloc::format!("education_account_{}_taxable", entry.id),
                    earnings,
                    &["income.education_distribution"],
                    SOURCE,
                    msg!("EXPLAIN_EDUCATION_ACCOUNT_N_TAXABLE"),
                );
                continue;
            }
            Q1099Transfer::ExcessLastYear => {
                totals.sources.push(EducationSource {
                    id: entry.id,
                    owner: entry.owner.owned(),
                    taxable: 0,
                    exceptions: 0,
                });
                note_once_scoped(
                    r,
                    msg!("NOTE_COVERDELL_EXCESS_PRIOR_RETURN"),
                    context.clone().alternatives(&[TaxSection::OtherIncome]),
                );
                continue;
            }
            // The account ceased to be a Coverdell ESA on January 1 and was
            // treated as distributing all its assets that day (IRC 530(e),
            // 408(e)(2)): box 1 is figured as a withdrawal that no education
            // expenses cover, with the 10% additional tax (530(d)(4)).
            Q1099Transfer::ProhibitedTransaction => (gross, DistributionReason::Withdrawal),
            // A transfer to the beneficiary's Roth IRA is not income (section 529(c)(3)(E)). Its
            // part above the year's room, or all of it when its records are not confirmed, is
            // not a qualified rollover: a distribution from the 529 (R11, R12).
            Q1099Transfer::RothIra => {
                let rollover =
                    data.roth_rollover.as_ref().expect("established 529-to-Roth records");
                let amount = if rollover.confirmed {
                    cx.qtp_excess.get(&entry.id).copied().unwrap_or(0)
                } else {
                    gross
                };
                if amount == 0 {
                    totals.sources.push(EducationSource {
                        id: entry.id,
                        owner: entry.owner.owned(),
                        taxable: 0,
                        exceptions: 0,
                    });
                    continue;
                }
                (amount, DistributionReason::Withdrawal)
            }
        };
        if amount <= 0 {
            totals.sources.push(EducationSource {
                id: entry.id,
                owner: entry.owner.owned(),
                taxable: 0,
                exceptions: 0,
            });
            continue;
        }
        let (earnings, remaining) =
            earnings_in(data, gross, amount).expect("established Coverdell worksheet inputs");
        let covered = matches!(data.transfer, Q1099Transfer::Withdrawal { .. })
            || (matches!(data.transfer, Q1099Transfer::RothIra)
                && data.roth_rollover.as_ref().is_some_and(|rollover| !rollover.confirmed));
        // The group: the person's pools, or this form's own.
        let (key, pool) = match (beneficiary, person) {
            (_, Some(person)) => {
                (Some(person), pools.iter().find(|(p, _)| *p == person).map(|(_, p)| p.clone()))
            }
            (
                FederalEducationBeneficiary::OtherStudent { expenses, assistance, credit_expenses },
                None,
            ) => (
                None,
                Some(Pools {
                    college: *expenses,
                    assistance: *assistance,
                    credit: *credit_expenses,
                    ..Pools::default()
                }),
            ),
            _ => (None, None),
        };
        if pool.is_none() && covered {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_EDUCATION_ACCOUNT_EXPENSES_ASSUMED_NONE",
                    alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                ),
                context.clone().alternatives(&[TaxSection::OtherIncome]),
            );
        }
        let pool = pool.unwrap_or_default();
        // An uncovered part (a transfer's non-qualifying part) joins no
        // student's costs: its group is its own, with no expenses.
        let group = if !covered {
            groups.push((None, Pools::default()));
            groups.len() - 1
        } else if let Some(person) = key {
            match groups.iter().position(|(p, _)| *p == Some(person)) {
                Some(index) => index,
                None => {
                    groups.push((Some(person), pool));
                    groups.len() - 1
                }
            }
        } else {
            groups.push((None, pool));
            groups.len() - 1
        };
        group_of.push(group);
        if let Some(left) = remaining {
            basis_left.push((entry_index, left));
        }
        items.push(Item {
            entry: entry_index,
            owner,
            qtp: data.account.is_qtp(),
            amount,
            earnings,
            reason,
        });
    }
    for (group, (person, pool)) in groups.iter().enumerate() {
        let members: Vec<&Item> = items
            .iter()
            .zip(&group_of)
            .filter(|(_, g)| **g == group)
            .map(|(item, _)| item)
            .collect();
        let qtp: i64 = members.iter().filter(|i| i.qtp).map(|i| i.amount).sum();
        let coverdell: i64 = members.iter().filter(|i| !i.qtp).map(|i| i.amount).sum();
        let with = spread(&members, pool.cover(true, qtp, coverdell));
        let without = spread(&members, pool.cover(false, qtp, coverdell));
        let whole: i64 = members.iter().map(|i| i.amount).sum();
        for (index, item) in members.iter().enumerate() {
            let entry = &cx.income[item.entry];
            let (covered_amount, loans) = with[index];
            let taxed = taxable(item.earnings, item.amount, covered_amount);
            let taxed_without = taxable(item.earnings, item.amount, without[index].0);
            let free = positive(item.earnings) - taxed;
            // Pub 970 / Form 5329 line 6: the exceptions to the 10% tax.
            let exception = if taxed == 0 {
                0
            } else if item.reason != DistributionReason::Withdrawal {
                taxed
            } else {
                let credit_part = taxed - taxed_without;
                let short = item.amount - phase::cap(without[index].0, [item.amount]);
                let help = fraction(pool.assistance + pool.academy, item.amount, whole.max(1));
                let scholarship = if short > 0 && taxed_without > 0 {
                    fraction(taxed_without, phase::cap(help, [short]), short)
                } else {
                    0
                };
                phase::cap(credit_part + scholarship, [taxed])
            };
            totals.sources.push(EducationSource {
                id: entry.id,
                owner: entry.owner.owned(),
                taxable: taxed,
                exceptions: exception,
            });
            totals.income[item.owner] += taxed;
            totals.exceptions[item.owner] += exception;
            totals.excluded_earnings[item.owner] += free;
            if let Some(person) = person {
                match totals.expenses_used.iter_mut().find(|(id, _)| id == person) {
                    Some((_, cents)) => *cents += phase::cap(covered_amount, [item.amount]),
                    None => totals
                        .expenses_used
                        .push((*person, phase::cap(covered_amount, [item.amount]))),
                }
                if (*person == -1 || (*person == -2 && joint)) && covered_amount > 0 && loans > 0 {
                    totals.loan_excluded += fraction(free, loans, covered_amount);
                }
            }
            let id = entry.id;
            let inputs: &[&str] = &[
                "income.education_distribution",
                "income.education_account_expenses",
                "credits.education",
            ];
            emit_lines(
                r,
                &mut (),
                [
                    ("distribution", item.amount, msg!("EXPLAIN_EDUCATION_ACCOUNT_N_DISTRIBUTION")),
                    ("earnings", item.earnings, msg!("EXPLAIN_EDUCATION_ACCOUNT_N_EARNINGS")),
                    (
                        "adjusted_expenses",
                        phase::cap(covered_amount, [item.amount]),
                        msg!("EXPLAIN_EDUCATION_ACCOUNT_N_ADJUSTED_EXPENSES"),
                    ),
                    ("taxable", taxed, msg!("EXPLAIN_EDUCATION_ACCOUNT_N_TAXABLE")),
                    ("exceptions", exception, msg!("EXPLAIN_EDUCATION_ACCOUNT_N_EXCEPTIONS")),
                ],
                |(suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("education_account_{id}_{suffix}"),
                        LineValue::MoneyCents(amount),
                        inputs,
                        SOURCE,
                        explanation,
                    ))
                },
            );
            if let Some((_, left)) = basis_left.iter().find(|(e, _)| *e == item.entry) {
                line_cents(
                    r,
                    &alloc::format!("education_account_{id}_basis_remaining"),
                    *left,
                    inputs,
                    SOURCE,
                    msg!("EXPLAIN_EDUCATION_ACCOUNT_N_BASIS_REMAINING"),
                );
            }
        }
    }
    if cx.income.iter().any(|e| matches!(e.income, FederalIncome::EducationDistribution { .. }))
        || cx.coverdell.iter().any(|v| !v.corrections.is_empty())
    {
        line_cents(
            r,
            "education_account_taxable_earnings",
            totals.income.iter().sum(),
            &["income.education_distribution", "coverdell_excess.corrections"],
            SOURCE,
            msg!("EXPLAIN_EDUCATION_ACCOUNT_TAXABLE_EARNINGS"),
        );
    }
    totals
}

// The note for a student whose education credit computes to zero but still
// holds expenses the credit would use: the 529 or Coverdell withdrawals cannot
// reuse them. Called once the credits are final; `aotc` and `llc` are the
// credits after the Form 8863 income phaseout.
pub(crate) fn note_credit_expenses(
    cx: &EducationContext,
    aotc: i64,
    llc: i64,
    r: &mut FederalResult,
) {
    let Some((entry, data)) = expenses_record(cx.income) else {
        return;
    };
    let used = cx.credits.education_expenses_used(cx.students, cx.credit_allowed);
    let wasted = data.students.iter().any(|student| {
        matches!(student.college, CollegeCosts::EducationCredit)
            && education_entry(cx.credits, student.person).is_some_and(|index| {
                let credit = match used[index].method {
                    crate::EducationMethod::AmericanOpportunity => aotc,
                    crate::EducationMethod::LifetimeLearning => llc,
                };
                used[index].used > 0 && credit == 0
            })
    });
    if wasted {
        note_once_scoped(
            r,
            msg!("NOTE_EDUCATION_ACCOUNT_CREDIT_EXPENSES"),
            crate::interview::income_issue_context(entry)
                .alternatives(&[TaxSection::OtherIncome, TaxSection::Education]),
        );
    }
}
