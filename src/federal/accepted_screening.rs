use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Additional coverage checks
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalScreening {
    /// Farm income averaging
    /// Optional Schedule J election using retained base-year returns. Leave absent for ordinary tax; the review compares your chosen election with tax without averaging.
    pub farm_averaging: Option<crate::FarmAveraging>,
    /// Optional self-employment earnings elections
    /// Leave empty for ordinary actual earnings. Each filer chooses independently; one election per person applies to all their farm/nonfarm businesses.
    // Tax structure: one optional self-employment election per filer.
    #[libertas_size(max = 2)]
    pub optional_se: Vec<crate::SeElection>,
    /// Foreign earned income exclusion (Form 2555)
    /// One for each person; it applies to all of that person's foreign pay and businesses.
    // Tax structure: one foreign-earned income claim per filer.
    #[libertas_size(max = 2)]
    /// ----
    /// Form 2555
    pub foreign_earned: Vec<crate::ForeignEarnedClaim>,
    /// Ministers, religious workers and approved Form4029 exemptions
    // Tax structure: one clergy claim per filer.
    #[libertas_size(max = 2)]
    /// ----
    /// Minister or religious worker
    pub clergy: Vec<crate::ClergyClaim>,
    /// Current farming-loss carryback election
    /// Leave empty when no farming NOL is generated. A missing choice preserves the statutory carryback; it never elects a carryforward. One election applies to the whole 2026 return, including both filers of a joint return.
    pub farm_nol: Option<crate::FarmNolElection>,
    /// Household employment taxes
    /// Leave empty if no household employer taxes need Schedule H. Use one record per filer, with payroll and state unemployment records.
    pub household_employment: Vec<crate::HouseholdEmployer>,
    /// Retired public-safety insurance election and premium allocation
    /// Leave empty to make no election. Order controls allocation of each officer's annual limit across distributions. The App adds remaining eligible premiums to medical itemization; never enter them twice.
    pub public_safety: Vec<crate::PublicSafetyPremium>,
    /// ABLE account contribution limits and excess tax
    /// Leave empty if no contribution-limit worksheet is needed. Add one entry per beneficiary on this return; this is not an income deduction or a Saver's Credit claim.
    // Tax structure: one ABLE contribution summary per filer.
    #[libertas_size(max = 2)]
    pub able_contributions: Vec<crate::AbleContributions>,
    /// 2026 Trump-account excess contributions for a beneficiary filing this return
    /// Enter only the taxpayer's account, or a spouse's on a joint return. A parent's contributions to a dependent child's account belong on the child's own Form 5329, not the parent's.
    /// ----
    /// Account beneficiary
    pub trump_accounts: Vec<crate::TrumpAccount>,
    /// Incentive stock option exercises with shares still held at year-end
    /// ----
    /// Exercise
    pub iso_exercises: Vec<crate::IsoExerciseSource>,
    /// Accepted employee stock purchase plan acquisition documents
    pub espp_acquisitions: Vec<crate::EsppAcquisition>,
    /// Keyed personal facts for ESPP sale documents
    pub espp_sale_answers: Vec<crate::EsppSaleAnswers>,
    /// Keyed personal facts for ISO sale documents
    pub iso_sale_answers: Vec<crate::IsoSaleAnswers>,
    /// Keyed personal facts for NUA sale documents
    pub nua_sale_answers: Vec<crate::NuaSaleAnswers>,
    /// Prior returns with capital losses
    // Tax structure: one prior capital-return history per filer.
    #[libertas_size(max = 2)]
    /// ----
    /// 2025 return
    pub capital_history: Vec<crate::PriorCapitalReturn>,
    /// Section 1231 business-property lookback
    /// Required when business-property items are entered. Leave empty only when no section 1231 worksheet is needed; missing records are not a confirmed zero history.
    pub section1231: Option<crate::Section1231History>,
    /// Prior alternative-tax net operating losses
    pub atnol: crate::AtnolHistory,
    /// Prior regular-tax net operating losses
    /// Currently supports standard-deduction wage, interest, dividend, unemployment and operating-business returns including supported capital sales/bad debts, IRA contributions, student-loan deductions and the non-itemizer cash-charity deduction. Other combinations remain explicit for further worksheets.
    pub nol: crate::NolHistory,
    /// Prior QBI losses
    pub qbi_history: crate::QbiHistory,
    /// Foreign financial account
    /// You had a financial interest in or signature authority over a bank, securities or other financial account in a foreign country (Schedule B line 7a). Your own bank and brokerage statements show accounts located outside the United States.
    pub foreign_account: bool,
    /// Whole-return election to amortize taxable bond premium
    pub taxable_premium_election: Option<bool>,
    /// Separate FBAR filing requirement, Schedule B line 7a
    pub fbar_required: Option<bool>,
    /// Countries of accounts reportable on FBAR, Schedule B line 7b
    pub fbar_countries: Vec<i64>,
    /// Foreign trust
    /// You received a distribution from, or were a grantor of or transferor to, a foreign trust (Schedule B line 8). The trust documents and distribution statements show it.
    pub foreign_trust: bool,
    /// Earlier Forms 8915-F
    /// ----
    /// Form
    pub disaster_carries: Vec<crate::DisasterCarry>,
    /// Highest total value of your foreign financial assets during 2026
    /// Enter only with a foreign account or trust answer; absent otherwise.
    #[libertas_money("USD", 2)]
    // Bound the source amount to leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub foreign_asset_max: Option<i64>,
    /// Installment payments from earlier-year sales
    /// ----
    /// Sale
    pub installment_payments: Vec<crate::EarlierInstallment>,
    /// Net section 1256 contracts loss carried back (Form 6781 box D, line 6)
    /// Absent when no carryback is elected. One election applies to the whole 2026 return.
    #[libertas_money("USD", 2)]
    // Bound the source amount to leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub section_1256_carryback: Option<i64>,
    /// Actual owners of the absorbed section 1256 carryback loss
    /// Separate from the joint election amount and surviving carryovers. Only for Form 8379; absence leaves its owner feed unavailable without changing ordinary tax.
    pub section_1256_carryback_owners: Option<crate::InjuredRetainedOwners>,
    /// Related-party exchanges from earlier years
    /// ----
    /// Exchange
    pub related_exchanges: Vec<crate::EarlierRelatedExchange>,
    /// Individual QBI aggregation elections
    /// Retained eligibility, election history and annual disclosure facts. Business membership is selected on the actual income sources, not repeated here.
    pub qbi_aggregations: Vec<crate::QbiAggregationFacts>,
}
