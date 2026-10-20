use super::*;
use crate::federal_model::{NoteIdentity, ReviewNote};
use crate::tests::support::client::survey::survey_preview_document;

#[test]
fn omitted_equal_name_records_keep_distinct_nonblocking_correction_links() {
    let mut d = complete(FederalStatus::Single);
    for id in [2, 3] {
        let mut entry = income(FederalIncome::Gambling {
            data: FederalGambling {
                winnings: 100_000,
                losses: 0,
                withholding: 0,
                own_records: false,
                ordinary: false,
            },
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
    assert!(
        correction_notes
            .iter()
            .all(|n| n.section == TaxSection::Gambling
                && status_text(&n.status).contains("Same name"))
    );
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
fn all_correction_rows_survive_above_64_and_ownerless_issues_have_no_self_link() {
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
    assert!(items.iter().all(|i| i.section == TaxSection::Wages));
    result.issues.clear();
    for context in [IssueContext::Review, IssueContext::income_id(999)] {
        crate::federal_rules::issue_localized(
            &mut result,
            &context,
            "No owner",
            "REVIEW_NOT_ANSWERED",
            "A readable issue",
        );
    }
    assert!(crate::interview::remaining_items(&d, &result, TaxSection::Review).is_empty());
    let displayed = FederalEstimate::from(result);
    assert_eq!(displayed.issues.len(), 2, "no correction link must not remove the issue");
}

#[test]
fn excluded_w2g_previews_zero_inclusion_without_mutating_accepted_answers() {
    let d = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d.clone());
    for (special, expected) in [(false, 100_000), (true, 0)] {
        let reply = send(&mut session, {
            let mut request = survey_preview_document(IncomeDocument::W2g(W2g {
                winnings: 100_000,
                withholding: 0,
                special,
                state_details: Default::default(),
            }));
            let P::PreviewDocument { foreign_countries: request_countries, .. } = &mut request
            else {
                unreachable!()
            };
            *request_countries = crate::foreign_accounts::countries();
            request
        });
        assert!(
            matches!(reply, Reply::Response(P::IncomePreview { preview: Some(amount), .. }) if amount == expected),
            "{reply:?}"
        );
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
                progress: vec![]
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
        P::OpenSection { user: TEST_USER, cookie, revision, from: page, section: note.section },
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
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.wages = 8_500_000;
    data.retirement_plan = true;
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
