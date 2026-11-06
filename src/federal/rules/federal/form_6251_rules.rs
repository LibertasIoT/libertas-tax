// form_6251 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// These are classifications of income already included through the activity
// owner, not additional AMTI adjustments. Passive activities take precedence
// over the basis/at-risk line (Form 6251 lines 2d, 2m and 2n).
// https://www.irs.gov/instructions/i6251
fn k1_amt_lines(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    regular: &crate::extended_rules::RentalResult,
    alternative: &crate::extended_rules::RentalResult,
) -> [Option<i64>; 3] {
    use crate::extended_rules::{K1AmtActivityAmounts, K1AmtActivitySource};
    let complete = regular.inputs_complete
        && alternative.inputs_complete
        && regular.post_pal.as_ref().is_some_and(|source| !source.alternative)
        && alternative.post_pal.as_ref().is_some_and(|source| source.alternative);
    let mut amounts = if complete { [Some(0i64); 3] } else { [None; 3] };
    let mut compare = |id,
                       regular: Option<&K1AmtActivityAmounts>,
                       alternative: Option<&K1AmtActivityAmounts>| {
        let delta = regular.zip(alternative).and_then(|(regular, alternative)| {
            let left = &regular.source;
            let right = &alternative.source;
            let category = if left.passive || right.passive {
                1
            } else if left.loss_limited || right.loss_limited {
                2
            } else {
                0
            };
            let delta = if category == 0 {
                match (left.depletion_cents, right.depletion_cents) {
                    (Some(0), Some(0)) => Some(0),
                    (Some(cents), Some(other)) if cents == other => alternative
                        .schedule_e_cents
                        .checked_sub(regular.schedule_e_cents)
                        .filter(|delta| *delta == cents)
                        .and_then(|_| {
                            alternative.schedule_e_dollars.checked_sub(regular.schedule_e_dollars)
                        }),
                    _ => None,
                }
            } else {
                alternative.schedule_e_dollars.checked_sub(regular.schedule_e_dollars)
            };
            amounts[category] =
                amounts[category].zip(delta).and_then(|(sum, delta)| sum.checked_add(delta));
            amounts[category]
        });
        if delta.is_none() {
            if regular.is_none() || alternative.is_none() {
                amounts = [None; 3];
            }
            if let Some(entry) = pass.allocated_entries.iter().find(|entry| entry.id == id) {
                note_once_scoped(
                    r,
                    msg!("ISSUE_K1_AMT_LINE_CLASSIFICATION_UNAVAILABLE"),
                    crate::interview::income_issue_context(entry)
                        .corrective(crate::TaxSection::PassThrough),
                );
            }
        }
    };
    if let (Some(left), Some(right)) = (&regular.post_pal, &alternative.post_pal)
        && !left.alternative
        && right.alternative
    {
        for source in &left.sources {
            let Some(regular) = &source.k1_amt else { continue };
            let mut matches = right.sources.iter().filter(|other| {
                other.income == source.income && other.component == source.component
            });
            let alternative = matches.next().and_then(|other| other.k1_amt.as_ref());
            if let Some(alternative) = alternative.filter(|_| matches.next().is_none()) {
                compare(source.income, Some(regular), Some(alternative));
            } else {
                compare(source.income, Some(regular), None);
            }
        }
        for source in &right.sources {
            let Some(alternative) = &source.k1_amt else { continue };
            if !left.sources.iter().any(|other| {
                other.income == source.income
                    && other.component == source.component
                    && other.k1_amt.is_some()
            }) {
                compare(source.income, None, Some(alternative));
            }
        }
    }
    // PTPs retain their own native passive owner, outside the common source
    // packet. A sole ordinary source uses its actual retained K-1 endpoint.
    for entry in pass.allocated_entries {
        let FederalIncome::Partnership { data } = &entry.income else { continue };
        if !data.publicly_traded {
            continue;
        }
        let Some(cents) = data.source.as_ref().and_then(|source| source.amt_depletion_adjustment())
        else {
            continue;
        };
        if cents == 0 {
            continue;
        }
        let endpoint = |result: &crate::extended_rules::RentalResult| {
            let mut values = result.k1_amounts.iter().filter(|source| source.id == entry.id);
            let value = values.next()?;
            if values.next().is_some() {
                return None;
            }
            let (_, ledger) = result.k1_ledgers.iter().find(|(id, _)| *id == entry.id)?;
            let ordinary = crate::k1_loss::K1LossClass::Ordinary;
            Some(K1AmtActivityAmounts {
                source: K1AmtActivitySource {
                    passive: data.material_for_losses() != crate::participation::Participation::Yes
                        || result.partners.iter().find(|source| source.id == entry.id).is_some_and(
                            |source| source.suspended.get() != 0 || source.prior_released != 0,
                        ),
                    loss_limited: ledger.basis_suspended[ordinary].get() != 0
                        || ledger.risk_suspended[ordinary].get() != 0,
                    depletion_cents: Some(cents),
                },
                schedule_e_cents: value.source_net_cents?,
                schedule_e_dollars: value.net,
            })
        };
        let left = endpoint(regular);
        let right = endpoint(alternative);
        compare(entry.id, left.as_ref(), right.as_ref());
    }
    amounts
}

pub(super) struct AmtAdjustmentsIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_health: &'a crate::health::HealthResult,
    pub(super) amt_property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) amt_direct_property_sources: &'a [(i64, crate::community::PropertyCents)],
    pub(super) amt_activity_issues: Vec<crate::FederalIssue>,
    pub(super) additional: [i64; 3],
    pub(super) royalty_interest: [i64; 2],
    pub(super) portfolio_interest: [i64; 2],
    pub(super) use_itemized: bool,
    pub(super) tax_refund: i64,
    pub(super) salt: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) foreign_tax_deductible: i64,
    pub(super) standard: i64,
    pub(super) agi: i64,
    pub(super) k1_amt_interest_adjustment: i64,
    pub(super) k1_amt_dividend_adjustment: i64,
    pub(super) amt_mortgage: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) loan_gain: i64,
    pub(super) amt_capital_independent_adjustment: i64,
    pub(super) unlimited_deduction: i64,
    pub(super) qbi_ded: i64,
    pub(super) iso_amt_wage_adjustment: i64,
    pub(super) amt_related_adjustment: i64,
    pub(super) medical_deduction: i64,
    pub(super) amt_medical_deduction: i64,
    pub(super) amt_property_ordinary: i64,
    pub(super) property_ordinary: i64,
    pub(super) business: i64,
    pub(super) half_se: i64,
    pub(super) amt_sep_deduction: i64,
    pub(super) business_excluded: i64,
    pub(super) amt_property_business_income: i64,
    pub(super) amt_direct_property_ordinary: i64,
    pub(super) excess_business_loss: i64,
    pub(super) business_depletion_adjustment: i64,
}

pub(super) struct AmtAdjustmentsOut {
    pub(super) iso: i64,
    pub(super) amt_tax_recovery: i64,
    pub(super) amt_base: i64,
    pub(super) amt_rental_adjustment: i64,
    pub(super) amt_excess_business_loss: i64,
    pub(super) k1_depletion_adjustment: Option<i64>,
    pub(super) amt_property_loss_deferred: Result<i64, (&'static str, &'static str)>,
}

pub(super) fn amt_adjustments_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    input: AmtAdjustmentsIn<'_>,
) -> AmtAdjustmentsOut {
    let &Pass { d, allocated_entries, j, context, .. } = pass;
    let source_claims = context.source_claims;
    let AmtAdjustmentsIn {
        rental,
        amt_rental,
        amt_health,
        amt_property_character,
        amt_direct_property_sources,
        amt_activity_issues,
        additional,
        royalty_interest,
        portfolio_interest,
        use_itemized,
        tax_refund,
        salt,
        investment_property_taxes,
        foreign_tax_deductible,
        standard,
        agi,
        k1_amt_interest_adjustment,
        k1_amt_dividend_adjustment,
        amt_mortgage,
        amt_loan_gain,
        loan_gain,
        amt_capital_independent_adjustment,
        unlimited_deduction,
        qbi_ded,
        iso_amt_wage_adjustment,
        amt_related_adjustment,
        medical_deduction,
        amt_medical_deduction,
        amt_property_ordinary,
        property_ordinary,
        business,
        half_se,
        amt_sep_deduction,
        business_excluded,
        amt_property_business_income,
        amt_direct_property_ordinary,
        excess_business_loss,
        business_depletion_adjustment,
    } = input;
    let iso = crate::extended_rules::iso_adjustment(d, r);
    // Form 6251 line 2b: the refunds of state or local income tax on Schedule 1 line 1 and of
    // real estate or personal property tax on line 8z (section 56(b)(1)(C)).
    let amt_tax_recovery = tax_refund + dollars(t.recovery_8z_taxes);
    form_line(
        r,
        FormLine::new(TaxForm::Form6251, "2a"),
        "amt_itemized_taxes",
        if use_itemized {
            salt + investment_property_taxes + foreign_tax_deductible
        } else {
            standard
        },
        &["deductions"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_ITEMIZED_TAXES"),
    );
    let amt_base = agi
        + k1_amt_interest_adjustment
        + k1_amt_dividend_adjustment
        + if use_itemized { amt_mortgage } else { 0 }
        + nol_run.deduction
        + amt_loan_gain
        - loan_gain
        + amt_capital_independent_adjustment
        - amt_tax_recovery
        - unlimited_deduction
        - additional.iter().sum::<i64>()
        - qbi_ded
        + if use_itemized {
            salt + investment_property_taxes + foreign_tax_deductible
        } else {
            standard
        }
        + dollars(t.private_activity)
        + dollars(t.estate_amt_box12a - t.estate_amt_reallocated)
        + dollars(t.estate_amt_independent)
        + dollars(t.qsbs_amt_preference)
        + iso
        + iso_amt_wage_adjustment
        + amt_related_adjustment
        + royalty_interest[0]
        - royalty_interest[1]
        + if use_itemized { medical_deduction - amt_medical_deduction } else { 0 }
        + if use_itemized { portfolio_interest[0] - portfolio_interest[1] } else { 0 };
    // QBI remains the regular-tax deduction under 26 CFR 1.199A-1(e)(5).
    // The passive worksheet uses regular MAGI under 1.55-1(b), but AMT
    // depreciation and activity loss allocations are computed separately.
    r.issues.extend(amt_activity_issues);
    let [k1_depletion_adjustment, passive, loss_limits] = k1_amt_lines(pass, r, rental, amt_rental);
    for (printed, id, amount, explanation) in [
        ("2m", "form6251_line2m_k1_passive", passive, msg!("EXPLAIN_K1_AMT_PASSIVE_ADJUSTMENT")),
        (
            "2n",
            "form6251_line2n_k1_loss_limits",
            loss_limits,
            msg!("EXPLAIN_K1_AMT_LOSS_LIMIT_ADJUSTMENT"),
        ),
    ] {
        if let Some(amount) = amount.filter(|amount| *amount != 0) {
            form_line(
                r,
                FormLine::new(TaxForm::Form6251, printed),
                id,
                amount,
                &["income.partnership", "income.scorp"],
                "https://www.irs.gov/instructions/i6251",
                explanation,
            );
        }
    }
    let amt_rental_adjustment = amt_rental.income - rental.income + amt_property_ordinary
        - property_ordinary
        + amt_rental.personal_expense_adjustment
        - rental.personal_expense_adjustment
        + dollars(t.amt_royalty_adjustment);
    if allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        && r.issues.is_empty()
    {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("K-1 AMT activity worksheet: AMT ordinary net ${}; regular ordinary net ${}; income adjustment ${}. Each amount uses its own basis, at-risk and passive balances. QBI remains the regular-tax deduction.", amt_rental.scorp_business, rental.scorp_business, amt_rental.scorp_business - rental.scorp_business),
            Message::new("NOTE_K1_AMT_ACTIVITY", alloc::vec![
                MessageArg::usd_dollars(amt_rental.scorp_business), MessageArg::usd_dollars(rental.scorp_business),
                MessageArg::usd_dollars(amt_rental.scorp_business - rental.scorp_business),
            ]),
        ));
    }
    let amt_business = dollars(
        t.business.iter().sum::<i64>() + t.community_business + t.business_depreciation_adjustment,
    );
    let amt_business_adjustment = amt_business - business;
    let amt_excess_business_loss = excess_business_loss_amount(
        amt_business - half_se - amt_sep_deduction - amt_health.deduction - business_excluded,
        amt_rental,
        amt_property_business_income,
        j,
    )
    .1;
    if amt_rental.inputs_complete && !amt_rental.partners.is_empty() {
        r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
            "Partnership-inclusive AMT excess-business-loss worksheet: ordinary pass-through business net ${}; excess business loss ${amt_excess_business_loss}. Separate AMT basis, risk and passive amounts determine this adjustment.",
            amt_rental.ordinary_pass_through_business(),
        ), Message::new("NOTE_PARTNERSHIP_AMT_EXCESS_BUSINESS_LOSS", alloc::vec![
            MessageArg::usd_dollars(amt_rental.ordinary_pass_through_business()), MessageArg::usd_dollars(amt_excess_business_loss),
        ])));
    }
    let amt_property_loss_deferred = amt_property_character.map_or(Ok(0), |character| {
        crate::section1231::property_excess_allocation(
            d,
            amt_rental,
            character,
            amt_excess_business_loss,
            true,
            amt_business - half_se - amt_sep_deduction - amt_health.deduction - business_excluded
                + amt_direct_property_ordinary,
            amt_direct_property_sources,
        )
    });
    let amt_base =
        amt_base + amt_business_adjustment + amt_rental_adjustment + amt_excess_business_loss
            - excess_business_loss;
    let amt_conservation_adjustment = crate::farm::sources(d)
        .filter(|source| !source.rental)
        .map(|source| {
            crate::farm_conservation::business_amt_adjustment_with_claims(
                d,
                source.entry.id,
                source_claims,
            )
        })
        .sum::<i64>();
    line(
        r,
        "amt_farm_conservation_adjustment",
        dollars(amt_conservation_adjustment),
        &["income.business.activity.farming.conservation"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_FARM_CONSERVATION_ADJUSTMENT"),
    );
    let amt_supplies_adjustment = crate::farm::sources(d)
        .filter(|source| !source.rental)
        .map(|source| {
            crate::farm_supplies::business_amt_adjustment_with_claims(
                d,
                source.entry.id,
                source_claims,
            )
        })
        .sum::<i64>();
    line(
        r,
        "amt_farm_supplies_adjustment",
        dollars(amt_supplies_adjustment),
        &["income.business.activity.farming.supplies"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_FARM_SUPPLIES_ADJUSTMENT"),
    );
    let amt_production_adjustment = crate::farm::sources(d)
        .filter(|source| !source.rental)
        .map(|source| crate::farm_production::business_amt_adjustment(d, source.entry.id))
        .sum::<i64>();
    line(
        r,
        "amt_farm_production_adjustment",
        dollars(amt_production_adjustment),
        &["income.business.activity.farming.specials.production"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_FARM_PRODUCTION_ADJUSTMENT"),
    );
    line(
        r,
        "amt_business_depreciation_adjustment",
        amt_business_adjustment
            - dollars(
                amt_conservation_adjustment
                    + amt_supplies_adjustment
                    + amt_production_adjustment
                    + business_depletion_adjustment,
            ),
        &["income.business.assets"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_BUSINESS_DEPRECIATION_ADJUSTMENT"),
    );
    AmtAdjustmentsOut {
        iso,
        amt_tax_recovery,
        amt_base,
        amt_rental_adjustment,
        amt_excess_business_loss,
        amt_property_loss_deferred,
        k1_depletion_adjustment,
    }
}

pub(super) struct AmtCurrentNolIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) amt_capital_result: &'a crate::capital::CapitalResult,
    pub(super) estate_business_carry: [Option<[i64; 2]>; 2],
    pub(super) royalty_interest: [i64; 2],
    pub(super) portfolio_interest: [i64; 2],
    pub(super) amt_capital_in: [i64; 2],
    pub(super) use_itemized: bool,
    pub(super) prior_capital_source_known: bool,
    pub(super) unlimited_deduction: i64,
    pub(super) salt: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) foreign_tax_deductible: i64,
    pub(super) amt_medical_deduction: i64,
    pub(super) medical_deduction: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) nonbusiness_income: i64,
    pub(super) k1_amt_interest_adjustment: i64,
    pub(super) k1_amt_dividend_adjustment: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) loan_gain: i64,
    pub(super) amt_k1_business_ordinary: i64,
    pub(super) k1_business_ordinary: i64,
    pub(super) amt_tax_recovery: i64,
    pub(super) amt_nonbusiness_capital_net: i64,
    pub(super) nonbusiness_capital_net: i64,
    pub(super) nonbusiness_deductions: i64,
    pub(super) sep_deduction: i64,
    pub(super) amt_sep_deduction: i64,
    pub(super) amt_ira_deduction: i64,
    pub(super) deduction: i64,
    pub(super) regular_business_casualty: i64,
    pub(super) claimed_casualty: i64,
    pub(super) amt_business_capital_net: i64,
    pub(super) amt_base: i64,
    pub(super) qbi_ded: i64,
    pub(super) amt_capital: i64,
    pub(super) nol: i64,
    pub(super) noncash_claim: i64,
    pub(super) amt_excess_business_loss: i64,
    pub(super) amt_rental_adjustment: i64,
}

pub(super) struct AmtCurrentNolOut {
    pub(super) amt_nol: i64,
    pub(super) farm_route: crate::farm::FarmLossRoute,
}

pub(super) fn amt_current_nol_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    partner_pass: &PartnerPass,
    input: AmtCurrentNolIn<'_>,
) -> core::ops::ControlFlow<(), AmtCurrentNolOut> {
    let &Pass { d, ded, c, screen, office_context, context, .. } = pass;
    let AmtCurrentNolIn {
        rental,
        amt_rental,
        ira,
        amt_capital_result,
        estate_business_carry,
        royalty_interest,
        portfolio_interest,
        amt_capital_in,
        use_itemized,
        prior_capital_source_known,
        unlimited_deduction,
        salt,
        investment_property_taxes,
        foreign_tax_deductible,
        amt_medical_deduction,
        medical_deduction,
        qualified_net_casualty,
        nonbusiness_income,
        k1_amt_interest_adjustment,
        k1_amt_dividend_adjustment,
        amt_loan_gain,
        loan_gain,
        amt_k1_business_ordinary,
        k1_business_ordinary,
        amt_tax_recovery,
        amt_nonbusiness_capital_net,
        nonbusiness_capital_net,
        nonbusiness_deductions,
        sep_deduction,
        amt_sep_deduction,
        amt_ira_deduction,
        deduction,
        regular_business_casualty,
        claimed_casualty,
        amt_business_capital_net,
        amt_base,
        qbi_ded,
        amt_capital,
        nol,
        noncash_claim,
        amt_excess_business_loss,
        amt_rental_adjustment,
    } = input;
    let amt_itemized = if use_itemized {
        unlimited_deduction - salt - investment_property_taxes - foreign_tax_deductible
            + portfolio_interest[1]
            - portfolio_interest[0]
            + amt_medical_deduction
            - medical_deduction
    } else {
        qualified_net_casualty
    };
    let amt_nonbusiness_income = nonbusiness_income
        + k1_amt_interest_adjustment
        + k1_amt_dividend_adjustment
        + royalty_interest[0]
        - royalty_interest[1]
        + amt_loan_gain
        - loan_gain
        - amt_k1_business_ordinary
        + k1_business_ordinary
        - amt_tax_recovery
        + positive(amt_nonbusiness_capital_net)
        - positive(nonbusiness_capital_net)
        + dollars(t.private_activity)
        + amt_rental.personal_expense_adjustment
        - rental.personal_expense_adjustment;
    // AMT starts from the pre-section-68 itemized amount. Restore the
    // regular casualty allocation before replacing that deduction, then
    // exclude the full itemized casualty from the AMT nonbusiness pool.
    let amt_nonbusiness_deductions = nonbusiness_deductions - sep_deduction - ira.deduction
        + amt_sep_deduction
        + amt_ira_deduction
        - deduction
        + regular_business_casualty
        + amt_itemized
        - if use_itemized { claimed_casualty } else { 0 };
    crate::require_localized!(
        r,
        prior_capital_source_known
            || amt_capital_in.iter().all(|v| *v == 0)
            || amt_business_capital_net <= 0
            || amt_base + qbi_ded >= 0,
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
        "AMT NOL capital-loss source",
        "ISSUE_PRIOR_AMT_CAPITAL_LOSSES_NEED_THEIR_BUSINESS_NONBUSINESS_SOURCE_ALL_680A28E5",
        "Prior AMT capital losses need their business/nonbusiness source allocation when property gains enter an ATNOL; the regular Form 8960 difference is not that source.",
    );
    let computed_amt_nol = crate::business_rules::current_nol(
        amt_base + qbi_ded,
        crate::business_rules::NolCapital {
            return_capital: amt_capital,
            nonbusiness_capital: amt_nonbusiness_capital_net,
            business_capital: amt_business_capital_net,
            section1202_excluded: nol_run.qsbs_excluded,
            schedule_capital_net: amt_capital_result.short + amt_capital_result.long,
        },
        amt_nonbusiness_deductions,
        amt_nonbusiness_income - positive(amt_nonbusiness_capital_net),
    );
    let amt_nol = if estate_business_carry[1].is_none() && amt_base + qbi_ded < 0 {
        note_once_scoped(
            r,
            msg!("WARN_ESTATE_CAPITAL_ATNOL_LEFT_OUT"),
            crate::IssueContext::Section(crate::TaxSection::PassThrough).manual(),
        );
        0
    } else {
        computed_amt_nol
    };
    crate::require_localized!(
        r,
        ded.charity_carryovers.is_empty() || (nol == 0 && amt_nol == 0),
        &crate::IssueContext::Section(crate::TaxSection::GiftCarryovers),
        "Charitable carryover current NOL",
        "ISSUE_A_2025_PUBLIC_STOCK_CARRYOVER_ALONGSIDE_A_2026_REGULAR_OR_AMT_OPERA_145BA6A2",
        "A charitable carryover alongside a 2026 regular or AMT operating loss requires coordinated Form 172 and charitable-carryover worksheets, even if this return takes the standard deduction.",
    );
    crate::require_localized!(
        r,
        !use_itemized || noncash_claim == 0 || (nol == 0 && amt_nol == 0),
        &crate::IssueContext::Section(crate::TaxSection::GiftCarryovers),
        "Noncash charity NOL",
        "ISSUE_A_CURRENT_REGULAR_OR_AMT_OPERATING_LOSS_WITH_A_NONCASH_CHARITABLE_D_0630C082",
        "A current regular or AMT operating loss with a noncash charitable deduction needs coordinated Form 172 and charitable-carryover worksheets.",
    );
    crate::require_localized!(
        r,
        estate_final_nol(d).is_none() || (nol == 0 && amt_nol == 0),
        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
        "Estate/trust NOL carryover",
        "ISSUE_A_BENEFICIARY_S_CURRENT_REGULAR_OR_AMT_OPERATING_LOSS_NEEDS_A_COMBI_ABC9E203",
        "A beneficiary's current regular or AMT operating loss needs a combined loss-year and source allocation with the final estate/trust K-1 carryover.",
    );
    crate::require_localized!(
        r,
        t.estate_amt_box12a == 0
            || (nol == 0
                && amt_nol == 0
                && !screen.nol.present()
                && !screen.atnol.present()
                && estate_final_nol(d).is_none()),
        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
        "Estate/trust AMT adjustment",
        "ISSUE_A_K_1_BOX_12_CODE_A_ADJUSTMENT_WITH_A_CURRENT_OR_PRIOR_REGULAR_AMT_E802F7A6",
        "A K-1 box 12 code A adjustment with a current or prior regular/AMT net operating loss needs its source-character and loss-absorption worksheets.",
    );
    let farm_route = match crate::farm::check_nol_with_section179_with_claims(
        context.forms(d),
        nol,
        amt_nol,
        [rental.farm_rental * 100, amt_rental.farm_rental * 100],
        partner_pass.section179.as_deref(),
        r,
        Some(office_context),
    ) {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    };
    emit_lines(
        r,
        &mut (),
        [
            (
                "amt_current_net_operating_loss",
                amt_nol,
                msg!("EXPLAIN_AMT_CURRENT_NET_OPERATING_LOSS"),
            ),
            (
                "amt_excess_business_loss_carryforward",
                amt_excess_business_loss,
                msg!("EXPLAIN_AMT_EXCESS_BUSINESS_LOSS_CARRYFORWARD"),
            ),
        ],
        |(id, amount, note), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &[
                    "income",
                    "deductions",
                    "amt_rental_adjustment",
                    "amt_business_depreciation_adjustment",
                ],
                "https://www.irs.gov/instructions/i6251",
                note,
            ))
        },
    );
    line(
        r,
        "amt_rental_adjustment",
        amt_rental_adjustment,
        &["rental_income", "amt_rental_income"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_RENTAL_ADJUSTMENT"),
    );
    crate::require_localized!(
        r,
        !screen.atnol.present() || !matches!(c.foreign_tax, ForeignTaxClaim::Ordinary { .. }),
        &crate::IssueContext::Business,
        "AMT loss history",
        "ISSUE_PRIOR_ATNOL_WITH_FORM_1116_ALLOCATION_REQUIRES_THE_FOREIGN_LOSS_ALL_E8CEA4D5",
        "Prior ATNOL with Form 1116 allocation requires the foreign loss-allocation worksheet. The eligible payee-statement election uses the existing shared regular/AMT credit.",
    );
    core::ops::ControlFlow::Continue(AmtCurrentNolOut { amt_nol, farm_route })
}

pub(super) struct AmtAbsorptionIn<'a> {
    pub(super) k1_depletion_adjustment: Option<i64>,
    pub(super) royalty_regular: &'a [crate::royalties::Depletion],
    pub(super) royalty_amt: &'a [crate::royalties::Depletion],
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) royalty_limits: &'a Option<[i64; 2]>,
    pub(super) royalty_next: &'a mut Option<[i64; 2]>,
    pub(super) kiddie_age: bool,
    pub(super) amt_elected: [i64; 2],
    pub(super) amt_base: i64,
    pub(super) business_depletion_adjustment: i64,
    pub(super) amt_capital: i64,
    pub(super) qbi_ded: i64,
    pub(super) before_qbi_signed: i64,
    pub(super) itemized_reduction: i64,
    pub(super) amt_short: i64,
    pub(super) amt_long: i64,
    pub(super) amt_capital_independent_adjustment: i64,
    pub(super) amt_tax_collectible: i64,
    pub(super) amt_tax_unrecaptured: i64,
    pub(super) amt_tax_pref: i64,
    pub(super) taxable: i64,
    pub(super) excluded: i64,
    pub(super) regular_tax_pref: i64,
    pub(super) regular_collectible: i64,
    pub(super) regular_unrecaptured: i64,
    pub(super) amt_qualified: i64,
    pub(super) amt_tax_net_capital: i64,
    pub(super) amt_schedule_d_18: i64,
    pub(super) amt_schedule_d_19: i64,
}

pub(super) struct AmtAbsorptionOut {
    pub(super) has_atnol: bool,
    pub(super) prior_atnol: alloc::boxed::Box<crate::atnol::AtnolResult>,
    pub(super) amt_base: i64,
    pub(super) amt_computation: crate::income_tax::AmtTaxResult,
    pub(super) tentative: i64,
}

pub(super) fn amt_absorption_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &crate::nol::NolRun,
    input: AmtAbsorptionIn<'_>,
) -> core::ops::ControlFlow<(), AmtAbsorptionOut> {
    let &Pass { d, s, j, p, repayment, .. } = pass;
    let AmtAbsorptionIn {
        k1_depletion_adjustment,
        royalty_regular,
        royalty_amt,
        foreign,
        royalty_limits,
        royalty_next,
        kiddie_age,
        amt_elected,
        amt_base,
        business_depletion_adjustment,
        amt_capital,
        qbi_ded,
        before_qbi_signed,
        itemized_reduction,
        amt_short,
        amt_long,
        amt_capital_independent_adjustment,
        amt_tax_collectible,
        amt_tax_unrecaptured,
        amt_tax_pref,
        taxable,
        excluded,
        regular_tax_pref,
        regular_collectible,
        regular_unrecaptured,
        amt_qualified,
        amt_tax_net_capital,
        amt_schedule_d_18,
        amt_schedule_d_19,
    } = input;
    // Section 172(b)(2)/(d) removes net capital losses and QBI when measuring
    // ATNOL absorption. AGI-based deductions retain regular-tax AGI under
    // 26 CFR 1.55-1(b); prior ATNOLs do not change that reference amount.
    let depletion_adjustment = match exact::dollars(
        &(crate::royalties::total(royalty_regular) - crate::royalties::total(royalty_amt)),
    )
    .filter(|amount| amount.checked_mul(100).is_some())
    {
        Some(amount) => amount,
        None => {
            crate::extended_rules::expense_output_unavailable(r, None);
            return core::ops::ControlFlow::Break(());
        }
    };
    let amt_base = amt_base + depletion_adjustment - dollars(business_depletion_adjustment);
    let amt_modified_income = amt_base + positive(-amt_capital) + qbi_ded + nol_run.qsbs_excluded;
    let projection = crate::estate_trust::losses(d, true);
    let has_atnol = !projection.vintages.is_empty();
    let prior_atnol = AtnolHistory::calculate(
        amt_base,
        amt_modified_income,
        projection.single_estate,
        projection.has_estate,
        &projection.vintages,
        r,
    );
    let prior_atnol = match prior_atnol {
        Ok(allocation) => allocation,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    // Section 613A(d)(1): taxable income without this depletion, section 199A
    // or an NOL carryback; this year's NOL and ATNOL deductions stay in it.
    if nol_run.royalty_projection && royalty_limits.is_none() {
        *royalty_next = Some([
            before_qbi_signed + itemized_reduction,
            amt_base + qbi_ded - prior_atnol.deduction,
        ]);
    }
    if let Some(k1_depletion_adjustment) = k1_depletion_adjustment
        && (!royalty_regular.is_empty() || k1_depletion_adjustment != 0)
    {
        form_line(
            r,
            FormLine::new(TaxForm::Form6251, "2d"),
            "form6251_line2d_depletion",
            depletion_adjustment + k1_depletion_adjustment,
            &["income.royalty.depletion", "income.partnership", "income.scorp"],
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_K1_AMT_DEPLETION"),
        );
    }
    let amt_base = amt_base - prior_atnol.deduction;
    let amt_capital_carry = crate::extended_rules::capital_carryover(
        amt_short,
        amt_long,
        positive(-amt_capital),
        amt_base,
    );
    emit_lines(
        r,
        &mut (),
        [
            ("amt_capital_gain", amt_capital),
            ("amt_personal_sale_disallowed_loss", dollars(t.amt_personal_sale_loss)),
            ("amt_capital_adjustment", amt_capital_independent_adjustment),
            ("amt_rate_28_gain", amt_tax_collectible),
            ("amt_unrecaptured_section_1250_gain", amt_tax_unrecaptured),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income", "screening.capital_history"],
                "https://www.irs.gov/instructions/i6251",
                msg!("EXPLAIN_FEDERAL_RULES_13"),
            ))
        },
    );
    let reported_amt_carry = repayment.future_capital.map_or(amt_capital_carry, |v| v[1]);
    emit_lines(
        r,
        &mut (),
        [
            ("amt_capital_short_carryforward", reported_amt_carry[0]),
            ("amt_capital_long_carryforward", reported_amt_carry[1]),
            ("amt_capital_loss_carryforward", reported_amt_carry.iter().sum()),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["capital_gain", "alternative_minimum_taxable_income"],
                "https://www.irs.gov/instructions/i6251",
                msg!("EXPLAIN_FEDERAL_RULES_14"),
            ))
        },
    );

    if !j && kiddie_age && p.taxpayer.parent_alive {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "2026 AMT uses the ordinary filing-status exemption: section 55(d)(4)(A)(iii), extended by P.L. 119-21 section 70107, disapplies the child earned-income cap. Rev. Proc. 2025-32 section 4.11 still lists that cap; this estimate follows the statute pending final IRS reconciliation.", crate::Message::new("NOTE_2026_AMT_USES_THE_ORDINARY_FILING_STATUS_EXEMPTION_SECTION_55_D_47C729E6", alloc::vec![])));
    }
    let amt_computation = crate::income_tax::AmtTaxWorksheet {
        year: 2026,
        status: s.filing_status(),
        base: amt_base,
        preferential: amt_tax_pref - amt_tax_collectible - amt_tax_unrecaptured,
        regular_ordinary: positive(
            taxable + positive(excluded) - regular_tax_pref
                + regular_collectible
                + regular_unrecaptured,
        ),
        excluded,
        special: [amt_tax_collectible, amt_tax_unrecaptured],
        foreign: foreign.election().then_some(crate::income_tax::Preference {
            dividends: positive(amt_qualified - amt_elected[1]),
            gain: amt_tax_net_capital,
            line18: amt_schedule_d_18,
            line19: amt_schedule_d_19,
        }),
    }
    .calculate();
    let tentative = amt_computation.tax;
    core::ops::ControlFlow::Continue(AmtAbsorptionOut {
        has_atnol,
        prior_atnol,
        amt_base,
        amt_computation,
        tentative,
    })
}

pub(super) struct AmtFinalTaxIn<'a> {
    pub(super) amt_computation: &'a crate::income_tax::AmtTaxResult,
    pub(super) unaveraged_regular_tax: i64,
    pub(super) bba_additional_tax: i64,
    pub(super) child_tax: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) premium_repayment: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) foreign_credit: i64,
    pub(super) bba_credit: i64,
    pub(super) tentative: i64,
    pub(super) amt_credit: i64,
    pub(super) amt_base: i64,
}

pub(super) fn amt_final_tax_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: AmtFinalTaxIn<'_>,
) -> core::ops::ControlFlow<(), i64> {
    let s = pass.s;
    let AmtFinalTaxIn {
        amt_computation,
        unaveraged_regular_tax,
        bba_additional_tax,
        child_tax,
        education_recapture_tax,
        premium_repayment,
        clean_vehicle_repayment,
        foreign_credit,
        bba_credit,
        tentative,
        amt_credit,
        amt_base,
    } = input;
    // Form 6251 line 10 subtracts the full negative Form 8978 line 14 before
    // comparing regular tax with tentative minimum tax; a positive amount is
    // already included in Form 1040 line 16, as is Form 8814's tax. A decrease can create AMT.
    // The payee-statement election uses the same credit for AMT (Form 6251 line 8).
    let amt_regular_offset = positive(
        unaveraged_regular_tax
            + bba_additional_tax
            + child_tax
            + education_recapture_tax
            + premium_repayment
            + clean_vehicle_repayment
            - foreign_credit
            - bba_credit,
    );
    let amt = positive(positive(tentative - amt_credit) - amt_regular_offset);
    if !crate::extended_rules::record_amt(
        r,
        amt_base,
        amt,
        s.filing_status(),
        tentative,
        amt_credit,
        amt_regular_offset,
    ) {
        // The AMT page owns this derived aggregate; no individual field cause is established.
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Alternative minimum tax",
            "AMT_OUTPUT_RANGE",
            "The calculated alternative minimum taxable income or tax exceeds the supported monetary range."
        );
        withhold_required_source_result(r);
        return core::ops::ControlFlow::Break(());
    }
    crate::income_tax::projection::emit(r, amt_computation, amt > 0);
    core::ops::ControlFlow::Continue(amt)
}

pub(super) struct AmtRelatedDeductionsIn<'a, 'd> {
    pub(super) health_earnings: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) business_exclusions: &'a [crate::foreign_business::BusinessExclusion; 2],
    pub(super) half: [i64; 2],
    pub(super) partner_half_attribution: Option<&'a alloc::collections::BTreeMap<i64, i64>>,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) compensation: [i64; 2],
    pub(super) compensation_nonbusiness: [i64; 2],
    pub(super) compensation_earned_profit: [i64; 2],
    pub(super) ira_facts: &'a Option<alloc::borrow::Cow<'d, crate::FederalIras>>,
    pub(super) qtp_transfers: &'a crate::ira::QtpTransfers,
    pub(super) carry_distributions: [i64; 2],
    pub(super) ira_magi: i64,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) sep_deduction: i64,
}

pub(super) struct AmtRelatedDeductionsOut {
    pub(super) amt_sep_deduction: i64,
    pub(super) amt_health: crate::health::HealthResult,
    pub(super) amt_ira_deduction: i64,
    pub(super) amt_related_adjustment: i64,
}

pub(super) fn amt_related_deductions_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    partner_pass: &PartnerPass,
    input: AmtRelatedDeductionsIn<'_, '_>,
) -> core::ops::ControlFlow<(), AmtRelatedDeductionsOut> {
    let d = pass.d;
    let allocated_entries = pass.allocated_entries;
    let office_context = pass.office_context;
    let p = pass.p;
    let s = pass.s;
    let basis = pass.basis;
    let selected_regular = pass.selected_regular;
    let selected_alternative = pass.selected_alternative;
    let source_claims = pass.context.source_claims;
    let AmtRelatedDeductionsIn {
        health_earnings,
        amt_rental,
        business_exclusions,
        half,
        partner_half_attribution,
        health,
        sep,
        compensation,
        compensation_nonbusiness,
        compensation_earned_profit,
        ira_facts,
        qtp_transfers,
        carry_distributions,
        ira_magi,
        ira,
        sep_deduction,
    } = input;
    // Form 6251 p. 9 Related Adjustments: reuse the deduction owners with
    // actual AMT business income. Neither Schedule SE nor regular phaseout
    // MAGI changes. These are runtime projections, not another fact ledger.
    let mut amt_business_income = alloc::collections::BTreeMap::new();
    let mut amt_health_earnings = health_earnings.clone();
    let mut amt_earned_adjustment = [0; 2];
    for entry in allocated_entries {
        if let FederalIncome::Business { data } = &entry.income {
            let delta = t.business_amt_adjustments.get(&entry.id).copied().unwrap_or(0);
            let profit = data.earned_receipts()
                - match crate::section179_return::expenses_with_claims(
                    d,
                    entry.id,
                    data,
                    selected_regular,
                    Some(office_context),
                    source_claims,
                )
                .map(|expense| {
                    expense
                        - selected_regular
                            .map_or(0, |allocation| allocation.ordinary_limit_adjustment(entry.id))
                }) {
                    Ok(value) => value,
                    Err(issue) => {
                        crate::section179_return::Issue::from(issue).report(r);
                        r.state = ResultState::Unsupported;
                        return core::ops::ControlFlow::Break(());
                    }
                };
            amt_business_income.insert(entry.id, profit + delta);
            amt_health_earnings.insert(
                entry.id,
                health_earnings.get(&entry.id).map_or_else(
                    || positive(dollars(profit + delta)),
                    |regular| positive(dollars(*regular * 100 + delta)),
                ),
            );
            amt_earned_adjustment[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                delta;
        }
    }
    // Reuse source-character earnings after the AMT activity's basis, at-risk,
    // passive and section 179 limits. The regular Schedule SE and its actual
    // half-tax deduction remain unchanged (Form 6251, Related Adjustments).
    // https://www.irs.gov/instructions/i6251
    fn unique_source(
        sources: &[(i64, crate::partnership::PartnerSeResult)],
        id: i64,
    ) -> Option<&crate::partnership::PartnerSeResult> {
        let mut found = sources.iter().filter(|(source, _)| *source == id);
        let source = found.next()?;
        found.next().is_none().then_some(&source.1)
    }
    let mut amt_partner_earned_adjustment = [0; 2];
    let mut exempt_partner_earnings = [false; 2];
    for (owner, exempt) in exempt_partner_earnings.iter_mut().enumerate() {
        *exempt = match crate::clergy::calculate_with_section179_with_claims(
            d,
            if owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value.sect,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
    }
    for entry in allocated_entries {
        let (Some(regular), Some(alternative)) = (
            unique_source(&partner_pass.inputs.partners, entry.id),
            unique_source(&amt_rental.partner_se, entry.id),
        ) else {
            continue;
        };
        amt_business_income.insert(entry.id, alternative.earnings_cents);
        amt_health_earnings.insert(entry.id, positive(alternative.earnings));
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        if !exempt_partner_earnings[owner] {
            amt_partner_earned_adjustment[owner] += alternative.earnings - regular.earnings;
        }
    }
    // An unavailable source-character projection cannot be replaced by an
    // aggregate ordinary net. Foreign section 911 expense allocation also
    // remains necessary when it changes the earned-income benefit limit.
    let unverified_earned_sources: Vec<_> = allocated_entries
        .iter()
        .filter(|entry| match &entry.income {
            FederalIncome::Partnership { .. } => {
                unique_source(&partner_pass.inputs.partners, entry.id).is_none()
                    || unique_source(&amt_rental.partner_se, entry.id).is_none()
            }
            FederalIncome::PartnershipActivities { .. } => {
                partner_pass.inputs.partners.iter().any(|(id, _)| *id == entry.id)
                    && (unique_source(&partner_pass.inputs.partners, entry.id).is_none()
                        || unique_source(&amt_rental.partner_se, entry.id).is_none())
            }
            FederalIncome::Business { data } => {
                matches!(data.source, crate::BusinessSource::ForeignServices { .. })
                    && business_exclusions
                        [usize::from(entry.owner == crate::DocumentOwner::Spouses)]
                    .elected
                    && t.business_amt_adjustments.get(&entry.id).is_some_and(|v| *v != 0)
            }
            _ => false,
        })
        .collect();
    let mut unverified_compensation = [false; 2];
    let mut amt_benefit_draft = None;
    for entry in &unverified_earned_sources {
        unverified_compensation[usize::from(entry.owner == crate::DocumentOwner::Spouses)] = true;
        let claims = match &entry.income {
            FederalIncome::Partnership { data } => {
                data.pensions != 0 || data.health_insurance != crate::BusinessHealthInsurance::None
            }
            FederalIncome::Business { data } => {
                data.retirement != crate::BusinessRetirement::None
                    || data.health_insurance != crate::BusinessHealthInsurance::None
            }
            _ => false,
        };
        if !claims {
            continue;
        }
        note_once_scoped(
            r,
            msg!("NOTE_AMT_EARNED_BENEFIT_SOURCE"),
            crate::interview::income_issue_context(entry).manual(),
        );
        // Normal supported working-copy omission: source facts and every
        // regular amount remain accepted. Remove the uncomputed benefit
        // before the existing owner so no invented unused premium enters
        // medical, including the owner's shared LTC age-cap calculation.
        let alternative = amt_benefit_draft.get_or_insert_with(|| boxed(|| d.clone()));
        if let Some(source) = alternative.income.iter_mut().find(|v| v.id == entry.id) {
            match &mut source.income {
                FederalIncome::Partnership { data } => {
                    data.retirement = crate::PartnerRetirement::None;
                    data.pensions = 0;
                    data.health_insurance = crate::BusinessHealthInsurance::None;
                    // These are subsets funding the omitted premium claim,
                    // not the actual guaranteed payments or distributions.
                    // Keep those owning income/basis facts unchanged.
                    data.health_guaranteed = 0;
                    data.health_distributions = 0;
                }
                FederalIncome::Business { data } => {
                    data.retirement = crate::BusinessRetirement::None;
                    data.health_insurance = crate::BusinessHealthInsurance::None;
                }
                _ => {}
            }
        }
    }
    let amt_benefits = amt_benefit_draft.as_deref().unwrap_or(d);
    let mut amt_related_rows = FederalResult::empty(d.revision);
    let amt_sep = match crate::sep::calculate_amt_with_section179_with_claims(
        amt_benefits,
        half,
        partner_half_attribution,
        &amt_rental.partner_se,
        &amt_business_income,
        selected_alternative,
        &mut amt_related_rows,
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
    let amt_sep_deduction = amt_sep.owners.iter().sum::<i64>();
    let amt_health = match crate::health::calculate_with_money_with_claims(
        amt_benefits,
        half,
        partner_half_attribution,
        &amt_health_earnings,
        &amt_sep,
        &basis.public_safety,
        partner_pass.specified.as_ref(),
        crate::marketplace::CalculationMoney::Dollars,
        selected_alternative,
        &mut amt_related_rows,
        Some(office_context),
        source_claims,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let amt_compensation = core::array::from_fn(|i| {
        if unverified_compensation[i] {
            // A business loss cannot reduce wages for IRA compensation;
            // the actual halfSE deduction still does. Independently sufficient
            // wages or joint spousal compensation can prove the same limit.
            return positive(compensation_nonbusiness[i] - amt_sep.owners[i]);
        }
        compensation[i] + sep.owners[i] - amt_sep.owners[i]
            + positive(
                compensation_earned_profit[i]
                    + dollars(
                        t.business[i] - t.business_se_exclusion[i] + amt_earned_adjustment[i],
                    )
                    - dollars(t.business[i] - t.business_se_exclusion[i])
                    + amt_partner_earned_adjustment[i],
            )
            - positive(compensation_earned_profit[i])
    });
    let amt_ira_deduction = ira_facts.as_deref().map_or(Ok(0), |facts| {
        crate::ira::amt_deduction(
            facts,
            p,
            s,
            amt_compensation,
            ira_magi,
            qtp_transfers,
            carry_distributions,
        )
    });
    let amt_ira_deduction = match amt_ira_deduction {
        Ok(amount) => amount,
        Err((key, message)) => {
            crate::issue_localized!(r, &ira_issue_context(key), "AMT IRA deduction", key, message,);
            #[cfg(not(test))]
            let _ = message;
            ira.deduction
        }
    };
    if unverified_compensation.iter().any(|v| *v) && amt_ira_deduction != ira.deduction {
        note_once_scoped(
            r,
            msg!("NOTE_AMT_IRA_COMPENSATION_SOURCE"),
            crate::IssueContext::Section(crate::TaxSection::Retirement),
        );
    }
    // The regular owner already reports every source/plan admission issue.
    // Preserve new AMT-only source failures without duplicate normal rows.
    for issue in amt_related_rows.issues {
        if !r.issues.iter().any(|known| known == &issue) {
            r.issues.push(issue);
        }
    }
    let amt_related_adjustment =
        sep_deduction - amt_sep_deduction + health.deduction - amt_health.deduction + ira.deduction
            - amt_ira_deduction;
    emit_lines(
        r,
        &mut (),
        [
            ("amt_sep_deduction", amt_sep_deduction),
            ("amt_self_employed_health_deduction", amt_health.deduction),
            ("amt_ira_deduction", amt_ira_deduction),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income.business.assets", "adjustments"],
                "https://www.irs.gov/instructions/i6251",
                msg!("EXPLAIN_AMT_RELATED_DEDUCTIONS"),
            ))
        },
    );
    core::ops::ControlFlow::Continue(AmtRelatedDeductionsOut {
        amt_sep_deduction,
        amt_health,
        amt_ira_deduction,
        amt_related_adjustment,
    })
}
