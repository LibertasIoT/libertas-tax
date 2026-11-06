use crate::schedule_k3::K3PaperPartII;
use crate::{
    K3Assets, K3Category, K3Item, K3PaperColumn, K3PaperColumns, K3PaperCountryCell,
    K3PaperCountryRow, K3PaperForeignTaxes, K3PaperInterestAssets, K3PaperOtherInformation,
    K3PaperResearchRow, K3PaperSourcedColumns, K3PaperTaxColumns, K3PaperTaxRow, ScheduleK3,
    ScheduleK3Paper,
};
use alloc::vec;

fn columns(branch: i64, passive: i64, general: i64, sourced: i64) -> K3PaperColumns {
    K3PaperColumns {
        branch,
        passive,
        general,
        sourced,
        total: crate::kernel::ratio::checked_sum([branch, passive, general, sourced])
            .expect("synthetic K-3 column total"),
        ..Default::default()
    }
}

fn assets(value: &K3Assets) -> K3PaperColumns {
    let mut result = columns(value.branch, value.passive, value.general, value.us);
    result.us = result.sourced;
    result.sourced = 0;
    result
}

fn tax_columns(branch: i64, passive: i64, general: i64) -> K3PaperTaxColumns {
    K3PaperTaxColumns {
        branch: crate::K3PaperTaxSourceColumns { foreign: branch, ..Default::default() },
        passive: crate::K3PaperTaxSourceColumns { foreign: passive, ..Default::default() },
        general: crate::K3PaperTaxSourceColumns { foreign: general, ..Default::default() },
        total: crate::kernel::ratio::checked_sum([branch, passive, general])
            .expect("synthetic K-3 tax total"),
        ..Default::default()
    }
}

// Author synthetic printed cells for the existing natural scenarios. This is
// test setup, not a production inverse or a reconstruction of imported paper.
// Withholding remains in the K-1's typed personal records.
pub(super) fn schedule_k3_paper(
    source: &ScheduleK3,
    mut paper: ScheduleK3Paper,
) -> ScheduleK3Paper {
    let ScheduleK3 {
        countries,
        withholding: _,
        us_gross,
        branch,
        passive,
        general,
        partner_deductions,
        partner_interest,
        items,
    } = source;
    let mut income = K3PaperPartII::default();
    let mut taxes = K3PaperForeignTaxes {
        paid: true,
        line_2: tax_columns(branch.reduction, passive.reduction, general.reduction),
        ..Default::default()
    };
    income.line_24.total.us = *us_gross;
    for country in countries {
        let column = match country.category {
            K3Category::ForeignBranch => K3PaperColumn::Branch,
            K3Category::Passive => K3PaperColumn::Passive,
            K3Category::General => K3PaperColumn::General,
        };
        for (line, amount) in
            [(&mut income.line_24, country.income), (&mut income.line_8, country.qualified)]
        {
            line.countries.push(K3PaperCountryRow {
                country: country.country.clone(),
                amounts: vec![K3PaperCountryCell { column, amount }],
            });
            let total = match country.category {
                K3Category::ForeignBranch => &mut line.total.branch,
                K3Category::Passive => &mut line.total.passive,
                K3Category::General => &mut line.total.general,
            };
            *total = crate::kernel::ratio::checked_sum([*total, amount])
                .expect("synthetic K-3 country total");
        }
        let (branch, passive, general) = match country.category {
            K3Category::ForeignBranch => (country.tax, 0, 0),
            K3Category::Passive => (0, country.tax, 0),
            K3Category::General => (0, 0, country.tax),
        };
        taxes.line_1.push(K3PaperTaxRow {
            country: country.country.clone(),
            tax_type: "OTH".into(),
            amounts: tax_columns(branch, passive, general),
        });
    }
    for line in [&mut income.line_24, &mut income.line_8] {
        line.total.total = crate::kernel::ratio::checked_sum([
            line.total.us,
            line.total.branch,
            line.total.passive,
            line.total.general,
        ])
        .expect("synthetic K-3 income total");
    }
    income.line_25 =
        columns(branch.deductions, passive.deductions, general.deductions, *partner_deductions);
    // Line 39 retains category columns in both printed layouts; 1120-S line 41
    // has only the shareholder-sourced column.
    income.line_39 = columns(branch.interest, passive.interest, general.interest, 0);
    income.line_41 = columns(0, 0, 0, partner_interest.business);
    income.line_43 = columns(0, 0, 0, partner_interest.passive_activity);
    income.line_11.total = columns(
        branch.capital.short_gain,
        passive.capital.short_gain,
        general.capital.short_gain,
        0,
    );
    income.line_12.total =
        columns(branch.capital.long_gain, passive.capital.long_gain, general.capital.long_gain, 0);
    income.line_27 = columns(
        branch.capital.short_loss,
        passive.capital.short_loss,
        general.capital.short_loss,
        0,
    );
    income.line_28 =
        columns(branch.capital.long_loss, passive.capital.long_loss, general.capital.long_loss, 0);
    let interest_assets = K3PaperInterestAssets {
        line_6b: assets(&partner_interest.business_assets),
        line_6d: assets(&partner_interest.passive_assets),
        ..Default::default()
    };
    let mut part_i = K3PaperOtherInformation::default();
    let mut foreign_corporations = false;
    for item in items {
        // These existing natural flags specify presence, not an amount. One
        // synthetic cent makes their printed presence explicit without changing
        // any normalized financial component.
        match item {
            K3Item::OtherCategories => income.line_20.total.other = 1,
            K3Item::ForeignRateGroups => income.line_13.total.branch = 1,
            K3Item::Research => {
                income.line_32 = vec![K3PaperResearchRow {
                    sic_code: "0000".into(),
                    amounts: K3PaperSourcedColumns { sourced: 1, total: 1 },
                }]
            }
            K3Item::OilAndGas => part_i.box_2 = true,
            K3Item::Loans => part_i.box_10 = true,
            K3Item::DisallowedDeduction => part_i.box_6 = true,
            K3Item::ForeignCorporations => foreign_corporations = true,
        }
    }
    match &mut paper {
        ScheduleK3Paper::Partnership(form) => {
            form.part_i = part_i;
            form.part_ii = income.into();
            form.interest_assets = interest_assets;
            form.foreign_taxes = taxes;
            form.part_6 = Some(foreign_corporations);
        }
        ScheduleK3Paper::Scorp(form) => {
            form.part_i = part_i;
            form.part_ii = income.into();
            form.interest_assets = interest_assets;
            form.foreign_taxes = taxes;
            form.part_5 = Some(foreign_corporations);
        }
    }
    // A native fixture that cannot be represented by printed cells must fail
    // explicitly, rather than quietly lose a value or a repeated entry.
    let ordered = |mut value: ScheduleK3| {
        value.countries.sort_by(|a, b| {
            a.country.cmp(&b.country).then((a.category as u8).cmp(&(b.category as u8)))
        });
        value.items.sort_by_key(|item| *item as u8);
        value
    };
    assert_eq!(
        ordered(paper.native(&source.withholding)),
        ordered(source.clone()),
        "native K-3 fixture cannot be represented by printed K-3 cells",
    );
    paper
}
