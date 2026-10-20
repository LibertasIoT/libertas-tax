// Coverage: Social Security lump-sum election (Form 1040 line 6c, Pub. 915 Worksheets 2–4) —
//   Supported (unit u5-adjust, 2026-09-28). Asked on the Retirement topic's "Social Security paid
//   in 2026 for earlier years" kind (LumpSumElection; absent is Regular). Verified against Pub.
//   915 (2025) Worksheets 2 (after 1993: 50%/85% with $25,000/$32,000 and $9,000/$12,000),
//   3 (before 1994: 50% only) and 4 (the smaller result elected). Rules on both sides:
//   LUMP_SUM_TAXABLE_ABOVE_BENEFITS, LUMP_SUM_REFIGURED_ABOVE_BENEFITS, LUMP_SUM_ELECTION_TWICE;
//   server only LUMP_SUM_YEAR_TWICE. Tests: tests/retirement/unasked_pages.rs
//   social_security_lump_sum_election_by_hand (fixture), lump_sum_election_is_used_only_when_lower,
//   lump_sum_rules_refuse_on_the_server; tests/retirement/retirement.rs (engine).
use crate::FederalStatus;
use crate::kernel::phase;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Taxation of Social Security or equivalent railroad lump sums for earlier years
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SocialElection {
    /// Regular method — include all benefits received this year
    /// Use when there are no earlier-year lump sums, or when you choose not to claim the lump-sum election.
    Regular,
    /// Compare the lump-sum election and use it if taxable benefits are lower
    /// Once made on a filed return, this election can be revoked only with IRS consent. This prototype files nothing.
    Compare {
        /// Earlier-year records, one combined entry for each year
        /// Include every earlier year represented in current lump sums. Do not amend those returns to make this election.
        // Tax structure: one benefit row per distinct year, 1984 through 2025.
        #[libertas_size(min = 1, max = 42)]
        years: Vec<SocialBenefitYear>,
        /// Records cover all lump sums and the same taxpayer or joint filing unit
        /// Compare each benefit statement with the retained returns for those years.
        /// Include both spouses' benefits and prior joint-return facts when applicable. Changed spouse or filing-unit ownership, incomplete records, treaty treatment and separate allocation to different prior returns require further review. No taxes all benefits under the regular 2026 method.
        complete: bool,
    },
}

// The Retirement topic's lump-sum kind (unit u5-adjust): the Compare
// branch of SocialElection as the page asks it; its absence is Regular.
/// Social Security paid in 2026 for earlier years
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LumpSumElection {
    /// Earlier years
    /// One for each earlier year a 2026 lump sum was paid for, with that year's return.
    /// ----
    /// Earlier year
    // Tax structure: one benefit row per distinct year, 1984 through 2025.
    #[libertas_size(min = 1, max = 42)]
    #[libertas_unordered]
    pub years: Vec<SocialBenefitYear>,
    /// These years cover every lump sum paid in 2026, for the same filers as those returns
    /// Compare each benefit statement with the retained returns for those years.
    /// Include both spouses' benefits when you filed those years jointly. Choose No if a year's return or payments are missing, the filers changed, or a treaty applies. No taxes all benefits under the regular 2026 method.
    pub complete: bool,
}
impl LumpSumElection {
    pub(crate) fn election(&self) -> SocialElection {
        SocialElection::Compare { years: self.years.clone(), complete: self.complete }
    }
    pub(crate) fn from_election(election: &SocialElection) -> Option<Self> {
        match election {
            SocialElection::Regular => None,
            SocialElection::Compare { years, complete } => {
                Some(Self { years: years.clone(), complete: *complete })
            }
        }
    }
    // Each year once, then the earlier return's taxable and refigured amounts.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(index) = self
            .years
            .iter()
            .enumerate()
            .position(|(i, y)| self.years[..i].iter().any(|p| p.year == y.year))
        {
            return Some(
                crate::validation::ValidationIssue::new("LUMP_SUM_YEAR_TWICE")
                    .at_field("year")
                    .in_list("years", index),
            );
        }
        self.years
            .iter()
            .enumerate()
            .find_map(|(index, year)| year.issue().map(|issue| issue.in_list("years", index)))
    }
}

/// Earlier-year Social Security or equivalent railroad return and payment records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct SocialBenefitYear {
    /// Year the benefits were for
    // Tax structure: earlier-year taxable Social Security records cover 1984 through 2025.
    #[libertas_number(min = 1984, max = 2025)]
    pub year: u32,
    /// Filing status for that earlier year
    pub status: FederalStatus,
    /// Lived apart from spouse for the entire earlier year, if filing separately
    pub apart: bool,
    /// Lump sum for this year received in 2026, already included in current box 5 amounts
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub received: i64,
    /// Earlier year's combined SSA-1099 and RRB-1099 box 5 benefits
    #[libertas_money("USD", 2)]
    pub benefits: i64,
    /// Other lump sums for this year received after that year but before 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub earlier_lumps: i64,
    /// Adjusted gross income on the earlier return
    #[libertas_money("USD", 2)]
    pub agi: i64,
    /// Taxable Social Security or equivalent railroad benefits included in that return's AGI
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub taxable: i64,
    /// Earlier taxable benefits after any previous lump-sum election recalculation
    /// With no previous lump sum, enter the taxable benefits from the earlier return. Otherwise use the retained earlier election worksheet's refigured taxable benefits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub refigured: i64,
    /// Tax-exempt interest from the earlier return
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt: i64,
    /// Earlier return exclusions and adjustments added back by Publication 915 Worksheet 2 or 3
    /// Combine qualified savings-bond interest, foreign earned/housing income and territorial exclusions; for post-1993 years also include applicable adoption benefits, student-loan interest, tuition/fees and domestic-production deduction. Use amounts actually claimed on the earlier return.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub addbacks: i64,
}
impl SocialBenefitYear {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.taxable != 0 && self.taxable > self.benefits {
            Some(
                crate::validation::ValidationIssue::new("LUMP_SUM_TAXABLE_ABOVE_BENEFITS")
                    .at_field("taxable"),
            )
        } else if self.refigured != 0
            && i128::from(self.refigured)
                > i128::from(self.benefits) + i128::from(self.earlier_lumps)
        {
            Some(
                crate::validation::ValidationIssue::new("LUMP_SUM_REFIGURED_ABOVE_BENEFITS")
                    .at_field("refigured"),
            )
        } else {
            None
        }
    }
    fn valid(&self) -> bool {
        (1984..=2025).contains(&self.year)
            && [
                self.received,
                self.earlier_lumps,
                self.taxable,
                self.refigured,
                self.exempt,
                self.addbacks,
            ]
            .iter()
            .all(|n| (0..=100_000_000_000).contains(n))
            && [self.benefits, self.agi]
                .iter()
                .all(|n| (-100_000_000_000..=100_000_000_000).contains(n))
            && self.taxable <= self.benefits.max(0)
            && self.refigured <= (self.benefits + self.earlier_lumps).max(0)
    }
    pub(crate) fn additional(&self) -> i64 {
        use crate::federal_rules::social_taxable;
        use crate::kernel::ratio::dollars;
        let benefits = dollars(phase::floor(self.benefits + self.earlier_lumps + self.received, 0));
        let other = dollars(self.agi) - dollars(self.taxable) + dollars(self.addbacks);
        let exempt = dollars(self.exempt);
        let revised = if self.year >= 1994 {
            social_taxable(benefits, other, exempt, self.status, self.apart)
        } else {
            let base = match self.status {
                FederalStatus::Joint => crate::year_constants::SOCIAL_SECURITY_BASE_JOINT_DOLLARS,
                FederalStatus::Separate if !self.apart => 0,
                _ => crate::year_constants::SOCIAL_SECURITY_BASE_OTHER_DOLLARS,
            };
            dollars(phase::cap(
                benefits * 50,
                [crate::kernel::ratio::trunc_i64(
                    phase::floor((other + exempt - base) * 100 + benefits * 50, 0),
                    2,
                )],
            ))
        };
        phase::floor(revised - dollars(self.refigured), 0)
    }
}
impl SocialElection {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Regular => true,
            Self::Compare { years, .. } => {
                !years.is_empty()
                    && years
                        .iter()
                        .enumerate()
                        .all(|(i, v)| v.valid() && !years[..i].iter().any(|p| p.year == v.year))
            }
        }
    }
    pub(crate) fn taxable(
        &self,
        benefits: i64,
        other: i64,
        exempt: i64,
        status: FederalStatus,
        apart: bool,
    ) -> i64 {
        use crate::federal_rules::social_taxable;
        use crate::kernel::ratio::dollars;
        let regular = social_taxable(benefits, other, exempt, status, apart);
        match self {
            Self::Regular => regular,
            Self::Compare { years, .. } => {
                let received = dollars(years.iter().map(|v| v.received).sum());
                let current = social_taxable(
                    phase::floor(benefits - received, 0),
                    other,
                    exempt,
                    status,
                    apart,
                );
                phase::cap(
                    regular,
                    [current + years.iter().map(SocialBenefitYear::additional).sum::<i64>()],
                )
            }
        }
    }
}
