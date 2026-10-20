// Coverage: direct Schedule C section194 original cost-year elections, actual
// prior recovery and the July/84-month clock. Disposition gain/loss/recapture,
// transfers, changed use, adjusted or missing histories remain unavailable.
// Sources: IRC194(a)-(c),1245(a)(2)(C)/(b)(7); Notice2006-47 C.2.
use crate::kernel::cost_recovery::{
    Month,
    monthly_history::{TimberElection, TimberRecovery},
};
use crate::kernel::lines::{form_count, line_cents, push_line};
use crate::{FederalResult, FormLine, TaxForm};
use alloc::{collections::BTreeSet, format, string::String};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Qualified timber property cost-year cohort
/// Consolidate this property's direct costs for one expenditure year once.
/// Keep them separate from ordinary expenses, equipment and entity statements.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessTimberCost {
    /// Cost or planting work described on Form 4562-B
    #[libertas_size(min = 1)]
    pub description: String,
    /// Original direct capitalized cost before reimbursements and elections
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Date costs were incurred under this business's accounting method
    /// Amortization starts in July of this expenditure year, including for costs
    /// incurred later in the year. Consolidate this property's cost-year facts.
    #[libertas_date_only]
    pub incurred: u32,
    /// Amortization actually allowed or allowable before the return year
    /// Use original returns and basis records, never a new normal-schedule
    /// estimate. Enter zero for current costs or a documented zero history.
    #[libertas_default(Unavailable)]
    pub prior_allowed: crate::SourceCents,
    /// Original property and election facts
    pub facts: ReforestationCost,
    /// Complete original records and unchanged basis/use
    /// Confirm the original taxpayer, property and elections, with no transfer,
    /// changed use, additional basis adjustment or omitted prior deduction.
    /// No retains the source but leaves this calculation unavailable.
    pub original_records_complete: bool,
    /// Disposition or retirement, including an earlier-year event
    /// The month limits amortization without restarting its original period.
    /// Disposed timber still needs gain, loss and recapture calculations before
    /// the return estimate is complete.
    /// [NullText]
    /// This cost cohort has not been disposed of
    /// [SetText]
    /// This cost cohort was disposed of through
    #[libertas_default($null)]
    pub ended: Option<AmortizationEnd>,
}

/// Original section194 elections for qualified timber property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReforestationCost {
    /// Stable property identifier across direct and entity records
    #[libertas_size(min = 1)]
    pub property_key: String,
    /// U.S. woodlot held for commercial timber quantities
    pub domestic_commercial_timber: bool,
    /// Government reimbursement for these original costs
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub government_reimbursement: i64,
    /// Corresponding reimbursement was included in gross income
    /// Only actually excluded reimbursement reduces qualifying cost. Report
    /// taxable reimbursement in this business's income separately, once.
    pub reimbursement_taxable: bool,
    /// Original section194(b) expense election for this expenditure year
    /// Current costs share the $10,000 property limit ($5,000 if married filing
    /// separately) with other direct/entity costs. An earlier election keeps its
    /// original amount and is never deducted again this year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub current_expense_elected: i64,
    /// Other amounts using this property's expense limit in the same cost year
    /// Include actual entity or other-source elections only. Do not repeat this
    /// direct cohort's election. This amount uses room; it is not deducted here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other_current_expense: i64,
    /// Original basis actually elected for section194(a) amortization
    /// After excluded reimbursements and expensing. Zero means no election;
    /// unelected qualified costs retain capital basis. Do not create a new
    /// election for an earlier year or amortize unused expense room automatically.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amortizable_basis_elected: i64,
}

/// Actual end of an amortizable cost cohort
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AmortizationEnd {
    /// Actual disposition date, including historical events
    #[libertas_date_only]
    pub date: u32,
    /// Actual disposition financial facts
    pub how: crate::DisposalKind,
}

impl BusinessTimberCost {
    fn prior_recovery(&self) -> Option<i64> {
        self.prior_allowed.source_fact().ok()?.require(|| ()).ok().map(|amount| amount.unwrap_or(0))
    }
    fn election(&self) -> TimberElection {
        TimberElection {
            capital_cost_cents: self.cost,
            excluded_reimbursement_cents: if self.facts.reimbursement_taxable {
                0
            } else {
                self.facts.government_reimbursement
            },
            original_expense_cents: self.facts.current_expense_elected,
            elected_basis_cents: self.facts.amortizable_basis_elected,
            prior_amortization_cents: self.prior_recovery(),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        !self.description.trim().is_empty()
            && !self.facts.property_key.trim().is_empty()
            && self.incurred >= 19800101
            && self.incurred <= crate::year_constants::TAX_YEAR_END_DATE
            && crate::federal_rules::valid_date(self.incurred)
            && (0..=self.cost).contains(&self.facts.government_reimbursement)
            && self.facts.other_current_expense >= 0
            && (self.incurred >= 20041023 || self.facts.current_expense_elected == 0)
            && self
                .prior_allowed
                .source_fact()
                .and_then(|fact| {
                    fact.check_reported(|amount| {
                        if *amount <= self.facts.amortizable_basis_elected
                            && (self.incurred / 10000 != crate::year_constants::TAX_YEAR
                                || *amount == 0)
                        {
                            Ok(())
                        } else {
                            Err("TIMBER_PRIOR_BASIS_INVALID")
                        }
                    })
                })
                .is_ok()
            && self.election().validate_basis().is_ok()
            && self.ended.as_ref().is_none_or(|end| {
                crate::federal_rules::valid_date(end.date)
                    && (self.incurred..=crate::year_constants::TAX_YEAR_END_DATE)
                        .contains(&end.date)
                    && end.how.valid()
            })
    }
    pub(crate) fn recovery(&self, year: u32) -> Option<TimberRecovery> {
        if !self.valid()
            || !self.original_records_complete
            || !self.facts.domestic_commercial_timber
        {
            return None;
        }
        self.election()
            .recover(
                self.incurred / 10000,
                year,
                self.ended
                    .as_ref()
                    .map(|end| Month { year: end.date / 10000, month: end.date / 100 % 100 }),
            )
            .ok()
    }
}

// A consolidated original direct property/year cohort is entered once across
// activities. This identifies the actual history; it is not a record-count cap.
pub(crate) fn source_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let separate =
        d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Separate);
    let mut seen = BTreeSet::new();
    let mut aggregate = 0_i64;
    for entry in &d.income {
        let crate::FederalIncome::Business { data } = &entry.income else { continue };
        // Preserve all pre-existing no-timber validation paths. Never call the
        // legacy deduction helpers until their own source bounds are proven.
        if data.timber.is_empty()
            || !data.startup.valid()
            || data.intangibles.len() > 100
            || !data.intangibles.iter().all(crate::BusinessIntangible::valid)
            || !data.timber.iter().all(BusinessTimberCost::valid)
        {
            continue;
        }
        // The standard input validator proves the remaining source bounds
        // before the operating-cost owner is read. Preserve its legacy error.
        if let Err((issue, _)) = crate::federal_rules::validate_income_issue(entry) {
            if issue.key == "TIMBER_TOTAL_UNREPRESENTABLE" {
                return Some((entry.id, issue));
            }
            continue;
        }
        for (index, cost) in data.timber.iter().enumerate() {
            let issue = |key, field| {
                (
                    entry.id,
                    crate::validation::ValidationIssue::new(key)
                        .at_field(field)
                        .in_field("facts")
                        .in_list("timber", index)
                        .in_field("more"),
                )
            };
            if !seen.insert((cost.facts.property_key.as_str(), cost.incurred / 10000)) {
                return Some(issue("TIMBER_COHORT_ONCE", "property_key"));
            }
            if i64::from(cost.incurred / 10000) == i64::from(d.tax_year)
                && !cost.election().expense_within_property_limit(
                    cost.facts.other_current_expense,
                    if separate { 500_000 } else { 1_000_000 },
                )
            {
                return Some(issue("TIMBER_PROPERTY_EXPENSE_LIMIT", "current_expense_elected"));
            }
        }
        let next =
            data.checked_amortization_cents().and_then(|amount| aggregate.checked_add(amount));
        let Some(next) = next else {
            return Some((
                entry.id,
                crate::validation::ValidationIssue::new("TIMBER_TOTAL_UNREPRESENTABLE")
                    .at_field("timber")
                    .in_field("more"),
            ));
        };
        aggregate = next;
    }
    None
}

pub(crate) fn return_issue(d: &crate::FederalDraft) -> Option<crate::validation::ValidationIssue> {
    source_issue(d).map(|(_, issue)| crate::validation::ValidationIssue::new(issue.key))
}

// Cents retained through each cohort; only the shared sum owner projects the
// aggregate into native cents. Unknown cohorts are reported by report(), as
// before, and never establish a complete zero deduction.
pub(crate) fn totals(costs: &[BusinessTimberCost]) -> Option<[i64; 3]> {
    let mut totals = [0_i64; 3];
    for cost in costs {
        let Some(recovery) = cost.recovery(crate::year_constants::TAX_YEAR) else { continue };
        totals[0] = totals[0].checked_add(recovery.current_expense_cents)?;
        let index = if cost.incurred / 10000 < crate::year_constants::TAX_YEAR { 1 } else { 2 };
        totals[index] = totals[index].checked_add(recovery.amortization_cents)?;
    }
    crate::kernel::ratio::checked_sum(totals)?;
    Some(totals)
}

pub(crate) fn deduction(costs: &[BusinessTimberCost]) -> Option<i64> {
    crate::kernel::ratio::checked_sum(totals(costs)?)
}

pub(crate) fn report(
    costs: &[BusinessTimberCost],
    result: &mut FederalResult,
    id: i64,
    #[cfg(test)] section: &str,
) {
    let inputs = ["income.business.timber"];
    let source = "https://www.irs.gov/instructions/i4562";
    for (index, cost) in costs.iter().enumerate() {
        let context = crate::IssueContext::income_id(id).item("data.timber", index);
        let recovered = cost.recovery(crate::year_constants::TAX_YEAR);
        crate::require_localized!(
            result,
            recovered.is_some(),
            &context,
            section,
            "ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED",
            "Confirm qualified U.S. commercial timber and complete original elections, unchanged basis/use and actual prior recovery before this cohort can be calculated."
        );
        let Some(recovery) = recovered else { continue };
        // The actual property owner resolves current dispositions. An unsupported
        // original expense character or missing basis remains unavailable.
        crate::require_localized!(
            result,
            crate::cohorts::timber_sale(cost).is_ok(),
            &context,
            section,
            "ISSUE_TIMBER_DISPOSITION_NEEDED",
            "This timber disposition needs its gain, loss and section 1245 expense/amortization recapture calculation. The complete return estimate is unavailable."
        );
        if let Ok(Some(sale)) = crate::cohorts::timber_sale(cost) {
            crate::asset_sale::report_amortized_sale(
                result,
                &format!("business_{id}_timber_{index}_disposal"),
                &sale,
                "income.business.timber",
            );
        }
        let current = cost.incurred / 10000 == crate::year_constants::TAX_YEAR;
        let tag = |line: &str| {
            (current && recovery.elected_basis_cents > 0)
                .then(|| FormLine::new(TaxForm::Form4562B, line))
        };
        let prefix = format!("business_{id}_timber_{index}");
        if current && recovery.elected_basis_cents > 0 {
            use crate::kernel::lines::{LineValue, row};
            for (line, suffix, explanation) in [
                (
                    "1(a)",
                    "description",
                    crate::Message::new(
                        "TEXT",
                        alloc::vec![crate::MessageArg::Text(cost.description.clone())],
                    ),
                ),
                (
                    "1(b)",
                    "started",
                    crate::Message::new(
                        "TIMBER_START_DATE",
                        alloc::vec![crate::MessageArg::DateOnly(i64::from(
                            cost.incurred / 10000 * 10000 + 701
                        ))],
                    ),
                ),
                (
                    "1(d)",
                    "code",
                    crate::Message::new("TEXT", alloc::vec![crate::MessageArg::Text("194".into())]),
                ),
            ] {
                result.lines.push(row(
                    tag(line),
                    &format!("{prefix}_{suffix}"),
                    LineValue::Text,
                    &inputs,
                    source,
                    explanation,
                ));
            }
        }
        push_line(
            result,
            tag("1(c)"),
            &format!("{prefix}_basis"),
            recovery.elected_basis_cents,
            &inputs,
            source,
            msg!("EXPLAIN_TIMBER_ELECTED_BASIS"),
        );
        if recovery.elected_basis_cents > 0 && current {
            form_count(
                result,
                FormLine::new(TaxForm::Form4562B, "1(e)"),
                &format!("{prefix}_period"),
                84,
                &inputs,
                source,
                msg!("EXPLAIN_TIMBER_PERIOD"),
            );
        }
        push_line(
            result,
            tag("1(f)"),
            &format!("{prefix}_amortization"),
            recovery.amortization_cents,
            &inputs,
            source,
            msg!("EXPLAIN_TIMBER_AMORTIZATION"),
        );
        line_cents(
            result,
            &format!("{prefix}_current_expense"),
            recovery.current_expense_cents,
            &inputs,
            source,
            msg!("EXPLAIN_TIMBER_CURRENT_EXPENSE"),
        );
        line_cents(
            result,
            &format!("{prefix}_remaining"),
            recovery.remaining_cents,
            &inputs,
            source,
            msg!("EXPLAIN_TIMBER_REMAINING"),
        );
    }
}
