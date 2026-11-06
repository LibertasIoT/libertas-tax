use crate::BusinessMileage;
use alloc::vec::Vec;

impl BusinessMileage {
    pub(crate) fn records(income: &crate::FederalIncome) -> &[Self] {
        match income {
            crate::FederalIncome::Business { data } => &data.vehicles,
            crate::FederalIncome::FarmRental { data } => &data.vehicles,
            _ => &[],
        }
    }
    pub(crate) fn ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
        let rows = match income {
            crate::FederalIncome::Business { data } => &mut data.vehicles,
            crate::FederalIncome::FarmRental { data } => &mut data.vehicles,
            _ => return Vec::new(),
        };
        rows.iter_mut().map(|row| &mut row.id).collect()
    }
}
