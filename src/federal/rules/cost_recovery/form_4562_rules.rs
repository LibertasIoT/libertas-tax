// form_4562 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// One asset's current bonus, MACRS, AMT depreciation and remaining-basis rows,
// each tagged with its Form 4562 line when the business or farm rental files
// that form (depreciation::form_4562_line). Shared by Schedule C and F
// businesses and by Form 4835 farm rentals.

pub(super) fn form_4562_asset_lines_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &mut FederalResult,
    id: i64,
    index: usize,
    asset: &crate::BusinessAsset,
    form_4562: bool,
    selected_pair: Option<&[crate::section179_return::Selected; 2]>,
) {
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    let whole_asset = d.income.iter().find_map(|entry| match &entry.income {
        FederalIncome::FarmRental { data } if entry.id == id => data.assets.get(index),
        _ => None,
    });
    let asset = whole_asset.unwrap_or(asset);
    let macrs = form_context.macrs();
    let invocation = d
        .income
        .iter()
        .find(|entry| entry.id == id)
        .map_or_else(crate::macrs_source::AdsInvocation::default, |entry| {
            macrs.context(entry.owner.owned(), Some(id), asset)
        });
    let recovery_unavailable = asset.macrs_source_issue_in_context(invocation).is_some();
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(d, Some(form_context.macrs().as_ref()));
    let (regular, alternative) = match (
        crate::cooperative_basis::source_amounts_with_claims(
            d,
            id,
            asset,
            mid_quarter,
            false,
            source_claims,
            Some(macrs.as_ref()),
        ),
        crate::cooperative_basis::source_amounts_with_claims(
            d,
            id,
            asset,
            mid_quarter,
            true,
            source_claims,
            Some(macrs.as_ref()),
        ),
    ) {
        (Ok(regular), Ok(alternative)) => (regular, alternative),
        (Err(issue), _) | (_, Err(issue)) => {
            crate::section179_return::Issue::Prepared(issue).report(r);
            return;
        }
    };
    let regular_recovery = crate::section179_return::asset_recovery_with_claims(
        d,
        id,
        index,
        asset,
        selected_pair.map(|pair| &pair[0].allocation),
        source_claims,
    );
    let alternative_recovery =
        selected_pair.and_then(|pair| pair[1].allocation.asset_recovery(id, asset.id));
    let regular_deduction = regular.bonus + regular.macrs;
    let alternative_deduction = alternative.bonus + alternative.macrs;
    let annual_classes = d
        .income
        .iter()
        .find_map(|entry| {
            if entry.id != id {
                return None;
            }
            match &entry.income {
                FederalIncome::Business { data } => Some(data.assets.as_slice()),
                FederalIncome::FarmRental { data } => Some(data.assets.as_slice()),
                _ => None,
            }
        })
        .is_some_and(|assets| assets.iter().any(crate::BusinessAsset::nonmacrs));
    for (suffix, amount, authority, explanation) in [
        (
            "bonus",
            regular.bonus,
            "https://www.irs.gov/irb/2026-13_IRB",
            msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_BONUS"),
        ),
        (
            "macrs",
            regular_recovery.as_ref().map_or(regular.macrs, |value| value.allowance.get()),
            "https://www.irs.gov/publications/p946",
            if asset.nonmacrs() {
                asset.nonmacrs_explanation()
            } else {
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_MACRS")
            },
        ),
        (
            "amt_depreciation",
            alternative_recovery
                .as_ref()
                .map_or(alternative_deduction, |value| value.allowance.get()),
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_AMT_DEPRECIATION"),
        ),
        (
            "amt_remaining_basis",
            alternative_recovery
                .as_ref()
                .map_or(alternative.opening - alternative_deduction, |value| {
                    value.closing_basis.get()
                }),
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_AMT_REMAINING_BASIS"),
        ),
        (
            "remaining_basis",
            // The recaptured excess depreciation is added back to the basis.
            regular_recovery
                .as_ref()
                .map_or(regular.opening - regular_deduction, |value| value.closing_basis.get())
                + asset
                    .recapture()
                    .or_else(|| asset.section179_recapture())
                    .map_or(0, |recapture| recapture.excess()),
            "https://www.irs.gov/publications/p946",
            msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_REMAINING_BASIS"),
        ),
    ] {
        if recovery_unavailable && suffix.ends_with("basis") {
            continue;
        }
        let form_line_tag = form_4562
            .then(|| crate::depreciation::form_4562_line_in_context(asset, suffix, invocation))
            .flatten()
            .map(|line| {
                if annual_classes {
                    FormLine::subrow(TaxForm::Form4562, line)
                } else {
                    FormLine::new(TaxForm::Form4562, line)
                }
            });
        push_line(
            r,
            form_line_tag,
            &alloc::format!("business_{}_asset_{}_{}", id, index, suffix),
            if whole_asset.is_some() {
                crate::community::income_amount(d, id, amount)
            } else {
                amount
            },
            &["income.business.assets"],
            authority,
            explanation,
        );
    }
}
pub(crate) fn form_4562_asset_class_lines(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    assets: &[crate::BusinessAsset],
    prefix: &str,
    input: &str,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<(), crate::Section179PreparedIssue> {
    if assets.iter().any(crate::BusinessAsset::nonmacrs)
        && crate::depreciation::assets_need_form_4562(assets)
    {
        let farm_assets = d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::FarmRental { data } if entry.id == id => Some(data.assets.as_slice()),
            _ => None,
        });
        let classes = if let Some(whole) = farm_assets {
            crate::nonmacrs::asset_depreciation_amounts_for(
                d,
                id,
                whole,
                crate::depreciation::mid_quarter_with_macrs(d, macrs),
                false,
                macrs,
            )?
            .classes
        } else if prefix == "rental" {
            crate::nonmacrs::rental_asset_amounts(
                assets,
                crate::nonmacrs::rental_recovery_source_with_macrs(d, id, false, macrs).as_ref(),
                crate::depreciation::mid_quarter_with_macrs(d, macrs),
                false,
            )
            .map_err(|_| crate::Section179PreparedIssue::AmountOverflow(id))?
        } else {
            crate::nonmacrs::asset_depreciation_amounts_for(
                d,
                id,
                assets,
                crate::depreciation::mid_quarter_with_macrs(d, macrs),
                false,
                macrs,
            )?
            .classes
        };
        for (class, amount) in &classes {
            if *amount != crate::kernel::ratio::exact::whole(0) {
                push_line(
                    r,
                    Some(FormLine::new(TaxForm::Form4562, class)),
                    &alloc::format!(
                        "{prefix}_{id}_depreciation_class_{}",
                        class.to_ascii_uppercase()
                    ),
                    {
                        let amount = crate::kernel::ratio::exact::dollars(amount)
                            .and_then(|amount| amount.checked_mul(100))
                            .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
                        if farm_assets.is_some() {
                            crate::community::income_amount(d, id, amount)
                        } else {
                            amount
                        }
                    },
                    &[input],
                    "https://www.irs.gov/instructions/i4562",
                    msg!("EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS"),
                );
            }
        }
    }
    Ok(())
}
// One business's Form 4562 line 12 (its allowed section 179 deduction) and
// line 22 (the total that Schedule C line 13 or Schedule F line 14 deducts:
// bonus, MACRS and the allowed section 179).

pub(super) fn form_4562_totals_with_section179_with_claims(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    assets: &[crate::BusinessAsset],
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<(), crate::Section179PreparedIssue> {
    form_4562_asset_class_lines(
        d,
        r,
        id,
        assets,
        "business",
        "income.business.assets",
        home.and_then(|home| home.macrs.as_deref()),
    )?;
    let section179 = crate::section179_return::allowed(d, id, selected, home)?;
    let total = business_depreciation_with_section179_with_claims(
        d,
        id,
        assets,
        selected,
        home,
        source_claims,
    )?;
    emit_lines(
        r,
        &mut (),
        [
            ("12", "section179_deduction", section179, msg!("EXPLAIN_BUSINESS_N_N_N_7")),
            ("22", "depreciation_total", total, msg!("EXPLAIN_BUSINESS_N_N_N_8")),
        ],
        |(line, suffix, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4562, line)),
                &alloc::format!("business_{id}_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.business.assets"],
                "https://www.irs.gov/instructions/i4562",
                explanation,
            ))
        },
    );
    Ok(())
}
// Current bonus, MACRS and allowed section 179 of one business: Form 4562
// line 22, Schedule C line 13, Schedule F line 14.

pub(crate) fn business_depreciation_with_section179_with_claims(
    d: &FederalDraft,
    id: i64,
    assets: &[crate::BusinessAsset],
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let mq =
        crate::depreciation::mid_quarter_with_macrs(d, home.and_then(|home| home.macrs.as_deref()));
    let depreciation = if assets.iter().any(crate::BusinessAsset::nonmacrs) {
        if let Some(whole) = d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::FarmRental { data } if entry.id == id => Some(data),
            _ => None,
        }) {
            let amount = crate::nonmacrs::asset_depreciation_amounts_for(
                d,
                id,
                &whole.assets,
                mq,
                false,
                home.and_then(|home| home.macrs.as_deref()),
            )?
            .expense_cents(0)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
            crate::community::farm_rental_amount(d, id, |_| amount).unwrap_or(amount)
        } else {
            crate::nonmacrs::asset_depreciation_amounts_for(
                d,
                id,
                assets,
                mq,
                false,
                home.and_then(|home| home.macrs.as_deref()),
            )?
            .expense_cents(0)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?
        }
    } else if let Some(whole) = d.income.iter().find_map(|entry| match &entry.income {
        FederalIncome::FarmRental { data } if entry.id == id => Some(data),
        _ => None,
    }) {
        let amount = crate::cooperative_basis::depreciation_with_claims(
            d,
            id,
            &whole.assets,
            mq,
            false,
            source_claims,
            home.and_then(|home| home.macrs.as_deref()),
        )?;
        match crate::community::farm_rental_amount(d, id, |_| amount) {
            Some(amount) => amount,
            None => crate::cooperative_basis::depreciation_with_claims(
                d,
                id,
                assets,
                mq,
                false,
                source_claims,
                home.and_then(|home| home.macrs.as_deref()),
            )?,
        }
    } else {
        crate::cooperative_basis::depreciation_with_claims(
            d,
            id,
            assets,
            mq,
            false,
            source_claims,
            home.and_then(|home| home.macrs.as_deref()),
        )?
    };
    let mut depreciation = depreciation;
    for (index, asset) in assets.iter().enumerate() {
        if let Some(recovery) = crate::section179_return::asset_recovery_with_claims(
            d,
            id,
            index,
            asset,
            selected,
            source_claims,
        ) {
            depreciation = depreciation
                .checked_sub(asset.deduction(mq))
                .and_then(|value| value.checked_add(recovery.allowance.get()))
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
        }
    }
    depreciation
        .checked_add(crate::section179_return::allowed(d, id, selected, home)?)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))
}
// Form 4562 Part V Section A columns for one actual-expense vehicle: line 25
// (special allowance), then line 26 (more than 50% qualified business use) or
// line 27 (50% or less, ADS straight line).
pub(super) fn vehicle_columns(
    r: &mut FederalResult,
    id: i64,
    index: usize,
    assets: &[crate::BusinessAsset],
    mileage: usize,
    mid_quarter: bool,
    recovery: Option<&crate::kernel::cost_recovery::annual::Amounts>,
) {
    let asset = &assets[index];
    let row = if asset.predominant() { "26" } else { "27" };
    let inputs = ["income.business.assets"];
    let source = "https://www.irs.gov/instructions/i4562";
    let bonus = asset.bonus_amount();
    let crate::ListedProperty::Vehicle { kind, .. } = asset.listed else {
        unreachable!("listed vehicle columns require a vehicle")
    };
    let property_type = match kind {
        crate::VehicleKind::PassengerCar => "PRINT_VEHICLE_PASSENGER_CAR",
        crate::VehicleKind::PassengerTruckVan => "PRINT_VEHICLE_PASSENGER_TRUCK_VAN",
        crate::VehicleKind::HeavySuv { .. } => "PRINT_VEHICLE_HEAVY_SUV",
        crate::VehicleKind::HeavyTruckVan => "PRINT_VEHICLE_HEAVY_TRUCK_VAN",
        crate::VehicleKind::QualifiedNonpersonal => {
            unreachable!("qualified nonpersonal vehicles are not listed property")
        }
    };
    emit_lines(
        r,
        &mut (),
        [
            (
                "type",
                "a",
                LineValue::Text,
                Message::new(
                    "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_TYPE",
                    alloc::vec![
                        crate::BusinessAsset::display_arg(assets, index, mileage),
                        crate::MessageArg::Resource(property_type.into()),
                    ],
                ),
            ),
            (
                "business_use",
                "c",
                LineValue::MoneyCents(asset.business_use_hundredths()),
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_BUSINESS_USE"),
            ),
            (
                "cost",
                "d",
                LineValue::MoneyCents(asset.cost - asset.credit_reduction),
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_COST"),
            ),
            (
                "depreciation_basis",
                "e",
                LineValue::MoneyCents(recovery.map_or_else(
                    || asset.opening_basis() - bonus,
                    |value| value.original_depreciation_basis.get(),
                )),
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_DEPRECIATION_BASIS"),
            ),
            (
                "depreciation",
                "h",
                LineValue::MoneyCents(recovery.map_or_else(
                    || asset.macrs_amount(mid_quarter),
                    |value| value.allowance.get(),
                )),
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_DEPRECIATION"),
            ),
            (
                "section179",
                "i",
                LineValue::MoneyCents(asset.elected179()),
                msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_SECTION179"),
            ),
        ],
        |(suffix, column, value, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form4562, &alloc::format!("{row}({column})"))),
                &alloc::format!("business_{id}_asset_{index}_listed_{suffix}"),
                value,
                &inputs,
                source,
                explanation,
            ))
        },
    );
    // Form 4797 Part IV: the vehicle's business use fell to 50% or less, so the
    // depreciation above what straight-line ADS would have allowed is income
    // (Schedule C line 6, which carries the excess in its other income).
    if let Some(recapture) = asset.recapture() {
        emit_lines(
            r,
            &mut (),
            [
                (
                    "35(b)",
                    "prior_depreciation",
                    recapture.allowed,
                    msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_PRIOR_DEPRECIATION"),
                ),
                (
                    "36(b)",
                    "recomputed",
                    recapture.recomputed,
                    msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECOMPUTED"),
                ),
                (
                    "37(b)",
                    "recapture",
                    recapture.excess(),
                    msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECAPTURE"),
                ),
            ],
            |(line, suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form4797, line)),
                    &alloc::format!("business_{id}_asset_{index}_listed_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &inputs,
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                ))
            },
        );
    }
    if bonus > 0 {
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form4562, "25")),
            &alloc::format!("business_{id}_asset_{index}_listed_special_allowance"),
            bonus,
            &inputs,
            source,
            msg!("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_SPECIAL_ALLOWANCE"),
        );
    }
}
