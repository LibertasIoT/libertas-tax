// Coverage: Pub.936 Table1 and Schedule A mortgage interest, tax year2026 (B10-4).
//   Home, refinance, partly home/other and other-use proceeds; qualified main/second homes,
//   boat/RV second homes (AMT dwelling add-back), and third homes (no personal-interest deduction).
//   Mixed-use principal is applied monthly to equity, grandfathered, then acquisition debt.
//   Two acquisition parts share principal pro rata. First/last assumes level reductions;
//   Monthly uses actual month-end balances; WithinLimit uses box2 as an upper bound.
//   Earlier refinance dates persist only through the original/balloon permitted term.
//   Table1 lines1,2,7,11,12,13,16 are review rows; line14 is rounded to three places.
//   Purchase points are8a; ratable/improvement points are8c, qualified-use shares only.
//   Ended amortized loans deduct the unamortized balance; premiums use the qualified-use share
//   and the existing AGI phaseout. NoForm and private sellers are8b. Box3 stays as printed.
//   Box4 same-year refunds reduce interest; deducted prior-year refunds join the owning
//   year's Pub.525 Worksheet2 in other_income.rs. No-benefit refunds do not become income.
//   Stable saved IDs link transferred-lender forms directly to one canonical balance ledger;
//   home/use/balance facts are asked only on the first lender. Reorder/rename does not relink.
//   The pure per-document projection exposes raw/acquisition-qualified/limited regular and
//   AMT interest, personal points, separate rental OID amortization and premiums. IRS Pub.527
//   de-minimis OID may use straight line; other rental points require an annual OID schedule
//   amount. Points paid and annual OID deduction are distinct facts; zero is an explicit amount.
//   Mortgage records survive Standard/Automatic/Itemized method changes.
//   Form1098 imports boxes1–6; blank4/5 is zero, blank6 selects no points. Box10 is Taxes5b.
//   Excludes binding-contract date exceptions and exact draw-ledger tracing when principal
//   was repaid between different-use credit-line draws (the input help explains the totals rule).
//   Sources checked2026-10-10: IRS Pub.527 (2025), IRS Pub.936 (2026 draft,Sep30), Schedule A (2026 draft,May12),
//   Form1098 (Rev.Apr2025), Pub.525 (2025), IRC163(h)(3)(B),(D),(F), IRC56(e).
//   Fixtures: schedule_a_specials A/B/C/G, its A–H footing assertions, and the existing
//   deductions/page suites for monthly allocation, points, payoff and document import.

use crate::FederalResult;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::year_constants::{
    MORTGAGE_INSURANCE_PHASEOUT_START_DOLLARS, MORTGAGE_INSURANCE_PHASEOUT_START_SEPARATE_DOLLARS,
    MORTGAGE_INSURANCE_PHASEOUT_STEP_DOLLARS, MORTGAGE_INSURANCE_PHASEOUT_STEP_SEPARATE_DOLLARS,
    MORTGAGE_INSURANCE_PHASEOUT_STEPS,
};
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::*;

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/p936--dft.pdf";

/// Home mortgage
/// Enter one for each Form 1098 or loan secured by your main home, your second home or another home.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageLoan {
    /// Loan record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Property label or address
    /// Optional manual label; it is not imported from Form 1098 box 8.
    pub property: String,
    /// Lender
    pub lender: MortgageLender,
    /// Mortgage interest paid in 2026 (Form 1098 box 1)
    /// Exclude points, mortgage insurance premiums and interest paid for a later year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub interest: i64,
    /// Outstanding mortgage principal (Form 1098 box 2)
    /// The balance on January 1, 2026, or on the date the loan began if later.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub outstanding_principal: i64,
    /// Date of this loan (Form 1098 box 3)
    #[libertas_date_only]
    pub incurred: u32,
    /// Loan facts or the earlier lender
    pub details: MortgageLoanDetails,
    /// Refund of overpaid interest (Form 1098 box 4)
    pub refund_of_interest: MortgageInterestRefund,
    /// Mortgage insurance premiums paid in 2026 (Form 1098 box 5)
    /// Deductible as home mortgage interest on Schedule A line 8d for 2026, reduced when your adjusted gross income is over $100,000 ($50,000 if married filing separately) and none above $109,000 ($54,500). Leave at zero when the contract was issued before 2007.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub mortgage_insurance_premiums: i64,
    /// Points paid on this loan (Form 1098 box 6)
    pub points: MortgagePoints,
}
// Removed: `acquisition_debt` (now `purpose`), `box_10_other_amount` (never written by the page;
// box 10 holds escrowed taxes entered on Taxes you paid). `incurred` is box 3 as printed; a
// refinance keeps its earlier date in `MortgageRefinance`.

/// Loan facts or the earlier lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageLoanDetails {
    /// First lender's entry for this loan
    FirstLender(MortgageLedger),
    /// Same loan as an earlier lender's entry
    EarlierLender {
        /// Earlier lender's loan identity
        #[libertas_number(min = 1)]
        loan: i64,
    },
}
/// The loan's year-wide facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageLedger {
    /// Home securing the loan
    pub home: MortgageHome,
    /// How the loan was used
    pub purpose: MortgageUse,
    /// Average balance for 2026
    pub average: MortgageAverageBalance,
}

/// How the loan was used
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageUse {
    /// To buy, build or substantially improve this home
    /// Every dollar went to buying, building or substantially improving the home that secures it. A line of credit opened before December 16, 2017 and drawn later dates from the draw: enter that date in box 3.
    Home,
    /// To refinance an earlier loan on this home
    /// Paid off a loan that bought, built or improved this home, with or without more money borrowed.
    Refinance(MortgageRefinance),
    /// Partly for this home, partly for other things
    /// A home equity loan or line of credit, or a loan secured by more than one property (Form 1098 box 9), used partly to buy, build or substantially improve this home.
    Partly(MortgageSplit),
    /// For other things only
    /// A home equity loan or line of credit used for a car, tuition, debts or anything other than this home. Its interest is not home mortgage interest; interest on money used in your business, rental or investments is entered there.
    Other,
}

/// Refinance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageRefinance {
    /// Balance of the earlier loan that this loan paid off
    /// Only the part that was still borrowed to buy, build or improve this home; any part of the earlier loan that was not belongs in the amount for other things below.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: original_amount() still sums these i64 loan parts unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 1, max = 100000000000)]
    pub refinanced: i64,
    /// Date of the earlier loan
    /// For a chain of refinances, the date of the original loan that bought, built or improved the home.
    #[libertas_date_only]
    // Tax-year source window: this date cannot be later than the applicable 2026 cutoff.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub earlier_incurred: u32,
    /// Date the original loan's term ended or ends
    /// Its scheduled final payment. When the original loan paid no principal until the end (a balloon note), the end of the first refinance's term, but not more than 30 years after that refinance. Needed when the original loan is from before October 14, 1987, or from before December 16, 2017 and this loan is later.
    /// [NullText]
    /// The original term is not needed for grandfathered-debt treatment
    /// [SetText]
    /// The original loan term ended or ends on
    #[libertas_date_only]
    pub earlier_term_ends: Option<u32>,
    /// More borrowed to buy, build or substantially improve this home
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: original_amount() still sums these i64 loan parts unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub improvement: i64,
    /// More borrowed for other things
    /// Cash out for anything else, including closing costs added to the loan.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: original_amount() still sums these i64 loan parts unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
}

/// Partly for this home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSplit {
    /// Amount used to buy, build or substantially improve this home
    /// For a line of credit, enter the total drawn for this use. The calculation treats the totals for each use as borrowed at the start. If you repaid principal between draws for different uses, this totals method can differ from tracing each draw.
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: original_amount() still sums these i64 loan parts unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 1, max = 100000000000)]
    pub improvement: i64,
    /// Date you borrowed that amount
    /// The loan's date, or for a line of credit the date of the draw (the latest one when you drew several times).
    #[libertas_date_only]
    // Tax-year source window: this date cannot be later than the applicable 2026 cutoff.
    #[libertas_number(min = $MORTGAGE_ACQUISITION_FIRST_DATE, max = $TAX_YEAR_END_DATE)]
    pub improvement_incurred: u32,
    /// Amount used for other things
    #[libertas_money("USD", 2)]
    // Temporary Unit 47 exclusion: original_amount() still sums these i64 loan parts unchecked; widen its receiver before removing this bound.
    #[libertas_number(min = 1, max = 100000000000)]
    pub other: i64,
}

/// Home securing the loan
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MortgageHome {
    /// Main home
    Main,
    /// Second home
    /// The one other home you choose for 2026. If you also rent it out, select this loan on the rental: the App applies the residence test from the rental's days, and when your personal use was not more than 14 days or 10% of the days rented, whichever is longer, the personal share of the interest is not deductible.
    Second,
    /// Second home that is a boat or recreational vehicle
    /// With sleeping, cooking and toilet facilities. Its interest is deductible, but not for the alternative minimum tax.
    SecondVehicle,
    /// Another home
    /// A third home, or a second home you do not choose for 2026. Its interest is not personally deductible home mortgage interest. For a home you also rent, keep the loan here and select it on the rental; the rental uses its acquisition-qualified interest without deducting it twice.
    NotQualified,
}

/// Lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageLender {
    /// Bank or other lender that sent a Form 1098
    Institution,
    /// Individual who sold you the home
    PrivateSeller {
        /// Seller's name
        #[libertas_size(min = 1)]
        name: String,
        /// Seller's SSN or EIN
        // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
        #[libertas_size(min = 9, max = 11)]
        identifying_number: String,
        /// Seller's address
        #[libertas_size(min = 1)]
        address: String,
    },
    /// A lender that sent no Form 1098, or your share of someone else's Form 1098
    /// Schedule A line 8b. When you and someone other than your spouse owe the loan and the Form 1098 is in that person's name, enter only the interest you paid.
    NoForm {
        /// Lender, or the person whose name is on the Form 1098
        #[libertas_size(min = 1)]
        name: String,
    },
}

/// How the 2026 average balance is figured
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageAverageBalance {
    /// Within the Pub 936 limit
    /// Confirmed, when this loan was saved, that every loan's outstanding balance together fits the applicable limit.
    WithinLimit,
    /// Average of the first and last balances
    FirstAndLast {
        /// Balance on the last day of 2026 the loan was secured by this home
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        last_balance: i64,
    },
    /// Interest paid divided by the interest rate
    InterestOverRate {
        /// Lowest annual interest rate in 2026, in thousandths of one percent
        /// 6125 means 6.125%.
        #[libertas_number(min = 1)]
        rate_thousandths_percent: u32,
    },
    /// From the lender's statements
    Statements {
        /// Average balance
        /// Add the monthly closing or average balances and divide by the months this home qualified, or use the lender's annual average.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        average_balance: i64,
    },
    /// Month-end balances from the lender's statements
    Monthly {
        /// Month-end balances
        /// ----
        /// Month
        // Tax structure: one entry for each of the twelve calendar months.
        #[libertas_size(min = 1, max = 12)]
        #[libertas_unordered]
        balances: Vec<MortgageMonthBalance>,
    },
}

/// Month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageMonthBalance {
    /// Month of 2026
    /// 1 is January.
    // Tax structure: a calendar year has twelve months.
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Balance at the end of the month
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub balance: i64,
}

/// Refund of overpaid interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageInterestRefund {
    /// No refund
    No,
    /// Yes
    Yes {
        /// Refund amount (Form 1098 box 4)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
        /// What the refund repays
        origin: MortgageRefundOrigin,
    },
}

/// What the box 4 refund repays
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageRefundOrigin {
    /// Interest you paid in 2026
    /// The lender's letter says so. It reduces this year's interest.
    ThisYear,
    /// Interest of an earlier year that you deducted
    /// Only the part that lowered that year's tax is income (Schedule 1 line 8z). Enter that year's figures; a refund and other recoveries of the same year are figured together.
    Deducted(crate::PriorYearRecovery),
    /// Interest of an earlier year that gave no tax benefit
    /// You took the standard deduction that year or did not deduct this interest. None of the refund is taxable.
    NotDeducted,
}

/// Points paid to obtain this loan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgagePoints {
    /// No points paid
    No,
    /// Points to buy your main home
    Purchase {
        /// Points (Form 1098 box 6)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
    },
    /// Points spread over the loan's term
    Amortized {
        /// Points paid
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
        /// Loan term in months
        #[libertas_number(min = 1)]
        term_months: u32,
        /// The loan ended in 2026
        ended: bool,
    },
    /// Points on a loan that paid to improve your main home
    Improvement {
        /// Points paid
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
        /// Loan term in months
        #[libertas_number(min = 1)]
        term_months: u32,
    },
}

impl MortgageLoan {
    pub(crate) fn ledger(&self) -> Option<&MortgageLedger> {
        match &self.details {
            MortgageLoanDetails::FirstLender(ledger) => Some(ledger),
            MortgageLoanDetails::EarlierLender { .. } => None,
        }
    }
    pub(crate) fn home(&self) -> MortgageHome {
        self.ledger().map_or(MortgageHome::NotQualified, |l| l.home)
    }
    fn purpose(&self) -> &MortgageUse {
        self.ledger().map_or(&MortgageUse::Other, |l| &l.purpose)
    }
    fn average(&self) -> &MortgageAverageBalance {
        self.ledger().map_or(&MortgageAverageBalance::WithinLimit, |l| &l.average)
    }

    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.details, MortgageLoanDetails::EarlierLender { .. }) {
            return None;
        }
        if matches!(self.average(), MortgageAverageBalance::FirstAndLast { last_balance } if *last_balance > self.outstanding_principal)
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_LAST_BALANCE_ABOVE_BOX_2")
                    .at_field("last_balance")
                    .in_field("average")
                    .in_field("details"),
            );
        }
        if matches!(self.points, MortgagePoints::Purchase { .. })
            && !(self.home() == MortgageHome::Main && matches!(self.purpose(), MortgageUse::Home))
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_PURCHASE_POINTS")
                    .at_field("points"),
            );
        }
        if matches!(self.points, MortgagePoints::Improvement { .. })
            && !(self.home() == MortgageHome::Main
                && match self.purpose() {
                    MortgageUse::Home | MortgageUse::Partly(_) => true,
                    MortgageUse::Refinance(r) => r.improvement > 0,
                    MortgageUse::Other => false,
                })
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_IMPROVEMENT_POINTS")
                    .at_field("points"),
            );
        }
        if let MortgageUse::Refinance(r) = self.purpose() {
            if r.earlier_incurred > self.incurred {
                return Some(
                    crate::validation::ValidationIssue::new("MORTGAGE_EARLIER_LOAN_AFTER_THIS_ONE")
                        .at_field("earlier_incurred")
                        .in_field("purpose")
                        .in_field("details"),
                );
            }
            if r.earlier_term_ends.is_none()
                && (r.earlier_incurred <= crate::year_constants::MORTGAGE_GRANDFATHERED_LAST_DATE
                    || r.earlier_incurred
                        <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE
                        && self.incurred > crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE)
            {
                return Some(
                    crate::validation::ValidationIssue::new("MORTGAGE_EARLIER_TERM_NEEDED")
                        .at_field("earlier_term_ends")
                        .in_field("purpose")
                        .in_field("details"),
                );
            }
        }
        if !matches!(self.purpose(), MortgageUse::Home | MortgageUse::Other)
            && self.outstanding_principal > self.original_amount()
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_BALANCE_ABOVE_AMOUNTS_BORROWED")
                    .at_field("outstanding_principal"),
            );
        }
        if self.other_amount() > 0
            && !matches!(self.purpose(), MortgageUse::Other)
            && matches!(
                self.average(),
                MortgageAverageBalance::Statements { .. }
                    | MortgageAverageBalance::InterestOverRate { .. }
            )
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_MIXED_USE_NEEDS_MONTHLY")
                    .at_field("average")
                    .in_field("details"),
            );
        }
        if let MortgageAverageBalance::Monthly { balances } = self.average()
            && let Some((index, _)) = balances
                .iter()
                .enumerate()
                .find(|(i, m)| balances[..*i].iter().any(|p| p.month == m.month))
        {
            return Some(
                crate::validation::ValidationIssue::new("MORTGAGE_MONTH_TWICE")
                    .at_field("month")
                    .in_list("balances", index)
                    .in_field("average")
                    .in_field("details"),
            );
        }
        if let MortgageInterestRefund::Yes { origin: MortgageRefundOrigin::Deducted(w), .. } =
            &self.refund_of_interest
            && w.deducted > w.itemized
        {
            return Some(
                crate::validation::ValidationIssue::new("RECOVERY_DEDUCTED_ABOVE_ITEMIZED")
                    .at_field("deducted")
                    .in_field("origin")
                    .in_field("refund_of_interest"),
            );
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        let money = |v: i64| (0..=100000000000).contains(&v);
        let date = |v: u32| v / 10000 <= 2026 && crate::federal_rules::valid_date(v);
        self.id > 0
            && match self.details {
                MortgageLoanDetails::EarlierLender { loan } => loan > 0,
                _ => true,
            }
            && money(self.interest)
            && money(self.outstanding_principal)
            && money(self.mortgage_insurance_premiums)
            && self.rule().is_none()
            && date(self.incurred)
            && match &self.refund_of_interest {
                MortgageInterestRefund::No => true,
                MortgageInterestRefund::Yes { amount, origin } => {
                    *amount > 0
                        && money(*amount)
                        && match origin {
                            MortgageRefundOrigin::Deducted(w) => w.valid(),
                            _ => true,
                        }
                }
            }
            && match self.points {
                MortgagePoints::No => true,
                MortgagePoints::Purchase { amount } => amount > 0 && money(amount),
                MortgagePoints::Amortized { amount, term_months, .. }
                | MortgagePoints::Improvement { amount, term_months } => {
                    amount > 0 && money(amount) && (1..=600).contains(&term_months)
                }
            }
            && match self.average() {
                MortgageAverageBalance::WithinLimit => true,
                MortgageAverageBalance::FirstAndLast { last_balance } => money(*last_balance),
                MortgageAverageBalance::InterestOverRate { rate_thousandths_percent } => {
                    (1..=30000).contains(rate_thousandths_percent)
                }
                MortgageAverageBalance::Statements { average_balance } => money(*average_balance),
                MortgageAverageBalance::Monthly { balances } => {
                    (1..=12).contains(&balances.len())
                        && balances.iter().all(|m| (1..=12).contains(&m.month) && money(m.balance))
                }
            }
            && match &self.lender {
                MortgageLender::Institution => true,
                MortgageLender::NoForm { name } => !name.is_empty(),
                MortgageLender::PrivateSeller { name, identifying_number, address } => {
                    !name.is_empty()
                        // Legal identifier format: nine-digit SSN/EIN with printed separators.
                        && (9..=11).contains(&identifying_number.len())
                        && !address.is_empty()
                }
            }
            && match self.purpose() {
                MortgageUse::Home | MortgageUse::Other => true,
                MortgageUse::Partly(s) => {
                    s.improvement > 0
                        && money(s.improvement)
                        && s.other > 0
                        && money(s.other)
                        && date(s.improvement_incurred)
                        && s.improvement_incurred
                            >= crate::year_constants::MORTGAGE_ACQUISITION_FIRST_DATE
                }
                MortgageUse::Refinance(r) => {
                    r.refinanced > 0
                        && money(r.refinanced)
                        && money(r.improvement)
                        && money(r.other)
                        && date(r.earlier_incurred)
                        && r.earlier_term_ends.is_none_or(crate::federal_rules::valid_date)
                }
            }
    }
    pub(crate) fn qualified_home(&self) -> bool {
        self.home() != MortgageHome::NotQualified
    }
    fn original_amount(&self) -> i64 {
        match self.purpose() {
            MortgageUse::Home | MortgageUse::Other => self.outstanding_principal,
            MortgageUse::Refinance(r) => r.refinanced + r.improvement + r.other,
            MortgageUse::Partly(s) => s.improvement + s.other,
        }
    }
    fn other_amount(&self) -> i64 {
        match self.purpose() {
            MortgageUse::Home => 0,
            MortgageUse::Other => self.outstanding_principal,
            MortgageUse::Refinance(r) => r.other,
            MortgageUse::Partly(s) => s.other,
        }
    }
    fn qualified_share(&self, amount: i64) -> i64 {
        match self.purpose() {
            MortgageUse::Home => amount,
            MortgageUse::Other => 0,
            _ => fraction(amount, self.qualified_original(), self.original_amount().max(1)),
        }
    }
    fn qualified_original(&self) -> i64 {
        self.original_amount() - self.other_amount()
    }
    fn average_balance(&self) -> i64 {
        match self.average() {
            MortgageAverageBalance::WithinLimit => self.outstanding_principal,
            MortgageAverageBalance::FirstAndLast { last_balance } => {
                fraction(self.outstanding_principal + last_balance, 1, 2)
            }
            MortgageAverageBalance::InterestOverRate { rate_thousandths_percent } => {
                fraction(self.interest_after_refund(), 100000, i64::from(*rate_thousandths_percent))
            }
            MortgageAverageBalance::Statements { average_balance } => *average_balance,
            MortgageAverageBalance::Monthly { balances } => {
                fraction(balances.iter().map(|m| m.balance).sum(), 1, 12)
            }
        }
    }
    pub(crate) fn interest_after_refund(&self) -> i64 {
        let refund = match &self.refund_of_interest {
            MortgageInterestRefund::Yes { amount, origin: MortgageRefundOrigin::ThisYear } => {
                *amount
            }
            _ => 0,
        };
        positive(self.interest - refund)
    }
    fn month_balances(&self) -> [i64; 12] {
        let mut months = [0; 12];
        let first = if self.incurred / 10000 < 2026 { 1 } else { self.incurred / 100 % 100 };
        match self.average() {
            MortgageAverageBalance::Monthly { balances } => {
                for m in balances {
                    if (1..=12).contains(&m.month) {
                        months[m.month as usize - 1] = m.balance;
                    }
                }
            }
            MortgageAverageBalance::FirstAndLast { last_balance } => {
                let n = i64::from(13 - first);
                for month in first..=12 {
                    months[month as usize - 1] = self.outstanding_principal
                        - fraction(
                            self.outstanding_principal - last_balance,
                            i64::from(month + 1 - first),
                            n,
                        );
                }
            }
            MortgageAverageBalance::WithinLimit => {
                for month in first..=12 {
                    months[month as usize - 1] = self.outstanding_principal;
                }
            }
            _ => months.fill(self.average_balance()),
        }
        months
    }
    fn category(date: u32) -> usize {
        if date <= crate::year_constants::MORTGAGE_GRANDFATHERED_LAST_DATE {
            0
        } else if date <= crate::year_constants::MORTGAGE_EARLIER_LIMIT_LAST_DATE {
            1
        } else {
            2
        }
    }
    // Pub. 936 payment order: equity, grandfathered, acquisition. Acquisition parts share
    // principal pro rata (inventor decision 2026-10-10). A refinance's old category ends
    // with its original term, tested at each month-end (§163(h)(3)(D)(iv), (F)(ii)(II)).
    fn month_parts(&self, month: usize, balance: i64) -> [i64; 3] {
        let mut initial = [0; 3];
        match self.purpose() {
            MortgageUse::Home => initial[Self::category(self.incurred)] = balance,
            MortgageUse::Other => {}
            MortgageUse::Partly(s) => {
                initial[Self::category(s.improvement_incurred)] = s.improvement
            }
            MortgageUse::Refinance(r) => {
                let end = crate::year_constants::TAX_YEAR_END_DATE / 10000 * 10000
                    + (month as u32 + 1) * 100
                    + match month {
                        1 => 28,
                        3 | 5 | 8 | 10 => 30,
                        _ => 31,
                    };
                let original = Self::category(r.earlier_incurred);
                let current = Self::category(self.incurred);
                let keeps = original == 2
                    || original == 1 && current == 1
                    || r.earlier_term_ends.is_some_and(|term| end <= term);
                initial[if keeps { original } else { current }] += r.refinanced;
                initial[current] += r.improvement;
            }
        }
        let acquisition = initial[1] + initial[2];
        let remaining_acquisition = phase::cap(acquisition, [balance]);
        let pre = if acquisition == 0 {
            0
        } else {
            fraction(remaining_acquisition, initial[1], acquisition)
        };
        [
            phase::cap(initial[0], [positive(balance - remaining_acquisition)]),
            pre,
            remaining_acquisition - pre,
        ]
    }
    fn parts_and_interest(&self) -> ([i64; 3], i64) {
        if matches!(self.purpose(), MortgageUse::Other) {
            return ([0; 3], 0);
        }
        if matches!(self.purpose(), MortgageUse::Home) {
            let mut parts = [0; 3];
            parts[Self::category(self.incurred)] = self.average_balance();
            return (parts, self.interest_after_refund());
        }
        let months = self.month_balances();
        let mut sums = [0; 3];
        for (month, balance) in months.iter().enumerate() {
            let parts = self.month_parts(month, *balance);
            for c in 0..3 {
                sums[c] += parts[c];
            }
        }
        let interest = fraction(
            self.interest_after_refund(),
            sums.iter().sum(),
            months.iter().sum::<i64>().max(1),
        );
        (sums.map(|v| fraction(v, 1, 12)), interest)
    }
    fn points_on_form_1098(&self) -> bool {
        matches!(self.lender, MortgageLender::Institution)
            && matches!(self.points, MortgagePoints::Purchase { .. })
    }
    // Points spread evenly over the loan term from the box 3 date; `ended`
    // deducts the rest this year.
    fn ratable_points(&self, amount: i64, term: u32, ended: bool) -> i64 {
        let term = i64::from(term);
        let prior = fraction(amount, months_through(self.incurred, 2025, term), term);
        if ended {
            amount - prior
        } else {
            fraction(amount, months_through(self.incurred, 2026, term), term) - prior
        }
    }
    fn points_deductible_current_year(&self) -> i64 {
        if matches!(self.purpose(), MortgageUse::Other) {
            return 0;
        }
        let original = self.original_amount().max(1);
        let qualified = self.qualified_original();
        let ratable = |amount, term, ended| self.ratable_points(amount, term, ended);
        match self.points {
            MortgagePoints::No => 0,
            MortgagePoints::Purchase { amount, .. } => {
                if self.incurred / 10000 == 2026 {
                    amount
                } else {
                    0
                }
            }
            MortgagePoints::Amortized { amount, term_months, ended } => {
                self.qualified_share(ratable(amount, term_months, ended))
            }
            MortgagePoints::Improvement { amount, term_months } => {
                let improvement = match self.purpose() {
                    MortgageUse::Home => original,
                    MortgageUse::Partly(s) => s.improvement,
                    MortgageUse::Refinance(r) => r.improvement,
                    MortgageUse::Other => 0,
                };
                let immediate = fraction(amount, improvement, original);
                let spread = fraction(amount, positive(qualified - improvement), original);
                (if self.incurred / 10000 == 2026 { immediate } else { 0 })
                    + ratable(spread, term_months, false)
            }
        }
    }
    // Pub. 527 "Points": the rental share of points is spread evenly over the
    // loan term, including the improvement share a main home deducts at once.
    // Purchase points of a main home keep the year-paid rule of section
    // 461(g)(2), which applies to the whole loan.
    fn rental_points_deductible_current_year(&self) -> i64 {
        match self.points {
            MortgagePoints::Improvement { amount, term_months }
                if !matches!(self.purpose(), MortgageUse::Other) =>
            {
                self.qualified_share(self.ratable_points(amount, term_months, false))
            }
            _ => self.points_deductible_current_year(),
        }
    }
}

fn months_through(started: u32, year: i64, term_months: i64) -> i64 {
    phase::bound(
        (year - i64::from(started / 10000)) * 12 + 13 - i64::from(started / 100 % 100),
        0,
        term_months,
    )
}

/// One document's interest, traced through its canonical loan's year-wide balances.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoanAllowance {
    pub id: i64,
    pub canonical_id: i64,
    pub raw_interest_cents: i64,
    pub qualified_interest_cents: i64,
    pub regular_interest_cents: i64,
    pub amt_interest_cents: i64,
    /// Acquisition-qualified current-year personal points before the debt cap.
    pub qualified_points_cents: i64,
    pub regular_points_cents: i64,
    pub amt_points_cents: i64,
    /// Current-year rental points amortization before the rental-use fraction.
    pub rental_points_cents: i64,
    pub qualified_insurance_cents: i64,
    /// Home-eligible premiums before the AGI phaseout, independent of the debt cap.
    pub regular_insurance_cents: i64,
    pub amt_insurance_cents: i64,
    pub points_reported_on_1098: bool,
    pub reported_on_1098: bool,
}

pub(crate) fn graph_rule(loans: &[MortgageLoan]) -> Option<&'static str> {
    graph_issue(loans).map(|(_, issue)| issue.key)
}

// The index belongs to this exact loan vector. Page adapters map the owning
// record's field names before adding the original submitted collection index.
pub(crate) fn graph_issue(
    loans: &[MortgageLoan],
) -> Option<(usize, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    if let Some((index, _)) = loans
        .iter()
        .enumerate()
        .find(|(i, loan)| loan.id <= 0 || loans[..*i].iter().any(|p| p.id == loan.id))
    {
        return Some((index, ValidationIssue::new("MORTGAGE_LOAN_ID_INVALID").at_field("id")));
    }
    for (index, loan) in loans.iter().enumerate() {
        if let MortgageLoanDetails::EarlierLender { loan: target } = loan.details {
            let linked =
                |key| (index, ValidationIssue::new(key).at_field("loan").in_field("details"));
            if target == loan.id {
                return Some(linked("MORTGAGE_LOAN_SELF_REFERENCE"));
            }
            let Some(root) = loans.iter().find(|root| root.id == target) else {
                return Some(linked("MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED"));
            };
            if root.ledger().is_none() {
                return Some(linked("MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED"));
            }
            if root.incurred != loan.incurred {
                return Some((
                    index,
                    ValidationIssue::new("MORTGAGE_LOAN_ORIGINATION_DIFFERS").at_field("incurred"),
                ));
            }
            if let Some(issue) = attributed(loans, loan).and_then(|loan| loan.issue()) {
                return Some((index, issue));
            }
        }
    }
    None
}
fn canonical<'a>(loans: &'a [MortgageLoan], loan: &'a MortgageLoan) -> Option<&'a MortgageLoan> {
    match loan.details {
        MortgageLoanDetails::FirstLender(_) => Some(loan),
        MortgageLoanDetails::EarlierLender { loan: target } => {
            loans.iter().find(|root| root.id == target && root.ledger().is_some())
        }
    }
}
fn attributed(loans: &[MortgageLoan], loan: &MortgageLoan) -> Option<MortgageLoan> {
    let root = canonical(loans, loan)?;
    let mut attributed = loan.clone();
    attributed.details = root.details.clone();
    // Balance allocation belongs to the canonical ledger, including its beginning balance/date.
    attributed.outstanding_principal = root.outstanding_principal;
    attributed.incurred = root.incurred;
    Some(attributed)
}
fn debt_categories(loans: &[MortgageLoan]) -> [i64; 3] {
    let mut categories = [0; 3];
    for root in loans.iter().filter(|loan| loan.ledger().is_some() && loan.qualified_home()) {
        let mut ledger = root.clone();
        if let MortgageAverageBalance::InterestOverRate { rate_thousandths_percent } =
            root.average()
        {
            let interest = loans
                .iter()
                .filter(|loan| canonical(loans, loan).is_some_and(|r| r.id == root.id))
                .map(MortgageLoan::interest_after_refund)
                .sum();
            if let MortgageLoanDetails::FirstLender(ref mut details) = ledger.details {
                details.average = MortgageAverageBalance::Statements {
                    average_balance: fraction(
                        interest,
                        100000,
                        i64::from(*rate_thousandths_percent),
                    ),
                };
            }
        }
        let (parts, _) = ledger.parts_and_interest();
        for (sum, part) in categories.iter_mut().zip(parts) {
            *sum += part;
        }
    }
    categories
}
fn debt_limit(categories: [i64; 3], separate: bool) -> i64 {
    let [line1, line2, line7] = categories;
    let line3 = if separate {
        crate::year_constants::MORTGAGE_DEBT_LIMIT_GRANDFATHERED_SEPARATE_CENTS
    } else {
        crate::year_constants::MORTGAGE_DEBT_LIMIT_GRANDFATHERED_CENTS
    };
    let line6 = phase::cap(phase::floor(line1, line3), [line1 + line2]);
    let line8 = if separate {
        crate::year_constants::MORTGAGE_DEBT_LIMIT_SEPARATE_CENTS
    } else {
        crate::year_constants::MORTGAGE_DEBT_LIMIT_CENTS
    };
    phase::cap(phase::floor(line6, line8), [line6 + line7])
}
fn debt_ratio(loans: &[MortgageLoan], separate: bool) -> i64 {
    let categories = debt_categories(loans);
    let balance: i64 = categories.iter().sum();
    let limit = debt_limit(categories, separate);
    if limit >= balance { 1000 } else { fraction(limit, 1000, balance) }
}
pub(crate) fn loan_allowances(loans: &[MortgageLoan], separate: bool) -> Vec<LoanAllowance> {
    let ratio = debt_ratio(loans, separate);
    loans
        .iter()
        .filter_map(|loan| {
            let root = canonical(loans, loan)?;
            let attributed = attributed(loans, loan)?;
            let (_, qualified) = attributed.parts_and_interest();
            let allowed = if root.qualified_home() { fraction(qualified, ratio, 1000) } else { 0 };
            let qualified_points = attributed.points_deductible_current_year();
            let points =
                if root.qualified_home() { fraction(qualified_points, ratio, 1000) } else { 0 };
            let qualified_insurance = attributed.qualified_share(loan.mortgage_insurance_premiums);
            let insurance = if root.qualified_home() { qualified_insurance } else { 0 };
            let vehicle = root.home() == MortgageHome::SecondVehicle;
            Some(LoanAllowance {
                id: loan.id,
                canonical_id: root.id,
                raw_interest_cents: loan.interest_after_refund(),
                qualified_interest_cents: qualified,
                regular_interest_cents: allowed,
                amt_interest_cents: if vehicle { 0 } else { allowed },
                qualified_points_cents: qualified_points,
                regular_points_cents: points,
                amt_points_cents: if vehicle { 0 } else { points },
                rental_points_cents: attributed.rental_points_deductible_current_year(),
                qualified_insurance_cents: qualified_insurance,
                regular_insurance_cents: insurance,
                amt_insurance_cents: if vehicle { 0 } else { insurance },
                points_reported_on_1098: loan.points_on_form_1098(),
                reported_on_1098: matches!(loan.lender, MortgageLender::Institution),
            })
        })
        .collect()
}
#[derive(Default)]
pub(crate) struct Limited {
    pub interest: [i64; 2],
    pub points: [i64; 2],
    pub amt_dwelling_interest: i64,
}
pub(crate) fn calculate(
    loans: &[MortgageLoan],
    separate: bool,
    result: &mut FederalResult,
) -> Limited {
    if loans.is_empty() {
        return Limited::default();
    }
    let categories = debt_categories(loans);
    let allowances = loan_allowances(loans, separate);
    let mut paid = [0; 2];
    let mut allowed = [0; 2];
    let mut points = [0; 2];
    let mut other_use = 0;
    let mut not_qualified = 0;
    let mut vehicle = 0;
    for allowance in &allowances {
        let Some(root) = loans.iter().find(|loan| loan.id == allowance.canonical_id) else {
            continue;
        };
        if !root.qualified_home() {
            not_qualified += allowance.raw_interest_cents;
            continue;
        }
        other_use += allowance.raw_interest_cents - allowance.qualified_interest_cents;
        let target = usize::from(!allowance.reported_on_1098);
        paid[target] += allowance.qualified_interest_cents;
        allowed[target] += allowance.regular_interest_cents;
        points[usize::from(!allowance.points_reported_on_1098)] += allowance.regular_points_cents;
        vehicle += allowance.regular_interest_cents - allowance.amt_interest_cents
            + allowance.regular_points_cents
            - allowance.amt_points_cents;
    }
    emit_lines(
        result,
        &mut (),
        [
            ("mortgage_grandfathered_balance", categories[0]),
            ("mortgage_pre_2018_balance", categories[1]),
            ("mortgage_post_2017_balance", categories[2]),
            ("mortgage_qualified_loan_limit", debt_limit(categories, separate)),
            ("mortgage_average_balance", categories.iter().sum()),
            ("mortgage_qualified_interest", paid.iter().sum()),
            ("mortgage_other_use_interest", other_use),
            ("mortgage_not_qualified_home_interest", not_qualified),
        ],
        |(id, cents), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyCents(cents),
                &["deductions.mortgage"],
                SOURCE,
                crate::Message::new(
                    alloc::format!("EXPLAIN_{}", id.to_ascii_uppercase()),
                    alloc::vec![],
                ),
            ))
        },
    );
    let interest = allowed.map(dollars);
    line(
        result,
        "mortgage_nondeductible_interest",
        dollars(paid.iter().sum()) - interest.iter().sum::<i64>(),
        msg!("EXPLAIN_MORTGAGE_NONDEDUCTIBLE_INTEREST"),
    );
    Limited { interest, points: points.map(dollars), amt_dwelling_interest: dollars(vehicle) }
}

fn insurance_remaining_steps(agi: i64, separate: bool) -> i64 {
    let (start, step) = if separate {
        (
            MORTGAGE_INSURANCE_PHASEOUT_START_SEPARATE_DOLLARS,
            MORTGAGE_INSURANCE_PHASEOUT_STEP_SEPARATE_DOLLARS,
        )
    } else {
        (MORTGAGE_INSURANCE_PHASEOUT_START_DOLLARS, MORTGAGE_INSURANCE_PHASEOUT_STEP_DOLLARS)
    };
    positive(
        MORTGAGE_INSURANCE_PHASEOUT_STEPS
            - phase::steps(agi - start, step, phase::StepRounding::Up),
    )
}
pub(crate) fn insurance_phaseout_cents(cents: i64, agi: i64, separate: bool) -> i64 {
    phase::remaining_fraction(
        cents,
        insurance_remaining_steps(agi, separate),
        MORTGAGE_INSURANCE_PHASEOUT_STEPS,
    )
}

pub(crate) fn insurance_phaseout_exact(
    cents: &crate::kernel::ratio::exact::Exact,
    agi: i64,
    separate: bool,
) -> crate::kernel::ratio::exact::Exact {
    use crate::kernel::ratio::exact;
    cents
        * exact::ratio(insurance_remaining_steps(agi, separate), MORTGAGE_INSURANCE_PHASEOUT_STEPS)
            .expect("the statutory premium phaseout has a positive number of steps")
}
pub(crate) fn insurance_phaseout(cents: i64, agi: i64, separate: bool) -> i64 {
    dollars(insurance_phaseout_cents(cents, agi, separate))
}
pub(crate) fn vehicle_insurance_premiums(loans: &[MortgageLoan], agi: i64, separate: bool) -> i64 {
    insurance_phaseout(
        loan_allowances(loans, separate)
            .iter()
            .map(|a| a.regular_insurance_cents - a.amt_insurance_cents)
            .sum(),
        agi,
        separate,
    )
}

fn line(result: &mut FederalResult, id: &str, amount: i64, explanation: crate::Message) {
    crate::kernel::lines::line(result, id, amount, &["deductions.mortgage"], SOURCE, explanation);
}
