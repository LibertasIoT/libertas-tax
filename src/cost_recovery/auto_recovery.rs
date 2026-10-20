// Coverage checkpoint: documented original passenger-auto schedules, including
// Rev.Proc.2019-13 and unrecovered basis after the original recovery period.
// Original ordinary/electric schedules use retained annual source records, never a new clock.
// Ordinary original-service caps cover1987 onward; changed-use recapture remains separate.
// Sources: Rev.Proc.2019-13 sections3–4; Pub.463 chapter4; Pub.946 AppendixA.
#[path = "auto_recovery/caps.rs"]
pub(crate) mod caps;

use crate::kernel::{
    carry::{Amount, Cents},
    cost_recovery::{Month, annual, gds},
    source_fact::SourceFact,
};
use crate::{
    AssetConvention, AssetDisposal, BusinessAsset, EquipmentAmtHistory, EquipmentClass,
    EquipmentHistory, EquipmentMethod, EquipmentSchedule, HistoricalAutoKind, ListedProperty,
    OriginalAssetMethod, OriginalBonusTreatment, UseFraction,
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
    let EquipmentHistory::Continuing {
        initial_section179,
        carryforward: 0,
        prior_depreciation,
        amt: EquipmentAmtHistory::Same,
        mid_quarter,
        schedule: EquipmentSchedule::Tables { initial_bonus },
    } = asset.history
    else {
        return None;
    };
    let ListedProperty::Vehicle { total_miles, business_miles, passenger_automobile: true, .. } =
        &asset.listed
    else {
        return None;
    };
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
    let method_matches = match records.original_method {
        OriginalAssetMethod::SafeHarbor2019 => safe,
        OriginalAssetMethod::Declining200 => !safe && asset.method == EquipmentMethod::Regular,
        OriginalAssetMethod::Declining150 => !safe && asset.method == EquipmentMethod::Declining150,
        OriginalAssetMethod::StraightLine => !safe && asset.method == EquipmentMethod::StraightLine,
        OriginalAssetMethod::OtherDocumented => false,
    };
    // The source cap table admits its documented ordinary and electric-car
    // periods. Current-year new autos retain EquipmentHistory::New.
    if placed_year >= 2026
        || !(placed_year..=2026).contains(&year)
        || asset.acquired > asset.placed
        || !asset.vehicle_valid()
        || !asset.predominant()
        || asset.class != EquipmentClass::Five
        || asset.disposal != AssetDisposal::No
        || asset.earlier_business_percent.is_some()
        || asset.partial.is_some()
        || !kind_matches
        || !method_matches
        || records.original_business_basis
            != asset.business_share_of(asset.cost - asset.credit_reduction)
        || records.original_section179_elected != initial_section179
        || records.original_bonus != initial_bonus
        || !records.carryovers.is_empty()
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
    let bonus_percent = match auto.first_year_bonus {
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
    };
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
        if !use_matches(row.income_use)
            || !use_matches(row.qualified_use)
            || row.basis_increase != 0
            || row.basis_decrease != 0
            || row.recaptured != 0
            || row.amt != row.regular
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
            method: match asset.method {
                EquipmentMethod::Regular => gds::Method::Regular,
                EquipmentMethod::Declining150 => gds::Method::Declining150,
                EquipmentMethod::StraightLine => gds::Method::StraightLine,
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
        bonus_percent,
        safe_harbor: safe,
        first_year_deduction: Amount::new(auto.first_year_capped_deduction).ok()?,
        original_bonus: Amount::new(initial_bonus).ok()?,
        records: &years,
        prior_depreciation: Amount::new(prior_depreciation).ok()?,
        post_recovery_basis: Amount::new(auto.post_recovery_basis).ok()?,
    };
    let cap = |tax_year| {
        crate::year_constants::original_auto_limit_cents(
            asset.acquired,
            asset.placed,
            tax_year,
            &auto.car_or_truck,
            bonus_percent > 0,
        )
        .and_then(|value| Amount::new(asset.business_share_of(value)).ok())
    };
    match annual::calculate(SourceFact::Reported(schedule), cap) {
        SourceFact::Reported(amounts) => Some(amounts.allowance.get()),
        SourceFact::NotReported | SourceFact::Unavailable => None,
    }
}

// Reuse the admitted original passenger-auto cap table after a current basis
// adjustment. The caller has already selected the actual regular/AMT history.
pub(crate) fn original_cap(asset: &BusinessAsset, year: u32) -> Option<i64> {
    let auto = asset.history_details.as_ref()?.auto_history.as_ref()?;
    crate::year_constants::original_auto_limit_cents(
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
