use super::HomeContext;
use crate::kernel::ratio::{self, exact};
use crate::{
    FederalDraft, FederalIncome, HomeCostKind, HomeOfficeClaim, HomeOfficeSite, HomeTenure,
    OfficeMethod, OfficeOldConvention, OfficeRecovery, Owner,
};
use alloc::{collections::BTreeMap, vec::Vec};

#[derive(Clone, Debug)]
pub(super) struct SiteCalculation {
    pub site: i64,
    pub owner: Owner,
    pub deductions: BTreeMap<i64, i64>,
    pub casualty: crate::casualty::home::ThirdTierResult,
    pub gross_cents: i64,
    pub lines: super::Lines,
}

fn amount(value: &exact::Exact) -> Result<i64, &'static str> {
    exact::rounded(value).ok_or("HOME_OUTPUT_UNAVAILABLE")
}

fn depreciation(site: &HomeOfficeSite, claim: &HomeOfficeClaim) -> Result<i64, &'static str> {
    let HomeTenure::Owned { building, improvements } = &site.tenure else {
        return Ok(0);
    };
    let building = building.as_deref().ok_or("HOME_DEPRECIATION_HISTORY_UNAVAILABLE")?;
    let fraction = claim.fraction(site.home_area).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
    let mut total = exact::whole(0);
    for building in core::iter::once(building).chain(improvements) {
        if building.stopped.is_some()
            || building.production.is_some()
            || !matches!(building.schedule.recovery, OfficeRecovery::Macrs39)
            || !matches!(building.schedule.convention, OfficeOldConvention::MidMonth)
            || building.schedule.straight_line_from.is_some()
        {
            return Err("HOME_DEPRECIATION_HISTORY_UNAVAILABLE");
        }
        if building.first_used / 10000 > 2026 {
            continue;
        }
        let basis = building
            .adjusted_basis
            .checked_sub(building.land_basis)
            .filter(|value| *value >= 0)
            .ok_or("HOME_LAND_ABOVE_BASIS")?;
        let value = building
            .conversion_value
            .checked_sub(building.land_value)
            .filter(|value| *value >= 0)
            .ok_or("HOME_LAND_ABOVE_VALUE")?;
        let rate = crate::depreciation::nonresidential_rate(
            building.first_used / 100 % 100,
            2026 - building.first_used / 10000,
        )
        .ok_or("HOME_DEPRECIATION_HISTORY_UNAVAILABLE")?;
        total += exact::whole(basis.min(value)) * &fraction * exact::ratio(rate, 100000).unwrap();
    }
    amount(&total)
}

fn casualty(
    draft: &FederalDraft,
    site: &HomeOfficeSite,
    claim: &HomeOfficeClaim,
    context: &HomeContext,
) -> Result<[i64; 3], &'static str> {
    let events = crate::home_sites::casualty_events(draft, false).map_err(|issue| match issue {
        crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => key,
        _ => "HOME_CASUALTY_OUTPUT_UNAVAILABLE",
    })?;
    let hypothetical = crate::casualty::all_personal_allowances(
        &events,
        context.modified_agi_cents.get(&site.id).copied().unwrap_or(0),
    )
    .map_err(|_| "HOME_CASUALTY_OUTPUT_UNAVAILABLE")?;
    let mut actual = exact::whole(0);
    let mut actual_by_event = BTreeMap::new();
    let mut otherwise = exact::whole(0);
    let mut hypothetical_whole = 0i64;
    for event_id in &site.casualties {
        let event = events
            .iter()
            .find(|event| event.id == *event_id)
            .ok_or("HOME_CASUALTY_PROPERTY_LINK_REQUIRED")?;
        if !event.source_complete {
            return Err("HOME_CASUALTY_SOURCE_UNAVAILABLE");
        }
        let property = event
            .properties
            .iter()
            .find(|property| property.home_site == Some(site.id))
            .ok_or("HOME_CASUALTY_PROPERTY_LINK_REQUIRED")?;
        let portion = crate::home_sites::property_portions(site, property)?
            .into_iter()
            .find(|portion| portion.owner == claim.owner)
            .ok_or("HOME_CASUALTY_SOURCE_UNAVAILABLE")?;
        let decrease = property
            .business_decrease()
            .require(|| "HOME_CASUALTY_DAMAGE_SOURCE_UNAVAILABLE")?
            .ok_or("HOME_CASUALTY_DAMAGE_SOURCE_UNAVAILABLE")?
            .map(|_| portion.amounts[1] - portion.amounts[2]);
        let [basis, _, _, reimbursement, received] = portion.amounts;
        if received > basis {
            return Err("HOME_CASUALTY_GAIN_CHARACTER_UNAVAILABLE");
        }
        let loss = crate::casualty::home::property_loss(basis, decrease, reimbursement)
            .map_err(|_| "HOME_CASUALTY_OUTPUT_UNAVAILABLE")?;
        actual += exact::whole(loss.loss_cents);
        actual_by_event.insert(event.id, loss.loss_cents);
    }
    // An empty linked-event list establishes zero; unavailable damage records
    // above remain an explicit source failure rather than an inferred loss.
    for row in hypothetical
        .iter()
        .filter(|row| row.home_site == Some(site.id) && site.casualties.contains(&row.event))
    {
        if !row.available {
            return Err("HOME_CASUALTY_SOURCE_UNAVAILABLE");
        }
        let whole_allowance = row
            .qualified_cents
            .checked_add(if context.itemizes.unwrap_or(true) { row.ordinary_cents } else { 0 })
            .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
        hypothetical_whole =
            hypothetical_whole.checked_add(whole_allowance).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
        let business_allowance =
            crate::home_sites::casualty_allowance(site, claim.owner, whole_allowance)?;
        let business_loss =
            actual_by_event.get(&row.event).ok_or("HOME_CASUALTY_PROPERTY_LINK_REQUIRED")?;
        // The hypothetical allowance characterizes an actual business loss.
        // Separately rounded property operands cannot create an extra cent.
        otherwise += exact::whole(business_allowance.min(*business_loss));
    }
    let otherwise = amount(&otherwise)?;
    let actual = amount(&actual)?;
    Ok([otherwise, (actual - otherwise).max(0), hypothetical_whole])
}

pub(super) fn calculate(
    draft: &FederalDraft,
    profits: &BTreeMap<i64, super::HomeIncome>,
    context: &HomeContext,
) -> Result<Vec<SiteCalculation>, (i64, &'static str)> {
    match crate::home_sites::casualty_binding(draft) {
        Ok(None) => {}
        Ok(Some(site)) => return Err((site, "HOME_CASUALTY_PROPERTY_LINK_REQUIRED")),
        Err(_) => return Err((0, "HOME_CASUALTY_HOME_BINDING_INVALID")),
    }
    let mut results = Vec::new();
    for site in &draft.home_sites {
        for claim in &site.claims {
            let result = (|| {
                if claim.qualifies() && draft.income.iter().any(|entry| {
                    entry.owner == crate::DocumentOwner::from(claim.owner)
                        && matches!(&entry.income, FederalIncome::HomeSale { data } if !data.other_property() && crate::HomeSale::from(data).gain() > 0)
                }) {
                    return Err("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE");
                }
                let mut gross = 0i64;
                let mut expenses = 0i64;
                for activity in &claim.activities {
                    let entry = draft
                        .income
                        .iter()
                        .find(|entry| entry.id == activity.activity)
                        .ok_or("HOME_ACTIVITY_SOURCE_UNAVAILABLE")?;
                    let FederalIncome::Business { data } = &entry.income else {
                        return Err("HOME_ACTIVITY_SOURCE_INVALID");
                    };
                    if claim.qualifies() {
                        let mid_quarter = crate::depreciation::mid_quarter(draft);
                        for asset in &data.assets {
                            if asset.disposal != crate::AssetDisposal::No {
                                let sale = asset.sale_for(mid_quarter, false)
                                    .ok_or("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE")?;
                                if sale.recognized() > 0 {
                                    return Err("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE");
                                }
                            }
                            if asset.partial.is_some() {
                                let part = asset.partial_for()
                                    .ok_or("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE")?;
                                if part.gain > 0 {
                                    return Err("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE");
                                }
                            }
                        }
                        if data.sale.as_ref().is_some_and(|sale| !sale.supported() || sale.goodwill > 0) {
                            return Err("HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE");
                        }
                    }
                    let income =
                        profits.get(&entry.id).ok_or("HOME_ACTIVITY_SOURCE_UNAVAILABLE")?;
                    gross = gross
                        .checked_add(ratio::fraction(
                            income.gross_cents,
                            crate::kernel::natural::units(activity.gross_share, 100),
                            10000,
                        ))
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    expenses = expenses
                        .checked_add(ratio::fraction(
                            income.operating_cents,
                            crate::kernel::natural::units(activity.expense_period_share, 100),
                            10000,
                        ))
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    expenses = expenses
                        .checked_add(ratio::fraction(
                            super::losses(draft, data)?,
                            crate::kernel::natural::units(activity.expense_period_share, 100),
                            10000,
                        ))
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                }
                let net_income = gross.checked_sub(expenses).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                let income = net_income.max(0);
                let mut lines = super::Lines {
                    l8: net_income,
                    itemizes: context.itemizes.unwrap_or(true),
                    ..Default::default()
                };
                let mut casualty_parts = [0; 3];
                let mut operating_carry = claim.prior_operating;
                let depreciation;
                let deduction;
                let tiers;
                if !claim.qualifies() || matches!(claim.method, OfficeMethod::Simplified { .. }) {
                    depreciation = 0;
                    deduction = if claim.qualifies() {
                        claim.simplified_base().ok_or("HOME_USE_FRACTION_UNAVAILABLE")?.min(income)
                    } else {
                        0
                    };
                    // Simplified preserves both characterized actual carries.
                    tiers = crate::casualty::home::third_tier(
                        0,
                        0,
                        0,
                        crate::casualty::home::ThirdTierCarry {
                            total_cents: claim
                                .prior_depreciation
                                .checked_add(claim.prior_casualty)
                                .ok_or("HOME_OUTPUT_UNAVAILABLE")?,
                            casualty_cents: Some(claim.prior_casualty),
                        },
                        0,
                    )
                    .map_err(|_| "HOME_OUTPUT_UNAVAILABLE")?;
                } else {
                    let costs =
                        site.business_costs(claim).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
                    let columns =
                        site.cost_columns(claim).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
                    let business_fraction =
                        claim.fraction(site.home_area).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
                    lines.percent = amount(&(exact::whole(10000) * &business_fraction))?;
                    lines.l18 = amount(&columns.indirect[1])?;
                    lines.l19 = amount(&columns.indirect[2])?;
                    lines.l20a = amount(&columns.direct[3])?;
                    lines.l20b = amount(&columns.indirect[3])?;
                    lines.l21 = amount(&columns.indirect[4])?;
                    lines.l22 = amount(&columns.indirect[5])?;

                    casualty_parts = casualty(draft, site, claim, context)?;
                    lines.l9 = casualty_parts[2];
                    lines.l29 = casualty_parts[1];
                    let taxes = costs[0];
                    let all_taxes = draft
                        .deductions
                        .as_ref()
                        .map_or(0, |ded| ded.state_local_taxes())
                        // Beneficiary taxes retain their Schedule A character;
                        // only this home's actual tax supplies the business share
                        // (i1041sk1 box 11 code B; i8829 Line 11 Worksheet).
                        .checked_add(context.estate_salt_cents)
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?
                        .checked_add(
                            ratio::checked_sum(
                                draft
                                    .home_sites
                                    .iter()
                                    .flat_map(|site| &site.costs)
                                    .filter(|cost| matches!(cost.kind, HomeCostKind::PropertyTax))
                                    .map(|cost| cost.amount),
                            )
                            .ok_or("HOME_OUTPUT_UNAVAILABLE")?,
                        )
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    let unlimited_taxes = if context.itemizes.unwrap_or(true) {
                        taxes.min((context.salt_cap_cents - (all_taxes - taxes)).max(0))
                    } else {
                        0
                    };
                    let share =
                        claim.fraction(site.home_area).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?
                            * site
                                .paid_fraction(claim.owner)
                                .ok_or("HOME_PAYMENT_SHARE_UNAVAILABLE")?;
                    let loans =
                        draft.deductions.as_ref().map_or_else(Vec::new, |ded| ded.mortgage_loans().unwrap_or_default());
                    let allowances =
                        crate::mortgage_limit::loan_allowances(&loans, super::separate(draft));
                    let mut qualified = 0i64;
                    let mut regular = 0i64;
                    for loan in allowances
                        .iter()
                        .filter(|loan| site.mortgage_loans.contains(&loan.canonical_id))
                    {
                        qualified = qualified
                            .checked_add(
                                ratio::checked_sum([
                                    loan.qualified_interest_cents,
                                    loan.qualified_points_cents,
                                    loan.qualified_insurance_cents,
                                ])
                                .ok_or("HOME_OUTPUT_UNAVAILABLE")?,
                            )
                            .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                        regular = regular
                            .checked_add(
                                ratio::checked_sum([
                                    loan.regular_interest_cents,
                                    loan.regular_points_cents,
                                    crate::mortgage_limit::insurance_phaseout_cents(
                                        loan.regular_insurance_cents,
                                        ratio::dollars(
                                            context
                                                .modified_agi_cents
                                                .get(&site.id)
                                                .copied()
                                                .unwrap_or(0),
                                        ),
                                        super::separate(draft),
                                    ),
                                ])
                                .ok_or("HOME_OUTPUT_UNAVAILABLE")?,
                            )
                            .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    }
                    let qualified = amount(&(exact::whole(qualified) * &share))?;
                    let regular = if context.itemizes.unwrap_or(true) {
                        amount(&(exact::whole(regular) * share))?
                    } else {
                        0
                    };
                    let first = ratio::checked_sum([unlimited_taxes, regular, casualty_parts[0]])
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    let whole_interest = if business_fraction == exact::whole(0) {
                        0
                    } else {
                        amount(&(exact::whole(regular) / &business_fraction))?
                    };
                    lines.l10 = whole_interest;
                    lines.step1 =
                        all_taxes <= if super::separate(draft) { 500_000 } else { 1_000_000 };
                    lines.w6 = taxes;
                    lines.w7a = all_taxes - taxes;
                    lines.w_magi_test = !lines.step1
                        && lines.w7a < if super::separate(draft) { 2_020_000 } else { 4_040_000 };
                    if context.itemizes.unwrap_or(true) && lines.step1 {
                        lines.l11b = amount(&columns.indirect[0])?;
                        lines.l12a = amount(&columns.direct[0])?;
                    } else if context.itemizes.unwrap_or(true) {
                        lines.l11a = unlimited_taxes;
                        lines.l12a = unlimited_taxes;
                    }
                    lines.l12b = ratio::checked_sum([lines.l9, lines.l10, lines.l11b])
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    lines.l13 = first - lines.l12a;
                    lines.l14 = first;
                    lines.l15 =
                        lines.l8.checked_sub(first).ok_or("HOME_OUTPUT_UNAVAILABLE")?.max(0);
                    lines.l16 = if business_fraction == exact::whole(0) {
                        0
                    } else {
                        amount(&(exact::whole(qualified - regular) / &business_fraction))?
                    };
                    lines.l17 = if context.itemizes.unwrap_or(true) {
                        taxes - unlimited_taxes
                    } else {
                        amount(&columns.indirect[0])?
                    };
                    lines.l23a = amount(&columns.direct[1..].iter().cloned().sum())?
                        .checked_add(if context.itemizes.unwrap_or(true) { lines.l17 } else { 0 })
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    lines.l23b = ratio::checked_sum([
                        amount(&columns.indirect[1..].iter().cloned().sum())?,
                        lines.l16,
                        if context.itemizes.unwrap_or(true) { 0 } else { lines.l17 },
                    ])
                    .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    lines.l24 = amount(&(exact::whole(lines.l23b) * &business_fraction))?;
                    lines.l25 = claim.prior_operating;

                    let room = (income - first).max(0);
                    let operating = ratio::checked_sum(costs[1..].iter().copied())
                        .and_then(|amount| amount.checked_add(taxes - unlimited_taxes))
                        .and_then(|amount| amount.checked_add(qualified - regular))
                        .and_then(|amount| amount.checked_add(claim.prior_operating))
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    let allowed_operating = operating.min(room);
                    operating_carry = operating - allowed_operating;
                    lines.l26 = operating;
                    lines.l27 = allowed_operating;
                    lines.l28 = room - allowed_operating;
                    depreciation = self::depreciation(site, claim)?;
                    if let HomeTenure::Owned { building: Some(building), .. } = &site.tenure {
                        lines.l37 = building.adjusted_basis.min(building.conversion_value);
                        lines.l38 = if building.adjusted_basis <= building.conversion_value {
                            building.land_basis
                        } else {
                            building.land_value
                        };
                        lines.l39 = lines.l37 - lines.l38;
                        lines.l40 = amount(&(exact::whole(lines.l39) * &business_fraction))?;
                        lines.l41 = if building.first_used / 10000 > 2026 {
                            0
                        } else {
                            crate::depreciation::nonresidential_rate(
                                building.first_used / 100 % 100,
                                2026 - building.first_used / 10000,
                            )
                            .ok_or("HOME_DEPRECIATION_HISTORY_UNAVAILABLE")?
                        };
                    }
                    lines.l30 = depreciation;
                    lines.l42 = depreciation;
                    lines.l31 = claim
                        .prior_depreciation
                        .checked_add(claim.prior_casualty)
                        .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    lines.l32 =
                        lines.l30.checked_add(lines.l31).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                    tiers = crate::casualty::home::third_tier(
                        casualty_parts[0],
                        casualty_parts[1],
                        depreciation,
                        crate::casualty::home::ThirdTierCarry {
                            total_cents: claim
                                .prior_depreciation
                                .checked_add(claim.prior_casualty)
                                .ok_or("HOME_OUTPUT_UNAVAILABLE")?,
                            casualty_cents: Some(claim.prior_casualty),
                        },
                        room - allowed_operating,
                    )
                    .map_err(|_| "HOME_OUTPUT_UNAVAILABLE")?;
                    deduction = first + allowed_operating + tiers.total_allowed_cents
                        - tiers
                            .allowed_casualty_cents()
                            .ok_or("HOME_CASUALTY_CHARACTER_UNAVAILABLE")?;
                }
                let _ = (depreciation, casualty_parts);
                lines.l33 = tiers.total_allowed_cents;
                lines.l35 =
                    tiers.allowed_casualty_cents().ok_or("HOME_CASUALTY_CHARACTER_UNAVAILABLE")?;
                lines.l34 = deduction.checked_add(lines.l35).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                lines.l36 = deduction;
                lines.l43 = operating_carry;
                lines.l44 = tiers.carry.total_cents;
                let weights: Vec<_> = claim
                    .activities
                    .iter()
                    .map(|activity| i64::from(activity.deduction_parts))
                    .collect();
                ratio::checked_sum(weights.iter().copied()).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
                let deductions = claim
                    .activities
                    .iter()
                    .map(|activity| activity.activity)
                    .zip(ratio::allocate_remaining(deduction, &weights))
                    .collect();
                Ok(SiteCalculation {
                    site: site.id,
                    owner: claim.owner,
                    deductions,
                    casualty: tiers,
                    gross_cents: gross,
                    lines,
                })
            })()
            .map_err(|key| (site.id, key))?;
            results.push(result);
        }
    }
    Ok(results)
}

pub(super) fn report_lines(
    result: &mut crate::FederalResult,
    calculation: &SiteCalculation,
    site: &HomeOfficeSite,
    claim: &HomeOfficeClaim,
) {
    use crate::kernel::lines::{LineValue, row as calculation_row};
    use crate::line_emission::emit as emit_lines;
    use crate::{FormLine, TaxForm};
    let l = &calculation.lines;
    if !claim.qualifies() || matches!(claim.method, OfficeMethod::Simplified { .. }) {
        emit_lines(
            result,
            &mut (),
            [
                ("deduction", l.l36),
                ("operating_carryover", l.l43),
                ("depreciation_carryover", l.l44),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "home_{}_owner_{}_worksheet_{}",
                        site.id,
                        claim.owner as i32,
                        suffix
                    ),
                    LineValue::MoneyCents(amount),
                    &["home_sites"],
                    "https://www.irs.gov/publications/p587",
                    msg!("EXPLAIN_CANONICAL_HOME_WORKSHEET"),
                ))
            },
        );
        return;
    }
    let percent = crate::MessageArg::Decimal { units: l.percent, scale: 2 };
    let rate = crate::MessageArg::Decimal { units: l.l41, scale: 3 };
    let share_note = match claim.method {
        OfficeMethod::Actual { exclusive_area, shared_daycare_area: 0 } => crate::Message::new(
            "EXPLAIN_BUSINESS_N_FORM_8829_INDIRECT_SHARE",
            alloc::vec![
                crate::MessageArg::Number(i64::from(exclusive_area)),
                crate::MessageArg::Number(i64::from(site.home_area)),
                percent.clone()
            ],
        ),
        _ => msg!("EXPLAIN_CANONICAL_HOME_WORKSHEET"),
    };
    let operating_note = crate::Message::new(
        "EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_SHARE",
        alloc::vec![percent.clone()],
    );
    let basis_note =
        crate::Message::new("EXPLAIN_BUSINESS_N_FORM_8829_BUSINESS_BASIS", alloc::vec![percent]);
    let depreciation_note = crate::Message::new(
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_ALLOWABLE",
        alloc::vec![rate],
    );
    let rows: [(&str, &str, i64, crate::Message); 36] = [
        ("8", "income", l.l8, msg!("EXPLAIN_BUSINESS_N_FORM_8829_INCOME")),
        (
            "10(b)",
            "mortgage_interest",
            l.l10,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_MORTGAGE_INTEREST"),
        ),
        (
            if l.step1 { "11(b)" } else { "11(a)" },
            "real_estate_taxes",
            l.l11a + l.l11b,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_REAL_ESTATE_TAXES"),
        ),
        (
            "12(a)",
            "deductible_direct",
            l.l12a,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEDUCTIBLE_DIRECT"),
        ),
        (
            "12(b)",
            "deductible_indirect",
            l.l12b,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEDUCTIBLE_INDIRECT"),
        ),
        ("13", "indirect_share", l.l13, share_note),
        ("14", "unlimited", l.l14, msg!("EXPLAIN_BUSINESS_N_FORM_8829_UNLIMITED")),
        ("15", "limit", l.l15, msg!("EXPLAIN_BUSINESS_N_FORM_8829_LIMIT")),
        ("16(b)", "excess_interest", l.l16, msg!("EXPLAIN_BUSINESS_N_FORM_8829_EXCESS_INTEREST")),
        (
            if l.itemizes { "17(a)" } else { "17(b)" },
            "excess_taxes",
            l.l17,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_EXCESS_TAXES"),
        ),
        ("18(b)", "insurance", l.l18, msg!("EXPLAIN_BUSINESS_N_FORM_8829_INSURANCE")),
        ("19(b)", "rent", l.l19, msg!("EXPLAIN_BUSINESS_N_FORM_8829_RENT")),
        ("20(a)", "direct_repairs", l.l20a, msg!("EXPLAIN_BUSINESS_N_FORM_8829_DIRECT_REPAIRS")),
        ("20(b)", "repairs", l.l20b, msg!("EXPLAIN_BUSINESS_N_FORM_8829_REPAIRS")),
        ("21(b)", "utilities", l.l21, msg!("EXPLAIN_BUSINESS_N_FORM_8829_UTILITIES")),
        ("22(b)", "other", l.l22, msg!("EXPLAIN_BUSINESS_N_FORM_8829_OTHER")),
        ("23(a)", "direct_total", l.l23a, msg!("EXPLAIN_BUSINESS_N_FORM_8829_DIRECT_TOTAL")),
        ("23(b)", "indirect_total", l.l23b, msg!("EXPLAIN_BUSINESS_N_FORM_8829_INDIRECT_TOTAL")),
        ("24", "operating_share", l.l24, operating_note),
        ("25", "prior_operating", l.l25, msg!("EXPLAIN_BUSINESS_N_FORM_8829_PRIOR_OPERATING")),
        ("26", "operating_total", l.l26, msg!("EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_TOTAL")),
        ("27", "operating_allowed", l.l27, msg!("EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_ALLOWED")),
        (
            "28",
            "depreciation_limit",
            l.l28,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_LIMIT"),
        ),
        ("30", "depreciation", l.l30, msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION")),
        (
            "31",
            "prior_depreciation",
            l.l31,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_PRIOR_DEPRECIATION"),
        ),
        (
            "32",
            "depreciation_total",
            l.l32,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_TOTAL"),
        ),
        (
            "33",
            "depreciation_allowed",
            l.l33,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_ALLOWED"),
        ),
        ("36", "allowed", l.l36, msg!("EXPLAIN_BUSINESS_N_FORM_8829_ALLOWED")),
        ("37", "basis", l.l37, msg!("EXPLAIN_BUSINESS_N_FORM_8829_BASIS")),
        ("38", "land", l.l38, msg!("EXPLAIN_BUSINESS_N_FORM_8829_LAND")),
        ("39", "building", l.l39, msg!("EXPLAIN_BUSINESS_N_FORM_8829_BUILDING")),
        ("40", "business_basis", l.l40, basis_note),
        ("42", "depreciation_allowable", l.l42, depreciation_note),
        (
            "43",
            "operating_carryover",
            l.l43,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_CARRYOVER"),
        ),
        (
            "44",
            "depreciation_carryover",
            l.l44,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_CARRYOVER"),
        ),
        (
            "W6",
            "worksheet_business_taxes",
            l.w6,
            msg!("EXPLAIN_BUSINESS_N_FORM_8829_WORKSHEET_BUSINESS_TAXES"),
        ),
    ];

    let extra = [
        ("9(b)", "casualty_otherwise_allowed", l.l9),
        ("29", "excess_casualty", l.l29),
        ("34", "total", l.l34),
        ("35", "casualty", l.l35),
    ];
    emit_lines(
        result,
        &mut (),
        rows.into_iter().chain(extra.map(|(line, suffix, cents)| {
            (line, suffix, cents, msg!("EXPLAIN_CANONICAL_HOME_WORKSHEET"))
        })),
        |(line, suffix, amount, explanation), _, _| {
            Some(calculation_row(
                (!line.starts_with('W')).then(|| FormLine::new(TaxForm::Form8829, line)),
                &alloc::format!(
                    "home_{}_owner_{}_form_8829_{}",
                    site.id,
                    claim.owner as i32,
                    suffix
                ),
                LineValue::MoneyCents(amount),
                &["home_sites"],
                "https://www.irs.gov/instructions/i8829",
                explanation,
            ))
        },
    );
}
