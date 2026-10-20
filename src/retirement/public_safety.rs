// Coverage: retired public safety officer premium exclusion (Form 1040 lines 4c/5c, Pub. 575) —
//   Supported (unit u5-adjust, 2026-09-28). Asked on each 1099-R (last question, "Health insurance
//   premiums for a retired public safety officer"), stored as PublicSafetyPremium rows linked to
//   that 1099-R's entry; the walk erases rows whose 1099-R left the return. Verified against Pub.
//   575 (2025): up to $3,000 a year per officer of premiums paid from an eligible governmental
//   plan distribution, no deduction for the excluded part. Rules on both sides:
//   PSO_NOT_FROM_AN_IRA, PSO_FUNDED_ABOVE_PREMIUMS, PSO_FUNDED_ABOVE_GROSS (a list sum).
//   The RareSituation PublicSafetyPremiums is removed. Tests: tests/retirement/unasked_pages.rs
//   public_safety_premiums_through_the_1099_r_by_hand (fixture),
//   public_safety_exclusion_is_capped_at_3000, public_safety_answer_rules_refuse_on_the_server,
//   removing_the_1099_r_removes_its_premiums.
use crate::kernel::lines::line_cents;
use crate::kernel::phase;
use crate::{FederalDraft, FederalIncome, FederalResult, Owner, RetirementKind};
use alloc::{collections::BTreeMap, format, vec::Vec};
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
    #[libertas_number(min = 0)]
    pub premiums: i64,
    /// Portion paid using the linked retirement distribution
    /// Includes direct plan-to-insurer payments and your payments from the distribution. Cannot exceed the total premium or the distribution available after rollovers. Enter zero for other premiums included only to coordinate the insured person's medical deduction.
    #[libertas_money("USD", 2)]
    // Copied from the admitted source; no independent derived upper bound.
    #[libertas_number(min = 0)]
    pub funded: i64,
    /// Coverage and remaining medical deduction
    pub coverage: PublicSafetyCoverage,
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
    #[libertas_number(min = 0)]
    pub premiums: i64,
    /// Part paid from this pension
    /// Paid by the plan to the insurer, or by you from this pension.
    #[libertas_money("USD", 2)]
    // The funded <= premiums/gross relationships own this source bound.
    #[libertas_number(min = 0)]
    pub funded: i64,
}
// The 1099-R's last question (unit u5-adjust): each premium row becomes one
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
#[derive(Default)]
pub(crate) struct PublicSafetyResult {
    excluded: BTreeMap<i64, i64>,
    pub medical: i64, // Non-LTC medical premiums, in cents.
    // Unexcluded long-term-care premiums by insured person, in cents.
    pub long_term: BTreeMap<i64, i64>,
}
impl PublicSafetyResult {
    pub(crate) fn excluded(&self, id: i64) -> i64 {
        self.excluded.get(&id).copied().unwrap_or(0)
    }
}
pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
) -> Result<PublicSafetyResult, crate::health::CalculationIssue> {
    let mut result = PublicSafetyResult::default();
    let Some(screen) = &d.screening else {
        return Ok(result);
    };
    // Section 402(l): the statutory $3,000 annual exclusion per officer.
    let mut remaining = [300000i64; 2];
    let mut paid: BTreeMap<i64, i64> = BTreeMap::new();
    for (index, premium) in screen.public_safety.iter().enumerate() {
        let source = d.income.iter().find(|e| e.id == premium.income_id);
        let eligible = source.and_then(|e| match &e.income {
            FederalIncome::Retirement { data }
                if data.kind == RetirementKind::Pension
                    && matches!(
                        data.taxable,
                        crate::RetirementTaxable::Payer { .. }
                            | crate::RetirementTaxable::Simplified { .. }
                            | crate::RetirementTaxable::GeneralAnnuity { .. }
                            | crate::RetirementTaxable::QualifiedWithdrawal { .. }
                            | crate::RetirementTaxable::FullDischarge { .. }
                    ) =>
            {
                Some((
                    e.owner,
                    data.taxable_income(),
                    data.disability.amount(),
                    Some(data.gross - data.rolled_amount()),
                ))
            }
            _ => None,
        });
        let Some((owner, taxable, disability, gross)) = eligible else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Public-safety insurance",
                "ISSUE_LINK_EACH_PREMIUM_TO_THE_OFFICER_S_GOVERNMENTAL_PENSION_OR_DISABILI_42B94848",
                "Link each premium to the officer's governmental pension or disability-pension Income entry with a supported taxable-amount calculation. An IRA, commercial annuity or railroad pension does not qualify.",
            );
            continue;
        };
        crate::require_localized!(
            r,
            disability == 0 || disability == taxable,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Public-safety disability allocation",
            "ISSUE_SPLIT_MIXED_QUALIFYING_AND_NONQUALIFYING_SCHEDULE_R_DISABILITY_PAYM_8A5A7A30",
            "Split mixed qualifying and nonqualifying Schedule R disability payments before allocating the insurance exclusion.",
        );
        let context = crate::IssueContext::Section(crate::TaxSection::Retirement)
            .item("public_safety", index);
        let funded = paid.entry(premium.income_id).or_default();
        *funded =
            crate::health::source_amount(funded.checked_add(premium.funded), context.clone())?;
        crate::require_localized!(
            r,
            gross.is_none_or(|g| *funded <= g),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Public-safety premium funding",
            "ISSUE_PREMIUMS_ALLOCATED_TO_ONE_PENSION_CANNOT_EXCEED_ITS_AVAILABLE_DISTR_9D7632D9",
            "Premiums allocated to one pension cannot exceed its available distribution after rollovers.",
        );
        let owner = usize::from(owner == Owner::Spouse);
        let available = crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            taxable,
            result.excluded(premium.income_id),
        )
        .get();
        let excluded = phase::cap(phase::cap(premium.funded, [remaining[owner]]), [available]);
        let prior_excluded = result.excluded.entry(premium.income_id).or_default();
        *prior_excluded =
            crate::health::source_amount(prior_excluded.checked_add(excluded), context.clone())?;
        remaining[owner] = crate::kernel::carry::native_balance(remaining[owner], excluded);
        let residual = crate::kernel::carry::native_balance(premium.premiums, excluded);
        match &premium.coverage {
            PublicSafetyCoverage::Medical => {
                result.medical = crate::health::source_amount(
                    result.medical.checked_add(residual),
                    context.clone(),
                )?;
            }
            PublicSafetyCoverage::LongTermCare { person } => {
                let amount = result.long_term.entry(*person).or_insert(0);
                *amount =
                    crate::health::source_amount(amount.checked_add(residual), context.clone())?;
            }
            PublicSafetyCoverage::Other => (),
        }
        line_cents(
            r,
            &format!("public_safety_{}_excluded", index + 1),
            excluded,
            &["screening.public_safety", "income.retirement"],
            "https://www.irs.gov/publications/p575",
            msg!("EXPLAIN_PUBLIC_SAFETY_N_EXCLUDED"),
        );
    }
    let standalone_care = crate::health::source_amount(
        crate::kernel::ratio::checked_sum(result.long_term.iter().map(|(person, amount)| {
            let age = crate::interview::person_age(d, *person).unwrap_or(0);
            phase::cap(*amount, [crate::health::care_limit(age)])
        })),
        crate::IssueContext::Section(crate::TaxSection::Retirement),
    )?;
    if !screen.public_safety.is_empty() {
        line_cents(
            r,
            "public_safety_medical_before_coordination",
            crate::health::source_amount(
                result.medical.checked_add(standalone_care),
                crate::IssueContext::Section(crate::TaxSection::Retirement),
            )?,
            &["screening.public_safety"],
            "https://www.irs.gov/irb/2025-45_IRB",
            msg!("EXPLAIN_PUBLIC_SAFETY_MEDICAL_BEFORE_COORDINATION"),
        );
    }
    Ok(result)
}
