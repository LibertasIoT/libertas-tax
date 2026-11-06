// Existing Unit19 individual rental/4835 risk and source-history cohort.
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Amount at risk for this activity (Form 6198)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ActivityRiskRecord {
    /// Regular income-tax computation
    pub regular: RiskMethod,
    /// Alternative minimum-tax computation
    pub amt: RiskAmtMethod,
    /// Financing included in this computation
    pub financing: Vec<RiskFinancing>,
    /// Earlier deducted losses and recapture
    /// If the actual detailed closing balance is negative, supply the history needed to calculate recapture. A simplified negative balance alone does not establish recapture.
    /// [NullText]
    /// No earlier deducted losses could require recapture
    /// [SetText]
    /// Earlier loss and recapture history is
    #[libertas_default($null)]
    pub recapture: Option<RiskRecaptureHistory>,
}

/// Alternative minimum-tax facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskAmtMethod {
    /// These basis, depletion and method facts are actually the same as regular tax
    /// A prior detailed return still uses its separately supplied AMT line 19b.
    SameFactsAsRegular,
    /// AMT has different facts
    Separate { facts: alloc::boxed::Box<RiskMethod> },
}

/// Form 6198 computation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskMethod {
    /// Simplified method (Part II)
    Simplified { facts: RiskSimplified },
    /// Detailed method (Part III)
    Detailed { facts: RiskDetailed },
    /// Complete both methods and use the larger amount (line 20)
    Compare { simplified: RiskSimplified, detailed: RiskDetailed },
}

/// Simplified computation (Form 6198 Part II)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskSimplified {
    /// Adjusted basis at the start of 2026, without subtracting liabilities
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_start: i64,
    /// Nonfinancing increases during 2026, excluding current income and gains
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub increases: i64,
    /// Nonfinancing decreases and excluded amounts, excluding current losses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub decreases: i64,
}

/// Detailed computation (Form 6198 Part III)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskDetailed {
    /// Starting point for the detailed method
    pub opening: RiskDetailedOpening,
    /// Nonfinancing increases over the applicable interval, excluding current Part I income and gains
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub increases: i64,
    /// Nonfinancing decreases over the applicable interval, excluding current Part I losses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub decreases: i64,
    /// Depletion and basis by year over that same interval
    pub depletion: Vec<RiskDepletionYear>,
}

/// Detailed computation starting point
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskDetailedOpening {
    /// You completed the detailed method on the immediately preceding return
    PriorDetailed { prior: PriorRiskBalance },
    /// Activity began on or after its applicable effective date, without a prior detailed computation
    BeganAfterEffectiveDate {
        /// Date the activity began
        #[libertas_date_only]
        began: u32,
    },
    /// Activity already existed at its applicable effective date
    AtEffectiveDate {
        /// Legal activity category
        category: RiskEffectiveCategory,
        /// Date the activity began
        #[libertas_date_only]
        began: u32,
        /// Investment at the effective date (line 11 worksheet)
        investment: RiskHistoricalInvestment,
        /// Eligible net outside collateral at the effective date (line 12)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        collateral: i64,
        /// Earlier losses and amounts not at risk by year (line 12 worksheet)
        earlier_losses: Vec<RiskEarlierLoss>,
        /// Other excluded amounts at the effective date, excluding financing entered below
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        excluded_other: i64,
    },
}

/// Activity category for the at-risk rules
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskEffectiveCategory {
    /// Holding, producing or distributing films or videotapes
    Film,
    /// Farming
    Farming,
    /// Leasing section 1245 property
    Leased1245,
    /// Exploring for or exploiting oil and gas
    OilGas,
    /// Exploring for or exploiting geothermal deposits
    Geothermal,
    /// Other trade or business
    OtherTradeBusiness,
    /// Holding real property other than mineral property
    HoldingRealProperty,
    /// Holding mineral property
    HoldingMineralProperty,
}

/// Prior detailed Form 6198 balance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one completed Form 6198 for the activity the user selected. Return only line 19b as regular. A blank, illegible or conflicting line 19b is an error: the form may contain only a simplified computation. Do not substitute line 10b or infer a tax year from the revision date. Do not return AMT amounts, activity identity, loan terms, carry loss or grouping. Multiple activities or conflicting copies require the user to select one completed form.",
    doc_type = "application/pdf,image/*"
)]
pub struct PriorRiskBalance {
    /// Tax year of the immediately preceding detailed return
    // Tax structure: this history precedes the current tax year.
    #[libertas_number(min = 1900, max = $RISK_PRIOR_YEAR)]
    pub year: u32,
    /// Regular line 19b, including an actual zero
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub regular: i64,
    /// Separately tracked AMT line 19b, including an actual zero
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt: i64,
}

/// Investment at the applicable effective date
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskHistoricalInvestment {
    /// Cash in the activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cash: i64,
    /// Inventory
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub inventory: i64,
    /// Cost or other basis of depreciable assets
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depreciable_basis: i64,
    /// Accumulated depreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depreciation: i64,
    /// Basis of depletable assets when contributed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depletable_basis: i64,
    /// Accumulated depletion since contribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depletion: i64,
    /// Adjusted land basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land_basis: i64,
    /// Other asset basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other_basis: i64,
    /// Accounting method at the effective date
    pub accounting: RiskHistoricalAccounting,
}

/// Historical accounting method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskHistoricalAccounting {
    /// Cash
    Cash,
    /// Accrual
    Accrual {
        /// Trade notes and accounts receivable
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        receivable: i64,
        /// Bad-debt reserve actually deducted under former section 166(c)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reserve: i64,
        /// Accounts payable
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        payable: i64,
    },
}

/// Loss before the applicable effective date
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskEarlierLoss {
    /// Loss year
    // Tax structure: this history precedes the current tax year.
    #[libertas_number(min = 1900, max = $RISK_PRIOR_YEAR)]
    pub year: u32,
    /// Net loss after the worksheet excess-depletion adjustment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub loss: i64,
    /// Amounts not at risk at that year end
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub not_at_risk: i64,
}

/// Depletion over the detailed computation interval
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskDepletionYear {
    /// Tax year
    // Tax structure: depletion history ends in the current tax year.
    #[libertas_number(min = 1900, max = $RISK_TAX_YEAR)]
    pub year: u32,
    /// Depletion deduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depletion: i64,
    /// Adjusted basis for that depletion computation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
}

/// Financing in this at-risk computation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskFinancing {
    /// Describe this financing
    #[libertas_size(min = 1)]
    pub description: String,
    /// Principal included in this computation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Where this amount enters the computation
    pub movement: RiskFinancingMovement,
    /// Actual financing terms
    pub terms: RiskFinancingTerms,
    /// Date this amount was borrowed
    /// Required for interested-lender financing and any qualified-nonrecourse exception. Use the actual borrowing date, not the business start date or a refinancing date for unchanged debt.
    /// [NullText]
    /// No borrowing date is needed for this financing
    /// [SetText]
    /// This amount was borrowed on
    #[libertas_default($null)]
    #[libertas_date_only]
    pub incurred: Option<u32>,
}

/// Financing movement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskFinancingMovement {
    /// Already included in opening adjusted basis
    IncludedInOpeningBasis,
    /// New financing during the computation interval
    IncreaseInInterval,
    /// Repayment of capital previously counted at risk during the interval
    DecreaseInInterval,
}

/// Financing terms
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskFinancingTerms {
    /// You are personally liable
    Recourse {
        /// Principal protected against economic loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        protected: i64,
        /// Lender has an interest in the activity other than as a creditor, or is related to such a person
        interested_lender: bool,
        /// Real-property exception facts
        /// Supply the actual facts if this financing would qualify as nonrecourse financing but for personal liability.
        /// [NullText]
        /// No real-property interested-lender exception applies
        /// [SetText]
        /// The real-property exception facts are
        #[libertas_default($null)]
        real_property_exception: Option<RiskQnfFacts>,
    },
    /// Your own property outside this activity secures the borrowing
    OutsideCollateral {
        /// Eligible net fair market value of outside collateral
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        collateral: i64,
        /// Principal protected against economic loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        protected: i64,
        /// Lender has an interest in the activity other than as a creditor, or is related to such a person
        interested_lender: bool,
        /// Real-property exception facts
        /// Supply the actual facts if this financing would qualify as nonrecourse financing but for personal liability.
        /// [NullText]
        /// No real-property interested-lender exception applies
        /// [SetText]
        /// The real-property exception facts are
        #[libertas_default($null)]
        real_property_exception: Option<RiskQnfFacts>,
    },
    /// No personal repayment liability
    Nonrecourse {
        /// Qualified nonrecourse financing facts
        ///
        /// [NullText]
        /// This is not qualified nonrecourse financing
        /// [SetText]
        /// The qualified nonrecourse financing facts are
        #[libertas_default($null)]
        qnf: Option<RiskQnfFacts>,
    },
    /// Other financing protected against loss
    OtherProtected,
}

/// Facts for the qualified-nonrecourse real-property exception
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskQnfFacts {
    /// Borrowed in connection with an activity of holding real property
    pub holding_real_property: bool,
    /// This is mineral property
    pub mineral_property: bool,
    /// Secured by real property used in that activity
    pub secured_real_property: bool,
    /// No person is personally liable for repayment
    pub no_personal_liability: bool,
    /// This debt can convert into an ownership interest
    pub convertible: bool,
    /// Lender or guarantor
    pub lender: RiskQnfLender,
    /// Property securing this financing
    pub collateral: RiskQnfCollateral,
}

/// Financing lender or guarantor
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskQnfLender {
    /// Federal, state or local government loan or guarantee
    GovernmentLoanOrGuarantee,
    /// Person regularly engaged in lending
    RegularLender {
        /// The lender is the seller or a fee recipient, or related to either
        seller_or_fee_recipient: bool,
    },
    /// Related person regularly engaged in lending
    RelatedRegularLender {
        /// Commercially reasonable terms match loans to unrelated borrowers
        commercial_terms_match: bool,
        /// The lender is the seller or a fee recipient, or related to either
        seller_or_fee_recipient: bool,
    },
    /// Other lender
    Other,
}

/// Collateral for real-property financing
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RiskQnfCollateral {
    /// Only real property and incidental personal property
    OnlyRealPropertyOrIncidentalPersonal,
    /// Other property also secures the financing
    Other {
        /// Total gross fair market value of collateral
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        total_gross_fmv: i64,
        /// Gross fair market value of nonincidental other collateral
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        other_gross_fmv: i64,
    },
}

/// Earlier deducted losses and recapture
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskRecaptureHistory {
    /// Regular income-tax history
    pub regular: RiskRecaptureAmounts,
    /// Separate AMT history
    pub amt: RiskRecaptureAmounts,
    /// Negative closing 1978 risk floor, only where that historical exception applies
    #[libertas_money("USD", 2)]
    // Tax structure: the historical 1978 risk floor is nonpositive.
    // Leave headroom for exact cents totals.
    #[libertas_number(max = 0, min = -100000000000)]
    #[libertas_default(0)]
    pub floor_1978: i64,
}

/// Recapture history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RiskRecaptureAmounts {
    /// Losses actually deducted after 1978
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted_post1978: i64,
    /// Prior recapture already reported
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub recaptured_before: i64,
}

// Project monetary source values once; legal category and history dates stay facts.
crate::community::divide_record!(ActivityRiskRecord {
    amounts: [regular, amt, financing, recapture],
    gains: [],
    facts: [],
});
crate::community::divide_enum!(RiskAmtMethod {
    records: [Separate { amounts: [facts], facts: [] },],
    units: [SameFactsAsRegular],
    tuples: [],
});
crate::community::divide_enum!(RiskMethod {
    records: [
        Simplified { amounts: [facts], facts: [] },
        Detailed { amounts: [facts], facts: [] },
        Compare { amounts: [simplified, detailed], facts: [] },
    ],
    units: [],
    tuples: [],
});
crate::community::divide_record!(RiskSimplified {
    amounts: [basis_start, increases, decreases],
    gains: [],
    facts: [],
});
crate::community::divide_record!(RiskDetailed {
    amounts: [opening, increases, decreases, depletion],
    gains: [],
    facts: [],
});
crate::community::divide_enum!(RiskDetailedOpening {
    records: [
        PriorDetailed { amounts: [prior], facts: [] },
        BeganAfterEffectiveDate { amounts: [], facts: [began] },
        AtEffectiveDate {
            amounts: [investment, collateral, earlier_losses, excluded_other],
            facts: [category, began]
        },
    ],
    units: [],
    tuples: [],
});
crate::community::divide_record!(PriorRiskBalance {
    amounts: [regular, amt],
    gains: [],
    facts: [year],
});
crate::community::divide_record!(RiskHistoricalInvestment {
    amounts: [
        cash,
        inventory,
        depreciable_basis,
        depreciation,
        depletable_basis,
        depletion,
        land_basis,
        other_basis,
        accounting
    ],
    gains: [],
    facts: [],
});
crate::community::divide_enum!(RiskHistoricalAccounting {
    records: [Accrual { amounts: [receivable, reserve, payable], facts: [] },],
    units: [Cash],
    tuples: [],
});
crate::community::divide_record!(RiskEarlierLoss {
    amounts: [loss, not_at_risk],
    gains: [],
    facts: [year],
});
crate::community::divide_record!(RiskDepletionYear {
    amounts: [depletion, basis],
    gains: [],
    facts: [year],
});
crate::community::divide_record!(RiskFinancing {
    amounts: [amount, terms],
    gains: [],
    facts: [description, movement, incurred],
});
crate::community::divide_enum!(RiskFinancingTerms {
    records: [
        Recourse { amounts: [protected, real_property_exception], facts: [interested_lender] },
        OutsideCollateral {
            amounts: [collateral, protected, real_property_exception],
            facts: [interested_lender]
        },
        Nonrecourse { amounts: [qnf], facts: [] },
    ],
    units: [OtherProtected],
    tuples: [],
});
// QNF collateral FMVs establish the source's factual 10% eligibility test.
// Keep that ratio exact; only the loan principal/capital contribution divides.
crate::community::divide_record!(RiskQnfFacts {
    amounts: [],
    gains: [],
    facts: [
        collateral,
        holding_real_property,
        mineral_property,
        secured_real_property,
        no_personal_liability,
        convertible,
        lender
    ],
});
crate::community::divide_enum!(RiskQnfCollateral {
    records: [Other { amounts: [], facts: [total_gross_fmv, other_gross_fmv] },],
    units: [OnlyRealPropertyOrIncidentalPersonal],
    tuples: [],
});
crate::community::divide_record!(RiskRecaptureHistory {
    amounts: [regular, amt, floor_1978],
    gains: [],
    facts: [],
});
crate::community::divide_record!(RiskRecaptureAmounts {
    amounts: [deducted_post1978, recaptured_before],
    gains: [],
    facts: [],
});
