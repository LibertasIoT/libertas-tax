use crate::clean_vehicle_source::FederalCleanVehicleClaim;
use crate::education_source::EducationStudent;
use crate::mortgage_certificate_source::FederalMortgageCredit;
use crate::mortgage_subsidy_source::MortgageSubsidyEntry;
use crate::personal_refueling_source::FederalPersonalRefueling;
use crate::saver_source::SaverChoice;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Other common credits
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCredits {
    /// You choose not to claim the earned income credit
    #[libertas_default(false)]
    pub decline_earned_income_credit: bool,
    /// 2025 employer tip FICA credit carried to 2026
    /// Enter one retained, nonpassive 2025-origin Form 8846 credit. Other general business credits need separate review.
    pub tip_fica_carryforward: Option<crate::EmployerTipFicaCarryforward>,
    /// General business credits, carryforwards, carrybacks and the married-separate answer
    /// Own source-form credits and supported K-1 credits use Form3800; historical research amounts retain their actual source.
    #[libertas_default($null)]
    pub business_credits: Option<crate::BusinessCreditHistory>,
    /// Form 8978 tax change from one 2026 BBA-audit pushout statement
    /// This supports a 2025 partnership portfolio-interest increase or decrease on a simple 2025 return only. AARs, multiple adjustments, and other affected tax attributes require separate review. Positive tax changes require a documented separate interest computation.
    #[libertas_default($null)]
    pub bba_audit_interest: Option<crate::BbaAuditSource>,
    /// Credit for prior-year minimum tax
    /// Leave empty only if there is no prior AMT or credit history to evaluate.
    #[libertas_default($null)]
    pub minimum_tax: Option<crate::MinimumTaxHistories>,
    /// Mortgage Credit Certificate credit and surviving prior-year balances
    /// Use one qualified certificate for current-year interest. A reissue needs the original mortgage schedule and refinance date; a 2026 reissue also records interest paid before refinancing so each rate period can be calculated. Multiple current certificates need separate review. Enter prior balances from 2025 Form 8396 even if there is no 2026 certificate interest.
    pub mortgage_credit: Option<FederalMortgageCredit>,
    /// Federally subsidized-home dispositions requiring Form 8828 or an explicit exemption review
    /// Give each home a distinct label. A sale is matched to its Sale of your main home entry; a sale can require Form 8828 even with no current Form 8396 credit.
    pub mortgage_subsidies: Vec<MortgageSubsidyEntry>,
    /// Unused District of Columbia first-time homebuyer credit from 2025 Form 8859, line 4
    /// This is a prior credit carryforward, not a new 2026 home-purchase credit. Leave empty if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub dc_homebuyer_carry: Option<i64>,
    /// Unused residential clean energy credit from 2025 Form 5695, line 16
    /// In 2026 only the prior credit carryforward remains; new 2026 purchases do not qualify. Leave empty if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub residential_energy_carry: Option<i64>,
    /// Personal-use alternative-fuel or electric-vehicle refueling property placed in service by June 30, 2026
    /// Enter each qualified single item separately for its own $1,000 credit ceiling; for EV charging, the item is a charging port. Allocate shared installation costs among ports once. Business or mixed-use property belongs to Form 3800 review.
    /// ----
    /// Refueling property
    pub personal_refueling: Vec<FederalPersonalRefueling>,
    /// Clean vehicles acquired by September 30, 2025 and first placed in service in 2026
    /// Use the accepted IRS time-of-sale report for each VIN. A dealer transfer or business-use allocation needs separate return review.
    pub clean_vehicles: Option<FederalCleanVehicleClaim>,
    /// Prior qualified-electric-vehicle passive credit released for 2026 on Form 8582-CR
    /// Enter only the amount allowed for this year, not the original pre-2007 vehicle credit or unreleased carryover. Form 8834 applies a second regular-tax and tentative-minimum-tax limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub legacy_ev_passive_allowed: Option<i64>,
    /// Education students
    /// ----
    /// Education entry
    pub education: Vec<crate::EducationSource>,
    /// Annual education answers, in accepted student order
    pub students: Vec<EducationStudent>,
    /// 2026 refund or tax-free assistance received after filing the 2025 return that reduces its sole education credit
    /// This ordinary path refigures a fully used 2025 American Opportunity or Lifetime Learning credit. Other prior-return histories need a complete refigure.
    #[libertas_default($null)]
    pub education_recapture: Option<crate::PriorYearEducationCreditRecapture>,
    /// Work-related dependent-care credit to evaluate
    pub care: crate::FederalCare,
    /// Saver’s Credit for retirement and ABLE contributions
    pub saver: SaverChoice,
    /// Adoptions
    pub adoptions: Vec<crate::AdoptionChild>,
    /// Prior nonrefundable adoption credit balances
    // Tax structure: adoption credits carry forward for five tax years.
    #[libertas_size(max = 5)]
    pub adoption_carry: Vec<crate::AdoptionCarry>,
    /// Marketplace coverage and advance credit reconciliation
    pub marketplace: crate::MarketplaceClaim,
    /// Foreign tax credit or itemized deduction choice
    pub foreign_tax: crate::ForeignTaxClaim,
    /// Nontaxable pensions for the elderly credit
    /// Enter each source once for filers on this return. Social Security is calculated from the income entries.
    /// ----
    /// Benefit
    pub exempt_pensions: Vec<crate::ExemptPension>,
    /// Qualified tax-credit and holder-credit Build America bonds, reported directly or through an entity
    /// Direct-holder box 1 produces deemed interest; entity reports retain actual designated income and link any already-reported RIC distribution. The App limits Form 8912's Schedule 3 line 6k credit. Enter documented allowed shares when multiple bonds share a partly limited line 12. Original CREBs, pre-October 4, 2008 QZABs and K-1-origin credits retain independently known income while naming the unavailable credit or deduction. Separate coupon interest belongs on the Interest page.
    /// ----
    /// Bond and Form 1097-BTC
    pub tax_credit_bonds: Vec<crate::FederalTaxCreditBond1097>,
    /// Credits claimed after an earlier disallowance (Form 8862)
    /// Answer for each credit the IRS reduced or disallowed for an earlier year. The App fills Form 8862 from the people, children and students already on this return.
    #[libertas_default($null)]
    pub after_disallowance: Option<crate::CreditsAfterDisallowance>,
}

impl FederalCredits {
    pub(crate) fn care_expenses(&self) -> i64 {
        self.care.filer_expenses()
    }
}

/// Credit situation
/// The refusal kinds of More credits as stored: payload-less, so the persisted
/// return carries no page item (a page item's choices of businesses and sales
/// are an EnumSource of the request and cannot be stored).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MoreCreditRefusal {
    /// A selected credit without a 2026 form line, retained for its left-out notice
    OtherCredit,
}
