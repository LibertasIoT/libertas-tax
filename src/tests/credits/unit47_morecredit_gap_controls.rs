use super::unit47_payment_credit_admission::case as checked_case;
use super::*;
use crate::tests::federal::minimum_tax;

fn minimum_histories(page: &mut P) -> &mut MinimumTaxHistoriesPage {
    let P::SaveMoreCredits { value, .. } = page else {
        unreachable!()
    };
    let MoreCredit::MinimumTax(value) = &mut value.credits[0] else {
        unreachable!()
    };
    value
}

fn minimum_amounts(session: &Interview, histories: usize) {
    let result = supported(&session.draft);
    // Each actual retained Single return: (220,000 - 88,100) * 26%
    // - 20,000 = 14,294 exclusion AMT; 20,000 - 14,294 + 10,000 = 15,706.
    for index in 0..histories {
        assert_eq!(
            value(
                &result,
                &alloc::format!("minimum_tax_history_{index}_line_21")
            ),
            15_706
        );
    }
    assert_eq!(
        value(&result, "minimum_tax_credit") + value(&result, "minimum_tax_credit_carryforward"),
        15_706 * histories as i64
    );
    assert_eq!(result.tax, Some(0));
    if histories == 1 {
        // $60,000 current Single wages less $16,100: table tax $5,023.
        assert_eq!(value(&result, "minimum_tax_credit"), 5_023);
        assert_eq!(value(&result, "minimum_tax_credit_carryforward"), 10_683);
    }
}

#[test]
fn unit47_minimum_tax_foreign_category_missing_save_witnesses() {
    for kind in ["category", "reclassification", "reduction", "negative-pool"] {
        let mut session = minimum_tax::session(FederalStatus::Single);
        let mut facts = minimum_tax::source();
        facts.foreign_credit = MinimumTaxForeignCredit::RetainedCategories {
            categories: vec![
                ForeignCreditCategory::Passive,
                ForeignCreditCategory::General,
            ]
            .into_iter()
            .map(|category| MinimumTaxForeignCategory {
                category,
                income: 0,
                paid: 0,
                carry: 0,
                reduction: 0,
                reclassified: 0,
            })
            .collect(),
            worldwide: 22_000_000,
        };
        let corrected =
            minimum_tax::request(&mut session, MinimumTaxHistories::from_current_unit(facts));
        let mut bad = corrected.clone();
        let MinimumTaxForeignCredit::RetainedCategories { categories, .. } =
            &mut minimum_histories(&mut bad).returns[0].facts.foreign_credit
        else {
            unreachable!()
        };
        let (key, suffix) = match kind {
            "category" => {
                categories[1].category = ForeignCreditCategory::Passive;
                ("MTC_FTC_CATEGORY_TWICE", "categories/1/category")
            }
            "reclassification" => {
                categories[0].reclassified = 1;
                ("MTC_FTC_RECLASSIFICATION_UNBALANCED", "categories")
            }
            "reduction" => {
                categories[0].reduction = 1;
                ("MTC_FTC_REDUCTION_ABOVE_TAX", "categories/0/reduction")
            }
            "negative-pool" => {
                categories[0].reclassified = -1;
                categories[1].reclassified = 1;
                ("MTC_FTC_NEGATIVE_POOL", "categories/0/reclassified")
            }
            _ => unreachable!(),
        };
        checked_case(
            &alloc::format!("gap-mtc-{kind}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            key,
            &alloc::format!("[value/credits/0/returns/0/facts/foreign_credit/{suffix}]"),
        );
        minimum_amounts(&session, 1);
    }
}

#[test]
fn unit47_minimum_tax_foreign_gain_and_regular_base_missing_save_witnesses() {
    for kind in ["needs-worksheets", "no-worksheets", "regular-base"] {
        let mut session = minimum_tax::session(FederalStatus::Single);
        let mut facts = minimum_tax::source();
        if kind != "regular-base" {
            facts.foreign_earned = Some(MinimumTaxForeignEarned {
                excluded: 0,
                related_deductions: 0,
                gain_source: if kind == "needs-worksheets" {
                    MinimumTaxForeignGainSource::NoPreferentialIncome
                } else {
                    MinimumTaxForeignGainSource::RetainedModifiedWorksheets
                },
            });
        }
        if kind != "needs-worksheets" {
            facts.gains = Some(MinimumTaxRateFacts::QualifiedDividends(
                MinimumTaxQualifiedRateFacts {
                    preferential: 0,
                    regular_ordinary: facts.base,
                },
            ));
        }
        let corrected =
            minimum_tax::request(&mut session, MinimumTaxHistories::from_current_unit(facts));
        let mut bad = corrected.clone();
        let facts = &mut minimum_histories(&mut bad).returns[0].facts;
        let (key, suffix) = if kind == "regular-base" {
            let Some(MinimumTaxRateFacts::QualifiedDividends(gains)) = &mut facts.gains else {
                unreachable!()
            };
            gains.regular_ordinary += 1;
            ("MTC_REGULAR_GAIN_BASE", "gains")
        } else {
            facts.foreign_earned.as_mut().unwrap().gain_source = if kind == "needs-worksheets" {
                MinimumTaxForeignGainSource::RetainedModifiedWorksheets
            } else {
                MinimumTaxForeignGainSource::NoPreferentialIncome
            };
            ("MTC_FOREIGN_GAIN_SOURCE", "foreign_earned/gain_source")
        };
        checked_case(
            &alloc::format!("gap-mtc-{kind}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            key,
            &alloc::format!("[value/credits/0/returns/0/facts/{suffix}]"),
        );
        minimum_amounts(&session, 1);
    }
}

#[test]
fn unit47_minimum_tax_joint_and_separate_history_missing_save_witness() {
    let mut session = minimum_tax::session(FederalStatus::Joint);
    let histories = MinimumTaxHistories {
        returns: [Owner::Taxpayer, Owner::Spouse]
            .into_iter()
            .map(|owner| OwnedMinimumTaxHistory {
                unit: MinimumTaxHistoryUnit::Individual { owner },
                facts: minimum_tax::source(),
            })
            .collect(),
    };
    let corrected = minimum_tax::request(&mut session, histories);
    let mut bad = corrected.clone();
    let source = &mut minimum_histories(&mut bad).returns[1];
    source.unit = MinimumTaxHistoryUnitPage::SameJointCouple;
    source.facts.status = FederalStatus::Joint;
    checked_case(
        "gap-mtc-joint-separate",
        &mut session,
        bad,
        corrected,
        TaxSection::MoreCredits,
        "MTC_JOINT_AND_SEPARATE_HISTORY",
        "[value/credits/0/returns/1/unit]",
    );
    minimum_amounts(&session, 2);
}

fn retained_foreign_records() -> ForeignCarryoverRecords {
    ForeignCarryoverRecords {
        schedules: [
            ForeignCreditCategory::General,
            ForeignCreditCategory::Passive,
        ]
        .into_iter()
        .map(|category| ForeignCarryoverSchedule {
            category,
            columns: [2024, 2025]
                .into_iter()
                .map(|year| ForeignCarryoverColumn {
                    year,
                    regular: ForeignCarryoverOpening {
                        carryover: 10_000,
                        carryback_adjustment: 0,
                        redetermination: 0,
                        other_adjustments: 0,
                    },
                    amt: ForeignCarryoverOpening {
                        carryover: 20_000,
                        carryback_adjustment: 0,
                        redetermination: 0,
                        other_adjustments: 0,
                    },
                })
                .collect(),
        })
        .collect(),
        same_filing_unit: true,
    }
}

fn foreign_records(page: &mut P) -> &mut ForeignCarryoverRecords {
    let P::SaveMoreCredits { value, .. } = page else {
        unreachable!()
    };
    let MoreCredit::ForeignTax(ForeignTaxPage::Ordinary {
        history: ForeignCreditHistory::Retained(records),
        ..
    }) = &mut value.credits[0]
    else {
        unreachable!()
    };
    records
}

#[test]
fn unit47_foreign_carryover_missing_save_witnesses() {
    for kind in [
        "category",
        "year",
        "regular-carryback",
        "amt-carryback",
        "regular-negative",
        "amt-negative",
    ] {
        let mut session = minimum_tax::session(FederalStatus::Single);
        let Reply::Edit(mut corrected) = from_review(&mut session, TaxSection::MoreCredits) else {
            unreachable!()
        };
        let mut claim =
            crate::tests::foreign::foreign::fund_claim(0, 0, 0, ForeignRateAdjustment::Exception);
        let ForeignTaxClaim::Ordinary {
            sources, history, ..
        } = &mut claim
        else {
            unreachable!()
        };
        sources.clear();
        *history = ForeignCreditHistory::Retained(retained_foreign_records());
        let P::SaveMoreCredits { value: inputs, .. } = &mut corrected else { unreachable!() };
        inputs.credits = vec![MoreCredit::ForeignTax(claim.try_into().unwrap())];
        let mut bad = corrected.clone();
        let records = foreign_records(&mut bad);
        let (key, suffix) = match kind {
            "category" => {
                records.schedules[1].category = ForeignCreditCategory::General;
                ("FOREIGN_CARRYOVER_CATEGORY_TWICE", "schedules/1/category")
            }
            "year" => {
                records.schedules[0].columns[1].year = 2024;
                ("FOREIGN_CARRYOVER_YEAR_TWICE", "schedules/0/columns/1/year")
            }
            "regular-carryback" => {
                records.schedules[0].columns[0].regular.carryback_adjustment = 1;
                (
                    "FOREIGN_CARRYBACK_ADJUSTMENT_PRIOR_YEAR_ONLY",
                    "schedules/0/columns/0/regular/carryback_adjustment",
                )
            }
            "amt-carryback" => {
                records.schedules[0].columns[0].amt.carryback_adjustment = 1;
                (
                    "FOREIGN_CARRYBACK_ADJUSTMENT_PRIOR_YEAR_ONLY",
                    "schedules/0/columns/0/amt/carryback_adjustment",
                )
            }
            "regular-negative" => {
                records.schedules[0].columns[0].regular.redetermination = -10_001;
                (
                    "FOREIGN_CARRYOVER_BELOW_ZERO",
                    "schedules/0/columns/0/regular",
                )
            }
            "amt-negative" => {
                records.schedules[0].columns[0].amt.redetermination = -20_001;
                ("FOREIGN_CARRYOVER_BELOW_ZERO", "schedules/0/columns/0/amt")
            }
            _ => unreachable!(),
        };
        checked_case(
            &alloc::format!("gap-ftc-{kind}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            key,
            &alloc::format!("[value/credits/0/history/{suffix}]"),
        );
        let result = supported(&session.draft);
        assert_eq!(value(&result, "foreign_tax_credit"), 0);
        assert_eq!(result.tax, Some(502_300));
        for category in ["general", "passive"] {
            assert_eq!(
                value(
                    &result,
                    &alloc::format!("foreign_{category}_regular_retained_carryforward")
                ),
                200
            );
            assert_eq!(
                value(
                    &result,
                    &alloc::format!("foreign_{category}_amt_retained_carryforward")
                ),
                400
            );
        }
    }
}
