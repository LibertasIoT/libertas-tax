use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::validation::ValidationIssue;
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use libertas_macros::*;

/// Conservation payment record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ConservationCost {
    /// Accepted farm land parcel
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Actual conservation plan and work records
    pub facts: FarmConservationFacts,
    /// Work or payment description
    #[libertas_size(min = 1)]
    pub label: String,
    /// Cost paid or incurred under your farm accounting method
    /// Exclude reimbursements, amounts deducted elsewhere, ordinary maintenance and separately depreciable assets.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    pub facts: FarmConservationFacts,
    /// District property or project
    #[libertas_size(min = 1)]
    pub label: String,
    /// Total assessment against all district members for this property
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub total: i64,
    /// Your complete assessment for this property, including future installments
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
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
    pub(crate) fn amounts_for(&self, deceased: bool) -> [i64; 3] {
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
    pub(crate) fn established(&self) -> bool {
        !matches!(self, Self::Unestablished)
    }
}
impl FarmConservationFacts {
    pub(crate) fn valid(&self) -> bool {
        self.plan_reference.as_ref().is_none_or(|value| !value.is_empty())
            && (matches!(self.plan, FarmConservationPlan::None) || self.plan_reference.is_some())
    }
    pub(crate) fn qualifies(&self) -> bool {
        match (&self.plan, &self.work) {
            (
                FarmConservationPlan::Nrcs | FarmConservationPlan::ComparableState,
                FarmConservationWork::QualifyingLandWork,
            ) => self.land_farmed,
            (
                FarmConservationPlan::EndangeredRecovery,
                FarmConservationWork::EndangeredRecovery,
            ) => true,
            _ => false,
        }
    }
}
impl FarmConservation {
    pub(crate) fn plan_reference_issue(&self) -> Option<ValidationIssue> {
        let missing = |facts: &FarmConservationFacts| {
            !matches!(facts.plan, FarmConservationPlan::None) && facts.plan_reference.is_none()
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

    pub(crate) fn costs(&self) -> &[ConservationCost] {
        match self {
            Self::Capitalize { costs, .. } | Self::Deduct { costs, .. } => costs,
        }
    }
    pub(crate) fn paid(&self) -> i64 {
        self.costs().iter().filter(|cost| land_capital_cost(cost)).map(|c| c.amount).sum()
    }
    fn available(&self) -> i64 {
        self.available_for(false)
    }
    fn available_for(&self, deceased: bool) -> i64 {
        self.available_for_tax(deceased, false)
    }
    pub(crate) fn current_available(&self, deceased: bool, amt: bool) -> i64 {
        let retained = match self {
            Self::Deduct { carry, .. } => {
                carry.iter().map(|row| if amt { row.amt } else { row.regular }).sum()
            }
            _ => 0,
        };
        self.available_for_tax(deceased, amt) - retained
    }
    pub(crate) fn available_for_tax(&self, deceased: bool, amt: bool) -> i64 {
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
}

/// Retained section 175 carry for one parcel and source year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmConservationCarry {
    /// Canonical farm land parcel
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Original cost year from retained worksheets
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Unused regular-tax 25-percent-limited cost
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// Corresponding unused AMT cost
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt: i64,
}

/// Conservation plan, work and actual land-use facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmConservationFacts {
    /// Actual approved plan source
    pub plan: FarmConservationPlan,
    /// Actual work rather than an eligibility checkbox
    pub work: FarmConservationWork,
    /// Land used in farming now or previously by you or your tenant
    /// A newly acquired operating farm also qualifies when you substantially continue the immediately preceding owner's farming use. Newly developed unused land does not qualify merely because future farming is intended.
    pub land_farmed: bool,
    /// NRCS/state or endangered-species site-specific approval identifier
    /// [NullText]
    /// No approved conservation plan applies
    /// [SetText]
    /// The conservation-plan approval identifier is
    #[libertas_size(min = 1)]
    pub plan_reference: Option<String>,
}

/// Actual conservation plan source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationPlan {
    /// NRCS-approved soil/water plan
    Nrcs,
    /// Comparable state plan
    ComparableState,
    /// Site-specific approved endangered-species recovery plan
    EndangeredRecovery,
    /// No approved qualifying plan; no elective deduction
    None,
}

/// Actual cost category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationWork {
    /// Conservation/erosion work otherwise eligible
    QualifyingLandWork,
    /// Site-specific endangered-species management
    EndangeredRecovery,
    /// Wetlands draining/filling; capitalize
    WetlandDrainage,
    /// Center-pivot irrigation land preparation; capitalize
    PivotPreparation,
    /// Owner depreciable facility; shared asset owner
    OwnerDepreciable,
    /// Ordinary repairs/maintenance; expense owner
    Maintenance,
}

impl FarmConservation {
    pub(crate) fn assessment_amounts(&self, deceased: bool) -> [i64; 3] {
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
    pub(crate) fn facts_omitted(&self) -> bool {
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

fn current_date(date: u32) -> bool {
    (crate::year_constants::TAX_YEAR_START_DATE..=crate::year_constants::TAX_YEAR_END_DATE)
        .contains(&date)
        && crate::dates::source_valid_date(date)
}
pub(crate) fn land_capital_cost(cost: &ConservationCost) -> bool {
    !matches!(
        cost.facts.work,
        FarmConservationWork::Maintenance | FarmConservationWork::OwnerDepreciable
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

crate::community::divide_record!(FarmConservationCarry {
    amounts: [regular, amt],
    gains: [],
    facts: [property, year],
});

crate::community::divide_record!(FarmConservationFacts {
    amounts: [plan, work],
    gains: [],
    facts: [land_farmed, plan_reference],
});

crate::community::divide_enum!(FarmConservationWork {
    records: [],
    units: [
        QualifyingLandWork,
        EndangeredRecovery,
        WetlandDrainage,
        PivotPreparation,
        OwnerDepreciable,
        Maintenance
    ],
    tuples: [],
});

crate::community::divide_enum!(FarmConservationPlan {
    records: [],
    units: [Nrcs, ComparableState, EndangeredRecovery, None],
    tuples: [],
});
