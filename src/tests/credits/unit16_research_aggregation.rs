// Original R20: native QBI aggregation must preserve each research source's
// actual deduction before the section41(g) taxable-income limitation.
use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

fn aggregated_research_return() -> (FederalDraft, [i64; 2]) {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
    let mut ids = [0; 2];
    for (index, (rent, expense)) in [(200_000, 0), (170_000, 120_000)].into_iter().enumerate() {
        let mut source = rental(rent, expense);
        if expense == 0 {
            // No expense records: an OtherExpense row requires a positive
            // amount in the normal editor, rather than a zero placeholder.
            source.expenses.clear();
        }
        source.buildings[0].cost = source.buildings[0].land;
        source.property_type = RentalPropertyType::VacationShortTerm {
            stays: ShortTermStays::SevenDaysOrLess { material_participation: true },
        };
        source.qbi_business = RentalBusiness::Yes { material_participation: true };
        source.qbi_aggregation_group = Some("Synthetic research shops".into());
        if index == 1 {
            // The ordinary rental expenses already contain this payroll.
            source.payroll = BusinessPayroll::Yes {
                statements: vec![PayrollW2 {
                    label: "Synthetic employee payroll".into(),
                    wages: 12_000_000,
                    medicare: 12_000_000,
                }],
                attribution: PayrollQbiAttribution::EntireBusiness,
            };
        }
        add(&mut d, income(FederalIncome::Rental { data: source }));
        ids[index] = d.income.last().unwrap().id;
        let mut credit = carry(BusinessCredit::ResearchSmallBusiness, 2024, 100_000);
        credit.research_source = Some(ids[index]);
        history(&mut d).carryforwards.push(credit);
    }
    (d, ids)
}

fn used(r: &FederalResult, id: i64) -> i64 {
    value(
        r,
        &alloc::format!("business_credit_research_small_business_source_{id}_filer_0_2024_used"),
    )
}

fn assert_grouped(r: &FederalResult, ids: [i64; 2]) {
    assert_eq!(value(r, "agi"), 500_000);
    assert_eq!(value(r, "qbi_deduction"), 50_000);
    assert_eq!(value(r, "taxable_income"), 433_900);
    assert_eq!(value(r, "form_3800_line_11"), 120_634);
    // QBI250000, group wage limit60000 => deduction50000. Actual source
    // shares40000/10000 leave160000/40000. The common16100 is allocated
    // over450000:5724/1431. Numerators154276/38569 times120634/433900
    // give literal independent caps42892/10723. No final3800 recursion.
    assert_eq!(ids.map(|id| used(r, id)), [42_892, 10_723]);
    assert_eq!(value(r, "general_business_credit"), 53_615);
}

#[test]
fn unit16_research_aggregated_qbi_uses_native_member_identity_and_source_deductions() {
    let (mut d, ids) = aggregated_research_return();
    let r = supported(&d);
    assert_grouped(&r, ids);
    assert_eq!(value(&r, "qbi_aggregation_1_total_income"), 250_000);
    assert_eq!(value(&r, "qbi_aggregation_1_total_wages"), 120_000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    let reordered = supported(&d);
    assert_grouped(&reordered, ids);
    assert_eq!((reordered.tax, reordered.refund), (r.tax, r.refund));
}

#[test]
fn unit16_research_removing_actual_aggregation_changes_only_native_source_caps() {
    let (mut d, ids) = aggregated_research_return();
    assert_grouped(&supported(&d), ids);
    for entry in &mut d.income {
        if let FederalIncome::Rental { data } = &mut entry.income {
            data.qbi_aggregation_group = None;
        }
    }
    let r = supported(&d);
    assert_ungrouped(&r, ids);
    assert_eq!(supported(&restart(d).draft), r);
}

fn assert_ungrouped(r: &FederalResult, ids: [i64; 2]) {
    // SourceA has no wages/UBIA and gets0 QBI deduction above the complete
    // phase-in; B gets10000. Same500000 AGI, source nets200000/40000,
    // common16100 over490000 =>6571/1314, numerators193429/38686.
    // Ordinary tax134634, TI473900 => caps54953/10991.
    assert_eq!(value(r, "agi"), 500_000);
    assert_eq!(value(r, "qbi_deduction"), 10_000);
    assert_eq!(value(r, "taxable_income"), 473_900);
    assert_eq!(value(r, "form_3800_line_11"), 134_634);
    assert_eq!(ids.map(|id| used(r, id)), [54_953, 10_991]);
    assert_eq!(value(r, "general_business_credit"), 65_944);
}

#[test]
fn unit16_research_aggregation_actual_rental_editor_save_reopen_restart_correction() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let (mut d, ids) = aggregated_research_return();
    for entry in &mut d.income {
        if let FederalIncome::Rental { data } = &mut entry.income {
            data.qbi_aggregation_group = None;
        }
    }
    let mut session = Interview::new(TEST_USER, d);
    let initial_seed = open_topic(&mut session, TaxSection::Rentals);
    let mut initial_answers = initial_seed.clone();
    let P::SaveIncomes { documents, .. } = &mut initial_answers else { panic!() };
    for row in &mut documents.documents {
        let IncomeDocument::Rental(page) = &mut row.document else { panic!() };
        page.more.qbi_group = "Synthetic research shops".into();
    }
    let correction = std::env::var("TAX_UNIT16_RESEARCH_CORRECTION").as_deref() == Ok("1");
    let native = NativeInterviewCase("TAX_UNIT16_RESEARCH");
    if correction {
        let first = native.input("INITIAL_PAYLOAD").unwrap_or(initial_answers.clone());
        accepted(&send(&mut session, first));
    }
    let seed =
        if correction { open_topic(&mut session, TaxSection::Rentals) } else { initial_seed };
    if native.required_calculation(&mut session) {
        return;
    }
    native.write_file("DIR", "seed.avro", &seed);
    let mut answers = if correction { seed } else { initial_answers };
    if correction {
        let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
        for row in &mut documents.documents {
            let IncomeDocument::Rental(page) = &mut row.document else { panic!() };
            page.more.qbi_group.clear();
        }
    }
    native.write_file("DIR", "answers.avro", &answers);
    if std::env::var("TAX_UNIT16_RESEARCH_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let request = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    if correction {
        assert_ungrouped(&r, ids);
    } else {
        assert_grouped(&r, ids);
    }
    assert_eq!(
        session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        ids
    );
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(session.draft.tax_year, &stored);
    assert_eq!(decoded, session.draft);
    assert_eq!(native_draft_records(&decoded), stored);
    if let Ok(dir) = std::env::var("TAX_UNIT16_RESEARCH_DIR") {
        write_native_draft_records(std::path::Path::new(&dir).join("persisted.avro"), &stored);
    }
    session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    native.write_file("DIR", "reopened.avro", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(supported(&session.draft), r);
    if !correction {
        let mut next = open_topic(&mut session, TaxSection::Rentals);
        let P::SaveIncomes { documents, .. } = &mut next else { panic!() };
        for row in &mut documents.documents {
            let IncomeDocument::Rental(page) = &mut row.document else { panic!() };
            page.more.qbi_group.clear();
        }
        accepted(&send(&mut session, next));
        assert_ungrouped(&supported(&session.draft), ids);
        assert_ungrouped(&supported(&restart(session.draft).draft), ids);
    }
}
