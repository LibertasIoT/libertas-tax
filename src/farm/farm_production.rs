use crate::validation::ValidationIssue;
// One cost projection for actual direct-farm production cohorts. Property
// recovery/ADS and interest-owner boundaries remain explicit; no second engine.
use crate::*;
use alloc::{format, vec::Vec};

impl FarmProductionCohort {
    pub(crate) fn production_issue(&self) -> Option<ValidationIssue> {
        let money = |v| (0..=100_000_000_000).contains(&v);
        let date = |v| {
            (19000101..=crate::year_constants::TAX_YEAR_END_DATE).contains(&v)
                && crate::federal_rules::valid_date(v)
        };
        for (field, invalid) in [
            ("id", self.id < 0 || self.id > 100_000_000_000),
            ("description", self.description.trim().is_empty()),
            ("began", !date(self.began)),
            ("productive", self.productive.is_some_and(|v| !date(v) || v < self.began)),
            (
                "normal_preproductive_months",
                !(1..=1200).contains(&self.normal_preproductive_months),
            ),
            ("prior_basis", !money(self.prior_basis)),
            ("prior_amt_basis", !money(self.prior_amt_basis)),
            ("costs", self.costs.len() > 400),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM_PRODUCTION_DATES").at_field(field));
            }
        }
        if let FarmCapitalization::SmallBusiness { facts } = &self.tax_treatment
            && let Some(issue) = facts.issue()
        {
            return Some(issue.in_field("facts").in_field("tax_treatment"));
        }
        if let FarmCapitalization::OriginalElectionOut { year } = &self.tax_treatment {
            let field = if !(1987..=crate::year_constants::TAX_YEAR).contains(&year.year) {
                Some("year")
            } else if year.statement.trim().is_empty() {
                Some("statement")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_PRODUCTION_METHOD_SOURCE")
                        .at_field(field)
                        .in_field("year")
                        .in_field("tax_treatment"),
                );
            }
        }
        for (i, cost) in self.costs.iter().enumerate() {
            if matches!(cost.treatment, BusinessExpenseKind::DeMinimis { .. }) {
                return Some(
                    ValidationIssue::new("FARM_PRODUCTION_EXPENSE_OWNER")
                        .at_field("treatment")
                        .in_list("costs", i),
                );
            }
            // Shared expense treatment remains the sole owner of tax_cost().
            let field = if !crate::federal_rules::expenses_valid(&[crate::FederalBusinessExpense {
                category: cost.category.clone(),
                amount: cost.cost,
                treatment: cost.treatment,
            }]) {
                Some("treatment")
            } else if cost.description.trim().is_empty() {
                Some("description")
            } else if !money(cost.cost) {
                Some("cost")
            } else if !money(cost.non263a_capital) {
                Some("non263a_capital")
            } else if !money(cost.non263a_amt_capital) {
                Some("non263a_amt_capital")
            } else if !money(cost.capital_allocated) {
                Some("capital_allocated")
            } else if !money(cost.amt_capital_allocated) {
                Some("amt_capital_allocated")
            } else if cost.non263a_capital + cost.capital_allocated > cost.tax_cost() {
                Some("capital_allocated")
            } else if cost.non263a_amt_capital + cost.amt_capital_allocated > cost.tax_cost() {
                Some("amt_capital_allocated")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_PRODUCTION_ALLOCATION")
                        .at_field(field)
                        .in_list("costs", i),
                );
            }
        }
        if let FarmBasisDestination::Inventory { released, amt_released } = self.destination {
            for (field, value) in [("released", released), ("amt_released", amt_released)] {
                if !money(value) {
                    return Some(
                        ValidationIssue::new("FARM_PRODUCTION_RELEASE")
                            .at_field(field)
                            .in_field("destination"),
                    );
                }
            }
            for amt in [false, true] {
                if let Some(required) = self.unicap_applies() {
                    let capital =
                        self.costs.iter().map(|cost| cost.capital(amt, required)).sum::<i64>();
                    let available =
                        if amt { self.prior_amt_basis } else { self.prior_basis } + capital;
                    if (if amt { amt_released } else { released }) > available {
                        return Some(
                            ValidationIssue::new("FARM_PRODUCTION_RELEASE")
                                .at_field(if amt { "amt_released" } else { "released" })
                                .in_field("destination"),
                        );
                    }
                }
            }
        }
        None
    }
    pub(crate) fn production_rule(&self) -> Option<&'static str> {
        self.production_issue().map(|issue| issue.key)
    }
    fn unicap_applies(&self) -> Option<bool> {
        Some(match &self.tax_treatment {
            FarmCapitalization::Required => true,
            FarmCapitalization::SmallBusiness { facts } => !facts.qualifies(),
            FarmCapitalization::ShortPlant { required_accrual_entity_or_shelter } => {
                *required_accrual_entity_or_shelter
                    || !matches!(self.property, FarmProducedKind::Plant)
                    || self.normal_preproductive_months > 24
            }
            FarmCapitalization::AnimalException { required_accrual_entity_or_shelter } => {
                *required_accrual_entity_or_shelter
                    || !matches!(self.property, FarmProducedKind::Animal)
            }
            // Actual election-out needs shared ADS, citrus/almond restrictions
            // and future1245 recapture source, not a guessed zero capital basis.
            FarmCapitalization::OriginalElectionOut { .. }
            | FarmCapitalization::UnresolvedAllocation => return None,
        })
    }
    // Current ordinary cost, released capital cost, retained closing basis.
    pub(crate) fn production_projection(&self, cash: bool, amt: bool) -> Option<[i64; 3]> {
        if !cash
            || self.production_rule().is_some()
            || self
                .costs
                .iter()
                .any(|cost| matches!(cost.kind, FarmProductionCostKind::InterestOwner))
        {
            return None;
        }
        let FarmBasisDestination::Inventory { released, amt_released } = self.destination else {
            return None;
        };
        // Recovery of sold/consumed capital inventory belongs to its actual
        // COGS or property recovery owner. An amount alone does not establish
        // the receipt category or gross-income treatment for sections175/6654.
        if released != 0 || amt_released != 0 {
            return None;
        }
        let required = self.unicap_applies()?;
        if required
            && self.costs.iter().any(|cost| {
                matches!(cost.kind, FarmProductionCostKind::Direct)
                    && cost.capital(amt, true) != cost.tax_cost()
            })
        {
            return None;
        }
        let capital = self.costs.iter().map(|cost| cost.capital(amt, required)).sum::<i64>();
        let current = self.costs.iter().map(|cost| cost.tax_cost()).sum::<i64>() - capital;
        let release = if amt { amt_released } else { released };
        let opening = if amt { self.prior_amt_basis } else { self.prior_basis };
        Some([current, release, opening + capital - release])
    }
}
impl FarmProductionCost {
    fn tax_cost(&self) -> i64 {
        crate::FederalBusinessExpense::total(&[crate::FederalBusinessExpense {
            category: self.category.clone(),
            amount: self.cost,
            treatment: self.treatment,
        }])
    }
    fn capital(&self, amt: bool, required: bool) -> i64 {
        let (base, uniform) = if amt {
            (self.non263a_amt_capital, self.amt_capital_allocated)
        } else {
            (self.non263a_capital, self.capital_allocated)
        };
        base + if required { uniform } else { 0 }
    }
}
impl FarmSpecials {
    pub(crate) fn production_deduction(&self, cash: bool, amt: bool) -> i64 {
        self.production
            .iter()
            .filter_map(|cohort| cohort.production_projection(cash, amt))
            .map(|amounts| amounts[0] + amounts[1])
            .sum()
    }
}
pub(crate) fn report(
    s: &FarmSpecials,
    cash: bool,
    id: i64,
    form: crate::farm::FarmForm,
    context: &IssueContext,
    r: &mut FederalResult,
) {
    for cohort in &s.production {
        if let FarmCapitalization::SmallBusiness { facts } = &cohort.tax_treatment
            && !facts.qualifies()
        {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_FARM263A_EXEMPTION_NOT_MET"),
                context.clone(),
            );
        }
        for amt in [false, true] {
            let Some(values) = cohort.production_projection(cash, amt) else {
                continue;
            };
            let suffix = if amt { "amt_retained_basis" } else { "retained_basis" };
            crate::kernel::lines::line_cents(
                r,
                &format!("farm_{id}_production_{}_{suffix}", cohort.id),
                values[2],
                &[&form.input("specials.production")],
                "https://www.irs.gov/publications/p225",
                msg!("EXPLAIN_FARM_PRODUCTION_RETAINED_BASIS"),
            );
        }
    }
}
// Schedule F instructions require current expense categories before 263A
// capitalization and a separate negative line32f adjustment. Costs required
// to be capitalized under other rules never become ordinary expense here.
pub(crate) fn category_totals(s: &FarmSpecials, cash: bool) -> Vec<(BusinessExpenseCategory, i64)> {
    let mut values = Vec::new();
    for cohort in &s.production {
        if cohort.production_projection(cash, false).is_none() {
            continue;
        }
        for cost in &cohort.costs {
            values.push((cost.category.clone(), cost.tax_cost() - cost.non263a_capital));
        }
    }
    values
}
pub(crate) fn business_amt_adjustment(d: &FederalDraft, id: i64) -> i64 {
    let Some(source) =
        crate::farm::sources(d).find(|source| source.entry.id == id && !source.rental)
    else {
        return 0;
    };
    let cash = matches!(source.farm.accounting, FarmAccounting::Cash { .. });
    source.farm.specials.production_deduction(cash, false)
        - source.farm.specials.production_deduction(cash, true)
}

// Current section263A allocation only; unrelated mandatory capital costs
// have already been excluded from the ordinary expense categories.
pub(crate) fn uniform_capital(s: &FarmSpecials, cash: bool) -> i64 {
    s.production
        .iter()
        .filter(|cohort| cohort.production_projection(cash, false).is_some())
        .filter(|cohort| cohort.unicap_applies() == Some(true))
        .flat_map(|cohort| &cohort.costs)
        .map(|cost| cost.capital_allocated)
        .sum()
}
