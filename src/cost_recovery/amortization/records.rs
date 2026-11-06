use crate::BusinessIntangible;
use alloc::vec::Vec;

pub(crate) fn records(income: &crate::FederalIncome) -> &[BusinessIntangible] {
    match income {
        crate::FederalIncome::Business { data } => &data.intangibles,
        _ => &[],
    }
}
pub(crate) fn ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    match income {
        crate::FederalIncome::Business { data } => {
            data.intangibles.iter_mut().map(|row| &mut row.id).collect()
        }
        _ => Vec::new(),
    }
}

pub(crate) fn predecessor_choices(business: &crate::FederalBusiness) -> Vec<crate::RecordChoice> {
    business
        .intangibles
        .iter()
        .filter(|record| record.id > 0)
        .map(|record| crate::RecordChoice {
            id: record.id,
            label: crate::Message::new(
                "TEXT",
                alloc::vec![crate::MessageArg::Text(record.label.clone())],
            )
            .encode(),
        })
        .collect()
}
