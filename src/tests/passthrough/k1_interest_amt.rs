use super::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::{
    k1_investment_interest_return, k1_investment_interest_source,
};
use alloc::format;

// 2026-10-19: Form 6251 line 2c uses each system's actual Form 4952,
// including its own basis-eligible line 1 and retained line 2. Form 6198
// line 4 coordination remains unavailable when the at-risk stage binds.
#[test]
fn current_interest_independent_basis_reaches_both_4952_forms_and_line2c() {
    for scorp in [false, true] {
        for (basis, income, allowed, carry, adjustment) in [
            ([10_049, 5_049], 200, [100, 50], [0, 0], 50),
            ([10_049, 5_049], 60, [60, 50], [40, 0], 10),
            ([5_049, 10_049], 200, [50, 100], [0, 0], -50),
            ([0, 10_049], 200, [0, 100], [0, 0], -100),
        ] {
            let d = k1_investment_interest_return(scorp, basis, income);
            let r = supported(&d);
            for (i, prefix) in
                ["investment_form_line", "amt_investment_form_line"].into_iter().enumerate()
            {
                assert_eq!(
                    value(&r, &format!("{prefix}1")),
                    crate::kernel::ratio::dollars(basis[i])
                );
                assert_eq!(value(&r, &format!("{prefix}7")), carry[i]);
                assert_eq!(value(&r, &format!("{prefix}8")), allowed[i]);
            }
            assert_eq!(value(&r, "amt_investment_interest_adjustment"), adjustment);
            assert_eq!(
                value(&r, "alternative_minimum_taxable_income"),
                200_000 + income - allowed[1]
            );
        }
    }
}

#[test]
fn current_interest_source_cents_and_basis_carry_stay_separate_from_4952_carry() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 300);
        let entry = d.income.last().unwrap();
        let facts = match &entry.income {
            FederalIncome::Partnership { data } => {
                crate::community::partner_facts(&d, entry, data).unwrap()
            }
            FederalIncome::Scorp { data } => {
                let source_view = ScorpK1::from(data);
                let data = &source_view;
                crate::community::scorp_facts(&d, entry, data)
            }
            _ => unreachable!(),
        };
        assert_eq!(facts.flows.investment_interest, 10_049);
        assert_eq!(facts.amt_flows.investment_interest, 5_049);
        let ledgers = match &entry.income {
            FederalIncome::Partnership { data } => [
                crate::community::partner_ledger(&d, entry.id, data, &data.regular).unwrap(),
                crate::community::partner_ledger(
                    &d,
                    entry.id,
                    data,
                    data.alternative.as_ref().unwrap(),
                )
                .unwrap(),
            ],
            FederalIncome::Scorp { data } => {
                let source_view = ScorpK1::from(data);
                let data = &source_view;
                [
                    crate::community::scorp_ledger(&d, entry.id, data, false).unwrap(),
                    crate::community::scorp_ledger(
                        &d,
                        entry.id,
                        data.amt.as_ref().unwrap().source(data).unwrap(),
                        true,
                    )
                    .unwrap(),
                ]
            }
            _ => unreachable!(),
        };
        assert_eq!(ledgers[0].basis_suspended[K1LossClass::InvestmentInterest].get(), 0);
        assert_eq!(ledgers[1].basis_suspended[K1LossClass::InvestmentInterest].get(), 5_000);
        add(
            &mut d,
            income(k1_investment_interest_source(scorp, 10_049, [10_049, 5_049], [100_000; 2])),
        );
        let r = supported(&d);
        // One rounding of each form's combined line: 20,098c / 10,098c.
        assert_eq!(value(&r, "investment_form_line1"), 201);
        assert_eq!(value(&r, "amt_investment_form_line1"), 101);
        assert_eq!(value(&r, "amt_investment_form_line7"), 0);
    }
}

#[test]
fn current_interest_and_existing_borrowing_keep_independent_4952_history() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 60);
        investment_borrowing(&mut d, 20, 30, 10);
        let r = supported(&d);
        for (id, expected) in [
            ("investment_form_line1", 120),
            ("amt_investment_form_line1", 70),
            ("investment_form_line2", 30),
            ("amt_investment_form_line2", 10),
            ("investment_form_line8", 60),
            ("amt_investment_form_line8", 60),
            ("investment_form_line7", 90),
            ("amt_investment_form_line7", 20),
            ("amt_investment_interest_adjustment", 0),
        ] {
            assert_eq!(value(&r, id), expected, "{id}");
        }
    }
}

#[test]
fn standard_deduction_keeps_independent_interest_forms_without_schedule_a_adjustment() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 60);
        d.deductions.as_mut().unwrap().itemize_override = None;
        let r = supported(&d);
        assert_eq!(value(&r, "investment_form_line8"), 60);
        assert_eq!(value(&r, "amt_investment_form_line8"), 50);
        assert_eq!(value(&r, "investment_interest_deduction"), 0);
        assert_eq!(value(&r, "amt_investment_interest_adjustment"), 0);
    }
}

#[test]
fn current_interest_at_risk_boundary_checks_each_actual_tax_system() {
    for scorp in [false, true] {
        for risk in [[0, 100_000], [100_000, 0], [0, 0]] {
            let mut d = empty_return(FederalStatus::Single);
            add(&mut d, income(k1_investment_interest_source(scorp, 10_000, [20_000; 2], risk)));
            let r = estimate(&d);
            assert_eq!(r.state, ResultState::Unsupported);
            assert_eq!(
                r.issues
                    .iter()
                    .filter(|issue| issue.display.key == "ISSUE_K1_INVESTMENT_INTEREST_AT_RISK")
                    .count(),
                1
            );
        }
    }
}

#[test]
fn current_interest_unknown_amt_risk_history_remains_a_named_boundary() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 200);
        match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                data.alternative.as_mut().unwrap().risk.start = ScorpRiskStart::Unknown;
            }
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;

                data.amt.as_mut().unwrap().risk.start = ScorpRiskStart::Unknown;
            }
            _ => unreachable!(),
        }
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        let key = if scorp {
            "ISSUE_SCORP_RISK_HISTORY_NEEDED"
        } else {
            "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED"
        };
        assert!(r.issues.iter().any(|issue| issue.display.key == key));
    }
}

#[test]
fn differing_interest_does_not_hide_unmapped_gift_allowances() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 200);
        match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => data.deductions.push(PartnerDeduction {
                code: PartnerDeductionCode::NoncashFifty,
                amount: 10_000,
            }),
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;
                data.deductions
                    .push(ScorpDeduction { code: ScorpDeductionCode::NoncashFifty, amount: 10_000 })
            }
            _ => unreachable!(),
        }
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(r.issues.iter().any(|issue| issue.display.key == "ISSUE_K1_AMT_LIMIT_DIFFERS"));
    }
}

#[test]
fn current_interest_native_save_reopen_keyed_restart_and_correction_keep_source_ids() {
    for scorp in [false, true] {
        let mut d = k1_investment_interest_return(scorp, [10_049, 5_049], 60);
        investment_borrowing(&mut d, 20, 30, 10);
        let id = d.income.last().unwrap().id;
        let saved = save_reopen_native_sources(
            d,
            &[TaxSection::Documents, TaxSection::OtherItemized],
            &[TaxSection::Documents, TaxSection::OtherItemized],
        );
        assert!(
            saved.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap().loans[0]
                .originating_k1
                .is_none()
        );
        assert_eq!(value(&supported(&saved), "investment_form_line7"), 90);
        assert_eq!(value(&supported(&saved), "amt_investment_form_line7"), 20);
        let mut session = restart(saved);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let row = documents.common.k1.documents.iter_mut().find(|row| row.id == id).unwrap();
        match &mut row.document {
            crate::K1Document::K1Partnership(page) => page.form.deductions[0].amount = Some(2_000),
            crate::K1Document::K1Scorp(page) => page.form.deductions[0].amount = Some(2_000),
            _ => unreachable!(),
        }
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.income.last().unwrap().id, id);
        let r = supported(&session.draft);
        assert_eq!(value(&r, "investment_form_line7"), 10);
        assert_eq!(value(&r, "amt_investment_form_line7"), 0);
        assert_eq!(value(&r, "investment_form_line8"), 60);
        assert_eq!(value(&r, "amt_investment_form_line8"), 50);
        assert_eq!(value(&r, "amt_investment_interest_adjustment"), 10);
        assert_eq!(estimate(&restart(session.draft).draft), r);
    }
}
