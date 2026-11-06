//! Tax return prototype
//! #[libertas_string_resources(CORE_STRINGS, ACTIVITY_STRINGS, ATTRIBUTION_STRINGS, BUSINESS_STRINGS, COST_RECOVERY_STRINGS, CREDIT_STRINGS, DEDUCTION_STRINGS, FAMILY_STRINGS, FARM_STRINGS, FEDERAL_STRINGS, FOREIGN_STRINGS, FORM_STRINGS, HEALTH_STRINGS, HOME_STRINGS, INCOME_STRINGS, INVESTMENT_STRINGS, PASSTHROUGH_STRINGS, PAYMENT_STRINGS, RETIREMENT_STRINGS)]
//! Interview, saved accepted answers, and a U.S. federal tax year 2026 estimate.
//! Use synthetic data only. This prototype does not file a return.

// Coverage evidence from the Phase 1 audit (2026-09-24) lives in `// Coverage:` blocks at the top of
// each form's owning module. Keep it in plain `//` comments: `///` and `//!` text becomes UI labels,
// help and package documentation. The forms below have no owning module; they are refused only
// by the generic coverage gate in federal_rules.rs.
// Form 8379's coverage (T3-8379, 2026-09-27) is in injured_spouse.rs.
// Form 8938: the requirement check lives in foreign_assets.rs (Partial, T3-small).
// Coverage: Schedule 1-A Part IV (qualified passenger vehicle loan interest) — Complete for the
//   estimate (T3-deduct, 2026-09-28): federal_rules.rs additional_deductions and the vehicle_loans
//   loop. Asked on the deductions page's "Interest on a loan for a new personal vehicle" list (one
//   row per Form 1098-VLI: lender, box 1 interest, the part deducted on Schedule C/E/F (column ii),
//   the line 28 questions (1) original use and (2) U.S. final assembly). Column (iii) is interest
//   less column (ii) for a loan answered Yes to both; a No gives nothing; Not sure is refused by
//   name. Lines 29-36: $10,000 cap, $200 per $1,000 or part of MAGI above $100,000 ($200,000
//   joint). VINs are not collected (printing deferred). The CarLoanInterest refusal is removed.
//   Client rule VEHICLE_BUSINESS_ABOVE_INTEREST with its server twin. Tests:
//   src/tests/deductions/deduction_pages.rs vehicle_loans_reach_schedule_1a_through_the_deductions_page (3,500),
//   vehicle_interest_is_capped_at_10000, vehicle_interest_phases_out_by_200_per_started_1000,
//   vehicle_business_part_above_interest_is_refused; src/tests/business/credits.rs
//   vehicle_interest_aggregates_loans_and_does_not_reduce_agi. Source: 2026 draft f1040s1a.
// Coverage: Form 4835 — see farm_rental.rs (unit B2, 2026-10-09).
// Coverage: Form 2106 — see employee_expenses.rs (2026-10-09).
// Coverage: Form 6781 — see section1256.rs (unit B2, 2026-10-09).
// Coverage: Form 3115 — see method_change.rs (unit 481a, 2026-10-09).
// Coverage: Forms 8332 and 2120 — Supported for their tax effects (unit form-8332, 2026-10-09).
//   Lines: the operative facts of the declarations, stated once on the people page's per-person
//   fold (PersonFacts YOU_RELEASED, RELEASED_TO_YOU, MULTIPLE_SUPPORT; stored as the bits
//   themselves in FederalDependent.facts and read through FederalDependent::custody_release and
//   multiple_support): Form 8332 Parts I and II (the 2026
//   release; a 1985-2008 decree or a pre-1985 instrument with $600 of support is the same fact),
//   Part III (a revocation delivered in 2025 or earlier is the absence of the fact) and Form 2120
//   lines 1-3 and tests 1-5 (the signed statements). IRC 152(e)(1)-(2) waives the residency,
//   tie-breaker and support tests for the parent holding the release: the child (or the
//   qualifying relative, any age) is the parent's dependent for the child tax credit or credit for
//   other dependents, the education credits and the Marketplace family, but never the parent's
//   head of household, earned income or dependent care person; the custodial parent who released
//   keeps head of household (IRC 2(b)(1)(A)(i) without 152(e)), the earned income credit (IRC
//   32(c)(3)(A)), the 7703(b)(1) considered-unmarried path and the Form 2441 qualifying
//   individual (IRC 21(e)(5)) but not the dependent, the credits or a qualifying surviving spouse.
//   IRC 152(d)(3) replaces the over-half support test of a qualifying relative and gives the
//   credit for other dependents but never head of household (IRC 2(b)(3)(B)(ii)); the
//   other tests, including gross income under $5,300, still apply. Six client rules over the
//   Bitflags facts with server twins in interview.rs dependent_rule (SUPPORTED_NEEDS_YOUR_SUPPORT
//   extended, CUSTODY_RELEASE_ONE_WAY, CUSTODY_RELEASE_NEEDS_YOUR_CHILD,
//   CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS, MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT,
//   MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE); notes NOTE_FORM_8332_ATTACH and
//   NOTE_FORM_2120_ATTACH. The competing claim is FederalDependent.competing (unit B9-3). The same unit stores the people page's situation
//   (FederalDependent.situation, the owner of the student, disabled, support and gross-income
//   tests) and a married-at-year-end fact (PersonFacts MARRIED_AT_YEAR_END) that replaces the
//   earlier head-of-household unknown for a non-citizen child. PersonFacts OWN_SUPPORT asks the
//   IRC 152(c)(1)(D) own-support test of a qualifying child (two more client rules with server
//   twins: OWN_SUPPORT_NOT_WITH_YOUR_SUPPORT, OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT); the earned
//   income credit ignores it (IRC 32(c)(3)(A)). Test:
//   a_child_who_pays_over_half_of_their_own_support_is_not_a_qualifying_child.
//   Exclusions: preparing, signing or attaching the declarations (names, SSNs, signatures, dates,
//   other supporters, Form 2120 lines 4-7), the Form 1040 checkbox and name items, the custodial
//   parent's nights and the section 152(e)(1) support-from-parents and living-apart conditions
//   (the person states them in the flag text), the tie-breaker, and the pre-1985 $600 amount.
//   Evidence: federal_rules.rs family() and refuse_person_facts, federal_model.rs PersonFacts,
//   CustodyRelease, FederalDependent (situation and facts) and the Dependent rules, interview.rs dependent_rule,
//   federal_dependent and dependent_item; tests: noncustodial_parent_with_a_release_gets_the_child_tax_credit_only,
//   custodial_parent_who_released_keeps_head_of_household_eic_and_care,
//   multiple_support_agreement_gives_the_credit_for_other_dependents_not_head_of_household,
//   released_child_aged_17_gets_the_credit_for_other_dependents,
//   released_qualifying_relative_aged_20_gets_the_credit_for_other_dependents,
//   a_release_never_makes_a_child_who_supports_themselves_a_dependent,
//   qualifying_survivor_cannot_rest_on_a_child_the_filer_released,
//   married_custodial_parent_who_released_the_child_is_considered_unmarried,
//   education_credit_follows_the_parent_who_claims_the_child (family.rs);
//   custody_release_and_multiple_support_rules_reject_on_save (interview.rs). Sources: f8332 and
//   f2120 Rev. December 2025, Pub. 501 (2025), Pub. 596, i2441--dft, i1040s8--dft, IRC 152(d)(3),
//   152(e), 2(b), 7703(b)(1), 21(e)(5), 32(c)(3)(A), Treas. Reg. 1.152-4.
// Coverage: Form 8814 — see child_election.rs (2026-10-09).
// Coverage: Form 8815 — see savings_bonds.rs (unit B1, 2026-10-09). FinalBonds is the decedent's
//   EE/I interest election on a final return, not 8815.
// Coverage: Form 8958 — see community.rs (unit B9-3, 2026-10-10).
// Coverage: Income in respect of a decedent (Pub. 559) — see after_death.rs (unit B9-3).
// Coverage: Form 8824 — see exchange.rs (unit B2, 2026-10-09).
// Coverage: Schedule K-3 (1065 / 1120S) — see schedule_k3.rs (unit k3, 2026-10-09).
// Coverage: Form 8857 — Missing (audited 2026-09-24).
//   Lines: No innocent-spouse model or calculation; no string match.
// Coverage: Form 8586 and Form 8582-CR — see general_business_credit.rs (unit form-8586,
//   2026-10-09).
// Coverage: Forms 8941 and 8881 — see general_business_credit.rs (T3-3800, 2026-09-27; Form 8941
//   lines 1-16, unit form-8941, 2026-10-09).
// Coverage: Form 8994 — see general_business_credit.rs (unit form-8994, 2026-10-09).
// Coverage: Form 8910 — Not applicable (audited 2026-10-09).
//   Lines: Section 30B(k) ended the credit for vehicles purchased after 2021 (P.L. 117-169
//   section 13401(i)(2) did not extend it); the latest Form 8910 is Rev. 1-2022, and the 2026
//   Form 3800 Part III line 1r is "Reserved" and Schedule 3 line 6e "Reserved for future use".
//   Business-part carryforwards: Form 3800 Part IV line 2x (Supported, general_business_credit.rs).
// Coverage: Form 8903 — N/A (audited 2026-09-24).
//   Lines: Repealed (section 199); no source match, correctly absent.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod kernel;

#[path = "federal/line_emission.rs"]
mod line_emission;

#[path = "federal/issue_macros.rs"]
mod issue_macros;

#[path = "core/shared/form_lines.rs"]
mod form_lines;
#[path = "core/shared/navigation.rs"]
mod navigation;

#[macro_use]
#[path = "core/source_macros.rs"]
mod source_macros;
pub(crate) use source_macros::fixture_english;

#[path = "core/shared/diagnostics.rs"]
mod diagnostics;
#[path = "federal/income_names.rs"]
mod income_names;
#[path = "federal/issue_helpers.rs"]
mod issue_helpers;
#[path = "core/shared/person_names.rs"]
mod person_names;
#[path = "core/shared/printed_state.rs"]
mod printed_state;
#[path = "core/shared/return_carriers.rs"]
mod return_carriers;
#[path = "core/shared/return_state.rs"]
mod return_state;
#[path = "core/shared/us_address.rs"]
pub(crate) mod us_address;
pub use return_carriers::{
    FarmTaxInstallments, InjuredAmount, InjuredAmountPage, InjuredPrintedRow, Withheld,
};
pub use return_state::{FederalAmount, FederalEstimate, FederalResult, FederalReviewNoteView};
#[path = "core/shared/section_labels.rs"]
mod section_labels;

#[path = "family/after_death.rs"]
mod after_death;
#[path = "cost_recovery/historical_recovery.rs"]
mod historical_recovery;
#[path = "cost_recovery/rental_recovery.rs"]
mod rental_recovery;
pub use historical_recovery::{OfficeRecoveryDeduction, OfficeRecoveryIssue};
pub use rental_recovery::{
    RentalRecoveryAllowed, RentalRecoveryAllowedRow, RentalRecoveryDate, RentalRecoveryInputs,
    RentalRecoveryMoreAllowed, RentalRecoveryMoreDates,
};
#[path = "cost_recovery/rental_energy.rs"]
mod rental_energy;
pub use rental_energy::{
    BuildingRecoveryRecord, RentalEnergyBuildingFacts, RentalEnergyBuildingFactsPage,
    RentalEnergyCertification, RentalEnergyClaim, RentalEnergyClaimPage,
    RentalEnergyDeductionSource, RentalEnergyIncreasedAmount, RentalEnergyProject,
    RentalEnergySystem,
};
#[path = "attribution/community.rs"]
mod community;
#[path = "core/shared/community_share.rs"]
mod community_share;
#[path = "cost_recovery/mileage_records.rs"]
mod mileage_records;
pub use after_death::{DividendsAfterDeath, InterestAfterDeath};
#[path = "passthrough/bba_audit.rs"]
mod bba_audit;
#[path = "foreign/foreign_assets.rs"]
mod foreign_assets;
#[path = "foreign/foreign_business.rs"]
mod foreign_business;
#[path = "foreign/foreign_housing.rs"]
mod foreign_housing;
#[path = "core/unknowns.rs"]
mod unknowns;
pub use foreign_housing::{
    ExclusionRevocation, ForeignEarnedClaim, ForeignHousing, ForeignHousingHousehold,
    ForeignHousingLocation, ForeignHousingStay, ForeignPay, Form2555, HousingCarryover,
    MultiYearPay, PriorYearHistory, RevokedElections,
};
#[path = "business/household_employment.rs"]
mod household_employment;
pub use household_employment::{
    HouseholdDisability, HouseholdEmployer, HouseholdEmployerView, HouseholdPay, HouseholdQuarter,
    HouseholdSickContribution, HouseholdSickEnd, HouseholdSickEntitlement, HouseholdSickFundPeriod,
    HouseholdSickFunding, HouseholdSickPayment, HouseholdSickPolicy, HouseholdSickPremium,
    HouseholdSickWages, HouseholdState, HouseholdStatePayment, HouseholdStatePeriod,
    HouseholdWageKind, HouseholdWorker,
};
#[path = "income/employee_wages.rs"]
mod employee_wages;
#[path = "income/employee_wages_admission.rs"]
mod employee_wages_admission;
mod forms;
#[path = "income/statutory_wages.rs"]
mod statutory_wages;
pub use employee_wages::{
    EmployeeCoverage, EmployeePayrollSource, EmployeeRailroadSource, EmployeeWageReason,
    EmployeeWages,
};
pub use forms::form_8862::{
    CreditDisallowance, CreditsAfterDisallowance, DisallowanceFacts, DisallowanceFinding,
    EicDisallowance, EicDisallowanceCause,
};
#[path = "income/employee_tips.rs"]
mod employee_tips;
pub use employee_tips::{
    CashTipMonth, CashTipRecords, CashTipsWithoutW2, EmployeeTips, TipCoverage, W2UncollectedTax,
};
#[path = "family/form_1099_qa_model.rs"]
mod form_1099_qa_model;
pub use form_1099_qa_model::Form1099QaPaper;
#[path = "family/able.rs"]
mod able;
#[path = "family/able_distribution_source.rs"]
mod able_distribution_source;
pub use able::{
    AbleContributions, AbleContributionsPage, AbleDistribution, AbleRollover, AbleSource,
    AbleTreatment, AbleWork, Qa1099,
};
#[path = "family/trump_account.rs"]
mod trump_account;
pub use trump_account::{
    TrumpAbleRollover, TrumpAccount, TrumpAccountView, TrumpDeathAcquisition, TrumpExcessReturn,
};
#[path = "family/adoption.rs"]
mod adoption;
#[path = "family/alimony.rs"]
mod alimony;
#[path = "business/moving.rs"]
mod moving;
pub use adoption::{
    AdoptionBenefitHistory, AdoptionBenefits, AdoptionBenefitsPage, AdoptionCarry, AdoptionChild,
    AdoptionChildInitial, AdoptionChildPage, AdoptionEmployerPayment, AdoptionEmployerPick,
    AdoptionExpense, AdoptionFinalization, AdoptionKind, AdoptionLimits, AdoptionPriorCredit,
    JointAdoptionExclusion,
};
pub use alimony::{AlimonyNewRules, AlimonyPaid, AlimonyPayment, AlimonyRecapture};
pub use moving::{MilitaryMove, MoveReason, MoveTrip, MovingClaim};
#[path = "cost_recovery/amortization.rs"]
mod amortization;
pub use amortization::{
    BusinessIntangible, BusinessIntangiblePage, IntangibleAcquisitionContext,
    IntangibleConsideration, IntangibleCost, IntangibleEventKind, IntangibleKind,
    IntangibleLeaseStatus, IntangibleObtained, IntangibleObtainedPage, IntangibleOriginEvidence,
    IntangiblePriorClaimed, IntangiblePriorClaimedPage, IntangibleSource, IntangibleSourcePage,
    IntangibleUseEnd,
};
#[path = "cost_recovery/cohorts.rs"]
mod cohorts;
#[path = "cost_recovery/timber.rs"]
mod timber;
pub use cohorts::{
    CreativePropertyCost, DomesticResearchAcceleration, LeaseAcquisitionCost, OtherAmortizableCost,
    OtherAmortizableCostPage, OtherAmortizationKind, OtherAmortizationKindPage,
    PollutionControlCost, PollutionStart, PreferenceWriteoff, ResearchCapitalization, ResearchCost,
    ResearchCostPage, ResearchLocation, ResearchTreatment,
};
pub use kernel::source_fact::{SourceCents, SourceDate, SourceUnits};
pub use timber::{AmortizationEnd, BusinessTimberCost, ReforestationCost};
#[path = "family/education_account.rs"]
mod education_account;
#[path = "family/education_account_source.rs"]
mod education_account_source;
#[path = "family/form_1099_q_model.rs"]
mod form_1099_q_model;
pub use education_account::{
    CollegeCosts, CoverdellAmounts, DistributionReason, EducationAccount, EducationAccountExpenses,
    EducationAccountStudent, EducationDistributionSource, FederalEducationAccountExpenses,
    FederalEducationAccountStudent, FederalEducationBeneficiary, FederalEducationDistribution,
    Q1099, Q1099Beneficiary, Q1099Transfer,
};
pub use form_1099_q_model::{
    Form1099QDistributionCode, Form1099QDistributionFrom, Form1099QFairMarketValue, Form1099QPaper,
};
#[path = "farm/farm_production.rs"]
mod farm_production;
#[path = "farm/farm_specials.rs"]
mod farm_specials;
#[path = "foreign/foreign_accounts.rs"]
mod foreign_accounts;
#[path = "cost_recovery/method_change.rs"]
mod method_change;
#[path = "investment/savings_bonds.rs"]
mod savings_bonds;
#[path = "cost_recovery/startup.rs"]
mod startup;
#[path = "core/year_constants.rs"]
mod year_constants;
pub use extended_model::InventoryMethod;
pub use farm_specials::{
    CccElectionHistory, CccSourceFacts, CccSourceFactsInput, ConservationAssessmentInput,
    ConservationCostInput, Farm126Election, Farm126Exclusion, Farm126History, FarmAffectedIncome,
    FarmAnimalYear, FarmAssetLink, FarmAssetLinkInput, FarmAveragingHistoryScope,
    FarmAveragingSelection, FarmAveragingSelectionInput, FarmBasisDestination,
    FarmBasisDestinationInput, FarmCapitalization, FarmChoice, FarmCodeChoice,
    FarmConservationCarryInput, FarmCostSharing, FarmCostSharingInput, FarmCropPractice,
    FarmElectionYear, FarmExternalSource, FarmExternalSourceInput, FarmFertilizerCost,
    FarmFertilizerLand, FarmFertilizerRecovery, FarmFertilizerTreatment, FarmGrossYear, FarmHeader,
    FarmHeaderInput, FarmHedge, FarmHedgeTiming, FarmIncomeRightValue, FarmInformationReturns,
    FarmLandBasis, FarmLivestockClass, FarmMethodChangeLink, FarmMethodChangeLinkInput,
    FarmMethodStatus, FarmMethodStatusInput, FarmOriginalMethod, FarmOtherIncome,
    FarmPositiveReceipt, FarmPriorWeather, FarmProducedKind, FarmProductionCohort,
    FarmProductionCohortInput, FarmProductionCost, FarmProductionCostKind,
    FarmProductionPoultryCurrent, FarmShareAgreement, FarmSignedReceipt, FarmSmallBusiness,
    FarmSourceCharacter, FarmSpecials, FarmSpecialsInput, FarmSupplyPaymentKind,
    FarmSupplyPurchase, FarmWeatherAssistance, FarmWeatherSale, FarmWeatherSource,
};
pub use method_change::{Adjustment481, MethodChange, MethodChangeAmt, MethodChangeConsent};
pub use startup::BusinessStartup;
pub use year_constants::{HEALTH_TAX_YEAR, PTC_TAX_YEAR, TAX_YEAR_END_DATE, TAX_YEAR_START_DATE};
#[path = "investment/capital.rs"]
mod capital;
#[path = "investment/qsbs.rs"]
mod qsbs;
#[path = "investment/qsbs_return.rs"]
mod qsbs_return;
pub use qsbs::{QsbIssuerSale, QsbStockLot, QsbsPaperLot, QsbsSale};
#[path = "investment/qsbs_followups.rs"]
mod qsbs_followups;
pub use qsbs_followups::{
    QsbsBooleanAnswer, QsbsGainAnswer, QsbsGainValue, QsbsValuationAnswer, QsbsZoneAnswer,
    QsbsZoneCertification, QsbsZoneValuation,
};
#[path = "investment/asset_sale.rs"]
mod asset_sale;
#[path = "investment/business_sale.rs"]
mod business_sale;
pub use asset_sale::{
    AssetDisposal, Buyer, DisposalKind, Livestock, LivestockPurpose, PartialDisposition,
    RentalSale, SoldLivestock,
};
pub use business_sale::BusinessSale;
#[path = "investment/exchange.rs"]
mod exchange;
pub use exchange::{
    EarlierRelatedExchange, ExchangeParty, ExchangeReplacement, RelatedDisposal, RelatedException,
    RelatedExchange, RentalExchange, ReplacementUse,
};
#[path = "investment/installment.rs"]
mod installment;
#[path = "investment/section1256.rs"]
mod section1256;
pub use installment::{
    EarlierInstallment, InstallmentMethod, InstallmentOther, InstallmentPayments, InstallmentQbi,
    InstallmentSale, InstallmentUse, InvestmentInstallmentSale, NoteInterest, PropertyCharacter,
    PropertyUse, ResaleException, SalePayment, SecondDisposition, SoldProperty, SoldQbi, SoldUse,
};
pub use section1256::Section1256Carryback;
#[path = "home/home_office.rs"]
mod home_office;
#[path = "home/sites.rs"]
mod home_sites;
pub use home_sites::{
    DaycareLicense, HomeCost, HomeCostKind, HomeCostOwners, HomeCostPage, HomeCostScope,
    HomeCostScopePage, HomeOfficeClaim, HomeOfficeClaimPage, HomeOfficeSite, HomeOfficeSitePage,
    HomeOffices, HomeTenure, OfficeActivity, OfficeActivityPage, OfficeBuilding, OfficeMethod,
    OfficeOldConvention, OfficeOldMethod, OfficeProduction, OfficeProductionAcquisition,
    OfficeProductionChange, OfficeProductionOperation, OfficeQualifiedUse, OfficeRecovery,
    OfficeRecoveryHistory, OfficeSimplifiedMonth,
};
#[path = "cost_recovery/auto_recovery.rs"]
mod auto_recovery;
#[path = "cost_recovery/depreciation_specials.rs"]
mod depreciation_specials;
#[path = "cost_recovery/interest_limit.rs"]
mod interest_limit;
pub use interest_limit::{
    InterestLimitElection, InterestLimitElectionKind, InterestLimitElectionRecord,
};
#[path = "cost_recovery/macrs_source.rs"]
mod macrs_source;
pub use macrs_source::{
    AdsClassLife, AdsElection, AdsLease, AdsLessee, AdsRestriction, AdsRestrictionPeriod,
    BuildingFirstService, BuildingUseSource, ClassLifeRecord, ForeignUseYear,
    ImprovementBuildingFacts, MacrsRecoverySource, MacrsRecoverySourcePage,
    MacrsSourceClassElection, MacrsSourceQuestionContext, MacrsSourceQuestionFacts,
    OriginalMacrsSystem, RealImprovement, RealImprovementKind, RealImprovementPage,
    RecoveryBuildingKind,
};
#[path = "cost_recovery/nonmacrs.rs"]
mod nonmacrs;
pub use depreciation_specials::{
    AssetAmt179CarryAnswer, AssetConvention, AssetYearHistory, AutoRecoveryHistory,
    CanonicalAssetPage, HistoricalAssetRecords, HistoricalAutoKind, MacrsAssetPage,
    OriginalAssetMethod, OriginalBonusTreatment, UseFraction,
};
pub use nonmacrs::{
    AcrsOriginalClass, AcrsPeriod, AssetRecoveryPage, FiniteLifeKind, FiniteLifeRecovery,
    ForecastLookback, ForecastParticipations, ForecastPropertyKind, GdsAssetRecovery,
    GdsEquipmentClass, GdsPropertyPage, HistoricalDepreciation, HistoricalPropertyKind,
    HistoricalRecoveryKind, IncomeForecastRecovery, NonMacrsAssetPage, NonMacrsBasicHistory,
    NonMacrsHistory, NonMacrsPropertyRecovery, NonMacrsRecovery, SoftwareKind, SoftwareRecovery,
    SoftwareTaxExemptLease, UnitProduction,
};
#[path = "cost_recovery/cooperative_basis.rs"]
mod cooperative_basis;
#[path = "cost_recovery/depreciation.rs"]
mod depreciation;
#[path = "credits/investment_credit.rs"]
mod investment_credit;
#[path = "investment/rental_home_carry.rs"]
mod rental_home_carry;
#[path = "investment/rental_property_history.rs"]
mod rental_property_history;
#[path = "investment/section1231.rs"]
mod section1231;
pub use rental_property_history::{RentalBuildingSaleHistory, RentalPropertyLossYear};
#[path = "cost_recovery/section179/mod.rs"]
mod section179;
#[path = "cost_recovery/section179_history.rs"]
mod section179_history;
#[path = "cost_recovery/section179_return.rs"]
mod section179_return;
#[path = "attribution/source_associations.rs"]
mod source_associations;
pub use depreciation::{
    BonusElection, BusinessAsset, BusinessLeasedVehicle, EquipmentAmtHistory, EquipmentClass,
    EquipmentHistory, EquipmentMethod, EquipmentSchedule, ListedProperty, Section179, SuvException,
    VehicleEvidence, VehicleKind,
};
pub use investment_credit::{
    CreditYear, EnergyProperty, EnergyRate, EnergySection, InvestmentCredit, K1InvestmentProperty,
    OwnInvestmentCredit, RehabilitationCredit,
};
pub use section179::{
    K1Section179AvailabilitySource, Ptp179Component, Ptp179ComponentAllocation, Ptp179PalInput,
    Ptp179SourceIssue, Section179BusinessIncomeSource, Section179DirectBusinessIncomeSource,
    Section179DirectIncomeSource, Section179IncomeAllocation, Section179IncomeClass,
    Section179IncomeK1Allocation, Section179IncomeSourceKind, Section179IncomeSystem,
    Section179OtherLimitedDeduction, Section179PreparedAllocation, Section179PreparedIssue,
    Section179PtpReportContext, Section179Share, Section179Spouse, SpouseSection179,
    k1_section179_availability_source, section179_direct_income_sources,
};
pub use section179_history::{Asset179Carryover, Section179IncomeCarryHistory};
pub use section1231::{BusinessPropertyLosses, Section1231History, Section1231LossYear};
#[path = "federal/atnol.rs"]
mod atnol;
#[path = "deductions/retained_charity.rs"]
mod retained_charity;
pub use retained_charity::{
    CharityAppreciationChoices, CharityAppreciationFacts, CharityCarryoverPage,
    CharitySourceAppreciation, OtherCharityEntity, ResearchCharityOrigin,
    ResearchCharityOriginPage, ResearchCharityPart, ResearchCharityPartPage, ResearchCharityParts,
    ResearchCharityPartsPage,
};
#[path = "credits/retained_research.rs"]
mod retained_research;
pub use retained_research::{
    NolYearPage, PriorCapitalReturnPage, ResearchCapitalPart, ResearchCapitalPartPage,
    ResearchCapitalParts, ResearchCapitalPartsPage, ResearchLossPart, ResearchLossPartPage,
    ResearchLossParts, ResearchLossPartsPage,
};
#[path = "federal/nol.rs"]
mod nol;
pub use nol::{LossCarryforward, LossSource, NolHistory, NolYear, PtpEblNol};
#[path = "activity/business_rules/mod.rs"]
mod business_rules;
pub use atnol::{AtnolHistory, AtnolYear};
pub use business_rules::{QbiHistory, ReitDividends};
#[path = "health/form_1099_sa_model.rs"]
mod form_1099_sa_model;
pub use form_1099_sa_model::{Form1099SaDistributionCode, Form1099SaPaper};
#[path = "health/medical_account.rs"]
mod medical_account;
#[path = "health/medical_account_return.rs"]
mod medical_account_return;
pub use medical_account::{
    FederalMedicalDistribution, HealthDocumentOrigin, HsaTrusteeStatementPaper,
    InheritedMedicalPayment, MedicalAccountWithdrawal, MedicalBeneficiary, MedicalSaAccount,
    MedicalSaPaper, MedicalSaPayment, MedicareMsaPrior, Sa1099, SaSource,
};
#[path = "health/medical_rollover.rs"]
mod medical_rollover;
#[path = "health/medical_rollover_return.rs"]
mod medical_rollover_return;
pub use medical_rollover::{
    HealthFailedDepositChoice, HealthFailedDepositFact, MedicalDepositCharacter, MedicalRollover,
    MedicalRolloverDeposit, MedicalRolloverDestination, MedicalRolloverReceipt,
};
#[path = "health/archer_contributions.rs"]
mod archer_contributions;
pub use archer_contributions::{
    ArcherContributions, ArcherContributionsPage, ArcherContributor, ArcherContributorRow,
    ArcherCorrection, ArcherCorrectionPage, ArcherCorrectionPayer, ArcherCoverage,
    ArcherEligibility, ArcherExcessAccounting, ArcherPriorEmployerExcessDeposit, ArcherPriorExcess,
    ArcherSharing,
};
#[path = "health/medical_deemed.rs"]
mod medical_deemed;
pub use hsa::{
    HsaAllocationMethod, HsaEligibilityLoss, HsaExcessAccounting, HsaFamilyShare,
    HsaFundingHistory, HsaLateExcessWithdrawal, HsaLossReason, HsaPriorEmployerExcessDeposit,
    HsaPriorExcess,
};
pub use medical_deemed::MedicalDeemed;
#[path = "activity/activity_risk.rs"]
mod activity_risk;
#[path = "activity/activity_risk_methods.rs"]
mod activity_risk_methods;
#[path = "activity/activity_risk_model.rs"]
mod activity_risk_model;
#[path = "activity/business_risk.rs"]
mod business_risk;
#[path = "health/care_benefit_return.rs"]
mod care_benefit_return;
#[path = "health/care_benefits.rs"]
mod care_benefits;
#[path = "income/clergy.rs"]
mod clergy;
#[path = "core/dates.rs"]
mod dates;
#[path = "federal/extended_model.rs"]
mod extended_model;
pub use activity_risk_model::{
    ActivityRiskRecord, PriorRiskBalance, RiskAmtMethod, RiskDepletionYear, RiskDetailed,
    RiskDetailedOpening, RiskEarlierLoss, RiskEffectiveCategory, RiskFinancing,
    RiskFinancingMovement, RiskFinancingTerms, RiskHistoricalAccounting, RiskHistoricalInvestment,
    RiskMethod, RiskQnfCollateral, RiskQnfFacts, RiskQnfLender, RiskRecaptureAmounts,
    RiskRecaptureHistory, RiskSimplified,
};
#[path = "federal/extended_rules.rs"]
mod extended_rules;
#[path = "business/optional_se.rs"]
mod optional_se;
pub use clergy::{ClergyClaim, ClergyRecord, ClergyRole, MinisterHousing};
pub use federal_model::{
    NextYearEstimateRecord, OverpaymentRecipient, PaymentsEstimateInput, W2Coverage,
};
#[path = "income/royalties.rs"]
mod royalties;
pub use optional_se::{NonfarmSeHistory, SeElection};
#[path = "farm/farm_crp.rs"]
mod farm_crp;
pub use farm_crp::CrpPayment;
#[path = "farm/farm_ccc.rs"]
mod farm_ccc;
pub use farm_ccc::{CccDisposition, CccLot, CccMethod, CccOrigin, FarmCcc};
#[path = "farm/farm_insurance.rs"]
mod farm_insurance;
#[path = "farm/farm_insurance_paper.rs"]
mod farm_insurance_paper;
pub use farm_insurance::{CropDeferral, CropPayment, FarmInsurance};
pub use farm_insurance_paper::{
    AcceptedCropClassification, AcceptedCropTiming, CropInsuranceActivityLink, CropInsuranceFacts,
    CropPaperClass, CropPaperFacts, CropPaperLink, CropPaperTiming, CropPracticeLink,
    FarmInsuranceRecords,
};
#[path = "farm/farm_averaging.rs"]
mod farm_averaging;
pub use farm_averaging::{
    AveragingBaseYear, AveragingCapital, AveragingForeign, AveragingForeignForm,
    AveragingInvestment, AveragingLoss, AveragingNol, AveragingOtherTax, AveragingSource,
    FarmAveraging,
};
#[path = "farm/farm_conservation.rs"]
mod farm_conservation;
#[path = "farm/farm_conservation_model.rs"]
mod farm_conservation_model;
pub use farm_conservation_model::{
    ConservationAssessment, ConservationAssessmentPayment, ConservationCost, FarmConservation,
    FarmConservationCapitalMethod, FarmConservationCarry, FarmConservationFacts,
    FarmConservationMethod, FarmConservationPlan, FarmConservationWork,
};
#[path = "farm/farm_deferral.rs"]
mod farm_deferral;
#[path = "farm/farm_land_return.rs"]
mod farm_land;
#[path = "farm/farm_land.rs"]
mod farm_land_model;
pub use farm_land_model::{
    FarmLandAcquisition, FarmLandAllocation, FarmLandAmt, FarmLandSale, FarmLandSituation,
};
#[path = "farm/farm_supplies.rs"]
mod farm_supplies;
pub use farm_supplies::{
    FarmPoultryMethod, FarmPoultryResale, FarmProductionPoultryCarry, FarmSupplies,
    FarmSupplyAdvance, FarmSupplyCarry, FarmSupplyException, FarmSupplyRelation, FarmSupplyYear,
};
#[path = "farm/cooperative.rs"]
mod cooperative;
#[path = "farm/cooperative_source.rs"]
mod cooperative_source;
#[path = "business/expense_source.rs"]
mod expense_source;
pub use cooperative_source::{
    CoopAllocation, CoopAllocationInput, CoopCreditFacts, CoopCreditFactsInput, CoopCreditIncome,
    CoopDistributionAmounts, CoopDistributionBox, CoopEarlierCharacter, CoopIncomeUse,
    CoopInvestmentKind, CoopInvestmentShare, CoopNoticeHolding, CoopNoticeKind, CoopNoticeStatus,
    CoopOtherItem, CoopOtherItemInput, CoopOtherKind, CoopPatronAllocation, CoopPropertyRefund,
    CoopPropertyRefundInput, CoopPropertyTreatment, CoopQualification, CoopRedemption,
    CoopSstbExpense,
};
#[path = "farm/farm.rs"]
mod farm;
pub use cooperative_source::FederalCooperative;
pub use farm::{FarmAccounting, FarmIncome, FarmNolElection, FarmReceipt};
#[path = "farm/farm_page.rs"]
mod farm_page;
#[path = "farm/farm_rental.rs"]
mod farm_rental;
pub use farm_page::{
    BaseYear, BaseYearSource, CarriedCost, CccLoan, CccLoans, CccOutcome, Conservation,
    CropInsurance, CropLoss, CrpItem, Farm, FarmAveragingElection, FarmDetails, FarmExpenses,
    FarmIncomeSource, FarmLossChoice, FarmLossWaiver, FarmMethod, FarmRental, FarmRentalDetails,
    FarmWorkValue, LandInstallments, LandSale, PoultryMethod, PoultryRecords, PoultryResale,
    PrepaidSupplies, ProductionPoultry, SupplyException, SupplyPurchase, SupplyYear,
};
pub use farm_rental::{
    FarmRentalPropertyUse, FarmRentalTenantConnection, FarmRentalValueEnhancement,
};
#[path = "credits/business_credit_page.rs"]
mod business_credit_page;
#[path = "credits/fuel_credit.rs"]
mod fuel_credit;
#[path = "credits/general_business_credit.rs"]
mod general_business_credit;
pub use business_credit_page::{
    BusinessCreditCarrybackInitial, BusinessCreditCarrybackPage, BusinessCreditCarryforwardInitial,
    BusinessCreditCarryforwardPage, BusinessCreditHistoryInitial, BusinessCreditHistoryPage,
    CreditSourcePage, EnteredBusinessCreditInitial, EnteredBusinessCreditPage, K1CreditCodePage,
    K1CreditPage, ResearchCreditBusiness, WageScorpEmployer, WageScorpEmployerView,
};
pub use fuel_credit::{
    FederalAviationNontaxableUse, FederalAviationVendorRegistration, FederalBlockedPumpAccess,
    FederalBlockedPumpKeroseneClaim, FederalBlockedPumpKeroseneSale, FederalCardIssuerClaim,
    FederalCardIssuerFuel, FederalCardIssuerSale, FederalCardVendorSettlement,
    FederalGovernmentFuelSale, FederalVendorAviationClaim, FederalVendorAviationSale,
    FederalVendorAviationTaxRate, FederalVendorBusClaim, FederalVendorBusSale,
    FederalVendorNoncommercialAviationClaim, FederalVendorNoncommercialAviationSale,
    FederalVendorNontaxableAviationClaim, FederalVendorNontaxableAviationSale,
    FederalVendorStateClaim, FederalVendorTaxRelief,
};
pub use general_business_credit::{
    BusinessCredit, BusinessCreditCarryback, BusinessCreditCarryforward, BusinessCreditHistory,
    CarbonOxideCreditOrigin, CarbonOxideEquipmentCredit, EmployeeHealthCoverage,
    EmployerTipEmployee, EmployerTipEmployeePage, EmployerTipFicaCarryforward,
    EmployerTipFicaCredit, EmployerTipFicaCreditPage, EmployerTipMonth, EmployerTipService,
    HealthCoverageTier, HealthCreditCoverage, HealthCreditEmployee, HealthCreditEmployeePage,
    HealthCreditService, K1Credit, K1CreditCode, PaidLeaveCredit, PaidLeaveCreditPage,
    PaidLeaveEmployee, PaidLeaveEmployeePage, PaidLeaveInsurance, PassiveWorksheet,
    PlanStartupCosts, RenewableFacilityCredit, RenewableFacilityCreditOrigin, RetirementPlanCredit,
    SmallEmployerHealthCredit, SmallEmployerHealthCreditPage, WorkOpportunityCredit,
    WorkOpportunityCreditPage, WorkOpportunityEmployee, WorkOpportunityEmployeePage,
    WorkOpportunityGroup,
};
pub use general_business_credit::{
    CarryFiler, CreditElection, CreditSource, EnteredBusinessCredit,
};
#[path = "federal/accepted_documents.rs"]
mod accepted_documents;
#[path = "federal/document_name_matcher.rs"]
mod document_name_matcher;
pub use accepted_documents::{BbaAuditSource, VehicleLoanSource};
#[path = "federal/document_names.rs"]
mod document_names;
#[path = "core/shared/document_parties.rs"]
mod document_parties;
pub use document_names::{
    AddedOwnership, DocumentEditTarget, DocumentOwner, DocumentOwnerSelection,
    DocumentOwnershipDerived, DocumentRecipient, OwnerSource, PersonDocuments,
    PrintedDocumentNames, SectionDocument,
};
#[path = "federal/accepted_draft.rs"]
mod accepted_draft;
#[path = "family/accepted_filers.rs"]
mod accepted_filers;
#[path = "federal/accepted_income.rs"]
mod accepted_income;
#[path = "cost_recovery/amortization/records.rs"]
mod amortization_records;
#[path = "cost_recovery/asset_model.rs"]
mod asset_model;
#[path = "cost_recovery/building_model.rs"]
mod building_model;
#[path = "investment/capital_sale_model.rs"]
mod capital_sale_model;
#[path = "deductions/claim_repayment_model.rs"]
mod claim_repayment_model;
#[path = "investment/disposition_model.rs"]
mod disposition_model;
#[path = "credits/education_recapture.rs"]
mod education_recapture;
#[path = "family/elderly_source.rs"]
mod elderly_source;
#[path = "investment/exchange_model.rs"]
mod exchange_model;
#[path = "farm/farm_rental_source.rs"]
mod farm_rental_source;
#[path = "federal/federal_model.rs"]
mod federal_model;
#[path = "federal/federal_rules.rs"]
mod federal_rules;
#[path = "family/filer_facts.rs"]
mod filer_facts;
#[path = "family/filer_source.rs"]
mod filer_source;
#[path = "federal/income_adapters.rs"]
mod income_adapters;
#[path = "federal/income_editor.rs"]
mod income_editor;
#[path = "attribution/injured_spouse_model.rs"]
mod injured_spouse_model;
#[path = "investment/installment_payment.rs"]
mod installment_payment;
#[path = "investment/installment_return.rs"]
mod installment_return;
#[path = "credits/investment_credit_model.rs"]
mod investment_credit_model;
#[path = "cost_recovery/macrs_records.rs"]
mod macrs_records;
#[path = "federal/minimum_tax_model.rs"]
mod minimum_tax_model;
#[path = "income/misc_source.rs"]
mod misc_source;
#[path = "family/parent_tax_source.rs"]
mod parent_tax_source;
#[path = "cost_recovery/rental_energy_model.rs"]
mod rental_energy_model;
#[path = "cost_recovery/rental_energy_records.rs"]
mod rental_energy_records;
#[path = "activity/rentals/property_type.rs"]
mod rental_property_type;
#[path = "federal/return_reporting.rs"]
mod return_reporting;
pub use education_recapture::PriorYearEducationCreditRecapture;
#[path = "health/contribution_documents.rs"]
mod contribution_documents;
#[path = "health/form_5498_sa_model.rs"]
mod form_5498_sa_model;
#[path = "health/health.rs"]
mod health;
pub use contribution_documents::{
    Form5498Sa, HealthContributionAllocation, HealthContributionAllocationPage,
    HealthContributionDocuments, HealthContributionKind, HealthContributionSourceRow,
    HealthStatementDeposit, HealthStatementDepositPage, HealthStatementFunding,
    HealthStatementFundingPage, HealthStatementW2Part, HealthStatementW2PartPage,
};
pub use form_5498_sa_model::{Form5498SaPaper, HealthStatementSource};
#[path = "health/health_employer.rs"]
mod health_employer;
pub use health_employer::{
    HealthEmployerBinding, HealthEmployerKind, HealthEmployerSourceRow, HealthEmployerYear,
};
#[path = "health/health_marketplace.rs"]
mod health_marketplace;
#[path = "health/health_ptc_iteration.rs"]
mod health_ptc_iteration;
pub use health_marketplace::{
    HealthMarketplace, HealthPolicyAllocation, HealthPolicyAllocationPage, HealthPolicyBenchmark,
    HealthPolicyBenchmarkPage, HealthPtcMethod, ReimbursedPolicyPremiums,
};
#[path = "health/care_treatment.rs"]
mod care_treatment;
#[path = "federal/income_tax/mod.rs"]
mod income_tax;
#[path = "attribution/injured_ltc_source.rs"]
mod injured_ltc_source;
#[path = "attribution/injured_spouse/mod.rs"]
mod injured_spouse;
pub use injured_ltc_source::{
    InjuredLtcPeriodPayees, InjuredLtcPeriodPayeesInput, InjuredLtcPeriodPayment,
    InjuredLtcPeriodPaymentInput,
};
#[path = "attribution/injured_spouse_sources.rs"]
mod injured_spouse_sources;
pub use injured_spouse_sources::{
    InjuredCapitalLossOwners, InjuredNolOwners, InjuredPriorCareClaimants,
    InjuredPriorCareIdentity, InjuredPriorCarePerson, InjuredQbiLossOwners, InjuredRetainedOwners,
};
pub use injured_spouse_sources::{
    InjuredPriorCareClaimantsInput, InjuredPriorCareIdentityInput, InjuredPriorCarePersonInput,
};
#[path = "attribution/injured_spouse_payments.rs"]
mod injured_spouse_payments;
pub use injured_spouse_payments::{
    AnnualEstimatedAmount, AnnualEstimatedAmountRecord, EstimatedPaymentEntry,
    EstimatedPaymentEntryRecord, InjuredDatedPayment, PaymentPayer, RefundManagement,
    RefundSourceProperty, RefundSourcePropertyFacts,
};
#[path = "family/able_contribution_source.rs"]
mod able_contribution_source;
#[path = "deductions/accepted_adjustments.rs"]
mod accepted_adjustments;
#[path = "credits/accepted_credits.rs"]
mod accepted_credits;
#[path = "deductions/accepted_deductions.rs"]
mod accepted_deductions;
#[path = "family/accepted_dependents.rs"]
mod accepted_dependents;
#[path = "federal/accepted_screening.rs"]
mod accepted_screening;
#[path = "family/adoption_source.rs"]
mod adoption_source;
#[path = "family/alimony_model.rs"]
mod alimony_model;
#[path = "health/archer_source.rs"]
mod archer_source;
#[path = "federal/atnol_model.rs"]
mod atnol_model;
#[path = "passthrough/bba_audit_source.rs"]
mod bba_audit_source;
#[path = "investment/bond_premium_source.rs"]
mod bond_premium_source;
#[path = "credits/business_credit_source.rs"]
mod business_credit_source;
#[path = "federal/capital_history_model.rs"]
mod capital_history_model;
#[path = "family/care_source.rs"]
mod care_source;
#[path = "deductions/charity_sources.rs"]
mod charity_sources;
#[path = "core/checklist_model.rs"]
mod checklist_model;
#[path = "core/income_without_form_model.rs"]
mod income_without_form_model;
pub use income_without_form_model::{IncomeWithoutFormAnswers, WithoutFormOther};
#[path = "credits/clean_vehicle_source.rs"]
mod clean_vehicle_source;
#[path = "income/clergy_source.rs"]
mod clergy_source;
#[path = "forms/disallowance_source.rs"]
mod disallowance_source;
#[path = "retirement/disaster_carry_model.rs"]
mod disaster_carry_model;
#[path = "credits/education_recapture_source.rs"]
mod education_recapture_source;
#[path = "family/education_source.rs"]
mod education_source;
#[path = "deductions/educator_source.rs"]
mod educator_source;
#[path = "business/employee_expense_model.rs"]
mod employee_expense_model;
#[path = "income/espp_source.rs"]
mod espp_source;
#[path = "farm/farm_averaging_source.rs"]
mod farm_averaging_source;
#[path = "farm/farm_nol_source.rs"]
mod farm_nol_source;
#[path = "foreign/foreign_allocation_source.rs"]
mod foreign_allocation_source;
#[path = "foreign/foreign_claim_source.rs"]
mod foreign_claim_source;
#[path = "foreign/foreign_credit_source.rs"]
mod foreign_credit_source;
#[path = "foreign/foreign_earned_source.rs"]
mod foreign_earned_source;
#[path = "foreign/foreign_investment_source.rs"]
mod foreign_investment_source;
#[path = "foreign/foreign_loss_source.rs"]
mod foreign_loss_source;
#[path = "foreign/foreign_source_basics.rs"]
mod foreign_source_basics;
#[path = "foreign/foreign_state_source.rs"]
mod foreign_state_source;
#[path = "health/form_1095a_source.rs"]
mod form_1095a_source;
#[path = "health/health_contribution_source.rs"]
mod health_contribution_source;
#[path = "health/health_employer_source.rs"]
mod health_employer_source;
#[path = "health/health_marketplace_source.rs"]
mod health_marketplace_source;
#[path = "home/home_site_source.rs"]
mod home_site_source;
#[path = "business/household_employment_source.rs"]
mod household_employment_source;
#[path = "health/hsa_source.rs"]
mod hsa_source;
#[path = "investment/investment_borrowing_source.rs"]
mod investment_borrowing_source;
#[path = "credits/investment_credit_source.rs"]
mod investment_credit_source;
#[path = "retirement/ira_contribution_sources.rs"]
mod ira_contribution_sources;
#[path = "retirement/ira_repayment_sources.rs"]
mod ira_repayment_sources;
#[path = "income/iso_source.rs"]
mod iso_source;
#[path = "health/marketplace_claim_source.rs"]
mod marketplace_claim_source;
#[path = "health/marketplace_record_source.rs"]
mod marketplace_record_source;
#[path = "health/medical_care_source.rs"]
mod medical_care_source;
#[path = "federal/minimum_tax_source.rs"]
mod minimum_tax_source;
#[path = "home/mortgage_certificate_source.rs"]
mod mortgage_certificate_source;
#[path = "home/mortgage_limit.rs"]
mod mortgage_limit;
#[path = "home/mortgage_point_treatment.rs"]
mod mortgage_point_treatment;
#[path = "credits/mortgage_recapture.rs"]
mod mortgage_recapture;
#[path = "home/mortgage_source.rs"]
mod mortgage_source;
#[path = "home/mortgage_subsidy_source.rs"]
mod mortgage_subsidy_source;
#[path = "business/moving_model.rs"]
mod moving_model;
#[path = "federal/nol_model.rs"]
mod nol_model;
#[path = "business/optional_se_source.rs"]
mod optional_se_source;
#[path = "retirement/pension_contribution_source.rs"]
mod pension_contribution_source;
#[path = "credits/personal_refueling_source.rs"]
mod personal_refueling_source;
#[path = "attribution/prior_care_identity_source.rs"]
mod prior_care_identity_source;
#[path = "income/prior_recovery_source.rs"]
mod prior_recovery_source;
#[path = "retirement/public_safety_source.rs"]
mod public_safety_source;
#[path = "activity/qbi_history_model.rs"]
mod qbi_history_model;
#[path = "core/shared/record_choice.rs"]
mod record_choice;
#[path = "investment/related_exchange_model.rs"]
mod related_exchange_model;
#[path = "retirement/reservist_sources.rs"]
mod reservist_sources;
#[path = "attribution/retained_loss_owners_model.rs"]
mod retained_loss_owners_model;
#[path = "credits/retained_research_model.rs"]
mod retained_research_model;
#[path = "retirement/retirement_adjustment_sources.rs"]
mod retirement_adjustment_sources;
#[path = "retirement/rmd_sources.rs"]
mod rmd_sources;
#[path = "retirement/roth_history.rs"]
mod roth_history;
#[path = "credits/saver_source.rs"]
mod saver_source;
#[path = "investment/section1231_model.rs"]
mod section1231_model;
#[path = "retirement/social_sources.rs"]
mod social_sources;
#[path = "deductions/student_loan_source.rs"]
mod student_loan_source;
#[path = "credits/tax_credit_bond_source.rs"]
mod tax_credit_bond_source;
#[path = "family/trump_account_source.rs"]
mod trump_account_source;
#[path = "deductions/vehicle_loan_source.rs"]
mod vehicle_loan_source;
#[path = "cost_recovery/vehicle_model.rs"]
mod vehicle_model;
pub use care_benefits::{
    CareBenefitPeriod, CareBenefitPeriodInput, CareBenefitRecipient, CarePeriodMethod,
    FederalLtcInsured, FederalLtcInsuredPerson, FederalLtcPayment, Ltc1099, Ltc1099Paper,
    LtcIllness, LtcInsured, LtcInsuredPerson, LtcPaidBasis, LtcPeriods, LtcPeriodsInput, LtcSource,
};
pub use care_treatment::{
    CareBenefitFunding, CareFirstYearFunding, CareFundingAmounts, CareFundingYear, CareInjuryKind,
    CarePermanentInjury, LtcContract, LtcContractPage, LtcPaymentOrigin, LtcSourceChoice,
    NonqualifiedCareTreatment, NonqualifiedCareTreatmentPage, ViaticalRegulation,
};
pub use mortgage_limit::{
    MortgageAverageBalance, MortgageHome, MortgageInterestRefund, MortgageLedger, MortgageLender,
    MortgageLoan, MortgageLoanDetails, MortgageMonthBalance, MortgagePoints, MortgageRefinance,
    MortgageRefundOrigin, MortgageSplit, MortgageUse,
};
#[path = "income/hobby.rs"]
mod hobby;
#[path = "home/home_sale.rs"]
mod home_sale;
pub use home_sale::{
    DutyHomeCare, DutyHomeUse, DutyReducedHomeUse, EarlierHomeSale, ExchangeUse, Form1099S,
    Form1099SPaper, FullDutyHomeUse, HomeCare, HomeDutyKind, HomeExchange, HomeExclusion,
    HomeLived, HomeReceived, HomeRentalUse, HomeSale, HomeSaleAnswers, HomeSaleFacts, HomeSaleMore,
    HomeSaleReason, HomeSaleSource, HomeSaleSpecial, HomeSaleTrigger, HomeSaleUse, HomeService,
    HomeServiceCare, HomeServiceElection, HomeSpouse, HomeUse, MainHomeSale, OtherRealEstateSale,
    PriorExchange, QualifiedHomeDuty, RealEstateHolding, RealEstateSaleUse, ReducedHomeUse,
    SeparateHomePart, SeparateUse,
};
#[path = "home/debt.rs"]
mod debt;
#[path = "home/debt_page.rs"]
mod debt_page;
pub use debt::{
    DebtInsolvency, DebtProperty, DebtPropertyKind, DebtPropertySale, FarmDebtExclusion,
    HealthLoanAssistance, HealthLoanProgram, PersonalBankruptcy, PersonalDebtCancellation,
    PersonalDebtTreatment, PersonalInsolvency, PropertyAcquisition, PropertyTransfer,
    RealPropertyDebtExclusion, StudentDischargeReason, StudentServiceLender, TransferredProperty,
    UnsupportedDebt,
};
pub use debt_page::{
    AcquiredProperty, AcquiredPropertyView, Acquisition1099A, Cancel1099C, CanceledDebtChoice,
    DebtSolvency, Form1099A, Form1099C, Form1099CEventCode, PersonalDebtSource,
    PropertyAcquisitionSource, PropertyTaken,
};
#[path = "income/employee_stock.rs"]
mod employee_stock;
#[path = "income/espp_return.rs"]
mod espp_return;
#[path = "income/espp_sale.rs"]
mod espp_sale;
#[path = "retirement/rrb_document.rs"]
mod rrb_document;
pub use rrb_document::{Rrb1099, Rrb1099Paper, RrbBenefitYear, RrbRecipient};
#[path = "income/iso_return.rs"]
mod iso_return;
#[path = "income/iso_sale.rs"]
mod iso_sale;
#[path = "federal/minimum_tax.rs"]
mod minimum_tax;
#[path = "federal/rare_additional_tax.rs"]
mod rare_additional_tax;
pub use rare_additional_tax::RareAdditionalTax;
// Work in progress: QBI history and full K-1 return integration remain pending.
#[path = "activity/participation.rs"]
mod participation;
#[path = "passthrough/scorp/mod.rs"]
mod scorp;
pub use participation::ScorpParticipation;
#[path = "business/participation.rs"]
mod direct_participation;
pub use direct_participation::{ParticipationFacts, ParticipationTest, ParticipationYear};
#[path = "passthrough/scorp_amt.rs"]
mod scorp_amt;
pub use scorp_amt::{ScorpAmt, ScorpAmtDebt};
#[path = "passthrough/scorp_qbi.rs"]
mod scorp_qbi;
pub use scorp_qbi::{ScorpLossQbi, ScorpLossYear, ScorpPropertyQbi, ScorpQbiStatement};
#[path = "passthrough/scorp_1231.rs"]
mod scorp_1231;
pub use scorp_1231::K1Section1231;
#[path = "passthrough/scorp_capital.rs"]
mod scorp_capital;
pub use scorp_capital::ScorpCapital;
#[path = "passthrough/scorp_risk.rs"]
mod scorp_risk;
pub use scorp_risk::{ScorpRisk, ScorpRiskStart};
#[path = "passthrough/k1_activity.rs"]
mod k1_activity;
pub use k1_activity::{
    K1ActivityChoiceContext, K1ActivityParticipation, K1ActivityPartnerBenefits,
    K1ActivityPersonalSource, K1ActivityScorpBenefits, K1ActivitySourceChoice, K1ActivitySources,
    K1ActivitySourcesPage, K1ComponentDetailsPage, K1ComponentQualified, K1ComponentStatement,
    K1ComponentStatementPage, K1CurrentFuelSource, K1CurrentProperty, K1CurrentSourceParts,
    K1CurrentStatementTotals, K1EntityOnlyAmounts, K1EntityOnlyAmt, K1InvestmentPropertySource,
    K1InvestmentPropertySourcePage, K1PartnerSeSource, K1PartnershipActivitiesPage, K1PassiveGroup,
    K1PriorFuelSource, K1QbiAllocation, K1QbiAllocationPage, K1QbiBlock, K1QbiBlockPage,
    K1QbiBusinessName, K1QualifiedCurrentControls, K1ReportedQbiStatement, K1RetainedLossCarry,
    K1RetainedSeHistory, K1RiskGroup, K1RiskInvestmentParts, K1RiskSource, K1ScorpActivitiesPage,
    K1ScorpBlockStatement, K1Section179SaleSource, K1Section179SaleSourcePage, K1StatementActivity,
    K1StatementActivityPage, K1StatementAmounts, K1StatementAmt, K1StatementCapitalCarry,
    K1StatementItems, K1StatementKind, K1StatementPartnerCapitalCarry, K1StatementPortfolio,
    K1StatementProperty, K1StatementScorpCapitalCarry, K1StatementSe, PartnerActivityK1,
    PartnerEntityBasis, PartnerEntitySource, PartnerEntitySourcePage, PartnerPersonSource,
    ScorpActivityK1, ScorpEntityBasis, ScorpEntityLoan, ScorpEntitySource, ScorpEntitySourcePage,
    ScorpPersonSource,
};
#[path = "passthrough/k1_depletion_sources.rs"]
mod k1_depletion_sources;
pub use k1_depletion_sources::{
    K1DepletionSourceAllocation, K1DepletionSourceAllocationPage, K1DepletionSourcesPage,
};
#[path = "passthrough/k1_excess.rs"]
mod k1_excess;
#[allow(dead_code)]
#[path = "passthrough/k1_items.rs"]
mod k1_items;
pub use k1_items::{
    K1Disposition, K1Section179Sale, K1Section179SaleAmounts, K1Section179SaleAmtSource,
    PartnerDeduction, PartnerDeductionCode, PartnerIncome, PartnerIncomeCode, PartnerIncomeTotal,
    PartnerSaleParts, PartnerSpecial, ScorpDeduction, ScorpDeductionCode, ScorpIncome,
    ScorpIncomeCode, ScorpSpecial,
};
#[path = "passthrough/k1_loss.rs"]
mod k1_loss;
#[path = "activity/ptp_passive.rs"]
mod ptp_passive;
pub use k1_loss::{K1LossCarry, K1LossCarryforwards, PartnerCapitalCarry};
pub use ptp_passive::{
    PtpPalActivity, PtpPalCharacter, PtpPalComponent, PtpPalIssue, ptp_passive_source,
};
#[path = "foreign/estate_foreign.rs"]
mod estate_foreign;
#[path = "passthrough/estate_trust.rs"]
mod estate_trust;
#[path = "foreign/foreign_allocation_page.rs"]
mod foreign_allocation_page;
pub use estate_foreign::{
    EstateForeignPortfolioAllocation, EstateForeignPortfolioAmounts, ForeignAllocationSource,
};
pub use estate_trust::{
    EstateActivities, EstateActivitiesPage, EstateActivity, EstateActivityKind, EstateActivityPage,
    EstateAmtAdjustment, EstateAmtAdjustmentPage, EstateAmtCode, EstateAmtLine,
    EstateAmtReconciliation, EstateAmtReconciliationPage, EstateAmtTreatment,
    EstateAmtTreatmentPage, EstateCapitalLossParts, EstateCreditCode, EstateFinalCapital,
    EstateFinalCharacter, EstateFinalCharacterPage, EstateFinalDeduction, EstateFinalDeductionPage,
    EstateForeignStatement, EstateForeignTreatment, EstateInvestmentCharacter,
    EstateInvestmentIncome, EstateInvestmentProperty, EstateInvestmentPropertyPage,
    EstateParticipation, EstatePassiveCarry, EstateQbiAllocation, EstateQbiAllocationPage,
    EstateQbiKind, EstateQbiStatement, EstateQbiStatementPage, EstateQbiStatements,
    EstateQbiStatementsPage, EstateReportedCredit, EstateReportedCreditPage, EstateSourceChoice,
};
pub use foreign_allocation_page::{
    ForeignAllocationSourcePage, ForeignInvestmentAssetPage, ForeignInvestmentAssetsPage,
    ForeignStateIncomePage, ForeignStatePaymentPage, ForeignStateTaxesPage,
    InvestmentAssetSourcePage, InvestmentAssetYieldPage,
};
#[path = "passthrough/k1_page.rs"]
mod k1_page;
#[path = "passthrough/k1_period.rs"]
mod k1_period;
pub use k1_period::K1EntityTaxPeriod;
#[path = "passthrough/k1_rental.rs"]
mod k1_rental;
#[path = "passthrough/k1_sale_qbi.rs"]
mod k1_sale_qbi;
#[path = "passthrough/k1_sales.rs"]
mod k1_sales;
#[path = "passthrough/k1_totals.rs"]
mod k1_totals;
pub use k1_rental::{
    K1RentalActivity, K1RentalAmounts, K1RentalFacts, K1RentalParticipation, K1RentalProperty,
    K1RentalQbi,
};
#[path = "passthrough/schedule_k3.rs"]
mod schedule_k3;
pub use schedule_k3::{
    K3Assets, K3Attached, K3Capital, K3Category, K3CategoryAmounts, K3Country, K3Item,
    K3PaperColumn, K3PaperColumns, K3PaperCountryCell, K3PaperCountryRow, K3PaperDeductions1065,
    K3PaperDeductions1120S, K3PaperForeignTaxes, K3PaperIncome1065, K3PaperIncome1120S,
    K3PaperIncomeLine, K3PaperInterestAssets, K3PaperOtherInformation, K3PaperPartII1065,
    K3PaperPartII1120S, K3PaperResearchRow, K3PaperSourcedColumns, K3PaperTaxColumns,
    K3PaperTaxRow, K3PaperTaxSourceColumns, K3PaperUsTotalColumns, K3PartnerInterest,
    K3PartnershipPaper, K3ScorpPaper, K3Withholding, ScheduleK3, ScheduleK3Paper,
};
#[allow(dead_code)]
#[path = "passthrough/scorp_basis.rs"]
mod scorp_basis;
pub use k1_page::{
    BasisStart, EstateCurrentAccountingPage, EstateCurrentPeriodPage, K1CreditCallBox,
    K1EstateTrust, K1EstateTrustAmt, K1EstateTrustBusiness, K1EstateTrustEstimatedTax,
    K1EstateTrustOtherBoxes, K1EstateTrustRoyalties, K1FinalYear, K1HousingAcquisition,
    K1IncomeSource, K1InvestmentConfirmation, K1PartnershipCalls, K1RentalBoxes,
    K1RoyaltyDeduction, K1ScorpAtRisk, K1ScorpBasis, K1ScorpBasisRecords, K1ScorpCalls,
    K1ScorpCarried, K1ScorpLossHistory, K1ScorpPropertyQbi, K1ScorpQbi, K1ScorpQbiRecords,
    K1ScorpStart, K1StockTerm, LimitedParticipation, Participation, PartnerAmtRecords,
    PartnerAtRisk, PartnerBasis, PartnerBasisRecords, PartnerCarried, PartnerDisposition,
    PartnerQbi, ScorpAmtRecords, ScorpDisposition, ScorpRecordFacts,
};
pub(crate) use k1_page::{K1PartnershipOtherBoxes, K1ScorpOtherBoxes, SeGross};
#[path = "passthrough/estate_trust_paper.rs"]
mod estate_trust_paper;
pub use estate_trust_paper::{
    EstateActivityPaper, EstateAmtAdjustmentPaper, EstateCreditCodePaper, EstateCreditSource,
    EstateCreditStatementPaper, EstateInvestmentPropertyPaper, EstatePropertyStatementPaper,
    EstateQbiStatementPaper, EstateRoyaltyDeductionCharacter, EstateTrustCodeRow,
    EstateTrustStatementsPaper, K1EstateTrustDocument, K1EstateTrustPaper,
};
#[path = "passthrough/scorp_source.rs"]
mod scorp_source;
pub use scorp_source::{
    K1ScorpDocument, K1ScorpPaper, K1ShareCountPair, ScorpIncomeStatement, ScorpPersonalRecords,
    ScorpPropertyStatement, ScorpSource, ScorpStatements, StoredScorpDocument,
};
#[path = "passthrough/partnership_source.rs"]
mod partnership_source;
pub use partnership_source::{
    K1CapitalAccount, K1CodeRow, K1DisregardedEntity, K1Liabilities, K1MoneyPair,
    K1PartnershipDocument, K1PartnershipPaper, K1PercentagePair, K1SharePercentages,
    PartnerBasisNative, PartnerBasisStartNative, PartnerCreditPersonal, PartnerInvestmentPersonal,
    PartnerPersonalRecords, PartnershipCreditStatement, PartnershipIncomeStatement,
    PartnershipK3Source, PartnershipPropertyStatement, PartnershipRentalStatement,
    PartnershipSource, PartnershipStatementSituation, PartnershipStatements,
    StoredPartnershipDocument,
};
// Calculation prerequisite; partner source/return integration remains in progress.
#[allow(dead_code)]
#[path = "passthrough/partnership.rs"]
mod partnership;
#[path = "passthrough/partnership_qbi.rs"]
mod partnership_qbi;
pub use partnership_qbi::{PartnerQbiHistory, PartnerQbiRecords, PartnerQbiStatement};
#[allow(dead_code)]
#[path = "passthrough/partnership_basis.rs"]
mod partnership_basis;
pub use minimum_tax::{
    MinimumTaxExclusion, MinimumTaxForeignCategory, MinimumTaxForeignCredit,
    MinimumTaxForeignEarned, MinimumTaxForeignGainSource, MinimumTaxGains, MinimumTaxHistories,
    MinimumTaxHistoriesPage, MinimumTaxHistory, MinimumTaxHistoryUnit, MinimumTaxHistoryUnitPage,
    MinimumTaxLoss, MinimumTaxLossAmount, MinimumTaxLossRecords, MinimumTaxQualifiedRateFacts,
    MinimumTaxRateFacts, MinimumTaxReturnForm, MinimumTaxScheduleDRateFacts,
    OwnedMinimumTaxHistory, OwnedMinimumTaxHistoryPage,
};
pub use partnership::{
    PartnerActivityRecord, PartnerBasisRecord, PartnerFuelTaxCredit, PartnerK1, PartnerK1Item,
    PartnerParticipation, PartnerPriorFuelCredit, PartnerRiskRecord, PartnerSeGross,
    PartnerSeHistory, PartnerSeparateItems, PartnerSeparateTotals,
};
pub use scorp::{
    ScorpK1, ScorpK1Item, ScorpLoan, ScorpLoanKind, ScorpNoteHolding, ScorpNoteRepayment,
    ScorpPassiveHistory, ScorpTradeKind,
};
#[path = "family/after_death_income_source.rs"]
mod after_death_income_source;
#[path = "investment/annual_debt_source.rs"]
mod annual_debt_source;
#[path = "investment/debt_reporting_source.rs"]
mod debt_reporting_source;
#[path = "investment/dividend_source.rs"]
mod dividend_source;
#[path = "investment/interest_source.rs"]
mod interest_source;
#[path = "investment/investment_allocation_source.rs"]
mod investment_allocation_source;
#[path = "investment/investment_statements/mod.rs"]
mod investment_statements;
#[path = "investment/investment_validation_source/mod.rs"]
mod investment_validation_source;
#[path = "investment/oid_source.rs"]
mod oid_source;
#[path = "investment/reit_dividend_source.rs"]
mod reit_dividend_source;
pub use investment_statements::{
    AnnualDebtRecord, AnnualDebtRecords, AnnualDebtTreatment, BondPremiumInstrument,
    BondPremiumSource, DebtComponents, DebtPremiumOffset, DebtReporting, DividendNominee,
    InterestAdjustments, InterestNominee, OidNominee, OriginalDebtIssue, PremiumHistory,
};
pub use investment_statements::{
    PrivateLoanInterest, PrivateLoanResidence, SellerMortgageInterest, SellerMortgageParty,
};
#[path = "retirement/nua_sale.rs"]
mod nua_sale;
#[path = "income/prizes.rs"]
mod prizes;
#[path = "retirement/social.rs"]
mod social;
#[path = "retirement/social_documents.rs"]
mod social_documents;
pub use social_documents::{SocialPremiumAllocation, SocialPremiumUse, SocialPremiumUsePage};
#[path = "investment/stock_loss.rs"]
mod stock_loss;
pub use employee_stock::{EmployeeShareSale, EmployeeShareSaleMore, EmployeeStockSale};
pub use espp_return::EsppAcquisitionPage;
pub use espp_sale::{
    EsppAcquisition, EsppAcquisitionChoice, EsppAcquisitionFacts, EsppPlanRecord, EsppSale,
    EsppSaleAnswers, EsppShareSale, EsppShareSaleMore, Form3922, Form3922Paper,
};
pub use iso_sale::{IsoSale, IsoSaleAnswers, IsoShareSale, IsoShareSaleMore};
pub use nua_sale::{
    NuaInheritance, NuaOwnership, NuaSale, NuaSaleAnswers, NuaShareSale, NuaShareSaleMore,
};
#[path = "investment/opportunity_fund.rs"]
mod opportunity_fund;
pub use opportunity_fund::{QofDeferral, QofEvent, QofGainCode, QofInvestment, QofInvestmentPaper};
#[path = "home/casualty/mod.rs"]
mod casualty;
#[path = "deductions/charity.rs"]
mod charity;
pub use charity::{
    AcquiredBy, Appraisal, CharityCarryover, CharityClass, ConservationPurpose, DoneeKind,
    Form1098C, Form1098CDocument, Form1098CPaper, Form1098CTreatment, GiftKind, GoodsCondition,
    HistoricBuildingGift, PropertyCategory, PropertyGift, PropertyGiftPage, SimilarPropertyKind,
    ValueMethod, VehicleGiftDocuments, VehicleGiftPage, VehicleGiftSource,
};
#[path = "investment/schedule_b.rs"]
mod schedule_b;
#[path = "deductions/student_host.rs"]
mod student_host;
#[path = "credits/tax_credit_bonds.rs"]
mod tax_credit_bonds;
#[path = "payments/tax_refund.rs"]
mod tax_refund;
pub use tax_credit_bonds::{
    CreditIncomeSource, FederalTaxCreditBond1097, FederalTaxCreditBondKind, Form1097Btc,
    Form1097BtcBondType, Form1097BtcIdentifierCode, Form1097BtcPaper, TaxCreditBondAnswers,
    TaxCreditBondAnswersInput, TaxCreditBondDocument, TaxCreditBondPage, TaxCreditDirect,
    TaxCreditEntity, TaxCreditEntityInput, TaxCreditIncomeKind, TaxCreditOrigin,
    TaxCreditOriginInput,
};
#[path = "deductions/volunteer_charity.rs"]
mod volunteer_charity;
#[path = "deductions/whaling_charity.rs"]
mod whaling_charity;
pub use casualty::rental::{
    CasualtyKind, PropertyCasualty, RentalCasualty, RentalCasualtyBasis, RentalCasualtyRecords,
};
pub use casualty::{
    CasualtyAcquisition, CasualtyDeclaration, CasualtyHomeLinkPage, CasualtyProperty,
    CasualtyPropertyPage, HomeCasualtyDamage, PersonalCasualty, PersonalCasualtyPage,
};
pub use hobby::{FederalHobbySales, HobbySales};
pub use social_sources::{LumpSumElection, SocialBenefitYear, SocialElection};
pub use stock_loss::{Section1244Stock, StockLossBasis, StockLossEvent, StockLossOwnership};
pub use student_host::{HostedStudent, HostedStudentMonth};
pub use tax_refund::{PeaseFilingStatus, PeaseWorksheet, TaxRefundHistory, TaxRefundYearPayment};
pub use volunteer_charity::{VolunteerCarCost, VolunteerExpense, VolunteerExpenseKind};
pub use whaling_charity::{
    WhalingBoatUse, WhalingCaptainClaim, WhalingExpense, WhalingExpenseKind, WhalingPaymentEvidence,
};
#[path = "family/scholarship.rs"]
mod scholarship;
#[path = "family/scholarship_source.rs"]
mod scholarship_source;
#[path = "payments/underpayment.rs"]
mod underpayment;
#[path = "payments/underpayment_model.rs"]
mod underpayment_model;
pub use prizes::PrizeDisposition;
#[path = "income/information_returns.rs"]
mod information_returns;
#[path = "income/other_income.rs"]
mod other_income;
pub use information_returns::{
    FederalForm1099K, FederalForm1099Misc, FederalForm1099Nec, FederalGrantUse, FederalKPart,
    FederalKUse, Form1099K, Form1099KPaper, Form1099KStateRow, Form1099Misc, Form1099Nec,
    Form1099NecPaper, GovernmentComponentIds, IncomeUse, K1099Source, KPart, KUse,
    MiscComponentIds, Nec1099Source,
};
pub use other_income::{OtherIncomeItem, OtherIncomeKind, PriorYearRecovery};
pub use underpayment::{
    AnnualizedIncome, AnnualizedPeriod, DatedPayment, PenaltyWaiver, PenaltyWaiverReason,
    PeriodWithholding, PriorYearTax,
};
#[path = "business/schedule_c_specials.rs"]
mod schedule_c_specials;
pub use schedule_c_specials::{
    BusinessDepletion, DeMinimisElection, InformationReturnFiling, MealExpense, MealKind,
    OtherBusinessIncome,
};
#[path = "retirement/sep.rs"]
mod sep;
pub use scholarship::{EducationAward, Scholarship, ScholarshipService};
pub use sep::{BusinessRetirement, PartnerRetirement, SimpleEmployer};
#[path = "retirement/rmd_inherited.rs"]
mod rmd_inherited;
#[path = "retirement/rmd_minimum.rs"]
mod rmd_minimum;
pub use rmd_inherited::{InheritedEligibility, InheritedIraKind, InheritedRmd};
#[path = "retirement/rmd_tables.rs"]
mod rmd_tables;
pub use rmd_minimum::{RmdIra, RmdMinimum, RmdPeriod, RmdSpouse};
#[path = "retirement/rmd.rs"]
mod rmd;
pub use rmd_sources::{
    MissedDistribution, RequiredDistribution, RmdCorrection, RmdFiling, RmdWaiver,
};
#[path = "retirement/inherited_ira.rs"]
mod inherited_ira;
#[path = "retirement/inherited_ira_source.rs"]
mod inherited_ira_source;
pub use inherited_ira_source::{
    FederalInheritedIra, InheritedEstateTax, InheritedIraAccount, InheritedIraBasis,
    InheritedIraKindFacts, InheritedRothBasis,
};
#[path = "retirement/roth.rs"]
mod roth;
pub use roth_history::{RothConversion, RothHistory, RothIraHistory, RothOrdering};
#[path = "retirement/roth_source.rs"]
mod roth_source;
pub use roth_source::{
    InheritedLink, RothCode, RothIraCode, RothIraCodeView, RothReason, RothReasonView,
    RothWithdrawal,
};
#[path = "health/hsa.rs"]
mod hsa;
#[path = "health/hsa_funding.rs"]
mod hsa_funding;
pub use hsa_funding::{
    HsaFunding, HsaFundingBasis, HsaFundingRow, HsaFundingSourceChoice, HsaFundingSourceKind,
    HsaFundingUseHistory,
};
#[path = "health/hsa_last_month.rs"]
mod hsa_last_month;
pub use hsa_last_month::{HsaFamilyAllocation, HsaLastMonthHistory};
#[path = "health/hsa_correction.rs"]
mod hsa_correction;
pub use health::{
    BusinessCareMonth, BusinessCarePremium, BusinessCarePremiumPage, BusinessHealthInsurance,
    BusinessHealthMonth, CareContractQualification, EntityPremiumPayment, HealthEstablishment,
    HealthEstablishmentPage, MedicalCarePremium, ReimbursedHealthPremiums,
};
pub use hsa_correction::{
    HealthReturnDocumentChoice, HsaCorrection, HsaCorrectionPage, HsaCorrectionPayer,
};
#[path = "health/form_1095a_paper.rs"]
mod form_1095a_paper;
#[path = "health/marketplace_allocation.rs"]
mod marketplace_allocation;
pub use form_1095a_paper::{
    Form1095ACoverageAmounts, Form1095ACoveredIndividual, Form1095APaper, Form1095ARecipient,
};
#[path = "health/marketplace_facts.rs"]
mod marketplace_facts;
#[path = "health/marketplace_marriage.rs"]
mod marketplace_marriage;
#[path = "health/marketplace_projection.rs"]
mod marketplace_projection;
#[path = "health/marketplace_qsehra.rs"]
mod marketplace_qsehra;
#[path = "health/marketplace_sources.rs"]
mod marketplace_sources;
pub use marketplace_facts::{
    MarketplaceAgreement, MarketplaceAllocatedBenchmark, MarketplaceAllocatedBenchmarkPage,
    MarketplaceAllocationKind, MarketplaceAllocationKindPage, MarketplaceAllocationPeriod,
    MarketplaceAllocationPeriodPage, MarketplaceAppeal, MarketplaceAppealPage,
    MarketplaceAttestation, MarketplaceAttestationPage, MarketplaceAttester,
    MarketplaceAttesterPage, MarketplaceBenchmarkGroup, MarketplaceBenchmarkGroupPage,
    MarketplaceBenchmarkMonth, MarketplaceBenchmarkMonthPage, MarketplaceBenchmarkSource,
    MarketplaceBenchmarkSourcePage, MarketplaceCoverageEntry, MarketplaceCoverageEntryPage,
    MarketplaceCoveredPerson, MarketplaceCoveredPersonPage, MarketplaceEligibilityMonth,
    MarketplaceEligibilityMonthPage, MarketplaceEnrollmentBasis, MarketplaceEventKind,
    MarketplaceFacts, MarketplaceFactsPage, MarketplaceFamily, MarketplaceFamilyPage,
    MarketplaceFamilyRef, MarketplaceFamilyRefPage, MarketplaceFamilyShare,
    MarketplaceFamilySharePage, MarketplaceFraction, MarketplaceIncomeQualification,
    MarketplaceLawfulPremiumMonth, MarketplaceLawfulPremiumMonthPage,
    MarketplaceLawfulPremiumSource, MarketplaceMaritalFamilyShare,
    MarketplaceMaritalFamilySharePage, MarketplaceMaritalPair, MarketplaceMaritalPairPage,
    MarketplaceMarriage, MarketplaceMarriageFamily, MarketplaceMarriageFamilyPage,
    MarketplaceMarriagePage, MarketplaceMecKind, MarketplaceMedicalCostMonth,
    MarketplaceNoAdvanceShare, MarketplaceNoAdvanceSharePage, MarketplaceNonreturnFamily,
    MarketplaceNonreturnFamilyPage, MarketplacePaymentCoverage, MarketplacePaymentMonth,
    MarketplacePaymentMonthPage, MarketplacePersonEligibility, MarketplacePersonFamily,
    MarketplacePersonFamilyPage, MarketplacePersonIdentity, MarketplacePersonIdentityPage,
    MarketplacePolicyCoverage, MarketplacePolicyCoveragePage, MarketplacePolicyRecord,
    MarketplacePolicyRecordPage, MarketplaceQsehra, MarketplaceQsehraDetermination,
    MarketplaceQsehraEmployeeFacts, MarketplaceQsehraEmployeeFactsPage,
    MarketplaceQsehraEmployeeHousehold, MarketplaceQsehraEmployeeHouseholdPage,
    MarketplaceQsehraHouseholdMember, MarketplaceQsehraHouseholdMemberPage,
    MarketplaceQsehraIncome, MarketplaceQsehraIncomeSource, MarketplaceQsehraMonth,
    MarketplaceQsehraMonthPage, MarketplaceQsehraPage, MarketplaceQsehraSource,
    MarketplaceQsehraSourcePage, MarketplaceStatementStatus,
};
#[path = "health/marketplace.rs"]
mod marketplace;
pub use marketplace::{MarketplaceClaim, MarketplaceDependent, MarketplaceResidence};
#[path = "retirement/annuity_tables.rs"]
mod annuity_tables;
#[path = "attribution/deduction_owners.rs"]
mod deduction_owners;
#[path = "investment/investment_interest.rs"]
mod investment_interest;
pub use kernel::attribution::DeductionOwners;
#[path = "retirement/ira.rs"]
mod ira;
pub use ira::QtpRothRollover;
pub use ira::{IraExcess, IraExcessEntry, IraKind, LateCorrection, ReturnedContribution};
#[path = "family/child_election.rs"]
mod child_election;
#[path = "family/child_election_source.rs"]
mod child_election_source;
#[path = "family/kiddie.rs"]
mod kiddie;
pub use child_election::{
    ChildChoice, ChildDividendStatement, ChildElectionFacts, ChildStatement, Form8814,
    OtherParentChoice,
};
#[path = "deductions/personal_taxes.rs"]
mod personal_taxes;
pub use investment_interest::{
    InvestmentBorrowing, InvestmentBorrowingPage, InvestmentElection, InvestmentIncomeTax,
    InvestmentInterest, InvestmentInterestUse, InvestmentK1Origin, InvestmentLimitedAllocation,
    InvestmentLoan, InvestmentLoanPage, InvestmentSourceChoice, InvestmentTaxCapAllocation,
};
pub use kiddie::{
    BusinessIncomeChoice, BusinessIncomeSource, ChildGains, ChildSpecialGains, OtherChildTax,
    ParentGains, ParentReturn, ParentSituation, ParentTax, WorkValue,
};
pub use personal_taxes::{PersonalTaxElection, TaxIncomeAllocation};
#[path = "retirement/pension.rs"]
mod pension;
pub use pension::{
    AnnuityDuration, AnnuityFrequency, AnnuityLoss, GeneralAnnuity, PensionBasis, PensionChange,
    PensionDuration, PensionHistory, PensionShare, VariableAnnuity, VariableEarlier,
    VariableHistory, VariablePayments, VariableRefigure, VariableTerm,
};
#[path = "foreign/foreign_investment_assets.rs"]
mod foreign_investment_assets;
#[path = "foreign/foreign_loss_history.rs"]
mod foreign_loss_history;
pub use foreign_loss_history::{ForeignLossAccount, ForeignLossHistory, ForeignLossSystem};
#[path = "foreign/foreign_losses.rs"]
mod foreign_losses;
pub use foreign_investment_assets::{
    ForeignInvestmentAsset, ForeignInvestmentAssets, InvestmentAssetBasis, InvestmentAssetSource,
    InvestmentAssetYield, InvestmentBasisChange,
};
#[path = "foreign/foreign_state_tax.rs"]
mod foreign_state_tax;
pub use foreign_state_tax::{ForeignStateIncome, ForeignStatePayment, ForeignStateTaxes};
#[path = "foreign/foreign_credit_history.rs"]
mod foreign_credit_history;
pub use foreign_credit_history::{
    ForeignCarryback, ForeignCarryoverColumn, ForeignCarryoverOpening, ForeignCarryoverRecords,
    ForeignCarryoverSchedule, ForeignCreditCategory, ForeignCreditHistory, ForeignPriorCredit,
    ForeignPriorLimit,
};
#[path = "foreign/foreign_capital.rs"]
mod foreign_capital;
#[path = "foreign/form_1116.rs"]
mod form_1116;
pub(crate) use form_1116::ForeignTaxSituation;
#[path = "retirement/retirement.rs"]
mod retirement;
#[path = "retirement/retirement_return.rs"]
mod retirement_return;
pub use extended_model::{
    BusinessInventory, BusinessMileage, BusinessPayroll, CapitalLossOrigin,
    Commercial1993ConstructionSource, Commercial1993ContractSource, Commercial1993Transition,
    ExemptPension, FederalFarmRental, FederalRental, FederalRoyalty, ForeignAmtLimitation,
    ForeignInterestAllocation, ForeignOrdinaryTax, ForeignQualification, ForeignRateAdjustment,
    ForeignSourceCategory, ForeignSourceRow, ForeignTaxBasis, ForeignTaxCategory, ForeignTaxClaim,
    ForeignTaxDeducted, ForeignTaxDeductedPage, ForeignTaxIncome, ForeignTaxIncomePage,
    ForeignTaxInitial, ForeignTaxPage, ForeignWages, ForeignWithholding, Form3921, Form3921More,
    Form3921Paper, HsaEligibility, InvestmentAmtCapitalParts, InvestmentCapitalCap,
    InvestmentCapitalParts, IsoExercise, IsoExerciseSource, IsoRecognition, PayrollQbiAttribution,
    PayrollW2, PriorAmtCapitalReturn, PriorCapitalReturn, PriorReturnOwners, RentalAcquisition,
    RentalAtRisk, RentalBuilding, RentalBusiness, RentalExpense, RentalExpenseCategory,
    RentalHistory, RentalHomeCarries, RentalLocation, RentalParticipation, RentalPersonalUse,
    RentalPropertyType, ResidenceBasis, RoyaltyDepletion, RoyaltyKind, RoyaltyMineralRate,
    ShortTermStays,
};
pub use federal_model::FederalTaxRefund;
pub use federal_model::{
    BadDebtSource, BasisReporting, BbaAuditAnswers, BusinessAccounting, BusinessActivity,
    BusinessExpenseCategory, BusinessExpenseKind, BusinessSource, CapitalDistributionKind,
    CapitalSaleAmt, CareBenefitRow, CarePage, CareQualifyingRow, CareSeed, CareSeedBenefitRow,
    CareSeedRow, ClaimBasis, CleanVehicleDisposition, CommunityAnswer, CommunityProperty,
    CommunityRule, CommunityShare, CommunityState, CompetingClaim, DeductionChoice, DependentCare,
    DependentResidency, DisabilityWages, DisasterActionKind, DisasterCarrierLoss,
    DisasterMitigationProgram, DisasterQualification, DisasterReliefFacts, DisasterReliefPurpose,
    EducationChoice, EducationMethod, EducationStudent, EducationStudentPage, EducatorClaim,
    EstateTrustAnswers, EstateTrustK1Item, EstateTrustNiitAdjustment, EstateTrustSource,
    EstateTrustStatements, FederalAdjustments, FederalAlternativeFuelQuantity,
    FederalAlternativeFuelUse, FederalAviationGasolineFarmUse, FederalAviationGasolineForeignTrade,
    FederalAviationGasolineType10Uses, FederalBbaAuditInterest, FederalBusFuelUses,
    FederalBusiness, FederalBusinessExpense, FederalBusinessOvertime, FederalBusinessTips,
    FederalCashFuelRecovery, FederalCashFuelRecoveryEvent, FederalCasualtyLivingInsurance,
    FederalCasualtyLivingPayment, FederalCleanVehicle, FederalCleanVehicleClaim,
    FederalCleanVehicleDetails, FederalCleanVehiclePrior, FederalCommercialAviationKerosene,
    FederalCredits, FederalDeductions, FederalDependent, FederalDisasterMitigation,
    FederalDisasterRelief, FederalDividend, FederalDraft, FederalEducation,
    FederalEducationExpenses, FederalEmulsionBlenderClaim, FederalEstateTrustPortfolioK1,
    FederalFarmAviationKerosene, FederalForeignTradeAircraftRegistration,
    FederalForeignTradeAviationKerosene, FederalFuelExpenseTreatment, FederalFuelLines,
    FederalFuelTaxCredit, FederalFuelUse, FederalGambling, FederalHomeKerosene,
    FederalHomeLossOfUse, FederalHsa, FederalIncome, FederalIncomeEntry, FederalInterest,
    FederalIraContribution, FederalIras, FederalIssue, FederalMortgageCertificate,
    FederalMortgageCredit, FederalOid, FederalOtherFuelUses, FederalPartialCashFuelTaxBenefit,
    FederalPayments, FederalPeople, FederalPerson, FederalPersonalRefueling, FederalRetirement,
    FederalSale, FederalSaver, FederalSaverDistribution, FederalScreening, FederalSetup,
    FederalSocial, FederalStatus, FederalUnemployment, FederalWage, FederalWildfireRelief,
    FederalWorkDeductions, FilerDependency, FilingChoice, FinalBondReporting, FinalBonds,
    FinalEstateTrustAccountingYear, FinalEstateTrustLossPeriod, FinalEstateTrustNol,
    FinalEstateTrustNolYear, FinalEstateTrustTaxPeriod, FinalEstateTrustWeekClosing, FinalReturn,
    Form1041K1, Form8986AmountRow, Form8986EntityReturn, Form8986Item, Form8986Paper,
    Form8986PartnerEntity, Form8986PassThrough, Form8986Penalty, Form8986PercentRow,
    Form8986Representative, Form8986Statement, Form8986SubmittingEntity, Form8986Totals, FormLine,
    FormLineKind, GainTerm, GovernmentPaymentSource, GuaranteeMotive, GuaranteeRight, HsaCoverage,
    InterestSource, IraNextYearDeposits, IraSpouse, ItemizeOverride, ItemizedExpenses,
    LiquidationProperty, MaritalState, MortgageCertificateHistory, MortgageInterestSource,
    MortgageOriginalInterestMethod, MortgageReissueFacts, MortgageSubsidyDisposition,
    MortgageSubsidyDispositionView, MortgageSubsidyEntry, MortgageSubsidyEntryView,
    MortgageSubsidyFacts, MortgageSubsidyGiftOwners, MortgageSubsidyKind, MortgageSubsidyPayoff,
    MortgageSubsidyQsmlGainShare, MortgageSubsidySpouseRecipient, MortgageSubsidyTransfer,
    MortgageSubsidyTransferView, NonbusinessPropertyKind, OvertimeClaim, PersonalRentalHistory,
    Relationship, RetirementKind, SaleAdjustment, SaleAmt, SaleUse, SaverChoice,
    SaverDistributionPeriod, SaverDistributionYear, SecurityAcquisition, SpouseFiler, SpouseLiving,
    StudentLoanClaim, SupportShare, TaxForm, TipsClaim, UndistributedGain, UndistributedGainSource,
    UnemploymentRepayment, W2Code, W2CodeAmount, W2Source, WageSource, WildfireReliefFacts,
    draft_return, form_coverage,
};
pub use federal_model::{
    CareCosts, Claimable, CustodyRelease, DeathYear, Dependent, DependentSituation, Dependents,
    Filer, FilerDetails, Filing, HeadMarital, Identification, LivedWithYou, ReturnSetup,
    SpouseAnswers, SpouseDetails,
};
pub use federal_model::{ChildIncome, ChildStatementKept, FederalChildElection};
// The relief and other-income kinds (unit B6b).
pub use federal_model::{GrantUse, JuryPay, ProfessionalGambling, RoyaltyProperty};
pub use federal_model::{IssueContext, PersonChoice};
// Savings bonds cashed for education (Form 8815) and the 1099-INT treatments it left unbundled.
pub use federal_model::{
    BondRedemptionLink, BondRedemptionSource, DividendSpecial, FederalSavingsBondExpenses,
    FederalSavingsBondStudent, FederalSavingsBonds, ForeignSituationChoice, Form8815EducationRow,
    Form8815Paper, Form8818BondRow, Form8818Paper, InterestSpecial, MoreCreditRefusal, OidSpecial,
    RecordChoice, SaleSpecial, SavingsBondAnswers, SavingsBondAnswersView, SavingsBondExpenses,
    SavingsBondStudent, SavingsBonds, SavingsBondsEntry, SavingsBondsSource,
};
pub use federal_model::{
    Form2439, Form2439Paper, FundQsbsExclusion, NonbusinessBadDebt, WorthlessSecurity,
};
// The person folds and the data-answer kinds (M3-P15).
pub use federal_model::line_label;
pub use federal_model::{
    ForeignAccountOrTrust, MoreAboutYou, PersonFacts, PersonUse, ProfessionalRentalElection,
    ProfessionalRentals, ProfessionalRentalsPage, QbiLossCarryforward,
};
// The situations people often miss and the ones the App does not prepare.
pub use federal_model::{
    MissedBenefits, MissedIncome, OftenMissed, OftenMissedAnswers, RareSituation, Unprepared,
};
pub use injured_spouse::{
    EstimatedSplit, InjuredAmountAnswered, InjuredAmountInitial, InjuredClaimant,
    InjuredClaimantFacts, InjuredCommunity, InjuredCommunityFacts, InjuredComponentChoice,
    InjuredDebtFacts, InjuredLoanBorrower, InjuredSpouse, InjuredSpouseAllocation,
    InjuredSpouseInitial, InjuredStateChoice, OffsetDebt,
};
#[path = "deductions/claim_repayment.rs"]
mod claim_repayment;
#[path = "credits/clean_vehicle.rs"]
mod clean_vehicle;
pub use clean_vehicle::{CleanVehicleActivity, CleanVehicleBusinessUse};
#[path = "credits/clean_vehicle_page.rs"]
mod clean_vehicle_page;
pub use clean_vehicle_page::{
    CleanVehicleActivityPage, CleanVehicleBusinessUsePage, FederalCleanVehicleClaimPage,
    FederalCleanVehiclePage,
};
#[path = "family/coverdell.rs"]
mod coverdell;
#[path = "family/coverdell_model.rs"]
mod coverdell_model;
#[path = "core/validation.rs"]
mod validation;
pub use coverdell::{
    CoverdellContribution, CoverdellContributor, CoverdellContributorKind,
    CoverdellContributorKindPage, CoverdellContributorPage, CoverdellCorrection,
    CoverdellCorrectionPage, CoverdellExcess, CoverdellExcessPage,
};
#[path = "family/historical_eic.rs"]
mod historical_eic;
#[path = "family/historical_eic_tables.rs"]
mod historical_eic_tables;
pub use claim_repayment::{
    RepaymentAmtLosses, RepaymentCapital, RepaymentClaim, RepaymentDistribution,
    RepaymentDividends, RepaymentEarly, RepaymentEic, RepaymentElderly, RepaymentExclusionBasis,
    RepaymentExclusionLosses, RepaymentFilers, RepaymentMethod, RepaymentRetirement, RepaymentSale,
    RepaymentSocial, RepaymentSsns, RepaymentTaxYear,
};
#[path = "income/elective_deferrals.rs"]
mod elective_deferrals;
#[path = "retirement/pension_contributions.rs"]
mod pension_contributions;
pub use pension_contributions::PensionContributions;
#[path = "business/employee_expenses.rs"]
mod employee_expenses;
pub use employee_expenses::{EmployeeCapacity, EmployeeExpenseClaim};
#[path = "retirement/care_distribution.rs"]
mod care_distribution;
#[path = "retirement/ira_repayment.rs"]
mod ira_repayment;
pub use ira_repayment::{IraRepayment, IraRepaymentDeposit};
#[path = "retirement/disaster/mod.rs"]
mod disaster;
pub use disaster::{DisasterCarry, EarlierForm8915, Form8915Repayments, Form8915Year};
#[path = "retirement/disaster_source.rs"]
mod disaster_source;
pub use disaster_source::{DisasterDistribution, DisasterRepayment, DisasterSpread};
#[path = "retirement/reservist.rs"]
mod reservist;
pub use reservist::{ReservistPayment, ReservistRepayment};
pub use retirement::{RailroadHolder, RailroadRepayment};
#[path = "retirement/legacy_annuity.rs"]
mod legacy_annuity;
pub use legacy_annuity::LegacyAnnuity;
pub use retirement::{
    AnnuityBalance, AnnuityDistribution, CareDistribution, ContractDischarge, DisabilityIncome,
    EmergencyHistory, IraBasis, PlanDistribution, QcdPayment, QualifiedCharity, Reemployment,
    ReserveTerm, RetirementAfterDeath, RetirementException, RetirementTaxable, RetirementTreatment,
    RolloverDestination, SeparationWork, SurvivorLumpSum, UnemploymentQualification,
};
#[path = "family/care.rs"]
mod care;
#[path = "family/care_return.rs"]
mod care_return;
pub use care::{
    CareBenefitAnswers, CareBenefitSource, CareBenefitSourceInput, CareFiler, CareFilerInput,
    CareFilerSituation, CareFilerSituationInput, CareFilingStatus, CareGraceBalance,
    CareGraceBalanceInput, CareOwnBusinessBenefit, CareOwnBusinessBenefitInput, CarePlanMaximum,
    CareProvider, CareSourceChoice, FederalCare, PriorYearCare, PriorYearCareInput,
    ProviderIdentification, ProviderRelationship,
};
#[path = "core/common.rs"]
mod common;
pub(crate) use issue_macros::{issue_localized, require_localized};
#[path = "core/line_labels.rs"]
mod line_labels;
pub use common::{
    AmountBasis, Answer, CalculationLine, LineKind, Message, MessageArg, Owner, ResultState,
};
#[path = "federal/interview/mod.rs"]
mod interview;
#[path = "core/model.rs"]
mod model;
#[path = "federal/state_return.rs"]
mod state_return;
pub use state_return::{
    BusinessStateDocument, BusinessStateForm, IncomeStateSource, InvestmentStateFields,
    LocalIncomeTax, PrintedParty, PrintedPartyNameAddress, RefundStateFields,
    RetirementStateFields, SocialSecurityStatement, StateIncomeFields, StateIncomeTax,
    StateLocalFields, StateTaxWithholding, StateWithholdingFields, TaxExemptAllocation,
    TaxExemptStateIncome, TaxPayerIdentity, TaxState, TippedOccupationCode, W2OtherItem,
    W2OtherValue, W2StateFields,
};
#[path = "core/protocol.rs"]
mod protocol;
#[cfg(test)]
mod tests;
pub use federal_model::{
    AlimonyItem, MoreAdjustment, MoreAdjustments, MoreCredit, MoreCreditInitial, MoreCredits,
    MoreCreditsInitial, MovingItem,
};
pub use federal_model::{
    AnnuityContract, CarePremium, Deductions, FinalAnnuityLoss, Medical, OtherItemized,
    SaltElection, SpouseBoxes, Taxes,
};
pub use federal_model::{
    Asset, AssetClass, AssetHistory, Box12Entry, Building, Business, BusinessExpenses,
    CapitalReturn1099Div, Contracts1099B, ContractsSource, DailyTipRecord, Disposition,
    DistributionCode, DistributionCodes, Div1099, DividendSource, EarlyException, EquipmentRent,
    ExceptionRepayment, FederalSection1256, FirstYearDeduction, Form1099DivPaper, Form1099G,
    Form1099GPaper, Form1099IntPaper, Form1099IntSource, Form1099OidPaper, Form1099OidSource,
    Form1099RPaper, Form8606, FormRrb1099RPaper, FormW2Paper, Improvement, InPlanRothBasis,
    IncomeDocumentItem, IncomeRecordItem, IncomeRecords, IncomeRefundManagementDerived,
    IncomeSource, InheritedLumpSumTax, Int1099, IntMore, InterestPrintedFacts, LumpSumTax,
    MileageMore, Oid1099, OidMore, OidPrintedFacts, OtherExpense, Patr1099, Patr1099Answers,
    Patr1099CreditAnswers, Patr1099Paper, Patr1099Source, PatrMore, PersonalUse, PrintedBox12Entry,
    PublicSafetyAnswer, QcdGift, R1099, R1099Answered, R1099Source, RCalls, RailroadRecipient,
    RailroadRecovery, RailroadSource, Refund1099G, RefundDeduction, Rental, RentalExpenses,
    RetirementSource, RetirementTreatmentView, RolloverTarget, Rrb1099R, Sale1099B, SaleSource,
    SeparateSpouseBoxes, SimpleDisposition, SocialBenefitStatement, Ssa1099, Ssa1099Paper,
    StatutoryBusiness, StatutoryBusinessView, TaxableMethod, Vehicle, VehicleMethod, W2, W2Amount,
    W2AnswerRow, W2Answers, W2AnswersView, W2Box14Ask, W2Box14Kind, W2Box14Meaning, W2Boxes,
    W2Calls, W2More, W2MoreBoxes, W2Payroll, W2StateBoxes, WageDeductionEligibility, WagesRecord,
    WagesRecordItem, WagesRecords,
};
pub use federal_model::{
    AverageBalance, BondPremiumSelection, EducatorRole, Form1098, GiftCarryovers, GoodsKind,
    LessCommonGift, Mortgage1098, Mortgage1098ConversionError, Mortgage1098Source,
    MortgageDebtDate, MortgageDetails, MortgageDetailsView, MortgageFirstLender, MortgageMore,
    MortgagePaidShare, Mortgages, OtherItemizedInput, OtherItemizedLine, VehicleUse,
    WhalingCaptain,
};
pub use federal_model::{
    BusinessBenefits, Educator, EducatorExpenses, EmployeeExpenseForm, EmployeeExpenses,
    FederalVehicleInterest, Form1098E, Form1098EPaper, Form1098VliPaper, HealthPlanPage,
    IraContributions, IraContributor, IraMore, PartnerBenefits, PartnerPlan, SeHealth,
    SeRetirement, SelfEmployedBenefits, StudentLoanDocument, StudentLoanInterest,
    StudentLoanSource, WageChoice,
};
pub use federal_model::{
    CanceledDebtDocument, IncomeRecord, IncomeSourceChoices, IncomeSourceKind,
    InterestDividendDocument, InvestmentSaleDocument, K1Document, OtherIncomeDocument,
    RetirementDocument, SelfEmploymentDocument, WageDocument,
};
pub use federal_model::{
    DivAfterDeathAnswer, DivBoxes, DivCallBoxes, DivCalls, DivNomineeAnswer, DividendEligibility,
    IntAfterDeathAnswer, IntBoxes, IntCallBoxes, IntCalls, IntNomineeAnswer, OidBoxes,
    OidCallBoxes, OidCalls, OidNomineeAnswer,
};
// Always-present detail records replacing the More switches.
pub use federal_model::{BusinessDetails, RentalDetails, SuspendedLoss};
pub use federal_model::{
    EducationCredits, EducationSource, Form1098TPaper, SaverContribution, SaverCredit, SaverMore,
    ScholarshipInclusion, StudentCredit, StudentCreditDocument, StudentMore, StudentMorePage,
};
pub use federal_model::{
    FamilySplit, HsaAccount, HsaCoverageMonths, HsaCoverageYear, HsaEligibilityPage, HsaPage,
};
pub use federal_model::{
    Farmer, FarmerNextYear, IncomeByPeriod, IncomeTiming, NextYearEstimate, NextYearInitial,
    Payments, PaymentsInitial, PaymentsMore, PaymentsMoreInitial, PenaltyChoice, PriorReturn,
    RefundChoice, Waiver, WithholdingTiming,
};
pub use federal_model::{FormW2GPaper, GamblingLosses, GamblingSource, W2g, W2gSource};
pub use federal_model::{OtherTax, OtherTaxes};
// The Form 1095-A page (T2-27p).
pub use federal_model::{DependentReturn, PolicyAmounts, PolicyMonth, SeparateRelief};
// Beyond your documents and its three catalogs; the filers' whole-return
// situations.
pub use federal_model::{
    ActivityTopics, CarryoverTopics, Checklist, ChecklistActivities, ChecklistAnswers,
    MissedIncomeTail, PaidTopics, SpouseSituations, YourSituations,
};
pub use model::{
    HouseholdPeople, HouseholdRecord, STORAGE_FORMAT, TaxAppData, TaxDataKey, WorkingHeader,
};
pub use protocol::{
    AllTopicsLink, CarriedItem, DocumentFacts, EstimateCarry, EstimateSoFar, HubTopics,
    NavigationAvailability, NavigationDestination, PersonCredits, ReviewItem, SaveWarning,
    TaxInterviewProtocol, TaxSection,
};

use alloc::boxed::Box;
use core::any::Any;
use core::num::NonZeroUsize;
use interview::Reply;
use libertas::{
    LibertasEndpoint, LibertasEndpointStatus, LibertasPeer, OP_ENDPOINT_REQ,
    libertas_endpoint_edit_request, libertas_endpoint_response,
    libertas_register_endpoint_listener,
};
use libertas_macros::{libertas_data_schema, libertas_export};

#[path = "core/storage.rs"]
mod storage;
use storage::Returns;

#[path = "investment/rental_foreign.rs"]
mod rental_foreign;

#[cfg(test)]
#[path = "core/mod.rs"]
mod core_catalog;
#[cfg(test)]
pub(crate) use core_catalog::CORE_STRINGS;
#[cfg(test)]
#[path = "activity/mod.rs"]
mod activity_catalog;
#[cfg(test)]
pub(crate) use activity_catalog::ACTIVITY_STRINGS;
#[cfg(test)]
#[path = "attribution/mod.rs"]
mod attribution_catalog;
#[cfg(test)]
pub(crate) use attribution_catalog::ATTRIBUTION_STRINGS;
#[cfg(test)]
#[path = "business/mod.rs"]
mod business_catalog;
#[cfg(test)]
pub(crate) use business_catalog::BUSINESS_STRINGS;
#[cfg(test)]
#[path = "cost_recovery/mod.rs"]
mod cost_recovery_catalog;
#[cfg(test)]
pub(crate) use cost_recovery_catalog::COST_RECOVERY_STRINGS;
#[cfg(test)]
#[path = "credits/mod.rs"]
mod credits_catalog;
#[cfg(test)]
pub(crate) use credits_catalog::CREDIT_STRINGS;
#[cfg(test)]
#[path = "deductions/mod.rs"]
mod deductions_catalog;
#[cfg(test)]
pub(crate) use deductions_catalog::DEDUCTION_STRINGS;
#[cfg(test)]
#[path = "family/mod.rs"]
mod family_catalog;
#[cfg(test)]
pub(crate) use family_catalog::FAMILY_STRINGS;
#[cfg(test)]
#[path = "farm/mod.rs"]
mod farm_catalog;
#[cfg(test)]
pub(crate) use farm_catalog::FARM_STRINGS;
#[cfg(test)]
#[path = "federal/mod.rs"]
mod federal_catalog;
#[cfg(test)]
pub(crate) use federal_catalog::FEDERAL_STRINGS;
#[cfg(test)]
#[path = "foreign/mod.rs"]
mod foreign_catalog;
#[cfg(test)]
pub(crate) use foreign_catalog::FOREIGN_STRINGS;
#[cfg(test)]
pub(crate) use forms::FORM_STRINGS;
#[cfg(test)]
#[path = "health/mod.rs"]
mod health_catalog;
#[cfg(test)]
pub(crate) use health_catalog::HEALTH_STRINGS;
#[cfg(test)]
#[path = "home/mod.rs"]
mod home_catalog;
#[cfg(test)]
pub(crate) use home_catalog::HOME_STRINGS;
#[cfg(test)]
#[path = "income/mod.rs"]
mod income_catalog;
#[cfg(test)]
pub(crate) use income_catalog::INCOME_STRINGS;
#[cfg(test)]
#[path = "investment/mod.rs"]
mod investment_catalog;
#[cfg(test)]
pub(crate) use investment_catalog::INVESTMENT_STRINGS;
#[cfg(test)]
#[path = "passthrough/mod.rs"]
mod passthrough_catalog;
#[cfg(test)]
pub(crate) use passthrough_catalog::PASSTHROUGH_STRINGS;
#[cfg(test)]
#[path = "payments/mod.rs"]
mod payments_catalog;
#[cfg(test)]
pub(crate) use payments_catalog::PAYMENT_STRINGS;
#[cfg(test)]
#[path = "retirement/mod.rs"]
mod retirement_catalog;
#[cfg(test)]
pub(crate) use retirement_catalog::RETIREMENT_STRINGS;
#[cfg(test)]
#[path = "core/string_resources.rs"]
mod string_resources;
#[cfg(test)]
pub use string_resources::APP_STRINGS;

fn handle_request(
    endpoint: LibertasEndpoint,
    opcode: u8,
    message: Option<TaxInterviewProtocol>,
    context: &mut Box<dyn Any>,
    transaction_id: u32,
    peer: LibertasPeer,
) -> LibertasEndpointStatus {
    if opcode != OP_ENDPOINT_REQ {
        return LibertasEndpointStatus::InvalidMessage;
    }
    let Some(returns) = context.downcast_mut::<Returns>() else {
        return LibertasEndpointStatus::InvalidMessage;
    };
    // The Hub verified the request's Fixed current-user field before this call.
    let Some(request) = message else {
        return LibertasEndpointStatus::InvalidMessage;
    };
    let Some(user) = request.user() else {
        return LibertasEndpointStatus::InvalidMessage;
    };
    let Some(reply) = returns.handle_wire(user, request) else {
        return LibertasEndpointStatus::InvalidMessage;
    };
    match reply {
        Reply::Response(value) => {
            libertas_endpoint_response(endpoint, &value, transaction_id, peer)
        }
        Reply::Edit(value) => {
            libertas_endpoint_edit_request(endpoint, &value, transaction_id, peer)
        }
    }
    LibertasEndpointStatus::Success
}

/// Tax return prototype
/// Prepares a 2026 U.S. federal income tax return estimate. Import your tax documents with AI, answer only the questions your documents and situation raise, and see your estimated tax, payments and balance due or refund. One saved return per user; resume where you left off. Alpha version.
#[libertas_export]
#[libertas_data_schema(key = TaxDataKey, value = TaxAppData)]
pub fn tax_interview(
    /*
     * Tax return
     * Open the synthetic U.S. federal tax year 2026 interview.
     * [DefaultText]
     * Tax return prototype
     */
    #[libertas_endpoint_schema(TaxInterviewProtocol)]
    #[libertas_endpoint_server]
    interview: LibertasEndpoint,
    /*
     * Cached return bytes
     * Maximum total encoded record bytes kept in memory. Least recently used returns
     * are reloaded from storage. A larger return works without being cached.
     */
    #[libertas_default(67108864)]
    #[libertas_number(min = 1)]
    cache_bytes: u32,
) {
    let capacity = NonZeroUsize::new(cache_bytes as usize).expect("cache_bytes must be positive");
    libertas_register_endpoint_listener::<TaxInterviewProtocol, _>(
        interview,
        handle_request,
        Box::new(Returns::new(capacity)),
    );
}

#[path = "retirement/public_safety.rs"]
mod public_safety;
pub use public_safety_source::{
    PublicSafetyCoverage, PublicSafetyPaid, PublicSafetyPaidCoverage, PublicSafetyPremium,
};

#[path = "retirement/lump_sum.rs"]
mod lump_sum;
#[path = "retirement/lump_sum_return.rs"]
mod lump_sum_return;
pub use lump_sum::{Form4972Election, Form4972Participant, LumpSumEvent, LumpSumMethod, NuaEvent};

#[path = "retirement/property_rollover.rs"]
mod property_rollover;
pub use property_rollover::PropertyRollover;

#[path = "retirement/child_annuity.rs"]
mod child_annuity;
pub use child_annuity::{ChildAnnuity, ChildAnnuityPayment};

#[path = "activity/qbi_aggregation.rs"]
mod qbi_aggregation;
#[path = "activity/qbi_aggregation_model.rs"]
mod qbi_aggregation_model;
pub use qbi_aggregation::{
    IncomeRecordEstimateInput, IncomeRecordsEstimateInput, QbiAggregationDerived,
    QbiAggregationEstimateInput, QbiAggregationFacts, QbiAggregationFactsEstimateInput,
    QbiAggregationFactsPage, QbiAggregationMemberChoice, QbiAggregationPage, QbiAggregationSource,
    QbiBusinessChanges, QbiBusinessDisclosure, QbiBusinessDisclosureEstimateInput,
    QbiBusinessDisclosurePage, QbiElectionHistory, QbiEntityEin, QbiIrsDisaggregationHistory,
    QbiSourceOrigin,
};

pub use section179::{
    K1Section179BasisIssue, K1Section179BasisSource, K1Section179CurrentSource,
    K1Section179RiskInput, k1_section179_current_source,
};

#[path = "core/household_records.rs"]
mod household_records;

#[path = "core/return_sections.rs"]
mod return_sections;

#[path = "core/income_records.rs"]
mod income_records;

#[path = "core/accepted_sections.rs"]
mod accepted_sections;

#[path = "core/return_records.rs"]
mod return_records;

#[path = "business/related_forms.rs"]
mod business_related_forms;

#[path = "federal/related_forms.rs"]
pub mod related_forms;
pub use related_forms::{
    ActivityRelatedForms, RelatedForm, RelatedFormChoice, RelatedFormSummary, RelatedFormView,
    RelatedForms, ReviewedRelatedForm,
};

#[path = "federal/document_center.rs"]
mod document_center;
#[path = "federal/document_followup_page.rs"]
mod document_followup_page;
pub use document_center::{
    CanceledDebtDocumentRow, CanceledDebtDocuments, CreditDocument, CreditDocuments,
    DocumentAnswer, DocumentCenter, DocumentQuestion, DocumentsYouMayHave, EducationDocumentAnswer,
    EducationDocumentQuestion, EstateAmtTreatmentAnswer, EstateQbiAllocationHistory,
    FarmingDocumentRow, FarmingDocuments, Form1095ADocument, Form1098Document, Form1098VliDocument,
    GamblingDocumentRow, GamblingDocuments, GovernmentPaymentDocumentRow,
    GovernmentPaymentDocuments, HsaDocumentAnswer, HsaDocumentQuestion,
    InjuredSpouseDocumentAnswer, InjuredSpouseDocumentQuestion, InterestDividendDocumentRow,
    InterestDividendDocuments, InvestmentSaleDocumentRow, InvestmentSaleDocuments, K1DocumentRow,
    K1Documents, K1InvestmentAnswer, K1InvestmentAnswerInput, K1QuestionChoices,
    MarketplaceDocuments, MortgageDocuments, OtherDocumentTypes, OtherIncomeDocumentRow,
    OtherIncomeDocuments, PaymentPartsAnswer, RetirementDocumentRow, RetirementDocuments,
    SelfEmploymentDocumentRow, SelfEmploymentDocuments, StudentLoanDocuments, VehicleLoanDocuments,
    WageDocumentRow, WageDocuments, WagesDocumentAnswer, WagesDocumentQuestion,
};
pub use document_center::{
    K1ActivityComponentRecords, K1ActivityGraphRecords, K1ActivityPropertyAssociation,
    K1ActivityQbiAllocation, K1ActivityQbiRecords, K1ActivitySaleAssociation,
    K1PartnerActivityPersonal, K1PartnerActivityRecords, K1ScorpActivityRecords,
};
pub use document_followup_page::{
    CreditEntityAnswer, CreditOriginAnswer, DocumentQuestionsLink, DocumentSituation,
    DocumentSituationAnswer, DocumentSituations, DocumentSituationsAnswered, DocumentStrictAnswer,
    DocumentStrictQuestion, DocumentStrictQuestionInitial, EducationOwnAnswer,
    EducationOwnQuestion, HsaOwnAnswer, HsaOwnQuestion, InjuredOwnAnswer, InjuredOwnQuestion,
    OwnAsked, StrictAcquiredProperty, StrictEsppAcquisition, StrictPartnerStart,
    StrictSavingsBondAnswers, StrictSavingsBondStudent, StrictScorpStart, StrictSocialPremiumUse,
    StrictTaxCreditEntity, StrictTaxCreditOrigin,
};

pub use document_center::{
    CanceledDebtDocumentView, CreditDocumentAnswer, CreditDocumentQuestion, CreditDocumentView,
    FarmingDocumentView, GamblingDocumentView, GovernmentPaymentDocumentView,
    InterestDividendDocumentView, InvestmentSaleDocumentView, K1DocumentView,
    MarketplaceDocumentView, MortgageDocumentView, MortgageLoanSeed, MortgageNewLoanSeed,
    MortgagePointTreatment, MortgagesSeed, OtherIncomeDocumentView, PartnershipAuditRow,
    RetirementDocumentView, SelfEmploymentDocumentView, StudentLoanAdditionalInterest,
    StudentLoanDocumentView, TopicDocumentViews, VehicleLoanDocumentView, WageDocumentView,
};
pub(crate) use federal_model::IncomeEdits;
pub use federal_model::{EducationDocumentView, StudentCreditView};

pub use federal_model::RetirementPrintedBoxes;

pub use federal_model::{
    SocialBenefitDescription, SocialBenefitDescriptionKind, SocialPrintedBoxes,
};

pub use document_center::VehicleRefundDocumentView;
pub use other_income::{VehicleInterestRecovery, VehicleRecoveryYear, VehicleRefundOrigin};

pub use trump_account::TrumpExcessDocumentView;

pub use social_sources::{PrintedSocialBenefitYearPage, SocialPriorReturnFacts};

pub use federal_model::{
    BusinessOvertimeAmount, BusinessOvertimeEntry, BusinessOvertimePage, BusinessTipsAmount,
    BusinessTipsEntry, BusinessTipsPage,
};

#[path = "income/form_1099_misc_paper.rs"]
mod form_1099_misc_paper;
pub use form_1099_misc_paper::{
    Form1099MiscPaper, Misc1099Source, Misc1099StateBoxes, Misc1099StateRow,
};
#[path = "investment/broker_document.rs"]
mod broker_document;
pub use broker_document::{
    BrokerAcquisitionDate, BrokerBasisAnswer, BrokerDocument, BrokerPaper, BrokerPersonalRecords,
    BrokerSaleAdjustment, BrokerSaleAnswers, BrokerSaleUse, ContractsPersonalRecords, Form1099B,
    Form1099B8949Checkbox, Form1099DA, Form1099DA8949Checkbox, SalePersonalRecords,
};

#[path = "investment/broker_page.rs"]
mod broker_page;

#[path = "investment/broker_calculation.rs"]
mod broker_calculation;

pub(crate) use federal_model::ssa_1099_model;
