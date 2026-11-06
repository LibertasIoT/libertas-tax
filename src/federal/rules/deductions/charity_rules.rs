// charity items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct CharitySourcesOut {
    pub(super) hosted_student_expenses: i64,
    pub(super) whaling_expenses: i64,
    pub(super) partner_cash: i64,
    pub(super) cash_charity: i64,
}

pub(super) fn charity_sources_phase(pass: &Pass<'_>, r: &mut FederalResult) -> CharitySourcesOut {
    let &Pass { d, ded, qcd, screen, c, .. } = pass;
    let volunteer_expenses =
        ded.volunteer_expenses.iter().map(crate::VolunteerExpense::cents).sum::<i64>();
    let hosted_student_expenses =
        ded.hosted_students.iter().map(crate::HostedStudent::allowed).sum::<i64>();
    let whaling_eligible = ded
        .whaling_captains
        .iter()
        .map(crate::WhalingCaptainClaim::eligible_expense_cents)
        .sum::<i64>();
    let whaling_expenses = ded
        .whaling_captains
        .iter()
        .map(crate::WhalingCaptainClaim::statutory_claim_cents)
        .sum::<i64>();
    // K-1 (1065) box 13 code A: the partner's share of the partnership's
    // 60%-class cash gifts joins the filer's own cash (section 702(a)(4)).
    // K-1 (1120-S) box 12 code A joins the same way (section 1366(a)(1)(A)).
    let partner_cash = crate::partnership::cash_charity(d) + crate::scorp::cash_charity(d);
    let cash_charity = ded.cash_charity
        + ded.fund_cash_charity
        + volunteer_expenses
        + qcd.cash_gifts
        + partner_cash;
    if !ded.charity_carryovers.is_empty()
        || !ded.property_gifts.is_empty()
        || !ded.vehicle_gifts.is_empty()
    {
        crate::require_localized!(
            r,
            !screen.nol.present()
                && !screen.atnol.present()
                && estate_final_nol(d).is_none()
                && matches!(
                    c.foreign_tax,
                    crate::ForeignTaxClaim::No | crate::ForeignTaxClaim::Deduction { .. }
                ),
            &crate::IssueContext::Section(crate::TaxSection::GiftCarryovers).manual(),
            "Charitable carryover coordination",
            "ISSUE_CHARITY_NOL_FTC_COORDINATION",
            "Charitable carryovers or noncash gifts with a prior regular/AMT NOL or Form 1116 need the retained loss-year and foreign-allocation worksheets.",
        );
    }
    emit_lines(r, &mut (), ded.volunteer_expenses.iter().enumerate(), |(index, expense), _, _| {
        Some(calculation_row(
            None,
            &alloc::format!("charity_volunteer_{}_expense", index + 1),
            LineValue::MoneyCents(expense.cents()),
            &["deductions.volunteer_expenses"],
            "https://www.irs.gov/publications/p526",
            msg!("EXPLAIN_CHARITY_VOLUNTEER_N_EXPENSE"),
        ))
    });
    for (index, student) in ded.hosted_students.iter().enumerate() {
        line_cents(
            r,
            &alloc::format!("charity_hosted_student_{}_eligible", index + 1),
            student.allowed(),
            &["deductions.hosted_students"],
            "https://www.irs.gov/publications/p526",
            msg!("EXPLAIN_CHARITY_HOSTED_STUDENT_N_ELIGIBLE"),
        );
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Hosted student {}: retain the written sponsor agreement, actual monthly expenses and a statement of joining date {}, full-time school attendance {}–{}, and school {} in {}. An itemized claim requires these attachments; the App calculates the on-screen amount but does not prepare them.",
            student.label,
            student.joined,
            student.school_start,
            student.school_end,
            student.school,
            student.school_location,
        ), Message::new("NOTE_HOSTED_STUDENT_RECORDS", alloc::vec![MessageArg::Text(student.label.clone()), MessageArg::DateOnly(i64::from(student.joined)), MessageArg::DateOnly(i64::from(student.school_start)), MessageArg::DateOnly(i64::from(student.school_end)), MessageArg::Text(student.school.clone()), MessageArg::Text(student.school_location.clone())])));
    }
    for claim in &ded.whaling_captains {
        let owner = match claim.owner {
            crate::Owner::Taxpayer => "taxpayer",
            crate::Owner::Spouse => "spouse",
        };
        line_cents(
            r,
            &alloc::format!("charity_whaling_{owner}_eligible_expenses"),
            claim.eligible_expense_cents(),
            &["deductions.whaling_captains"],
            "https://www.irs.gov/irb/2006-47_IRB#RP-2006-50",
            msg!("EXPLAIN_CHARITY_WHALING_N_ELIGIBLE_EXPENSES"),
        );
        line_cents(
            r,
            &alloc::format!("charity_whaling_{owner}_statutory_claim"),
            claim.statutory_claim_cents(),
            &["deductions.whaling_captains"],
            "https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm",
            msg!("EXPLAIN_CHARITY_WHALING_N_STATUTORY_CLAIM"),
        );
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("Whaling captain ({owner}): retain AEWC recognition, copies of reports submitted to the AEWC, contemporaneous trip departure/return and area logs, expense reports with date/amount/nature, receipts for expenses of at least $75, and evidence of any qualifying involuntary record loss and reconstruction. The App calculates the on-screen deduction but does not prepare attachments."),
            Message::new(if claim.owner == Owner::Taxpayer { "NOTE_WHALING_CAPTAIN_TAXPAYER" } else { "NOTE_WHALING_CAPTAIN_SPOUSE" }, alloc::vec![]),
        ));
    }
    if !ded.whaling_captains.is_empty() {
        line_cents(
            r,
            "charity_whaling_eligible_expenses",
            whaling_eligible,
            &["deductions.whaling_captains"],
            "https://www.irs.gov/irb/2006-47_IRB#RP-2006-50",
            msg!("EXPLAIN_CHARITY_WHALING_ELIGIBLE_EXPENSES"),
        );
        line_cents(
            r,
            "charity_whaling_statutory_claim",
            whaling_expenses,
            &["charity_whaling_eligible_expenses"],
            "https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm",
            msg!("EXPLAIN_CHARITY_WHALING_STATUTORY_CLAIM"),
        );
    }
    CharitySourcesOut { hosted_student_expenses, whaling_expenses, partner_cash, cash_charity }
}

pub(super) struct CharityWorksheetIn {
    pub(super) agi: i64,
    pub(super) cash_charity: i64,
    pub(super) hosted_student_expenses: i64,
    pub(super) whaling_expenses: i64,
    pub(super) partner_cash: i64,
}

pub(super) struct CharityWorksheetOut {
    pub(super) charity_claims: crate::charity::ClassClaims,
    pub(super) gifts: Vec<crate::PropertyGift>,
    pub(super) charity: crate::charity::Worksheet,
    pub(super) noncash_claim: i64,
    pub(super) charity_excess: i64,
    pub(super) charity_carry: i64,
    pub(super) charity_deduction: i64,
    pub(super) standard_cash_allowed: i64,
}

pub(super) fn charity_worksheet_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    input: CharityWorksheetIn,
) -> core::ops::ControlFlow<(), CharityWorksheetOut> {
    let &Pass { ded, qcd, .. } = pass;
    let CharityWorksheetIn {
        agi,
        cash_charity,
        hosted_student_expenses,
        whaling_expenses,
        partner_cash,
    } = input;
    // One class worksheet owns current gifts, K-1 inputs and original-limit carries.
    let mut charity_claims = crate::charity::ClassClaims::default();
    charity_claims.current[1] = dollars(cash_charity);
    charity_claims.cash[1] = charity_claims.current[1];
    charity_claims.current[2] = dollars(
        ded.other_cash_charity + hosted_student_expenses + whaling_expenses + t.k1_charity[0],
    );
    charity_claims.cash[2] = dollars(
        ded.other_cash_charity + hosted_student_expenses + whaling_expenses + t.k1_charity_cash30,
    );
    charity_claims.current[0] = dollars(t.k1_charity[1]);
    charity_claims.current[3] = dollars(t.k1_charity[2]);
    let gifts = match ded.gifts() {
        Ok(gifts) => gifts,
        Err(issue) => {
            let section = if issue.key == "VEHICLE_GIFT_FACTS_NEEDED" {
                crate::TaxSection::GiftCarryovers
            } else {
                crate::TaxSection::Documents
            };
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(section),
                "Vehicle gifts",
                issue.key,
                "Complete the printed vehicle certification before calculating this gift.",
            );
            withhold_required_source_result(r);
            r.financial_source_scope_complete = false;
            return core::ops::ControlFlow::Break(());
        }
    };
    for gift in &gifts {
        let (class, amount) = gift.claim(ded.basis_election);
        charity_claims.current[class.index()] += dollars(amount);
    }
    let charity = crate::charity::Worksheet::calculate(
        agi,
        charity_claims,
        &ded.charity_carryovers,
        ded.basis_election,
    );
    let noncash_claim = gifts.iter().map(|g| g.claim(ded.basis_election).1).sum::<i64>()
        + positive(t.k1_charity[0] - t.k1_charity_cash30)
        + t.k1_charity[1]
        + t.k1_charity[2];
    let charity_excess = charity.classes.iter().map(|v| v.current_excess).sum::<i64>();
    let charity_carry = charity.classes.iter().map(|v| v.current_carry).sum::<i64>();
    let charity_deduction = nol_run.limit_charity(&charity);
    // Non-itemizers exclude DAF/supporting and second-category gifts, service costs and carries.
    let standard_cash_allowed = phase::cap(
        dollars(ded.cash_charity + qcd.cash_gifts + partner_cash),
        [fraction(positive(agi), 3, 5)],
    );
    core::ops::ControlFlow::Continue(CharityWorksheetOut {
        charity_claims,
        gifts,
        charity,
        noncash_claim,
        charity_excess,
        charity_carry,
        charity_deduction,
        standard_cash_allowed,
    })
}

pub(super) struct CharitySettlementReportingIn<'a> {
    pub(super) charity: &'a crate::charity::Worksheet,
    pub(super) gifts: &'a [crate::PropertyGift],
    pub(super) use_itemized: bool,
    pub(super) charity_deduction: i64,
    pub(super) charity_carry: i64,
    pub(super) charity_excess: i64,
    pub(super) hosted_student_expenses: i64,
    pub(super) whaling_expenses: i64,
}

pub(super) fn charity_settlement_reporting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &mut crate::nol::NolRun,
    input: CharitySettlementReportingIn<'_>,
) {
    let &Pass { ded, .. } = pass;
    let CharitySettlementReportingIn {
        charity,
        gifts,
        use_itemized,
        charity_deduction,
        charity_carry,
        charity_excess,
        hosted_student_expenses,
        whaling_expenses,
    } = input;
    nol_run.charity_deduction = if use_itemized { charity_deduction } else { 0 };
    nol_run.charity_class_deductions = core::array::from_fn(|i| {
        if use_itemized {
            charity.classes[i].current_allowed - charity.classes[i].current_floor
        } else {
            0
        }
    });
    nol_run.charity_class_carry = core::array::from_fn(|i| {
        if use_itemized {
            charity.classes[i].current_carry
        } else {
            charity.classes[i].current_excess
        }
    });
    // The percentage-limit excess retains its gift year even when the filer
    // takes the standard deduction. Section 170(p) does not apply the itemizer
    // floor, so only itemizers can add the floor amount to this carryover.
    nol_run.charity_carry = if use_itemized { charity_carry } else { charity_excess };
    crate::charity::report(
        r,
        ded,
        gifts,
        charity,
        use_itemized,
        nol_run.charity_nol_reduction,
        nol_run.charity_class_reduction,
    );
    for (id, amount) in [
        (
            "charity_other_30_percent_deduction",
            charity.classes[2].current_allowed - charity.classes[2].current_floor,
        ),
        (
            "charity_other_30_percent_return_year_carryforward",
            if use_itemized {
                charity.classes[2].current_carry - nol_run.charity_class_reduction[2]
            } else {
                charity.classes[2].current_excess - nol_run.charity_class_reduction[2]
            },
        ),
    ] {
        line(
            r,
            id,
            if use_itemized || id.ends_with("carryforward") { amount } else { 0 },
            &[
                "deductions.hosted_students",
                "deductions.whaling_captains",
                "deductions.other_cash_charity",
                "agi",
            ],
            "https://www.irs.gov/publications/p526",
            msg!("EXPLAIN_CHARITY_CLASS_WORKSHEET"),
        );
    }
    for (label, present, other) in [
        ("whaling", whaling_expenses > 0, hosted_student_expenses),
        ("hosted_student", hosted_student_expenses > 0, whaling_expenses),
    ] {
        if present && other == 0 && ded.other_cash_charity == 0 && t.k1_charity[0] == 0 {
            line(
                r,
                &alloc::format!("charity_{label}_deduction"),
                if use_itemized {
                    charity.classes[2].current_allowed - charity.classes[2].current_floor
                } else {
                    0
                },
                &["deductions.hosted_students", "deductions.whaling_captains", "agi"],
                "https://www.irs.gov/publications/p526",
                msg!("EXPLAIN_CHARITY_CLASS_WORKSHEET"),
            );
            line(
                r,
                &alloc::format!("charity_{label}_return_year_carryforward"),
                if use_itemized {
                    charity.classes[2].current_carry - nol_run.charity_class_reduction[2]
                } else {
                    charity.classes[2].current_excess - nol_run.charity_class_reduction[2]
                },
                &["deductions.hosted_students", "deductions.whaling_captains", "agi"],
                "https://www.irs.gov/publications/p526",
                msg!("EXPLAIN_CHARITY_CLASS_WORKSHEET"),
            );
        }
    }
}
