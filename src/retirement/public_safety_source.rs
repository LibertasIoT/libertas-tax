// Retired public-safety officer premium source facts and intrinsic validation.
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Retired public-safety officer insurance election
/// Enter premiums once, in your chosen exclusion allocation order. The App shares one $3,000 annual exclusion per officer across all listed plans, then carries remaining eligible medical premiums into itemization. Do not enter these premiums again in Medical, Marketplace or business health insurance. Leave the list empty to make no election. This is separate from the early-distribution public-safety exception. Each record is for a law-enforcement officer, firefighter, chaplain or rescue/ambulance crew member who retired because of disability or at normal retirement age and receives this pension from that employer's governmental plan.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PublicSafetyPremium {
    /// Retirement or disability-pension Income entry ID
    /// The Income entry belongs to the retired officer. A surviving spouse cannot claim the deceased officer's exclusion. Use the plan of the employer from which the officer retired, not an IRA or personally purchased annuity.
    #[libertas_number(min = 1)]
    pub income_id: i64,
    /// Total qualifying premiums paid in 2026 for this record
    /// Coverage is for the officer, spouse or qualifying dependents. Include the full unreimbursed premium, including any portion paid from other taxable funds. Split records when using multiple source distributions; do not repeat the full premium. Exclude reimbursements and payments from other tax-free funds. Allocate each premium dollar only once, even across spouses or policies.
    #[libertas_money("USD", 2)]
    // Copied from the admitted source; no independent derived upper bound.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Portion paid using the linked retirement distribution
    /// Includes direct plan-to-insurer payments and your payments from the distribution. Cannot exceed the total premium or the distribution available after rollovers. Enter zero for other premiums included only to coordinate the insured person's medical deduction.
    #[libertas_money("USD", 2)]
    // Copied from the admitted source; no independent derived upper bound.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub funded: i64,
    /// Coverage and remaining medical deduction
    pub coverage: PublicSafetyCoverage,
}

/// Coverage funded with the retirement distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PublicSafetyCoverage {
    /// Medical, dental, vision or qualifying Medicare coverage
    /// Remaining unreimbursed premiums qualify as medical expenses for the covered people and are added automatically before the 7.5% AGI floor. No Marketplace subsidy or other deduction on these premiums.
    Medical,
    /// Qualified long-term-care insurance
    LongTermCare {
        /// Insured person
        /// The person's identity on the people page: -1 for you, -2 for your spouse, otherwise the dependent's identity.
        person: i64,
    },
    /// Other qualifying accident or health coverage without a medical deduction
    /// Use only when the coverage qualifies for the section 402(l) exclusion but its remaining premiums do not qualify for section 213 medical itemization. Life insurance is not eligible.
    Other,
}

impl PublicSafetyPremium {
    pub(crate) fn valid(&self) -> bool {
        self.income_id > 0
            // Canonical source validity: the same raw-premium arithmetic envelope.
            && (0..=100000000000).contains(&self.premiums)
            && (0..=self.premiums).contains(&self.funded)
            && match &self.coverage {
                PublicSafetyCoverage::LongTermCare { person } => *person != 0,
                _ => true,
            }
    }
}

/// Coverage
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PublicSafetyPaid {
    /// Coverage for you, your spouse or your dependents
    pub coverage: PublicSafetyPaidCoverage,
    /// Premiums paid in 2026
    #[libertas_money("USD", 2)]
    // Arithmetic safety for the current unchecked stored medical/LTC sums: the
    // Data API's 4 MiB value admits at most 4,194,304 one-byte-or-larger rows, so
    // 100b cents per row totals at most 419,430,400,000,000,000 cents, within i64.
    // This is a durable-calculation bound, not a tax or App-request limit; the
    // separate amount sweep must replace it when checked owners cover these sums.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Part paid from this pension
    /// Paid by the plan to the insurer, or by you from this pension.
    #[libertas_money("USD", 2)]
    // The funded <= premiums/gross relationships own this source bound.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub funded: i64,
}
// The 1099-R's last question: each premium row becomes one
// PublicSafetyPremium linked to that 1099-R's income entry; the list's label
// states the officer's eligibility, so a row is the election and the record
// stores no separate answer. An empty list makes no election.
// The stored records of one 1099-R's income entry.
pub(crate) fn premium_records(
    paid: &[PublicSafetyPaid],
    income_id: i64,
) -> Vec<PublicSafetyPremium> {
    paid.iter()
        .map(|p| PublicSafetyPremium {
            income_id,
            premiums: p.premiums,
            funded: p.funded,
            coverage: p.coverage.record(),
        })
        .collect()
}
// The reverse: the records of that entry, in their order.
pub(crate) fn paid_from_records(records: &[&PublicSafetyPremium]) -> Vec<PublicSafetyPaid> {
    records
        .iter()
        .map(|r| PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::of(&r.coverage),
            premiums: r.premiums,
            funded: r.funded,
        })
        .collect()
}
// The server twins: each row's funded part inside its premiums and no
// election on an IRA; with the page's gross, the funded parts together
// inside the distribution (the client rules on PublicSafetyPaid and the 1099-R page).
// These paths are relative to the actual R1099 Page, including its source list.
pub(crate) fn paid_issue(
    paid: &[PublicSafetyPaid],
    ira: bool,
    gross: i64,
) -> Option<crate::validation::ValidationIssue> {
    if paid.is_empty() {
        None
    } else if ira {
        Some(
            crate::validation::ValidationIssue::new("PSO_NOT_FROM_AN_IRA")
                .at_field("public_safety"),
        )
    } else if let Some((index, row)) = paid
        .iter()
        .enumerate()
        .find(|(_, p)| p.funded > p.premiums || p.premiums < 0 || p.funded < 0)
    {
        Some(
            crate::validation::ValidationIssue::new("PSO_FUNDED_ABOVE_PREMIUMS")
                .at_field(if row.premiums < 0 { "premiums" } else { "funded" })
                .in_list("public_safety", index),
        )
    } else if paid
        .iter()
        .try_fold(0i64, |sum, row| sum.checked_add(row.funded))
        .is_none_or(|total| total > gross)
    {
        Some(
            crate::validation::ValidationIssue::new("PSO_FUNDED_ABOVE_GROSS")
                .at_field("public_safety"),
        )
    } else {
        None
    }
}

/// Coverage funded with the retirement distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PublicSafetyPaidCoverage {
    /// Medical, dental, vision or qualifying Medicare coverage
    /// Remaining unreimbursed premiums qualify as medical expenses for the covered people and are added automatically before the 7.5% AGI floor. No Marketplace subsidy or other deduction on these premiums.
    Medical,
    /// Qualified long-term-care insurance
    LongTermCare {
        /// Insured person
        /// Choose the person from the people on this return. The App shares the age limit of this person's other long-term-care records, with the age at the end of 2026 from the people page. Allocate each paid premium dollar to only one source; do not repeat unexcluded premiums here and under a business.
        #[libertas_enum_source("$.people")]
        person: i64,
    },
    /// Other qualifying accident or health coverage without a medical deduction
    /// Use only when the coverage qualifies for the section 402(l) exclusion but its remaining premiums do not qualify for section 213 medical itemization. Life insurance is not eligible.
    Other,
}
impl PublicSafetyPaidCoverage {
    // The stored coverage of this answer.
    fn record(&self) -> PublicSafetyCoverage {
        match self {
            Self::Medical => PublicSafetyCoverage::Medical,
            Self::LongTermCare { person } => PublicSafetyCoverage::LongTermCare { person: *person },
            Self::Other => PublicSafetyCoverage::Other,
        }
    }
    // The page answer of a stored coverage.
    fn of(coverage: &PublicSafetyCoverage) -> Self {
        match coverage {
            PublicSafetyCoverage::Medical => Self::Medical,
            PublicSafetyCoverage::LongTermCare { person } => Self::LongTermCare { person: *person },
            PublicSafetyCoverage::Other => Self::Other,
        }
    }
}
