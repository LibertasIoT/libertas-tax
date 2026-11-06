use crate::accepted_credits::{FederalCredits, MoreCreditRefusal};
use crate::accepted_income::FederalIncomeEntry;
use crate::accepted_screening::FederalScreening;
use crate::checklist_model::{Checklist, OftenMissed};
use crate::{
    FederalAdjustments, FederalDeductions, FederalDependent, FederalPayments, FederalPeople,
    FederalSetup,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Accepted federal return
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalDraft {
    /// Tax year
    pub tax_year: i32,
    /// Next document or person identity
    pub next_id: i64,
    /// Return identity
    pub cookie: String,
    /// Accepted revision
    pub revision: i64,
    /// Return setup
    pub setup: Option<FederalSetup>,
    /// Filer facts
    pub people: Option<FederalPeople>,
    /// Accepted children and dependents
    /// Empty when you support no one; the people page's visit is in finished sections.
    pub dependents: Vec<FederalDependent>,
    /// Accepted income documents
    pub income: Vec<FederalIncomeEntry>,
    /// Accepted answers about income without a form
    /// Absent until this page has been visited and saved.
    pub without_form: Option<crate::IncomeWithoutFormAnswers>,
    /// Homes used by this return's businesses
    pub home_sites: Vec<crate::HomeOfficeSite>,
    /// Finished sections
    pub finished_sections: Vec<crate::TaxSection>,
    /// Documents whose rare situations no saved topic page has answered yet
    /// A Document center Save lists each new paper that can have one; its topic page's Save answers them.
    pub situations_unanswered: Vec<i64>,
    /// Accepted adjustments
    pub adjustments: Option<FederalAdjustments>,
    /// Accepted deductions
    pub deductions: Option<FederalDeductions>,
    /// Accepted common credits
    pub credits: Option<FederalCredits>,
    /// Accepted coverage review
    pub screening: Option<FederalScreening>,
    /// Accepted payments
    pub payments: Option<FederalPayments>,
    /// Review completed
    pub finished: bool,
    /// Credits chosen that are not calculated yet
    pub credit_refusals: Vec<MoreCreditRefusal>,
    /// Situations people often miss
    /// What the last page before the review stores: the answers that plan pages and the situations the App does not prepare.
    pub often_missed: OftenMissed,
    /// Beneficiary-owned Coverdell ESA excess and retained carry
    pub coverdell_excess: Vec<crate::CoverdellExcess>,
    /// Beyond your documents
    /// Every box the person checked; a box with entries is always checked, and a box only a document checks never is.
    pub checklist: Checklist,
    /// Injured spouse allocation (Form 8379)
    pub injured_spouse: Option<crate::InjuredSpouseAllocation>,
}

impl FederalDraft {
    pub(crate) fn empty(cookie: String) -> Self {
        Self {
            tax_year: 2026,
            next_id: 1,
            home_sites: Vec::new(),
            cookie,
            revision: 0,
            setup: None,
            people: None,
            dependents: Vec::new(),
            income: Vec::new(),
            without_form: None,
            finished_sections: Vec::new(),
            situations_unanswered: Vec::new(),
            adjustments: None,
            deductions: None,
            credits: None,
            screening: None,
            // Estimated and extension payments are answered on the federal
            // payments page: a new return starts at zero.
            payments: Some(FederalPayments::none()),
            finished: false,
            credit_refusals: Vec::new(),
            often_missed: crate::OftenMissed::default(),
            coverdell_excess: Vec::new(),
            // The opening checklist starts with nothing ticked.
            checklist: crate::Checklist::NONE,
            injured_spouse: None,
        }
    }
}
