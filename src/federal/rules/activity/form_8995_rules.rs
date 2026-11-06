// form_8995 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

type PtpLossSource = (
    i64,
    crate::DocumentOwner,
    crate::kernel::ratio::exact::Exact,
    crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    i64,
);

pub(super) struct QbiBasisHalfSeIn {
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) senior_ded: i64,
    pub(super) additional: [i64; 3],
    pub(super) half: [i64; 2],
    pub(super) church_half: [i64; 2],
    pub(super) rental_inputs_complete: bool,
}

pub(super) struct QbiBasisHalfSeOut {
    pub(super) before_qbi_signed: i64,
    pub(super) before_qbi: i64,
    pub(super) partner_qbi_complete: bool,
    pub(super) qbi_gross_unknown: bool,
    pub(super) qbi_half_se: crate::partnership_qbi::QbiHalfSeAllocation,
}

pub(super) fn qbi_basis_half_se_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: QbiBasisHalfSeIn,
) -> QbiBasisHalfSeOut {
    let QbiBasisHalfSeIn {
        agi,
        deduction,
        senior_ded,
        additional,
        half,
        church_half,
        rental_inputs_complete,
    } = input;
    let d = pass.d;
    // Taxable income before the QBI deduction; the signed value feeds the section 68
    // reduction and the final taxable income (Itemized Deductions Worksheet), which are not
    // floored at zero before the reduction is added back.
    let before_qbi_signed = agi - deduction - senior_ded - additional.iter().sum::<i64>();
    let before_qbi = positive(before_qbi_signed);
    let mut partner_qbi_complete = rental_inputs_complete;
    let mut qbi_gross_unknown = false;
    let qbi_half = [half[0] - church_half[0], half[1] - church_half[1]];
    let qbi_half_se = match crate::partnership_qbi::half_se_by_gross_with_claims(
        d,
        qbi_half,
        pass.context.source_claims,
    ) {
        Ok(value) => value,
        Err(crate::partnership_qbi::QbiGrossError::Missing(contexts)) => {
            qbi_gross_unknown = true;
            for context in contexts {
                let display = MessageArg::Nested(alloc::boxed::Box::new(context.label().clone()));
                note_once_scoped(
                    r,
                    Message::new("WARN_QBI_GROSS_ALLOCATION_LEFT_OUT", alloc::vec![display]),
                    context,
                );
            }
            crate::partnership_qbi::QbiHalfSeAllocation::default()
        }
        Err(error) => {
            partner_qbi_complete = false;
            let (context, key, _message) = match error {
                crate::partnership_qbi::QbiGrossError::Inconsistent(context) => (
                    context,
                    "ISSUE_PARTNERSHIP_QBI_QUALIFIED_PARTNERSHIP_GROSS_SE_INCOME_MUST_AB4370A9",
                    crate::fixture_english!(
                        "Qualified partnership gross SE income must fit total gross SE income."
                    ),
                ),
                crate::partnership_qbi::QbiGrossError::UnavailableWeight(context) => (
                    context,
                    "ISSUE_PARTNERSHIP_QBI_ESTABLISH_POSITIVE_GROSS_INCOME_ALLOCATION_WEIGHTS_FOR_010B86AC",
                    crate::fixture_english!(
                        "Establish positive gross-income allocation weights for the owner's half-SE deduction."
                    ),
                ),
                crate::partnership_qbi::QbiGrossError::Missing(_) => unreachable!(),
            };
            crate::issue_localized!(r, &context, "QBI deduction allocation", key, _message,);
            crate::partnership_qbi::QbiHalfSeAllocation::default()
        }
    };
    QbiBasisHalfSeOut {
        before_qbi_signed,
        before_qbi,
        partner_qbi_complete,
        qbi_gross_unknown,
        qbi_half_se,
    }
}

// The earlier-year installment gains a section 1231 lookback makes ordinary,
// as qualified business income by the business the person named.

#[inline(never)]
pub(super) fn installment_qbi_shares_with_claims(
    d: &FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<crate::installment_return::QbiShares, crate::section1231::SourceError> {
    crate::installment_return::qbi_shares_with_claims(d, rental, character, source_claims)
}
// A business no longer on the return is a stand-alone QBI activity with no W-2
// wages and no qualified property.
#[inline(never)]
pub(super) fn standalone_installment_qbi(
    d: &FederalDraft,
    shares: &crate::installment_return::QbiShares,
    activities: &mut Vec<crate::business_rules::QbiActivity>,
) {
    for (_, part, owner) in shares.iter().filter(|(business, _, _)| {
        business.is_none_or(|id| {
            !d.income
                .iter()
                .any(|e| e.id == id && matches!(e.income, FederalIncome::Business { .. }))
        })
    }) {
        activities.push(crate::business_rules::QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: Some(*owner),
            income: *part,
            wages: 0,
            property: 0,
            active: true,
            service: Some(false),
            group: None,
        });
    }
}
// The QBI form's lines (Form 8995, or Form 8995-A above the threshold): one
// row per activity (line 1 on Form 8995; lines 2, 3 and 15 per column on
// Form 8995-A, omitted when aggregation merged activities), then the totals
// through the income limit and the $400 minimum. The deduction itself
// (8995 line 17, 8995-A line 41) is the tagged qbi_deduction line.
pub(super) fn qbi_lines(
    r: &mut FederalResult,
    form: TaxForm,
    lines: &crate::business_rules::QbiLines,
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
) {
    crate::business_rules::projection::emit(r, form, lines);
    let a = form == TaxForm::Form8995A;
    let source = if a {
        "https://www.irs.gov/instructions/i8995a"
    } else {
        "https://www.irs.gov/instructions/i8995"
    };
    let inputs = ["income.business", "self_employment_tax", "agi", "deduction"];
    if !lines.grouped {
        for (index, [income, netted, twenty, component]) in lines.activities.iter().enumerate() {
            // A released earlier-year loss is part of line 3, not a row.
            if lines.released.get(index).copied().unwrap_or(false) {
                continue;
            }
            let name = match business_ids.get(&index) {
                Some(id) => alloc::format!("business_{id}_qbi"),
                None => alloc::format!("qbi_activity_{}", index + 1),
            };
            // A patron's columns also print line 13 and the Schedule D patron
            // reduction of line 14 (zero: the farm has no W-2 wages).
            let patron_rows: &[(&str, &str, i64, &str)] = &[
                ("13", "_limited", lines.pre_patron[index], "EXPLAIN_QBI_ACTIVITY_LIMITED"),
                (
                    "14",
                    "_patron_reduction",
                    lines.patron_reductions[index],
                    "EXPLAIN_QBI_ACTIVITY_PATRON_REDUCTION",
                ),
            ];
            let rows: Vec<(&str, &str, i64, &str)> = if a {
                let mut rows = alloc::vec![
                    ("2", "", *netted, "EXPLAIN_QBI_ACTIVITY"),
                    ("3", "_twenty_percent", *twenty, "EXPLAIN_QBI_ACTIVITY_TWENTY_PERCENT",),
                ];
                if lines.patron {
                    rows.extend_from_slice(patron_rows);
                }
                rows.push(("15", "_component", *component, "EXPLAIN_QBI_ACTIVITY_COMPONENT"));
                rows
            } else {
                alloc::vec![("1", "", *income, "EXPLAIN_QBI_ACTIVITY_2")]
            };
            emit_lines(r, &mut (), rows, |(line, suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(form, line)),
                    &alloc::format!("{name}{suffix}"),
                    LineValue::MoneyDollars(amount),
                    &inputs,
                    source,
                    crate::Message::new(explanation, alloc::vec![]),
                ))
            });
        }
    }
    if a && lines.patron {
        for (index, [income, wages]) in lines.patron_allocations.iter().enumerate() {
            if *income == 0 && *wages == 0 {
                continue;
            }
            emit_lines(
                r,
                &mut (),
                [
                    ("2", "income", *income),
                    ("3", "nine_percent", fraction(positive(*income), 9, 100)),
                    ("4", "wages", *wages),
                    ("5", "half_wages", fraction(positive(*wages), 1, 2)),
                ],
                |(line, suffix, amount), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(form, &alloc::format!("Schedule D {line}"))),
                        &alloc::format!("qbi_activity_{}_patron_{suffix}", index + 1),
                        LineValue::MoneyDollars(amount),
                        &inputs,
                        source,
                        msg!("EXPLAIN_QBI_PATRON_REDUCTION"),
                    ))
                },
            );
        }
    }
    if a && lines.patron {
        form_line(
            r,
            FormLine::new(form, "Schedule D 6"),
            "qbi_patron_reduction",
            lines.patron_reductions.iter().sum(),
            &inputs,
            source,
            msg!("EXPLAIN_QBI_PATRON_REDUCTION"),
        );
    }
    let rows: [(&str, &str, &str, i64, &str); 14] = [
        ("2", "", "qbi_total", lines.total_income, "EXPLAIN_QBI_TOTAL"),
        ("3", "", "qbi_prior_loss", lines.prior_loss, "EXPLAIN_QBI_PRIOR_LOSS"),
        ("4", "", "qbi_net", lines.net_income, "EXPLAIN_QBI_NET"),
        ("5", "27", "qbi_component", lines.component, "EXPLAIN_QBI_COMPONENT"),
        ("6", "28", "qbi_reit_ptp", lines.reit, "EXPLAIN_QBI_REIT_PTP"),
        (
            "7",
            "29",
            "qbi_prior_reit_ptp_loss",
            lines.prior_reit_loss,
            "EXPLAIN_QBI_PRIOR_REIT_PTP_LOSS",
        ),
        ("8", "30", "qbi_reit_ptp_net", lines.reit_net, "EXPLAIN_QBI_REIT_PTP_NET"),
        (
            "9",
            "31",
            "qbi_reit_ptp_component",
            lines.reit_component,
            "EXPLAIN_QBI_REIT_PTP_COMPONENT",
        ),
        (
            "10",
            "32",
            "qbi_before_income_limit",
            lines.before_limit,
            "EXPLAIN_QBI_BEFORE_INCOME_LIMIT",
        ),
        ("11", "33", "qbi_taxable_income", lines.taxable, "EXPLAIN_QBI_TAXABLE_INCOME"),
        ("12", "34", "qbi_net_capital_gain", lines.gains, "EXPLAIN_QBI_NET_CAPITAL_GAIN"),
        ("13", "35", "qbi_income_less_gain", lines.limit_base, "EXPLAIN_QBI_INCOME_LESS_GAIN"),
        ("14", "36", "qbi_income_limit", lines.income_limit, "EXPLAIN_QBI_INCOME_LIMIT"),
        ("15", "37", "qbi_limited_deduction", lines.limited, "EXPLAIN_QBI_LIMITED_DEDUCTION"),
    ];
    for (line_8995, line_8995a, id, amount, explanation) in rows {
        let line = if a { line_8995a } else { line_8995 };
        if line.is_empty() {
            continue;
        }
        form_line(
            r,
            FormLine::new(form, line),
            id,
            amount,
            &inputs,
            source,
            crate::Message::new(explanation, alloc::vec![]),
        );
    }
    if a {
        form_line(
            r,
            FormLine::new(form, "38"),
            "qbi_cooperative_dpad",
            lines.dpad,
            &inputs,
            source,
            msg!("EXPLAIN_QBI_COOPERATIVE_DPAD"),
        );
        form_line(
            r,
            FormLine::new(form, "39"),
            "qbi_after_cooperative",
            lines.limited + lines.dpad,
            &inputs,
            source,
            msg!("EXPLAIN_QBI_AFTER_COOPERATIVE"),
        );
    }
    form_line(
        r,
        FormLine::new(form, if a { "40" } else { "16" }),
        "qbi_active_minimum",
        lines.minimum,
        &inputs,
        source,
        msg!("EXPLAIN_QBI_ACTIVE_MINIMUM"),
    );
}

pub(super) struct DirectQbiActivitiesIn<'a> {
    pub(super) rental: &'a mut crate::extended_rules::RentalResult,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) business_exclusions: &'a [crate::foreign_business::BusinessExclusion; 2],
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) tips_deducted: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) qbi_half_se: &'a crate::partnership_qbi::QbiHalfSeAllocation,
    pub(super) before_qbi: i64,
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) research_business_income: alloc::collections::BTreeMap<i64, i64>,
}

pub(super) struct DirectQbiActivitiesOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) research_business_income: alloc::collections::BTreeMap<i64, i64>,
    pub(super) farm_qbi_indices: Vec<usize>,
    pub(super) business_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) foreign_business_losses: i64,
    pub(super) foreign_business_profits: i64,
}

pub(super) fn direct_qbi_activities_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    input: DirectQbiActivitiesIn<'_>,
) -> core::ops::ControlFlow<(), DirectQbiActivitiesOut> {
    let &Pass { d, s, allocated_entries, office_context, selected_regular, context, .. } = pass;
    let source_claims = context.source_claims;
    let direct_participation = context.participation;
    let DirectQbiActivitiesIn {
        rental,
        property_character,
        business_exclusions,
        business_half_se,
        health,
        sep,
        tips_deducted,
        qbi_half_se,
        before_qbi,
        mut activities,
        mut research_business_income,
    } = input;
    // The ordinary part of earlier-year installment gain a section 1231 lookback
    // makes ordinary is qualified business income of the business that sold
    // the property, or of a stand-alone activity with no wages or property
    // when that business is no longer on the return (Reg. 1.199A-3(b)(2)(ii)(A)).
    let installment_qbi =
        match installment_qbi_shares_with_claims(d, rental, property_character, source_claims) {
            Ok(value) => value,
            Err(error) => {
                error.report_at(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                    #[cfg(test)]
                    "Earlier-year installment QBI",
                );
                if error.is_required_source() {
                    return core::ops::ControlFlow::Break(());
                }
                Vec::new()
            }
        };
    let mut farm_qbi_indices = Vec::new();
    // Form 8995 line 1 names a Schedule C activity by its business.
    let mut business_qbi_ids = alloc::collections::BTreeMap::new();
    let mut foreign_business_losses = 0;
    let mut foreign_business_profits = 0;
    for (owner, business_exclusion) in business_exclusions.iter().enumerate() {
        let mut entries: Vec<_> = allocated_entries
            .iter()
            .filter_map(|e| match &e.income {
                FederalIncome::Business { data: v }
                    if usize::from(e.owner == crate::DocumentOwner::Spouses) == owner =>
                {
                    Some((e.id, v))
                }
                _ => None,
            })
            .collect();
        // Stable record identity owns whole-dollar allocation remainders.
        entries.sort_by_key(|(id, _)| *id);
        let allocated: Vec<_> =
            entries.iter().map(|(id, _)| business_half_se.get(id).copied().unwrap_or(0)).collect();
        let receipts: Vec<_> = entries
            .iter()
            .map(|(_, v)| {
                if matches!(v.source, crate::BusinessSource::ForeignServices { .. }) {
                    v.receipts()
                } else {
                    0
                }
            })
            .collect();
        let expenses = match entries
            .iter()
            .zip(&allocated)
            .map(|((id, v), half)| {
                Ok::<_, crate::Section179PreparedIssue>(
                    if matches!(v.source, crate::BusinessSource::ForeignServices { .. }) {
                        crate::section179_return::expenses_with_claims(
                            d,
                            *id,
                            v,
                            selected_regular,
                            Some(office_context),
                            source_claims,
                        )
                        .map(|expense| {
                            expense
                                - selected_regular.map_or(0, |allocation| {
                                    allocation.ordinary_limit_adjustment(*id)
                                })
                        })? + half * 100
                    } else {
                        0
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
        // Allocate excluded receipts and restored expenses separately. Stable IDs
        // retain whole-dollar remainders, and signed net shares conserve Form 2555.
        let exclusions = business_exclusion.allocate(&receipts, &expenses);
        for (((id, v), allocated), exclusion) in entries.iter().zip(allocated).zip(exclusions) {
            if matches!(v.source, crate::BusinessSource::ForeignServices { .. }) {
                let net = dollars(
                    v.receipts()
                        - match crate::section179_return::expenses_with_claims(
                            d,
                            *id,
                            v,
                            selected_regular,
                            Some(office_context),
                            source_claims,
                        )
                        .map(|expense| {
                            expense
                                - selected_regular.map_or(0, |allocation| {
                                    allocation.ordinary_limit_adjustment(*id)
                                })
                        }) {
                            Ok(value) => value,
                            Err(issue) => {
                                crate::section179_return::Issue::from(issue).report(r);
                                r.state = ResultState::Unsupported;
                                return core::ops::ControlFlow::Break(());
                            }
                        },
                ) - exclusion
                    - allocated
                    - health.by_business.get(id).copied().unwrap_or(0)
                    - sep.by_business.get(id).copied().unwrap_or(0);
                research_business_income.insert(*id, net);
                foreign_business_losses += positive(-net);
                foreign_business_profits += positive(net);
                line(
                    r,
                    &alloc::format!("business_{}_foreign_net_income", id),
                    dollars(
                        v.receipts()
                            - match crate::section179_return::expenses_with_claims(
                                d,
                                *id,
                                v,
                                selected_regular,
                                Some(office_context),
                                source_claims,
                            )
                            .map(|expense| {
                                expense
                                    - selected_regular.map_or(0, |allocation| {
                                        allocation.ordinary_limit_adjustment(*id)
                                    })
                            }) {
                                Ok(value) => value,
                                Err(issue) => {
                                    crate::section179_return::Issue::from(issue).report(r);
                                    r.state = ResultState::Unsupported;
                                    return core::ops::ControlFlow::Break(());
                                }
                            },
                    ),
                    &["income.business.source", "income.business"],
                    "https://www.irs.gov/instructions/i8995",
                    msg!("EXPLAIN_BUSINESS_N_FOREIGN_NET_INCOME"),
                );
                continue;
            }
            if matches!(v.activity, crate::BusinessActivity::Farming { .. }) {
                farm_qbi_indices.push(activities.len());
            }
            let land_qbi = match crate::farm_land::qbi_cents_with_claims(
                d,
                rental,
                property_character,
                *id,
                source_claims,
            ) {
                Ok(value) => value,
                Err(error) => {
                    error.report_at(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::Farming),
                        #[cfg(test)]
                        "Farm land QBI",
                    );
                    if error.is_required_source() {
                        return core::ops::ControlFlow::Break(());
                    }
                    0
                }
            };
            let installment_part = installment_qbi
                .iter()
                .filter(|(business, _, _)| *business == Some(*id))
                .map(|(_, part, _)| *part)
                .sum::<i64>();
            let installment_part =
                match crate::extended_rules::annual_dollars_to_working_cents(installment_part) {
                    Ok(value) => value,
                    Err(error) => {
                        crate::activity_risk::SourceIssue::new(*id, error.into()).report(r);
                        return core::ops::ControlFlow::Break(());
                    }
                };
            let land_qbi = land_qbi + installment_part;
            business_qbi_ids.insert(activities.len(), *id);
            // A community-property return takes its share of the business's
            // profit, wages and property (Form 8958); the operator's own
            // self-employment deductions stay with the operator.
            let divided = allocated_entries
                .iter()
                .find(|e| e.id == *id)
                .and_then(|e| crate::community::share(s, e.owner.owned(), e.community, &e.income));
            let mut projected = t.business_sources[id][0];
            let operator = divided.is_none_or(|share| share.filer_owns());
            let source_operating =
                match farm_receipts_with_refunds_with_claims(d, *id, v, false, source_claims) {
                    Ok(value) => value,
                    Err(issue) => {
                        crate::section179_return::Issue::from(issue).report(r);
                        r.state = ResultState::Unsupported;
                        return core::ops::ControlFlow::Break(());
                    }
                } - match crate::section179_return::expenses_with_claims(
                    d,
                    *id,
                    v,
                    selected_regular,
                    Some(office_context),
                    source_claims,
                ) {
                    Ok(value) => value,
                    Err(issue) => {
                        crate::section179_return::Issue::from(issue).report(r);
                        r.state = ResultState::Unsupported;
                        return core::ops::ControlFlow::Break(());
                    }
                } + selected_regular
                    .map_or(0, |allocation| allocation.ordinary_limit_adjustment(*id));
            let cooperative = match crate::cooperative::receipt_totals_with_claims(
                d,
                if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
                *id,
                false,
                source_claims,
            ) {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            };
            research_business_income.insert(
                *id,
                dollars(projected.qualified_income(d, *id, source_operating))
                    - if operator {
                        business_half_se.get(id).copied().unwrap_or(0)
                            + health.by_business.get(id).copied().unwrap_or(0)
                            + sep.by_business.get(id).copied().unwrap_or(0)
                            + tips_deducted.get(id).copied().unwrap_or(0)
                    } else {
                        0
                    },
            );
            let qualified_net = source_operating - cooperative.nonqualified
                + match crate::clergy::nondeductible_with_section179_with_claims(
                    d,
                    *id,
                    selected_regular,
                    Some(office_context),
                    source_claims,
                ) {
                    Ok(value) => value,
                    Err(issue) => {
                        crate::section179_return::Issue::from(issue).report(r);
                        r.state = ResultState::Unsupported;
                        return core::ops::ControlFlow::Break(());
                    }
                };
            let operator_deductions = if operator {
                qbi_half_se.sources.get(id).copied().unwrap_or(0)
                    + health.by_business.get(id).copied().unwrap_or(0)
                    + sep.by_business.get(id).copied().unwrap_or(0)
                    + tips_deducted.get(id).copied().unwrap_or(0)
            } else {
                0
            };
            let operator_deductions =
                match crate::extended_rules::annual_dollars_to_working_cents(operator_deductions) {
                    Ok(value) => value,
                    Err(error) => {
                        crate::activity_risk::SourceIssue::new(*id, error.into()).report(r);
                        return core::ops::ControlFlow::Break(());
                    }
                };
            let qualified =
                projected.qualified_income(d, *id, qualified_net) + land_qbi - operator_deductions;
            let property = v
                .assets
                .iter()
                .filter(|asset| {
                    asset.qbi_property_in_context(office_context.macrs.as_deref().map_or_else(
                        Default::default,
                        |view| {
                            view.context(
                                if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
                                Some(*id),
                                asset,
                            )
                        },
                    ))
                })
                .map(crate::BusinessAsset::business_basis)
                .sum::<i64>();
            projected.bind_qbi(d, *id, qualified, v.payroll.qbi_wages(), property);
            t.business_sources.get_mut(id).unwrap()[0] = projected;
            activities.push(crate::business_rules::QbiActivity {
                source_wages_cents: Some(projected.qbi_wages_cents),
                source_property_cents: Some(projected.qbi_property_cents),
                source_income_cents: Some(projected.qbi_income_cents),
                owner: Some(if owner == 0 { Owner::Taxpayer } else { Owner::Spouse }),
                income: dollars(projected.qbi_income_cents),
                wages: dollars(projected.qbi_wages_cents),
                property: dollars(projected.qbi_property_cents),
                active: (direct_participation.get(*id) == crate::participation::Participation::Yes),
                service: Some(v.specified_service),
                group: v.qbi_aggregation_group.clone(),
            });
            crate::business_rules::assume_service(
                activities.last().unwrap(),
                before_qbi,
                s.filing_status(),
                r,
                crate::IssueContext::income_id(*id),
            );
        }
    }
    crate::community::SourceProjection::bind_business(rental, &t.business_sources, false);
    for entry in allocated_entries {
        if let FederalIncome::Wage { data } = &entry.income
            && crate::FederalWage::from(data).statutory_employee
            && crate::FederalWage::from(data).statutory_business.is_none()
        {
            let data = crate::FederalWage::from(data);
            let activity = crate::business_rules::QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                owner: Some(entry.owner.owned()),
                income: dollars(data.wages),
                wages: 0,
                property: 0,
                active: true,
                service: None,
                group: None,
            };
            crate::business_rules::assume_service(
                &activity,
                before_qbi,
                s.filing_status(),
                r,
                crate::interview::income_issue_context(entry),
            );
            activities.push(activity);
            for (reference, suffix) in [("1", "gross_receipts"), ("31", "net_profit")] {
                form_line(
                    r,
                    FormLine::new(TaxForm::ScheduleC, reference),
                    &alloc::format!("statutory_wage_{}_{}", entry.id, suffix),
                    dollars(data.wages),
                    &["income.wages"],
                    "https://www.irs.gov/instructions/i1040sc",
                    msg!("EXPLAIN_BUSINESS_STATUTORY_W2"),
                );
            }
        }
    }
    standalone_installment_qbi(d, &installment_qbi, &mut activities);
    core::ops::ControlFlow::Continue(DirectQbiActivitiesOut {
        activities,
        research_business_income,
        farm_qbi_indices,
        business_qbi_ids,
        foreign_business_losses,
        foreign_business_profits,
    })
}

pub(super) struct QbiExcessLossIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) property_loss_deferred: Result<i64, (&'static str, &'static str)>,
    pub(super) business_qbi_ids: &'a alloc::collections::BTreeMap<usize, i64>,
    pub(super) research_qbi_ids: &'a alloc::collections::BTreeMap<usize, i64>,
    pub(super) ptp_loss_sources: &'a [PtpLossSource],
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) foreign_business: bool,
    pub(super) qbi_gross_unknown: bool,
    pub(super) partner_qbi_complete: bool,
    pub(super) property_business_income: i64,
    pub(super) excess_business_loss: i64,
    pub(super) k1_business_ordinary: i64,
    pub(super) before_qbi: i64,
    pub(super) foreign_business_losses: i64,
    pub(super) foreign_business_profits: i64,
}

pub(super) struct QbiExcessLossOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
}

pub(super) fn qbi_excess_loss_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: QbiExcessLossIn<'_>,
) -> core::ops::ControlFlow<(), QbiExcessLossOut> {
    let &Pass { d, s, allocated_entries, selected_regular, context, .. } = pass;
    let source_claims = context.source_claims;
    let QbiExcessLossIn {
        rental,
        property_character,
        property_loss_deferred,
        business_qbi_ids,
        research_qbi_ids,
        ptp_loss_sources,
        mut activities,
        mut ptp_qualified,
        mut injured_ptp,
        foreign_business,
        qbi_gross_unknown,
        partner_qbi_complete,
        property_business_income,
        excess_business_loss,
        k1_business_ordinary,
        before_qbi,
        foreign_business_losses,
        foreign_business_profits,
    } = input;
    let direct_business_loss_scope = property_business_income == 0
        && rental.ordinary_pass_through_business() == 0
        && rental.qbi == 0
        && !allocated_entries.iter().any(|e| {
            matches!(
                e.income,
                FederalIncome::Scorp { .. }
                    | FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Partnership { .. }
            )
        });
    let exclusively_foreign_loss = foreign_business
        && activities.iter().all(|activity| activity.income >= 0)
        && direct_business_loss_scope;
    let ordinary_domestic_sources = allocated_entries.iter().all(|e| match &e.income {
        FederalIncome::Business { data } if data.source == crate::BusinessSource::UnitedStates => {
            matches!(data.activity, crate::BusinessActivity::Ordinary { .. })
        }
        _ => true,
    });
    let fully_qualified_domestic_sources =
        crate::business_rules::qbi_phase(before_qbi, s.filing_status()).0 == 0
            || allocated_entries.iter().all(|e| match &e.income {
                FederalIncome::Business { data }
                    if data.source == crate::BusinessSource::UnitedStates =>
                {
                    !data.specified_service
                }
                _ => true,
            });
    let exclusively_qualified_loss = foreign_business
        && foreign_business_losses == 0
        && direct_business_loss_scope
        && ordinary_domestic_sources
        && fully_qualified_domestic_sources;
    let mixed_loss_only = foreign_business_losses > 0
        && foreign_business_profits == 0
        && direct_business_loss_scope
        && ordinary_domestic_sources
        && activities.iter().any(|activity| activity.income < 0)
        && activities.iter().all(|activity| activity.income <= 0);
    crate::require_localized!(
        r,
        !foreign_business
            || qbi_gross_unknown
            || excess_business_loss == 0
            || exclusively_foreign_loss
            || exclusively_qualified_loss
            || mixed_loss_only
            // The partnership-aware allocator below owns mixed source validation.
            || allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Partnership { .. })),
        &crate::IssueContext::Business,
        "Foreign business loss allocation",
        "ISSUE_MIXED_QUALIFIED_NONQUALIFIED_LOSSES_COMBINED_WITH_PROFITABLE_ACTIVI_843F0E1E",
        "Mixed qualified/nonqualified losses combined with profitable activities, or special entity/property character, need further suspended-loss allocation. Loss-only ordinary sources preserve their qualified proportions.",
    );
    if !qbi_gross_unknown && exclusively_foreign_loss && excess_business_loss > 0 {
        line(
            r,
            "business_nonqualified_excess_loss",
            excess_business_loss,
            &["income.business.source", "excess_business_loss"],
            "https://www.irs.gov/instructions/i8995",
            msg!("EXPLAIN_BUSINESS_NONQUALIFIED_EXCESS_LOSS"),
        );
    }
    if !qbi_gross_unknown && exclusively_qualified_loss && excess_business_loss > 0 {
        line(
            r,
            "business_qualified_excess_loss",
            excess_business_loss,
            &["income.business.source", "excess_business_loss"],
            "https://www.irs.gov/instructions/i8995",
            msg!("EXPLAIN_BUSINESS_QUALIFIED_EXCESS_LOSS"),
        );
    }
    let k1_excess_pending = !qbi_gross_unknown
        && excess_business_loss > 0
        && allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }));
    let partner_excess_pending = !qbi_gross_unknown
        && excess_business_loss > 0
        && allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Partnership { .. }));
    let resolved_k1_excess = if property_loss_deferred.is_ok()
        && partner_qbi_complete
        && !qbi_gross_unknown
        && ptp_loss_sources.is_empty()
    {
        crate::k1_excess::qualified_deferral(
            d,
            rental,
            &mut activities,
            rental.ordinary_pass_through_business() + property_business_income,
            excess_business_loss,
            rental.ordinary_pass_through_business() == 0
                && k1_business_ordinary == 0
                && property_character.is_some_and(|v| v.net < 0),
            (before_qbi, s.filing_status()),
        )
    } else {
        None
    };
    if partner_excess_pending {
        let allocation = if !ptp_loss_sources.is_empty() {
            (|| {
                // Registered PTP losses must never fall back to an
                // ordinary-only allocation when property is unresolved.
                if !partner_qbi_complete
                    || property_loss_deferred.is_err()
                    || property_business_income != 0
                {
                    return Err(crate::partnership_qbi::ExcessError::Rule((
                        "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER",
                        "Resolve partnership QBI and property character before excess-loss allocation.",
                    )));
                }
                let error = crate::partnership_qbi::ExcessError::Rule((
                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    "Review the document amounts; they must be within the supported input range.",
                ));
                for entry in allocated_entries {
                    if let FederalIncome::Partnership { data } = &entry.income
                        && data.publicly_traded
                        && let Some((_, _, _, current, historical, financial_net)) =
                            ptp_loss_sources.iter().find(|(id, _, _, _, _, _)| *id == entry.id)
                    {
                        crate::partnership_qbi::ptp_excess_source(
                            data,
                            *current,
                            *historical,
                            *financial_net,
                        )
                        .map_err(|detail| {
                            let mut source = crate::section1231::SourceError::from(detail);
                            source.context = alloc::boxed::Box::new(
                                crate::interview::income_issue_context(entry),
                            );
                            crate::partnership_qbi::ExcessError::Source(source)
                        })?;
                    }
                }
                let ordinary_count = activities.len();
                let mut candidate = activities.clone();
                for (_, owner, _, current, historical, _) in ptp_loss_sources {
                    for amount in [current.get(), historical.get()] {
                        candidate.push(crate::business_rules::QbiActivity {
                            source_wages_cents: Some(0),
                            source_property_cents: Some(0),
                            source_income_cents: Some(-amount),
                            owner: Some((*owner).owned()),
                            income: -dollars(amount),
                            wages: 0,
                            property: 0,
                            active: false,
                            service: Some(false),
                            group: None,
                        });
                    }
                }
                // One shared fraction/allocation covers every resolved
                // loss-only source, including different original qualified
                // fractions. Existing profit/mixed-source guards still own
                // admission; no PTP net can mask an individual profit.
                let qualified =
                    crate::partnership_qbi::allocate_excess_with_section179_with_claims(
                        context.forms(d),
                        rental,
                        &mut candidate,
                        excess_business_loss,
                        before_qbi,
                        s.filing_status(),
                        selected_regular,
                    )?;
                // Validate all replacements before changing either category.
                let replacements = candidate[ordinary_count..]
                    .chunks_exact(2)
                    .map(|rows| {
                        rows.iter().try_fold(0i64, |total, row| {
                            total.checked_add(row.source_income_cents?)
                        })
                    })
                    .collect::<Option<Vec<_>>>()
                    .ok_or(error)?;
                activities.clone_from_slice(&candidate[..ordinary_count]);
                for ((_, owner, previous, _, _, _), cents) in
                    ptp_loss_sources.iter().zip(replacements)
                {
                    let replacement = crate::kernel::ratio::exact::whole(cents);
                    ptp_qualified -= previous;
                    ptp_qualified += &replacement;
                    let owner_index = usize::from(*owner == crate::DocumentOwner::Spouses);
                    injured_ptp[owner_index] -= previous;
                    injured_ptp[owner_index] += replacement;
                }
                Ok(qualified)
            })()
        } else if let Some(qualified) = resolved_k1_excess.filter(|_| partner_qbi_complete) {
            Ok(qualified)
        } else if partner_qbi_complete
            && property_loss_deferred.is_ok()
            && rental.k1_section1231.iter().any(|source| source.net != 0)
        {
            crate::partnership_qbi::excess_with_property_with_claims(
                d,
                rental,
                &mut activities,
                excess_business_loss,
                before_qbi,
                s.filing_status(),
                source_claims,
            )
            .map_err(crate::partnership_qbi::ExcessError::Source)
        } else if partner_qbi_complete && property_business_income == 0 {
            crate::partnership_qbi::allocate_excess_with_section179_with_claims(
                context.forms(d),
                rental,
                &mut activities,
                excess_business_loss,
                before_qbi,
                s.filing_status(),
                selected_regular,
            )
        } else {
            Err(crate::partnership_qbi::ExcessError::Rule((
                "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER",
                "Resolve partnership QBI and property character before excess-loss allocation.",
            )))
        };
        match allocation {
            Ok(qualified) => r.review_notes.push(ReviewNote::localized(
                #[cfg(test)] alloc::format!("Partnership excess-loss character: NOL carryforward ${excess_business_loss}; qualified portion ${qualified}; nonqualified portion ${}. Qualified suspension is excluded from current QBI loss and retained for the year deducted; these portions are not additional NOL amounts.", excess_business_loss - qualified),
                Message::new("NOTE_PARTNERSHIP_EXCESS_LOSS_CHARACTER", alloc::vec![
                    MessageArg::usd_dollars(excess_business_loss), MessageArg::usd_dollars(qualified),
                    MessageArg::usd_dollars(excess_business_loss - qualified),
                ]),
            )),
            Err(crate::partnership_qbi::ExcessError::Source(error)) => {
                error.report_at(r, &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                    #[cfg(test)] "Partnership excess-loss allocation",
                );
                if error.is_required_source() {
                    return core::ops::ControlFlow::Break(());
                }
            }
            Err(crate::partnership_qbi::ExcessError::Prepared(issue)) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
            Err(crate::partnership_qbi::ExcessError::Rule((key, _message))) => crate::issue_localized!(r, &crate::IssueContext::Section(crate::TaxSection::PassThrough),"Partnership excess-loss allocation", key,_message),
        }
    } else if k1_excess_pending {
        let native_sources: Vec<_> = allocated_entries
            .iter()
            .filter_map(|e| match &e.income {
                FederalIncome::Scorp { data } => Some(crate::ScorpK1::from(data)),
                _ => None,
            })
            .collect();
        let sources: Vec<_> = native_sources.iter().collect();
        let other_business = allocated_entries.iter().any(|e| {
            matches!(
                e.income,
                FederalIncome::Business { .. }
                    | FederalIncome::Rental { .. }
                    | FederalIncome::FarmRental { .. }
            )
        });
        let allocation = if let Some(qualified) = resolved_k1_excess {
            Ok(qualified)
        } else if !other_business
            && sources.len() == 1
            && property_character.is_some_and(|v| sources[0].ordinary != 0 || v.net > 0)
        {
            property_loss_deferred.and_then(|_| {
                crate::scorp_qbi::excess_with_gain(
                    sources[0],
                    &mut activities,
                    excess_business_loss,
                    before_qbi,
                    s.filing_status(),
                )
            })
        } else if property_character.is_some_and(|v| v.net < 0) {
            property_loss_deferred.and_then(|deferred| {
                crate::scorp_qbi::defer_loss_only_qbi(
                    &mut activities,
                    -property_character.unwrap().net,
                    deferred,
                )
            })
        } else if !other_business {
            crate::scorp_qbi::uniform_business_excess(
                &sources,
                &mut activities,
                excess_business_loss,
            )
            .or_else(|_message| {
                if r.issues.is_empty() {
                    crate::scorp_qbi::loss_only_business_excess(
                        &sources,
                        &mut activities,
                        -rental.ordinary_pass_through_business(),
                        excess_business_loss,
                    )
                } else {
                    Err(_message)
                }
            })
        } else {
            Err((
                "ISSUE_MULTIPLE_BUSINESS_EXCESS_LOSS_ALLOCATION",
                crate::fixture_english!(
                    "Excess-loss allocation across multiple businesses remains incomplete."
                ),
            ))
        };
        match allocation {
            Ok(qualified) => r.review_notes.push(ReviewNote::localized(
                #[cfg(test)] alloc::format!("K-1 excess-loss character: NOL carryforward ${excess_business_loss}; qualified portion ${qualified}. This portion is excluded from current QBI and retained for the year the NOL is deducted; current QBI loss carryforward is separate."),
                Message::new("NOTE_K1_EXCESS_LOSS_CHARACTER", alloc::vec![
                    MessageArg::usd_dollars(excess_business_loss), MessageArg::usd_dollars(qualified),
                ]),
            )),
            Err((key, _message)) => crate::issue_localized!(r, &crate::IssueContext::Section(crate::TaxSection::PassThrough),"K-1 excess-loss allocation", key,_message),
        }
    } else if !qbi_gross_unknown && mixed_loss_only {
        let total_loss = foreign_business_losses
            + activities.iter().map(|activity| -activity.income).sum::<i64>();
        // Fix loss-year SSTB eligibility before allocating suspension. The
        // helper marks eligibility as resolved, preventing a second phaseout.
        for activity in &mut activities {
            match crate::business_rules::eligible_qbi_activity(
                activity,
                before_qbi,
                s.filing_status(),
            ) {
                Ok(eligible) => *activity = eligible,
                Err((key, _message)) => crate::issue_localized!(
                    r,
                    &crate::IssueContext::Business,
                    "Foreign business loss allocation",
                    key,
                    _message,
                ),
            }
        }
        match crate::scorp_qbi::defer_loss_only_qbi(
            &mut activities,
            total_loss,
            excess_business_loss,
        ) {
            Ok(qualified) => {
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("business_qualified_excess_loss", qualified),
                        ("business_nonqualified_excess_loss", excess_business_loss - qualified),
                    ],
                    |(id, amount), _, _| {
                        Some(calculation_row(
                            None,
                            id,
                            LineValue::MoneyDollars(amount),
                            &["income.business.source", "excess_business_loss"],
                            "https://www.irs.gov/instructions/i8995",
                            msg!("EXPLAIN_FEDERAL_RULES_7"),
                        ))
                    },
                );
            }
            Err((key, _message)) => crate::issue_localized!(
                r,
                &crate::IssueContext::Business,
                "Foreign business loss allocation",
                key,
                _message,
            ),
        }
    } else if !qbi_gross_unknown {
        let loss_weights: Vec<_> = activities.iter().map(|v| positive(-v.income)).collect();
        let deferred =
            crate::kernel::ratio::allocate_remaining(excess_business_loss, &loss_weights);
        for (index, (activity, deferred)) in activities.iter_mut().zip(deferred).enumerate() {
            if deferred != 0 {
                // Positive K-1 deferrals use the owner-specific branches above.
                // This is the non-K1 annual report adapter.
                activity.income += deferred;
                if let Some(source) = activity.source_income_cents {
                    let change =
                        match crate::extended_rules::annual_dollars_to_working_cents(deferred)
                            .and_then(|change| {
                                source
                                    .checked_add(change)
                                    .ok_or(crate::kernel::loss_limits::LossLimitError::Overflow)
                            }) {
                            Ok(value) => value,
                            Err(error) => {
                                crate::activity_risk::SourceIssue::new(
                                    business_qbi_ids
                                        .get(&index)
                                        .or_else(|| research_qbi_ids.get(&index))
                                        .copied()
                                        .unwrap_or(0),
                                    error.into(),
                                )
                                .report(r);
                                return core::ops::ControlFlow::Break(());
                            }
                        };
                    activity.source_income_cents = Some(change);
                }
            }
        }
    }
    core::ops::ControlFlow::Continue(QbiExcessLossOut { activities, ptp_qualified, injured_ptp })
}

pub(super) struct QbiFilingIn {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) business_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) qbi_gross_unknown: bool,
    pub(super) ptp_nol_complete: bool,
    pub(super) ptp_nol_owners_complete: bool,
    pub(super) before_qbi: i64,
    pub(super) pref: i64,
}

pub(super) struct QbiFilingOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) prior_business: i64,
    pub(super) prior_reit: i64,
    pub(super) qbi: crate::business_rules::QbiResult,
    pub(super) qbi_ded: i64,
    pub(super) injured_qbi: Option<crate::injured_spouse::QbiSource>,
    pub(super) research_qbi_deductions: alloc::collections::BTreeMap<i64, i64>,
    pub(super) qbi_carry: i64,
    pub(super) qbi_form: Option<TaxForm>,
}

pub(super) fn qbi_filing_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    input: QbiFilingIn,
) -> core::ops::ControlFlow<(), QbiFilingOut> {
    let &Pass { d, s, allocated_entries, selected_regular, office_context, context, .. } = pass;
    let source_claims = context.source_claims;
    let QbiFilingIn {
        mut activities,
        released_qbi,
        mut research_qbi_ids,
        business_qbi_ids,
        ptp_qualified,
        injured_ptp,
        qbi_gross_unknown,
        ptp_nol_complete,
        ptp_nol_owners_complete,
        before_qbi,
        pref,
    } = input;
    let history = &d.screening.as_ref().unwrap().qbi_history;
    let [prior_business, prior_reit] = history.amounts().map(dollars);
    // A patron of a specified agricultural or horticultural cooperative
    // (Form 1099-PATR box 7) files Form 8995-A at any income (2026 Form 8995
    // header; i8995 Form 1099-PATR items).
    let qbi_patron = allocated_entries.iter().any(
        |e| matches!(&e.income, FederalIncome::Cooperative { data } if data.form.specified_cooperative || data.form.qualified_payments > 0 || data.form.section_199a_g > 0),
    );
    research_qbi_ids.extend(business_qbi_ids.iter().map(|(i, id)| (*i, *id)));
    let (cooperative_reductions, cooperative_dpad, injured_dpad, cooperative_dpad_shares) =
        match crate::cooperative::qbi_filing_with_section179_with_claims(
            d,
            &mut activities,
            &business_qbi_ids,
            before_qbi,
            s.filing_status(),
            selected_regular,
            r,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
    // Equivalent-source cents meet at the single reported REIT/PTP category.
    let reit_ptp_cents = crate::kernel::ratio::exact::whole(t.reit_dividends) + ptp_qualified;
    let reported_reit_ptp =
        crate::kernel::ratio::exact::dollars(&reit_ptp_cents).expect("bounded REIT/PTP category");
    // These are actual owner sources after the existing community projection,
    // not weights or a second Share. The canonical signed owner boundary
    // independently checks footing to the joint report. Missing owner
    // provenance stays Unavailable instead of becoming a Spouse residual.
    let reit_ptp_owners = (ptp_nol_owners_complete && !t.injured_reit_owner_unavailable)
        .then(|| {
            crate::kernel::ratio::exact::project_rounded_pair(
                reported_reit_ptp,
                &reit_ptp_cents,
                [0, 1].map(|owner| &t.injured_reit_dividends[owner] + &injured_ptp[owner]),
                0,
            )
        })
        .flatten();
    let qbi = crate::business_rules::qbi_filed(
        if qbi_gross_unknown { &[] } else { &activities },
        before_qbi,
        pref,
        s.filing_status(),
        crate::business_rules::QbiOther {
            business_loss: prior_business,
            reit: if qbi_gross_unknown || !ptp_nol_complete { 0 } else { reported_reit_ptp },
            reit_loss: prior_reit,
        },
        &crate::business_rules::QbiFiling {
            released: released_qbi,
            patron: qbi_patron,
            patron_allocations: cooperative_reductions,
            dpad: if qbi_gross_unknown { 0 } else { cooperative_dpad },
        },
        r,
    );
    // NOL limit and absorption exclude QBI before section 68 is evaluated,
    // not merely by adding QBI back to the resulting taxable income.
    let qbi_ded = if nol_run.exclude_qbi { 0 } else { qbi.deduction };
    let injured_qbi = (|| {
        if qbi_gross_unknown || !ptp_nol_complete {
            return None;
        }
        let retained = if history.any() {
            let owners = history.injured_owners.as_ref()?;
            if !owners.valid(history.amounts()) {
                return None;
            }
            owners.amounts()
        } else {
            [[0; 2]; 2]
        };
        // Project each known source category to the same rounded joint pool.
        let loss = [0, 1].map(|category| {
            let first = dollars(retained[0][category]);
            [first, [prior_business, prior_reit][category] - first]
        });
        let income = qbi.owner_income?;
        // Same reported category and signed source projection as Form8995;
        // prior reported loss remains fixed and is never rephased.
        let reit = reit_ptp_owners?;
        let mut weights = [0, 1].map(|owner| {
            positive(income[owner] - loss[0][owner]) + positive(reit[owner] - loss[1][owner])
        });
        if weights == [0; 2] && qbi.lines.minimum > qbi.lines.limited + qbi.lines.dpad {
            // The joint minimum is established by actual active-business QBI,
            // even when retained losses eliminate the ordinary percentage base.
            weights = qbi.owner_active?.map(positive);
        }
        Some(crate::injured_spouse::QbiSource {
            weights,
            dpad: injured_dpad,
            allowed_dpad: qbi.lines.dpad,
        })
    })();
    let mut research_qbi_deductions = alloc::collections::BTreeMap::<i64, i64>::new();
    if !nol_run.exclude_qbi {
        let dpad_parts =
            crate::kernel::ratio::allocate_remaining(qbi.lines.dpad, &cooperative_dpad_shares);
        for (i, share) in qbi.activity_shares.iter().enumerate() {
            if let Some(id) = research_qbi_ids.get(&i) {
                *research_qbi_deductions.entry(*id).or_default() +=
                    *share + dpad_parts.get(i).copied().unwrap_or(0);
            }
        }
    }
    let qbi_carry = qbi.business_loss;
    // The QBI form the return files: Form 8995 at or below the threshold,
    // Form 8995-A above it; none without QBI, REIT/PTP income or history.
    let qbi_form = (!qbi_gross_unknown
        && ptp_nol_complete
        && !nol_run.exclude_qbi
        && (!activities.is_empty()
            || prior_business != 0
            || prior_reit != 0
            || reported_reit_ptp != 0))
        .then(|| match crate::business_rules::qbi_phase(before_qbi, s.filing_status()).0 {
            0 if !qbi_patron => TaxForm::Form8995,
            _ => TaxForm::Form8995A,
        });
    if let Some(form) = qbi_form {
        qbi_lines(r, form, &qbi.lines, &business_qbi_ids);
    }
    core::ops::ControlFlow::Continue(QbiFilingOut {
        activities,
        prior_business,
        prior_reit,
        qbi,
        qbi_ded,
        injured_qbi,
        research_qbi_deductions,
        qbi_carry,
        qbi_form,
    })
}

pub(super) struct QbiSourceSetupIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) partner_qbi_complete: bool,
    pub(super) excess_business_loss: i64,
}

pub(super) struct QbiSourceSetupOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) community_qbi_loss_sources: alloc::collections::BTreeSet<i64>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) research_business_income: alloc::collections::BTreeMap<i64, i64>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) partner_qbi_complete: bool,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) ptp_nol_complete: bool,
    pub(super) ptp_nol_owners_complete: bool,
    pub(super) ptp_nol_predebt_carry: bool,
    pub(super) ptp_loss_sources: Vec<PtpLossSource>,
}

pub(super) fn qbi_source_setup_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    nol_run: &crate::nol::NolRun,
    input: QbiSourceSetupIn<'_>,
) -> core::ops::ControlFlow<(), QbiSourceSetupOut> {
    let &Pass { d, allocated_entries, context, .. } = pass;
    let source_claims = context.source_claims;
    let QbiSourceSetupIn { rental, mut partner_qbi_complete, excess_business_loss } = input;
    let mut activities = rental.activities.clone();
    let mut community_qbi_loss_sources: alloc::collections::BTreeSet<i64> =
        rental.qbi_loss_sources.iter().copied().collect();
    let mut research_qbi_ids: alloc::collections::BTreeMap<usize, i64> =
        rental.qbi_source_ids.iter().copied().enumerate().collect();
    let research_business_income = alloc::collections::BTreeMap::<i64, i64>::new();
    // Positions of released earlier-year qualified losses (Form 8995 line 3).
    let mut released_qbi: Vec<usize> = Vec::new();
    match crate::section1231::rental_property_qbi_with_claims(d, rental, source_claims) {
        Ok(amounts) => {
            for (index, amount, source_amount) in amounts.current {
                match source_amount {
                    Some(amount) => {
                        let source = activities[index]
                            .source_income_cents
                            .or_else(|| {
                                crate::extended_rules::annual_dollars_to_working_cents(
                                    activities[index].income,
                                )
                                .ok()
                            })
                            .and_then(|source| source.checked_add(amount));
                        activities[index].source_income_cents = source;
                        if let Some(source) = source {
                            activities[index].income = dollars(source);
                        } else {
                            partner_qbi_complete = false;
                            crate::issue_localized!(
                                r,
                                &crate::IssueContext::income_id(rental.qbi_source_ids[index]),
                                "K-1 qualified business income",
                                "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
                                "Resolve the K-1 activity's source amounts before completing its qualified business income."
                            );
                        }
                    }
                    None => activities[index].income += amount,
                }
                if activities[index].income < 0 {
                    community_qbi_loss_sources.insert(rental.qbi_source_ids[index]);
                }
            }
            for (id, loss) in amounts.historical {
                let Some(entry) = allocated_entries.iter().find(|entry| entry.id == id) else {
                    continue;
                };
                community_qbi_loss_sources.insert(id);
                released_qbi.push(activities.len());
                research_qbi_ids.insert(activities.len(), id);
                activities.push(crate::business_rules::QbiActivity {
                    source_wages_cents: None,
                    source_property_cents: None,
                    source_income_cents: None,
                    owner: Some(entry.owner.owned()),
                    income: -loss,
                    wages: 0,
                    property: 0,
                    active: false,
                    service: Some(false),
                    group: None,
                });
            }
        }
        Err(error) => {
            partner_qbi_complete = false;
            error.report_at(
                r,
                &error.context,
                #[cfg(test)]
                "Rental property QBI",
            );
            if error.is_required_source() {
                return core::ops::ControlFlow::Break(());
            }
        }
    }
    // Qualified PTP income or loss (Reg. 1.199A-1(d)(3)(iii)): combines with
    // qualified REIT dividends below (Form 8995 line 6 / Form 8995-A line 28)
    // instead of joining `activities`, so it is never wage/UBIA-limited or
    // aggregated and never nets against ordinary QBI business losses.
    // Transient source cents only. No document or equivalent-source amount
    // rounds before the single REIT/PTP reported category below.
    let mut ptp_qualified = crate::kernel::ratio::exact::whole(0);
    let mut injured_ptp =
        core::array::from_fn::<_, 2, _>(|_| crate::kernel::ratio::exact::whole(0));
    // Section 461(l) PTP losses enter only when the settled NOL allocation
    // actually deducts them. They are separate PTP losses, never ordinary QBI
    // activities or a second prior Form 8995 carry. The limit passes exclude
    // QBI and never feed this character back into the section 172 limit.
    let mut ptp_nol_complete = true;
    let mut ptp_nol_owners_complete = true;
    let mut ptp_nol_predebt_carry = false;
    if !nol_run.exclude_qbi
        && let Some(allocation) = &nol_run.allocation
    {
        for part in allocation.ptp_deductions(d) {
            if let Some(qualified) = part.qualified {
                ptp_qualified -= crate::kernel::ratio::exact::whole(qualified);
                if let Some(owners) = part.owners {
                    for owner in 0..2 {
                        injured_ptp[owner] -= crate::kernel::ratio::exact::whole(owners[owner]);
                    }
                } else {
                    ptp_nol_owners_complete = false;
                }
                ptp_nol_predebt_carry |= part.closing.is_some_and(|amount| amount > 0);
                for (suffix, amount, explanation) in [
                    ("deducted", Some(qualified), msg!("EXPLAIN_NOL_PTP_EBL_QUALIFIED")),
                    (
                        "pre_debt_closing",
                        part.closing,
                        msg!("EXPLAIN_NOL_PTP_EBL_PRE_DEBT_CLOSING"),
                    ),
                    ("lost", part.lost, msg!("EXPLAIN_NOL_PTP_EBL_LOST")),
                ] {
                    if let Some(amount) = amount {
                        line_cents(
                            r,
                            &alloc::format!("nol_{}_ptp_qualified_{suffix}", part.year),
                            amount,
                            &["screening.nol.ptp_ebl", "prior_nol_deduction"],
                            "https://www.ecfr.gov/current/title-26/section-1.199A-3",
                            explanation,
                        );
                    }
                }
            } else {
                ptp_nol_complete = false;
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_NOL_PTP_EBL_UNAVAILABLE",
                        alloc::vec![MessageArg::Number(part.year as i64)],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::SelfEmployment),
                );
            }
        }
    }
    // Loss-only EBL work keeps PTP source identity/category separate from
    // ordinary activities. Each tuple carries the already computed source,
    // its actual owner, and current versus original-year qualified losses.
    let ptp_loss_sources = Vec::new();
    // The qualified PTP items on a K-1's own section 199A statement
    // (box 20 code Z, box 17 code V) join the same component; its section 199A
    // dividends are in `t.reit_dividends`.
    for e in allocated_entries {
        let current_ptp = match &e.income {
            FederalIncome::Partnership { data } => data.qbi.as_ref().map_or(0, |v| v.ptp_income),
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.qbi_statement.as_ref().map_or(0, |v| v.ptp_income)
            }
            _ => 0,
        };
        // A separate scalar statement does not identify the financial gross
        // loss/business needed for EBL attribution. Check each actual source,
        // so another issuer's positive PTP items cannot mask this boundary.
        if excess_business_loss > 0 && current_ptp < 0 {
            partner_qbi_complete = false;
            crate::issue_localized!(
                r,
                &crate::interview::income_issue_context(e),
                "PTP component source",
                "ISSUE_QBI_PTP_COMPONENT_CLASSIFICATION",
                "Separately reported PTP items need their actual business and specified-service source classification before the income limit can be applied."
            );
            continue;
        }
        let current_ptp = crate::kernel::ratio::exact::whole(current_ptp);
        ptp_qualified += &current_ptp;
        injured_ptp[usize::from(e.owner == crate::DocumentOwner::Spouses)] += current_ptp;
    }
    for entry in allocated_entries {
        if let FederalIncome::ScorpActivities { data } = &entry.income
            && rental.ordinary_business_by_id.contains_key(&entry.id)
        {
            if let Ok(output) = data.ordinary_source(false) {
                for activity in output.current_qualified {
                    research_qbi_ids.insert(activities.len(), entry.id);
                    activities.push(activity.activity.with_owner(entry.owner.owned()));
                }
            }
            // Only an admitted graph establishes the no-statement cohort.
            // Unsupported statements/history retain their source boundary.
            if data.activities.source().is_some_and(|source| {
                source.qbi_blocks.is_empty()
                    || source.qbi_blocks.iter().any(|block| {
                        matches!(
                            block.statement,
                            crate::k1_activity::K1ReportedQbiStatement::Scorp { statement: None }
                        )
                    })
            }) {
                note_once_scoped(
                    r,
                    Message::new(
                        if data.current.ordinary < 0 {
                            "WARN_K1_QBI_LOSS_NO_STATEMENT"
                        } else {
                            "WARN_K1_QBI_NO_STATEMENT"
                        },
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    crate::interview::income_issue_context(entry),
                );
            }
        }
    }
    core::ops::ControlFlow::Continue(QbiSourceSetupOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        research_business_income,
        released_qbi,
        partner_qbi_complete,
        ptp_qualified,
        injured_ptp,
        ptp_nol_complete,
        ptp_nol_owners_complete,
        ptp_nol_predebt_carry,
        ptp_loss_sources,
    })
}

pub(super) struct PartnershipQbiSourcesIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) qbi_half_se: &'a crate::partnership_qbi::QbiHalfSeAllocation,
    pub(super) care: &'a crate::care::CareDeductions,
    pub(super) before_qbi: i64,
    pub(super) excess_business_loss: i64,
    pub(super) qbi_gross_unknown: bool,
    pub(super) partner_qbi_complete: bool,
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) community_qbi_loss_sources: alloc::collections::BTreeSet<i64>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) ptp_loss_sources: Vec<PtpLossSource>,
}

pub(super) struct PartnershipQbiSourcesOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) community_qbi_loss_sources: alloc::collections::BTreeSet<i64>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) ptp_loss_sources: Vec<PtpLossSource>,
    pub(super) partner_qbi_complete: bool,
}

pub(super) fn partnership_qbi_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: PartnershipQbiSourcesIn<'_>,
) -> core::ops::ControlFlow<(), PartnershipQbiSourcesOut> {
    let &Pass { d, s, allocated_entries, context, .. } = pass;
    let source_claims = context.source_claims;
    let PartnershipQbiSourcesIn {
        rental,
        health,
        sep,
        qbi_half_se,
        care,
        before_qbi,
        excess_business_loss,
        qbi_gross_unknown,
        mut partner_qbi_complete,
        mut activities,
        mut community_qbi_loss_sources,
        mut research_qbi_ids,
        mut released_qbi,
        mut ptp_qualified,
        mut injured_ptp,
        mut ptp_loss_sources,
    } = input;
    for partner in &rental.partners {
        let Some(entry) = allocated_entries.iter().find(|e| e.id == partner.id) else {
            continue;
        };
        if let FederalIncome::PartnershipActivities { data } = &entry.income {
            // Each admitted component keeps current statement qualification or
            // original-year retained qualification through its existing owner.
            if data.activities.source().is_some_and(|source| {
                source.qbi_blocks.is_empty()
                    || source.qbi_blocks.iter().any(|block| {
                        matches!(
                            block.statement,
                            crate::k1_activity::K1ReportedQbiStatement::Partnership {
                                statement: None
                            }
                        )
                    })
            }) {
                note_once_scoped(
                    r,
                    Message::new(
                        if data.current.ordinary < 0 {
                            "WARN_K1_QBI_LOSS_NO_STATEMENT"
                        } else {
                            "WARN_K1_QBI_NO_STATEMENT"
                        },
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    crate::interview::income_issue_context(entry),
                );
            }
            if let Ok(output) = data.ordinary_source(false) {
                for source in output.current_qualified {
                    research_qbi_ids.insert(activities.len(), entry.id);
                    // The actual block's global gross share reduces its QBI before
                    // the shared owner applies current SSTB and wage/UBIA eligibility.
                    let mut activity = source.activity;
                    let half = qbi_half_se
                        .native_blocks
                        .get(&(entry.id, source.block))
                        .copied()
                        .unwrap_or(0);
                    let cents = activity.source_income_cents.and_then(|income| {
                        half.checked_mul(100).and_then(|half| income.checked_sub(half))
                    });
                    let Some(cents) = cents else {
                        partner_qbi_complete = false;
                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(entry),
                            "K-1 qualified business income",
                            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                            "Review the document amounts; they must be within the supported input range."
                        );
                        continue;
                    };
                    activity.source_income_cents = Some(cents);
                    activity.income = dollars(cents);
                    activities.push(activity.with_owner(entry.owner.owned()));
                }
                for qualified in output.qualified {
                    if qualified.released.get() > 0 {
                        community_qbi_loss_sources.insert(entry.id);
                        released_qbi.push(activities.len());
                        research_qbi_ids.insert(activities.len(), entry.id);
                        activities.push(crate::business_rules::QbiActivity {
                            source_wages_cents: Some(0),
                            source_property_cents: Some(0),
                            source_income_cents: Some(-qualified.released.get()),
                            owner: Some(entry.owner.owned()),
                            income: -dollars(qualified.released.get()),
                            wages: 0,
                            property: 0,
                            active: false,
                            service: Some(false),
                            group: None,
                        });
                    }
                    let source = MessageArg::Text(alloc::format!(
                        "{} ({})",
                        qualified.name,
                        qualified.component
                    ));
                    let context = crate::interview::income_issue_context(entry);
                    for (key, carry) in
                        ["NOTE_PARTNERSHIP_BASIS_LOSS_CARRY", "NOTE_PARTNERSHIP_AT_RISK_LOSS_CARRY"]
                            .into_iter()
                            .zip(qualified.carries)
                    {
                        for year in carry.remaining.iter().filter(|v| v.remaining.get() > 0) {
                            note_once_scoped(
                                r,
                                Message::new(
                                    key,
                                    alloc::vec![
                                        source.clone(),
                                        MessageArg::Number(i64::from(year.year)),
                                        MessageArg::usd_cents(year.remaining.get()),
                                        MessageArg::usd_cents(year.remaining_qbi.get()),
                                        MessageArg::usd_cents(year.original.get()),
                                        MessageArg::usd_cents(year.original_qbi.get()),
                                    ],
                                ),
                                context.clone(),
                            );
                        }
                    }
                    note_once_scoped(
                        r,
                        Message::new(
                            "NOTE_PARTNERSHIP_HISTORICAL_QBI_RELEASE",
                            alloc::vec![source, MessageArg::usd_cents(qualified.released.get()),],
                        ),
                        context,
                    );
                }
            }
            continue;
        }
        let FederalIncome::Partnership { data } = &entry.income else {
            continue;
        };
        let ptp_native_net = if data.publicly_traded && excess_business_loss > 0 {
            let Some(cents) = rental
                .k1_amounts
                .iter()
                .find(|source| source.id == entry.id)
                .and_then(|source| source.source_net_cents)
            else {
                partner_qbi_complete = false;
                crate::issue_localized!(
                    r,
                    &crate::interview::income_issue_context(entry),
                    "PTP excess-loss source",
                    "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB",
                    "Reconcile partnership QBI with the resolved passive limit and partner deductions."
                );
                continue;
            };
            Some(cents)
        } else {
            None
        };
        let partner_half = qbi_half_se.sources.get(&entry.id).copied().unwrap_or(0);
        let insurance = health.by_business.get(&entry.id).copied().unwrap_or(0);
        let retirement = sep.by_business.get(&entry.id).copied().unwrap_or(0);
        // No statement reports zero current QBI (Reg. 1.199A-6(b)(3)(iii)).
        // Retained qualified losses still count when allowed, with their
        // original-year status (i8995); no current allocation input is needed.
        let Some(statement) = data.qbi.as_ref() else {
            let context = crate::interview::income_issue_context(entry);
            note_once_scoped(
                r,
                Message::new(
                    if data.regular.ordinary < 0
                        || crate::k1_sale_qbi::has_loss(data.regular.sale_results(&data.items()))
                    {
                        "WARN_K1_QBI_LOSS_NO_STATEMENT"
                    } else {
                        "WARN_K1_QBI_NO_STATEMENT"
                    },
                    alloc::vec![crate::interview::income_display_arg(entry)],
                ),
                context.clone(),
            );
            if data.qbi_records.history.is_none()
                && [
                    data.regular.basis.prior_loss,
                    data.regular.risk.prior_loss,
                    data.regular.passive_loss,
                ]
                .iter()
                .any(|value| *value != 0)
            {
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_PARTNERSHIP_QBI_HISTORY_LEFT_OUT",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    context.clone(),
                );
            }
            match crate::partnership_qbi::historical_without_statement_with_pool(
                data,
                partner.suspended,
                crate::community::partner_pool(d, entry.id, data, &data.regular).ok().as_ref(),
            ) {
                Ok(Some(prepared)) => {
                    if data.publicly_traded {
                        let released =
                            crate::kernel::ratio::exact::whole(prepared.historical.get());
                        ptp_qualified -= &released;
                        injured_ptp[usize::from(entry.owner == crate::DocumentOwner::Spouses)] -=
                            &released;
                        if let Some(financial_net) = ptp_native_net
                            && (financial_net < 0 || prepared.historical.get() > 0)
                        {
                            ptp_loss_sources.push((entry.id, entry.owner, -released,
                                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::default(),
                                prepared.historical, financial_net));
                        }
                    } else if prepared.historical.get() > 0 {
                        community_qbi_loss_sources.insert(entry.id);
                        released_qbi.push(activities.len());
                        research_qbi_ids.insert(activities.len(), entry.id);
                        activities.push(crate::business_rules::QbiActivity {
                            source_wages_cents: Some(0),
                            source_property_cents: Some(0),
                            source_income_cents: Some(-prepared.historical.get()),
                            owner: Some(entry.owner.owned()),
                            income: -dollars(prepared.historical.get()),
                            wages: 0,
                            property: 0,
                            active: false,
                            service: Some(false),
                            group: None,
                        });
                    }
                    for (key, carry) in [
                        "NOTE_PARTNERSHIP_BASIS_LOSS_CARRY",
                        "NOTE_PARTNERSHIP_AT_RISK_LOSS_CARRY",
                        "NOTE_PARTNERSHIP_PASSIVE_LOSS_CARRY",
                    ]
                    .into_iter()
                    .zip(prepared.carries)
                    {
                        for year in carry.remaining.iter().filter(|value| value.remaining.get() > 0)
                        {
                            note_once_scoped(
                                r,
                                Message::new(
                                    key,
                                    alloc::vec![
                                        crate::interview::income_display_arg(entry),
                                        MessageArg::Number(year.year as i64),
                                        MessageArg::usd_cents(year.remaining.get()),
                                        MessageArg::usd_cents(year.remaining_qbi.get()),
                                        MessageArg::usd_cents(year.original.get()),
                                        MessageArg::usd_cents(year.original_qbi.get()),
                                    ],
                                ),
                                context.clone(),
                            );
                        }
                    }
                    note_once_scoped(
                        r,
                        Message::new(
                            "NOTE_PARTNERSHIP_HISTORICAL_QBI_RELEASE",
                            alloc::vec![
                                crate::interview::income_display_arg(entry),
                                MessageArg::usd_cents(prepared.historical.get()),
                            ],
                        ),
                        context,
                    );
                }
                Ok(None) => {
                    if let Some(financial_net) = ptp_native_net
                        && financial_net < 0
                        && [
                            data.regular.basis.prior_loss,
                            data.regular.risk.prior_loss,
                            data.regular.passive_loss,
                        ] == [0; 3]
                    {
                        ptp_loss_sources.push((
                            entry.id,
                            entry.owner,
                            crate::kernel::ratio::exact::whole(0),
                            crate::kernel::carry::Amount::default(),
                            crate::kernel::carry::Amount::default(),
                            financial_net,
                        ));
                    }
                }
                Err((key, _message)) => {
                    partner_qbi_complete = false;
                    crate::issue_localized!(
                        r,
                        &if data.publicly_traded { context.manual() } else { context },
                        "Partnership retained QBI losses",
                        key,
                        _message,
                    );
                }
            }
            continue;
        };
        if qbi_gross_unknown {
            continue;
        }
        if data.qbi_records.history.is_none()
            && [
                data.regular.basis.prior_loss,
                data.regular.risk.prior_loss,
                data.regular.passive_loss,
            ]
            .iter()
            .any(|value| *value != 0)
        {
            note_once_scoped(
                r,
                Message::new(
                    "WARN_PARTNERSHIP_QBI_HISTORY_LEFT_OUT",
                    alloc::vec![crate::interview::income_display_arg(entry)],
                ),
                crate::interview::income_issue_context(entry),
            );
            continue;
        }
        let value = Ok::<_, crate::section1231::SourceError>(statement).and_then(|statement| {
            let qualified_deductions = statement
                .qualified_deduction(data, insurance + retirement)?
                + statement.qualified_care_deduction(data, care.source(entry.id))?;
            // Statement A states ordinary QBI before the box 12 section 179
            // deduction; the part Form 4562 allows this year is QBI of this
            // business; a suspended part is not (i8995).
            let section179 = dollars(rental.section179_qbi_cents(d, entry.id));
            let sales = crate::k1_sale_qbi::current(entry, rental, before_qbi, s.filing_status())?;
            sales.report(entry.id, r);
            let property = crate::section1231::partner_property_qbi_items_with_claims(
                d,
                rental,
                entry.id,
                Some((before_qbi, s.filing_status())),
                source_claims,
            )?;
            property.report(entry.id, r);
            statement
                .calculate_combined_with_pool(
                    data,
                    partner.suspended,
                    partner_half + qualified_deductions + section179,
                    (before_qbi, s.filing_status()),
                    &sales,
                    &property,
                    crate::community::partner_pool(d, entry.id, data, &data.regular).ok().as_ref(),
                )
                .map(|prepared| (prepared, qualified_deductions))
                .map_err(Into::into)
        });
        match value {
            Ok((prepared, qualified_deductions)) => {
                // Read the whole prepared source before its activity/carries move.
                if let Some(financial_net) = ptp_native_net
                    && (financial_net < 0 || prepared.ptp < crate::kernel::ratio::exact::whole(0))
                {
                    let current = match prepared.ptp_current_for_excess() {
                        Ok(current) => current,
                        Err((key, _message)) => {
                            partner_qbi_complete = false;
                            crate::issue_localized!(
                                r,
                                &crate::interview::income_issue_context(entry),
                                "PTP qualified excess loss",
                                key,
                                _message
                            );
                            continue;
                        }
                    };
                    ptp_loss_sources.push((
                        entry.id,
                        entry.owner,
                        prepared.ptp.clone(),
                        current,
                        prepared.historical_cents,
                        financial_net,
                    ));
                }
                let mut activity = prepared.current;
                activity.group = data.qbi_aggregation_group.clone();
                if !data.publicly_traded && activity.income - prepared.historical < 0 {
                    community_qbi_loss_sources.insert(entry.id);
                }
                if !data.publicly_traded && prepared.historical_cents.get() > 0 {
                    released_qbi.push(activities.len());
                    research_qbi_ids.insert(activities.len(), entry.id);
                    activities.push(crate::business_rules::QbiActivity {
                        source_wages_cents: Some(0),
                        source_property_cents: Some(0),
                        source_income_cents: Some(-prepared.historical_cents.get()),
                        owner: Some(entry.owner.owned()),
                        income: -prepared.historical,
                        wages: 0,
                        property: 0,
                        active: false,
                        service: Some(false),
                        group: None,
                    });
                }
                for ((_stage, key), carry) in [
                    (crate::fixture_english!("basis"), "NOTE_PARTNERSHIP_BASIS_LOSS_CARRY"),
                    (crate::fixture_english!("at-risk"), "NOTE_PARTNERSHIP_AT_RISK_LOSS_CARRY"),
                    (crate::fixture_english!("passive"), "NOTE_PARTNERSHIP_PASSIVE_LOSS_CARRY"),
                ]
                .into_iter()
                .zip(prepared.carries)
                {
                    for year in carry.remaining.iter().filter(|v| v.remaining.get() > 0) {
                        let message = Message::new(
                            key,
                            alloc::vec![
                                MessageArg::Number(entry.id),
                                MessageArg::Number(year.year as i64),
                                MessageArg::usd_cents(year.remaining.get()),
                                MessageArg::usd_cents(year.remaining_qbi.get()),
                                MessageArg::usd_cents(year.original.get()),
                                MessageArg::usd_cents(year.original_qbi.get()),
                            ],
                        );
                        r.review_notes.push(ReviewNote::localized(
                            #[cfg(test)]
                            message.english(),
                            message,
                        ));
                    }
                }
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("Partnership document {}, released historical qualified loss: ${}. Retains loss-year eligibility and enters the QBI engine separately.", entry.id, prepared.historical),
                    Message::new("NOTE_PARTNERSHIP_HISTORICAL_QBI_RELEASE", alloc::vec![
                        MessageArg::Number(entry.id), MessageArg::usd_dollars(prepared.historical),
                    ]),
                ));
                // The aggregate began with all owners' half-SE deductions;
                // this activity already deducts its own share once.
                if data.publicly_traded {
                    ptp_qualified += &prepared.ptp;
                    injured_ptp[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                        &prepared.ptp;
                    // A source note is a display projection, never an operand
                    // of category eligibility, netting or carry calculation.
                    let displayed_ptp = crate::kernel::ratio::exact::dollars(&prepared.ptp)
                        .expect("bounded PTP source display");
                    r.review_notes.push(ReviewNote::localized(
                        #[cfg(test)] alloc::format!("Partnership document {}, qualified publicly traded partnership income or loss: ${}. Combines with qualified REIT dividends without a wage/UBIA limit or aggregation (Form 8995 line 6 / Form 8995-A line 28); a net loss carries forward with that combined balance, not the ordinary QBI business-loss carryforward.", entry.id, displayed_ptp),
                        Message::new("NOTE_PARTNERSHIP_PTP_QBI", alloc::vec![MessageArg::Number(entry.id), MessageArg::usd_dollars(displayed_ptp)]),
                    ));
                } else {
                    r.review_notes.push(ReviewNote::localized(
                        #[cfg(test)] alloc::format!("Partnership document {}, current QBI after allocated half-SE deduction ${partner_half} and SSTB eligibility: ${}; allocated wages ${}; qualified-property basis ${}. Health-insurance deduction ${insurance} and retirement deduction ${retirement} contribute a combined qualified deduction of ${qualified_deductions} before SSTB eligibility. Enters the shared QBI deduction engine; other return limits remain separate.", entry.id, activity.income, activity.wages, activity.property),
                        Message::new("NOTE_PARTNERSHIP_CURRENT_QBI", alloc::vec![
                            MessageArg::Number(entry.id), MessageArg::usd_dollars(partner_half), MessageArg::usd_dollars(activity.income),
                            MessageArg::usd_dollars(activity.wages), MessageArg::usd_dollars(activity.property),
                            MessageArg::usd_dollars(insurance), MessageArg::usd_dollars(retirement),
                            MessageArg::usd_dollars(qualified_deductions),
                        ]),
                    ));
                }
                research_qbi_ids.insert(activities.len(), entry.id);
                activities.push(activity.with_owner(entry.owner.owned()));
            }
            Err(error) => {
                partner_qbi_complete = false;
                error.report_at(
                    r,
                    &crate::interview::income_issue_context(entry),
                    #[cfg(test)]
                    "Partnership QBI statement",
                );
                if error.is_required_source() {
                    return core::ops::ControlFlow::Break(());
                }
            }
        }
    }
    core::ops::ControlFlow::Continue(PartnershipQbiSourcesOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        released_qbi,
        ptp_qualified,
        injured_ptp,
        ptp_loss_sources,
        partner_qbi_complete,
    })
}

pub(super) struct ScorpEstateQbiSourcesIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) before_qbi: i64,
    pub(super) partner_qbi_complete: bool,
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) community_qbi_loss_sources: alloc::collections::BTreeSet<i64>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
}

pub(super) struct ScorpEstateQbiSourcesOut {
    pub(super) activities: Vec<crate::business_rules::QbiActivity>,
    pub(super) community_qbi_loss_sources: alloc::collections::BTreeSet<i64>,
    pub(super) research_qbi_ids: alloc::collections::BTreeMap<usize, i64>,
    pub(super) released_qbi: Vec<usize>,
    pub(super) ptp_qualified: crate::kernel::ratio::exact::Exact,
    pub(super) injured_ptp: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) partner_qbi_complete: bool,
}

pub(super) fn scorp_estate_qbi_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: ScorpEstateQbiSourcesIn<'_>,
) -> core::ops::ControlFlow<(), ScorpEstateQbiSourcesOut> {
    let &Pass { d, s, allocated_entries, context, .. } = pass;
    let source_claims = context.source_claims;
    let ScorpEstateQbiSourcesIn {
        rental,
        health,
        property_character,
        before_qbi,
        mut partner_qbi_complete,
        mut activities,
        mut community_qbi_loss_sources,
        mut research_qbi_ids,
        mut released_qbi,
        mut ptp_qualified,
        mut injured_ptp,
    } = input;
    for &(id, suspended) in &rental.scorp_suspensions {
        if let Some(entry) = allocated_entries.iter().find(|e| e.id == id)
            && let FederalIncome::Scorp { data: source } = &entry.income
        {
            let source = &crate::ScorpK1::from(source);
            let Some(statement) = &source.qbi_statement else {
                match source.historical_qbi_activities_with_events(
                    [suspended, crate::kernel::carry::Amount::default()],
                    None,
                    crate::community::scorp_events(d, id, false).ok().flatten().as_ref(),
                ) {
                    Ok(additions) => {
                        if additions.iter().map(|value| value.income).sum::<i64>() < 0 {
                            community_qbi_loss_sources.insert(id);
                        }
                        released_qbi.extend(activities.len()..activities.len() + additions.len());
                        research_qbi_ids.extend(
                            (activities.len()..activities.len() + additions.len())
                                .map(|i| (i, entry.id)),
                        );
                        activities.extend(
                            additions.into_iter().map(|v| v.with_owner(entry.owner.owned())),
                        );
                    }
                    Err((key, _message)) => {
                        partner_qbi_complete = false;
                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(entry),
                            "S-corporation historical QBI",
                            key,
                            _message,
                        );
                    }
                }
                continue;
            };
            if let Some(key) = source.qbi_left_out() {
                note_once_scoped(
                    r,
                    Message::new(key, alloc::vec![crate::interview::income_display_arg(entry)]),
                    crate::interview::income_issue_context(entry),
                );
                continue;
            }
            match statement.prepare(source, before_qbi, s.filing_status()).and_then(|prepared| {
                let sales =
                    crate::k1_sale_qbi::current(entry, rental, before_qbi, s.filing_status())?;
                sales.report(id, r);
                let eligible = statement.combined_with_sales_with_events(
                    source,
                    [suspended, crate::kernel::carry::Amount::default()],
                    crate::section1231::PropertyCharacter::Capital,
                    (Some(before_qbi), s.filing_status()),
                    dollars(rental.section179_qbi_cents(d, id)),
                    &sales,
                    health.by_business.get(&id).copied().unwrap_or(0),
                    crate::community::scorp_events(d, id, false)?.as_ref(),
                )?;
                Ok((prepared, eligible))
            }) {
                Ok((prepared, additions)) => {
                    let net = additions.iter().map(|v| v.income).sum::<i64>();
                    if net < 0 {
                        community_qbi_loss_sources.insert(id);
                    }
                    r.review_notes.push(ReviewNote::localized(
                        #[cfg(test)] alloc::format!("S-corporation document {id}, QBI income less released historical QBI losses: ${net}. Current business income reflects current SSTB eligibility; historical qualified losses retain their loss-year character. They enter the QBI deduction engine separately; other return limits apply separately."),
                        Message::new("NOTE_SCORP_QBI_AFTER_HISTORICAL", alloc::vec![MessageArg::Number(id), MessageArg::usd_dollars(net)]),
                    ));
                    if prepared.ordinary < 0 {
                        let qbi = &prepared.loss_qbi;
                        if let Ok(basis) = prepared.basis_result() {
                            qbi.report(&prepared, &basis, id, r);
                        }
                        qbi.report_passive(&prepared, id, suspended, r);
                    }
                    // The first activity is current; prior-year qualification stays
                    // in its existing separate released-loss destination.
                    released_qbi.extend(activities.len() + 1..activities.len() + additions.len());
                    research_qbi_ids.extend(
                        (activities.len()..activities.len() + additions.len())
                            .map(|i| (i, entry.id)),
                    );
                    activities.extend(additions.into_iter().enumerate().map(|(i, mut v)| {
                        if i == 0 {
                            v.group = source.qbi_aggregation_group.clone();
                        }
                        v.with_owner(entry.owner.owned())
                    }));
                }
                Err((key, _message)) => {
                    partner_qbi_complete = false;
                    crate::issue_localized!(
                        r,
                        &crate::interview::income_issue_context(entry),
                        "S-corporation QBI statement",
                        key,
                        _message,
                    );
                }
            }
        }
    }
    if let Some(character) = property_character {
        for property in &rental.k1_section1231 {
            if let Some(entry) = allocated_entries.iter().find(|entry| entry.id == property.id)
                && let FederalIncome::Scorp { data: source } = &entry.income
                && let Some((_, ordinary_suspended)) =
                    rental.scorp_qbi_ordinary_suspensions.iter().find(|(id, _)| *id == property.id)
                && crate::ScorpK1::from(source).qbi_left_out().is_none()
                && (crate::ScorpK1::from(source).qbi_statement.is_some()
                    || crate::ScorpK1::from(source).loss_tracking()
                    || crate::ScorpK1::from(source).property_tracking())
            {
                let source = &crate::ScorpK1::from(source);
                let values = crate::section1231::component_character_with_claims(
                    d,
                    rental,
                    character,
                    property.id,
                    false,
                    0,
                    source_claims,
                )
                .and_then(|treatment| {
                    let packet =
                        rental.post_pal.as_ref().filter(|packet| !packet.alternative).ok_or((
                            crate::k1_activity::ordinary::UNAVAILABLE.0.key,
                            crate::k1_activity::ordinary::UNAVAILABLE.1,
                        ))?;
                    let owner = packet
                        .sources
                        .iter()
                        .find(|value| {
                            value.income == entry.id
                                && value.owner == entry.owner.owned()
                                && value.component
                                    == crate::extended_rules::PostPalComponent::ScorpBusiness
                                && value.common_pool
                        })
                        .ok_or((
                            crate::k1_activity::ordinary::UNAVAILABLE.0.key,
                            crate::k1_activity::ordinary::UNAVAILABLE.1,
                        ))?;
                    let held = owner
                        .characters
                        .iter()
                        .find(|value| {
                            value.character == crate::extended_rules::PostPalCharacter::Section1231
                        })
                        .ok_or((
                            crate::k1_activity::ordinary::UNAVAILABLE.0.key,
                            crate::k1_activity::ordinary::UNAVAILABLE.1,
                        ))?;
                    let property_held =
                        crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                            held.suspended_cents,
                        )
                        .expect("source property hold is nonnegative");
                    if let Some(statement) = &source.qbi_statement {
                        let sales = crate::k1_sale_qbi::current(
                            entry,
                            rental,
                            before_qbi,
                            s.filing_status(),
                        )?;
                        sales.report(property.id, r);
                        statement
                            .combined_with_sales_with_events(
                                source,
                                [*ordinary_suspended, property_held],
                                treatment,
                                (Some(before_qbi), s.filing_status()),
                                dollars(rental.section179_qbi_cents(d, property.id)),
                                &sales,
                                health.by_business.get(&property.id).copied().unwrap_or(0),
                                crate::community::scorp_events(d, property.id, false)?.as_ref(),
                            )
                            .map_err(Into::into)
                    } else {
                        source
                            .historical_qbi_activities_with_events(
                                [*ordinary_suspended, property_held],
                                Some(treatment),
                                crate::community::scorp_events(d, property.id, false)?.as_ref(),
                            )
                            .map_err(Into::into)
                    }
                });
                match values {
                    Ok(values) => {
                        if values.iter().map(|value| value.income).sum::<i64>() < 0 {
                            community_qbi_loss_sources.insert(property.id);
                        }
                        if source.qbi_statement.is_some() {
                            let current = &values[0];
                            let historical = -values.iter().skip(1).map(|v| v.income).sum::<i64>();
                            r.review_notes.push(ReviewNote::localized(
                            #[cfg(test)] alloc::format!("S-corporation document {}, combined business QBI after eligibility: current ${}; historical qualified losses ${historical}; allocated wages ${}; qualified-property basis ${}. One business wage/property limit applies using taxable income before QBI ${before_qbi}. Final totals require all applicable return checks.", property.id, current.income, current.wages, current.property),
                            Message::new("NOTE_SCORP_COMBINED_BUSINESS_QBI", alloc::vec![
                                MessageArg::Number(property.id), MessageArg::usd_dollars(current.income), MessageArg::usd_dollars(historical),
                                MessageArg::usd_dollars(current.wages), MessageArg::usd_dollars(current.property), MessageArg::usd_dollars(before_qbi),
                            ]),
                            ));
                        }
                        let first_released = usize::from(source.qbi_statement.is_some());
                        released_qbi.extend(
                            activities.len() + first_released..activities.len() + values.len(),
                        );
                        research_qbi_ids.extend(
                            (activities.len()..activities.len() + values.len())
                                .map(|i| (i, property.id)),
                        );
                        activities.extend(values.into_iter().enumerate().map(|(i, mut v)| {
                            if i == 0 && source.qbi_statement.is_some() {
                                v.group = source.qbi_aggregation_group.clone();
                            }
                            v.with_owner(entry.owner.owned())
                        }));
                    }
                    Err(error) => {
                        error.report_at(
                            r,
                            &crate::interview::income_issue_context(entry),
                            #[cfg(test)]
                            "Combined S-corporation QBI",
                        );
                        if error.is_required_source() {
                            return core::ops::ControlFlow::Break(());
                        }
                    }
                }
            }
        }
    }
    for entry in allocated_entries {
        let FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        if !rental.inputs_complete {
            continue;
        }
        for statement in &data.qbi.statements {
            let context = crate::interview::income_issue_context(entry);
            if statement.kind == crate::EstateQbiKind::Cooperative {
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_ESTATE_QBI_COOPERATIVE_LEFT_OUT",
                        alloc::vec![MessageArg::Text(statement.name.clone())],
                    ),
                    context,
                );
                continue;
            }
            match crate::estate_trust::qbi_projection(
                entry.id,
                data,
                statement,
                &rental.estate_activities,
                before_qbi,
                s.filing_status(),
            ) {
                Ok(projected) => {
                    if projected.current.income - projected.historical < 0 {
                        community_qbi_loss_sources.insert(entry.id);
                    }
                    if projected.history_missing {
                        note_once_scoped(
                            r,
                            Message::new(
                                "WARN_ESTATE_QBI_HISTORY_LEFT_OUT",
                                alloc::vec![MessageArg::Text(statement.name.clone())],
                            ),
                            context.clone(),
                        );
                    }
                    if statement.kind == crate::EstateQbiKind::PubliclyTradedPartnership {
                        // A linked fiduciary activity is not a reconstructed PTP
                        // loss pool. Unlinked allocated PTP QBI is already the
                        // beneficiary's own post-entity statement amount.
                        if statement.activities.is_empty() {
                            // This owner's established output is already
                            // annual Dollars. Lift that reported amount to the
                            // category's cent unit; do not reconstruct source
                            // cents or apply current eligibility again.
                            let reported_cents =
                                crate::kernel::ratio::exact::whole(projected.current.income)
                                    * crate::kernel::ratio::exact::whole(100);
                            ptp_qualified += &reported_cents;
                            injured_ptp
                                [usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                                reported_cents;
                        } else {
                            note_once_scoped(
                                r,
                                Message::new(
                                    "WARN_ESTATE_QBI_SOURCE_LEFT_OUT",
                                    alloc::vec![MessageArg::Text(statement.name.clone())],
                                ),
                                context.clone(),
                            );
                        }
                    } else {
                        activities.push(projected.current.with_owner(entry.owner.owned()));
                        if projected.historical > 0 {
                            released_qbi.push(activities.len());
                            activities.push(crate::business_rules::QbiActivity {
                                source_wages_cents: None,
                                source_property_cents: None,
                                source_income_cents: None,
                                owner: Some(entry.owner.owned()),
                                income: -projected.historical,
                                wages: 0,
                                property: 0,
                                active: false,
                                service: Some(false),
                                group: None,
                            });
                        }
                    }
                    for year in projected.remaining.iter().filter(|v| v.remaining.get() > 0) {
                        note_once_scoped(
                            r,
                            Message::new(
                                "NOTE_ESTATE_QBI_PASSIVE_CARRY",
                                alloc::vec![
                                    MessageArg::Text(statement.name.clone()),
                                    MessageArg::Number(year.year as i64),
                                    MessageArg::usd_dollars(year.remaining.get()),
                                    MessageArg::usd_dollars(year.remaining_qbi.get())
                                ],
                            ),
                            context.clone(),
                        );
                    }
                }
                Err(key) => note_once_scoped(
                    r,
                    Message::new(key, alloc::vec![MessageArg::Text(statement.name.clone())]),
                    context,
                ),
            }
        }
    }
    core::ops::ControlFlow::Continue(ScorpEstateQbiSourcesOut {
        activities,
        community_qbi_loss_sources,
        research_qbi_ids,
        released_qbi,
        ptp_qualified,
        injured_ptp,
        partner_qbi_complete,
    })
}

pub(super) struct QbiCarryReportingIn<'a> {
    pub(super) qbi: &'a crate::business_rules::QbiResult,
    pub(super) qbi_carry: i64,
    pub(super) qbi_gross_unknown: bool,
    pub(super) ptp_nol_complete: bool,
}

pub(super) fn qbi_carry_reporting_phase(
    r: &mut FederalResult,
    input: QbiCarryReportingIn<'_>,
    qbi_line: impl Fn(&str, &str) -> Option<FormLine>,
) {
    let QbiCarryReportingIn { qbi, qbi_carry, qbi_gross_unknown, ptp_nol_complete } = input;
    if !qbi_gross_unknown && ptp_nol_complete {
        push_line(
            r,
            qbi_line("19", "42"),
            "reit_ptp_loss_carryforward",
            qbi.reit_loss * 100,
            &["income.dividends", "screening.qbi_history"],
            "https://www.irs.gov/instructions/i8995",
            msg!("EXPLAIN_REIT_PTP_LOSS_CARRYFORWARD"),
        );
    }
    if !qbi_gross_unknown {
        push_line(
            r,
            qbi_line("18", "Schedule C 6"),
            "qbi_loss_carryforward",
            qbi_carry * 100,
            &["income", "rental_income"],
            "https://www.irs.gov/instructions/i8995a",
            msg!("EXPLAIN_QBI_LOSS_CARRYFORWARD"),
        );
    }
}
