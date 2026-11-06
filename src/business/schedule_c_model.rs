// schedule_c items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub use crate::expense_source::{
    BusinessExpenseCategory, BusinessExpenseKind, FederalBusinessExpense,
};

/// Why this W2 shows no Social Security or Medicare wages
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum W2Coverage {
    /// Ordinary pay
    Ordinary,
    /// Church employee of a church that elected out of Social Security and Medicare taxes
    /// Not a minister. Box1 is ScheduleSE line5a church employee income.
    ChurchEmployee,
    /// Pay for ministerial services or services required by your religious order
    /// Box1 goes to ScheduleSE line2 unless your approved exemption applies. Enter your status once under Minister, religious worker or Form4029 exemption.
    Ministerial,
    /// Railroad retirement (RRTA) pay
    /// The railroad employer lists the RRTA compensation and taxes in box 14a. The App figures Additional Medicare Tax separately on this compensation and credits the Additional Medicare Tax withheld; Tier 1 tax counts with Social Security tax for the excess-withholding credit.
    Railroad {
        /// RRTA compensation, including reported tips
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
        /// Tier 1 tax
        /// [NullText]
        /// Tier 1 tax is blank
        /// [SetText]
        /// Tier 1 tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        tier_1: Option<i64>,
        /// Tier 2 tax, claimed separately on Form 843 for an overpayment
        /// [NullText]
        /// Tier 2 tax, claimed separately on Form 843 for an overpayment is blank
        /// [SetText]
        /// Tier 2 tax, claimed separately on Form 843 for an overpayment
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        tier_2: Option<i64>,
        /// Medicare tax, without Additional Medicare Tax
        /// [NullText]
        /// Medicare tax, without Additional Medicare Tax is blank
        /// [SetText]
        /// Medicare tax, without Additional Medicare Tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        medicare: Option<i64>,
        /// Additional Medicare Tax withheld
        /// [NullText]
        /// Additional Medicare Tax withheld is blank
        /// [SetText]
        /// Additional Medicare Tax withheld
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        additional_medicare: Option<i64>,
    },
}

/// Schedule C accounting method
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BusinessAccounting {
    /// Cash method
    Cash,
    /// Accrual method
    Accrual,
}

/// Business activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Business activities, like income documents, own their complete schema payload inline.
#[allow(clippy::large_enum_variant)]
pub enum BusinessActivity {
    /// Ordinary trade or business — Schedule C
    Ordinary {
        /// Returns and allowances (Schedule C line 2)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        returns: i64,
        /// Gross receipts under your accounting method, including reported 1099 income once
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        receipts: i64,
        /// Established accounting method for this Schedule C activity
        accounting: BusinessAccounting,
    },
    /// Farming — Schedule F
    Farming { data: crate::FarmIncome },
    /// Professional gambling
    Gambling {
        /// Returns and allowances (Schedule C line 2)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        returns: i64,
        /// Gross winning receipts, before losing wagers and expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        receipts: i64,
        /// Accounting method for this Schedule C activity
        accounting: BusinessAccounting,
        /// Documented wagering losses before the 2026 limit
        /// Do not include losing wagers again in Business expenses or net them against gross winning receipts.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        losses: i64,
        /// Federal W-2G withholding on winnings included in receipts
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        withholding: i64,
        /// Regular profit-seeking gambling business and complete reconciled records
        /// Receipts contain only winning amounts for this activity. Confirm professional status and no unresolved wager/session or prize treatment. Enter ordinary related business costs separately; the App applies their normal limits before the wagering limit. Professional status depends on whether you gamble regularly and continuously to make a profit (your records, hours and history); Yes or No is required. No leaves this business out of the estimate with a notice: enter casual gambling on the gambling page instead.
        eligible: bool,
    },
}
impl BusinessActivity {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Ordinary { receipts, returns, .. } => {
                [*receipts, *returns].iter().all(|amount| (0..=100000000000).contains(amount))
            }
            Self::Farming { data } => data.valid(),
            Self::Gambling { receipts, returns, losses, withholding, .. } => {
                (0..=*receipts).contains(returns)
                    && [*receipts, *losses, *withholding]
                        .iter()
                        .all(|v| (0..=100000000000).contains(v))
            }
        }
    }
    pub(crate) fn receipts(&self) -> i64 {
        match self {
            Self::Ordinary { receipts, returns, .. } | Self::Gambling { receipts, returns, .. } => {
                receipts - returns
            }
            Self::Farming { data } => data.gross(),
        }
    }
}

/// Business source and U.S. connection
/// For services, use where the work was performed, not the customer, bank or payer address. Mixed operations and exceptional U.S. connections need their own allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessSource {
    /// In the United States
    /// All entered business income and deductions are connected with conducting this business in the United States.
    UnitedStates,
    /// Services performed wholly in a foreign country
    ForeignServices {
        /// Covered by the foreign social security system under a totalization agreement
        /// You have, or will attach, its certificate of coverage. No U.S. self-employment tax applies to this business.
        totalization_exempt: bool,
        /// Ordinary foreign service business confirmed
        /// All services were performed outside the United States and its territories by you; capital such as inventory, equipment or property is not a material income-producing factor; this income has no effective connection to a U.S. business. No foreign business tax credit or deduction, currency adjustment, treaty benefit or special foreign treatment is requested. Receipts and expenses are recorded in U.S. dollars. Your travel and work calendar shows where services were performed. No leaves this business out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// Qualified tips received in this sole-proprietor business
/// Enter only tips separately reported to you on a 2026 information return. Include them in this business's Schedule C ordinary receipts or Schedule F other farm operating receipts once; this record claims a deduction, not more income.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBusinessTips {
    /// Qualified cash tips already included in this business's ordinary or other farm receipts
    /// A single linked information return supplies this amount automatically. When several statements overlap or serve several activities, retain only this activity's qualified, nonduplicated portion.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Occupation, voluntary-tip rules and separate information-return reporting are confirmed
    /// The tips were paid by customers for an IRS-listed occupation, not by this business or its owner; they meet the applicable 2026 specified-service rule. No claims no qualified-tips deduction for this business, with a note.
    /// Check the payment statements and records identifying qualified tips.
    pub eligible: bool,
}

/// Qualified FLSA overtime separately reported on a 2026 Form 1099
/// This applies only when the worker is covered by the FLSA overtime requirement but the payer reports the compensation on Form 1099-NEC or Form 1099-MISC for federal income-tax purposes. The premium is already part of this business's ordinary receipts.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBusinessOvertime {
    /// FLSA-required premium above the regular pay rate included in ordinary receipts
    /// A single linked information return supplies this amount automatically. With overlapping statements, retain only the qualified portion once.
    /// Use only the amount separately reported in 2026 Form 1099-NEC box 1d or Form 1099-MISC box 14. Exclude the regular rate and voluntary or state-only overtime premiums.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// FLSA coverage, required premium, 1099 reporting and no duplicate W-2 claim confirmed
    /// The same compensation is included in this business's receipts once. No claims no qualified-overtime deduction for this business, with a note.
    /// Check the payment statements and records identifying qualified overtime.
    pub eligible: bool,
}

/// Qualified cash tips: printed source or an actual qualified subset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessTipsAmount {
    /// Printed amount from the linked information return
    Printed {
        /// Printed amount
        #[libertas_read_only]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        amount: i64,
    },
    /// Qualified amount established by your records
    /// Use the nonduplicated portion for mixed occupations, overlapping statements or separately documented payments.
    QualifiedSubset {
        /// Qualified amount included in receipts once
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
/// Qualified cash tips eligibility for this activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessTipsPage {
    /// Amount established by the printed source or your qualified-subset records
    #[libertas_constrained_by("^<BusinessDetails>.tips_amount_kinds")]
    pub amount: BusinessTipsAmount,
    /// Occupation, voluntary-tip rules and separate information-return reporting are confirmed
    /// The tips were paid by customers for an IRS-listed occupation, not by this business or its owner; they meet the applicable 2026 specified-service rule. No claims no qualified-tips deduction for this business, with a note.
    /// Check the payment statements and records identifying qualified tips.
    /// Leave unanswered while those facts are unknown. A linked printed amount stays pending until Yes or No is answered; No retains the explicit decision and claims no deduction.
    /// [NullText]
    /// Eligibility not established yet
    pub eligible: Option<bool>,
}
impl From<FederalBusinessTips> for BusinessTipsPage {
    fn from(value: FederalBusinessTips) -> Self {
        Self {
            amount: BusinessTipsAmount::QualifiedSubset { amount: value.amount },
            eligible: Some(value.eligible),
        }
    }
}
impl BusinessTipsPage {
    pub(crate) fn into_record(self) -> Option<FederalBusinessTips> {
        let value = self;
        let amount = match value.amount {
            BusinessTipsAmount::Printed { amount }
            | BusinessTipsAmount::QualifiedSubset { amount } => amount,
        };
        Some(FederalBusinessTips { amount, eligible: value.eligible? })
    }
}
/// Tips source and independently answered eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessTipsEntry {
    /// Printed amount with eligibility to answer
    Printed { data: BusinessTipsPage },
    /// Optional amount established by your qualified-subset records
    Manual {
        /// [NullText]
        /// No qualified tips claimed for this activity
        /// [SetText]
        /// Enter the actual qualified tips subset
        data: Option<BusinessTipsPage>,
    },
}
impl From<Option<FederalBusinessTips>> for BusinessTipsEntry {
    fn from(value: Option<FederalBusinessTips>) -> Self {
        Self::Manual { data: value.map(Into::into) }
    }
}
impl BusinessTipsEntry {
    pub(crate) fn into_record(self) -> Option<FederalBusinessTips> {
        match self {
            Self::Printed { data } => data.into_record(),
            Self::Manual { data } => data.and_then(BusinessTipsPage::into_record),
        }
    }
}
/// Qualified FLSA overtime: printed source or an actual qualified subset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessOvertimeAmount {
    /// Printed amount from the linked information return
    Printed {
        /// Printed amount
        #[libertas_read_only]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        amount: i64,
    },
    /// Qualified amount established by your records
    /// Use the nonduplicated portion for mixed occupations, overlapping statements or separately documented payments.
    QualifiedSubset {
        /// Qualified amount included in receipts once
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
/// Qualified FLSA overtime eligibility for this activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessOvertimePage {
    /// Amount established by the printed source or your qualified-subset records
    #[libertas_constrained_by("^<BusinessDetails>.overtime_amount_kinds")]
    pub amount: BusinessOvertimeAmount,
    /// FLSA coverage, required premium, 1099 reporting and no duplicate W-2 claim confirmed
    /// The same compensation is included in this business's receipts once. No claims no qualified-overtime deduction for this business, with a note.
    /// Check the payment statements and records identifying qualified overtime.
    /// Leave unanswered while those facts are unknown. A linked printed amount stays pending until Yes or No is answered; No retains the explicit decision and claims no deduction.
    /// [NullText]
    /// Eligibility not established yet
    pub eligible: Option<bool>,
}
impl From<FederalBusinessOvertime> for BusinessOvertimePage {
    fn from(value: FederalBusinessOvertime) -> Self {
        Self {
            amount: BusinessOvertimeAmount::QualifiedSubset { amount: value.amount },
            eligible: Some(value.eligible),
        }
    }
}
impl BusinessOvertimePage {
    pub(crate) fn into_record(self) -> Option<FederalBusinessOvertime> {
        let value = self;
        let amount = match value.amount {
            BusinessOvertimeAmount::Printed { amount }
            | BusinessOvertimeAmount::QualifiedSubset { amount } => amount,
        };
        Some(FederalBusinessOvertime { amount, eligible: value.eligible? })
    }
}
/// Overtime source and independently answered eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessOvertimeEntry {
    /// Printed amount with eligibility to answer
    Printed { data: BusinessOvertimePage },
    /// Optional amount established by your qualified-subset records
    Manual {
        /// [NullText]
        /// No qualified overtime claimed for this activity
        /// [SetText]
        /// Enter the actual qualified overtime subset
        data: Option<BusinessOvertimePage>,
    },
}
impl From<Option<FederalBusinessOvertime>> for BusinessOvertimeEntry {
    fn from(value: Option<FederalBusinessOvertime>) -> Self {
        Self::Manual { data: value.map(Into::into) }
    }
}
impl BusinessOvertimeEntry {
    pub(crate) fn into_record(self) -> Option<FederalBusinessOvertime> {
        match self {
            Self::Printed { data } => data.into_record(),
            Self::Manual { data } => data.and_then(BusinessOvertimePage::into_record),
        }
    }
}
/// Sole-proprietor business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBusiness {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Actual election out of the business interest limit
    pub interest_limit_election: Option<crate::InterestLimitElection>,
    /// Claimed information-return component identities.
    pub related: Vec<i64>,
    /// Principal business or profession, including product or service
    /// Schedule C line A, for example "appraisal of real estate for lending institutions". Needed only for a printed return.
    #[libertas_default("")]
    pub principal_business: String,
    /// Business code
    /// Schedule C line B: the six-digit principal business or professional activity code from the Schedule C instructions, or 999999 when none fits. Leave empty until the return is printed.
    /// [NullText]
    /// Business code will be supplied when printing the return
    /// [SetText]
    /// The business activity code is
    // Legal tax classification: the business activity code is six digits.
    #[libertas_number(min = 100000, max = 999999)]
    #[libertas_default($null)]
    pub business_code: Option<u32>,
    /// Employer identification number
    /// Schedule C line D: the EIN issued to this business on Form SS-4, as 12-3456789; never your Social Security number. Leave empty when the business has none. Schedule 1-A prints it beside this business's qualified tips and overtime.
    // An EIN has nine digits, optionally separated by its hyphen.
    #[libertas_size(min = 0, max = 10)]
    #[libertas_default("")]
    #[libertas_pattern(r"([0-9]{2}-[0-9]{7})?")]
    pub ein: String,
    /// Business address
    /// Schedule C line E, street address, city, state and ZIP code. Leave empty when you run the business from the home at the address on your return.
    #[libertas_default("")]
    pub address: String,
    /// Started, acquired, reopened or restarted this business in 2026
    /// Schedule C line H: also Yes when you reopened it after a temporary closing and filed no 2025 Schedule C for it.
    #[libertas_default(false)]
    pub started_current_year: bool,
    /// Forms 1099 you had to file for 2026 payments
    /// Schedule C lines I and J. For 2026 a Form 1099 is required for payments of $2,000 or more to a nonemployee, and for rents, interest, royalties and other payments the Instructions for Forms 1099-MISC and 1099-NEC list.
    #[libertas_default(NoneRequired)]
    pub information_returns: InformationReturnFiling,
    /// Other business income (Schedule C line 6)
    pub other_income: Vec<crate::OtherBusinessIncome>,
    /// De minimis safe-harbor election
    #[libertas_default($null)]
    pub de_minimis: Option<crate::DeMinimisElection>,
    /// Leased vehicles deducted using actual expenses
    pub leased_vehicles: Vec<crate::BusinessLeasedVehicle>,
    /// Depletion of a working interest
    #[libertas_default($null)]
    pub depletion: Option<crate::BusinessDepletion>,
    /// Fees for ministerial services as a self-employed minister, religious-order member or Christian Science practitioner
    /// Weddings, baptisms, funerals, speeches and similar fees paid directly to you. Mark the whole ordinary business; do not mark farming or professional gambling.
    #[libertas_default(false)]
    pub ministerial: bool,
    /// Net profit from services as a notary public included in this business
    /// Exempt from self-employment tax. Enter only the net notary portion, not gross fees; zero if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub notary_profit: i64,
    /// Notary gross income already included in this business
    /// The notary share of gross income after its cost of goods, before other expenses. This is a source amount, not another receipt. An optional method with entered notary net profit needs this actual gross; without it the regular method applies. Enter the gross even when notary services produced no net profit.
    /// [NullText]
    /// No notary gross amount is supplied
    /// [SetText]
    /// Notary gross income included above is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub notary_gross: Option<i64>,

    /// Where business income is earned
    pub source: BusinessSource,
    /// Income-producing factors
    /// Distinguish the contribution of your personal services from material income-producing capital for young-filer tax rules. This is separate from QBI specified-service classification. Independent young filers can combine service-only, capital-only and mixed businesses. Each mixed activity uses its own reasonable compensation and attributable half-SE deduction; service-only profits and losses are combined separately. Capital/mixed-business dependents are supported when nonbusiness earnings, after the full half-SE deduction, already establish the maximum standard deduction; other dependent cases still need coordination.
    pub income_source: crate::BusinessIncomeSource,
    /// Activity and applicable expense limits
    pub activity: BusinessActivity,
    /// Inventory and cost of goods sold
    /// Farms use their own cash-basis or accrual inventory records; choose None here.
    pub inventory: crate::BusinessInventory,
    /// Start-up costs and their original election
    pub startup: crate::BusinessStartup,
    /// Accounting-method changes with a section 481(a) adjustment in 2026
    /// ----
    /// Change
    pub method_changes: Vec<crate::MethodChange>,
    /// Section 197 intangible assets
    /// Enter original business acquisition records, including continuing assets. These are separate from start-up costs and tangible equipment.
    pub intangibles: Vec<crate::BusinessIntangible>,
    /// Qualified timber costs and original section 194 elections
    /// Enter each direct property's expenditure-year cohort once, separate from
    /// ordinary expenses and entity statements. Missing original records and
    /// dispositions require further calculation before a complete estimate.
    pub timber: Vec<crate::BusinessTimberCost>,
    /// Other statutory amortization cost cohorts (Form4562-B)
    /// Research, pollution control, lease acquisition, preference write-offs and
    /// qualifying creative property. Keep original elections and actual history.
    pub other_amortization: Vec<crate::OtherAmortizableCost>,
    /// Owner health insurance, separate from business expenses
    pub health_insurance: crate::BusinessHealthInsurance,
    /// Owner retirement plan
    pub retirement: crate::BusinessRetirement,
    /// Vehicles using standard mileage
    /// Leave empty if none. Actual-expense vehicles require a separate worksheet.
    pub vehicles: Vec<crate::BusinessMileage>,
    /// Purchased business equipment
    /// Leave empty if none. Enter every qualifying asset once, using its recovery class and class-wide depreciation election. The App calculates the convention and retains business acquisition basis for QBI property limits.
    pub assets: Vec<crate::BusinessAsset>,
    /// Business expenses
    /// The App subtracts these expenses and calculated cost of goods sold from receipts. Do not list inventory or production costs here a second time.
    /// ----
    /// Business expense
    pub expenses: Vec<FederalBusinessExpense>,
    /// Qualified self-employed tips in this business
    /// These tips are already in Schedule C ordinary gross receipts or Schedule F other farm operating receipts. The App limits the deduction separately to this business's net income after its allocable deductions, then applies the shared annual cap and income phaseout. Leave empty when there are no qualified tips.
    pub tips: Option<FederalBusinessTips>,
    /// FLSA-qualified overtime separately reported on a 2026 Form 1099
    /// This premium is already in ordinary gross receipts. The App combines it with qualified W-2 overtime under the shared annual cap and income phaseout, without changing business profit or self-employment tax.
    pub overtime: Option<FederalBusinessOvertime>,
    /// Federal fuel tax paid on this business's qualified fuel
    /// The calculated credit is a refundable payment on Schedule 3 line 12. If full fuel cost was deducted, the App includes it in 2026 income only for an accrual-method activity. For cash-method activity, include it on the return for the year Form 4136 is filed; do not enter it in 2026 receipts merely because 2026 fuel was used.
    pub fuel_tax_credit: Option<FederalFuelTaxCredit>,
    /// Cash-method fuel-tax credit or refund included in 2026 income
    /// Record an original prior-year Form 4136 return filed in 2026, an amended prior-year credit received in 2026, or a fuel-tax refund received in 2026. Confirm either the full prior tax benefit or the narrowly documented partial 2025 benefit and avoid duplicating receipts. The 2026 return's own Form 4136 credit normally belongs in its later filing year.
    pub cash_fuel_tax_recovery: Option<FederalCashFuelRecovery>,
    /// Employer FICA credit for reported tips in an eligible business
    /// Use only for one directly owned, active U.S. food/beverage or qualifying beauty-service Schedule C employer with complete payroll records. The App computes Form 8846 and the related expense reduction from the same monthly records. Leave empty when no employer-tip credit is claimed.
    #[libertas_default($null)]
    pub employer_tip_fica_credit: Option<crate::EmployerTipFicaCredit>,
    /// Work opportunity credit for certified employees
    /// Use for wages this business paid in 2026 to employees certified for a targeted group who began work before 2026. The App computes Form 5884 and reduces the wages deduction by the credit. Leave empty when not claimed.
    #[libertas_default($null)]
    pub work_opportunity_credit: Option<crate::WorkOpportunityCredit>,
    /// Small employer retirement plan credits
    /// Use for a new employee retirement plan's start-up costs and automatic enrollment. The App computes Form 8881 Parts I and II and reduces the start-up costs deduction by that credit. Leave empty when not claimed.
    #[libertas_default($null)]
    pub retirement_plan_credit: Option<crate::RetirementPlanCredit>,
    /// Small employer health insurance premiums credit
    /// Use for premiums this business paid for its employees' SHOP Marketplace coverage. The App computes Form 8941 lines 1–16 and reduces the employee benefit programs deduction by the credit. Leave empty when not claimed.
    #[libertas_default($null)]
    pub small_employer_health_credit: Option<crate::SmallEmployerHealthCredit>,
    /// Employer credit for paid family and medical leave
    /// Use for wages this business paid to qualifying employees on family and medical leave, or premiums for a paid leave insurance policy. The App computes Form 8994 lines 1a–3 and reduces the wages and employee benefit programs deductions by the credit. Leave empty when not claimed.
    #[libertas_default($null)]
    pub paid_leave_credit: Option<crate::PaidLeaveCredit>,
    /// Qualifying work and material-participation test for this business
    pub material_participation: crate::ParticipationFacts,
    /// Specified service trade or business for the QBI deduction
    /// Includes health, law, accounting, actuarial science, performing arts, consulting, athletics, financial/brokerage services, investing/trading and dealing in securities. Architecture and engineering are excluded. Check the business’s actual services and retained Section 199A records.
    pub specified_service: bool,
    /// At-risk investment and loss limitations (Form 6198)
    pub at_risk: crate::RentalAtRisk,
    /// Employee payroll
    pub payroll: crate::BusinessPayroll,
    /// Unlisted inventory treatment, depreciation/amortization beyond the entered assets, unlisted home-office treatment, unlisted farming rules, partnership/S-corporation, foreign treatment beyond the selected source, prior losses or another special tax election applies
    /// These are your own elections, books and prior returns. Yes leaves this business out of the estimate with a notice; that treatment is not supported.
    pub special_treatment: bool,
    /// Aggregation group for the QBI deduction
    /// Leave empty unless you elect to treat this activity as part of one combined trade or business. Give the same group name to every business or rental in the group. Each member must be at least 50% owned by the same person or group for most of the year, including the last day of the year; use the same tax year; not be a specified service trade or business; and share at least two of these: the same or customary related products or services, shared facilities or centralized business elements, and coordinated or interdependent operation. An elected group must be kept in later years. The group affects only the wage and property limit above the income threshold.
    #[libertas_size(min = 1)]
    #[libertas_default($null)]
    pub qbi_aggregation_group: Option<String>,
    /// Federal income tax withheld on Forms 1099-NEC, 1099-MISC or 1099-K for this business
    #[libertas_default(0)]
    pub withholding: i64,
    /// Commercial fishing business
    /// Catching, taking, harvesting or farming fish, shellfish, crustaceans, sponges, seaweed or other aquatic life. Used for the farming and fishing estimated-tax test.
    #[libertas_default(false)]
    pub commercial_fishing: bool,
    /// Sale of the business as a whole (Form 8594)
    /// Leave empty unless you sold the whole business in 2026.
    #[libertas_default($null)]
    pub sale: Option<crate::BusinessSale>,
}

impl FederalBusiness {
    pub(crate) fn cash_accounting(&self) -> bool {
        match &self.activity {
            BusinessActivity::Ordinary { accounting, .. } => {
                *accounting == BusinessAccounting::Cash
            }
            BusinessActivity::Farming { data } => {
                matches!(data.accounting, crate::FarmAccounting::Cash { .. })
            }
            BusinessActivity::Gambling { accounting, .. } => {
                *accounting == BusinessAccounting::Cash
            }
        }
    }

    pub(crate) fn accrual_accounting(&self) -> bool {
        match &self.activity {
            BusinessActivity::Ordinary { accounting, .. } => {
                *accounting == BusinessAccounting::Accrual
            }
            BusinessActivity::Farming { data } => {
                matches!(data.accounting, crate::FarmAccounting::Accrual { .. })
            }
            BusinessActivity::Gambling { accounting, .. } => {
                *accounting == BusinessAccounting::Accrual
            }
        }
    }

    pub(crate) fn receipts(&self) -> i64 {
        self.activity.receipts()
            + self.other_income.iter().map(|v| v.amount).sum::<i64>()
            + crate::method_change::increase(&self.method_changes)
            // Schedule C line 6: recapture of excess depreciation on listed property
            // whose business use fell to 50% or less (Form 4797 Part IV).
            + self
                .assets
                .iter()
                .filter_map(|asset| asset.recapture().or_else(|| asset.section179_recapture()))
                .map(|recapture| recapture.excess())
                .sum::<i64>()
            // Schedule C line 6: a covenant is Schedule 1 line 8z, the
            // inventory sold with the business is a sale on line 1.
            + self
                .sale
                .as_ref()
                .filter(|sale| sale.supported())
                .map_or(0, |sale| sale.inventory)
            + self
                .cash_fuel_tax_recovery
                .as_ref()
                .map_or(0, |v| v.taxable_amount())
            + self.fuel_tax_credit.as_ref().map_or(0, |claim| {
                if self.accrual_accounting()
                    && claim.expense_treatment == FederalFuelExpenseTreatment::FullCostDeducted
                {
                    claim.expense_recovery_cents()
                } else {
                    0
                }
            })
            + self.fuel_tax_credit.as_ref().map_or(0, |claim| {
                if self.accrual_accounting() {
                    claim
                        .fuels
                        .card_issuer
                        .as_ref()
                        .map_or(0, crate::FederalCardIssuerClaim::retained_income_cents)
                } else {
                    0
                }
            })
    }
    pub(crate) fn se_exclusion(&self) -> i64 {
        match &self.activity {
            BusinessActivity::Farming { data } => {
                data.crp.iter().map(crate::CrpPayment::se_exclusion).sum()
            }
            _ => 0,
        }
    }
    pub(crate) fn earned_receipts(&self) -> i64 {
        self.receipts() - self.se_exclusion()
    }
    pub(crate) fn operating_cost_with_adjustment(&self, mid_quarter: bool, additional: i64) -> i64 {
        self.operating_cost_with_supplies(
            self.operating_cost_before_supplies(mid_quarter) + additional,
        )
    }
    pub(crate) fn operating_cost_with_supplies(&self, other: i64) -> i64 {
        other
            + match &self.activity {
                BusinessActivity::Farming { data } => {
                    data.supplies.as_ref().map_or(0, |s| s.amounts(other, self)[2])
                }
                _ => 0,
            }
    }
    // Schedule C line 23 beyond the taxes-and-licenses category: the employer
    // Social Security and Medicare tax on reported tips that Form 8846 does not
    // take (2026 i1040sc line 23: "Reduce your deduction by the amount shown on
    // Form 8846, line 4") and the business share of a standard-mileage
    // vehicle's personal property tax (line 9, "Standard mileage rate").
    pub(crate) fn line23_additions(&self) -> i64 {
        self.employer_tip_fica_credit
            .as_ref()
            .map_or(0, |claim| claim.remaining_fica_deduction_cents())
            + crate::BusinessMileage::property_tax(&self.vehicles)
    }
    // Schedule C Part V (line 48, carried to line 27b) beyond the Other
    // categories: the start-up cost deduction and amortization and the
    // section 197 amortization (2026 i1040sc Part V).
    pub(crate) fn checked_amortization_cents(&self) -> Option<i64> {
        crate::kernel::ratio::checked_sum(
            core::iter::once(self.startup.deduction())
                .chain(self.intangibles.iter().map(crate::BusinessIntangible::deduction))
                .chain(core::iter::once(crate::timber::deduction(&self.timber)?))
                .chain(core::iter::once(crate::cohorts::deduction(
                    &self.other_amortization,
                    &self.assets,
                )?)),
        )
    }
    pub(crate) fn amortization_cents(&self) -> i64 {
        // Native accepted-source accessor. Save validation and the estimator's
        // existing stored_issue boundary run before these operating-cost callers.
        self.checked_amortization_cents().expect("validated amortization total")
    }
    // The Form 8881 start-up credit (line 5) that reduces the deduction for
    // the plan's start-up costs (section 45E(e)(2)).
    pub(crate) fn plan_startup_credit_cents(&self) -> i64 {
        self.retirement_plan_credit.as_ref().map_or(0, |claim| claim.startup_credit() * 100)
    }
    pub(crate) fn checked_operating_cost_before_supplies(&self, mid_quarter: bool) -> Option<i64> {
        let original_amortization = crate::kernel::ratio::checked_sum(
            core::iter::once(self.startup.deduction())
                .chain(self.intangibles.iter().map(crate::BusinessIntangible::deduction)),
        )?;
        self.operating_cost_with_recovery(
            original_amortization,
            crate::kernel::ratio::checked_sum(
                self.assets.iter().map(|asset| asset.deduction(mid_quarter)),
            )?,
        )?
        .checked_add(crate::timber::deduction(&self.timber)?)?
        .checked_add(crate::cohorts::deduction(&self.other_amortization, &self.assets)?)
    }
    pub(crate) fn operating_cost_before_supplies(&self, mid_quarter: bool) -> i64 {
        self.checked_operating_cost_before_supplies(mid_quarter)
            .expect("validated operating costs including amortization")
    }
    pub(crate) fn operating_cost_with_asset_deductions(&self, depreciation: i64) -> Option<i64> {
        self.operating_cost_with_recovery(self.checked_amortization_cents()?, depreciation)
    }
    pub(super) fn operating_cost_with_recovery(
        &self,
        amortization: i64,
        depreciation: i64,
    ) -> Option<i64> {
        let other = (match &self.activity {
            BusinessActivity::Farming { data } => {
                data.conservation.as_ref().map_or(0, |c| {
                    c.deduction(data.gross() + crate::method_change::increase(&self.method_changes))
                }) + data.ccc.as_ref().map_or(0, crate::FarmCcc::deficiency)
                    + data.specials.fertilizer_deduction()
                    + data.specials.production_deduction(matches!(data.accounting,crate::FarmAccounting::Cash {..}),false)
            }
            _ => 0,
        }) + FederalBusinessExpense::total(&self.expenses)
            // Ordinary expenses omit employer FICA on reported tips. Section
            // 45B allows only the tax not claimed on Form 8846.
            + self.employer_tip_fica_credit.as_ref().map_or(0, |claim| {
                claim.remaining_fica_deduction_cents()
            })
            - self.work_opportunity_credit.as_ref().map_or(0, |claim| claim.credit() * 100)
            - self.plan_startup_credit_cents()
            - self
                .small_employer_health_credit
                .as_ref()
                .map_or(0, |claim| claim.credit(&self.payroll) * 100)
            - self.paid_leave_credit.as_ref().map_or(0, |claim| claim.credit() * 100)
            + self.inventory.cost()
            + self.depletion.as_ref().map_or(0, |v| match v.depletion {
                crate::RoyaltyDepletion::Cost { adjusted_basis, recoverable_units, units_sold }
                    if recoverable_units > 0 =>
                {
                    crate::kernel::ratio::fraction(adjusted_basis, units_sold, recoverable_units)
                }
                _ => 0,
            })
            + self.de_minimis.as_ref().map_or(0, |v| v.purchases)
            + self.leased_vehicles.iter().map(|v| v.amounts().total()).sum::<i64>()
            + crate::method_change::decrease(&self.method_changes)
            + crate::BusinessMileage::total(&self.vehicles)
            + self.assets.iter().map(crate::BusinessAsset::vehicle_operating).sum::<i64>()
            + self.assets.iter().map(crate::BusinessAsset::vehicle_loan_interest).sum::<i64>()
            + depreciation;
        other.checked_add(amortization)
    }
    pub(crate) fn expense_total(&self, mid_quarter: bool) -> i64 {
        self.expense_with_adjustment(mid_quarter, 0)
    }
    pub(crate) fn expense_with_adjustment(&self, mid_quarter: bool, additional: i64) -> i64 {
        self.expense_from_operating(self.operating_cost_with_adjustment(mid_quarter, additional))
    }
    pub(crate) fn expense_from_operating(&self, operating: i64) -> i64 {
        self.expense_with_office(operating, 0)
    }
    // Operating costs plus the Schedule C line 30 amount already figured.
    pub(crate) fn expense_with_office(&self, operating: i64, office: i64) -> i64 {
        let expenses = operating + office;
        match self.activity {
            BusinessActivity::Ordinary { .. } | BusinessActivity::Farming { .. } => expenses,
            BusinessActivity::Gambling { losses, .. } => phase::cap(
                self.receipts(),
                [crate::kernel::ratio::fraction(losses + expenses, 90, 100)],
            ),
        }
    }
}

/// Self employment document
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
pub enum SelfEmploymentDocument {
    /// 1099-NEC
    /// One for each Form 1099-NEC addressed to you or your spouse. Pay from a firm that should have treated you as its employee is Employee pay without a W-2 (Form 8919) under Wages; Medicaid waiver payments for care in your home are on Other income; a farm's forms are entered on its Farming lines.
    Form1099Nec(crate::Form1099NecPaper),
    /// 1099-K
    /// One for each Form 1099-K from a payment app, online marketplace or card processor addressed to you or your spouse. Split box 1a by where its payments belong; the App adds each part there. A farm's forms are entered on its Farming lines.
    Form1099K(crate::Form1099KPaper),
}

/// Business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Business {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Actual election out of the business interest limit
    /// [NullText]
    /// No election out of the business interest limit
    /// [SetText]
    /// Election record
    #[libertas_constrained_by("^.allowed_interest_limit_election")]
    pub interest_limit_election: Option<crate::InterestLimitElectionRecord>,
    /// Applicable activity election record
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_interest_limit_choices")]
    pub allowed_interest_limit_election: Vec<i32>,
    /// Related forms
    /// Select this business's information returns. Open a related form to review its printed state rows; edit those rows on the original form in Documents. The linked form does not add receipts a second time.
    pub related: crate::RelatedForms,
    /// Principal business or profession, including product or service
    /// Schedule C line A, for example "appraisal of real estate for lending institutions". Needed only for a printed return.
    #[libertas_default("")]
    pub principal_business: String,
    /// Business code
    /// Schedule C line B: the six-digit principal business or professional activity code from the Schedule C instructions, or 999999 when none fits. Leave empty until the return is printed.
    /// [NullText]
    /// Business code will be supplied when printing the return
    /// [SetText]
    /// The business activity code is
    // Legal tax classification: the business activity code is six digits.
    #[libertas_number(min = 100000, max = 999999)]
    #[libertas_default($null)]
    pub business_code: Option<u32>,
    /// Employer identification number
    /// Schedule C line D: the EIN issued to this business on Form SS-4, as 12-3456789; never your Social Security number. Leave empty when the business has none. Schedule 1-A prints it beside this business's qualified tips and overtime.
    // An EIN has nine digits, optionally separated by its hyphen.
    #[libertas_size(min = 0, max = 10)]
    #[libertas_default("")]
    #[libertas_pattern(r"([0-9]{2}-[0-9]{7})?")]
    pub ein: String,
    /// Business address
    /// Schedule C line E, street address, city, state and ZIP code. Leave empty when you run the business from the home at the address on your return.
    #[libertas_default("")]
    pub address: String,
    /// Started, acquired, reopened or restarted this business in 2026
    /// Schedule C line H: also Yes when you reopened it after a temporary closing and filed no 2025 Schedule C for it.
    #[libertas_default(false)]
    pub started_current_year: bool,
    /// Forms 1099 you had to file for 2026 payments
    /// Schedule C lines I and J. For 2026 a Form 1099 is required for payments of $2,000 or more to a nonemployee, and for rents, interest, royalties and other payments the Instructions for Forms 1099-MISC and 1099-NEC list.
    #[libertas_default(NoneRequired)]
    pub information_returns: InformationReturnFiling,
    /// Line 2 returns and allowances
    /// Refunds, rebates, allowances and credits you gave customers in 2026 on the sales in line 1, as a positive amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub returns: i64,
    /// Line 6 other business income
    /// Income of this business that is not a sale or service receipt: finance reserve income, scrap sales, a bad debt you deducted in an earlier year and recovered, interest on notes and accounts receivable, business prizes and awards, Form 1099-PATR amounts from business purchases, and sales tax a state let you keep. The App adds the fuel-tax credit, a positive section 481(a) adjustment and listed-property recapture itself; do not enter those.
    #[libertas_unordered]
    /// ----
    /// Other income
    pub other_income: Vec<OtherBusinessIncome>,

    /// Business name
    /// Schedule C line C. Leave empty to use your name.
    #[libertas_default("")]
    pub name: String,
    /// Line 1 gross receipts not on a Form 1099 you entered
    /// Receipts under your accounting method before returns and allowances, leaving out each Form 1099-NEC and each 1099-MISC box or 1099-K part related to this business from forms you entered in Documents; the App adds those.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub receipts: i64,
    /// Accounting method
    #[libertas_default(Cash)]
    pub accounting: BusinessAccounting,
    /// Part II expenses
    pub expenses: BusinessExpenses,
    /// Vehicles
    #[libertas_unordered]
    /// ----
    /// Vehicle
    pub vehicles: Vec<Vehicle>,
    /// Equipment and other property
    #[libertas_unordered]
    /// ----
    /// Asset
    pub assets: Vec<Asset>,
    /// Health, law, accounting, consulting, athletics, performing arts or financial services business
    /// A specified service trade or business for the qualified business income deduction; architecture and engineering are not. Check the business’s actual services and retained Section 199A records; the classification matters above the income threshold.
    pub specified_service: bool,
    /// More about this business
    pub more: BusinessDetails,
}

/// Part II expenses
/// Enter each line from your records; leave the others at zero. Car and truck expenses (line 9) and depreciation (line 13) come from the vehicles and property below. Home use (line 30) comes from the linked home in Business use of your homes.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessExpenses {
    /// Line 19 pension and profit-sharing plans
    /// Contributions you made for your employees to a pension, profit-sharing, SEP, SIMPLE or annuity plan: for each employee at most 25% of their compensation (SEP, profit-sharing) and $72,000, counting compensation up to $360,000, and the SIMPLE match or 2% nonelective amount. Your own contributions as a self-employed person go on Self-employed benefits, not here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub pension_plans: i64,

    /// Line 8 advertising
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub advertising: i64,
    /// Line 10 commissions and fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub commissions: i64,
    /// Line 11 contract labor
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contract_labor: i64,
    /// Line 14 employee benefit programs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employee_benefits: i64,
    /// Line 15 insurance, other than health
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub insurance: i64,
    /// Line 16a mortgage interest paid to banks
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub mortgage_interest: i64,
    /// Line 16c other interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_interest: i64,
    /// Line 17 legal and professional services
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub legal_and_professional: i64,
    /// Line 18 office expense
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub office: i64,
    /// Line 20a rent or lease of vehicles, machinery and equipment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub vehicle_and_equipment_rent: i64,
    /// Line 20b rent or lease of other business property
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rent: i64,
    /// Line 21 repairs and maintenance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub repairs: i64,
    /// Line 22 supplies
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub supplies: i64,
    /// Line 23 taxes and licenses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxes_and_licenses: i64,
    /// Line 24a travel
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub travel: i64,
    /// Line 24b meals
    /// Each kind once, at full cost or the standard meal allowance (GSA.gov/perdiem); the App applies the 50%, 80% or 100% rule. You or an employee was present and the meal was not lavish. Leave out entertainment, meals at an employer-operated eating facility and meals furnished for your convenience as the employer (no deduction after 2025).
    // Tax structure: one entry for each of the three meal deduction treatments.
    #[libertas_size(max = 3)]
    #[libertas_unordered]
    /// ----
    /// Meals
    pub meals: Vec<MealExpense>,

    /// Line 25 utilities
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub utilities: i64,
    /// Line 26 wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wages: i64,
    /// Line 27b other expenses
    #[libertas_unordered]
    /// ----
    /// Other expense
    pub other: Vec<OtherExpense>,
}

/// Other expense
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherExpense {
    /// Description
    #[libertas_size(min = 1)]
    pub description: String,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}

/// Vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Vehicle {
    /// How you deduct this vehicle
    #[libertas_default(StandardMileage)]
    pub method: VehicleMethod,
}

/// How you deduct this vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum VehicleMethod {
    /// Leased, actual expenses
    /// Lease payments and operating costs for a vehicle leased at least 30 days; not the standard-mileage method.
    Leased { vehicle: crate::BusinessLeasedVehicle },
    /// Standard mileage rate
    StandardMileage {
        /// Canonical vehicle identity assigned on Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        id: i64,
        /// Business distance, January 1 – June 30
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("mile")]
        first_half: u32,
        /// Business distance, July 1 – December 31
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("mile")]
        second_half: u32,
        /// Total distance driven in 2026
        /// Business, commuting and personal driving together, from your driving log or odometer readings.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("mile")]
        total: u32,
        /// Other vehicle facts
        more: MileageMore,
    },
    /// Actual expenses
    /// You own this car, van or truck and deduct its operating costs and depreciation instead of the mileage rate.
    #[libertas_calculation_root("VehicleAmtCarryOptions")]
    Actual {
        /// Supplied recovery records retained with this actual vehicle
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default($null)]
        macrs_source: Option<alloc::boxed::Box<crate::MacrsRecoverySource>>,
        /// Canonical property identity assigned on Save
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        id: i64,
        /// Original schedules and annual use needed for historical recovery
        /// [NullText]
        /// No changed-use or original annual schedule is supplied
        /// [SetText]
        /// The original schedule and annual records are
        #[libertas_default($null)]
        history_details: Option<alloc::boxed::Box<crate::HistoricalAssetRecords>>,
        /// Purchase cost
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        cost: i64,
        /// Credit that reduces its cost
        /// Leave zero unless a tax credit was taken for this vehicle. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) reduces its cost by the whole credit. Enter the reduction itself.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        credit_reduction: i64,
        /// Date bought, if before it was placed in service
        /// Use the acquisition date from the original purchase records; it sets bonus eligibility and the applicable historical automobile limit.
        /// [NullText]
        /// Bought on the date placed in service
        /// [SetText]
        /// Bought earlier on
        #[libertas_date_only]
        acquired: Option<u32>,
        /// Date placed in service
        #[libertas_date_only]
        placed: u32,
        /// Total distance driven in 2026
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("mile")]
        total_miles: u32,
        /// Business distance in 2026
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("mile")]
        business_miles: u32,
        /// Fuel, insurance, repairs and other operating costs for the whole vehicle
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        operating_costs: i64,
        /// Annual vehicle-loan interest paid
        /// Enter the whole vehicle amount; the App allocates its business-mile share to the loan-interest line. Zero when there is no loan.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        loan_interest: i64,
        /// Vehicle design and applicable legal cap
        kind: crate::VehicleKind,
        /// Business use in earlier years, when it was more than 50%
        /// Only for a vehicle placed in service before 2026 whose business use this year is 50% or less. Enter its whole-number business use in the earlier years, the same in each. For a listed vehicle, the App adds depreciation allowed above straight-line ADS to other business income (Form 4797 Part IV column (b), section 280F(b)(2)). A qualified nonpersonal-use vehicle follows the ordinary section 179 recapture rules in column (a), without listed-property ADS. Choose “Business use did not fall from above 50% to 50% or less” if it was used 50% or less in every earlier year.
        /// [NullText]
        /// Business use did not fall from above 50% to 50% or less
        /// [SetText]
        /// The earlier business use above 50% was
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 51, max = 100)]
        #[libertas_physical_unit("percent")]
        earlier_business_percent: Option<u32>,
        /// Depreciation history
        #[libertas_default(New)]
        #[libertas_calculation_request("VehicleAmtCarryOptions")]
        history: AssetHistory,
        /// Sold or stopped using in 2026
        #[libertas_default(No)]
        disposal: crate::AssetDisposal,
        /// Actual independently reported AMT unused section179 carry
        /// Complete original annual records supply it without another answer.
        /// [NullText]
        /// No separately supplied actual AMT carry
        /// [SetText]
        /// Actual AMT unused elected-cost carry
        #[libertas_default($null)]
        #[libertas_constrained_by("^<VehicleMethod>.Actual.allowed_amt_carry")]
        amt_section179_income_carry: Option<alloc::boxed::Box<crate::AssetAmt179CarryAnswer>>,
        /// Applicable actual AMT source answer; a new ordinary row starts empty.
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_amt_carry: Vec<i32>,
    },
}

/// Other vehicle facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MileageMore {
    /// Date placed in service for business (Schedule C Part IV line 43)
    /// [NullText]
    /// Date placed in service will be supplied when printing the return
    /// [SetText]
    /// The vehicle was placed in service for business on
    #[libertas_date_only]
    #[libertas_default($null)]
    pub placed: Option<u32>,
    /// Available for personal use during off-duty hours (line 45)
    #[libertas_default(true)]
    pub personal_use_available: bool,
    /// Another vehicle available for personal use (line 46)
    #[libertas_default(true)]
    pub other_vehicle: bool,
    /// Evidence supporting the vehicle deduction (lines 47a and 47b)
    #[libertas_default(Written)]
    pub evidence: crate::VehicleEvidence,

    /// Commuting distance
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    #[libertas_physical_unit("mile")]
    pub commuting: u32,
    /// Business parking and tolls, excluding commuting
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub parking_tolls: i64,
    /// Annual vehicle-loan interest paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Annual value-based personal property tax on the vehicle
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub property_tax: i64,
    /// Leased, not owned
    #[libertas_default(false)]
    pub leased: bool,
}

impl Default for MileageMore {
    fn default() -> Self {
        Self {
            placed: None,
            personal_use_available: true,
            other_vehicle: true,
            evidence: crate::VehicleEvidence::Written,
            commuting: 0,
            parking_tolls: 0,
            interest: 0,
            property_tax: 0,
            leased: false,
        }
    }
}

/// Asset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("AssetAmtCarryOptions,NonMacrsAssetCarryOptions")]
pub struct Asset {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    #[libertas_constrained_by("^<Asset>.allowed_farm_rental_use")]
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,
    /// Applicable property-use choices supplied by the Farming editor
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_farm_rental_use: Vec<i32>,

    /// Supplied MACRS-only records retained when the property has another recovery treatment
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub macrs_source: Option<crate::MacrsRecoverySource>,
    /// Canonical property identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Original schedules and annual use needed for historical recovery
    /// [NullText]
    /// No changed-use or original annual schedule is supplied
    /// [SetText]
    /// The original schedule and annual records are
    pub history_details: Option<crate::HistoricalAssetRecords>,
    /// Cost, including delivery and installation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost: i64,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) reduces its cost by the whole credit. Enter the reduction itself. When you choose the investment credit below, the App figures its reduction (the whole rehabilitation credit, half of an energy credit) and ignores this amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Date placed in service
    #[libertas_date_only]
    pub placed: u32,
    /// Original lawful recovery system and its source records
    pub recovery: crate::AssetRecoveryPage,
    /// Business use
    /// For a continuing asset whose use fell to 50% or less this year, enter this year's use here and the earlier use below.
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 1, max = 100)]
    #[libertas_default(100)]
    #[libertas_physical_unit("percent")]
    pub business_percent: u32,
    /// Business use in earlier years, when it was more than 50%
    /// Only for an asset placed in service before 2026 whose business use this year is 50% or less. Enter its whole-number business use in the earlier years, the same in each. The App adds back the section 179 benefit (Form 4797 Part IV column (a)) as other income of the business, and raises the asset's basis by it. Choose “Business use did not fall from above 50% to 50% or less” if it was used 50% or less in every earlier year.
    /// [NullText]
    /// Business use did not fall from above 50% to 50% or less
    /// [SetText]
    /// The earlier business use above 50% was
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 51, max = 100)]
    #[libertas_physical_unit("percent")]
    pub earlier_business_percent: Option<u32>,
    /// Date bought, if before it was placed in service
    /// This date sets the bonus depreciation rate.
    /// [NullText]
    /// Bought on the date placed in service
    /// [SetText]
    /// Bought earlier on
    #[libertas_date_only]
    pub acquired: Option<u32>,
    /// Sold or stopped using in 2026
    #[libertas_default(No)]
    pub disposal: crate::AssetDisposal,
    /// Investment credit (Form 3468)
    /// For a certified historic building's rehabilitation, or solar, wind, energy storage or geothermal heat pump equipment.
    #[libertas_default(No)]
    pub investment_credit: crate::InvestmentCredit,
    /// Part of this building replaced or removed in 2026
    /// Include partial sales and other mandatory component dispositions. For replacement or removal, enter the component when electing separate partial-disposition treatment; otherwise choose no separately recognized disposition.
    /// [NullText]
    /// No building-component disposition is recognized separately in 2026
    /// [SetText]
    /// The separately recognized building-component disposition is
    pub partial: Option<crate::PartialDisposition>,
    /// Actual independently reported AMT unused section179 carry
    /// Complete original annual records supply it without another answer.
    /// [NullText]
    /// No separately supplied actual AMT carry
    /// [SetText]
    /// Actual AMT unused elected-cost carry
    #[libertas_constrained_by("^<Asset>.allowed_amt_carry")]
    pub amt_section179_income_carry: Option<crate::AssetAmt179CarryAnswer>,
    /// Applicable actual AMT source answer; a new ordinary row starts empty.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_amt_carry: Vec<i32>,
}

/// Kind of property
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AssetClass {
    /// Computers, office machines and light vehicles (5-year)
    Five,
    /// Office furniture and most equipment (7-year)
    Seven,
    /// Tractor units and similar (3-year)
    Three,
    /// Vessels, barges and similar (10-year)
    Ten,
    /// Land improvements such as fences and paving (15-year)
    Fifteen,
    /// Farm buildings and utility property (20-year)
    Twenty,
    /// Building or structural component (39-year)
    ThirtyNine,
}

/// Depreciation history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AssetHistory {
    /// Placed in service in 2026
    New {
        /// First-year deduction
        #[libertas_default(Bonus)]
        first_year: FirstYearDeduction,
    },
    /// Placed in service before 2026
    Continuing {
        /// Depreciation taken through 2025
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_depreciation: i64,
        /// Section 179 taken in its first year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        first_year_179: i64,
        /// Bonus depreciation taken in its first year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        first_year_bonus: i64,
        /// Unused section 179 carried to 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        carryforward: i64,
        /// Mid-quarter convention in its first year
        #[libertas_default(false)]
        mid_quarter: bool,
    },
}

/// First-year deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FirstYearDeduction {
    /// Bonus depreciation
    /// 100% for property bought after January 19, 2025.
    Bonus,
    /// Section 179 expense
    Section179 {
        /// Description (Form 4562 line 6)
        #[libertas_size(min = 1)]
        description: String,
        /// Cost you elect to expense
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        elected: i64,
    },
    /// Regular depreciation only
    /// Elects out of bonus depreciation for all 2026 property of this kind.
    Regular,
}

/// More about this business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessDetails {
    /// The actual source determines whether the amount is printed or independently allocated
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.source_choices.manual_work_amount_kinds")]
    pub tips_amount_kinds: Vec<i32>,
    /// The actual source determines whether the amount is printed or independently allocated
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_copy_from("$.source_choices.manual_work_amount_kinds")]
    pub overtime_amount_kinds: Vec<i32>,
    /// Actual saved intangible agreements and rights in this business
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_intangible_predecessors")]
    pub intangible_predecessor_choices: Vec<RecordChoice>,
    /// Actual saved property in this business
    #[libertas_hidden]
    #[libertas_read_only]
    pub research_property_choices: Vec<RecordChoice>,
    /// De minimis safe-harbor election
    /// Choose whether you made this annual election; do not include its purchases again as ordinary expenses or assets.
    /// [NullText]
    /// No de minimis election was made for 2026
    /// [SetText]
    /// The de minimis election made for 2026 is
    pub de_minimis: Option<crate::DeMinimisElection>,
    /// FLSA overtime separately reported on a 2026 Form 1099
    /// The printed amount is shown when one linked source establishes it. Otherwise enter the actual qualified subset. Its premium is already included in this business's receipts; answer eligibility separately, including No when it does not qualify.
    #[libertas_constrained_by("^<BusinessDetails>.overtime_amount_kinds")]
    pub overtime: crate::BusinessOvertimeEntry,
    /// Depletion of an oil, gas or mineral working interest
    /// Choose whether this business claims depletion of its one property.
    /// [NullText]
    /// No depletion deduction is claimed for this business
    /// [SetText]
    /// The depletion claim for this property is
    pub depletion: Option<crate::BusinessDepletion>,
    /// Fees for ministerial services as a self-employed minister, religious-order member or Christian Science practitioner
    /// Weddings, baptisms, funerals, speeches and similar fees paid directly to you. Mark the whole ordinary business; do not mark farming or professional gambling.
    #[libertas_default(false)]
    pub ministerial: bool,
    /// Net profit from services as a notary public included in this business
    /// Exempt from self-employment tax. Enter only the net notary portion, not gross fees; zero if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub notary_profit: i64,
    /// Notary gross income already included in this business
    /// The notary share of gross income after its cost of goods, before other expenses. This is a source amount, not another receipt. An optional method with entered notary net profit needs this actual gross; without it the regular method applies. Enter the gross even when notary services produced no net profit.
    /// [NullText]
    /// No notary gross amount is supplied
    /// [SetText]
    /// Notary gross income included above is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub notary_gross: Option<i64>,

    /// Inventory and cost of goods sold
    #[libertas_default(None)]
    pub inventory: crate::BusinessInventory,
    /// Start-up costs
    #[libertas_default(None)]
    pub startup: crate::BusinessStartup,
    /// Accounting-method changes (Form 3115)
    /// Only a change whose section 481(a) adjustment still has an amount in 2026.
    #[libertas_unordered]
    /// ----
    /// Change
    pub method_changes: Vec<crate::MethodChange>,
    /// Business intangible agreements and property (Form 4562-B)
    /// Enter actual acquisition facts. Software and excluded finite-life rights retain their normal property editors; enter each cost once.
    #[libertas_unordered]
    /// ----
    /// Intangible
    pub intangibles: Vec<crate::BusinessIntangiblePage>,
    /// Qualified timber costs and original section 194 elections
    /// Enter each direct property's expenditure-year cohort once, separate from
    /// ordinary expenses and entity statements. Missing original records and
    /// dispositions require further calculation before a complete estimate.
    pub timber: Vec<crate::BusinessTimberCost>,
    /// Other statutory amortization cost cohorts (Form4562-B)
    /// Research, pollution control, lease acquisition, preference write-offs and
    /// qualifying creative property. Keep original elections and actual history.
    pub other_amortization: Vec<crate::OtherAmortizableCostPage>,
    /// Employees' W-2 wages
    #[libertas_default(No)]
    pub payroll: crate::BusinessPayroll,
    /// Work opportunity credit (Form 5884)
    /// For this business's employees; it needs their W-2s under Employees' W-2 wages. The App carries any credit it cannot use this year through Form 3800.
    /// [NullText]
    /// No work opportunity credit claimed for this business
    /// [SetText]
    /// Claim the work opportunity credit using
    #[libertas_default($null)]
    pub work_opportunity: Option<crate::WorkOpportunityCreditPage>,
    /// Small employer retirement plan credits (Form 8881)
    /// For this business's employees; it needs their W-2s under Employees' W-2 wages. The App carries any credit it cannot use this year through Form 3800.
    /// [NullText]
    /// No small employer retirement plan credit claimed for this business
    /// [SetText]
    /// Claim small employer retirement plan credits using
    #[libertas_default($null)]
    pub retirement_plan: Option<crate::RetirementPlanCredit>,
    /// Employer Social Security and Medicare taxes on tips (Form 8846)
    /// For this business's employees; it needs their W-2s under Employees' W-2 wages. The App carries any credit it cannot use this year through Form 3800.
    /// [NullText]
    /// No employer tip FICA credit claimed for this business
    /// [SetText]
    /// Claim the employer tip FICA credit using
    #[libertas_default($null)]
    pub tip_fica: Option<crate::EmployerTipFicaCreditPage>,
    /// Small employer health insurance credit (Form 8941)
    /// For this business's employees' SHOP Marketplace coverage; it needs their W-2s under Employees' W-2 wages. The App carries any credit it cannot use this year through Form 3800.
    /// [NullText]
    /// No small employer health insurance credit claimed for this business
    /// [SetText]
    /// Claim the small employer health insurance credit using
    #[libertas_default($null)]
    pub small_employer_health: Option<crate::SmallEmployerHealthCreditPage>,
    /// Employer credit for paid family and medical leave (Form 8994)
    /// For leave wages or paid leave insurance premiums of this business's employees; it needs their W-2s under Employees' W-2 wages. The App carries any credit it cannot use this year through Form 3800.
    /// [NullText]
    /// No employer paid family and medical leave credit claimed for this business
    /// [SetText]
    /// Claim the employer paid leave credit using
    #[libertas_default($null)]
    pub paid_leave: Option<crate::PaidLeaveCreditPage>,
    /// Federal fuel tax credit (Form 4136)
    /// Federal excise tax on fuel this business used off-highway, on a farm or in another nontaxable use. The credit is a payment on Schedule 3 line 12.
    /// [NullText]
    /// No federal fuel tax credit claimed for this business
    /// [SetText]
    /// Claim the federal fuel tax credit using
    #[libertas_default($null)]
    pub fuel_tax_credit: Option<crate::FederalFuelTaxCredit>,
    /// Fuel tax credit or refund from an earlier year, received in 2026
    /// For a cash-method business: an earlier year's Form 4136 credit claimed on a return filed in 2026, or a fuel-tax refund received in 2026, when the fuel's full cost was deducted.
    /// [NullText]
    /// No earlier-year fuel tax credit or refund was received in 2026
    /// [SetText]
    /// The earlier-year fuel tax credit or refund received in 2026 was
    pub cash_fuel_tax_recovery: Option<crate::FederalCashFuelRecovery>,
    /// Qualified tips included in this business's receipts (Schedule 1-A)
    /// Only tips separately reported to you on a 2026 information return and already inside this business's gross receipts. The App deducts them on Schedule 1-A, within the annual cap and income phaseout, and leaves them out of the qualified business income deduction. The printed amount is shown when one linked source establishes it; otherwise enter the actual qualified subset. Answer eligibility separately, including No when it does not qualify; the receipts remain taxable.
    #[libertas_constrained_by("^<BusinessDetails>.tips_amount_kinds")]
    pub tips: crate::BusinessTipsEntry,
    /// Qualifying work and material-participation test for this business
    pub material_participation: crate::ParticipationFacts,
    /// At-risk investment and loss limitations (Form 6198)
    /// Use your investment, financing and prior-return records. A current ordinary business can compute its limited investment; prior losses must retain their original form and character.
    pub at_risk: crate::RentalAtRisk,
    /// Group this business with others for the qualified business income deduction
    #[libertas_default("")]
    pub qbi_group: String,
    /// What produces this business's income
    /// Asked only when the tax on a child's unearned income (Form 8615) can apply to this return.
    #[libertas_constrained_by("$.allowed_business_source")]
    pub income_source: crate::BusinessIncomeChoice,
    /// Gambling as a business
    /// You gamble regularly and continuously to make a profit, with records, hours and a profit history. Line 1 gross receipts are then your gross winnings, each W-2G for this activity included once.
    /// [NullText]
    /// This is not a professional gambling business
    /// [SetText]
    /// This is a professional gambling business with these facts
    pub gambling: Option<crate::ProfessionalGambling>,
    /// Where you did this work
    /// Choose services performed wholly in a foreign country for a business you ran abroad, then claim the exclusion and housing on Work abroad (Form 2555).
    #[libertas_default(UnitedStates)]
    pub source: crate::BusinessSource,
    /// Timber depletion (Form T), a Form 7205 deduction or another unlisted special treatment applies
    #[libertas_default(false)]
    pub special: bool,
    /// Commercial fishing business
    /// Catching, taking, harvesting or farming fish, shellfish, crustaceans, sponges, seaweed or other aquatic life. Used for the farming and fishing estimated-tax test.
    #[libertas_default(false)]
    pub commercial_fishing: bool,
    /// Sale of the business
    /// Fill in only if you sold the whole business in 2026 (Form 8594). Each asset, vehicle and building sold is entered on its own row above.
    /// [NullText]
    /// The whole business was not sold in 2026
    /// [SetText]
    /// The whole business was sold with these allocation facts
    pub sale: Option<crate::BusinessSale>,
}

impl From<&W2Coverage> for crate::W2Payroll {
    fn from(value: &W2Coverage) -> Self {
        match value {
            W2Coverage::Railroad {
                compensation,
                tier_1,
                tier_2,
                medicare,
                additional_medicare,
            } => Self::Railroad {
                compensation: *compensation,
                tier_1: tier_1.unwrap_or(0),
                tier_2: tier_2.unwrap_or(0),
                medicare: medicare.unwrap_or(0),
                additional_medicare: additional_medicare.unwrap_or(0),
            },
            _ => Self::SocialSecurity,
        }
    }
}
