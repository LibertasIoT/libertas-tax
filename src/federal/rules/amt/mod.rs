use super::*;

pub(crate) fn iso_adjustment(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut cents: i128 = 0;
    for (position, source) in d.screening.as_ref().unwrap().iso_exercises.iter().enumerate() {
        let Ok(v) = IsoExercise::try_from(source) else {
            crate::issue_localized!(
                r,
                &crate::IssueContext::owner(crate::TaxSection::InvestmentSales, source.owner)
                    .item("iso_exercises", position)
                    .named(&source.form.transferor.name),
                "Incentive stock options",
                "DOCUMENT_ANSWER_ISOEXERCISEFACTS",
                "Shares from this incentive stock option exercise",
            );
            continue;
        };
        let price = v.recognition_price();
        crate::require_localized!(
            r,
            v.valid_timing()
                && v.held_units().is_some()
                && (0..=100_000_000_000).contains(&v.option_cost)
                && (0..=100_000_000_000).contains(&v.exercise_price)
                && (0..=100_000_000_000).contains(&v.market_price)
                && (0..=100_000_000_000).contains(&price)
                && (v.owner != Owner::Spouse
                    || d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint),
            &crate::IssueContext::owner(crate::TaxSection::InvestmentSales, v.owner)
                .item("iso_exercises", position)
                .named(&v.label),
            "Incentive stock options",
            "ISSUE_REVIEW_THE_OWNER_SHARE_COUNT_EXERCISE_PRICES_AND_SPECIAL_STOCK_REST_EE383591",
            "Review the owner, share count, exercise prices and special stock restrictions.",
        );
        // Form 6251 line 2i: the shares still held at year-end, at their
        // value when they entered AMT income, less their exercise price and
        // the option cost allocated to them. Shares disposed of in 2026, the
        // exercise year, have no adjustment (i6251).
        if let Some(held) = v.held_units()
            && (0..=100_000_000_000).contains(&price)
            && (0..=100_000_000_000).contains(&v.exercise_price)
            && (0..=100_000_000_000).contains(&v.option_cost)
        {
            let transferred = crate::employee_stock::share_units(v.shares).unwrap_or(1);
            let spread =
                crate::kernel::ratio::fraction(positive(price - v.exercise_price), held, 1_000_000);
            // The option cost covers every share of the exercise; the sold
            // shares take their part of it.
            let option_cost = crate::kernel::ratio::fraction(v.option_cost, held, transferred);
            cents += i128::from(phase::floor(spread.saturating_sub(option_cost), 0));
        }
    }
    crate::require_localized!(
        r,
        cents <= 100_000_000_000,
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
        "Incentive stock options",
        "ISSUE_THE_TOTAL_EXERCISE_SPREAD_EXCEEDS_THIS_PROTOTYPE_S_SUPPORTED_AMOUNT_173AE371",
        "The total exercise spread exceeds this prototype's supported amount range.",
    );
    dollars(min(cents, 100_000_000_000) as i64)
}

#[cfg(test)]
pub(crate) fn amt_tax(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
) -> i64 {
    amt_tax_with_collectibles(base, preferential, regular_ordinary, status, excluded, 0)
}
#[cfg(test)]
pub(crate) fn amt_tax_with_collectibles(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    collectibles: i64,
) -> i64 {
    amt_tax_with_special_gains(
        base,
        preferential,
        regular_ordinary,
        status,
        excluded,
        [collectibles, 0],
    )
}
#[cfg(test)]
pub(crate) fn amt_tax_with_special_gains(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    special: [i64; 2],
) -> i64 {
    amt_tax_with_foreign_gains(
        base,
        preferential,
        regular_ordinary,
        status,
        excluded,
        special,
        None,
    )
}
#[cfg(test)]
pub(crate) fn amt_tax_with_foreign_gains(
    base: i64,
    preferential: i64,
    regular_ordinary: i64,
    status: FederalStatus,
    excluded: i64,
    special: [i64; 2],
    foreign: Option<crate::income_tax::Preference>,
) -> i64 {
    crate::income_tax::AmtTaxWorksheet {
        year: 2026,
        status,
        base,
        preferential,
        regular_ordinary,
        excluded,
        special,
        foreign,
    }
    .tax()
}

pub(crate) fn record_amt(
    r: &mut FederalResult,
    base: i64,
    amt: i64,
    status: FederalStatus,
    tentative: i64,
    foreign_credit: i64,
    regular_offset: i64,
) -> bool {
    let [income, _, _, _, _, exemption, taxable] =
        crate::income_tax::AmtRates::for_year(2026, status).unwrap().exemption_lines(base);
    // The MFS income addition can exceed the cent carrier even when the
    // source income fits. Check the actual projected export before any rows.
    let Some((income_cents, amt_cents)) = income.checked_mul(100).zip(amt.checked_mul(100)) else {
        return false;
    };
    let required = amt > 0;
    // A positive line11 establishes a Form6251 filing condition. Other filing
    // conditions and PartI/III projection are separate; never
    // substitute positive AMTI for a filing requirement. These rows reuse the
    // actual exemption and comparison owners, including MFS income additions,
    // foreign-credit subtraction and unaveraged ScheduleJ regular tax.
    crate::line_emission::push_line(
        r,
        required.then(|| FormLine::new(TaxForm::Form6251, "4")),
        "alternative_minimum_taxable_income",
        income_cents,
        &["agi", "deductions", "income", "screening.iso_exercises"],
        AMT,
        msg!("EXPLAIN_ALTERNATIVE_MINIMUM_TAXABLE_INCOME"),
    );
    crate::line_emission::push_line(
        r,
        required.then(|| FormLine::new(TaxForm::Form6251, "11")),
        "alternative_minimum_tax",
        amt_cents,
        &["alternative_minimum_taxable_income", "regular_income_tax", "foreign_tax_credit"],
        AMT,
        msg!("EXPLAIN_ALTERNATIVE_MINIMUM_TAX"),
    );
    if required {
        emit_lines(
            r,
            &mut (),
            [
                ("5", "amt_exemption", exemption),
                ("6", "amt_taxable_base", taxable),
                ("7", "amt_tax_before_foreign_credit", tentative),
                ("8", "amt_foreign_credit", foreign_credit),
                ("9", "amt_tentative_minimum_tax", positive(tentative - foreign_credit)),
                ("10", "amt_regular_tax_offset", regular_offset),
            ],
            |(number, id, value), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form6251, number)),
                    id,
                    LineValue::MoneyDollars(value),
                    &[
                        "alternative_minimum_taxable_income",
                        "regular_income_tax",
                        "foreign_tax_credit",
                    ],
                    AMT,
                    msg!("EXPLAIN_AMT_COMPARISON"),
                ))
            },
        );
    }
    true
}

pub(crate) fn valid_exercises(values: &[crate::IsoExerciseSource]) -> bool {
    valid_exercise_sources(values)
        && values.iter().all(|source| {
            IsoExercise::try_from(source)
                .is_ok_and(|value| value.valid_timing() && value.held_units().is_some())
        })
}

pub(crate) fn form3921_paper_issue(
    form: &crate::Form3921Paper,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if form.granted > form.exercised {
        Some(ValidationIssue::new("ISO_EXERCISED_BEFORE_GRANTED").at_field("exercised"))
    } else if !form.transferor.name.trim().is_empty()
        && (form.corporation.name.is_empty() || !form.corporation.name.trim().is_empty())
        && crate::federal_rules::valid_birth(form.granted)
        && crate::federal_rules::valid_birth(form.exercised)
        && crate::employee_stock::share_units(form.shares).is_some()
        && (0..=100_000_000_000).contains(&form.exercise_price)
        && (0..=100_000_000_000).contains(&form.market_price)
    {
        None
    } else {
        Some(ValidationIssue::new(
            "MESSAGE_FORM_3921_CHECK_THE_SHARE_COUNTS_UP_TO_SIX_DECIMAL_PLACES_AND_THE_NAMES",
        ))
    }
}

// Corrections can leave accepted topic answers inconsistent with the new paper.
// Their recognition timing and sold-share relation are resolved on the topic.
pub(crate) fn valid_exercise_sources(values: &[crate::IsoExerciseSource]) -> bool {
    values.iter().all(|source| {
        form3921_paper_issue(&source.form).is_none()
            && IsoExercise::try_from(source).ok().is_none_or(|v| {
                (v.sold == 0.0 || crate::employee_stock::share_units(v.sold).is_some())
                    && match v.recognition {
                        crate::IsoRecognition::Vesting { date, .. } => {
                            crate::federal_rules::valid_birth(date)
                        }
                        _ => true,
                    }
                    && (0..=100_000_000_000).contains(&v.option_cost)
                    && (0..=100_000_000_000).contains(&v.recognition_price())
            })
    })
}
