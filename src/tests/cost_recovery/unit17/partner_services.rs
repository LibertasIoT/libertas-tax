use super::*;
use crate::tests::support::assert::accepted;
use crate::tests::support::builders::partner_service179_session;
use crate::tests::support::client::income_pages::open_topic;

#[test]
fn unit17_partner_services_enter_owner_income_once_in_each_system() {
    // Approved map R3 / Reg.1.179-2(c)(2),(6): service compensation is
    // the partner's own income, outside the entity's ordinary-income pool.
    let mut draft = partner_service179_session(200_049, 0).draft;
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.alternative.as_mut().unwrap().ordinary = 700_000;
    for (system, ordinary, limit) in [
        (Section179IncomeSystem::Regular, 500_000, 6_700_049),
        (Section179IncomeSystem::Alternative, 700_000, 6_900_049),
    ] {
        let FederalIncome::Partnership { data } = &draft.income.last().unwrap().income else {
            unreachable!()
        };
        let record = if system == Section179IncomeSystem::Regular {
            &data.regular
        } else {
            data.alternative.as_ref().unwrap()
        };
        let activity = crate::community::partner_pool(&draft, id, data, record).unwrap();
        assert_eq!(activity.current_ordinary_with_items, ordinary);
        let selected = crate::section179_return::prepare_with_home(
            &draft,
            system,
            &crate::home_office::HomeContext::default(),
        )
        .unwrap();
        assert_eq!(selected.allocation.income.business_income_limit_cents, limit);
        assert_eq!(selected.allocation.income.k1[&id].allowed_cents, limit);
        assert_eq!(
            selected.allocation.income.k1[&id].business_income_carry_cents,
            7_000_000 - limit
        );
        let mut result = FederalResult::empty(draft.revision);
        selected.report(&mut result).unwrap();
        assert!(
            result.issues.iter().all(|issue| !issue.display.key.contains("GUARANTEED_PAYMENTS"))
        );
    }
    assert_eq!(crate::section179::calculate(&draft).income, 6_700_049);
}

#[test]
fn unit17_partner_services_preserve_capital_ptp_and_owner_deduction_boundaries() {
    for boundary in ["capital", "ptp", "pension"] {
        let mut draft = partner_service179_session(200_000, 0).draft;
        let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income
        else {
            unreachable!()
        };
        match boundary {
            "capital" => data.guaranteed_capital = 100_000,
            "ptp" => data.publicly_traded = true,
            "pension" => data.pensions = 100_000,
            _ => unreachable!(),
        }
        let selected = crate::section179_return::prepare_with_home(
            &draft,
            Section179IncomeSystem::Regular,
            &crate::home_office::HomeContext::default(),
        );
        let mut result = FederalResult::empty(draft.revision);
        match selected {
            Ok(selected) => {
                assert_eq!(
                    selected.allocation.income.business_income_limit_cents,
                    if boundary == "ptp" { 6_500_000 } else { 6_700_000 }
                );
                if let Err(issue) = selected.report(&mut result) {
                    issue.report(&mut result);
                }
            }
            Err(issue) => issue.report(&mut result),
        }
        assert!(
            result.issues.iter().any(|issue| issue.display.key.contains(
                if boundary == "pension" { "OWNER_DEDUCTIONS" } else { "GUARANTEED_PAYMENTS" }
            )),
            "{boundary}: {result:?}"
        );
    }
}

#[test]
fn unit17_partner_services_actual_save_reopen_restart_and_correction() {
    let mut session = partner_service179_session(200_049, 0);
    let original = session.draft.clone();
    let result = supported(&original);
    assert_eq!(line_amount(&result, "section179_business_income_limit"), 6_700_049);
    assert_eq!(line_amount(&result, "section179_deduction"), 6_700_049);
    session = restart(session.draft);
    let mut reopened = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut reopened else { unreachable!() };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.services, 200_049);
    assert_eq!(page.capital, 0);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, original);
    assert_eq!(estimate(&session.draft), result);
    let mut correction = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.services = 0;
    accepted(&send(&mut session, correction));
    let corrected = supported(&session.draft);
    assert_eq!(line_amount(&corrected, "section179_deduction"), 6_500_000);
    assert_eq!(value(&corrected, "section179_carryforward"), 5_000);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
}

#[test]
fn unit17_partner_services_income_overflow_stays_unavailable() {
    let mut draft = partner_service179_session(200_000, 0).draft;
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    // Exercise the native API failure boundary, independently of Save's
    // tighter source bounds; do not panic or turn an overflow into zero income.
    data.guaranteed_services = i64::MAX;
    assert_eq!(
        crate::section179::calculate_with_home(&draft, &crate::home_office::HomeContext::default()),
        Err(crate::Section179PreparedIssue::AmountOverflow(id)),
    );
    assert!(!crate::section179::calculate(&draft).income_known);
    let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.guaranteed_services = 200_000;
    // Externally supplied class amounts use the checked prepared interface.
    // The ordinary amount fits Cents by itself; adding the actual services
    // does not. No out-of-bounds stored payment is needed for this case.
    assert_eq!(
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            Section179IncomeSystem::Regular,
            &[],
            &[section179_income_source(id, i64::MAX, 0, 0, 0)],
            |_, _, _| Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Regular,
                amount_cents: 0,
            }),
        ),
        Err(crate::Section179PreparedIssue::AmountOverflow(id)),
    );
}

#[test]
fn unit17_partner_services_range_refusal_is_located_and_atomic() {
    let mut session = partner_service179_session(200_049, 0);
    let saved = session.draft.clone();
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.services = 9_000_000_000_000_000;
    for request in [estimate_documents_from_save(&request), request] {
        let Reply::Response(P::Problem { field, .. }) = send(&mut session, request) else {
            panic!("range refusal")
        };
        assert_eq!(field.as_deref(), Some("[documents/documents/0/document/services]"));
        assert_eq!(session.draft, saved);
    }
    assert_eq!(restart(saved.clone()).draft, saved);
}
