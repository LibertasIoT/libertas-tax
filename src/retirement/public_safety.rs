// Coverage: retired public safety officer premium exclusion (Form 1040 lines 4c/5c, Pub. 575) —
//   Supported (2026-09-28). Asked on each 1099-R (last question, "Health insurance
//   premiums for a retired public safety officer"), stored as PublicSafetyPremium rows linked to
//   that 1099-R's entry; the walk erases rows whose 1099-R left the return. Verified against Pub.
//   575 (2025): up to $3,000 a year per officer of premiums paid from an eligible governmental
//   plan distribution, no deduction for the excluded part. Rules on both sides:
//   PSO_NOT_FROM_AN_IRA, PSO_FUNDED_ABOVE_PREMIUMS, PSO_FUNDED_ABOVE_GROSS (a list sum).
//   The RareSituation PublicSafetyPremiums is removed. Tests: tests/retirement/unasked_pages.rs
//   public_safety_premiums_through_the_1099_r_by_hand (fixture),
//   public_safety_exclusion_is_capped_at_3000, public_safety_answer_rules_refuse_on_the_server,
//   removing_the_1099_r_removes_its_premiums.
use crate::kernel::phase;
use crate::line_emission::line_cents;
use crate::{FederalDraft, FederalIncome, FederalResult, RetirementKind};
use alloc::{collections::BTreeMap, format};

use crate::public_safety_source::PublicSafetyCoverage;

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
            FederalIncome::Retirement { data: source }
                if {
                    let data = &crate::FederalRetirement::from(source);
                    data.kind == RetirementKind::Pension
                        && matches!(
                            data.taxable,
                            Some(crate::RetirementTaxable::Payer)
                                | Some(crate::RetirementTaxable::Simplified { .. })
                                | Some(crate::RetirementTaxable::GeneralAnnuity { .. })
                                | Some(crate::RetirementTaxable::QualifiedWithdrawal { .. })
                                | Some(crate::RetirementTaxable::FullDischarge { .. })
                        )
                } =>
            {
                let data = &crate::FederalRetirement::from(source);
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
        let owner = usize::from(owner == crate::DocumentOwner::Spouses);
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
