use crate::federal_rules::valid_date;
use crate::kernel::lines::line_cents;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::{
    FederalDraft, FederalIncome, FederalResult, FederalStatus, Owner, RetirementException,
    RetirementKind, RetirementTreatment,
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Certified long-term-care distribution records
/// Reduces only the additional early-distribution tax, not pension income. The plan must offer this distribution and accept the issuer's annual premium statement. No three-year repayment treatment. Enter any separately deductible premiums in the usual medical or business expense records; this exception does not create a deduction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CareDistribution {
    /// Consistent fictional plan label
    /// Use the same label for every payment from this plan and a different label for another plan. Multiple plans are supported when their combined qualifying amount stays within $2,600; larger cross-plan claims require further review.
    #[libertas_size(min = 1)]
    pub plan: String,
    /// Vested benefit value used by the administrator for the annual distribution limit
    /// Repeat the same annual valuation for payments from this plan. Do not enter an unvested balance or a calculated 10% limit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub balance: i64,
    /// Coverage is for this employee's spouse
    /// Otherwise the employee is the covered person. Spouse coverage receives this exception only on a joint return. No other relative qualifies.
    #[libertas_default(false)]
    pub spouse_coverage: bool,
    /// Annual certified premiums paid by or assessed to this employee for this covered person
    /// Use the issuer's annual premium statement, including eligible certified riders. Repeat the same annual total on additional payments for this person; do not repeat the other spouse's separately allocated premiums. Assessed premiums can qualify even before payment, but that does not establish a medical deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub premiums: i64,
    /// Date the issuer's 2026 premium statement was filed with the plan
    #[libertas_date_only]
    pub statement_filed: u32,
    /// Participating plan and certified insurance documentation confirmed
    /// Check the plan administrator’s confirmation and issuer’s annual premium statement.
    /// No leaves this exception out, so the 10% additional tax applies. Qualified defined-contribution 401(a), 403(a) or 403(b) plan; issuer disclosure acknowledged by IRS and annual statement accepted before payment. All this employee's qualifying payments are entered. No IRA, designated Roth source, commercial annuity, defined-benefit pension or governmental 457 source. This is not a retroactive designation of an ordinary withdrawal from a plan that does not offer this option. Source valuations and premium allocations are confirmed by the administrator and issuer. A payer-reported taxable amount must already include the special section 401(a)(39)(D)/402(l)(3) pretax-first allocation. Calculated partial account-basis withdrawals require that separate allocation before use here.
    pub confirmed: bool,
}
impl CareDistribution {
    pub(crate) fn valid(&self) -> bool {
        !self.plan.trim().is_empty()
            && self.plan.trim() == self.plan
            && (0..=100000000000).contains(&self.balance)
            && (0..=100000000000).contains(&self.premiums)
            && valid_date(self.statement_filed)
    }
}

pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement::BasisResult,
) -> BTreeMap<i64, i64> {
    let mut out = BTreeMap::new();
    let mut entries: Vec<_> = d
        .income
        .iter()
        .filter_map(|e| {
            let FederalIncome::Retirement { data: v } = &e.income else {
                return None;
            };
            let RetirementTreatment::Exception {
                distributed,
                facts: RetirementException::LongTermCare { data },
                ..
            } = &v.ordinary_distribution
            else {
                return None;
            };
            Some((e, v, *distributed, data))
        })
        .collect();
    entries.sort_by_key(|(e, _, date, _)| (*date, e.id));
    let mut plans: BTreeMap<(usize, String), (i64, i64, i64)> = BTreeMap::new();
    let mut premiums: BTreeMap<(usize, bool), (i64, i64, i64)> = BTreeMap::new();
    let mut owner_used = [0, 0];
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint);
    for (e, v, date, data) in entries {
        let owner = usize::from(e.owner == Owner::Spouse);
        if !data.confirmed {
            continue;
        }
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Pension && data.statement_filed <= date,
            &crate::interview::income_issue_context(e),
            "Long-term-care distribution",
            "ISSUE_CONFIRM_A_PARTICIPATING_QUALIFIED_DEFINED_CONTRIBUTION_PLAN_AND_THE_442EED49",
            "Confirm a participating qualified defined-contribution plan and the issuer's certified annual premium statement filed before payment. IRA and ordinary retroactively designated withdrawals do not qualify.",
        );
        let plan = plans.entry((owner, data.plan.clone())).or_insert((data.balance, 0, e.id));
        for id in [plan.2, e.id] {
            crate::require_localized!(
                r,
                plan.0 == data.balance,
                &crate::IssueContext::income_id(id),
                "Long-term-care distribution",
                "ISSUE_REPEAT_ONE_PLAN_S_ANNUAL_VALUATION_CONSISTENTLY_ACROSS_ITS_PAYMENTS_FB3D8571",
                "Repeat one plan's annual valuation consistently across its payments.",
            );
        }
        let premium =
            premiums.entry((owner, data.spouse_coverage)).or_insert((data.premiums, 0, e.id));
        for id in [premium.2, e.id] {
            crate::require_localized!(
                r,
                premium.0 == data.premiums,
                &crate::IssueContext::income_id(id),
                "Long-term-care distribution",
                "ISSUE_USE_ONE_CONSISTENT_ANNUAL_CERTIFIED_PREMIUM_TOTAL_FOR_EACH_COVERED_255AFF79",
                "Use one consistent annual certified premium total for each covered person, not a fresh allowance per payment.",
            );
        }
        // Section 401(a)(39)(D) invokes 402(l)(3): qualifying amounts are
        // taxable amounts, not a pro-rata fraction of the gross payment.
        let taxable = phase::floor(basis.taxable(e.id, v), 0);
        crate::require_localized!(
            r,
            taxable == v.gross
                || matches!(
                    v.taxable,
                    crate::RetirementTaxable::Payer { .. }
                        | crate::RetirementTaxable::FullDischarge { .. }
                ),
            &crate::interview::income_issue_context(e),
            "Long-term-care distribution basis",
            "ISSUE_USE_THE_PAYER_S_TAXABLE_AMOUNT_AFTER_THE_SPECIAL_PRETAX_FIRST_ALLOC_E414AE3F",
            "Use the payer's taxable amount after the special pretax-first allocation, or a fully taxable or completely discharged contract. The ordinary proportional partial-withdrawal worksheet does not perform this allocation.",
        );
        let limit = phase::cap(
            crate::kernel::ratio::trunc_i64(data.balance, 10),
            [crate::year_constants::LONG_TERM_CARE_DISTRIBUTION_LIMIT_CENTS],
        );
        let possible = if data.spouse_coverage && !joint {
            0
        } else {
            phase::cap(
                phase::cap(taxable, [phase::floor(limit - plan.1, 0)]),
                [phase::floor(premium.0 - premium.1, 0)],
            )
        };
        // One person's limit is shared by every plan they take it from, in entry order
        // (section 72(t)(2)(G)(ii); Notice 2025-67).
        let remaining_owner = phase::floor(
            crate::year_constants::LONG_TERM_CARE_DISTRIBUTION_LIMIT_CENTS - owner_used[owner],
            0,
        );
        let eligible = phase::cap(possible, [remaining_owner]);
        plan.1 += eligible;
        premium.1 += eligible;
        owner_used[owner] += eligible;
        out.insert(e.id, eligible);
        emit_lines(
            r,
            &mut (),
            [
                ("limit", limit, msg!("EXPLAIN_CARE_DISTRIBUTION_N_N_LIMIT")),
                ("exception", eligible, msg!("EXPLAIN_CARE_DISTRIBUTION_N_N_EXCEPTION")),
                ("used", plan.1, msg!("EXPLAIN_CARE_DISTRIBUTION_N_N_USED")),
            ],
            |(suffix, amount, note), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("care_distribution_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement"],
                    "https://www.irs.gov/pub/irs-drop/n-26-33.pdf",
                    note,
                ))
            },
        );
    }
    for (owner, used) in owner_used.iter().enumerate() {
        if *used > 0 {
            line_cents(
                r,
                if owner == 0 { "care_owner_taxpayer_used" } else { "care_owner_spouse_used" },
                *used,
                &["income.retirement"],
                "https://www.irs.gov/pub/irs-drop/n-26-33.pdf",
                msg!("EXPLAIN_CARE_OWNER_USED"),
            );
        }
    }
    out
}
