// Independent normal-lifecycle witnesses for the approved one-business K1
// aggregation source increment. Monetary source fixture is the surviving
// unit22 last-gap control: A QBI100000/wages0, B QBI25000/wages120000.
// This is native protocol acceptance, not a fresh Studio client execution.
extern crate std;
use super::*;
use crate::tests::support::builders::k1res::qbi_aggregation_facts as facts;

fn base() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 40_000_000, 0);
    d
}

fn source(name: &str, profit: i64, wages: i64) -> IncomeDocumentItem {
    let mut p = k1_1120s_fixture(profit);
    p.name = name.into();
    if let K1ScorpQbi::Statement { wages: printed, .. } = &mut p.qbi {
        *printed = wages * 100;
    }
    p.qbi_group.clear();
    document(0, IncomeDocument::K1Scorp(p))
}

fn documents_mut(page: &mut P) -> &mut IncomeDocuments {
    let P::SaveIncomes { documents, .. } = page else { panic!("expected normal income editor") };
    documents
}

fn inputs(page: &IncomeDocuments) -> IncomeDocumentsEstimateInput {
    IncomeDocumentsEstimateInput {
        receipt_context: page.receipt_context.clone(),
        documents: page
            .documents
            .iter()
            .map(|r| IncomeDocumentEstimateInput {
                id: r.id,
                owner: r.owner,
                document: r.document.clone(),
                community: r.community,
                refund_management: r.refund_management,
            })
            .collect(),
        qbi_aggregations: page
            .qbi_aggregations
            .iter()
            .map(|g| QbiAggregationEstimateInput {
                facts: QbiAggregationFactsEstimateInput {
                    name: g.facts.name.clone(),
                    common_ownership: g.facts.common_ownership,
                    majority_year_last_day: g.facts.majority_year_last_day,
                    same_tax_year: g.facts.same_tax_year,
                    same_or_customary_products: g.facts.same_or_customary_products,
                    shared_facilities_or_elements: g.facts.shared_facilities_or_elements,
                    coordinated_or_interdependent: g.facts.coordinated_or_interdependent,
                    election: g.facts.election.clone(),
                    irs_history: g.facts.irs_history.clone(),
                    disclosure: g
                        .facts
                        .disclosure
                        .iter()
                        .map(|r| QbiBusinessDisclosureEstimateInput {
                            source: r.source.clone(),
                            entity_name: r.entity_name.clone(),
                            ein: r.ein.clone(),
                            business_description: r.business_description.clone(),
                            changes: r.changes.clone(),
                        })
                        .collect(),
                },
            })
            .collect(),
    }
}

// Copy the genuine ordinary editor inputs; exclude the derived source catalog
// from the reduced calculation request. This helper never changes tax facts.
fn calculation(page: &P) -> P {
    let P::SaveIncomes { user, documents, .. } = page else {
        panic!("expected normal income editor")
    };
    let mut request = crate::tests::support::client::estimate_documents_from_save(page);
    let P::EstimateDocuments { user: request_user, documents: request_documents, .. } =
        &mut request
    else {
        unreachable!()
    };
    *request_user = *user;
    *request_documents = inputs(documents);
    request
}

fn refresh(s: &mut Interview, editor: &mut P) {
    let before = s.draft.clone();
    let reply = send(s, calculation(editor));
    let Reply::Response(P::IncomeRunningEstimate {
        estimate,
        warnings,
        progress,
        groups,
        sources,
        ..
    }) = reply
    else {
        panic!("expected ordinary reduced income callback, got {reply:?}")
    };
    let P::SaveIncomes {
        documents,
        qbi_sources,
        estimate: text,
        warnings: notes,
        progress: walk,
        ..
    } = editor
    else {
        unreachable!()
    };
    assert_eq!(groups.len(), documents.qbi_aggregations.len());
    for (g, derived) in documents.qbi_aggregations.iter_mut().zip(groups) {
        g.members = derived.members;
        g.disclosure_sources = derived.disclosure_sources;
        g.businesses = derived.businesses;
    }
    *qbi_sources = sources;
    *text = Some(estimate);
    *notes = warnings;
    *walk = Some(progress);
    assert_eq!(s.draft, before, "a calculation must not accept the candidate");
}

fn new_sources() -> Interview {
    let mut s = Interview::new(TEST_USER, base());
    let original_wages = s.draft.income[0].clone();
    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    documents_mut(&mut editor).documents = vec![
        source("Independent entity A", 100_000, 0),
        source("Independent entity B", 25_000, 120_000),
    ];
    assert!(page_documents(&editor).iter().all(|r| r.id == 0));
    accepted(&send(&mut s, editor));
    let reopened = open_topic(&mut s, TaxSection::PassThrough);
    let rows = page_documents(&reopened);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.id > 0));
    assert_ne!(rows[0].id, rows[1].id);
    assert!(
        rows.iter()
            .all(|r| matches!(&r.document, IncomeDocument::K1Scorp(k) if k.qbi_group.is_empty()))
    );
    assert_eq!(s.draft.income.iter().find(|e| e.id == original_wages.id), Some(&original_wages));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 525_000);
    assert_eq!(value(&r, "qbi_deduction"), 5_000);
    s
}

fn election_editor(s: &mut Interview) -> P {
    let mut editor = open_topic(s, TaxSection::PassThrough);
    let documents = documents_mut(&mut editor);
    for row in &mut documents.documents {
        assert!(row.id > 0);
        match &mut row.document {
            IncomeDocument::K1Scorp(k) => k.qbi_group = "Actual elected A+B".into(),
            IncomeDocument::K1Partnership(k) => k.qbi_group = "Actual elected A+B".into(),
            _ => panic!("actual one-business K1source"),
        }
    }
    documents.qbi_aggregations = vec![facts()];
    refresh(s, &mut editor);
    let documents = documents_mut(&mut editor);
    let g = &mut documents.qbi_aggregations[0];
    assert_eq!(g.members.len(), 2);
    assert_eq!(g.businesses, Some(2));
    assert!(
        g.members
            .iter()
            .all(|m| m.group.as_deref() == Some("Actual elected A+B") && m.service == Some(false))
    );
    g.facts.disclosure = g
        .members
        .iter()
        .enumerate()
        .map(|(i, m)| QbiBusinessDisclosurePage {
            source: m.key.clone(),
            entity_name: if i == 0 {
                "Independent entity A".into()
            } else {
                "Independent entity B".into()
            },
            ein: QbiEntityEin::Known {
                number: if i == 0 { "123456789".into() } else { "987654321".into() },
            },
            business_description: "Actual source-owned ordinary business".into(),
            changes: QbiBusinessChanges::None,
        })
        .collect();
    refresh(s, &mut editor);
    editor
}

fn assert_restores_and_resaves(s: &mut Interview) {
    let before = s.draft.clone();
    let expected = estimate(&before);
    let editor = open_topic(s, TaxSection::PassThrough);
    let saved_page = documents_mut(&mut editor.clone()).clone();
    accepted(&send(s, editor));
    assert_eq!(s.draft.income, before.income);
    assert_eq!(
        s.draft.screening.as_ref().unwrap().qbi_aggregations,
        before.screening.as_ref().unwrap().qbi_aggregations
    );
    let mut restarted = restart(s.draft.clone());
    assert_eq!(estimate(&restarted.draft), estimate(&s.draft));
    let reopened = open_topic(&mut restarted, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = reopened else { unreachable!() };
    assert_eq!(
        documents.documents.iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        saved_page.documents.iter().map(|r| r.document.clone()).collect::<Vec<_>>()
    );
    assert_eq!(documents.qbi_aggregations, saved_page.qbi_aggregations);
    let mut expected = expected;
    expected.revision = s.draft.revision;
    assert_eq!(estimate(&s.draft), expected);
}

#[test]
fn qbi_aggregation_actual_two_save_election_changes_only_proved_wage_limit_and_restores() {
    let mut s = new_sources();
    let known_sources = s.draft.income.clone();
    let editor = election_editor(&mut s);
    accepted(&send(&mut s, editor));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 525_000);
    assert_eq!(value(&r, "qbi_deduction"), 25_000);
    for original in known_sources {
        let actual = s.draft.income.iter().find(|e| e.id == original.id).unwrap();
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.owner, original.owner);
        if let (FederalIncome::Scorp { data: old }, FederalIncome::Scorp { data: new }) =
            (&original.income, &actual.income)
        {
            let mut expected = old.clone();
            expected.qbi_aggregation_group = Some("Actual elected A+B".into());
            assert_eq!(*new, expected);
        } else {
            assert_eq!(actual, &original);
        }
    }
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_structural_source_and_readonly_forgery_reject_atomically() {
    let mut s = new_sources();
    let genuine = election_editor(&mut s);
    for bad in 0..13 {
        let mut editor = genuine.clone();
        let P::SaveIncomes { documents, qbi_sources, .. } = &mut editor else { unreachable!() };
        let g = &mut documents.qbi_aggregations[0];
        match bad {
            0 => g.facts.disclosure[0].source = "income:0".into(),
            1 => g.facts.disclosure.push(g.facts.disclosure[0].clone()),
            2 => {
                g.members.pop();
            }
            3 => g.members[0].owner = Owner::Spouse,
            4 => g.members[0].service = Some(true),
            5 => g.businesses = Some(3),
            6 => {
                qbi_sources.pop();
            }
            7 => qbi_sources[0].key = "forged-source".into(),
            8 => g.members[0].origin = QbiSourceOrigin::PartnershipBusiness,
            9 => g.members.push(g.members[0].clone()),
            10 => {
                g.disclosure_sources.pop();
            }
            11 => g.disclosure_sources[0].owner = Owner::Spouse,
            12 => g.disclosure_sources.push(g.disclosure_sources[0].clone()),
            _ => unreachable!(),
        }
        let before = s.draft.clone();
        rejected_with(send(&mut s, editor), "QBI_AGGREGATION_SOURCES");
        assert_eq!(s.draft, before, "negative {bad} accepted any source mutation");
    }
    accepted(&send(&mut s, genuine));
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
}

#[test]
fn qbi_aggregation_unfavorable_or_unavailable_election_facts_save_without_silent_disaggregation() {
    for case in 0..5 {
        let mut s = new_sources();
        let mut editor = election_editor(&mut s);
        let facts = &mut documents_mut(&mut editor).qbi_aggregations[0].facts;
        match case {
            0 => facts.common_ownership = false,
            1 => facts.same_tax_year = false,
            2 => facts.irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2023 },
            3 => facts.election = QbiElectionHistory::FirstOnAmendedReturn,
            4 => facts.irs_history = QbiIrsDisaggregationHistory::Unavailable,
            _ => unreachable!(),
        }
        let expected = facts.clone();
        accepted(&send(&mut s, editor));
        let reopened = open_topic(&mut s, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = reopened else { unreachable!() };
        assert_eq!(documents.qbi_aggregations[0].facts, expected);
        assert!(documents.documents.iter().all(|r| matches!(&r.document, IncomeDocument::K1Scorp(k) if k.qbi_group == "Actual elected A+B")));
        let result = estimate(&s.draft);
        assert_eq!(result.state, ResultState::Incomplete);
        assert!(!result.lines.iter().any(|row| row.id == "qbi_deduction"));
        assert!(
            result.issues.iter().any(|i| i.display.key == "QBI_AGGREGATION_FACTS"),
            "{:?}",
            result.issues
        );
        assert_restores_and_resaves(&mut s);
    }
}

#[test]
fn qbi_aggregation_malformed_restored_facts_reject_without_panic_or_tax_output() {
    let mut s = new_sources();
    let editor = election_editor(&mut s);
    accepted(&send(&mut s, editor));
    assert!(s.draft.valid_stored());
    for case in 0..8 {
        let mut restored = s.draft.clone();
        let facts = &mut restored.screening.as_mut().unwrap().qbi_aggregations;
        match case {
            0 => {
                facts[0].irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: u32::MAX }
            }
            1 => facts[0].irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2017 },
            2 => facts[0].irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2027 },
            3 => facts[0].election = QbiElectionHistory::Continuing { original_year: 2026 },
            4 => facts.push(facts[0].clone()),
            5 => {
                let duplicate = facts[0].disclosure[0].clone();
                facts[0].disclosure.push(duplicate);
            }
            6 => facts[0].disclosure[0].ein = QbiEntityEin::Known { number: "12345678".into() },
            7 => facts[0].disclosure[0].ein = QbiEntityEin::Known { number: "1234x6789".into() },
            _ => unreachable!(),
        }
        assert!(!restored.valid_stored(), "malformed restored case {case}");
        let unchanged = restored.clone();
        let result = std::panic::catch_unwind(|| estimate(&restored)).expect(
            "malformed restored aggregation must return its source issue without panicking",
        );
        assert_eq!(result.state, ResultState::Incomplete, "case {case}");
        assert!(
            result.issues.iter().any(|issue| issue.display.key == "QBI_AGGREGATION_SOURCES"),
            "case {case}: {:?}",
            result.issues
        );
        assert!(!result.lines.iter().any(|row| row.id == "qbi_deduction"));
        assert_eq!(restored, unchanged, "estimate rewrote restored source case {case}");
    }
    // Unavailable is genuine retained source data, not a malformed year/EIN.
    let mut unavailable = s.draft.clone();
    unavailable.screening.as_mut().unwrap().qbi_aggregations[0].irs_history =
        QbiIrsDisaggregationHistory::Unavailable;
    assert!(unavailable.valid_stored());
    let result = estimate(&unavailable);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.issues.iter().any(|issue| issue.display.key == "QBI_AGGREGATION_FACTS"));
    assert!(!result.issues.iter().any(|issue| issue.display.key == "QBI_AGGREGATION_SOURCES"));
    assert!(!result.lines.iter().any(|row| row.id == "qbi_deduction"));
}

#[test]
fn qbi_aggregation_irs2022_three_subsequent_years_expire_before2026() {
    let mut s = new_sources();
    let mut editor = election_editor(&mut s);
    documents_mut(&mut editor).qbi_aggregations[0].facts.irs_history =
        QbiIrsDisaggregationHistory::Disaggregated { year: 2022 };
    accepted(&send(&mut s, editor));
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
    assert_restores_and_resaves(&mut s);
}

fn set_group(editor: &mut P, name: &str) {
    for row in &mut documents_mut(editor).documents {
        match &mut row.document {
            IncomeDocument::K1Scorp(k) => k.qbi_group = name.into(),
            IncomeDocument::K1Partnership(k) => k.qbi_group = name.into(),
            _ => panic!("expected genuine K1 source"),
        }
    }
}

#[test]
fn qbi_aggregation_inactive_history_uses_live_disclosure_catalog_without_current_members() {
    let mut s = new_sources();
    let mut editor = election_editor(&mut s);
    set_group(&mut editor, "");
    documents_mut(&mut editor).qbi_aggregations[0].facts.irs_history =
        QbiIrsDisaggregationHistory::Disaggregated { year: 2023 };
    refresh(&mut s, &mut editor);
    let group = &documents_mut(&mut editor).qbi_aggregations[0];
    assert!(group.members.is_empty());
    assert_eq!(group.businesses, Some(0));
    assert_eq!(group.disclosure_sources.len(), 2);
    assert_eq!(group.facts.disclosure.len(), 2);
    accepted(&send(&mut s, editor));
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 5_000);
    assert_restores_and_resaves(&mut s);

    // A second label does not erase retained IRS restrictions associated with
    // these actual typed sources. Unrelated incomplete metadata stays harmless.
    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    let mut current = documents_mut(&mut editor).qbi_aggregations[0].clone();
    current.facts.name = "Renamed current election".into();
    current.facts.irs_history = QbiIrsDisaggregationHistory::None;
    documents_mut(&mut editor).qbi_aggregations.push(current);
    set_group(&mut editor, "Renamed current election");
    refresh(&mut s, &mut editor);
    accepted(&send(&mut s, editor));
    let result = estimate(&s.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(!result.lines.iter().any(|r| r.id == "qbi_deduction"));
    assert!(result.issues.iter().any(|i| i.display.key == "QBI_AGGREGATION_FACTS"));
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_explicit_history_correction_and_rename_share_one_normal_save() {
    let mut s = new_sources();
    let mut editor = election_editor(&mut s);
    documents_mut(&mut editor).qbi_aggregations[0].facts.irs_history =
        QbiIrsDisaggregationHistory::Disaggregated { year: 2023 };
    accepted(&send(&mut s, editor));
    assert_eq!(estimate(&s.draft).state, ResultState::Incomplete);

    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    set_group(&mut editor, "Corrected and renamed");
    let facts = &mut documents_mut(&mut editor).qbi_aggregations[0].facts;
    facts.name = "Corrected and renamed".into();
    facts.irs_history = QbiIrsDisaggregationHistory::None;
    refresh(&mut s, &mut editor);
    accepted(&send(&mut s, editor));
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_rename_alone_preserves_known_and_unavailable_irs_history() {
    for history in [
        QbiIrsDisaggregationHistory::Disaggregated { year: 2023 },
        QbiIrsDisaggregationHistory::Unavailable,
    ] {
        let mut s = new_sources();
        let mut editor = election_editor(&mut s);
        documents_mut(&mut editor).qbi_aggregations[0].facts.irs_history = history.clone();
        accepted(&send(&mut s, editor));
        let mut editor = open_topic(&mut s, TaxSection::PassThrough);
        set_group(&mut editor, "Same businesses renamed");
        documents_mut(&mut editor).qbi_aggregations[0].facts.name =
            "Same businesses renamed".into();
        refresh(&mut s, &mut editor);
        accepted(&send(&mut s, editor));
        assert_eq!(s.draft.screening.as_ref().unwrap().qbi_aggregations[0].irs_history, history);
        assert_eq!(estimate(&s.draft).state, ResultState::Incomplete);
        assert_restores_and_resaves(&mut s);
    }
}

#[test]
fn qbi_aggregation_dropping_history_with_live_sources_rejects_atomically() {
    for case in 0..3 {
        let mut s = new_sources();
        let mut editor = election_editor(&mut s);
        let facts = &mut documents_mut(&mut editor).qbi_aggregations[0].facts;
        match case {
            0 => facts.irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2023 },
            1 => facts.irs_history = QbiIrsDisaggregationHistory::Unavailable,
            2 => facts.election = QbiElectionHistory::Continuing { original_year: 2020 },
            _ => unreachable!(),
        }
        accepted(&send(&mut s, editor));
        let before = s.draft.clone();
        let mut editor = open_topic(&mut s, TaxSection::PassThrough);
        set_group(&mut editor, "");
        documents_mut(&mut editor).qbi_aggregations.clear();
        rejected_with(send(&mut s, editor), "QBI_AGGREGATION_HISTORY");
        assert_eq!(s.draft, before);
        assert_restores_and_resaves(&mut s);
    }
}

#[test]
fn qbi_aggregation_continuing_election_cannot_be_silently_abandoned_or_reset_by_new_label() {
    let mut s = new_sources();
    let mut editor = election_editor(&mut s);
    documents_mut(&mut editor).qbi_aggregations[0].facts.election =
        QbiElectionHistory::Continuing { original_year: 2020 };
    accepted(&send(&mut s, editor));
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    set_group(&mut editor, "");
    refresh(&mut s, &mut editor);
    accepted(&send(&mut s, editor));
    let result = estimate(&s.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.issues.iter().any(|i| i.display.key == "QBI_AGGREGATION_HISTORY"));
    assert_restores_and_resaves(&mut s);

    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    let mut current = documents_mut(&mut editor).qbi_aggregations[0].clone();
    current.facts.name = "Replacement label".into();
    current.facts.election = QbiElectionHistory::FirstTimelyOriginalReturn;
    documents_mut(&mut editor).qbi_aggregations.push(current);
    set_group(&mut editor, "Replacement label");
    refresh(&mut s, &mut editor);
    accepted(&send(&mut s, editor));
    let result = estimate(&s.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.issues.iter().any(|i| i.display.key == "QBI_AGGREGATION_HISTORY"));
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_long_utf8_disclosure_names_preserve_tax_and_cold_resave() {
    // D-2026-10-15-13 removes only the native disclosure-name maximum.
    for label in ["a".repeat(400), "é".repeat(200), "😀".repeat(100), "a\u{301}".repeat(133) + "a"]
    {
        assert_eq!(label.len(), 400);
        let mut s = new_sources();
        let mut valid = election_editor(&mut s);
        documents_mut(&mut valid).qbi_aggregations[0].facts.disclosure[0].entity_name =
            label.clone();
        accepted(&send(&mut s, valid));
        let before = s.draft.clone();
        let tax = supported(&before).tax;
        assert_eq!(value(&supported(&before), "qbi_deduction"), 25_000);
        let mut renamed = open_topic(&mut s, TaxSection::PassThrough);
        let long_name = label + "a";
        documents_mut(&mut renamed).qbi_aggregations[0].facts.disclosure[0].entity_name =
            long_name.clone();
        accepted(&send(&mut s, renamed));
        assert_eq!(s.draft.income, before.income);
        assert_eq!(
            s.draft.screening.as_ref().unwrap().qbi_aggregations[0].disclosure[0].entity_name,
            long_name
        );
        assert_eq!(supported(&s.draft).tax, tax);
        assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
        assert_restores_and_resaves(&mut s);
        let mut cold = restart(s.draft.clone());
        let mut expected = estimate(&cold.draft);
        let reopened = open_topic(&mut cold, TaxSection::PassThrough);
        accepted(&send(&mut cold, reopened));
        assert_eq!(
            cold.draft.screening.as_ref().unwrap().qbi_aggregations,
            s.draft.screening.as_ref().unwrap().qbi_aggregations
        );
        expected.revision = cold.draft.revision;
        assert_eq!(estimate(&cold.draft), expected);
    }
}

#[test]
fn qbi_aggregation_source_group_accepts_long_names_and_locates_duplicates() {
    for label in ["a".repeat(40), "é".repeat(20), "😀".repeat(10), "a\u{301}".repeat(13) + "a"] {
        assert_eq!(label.len(), 40);
        let mut s = new_sources();
        let mut valid = election_editor(&mut s);
        documents_mut(&mut valid).qbi_aggregations[0].facts.name = label.clone();
        set_group(&mut valid, &label);
        refresh(&mut s, &mut valid);
        accepted(&send(&mut s, valid));
        assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
        // D-2026-10-15-13 removes the arbitrary 40-byte name cap. Rename
        // the actual election and its selected sources together, then refresh
        // their derived membership through the ordinary calculation request.
        let long_name = label + "a";
        let before = s.draft.clone();
        let dirty = s.dirty;
        // Changing only the source label leaves submitted membership stale.
        // Save must still reject that actual relationship contradiction.
        let mut stale = open_topic(&mut s, TaxSection::PassThrough);
        set_group(&mut stale, &long_name);
        let Reply::Response(P::Problem { message, field }) = send(&mut s, stale) else {
            panic!("expected stale derived membership refusal")
        };
        assert_eq!(message, libertas::libertas_formatted_text("QBI_AGGREGATION_SOURCES", &[]));
        assert_eq!(field.as_deref(), Some("[documents/qbi_aggregations/0/members]"));
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
        let mut renamed = open_topic(&mut s, TaxSection::PassThrough);
        documents_mut(&mut renamed).qbi_aggregations[0].facts.name = long_name.clone();
        set_group(&mut renamed, &long_name);
        refresh(&mut s, &mut renamed);
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
        accepted(&send(&mut s, renamed));
        assert_eq!(s.draft.screening.as_ref().unwrap().qbi_aggregations[0].name, long_name);
        assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
        assert_restores_and_resaves(&mut s);
        let before = s.draft.clone();
        let dirty = s.dirty;
        let mut invalid = open_topic(&mut s, TaxSection::PassThrough);
        let groups = &mut documents_mut(&mut invalid).qbi_aggregations;
        groups.push(groups[0].clone());
        for request in [invalid.clone(), calculation(&invalid)] {
            let request = P::from_avro(&request.to_avro()).unwrap();
            let Reply::Response(P::Problem { message, field }) = send(&mut s, request) else {
                panic!("expected duplicate actual group refusal")
            };
            assert_eq!(message, libertas::libertas_formatted_text("QBI_AGGREGATION_SOURCES", &[]));
            assert_eq!(field.as_deref(), Some("[documents/qbi_aggregations/1/facts/name]"));
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        }
    }
}

#[test]
fn qbi_aggregation_new_direct_source_with_election_facts_uses_normal_two_save_identity() {
    let mut s = new_sources();
    let mut editor = open_topic(&mut s, TaxSection::SelfEmployment);
    let mut page = business(1_000_000);
    page.more.qbi_group = "Actual elected A+B".into();
    documents_mut(&mut editor).documents = vec![document(0, IncomeDocument::Business(page))];
    documents_mut(&mut editor).qbi_aggregations = vec![facts()];
    let before = s.draft.clone();
    rejected_with(send(&mut s, editor.clone()), "QBI_AGGREGATION_ACCEPT_SOURCE_FIRST");
    assert_eq!(s.draft, before);
    let IncomeDocument::Business(source) = &mut documents_mut(&mut editor).documents[0].document
    else {
        unreachable!()
    };
    source.more.qbi_group.clear();
    refresh(&mut s, &mut editor);
    accepted(&send(&mut s, editor));
    let mut editor = open_topic(&mut s, TaxSection::SelfEmployment);
    let id = documents_mut(&mut editor).documents[0].id;
    assert!(id > 0);
    let IncomeDocument::Business(source) = &mut documents_mut(&mut editor).documents[0].document
    else {
        unreachable!()
    };
    source.more.qbi_group = "Actual elected A+B".into();
    refresh(&mut s, &mut editor);
    let group = &documents_mut(&mut editor).qbi_aggregations[0];
    assert_eq!(group.members.len(), 1);
    assert_eq!(group.members[0].source, QbiAggregationSource::Income { document: id });
    assert_eq!(group.businesses, Some(1));
    accepted(&send(&mut s, editor));
    assert_eq!(estimate(&s.draft).state, ResultState::Incomplete);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

#[test]
fn qbi_aggregation_opposite_topic_save_preserves_global_election_and_actual_k1_sources() {
    let mut s = new_sources();
    let editor = election_editor(&mut s);
    accepted(&send(&mut s, editor));
    let pass_through = s
        .draft
        .income
        .iter()
        .filter(|e| matches!(&e.income, FederalIncome::Scorp { .. }))
        .cloned()
        .collect::<Vec<_>>();
    let election = s.draft.screening.as_ref().unwrap().qbi_aggregations.clone();
    let mut wages = open_topic(&mut s, TaxSection::Wages);
    assert_eq!(documents_mut(&mut wages).qbi_aggregations.len(), 1);
    let IncomeDocument::W2(w2) = &mut documents_mut(&mut wages).documents[0].document else {
        unreachable!()
    };
    w2.withholding += 100;
    refresh(&mut s, &mut wages);
    accepted(&send(&mut s, wages));
    assert_eq!(
        s.draft
            .income
            .iter()
            .filter(|e| matches!(&e.income, FederalIncome::Scorp { .. }))
            .cloned()
            .collect::<Vec<_>>(),
        pass_through
    );
    assert_eq!(s.draft.screening.as_ref().unwrap().qbi_aggregations, election);
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_new_source_cannot_elect_before_ordinary_id_assignment() {
    let mut s = Interview::new(TEST_USER, base());
    let mut editor = open_topic(&mut s, TaxSection::PassThrough);
    let mut row = source("Unsaved actual source", 100_000, 0);
    let IncomeDocument::K1Scorp(k) = &mut row.document else { unreachable!() };
    k.qbi_group = "Premature group".into();
    documents_mut(&mut editor).documents = vec![row];
    let before = s.draft.clone();
    rejected_with(send(&mut s, editor), "QBI_AGGREGATION_ACCEPT_SOURCE_FIRST");
    assert_eq!(s.draft, before);
}

fn client_case(editor: &mut P, case: &str) {
    let facts = &mut documents_mut(editor).qbi_aggregations[0].facts;
    match case {
        "eligible" => {}
        "ownership_no" => facts.common_ownership = false,
        "irs2023" => facts.irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2023 },
        "irs2022" => facts.irs_history = QbiIrsDisaggregationHistory::Disaggregated { year: 2022 },
        _ => panic!("unknown actual-client case {case}"),
    }
}

fn write_client_artifact(variable: &str, bytes: &[u8]) {
    if let Ok(path) = std::env::var(variable) {
        std::fs::write(path, bytes).unwrap();
    }
}

// Artifact bridge only. The unconditioned numerical/lifecycle tests above are
// independent native acceptance. This test claims actualclient coverage only
// after a fresh real editor emits the complete payload and this bridge consumes
// it. It never provides calculated catalogs as editable source answers.
#[test]
fn qbi_aggregation_actual_client_artifact_bridge() {
    let Ok(case) = std::env::var("TAX_QBI_AGGREGATION_CASE") else {
        return;
    };
    let mut s = new_sources();
    let seed = open_topic(&mut s, TaxSection::PassThrough);
    let mut answers = election_editor(&mut s);
    client_case(&mut answers, &case);
    refresh(&mut s, &mut answers);
    write_client_artifact("TAX_QBI_AGGREGATION_SEED", &seed.to_avro());
    write_client_artifact("TAX_QBI_AGGREGATION_ANSWERS", &answers.to_avro());
    if let Ok(input) = std::env::var("TAX_QBI_AGGREGATION_CALC_INPUT") {
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let before = s.draft.clone();
        let reply = send(&mut s, request);
        let Reply::Response(response) = reply else {
            panic!("normal callback required, got {reply:?}")
        };
        assert!(!matches!(response, P::Problem { .. }), "normal callback failed: {response:?}");
        assert_eq!(s.draft, before);
        write_client_artifact("TAX_QBI_AGGREGATION_CALC_OUTPUT", &response.to_avro());
        return;
    }
    let Ok(payload) = std::env::var("TAX_QBI_AGGREGATION_PAYLOAD") else {
        return;
    };
    let payload = std::path::Path::new(&payload);
    if !payload.exists() {
        return;
    }
    let request = P::from_avro(&std::fs::read(payload).unwrap()).unwrap();
    assert!(matches!(request, P::SaveIncomes { .. }));
    accepted(&send(&mut s, request));
    let result = estimate(&s.draft);
    if matches!(case.as_str(), "eligible" | "irs2022") {
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
        assert_eq!(value(&result, "agi"), 525_000);
        assert_eq!(value(&result, "qbi_deduction"), 25_000);
    } else {
        assert_eq!(result.state, ResultState::Incomplete);
        assert!(!result.lines.iter().any(|row| row.id == "qbi_deduction"));
        assert!(
            result.issues.iter().any(|i| i.display.key == "QBI_AGGREGATION_FACTS"),
            "{:?}",
            result.issues
        );
    }
    let reopened = open_topic(&mut s, TaxSection::PassThrough);
    let expected_facts = documents_mut(&mut answers).qbi_aggregations[0].facts.clone();
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(documents.qbi_aggregations[0].facts, expected_facts);
    write_client_artifact("TAX_QBI_AGGREGATION_REOPEN", &reopened.to_avro());
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_one_business_partnership_and_scorp_use_the_same_election_owner_once() {
    let mut p = k1_1065("Independent entity B");
    p.ordinary = 2_500_000;
    p.interest = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Over500Hours { hours: 600 },
    };
    p.basis.start = k1_1065_continuing(
        1_000_000,
        ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
    );
    p.qbi = PartnerQbi::Statement {
        income: 2_500_000,
        wages: 12_000_000,
        property: 0,
        service: false,
        section1231_qbi: 0,
        reit_dividends: 0,
        ptp_income: 0,
    };
    p.qbi_group.clear();
    let mut s = Interview::new(TEST_USER, base());
    let mut first = open_topic(&mut s, TaxSection::PassThrough);
    documents_mut(&mut first).documents = vec![
        source("Independent entity A", 100_000, 0),
        document(0, IncomeDocument::K1Partnership(p)),
    ];
    accepted(&send(&mut s, first));
    assert_eq!(value(&supported(&s.draft), "agi"), 525_000);
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 5_000);
    let before = s
        .draft
        .income
        .iter()
        .find(|e| matches!(&e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .clone();
    let editor = election_editor(&mut s);
    accepted(&send(&mut s, editor));
    assert_eq!(value(&supported(&s.draft), "agi"), 525_000);
    assert_eq!(value(&supported(&s.draft), "qbi_deduction"), 25_000);
    let actual = s.draft.income.iter().find(|e| e.id == before.id).unwrap();
    let FederalIncome::Partnership { data: old } = before.income else { unreachable!() };
    let FederalIncome::Partnership { data: new } = &actual.income else { unreachable!() };
    let mut expected = old;
    expected.qbi_aggregation_group = Some("Actual elected A+B".into());
    assert_eq!(*new, expected);
    assert_restores_and_resaves(&mut s);
}

#[test]
fn qbi_aggregation_shared_locator_keeps_real_header_and_disclosure_positions() {
    for (case, path) in [
        (0, "[documents/qbi_aggregations/1/facts/name]"),
        (1, "[documents/qbi_aggregations/0/facts/disclosure/1/entity_name]"),
        (2, "[documents/qbi_aggregations/0/businesses]"),
    ] {
        let mut s = new_sources();
        let mut editor = election_editor(&mut s);
        let groups = &mut documents_mut(&mut editor).qbi_aggregations;
        match case {
            0 => groups.push(groups[0].clone()),
            1 => groups[0].facts.disclosure[1].entity_name.clear(),
            _ => groups[0].businesses = Some(3),
        }
        let before = s.draft.clone();
        let Reply::Response(P::Problem { message, field }) = send(&mut s, editor.clone()) else {
            panic!("expectedactualSave refusal")
        };
        assert_eq!(message, libertas::libertas_formatted_text("QBI_AGGREGATION_SOURCES", &[]));
        assert_eq!(field.as_deref(), Some(path));
        assert_eq!(s.draft, before);
        // Derived count is absent from the reduced input and is rebuilt, so
        // only the two actual source errors must also reject the callback.
        if case < 2 {
            let Reply::Response(P::Problem { message, field }) = send(&mut s, calculation(&editor))
            else {
                panic!("expectedactualEstimate refusal")
            };
            assert_eq!(message, libertas::libertas_formatted_text("QBI_AGGREGATION_SOURCES", &[]));
            assert_eq!(field.as_deref(), Some(path));
            assert_eq!(s.draft, before);
        }
    }
}
