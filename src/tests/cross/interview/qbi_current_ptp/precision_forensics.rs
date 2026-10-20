// 2026-10-15: independent source-cent precision controls on unchanged cadf
// production. These are intended red witnesses until the existing eligibility
// owner preserves source cents through the single reported category boundary.
use super::*;

struct PrecisionCase {
    name: &'static str,
    raw_source_cents: &'static [i64],
    schedule_e_dollars: i64,
    taxable_dollars: i64,
    category_dollars: i64,
    deduction_dollars: i64,
    carry_dollars: i64,
}

fn run_precision_case(case: PrecisionCase) {
    let mut draft = complete(FederalStatus::Single);
    // These explicit synthetic W-2 wages offset the ORIGINAL ordinary-income
    // owner's separately rounded Schedule E amounts. No source is changed to
    // produce a desired QBI result; actual AGI and pre-QBI taxable income below
    // independently verify the tax point before the precision oracle.
    let wages_cents = (case.taxable_dollars + 16_100 - case.schedule_e_dollars) * 100;
    draft.income[0] = wage(1, Owner::Taxpayer, wages_cents, 0);
    let wage_source = draft.income[0].clone();
    let mut session = Interview::new(TEST_USER, draft);
    ptp_save_prior_carry(&mut session, &alloc::format!("{}-prior-carry", case.name), exchange);
    let mut pages = Vec::new();
    for (index, raw) in case.raw_source_cents.iter().enumerate() {
        let mut page = ptp_page(0, true);
        page.name = alloc::format!("Precision PTP source {}", index + 1);
        page.ordinary = *raw;
        let PartnerQbi::Statement { income, .. } = &mut page.qbi else {
            unreachable!()
        };
        *income = *raw;
        pages.push(page);
    }
    let preview = exchange(
        &mut session,
        credit_preview(IncomeDocument::K1Partnership(pages[0].clone())),
        &alloc::format!("{}-preview", case.name),
        true,
    );
    assert!(matches!(preview, Reply::Response(P::IncomePreview { .. })));
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        pages
            .iter()
            .cloned()
            .map(|page| document(0, IncomeDocument::K1Partnership(page)))
            .collect(),
    );
    let candidate_display = ptp_estimate_and_save(&mut session, request, case.name, exchange, credit_estimate_documents_from_save);
    let result = supported(&session.draft);
    // Capture actual financial values before any precision assertion. Pure
    // EstimateDocuments exports contain the actual candidate response, whereas
    // estimate-after.avro is explicitly the unchanged accepted-prior estimate.
    let actual = serde_json::json!({
        "agi_dollars": value(&result, "agi"),
        "pre_qbi_taxable_dollars": value(&result, "qbi_taxable_income"),
        "reported_reit_ptp_category_dollars": value(&result, "qbi_reit_ptp"),
        "deduction_dollars": value(&result, "qbi_deduction"),
        "taxable_dollars": value(&result, "taxable_income"),
        "business_carry_dollars": value(&result, "qbi_loss_carryforward"),
        "reit_ptp_carry_dollars": value(&result, "reit_ptp_loss_carryforward"),
    });
    if let Ok(root) = std::env::var("QBI_CURRENT_PTP_CASE_DIR") {
        let directory = std::path::Path::new(&root).join(alloc::format!("{}-oracle", case.name));
        std::fs::create_dir_all(&directory).unwrap();
        let oracle = serde_json::json!({
            "pid":std::process::id(), "case":case.name,
            "raw_current_source_cents":case.raw_source_cents,
            "unchanged_schedule_e_reported_dollars":case.schedule_e_dollars,
            "w2_cents":wages_cents, "fixed_prior_reported_qualified_carry_cents":300_000,
            "actual_saved_financial_result":actual,
            "independent_expected":{
                "agi_dollars":case.taxable_dollars+16_100,
                "pre_qbi_taxable_dollars":case.taxable_dollars,
                "reported_reit_ptp_category_dollars":case.category_dollars,
                "deduction_dollars":case.deduction_dollars,
                "reit_ptp_carry_dollars":case.carry_dollars,
            },
            "authority":"doc/app-contract.md Money precision: exact source cents through ratios; equivalent sources combined before one reported-category dollar rounding; fixed reported prior carry never rephased",
            "scope":"Actual native handler artifacts emitted before this independent oracle. No client/schema execution inferred. Candidate display is the actual IncomeRunningEstimate response; estimate-after.avro for pure estimate is accepted-prior state."
        });
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("financial-oracle.json"))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(&oracle).unwrap())
            .unwrap();
    }
    // Lifecycle and exact raw-source preservation execute before the final
    // precision assertions so a red financial result retains forensic state.
    assert_eq!(session.draft.income[0], wage_source);
    assert_eq!(
        session
            .draft
            .screening
            .as_ref()
            .unwrap()
            .qbi_history
            .reit_ptp,
        300_000
    );
    ptp_reopen_and_cold_resave(
        &mut session,
        &pages.iter().cloned().map(IncomeDocument::K1Partnership).collect::<Vec<_>>(),
        &result,
        case.name,
        exchange,
    );
    assert_eq!(candidate_display, crate::interview::estimate_text(&result));
    assert_eq!(value(&result, "agi"), case.taxable_dollars + 16_100);
    assert_eq!(value(&result, "qbi_taxable_income"), case.taxable_dollars);
    assert_eq!(
        value(&result, "qbi_reit_ptp"),
        case.category_dollars,
        "{}: round exact equivalent-source eligibility once at the reported category",
        case.name
    );
    assert_eq!(value(&result, "qbi_deduction"), case.deduction_dollars);
    assert_eq!(
        value(&result, "reit_ptp_loss_carryforward"),
        case.carry_dollars
    );
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    assert_eq!(
        value(&result, "taxable_income"),
        case.taxable_dollars - case.deduction_dollars
    );
}

macro_rules! precision_case {
    ($name:ident, $raw:expr, $ordinary:expr, $taxable:expr, $category:expr, $deduction:expr, $carry:expr) => {
        #[test]
        fn $name() {
            run_precision_case(PrecisionCase {
                name: stringify!($name),
                raw_source_cents: $raw,
                schedule_e_dollars: $ordinary,
                taxable_dollars: $taxable,
                category_dollars: $category,
                deduction_dollars: $deduction,
                carry_dollars: $carry,
            });
        }
    };
}

// The original source-cent amounts remain unchanged at all three tax points.
// Midpoint positive: 610450/2=305225c -> reported3052; net52 -> deduction10.
// Midpoint negative: -10450/2=-5225c -> reported-52 -> fixedcarry3052.
precision_case!(
    ptp_precision_positive_threshold,
    &[610_450],
    6_105,
    201_750,
    6_105,
    621,
    0
);
precision_case!(
    ptp_precision_positive_midpoint,
    &[610_450],
    6_105,
    239_250,
    3_052,
    10,
    0
);
precision_case!(
    ptp_precision_positive_ceiling,
    &[610_450],
    6_105,
    276_750,
    0,
    0,
    3_000
);
precision_case!(
    ptp_precision_negative_threshold,
    &[-10_450],
    -105,
    201_750,
    -105,
    0,
    3_105
);
precision_case!(
    ptp_precision_negative_midpoint,
    &[-10_450],
    -105,
    239_250,
    -52,
    0,
    3_052
);
precision_case!(
    ptp_precision_negative_ceiling,
    &[-10_450],
    -105,
    276_750,
    0,
    0,
    3_000
);
// Two equivalent real sources each10050c: each exacteligible5025c, combined
// category10050c -> reported101. Early dollars gives102; per-source eligible
// dollar rounding gives100. Fixedcarry3000 distinguishes all three outcomes.
precision_case!(
    ptp_precision_two_positive_equivalent_sources,
    &[10_050, 10_050],
    202,
    239_250,
    101,
    0,
    2_899
);
precision_case!(
    ptp_precision_two_negative_equivalent_sources,
    &[-10_050, -10_050],
    -202,
    239_250,
    -101,
    0,
    3_101
);
