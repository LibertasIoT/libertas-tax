// home_sale items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_home_sale(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::MainHomeSale,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    match &data.answers {
        Some(crate::RealEstateSaleUse::Business) => {
            // Counted on the property's own page (Form 4797), whose sale
            // price must be this form's box 2a.
            let price = data.form().proceeds;
            crate::federal_rules::require_message(
                r,
                crate::debt::business_dispositions(d).contains(&price),
                &crate::IssueContext::income_id(e.id)
                    .alternatives(&crate::debt::business_sale_pages(d, e.id)),
                #[cfg(test)]
                source,
                #[cfg(test)]
                alloc::format!(
                    "Form 1099-S: enter this business, farm or rental property's 2026 sale on its own page with ${}.{:02} (box 2a) as the sale price.",
                    price / 100,
                    price % 100
                ),
                Message::new(
                    "ISSUE_REAL_ESTATE_SALE_PRICE",
                    alloc::vec![crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 2,
                        minor_units: price,
                    }],
                ),
            );
            return Ok(());
        }
        Some(crate::RealEstateSaleUse::Other(other)) => {
            let form = data.form();
            // Property or services received (box 6) are part of the amount
            // realized, and a foreign seller (box 7) has withholding this
            // sale does not figure.
            crate::require_localized!(
                r,
                !form.property_received && !form.foreign_person,
                &context,
                &source,
                "ISSUE_REAL_ESTATE_SALE_BOX_6_OR_7",
                "Form 1099-S box 6 or box 7 is checked: the property or services received, or the foreign seller's withholding, are not figured for this sale, so it is left out of this estimate.",
            );
            if !form.property_received && !form.foreign_person {
                let sale = other.sale(form.proceeds);
                line_cents(
                    r,
                    &alloc::format!("real_estate_{}_gain", e.id),
                    sale.gain(),
                    &["income.home_sale"],
                    "https://www.irs.gov/instructions/i8949",
                    msg!("EXPLAIN_REAL_ESTATE_N_GAIN"),
                );
                collect_sale(
                    t,
                    &sale,
                    e,
                    r,
                    context,
                    #[cfg(test)]
                    source,
                );
            }
            return Ok(());
        }
        Some(crate::RealEstateSaleUse::MainHome(_)) | None => {}
    }
    let data = &crate::HomeSale::from(data);
    let status = d.setup.as_ref().unwrap().filing_status();
    let excluded = match data.excluded(status) {
        Ok(value) => value,
        Err((key, _message)) => {
            crate::issue_localized!(r, &context, &source, key, _message,);
            0
        }
    };
    if excluded > 0 {
        cx.excluded_homes += 1;
    }
    crate::require_localized!(
        r,
        cx.excluded_homes <= 1,
        &context,
        &source,
        "ISSUE_MULTIPLE_EXCLUDED_HOME_SALES_IN_ONE_YEAR_REQUIRE_COORDINATED_SPOUSE_287D0D61",
        "Multiple excluded home sales in one year require coordinated spouse and prior-sale allocation.",
    );
    let (price, expenses, basis) = data.home_amounts();
    // An installment sale reports the year's part of the gain on
    // Form 6252 (Pub. 523, Installment Sale): the exclusion comes off
    // the gross profit, and unrecaptured section 1250 gain is
    // recognized first (Schedule D instructions).
    let installment = data.installment(excluded);
    // An exchange for like-kind property recognizes only the cash and
    // other property received beyond the exclusion (Form 8824 line 20,
    // Rev. Proc. 2005-14): a rental's gain is section 1231 (Form 4797
    // line 5, with the netting), an investment's a capital gain.
    let exchange = data.exchange_form(excluded);
    let rental_exchange = data.exchange_1231(excluded).is_some();
    let gain = match (&installment, exchange) {
        (Some((form, _)), _) => form.income * 100,
        (None, Some(form)) => dollars(form.line20) * 100,
        (None, None) => data.gain() - excluded,
    };
    if !rental_exchange {
        t.unrecaptured_long += if installment.is_some() || exchange.is_some() {
            crate::kernel::phase::cap(gain, [data.depreciation_gain()])
        } else {
            data.depreciation_gain()
        };
        match data.term {
            GainTerm::Short => t.add_short(gain),
            GainTerm::Long => t.add_long(gain),
        }
    }
    let firpta = data.firpta_credit();
    if firpta > 0 {
        // Tax withheld on the foreign seller's price (Form 8288-A):
        // Form 1040 line 25c.
        t.withholding_other += firpta;
        line_cents(
            r,
            &alloc::format!("home_{}_firpta_withholding", e.id),
            firpta,
            &["income.home_sale"],
            "https://www.irs.gov/instructions/i1040gi",
            msg!("EXPLAIN_HOME_N_N_FIRPTA_WITHHOLDING"),
        );
    }
    if data.like_kind_denied() {
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "{}: the home was got in a like-kind exchange less than 5 years before the sale, so none of the gain is excluded (section 121(d)(10)).",
            e.label
        ), Message::new("NOTE_HOME_SALE_LIKE_KIND_NO_EXCLUSION", alloc::vec![crate::interview::income_display_arg(e)])));
    }
    // Form 8949 when a 1099-S was received or some gain is
    // taxable (Schedule D instructions, Sale of Your Home): box C
    // or F, columns (d) and (e) as sold and bought, column (g)
    // the selling expenses (code E) with the exclusion (code H)
    // or the nondeductible loss (code L), column (h) the taxable
    // gain on Schedule D line 3 or 10.
    if let Some(form) = exchange {
        if !rental_exchange {
            t.schedule_d_line_11 += gain;
        }
        for (line, amount, explanation) in [
            ("15", form.line15, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE15")),
            ("16", form.line16, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE16")),
            ("18", form.line18, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE18")),
            ("19", form.line19, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE19")),
            ("20", form.line20, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE20")),
            ("24", form.line24, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE24")),
            ("25", form.line25, msg!("EXPLAIN_HOME_N_N_EXCHANGE_LINE25")),
        ] {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form8824, line)),
                &alloc::format!("home_{}_form8824_line{line}", e.id),
                dollars(amount) * 100,
                &["income.home_sale"],
                "https://www.irs.gov/instructions/i8824",
                explanation,
            );
        }
        // A rental's recognized gain is section 1231 gain: Form 8824
        // carries it to Form 4797 line 5 (Form 8824 instructions,
        // line 22), as a business exchange does.
        if rental_exchange && form.line20 > 0 {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4797, "5")),
                &alloc::format!("home_{}_exchange_section1231", e.id),
                dollars(form.line20) * 100,
                &["income.home_sale"],
                "https://www.irs.gov/instructions/i8824",
                msg!("EXPLAIN_EXCHANGE_SECTION1231"),
            );
        }
    } else if let Some((form, line15)) = &installment {
        match data.term {
            GainTerm::Short => t.schedule_d_line_4 += gain,
            GainTerm::Long => t.schedule_d_line_11 += gain,
        }
        let prefix = alloc::format!("home_{}_form6252", e.id);
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form6252, "15")),
            &alloc::format!("{prefix}_exclusion"),
            line15 * 100,
            &["income.home_sale"],
            "https://www.irs.gov/pub/irs-dft/f6252--dft.pdf",
            msg!("EXPLAIN_HOME_N_N_INSTALLMENT_EXCLUSION"),
        );
        crate::installment_return::report_sale(r, &prefix, "income.home_sale", form, 0, false);
    } else if data.reported(gain) {
        let (line, box_letter) = match data.term {
            GainTerm::Short => (3, 'C'),
            GainTerm::Long => (7, 'F'),
        };
        // The home's own lines below are its row; the Schedule D columns total it.
        t.add_row(&crate::capital::Form8949Row {
            entry: e.id,
            row: 0,
            line,
            box_letter,
            label: crate::interview::income_display_arg(e),
            proceeds: price,
            basis,
            adjustment: gain - (price - basis),
            codes: String::new(),
            summary: false,
        });
        let realized = price - expenses - basis;
        let codes = match (expenses > 0, realized < 0, excluded > 0) {
            (true, true, _) => "EL",
            (false, true, _) => "L",
            (true, false, true) => "EH",
            (false, false, true) => "H",
            (true, false, false) => "E",
            (false, false, false) => "",
        };
        emit_lines(
            r,
            &mut (),
            [
                ("form_8949_proceeds", price),
                ("form_8949_basis", basis),
                ("form_8949_adjustment", gain - (price - basis)),
                ("form_8949_gain", gain),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("home_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.home_sale"],
                    "https://www.irs.gov/instructions/i8949",
                    Message::new(
                        match (data.term, suffix, codes.is_empty()) {
                            (GainTerm::Short, "form_8949_proceeds", true) => {
                                "EXPLAIN_HOME_SHORT_PROCEEDS"
                            }
                            (GainTerm::Short, "form_8949_proceeds", false) => {
                                "EXPLAIN_HOME_SHORT_PROCEEDS_WITH_CODES"
                            }
                            (GainTerm::Short, "form_8949_basis", true) => {
                                "EXPLAIN_HOME_SHORT_BASIS"
                            }
                            (GainTerm::Short, "form_8949_basis", false) => {
                                "EXPLAIN_HOME_SHORT_BASIS_WITH_CODES"
                            }
                            (GainTerm::Short, "form_8949_adjustment", true) => {
                                "EXPLAIN_HOME_SHORT_ADJUSTMENT"
                            }
                            (GainTerm::Short, "form_8949_adjustment", false) => {
                                "EXPLAIN_HOME_SHORT_ADJUSTMENT_WITH_CODES"
                            }
                            (GainTerm::Short, "form_8949_gain", true) => "EXPLAIN_HOME_SHORT_GAIN",
                            (GainTerm::Short, "form_8949_gain", false) => {
                                "EXPLAIN_HOME_SHORT_GAIN_WITH_CODES"
                            }
                            (GainTerm::Long, "form_8949_proceeds", true) => {
                                "EXPLAIN_HOME_LONG_PROCEEDS"
                            }
                            (GainTerm::Long, "form_8949_proceeds", false) => {
                                "EXPLAIN_HOME_LONG_PROCEEDS_WITH_CODES"
                            }
                            (GainTerm::Long, "form_8949_basis", true) => "EXPLAIN_HOME_LONG_BASIS",
                            (GainTerm::Long, "form_8949_basis", false) => {
                                "EXPLAIN_HOME_LONG_BASIS_WITH_CODES"
                            }
                            (GainTerm::Long, "form_8949_adjustment", true) => {
                                "EXPLAIN_HOME_LONG_ADJUSTMENT"
                            }
                            (GainTerm::Long, "form_8949_adjustment", false) => {
                                "EXPLAIN_HOME_LONG_ADJUSTMENT_WITH_CODES"
                            }
                            (GainTerm::Long, "form_8949_gain", true) => "EXPLAIN_HOME_LONG_GAIN",
                            (GainTerm::Long, "form_8949_gain", false) => {
                                "EXPLAIN_HOME_LONG_GAIN_WITH_CODES"
                            }
                            _ => unreachable!("known home sale column"),
                        },
                        if codes.is_empty() {
                            alloc::vec![]
                        } else {
                            alloc::vec![MessageArg::Text(codes.into())]
                        },
                    ),
                ))
            },
        );
    } else {
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "{}: the whole gain is excluded and no Form 1099-S was received, so the sale is not reported on Form 8949 or Schedule D.",
            e.label
        ), Message::new("NOTE_HOME_SALE_GAIN_EXCLUDED", alloc::vec![crate::interview::income_display_arg(e)])));
    }
    emit_lines(
        r,
        &mut (),
        [
            (
                "depreciation_gain",
                data.depreciation_gain(),
                msg!("EXPLAIN_HOME_N_N_DEPRECIATION_GAIN"),
            ),
            (
                "nonqualified_gain",
                data.nonqualified_gain(),
                msg!("EXPLAIN_HOME_N_N_NONQUALIFIED_GAIN"),
            ),
            ("realized_gain", price - expenses - basis, msg!("EXPLAIN_HOME_N_N_REALIZED_GAIN")),
            ("excluded_gain", excluded, msg!("EXPLAIN_HOME_N_N_EXCLUDED_GAIN")),
            ("taxable_gain", gain, msg!("EXPLAIN_HOME_N_N_TAXABLE_GAIN")),
        ],
        |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("home_{}_{}", e.id, suffix),
                LineValue::MoneyCents(amount),
                &["income.home_sale"],
                "https://www.irs.gov/publications/p523",
                explanation,
            ))
        },
    );
    // The separate business or rental space: Form 4797 line 2
    // (Pub. 523, "Business Use or Rental of Home"); the gain
    // joins the section 1231 netting (home_sale::separate_sources).
    if let (Some(sale), Some((separate_gain, separate_unrecaptured))) =
        (data.separate_sale(), data.separate_form())
    {
        emit_lines(
            r,
            &mut (),
            [
                (None, "separate_price", sale.price, msg!("EXPLAIN_HOME_N_N_SEPARATE_PRICE")),
                (
                    None,
                    "separate_expenses",
                    sale.expenses,
                    msg!("EXPLAIN_HOME_N_N_SEPARATE_EXPENSES"),
                ),
                (None, "separate_basis", sale.basis, msg!("EXPLAIN_HOME_N_N_SEPARATE_BASIS")),
                (
                    Some("2"),
                    "separate_gain",
                    separate_gain * 100,
                    msg!("EXPLAIN_HOME_N_N_SEPARATE_GAIN"),
                ),
                (
                    None,
                    "separate_unrecaptured_1250",
                    separate_unrecaptured * 100,
                    msg!("EXPLAIN_HOME_N_N_SEPARATE_UNRECAPTURED"),
                ),
            ],
            |(line, suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    line.map(|line| FormLine::new(TaxForm::Form4797, line)),
                    &alloc::format!("home_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.home_sale"],
                    "https://www.irs.gov/publications/p523",
                    explanation,
                ))
            },
        );
    }
    Ok(())
}
