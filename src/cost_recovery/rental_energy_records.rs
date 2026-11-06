use crate::FederalIncome;
use alloc::vec::Vec;

pub(crate) fn ids(income: &FederalIncome) -> Vec<i64> {
    match income {
        FederalIncome::Rental { data } => data
            .energy_buildings
            .iter()
            .map(|b| b.id)
            .chain(data.buildings.iter().map(|b| b.id))
            .collect(),
        _ => Vec::new(),
    }
}
pub(crate) fn ids_mut(income: &mut FederalIncome) -> Vec<&mut i64> {
    match income {
        FederalIncome::Rental { data } => {
            let associated = !data.energy_buildings.is_empty()
                || data.buildings.iter().any(|cost| cost.energy.is_some());
            data.energy_buildings
                .iter_mut()
                .map(|b| &mut b.id)
                .chain(
                    data.buildings
                        .iter_mut()
                        .filter(|cost| associated || cost.id != 0)
                        .map(|b| &mut b.id),
                )
                .collect()
        }
        _ => Vec::new(),
    }
}
