use super::*;
use crate::tests::cross::interview::{
    accepted, cash_farm, document, farm_only_return, farm_save, rejected_with, reopened_farming,
    stored_farm,
};

// Pub 225 (2025), chapter 3, p11, weather-sale example:
// https://www.irs.gov/publications/p225
// Uniformly shift 2024/2025 to 2026/2027; preserve all printed dollar/count
// facts. Prior counts and narrative are explicit completion facts supporting
// the source's stated usual practice, not additional published numbers.
fn weather_example() -> FarmWeatherSale {
    FarmWeatherSale {
        class: FarmLivestockClass::Cattle,
        source: FarmWeatherSource::RaisedForSale,
        normal_sold: 100,
        all_sold: 135,
        weather_excess: 35,
        class_income: 12_150_000,
        principal_farming: true,
        assistance: FarmWeatherAssistance::Designated { date: Some(20260810) },
        weather_relationship: "Drought reduced winter feed on this farm; 35 calves sold at weaning instead of next January.".into(),
        prior_sales: (2023..=2025).map(|year| FarmAnimalYear { year, count: 100 }).collect(),
        usual_practice_support: "Normally sell 100 overwintered calves in January or February; farm sales records.".into(),
        elect: true,
    }
}
fn save_farms(session: &mut Interview, farms: Vec<Farm>) {
    let mut documents =
        farms.into_iter().map(|farm| document(0, IncomeDocument::Farm(farm))).collect::<Vec<_>>();
    crate::tests::farm::interview::accept_farm_sale_parcels(session, &mut documents);
    let before = session.draft.clone();
    accepted(&send(session, farm_save(&before, documents)));
}

fn row(r: &FederalResult, suffix: &str) -> i64 {
    r.lines
        .iter()
        .find(|line| line.id.ends_with(suffix))
        .unwrap_or_else(|| panic!("missing {suffix}"))
        .amount
}
fn form(r: &FederalResult, line: &str) -> i64 {
    r.lines
        .iter()
        .filter(|item| {
            item.form_line
                .as_ref()
                .is_some_and(|tag| tag.form == TaxForm::ScheduleF && tag.line == line)
        })
        .map(|item| item.amount)
        .sum()
}
fn assert_cash_receipts_foot(r: &FederalResult) {
    assert_eq!(form(r, "1c"), form(r, "1a") - form(r, "1b"));
    assert_eq!(
        form(r, "9"),
        ["1c", "2", "3b", "4b", "5b", "6b", "7", "8"]
            .into_iter()
            .map(|line| form(r, line))
            .sum::<i64>()
    );
}
#[test]
fn farm_special_weather_irs_example_is_public_and_reopens_with_exact_source_facts() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_150_000, 0);
    farm.more.specials.weather_sales = vec![weather_example()];
    save_farms(&mut session, vec![farm.clone()]);
    let report = supported(&session.draft);
    assert_eq!(row(&report, "_weather_postponed"), 3_150_000);
    assert_eq!(form(&report, "2"), 9_000_000);
    assert_eq!(form(&report, "9"), 9_000_000);
    assert_cash_receipts_foot(&report);
    assert_eq!(reopened_farming(&mut session).1, vec![IncomeDocument::Farm(farm)]);
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
}
#[test]
fn farm_special_weather_prior_is_included_once_without_inflating_current_class_average() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_150_000, 0);
    farm.more.specials.weather_sales = vec![weather_example()];
    farm.more.specials.prior_weather = vec![FarmPriorWeather {
        year: 2025,
        class: FarmLivestockClass::Cattle,
        source: FarmWeatherSource::RaisedForSale,
        deferred: 3_150_000,
    }];
    save_farms(&mut session, vec![farm]);
    let report = supported(&session.draft);
    assert_eq!(row(&report, "_weather_postponed"), 3_150_000);
    assert_eq!(row(&report, "_weather_prior_included"), 3_150_000);
    assert_eq!(form(&report, "2"), 12_150_000);
    assert_cash_receipts_foot(&report);
}
#[test]
fn farm_special_weather_mixed_products_preserves_purchased_basis_and_actual_unknown_designation_date()
 {
    // Derived product allocation probe, not another IRS worked example.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(6_075_000, 0);
    farm.accounting = FarmMethod::Cash {
        raised: 6_075_000,
        resale: 8_075_000,
        resale_cost: 2_000_000,
        program: 0,
        custom_hire: 0,
        other: 0,
    };
    let mut sale = weather_example();
    sale.source = FarmWeatherSource::MixedProducts { raised: 6_075_000 };
    sale.assistance = FarmWeatherAssistance::Designated { date: None };
    farm.more.specials.weather_sales = vec![sale];
    save_farms(&mut session, vec![farm]);
    let report = supported(&session.draft);
    assert_eq!(form(&report, "1a"), 6_500_000);
    assert_eq!(form(&report, "1b"), 2_000_000);
    assert_eq!(form(&report, "1c"), 4_500_000);
    assert_eq!(form(&report, "2"), 4_500_000);
    assert_cash_receipts_foot(&report);
}
#[test]
fn farm_special_weather_unestablished_relief_and_shared_class_preserve_known_income() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_150_000, 0);
    let mut sale = weather_example();
    sale.assistance = FarmWeatherAssistance::NotDesignated;
    farm.more.specials.weather_sales = vec![sale];
    save_farms(&mut session, vec![farm]);
    let report = supported(&session.draft);
    assert_eq!(form(&report, "9"), 12_150_000);
    assert!(noted(&report, "NOTE_FARM_WEATHER_NOT_POSTPONED"));
    // Each independent farm claims the same generic class. Until its actual
    // taxpayer-wide statement is reconciled, omit only this deferral.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_150_000, 0);
    farm.more.specials.weather_sales = vec![weather_example()];
    save_farms(&mut session, vec![farm.clone(), farm]);
    let accepted_draft = session.draft.clone();
    let report = supported(&session.draft);
    assert_eq!(form(&report, "9"), 24_300_000);
    assert!(noted(&report, "WARN_FARM_WEATHER_SHARED_CLASS"));
    assert_eq!(session.draft, accepted_draft);
}
#[test]
fn farm_special_weather_server_rejects_overlapping_count_or_income_without_consuming_state() {
    let original = farm_only_return();
    for amount in [12_150_000, 12_150_001] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut farm = cash_farm(12_150_000, 0);
        let mut sale = weather_example();
        sale.class_income = amount;
        if amount == 12_150_000 {
            sale.weather_excess = 36;
        }
        farm.more.specials.weather_sales = vec![sale];
        rejected_with(
            send(&mut session, farm_save(&original, vec![document(0, IncomeDocument::Farm(farm))])),
            if amount == 12_150_000 {
                "FARM_WEATHER_COUNTS"
            } else {
                "FARM_WEATHER_ABOVE_PRODUCT_INCOME"
            },
        );
        assert_eq!(session.draft, original);
    }
}
#[test]
fn farm_special_signed_current_hedge_and_irrigation_feed_other_income_once() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    farm.more.specials.other_income = vec![
        FarmOtherIncome::Hedge {
            facts: FarmHedge {
                contract: "Corn hedge settlement".into(),
                identified: 20260102,
                entered: 20260102,
                settled: 20260301,
                settlement: -50_000,
                business_risk: "Offsets price risk on growing corn inventory.".into(),
                timing: FarmHedgeTiming::CurrentSingleYear,
            },
        },
        FarmOtherIncome::Irrigation { amount: FarmPositiveReceipt { amount: 10_000 } },
        FarmOtherIncome::Other {
            facts: FarmSignedReceipt {
                description: "Ordinary farm receipt".into(),
                amount: 20_000,
            },
        },
    ];
    save_farms(&mut session, vec![farm]);
    let report = supported(&session.draft);
    assert_eq!(form(&report, "8"), -20_000);
    assert_eq!(form(&report, "9"), 980_000);
    assert_cash_receipts_foot(&report);
}
#[test]
fn farm_special_land_identity_is_assigned_only_by_accepted_save_and_survives_reordering() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    let parcel = |label: &str| FarmLandBasis {
        id: 0,
        label: label.into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20260101,
        regular: 2_000_000,
        amt: 2_000_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    };
    farm.more.land_basis = vec![parcel("Same name"), parcel("Same name")];
    save_farms(&mut session, vec![farm]);
    let accepted_ids: Vec<_> =
        stored_farm(&session.draft).1.land_basis.iter().map(|land| land.id).collect();
    assert!(accepted_ids.iter().all(|id| *id > 0));
    assert_ne!(accepted_ids[0], accepted_ids[1]);
    let before = session.draft.clone();
    let mut farms = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut farms[0] else { unreachable!() };
    farm.more.land_basis.reverse();
    farm.more.land_basis[0].label = "Renamed".into();
    let id = session.draft.income[0].id;
    accepted(&send(&mut session, farm_save(&before, vec![document(id, farms[0].clone())])));
    assert_eq!(
        stored_farm(&session.draft).1.land_basis.iter().map(|land| land.id).collect::<Vec<_>>(),
        vec![accepted_ids[1], accepted_ids[0]]
    );
    assert_eq!(
        stored_farm(&restart(session.draft.clone()).draft).1.land_basis,
        stored_farm(&session.draft).1.land_basis
    );
    let before = session.draft.clone();
    let IncomeDocument::Farm(farm) = &mut farms[0] else { unreachable!() };
    farm.more.land_basis[0].id = 9999;
    rejected_with(
        current_income_refusal(
            "income-source-607",
            &mut session,
            farm_save(&before, vec![document(id, farms[0].clone())]),
            "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
            "[documents/documents/0/document/more/land_basis/0]",
        ),
        "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn farm_special_required_business_animal_or_multiyear_hedge_character_is_not_silently_omitted() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(0, 0);
    let mut sale = weather_example();
    sale.source = FarmWeatherSource::IncludesBusinessAnimals;
    sale.elect = false;
    farm.more.specials.weather_sales = vec![sale];
    save_farms(&mut session, vec![farm]);
    unsupported(&session.draft, "Farm source timing and character");
    // Same-year identification is not enough to establish multiyear timing.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    farm.more.specials.other_income = vec![FarmOtherIncome::Hedge {
        facts: FarmHedge {
            contract: "Corn hedge settlement".into(),
            identified: 20251231,
            entered: 20251231,
            settled: 20260301,
            settlement: -50_000,
            business_risk: "Growing corn price risk".into(),
            timing: FarmHedgeTiming::CurrentSingleYear,
        },
    }];
    save_farms(&mut session, vec![farm]);
    unsupported(&session.draft, "Farm source timing and character");
}
#[test]
fn farm_special_designation_may_follow_sale_year_and_new_farm_does_not_invent_prior_sales() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_150_000, 0);
    let mut sale = weather_example();
    sale.assistance = FarmWeatherAssistance::Designated { date: Some(20270201) };
    sale.prior_sales.clear();
    sale.usual_practice_support = "New farm: 100 normal annual sales supported by retained records of similarly situated local farms.".into();
    farm.more.specials.weather_sales = vec![sale];
    save_farms(&mut session, vec![farm]);
    assert_eq!(row(&supported(&session.draft), "_weather_postponed"), 3_150_000);
}

// Pub225(2025), chapter3, pp14–15, cost-sharing example. Printed $500,000
// total/$490,000 USDA/$10,000 owner/$15,000 improvement/$1,380 and$1,550
// income-right appraisals. Exact day, old parcel opening basis and preceding
// gross receipts are neutral completion facts; no discount rate is invented.
fn cost_sharing_example(election: Farm126Election, property: i64) -> FarmCostSharingInput {
    FarmCostSharingInput {
        installation_year: 2026,
        description: "NRCS reclaimed 100 acres".into(),
        program:
            "Rural abandoned mine program; retained USDA contract and conservation certification"
                .into(),
        program_approved: true,
        conservation_certified: true,
        government_received: 49_000_000,
        total_cost: 50_000_000,
        owner_contribution: 1_000_000,
        fmv_improvement: 1_500_000,
        non126_program: 0,
        uncertified: 0,
        rent_services: 0,
        current_deductible: 0,
        excluded_funding_applied: None,
        election,
        received: 20260901,
        asset: property,
    }
}
pub(in crate::tests) fn cost_sharing_facts(
    value: Option<FarmIncomeRightValue>,
) -> Farm126Exclusion {
    Farm126Exclusion {
        acres_hundredths: 10_000,
        prior_income: (2023..=2025)
            .map(|year| FarmAffectedIncome { year, income: 222_580 })
            .collect(),
        valuation: value,
    }
}
pub(in crate::tests) fn cost_sharing_valuation() -> FarmIncomeRightValue {
    FarmIncomeRightValue { ten_percent_value: 138_000, acre_value: 155_000, support: "Retained appraiser's income-right present values and property/source assumptions; use the actual valuations, not an invented discount formula.".into() }
}
pub(in crate::tests) fn cost_sharing_session(election: Farm126Election) -> Interview {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    farm.more.land_basis = vec![FarmLandBasis {
        id: 0,
        label: "Reclaimed parcel".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20000101,
        regular: 20_000_000,
        amt: 20_000_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }];
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let property = stored_farm(&before).1.land_basis[0].id;
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    farm.more.specials.cost_sharing = vec![cost_sharing_example(election, property)];
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, pages.remove(0))]),
    ));
    session
}
#[test]
fn farm_126_irs_example_uses_supported_appraisal_and_adjusts_one_canonical_parcel() {
    let session = cost_sharing_session(Farm126Election::Exclude {
        facts: cost_sharing_facts(Some(cost_sharing_valuation())),
    });
    let result = supported(&session.draft);
    assert_eq!(form(&result, "4a"), 49_000_000);
    assert_eq!(form(&result, "4b"), 345_000);
    assert_eq!(row(&result, "_costsharing_0_value"), 1_500_000);
    assert_eq!(row(&result, "_costsharing_0_basis"), 1_345_000);
    assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 155_000);
    let farm = stored_farm(&session.draft).1;
    assert_eq!(farm.parcel_basis(farm.land_basis[0].id, false), Some(21_345_000));
    assert_eq!(farm.parcel_basis(farm.land_basis[0].id, true), Some(21_345_000));
    assert_cash_receipts_foot(&result);
    assert_eq!(supported(&restart(session.draft.clone()).draft), result);
}
#[test]
fn farm_126_actual_full_no_exclusion_election_needs_no_irrelevant_appraisal_or_prior_income() {
    let session = cost_sharing_session(Farm126Election::NotExcludeAll);
    let result = supported(&session.draft);
    // Pub225 explicitly prints the no-exclusion $5,000/$15,000 result.
    assert_eq!(form(&result, "4b"), 500_000);
    assert_eq!(row(&result, "_costsharing_0_basis"), 1_500_000);
    assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 0);
    assert!(!noted(&result, "NOTE_FARM126_VALUATION_OMITTED"));
    assert_cash_receipts_foot(&result);
}
#[test]
fn farm_126_missing_appraisal_omits_exclusion_without_inventing_property_basis() {
    let session =
        cost_sharing_session(Farm126Election::Exclude { facts: cost_sharing_facts(None) });
    let result = supported(&session.draft);
    assert_eq!(form(&result, "4b"), 500_000);
    assert!(noted(&result, "NOTE_FARM126_VALUATION_OMITTED"));
    let farm = stored_farm(&session.draft).1;
    assert_eq!(farm.parcel_basis(farm.land_basis[0].id, false), None);
    assert!(!result.lines.iter().any(|row| row.id.ends_with("_costsharing_0_basis")));
    assert_cash_receipts_foot(&result);
}
#[test]
fn farm_126_partial_election_is_bounded_by_actual_appraisal_and_stale_property_is_rejected() {
    // Derived partial-election probe:1550−500=1050;15000−10000−1050=3950.
    let mut session = cost_sharing_session(Farm126Election::NotExcludePart {
        amount: 50_000,
        facts: cost_sharing_facts(Some(cost_sharing_valuation())),
    });
    let result = supported(&session.draft);
    assert_eq!(form(&result, "4b"), 395_000);
    assert_eq!(row(&result, "_costsharing_0_basis"), 1_395_000);
    assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 105_000);
    let before = session.draft.clone();
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    farm.more.land_basis.clear();
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, pages.remove(0))]),
        ),
        "FARM126_PROPERTY_SOURCE",
    );
    assert_eq!(session.draft, before);
}

// Synthetic, independently computed mixed-financing control under approved R2:
// $10,000 cost = $9,000 government + $1,000 owner; $1,000 is nonqualified.
// $6,000 FMV * $9,000/$10,000 = $5,400 qualified improvement value;
// - $1,000 owner - $1,000 supported PV + $1,000 nonqualified = $4,400 taxable.
// Actual excluded funding applied is independently $4,000, giving $6,000
// basis, not owner + taxable ($5,400) or cost - PV ($9,000).
fn mixed_cost_sharing(property: i64, excluded: Option<i64>) -> FarmCostSharingInput {
    let mut source = cost_sharing_example(
        Farm126Election::Exclude {
            facts: cost_sharing_facts(Some(FarmIncomeRightValue {
                ten_percent_value: 100_000,
                acre_value: 90_000,
                support:
                    "Retained independent income-right appraisal: present values $1,000 and $900."
                        .into(),
            })),
        },
        property,
    );
    source.description =
        "Mixed-financed conservation improvement; retained allocation statement".into();
    source.government_received = 900_000;
    source.total_cost = 1_000_000;
    source.owner_contribution = 100_000;
    source.fmv_improvement = 600_000;
    source.non126_program = 100_000;
    source.excluded_funding_applied = excluded;
    source
}
fn mixed_cost_sharing_editor(session: &mut Interview, excluded: Option<i64>) -> (P, P) {
    let seed = crate::tests::cross::interview::open_topic(session, TaxSection::Farming);
    let mut request = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { unreachable!() };
    let property = farm.more.land_basis[0].id;
    farm.more.specials.cost_sharing = vec![mixed_cost_sharing(property, excluded)];
    (seed, request)
}
fn empty_cost_sharing_session() -> Interview {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    farm.more.land_basis = vec![FarmLandBasis {
        id: 0,
        label: "Mixed-financing parcel".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20000101,
        regular: 20_000_000,
        amt: 20_000_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }];
    save_farms(&mut session, vec![farm]);
    session
}
// Optional ordinary Avro bridge: actual Studio fresh-row answers must pass
// this same normal Save, accepted record, reopen and restart lifecycle.
fn farm126_client_artifacts(session: &mut Interview, seed: &P, request: &mut P) -> bool {
    extern crate std;
    if std::env::var("TAX_B11_FARM126_CASE").as_deref() != Ok("mixed") {
        return false;
    }
    if let Ok(path) = std::env::var("TAX_B11_FARM126_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(
            calculation,
            P::EstimateDocuments { .. }
                | P::DocumentPropertyOptions { .. }
                | P::PreviewDocument { .. }
        ));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(session, calculation) else {
            panic!("expected normal farm calculation reply")
        };
        assert!(matches!(
            reply,
            P::RunningEstimate { .. } | P::DocumentPropertyChoices { .. } | P::IncomePreview { .. }
        ));
        assert_eq!(session.draft, before);
        std::fs::write(std::env::var("TAX_B11_FARM126_CALC_OUTPUT").unwrap(), reply.to_avro())
            .unwrap();
        return true;
    }
    for (name, value) in [("SEED", seed), ("ANSWERS", &*request)] {
        if let Ok(path) = std::env::var(alloc::format!("TAX_B11_FARM126_{name}")) {
            std::fs::write(path, value.to_avro()).unwrap();
        }
    }
    if let Ok(path) = std::env::var("TAX_B11_FARM126_PAYLOAD") {
        *request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveIncomes { .. }));
    }
    false
}
#[test]
fn farm_126_mixed_funding_source_saves_reopens_restarts_and_corrects_one_basis() {
    let mut session = empty_cost_sharing_session();
    let (seed, mut request) = mixed_cost_sharing_editor(&mut session, Some(400_000));
    if farm126_client_artifacts(&mut session, &seed, &mut request) {
        return;
    }
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(form(&result, "4a"), 900_000);
    assert_eq!(form(&result, "4b"), 440_000);
    assert_eq!(row(&result, "_costsharing_0_value"), 540_000);
    assert_eq!(row(&result, "_costsharing_0_basis"), 600_000);
    assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 100_000);
    assert_cash_receipts_foot(&result);
    let farm = stored_farm(&session.draft).1;
    let property = farm.land_basis[0].id;
    let original = farm.specials.cost_sharing[0].clone();
    assert_eq!(original.excluded_funding_applied, Some(400_000));
    assert_eq!(farm.parcel_basis(property, false), Some(20_600_000));
    assert_eq!(farm.parcel_basis(property, true), Some(20_600_000));
    let reopened = crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming);
    extern crate std;
    if std::env::var("TAX_B11_FARM126_CASE").as_deref() == Ok("mixed")
        && let Ok(path) = std::env::var("TAX_B11_FARM126_REOPEN")
    {
        std::fs::write(path, reopened.to_avro()).unwrap();
    }
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    let IncomeDocument::Farm(page) = &documents.documents[0].document else { unreachable!() };
    assert_eq!(FarmCostSharing::from(&page.more.specials.cost_sharing[0]), original);
    let restarted = restart(session.draft.clone());
    assert_eq!(stored_farm(&restarted.draft).1.specials.cost_sharing[0], original);
    assert_eq!(supported(&restarted.draft), result);
    let (_, correction) = mixed_cost_sharing_editor(&mut session, Some(300_000));
    accepted(&send(&mut session, correction));
    let corrected = supported(&session.draft);
    assert_eq!(form(&corrected, "4b"), 440_000);
    assert_eq!(row(&corrected, "_costsharing_0_recapture_portion"), 100_000);
    assert_eq!(row(&corrected, "_costsharing_0_basis"), 700_000);
    assert_eq!(stored_farm(&session.draft).1.parcel_basis(property, false), Some(20_700_000));
}
#[test]
fn farm_126_mixed_missing_source_is_unresolved_and_zero_is_an_answer() {
    let mut session = empty_cost_sharing_session();
    let (_, missing) = mixed_cost_sharing_editor(&mut session, None);
    accepted(&send(&mut session, missing));
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ISSUE_FARM126_MIXED_SOURCE_OWNER")
    );
    let farm = stored_farm(&session.draft).1;
    assert_eq!(farm.parcel_basis(farm.land_basis[0].id, false), None);
    let (_, zero) = mixed_cost_sharing_editor(&mut session, Some(0));
    accepted(&send(&mut session, zero));
    let result = supported(&session.draft);
    assert_eq!(row(&result, "_costsharing_0_basis"), 1_000_000);
    assert_eq!(form(&result, "4b"), 440_000);
}
#[test]
fn farm_126_mixed_funding_boundaries_reject_without_changing_accepted_sources() {
    let mut session = empty_cost_sharing_session();
    let (_, valid) = mixed_cost_sharing_editor(&mut session, Some(900_000));
    accepted(&send(&mut session, valid));
    assert_eq!(row(&supported(&session.draft), "_costsharing_0_basis"), 100_000);
    for amount in [-1, 900_001, 1_000_001, 100_000_000_001] {
        let before = session.draft.clone();
        let (_, invalid) = mixed_cost_sharing_editor(&mut session, Some(amount));
        rejected_with(send(&mut session, invalid), "FARM126_EXCLUDED_FUNDING_ABOVE_SOURCE");
        assert_eq!(session.draft, before);
    }
}
#[test]
fn farm_126_mixed_known_nonexcluded_sources_require_zero_funding_atomically() {
    for branch in 0..3 {
        let mut session = empty_cost_sharing_session();
        let (_, mut request) = mixed_cost_sharing_editor(&mut session, Some(1));
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let source = &mut farm.more.specials.cost_sharing[0];
        match branch {
            0 => source.program_approved = false,
            1 => source.conservation_certified = false,
            _ => source.election = Farm126Election::NotExcludeAll,
        }
        let before = session.draft.clone();
        rejected_with(send(&mut session, request.clone()), "FARM126_FUNDING_WITHOUT_EXCLUSION");
        assert_eq!(session.draft, before);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else {
            unreachable!()
        };
        farm.more.specials.cost_sharing[0].excluded_funding_applied = Some(0);
        accepted(&send(&mut session, request));
        let result = supported(&session.draft);
        assert_eq!(form(&result, "4b"), if branch < 2 { 900_000 } else { 540_000 });
        assert_eq!(row(&result, "_costsharing_0_basis"), 1_000_000);
        assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 0);
    }
}

#[test]
fn farm_126_mixed_categories_are_taxable_separately_from_the_improvement_value() {
    // Reg16A.126-1(g) examples1-3 / approved map fixtureE:
    // 700000 - 34500 - 150000 - 15500 = 500000 qualified cost;
    // 21000 * 500000 / 700000 = 15000 value; -10000 owner -1550 PV
    // =3450 qualified income. Separately taxable components total200000.
    let mut session = empty_cost_sharing_session();
    let (_, mut request) = mixed_cost_sharing_editor(&mut session, Some(48_655_000));
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { unreachable!() };
    let source = &mut farm.more.specials.cost_sharing[0];
    *source = cost_sharing_example(
        Farm126Election::Exclude { facts: cost_sharing_facts(Some(cost_sharing_valuation())) },
        source.asset,
    );
    source.total_cost = 70_000_000;
    source.government_received = 69_000_000;
    source.fmv_improvement = 2_100_000;
    source.uncertified = 3_450_000;
    source.rent_services = 15_000_000;
    source.current_deductible = 1_550_000;
    source.excluded_funding_applied = Some(48_655_000);
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(form(&result, "4a"), 69_000_000);
    assert_eq!(form(&result, "4b"), 20_345_000);
    assert_eq!(row(&result, "_costsharing_0_value"), 1_500_000);
    assert_eq!(row(&result, "_costsharing_0_basis"), 21_345_000);
    assert_eq!(row(&result, "_costsharing_0_recapture_portion"), 155_000);
    assert_cash_receipts_foot(&result);
}

fn parcel_sale_session(mut parcel: FarmLandBasis, payment: crate::SalePayment) -> Interview {
    let initial = farm_only_return();
    let mut session = Interview::new(TEST_USER, initial);
    let mut farm = cash_farm(0, 0);
    parcel.id = 0;
    farm.more.land_basis = vec![parcel];
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let property = stored_farm(&before).1.land_basis[0].id;
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    farm.more.land_sales = vec![LandSale {
        id: 0,
        basis_property: property,
        label: "Disposal of accepted parcel".into(),
        sold: 20260410,
        proceeds: 7_500_000,
        selling: 0,
        payment,
        installments: LandInstallments::No,
        situation: FarmLandSituation::Direct,
    }];
    accepted(&send(
        &mut session,
        farm_save(
            &before,
            vec![
                document(before.income[0].id, pages.remove(0)),
                document(
                    0,
                    IncomeDocument::BusinessPropertyLosses(crate::BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                ),
            ],
        ),
    ));
    session
}
fn section1255_parcel() -> FarmLandBasis {
    FarmLandBasis {
        id: 0,
        label: "Section126 property".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20000101,
        regular: 5_250_000,
        amt: 5_250_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![Farm126History {
            received: 20210105,
            excluded: 1_800_000,
            statement: "Original retained section126 exclusion statement, last certified payment"
                .into(),
        }],
    }
}

#[test]
fn farm_1255_regulation_example_uses_canonical_basis_and_retains_sale_identity() {
    // Official2025CFR26vol16 printedp170, Reg16A.1255-1(d):
    // https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol16/pdf/CFR-2025-title26-vol16-sec16A-1255-2.pdf
    // Uniformly shift1990/1995 to2021/2026; exact75000price,52500basis,
    // 22500gain,18000ordinary and4500section1231 are printed source figures.
    // Acquisition in2000 and no earlier1231losses are neutral completion facts.
    let mut session = parcel_sale_session(section1255_parcel(), crate::SalePayment::InFull);
    let result = supported(&session.draft);
    assert_eq!(row(&result, "_land_0_gain"), 2_250_000);
    assert_eq!(row(&result, "_land_0_conservation_recapture"), 0);
    assert_eq!(row(&result, "_land_0_cost_sharing_recapture"), 1_800_000);
    assert_eq!(row(&result, "_land_0_section1231"), 450_000);
    let (_, farm) = stored_farm(&session.draft);
    let parcel = farm.land_basis[0].id;
    let sale = farm.land_sales[0].id;
    assert!(sale > 0 && sale != parcel);
    assert_eq!(farm.land_sales[0].basis_property, parcel);
    let before = session.draft.clone();
    let pages = reopened_farming(&mut session).1;
    accepted(&send(
        &mut session,
        farm_save(
            &before,
            pages
                .into_iter()
                .map(|page| {
                    let id = if matches!(page, IncomeDocument::Farm(_)) {
                        before.income[0].id
                    } else {
                        0
                    };
                    document(id, page)
                })
                .collect(),
        ),
    ));
    assert_eq!(session.draft.revision, before.revision);
    let restored = restart(session.draft.clone());
    assert_eq!(stored_farm(&restored.draft).1.land_sales[0].id, sale);
    assert_eq!(supported(&restored.draft).tax, result.tax);
}

#[test]
fn farm_1255_current_installment_gain_takes_1252_then_1255_once() {
    // Derived priority probe, not a printed IRS dollar example:
    // Reg16A.1255-1(c)(3) ordinary gain follows each payment, otherPartIV
    // recapture before1255.5000prior175*60%=3000, then18000section1255.
    let mut parcel = section1255_parcel();
    parcel.acquired = 20200101;
    parcel.prior_conservation = 500_000;
    parcel.prior_amt_conservation = 500_000;
    let session = parcel_sale_session(
        parcel,
        crate::SalePayment::Installment {
            assumed_debt: 0,
            payments: 3_750_000,
            interest: crate::NoteInterest::Adequate,
            special: false,
        },
    );
    let result = supported(&session.draft);
    assert_eq!(row(&result, "_land_0_form6252_income"), 1_125_000);
    assert_eq!(row(&result, "_land_0_conservation_recapture"), 300_000);
    assert_eq!(row(&result, "_land_0_cost_sharing_recapture"), 825_000);
    assert_eq!(row(&result, "_land_0_section1231"), 0);
    assert_eq!(row(&result, "_land_0_form6252_recapture"), 0);
    assert_eq!(row(&result, "_land_0_conservation_recapture_remaining"), 0);
    assert_eq!(row(&result, "_land_0_cost_sharing_recapture_remaining"), 975_000);
}

#[test]
fn farm_1255_applicable_percentage_uses_actual_payment_anniversary() {
    // Derived boundary probes under Reg16A.1255-1(a)(4); exact calendar
    // anniversaries use the shared date owner, not numeric date subtraction.
    for (received, ordinary) in
        [(20160410, 1_800_000), (20160409, 1_620_000), (20070410, 180_000), (20060410, 0)]
    {
        let mut parcel = section1255_parcel();
        parcel.prior_cost_sharing[0].received = received;
        let session = parcel_sale_session(parcel, crate::SalePayment::InFull);
        let result = supported(&session.draft);
        assert_eq!(row(&result, "_land_0_cost_sharing_recapture"), ordinary);
    }
}

#[test]
fn farm_1255_sale_identity_cannot_take_its_parcel_identity_and_failed_save_is_pure() {
    let mut session = parcel_sale_session(section1255_parcel(), crate::SalePayment::InFull);
    let before = session.draft.clone();
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    farm.more.land_sales[0].id = farm.more.land_basis[0].id;
    rejected_with(
        current_income_refusal(
            "income-source-516",
            &mut session,
            farm_save(
                &before,
                pages
                    .into_iter()
                    .map(|page| {
                        let id = if matches!(page, IncomeDocument::Farm(_)) {
                            before.income[0].id
                        } else {
                            0
                        };
                        document(id, page)
                    })
                    .collect(),
            ),
            "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
            "[documents/documents/0/document/more/land_sales/0]",
        ),
        "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn farm_126_unknown_current_basis_prevents_a_linked_sale_from_claiming_known_gain() {
    let mut session =
        cost_sharing_session(Farm126Election::Exclude { facts: cost_sharing_facts(None) });
    let before = session.draft.clone();
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    farm.more.land_sales = vec![LandSale {
        id: 0,
        basis_property: farm.more.land_basis[0].id,
        label: "Improved parcel sold".into(),
        sold: 20261210,
        proceeds: 25_000_000,
        selling: 0,
        payment: crate::SalePayment::InFull,
        installments: LandInstallments::No,
        situation: FarmLandSituation::Direct,
    }];
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, pages.remove(0))]),
    ));
    let accepted = session.draft.clone();
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED")
    );
    assert_eq!(session.draft, accepted);
}

pub(in crate::tests) fn fertilizer_source() -> FarmFertilizerCost {
    FarmFertilizerCost {
        description: "Capital soil conditioning materials and application".into(),
        cost: 300_000,
        paid_or_incurred: 20260501,
        land_use: FarmFertilizerLand::PreviouslyFarmed,
        initial_preparation: false,
        applied: 20260502,
        treatment: FarmFertilizerTreatment::Deduct180,
    }
}
#[test]
fn farm_180_actual_current_election_joins_ordinary_fertilizer_expense_once() {
    // Derived arithmetic probe under Reg1.180-1/2,2025CFRvol4pp247–248:
    // https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol4/pdf/CFR-2025-title26-vol4-sec1-180-2.pdf
    // Claiming the current deduction is the election for that tax year.
    // $1000 ordinary +$3000 elected capital fertilizer=$4000 onF17.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(4_000_000, 0);
    farm.expenses.fertilizers = 100_000;
    farm.more.specials.fertilizer = vec![fertilizer_source()];
    save_farms(&mut session, vec![farm]);
    let result = supported(&session.draft);
    assert_eq!(form(&result, "17"), 400_000);
    assert_eq!(form(&result, "33"), 400_000);
    assert_eq!(form(&result, "34"), 3_600_000);
    assert_eq!(supported(&restart(session.draft.clone()).draft).tax, result.tax);
}
#[test]
fn farm_rental_180_uses_the_same_owner_and_form4835_expense() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut rental = crate::tests::farm::farm_rental::rental_page(40_000);
    rental.expenses.fertilizers = 100_000;
    rental.more.specials.fertilizer = vec![fertilizer_source()];
    let before = session.draft.clone();
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(0, IncomeDocument::FarmRental(rental))]),
    ));
    let result = supported(&session.draft);
    let total = |line: &str| {
        result
            .lines
            .iter()
            .filter(|row| {
                row.form_line
                    .as_ref()
                    .is_some_and(|tag| tag.form == TaxForm::Form4835 && tag.line == line)
            })
            .map(|row| row.amount)
            .sum::<i64>()
    };
    assert_eq!(total("15"), 400_000);
    assert_eq!(total("31"), 400_000);
    assert_eq!(total("32"), 3_600_000);
}
#[test]
fn farm_180_initial_preparation_keeps_known_income_without_claiming_deduction() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(4_000_000, 0);
    let mut cost = fertilizer_source();
    cost.initial_preparation = true;
    cost.land_use = FarmFertilizerLand::NotYetFarmed;
    farm.more.specials.fertilizer = vec![cost];
    save_farms(&mut session, vec![farm]);
    let result = supported(&session.draft);
    assert_eq!(form(&result, "17"), 0);
    assert_eq!(form(&result, "34"), 4_000_000);
    assert!(noted(&result, "NOTE_FARM_FERTILIZER_180_NOT_ELIGIBLE"));
}
#[test]
fn farm_fertilizer_capital_history_does_not_invent_a_monthly_recovery_rule() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(4_000_000, 0);
    let mut cost = fertilizer_source();
    cost.treatment = FarmFertilizerTreatment::Capitalize {
        recovery: FarmFertilizerRecovery { benefit_months: 36, prior_recovered: 0 },
    };
    farm.more.specials.fertilizer = vec![cost];
    save_farms(&mut session, vec![farm]);
    let result = supported(&session.draft);
    assert_eq!(form(&result, "17"), 0);
    assert!(noted(&result, "NOTE_FARM_FERTILIZER_RECOVERY_OMITTED"));
}

// Derived identity/graph probes. The existing Form 3115 owner, not the link,
// computes each adjustment; all dollar facts here are synthetic.
fn method_record(year: u32) -> MethodChange {
    MethodChange {
        id: 0,
        year,
        consent: MethodChangeConsent::Automatic { dcn: 122 },
        adjustment: Adjustment481::FourYears { amount: 400_000, rest_in_current_year: false },
        amt: crate::MethodChangeAmt::Same,
    }
}
#[test]
fn farm_special_method_sources_keep_real_ids_through_reorder_rename_and_restart() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(3_000_000, 0);
    farm.more.method_changes = vec![method_record(2024), method_record(2025)];
    save_farms(&mut session, vec![farm]);
    let saved = stored_farm(&session.draft).0.method_changes.clone();
    assert!(saved.iter().all(|record| record.id > 0));
    assert_ne!(saved[0].id, saved[1].id);
    let next_id = session.draft.next_id;
    let mut request = crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, farm_method_changes, .. } = &mut request else { panic!() };
    assert_eq!(
        farm_method_changes.iter().map(|item| item.id).collect::<Vec<_>>(),
        saved.iter().map(|record| record.id).collect::<Vec<_>>()
    );
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { panic!() };
    farm.name = "Renamed farm".into();
    farm.more.method_changes.reverse();
    farm.more.specials.method_status.change = Some(FarmMethodChangeLinkInput {
        change: saved[0].id,
        approval: "Actual retained automatic-change statement".into(),
    });
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.next_id, next_id);
    let (business, farm) = stored_farm(&session.draft);
    assert_eq!(business.method_changes, vec![saved[1].clone(), saved[0].clone()]);
    assert_eq!(farm.specials.method_status.change.as_ref().unwrap().change, saved[0].id);
    let accepted_draft = session.draft.clone();
    let result = supported(&accepted_draft);
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(supported(&restart(accepted_draft.clone()).draft), result);
    assert_eq!(restart(accepted_draft.clone()).draft, accepted_draft);
}
#[test]
fn farm_special_method_source_rejects_sibling_deleted_and_unconfirmed_links_without_mutation() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(3_000_000, 0);
    farm.more.method_changes = vec![method_record(2024)];
    save_farms(&mut session, vec![farm.clone(), farm]);
    let first = crate::method_change::records(&session.draft.income[0].income).unwrap()[0].id;
    let sibling = crate::method_change::records(&session.draft.income[1].income).unwrap()[0].id;
    for (case, mode) in
        [("income-source-576", 0), ("income-source-585", 1), ("income-source-595", 3)]
    {
        let before = session.draft.clone();
        let mut request =
            crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming);
        let P::SaveIncomes { documents, .. } = &mut request else { panic!() };
        let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { panic!() };
        farm.more.specials.method_status.change = Some(FarmMethodChangeLinkInput {
            change: if mode == 0 { sibling } else { first },
            approval: "Actual retained automatic-change statement".into(),
        });
        if mode == 1 {
            farm.more.method_changes.clear();
        }
        if mode == 3 {
            farm.more.specials.method_status.adopted = 2025;
        }
        rejected_with(
            current_income_refusal(
                case,
                &mut session,
                request,
                "FARM_METHOD_SOURCE_ID",
                "[documents/documents/0/document/more/specials/method_status/change/change]",
            ),
            "FARM_METHOD_SOURCE_ID",
        );
        assert_eq!(session.draft, before);
    }
    // Corrupt restored identities are invalid rather than renumbered.
    let mut corrupt = session.draft.clone();
    crate::method_change::records_mut(&mut corrupt.income[0].income).unwrap()[0].id = sibling;
    assert!(!corrupt.valid_stored());
    let mut corrupt = session.draft.clone();
    crate::method_change::records_mut(&mut corrupt.income[0].income).unwrap()[0].id = 0;
    assert!(!corrupt.valid_stored());
}

#[test]
fn farm_special_named_required_sources_keep_known_receipts_and_identify_unknown_tax() {
    // Derived scope probes: no published IRS dollar result is claimed.
    for (source, key) in [
        (1, "ISSUE_FARM_FOREIGN_SOURCE"),
        (2, "ISSUE_FARM_ENTITY_SOURCE"),
        (3, "ISSUE_FARM_METHOD_TRANSITION_SOURCE"),
        (4, "ISSUE_FARM126_ECONOMIC_VALUE_SOURCE"),
        (6, "ISSUE_FARM_INTEREST_ALLOCATION_SOURCE"),
        (8, "ISSUE_FARM_HEDGE_ALLOCATION_SOURCE"),
        (10, "ISSUE_FARM_ADDITIONAL_SOURCE"),
    ] {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let mut farm = cash_farm(3_000_000, 0);
        farm.more.specials.limitations = vec![source];
        if source == 10 {
            farm.more.specials.other_situation = Some(
                "Current canceled farm debt with unresolved actual exclusion and basis ordering"
                    .into(),
            );
        }
        save_farms(&mut session, vec![farm.clone()]);
        let before = session.draft.clone();
        let result = estimate(&session.draft);
        // The normal unsupported-result owner suppresses all breakdown rows.
        // Check actual retained source income on its working copy instead.
        let mut notices = vec![];
        let working = crate::unknowns::supported(&session.draft, &mut notices)
            .unwrap_or_else(|| session.draft.clone());
        assert_eq!(stored_farm(&working).1.gross(), 3_000_000, "{source}");
        assert_eq!(working.income.len(), session.draft.income.len());
        assert_eq!(result.tax, None, "{source}");
        assert!(
            result.issues.iter().any(|issue| issue.display.key == key),
            "{source}: {:?}",
            result.issues
        );
        assert_eq!(reopened_farming(&mut session).1, vec![IncomeDocument::Farm(farm)]);
        assert_eq!(session.draft, before);
    }
}
#[test]
fn farm_special_other_source_description_matches_the_actual_selected_source() {
    for (selected, description) in
        [(true, None), (false, Some("Unselected source".into())), (true, Some(" ".into()))]
    {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let before = session.draft.clone();
        let mut farm = cash_farm(3_000_000, 0);
        if selected {
            farm.more.specials.limitations = vec![10];
        }
        farm.more.specials.other_situation = description;
        let request = farm_save(&before, vec![document(0, IncomeDocument::Farm(farm))]);
        rejected_with(send(&mut session, request), "FARM_OTHER_SOURCE_REQUIRED");
        assert_eq!(session.draft, before);
    }
}
#[test]
fn farm_special_unestablished_assessment_and_averaging_omit_only_their_benefits() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(3_000_000, 0);
    farm.more.specials.limitations = vec![5, 9];
    farm.more.conservation = Conservation::Deduct {
        costs: vec![ConservationCostInput {
            property: 0,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Revised district land work".into(),
            amount: 400_000,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    };
    let mut documents = vec![
        document(0, IncomeDocument::Farm(farm)),
        document(
            0,
            IncomeDocument::FarmAveraging(crate::tests::cross::interview::averaging_election()),
        ),
    ];
    crate::tests::farm::interview::accept_farm_sale_parcels(&mut session, &mut documents);
    let before = session.draft.clone();
    let request = farm_save(&before, documents);
    accepted(&send(&mut session, request));
    let before = session.draft.clone();
    let result = supported(&session.draft);
    assert_eq!(form(&result, "2"), 3_000_000);
    assert_eq!(form(&result, "12"), 0);
    assert!(noted(&result, "WARN_FARM_REVISED_ASSESSMENT_OMITTED"));
    assert!(noted(&result, "WARN_FARM_AVERAGING_HISTORY_OMITTED"));
    assert!(!result.lines.iter().any(|line| line.id == "farm_averaging_elected_income"));
    assert_eq!(session.draft, before);
}
#[test]
fn farm_special_property_history_omits_deduction_but_never_erases_actual_disposition() {
    for sold in [false, true] {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let mut farm = cash_farm(3_000_000, 0);
        farm.more.specials.limitations = vec![7];
        let mut asset = crate::tests::cross::interview::asset(100_000, 20260501, AssetClass::Five);
        if sold {
            asset.disposal = AssetDisposal::Disposed {
                date: 20261001,
                how: DisposalKind::Sold {
                    proceeds: 150_000,
                    selling_expenses: 0,
                    payment: SalePayment::InFull,
                    livestock: crate::Livestock::No,
                    buyer: crate::Buyer::Unrelated,
                },
            };
        }
        farm.assets = vec![asset];
        save_farms(&mut session, vec![farm]);
        let before = session.draft.clone();
        let result = estimate(&session.draft);
        let mut notices = vec![];
        let working = crate::unknowns::supported(&session.draft, &mut notices)
            .unwrap_or_else(|| session.draft.clone());
        assert_eq!(stored_farm(&working).1.gross(), 3_000_000);
        if sold {
            assert!(noted(&result, "NOTE_FARM_PROPERTY_DISPOSITION_SOURCE"));
            assert_eq!(stored_farm(&working).0.assets.len(), 1);
            assert_eq!(result.tax, None);
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_FARM_PROPERTY_DISPOSITION_HISTORY")
            );
        } else {
            assert!(noted(&result, "WARN_FARM_PROPERTY_HISTORY_OMITTED"));
            assert_eq!(form(&result, "2"), 3_000_000);
            assert_eq!(form(&result, "14"), 0);
            assert!(result.tax.is_some());
        }
        assert_eq!(session.draft, before);
    }
}
#[test]
fn farm_special_positive_method_with_distinct_amt_is_calculated() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(3_000_000, 0);
    let mut record = method_record(2024);
    record.amt = MethodChangeAmt::Different { amount: 800_000 };
    farm.more.method_changes = vec![record];
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    assert!(result.tax.is_some());
    assert_eq!(session.draft, before);
}

// Pub225 (2025), chapter5, printed p32, gross $32,000/current conservation
// cost $10,000 => $8,000 deducted/$2,000 carried. Uniform shift to 2026;
// the plan/election records below are explicit neutral completion facts.
// Mixed additional work is a derived boundary probe, not a published IRS example.
#[test]
pub(in crate::tests) fn farm_special_conservation_plan_work_and_election_are_public_and_item_specific()
 {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(3_200_000, 0);
    let mut wetlands = conservation_facts();
    wetlands.work = FarmConservationWork::WetlandDrainage;
    farm.more.conservation = Conservation::Deduct {
        costs: vec![
            ConservationCostInput {
                property: 0,
                paid_or_incurred: None,
                label: "NRCS terrace work".into(),
                amount: 1_000_000,
                facts: conservation_facts(),
            },
            ConservationCostInput {
                property: 0,
                paid_or_incurred: None,
                label: "Wetland drainage; not section175".into(),
                amount: 500_000,
                facts: wetlands,
            },
        ],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    };
    save_farms(&mut session, vec![farm.clone()]);
    let report = supported(&session.draft);
    let _ = FederalEstimate::from(report.clone());
    assert_eq!(form(&report, "2"), 3_200_000);
    assert_eq!(form(&report, "12"), 800_000);
    assert_eq!(row(&report, "_conservation_carryforward"), 200_000);
    assert!(noted(&report, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    assert_eq!(
        reopened_farming(&mut session).1,
        vec![IncomeDocument::Farm({
            let mut expected = farm;
            let accepted = stored_farm(&session.draft).1;
            expected.more.land_basis = accepted.land_basis.clone();
            if let Conservation::Deduct { costs, .. } = &mut expected.more.conservation {
                for cost in costs {
                    cost.property = accepted.land_basis[0].id;
                }
            }
            expected
        })]
    );
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
}
#[test]
fn farm_special_conservation_no_plan_and_unestablished_method_omit_only_the_deduction() {
    for unknown_method in [false, true] {
        let mut facts = conservation_facts();
        if !unknown_method {
            facts.plan = FarmConservationPlan::None;
            facts.plan_reference = None;
        }
        let mut farm = cash_farm(3_200_000, 0);
        farm.more.conservation = Conservation::Deduct {
            costs: vec![ConservationCostInput {
                property: 0,
                paid_or_incurred: None,
                label: "Actual land work".into(),
                amount: 1_000_000,
                facts,
            }],
            assessments: vec![],
            carry: vec![],
            method: if unknown_method {
                FarmConservationMethod::Unestablished
            } else {
                conservation_method()
            },
        };
        let mut session = Interview::new(TEST_USER, farm_only_return());
        save_farms(&mut session, vec![farm.clone()]);
        let report = supported(&session.draft);
        assert_eq!(form(&report, "2"), 3_200_000);
        assert_eq!(form(&report, "12"), 0);
        assert!(noted(&report, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
        assert_eq!(
            reopened_farming(&mut session).1,
            vec![IncomeDocument::Farm({
                let mut expected = farm;
                let accepted = stored_farm(&session.draft).1;
                expected.more.land_basis = accepted.land_basis.clone();
                if let Conservation::Deduct { costs, .. } = &mut expected.more.conservation {
                    for cost in costs {
                        cost.property = accepted.land_basis[0].id;
                    }
                }
                expected
            })]
        );
    }
}
#[test]
fn farm_special_conservation_site_specific_recovery_does_not_require_prior_farm_use() {
    let mut facts = conservation_facts();
    facts.plan = FarmConservationPlan::EndangeredRecovery;
    facts.work = FarmConservationWork::EndangeredRecovery;
    facts.land_farmed = false;
    let mut farm = cash_farm(3_200_000, 0);
    farm.more.conservation = Conservation::Deduct {
        costs: vec![ConservationCostInput {
            property: 0,
            paid_or_incurred: None,
            label: "Approved site-specific recovery work".into(),
            amount: 1_000_000,
            facts,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    };
    let mut session = Interview::new(TEST_USER, farm_only_return());
    save_farms(&mut session, vec![farm]);
    let report = supported(&session.draft);
    assert_eq!(form(&report, "12"), 800_000);
    assert!(!noted(&report, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
}
#[test]
fn farm_special_conservation_plan_reference_has_matching_page_and_server_guard() {
    let mut facts = conservation_facts();
    facts.plan_reference = None;
    let mut farm = cash_farm(3_200_000, 0);
    farm.more.conservation = Conservation::Deduct {
        costs: vec![ConservationCostInput {
            property: 0,
            paid_or_incurred: None,
            label: "Missing approved-plan identifier".into(),
            amount: 1_000_000,
            facts,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    };
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let before = session.draft.clone();
    let request = farm_save(&before, vec![document(0, IncomeDocument::Farm(farm))]);
    rejected_with(send(&mut session, request), "FARM_CONSERVATION_PLAN_REQUIRED");
    assert_eq!(session.draft, before);
}

// Derived separate-character probe, not an IRS worked-dollar example:
// current cost10,000 + retained regular5,000/AMT2,000, below the30,000cap,
// gives regular15,000/AMT12,000 and a3,000 AMT business-income increase.
#[test]
fn farm_special_conservation_retained_amt_carry_feeds_the_existing_amt_collector_once() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { costs, carry, .. }) = &mut data.conservation else {
        unreachable!()
    };
    costs[0].amount = 1_000_000;
    carry[0].amt = 200_000;
    let result = supported(&d);
    assert_eq!(row(&result, "_conservation_deduction"), 1_500_000);
    assert_eq!(row(&result, "_conservation_amt_deduction"), 1_200_000);
    assert_eq!(value(&result, "amt_farm_conservation_adjustment"), 3000);
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 0);
    assert_eq!(value(&result, "self_employment_tax"), 12010);
    assert_eq!(supported(&restart(d.clone()).draft), result);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { carry, .. }) = &mut data.conservation else {
        unreachable!()
    };
    carry[0].amt = 500_000;
    let same = supported(&d);
    assert_eq!(value(&same, "amt_farm_conservation_adjustment"), 0);
    assert_eq!(value(&same, "self_employment_tax"), 12010);
    assert_eq!(value(&same, "regular_income_tax"), value(&result, "regular_income_tax"));
}
#[test]
fn farm_special_conservation_parcel_and_source_year_guards_preserve_failed_save_state() {
    let mut farm = cash_farm(3_200_000, 0);
    farm.more.conservation = Conservation::Deduct {
        costs: vec![ConservationCostInput {
            property: 0,
            paid_or_incurred: Some(20260401),
            label: "NRCS terrace".into(),
            amount: 1_000_000,
            facts: conservation_facts(),
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    };
    let mut session = Interview::new(TEST_USER, farm_only_return());
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let (_, documents) = reopened_farming(&mut session);
    for case in 0..5 {
        let mut changed = documents.clone();
        let IncomeDocument::Farm(farm) = &mut changed[0] else { unreachable!() };
        let parcel = farm.more.land_basis[0].id;
        let Conservation::Deduct { costs, carry, assessments, .. } = &mut farm.more.conservation
        else {
            unreachable!()
        };
        let key = match case {
            0 => {
                costs[0].property = 99_999_999;
                "FARM_CONSERVATION_PROPERTY_SOURCE"
            }
            1 => {
                *carry = vec![
                    FarmConservationCarryInput {
                        property: parcel,
                        year: 2025,
                        regular: 100,
                        amt: 200
                    };
                    2
                ];
                "FARM_CONSERVATION_CARRY_DUPLICATE"
            }
            2 => {
                *carry = vec![FarmConservationCarryInput {
                    property: parcel,
                    year: 2026,
                    regular: 100,
                    amt: 200,
                }];
                "FARM_CONSERVATION_CARRY_YEAR"
            }
            3 => {
                *carry = vec![FarmConservationCarryInput {
                    property: parcel,
                    year: 2024,
                    regular: 100,
                    amt: 200,
                }];
                "FARM_CONSERVATION_METHOD_HISTORY"
            }
            _ => {
                *assessments = vec![ConservationAssessmentInput {
                    property: parcel,
                    facts: conservation_facts(),
                    label: "Earlier assessment payment".into(),
                    total: 700_000,
                    share: 90_000,
                    payments: vec![ConservationAssessmentPayment {
                        year: 2024,
                        amount: 90_000,
                        current_date: None,
                    }],
                }];
                "FARM_CONSERVATION_METHOD_HISTORY"
            }
        };
        let actual_id = before.income[0].id;
        let request =
            farm_save(&before, changed.into_iter().map(|item| document(actual_id, item)).collect());
        rejected_with(send(&mut session, request), key);
        assert_eq!(session.draft, before);
    }
}

// Derived public-page fixtures from Pub225ch5/Reg1.175-5 allocation rules.
// Dollar combinations are independent synthetic arithmetic, not printed IRS
// examples. Opening basis includes prior capital costs and excludes live175.
fn farm_conservation_sale_session(conservation: Conservation) -> Interview {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_000_000, 0);
    // This synthetic prior assessment fixture actually deducted70 in2025;
    // retain that allowed history separately from its opening basis.
    let prior = if matches!(&conservation, Conservation::Deduct { assessments, .. } if !assessments.is_empty())
    {
        7000
    } else {
        0
    };
    farm.more.land_basis = vec![FarmLandBasis {
        id: 0,
        label: "Conservation parcel".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20220101,
        regular: 10_000_000,
        amt: 9_000_000,
        prior_conservation: prior,
        prior_amt_conservation: prior,
        prior_cost_sharing: vec![],
    }];
    farm.more.conservation = conservation;
    farm.more.land_sales = vec![LandSale {
        id: 0,
        basis_property: 0,
        label: "Whole conservation parcel".into(),
        sold: 20260910,
        proceeds: 14_000_000,
        selling: 0,
        installments: LandInstallments::No,
        situation: FarmLandSituation::Direct,
        payment: crate::SalePayment::InFull,
    }];
    save_farms(&mut session, vec![farm]);
    let mut pages = reopened_farming(&mut session).1;
    save_conservation_sale_page(&mut session, pages.remove(0));
    session
}
fn save_conservation_sale_page(session: &mut Interview, page: IncomeDocument) {
    let before = session.draft.clone();
    let documents = vec![
        document(before.income[0].id, page),
        document(
            0,
            IncomeDocument::BusinessPropertyLosses(crate::BusinessPropertyLosses {
                regular: vec![],
                amt: vec![],
            }),
        ),
    ];
    accepted(&send(session, farm_save(&before, documents)));
}
fn conservation_sale_cost(amount: i64) -> ConservationCostInput {
    ConservationCostInput {
        property: 0,
        label: "Actual pre-sale terracing".into(),
        amount,
        facts: conservation_facts(),
        paid_or_incurred: Some(20260601),
    }
}
#[test]
fn farm_conservation_current_sale_reuses_separate_pooled_allowances_without_carry_basis() {
    let session = farm_conservation_sale_session(Conservation::Deduct {
        method: conservation_method(),
        costs: vec![conservation_sale_cost(1_000_000)],
        assessments: vec![],
        carry: vec![FarmConservationCarryInput {
            property: 0,
            year: 2025,
            regular: 500_000,
            amt: 200_000,
        }],
    });
    let result = supported(&session.draft);
    assert_eq!(form(&result, "12"), 1_500_000);
    let (_, farm) = stored_farm(&session.draft);
    let sale =
        farm.land_sales[0].resolve(&session.draft, session.draft.income[0].id, farm).unwrap();
    // Current allowed15000/12000 enters1252, not land basis. Regular
    // gain40000 and AMTgain50000 retain their independent opening basis.
    assert_eq!((sale.basis, sale.conservation), (10_000_000, 1_500_000));
    assert_eq!(
        (sale.calculate().gain, sale.calculate().recapture, sale.calculate().section1231),
        (40000, 15000, 25000)
    );
    assert_eq!((sale.calculate_for(true).gain, sale.calculate_for(true).recapture), (50000, 12000));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
}
#[test]
fn farm_conservation_sale_capitalizes_only_remaining_assessment_spread() {
    let mut session = farm_conservation_sale_session(Conservation::Deduct {
        method: conservation_method(),
        costs: vec![],
        carry: vec![],
        assessments: vec![ConservationAssessmentInput {
            property: 0,
            facts: conservation_facts(),
            label: "District dam".into(),
            total: 700_000,
            share: 90_000,
            payments: vec![ConservationAssessmentPayment {
                year: 2025,
                amount: 90_000,
                current_date: None,
            }],
        }],
    });
    let result = supported(&session.draft);
    let (_, farm) = stored_farm(&session.draft);
    let sale =
        farm.land_sales[0].resolve(&session.draft, session.draft.income[0].id, farm).unwrap();
    // 2025eligible700:70then,630spread. 2026deduct70,560remains
    // and becomes basis at disposal. Prior capital200 was already in the
    // explicit entering-year basis; it is not added again.
    assert_eq!(form(&result, "12"), 7000);
    assert_eq!((sale.basis, sale.conservation), (10_056_000, 14000));
    assert_eq!(sale.calculate().gain, 39440);
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    let FarmMethod::Cash { raised, .. } = &mut farm.accounting else { unreachable!() };
    *raised = 0;
    save_conservation_sale_page(&mut session, pages.remove(0));
    let result = supported(&session.draft);
    let (_, farm) = stored_farm(&session.draft);
    let sale =
        farm.land_sales[0].resolve(&session.draft, session.draft.income[0].id, farm).unwrap();
    // With no production gross, the current70 is25%-limited carry, not
    // additional sale basis. Only the unexpired560 spread capitalizes.
    assert_eq!(form(&result, "12"), 0);
    assert_eq!((sale.basis, sale.conservation), (10_056_000, 7000));
}
#[test]
fn farm_conservation_capital_history_and_missing_sale_timing_never_guess_basis() {
    for method in [
        FarmConservationCapitalMethod::NeverElected,
        FarmConservationCapitalMethod::ApprovedChange {
            year: 2026,
            approval: "Actual IRS consent covers this parcel project.".into(),
        },
    ] {
        let session = farm_conservation_sale_session(Conservation::Capitalize {
            method,
            costs: vec![conservation_sale_cost(4_000_000)],
        });
        let result = supported(&session.draft);
        let (_, farm) = stored_farm(&session.draft);
        let sale =
            farm.land_sales[0].resolve(&session.draft, session.draft.income[0].id, farm).unwrap();
        assert_eq!((sale.basis, sale.conservation, sale.calculate().gain), (14_000_000, 0, 0));
        assert_eq!(form(&result, "12"), 0);
    }
    let session = farm_conservation_sale_session(Conservation::Capitalize {
        method: FarmConservationCapitalMethod::Unestablished,
        costs: vec![conservation_sale_cost(4_000_000)],
    });
    let result = estimate(&session.draft);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED")
    );
    assert!(noted(&result, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    for date in [None, Some(20261001)] {
        let mut cost = conservation_sale_cost(4_000_000);
        cost.paid_or_incurred = date;
        let session = farm_conservation_sale_session(Conservation::Capitalize {
            method: FarmConservationCapitalMethod::NeverElected,
            costs: vec![cost],
        });
        assert_eq!(estimate(&session.draft).tax, None);
    }
}
#[test]
fn farm_conservation_sale_cannot_reuse_one_whole_parcel_twice() {
    let mut session = farm_conservation_sale_session(Conservation::Capitalize {
        method: FarmConservationCapitalMethod::NeverElected,
        costs: vec![conservation_sale_cost(4_000_000)],
    });
    let before = session.draft.clone();
    let mut pages = reopened_farming(&mut session).1;
    let IncomeDocument::Farm(farm) = &mut pages[0] else { unreachable!() };
    let mut repeated = farm.more.land_sales[0].clone();
    repeated.id = 0;
    farm.more.land_sales.push(repeated);
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, pages.remove(0))]),
        ),
        "FARM_LAND_DUPLICATE_PARCEL",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn farm_conservation_ineligible_land_costs_capitalize_without_an_elective_deduction() {
    let mut cost = conservation_sale_cost(1_000_000);
    cost.facts.plan = FarmConservationPlan::None;
    cost.facts.plan_reference = None;
    let mut district = conservation_facts();
    district.plan = FarmConservationPlan::None;
    district.plan_reference = None;
    let session = farm_conservation_sale_session(Conservation::Deduct {
        method: conservation_method(),
        costs: vec![cost],
        carry: vec![],
        assessments: vec![ConservationAssessmentInput {
            property: 0,
            facts: district,
            label: "Nonqualifying district project".into(),
            total: 700_000,
            share: 90_000,
            payments: vec![ConservationAssessmentPayment {
                year: 2026,
                amount: 90_000,
                current_date: Some(20260601),
            }],
        }],
    });
    let result = supported(&session.draft);
    let (_, farm) = stored_farm(&session.draft);
    let sale =
        farm.land_sales[0].resolve(&session.draft, session.draft.income[0].id, farm).unwrap();
    // Actual land-capital costs10000+900are not deductible175costs.
    // The separate explicit70prior deduction is retained, not doubled.
    assert_eq!(form(&result, "12"), 0);
    assert_eq!((sale.basis, sale.conservation), (11_090_000, 7000));
    assert!(noted(&result, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
}
#[test]
fn farm_conservation_taxpayer_method_scope_is_checked_without_merging_spouses() {
    let mut original = empty_return(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut capital = cash_farm(1_000_000, 0);
    capital.name = "Capital method farm".into();
    capital.more.conservation = Conservation::Capitalize {
        method: FarmConservationCapitalMethod::NeverElected,
        costs: vec![conservation_sale_cost(100_000)],
    };
    let mut deduction = cash_farm(1_000_000, 0);
    deduction.name = "Deduction method farm".into();
    deduction.more.conservation = Conservation::Deduct {
        method: conservation_method(),
        costs: vec![conservation_sale_cost(100_000)],
        assessments: vec![],
        carry: vec![],
    };
    let mut documents = vec![
        document(0, IncomeDocument::Farm(capital)),
        document(0, IncomeDocument::Farm(deduction)),
    ];
    crate::tests::farm::interview::accept_farm_sale_parcels(&mut session, &mut documents);
    let before = session.draft.clone();
    rejected_with(
        send(&mut session, farm_save(&before, documents.clone())),
        "FARM_CONSERVATION_METHOD_CONFLICT",
    );
    assert_eq!(session.draft, before);
    documents[1].owner = Owner::Spouse;
    accepted(&send(&mut session, farm_save(&before, documents)));
    let result = supported(&session.draft);
    assert_eq!(form(&result, "12"), 100_000);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    // Even without another current deduction election, actual prior allowed
    // deductions contradict this taxpayer's never-elected history.
    let before = session.draft.clone();
    let P::SaveIncomes { mut documents, .. } =
        crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
    else {
        unreachable!()
    };
    let IncomeDocument::Farm(capital) = &mut documents.documents[0].document else {
        unreachable!()
    };
    capital.more.land_basis[0].prior_conservation = 7000;
    rejected_with(
        send(&mut session, farm_save(&before, documents.documents)),
        "FARM_CONSERVATION_METHOD_CONFLICT",
    );
    assert_eq!(session.draft, before);
    // A restored contradictory source cannot claim a known method or tax.
    original = session.draft.clone();
    original.income[1].owner = Owner::Taxpayer;
    let result = estimate(&original);
    assert_eq!(result.tax, None);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "FARM_CONSERVATION_METHOD_CONFLICT")
    );
}

#[test]
fn farm_supply_actual_terms_keep_deposits_separate_and_refunds_out_of_cost() {
    // Derived actual-source probe, not a printed IRS example. $10,000 other
    // expenses permit $5,000 prepaid deduction; the binding $6,000 payment
    // less its actual $1,000 refund fits. A separate $2,000 deposit remains.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(10_000_000, 1_000_000);
    let binding = FarmSupplyPurchase { refunded: 100_000, ..farm_supply_purchase_source() };
    let deposit = FarmSupplyPurchase {
        kind: FarmSupplyPaymentKind::Deposit,
        quantity_price_terms: "".into(),
        business_purpose: "".into(),
        ..farm_supply_purchase_source()
    };
    farm.more.supplies = PrepaidSupplies::Supplies {
        advances: vec![
            SupplyPurchase { amount: 600_000, purchase: binding },
            SupplyPurchase { amount: 200_000, purchase: deposit },
        ],
        carried: vec![],
        exception: SupplyException::NotNeeded,
        poultry: PoultryRecords { current: vec![], resale: vec![], production: vec![] },
    };
    save_farms(&mut session, vec![farm.clone()]);
    let report = supported(&session.draft);
    assert_eq!(row(&report, "_supplies_eligible"), 500_000);
    assert_eq!(row(&report, "_supplies_deduction"), 500_000);
    assert_eq!(row(&report, "_supplies_remaining"), 200_000);
    assert!(noted(&report, "WARN_FARM_SUPPLIES_LEFT_OUT"));
    assert_eq!(reopened_farming(&mut session).1, vec![IncomeDocument::Farm(farm)]);
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
}

#[test]
fn farm_supply_invalid_refund_and_unknown_terms_preserve_accepted_state() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(10_000_000, 1_000_000);
    farm.more.supplies = PrepaidSupplies::Supplies {
        advances: vec![SupplyPurchase { amount: 600_000, purchase: farm_supply_purchase_source() }],
        carried: vec![],
        exception: SupplyException::NotNeeded,
        poultry: PoultryRecords { current: vec![], resale: vec![], production: vec![] },
    };
    save_farms(&mut session, vec![farm]);
    let P::SaveIncomes { mut documents, .. } =
        crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
    else {
        unreachable!()
    };
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { unreachable!() };
    let PrepaidSupplies::Supplies { advances, .. } = &mut farm.more.supplies else {
        unreachable!()
    };
    advances[0].purchase.refunded = 600_001;
    let before = session.draft.clone();
    let request = farm_save(&before, documents.documents.clone());
    rejected_with(send(&mut session, request), "FARM_SUPPLY_REFUND_ABOVE_PAYMENT");
    assert_eq!(session.draft, before);
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { unreachable!() };
    let PrepaidSupplies::Supplies { advances, .. } = &mut farm.more.supplies else {
        unreachable!()
    };
    advances[0].purchase.refunded = 0;
    advances[0].purchase.kind = FarmSupplyPaymentKind::UnresolvedTerms;
    let request = farm_save(&before, documents.documents.clone());
    accepted(&send(&mut session, request));
    let report = supported(&session.draft);
    assert_eq!(row(&report, "_supplies_deduction"), 0);
    assert_eq!(row(&report, "_supplies_remaining"), 600_000);
    assert_eq!(form(&report, "9"), 10_000_000);
}

#[test]
fn farm_supply_separate_retained_amt_cost_changes_each_ceiling_once() {
    // Derived public-source probe. Current10,000unused plus10,000ordinary
    // expenses. Old regular1,000/AMT600consumed costs release once and are
    // other expenses: current ceilings5,500/5,300; total6,500/5,900.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(10_000_000, 1_000_000);
    farm.more.supplies = PrepaidSupplies::Supplies {
        advances: vec![SupplyPurchase {
            amount: 1_000_000,
            purchase: farm_supply_purchase_source(),
        }],
        carried: vec![CarriedCost { balance: 100_000, amt_balance: 60_000, used: true }],
        exception: SupplyException::NotNeeded,
        poultry: PoultryRecords { current: vec![], resale: vec![], production: vec![] },
    };
    save_farms(&mut session, vec![farm.clone()]);
    let r = supported(&session.draft);
    assert_eq!(row(&r, "_supplies_deduction"), 650_000);
    assert_eq!(row(&r, "_supplies_amt_deduction"), 590_000);
    assert_eq!(row(&r, "_supplies_remaining"), 450_000);
    assert_eq!(row(&r, "_supplies_amt_remaining"), 470_000);
    assert_eq!(value(&r, "amt_farm_supplies_adjustment"), 600);
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), 0);
    assert_eq!(reopened_farming(&mut session).1, vec![IncomeDocument::Farm(farm)]);
    assert_eq!(supported(&restart(session.draft.clone()).draft), r);
}

#[test]
fn farm_supply_ceiling_uses_each_conservation_allowance_without_duplicate_amt() {
    // Derived public source: regular/AMTconservation15,000/12,000 plus
    //20,000ordinaryexpenses permit17,500/16,000current prepaid cost.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(12_000_000, 2_000_000);
    farm.more.land_basis = vec![FarmLandBasis {
        id: 0,
        label: "Retained terrace parcel".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20200101,
        regular: 10_000_000,
        amt: 10_000_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }];
    save_farms(&mut session, vec![farm]);
    let P::SaveIncomes { mut documents, .. } =
        crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
    else {
        unreachable!()
    };
    let IncomeDocument::Farm(farm) = &mut documents.documents[0].document else { unreachable!() };
    let parcel = farm.more.land_basis[0].id;
    farm.more.conservation = Conservation::Deduct {
        costs: vec![ConservationCostInput {
            property: parcel,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Current terraces".into(),
            amount: 1_000_000,
        }],
        assessments: vec![],
        carry: vec![FarmConservationCarryInput {
            property: parcel,
            year: 2025,
            regular: 500_000,
            amt: 200_000,
        }],
        method: conservation_method(),
    };
    farm.more.supplies = PrepaidSupplies::Supplies {
        advances: vec![SupplyPurchase {
            amount: 5_000_000,
            purchase: farm_supply_purchase_source(),
        }],
        carried: vec![],
        exception: SupplyException::NotNeeded,
        poultry: PoultryRecords { current: vec![], resale: vec![], production: vec![] },
    };
    let before = session.draft.clone();
    accepted(&send(&mut session, farm_save(&before, documents.documents)));
    let r = supported(&session.draft);
    assert_eq!(row(&r, "_supplies_deduction"), 1_750_000);
    assert_eq!(row(&r, "_supplies_amt_deduction"), 1_600_000);
    assert_eq!(value(&r, "amt_farm_conservation_adjustment"), 3000);
    assert_eq!(value(&r, "amt_farm_supplies_adjustment"), 1500);
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), 0);
    assert_eq!(supported(&restart(session.draft.clone()).draft), r);
}

// Derived source-allocation probes under Pub225 chapter6 and ScheduleF
// instructions' preproductive-cost paragraph, not published dollar examples.
// https://www.irs.gov/publications/p225
// https://www.irs.gov/instructions/i1040sf
pub(in crate::tests) fn production_inventory() -> FarmProductionCohortInput {
    FarmProductionCohortInput {
        id: 0,
        description: "Ornamental trees held for sale".into(),
        property: FarmProducedKind::Plant,
        began: 20250101,
        productive: None,
        normal_preproductive_months: 36,
        tax_treatment: FarmCapitalization::Required,
        costs: vec![
            FarmProductionCost {
                description: "Direct cultivation".into(),
                category: BusinessExpenseCategory::Seeds,
                treatment: BusinessExpenseKind::Ordinary,
                cost: 1_000_000,
                non263a_capital: 0,
                non263a_amt_capital: 0,
                capital_allocated: 1_000_000,
                amt_capital_allocated: 1_000_000,
                kind: FarmProductionCostKind::Direct,
            },
            FarmProductionCost {
                description: "Documented indirect allocation".into(),
                category: BusinessExpenseCategory::Supplies,
                treatment: BusinessExpenseKind::Ordinary,
                cost: 200_000,
                non263a_capital: 10_000,
                non263a_amt_capital: 20_000,
                capital_allocated: 50_000,
                amt_capital_allocated: 80_000,
                kind: FarmProductionCostKind::IndirectAllocated,
            },
        ],
        prior_basis: 500_000,
        prior_amt_basis: 600_000,
        destination: FarmBasisDestinationInput::Inventory { released: 0, amt_released: 0 },
    }
}
#[test]
fn farm_production_inventory_retains_basis_and_reports_unreduced_costs_then_263a() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(5_000_000, 0);
    farm.more.specials.production = vec![production_inventory()];
    save_farms(&mut session, vec![farm]);
    let r = supported(&session.draft);
    assert_eq!(form(&r, "26"), 1_000_000);
    assert_eq!(form(&r, "28"), 190_000);
    assert_eq!(form(&r, "32f"), -1_050_000);
    assert_eq!(form(&r, "33"), 140_000);
    assert_eq!(form(&r, "34"), 4_860_000);
    assert_eq!(row(&r, "_retained_basis"), 1_560_000);
    assert_eq!(row(&r, "_amt_retained_basis"), 1_700_000);
    assert_eq!(value(&r, "amt_farm_production_adjustment"), 400);
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), 0);
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut reopened) = docs[0].clone() else { unreachable!() };
    let id = reopened.more.specials.production[0].id;
    assert!(id > 0);
    reopened.more.specials.production[0].description = "Renamed same trees".into();
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(reopened))]),
    ));
    assert_eq!(stored_farm(&session.draft).1.specials.production[0].id, id);
    assert_eq!(supported(&restart(session.draft.clone()).draft), supported(&session.draft));
}
#[test]
fn farm_production_small_business_exemption_preserves_other_capital_rules() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(5_000_000, 0);
    let mut cohort = production_inventory();
    cohort.tax_treatment = FarmCapitalization::SmallBusiness {
        facts: FarmSmallBusiness {
            first_tax_year: 2024,
            receipts: vec![
                FarmGrossYear { year: 2024, months: 6, gross: 1_600_000_000 },
                FarmGrossYear { year: 2025, months: 12, gross: 3_200_000_000 },
            ],
            tax_shelter: false,
        },
    };
    farm.more.specials.production = vec![cohort];
    save_farms(&mut session, vec![farm]);
    let r = supported(&session.draft);
    assert_eq!(form(&r, "26"), 1_000_000);
    assert_eq!(form(&r, "28"), 190_000);
    assert_eq!(form(&r, "32f"), 0);
    assert_eq!(form(&r, "33"), 1_190_000);
    assert_eq!(row(&r, "_retained_basis"), 510_000);
    assert_eq!(row(&r, "_amt_retained_basis"), 620_000);
    assert_eq!(value(&r, "amt_farm_production_adjustment"), 100);
    assert!(!noted(&r, "NOTE_FARM263A_EXEMPTION_NOT_MET"));
    // One cent in the six-month source year annualizes to two cents;
    // the two-year average is now one cent above the exact $32m limit.
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let FarmCapitalization::SmallBusiness { facts } =
        &mut farm.more.specials.production[0].tax_treatment
    else {
        unreachable!()
    };
    facts.receipts[0].gross += 1;
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
    ));
    let r = supported(&session.draft);
    assert_eq!(form(&r, "33"), 140_000);
    assert_eq!(row(&r, "_retained_basis"), 1_560_000);
    assert!(noted(&r, "NOTE_FARM263A_EXEMPTION_NOT_MET"));
}
#[test]
fn farm_production_missing_recovery_owner_is_explicit_and_bad_allocation_is_pure() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(5_000_000, 0);
    let mut cohort = production_inventory();
    cohort.destination =
        FarmBasisDestinationInput::Inventory { released: 100_000, amt_released: 100_000 };
    farm.more.specials.production = vec![cohort];
    save_farms(&mut session, vec![farm]);
    let r = estimate(&session.draft);
    assert!(r.issues.iter().any(|issue| issue.display.key == "ISSUE_FARM_PRODUCTION_BASIS_OWNER"));
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    farm.more.specials.production[0].costs[0].capital_allocated += 1;
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
        ),
        "FARM_PRODUCTION_ALLOCATION",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn farm_production_owned_ids_survive_reorder_and_reject_duplicate_without_allocation() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(5_000_000, 0);
    let mut second = production_inventory();
    second.description = "Separate production lot".into();
    farm.more.specials.production = vec![production_inventory(), second];
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let first = farm.more.specials.production[0].id;
    let second = farm.more.specials.production[1].id;
    assert!(first > 0 && second > first);
    farm.more.specials.production.reverse();
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
    ));
    assert_eq!(
        stored_farm(&session.draft).1.specials.production.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![second, first]
    );
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    farm.more.specials.production[0].id = first;
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
        ),
        "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
    );
    assert_eq!(session.draft, before);
    assert_eq!(restart(before.clone()).draft, before);
}

#[test]
fn farm_ccc_original_election_source_is_public_and_preserves_canonical_loan_amounts() {
    // Derived current-year source probe with Pub225's5200/4000/6500
    // commodity amounts, not a claim to reproduce its internally inconsistent
    // narrative/printed prior-year sale example. The2026original election and
    // zero other receipts are explicit neutral completion facts.
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(0, 0);
    let mut source = ccc_source_input(&CccMethod::Income);
    source.history = CccElectionHistory::IncomeElection {
        year: 2026,
        statement: "Timely original 2026 statement listing this 5200 commodity loan".into(),
    };
    farm.more.ccc = CccLoans::Loans {
        method: CccMethod::Income,
        loans: vec![CccLoan::New {
            id: 0,
            principal: 520_000,
            outcome: CccOutcome::Redeemed { paid: 400_000, sale: Some(650_000) },
        }],
        withholding: 0,
        source: source.clone(),
    };
    save_farms(&mut session, vec![farm]);
    let r = supported(&session.draft);
    assert_eq!(form(&r, "5a"), 520_000);
    assert_eq!(form(&r, "1a"), 650_000);
    assert_eq!(form(&r, "1b"), 400_000);
    assert_eq!(form(&r, "9"), 770_000);
    // The existing generic no-CCC helper omits taxable current advances;
    // compare this case's printed source components explicitly instead.
    assert_eq!(form(&r, "9"), form(&r, "1a") - form(&r, "1b") + form(&r, "5a"));
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(farm) = &docs[0] else { unreachable!() };
    let CccLoans::Loans { source: reopened, loans, .. } = &farm.more.ccc else { unreachable!() };
    assert_eq!(reopened, &source);
    assert!(matches!(loans[0], CccLoan::New { id, .. } if id > 0));
    assert_eq!(supported(&restart(session.draft.clone()).draft), r);
}
#[test]
fn farm_ccc_unknown_basis_and_changed_method_keep_records_and_failed_source_save_is_pure() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(1_000_000, 0);
    let mut source = ccc_source_input(&CccMethod::Loan);
    source.unrecovered_production_cost = 100_000;
    farm.more.ccc = CccLoans::Loans {
        method: CccMethod::Loan,
        loans: vec![CccLoan::New { id: 0, principal: 520_000, outcome: CccOutcome::Outstanding }],
        withholding: 0,
        source,
    };
    save_farms(&mut session, vec![farm]);
    let before = session.draft.clone();
    let r = estimate(&before);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.display.key == "ISSUE_CCC_COMMODITY_BASIS_SOURCE"));
    assert_eq!(stored_farm(&before).1.ccc.as_ref().unwrap().lots.len(), 1);
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let CccLoans::Loans { source, .. } = &mut farm.more.ccc else { unreachable!() };
    source.unrecovered_production_cost = 0;
    source.history = CccElectionHistory::ChangedMethod {
        year: 2026,
        authority:
            "Documented automatic-change authority; actual transition basis not yet supplied".into(),
    };
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
    ));
    assert!(
        estimate(&session.draft)
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_CCC_ELECTION_TRANSITION_SOURCE")
    );
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let CccLoans::Loans { source, .. } = &mut farm.more.ccc else { unreachable!() };
    source.history = CccElectionHistory::IncomeElection {
        year: 2027,
        statement: "Invalid future election".into(),
    };
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
        ),
        "CCC_ELECTION_HISTORY_SOURCE",
    );
    assert_eq!(session.draft, before);
}

// Derived public normal-practice probes, not IRS worked-dollar examples.
// Pub225 chapter3, Reg1.451-6(a), and IRS INFO2004-0089 establish current
// cash receipts, physical loss, >50%normal following-year practice, and
// all eligible proceeds for one business. No mixed-practice aggregation
// interpretation is presumed by these fixtures.
fn crop_source_farm(following: i64) -> Farm {
    let mut farm = cash_farm(5_000_000, 0);
    farm.insurance = CropInsurance::Payments {
        physical: vec![CropLoss { crop: "Wheat".into(), amount: 1_000_000, year: 2026 }],
        price_only: 200_000,
        from_prior_year: 300_000,
        withholding: 100_000,
        election: CropDeferralChoice::Defer {
            practice: vec![FarmCropPractice {
                crop: "Wheat".into(),
                normal_total: 1_000_000,
                normal_following: following,
            }],
        },
    };
    farm
}
#[test]
fn farm_crop_actual_normal_practice_half_boundary_keeps_prior_income_and_withholding() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    save_farms(&mut session, vec![crop_source_farm(500_000)]);
    let r = supported(&session.draft);
    assert_eq!(form(&r, "6a"), 1_200_000);
    assert_eq!(form(&r, "6b"), 1_200_000);
    assert_eq!(form(&r, "6d"), 300_000);
    assert_eq!(form(&r, "9"), 6_500_000);
    assert!(noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let CropInsurance::Payments { election: CropDeferralChoice::Defer { practice }, .. } =
        &mut farm.insurance
    else {
        unreachable!()
    };
    practice[0].normal_following += 1;
    accepted(&send(
        &mut session,
        farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
    ));
    let r = supported(&session.draft);
    assert_eq!(form(&r, "6a"), 1_200_000);
    assert_eq!(form(&r, "6b"), 200_000);
    assert_eq!(form(&r, "6d"), 300_000);
    assert_eq!(form(&r, "9"), 5_500_000);
    assert_eq!(row(&r, "_crop_insurance_withholding"), 100_000);
    assert_eq!(row(&r, "_crop_insurance_deferred_next_year"), 1_000_000);
    assert!(!noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
    assert_eq!(supported(&restart(session.draft.clone()).draft), r);
}
#[test]
fn farm_crop_mixed_practices_leave_only_deferral_out_and_bad_sources_reject_purely() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = crop_source_farm(750_000);
    let CropInsurance::Payments {
        physical, election: CropDeferralChoice::Defer { practice }, ..
    } = &mut farm.insurance
    else {
        unreachable!()
    };
    physical.push(CropLoss { crop: "Corn".into(), amount: 400_000, year: 2026 });
    practice.push(FarmCropPractice {
        crop: "Corn".into(),
        normal_total: 1_000_000,
        normal_following: 250_000,
    });
    save_farms(&mut session, vec![farm]);
    let r = supported(&session.draft);
    assert_eq!(form(&r, "6b"), 1_600_000);
    assert_eq!(form(&r, "9"), 6_900_000);
    assert!(noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
    let before = session.draft.clone();
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let CropInsurance::Payments { election: CropDeferralChoice::Defer { practice }, .. } =
        &mut farm.insurance
    else {
        unreachable!()
    };
    practice[0].normal_following = practice[0].normal_total + 1;
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
        ),
        "CROP_PRACTICE_ABOVE_TOTAL",
    );
    assert_eq!(session.draft, before);
    let (_, docs) = reopened_farming(&mut session);
    let IncomeDocument::Farm(mut farm) = docs[0].clone() else { unreachable!() };
    let CropInsurance::Payments { election: CropDeferralChoice::Defer { practice }, .. } =
        &mut farm.insurance
    else {
        unreachable!()
    };
    practice.pop();
    rejected_with(
        send(
            &mut session,
            farm_save(&before, vec![document(before.income[0].id, IncomeDocument::Farm(farm))]),
        ),
        "CROP_PRACTICE_SOURCE",
    );
    assert_eq!(session.draft, before);
}

// Derived public-lifecycle probes for the Schedule J owner boundaries, not
// published IRS dollar examples. 2025 final i1040sj lines2a–2c require actual
// attributable income/gain; the 2026 DRAFT form preserves those components.
// https://www.irs.gov/instructions/i1040sj
// https://www.irs.gov/pub/irs-dft/f1040sj--dft.pdf
#[test]
fn farm_averaging_named_history_and_source_limits_preserve_ordinary_tax_and_records() {
    let original = averaging_return(60_000);
    let mut ordinary = original.clone();
    ordinary.screening.as_mut().unwrap().farm_averaging = None;
    let regular = supported(&ordinary);
    let wage = original
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap()
        .id;
    for (history_scope, capital, sources, warning) in [
        (
            FarmAveragingHistoryScope::SeparateSpouseHistories,
            0,
            vec![],
            "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT",
        ),
        (FarmAveragingHistoryScope::PriorNol, 0, vec![], "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT"),
        (FarmAveragingHistoryScope::ArcherMsa, 0, vec![], "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT"),
        (FarmAveragingHistoryScope::MinorTax, 0, vec![], "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT"),
        (
            FarmAveragingHistoryScope::ShortYearOrForeign,
            0,
            vec![],
            "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT",
        ),
        (
            FarmAveragingHistoryScope::CompleteSupported,
            100_000,
            vec![],
            "WARN_FARM_AVERAGING_CAPITAL_SOURCE_LEFT_OUT",
        ),
        (
            FarmAveragingHistoryScope::CompleteSupported,
            0,
            vec![wage],
            "WARN_FARM_AVERAGING_ADDITIONAL_SOURCE_LEFT_OUT",
        ),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let P::SaveIncomes { mut documents, .. } =
            crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
        else {
            panic!()
        };
        let page = documents
            .documents
            .iter_mut()
            .find_map(|entry| match &mut entry.document {
                IncomeDocument::FarmAveraging(page) => Some(page),
                _ => None,
            })
            .unwrap();
        page.selection = FarmAveragingSelectionInput {
            capital_elected: capital,
            unrecaptured_elected: 0,
            sources,
            history_scope,
        };
        let selected = page.selection.clone();
        let before = session.draft.clone();
        accepted(&send(&mut session, farm_save(&before, documents.documents)));
        let report = supported(&session.draft);
        assert_eq!(report.tax, regular.tax);
        assert_eq!(value(&report, "agi"), value(&regular, "agi"));
        assert!(noted(&report, warning));
        assert_eq!(
            session.draft.screening.as_ref().unwrap().farm_averaging.as_ref().unwrap().selection,
            (&selected).into()
        );
        assert_eq!(supported(&restart(session.draft.clone()).draft), report);
        let reopened = reopened_farming(&mut session).1;
        assert!(reopened.iter().any(|document| matches!(document,IncomeDocument::FarmAveraging(page) if page.selection == selected)));
    }
}

#[test]
fn farm_averaging_component_and_catalog_guards_reject_without_consuming_ids() {
    let original = averaging_return(60_000);
    let wage = original
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap()
        .id;
    for (capital, unrecaptured, sources, key) in [
        (6_000_001, 0, vec![], "FARM_AVERAGING_SELECTION_INVALID"),
        (100_000, 100_001, vec![], "FARM_AVERAGING_SELECTION_INVALID"),
        (0, 0, vec![wage, wage], "FARM_AVERAGING_SOURCES_INVALID"),
        (0, 0, vec![99_999_999_999], "FARM_AVERAGING_SOURCES_INVALID"),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let P::SaveIncomes { mut documents, .. } =
            crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
        else {
            panic!()
        };
        let page = documents
            .documents
            .iter_mut()
            .find_map(|entry| match &mut entry.document {
                IncomeDocument::FarmAveraging(page) => Some(page),
                _ => None,
            })
            .unwrap();
        page.selection.capital_elected = capital;
        page.selection.unrecaptured_elected = unrecaptured;
        page.selection.sources = sources;
        let before = session.draft.clone();
        rejected_with(send(&mut session, farm_save(&before, documents.documents)), key);
        assert_eq!(session.draft, before);
    }
}

#[test]
fn farm_averaging_service_and_resale_receipts_need_net_production_attribution() {
    for service in [true, false] {
        let mut session = Interview::new(TEST_USER, averaging_return(60_000));
        let P::SaveIncomes { mut documents, .. } =
            crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming)
        else {
            panic!()
        };
        let farm = documents
            .documents
            .iter_mut()
            .find_map(|entry| match &mut entry.document {
                IncomeDocument::Farm(farm) => Some(farm),
                _ => None,
            })
            .unwrap();
        let FarmMethod::Cash { custom_hire, resale, resale_cost, .. } = &mut farm.accounting else {
            panic!()
        };
        if service {
            *custom_hire = 1_000_000;
        } else {
            *resale = 1_000_000;
            *resale_cost = 600_000;
        }
        let before = session.draft.clone();
        accepted(&send(&mut session, farm_save(&before, documents.documents)));
        let report = supported(&session.draft);
        assert!(noted(&report, "WARN_FARM_AVERAGING_PRODUCTION_SOURCE_LEFT_OUT"));
        let mut ordinary = session.draft.clone();
        ordinary.screening.as_mut().unwrap().farm_averaging = None;
        assert_eq!(report.tax, supported(&ordinary).tax);
        assert_eq!(supported(&restart(session.draft.clone()).draft), report);
    }
}

// Derived whole-calendar-month arithmetic under section464(b)/(d) and
// Pub 225 (2025), chapter 4, Prepaid Farm Supplies. Not a printed IRS example.
// Actual first-day start avoids inventing a partial-month convention.
// https://www.govinfo.gov/content/pkg/USCODE-2024-title26/html/USCODE-2024-title26-subtitleA-chap1-subchapE-partII-subpartC-sec464.htm
// https://www.irs.gov/publications/p225
fn current_poultry_farm(other: i64) -> Farm {
    let mut farm = cash_farm(5_000_000, other);
    farm.more.supplies = PrepaidSupplies::Supplies {
        advances: vec![],
        carried: vec![],
        exception: SupplyException::NotNeeded,
        poultry: PoultryRecords {
            resale: vec![],
            production: vec![],
            current: vec![FarmProductionPoultryCurrent {
                label: "July laying flock".into(),
                cost: 1_200_000,
                paid: 20260701,
                started: 20260701,
                useful_months: 18,
                adopted: FarmPoultryMethod::Paid,
            }],
        },
    };
    farm
}
#[test]
fn farm_current_production_poultry_limits_only_following_year_ratable_cost() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    save_farms(&mut session, vec![current_poultry_farm(100_000)]);
    let report = supported(&session.draft);
    // Twelve-month cap: July–December is 6/12 of 12000, or 6000 current.
    // Other 1000 + current 6000 supports 3500 of 6000 prepaid; total 9500.
    assert_eq!(row(&report, "_supplies_production_poultry_current_portion"), 600_000);
    assert_eq!(row(&report, "_supplies_production_poultry_prepaid_portion"), 600_000);
    assert_eq!(row(&report, "_supplies_eligible"), 600_000);
    assert_eq!(row(&report, "_supplies_deduction"), 950_000);
    assert_eq!(row(&report, "_supplies_remaining"), 250_000);
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
    let mut farm = reopened_farming(&mut session)
        .1
        .into_iter()
        .find_map(|document| match document {
            IncomeDocument::Farm(farm) => Some(farm),
            _ => None,
        })
        .unwrap();
    let PrepaidSupplies::Supplies { poultry, .. } = &mut farm.more.supplies else { panic!() };
    assert_eq!(poultry.current[0].useful_months, 18);
    poultry.current[0].useful_months = 3;
    save_farms(&mut session, vec![farm]);
    let short = supported(&session.draft);
    assert_eq!(row(&short, "_supplies_production_poultry_current_portion"), 1_200_000);
    assert_eq!(row(&short, "_supplies_eligible"), 0);
    assert_eq!(row(&short, "_supplies_deduction"), 1_200_000);
    assert_eq!(row(&short, "_supplies_remaining"), 0);
}
#[test]
fn farm_current_production_poultry_uses_one_shared_prepaid_pool() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    save_farms(&mut session, vec![current_poultry_farm(100_000), current_poultry_farm(900_000)]);
    let report = supported(&session.draft);
    // Total other 10000 + fixed 12000 gives an 11000 cap against 12000 eligible.
    // Equal 6000 candidate costs receive 5500 each; each deduction is 11500.
    let rows: Vec<_> = report
        .lines
        .iter()
        .filter(|line| line.id.ends_with("_supplies_deduction"))
        .map(|line| line.amount)
        .collect();
    assert_eq!(rows, vec![1_150_000, 1_150_000]);
    assert_eq!(
        report
            .lines
            .iter()
            .filter(|line| line.id.ends_with("_supplies_remaining"))
            .map(|line| line.amount)
            .sum::<i64>(),
        100_000
    );
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
}
#[test]
fn farm_current_production_poultry_unknown_timing_retains_source_without_guessing() {
    for (started, method) in [
        (20260702, FarmPoultryMethod::Paid),
        (20250701, FarmPoultryMethod::Paid),
        (20260701, FarmPoultryMethod::Sold),
    ] {
        let mut farm = current_poultry_farm(100_000);
        let PrepaidSupplies::Supplies { poultry, .. } = &mut farm.more.supplies else { panic!() };
        poultry.current[0].started = started;
        poultry.current[0].adopted = method.clone();
        let mut session = Interview::new(TEST_USER, farm_only_return());
        save_farms(&mut session, vec![farm]);
        unsupported(&session.draft, "Current production poultry timing");
        let kept = session.draft.clone();
        let reopened = reopened_farming(&mut session).1;
        assert!(reopened.iter().any(|document| matches!(document,IncomeDocument::Farm(farm) if matches!(&farm.more.supplies,PrepaidSupplies::Supplies {poultry,..} if poultry.current[0].started==started && poultry.current[0].adopted==method))));
        assert_eq!(session.draft, kept);
        unsupported(&restart(kept).draft, "Current production poultry timing");
    }
}
