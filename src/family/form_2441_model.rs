#[allow(unused_imports)]
use super::*;

pub use crate::accepted_dependents::DependentCare;

/// Care you paid for so you could work or look for work
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CareCosts {
    /// No
    No,
    /// Yes
    Yes {
        /// Paid in 2026 for care while under 13, or while unable to care for themselves
        /// Include care an employer plan paid or reimbursed; Child and dependent care removes excluded benefits once. Exclude state social-services reimbursements, 2025 care paid now and 2027 care prepaid now. Care outside your home for someone age 13 or older requires at least 8 hours daily in your home; a care center must meet state and local regulations. Overnight camp does not count.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
        /// Age 13 or older and physically or mentally unable to care for themselves
        #[libertas_default(false)]
        incapable_self_care: bool,
    },
}

/// Child and dependent care
/// Form 2441: the care paid for each child or dependent, who provided the care, employer dependent-care benefits (W-2 box 10), a spouse unable to care for themselves, a student or disabled spouse's deemed income, and 2025 care paid in 2026.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CarePage {
    /// Combined maximum of your dependent-care plans (Form 2441 line 21)
    pub plan_maximum: crate::CarePlanMaximum,
    /// Care you paid for each person
    /// One row for each person on Children and people you support.
    // The service makes one row per person; the person edits the rows but
    // never adds or removes one (ReadOnly fixes a List's membership).
    #[libertas_read_only]
    /// ----
    /// Person
    pub qualifying_people: Vec<CareQualifyingRow>,
    /// People whose care you have not answered yet
    /// Added on Children and people you support since this page was saved.
    // Asked unanswered beside the answered people (BeginCare); empty once
    // everyone is answered.
    #[libertas_read_only]
    /// ----
    /// Person
    pub new_people: Vec<CareQualifyingRow>,
    /// Care providers
    /// Everyone you paid for care in 2026 (Form 2441 Part I). Leave the list empty if you only received employer benefits and paid no one.
    #[libertas_unordered]
    /// ----
    /// Provider
    pub providers: Vec<CareProvider>,
    /// Employer dependent-care benefits (W-2 box 10)
    /// One row for each W-2 that shows an amount in box 10. Enter or import the W-2 on Wages first.
    // The service makes one row per W-2; the person edits the rows but never
    // adds or removes one (ReadOnly fixes a List's membership).
    #[libertas_read_only]
    /// ----
    /// W-2
    pub benefits: Vec<CareBenefitRow>,
    /// 2025 plan balance used for care in 2026 during the grace period
    /// Form 2441 line 13: an unused 2025 dependent-care plan amount your employer let you use for care early in 2026 (Notice 2005-42). Leave the list empty if none.
    // Tax structure: one prior-year dependent-care grace balance per filer.
    #[libertas_size(max = 2)]
    #[libertas_unordered]
    /// ----
    /// Employee
    pub grace: Vec<crate::CareGraceBalanceInput>,
    /// Benefits from your own business or partnership
    /// A dependent care assistance program of your sole proprietorship, farm or partnership (Schedule K-1 box 13 code O). Form 2441 lines 22 and 24. Leave the list empty if none.
    #[libertas_unordered]
    /// ----
    /// Benefit
    pub own_business: Vec<crate::CareOwnBusinessBenefitInput>,
    /// A filer who was a full-time student or unable to care for themselves (joint return)
    /// Each of you whose own page says full-time student or unable to care for yourself. Form 2441 line B deemed income of $250 or $500 a month, and line 2 for a spouse unable to care for themselves.
    // Tax structure: one dependent-care deemed-income entry per filer.
    #[libertas_size(max = 2)]
    #[libertas_unordered]
    /// ----
    /// Filer
    pub filers: Vec<crate::CareFilerInput>,
    /// Filing-status facts (Form 2441 line 19)
    /// Follows the filing status chosen on Your 2026 return.
    #[libertas_constrained_by("$.allowed_filing")]
    pub filing: crate::CareFilingStatus,
    /// 2026 care you had not paid by December 31, 2026
    /// Form 2441 line 16 counts care incurred in 2026 whether paid or not. Claim the amount that remained unpaid if you want it included in the benefits worksheet. Care you prepaid in 2026 for 2027 is never counted.
    /// [NullText]
    /// No unpaid 2026 care expenses are claimed
    /// [SetText]
    /// The unpaid 2026 care expenses claimed are
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    #[libertas_default($null)]
    pub unpaid_return_year_care: Option<i64>,
    /// 2025 care you paid in 2026
    /// Form 2441 line 9b, Worksheet A: expenses incurred in 2025 and first paid in 2026, when your 2025 credit was not on the maximum expenses. Choose the null answer when no prior-year expenses are claimed.
    /// [NullText]
    /// No 2025 care expenses paid in 2026 are claimed
    /// [SetText]
    /// The 2025 care expenses paid in 2026 are
    #[libertas_default($null)]
    pub prior_year: Option<crate::PriorYearCareInput>,
}

// The care page as BeginCare carries it before every row is answered: the
// page's values by the same names, each row without its care answer, so the
// editor built from it starts every row unanswered.
/// Child and dependent care
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareSeed {
    /// Combined maximum of your dependent-care plans (Form 2441 line 21)
    pub plan_maximum: crate::CarePlanMaximum,
    /// Care you paid for each person
    pub qualifying_people: Vec<CareQualifyingRow>,
    /// People whose care you have not answered yet
    pub new_people: Vec<CareSeedRow>,
    /// Care providers
    pub providers: Vec<CareProvider>,
    /// Employer dependent-care benefits (W-2 box 10)
    pub benefits: Vec<CareSeedBenefitRow>,
    /// 2025 plan balance used for care in 2026 during the grace period
    // Tax structure: one prior-year dependent-care grace balance per filer.
    #[libertas_size(max = 2)]
    pub grace: Vec<crate::CareGraceBalanceInput>,
    /// Benefits from your own business or partnership
    pub own_business: Vec<crate::CareOwnBusinessBenefitInput>,
    /// A filer who was a full-time student or unable to care for themselves (joint return)
    // Tax structure: one dependent-care deemed-income entry per filer.
    #[libertas_size(max = 2)]
    pub filers: Vec<crate::CareFilerInput>,
    /// Filing-status facts (Form 2441 line 19)
    pub filing: crate::CareFilingStatus,
    /// 2026 care you had not paid by December 31, 2026
    /// [NullText]
    /// No unpaid 2026 care expenses are claimed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub unpaid_return_year_care: Option<i64>,
    /// 2025 care you paid in 2026
    /// [NullText]
    /// No 2025 care expenses paid in 2026 are claimed
    pub prior_year: Option<crate::PriorYearCareInput>,
}

/// W-2
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareSeedBenefitRow {
    /// Income record identity
    pub wage: i64,
    /// Employer
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Employee
    pub owner: Owner,
    /// Box 10 dependent care benefits
    #[libertas_money("USD", 2)]
    pub box10: i64,
    /// Part of box 10 that is also in box 1 wages
    /// [NullText]
    /// No box 10 benefits are also included in box 1 wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub in_box_1: Option<i64>,
    /// Forfeited, or carried forward to 2027 (line 14)
    /// [NullText]
    /// No box 10 benefits were forfeited or carried forward to 2027
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub forfeited: Option<i64>,
}
impl From<CareBenefitRow> for CareSeedBenefitRow {
    fn from(v: CareBenefitRow) -> Self {
        let CareBenefitRow { wage, label, owner, box10, in_box_1, forfeited } = v;
        Self { wage, label, owner, box10, in_box_1, forfeited }
    }
}
impl From<CareSeedBenefitRow> for CareBenefitRow {
    fn from(v: CareSeedBenefitRow) -> Self {
        let CareSeedBenefitRow { wage, label, owner, box10, in_box_1, forfeited } = v;
        Self { wage, label, owner, box10, in_box_1, forfeited }
    }
}

/// Person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareSeedRow {
    /// Person identity
    pub id: i64,
    /// Person
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Qualifying child under 13 during 2026
    pub under_13_child: bool,
}

/// Person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareQualifyingRow {
    /// Person identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Person
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Care you paid for so you could work or look for work
    pub care: CareCosts,
    /// Qualifying child under 13 during 2026
    #[libertas_hidden]
    #[libertas_read_only]
    pub under_13_child: bool,
}

/// W-2
/// One W-2 with an amount in box 10; the employer and amount come from the W-2 you entered.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareBenefitRow {
    /// Income record identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub wage: i64,
    /// Employer
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub label: Vec<u8>,
    /// Employee
    #[libertas_hidden]
    #[libertas_read_only]
    pub owner: Owner,
    /// Box 10 dependent care benefits
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub box10: i64,
    /// Part of box 10 that is also in box 1 wages
    /// Benefits above the plan's exclusion are reported in box 10 and again as wages in box 1. Your employer's statement shows it. Choose the null answer if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    /// [NullText]
    /// No box 10 benefits are also included in box 1 wages
    /// [SetText]
    /// Box 10 benefits also included in box 1 wages were
    pub in_box_1: Option<i64>,
    /// Forfeited, or carried forward to 2027 (line 14)
    /// Any part of box 10 you did not receive because you did not incur the expense, or that the plan lets you carry to 2027. Not amounts you still expect for 2026 care. Choose the null answer if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    /// [NullText]
    /// No box 10 benefits were forfeited or carried forward to 2027
    /// [SetText]
    /// Box 10 benefits forfeited or carried forward to 2027 were
    pub forfeited: Option<i64>,
}
