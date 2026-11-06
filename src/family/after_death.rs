// Coverage: Pub. 559 income in respect of a decedent on a final return — Calculation for Forms
//   1099-INT, 1099-DIV and 1099-R (audited 2026-10-10, unit B9-3).
//   Lines: the part of a statement addressed to a person who died in 2026 that was paid for the
//   time after the death is income in respect of a decedent (section 691(a)(1)); it belongs to the
//   estate or the recipient and is subtracted from the statement before the return is figured
//   (Pub. 559 "Interest and Dividend Income (Forms 1099)", the "Nominee Distribution" steps, and
//   "Wages"/"Inherited IRAs" for a pension payment). 1099-INT boxes 1, 3 and 8; 1099-DIV boxes 1a,
//   1b, 2a and 12 plus each separately documented premium, private-activity, foreign-tax,
//   REIT and special-capital component; 1099-R boxes 1 and 2a. Withholding stays on the final
//   return. Disjoint nominee and later-recipient components are checked together before
//   subtraction. The existing retirement scope limitation remains unchanged.
//   Evidence: src/tests/federal/general.rs final_return_* (fixtures F1), interview.rs after-death pages;
//   Pub. 559, i1040gi "Death of a Taxpayer".
use crate::federal_model::ReviewNote;
use crate::{
    FederalDraft, FederalIncome, FederalRetirement, IssueContext, Message, MessageArg,
    RetirementTaxable, RetirementTreatment,
};
use alloc::vec::Vec;

pub use crate::after_death_income_source::{DividendsAfterDeath, InterestAfterDeath};

// The after-death part a stored statement carries, whatever its kind.
pub(crate) fn recorded(income: &FederalIncome) -> bool {
    recorded_issue(income).is_some()
}
pub(crate) fn recorded_issue(income: &FederalIncome) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let issue =
        || ValidationIssue::new("ISSUE_AFTER_DEATH_WITHOUT_A_DEATH").at_field("after_death");
    if let Some(index) = crate::investment_statements::annual_records(income)
        .and_then(|a| a.records.iter().position(|r| r.after_death.is_some()))
    {
        return Some(
            issue()
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    match income {
        FederalIncome::Interest { data }
            if crate::FederalInterest::from(data).after_death.is_some() =>
        {
            Some(issue())
        }
        FederalIncome::Dividend { data } if data.after_death.is_some() => {
            Some(issue().in_field("answers"))
        }
        FederalIncome::Retirement { data }
            if crate::FederalRetirement::from(data).after_death.is_some() =>
        {
            Some(issue())
        }
        _ => None,
    }
}

fn retirement_has_other_components(v: &FederalRetirement) -> bool {
    !matches!(
        v.ordinary_distribution,
        Some(RetirementTreatment::Ordinary)
            | Some(RetirementTreatment::Early)
            | Some(RetirementTreatment::EarlySimple)
            | Some(RetirementTreatment::Exception { .. })
    ) || !matches!(
        v.taxable,
        Some(RetirementTaxable::Payer)
            | Some(RetirementTaxable::Simplified { .. })
            | Some(RetirementTaxable::IraBasis)
    )
}

// Pub. 559: leave off each statement of a person who died in 2026 the part paid
// after the death. A part recorded for a person who is alive stays on the
// statement, where the calculation reports it (`recorded`).
pub(crate) fn apply(d: &mut FederalDraft, notes: &mut Vec<ReviewNote>) -> bool {
    let mut financial_omission = false;
    let Some(people) = d.people.clone() else {
        return false;
    };
    for entry in d.income.iter_mut() {
        let Some(person) = people.person(entry.owner.owned()).filter(|p| p.died.is_some()) else {
            continue;
        };
        let context = crate::interview::income_issue_context(entry);
        let row = MessageArg::Nested(alloc::boxed::Box::new(context.label()));
        let name = MessageArg::Encoded(person.name());
        let (left_out, amount) = match &mut entry.income {
            FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } => {
                let Some(part) = data.after_death.clone() else {
                    continue;
                };
                if !part.valid(&crate::FederalInterest::from(&*data)) {
                    (true, 0)
                } else {
                    data.after_death = None;
                    data.form.interest_income -= part.taxable;
                    data.form.us_savings_bond_and_treasury_interest -= part.us_obligations;
                    data.form.tax_exempt_interest -= part.exempt;
                    data.form.specified_private_activity_bond_interest -= part.private_activity;
                    data.form.foreign_tax_paid -= part.foreign_tax;
                    data.form.bond_premium -= part.bond_premium;
                    data.form.bond_premium_on_treasury_obligations -= part.treasury_bond_premium;
                    data.form.bond_premium_on_tax_exempt_bond -= part.tax_exempt_bond_premium;
                    data.accrued_interest -= part.accrued_interest;
                    data.form.market_discount -= part.market_discount;
                    (false, part.income())
                }
            }
            FederalIncome::Dividend { data } => {
                let Some(part) = data.after_death.as_ref().map(|answer| answer.record().clone())
                else {
                    continue;
                };
                if !part.valid(&crate::FederalDividend::from(&*data)) {
                    (true, 0)
                } else {
                    data.after_death = None;
                    let form = &mut data.form;
                    crate::dividend_source::reduce_box(&mut form.ordinary, part.ordinary);
                    crate::dividend_source::reduce_box(&mut form.qualified, part.qualified);
                    crate::dividend_source::reduce_box(
                        &mut form.capital_distributions,
                        part.capital_distributions,
                    );
                    crate::dividend_source::reduce_box(
                        &mut form.unrecaptured_distributions,
                        part.unrecaptured_distributions,
                    );
                    crate::dividend_source::reduce_box(
                        &mut form.collectible_distributions,
                        part.collectible_distributions,
                    );
                    crate::dividend_source::reduce_box(
                        &mut form.section_1202_gain,
                        part.section_1202_gain,
                    );
                    if form.section199a.unwrap_or(0) != 0 {
                        crate::dividend_source::reduce_box(&mut form.section199a, part.section199a);
                    }
                    crate::dividend_source::reduce_box(
                        &mut form.exempt_dividends,
                        part.exempt_dividends,
                    );
                    crate::dividend_source::reduce_box(
                        &mut form.private_activity_dividends,
                        part.private_activity_dividends,
                    );
                    crate::dividend_source::reduce_box(&mut form.foreign_tax, part.foreign_tax);
                    crate::dividend_source::reduce_box(
                        &mut form.section_897_ordinary,
                        part.section_897_ordinary,
                    );
                    crate::dividend_source::reduce_box(
                        &mut form.section_897_capital,
                        part.section_897_capital,
                    );
                    (false, part.income())
                }
            }
            FederalIncome::Retirement { data: source } => {
                let record = crate::FederalRetirement::from(&*source);
                let Some(part) = record.after_death.clone() else {
                    continue;
                };
                source.clear_after_death();
                if retirement_has_other_components(&record) || !part.valid(&record) {
                    (true, 0)
                } else {
                    let payer = matches!(record.taxable, Some(RetirementTaxable::Payer));
                    source.reduce_distribution(part.gross, if payer { part.taxable } else { 0 });
                    (false, part.taxable)
                }
            }
            _ => continue,
        };
        financial_omission |= left_out;
        let (key, args) = if left_out {
            ("WARN_AFTER_DEATH_SPLIT_LEFT_OUT", alloc::vec![row])
        } else if amount > 0 {
            (
                "NOTE_INCOME_IN_RESPECT_OF_A_DECEDENT",
                alloc::vec![row, MessageArg::usd_cents(amount), name],
            )
        } else {
            continue;
        };
        note_row(notes, key, args, &context);
    }
    financial_omission
}

fn note_row(notes: &mut Vec<ReviewNote>, key: &str, args: Vec<MessageArg>, context: &IssueContext) {
    let message = Message::new(key, args);
    let note = ReviewNote::scoped(
        #[cfg(test)]
        message.english(),
        message,
        context.clone(),
    );
    if !notes.iter().any(|old| old.same_notice(&note)) {
        notes.push(note);
    }
}
