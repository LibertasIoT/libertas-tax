// se_benefits items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The self-employed benefits, IRA contributions, educator and student-loan
// topics of the adjustments hub (unit M3-P8). Their stored counterparts are
// FederalBusiness.health_insurance/.retirement (below), FederalIras,
// EducatorClaim and StudentLoanClaim; these are the page shapes the
// interview maps onto and back from them.
/// Self-employed health insurance and retirement plan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SelfEmployedBenefits {
    /// Your businesses
    #[libertas_read_only]
    #[libertas_suppress_label]
    /// ----
    /// Business
    pub businesses: Vec<BusinessBenefits>,
    /// Your partnerships
    #[libertas_read_only]
    #[libertas_suppress_label]
    /// ----
    /// Partnership
    pub partnerships: Vec<PartnerBenefits>,
    /// Your S corporations
    #[libertas_read_only]
    #[libertas_suppress_label]
    /// ----
    /// S corporation
    pub scorporations: Vec<HealthPlanPage>,
    /// Premium tax credit and health deduction calculation method
    #[libertas_default(Simplified)]
    pub ptc_method: crate::HealthPtcMethod,
    /// Marketplace policies established under these businesses
    #[libertas_unordered]
    pub policies: Vec<crate::HealthPolicyAllocationPage>,
    /// Actual mixed coverage-family benchmarks
    #[libertas_unordered]
    pub benchmarks: Vec<crate::HealthPolicyBenchmarkPage>,
    /// Partnerships with activity statements
    #[libertas_read_only]
    #[libertas_suppress_label]
    /// ----
    /// Partnership
    pub activity_partnerships: Vec<crate::K1ActivityPartnerBenefits>,
    /// S corporations with activity statements
    #[libertas_read_only]
    #[libertas_suppress_label]
    /// ----
    /// S corporation
    pub activity_scorporations: Vec<crate::K1ActivityScorpBenefits>,
}
/// Business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessBenefits {
    /// Business
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Business
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: Vec<u8>,
    /// Health insurance you paid for yourself and your family
    #[libertas_default(None)]
    pub health: SeHealth,
    /// Retirement plan
    #[libertas_default(None)]
    pub retirement: SeRetirement,
}
/// S corporation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthPlanPage {
    /// S corporation
    #[libertas_hidden]
    #[libertas_read_only]
    pub activity: i64,
    /// S corporation
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: Vec<u8>,
    /// Health plan established under this corporation
    #[libertas_default(None)]
    pub health: SeHealth,
}
/// Health insurance you paid for yourself and your family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SeHealth {
    /// None
    None,
    /// Actual plan and paid coverage
    Plan {
        /// Actual business establishment and entity payment
        establishment: crate::HealthEstablishmentPage,
        /// Ordinary coverage months
        /// Include each month's actual employer eligibility and any included child-only premium increase. Marketplace premiums come from saved policy assignments below; ordinary premiums must not repeat them.
        // Tax structure: one coverage entry for each calendar month.
        #[libertas_size(max = 12)]
        #[libertas_unordered]
        months: Vec<crate::BusinessHealthMonth>,
        /// Long-term-care coverage by actual insured person
        #[libertas_unordered]
        long_term_care: Vec<crate::BusinessCarePremiumPage>,
    },
}
/// Retirement plan
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SeRetirement {
    /// None
    None,
    /// Traditional SEP
    /// The plan is timely established and funded; this is your only self-employment business and your only self-employed plan, with no related-employer plan, excess carryover or special treatment. Employees' SEP contributions, at the same plan rate, go on Schedule C line 19.
    Sep {
        /// Employer contribution for your own SEP IRA for 2026
        /// Include timely contributions paid by the return deadline including extensions. Exclude regular personal IRA contributions, Roth SEP contributions, employee contributions and prior-year amounts. Do not list this as a business expense.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contribution: i64,
        /// Plan contribution rate
        /// Enter the employer rate stated in your plan, before the self-employed adjustment: The App calculates the reduced owner rate; do not enter 20% for a 25% plan.
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 0, max = 25, step = 0.01)]
        #[libertas_physical_unit("percent")]
        rate: f64,
    },
    /// SIMPLE IRA
    /// Your compensation is this business's net earnings from self-employment (Schedule SE line 4) before your SIMPLE contributions. Employees' matching or nonelective contributions go on Schedule C line 19.
    Simple {
        /// Your salary reduction contributions for 2026
        /// At most $17,000, plus a catch-up of $4,000 from age 50 ($5,250 at ages 60 to 63), and not more than your compensation; the App applies your age from your birth date. Your elective deferrals under a 401(k) or other plan of an employer count toward the same yearly limit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deferral: i64,
        /// Employer part of the plan
        employer: SimpleEmployer,
        /// Your employer contribution for yourself for 2026
        /// The match or nonelective amount you deposited for your own account by the return due date including extensions. The App deducts no more than the plan formula allows.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        employer_contribution: i64,
    },
    /// 401(k), profit-sharing, money purchase or other defined-contribution plan
    /// One plan of this business; the App figures the Pub. 560 Deduction Worksheet for Self-Employed. Employees' contributions go on Schedule C line 19.
    Qualified {
        /// Plan contribution rate
        /// The employer rate stated in your plan, before the self-employed adjustment: 2500 means 25%. Zero for a plan with elective deferrals only.
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 0, max = 25, step = 0.01)]
        #[libertas_physical_unit("percent")]
        rate: f64,
        /// Employer contribution for yourself for 2026
        /// The profit-sharing or money-purchase amount you deposited for your own account by the return due date including extensions, without the deferrals below.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contribution: i64,
        /// Your elective deferrals for 2026
        /// At most $24,500 across this plan and any employer plan you deferred into (W-2 box 12 codes D, E, F, S, AA, BB). Catch-up contributions go below.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        deferral: i64,
        /// Your catch-up contributions for 2026
        /// Only from age 50: at most $8,000, or $11,250 at ages 60 to 63; the App applies your age from your birth date.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        catch_up: i64,
        /// Designated Roth part of the deferrals and catch-up
        /// Not deductible; the rest of the deferrals is.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        roth: i64,
    },
    /// Defined benefit plan
    /// The deductible contribution is figured by the plan's actuary; the App deducts it up to this business's net earnings and does not check the actuarial limit.
    DefinedBenefit {
        /// Deductible contribution for 2026 from the actuary's report
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contribution: i64,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for SeRetirement {}

/// Partnership
// One row per Schedule K-1 (Form 1065) (unit T2-43p): boxes 13 code M and R
// and box 19 code A are read-only copies of the K-1 topic's record.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerBenefits {
    /// Partnership
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Partnership
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: Vec<u8>,
    /// K-1 box 13 code M amounts paid for medical insurance
    /// Entered on the Schedule K-1 topic. The partnership also included these premiums in box 4a; count them once, in the premiums below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_read_only]
    pub medical_insurance: i64,
    /// K-1 box 13 code R pensions and IRAs
    /// Entered on the Schedule K-1 topic.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_read_only]
    pub pensions: i64,
    /// Box 19 code A cash distributions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub cash_distributions: i64,
    /// Health insurance for yourself and your family
    /// Policies in the partnership's name, or in your name when the partnership paid or reimbursed the premiums and reported them on your K-1. Include every premium once, however it was paid. A policy in your own name the partnership did not reimburse is not coverage under the partnership: enter it under Medical expenses.
    #[libertas_default(None)]
    pub health: SeHealth,
    /// Part the partnership paid and charged against your distributions
    /// Premiums for a policy in the partnership's name that it paid for you and subtracted from your cash distributions instead of reporting them as guaranteed payments. They are part of the premiums above and of box 19 code A.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub from_distributions: i64,
    /// Retirement plan contribution on this K-1
    #[libertas_default(None)]
    pub retirement: PartnerPlan,
}
/// Retirement plan contribution on this K-1
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PartnerPlan {
    /// No plan contribution
    /// Box 13 code R is blank.
    None,
    /// A traditional SEP the partnership set up
    /// The partnership established and funded it on time for every eligible partner and employee, and it is your only self-employed retirement plan.
    Sep {
        /// Partnership plan contribution rate
        /// Use the stated employer rate before the self-employed reduction, not the reduced 20% owner rate.
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 0, max = 25, step = 0.01)]
        #[libertas_physical_unit("percent")]
        #[libertas_default(25)]
        rate: f64,
    },
    /// A payment to your own IRA, or a SEP with other facts
    /// Its deduction is not calculated yet; your return stops with a message naming it.
    Other,
    /// A SIMPLE IRA
    /// Your compensation is this partnership's net earnings from self-employment. Enter what you contributed for yourself, including what the partnership reported in box 13 code R.
    Simple {
        /// Your salary reduction contributions for 2026
        /// At most $17,000, plus a catch-up of $4,000 from age 50 ($5,250 at ages 60 to 63); the App applies your age from your birth date.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deferral: i64,
        /// Employer part of the plan
        employer: SimpleEmployer,
        /// Your employer contribution for yourself for 2026
        /// The match or nonelective amount deposited for your own account by the return due date including extensions.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        employer_contribution: i64,
    },
    /// A 401(k), profit-sharing or money purchase plan
    /// The App figures the Pub. 560 Deduction Worksheet for Self-Employed on this partnership's earnings. Enter what you contributed for yourself, including what the partnership reported in box 13 code R.
    Qualified {
        /// Plan contribution rate
        /// The employer rate stated in your plan. Zero for a plan with elective deferrals only.
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 0, max = 25, step = 0.01)]
        #[libertas_physical_unit("percent")]
        rate: f64,
        /// Employer contribution for yourself for 2026
        /// Without the deferrals below.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contribution: i64,
        /// Your elective deferrals for 2026
        /// At most $24,500 across this plan and any employer plan you deferred into.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        deferral: i64,
        /// Your catch-up contributions for 2026
        /// Only from age 50: at most $8,000, or $11,250 at ages 60 to 63.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        catch_up: i64,
        /// Designated Roth part of the deferrals and catch-up
        /// Not deductible.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        roth: i64,
    },
    /// A defined benefit plan
    /// The plan's actuary figures the deductible contribution; the App deducts it up to your earnings from this partnership.
    DefinedBenefit {
        /// Deductible contribution for 2026 from the actuary's report
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contribution: i64,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for PartnerPlan {}
