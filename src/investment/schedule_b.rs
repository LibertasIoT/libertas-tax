// Schedule B presentation consumes the investment owner's retained projection.
// It is not a second interest/premium calculation. Latest 2026 draft instructions,
// editorial August 10, 2026 pp.B-1–B-3: payer listings and named adjustments;
// https://www.irs.gov/pub/irs-dft/i1040sb--dft.pdf . Draft provenance is explicit.
use crate::{FederalDraft, FederalIncome, FederalIncomeEntry, FederalResult, Message, MessageArg};
use alloc::{string::String, vec::Vec};

#[derive(Default)]
struct Listing {
    gross: i64,
    nominee: i64,
    estate: i64,
    prior: i64,
    accrued: i64,
    net: i64,
}
fn listing(original: &FederalIncomeEntry, retained: &FederalIncomeEntry) -> Listing {
    let parts = crate::investment_statements::components(retained).expect("investment statement");
    if let Some(records) = crate::investment_statements::annual_records(&original.income) {
        let total = |v: &crate::DebtComponents| {
            v.taxable
                + v.us_obligations
                + v.oid
                + v.other_periodic_interest
                + v.treasury_oid
                + v.market_discount
        };
        return Listing {
            gross: records.records.iter().map(|r| total(&r.amounts)).sum(),
            nominee: records.records.iter().filter_map(|r| r.nominee.as_ref()).map(total).sum(),
            estate: records.records.iter().filter_map(|r| r.after_death.as_ref()).map(total).sum(),
            prior: records.records.iter().map(|r| r.previously_reported).sum(),
            accrued: records
                .records
                .iter()
                .map(|r| {
                    let v = r.retained();
                    v.accrued_interest + v.treasury_accrued_interest
                })
                .sum(),
            net: parts.interest,
        };
    }
    match &original.income {
        FederalIncome::Interest { data: v } => Listing {
            gross: v.taxable + v.us_obligations + v.market_discount,
            nominee: v
                .nominee
                .as_ref()
                .map_or(0, |n| n.taxable + n.us_obligations + n.market_discount),
            estate: v
                .after_death
                .as_ref()
                .map_or(0, |a| a.taxable + a.us_obligations + a.market_discount),
            prior: v.adjustments.as_ref().map_or(0, |a| a.previously_reported),
            accrued: v.accrued_interest
                - v.nominee.as_ref().map_or(0, |n| n.accrued_interest)
                - v.after_death.as_ref().map_or(0, |a| a.accrued_interest),
            net: parts.interest,
        },
        FederalIncome::Oid { data: v } => Listing {
            gross: v.oid + v.other_periodic_interest + v.treasury_oid + v.market_discount,
            nominee: v.nominee.as_ref().map_or(0, |n| {
                n.oid + n.other_periodic_interest + n.treasury_oid + n.market_discount
            }),
            net: parts.interest,
            ..Default::default()
        },
        FederalIncome::Dividend { data: v } => Listing {
            gross: v.ordinary,
            nominee: v.nominee.as_ref().map_or(0, |n| n.ordinary),
            estate: v.after_death.as_ref().map_or(0, |a| a.ordinary),
            net: parts.ordinary_dividends,
            ..Default::default()
        },
        _ => unreachable!(),
    }
}
fn emit(
    r: &mut FederalResult,
    part: &str,
    id: i64,
    suffix: &str,
    amount: i64,
    label: &str,
    kind: &str,
) {
    crate::kernel::lines::push_line(
        r,
        Some(crate::FormLine::subrow(
            crate::TaxForm::ScheduleB,
            if part == "interest" { "1" } else { "5" },
        )),
        &alloc::format!("schedule_b_{part}_{id}_{suffix}"),
        amount,
        &["income.investment_statement"],
        "https://www.irs.gov/pub/irs-dft/i1040sb--dft.pdf",
        Message::new(
            "EXPLAIN_SCHEDULE_B_SOURCE",
            alloc::vec![
                MessageArg::Text(String::from(label)),
                MessageArg::Resource(String::from(kind)),
            ],
        ),
    );
}
fn answer(value: Option<bool>) -> MessageArg {
    MessageArg::Resource(String::from(match value {
        Some(true) => "SCHEDULE_B_YES",
        Some(false) => "SCHEDULE_B_NO",
        None => "SCHEDULE_B_NOT_APPLICABLE",
    }))
}

pub(crate) fn report(original: &FederalDraft, working: &FederalDraft, r: &mut FederalResult) {
    if r.state != crate::ResultState::EstimatedForSupportedScope {
        return;
    }
    let amount = |reference: &str| {
        r.return_lines
            .iter()
            .find(|row| {
                row.form_line.as_ref().is_some_and(|line| {
                    line.form == crate::TaxForm::Form1040 && line.line == reference
                })
            })
            .map_or(0, |row| row.amount)
    };
    let taxable = amount("2b");
    let dividends = amount("3b");
    let exclusion = r
        .lines
        .iter()
        .find(|line| line.id == "savings_bond_exclusion")
        .map_or(0, |line| line.amount);
    let interest = taxable + exclusion;
    let mut statements: Vec<(&FederalIncomeEntry, Listing)> = Vec::new();
    for entry in &working.income {
        if crate::investment_statements::components(entry).is_none() {
            continue;
        }
        let Some(raw) = original.income.iter().find(|raw| raw.id == entry.id) else {
            continue;
        };
        let mut row = listing(raw, entry);
        if let Some(share) = working.setup.as_ref().and_then(|setup| {
            crate::community::share(setup, entry.owner, entry.community, &entry.income)
        }) {
            row.gross = share.filer(row.gross);
            row.nominee = share.filer(row.nominee);
            row.estate = share.filer(row.estate);
            row.prior = share.filer(row.prior);
            row.accrued = share.filer(row.accrued);
            row.net = share.filer(row.net);
        }
        statements.push((raw, row));
    }
    // Seller-financed home mortgage interest is listed first, as instructed.
    statements.sort_by_key(|(entry, _)| {
        !matches!(&entry.income,
        FederalIncome::Interest { data } if data.seller_mortgage.is_some())
    });
    let first_required = interest > 150_000 || exclusion > 0 || statements.iter().any(|(e, row)|
        !matches!(e.income, FederalIncome::Dividend { .. }) && (row.gross != row.net
            || matches!(&e.income, FederalIncome::Interest { data } if data.seller_mortgage.is_some())));
    let second_required = dividends > 150_000
        || statements.iter().any(|(e, row)| {
            matches!(e.income, FederalIncome::Dividend { .. }) && row.gross != row.net
        });
    let screen = original.screening.as_ref();
    let third_required = taxable > 150_000
        || dividends > 150_000
        || screen.is_some_and(|s| s.foreign_account || s.foreign_trust);
    r.lines
        .retain(|line| line.form_line.as_ref().is_none_or(|f| f.form != crate::TaxForm::ScheduleB));
    let mut listed = [0_i64; 2];
    for (entry, row) in statements {
        let dividend = matches!(entry.income, FederalIncome::Dividend { .. });
        if if dividend { !second_required } else { !first_required } {
            continue;
        }
        let part = if dividend { "dividend" } else { "interest" };
        let label = if let FederalIncome::Interest { data } = &entry.income
            && let Some(buyer) = &data.seller_mortgage
        {
            alloc::format!("{}; {}; {}", buyer.buyer, buyer.buyer_tin, buyer.buyer_address)
        } else {
            entry.label.clone()
        };
        emit(r, part, entry.id, "payer", row.gross, &label, "SCHEDULE_B_PAYER");
        for (suffix, value, key) in [
            ("nominee", row.nominee, "SCHEDULE_B_NOMINEE"),
            ("estate", row.estate, "SCHEDULE_B_ESTATE"),
            ("prior", row.prior, "SCHEDULE_B_PRIOR"),
            ("accrued", row.accrued, "SCHEDULE_B_ACCRUED"),
            (
                "adjustment",
                row.gross - row.nominee - row.estate - row.prior - row.accrued - row.net,
                "SCHEDULE_B_ADJUSTMENT",
            ),
        ] {
            if value != 0 {
                emit(r, part, entry.id, suffix, -value, &label, key);
            }
        }
        listed[usize::from(dividend)] += row.net;
    }
    for (index, required, total, part) in
        [(0, first_required, interest, "interest"), (1, second_required, dividends, "dividend")]
    {
        if required && total != listed[index] {
            emit(r, part, 0, "other", total - listed[index], "", "SCHEDULE_B_OTHER");
        }
    }
    for (required, reference, id, total, explanation) in [
        (
            first_required,
            "2",
            "schedule_b_total_interest",
            interest,
            "EXPLAIN_SCHEDULE_B_TOTAL_INTEREST",
        ),
        (
            exclusion > 0,
            "3",
            "schedule_b_savings_bond_exclusion",
            exclusion,
            "EXPLAIN_SCHEDULE_B_SAVINGS_BOND_EXCLUSION",
        ),
        (
            first_required,
            "4",
            "schedule_b_taxable_interest",
            taxable,
            "EXPLAIN_SCHEDULE_B_TAXABLE_INTEREST",
        ),
        (
            second_required,
            "6",
            "schedule_b_ordinary_dividends",
            dividends,
            "EXPLAIN_SCHEDULE_B_ORDINARY_DIVIDENDS",
        ),
    ] {
        if required {
            crate::kernel::lines::push_line(
                r,
                Some(crate::FormLine::new(crate::TaxForm::ScheduleB, reference)),
                id,
                total,
                &["income.investment_statement", "savings_bond_exclusion"],
                "https://www.irs.gov/pub/irs-dft/i1040sb--dft.pdf",
                Message::new(explanation, alloc::vec![]),
            );
        }
    }
    if third_required && let Some(s) = screen {
        for (number, id, value) in [
            ("7a", "schedule_b_foreign_accounts", answer(Some(s.foreign_account))),
            ("7a(2)", "schedule_b_fbar_required", answer(s.fbar_required)),
            ("8", "schedule_b_foreign_trust", answer(Some(s.foreign_trust))),
        ] {
            crate::kernel::lines::form_value(
                r,
                crate::FormLine::new(crate::TaxForm::ScheduleB, number),
                id,
                crate::kernel::lines::LineValue::Text,
                &["screening.foreign_accounts"],
                "https://www.irs.gov/pub/irs-dft/i1040sb--dft.pdf",
                Message::new("EXPLAIN_PRINT_ANSWER", alloc::vec![value]),
            );
        }
        let message = Message::new(
            "NOTE_SCHEDULE_B_PART_III",
            alloc::vec![
                answer(Some(s.foreign_account)),
                answer(s.fbar_required),
                answer(Some(s.foreign_trust)),
            ],
        );
        r.review_notes.push(crate::federal_model::ReviewNote::scoped(
            #[cfg(test)]
            message.english(),
            message,
            crate::IssueContext::Section(crate::TaxSection::InterestDividends),
        ));
        for id in &s.fbar_countries {
            if let Some(key) = crate::foreign_accounts::country_key(*id) {
                crate::kernel::lines::form_value(
                    r,
                    crate::FormLine::subrow(crate::TaxForm::ScheduleB, "7b"),
                    &alloc::format!("schedule_b_country_{id}"),
                    crate::kernel::lines::LineValue::Text,
                    &["screening.fbar_countries"],
                    "https://www.irs.gov/pub/irs-dft/i1040sb--dft.pdf",
                    Message::new(
                        "EXPLAIN_PRINT_ANSWER",
                        alloc::vec![MessageArg::Resource(String::from(key))],
                    ),
                );
                let message = Message::new(
                    "NOTE_SCHEDULE_B_COUNTRY",
                    alloc::vec![MessageArg::Resource(String::from(key))],
                );
                r.review_notes.push(crate::federal_model::ReviewNote::scoped(
                    #[cfg(test)]
                    message.english(),
                    message,
                    crate::IssueContext::Section(crate::TaxSection::InterestDividends),
                ));
            }
        }
    }
}
