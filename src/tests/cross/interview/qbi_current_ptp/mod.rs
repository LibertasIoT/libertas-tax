use crate::tests::support::client::qbi_consolidated::{
    ptp_estimate_and_save, ptp_reopen_and_cold_resave, ptp_save_prior_carry,
};
use crate::tests::support::client::{read_native_draft, write_native_draft};
extern crate std;
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save};
use super::*;

const PTP_HISTORY: &str =
    "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554";
const DOCUMENT_AMOUNTS: &str =
    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26";

use crate::tests::support::builders::current_ptp_page as ptp_page;

fn exchange(session: &mut Interview, request: P, case: &str, atomic: bool) -> Reply {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let bytes = request.to_avro();
    let request = P::from_avro(&bytes).unwrap();
    let reply = if atomic && matches!(request, P::SaveDocuments { .. }) {
        let mut candidate=restart(session.draft.clone());
        let reply=crate::tests::support::client::income_pages::save_source_fixture_request(&mut candidate,request);
        Some(if matches!(reply,Reply::Response(P::Problem{..})) {
            reply
        } else {
            send(&mut candidate,P::ReviewAccepted{user:TEST_USER})
        })
    } else if matches!(request, P::SaveDocuments { .. }) {
        Some(crate::tests::support::client::income_pages::save_source_fixture_request(session, request))
    } else { session.handle(request) };
    if let Ok(root) = std::env::var("QBI_CURRENT_PTP_CASE_DIR") {
        let directory = std::path::Path::new(&root).join(case);
        std::fs::create_dir_all(&directory).unwrap();
        let response = match &reply {
            Some(Reply::Response(value) | Reply::Edit(value)) => value,
            None => panic!("actual normal handler reply"),
        };
        for (file, data) in [
            ("request.avro", bytes),
            ("response.avro", response.to_avro()),
            ("estimate-after.avro", FederalEstimate::from(estimate(&session.draft)).to_avro()),
        ] {
            use std::io::Write;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(file))
                .unwrap()
                .write_all(&data)
                .unwrap();
        }
        write_native_draft(directory.join("draft-before.avro"), &before);
        write_native_draft(directory.join("draft-after.avro"), &session.draft);
        std::fs::write(directory.join("exchange.json"), serde_json::to_vec_pretty(
            &serde_json::json!({"pid":std::process::id(),"dirty_before":dirty,
                "dirty_after":session.dirty,"draft_unchanged":session.draft==before,
                "response_role":if matches!(reply,Some(Reply::Edit(_))) {"EditRequest"} else {"Response"},
                "scope":"Actual normal request/reply/accepted state exported before acceptance/refusal assertions. No client execution inferred."})
        ).unwrap()).unwrap();
    }
    let reply = reply.expect("actual normal reply");
    if atomic || matches!(&reply, Reply::Response(P::Problem { .. })) {
        assert_eq!(session.draft, before, "{case}: candidate wrote accepted source");
        assert_eq!(session.dirty, dirty, "{case}: candidate changed dirty state");
    }
    reply
}

#[test]
fn current_ptp_all_statuses_signed_phase_and_fixed_combined_carry() {
    // Rev. Proc.2025-32 §4.26 and the approved map's independent numeric
    // controls. These are constants, not values calculated by the owner tested.
    for (status, threshold, midpoint, end) in [
        (FederalStatus::Single, 201_750, 239_250, 276_750),
        (FederalStatus::Separate, 201_775, 239_275, 276_775),
        (FederalStatus::Joint, 403_500, 478_500, 553_500),
        (FederalStatus::Head, 201_750, 239_250, 276_750),
        (FederalStatus::Surviving, 201_750, 239_250, 276_750),
    ] {
        for (taxable, eligible) in [
            (threshold - 1, 10_000),
            (threshold, 10_000),
            (midpoint, 5_000),
            (end, 0),
            (end + 1, 0),
        ] {
            for amount in [10_000, -10_000] {
                for service in [false, true] {
                    let page = ptp_page(amount, service);
                    let owner = page.record();
                    let statement = owner.qbi.as_ref().unwrap();
                    let prepared = statement
                        .calculate(
                            &owner,
                            crate::tests::support::builders::k1_amount(0),
                            0,
                            taxable,
                            status,
                        )
                        .unwrap();
                    let expected = if service { eligible * amount.signum() } else { amount };
                    assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(expected * 100));
                    assert_eq!(prepared.historical, 0);
                    assert_eq!(prepared.current.income, 0);
                    assert_eq!(prepared.current.wages, 0);
                    assert_eq!(prepared.current.property, 0);
                    assert!(prepared.current.group.is_none());
                    let result = crate::business_rules::qbi_with_history(
                        &[],
                        taxable,
                        0,
                        status,
                        crate::business_rules::QbiOther {
                            business_loss: 0,
                            reit: crate::kernel::ratio::exact::dollars(&prepared.ptp).unwrap(),
                            reit_loss: 3_000,
                        },
                        &mut FederalResult::empty(0),
                    );
                    assert_eq!(result.business_loss, 0);
                    let (deduction, carry) = if expected > 0 {
                        (if expected == 10_000 { 1_400 } else { 400 }, 0)
                    } else {
                        (0, 3_000 - expected)
                    };
                    assert_eq!(result.deduction, deduction);
                    assert_eq!(result.reit_loss, carry);
                }
            }
        }
    }
}

#[test]
fn current_ptp_public_save_estimate_preview_and_source_lifecycle_matrix() {
    for (status, standard, points) in [
        (FederalStatus::Single, 16_100, [(201_750, 10_000), (239_250, 5_000), (276_750, 0)]),
        (FederalStatus::Separate, 16_100, [(201_775, 10_000), (239_275, 5_000), (276_775, 0)]),
        (FederalStatus::Joint, 32_200, [(403_500, 10_000), (478_500, 5_000), (553_500, 0)]),
    ] {
        for (taxable, eligible) in points {
            for amount in [10_000, -10_000] {
                let case = alloc::format!("ptp-{:?}-{taxable}-{amount}", status);
                let mut draft = complete(status);
                // Explicit synthetic wage source fixes taxable income before
                // QBI; ordinary income/loss itself is never phased for AGI.
                draft.income[0] = wage(1, Owner::Taxpayer, (taxable + standard - amount) * 100, 0);
                name_accepted_income_documents(&mut draft);
                let wage_source = draft.income[0].clone();
                let mut session = Interview::new(TEST_USER, draft);
                ptp_save_prior_carry(
                    &mut session,
                    &alloc::format!("{case}-prior-carry-save"),
                    exchange,
                );
                let page = ptp_page(amount, true);

                let request = crate::tests::support::client::income_pages::save_documents(
                    &mut session,
                    vec![document(
                        0,
                        IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
                    )],
                );
                let running_estimate = ptp_estimate_and_save(
                    &mut session,
                    request,
                    &case,
                    exchange,
                    credit_estimate_documents_from_save,
                );
                let result = supported(&session.draft);
                let signed = eligible * amount.signum();
                let deduction = if signed == 10_000 {
                    1_400
                } else if signed == 5_000 {
                    400
                } else {
                    0
                };
                let carry = if signed > 0 { 0 } else { 3_000 - signed };
                assert_eq!(value(&result, "agi"), taxable + standard);
                assert_eq!(value(&result, "qbi_taxable_income"), taxable);
                assert_eq!(value(&result, "qbi_deduction"), deduction);
                assert_eq!(value(&result, "taxable_income"), taxable - deduction);
                assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
                assert_eq!(value(&result, "reit_ptp_loss_carryforward"), carry);
                // Independent expected QBI/carry/AGI figures above establish
                // the saved financial result; the pure candidate must display
                // that same resulting balance without writing accepted state.
                assert_eq!(running_estimate, crate::interview::estimate_text(&result));
                assert_eq!(session.draft.income[0], wage_source);
                assert_eq!(session.draft.screening.as_ref().unwrap().qbi_history.reit_ptp, 300_000);
                ptp_reopen_and_cold_resave(
                    &mut session,
                    &[IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }],
                    &result,
                    &case,
                    exchange,
                );
            }
        }
    }
}

fn refusal(session: &mut Interview, request: P, case: &str, key: &str, path: &str) {
    if let Some((mut candidate, topic, path)) = crate::tests::support::client::income_pages::topic_fixture_control(session, &request, key, path) {
        refusal(&mut candidate, topic, case, key, &path);
        return;
    }
    let reply = exchange(session, request.clone(), case, true);
    let Reply::Response(problem @ P::Problem { .. }) = reply else {
        panic!("{case}: expected refusal")
    };
    let P::Problem { message, field } = &problem else { unreachable!() };
    assert_eq!(*message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(field.as_deref(), Some(path));
}

#[test]
fn current_ptp_wage_property_group_bounds_keep_actual_indices_and_atomic_corrections() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let first = ptp_page(2_000, false);
    let good = ptp_page(10_000, true);
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![
            document(0, IncomeSource::K1 { value: crate::K1Document::K1Partnership((first).into()) }),
            document(0, IncomeSource::K1 { value: crate::K1Document::K1Partnership((good).into()) }),
        ],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
    let saved = session.draft.clone();
    for field in ["wages", "property"] {
        let corrected =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let mut bad = corrected.clone();
        let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
        let crate::K1Document::K1Partnership(page) = &mut documents.common.k1.documents[1].document
        else {
            unreachable!()
        };
        let PartnerQbi::Statement { wages, property, .. } = &mut page.statements.qbi else { unreachable!() };
        if field == "wages" {
            *wages = 1
        } else {
            *property = 1
        };
        for (operation, request) in
            [("save", bad.clone()), ("estimate", credit_estimate_documents_from_save(&bad))]
        {
            refusal(
                &mut session,
                request,
                &alloc::format!("ptp-{field}-{operation}"),
                DOCUMENT_AMOUNTS,
                &alloc::format!("[documents/common/k1/documents/1/document/qbi/{field}]"),
            );
        }
        accepted(&exchange(
            &mut session,
            corrected,
            &alloc::format!("ptp-{field}-corrected-save"),
            false,
        ));
        assert_eq!(session.draft, saved);
    }
    let corrected = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let mut bad = corrected.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    assert!(documents.common.k1.documents[1].id > 0);
    let crate::K1Document::K1Partnership(page) = &mut documents.common.k1.documents[1].document
    else {
        unreachable!()
    };
    page.personal.as_mut().unwrap().qbi_group = "Cannot aggregate a PTP".into();
    for (operation, request) in
        [("save", bad.clone()), ("estimate", credit_estimate_documents_from_save(&bad))]
    {
        refusal(
            &mut session,
            request,
            &alloc::format!("ptp-group-{operation}"),
            "QBI_AGGREGATION_PARENT_NOT_BUSINESS",
            "[documents/common/k1/documents/1/document/qbi_group]",
        );
    }
    accepted(&exchange(&mut session, corrected, "ptp-group-corrected-save", false));
    assert_eq!(session.draft, saved);
    assert_eq!(value(&supported(&saved), "qbi_deduction"), 2_400);
    assert_eq!(restart(saved.clone()).draft, saved);
}

#[test]
fn current_ptp_unsupported_sources_and_separate_items_keep_named_priority() {
    // 2026-10-19: i8995 Tracking Losses permits retained ordinary losses;
    // only the independently unsupported source combinations retain this key.
    for branch in 0..4 {
        let mut owner = ptp_page(10_000, true).record();
        let mut deductions = 0;
        match branch {
            0 => deductions = 1,
            1 => owner.regular.carried.charity = 1,
            2 => owner.regular.capital_carried.short.basis = 1,
            3 => {
                // No prior/history/PAL counter flags this unsupported sale:
                // its actual current basis-held ordinary loss must do so.
                owner.disposition = Some(crate::K1Disposition::Sold { proceeds: 0 });
                owner.regular.ordinary = -100_000;
                owner.regular.basis.opening = Some(0);
                owner.regular.risk.start =
                    ScorpRiskStart::Detailed { capacity: 100_000, profit: 0, loss: 0 };
                owner.qbi.as_mut().unwrap().income = -100_000;
                assert!(owner.qbi_records.history.is_none());
                assert_eq!(
                    [
                        owner.regular.basis.prior_loss,
                        owner.regular.risk.prior_loss,
                        owner.regular.passive_loss
                    ],
                    [0; 3]
                );
                let pool = owner.pool(&owner.regular).unwrap();
                assert_eq!(
                    pool.activity.basis.suspended[crate::k1_loss::K1LossClass::Ordinary].get(),
                    100_000
                );
                assert_eq!(pool.passive, 0);
            }
            _ => unreachable!(),
        }
        owner.qbi.as_mut().unwrap().ptp_income = 100;
        assert_eq!(
            owner
                .qbi
                .as_ref()
                .unwrap()
                .calculate(
                    &owner,
                    crate::tests::support::builders::k1_amount(0),
                    deductions,
                    239_250,
                    FederalStatus::Single,
                )
                .err()
                .expect("unsupported qualified source combination")
                .0,
            PTP_HISTORY,
        );
    }
    let mut page = ptp_page(10_000, true);
    let PartnerQbi::Statement { ptp_income, .. } = &mut page.qbi else { unreachable!() };
    *ptp_income = 500_000;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
    let saved = session.draft.clone();
    let result = estimate(&saved);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result.issues.iter().any(|i| i.display.key == "ISSUE_QBI_PTP_COMPONENT_CLASSIFICATION")
    );
    let reopened = crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert_eq!(
        page_documents(&reopened)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
            .unwrap()
            .document,
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            vec![IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }]
        )[0]
    );
    accepted(&exchange(&mut session, reopened, "ptp-separate-source-resave", false));
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);
    // Keep these real reported separate items and their unknown classification.
    // The supported zero-separate-items twins are independent source fixtures
    // above; do not prune an actual amount to manufacture a complete estimate.
}

#[test]
fn current_ptp_missing_statement_keeps_income_and_distinct_source_notices() {
    for amount in [10_000, -10_000] {
        let mut page = ptp_page(amount, true);
        // Independent no-code-Z source twin. Do not remove an actual reported
        // separate amount or historical record to manufacture eligibility.
        page.qbi = PartnerQbi::NoStatement;
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
            )],
        );
        let case = alloc::format!("ptp-no-statement-{amount}");
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 60_000 + amount);
        assert_eq!(value(&result, "qbi_deduction"), 0);
        assert!(noted(
            &result,
            if amount > 0 { "WARN_K1_QBI_NO_STATEMENT" } else { "WARN_K1_QBI_LOSS_NO_STATEMENT" }
        ));
        let saved = session.draft.clone();
        let reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        assert_eq!(
            page_documents(&reopened)
                .into_iter()
                .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
                .unwrap()
                .document,
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }]
            )[0]
        );
        accepted(&exchange(&mut session, reopened, &alloc::format!("{case}-resave"), false));
        assert_eq!(session.draft, saved);
        assert_eq!(restart(saved.clone()).draft, saved);
    }
}

#[test]
fn current_ptp_sstb_keeps_rental_section179_and_other_carried_source_boundaries() {
    for branch in 0..3 {
        let mut page = ptp_page(if branch == 0 { 0 } else { 20_000 }, true);
        let key = match branch {
            0 => {
                page.rentals.real_estate = 100_000;
                page.rentals.participation = Some(crate::K1RentalParticipation::NotActive);
                // 2026-10-20: rental tax income now has its own PTP pool.
                // The rental's qualified/SSTB source remains unresolved.
                page.rentals.real_estate_qbi = Some(K1RentalQbi { wages: 0, property: 0 });
                "ISSUE_K1_RENTAL_PTP_QBI"
            }
            1 => {
                page.more.section179 = 500_000;
                // 2026-10-16 D6: no AMT income-carry source is supplied here.
                "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"
            }
            2 => {
                let BasisStart::Continuing { carried, .. } = &mut page.basis.start else {
                    unreachable!()
                };
                // 2026-10-19: actual ordinary retained losses now participate;
                // the unrelated capital-loss character still has no QBI source.
                carried.basis_short = 100_000;
                PTP_HISTORY
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
            )],
        );
        let case = alloc::format!("ptp-existing-supplier-boundary-{branch}");

        let running = exchange(
            &mut session,
            credit_estimate_documents_from_save(&request),
            &alloc::format!("{case}-estimate"),
            true,
        );
        assert!(matches!(running, Reply::Response(P::Review { .. })));
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
        let saved = session.draft.clone();
        let result = estimate(&saved);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{:?}", result.issues);
        let reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        assert_eq!(
            page_documents(&reopened)
                .into_iter()
                .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
                .unwrap()
                .document,
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }]
            )[0]
        );
        accepted(&exchange(&mut session, reopened, &alloc::format!("{case}-resave"), false));
        assert_eq!(session.draft, saved);
        assert_eq!(estimate(&restart(saved.clone()).draft), result);
    }
}

#[test]
fn current_ptp_actual_request_native_bridge() {
    let Ok(request_file) = std::env::var("QBI_CURRENT_PTP_BRIDGE_REQUEST") else {
        return;
    };
    let draft = read_native_draft(std::env::var("QBI_CURRENT_PTP_BRIDGE_DRAFT").unwrap());
    let request = P::from_avro(&std::fs::read(request_file).unwrap()).unwrap();
    let mut session = Interview::new(TEST_USER, draft);
    let case = std::env::var("QBI_CURRENT_PTP_BRIDGE_CASE").unwrap_or_else(|_| {
        alloc::format!("ptp-client-actual-native-request-{}", std::process::id())
    });
    let reply = exchange(&mut session, request, &case, false);
    let (Reply::Response(value) | Reply::Edit(value)) = reply;
    std::fs::write(std::env::var("QBI_CURRENT_PTP_BRIDGE_RESPONSE").unwrap(), value.to_avro())
        .unwrap();
    write_native_draft(std::env::var("QBI_CURRENT_PTP_BRIDGE_AFTER").unwrap(), &session.draft);
}

// Keep the existing acceptance harness's filter and environment contract. These
// are captured FederalDraft bytes and normal keyed records, never a storage writer.
#[test]
fn scratch_accept_convert_draft() {
    let Ok(records_in) = std::env::var("SCRATCH_ACCEPT_RECORDS_IN") else {
        return;
    };
    let draft_input = std::env::var("SCRATCH_ACCEPT_DRAFT_AVRO")
        .ok()
        .map(|input| (input, std::env::var("SCRATCH_ACCEPT_DRAFT_RECORDS").unwrap()));
    convert_acceptance_draft(
        draft_input
            .as_ref()
            .map(|(input, output)| (std::path::Path::new(input), std::path::Path::new(output))),
        std::path::Path::new(&records_in),
        std::path::Path::new(&std::env::var("SCRATCH_ACCEPT_AVRO_OUT").unwrap()),
        std::path::Path::new(&std::env::var("SCRATCH_ACCEPT_REVISION_OUT").unwrap()),
    );
}

fn convert_acceptance_draft(
    draft_input: Option<(&std::path::Path, &std::path::Path)>,
    records_in: &std::path::Path,
    avro_out: &std::path::Path,
    revision_out: &std::path::Path,
) {
    if let Some((input, output)) = draft_input {
        let bytes = std::fs::read(input).unwrap();
        let draft = FederalDraft::from_avro(&bytes).unwrap();
        write_native_draft(output, &draft);
        assert_eq!(read_native_draft(output).to_avro(), bytes, "captured draft round trip");
    }
    let draft = read_native_draft(records_in);
    std::fs::write(avro_out, draft.to_avro()).unwrap();
    std::fs::write(revision_out, alloc::format!("{}", draft.revision)).unwrap();
}

#[test]
fn acceptance_draft_converter_preserves_native_records() {
    use crate::tests::support::client::{native_draft_records, read_native_draft_records};

    let directory = std::env::temp_dir()
        .join(alloc::format!("tax-acceptance-draft-converter-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let input = directory.join("capture.avro");
    let records = directory.join("records");
    let output = directory.join("roundtrip.avro");
    let revision = directory.join("revision");
    let mut draft = FederalDraft::empty("acceptance-native-identity".into());
    draft.next_id = 9_007_199_254_740_993;
    draft.revision = 17;
    // Unanswered optional sections stay absent; conversion does not fill facts.
    let bytes = draft.to_avro();
    std::fs::write(&input, &bytes).unwrap();
    convert_acceptance_draft(Some((&input, &records)), &records, &output, &revision);
    let normal_records = native_draft_records(&draft);
    assert_eq!(read_native_draft_records(&records), normal_records);
    assert_eq!(read_native_draft(&records), draft);
    assert_eq!(std::fs::read(&output).unwrap(), bytes);
    assert_eq!(std::fs::read_to_string(&revision).unwrap(), "17");

    // The inverse-only mode is used when comparing a client-saved return.
    std::fs::remove_file(&input).unwrap();
    std::fs::remove_file(&output).unwrap();
    std::fs::remove_file(&revision).unwrap();
    convert_acceptance_draft(None, &records, &output, &revision);
    assert_eq!(std::fs::read(&output).unwrap(), bytes);
    assert_eq!(std::fs::read_to_string(&revision).unwrap(), "17");
    assert_eq!(read_native_draft_records(&records), normal_records);
    std::fs::remove_dir_all(directory).unwrap();
}

// CFR1.199A-3(b)(1)(iv)/(c)(3)(ii), i8995 Tracking Losses and
// i8582 PTP instructions: 2017 financial losses still consume FIFO room;
// only the later retained qualified fraction reaches the REIT/PTP category.
#[test]
fn retained_ptp_fifo_loss_year_sstb_and_no_statement_survive_native_corrections() {
    for mode in 0..3 {
        let case = alloc::format!("ptp-retained-fifo-{mode}");
        let mut draft = complete(FederalStatus::Single);
        // PTP1500 profit is fully offset by its own1500 prior financial
        // loss. REIT2000 is taxable, so wage314100 fixes preQBI at300000.
        draft.income[0] = wage(1, Owner::Taxpayer, 31_410_000, 0);
        let mut session = Interview::new(TEST_USER, draft);
        let dividends = {
            let mut source = crate::DividendSource::printed(dividend_document(Div1099 {
             section199a: 200_000,

            ..native_div1099("Synthetic retained-loss REIT".into(), 200_000, 0)
            }));
            source.reit_eligible = Some(true.into());
            dividend_document(source)
        };
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividends),
                },
            )],
        );
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
        let mut page = ptp_page(1_500, mode == 1);
        (page.participation, page.limited_for_se) = (Some((LimitedParticipation::Investor { prior_years: 0 }).record()), Some(true));
        let BasisStart::Continuing { carried, .. } = &mut page.basis.start else { unreachable!() };
        carried.passive_loss = 300_000;
        page.qbi_records.as_mut().unwrap().history = Some(PartnerQbiHistory {
            basis: vec![],
            risk: vec![],
            // Reverse input order deliberately: source year owns FIFO.
            passive: vec![
                loss_year(2023, 2_000, 1_000, 2_000, 1_000),
                loss_year(2017, 1_000, 0, 1_000, 0),
            ],
        });
        if mode == 2 {
            page.qbi = PartnerQbi::NoStatement;
        }
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1Partnership((page.clone()).into()) },
            )],
        );
        let displayed = ptp_estimate_and_save(
            &mut session,
            request,
            &case,
            exchange,
            credit_estimate_documents_from_save,
        );
        let result = supported(&session.draft);
        // FIFO releases1000 nonqualified +500 at50%=250 qualified.
        // Current nonSSTB1500 survives; SSTB above276750 or no statement
        // contributes0 current, without rephasing the historical250.
        let (category, deduction) = if mode == 0 { (3_250, 650) } else { (1_750, 350) };
        assert_eq!(value(&result, "agi"), 316_100);
        assert_eq!(value(&result, "qbi_taxable_income"), 300_000);
        assert_eq!(value(&result, "qbi_reit_ptp"), category);
        assert_eq!(value(&result, "qbi_deduction"), deduction);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
        assert_eq!(displayed, crate::interview::estimate_text(&result));
        ptp_reopen_and_cold_resave(
            &mut session,
            &[IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }],
            &result,
            &case,
            exchange,
        );
        if mode == 2 {
            let mut correction =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
            let crate::K1Document::K1Partnership(page) =
                &mut documents.common.k1.documents[0].document
            else {
                unreachable!()
            };
            let year = &mut page.personal.as_mut().unwrap().qbi_records.history.as_mut().unwrap().passive[0];
            year.qualified = 200_000;
            year.remaining_qualified = 200_000;
            let corrected_page = page.clone();
            accepted(&exchange(
                &mut session,
                correction,
                &alloc::format!("{case}-correct-qualified"),
                false,
            ));
            let result = supported(&session.draft);
            // Same financial loss/AGI, corrected source fraction100%:
            // 500 qualified released, REIT2000−500=1500; deduction300.
            assert_eq!(value(&result, "agi"), 316_100);
            assert_eq!(value(&result, "qbi_reit_ptp"), 1_500);
            assert_eq!(value(&result, "qbi_deduction"), 300);
            ptp_reopen_and_cold_resave(
                &mut session,
                &[IncomeSource::K1 { value: crate::K1Document::K1Partnership(corrected_page) }],
                &result,
                &alloc::format!("{case}-corrected"),
                exchange,
            );
        }
    }
}

#[test]
fn retained_ptp_current_loss_is_not_released_by_another_ptps_profit() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_335_000, 0);
    let mut session = Interview::new(TEST_USER, draft);
    let mut loss = ptp_page(-1_000, true);
    (loss.participation, loss.limited_for_se) = (Some((LimitedParticipation::Investor { prior_years: 0 }).record()), Some(true));
    let profit = ptp_page(2_000, false);
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![
            document(0, IncomeSource::K1 { value: crate::K1Document::K1Partnership((loss.clone()).into()) }),
            document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1Partnership((profit.clone()).into()) },
            ),
        ],
    );
    let displayed = ptp_estimate_and_save(
        &mut session,
        request,
        "ptp-retained-silos",
        exchange,
        credit_estimate_documents_from_save,
    );
    let result = supported(&session.draft);
    // Wage253350 +otherPTP2000−standard16100=239250 midpoint.
    // First PTP's financial1000 and eligible500 are both still suspended;
    // the other PTP's2000 alone produces the400 deduction.
    assert_eq!(value(&result, "agi"), 255_350);
    assert_eq!(value(&result, "qbi_taxable_income"), 239_250);
    assert_eq!(value(&result, "qbi_reit_ptp"), 2_000);
    assert_eq!(value(&result, "qbi_deduction"), 400);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    assert_eq!(displayed, crate::interview::estimate_text(&result));
    ptp_reopen_and_cold_resave(
        &mut session,
        &[
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((loss).into()) },
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((profit).into()) },
        ],
        &result,
        "ptp-retained-silos",
        exchange,
    );
}

mod precision_forensics;

mod exact_category_controls;

mod partner_deductions;
