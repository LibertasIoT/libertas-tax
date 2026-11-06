use crate::misc_source::JuryPay;
use crate::{
    CapitalDistributionKind, CommunityShare, ContractsSource, DividendSource, FederalBusiness,
    FederalCasualtyLivingInsurance, FederalChildElection, FederalDisasterMitigation,
    FederalDisasterRelief, FederalWildfireRelief, Form1099OidSource, GainTerm, GamblingSource,
    InterestSource, LiquidationProperty, NonbusinessBadDebt, NonbusinessPropertyKind, SaleSource,
    SavingsBondsSource, UndistributedGainSource, WorthlessSecurity,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Income document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Like the interview protocol, income documents own their complete schema payload inline.
#[allow(clippy::large_enum_variant)]
pub enum FederalIncome {
    /// Employee wages omitted from W-2 — actual payroll coverage
    EmployeeWages { data: crate::EmployeeWages },
    /// Partnership K-1 — Form 1065
    /// A complete ordinary U.S. activity can enter the return after its separate basis, at-risk, passive, self-employment, QBI and AMT checks. Identify additional or unanswered items; do not enter partnership amounts as S-corporation income.
    Partnership { data: crate::PartnerK1 },
    /// S-corporation K-1 — Form 1120-S
    /// Ordinary business and supported portfolio income, with shareholder basis, loss limits and separate AMT records. Do not use this for a partnership Form 1065 or estate/trust Form 1041 K-1; use the matching income type when available and otherwise report income not covered by the entered records. Missing or additional items require their identified worksheets before return totals can be shown.
    Scorp { data: crate::ScorpSource },
    /// Beneficiary K-1 — Form 1041
    /// Use when boxes 1–4c, optional box 5 nonbusiness royalties, boxes 6–8 business and rental income with their box 9 deductions and box 14 code I trade or business statement, limited box 10, final-year box 11 codes A–F with confirmed character, standalone box 12 code A or a reconciled B–F component set, box 13 codes A/B payments and box 14 codes A/F/H/I are the entire 2026 domestic source. Other foreign, AMT, credit, deduction, final-year and K-1 items require separate calculations.
    EstateTrustPortfolio { data: crate::EstateTrustSource },
    /// Health-service student-loan repayment assistance
    HealthLoanAssistance { data: crate::HealthLoanAssistance },
    /// Qualified 2026 wildfire relief payment excluded under section 139M
    WildfireRelief { data: FederalWildfireRelief },
    /// Qualified 2026 disaster relief payment excluded under section 139
    DisasterRelief { data: FederalDisasterRelief },
    /// Insurer-paid temporary living expenses after a 2026 main-home casualty
    CasualtyLivingInsurance { data: FederalCasualtyLivingInsurance },
    /// Qualified 2026 hazard-mitigation payment excluded under section 139(g)
    DisasterMitigation { data: FederalDisasterMitigation },
    /// Canceled debt — Form 1099-C
    PersonalDebt {
        /// Form 1099-C as printed and its topic answers
        data: crate::PersonalDebtSource,
    },
    /// Inherited IRA or Roth IRA, one for each person inherited from and each kind
    /// The Forms 1099-R that choose this record supply its distributions and withholding.
    InheritedIra { data: crate::FederalInheritedIra },
    /// Roth IRA withdrawals, Form 1099-R
    Roth {
        /// Form 1099-R as printed and its topic answers
        data: crate::R1099Source,
    },
    /// Long-term-care insurance and accelerated death benefits, Form 1099-LTC
    LtcBenefits {
        /// The form's boxes and the insured person's periods record
        data: crate::LtcSource,
    },
    /// ABLE account distributions, Form 1099-QA
    Able {
        /// The form's boxes and what the payment was
        data: crate::AbleSource,
    },
    /// Trump-account death distribution — Form 1099-R
    /// For an account beneficiary who died during the growth period, enter the death-date value and trustee basis on the acquirer's return. If the beneficiary's estate acquired it, use the beneficiary's final return instead.
    TrumpDeath {
        /// Form 1099-R as printed and its topic answers
        data: crate::R1099Source,
    },
    /// Entire Trump account transferred directly to this beneficiary's ABLE account
    /// Only during the calendar year the beneficiary turns 17; taxable income is zero and this is excluded from the ABLE annual-contribution limit.
    TrumpAbleRollover {
        /// Form 1099-R as printed and its topic answers
        data: crate::R1099Source,
    },
    /// Royalty property (Schedule E line 4)
    Royalty {
        /// Royalty property
        data: crate::FederalRoyalty,
    },
    /// Nonbusiness personal-property rent
    NonbusinessProperty {
        /// Income source
        kind: NonbusinessPropertyKind,
        /// Gross receipts, before fees
        /// Include cash and fair market value of property or services received, including Form 1099-MISC royalties or rent. Enter one royalty property or rental activity per entry.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        receipts: i64,
        /// Ordinary administration, collection, legal, agent and rental operating expenses paid
        /// Include only documented expenses attributable solely to earning income from this property. Exclude your labor, acquisition/improvement costs, interest, depreciation, amortization and depletion; these require separate treatment. Do not deduct these fees elsewhere.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
        /// Directly owned, for-profit nonbusiness property income with no special treatment
        /// Not current writing, inventing, artistic or other self-employment services; no operating mineral interest, mineral depletion, amortizable basis, patent/rights sale, partnership/trust distribution, foreign allocation, loss protection, undeclared prior suspended losses or personal hobby. All operating expenses are paid out of pocket and fully at risk; purchased rental assets are fully at risk. For rentals, customers have exclusive use of the rented property for an average of more than seven days, no significant personal services are provided, and the rental is not incidental to investment, business or development activity, not to an activity in which you materially participate, and not subject to special grouping rules. Also exclude personal-use expense allocation, prior suspended deductions outside the declared history and depreciation or basis recovery outside the declared purchased rental equipment. Business royalties or rentals belong in Business; special cases need further review. Check the Form 1099-MISC, the property's ownership papers and your expense records. No leaves this income out of this estimate.
        ordinary: bool,
    },
    /// Section 1244 small-business stock loss
    SmallBusinessStock {
        /// Issuance records and loss event
        data: crate::Section1244Stock,
    },
    /// Direct sale of section 1202 qualified small-business stock
    QualifiedSmallBusinessSale {
        /// Issuer history and original-issue lots
        data: crate::QsbIssuerSale,
    },
    /// Security that became wholly worthless
    WorthlessSecurity {
        /// Basis, acquisition and worthlessness records
        data: WorthlessSecurity,
    },
    /// Nonbusiness loan, purchased debt, paid guarantee, contractor deposit or mortgage shortfall that became wholly worthless
    NonbusinessBadDebt {
        /// Source facts, purpose and worthlessness records
        data: NonbusinessBadDebt,
    },
    /// Main home sale
    HomeSale {
        /// Closing records and home history
        data: crate::MainHomeSale,
    },
    /// Return of capital or corporate liquidation
    /// Enter one distribution allocated to one share lot, using its basis immediately before payment. Split different lots and holding periods; later payments use the remaining basis from earlier payments. Do not also enter this amount as a dividend or sale.
    ReturnOfCapital {
        /// Distribution type
        /// Use the corporation's liquidation records to establish finality; an interim payment cannot establish a loss. In complete liquidation, allocate payments among blocks by share count; in partial liquidation, allocate to redeemed shares only.
        kind: CapitalDistributionKind,
        /// Cash distribution allocated to this lot
        /// Form 1099-DIV box 3 or cash liquidation box 9 and supporting broker records; property received is listed separately.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Property received in a taxable liquidation
        /// List the portion of each asset allocated to this stock lot from Form 1099-DIV box 10 and supporting records. Leave empty for cash-only or nonliquidating distributions; never duplicate an asset's total value across stock lots.
        property: Vec<LiquidationProperty>,
        /// Adjusted basis of this lot immediately before the distribution
        /// Include all earlier basis adjustments, but not this distribution's reduction. Retain the resulting basis for later distributions and sales; do not subtract it twice from broker-adjusted basis.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Holding period on the distribution date
        term: GainTerm,
        /// Shares held for someone else, in a retirement account or a partnership or S corporation, debt the corporation assumed, a reorganization or another special treatment
        /// Yes leaves this distribution out of the estimate with a notice; that treatment is not supported.
        special: bool,
    },
    /// Undistributed fund capital gain — Form 2439
    UndistributedGain {
        /// Fund statement and its topic answers
        data: UndistributedGainSource,
    },
    /// Hobby goods sales
    HobbySales {
        /// Receipts and inventory records
        data: crate::FederalHobbySales,
    },
    /// Other income — Schedule 1 lines 8g, 8i, 8m, 8u, 8v and 8z
    OtherIncome {
        /// The item and its facts
        data: crate::OtherIncomeItem,
    },
    /// Form 1099-NEC
    Nec1099 {
        /// Form 1099-NEC as printed and where its income belongs
        data: crate::information_returns::Nec1099Source,
    },
    /// Form 1099-MISC
    Misc1099 {
        /// Form 1099-MISC as printed and where each box belongs
        data: crate::Misc1099Source,
    },
    /// Form 1099-K
    K1099 {
        /// Form 1099-K as printed and the parts of box 1a
        data: crate::information_returns::K1099Source,
    },
    /// Scholarship or fellowship
    Scholarship {
        /// Award and expense records
        data: crate::Scholarship,
    },
    /// Jury duty
    JuryDuty {
        /// Jury pay received and turned over to the employer
        data: JuryPay,
    },
    /// Casual gambling winnings and losses
    Gambling {
        /// Winnings, losses and withholding records
        data: GamblingSource,
    },
    /// Alimony received
    Alimony {
        /// Instrument and payments received
        data: crate::AlimonyPayment,
    },
    /// Foreign employment
    ForeignEmployment {
        /// Pay and exclusion election
        data: crate::ForeignWages,
    },
    /// Residential rental
    Rental {
        /// Property income and expenses
        data: crate::FederalRental,
    },
    /// W-2
    Wage {
        /// Document amounts and facts
        data: crate::WageSource,
    },
    /// Interest
    Interest {
        /// Document amounts and facts
        data: InterestSource,
    },
    /// Dividends
    Dividend {
        /// Document amounts and facts
        data: DividendSource,
    },
    /// Employer-stock sale with deferred NUA
    NuaSale { data: crate::NuaSale },
    /// Ordinary employee stock sale
    EmployeeStockSale {
        /// Acquisition, compensation and sale source records
        data: crate::EmployeeStockSale,
    },
    /// Employee stock purchase plan sale
    EsppSale {
        /// Complete internal lot used by the ESPP calculation owner
        data: crate::EsppSale,
    },
    /// Printed ESPP disposition awaiting its source projection
    EsppSaleSource {
        /// Printed sale; its personal facts belong to the investment topic
        data: crate::EsppShareSale,
    },
    /// Incentive stock option sale
    IsoSale {
        /// Complete internal lot used by the ISO calculation owner
        data: crate::IsoSale,
    },
    /// Printed ISO sale awaiting its answers
    IsoSaleSource {
        /// Printed sale and Form 3921; its personal facts belong to the investment topic
        data: crate::IsoShareSale,
    },
    /// Printed employer-stock sale with deferred NUA awaiting its answers
    NuaSaleSource {
        /// Printed sale and distribution facts; its personal facts belong to the investment topic
        data: crate::NuaShareSale,
    },
    /// Investment or personal capital-asset sale
    Sale {
        /// Form 1099-B or 1099-DA and its topic answers
        data: SaleSource,
    },
    /// Pension or IRA
    Retirement {
        /// Form 1099-R as printed and its topic answers, or the railroad annuity
        data: crate::RetirementSource,
    },
    /// Social Security or equivalent railroad benefits
    SocialSecurity {
        /// Benefit statement as printed
        data: crate::SocialBenefitStatement,
    },
    /// Government payments and state or local income tax refunds
    GovernmentPayment {
        /// One Form 1099-G as printed and its independent topic answers
        data: crate::GovernmentPaymentSource,
    },
    /// Self-employment
    Business {
        /// Document amounts and facts
        data: FederalBusiness,
    },
    /// Original issue discount
    Oid {
        /// Document amounts and facts
        data: Form1099OidSource,
    },
    /// Cooperative distributions — Form 1099-PATR
    Cooperative {
        /// Form 1099-PATR as printed
        data: crate::Patr1099Source,
    },
    /// Property a lender acquired — Form 1099-A
    PropertyAcquisition {
        /// Form 1099-A as printed and its topic answer
        data: crate::PropertyAcquisitionSource,
    },
    /// Savings bonds cashed for education — Form 8815
    SavingsBonds {
        /// Proceeds, interest, issue date and the people whose education was paid for
        data: SavingsBondsSource,
    },
    /// Child's interest and dividends (Form 8814)
    ChildElection {
        /// The child's statements and the election's facts
        data: FederalChildElection,
    },
    /// Section 1256 contracts — Form 6781 Part I
    Section1256 {
        /// Form 1099-B boxes 8 through 11 and its topic answer
        data: ContractsSource,
    },
    /// Payment from a 529 plan or Coverdell ESA — Form 1099-Q
    EducationDistribution {
        /// Boxes, what happened to the money and the student
        data: crate::EducationDistributionSource,
    },
    /// 529 and Coverdell expenses
    EducationAccountExpenses {
        /// One row for each student on this return
        data: crate::FederalEducationAccountExpenses,
    },
    /// Farm rental — Form 4835
    FarmRental {
        /// Production-share income, expenses and passive-loss facts
        data: crate::FederalFarmRental,
    },
    /// Form 1099-SA from an Archer MSA or Medicare Advantage MSA, or from an inherited account
    MedicalDistribution {
        /// The form's boxes
        data: crate::SaSource,
    },
    /// Long-term care periods of one insured person
    LtcInsured {
        /// The insured person and the periods of care
        data: crate::FederalLtcInsured,
    },
    /// Qualified opportunity fund investment made before 2026 — Form 8997
    OpportunityFund {
        /// The investment, its deferred gain and what happened to it in 2026
        data: crate::QofInvestment,
    },
    /// 2026 gain invested in a qualified opportunity fund in 2027 — Form 8997 Part II
    OpportunityFundDeferral {
        /// The fund, the date invested and the gain deferred
        data: crate::QofDeferral,
    },
    /// Partnership K-1 with actual distinct-activity statement sources.
    /// Keep the original statement controls and separate activities. Missing owner worksheets remain explicit before return totals are available.
    PartnershipActivities { data: alloc::boxed::Box<crate::PartnerActivityK1> },
    /// S-corporation K-1 with actual distinct-activity statement sources.
    /// Keep one whole-shareholding basis source and the actual separate activity records; do not enter these as another copy of the same K-1.
    ScorpActivities { data: crate::ScorpActivityK1 },
    /// Current-year investment or personal-use installment sale
    InstallmentSale { data: crate::InvestmentInstallmentSale },
    /// Printed broker source and its independent personal records
    Broker { data: crate::BrokerDocument },
    /// Form 8818 — savings bond redemption record
    BondRedemption { data: crate::BondRedemptionSource },
}

/// Income source
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalIncomeEntry {
    /// Printed recipient and issuer names
    /// [NullText]
    /// No printed names entered
    /// [SetText]
    /// Printed names
    #[libertas_default($null)]
    pub printed: Option<crate::PrintedDocumentNames>,
    /// Accepted record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Name the person typed for the source
    /// A payer, description, business, address or creditor; empty when none was typed. The document kind and identity name an unnamed source wherever it is shown.
    /// Holds the full source value: rental addresses and QSBS issuers admit 120 bytes; other named sources retain their own 80-byte limit.
    pub label: String,
    /// Income owner
    /// A document's owner is assigned only by the App: from its printed name, or in a person's section. A record's owner is the person's answer.
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub owner: crate::DocumentOwner,
    /// Community or separate
    /// How this item is divided on a community-property separate return; not needed on any other return. A rental, farm rental or Schedule K-1 is entered for the whole interest as its documents show it, carried losses and basis included; this return takes its share of every amount, and your spouse's return the rest. If your 2025 separate returns already divided it, enter the two returns' carried amounts added together.
    pub community: CommunityShare,
    /// Document type and facts
    pub income: FederalIncome,
    /// Accepted state/local boxes and source information
    /// Retained independently of the federal calculation projection.
    pub state_source: Option<crate::IncomeStateSource>,
    /// Management of the funds used for withholding
    #[libertas_default($null)]
    pub refund_management: Option<crate::RefundManagement>,
}
