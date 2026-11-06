use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Expense treatment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BusinessExpenseKind {
    /// Ordinary fully deductible business expense
    /// Exclude capital assets, home-office costs, inventory costs and expenses subject to another limit. Meals qualify here only under a documented full-deduction exception, such as eligible employee recreation.
    Ordinary,
    /// De minimis tangible-property election
    /// Deduct qualifying purchases paid in 2026 under the annual election. Enter only the business portion in Business expense amount. Do not also enter these purchases as inventory, ordinary expenses or depreciable assets.
    DeMinimis {
        /// Largest invoice or separately substantiated item cost
        /// Use the full cost before personal-use allocation. Include allocable delivery, installation and other acquisition charges on the same invoice. Do not split an item artificially to meet the limit. The total expense category may exceed this per-item limit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        largest_cost: i64,
        /// Applicable financial statement
        /// Yes means an SEC filing, a CPA-certified audited statement used for credit/shareholders or another substantial non-tax purpose, or a statement required by a federal/state agency other than the IRS or SEC. Ordinary bookkeeping or an unaudited statement is not enough.
        financial_statement: bool,
        /// Eligible book expensing and annual election confirmed
        /// Every purchase was expensed for books under a policy in place at the start of 2026 (written when there is an applicable financial statement), including any lower policy threshold or eligible 12-month useful-life policy. Confirm the election covers ALL qualifying expenditures for the year. Exclude land, inventory, special spare-parts methods and costs subject to section 263A capitalization. Attach the annual election statement to the timely original return, including extensions; retain invoices. An over-limit item needs its ordinary expense or depreciation treatment instead. The start-of-year book expensing policy and the election statement show this. No leaves this expense out of the estimate with a notice; enter it under its ordinary treatment instead.
        eligible: bool,
    },
    /// Eligible business meals, actual cost
    /// You or your employee was present, the meal had a business purpose and was not lavish. Separate meals from entertainment charges. The App applies the 50% limit. This is not the 2026 employer-convenience/facility-meal category.
    Meals,
    /// Away-from-home meals during DOT hours-of-service duty
    /// Applies only to qualifying transportation workers' meals during or incident to duty subject to Department of Transportation hours-of-service limits. The App applies the 80% limit.
    TransportationMeals,
    /// Nondeductible entertainment or other nondeductible cost
    /// Kept in the expense record but excluded from the deduction. Include ordinary 2026 employer-convenience/facility meals here unless a statutory exception applies.
    Nondeductible,
}

/// Part II expense line
/// Choose the printed line this expense belongs on. The App applies the meals/DOT/de minimis
/// treatment selected below; this only picks where the amount is reported. The farm lines are
/// for a farm's Schedule F only.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessExpenseCategory {
    /// Line 8 advertising
    Advertising,
    /// Line 10 commissions and fees
    Commissions,
    /// Line 11 contract labor
    ContractLabor,
    /// Line 14 employee benefit programs
    EmployeeBenefits,
    /// Line 15 insurance, other than health
    Insurance,
    /// Line 16a mortgage interest paid to banks or other financial institutions
    MortgageInterest,
    /// Line 16c other business interest
    OtherInterest,
    /// Line 17 legal and professional services
    LegalAndProfessional,
    /// Line 18 office expense
    Office,
    /// Line 20a rent or lease of vehicles, machinery and equipment
    VehicleAndEquipmentRent,
    /// Line 20b rent or lease of other business property
    OtherRent,
    /// Line 21 repairs and maintenance
    Repairs,
    /// Line 22 supplies not included in cost of goods sold
    Supplies,
    /// Line 23 taxes and licenses
    TaxesAndLicenses,
    /// Line 24a travel
    Travel,
    /// Line 24b deductible meals
    /// The App applies the Meals or DOT-hours-of-service treatment's limit below.
    Meals,
    /// Line 25 utilities
    Utilities,
    /// Line 26 wages, less any employment credits
    Wages,
    /// Line 27b other expenses
    Other {
        /// Description of this other expense
        #[libertas_size(min = 1)]
        label: String,
    },
    /// Farm line 11 chemicals
    Chemicals,
    /// Farm line 13 custom hire (machine work)
    CustomHire,
    /// Farm line 16 feed
    Feed,
    /// Farm line 17 fertilizers and lime
    Fertilizers,
    /// Farm line 18 freight and trucking
    Freight,
    /// Farm line 19 gasoline, fuel and oil
    Fuel,
    /// Schedule C line 19 or Schedule F line 23 employees' pension and profit-sharing plans
    /// Contributions for your own account belong under the owner retirement plan instead.
    EmployeePension,
    /// Farm line 26 seeds and plants
    Seeds,
    /// Farm line 27 storage and warehousing
    Storage,
    /// Farm line 31 veterinary, breeding and medicine
    Veterinary,
}

impl BusinessExpenseCategory {
    // Order matches the printed Schedule C line, then the farm-only Schedule F
    // lines; used only to key the per-category return-line totals below, never
    // persisted or compared across categories.
    pub(crate) const COUNT: usize = 29;
    // The first SCHEDULE_C categories have a Schedule C line (`LINES`); the
    // rest are Schedule F lines only a farm may use.
    pub(crate) const SCHEDULE_C: usize = 20;
    pub(crate) fn farm_only(&self) -> bool {
        self.index() >= Self::SCHEDULE_C
    }
    pub(crate) fn index(&self) -> usize {
        match self {
            Self::Advertising => 0,
            Self::Commissions => 1,
            Self::ContractLabor => 2,
            Self::EmployeeBenefits => 3,
            Self::Insurance => 4,
            Self::MortgageInterest => 5,
            Self::OtherInterest => 6,
            Self::LegalAndProfessional => 7,
            Self::Office => 8,
            Self::VehicleAndEquipmentRent => 10,
            Self::OtherRent => 11,
            Self::Repairs => 12,
            Self::Supplies => 13,
            Self::TaxesAndLicenses => 14,
            Self::Travel => 15,
            Self::Meals => 16,
            Self::Utilities => 17,
            Self::Wages => 18,
            Self::Other { .. } => 19,
            Self::Chemicals => 20,
            Self::CustomHire => 21,
            Self::Feed => 22,
            Self::Fertilizers => 23,
            Self::Freight => 24,
            Self::Fuel => 25,
            Self::EmployeePension => 9,
            Self::Seeds => 26,
            Self::Storage => 27,
            Self::Veterinary => 28,
        }
    }
    // (return-line id suffix, explanation) of the Schedule C categories, in
    // `index()` order.
    pub(crate) const LINES: [(&'static str, &'static str); Self::SCHEDULE_C] = [
        ("line8_advertising", "EXPLAIN_BUSINESS_N_EXPENSE_LINE8_ADVERTISING"),
        ("line10_commissions_and_fees", "EXPLAIN_BUSINESS_N_EXPENSE_LINE10_COMMISSIONS_AND_FEES"),
        ("line11_contract_labor", "EXPLAIN_BUSINESS_N_EXPENSE_LINE11_CONTRACT_LABOR"),
        (
            "line14_employee_benefit_programs",
            "EXPLAIN_BUSINESS_N_EXPENSE_LINE14_EMPLOYEE_BENEFIT_PROGRAMS",
        ),
        ("line15_insurance", "EXPLAIN_BUSINESS_N_EXPENSE_LINE15_INSURANCE"),
        ("line16a_mortgage_interest", "EXPLAIN_BUSINESS_N_EXPENSE_LINE16A_MORTGAGE_INTEREST"),
        ("line16c_other_interest", "EXPLAIN_BUSINESS_N_EXPENSE_LINE16C_OTHER_INTEREST"),
        (
            "line17_legal_and_professional",
            "EXPLAIN_BUSINESS_N_EXPENSE_LINE17_LEGAL_AND_PROFESSIONAL",
        ),
        ("line18_office", "EXPLAIN_BUSINESS_N_EXPENSE_LINE18_OFFICE"),
        ("line19_pension_plans", "EXPLAIN_BUSINESS_N_EXPENSE_LINE19_PENSION_PLANS"),
        (
            "line20a_rent_vehicles_equipment",
            "EXPLAIN_BUSINESS_N_EXPENSE_LINE20A_RENT_VEHICLES_EQUIPMENT",
        ),
        ("line20b_rent_other", "EXPLAIN_BUSINESS_N_EXPENSE_LINE20B_RENT_OTHER"),
        ("line21_repairs", "EXPLAIN_BUSINESS_N_EXPENSE_LINE21_REPAIRS"),
        ("line22_supplies", "EXPLAIN_BUSINESS_N_EXPENSE_LINE22_SUPPLIES"),
        ("line23_taxes_and_licenses", "EXPLAIN_BUSINESS_N_EXPENSE_LINE23_TAXES_AND_LICENSES"),
        ("line24a_travel", "EXPLAIN_BUSINESS_N_EXPENSE_LINE24A_TRAVEL"),
        ("line24b_meals", "EXPLAIN_BUSINESS_N_EXPENSE_LINE24B_MEALS"),
        ("line25_utilities", "EXPLAIN_BUSINESS_N_EXPENSE_LINE25_UTILITIES"),
        ("line26_wages", "EXPLAIN_BUSINESS_N_EXPENSE_LINE26_WAGES"),
        ("line27b_other", "EXPLAIN_BUSINESS_N_EXPENSE_LINE27B_OTHER"),
    ];
}

/// Business expense
/// Enter each expense or bookkeeping category once. Exclude personal expenses and expenses claimed elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBusinessExpense {
    /// Expense line
    pub category: BusinessExpenseCategory,
    /// Business expense amount
    /// Use the amount recognized under this activity's accounting method: allowable paid costs for cash accounting, or properly accrued costs for accrual accounting. Exclude expenses deferred by related-party or economic-performance rules.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Expense treatment
    pub treatment: BusinessExpenseKind,
}
impl FederalBusinessExpense {
    pub(crate) fn total(expenses: &[Self]) -> i64 {
        let mut costs = [0; 3];
        for e in expenses {
            match e.treatment {
                BusinessExpenseKind::Ordinary | BusinessExpenseKind::DeMinimis { .. } => {
                    costs[0] += e.amount
                }
                BusinessExpenseKind::Meals => costs[1] += e.amount,
                BusinessExpenseKind::TransportationMeals => costs[2] += e.amount,
                BusinessExpenseKind::Nondeductible => {}
            }
        }
        costs[0]
            + crate::kernel::ratio::fraction(costs[1], 50, 100)
            + crate::kernel::ratio::fraction(costs[2], 80, 100)
    }
    // Same per-item ordinary/meals/DOT/nondeductible treatment as `total` above,
    // applied per item so it can be grouped by printed line without changing `total`'s
    // own grouped-then-rounded result.
    pub(super) fn contribution(&self) -> i64 {
        match self.treatment {
            BusinessExpenseKind::Ordinary | BusinessExpenseKind::DeMinimis { .. } => self.amount,
            BusinessExpenseKind::Meals => crate::kernel::ratio::fraction(self.amount, 50, 100),
            BusinessExpenseKind::TransportationMeals => {
                crate::kernel::ratio::fraction(self.amount, 80, 100)
            }
            BusinessExpenseKind::Nondeductible => 0,
        }
    }
    // One total per Schedule C Part II printed line, in `BusinessExpenseCategory::LINES` order.
    pub(crate) fn category_totals(expenses: &[Self]) -> [i64; BusinessExpenseCategory::COUNT] {
        let mut totals = [0; BusinessExpenseCategory::COUNT];
        for e in expenses {
            totals[e.category.index()] += e.contribution();
        }
        totals
    }
}
