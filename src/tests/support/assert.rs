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
    add(&mut d, income(FederalIncome::Scorp { data: source }));
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
    let refusal = item.refused();
    let section = crate::interview::section_name(refusal.section);
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

// The source fields explicitly marked for document import, in source order.
pub(in crate::tests) fn imported_fields(page: &str) -> Vec<&str> {
    page.lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(text.trim_start_matches("pub ").split_once(':').map(|(n, _)| n));
            }
            Some(None)
        })
        .flatten()
        .collect()
}

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
