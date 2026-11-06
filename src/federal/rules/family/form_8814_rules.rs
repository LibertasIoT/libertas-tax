// form_8814 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_child_election(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalChildElection,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    let lines = data.lines();
    t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
    if lines.reported() {
        t.injured_other_sources
            .insert(e.id, Some(exact::whole((lines.line_9 + lines.line_12) * 100)));
        t.child.qualified_by_owner[owner] += lines.line_9;
        t.child.investment_by_owner[owner] += lines.investment_income();
        t.child.tax_by_owner[owner] += lines.line_15;
        // Line 9 joins Form 1040 lines 3a and 3b, line 10
        // Schedule D line 13 with its rate shares, line 12
        // Schedule 1 line 8z, line 15 Form 1040 line 16.
        t.dividends += lines.line_9 * 100;
        t.qualified += lines.line_9 * 100;
        t.add_long(lines.line_10 * 100);
        t.schedule_d_distributions += lines.line_10 * 100;
        t.investment_gains[1] += lines.line_10 * 100;
        t.collectible_long += lines.collectible * 100;
        t.unrecaptured_long += lines.unrecaptured * 100;
        t.private_activity += lines.private_activity;
        t.child.qualified += lines.line_9;
        t.child.distributions += lines.line_10;
        t.child.other += lines.line_12;
        t.child.other_by_owner[owner] += lines.line_12;
        t.child.investment += lines.investment_income();
        t.child.exempt += lines.line_1b;
        t.child.household_income += lines.line_1b
            + dollars(data.nontaxable_social)
            + phase::cap(lines.line_4, [crate::year_constants::CHILD_ELECTION_BASE_DOLLARS]);
        t.child.tax += lines.line_15;
        crate::child_election::report(r, e.id, &lines);
        if data.foreign_account() {
            note_once_scoped(
                r,
                Message::new(
                    "NOTE_CHILD_FOREIGN_ACCOUNT",
                    alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                ),
                context.clone(),
            );
        }
    } else {
        note_once_scoped(
            r,
            Message::new(
                "NOTE_CHILD_ELECTION_NOT_REQUIRED",
                alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
            ),
            context.clone(),
        );
    }
    Ok(())
}

pub(super) fn form_8814_projection_phase(r: &mut FederalResult, t: &Totals) {
    // The Form 8814 elections' totals, each already inside the Form 1040
    // line it joins (3a/3b, Schedule D line 13, Schedule 1 line 8z, line 16).
    if t.child.tax != 0
        || t.child.other != 0
        || t.child.qualified != 0
        || t.child.distributions != 0
        || t.child.exempt != 0
    {
        emit_lines(
            r,
            &mut (),
            [
                (
                    "child_election_qualified_dividends",
                    t.child.qualified,
                    msg!("EXPLAIN_CHILD_ELECTION_QUALIFIED_DIVIDENDS"),
                ),
                (
                    "child_election_capital_gain_distributions",
                    t.child.distributions,
                    msg!("EXPLAIN_CHILD_ELECTION_CAPITAL_GAIN_DISTRIBUTIONS"),
                ),
                (
                    "child_election_other_income",
                    t.child.other,
                    msg!("EXPLAIN_CHILD_ELECTION_OTHER_INCOME"),
                ),
                (
                    "child_election_investment_income",
                    t.child.investment,
                    msg!("EXPLAIN_CHILD_ELECTION_INVESTMENT_INCOME"),
                ),
                (
                    "child_election_exempt_interest",
                    t.child.exempt,
                    msg!("EXPLAIN_CHILD_ELECTION_EXEMPT_INTEREST"),
                ),
                ("child_election_tax", t.child.tax, msg!("EXPLAIN_CHILD_ELECTION_TAX")),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["income.child_election"],
                    "https://www.irs.gov/instructions/i8814",
                    explanation,
                ))
            },
        );
    }
}
