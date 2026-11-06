// The topic page's human-facing view of the accepted documents' questions.
// The canonical rows (`DocumentQuestion`) stay the one owner of which
// question a paper calls for and how its answer is applied; this file only
// regroups them for the person: a question a printed box triggers is a
// required answer per paper, a rare situation is asked once per page.
use crate::DocumentAnswer as A;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A question this document calls for
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DocumentStrictQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Answer called for by this document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub allowed_values: Vec<i32>,
    /// Forfeited CCC loans this answer can name
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub ccc_lots: Vec<crate::RecordChoice>,
    /// Existing self-employed health plans this answer can name
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub health_plans: Vec<crate::RecordChoice>,
    /// Accepted Form 3922 documents this answer can name
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub espp_acquisitions: Vec<crate::RecordChoice>,
    /// Accepted interest statements these savings bonds can name
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub interest_statements: Vec<crate::RecordChoice>,
    /// People whose education you paid for can be chosen from here
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub people: Vec<crate::PersonChoice>,
    /// Accepted sources that already include a bond's designated income
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub credit_income_sources: Vec<crate::RecordChoice>,
    /// Your answer
    #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_answers")]
    pub answer: DocumentStrictAnswer,
}

/// A question this document calls for, not answered yet
// The page's construction source: the answer is absent, so the copied
// question starts unanswered and must be answered before Save.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct DocumentStrictQuestionInitial {
    /// Accepted question identity
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    pub prompt: Vec<u8>,
    /// Answer called for by this document
    #[libertas_hidden]
    pub allowed_answers: Vec<i32>,
    /// Available choices for this answer
    #[libertas_hidden]
    pub allowed_values: Vec<i32>,
    /// Forfeited CCC loans this answer can name
    #[libertas_hidden]
    pub ccc_lots: Vec<crate::RecordChoice>,
    /// Existing self-employed health plans this answer can name
    #[libertas_hidden]
    pub health_plans: Vec<crate::RecordChoice>,
    /// Accepted Form 3922 documents this answer can name
    #[libertas_hidden]
    pub espp_acquisitions: Vec<crate::RecordChoice>,
    /// Accepted interest statements these savings bonds can name
    #[libertas_hidden]
    pub interest_statements: Vec<crate::RecordChoice>,
    /// People whose education you paid for can be chosen from here
    #[libertas_hidden]
    pub people: Vec<crate::PersonChoice>,
    /// Accepted sources that already include a bond's designated income
    #[libertas_hidden]
    pub credit_income_sources: Vec<crate::RecordChoice>,
}

// The pages that keep their own question lists (education, medical accounts,
// injured spouse) carry their required answers in these narrow twins of the
// questions page's answers.
pub trait OwnAsked: Sized {
    fn make(
        key: alloc::string::String,
        prompt: Vec<u8>,
        allowed_values: Vec<i32>,
        answer: DocumentStrictAnswer,
    ) -> Option<Self>;
    fn parts(self) -> (alloc::string::String, DocumentStrictAnswer);
}

/// A must-answer question about an education-account document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducationOwnQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_values: Vec<i32>,
    /// Your answer
    pub answer: EducationOwnAnswer,
}

/// Your answer
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EducationOwnAnswer {
    /// The student for this education-account distribution
    EducationBeneficiary {
        /// The student is
        #[libertas_constrained_by("^<EducationOwnQuestion>.allowed_values")]
        value: crate::Q1099Beneficiary,
    },
}

/// A must-answer question about a medical-account document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HsaOwnQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_values: Vec<i32>,
    /// Your answer
    pub answer: HsaOwnAnswer,
}

/// Your answer
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaOwnAnswer {
    /// Your medical-account payment records
    MedicalPayment {
        /// The payment records are
        #[libertas_constrained_by("^<HsaOwnQuestion>.allowed_values")]
        value: crate::MedicalSaPayment,
    },
    /// Whether your spouse's Medicare MSA became your own
    MedicareSpousal {
        /// The Medicare MSA was inherited from my spouse
        /// Yes when the account came to you as your late or former spouse's Medicare MSA.
        value: bool,
    },
    /// Your Medicare MSA prior-year account records
    MedicarePrior {
        /// The prior-year account records are
        value: crate::MedicareMsaPrior,
    },
}

/// A must-answer question about withholding on a document
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredOwnQuestion {
    /// Accepted question identity
    #[libertas_hidden]
    #[libertas_key]
    #[libertas_unique]
    pub key: alloc::string::String,
    /// Question and accepted document
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_hidden]
    pub prompt: Vec<u8>,
    /// Available choices for this answer
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_values: Vec<i32>,
    /// Your answer
    pub answer: InjuredOwnAnswer,
}

/// Your answer
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredOwnAnswer {
    /// Management of withholding funds
    Management {
        /// The withholding funds were managed as
        #[libertas_constrained_by("^<InjuredOwnQuestion>.allowed_values")]
        value: crate::RefundManagement,
    },
}

impl OwnAsked for EducationOwnQuestion {
    fn make(
        key: alloc::string::String,
        prompt: Vec<u8>,
        allowed_values: Vec<i32>,
        answer: DocumentStrictAnswer,
    ) -> Option<Self> {
        let DocumentStrictAnswer::EducationBeneficiary { value } = answer else { return None };
        Some(Self {
            key,
            prompt,
            allowed_values,
            answer: EducationOwnAnswer::EducationBeneficiary { value },
        })
    }
    fn parts(self) -> (alloc::string::String, DocumentStrictAnswer) {
        let EducationOwnAnswer::EducationBeneficiary { value } = self.answer;
        (self.key, DocumentStrictAnswer::EducationBeneficiary { value })
    }
}

impl OwnAsked for HsaOwnQuestion {
    fn make(
        key: alloc::string::String,
        prompt: Vec<u8>,
        allowed_values: Vec<i32>,
        answer: DocumentStrictAnswer,
    ) -> Option<Self> {
        use DocumentStrictAnswer as T;
        let answer = match answer {
            T::MedicalPayment { value } => HsaOwnAnswer::MedicalPayment { value },
            T::MedicareSpousal { value } => HsaOwnAnswer::MedicareSpousal { value },
            T::MedicarePrior { value } => HsaOwnAnswer::MedicarePrior { value },
            _ => return None,
        };
        Some(Self { key, prompt, allowed_values, answer })
    }
    fn parts(self) -> (alloc::string::String, DocumentStrictAnswer) {
        use DocumentStrictAnswer as T;
        (
            self.key,
            match self.answer {
                HsaOwnAnswer::MedicalPayment { value } => T::MedicalPayment { value },
                HsaOwnAnswer::MedicareSpousal { value } => T::MedicareSpousal { value },
                HsaOwnAnswer::MedicarePrior { value } => T::MedicarePrior { value },
            },
        )
    }
}

impl OwnAsked for InjuredOwnQuestion {
    fn make(
        key: alloc::string::String,
        prompt: Vec<u8>,
        allowed_values: Vec<i32>,
        answer: DocumentStrictAnswer,
    ) -> Option<Self> {
        let DocumentStrictAnswer::Management { value } = answer else { return None };
        Some(Self { key, prompt, allowed_values, answer: InjuredOwnAnswer::Management { value } })
    }
    fn parts(self) -> (alloc::string::String, DocumentStrictAnswer) {
        let InjuredOwnAnswer::Management { value } = self.answer;
        (self.key, DocumentStrictAnswer::Management { value })
    }
}

/// Your answer
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum DocumentStrictAnswer {
    /// Section 199A dividends in box 5
    DividendEligibility {
        /// You held the fund shares more than 45 days around the ex-dividend date and had no obligation to make related payments
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::DividendEligibility,
    },
    /// The fund's section 1202 exclusion
    DividendFundExclusion {
        /// Exclusion the fund's statement gives
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::FundQsbsExclusion,
    },
    /// What the distribution codes mean for you
    RetirementTreatment {
        /// Treatment
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        code: crate::RetirementTreatmentView,
    },
    /// How to figure the taxable distribution
    RetirementMethod {
        /// Taxable amount method
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        method: crate::TaxableMethod,
    },
    /// Nontaxable part of the in-plan Roth rollover in box 10
    RetirementBasis {
        /// Basis
        basis: crate::InPlanRothBasis,
    },
    /// Whose railroad annuity this is
    RailroadRecipient {
        /// Annuity
        value: crate::RailroadRecipient,
    },
    /// Community or separate income
    Community {
        /// Community or separate
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::CommunityShare,
    },
    /// How you deducted the refunded tax in the year it was paid
    RefundDeduction {
        /// Deduction
        value: crate::RefundDeduction,
    },
    /// What happened to this opportunity fund investment in 2026
    OpportunityFundEvent {
        /// What happened
        value: crate::QofEvent,
    },
    /// Other original issue discount circumstances
    OidTreatment {
        /// A treatment this page does not calculate
        value: crate::OidSpecial,
    },
    /// Box 12 code TT overtime pay
    WageOvertime {
        /// Is this overtime pay the Fair Labor Standards Act requires your employer to pay?
        value: crate::WageDeductionEligibility,
    },
    /// What a box 14 item on the W-2 is
    WageBox14 {
        /// What is this box 14 item?
        kind: crate::W2Box14Kind,
        /// The state of the item
        /// Needed for a state disability, family-leave or pickup amount when the label and box 15 show none.
        /// [NullText]
        /// This item belongs to no state
        state: Option<crate::TaxState>,
    },
    /// Tips in an occupation on the IRS list of tipped occupations
    WageTipOccupation {
        /// Is your occupation for this employer on the IRS list of tipped occupations?
        /// Box 14b shows the occupation code when your employer reported tips.
        value: crate::WageDeductionEligibility,
    },
    /// For the section 179 income limit
    RentalSection179Business {
        /// Do you actively conduct this rental as a trade or business?
        /// Yes when you take a meaningful part in its management or operations; a rental held only as an investment is No. Only Yes counts its net income toward the section 179 business income limit.
        value: bool,
    },
    /// Last year's home-use worksheet carries for this rental
    RentalHomeCarries {
        /// Carries from the 2025 home-use worksheet
        value: crate::RentalHomeCarries,
    },
    /// Whether wages already include the compensation from this ESPP sale
    EsppWageInclusion {
        /// Your W-2 box 1 already includes this sale's full calculated compensation
        /// Check the employer's statement. If No, the compensation enters Schedule 1 once.
        value: bool,
    },
    /// Fees and special treatment for this ISO sale
    IsoAdjustments {
        /// Fees and special treatment
        value: crate::IsoShareSaleMore,
    },
    /// Whether wages already include the compensation from this ISO sale
    IsoWageInclusion {
        /// Your W-2 box 1 already includes this sale's compensation
        /// Applies because the sale is within 2 years of the grant or 1 year of the exercise. Check the employer's statement. If No, the compensation enters Schedule 1 once.
        value: bool,
    },
    /// Wash sale and special treatment for this employer-stock sale
    NuaAdjustments {
        /// Wash sale and special treatment
        value: crate::NuaShareSaleMore,
    },
    /// Whose employer shares these were
    NuaOwnership {
        /// Whose shares
        value: crate::NuaOwnership,
    },
    /// Medicare premiums on this statement reimbursed tax-free from an HSA
    SocialReimbursedPremiums {
        /// Actual part included in the qualified HSA spending already entered
        /// Enter zero if none. Do not repeat a premium used by a business health plan.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reimbursed: i64,
    },
    /// Treatment of this canceled debt
    DebtTreatment {
        /// Treatment
        value: crate::CanceledDebtChoice,
    },
    /// Property the creditor actually took
    DebtProperty {
        /// Property
        value: crate::TransferredProperty,
    },
    /// Actual source of the box 1 benefits
    LtcBenefitOrigin {
        /// Source of the box 1 benefits
        value: crate::LtcPaymentOrigin,
    },
    /// Actual source of the box 2 accelerated death benefits
    LtcAcceleratedOrigin {
        /// Source of the box 2 accelerated benefits
        value: crate::LtcPaymentOrigin,
    },
    /// Shares from this incentive stock option exercise
    IsoExerciseFacts {
        /// Share details
        value: crate::Form3921More,
    },
    /// What you sold
    HomeSale {
        /// What you sold
        value: crate::RealEstateSaleUse,
    },
    /// What the ABLE payment was
    AbleTreatment {
        /// Payment treatment
        value: crate::AbleTreatment,
    },
    /// Accounting election accompanying the current entity period
    EstateAccounting {
        /// Accounting year
        value: crate::FinalEstateTrustAccountingYear,
    },
    /// Holding-period eligibility of the printed section 199A dividends
    EstateReitEligibility {
        /// The reported dividends meet the holding-period rules
        value: bool,
    },
    /// What happened to your entity interest
    K1EntityDisposition {
        /// What happened to this interest
        value: crate::K1Disposition,
    },
    /// Section 1256 contract treatment
    BrokerContractsTreatment {
        /// A business or farm hedge, a section 475(f) or 988 election, or dealer contracts
        /// Yes retains the statement with the existing uncalculated-treatment notice.
        value: bool,
    },
    /// Beneficiary's Form 3800 line for box 13 code I
    EstateResearchCreditLine {
        /// Choose line 4i only when you and the estate or trust both meet the eligible-small-business test; otherwise line 1c.
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::BusinessCredit,
    },
    /// Your eligibility for the K-1 research credit
    K1ResearchCreditLine {
        /// Choose line 4i only when you and the entity both meet the eligible-small-business test; otherwise line 1c.
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::BusinessCredit,
    },
    /// Classification of this acquisition or abandonment
    DebtAcquisition {
        /// The property was
        value: StrictAcquiredProperty,
    },
    /// Acquisition records for these actual ESPP shares sold
    EsppAcquisition {
        /// Acquisition source
        value: StrictEsppAcquisition,
    },
    /// Bond redemption and education records supporting this Form 8815
    SavingsBondsRecords {
        /// Interest statement and the people whose education you paid for
        value: StrictSavingsBondAnswers,
    },
    /// Where the Medicare premiums on this statement are already included
    SocialPremiums {
        /// Where these premiums are already included
        value: StrictSocialPremiumUse,
    },
    /// Allocation between medical expenses, a business health plan and HSA reimbursement
    SocialSharedPremiums {
        /// Where these premiums are already included
        value: StrictSocialPremiumUse,
        /// Actual part reimbursed tax-free and included in existing qualified HSA spending
        /// Enter zero if none; this part must not also be included in the selected plan.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reimbursed: i64,
    },
    /// Whether you succeed to the terminated estate or trust
    EstateFinalSuccessor {
        /// The estate or trust terminated in 2026 and you succeed to its property
        /// Yes when the K-1 is the final one and shows your share of the final-year deductions or carryovers.
        value: bool,
    },
    /// When you became a partner and your opening basis
    K1PartnerBasisStart {
        /// When you became a partner
        value: StrictPartnerStart,
    },
    /// When you became a shareholder and your opening basis
    K1ScorpBasisStart {
        /// When you became a shareholder
        value: StrictScorpStart,
    },
    /// How you took part in the rental real estate (box 2)
    K1RentalParticipation {
        /// How you took part in the rental real estate in 2026
        value: crate::K1RentalParticipation,
    },
    /// The student for this education-account distribution
    EducationBeneficiary {
        /// The student is
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::Q1099Beneficiary,
    },
    /// Your medical-account payment records
    MedicalPayment {
        /// The payment records are
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::MedicalSaPayment,
    },
    /// Whether your spouse's Medicare MSA became your own
    MedicareSpousal {
        /// The Medicare MSA was inherited from my spouse
        /// Yes when the account came to you as your late or former spouse's Medicare MSA.
        value: bool,
    },
    /// Your Medicare MSA prior-year account records
    MedicarePrior {
        /// The prior-year account records are
        value: crate::MedicareMsaPrior,
    },
    /// Management of withholding funds
    Management {
        /// The withholding funds were managed as
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::RefundManagement,
    },
    /// How you took part in this partnership
    K1PartnerParticipation {
        /// How you took part in 2026
        value: crate::PartnerParticipation,
    },
    /// Limited partner for self-employment tax
    K1LimitedForSe {
        /// Limited-partner treatment for self-employment tax applies
        /// Yes only when you are a limited partner under section 1402(a)(13); your share of ordinary income then has no self-employment tax.
        value: bool,
    },
    /// Your partnership alternative minimum tax records
    K1PartnerAmt {
        /// AMT records
        value: crate::PartnerAmtRecords,
    },
    /// What happened to your partnership interest
    K1PartnerDisposition {
        /// What happened to this interest
        value: crate::PartnerDisposition,
    },
    /// How you took part in this S corporation
    K1ScorpParticipation {
        /// Your work and participation history
        value: crate::Participation,
    },
    /// Your S-corporation alternative minimum tax records
    K1ScorpAmt {
        /// AMT records
        value: crate::ScorpAmtRecords,
    },
    /// What happened to your S-corporation shares
    K1ScorpDisposition {
        /// What happened to your shares
        value: crate::ScorpDisposition,
    },
    /// Your AMT section 179 carryover history
    K1Section179Amt {
        /// From your own prior AMT Form 4562 records, independently of the regular carryover. Enter actual zero amounts when the AMT carryover is zero.
        value: crate::Section179IncomeCarryHistory,
    },
    /// Where the other income belongs
    MiscellaneousUse {
        /// Where box 3 belongs
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::IncomeUse,
    },
    /// Low-income housing interest acquisition
    K1CreditConfirmations {
        /// When you acquired the low-income housing interest
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        value: crate::K1HousingAcquisition,
    },
    /// 2025 Form 1040 line 1z wages
    BbaPriorWages {
        /// Total 2025 Form 1040 line 1z wages on the filed or IRS-adjusted return
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
    },
    /// 2025 taxable income
    BbaPriorTaxableIncome {
        /// Taxable income on the filed or IRS-adjusted 2025 Form 1040 line 15
        /// This calculation supports original and corrected taxable income below $100,000.
        #[libertas_money("USD", 2)]
        // Tax structure: this calculation uses the IRS Tax Table below $100,000 taxable income.
        #[libertas_number(min = 0, max = 9999900)]
        value: i64,
    },
    /// 2025 income tax
    BbaPriorIncomeTax {
        /// Income tax on the filed or IRS-adjusted 2025 Form 1040 line 16
        /// For this case, lines 16, 18 and 24 must be equal; there were no other income taxes or credits.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
    },
    /// 2025 filing status
    BbaPriorStatus {
        /// 2025 filing status used on the return and its Tax Table
        value: crate::FederalStatus,
    },
    /// Only one BBA-audit adjustment this year
    BbaSingleAdjustment {
        /// This is the only Form 8986 or Form 8978 adjustment for 2026, from a BBA audit rather than an AAR
        value: bool,
    },
    /// The 2025 return is the simple Tax Table case
    BbaSimplePriorReturn {
        /// The filed 2025 return used ordinary Tax Table income tax, with only wages, this K-1 interest and the basic standard deduction
        /// Neither filer was age 65 or blind, no one could claim either filer as a dependent, no dependents were claimed, and no other adjustment or deduction applied. There was no Social Security, preferential income, QBI, NIIT, FTC, EIC or other credit, AMT, other tax, AGI-based deduction or phaseout, or prior correction, before or after the adjustment. Filed Forms 1040 lines 16, 18 and 24 were equal.
        value: bool,
    },
    /// No other effects of the adjustment
    BbaNoOtherEffects {
        /// The interest change has no other deduction, credit, carryover, tax-attribute or 2026-income effect
        /// No inconsistent K-1 position, penalty, section 6603 deposit, prior payment toward this adjustment or other Form 8986 amount applies.
        value: bool,
    },
    /// Form 8978 line 17 interest
    BbaLine17Interest {
        /// Form 8978 line 17 interest from the separate supporting computation
        /// Compute from the original 2025 return due date through the planned payment date using the applicable quarterly short-term rate plus five percentage points. Enter zero when the documented interest is zero. This interest is outside Form 1040 tax and balance.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        interest: i64,
        /// The separate Form 8978 line 17 interest calculation is complete and documented
        prepared: bool,
    },
    /// Bond class
    BondClass {
        /// Bond class from the underlying issue records
        value: crate::FederalTaxCreditBondKind,
    },
    /// Original bond issue date
    BondIssueDate {
        /// Original bond issue date
        #[libertas_date_only]
        value: u32,
    },
    /// How the bond credit reached you
    BondOrigin {
        /// Direct holding or actual entity report
        value: StrictTaxCreditOrigin,
    },
    /// Unused 2025 credit for this bond
    BondPriorCarry {
        /// Unused 2025 Form 8912 whole-dollar credit attributable to this same bond
        /// Copy the whole-dollar amount from the prior Form 8912 carryforward worksheet. Enter zero if none. Do not enter cents or re-round it.
        #[libertas_number(min = 0, max = 1000000000)]
        dollars: i64,
        /// The prior carry is documented on a prior Form 8912 worksheet for this same bond
        /// Matters only when a positive prior carry is entered: check the 2025 Form 8912 carryforward worksheet. No leaves the prior carry unclaimed.
        confirmed: bool,
    },
    /// This bond's share of the allowed credit
    BondAllowedShare {
        /// This bond's whole-dollar share of the allowed 2026 Form 8912 line 12 credit
        /// Your bonds' credit is partly limited. Enter a share for every bond, including zero; together they must equal line 12, and no share may exceed that bond's box 1 credit rounded to dollars plus its same-bond prior carry. The IRS specifies no bond ordering.
        #[libertas_number(min = 0, max = 1000000000)]
        dollars: i64,
        /// Confirm this bond's documented share of the allowed credit
        /// From the retained Form 8912 line 12 allocation worksheet tying this bond's Form 1097-BTC identifier to its share.
        confirmed: bool,
    },
    /// The year your employer designated this W-2 health-account contribution for
    EmployerYear {
        /// Employer's designated contribution year
        /// Use the employer's or custodian's actual designation. A prior-year part is not automatically taxable prior excess.
        value: crate::HealthEmployerYear,
    },
    /// What the box 4 refund on this Form 1098 repays
    MortgageRefundOrigin {
        /// What the box 4 refund repays
        /// Check the lender's statement and the earlier return to identify its treatment.
        value: crate::MortgageRefundOrigin,
    },
    /// The other filer's HSA eligibility in 2026
    OtherHsaEligibility {
        /// The other filer's HSA eligibility
        /// A married filer making eligible contributions needs the other filer's actual eligibility, even if that person owns no HSA. Choose full-year ineligible when appropriate.
        value: crate::HsaEligibilityPage,
    },
}

/// A situation that applies to one of these documents
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DocumentSituation {
    /// Situation
    #[libertas_ask_on_add]
    #[libertas_constrained_by("$.allowed_situations")]
    pub situation: DocumentSituationAnswer,
    /// Document it applies to
    #[libertas_ask_on_add]
    #[libertas_enum_source("$.situation_documents")]
    pub document: i64,
}

/// Do any of these apply to your documents?
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DocumentSituations {
    /// None of these
    NoneOfThese,
    /// These apply
    These {
        /// Each situation and the document it applies to
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        /// ----
        /// Situation
        situations: Vec<DocumentSituation>,
    },
}

/// The saved answer to the situations question
// A construction source only: a page whose situations are still to be
// answered sends none, so the question starts unanswered.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct DocumentSituationsAnswered {
    /// Situations
    pub value: DocumentSituations,
}

/// Situation
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum DocumentSituationAnswer {
    /// Interest reported to you that belongs to someone else
    /// You received it as a nominee, or it was paid on an account you hold with someone outside this return.
    InterestNominee {
        /// Allocation
        value: crate::IntNomineeAnswer,
    },
    /// Dividends reported to you that belong to someone else
    DividendNominee {
        /// Allocation
        value: crate::DivNomineeAnswer,
    },
    /// Original issue discount reported to you that belongs to someone else
    OidNominee {
        /// Allocation
        value: crate::OidNomineeAnswer,
    },
    /// Adjustments from your own records to interest or original issue discount
    /// For example bond premium you amortize, acquisition premium, or interest already reported on an earlier return.
    ReportingAdjustments {
        /// Adjustments
        value: crate::InterestAdjustments,
    },
    /// Accrued interest you paid when you bought a bond
    AccruedInterest {
        /// Accrued interest paid
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        value: i64,
    },
    /// Bond premium on the tax-exempt original issue discount
    TaxExemptOidPremium {
        /// Tax-exempt bond premium
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        value: i64,
    },
    /// Private activity bond interest within the tax-exempt original issue discount
    PrivateActivityOid {
        /// Private activity bond interest
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        value: i64,
    },
    /// A qualified disaster distribution
    DisasterDistribution {
        /// Disaster
        #[libertas_constrained_by("$.disaster_choices")]
        value: crate::DisasterDistribution,
    },
    /// Insurance premiums paid from a public safety officer's pension
    PublicSafetyPremiums {
        /// Premiums
        value: crate::PublicSafetyAnswer,
    },
    /// Disability retirement income for the credit for the elderly or the disabled
    DisabilityRetirement {
        /// Qualifying disability amount already included in this entry
        /// Enter only employer accident, health or pension-plan payments replacing wages while absent due to permanent and total disability. Exclude accrued leave, Social Security, tax-free benefits and amounts after mandatory retirement age; do not add this amount to income again.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Retired on permanent and total disability, with the required medical certification
        /// You were disabled when you retired, had not reached mandatory retirement age on January 1, 2026, and retain the Schedule R physician/VA statement or qualify to rely on the prior statement. No claims no Schedule R credit on this income.
        eligible: bool,
    },
    /// Interest received after the owner's death
    InterestAfterDeath {
        /// Allocation
        value: crate::IntAfterDeathAnswer,
    },
    /// Dividends received after the owner's death
    DividendAfterDeath {
        /// Allocation
        value: crate::DivAfterDeathAnswer,
    },
    /// Retirement payments made after the owner's death
    PaymentsAfterDeath {
        /// Part paid after the death
        value: crate::RetirementAfterDeath,
    },
    /// A daily tip record that replaces the box 8 allocated tips
    DailyTips {
        /// Tip record
        value: crate::DailyTipRecord,
    },
    /// Noncash tips from customers
    /// Goods, passes and other noncash tips. Income on Form 1040 line 1c, with no tip payroll tax or tips deduction.
    NoncashTips {
        /// Noncash tips
        value: crate::W2Amount,
    },
    /// Pay as a church employee of a church that elected out of Social Security and Medicare
    ChurchEmployeePay,
    /// Pay for ministerial services or services your religious order requires
    MinisterialPay,
    /// Part of box 1 was paid for work you did abroad
    /// For Form 2555: services you performed in a foreign country while your tax home was there.
    PayAbroad {
        /// Pay for work abroad
        value: crate::W2Amount,
    },
    /// Box 1 holds a pension from a nonqualified deferred compensation or nongovernmental 457 plan
    DeferredPension {
        /// Pension in box 1
        value: crate::W2Amount,
    },
    /// Box 1 includes pay for work while an inmate
    InmatePay {
        /// Pay for work while an inmate
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        value: i64,
    },
    /// Box 1 includes Medicaid waiver payments you can exclude
    /// State Medicaid waiver payments for caring for someone who lived in your home with you (Notice 2014-7).
    MedicaidWaiver {
        /// Medicaid waiver payments in box 1
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        value: i64,
    },
    /// Your statutory employee pay has its own saved Schedule C
    StatutoryBusiness {
        /// Schedule C
        value: crate::StatutoryBusinessView,
    },
    /// One of your S corporations paid this W-2
    OwnScorporation {
        /// S corporation
        value: crate::WageScorpEmployerView,
    },
    /// Benefits you repaid in 2026
    BenefitsRepaid {
        /// Repayments
        #[libertas_size(min = 1)]
        /// ----
        /// Repayment
        value: Vec<crate::UnemploymentRepayment>,
    },
    /// Paid family leave that includes paid medical leave
    FamilyLeaveMedical,
    /// A business or farm hedge, a section 475(f) or 988 election, or you deal in these contracts
    /// The statement stays out of the estimate with a notice; that treatment is not calculated.
    ContractsHedge,
    /// Interest taxed under a tax treaty
    InterestTreaty,
    /// Foreign treatment beyond the foreign tax credit election
    /// A treaty rate or another foreign-tax treatment the credit election does not cover.
    ForeignDividendTreatment,
    /// Another treatment this App does not calculate
    /// The document stays out of the estimate with a notice.
    AnotherTreatment,
}

type S = DocumentSituationAnswer;

// Strict questions: required answers a printed box calls for. `None` for any
// other answer; `Some(None)` while the accepted answer is still missing.
pub(crate) fn strict_answer(answer: &A) -> Option<Option<DocumentStrictAnswer>> {
    use DocumentStrictAnswer as T;
    Some(match answer {
        A::DividendEligibility { value } => value.map(|value| T::DividendEligibility { value }),
        A::DividendFundExclusion { value } => value.map(|value| T::DividendFundExclusion { value }),
        A::RetirementTreatment { code } => code.clone().map(|code| T::RetirementTreatment { code }),
        A::RetirementMethod { method } => {
            method.clone().map(|method| T::RetirementMethod { method })
        }
        A::RetirementBasis { basis } => basis.map(|basis| T::RetirementBasis { basis }),
        A::RailroadRecipient { value } => value.clone().map(|value| T::RailroadRecipient { value }),
        A::Community { value } => {
            (*value != crate::CommunityShare::NotNeeded).then_some(T::Community { value: *value })
        }
        A::RefundDeduction { value } => value.clone().map(|value| T::RefundDeduction { value }),
        A::OpportunityFundEvent { value } => {
            value.clone().map(|value| T::OpportunityFundEvent { value })
        }
        A::OidTreatment { value } => value.map(|value| T::OidTreatment { value }),
        A::RentalSection179Business { value } => {
            value.map(|value| T::RentalSection179Business { value })
        }
        A::RentalHomeCarries { value } => value.clone().map(|value| T::RentalHomeCarries { value }),
        A::EsppWageInclusion { value } => (*value).map(|value| T::EsppWageInclusion { value }),
        A::IsoAdjustments { value } => value.clone().map(|value| T::IsoAdjustments { value }),
        A::IsoWageInclusion { value } => (*value).map(|value| T::IsoWageInclusion { value }),
        A::NuaAdjustments { value } => value.clone().map(|value| T::NuaAdjustments { value }),
        A::NuaOwnership { value } => value.clone().map(|value| T::NuaOwnership { value }),
        A::SocialReimbursedPremiums { reimbursed } => {
            (*reimbursed).map(|reimbursed| T::SocialReimbursedPremiums { reimbursed })
        }
        A::DebtTreatment { value } => value.clone().map(|value| T::DebtTreatment { value }),
        A::DebtProperty { value } => value.clone().map(|value| T::DebtProperty { value }),
        A::LtcBenefitOrigin { value } => value.clone().map(|value| T::LtcBenefitOrigin { value }),
        A::LtcAcceleratedOrigin { value } => {
            value.clone().map(|value| T::LtcAcceleratedOrigin { value })
        }
        A::IsoExerciseFacts { value } => value.clone().map(|value| T::IsoExerciseFacts { value }),
        A::HomeSale { value } => value.clone().map(|value| T::HomeSale { value }),
        A::AbleTreatment { value } => value.clone().map(|value| T::AbleTreatment { value }),
        A::EstateAccounting { value } => value.clone().map(|value| T::EstateAccounting { value }),
        A::EstateReitEligibility { value } => value.map(|value| T::EstateReitEligibility { value }),
        A::K1EntityDisposition { value } => {
            value.clone().map(|value| T::K1EntityDisposition { value })
        }
        A::BrokerContractsTreatment { value } => {
            value.map(|value| T::BrokerContractsTreatment { value })
        }
        A::K1RentalParticipation { value } => value.map(|value| T::K1RentalParticipation { value }),
        A::EstateResearchCreditLine { value } => {
            value.map(|value| T::EstateResearchCreditLine { value })
        }
        A::K1ResearchCreditLine { value } => value.map(|value| T::K1ResearchCreditLine { value }),
        A::DebtAcquisition { value } => {
            value.clone().map(|value| T::DebtAcquisition { value: value.into() })
        }
        A::EsppAcquisition { value } => {
            value.clone().map(|value| T::EsppAcquisition { value: value.into() })
        }
        A::SavingsBondsRecords { value } => {
            value.clone().map(|value| T::SavingsBondsRecords { value: value.into() })
        }
        A::SocialPremiums { value } => {
            value.clone().map(|value| T::SocialPremiums { value: value.into() })
        }
        A::SocialSharedPremiums { value, reimbursed } => value
            .clone()
            .zip(*reimbursed)
            .map(|(value, reimbursed)| T::SocialSharedPremiums { value: value.into(), reimbursed }),
        A::EstateFinalSuccessor { value } => value.map(|value| T::EstateFinalSuccessor { value }),
        A::K1PartnerBasisStart { value } => value
            .as_ref()
            .and_then(StrictPartnerStart::from_stored)
            .map(|value| T::K1PartnerBasisStart { value }),
        A::K1ScorpBasisStart { value } => value
            .as_ref()
            .and_then(StrictScorpStart::from_stored)
            .map(|value| T::K1ScorpBasisStart { value }),
        A::EducationBeneficiary { value } => {
            value.clone().map(|value| T::EducationBeneficiary { value })
        }
        A::MedicalPayment { value } => value.clone().map(|value| T::MedicalPayment { value }),
        A::MedicareSpousal { value } => (*value).map(|value| T::MedicareSpousal { value }),
        A::MedicarePrior { value } => value.clone().map(|value| T::MedicarePrior { value }),
        A::Management { value } => (*value).map(|value| T::Management { value }),
        A::K1PartnerParticipation { value } => {
            value.clone().map(|value| T::K1PartnerParticipation { value })
        }
        A::K1LimitedForSe { value } => value.map(|value| T::K1LimitedForSe { value }),
        A::K1PartnerAmt { value } => value.clone().map(|value| T::K1PartnerAmt { value }),
        A::K1PartnerDisposition { value } => {
            value.clone().map(|value| T::K1PartnerDisposition { value })
        }
        A::K1ScorpParticipation { value } => {
            value.clone().map(|value| T::K1ScorpParticipation { value })
        }
        A::K1ScorpAmt { value } => value.clone().map(|value| T::K1ScorpAmt { value }),
        A::K1ScorpDisposition { value } => {
            value.clone().map(|value| T::K1ScorpDisposition { value })
        }
        A::K1Section179Amt { value } => value.clone().map(|value| T::K1Section179Amt { value }),
        A::MiscellaneousUse { value } => value.clone().map(|value| T::MiscellaneousUse { value }),
        A::K1CreditConfirmations { housing: value } => {
            value.map(|value| T::K1CreditConfirmations { value })
        }
        A::BbaPriorWages { value } => value.map(|value| T::BbaPriorWages { value }),
        A::BbaPriorTaxableIncome { value } => value.map(|value| T::BbaPriorTaxableIncome { value }),
        A::BbaPriorIncomeTax { value } => value.map(|value| T::BbaPriorIncomeTax { value }),
        A::BbaPriorStatus { value } => value.map(|value| T::BbaPriorStatus { value }),
        A::BbaSingleAdjustment { value } => value.map(|value| T::BbaSingleAdjustment { value }),
        A::BbaSimplePriorReturn { value } => value.map(|value| T::BbaSimplePriorReturn { value }),
        A::BbaNoOtherEffects { value } => value.map(|value| T::BbaNoOtherEffects { value }),
        A::BbaLine17Interest { interest, prepared } => interest
            .zip(*prepared)
            .map(|(interest, prepared)| T::BbaLine17Interest { interest, prepared }),
        A::BondClass { value } => value.map(|value| T::BondClass { value }),
        A::BondIssueDate { value } => value.map(|value| T::BondIssueDate { value }),
        A::BondOrigin { value } => value.clone().map(|value| T::BondOrigin { value: value.into() }),
        A::BondPriorCarry { dollars, confirmed } => {
            confirmed.map(|confirmed| T::BondPriorCarry { dollars: *dollars, confirmed })
        }
        A::BondAllowedShare { dollars, confirmed } => dollars
            .zip(*confirmed)
            .map(|(dollars, confirmed)| T::BondAllowedShare { dollars, confirmed }),
        _ => return None,
    })
}

impl DocumentStrictAnswer {
    // The canonical answer of a whole-row question; a W-2's own questions are
    // parts of its row (followup_page).
    pub(crate) fn into_canonical(self) -> Option<A> {
        Some(match self {
            Self::DividendEligibility { value } => A::DividendEligibility { value: Some(value) },
            Self::DividendFundExclusion { value } => {
                A::DividendFundExclusion { value: Some(value) }
            }
            Self::RetirementTreatment { code } => A::RetirementTreatment { code: Some(code) },
            Self::RetirementMethod { method } => A::RetirementMethod { method: Some(method) },
            Self::RetirementBasis { basis } => A::RetirementBasis { basis: Some(basis) },
            Self::RailroadRecipient { value } => A::RailroadRecipient { value: Some(value) },
            Self::Community { value } => A::Community { value },
            Self::RefundDeduction { value } => A::RefundDeduction { value: Some(value) },
            Self::OpportunityFundEvent { value } => A::OpportunityFundEvent { value: Some(value) },
            Self::OidTreatment { value } => A::OidTreatment { value: Some(value) },
            Self::RentalSection179Business { value } => {
                A::RentalSection179Business { value: Some(value) }
            }
            Self::RentalHomeCarries { value } => A::RentalHomeCarries { value: Some(value) },
            Self::EsppWageInclusion { value } => A::EsppWageInclusion { value: Some(value) },
            Self::IsoAdjustments { value } => A::IsoAdjustments { value: Some(value) },
            Self::IsoWageInclusion { value } => A::IsoWageInclusion { value: Some(value) },
            Self::NuaAdjustments { value } => A::NuaAdjustments { value: Some(value) },
            Self::NuaOwnership { value } => A::NuaOwnership { value: Some(value) },
            Self::SocialReimbursedPremiums { reimbursed } => {
                A::SocialReimbursedPremiums { reimbursed: Some(reimbursed) }
            }
            Self::DebtTreatment { value } => A::DebtTreatment { value: Some(value) },
            Self::DebtProperty { value } => A::DebtProperty { value: Some(value) },
            Self::LtcBenefitOrigin { value } => A::LtcBenefitOrigin { value: Some(value) },
            Self::LtcAcceleratedOrigin { value } => A::LtcAcceleratedOrigin { value: Some(value) },
            Self::IsoExerciseFacts { value } => A::IsoExerciseFacts { value: Some(value) },
            Self::HomeSale { value } => A::HomeSale { value: Some(value) },
            Self::AbleTreatment { value } => A::AbleTreatment { value: Some(value) },
            Self::EstateAccounting { value } => A::EstateAccounting { value: Some(value) },
            Self::EstateReitEligibility { value } => {
                A::EstateReitEligibility { value: Some(value) }
            }
            Self::K1EntityDisposition { value } => A::K1EntityDisposition { value: Some(value) },
            Self::BrokerContractsTreatment { value } => {
                A::BrokerContractsTreatment { value: Some(value) }
            }
            Self::K1RentalParticipation { value } => {
                A::K1RentalParticipation { value: Some(value) }
            }
            Self::EstateResearchCreditLine { value } => {
                A::EstateResearchCreditLine { value: Some(value) }
            }
            Self::K1ResearchCreditLine { value } => A::K1ResearchCreditLine { value: Some(value) },
            Self::DebtAcquisition { value } => A::DebtAcquisition { value: Some(value.into()) },
            Self::EsppAcquisition { value } => A::EsppAcquisition { value: Some(value.into()) },
            Self::SavingsBondsRecords { value } => {
                A::SavingsBondsRecords { value: Some(value.into()) }
            }
            Self::SocialPremiums { value } => A::SocialPremiums { value: Some(value.into()) },
            Self::SocialSharedPremiums { value, reimbursed } => {
                A::SocialSharedPremiums { value: Some(value.into()), reimbursed: Some(reimbursed) }
            }
            Self::EstateFinalSuccessor { value } => A::EstateFinalSuccessor { value: Some(value) },
            Self::K1PartnerBasisStart { value } => {
                A::K1PartnerBasisStart { value: Some(value.stored()) }
            }
            Self::K1ScorpBasisStart { value } => {
                A::K1ScorpBasisStart { value: Some(value.stored()) }
            }
            Self::EducationBeneficiary { value } => A::EducationBeneficiary { value: Some(value) },
            Self::MedicalPayment { value } => A::MedicalPayment { value: Some(value) },
            Self::MedicareSpousal { value } => A::MedicareSpousal { value: Some(value) },
            Self::MedicarePrior { value } => A::MedicarePrior { value: Some(value) },
            Self::Management { value } => A::Management { value: Some(value) },
            Self::K1PartnerParticipation { value } => {
                A::K1PartnerParticipation { value: Some(value) }
            }
            Self::K1LimitedForSe { value } => A::K1LimitedForSe { value: Some(value) },
            Self::K1PartnerAmt { value } => A::K1PartnerAmt { value: Some(value) },
            Self::K1PartnerDisposition { value } => A::K1PartnerDisposition { value: Some(value) },
            Self::K1ScorpParticipation { value } => A::K1ScorpParticipation { value: Some(value) },
            Self::K1ScorpAmt { value } => A::K1ScorpAmt { value: Some(value) },
            Self::K1ScorpDisposition { value } => A::K1ScorpDisposition { value: Some(value) },
            Self::K1Section179Amt { value } => A::K1Section179Amt { value: Some(value) },
            Self::MiscellaneousUse { value } => A::MiscellaneousUse { value: Some(value) },
            Self::K1CreditConfirmations { value } => {
                A::K1CreditConfirmations { housing: Some(value) }
            }
            Self::BbaPriorWages { value } => A::BbaPriorWages { value: Some(value) },
            Self::BbaPriorTaxableIncome { value } => {
                A::BbaPriorTaxableIncome { value: Some(value) }
            }
            Self::BbaPriorIncomeTax { value } => A::BbaPriorIncomeTax { value: Some(value) },
            Self::BbaPriorStatus { value } => A::BbaPriorStatus { value: Some(value) },
            Self::BbaSingleAdjustment { value } => A::BbaSingleAdjustment { value: Some(value) },
            Self::BbaSimplePriorReturn { value } => A::BbaSimplePriorReturn { value: Some(value) },
            Self::BbaNoOtherEffects { value } => A::BbaNoOtherEffects { value: Some(value) },
            Self::BbaLine17Interest { interest, prepared } => {
                A::BbaLine17Interest { interest: Some(interest), prepared: Some(prepared) }
            }
            Self::BondClass { value } => A::BondClass { value: Some(value) },
            Self::BondIssueDate { value } => A::BondIssueDate { value: Some(value) },
            Self::BondOrigin { value } => A::BondOrigin { value: Some(value.into()) },
            Self::BondPriorCarry { dollars, confirmed } => {
                A::BondPriorCarry { dollars, confirmed: Some(confirmed) }
            }
            Self::BondAllowedShare { dollars, confirmed } => {
                A::BondAllowedShare { dollars: Some(dollars), confirmed: Some(confirmed) }
            }
            Self::WageOvertime { .. }
            | Self::WageTipOccupation { .. }
            | Self::WageBox14 { .. }
            | Self::EmployerYear { .. }
            | Self::MortgageRefundOrigin { .. }
            | Self::OtherHsaEligibility { .. } => {
                return None;
            }
        })
    }
    // The strict variant standing for a canonical answer kind.
    pub(crate) fn kind_of(answer: &A) -> Option<i32> {
        use libertas_macros::variant_index;
        Some(match answer {
            A::DividendEligibility { .. } => variant_index!(Self::DividendEligibility),
            A::DividendFundExclusion { .. } => variant_index!(Self::DividendFundExclusion),
            A::RetirementTreatment { .. } => variant_index!(Self::RetirementTreatment),
            A::RetirementMethod { .. } => variant_index!(Self::RetirementMethod),
            A::RetirementBasis { .. } => variant_index!(Self::RetirementBasis),
            A::RailroadRecipient { .. } => variant_index!(Self::RailroadRecipient),
            A::Community { .. } => variant_index!(Self::Community),
            A::RefundDeduction { .. } => variant_index!(Self::RefundDeduction),
            A::OpportunityFundEvent { .. } => variant_index!(Self::OpportunityFundEvent),
            A::OidTreatment { .. } => variant_index!(Self::OidTreatment),
            A::RentalSection179Business { .. } => variant_index!(Self::RentalSection179Business),
            A::RentalHomeCarries { .. } => variant_index!(Self::RentalHomeCarries),
            A::EsppWageInclusion { .. } => variant_index!(Self::EsppWageInclusion),
            A::IsoAdjustments { .. } => variant_index!(Self::IsoAdjustments),
            A::IsoWageInclusion { .. } => variant_index!(Self::IsoWageInclusion),
            A::NuaAdjustments { .. } => variant_index!(Self::NuaAdjustments),
            A::NuaOwnership { .. } => variant_index!(Self::NuaOwnership),
            A::SocialReimbursedPremiums { .. } => variant_index!(Self::SocialReimbursedPremiums),
            A::DebtTreatment { .. } => variant_index!(Self::DebtTreatment),
            A::DebtProperty { .. } => variant_index!(Self::DebtProperty),
            A::LtcBenefitOrigin { .. } => variant_index!(Self::LtcBenefitOrigin),
            A::LtcAcceleratedOrigin { .. } => variant_index!(Self::LtcAcceleratedOrigin),
            A::IsoExerciseFacts { .. } => variant_index!(Self::IsoExerciseFacts),
            A::HomeSale { .. } => variant_index!(Self::HomeSale),
            A::AbleTreatment { .. } => variant_index!(Self::AbleTreatment),
            A::EstateAccounting { .. } => variant_index!(Self::EstateAccounting),
            A::EstateReitEligibility { .. } => variant_index!(Self::EstateReitEligibility),
            A::K1EntityDisposition { .. } => variant_index!(Self::K1EntityDisposition),
            A::BrokerContractsTreatment { .. } => variant_index!(Self::BrokerContractsTreatment),
            A::K1RentalParticipation { .. } => variant_index!(Self::K1RentalParticipation),
            A::EstateResearchCreditLine { .. } => variant_index!(Self::EstateResearchCreditLine),
            A::K1ResearchCreditLine { .. } => variant_index!(Self::K1ResearchCreditLine),
            A::DebtAcquisition { .. } => variant_index!(Self::DebtAcquisition),
            A::EsppAcquisition { .. } => variant_index!(Self::EsppAcquisition),
            A::SavingsBondsRecords { .. } => variant_index!(Self::SavingsBondsRecords),
            A::SocialPremiums { .. } => variant_index!(Self::SocialPremiums),
            A::SocialSharedPremiums { .. } => variant_index!(Self::SocialSharedPremiums),
            A::EstateFinalSuccessor { .. } => variant_index!(Self::EstateFinalSuccessor),
            A::K1PartnerBasisStart { .. } => variant_index!(Self::K1PartnerBasisStart),
            A::K1ScorpBasisStart { .. } => variant_index!(Self::K1ScorpBasisStart),
            A::EducationBeneficiary { .. } => variant_index!(Self::EducationBeneficiary),
            A::MedicalPayment { .. } => variant_index!(Self::MedicalPayment),
            A::MedicareSpousal { .. } => variant_index!(Self::MedicareSpousal),
            A::MedicarePrior { .. } => variant_index!(Self::MedicarePrior),
            A::Management { .. } => variant_index!(Self::Management),
            A::K1PartnerParticipation { .. } => variant_index!(Self::K1PartnerParticipation),
            A::K1LimitedForSe { .. } => variant_index!(Self::K1LimitedForSe),
            A::K1PartnerAmt { .. } => variant_index!(Self::K1PartnerAmt),
            A::K1PartnerDisposition { .. } => variant_index!(Self::K1PartnerDisposition),
            A::K1ScorpParticipation { .. } => variant_index!(Self::K1ScorpParticipation),
            A::K1ScorpAmt { .. } => variant_index!(Self::K1ScorpAmt),
            A::K1ScorpDisposition { .. } => variant_index!(Self::K1ScorpDisposition),
            A::K1Section179Amt { .. } => variant_index!(Self::K1Section179Amt),
            A::MiscellaneousUse { .. } => variant_index!(Self::MiscellaneousUse),
            A::K1CreditConfirmations { .. } => variant_index!(Self::K1CreditConfirmations),
            A::BbaPriorWages { .. } => variant_index!(Self::BbaPriorWages),
            A::BbaPriorTaxableIncome { .. } => variant_index!(Self::BbaPriorTaxableIncome),
            A::BbaPriorIncomeTax { .. } => variant_index!(Self::BbaPriorIncomeTax),
            A::BbaPriorStatus { .. } => variant_index!(Self::BbaPriorStatus),
            A::BbaSingleAdjustment { .. } => variant_index!(Self::BbaSingleAdjustment),
            A::BbaSimplePriorReturn { .. } => variant_index!(Self::BbaSimplePriorReturn),
            A::BbaNoOtherEffects { .. } => variant_index!(Self::BbaNoOtherEffects),
            A::BbaLine17Interest { .. } => variant_index!(Self::BbaLine17Interest),
            A::BondClass { .. } => variant_index!(Self::BondClass),
            A::BondIssueDate { .. } => variant_index!(Self::BondIssueDate),
            A::BondOrigin { .. } => variant_index!(Self::BondOrigin),
            A::BondPriorCarry { .. } => variant_index!(Self::BondPriorCarry),
            A::BondAllowedShare { .. } => variant_index!(Self::BondAllowedShare),
            _ => return None,
        } as i32)
    }
}

// A retirement question whose one allowed choice carries nothing to answer is
// settled by the paper: the printed code or box 2a is the fact.
pub(crate) fn settled_answer(answer: &A, allowed: &[i32]) -> Option<A> {
    use crate::{RetirementTreatmentView as V, TaxableMethod as M};
    use libertas_macros::variant_index;
    let [only] = allowed else { return None };
    let only = *only as usize;
    match answer {
        // TaxableMethod's first member is BoxTwoA.
        A::RetirementMethod { .. } if only == 0 => {
            Some(A::RetirementMethod { method: Some(M::BoxTwoA) })
        }
        A::RetirementTreatment { .. } => {
            let code = [
                (variant_index!(V::Nonqualified), V::Nonqualified),
                (variant_index!(V::Other), V::Other),
                (variant_index!(V::Exchange), V::Exchange),
                (variant_index!(V::NotIncome), V::NotIncome),
                (variant_index!(V::NoAdditionalTax), V::NoAdditionalTax),
                (variant_index!(V::QualifiedCareCharge), V::QualifiedCareCharge),
                (variant_index!(V::EmployerRothContribution), V::EmployerRothContribution),
            ]
            .into_iter()
            .find_map(|(index, code)| (index == only).then_some(code))?;
            Some(A::RetirementTreatment { code: Some(code) })
        }
        _ => None,
    }
}

// Rare situations: the situations a canonical answer can stand for, as
// `DocumentSituationAnswer` variant indices. Empty for any other answer.
pub(crate) fn rare_situations(answer: &A) -> Vec<i32> {
    use libertas_macros::variant_index as v;
    let kinds: Vec<usize> = match answer {
        A::InterestNominee { .. } => alloc::vec![v!(S::InterestNominee)],
        A::DividendNominee { .. } => alloc::vec![v!(S::DividendNominee)],
        A::OidNominee { .. } => alloc::vec![v!(S::OidNominee)],
        A::InterestAdjustment { .. } => alloc::vec![v!(S::ReportingAdjustments)],
        A::InterestAccrued { .. } => alloc::vec![v!(S::AccruedInterest)],
        A::OidPremium { .. } => alloc::vec![v!(S::TaxExemptOidPremium)],
        A::OidPrivateActivity { .. } => alloc::vec![v!(S::PrivateActivityOid)],
        A::RetirementDisaster { .. } => alloc::vec![v!(S::DisasterDistribution)],
        A::RetirementPublicSafety { .. } => alloc::vec![v!(S::PublicSafetyPremiums)],
        A::RetirementDisability { .. } => alloc::vec![v!(S::DisabilityRetirement)],
        A::InterestAfterDeath { .. } => alloc::vec![v!(S::InterestAfterDeath)],
        A::DividendAfterDeath { .. } => alloc::vec![v!(S::DividendAfterDeath)],
        A::RetirementAfterDeath { .. } => alloc::vec![v!(S::PaymentsAfterDeath)],
        A::GovernmentRepayments { .. } => alloc::vec![v!(S::BenefitsRepaid)],
        A::GovernmentMedicalLeave { .. } => alloc::vec![v!(S::FamilyLeaveMedical)],
        A::BrokerContractsTreatment { .. } | A::ContractsSpecial { .. } => {
            alloc::vec![v!(S::ContractsHedge)]
        }
        A::InterestTreatment { .. } => alloc::vec![v!(S::InterestTreaty)],
        A::DividendTreatment { .. } => {
            alloc::vec![v!(S::ForeignDividendTreatment), v!(S::AnotherTreatment)]
        }
        A::UndistributedGainSpecial { .. }
        | A::OpportunityFundSpecial { .. }
        | A::GovernmentSpecial { .. }
        | A::GamblingSpecial { .. } => alloc::vec![v!(S::AnotherTreatment)],
        _ => Vec::new(),
    };
    kinds.into_iter().map(|kind| kind as i32).collect()
}

// The situation an accepted answer already records, if any.
pub(crate) fn rare_chosen(answer: &A) -> Option<S> {
    use crate::{DividendSpecial as D, InterestSpecial as I};
    match answer {
        A::InterestNominee { value } => value.clone().map(|value| S::InterestNominee { value }),
        A::DividendNominee { value } => value.clone().map(|value| S::DividendNominee { value }),
        A::OidNominee { value } => value.clone().map(|value| S::OidNominee { value }),
        A::InterestAdjustment { value } => {
            value.clone().map(|value| S::ReportingAdjustments { value })
        }
        A::InterestAccrued { value } => {
            (*value > 0).then_some(S::AccruedInterest { value: *value })
        }
        A::OidPremium { value } => (*value > 0).then_some(S::TaxExemptOidPremium { value: *value }),
        A::OidPrivateActivity { value } => {
            (*value > 0).then_some(S::PrivateActivityOid { value: *value })
        }
        A::RetirementDisaster { value } => value
            .clone()
            .filter(|value| *value != crate::DisasterDistribution::No)
            .map(|value| S::DisasterDistribution { value }),
        A::RetirementPublicSafety { value } => {
            value.clone().map(|value| S::PublicSafetyPremiums { value })
        }
        A::RetirementDisability {
            disability: Some(crate::DisabilityIncome::Qualifying { amount, eligible }),
        } => Some(S::DisabilityRetirement { amount: *amount, eligible: *eligible }),
        A::InterestAfterDeath { value } => {
            value.clone().map(|value| S::InterestAfterDeath { value })
        }
        A::DividendAfterDeath { value } => {
            value.clone().map(|value| S::DividendAfterDeath { value })
        }
        A::RetirementAfterDeath { value } => {
            value.clone().map(|value| S::PaymentsAfterDeath { value })
        }
        A::GovernmentRepayments { value } => {
            (!value.is_empty()).then(|| S::BenefitsRepaid { value: value.clone() })
        }
        A::GovernmentMedicalLeave { value } => value.then_some(S::FamilyLeaveMedical),
        A::BrokerContractsTreatment { value } => {
            (*value == Some(true)).then_some(S::ContractsHedge)
        }
        A::ContractsSpecial { value } => value.then_some(S::ContractsHedge),
        A::InterestTreatment { value } => (*value == I::Treaty).then_some(S::InterestTreaty),
        A::DividendTreatment { value: D::Another } => Some(S::AnotherTreatment),
        A::DividendTreatment { value: D::Foreign } => Some(S::ForeignDividendTreatment),
        A::UndistributedGainSpecial { value }
        | A::OpportunityFundSpecial { value }
        | A::GovernmentSpecial { value }
        | A::GamblingSpecial { value } => value.then_some(S::AnotherTreatment),
        _ => None,
    }
}

// The canonical answer for a rare situation: the chosen situation's facts, or
// the answer that none of the page's situations applies to this document.
// None when the chosen situation cannot apply to this answer.
pub(crate) fn rare_answer(answer: &A, chosen: Option<&S>) -> Option<A> {
    Some(match (answer, chosen) {
        (A::InterestNominee { .. }, None) => A::InterestNominee { value: None },
        (A::InterestNominee { .. }, Some(S::InterestNominee { value })) => {
            A::InterestNominee { value: Some(value.clone()) }
        }
        (A::DividendNominee { .. }, None) => A::DividendNominee { value: None },
        (A::DividendNominee { .. }, Some(S::DividendNominee { value })) => {
            A::DividendNominee { value: Some(value.clone()) }
        }
        (A::OidNominee { .. }, None) => A::OidNominee { value: None },
        (A::OidNominee { .. }, Some(S::OidNominee { value })) => {
            A::OidNominee { value: Some(value.clone()) }
        }
        (A::InterestAdjustment { .. }, None) => A::InterestAdjustment { value: None },
        (A::InterestAdjustment { .. }, Some(S::ReportingAdjustments { value })) => {
            A::InterestAdjustment { value: Some(value.clone()) }
        }
        (A::InterestAccrued { .. }, None) => A::InterestAccrued { value: 0 },
        (A::InterestAccrued { .. }, Some(S::AccruedInterest { value })) => {
            A::InterestAccrued { value: *value }
        }
        (A::OidPremium { .. }, None) => A::OidPremium { value: 0 },
        (A::OidPremium { .. }, Some(S::TaxExemptOidPremium { value })) => {
            A::OidPremium { value: *value }
        }
        (A::OidPrivateActivity { .. }, None) => A::OidPrivateActivity { value: 0 },
        (A::OidPrivateActivity { .. }, Some(S::PrivateActivityOid { value })) => {
            A::OidPrivateActivity { value: *value }
        }
        (A::RetirementDisaster { .. }, None) => A::RetirementDisaster { value: None },
        (A::RetirementDisaster { .. }, Some(S::DisasterDistribution { value })) => {
            A::RetirementDisaster { value: Some(value.clone()) }
        }
        (A::RetirementPublicSafety { .. }, None) => A::RetirementPublicSafety { value: None },
        (A::RetirementPublicSafety { .. }, Some(S::PublicSafetyPremiums { value })) => {
            A::RetirementPublicSafety { value: Some(value.clone()) }
        }
        (A::RetirementDisability { .. }, None) => A::RetirementDisability { disability: None },
        (A::RetirementDisability { .. }, Some(S::DisabilityRetirement { amount, eligible })) => {
            A::RetirementDisability {
                disability: Some(crate::DisabilityIncome::Qualifying {
                    amount: *amount,
                    eligible: *eligible,
                }),
            }
        }
        (A::InterestAfterDeath { .. }, None) => A::InterestAfterDeath { value: None },
        (A::InterestAfterDeath { .. }, Some(S::InterestAfterDeath { value })) => {
            A::InterestAfterDeath { value: Some(value.clone()) }
        }
        (A::DividendAfterDeath { .. }, None) => A::DividendAfterDeath { value: None },
        (A::DividendAfterDeath { .. }, Some(S::DividendAfterDeath { value })) => {
            A::DividendAfterDeath { value: Some(value.clone()) }
        }
        (A::RetirementAfterDeath { .. }, None) => A::RetirementAfterDeath { value: None },
        (A::RetirementAfterDeath { .. }, Some(S::PaymentsAfterDeath { value })) => {
            A::RetirementAfterDeath { value: Some(value.clone()) }
        }
        (A::GovernmentRepayments { .. }, None) => A::GovernmentRepayments { value: Vec::new() },
        (A::GovernmentRepayments { .. }, Some(S::BenefitsRepaid { value })) => {
            A::GovernmentRepayments { value: value.clone() }
        }
        (A::GovernmentMedicalLeave { .. }, None) => A::GovernmentMedicalLeave { value: false },
        (A::GovernmentMedicalLeave { .. }, Some(S::FamilyLeaveMedical)) => {
            A::GovernmentMedicalLeave { value: true }
        }
        _ => return rare_flag(answer, chosen),
    })
}

// Situations that are a choice and carry no facts of their own.
fn rare_flag(answer: &A, chosen: Option<&S>) -> Option<A> {
    use crate::{DividendSpecial as D, InterestSpecial as I};
    let hedge = matches!(chosen, Some(S::ContractsHedge));
    let another = matches!(chosen, Some(S::AnotherTreatment));
    let flag = |fits: bool| if chosen.is_none() { Some(false) } else { fits.then_some(true) };
    Some(match answer {
        A::BrokerContractsTreatment { .. } => {
            A::BrokerContractsTreatment { value: Some(flag(hedge)?) }
        }
        A::ContractsSpecial { .. } => A::ContractsSpecial { value: flag(hedge)? },
        A::UndistributedGainSpecial { .. } => A::UndistributedGainSpecial { value: flag(another)? },
        A::OpportunityFundSpecial { .. } => A::OpportunityFundSpecial { value: flag(another)? },
        A::GovernmentSpecial { .. } => A::GovernmentSpecial { value: flag(another)? },
        A::GamblingSpecial { .. } => A::GamblingSpecial { value: flag(another)? },
        A::InterestTreatment { .. } => A::InterestTreatment {
            value: match chosen {
                None => I::NoneOfThese,
                Some(S::InterestTreaty) => I::Treaty,
                Some(_) => return None,
            },
        },
        A::DividendTreatment { .. } => A::DividendTreatment {
            value: match chosen {
                None => D::NoneOfThese,
                Some(S::ForeignDividendTreatment) => D::Foreign,
                Some(S::AnotherTreatment) => D::Another,
                Some(_) => return None,
            },
        },
        _ => return None,
    })
}

impl DocumentSituationAnswer {
    // The variant's position, the value `allowed_situations` lists.
    pub(crate) fn index(&self) -> i32 {
        use libertas_macros::variant_index as v;
        (match self {
            Self::InterestNominee { .. } => v!(S::InterestNominee),
            Self::DividendNominee { .. } => v!(S::DividendNominee),
            Self::OidNominee { .. } => v!(S::OidNominee),
            Self::ReportingAdjustments { .. } => v!(S::ReportingAdjustments),
            Self::AccruedInterest { .. } => v!(S::AccruedInterest),
            Self::TaxExemptOidPremium { .. } => v!(S::TaxExemptOidPremium),
            Self::PrivateActivityOid { .. } => v!(S::PrivateActivityOid),
            Self::DisasterDistribution { .. } => v!(S::DisasterDistribution),
            Self::PublicSafetyPremiums { .. } => v!(S::PublicSafetyPremiums),
            Self::DisabilityRetirement { .. } => v!(S::DisabilityRetirement),
            Self::InterestAfterDeath { .. } => v!(S::InterestAfterDeath),
            Self::DividendAfterDeath { .. } => v!(S::DividendAfterDeath),
            Self::PaymentsAfterDeath { .. } => v!(S::PaymentsAfterDeath),
            Self::BenefitsRepaid { .. } => v!(S::BenefitsRepaid),
            Self::DailyTips { .. } => v!(S::DailyTips),
            Self::NoncashTips { .. } => v!(S::NoncashTips),
            Self::ChurchEmployeePay => v!(S::ChurchEmployeePay),
            Self::MinisterialPay => v!(S::MinisterialPay),
            Self::PayAbroad { .. } => v!(S::PayAbroad),
            Self::DeferredPension { .. } => v!(S::DeferredPension),
            Self::InmatePay { .. } => v!(S::InmatePay),
            Self::MedicaidWaiver { .. } => v!(S::MedicaidWaiver),
            Self::StatutoryBusiness { .. } => v!(S::StatutoryBusiness),
            Self::OwnScorporation { .. } => v!(S::OwnScorporation),
            Self::FamilyLeaveMedical => v!(S::FamilyLeaveMedical),
            Self::ContractsHedge => v!(S::ContractsHedge),
            Self::InterestTreaty => v!(S::InterestTreaty),
            Self::ForeignDividendTreatment => v!(S::ForeignDividendTreatment),
            Self::AnotherTreatment => v!(S::AnotherTreatment),
        }) as i32
    }
}

impl DocumentStrictAnswer {
    // The variant's position, the value `allowed_answers` lists.
    pub(crate) fn index(&self) -> i32 {
        use libertas_macros::variant_index as v;
        (match self {
            Self::DividendEligibility { .. } => v!(Self::DividendEligibility),
            Self::DividendFundExclusion { .. } => v!(Self::DividendFundExclusion),
            Self::RetirementTreatment { .. } => v!(Self::RetirementTreatment),
            Self::RetirementMethod { .. } => v!(Self::RetirementMethod),
            Self::RetirementBasis { .. } => v!(Self::RetirementBasis),
            Self::RailroadRecipient { .. } => v!(Self::RailroadRecipient),
            Self::Community { .. } => v!(Self::Community),
            Self::RefundDeduction { .. } => v!(Self::RefundDeduction),
            Self::OpportunityFundEvent { .. } => v!(Self::OpportunityFundEvent),
            Self::OidTreatment { .. } => v!(Self::OidTreatment),
            Self::WageOvertime { .. } => v!(Self::WageOvertime),
            Self::WageTipOccupation { .. } => v!(Self::WageTipOccupation),
            Self::WageBox14 { .. } => v!(Self::WageBox14),
            Self::RentalSection179Business { .. } => v!(Self::RentalSection179Business),
            Self::RentalHomeCarries { .. } => v!(Self::RentalHomeCarries),
            Self::EsppWageInclusion { .. } => v!(Self::EsppWageInclusion),
            Self::IsoAdjustments { .. } => v!(Self::IsoAdjustments),
            Self::IsoWageInclusion { .. } => v!(Self::IsoWageInclusion),
            Self::NuaAdjustments { .. } => v!(Self::NuaAdjustments),
            Self::NuaOwnership { .. } => v!(Self::NuaOwnership),
            Self::SocialReimbursedPremiums { .. } => v!(Self::SocialReimbursedPremiums),
            Self::DebtTreatment { .. } => v!(Self::DebtTreatment),
            Self::DebtProperty { .. } => v!(Self::DebtProperty),
            Self::LtcBenefitOrigin { .. } => v!(Self::LtcBenefitOrigin),
            Self::LtcAcceleratedOrigin { .. } => v!(Self::LtcAcceleratedOrigin),
            Self::IsoExerciseFacts { .. } => v!(Self::IsoExerciseFacts),
            Self::HomeSale { .. } => v!(Self::HomeSale),
            Self::AbleTreatment { .. } => v!(Self::AbleTreatment),
            Self::EstateAccounting { .. } => v!(Self::EstateAccounting),
            Self::EstateReitEligibility { .. } => v!(Self::EstateReitEligibility),
            Self::K1EntityDisposition { .. } => v!(Self::K1EntityDisposition),
            Self::BrokerContractsTreatment { .. } => v!(Self::BrokerContractsTreatment),
            Self::K1RentalParticipation { .. } => v!(Self::K1RentalParticipation),
            Self::EstateResearchCreditLine { .. } => v!(Self::EstateResearchCreditLine),
            Self::K1ResearchCreditLine { .. } => v!(Self::K1ResearchCreditLine),
            Self::DebtAcquisition { .. } => v!(Self::DebtAcquisition),
            Self::EsppAcquisition { .. } => v!(Self::EsppAcquisition),
            Self::SavingsBondsRecords { .. } => v!(Self::SavingsBondsRecords),
            Self::SocialPremiums { .. } => v!(Self::SocialPremiums),
            Self::SocialSharedPremiums { .. } => v!(Self::SocialSharedPremiums),
            Self::EstateFinalSuccessor { .. } => v!(Self::EstateFinalSuccessor),
            Self::K1PartnerBasisStart { .. } => v!(Self::K1PartnerBasisStart),
            Self::K1ScorpBasisStart { .. } => v!(Self::K1ScorpBasisStart),
            Self::EducationBeneficiary { .. } => v!(Self::EducationBeneficiary),
            Self::MedicalPayment { .. } => v!(Self::MedicalPayment),
            Self::MedicareSpousal { .. } => v!(Self::MedicareSpousal),
            Self::MedicarePrior { .. } => v!(Self::MedicarePrior),
            Self::Management { .. } => v!(Self::Management),
            Self::K1PartnerParticipation { .. } => v!(Self::K1PartnerParticipation),
            Self::K1LimitedForSe { .. } => v!(Self::K1LimitedForSe),
            Self::K1PartnerAmt { .. } => v!(Self::K1PartnerAmt),
            Self::K1PartnerDisposition { .. } => v!(Self::K1PartnerDisposition),
            Self::K1ScorpParticipation { .. } => v!(Self::K1ScorpParticipation),
            Self::K1ScorpAmt { .. } => v!(Self::K1ScorpAmt),
            Self::K1ScorpDisposition { .. } => v!(Self::K1ScorpDisposition),
            Self::K1Section179Amt { .. } => v!(Self::K1Section179Amt),
            Self::MiscellaneousUse { .. } => v!(Self::MiscellaneousUse),
            Self::K1CreditConfirmations { .. } => v!(Self::K1CreditConfirmations),
            Self::BbaPriorWages { .. } => v!(Self::BbaPriorWages),
            Self::BbaPriorTaxableIncome { .. } => v!(Self::BbaPriorTaxableIncome),
            Self::BbaPriorIncomeTax { .. } => v!(Self::BbaPriorIncomeTax),
            Self::BbaPriorStatus { .. } => v!(Self::BbaPriorStatus),
            Self::BbaSingleAdjustment { .. } => v!(Self::BbaSingleAdjustment),
            Self::BbaSimplePriorReturn { .. } => v!(Self::BbaSimplePriorReturn),
            Self::BbaNoOtherEffects { .. } => v!(Self::BbaNoOtherEffects),
            Self::BbaLine17Interest { .. } => v!(Self::BbaLine17Interest),
            Self::BondClass { .. } => v!(Self::BondClass),
            Self::BondIssueDate { .. } => v!(Self::BondIssueDate),
            Self::BondOrigin { .. } => v!(Self::BondOrigin),
            Self::BondPriorCarry { .. } => v!(Self::BondPriorCarry),
            Self::BondAllowedShare { .. } => v!(Self::BondAllowedShare),
            Self::EmployerYear { .. } => v!(Self::EmployerYear),
            Self::MortgageRefundOrigin { .. } => v!(Self::MortgageRefundOrigin),
            Self::OtherHsaEligibility { .. } => v!(Self::OtherHsaEligibility),
        }) as i32
    }
}

// A W-2's own questions, by the part of its answers each one fills.
pub(crate) fn wage_questions(
    answers: &crate::W2AnswersView,
) -> Vec<(alloc::string::String, Option<DocumentStrictAnswer>)> {
    use DocumentStrictAnswer as T;
    let mut questions = Vec::new();
    if !answers.calls.overtime.is_empty() {
        questions.push((
            "overtime".into(),
            answers.overtime_flsa_required.map(|value| T::WageOvertime { value }),
        ));
    }
    if !answers.calls.occupation.is_empty() {
        questions.push((
            "occupation".into(),
            answers.tips_listed_occupation.map(|value| T::WageTipOccupation { value }),
        ));
    }
    // One question for each box 14 item the label and box 15 could not settle.
    for ask in &answers.box14 {
        let answer = ask.kind.map(|kind| T::WageBox14 { kind, state: ask.state });
        questions.push((alloc::format!("box14/{}", ask.item), answer));
    }
    questions
}

pub(crate) fn apply_wage_question(
    answers: &mut crate::W2AnswersView,
    part: &str,
    answer: DocumentStrictAnswer,
) -> bool {
    match answer {
        DocumentStrictAnswer::WageBox14 { kind, state } => {
            let Some(ask) = part
                .strip_prefix("box14/")
                .and_then(|item| item.parse::<u32>().ok())
                .and_then(|item| answers.box14.iter_mut().find(|ask| ask.item == item))
            else {
                return false;
            };
            (ask.kind, ask.state) = (Some(kind), state);
        }
        DocumentStrictAnswer::WageOvertime { value } => {
            answers.overtime_flsa_required = Some(value)
        }
        DocumentStrictAnswer::WageTipOccupation { value } => {
            answers.tips_listed_occupation = Some(value)
        }
        _ => return false,
    }
    true
}

// The rare situations a W-2's boxes and the return offer.
pub(crate) fn wage_situations(answers: &crate::W2AnswersView, scorporations: bool) -> Vec<i32> {
    use libertas_macros::variant_index as v;
    let calls = &answers.calls;
    let mut kinds = Vec::new();
    if !calls.tips.is_empty() {
        kinds.extend([v!(S::DailyTips), v!(S::NoncashTips)]);
    }
    if !calls.coverage.is_empty() {
        kinds.extend([v!(S::ChurchEmployeePay), v!(S::MinisterialPay)]);
    }
    if !calls.disability.is_empty() {
        kinds.push(v!(S::DisabilityRetirement));
    }
    if !calls.abroad.is_empty() {
        kinds.push(v!(S::PayAbroad));
    }
    if !calls.deferred_pension.is_empty() {
        kinds.push(v!(S::DeferredPension));
    }
    kinds.extend([v!(S::InmatePay), v!(S::MedicaidWaiver)]);
    if !calls.statutory_business.is_empty() {
        kinds.push(v!(S::StatutoryBusiness));
    }
    if scorporations {
        kinds.push(v!(S::OwnScorporation));
    }
    kinds.into_iter().map(|kind| kind as i32).collect()
}

// The situations a W-2's saved answers record.
pub(crate) fn wage_chosen(answers: &crate::W2AnswersView) -> Vec<S> {
    let mut chosen = Vec::new();
    if let Some(value) = &answers.tip_records {
        chosen.push(S::DailyTips { value: value.clone() });
    }
    if let Some(value) = answers.noncash_tips {
        chosen.push(S::NoncashTips { value });
    }
    match answers.coverage {
        Some(crate::W2Coverage::ChurchEmployee) => chosen.push(S::ChurchEmployeePay),
        Some(crate::W2Coverage::Ministerial) => chosen.push(S::MinisterialPay),
        _ => {}
    }
    if let Some(crate::DisabilityIncome::Qualifying { amount, eligible }) = &answers.disability {
        chosen.push(S::DisabilityRetirement { amount: *amount, eligible: *eligible });
    }
    if let Some(value) = answers.abroad {
        chosen.push(S::PayAbroad { value });
    }
    if let Some(value) = answers.deferred_pension {
        chosen.push(S::DeferredPension { value });
    }
    if answers.inmate_pay > 0 {
        chosen.push(S::InmatePay { value: answers.inmate_pay });
    }
    if answers.medicaid_waiver > 0 {
        chosen.push(S::MedicaidWaiver { value: answers.medicaid_waiver });
    }
    if let Some(value) = answers.statutory_business {
        chosen.push(S::StatutoryBusiness { value });
    }
    if let Some(value) = &answers.paying_scorporation {
        chosen.push(S::OwnScorporation { value: value.clone() });
    }
    chosen
}

// Every situation the W-2 offers takes the chosen facts, or none.
pub(crate) fn apply_wage_situations(
    answers: &mut crate::W2AnswersView,
    offered: &[i32],
    chosen: &[&S],
) {
    let is = |kind: usize| offered.contains(&(kind as i32));
    use libertas_macros::variant_index as v;
    let pick = |kind: usize| chosen.iter().copied().find(|s| s.index() == kind as i32);
    if is(v!(S::DailyTips)) {
        answers.tip_records = match pick(v!(S::DailyTips)) {
            Some(S::DailyTips { value }) => Some(value.clone()),
            _ => None,
        };
        answers.noncash_tips = match pick(v!(S::NoncashTips)) {
            Some(S::NoncashTips { value }) => Some(*value),
            _ => None,
        };
    }
    if is(v!(S::ChurchEmployeePay)) {
        answers.coverage = if pick(v!(S::ChurchEmployeePay)).is_some() {
            Some(crate::W2Coverage::ChurchEmployee)
        } else if pick(v!(S::MinisterialPay)).is_some() {
            Some(crate::W2Coverage::Ministerial)
        } else {
            None
        };
    }
    if is(v!(S::DisabilityRetirement)) {
        answers.disability = match pick(v!(S::DisabilityRetirement)) {
            Some(S::DisabilityRetirement { amount, eligible }) => {
                Some(crate::DisabilityIncome::Qualifying { amount: *amount, eligible: *eligible })
            }
            _ => None,
        };
    }
    if is(v!(S::PayAbroad)) {
        answers.abroad = match pick(v!(S::PayAbroad)) {
            Some(S::PayAbroad { value }) => Some(*value),
            _ => None,
        };
    }
    if is(v!(S::DeferredPension)) {
        answers.deferred_pension = match pick(v!(S::DeferredPension)) {
            Some(S::DeferredPension { value }) => Some(*value),
            _ => None,
        };
    }
    answers.inmate_pay = match pick(v!(S::InmatePay)) {
        Some(S::InmatePay { value }) => *value,
        _ => 0,
    };
    answers.medicaid_waiver = match pick(v!(S::MedicaidWaiver)) {
        Some(S::MedicaidWaiver { value }) => *value,
        _ => 0,
    };
    if is(v!(S::StatutoryBusiness)) {
        answers.statutory_business = match pick(v!(S::StatutoryBusiness)) {
            Some(S::StatutoryBusiness { value }) => Some(*value),
            _ => None,
        };
    }
    if is(v!(S::OwnScorporation)) {
        answers.paying_scorporation = match pick(v!(S::OwnScorporation)) {
            Some(S::OwnScorporation { value }) => Some(value.clone()),
            _ => None,
        };
    }
}

/// When you became a partner
// Strict-page view of the stored `BasisStart`: the opening basis and its
// at-risk history are answered, never left unestablished.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum StrictPartnerStart {
    /// You became a partner in 2026
    New,
    /// You were a partner before 2026
    Continuing {
        /// Tax basis at the beginning of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: i64,
        /// Your at-risk history
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        carried: crate::PartnerCarried,
    },
}

/// When you became a shareholder
// Strict-page view of the stored `K1ScorpStart`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum StrictScorpStart {
    /// In 2026
    New,
    /// Before 2026
    Continuing {
        /// Stock basis at the beginning of 2026 (Form 7203 line 1)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: i64,
        /// Your at-risk history
        #[libertas_constrained_by("^<DocumentStrictQuestion>.allowed_values")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        carried: crate::K1ScorpCarried,
    },
}

impl StrictPartnerStart {
    // None while a continuing interest's opening basis is unanswered.
    pub(crate) fn from_stored(start: &crate::BasisStart) -> Option<Self> {
        Some(match start {
            crate::BasisStart::New => Self::New,
            crate::BasisStart::Continuing { opening, risk_history, carried } => Self::Continuing {
                opening: (*opening)?,
                risk_history: risk_history.clone(),
                carried: carried.clone(),
            },
        })
    }

    pub(crate) fn stored(self) -> crate::BasisStart {
        match self {
            Self::New => crate::BasisStart::New,
            Self::Continuing { opening, risk_history, carried } => {
                crate::BasisStart::Continuing { opening: Some(opening), risk_history, carried }
            }
        }
    }
}

impl StrictScorpStart {
    pub(crate) fn from_stored(start: &crate::K1ScorpStart) -> Option<Self> {
        Some(match start {
            crate::K1ScorpStart::New => Self::New,
            crate::K1ScorpStart::Continuing { opening, risk_history, carried } => {
                Self::Continuing {
                    opening: (*opening)?,
                    risk_history: risk_history.clone(),
                    carried: carried.clone(),
                }
            }
        })
    }

    pub(crate) fn stored(self) -> crate::K1ScorpStart {
        match self {
            Self::New => crate::K1ScorpStart::New,
            Self::Continuing { opening, risk_history, carried } => {
                crate::K1ScorpStart::Continuing { opening: Some(opening), risk_history, carried }
            }
        }
    }
}

/// What the property was
// Strict-page sibling of `AcquiredPropertyView`: the forfeited loan is
// chosen from the strict question's own `ccc_lots`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum StrictAcquiredProperty {
    /// A crop forfeited to the Commodity Credit Corporation
    /// The crop pledged for one of the CCC loans entered on your farm or farm rental. The loan is counted there, not again here.
    CccCrop {
        /// Forfeited loan
        /// A loan marked forfeited in 2026 whose principal equals box 2. A farm entered on this same page is chosen after you save it.
        #[libertas_enum_source("^<DocumentStrictQuestion>.ccc_lots")]
        loan: i64,
    },
    /// Other property
    Property {
        /// What the property was
        kind: crate::TransferredProperty,
    },
}

impl From<crate::AcquiredPropertyView> for StrictAcquiredProperty {
    fn from(property: crate::AcquiredPropertyView) -> Self {
        match property {
            crate::AcquiredPropertyView::CccCrop { loan } => Self::CccCrop { loan },
            crate::AcquiredPropertyView::Property { kind } => Self::Property { kind },
        }
    }
}

impl From<StrictAcquiredProperty> for crate::AcquiredPropertyView {
    fn from(property: StrictAcquiredProperty) -> Self {
        match property {
            StrictAcquiredProperty::CccCrop { loan } => Self::CccCrop { loan },
            StrictAcquiredProperty::Property { kind } => Self::Property { kind },
        }
    }
}

/// Personal source links and education records supporting Form 8815
// Strict-page sibling of `SavingsBondAnswersView`: the interest statement and
// the students come from the strict question's own `interest_statements` and
// `people`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StrictSavingsBondAnswers {
    /// Accepted interest statement or annual instrument for these bonds
    #[libertas_enum_source("^<DocumentStrictQuestion>.interest_statements")]
    pub interest_statement: i64,
    /// People whose education you paid for in 2026
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    /// ----
    /// Person
    pub students: Vec<StrictSavingsBondStudent>,
}

/// Person whose education you paid for
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StrictSavingsBondStudent {
    /// Person
    #[libertas_enum_source("^<DocumentStrictQuestion>.people")]
    #[libertas_unique]
    pub person: i64,
    /// Tuition and fees
    #[libertas_default(EducationCredit)]
    pub expenses: crate::SavingsBondExpenses,
    /// Contributions in 2026 to a 529 plan or Coverdell ESA for this person
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contributions: i64,
}

impl From<crate::SavingsBondAnswersView> for StrictSavingsBondAnswers {
    fn from(value: crate::SavingsBondAnswersView) -> Self {
        let crate::SavingsBondAnswersView { interest_statement, students } = value;
        Self {
            interest_statement,
            students: students
                .into_iter()
                .map(|crate::SavingsBondStudent { person, expenses, contributions }| {
                    StrictSavingsBondStudent { person, expenses, contributions }
                })
                .collect(),
        }
    }
}

impl From<StrictSavingsBondAnswers> for crate::SavingsBondAnswersView {
    fn from(value: StrictSavingsBondAnswers) -> Self {
        let StrictSavingsBondAnswers { interest_statement, students } = value;
        Self {
            interest_statement,
            students: students
                .into_iter()
                .map(|StrictSavingsBondStudent { person, expenses, contributions }| {
                    crate::SavingsBondStudent { person, expenses, contributions }
                })
                .collect(),
        }
    }
}

/// Acquisition source
// Strict-page sibling of `EsppAcquisitionPage`: the Form 3922 is chosen
// from the strict question's own `espp_acquisitions`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StrictEsppAcquisition {
    /// A Form 3922 already entered in Documents
    Printed {
        /// Acquisition document
        #[libertas_enum_source("^<DocumentStrictQuestion>.espp_acquisitions")]
        document_id: i64,
    },
    /// Actual acquisition records when no Form 3922 is available
    Records { facts: crate::EsppAcquisitionFacts },
}

impl From<crate::EsppAcquisitionPage> for StrictEsppAcquisition {
    fn from(value: crate::EsppAcquisitionPage) -> Self {
        match value {
            crate::EsppAcquisitionPage::Printed { document_id } => Self::Printed { document_id },
            crate::EsppAcquisitionPage::Records { facts } => Self::Records { facts },
        }
    }
}

impl From<StrictEsppAcquisition> for crate::EsppAcquisitionPage {
    fn from(value: StrictEsppAcquisition) -> Self {
        match value {
            StrictEsppAcquisition::Printed { document_id } => Self::Printed { document_id },
            StrictEsppAcquisition::Records { facts } => Self::Records { facts },
        }
    }
}

/// Prevent the same Medicare premium from entering two expense sources
// Strict-page sibling of `SocialPremiumUsePage`: the plan is chosen from the
// strict question's own `health_plans`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StrictSocialPremiumUse {
    /// None included in a self-employed health plan
    /// The statement premiums enter itemized medical expenses automatically; do not type them again.
    Medical,
    /// Some or all already included in a self-employed health plan
    AlreadyInSelfEmployedPlan {
        /// Business whose health coverage already includes these premiums
        #[libertas_enum_source("^<DocumentStrictQuestion>.health_plans")]
        income_id: i64,
        /// Part of the printed premiums already included in that plan's monthly premiums
        /// This is an allocation of an existing expense, not an additional premium.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

impl From<crate::SocialPremiumUsePage> for StrictSocialPremiumUse {
    fn from(value: crate::SocialPremiumUsePage) -> Self {
        match value {
            crate::SocialPremiumUsePage::Medical => Self::Medical,
            crate::SocialPremiumUsePage::AlreadyInSelfEmployedPlan { income_id, amount } => {
                Self::AlreadyInSelfEmployedPlan { income_id, amount }
            }
        }
    }
}

impl From<StrictSocialPremiumUse> for crate::SocialPremiumUsePage {
    fn from(value: StrictSocialPremiumUse) -> Self {
        match value {
            StrictSocialPremiumUse::Medical => Self::Medical,
            StrictSocialPremiumUse::AlreadyInSelfEmployedPlan { income_id, amount } => {
                Self::AlreadyInSelfEmployedPlan { income_id, amount }
            }
        }
    }
}

/// How the credit reached you
// Strict-page sibling of `TaxCreditOriginInput`: the already-reported source
// is chosen from the strict question's own `credit_income_sources`.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StrictTaxCreditOrigin {
    /// You directly held the bond
    Direct(crate::TaxCreditDirect),
    /// Entity Form 1097-BTC
    Entity1097(StrictTaxCreditEntity),
    /// Other entity statement
    EntityStatement(StrictTaxCreditEntity),
}

/// An entity's designated credit-related income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StrictTaxCreditEntity {
    /// Entity name from its report
    #[libertas_size(min = 1)]
    pub entity: alloc::string::String,
    /// Current distribution or interest designated by the entity
    /// From actual entity records, not an assumed second inclusion of box 1 credit. Unreported RIC income enters ordinary dividends. To retain any issuer-designated qualified amount, enter it with the actual DIV first and link that already-reported distribution.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub designated_income: i64,
    /// Designated income already included in the linked accepted source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income_already_reported: i64,
    /// Income character from the entity's statement
    pub income_kind: crate::TaxCreditIncomeKind,
    /// Accepted source containing the already reported income
    /// [NullText]
    /// No already-reported income is being reused
    /// [SetText]
    /// The already reported designated income is in
    #[libertas_enum_source("^<DocumentStrictQuestion>.credit_income_sources")]
    #[libertas_default($null)]
    pub income_source: Option<i64>,
}

impl From<crate::TaxCreditEntity> for StrictTaxCreditEntity {
    fn from(v: crate::TaxCreditEntity) -> Self {
        Self {
            entity: v.entity,
            designated_income: v.designated_income,
            income_already_reported: v.income_already_reported,
            income_kind: v.income_kind,
            income_source: v.income_source,
        }
    }
}

impl From<StrictTaxCreditEntity> for crate::TaxCreditEntity {
    fn from(v: StrictTaxCreditEntity) -> Self {
        Self {
            entity: v.entity,
            designated_income: v.designated_income,
            income_already_reported: v.income_already_reported,
            income_kind: v.income_kind,
            income_source: v.income_source,
        }
    }
}

impl From<crate::TaxCreditOrigin> for StrictTaxCreditOrigin {
    fn from(v: crate::TaxCreditOrigin) -> Self {
        match v {
            crate::TaxCreditOrigin::Direct(v) => Self::Direct(v),
            crate::TaxCreditOrigin::Entity1097(v) => Self::Entity1097(v.into()),
            crate::TaxCreditOrigin::EntityStatement(v) => Self::EntityStatement(v.into()),
        }
    }
}

impl From<StrictTaxCreditOrigin> for crate::TaxCreditOrigin {
    fn from(v: StrictTaxCreditOrigin) -> Self {
        match v {
            StrictTaxCreditOrigin::Direct(v) => Self::Direct(v),
            StrictTaxCreditOrigin::Entity1097(v) => Self::Entity1097(v.into()),
            StrictTaxCreditOrigin::EntityStatement(v) => Self::EntityStatement(v.into()),
        }
    }
}

/// Answer the questions about your documents
// A topic page's one link to its strict document questions: shown while any
// question about its documents is unanswered, or (Mortgage) to correct an answer.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
#[libertas_link("OpenDocumentQuestions")]
pub struct DocumentQuestionsLink {
    /// Topic
    #[libertas_hidden]
    #[libertas_read_only]
    pub topic: crate::TaxSection,
    /// Questions about your documents
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub status: Vec<u8>,
}

/// How the credit reached you
// The More credits page's editable answer: as `StrictTaxCreditOrigin`, but the
// already-reported source is chosen from the canonical row's own list.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CreditOriginAnswer {
    /// You directly held the bond
    Direct(crate::TaxCreditDirect),
    /// Entity Form 1097-BTC
    Entity1097(CreditEntityAnswer),
    /// Other entity statement
    EntityStatement(CreditEntityAnswer),
}

/// An entity's designated credit-related income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditEntityAnswer {
    /// Entity name from its report
    #[libertas_size(min = 1)]
    pub entity: alloc::string::String,
    /// Current distribution or interest designated by the entity
    /// From actual entity records, not an assumed second inclusion of box 1 credit. Unreported RIC income enters ordinary dividends. To retain any issuer-designated qualified amount, enter it with the actual DIV first and link that already-reported distribution.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub designated_income: i64,
    /// Designated income already included in the linked accepted source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income_already_reported: i64,
    /// Income character from the entity's statement
    pub income_kind: crate::TaxCreditIncomeKind,
    /// Accepted source containing the already reported income
    /// [NullText]
    /// No already-reported income is being reused
    /// [SetText]
    /// The already reported designated income is in
    #[libertas_enum_source("^<CreditDocumentQuestion>.credit_income_sources")]
    #[libertas_default($null)]
    pub income_source: Option<i64>,
}

macro_rules! same_entity {
    ($from:ty => $to:ty) => {
        impl From<$from> for $to {
            fn from(v: $from) -> Self {
                Self {
                    entity: v.entity,
                    designated_income: v.designated_income,
                    income_already_reported: v.income_already_reported,
                    income_kind: v.income_kind,
                    income_source: v.income_source,
                }
            }
        }
    };
}
same_entity!(CreditEntityAnswer => crate::TaxCreditEntity);
same_entity!(crate::TaxCreditEntity => CreditEntityAnswer);
same_entity!(CreditEntityAnswer => StrictTaxCreditEntity);
same_entity!(StrictTaxCreditEntity => CreditEntityAnswer);

macro_rules! same_origin {
    ($from:ident => $to:ident) => {
        impl From<$from> for $to {
            fn from(v: $from) -> Self {
                match v {
                    $from::Direct(v) => Self::Direct(v),
                    $from::Entity1097(v) => Self::Entity1097(v.into()),
                    $from::EntityStatement(v) => Self::EntityStatement(v.into()),
                }
            }
        }
    };
}
use crate::TaxCreditOrigin;
same_origin!(CreditOriginAnswer => TaxCreditOrigin);
same_origin!(TaxCreditOrigin => CreditOriginAnswer);
same_origin!(CreditOriginAnswer => StrictTaxCreditOrigin);
same_origin!(StrictTaxCreditOrigin => CreditOriginAnswer);
