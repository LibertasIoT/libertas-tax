use crate::validation::ValidationIssue;
// Coverage: section175 direct source fitting (2026-10-11), partial.
// Actual plan/work/land-use and original-election/approved-change facts replace
// the former whole-record eligibility switch. Current costs, district assessment
// years and retained regular/AMT carry link to accepted parcels of their own farm.
// The existing taxpayer-wide25% pool allocates by eligible activity cost with
// stable accepted-ID exact-cent remainder; this split is a bookkeeping convention.
// Separate AMT carries feed the existing business/rental AMT income collectors.
// F12/4835-10 is tagged. Tests: src/tests/farm/farm.rs, src/tests/farm/farm_rental.rs, src/tests/farm/farm_specials.rs;
// Pub225(2025)ch5;26CFR1.175-4/-5/-6. Published1570/2400assessment and8000/2000
// limit/carry oracles remain unchanged; additional ownership/AMT probes are derived.
// Remaining work: economic production-gross source classification, supported
// current-sale basis/1252 allocation, capitalization-method scope and the distinct
// AMT prepaid denominator. The last combination names a required-tax boundary.
// No whole-Farm scope or native-client acceptance is claimed by these checkpoints.
use crate::kernel::lines::line_cents;
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::{FederalDraft, FederalResult};
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use libertas_macros::*;

/// Conservation payment record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ConservationCost {
    /// Accepted farm land parcel
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Actual conservation plan and work records
    pub facts: crate::FarmConservationFacts,
    /// Work or payment description
    #[libertas_size(min = 1)]
    pub label: String,
    /// Cost paid or incurred under your farm accounting method
    /// Exclude reimbursements, amounts deducted elsewhere, ordinary maintenance and separately depreciable assets.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Date paid or incurred under your method, when needed for a current land sale
    /// Leave absent if the day is not established. A sale of this parcel needs its actual pre-sale and post-sale costs.
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $crate::year_constants::TAX_YEAR_START_DATE, max = $crate::year_constants::TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub paid_or_incurred: Option<u32>,
}

/// District assessment payment or accrual
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ConservationAssessmentPayment {
    /// Tax year paid or incurred under the farm's accounting method
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Amount for this property's depreciable assessment
    /// Exclude interest, ordinary maintenance and nondepreciable work reported elsewhere.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Current-year payment or accrual date, when this parcel is sold
    /// Historical records need only their actual tax year. Do not invent a day for an earlier year.
    /// [NullText]
    /// No current-year parcel-sale payment date is needed
    /// [SetText]
    /// The 2026 assessment payment date is
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $crate::year_constants::TAX_YEAR_START_DATE, max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub current_date: Option<u32>,
}

/// Conservation district's depreciable property assessment
/// One record per property assessment, combining all installments. Use the district's original complete assessment and all payment/accrual records through 2026; do not enter a computed deductible amount. No revised assessment, reimbursement or land disposal is handled here. A final return uses the linked farm owner's death facts to accelerate eligible prior-year assessment balances.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ConservationAssessment {
    /// Accepted farm land parcel
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Actual conservation plan and work records
    pub facts: crate::FarmConservationFacts,
    /// District property or project
    #[libertas_size(min = 1)]
    pub label: String,
    /// Total assessment against all district members for this property
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub total: i64,
    /// Your complete assessment for this property, including future installments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub share: i64,
    /// Payments or accruals under your accounting method
    /// Cash filers enter each paid installment; accrual filers enter the amount in its proper incurred year, not again when paid. The App combines entries for the same year.
    #[libertas_size(min = 1)]
    pub payments: Vec<ConservationAssessmentPayment>,
}
impl ConservationAssessment {
    pub(crate) fn relationships_issue(&self) -> Option<ValidationIssue> {
        if self.share > self.total {
            return Some(ValidationIssue::new("CONSERVATION_SHARE_ABOVE_TOTAL").at_field("share"));
        }
        if self.payments.iter().map(|p| i128::from(p.amount)).sum::<i128>() > i128::from(self.share)
        {
            return Some(
                ValidationIssue::new("CONSERVATION_PAYMENTS_ABOVE_SHARE").at_field("payments"),
            );
        }
        None
    }
    fn valid(&self) -> bool {
        !self.label.is_empty()
            && self.facts.valid()
            && (1..=100000000000).contains(&self.property)
            && (1..=100000000000).contains(&self.total)
            && self.share >= 1
            && self.relationships_issue().is_none()
            && (1..=400).contains(&self.payments.len())
            && self.payments.iter().all(|p| {
                (1900..=crate::year_constants::TAX_YEAR).contains(&p.year)
                    && (0..=100000000000).contains(&p.amount)
                    && p.current_date.is_none_or(|date| {
                        p.year == crate::year_constants::TAX_YEAR && current_date(date)
                    })
            })
    }
    // Current deduction before the 25% limit, future deemed payments, and
    // current capitalized share. Future unpaid installments are not deducted.
    #[cfg(test)]
    pub(crate) fn amounts(&self) -> [i64; 3] {
        self.amounts_for(false)
    }
    fn amounts_for(&self, deceased: bool) -> [i64; 3] {
        let eligible_share = phase::cap(self.share, [fraction(self.total, 10, 100)]);
        let annual = fraction(eligible_share, 10, 100);
        let mut years = BTreeMap::<u32, i64>::new();
        for payment in &self.payments {
            *years.entry(payment.year).or_default() += payment.amount;
        }
        let mut result = [0; 3];
        let mut paid = 0;
        for (year, amount) in years {
            let before = fraction(paid, eligible_share, self.share.max(1));
            paid += amount;
            let allowed = fraction(paid, eligible_share, self.share.max(1)) - before;
            let spread = if allowed > annual + 50000 { allowed - annual } else { 0 };
            let elapsed = 2026 - year;
            if elapsed == 0 {
                result[0] += allowed - spread;
                result[2] += amount - allowed;
            } else if elapsed <= 9 {
                result[0] += fraction(spread, i64::from(elapsed), 9)
                    - fraction(spread, i64::from(elapsed - 1), 9);
            }
            let remaining = crate::kernel::carry::native_balance(
                spread,
                fraction(spread, i64::from(phase::cap(elapsed, [9])), 9),
            );
            if deceased && (1..=9).contains(&elapsed) {
                // Section 175(f)(3): death during the nine succeeding years.
                result[0] += remaining;
            } else {
                result[1] += remaining;
            }
        }
        result
    }
}

/// Soil and water conservation treatment
/// Keep these payments outside ordinary business expenses. Deducting under section 175 is an accounting-method choice, not an annual choice to optimize tax; changing an adopted method generally requires IRS approval.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmConservation {
    /// Capitalize into land basis
    /// No current deduction. Keep the displayed addition with the land's basis records. Do not use this branch to abandon an existing deduction method without required approval.
    Capitalize {
        /// Original capitalization history or actual IRS-approved change
        method: FarmConservationCapitalMethod,
        /// Current capital costs
        costs: Vec<ConservationCost>,
    },
    /// Deduct under a valid section 175 election
    Deduct {
        /// Current qualifying conservation costs
        costs: Vec<ConservationCost>,
        /// District assessments for depreciable conservation property
        /// Nondepreciable district work belongs in current qualifying costs above. Do not repeat any payment in both lists or in ordinary business expenses.
        assessments: Vec<ConservationAssessment>,
        /// Prior unused 25-percent-limited costs, by parcel and source year
        /// Exclude costs already deducted, included in an NOL or capitalized in basis.
        carry: Vec<crate::FarmConservationCarry>,
        /// Original election or IRS-approved change to the section 175 deduction method
        method: FarmConservationMethod,
    },
}
/// Actual capitalization-method history for these conservation costs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationCapitalMethod {
    /// This taxpayer has never adopted a section 175 deduction method
    /// Check prior original returns for all your farms. A continuing deduction election cannot be abandoned by selecting capitalization for a new project.
    NeverElected,
    /// IRS approved capitalization for this farm or project
    ApprovedChange {
        /// Effective year covered by the IRS consent
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
        /// Actual consent and farm/project scope from your records
        #[libertas_size(min = 1)]
        approval: String,
    },
    /// Original method or required consent is not established
    /// No current deduction is claimed. A sale needs the actual method and adjusted parcel basis; no basis treatment is guessed.
    Unestablished,
}
impl FarmConservationCapitalMethod {
    fn valid(&self) -> bool {
        match self {
            Self::ApprovedChange { year, approval } => {
                (1954..=crate::year_constants::TAX_YEAR).contains(year) && !approval.is_empty()
            }
            _ => true,
        }
    }
    pub(crate) fn established(&self) -> bool {
        !matches!(self, Self::Unestablished)
    }
}
/// Section 175 election and method history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationMethod {
    /// Deduction method elected on the original return for this year and continued since
    OriginalElection {
        /// Original election year
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
    },
    /// IRS-approved change to the deduction method
    ApprovedChange {
        /// Effective year of the approved change
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
        /// IRS approval reference retained with your records
        #[libertas_size(min = 1)]
        approval: String,
    },
    /// Election or required IRS approval not established; omit only this deduction
    Unestablished,
}
impl FarmConservationMethod {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::OriginalElection { year } => {
                (1954..=crate::year_constants::TAX_YEAR).contains(year)
            }
            Self::ApprovedChange { year, approval } => {
                (1954..=crate::year_constants::TAX_YEAR).contains(year) && !approval.is_empty()
            }
            Self::Unestablished => true,
        }
    }
    fn established(&self) -> bool {
        !matches!(self, Self::Unestablished)
    }
}
impl crate::FarmConservationFacts {
    pub(crate) fn valid(&self) -> bool {
        self.plan_reference.as_ref().is_none_or(|value| !value.is_empty())
            && (matches!(self.plan, crate::FarmConservationPlan::None)
                || self.plan_reference.is_some())
    }
    pub(crate) fn qualifies(&self) -> bool {
        match (&self.plan, &self.work) {
            (
                crate::FarmConservationPlan::Nrcs | crate::FarmConservationPlan::ComparableState,
                crate::FarmConservationWork::QualifyingLandWork,
            ) => self.land_farmed,
            (
                crate::FarmConservationPlan::EndangeredRecovery,
                crate::FarmConservationWork::EndangeredRecovery,
            ) => true,
            _ => false,
        }
    }
}
impl FarmConservation {
    pub(crate) fn plan_reference_issue(&self) -> Option<ValidationIssue> {
        let missing = |facts: &crate::FarmConservationFacts| {
            !matches!(facts.plan, crate::FarmConservationPlan::None)
                && facts.plan_reference.is_none()
        };
        for (i, cost) in self.costs().iter().enumerate() {
            if missing(&cost.facts) {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_PLAN_REQUIRED")
                        .at_field("plan_reference")
                        .in_field("facts")
                        .in_list("costs", i),
                );
            }
        }
        if let Self::Deduct { assessments, .. } = self {
            for (i, assessment) in assessments.iter().enumerate() {
                if missing(&assessment.facts) {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PLAN_REQUIRED")
                            .at_field("plan_reference")
                            .in_field("facts")
                            .in_list("assessments", i),
                    );
                }
            }
        }
        None
    }

    fn costs(&self) -> &[ConservationCost] {
        match self {
            Self::Capitalize { costs, .. } | Self::Deduct { costs, .. } => costs,
        }
    }
    fn paid(&self) -> i64 {
        self.costs().iter().filter(|cost| land_capital_cost(cost)).map(|c| c.amount).sum()
    }
    fn available(&self) -> i64 {
        self.available_for(false)
    }
    fn available_for(&self, deceased: bool) -> i64 {
        self.available_for_tax(deceased, false)
    }
    fn available_for_tax(&self, deceased: bool, amt: bool) -> i64 {
        match self {
            Self::Deduct { carry, assessments, method, .. } if method.established() => {
                self.costs().iter().filter(|c| c.facts.qualifies()).map(|c| c.amount).sum::<i64>()
                    + carry
                        .iter()
                        .map(|record| if amt { record.amt } else { record.regular })
                        .sum::<i64>()
                    + assessments
                        .iter()
                        .filter(|a| a.facts.qualifies())
                        .map(|a| a.amounts_for(deceased)[0])
                        .sum::<i64>()
            }
            _ => 0,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.costs().len() <= 400
            && self.costs().iter().all(|c| {
                !c.label.is_empty()
                    && (0..=100000000000).contains(&c.amount)
                    && c.facts.valid()
                    && (1..=100000000000).contains(&c.property)
                    && c.paid_or_incurred.is_none_or(current_date)
            })
            && match self {
                Self::Deduct { carry, assessments, method, .. } => {
                    method.valid()
                        && carry.len() <= 400
                        && carry.iter().all(|record| {
                            record.property > 0
                                && record.property <= 100000000000
                                && (1954..crate::year_constants::TAX_YEAR).contains(&record.year)
                                && (0..=100000000000).contains(&record.regular)
                                && (0..=100000000000).contains(&record.amt)
                        })
                        && carry.iter().enumerate().all(|(i, record)| {
                            !carry[..i].iter().any(|prior| {
                                prior.property == record.property && prior.year == record.year
                            })
                        })
                        && assessments.len() <= 400
                        && assessments.iter().all(ConservationAssessment::valid)
                }
                Self::Capitalize { method, .. } => method.valid(),
            }
    }
    pub(crate) fn deduction(&self, gross: i64) -> i64 {
        phase::cap(self.available(), [fraction(positive(gross), 25, 100)])
    }
    pub(crate) fn report(
        &self,
        _gross: i64,
        d: &FederalDraft,
        id: i64,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let context = d
            .income
            .iter()
            .find(|e| e.id == id)
            .map(crate::interview::income_issue_context)
            .unwrap_or(crate::IssueContext::Section(crate::TaxSection::Farming));
        crate::require_localized!(
            r,
            consistent_methods(d),
            &context,
            "Farm conservation method",
            "FARM_CONSERVATION_METHOD_CONFLICT",
            "A taxpayer's never-elected capitalization history conflicts with an established section175 deduction method on another farm. Resolve the original returns or actual project-specific IRS consent before treating these costs.",
        );
        if self.facts_omitted() {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FARM_CONSERVATION_FACTS_OMITTED"),
                context.clone(),
            );
        }
        let input = form.input("conservation");
        let deceased = owner_deceased(d, id);
        let allowed = deductions(d);
        if let Some((limit, _)) = &allowed {
            line_cents(
                r,
                &format!("farm_{id}_conservation_income_limit"),
                *limit,
                &[form.root()],
                "https://www.irs.gov/publications/p225",
                msg!("EXPLAIN_FARM_N_CONSERVATION_INCOME_LIMIT"),
            );
        }

        if matches!(self, Self::Deduct { .. }) && allowed.is_none() {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FARM_CONSERVATION_GROSS_SOURCE"),
                context.clone(),
            );
        }
        let deduction = allowed.as_ref().and_then(|(_, v)| v.get(&id)).copied().unwrap_or(0);
        let assessment_amounts = self.assessment_amounts(deceased);
        crate::require_localized!(
            r,
            !deceased || assessment_amounts[1] == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm conservation",
            "ISSUE_AN_ASSESSMENT_FIRST_SPREAD_IN_THE_YEAR_OF_DEATH_REQUIRES_SEPARATE_F_18A72DF0",
            "An assessment first spread in the year of death requires separate final-year treatment; the statutory acceleration covers death during the nine succeeding years.",
        );
        let amt_deduction = deductions_for(d, true)
            .as_ref()
            .and_then(|(_, values)| values.get(&id))
            .copied()
            .unwrap_or(0);
        let amt_unused = crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
            self.available_for_tax(deceased, true),
            amt_deduction,
        )
        .expect("AMT conservation deduction is capped by availability")
        .get();
        let unused = crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
            self.available_for(deceased),
            deduction,
        )
        .expect("conservation deduction is capped by availability")
        .get();
        for (name, amount, explanation) in [
            (
                "conservation_amt_deduction",
                amt_deduction,
                msg!("EXPLAIN_FARM_CONSERVATION_AMT_DEDUCTION"),
            ),
            (
                "conservation_amt_carryforward",
                if deceased { 0 } else { amt_unused },
                msg!("EXPLAIN_FARM_CONSERVATION_AMT_CARRY"),
            ),
            (
                "conservation_expired",
                if deceased { unused } else { 0 },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_EXPIRED"),
            ),
            (
                "conservation_assessment_available",
                assessment_amounts[0],
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_AVAILABLE"),
            ),
            (
                "conservation_assessment_deferred",
                assessment_amounts[1],
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_DEFERRED"),
            ),
            ("conservation_deduction", deduction, msg!("EXPLAIN_FARM_N_N_CONSERVATION_DEDUCTION")),
            (
                "conservation_carryforward",
                if deceased { 0 } else { unused },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_CARRYFORWARD"),
            ),
            (
                "conservation_basis_addition",
                if let Self::Capitalize { method, .. } = self {
                    if method.established() { self.paid() } else { 0 }
                } else {
                    assessment_amounts[2]
                        + self
                            .costs()
                            .iter()
                            .filter(|cost| !cost.facts.qualifies() && land_capital_cost(cost))
                            .map(|cost| cost.amount)
                            .sum::<i64>()
                },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_BASIS_ADDITION"),
            ),
        ] {
            let tag = (name == "conservation_deduction").then(|| {
                crate::FormLine::new(
                    form.tax_form(),
                    match form {
                        crate::farm::FarmForm::ScheduleF => "12",
                        crate::farm::FarmForm::Form4835 => "10",
                    },
                )
            });
            crate::kernel::lines::push_line(
                r,
                tag,
                &format!("farm_{id}_{name}"),
                amount,
                &[&input, "people"],
                "https://www.irs.gov/publications/p225",
                explanation,
            );
        }
    }
}

// The section 175 limit belongs to the taxpayer, not to each Schedule F.
// Allocate a binding limit in proportion to each activity's eligible cost.
// This is the approved bookkeeping policy; the IRS does not mandate this split.
// Accepted source IDs determine the exact-cent remainder independently of order.
fn deductions(d: &FederalDraft) -> Option<(i64, BTreeMap<i64, i64>)> {
    deductions_for(d, false)
}

impl FarmConservation {
    fn assessment_amounts(&self, deceased: bool) -> [i64; 3] {
        let mut assessment_amounts = [0; 3];
        if let Self::Deduct { assessments, .. } = self {
            for assessment in assessments {
                let amounts = if assessment.facts.qualifies() {
                    assessment.amounts_for(deceased)
                } else {
                    [
                        0,
                        0,
                        assessment
                            .payments
                            .iter()
                            .filter(|payment| payment.year == crate::year_constants::TAX_YEAR)
                            .map(|payment| payment.amount)
                            .sum(),
                    ]
                };
                for (total, amount) in assessment_amounts.iter_mut().zip(amounts) {
                    *total += amount;
                }
            }
        }
        assessment_amounts
    }
    fn facts_omitted(&self) -> bool {
        match self {
            Self::Deduct { costs, assessments, method, .. } => {
                !method.established()
                    || costs.iter().any(|cost| !cost.facts.qualifies())
                    || assessments.iter().any(|assessment| !assessment.facts.qualifies())
            }
            Self::Capitalize { method, .. } => !method.established(),
        }
    }
}

// Prepared direct-business inputs need the actual selected pool, not the
// legacy report's zero fallback. Keep the pool's existing community view.
pub(crate) fn selected_expense_adjustments(
    d: &FederalDraft,
    amt: bool,
) -> Result<BTreeMap<i64, i64>, (i64, &'static str)> {
    let farms: Vec<_> = d
        .income
        .iter()
        .filter_map(|entry| {
            let crate::FederalIncome::Business { data: business } = &entry.income else {
                return None;
            };
            let crate::BusinessActivity::Farming { data: farm } = &business.activity else {
                return None;
            };
            Some((entry.id, business, farm, farm.conservation.as_ref()?))
        })
        .collect();
    let Some((first, _, _, _)) = farms.first() else {
        return Ok(BTreeMap::new());
    };
    if !consistent_methods(d) {
        return Err((*first, "FARM_CONSERVATION_METHOD_CONFLICT"));
    }
    if let Some((id, _, _, _)) =
        farms.iter().find(|(_, _, _, conservation)| conservation.facts_omitted())
    {
        return Err((*id, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    }
    let allowed = if farms
        .iter()
        .any(|(_, _, _, conservation)| matches!(conservation, FarmConservation::Deduct { .. }))
    {
        deductions_for(d, amt).ok_or((*first, "WARN_FARM_CONSERVATION_GROSS_SOURCE"))?.1
    } else {
        BTreeMap::new()
    };
    farms.into_iter().map(|(id, business, farm, conservation)| {
        if owner_deceased(d, id) && conservation.assessment_amounts(true)[1] != 0 {
            return Err((id, "ISSUE_AN_ASSESSMENT_FIRST_SPREAD_IN_THE_YEAR_OF_DEATH_REQUIRES_SEPARATE_F_18A72DF0"));
        }
        let selected = if matches!(conservation, FarmConservation::Deduct { .. }) {
            *allowed.get(&id).ok_or((id, "WARN_FARM_CONSERVATION_GROSS_SOURCE"))?
        } else { 0 };
        let local = conservation.deduction(farm.gross() + crate::method_change::increase(&business.method_changes));
        Ok((id, selected - local))
    }).collect()
}
fn deductions_for(d: &FederalDraft, amt: bool) -> Option<(i64, BTreeMap<i64, i64>)> {
    if !consistent_methods(d) {
        return None;
    }
    // Schedule F farms and Form 4835 farm rentals share one limit: the rent
    // is gross income from the business of farming (Reg. 1.175-3, 1.175-5).
    let allocated_entries = crate::community::entries(d);
    let farms: Vec<_> = allocated_entries
        .iter()
        .filter_map(|entry| {
            let (farm, method_increase) = match &entry.income {
                crate::FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Farming { data: farm } => {
                        (farm, crate::method_change::increase(&data.method_changes))
                    }
                    _ => return None,
                },
                crate::FederalIncome::FarmRental { data } => (&data.farm, 0),
                _ => return None,
            };
            let gross = farm.conservation_gross().and_then(|gross| {
                let owner = usize::from(entry.owner == crate::Owner::Spouse);
                let cooperative = crate::cooperative::conservation_gross(d, entry.owner, entry.id)?;
                Some(
                    gross
                        + method_increase
                        + crate::community::income_amount(
                            d,
                            entry.id,
                            cooperative
                                + crate::federal_rules::farm_1099g_program(d, owner, entry.id),
                        ),
                )
            });
            Some((entry.as_ref(), farm, gross))
        })
        .collect();
    let owner = farms.first()?.0.owner;
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
    let community = d.setup.as_ref().is_some_and(crate::community::applies);
    if farms
        .iter()
        .any(|(e, _, gross)| (!joint && !community && e.owner != owner) || gross.is_none())
    {
        return None;
    }
    let gross = farms.iter().map(|(_, _, gross)| gross.unwrap()).sum::<i64>();
    let limit = fraction(positive(gross), 25, 100);
    let available: BTreeMap<_, _> = farms
        .iter()
        .map(|(e, f, _)| {
            (
                e.id,
                f.conservation
                    .as_ref()
                    .map_or(0, |c| c.available_for_tax(owner_deceased(d, e.id), amt)),
            )
        })
        .collect();
    let total: i64 = available.values().sum();
    let used = phase::cap(total, [limit]);
    let weights: Vec<_> = available.values().copied().collect();
    let allocated = crate::kernel::ratio::allocate_remaining(used, &weights);
    Some((limit, available.into_keys().zip(allocated).collect()))
}

pub(crate) fn expense_adjustment(d: &FederalDraft, id: i64, b: &crate::FederalBusiness) -> i64 {
    let crate::BusinessActivity::Farming { data } = &b.activity else {
        return 0;
    };
    let Some(c) = &data.conservation else {
        return 0;
    };
    let gross = data.gross() + crate::method_change::increase(&b.method_changes);
    // The deduction the cost stage already took is the one on the farm's own
    // records; the routed 1099-G and 1099-PATR income raises the limit.
    deductions(d).and_then(|(_, v)| v.get(&id).copied()).unwrap_or(0) - c.deduction(gross)
}

// Form 1099-G box 7 and 1099-PATR income routed to this farm or farm rental
// is gross income from farming for the section 175(b) limit.
fn routed(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Result<i64, crate::Section179PreparedIssue> {
    crate::federal_rules::farm_routed_income(
        d,
        usize::from(entry.owner == crate::Owner::Spouse),
        entry.id,
    )
}
pub(crate) fn routed_to(d: &FederalDraft, id: i64) -> Result<i64, crate::Section179PreparedIssue> {
    let entry = d
        .income
        .iter()
        .find(|entry| entry.id == id)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(id))?;
    routed(d, entry)
}

// The deduction a farm rental takes: its share of the pooled section 175 limit,
// or zero with a scoped benefit notice when actual production gross is unknown.
pub(crate) fn rental_deduction(
    d: &FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
) -> i64 {
    if farm.conservation.is_none() {
        return 0;
    }
    deductions_for(d, amt).and_then(|(_, v)| v.get(&id).copied()).unwrap_or(0)
}

fn owner_deceased(d: &FederalDraft, id: i64) -> bool {
    crate::farm::owner_death_date(d, id).is_some()
}

fn current_date(date: u32) -> bool {
    (crate::year_constants::TAX_YEAR_START_DATE..=crate::year_constants::TAX_YEAR_END_DATE)
        .contains(&date)
        && crate::federal_rules::valid_date(date)
}
impl FarmConservation {
    pub(crate) fn parcel_issue(&self, farm: &crate::FarmIncome) -> Option<ValidationIssue> {
        let parcel = |id| farm.land_basis.iter().find(|p| p.id > 0 && p.id == id);
        for (i, cost) in self.costs().iter().enumerate() {
            if parcel(cost.property).is_none() {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                        .at_field("property")
                        .in_list("costs", i),
                );
            }
        }
        if let Self::Deduct { assessments, carry, method, .. } = self {
            if let FarmConservationMethod::OriginalElection { year } = method {
                if let Some((i, _)) =
                    carry.iter().enumerate().find(|(_, record)| record.year < *year)
                {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_METHOD_HISTORY")
                            .at_field("year")
                            .in_list("carry", i),
                    );
                }
                for (i, record) in assessments.iter().enumerate() {
                    if let Some((j, _)) = record
                        .payments
                        .iter()
                        .enumerate()
                        .find(|(_, payment)| payment.amount != 0 && payment.year < *year)
                    {
                        return Some(
                            ValidationIssue::new("FARM_CONSERVATION_METHOD_HISTORY")
                                .at_field("year")
                                .in_list("payments", j)
                                .in_list("assessments", i),
                        );
                    }
                }
            }
            for (i, record) in assessments.iter().enumerate() {
                if parcel(record.property).is_none() {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                            .at_field("property")
                            .in_list("assessments", i),
                    );
                }
            }
            for (i, record) in carry.iter().enumerate() {
                if parcel(record.property).is_none() {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                            .at_field("property")
                            .in_list("carry", i),
                    );
                }
            }
            if let Some((i, _)) = carry
                .iter()
                .enumerate()
                .find(|(_, record)| !(1954..crate::year_constants::TAX_YEAR).contains(&record.year))
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_CARRY_YEAR")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
            if let Some((i, _)) = carry.iter().enumerate().find(|(i, record)| {
                carry[..*i]
                    .iter()
                    .any(|prior| prior.property == record.property && prior.year == record.year)
            }) {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_CARRY_DUPLICATE")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
            for (i, record) in assessments.iter().enumerate() {
                if let Some((j, _)) = record.payments.iter().enumerate().find(|(_, payment)| {
                    payment.current_date.is_some()
                        && payment.year != crate::year_constants::TAX_YEAR
                }) {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PAYMENT_DATE")
                            .at_field("current_date")
                            .in_list("payments", j)
                            .in_list("assessments", i),
                    );
                }
            }
            for (i, record) in assessments.iter().enumerate() {
                if let Some((j, _)) = record.payments.iter().enumerate().find(|(_, payment)| {
                    payment.year < parcel(record.property).unwrap().acquired / 10000
                }) {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                            .at_field("year")
                            .in_list("payments", j)
                            .in_list("assessments", i),
                    );
                }
            }
            if let Some((i, _)) = carry
                .iter()
                .enumerate()
                .find(|(_, record)| record.year < parcel(record.property).unwrap().acquired / 10000)
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
        }
        for (i, cost) in self.costs().iter().enumerate() {
            if cost
                .paid_or_incurred
                .is_some_and(|date| date < parcel(cost.property).unwrap().acquired)
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                        .at_field("paid_or_incurred")
                        .in_list("costs", i),
                );
            }
        }
        None
    }
}

// The business collector has already deducted the regular allowed amount.
// Supply the separate legal AMT carry/pool difference once; no second tax engine.
pub(crate) fn business_amt_adjustment(d: &FederalDraft, id: i64) -> i64 {
    let source = crate::farm::sources(d).find(|source| source.entry.id == id && !source.rental);
    let Some(source) = source else {
        return 0;
    };
    if source.farm.conservation.is_none() {
        return 0;
    }
    let allowed =
        |amt| deductions_for(d, amt).and_then(|(_, values)| values.get(&id).copied()).unwrap_or(0);
    allowed(false) - allowed(true)
}

// Ephemeral disposal projection from the same legal section175 pool. Aggregate
// current/carry character by accepted parcel and source year before allocating
// within an activity; unordered cost rows cannot change exact-cent ownership.
// Twenty-five-percent-limited carry never becomes basis. Only actual capital
// costs and the unexpired assessment spread do (Pub225ch5).
pub(crate) fn parcel_adjustment(
    d: &FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    property: i64,
    sold: u32,
    amt: bool,
) -> Option<(i64, i64)> {
    if !consistent_methods(d) {
        return None;
    }
    let Some(conservation) = &farm.conservation else {
        return Some((0, 0));
    };
    let costs = conservation.costs();
    // An actual sale needs actual current timing, rather than an invented
    // payment day or a guessed allocation of post-disposition obligations.
    if costs
        .iter()
        .filter(|cost| cost.property == property && cost.amount != 0)
        .any(|cost| cost.paid_or_incurred.is_none_or(|date| date > sold))
    {
        return None;
    }
    match conservation {
        FarmConservation::Capitalize { method, .. } => {
            if !method.established()
                && costs.iter().any(|cost| cost.property == property && cost.amount != 0)
            {
                return None;
            }
            Some((
                costs
                    .iter()
                    .filter(|cost| cost.property == property && land_capital_cost(cost))
                    .map(|cost| cost.amount)
                    .sum(),
                0,
            ))
        }
        FarmConservation::Deduct { assessments, carry, method, .. } => {
            if !method.established() {
                return None;
            }
            if assessments.iter().filter(|record| record.property == property).any(|record| {
                record.payments.iter().any(|payment| {
                    payment.amount != 0
                        && payment.year == crate::year_constants::TAX_YEAR
                        && payment.current_date.is_none_or(|date| date > sold)
                })
            }) {
                return None;
            }
            let allowed = deductions_for(d, amt)?.1.get(&id).copied().unwrap_or(0);
            let deceased = owner_deceased(d, id);
            let mut weights = BTreeMap::<(i64, u32), i64>::new();
            for cost in costs.iter().filter(|cost| cost.facts.qualifies()) {
                *weights.entry((cost.property, crate::year_constants::TAX_YEAR)).or_default() +=
                    cost.amount;
            }
            for record in carry {
                *weights.entry((record.property, record.year)).or_default() +=
                    if amt { record.amt } else { record.regular };
            }
            let mut basis = costs
                .iter()
                .filter(|cost| {
                    cost.property == property && !cost.facts.qualifies() && land_capital_cost(cost)
                })
                .map(|cost| cost.amount)
                .sum::<i64>();
            for assessment in assessments {
                if !assessment.facts.qualifies() {
                    if assessment.property == property {
                        basis += assessment
                            .payments
                            .iter()
                            .filter(|payment| payment.year == crate::year_constants::TAX_YEAR)
                            .map(|payment| payment.amount)
                            .sum::<i64>();
                    }
                    continue;
                }
                let amounts = assessment.amounts_for(deceased);
                *weights
                    .entry((assessment.property, crate::year_constants::TAX_YEAR))
                    .or_default() += amounts[0];
                if assessment.property == property {
                    basis += amounts[2] + amounts[1];
                }
            }
            let amounts = weights.values().copied().collect::<Vec<_>>();
            let allocated = crate::kernel::ratio::allocate_remaining(allowed, &amounts);
            let recapture = weights
                .into_keys()
                .zip(allocated)
                .filter(|((parcel, _), _)| *parcel == property)
                .map(|(_, amount)| amount)
                .sum();
            Some((basis, recapture))
        }
    }
}

// Original method is taxpayer-wide; an actual IRS consent can identify a
// different farm/project. Joint spouses remain separate method owners even
// though their eligible gross and costs share the joint-return25%limit.
pub(crate) fn consistent_methods(d: &FederalDraft) -> bool {
    method_issue(d).is_none()
}
pub(crate) fn method_issue(d: &FederalDraft) -> Option<(i64, crate::validation::ValidationIssue)> {
    crate::farm::sources(d).find(|source| {
        matches!(&source.farm.conservation,
            Some(FarmConservation::Capitalize { method: FarmConservationCapitalMethod::NeverElected, .. }))
            && crate::farm::sources(d).any(|other| other.entry.owner == source.entry.owner
                && (other.farm.land_basis.iter().any(|parcel| parcel.prior_conservation > 0 || parcel.prior_amt_conservation > 0)
                    || matches!(&other.farm.conservation, Some(FarmConservation::Deduct { method, .. }) if method.established())))
    }).map(|source| (source.entry.id, crate::validation::ValidationIssue::new("FARM_CONSERVATION_METHOD_CONFLICT").at_field("method").in_field("conservation")))
}

fn land_capital_cost(cost: &ConservationCost) -> bool {
    !matches!(
        cost.facts.work,
        crate::FarmConservationWork::Maintenance | crate::FarmConservationWork::OwnerDepreciable
    )
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(FarmConservation {
    records: [
        Capitalize { amounts: [method, costs], facts: [] },
        Deduct { amounts: [costs, assessments, carry, method], facts: [] }
    ],
    units: [],
    tuples: [],
});

crate::community::divide_enum!(FarmConservationMethod {
    records: [
        OriginalElection { amounts: [], facts: [year] },
        ApprovedChange { amounts: [], facts: [year, approval] }
    ],
    units: [Unestablished],
    tuples: [],
});

crate::community::divide_record!(ConservationAssessment {
    amounts: [facts, share, payments],
    gains: [],
    facts: [property, label, total],
});

crate::community::divide_record!(ConservationAssessmentPayment {
    amounts: [amount],
    gains: [],
    facts: [year, current_date],
});

crate::community::divide_record!(ConservationCost {
    amounts: [facts, amount],
    gains: [],
    facts: [property, label, paid_or_incurred],
});

crate::community::divide_enum!(FarmConservationCapitalMethod {
    records: [ApprovedChange { amounts: [], facts: [year, approval] }],
    units: [NeverElected, Unestablished],
    tuples: [],
});
