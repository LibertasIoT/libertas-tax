// Actual Save/Estimate controls for remaining payer-statement relationships.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::support::client::income_pages::open_documents;

struct InvestmentGap {
    case: &'static str,
    key: &'static str,
    field: &'static str,
    bad: alloc::boxed::Box<dyn Fn() -> IncomeSource>,
    corrected: alloc::boxed::Box<dyn Fn() -> IncomeSource>,
}

// The admitted fixtures below all report components and use a final-return seed.
fn interest_gap_base() -> Int1099 {
    let mut value = native_int1099("Actual interest source".into(), 0);
    value.more.calls = IntCalls { nominee: vec![0], after_death: vec![0] };
    value
}
fn dividend_gap_base() -> DividendSource {
    let mut value = div1099_source(native_div1099("Actual dividend source".into(), 0, 0));
    value.calls = DivCalls { nominee: vec![0], after_death: vec![0], ..Default::default() };
    value
}
fn oid_gap_base() -> Oid1099 {
    let mut value = native_oid1099("Actual OID source".into(), 0, 0);
    value.more.calls = OidCalls { nominee: vec![0] };
    value
}

fn investment_gap_seed(corrected: &IncomeSource) -> (Interview, P, FederalResult) {
    let mut draft = complete(FederalStatus::Single);
    // Authorize genuine final-return recipient parts before testing their
    // relationships; a missing death must not preempt the intended owner.
    draft.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20260630, ordinary: true });
    let mut session = Interview::new(TEST_USER, draft);
    let mut unrelated = interest_gap_base();
    unrelated.payer.name = "Unrelated accepted interest source".into();
    unrelated.taxable = 10_000;
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(unrelated)),
                },
            ),
            document(0, corrected.clone()),
        ],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
    let baseline = estimate(&session.draft);
    let request = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
    let expected = expected_named_sources(&session.draft, Owner::Taxpayer, vec![corrected.clone()]);
    let IncomeSource::InterestDividend { value } = &expected[0] else { unreachable!() };
    assert_eq!(&documents.common.interest_dividends.documents[1].document, value);
    assert!(documents.common.interest_dividends.documents[1].id > 0);
    (session, request, baseline)
}

const DIV_PAPER_BOXES: &[&str] = &[
    "ordinary",
    "qualified",
    "capital_distributions",
    "unrecaptured_distributions",
    "section_1202_gain",
    "collectible_distributions",
    "section_897_ordinary",
    "section_897_capital",
    "nondividend_distributions",
    "withholding",
    "section199a",
    "investment_expenses",
    "foreign_tax",
    "foreign_country",
    "cash_liquidation",
    "noncash_liquidation",
    "exempt_dividends",
    "private_activity_dividends",
];

fn exercise_investment_gap(gap: InvestmentGap) {
    let corrected_source = (gap.corrected)();
    let (mut session, corrected, baseline) = investment_gap_seed(&corrected_source);
    let mut invalid = corrected.clone();
    let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
    let IncomeSource::InterestDividend { value } = (gap.bad)() else { unreachable!() };
    // The 1099-DIV paper prints its boxes directly, without the old page's `more` group.
    let field = match (&value, gap.field.strip_prefix("more/")) {
        (crate::InterestDividendDocument::Div1099(_), Some(box_field))
            if DIV_PAPER_BOXES.contains(&box_field.split('/').next().unwrap_or_default()) =>
        {
            box_field
        }
        _ => gap.field,
    };
    documents.common.interest_dividends.documents[1].document = value;
    let path =
        alloc::format!("[documents/common/interest_dividends/documents/1/document/{field}]");
    let invalid_estimate = credit_estimate_documents_from_save(&invalid);
    credit_refusal(&alloc::format!("{}-save", gap.case), &mut session, invalid, gap.key, &path);
    credit_refusal(
        &alloc::format!("{}-estimate", gap.case),
        &mut session,
        invalid_estimate,
        gap.key,
        &path,
    );
    let corrected_source = (gap.corrected)();
    let accepted_draft = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(&mut session, credit_estimate_documents_from_save(&corrected)),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, corrected));
    // The equal-boundary/source correction must preserve the full original
    // financial result, source identities, withholding and accepted records.
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(estimate(&session.draft), baseline);
    let expected = expected_named_source(&session.draft, Owner::Taxpayer, corrected_source);
    let reopened = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    let IncomeSource::InterestDividend { value } = &expected else { unreachable!() };
    assert_eq!(&documents.common.interest_dividends.documents[1].document, value);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, accepted_draft);
    let mut cold = restart(accepted_draft.clone());
    let reopened = open_documents(&mut cold);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    let IncomeSource::InterestDividend { value } = &expected else { unreachable!() };
    assert_eq!(&documents.common.interest_dividends.documents[1].document, value);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(estimate(&cold.draft), baseline);
}

fn interest_gap(index: usize) -> InvestmentGap {
    match index {
        0 => {
            let mut good = interest_gap_base();
            good.taxable = 100;
            good.nominee = Some(InterestNominee { taxable: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().taxable = 101;
            InvestmentGap {
                case: "int-outside-taxable",
                key: "INT_OUTSIDE_TAXABLE_ABOVE_BOX",
                field: "taxable",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        1 => {
            let mut good = interest_gap_base();
            good.more.us_obligations = 100;
            good.nominee =
                Some(InterestNominee { us_obligations: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().us_obligations = 101;
            InvestmentGap {
                case: "int-outside-us-obligations",
                key: "INT_OUTSIDE_US_OBLIGATIONS_ABOVE_BOX",
                field: "more/us_obligations",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        2 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.nominee = Some(InterestNominee { exempt: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "int-outside-exempt",
                key: "INT_OUTSIDE_EXEMPT_ABOVE_BOX",
                field: "more/exempt",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        3 => {
            let mut good = interest_gap_base();
            good.more.private_activity = 100;
            good.nominee =
                Some(InterestNominee { private_activity: 100, ..Default::default() }.into());
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "int-outside-private-activity",
                key: "INT_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "more/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        4 => {
            let mut good = interest_gap_base();
            good.more.foreign_tax = 100;
            good.nominee = Some(InterestNominee { foreign_tax: 100, ..Default::default() }.into());
            good.more.foreign_country = "Canada".into();
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().foreign_tax = 101;
            InvestmentGap {
                case: "int-outside-foreign-tax",
                key: "INT_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
                field: "more/foreign_tax",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        5 => {
            let mut good = interest_gap_base();
            good.more.bond_premium = 100;
            good.nominee = Some(InterestNominee { bond_premium: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().bond_premium = 101;
            InvestmentGap {
                case: "int-outside-bond-premium",
                key: "INT_OUTSIDE_BOND_PREMIUM_ABOVE_BOX",
                field: "more/bond_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        6 => {
            let mut good = interest_gap_base();
            good.more.treasury_bond_premium = 100;
            good.nominee =
                Some(InterestNominee { treasury_bond_premium: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_bond_premium = 101;
            InvestmentGap {
                case: "int-outside-treasury-bond-premium",
                key: "INT_OUTSIDE_TREASURY_BOND_PREMIUM_ABOVE_BOX",
                field: "more/treasury_bond_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        7 => {
            let mut good = interest_gap_base();
            good.more.tax_exempt_bond_premium = 100;
            good.nominee =
                Some(InterestNominee { tax_exempt_bond_premium: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().tax_exempt_bond_premium = 101;
            InvestmentGap {
                case: "int-outside-tax-exempt-bond-premium",
                key: "INT_OUTSIDE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
                field: "more/tax_exempt_bond_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        8 => {
            let mut good = interest_gap_base();
            good.more.accrued_interest = 100;
            good.nominee =
                Some(InterestNominee { accrued_interest: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().accrued_interest = 101;
            InvestmentGap {
                case: "int-outside-accrued-interest",
                key: "INT_OUTSIDE_ACCRUED_INTEREST_ABOVE_BOX",
                field: "more/accrued_interest",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        9 => {
            let mut good = interest_gap_base();
            good.more.market_discount = 100;
            good.nominee =
                Some(InterestNominee { market_discount: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().market_discount = 101;
            InvestmentGap {
                case: "int-outside-market-discount",
                key: "INT_OUTSIDE_MARKET_DISCOUNT_ABOVE_BOX",
                field: "more/market_discount",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        10 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee = Some(
                InterestNominee { exempt: 50, private_activity: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "int-nominee-private",
                key: "INT_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "nominee/value/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        11 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.after_death = Some(
                InterestAfterDeath { exempt: 50, private_activity: 50, ..Default::default() }
                    .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "int-after-death-private",
                key: "INT_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "after_death/value/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        12 => {
            let mut good = interest_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee = Some(
                InterestNominee { exempt: 1, private_activity: 1, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "int-retained-private",
                key: "INT_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "more/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
            }
        }
        13 => {
            let mut good = interest_gap_base();
            good.more.us_obligations = 1_000;
            good.nominee =
                Some(InterestNominee { us_obligations: 100, ..Default::default() }.into());
            good.adjustments =
                Some(InterestAdjustments { previously_reported: 900, ..Default::default() });
            let mut bad = good.clone();
            bad.adjustments.as_mut().unwrap().previously_reported = 901;
            InvestmentGap {
                case: "int-prior-retained",
                key: "INTEREST_PRIOR_ABOVE_RETAINED",
                field: "adjustments/previously_reported",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(good.clone())),
                }),
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
            good.form.ordinary = Some(100);
            good.form.qualified = Some(100);
            good.nominee =
                Some(DividendNominee { ordinary: 50, qualified: 50, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 51;
            InvestmentGap {
                case: "div-nominee-qualified",
                key: "DIV_NOMINEE_QUALIFIED_ABOVE_ORDINARY",
                field: "nominee/value/qualified",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        1 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.qualified = Some(100);
            good.after_death = Some(
                DividendsAfterDeath { ordinary: 50, qualified: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().qualified = 51;
            InvestmentGap {
                case: "div-after-qualified",
                key: "DIV_AFTER_QUALIFIED_ABOVE_ORDINARY",
                field: "answers/after_death/value/qualified",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        2 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.qualified = Some(100);
            good.nominee =
                Some(DividendNominee { ordinary: 1, qualified: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 0;
            InvestmentGap {
                case: "div-retained-qualified",
                key: "DIV_RETAINED_QUALIFIED_ABOVE_ORDINARY",
                field: "qualified",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        3 => {
            let mut good = dividend_gap_base();
            good.form.exempt_dividends = Some(100);
            good.form.private_activity_dividends = Some(100);
            good.nominee = Some(
                DividendNominee { exempt: 50, private_activity: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "div-nominee-private-activity",
                key: "DIV_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "nominee/value/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        4 => {
            let mut good = dividend_gap_base();
            good.form.exempt_dividends = Some(100);
            good.form.private_activity_dividends = Some(100);
            good.after_death = Some(
                DividendsAfterDeath {
                    exempt_dividends: 50,
                    private_activity_dividends: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().private_activity_dividends = 51;
            InvestmentGap {
                case: "div-after-private-activity",
                key: "DIV_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "answers/after_death/value/private_activity_dividends",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        5 => {
            let mut good = dividend_gap_base();
            good.form.exempt_dividends = Some(100);
            good.form.private_activity_dividends = Some(100);
            good.nominee = Some(
                DividendNominee { exempt: 1, private_activity: 1, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "div-retained-private-activity",
                key: "DIV_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                field: "more/private_activity_dividends",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        6 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section199a = Some(100);
            good.calls.reit_eligible = vec![0, 1];
            good.reit_eligible = Some(true.into());
            good.nominee = Some(
                DividendNominee { ordinary: 50, section_199a: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 51;
            InvestmentGap {
                case: "div-nominee-section-199a",
                key: "DIV_NOMINEE_SECTION_199A_ABOVE_ORDINARY",
                field: "nominee/value/section_199a",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        7 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section199a = Some(100);
            good.calls.reit_eligible = vec![0, 1];
            good.reit_eligible = Some(true.into());
            good.after_death = Some(
                DividendsAfterDeath { ordinary: 50, section199a: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().section199a = 51;
            InvestmentGap {
                case: "div-after-section-199a",
                key: "DIV_AFTER_SECTION_199A_ABOVE_ORDINARY",
                field: "answers/after_death/value/section199a",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        8 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section199a = Some(100);
            good.calls.reit_eligible = vec![0, 1];
            good.reit_eligible = Some(true.into());
            good.nominee =
                Some(DividendNominee { ordinary: 1, section_199a: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 0;
            InvestmentGap {
                case: "div-retained-section-199a",
                key: "DIV_RETAINED_SECTION_199A_ABOVE_ORDINARY",
                field: "more/section199a",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        9 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section_897_ordinary = Some(100);
            good.nominee = Some(
                DividendNominee { ordinary: 50, section_897_ordinary: 50, ..Default::default() }
                    .into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 51;
            InvestmentGap {
                case: "div-nominee-section-897-ordinary",
                key: "DIV_NOMINEE_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "nominee/value/section_897_ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        10 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section_897_ordinary = Some(100);
            good.after_death = Some(
                DividendsAfterDeath {
                    ordinary: 50,
                    section_897_ordinary: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().section_897_ordinary = 51;
            InvestmentGap {
                case: "div-after-section-897-ordinary",
                key: "DIV_AFTER_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "answers/after_death/value/section_897_ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        11 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.section_897_ordinary = Some(100);
            good.nominee = Some(
                DividendNominee { ordinary: 1, section_897_ordinary: 1, ..Default::default() }
                    .into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 0;
            InvestmentGap {
                case: "div-retained-section-897-ordinary",
                key: "DIV_RETAINED_SECTION_897_ORDINARY_ABOVE_ORDINARY",
                field: "more/section_897_ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        12 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_897_capital = Some(100);
            good.nominee = Some(
                DividendNominee { capital: 50, section_897_capital: 50, ..Default::default() }
                    .into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 51;
            InvestmentGap {
                case: "div-nominee-section-897-capital",
                key: "DIV_NOMINEE_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "nominee/value/section_897_capital",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        13 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_897_capital = Some(100);
            good.after_death = Some(
                DividendsAfterDeath {
                    capital_distributions: 50,
                    section_897_capital: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().section_897_capital = 51;
            InvestmentGap {
                case: "div-after-section-897-capital",
                key: "DIV_AFTER_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "answers/after_death/value/section_897_capital",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        14 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_897_capital = Some(100);
            good.nominee = Some(
                DividendNominee { capital: 1, section_897_capital: 1, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 0;
            InvestmentGap {
                case: "div-retained-section-897-capital",
                key: "DIV_RETAINED_SECTION_897_CAPITAL_ABOVE_CAPITAL",
                field: "more/section_897_capital",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        15 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.unrecaptured_distributions = Some(100);
            good.nominee = Some(
                DividendNominee { capital: 50, unrecaptured: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 51;
            InvestmentGap {
                case: "div-nominee-unrecaptured",
                key: "DIV_NOMINEE_UNRECAPTURED_ABOVE_CAPITAL",
                field: "nominee/value/unrecaptured",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        16 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.unrecaptured_distributions = Some(100);
            good.after_death = Some(
                DividendsAfterDeath {
                    capital_distributions: 50,
                    unrecaptured_distributions: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().unrecaptured_distributions = 51;
            InvestmentGap {
                case: "div-after-unrecaptured",
                key: "DIV_AFTER_UNRECAPTURED_ABOVE_CAPITAL",
                field: "answers/after_death/value/unrecaptured_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        17 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.unrecaptured_distributions = Some(100);
            good.nominee =
                Some(DividendNominee { capital: 1, unrecaptured: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 0;
            InvestmentGap {
                case: "div-retained-unrecaptured",
                key: "DIV_RETAINED_UNRECAPTURED_ABOVE_CAPITAL",
                field: "more/unrecaptured_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        18 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.collectible_distributions = Some(100);
            good.nominee =
                Some(DividendNominee { capital: 50, collectible: 50, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 51;
            InvestmentGap {
                case: "div-nominee-collectible",
                key: "DIV_NOMINEE_COLLECTIBLE_ABOVE_CAPITAL",
                field: "nominee/value/collectible",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        19 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.collectible_distributions = Some(100);
            good.after_death = Some(
                DividendsAfterDeath {
                    capital_distributions: 50,
                    collectible_distributions: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().collectible_distributions = 51;
            InvestmentGap {
                case: "div-after-collectible",
                key: "DIV_AFTER_COLLECTIBLE_ABOVE_CAPITAL",
                field: "answers/after_death/value/collectible_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        20 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.collectible_distributions = Some(100);
            good.nominee =
                Some(DividendNominee { capital: 1, collectible: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 0;
            InvestmentGap {
                case: "div-retained-collectible",
                key: "DIV_RETAINED_COLLECTIBLE_ABOVE_CAPITAL",
                field: "more/collectible_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        21 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.nominee = Some(
                DividendNominee { capital: 50, section_1202: 50, ..Default::default() }.into(),
            );
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 51;
            InvestmentGap {
                case: "div-nominee-section-1202",
                key: "DIV_NOMINEE_SECTION_1202_ABOVE_CAPITAL",
                field: "nominee/value/section_1202",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        22 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.after_death = Some(
                DividendsAfterDeath {
                    capital_distributions: 50,
                    section_1202_gain: 50,
                    ..Default::default()
                }
                .into(),
            );
            let mut bad = good.clone();
            bad.after_death.as_mut().unwrap().section_1202_gain = 51;
            InvestmentGap {
                case: "div-after-section-1202",
                key: "DIV_AFTER_SECTION_1202_ABOVE_CAPITAL",
                field: "answers/after_death/value/section_1202_gain",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        23 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            good.nominee =
                Some(DividendNominee { capital: 1, section_1202: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 0;
            InvestmentGap {
                case: "div-retained-section-1202",
                key: "DIV_RETAINED_SECTION_1202_ABOVE_CAPITAL",
                field: "more/section_1202_gain",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        24 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.nominee = Some(DividendNominee { ordinary: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().ordinary = 101;
            InvestmentGap {
                case: "div-outside-ordinary",
                key: "DIV_OUTSIDE_ORDINARY_ABOVE_BOX",
                field: "ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        25 => {
            let mut good = dividend_gap_base();
            good.form.qualified = Some(100);
            good.nominee = Some(DividendNominee { qualified: 100, ..Default::default() }.into());
            good.form.ordinary = Some(100);
            good.nominee.as_mut().unwrap().ordinary = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().qualified = 101;
            InvestmentGap {
                case: "div-outside-qualified",
                key: "DIV_OUTSIDE_QUALIFIED_ABOVE_BOX",
                field: "qualified",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        26 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.nominee = Some(DividendNominee { capital: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().capital = 101;
            InvestmentGap {
                case: "div-outside-capital",
                key: "DIV_OUTSIDE_CAPITAL_ABOVE_BOX",
                field: "more/capital_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        27 => {
            let mut good = dividend_gap_base();
            good.form.unrecaptured_distributions = Some(100);
            good.nominee = Some(DividendNominee { unrecaptured: 100, ..Default::default() }.into());
            good.form.capital_distributions = Some(100);
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().unrecaptured = 101;
            InvestmentGap {
                case: "div-outside-unrecaptured",
                key: "DIV_OUTSIDE_UNRECAPTURED_ABOVE_BOX",
                field: "more/unrecaptured_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        28 => {
            let mut good = dividend_gap_base();
            good.form.collectible_distributions = Some(100);
            good.nominee = Some(DividendNominee { collectible: 100, ..Default::default() }.into());
            good.form.capital_distributions = Some(100);
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().collectible = 101;
            InvestmentGap {
                case: "div-outside-collectible",
                key: "DIV_OUTSIDE_COLLECTIBLE_ABOVE_BOX",
                field: "more/collectible_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        29 => {
            let mut good = dividend_gap_base();
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.nominee = Some(DividendNominee { section_1202: 100, ..Default::default() }.into());
            good.form.capital_distributions = Some(100);
            good.nominee.as_mut().unwrap().capital = 100;
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_1202 = 101;
            InvestmentGap {
                case: "div-outside-section1202",
                key: "DIV_OUTSIDE_SECTION_1202_ABOVE_BOX",
                field: "more/section_1202_gain",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        30 => {
            let mut good = dividend_gap_base();
            good.form.section199a = Some(100);
            good.calls.reit_eligible = vec![0, 1];
            good.nominee = Some(DividendNominee { section_199a: 100, ..Default::default() }.into());
            good.form.ordinary = Some(100);
            good.nominee.as_mut().unwrap().ordinary = 100;
            good.reit_eligible = Some(true.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_199a = 101;
            InvestmentGap {
                case: "div-outside-section199a",
                key: "DIV_OUTSIDE_SECTION_199A_ABOVE_BOX",
                field: "more/section199a",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        31 => {
            let mut good = dividend_gap_base();
            good.form.exempt_dividends = Some(100);
            good.nominee = Some(DividendNominee { exempt: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "div-outside-exempt",
                key: "DIV_OUTSIDE_EXEMPT_ABOVE_BOX",
                field: "more/exempt_dividends",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        32 => {
            let mut good = dividend_gap_base();
            good.form.private_activity_dividends = Some(100);
            good.nominee =
                Some(DividendNominee { private_activity: 100, ..Default::default() }.into());
            good.form.exempt_dividends = Some(100);
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "div-outside-private",
                key: "DIV_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "more/private_activity_dividends",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        33 => {
            let mut good = dividend_gap_base();
            good.form.foreign_tax = Some(100);
            good.nominee = Some(DividendNominee { foreign_tax: 100, ..Default::default() }.into());
            good.form.foreign_country = "Canada".into();
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().foreign_tax = 101;
            InvestmentGap {
                case: "div-outside-foreign-tax",
                key: "DIV_OUTSIDE_FOREIGN_TAX_ABOVE_BOX",
                field: "more/foreign_tax",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        34 => {
            let mut good = dividend_gap_base();
            good.form.section_897_ordinary = Some(100);
            good.nominee =
                Some(DividendNominee { section_897_ordinary: 100, ..Default::default() }.into());
            good.form.ordinary = Some(100);
            good.nominee.as_mut().unwrap().ordinary = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_ordinary = 101;
            InvestmentGap {
                case: "div-outside-section897ordinary",
                key: "DIV_OUTSIDE_SECTION_897_ORDINARY_ABOVE_BOX",
                field: "more/section_897_ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        35 => {
            let mut good = dividend_gap_base();
            good.form.section_897_capital = Some(100);
            good.nominee =
                Some(DividendNominee { section_897_capital: 100, ..Default::default() }.into());
            good.form.capital_distributions = Some(100);
            good.nominee.as_mut().unwrap().capital = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().section_897_capital = 101;
            InvestmentGap {
                case: "div-outside-section897capital",
                key: "DIV_OUTSIDE_SECTION_897_CAPITAL_ABOVE_BOX",
                field: "more/section_897_capital",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        36 => {
            let mut good = dividend_gap_base();
            good.form.ordinary = Some(100);
            good.form.qualified = Some(60);
            good.form.section199a = Some(40);
            good.calls.reit_eligible = vec![0, 1];
            good.reit_eligible = Some(true.into());
            let mut bad = good.clone();
            bad.form.section199a = Some(41);
            InvestmentGap {
                case: "div-qualified-reit-overlap",
                key: "DIV_OUTSIDE_QUALIFIED_REIT_OVERLAP",
                field: "ordinary",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        37 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.unrecaptured_distributions = Some(60);
            good.form.collectible_distributions = Some(40);
            let mut bad = good.clone();
            bad.form.collectible_distributions = Some(41);
            InvestmentGap {
                case: "div-capital-parts-overlap",
                key: "DIV_OUTSIDE_CAPITAL_PARTS_OVERLAP",
                field: "more/capital_distributions",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        38 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.form.section_1202_gain = Some(101);
            InvestmentGap {
                case: "div-1202-above-box",
                key: "DIV_SECTION_1202_ABOVE_2A",
                field: "more/section_1202_gain",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
            }
        }
        39 => {
            let mut good = dividend_gap_base();
            good.form.capital_distributions = Some(100);
            good.form.section_1202_gain = Some(100);
            good.calls.section_1202_exclusion = vec![0, 1, 2, 3];
            good.section_1202_exclusion = Some(FundQsbsExclusion::None);
            let mut bad = good.clone();
            bad.section_1202_exclusion = None;
            InvestmentGap {
                case: "div-1202-needs-answer",
                key: "DIV_SECTION_1202_NEEDS_EXCLUSION",
                field: "more/section_1202_exclusion",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(good.clone())),
                }),
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
            good.nominee = Some(OidNominee { oid: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().oid = 101;
            InvestmentGap {
                case: "oid-nominee-oid",
                key: "OID_NOMINEE_OID_ABOVE_BOX",
                field: "nominee/value/oid",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        1 => {
            let mut good = oid_gap_base();
            good.other_periodic_interest = 100;
            good.nominee =
                Some(OidNominee { other_periodic_interest: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().other_periodic_interest = 101;
            InvestmentGap {
                case: "oid-nominee-other-periodic-interest",
                key: "OID_NOMINEE_OTHER_PERIODIC_INTEREST_ABOVE_BOX",
                field: "nominee/value/other_periodic_interest",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        2 => {
            let mut good = oid_gap_base();
            good.more.market_discount = 100;
            good.nominee = Some(OidNominee { market_discount: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().market_discount = 101;
            InvestmentGap {
                case: "oid-nominee-market-discount",
                key: "OID_NOMINEE_MARKET_DISCOUNT_ABOVE_BOX",
                field: "nominee/value/market_discount",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        3 => {
            let mut good = oid_gap_base();
            good.more.acquisition_premium = 100;
            good.nominee =
                Some(OidNominee { acquisition_premium: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().acquisition_premium = 101;
            InvestmentGap {
                case: "oid-nominee-acquisition-premium",
                key: "OID_NOMINEE_ACQUISITION_PREMIUM_ABOVE_BOX",
                field: "nominee/value/acquisition_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        4 => {
            let mut good = oid_gap_base();
            good.more.bond_premium = 100;
            good.nominee = Some(OidNominee { bond_premium: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().bond_premium = 101;
            InvestmentGap {
                case: "oid-nominee-bond-premium",
                key: "OID_NOMINEE_BOND_PREMIUM_ABOVE_BOX",
                field: "nominee/value/bond_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        5 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.nominee = Some(OidNominee { exempt: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().exempt = 101;
            InvestmentGap {
                case: "oid-nominee-exempt",
                key: "OID_NOMINEE_EXEMPT_ABOVE_BOX",
                field: "nominee/value/exempt",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        6 => {
            let mut good = oid_gap_base();
            good.more.tax_exempt_bond_premium = 100;
            good.nominee =
                Some(OidNominee { tax_exempt_bond_premium: 100, ..Default::default() }.into());
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().tax_exempt_bond_premium = 101;
            InvestmentGap {
                case: "oid-nominee-tax-exempt-bond-premium",
                key: "OID_NOMINEE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX",
                field: "nominee/value/tax_exempt_bond_premium",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        7 => {
            let mut good = oid_gap_base();
            good.more.private_activity = 100;
            good.nominee = Some(OidNominee { private_activity: 100, ..Default::default() }.into());
            good.more.exempt = 100;
            good.nominee.as_mut().unwrap().exempt = 100;
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 101;
            InvestmentGap {
                case: "oid-nominee-private-activity",
                key: "OID_NOMINEE_PRIVATE_ACTIVITY_ABOVE_BOX",
                field: "nominee/value/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        8 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = 100;
            good.nominee = Some(OidNominee { treasury_oid: 100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = 101;
            InvestmentGap {
                case: "oid-treasury-positive",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/value/treasury_oid",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        9 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = -100;
            good.nominee = Some(OidNominee { treasury_oid: -100, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = -101;
            InvestmentGap {
                case: "oid-treasury-negative",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/value/treasury_oid",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        10 => {
            let mut good = oid_gap_base();
            good.more.treasury_oid = 100;
            good.nominee = Some(OidNominee { treasury_oid: 0, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().treasury_oid = -1;
            InvestmentGap {
                case: "oid-treasury-sign",
                key: "OID_NOMINEE_TREASURY_SIGN_OR_BOUND",
                field: "nominee/value/treasury_oid",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        11 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(OidNominee { exempt: 50, private_activity: 50, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 51;
            InvestmentGap {
                case: "oid-nominee-private",
                key: "OID_NOMINEE_PRIVATE_ABOVE_EXEMPT",
                field: "nominee/value/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
            }
        }
        12 => {
            let mut good = oid_gap_base();
            good.more.exempt = 100;
            good.more.private_activity = 100;
            good.nominee =
                Some(OidNominee { exempt: 1, private_activity: 1, ..Default::default() }.into());
            let mut bad = good.clone();
            bad.nominee.as_mut().unwrap().private_activity = 0;
            InvestmentGap {
                case: "oid-retained-private",
                key: "OID_RETAINED_PRIVATE_ABOVE_EXEMPT",
                field: "more/private_activity",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
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
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
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
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(good.clone())),
                }),
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
            let good = Form2439 {
                fund: crate::PrintedParty::from_name("Actual fund source".into()),
                gain: 100,
                small_business: 100,
                ..Default::default()
            };
            let mut bad = good.clone();
            bad.unrecaptured = 1;
            InvestmentGap {
                case: "2439-parts-above-1a",
                key: "FORM_2439_PARTS_ABOVE_1A",
                field: "total_undistributed_long_term_capital_gains",
                bad: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form2439(form2439_paper(bad.clone())),
                }),
                corrected: alloc::boxed::Box::new(move || IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form2439(form2439_paper(good.clone())),
                }),
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
