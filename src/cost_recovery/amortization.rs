#[path = "amortization/source.rs"]
mod source;
use crate::amortization_records::records;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{form_count, line_cents, push_line};
use crate::{BusinessStartup, BusinessTimberCost};
use crate::{FederalResult, FormLine, TaxForm};
use alloc::{collections::BTreeSet, format};
pub use source::{
    BusinessIntangible, BusinessIntangiblePage, IntangibleAcquisitionContext,
    IntangibleConsideration, IntangibleCost, IntangibleEventKind, IntangibleKind,
    IntangibleLeaseStatus, IntangibleObtained, IntangibleObtainedPage, IntangibleOriginEvidence,
    IntangiblePriorClaimed, IntangiblePriorClaimedPage, IntangibleSource, IntangibleSourcePage,
    IntangibleUseEnd,
};

impl BusinessIntangible {
    pub(crate) fn report(
        &self,
        r: &mut FederalResult,
        id: i64,
        index: usize,
        #[cfg(test)] source: &str,
    ) {
        if let Some(key) = self.boundary() {
            crate::issue_localized!(
                r,
                &crate::IssueContext::income_id(id),
                source,
                key,
                &crate::Message::new(key, alloc::vec![]).english()
            );
            return;
        }
        // Amortization that begins in 2026 is Form 4562-B line 1; the earlier
        // starts are only the line 2 total (report_form_4562b).
        let begins_current_year = self.started() / 10000 == 2026;
        let tag = |line: &str| begins_current_year.then(|| FormLine::new(TaxForm::Form4562B, line));
        let inputs = ["income.business.intangibles"];
        let source = "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol4/pdf/CFR-2025-title26-vol4-sec1-197-2.pdf";
        if begins_current_year {
            push_line(
                r,
                tag("1(c)"),
                &format!("business_{id}_intangible_{index}_amortizable"),
                crate::kernel::ratio::checked_sum(
                    core::iter::once(self.cost).chain(self.additions.iter().map(|a| a.amount)),
                )
                .expect("validated intangible basis"),
                &inputs,
                source,
                crate::Message::new(
                    "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZABLE",
                    alloc::vec![
                        crate::MessageArg::Text(self.label.clone()),
                        crate::MessageArg::DateOnly(i64::from(self.started())),
                    ],
                ),
            );
            form_count(
                r,
                FormLine::new(TaxForm::Form4562B, "1(e)"),
                &format!("business_{id}_intangible_{index}_period"),
                180,
                &inputs,
                source,
                msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_PERIOD"),
            );
        }
        push_line(
            r,
            tag("1(f)"),
            &format!("business_{id}_intangible_{index}_amortization"),
            self.deduction(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZATION"),
        );
        line_cents(
            r,
            &format!("business_{id}_intangible_{index}_remaining_basis"),
            self.remaining(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_REMAINING_BASIS"),
        );
    }
}
// Form 4562-B lines 2 and 3 of one business: line 2 is the 2026 amortization of
// costs that began in an earlier year, line 3 adds line 1 column (f). Printed
// whenever the business amortizes anything.
pub(crate) fn report_form_4562b(
    r: &mut FederalResult,
    id: i64,
    startup: &crate::BusinessStartup,
    intangibles: &[BusinessIntangible],
    timber: &[crate::BusinessTimberCost],
    other: &[crate::OtherAmortizableCost],
    assets: &[crate::BusinessAsset],
) {
    if startup.amortization_begins().is_none()
        && intangibles.is_empty()
        && timber.is_empty()
        && !crate::cohorts::cost_cohorts(other, 2026, assets)
            .parts
            .iter()
            .any(|part| part.as_ref().is_ok_and(|part| part.months > 0 || part.amortization != 0))
    {
        return;
    }
    let earlier_start = |started: u32| started / 10000 < 2026;
    let earlier = startup
        .amortization_begins()
        .filter(|s| earlier_start(*s))
        .map_or(0, |_| startup.amortization())
        + intangibles
            .iter()
            .filter(|i| i.boundary().is_none() && earlier_start(i.started()))
            .map(BusinessIntangible::deduction)
            .sum::<i64>();
    let timber = crate::timber::totals(timber).expect("validated timber form totals");
    let other = crate::cohorts::totals(other, assets).expect("validated cost cohort totals");
    let earlier = earlier.checked_add(other[1]).expect("validated amortization total");
    let current_other = other[2];
    let earlier = earlier.checked_add(timber[1]).expect("validated amortization total");
    let current = startup
        .amortization_begins()
        .filter(|s| !earlier_start(*s))
        .map_or(0, |_| startup.amortization())
        + intangibles
            .iter()
            .filter(|i| i.boundary().is_none() && !earlier_start(i.started()))
            .map(BusinessIntangible::deduction)
            .sum::<i64>();
    let current = current.checked_add(current_other).expect("validated amortization total");
    let current = current.checked_add(timber[2]).expect("validated amortization total");
    let total = earlier.checked_add(current).expect("validated amortization total");
    emit_lines(
        r,
        &mut (),
        [
            ("2", "earlier", earlier, msg!("EXPLAIN_BUSINESS_N_FORM_4562B_EARLIER")),
            ("3", "total", total, msg!("EXPLAIN_BUSINESS_N_FORM_4562B_TOTAL")),
        ],
        |(line, suffix, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4562B, line)),
                &format!("business_{id}_form_4562b_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.business.startup", "income.business.intangibles"],
                "https://www.irs.gov/instructions/i4562",
                explanation,
            ))
        },
    );
}

pub(crate) fn identity_issue(
    draft: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let associations = crate::source_associations::SourceAssociations::new(draft);
    for entry in &draft.income {
        for (index, record) in records(&entry.income).iter().enumerate() {
            if let IntangibleObtained::Renewal { previous_intangible_id: Some(_) } =
                record.source.obtained
            {
                let valid = associations
                    .intangible_predecessor(entry.id, record.id)
                    .ok()
                    .flatten()
                    .is_some_and(|previous| {
                        previous.acquired < record.acquired
                            && match &record.source.first_existed {
                                Some(IntangibleOriginEvidence::CreatedOn { date }) => {
                                    *date <= previous.acquired
                                }
                                Some(IntangibleOriginEvidence::CreatedAfter { date }) => {
                                    *date < previous.acquired
                                }
                                _ => true,
                            }
                    });
                if !valid {
                    return Some((
                        entry.id,
                        crate::validation::ValidationIssue::new("INTANGIBLE_PREDECESSOR_INVALID")
                            .at_field("previous_intangible_id")
                            .in_field("Renewal")
                            .in_field("obtained")
                            .in_field("source")
                            .in_list("intangibles", index)
                            .in_field("more"),
                    ));
                }
            }
        }
    }
    None
}

// A consolidated original direct property/year cohort is entered once across
// activities. This identifies the actual history; it is not a record-count cap.
pub(crate) fn timber_source_issue(
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

pub(crate) fn timber_return_issue(
    d: &crate::FederalDraft,
) -> Option<crate::validation::ValidationIssue> {
    timber_source_issue(d).map(|(_, issue)| crate::validation::ValidationIssue::new(issue.key))
}

pub(crate) fn report_timber(
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

impl BusinessStartup {
    pub(crate) fn report(&self, r: &mut FederalResult, id: i64, #[cfg(test)] source: &str) {
        let Self::Costs { started, .. } = *self else {
            return;
        };

        crate::require_localized!(
            r,
            (20041023..=20261231).contains(&started),
            &crate::IssueContext::income_id(id),
            source,
            "ISSUE_CONFIRM_COMPLETE_SECTION_195_START_UP_COSTS_THE_ORIGINAL_DEDUCTION_D6F699EB",
            "Confirm complete section 195 start-up costs, the original deduction/capitalization election and an active business begun after October 22, 2004 and continuing through 2026. Disposition, missing prior deductions or other special treatment needs its own calculation.",
        );
        // Amortization that begins in 2026 is Form 4562-B line 1; the earlier
        // starts are only the line 2 total (amortization::report_form_4562b).
        let begins_current_year = self.amortization_begins().is_some_and(|s| s / 10000 == 2026);
        let tag = |line: &str| begins_current_year.then(|| FormLine::new(TaxForm::Form4562B, line));
        let amortizable = match *self {
            Self::Costs { costs, .. } if self.deducted() => costs - self.initial(),
            _ => 0,
        };
        let inputs = ["income.business.startup"];
        let source = "https://www.irs.gov/instructions/i4562";
        push_line(
            r,
            None,
            &format!("business_{id}_startup_initial"),
            self.initial_current(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_INITIAL"),
        );
        if begins_current_year {
            push_line(
                r,
                tag("1(c)"),
                &format!("business_{id}_startup_amortizable"),
                amortizable,
                &inputs,
                source,
                crate::Message::new(
                    "EXPLAIN_BUSINESS_N_STARTUP_AMORTIZABLE",
                    alloc::vec![crate::MessageArg::DateOnly(i64::from(started))],
                ),
            );
            form_count(
                r,
                FormLine::new(TaxForm::Form4562B, "1(e)"),
                &format!("business_{id}_startup_period"),
                180,
                &inputs,
                source,
                msg!("EXPLAIN_BUSINESS_N_STARTUP_PERIOD"),
            );
        }
        push_line(
            r,
            tag("1(f)"),
            &format!("business_{id}_startup_amortization"),
            self.amortization(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_AMORTIZATION"),
        );
        line_cents(
            r,
            &format!("business_{id}_startup_remaining_basis"),
            self.remaining(),
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_STARTUP_N_REMAINING_BASIS"),
        );
    }
}
