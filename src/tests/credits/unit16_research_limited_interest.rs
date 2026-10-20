use super::*;
use crate::investment_interest::{
    InvestmentBorrowing, InvestmentElection, InvestmentInterestUse, InvestmentLimitedAllocation,
    InvestmentLoan,
};
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

fn draft() -> (FederalDraft, [i64; 2]) {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    add(&mut d, income(savings_income(9000, 0)));
    let mut ids = [0; 2];
    for (index, profit) in [50_000, 30_000].into_iter().enumerate() {
        let mut entry =
            income(FederalIncome::Scorp { data: complete_ordinary_scorp(profit, true) });
        entry.label = alloc::format!("Synthetic research corporation {}", index + 1);
        add(&mut d, entry);
        ids[index] = d.income.last().unwrap().id;
        let mut retained = carry(BusinessCredit::ResearchSmallBusiness, 2024, 100_000);
        retained.research_source = Some(ids[index]);
        history(&mut d).carryforwards.push(retained);
    }
    let deductions = d.deductions.as_mut().unwrap();
    itemized(deductions).real_estate_taxes = 2_000_000;
    let facts = itemized(deductions).clone();
    deductions.choice = DeductionSelection::Itemized { facts };
    deductions.investment_interest = Some(InvestmentBorrowing {
        loans: [Some(ids[0]), Some(ids[1]), None]
            .into_iter()
            .enumerate()
            .map(|(index, originating_k1)| InvestmentLoan {
                originating_k1,
                ..native_investment_loan(
                    alloc::format!("Retained portfolio loan {}", index + 1),
                    0,
                    900_000,
                    900_000,
                )
            })
            .collect(),
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    (d, ids)
}

fn borrowing(d: &FederalDraft) -> &InvestmentBorrowing {
    d.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap()
}

fn open(session: &mut Interview) -> P {
    let saved = session.draft.clone();
    match send(session, crate::tests::cross::interview::navigate(&saved, TaxSection::OtherItemized))
    {
        Reply::Edit(request @ P::SaveOtherItemized { .. }) => request,
        other => panic!("expected the other itemized editor: {other:?}"),
    }
}

fn assert_return(d: &FederalDraft, ids: [i64; 2], corrected: bool) {
    let r = supported(d);
    for (key, expected) in [
        ("agi", 189_000),
        ("qbi_deduction", 16_000),
        ("taxable_income", 144_000),
        ("regular_income_tax", 27_158),
        ("investment_interest_deduction", 9000),
        ("investment_interest_carryforward", 18_000),
        ("form_3800_line_11", 27_158),
    ] {
        assert_eq!(value(&r, key), expected, "{key}");
    }
    // Joint allowed9000 on three actual9000 claims: each gets3000. Correcting
    // only the third origin makes A6000/B3000. Native QBI shares10000/6000
    // and common20000 over164000 leave A32488 (corrected29854), B18439.
    // Independent line11/TI fractions give6127 (corrected5630) and3478.
    for (id, cap) in ids.into_iter().zip([if corrected { 5630 } else { 6127 }, 3478]) {
        let prefix =
            alloc::format!("business_credit_research_small_business_source_{id}_filer_0_2024");
        assert_eq!(value(&r, &alloc::format!("{prefix}_used")), cap);
        assert_eq!(value(&r, &alloc::format!("{prefix}_to_next_year")), 100_000 - cap);
    }
    assert_eq!(value(&r, "general_business_credit"), if corrected { 9108 } else { 9605 });
    let loan = borrowing(d);
    let routed = loan.route(loan.aggregate().calculate(9000, 0, 0, 0)).unwrap();
    assert_eq!(routed.forms, [[9000, 18_000], [9000, 18_000]]);
    assert_eq!(routed.sources.len(), 1);
    assert_eq!(routed.sources.get(&0), Some(&[[9000, 18_000], [9000, 18_000]]));
    assert_eq!(
        crate::investment_interest::research_parts(Some(loan), &Default::default(), 9000),
        [(ids[0], if corrected { 6000 } else { 3000 }), (ids[1], 3000)].into_iter().collect()
    );
}

#[test]
fn unit16_research_limited_interest_preserves_origins_and_one_allowed_deduction() {
    let (mut d, ids) = draft();
    assert_return(&d, ids, false);
    assert_return(&restart(d.clone()).draft, ids, false);
    d.income.reverse();
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans.reverse();
    assert_return(&d, ids, false);
    d.deductions
        .as_mut()
        .unwrap()
        .investment_interest
        .as_mut()
        .unwrap()
        .loans
        .iter_mut()
        .find(|loan| loan.originating_k1.is_none())
        .unwrap()
        .originating_k1 = Some(ids[0]);
    assert_return(&d, ids, true);
}

#[test]
fn unit16_research_limited_interest_actual_editor_save_reopen_restart_correction() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let (d, ids) = draft();
    let mut session = Interview::new(TEST_USER, d);
    let native = NativeInterviewCase("TAX_UNIT16_LIMITED_INTEREST");
    if native.calculation(&mut session) {
        return;
    }
    let seed = open(&mut session);
    native.write("SEED", &seed);
    let mut answers = seed;
    let corrected = std::env::var("TAX_UNIT16_LIMITED_INTEREST_CORRECTION").as_deref() == Ok("1");
    let P::SaveOtherItemized { value, .. } = &mut answers else { panic!() };
    let loan = &mut value.investment_interest.as_mut().unwrap().loans[2];
    loan.label = "Actual retained portfolio loan".into();
    if corrected {
        loan.originating_k1 = Some(crate::InvestmentK1Origin::Entity { source: ids[0] });
    }
    native.write("ANSWERS", &answers);
    if std::env::var("TAX_UNIT16_LIMITED_INTEREST_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let request = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, request));
    assert_return(&session.draft, ids, corrected);
    assert!(borrowing(&session.draft).loans.iter().all(|loan| loan.source == 0));
    assert_eq!(borrowing(&session.draft).loans[2].originating_k1, corrected.then_some(ids[0]));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open(&mut session);
    native.write("REOPENED", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_return(&session.draft, ids, corrected);
    if std::env::var("TAX_UNIT16_LIMITED_INTEREST_PAYLOAD").is_err() && !corrected {
        // The standalone native test also follows the accepted initial Save
        // with the actual origin correction, rather than mutating the draft.
        let mut correction = open(&mut session);
        let P::SaveOtherItemized { value, .. } = &mut correction else { panic!() };
        value.investment_interest.as_mut().unwrap().loans[2].originating_k1 =
            Some(crate::InvestmentK1Origin::Entity { source: ids[0] });
        accepted(&send(&mut session, correction));
        assert_return(&session.draft, ids, true);
        assert_return(&restart(session.draft.clone()).draft, ids, true);
    }
}
