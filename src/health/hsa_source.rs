// HSA contribution source facts and intrinsic validation.
use crate::common::Owner;
use crate::dates::{source_valid_date as valid_date, valid_birth};
use crate::kernel::ratio::exact::{Exact, whole};
use crate::validation::ValidationIssue;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// HSA contribution eligibility on the first day of the month
/// You must have qualifying coverage, no disqualifying other coverage and no Medicare enrollment, including retroactive enrollment. People separately establishes whether someone can claim you as a dependent. Qualifying coverage includes ordinary HDHPs and eligible individual-market bronze or catastrophic plans: an Exchange plan, the same plan sold off-Exchange (including an otherwise identical plan without cost-sharing loading), or the Notice 2026-5 off-Exchange safe harbor when you have no reason to believe it is unavailable on an Exchange. SHOP bronze coverage must satisfy the ordinary HDHP rules. The recent-IHS-service restriction does not apply to the qualifying American Indian/Alaska Native bronze variant described in Notice 2026-5. An HRA that pays only premiums does not by itself disqualify you; other reimbursements can.
/// Telehealth before the deductible does not by itself disqualify an otherwise eligible HDHP; connected in-person services, equipment and drugs do not automatically receive that exception. A qualifying direct-primary-care arrangement also does not disqualify you when all its fixed periodic fees, annualized by month, total at most $150 for one person or $300 for an arrangement covering more than one person. It must provide solely qualifying primary care by a family/internal/geriatric/pediatric physician, nurse practitioner, clinical nurse specialist or physician assistant, with no separate charge for care within the arrangement. General anesthesia, non-vaccine prescription drugs and laboratory services outside an ordinary ambulatory primary-care setting disqualify that arrangement; merely declining those services does not cure its terms. Separate services available to members and nonmembers outside the arrangement are permitted.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HsaCoverage {
    /// Not eligible for HSA contributions
    None,
    /// Self-only eligible coverage
    SelfOnly,
    /// Family eligible coverage
    Family,
}

/// HSA contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalHsa {
    /// Account owner
    pub owner: Owner,
    /// Contribution eligibility
    pub coverage: HsaEligibility,
    /// Other personal contributions not excluded from wages
    /// Original principal before timely returns, excluding source-linked failed indirect-rollover deposits. The existing contribution owner adds those deposits on its calculation copy only.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub personal: i64,
    /// Personal principal outside the accepted Forms 5498-SA
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub personal_outside: i64,
    /// Actual employer and payroll-excluded contributions for 2026
    /// Use all deposits allocated to 2026, including a verified deposit omitted from W-2 box 12 code W; do not treat a reporting omission as a zero contribution. Reconcile the W-2 with employer and HSA custodian records and request a corrected W-2 when needed. The App uses actual contributions for Form 8889 limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employer principal designated for 2026 outside entered W-2 sources
    /// Includes actual next-calendar-year deposits designated for this return; never includes a source allocated above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer_outside: i64,
    /// Prior-year excess withdrawn after its correction deadline
    /// Even medical use is taxable when the excess is returned late. Enter the full distribution here, not again as a medical or nonmedical withdrawal. The App uses it to reduce any retained prior excess.
    pub late_excess: Vec<crate::HsaLateExcessWithdrawal>,
    /// Uncorrected employer excess deposited in 2026 but designated for 2025
    /// These deposits belong to the retained 2025 contribution worksheet, not the 2026 contribution totals; their taxable income follows actual deposit year.
    pub prior_employer_excess: Vec<crate::HsaPriorEmployerExcessDeposit>,
    /// Timely excess-contribution corrections
    pub corrections: Vec<crate::HsaCorrection>,
    /// 2026 and retained prior-year HSA excess accounting, if needed
    /// Leave absent when contributions fit the 2026 limit and no retained 2025 excess remains. Supply 2025 Form 5329 facts, current account values and any employer excess already included in W-2 wages; the App computes prior absorption, any deduction, current excess, other income and excise tax. A qualified IRA-to-HSA funding transfer uses its own contribution room first.
    pub excess_accounting: Option<crate::HsaExcessAccounting>,
    /// First known loss in the December 2026–December 2027 testing period
    /// Leave absent if no loss has occurred. Enter the first loss even if eligibility later returned; death or permanent and total disability must cause it for the exception. The App uses December eligibility automatically when it increases an independent contribution limit and identifies the amount requiring continued eligibility through December 31, 2027.
    pub last_month_loss: Option<crate::HsaEligibilityLoss>,
    /// Prior last-month-rule history, if used in 2025
    pub last_month_history: Option<crate::HsaLastMonthHistory>,
    /// Current qualified IRA-to-HSA funding
    // Tax structure: at most two qualified lifetime IRA-to-HSA transfers per account owner.
    #[libertas_size(max = 2)]
    pub funding: Vec<crate::HsaFunding>,
    /// Prior IRA-to-HSA funding distributions
    /// Historical 2025 transfers whose testing periods can end in 2026.
    // Tax structure: at most two qualified IRA-to-HSA transfers per account owner.
    #[libertas_size(max = 2)]
    pub funding_history: Vec<crate::HsaFundingHistory>,
    /// Qualified IRA-to-HSA funding before 2025
    /// Lifetime uses from 2007 through 2024 only. Keep 2025 transfers in the testing-period history above; neither historical source is a current contribution. A prior qualified transfer consumes lifetime use even if its testing period later failed.
    pub uses: Vec<crate::HsaFundingUseHistory>,
}

/// HSA contribution-limit method and family eligibility context
/// Choose actual monthly capacity or the December last-month method. Spouses who are both eligible share a family allowance equally unless they actually agree otherwise. Other-filer coverage is factual context when that person has no HSA entry; it creates no account or income. December requires actual December 1 eligibility and its testing period through December 31, 2027, except for death or disability.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaFamilyShare {
    /// Contribution-limit method
    #[libertas_default("Monthly")]
    pub method: HsaAllocationMethod,
    /// Taxpayer's agreed parts
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub taxpayer_parts: u32,
    /// Spouse's agreed parts
    /// One part each means equal shares. Zero for one spouse is allowed; both cannot be zero. The same ratio applies when recomputing the ordinary monthly family allowance for recapture.
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub spouse_parts: u32,
    /// Other filer's eligibility when that person has no HSA entry
    /// Supply the other filer's actual first-day monthly eligibility when only one account owner is listed, including an ineligible full year. On a separate return this is the spouse; on a joint return it is the missing account owner. This context creates no HSA account, contribution, distribution or catch-up. Leave absent when both owners have entries. Report actual Archer contributions in their existing records.
    #[libertas_default($null)]
    pub other_coverage: Option<HsaEligibility>,
}
/// HSA contribution-limit method
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HsaAllocationMethod {
    /// Actual monthly eligibility
    /// Divide the allowance for eligible family months, then add each owner's self-only months and personal catch-up. This method does not use the current last-month rule.
    Monthly,
    /// December eligibility
    /// Use December1 eligibility and coverage for the annual last-month-rule capacity, with each eligible owner's personal catch-up and the testing period. Divide family capacity only when both spouses are eligible; a self-only December plan uses its own self-only limit.
    December,
}
impl HsaFamilyShare {
    pub(crate) fn valid(&self) -> bool {
        u64::from(self.taxpayer_parts) + u64::from(self.spouse_parts) > 0
            && self.other_coverage.as_ref().is_none_or(|v| match v {
                HsaEligibility::FullYear { .. } => true,
                HsaEligibility::Monthly { months } => months.len() == 12,
            })
    }
    pub(crate) fn amount(&self, value: i64, owner: usize) -> i64 {
        let parts = if owner == 0 { self.taxpayer_parts } else { self.spouse_parts };
        crate::kernel::ratio::fraction(
            value,
            i64::from(parts),
            (i64::from(self.taxpayer_parts) + i64::from(self.spouse_parts)).max(1),
        )
    }
}

/// Retained 2025 Form 5329 HSA excess
/// Use the actual prior return after any amendment. Its tax determines whether the excess enters line 42 of the 2026 form; the App recalculates how much remains after 2026 activity.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaPriorExcess {
    /// Retained 2025 Form 5329 line 48 excess contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub line_48: i64,
    /// Retained 2025 Form 5329 line 49 tax
    /// The 2026 line 42 imports line 48 only when this tax was positive.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub line_49: i64,
}
impl HsaPriorExcess {
    pub(crate) fn valid(&self) -> bool {
        self.line_48 >= 1 && self.line_49 >= 1
    }
}

/// 2026 Form 5329 HSA excess-accounting facts
/// Use when personal or employer excess remains in 2026 or a retained 2025 excess enters this year. A qualified IRA-to-HSA funding transfer uses contribution room before employer and personal deposits. The App calculates current excess, prior absorption, deduction, income and tax; do not enter current precomputed tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaExcessAccounting {
    /// 2025 excess and tax on the retained Form 5329, if any
    /// [NullText]
    /// No 2025 HSA excess carries into 2026
    /// [SetText]
    /// The retained 2025 excess and tax are
    pub prior_year: Option<HsaPriorExcess>,
    /// Part of the uncorrected excess from employer deposits actually made in 2026 already included in 2026 Form W-2 box 1 wages
    /// Enter only the taxable 2026-deposit excess principal, not all HSA wages or timely corrected amounts. The remaining 2026-deposit excess enters Schedule 1 other income. A 2027 deposit follows 2027 income reporting. Confirm the W-2 wage source is entered once in Income.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer_excess_in_wages: i64,
    /// Fair market value on December 31, 2026 of this owner's HSAs OUTSIDE the accepted Forms5498-SA
    /// Enter actual values for every other account, or zero when all accounts are represented above. The App adds each statement's box5 once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end_value: i64,
    /// Additional contributions made after December 31, 2026 and designated for 2026
    /// Enter only net later deposits OUTSIDE the accepted Forms5498-SA, excluding failed indirect-rollover deposits entered on Other income. Include other deposits in the manual personal/employer totals as appropriate. The App adds linked cash once; Form5329 uses both net later subsets for its2026 account-value ceiling.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub later_return_year_deposits: i64,
    /// Employer part of the later deposits designated for 2026
    /// Enter only employer deposits outside the accepted Forms5498-SA. A taxable employer excess from a deposit actually made in 2027 belongs to 2027 wages or other income, although the designated deposit still affects the 2026 Form 8889 limit and Form 5329 value ceiling.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub later_employer_deposits: i64,
}

impl HsaExcessAccounting {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.later_employer_deposits > self.later_return_year_deposits).then_some(
            crate::validation::ValidationIssue::new("HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER")
                .at_field("later_employer_deposits"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        self.year_end_value >= 0
            && self.later_return_year_deposits >= 0
            && (0..=self.later_return_year_deposits).contains(&self.later_employer_deposits)
            && self.employer_excess_in_wages >= 0
            && self.prior_year.as_ref().is_none_or(HsaPriorExcess::valid)
    }
}

/// Late withdrawal of a prior-year HSA excess
/// A distribution after that contribution year's correction deadline is taxable even when the cash paid medical expenses. Enter the entire 2026 distribution once here, excluding it from medical and ordinary nonmedical withdrawals. This is not a timely contribution correction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaLateExcessWithdrawal {
    /// Year of the original excess contribution
    // Tax structure: this history concerns contributions for years before the current return.
    #[libertas_number(min = 2004, max = 2025)]
    pub contribution_year: u32,
    /// Expired correction deadline including any valid extension or relief
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20040101, max = 20261231)]
    pub deadline: u32,
    /// Date distributed in 2026
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Entire late distribution, including any earnings paid with it
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Account owner was permanently and totally disabled before distribution
    /// Age 65 is determined from the owner's birth date. Both exceptions remove the 20% tax, not ordinary income. Check the disability determination and its effective date. No applies the 20% tax before age 65.
    pub disabled: bool,
}
impl HsaLateExcessWithdrawal {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.date <= self.deadline).then_some(
            crate::validation::ValidationIssue::new("HSA_LATE_EXCESS_NOT_LATE").at_field("date"),
        )
    }
    fn record_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let invalid =
            |field| crate::validation::ValidationIssue::new("HSA_RECORD_INVALID").at_field(field);
        if !(2004..=2025).contains(&self.contribution_year) {
            Some(invalid("contribution_year"))
        } else if !valid_date(self.deadline)
            || !(self.contribution_year..=2026).contains(&(self.deadline / 10000))
        {
            Some(invalid("deadline"))
        } else if !valid_date(self.date) || self.date / 10000 != 2026 || self.date <= self.deadline
        {
            Some(invalid("date"))
        } else if self.amount < 1 {
            Some(invalid("amount"))
        } else {
            None
        }
    }
}

/// Uncorrected prior-year HSA employer excess deposited in 2026
/// This deposit was designated for 2025 and belongs on that year's Form 8889, not in current personal or employer totals. Its taxable principal enters 2026 wages or other income because the employer actually deposited it in 2026. The retained 2025 Form 8889 shows it above the allowable limit; a positive 2025 Form 5329 tax must be supplied separately as retained prior excess.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaPriorEmployerExcessDeposit {
    /// Actual 2026 HSA deposit date
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub contributed_on: u32,
    /// Last permitted date for a 2025-designated contribution
    /// Use April 15, 2026 or a documented later deadline under applicable relief; an ordinary filing extension does not extend this contribution deadline.
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub deadline: u32,
    /// Principal established as uncorrected employer excess on the retained 2025 return
    /// This is only the taxable excess part of the deposit, not its full code W amount. Do not also enter it as a 2026-designated employer contribution or as a timely correction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Taxable principal is already in 2026 W-2 box 1 wages
    /// Yes requires those wages in Income. No sends the principal to Schedule 1 other income. Ask the employer, or compare W-2 box 1 with the payroll statement.
    pub in_wages: bool,
}
impl HsaPriorEmployerExcessDeposit {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.contributed_on > self.deadline).then_some(
            crate::validation::ValidationIssue::new("HSA_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE")
                .at_field("deadline"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        valid_date(self.contributed_on)
            && self.contributed_on / 10000 == 2026
            && valid_date(self.deadline)
            && self.deadline / 10000 == 2026
            && self.contributed_on <= self.deadline
            && self.amount >= 1
    }
}
impl FederalHsa {
    pub(crate) fn valid(&self) -> bool {
        self.record_issue().is_none()
    }
    pub(crate) fn record_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let invalid = || ValidationIssue::new("HSA_RECORD_INVALID");
        for (field, amount) in [
            ("personal", self.personal),
            ("employer", self.employer),
            ("employer_outside", self.employer_outside),
        ] {
            if amount < 0 {
                return Some(invalid().at_field(field));
            }
        }
        if matches!(&self.coverage, HsaEligibility::Monthly { months } if months.len() != 12) {
            return Some(invalid().at_field("coverage"));
        }
        if self
            .last_month_loss
            .as_ref()
            .is_some_and(|v| !valid_date(v.date) || !(20261201..=20271231).contains(&v.date))
        {
            return Some(invalid().at_field("last_month_loss"));
        }
        // Section 223 permits only the self-only-to-family second qualified transfer.
        // IRC 408(d)(9): one lifetime transfer plus the same-year family-coverage exception.
        if self.funding.len() > 2 || self.funding.iter().any(|v| !v.valid()) {
            // Stored funding is flattened into the Page's separate collection;
            // keep opaque intrinsic failures unlocated here rather than invent
            // an accounts/{i}/funding path which the request does not contain.
            return Some(invalid());
        }
        if let Some(index) = self.corrections.iter().position(|v| !v.valid()) {
            return Some(invalid().at_list_item("corrections", index));
        }
        if HsaCorrection::manual_returned_for_return_year(&self.corrections, false)
            > whole(self.personal)
        {
            return Some(invalid().at_field("corrections"));
        }
        if self.corrected(true) > whole(self.employer) {
            return Some(invalid().at_field("corrections"));
        }
        if self.excess_accounting.as_ref().is_some_and(|v| {
            !v.valid()
                || whole(v.later_employer_deposits) > self.net_employer()
                || whole(v.later_return_year_deposits) - whole(v.later_employer_deposits)
                    > whole(self.personal)
                        - HsaCorrection::manual_returned_for_return_year(&self.corrections, false)
        }) {
            return Some(invalid().at_field("excess_accounting"));
        }
        if self.last_month_history.as_ref().is_some_and(|v| !v.valid()) {
            return Some(invalid().at_field("last_month_history"));
        }
        // The same statutory transfer limit applies to the prior-year testing history.
        if self.funding_history.len() > 2 {
            return Some(invalid().at_field("funding_history"));
        }
        if let Some(index) = self.funding_history.iter().position(|v| !v.valid()) {
            return Some(invalid().at_list_item("funding_history", index));
        }
        if let Some(issue) = self.uses.iter().enumerate().find_map(|(index, value)| {
            value.record_issue().map(|issue| issue.in_list("uses", index))
        }) {
            return Some(issue);
        }
        if HsaFundingUseHistory::same_month(&self.uses).is_some() {
            return Some(invalid().at_field("uses"));
        }
        if self.funding_history.len() >= 2
            && self.funding_history[0].funded / 100 == self.funding_history[1].funded / 100
        {
            return Some(invalid().at_field("funding_history"));
        }
        if let Some(issue) = self.late_excess.iter().enumerate().find_map(|(index, value)| {
            value.record_issue().map(|issue| issue.in_list("late_excess", index))
        }) {
            return Some(issue);
        }
        self.prior_employer_excess
            .iter()
            .position(|v| !v.valid())
            .map(|index| invalid().at_list_item("prior_employer_excess", index))
    }
    pub(crate) fn corrected(&self, employer: bool) -> Exact {
        HsaCorrection::returned_for_return_year(&self.corrections, employer)
    }
    pub(crate) fn net_personal(&self) -> Exact {
        whole(self.personal) - self.corrected(false)
    }
    pub(crate) fn net_employer(&self) -> Exact {
        whole(self.employer) - self.corrected(true)
    }
    pub(crate) fn month(&self, month: usize) -> HsaCoverage {
        self.coverage.month(month)
    }
}
impl HsaEligibility {
    pub(crate) fn month(&self, month: usize) -> HsaCoverage {
        match self {
            Self::FullYear { coverage } => *coverage,
            Self::Monthly { months } => months.get(month).copied().unwrap_or(HsaCoverage::None),
        }
    }
}

// Transient twelfths of cents from the existing monthly/December branches.
// Independently rounded form details can differ by a cent from their rounded
// sum; retain the original combined numerator for the actual owner capacity.

/// Prior qualified IRA-to-HSA funding transfer
/// These are 2025 qualified direct IRA-to-HSA funding distributions, not ordinary HSA rollovers. Each has its own testing period through the end of the same month in 2026. Enter up to two only for the permitted self-only-to-family coverage change.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaFundingHistory {
    /// Date the HSA received the IRA transfer in 2025
    #[libertas_date_only]
    // Tax structure: retained prior-year history ends before the current return year.
    #[libertas_number(min = 20250101, max = 20251231)]
    pub funded: u32,
    /// Qualified funding amount transferred
    /// Use the original qualified transfer, not current account value or a precomputed recapture tax. Do not include this historical amount in current contributions or withdrawals.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// First loss of HSA eligibility after funding, if any
    /// Absent means eligibility continued through the testing period. Use the first date you failed to be an eligible individual, including disqualifying insurance, Medicare or dependent status. A later restoration of eligibility does not undo recapture.
    /// [NullText]
    /// HSA eligibility continued through the funding testing period
    /// [SetText]
    /// The first loss of HSA eligibility was
    pub loss: Option<HsaEligibilityLoss>,
}
/// First loss of HSA eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaEligibilityLoss {
    /// First ineligible date
    #[libertas_date_only]
    pub date: u32,
    /// Cause of eligibility loss
    pub reason: HsaLossReason,
}
/// Cause of HSA eligibility loss
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HsaLossReason {
    /// Insurance, Medicare enrollment, dependent status or another nonexcepted cause
    Other,
    /// Permanent and total disability caused the loss
    Disability,
    /// Account owner's death
    Death,
}
impl HsaFundingHistory {
    // Two transfers in one month, retaining the actual later duplicate row.
    pub(crate) fn same_month_issue(
        list: &[HsaFundingHistory],
    ) -> Option<crate::validation::ValidationIssue> {
        list.iter().enumerate().find_map(|(index, value)| {
            list[..index].iter().any(|previous| previous.funded / 100 == value.funded / 100).then(
                || {
                    crate::validation::ValidationIssue::new("HSA_FUNDING_HISTORY_SAME_MONTH")
                        .at_field("funded")
                        .in_list("funding_history", index)
                },
            )
        })
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.loss.as_ref().is_some_and(|v| v.date < self.funded).then_some(
            crate::validation::ValidationIssue::new("HSA_FUNDING_LOSS_BEFORE_FUNDING")
                .at_field("date")
                .in_field("loss"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        valid_birth(self.funded)
            && self.funded / 10000 == 2025
            && self.amount >= 0
            && self.loss.as_ref().is_none_or(|v| valid_birth(v.date) && v.date >= self.funded)
    }
}
/// Timely HSA excess-contribution return
/// Use the custodian's completed corrective distribution (Form 1099-SA code 2), not an ordinary withdrawal, and enter it once. Split by contribution year and payer. Current-year contribution totals above include this original principal; the App subtracts it once. Exclude the corrective payment from ordinary medical/nonmedical withdrawals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaCorrection {
    /// Actual returned-excess statement, if entered on Other income
    /// The existing accepted source identity; never a list position or account number. Unlinked manual corrections retain their actual custodian facts.
    #[libertas_default($null)]
    #[libertas_number(min = 1)]
    pub document: Option<i64>,
    /// Original failed indirect-rollover deposit, if this return corrects one
    /// Choose the original Other-income statement, separately from the returned-excess statement above. The contribution date and year must match its actual failed cash deposit. Leave empty only for other manually reported contributions.
    #[libertas_default($null)]
    #[libertas_number(min = 1)]
    pub failed_deposit: Option<i64>,
    /// Year for which the contribution was designated
    // Tax structure: these corrections concern the prior or current return year (2025 or 2026).
    #[libertas_number(min = 2025, max = 2026)]
    pub year: u32,
    /// Date the employer or individual actually deposited the principal
    /// A deposit designated for a prior tax year may have been made the following year. Its Form 8889 limit uses the designated year, while employer wage and other-income timing follows the actual deposit year.
    #[libertas_date_only]
    // Tax structure: a 2025 or 2026 designated contribution may be deposited in the following calendar year.
    #[libertas_number(min = 20250101, max = 20271231)]
    pub contributed_on: u32,
    /// Principal contribution returned
    /// Original excess principal, before the custodian's gain or loss adjustment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub principal: i64,
    /// Attributable net income returned
    /// Use the custodian's allocation, including a negative amount for loss. A loss reduces the cash returned, not the principal removed from contributions. Positive earnings enter income in the withdrawal year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings: i64,
    /// Date principal and all attributable net income were returned
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub returned: u32,
    /// Applicable correction deadline
    /// Use the actual return due date including a valid extension, or the documented deadline under applicable IRS relief. Confirm entitlement to any later deadline, including timely filing and amendment requirements for six-month relief.
    #[libertas_date_only]
    #[libertas_number(min = 20250101)]
    pub deadline: u32,
    /// Original contribution source
    pub payer: HsaCorrectionPayer,
}

/// Source of corrected HSA contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaCorrectionPayer {
    /// Personal or other after-tax contribution
    Personal,
    /// Employer or payroll-excluded contribution
    Employer {
        /// Corrected principal already included in the contribution-year Form W-2 wages
        /// For 2025, No requires the principal on the corrected return for the actual deposit year as other income. For 2026, Yes requires those wages in that year's Income; No adds the principal to other income in the actual deposit year, not wages or earned-income/IRA compensation. A 2027 deposit designated for 2026 does not add 2026 income. Compare W-2 box 1 with box 12 code W, or a Form W-2c.
        in_wages: bool,
    },
}
impl HsaCorrection {
    // The principal returned for 2026 by one payer: the sum the page rules
    // HSA_CORRECTED_*_ABOVE_CONTRIBUTED and HSA_LATER_*_ABOVE_NET take.
    pub(crate) fn returned_for_return_year(corrections: &[HsaCorrection], employer: bool) -> Exact {
        corrections
            .iter()
            .filter(|v| {
                v.year == 2026 && matches!(v.payer, HsaCorrectionPayer::Employer { .. }) == employer
            })
            .map(|v| whole(v.principal))
            .sum()
    }
    pub(crate) fn manual_returned_for_return_year(
        corrections: &[HsaCorrection],
        employer: bool,
    ) -> Exact {
        corrections
            .iter()
            .filter(|c| {
                c.failed_deposit.is_none()
                    && c.year == 2026
                    && matches!(c.payer, HsaCorrectionPayer::Employer { .. }) == employer
            })
            .map(|c| whole(c.principal))
            .sum()
    }
    // The rules the page checks, checked again on save with the same keys.

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.failed_deposit.is_some() && !matches!(self.payer, HsaCorrectionPayer::Personal) {
            Some(
                crate::validation::ValidationIssue::new("HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED")
                    .at_field("payer"),
            )
        } else if self.returned < self.contributed_on {
            Some(
                crate::validation::ValidationIssue::new("HSA_CORRECTION_RETURNED_BEFORE_DEPOSIT")
                    .at_field("returned"),
            )
        } else if self.deadline < self.contributed_on {
            Some(
                crate::validation::ValidationIssue::new("HSA_CORRECTION_DEADLINE_BEFORE_DEPOSIT")
                    .at_field("deadline"),
            )
        } else if whole(self.earnings) + whole(self.principal) < whole(0) {
            Some(
                crate::validation::ValidationIssue::new("HSA_CORRECTION_LOSS_ABOVE_PRINCIPAL")
                    .at_field("earnings"),
            )
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.document.is_none_or(|id| id > 0)
            && self
                .failed_deposit
                .is_none_or(|id| id > 0 && matches!(self.payer, HsaCorrectionPayer::Personal))
            && (2025..=2026).contains(&self.year)
            && valid_date(self.contributed_on)
            && (self.year..=self.year + 1).contains(&(self.contributed_on / 10000))
            && self.contributed_on <= self.returned
            && self.contributed_on <= self.deadline
            && self.principal >= 0
            && self.earnings >= -self.principal
            && valid_date(self.returned)
            && valid_date(self.deadline)
            && self.returned / 10000 >= self.year
            && self.returned / 10000 <= 2027
            && self.deadline / 10000 >= self.year
    }
    pub(crate) fn eligible(&self) -> bool {
        self.returned <= self.deadline
    }
}
/// IRA-to-HSA funding before 2025
/// Enter every actual qualified funding transfer made from 2007 through 2024, including a transfer whose testing period later failed. These facts establish lifetime use, not current contributions or recapture income. Enter 2025 transfers only in the existing 2025 testing-period history; do not repeat them here. Ordinary IRA rollovers, HSA-to-HSA rollovers and trustee transfers between HSAs are not qualified IRA-to-HSA funding.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaFundingUseHistory {
    /// Actual direct funding date
    #[libertas_date_only]
    // Tax structure: earlier funding history excludes the separately retained 2025 testing period.
    #[libertas_number(min = 20070101, max = $HEALTH_EARLIER_FUNDING_END)]
    pub funded: u32,
    /// Eligible coverage at that transfer
    pub coverage: HsaCoverage,
    /// Original qualified transfer amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}
impl HsaFundingUseHistory {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.coverage == HsaCoverage::None).then_some(
            crate::validation::ValidationIssue::new("HSA_FUNDING_NEEDS_COVERAGE")
                .at_field("coverage"),
        )
    }
    #[cfg(test)]
    pub(crate) fn valid(&self) -> bool {
        self.record_issue().is_none()
    }
    pub(crate) fn record_issue(&self) -> Option<ValidationIssue> {
        let invalid = |field| ValidationIssue::new("HSA_RECORD_INVALID").at_field(field);
        if !valid_date(self.funded)
            || !(20070101..=crate::year_constants::HEALTH_EARLIER_FUNDING_END)
                .contains(&self.funded)
        {
            Some(invalid("funded"))
        } else if self.coverage == HsaCoverage::None {
            Some(invalid("coverage"))
        } else if self.amount <= 0 {
            Some(invalid("amount"))
        } else {
            None
        }
    }
    pub(crate) fn same_month(list: &[Self]) -> Option<&'static str> {
        Self::same_month_issue(list).map(|issue| issue.key)
    }
    pub(crate) fn same_month_issue(list: &[Self]) -> Option<ValidationIssue> {
        list.iter().enumerate().find_map(|(index, value)| {
            list[..index].iter().any(|previous| previous.funded / 100 == value.funded / 100).then(
                || {
                    ValidationIssue::new("HSA_FUNDING_HISTORY_SAME_MONTH")
                        .at_field("funded")
                        .in_list("uses", index)
                },
            )
        })
    }
}

/// Qualified 2026 IRA-to-HSA transfer
/// A completed direct transfer from the owner's traditional or Roth IRA to their own HSA. Enter its Form 1099-R once in Income; link that entry below. This transfer is not a personal contribution, deduction or ordinary rollover. Prior-basis funding uses source balances and the owner-level IRA worksheet. Inherited sources use only the linked beneficiary/deceased-owner group. Dated nonqualified Roth withdrawals can be coordinated from the applicable opening history; other earlier basis activity needs further support.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaFunding {
    /// Actual source Form 1099-R income entry ID
    #[libertas_number(min = 1)]
    pub income: i64,
    /// Was the funding IRA an ongoing SEP or SIMPLE IRA?
    /// Yes if an employer contribution was made for that account's plan year ending with or within the distribution tax year, including a contribution made later for that plan year. Check the actual funding account, not other IRAs or HSA employer contributions.
    pub ongoing_sep_simple: bool,
    /// HSA receipt date in 2026
    #[libertas_date_only]
    pub funded: u32,
    /// Amount actually transferred directly
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// IRA basis treatment
    pub basis: HsaFundingBasis,
    /// Eligible HDHP coverage at the time of transfer
    /// This can differ from coverage on the first day of that month. No disqualifying insurance, Medicare enrollment or dependent status at transfer.
    pub coverage: HsaCoverage,
    /// First known loss of eligibility after transfer, if any
    /// Enter the first loss even if coverage later returned. The testing period ends on the last day of the same month in 2027. Absence means no known loss; retain coverage records until the period ends.
    pub loss: Option<HsaEligibilityLoss>,
}

/// IRA basis at qualified funding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaFundingBasis {
    /// Wholly pretax traditional IRAs
    /// The applicable personal traditional IRA pool or linked inherited group has no after-tax basis at transfer. Personal income uses the payer amount equal to gross; inherited income confirms no allocated basis. Never combine inherited groups with personal accounts.
    Pretax {
        /// Actual IRA distribution date in 2026
        /// Use the funding IRA's transaction records, not the HSA receipt date or a contribution designation for another year.
        #[libertas_date_only]
        distributed: u32,
    },
    /// Roth IRA
    /// Link the Roth withdrawal that funded the transfer and supply its complete opening Roth history. Taxable earnings are used first; any remaining transfer consumes regular and conversion basis in their normal order without early-distribution tax.
    Roth {
        /// Combined Roth IRA value for the applicable group immediately before the source distribution
        /// For personal sources include all this owner's personal Roth IRAs. For inherited sources include only this beneficiary's Roth IRAs from the linked deceased owner. Use contemporaneous source statements; a second transfer uses its own value after the first transfer and intervening market changes.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
        /// All qualified Roth funding preceded other current-year Roth basis activity
        /// No regular contribution, conversion, rollover, ordinary withdrawal or other Roth basis adjustment before or between the funding transfers. Later contributions, conversions and withdrawals use their normal entries. Market changes alone are allowed. The transaction dates on the account statements show it. With No, dated nonqualified own-account ordinary withdrawals require confirmed opening history and no current-year incoming basis. Retained regular contributions also qualify when every transfer’s recorded Roth value covers its amount plus all opening basis and annual regular contributions. Current conversions, 529 transfers, corrections and other incoming basis need further support. A nonqualified inherited Roth can also coordinate ordinary withdrawals from the same deceased-owner group using its allocated opening regular and conversion basis. Ordinary inherited payments on the same date are allowed when no funding payment shares it. Personal contributions and other inherited groups stay separate. Enter each actual withdrawal separately. Same-day funding requires a unique actual sequence from1 through100 on every payment in that Roth pool that day, entered with the Form1099-R source. Qualified payments or a source combining ordinary and funding amounts need further support; other earlier-activity cases leave the exclusion out with a notice. A funding-first answer cannot override an entered earlier ordinary withdrawal date in that same Roth group.
        before_other_activity: bool,
    },
    /// Traditional IRA basis at funding
    /// For personal sources, use the owner-level IRA worksheet and prior filed basis in Adjustments. For inherited sources, use allocated basis in the linked annual income entry. Funding uses taxable money first; basis consumed by the transfer does not carry into the HSA.
    PriorBasis {
        /// Actual IRA distribution date in 2026
        /// Use the funding IRA's transaction records, not the HSA receipt date or a contribution designation for another year.
        #[libertas_date_only]
        distributed: u32,
        /// Combined traditional, SEP and SIMPLE IRA value immediately before the IRA source distribution
        /// Use actual dated account statements for all this owner's accounts, not just the source account. Exclude Roth and spouse accounts; personal and each inherited deceased-owner group remain separate. For a second transfer use its own contemporaneous value after the first transfer and intervening market changes.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
        /// All qualified funding preceded other current-year IRA basis activity
        /// Confirm no regular IRA contribution, reservist repayment, other distribution, QCD, conversion, incoming after-tax rollover or other basis adjustment occurred before or between these funding transfers. Enter later activity normally; market value changes alone are allowed. With No, one separate personal traditional funding payment can still qualify when its actual source-event value covers the transfer plus all opening basis, with no current contributions, incoming basis, corrections, QCD or other basis changes. This proves no funding basis use independently of ordinary withdrawal order. Other earlier activity leaves the exclusion out with a notice; never infer an IRA source date from the HSA receipt date.
        before_other_activity: bool,
    },
}
impl HsaFundingBasis {
    pub(crate) fn traditional_date(&self) -> Option<u32> {
        match self {
            Self::Pretax { distributed } | Self::PriorBasis { distributed, .. } => {
                Some(*distributed)
            }
            Self::Roth { .. } => None,
        }
    }
}

impl HsaFunding {
    // Context-free support. The Roth owner also admits a bounded earlier
    // ordinary-withdrawal path from the existing accepted source facts.
    pub(crate) fn supported(&self) -> bool {
        match self.basis {
            HsaFundingBasis::Pretax { .. } => true,
            // Notice 2008-51: a QHFD comes first from includible amounts; Form 8606
            // line 7 excludes it, so other same-year activity cannot change it.
            HsaFundingBasis::PriorBasis { .. } => true,
            HsaFundingBasis::Roth { before_other_activity, .. } => before_other_activity,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.income > 0
            && !self.ongoing_sep_simple
            && valid_date(self.funded)
            && self.funded / 10000 == 2026
            // Arithmetic exclusion: calculate still adds both transfers in native
            // cumulative/excluded/basis pools before reporting its statutory limit.
            && (1..=100000000000).contains(&self.amount)
            && self.coverage != HsaCoverage::None
            && match self.basis {
                HsaFundingBasis::Pretax { distributed } => {
                    valid_date(distributed)
                        && distributed / 10000 == crate::year_constants::TAX_YEAR
                        && distributed <= self.funded
                }
                HsaFundingBasis::PriorBasis { distributed, value, .. } => {
                    valid_date(distributed)
                        && distributed / 10000 == crate::year_constants::TAX_YEAR
                        && distributed <= self.funded
                        // Receiver exclusion: the retirement owner
                        // independently admits this same bounded source value. Its owner
                        // and native Roth/opening-basis consumers must migrate together.
                        && (0..=100000000000).contains(&value)
                }
                // Same retirement-owner native value/opening-basis receiver as PriorBasis.
                HsaFundingBasis::Roth { value, .. } => (0..=100000000000).contains(&value),
            }
            && self.loss.as_ref().is_none_or(|v| {
                valid_date(v.date) && v.date >= self.funded && v.date / 10000 <= 2027
            })
    }
    pub(crate) fn consistent_loss(
        &self,
        start: u32,
        end_month: u32,
        loss: Option<&HsaEligibilityLoss>,
    ) -> bool {
        let start = start.max(self.funded);
        if self.loss.as_ref().is_some_and(|v| v.date < start)
            || loss.is_some_and(|v| v.date < start)
        {
            return true;
        }
        let end = end_month.min(self.funded / 100 + 100);
        self.loss.as_ref().filter(|v| v.date / 100 <= end) == loss.filter(|v| v.date / 100 <= end)
    }
}
/// Prior HSA last-month-rule history
/// Use the owner's 2025 source records. These historical contributions are not 2026 contributions or withdrawals. The App recomputes the ordinary monthly limit to determine the amount attributable to the last-month rule.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HsaLastMonthHistory {
    /// January through December 2025 eligibility
    /// Actual coverage on each month's first day, before the last-month rule. None includes Medicare, disqualifying coverage or dependent status. December must be eligible.
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 12, max = 12)]
    pub months: Vec<HsaCoverage>,
    /// Total 2025 HSA contributions retained
    /// Personal plus employer contributions deducted or excluded under the last-month rule, after timely corrections. Exclude rollovers and IRA funding. Use original contribution records, not a calculated recapture amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
    /// 2025 Archer MSA contributions
    /// Total personal and employer deposits for this owner. These reduce the ordinary HSA contribution limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub archer: i64,
    /// Married family-limit allocation, if applicable
    /// Required when either spouse had family coverage in 2025. Actual monthly coverage controls the shared allowance; December self-only coverage uses the owner’s independent last-month allowance.
    /// [NullText]
    /// The married family-coverage allocation does not apply
    /// [SetText]
    /// The married family-limit allocation is
    pub family: Option<HsaFamilyAllocation>,
    /// First loss of eligibility during or after December 2025, if any
    /// Absent means eligibility continued through December 31, 2026. A later restoration does not erase the first loss. Death or permanent and total disability must cause the loss to qualify for the exception.
    /// [NullText]
    /// HSA eligibility continued through December 31, 2026
    /// [SetText]
    /// The first loss of HSA eligibility was
    pub loss: Option<HsaEligibilityLoss>,
}
impl HsaLastMonthHistory {
    // The page rule, checked again on save with the same key: the owner was
    // eligible in December, the month the last-month rule depends on.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.months.get(11) == Some(&HsaCoverage::None)).then(|| {
            crate::validation::ValidationIssue::new("HSA_LAST_MONTH_DECEMBER_NEEDED")
                .at_list_item("months", 11)
        })
    }
    pub(crate) fn valid(&self) -> bool {
        self.months.len() == 12
            && self.issue().is_none()
            // Arithmetic exclusion: recapture enters the federal before_rental,
            // unearned and Olympic AGI-to-Cents receiver chain before final AGI.
            && (0..=100000000000).contains(&self.contributions)
            && self.archer >= 0
            && self.family.as_ref().is_none_or(|v| {
                v.valid()
                    && (self.months.contains(&HsaCoverage::Family)
                        || v.spouse_months.contains(&HsaCoverage::Family))
            })
            && self.loss.as_ref().is_none_or(|v| valid_birth(v.date) && v.date >= 20251201)
    }
}
impl HsaLastMonthHistory {
    pub(crate) fn consistent_loss(&self, transfer: &HsaFundingHistory) -> bool {
        // Earlier loss followed by restored eligibility can precede the other
        // testing period. Otherwise both histories describe the same first loss
        // within their overlap. None confirms only through that period's end.
        let start = transfer.funded.max(20251201);
        if self.loss.as_ref().is_some_and(|v| v.date < start)
            || transfer.loss.as_ref().is_some_and(|v| v.date < start)
        {
            return true;
        }
        let end_month = transfer.funded / 100 + 100;
        self.loss.as_ref().filter(|v| v.date / 100 <= end_month)
            == transfer.loss.as_ref().filter(|v| v.date / 100 <= end_month)
    }
}
/// Agreed share of the historical family HSA allowance
/// Describe the original agreement, not a computed contribution limit or recapture. Equal division is one part each; any agreed ratio, including zero for one spouse, is allowed. Each owner's age-55 catch-up remains separate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaFamilyAllocation {
    /// Spouse's actual January–December eligibility
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 12, max = 12)]
    pub spouse_months: Vec<HsaCoverage>,
    /// Spouse's 2025 Archer contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub spouse_archer: i64,
    /// This owner's agreed parts
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub owner_parts: u32,
    /// Spouse's agreed parts
    /// At least one spouse must have a nonzero share. For two histories on this return, enter reciprocal shares and matching coverage/Archer facts.
    #[libertas_default(1)]
    #[libertas_number(min = 0)]
    pub spouse_parts: u32,
}
impl HsaFamilyAllocation {
    pub(crate) fn valid(&self) -> bool {
        self.spouse_months.len() == 12
            && self.spouse_archer >= 0
            && u64::from(self.owner_parts) + u64::from(self.spouse_parts) > 0
    }
}
impl HsaFamilyAllocation {
    pub(crate) fn reciprocal(&self, other: &Self) -> bool {
        u64::from(self.owner_parts) * u64::from(other.owner_parts)
            == u64::from(self.spouse_parts) * u64::from(other.spouse_parts)
    }
}

/// HSA contribution eligibility during 2026
/// Use actual first-day eligibility, including retroactive Medicare enrollment. Eligible bronze/catastrophic coverage, telehealth and qualifying direct-primary-care exceptions follow the monthly coverage choices; owning an HSA alone does not establish contribution eligibility.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaEligibility {
    /// Same eligibility all year
    FullYear { coverage: crate::HsaCoverage },
    /// Eligibility changes during the year
    Monthly {
        /// January through December, in order
        /// Use eligibility on the first day of each month. Medicare enrollment, including retroactive enrollment, makes that month ineligible.
        // Tax structure: one eligibility entry for each of the twelve calendar months.
        #[libertas_size(min = 12, max = 12)]
        months: Vec<crate::HsaCoverage>,
    },
}
