// Actual Save/Estimate controls for remaining payer-statement relationships.
// Source-authored at602; execution and real-schema locator proof are separate.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;

struct InvestmentGap {
    case: &'static str,
    key: &'static str,
    field: &'static str,
    bad: IncomeDocument,
    corrected: IncomeDocument,
}

fn interest_gap_base() -> Int1099 {
    native_int1099("Actual interest source".into(), 0)
}
fn dividend_gap_base() -> Div1099 {
    native_div1099("Actual dividend source".into(), 0, 0)
}
fn oid_gap_base() -> Oid1099 {
    native_oid1099("Actual OID source".into(), 0, 0)
}

fn investment_gap_seed(corrected: &IncomeDocument) -> (Interview, P, FederalResult) {
    let mut draft = complete(FederalStatus::Single);
    // Authorize genuine final-return recipient parts before testing their
    // relationships; a missing death must not preempt the intended owner.
    draft.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20260630, ordinary: true });
    let mut session = Interview::new(TEST_USER, draft);
    let mut unrelated = interest_gap_base();
    unrelated.payer = "Unrelated accepted interest source".into();
    unrelated.taxable = 10_000;
    let request = save_topic(
        &session.draft,
        TaxSection::InterestDividends,
        vec![document(0, IncomeDocument::Int1099(unrelated)), document(0, corrected.clone())],
    );
    accepted(&send(&mut session, request));
    let baseline = estimate(&session.draft);
    let request = open_topic(&mut session, TaxSection::InterestDividends);
    assert_eq!(page_documents(&request)[1].document, *corrected);
    assert!(page_documents(&request)[1].id > 0);
    (session, request, baseline)
}

fn exercise_investment_gap(gap: InvestmentGap) {
    let (mut session, corrected, baseline) = investment_gap_seed(&gap.corrected);
    let mut invalid = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    documents.documents[1].document = gap.bad;
    let path = alloc::format!("[documents/documents/1/document/{}]", gap.field);
    let invalid_estimate = credit_estimate_documents_from_save(&invalid);
    credit_refusal(&alloc::format!("{}-save", gap.case), &mut session, invalid, gap.key, &path);
    credit_refusal(
        &alloc::format!("{}-estimate", gap.case),
        &mut session,
        invalid_estimate,
        gap.key,
        &path,
    );
    let accepted_draft = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(&mut session, credit_estimate_documents_from_save(&corrected)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, corrected));
    // The equal-boundary/source correction must preserve the full original
    // financial result, source identities, withholding and accepted records.
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(estimate(&session.draft), baseline);
    let reopened = open_topic(&mut session, TaxSection::InterestDividends);
    assert_eq!(page_documents(&reopened)[1].document, gap.corrected);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, accepted_draft);
    let mut cold = restart(accepted_draft.clone());
    let reopened = open_topic(&mut cold, TaxSection::InterestDividends);
    assert_eq!(page_documents(&reopened)[1].document, gap.corrected);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(estimate(&cold.draft), baseline);
}

fn interest_gap(index: usize) -> InvestmentGap {
    match index {
        0 => {
            let mut good = interest_gap_base();
            good.taxable = 100;
            good.nominee = Some(InterestNominee { taxable: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().taxable = 101;
            InvestmentGap {
                case: "int-outside-taxable",
                key: "INT_OUTSIDE_TAXABLE_ABOVE_BOX",
                field: "taxable",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        1 => {
            let mut good = interest_gap_base();
            good.more.us_obligations = 100;
            good.nominee = Some(InterestNominee { us_obligations: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().us_obligations = 101;
            InvestmentGap {
                case: "int-outside-us-obligations",
                key: "INT_OUTSIDE_US_OBLIGATIONS_ABOVE_BOX",
                field: "more/us_obligations",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        2 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.nominee = Some(InterestNominee { exempt: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "int-outside-exempt",
                key: "INT_OUTSIDE_EXEMPT_ABOVE_BOX",
                field: "more/exempt",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        3 => {
            let mut good = interest_gap_base();
            good.more.private_activity = 100;
            good.nominee = Some(InterestNominee { private_activity: 100, ..Default::default() });
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "int-outside-private-activity",
                key: "INT_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "more/private_activity",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        4 => {
            let mut good = interest_gap_base();
            good.more.foreign_tax = 100;
            good.nominee = Some(InterestNominee { foreign_tax: 100, ..Default::default() });
            good.more.foreign_country = "Canada".into();
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().foreign_tax = 101;
            InvestmentGap {
                case: "int-outside-foreign-tax",
                key: "INT_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
                field: "more/foreign_tax",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        5 => {
            let mut good = interest_gap_base();
            good.more.bond_premium = 100;
            good.nominee = Some(InterestNominee { bond_premium: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().bond_premium = 101;
            InvestmentGap {
                case: "int-outside-bond-premium",
                key: "INT_OUTSIDE_BOND_PREMIUM_ABOVE_BOX",
                field: "more/bond_premium",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        6 => {
            let mut good = interest_gap_base();
            good.more.treasury_bond_premium = 100;
            good.nominee =
                Some(InterestNominee { treasury_bond_premium: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_bond_premium = 101;
            InvestmentGap {
                case: "int-outside-treasury-bond-premium",
                key: "INT_OUTSIDE_TREASURY_BOND_PREMIUM_ABOVE_BOX",
                field: "more/treasury_bond_premium",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        7 => {
            let mut good = interest_gap_base();
            good.more.tax_exempt_bond_premium = 100;
            good.nominee =
                Some(InterestNominee { tax_exempt_bond_premium: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().tax_exempt_bond_premium = 101;
            InvestmentGap {
                case: "int-outside-tax-exempt-bond-premium",
                key: "INT_OUTSIDE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
                field: "more/tax_exempt_bond_premium",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        8 => {
            let mut good = interest_gap_base();
            good.more.accrued_interest = 100;
            good.nominee = Some(InterestNominee { accrued_interest: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().accrued_interest = 101;
            InvestmentGap {
                case: "int-outside-accrued-interest",
                key: "INT_OUTSIDE_ACCRUED_INTEREST_ABOVE_BOX",
                field: "more/accrued_interest",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        9 => {
            let mut good = interest_gap_base();
            good.more.market_discount = 100;
            good.nominee = Some(InterestNominee { market_discount: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().market_discount = 101;
            InvestmentGap {
                case: "int-outside-market-discount",
                key: "INT_OUTSIDE_MARKET_DISCOUNT_ABOVE_BOX",
                field: "more/market_discount",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        10 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(InterestNominee { exempt: 50, private_activity: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "int-nominee-private",
                key: "INT_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "nominee/private_activity",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        11 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.after_death =
                Some(InterestAfterDeath { exempt: 50, private_activity: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "int-after-death-private",
                key: "INT_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "after_death/private_activity",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        12 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(InterestNominee { exempt: 1, private_activity: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "int-retained-private",
                key: "INT_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "more/private_activity",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        13 => {
            let mut good = interest_gap_base();
            good.more.us_obligations = 1_000;
            good.nominee = Some(InterestNominee { us_obligations: 100, ..Default::default() });
            good.adjustments =
                Some(InterestAdjustments { previously_reported: 900, ..Default::default() });
            let mut bad = good.clone();
            bad.adjustments.as_mut().unwrap().previously_reported = 901;
            InvestmentGap {
                case: "int-prior-retained",
                key: "INTEREST_PRIOR_ABOVE_RETAINED",
                field: "adjustments/previously_reported",
                bad: IncomeDocument::Int1099(bad),
                corrected: IncomeDocument::Int1099(good),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit47_interest_missing_save_relationships_have_actual_source_controls() {
    for index in 0..14 {
        exercise_investment_gap(interest_gap(index));
    }
}

fn dividend_gap(index: usize) -> InvestmentGap {
    match index {
        0 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.qualified = 100;
            good.nominee =
                Some(DividendNominee { ordinary: 50, qualified: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 51;
            InvestmentGap {
                case: "div-nominee-qualified",
                key: "DIV_NOMINEE_QUALIFIED_ABOVE_ORDINARY",
                field: "nominee/qualified",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        1 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.qualified = 100;
            good.answers.after_death =
                Some(DividendsAfterDeath { ordinary: 50, qualified: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().qualified = 51;
            InvestmentGap {
                case: "div-after-qualified",
                key: "DIV_AFTER_QUALIFIED_ABOVE_ORDINARY",
                field: "answers/after_death/qualified",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        2 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.qualified = 100;
            good.nominee =
                Some(DividendNominee { ordinary: 1, qualified: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 0;
            InvestmentGap {
                case: "div-retained-qualified",
                key: "DIV_RETAINED_QUALIFIED_ABOVE_ORDINARY",
                field: "qualified",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        3 => {
            let mut good = dividend_gap_base();
            good.more.exempt_dividends = 100;
            good.more.private_activity_dividends = 100;
            good.nominee =
                Some(DividendNominee { exempt: 50, private_activity: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "div-nominee-private-activity",
                key: "DIV_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "nominee/private_activity",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        4 => {
            let mut good = dividend_gap_base();
            good.more.exempt_dividends = 100;
            good.more.private_activity_dividends = 100;
            good.answers.after_death = Some(DividendsAfterDeath {
                exempt_dividends: 50,
                private_activity_dividends: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().private_activity_dividends = 51;
            InvestmentGap {
                case: "div-after-private-activity",
                key: "DIV_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "answers/after_death/private_activity_dividends",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        5 => {
            let mut good = dividend_gap_base();
            good.more.exempt_dividends = 100;
            good.more.private_activity_dividends = 100;
            good.nominee =
                Some(DividendNominee { exempt: 1, private_activity: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "div-retained-private-activity",
                key: "DIV_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "more/private_activity_dividends",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        6 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section199a = 100;
            good.answers.reit_eligible = Some(true);
            good.nominee =
                Some(DividendNominee { ordinary: 50, section_199a: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 51;
            InvestmentGap {
                case: "div-nominee-section-199a",
                key: "DIV_NOMINEE_SECTION_199A_ABOVE_ORDINARY",
                field: "nominee/section_199a",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        7 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section199a = 100;
            good.answers.reit_eligible = Some(true);
            good.answers.after_death =
                Some(DividendsAfterDeath { ordinary: 50, section199a: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().section199a = 51;
            InvestmentGap {
                case: "div-after-section-199a",
                key: "DIV_AFTER_SECTION_199A_ABOVE_ORDINARY",
                field: "answers/after_death/section199a",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        8 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section199a = 100;
            good.answers.reit_eligible = Some(true);
            good.nominee =
                Some(DividendNominee { ordinary: 1, section_199a: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 0;
            InvestmentGap {
                case: "div-retained-section-199a",
                key: "DIV_RETAINED_SECTION_199A_ABOVE_ORDINARY",
                field: "more/section199a",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        9 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section_897_ordinary = 100;
            good.nominee = Some(DividendNominee {
                ordinary: 50,
                section_897_ordinary: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 51;
            InvestmentGap {
                case: "div-nominee-section-897-ordinary",
                key: "DIV_NOMINEE_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "nominee/section_897_ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        10 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section_897_ordinary = 100;
            good.answers.after_death = Some(DividendsAfterDeath {
                ordinary: 50,
                section_897_ordinary: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().section_897_ordinary = 51;
            InvestmentGap {
                case: "div-after-section-897-ordinary",
                key: "DIV_AFTER_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "answers/after_death/section_897_ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        11 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.more.section_897_ordinary = 100;
            good.nominee = Some(DividendNominee {
                ordinary: 1,
                section_897_ordinary: 1,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 0;
            InvestmentGap {
                case: "div-retained-section-897-ordinary",
                key: "DIV_RETAINED_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "more/section_897_ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        12 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_897_capital = 100;
            good.nominee = Some(DividendNominee {
                capital: 50,
                section_897_capital: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 51;
            InvestmentGap {
                case: "div-nominee-section-897-capital",
                key: "DIV_NOMINEE_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "nominee/section_897_capital",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        13 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_897_capital = 100;
            good.answers.after_death = Some(DividendsAfterDeath {
                capital_distributions: 50,
                section_897_capital: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().section_897_capital = 51;
            InvestmentGap {
                case: "div-after-section-897-capital",
                key: "DIV_AFTER_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "answers/after_death/section_897_capital",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        14 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_897_capital = 100;
            good.nominee =
                Some(DividendNominee { capital: 1, section_897_capital: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 0;
            InvestmentGap {
                case: "div-retained-section-897-capital",
                key: "DIV_RETAINED_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "more/section_897_capital",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        15 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.unrecaptured_distributions = 100;
            good.nominee =
                Some(DividendNominee { capital: 50, unrecaptured: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 51;
            InvestmentGap {
                case: "div-nominee-unrecaptured",
                key: "DIV_NOMINEE_UNRECAPTURED_ABOVE_CAPITAL",
                field: "nominee/unrecaptured",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        16 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.unrecaptured_distributions = 100;
            good.answers.after_death = Some(DividendsAfterDeath {
                capital_distributions: 50,
                unrecaptured_distributions: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().unrecaptured_distributions = 51;
            InvestmentGap {
                case: "div-after-unrecaptured",
                key: "DIV_AFTER_UNRECAPTURED_ABOVE_CAPITAL",
                field: "answers/after_death/unrecaptured_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        17 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.unrecaptured_distributions = 100;
            good.nominee =
                Some(DividendNominee { capital: 1, unrecaptured: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 0;
            InvestmentGap {
                case: "div-retained-unrecaptured",
                key: "DIV_RETAINED_UNRECAPTURED_ABOVE_CAPITAL",
                field: "more/unrecaptured_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        18 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.collectible_distributions = 100;
            good.nominee =
                Some(DividendNominee { capital: 50, collectible: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 51;
            InvestmentGap {
                case: "div-nominee-collectible",
                key: "DIV_NOMINEE_COLLECTIBLE_ABOVE_CAPITAL",
                field: "nominee/collectible",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        19 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.collectible_distributions = 100;
            good.answers.after_death = Some(DividendsAfterDeath {
                capital_distributions: 50,
                collectible_distributions: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().collectible_distributions = 51;
            InvestmentGap {
                case: "div-after-collectible",
                key: "DIV_AFTER_COLLECTIBLE_ABOVE_CAPITAL",
                field: "answers/after_death/collectible_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        20 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.collectible_distributions = 100;
            good.nominee =
                Some(DividendNominee { capital: 1, collectible: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 0;
            InvestmentGap {
                case: "div-retained-collectible",
                key: "DIV_RETAINED_COLLECTIBLE_ABOVE_CAPITAL",
                field: "more/collectible_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        21 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_1202_gain = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.nominee =
                Some(DividendNominee { capital: 50, section_1202: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 51;
            InvestmentGap {
                case: "div-nominee-section-1202",
                key: "DIV_NOMINEE_SECTION_1202_ABOVE_CAPITAL",
                field: "nominee/section_1202",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        22 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_1202_gain = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.answers.after_death = Some(DividendsAfterDeath {
                capital_distributions: 50,
                section_1202_gain: 50,
                ..Default::default()
            });
            let mut bad = good.clone();
            bad.answers.after_death.as_mut().unwrap().section_1202_gain = 51;
            InvestmentGap {
                case: "div-after-section-1202",
                key: "DIV_AFTER_SECTION_1202_ABOVE_CAPITAL",
                field: "answers/after_death/section_1202_gain",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        23 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_1202_gain = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.nominee =
                Some(DividendNominee { capital: 1, section_1202: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 0;
            InvestmentGap {
                case: "div-retained-section-1202",
                key: "DIV_RETAINED_SECTION_1202_ABOVE_CAPITAL",
                field: "more/section_1202_gain",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        24 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.nominee = Some(DividendNominee { ordinary: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().ordinary = 101;
            InvestmentGap {
                case: "div-outside-ordinary",
                key: "DIV_OUTSIDE_ORDINARY_ABOVE_BOX",
                field: "ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        25 => {
            let mut good = dividend_gap_base();
            good.qualified = 100;
            good.nominee = Some(DividendNominee { qualified: 100, ..Default::default() });
            good.ordinary = 100;
            good.nominee.as_mut().unwrap().ordinary = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 101;
            InvestmentGap {
                case: "div-outside-qualified",
                key: "DIV_OUTSIDE_QUALIFIED_ABOVE_BOX",
                field: "qualified",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        26 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.nominee = Some(DividendNominee { capital: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().capital = 101;
            InvestmentGap {
                case: "div-outside-capital",
                key: "DIV_OUTSIDE_CAPITAL_ABOVE_BOX",
                field: "more/capital_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        27 => {
            let mut good = dividend_gap_base();
            good.more.unrecaptured_distributions = 100;
            good.nominee = Some(DividendNominee { unrecaptured: 100, ..Default::default() });
            good.more.capital_distributions = 100;
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 101;
            InvestmentGap {
                case: "div-outside-unrecaptured",
                key: "DIV_OUTSIDE_UNRECAPTURED_ABOVE_BOX",
                field: "more/unrecaptured_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        28 => {
            let mut good = dividend_gap_base();
            good.more.collectible_distributions = 100;
            good.nominee = Some(DividendNominee { collectible: 100, ..Default::default() });
            good.more.capital_distributions = 100;
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 101;
            InvestmentGap {
                case: "div-outside-collectible",
                key: "DIV_OUTSIDE_COLLECTIBLE_ABOVE_BOX",
                field: "more/collectible_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        29 => {
            let mut good = dividend_gap_base();
            good.more.section_1202_gain = 100;
            good.nominee = Some(DividendNominee { section_1202: 100, ..Default::default() });
            good.more.capital_distributions = 100;
            good.nominee.as_mut().unwrap().capital = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 101;
            InvestmentGap {
                case: "div-outside-section1202",
                key: "DIV_OUTSIDE_SECTION_1202_ABOVE_BOX",
                field: "more/section_1202_gain",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        30 => {
            let mut good = dividend_gap_base();
            good.more.section199a = 100;
            good.nominee = Some(DividendNominee { section_199a: 100, ..Default::default() });
            good.ordinary = 100;
            good.nominee.as_mut().unwrap().ordinary = 100;
            good.answers.reit_eligible = Some(true);
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 101;
            InvestmentGap {
                case: "div-outside-section199a",
                key: "DIV_OUTSIDE_SECTION_199A_ABOVE_BOX",
                field: "more/section199a",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        31 => {
            let mut good = dividend_gap_base();
            good.more.exempt_dividends = 100;
            good.nominee = Some(DividendNominee { exempt: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "div-outside-exempt",
                key: "DIV_OUTSIDE_EXEMPT_ABOVE_BOX",
                field: "more/exempt_dividends",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        32 => {
            let mut good = dividend_gap_base();
            good.more.private_activity_dividends = 100;
            good.nominee = Some(DividendNominee { private_activity: 100, ..Default::default() });
            good.more.exempt_dividends = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "div-outside-private",
                key: "DIV_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "more/private_activity_dividends",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        33 => {
            let mut good = dividend_gap_base();
            good.more.foreign_tax = 100;
            good.nominee = Some(DividendNominee { foreign_tax: 100, ..Default::default() });
            good.more.foreign_country = "Canada".into();
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().foreign_tax = 101;
            InvestmentGap {
                case: "div-outside-foreign-tax",
                key: "DIV_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
                field: "more/foreign_tax",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        34 => {
            let mut good = dividend_gap_base();
            good.more.section_897_ordinary = 100;
            good.nominee =
                Some(DividendNominee { section_897_ordinary: 100, ..Default::default() });
            good.ordinary = 100;
            good.nominee.as_mut().unwrap().ordinary = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 101;
            InvestmentGap {
                case: "div-outside-section897ordinary",
                key: "DIV_OUTSIDE_SECTION_897_ORDINARY_ABOVE_BOX",
                field: "more/section_897_ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        35 => {
            let mut good = dividend_gap_base();
            good.more.section_897_capital = 100;
            good.nominee = Some(DividendNominee { section_897_capital: 100, ..Default::default() });
            good.more.capital_distributions = 100;
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 101;
            InvestmentGap {
                case: "div-outside-section897capital",
                key: "DIV_OUTSIDE_SECTION_897_CAPITAL_ABOVE_BOX",
                field: "more/section_897_capital",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        36 => {
            let mut good = dividend_gap_base();
            good.ordinary = 100;
            good.qualified = 60;
            good.more.section199a = 40;
            good.answers.reit_eligible = Some(true);
            let mut bad = good.clone();
            bad.more.section199a = 41;
            InvestmentGap {
                case: "div-qualified-reit-overlap",
                key: "DIV_OUTSIDE_QUALIFIED_REIT_OVERLAP",
                field: "ordinary",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        37 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.unrecaptured_distributions = 60;
            good.more.collectible_distributions = 40;
            let mut bad = good.clone();
            bad.more.collectible_distributions = 41;
            InvestmentGap {
                case: "div-capital-parts-overlap",
                key: "DIV_OUTSIDE_CAPITAL_PARTS_OVERLAP",
                field: "more/capital_distributions",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        38 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_1202_gain = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.more.section_1202_gain = 101;
            InvestmentGap {
                case: "div-1202-above-box",
                key: "DIV_SECTION_1202_ABOVE_2A",
                field: "more/section_1202_gain",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        39 => {
            let mut good = dividend_gap_base();
            good.more.capital_distributions = 100;
            good.more.section_1202_gain = 100;
            good.more.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.more.section_1202_exclusion = None;
            InvestmentGap {
                case: "div-1202-needs-answer",
                key: "DIV_SECTION_1202_NEEDS_EXCLUSION",
                field: "more/section_1202_exclusion",
                bad: IncomeDocument::Div1099(bad),
                corrected: IncomeDocument::Div1099(good),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit47_dividend_missing_save_relationships_have_actual_source_controls() {
    for index in 0..40 {
        exercise_investment_gap(dividend_gap(index));
    }
}

fn oid_gap(index: usize) -> InvestmentGap {
    match index {
        0 => {
            let mut good = oid_gap_base();
            good.oid = 100;
            good.nominee = Some(OidNominee { oid: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().oid = 101;
            InvestmentGap {
                case: "oid-nominee-oid",
                key: "OID_NOMINEE_OID_ABOVE_BOX",
                field: "nominee/oid",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        1 => {
            let mut good = oid_gap_base();
            good.other_periodic_interest = 100;
            good.nominee = Some(OidNominee { other_periodic_interest: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().other_periodic_interest = 101;
            InvestmentGap {
                case: "oid-nominee-other-periodic-interest",
                key: "OID_NOMINEE_OTHER_PERIODIC_INTEREST_ABOVE_BOX",
                field: "nominee/other_periodic_interest",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        2 => {
            let mut good = oid_gap_base();
            good.more.market_discount = 100;
            good.nominee = Some(OidNominee { market_discount: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().market_discount = 101;
            InvestmentGap {
                case: "oid-nominee-market-discount",
                key: "OID_NOMINEE_MARKET_DISCOUNT_ABOVE_BOX",
                field: "nominee/market_discount",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        3 => {
            let mut good = oid_gap_base();
            good.more.acquisition_premium = 100;
            good.nominee = Some(OidNominee { acquisition_premium: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().acquisition_premium = 101;
            InvestmentGap {
                case: "oid-nominee-acquisition-premium",
                key: "OID_NOMINEE_ACQUISITION_PREMIUM_ABOVE_BOX",
                field: "nominee/acquisition_premium",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        4 => {
            let mut good = oid_gap_base();
            good.more.bond_premium = 100;
            good.nominee = Some(OidNominee { bond_premium: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().bond_premium = 101;
            InvestmentGap {
                case: "oid-nominee-bond-premium",
                key: "OID_NOMINEE_BOND_PREMIUM_ABOVE_BOX",
                field: "nominee/bond_premium",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        5 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.nominee = Some(OidNominee { exempt: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "oid-nominee-exempt",
                key: "OID_NOMINEE_EXEMPT_ABOVE_BOX",
                field: "nominee/exempt",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        6 => {
            let mut good = oid_gap_base();
            good.more.tax_exempt_bond_premium = 100;
            good.nominee = Some(OidNominee { tax_exempt_bond_premium: 100, ..Default::default() });
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().tax_exempt_bond_premium = 101;
            InvestmentGap {
                case: "oid-nominee-tax-exempt-bond-premium",
                key: "OID_NOMINEE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
                field: "nominee/tax_exempt_bond_premium",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        7 => {
            let mut good = oid_gap_base();
            good.more.private_activity = 100;
            good.nominee = Some(OidNominee { private_activity: 100, ..Default::default() });
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "oid-nominee-private-activity",
                key: "OID_NOMINEE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "nominee/private_activity",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        8 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = 100;
            good.nominee = Some(OidNominee { treasury_oid: 100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = 101;
            InvestmentGap {
                case: "oid-treasury-positive",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/treasury_oid",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        9 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = -100;
            good.nominee = Some(OidNominee { treasury_oid: -100, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = -101;
            InvestmentGap {
                case: "oid-treasury-negative",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/treasury_oid",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        10 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = 100;
            good.nominee = Some(OidNominee { treasury_oid: 0, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = -1;
            InvestmentGap {
                case: "oid-treasury-sign",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/treasury_oid",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        11 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(OidNominee { exempt: 50, private_activity: 50, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "oid-nominee-private",
                key: "OID_NOMINEE_PRIVATE_ABOVE_EXEMPT",
                field: "nominee/private_activity",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        12 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(OidNominee { exempt: 1, private_activity: 1, ..Default::default() });
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "oid-retained-private",
                key: "OID_RETAINED_PRIVATE_ABOVE_EXEMPT",
                field: "more/private_activity",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        13 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.more.tax_exempt_bond_premium = 100;
            let mut bad = good.clone();
            bad.more.tax_exempt_bond_premium = 101;
            InvestmentGap {
                case: "oid-exempt-reduction",
                key: "EXEMPT_REDUCTION_ABOVE_AMOUNT",
                field: "more/tax_exempt_bond_premium",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        14 => {
            let mut good = oid_gap_base();
            good.oid = 100;
            good.adjustments = Some(InterestAdjustments::default());
            let mut bad = good.clone();
            bad.adjustments.as_mut().unwrap().previously_reported = 1;
            InvestmentGap {
                case: "oid-prior-not-penalty",
                key: "OID_PRIOR_IS_NOT_BOX3",
                field: "adjustments/previously_reported",
                bad: IncomeDocument::Oid1099(bad),
                corrected: IncomeDocument::Oid1099(good),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit47_oid_missing_save_relationships_have_actual_source_controls() {
    for index in 0..15 {
        exercise_investment_gap(oid_gap(index));
    }
}

fn supplemental_gap(index: usize) -> InvestmentGap {
    match index {
        0 => {
            let mut good = Form2439 {
                fund: "Actual fund source".into(),
                gain: 100,
                unrecaptured: 0,
                small_business: 0,
                section_1202_exclusion: Some(FundQsbsExclusion::None),
                collectibles: 0,
                tax_paid: 0,
                special: false,
            };
            good.small_business = 100;
            let mut bad = good.clone();
            bad.section_1202_exclusion = None;
            InvestmentGap {
                case: "2439-1202-needs-answer",
                key: "FORM_2439_SECTION_1202_NEEDS_EXCLUSION",
                field: "section_1202_exclusion",
                bad: IncomeDocument::Form2439(bad),
                corrected: IncomeDocument::Form2439(good),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit47_supplemental_missing_save_relationships_have_actual_source_controls() {
    for index in 0..1 {
        exercise_investment_gap(supplemental_gap(index));
    }
}
