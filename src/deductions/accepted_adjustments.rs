use crate::common::Owner;
use crate::educator_source::{EducatorClaim, EducatorRole};
use crate::hsa_source::FederalHsa;
use crate::ira_contribution_sources::FederalIras;
use crate::student_loan_source::StudentLoanClaim;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Adjustments to income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalAdjustments {
    /// Original custodian Forms 5498-SA
    pub health_statements: Vec<crate::HealthStatementSource>,
    /// Actual nonprinted contribution classifications keyed by statement identity
    pub health_contribution_allocations: Vec<crate::HealthContributionAllocation>,
    /// Keyed allocations of printed Social Security and railroad Medicare premiums
    pub social_premiums: Vec<crate::SocialPremiumAllocation>,
    /// W-2 employer HSA and Archer MSA contribution-year allocations
    /// Actual accepted source identities, designated years and source amounts; no duplicate employer payment entry.
    pub health_employer_w2: Vec<crate::HealthEmployerBinding>,
    /// Shared Marketplace policy assignments and health calculation method
    pub health_marketplace: crate::HealthMarketplace,
    /// Social Security lump-sum election
    pub social_election: crate::SocialElection,
    /// Alimony paid under divorce or separation instruments
    /// Leave empty if none. Enter each payment obligation once; the App determines whether pre-2019 taxable/deductible treatment applies. The recipient's tax ID is required when filing, but do not enter real identifiers in this prototype.
    pub alimony_paid: Vec<crate::AlimonyPaid>,
    /// Moves under military orders or an intelligence-community reassignment
    /// ----
    /// Move
    pub moving: Vec<crate::MovingClaim>,
    /// Student-loan interest to evaluate
    pub student_loan: StudentLoanClaim,
    /// Educator expenses to evaluate
    pub educators: EducatorClaim,
    /// Forms 2106
    /// ----
    /// Form 2106
    pub employee_expenses: Vec<crate::EmployeeExpenseClaim>,
    /// Health savings accounts
    /// ----
    /// Hsa entry
    pub hsa: Vec<FederalHsa>,
    /// Family contribution method and agreed shares, if used
    /// Choose an exact monthly family allocation or the December family method. Joint returns use both HSA entries; separate returns supply spouse coverage only. Joint monthly contributions may leave this absent when the existing allocation confirmation suffices.
    pub hsa_family_share: Option<crate::HsaFamilyShare>,
    /// Archer MSA contributions and household coverage
    pub archer: crate::ArcherContributions,
    /// Direct IRA gifts to charities
    // Tax structure: one qualified charitable distribution summary per filer.
    #[libertas_size(max = 2)]
    pub qualified_charity: Vec<crate::QualifiedCharity>,
    /// Prior nondeductible traditional IRA contributions
    // Tax structure: one IRA basis summary per filer.
    #[libertas_size(max = 2)]
    pub ira_basis: Vec<crate::IraBasis>,
    /// Required retirement distributions and shortfall relief
    pub required_distributions: Vec<crate::RequiredDistribution>,
    /// Roth IRA source history for withdrawals
    // Tax structure: one Roth history per filer.
    #[libertas_size(max = 2)]
    pub roth_history: Vec<crate::RothHistory>,
    /// Employee-funded section 501(c)(18)(D) trusts
    // Tax structure: one pension contribution summary per filer.
    #[libertas_size(max = 2)]
    /// ----
    /// Annual contribution record
    pub pension_contributions: Vec<crate::PensionContributions>,
    /// Traditional and Roth IRA contributions
    /// Zero amounts and no records when there are none.
    pub iras: FederalIras,
}

impl FederalAdjustments {
    pub(crate) fn student_loan_interest(&self) -> i64 {
        let claim = &self.student_loan;
        claim.student_loan_interest
            + claim.forms.iter().map(|source| source.qualified_interest()).sum::<i64>()
    }
    pub(crate) fn student_loan_employer_paid(&self) -> i64 {
        self.student_loan.employer_paid_interest
    }
    #[cfg(test)]
    pub(crate) fn set_student_loan_interest(&mut self, value: i64) {
        self.student_loan.student_loan_interest = value;
    }
}

impl FederalAdjustments {
    pub(crate) fn educator_itemized_only(&self, owner: Owner) -> i64 {
        match owner {
            Owner::Taxpayer => self.educators.taxpayer.as_ref(),
            Owner::Spouse => self.educators.spouse.as_ref(),
        }
        .map_or(0, EducatorRole::itemized_only)
    }
    pub(crate) fn educator_taxpayer(&self) -> i64 {
        self.educators.taxpayer.as_ref().map_or(0, EducatorRole::classroom)
    }
    #[cfg(test)]
    pub(crate) fn set_educator_taxpayer(&mut self, value: i64) {
        self.educators.taxpayer =
            Some(EducatorRole::Educator { classroom: value, instructional: 0 });
    }
    pub(crate) fn educator_spouse(&self) -> i64 {
        self.educators.spouse.as_ref().map_or(0, EducatorRole::classroom)
    }
}
impl FederalAdjustments {
    /// The IRA facts when any is entered; with none the IRA rules have
    /// nothing to compute.
    pub(crate) fn entered_iras(&self) -> Option<&FederalIras> {
        self.iras.entered().then_some(&self.iras)
    }
}
