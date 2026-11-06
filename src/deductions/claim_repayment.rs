// Coverage: Claim-of-right repayments (IRC 1341, Pub. 525; Schedule A line 17h, Schedule 3
//   line 13b) — Partial (T3-deduct, 2026-09-28). Earlier-year unemployment benefits
//   repaid in 2026, recorded on their income entry (railroad benefits repaid are entered by year
//   on the RRB-1099-R); above $3,000 (recorded_repayments) the
//   deductions page offers "Repaid more than $3,000 of income you reported in an earlier year"
//   (allowed_repayment), whose Claim holds the original-year records and the Automatic, Deduction
//   or Credit method; at or below $3,000 nothing is deductible and the page offers only No. The
//   walk erases a stored comparison when the recorded repayments fall to $3,000 or less. Other
//   repaid income is not entered. Tests: tests/deductions/repayment.rs (the method comparison),
//   tests/deductions/deduction_pages.rs repayment_comparison_through_the_deductions_page (credit 1,100),
//   repayment_comparison_is_offered_only_above_3000.
// Form 8862 is owned by forms/form_8862.rs.
// Unavailable original-year EIC source records leave the historical credit unchanged.

// IRC 1341 and IRS Pub. 525: compare complete current-year methods.
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line;
use crate::{FederalDraft, FederalIncome, FederalResult, FederalStatus, Owner};
use alloc::{collections::BTreeMap, vec::Vec};
const SOURCE: &str = "https://www.irs.gov/publications/p525";

pub use crate::claim_repayment_model::{
    RepaymentAmtLosses, RepaymentCapital, RepaymentClaim, RepaymentDistribution,
    RepaymentDividends, RepaymentEarly, RepaymentEic, RepaymentElderly, RepaymentExclusionBasis,
    RepaymentExclusionLosses, RepaymentMethod, RepaymentRetirement, RepaymentSale, RepaymentSocial,
    RepaymentSsns, RepaymentTaxYear,
};
pub use crate::diagnostics::RepaymentFilers;

fn repayment_context(year: u32, filers: RepaymentFilers) -> crate::IssueContext {
    let section = crate::TaxSection::Deductions;
    crate::IssueContext::Record {
        section,
        identity: crate::federal_model::NoteIdentity::FilingYear {
            section,
            year: year as i32,
            filers,
        },
        label: crate::Message::new(
            "ISSUE_REPAYMENT_SOURCE_YEAR",
            alloc::vec![
                crate::MessageArg::Number(i64::from(year)),
                crate::MessageArg::Resource(
                    match filers {
                        RepaymentFilers::Taxpayer => "PERSON_YOU",
                        RepaymentFilers::Spouse => "PERSON_SPOUSE",
                        RepaymentFilers::Both => "PERSON_YOU_AND_SPOUSE",
                    }
                    .into()
                ),
            ],
        ),
    }
}
impl RepaymentCapital {
    fn exclusion_amounts(
        &self,
        funds: [i64; 3],
        r: &mut FederalResult,
    ) -> Option<([i64; 2], crate::capital::CapitalAmounts)> {
        let Self::Records { exclusion: Some(source), .. } = self else {
            return None;
        };
        let incoming = [dollars(source.short), dollars(source.long)];
        let mut worksheet = self.clone();
        if let Self::Records { sales, .. } = &mut worksheet {
            for record in sales {
                record.sale = record.exclusion_sale();
            }
        }
        Some((incoming, worksheet.amounts(funds, true, r)))
    }

    fn amounts(
        &self,
        distributions: [i64; 3],
        alternative: bool,
        r: &mut FederalResult,
    ) -> crate::capital::CapitalAmounts {
        let [distributions, unrecaptured, collectibles] = distributions;
        let mut short = 0;
        let mut long = 0;
        let mut collectible = 0;
        if let Self::Records { sales, .. } = self {
            for record in sales {
                let sale = &record.sale;
                crate::require_localized!(
                    r,
                    sale.special == crate::SaleSpecial::NoneOfThese,
                    &crate::IssueContext::Section(crate::TaxSection::Deductions),
                    "Repayment capital",
                    "ISSUE_RESOLVE_EACH_ORIGINAL_SALE_S_SPECIAL_TREATMENT_BASIS_AND_WASH_SALE_3483FFBE",
                    "Resolve each original sale's special treatment, basis and wash-sale records for both tax systems.",
                );
                let gain = if alternative { sale.amt_gain() } else { sale.gain() };
                let term = if alternative { sale.amt_term() } else { sale.term };
                if term == crate::GainTerm::Short {
                    short += gain;
                } else {
                    long += gain;
                    if sale.collectible {
                        collectible += gain;
                    }
                }
            }
        }
        crate::capital::CapitalAmounts {
            short: dollars(short),
            long: dollars(long) + distributions,
            collectibles: dollars(collectible) + collectibles,
            unrecaptured,
        }
    }
}

// Earlier-year unemployment and railroad benefits repaid in 2026, as the
// income entries record them: the amount Pub. 525's $3,000 test reads.
pub(crate) fn recorded_repayments(d: &FederalDraft) -> i64 {
    d.income
        .iter()
        .map(|e| match &e.income {
            FederalIncome::GovernmentPayment { data } => crate::FederalUnemployment::from(data)
                .repayments
                .iter()
                .filter(|p| p.received_year < 2026)
                .map(|p| p.amount)
                .sum::<i64>(),
            FederalIncome::Retirement { data: source } => {
                let data = &crate::FederalRetirement::from(source);
                match &data.taxable {
                    Some(crate::RetirementTaxable::Railroad { repayments, .. }) => {
                        repayments.iter().map(|p| p.amount).sum::<i64>()
                    }
                    _ => 0,
                }
            }
            _ => 0,
        })
        .sum()
}
// The deductions page asks for the comparison only above $3,000; at or
// below it no deduction or credit is allowed (Schedule A line 17h).
pub(crate) fn repayment_offered(d: &FederalDraft) -> bool {
    recorded_repayments(d) > 300_000
}

#[derive(Default)]
pub(crate) struct RepaymentResult {
    pub amount: i64,
    pub deduction: i64,
    pub credit: i64,
    pub future_capital: Option<[[i64; 2]; 2]>,
    pub inferred_capital: Vec<crate::PriorCapitalReturn>,
    pub inferred_minimum: Option<crate::MinimumTaxHistory>,
}

struct CapitalHistory {
    year: u32,
    regular: [[i64; 2]; 2],
    amt: [[i64; 2]; 2],
    revised: crate::PriorCapitalReturn,
    permanent_sources: bool,
    exclusion: Option<[[i64; 2]; 2]>,
}

// Source descriptions and grouping do not change the worksheet's amounts.
fn same_minimum_sources(a: &crate::MinimumTaxHistory, b: &crate::MinimumTaxHistory) -> bool {
    a.return_form == b.return_form
        && a.status == b.status
        && a.base == b.base
        && a.regular == b.regular
        && a.amt == b.amt
        && a.carry == b.carry
        && a.gains == b.gains
        && a.regular_nol == b.regular_nol
        && a.mtcnol == b.mtcnol
        && a.foreign_earned == b.foreign_earned
        && a.foreign_credit == b.foreign_credit
        && a.old_vehicle_credit == b.old_vehicle_credit
        && a.exclusions.iter().map(|v| dollars(v.amount)).sum::<i64>()
            == b.exclusions.iter().map(|v| dollars(v.amount)).sum::<i64>()
}

struct MinimumHistory {
    year: u32,
    sources: [crate::MinimumTaxHistory; 2],
}

struct Restoration {
    decrease: i64,
    future: [[i64; 2]; 2],
}

pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> RepaymentResult {
    let claim = &d.deductions.as_ref().unwrap().repayment;
    let RepaymentClaim::Claim { method, years, .. } = claim else {
        return RepaymentResult::default();
    };
    let mut ledger = BTreeMap::<(u32, usize, u8), i64>::new();
    let mut social = 0;
    for e in &d.income {
        match &e.income {
            FederalIncome::GovernmentPayment { data } => {
                for p in &crate::FederalUnemployment::from(data).repayments {
                    if p.received_year < 2026 {
                        *ledger
                            .entry((
                                p.received_year,
                                usize::from(e.owner == crate::DocumentOwner::Spouses),
                                0,
                            ))
                            .or_default() += p.amount;
                    }
                }
            }
            FederalIncome::Retirement { data: source } => {
                let data = &crate::FederalRetirement::from(source);
                if let Some(crate::RetirementTaxable::Railroad { repayments, .. }) = &data.taxable {
                    for p in repayments {
                        *ledger
                            .entry((
                                p.received_year,
                                usize::from(e.owner == crate::DocumentOwner::Spouses),
                                if p.contributory { 1 } else { 2 },
                            ))
                            .or_default() += p.amount;
                    }
                }
            }
            FederalIncome::SocialSecurity { data } => {
                social += crate::FederalSocial::from(data).benefits
            }
            _ => (),
        }
    }
    ledger.retain(|_, v| *v > 0);
    let amount: i64 = ledger.values().sum();
    crate::require_localized!(
        r,
        amount > 300000 && social >= 0,
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Repayment comparison",
        "ISSUE_CONFIRM_CLAIM_OF_RIGHT_ELIGIBILITY_AND_COMPLETE_SUPPORTED_HISTORICA_D6242D7E",
        "The repayment comparison needs more than $3,000 of earlier-year unemployment or railroad repayments. Net Social Security repayments need their own prior-taxability calculation.",
    );
    // Unsupported original-year records: the credit is left out and only the
    // itemized deduction is figured (the method was set to Deduction).
    if !years.iter().all(RepaymentTaxYear::records_supported) {
        return RepaymentResult {
            amount: dollars(amount),
            deduction: dollars(amount),
            ..RepaymentResult::default()
        };
    }
    let mut credit = 0;
    let mut inferred_capital = Vec::new();
    let mut inferred_minimum = None;
    let mut minimum_histories = BTreeMap::<u8, MinimumHistory>::new();
    let people = d.people.as_ref().unwrap();
    // Each filing unit owns its independent actual/refigured regular and AMT
    // balances. Source record order never determines tax-year order.
    let mut ordered: Vec<_> = years.iter().enumerate().collect();
    ordered.sort_by_key(|(_, y)| y.year);
    let mut histories = BTreeMap::<u8, CapitalHistory>::new();
    let mut seen = BTreeMap::<(u32, usize), bool>::new();
    for (index, y) in ordered {
        let context = repayment_context(y.year, y.filers);
        let owners: &[usize] = match y.filers {
            RepaymentFilers::Taxpayer => &[0],
            RepaymentFilers::Spouse => &[1],
            RepaymentFilers::Both => &[0, 1],
        };
        let joint = y.status == FederalStatus::Joint;
        let current_joint = d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint;
        let ownership =
            joint == (y.filers == RepaymentFilers::Both) && (owners == [0] || current_joint);
        crate::require_localized!(
            r,
            ownership,
            &context,
            "Repayment comparison",
            "ISSUE_MATCH_EACH_ORIGINAL_FILING_UNIT_TO_THESE_FILERS_A_PRIOR_JOINT_RETUR_110F45B0",
            "Match each original filing unit to these filers. A prior joint return with a different or absent spouse requires ownership allocation.",
        );
        let mask = owners.iter().fold(0_u8, |mask, i| mask | (1 << i));
        crate::require_localized!(
            r,
            owners.iter().all(|i| seen.insert((y.year, *i), true).is_none()),
            &context,
            "Repayment comparison",
            "ISSUE_ENTER_EACH_PERSON_S_ORIGINAL_RETURN_ONLY_ONCE_PER_YEAR_FA141A9A",
            "Enter each person's original return only once per year.",
        );
        let prior = histories.remove(&mask);
        let prior_minimum = minimum_histories.remove(&mask);
        let incoming = y.capital.incoming(false);
        let amt_incoming = y.capital.incoming(true);
        let mut regular_prior = [incoming; 2];
        let mut amt_prior = [amt_incoming; 2];
        if let Some(history) = &prior {
            let CapitalHistory { year, regular, amt, .. } = history;
            crate::require_localized!(
                r,
                year + 1 == y.year && regular[0] == incoming && amt[0] == amt_incoming,
                &context,
                "Repayment capital",
                "ISSUE_SUPPLY_EVERY_INTERVENING_ORIGINAL_RETURN_AND_RECONCILE_ITS_ACTUAL_R_924EDFC7",
                "Supply every intervening original return and reconcile its actual regular and AMT incoming capital losses. Changed filing-unit ownership needs allocation.",
            );
            regular_prior = *regular;
            amt_prior = *amt;
        }
        let mut removed_by_source = [0_i64; 3];
        for owner in owners {
            let mut available = [0_i64; 2];
            for source in &y.retirement {
                if let RepaymentRetirement::Railroad {
                    owner: recipient,
                    gross,
                    contributory,
                    basis,
                    ..
                } = source
                    && usize::from(*recipient == Owner::Spouse) == *owner
                {
                    if basis.as_ref().is_none_or(|b| b.excluded(*contributory) == 0) {
                        available[0] += contributory;
                    }
                    available[1] += gross - contributory;
                }
            }
            for kind in 0..3 {
                let amount = ledger.remove(&(y.year, *owner, kind as u8)).unwrap_or(0);
                if kind > 0 {
                    crate::require_localized!(
                        r,
                        amount <= available[kind - 1],
                        &context,
                        "Repayment railroad",
                        "ISSUE_MATCH_REPAYMENTS_TO_THE_ORIGINAL_RECIPIENT_AND_BENEFIT_COMPONENT_CO_5A703723",
                        "Match repayments to the original recipient and benefit component. Contributory repayments with tax-free basis recovery require a separate allocation; unrelated pension or unemployment income cannot supply the repaid amount.",
                    );
                }
                removed_by_source[kind] += amount;
            }
        }
        let removed = removed_by_source[0];
        let railroad_removed = removed_by_source[1] + removed_by_source[2];
        crate::require_localized!(
            r,
            (removed + railroad_removed > 0 || prior.is_some() || prior_minimum.is_some())
                && removed <= y.benefits,
            &context,
            "Repayment comparison",
            "ISSUE_EACH_ORIGINAL_RETURN_MUST_COVER_MATCHING_REPAYMENTS_ONCE_OR_CONTINU_978FB550",
            "Each original return must cover matching repayments once or continue an affected loss or minimum-tax-credit history, within the benefits originally included in its income.",
        );
        let mut retirement_gross = [0; 2];
        let mut retirement_taxable = [0; 2];
        for pension in &y.retirement {
            if let RepaymentRetirement::Railroad { owner, .. } = pension {
                crate::require_localized!(
                    r,
                    owners.contains(&usize::from(*owner == Owner::Spouse)),
                    &context,
                    "Repayment railroad",
                    "ISSUE_ORIGINAL_RAILROAD_INCOME_MUST_BELONG_TO_A_FILER_ON_THIS_HISTORICAL_2207B0A5",
                    "Original railroad income must belong to a filer on this historical return.",
                );
            }
            let (gross, taxable, _) = pension.amounts();
            crate::require_localized!(
                r,
                taxable <= gross,
                &context,
                "Repayment retirement",
                "ISSUE_CONFIRM_ORDINARY_ORIGINAL_YEAR_RETIREMENT_SOURCE_RECORDS_INCLUDING_0068D19C",
                "An original-year retirement taxable amount cannot exceed its gross distribution.",
            );
            let category =
                usize::from(!matches!(pension, RepaymentRetirement::TraditionalIra { .. }));
            retirement_gross[category] += gross;
            retirement_taxable[category] += taxable;
        }
        // Combine source cents on each 1040 line before whole-dollar rounding.
        let revised_retirement_taxable = dollars(retirement_taxable[0])
            + dollars(positive(retirement_taxable[1] - railroad_removed));
        let retirement_gross = retirement_gross.map(dollars);
        let retirement_taxable = retirement_taxable.map(dollars);
        let [ira_taxable, pension_taxable] = retirement_taxable;
        let retirement_taxable: i64 = retirement_taxable.iter().sum();
        let [ordinary_dividends, qualified_dividends, distributions, unrecaptured, collectibles] =
            y.dividends.amounts().map(dollars);
        let funds = [distributions, unrecaptured, collectibles];
        let amounts = y.capital.amounts(funds, false, r);
        let alternative_amounts = y.capital.amounts(funds, true, r);
        let separate = y.status == FederalStatus::Separate;
        let capital = regular_prior.map(|prior| amounts.calculate(prior, separate, false));
        let amt_capital =
            amt_prior.map(|prior| alternative_amounts.calculate(prior, separate, false));
        let exclusion_worksheet = y.capital.exclusion_amounts(funds, r);
        let exclusion_capital = exclusion_worksheet.map(|(incoming, amounts)| {
            let mut balances = [incoming; 2];
            if let Some(history) = &prior {
                let retained = history.exclusion.or_else(|| history.permanent_sources.then_some(history.amt));
                crate::require_localized!(r, retained.is_some(), &context, "Repayment minimum-tax credit",
                    "ISSUE_THE_PRECEDING_AFFECTED_LOSS_HISTORY_NEEDS_ITS_EXCLUSION_ONLY_WORKSH_C997588B", "The preceding affected loss history needs its exclusion-only worksheet before continuation.");
                if let Some(retained) = retained {
                    crate::require_localized!(r, retained[0] == incoming, &context, "Repayment minimum-tax credit",
                        "ISSUE_ORIGINAL_EXCLUSION_ONLY_INCOMING_LOSSES_MUST_RECONCILE_WITH_THE_PRE_6E925486", "Original exclusion-only incoming losses must reconcile with the preceding worksheet.");
                    balances = retained;
                }
            }
            balances.map(|v| amounts.calculate(v, separate, false))
        });
        crate::require_localized!(
            r,
            exclusion_capital.is_none() || y.capital_exclusions.is_none(),
            &context,
            "Repayment minimum-tax credit",
            "ISSUE_USE_EITHER_COMPLETE_EXCLUSION_ONLY_CAPITAL_SOURCES_OR_FIXED_CAPITAL_199A71BD",
            "Use either complete exclusion-only capital sources or fixed capital adjustments, never both.",
        );
        crate::require_localized!(
            r,
            prior.as_ref().is_none_or(|v| v.exclusion.is_none()) || exclusion_capital.is_some(),
            &context,
            "Repayment minimum-tax credit",
            "ISSUE_SUPPLY_THE_EXCLUSION_ONLY_WORKSHEET_FOR_EVERY_AFFECTED_CONTINUATION_8243FE90",
            "Supply the exclusion-only worksheet for every affected continuation year.",
        );
        let preferences = [&capital[0], &capital[1]].map(|v| {
            crate::income_tax::Preference {
                dividends: qualified_dividends,
                gain: v.net_gain,
                line18: v.line18,
                line19: v.line19,
            }
            .capped()
        });
        let amt_preferences = [&amt_capital[0], &amt_capital[1]].map(|v| {
            crate::income_tax::Preference {
                dividends: qualified_dividends,
                gain: v.net_gain,
                line18: v.line18,
                line19: v.line19,
            }
            .capped()
        });
        let before_social = [
            dollars(y.wages)
                + dollars(y.interest)
                + dollars(y.benefits)
                + retirement_taxable
                + ordinary_dividends
                + capital[0].income,
            dollars(y.wages)
                + dollars(y.interest)
                + dollars(positive(y.benefits - removed))
                + revised_retirement_taxable
                + ordinary_dividends
                + capital[1].income,
        ];
        let (social_amount, social_taxable, lived_apart) = historical_social(y, before_social, r);
        let agi = before_social[0] + social_taxable[0];
        let after = before_social[1] + social_taxable[1];
        let social_exempt = social_taxable.map(|taxable| positive(social_amount - taxable));
        let base_standard = match (y.year, y.status) {
            (2023, FederalStatus::Joint | FederalStatus::Surviving) => 27700,
            (2024, FederalStatus::Joint | FederalStatus::Surviving) => 29200,
            (2025, FederalStatus::Joint | FederalStatus::Surviving) => 31500,
            (2023, FederalStatus::Head) => 20800,
            (2024, FederalStatus::Head) => 21900,
            (2025, FederalStatus::Head) => 23625,
            (2023, _) => 13850,
            (2024, _) => 14600,
            _ => 15750,
        };
        let senior_owners: Vec<usize> = owners
            .iter()
            .copied()
            .filter(|i| {
                let p = if *i == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
                p.is_some_and(|p| p.birth_date <= (y.year - 64) * 10000 + 101)
            })
            .collect();
        let senior_count = senior_owners.len();
        let owner_mask = owners.iter().fold(0_u8, |mask, i| mask | (1 << i));
        crate::require_localized!(
            r,
            y.ssns.mask().is_none_or(|mask| mask & !owner_mask == 0),
            &context,
            "Repayment comparison",
            "ISSUE_THE_ORIGINAL_YEAR_SSN_SELECTION_MUST_IDENTIFY_ONLY_PEOPLE_INCLUDED_AD8525A8",
            "The original-year SSN selection must identify only people included on that original return.",
        );
        let enhanced_applies =
            y.year >= 2025 && senior_count > 0 && y.status != FederalStatus::Separate;
        // Timely SSNs not supplied: no enhanced senior deduction is assumed.
        if enhanced_applies && y.ssns.mask().is_none() {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_REPAYMENT_SSNS_ASSUMED_NONE"),
                repayment_context(y.year, y.filers),
            );
        }
        let eligible_seniors = if enhanced_applies {
            senior_owners
                .iter()
                .filter(|i| y.ssns.mask().is_some_and(|mask| mask & (1 << **i) != 0))
                .count() as i64
        } else {
            0
        };
        let original_senior =
            crate::extended_rules::senior_deduction(y.status, eligible_seniors, agi);
        let refigured_senior =
            crate::extended_rules::senior_deduction(y.status, eligible_seniors, after);
        let extra_unit =
            match (y.year, matches!(y.status, FederalStatus::Single | FederalStatus::Head)) {
                (2023, true) => 1850,
                (2024, true) => 1950,
                (2025, true) => 2000,
                (2023, false) => 1500,
                (2024, false) => 1550,
                _ => 1600,
            };
        let extra = dollars(y.standard) - base_standard;
        let fixed_standard = extra >= senior_count as i64 * extra_unit
            && extra % extra_unit == 0
            && crate::kernel::ratio::trunc_i64(extra, extra_unit)
                <= (senior_count + owners.len()) as i64;
        crate::require_localized!(
            r,
            fixed_standard,
            &context,
            "Repayment comparison",
            "ISSUE_AN_UNRECONCILED_FIXED_STANDARD_DEDUCTION_REQUIRES_EXPANDED_PRIOR_YE_179655DC",
            "An unreconciled fixed standard deduction requires expanded prior-year calculations. Do not freeze an income-sensitive amount from the filed return.",
        );
        let Some(rates) = crate::income_tax::Rates::with_method(y.year, y.status, true) else {
            crate::require_localized!(
                r,
                false,
                &context.clone().manual(),
                "Repayment comparison",
                "ISSUE_ORIGINAL_YEAR_FILING_TABLES_ARE_NOT_IMPLEMENTED_FOR_THIS_YEAR_A3B10A72",
                "Original-year filing tables are not implemented for this year.",
            );
            continue;
        };
        let taxable = [
            positive(agi - dollars(y.standard) - original_senior),
            positive(after - dollars(y.standard) - refigured_senior),
        ];
        let [original_regular, refigured_regular] = [0, 1].map(|i| {
            let [preferential, collectibles, unrecaptured] = preferences[i];
            rates.schedule_d(
                taxable[i],
                preferential,
                collectibles + unrecaptured,
                unrecaptured,
                collectibles > 0,
            )
        });
        let amt_rates = crate::income_tax::AmtRates::for_year(y.year, y.status).unwrap();
        // Standard deduction and the 2025 enhanced senior deduction are added
        // back under Form 6251. PAB interest was already in Social Security's
        // provisional-income base and must not be counted there a second time.
        let amt_base = [0, 1].map(|i| {
            [agi, after][i] + dollars(y.private_activity_interest) + amt_capital[i].income
                - capital[i].income
        });
        let amti = amt_base.map(|base| amt_rates.income(base));
        let tmt = [0, 1].map(|i| {
            let [preferential, collectibles, unrecaptured] = amt_preferences[i];
            let [regular_preferential, regular_collectibles, regular_unrecaptured] = preferences[i];
            crate::income_tax::AmtTaxWorksheet {
                year: y.year,
                status: y.status,
                base: amt_base[i],
                preferential: preferential - collectibles - unrecaptured,
                regular_ordinary: positive(
                    taxable[i] - regular_preferential + regular_collectibles + regular_unrecaptured,
                ),
                excluded: 0,
                special: [collectibles, unrecaptured],
                foreign: None,
            }
            .tax()
        });
        let amt = [positive(tmt[0] - original_regular), positive(tmt[1] - refigured_regular)];
        let different_sources = incoming != amt_incoming
            || amounts.short != alternative_amounts.short
            || amounts.long != alternative_amounts.long
            || amounts.collectibles != alternative_amounts.collectibles;
        // With identical original bases/balances, differences introduced solely
        // by the known standard/senior deductions remain permanent exclusions.
        // Retain that provenance through the computed carryover chain.
        let permanent_sources = !different_sources
            && exclusion_capital.is_none()
            && prior.as_ref().is_none_or(|v| v.permanent_sources)
            && y.capital_exclusions
                .as_ref()
                .is_none_or(|v| v.iter().all(|v| dollars(v.amount) == 0));
        let exclusion_sources_ready = if exclusion_capital.is_some() {
            true
        } else if y.capital_exclusions.is_some() {
            // A retained original adjustment cannot be frozen when loss
            // utilization changes. Such cases need separate exclusion netting.
            (regular_prior[0] == regular_prior[1] && amt_prior[0] == amt_prior[1])
                || permanent_sources
        } else {
            permanent_sources
        };
        crate::require_localized!(
            r,
            !different_sources || amt == [0; 2] || exclusion_sources_ready,
            &context,
            "Repayment minimum-tax credit",
            "ISSUE_DIFFERENT_REGULAR_AMT_CAPITAL_SOURCES_WITH_AMT_PAYABLE_NEED_PERMANE_97050D2E",
            "Different regular/AMT capital sources with AMT payable need permanent-exclusion source records. Changed incoming losses need independent exclusion-only netting; do not assume every AMT difference is a deferral.",
        );
        let (original_elderly, refigured_elderly) = historical_elderly(
            y,
            senior_count,
            [agi, after],
            [original_regular + amt[0], refigured_regular + amt[1]],
            social_exempt,
            lived_apart,
            r,
        );
        let zero_balance =
            crate::minimum_tax::CreditBalance { excluded: 0, generated: 0, available: 0 };
        let minimum_balance = if let Some(history) = &y.minimum_tax {
            let balance = history.balance_for_year(y.year - 1, y.status);
            crate::require_localized!(
                r,
                balance.is_some(),
                &context,
                "Repayment minimum-tax credit",
                "ISSUE_PROVIDE_COMPLETE_PRECEDING_YEAR_EXCLUSION_AND_RATE_RECORDS_FOR_THIS_5163B161",
                "Provide complete preceding-year exclusion and rate records for this original filing unit. Missing facts and changed credit ownership need their own calculation.",
            );
            balance.unwrap_or(zero_balance)
        } else {
            zero_balance
        };
        let minimum_balances = if let Some(history) = &prior_minimum {
            crate::require_localized!(
                r,
                history.year + 1 == y.year && history.sources[0].status == y.status,
                &context,
                "Repayment minimum-tax credit",
                "ISSUE_SUPPLY_EACH_INTERVENING_RETURN_FOR_THE_SAME_FILING_UNIT_CHANGED_CRE_B1F18F77",
                "Supply each intervening return for the same filing unit; changed credit ownership needs allocation.",
            );
            let balances = history.sources.each_ref().map(|source| {
                let balance = source.balance_for_year(history.year, y.status);
                crate::require_localized!(r, balance.is_some(), &context, "Repayment minimum-tax credit",
                    "ISSUE_THE_PROPAGATED_SOURCE_WORKSHEET_MUST_RESOLVE_ITS_EXCLUSION_TAX_AND_14C93AEC", "The propagated source worksheet must resolve its exclusion tax and ownership before calculating credit use.");
                balance.unwrap_or(zero_balance)
            });
            if let Some(source) = &y.minimum_tax {
                crate::require_localized!(
                    r,
                    same_minimum_sources(source, &history.sources[0]),
                    &context,
                    "Repayment minimum-tax credit",
                    "ISSUE_THE_SEPARATELY_ENTERED_PRECEDING_YEAR_MINIMUM_TAX_HISTORY_CONFLICTS_4A9AE2AC",
                    "The separately entered preceding-year minimum-tax history conflicts with the calculated original worksheet. Omit a duplicate worksheet or reconcile its source facts.",
                );
            }
            balances
        } else {
            [minimum_balance; 2]
        };
        let minimum = [0, 1].map(|i| {
            minimum_balances[i].limit(
                [original_regular - original_elderly, refigured_regular - refigured_elderly][i],
                tmt[i],
            )
        });
        let current_status = d.setup.as_ref().unwrap().filing_status();
        let same_current_unit = y.year == 2025
            && y.status == current_status
            && mask == if current_status == FederalStatus::Joint { 3 } else { 1 };
        let reconcile_current =
            same_current_unit && d.credits.as_ref().unwrap().minimum_tax.is_some();
        if minimum.iter().any(|v| v.carried != 0) || amt != [0; 2] || reconcile_current {
            // These snapshots contain only known permanent adjustments. Distinct
            // capital sources need an exclusion-only worksheet, not an assumption
            // that every AMT difference is a credit-generating deferral.
            let snapshot_ready = exclusion_sources_ready;
            crate::require_localized!(
                r,
                snapshot_ready,
                &context,
                "Repayment minimum-tax credit",
                "ISSUE_CARRIED_MINIMUM_TAX_CREDITS_WITH_DIFFERING_REGULAR_AMT_CAPITAL_SOUR_9A3AD596",
                "Carried minimum-tax credits with differing regular/AMT capital sources require exclusion-only source worksheets before propagation.",
            );
            if snapshot_ready {
                let mut sources = [0, 1].map(|i| crate::MinimumTaxHistory {
                    return_form: crate::MinimumTaxReturnForm::Form1040,
                    status: y.status,
                    base: ([agi, after][i] - dollars(y.standard)) * 100,
                    exclusions: alloc::vec![
                        crate::MinimumTaxExclusion {
                            source: "Form6251 standard deduction".into(),
                            amount: y.standard,
                        },
                        crate::MinimumTaxExclusion {
                            source: "Form6251 specified private-activity interest".into(),
                            amount: y.private_activity_interest,
                        },
                    ],
                    regular: [original_regular, refigured_regular][i] * 100,
                    amt: amt[i] * 100,
                    carry: minimum[i].carried * 100,
                    gains: rates.minimum_tax_gains(taxable[i], preferences[i], amt_preferences[i]),
                    regular_nol: 0,
                    mtcnol: crate::MinimumTaxLoss::None,
                    foreign_earned: None,
                    foreign_credit: crate::MinimumTaxForeignCredit::None,
                    old_vehicle_credit: 0,
                });
                if permanent_sources || exclusion_capital.is_some() {
                    for i in 0..2 {
                        let exclusion_income = exclusion_capital
                            .as_ref()
                            .map_or(amt_capital[i].income, |v| v[i].income);
                        let adjustment = exclusion_income - capital[i].income;
                        if adjustment != 0 {
                            sources[i].exclusions.push(crate::MinimumTaxExclusion {
                                source: "Capital income from retained exclusion-only sources"
                                    .into(),
                                amount: adjustment * 100,
                            });
                        }
                    }
                }
                if let Some(exclusions) = &y.capital_exclusions {
                    for source in &mut sources {
                        source.exclusions.extend(exclusions.iter().cloned());
                    }
                }
                let next_balances =
                    sources.each_ref().map(|source| source.balance_for_year(y.year, y.status));
                crate::require_localized!(
                    r,
                    next_balances.iter().all(Option::is_some),
                    &context,
                    "Repayment minimum-tax credit",
                    "ISSUE_RESOLVE_ALL_EXCLUSION_ONLY_SOURCE_FACTS_BEFORE_CARRYING_GENERATED_O_2FD26EAE",
                    "Resolve all exclusion-only source facts before carrying generated or unused minimum-tax credit.",
                );
                let active = next_balances.iter().any(|v| v.is_some_and(|v| v.available != 0));
                if y.year == 2025 && (active || reconcile_current) {
                    crate::require_localized!(
                        r,
                        same_current_unit,
                        &context,
                        "Repayment minimum-tax credit",
                        "ISSUE_THE_CURRENT_RETURN_MUST_PRESERVE_CREDIT_OWNERSHIP_CHANGED_FILING_UN_91D8920A",
                        "The current return must preserve credit ownership; changed filing units require allocation.",
                    );
                    if same_current_unit {
                        if let Some(record) = &d.credits.as_ref().unwrap().minimum_tax {
                            crate::require_localized!(
                                r,
                                record.single_current_unit(current_status).is_some_and(|record| {
                                    same_minimum_sources(record, &sources[0])
                                }),
                                &context.clone().alternatives(&[
                                    crate::TaxSection::Deductions,
                                    crate::TaxSection::MoreCredits,
                                ]),
                                "Repayment minimum-tax credit",
                                "ISSUE_CURRENT_MINIMUM_TAX_SOURCE_HISTORY_CONFLICTS_WITH_THE_CALCULATED_OR_E3AADAFB",
                                "Current minimum-tax source history conflicts with the calculated original 2025 worksheet.",
                            );
                        } else if active {
                            inferred_minimum = Some(sources[0].clone());
                        }
                    }
                }
                if active {
                    minimum_histories.insert(mask, MinimumHistory { year: y.year, sources });
                }
            }
        }
        let original = original_regular + amt[0] - original_elderly - minimum[0].used;
        let refigured = refigured_regular + amt[1] - refigured_elderly - minimum[1].used;
        crate::require_localized!(
            r,
            original == dollars(y.income_tax),
            &context,
            "Repayment comparison",
            "ISSUE_THE_HISTORICAL_INCOME_DEDUCTIONS_AND_CREDITS_DO_NOT_REPRODUCE_ORIGI_554CD63D",
            "The historical income, deductions and credits do not reproduce original line 22 tax. Reconcile missing income, deductions, credits and tax methods.",
        );
        let (original_eic, refigured_eic) = historical_eic(
            y,
            owners,
            people,
            [agi, after],
            [0, 1].map(|i| positive(capital[i].short + capital[i].long)),
            r,
        );
        let carry = [0, 1].map(|i| {
            crate::extended_rules::capital_carryover(
                capital[i].short,
                capital[i].long,
                positive(-capital[i].income),
                [agi, after][i] - dollars(y.standard),
            )
        });
        let amt_carry = [0, 1].map(|i| {
            crate::extended_rules::capital_carryover(
                amt_capital[i].short,
                amt_capital[i].long,
                positive(-amt_capital[i].income),
                amti[i],
            )
        });
        let exclusion_carry = exclusion_capital.as_ref().map(|v| {
            [0, 1].map(|i| {
                let base = [agi, after][i] + dollars(y.private_activity_interest) + v[i].income
                    - capital[i].income;
                crate::extended_rules::capital_carryover(
                    v[i].short,
                    v[i].long,
                    positive(-v[i].income),
                    amt_rates.income(base),
                )
            })
        });
        for (i, label) in ["original", "refigured"].iter().enumerate() {
            if let (Some(values), Some(carry)) = (&exclusion_capital, &exclusion_carry) {
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("income", values[i].income),
                        ("short_carry", carry[i][0]),
                        ("long_carry", carry[i][1]),
                    ],
                    |(name, value), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("repayment_year_{index}_{label}_exclusion_{name}"),
                            LineValue::MoneyDollars(value),
                            &["deductions.repayment"],
                            "https://www.irs.gov/instructions/i8801",
                            msg!("EXPLAIN_REPAYMENT_YEAR_N_N_EXCLUSION_N"),
                        ))
                    },
                );
            }
        }
        crate::require_localized!(
            r,
            y.year != 2025 || exclusion_carry.is_none_or(|v| v[0] == v[1]),
            &context.clone().manual(),
            "Repayment minimum-tax credit",
            "ISSUE_DIFFERING_EXCLUSION_ONLY_LOSSES_ENTERINGRETURN_YEAR_NEED_RESTORATION_YEAR_DA1A8AC3",
            "Differing exclusion-only losses entering2026 need restoration-year exclusion and future-credit coordination.",
        );
        let snapshot = |i: usize| crate::PriorCapitalReturn {
            owners: match y.filers {
                RepaymentFilers::Taxpayer => crate::PriorReturnOwners::Taxpayer,
                RepaymentFilers::Spouse => crate::PriorReturnOwners::Spouse,
                RepaymentFilers::Both => crate::PriorReturnOwners::Joint,
            },
            injured_owners: None,
            status: y.status,
            agi: [agi, after][i] * 100,
            deductions: y.standard + [original_senior, refigured_senior][i] * 100,
            senior_deduction: [original_senior, refigured_senior][i] * 100,
            short: capital[i].short * 100,
            long: capital[i].long * 100,
            deducted_loss: positive(-capital[i].income) * 100,
            section_1256: 0,
            amt: Some(crate::PriorAmtCapitalReturn {
                taxable_income: amti[i] * 100,
                short: amt_capital[i].short * 100,
                long: amt_capital[i].long * 100,
                deducted_loss: positive(-amt_capital[i].income) * 100,
            }),
            origins: Vec::new(),
            ordinary_history: true,
            investment_parts: if incoming == [0; 2]
                || prior
                    .as_ref()
                    .is_some_and(|v| crate::extended_rules::all_capital_is_investment(&v.revised))
            {
                Some(crate::InvestmentCapitalParts {
                    short: carry[i][0] * 100,
                    long: carry[i][1] * 100,
                    amt: Some(crate::InvestmentAmtCapitalParts {
                        short: amt_carry[i][0] * 100,
                        long: amt_carry[i][1] * 100,
                    }),
                })
            } else {
                None
            },
            niit_difference: Some(0),
            injured_niit_owners: None,
            research_parts: None,
        };
        if y.year == 2025 {
            let mut actual_source = snapshot(0);
            if let Some((index, actual)) = current_capital(d, &actual_source.owners, r) {
                crate::require_localized!(
                    r,
                    actual == [carry[0], amt_carry[0]]
                        && d.screening.as_ref().unwrap().capital_history[index].status == y.status,
                    &context.clone().alternatives(&[
                        crate::TaxSection::Deductions,
                        crate::TaxSection::InvestmentSales,
                    ]),
                    "Repayment capital",
                    "ISSUE_THE_RECORDED_CURRENT_CAPITAL_HISTORY_CONFLICTS_WITH_THE_ACTUAL_PRIOR_YEAR_4845B541",
                    "The recorded current capital history conflicts with the actual 2025 repayment worksheet's regular or AMT loss balance.",
                );
            } else if carry[0] != [0; 2] || amt_carry[0] != [0; 2] {
                // Reuse complete historical facts instead of asking the user
                // to enter the same2025 worksheet twice. Unknown investment
                // attribution still needs current Form8960/4952 source facts.
                // These confirmed historical ordinary sales exclude business,
                // foreign and special sources. With no unknown incoming loss,
                // their actual loss sources establish zero section1411 excluded
                // difference directly; Form4952 portions alone never do so.
                actual_source.niit_difference = if incoming == [0; 2]
                    || prior.as_ref().is_some_and(|v| v.revised.niit_difference == Some(0))
                {
                    Some(0)
                } else {
                    None
                };
                inferred_capital.push(actual_source);
            }
        }
        if carry[0] != carry[1]
            || amt_carry[0] != amt_carry[1]
            || exclusion_carry.is_some_and(|v| v[0] != v[1])
        {
            histories.insert(
                mask,
                CapitalHistory {
                    year: y.year,
                    regular: carry,
                    amt: amt_carry,
                    revised: snapshot(1),
                    permanent_sources,
                    exclusion: exclusion_carry,
                },
            );
        }
        let early = historical_early(y, owners, [agi, after], r);
        let decrease = original + early[0] - original_eic - refigured - early[1] + refigured_eic;
        credit += decrease;
        emit_lines(
            r,
            &mut (),
            [
                ("ordinary_dividends", ordinary_dividends),
                ("qualified_dividends", qualified_dividends),
                ("capital_distributions", distributions),
                ("net_short_capital", capital[0].short),
                ("net_long_capital", capital[0].long),
                ("capital_income", capital[0].income),
                ("original_amt_capital_income", amt_capital[0].income),
                ("original_amt_short_capital", amt_capital[0].short),
                ("original_amt_long_capital", amt_capital[0].long),
                ("original_short_carry", carry[0][0]),
                ("original_long_carry", carry[0][1]),
                ("refigured_short_carry", carry[1][0]),
                ("refigured_long_carry", carry[1][1]),
                ("original_amt_short_carry", amt_carry[0][0]),
                ("original_amt_long_carry", amt_carry[0][1]),
                ("refigured_amt_short_carry", amt_carry[1][0]),
                ("refigured_amt_long_carry", amt_carry[1][1]),
                ("unrecaptured_gain", preferences[0][2]),
                ("refigured_capital_income", capital[1].income),
                ("refigured_amt_capital_income", amt_capital[1].income),
                ("collectibles_gain", preferences[0][1]),
                ("original_agi", agi),
                ("refigured_agi", after),
                ("ira_gross", retirement_gross[0]),
                ("ira_taxable", ira_taxable),
                ("pension_gross", retirement_gross[1]),
                ("pension_taxable", pension_taxable),
                ("refigured_retirement_taxable", revised_retirement_taxable),
                ("retirement_taxable", retirement_taxable),
                ("social_benefits", social_amount),
                ("original_taxable_social", social_taxable[0]),
                ("refigured_taxable_social", social_taxable[1]),
                ("original_amti", amti[0]),
                ("refigured_amti", amti[1]),
                ("original_tentative_minimum_tax", tmt[0]),
                ("refigured_tentative_minimum_tax", tmt[1]),
                ("original_amt", amt[0]),
                ("refigured_amt", amt[1]),
                ("original_early_tax", early[0]),
                ("refigured_early_tax", early[1]),
                ("minimum_exclusion_tax", minimum_balances[0].excluded),
                ("minimum_credit_generated", minimum_balances[0].generated),
                ("minimum_credit_available", minimum_balances[0].available),
                ("refigured_minimum_credit_available", minimum_balances[1].available),
                ("original_minimum_credit", minimum[0].used),
                ("refigured_minimum_credit", minimum[1].used),
                ("original_minimum_credit_carry", minimum[0].carried),
                ("refigured_minimum_credit_carry", minimum[1].carried),
                ("original_tax", original),
                ("refigured_tax", refigured),
                ("original_senior_deduction", original_senior),
                ("refigured_senior_deduction", refigured_senior),
                ("original_elderly_credit", original_elderly),
                ("refigured_elderly_credit", refigured_elderly),
                ("original_eic", original_eic),
                ("refigured_eic", refigured_eic),
                ("decrease", decrease),
            ],
            |(suffix, value), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("repayment_year_{index}_{suffix}"),
                    LineValue::MoneyDollars(value),
                    &["deductions.repayment", "income.unemployment.repayments"],
                    SOURCE,
                    msg!("EXPLAIN_REPAYMENT_YEAR_N_N"),
                ))
            },
        );
    }
    crate::require_localized!(
        r,
        ledger.is_empty(),
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Repayment comparison",
        "ISSUE_PROVIDE_A_COMPLETE_ORIGINAL_RETURN_RECORD_FOR_EVERY_REPAYMENT_YEAR_DDF8E085",
        "Provide a complete original-return record for every repayment year and owner; missing years cannot contribute a zero credit.",
    );
    let restoration =
        if (!histories.is_empty() || !minimum_histories.is_empty()) && r.issues.is_empty() {
            restoration(d, histories, minimum_histories, r)
        } else {
            None
        };
    if let Some(v) = &restoration {
        credit += v.decrease;
    }
    credit = positive(credit);
    let out = RepaymentResult {
        inferred_capital,
        inferred_minimum,
        future_capital: if *method == RepaymentMethod::Credit {
            restoration.map(|v| v.future)
        } else {
            None
        },
        amount: dollars(amount),
        deduction: if *method == RepaymentMethod::Deduction { dollars(amount) } else { 0 },
        credit: if *method == RepaymentMethod::Credit { credit } else { 0 },
    };
    line(
        r,
        "repayment_credit_potential",
        credit,
        &["deductions.repayment"],
        SOURCE,
        msg!("EXPLAIN_REPAYMENT_CREDIT_POTENTIAL"),
    );
    out
}

fn current_capital(
    d: &FederalDraft,
    owners: &crate::PriorReturnOwners,
    r: &mut FederalResult,
) -> Option<(usize, [[i64; 2]; 2])> {
    let records = &d.screening.as_ref().unwrap().capital_history;
    let index = records.iter().position(|v| &v.owners == owners)?;
    let mut isolated = d.clone();
    isolated.screening.as_mut().unwrap().capital_history = alloc::vec![records[index].clone()];
    Some((index, crate::extended_rules::prior_capital(&isolated, r)))
}

// Reuse the full current-return engine with the claim removed. Earlier-year
// repayments have no independent current income effect; removing only those
// records prevents recursion and the missing-claim diagnostic in the comparison.
fn restoration(
    d: &FederalDraft,
    histories: BTreeMap<u8, CapitalHistory>,
    minimum_histories: BTreeMap<u8, MinimumHistory>,
    r: &mut FederalResult,
) -> Option<Restoration> {
    let mut original = d.clone();
    original.deductions.as_mut().unwrap().repayment = RepaymentClaim::None;
    for entry in &mut original.income {
        match &mut entry.income {
            FederalIncome::GovernmentPayment { data } => {
                data.repayments.retain(|v| v.received_year >= 2026)
            }
            FederalIncome::Retirement { data } => data.clear_railroad_repayments(),
            _ => (),
        }
    }
    let mut revised = original.clone();
    for history in histories.into_values() {
        crate::require_localized!(
            r,
            history.year == 2025,
            &crate::IssueContext::Section(crate::TaxSection::Deductions),
            "Repayment capital",
            "ISSUE_PROVIDE_EVERY_AFFECTED_HISTORICAL_RETURN_THROUGH_PRIOR_YEAR_BEFORE_THE_RE_D8060934",
            "Provide every affected historical return through 2025 before the restoration-year comparison; a missing intervening year is not zero income.",
        );
        let records = &original.screening.as_ref().unwrap().capital_history;
        let (found, actual) = current_capital(&original, &history.revised.owners, r)
            .map_or((None, [[0; 2]; 2]), |(index, amounts)| (Some(index), amounts));
        crate::require_localized!(
            r,
            actual == [history.regular[0], history.amt[0]],
            &repayment_context(
                history.year,
                match history.revised.owners {
                    crate::PriorReturnOwners::Taxpayer => RepaymentFilers::Taxpayer,
                    crate::PriorReturnOwners::Spouse => RepaymentFilers::Spouse,
                    crate::PriorReturnOwners::Joint => RepaymentFilers::Both,
                },
            )
            .alternatives(&[crate::TaxSection::Deductions, crate::TaxSection::InvestmentSales]),
            "Repayment capital",
            "ISSUE_THE_CURRENT_RETURN_S_RETAINED_PRIOR_YEAR_CAPITAL_WORKSHEET_MUST_REPRODUCE_A23F3604",
            "The current return's retained 2025 capital worksheet must reproduce the actual regular and AMT balances of the repayment history, for the same filing unit.",
        );
        let mut replacement = history.revised;
        if let Some(index) = found {
            crate::require_localized!(
                r,
                records[index].status == replacement.status,
                &repayment_context(
                    history.year,
                    match replacement.owners {
                        crate::PriorReturnOwners::Taxpayer => RepaymentFilers::Taxpayer,
                        crate::PriorReturnOwners::Spouse => RepaymentFilers::Spouse,
                        crate::PriorReturnOwners::Joint => RepaymentFilers::Both,
                    },
                )
                .alternatives(&[crate::TaxSection::Deductions, crate::TaxSection::InvestmentSales]),
                "Repayment capital",
                "ISSUE_THE_TWO_SOURCE_RECORDS_MUST_IDENTIFY_THE_SAME_PRIOR_YEAR_FILING_STATUS_BCDFFB7F",
                "The two source records must identify the same 2025 filing status.",
            );
            replacement.investment_parts =
                crate::extended_rules::revised_investment_parts(&records[index], &replacement);
            if crate::extended_rules::regular_capital_carry(&records[index])
                == crate::extended_rules::regular_capital_carry(&replacement)
            {
                replacement.research_parts = records[index].research_parts.clone();
            }
            replacement.niit_difference = records[index].niit_difference;
            revised.screening.as_mut().unwrap().capital_history[index] = replacement;
        } else {
            // Actual zero balances need no duplicate current-year loss record.
            // Source sales establish investment attribution when no unknown
            // incoming loss exists; otherwise Form4952 asks for its sources.
            revised.screening.as_mut().unwrap().capital_history.push(replacement);
        }
    }
    if !r.issues.is_empty() {
        return None;
    }
    for history in minimum_histories.into_values() {
        crate::require_localized!(
            r,
            history.year == 2025,
            &crate::IssueContext::Section(crate::TaxSection::Deductions),
            "Repayment minimum-tax credit",
            "ISSUE_SUPPLY_EVERY_AFFECTED_MINIMUM_TAX_CREDIT_CONTINUATION_RETURN_THROUG_449A27CB",
            "Supply every affected minimum-tax-credit continuation return through 2025; a missing year is not zero credit use.",
        );
        original.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(history.sources[0].clone()));
        revised.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(history.sources[1].clone()));
    }
    if !r.issues.is_empty() {
        return None;
    }
    let before = crate::federal_rules::estimate(&original);
    let after = crate::federal_rules::estimate(&revised);
    for result in [&before, &after] {
        if result.state != crate::ResultState::EstimatedForSupportedScope {
            r.issues.extend(result.issues.iter().cloned());
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::Deductions),
                "Repayment capital",
                "ISSUE_BOTH_COMPLETE_RESTORATION_YEAR_CALCULATIONS_MUST_RESOLVE_BEFORE_THE_92B28377",
                "Both complete restoration-year calculations must resolve before the repayment credit or future balances can be reported.",
            );
        }
    }
    if !r.issues.is_empty() {
        return None;
    }
    let amount = |result: &FederalResult, id: &str| {
        result.lines.iter().find(|line| line.id == id).map(|line| dollars(line.amount))
    };
    let (Some(before_tax), Some(after_tax)) =
        (amount(&before, "chapter_one_net_tax"), amount(&after, "chapter_one_net_tax"))
    else {
        crate::require_localized!(
            r,
            false,
            &crate::IssueContext::Section(crate::TaxSection::Deductions),
            "Repayment capital",
            "ISSUE_THE_COMPLETE_RETURN_MUST_PROVIDE_ITS_CHAPTER1_TAX_COMPARISON_F941E6E4",
            "The complete return must provide its Chapter1 tax comparison.",
        );
        return None;
    };
    let minimum_carries =
        [&before, &after].map(|v| amount(v, "minimum_tax_credit_carryforward").unwrap_or(0));
    crate::require_localized!(
        r,
        minimum_carries[0] == minimum_carries[1],
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Repayment minimum-tax credit",
        "ISSUE_A_CHANGED_MINIMUM_TAX_CREDIT_BALANCE_REMAINING_AFTER_THE_RESTORATIO_8190D773",
        "A changed minimum-tax-credit balance remaining after the restoration year requires post-restoration coordination before finalizing this comparison.",
    );
    if !r.issues.is_empty() {
        return None;
    }
    let mut future = [[0; 2]; 2];
    for (i, ids) in [
        ["capital_short_carryforward", "capital_long_carryforward"],
        ["amt_capital_short_carryforward", "amt_capital_long_carryforward"],
    ]
    .iter()
    .enumerate()
    {
        for (j, id) in ids.iter().enumerate() {
            let Some(value) = amount(&after, id) else {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::Deductions),
                    "Repayment capital",
                    "ISSUE_THE_COMPLETE_RETURN_MUST_PROVIDE_BOTH_REGULAR_AND_AMT_FUTURE_CAPITA_EB470ADA",
                    "The complete return must provide both regular and AMT future capital-loss balances.",
                );
                return None;
            };
            future[i][j] = value;
        }
    }
    let decrease = before_tax - after_tax;
    emit_lines(
        r,
        &mut (),
        [
            ("repayment_return_year_original_chapter_one_net_tax", before_tax),
            ("repayment_return_year_refigured_chapter_one_net_tax", after_tax),
            ("repayment_return_year_decrease", decrease),
        ],
        |(id, value), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(value),
                &["deductions.repayment", "screening.capital_history"],
                "https://www.law.cornell.edu/cfr/text/26/1.1341-1",
                msg!("EXPLAIN_CLAIM_REPAYMENT"),
            ))
        },
    );
    Some(Restoration { decrease, future })
}

fn historical_eic(
    y: &RepaymentTaxYear,
    owners: &[usize],
    people: &crate::FederalPeople,
    incomes: [i64; 2],
    capital_gains: [i64; 2],
    r: &mut FederalResult,
) -> (i64, i64) {
    let joint = y.status == FederalStatus::Joint;
    let Some(cutoff) = crate::historical_eic::cutoff(y.year, joint) else {
        return (0, 0);
    };
    // These are the same people composing the 2026 return; their dates of birth
    // also determine their original-year ages. Current residency/SSNs do not.
    let age = owners.iter().any(|i| {
        let p = if *i == 0 { Some(&people.taxpayer) } else { people.spouse.as_ref() };
        p.is_some_and(|p| {
            p.birth_date > (y.year - 65) * 10000 + 1231
                && p.birth_date <= (y.year - 24) * 10000 + 101
        })
    });
    let investment_limit = match y.year {
        2023 => 11000,
        2024 => 11600,
        _ => 11950,
    };
    let all_ssns = y.ssns.mask().map(|mask| owners.iter().all(|i| mask & (1 << i) != 0));
    let possible = [0, 1].map(|i| {
        all_ssns != Some(false)
            && age
            && y.status != FederalStatus::Separate
            && dollars(y.wages) > 0
            && dollars(y.wages) < cutoff
            && incomes[i] < cutoff
            && dollars(y.interest)
                + dollars(y.exempt_interest)
                + dollars(y.dividends.amounts()[0])
                + capital_gains[i]
                <= investment_limit
    });
    let mut original = 0;
    let mut refigured = 0;
    match &y.eic {
        // Not supplied: the credit is assumed unchanged (no increase from
        // excluding the repaid benefits), with a warning where it could change.
        RepaymentEic::NotSupplied => {
            if possible.iter().any(|v| *v) {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_REPAYMENT_EIC_ASSUMED_UNCHANGED"),
                    repayment_context(y.year, y.filers),
                );
            }
        }
        RepaymentEic::Facts { home, independent, allowed, filed_credit } => {
            // Unknown historical facts cannot establish an increase in EIC.
            // Keep the filed credit unchanged on both sides of the comparison;
            // an assumed zero is not a known source-return inconsistency.
            let unknown_ssns = all_ssns.is_none();
            let known_denial = !*home || !*independent || !*allowed || all_ssns == Some(false);
            let warn_unknown = possible.iter().any(|v| *v) || *filed_credit > 0;
            if unknown_ssns && !known_denial {
                if warn_unknown {
                    crate::federal_rules::note_once_scoped(
                        r,
                        msg!("WARN_REPAYMENT_SSNS_ASSUMED_NONE"),
                        repayment_context(y.year, y.filers),
                    );
                }
                let unchanged = dollars(*filed_credit);
                return (unchanged, unchanged);
            }
            if *home && all_ssns == Some(true) && *independent && *allowed {
                [original, refigured] = [0, 1].map(|i| {
                    if possible[i] {
                        crate::historical_eic::amount(y.year, dollars(y.wages), incomes[i], joint)
                            .unwrap()
                    } else {
                        0
                    }
                });
            }
            crate::require_localized!(
                r,
                original == dollars(*filed_credit),
                &repayment_context(y.year, y.filers),
                "Repayment comparison",
                "ISSUE_ORIGINAL_YEAR_EARNED_INCOME_CREDIT_FACTS_AND_PUBLISHED_TABLE_DO_NOT_3B979D7E",
                "Original-year earned-income-credit facts and published table do not reproduce filed Form 1040 line 27. Reconcile the source return and any IRS changes.",
            );
        }
    }
    (original, refigured)
}

fn historical_elderly(
    y: &RepaymentTaxYear,
    seniors: usize,
    incomes: [i64; 2],
    taxes: [i64; 2],
    social_exempt: [i64; 2],
    lived_apart: Option<bool>,
    r: &mut FederalResult,
) -> (i64, i64) {
    // The same Schedule R base, income reduction and tax cap apply in 2023–2025.
    // Unknown separate living arrangements use the largest possible credit only
    // to decide whether a missing source fact matters; never to publish a credit.
    let potential = [0, 1].map(|i| {
        crate::extended_rules::elderly_credit(
            y.status,
            lived_apart != Some(false),
            seniors,
            [0; 2],
            incomes[i],
            social_exempt[i],
        )
    });
    let matters = (0..2).any(|i| phase::cap(potential[i], [taxes[i]]) > 0);
    // A separate return's living arrangement not supplied: assumed together,
    // so no credit in either calculation.
    if matters && y.status == FederalStatus::Separate && lived_apart.is_none() {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("WARN_REPAYMENT_LIVING_APART_ASSUMED"),
            repayment_context(y.year, y.filers),
        );
    }
    let apart = lived_apart == Some(true);
    let exempt = match &y.elderly {
        // Pension records not supplied: no credit in either calculation.
        RepaymentElderly::NotSupplied => {
            if matters {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_REPAYMENT_ELDERLY_ASSUMED_NONE"),
                    repayment_context(y.year, y.filers),
                );
            }
            return (0, 0);
        }
        RepaymentElderly::Facts { exempt_pensions } => {
            dollars(exempt_pensions.iter().map(|p| p.amount).sum())
        }
    };
    let calculate = |i: usize| {
        phase::cap(
            taxes[i],
            [crate::extended_rules::elderly_credit(
                y.status,
                apart,
                seniors,
                [0; 2],
                incomes[i],
                exempt + social_exempt[i],
            )],
        )
    };
    (calculate(0), calculate(1))
}

// Returns the benefits, the original and refigured taxable amounts, and the
// living-apart answer used (resolved here when a separate return's answer
// was not supplied).
fn historical_social(
    y: &RepaymentTaxYear,
    other: [i64; 2],
    r: &mut FederalResult,
) -> (i64, [i64; 2], Option<bool>) {
    let RepaymentSocial::Benefits { amount, filed_taxable, .. } = &y.social else {
        return (0, [0; 2], y.separate_lived_apart);
    };
    let gross = dollars(*amount);
    let taxable_for = |apart: bool| {
        other.map(|other| {
            crate::federal_rules::social_taxable(
                gross,
                other,
                dollars(y.exempt_interest),
                y.status,
                apart,
            )
        })
    };
    // A separate return's living arrangement not supplied: the arrangement
    // that reproduces the filed taxable benefits and, of two, the one with
    // the smaller decrease (the smaller credit), with a warning.
    let lived_apart = match y.separate_lived_apart {
        None if gross > 0 && y.status == FederalStatus::Separate => {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_REPAYMENT_LIVING_APART_ASSUMED"),
                repayment_context(y.year, y.filers),
            );
            let [together, apart] = [false, true].map(taxable_for);
            let filed = dollars(*filed_taxable);
            Some(match (together[0] == filed, apart[0] == filed) {
                (false, true) => true,
                (true, true) => apart[1] > together[1],
                _ => false,
            })
        }
        other => other,
    };
    let taxable = taxable_for(lived_apart == Some(true));
    crate::require_localized!(
        r,
        taxable[0] == dollars(*filed_taxable),
        &repayment_context(y.year, y.filers),
        "Repayment comparison",
        "ISSUE_ORIGINAL_SOCIAL_SECURITY_SOURCE_RECORDS_DO_NOT_REPRODUCE_FORM_1040_3B7DC59A",
        "Original Social Security source records do not reproduce Form 1040 line 6b. Reconcile benefits, exempt interest, filing status and original-year adjustments.",
    );
    (gross, taxable, lived_apart)
}

fn historical_early(
    y: &RepaymentTaxYear,
    owners: &[usize],
    agi: [i64; 2],
    r: &mut FederalResult,
) -> [i64; 2] {
    let mut before = [0; 2];
    let mut medical = [[0; 2]; 2];
    let mut other = [[0; 2]; 2];
    let mut any_early = false;
    let mut any_medical = false;
    for entry in &y.retirement {
        let (treatment, ira) = match entry {
            RepaymentRetirement::TraditionalIra { treatment, .. } => (treatment, true),
            RepaymentRetirement::Pension { treatment, .. } => (treatment, false),
            RepaymentRetirement::Railroad { .. } => continue,
        };
        if let RepaymentDistribution::Early { owner, simple, medical: use_medical } = *treatment {
            any_early = true;
            let owner = usize::from(owner == Owner::Spouse);
            crate::require_localized!(
                r,
                owners.contains(&owner) && (!simple || ira),
                &repayment_context(y.year, y.filers),
                "Repayment early distribution",
                "ISSUE_AN_EARLY_DISTRIBUTION_MUST_BELONG_TO_A_FILER_ON_THIS_ORIGINAL_RETUR_873AF245",
                "An early distribution must belong to a filer on this original return. The SIMPLE 25% rate applies only to an IRA, not an employer-plan pension.",
            );
            let taxable = entry.amounts().1;
            before[owner] += taxable;
            if use_medical {
                any_medical = true;
                medical[owner][usize::from(simple)] += taxable;
            } else {
                other[owner][usize::from(simple)] += taxable;
            }
        }
    }
    let RepaymentEarly::Facts { medical_expenses, filed_tax, .. } = &y.early else {
        crate::require_localized!(
            r,
            !any_early,
            &repayment_context(y.year, y.filers),
            "Repayment early distribution",
            "ISSUE_PROVIDE_COMPLETE_ORIGINAL_FORM_5329_PART_I_AND_MEDICAL_EXPENSE_RECO_F0EA888F",
            "Provide complete original Form 5329 Part I and medical-expense records for the entered early distributions, even when the original tax was zero.",
        );
        return [0; 2];
    };
    crate::require_localized!(
        r,
        *medical_expenses == 0 || any_medical,
        &repayment_context(y.year, y.filers),
        "Repayment early distribution",
        "ISSUE_CONFIRM_ORIGINAL_YEAR_ELIGIBILITY_RATE_COMPLETE_EXCEPTION_FACTS_AND_0E6D0500",
        "Medical expenses on the original Form 5329 require a selected medical-exception distribution.",
    );
    let tax = agi.map(|agi| {
        let remaining =
            crate::retirement_return::medical_exception_allocation(medical, *medical_expenses, agi);
        (0..2)
            .map(|owner| {
                crate::retirement_return::early_tax_lines(
                    before[owner],
                    [other[owner][0] + remaining[owner][0], other[owner][1] + remaining[owner][1]],
                )[3]
            })
            .sum()
    });
    crate::require_localized!(
        r,
        tax[0] == dollars(*filed_tax),
        &repayment_context(y.year, y.filers),
        "Repayment early distribution",
        "ISSUE_THE_ORIGINAL_RETIREMENT_SOURCES_AND_EXCEPTIONS_DO_NOT_REPRODUCE_THE_90845CAF",
        "The original retirement sources and exceptions do not reproduce the filed early-distribution tax. Reconcile each original Form 5329 before calculating the repayment credit.",
    );
    tax
}
