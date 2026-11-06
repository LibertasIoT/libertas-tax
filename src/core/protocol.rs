pub use crate::navigation::{
    AllTopicsLink, CarriedItem, DocumentFacts, EstimateCarry, EstimateSoFar, HubTopics,
    NavigationAvailability, NavigationDestination, PersonCredits, ReviewItem, SaveWarning,
    TaxSection,
};

use crate::*;
use alloc::{string::String, vec::Vec};
use libertas::LibertasUser;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Tax return
/// Accepted pages are saved. Larger forms offer recovery of completed field edits on this device; small-page edits and unfinished typing are discarded on exit. Synthetic data only; no return is filed.
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroDecode,
    LibertasAvroEncode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
// No request of the tax interview is offered to the Hub LLM as a tool call:
// every page needs its editor, its cookie and the filer's confirmation.
#[libertas_llm_tool_call(false)]
// Complete editor payloads are owned inline; keep the wire model free of allocation-only wrappers.
#[allow(clippy::large_enum_variant)]
pub enum TaxInterviewProtocol {
    // Entry action, also offered on Review while an interview step remains.
    // Workflow PrevRequest exposes earlier-section navigation through Back.
    /// Start or resume
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    OpenInterview {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Your 2026 return
    #[libertas_response]
    #[libertas_workflow_request("SaveSetup")]
    BeginSetup {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    // Cache large forms by return identity, accepted-data revision, and navigation
    // context. A changed accepted page invalidates drafts; review-only Next does not.
    // Keeping review in the tuple prevents restoring an obsolete Submit destination.
    // The first section has no previous section. Native Back follows page provenance;
    // do not expose a PrevRequest that would reload this same editor.
    /// Your 2026 return
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSetup")]
    SaveSetup {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Your 2026 return
        #[libertas_calculation_request("EstimateSetup")]
        value: ReturnSetup,
    },
    /// About you
    #[libertas_response]
    #[libertas_workflow_request("SavePeople")]
    BeginPeople {
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// About you
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateYou")]
    SavePeople {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// About you
        #[libertas_calculation_request("EstimateYou")]
        you: Filer,
    },
    // Sent only on a joint return, after the filer's own page.
    /// About your spouse
    #[libertas_response]
    #[libertas_workflow_request("SaveSpouse")]
    BeginSpouse {
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// About your spouse
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSpouse")]
    SaveSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Existing annual rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// About your spouse
        #[libertas_calculation_request("EstimateSpouse")]
        spouse: SpouseAnswers,
    },
    // The deductions section page (unit M3-P9): the route and cash gifts.
    /// Deductions
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateDeductions")]
    SaveDeductions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available repayment answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_repayment")]
        #[libertas_calculation_request("EstimateDeductions")]
        allowed_repayment: Vec<i32>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Deductions
        #[libertas_calculation_request("EstimateDeductions")]
        value: Deductions,
        /// Saved business homes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_sites")]
        home_sites: Vec<RecordChoice>,
    },
    // The people page (M2 page 3), asked once on every return after the
    // filer pages (gate fixes batch A). A return that lists nobody yet opens
    // it through the workflow response.
    /// Children and people you support
    #[libertas_response]
    #[libertas_workflow_request("SaveChildren")]
    BeginChildren {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        days_alive: u32,
    },
    /// Children and people you support
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateDependents")]
    SaveChildren {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_lived")]
        #[libertas_calculation_request("EstimateDependents")]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.days_alive")]
        #[libertas_calculation_request("EstimateDependents")]
        days_alive: u32,
        /// Children and people you support
        #[libertas_calculation_request("EstimateDependents")]
        dependents: Dependents,
        /// Credits from the people you listed
        /// Worked out from the answers above, and updated as you change them. Each amount is the most the credit gives before the income limit; the estimate applies the limit.
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.credits")]
        /// ----
        /// Person
        credits: Vec<PersonCredits>,
    },
    /// Income documents
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateDocuments")]
    SaveIncomes {
        /// A newly added business has no saved intangible predecessor
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_intangible_predecessors")]
        initial_intangible_predecessors: Vec<crate::RecordChoice>,
        /// Empty applicability until this intangible's facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_intangible_choices")]
        initial_intangible_choices: Vec<i32>,
        /// Available source categories and their kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.source_choices")]
        #[libertas_calculation_request("EstimateDocuments")]
        source_choices: crate::IncomeSourceChoices,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Empty election choices until this activity has affected property
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_interest_limit_choices")]
        initial_interest_limit_choices: Vec<i32>,
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Businesses available for individual QBI aggregation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.qbi_sources")]
        qbi_sources: Vec<crate::QbiAggregationMemberChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Saved personal casualty properties
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.casualty_properties")]
        casualty_properties: Vec<RecordChoice>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        #[libertas_calculation_request(
            "EstimateDocuments,K1InvestmentOptions,BusinessRelatedOptions"
        )]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Option<Vec<u8>>,
        // Part of the draft identity, so a Wages draft never restores into
        // the Interest and dividends page.
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.topic")]
        #[libertas_calculation_request("EstimateDocuments")]
        topic: TaxSection,
        /// Allowed document kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_documents")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_documents: Vec<i32>,
        /// Allowed document owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_owners: Vec<i32>,
        /// Allowed community-property answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_community")]
        #[libertas_calculation_request("EstimateDocuments")]
        allowed_community: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::code_choices (FARM_CODES).
        #[libertas_size(max = 17)]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::limitation_choices (codes 1 through 10).
        #[libertas_size(max = 10)]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Expected W-2 tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Income documents
        #[libertas_calculation_request("EstimateDocuments")]
        documents: IncomeRecords,
        /// Accepted category documents; edit them in the Document center
        #[libertas_read_only]
        source_documents: Option<crate::TopicDocumentViews>,
        /// Your answers about accepted documents
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateDocuments")]
        /// ----
        /// Question
        followups: Vec<crate::DocumentQuestion>,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateDocuments")]
        /// ----
        /// Question
        questions: Vec<crate::DocumentStrictQuestion>,
        /// Do any of these apply to your documents?
        #[libertas_constrained_by("$.situation_answers")]
        #[libertas_calculation_request("EstimateDocuments")]
        situations: crate::DocumentSituations,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        disaster_choices: Vec<i32>,

        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
    },
    /// Wages
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateWagesAnswers")]
    SaveWagesAnswers {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted editor cookie
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted editor revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted editor page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Accepted editor previous
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Accepted editor review
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// Warnings
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        warnings: Vec<SaveWarning>,
        /// Progress
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_formatted_text]
        #[libertas_copy_from("$.progress")]
        progress: Option<Vec<u8>>,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Available wage scorporations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available allowed wage scorporations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Available allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Available allowed community
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_community")]
        allowed_community: Vec<i32>,
        /// Available allowed refund management
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Your W-2 answers
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateWagesAnswers")]
        wages: Vec<W2AnswerRow>,
        /// Other wage answers
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateWagesAnswers")]
        followups: Vec<crate::WagesDocumentQuestion>,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateWagesAnswers")]
        /// ----
        /// Question
        questions: Vec<crate::DocumentStrictQuestion>,
        /// Do any of these apply to your documents?
        #[libertas_constrained_by("$.situation_answers")]
        #[libertas_calculation_request("EstimateWagesAnswers")]
        situations: crate::DocumentSituations,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        disaster_choices: Vec<i32>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        people: Vec<PersonChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Other wage records
        #[libertas_calculation_request("EstimateWagesAnswers")]
        records: WagesRecords,
    },
    // The HSA topic of the adjustments hub (unit M3-P7). HsaOptions reads
    // only the accounts and rewrites the family split's allowed set; it never
    // reads the split, which EstimateHsa reads downstream.
    /// Health and medical savings accounts
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateHsa,HsaOptions")]
    SaveHsa {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateHsa,HsaOptions")]
        allowed_owners: Vec<i32>,
        /// Other-filer HSA eligibility applies on this married return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.other_coverage_allowed")]
        other_coverage_allowed: bool,
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_family_split")]
        #[libertas_calculation_request("EstimateHsa")]
        allowed_family_split: Vec<i32>,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// IRA distributions on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ira_sources")]
        #[libertas_calculation_request("EstimateHsa")]
        ira_sources: Vec<crate::HsaFundingSourceChoice>,
        /// Earned compensation on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.archer_income")]
        #[libertas_calculation_request("EstimateHsa")]
        archer_income: Vec<RecordChoice>,
        /// Health and medical savings accounts
        #[libertas_calculation_request("EstimateHsa")]
        value: HsaPage,
        /// Questions about your health-account distributions
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateHsa")]
        followups: Vec<crate::HsaDocumentQuestion>,
        /// Questions your health-account documents call for
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateHsa")]
        /// ----
        /// Question
        questions: Vec<crate::HsaOwnQuestion>,
    },
    // Child and dependent care while a person's care is not answered yet:
    // the page asks every row unanswered (BeginCare), then reopens by
    // EditRequest.
    /// Child and dependent care
    #[libertas_response]
    #[libertas_workflow_request("SaveCare")]
    BeginCare {
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_injured_source_facts: Vec<i32>,
        /// People already on this return
        #[libertas_hidden]
        #[libertas_read_only]
        people: Vec<PersonChoice>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Allowed filers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_owners: Vec<i32>,
        /// Filers eligible for monthly deemed income
        #[libertas_hidden]
        #[libertas_read_only]
        care_filers: Vec<i32>,
        /// Calendar months
        #[libertas_hidden]
        #[libertas_read_only]
        care_months: Vec<RecordChoice>,
        /// Businesses
        #[libertas_hidden]
        #[libertas_read_only]
        businesses: Vec<CareSourceChoice>,
        /// Farms
        #[libertas_hidden]
        #[libertas_read_only]
        farms: Vec<CareSourceChoice>,
        /// Partnership K-1s
        #[libertas_hidden]
        #[libertas_read_only]
        partnerships: Vec<CareSourceChoice>,
        /// Filing status from Your 2026 return
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_filing: Vec<i32>,
        /// Child and dependent care
        #[libertas_hidden]
        #[libertas_read_only]
        value: crate::CareSeed,
    },
    /// Child and dependent care
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateCare")]
    SaveCare {
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// People already on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,

        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Filers eligible for monthly deemed income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_filers")]
        care_filers: Vec<i32>,
        /// Calendar months
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.care_months")]
        care_months: Vec<RecordChoice>,
        /// Businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<CareSourceChoice>,
        /// Farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<CareSourceChoice>,
        /// Partnership K-1s
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.partnerships")]
        partnerships: Vec<CareSourceChoice>,
        /// Child and dependent care
        /// Filing status from Your 2026 return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_filing")]
        allowed_filing: Vec<i32>,
        #[libertas_calculation_request("EstimateCare")]
        #[libertas_copy_from("$.value")]
        value: CarePage,
    },
    // The self-employed health insurance and retirement plan topic of the
    // adjustments hub (unit M3-P8). Its rows are membership-only: the task
    // builds one row per business the return already has, and the client
    // edits only each row's health and retirement fields.
    /// Self-employed health insurance and retirement plan
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSelfEmployedBenefits")]
    SaveSelfEmployedBenefits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Estimate will appear after the current answers are calculated.
        /// [SetText]
        /// Estimate
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Interview progress will appear after the current answers are calculated.
        /// [SetText]
        /// Interview progress
        progress: Option<Vec<u8>>,
        /// Accepted Marketplace source statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Current businesses and entities
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_activities")]
        health_activities: Vec<RecordChoice>,
        /// Current people available to insure
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_people")]
        health_people: Vec<PersonChoice>,
        /// Existing wage statements for shareholder plan linkage
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.health_wages")]
        health_wages: Vec<RecordChoice>,
        /// Self-employed health insurance and retirement plan
        #[libertas_calculation_request("EstimateSelfEmployedBenefits")]
        value: SelfEmployedBenefits,
    },
    // The IRA contributions topic of the adjustments hub (unit M3-P8).
    /// IRA contributions
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateIraContributions")]
    SaveIraContributions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed contributors
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateIraContributions")]
        allowed_owners: Vec<i32>,
        /// IRA contributions
        #[libertas_calculation_request("EstimateIraContributions")]
        value: IraContributions,
    },
    // The educator expenses topic of the adjustments hub (unit M3-P8).
    /// Educator expenses
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateEducator")]
    SaveEducator {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed educators
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateEducator")]
        allowed_owners: Vec<i32>,
        /// Educator expenses
        #[libertas_calculation_request("EstimateEducator")]
        value: EducatorExpenses,
    },
    // The student loan interest topic of the adjustments hub (unit M3-P8).
    /// Student loan interest
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateStudentLoan")]
    SaveStudentLoan {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Student loan interest
        #[libertas_calculation_request("EstimateStudentLoan")]
        value: StudentLoanInterest,
    },
    // The Form 2106 employee business expenses topic of the adjustments hub.
    /// Reservist, performing artist and fee-basis official expenses
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateEmployeeExpenses")]
    SaveEmployeeExpenses {
        /// Accepted physical buildings available to improvement sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateEmployeeExpenses")]
        allowed_owners: Vec<i32>,
        /// Reservist, performing artist and fee-basis official expenses
        #[libertas_calculation_request("EstimateEmployeeExpenses")]
        value: EmployeeExpenses,
    },
    // The medical topic of the itemized hub (unit M3-P9).
    /// Medical and dental expenses
    /// On a community-property separate return, enter your share of personal expenses paid from community funds and the expenses you paid from your separate funds. Do not enter the household total again.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateMedical")]
    SaveMedical {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Medical and dental expenses
        #[libertas_calculation_request("EstimateMedical")]
        value: Medical,
    },
    // The taxes topic of the itemized hub (unit M3-P9).
    /// Taxes you paid
    /// On a community-property separate return, enter the amount attributable to this return under the community or separate source of the payment. Keep the full home-office costs where the page asks for them; the App allocates those costs once.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateTaxes")]
    SaveTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Taxes you paid
        #[libertas_calculation_request("EstimateTaxes")]
        value: Taxes,
    },
    // The investment interest and other topic of the itemized hub (unit M3-P9).
    /// Investment interest and other deductions
    /// On a community-property separate return, enter only the deduction attributable to this return. An amount already supplied by an income statement or another source page is not entered again.
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateOtherItemized")]
    SaveOtherItemized {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Investment borrowing sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.interest_sources")]
        interest_sources: Vec<crate::InvestmentSourceChoice>,
        /// Actual K-1 entities for a retained interest balance
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.investment_k1_sources")]
        investment_k1_sources: Vec<crate::RecordChoice>,
        /// Applicable retained-interest origin question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.investment_origin_choices")]
        investment_origin_choices: Vec<i32>,
        /// Investment interest and other deductions
        #[libertas_calculation_request("EstimateOtherItemized")]
        value: OtherItemized,
    },
    // Home mortgage interest while a loan's facts are not answered yet:
    // answered loans as saved and new ones unanswered (BeginMortgages).
    /// Home mortgage interest
    #[libertas_response]
    #[libertas_workflow_request("SaveMortgages")]
    BeginMortgages {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        mortgage_loans: Vec<RecordChoice>,
        /// Loans currently selected by rentals
        #[libertas_hidden]
        #[libertas_read_only]
        rental_mortgage_loans: Vec<i64>,
        /// Home mortgage interest
        #[libertas_hidden]
        #[libertas_read_only]
        value: crate::MortgagesSeed,
    },
    // The home mortgage topic of the itemized hub (unit M3-P10).
    /// Home mortgage interest
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateMortgages")]
    SaveMortgages {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_output_only]
        document_questions: Vec<crate::DocumentQuestionsLink>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.document_facts")]
        document_facts: DocumentFacts,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Loans currently selected by rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_mortgage_loans")]
        rental_mortgage_loans: Vec<i64>,
        /// Home mortgage interest
        #[libertas_calculation_request("EstimateMortgages")]
        #[libertas_copy_from("$.value")]
        value: Mortgages,
    },
    /// Business use of your homes
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateHomeOffices")]
    SaveHomeOffices {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved businesses and farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.home_activities")]
        home_activities: Vec<RecordChoice>,
        /// Saved casualty events
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.casualty_events")]
        casualty_events: Vec<RecordChoice>,
        /// Allowed home owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_owners")]
        allowed_home_owners: Vec<i32>,
        /// Business use of your homes
        #[libertas_calculation_request("EstimateHomeOffices")]
        value: HomeOffices,
    },
    // The charity carryovers topic of the itemized hub (unit M3-P10).
    /// Gifts of property and carryovers
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateGiftCarryovers")]
    SaveGiftCarryovers {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed captains
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateGiftCarryovers")]
        allowed_owners: Vec<i32>,
        /// Saved appraisals available to gift rows
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.appraisal_choices")]
        appraisal_choices: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Actual current K-1 sources for retained activity gifts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_k1s")]
        retained_charity_k1s: Vec<RecordChoice>,
        /// Actual retained-gift origin choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_charity_origins")]
        retained_charity_origins: Vec<i32>,
        /// Applicable retained-gift source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Gifts of property and carryovers
        #[libertas_calculation_request("EstimateGiftCarryovers")]
        value: GiftCarryovers,
    },
    // The education topic of the credits hub (unit M3-P11).
    /// Education credits
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateEducation")]
    SaveEducation {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        people: Vec<PersonChoice>,
        /// Questions about your education-account documents
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateEducation")]
        followups: Vec<crate::document_center::EducationDocumentQuestion>,
        /// Questions your education-account documents call for
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateEducation")]
        /// ----
        /// Question
        questions: Vec<crate::EducationOwnQuestion>,
        /// Education credits
        #[libertas_calculation_request("EstimateEducation")]
        value: crate::EducationDocumentView,
    },
    // The retirement savings topic of the credits hub (unit M3-P11).
    /// Retirement savings credit
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSaver")]
    SaveSaver {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Retirement savings credit
        #[libertas_calculation_request("EstimateSaver")]
        value: SaverCredit,
    },
    // The Form 1095-A topic of the credits hub (unit T2-27p).
    /// Health insurance marketplace
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateMarketplace")]
    SaveMarketplace {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// People supported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Current people and saved external enrollees
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_people")]
        ptc_people: Vec<RecordChoice>,
        /// Saved other tax families
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_families")]
        ptc_families: Vec<RecordChoice>,
        /// Saved policy sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Accepted W-2 code FF sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_w2")]
        ptc_w2: Vec<RecordChoice>,
        /// Employees on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_employees")]
        ptc_employees: Vec<RecordChoice>,
        /// Current people outside the calculated tax family
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_nonreturn_people")]
        ptc_nonreturn_people: Vec<RecordChoice>,
        /// Claimed dependents on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_claimed_dependents")]
        ptc_claimed_dependents: Vec<RecordChoice>,
        /// Available marriage calculation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_marriage")]
        allowed_marriage: Vec<i32>,
        /// Available separate-return answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_separate")]
        allowed_separate: Vec<i32>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Health insurance marketplace
        #[libertas_calculation_request("EstimateMarketplace")]
        value: MarketplaceFactsPage,
    },
    // The credits hub's last topic (unit M3-P12).
    /// More credits
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateMoreCredits")]
    SaveMoreCredits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_output_only]
        document_questions: Vec<crate::DocumentQuestionsLink>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_businesses")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.capital_sales")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        capital_sales: Vec<RecordChoice>,
        /// Estate or trust foreign-allocation sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estate_foreign_sources")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        estate_foreign_sources: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.foreign_situations")]
        #[libertas_calculation_request("EstimateMoreCredits")]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.clean_vehicle_properties")]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.entered_credits")]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.carryback_credits")]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_carry_filers")]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_credit_elections")]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_research_choices")]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_calculation_request("EstimateMoreCredits")]
        #[libertas_copy_from("$.value")]
        value: MoreCredits,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
        /// Accepted sources containing already reported bond income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.credit_income_sources")]
        credit_income_sources: Vec<crate::CreditIncomeSource>,
        /// Available credit owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_credit_owners")]
        allowed_credit_owners: Vec<i32>,
    },
    // The other taxes section's page (u5-credits): its calculated taxes
    // and the refusal kinds, one list.
    /// Other taxes
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateOtherTaxes")]
    SaveOtherTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        #[libertas_calculation_request("EstimateOtherTaxes")]
        allowed_owners: Vec<i32>,
        /// Allowed owners of a joint subsidized-home gift
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_gift_owners")]
        mortgage_gift_owners: Vec<i32>,
        /// Existing current-year Coverdell correction sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.coverdell_corrections")]
        #[libertas_calculation_request("EstimateOtherTaxes")]
        coverdell_corrections: Vec<RecordChoice>,
        /// Other taxes
        #[libertas_calculation_request("EstimateOtherTaxes")]
        value: OtherTaxes,
    },
    // The adjustments hub's last topic (unit u5-adjust).
    /// More adjustments
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateMoreAdjustments")]
    SaveMoreAdjustments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// More adjustments
        #[libertas_calculation_request("EstimateMoreAdjustments")]
        value: MoreAdjustments,
    },
    // The federal payments section page (unit M3-P13).
    /// Federal payments
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimatePayments")]
    SavePayments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_prior_returns")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farmer_next_year")]
        #[libertas_calculation_request("EstimatePayments")]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_calculation_request("EstimatePayments")]
        #[libertas_copy_from("$.value")]
        value: Payments,
    },
    // Form 2210 Schedule AI's income by period, after the payments page when
    // box C is chosen (unit M3-P14).
    /// Income by period
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateIncomeByPeriod")]
    SaveIncomeByPeriod {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Income by period
        #[libertas_calculation_request("EstimateIncomeByPeriod")]
        value: IncomeByPeriod,
    },
    // Form 8379, after the payments section while the checklist's injured
    // spouse item is ticked on a joint return (unit T3-8379).
    /// Injured spouse allocation (Form 8379)
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateInjuredSpouse")]
    SaveInjuredSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Injured spouse allocation (Form 8379)
        #[libertas_calculation_request("EstimateInjuredSpouse")]
        #[libertas_copy_from("$.value")]
        value: InjuredSpouse,
        /// Who managed the withholding from your documents
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateInjuredSpouse")]
        #[libertas_copy_from("$.followups")]
        followups: Vec<crate::InjuredSpouseDocumentQuestion>,
        /// Who managed the withholding from the documents that call for it
        #[libertas_read_only]
        #[libertas_calculation_request("EstimateInjuredSpouse")]
        #[libertas_copy_from("$.questions")]
        /// ----
        /// Question
        questions: Vec<crate::InjuredOwnQuestion>,
    },
    // Form 8615's parent facts (u5-losses), after the other taxes page while
    // Form 8615 applies to this return.
    /// Your parent's return (Form 8615)
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateParentTax")]
    SaveParentTax {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Your parent's return (Form 8615)
        #[libertas_calculation_request("EstimateParentTax")]
        value: ParentReturn,
    },
    // Beyond your documents, after the household pages: asked unanswered
    // until first saved (BeginChecklist), then reopened by EditRequest with
    // the answers; a hub's "What you told us" row reopens it.
    /// Beyond your documents
    #[libertas_response]
    #[libertas_workflow_request("SaveChecklist")]
    BeginChecklist {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// From your documents
        #[libertas_formatted_text]
        #[libertas_read_only]
        from_documents: Vec<u8>,
    },
    /// Beyond your documents
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateChecklist")]
    SaveChecklist {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// From your documents
        /// Your documents already bring these topics into your return. Tick a box only for what you did or paid.
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.from_documents")]
        from_documents: Vec<u8>,
        /// Beyond your documents
        #[libertas_calculation_request("EstimateChecklist")]
        value: ChecklistAnswers,
    },
    /// Situations people often miss
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateOftenMissed")]
    SaveOftenMissed {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Situations people often miss
        #[libertas_calculation_request("EstimateOftenMissed")]
        #[libertas_copy_from("$.value")]
        value: crate::federal_model::OftenMissedAnswers,
        /// Not listed? See all topics or Search
        #[libertas_read_only]
        #[libertas_copy_from("$.all_topics")]
        #[libertas_link("Navigate")]
        all_topics: AllTopicsLink,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.navigation")]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.suggested_destination")]
        suggested_destination: NavigationDestination,
    },
    /// Back
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    Back {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Previous page
        #[libertas_hidden]
        #[libertas_copy_from("$.previous")]
        previous: i32,
    },
    // Opened from a hub or the review: `from` is where a saved page returns,
    // and the popup starts at the service's suggestion.
    /// Go to
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    Navigate {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Opened from
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        from: i32,
        /// Page
        #[libertas_constrained_by("$.navigation.groups")]
        #[libertas_copy_from("$.suggested_destination")]
        section: NavigationDestination,
        /// Available choices
        #[libertas_hidden]
        #[libertas_copy_from("$.navigation")]
        navigation: NavigationAvailability,
    },
    // The link-only target of every hub and review row (M3-N1): it has no input,
    // so a tap on a row sends it at once with the tapped row's section. When
    // no row is bound, CopyFrom falls back to the service's suggestion.
    /// Open the suggestion
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_link_only]
    OpenSection {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Opened from
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        from: i32,
        /// Section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<ReviewItem>.section,$.suggested")]
        section: TaxSection,
        /// Answer field
        #[libertas_hidden]
        #[libertas_reveal_path]
        #[libertas_copy_from("^<ReviewItem>.field")]
        reveal_path: Option<String>,
    },
    // The refresh request of the hub's master-detail arrangement
    // (display.json): after a topic is saved in the trailing pane, the client
    // builds it from the hub as shown and the service answers with the hub as
    // it is now. It only navigates: it is admitted by identity, stores
    // nothing, and is never offered as an action.
    /// Reload the topics
    #[libertas_request]
    #[libertas_next_response("Hub,Problem,StoredReturnUnavailable")]
    #[libertas_exclude_ui]
    OpenHub {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
    },
    // The task offers this section's Continue until the whole walkthrough is
    // complete, then Review. Link-only topic and previous-section rows stay
    // available independently of Actions and the secondary Go to control.
    /// Topics
    #[libertas_response]
    #[libertas_ai_navigation_prompt("TAX_NAVIGATION_PROMPT")]
    #[libertas_next_request(
        "ContinueIncome,ContinueAdjustments,ContinueItemized,ContinueCredits,ReviewAccepted,Navigate,OpenSection,OpenCropInsuranceFacts"
    )]
    #[libertas_constrained_by("$.allowed_actions")]
    Hub {
        /// Crop-insurance timing and missing statement details
        /// Printed proceeds are counted automatically. Open a farm here to choose income timing or complete an elected deferral.
        #[libertas_read_only]
        crop_insurance: Vec<crate::farm_insurance_paper::CropInsuranceActivityLink>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateSoFar>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// What you told us
        /// The checklist's answer for this section; a tap opens the checklist.
        #[libertas_read_only]
        told: ReviewItem,
        /// Section topics
        #[libertas_read_only]
        topics: HubTopics,
        /// Previous section
        /// Open the preceding section without submitting an open editor.
        #[libertas_read_only]
        previous_section: ReviewItem,
        /// Available actions
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_actions: Vec<i32>,
        /// Suggested next step
        #[libertas_hidden]
        #[libertas_read_only]
        suggested: TaxSection,
        /// Available steps
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_sections: Vec<i32>,
        /// Continue or review
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("Actions")]
        primary_actions: Vec<i32>,
        /// Go to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("GoTo")]
        go_to_actions: Vec<i32>,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },
    // Finish is offered only when nothing remains (M3-P16): the response's
    // allowed actions hold StartFinish only with an empty remaining list.
    // Navigate and link-only OpenSection are always allowed. OpenInterview
    // is offered only when it opens an unfinished interview step. The review
    // remains visible when Go to is its sole menu action (M3-N1).
    /// Review your federal estimate
    #[libertas_response]
    #[libertas_ai_navigation_prompt("TAX_NAVIGATION_PROMPT")]
    #[libertas_next_request(
        "Navigate,OpenSection,StartFinish,OpenInterview,ViewReviewedRelatedForm"
    )]
    #[libertas_constrained_by("$.allowed_actions")]
    Review {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Estimate
        #[libertas_formatted_text]
        #[libertas_read_only]
        estimate: Vec<u8>,
        /// Remaining items
        /// Fix these before finishing. Tap an item to open its page.
        #[libertas_suppress_label]
        #[libertas_read_only]
        /// ----
        /// Item
        remaining_items: Vec<ReviewItem>,
        /// Estimate notes you can update
        /// These assumptions and items left out do not prevent finishing. Tap a note to review its answers.
        #[libertas_suppress_label]
        #[libertas_read_only]
        /// ----
        /// Note
        correction_notes: Vec<ReviewItem>,
        /// Your answers
        /// Tap a section to open it.
        #[libertas_suppress_label]
        /// ----
        /// Item
        answers: Vec<ReviewItem>,
        /// Related forms by activity
        #[libertas_read_only]
        related_forms: Vec<crate::related_forms::ActivityRelatedForms>,
        /// Federal estimate and coverage
        result: FederalEstimate,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Suggested next step
        #[libertas_hidden]
        #[libertas_read_only]
        suggested: TaxSection,
        /// Available sections
        #[libertas_hidden]
        allowed_sections: Vec<i32>,
        /// Available actions
        #[libertas_hidden]
        allowed_actions: Vec<i32>,
        /// Resume or finish
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("Actions")]
        primary_actions: Vec<i32>,
        /// Go to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_action_control("GoTo")]
        go_to_actions: Vec<i32>,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },
    /// Finish review
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    StartFinish {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Finish prototype review
    #[libertas_response]
    #[libertas_workflow_request("Finish")]
    BeginFinish {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Your refund
        #[libertas_read_only]
        refund: crate::RefundChoice,
    },
    /// Finish prototype review
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    Finish {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Your refund
        #[libertas_copy_from("$.refund")]
        refund: crate::RefundChoice,
    },
    /// Prototype review completed
    #[libertas_response]
    #[libertas_next_request("ReviewAccepted")]
    Finished {
        /// Completion status
        #[libertas_formatted_text]
        #[libertas_read_only]
        message: Vec<u8>,
        /// Federal estimate
        result: FederalEstimate,
    },
    /// Please review your answer
    #[libertas_response]
    #[libertas_error]
    Problem {
        /// What needs attention
        #[libertas_formatted_text]
        #[libertas_read_only]
        message: Vec<u8>,
        /// Native source field needing attention
        /// [NullText]
        /// No source field has been identified
        /// [SetText]
        /// Source field needing attention
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_error_field_path]
        field: Option<String>,
    },
    /// Applicable property answers for this document
    #[libertas_request]
    #[libertas_next_response("DocumentPropertyChoices")]
    DocumentPropertyOptions {
        /// Available source categories and their kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.source_choices")]
        source_choices: crate::IncomeSourceChoices,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Empty election choices until this activity has affected property
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_interest_limit_choices")]
        initial_interest_limit_choices: Vec<i32>,
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farms")]
        farms: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Saved personal casualty properties
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.casualty_properties")]
        casualty_properties: Vec<RecordChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.children")]
        children: Vec<crate::ChildChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Document
        #[libertas_copy_from("@.document")]
        document: IncomeRecord,
    },
    /// Applicable property answers for the bound row
    #[libertas_response]
    DocumentPropertyChoices {
        /// Allowed community-property answers
        #[libertas_copy_to("@.allowed_community")]
        allowed_community: Vec<i32>,
    },
    /// Calculate document preview
    #[libertas_request]
    #[libertas_next_response("IncomePreview,Problem")]
    PreviewDocument {
        /// Available source categories and their kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.source_choices")]
        source_choices: crate::IncomeSourceChoices,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Empty election choices until this activity has affected property
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_interest_limit_choices")]
        initial_interest_limit_choices: Vec<i32>,
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Accepted record identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Whose document
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Community or separate
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.community")]
        community: CommunityShare,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Saved personal casualty properties
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.casualty_properties")]
        casualty_properties: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("^<SaveIncomes>.children")]
        children: Vec<crate::ChildChoice>,
        /// Document
        #[libertas_copy_from("@.document")]
        document: IncomeRecord,
    },
    /// Document preview
    #[libertas_response]
    IncomePreview {
        /// Calculated carryover bounds from the edited source values
        #[libertas_copy_to("@.investment_caps")]
        investment_caps: Option<crate::InvestmentCapitalCap>,
        /// Document amount before return-level calculations
        #[libertas_money("USD", 2)]
        #[libertas_copy_to("@.preview")]
        preview: Option<i64>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSetup {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your 2026 return
        #[libertas_copy_from("@.value")]
        value: ReturnSetup,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateYou {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.professional_election,&.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Current editor's existing rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_professional_elections,&.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// About you
        #[libertas_copy_from("@.you")]
        you: Filer,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing annual rental election, for initializing newly chosen qualification facts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.professional_election,&.professional_election")]
        professional_election: crate::ProfessionalRentalElection,
        /// Current editor's existing rental election answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_professional_elections,&.allowed_professional_elections")]
        allowed_professional_elections: Vec<i32>,
        /// About your spouse
        #[libertas_copy_from("@.spouse")]
        spouse: SpouseAnswers,
    },
    /// Ownership choices for the selected documents
    #[libertas_request]
    #[libertas_next_response("DocumentOwnershipChoices,Problem")]
    DocumentOwnershipOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Person whose documents are being edited
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.person")]
        person: i64,
        /// Selected documents
        #[libertas_copy_from("@.documents")]
        documents: Vec<crate::DocumentOwnerSelection>,
    },
    /// Applicable ownership and share choices
    #[libertas_response]
    DocumentOwnershipChoices {
        /// Choices in the current document order
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.documents")]
        documents: Vec<crate::DocumentOwnershipDerived>,
    },
    /// Calculate the running wage estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateWagesAnswers {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current businesses
        #[libertas_copy_from("@.businesses")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        businesses: Vec<RecordChoice>,
        /// Current wage scorporations
        #[libertas_copy_from("@.wage_scorporations")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        wage_scorporations: Vec<RecordChoice>,
        /// Current allowed wage scorporations
        #[libertas_copy_from("@.allowed_wage_scorporations")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_wage_scorporations: Vec<i32>,
        /// Current allowed owners
        #[libertas_copy_from("@.allowed_owners")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_owners: Vec<i32>,
        /// Current allowed community
        #[libertas_copy_from("@.allowed_community")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_community: Vec<i32>,
        /// Current allowed refund management
        #[libertas_copy_from("@.allowed_refund_management")]
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        allowed_refund_management: Vec<i32>,
        /// Current wages
        #[libertas_copy_from("@.wages")]
        wages: Vec<W2AnswerRow>,
        /// Current questions
        #[libertas_copy_from("@.followups")]
        followups: Vec<crate::WagesDocumentQuestion>,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_copy_from("@.questions")]
        /// ----
        /// Question
        questions: Vec<crate::DocumentStrictQuestion>,
        /// Do any of these apply to your documents?
        #[libertas_constrained_by("$.situation_answers")]
        #[libertas_copy_from("@.situations")]
        situations: crate::DocumentSituations,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.allowed_situations")]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.situation_answers")]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.situation_documents")]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.disaster_choices")]
        disaster_choices: Vec<i32>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.people")]
        people: Vec<PersonChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Current records
        #[libertas_copy_from("@.records")]
        records: WagesRecords,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("IncomeRunningEstimate,Problem")]
    EstimateDocuments {
        /// Current editor's accepted return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.cookie")]
        cookie: String,
        /// Current editor's accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.revision")]
        #[libertas_calculation_request("K1InvestmentOptions")]
        revision: i64,
        /// Available source categories and their kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.source_choices")]
        source_choices: crate::IncomeSourceChoices,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Empty election choices until this activity has affected property
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.initial_interest_limit_choices")]
        initial_interest_limit_choices: Vec<i32>,
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.topic")]
        topic: TaxSection,
        /// Allowed document kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_documents")]
        allowed_documents: Vec<i32>,
        /// Allowed document owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Allowed community-property answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_community")]
        allowed_community: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.foreign_countries")]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.interest_statements")]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Saved personal casualty properties
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.casualty_properties")]
        casualty_properties: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farm_codes")]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farm_limitations")]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farm_sources")]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.basis_assets")]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farm_method_changes")]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.children")]
        children: Vec<crate::ChildChoice>,
        /// Income documents
        #[libertas_copy_from("@.documents")]
        documents: crate::IncomeRecordsEstimateInput,
        /// Your answers about accepted documents
        #[libertas_copy_from("@.followups")]
        followups: Vec<crate::DocumentQuestion>,
        /// Questions about your documents
        #[libertas_copy_from("@.questions")]
        questions: Vec<crate::DocumentStrictQuestion>,
        /// Situations that apply to your documents
        #[libertas_copy_from("@.situations")]
        situations: crate::DocumentSituations,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.allowed_situations")]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.situation_answers")]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.situation_documents")]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.disaster_choices")]
        disaster_choices: Vec<i32>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("DependentsRunningEstimate,Problem")]
    EstimateDependents {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed residence answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_lived,&.allowed_lived")]
        allowed_lived: Vec<i32>,
        /// Days you were alive in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.days_alive,&.days_alive")]
        days_alive: u32,
        /// Children and people you support
        #[libertas_copy_from("@.dependents")]
        dependents: Dependents,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    #[libertas_calculation_root("HsaOptions")]
    EstimateHsa {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        // The page's accounts carry HsaOptions wherever they appear, so this
        // request's copy of the owners is its input too.
        #[libertas_calculation_request("HsaOptions")]
        allowed_owners: Vec<i32>,
        /// Other-filer HSA eligibility applies on this married return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.other_coverage_allowed")]
        other_coverage_allowed: bool,
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_family_split")]
        allowed_family_split: Vec<i32>,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// IRA distributions on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ira_sources")]
        ira_sources: Vec<crate::HsaFundingSourceChoice>,
        /// Earned compensation on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.archer_income")]
        archer_income: Vec<RecordChoice>,
        /// Health and medical savings accounts
        #[libertas_copy_from("@.value")]
        value: HsaPage,
        /// Questions about your health-account distributions
        #[libertas_copy_from("@.followups")]
        followups: Vec<crate::HsaDocumentQuestion>,
        /// Questions your health-account documents call for
        #[libertas_copy_from("@.questions")]
        questions: Vec<crate::HsaOwnQuestion>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSelfEmployedBenefits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted Marketplace source statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Current businesses and entities
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.health_activities")]
        health_activities: Vec<RecordChoice>,
        /// Current people available to insure
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.health_people")]
        health_people: Vec<PersonChoice>,
        /// Existing wage statements for shareholder plan linkage
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.health_wages")]
        health_wages: Vec<RecordChoice>,
        /// Self-employed health insurance and retirement plan
        #[libertas_copy_from("@.value")]
        value: SelfEmployedBenefits,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateIraContributions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed contributors
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// IRA contributions
        #[libertas_copy_from("@.value")]
        value: IraContributions,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEducator {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed educators
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Educator expenses
        #[libertas_copy_from("@.value")]
        value: EducatorExpenses,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateStudentLoan {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Student loan interest
        #[libertas_copy_from("@.value")]
        value: StudentLoanInterest,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEmployeeExpenses {
        /// Accepted physical buildings available to improvement sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Reservist, performing artist and fee-basis official expenses
        #[libertas_copy_from("@.value")]
        value: EmployeeExpenses,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateDeductions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available repayment answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_repayment")]
        allowed_repayment: Vec<i32>,
        /// Deductions
        #[libertas_copy_from("@.value")]
        value: Deductions,
        /// Saved business homes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.home_sites")]
        home_sites: Vec<RecordChoice>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMedical {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Medical and dental expenses
        #[libertas_copy_from("@.value")]
        value: Medical,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Taxes you paid
        #[libertas_copy_from("@.value")]
        value: Taxes,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateOtherItemized {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed deduction owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Investment borrowing sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.interest_sources")]
        interest_sources: Vec<crate::InvestmentSourceChoice>,
        /// Actual K-1 entities for a retained interest balance
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.investment_k1_sources")]
        investment_k1_sources: Vec<crate::RecordChoice>,
        /// Applicable retained-interest origin question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.investment_origin_choices")]
        investment_origin_choices: Vec<i32>,
        /// Investment interest and other deductions
        #[libertas_copy_from("@.value")]
        value: OtherItemized,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMortgages {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Loans currently selected by rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.rental_mortgage_loans")]
        rental_mortgage_loans: Vec<i64>,
        /// Home mortgage interest
        #[libertas_copy_from("@.value")]
        value: Mortgages,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateHomeOffices {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Saved businesses and farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.home_activities")]
        home_activities: Vec<RecordChoice>,
        /// Saved casualty events
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.casualty_events")]
        casualty_events: Vec<RecordChoice>,
        /// Allowed home owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_home_owners")]
        allowed_home_owners: Vec<i32>,
        /// Business use of your homes
        #[libertas_copy_from("@.value")]
        value: HomeOffices,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateGiftCarryovers {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed captains
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Actual current K-1 sources for retained activity gifts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.retained_charity_k1s")]
        retained_charity_k1s: Vec<RecordChoice>,
        /// Actual retained-gift origin choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.retained_charity_origins")]
        retained_charity_origins: Vec<i32>,
        /// Applicable retained-gift source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Gifts of property and carryovers
        #[libertas_copy_from("@.value")]
        value: GiftCarryovers,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateCare {
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// People already on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.people")]
        people: Vec<PersonChoice>,
        /// Filing status from Your 2026 return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_filing")]
        allowed_filing: Vec<i32>,

        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Filers eligible for monthly deemed income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.care_filers")]
        care_filers: Vec<i32>,
        /// Calendar months
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.care_months")]
        care_months: Vec<RecordChoice>,
        /// Businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.businesses")]
        businesses: Vec<CareSourceChoice>,
        /// Farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farms")]
        farms: Vec<CareSourceChoice>,
        /// Partnership K-1s
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.partnerships")]
        partnerships: Vec<CareSourceChoice>,
        /// Child and dependent care
        #[libertas_copy_from("@.value")]
        value: CarePage,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateEducation {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.people")]
        people: Vec<PersonChoice>,
        /// Questions about your education-account documents
        #[libertas_copy_from("@.followups")]
        followups: Vec<crate::document_center::EducationDocumentQuestion>,
        /// Questions your education-account documents call for
        #[libertas_copy_from("@.questions")]
        questions: Vec<crate::EducationOwnQuestion>,
        /// Education credits
        #[libertas_copy_from("@.value")]
        value: crate::EducationDocumentView,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSaver {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Retirement savings credit
        #[libertas_copy_from("@.value")]
        value: SaverCredit,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMarketplace {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// People supported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.people")]
        people: Vec<PersonChoice>,
        /// Current people and saved external enrollees
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_people")]
        ptc_people: Vec<RecordChoice>,
        /// Saved other tax families
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_families")]
        ptc_families: Vec<RecordChoice>,
        /// Saved policy sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Accepted W-2 code FF sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_w2")]
        ptc_w2: Vec<RecordChoice>,
        /// Employees on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_employees")]
        ptc_employees: Vec<RecordChoice>,
        /// Current people outside the calculated tax family
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_nonreturn_people")]
        ptc_nonreturn_people: Vec<RecordChoice>,
        /// Claimed dependents on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.ptc_claimed_dependents")]
        ptc_claimed_dependents: Vec<RecordChoice>,
        /// Available marriage calculation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_marriage")]
        allowed_marriage: Vec<i32>,
        /// Available separate-return answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_separate")]
        allowed_separate: Vec<i32>,
        /// Health insurance marketplace
        #[libertas_copy_from("@.value")]
        value: MarketplaceFactsPage,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMoreCredits {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners,&.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.foreign_businesses,&.foreign_businesses")]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.capital_sales,&.capital_sales")]
        capital_sales: Vec<RecordChoice>,
        /// Estate or trust foreign-allocation sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.estate_foreign_sources,&.estate_foreign_sources")]
        estate_foreign_sources: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.foreign_situations,&.foreign_situations")]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.mortgage_loans,&.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.businesses,&.businesses")]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.farms,&.farms")]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.clean_vehicle_properties,&.clean_vehicle_properties")]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.rentals,&.rentals")]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.research_businesses,&.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.entered_credits,&.entered_credits")]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.carryback_credits,&.carryback_credits")]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_carry_filers,&.allowed_carry_filers")]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.initial_credit_elections,&.initial_credit_elections")]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.initial_research_choices,&.initial_research_choices")]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_copy_from("@.value")]
        value: MoreCredits,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.initial_carbon_oxide_origins,&.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from(
            "@.initial_renewable_facility_origins,&.initial_renewable_facility_origins"
        )]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
        /// Accepted sources containing already reported bond income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.credit_income_sources,&.credit_income_sources")]
        credit_income_sources: Vec<crate::CreditIncomeSource>,
        /// Available credit owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_credit_owners,&.allowed_credit_owners")]
        allowed_credit_owners: Vec<i32>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateOtherTaxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Allowed owners of a joint subsidized-home gift
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.mortgage_gift_owners")]
        mortgage_gift_owners: Vec<i32>,
        /// Existing current-year Coverdell correction sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.coverdell_corrections")]
        coverdell_corrections: Vec<RecordChoice>,
        /// Other taxes
        #[libertas_copy_from("@.value")]
        value: OtherTaxes,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateMoreAdjustments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Allowed owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// More adjustments
        #[libertas_copy_from("@.value")]
        value: MoreAdjustments,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("PaymentsEstimate,Problem")]
    EstimatePayments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_prior_returns,&.allowed_prior_returns")]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_farmer,&.allowed_farmer")]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_farmer_next_year,&.allowed_farmer_next_year")]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_refund_management,&.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_copy_from("@.value")]
        value: crate::PaymentsEstimateInput,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateIncomeByPeriod {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Income by period
        #[libertas_copy_from("@.value")]
        value: IncomeByPeriod,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateInjuredSpouse {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Injured spouse allocation (Form 8379)
        #[libertas_copy_from("@.value,&.value")]
        value: InjuredSpouse,
        /// Who managed the withholding from your documents
        #[libertas_copy_from("@.followups,&.followups")]
        followups: Vec<crate::InjuredSpouseDocumentQuestion>,
        /// Who managed the withholding from the documents that call for it
        #[libertas_copy_from("@.questions,&.questions")]
        questions: Vec<crate::InjuredOwnQuestion>,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateParentTax {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your parent's return (Form 8615)
        #[libertas_copy_from("@.value")]
        value: ParentReturn,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateChecklist {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Beyond your documents
        #[libertas_copy_from("@.value")]
        value: ChecklistAnswers,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateOftenMissed {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Situations people often miss
        #[libertas_copy_from("@.value")]
        value: crate::federal_model::OftenMissedAnswers,
    },
    /// Running estimate
    #[libertas_response]
    RunningEstimate {
        /// Estimate
        #[libertas_read_only]
        #[libertas_copy_to("@.estimate")]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_copy_to("@.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("@.progress")]
        progress: Vec<u8>,
    },

    /// Calculate the successful Marketplace appeal payment deadline
    #[libertas_request]
    #[libertas_next_response("MarketplaceAppealDeadline,Problem")]
    CalculateMarketplaceAppeal {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Appeal decision date
        #[libertas_date_only]
        #[libertas_copy_from("@.determined")]
        determined: u32,
    },
    /// Successful Marketplace appeal payment deadline
    #[libertas_response]
    MarketplaceAppealDeadline {
        /// Applicable payment deadline
        #[libertas_date_only]
        #[libertas_read_only]
        #[libertas_copy_to("@.required_deadline")]
        required_deadline: u32,
    },
    /// Calculate the family split choices
    #[libertas_request]
    #[libertas_next_response("HsaFamilyOptions,Problem")]
    HsaOptions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual failed cash deposits into HSA accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.hsa_failed_deposits")]
        hsa_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual failed cash deposits into ARCHER accounts
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.archer_failed_deposits")]
        archer_failed_deposits: Vec<crate::HealthFailedDepositChoice>,
        /// Actual accepted HSA corrective statements used by account-row constraints
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.hsa_return_documents")]
        hsa_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Actual accepted Archer corrective statements
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.archer_return_documents")]
        archer_return_documents: Vec<crate::HealthReturnDocumentChoice>,
        /// Allowed account owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("@.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Accounts
        #[libertas_copy_from("@.value.accounts")]
        accounts: Vec<HsaAccount>,
        /// Other filer's actual eligibility context
        /// [NullText]
        /// Not entered here: it is asked after the accounts are saved, when this return needs it
        /// [SetText]
        /// The other filer's HSA eligibility is
        #[libertas_copy_from("@.value.other_coverage")]
        #[libertas_default($null)]
        other_coverage: Option<crate::HsaEligibilityPage>,
        /// Contribution-limit method
        #[libertas_copy_from("@.value.family_method")]
        family_method: crate::HsaAllocationMethod,
    },
    /// Family split choices
    #[libertas_response]
    HsaFamilyOptions {
        /// Allowed family contribution splits
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_family_split")]
        allowed_family_split: Vec<i32>,
    },
    /// Open tax return
    #[libertas_request]
    #[libertas_default_request]
    #[libertas_next_response("Actions,Problem,StoredReturnUnavailable")]
    Enter {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Tax interview actions
    #[libertas_response]
    #[libertas_next_request("OpenInterview")]
    #[libertas_constrained_by("$.allowed_actions")]
    Actions {
        /// Available actions
        #[libertas_hidden]
        allowed_actions: Vec<i32>,
    },
    /// Review
    /// Review accepted answers and the estimate.
    #[libertas_request]
    #[libertas_next_response("Review,Problem,StoredReturnUnavailable")]
    ReviewAccepted {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
    },
    /// Applicable originating-entity question for retained investment interest
    #[libertas_request]
    #[libertas_next_response("InvestmentOriginChoices,Problem")]
    InvestmentOriginOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Existing deduction destination
        #[libertas_copy_from("@.source")]
        source: i64,
        /// Retained regular balance
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.carry")]
        carry: i64,
        /// Retained AMT balance
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.amt_carry")]
        amt_carry: i64,
    },
    /// Applicable originating-entity choices
    #[libertas_response]
    InvestmentOriginChoices {
        /// Allowed originating-entity question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_origin_choices")]
        allowed_origin_choices: Vec<i32>,
    },
    /// Applicable choices for this credit
    #[libertas_request]
    #[libertas_next_response("BusinessCreditElectionChoices,Problem")]
    BusinessCreditElectionOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Credit
        #[libertas_copy_from("@.credit")]
        credit: crate::BusinessCredit,
    },
    /// Applicable credit choices
    #[libertas_response]
    BusinessCreditElectionChoices {
        /// Allowed choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_elections")]
        allowed_elections: Vec<i32>,
        /// Applicable carbon-capture source detail
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_carbon_oxide_origins")]
        allowed_carbon_oxide_origins: Vec<i32>,
        /// Applicable renewable-facility source detail
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_renewable_facility_origins")]
        allowed_renewable_facility_origins: Vec<i32>,
    },
    /// Applicable choices for this credit
    #[libertas_request]
    #[libertas_next_response("ResearchCreditSourceChoices,Problem")]
    ResearchCreditSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Credit
        #[libertas_copy_from("@.credit")]
        credit: crate::BusinessCredit,
    },
    /// Applicable credit choices
    #[libertas_response]
    ResearchCreditSourceChoices {
        /// Allowed choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_research_choices")]
        allowed_research_choices: Vec<i32>,
    },
    // A 1099-R row's treatment choices from its printed box 7a codes and
    // box 7b (retirement.rs DistributionCodes::leads_to), bound to the row:
    // box7 and ira invoke it, and the reply writes the row's hidden set.
    /// Refresh automatic investment-credit confirmation choices
    #[libertas_request]
    #[libertas_next_response("K1InvestmentChoices,Problem")]
    K1InvestmentOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.cookie,^<EstimateDocuments>.cookie")]
        cookie: String,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.revision,^<EstimateDocuments>.revision")]
        revision: i64,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.key")]
        key: String,
        #[libertas_copy_from(
            "@.answer.K1InvestmentRecords.properties,@.answer.EstateInvestmentRecords.properties"
        )]
        properties: Vec<crate::K1InvestmentAnswerInput>,
    },
    /// Automatic investment-credit confirmation choices
    #[libertas_response]
    K1InvestmentChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to(
            "@.answer.K1InvestmentRecords.allowed_choices,@.answer.EstateInvestmentRecords.allowed_choices"
        )]
        choices: Vec<i32>,
    },
    /// Continue
    /// Leave income and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    ContinueIncome {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave adjustments and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    ContinueAdjustments {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave itemized deductions and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    ContinueItemized {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Continue
    /// Leave credits and open the next section of the walkthrough.
    #[libertas_request]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    ContinueCredits {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
    },
    /// Applicable retained regular-loss source question
    #[libertas_request]
    #[libertas_next_response("RetainedLossSourceChoices,Problem")]
    RetainedLossSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// This loss year's retained regular balance
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.balance")]
        balance: i64,
        /// Accepted research-source applicability
        #[libertas_hidden]
        #[libertas_copy_from(
            "^<SaveIncomes>.retained_research_choices,^<SaveGiftCarryovers>.retained_research_choices,^<DocumentPropertyOptions>.retained_research_choices,^<PreviewDocument>.retained_research_choices,^<EstimateDocuments>.retained_research_choices,^<EstimateGiftCarryovers>.retained_research_choices"
        )]
        available: Vec<i32>,
    },
    /// Applicable retained regular-loss choices
    #[libertas_response]
    RetainedLossSourceChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_research_parts")]
        allowed_research_parts: Vec<i32>,
    },
    /// Applicable retained regular-capital source question
    #[libertas_request]
    #[libertas_next_response("RetainedCapitalSourceChoices,Problem")]
    RetainedCapitalSourceOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        // Match the retained source amount's bounds.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.agi")]
        agi: i64,
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.deductions")]
        deductions: i64,
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 1200000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.senior_deduction")]
        senior_deduction: i64,
        // Match the retained source amount's bounds.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.short")]
        short: i64,
        // Match the retained source amount's bounds.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.long")]
        long: i64,
        // Match the retained source amount's bounds.
        #[libertas_number(min = 0, max = 300000)]
        #[libertas_money("USD", 2)]
        #[libertas_copy_from("@.deducted_loss")]
        deducted_loss: i64,
        #[libertas_copy_from("@.ordinary_history")]
        ordinary_history: bool,
        /// Accepted research-source applicability
        #[libertas_hidden]
        #[libertas_copy_from(
            "^<SaveIncomes>.retained_research_choices,^<DocumentPropertyOptions>.retained_research_choices,^<PreviewDocument>.retained_research_choices,^<EstimateDocuments>.retained_research_choices"
        )]
        available: Vec<i32>,
    },
    /// Applicable retained regular-capital choices
    #[libertas_response]
    RetainedCapitalSourceChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.allowed_research_parts")]
        allowed_research_parts: Vec<i32>,
    },
    /// Applicable retained charitable-gift appreciation questions
    #[libertas_request]
    #[libertas_next_response("RetainedCharityAppreciationChoices,Problem")]
    RetainedCharityAppreciationOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.basis_election")]
        election: bool,
        #[libertas_copy_from("@.carryovers")]
        records: Vec<crate::CharityAppreciationFacts>,
        #[libertas_hidden]
        #[libertas_copy_from(
            "^<SaveGiftCarryovers>.retained_research_choices,^<EstimateGiftCarryovers>.retained_research_choices"
        )]
        available: Vec<i32>,
    },
    /// Derived appreciation applicability for the same retained rows
    #[libertas_response]
    RetainedCharityAppreciationChoices {
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.carryovers")]
        records: Vec<crate::CharityAppreciationChoices>,
    },
    /// Running estimate and the credits from the people you listed
    #[libertas_response]
    DependentsRunningEstimate {
        /// Estimate
        #[libertas_read_only]
        #[libertas_copy_to("@.estimate")]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_copy_to("@.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("@.progress")]
        progress: Vec<u8>,
        /// Credits from the people you listed
        #[libertas_read_only]
        #[libertas_copy_to("@.credits")]
        /// ----
        /// Person
        credits: Vec<PersonCredits>,
    },
    /// Income estimate and current aggregation choices
    #[libertas_response]
    IncomeRunningEstimate {
        /// Estimate
        #[libertas_read_only]
        #[libertas_copy_to("@.estimate")]
        estimate: Option<EstimateCarry>,
        /// What saving these edits would remove or reopen
        #[libertas_read_only]
        #[libertas_copy_to("@.warnings")]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("@.progress")]
        progress: Vec<u8>,
        /// Current group members and distinct business counts
        #[libertas_read_only]
        #[libertas_copy_to("@.documents.qbi_aggregations")]
        groups: Vec<crate::QbiAggregationDerived>,
        /// Businesses available for individual QBI aggregation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.qbi_sources")]
        sources: Vec<crate::QbiAggregationMemberChoice>,
        /// Actual native cash-source applicability in the current document order
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("@.documents.documents")]
        refund_management: Vec<crate::IncomeRefundManagementDerived>,
    },

    /// Questions about your documents
    // Opens before a topic page whose documents ask something not answered
    // yet: the box questions per paper and the page's situations question.
    #[libertas_response]
    #[libertas_workflow_request("SaveDocumentQuestions")]
    BeginDocumentQuestions {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        topic: TaxSection,
        /// Questions about your documents
        #[libertas_read_only]
        questions: Vec<crate::DocumentStrictQuestionInitial>,
        /// Your saved answer to the situations question
        #[libertas_read_only]
        situations: Vec<crate::DocumentSituationsAnswered>,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        disaster_choices: Vec<i32>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        inherited_iras: Vec<RecordChoice>,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        businesses: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        people: Vec<PersonChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Available wage scorporations
        #[libertas_hidden]
        #[libertas_read_only]
        wage_scorporations: Vec<RecordChoice>,
    },
    /// Questions about your documents
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,SaveChecklist,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    SaveDocumentQuestions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.topic")]
        topic: TaxSection,
        /// Questions about your documents
        #[libertas_read_only]
        #[libertas_copy_from("$.questions")]
        /// ----
        /// Question
        questions: Vec<crate::DocumentStrictQuestion>,
        /// Do any of these apply to your documents?
        #[libertas_constrained_by("$.situation_answers")]
        #[libertas_copy_from("$.situations[0].value")]
        situations: crate::DocumentSituations,
        /// Situations these documents can have
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_situations")]
        allowed_situations: Vec<i32>,
        /// Answers the situations question offers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.situation_answers")]
        situation_answers: Vec<i32>,
        /// Documents a situation can apply to
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.situation_documents")]
        situation_documents: Vec<RecordChoice>,
        /// Available disaster answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.disaster_choices")]
        disaster_choices: Vec<i32>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Available wage scorporations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
    },

    /// Situations people often miss
    #[libertas_response]
    #[libertas_workflow_request("SaveOftenMissed")]
    BeginOftenMissed {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Not listed? See all topics or Search
        #[libertas_read_only]
        all_topics: AllTopicsLink,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },

    /// Federal payments
    #[libertas_response]
    #[libertas_workflow_request("SavePayments")]
    BeginPayments {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Available 2025 return choices
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_prior_returns: Vec<i32>,
        /// Available farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_farmer: Vec<i32>,
        /// Available 2027 farmer and fisherman answers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_farmer_next_year: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_refund_management: Vec<i32>,
        /// Federal payments
        #[libertas_read_only]
        value: crate::federal_model::PaymentsInitial,
    },

    /// More credits
    #[libertas_response]
    #[libertas_workflow_request("SaveMoreCredits")]
    BeginMoreCredits {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        /// [NullText]
        /// Current estimate is unavailable while these answers need correction.
        /// [SetText]
        /// Current estimate
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        /// [NullText]
        /// Current interview progress is unavailable while these answers need correction.
        /// [SetText]
        /// Current interview progress
        progress: Option<Vec<u8>>,
        /// Available owners
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_owners: Vec<i32>,
        /// Businesses run abroad on this return
        #[libertas_hidden]
        #[libertas_read_only]
        foreign_businesses: Vec<RecordChoice>,
        /// Investment sales on this return
        #[libertas_hidden]
        #[libertas_read_only]
        capital_sales: Vec<RecordChoice>,
        /// Estate or trust foreign-allocation sources on this return
        #[libertas_hidden]
        #[libertas_read_only]
        estate_foreign_sources: Vec<RecordChoice>,
        /// Foreign tax situations
        #[libertas_hidden]
        #[libertas_read_only]
        foreign_situations: Vec<ForeignSituationChoice>,
        /// Saved first-lender loans
        #[libertas_hidden]
        #[libertas_read_only]
        mortgage_loans: Vec<RecordChoice>,
        /// Expected document tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Available businesses
        #[libertas_hidden]
        #[libertas_read_only]
        businesses: Vec<RecordChoice>,
        /// Available farms
        #[libertas_hidden]
        #[libertas_read_only]
        farms: Vec<RecordChoice>,
        /// Saved clean vehicle property choices
        #[libertas_hidden]
        #[libertas_read_only]
        clean_vehicle_properties: Vec<RecordChoice>,
        /// Available rentals
        #[libertas_hidden]
        #[libertas_read_only]
        rentals: Vec<RecordChoice>,
        /// Available research businesses
        #[libertas_hidden]
        #[libertas_read_only]
        research_businesses: Vec<RecordChoice>,
        /// Available entered credits
        #[libertas_hidden]
        #[libertas_read_only]
        entered_credits: Vec<i32>,
        /// Available carryback credits
        #[libertas_hidden]
        #[libertas_read_only]
        carryback_credits: Vec<i32>,
        /// Available allowed carry filers
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_carry_filers: Vec<i32>,
        /// Available initial credit elections
        #[libertas_hidden]
        #[libertas_read_only]
        initial_credit_elections: Vec<i32>,
        /// Available initial research choices
        #[libertas_hidden]
        #[libertas_read_only]
        initial_research_choices: Vec<i32>,
        /// More credits
        #[libertas_read_only]
        value: crate::federal_model::MoreCreditsInitial,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        initial_renewable_facility_origins: Vec<i32>,
        /// Accepted sources containing already reported bond income
        #[libertas_hidden]
        #[libertas_read_only]
        credit_income_sources: Vec<crate::CreditIncomeSource>,
        /// Available credit owners
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_credit_owners: Vec<i32>,
    },
    /// Injured spouse allocation (Form 8379)
    #[libertas_response]
    #[libertas_workflow_request("SaveInjuredSpouse")]
    BeginInjuredSpouse {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Accepted allocation and question context
        #[libertas_read_only]
        value: InjuredSpouseInitial,
        /// Who managed the withholding from your documents
        #[libertas_read_only]
        followups: Vec<crate::InjuredSpouseDocumentQuestion>,
        /// Who managed the withholding from the documents that call for it
        #[libertas_read_only]
        /// ----
        /// Question
        questions: Vec<crate::InjuredOwnQuestion>,
    },
    /// Applicable original-system source questions
    #[libertas_request]
    #[libertas_next_response("AssetMacrsSourceChoices")]
    AssetMacrsSourceOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.history")]
        history: crate::AssetHistory,
        /// The selected source's independent factual prerequisites
        #[libertas_copy_from("@.source")]
        source: crate::MacrsSourceQuestionFacts,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.macrs_question_context")]
        context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    },
    /// Original-system source choices
    #[libertas_response]
    AssetMacrsSourceChoices {
        #[libertas_copy_to("@.source.allowed_original_system")]
        original_system: Vec<i32>,
    },
    /// Applicable class-life source questions after original-system choices settle
    #[libertas_request]
    #[libertas_next_response("AssetMacrsClassLifeChoices")]
    AssetMacrsClassLifeOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.class")]
        class: crate::AssetClass,
        #[libertas_copy_from("@.source")]
        source: crate::MacrsSourceQuestionFacts,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.source.original_system")]
        original_system: Option<crate::OriginalMacrsSystem>,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.macrs_question_context")]
        context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    },
    /// Applicable class-life source choices
    #[libertas_response]
    AssetMacrsClassLifeChoices {
        #[libertas_copy_to("@.source.allowed_class_life")]
        class_life: Vec<i32>,
    },
    /// Applicable original-system source questions
    #[libertas_request]
    #[libertas_next_response("PropertyMacrsSourceChoices")]
    PropertyMacrsSourceOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.history")]
        history: crate::EquipmentHistory,
        /// The selected source's independent factual prerequisites
        #[libertas_copy_from("@.source")]
        source: crate::MacrsSourceQuestionFacts,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.macrs_question_context")]
        context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    },
    /// Original-system source choices
    #[libertas_response]
    PropertyMacrsSourceChoices {
        #[libertas_copy_to("@.source.allowed_original_system")]
        original_system: Vec<i32>,
    },
    /// Applicable class-life source questions after original-system choices settle
    #[libertas_request]
    #[libertas_next_response("PropertyMacrsClassLifeChoices")]
    PropertyMacrsClassLifeOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.class")]
        class: crate::GdsEquipmentClass,
        #[libertas_copy_from("@.source")]
        source: crate::MacrsSourceQuestionFacts,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.source.original_system")]
        original_system: Option<crate::OriginalMacrsSystem>,
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.macrs_question_context")]
        context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    },
    /// Applicable class-life source choices
    #[libertas_response]
    PropertyMacrsClassLifeChoices {
        #[libertas_copy_to("@.source.allowed_class_life")]
        class_life: Vec<i32>,
    },

    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("AssetAmtCarryChoices")]
    AssetAmtCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("@.recovery.Gds.facts.history")]
        history: crate::AssetHistory,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    AssetAmtCarryChoices {
        #[libertas_copy_to("@.allowed_amt_carry")]
        allowed: Vec<i32>,
    },

    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("VehicleAmtCarryChoices")]
    VehicleAmtCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("@.history")]
        history: crate::AssetHistory,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    VehicleAmtCarryChoices {
        #[libertas_copy_to("@.allowed_amt_carry")]
        allowed: Vec<i32>,
    },

    /// Applicable actual AMT section179 carry source for this property
    #[libertas_request]
    #[libertas_next_response("CanonicalAssetAmtCarryChoices")]
    CanonicalAssetAmtCarryOptions {
        /// Existing physical-building associations available to supplied improvement records
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from(
            "^<SaveIncomes>.rental_energy_buildings,^<DocumentPropertyOptions>.rental_energy_buildings,^<PreviewDocument>.rental_energy_buildings,^<EstimateDocuments>.rental_energy_buildings"
        )]
        rental_energy_buildings: Vec<RecordChoice>,
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current source facts used only to decide whether a separate answer is needed
        #[libertas_copy_from("@.financial")]
        financial: crate::GdsPropertyPage,
    },
    /// Current applicable actual AMT source answer
    #[libertas_response]
    CanonicalAssetAmtCarryChoices {
        #[libertas_copy_to("@.allowed_amt_carry")]
        allowed: Vec<i32>,
    },
    /// Applicable actual AMT carry source for non-MACRS business property
    #[libertas_request]
    #[libertas_next_response("NonMacrsAssetCarryChoices")]
    NonMacrsAssetCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual original recovery facts
        #[libertas_copy_from("@.recovery.NonMacrs.facts")]
        recovery: crate::NonMacrsPropertyRecovery,
    },
    /// Applicable actual AMT carry source choices
    #[libertas_response]
    NonMacrsAssetCarryChoices {
        #[libertas_copy_to("@.allowed_amt_carry")]
        allowed: Vec<i32>,
    },
    /// Questions called for by the actual intangible source and start date
    #[libertas_request]
    #[libertas_next_response("IntangibleSourceChoices")]
    IntangibleSourceOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        #[libertas_copy_from("@.source.kind")]
        kind: crate::IntangibleKind,
        #[libertas_copy_from("@.source.obtained")]
        obtained: crate::IntangibleObtained,
        #[libertas_copy_from("@.source.context")]
        context: crate::IntangibleAcquisitionContext,
        #[libertas_date_only]
        #[libertas_copy_from("@.acquired")]
        acquired: u32,
        #[libertas_date_only]
        #[libertas_copy_from("@.business_started")]
        business_started: u32,
    },
    /// Applicable origin and prior-deduction questions
    #[libertas_response]
    IntangibleSourceChoices {
        #[libertas_copy_to("@.origin_choices")]
        origin_choices: Vec<i32>,
        #[libertas_copy_to("@.prior_choices")]
        prior_choices: Vec<i32>,
    },
    /// Applicable actual AMT carry source for non-MACRS rental property
    #[libertas_request]
    #[libertas_next_response("NonMacrsCanonicalCarryChoices")]
    NonMacrsCanonicalCarryOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual original recovery facts
        #[libertas_copy_from("@.recovery")]
        recovery: crate::NonMacrsPropertyRecovery,
    },
    /// Applicable actual AMT carry source choices
    #[libertas_response]
    NonMacrsCanonicalCarryChoices {
        #[libertas_copy_to("@.allowed_amt_carry")]
        allowed: Vec<i32>,
    },

    /// Running estimate
    #[libertas_response]
    PaymentsEstimate {
        /// Estimate
        #[libertas_read_only]
        #[libertas_copy_to("@.estimate")]
        estimate: Option<EstimateCarry>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_copy_to("@.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_copy_to("@.progress")]
        progress: Vec<u8>,
    },
    /// Applicable historical choices for the actual rental source dates
    #[libertas_request]
    #[libertas_next_response("RentalRecoveryChoices")]
    RentalRecoveryOptions {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Actual property kind and source dates
        #[libertas_copy_from("@")]
        value: crate::RentalRecoveryInputs,
    },
    /// Historical choices for the same current building and improvement rows
    #[libertas_response]
    RentalRecoveryChoices {
        /// Original building choices, without replacing its selected Rental document
        #[libertas_copy_to("@.building")]
        building: crate::RentalRecoveryAllowedRow,
        /// Existing improvement choices, preserving source values and row membership
        #[libertas_copy_to("@.more")]
        more: crate::RentalRecoveryMoreAllowed,
    },
    /// Document center
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    SaveDocuments {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Available source categories and their kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.source_choices")]
        source_choices: crate::IncomeSourceChoices,
        /// Empty construction choices until actual asset recovery facts are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
        initial_asset_amt_carry_choices: Vec<i32>,
        /// Empty construction choices until actual rental dates are supplied
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_rental_recovery_choices")]
        initial_rental_recovery_choices: Vec<i32>,
        /// Businesses available for individual QBI aggregation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.qbi_sources")]
        qbi_sources: Vec<crate::QbiAggregationMemberChoice>,
        /// Saved physical buildings for section 179D cost-source claims
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rental_energy_buildings")]
        rental_energy_buildings: Vec<RecordChoice>,
        /// Saved personal casualty properties
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.casualty_properties")]
        casualty_properties: Vec<RecordChoice>,
        /// Empty construction choices for a new K-1 source; accepted parent choices come with its supplied editor value.
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.empty_k1_activity_choices")]
        empty_k1_activity_choices: alloc::boxed::Box<crate::K1ActivityChoiceContext>,
        /// Source ownership facts apply only to an injured-spouse claim
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_source_facts")]
        allowed_injured_source_facts: Vec<i32>,
        /// Actual ownership answers for retained Form 8960 and absorbed carryback sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_injured_retained_owners")]
        allowed_injured_retained_owners: Vec<i32>,
        /// Existing long-term-care statements for actual period payments
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_payee_sources")]
        ltc_payee_sources: Vec<RecordChoice>,

        /// S-corporation K-1 records that may have paid a W-2
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.wage_scorporations")]
        wage_scorporations: Vec<RecordChoice>,
        /// Available S-corporation employer relation
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_wage_scorporations")]
        allowed_wage_scorporations: Vec<i32>,
        /// Initial automatic investment-confirmation choices for a fresh K-1 row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_k1_investment_choices")]
        initial_k1_investment_choices: Vec<i32>,
        /// Actual current businesses available for retained research attribution
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_businesses")]
        research_businesses: Vec<RecordChoice>,
        /// Applicable retained-source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.retained_research_choices")]
        retained_research_choices: Vec<i32>,
        /// Available k1 credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.k1_credit_lines")]
        k1_credit_lines: Vec<i32>,
        /// Available research credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.research_credit_lines")]
        research_credit_lines: Vec<i32>,
        /// Available renewable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.renewable_credit_lines")]
        renewable_credit_lines: Vec<i32>,
        /// Available transferable credit lines
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.transferable_credit_lines")]
        transferable_credit_lines: Vec<i32>,
        /// Available at-risk histories for ownership begun before 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_continuing_risk")]
        allowed_continuing_risk: Vec<i32>,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        /// [NullText]
        /// Awaiting corrected income source answers
        /// [SetText]
        /// Current interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Option<Vec<u8>>,
        // Part of the draft identity, so a Wages draft never restores into
        // the Interest and dividends page.
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.topic")]
        topic: TaxSection,
        /// Allowed document kinds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_documents")]
        allowed_documents: Vec<i32>,
        /// Allowed document owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_owners")]
        allowed_owners: Vec<i32>,
        /// Allowed community-property answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_community")]
        allowed_community: Vec<i32>,
        /// Allowed management of community funds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_refund_management")]
        allowed_refund_management: Vec<i32>,
        /// Foreign countries for the separate FBAR reporting answer
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        foreign_countries: Vec<RecordChoice>,
        /// Accepted interest statements and annual instruments for education bonds
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        interest_statements: Vec<RecordChoice>,
        /// People on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.people")]
        people: Vec<PersonChoice>,
        /// Farms and farm rentals on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.farms")]
        farms: Vec<RecordChoice>,
        /// Accepted same-owner business, farm and farm-rental recipients
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_activities")]
        cooperative_activities: Vec<RecordChoice>,
        /// Accepted property-owner links; unavailable owners supply no choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.cooperative_properties")]
        cooperative_properties: Vec<RecordChoice>,
        /// Businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.businesses")]
        businesses: Vec<RecordChoice>,
        /// Schedule C and Schedule F businesses on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.qbi_businesses")]
        qbi_businesses: Vec<RecordChoice>,
        /// Rental properties on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.rentals")]
        rentals: Vec<RecordChoice>,
        /// Saved canonical mortgage loans for home rentals
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.mortgage_loans")]
        mortgage_loans: Vec<RecordChoice>,
        /// Royalty entries on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.royalty_properties")]
        royalty_properties: Vec<RecordChoice>,
        /// Equipment rented out on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.equipment_rentals")]
        equipment_rentals: Vec<RecordChoice>,
        /// Hobbies on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.hobbies")]
        hobbies: Vec<RecordChoice>,
        /// Commodity Credit Corporation loans forfeited in 2026
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ccc_lots")]
        ccc_lots: Vec<RecordChoice>,
        /// Current principal agricultural codes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::code_choices (FARM_CODES).
        #[libertas_size(max = 17)]
        farm_codes: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_limitations
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        // Fixed farm-code taxonomy supplied by farm_specials::limitation_choices (codes 1 through 10).
        #[libertas_size(max = 10)]
        farm_limitations: Vec<FarmCodeChoice>,
        /// Current accepted farm source catalog: farm_sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_sources: Vec<FarmChoice>,
        /// Current accepted farm source catalog: basis_assets
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        basis_assets: Vec<FarmChoice>,
        /// Current accepted farm source catalog: farm_method_changes
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        farm_method_changes: Vec<FarmChoice>,
        /// Inherited traditional IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_iras")]
        inherited_iras: Vec<RecordChoice>,
        /// Inherited Roth IRAs on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.inherited_roth_iras")]
        inherited_roth_iras: Vec<RecordChoice>,
        /// Long-term care periods records on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ltc_insured")]
        ltc_insured: Vec<RecordChoice>,
        /// Actual nonqualified annuity sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.annuity_sources")]
        annuity_sources: Vec<crate::LtcSourceChoice>,
        /// Actual prior-year deduction recovery sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.recovery_sources")]
        recovery_sources: Vec<crate::LtcSourceChoice>,
        /// Allowed business income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_business_source")]
        allowed_business_source: Vec<i32>,
        /// Allowed farm income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_farm_source")]
        allowed_farm_source: Vec<i32>,
        /// Allowed partnership income sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_partner_source")]
        allowed_partner_source: Vec<i32>,
        /// Allowed home sale spouse answers
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_home_spouse")]
        allowed_home_spouse: Vec<i32>,
        /// Allowed answers about the return reporting a child's income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_other_parent")]
        allowed_other_parent: Vec<i32>,
        /// Children who can be reported on this return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.children")]
        children: Vec<crate::ChildChoice>,
        /// Expected W-2 tax year
        #[libertas_hidden]
        #[libertas_read_only]
        document_facts: DocumentFacts,
        /// Income documents
        documents: crate::DocumentCenter,
        /// Initial equipment-origin choices for a fresh credit row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.initial_carbon_oxide_origins")]
        initial_carbon_oxide_origins: Vec<i32>,
        /// Initial applicability for a newly added renewable-facility source row
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.initial_renewable_facility_origins")]
        #[libertas_output_only]
        initial_renewable_facility_origins: Vec<i32>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_people")]
        ptc_people: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_families")]
        ptc_families: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_policies")]
        ptc_policies: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_w2")]
        ptc_w2: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_employees")]
        ptc_employees: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_nonreturn_people")]
        ptc_nonreturn_people: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.ptc_claimed_dependents")]
        ptc_claimed_dependents: Vec<RecordChoice>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_marriage")]
        allowed_marriage: Vec<i32>,
        /// Available Marketplace source choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_separate")]
        allowed_separate: Vec<i32>,
        /// Saved rental mortgage sources
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.rental_mortgage_loans")]
        rental_mortgage_loans: Vec<i64>,
        /// Accepted sources containing already reported bond income
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.credit_income_sources")]
        credit_income_sources: Vec<crate::CreditIncomeSource>,
        /// Available credit owners
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_credit_owners")]
        allowed_credit_owners: Vec<i32>,
        /// Initial ownership choice until a document is selected
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.allowed_document_ownership")]
        allowed_document_ownership: Vec<i32>,
        /// Whose documents
        #[libertas_read_only]
        #[libertas_copy_from("$.document_owners")]
        document_owners: Vec<crate::PersonDocuments>,
    },

    /// Project the related form
    #[libertas_request]
    #[libertas_next_response("RelatedFormProjected,Problem")]
    ProjectRelatedForm {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Chosen form
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.key")]
        key: i64,
    },
    /// Related form summary
    #[libertas_response]
    RelatedFormProjected {
        /// Summary
        #[libertas_read_only]
        #[libertas_copy_to("@.summary")]
        summary: Option<RelatedFormSummary>,
    },
    /// View the related form
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("FormViewed,Problem,StoredReturnUnavailable")]
    ViewRelatedForm {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted form or part identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^.key")]
        key: i64,
    },
    /// View the reviewed form
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("FormViewed,Problem,StoredReturnUnavailable")]
    ViewReviewedRelatedForm {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted form or part identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<ReviewedRelatedForm>.key")]
        key: i64,
    },
    /// Related form
    /// Edit this form in Documents.
    #[libertas_response]
    FormViewed {
        /// Whose form
        #[libertas_read_only]
        owner: Option<Owner>,
        /// Payer or lender
        #[libertas_read_only]
        payer: String,
        /// Accepted form
        #[libertas_read_only]
        form: RelatedForm,
    },
    /// Forms available to this farming activity
    #[libertas_request]
    #[libertas_next_response("FarmRelatedChoices,Problem")]
    FarmRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Activity owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current activity title identifies this selected record variant
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.Farm.name")]
        name: String,
    },
    /// Forms available to this farming activity
    #[libertas_response]
    FarmRelatedChoices {
        #[libertas_read_only]
        #[libertas_copy_to("@.document.Farm.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },

    /// Forms available to this farming activity
    #[libertas_request]
    #[libertas_next_response("FarmRentalRelatedChoices,Problem")]
    FarmRentalRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Activity owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current activity title identifies this selected record variant
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.FarmRental.name")]
        name: String,
    },
    /// Forms available to this farming activity
    #[libertas_response]
    FarmRentalRelatedChoices {
        #[libertas_read_only]
        #[libertas_copy_to("@.document.FarmRental.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },
    /// Forms available to the activity being edited
    #[libertas_request]
    #[libertas_next_response("HobbyRelatedChoices,Problem")]
    HobbyRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Activity owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current activity input
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.HobbySales.ordinary")]
        activity_input: bool,
    },
    /// Applicable activity forms
    #[libertas_response]
    HobbyRelatedChoices {
        /// Forms available to this activity
        #[libertas_read_only]
        #[libertas_copy_to("@.document.HobbySales.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },

    /// Forms available to the activity being edited
    #[libertas_request]
    #[libertas_next_response("EquipmentRelatedChoices,Problem")]
    EquipmentRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Activity owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current activity input
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.EquipmentRental.basis")]
        #[libertas_money("USD", 2)]
        // Match the source property's cost bounds.
        #[libertas_number(min = 0, max = 100000000000)]
        activity_input: i64,
    },
    /// Applicable activity forms
    #[libertas_response]
    EquipmentRelatedChoices {
        /// Forms available to this activity
        #[libertas_read_only]
        #[libertas_copy_to("@.document.EquipmentRental.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },

    /// Forms available to the activity being edited
    #[libertas_request]
    #[libertas_next_response("RoyaltyRelatedChoices,Problem")]
    RoyaltyRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Activity owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current activity input
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.Royalties.special")]
        activity_input: bool,
    },
    /// Applicable activity forms
    #[libertas_response]
    RoyaltyRelatedChoices {
        /// Forms available to this activity
        #[libertas_read_only]
        #[libertas_copy_to("@.document.Royalties.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },
    /// Forms available to the business being edited
    #[libertas_request]
    #[libertas_next_response("BusinessRelatedChoices,Problem")]
    BusinessRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Current return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.cookie")]
        cookie: String,
        /// Current accepted source-answer revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.revision")]
        revision: i64,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Business owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current business source
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.Business.more.source")]
        source: BusinessSource,
        /// Current professional-gambling classification
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.Business.more.gambling")]
        gambling: Option<crate::ProfessionalGambling>,
    },
    /// Split this Form 1099-K's payments
    /// Accept the source amounts first, then relate their stable identities from the activity.
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("PaymentPartsSaved,Problem")]
    SavePaymentParts {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.cookie,^<EstimateDocuments>.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<SaveIncomes>.revision,^<EstimateDocuments>.revision")]
        revision: i64,
        /// Accepted source question
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<DocumentQuestion>.key")]
        key: String,
        /// Parts of printed box 1a
        /// Separate activity payments from non-activity payments. The parts must equal box 1a; refunds belong to their actual part.
        #[libertas_copy_from("^<DocumentQuestion>.answer.PaymentParts.value")]
        #[libertas_size(min = 1)]
        value: Vec<crate::KPart>,
    },
    /// Accepted payment parts
    /// This plain link return preserves every unrelated pending editor field.
    #[libertas_response]
    PaymentPartsSaved {
        /// Accepted parts with their durable source identities
        #[libertas_read_only]
        #[libertas_copy_to("^<DocumentQuestion>.answer.PaymentParts.value")]
        value: Vec<crate::KPart>,
        /// New accepted revision of this same return
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_to("^<SaveIncomes>.revision,^<EstimateDocuments>.revision")]
        revision: i64,
    },

    /// Applicable business forms
    #[libertas_response]
    BusinessRelatedChoices {
        /// Forms available to this business
        #[libertas_read_only]
        #[libertas_copy_to("@.document.Business.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },
    /// Your investment at risk
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("RentalRiskAnswered,Problem")]
    AnswerRentalRisk {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Your investment at risk
        /// Cash investment, recourse borrowing and qualifying nonrecourse real-estate financing can qualify. Choose Limited for Form 6198 amounts and prior disallowed losses.
        #[libertas_copy_from("^<RentalDetails>.known_at_risk[0]")]
        value: crate::RentalAtRisk,
    },
    /// Your investment at risk
    #[libertas_response]
    RentalRiskAnswered {
        /// Your at-risk investment
        #[libertas_copy_to("^<RentalDetails>.at_risk")]
        value: crate::RentalAtRisk,
        /// Known answer for later corrections
        #[libertas_copy_to("^<RentalDetails>.known_at_risk")]
        known: Vec<crate::RentalAtRisk>,
    },
    /// Forms available to the rental being edited
    #[libertas_request]
    #[libertas_next_response("RentalRelatedChoices,Problem")]
    RentalRelatedOptions {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Accepted activity identity, or zero for a new activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.id")]
        id: i64,
        /// Rental owner
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.owner")]
        owner: Owner,
        /// Current personal use
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("@.document.Rental.personal_use")]
        personal_use: PersonalUse,
    },
    /// Applicable rental forms
    #[libertas_response]
    RentalRelatedChoices {
        /// Forms available to this rental
        #[libertas_read_only]
        #[libertas_copy_to("@.document.Rental.related.candidates")]
        candidates: Vec<RelatedFormChoice>,
    },
    // Income without a form, opened by the walk at once after the Document
    // center: asked unanswered until saved (BeginIncomeWithoutForm).
    /// Income without a form
    #[libertas_response]
    #[libertas_workflow_request("SaveIncomeWithoutForm")]
    BeginIncomeWithoutForm {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        suggested_destination: NavigationDestination,
    },
    /// Income without a form
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateIncomeWithoutForm")]
    SaveIncomeWithoutForm {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_cache_identity]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Income without a form
        #[libertas_calculation_request("EstimateIncomeWithoutForm")]
        #[libertas_copy_from("$.value")]
        value: crate::IncomeWithoutFormAnswers,
        /// Available navigation choices
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.navigation")]
        navigation: NavigationAvailability,
        /// Suggested destination
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.suggested_destination")]
        suggested_destination: NavigationDestination,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateIncomeWithoutForm {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Income without a form
        #[libertas_copy_from("@.value")]
        value: crate::IncomeWithoutFormAnswers,
    },
    /// Saved return cannot be opened
    #[libertas_response]
    #[libertas_next_request("StartNewReturn")]
    StoredReturnUnavailable {
        /// Saved return
        #[libertas_formatted_text]
        #[libertas_read_only]
        message: Vec<u8>,
        /// Start a new return
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_link("StartNewReturn")]
        action: Vec<u8>,
        /// Return year
        #[libertas_hidden]
        tax_year: i32,
        /// Saved format
        #[libertas_hidden]
        format: Option<u32>,
    },
    /// Start a new 2026 return
    /// Permanently delete your saved return for this year and open an empty return. Cancel keeps the saved return.
    #[libertas_request]
    #[libertas_request_confirmation]
    #[libertas_link_only]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    StartNewReturn {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return year
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.tax_year")]
        tax_year: i32,
        /// Saved format
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.format")]
        format: Option<u32>,
    },
    // A separate return's community-property question, a step of the filers
    // section after About you: asked unanswered until saved (BeginCommunityProperty).
    /// Community property
    #[libertas_response]
    #[libertas_workflow_request("SaveCommunityProperty")]
    BeginCommunityProperty {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// Community property
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateCommunityProperty")]
    SaveCommunityProperty {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// You or your spouse were domiciled in a community-property state or country in 2026
        /// Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington or Wisconsin, or a country with community-property law, for all or part of 2026. Its law divides your income between your separate returns (Form 8958, Pub. 555).
        #[libertas_calculation_request("EstimateCommunityProperty")]
        value: crate::CommunityAnswer,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateCommunityProperty {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// You or your spouse were domiciled in a community-property state or country in 2026
        /// Arizona, California, Idaho, Louisiana, Nevada, New Mexico, Texas, Washington or Wisconsin, or a country with community-property law, for all or part of 2026. Its law divides your income between your separate returns (Form 8958, Pub. 555).
        #[libertas_copy_from("@.value")]
        value: crate::CommunityAnswer,
    },
    // A separate return's spouse boxes, a step of the deductions section after its
    // page while the standard deduction can apply: asked unanswered until saved.
    /// Your spouse's standard deduction boxes
    #[libertas_response]
    #[libertas_workflow_request("SaveSpouseBoxes")]
    BeginSpouseBoxes {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// Your spouse's standard deduction boxes
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,BeginDocumentQuestions,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,BeginMortgages,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSpouseBoxes")]
    SaveSpouseBoxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// Boxes that apply to your spouse
        /// Only when your spouse had no gross income, is not filing a return and cannot be claimed as another person's dependent (Form 1040 line 12d). Each box adds $1,650 to your standard deduction.
        #[libertas_calculation_request("EstimateSpouseBoxes")]
        value: crate::SpouseBoxes,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSpouseBoxes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Boxes that apply to your spouse
        /// Only when your spouse had no gross income, is not filing a return and cannot be claimed as another person's dependent (Form 1040 line 12d). Each box adds $1,650 to your standard deduction.
        #[libertas_copy_from("@.value")]
        value: crate::SpouseBoxes,
    },
    /// Does your spouse itemize on their separate return?
    #[libertas_response]
    #[libertas_workflow_request("SaveSpouseItemizes")]
    BeginSpouseItemizes {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        estimate: Option<EstimateCarry>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        progress: Vec<u8>,
    },
    /// Does your spouse itemize on their separate return?
    #[libertas_request]
    #[libertas_prev_request("Back")]
    #[libertas_next_response(
        "BeginSetup,SaveSetup,BeginPeople,SavePeople,BeginSpouse,SaveSpouse,BeginCommunityProperty,SaveCommunityProperty,BeginChildren,SaveChildren,SaveDocuments,SaveIncomes,SaveWagesAnswers,SaveHsa,BeginCare,SaveCare,SaveSelfEmployedBenefits,SaveIraContributions,SaveEducator,SaveStudentLoan,SaveEmployeeExpenses,SaveDeductions,BeginSpouseItemizes,SaveSpouseItemizes,BeginSpouseBoxes,SaveSpouseBoxes,SaveMedical,SaveTaxes,SaveOtherItemized,SaveMortgages,SaveHomeOffices,SaveGiftCarryovers,SaveEducation,SaveSaver,SaveMarketplace,BeginMoreCredits,SaveMoreCredits,SaveMoreAdjustments,SaveOtherTaxes,BeginPayments,SavePayments,SaveIncomeByPeriod,BeginInjuredSpouse,SaveInjuredSpouse,SaveParentTax,BeginChecklist,SaveChecklist,BeginIncomeWithoutForm,SaveIncomeWithoutForm,BeginOftenMissed,SaveOftenMissed,Hub,Review,BeginFinish,Finish,Finished,Problem,StoredReturnUnavailable"
    )]
    #[libertas_calculation_root("EstimateSpouseItemizes")]
    SaveSpouseItemizes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Current section
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.page")]
        page: i32,
        /// Previous page
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.previous")]
        previous: i32,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Estimate
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.estimate")]
        estimate: Option<EstimateSoFar>,
        /// What saving the current edits would remove or reopen
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.warnings")]
        /// ----
        /// Change
        warnings: Vec<SaveWarning>,
        /// Interview progress
        #[libertas_formatted_text]
        #[libertas_read_only]
        #[libertas_output_only]
        #[libertas_copy_from("$.progress")]
        progress: Vec<u8>,
        /// My spouse itemizes on their separate return
        #[libertas_calculation_request("EstimateSpouseItemizes")]
        value: bool,
    },
    /// Calculate the running estimate
    #[libertas_request]
    #[libertas_next_response("RunningEstimate,Problem")]
    EstimateSpouseItemizes {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// My spouse itemizes on their separate return
        #[libertas_copy_from("@.value")]
        value: bool,
    },
    /// Crop-insurance timing and statement details
    #[libertas_request]
    #[libertas_next_response("CropInsuranceFactsPage,Problem")]
    #[libertas_link_only]
    OpenCropInsuranceFacts {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity,^<CropInsuranceActivityLink>.activity")]
        activity: i64,
    },
    /// Crop-insurance details
    #[libertas_response]
    #[libertas_next_request(
        "SaveCropInsuranceTiming,CloseCropInsuranceFacts,OpenCropPaperFacts,OpenCropPractice"
    )]
    #[libertas_constrained_by("$.allowed_actions")]
    CropInsuranceFactsPage {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        activity: i64,
        /// Farm or farm rental
        #[libertas_read_only]
        #[libertas_formatted_text]
        label: Vec<u8>,
        /// Available actions
        #[libertas_hidden]
        #[libertas_read_only]
        allowed_actions: Vec<i32>,
        /// Accepted timing answer
        #[libertas_hidden]
        #[libertas_read_only]
        accepted: Option<crate::farm_insurance_paper::AcceptedCropTiming>,
        /// Related printed statements
        /// Their printed proceeds are counted automatically. Physical-loss details are needed only for a deferral election.
        #[libertas_read_only]
        papers: Vec<crate::farm_insurance_paper::CropPaperLink>,
        /// Normal sales practice for affected crops
        #[libertas_read_only]
        crops: Vec<crate::farm_insurance_paper::CropPracticeLink>,
    },
    /// Choose income timing
    #[libertas_request]
    #[libertas_next_response("CropInsuranceFactsPage,Problem")]
    #[libertas_prev_request("OpenCropInsuranceFacts")]
    SaveCropInsuranceTiming {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
        /// Income timing for all eligible proceeds of this farming business
        #[libertas_copy_from("$.accepted.timing")]
        timing: crate::farm_insurance_paper::CropPaperTiming,
    },
    /// Return to farming
    #[libertas_request]
    #[libertas_next_response("SaveIncomes,Problem")]
    CloseCropInsuranceFacts {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
    },
    /// Physical-loss details for this statement
    #[libertas_request]
    #[libertas_next_response("BeginCropPaperFacts,Problem")]
    #[libertas_link_only]
    OpenCropPaperFacts {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
        /// Printed crop-insurance component
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<CropPaperLink>.source")]
        source: i64,
    },
    /// Character of these printed proceeds
    #[libertas_response]
    #[libertas_next_request("SaveCropPaperFacts")]
    BeginCropPaperFacts {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        activity: i64,
        /// Printed crop-insurance component
        #[libertas_hidden]
        #[libertas_read_only]
        source: i64,
        /// Printed proceeds
        #[libertas_read_only]
        summary: crate::RelatedFormSummary,
        /// Accepted characterization
        #[libertas_hidden]
        #[libertas_read_only]
        accepted: Option<crate::farm_insurance_paper::AcceptedCropClassification>,
    },
    /// Physical-loss facts
    #[libertas_request]
    #[libertas_next_response("CropInsuranceFactsPage,Problem")]
    #[libertas_prev_request("OpenCropInsuranceFacts")]
    SaveCropPaperFacts {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
        /// Printed crop-insurance component
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.source")]
        source: i64,
        /// Proceeds from the printed statement
        #[libertas_read_only]
        #[libertas_copy_from("$.summary")]
        summary: crate::RelatedFormSummary,
        /// Character of the printed proceeds
        #[libertas_copy_from("$.accepted.classification")]
        classification: crate::farm_insurance_paper::CropPaperClass,
    },
    /// Normal sales practice for this crop
    #[libertas_request]
    #[libertas_next_response("BeginCropPractice,Problem")]
    #[libertas_link_only]
    OpenCropPractice {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
        /// Affected crop
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<CropPracticeLink>.crop")]
        crop: String,
    },
    /// Normal sales practice
    #[libertas_response]
    #[libertas_next_request("SaveCropPractice")]
    BeginCropPractice {
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        activity: i64,
        /// Affected crop
        #[libertas_read_only]
        crop: String,
        /// Accepted practice record
        #[libertas_hidden]
        #[libertas_read_only]
        accepted: Option<crate::FarmCropPractice>,
    },
    /// Normal sales practice
    #[libertas_request]
    #[libertas_next_response("CropInsuranceFactsPage,Problem")]
    #[libertas_prev_request("OpenCropInsuranceFacts")]
    SaveCropPractice {
        /// Signed-in user
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Accepted farming activity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.activity")]
        activity: i64,
        /// Affected crop
        #[libertas_read_only]
        #[libertas_copy_from("$.crop")]
        crop: String,
        /// Normal total income from this crop
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1, max = 100000000000)]
        #[libertas_copy_from("$.accepted.normal_total")]
        normal_total: i64,
        /// Normal income in a year after the damage year
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_copy_from("$.accepted.normal_following")]
        normal_following: i64,
    },
    /// Answer the question about this stock sale
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("QsbsBooleanAnswered,Problem")]
    AnswerQsbsBoolean {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Stock-sale question
        #[libertas_read_only]
        #[libertas_formatted_text]
        #[libertas_copy_from("^<DocumentQuestion>.prompt")]
        prompt: Vec<u8>,
        /// Your answer
        #[libertas_copy_from("^<QsbsBooleanAnswer>.known[0]")]
        value: bool,
    },
    /// Stock-sale answer
    #[libertas_response]
    QsbsBooleanAnswered {
        #[libertas_copy_to("^<QsbsBooleanAnswer>.value")]
        value: bool,
        #[libertas_copy_to("^<QsbsBooleanAnswer>.known")]
        known: Vec<bool>,
    },
    /// Reconcile prior eligible gain for this issuer
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("QsbsPriorGainAnswered,Problem")]
    AnswerQsbsPriorGain {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Stock-sale question
        #[libertas_read_only]
        #[libertas_formatted_text]
        #[libertas_copy_from("^<DocumentQuestion>.prompt")]
        prompt: Vec<u8>,
        /// Eligible gain already taken into account for this issuer before 2026
        /// Reconcile the taxpayer's and, on a joint return, both spouses' prior returns. Allocate gain from an earlier joint return equally to its spouses. Enter zero only after checking the history, rather than entering prior proceeds or taxable gain.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_copy_from("^<QsbsGainAnswer>.known[0].value")]
        value: i64,
    },
    /// Reconciled prior eligible gain
    #[libertas_response]
    QsbsPriorGainAnswered {
        #[libertas_money("USD", 2)]
        #[libertas_copy_to("^<QsbsGainAnswer>.value")]
        value: i64,
        #[libertas_copy_to("^<QsbsGainAnswer>.known")]
        known: Vec<crate::QsbsGainValue>,
    },
    /// Answer the question about empowerment-zone records
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("QsbsZoneAnswered,Problem")]
    AnswerQsbsZone {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Stock-sale question
        #[libertas_read_only]
        #[libertas_formatted_text]
        #[libertas_copy_from("^<DocumentQuestion>.prompt")]
        prompt: Vec<u8>,
        /// Your records
        #[libertas_copy_from("^<QsbsZoneAnswer>.known[0]")]
        value: crate::QsbsZoneCertification,
    },
    /// Answer about empowerment-zone records
    #[libertas_response]
    QsbsZoneAnswered {
        #[libertas_copy_to("^<QsbsZoneAnswer>.value")]
        value: crate::QsbsZoneCertification,
        #[libertas_copy_to("^<QsbsZoneAnswer>.known")]
        known: Vec<crate::QsbsZoneCertification>,
    },
    /// Answer the question about empowerment-zone records
    #[libertas_request]
    #[libertas_link_only]
    #[libertas_next_response("QsbsValuationAnswered,Problem")]
    AnswerQsbsValuation {
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Stock-sale question
        #[libertas_read_only]
        #[libertas_formatted_text]
        #[libertas_copy_from("^<DocumentQuestion>.prompt")]
        prompt: Vec<u8>,
        /// Sold lot from the accepted statement
        #[libertas_read_only]
        #[libertas_copy_from("^<QsbsValuationAnswer>.lot")]
        lot: crate::QsbsPaperLot,
        /// Your records
        #[libertas_copy_from("^<QsbsValuationAnswer>.known[0]")]
        value: crate::QsbsZoneValuation,
    },
    /// Answer about empowerment-zone records
    #[libertas_response]
    QsbsValuationAnswered {
        #[libertas_copy_to("^<QsbsValuationAnswer>.value")]
        value: crate::QsbsZoneValuation,
        #[libertas_copy_to("^<QsbsValuationAnswer>.known")]
        known: Vec<crate::QsbsZoneValuation>,
    },
    // The More credits page's link to its strict document questions
    // (D-2026-11-04-07): offered while any are open; it has no input.
    /// Answer the questions about your documents
    #[libertas_request]
    #[libertas_next_response(
        "BeginDocumentQuestions,SaveDocumentQuestions,BeginMoreCredits,SaveMoreCredits,BeginMortgages,SaveMortgages,Problem,StoredReturnUnavailable"
    )]
    #[libertas_link_only]
    OpenDocumentQuestions {
        /// Signed-in user
        /// The Hub verifies this identity on every request; the App keeps each user's return apart.
        #[libertas_hidden]
        #[libertas_fixed("current-user")]
        user: LibertasUser,
        /// Return identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.cookie")]
        cookie: String,
        /// Accepted revision
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.revision")]
        revision: i64,
        /// Return to review after saving
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("$.review")]
        review: bool,
        /// Topic
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_copy_from("^<DocumentQuestionsLink>.topic")]
        topic: TaxSection,
    },
}

impl TaxInterviewProtocol {
    // Calculation replies update the invoking editor and must never offer navigation.
    pub(crate) fn is_calculation(&self) -> bool {
        matches!(
            self,
            Self::AssetAmtCarryOptions { .. }
                | Self::AssetMacrsClassLifeOptions { .. }
                | Self::AssetMacrsSourceOptions { .. }
                | Self::BusinessCreditElectionOptions { .. }
                | Self::BusinessRelatedOptions { .. }
                | Self::CalculateMarketplaceAppeal { .. }
                | Self::CanonicalAssetAmtCarryOptions { .. }
                | Self::DocumentOwnershipOptions { .. }
                | Self::DocumentPropertyOptions { .. }
                | Self::EquipmentRelatedOptions { .. }
                | Self::EstimateCare { .. }
                | Self::EstimateChecklist { .. }
                | Self::EstimateDeductions { .. }
                | Self::EstimateDependents { .. }
                | Self::EstimateDocuments { .. }
                | Self::EstimateEducation { .. }
                | Self::EstimateEducator { .. }
                | Self::EstimateEmployeeExpenses { .. }
                | Self::EstimateGiftCarryovers { .. }
                | Self::EstimateHomeOffices { .. }
                | Self::EstimateHsa { .. }
                | Self::EstimateIncomeByPeriod { .. }
                | Self::EstimateIncomeWithoutForm { .. }
                | Self::EstimateInjuredSpouse { .. }
                | Self::EstimateIraContributions { .. }
                | Self::EstimateMarketplace { .. }
                | Self::EstimateMedical { .. }
                | Self::EstimateMoreAdjustments { .. }
                | Self::EstimateMoreCredits { .. }
                | Self::EstimateMortgages { .. }
                | Self::EstimateOftenMissed { .. }
                | Self::EstimateOtherItemized { .. }
                | Self::EstimateOtherTaxes { .. }
                | Self::EstimateParentTax { .. }
                | Self::EstimatePayments { .. }
                | Self::EstimateSaver { .. }
                | Self::EstimateSelfEmployedBenefits { .. }
                | Self::EstimateSetup { .. }
                | Self::EstimateSpouseBoxes { .. }
                | Self::EstimateSpouseItemizes { .. }
                | Self::EstimateSpouse { .. }
                | Self::EstimateStudentLoan { .. }
                | Self::EstimateTaxes { .. }
                | Self::EstimateWagesAnswers { .. }
                | Self::EstimateYou { .. }
                | Self::FarmRelatedOptions { .. }
                | Self::FarmRentalRelatedOptions { .. }
                | Self::HobbyRelatedOptions { .. }
                | Self::HsaOptions { .. }
                | Self::IntangibleSourceOptions { .. }
                | Self::InvestmentOriginOptions { .. }
                | Self::K1InvestmentOptions { .. }
                | Self::NonMacrsAssetCarryOptions { .. }
                | Self::NonMacrsCanonicalCarryOptions { .. }
                | Self::PreviewDocument { .. }
                | Self::ProjectRelatedForm { .. }
                | Self::PropertyMacrsClassLifeOptions { .. }
                | Self::PropertyMacrsSourceOptions { .. }
                | Self::RentalRecoveryOptions { .. }
                | Self::RentalRelatedOptions { .. }
                | Self::ResearchCreditSourceOptions { .. }
                | Self::RetainedCapitalSourceOptions { .. }
                | Self::RetainedCharityAppreciationOptions { .. }
                | Self::RetainedLossSourceOptions { .. }
                | Self::RoyaltyRelatedOptions { .. }
                | Self::VehicleAmtCarryOptions { .. }
        )
    }

    /// The verified user carried by every request; None for a response.
    pub fn user(&self) -> Option<LibertasUser> {
        match self {
            Self::SavePaymentParts { user, .. }
            | Self::OpenCropInsuranceFacts { user, .. }
            | Self::SaveCropInsuranceTiming { user, .. }
            | Self::CloseCropInsuranceFacts { user, .. }
            | Self::OpenCropPaperFacts { user, .. }
            | Self::SaveCropPaperFacts { user, .. }
            | Self::OpenCropPractice { user, .. }
            | Self::SaveCropPractice { user, .. }
            | Self::OpenInterview { user, .. }
            | Self::StartNewReturn { user, .. }
            | Self::SaveSetup { user, .. }
            | Self::OpenHub { user, .. }
            | Self::ContinueIncome { user, .. }
            | Self::ContinueAdjustments { user, .. }
            | Self::ContinueItemized { user, .. }
            | Self::ContinueCredits { user, .. }
            | Self::SavePeople { user, .. }
            | Self::SaveSpouse { user, .. }
            | Self::SaveCommunityProperty { user, .. }
            | Self::SaveSpouseBoxes { user, .. }
            | Self::SaveSpouseItemizes { user, .. }
            | Self::SaveDeductions { user, .. }
            | Self::SaveChildren { user, .. }
            | Self::SaveDocuments { user, .. }
            | Self::SaveIncomes { user, .. }
            | Self::SaveDocumentQuestions { user, .. }
            | Self::SaveWagesAnswers { user, .. }
            | Self::EstimateWagesAnswers { user, .. }
            | Self::DocumentOwnershipOptions { user, .. }
            | Self::SaveHsa { user, .. }
            | Self::SaveCare { user, .. }
            | Self::SaveSelfEmployedBenefits { user, .. }
            | Self::SaveIraContributions { user, .. }
            | Self::SaveEducator { user, .. }
            | Self::SaveStudentLoan { user, .. }
            | Self::SaveEmployeeExpenses { user, .. }
            | Self::SaveMedical { user, .. }
            | Self::SaveTaxes { user, .. }
            | Self::SaveOtherItemized { user, .. }
            | Self::SaveMortgages { user, .. }
            | Self::SaveHomeOffices { user, .. }
            | Self::SaveGiftCarryovers { user, .. }
            | Self::SaveEducation { user, .. }
            | Self::SaveSaver { user, .. }
            | Self::SaveMarketplace { user, .. }
            | Self::SaveMoreCredits { user, .. }
            | Self::SaveMoreAdjustments { user, .. }
            | Self::SaveOtherTaxes { user, .. }
            | Self::SavePayments { user, .. }
            | Self::SaveIncomeByPeriod { user, .. }
            | Self::SaveInjuredSpouse { user, .. }
            | Self::SaveParentTax { user, .. }
            | Self::SaveChecklist { user, .. }
            | Self::SaveOftenMissed { user, .. }
            | Self::SaveIncomeWithoutForm { user, .. }
            | Self::Back { user, .. }
            | Self::Navigate { user, .. }
            | Self::OpenSection { user, .. }
            | Self::OpenDocumentQuestions { user, .. }
            | Self::StartFinish { user, .. }
            | Self::Finish { user, .. }
            | Self::AssetMacrsSourceOptions { user, .. }
            | Self::AssetMacrsClassLifeOptions { user, .. }
            | Self::PropertyMacrsSourceOptions { user, .. }
            | Self::PropertyMacrsClassLifeOptions { user, .. }
            | Self::AssetAmtCarryOptions { user, .. }
            | Self::VehicleAmtCarryOptions { user, .. }
            | Self::CanonicalAssetAmtCarryOptions { user, .. }
            | Self::NonMacrsAssetCarryOptions { user, .. }
            | Self::IntangibleSourceOptions { user, .. }
            | Self::NonMacrsCanonicalCarryOptions { user, .. }
            | Self::RentalRecoveryOptions { user, .. }
            | Self::DocumentPropertyOptions { user, .. }
            | Self::FarmRelatedOptions { user, .. }
            | Self::FarmRentalRelatedOptions { user, .. }
            | Self::HobbyRelatedOptions { user, .. }
            | Self::EquipmentRelatedOptions { user, .. }
            | Self::RoyaltyRelatedOptions { user, .. }
            | Self::BusinessRelatedOptions { user, .. }
            | Self::RentalRelatedOptions { user, .. }
            | Self::AnswerRentalRisk { user, .. }
            | Self::ProjectRelatedForm { user, .. }
            | Self::ViewRelatedForm { user, .. }
            | Self::ViewReviewedRelatedForm { user, .. }
            | Self::PreviewDocument { user, .. }
            | Self::EstimateSetup { user, .. }
            | Self::EstimateYou { user, .. }
            | Self::EstimateSpouse { user, .. }
            | Self::EstimateDocuments { user, .. }
            | Self::EstimateDependents { user, .. }
            | Self::EstimateHsa { user, .. }
            | Self::EstimateSelfEmployedBenefits { user, .. }
            | Self::EstimateIraContributions { user, .. }
            | Self::EstimateEducator { user, .. }
            | Self::EstimateStudentLoan { user, .. }
            | Self::EstimateEmployeeExpenses { user, .. }
            | Self::EstimateDeductions { user, .. }
            | Self::EstimateMedical { user, .. }
            | Self::EstimateTaxes { user, .. }
            | Self::EstimateOtherItemized { user, .. }
            | Self::EstimateMortgages { user, .. }
            | Self::EstimateHomeOffices { user, .. }
            | Self::EstimateGiftCarryovers { user, .. }
            | Self::EstimateCare { user, .. }
            | Self::EstimateEducation { user, .. }
            | Self::EstimateSaver { user, .. }
            | Self::EstimateMarketplace { user, .. }
            | Self::EstimateMoreCredits { user, .. }
            | Self::EstimateMoreAdjustments { user, .. }
            | Self::EstimateOtherTaxes { user, .. }
            | Self::EstimatePayments { user, .. }
            | Self::EstimateIncomeByPeriod { user, .. }
            | Self::EstimateInjuredSpouse { user, .. }
            | Self::EstimateCommunityProperty { user, .. }
            | Self::EstimateSpouseBoxes { user, .. }
            | Self::EstimateSpouseItemizes { user, .. }
            | Self::EstimateParentTax { user, .. }
            | Self::EstimateChecklist { user, .. }
            | Self::EstimateOftenMissed { user, .. }
            | Self::EstimateIncomeWithoutForm { user, .. }
            | Self::HsaOptions { user, .. }
            | Self::CalculateMarketplaceAppeal { user, .. }
            | Self::BusinessCreditElectionOptions { user, .. }
            | Self::ResearchCreditSourceOptions { user, .. }
            | Self::RetainedLossSourceOptions { user, .. }
            | Self::RetainedCharityAppreciationOptions { user, .. }
            | Self::RetainedCapitalSourceOptions { user, .. }
            | Self::InvestmentOriginOptions { user, .. }
            | Self::K1InvestmentOptions { user, .. }
            | Self::Enter { user, .. }
            | Self::AnswerQsbsZone { user, .. }
            | Self::AnswerQsbsValuation { user, .. }
            | Self::AnswerQsbsBoolean { user, .. }
            | Self::AnswerQsbsPriorGain { user, .. }
            | Self::ReviewAccepted { user, .. } => Some(*user),
            _ => None,
        }
    }
}
