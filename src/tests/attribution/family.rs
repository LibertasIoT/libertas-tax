// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
use crate::tests::family::family::{community_item_agi, community_pair, plain_item_agi};
use alloc::format;

#[test]
fn joint_financial_mfs_share_reaches_native_portfolio_lines_and_valid_endpoints() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::native_income::native_int1099;
    use crate::tests::support::builders::{
        document_owners_mut, joint_financial_session, set_joint_financial_share,
    };
    let (mut session, papers) = joint_financial_session(
        FederalStatus::Separate,
        vec![
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                    "Synthetic joint bank".into(),
                    1_001,
                ))),
            },
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(dividend_document(
                    dividend_source(native_dividend(2_001, 1_001, 3_001)),
                )),
            },
        ],
    );
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    let assert_no_community_report = |result: &FederalResult| {
        assert!(!result.lines.iter().any(|line| line.id.starts_with("form8958_")));
        assert!(!noted(result, "NOTE_FORM_8958_ALLOCATION"));
    };
    // Pub. 550, "Joint accounts": ownership determines each spouse's income.
    // The source's odd cents survive until each native reported line rounds.
    for (share, interest, ordinary, qualified, capital) in
        [(40, 4, 8, 4, 12), (0, 0, 0, 0, 0), (100, 10, 20, 10, 30)]
    {
        let mut request = information_return_answers_and_owners(
            &mut session,
            &papers,
            TaxSection::InterestDividends,
        )
        .unwrap();
        set_joint_financial_share(&mut session, &mut request, &ids, share);
        accepted(&send(&mut session, request));
        let result = supported(&session.draft);
        assert_no_community_report(&result);
        for (line, dollars) in
            [("2b", interest), ("3b", ordinary), ("3a", qualified), ("7a", capital)]
        {
            assert_eq!(return_line(&result, line), dollars * 100, "share {share}, line {line}");
        }
        assert_eq!(value(&result, "agi"), interest + ordinary + capital);
        assert_eq!(value(&result, "eic_investment_income"), interest + ordinary + capital);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
    // An exclusive spouse account is the same zero contribution on this
    // separate return; accept the ordinary owner answer instead of refusing it.
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    for person in document_owners_mut(&mut request).unwrap() {
        if person.person == -1 {
            person.documents.clear();
        } else if person.person == -2 {
            for row in &mut person.documents {
                row.ownership = crate::AddedOwnership::Yours;
            }
        }
    }
    accepted(&send(&mut session, request));
    assert!(session.draft.income.iter().all(|entry| entry.owner == crate::DocumentOwner::Spouses));
    let result = supported(&session.draft);
    assert_no_community_report(&result);
    assert_eq!(value(&result, "agi"), 0);
}

#[test]
fn joint_financial_community_treatment_composes_ownership_and_conserves_source_cents() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::native_income::native_int1099;
    use crate::tests::support::builders::{joint_financial_session, set_joint_financial_share};
    let (mut session, mut papers) = joint_financial_session(
        FederalStatus::Separate,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Synthetic joint bank".into(),
                1_001,
            ))),
        }],
    );
    session.draft.setup.as_mut().unwrap().community_property = community_arizona();
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    // Pub. 555: Arizona separate-property interest follows ownership; only
    // the community portion is halved. 40% * 90% + 60% * 10% = 42%.
    for (treatment, cents, dollars) in [
        (CommunityShare::Separate, 400, 4),
        (CommunityShare::Partly { percent: 20 }, 420, 4),
        (CommunityShare::Community, 501, 5),
    ] {
        papers[0].community = treatment;
        let mut request = information_return_answers_and_owners(
            &mut session,
            &papers,
            TaxSection::InterestDividends,
        )
        .unwrap();
        set_joint_financial_share(&mut session, &mut request, &ids, 40);
        accepted(&send(&mut session, request));
        let source = crate::community::SourceAmount::income(&session.draft, ids[0], 1_001);
        assert_eq!(source.filer, cents);
        assert_eq!(source.filer + source.counterpart, source.whole);
        assert_eq!(return_line(&supported(&session.draft), "2b"), dollars * 100);
    }
}

#[test]
fn joint_financial_fund_exclusion_applies_the_mfs_cap_after_ownership() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::{
        document_owners_mut, joint_financial_session, set_joint_financial_share,
    };
    let mut dividend = native_dividend(0, 0, 600_000_000);
    dividend.section_1202_gain = 600_000_000;
    dividend.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
    let (mut session, papers) = joint_financial_session(
        FederalStatus::Separate,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(dividend_source(
                dividend,
            ))),
        }],
    );
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    set_joint_financial_share(&mut session, &mut request, &ids, 50);
    accepted(&send(&mut session, request));
    // Each spouse owns $3 million, below the $5 million separate-return cap.
    // Capping the original $6 million before ownership would leave $500,000 taxable.
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "7a"), 0);
    assert_eq!(value(&result, "agi"), 0);
    for id in &ids {
        assert!(!result.lines.iter().any(|line| {
            line.id == format!("form8958_{id}_total")
                || line.id == format!("form8958_{id}_this_return")
        }));
    }
    let mut reopened =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    for person in document_owners_mut(&mut reopened).unwrap() {
        assert_eq!(person.income_share, Some(0), "person {}", person.person);
    }
}

#[test]
fn joint_financial_texas_separate_dividends_keep_distinct_income_and_gain_shares() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::{
        document_owners_mut, joint_financial_session, set_joint_financial_share,
    };
    let (mut session, mut papers) = joint_financial_session(
        FederalStatus::Separate,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(dividend_source(
                native_dividend(1_001, 501, 3_001),
            ))),
        }],
    );
    session.draft.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    papers[0].community = CommunityShare::Separate;
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    set_joint_financial_share(&mut session, &mut request, &ids, 40);
    accepted(&send(&mut session, request));
    // Pub. 555: separate-property income is community in Texas, while the
    // property's gain retains actual ownership. Qualified dividends are included.
    let result = supported(&session.draft);
    for (line, cents) in [("3b", 500), ("3a", 300), ("7a", 1_200)] {
        assert_eq!(return_line(&result, line), cents, "line {line}");
    }
    assert_eq!(value(&result, "agi"), 17);
    let mut reopened =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    for person in document_owners_mut(&mut reopened).unwrap() {
        assert_eq!(
            person.income_share,
            Some(if person.person == -1 { 1_701 } else { 2_301 }),
            "person {}",
            person.person,
        );
    }
}

#[test]
fn joint_financial_separate_niit_uses_resident_gains_and_savings_penalty() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::{joint_financial_session, set_joint_financial_share};
    let crate::InterestSource::Int1099(interest) =
        interest_source(native_interest(100_000, 10_000))
    else {
        unreachable!("printed interest fixture")
    };
    let (mut session, papers) = joint_financial_session(
        FederalStatus::Joint,
        vec![
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
            },
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(dividend_document(
                    dividend_source(native_dividend(200_000, 100_000, 50_000_000)),
                )),
            },
        ],
    );
    session.draft.set_situations(SpouseSituations::NONRESIDENT_SPOUSE.0, true);
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    set_joint_financial_share(&mut session, &mut request, &ids, 40);
    accepted(&send(&mut session, request));
    // Form 8960's default separate method: the resident owns $400 interest,
    // $800 ordinary dividends (qualified included), $200,000 gains and a $40
    // savings penalty. $201,160 minus $125,000, times 3.8%, rounds to $2,894.
    let result = supported(&session.draft);
    assert_eq!(value(&result, "net_investment_modified_agi"), 201_160);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 201_160);
    assert_eq!(value(&result, "net_investment_income_tax"), 2_894);
}

#[test]
fn joint_financial_separate_niit_applies_the_residents_fund_exclusion_cap() {
    use crate::tests::cross::interview::accepted;
    use crate::tests::support::builders::information_return_answers_and_owners;
    use crate::tests::support::builders::{joint_financial_session, set_joint_financial_share};
    let mut dividend = native_dividend(0, 0, 1_200_000_000);
    dividend.section_1202_gain = 1_200_000_000;
    dividend.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
    let (mut session, papers) = joint_financial_session(
        FederalStatus::Joint,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(dividend_source(
                dividend,
            ))),
        }],
    );
    session.draft.set_situations(SpouseSituations::NONRESIDENT_SPOUSE.0, true);
    let ids = session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>();
    let mut request =
        information_return_answers_and_owners(&mut session, &papers, TaxSection::InterestDividends)
            .unwrap();
    set_joint_financial_share(&mut session, &mut request, &ids, 40);
    accepted(&send(&mut session, request));
    // The resident's $4.8 million gain is below the MFS $5 million cap;
    // the joint return's $10 million cap does not determine separate NIIT.
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "7a"), 200_000_000);
    assert_eq!(value(&result, "net_investment_modified_agi"), 0);
    // 2026-10-27: Form 8960's Who Must File rule omits the form below the
    // resident's MAGI threshold: https://www.irs.gov/instructions/i8960
    assert!(!result.lines.iter().any(|line| line.id == "form_8960_net_investment_income"));
    assert_eq!(value(&result, "net_investment_income_tax"), 0);
}

#[test]
fn community_f4_arizona_halves_wages_and_interest_by_hand() {
    // Emery's return: wages 25,000 + 35,000 = 60,000; interest half of the
    // joint bank's 1,200 and none of Finley's inherited 800 (Arizona keeps
    // income from separate property separate); AGI 60,600; standard deduction
    // 16,100; taxable 44,500; Tax Table midpoint 44,525 = 1,240 + 12% x
    // 32,125 = 5,095; withholding 2,500 + 4,000.
    let d = community_pair(CommunityState::Arizona, CommunityRule::Halves, true);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 6_000_000);
    assert_eq!(return_line(&r, "2b"), 60_000);
    assert_eq!(value(&r, "agi"), 60_600);
    assert_eq!(return_line(&r, "15"), 4_450_000);
    assert_eq!(return_line(&r, "16"), 509_500);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(r.refund, Some(140_500));
    // Form 8958: column A is the whole item, B or C this return's share.
    for (id, total, this) in
        [(1, 5_000_000, 2_500_000), (2, 7_000_000, 3_500_000), (3, 120_000, 60_000), (4, 80_000, 0)]
    {
        assert_eq!(value_cents(&r, &alloc::format!("form8958_{id}_total")), total, "{id}");
        assert_eq!(value_cents(&r, &alloc::format!("form8958_{id}_this_return")), this, "{id}");
    }
    for (id, total, this) in [(1, 500_000, 250_000), (2, 800_000, 400_000)] {
        assert_eq!(value_cents(&r, &alloc::format!("form8958_withholding_{id}_total")), total);
        assert_eq!(value_cents(&r, &alloc::format!("form8958_withholding_{id}_this_return")), this);
    }
    assert!(noted(&r, "NOTE_FORM_8958_ALLOCATION"));
    // Emery's own Social Security and Medicare wages stay $50,000: no
    // Additional Medicare Tax, and Finley's payroll never joins this return.
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn community_f4_the_two_separate_returns_add_to_each_form_8958_row() {
    // Finley's return: 35,000 + 25,000 wages, 600 + 800 interest; AGI 61,400;
    // taxable 45,300; midpoint 45,325 = 1,240 + 12% x 32,925 = 5,191; the
    // same withholding halves; refund 1,309.
    let emery = supported(&community_pair(CommunityState::Arizona, CommunityRule::Halves, true));
    let finley = supported(&community_pair(CommunityState::Arizona, CommunityRule::Halves, false));
    assert_eq!(value(&finley, "agi"), 61_400);
    assert_eq!(return_line(&finley, "16"), 519_100);
    assert_eq!(finley.refund, Some(130_900));
    // Columns B + C = A: Emery's wages are item 1 on his return and item 2 on hers.
    assert_eq!(
        value_cents(&emery, "form8958_1_this_return")
            + value_cents(&finley, "form8958_2_this_return"),
        value_cents(&emery, "form8958_1_total")
    );
    assert_eq!(
        value_cents(&emery, "form8958_3_this_return")
            + value_cents(&finley, "form8958_3_this_return"),
        value_cents(&emery, "form8958_3_total")
    );
    // The inherited account: all on Finley's return, none on Emery's.
    assert_eq!(value_cents(&emery, "form8958_4_this_return"), 0);
    assert_eq!(value_cents(&finley, "form8958_4_this_return"), 80_000);
}

#[test]
fn community_f4_separate_property_income_is_community_in_texas_idaho_louisiana_wisconsin() {
    // Pub. 555 "Income from separate property": Finley's inherited account's
    // interest is community income in Texas, so Emery reports 600 + 400 = 1,000:
    // AGI 61,000, taxable 44,900, midpoint 44,925 = 1,240 + 12% x 32,525 = 5,143.
    for state in [
        CommunityState::Texas,
        CommunityState::Idaho,
        CommunityState::Louisiana,
        CommunityState::Wisconsin,
        CommunityState::Country { separate_income_is_community: true },
    ] {
        let r = supported(&community_pair(state, CommunityRule::Halves, true));
        assert_eq!(return_line(&r, "2b"), 100_000, "{state:?}");
        assert_eq!(value(&r, "agi"), 61_000, "{state:?}");
        assert_eq!(return_line(&r, "16"), 514_300, "{state:?}");
    }
    for state in [
        CommunityState::Arizona,
        CommunityState::California,
        CommunityState::Nevada,
        CommunityState::NewMexico,
        CommunityState::Washington,
        CommunityState::Country { separate_income_is_community: false },
    ] {
        let r = supported(&community_pair(state, CommunityRule::Halves, true));
        assert_eq!(return_line(&r, "2b"), 60_000, "{state:?}");
    }
}

#[test]
fn community_f4_spouses_living_apart_all_year_and_a_nonresident_spouse_split_by_earner() {
    // Section 66(a) and section 879: pay goes to the spouse who earned it
    // (Emery 50,000, withholding 5,000), the joint account's interest is
    // still halved (600), Finley's separate interest stays Finley's: AGI
    // 50,600, taxable 34,500, midpoint 34,525 = 1,240 + 12% x 22,125 = 3,895.
    for rule in [CommunityRule::LivingApartAllYear, CommunityRule::NonresidentSpouse] {
        let d = community_pair(CommunityState::Texas, rule, true);
        let r = supported(&d);
        assert_eq!(return_line(&r, "1a"), 5_000_000, "{rule:?}");
        assert_eq!(return_line(&r, "2b"), 60_000, "{rule:?}");
        assert_eq!(value(&r, "agi"), 50_600, "{rule:?}");
        assert_eq!(return_line(&r, "16"), 389_500, "{rule:?}");
        assert_eq!(return_line(&r, "25a"), 500_000, "{rule:?}");
        assert_eq!(value_cents(&r, "form8958_2_this_return"), 0);
    }
}

// The cents of a calculation row (value() is whole dollars).
fn value_cents(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}")).amount
}
#[test]
fn community_multiple_w2s_allocate_each_employer_then_aggregate_return() {
    let d = community_multi_w2_draft();
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 8_000_000);
    assert_eq!(return_line(&r, "25a"), 800_000);
    assert_eq!(value(&r, "agi"), 80_000);
    for (id, wages, withheld) in [
        (1, 4_000_002, 400_002),
        (2, 7_000_000, 700_000),
        (3, 1_999_998, 199_998),
        (4, 3_000_000, 300_000),
    ] {
        let cents = |name: alloc::string::String| line_amount(&r, &name);
        assert_eq!(cents(alloc::format!("form8958_{id}_total")), wages);
        assert_eq!(cents(alloc::format!("form8958_{id}_this_return")), wages / 2);
        assert_eq!(cents(alloc::format!("form8958_withholding_{id}_total")), withheld);
        assert_eq!(cents(alloc::format!("form8958_withholding_{id}_this_return")), withheld / 2);
    }
    assert!(noted(&r, "NOTE_FORM_8958_ALLOCATION"));

    let mut control = complete(FederalStatus::Separate);
    control.deductions.as_mut().unwrap().itemize_override = None;
    control.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 800_000);
    let expected = supported(&control);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(r.refund, expected.refund);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn community_multiple_w2s_keep_worker_payroll_and_earned_compensation() {
    let mut high = community_w2_draft(12_000_000, 13_000_000, 0, 0);
    add(&mut high, wage(0, Owner::Taxpayer, 12_000_000, 0));
    add(&mut high, wage(0, Owner::Spouse, 13_000_000, 0));
    let r = supported(&high);
    assert_eq!(value(&r, "wages"), 250_000);
    assert_eq!(value(&r, "additional_medicare_tax"), 1_035);
    assert_eq!(return_line(&r, "25c"), 0);
    assert_eq!(return_line(&r, "31"), 344_100); // Own two employers' excess Social Security withholding only.

    let mut ira = community_w2_draft(250_000, 5_000_000, 0, 0);
    add(&mut ira, wage(0, Owner::Taxpayer, 250_000, 0));
    add(&mut ira, wage(0, Owner::Spouse, 5_000_000, 0));
    let mut contribution = ira_facts(400_000, 0, Answer::No);
    contribution.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    ira.adjustments.as_mut().unwrap().iras = contribution;
    assert_eq!(value(&supported(&ira), "ira_deduction"), 4_000);
    if let FederalIncome::Wage { data } = &mut ira.income[2].income {
        edit_wage(data).wages = 0;
        edit_wage(data).social_wages = 0;
        edit_wage(data).social_tax = 0;
        edit_wage(data).medicare_wages = 0;
        edit_wage(data).medicare_tax = 0;
    }
    unsupported(&ira, "IRA");

    let mut eic = community_w2_draft(500_000, 0, 0, 0);
    add(&mut eic, wage(0, Owner::Taxpayer, 500_000, 0));
    add(&mut eic, wage(0, Owner::Spouse, 0, 0));
    eic.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut eic);
    let r = supported(&eic);
    assert_eq!(value(&r, "agi"), 5_000);
    assert_eq!(value(&r, "earned_income_credit"), eic_amount(10_000, 5_000, 1, false));
    assert_eq!(restart(eic.clone()).draft, eic);
}

#[test]
fn community_multiple_portfolio_payers_split_each_form_8958_row_and_full_return_once() {
    let d = community_portfolio_with_extra_w2s(community_portfolio_with_second_payers(
        community_joint_portfolio_draft(6_000_002, 10_000_000, 200_000, 400_000, 200_000),
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 11_000_000);
    assert_eq!(return_line(&r, "2b"), 250_000);
    assert_eq!(return_line(&r, "3a"), 150_000);
    assert_eq!(return_line(&r, "3b"), 500_000);
    assert_eq!(return_line(&r, "25a"), 1_100_000);
    assert_eq!(value(&r, "agi"), 117_500);
    for (id, amount, payer) in [(3, 200_000, "Bank A"), (5, 300_000, "Bank B")] {
        let total = alloc::format!("form8958_{id}_total");
        let each = alloc::format!("form8958_{id}_this_return");
        let row = r.lines.iter().find(|line| line.id == total).unwrap();
        assert_eq!(row.amount, amount);
        assert!(row.explanation.english().contains(payer));
        assert_eq!(line_amount(&r, &each), amount / 2);
    }
    for (id, ordinary, payer) in [(4, 400_000, "Fund A"), (6, 600_000, "Fund B")] {
        let total = alloc::format!("form8958_{id}_total");
        let each = alloc::format!("form8958_{id}_this_return");
        let row = r.lines.iter().find(|line| line.id == total).unwrap();
        assert_eq!(row.amount, ordinary);
        assert!(row.explanation.english().contains(payer));
        assert_eq!(line_amount(&r, &each), ordinary / 2);
    }
    assert_eq!(
        r.lines
            .iter()
            .filter(|line| line.id.starts_with("form8958_")
                && line.id.ends_with("_this_return")
                && !line.id.contains("withholding"))
            .count(),
        8
    );

    let mut control = complete(FederalStatus::Separate);
    control.deductions.as_mut().unwrap().itemize_override = None;
    control.income[0] = wage(1, Owner::Taxpayer, 11_000_000, 1_100_000);
    add(
        &mut control,
        income(FederalIncome::Interest { data: interest_source(native_interest(250_000, 0)) }),
    );
    add(
        &mut control,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(500_000, 150_000, 0)
            }),
        }),
    );
    let expected = supported(&control);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(r.refund, expected.refund);
    let mut no_preference = d.clone();
    for entry in &mut no_preference.income {
        if let FederalIncome::Dividend { data } = &mut entry.income {
            edit_dividend(data).qualified = 0;
        }
    }
    assert!(
        value(&r, "regular_income_tax") < value(&supported(&no_preference), "regular_income_tax")
    );
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn community_multiple_portfolio_payers_feed_niit_and_eic_without_splitting_worker_wages() {
    let mut high = community_portfolio_with_second_payers(community_joint_portfolio_draft(
        6_500_000, 6_500_000, 200_000, 400_000, 200_000,
    ));
    add(&mut high, wage(0, Owner::Taxpayer, 6_500_000, 0));
    add(&mut high, wage(0, Owner::Spouse, 6_500_000, 0));
    let result = supported(&high);
    assert_eq!(value(&result, "agi"), 137_500);
    assert_eq!(value(&result, "eic_investment_income"), 7_500);
    assert_eq!(value(&result, "net_investment_income_tax"), 285);
    assert_eq!(value(&result, "additional_medicare_tax"), 45);

    let mut eic = community_joint_portfolio_draft(250_000, 0, 100_000, 100_000, 50_000);
    let mut second_interest = eic.income[2].clone();
    second_interest.label = "Second bank".into();
    add(&mut eic, second_interest);
    let mut second_dividend = eic.income[3].clone();
    second_dividend.label = "Second fund".into();
    add(&mut eic, second_dividend);
    add(&mut eic, wage(0, Owner::Taxpayer, 250_000, 0));
    add(&mut eic, wage(0, Owner::Spouse, 0, 0));
    eic.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut eic);
    let result = supported(&eic);
    assert_eq!(value(&result, "agi"), 4_500);
    assert_eq!(value(&result, "eic_investment_income"), 2_000);
    assert_eq!(value(&result, "earned_income_credit"), eic_amount(5_000, 4_500, 1, false));
    assert_eq!(restart(eic.clone()).draft, eic);
}

#[test]
fn community_joint_portfolio_combines_niit_and_eic_investment_income() {
    let d =
        community_joint_portfolio_draft(13_000_000, 13_000_000, 1_000_000, 2_000_000, 1_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 145_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 570);
    assert_eq!(return_line(&r, "2b"), 500_000);
    assert_eq!(return_line(&r, "3b"), 1_000_000);

    let mut eic = community_joint_portfolio_draft(1_000_000, 0, 200_000, 200_000, 100_000);
    eic.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut eic);
    let result = supported(&eic);
    assert_eq!(value(&result, "agi"), 7_000);
    assert_eq!(value(&result, "eic_investment_income"), 2_000);
    assert_eq!(value(&result, "earned_income_credit"), eic_amount(10_000, 7_000, 1, false));
    assert_eq!(restart(eic.clone()).draft, eic);

    let mut over_limit = community_joint_portfolio_draft(1_000_000, 0, 1_000_000, 1_500_000, 0);
    over_limit.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut over_limit);
    let result = supported(&over_limit);
    assert_eq!(value(&result, "eic_investment_income"), 12_500);
    assert_eq!(value(&result, "earned_income_credit"), 0);
}

#[test]
fn community_qualified_dividends_use_preferential_worksheet_without_extra_income() {
    let qualified =
        community_dividend_draft(8_000_000, 8_000_000, 4_000_000, 4_000_000, Owner::Taxpayer);
    let mut ordinary = qualified.clone();
    let FederalIncome::Dividend { data } = &mut ordinary.income[2].income else { unreachable!() };
    edit_dividend(data).qualified = 0;
    let qualified_result = supported(&qualified);
    let ordinary_result = supported(&ordinary);
    assert_eq!(value(&qualified_result, "agi"), 100_000);
    assert_eq!(value(&ordinary_result, "agi"), 100_000);
    assert_eq!(return_line(&qualified_result, "3b"), 2_000_000);
    assert_eq!(return_line(&qualified_result, "3a"), 2_000_000);
    assert_eq!(return_line(&ordinary_result, "3a"), 0);
    assert!(
        value(&qualified_result, "regular_income_tax")
            < value(&ordinary_result, "regular_income_tax")
    );
}

#[test]
fn community_dividends_feed_niit_and_eic_investment_but_not_earned_wages() {
    let d = community_dividend_draft(13_000_000, 13_000_000, 2_000_000, 1_000_000, Owner::Taxpayer);
    let result = supported(&d);
    assert_eq!(return_line(&result, "3b"), 1_000_000);
    assert_eq!(return_line(&result, "3a"), 500_000);
    assert_eq!(value(&result, "agi"), 140_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 380);

    let mut eic = community_dividend_draft(1_000_000, 0, 200_000, 100_000, Owner::Spouse);
    eic.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut eic);
    let eic_result = supported(&eic);
    assert_eq!(value(&eic_result, "agi"), 6_000);
    assert_eq!(value(&eic_result, "eic_investment_income"), 1_000);
    assert_eq!(value(&eic_result, "earned_income_credit"), eic_amount(10_000, 6_000, 1, false));
    assert_eq!(restart(eic.clone()).draft, eic);

    let mut over_limit =
        community_dividend_draft(1_000_000, 0, 2_500_000, 2_000_000, Owner::Taxpayer);
    over_limit.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut over_limit);
    let result = supported(&over_limit);
    assert_eq!(value(&result, "eic_investment_income"), 12_500);
    assert_eq!(value(&result, "earned_income_credit"), 0);

    let mut ira = community_dividend_draft(500_000, 10_000_000, 200_000, 100_000, Owner::Spouse);
    let mut contribution = ira_facts(400_000, 0, Answer::No);
    contribution.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    ira.adjustments.as_mut().unwrap().iras = contribution;
    assert_eq!(value(&supported(&ira), "ira_deduction"), 4_000);
    ira.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    unsupported(&ira, "IRA");
}

#[test]
fn community_interest_feeds_niit_and_eic_investment_without_earning_wages() {
    let d = community_interest_draft(13_000_000, 13_000_000, 2_000_000, Owner::Taxpayer);
    let result = supported(&d);
    assert_eq!(return_line(&result, "2b"), 1_000_000);
    assert_eq!(value(&result, "agi"), 140_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 380);

    let mut d = community_interest_draft(1_000_000, 0, 200_000, Owner::Spouse);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut d);
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 6_000);
    assert_eq!(value(&result, "eic_investment_income"), 1_000);
    assert_eq!(value(&result, "earned_income_credit"), eic_amount(10_000, 6_000, 1, false));
    assert_eq!(restart(d.clone()).draft, d);

    let mut too_much_investment =
        community_interest_draft(1_000_000, 0, 2_500_000, Owner::Taxpayer);
    too_much_investment.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut too_much_investment);
    let result = supported(&too_much_investment);
    assert_eq!(value(&result, "eic_investment_income"), 12_500);
    assert_eq!(value(&result, "earned_income_credit"), 0);

    let mut ira_draft = community_interest_draft(500_000, 10_000_000, 200_000, Owner::Spouse);
    let mut ira = ira_facts(400_000, 0, Answer::No);
    ira.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    ira_draft.adjustments.as_mut().unwrap().iras = ira;
    assert_eq!(value(&supported(&ira_draft), "ira_deduction"), 4_000);
    ira_draft.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    unsupported(&ira_draft, "IRA");
}

#[test]
fn community_w2_mfs_allocates_form_8958_wages_and_income_tax_withholding() {
    let d = community_w2_draft(6_000_000, 10_000_000, 650_000, 1_200_000);
    let r = supported(&d);
    assert_eq!(value(&r, "wages"), 80_000);
    assert_eq!(return_line(&r, "1a"), 8_000_000);
    assert_eq!(return_line(&r, "25a"), 925_000);
    assert_eq!(value(&r, "form8958_1_this_return"), 30_000);
    assert_eq!(value(&r, "form8958_2_this_return"), 50_000);
    assert_eq!(value(&r, "form8958_1_total"), 60_000);
    assert_eq!(value(&r, "form8958_2_total"), 100_000);
    assert_eq!(value(&r, "form8958_withholding_1_this_return"), 3_250);
    assert_eq!(value(&r, "form8958_withholding_2_this_return"), 6_000);
    assert_eq!(value(&r, "form8958_withholding_1_total"), 6_500);
    assert_eq!(value(&r, "form8958_withholding_2_total"), 12_000);
    let mut one_w2 = complete(FederalStatus::Separate);
    one_w2.deductions.as_mut().unwrap().itemize_override = None;
    one_w2.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 925_000);
    let control = supported(&one_w2);
    assert_eq!(r.tax, control.tax);
    assert_eq!(r.refund, control.refund);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn community_dividends_split_form_8958_and_full_return_with_qualified_subset() {
    let mut d = community_dividend_draft(6_000_000, 10_000_000, 400_000, 200_000, Owner::Spouse);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).withholding = 650_000;
    }
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        edit_wage(data).withholding = 1_200_000;
    }
    let result = supported(&d);
    assert_eq!(return_line(&result, "1a"), 8_000_000);
    assert_eq!(return_line(&result, "3a"), 100_000);
    assert_eq!(return_line(&result, "3b"), 200_000);
    assert_eq!(return_line(&result, "25a"), 925_000);
    assert_eq!(value(&result, "agi"), 82_000);
    assert_eq!(value(&result, "form8958_3_total"), 4_000);
    assert_eq!(value(&result, "form8958_3_this_return"), 2_000);
    assert!(noted(&result, "NOTE_FORM_8958_ALLOCATION"));

    let mut control = complete(FederalStatus::Separate);
    control.deductions.as_mut().unwrap().itemize_override = None;
    control.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 925_000);
    add(
        &mut control,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(200_000, 100_000, 0)
            }),
        }),
    );
    let expected = supported(&control);
    assert_eq!(result.tax, expected.tax);
    assert_eq!(result.refund, expected.refund);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, result.tax);
}

#[test]
fn community_interest_mfs_allocates_form_8958_and_full_return_once() {
    let mut d = community_interest_draft(6_000_000, 10_000_000, 400_000, Owner::Spouse);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).withholding = 650_000;
    }
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        edit_wage(data).withholding = 1_200_000;
    }
    let result = supported(&d);
    assert_eq!(return_line(&result, "1a"), 8_000_000);
    assert_eq!(return_line(&result, "2b"), 200_000);
    assert_eq!(return_line(&result, "25a"), 925_000);
    assert_eq!(value(&result, "agi"), 82_000);
    assert_eq!(value(&result, "form8958_3_total"), 4_000);
    assert_eq!(value(&result, "form8958_3_this_return"), 2_000);
    assert_eq!(value(&result, "form8958_1_this_return"), 30_000);
    assert_eq!(value(&result, "form8958_2_this_return"), 50_000);
    assert!(noted(&result, "NOTE_FORM_8958_ALLOCATION"));

    let mut control = complete(FederalStatus::Separate);
    control.deductions.as_mut().unwrap().itemize_override = None;
    control.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 925_000);
    add(
        &mut control,
        income(FederalIncome::Interest { data: interest_source(native_interest(200_000, 0)) }),
    );
    let expected = supported(&control);
    assert_eq!(result.tax, expected.tax);
    assert_eq!(result.refund, expected.refund);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, result.tax);
}

#[test]
fn the_community_factor_divides_every_income_kind_that_enters_the_totals() {
    // Each kind below enters Form 1040 through the return totals; its $2,000 item
    // is half the filer's when Community (whoever owns the document), the whole
    // item when the filer's Separate, and none of it when the spouse's Separate.
    let kinds: Vec<(&str, FederalIncomeEntry)> = vec![
        ("interest", income(savings_income(2000, 0))),
        ("dividend", income(qualified_dividend(200_000))),
        ("sale", income(ordinary_investment_sale(6000, 2000, GainTerm::Long))),
        ("royalty", income(royalty(200_000, 0))),
        ("personal rent", income(personal_rent(2000, 0))),
        (
            "pension",
            retirement_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                2000,
                RetirementTreatment::Ordinary,
            ),
        ),
        ("omitted wages", income(omitted_employee_wages(2000))),
        ("prize", income(prize(PrizeDisposition::Received))),
        ("undistributed gain", income(undistributed_gain())),
        ("hobby", income(hobby_sales(2000, 0, 0))),
        ("unemployment", government_unemployment(2000, &[], 300)),
        ("gambling", gambling(2000, 0)),
        ("return of capital", income(capital_return(200_000, 0, GainTerm::Long))),
    ];
    for (name, entry) in kinds {
        // The item alone: what a plain separate return adds above the wages.
        let plain = plain_item_agi(entry.clone());
        let item = plain - 30_000;
        assert!(item > 0, "{name}");
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let r = community_item_agi(entry.clone(), owner, CommunityShare::Community);
            assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{name}: {r:?}");
            assert_eq!(value(&r, "agi"), 15_000 + item / 2, "{name} {owner:?}");
        }
        let own = community_item_agi(entry.clone(), Owner::Taxpayer, CommunityShare::Separate);
        assert_eq!(value(&own, "agi"), 15_000 + item, "{name}");
        let theirs = community_item_agi(entry.clone(), Owner::Spouse, CommunityShare::Separate);
        assert_eq!(value(&theirs, "agi"), 15_000, "{name}");
        // Partly community at 60%: the owner keeps 40% + half of 60% = 70%.
        let partly = community_item_agi(
            entry.clone(),
            Owner::Taxpayer,
            CommunityShare::Partly { percent: 60 },
        );
        assert_eq!(value(&partly, "agi"), 15_000 + item * 7 / 10, "{name}");
        let spouse_partly = community_item_agi(
            entry.clone(),
            Owner::Spouse,
            CommunityShare::Partly { percent: 60 },
        );
        assert_eq!(value(&spouse_partly, "agi"), 15_000 + item * 3 / 10, "{name}");
    }
}

#[test]
fn community_source_records_allocate_before_their_own_limits() {
    let rented = income(FederalIncome::Rental { data: rental(40_000, 0) });
    let own = community_item_agi(rented.clone(), Owner::Taxpayer, CommunityShare::Separate);
    assert_eq!(value(&own, "agi"), 45_000);
    let theirs = community_item_agi(rented.clone(), Owner::Spouse, CommunityShare::Separate);
    assert_eq!(value(&theirs, "agi"), 15_000);
    assert!(noted(&theirs, "NOTE_COMMUNITY_ITEM_IS_THE_SPOUSES"));
    for (owner, share, rental_income) in [
        (Owner::Taxpayer, CommunityShare::Community, 15_000),
        (Owner::Spouse, CommunityShare::Community, 15_000),
        (Owner::Taxpayer, CommunityShare::Partly { percent: 30 }, 25_500),
    ] {
        let r = community_item_agi(rented.clone(), owner, share);
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
        assert_eq!(value(&r, "agi"), 15_000 + rental_income);
    }
}

fn unit21_base(
    state: CommunityState,
    rule: CommunityRule,
    wages: i64,
    withholding: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes { state, rule };
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, withholding * 100)];
    d.income[0].community = CommunityShare::Community;
    d.next_id = 2;
    d
}
fn unit21_add(
    d: &mut FederalDraft,
    owner: Owner,
    share: CommunityShare,
    source: FederalIncome,
) -> i64 {
    let mut entry = income(source);
    entry.owner = crate::DocumentOwner::from(owner);
    entry.community = share;
    let id = d.next_id;
    add(d, entry);
    id
}
#[test]
fn unit21_f1_rental_loss_and_passive_history_are_halved_before_the_allowance() {
    let mut d = unit21_base(CommunityState::Arizona, CommunityRule::Halves, 60_000, 6_000);
    let id = unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        wage(0, Owner::Spouse, 4_000_000, 400_000).income,
    );
    assert_eq!(id, 2);
    let mut v = rental(24_000, 18_000);
    v.history = RentalHistory { regular: 600_000, amt: 600_000, active_in_loss_year: true };
    let id = unit21_add(
        &mut d,
        Owner::Taxpayer,
        CommunityShare::Community,
        FederalIncome::Rental { data: v },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50_000);
    assert_eq!(value(&r, &format!("rental_{id}_suspended_loss")), 5_000);
    assert_eq!(r.tax, Some(382_300));
    assert_eq!(r.refund, Some(117_700));
    assert_eq!(value_cents(&r, &format!("form8958_{id}_total")), -400_000);
    assert_eq!(value_cents(&r, &format!("form8958_{id}_this_return")), -200_000);
    let mut separate = d.clone();
    separate.income.last_mut().unwrap().community = CommunityShare::Separate;
    let own = supported(&separate);
    assert_eq!(value(&own, "agi"), 50_000);
    assert_eq!(value(&own, &format!("rental_{id}_suspended_loss")), 10_000);
    assert_eq!(value_cents(&own, &format!("form8958_{id}_this_return")), -400_000);
    d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
        state: CommunityState::Arizona,
        rule: CommunityRule::LivingApartAllYear,
    };
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 55_000);
    assert_eq!(r.tax, Some(442_300));
    assert_eq!(r.refund, Some(157_700));
}
#[test]
fn unit21_f2_partner_basis_loss_has_this_returns_share_and_no_spouse_se() {
    let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
    let mut data = ordinary_partner_source(-30_000, true);
    data.limited_for_se = Some(false);
    data.reported_se = Some(-3_000_000);
    data.regular.basis.opening = Some(2_000_000);
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 2_000_000, profit: 0, loss: 0 };
    data.alternative = Some(data.regular.clone());
    data.qbi = Some(partner_qbi_statement(-30_000));
    let id = unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        FederalIncome::Partnership { data },
    );
    // An unresolved negative box14 source is refused, including on the
    // nonpartner's return.
    let refused = estimate(&d);
    assert_eq!(refused.state, ResultState::Unsupported);
    assert!(
        refused
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_COMMUNITY_COUNTERPART_PARTNER_SE_UNRESOLVED")
    );
    assert!(refused.tax.is_none() && refused.return_lines.is_empty());
    // Independent financial allocation control: an actual zero box14 source
    // has no counterpart SE uncertainty; all original income/basis/QBI facts
    // and withholding remain unchanged.
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.reported_se = Some(0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 20_000);
    assert_eq!(r.tax, Some(39_300));
    assert_eq!(r.refund, Some(260_700));
    assert_eq!(value_cents(&r, &format!("form8958_{id}_total")), -3_000_000);
    assert_eq!(value_cents(&r, &format!("form8958_{id}_this_return")), -1_500_000);
}
#[test]
fn unit21_f3_scorp_spouses_participation_and_7203_basis_are_allocated() {
    let mut d = unit21_base(CommunityState::Washington, CommunityRule::Halves, 50_000, 5_000);
    let mut data = complete_ordinary_scorp(40_000, true);
    data.loans.clear(); // The original F3 states that there are no loans.
    data.opening = Some(500_000);
    data.distributions = 3_000_000;
    data.qbi_statement.as_mut().unwrap().wages = 2_000_000;
    data.amt = Some(amt_records(&data));
    unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        FederalIncome::Scorp { data: scorp_source(data) },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 45_000);
    assert_eq!(value(&r, "qbi_deduction"), 4_000);
    assert_eq!(r.tax, Some(274_300));
    assert_eq!(r.balance, Some(24_300));
}
#[test]
fn unit21_f4_separate_property_income_differs_from_its_property_gain() {
    for (state, rule, agi, tax, refund) in [
        (CommunityState::Texas, CommunityRule::Halves, 36_000, 214_300, 85_700),
        (CommunityState::Arizona, CommunityRule::Halves, 30_000, 142_300, 157_700),
        (CommunityState::Texas, CommunityRule::LivingApartAllYear, 60_000, 502_300, 97_700),
    ] {
        let mut d = unit21_base(state, rule, 60_000, 6_000);
        unit21_add(
            &mut d,
            Owner::Spouse,
            CommunityShare::Separate,
            FederalIncome::Rental { data: rental(30_000, 8_000) },
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(r.refund, Some(refund));
    }
}

#[test]
fn unit21_special_rule_keeps_distributive_shares_whole_but_divides_external_sale_gain() {
    for rule in [CommunityRule::LivingApartAllYear, CommunityRule::NonresidentSpouse] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut d = unit21_base(CommunityState::Arizona, rule, 60_000, 6_000);
            let mut source = ordinary_partner_source(40_000, true);
            source.regular.basis.opening = Some(6_000_000);
            source.regular.risk.start =
                ScorpRiskStart::Detailed { capacity: 6_000_000, profit: 0, loss: 0 };
            source.alternative = Some(source.regular.clone());
            source.disposition = Some(K1Disposition::Sold { proceeds: 15_000_000 });
            source.interest_term = Some(GainTerm::Long);
            let id = unit21_add(
                &mut d,
                owner,
                CommunityShare::Community,
                FederalIncome::Partnership { data: source },
            );
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), if owner == Owner::Taxpayer { 125_000 } else { 85_000 });
            assert_eq!(value_cents(&r, &format!("form8958_gain_{id}_total")), 5_000_000);
            assert_eq!(value_cents(&r, &format!("form8958_gain_{id}_this_return")), 2_500_000);
        }
    }
}
#[test]
fn unit21_positive_partner_se_is_whole_and_the_nonpartner_has_none() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
        let mut source = ordinary_partner_source(30_000, true);
        source.limited_for_se = Some(false);
        source.reported_se = Some(3_000_000);
        source.se_gross = Some(PartnerSeGross::Nonfarm { gross: 3_000_000 });
        source.qbi = Some(partner_qbi_statement(30_000));
        source.qbi_records.qualified_se_gross = 3_000_000;
        unit21_add(
            &mut d,
            owner,
            CommunityShare::Community,
            FederalIncome::Partnership { data: source },
        );
        let r = supported(&d);
        // 30000 × .9235 =27705; ×.153 =4238.865 =>4239; half2120.
        assert_eq!(
            value(&r, "self_employment_tax"),
            if owner == Owner::Taxpayer { 4_239 } else { 0 }
        );
        assert_eq!(value(&r, "agi"), if owner == Owner::Taxpayer { 42_880 } else { 45_000 });
        assert_eq!(
            value(&r, "qbi_deduction"),
            if owner == Owner::Taxpayer { 2_576 } else { 3_000 }
        );
    }
}
#[test]
fn unit21_divided_limited_negative_se_keeps_the_named_refusal() {
    let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
    let mut source = ordinary_partner_source(-30_000, true);
    source.limited_for_se = Some(false);
    source.reported_se = Some(-3_000_000);
    source.regular.basis.opening = Some(2_000_000);
    source.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 2_000_000, profit: 0, loss: 0 };
    source.alternative = Some(source.regular.clone());
    unit21_add(
        &mut d,
        Owner::Taxpayer,
        CommunityShare::Community,
        FederalIncome::Partnership { data: source },
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|issue| issue.display.key
            == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"),
        "{r:?}"
    );
}

#[test]
fn unit21_one_cent_source_credit_does_not_invalidate_a_zero_allocated_share() {
    let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
    let mut data = complete_ordinary_scorp(40_000, true);
    data.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 1,
    });
    data.amt = Some(amt_records(&data));
    assert!(data.valid());
    unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        FederalIncome::Scorp { data: scorp_source(data) },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50_000);
}

#[test]
fn unit21_counterpart_se_form8958_requires_the_partners_resolved_schedule_se() {
    let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
    for (ordinary, opening) in [(40_000, 0), (-30_000, 20_000)] {
        let mut source = ordinary_partner_source(ordinary, true);
        source.limited_for_se = Some(false);
        source.reported_se = Some(ordinary * 100);
        source.se_gross = Some(PartnerSeGross::Nonfarm { gross: ordinary.max(0) * 100 });
        source.regular.basis.opening = Some(opening * 100);
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: opening * 100, profit: 0, loss: 0 };
        source.alternative = Some(source.regular.clone());
        unit21_add(
            &mut d,
            Owner::Spouse,
            CommunityShare::Community,
            FederalIncome::Partnership { data: source },
        );
    }
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_COMMUNITY_COUNTERPART_PARTNER_SE_UNRESOLVED")
    );
    assert!(r.tax.is_none() && r.return_lines.is_empty());
    // The partner's own ScheduleSE remains intentionally unresolved. Its
    // unknown tax cannot be manufactured for the other return's Form8958.
    for entry in d.income.iter_mut().skip(1) {
        entry.owner = crate::DocumentOwner::Yours;
    }
    let partner = estimate(&d);
    assert_eq!(partner.state, ResultState::Unsupported);
    assert!(
        partner.issues.iter().any(|issue| issue.display.key
            == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"),
        "{partner:?}"
    );
}

#[test]
fn unit21_form8958_partner_se_combines_sources_and_the_actual_partners_wage_base() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
        d.income[0].owner = if owner == Owner::Taxpayer {
            crate::DocumentOwner::Spouses
        } else {
            crate::DocumentOwner::Yours
        };
        unit21_add(&mut d, owner, CommunityShare::Community, wage(0, owner, 18_000_000, 0).income);
        let mut ids = Vec::new();
        for ordinary in [30_000, 20_000] {
            let mut source = ordinary_partner_source(ordinary, true);
            source.limited_for_se = Some(false);
            source.reported_se = Some(ordinary * 100);
            source.se_gross = Some(PartnerSeGross::Nonfarm { gross: ordinary * 100 });
            ids.push(unit21_add(
                &mut d,
                owner,
                CommunityShare::Community,
                FederalIncome::Partnership { data: source },
            ));
        }
        let r = supported(&d);
        // Annual net50,000*.9235=46,175. Remaining SS base4,500:
        // tax4,500*.124+46,175*.029=1,897.075=>1,897; half949.
        assert_eq!(
            value(&r, "self_employment_tax"),
            if owner == Owner::Taxpayer { 1_897 } else { 0 }
        );
        let mut tax = 0;
        let mut half = 0;
        for (index, id) in ids.into_iter().enumerate() {
            let allocated_tax = if index == 0 { 113_820 } else { 75_880 };
            let allocated_half = if index == 0 { 56_940 } else { 37_960 };
            assert_eq!(value_cents(&r, &format!("form8958_se_tax_{id}_total")), allocated_tax);
            assert_eq!(
                value_cents(&r, &format!("form8958_se_deduction_{id}_total")),
                allocated_half
            );
            assert_eq!(
                value_cents(&r, &format!("form8958_se_tax_{id}_this_return")),
                if owner == Owner::Taxpayer { allocated_tax } else { 0 }
            );
            assert_eq!(
                value_cents(&r, &format!("form8958_se_deduction_{id}_this_return")),
                if owner == Owner::Taxpayer { allocated_half } else { 0 }
            );
            tax += allocated_tax;
            half += allocated_half;
        }
        assert_eq!(tax, 189_700);
        assert_eq!(half, 94_900);
        crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
    }
}

#[test]
fn unit21_form8958_partner_se_with_church_income_agrees_on_both_returns() {
    // Church20,000*.9235=18,470 and partner30,000*.9235=27,705;
    // annual net46,175. Without ordinary wages tax7,065 and half3,533;
    // the nonbusiness2/5 share removes tax2,826 and half1,413. With
    // SS wages180,000, annual tax1,897/half949 removes rounded759/380.
    // Source attribution retains these rounded annual amounts, independently
    // for tax and deduction, before the existing per-entity allocator.
    for (wages, annual_tax, source_tax, source_half) in
        [(0, 7_065, 423_900, 212_000), (180_000, 1_897, 113_800, 56_900)]
    {
        let mut totals = Vec::new();
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 0, 0);
            if wages != 0 {
                unit21_add(
                    &mut d,
                    owner,
                    CommunityShare::Community,
                    wage(0, owner, wages * 100, 0).income,
                );
            }
            let mut church = wage(0, owner, 2_000_000, 0).income;
            let FederalIncome::Wage { data } = &mut church else { unreachable!() };
            edit_wage(data).coverage = W2Coverage::ChurchEmployee;
            edit_wage(data).social_wages = 0;
            edit_wage(data).social_tax = 0;
            edit_wage(data).medicare_wages = 0;
            edit_wage(data).medicare_tax = 0;
            unit21_add(&mut d, owner, CommunityShare::Community, church);
            let mut source = ordinary_partner_source(30_000, true);
            source.limited_for_se = Some(false);
            source.reported_se = Some(3_000_000);
            source.se_gross = Some(PartnerSeGross::Nonfarm { gross: 3_000_000 });
            let id = unit21_add(
                &mut d,
                owner,
                CommunityShare::Community,
                FederalIncome::Partnership { data: source },
            );
            let r = supported(&d);
            assert_eq!(
                value(&r, "self_employment_tax"),
                if owner == Owner::Taxpayer { annual_tax } else { 0 }
            );
            let total = (
                value_cents(&r, &format!("form8958_se_tax_{id}_total")),
                value_cents(&r, &format!("form8958_se_deduction_{id}_total")),
            );
            assert_eq!(total, (source_tax, source_half));
            for (kind, amount) in [("se_tax", source_tax), ("se_deduction", source_half)] {
                assert_eq!(
                    value_cents(&r, &format!("form8958_{kind}_{id}_this_return")),
                    if owner == Owner::Taxpayer { amount } else { 0 }
                );
            }
            crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
            totals.push(total);
        }
        assert_eq!(totals[0], totals[1]);
    }
}

#[test]
fn unit21_k1_portfolio_credits_and_source_columns_conserve_both_returns() {
    let mut d = unit21_base(CommunityState::Washington, CommunityRule::Halves, 60_000, 6_000);
    let mut data = complete_ordinary_scorp(10_000, true);
    data.interest = 200_000;
    data.dividends = 400_000;
    data.qualified_dividends = 100_000;
    data.royalties = 1_000_000;
    data.withholding = 50_000;
    data.deductions
        .push(ScorpDeduction { code: ScorpDeductionCode::RoyaltyDeductions, amount: 200_000 });
    data.credits.push(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 200_000,
    });
    data.amt = Some(amt_records(&data));
    let id = unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        FederalIncome::Scorp { data: scorp_source(data) },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 42_000);
    for (kind, full, own) in [
        ("interest", 200_000, 100_000),
        ("dividends", 400_000, 200_000),
        ("royalties", 800_000, 400_000),
        ("credit", 200_000, 100_000),
        ("withholding", 50_000, 25_000),
    ] {
        assert_eq!(value_cents(&r, &format!("form8958_{kind}_{id}_total")), full);
        assert_eq!(value_cents(&r, &format!("form8958_{kind}_{id}_this_return")), own);
    }
    assert_eq!(value_cents(&r, &format!("form8958_{id}_total")), 1_000_000);
    assert_eq!(value_cents(&r, &format!("form8958_{id}_this_return")), 500_000);
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
}

#[test]
fn unit21_partner_and_shareholder_basis_risk_ledgers_are_linear_in_the_share() {
    for scorp in [false, true] {
        let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
        let whole = if scorp {
            let mut data = complete_ordinary_scorp(-30_000, true);
            data.loans.clear();
            data.opening = Some(2_000_000);
            data.risk.start = ScorpRiskStart::Detailed { capacity: 1_500_000, profit: 0, loss: 0 };
            data.amt = Some(amt_records(&data));
            FederalIncome::Scorp { data: scorp_source(data) }
        } else {
            let mut data = ordinary_partner_source(-30_000, true);
            data.regular.basis.opening = Some(2_000_000);
            data.regular.risk.start =
                ScorpRiskStart::Detailed { capacity: 1_500_000, profit: 0, loss: 0 };
            data.alternative = Some(data.regular.clone());
            FederalIncome::Partnership { data }
        };
        let id = unit21_add(&mut d, Owner::Spouse, CommunityShare::Community, whole.clone());
        let ledger = |income: &FederalIncome| match income {
            FederalIncome::Scorp { data } => {
                ScorpK1::from(data).pool_activity_result().unwrap().ledger
            }
            FederalIncome::Partnership { data } => {
                data.pool(&data.regular).unwrap().activity.ledger
            }
            _ => unreachable!(),
        };
        let full = ledger(&whole);
        let divided =
            crate::community::record(&d, d.income.iter().find(|entry| entry.id == id).unwrap());
        let allocated = ledger(&divided);
        use crate::k1_loss::K1LossClass::Ordinary;
        assert_eq!(crate::kernel::ratio::dollars(full.basis_allowed[Ordinary].get()), 20_000);
        assert_eq!(crate::kernel::ratio::dollars(full.basis_suspended[Ordinary].get()), 10_000);
        assert_eq!(crate::kernel::ratio::dollars(full.risk_allowed[Ordinary].get()), 15_000);
        assert_eq!(crate::kernel::ratio::dollars(full.risk_suspended[Ordinary].get()), 5_000);
        assert_eq!(crate::kernel::ratio::dollars(allocated.basis_allowed[Ordinary].get()), 10_000);
        assert_eq!(crate::kernel::ratio::dollars(allocated.basis_suspended[Ordinary].get()), 5_000);
        assert_eq!(crate::kernel::ratio::dollars(allocated.risk_allowed[Ordinary].get()), 7_500);
        assert_eq!(crate::kernel::ratio::dollars(allocated.risk_suspended[Ordinary].get()), 2_500);
        let r = supported(&d);
        assert!(noted(&r, "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE"));
        assert_eq!(value(&r, "agi"), 22_500);
        crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
    }
}

#[test]
fn unit21_counterpart_negative_se_refusal_requires_an_actual_binding_limit() {
    for basis in [30_000, 29_998] {
        let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
        let mut data = ordinary_partner_source(-30_000, true);
        data.limited_for_se = Some(false);
        data.reported_se = Some(-3_000_000);
        data.regular.basis.opening = Some(basis * 100);
        data.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 3_000_000, profit: 0, loss: 0 };
        data.alternative = Some(data.regular.clone());
        let id = unit21_add(
            &mut d,
            Owner::Spouse,
            CommunityShare::Community,
            FederalIncome::Partnership { data },
        );
        if basis == 30_000 {
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 15_000);
            assert_eq!(value_cents(&r, &format!("form8958_se_tax_{id}_total")), 0);
            assert_eq!(r.refund, Some(300_000));
        } else {
            let r = estimate(&d);
            assert_eq!(r.state, ResultState::Unsupported);
            assert!(
                r.issues.iter().any(|issue| issue.display.key
                    == "ISSUE_COMMUNITY_COUNTERPART_PARTNER_SE_UNRESOLVED")
            );
        }
    }
}

#[test]
fn unit21_section1256_k1_rows_reach_the_shared_form_at_the_allocated_amount() {
    for partnership in [false, true] {
        let mut d = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
        let source = if partnership {
            let mut data = ordinary_partner_source(0, true);
            data.income.push(PartnerIncomeTotal {
                code: PartnerIncomeCode::Section1256,
                amount: 2_000_000,
            });
            FederalIncome::Partnership { data }
        } else {
            let mut data = complete_ordinary_scorp(0, true);
            data.income.push(ScorpIncome { code: ScorpIncomeCode::Section1256, amount: 2_000_000 });
            data.amt = Some(amt_records(&data));
            FederalIncome::Scorp { data: scorp_source(data) }
        };
        let id = unit21_add(&mut d, Owner::Spouse, CommunityShare::Community, source);
        let r = supported(&d);
        assert_eq!(crate::section1256::form_6781(&d).net, 10_000);
        assert_eq!(value(&r, "agi"), 40_000);
        assert_eq!(return_line(&r, "7a"), 1_000_000);
        assert_eq!(value_cents(&r, &format!("form8958_gain_{id}_total")), 2_000_000);
        assert_eq!(value_cents(&r, &format!("form8958_gain_{id}_this_return")), 1_000_000);
        crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
    }
}

#[test]
fn unit21_estate_final_capital_history_is_allocated_before_the_mfs_loss_limit() {
    let seed = estate_trust_code_c_draft(0, 0);
    let mut data = match seed.income[1].income.clone() {
        FederalIncome::EstateTrustPortfolio { data } => FederalEstateTrustPortfolioK1::from(&data),
        _ => unreachable!(),
    };
    data.final_short_carry = 1_000_000;
    data.final_long_carry = 1_000_000;
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    data.final_carry_same_tax_character = true;
    // Community property gives each return $5,000 short + $5,000 long
    // before its own $1,500 cap. Texas separate-property income shares do
    // not transfer the other spouse's property gains or capital losses.
    for (owner, community, deduction, short, long, agi) in [
        (Owner::Spouse, CommunityShare::Community, -150_000, 3_500, 5_000, 28_500),
        (Owner::Taxpayer, CommunityShare::Separate, -150_000, 8_500, 10_000, 28_500),
        (Owner::Spouse, CommunityShare::Separate, 0, 0, 0, 30_000),
    ] {
        let mut d = unit21_base(CommunityState::Texas, CommunityRule::Halves, 60_000, 6_000);
        unit21_add(
            &mut d,
            owner,
            community,
            FederalIncome::EstateTrustPortfolio { data: estate_source(data.clone()) },
        );
        let before = d.clone();
        let r = supported(&d);
        assert_eq!(return_line(&r, "7a"), deduction);
        assert_eq!(value(&r, "capital_short_carryforward"), short);
        assert_eq!(value(&r, "capital_long_carryforward"), long);
        assert_eq!(value(&r, "amt_capital_short_carryforward"), short);
        assert_eq!(value(&r, "amt_capital_long_carryforward"), long);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(d, before);
    }
}

#[test]
fn unit21_qualified_loss_carry_note_uses_the_final_qbi_owner() {
    for offset in [false, true] {
        let mut d = unit21_base(CommunityState::Washington, CommunityRule::Halves, 60_000, 6_000);
        let mut loss = complete_ordinary_scorp(-1_000, true);
        loss.loss_qbi.current = 100_000;
        loss.amt = Some(amt_records(&loss));
        unit21_add(
            &mut d,
            Owner::Spouse,
            CommunityShare::Community,
            FederalIncome::Scorp { data: scorp_source(loss) },
        );
        if offset {
            unit21_add(
                &mut d,
                Owner::Spouse,
                CommunityShare::Community,
                FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(2_000, true)) },
            );
        }
        let r = supported(&d);
        assert_eq!(value(&r, "qbi_loss_carryforward"), if offset { 0 } else { 500 });
        assert_eq!(noted(&r, "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE"), !offset);
    }
}

#[test]
fn unit21_estate_source_credit_columns_precede_the_returns_credit_cap() {
    let seed = estate_trust_code_c_draft(0, 0);
    let mut data = match seed.income[1].income.clone() {
        FederalIncome::EstateTrustPortfolio { data } => FederalEstateTrustPortfolioK1::from(&data),
        _ => unreachable!(),
    };
    data.activities.activities.push(EstateActivity {
        id: 3,
        name: "Synthetic estate business".into(),
        kind: EstateActivityKind::Business,
        income: 0,
        depreciation: 0,
        depletion: 0,
        amortization: 0,
        participation: EstateParticipation::Material(ScorpParticipation {
            minutes: 30_001,
            ..participation_facts()
        }),
        carried: EstatePassiveCarry { regular: 0, alternative: 0 },
        amt_difference: 0,
    });
    data.reported_credits.push(EstateReportedCredit {
        code: EstateCreditCode::F,
        amount: 200_000,
        activities: vec![3],
    });
    data.investment_property.push(EstateInvestmentProperty {
        property: K1InvestmentProperty::Energy {
            id: 0,
            own_form: None,
            basis: 1_000_000,
            rate: EnergyRate {
                increased_rate: true,
                domestic_content: false,
                energy_community: false,
            },
        },
        activities: vec![3],
    });
    let mut d = unit21_base(CommunityState::Texas, CommunityRule::Halves, 60_000, 6_000);
    let id = unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Separate,
        FederalIncome::EstateTrustPortfolio { data: estate_source(data) },
    );
    d.next_id = d.next_id.max(4);
    let r = supported(&d);
    assert_eq!(value_cents(&r, &format!("form8958_credit_{id}_total")), 500_000);
    assert_eq!(value_cents(&r, &format!("form8958_credit_{id}_this_return")), 250_000);
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
    // An owner keeps the odd cent. A valid one-cent source claim therefore
    // allocates zero to this nonowner; its zero transient row is no new claim.
    let FederalIncome::EstateTrustPortfolio { data } = &mut d.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    {
        let mut data = edit_estate(data);
        data.reported_credits[0].amount = 1;
        let K1InvestmentProperty::Energy { basis, .. } = &mut data.investment_property[0].property
        else {
            unreachable!()
        };
        *basis = 1;
    }
    let zero = supported(&d);
    assert_eq!(value(&zero, "agi"), 30_000);
    assert_eq!(value_cents(&zero, &format!("form8958_credit_{id}_total")), 1);
    assert_eq!(value_cents(&zero, &format!("form8958_credit_{id}_this_return")), 0);
    let FederalIncome::EstateTrustPortfolio { data } = &d.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(FederalEstateTrustPortfolioK1::from(data).reported_credits[0].amount, 1);
}

#[test]
fn unit21_f3_scorp_loss_contrast_has_only_the_stated_stock_basis() {
    let mut d = unit21_base(CommunityState::Washington, CommunityRule::Halves, 50_000, 5_000);
    let mut source = complete_ordinary_scorp(-10_000, true);
    source.opening = Some(500_000);
    source.loans.clear();
    source.distributions = 0;
    source.loss_qbi.current = 1_000_000;
    source.amt = Some(amt_records(&source));
    let id = unit21_add(
        &mut d,
        Owner::Spouse,
        CommunityShare::Community,
        FederalIncome::Scorp { data: scorp_source(source) },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 22_500);
    let allocated =
        crate::community::record(&d, d.income.iter().find(|entry| entry.id == id).unwrap());
    let FederalIncome::Scorp { data } = allocated.as_ref() else { unreachable!() };
    let ledger = ScorpK1::from(data).ledger().unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            ledger.basis_allowed[crate::k1_loss::K1LossClass::Ordinary].get()
        ),
        2_500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            ledger.basis_suspended[crate::k1_loss::K1LossClass::Ordinary].get()
        ),
        2_500
    );
    assert!(noted(&r, "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE"));
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(&d, &r);
}

#[test]
fn partial_partner_fuel_recovery_projects_the_source_taxable_amount_before_community_shares() {
    // Owner keeps the odd source cent; the counterpart receives the residual.
    // Both project $718.50 of recovery, rather than halving a $5,000 credit
    // and reusing the unsplit worksheet against each half.
    for qualified in [true, false] {
        let mut projected = 0;
        for (owner, expected) in [(Owner::Taxpayer, 35_925), (Owner::Spouse, 35_925)] {
            let mut d =
                unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
            let mut partner = ordinary_partner_source(20_000, true);
            partner.qbi = Some(partner_qbi_statement(20_000));
            let mut prior = partial_partner_fuel_recovery();
            let partial = prior.partial_prior_year_benefit.as_mut().unwrap();
            partial.refigured_prior_year_line_11b = partial.refigured_prior_year_line_14 + 71_850;
            partial.comparison_complete = qualified;
            partner.prior_fuel_credit = Some(prior.clone());
            let id = unit21_add(
                &mut d,
                owner,
                CommunityShare::Community,
                FederalIncome::Partnership { data: partner },
            );
            let before = d.clone();
            let report = supported(&d);
            let actual =
                line_amount(&report, &format!("partnership_{id}_prior_fuel_credit_recovery"));
            assert_eq!(actual, if qualified { expected } else { 250_000 });
            projected += actual;
            assert_eq!(noted(&report, "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT"), !qualified);
            assert_eq!(d, before);
            assert_eq!(restart(d.clone()).draft, before);
            let FederalIncome::Partnership { data } = &d.income[1].income else { unreachable!() };
            assert_eq!(data.prior_fuel_credit, Some(prior));
        }
        assert_eq!(projected, if qualified { 71_850 } else { 500_000 });
    }
}

#[test]
fn partial_partner_fuel_recovery_keeps_a_whole_source_and_its_worksheet_unchanged() {
    let draft = unit21_base(CommunityState::California, CommunityRule::Halves, 60_000, 6_000);
    let income = FederalIncome::Partnership { data: ordinary_partner_source(20_000, true) };
    let share = crate::community::share(
        draft.setup.as_ref().unwrap(),
        Owner::Taxpayer,
        CommunityShare::Separate,
        &income,
    )
    .unwrap();
    assert!(share.whole());
    let prior = partial_partner_fuel_recovery();
    assert_eq!(crate::community::Divide::divided(&prior, share), prior);
    assert_eq!(prior.taxable_amount(), 71_800);
}

#[test]
fn community_business_owner_boundary_preserves_native_cent_projection() {
    // The community owner projects99c×1/2=49.5c once
    // half-up to50c at its owner boundary. The reported category is then$1.
    let mut draft = community_w2_draft(3_000_000, 0, 0, 0);
    let mut business = cash_business(0);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts = 99;
    add(&mut draft, income(business));
    let result = estimate(&draft);
    assert_eq!(value(&result, "schedule_1_business_income"), 1);
}

#[test]
fn community_business_parcel_keeps_the_existing_five_thousand_dollar_control() {
    // Reg1.175-5 and Pub555: an Arizona separate return reports its half
    // of the original farm property's independently established $5,000 gain.
    let mut draft = conservation_return();
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Community;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.conservation = None;
    let mut sale = farm_land_sale(20220101, 20261201);
    sale.proceeds = 11_500_000;
    assert_eq!((sale.calculate().gain, sale.calculate().ordinary), (5_000, 5_000));
    set_land_sales(farm, vec![sale], &mut draft.next_id);
    let result = supported(&draft);
    assert_eq!(value_cents(&result, "farm_2_land_0_ordinary"), 250_000);
    assert_eq!(value_cents(&result, "form8958_gain_2_total"), 500_000);
    assert_eq!(value_cents(&result, "form8958_gain_2_this_return"), 250_000);
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(
        &draft, &result,
    );
}

#[test]
fn community_original_property_parts_conserve_a_partly_shared_cent() {
    let mut draft = farmer_gross_return(0, 30_000);
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Partly { percent: 1 };
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets = vec![farmer_gross_asset(1, Livestock::No), farmer_gross_asset(1, Livestock::No)];
    data.assets[0].id = 1001;
    data.assets[1].id = 1002;
    draft.next_id = 1003;
    let claims = crate::related_forms::effective_claimants(&draft);
    for (owner, total, parts) in
        [(Owner::Taxpayer, 199, vec![100, 99]), (Owner::Spouse, 1, vec![0, 1])]
    {
        draft.income[1].owner = crate::DocumentOwner::from(owner);
        let pool =
            crate::extended_rules::RentalResult { inputs_complete: true, ..Default::default() };
        let projection =
            crate::community::source_projection(&draft, &pool, false, &claims).unwrap();
        assert_eq!(
            projection.direct.iter().find(|source| source.income == 2).unwrap().filer.gain,
            total
        );
        let components = projection.components[&2].as_ref().unwrap();
        assert_eq!(components.iter().map(|row| row.filer.gain).collect::<Vec<_>>(), parts);
        assert!(
            components.iter().all(
                |row| row.whole.unwrap().gain == row.filer.gain + row.counterpart.unwrap().gain
            )
        );
        let report = supported(&draft);
        assert_eq!(
            line_amount(&report, "business_2_asset_0_disposal_gain")
                + line_amount(&report, "business_2_asset_1_disposal_gain"),
            total
        );
    }
}

#[test]
fn community_source_income_keeps_signed_odd_cent_counterparts() {
    let mut draft = community_w2_draft(3_000_000, 0, 0, 0);
    let mut source = income(cash_business(0));
    source.community = CommunityShare::Community;
    add(&mut draft, source);
    for whole in [99, -99] {
        draft.income[1].owner = crate::DocumentOwner::Yours;
        let owner = crate::community::SourceAmount::income(&draft, 2, whole);
        draft.income[1].owner = crate::DocumentOwner::Spouses;
        let other = crate::community::SourceAmount::income(&draft, 2, whole);
        assert_eq!(owner.filer, if whole > 0 { 50 } else { -50 });
        assert_eq!(other.filer, if whole > 0 { 49 } else { -49 });
        assert_eq!(owner.filer + other.filer, whole);
    }
}

#[test]
fn community_mixed_property_characters_partition_the_recognized_cent() {
    let mut draft = farmer_gross_return(0, 30_000);
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Partly { percent: 1 };
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    // $1,000 of recovered depreciation, $1 of section1231 gain, plus
    // another $1 ordinary gain. At99.5%, the aggregate keeps the residual cent.
    data.assets =
        vec![farmer_gross_asset(1, Livestock::No), farmer_gross_asset(1_001, Livestock::No)];
    data.assets[0].id = 1001;
    data.assets[1].id = 1002;
    draft.next_id = 1003;
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "form8958_gain_2_this_return"), 99_699);
    assert_eq!(line_amount(&result, "form8958_gain_2_ordinary_this_return"), 99_600);
    assert_eq!(line_amount(&result, "form8958_gain_2_section1231_this_return"), 99);
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(
        &draft, &result,
    );
}

#[test]
fn community_conservation_current_cap_keeps_the_entered_tax_carry() {
    let mut draft = conservation_return();
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Community;
    let claims = crate::related_forms::effective_claimants(&draft);
    // $120,000 physical gross establishes the operator's $30,000 current cap.
    // The filer has $60,000 gross and a $15,000 current cap. Current $40,000
    // costs alone fill both; the original $5,000 tax carry stays undivided.
    assert_eq!(
        crate::farm_conservation::operator_deduction_with_claims(&draft, 2, false, &claims),
        Some(3_000_000)
    );
    assert_eq!(
        crate::farm_conservation::income_deduction_with_claims(&draft, 2, false, &claims),
        Some(1_500_000)
    );
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "farm_2_conservation_carryforward"), 1_000_000);
    // The other return's current amount is established, but its historical
    // tax balance and closing carry are not supplied by this return.
    draft.income[1].owner = crate::DocumentOwner::Spouses;
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "farm_2_conservation_deduction"), 1_500_000);
    assert!(result.lines.iter().all(|row| row.id != "farm_2_conservation_carryforward"));
    assert!(noted(&result, "WARN_FARM_CONSERVATION_CARRY_SOURCE"));
}

#[test]
fn community_current_only_conservation_uses_the_same_operator_pool_across_farms() {
    let mut draft = conservation_return();
    draft.setup = complete(FederalStatus::Separate).setup;
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Community;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { carry, .. }) = &mut farm.conservation else {
        unreachable!()
    };
    carry.clear();
    let mut second = draft.income[1].clone();
    second.id = 1001;
    draft.income.push(second);
    let claims = crate::related_forms::effective_claimants(&draft);
    for id in [2, 1001] {
        assert_eq!(
            crate::farm_conservation::operator_deduction_with_claims(&draft, id, false, &claims),
            Some(3_000_000)
        );
        assert_eq!(
            crate::farm_conservation::income_deduction_with_claims(&draft, id, false, &claims),
            Some(1_500_000)
        );
    }
}

#[test]
fn community_unrecaptured_building_gain_stays_within_each_spouses_1231_interval() {
    let mut draft = farmer_gross_return(0, 30_000);
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income[0].community = CommunityShare::Community;
    draft.income[1].community = CommunityShare::Partly { percent: 1 };
    let building = crate::tests::business::business::shop_building(
        crate::tests::business::business::sold(20260620, 28_582_400, 0),
    );
    let native = building.sale_in_context(false, false, Default::default()).unwrap();
    // Original $390,000 building less $104,177 native depreciation gives
    // $285,823 adjusted basis; $285,824 proceeds produces $1 high-rate gain.
    assert_eq!(
        (native.basis, native.depreciation, native.gain, native.section1231, native.unrecaptured),
        (390_000, 104_177, 1, 1, 1)
    );
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets = vec![farmer_gross_asset(1, Livestock::No), building];
    data.assets[0].id = 1001;
    data.assets[1].id = 1002;
    draft.next_id = 1003;
    let claims = crate::related_forms::effective_claimants(&draft);
    for (owner, expected) in [(Owner::Taxpayer, [199, 100, 99, 99]), (Owner::Spouse, [1, 0, 1, 1])]
    {
        draft.income[1].owner = crate::DocumentOwner::from(owner);
        let pool = crate::extended_rules::RentalResult::default();
        let projection =
            crate::community::source_projection(&draft, &pool, false, &claims).unwrap();
        let source = projection.direct.iter().find(|row| row.income == 2).unwrap();
        assert_eq!(
            [
                source.filer.gain,
                source.filer.ordinary,
                source.filer.section1231,
                source.filer.unrecaptured
            ],
            expected
        );
        let parts = projection.components[&2].as_ref().unwrap();
        assert_eq!(
            parts.iter().map(|row| row.filer.unrecaptured).sum::<i64>(),
            source.filer.unrecaptured
        );
        for part in parts {
            let other = part.counterpart.unwrap();
            assert!(part.filer.unrecaptured <= part.filer.section1231.max(0));
            assert!(other.unrecaptured <= other.section1231.max(0));
            assert_eq!(
                part.filer.unrecaptured + other.unrecaptured,
                part.whole.unwrap().unrecaptured
            );
        }
        supported(&draft);
    }
}
