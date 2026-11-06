// Form 8949 capital-gain rows and return aggregation.
#[allow(unused_imports)]
use super::*;
use crate::FederalIncomeEntry;
use crate::capital::Form8949Row;
use crate::opportunity_fund::{Inclusion, QofDeferral};
use alloc::vec::Vec;

impl Inclusion {
    // The Form 8949 rows of the inclusion: code Y in Part I box C and Part II box F; a section
    // 1231 gain is the Y row and a code O row that backs it out of Schedule D (it is on Form 4797
    // Part I instead), netting to zero.
    pub(crate) fn rows(&self, entry: &FederalIncomeEntry) -> Vec<Form8949Row> {
        let mut rows = Vec::new();
        let mut push = |line: usize, box_letter: char, adjustment: i64, codes: &str| {
            rows.push(Form8949Row {
                entry: entry.id,
                row: rows.len() as u32 + 1,
                line,
                box_letter,
                label: crate::interview::income_display_arg(entry),
                proceeds: 0,
                basis: 0,
                adjustment,
                codes: codes.into(),
                summary: false,
            });
        };
        if self.short != 0 {
            push(3, 'C', self.short, "Y");
        }
        if self.long != 0 {
            push(7, 'F', self.long, "Y");
        }
        if self.section_1231 != 0 {
            push(7, 'F', self.section_1231, "Y");
            push(7, 'F', -self.section_1231, "O");
        }
        rows
    }
}

impl QofDeferral {
    // The Form 8949 rows of the election: code Z, the deferred gain as a negative column (g).
    pub(crate) fn rows(&self, entry: &FederalIncomeEntry) -> Vec<Form8949Row> {
        let mut rows = Vec::new();
        for (line, box_letter, amount) in [(3, 'C', self.short_term), (7, 'F', self.long_term)] {
            if amount != 0 {
                rows.push(Form8949Row {
                    entry: entry.id,
                    row: rows.len() as u32 + 1,
                    line,
                    box_letter,
                    label: crate::interview::income_display_arg(entry),
                    proceeds: 0,
                    basis: 0,
                    adjustment: -amount,
                    codes: "Z".into(),
                    summary: false,
                });
            }
        }
        rows
    }
}

// The personal-use part of an employee vehicle's sale is a Form 8949 row under
// a record number no income entry has.
#[inline(never)]
pub(super) fn employee_personal_parts(
    d: &FederalDraft,
    t: &mut Totals,
    r: &mut FederalResult,
    employee: &crate::employee_expenses::Calculation,
) {
    for (position, form) in employee.forms.iter().enumerate() {
        let assets = form.claim.all_assets();
        for index in 0..assets.len() {
            t.personal_part(
                r,
                crate::asset_sale::employee_source_id(position),
                crate::BusinessAsset::display_arg(&assets, index, form.claim.mileage.len()),
                assets[index].sale_for(crate::depreciation::mid_quarter(d), false),
            );
        }
    }
}
// The personal-use part of a sold rental: a gain is a capital gain on Form
// 8949 (box F, or C within a year of the first building), a loss is not
// deductible (Pub 544, "Property Used Partly for Business or Rental").
#[inline(never)]
pub(super) fn rental_personal_part(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    t: &mut Totals,
    r: &mut FederalResult,
    entry: &FederalIncomeEntry,
    data: &crate::FederalRental,
) {
    if crate::asset_sale::rental_sale_refusal(
        data,
        crate::nonmacrs::rental_recovery_source_with_macrs(
            form_context.draft,
            entry.id,
            true,
            Some(form_context.macrs().as_ref()),
        )
        .as_ref(),
    )
    .is_none()
        && let Ok(Some(sale)) = crate::asset_sale::rental_sale(data, false)
        && sale.personal > 0
        && let Some(date) = data.sale.date()
    {
        let first = data.buildings.iter().map(|b| b.in_service).min().unwrap_or(date);
        t.property_row(
            r,
            entry.id,
            crate::interview::income_display_arg(entry),
            crate::dates::held_more_than_one_year(first, date),
            [sale.personal_proceeds, sale.personal_basis, 0],
            true,
        );
    }
}

// A code Z deferral (IRC 1400Z-2(a)(1)(A)): the deferred gain is not gross income, so it leaves
// the Schedule D net and Form 1116 line 3e together, takes the entry's own community share when
// settled, and shows its code Z Form 8949 rows.
pub(super) fn qof_deferral(
    t: &mut Totals,
    r: &mut FederalResult,
    e: &FederalIncomeEntry,
    data: &crate::QofDeferral,
) {
    t.exclude_short(data.short_term);
    t.exclude_long(data.long_term);
    t.investment_gains[0] -= data.short_term;
    t.investment_gains[1] -= data.long_term;
    for row in data.rows(e) {
        row.emit(r);
        t.add_row(&row);
    }
    emit_lines(
        r,
        &mut (),
        [
            ("short", data.short_term, msg!("EXPLAIN_QOF_DEFERRAL_N_SHORT")),
            ("long", data.long_term, msg!("EXPLAIN_QOF_DEFERRAL_N_LONG")),
        ],
        |(suffix, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("qof_deferral_{}_{}", e.id, suffix),
                LineValue::MoneyCents(amount),
                &["income.opportunity_fund_deferral"],
                "https://www.irs.gov/instructions/i8949",
                explanation,
            ))
        },
    );
}

// A fund's section 1202 gain (Form 1099-DIV box 2c, Form 2439 box 1c): the full gain stays on its
// own line and the exclusion is a Form 8949 Part II box F row, code Q, with a negative column (g)
// (the Instructions for Schedule D, "Gain from Form 1099-DIV" and "Gain from Form 2439"). The
// 50 and 75 percent exclusions leave the rest in the 28 percent rate pool and carry the 7 percent
// AMT preference of Form 6251 line 2h (qsbs.rs).
pub(super) fn fund_section_1202(
    t: &mut Totals,
    r: &mut FederalResult,
    e: &FederalIncomeEntry,
    gain: i64,
    exclusion: Option<crate::FundQsbsExclusion>,
    share: Option<crate::community::Share>,
    separate: bool,
) {
    let fund = crate::investment_statements::allocated_fund_gain(gain, exclusion, share, separate);
    if fund.excluded == 0 {
        return;
    }
    // Section 1202(a): the excluded part is not gross income, so it leaves Form 1116 line 3e as
    // it leaves the net (the individual QSBS path adds only its taxable gain).
    t.exclude_long(fund.excluded);
    t.investment_gains[1] -= fund.excluded;
    t.qsbs_excluded += fund.excluded;
    t.collectible_long += fund.rate_28;
    t.qsbs_amt_preference += fund.amt_preference;
    t.row(r, e, 7, 'F', [0, 0, -fund.excluded], "Q");
}

pub(super) fn collect_sale(
    t: &mut Totals,
    v: &crate::FederalSale,
    e: &FederalIncomeEntry,
    r: &mut FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] _source: &str,
) {
    checked_collect_sale(
        t,
        v,
        e,
        r,
        context,
        #[cfg(test)]
        _source,
    )
    .expect("admitted native sale totals");
}

pub(super) fn collect_nua_sale(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::NuaSale,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    crate::require_localized!(
        r,
        data.confirmed(),
        context,
        source,
        "ISSUE_CONFIRM_THE_ELIGIBLE_NUA_DISTRIBUTION_AND_LOT_HISTORY_INHERITED_SHA_9B0E8900",
        "Confirm the eligible NUA distribution and lot history. Inherited shares require a reconciled estate valuation at least equal to deferred NUA and no allocable estate or generation-skipping tax; unresolved estate allocations require further calculations.",
    );
    if data.confirmed() {
        let nua = data.realized_nua();
        t.nua_exclusion_sources.push((e.id, Some(nua)));
        t.nua_excluded_gain += nua;
        for part in data.parts() {
            collect_sale(
                t,
                &part,
                e,
                r,
                context,
                #[cfg(test)]
                source,
            );
        }
        emit_lines(
            r,
            &mut (),
            [
                ("basis", data.basis()),
                ("realized_nua", nua),
                ("post_distribution_gain", data.proceeds - data.basis() - nua + data.wash_sale),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("nua_sale_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.nua_sale"],
                    if data.inheritance.is_some() {
                        "https://bradfordtaxinstitute.com/Endnotes/Rev_Rul_75-125.pdf"
                    } else {
                        "https://www.irs.gov/publications/p575"
                    },
                    msg!("EXPLAIN_NUA_SALE_N_N"),
                ))
            },
        );
    } else {
        t.nua_exclusion_sources.push((e.id, None));
    }
    Ok(())
}

pub(super) fn collect_employee_stock_sale(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::EmployeeStockSale,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    // An unconfirmed sale was left out before the rules ran.
    if !data.special {
        let sale = data.sale();
        emit_lines(
            r,
            &mut (),
            [
                ("compensation", data.compensation()),
                ("basis", sale.basis_amount()),
                ("capital_gain", sale.gain()),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("employee_stock_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.employee_stock_sale"],
                    "https://www.irs.gov/publications/p525",
                    msg!("EXPLAIN_EMPLOYEE_STOCK_N_N"),
                ))
            },
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
    Ok(())
}

pub(super) fn collect_espp_sale(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::EsppSale,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    // An unconfirmed sale was left out before the rules ran.
    if !data.special {
        let sale = data.sale();
        if !data.in_wages {
            t.iso_unreported[owner] += data.compensation();
        }
        emit_lines(
            r,
            &mut (),
            [
                ("compensation", data.compensation()),
                ("basis", sale.basis_amount()),
                ("capital_gain", sale.gain()),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("espp_sale_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.espp_sale"],
                    "https://www.irs.gov/publications/p525",
                    msg!("EXPLAIN_ESPP_SALE_N_N"),
                ))
            },
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
    Ok(())
}

pub(super) fn collect_iso_sale(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::IsoSale,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    if data.confirmed {
        let sale = data.sale();
        if !data.in_wages {
            t.iso_unreported[owner] += data.compensation();
        }
        t.iso_amt_wage_adjustment[owner] += data.amt_compensation_adjustment();
        emit_lines(
            r,
            &mut (),
            [
                ("compensation", data.compensation()),
                ("amt_compensation_adjustment", data.amt_compensation_adjustment()),
                ("regular_basis", sale.basis_amount()),
                ("amt_basis", sale.amt.different().unwrap().basis),
                ("capital_gain", sale.gain()),
                ("amt_capital_gain", sale.amt_gain()),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("iso_sale_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(amount),
                    &["income.iso_sale"],
                    "https://www.irs.gov/publications/p525",
                    msg!("EXPLAIN_ISO_SALE_N_N"),
                ))
            },
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
    Ok(())
}

pub(super) fn collect_sale_source(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::SaleSource,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    collect_sale(
        t,
        &crate::FederalSale::from(data),
        e,
        r,
        context,
        #[cfg(test)]
        source,
    );
    Ok(())
}

pub(super) fn collect_broker(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::BrokerDocument,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    let view = crate::broker_calculation::projection(data);
    if let Some(sale) = &view.sale
        && checked_collect_sale(
            t,
            sale,
            e,
            r,
            context,
            #[cfg(test)]
            source,
        )
        .is_none()
    {
        return Err(CollectFailure::Broker(context.clone()));
    }
    // Components carry no withholding. The document contributes
    // its actual amount once, including at an unresolved boundary.
    checked_native_add(&mut t.withholding_1099, crate::broker_calculation::source_withholding(e))
        .ok_or_else(|| CollectFailure::Broker(context.clone()))?;
    Ok(())
}

pub(super) fn collect_worthless_security(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::WorthlessSecurity,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    // An unconfirmed or special security was left out before
    // the rules ran. Form 8949 box C or F, "Worthless" in
    // columns (a)/(c).
    let basis = &data.basis;
    let term = data.acquisition.term();
    t.investment_gains[usize::from(term == GainTerm::Long)] -= basis;
    match term {
        GainTerm::Short => {
            t.add_short(-basis);
            t.row(r, e, 3, 'C', [0, *basis, 0], "");
        }
        GainTerm::Long => {
            t.add_long(-basis);
            t.row(r, e, 7, 'F', [0, *basis, 0], "");
        }
    }
    line_cents(
        r,
        &alloc::format!("worthless_security_{}_loss", e.id),
        -basis,
        &["income.worthless_security"],
        "https://www.irs.gov/publications/p550",
        if term == GainTerm::Long {
            msg!("EXPLAIN_WORTHLESS_SECURITY_N_LOSS")
        } else {
            msg!("EXPLAIN_WORTHLESS_SECURITY_N_LOSS_2")
        },
    );
    Ok(())
}

pub(super) fn collect_nonbusiness_bad_debt(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::NonbusinessBadDebt,
) -> Result<(), CollectFailure> {
    let Source { e, .. } = *src;
    let bad_debt_source = &data.source;
    let investment = &data.investment;
    // The debtor, worthlessness and source answers were settled
    // before the rules ran (unknowns.rs: a failing fact is no
    // loss, an unresolved one is assumed no loss with a note); a
    // personal claim is never an investment debt, which the
    // page's rules enforce.
    let loss = bad_debt_source.unrecovered_basis();
    t.add_short(-loss);
    // Form 8949 box C, "bad debt statement attached".
    t.row(r, e, 3, 'C', [0, loss, 0], "");
    if *investment {
        t.investment_gains[0] -= loss;
    }
    line_cents(
        r,
        &alloc::format!("bad_debt_{}_short_term_loss", e.id),
        -loss,
        &["income.nonbusiness_bad_debt"],
        "https://www.irs.gov/taxtopics/tc453",
        msg!("EXPLAIN_BAD_DEBT_N_SHORT_TERM_LOSS"),
    );
    Ok(())
}

pub(super) fn collect_opportunity_fund(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::QofInvestment,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    // A special gain code or treatment the App cannot compute was left out before
    // the rules ran (unknowns.rs). The remaining deferred gain is included on the
    // earlier of the sale and December 31, 2026 (IRC 1400Z-2(b)(1)(B)).
    let inclusion =
        data.inclusion().ok_or_else(|| CollectFailure::OpportunityFundEvent(context.clone()))?;
    if inclusion.value_assumed {
        note_once_scoped(
            r,
            Message::new(
                "WARN_QOF_VALUE_ASSUMED",
                alloc::vec![crate::interview::income_display_arg(e)],
            ),
            context.clone(),
        );
    }
    // The section 1231 part joins Form 4797 Part I instead (section1231::sources).
    t.add_short(inclusion.short);
    t.add_long(inclusion.long);
    t.investment_gains[0] += inclusion.short;
    t.investment_gains[1] += inclusion.long;
    t.collectible_long += inclusion.collectibles;
    for row in inclusion.rows(e) {
        row.emit(r);
        t.add_row(&row);
    }
    // The code Y and code O rows of the section 1231 part net to zero on Schedule D:
    // the Y row is not a gain a code Z deferral can come from.
    t.positive_rows[1] -= positive(inclusion.section_1231);
    for (suffix, amount, explanation) in [
        ("basis_increase", inclusion.increase, msg!("EXPLAIN_QOF_N_BASIS_INCREASE")),
        ("included_short", inclusion.short, msg!("EXPLAIN_QOF_N_INCLUDED_SHORT")),
        (
            "included_long",
            inclusion.long + inclusion.section_1231,
            msg!("EXPLAIN_QOF_N_INCLUDED_LONG"),
        ),
        ("included_property", inclusion.section_1231, msg!("EXPLAIN_QOF_N_INCLUDED_PROPERTY")),
        ("basis_after", inclusion.basis_after, msg!("EXPLAIN_QOF_N_BASIS_AFTER")),
    ] {
        // The section 1231 gain is a Form 4797 Part I line 2 source.
        push_line(
            r,
            (suffix == "included_property").then(|| FormLine::new(TaxForm::Form4797, "2")),
            &alloc::format!("qof_{}_{}", e.id, suffix),
            amount,
            &["income.opportunity_fund"],
            "https://www.irs.gov/instructions/i8949",
            explanation,
        );
    }
    Ok(())
}

pub(super) fn collect_opportunity_fund_deferral<'a>(
    cx: &mut Collect<'a>,
    src: &Source<'a, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::QofDeferral,
) -> Result<(), CollectFailure> {
    let Source { e, accepted_entry, .. } = *src;
    cx.deferrals.push(accepted_entry);
    qof_deferral(t, r, e, data);
    Ok(())
}
