// schedule_c items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// A Schedule C line of an ordinary business; a farm reports on Schedule F
// and a professional gambler's capped expenses have no per-line split here.
pub(super) fn schedule_c_line(v: &crate::FederalBusiness, line: &str) -> Option<FormLine> {
    matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
        .then(|| FormLine::new(TaxForm::ScheduleC, line))
}
// One business's Schedule C summary lines, from the same amounts the return
// adds to business income: line 31 is receipts less every expense (cost of
// goods sold and the home office included); 29 adds the home office back, 28
// is the gross income left after 29. A professional gambler's Schedule C
// shows only line 31; a farm has none (Schedule F).
// Forward the same selected allocation and invocation-only HOME facts.

#[allow(clippy::too_many_arguments)]
pub(super) fn schedule_c_lines_with_claims(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    owner: usize,
    v: &crate::FederalBusiness,
    net_profit: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<(), crate::Section179PreparedIssue> {
    let source = "https://www.irs.gov/instructions/i1040sc";
    let inputs = ["income.business"];
    let net_line = match v.activity {
        crate::BusinessActivity::Farming { .. } => return Ok(()),
        crate::BusinessActivity::Gambling { .. } => FormLine::new(TaxForm::ScheduleC, "31"),
        crate::BusinessActivity::Ordinary { receipts, returns, .. } => {
            let office = crate::section179_return::office_with_claims(
                d,
                id,
                v,
                selected,
                home,
                source_claims,
            )?;
            let goods = v.inventory.cost();
            let cooperative = farm_cooperative_with_claims(d, owner, id, source_claims)?.taxable;
            let gross = v.receipts() + cooperative - goods;
            let depreciation = business_depreciation_with_section179_with_claims(
                d,
                id,
                &v.assets,
                selected,
                home,
                source_claims,
            )?;
            emit_lines(
                r,
                &mut (),
                [
                    ("1", "gross_receipts", receipts, msg!("EXPLAIN_BUSINESS_N_N_N")),
                    (
                        "6",
                        "other_income",
                        v.receipts() + cooperative - receipts + returns,
                        msg!("EXPLAIN_BUSINESS_N_N_N_2"),
                    ),
                    (
                        "2",
                        "returns",
                        returns,
                        crate::Message::new("EXPLAIN_BUSINESS_RETURNS", alloc::vec![]),
                    ),
                    (
                        "3",
                        "net_receipts",
                        receipts - returns,
                        crate::Message::new("EXPLAIN_BUSINESS_NET_RECEIPTS", alloc::vec![]),
                    ),
                    (
                        "5",
                        "gross_profit",
                        receipts - returns - goods,
                        crate::Message::new("EXPLAIN_BUSINESS_GROSS_PROFIT", alloc::vec![]),
                    ),
                    ("7", "gross_income", gross, msg!("EXPLAIN_BUSINESS_N_N_N_3")),
                    ("13", "depreciation", depreciation, msg!("EXPLAIN_BUSINESS_N_N_N_4")),
                    (
                        "28",
                        "total_expenses",
                        gross - net_profit
                            + selected
                                .map_or(0, |allocation| allocation.ordinary_limit_adjustment(id))
                            - office,
                        msg!("EXPLAIN_BUSINESS_N_N_N_5"),
                    ),
                    (
                        "29",
                        "tentative_profit",
                        net_profit
                            - selected
                                .map_or(0, |allocation| allocation.ordinary_limit_adjustment(id))
                            + office,
                        msg!("EXPLAIN_BUSINESS_N_N_N_6"),
                    ),
                ],
                |(line, suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(TaxForm::ScheduleC, line)),
                        &alloc::format!("business_{id}_{suffix}"),
                        LineValue::MoneyCents(amount),
                        &inputs,
                        source,
                        explanation,
                    ))
                },
            );
            if let crate::BusinessInventory::Cost {
                opening,
                purchases,
                personal,
                labor,
                materials,
                other,
                closing,
                ..
            } = v.inventory
            {
                let total = opening + purchases - personal + labor + materials + other;
                emit_lines(
                    r,
                    &mut (),
                    [
                        (35, opening),
                        (36, purchases - personal),
                        (37, labor),
                        (38, materials),
                        (39, other),
                        (40, total),
                        (41, closing),
                        (42, goods),
                    ],
                    |(number, amount), _, _| {
                        Some(calculation_row(
                            Some(FormLine::new(TaxForm::ScheduleC, &alloc::format!("{number}"))),
                            &alloc::format!("business_{id}_inventory_line{number}"),
                            LineValue::MoneyCents(amount),
                            &inputs,
                            source,
                            crate::Message::new("EXPLAIN_BUSINESS_INVENTORY", alloc::vec![]),
                        ))
                    },
                );
            }
            for (index, item) in v.other_income.iter().enumerate() {
                line_cents(
                    r,
                    &alloc::format!("business_{id}_other_income_{index}"),
                    item.amount,
                    &["income.business.other_income"],
                    source,
                    crate::Message::new(
                        "EXPLAIN_BUSINESS_OTHER_INCOME",
                        alloc::vec![crate::MessageArg::Text(item.description.clone())],
                    ),
                );
            }
            FormLine::new(TaxForm::ScheduleC, "31")
        }
    };
    push_line(
        r,
        Some(net_line),
        &alloc::format!("business_{id}_net_profit"),
        net_profit,
        &inputs,
        source,
        msg!("EXPLAIN_BUSINESS_N_NET_PROFIT"),
    );
    Ok(())
}

// Certify the present receipt sources of a supplied supported/routed working
// view. The preparation owner still owns raw omissions and global graph checks;
// this helper neither routes again nor constructs a completed return result.

pub(crate) fn direct_business_receipt_source_issue_with_claims(
    d: &FederalDraft,
    sources: Option<&alloc::collections::BTreeSet<i64>>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, &'static str)> {
    let activities: Vec<_> = d.income.iter().filter_map(|entry| {
        if sources.is_some_and(|sources| !sources.contains(&entry.id)) { return None; }
        matches!(&entry.income, FederalIncome::Business { data }
            if matches!(data.activity, crate::BusinessActivity::Ordinary { .. } | crate::BusinessActivity::Farming { .. }))
            .then_some((entry.id, entry.owner.owned()))
    }).collect();
    for entry in &d.income {
        let FederalIncome::GovernmentPayment { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalUnemployment::from(data);
        if !d.income.iter().any(|farm| {
            farm.owner == entry.owner
                && matches!(&farm.income, FederalIncome::Business { data }
                if matches!(data.activity, crate::BusinessActivity::Farming { .. }))
        }) {
            continue;
        }
        let routing = routed_farm(
            d,
            usize::from(entry.owner == crate::DocumentOwner::Spouses),
            data.component_ids.agriculture,
            source_claims,
        );
        if routing.is_ok_and(|id| !activities.contains(&(id, entry.owner.owned()))) {
            // The actual receiver is a farm rental, not this C/F input pool.
            continue;
        }
        if let Some((_, key, _, _)) =
            farm_receipt_requirements(data, routing, ccc_market_gain_reconciled(d, entry.owner))
                .into_iter()
                .find(|(known, _, _, _)| !known)
        {
            return Some((entry.id, key));
        }
    }
    crate::cooperative::before179_receipt_source_issue_with_claims(d, &activities, source_claims)
}

pub(super) struct BusinessCollectIn<'a> {
    pub(super) direct_participation:
        &'a alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,
    pub(super) selected: Option<&'a crate::Section179PreparedAllocation>,
    #[cfg(test)]
    pub(super) source: &'a alloc::string::String,
}

pub(super) fn collect_business(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    v: &crate::FederalBusiness,
    input: BusinessCollectIn<'_>,
) -> Result<(), CollectFailure> {
    let Collect { d, form_context, source_claims, selected_pair, home, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    let BusinessCollectIn {
        direct_participation,
        selected,
        #[cfg(test)]
        source,
    } = input;
    crate::require_localized!(
        r,
        direct_participation.get(e.id) != crate::participation::Participation::Unavailable,
        &context.clone().manual(),
        "Material participation",
        "DIRECT_PARTICIPATION_OTHER_ACTIVITY_SOURCE_REQUIRED",
        "Establish the qualifying hours and legal activity grouping of the other trades or businesses before applying the combined significant-participation test."
    );
    // The Forms 1099-NEC, 1099-K and 1099-MISC amounts that route to this
    // business are inside its line 1 receipts (information_returns::route).
    let from_forms = crate::information_returns::routed_to_with_claims(d, e.id, source_claims);
    if from_forms != 0 && matches!(v.activity, crate::BusinessActivity::Ordinary { .. }) {
        line_cents(
            r,
            &alloc::format!("business_{}_from_forms", e.id),
            from_forms,
            &["income.nec_1099", "income.misc_1099", "income.k_1099"],
            "https://www.irs.gov/instructions/i1040sc",
            msg!("EXPLAIN_BUSINESS_N_FORM_1099_RECEIPTS"),
        );
    }
    if let Some(claim) = &v.employer_tip_fica_credit {
        let supported = matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
            && v.source == crate::BusinessSource::UnitedStates
            && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
            && !matches!(claim.service, crate::EmployerTipService::Other)
            && claim.employees.iter().all(|employee| {
                employee
                    .months
                    .iter()
                    .map(|month| month.wages_excluding_tips + month.reported_tips)
                    .sum::<i64>()
                    <= crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS
            })
            && d.income
                .iter()
                .filter(|entry| {
                    matches!(&entry.income,
                FederalIncome::Business { data } if data.employer_tip_fica_credit.is_some())
                })
                .count()
                == 1;
        let credit_context = if supported { context.clone() } else { context.clone().manual() };
        crate::require_localized!(
            r,
            matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
                && v.source == crate::BusinessSource::UnitedStates
                && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
                && claim.qualified(&v.payroll)
                && d.income.iter().filter(|entry| {
                    matches!(&entry.income, FederalIncome::Business { data } if data.employer_tip_fica_credit.is_some())
                }).count() == 1,
            &credit_context,"Current employer tip FICA credit",
            "ISSUE_USE_ONE_ACTIVE_U_S_FOOD_BEVERAGE_OR_QUALIFYING_BEAUTY_SERVICE_SCHED_60A01969","Use one active U.S. food/beverage or qualifying beauty-service Schedule C employer with complete monthly employee records and matching W-2 box 5 payroll. Beauty service must identify barbering/hair care, nail care, esthetics, or body/spa treatment; other tipped services are unsupported. Confirm customary customer tipping, employer FICA payment, all employee wage-base limits, and the separate 2025 carryback. Ordinary expenses must omit employer FICA on these tips while including the other payroll costs once. Multiple credit sources, pass-through, passive, service-charge, over-wage-base or incomplete payroll cases need separate Form 8846/3800 review.",
        );
        claim.report(e.id, r);
    }
    if let Some(claim) = &v.work_opportunity_credit {
        let credit_context = if matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
            && v.source == crate::BusinessSource::UnitedStates
            && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
        {
            context.clone()
        } else {
            context.clone().manual()
        };
        let wages: i64 = v
            .expenses
            .iter()
            .filter(|x| x.category == crate::BusinessExpenseCategory::Wages)
            .map(|x| x.amount)
            .sum();
        crate::require_localized!(
            r,
            matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
                && v.source == crate::BusinessSource::UnitedStates
                && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
                && claim.qualified(&v.payroll, wages),
            &credit_context,
            "Work opportunity credit",
            "ISSUE_FORM_5884_LINES_1A_1C_NEED_AN_ACTIVE_U_S_SCHEDULE_C_EMPLOYER_WHOSE_F7748056",
            "Form 5884 lines 1a–1c need an active U.S. Schedule C employer whose certified employees each match one of its W-2s, with the 2026 credit wages within that W-2's Medicare wages and included in full on Schedule C line 26, a start date after January 1, 2025 for first-year wages (after January 1, 2024 for second-year wages) and before 2026, and 2025 wages within each targeted group's limit. Successor employers, controlled groups, summer youth employees, designated community residents, passive activities and unconfirmed certifications are not calculated.",
        );
        claim.report(e.id, r);
    }
    if let Some(claim) = &v.retirement_plan_credit {
        let credit_context = if matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
            && v.source == crate::BusinessSource::UnitedStates
            && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
        {
            context.clone()
        } else {
            context.clone().manual()
        };
        crate::require_localized!(
            r,
            matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
                && v.source == crate::BusinessSource::UnitedStates
                && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
                && matches!(v.payroll, crate::BusinessPayroll::Yes { .. })
                && claim.valid()
                && claim.confirmed
                && claim.startup.as_ref().is_none_or(|s| {
                    s.costs <= crate::FederalBusinessExpense::total(&v.expenses)
                }),
            &credit_context,
            "Small employer retirement plan credits",
            "ISSUE_FORM_8881_PART_I_SECTION_1_AND_PART_II_NEED_AN_ACTIVE_U_S_SCHEDULE_76108B89",
            "Form 8881 Part I Section 1 and Part II need an active U.S. Schedule C employer with employee W-2s, an eligible small employer (no more than 100 employees paid $5,000 in the prior year, no plan for substantially the same employees in the prior three years, no controlled or affiliated service group), and start-up costs included in Business expenses. The employer contributions credit (Form 8881 lines 6a–6g, which needs Schedule C line 19) and the military spouse participation credit (Part III lines 12–13) are not calculated for a business's own plan.",
        );
        claim.report(e.id, r);
    }
    if let Some(claim) = &v.small_employer_health_credit {
        let ordinary = matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
            && v.source == crate::BusinessSource::UnitedStates
            && (direct_participation.get(e.id) == crate::participation::Participation::Yes);
        let credit_context = if ordinary { context.clone() } else { context.clone().manual() };
        let benefits = crate::FederalBusinessExpense::category_totals(&v.expenses)
            [crate::BusinessExpenseCategory::EmployeeBenefits.index()];
        crate::require_localized!(
            r,
            ordinary && claim.qualified(&v.payroll, benefits),
            &credit_context,
            "Small employer health insurance credit",
            "ISSUE_FORM_8941_NEEDS_AN_ACTIVE_U_S_SCHEDULE_C_EMPLOYER_WITH_MATCHING_W2S",
            "Form 8941 needs an active U.S. Schedule C employer whose listed employees each match one of its W-2s, with the premiums you paid included in its employee benefit programs expense, and one Form 8941 per owner. Other businesses, household or leased employees, controlled groups and several rating areas are not calculated.",
        );
        // All of one owner's businesses are one employer (Form 8941
        // instructions, Example 1).
        require_one_per_owner(
            r,
            d,
            e.owner.owned(),
            context,
            #[cfg(test)]
            "Small employer health insurance credit",
            |b| b.small_employer_health_credit.is_some(),
        );
        claim.report(
            e.id,
            &v.payroll,
            k1_credit_code(d, e.owner.owned(), crate::K1CreditCode::SmallEmployerHealth),
            r,
        );
    }
    if let Some(claim) = &v.paid_leave_credit {
        let ordinary = matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
            && v.source == crate::BusinessSource::UnitedStates
            && (direct_participation.get(e.id) == crate::participation::Participation::Yes);
        let credit_context = if ordinary { context.clone() } else { context.clone().manual() };
        let categories = crate::FederalBusinessExpense::category_totals(&v.expenses);
        let wages = categories[crate::BusinessExpenseCategory::Wages.index()];
        let benefits = categories[crate::BusinessExpenseCategory::EmployeeBenefits.index()];
        let health_premiums = v
            .small_employer_health_credit
            .as_ref()
            .map_or(0, crate::SmallEmployerHealthCredit::premiums_paid);
        crate::require_localized!(
            r,
            ordinary
                && claim.qualified(
                    &v.payroll,
                    wages,
                    benefits,
                    v.work_opportunity_credit.as_ref(),
                    health_premiums,
                ),
            &credit_context,
            "Paid family and medical leave credit",
            "ISSUE_FORM_8994_NEEDS_AN_ACTIVE_U_S_SCHEDULE_C_EMPLOYER_WITH_MATCHING_W2S",
            "Form 8994 needs an active U.S. Schedule C employer whose listed employees each match one of its W-2s, with each employee's leave wages (and any work opportunity credit wages) within that W-2's Medicare wages, the leave wages included in its wages expense and the insurance premiums in its employee benefit programs expense, and one Form 8994 per owner.",
        );
        // All of one owner's businesses are one employer (section
        // 45S(c)(3) aggregation, as for Form 8941).
        require_one_per_owner(
            r,
            d,
            e.owner.owned(),
            context,
            #[cfg(test)]
            "Paid family and medical leave credit",
            |b| b.paid_leave_credit.is_some(),
        );
        claim.report(e.id, k1_credit_code(d, e.owner.owned(), crate::K1CreditCode::PaidLeave), r);
    }
    if let Some(claim) = &v.fuel_tax_credit {
        crate::require_localized!(
            r,
            v.source == crate::BusinessSource::UnitedStates
                && (v.cash_accounting() || v.accrual_accounting())
                && claim.records_complete
                && claim.expense_treatment != crate::FederalFuelExpenseTreatment::Other
                && claim.fuels.qualified()
                && claim.fuels.card_issuer.as_ref().is_none_or(|issuer| {
                    !issuer.has_retained_tax()
                        || ((direct_participation.get(e.id)
                            == crate::participation::Participation::Yes)
                            && v.cash_fuel_tax_recovery.is_none()
                            && claim.expense_treatment
                                == crate::FederalFuelExpenseTreatment::TaxExcludedFromExpenses
                            && ((v.accrual_accounting()
                                && issuer.retained_accrual_accounting_confirmed == Some(true))
                                || (v.cash_accounting()
                                    && issuer.retained_cash_accounting_confirmed == Some(true))))
                })
                && (!claim.fuels.has_farm_operator_use()
                    || matches!(v.activity, crate::BusinessActivity::Farming { .. }))
                && (claim.fuels.vendor_state_diesel.is_none()
                    && claim.fuels.vendor_state_kerosene.is_none()
                    && claim.fuels.vendor_blocked_pump_kerosene.is_none()
                    && claim.fuels.vendor_bus_diesel.is_none()
                    && claim.fuels.vendor_bus_kerosene.is_none()
                    && claim.fuels.vendor_aviation_kerosene.is_none()
                    && claim.fuels.vendor_aviation_noncommercial.is_none()
                    && claim.fuels.vendor_aviation_nontaxable.is_none()
                    && claim.fuels.card_issuer.is_none()
                    || matches!(v.activity, crate::BusinessActivity::Ordinary { .. })),
            &context,
            "Fuel tax credit",
            "ISSUE_CONFIRM_THE_U_S_SCHEDULE_C_OR_FARM_ACTIVITY_TAX_PAID_QUALIFIED_USE_5EDA324A",
            "Confirm the U.S. Schedule C or farm activity, tax-paid qualified use, purchase/equipment records, no prior Form 8849/8864/720 claim, and whether the credited tax was deducted or removed from expenses. Farm-operator fuel lines require a farm activity; a Schedule C custom aerial applicator may claim its own farm-use aviation gasoline or kerosene, with an unwaived kerosene claim; public-bus diesel/kerosene and foreign-trade aviation kerosene need unwaived claims; foreign-trade aviation gasoline needs civil-aircraft trade and reciprocity for foreign registry; exports need retained proof, and a blender claim needs M registration and certification. Registered-vendor government diesel line 6a and nonaviation-kerosene line 7a require an ordinary Schedule C seller, active UV registration, government-buyer certificate and records, tax-excluded sale, tax repayment or buyer consent, and no government-issued credit card. Blocked-pump kerosene line 7b requires an ordinary Schedule C seller, active UP registration, qualifying fixed and marked pump, nontaxable buyer use, and buyer records for locked-pump sales over five gallons. Registered-vendor bus lines 6b and 7c require an ordinary Schedule C seller, active UB registration, current buyer waivers, qualifying public-bus service and distinct sale records. Commercial-aviation kerosene vendor lines 8a and 8b require an ordinary Schedule C seller, active UA registration, current signed Model Waiver L, and separate original-tax-rate sale and cost records. Nonexempt noncommercial aviation line 8c requires an ordinary Schedule C seller, active UA registration, 24.4-cent original tax, current signed Model Certificate Q, distinct sale and cost records, and no duplicated gallons. Nontaxable noncommercial aviation lines 8d/8e require an ordinary Schedule C seller, active UA registration and current Model Waiver L, or UV registration and Model Certificate P for government sales, eligible use, original tax rate, and distinct sale and cost records; foreign-trade gallons also generate line 8f LUST credit. A registered card issuer on line 13 needs an ordinary Schedule C business, active CC registration, government-issued credit-card purchase, current signed Model Certificate R, buyer tax relief, vendor settlement, actual cost, and no duplicate Form 720/8849 claim. Retained buyer tax additionally requires a direct active issuer, exact line-13 reconciliation, no buyer/vendor profit-and-loss entries, no tax-cost deduction or duplicated credit income, and no separate cash-recovery history. An accrual issuer includes the retained benefit as 2026 Schedule C other income; a cash issuer with no 2026 receipt reviews that separate income in its actual or constructive receipt or credit-application year. Both claim the 2026 Form 4136 refundable payment once. Other fuel uses need their own worksheet.",
        );
        emit_lines(r, &mut (), claim.labeled_lines(), |(form_line, amount), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("business_{}_fuel_{}", e.id, form_line),
                LineValue::MoneyCents(amount),
                &["income.business.fuel_tax_credit"],
                "https://www.irs.gov/pub/irs-dft/f4136--dft.pdf",
                msg!("EXPLAIN_BUSINESS_N_FUEL_N"),
            ))
        });
        if let Some(issuer) = claim.fuels.card_issuer.as_ref()
            && issuer.has_retained_tax()
        {
            if v.accrual_accounting() {
                line_cents(
                    r,
                    &alloc::format!("business_{}_card_issuer_retained_schedule_c_6", e.id),
                    issuer.retained_income_cents(),
                    &["income.business.fuel_tax_credit.card_issuer"],
                    "https://www.irs.gov/instructions/i1040sc",
                    msg!("EXPLAIN_BUSINESS_N_CARD_ISSUER_RETAINED_SCHEDULE_C_N"),
                );
            } else if v.cash_accounting() {
                r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                    "2026 registered card issuer {}: ${}.{:02} retained buyer-funded Form 4136 line 13 credit is a 2026 refundable payment on Schedule 3 line 12, not 2026 Schedule C income. Buyer payment and vendor settlement are card-principal flows. Review the separate section 61 business gain as Schedule C line 6 income in the year the benefit is actually or constructively received or applied as a credit, often 2027. Filing the claim alone does not establish receipt when a refund is delayed. This App does not compute a later return. No 2026 claim payment, credit application, or constructive receipt is assumed. Section 61 and Schedule C line 6 support the gain; Publication 510's filing-year rule is directly for deducted-tax recovery, not the authority for this no-deduction claim.",
                    e.id,
                    issuer.retained_income_cents() / 100,
                    issuer.retained_income_cents() % 100,
                ), Message::new("NOTE_CARD_ISSUER_RETAINED_FUEL_CREDIT", alloc::vec![
                    MessageArg::Number(e.id), MessageArg::usd_cents(issuer.retained_income_cents()),
                ])));
            }
        }
    }
    if let Some(recovery) = &v.cash_fuel_tax_recovery {
        crate::require_localized!(
            r,
            v.source == crate::BusinessSource::UnitedStates
                && v.cash_accounting()
                && (direct_participation.get(e.id) == crate::participation::Participation::Yes)
                && recovery.qualified(),
            &context,
            "Cash fuel tax recovery",
            "ISSUE_CONFIRM_THE_CASH_METHOD_ORIGINAL_PRIOR_YEAR_FORM_4136_RETURN_WAS_FI_DAD735AB",
            "Confirm the cash-method original prior-year Form 4136 return was filed in 2026, or the amended-return credit or fuel-tax refund was received in 2026. For full benefit, the fuel-tax deduction reduced prior income tax in full. The narrow partial 2025 case needs the filed return and retained refigured regular-tax comparison, negative taxable-income exclusion, active deduction, and no AMT, NIIT, credit, carryover or duplicate recovery. Other partial-benefit, passive, NOL and distinct AMT/NIIT histories need separate review.",
        );
        if let Some(partial) = &recovery.partial_prior_year_benefit {
            line_cents(
                r,
                &alloc::format!("business_{}_cash_fuel_tax_recovery_gross", e.id),
                recovery.amount,
                &["income.business.cash_fuel_tax_recovery"],
                "https://www.irs.gov/publications/p510",
                msg!("EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_GROSS"),
            );
            line_cents(
                r,
                &alloc::format!("business_{}_cash_fuel_tax_recovery_excluded", e.id),
                recovery.excluded_amount(),
                &["income.business.cash_fuel_tax_recovery.partial_prior_year_benefit"],
                "https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol2/pdf/CFR-2025-title26-vol2-sec1-111-1.pdf",
                msg!("EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_EXCLUDED"),
            );
            r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                "Partial 2025 fuel-tax benefit for business {}: retain {} with the filed Form 1040 and refigured comparison. The 2026 Schedule C recovery equals the increase in 2025 taxable income after removing only this fuel-tax deduction, including its half-SE and QBI effects; original taxable income was zero. The refigured regular tax confirms a benefit but is not the recovery amount. No 2025 AMT, NIIT, passive limitation, NOL, credit or increased carryover is assumed.",
                e.id, partial.worksheet_reference,
            ), Message::new("NOTE_PARTIAL_FUEL_TAX_BENEFIT", alloc::vec![
                MessageArg::Number(e.id), MessageArg::Text(partial.worksheet_reference.clone()),
            ])));
        }
        line_cents(
            r,
            &alloc::format!("business_{}_cash_fuel_tax_recovery", e.id),
            recovery.taxable_amount(),
            &["income.business.cash_fuel_tax_recovery"],
            "https://www.irs.gov/publications/p525",
            if matches!(v.activity, crate::BusinessActivity::Farming { .. }) {
                msg!("EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY")
            } else {
                msg!("EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_2")
            },
        );
    }
    v.startup.report(
        r,
        e.id,
        #[cfg(test)]
        source,
    );
    crate::method_change::report(
        &v.method_changes,
        r,
        &if matches!(v.activity, crate::BusinessActivity::Farming { .. }) {
            alloc::format!("farm_{}", e.id)
        } else {
            alloc::format!("business_{}", e.id)
        },
        "income.business.method_changes",
        &crate::interview::income_display_arg(e),
        context,
    );
    for (index, asset) in v.intangibles.iter().enumerate() {
        asset.report(
            r,
            e.id,
            index,
            #[cfg(test)]
            source,
        );
    }
    crate::amortization::report_timber(
        &v.timber,
        r,
        e.id,
        #[cfg(test)]
        source,
    );
    match crate::cohorts::property_sales(v) {
        Ok(sales) => {
            for (index, sale) in sales {
                if !matches!(
                    v.other_amortization[index].kind,
                    crate::OtherAmortizationKind::Creative { .. }
                ) {
                    crate::asset_sale::report_amortized_sale(
                        r,
                        &alloc::format!("business_{}_amortization_{index}_disposal", e.id),
                        &sale,
                        "income.business.other_amortization",
                    );
                }
            }
        }
        Err(key) => crate::federal_rules::require_localized(
            r,
            false,
            context,
            #[cfg(test)]
            source,
            key,
            #[cfg(test)]
            "Resolve actual original property basis and disposition treatment before the complete estimate can be calculated.",
        ),
    }
    for (index, cost) in v.other_amortization.iter().enumerate() {
        crate::federal_rules::require_localized(
            r,
            !cost.needs_amt_source(
                direct_participation.get(e.id) == crate::participation::Participation::Yes,
            ),
            &crate::IssueContext::income_id(e.id).item("data.other_amortization", index),
            #[cfg(test)]
            source,
            "ISSUE_AMORTIZATION_AMT_SOURCE_NEEDED",
            #[cfg(test)]
            "This cohort needs its separate AMT recovery and basis before the complete estimate can be calculated.",
        );
    }
    crate::cohorts::report_other_costs(
        &v.other_amortization,
        &v.assets,
        r,
        e.id,
        #[cfg(test)]
        source,
    );
    crate::amortization::report_form_4562b(
        r,
        e.id,
        &v.startup,
        &v.intangibles,
        &v.timber,
        &v.other_amortization,
        &v.assets,
    );
    expense_elections(
        r,
        &v.expenses,
        context,
        #[cfg(test)]
        source,
        &alloc::format!("business_{}", e.id),
    );
    // A farm's expenses print on Schedule F Part II, never Schedule C.
    if matches!(v.activity, crate::BusinessActivity::Farming { .. }) {
        crate::farm::expense_lines(r, e.id, v);
    } else {
        // Schedule F lines 11, 13, 16-19, 23, 26, 27 and 31 have no
        // Schedule C line.
        if let Some(farm) = v.expenses.iter().find(|e| e.category.farm_only()) {
            r.issues.push(FederalIssue {
                context: context.clone(),
                #[cfg(test)]
                section: source.clone(),
                #[cfg(test)]
                message: "FARM_ONLY_EXPENSE_IN_BUSINESS".into(),
                display: crate::Message::new(
                    "FARM_ONLY_EXPENSE_IN_BUSINESS",
                    alloc::vec![crate::MessageArg::Nested(alloc::boxed::Box::new(
                        crate::farm::expense_explanation(&farm.category),
                    ))],
                ),
            });
        }
        // Line 26 is wages less the Form 5884 employment credit and
        // the Form 8994 line 1a credit; line 14 is the employee
        // benefit programs less the Form 8941 credit (section
        // 280C(h)) and the Form 8994 line 1b credit (section
        // 280C(a)).
        let employment_credit =
            v.work_opportunity_credit.as_ref().map_or(0, |claim| claim.credit() * 100);
        let health_credit = v
            .small_employer_health_credit
            .as_ref()
            .map_or(0, |claim| claim.credit(&v.payroll) * 100);
        let (leave_wage_credit, leave_premium_credit) = v
            .paid_leave_credit
            .as_ref()
            .map_or((0, 0), |claim| (claim.wage_credit() * 100, claim.premium_credit() * 100));
        for ((suffix, explanation), amount) in crate::BusinessExpenseCategory::LINES
            .into_iter()
            .zip(crate::FederalBusinessExpense::category_totals(&v.expenses))
        {
            let amount = match suffix {
                "line26_wages" => amount - employment_credit - leave_wage_credit,
                "line14_employee_benefit_programs" => amount - health_credit - leave_premium_credit,
                // The plan start-up costs are in Business expenses (line 17);
                // the Form 8881 credit takes that part of the deduction away
                // (section 45E(e)(2)).
                "line17_legal_and_professional" => amount - v.plan_startup_credit_cents(),
                // Line 23 also holds the employer FICA on tips that Form 8846
                // does not take and a standard-mileage vehicle's property tax.
                "line23_taxes_and_licenses" => amount + v.line23_additions(),
                // Part V (line 48) holds a section 481(a) decrease, the start-up
                // costs and the section 197 amortization.
                "line20a_rent_vehicles_equipment" => {
                    amount
                        + v.leased_vehicles
                            .iter()
                            .map(|v| {
                                let a = v.amounts();
                                a.net_payments()
                            })
                            .sum::<i64>()
                }
                "line27b_other" => {
                    amount
                        + v.de_minimis.as_ref().map_or(0, |v| v.purchases)
                        + crate::method_change::decrease(&v.method_changes)
                        + v.amortization_cents()
                }
                _ => amount,
            };
            if amount != 0 {
                push_line(
                    r,
                    schedule_c_line(v, printed_line(suffix)),
                    &alloc::format!("business_{}_expense_{suffix}", e.id),
                    amount,
                    &["income.business.expenses"],
                    "https://www.irs.gov/instructions/i1040sc",
                    crate::Message::new(explanation, alloc::vec![]),
                );
            }
        }
    }
    crate::require_localized!(
        r,
        crate::depreciation::consistent_elections(d),
        &context,
        &source,
        "ISSUE_USE_ONE_CONSISTENT_BONUS_GDS_METHOD_ELECTION_FOR_EACH_RECOVERY_CLAS_5ABD1EF0",
        "Use one consistent bonus/GDS method election for each recovery class across all businesses and nonbusiness rentals on this return.",
    );
    let form_4562 = crate::depreciation::assets_need_form_4562(&v.assets)
        // Other-than-Schedule-C vehicle use requires Form 4562,
        // even when standard mileage leaves no depreciable asset.
        || (matches!(v.activity, crate::BusinessActivity::Farming { .. })
            && !v.vehicles.is_empty());
    for (index, asset) in v.assets.iter().enumerate() {
        if asset.is_vehicle() {
            crate::require_localized!(
                r,
                asset.vehicle_supported()
                    && matches!(
                        v.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    ),
                &if !asset.vehicle_supported() && asset.class != crate::EquipmentClass::Five {
                    context.item("business_assets", index)
                } else {
                    context.item("business_assets", index).manual()
                },
                "Actual-expense vehicle",
                "ISSUE_CONFIRM_AN_OWNED_5_YEAR_VEHICLE_ON_SCHEDULE_C_WITH_ACTUAL_EXPENSES_93473DC7",
                "Confirm an owned 5-year vehicle on Schedule C or Schedule F with actual expenses and its actual vehicle design. New vehicles need more than 50% business use for section 179; a heavy passenger SUV without a design exception has a $32,000 election limit. Continuing passenger automobiles need a supported original schedule and annual cap history; unsupported changed-use, carryforward, lease and employer-provided cases need their own rules (Form 4562 Part V).",
            );
            if asset.is_listed_vehicle() {
                vehicle_columns(
                    r,
                    e.id,
                    index,
                    &v.assets,
                    v.vehicles.len(),
                    crate::depreciation::mid_quarter_with_macrs(
                        d,
                        Some(form_context.macrs().as_ref()),
                    ),
                    crate::section179_return::asset_recovery_with_claims(
                        d,
                        e.id,
                        index,
                        asset,
                        selected,
                        source_claims,
                    )
                    .as_ref(),
                );
            }
        }
        crate::require_localized!(
            r,
            asset.qualifies(),
            &context,
            &source,
            asset.nonmacrs_issue().unwrap_or(
                "ISSUE_CONFIRM_QUALIFYING_PURCHASED_EQUIPMENT_WITH_THE_ENTERED_SERVICE_DAT_EDE7B97E"
            ),
            "Confirm qualifying purchased equipment with the entered service date, first-year election or complete continuing history with identical regular/AMT basis or the separately recorded required AMT schedule. A first-use 2026 asset acquired before September 28, 2017 receives no bonus; ordinary 200% GDS for a 3-, 5-, 7-, or 10-year class uses the same basis, recovery period, and convention with 150% for AMT. Continuing earlier purchases require post-1998 service and 150%/straight-line GDS or a supported separate 200%/150% history. Unrecovered basis after the recovery period or an unsupported history needs its separate worksheet.",
        );
        asset_disposition_rows_with_claims(
            d,
            t,
            r,
            #[cfg(test)]
            source,
            crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref())),
            e.id,
            index,
            &v.assets,
            v.vehicles.len(),
            source_claims,
            Some(form_context.macrs().as_ref()),
        );
        form_4562_asset_lines_with_claims(
            crate::related_forms::RelatedFormContext::bound(d, source_claims, direct_participation)
                .with_macrs(&form_context.macrs()),
            r,
            e.id,
            index,
            asset,
            form_4562,
            selected_pair,
        );
    }
    if form_4562 {
        form_4562_totals_with_section179_with_claims(
            d,
            r,
            e.id,
            &v.assets,
            selected,
            home,
            source_claims,
        )?;
    }
    if let Some(sale) = v.sale.as_ref().filter(|sale| sale.supported()) {
        business_sale_rows(t, r, e, sale);
    }
    if v.assets.iter().any(|asset| {
        asset.needs_macrs_in_context(form_context.macrs().context(
            e.owner.owned(),
            Some(e.id),
            asset,
        ))
    }) {
        line(
            r,
            &alloc::format!("business_{}_macrs_mid_quarter", e.id),
            i64::from(crate::depreciation::mid_quarter_with_macrs(
                d,
                Some(form_context.macrs().as_ref()),
            )),
            &["income.business.assets"],
            "https://www.irs.gov/publications/p946",
            msg!("EXPLAIN_BUSINESS_N_MACRS_MID_QUARTER"),
        );
    }
    if let crate::BusinessActivity::Gambling { withholding, eligible, .. } = v.activity {
        t.withholding_other += withholding;
        let professional_count = d.income.iter().filter(|entry| matches!(&entry.income, FederalIncome::Business { data } if matches!(data.activity, crate::BusinessActivity::Gambling { .. }))).count();
        crate::require_localized!(
            r,
            eligible
                && professional_count == 1
                && !d
                    .income
                    .iter()
                    .any(|entry| { matches!(entry.income, FederalIncome::Gambling { .. }) })
                && matches!(v.inventory, crate::BusinessInventory::None)
                && !crate::home_office::uses_activity(d, e.id)
                && matches!(v.health_insurance, crate::BusinessHealthInsurance::None)
                && matches!(v.retirement, crate::BusinessRetirement::None),
            &context,
            &source,
            "ISSUE_CONFIRM_PROFESSIONAL_GAMBLING_MIXED_CASUAL_PROFESSIONAL_OR_MULTIPLE_C88656A0",
            "Confirm professional gambling. Mixed casual/professional or multiple wagering activities, inventory, home-office and owner health/retirement coordination need additional worksheets.",
        );
        line(
            r,
            &alloc::format!("business_{}_wagering_deduction", e.id),
            dollars(crate::section179_return::expenses_with_claims(
                d,
                e.id,
                v,
                selected,
                home,
                source_claims,
            )?),
            &["income.business.activity", "income.business.expenses"],
            "https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm",
            msg!("EXPLAIN_BUSINESS_N_WAGERING_DEDUCTION"),
        );
    }
    if let crate::BusinessActivity::Farming { data } = &v.activity {
        data.report_with_section179_with_claims(
            crate::related_forms::RelatedFormContext::bound(d, source_claims, direct_participation)
                .with_macrs(&form_context.macrs()),
            v,
            e.id,
            r,
            selected,
            home,
        )?;
        t.withholding_1099 +=
            data.insurance.withholding() + data.crp.iter().map(|p| p.withholding).sum::<i64>();
        if let Some(ccc) = &data.ccc {
            t.withholding_1099 += ccc.withholding;
            crate::farm::require_same_ccc_method(d, e.owner.owned(), ccc, r);
        }
    }
    let statutory = crate::statutory_wages::business(d, e.id);
    let clergy_source = crate::clergy::business_source_with_section179_with_claims(
        d,
        e.id,
        v,
        selected,
        home,
        source_claims,
    )?;
    let source_se_exclusion = if statutory {
        v.receipts()
            - crate::section179_return::expenses_with_claims(
                d,
                e.id,
                v,
                selected,
                home,
                source_claims,
            )?
            + selected.map_or(0, |allocation| allocation.ordinary_limit_adjustment(e.id))
    } else {
        v.se_exclusion() + clergy_source.excluded_cents + clergy_source.expense_correction_cents
    };
    t.business_se_exclusion[owner] += source_se_exclusion;
    // Statutory C contains only the associated W-2 pay and keeps
    // its wage exclusion. Other passive trade items not actually
    // taken into SE retain their native source amount for1411.
    if !statutory && v.current_passive_supported(direct_participation.get(e.id)) {
        t.passive_business_niit.insert(
            e.id,
            if clergy_source.whole_excluded {
                source_se_exclusion
            } else {
                clergy_source.excluded_cents
            },
        );
    }
    let source_profit = farm_receipts_with_refunds_with_claims(d, e.id, v, false, source_claims)?
        - crate::section179_return::expenses_with_claims(
            d,
            e.id,
            v,
            selected,
            home,
            source_claims,
        )?
        + clergy_source.expense_correction_cents;
    let source_profit = selected
        .and_then(|allocation| allocation.final_ordinary_profit(e.id))
        .unwrap_or(source_profit);
    t.business[owner] += source_profit;
    t.business_profits.insert(e.id, source_profit);
    crate::require_localized!(
        r,
        v.notary_profit
            <= positive(
                v.receipts() + farm_routed_income_with_claims(d, owner, e.id, source_claims)?
                    - crate::section179_return::expenses_with_claims(
                        d,
                        e.id,
                        v,
                        selected,
                        home,
                        source_claims
                    )?,
            ),
        &context,
        &source,
        "ISSUE_NOTARY_PROFIT_ABOVE_NET",
        "Notary profit cannot exceed this business's net profit.",
    );
    crate::require_localized!(
        r,
        v.notary_gross.is_none_or(|gross| gross <= v.receipts() - v.inventory.cost()),
        &context,
        &source,
        "ISSUE_NOTARY_GROSS_ABOVE_INCOME",
        "Notary gross income cannot exceed this business’s gross income after cost of goods.",
    );
    schedule_c_lines_with_claims(
        d,
        r,
        e.id,
        owner,
        v,
        source_profit,
        selected,
        home,
        source_claims,
    )?;
    let amt_adjustment =
        crate::cooperative_basis::depreciation_with_claims(
            d,
            e.id,
            &v.assets,
            crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref())),
            false,
            source_claims,
            Some(form_context.macrs().as_ref()),
        )? - crate::cooperative_basis::depreciation_with_claims(
            d,
            e.id,
            &v.assets,
            crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref())),
            true,
            source_claims,
            Some(form_context.macrs().as_ref()),
        )? + farm_routed_income_with_refunds_with_claims(d, owner, e.id, true, source_claims)?
            - farm_routed_income_with_refunds_with_claims(d, owner, e.id, false, source_claims)?;
    let amt_adjustment = amt_adjustment
        + crate::farm_conservation::business_amt_adjustment_with_claims(d, e.id, source_claims)
        + crate::farm_supplies::business_amt_adjustment_with_claims(d, e.id, source_claims)
        + crate::farm_production::business_amt_adjustment(d, e.id)
        + crate::method_change::amt_difference(&v.method_changes);
    let amt_adjustment =
        crate::section179_return::amt_adjustment(e.id, amt_adjustment, selected_pair)?;
    let amt_adjustment = amt_adjustment
        + selected_pair.map_or(0, |pair| {
            pair[1].allocation.ordinary_limit_adjustment(e.id)
                - pair[0].allocation.ordinary_limit_adjustment(e.id)
                + pair[1]
                    .allocation
                    .passive_ordinary
                    .get(&e.id)
                    .map_or(0, |source| source.operating_correction_cents)
                - pair[0]
                    .allocation
                    .passive_ordinary
                    .get(&e.id)
                    .map_or(0, |source| source.operating_correction_cents)
        });
    t.business_depreciation_adjustment += amt_adjustment;
    t.business_amt_adjustments.insert(e.id, amt_adjustment);
    t.business_sources.insert(
        e.id,
        [
            crate::community::BusinessSourceProjection::produce(
                d,
                e.id,
                source_profit,
                false,
                source_claims,
            )?,
            crate::community::BusinessSourceProjection::produce(
                d,
                e.id,
                source_profit + amt_adjustment,
                true,
                source_claims,
            )?,
        ],
    );
    if let Some(source) =
        selected_pair.and_then(|pair| pair[1].allocation.final_ordinary.get(&e.id))
    {
        crate::extended_rules::report_risk_projection(
            e.id,
            true,
            "income.business.at_risk",
            source.projection.clone(),
            r,
        )
        .map_err(|issue| {
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(e.id, issue.key())
        })?;
    }
    // A vehicle is either a standard-mileage vehicle or an
    // actual-expense asset by its page method; it cannot be both.
    if !v.vehicles.is_empty()
        || !v.leased_vehicles.is_empty()
        || v.assets.iter().any(|asset| asset.is_vehicle())
    {
        // Schedule C line 9, or Schedule F line 10 for a farm.
        let farm = matches!(v.activity, crate::BusinessActivity::Farming { .. });
        // The 2026 drafts print the business share of vehicle-loan
        // interest on its own line (Schedule C 16b, Schedule F 21b),
        // so line 9 / 10 holds the rest; the sum is unchanged.
        let loan_interest = crate::BusinessMileage::loan_interest(&v.vehicles)
            + v.assets.iter().map(crate::BusinessAsset::vehicle_loan_interest).sum::<i64>();
        push_line(
            r,
            Some(if farm {
                FormLine::new(TaxForm::ScheduleF, "10")
            } else {
                FormLine::new(TaxForm::ScheduleC, "9")
            }),
            &alloc::format!("business_{}_car_and_truck", e.id),
            crate::BusinessMileage::total(&v.vehicles)
                + v.leased_vehicles.iter().map(|v|v.amounts().operating).sum::<i64>()
                - crate::BusinessMileage::loan_interest(&v.vehicles)
                // Schedule C prints the property tax on line 23 and
                // Schedule F on line 29 (2026 i1040sf line 10).
                - crate::BusinessMileage::property_tax(&v.vehicles)
                + v.assets.iter().map(crate::BusinessAsset::vehicle_operating).sum::<i64>(),
            &["income.business.vehicles", "income.business.assets"],
            if farm {
                "https://www.irs.gov/instructions/i1040sf"
            } else {
                "https://www.irs.gov/instructions/i1040sc"
            },
            msg!("EXPLAIN_BUSINESS_N_CAR_AND_TRUCK"),
        );
        if loan_interest != 0 {
            push_line(
                r,
                Some(if farm {
                    FormLine::new(TaxForm::ScheduleF, "21b")
                } else {
                    FormLine::new(TaxForm::ScheduleC, "16b")
                }),
                &alloc::format!("business_{}_vehicle_loan_interest", e.id),
                loan_interest,
                &["income.business.vehicles", "income.business.assets"],
                if farm {
                    "https://www.irs.gov/instructions/i1040sf"
                } else {
                    "https://www.irs.gov/instructions/i1040sc"
                },
                msg!("EXPLAIN_BUSINESS_N_VEHICLE_LOAN_INTEREST"),
            );
        }
    }
    if let Some(election) = &v.de_minimis {
        line_cents(
            r,
            &alloc::format!("business_{}_de_minimis", e.id),
            election.purchases,
            &["income.business.de_minimis"],
            "https://www.irs.gov/businesses/small-businesses-self-employed/tangible-property-final-regulations",
            crate::Message::new("EXPLAIN_BUSINESS_DE_MINIMIS", alloc::vec![]),
        );
    }
    for (n, vehicle) in v.leased_vehicles.iter().enumerate() {
        let a = vehicle.amounts();
        for (suffix, amount) in [("payments", a.payments), ("inclusion", a.inclusion)] {
            line_cents(
                r,
                &alloc::format!("business_{}_leased_vehicle_{n}_{suffix}", e.id),
                amount,
                &["income.business.leased_vehicles"],
                "https://www.irs.gov/publications/p463",
                crate::Message::new("EXPLAIN_LEASE_AMOUNT", alloc::vec![]),
            );
        }
    }
    for (index, vehicle) in v.vehicles.iter().enumerate() {
        line(
            r,
            &alloc::format!("business_{}_vehicle_{}_expense", e.id, index),
            dollars(vehicle.expense()),
            &["income.business.vehicles"],
            "https://www.irs.gov/irb/2026-29_irb",
            msg!("EXPLAIN_BUSINESS_N_VEHICLE_N_EXPENSE"),
        );
        line(
            r,
            &alloc::format!("business_{}_vehicle_{}_basis_reduction", e.id, index),
            dollars(vehicle.basis_reduction()),
            &["income.business.vehicles"],
            "https://www.irs.gov/pub/irs-drop/n-26-10.pdf",
            msg!("EXPLAIN_BUSINESS_N_VEHICLE_N_BASIS_REDUCTION"),
        );
    }

    let office =
        crate::section179_return::office_with_claims(d, e.id, v, selected, home, source_claims)?;
    push_line(
        r,
        schedule_c_line(v, "30"),
        &alloc::format!("business_{}_home_office", e.id),
        dollars(office) * 100,
        &["home_sites"],
        "https://www.irs.gov/instructions/i1040sc",
        msg!("EXPLAIN_BUSINESS_N_N_HOME_OFFICE"),
    );

    push_line(
        r,
        schedule_c_line(v, "4"),
        &alloc::format!("business_{}_cost_of_goods_sold", e.id),
        dollars(v.inventory.cost()) * 100,
        &["income.business.inventory"],
        "https://www.irs.gov/instructions/i1040sc",
        msg!("EXPLAIN_BUSINESS_N_COST_OF_GOODS_SOLD"),
    );
    crate::require_localized!(
        r,
        match v.source {
            crate::BusinessSource::UnitedStates => true,
            crate::BusinessSource::ForeignServices { confirmed, .. } => {
                confirmed
                    && matches!(v.income_source, crate::BusinessIncomeSource::Services)
                    && matches!(v.activity, crate::BusinessActivity::Ordinary { .. })
                    && matches!(v.inventory, crate::BusinessInventory::None)
                    && v.assets.is_empty()
                    && v.vehicles.is_empty()
                    && v.intangibles.is_empty()
                    && v.timber.is_empty()
                    && !crate::home_office::uses_activity(d, e.id)
            }
        },
        &context,
        "Business income source",
        "ISSUE_CONFIRM_ORDINARY_WHOLLY_FOREIGN_SERVICES_SUBJECT_TO_U_S_SE_TAX_WITH_70B32FC6",
        "Confirm ordinary wholly foreign services subject to U.S. SE tax, without special foreign treatment. Mixed sources, capital-producing activities and foreign property/vehicle/home-office deductions need their additional source and depreciation worksheets.",
    );
    // Form 8829: its lines and refusals by name.
    crate::home_office::report_with_section179_with_claims(
        d,
        r,
        e.id,
        v,
        selected,
        home,
        source_claims,
    )?;
    // Passive, not-at-risk and special-treatment businesses are
    // left out before the rules run.
    crate::require_localized!(
        r,
        v.inventory.eligible(),
        &context,
        &source,
        "ISSUE_CONFIRM_ACTIVE_OPERATION_FULL_AT_RISK_EXPOSURE_AND_THE_PERMITTED_AC_DFECB6C3",
        "Confirm active operation, full at-risk exposure and the permitted accounting/inventory method; unlisted business treatments require their own worksheets.",
    );
    t.withholding_1099 += v.withholding;
    Ok(())
}
