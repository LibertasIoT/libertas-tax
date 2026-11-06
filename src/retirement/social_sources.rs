use crate::filer_facts::FederalStatus;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Use of Medicare premiums withheld on an accepted benefit statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SocialPremiumAllocation {
    /// Accepted statement identity
    #[libertas_key]
    #[libertas_unique]
    pub document_id: i64,
    /// Where the printed premiums have already been included
    pub usage: SocialPremiumUse,
    /// Part of these premiums reimbursed tax-free from an HSA
    /// [NullText]
    /// Reimbursed portion not answered yet
    /// [SetText]
    /// Actual reimbursed portion, including zero
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reimbursed: Option<i64>,
}

/// Prevent the same Medicare premium from entering two expense sources
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SocialPremiumUse {
    /// None included in a self-employed health plan
    /// The statement premiums enter itemized medical expenses automatically; do not type them again.
    Medical,
    /// Some or all already included in a self-employed health plan
    AlreadyInSelfEmployedPlan {
        /// Business whose health coverage already includes these premiums
        income_id: i64,
        /// Part of the printed premiums already included in that plan's monthly premiums
        /// This is an allocation of an existing expense, not an additional premium.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

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
        #[libertas_size(max = 42)]
        years: Vec<SocialBenefitYear>,
        /// Records cover all lump sums and the same taxpayer or joint filing unit
        /// Compare each benefit statement with the retained returns for those years.
        /// Include both spouses' benefits and prior joint-return facts when applicable. Changed spouse or filing-unit ownership, incomplete records, treaty treatment and separate allocation to different prior returns require further review. No taxes all benefits under the regular 2026 method.
        complete: bool,
    },
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub benefits: i64,
    /// Other lump sums for this year received after that year but before 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub earlier_lumps: i64,
    /// Adjusted gross income on the earlier return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// Taxable Social Security or equivalent railroad benefits included in that return's AGI
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: i64,
    /// Earlier taxable benefits after any previous lump-sum election recalculation
    /// With no previous lump sum, enter the taxable benefits from the earlier return. Otherwise use the retained earlier election worksheet's refigured taxable benefits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub refigured: i64,
    /// Tax-exempt interest from the earlier return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Earlier return exclusions and adjustments added back by Publication 915 Worksheet 2 or 3
    /// Combine qualified savings-bond interest, foreign earned/housing income and territorial exclusions; for post-1993 years also include applicable adoption benefits, student-loan interest, tuition/fees and domestic-production deduction. Use amounts actually claimed on the earlier return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
}

impl SocialElection {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Regular => true,
            Self::Compare { years, complete } => {
                (!*complete || !years.is_empty())
                    && years
                        .iter()
                        .enumerate()
                        .all(|(i, v)| v.valid() && !years[..i].iter().any(|p| p.year == v.year))
            }
        }
    }
}

// The Retirement page wraps the Compare branch; absence selects Regular.
/// Social Security paid in 2026 for earlier years
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LumpSumElection {
    /// Earlier-year amounts printed on this return's benefit statements
    /// Supply only the prior-return facts when comparing the election.
    #[libertas_copy_from("$.source_choices.printed_lump_years")]
    pub printed_years: Vec<PrintedSocialBenefitYearPage>,
    /// Earlier years
    /// One for each earlier year a 2026 lump sum was paid for, with that year's return.
    // Tax structure: one benefit row per distinct year, 1984 through 2025.
    #[libertas_size(max = 42)]
    #[libertas_unordered]
    /// ----
    /// Earlier year
    pub years: Vec<SocialBenefitYear>,
    /// These years cover every lump sum paid in 2026, for the same filers as those returns
    /// Compare each benefit statement with the retained returns for those years.
    /// Include both spouses' benefits when you filed those years jointly. Choose No if a year's return or payments are missing, the filers changed, or a treaty applies. No taxes all benefits under the regular 2026 method.
    pub complete: bool,
}
impl LumpSumElection {
    pub(crate) fn election(&self) -> SocialElection {
        let mut years = self.years.clone();
        for printed in &self.printed_years {
            if let Some(prior) = &printed.prior {
                years.push(prior.year(printed.year, printed.received));
            }
        }
        let complete = self.complete
            && !years.is_empty()
            && self.printed_years.iter().all(|row| row.prior.is_some());
        SocialElection::Compare { years, complete }
    }
    pub(crate) fn from_election(election: &SocialElection) -> Option<Self> {
        match election {
            SocialElection::Regular => None,
            SocialElection::Compare { years, complete } => {
                Some(Self { printed_years: Vec::new(), years: years.clone(), complete: *complete })
            }
        }
    }
    // Each year once, then the earlier return's taxable and refigured amounts.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let SocialElection::Compare { years, .. } = self.election() else {
            return None;
        };
        if let Some(index) =
            years.iter().enumerate().position(|(i, y)| years[..i].iter().any(|p| p.year == y.year))
        {
            return Some(
                crate::validation::ValidationIssue::new("LUMP_SUM_YEAR_TWICE")
                    .at_field("year")
                    .in_list("years", index),
            );
        }
        years
            .iter()
            .enumerate()
            .find_map(|(index, year)| year.issue().map(|issue| issue.in_list("years", index)))
    }
}
/// Prior-return facts used by the Social Security lump-sum owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SocialPriorReturnFacts {
    /// Filing status for that earlier year
    pub status: FederalStatus,
    /// Lived apart from spouse for the entire earlier year, if filing separately
    pub apart: bool,
    /// Earlier year's combined SSA-1099 and RRB-1099 box 5 benefits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub benefits: i64,
    /// Other lump sums for this year received after that year but before 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub earlier_lumps: i64,
    /// Adjusted gross income on the earlier return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// Taxable Social Security or equivalent railroad benefits included in that return's AGI
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: i64,
    /// Earlier taxable benefits after any previous lump-sum election recalculation
    /// With no previous lump sum, enter the taxable benefits from the earlier return. Otherwise use the retained earlier election worksheet's refigured taxable benefits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub refigured: i64,
    /// Tax-exempt interest from the earlier return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt: i64,
    /// Earlier return exclusions and adjustments added back by Publication 915 Worksheet 2 or 3
    /// Combine qualified savings-bond interest, foreign earned/housing income and territorial exclusions; for post-1993 years also include applicable adoption benefits, student-loan interest, tuition/fees and domestic-production deduction. Use amounts actually claimed on the earlier return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub addbacks: i64,
}
impl SocialPriorReturnFacts {
    pub(crate) fn year(&self, year: u32, received: i64) -> SocialBenefitYear {
        SocialBenefitYear {
            year,
            received,
            status: self.status,
            apart: self.apart,
            benefits: self.benefits,
            earlier_lumps: self.earlier_lumps,
            agi: self.agi,
            taxable: self.taxable,
            refigured: self.refigured,
            exempt: self.exempt,
            addbacks: self.addbacks,
        }
    }
    pub(crate) fn from_year(year: &SocialBenefitYear) -> Self {
        Self {
            status: year.status,
            apart: year.apart,
            benefits: year.benefits,
            earlier_lumps: year.earlier_lumps,
            agi: year.agi,
            taxable: year.taxable,
            refigured: year.refigured,
            exempt: year.exempt,
            addbacks: year.addbacks,
        }
    }
}
/// Earlier-year payment identified by the printed benefit statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PrintedSocialBenefitYearPage {
    /// Printed benefit year
    #[libertas_read_only]
    pub year: u32,
    /// Amount received in the current year, from the printed descriptions
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub received: i64,
    /// Prior-return facts for the optional comparison
    /// [NullText]
    /// No prior-return facts supplied: use the regular current-year method
    /// [SetText]
    /// Compare using these prior-return facts
    #[libertas_default($null)]
    pub prior: Option<SocialPriorReturnFacts>,
}
