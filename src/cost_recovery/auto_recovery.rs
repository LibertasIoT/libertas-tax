// Coverage checkpoint: documented original passenger-auto schedules, including
// Rev.Proc.2019-13 and unrecovered basis after the original recovery period.
// Original ordinary/electric schedules use retained annual source records, never a new clock.
// Ordinary original-service caps cover1987 onward; changed-use recapture remains separate.
// Actual annual179 releases retain their reserved basis and share each280F cap
// with selected current179. Independent AMT179 uses actual annual source facts;
// its capped bonus uses the same annual owner. Disposal restores unused179 once.
// Remaining changed-use/basis regimes retain their named source boundaries.
// Sources: Rev.Proc.2019-13 sections3–4; Pub.463 chapter4; Pub.946 AppendixA.
#[path = "auto_recovery/caps.rs"]
pub(crate) mod caps;

use crate::AssetDisposal;
use crate::asset_model::{
    AssetConvention, BusinessAsset, EquipmentAmtHistory, EquipmentClass, EquipmentHistory,
    EquipmentMethod, EquipmentSchedule, HistoricalAutoKind, ListedProperty, OriginalAssetMethod,
    OriginalBonusTreatment, UseFraction,
};
use crate::kernel::{
    carry::{Amount, Cents},
    cost_recovery::{Month, annual, gds},
    source_fact::SourceFact,
};

// Cross-row reconciliation belongs to the annual owner. Point to the submitted
// history list rather than guessing which row or prior total should be changed.
pub(crate) fn history_issue(asset: &BusinessAsset) -> Option<crate::validation::ValidationIssue> {
    (asset.history_details.as_ref().is_some_and(|records| records.auto_history.is_some())
        && !asset.valid())
    .then(|| {
        crate::validation::ValidationIssue::new("AUTO_RECOVERY_HISTORY_INCONSISTENT")
            .at_field("years")
            .in_field("history_details")
    })
}

// A source-year parameter verifies published annual schedules. The return calls
// this owner only with2026; no return-wide year or stored source date changes.
pub(crate) fn amount(asset: &BusinessAsset, year: u32) -> Option<i64> {
    let result = recovery(asset, year, 0)?;
    Some(result.allowance.get() + result.section179.get())
}

pub(crate) fn recovery(
    asset: &BusinessAsset,
    year: u32,
    selected179: i64,
) -> Option<annual::Amounts> {
    recovery_in_system(asset, year, selected179, false)
}
// ADS passenger autos retain this owner's actual annual caps and original
// records. The caller establishes the statutory five-year ADS period.
pub(crate) fn ads_amount(asset: &BusinessAsset, year: u32) -> Option<i64> {
    let result = recovery_in_system(asset, year, 0, true)?;
    Some(result.allowance.get() + result.section179.get())
}
fn recovery_in_system(
    asset: &BusinessAsset,
    year: u32,
    selected179: i64,
    ads: bool,
) -> Option<annual::Amounts> {
    let EquipmentHistory::Continuing {
        initial_section179,
        carryforward,
        prior_depreciation,
        amt,
        mid_quarter,
        schedule: EquipmentSchedule::Tables { initial_bonus },
    } = asset.history
    else {
        return None;
    };
    if matches!(amt, EquipmentAmtHistory::Separate { .. })
        && !(asset.acquired < 20170928 && asset.method == EquipmentMethod::Regular)
    {
        return None;
    }
    let ListedProperty::Vehicle { total_miles, business_miles, kind, .. } = &asset.listed else {
        return None;
    };
    if !kind.passenger_automobile() {
        return None;
    }
    let records = asset.history_details.as_ref()?;
    let auto = records.auto_history.as_ref()?;
    if initial_section179 < 0
        || prior_depreciation < 0
        || initial_bonus < 0
        || records.original_business_basis < 0
        || auto.first_year_capped_deduction < 0
        || initial_section179 > records.original_business_basis
    {
        return None;
    }
    let placed_year = asset.placed / 10000;
    let use_matches = |fraction: UseFraction| {
        fraction.total > 0
            && fraction.used <= fraction.total
            && u64::from(fraction.used) * u64::from(*total_miles)
                == u64::from(*business_miles) * u64::from(fraction.total)
    };
    // The saved ordinary passenger classification establishes section280F;
    // the original schedule supplies its actual car/truck cap-table fact.
    let kind_matches = match auto.car_or_truck {
        HistoricalAutoKind::Car | HistoricalAutoKind::TruckVan => true,
        HistoricalAutoKind::ElectricCar => (19970806..=20061231).contains(&asset.placed),
    };
    let safe = auto.safe_harbor_adopted_year.is_some();
    let method_matches = if ads {
        records.original_method == OriginalAssetMethod::StraightLine && !safe
    } else {
        match records.original_method {
            OriginalAssetMethod::SafeHarbor2019 => safe,
            OriginalAssetMethod::Declining200 => !safe && asset.method == EquipmentMethod::Regular,
            OriginalAssetMethod::Declining150 => {
                !safe && asset.method == EquipmentMethod::Declining150
            }
            OriginalAssetMethod::StraightLine => {
                !safe && asset.method == EquipmentMethod::StraightLine
            }
            OriginalAssetMethod::OtherDocumented => false,
        }
    };
    // The source cap table admits its documented ordinary and electric-car
    // periods. Current-year new autos retain EquipmentHistory::New.
    if placed_year >= 2026
        || !(placed_year..=2026).contains(&year)
        || asset.acquired > asset.placed
        || !asset.vehicle_valid()
        || !asset.predominant()
        || asset.class != EquipmentClass::Five
        || (asset.disposal != AssetDisposal::No && !asset.disposal.valid())
        || asset.earlier_business_percent.is_some()
        || asset.partial.is_some()
        || !kind_matches
        || !method_matches
        || records.original_business_basis
            != asset.business_share_of(asset.cost - asset.credit_reduction)
        || records.original_section179_elected != initial_section179
        || records.original_bonus != initial_bonus
        || if carryforward == 0 {
            !records.carryovers.is_empty()
        } else {
            records.carryovers.len() != 1
                || records.carryovers[0].year != placed_year
                || records.carryovers[0].amount != carryforward
        }
        || !use_matches(records.original_income_use)
        || !use_matches(records.original_qualified_use)
        || !matches!(
            (&records.convention, mid_quarter),
            (AssetConvention::HalfYear, false) | (AssetConvention::MidQuarter, true)
        )
        || records.years.len() != (2026 - placed_year) as usize
        || auto.post_recovery_basis < 0
    {
        return None;
    }
    let bonus_percent = bonus_percent(asset, auto)?;
    if safe
        && (bonus_percent != 100
            || !(2018..=2022).contains(&placed_year)
            || initial_section179 != 0
            || auto.safe_harbor_adopted_year != Some(placed_year + 1))
    {
        return None;
    }
    let mut years = alloc::vec::Vec::with_capacity(records.years.len());
    for row in &records.years {
        if row.amt < 0
            || row.amt_section179_allowed.source_fact().is_err()
            || matches!(row.amt_section179_allowed, crate::SourceCents::Reported { amount } if amount > initial_section179)
            || !use_matches(row.income_use)
            || !use_matches(row.qualified_use)
            || row.basis_increase != 0
            || row.basis_decrease != 0
            || row.recaptured != 0
        {
            return None;
        }
        years.push(annual::Record {
            year: row.year,
            section179: Amount::<Cents>::new(row.section179_allowed).ok()?,
            bonus: Amount::new(row.bonus).ok()?,
            regular: Amount::new(row.regular).ok()?,
        });
    }
    let schedule = annual::Schedule {
        table: gds::Table {
            placed: Month { year: placed_year, month: asset.placed / 100 % 100 },
            period: gds::Period::Five,
            method: if ads {
                gds::Method::StraightLine
            } else {
                match asset.method {
                    EquipmentMethod::Regular => gds::Method::Regular,
                    EquipmentMethod::Declining150 => gds::Method::Declining150,
                    EquipmentMethod::StraightLine => gds::Method::StraightLine,
                }
            },
            convention: if mid_quarter {
                gds::Convention::MidQuarter
            } else {
                gds::Convention::HalfYear
            },
        },
        target_year: year,
        record_end_year: 2026,
        basis: Amount::new(records.original_business_basis).ok()?,
        original_section179: Amount::new(initial_section179).ok()?,
        opening_section179_carry: Amount::new(carryforward).ok()?,
        current_section179: Amount::new(selected179).ok()?,
        disposed: asset
            .disposal_date()
            .map(|date| Month { year: date / 10000, month: date / 100 % 100 }),
        bonus_percent,
        safe_harbor: safe,
        first_year_deduction: Amount::new(auto.first_year_capped_deduction).ok()?,
        original_bonus: Amount::new(initial_bonus).ok()?,
        records: &years,
        prior_depreciation: Amount::new(prior_depreciation).ok()?,
        post_recovery_basis: Amount::new(auto.post_recovery_basis).ok()?,
    };
    let cap = |tax_year| {
        crate::auto_recovery::caps::original_auto_limit_cents(
            asset.acquired,
            asset.placed,
            tax_year,
            &auto.car_or_truck,
            bonus_percent > 0,
        )
        .and_then(|value| Amount::new(asset.business_share_of(value)).ok())
    };
    match annual::calculate(SourceFact::Reported(schedule), cap) {
        SourceFact::Reported(amounts) => Some(amounts),
        SourceFact::NotReported | SourceFact::Unavailable => None,
    }
}

// Reuse the admitted original passenger-auto cap table after a current basis
// adjustment. The caller has already selected the actual regular/AMT history.
pub(crate) fn original_cap(asset: &BusinessAsset, year: u32) -> Option<i64> {
    let auto = asset.history_details.as_ref()?.auto_history.as_ref()?;
    crate::auto_recovery::caps::original_auto_limit_cents(
        asset.acquired,
        asset.placed,
        year,
        &auto.car_or_truck,
        !matches!(
            auto.first_year_bonus,
            OriginalBonusTreatment::ElectedOut | OriginalBonusTreatment::Ineligible
        ),
    )
    .map(|cap| asset.business_share_of(cap))
}

pub(crate) fn purchase_post_recovery_amount(
    asset: &BusinessAsset,
    year: u32,
    reduction: i64,
) -> Option<i64> {
    let native = amount(asset, year)?;
    let EquipmentHistory::Continuing { mid_quarter, .. } = asset.history else { return None };
    let post_year = (asset.placed / 10000)
        .checked_add(u32::try_from(asset.table_rates(mid_quarter).len()).ok()?)?;
    if year < post_year {
        return None;
    }
    // The annual owner already admitted the post-recovery original cap. Only
    // its available current basis changes; actual earlier rows stay intact.
    let available = asset.opening_basis().checked_sub(reduction)?;
    (available >= 0).then(|| crate::kernel::phase::cap(native, [available]))
}

// The retained annual AMT amounts establish their own179 releases. No equality
// of MACRS methods establishes equality of independently income-limited179.
pub(crate) fn amt_history(asset: &BusinessAsset) -> Option<(i64, i64)> {
    let EquipmentHistory::Continuing {
        initial_section179, carryforward, prior_depreciation, ..
    } = asset.history
    else {
        return None;
    };
    let records = asset.history_details.as_ref()?;
    records.auto_history.as_ref()?;
    let mut regular179 = 0i64;
    let mut alternative179 = 0i64;
    let mut regular_prior = 0i64;
    let mut alternative_prior = 0i64;
    for row in &records.years {
        let alternative = if initial_section179 == 0 {
            0
        } else {
            match row.amt_section179_allowed.source_fact().ok()? {
                SourceFact::Reported(value) => value,
                SourceFact::NotReported => 0,
                SourceFact::Unavailable => return None,
            }
        };
        regular179 = regular179.checked_add(row.section179_allowed)?;
        alternative179 = alternative179.checked_add(alternative)?;
        regular_prior = regular_prior
            .checked_add(row.section179_allowed)?
            .checked_add(row.bonus)?
            .checked_add(row.regular)?;
        let bonus =
            if row.year == asset.placed / 10000 { amt_bonus(asset, alternative)? } else { 0 };
        alternative_prior =
            alternative_prior.checked_add(alternative)?.checked_add(bonus)?.checked_add(row.amt)?;
    }
    if regular179.checked_add(carryforward)? != initial_section179
        || regular_prior != prior_depreciation
        || !(0..=initial_section179).contains(&alternative179)
    {
        return None;
    }
    Some((initial_section179 - alternative179, alternative_prior))
}

pub(crate) fn amt_projection(asset: &BusinessAsset) -> Option<BusinessAsset> {
    let (carry, prior) = amt_history(asset)?;
    if let EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Separate { schedule, .. },
        ..
    } = asset.history
    {
        let EquipmentSchedule::Tables { initial_bonus } = schedule else {
            return None;
        };
        let first = asset
            .history_details
            .as_ref()?
            .years
            .iter()
            .find(|row| row.year == asset.placed / 10000)?;
        let allowed = match first.amt_section179_allowed.source_fact().ok()? {
            SourceFact::Reported(value) => value,
            SourceFact::NotReported => 0,
            SourceFact::Unavailable
                if asset.history_details.as_ref()?.original_section179_elected == 0 =>
            {
                0
            }
            SourceFact::Unavailable => return None,
        };
        if initial_bonus != amt_bonus(asset, allowed)? {
            return None;
        }
    }
    let mut projected = asset.clone();
    let records = projected.history_details.as_mut()?;
    let separate = matches!(
        asset.history,
        EquipmentHistory::Continuing { amt: EquipmentAmtHistory::Separate { .. }, .. }
    );
    if separate {
        projected.method = EquipmentMethod::Declining150;
        records.original_method = OriginalAssetMethod::Declining150;
    }
    for row in &mut records.years {
        row.section179_allowed = if records.original_section179_elected == 0 {
            0
        } else {
            match row.amt_section179_allowed.source_fact().ok()? {
                SourceFact::Reported(value) => value,
                SourceFact::NotReported => 0,
                SourceFact::Unavailable => return None,
            }
        };
        row.bonus = if row.year == asset.placed / 10000 {
            amt_bonus(asset, row.section179_allowed)?
        } else {
            0
        };
        row.regular = row.amt;
    }
    let auto = records.auto_history.as_mut()?;
    let first = records.years.iter().find(|row| row.year == asset.placed / 10000)?;
    records.original_bonus = first.bonus;
    auto.first_year_capped_deduction =
        first.section179_allowed.checked_add(first.bonus)?.checked_add(first.regular)?;
    // Post-recovery basis is an independently reconciled depreciation basis.
    // When the actual AMT source differs, derive it from its actual annual rows.
    let rates = gds::Table {
        placed: Month { year: asset.placed / 10000, month: asset.placed / 100 % 100 },
        period: gds::Period::Five,
        method: if separate {
            gds::Method::Declining150
        } else {
            match asset.method {
                EquipmentMethod::Regular => gds::Method::Regular,
                EquipmentMethod::Declining150 => gds::Method::Declining150,
                EquipmentMethod::StraightLine => gds::Method::StraightLine,
            }
        },
        convention: if records.convention == AssetConvention::MidQuarter {
            gds::Convention::MidQuarter
        } else {
            gds::Convention::HalfYear
        },
    }
    .rates();
    let post_year = asset.placed / 10000 + rates.len() as u32;
    if post_year <= 2026 {
        let mut used = 0i64;
        let mut released = 0i64;
        for row in records.years.iter().filter(|row| row.year < post_year) {
            used = used
                .checked_add(row.section179_allowed)?
                .checked_add(row.bonus)?
                .checked_add(row.regular)?;
            released = released.checked_add(row.section179_allowed)?;
        }
        auto.post_recovery_basis = records
            .original_business_basis
            .checked_sub(used)?
            .checked_sub(records.original_section179_elected.checked_sub(released)?)?;
    }
    records.carryovers = if carry == 0 {
        alloc::vec![]
    } else {
        alloc::vec![crate::Asset179Carryover { year: asset.placed / 10000, amount: carry }]
    };
    if let EquipmentHistory::Continuing {
        carryforward, prior_depreciation, amt, schedule, ..
    } = &mut projected.history
    {
        if let EquipmentAmtHistory::Separate { prior_depreciation: supplied, .. } = amt
            && *supplied != prior
        {
            return None;
        }
        *carryforward = carry;
        *prior_depreciation = prior;
        *amt = EquipmentAmtHistory::Same;
        *schedule = EquipmentSchedule::Tables { initial_bonus: records.original_bonus };
    }
    projected.amt_section179_income_carry = None;
    Some(projected)
}

// New annual179 releases and any independently different AMT carry need the
// selected-system coordinator, including when the regular carry was exhausted.
pub(crate) fn needs_selected(asset: &BusinessAsset) -> bool {
    let EquipmentHistory::Continuing { initial_section179, carryforward, .. } = asset.history
    else {
        return false;
    };
    if asset.history_details.as_ref().is_none_or(|records| records.auto_history.is_none())
        || initial_section179 == 0
    {
        return false;
    }
    let Some((alternative, _)) = amt_history(asset) else {
        return true;
    };
    if asset.documented_179_reserve() && alternative == carryforward {
        return false;
    }
    carryforward > 0
        || alternative != 0
        || asset.amt_section179_income_carry.is_some_and(|actual| actual != alternative)
}

fn bonus_percent(asset: &BusinessAsset, auto: &crate::AutoRecoveryHistory) -> Option<u32> {
    let placed_year = asset.placed / 10000;
    Some(match auto.first_year_bonus {
        OriginalBonusTreatment::HundredPercent
            if placed_year >= 2018
                && asset.acquired >= 20170928
                && (placed_year <= 2022 || asset.acquired >= 20250120) =>
        {
            100
        }
        OriginalBonusTreatment::Partial if asset.acquired >= 20170928 => match placed_year {
            2023 => 80,
            2024 => 60,
            2025 if asset.acquired < 20250120 => 40,
            _ => return None,
        },
        OriginalBonusTreatment::ElectedOut | OriginalBonusTreatment::Ineligible => 0,
        _ => return None,
    })
}

fn amt_bonus(asset: &BusinessAsset, allowed179: i64) -> Option<i64> {
    let records = asset.history_details.as_ref()?;
    let auto = records.auto_history.as_ref()?;
    let cap = crate::auto_recovery::caps::original_auto_limit_cents(
        asset.acquired,
        asset.placed,
        asset.placed / 10000,
        &auto.car_or_truck,
        bonus_percent(asset, auto)? > 0,
    )?;
    Some(
        annual::first_bonus(
            &Amount::new(records.original_business_basis).ok()?,
            &Amount::new(records.original_section179_elected).ok()?,
            bonus_percent(asset, auto)?,
            Amount::new(asset.business_share_of(cap)).ok()?,
            Amount::new(allowed179).ok()?,
        )?
        .get(),
    )
}

pub(crate) fn valid_amt_records(asset: &BusinessAsset) -> bool {
    let Some(records) =
        asset.history_details.as_ref().filter(|records| records.auto_history.is_some())
    else {
        return true;
    };
    let mut reported = 0i64;
    let mut complete = true;
    for row in &records.years {
        match row.amt_section179_allowed.source_fact() {
            Ok(SourceFact::Reported(value)) => {
                let Some(total) = reported.checked_add(value) else {
                    return false;
                };
                reported = total;
            }
            Ok(SourceFact::NotReported) => {}
            Ok(SourceFact::Unavailable) => complete = false,
            Err(_) => return false,
        }
    }
    if reported > records.original_section179_elected {
        return false;
    }
    !complete
        || amt_projection(asset).is_some_and(|projection| recovery(&projection, 2026, 0).is_some())
}
