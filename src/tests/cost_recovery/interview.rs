// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, owned};
use super::support::client::income_pages::{open_topic, save_topic};
use crate::tests::cross::interview::{
    assert_round_trip, asset, business, k1_1065, k1_1065_continuing, k1_1120s,
    k1_sale_input_statement, save_businesses, stored_business, value_of, with_first_year,
};

#[test]
fn first_year_choices_map_onto_179_and_bonus() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let continuing = Asset {
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 400_000,
        credit_reduction: 0,
        placed: 20240315,
        class: AssetClass::Seven,
        business_percent: 100,
        acquired: None,
        history: AssetHistory::Continuing {
            prior_depreciation: 160_000,
            first_year_179: 0,
            first_year_bonus: 100_000,
            carryforward: 0,
            mid_quarter: false,
        },
        disposal: crate::AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    };
    let mut bought_earlier = asset(240_000, 20260210, AssetClass::Five);
    bought_earlier.acquired = Some(20250110);
    let mut part_business = asset(100_000, 20260501, AssetClass::Ten);
    part_business.business_percent = 40;
    let mut b = business(5_000_000);
    b.assets = vec![
        bought_earlier,
        with_first_year(
            asset(500_000, 20260315, AssetClass::Seven),
            FirstYearDeduction::Section179 { description: "Office desk".into(), elected: 300_000 },
        ),
        with_first_year(asset(150_000, 20260401, AssetClass::Three), FirstYearDeduction::Regular),
        part_business,
        asset(1_000_000, 20260601, AssetClass::ThirtyNine),
        continuing,
    ];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    let new = |section179, bonus| EquipmentHistory::New { section179, bonus, tables: true };
    let label_texts: Vec<_> = (0..data.assets.len())
        .map(|i| BusinessAsset::display_text(&data.assets, i, data.vehicles.len()))
        .collect();
    let summary: Vec<_> = data
        .assets
        .iter()
        .zip(&label_texts)
        .map(|(a, text)| (text.as_str(), a.description.as_str(), a.acquired, a.class, a.history))
        .collect();
    assert_eq!(
        summary,
        vec![
            // Bonus over 50% business use: section 179 property electing nothing.
            (
                "Asset 1",
                "",
                20250110,
                EquipmentClass::Five,
                new(Section179::Eligible { elected: 0 }, BonusElection::Apply)
            ),
            // Section 179: its description is the Form 4562 line 6 label, and
            // what a note calls it.
            (
                "Office desk",
                "Office desk",
                20260315,
                EquipmentClass::Seven,
                new(Section179::Eligible { elected: 300_000 }, BonusElection::Apply)
            ),
            // Regular elects out of bonus for its class.
            (
                "Asset 3",
                "",
                20260401,
                EquipmentClass::Three,
                new(Section179::Eligible { elected: 0 }, BonusElection::ElectOut)
            ),
            // 50% or less business use and buildings are not section 179 property.
            (
                "Asset 4",
                "",
                20260501,
                EquipmentClass::Ten,
                new(Section179::Ineligible, BonusElection::Apply)
            ),
            (
                "Asset 5",
                "",
                20260601,
                EquipmentClass::ThirtyNine,
                new(Section179::Ineligible, BonusElection::Apply)
            ),
            (
                "Asset 6",
                "",
                20240315,
                EquipmentClass::Seven,
                EquipmentHistory::Continuing {
                    initial_section179: 0,
                    carryforward: 0,
                    prior_depreciation: 160_000,
                    amt: EquipmentAmtHistory::Same,
                    mid_quarter: false,
                    schedule: EquipmentSchedule::Tables { initial_bonus: 100_000 },
                }
            ),
        ]
    );
    assert!(data.assets.iter().all(|a| a.method == EquipmentMethod::Regular));
    assert_eq!(
        data.assets.iter().map(|a| a.listed.clone()).collect::<Vec<_>>(),
        [100, 100, 100, 40, 100, 100]
            .into_iter()
            .map(|business_percent| ListedProperty::No { business_percent })
            .collect::<Vec<_>>()
    );
    // Bought January 10, 2025, before January 20, 2025: 20% bonus, $480.
    // The section 179 desk: $3,000 of its $5,000 elected; the $2,000 rest
    // takes the 100% bonus. The 3-year asset elected out: no bonus.
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_0_bonus")), 480);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_1_bonus")), 2_000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_2_bonus")), 0);
    assert_round_trip(&mut session, &[b]);
    // The service-year and asset page rules refuse by name on save.
    let before = session.draft.clone();
    let mut new_in_2025 = business(100_000);
    new_in_2025.assets = vec![asset(100_000, 20251231, AssetClass::Five)];
    rejected_with(save_businesses(&mut session, vec![new_in_2025]), "ASSET_NEW_PLACED_IN_2026");
    let mut old_in_2026 = business(100_000);
    old_in_2026.assets = vec![Asset {
        placed: 20260101,
        ..asset_with_history(AssetHistory::Continuing {
            prior_depreciation: 0,
            first_year_179: 0,
            first_year_bonus: 0,
            carryforward: 0,
            mid_quarter: false,
        })
    }];
    rejected_with(save_businesses(&mut session, vec![old_in_2026]), "ASSET_OLD_PLACED_BEFORE_2026");
    let mut bought_after = business(100_000);
    let mut late = asset(100_000, 20260210, AssetClass::Five);
    late.acquired = Some(20260211);
    bought_after.assets = vec![late];
    rejected_with(save_businesses(&mut session, vec![bought_after]), "ASSET_ACQUIRED_AFTER_PLACED");
    // Half of a $2,400 cost is $1,200 of business cost: one cent more is refused.
    let mut above = business(100_000);
    let mut half = with_first_year(
        asset(240_000, 20260210, AssetClass::Five),
        FirstYearDeduction::Section179 { description: "Laptop".into(), elected: 120_001 },
    );
    half.business_percent = 50;
    above.assets = vec![half];
    rejected_with(save_businesses(&mut session, vec![above]), "ASSET_179_ABOVE_BUSINESS_COST");
    assert_eq!(session.draft, before);
}
fn asset_with_history(history: AssetHistory) -> Asset {
    Asset { history, ..asset(100_000, 20240101, AssetClass::Five) }
}

#[test]
fn bonus_election_must_match_within_a_class() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let mut mixed = business(5_000_000);
    mixed.assets = vec![
        asset(100_000, 20260210, AssetClass::Five),
        with_first_year(asset(200_000, 20260310, AssetClass::Five), FirstYearDeduction::Regular),
    ];
    rejected_with(
        current_document_refusal(
            "income-source-1152",
            &mut session,
            vec![document(0, IncomeDocument::Business(mixed))],
            "ASSET_BONUS_ELECTION_BY_CLASS",
            "[documents/documents/0/document/assets/1]",
        ),
        "ASSET_BONUS_ELECTION_BY_CLASS",
    );
    assert_eq!(session.draft, before);
    // Different classes may choose differently.
    let mut separate = business(5_000_000);
    separate.assets = vec![
        asset(100_000, 20260210, AssetClass::Five),
        with_first_year(asset(200_000, 20260310, AssetClass::Seven), FirstYearDeduction::Regular),
    ];
    accepted(&save_businesses(&mut session, vec![separate]));
    // Original controls1152/1183 also exercise the ordinary identity owner:
    // only the accepted Save allocates the source and its two property IDs.
    assert_eq!(session.draft.income.len(), before.income.len() + 1);
    assert_eq!(session.draft.income[0], before.income[0]);
    let source = session.draft.income.last().unwrap();
    assert_eq!(source.id, before.next_id);
    assert_eq!(session.draft.revision, before.revision + 1);
    let FederalIncome::Business { data } = &source.income else { unreachable!() };
    assert_eq!(data.assets.len(), 2);
    assert_eq!(session.draft.next_id, before.next_id + 1 + data.assets.len() as i64);
    for (index, asset) in data.assets.iter().enumerate() {
        assert!(asset.id > source.id && asset.id < session.draft.next_id);
        assert!(!data.assets[..index].iter().any(|other| other.id == asset.id));
        assert_eq!(asset.history_details, None);
        assert_eq!(asset.amt_section179_income_carry, None);
    }

    // The election is return-wide: rented equipment of the 7-year class
    // must elect out as the business did.
    let rent = |first_year| EquipmentRent {
        receipts: 150_000,
        expenses: 0,
        basis: 300_000,
        depreciable_basis: 300_000,
        assets: vec![with_first_year(asset(300_000, 20260410, AssetClass::Seven), first_year)],
        history: PersonalRentalHistory::NONE,
    };
    let request = save_topic(
        &session.draft.clone(),
        TaxSection::OtherIncome,
        vec![document(0, IncomeDocument::EquipmentRental(rent(FirstYearDeduction::Bonus)))],
    );
    rejected_with(
        current_income_refusal(
            "income-source-1183",
            &mut session,
            request,
            "ASSET_BONUS_ELECTION_BY_CLASS",
            "[documents/documents/0/document/assets/0]",
        ),
        "ASSET_BONUS_ELECTION_BY_CLASS",
    );
    let request = save_topic(
        &session.draft.clone(),
        TaxSection::OtherIncome,
        vec![document(0, IncomeDocument::EquipmentRental(rent(FirstYearDeduction::Regular)))],
    );
    accepted(&send(&mut session, request));
    supported(&session.draft);
}

#[test]
fn unit17_ptp179_actual_two_rows_complete_save_reopen_restart() {
    let client = NativeInterviewCase("TAX_UNIT17_PTP");
    let draft = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, draft);
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    if client.required_calculation(&mut session) {
        return;
    }
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = [
        ("Synthetic PTP A", 2_000_000, 500_000, 200_000),
        ("Synthetic PTP B", 1_000_000, 300_000, 0),
    ]
    .into_iter()
    .map(|(name, profit, election, carry)| {
        let mut page = k1_1065(name);
        page.publicly_traded = true;
        page.ordinary = profit;
        page.more.section179 = election;
        page.more.section179_carryover = carry;
        // The selected AMT source needs its own actual income-carry record;
        // a regular balance alone does not establish that history.
        page.more.amt_section179_income_carry = Some(Section179IncomeCarryHistory {
            gross: carry,
            sold: 0,
            vintages: if carry == 0 {
                vec![]
            } else {
                vec![Asset179Carryover { year: 2023, amount: carry }]
            },
        });
        page.interest =
            PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
        page.basis.start = k1_1065_continuing(
            10_000_000,
            ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
        );
        page.qbi = PartnerQbi::NoStatement;
        owned(Owner::Taxpayer, 0, IncomeDocument::K1Partnership(page))
    })
    .collect();
    if client
        .files("SEED_DIR", &[("seed.avro", seed.to_avro()), ("answers.avro", answers.to_avro())])
    {
        return;
    }
    client.submit(&mut session, answers);
    let saved = session.draft.clone();
    let rows = saved
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Partnership { .. }))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![2, 3]);
    let selected =
        [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative].map(|system| {
            let sources = rows
                .iter()
                .map(|row| {
                    let FederalIncome::Partnership { data } = &row.income else { unreachable!() };
                    let record = if system == Section179IncomeSystem::Alternative {
                        data.alternative.as_ref().unwrap()
                    } else {
                        &data.regular
                    };
                    Section179BusinessIncomeSource {
                        source: row.id,
                        system,
                        ordinary: Section179IncomeClass {
                            current_net_cents: record.ordinary,
                            prior_allowed_loss_cents: 0,
                        },
                        section1231: Section179IncomeClass {
                            current_net_cents: 0,
                            prior_allowed_loss_cents: 0,
                        },
                    }
                })
                .collect::<Vec<_>>();
            crate::section179::section179_with_native_prepared_income_and_operating_context(
                &saved,
                system,
                &[],
                &sources,
                |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
            )
            .unwrap()
        });
    for (row, expected) in rows.iter().zip([700_000, 300_000]) {
        for alternative in [false, true] {
            let delivered = selected[usize::from(alternative)]
                .ptp_section179_pal_input(row.id)
                .unwrap()
                .clone();
            assert_eq!(delivered.allowed_cents, expected);
            assert_eq!(delivered.business_income_carry_cents, 0);
            let FederalIncome::Partnership { data } = &row.income else { unreachable!() };
            let record =
                if alternative { data.alternative.as_ref().unwrap() } else { &data.regular };
            let pool = crate::community::partner_pool(&saved, row.id, data, record).unwrap();
            let pal = crate::ptp_passive_source(
                row.id,
                alternative,
                &[crate::PtpPalActivity {
                    k1: row.id,
                    source: row.id,
                    components: vec![
                        crate::PtpPalComponent {
                            character: crate::PtpPalCharacter::Ordinary,
                            passive_current_cents: pool.combined.passive + pool.combined.prior_pool,
                            prior_passive_loss_cents: pool.combined.prior_pool,
                            nonpassive_cents: pool.combined.nonpassive,
                        },
                        crate::PtpPalComponent {
                            character: crate::PtpPalCharacter::Section179,
                            passive_current_cents: 0,
                            prior_passive_loss_cents: 0,
                            nonpassive_cents: -delivered.allowed_cents,
                        },
                    ],
                }],
                &delivered,
            )
            .unwrap();
            assert_eq!(pal.suspended_cents, 0);
            assert_eq!(
                pal.components
                    .iter()
                    .map(|component| component.allowed_passive_net_cents
                        + component.original.nonpassive_cents)
                    .sum::<i64>(),
                if row.id == 2 { 1_300_000 } else { 700_000 }
            );
        }
    }
    let source_allocation =
        crate::section179_return::prepare(&saved, Section179IncomeSystem::Regular)
            .unwrap()
            .allocation
            .income;
    assert_eq!(source_allocation.k1[&2].allowed_cents, 700_000);
    assert_eq!(source_allocation.k1[&3].allowed_cents, 300_000);
    assert_eq!(source_allocation.k1[&2].business_income_carry_cents, 0);
    assert_eq!(
        source_allocation,
        crate::section179_return::prepare(
            &restart(saved.clone()).draft,
            Section179IncomeSystem::Regular
        )
        .unwrap()
        .allocation
        .income
    );
    // 2026-10-16 D6 source coordination consumes the actual selected PTP
    // allocation. Both AMT income-carry histories are explicitly supplied
    // above; the original native source/PAL figures now reach the return.
    let production = supported(&saved);
    assert_eq!(value_of(&production, "section179_deduction"), 10_000);
    for prefix in ["", "amt_"] {
        assert_eq!(
            value_of(&production, &alloc::format!("{prefix}partnership_2_activity_net")),
            13_000
        );
        assert_eq!(
            value_of(&production, &alloc::format!("{prefix}partnership_3_activity_net")),
            7_000
        );
    }
    client.restart(&mut session);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    client.write_file("OUTPUT_DIR", "reopened.avro", &reopened);
    let request = client.input("REOPEN_PAYLOAD").unwrap_or(reopened);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
    assert_eq!(estimate(&session.draft), production);
    let mut correction = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[1].document else {
        unreachable!()
    };
    page.more.section179 = 200_000;
    accepted(&send(&mut session, correction));
    assert_eq!(
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
            .unwrap()
            .allocation
            .ptp_section179_pal_input(3)
            .unwrap()
            .allowed_cents,
        200_000
    );
    let corrected_source =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular)
            .unwrap()
            .allocation
            .income;
    assert_eq!(corrected_source.k1[&3].allowed_cents, 200_000);
    assert_eq!(corrected_source.k1[&2].allowed_cents, 700_000);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let corrected = supported(&session.draft);
    assert_eq!(value_of(&corrected, "section179_deduction"), 9_000);
    for prefix in ["", "amt_"] {
        assert_eq!(
            value_of(&corrected, &alloc::format!("{prefix}partnership_2_activity_net")),
            13_000
        );
        assert_eq!(
            value_of(&corrected, &alloc::format!("{prefix}partnership_3_activity_net")),
            8_000
        );
    }
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
}

#[test]
fn unit17_k1_sale_identity_complete_editor_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT17_SALE");
    for partner in [true, false] {
        if let Ok(kind) = std::env::var("TAX_UNIT17_SALE_KIND")
            && (kind == "partner") != partner
        {
            continue;
        }
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let seed = open_topic(&mut session, TaxSection::PassThrough);
        if let Some(request) = client.input("CALC_INPUT") {
            if let Ok(file) = std::env::var("TAX_UNIT17_SALE_CALC_DRAFT") {
                session = restart(read_native_draft(file));
                let _ = open_topic(&mut session, TaxSection::PassThrough);
            }
            let Reply::Response(response) = send(&mut session, request) else {
                panic!("calculation response")
            };
            client.required_write("CALC_OUTPUT", &response);
            return;
        }
        let mut properties = vec![k1_sale_input_statement(), k1_sale_input_statement()];
        properties[0].alternative =
            crate::K1Section179SaleAmtSource::from_known_regular(&properties[0].regular);
        properties[0].alternative.reported_amounts_mut().unwrap().depreciation = 600_000;
        properties[1].regular.as_mut().unwrap().price = 3_500_000;
        properties[1].alternative =
            crate::K1Section179SaleAmtSource::from_known_regular(&properties[1].regular);
        properties[1].alternative.reported_amounts_mut().unwrap().depreciation = 400_000;
        let unavailable = std::env::var("TAX_UNIT17_SALE_AMT_UNAVAILABLE").is_ok();
        if unavailable {
            properties[0].alternative = crate::K1Section179SaleAmtSource::Unavailable;
        }
        let absence_correction = std::env::var("TAX_UNIT17_SALE_AMT_ABSENCE_CORRECTION").is_ok();
        let disjoint40 = std::env::var("TAX_UNIT17_SALE_DISJOINT40").is_ok();
        if disjoint40 {
            let mut regular = properties[0].clone();
            regular.alternative = crate::K1Section179SaleAmtSource::NotReported;
            let mut alternative = properties[0].clone();
            alternative.regular = None;
            properties = vec![regular; 20];
            properties.extend(vec![alternative; 20]);
        }
        let make = |name: &str, sales: Vec<crate::K1Section179Sale>| {
            if partner {
                let mut page = k1_1065(name);
                let record = page.record().regular;
                page.more.section179_sales = sales;
                page.amt = PartnerAmtRecords::Separate { record };
                IncomeDocument::K1Partnership(page)
            } else {
                let mut page = k1_1120s(name);
                let record = ScorpAmtRecords::same_as(&page.record());
                page.more.section179_sales = sales;
                page.amt = ScorpAmtRecords::Separate { record };
                IncomeDocument::K1Scorp(page)
            }
        };
        let mut request = seed.clone();
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        documents.documents = vec![
            document(0, make("First actual source", properties.clone())),
            document(0, make("Other actual parent", vec![properties[0].clone()])),
        ];
        if client.files(
            "SEED_DIR",
            &[("seed.avro", seed.to_avro()), ("answers.avro", request.to_avro())],
        ) {
            return;
        }
        let request = client.input("PAYLOAD").unwrap_or(request);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        let saved = session.draft.clone();
        let entries: Vec<_> = saved
            .income
            .iter()
            .filter(|entry| !crate::k1_items::k1_sale_ids(&entry.income).is_empty())
            .collect();
        assert_eq!(entries.len(), 2);
        let ids = crate::k1_items::k1_sale_ids(&entries[0].income);
        let foreign = crate::k1_items::k1_sale_ids(&entries[1].income)[0];
        assert_eq!(ids.len(), if disjoint40 { 40 } else { 2 });
        assert!(ids.iter().all(|id| *id > 0 && *id < saved.next_id));
        assert_ne!(ids[0], ids[1]);
        assert!(!ids.contains(&foreign));
        let sales = |income: &FederalIncome| match income {
            FederalIncome::Partnership { data } => data.section179_sales.clone(),
            FederalIncome::Scorp { data } => data.section179_sales.clone(),
            _ => unreachable!(),
        };
        let actual = sales(&entries[0].income);
        for amt in [false, true] {
            if unavailable && amt {
                assert_eq!(actual[0].monetary_view(true), None);
                assert_eq!(
                    crate::K1Section179Sale::views(&actual, true).err().unwrap().0,
                    "ISSUE_K1_SALE_AMT_UNAVAILABLE"
                );
                assert_eq!(
                    crate::k1_sales::facts(entries[0], true, None).err().unwrap().0,
                    "ISSUE_K1_SALE_AMT_UNAVAILABLE"
                );
                continue;
            }
            let views: Vec<_> = crate::K1Section179Sale::views(&actual, amt).unwrap();
            if disjoint40 {
                assert_eq!(views.len(), 20);
                assert_eq!(
                    views.iter().map(|sale| sale.id).collect::<Vec<_>>(),
                    if amt { ids[20..].to_vec() } else { ids[..20].to_vec() }
                );
            } else {
                assert_eq!(views.iter().map(|sale| sale.id).collect::<Vec<_>>(), ids);
            }
            let values = crate::K1Section179Sale::worksheets(&views, 0, 0).unwrap();
            assert_eq!(
                values.iter().map(|value| value.gain).collect::<Vec<_>>(),
                if disjoint40 {
                    vec![if amt { 6_000 } else { 8_000 }; 20]
                } else if amt {
                    vec![6_000, 9_000]
                } else {
                    vec![8_000, 13_000]
                }
            );
        }
        client.restart(&mut session);
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        client.files("OUTPUT_DIR", &[("reopened.avro", reopened.to_avro())]);
        if let Ok(directory) = std::env::var("TAX_UNIT17_SALE_OUTPUT_DIR") {
            write_native_draft(std::path::Path::new(&directory).join("draft.avro"), &saved);
        }
        let unchanged = client.input("REOPEN_PAYLOAD").unwrap_or(reopened.clone());
        accepted(&send(&mut session, unchanged));
        assert_eq!(session.draft, saved);
        fn page_sales(doc: &mut IncomeDocument) -> &mut Vec<crate::K1Section179Sale> {
            match doc {
                IncomeDocument::K1Partnership(page) => &mut page.more.section179_sales,
                IncomeDocument::K1Scorp(page) => &mut page.more.section179_sales,
                _ => unreachable!(),
            }
        }
        if disjoint40 {
            for amt in [false, true] {
                let mut invalid = reopened.clone();
                let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
                let rows = page_sales(&mut documents.documents[0].document);
                if amt {
                    rows[0].alternative = rows[20].alternative.clone();
                } else {
                    rows[20].regular = rows[0].regular.clone();
                }
                rejected_with(send(&mut session, invalid), "ISSUE_K1_SALE_RECORDS");
                assert_eq!(session.draft, saved);
            }
        }
        for (bad, key) in [
            (ids[1], "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425"),
            (foreign, "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C"),
            (saved.next_id, "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C"),
        ] {
            let mut invalid = reopened.clone();
            let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
            page_sales(&mut documents.documents[0].document)[0].id = bad;
            rejected_with(send(&mut session, invalid), key);
            assert_eq!(session.draft, saved);
        }
        let mut correction = reopened;
        let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
        let rows = page_sales(&mut documents.documents[0].document);
        if absence_correction {
            rows[0].alternative = crate::K1Section179SaleAmtSource::NotReported;
        } else {
            rows.swap(0, 1);
            rows[0].description = "Renamed actual property".into();
        }
        let correction = client.input("CORRECTION_PAYLOAD").unwrap_or(correction);
        accepted(&send(&mut session, correction));
        let current = session.draft.income.iter().find(|entry| entry.id == entries[0].id).unwrap();
        let mut reordered = sales(&current.income);
        reordered.sort_by_key(|sale| sale.id);
        let mut original = actual;
        original.sort_by_key(|sale| sale.id);
        for (current, original) in reordered.iter().zip(&original) {
            assert_eq!(current.id, original.id);
            assert_eq!(current.regular, original.regular);
            if absence_correction && current.id == ids[0] {
                assert_eq!(current.alternative, crate::K1Section179SaleAmtSource::NotReported);
            } else {
                assert_eq!(current.alternative, original.alternative);
            }
            assert_eq!((current.acquired, current.sold), (original.acquired, original.sold));
        }
        assert_eq!(session.draft.next_id, saved.next_id);
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn unit17_k1_sale_amt_unavailable_save_reopen_restart_and_correction() {
    for partner in [true, false] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let mut sale = k1_sale_input_statement();
        sale.alternative = crate::K1Section179SaleAmtSource::Unavailable;
        let page = if partner {
            let mut page = k1_1065("Known regular sale, unknown AMT source");
            page.more.section179_sales = vec![sale.clone()];
            // The parent convention cannot override explicit unknown property data.
            page.amt = PartnerAmtRecords::SameAsRegular;
            IncomeDocument::K1Partnership(page)
        } else {
            let mut page = k1_1120s("Known regular sale, unknown AMT source");
            page.more.section179_sales = vec![sale.clone()];
            page.amt = ScorpAmtRecords::SameAsRegular;
            IncomeDocument::K1Scorp(page)
        };
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        documents.documents = vec![document(0, page)];
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        let saved = session.draft.clone();
        let source = saved
            .income
            .iter()
            .find(|e| !crate::k1_items::k1_sale_ids(&e.income).is_empty())
            .unwrap();
        let sales = |entry: &FederalIncome| match entry {
            FederalIncome::Partnership { data } => data.section179_sales.clone(),
            FederalIncome::Scorp { data } => data.section179_sales.clone(),
            _ => unreachable!(),
        };
        let actual = sales(&source.income);
        assert!(actual[0].id > 0);
        assert_eq!(actual[0].regular, sale.regular);
        assert_eq!(actual[0].monetary_view(true), None);
        assert_eq!(
            crate::k1_sales::facts(source, true, None).err().unwrap().0,
            "ISSUE_K1_SALE_AMT_UNAVAILABLE"
        );
        let stored = native_draft_records(&saved);
        let decoded = draft_from_native_records(saved.tax_year, &stored);
        assert_eq!(native_draft_records(&decoded), stored);
        assert_eq!(decoded, saved);
        let mut session = restart(decoded);
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(session.draft, saved);
        let mut correction = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
        let rows = match &mut documents.documents[0].document {
            IncomeDocument::K1Partnership(page) => &mut page.more.section179_sales,
            IncomeDocument::K1Scorp(page) => &mut page.more.section179_sales,
            _ => unreachable!(),
        };
        rows[0].alternative = crate::K1Section179SaleAmtSource::NotReported;
        accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
        let current = session.draft.income.iter().find(|e| e.id == source.id).unwrap();
        let corrected = sales(&current.income);
        assert_eq!(corrected[0].id, actual[0].id);
        assert_eq!(corrected[0].regular, actual[0].regular);
        assert_eq!(corrected[0].monetary_view(true), Some(None));
        assert!(crate::k1_sales::facts(current, true, None).unwrap().is_none());
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}
