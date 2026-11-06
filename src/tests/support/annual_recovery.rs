use crate::kernel::{
    carry::Amount,
    cost_recovery::{Month, annual, gds},
};

pub(crate) fn rounded_records() -> alloc::vec::Vec<annual::Record> {
    [20_000, 32_000, 19_200, 11_500, 11_500, 5_800]
        .into_iter()
        .enumerate()
        .map(|(index, regular)| annual::Record {
            year: 2020 + index as u32,
            section179: Amount::new(0).unwrap(),
            bonus: Amount::new(0).unwrap(),
            regular: Amount::new(regular).unwrap(),
        })
        .collect()
}

pub(crate) fn schedule(records: &[annual::Record]) -> annual::Schedule<'_> {
    annual::Schedule {
        table: gds::Table {
            placed: Month { year: 2020, month: 1 },
            period: gds::Period::Five,
            method: gds::Method::Regular,
            convention: gds::Convention::HalfYear,
        },
        target_year: 2026,
        record_end_year: 2026,
        basis: Amount::new(100_001).unwrap(),
        opening_section179_carry: Amount::new(0).unwrap(),
        current_section179: Amount::new(0).unwrap(),
        disposed: None,
        original_section179: Amount::new(0).unwrap(),
        bonus_percent: 0,
        safe_harbor: false,
        first_year_deduction: Amount::new(20_000).unwrap(),
        original_bonus: Amount::new(0).unwrap(),
        records,
        prior_depreciation: Amount::new(100_000).unwrap(),
        post_recovery_basis: Amount::new(1).unwrap(),
    }
}
