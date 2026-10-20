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
    FederalDividend, FederalDraft, FederalIncome, FederalInterest, FederalRetirement, IssueContext,
    Message, MessageArg, RetirementTaxable, RetirementTreatment,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Interest for the time after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestAfterDeath {
    /// Part of box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// Part of box 3
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Part of box 8
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private activity belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,

    /// Foreign tax belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_tax: i64,

    /// Bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub bond_premium: i64,

    /// Treasury bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,

    /// Tax exempt bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,

    /// Accrued interest belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub accrued_interest: i64,

    /// Market discount belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub market_discount: i64,
}

/// Dividends for the time after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DividendsAfterDeath {
    /// Part of box 1a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Part of box 1b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Part of box 2a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub capital_distributions: i64,
    /// Part of box 12
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_dividends: i64,
    /// Unrecaptured distributions belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_distributions: i64,

    /// Collectible distributions belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub collectible_distributions: i64,

    /// Section 1202 gain belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_1202_gain: i64,

    /// Section199a belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section199a: i64,

    /// Private activity dividends belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity_dividends: i64,

    /// Foreign tax belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_tax: i64,

    /// Section 897 ordinary belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_897_ordinary: i64,

    /// Section 897 capital belonging to the later recipient
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_897_capital: i64,
}

/// Payments made after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RetirementAfterDeath {
    /// Part of box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub gross: i64,
    /// Part of box 2a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub taxable: i64,
}

impl InterestAfterDeath {
    fn income(&self) -> i64 {
        self.taxable + self.us_obligations + self.market_discount
    }
    pub(crate) fn valid(&self, statement: &FederalInterest) -> bool {
        let mut checked = statement.clone();
        checked.after_death = Some(self.clone());
        crate::investment_statements::int_rule(&checked).is_none()
    }
}
impl DividendsAfterDeath {
    fn income(&self) -> i64 {
        self.ordinary + self.capital_distributions
    }
    pub(crate) fn valid(&self, statement: &FederalDividend) -> bool {
        let mut checked = statement.clone();
        checked.after_death = Some(self.clone());
        crate::investment_statements::div_rule(&checked).is_none()
    }
}

impl RetirementAfterDeath {
    pub(crate) fn valid(&self, statement: &FederalRetirement) -> bool {
        let printed = match statement.taxable {
            RetirementTaxable::Payer { amount } => amount,
            _ => statement.gross,
        };
        (0..=statement.gross).contains(&self.gross)
            && (0..=printed).contains(&self.taxable)
            && self.taxable <= self.gross
    }
}

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
        FederalIncome::Interest { data } if data.after_death.is_some() => Some(issue()),
        FederalIncome::Dividend { data } if data.after_death.is_some() => {
            Some(issue().in_field("answers"))
        }
        FederalIncome::Retirement { data } if data.after_death.is_some() => Some(issue()),
        _ => None,
    }
}

fn retirement_has_other_components(v: &FederalRetirement) -> bool {
    !matches!(
        v.ordinary_distribution,
        RetirementTreatment::Ordinary
            | RetirementTreatment::Early
            | RetirementTreatment::EarlySimple
            | RetirementTreatment::Exception { .. }
    ) || !matches!(
        v.taxable,
        RetirementTaxable::Payer { .. }
            | RetirementTaxable::Simplified { .. }
            | RetirementTaxable::IraBasis
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
        let Some(person) = people.person(entry.owner).filter(|p| p.died.is_some()) else {
            continue;
        };
        let context = crate::interview::income_issue_context(entry);
        let row = MessageArg::Nested(alloc::boxed::Box::new(context.label()));
        let name = MessageArg::Encoded(person.name());
        let (left_out, amount) = match &mut entry.income {
            FederalIncome::Interest { data } => {
                let Some(part) = data.after_death.clone() else {
                    continue;
                };
                if !part.valid(data) {
                    (true, 0)
                } else {
                    data.after_death = None;
                    data.taxable -= part.taxable;
                    data.us_obligations -= part.us_obligations;
                    data.exempt -= part.exempt;
                    data.private_activity -= part.private_activity;
                    data.foreign_tax -= part.foreign_tax;
                    data.bond_premium -= part.bond_premium;
                    data.treasury_bond_premium -= part.treasury_bond_premium;
                    data.tax_exempt_bond_premium -= part.tax_exempt_bond_premium;
                    data.accrued_interest -= part.accrued_interest;
                    data.market_discount -= part.market_discount;
                    (false, part.income())
                }
            }
            FederalIncome::Dividend { data } => {
                let Some(part) = data.after_death.clone() else {
                    continue;
                };
                if !part.valid(data) {
                    (true, 0)
                } else {
                    data.after_death = None;
                    data.ordinary -= part.ordinary;
                    data.qualified -= part.qualified;
                    data.capital_distributions -= part.capital_distributions;
                    data.unrecaptured_distributions -= part.unrecaptured_distributions;
                    data.collectible_distributions -= part.collectible_distributions;
                    data.section_1202_gain -= part.section_1202_gain;
                    if let crate::ReitDividends::Reported { amount, .. } = &mut data.section199a {
                        *amount -= part.section199a;
                    }
                    data.exempt_dividends -= part.exempt_dividends;
                    data.private_activity_dividends -= part.private_activity_dividends;
                    data.foreign_tax -= part.foreign_tax;
                    data.section_897_ordinary -= part.section_897_ordinary;
                    data.section_897_capital -= part.section_897_capital;
                    (false, part.income())
                }
            }
            FederalIncome::Retirement { data } => {
                let Some(part) = data.after_death.take() else {
                    continue;
                };
                if retirement_has_other_components(data) || !part.valid(data) {
                    (true, 0)
                } else {
                    data.gross -= part.gross;
                    if let RetirementTaxable::Payer { amount } = &mut data.taxable {
                        *amount -= part.taxable;
                    }
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
