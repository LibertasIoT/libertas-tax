// Coverage: see charity.rs (Unit15; original T3-deduct, 2026-09-28): hosted students are asked on the itemized
//   hub's gifts of property and carryovers page.
// Section 170(g): 2026 qualifying student-hosting expenses.
// https://www.irs.gov/publications/p526
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// One month of actual student-hosting expenses in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HostedStudentMonth {
    /// 2026 calendar month, January = 1
    // Tax structure: calendar months are numbered 1 through 12.
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Days in this month when residence, written agreement and full-time school conditions all held
    /// Fifteen qualifying days make the month eligible for the $50 ceiling.
    // Tax structure: a calendar month has at most 31 qualifying days.
    #[libertas_number(min = 0, max = 31)]
    pub qualifying_days: u32,
    /// Actual unreimbursed expense paid in this month for the student's well-being
    /// Exclude imputed lodging, home depreciation, general taxes, insurance and repairs.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
}
impl HostedStudentMonth {
    fn valid(&self, joined: u32, school_start: u32, school_end: u32) -> bool {
        if !(1..=12).contains(&self.month) || self.cost < 0 {
            return false;
        }
        let first = 20260000 + self.month * 100 + 1;
        let last_day =
            (28..=31).rev().find(|day| valid_date(20260000 + self.month * 100 + day)).unwrap();
        let last = 20260000 + self.month * 100 + last_day;
        let start = first.max(joined).max(school_start);
        let end = last.min(school_end);
        let possible_days = if start <= end { end % 100 - start % 100 + 1 } else { 0 };
        self.qualifying_days <= possible_days
    }
    pub(crate) fn allowed(&self) -> i64 {
        // The statutory monthly ceiling bounds every admitted nonnegative raw
        // cost before any consumer sums the at-most-twelve monthly allowances.
        if self.qualifying_days >= 15 { phase::cap(self.cost, [5_000]) } else { 0 }
    }
}

/// Student living in the taxpayer's home under a qualifying education program
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HostedStudent {
    /// Distinct local student label for this return
    #[libertas_size(min = 1)]
    pub label: String,
    /// Eligible sponsoring charity, veterans' organization or fraternal society; cemetery companies and government units do not qualify
    #[libertas_size(min = 1)]
    pub sponsor: String,
    /// Written education-program agreement reference
    #[libertas_size(min = 1)]
    pub agreement: String,
    /// Student's U.S. school
    #[libertas_size(min = 1)]
    pub school: String,
    /// School city and state
    #[libertas_size(min = 1)]
    pub school_location: String,
    /// Full-time grade, kindergarten = 0 and high-school grade 12 = 12
    // Tax structure: the section 170(g) student deduction covers kindergarten through grade 12.
    #[libertas_number(min = 0, max = 12)]
    pub grade: u32,
    /// Actual date the student joined this household, possibly before 2026
    #[libertas_date_only]
    pub joined: u32,
    /// First full-time school attendance day relevant to this 2026 claim
    #[libertas_date_only]
    pub school_start: u32,
    /// Last full-time school attendance day relevant to this 2026 claim
    #[libertas_date_only]
    pub school_end: u32,
    /// Actual 2026 monthly costs and qualifying days for this student
    /// ----
    /// Month
    // Tax structure: section 170(g) records have one unique entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    pub months: Vec<HostedStudentMonth>,
    /// Sponsor, student and reimbursement conditions confirmed with required statement records
    /// Confirm a written educational-placement agreement with a section 170(c)(2), (3) or (4) organization; the student was full time in U.S. grade K–12 and neither a relative nor a dependent under the special section 170(g) definition. No mutual exchange, ordinary compensation or reimbursement applies; any extraordinary one-time reimbursement is excluded from entered costs. Keep the agreement, actual expenses, household-joining date, attendance dates and school location for the required return statement. Amounts are neither reimbursed nor claimed elsewhere. The written placement agreement and your expense records show it. No leaves this student's deduction out of the estimate with a notice; that situation is not supported.
    pub documented: bool,
}
impl HostedStudent {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let issue = |field| ValidationIssue::new(VALIDATION_KEY).at_field(field);
        for (field, value) in [
            ("label", &self.label),
            ("sponsor", &self.sponsor),
            ("agreement", &self.agreement),
            ("school", &self.school),
            ("school_location", &self.school_location),
        ] {
            if value.trim().is_empty() {
                return Some(issue(field));
            }
        }
        if self.grade > 12 {
            return Some(issue("grade"));
        }
        if !valid_date(self.joined) || self.joined > 20261231 {
            return Some(issue("joined"));
        }
        if !(20260101..=20261231).contains(&self.school_start) || !valid_date(self.school_start) {
            return Some(issue("school_start"));
        }
        if !(20260101..=20261231).contains(&self.school_end)
            || !valid_date(self.school_end)
            || self.school_start > self.school_end
        {
            return Some(issue("school_end"));
        }
        if !(1..=12).contains(&self.months.len()) {
            return Some(issue("months"));
        }
        for (index, month) in self.months.iter().enumerate() {
            if !month.valid(self.joined, self.school_start, self.school_end) {
                let field = if !(1..=12).contains(&month.month) {
                    "month"
                } else if month.cost < 0 {
                    "cost"
                } else {
                    "qualifying_days"
                };
                return Some(issue(field).in_list("months", index));
            }
            if self.months[..index].iter().any(|prior| prior.month == month.month) {
                return Some(issue("month").in_list("months", index));
            }
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn allowed(&self) -> i64 {
        self.months.iter().map(HostedStudentMonth::allowed).sum()
    }
}

pub(crate) const VALIDATION_KEY: &str =
    "MESSAGE_CHECK_EACH_HOSTED_STUDENT_THE_RETURN_YEAR_SCHOOL_DATES_A_JOINING_DATE_ONE_ROW_PER_M";
