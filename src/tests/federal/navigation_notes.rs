use super::*;
use crate::federal_model::{NoteIdentity, ReviewNote};
use alloc::string::{String, ToString};

#[test]
fn pending_household_issues_open_the_page_that_holds_the_answer() {
    let draft = estimate_issue_household();
    let result = estimate(&draft);
    let items = crate::interview::remaining_items(&draft, &result, TaxSection::Review);
    assert!(!items.is_empty());
    assert!(items.iter().any(|row| row.section == TaxSection::Education));
    assert!(items.iter().any(|row| row.section == TaxSection::Mortgage));
    for issue in &result.issues {
        for section in
            crate::interview::issue_pages(&draft, &issue.context, Some(&issue.display.key))
        {
            assert!(matches!(section, TaxSection::Education | TaxSection::Mortgage), "{issue:?}");
        }
    }
    let mut session = Interview::new(TEST_USER, draft);
    let accepted = session.draft.clone();
    for item in items {
        let path =
            item.field.as_deref().expect("a Review Fix names its answer on the destination page");
        let expected: Vec<String> = match item.section {
            TaxSection::Education => {
                let student = accepted.credits.as_ref().unwrap().students[0].student;
                let base =
                    libertas::SchemaPath::root().field("value").field("students").key(student);
                vec![
                    base.clone().field("method").into(),
                    base.field("more").field("american_opportunity_used_up").into(),
                ]
            }
            TaxSection::Mortgage => {
                let loan = accepted.deductions.as_ref().unwrap().mortgage[0].id;
                let base = libertas::SchemaPath::root().field("value").field("loans").key(loan);
                let asked = libertas::SchemaPath::root()
                    .field("questions")
                    .key(alloc::format!("mortgage/{loan}").as_str())
                    .field("answer");
                vec![base.field("points").into(), asked.into()]
            }
            other => panic!("fixture has no pending question on {other:?}"),
        };
        assert!(expected.iter().any(|expected| expected == path), "{item:?}");
        let request = P::OpenSection {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            from: TaxSection::Review as i32,
            section: item.section,
            reveal_path: item.field.clone(),
        };
        match (item.section, send(&mut session, request)) {
            (TaxSection::Education, Reply::Edit(P::SaveEducation { value, .. })) => {
                let student = value.students.iter().find(|student| student.student == -1).unwrap();
                assert!(
                    student.method.is_none() || student.more.american_opportunity_used_up.is_none()
                );
            }
            (TaxSection::Mortgage, Reply::Edit(P::SaveMortgages { value, .. })) => {
                assert!(value.loans.iter().any(|loan| loan.refund_origin.is_none()));
            }
            (section, reply) => panic!("{section:?} has no answer row: {reply:?}"),
        }
        assert_eq!(session.draft, accepted);
        assert!(!session.dirty);
    }
}

#[test]
fn estimate_pages_show_local_fixes_and_one_review_count_without_changing_the_return() {
    let mut session = Interview::new(TEST_USER, estimate_issue_household());
    let accepted = session.draft.clone();
    let result = estimate(&accepted);
    let full = crate::tests::cross::interview::review_state(&mut session).0;
    let full: Vec<_> = full
        .into_iter()
        .filter(|row| status_message(&row.status).key != "REVIEW_CONTINUE_FROM")
        .collect();
    for section in [TaxSection::Education, TaxSection::Mortgage, TaxSection::People] {
        let reply = crate::tests::cross::interview::from_review(&mut session, section);
        let shown = match reply {
            Reply::Edit(P::SaveEducation { estimate: Some(estimate), .. })
            | Reply::Edit(P::SaveMortgages { estimate: Some(estimate), .. })
            | Reply::Edit(P::SavePeople { estimate: Some(estimate), .. }) => estimate,
            other => panic!("{section:?}: {other:?}"),
        };
        let local = full.iter().filter(|row| row.section == section).count();
        let other = full.len() - local;
        assert_eq!(shown.remaining_items.len(), local + usize::from(other != 0));
        for row in shown.remaining_items.iter().take(local) {
            assert_eq!(row.section, section);
            assert_eq!(status_message(&row.status).key, "DOCUMENT_PENDING_FIX");
            let path = row.field.as_deref().expect("a local Fix locates its answer");
            match section {
                TaxSection::Education => {
                    let student = accepted.credits.as_ref().unwrap().students[0].student;
                    let base =
                        libertas::SchemaPath::root().field("value").field("students").key(student);
                    assert!(
                        [
                            base.clone().field("method").to_string(),
                            base.field("more").field("american_opportunity_used_up").to_string()
                        ]
                        .iter()
                        .any(|value| value == path)
                    );
                }
                TaxSection::Mortgage => {
                    let loan = accepted.deductions.as_ref().unwrap().mortgage[0].id;
                    let base = libertas::SchemaPath::root().field("value").field("loans").key(loan);
                    let asked = libertas::SchemaPath::root()
                        .field("questions")
                        .key(alloc::format!("mortgage/{loan}").as_str())
                        .field("answer");
                    assert!(
                        [base.field("points").to_string(), asked.to_string()]
                            .iter()
                            .any(|value| value == path)
                    );
                }
                _ => panic!("fixture has no local issue on {section:?}"),
            }

            assert!(full.iter().any(|source| source.section == section
                && (source.item == row.item || source.status == row.item)));
        }
        if other != 0 {
            let row = shown.remaining_items.last().unwrap();
            assert_eq!(row.section, TaxSection::Review);
            assert_eq!(row.field, None);
            assert_eq!(
                status_text(&row.status),
                alloc::format!(
                    "{other} more {} in later sections",
                    if other == 1 { "item" } else { "items" }
                )
            );
        }
        assert_eq!(session.draft, accepted);
        assert_eq!(estimate(&session.draft), result);
    }
    assert_eq!(
        crate::tests::cross::interview::review_state(&mut session)
            .0
            .into_iter()
            .filter(|row| status_message(&row.status).key != "REVIEW_CONTINUE_FROM")
            .collect::<Vec<_>>(),
        full
    );

    let mut only_education = accepted;
    only_education.deductions.as_mut().unwrap().mortgage.clear();
    let mut session = Interview::new(TEST_USER, only_education);
    let Reply::Edit(P::SaveEducation { estimate: Some(shown), .. }) =
        crate::tests::cross::interview::from_review(&mut session, TaxSection::Education)
    else {
        panic!()
    };
    assert!(!shown.remaining_items.is_empty());
    assert!(shown.remaining_items.iter().all(|row| row.section == TaxSection::Education
        && status_message(&row.status).key == "DOCUMENT_PENDING_FIX"));
}

#[test]
fn omitted_equal_name_records_keep_distinct_nonblocking_correction_links() {
    let mut d = complete(FederalStatus::Single);
    for id in [2, 3] {
        let mut entry = income(FederalIncome::Gambling {
            data: gambling_source(FederalGambling {
                winnings: 100_000,
                losses: 0,
                withholding: 0,
                own_records: false,
                ordinary: false,
            }),
        });
        entry.id = id;
        entry.label = "Same name".into();
        add(&mut d, entry);
    }
    let result = supported(&d);
    let notices: Vec<_> =
        result.review_notes.iter().filter(|n| n.display.key == "WARN_GAMBLING_LEFT_OUT").collect();
    assert_eq!(notices.len(), 2);
    assert_ne!(
        notices[0].context.as_ref().unwrap().identity(),
        notices[1].context.as_ref().unwrap().identity()
    );
    let completed = FederalEstimate::from(result);
    assert_eq!(
        completed
            .review_notes
            .iter()
            .filter(|n| status_text(&n.message).contains("Same name"))
            .count(),
        2,
        "completed estimates retain both affected-record labels"
    );
    let mut session = Interview::new(TEST_USER, d.clone());
    let Reply::Response(P::Review {
        correction_notes,
        remaining_items,
        allowed_actions,
        result,
        ..
    }) = send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!()
    };
    assert!(remaining_items.is_empty());
    assert_eq!(correction_notes.len(), 2);
    assert!(correction_notes.iter().all(
        |n| n.section == TaxSection::Documents && status_text(&n.status).contains("Same name")
    ));
    assert!(allowed_actions.contains(&(libertas_macros::variant_index!(P::StartFinish) as i32)));
    assert!(
        !result
            .review_notes
            .iter()
            .any(|n| status_message(&n.message).key == "WARN_GAMBLING_LEFT_OUT"),
        "linked notes appear once"
    );
    assert_eq!(session.draft, d, "review never removes accepted source records");
}

#[test]
fn record_display_prefers_name_and_uses_real_id_for_blank_name() {
    assert_eq!(
        crate::interview::record_display_arg(" Acme ", 47),
        MessageArg::Text(" Acme ".into())
    );
    for name in ["", "   ", "\t"] {
        assert_eq!(crate::interview::record_display_arg(name, 47), MessageArg::Number(47));
    }
    let mut entry = wage(47, Owner::Taxpayer, 1_000_000, 0);
    // An unnamed source is shown by its kind and real identity.
    entry.label = "".into();
    assert_eq!(crate::interview::income_display_arg(&entry), MessageArg::Number(47));
    let label = crate::interview::income_issue_context(&entry).label();
    assert_eq!(label.key, "LABEL_ENTRY_W2");
    assert_eq!(label.args, vec![MessageArg::Number(47)]);
    assert_eq!(label.english(), "W-2 47");
    entry.label = "Acme".into();
    assert_eq!(crate::interview::income_display_arg(&entry), MessageArg::Text("Acme".into()));
}

#[test]
fn owner_and_nested_raw_omission_scopes_do_not_collapse_or_advertise_an_editor() {
    let mut d = complete(FederalStatus::Joint);
    let mut accounts = Vec::new();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut account = hsa_entry(owner, HsaCoverage::SelfOnly, 0);
        // A Roth transfer that did not come first in the year is left out (traditional funding is always supported since 2026-10-19).
        let funding = HsaFunding {
            ongoing_sep_simple: false,
            income: 1,
            basis: HsaFundingBasis::Roth { value: 10_000, before_other_activity: false },
            funded: 20260115,
            amount: 100,
            coverage: HsaCoverage::SelfOnly,
            loss: None,
        };
        account.funding = vec![funding.clone(), funding];
        accounts.push(account);
    }
    d.adjustments.as_mut().unwrap().hsa = accounts;
    let mut notes = Vec::new();
    let projected = crate::unknowns::supported(&d, &mut notes).unwrap();
    let notices: Vec<_> =
        notes.iter().filter(|n| n.display.key == "WARN_HSA_FUNDING_LEFT_OUT").collect();
    assert_eq!(notices.len(), 4);
    for (i, first) in notices.iter().enumerate() {
        for second in &notices[i + 1..] {
            assert!(!first.same_notice(second));
        }
        assert_eq!(crate::interview::issue_page(&d, first.context.as_ref().unwrap(), None), None);
    }
    assert!(projected.adjustments.unwrap().hsa.iter().all(|a| a.funding.is_empty()));
    assert_eq!(d.adjustments.unwrap().hsa[0].funding.len(), 2);
}

#[test]
fn correction_overrides_keep_identity_and_offer_each_actual_owner() {
    let d = complete(FederalStatus::Single);
    let source = crate::interview::income_issue_context(&d.income[0]);
    let override_context =
        source.clone().alternatives(&[TaxSection::Deductions, TaxSection::Taxes]);
    assert_eq!(override_context.identity(), source.identity());
    assert_eq!(override_context.label(), source.label());
    assert_eq!(
        crate::interview::issue_pages(&d, &override_context, None),
        vec![TaxSection::Deductions, TaxSection::Taxes]
    );
    assert!(crate::interview::issue_pages(&d, &source.manual(), None).is_empty());
    let message = Message::new("TEXT", vec![MessageArg::Text("same warning".into())]);
    let make = |target| {
        ReviewNote::scoped(
            "same warning",
            message.clone(),
            IssueContext::income_id(d.income[0].id).corrective(target),
        )
    };
    assert!(!make(TaxSection::Taxes).same_notice(&make(TaxSection::Deductions)));
    assert!(make(TaxSection::Taxes).same_notice(&make(TaxSection::Taxes)));
}

#[test]
fn all_correction_rows_survive_above_64_and_issues_without_an_editor_open_their_source() {
    let mut d = complete(FederalStatus::Single);
    d.income = (1..=100).map(|id| wage(id, Owner::Taxpayer, 100_000, 0)).collect();
    let mut result = estimate(&d);
    result.issues.clear();
    for entry in &d.income {
        crate::federal_rules::issue_localized(
            &mut result,
            &crate::interview::income_issue_context(entry),
            "Income",
            "REVIEW_NOT_ANSWERED",
            "Check the record",
        );
    }
    let items = crate::interview::remaining_items(&d, &result, TaxSection::Review);
    assert_eq!(items.len(), 100);
    assert!(items.iter().all(|i| i.section == TaxSection::Documents));
    result.issues.clear();
    crate::federal_rules::issue_localized(
        &mut result,
        &crate::interview::income_issue_context(&d.income[0]).manual(),
        "No editor",
        "REVIEW_NOT_ANSWERED",
        "A readable issue",
    );
    let items = crate::interview::remaining_items(&d, &result, TaxSection::Review);
    assert_eq!(items.iter().map(|i| i.section).collect::<Vec<_>>(), [TaxSection::Documents]);
}

#[test]
fn excluded_w2g_previews_zero_inclusion_without_mutating_accepted_answers() {
    let d = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d.clone());
    for (special, expected) in [(false, 100_000), (true, 0)] {
        let amount = crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::Gambling(w2g_document(W2g {
                payer: Default::default(),
                winner: Default::default(),
                winnings: 100_000,
                withholding: 0,
                special,
                state_details: Default::default(),
            })),
        );
        assert_eq!(amount, Some(expected));
        assert_eq!(session.draft, d);
    }
}

#[test]
fn completed_review_can_reopen_accepted_review_for_correction_notes() {
    let d = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d.clone());
    assert!(matches!(
        send(
            &mut session,
            P::Finish {
                user: TEST_USER,
                cookie: d.cookie,
                revision: d.revision,
                page: TaxSection::Review as i32,
                previous: TaxSection::Review as i32,
                review: true,
                progress: vec![],
                refund: crate::tests::cross::interview::whole_refund(),
            }
        ),
        Reply::Response(P::Finished { .. })
    ));
    assert!(matches!(
        send(&mut session, P::ReviewAccepted { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    assert!(
        include_str!("../../core/protocol.rs")
            .contains("#[libertas_next_request(\"ReviewAccepted\")]\n    Finished")
    );
}

#[test]
fn renaming_is_not_the_identity_of_a_scoped_note() {
    let d = complete(FederalStatus::Single);
    let mut context = crate::interview::income_issue_context(&d.income[0]);
    let message = Message::new("REVIEW_NOT_ANSWERED", vec![]);
    let first = ReviewNote::scoped("unchanged", message.clone(), context.clone());
    let IssueContext::Income { label, .. } = &mut context else { panic!() };
    *label = Message::new("TEXT", vec![MessageArg::Text("Renamed".into())]);
    assert!(first.same_notice(&ReviewNote::scoped("unchanged", message, context.clone())));
    assert_eq!(first.context.unwrap().identity(), NoteIdentity::Income(d.income[0].id));
    let child = context.item("children", 0);
    assert_eq!(
        child.clone().named(" Acme ").label(),
        Message::new("TEXT", vec![MessageArg::Text(" Acme ".into())])
    );
    assert_eq!(child.clone().named("  ").label(), child.label());
    assert_eq!(child.clone().named("Acme").identity(), child.identity());
    let manual_child = IssueContext::Section(TaxSection::Medical).item("premiums", 1).manual();
    let displayed = crate::interview::contextual_message(
        &Message::new("REVIEW_NOT_ANSWERED", vec![]),
        &manual_child,
    );
    assert_eq!(displayed.key, "REVIEW_RECORD_STATUS");
    assert!(displayed.english().contains("item 2"));
}

#[test]
fn omitted_amt_loss_history_opens_its_actual_self_employment_editor() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol = AtnolHistory::Records {
        years: vec![AtnolYear { year: 2025, balance: 100_000, special_exception: false }],
        confirmed: false,
    };
    let result = supported(&d);
    let mut session = Interview::new(TEST_USER, d);
    let Reply::Response(P::Review { correction_notes, cookie, revision, page, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!()
    };
    let note = correction_notes
        .iter()
        .find(|n| status_message(&n.status).key == "WARN_ATNOL_LEFT_OUT")
        .unwrap();
    assert_eq!(note.section, TaxSection::SelfEmployment);
    assert!(result.issues.is_empty());
    let reply = send(
        &mut session,
        P::OpenSection {
            user: TEST_USER,
            cookie,
            revision,
            from: page,
            section: note.section,
            reveal_path: note.field.clone(),
        },
    );
    assert!(
        matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. })),
        "{reply:?}"
    );
}

#[test]
fn raw_only_hsa_funding_omissions_stay_distinct_and_readable_without_false_links() {
    let mut d = complete(FederalStatus::Single);
    let mut account = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    // A Roth transfer that did not come first in the year is left out (traditional funding is always supported since 2026-10-19).
    let funding = HsaFunding {
        ongoing_sep_simple: false,
        income: 1,
        basis: HsaFundingBasis::Roth { value: 10_000, before_other_activity: false },
        funded: 20260115,
        amount: 100,
        coverage: HsaCoverage::SelfOnly,
        loss: None,
    };
    account.funding = vec![funding.clone(), funding];
    d.adjustments.as_mut().unwrap().hsa = vec![account];
    let mut notes = Vec::new();
    crate::unknowns::supported(&d, &mut notes).unwrap();
    let notes: Vec<_> =
        notes.into_iter().filter(|n| n.display.key == "WARN_HSA_FUNDING_LEFT_OUT").collect();
    assert_eq!(notes.len(), 2);
    assert!(!notes[0].same_notice(&notes[1]));
    assert!(
        notes.iter().all(
            |n| crate::interview::issue_pages(&d, n.context.as_ref().unwrap(), None).is_empty()
        )
    );
    let mut result = estimate(&complete(FederalStatus::Single));
    result.review_notes = notes;
    let displayed = FederalEstimate::from(result);
    assert_eq!(displayed.review_notes.len(), 2);
    assert!(status_text(&displayed.review_notes[0].message).contains("item 1"));
    assert!(status_text(&displayed.review_notes[1].message).contains("item 2"));
}

#[test]
fn readonly_coverage_projection_preserves_omitted_ira_records() {
    let mut d = complete(FederalStatus::Single);
    let FederalIncome::Wage { data: crate::WageSource::W2(data) } = &mut d.income[0].income else {
        panic!()
    };
    data.form.wages = Some(8_500_000);
    data.form.retirement_plan = true;
    let iras = ira_facts(600_000, 0, Answer::No);
    let mut rollover = qtp_roth_rollover();
    rollover.rollover.confirmed = false;
    d.adjustments.as_mut().unwrap().iras = iras;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[rollover]);
    // Nothing is left out of an unconfirmed transfer any more: it is an ordinary 529
    // withdrawal with a note, and the IRA records stay as entered.
    let actual = supported(&d);
    assert!(noted(&actual, "NOTE_QTP_ROTH_NOT_QUALIFIED"));
    let retained = &d.adjustments.as_ref().unwrap().iras;
    assert!(!retained.taxpayer.covered_at_work);
    assert!(
        d.income.iter().any(|e| matches!(e.income, FederalIncome::EducationDistribution { .. }))
    );
}

#[test]
fn archer_wage_answers_preserve_accepted_child_amounts() {
    for corrections in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        let mut owner = archer_contributor(Owner::Taxpayer, ArcherCoverage::None, 0);
        owner.income.clear();
        let (_collection, amount_line) = if corrections {
            let first = ArcherCorrection {
                failed_deposit: None,
                document: None,
                year: 2025,
                contributed_on: 20260401,
                principal: 10_000,
                earnings: 0,
                returned: 20260601,
                deadline: 20261015,
                payer: ArcherCorrectionPayer::Employer { in_wages: false },
            };
            let mut second = first.clone();
            second.contributed_on = 20260402;
            second.principal = 20_000;
            second.payer = ArcherCorrectionPayer::Employer { in_wages: false };
            owner.corrections = vec![first, second];
            ("owner.corrections", "archer_correction_employer_principal_other_income")
        } else {
            let first = ArcherPriorEmployerExcessDeposit {
                contributed_on: 20260401,
                deadline: 20260415,
                amount: 10_000,
                in_wages: false,
            };
            let mut second = first.clone();
            second.contributed_on = 20260402;
            second.amount = 20_000;
            second.in_wages = false;
            owner.prior_employer_excess = vec![first, second];
            ("owner.prior_employer_excess", "archer_prior_designated_employer_excess_other_income")
        };
        set_archer(&mut draft, vec![owner], ArcherSharing::Equal);
        let accepted = draft.clone();
        let result = supported(&draft);
        // Both records count (the printed dates decide); the Not sure one is
        // assumed not in wages: $100 + $200.
        assert_eq!(value(&result, amount_line), 300);
        assert_eq!(draft, accepted);
    }
}

#[test]
fn ltc_omission_keeps_the_accepted_insured_child() {
    let mut draft = complete(FederalStatus::Joint);
    set_age(&mut draft, -1, 30);
    set_age(&mut draft, -2, 30);
    let mut entry = income(health_business(20_000, 1_000, false));
    entry.label = "Acme business".into();
    let FederalIncome::Business { data } = &mut entry.income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    let first = BusinessCarePremium {
        person: -1,
        months: vec![BusinessCareMonth {
            month: 1,
            premiums: 10_000,
            employer_eligible: false,
            reimbursed: None,
        }],
        qualified: CareContractQualification::Nonqualified,
    };
    let mut second = first.clone();
    second.person = -2;
    second.qualified = CareContractQualification::Qualified;
    second.months[0].employer_eligible = true;
    *long_term_care = vec![first, second];
    add(&mut draft, entry);
    let accepted = draft.clone();
    let source = crate::interview::income_issue_context(&draft.income[1]);
    let result = supported(&draft);
    let omission = result
        .review_notes
        .iter()
        .find(|note| note.display.key == "WARN_LTC_PREMIUM_LEFT_OUT")
        .unwrap();
    assert_eq!(
        omission.context.as_ref().unwrap().identity(),
        source.item("long_term_care", 0).identity()
    );
    assert_eq!(draft, accepted);
}

// Native protocol values for the actual Studio preview; no production App hook.
#[test]
fn estimate_issue_client_fixture() {
    extern crate std;
    let mut session = Interview::new(TEST_USER, estimate_issue_household());
    let Reply::Response(actions) = send(&mut session, P::Enter { user: TEST_USER }) else {
        unreachable!()
    };
    if let Ok(directory) = std::env::var("TAX_ESTIMATE_CLIENT_FIXTURE_DIR") {
        std::fs::write(std::path::Path::new(&directory).join("actions.avro"), actions.to_avro())
            .unwrap();
    }
    for (section, name) in [
        (TaxSection::Education, "education"),
        (TaxSection::Mortgage, "mortgage"),
        (TaxSection::Review, "review"),
    ] {
        let reply = crate::tests::cross::interview::from_review(&mut session, section);
        let value = match reply {
            Reply::Edit(value) | Reply::Response(value) => value,
        };
        assert!(matches!(
            value,
            P::SaveEducation { .. } | P::SaveMortgages { .. } | P::Review { .. }
        ));
        if let Ok(directory) = std::env::var("TAX_ESTIMATE_CLIENT_FIXTURE_DIR") {
            std::fs::write(
                std::path::Path::new(&directory).join(alloc::format!("{name}.avro")),
                value.to_avro(),
            )
            .unwrap();
        }
    }
}

#[test]
fn mortgage_average_issue_opens_the_mortgage_answers_and_reveals_the_loan_list() {
    let mut draft = estimate_issue_household();
    let loan = &mut draft.deductions.as_mut().unwrap().mortgage[0];
    loan.form.box_2 = 90_000_000;
    loan.form.box_4 = 0;
    loan.form.box_5 = 0;
    loan.form.box_6 = 0;
    let mut session = Interview::new(TEST_USER, draft);
    let full = crate::tests::cross::interview::review_state(&mut session).0;
    let row = full
        .iter()
        .find(|row| status_message(&row.status).key == "MORTGAGE_AVERAGE_NEEDED")
        .unwrap();
    assert_eq!(row.section, TaxSection::Mortgage);
    let Reply::Edit(P::SaveMortgages { estimate: Some(estimate), .. }) =
        crate::tests::cross::interview::from_review(&mut session, row.section)
    else {
        unreachable!()
    };
    let row = estimate
        .remaining_items
        .iter()
        .find(|row| status_message(&row.item).key == "MORTGAGE_AVERAGE_NEEDED")
        .unwrap();
    assert_eq!(row.field.as_deref(), Some("[value/loans]"));
    assert_eq!(status_message(&row.status).key, "DOCUMENT_PENDING_FIX");
}

#[test]
fn pending_questions_omit_blank_document_names_and_retain_supplied_names() {
    let mut draft = estimate_issue_household();
    let mut loan =
        nol_work_deductions(60_000, 0, 0, 100).deductions.unwrap().vehicle_loans.remove(0);
    loan.id = draft.next_id;
    draft.next_id += 1;
    loan.document.box_5 = Some(100);
    draft.deductions.as_mut().unwrap().vehicle_loans = vec![loan];
    draft.credits.as_mut().unwrap().marketplace = marketplace_claim(
        &mut draft,
        vec![marketplace_month(1, 500, 600, 200), marketplace_month(2, 500, 600, 200)],
    );
    // Keep the required first coverage row; the second printed month still needs its answer.
    marketplace_facts_mut(&mut draft).forms[0].coverage.pop().unwrap();
    for name in ["", "   ", "Synthetic source"] {
        draft.deductions.as_mut().unwrap().mortgage[0].property = name.into();
        draft.deductions.as_mut().unwrap().vehicle_loans[0].document.lender.name = name.into();
        // The issuer's schema requires at least one character; whitespace is a valid
        // blank-rendering boundary. The other two source labels also permit empty text.
        marketplace_facts_mut(&mut draft).forms[0].form.issuer =
            if name.is_empty() { " " } else { name }.into();
        let result = estimate(&draft);
        let items = crate::interview::remaining_items(&draft, &result, TaxSection::Review);
        for key in [
            "DOCUMENT_MORTGAGE_REFUND_ORIGIN",
            "DOCUMENT_VEHICLE_REFUND_BENEFIT",
            "DOCUMENT_MARKETPLACE_ANSWERS",
        ] {
            if key == "DOCUMENT_MORTGAGE_REFUND_ORIGIN" {
                // A Form 1098 is named by its printed lender and borrower, so the item names
                // the document whatever the property label is, and is never blank.
                assert!(
                    items.iter().any(|item| {
                        let message = status_message(&item.item);
                        message.key == "DOCUMENT_PENDING_ANSWER"
                            && message.args.first()
                                == Some(&crate::MessageArg::Resource(key.into()))
                            && message.args.len() == 2
                    }),
                    "{key}: {items:?}"
                );
                continue;
            }
            let expected = if name.trim().is_empty() {
                crate::Message::new(key, vec![])
            } else {
                crate::Message::new(
                    "DOCUMENT_PENDING_ANSWER",
                    vec![
                        crate::MessageArg::Resource(key.into()),
                        crate::MessageArg::Nested(Box::new(crate::Message::new(
                            "TEXT",
                            vec![crate::MessageArg::Text(name.into())],
                        ))),
                    ],
                )
            };
            assert!(
                items.iter().any(|item| status_message(&item.item) == expected),
                "{key}: {items:?}"
            );
        }
    }
}
