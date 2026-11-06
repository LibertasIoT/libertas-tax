use crate::building_model::ImprovementBuildingFacts;
use alloc::vec::Vec;

// One borrowed adapter over the actual source carriers; accepted Save assigns
// their IDs with the same return-wide allocator as equipment and cost records.
pub(crate) fn improvement_buildings(income: &crate::FederalIncome) -> &[ImprovementBuildingFacts] {
    match income {
        crate::FederalIncome::Business { data } => &data.improvement_buildings,
        crate::FederalIncome::FarmRental { data } => &data.improvement_buildings,
        crate::FederalIncome::Rental { data } => &data.improvement_buildings,
        _ => &[],
    }
}
pub(crate) fn improvement_building_ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    let rows = match income {
        crate::FederalIncome::Business { data } => &mut data.improvement_buildings,
        crate::FederalIncome::FarmRental { data } => &mut data.improvement_buildings,
        crate::FederalIncome::Rental { data } => &mut data.improvement_buildings,
        _ => return Vec::new(),
    };
    rows.iter_mut().map(|row| &mut row.id).collect()
}
pub(crate) use crate::building_model::building_facts_issue;
