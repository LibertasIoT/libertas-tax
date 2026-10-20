use super::*;
use crate::tests::cross::interview::{accepted, document, open_topic};

fn actual_sources(ordinary: i64, election: i64, capacity: i64) -> [FederalIncome; 2] {
    let history = || Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() };
    let risk = ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
    let mut partner = ordinary_partner_source(ordinary, true);
    partner.regular.basis.opening = 100_000_000_000;
    partner.regular.risk.start = risk.clone();
    partner.regular.risk.prior_loss = 0;
    partner.regular.risk.deducted = 0;
    partner.regular.risk.recaptured = 0;
    partner.section179 = election;
    partner.alternative = Some(partner.regular.clone());
    partner.amt_section179_income_carry = Some(history());
    let mut scorp = complete_ordinary_scorp(ordinary, true);
    scorp.loans.clear();
    scorp.opening = 100_000_000_000;
    scorp.risk.start = risk;
    scorp.risk.prior_loss = 0;
    scorp.risk.deducted = 0;
    scorp.risk.recaptured = 0;
    scorp.section179 = election;
    scorp.qbi_statement = None;
    scorp.amt = Some(amt_records(&scorp));
    scorp.amt_section179_income_carry = Some(history());
    [FederalIncome::Partnership { data: partner }, FederalIncome::Scorp { data: scorp }]
}

#[test]
fn unit17_native_risk_keeps_selected_system_before179_and_basis_results() {
    for mut source in actual_sources(-20_000, 1_000_000, 10_000_000) {
        match &mut source {
            FederalIncome::Partnership { data } => {
                let amt = data.alternative.as_mut().unwrap();
                amt.ordinary = -1_000_000;
                amt.risk.start =
                    ScorpRiskStart::Detailed { capacity: 4_000_000, profit: 0, loss: 0 };
            }
            FederalIncome::Scorp { data } => {
                let amt = data.amt.as_mut().unwrap();
                amt.adjustment = 1_000_000;
                amt.risk.start =
                    ScorpRiskStart::Detailed { capacity: 4_000_000, profit: 0, loss: 0 };
            }
            _ => unreachable!(),
        }
        let (draft, id) = crate::tests::passthrough::k1_sale_return::draft(source);
        let before = draft.clone();
        for (alternative, remaining) in [(false, 8_000_000), (true, 3_000_000)] {
            let actual = crate::k1_section179_current_source(&draft, id, alternative).unwrap();
            assert_eq!(
                actual.risk,
                K1Section179RiskInput { k1: id, alternative, remaining_cents: Some(remaining) }
            );
            // The native risk stage has deducted the actual ordinary loss,
            // but has not deducted this separately owned current179 election.
            assert_eq!(actual.basis.elected_cents, 1_000_000);
            assert_eq!(actual.basis.basis_allowed_cents, 1_000_000);
        }
        assert_eq!(draft, before);
    }
}

#[test]
fn unit17_native_risk_unknown_is_not_zero_and_does_not_reuse_basis() {
    for mut source in actual_sources(1_000, 0, 10_000_000) {
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.regular.risk.start = ScorpRiskStart::Unknown;
                data.alternative.as_mut().unwrap().risk.start = ScorpRiskStart::Unknown;
            }
            FederalIncome::Scorp { data } => {
                data.risk.start = ScorpRiskStart::Unknown;
                data.amt.as_mut().unwrap().risk.start = ScorpRiskStart::Unknown;
            }
            _ => unreachable!(),
        }
        let (draft, id) = crate::tests::passthrough::k1_sale_return::draft(source);
        for alternative in [false, true] {
            let actual = crate::k1_section179_current_source(&draft, id, alternative).unwrap();
            assert_eq!(actual.risk.remaining_cents, None);
            assert_eq!(actual.basis.basis_allowed_cents, 0);
        }
    }
}

#[test]
fn unit17_native_risk_preserves_source_cents_before_the_annual_boundary() {
    for (capacity, represented) in
        [(99_999_999_949, 99_999_999_900), (99_999_999_950, 100_000_000_000)]
    {
        for source in actual_sources(0, 0, capacity) {
            let (draft, id) = crate::tests::passthrough::k1_sale_return::draft(source);
            for alternative in [false, true] {
                let actual = crate::k1_section179_current_source(&draft, id, alternative).unwrap();
                // 2026-10-19, app-contract Money precision: intermediate
                // risk capacity retains source cents; only the annual endpoint rounds.
                assert_eq!(actual.risk.remaining_cents, Some(capacity));
                assert_eq!(
                    crate::kernel::ratio::dollars(actual.risk.remaining_cents.unwrap()),
                    represented / 100
                );
            }
        }
    }
}

#[test]
fn unit17_native_risk_normal_save_restart_and_correction_preserve_source_id() {
    for source in actual_sources(-20_000, 1_000_000, 10_000_000) {
        let page = match source {
            FederalIncome::Partnership { data } => IncomeDocument::K1Partnership(
                K1Partnership::from_record("Synthetic risk source", &data).unwrap(),
            ),
            FederalIncome::Scorp { data } => IncomeDocument::K1Scorp(
                K1Scorp::from_record("Synthetic risk source", &data).unwrap(),
            ),
            _ => unreachable!(),
        };
        let mut initial = complete(FederalStatus::Single);
        initial.income.clear();
        let mut session = restart(initial);
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        documents.documents = vec![document(0, page)];
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        let id = session.draft.income[0].id;
        assert!(id > 0);
        let actual = crate::k1_section179_current_source(&session.draft, id, false).unwrap();
        assert_eq!(actual.risk.remaining_cents, Some(8_000_000));
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(session.draft.tax_year, &stored);
        assert_eq!(draft, session.draft);
        assert_eq!(native_draft_records(&draft), stored);
        let mut session = restart(draft);
        let request = open_topic(&mut session, TaxSection::PassThrough);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(session.draft.income[0].id, id);
        assert_eq!(crate::k1_section179_current_source(&session.draft, id, false).unwrap(), actual);
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        match &mut documents.documents[0].document {
            IncomeDocument::K1Partnership(page) => page.ordinary = -1_000_000,
            IncomeDocument::K1Scorp(page) => page.ordinary = -1_000_000,
            _ => unreachable!(),
        }
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(session.draft.income[0].id, id);
        assert_eq!(
            crate::k1_section179_current_source(&session.draft, id, false)
                .unwrap()
                .risk
                .remaining_cents,
            Some(9_000_000)
        );
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn unit17_native_current_risk_does_not_require_unrelated_amt_income_carry() {
    for mut source in actual_sources(-20_000, 1_000_000, 10_000_000) {
        match &mut source {
            FederalIncome::Partnership { data } => data.amt_section179_income_carry = None,
            FederalIncome::Scorp { data } => data.amt_section179_income_carry = None,
            _ => unreachable!(),
        }
        let (draft, id) = crate::tests::passthrough::k1_sale_return::draft(source);
        let current = crate::k1_section179_current_source(&draft, id, true).unwrap();
        assert_eq!(current.risk.remaining_cents, Some(8_000_000));
        assert_eq!(current.basis.basis_allowed_cents, 1_000_000);
    }
}

#[test]
fn unit17_native_current_risk_retains_canonical_scorp_amt_derived_capacity() {
    let FederalIncome::Scorp { mut data } = actual_sources(1_000_000_000, 1_000_000, 0)
        .into_iter()
        .find(|source| matches!(source, FederalIncome::Scorp { .. }))
        .unwrap()
    else {
        unreachable!()
    };
    data.amt.as_mut().unwrap().adjustment = 100_000_000_000;
    let (draft, id) =
        crate::tests::passthrough::k1_sale_return::draft(FederalIncome::Scorp { data });
    for (alternative, remaining) in [(false, 100_000_000_000), (true, 200_000_000_000)] {
        let source = crate::k1_section179_current_source(&draft, id, alternative).unwrap();
        assert_eq!(source.risk.remaining_cents, Some(remaining));
        assert_eq!(source.basis.elected_cents, 1_000_000);
        assert_eq!(source.basis.basis_allowed_cents, 1_000_000);
    }
}

#[test]
fn unit17_k1_source_lookup_preserves_errors_and_income_only_scope() {
    use crate::K1Section179BasisIssue as E;
    let errors = |draft: &FederalDraft, id| {
        [
            crate::k1_section179_current_source(draft, id, false).err(),
            crate::k1_section179_availability_source(draft, id, false).err(),
        ]
    };
    for source in actual_sources(-20_000, 1_000_000, 10_000_000) {
        let (mut draft, id) = crate::tests::passthrough::k1_sale_return::draft(source);
        for missing in [-1, 0, 999] {
            assert_eq!(errors(&draft, missing), [Some(E::UnknownSource); 2]);
        }
        let original = draft.income[0].clone();
        draft.income[0].income = cash_business(20_000);
        assert_eq!(errors(&draft, id), [Some(E::NotK1); 2]);
        // Ambiguity takes precedence even when neither match is a K-1.
        draft.income.push(draft.income[0].clone());
        assert_eq!(errors(&draft, id), [Some(E::InvalidSource("K1_SOURCE_ID_TWICE")); 2]);
        let setup = draft.setup.take();
        for requested in [-1, id, 999] {
            assert_eq!(errors(&draft, requested), [Some(E::MissingReturnContext); 2]);
        }
        draft.setup = setup;
        draft.income.pop();
        draft.income[0] = original;

        let current = crate::k1_section179_current_source(&draft, id, false).unwrap();
        let availability = crate::k1_section179_availability_source(&draft, id, false).unwrap();
        let FederalIncome::Business { mut data } = cash_business(20_000) else { unreachable!() };
        let mut asset = depreciable_asset(14_000, EquipmentClass::Five, 20260101, 0);
        // Native suppliers search income entries, so a nested property with
        // this ID must not turn the unique income source into an ambiguity.
        asset.id = id;
        data.assets.push(asset);
        add(&mut draft, income(FederalIncome::Business { data }));
        assert_eq!(crate::k1_section179_current_source(&draft, id, false), Ok(current));
        assert_eq!(crate::k1_section179_availability_source(&draft, id, false), Ok(availability));
    }
}
