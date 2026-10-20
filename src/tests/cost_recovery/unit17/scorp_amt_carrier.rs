// Dispatcher18b70772: exact Unit47 counterexample4674305e, registered in its assigned owner.
extern crate std;
use crate::tests::cross::interview::{document, k1_1120s, save_topic};
use crate::tests::*;

fn attempt(session: &mut Interview, request: P, case: &str) -> bool {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = session.handle(P::from_avro(&request.to_avro()).unwrap()).unwrap();
    let (Reply::Response(value) | Reply::Edit(value)) = &reply;
    if let Ok(root) = std::env::var("TAX_UNIT47_SCORP_AMT_CASE_DIR") {
        use std::io::Write;
        let directory = std::path::Path::new(&root).join(case);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("exchange.json"),serde_json::to_vec_pretty(&serde_json::json!({"native_pid":std::process::id(),"problem":matches!(value,P::Problem{..}),"draft_unchanged":session.draft==before,"before_dirty":dirty,"after_dirty":session.dirty})).unwrap()).unwrap();
        for (name, bytes) in [("request.avro", request.to_avro()), ("reply.avro", value.to_avro())]
        {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(directory.join(name))
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
        write_native_draft(directory.join("draft-before.avro"), &before);
        write_native_draft(directory.join("draft-after.avro"), &session.draft);
    }
    assert_eq!(P::from_avro(&value.to_avro()).unwrap(), *value);
    let passes = !matches!(value, P::Problem { .. });
    if !passes {
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
    passes
}

#[test]
fn unit17_scorp_amt_ordinary_projection_keeps_separately_admitted_source_amounts() {
    const MAX: i64 = 100_000_000_000;
    let mut page = k1_1120s("Explicit regular and AMT source");
    page.ordinary = MAX;
    // The synthetic source explicitly states the ordinary adjustment and all
    // separate balances, no capital/property/rental/loan records and no other
    // AMT differences. This uses the same source restatement as the existing
    // K1 page round-trip fixture; no unknown balance is inferred from omission.
    let regular = page.record();
    assert!(regular.valid());
    let mut alternate = ScorpAmtRecords::SameAsRegular.record(&regular);
    assert!(alternate.complete);
    assert_eq!(alternate.adjustment, 0);
    assert!(alternate.capital.is_none() && alternate.section1231.is_none());
    assert!(alternate.loans.is_empty());
    page.amt = ScorpAmtRecords::Separate { record: alternate.clone() };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1Scorp(page.clone()))],
    );
    assert!(attempt(&mut session, seed, "regular-max-explicit-amt-zero"));
    let id = session
        .draft
        .income
        .iter()
        .find_map(|entry| matches!(entry.income, FederalIncome::Scorp { .. }).then_some(entry.id))
        .unwrap();
    assert!(id > 0);
    alternate.adjustment = MAX;
    // Two independently admitted signed source amounts; the checked addition
    // is representable in i64. The resulting ordinary amount exists only in
    // the canonical AMT validation/calculation projection, not as a new input.
    assert_eq!(page.ordinary.checked_add(alternate.adjustment), Some(2 * MAX));
    page.amt = ScorpAmtRecords::Separate { record: alternate };
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Scorp(page))],
    );
    let passes = attempt(&mut session, request, "regular-max-plus-explicit-amt-max");
    // Preserve actual neutral handler reply and complete accepted state before
    // testing the recurring derived-carrier requirement. A failure is source
    // evidence for the protected owner, never permission to alter its formula.
    assert!(passes, "independent source bounds must not narrow the derived AMT carrier");
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let stored = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    let FederalIncome::Scorp { data } = &stored.income else { panic!("actual source") };
    assert_eq!(data.ordinary, MAX);
    assert_eq!(data.amt.as_ref().unwrap().adjustment, MAX);
    let projected = data.amt.as_ref().unwrap().source(data).unwrap();
    assert_eq!(projected.ordinary, 2 * MAX);
    assert!(!ScorpK1::valid(&projected));
    assert_eq!(
        crate::kernel::ratio::dollars(projected.basis_result().unwrap().stock.get()),
        2_000_000_000
    );
    // 2026-10-19: pool operands retain the two actual MAX-cent sources.
    assert_eq!(projected.pool_activity_result().unwrap().combined.passive, 200_000_000_000);
}

#[test]
fn unit17_scorp_amt_negative_sources_and_both_original_caps_remain_distinct() {
    const MAX: i64 = 100_000_000_000;
    let mut page = k1_1120s("Signed actual AMT source");
    page.ordinary = -MAX;
    let owner = page.record();
    assert!(owner.valid());
    let mut alternate = ScorpAmtRecords::SameAsRegular.record(&owner);
    alternate.adjustment = -MAX;
    assert!(alternate.valid(&owner));
    let projected = alternate.source(&owner).expect("two admitted signed sources");
    assert_eq!(projected.ordinary, -2 * MAX);
    let basis = projected.basis_result().unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(basis.suspended[crate::k1_loss::K1LossClass::Ordinary].get()),
        2_000_000_000
    );
    assert_eq!(projected.pool_activity_result().unwrap().combined.passive, 0);
    assert!(!ScorpK1::valid(&projected));
    // Neither editable cap is widened, even if the other amount cancels it.
    let mut oversized = owner.clone();
    oversized.ordinary = -MAX - 1;
    alternate.adjustment = 1;
    assert!(!alternate.valid(&oversized));
    assert!(alternate.source(&oversized).is_err());
    alternate.adjustment = -MAX - 1;
    assert!(!alternate.valid(&owner));
    assert!(alternate.source(&owner).is_err());
    // The capacity fit preserves the unrelated AMT balance/source guards.
    alternate.adjustment = -MAX;
    alternate.stock = MAX + 1;
    assert!(!alternate.valid(&owner));
    alternate.stock = 0;
    alternate.prior_nondeductible = 1;
    assert!(!alternate.valid(&owner));
}

#[test]
fn unit17_scorp_derived_capacity_never_admits_an_oversized_regular_source() {
    const MAX: i64 = 100_000_000_000;
    for amount in [-MAX - 1, MAX + 1] {
        let mut page = k1_1120s("Oversized regular source remains refused");
        page.ordinary = amount;
        let owner = page.record();
        assert!(!owner.valid());
        assert!(owner.basis_result().is_err());
        let mut alternate = ScorpAmtRecords::SameAsRegular.record(&owner);
        // Cancellation cannot turn an invalid original input into a genuine
        // derived source, even though its resulting amount is within100b.
        alternate.adjustment = if amount > 0 { -1 } else { 1 };
        assert!(!alternate.valid(&owner));
        assert!(alternate.source(&owner).is_err());
    }
}

#[test]
fn unit17_scorp_amt_capacity_survives_the_actual_sale_basis_risk_and_outcome_path() {
    const MAX: i64 = 100_000_000_000;
    let [_, FederalIncome::Scorp { mut data }] =
        crate::tests::passthrough::k1_sale_return::sources(MAX / 100, 0, 0, 20_000)
    else {
        panic!("existing actual S-corporation fixture")
    };
    // This independent control exercises the short-held OtherLoss/output
    // path; the existing long-held source fixtures keep section1231 intact.
    data.section179_sales[0].acquired = 20260101;
    data.amt.as_mut().unwrap().adjustment = MAX;
    assert!(data.valid());
    let projected = data.amt.as_ref().unwrap().source(&data).unwrap();
    assert_eq!(projected.ordinary, 2 * MAX);
    assert_eq!(projected.sale_results().unwrap()[0].gain, -2_000);
    let basis = projected.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 1_999_998_000);
    let ledger = projected.ledger().unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            ledger.basis_allowed[crate::k1_loss::K1LossClass::OtherLoss].get()
        ),
        2_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            ledger.risk_allowed[crate::k1_loss::K1LossClass::OtherLoss].get()
        ),
        2_000
    );
    let (flows, _) = projected.outcome();
    assert_eq!(flows.sales.len(), 1);
    assert_eq!(flows.sales[0].2, -200_000);
    // A missing AMT sale view is still unavailable; derived capacity does not
    // admit the regular monetary view as an AMT substitute.
    data.section179_sales[0].alternative = K1Section179SaleAmtSource::Unavailable;
    assert_eq!(
        data.amt.as_ref().unwrap().source(&data).err().unwrap().0,
        "ISSUE_K1_SALE_AMT_UNAVAILABLE"
    );
}

#[test]
fn unit17_scorp_amt_capacity_keeps_portfolio_and_business_property_source_guards() {
    const MAX: i64 = 100_000_000_000;
    let mut owner = k1_1120s("Explicit AMT source characters").record();
    owner.ordinary = MAX;
    owner.capital = Some(ScorpCapital {
        short: -10_000,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        basis_short: 0,
        basis_long: 0,
        basis_collectibles: 0,
        risk_short: 0,
        risk_long: 0,
        risk_collectibles: 0,
    });
    owner.section1231 = Some(K1Section1231 {
        net: 20_000,
        unrecaptured: 0,
        basis_loss: 0,
        risk_loss: 0,
        passive_loss: 0,
    });
    let mut alternate = amt_records(&owner);
    alternate.adjustment = MAX;
    let projected = alternate.source(&owner).unwrap();
    let basis = projected.basis_result().unwrap();
    let mut result = FederalResult::empty(0);
    assert_eq!(
        projected.capital_result_with_events(&basis, 1, true, &mut result, None).unwrap(),
        // 2026-10-19: the admitted short-term source remains -10,000 cents.
        [-10_000, 0, 0, 0]
    );
    projected.report_section1231(&basis, 1, true, &mut result);
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    // The same raw-cent pool includes the separately admitted 20,000-cent property gain.
    assert_eq!(projected.pool_activity_result().unwrap().combined.passive, 200_000_020_000);
    alternate.capital = None;
    assert_eq!(
        alternate.source(&owner).err().unwrap().0,
        "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_CAPITAL_AFCA6E93"
    );
    alternate.capital = owner.capital.clone();
    alternate.section1231 = None;
    assert_eq!(
        alternate.source(&owner).err().unwrap().0,
        "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_SECTION_D67066FF"
    );
}
