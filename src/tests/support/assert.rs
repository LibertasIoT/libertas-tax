extern crate std;

use crate::tests::*;
pub(in crate::tests) fn value(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}: {r:?}")).amount
        / 100
}
// The printed form and line a calculation row carries.
pub(in crate::tests) fn printed_line(
    r: &FederalResult,
    id: &str,
) -> (crate::TaxForm, alloc::string::String) {
    let f = r
        .lines
        .iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("missing {id}"))
        .form_line
        .clone()
        .unwrap_or_else(|| panic!("{id} has no form line"));
    (f.form, f.line)
}
pub(in crate::tests) fn supported(d: &FederalDraft) -> FederalResult {
    let r = estimate(d);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:#?}", r.issues);
    crate::tests::federal::schedules_1_3::assert_schedule_1_foots(&r);
    crate::tests::federal::schedules_1_3::assert_total_tax_adds(&r);
    crate::tests::federal::schedules_1_3::assert_credit_bases_are_income_tax(&r);
    crate::tests::federal::schedules_1_3::assert_schedules_2_and_3_foot(&r);
    crate::tests::federal::schedules_1_3::assert_form_8582_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_8958_foots(d, &r);
    crate::tests::federal::schedules_1_3::assert_form_8959_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_6251_comparison_foots(&r);
    crate::tests::federal::amt_projection::assert_form_6251_rates_foot(&r);
    crate::tests::federal::schedules_1_3::assert_form_8801_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_5329_ira_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_5329_health_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_5329_rmd_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_8606_roth_foots(d, &r);
    crate::tests::federal::schedules_1_3::assert_form_8606_traditional_foots(&r);
    crate::tests::federal::schedules_1_3::assert_form_8915_foots(d, &r);
    crate::tests::federal::schedules_1_3::assert_form_4952_foots(&r);
    crate::tests::federal::schedules_1_3::assert_schedule_b_foots(&r);
    crate::tests::federal::schedules_1_3::assert_schedule_d_summary_foots(d, &r);
    crate::tests::activity::qbi_projection::assert_form_8995_foots(&r);
    // Form 1116 line 3e's Schedule D part is the gains before losses of every contributor after
    // its exclusions: the net plus the losses netted, less the K-3 gains kept out on purpose
    // (whole dollars: a community share rounds each total on its own).
    let [gross, footed] = r.capital_footing.expect("the estimate records its capital footing");
    assert_eq!(
        crate::kernel::ratio::dollars(gross),
        crate::kernel::ratio::dollars(footed),
        "capital_gross {gross} is not the Schedule D contributors after exclusions {footed}"
    );
    r
}
// A review note the estimate carries (Answers and unknowns: an assumption
// warning or a per-item notice), by resource key.
pub(in crate::tests) fn noted(r: &FederalResult, key: &str) -> bool {
    r.review_notes.iter().any(|n| n.display.key == key)
}
pub(in crate::tests) fn unsupported(d: &FederalDraft, section: &str) {
    let r = estimate(d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.section.contains(section)), "{r:?}");
    // Form 8615 line 1 is the one line an unsupported estimate keeps.
    assert!(
        r.tax.is_none()
            && r.refund.is_none()
            && r.lines.iter().all(|l| l.id == crate::federal_rules::FORM_8615_UNEARNED)
    );
}
// The review's rows as the user reads them.
pub(in crate::tests) struct Row {
    pub(in crate::tests) label: alloc::string::String,
    pub(in crate::tests) amount: i64,
    pub(in crate::tests) explanation: alloc::string::String,
    pub(in crate::tests) form_line: Option<FormLine>,
}
pub(in crate::tests) fn rows(amounts: &[FederalAmount]) -> Vec<Row> {
    amounts
        .iter()
        .map(|a| Row {
            label: status_text(&a.label),
            amount: a.amount,
            explanation: status_text(&a.explanation),
            form_line: a.form_line.clone(),
        })
        .collect()
}
// The calculation breakdown the review shows for a result.
pub(in crate::tests) fn breakdown(r: &FederalResult) -> Vec<Row> {
    rows(&FederalEstimate::from(r.clone()).breakdown)
}

// The review's rows, from the reply (FederalEstimate) or the calculation
// (FederalResult).
pub(in crate::tests) trait ReturnRows {
    fn rows(&self) -> &[FederalAmount];
}
impl ReturnRows for FederalResult {
    fn rows(&self) -> &[FederalAmount] {
        &self.return_lines
    }
}
impl ReturnRows for FederalEstimate {
    fn rows(&self) -> &[FederalAmount] {
        &self.return_lines
    }
}
// A Form 1116 calculation line in dollars, zero when the record does not exist.
pub(in crate::tests) fn form_1116_value(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount / 100)
}
// The first Form 1116 line of the regular tax or the AMT whose id ends with
// the suffix (lines 3b, 3e and 18 are the same in every record).
pub(in crate::tests) fn form_1116_any(r: &FederalResult, amt: bool, suffix: &str) -> i64 {
    r.lines
        .iter()
        .find(|l| {
            l.id.starts_with(if amt { "foreign_amt_" } else { "foreign_" })
                && (amt || !l.id.starts_with("foreign_amt_"))
                && l.id.ends_with(suffix)
        })
        .map_or(0, |l| l.amount / 100)
}
pub(in crate::tests) fn return_line(result: &impl ReturnRows, reference: &str) -> i64 {
    let line = FormLine::new(TaxForm::Form1040, reference);
    result
        .rows()
        .iter()
        .find(|row| row.form_line.as_ref() == Some(&line))
        .unwrap_or_else(|| panic!("no Form 1040 line {reference}"))
        .amount
}
pub(in crate::tests) fn scorp_risk_result(source: ScorpK1) -> (FederalDraft, FederalResult, i64) {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    assert!(d.valid_stored());
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    (d, r, id)
}

pub(in crate::tests) fn property_result(
    id: i64,
    net: i64,
    suspended: i64,
    nonpassive: i64,
) -> crate::extended_rules::K1PropertyResult {
    crate::extended_rules::K1PropertyResult {
        id,
        net,
        suspended,
        nonpassive,
        rentals: Vec::new(),
        net_cents: net.checked_mul(100).expect("finite fixture dollars"),
        section179_sales_cents: Vec::new(),
        nonpassive_cents: nonpassive.checked_mul(100).expect("finite fixture dollars"),
        suspended_cents: crate::tests::support::builders::k1_amount(suspended),
    }
}
pub(in crate::tests) fn ccc_value(d: &FederalDraft, r: &FederalResult, name: &str) -> i64 {
    value(r, &alloc::format!("farm_{}_ccc_{name}", d.income[0].id))
}

pub(in crate::tests) fn draft_2026_table_tax(income: i64, status: FederalStatus) -> i64 {
    let column = match status {
        FederalStatus::Single => 2,
        FederalStatus::Joint | FederalStatus::Surviving => 3,
        FederalStatus::Separate => 4,
        FederalStatus::Head => 5,
    };
    for line in include_str!("../../../tests/fixtures/irs-tax-table-2026.csv").lines() {
        let row: alloc::vec::Vec<i64> =
            line.split(',').map(|value| value.parse().unwrap()).collect();
        if row[0] <= income && income < row[1] {
            return row[column];
        }
    }
    panic!("no 2026 tax-table row for {income}")
}
pub(in crate::tests) fn farm_supply_value(d: &FederalDraft, r: &FederalResult, name: &str) -> i64 {
    value(r, &alloc::format!("farm_{}_supplies_{name}", d.income[0].id))
}

pub(in crate::tests) fn assert_situation_refused(
    d: &FederalDraft,
    item: impl Into<Chosen>,
    needles: &[&str],
) {
    let item = item.into();
    assert!(d.refusals().next().is_none());
    let mut refused = d.clone();
    item.choose(&mut refused);
    let r = estimate(&refused);
    assert_eq!(r.state, ResultState::Unsupported, "{item:?}: {r:?}");
    let section = crate::interview::section_name(TaxSection::OftenMissed);
    assert!(
        r.issues.iter().any(|i| i.section == section
            && needles.iter().all(|n| i.message.contains(n))
            && i.message.ends_with("is not calculated yet.")),
        "{item:?}: {r:?}"
    );
    assert!(r.tax.is_none() && r.refund.is_none() && r.lines.is_empty());
}
// A fuel sale or claim whose answers do not qualify is left out with a
// notice (Answers and unknowns); the estimate proceeds.
pub(in crate::tests) fn fuel_left_out(d: &FederalDraft) {
    let r = supported(d);
    assert!(
        noted(&r, "WARN_FUEL_SALE_LEFT_OUT") || noted(&r, "WARN_FUEL_CLAIM_LEFT_OUT"),
        "{:?}",
        r.review_notes
    );
}
// Either refused (inconsistent records) or left out with a notice: never a
// silent credit.
pub(in crate::tests) fn fuel_not_claimed(d: &FederalDraft) {
    let r = estimate(d);
    assert!(
        r.state == ResultState::Unsupported
            || noted(&r, "WARN_FUEL_SALE_LEFT_OUT")
            || noted(&r, "WARN_FUEL_CLAIM_LEFT_OUT"),
        "{r:?}"
    );
}
// Answers and unknowns, batch 5a: an ultimate-purchaser fuel use, or a
// credit without complete records, is left out of the fuel tax credit with a
// notice instead of refusing the estimate.
pub(in crate::tests) fn fuel_use_left_out(d: &FederalDraft) {
    let r = supported(d);
    assert!(
        noted(&r, "WARN_FUEL_USE_LEFT_OUT") || noted(&r, "WARN_FUEL_CREDIT_LEFT_OUT"),
        "{:?}",
        r.review_notes
    );
}
// A refund that needs the full tax-benefit worksheet or an unreconciled
// allocation is left out with a notice (Answers and unknowns).
pub(in crate::tests) fn refund_left_out(d: &FederalDraft) {
    let r = supported(d);
    assert!(noted(&r, "WARN_TAX_REFUND_LEFT_OUT"), "{r:?}");
}
// Either refused (inconsistent records) or left out with a notice.
pub(in crate::tests) fn refund_not_taxed_silently(d: &FederalDraft) {
    let r = estimate(d);
    assert!(r.state == ResultState::Unsupported || noted(&r, "WARN_TAX_REFUND_LEFT_OUT"), "{r:?}");
}

// The dollars each owner is told to carry into next year's Form 8606 line 2 (the 2026 Line 15c
// Worksheet's line 6), in the order of the notes: taxpayer first.
pub(in crate::tests) fn next_year_basis_notes(r: &FederalResult, key: &str) -> Vec<i64> {
    r.review_notes
        .iter()
        .filter(|n| n.display.key == key)
        .filter_map(|n| match n.display.args.get(1) {
            Some(MessageArg::Currency { scale, minor_units, .. }) => {
                Some(minor_units / 10_i64.pow(*scale as u32))
            }
            _ => None,
        })
        .collect()
}

pub(in crate::tests) fn care_line_value(r: &FederalResult, line: u32) -> i64 {
    match line {
        25 => value(r, "care_benefits_excluded"),
        26 => value(r, "care_benefits_taxable"),
        _ => value(r, &alloc::format!("care_form2441_line{line}")),
    }
}

pub(crate) fn carrier_schema() -> Option<serde_json::Value> {
    let path = std::env::var("TAX_UNIT47_CARRIER_SCHEMA").ok()?;
    Some(serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
}

pub(crate) fn schema_field<'a>(
    schema: &'a serde_json::Value,
    owner: &str,
    fields: &[&str],
) -> &'a serde_json::Value {
    let mut node = schema["Types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["NativeName"] == owner)
        .unwrap();
    for field in fields {
        node = node["Children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["NativeName"] == *field)
            .unwrap();
    }
    node
}

pub(in crate::tests) fn assert_native_cash_money(
    d: &FederalDraft,
    categories: [i64; 3],
    agi: i64,
) -> FederalResult {
    let r = supported(d);
    assert_eq!(value(&r, "agi"), agi);
    if d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint {
        assert_eq!(value(&r, "taxable_income"), 0);
        assert_eq!(r.tax, Some(0));
        assert_eq!(r.payments, Some(10_000 + categories.iter().sum::<i64>() * 100));
        assert_eq!(r.refund, r.payments);
    } else {
        // The actual MFS native category is30000 wages less16100 standard;
        // the13900–13950 TaxTable interval gives1423, paid200, due1223.
        assert_eq!(value(&r, "taxable_income"), 13_900);
        assert_eq!((r.tax, r.payments, r.balance), (Some(142_300), Some(20_000), Some(122_300)));
    }
    for (line, amount) in ["25a", "25b", "25c"].into_iter().zip(categories) {
        assert_eq!(return_line(&r, line), amount * 100);
    }
    r
}
pub(in crate::tests) fn assert_native_cash_row(
    r: &FederalResult,
    id: i64,
    owner: Owner,
    cents: [i64; 3],
) {
    let packet = r.withholding_sources.as_ref().expect("actual collection packet");
    assert!(packet.complete, "{packet:?}");
    let matches: Vec<_> = packet.rows.iter().filter(|r| r.id == id).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].owner, owner);
    assert_eq!(matches[0].cents, cents);
}

pub(crate) fn source_ids(session: &Interview) -> Vec<i64> {
    let ids: Vec<_> = session.draft.income.iter().map(|e| e.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    assert_eq!(ids.iter().copied().collect::<alloc::collections::BTreeSet<_>>().len(), ids.len());
    ids
}

// Derived catalogs have no independent count policy. Fresh-schema runs check
// the same edited declaration through the shared schema traversal.
pub(crate) fn catalog_unbounded(path: &[&str], element: &str) {
    let Some(schema) = carrier_schema() else {
        return;
    };
    let node = schema_field(&schema, path[0], &path[1..]);
    assert_eq!(node["Type"], "List");
    assert_eq!(node["Children"][0]["Type"], element);
    let attributes = node["Attributes"].as_array().unwrap();
    assert!(attributes.iter().any(|a| a["Name"] == "ReadOnly"));
    assert!(!attributes.iter().any(|a| a["Name"] == "SizeMax"));
}

/// An accepted label correction changes only that source label and revision.
pub(in crate::tests) fn source_label_change(
    before: &crate::FederalDraft,
    after: &crate::FederalDraft,
    source: i64,
    label: &str,
) {
    let mut expected = before.clone();
    expected.revision += 1;
    expected.income.iter_mut().find(|entry| entry.id == source).unwrap().label = label.into();
    assert_eq!(*after, expected);
}

pub(crate) mod k1res;

mod injured;
mod niit;

pub(in crate::tests) use injured::{
    assert_child_election_owner_source, assert_debt_owner_source, assert_ebl_money,
    assert_income_source_columns, assert_injured_ordinary_unchanged, assert_native_money_unchanged,
    assert_nonallocation_ordinary_unchanged, assert_noninvestment_election_money,
    assert_noninvestment_election_taxes, assert_portfolio_ordinary_unchanged,
    assert_resolved_income_sources, assert_rest_rounding_sources, assert_settled_care_sources,
    assert_wage_classification_money, column, expected_wage_classification_columns,
    niit_optional_column,
};

pub(in crate::tests) use niit::{
    assert_current_nua_money, assert_hobby_niit_money, assert_hobby_owner_columns,
    assert_k1_portfolio_owners, assert_k1_portfolio_owners_context, assert_mixed_niit_money,
    assert_niit_binding_money, assert_niit_binding_money_with_paid, assert_niit_deduction_money,
    assert_niit_person_money, assert_niit_zero_money, assert_retained_niit_money,
};

pub(in crate::tests) mod payroll;

pub(in crate::tests) fn injured_ordinary_money_unchanged(
    result: &FederalResult,
    ordinary: &FederalResult,
) {
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
    assert_eq!(
        result.lines.iter().filter(|line| !line.id.starts_with("form_8379_")).collect::<Vec<_>>(),
        ordinary.lines.iter().filter(|line| !line.id.starts_with("form_8379_")).collect::<Vec<_>>()
    );
}

pub(in crate::tests) mod injured_income;

pub(in crate::tests) mod injured_credit;

#[track_caller]
pub(in crate::tests) fn accepted(reply: &Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

pub(in crate::tests) fn rejected_with(reply: Reply, key: &str) {
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected {key}, got {reply:?}")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
}

pub(in crate::tests) mod k1_consolidated;

// Required calculation line, in native cents. Absence is a test failure.
pub(in crate::tests) fn line_amount(result: &FederalResult, id: &str) -> i64 {
    result
        .lines
        .iter()
        .find(|line| line.id == id)
        .unwrap_or_else(|| panic!("missing required line {id}: {result:?}"))
        .amount
}

pub(in crate::tests) mod unit50_retirement;

pub(in crate::tests) mod unit50_fuel;

// Actual Review money for a calculation's form row, after display projection.
pub(in crate::tests) fn displayed_line_amount(result: &FederalResult, id: &str) -> i64 {
    let line = result.lines.iter().find(|line| line.id == id).expect("calculation row");
    assert_eq!(line.kind, LineKind::Money);
    let label = line.form_line.as_ref().expect("form row").row_label(id).encode();
    crate::draft_return(result)
        .into_iter()
        .find(|row| row.label == label)
        .expect("displayed form row")
        .money
        .expect("displayed money")
}

pub(in crate::tests) fn assert_displayed_line_ratio(
    result: &FederalResult,
    id: &str,
    expected: i64,
    rendered_ratio: &str,
) {
    let line = result.lines.iter().find(|line| line.id == id).expect("calculation ratio");
    assert_eq!(line.kind, LineKind::Ratio);
    assert_eq!(line.amount, expected);
    let label = crate::line_label(id).english();
    assert!(!label.starts_with("LINE_"), "unresolved label: {label}");
    let rows = crate::draft_return(result);
    let row = rows
        .iter()
        .find(|row| row.form_line == line.form_line && row.explanation == line.explanation.encode())
        .expect("displayed ratio row");
    assert!(row.money.is_none());
    use libertas::{AvroDecode, LibertasMessageArgumentDecode as A};
    let mut offset = 0;
    let (key, args) = crate::common::FormattedMessage::avro_decode(&row.label, &mut offset)
        .expect("native Review ratio label");
    assert_eq!(offset, row.label.len());
    assert_eq!(key, "DRAFT_ROW_WITH_RATIO");
    let [A::FormattedText(label_bytes), A::Double(ratio)] = args.as_slice() else {
        panic!("native ratio arguments: {args:?}");
    };
    assert!(crate::Message::decode(label_bytes).english().contains(&label));
    assert_eq!(*ratio, rendered_ratio.parse::<f64>().expect("expected decimal ratio"));
    assert_eq!(alloc::format!("{ratio:.4}"), rendered_ratio);
}

pub(in crate::tests) fn stored_k1_1041(
    session: &Interview,
) -> (alloc::string::String, FederalEstateTrustPortfolioK1) {
    session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::EstateTrustPortfolio { data } => {
                Some((e.label.clone(), FederalEstateTrustPortfolioK1::from(data)))
            }
            _ => None,
        })
        .expect("a stored Schedule K-1 (Form 1041)")
}

pub(in crate::tests) fn admitted_brokerage_lots_capacity(count: usize) {
    use crate::tests::cross::interview::document;
    use crate::tests::support::client::income_pages::open_document_center;
    use libertas::data::{EncodedKey, wire};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.common.investment_sales.documents = (0..count)
        .map(|index| {
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(sale_document(SaleSource {
                        form: Sale1099B {
                            payer: crate::TaxPayerIdentity {
                                name: "Synthetic Brokerage".into(),
                                federal_id: "00-0000000".into(),
                                address: "123 Synthetic Street, Albany NY 00000".into(),
                            },
                            recipient: Default::default(),
                            // Descriptions have no App maximum. This deliberately long
                            // synthetic substitute-statement label identifies each lot.
                            description: alloc::format!(
                                "Synthetic brokerage lot {index:04} {}",
                                "X".repeat(512)
                            ),
                            digital_asset: false,
                            term: Some(GainTerm::Short),
                            proceeds: 90_000,
                            basis: Some(100_000),
                            reporting: BasisReporting::ReportedToIrs,
                            use_kind: SaleUse::Investment { wash_sale: 1_000 },
                            collectible: false,
                            market_discount: 0,
                            several_sales: false,
                            withholding: 0,
                            state_details: crate::StateWithholdingFields {
                                states: vec![crate::StateTaxWithholding {
                                    state: crate::TaxState::NY,
                                    payer_state_id: "SYNTHETIC-0000".into(),
                                    withholding: Some(0),
                                }],
                            },
                        },
                        amt: SaleAmt::SameAsRegular,
                        adjustment: SaleAdjustment::BasisCorrection { reported_basis: 99_000 },
                        special: SaleSpecial::NoneOfThese,
                    })),
                },
            )
        })
        .map(|row| crate::InvestmentSaleDocumentRow {
            id: row.id,
            owner: row.owner,
            community: row.community,
            refund_management: row.refund_management,
            allowed_community: row.allowed_community,
            preview: row.preview,
            investment_caps: row.investment_caps,
            refund_management_needed: row.refund_management_needed,
            document: match row.document {
                IncomeSource::InvestmentSale { value } => value,
                _ => unreachable!(),
            },
        })
        .collect();
    let expected: Vec<_> = documents
        .common
        .investment_sales
        .documents
        .iter()
        .map(|item| item.document.clone())
        .collect();
    let original_revision = session.draft.revision;
    let request_bytes = page.to_avro().len();
    let answers = crate::tests::support::client::income_pages::page_documents(&page);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.revision, original_revision + 1);
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &answers,
    );
    assert_eq!(session.draft.income.len(), count + 1, "the existing W-2 is retained");
    let reopened = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = reopened else { unreachable!() };
    assert_eq!(
        documents
            .common
            .investment_sales
            .documents
            .iter()
            .map(|item| &item.document)
            .collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>(),
        "every long label, correction and nested state fact survives native projection"
    );
    assert!(session.draft.valid_stored(), "{:?}", session.draft.stored_issue());
    crate::tests::core_tests::storage::exercise_heavy(&session.draft);
    let lots: Vec<_> = session
        .draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Sale { .. }))
        .collect();
    assert_eq!(lots.len(), count);
    let records = crate::income_records::project(TEST_USER, 0, &session.draft).unwrap();
    let mut payload_bytes = 0;
    let mut max_value = 0;
    let mut cold_records = Vec::new();
    for (key, value) in records {
        let encoded = EncodedKey::new(&key).unwrap();
        assert_eq!(encoded.as_bytes().len(), 32);
        assert_eq!(encoded.decode().unwrap(), key);
        let bytes = value.to_avro();
        assert!(bytes.len() < wire::MAX_VALUE_BYTES);
        max_value = max_value.max(bytes.len());
        // Hub mutation accounting is encoded key bytes plus encoded value bytes.
        payload_bytes += bytes.len() + encoded.as_bytes().len();
        cold_records.push((encoded.decode().unwrap(), TaxAppData::from_avro(&bytes).unwrap()));
    }
    cold_records.reverse();
    let mut cold = session.draft.clone();
    cold.income.clear();
    crate::income_records::assemble(
        TEST_USER,
        0,
        &mut cold,
        session.draft.income.len(),
        cold_records,
    )
    .unwrap();
    assert_eq!(cold, session.draft);
    let restored = restart(session.draft.clone());
    assert_eq!(restored.draft, session.draft);
    assert!(request_bytes < wire::MAX_REQUEST_BYTES);
    assert!(payload_bytes < wire::MAX_REQUEST_BYTES);
    // Income records alone are a lower bound: headers, people, sections,
    // order metadata and the copy counter add work.
    std::println!(
        "Admitted brokerage lots={count} request={request_bytes} income_record_mutation_bytes={payload_bytes} income_mutations={} max_value={max_value}",
        session.draft.income.len()
    );
}

// Each client rule's server twin: accept the boundary and reject one cent past it.
pub(in crate::tests) fn k1_1120s_boundary(index: usize) {
    use crate::tests::cross::interview::{
        k1_1120s, k1_1120s_more_mut, no_property_qbi, save_k1_1120s,
    };
    type Edit = Box<dyn Fn(&mut K1ScorpPaper, i64)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "SCORP_QUALIFIED_ABOVE_DIVIDENDS",
            Box::new(|p, extra| {
                p.dividends = 100_000;
                p.qualified_dividends = 100_000 + extra;
            }),
        ),
        (
            "SCORP_QBI_ABOVE_BOX_1",
            Box::new(|p, extra| {
                p.ordinary = 100_000;
                p.qbi = K1ScorpQbi::Statement {
                    ptp_income: 0,
                    reit_dividends: 0,
                    income: 100_000 + extra,
                    wages: 0,
                    property: 0,
                    service: false,
                    current_loss: 0,
                    property_qbi: no_property_qbi(),
                };
            }),
        ),
        (
            "SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
            Box::new(|p, extra| {
                let K1ScorpOtherBoxes { private_activity, exempt_interest, .. } =
                    k1_1120s_more_mut(p);
                *exempt_interest = 50_000;
                *private_activity = 50_000 + extra;
            }),
        ),
        (
            "SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION",
            Box::new(|p, extra| {
                // A zero carryover needs no election; one cent does.
                {
                    let K1ScorpCarried { basis_nondeductible, .. } = k1_boundary_continuing(p);
                    *basis_nondeductible = extra;
                }
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { basis_long, basis_collectibles, .. } =
                    k1_boundary_continuing(p);
                *basis_long = 20_000;
                *basis_collectibles = 20_000 + extra;
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { risk_long, risk_collectibles, .. } = k1_boundary_continuing(p);
                *risk_long = 20_000;
                *risk_collectibles = 20_000 + extra;
            }),
        ),
    ];
    let (key, edit) = cases.into_iter().nth(index).unwrap();
    let mut page = k1_1120s("Synthetic Corp");
    edit(&mut page, 0);
    let (_, reply) = save_k1_1120s(vec![page]);
    accepted(&reply);
    let mut page = k1_1120s("Synthetic Corp");
    edit(&mut page, 1);
    let (_, reply) = save_k1_1120s(vec![page]);
    rejected_with(reply, key);
}

// Immutable accepted source facts and request context; each caller owns its session.
pub(in crate::tests) fn retained_estate_period_baseline()
-> &'static (FederalDraft, P, FederalResult, [bool; 3]) {
    use crate::tests::cross::interview::{estate_nol_page, save_k1s};
    static BASELINE: std::sync::OnceLock<(FederalDraft, P, FederalResult, [bool; 3])> =
        std::sync::OnceLock::new();
    BASELINE.get_or_init(|| {
        // Retain actual short-period identity and chronology
        // above the old 100-loss / 1,000-period ceilings. Each post-2017 loss
        // is $1: all $101 is below both current regular and AMT income limits.
        let mut page = estate_nol_page("Synthetic many short periods", 2018, 1, 1, None);
        let K1FinalYear::Final { nol: Some(history), .. } = &mut page.more.final_year else {
            unreachable!()
        };
        retained_estate_nol_periods(history);
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let saved = session.draft.clone();
        let result = supported(&saved);
        assert_eq!(value(&result, "prior_nol_deduction"), 101);
        assert_eq!(value(&result, "prior_atnol_deduction"), 101);
        assert_eq!(value(&result, "regular_total_nol_carryforward"), 0);
        assert_eq!(value(&result, "amt_total_nol_carryforward"), 0);
        let (_, stored) = stored_k1_1041(&session);
        let history = stored.final_nol.unwrap();
        assert_eq!(history.years.len(), 101);
        assert_eq!(history.taxable_periods.len(), 1007);
        let correct =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let flags = session.oracle_flags();
        (saved, correct, result, flags)
    })
}

pub(in crate::tests) fn retained_estate_period_refusal(period_overlap: bool) {
    let (saved, _, _, flags) = retained_estate_period_baseline();
    let mut session = Interview::oracle_restore(TEST_USER, saved.clone(), *flags);
    let mut bad = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let dirty = session.dirty;
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    let (index, row) = documents
        .common
        .k1
        .documents
        .iter_mut()
        .enumerate()
        .find(|(_, row)| matches!(row.document, crate::K1Document::K1EstateTrust(_)))
        .unwrap();
    let crate::K1Document::K1EstateTrust(document) = &mut row.document else { unreachable!() };
    let history = document.statements.final_nol.as_mut().unwrap();
    let prefix =
        alloc::format!("documents/common/k1/documents/{index}/document/statements/final_nol");
    let (key, path) = if period_overlap {
        history.taxable_periods[1000].period.began = history.taxable_periods[999].period.ended;
        (
            "ESTATE_NOL_PERIOD_INVALID",
            alloc::format!("[{prefix}/taxable_periods/1000/period/began]"),
        )
    } else {
        history.years[100].original_period = history.years[99].original_period.clone();
        ("NOL_YEAR_TWICE", alloc::format!("[{prefix}/years/100/year]"))
    };
    for request in [crate::tests::support::client::estimate_documents_from_save(&bad), bad] {
        let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
            panic!("invalid retained estate source")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(field.as_deref(), Some(path.as_str()));
        assert_eq!(&session.draft, saved);
        assert_eq!(session.dirty, dirty);
    }
}

// A text form column must reach Review without a currency amount.
pub(in crate::tests) fn assert_displayed_text_line(result: &FederalResult, id: &str, text: &str) {
    let line = result.lines.iter().find(|line| line.id == id).expect("calculation row");
    assert_eq!(line.kind, LineKind::Text);
    assert_eq!(line.amount, 0);
    let rows = crate::draft_return(result);
    let row = rows
        .iter()
        .find(|row| row.explanation == line.explanation.encode())
        .expect("text form row");
    assert_eq!(row.form_line, line.form_line);
    assert_eq!(row.money, None);
    assert_eq!(row.amount, 0);
    assert!(
        Message::decode(&row.label).english().contains(text),
        "{}",
        Message::decode(&row.label).english()
    );
}

// A missing recipient correction identifies the durable component and its owning activity pages.
pub(in crate::tests) fn missing_related_activity(
    draft: &FederalDraft,
    result: &FederalResult,
    kind: crate::related_forms::DocumentKind,
    sections: &[TaxSection],
) {
    let components: Vec<_> = crate::related_forms::catalog(draft)
        .into_iter()
        .filter(|source| source.kind == kind)
        .collect();
    assert_eq!(components.len(), 1);
    let source = &components[0];
    let issues: Vec<_> = result
        .issues
        .iter()
        .filter(|issue| {
            issue.display.key == "ISSUE_RELATED_FORM_ACTIVITY_MISSING"
                && issue.context.identity()
                    == crate::federal_model::NoteIdentity::Income(source.form_id)
        })
        .collect();
    assert_eq!(issues.len(), 1, "{result:?}");
    assert_eq!(issues[0].display.args, vec![MessageArg::Encoded(source.part.clone())]);
    assert!(
        matches!(&issues[0].context, IssueContext::Corrective { sections: actual, .. } if actual == sections)
    );
}

// 2026-10-23: Reg. 1.179-2(c)(6), (8) requires the separate filer's own
// current conduct; combined spousal participation cannot certify that fact.
pub(in crate::tests) fn assert_mfs_section179_work_refused(
    draft: &FederalDraft,
    source: i64,
    page: TaxSection,
) {
    let result = estimate(draft);
    assert_eq!(result.state, ResultState::Unsupported);
    let issue = result.issues.iter().find(|issue|
        issue.display.key == "ISSUE_SECTION179_SEPARATE_OWN_WORK"
        && matches!(issue.context.source(), IssueContext::Income { id, .. } if *id == source)
    ).unwrap_or_else(|| panic!("{result:?}"));
    assert_eq!(
        crate::interview::issue_page(draft, &issue.context, Some(&issue.display.key)),
        Some(page)
    );
}
