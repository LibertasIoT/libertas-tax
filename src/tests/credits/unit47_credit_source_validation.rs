use alloc::string::ToString;
use libertas::SchemaPath;
// Normal Save/Preview source-owner controls. Expected keys and boundary amounts
// are source facts; financial outputs remain covered by the existing form-owner fixtures.
use super::*;
use crate::tests::support::client::income_pages::{open_documents, save_documents};
use alloc::string::String;

pub(super) use crate::tests::support::client::income_pages::catalog_preview_document as credit_preview;

pub(super) use crate::tests::support::client::survey::survey_refusal as credit_refusal;

fn credit_accept(session: &mut Interview, request: P) {
    let reply = if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(session, request)
    } else {
        send(session, request)
    };
    // A cooperative's allocation with no receiving activity yet is retained;
    // its topic refuses further answers until a farm receives it.
    if !crate::tests::support::client::income_pages::awaits_activity(&reply) {
        accepted(&reply);
    }
}

fn credit_personal_refusal_request(session: &mut Interview, desired: &[IncomeDocumentItem]) -> P {
    let request = save_documents(session, desired.to_vec());
    // The printed facts are valid; the supplied personal answer is deliberately
    // invalid and is sent separately to its real topic below.
    accepted(&send(session, request));
    crate::tests::support::client::income_pages::information_return_answers(session, desired)
}

fn credit_save_rows(
    session: &mut Interview,
    topic: TaxSection,
    rows: Vec<IncomeDocumentItem>,
) -> P {
    if rows.iter().all(|row| matches!(row.document, IncomeSource::Record { .. })) {
        crate::tests::support::client::income_pages::save_topic(session, topic, rows)
    } else {
        assert!(
            rows.iter().all(|row| !matches!(row.document, IncomeSource::Record { .. })),
            "accept prerequisite facts explicitly before the attempted owner Save"
        );
        save_documents(session, rows)
    }
}
fn credit_document_path(document: &IncomeSource, index: usize, local: &str) -> String {
    let category = match document {
        IncomeSource::Record { .. } => "documents",
        IncomeSource::Wage { .. } => "common/wages/documents",
        IncomeSource::InterestDividend { .. } => "common/interest_dividends/documents",
        IncomeSource::InvestmentSale { .. } => "common/investment_sales/documents",
        IncomeSource::Retirement { .. } => "common/retirement/documents",
        IncomeSource::SelfEmployment { .. } => "common/self_employment/documents",
        IncomeSource::K1 { .. } => "common/k1/documents",
        IncomeSource::GovernmentPayment { .. } => "common/government_payments/documents",
        IncomeSource::Gambling(_) => "common/gambling/documents",
        IncomeSource::Farming(_) => "other/farming/documents",
        IncomeSource::OtherIncome { .. } => "other/other_income/documents",
        IncomeSource::CanceledDebt { .. } => "other/canceled_debt/documents",
    };
    let mut local = String::from(local);
    if matches!(document, IncomeSource::K1 { value: crate::K1Document::K1Partnership(_) }) {
        for (before, after) in [
            ("more/qualified_dividends", "form/qualified_dividends"),
            ("more/dividends", "form/dividends"),
            ("more/medical_insurance", "form/deductions"),
            ("more/income", "statements/income"),
            ("more/deductions", "form/deductions"),
            ("more/credits", "statements/credits"),
            ("more/investment_property", "statements/investment_property"),
            ("more/section179_sales", "statements/section179_sales"),
            ("qbi", "statements/qbi"),
            ("schedule_k3", "statements/schedule_k3"),
            ("reporting_period", "form/reporting_period"),
        ] {
            if local == before || local.starts_with(&alloc::format!("{before}/")) {
                local = alloc::format!("{after}{}", &local[before.len()..]);
                break;
            }
        }
    }
    // The S-corporation and estate K-1 pages are the paper (`form`) plus its
    // attached statements; each old page field keeps its leaf under one of them.
    // `more/` named the statements; a bare name is a paper box unless only a
    // statement carries it.
    type RelocatedFields<'a> = (&'a [&'a str], &'a [&'a str], &'a [(&'a str, &'a str)]);
    let relocated: Option<RelocatedFields<'_>> = match document {
        IncomeSource::K1 { value: crate::K1Document::K1Scorp(_) } => Some((
            &[
                "qbi",
                "rentals",
                "property",
                "income",
                "credits",
                "investment_property",
                "section179_sales",
                "private_activity",
                "schedule_k3",
                "situations",
            ],
            &["income", "credits"],
            &[],
        )),
        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(_) } => Some((
            &[
                "final_character",
                "amt_other_in_a",
                "amt_adjustments",
                "investment_income",
                "reported_credits",
                "foreign_statement",
                "investment_property",
                "activities",
                "qbi",
                "private_activity_interest",
                "section199a_dividends",
            ],
            &[],
            &[
                ("business/box6", "business_income"),
                ("business/box7", "rental_real_estate_income"),
                ("business/box8", "other_rental_income"),
                ("business/activities", "activities"),
                ("activities/activities", "activities"),
                ("qbi/statements", "qbi"),
                ("entity_period/period/began", "began"),
                ("entity_period/period/ended", "ended"),
                ("entity_period/began", "began"),
                ("entity_period/ended", "ended"),
                ("amt_reconciliation/other_in_a", "amt_other_in_a"),
                ("amt_reconciliation/adjustments", "amt_adjustments"),
            ],
        )),
        _ => None,
    };
    let page_local = local.clone();
    if let Some((statements, paper_names, renamed)) = relocated
        && !local.is_empty()
    {
        for (before, after) in renamed {
            if local == *before || local.starts_with(&alloc::format!("{before}/")) {
                local = alloc::format!("{after}{}", &local[before.len()..]);
                break;
            }
        }
        let more = local.starts_with("more/");
        if more {
            local = String::from(&local["more/".len()..]);
        }
        let head = local.split('/').next().unwrap_or_default();
        let statement = statements.contains(&head) && (more || !paper_names.contains(&head));
        let part = if statement { "statements" } else { "form" };
        local = alloc::format!("{part}/{local}");
    }
    let suffix = if local.is_empty() { String::new() } else { alloc::format!("/{local}") };
    let path = alloc::format!("[documents/{category}/{index}/document{suffix}]");
    if local != page_local {
        // A refusal raised by the topic answers is still located by the
        // validators' K-1 page field, so keep it for the topic control.
        crate::tests::support::client::income_pages::remember_k1_page_local(&path, &page_local);
    }
    path
}
fn credit_resave(case: &str, session: &mut Interview, topic: TaxSection, source: bool) {
    let saved = session.draft.clone();
    let page = if source { open_documents(session) } else { open_topic(session, topic) };
    // A cooperative with no receiving farm is refused by its topic; nothing changes.
    let reply = send(session, page);
    if !crate::tests::support::client::income_pages::awaits_activity(&reply) {
        accepted(&reply);
    }
    assert_eq!(session.draft, saved, "{case}: accepted owner reopen/resave");
    let mut restored = restart(saved.clone());
    let page =
        if source { open_documents(&mut restored) } else { open_topic(&mut restored, topic) };
    let reply = send(&mut restored, page);
    if !crate::tests::support::client::income_pages::awaits_activity(&reply) {
        accepted(&reply);
    }
    assert_eq!(restored.draft, saved, "{case}: accepted owner restart/resave");
}

fn credit_document_case(
    case: &str,
    bad: impl FnOnce() -> IncomeSource,
    corrected: impl FnOnce() -> IncomeSource,
    key: &str,
    local: &str,
) {
    credit_document_case_in_draft(
        case,
        bad,
        corrected,
        key,
        local,
        complete(FederalStatus::Single),
    );
}

// A K-1 fixture's answers are kept per printed paper. A bad answer on an
// unchanged paper shares that paper with its correction, so each side's answers
// are restored before that side is submitted.
enum CreditK1Answers {
    Scorp(
        alloc::boxed::Box<crate::K1ScorpDocument>,
        alloc::boxed::Box<crate::ScorpPersonalRecords>,
    ),
    Estate(
        alloc::boxed::Box<crate::K1EstateTrustDocument>,
        alloc::boxed::Box<crate::EstateTrustSource>,
    ),
}
fn credit_k1_answers(source: &IncomeSource) -> Option<CreditK1Answers> {
    match source {
        IncomeSource::K1 { value: crate::K1Document::K1Scorp(document) } => {
            Some(CreditK1Answers::Scorp(
                document.clone().into(),
                scorp_fixture_source(document).personal.into(),
            ))
        }
        IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(document) } => {
            Some(CreditK1Answers::Estate(
                document.clone().into(),
                estate_fixture_source(document).into(),
            ))
        }
        _ => None,
    }
}
fn credit_restore_k1_answers(answers: &Option<CreditK1Answers>) {
    match answers {
        Some(CreditK1Answers::Scorp(document, personal)) => {
            remember_scorp_document(document.as_ref().clone(), personal.as_ref().clone());
        }
        Some(CreditK1Answers::Estate(document, answers)) => {
            remember_estate_document(document.as_ref().clone(), answers.as_ref().clone());
        }
        None => {}
    }
}
// The valid earlier row of an answer-only case: the same paper from a
// differently named entity, carrying the corrected answers.
fn credit_distinct_k1_row(answers: &Option<CreditK1Answers>) -> Option<IncomeSource> {
    Some(match answers.as_ref()? {
        CreditK1Answers::Scorp(document, personal) => {
            let mut document = document.as_ref().clone();
            document.form.corporation.name.push_str(" (earlier)");
            IncomeSource::K1 {
                value: crate::K1Document::K1Scorp(remember_scorp_document(
                    document,
                    personal.as_ref().clone(),
                )),
            }
        }
        CreditK1Answers::Estate(document, answers) => {
            let mut document = document.as_ref().clone();
            document.form.estate_trust.name.push_str(" (earlier)");
            let mut source = answers.as_ref().clone();
            source.form = document.form.clone();
            IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(remember_estate_document(document, source)),
            }
        }
    })
}

fn credit_document_case_in_draft(
    case: &str,
    bad: impl FnOnce() -> IncomeSource,
    corrected: impl FnOnce() -> IncomeSource,
    key: &str,
    local: &str,
    original: FederalDraft,
) {
    let bad = bad();
    let bad_answers = fixture_paper_answers(&bad);
    let bad_k1 = credit_k1_answers(&bad);
    let bad_cooperative = match &bad {
        IncomeSource::Farming(form) => fixture_cooperative_answers(form),
        _ => None,
    };
    let corrected = corrected();
    let corrected_k1 = credit_k1_answers(&corrected);
    let same_k1_paper = bad_k1.is_some() && bad == corrected;
    let answers = if key.starts_with("DEBT_") {
        let select = |answers: Vec<crate::DocumentAnswer>| {
            let mut answers: Vec<_> = answers
                .into_iter()
                .filter(|answer| match answer {
                    crate::DocumentAnswer::InterestAdjustment { .. } => true,
                    crate::DocumentAnswer::InterestAccrued { value }
                    | crate::DocumentAnswer::OidPremium { value }
                    | crate::DocumentAnswer::OidPrivateActivity { value } => *value != 0,
                    _ => false,
                })
                .collect();
            answers.sort_by_key(|answer| {
                !matches!(answer, crate::DocumentAnswer::InterestAdjustment { .. })
            });
            answers
        };
        let path = local
            .strip_prefix("adjustments/")
            .map_or_else(|| String::from("value"), |tail| alloc::format!("value/{tail}"));
        Some((select(bad_answers), select(fixture_paper_answers(&corrected)), path))
    } else if let Some(bad) = bad_cooperative {
        let IncomeSource::Farming(form) = &corrected else { unreachable!() };
        let corrected = fixture_cooperative_answers(form).unwrap();
        if let Some(tail) = local.strip_prefix("more/allocations/") {
            Some((
                vec![crate::DocumentAnswer::CooperativeAllocations {
                    value: bad.allocations.into_iter().map(Into::into).collect(),
                }],
                vec![crate::DocumentAnswer::CooperativeAllocations {
                    value: corrected.allocations.into_iter().map(Into::into).collect(),
                }],
                alloc::format!("value/{tail}"),
            ))
        } else if let Some(tail) = local.strip_prefix("more/credit_facts/") {
            let answer = |facts: crate::Patr1099CreditAnswers| {
                crate::DocumentAnswer::CooperativeCreditRecords {
                    investment_shares: facts.investment_shares,
                    other: facts.other.into_iter().map(Into::into).collect(),
                    participation: facts.participation,
                    activity: facts.activity,
                    notice: facts.notice,
                }
            };
            Some((
                vec![answer(bad.credit_facts)],
                vec![answer(corrected.credit_facts)],
                tail.into(),
            ))
        } else {
            None
        }
    } else {
        None
    };
    if let Some((bad_answers, corrected_answers, answer_path)) = answers {
        let mut session = Interview::new(TEST_USER, original);
        credit_local_source_case_in_session(
            case,
            &mut session,
            document(0, corrected.clone()),
            (bad, corrected),
            key,
            local,
            Some((bad_answers, corrected_answers, &answer_path)),
        );
        return;
    }
    let source = !matches!(&corrected, IncomeSource::Record { .. });
    let path = credit_document_path(&corrected, 1, local);
    let topic = topic_for(&corrected);
    let mut session = Interview::new(TEST_USER, original.clone());
    // A valid earlier row proves that failures retain original submitted indices.
    let earlier = if same_k1_paper {
        credit_distinct_k1_row(&corrected_k1).unwrap()
    } else {
        corrected.clone()
    };
    if same_k1_paper {
        credit_restore_k1_answers(&bad_k1);
    }
    let request =
        credit_save_rows(&mut session, topic, vec![document(0, earlier), document(0, bad.clone())]);
    credit_refusal(case, &mut session, request, key, &path);
    if same_k1_paper {
        credit_restore_k1_answers(&corrected_k1);
    }
    if !source {
        credit_refusal(
            &alloc::format!("{case}-preview"),
            &mut session,
            credit_preview(bad),
            key,
            &alloc::format!("[document/{local}]"),
        );
    }
    let request = credit_save_rows(
        &mut session,
        topic,
        vec![document(0, corrected.clone()), document(0, corrected)],
    );
    if source {
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        ));
    } else {
        credit_accept(&mut session, request);
    }
    credit_resave(case, &mut session, topic, source);
}

#[test]
fn unit47_credit_employer_sources_report_actual_employee_and_boundary() {
    for (name, group, cap) in [
        ("standard", WorkOpportunityGroup::Standard, 600_000),
        ("disabled", WorkOpportunityGroup::DisabledVeteran, 1_200_000),
        ("unemployed", WorkOpportunityGroup::LongUnemployedVeteran, 1_400_000),
        ("disabled-unemployed", WorkOpportunityGroup::DisabledLongUnemployedVeteran, 2_400_000),
        (
            "family",
            WorkOpportunityGroup::LongTermFamilyAssistance {
                second_year_paid_in_year: 0,
                second_year_paid_in_prior_year: 0,
            },
            1_000_000,
        ),
    ] {
        let mut equal = employer_credit_business();
        let e = &mut equal.more.work_opportunity.as_mut().unwrap().employees[2];
        e.group = group;
        e.first_year_paid_in_prior_year = cap;
        let mut bad = equal.clone();
        bad.more.work_opportunity.as_mut().unwrap().employees[2].first_year_paid_in_prior_year =
            cap + 1;
        credit_document_case(
            &alloc::format!("credit-wotc-{name}"),
            || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
            || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
            "WOTC_PRIOR_YEAR_WAGES_ABOVE_CAP",
            "more/work_opportunity/employees/2/first_year_paid_in_prior_year",
        );
    }
    let equal = employer_credit_business();
    let mut bad = equal.clone();
    bad.more.work_opportunity.as_mut().unwrap().employees[2].label = "Employee A".into();
    credit_document_case(
        "credit-wotc-duplicate",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
        "WOTC_EMPLOYEE_TWICE",
        "more/work_opportunity/employees/2/label",
    );
    let mut equal = health_credit_business();
    let c =
        equal.more.small_employer_health.as_mut().unwrap().employees[2].coverage.as_mut().unwrap();
    c.employer_paid = c.total_premium;
    let mut bad = equal.clone();
    bad.more.small_employer_health.as_mut().unwrap().employees[2]
        .coverage
        .as_mut()
        .unwrap()
        .employer_paid += 1;
    credit_document_case(
        "credit-health-premium",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
        "SMALL_EMPLOYER_HEALTH_SHARE_ABOVE_PREMIUM",
        "more/small_employer_health/employees/2/coverage/employer_paid",
    );
    let equal = health_credit_business();
    let mut bad = equal.clone();
    bad.more.small_employer_health.as_mut().unwrap().employees[2].label = "E1".into();
    credit_document_case(
        "credit-health-duplicate",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal.clone()) },
        "SMALL_EMPLOYER_HEALTH_EMPLOYEE_TWICE",
        "more/small_employer_health/employees/2/label",
    );
    let mut bad = equal.clone();
    for e in &mut bad.more.small_employer_health.as_mut().unwrap().employees {
        e.coverage = None;
    }
    credit_document_case(
        "credit-health-enrollment",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
        "SMALL_EMPLOYER_HEALTH_NOBODY_ENROLLED",
        "more/small_employer_health/employees",
    );
    let mut equal = health_credit_business();
    let c = equal.more.small_employer_health.as_mut().unwrap();
    c.coverage = HealthCreditCoverage::CountyException;
    c.first_credit_year = 2025;
    let mut bad = equal.clone();
    bad.more.small_employer_health.as_mut().unwrap().first_credit_year = 2026;
    credit_document_case(
        "credit-health-county",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
        "SMALL_EMPLOYER_HEALTH_COUNTY_RELIEF_NEEDS_PRIOR_YEAR",
        "more/small_employer_health/first_credit_year",
    );
    let mut equal = paid_leave_business();
    equal.more.paid_leave.as_mut().unwrap().employees[1].leave_hours = 480.0;
    let mut bad = equal.clone();
    bad.more.paid_leave.as_mut().unwrap().employees[1].leave_hours += 0.01;
    credit_document_case(
        "credit-leave-weeks",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal.clone()) },
        "PAID_LEAVE_ABOVE_12_WEEKS",
        "more/paid_leave/employees/1/leave_hours",
    );
    let mut bad = equal.clone();
    bad.more.paid_leave.as_mut().unwrap().employees[1].label = "A".into();
    credit_document_case(
        "credit-leave-duplicate",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal.clone()) },
        "PAID_LEAVE_EMPLOYEE_TWICE",
        "more/paid_leave/employees/1/label",
    );
    let mut bad = equal.clone();
    let c = bad.more.paid_leave.as_mut().unwrap();
    c.employees.clear();
    c.insurance = None;
    credit_document_case(
        "credit-leave-empty",
        || IncomeSource::Record { value: crate::IncomeRecord::Business(bad) },
        || IncomeSource::Record { value: crate::IncomeRecord::Business(equal) },
        "PAID_LEAVE_NOTHING_CLAIMED",
        "more/paid_leave/employees",
    );
}

fn credit_carry(credit: BusinessCredit, year: u32) -> BusinessCreditCarryforward {
    BusinessCreditCarryforward {
        credit,
        origin_year: year,
        amount: 100,
        passive: None,
        filer: crate::CarryFiler::SameFilers,
        research_source: None,
    }
}
fn credit_history_case(
    case: &str,
    bad: BusinessCreditHistory,
    corrected: BusinessCreditHistory,
    key: &str,
    local: &str,
) {
    let mut seed = Interview::new(TEST_USER, complete(FederalStatus::Single));
    if bad
        .credits
        .iter()
        .chain(&corrected.credits)
        .any(|item| matches!(item.source, crate::CreditSource::Business { .. }))
    {
        accepted(&save_businesses(&mut seed, vec![business(0)]));
    }
    let (mut session, _) = at_the_credits_hub(seed.draft);
    let request = save_more_credits(
        &session.draft,
        MoreCredits { answers: Vec::new(), credits: vec![MoreCredit::BusinessCredits(bad.into())], documents: vec![] },
    );
    credit_refusal(case, &mut session, request, key, &alloc::format!("[value/credits/0/{local}]"));
    let request = save_more_credits(
        &session.draft,
        MoreCredits { answers: Vec::new(),
            credits: vec![MoreCredit::BusinessCredits(corrected.clone().into())],
            documents: vec![],
        },
    );
    credit_accept(&mut session, request);
    assert_eq!(session.draft.credits.as_ref().unwrap().business_credits.as_ref(), Some(&corrected));
    let saved = session.draft.clone();
    let restored = restart(saved.clone());
    assert_eq!(restored.draft, saved);
}

fn credit_entered(
    credit: BusinessCredit,
    source: crate::CreditSource,
    election: crate::CreditElection,
    amount: i64,
) -> BusinessCreditHistory {
    let mut history = empty_business_credit_history();
    history.credits = vec![crate::EnteredBusinessCredit {
        credit,
        source,
        election,
        amount,
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
    }];
    history
}

#[test]
fn unit47_credit_elections_keep_source_amount_and_descriptor_paths() {
    use crate::{CreditElection as E, CreditSource as S};
    let own = || S::Business { business: None, reduced: true };
    let carbon = BusinessCredit::CarbonOxide;
    credit_history_case(
        "credit-election-credit",
        credit_entered(
            BusinessCredit::WorkOpportunity,
            own(),
            E::ElectivePayment { amount: 100 },
            100,
        ),
        credit_entered(carbon, own(), E::ElectivePayment { amount: 100 }, 100),
        "ELECTIVE_PAYMENT_NOT_FOR_THIS_CREDIT",
        "credits/0/credit",
    );
    credit_history_case(
        "credit-election-source",
        credit_entered(carbon, S::Bought, E::ElectivePayment { amount: 100 }, 100),
        credit_entered(carbon, own(), E::ElectivePayment { amount: 100 }, 100),
        "ELECTIVE_PAYMENT_OWN_PROPERTY",
        "credits/0/source",
    );
    for sold in [false, true] {
        let election = |amount| {
            if sold { E::Sold { amount } } else { E::ElectivePayment { amount } }
        };
        credit_history_case(
            if sold { "credit-sale-amount" } else { "credit-payment-amount" },
            credit_entered(carbon, own(), election(200), 100),
            credit_entered(carbon, own(), election(100), 100),
            "BUSINESS_CREDIT_ELECTION_ABOVE_CREDIT",
            "credits/0/election/amount",
        );
    }
    credit_history_case(
        "credit-bought-sale",
        credit_entered(carbon, S::Bought, E::Sold { amount: 100 }, 100),
        credit_entered(carbon, S::Bought, E::None, 100),
        "BOUGHT_CREDIT_CANNOT_BE_SOLD",
        "credits/0/source",
    );
    credit_history_case(
        "credit-transfer-kind",
        credit_entered(BusinessCredit::WorkOpportunity, own(), E::Sold { amount: 100 }, 100),
        credit_entered(carbon, own(), E::Sold { amount: 100 }, 100),
        "TRANSFER_NOT_FOR_THIS_CREDIT",
        "credits/0/credit",
    );
    let cap = crate::year_constants::DISABLED_ACCESS_CREDIT_LIMIT_CENTS;
    credit_history_case(
        "credit-disabled-access-cap",
        credit_entered(BusinessCredit::DisabledAccess, own(), E::None, cap + 100),
        credit_entered(BusinessCredit::DisabledAccess, own(), E::None, cap),
        "DISABLED_ACCESS_ABOVE_LIMIT",
        "credits/0/amount",
    );
}

#[test]
fn unit47_credit_catalog_first_years_reject_previous_year_and_accept_boundary() {
    for (credit, year) in [
        (BusinessCredit::SmallEmployerHealth, 2010),
        (BusinessCredit::AutoEnrollment, 2020),
        (BusinessCredit::MilitarySpouse, 2023),
        (BusinessCredit::PaidLeave, 2018),
        (BusinessCredit::LowIncomeHousing, 2008),
        (BusinessCredit::Rehabilitation, 2008),
        (BusinessCredit::CleanElectricity, 2025),
        (BusinessCredit::CleanFuel, 2025),
        (BusinessCredit::AdvancedEnergyProject, 2009),
        (BusinessCredit::TherapeuticDiscovery, 2009),
        (BusinessCredit::AdvancedManufacturingInvestment, 2023),
        (BusinessCredit::ResearchSmallBusiness, 2016),
        (BusinessCredit::DistilledSpirits, 2018),
        (BusinessCredit::AgriculturalChemicals, 2008),
        (BusinessCredit::DifferentialWage, 2008),
        (BusinessCredit::CarbonOxide, 2008),
        (BusinessCredit::CleanVehicle, 2010),
        (BusinessCredit::MidwesternRetention, 2008),
        (BusinessCredit::MidwesternHousing, 2008),
        (BusinessCredit::ZeroEmissionNuclear, 2024),
        (BusinessCredit::SustainableAviationFuel, 2023),
        (BusinessCredit::CleanHydrogen, 2023),
        (BusinessCredit::CommercialCleanVehicle, 2023),
        (BusinessCredit::AdvancedManufacturingProduction, 2023),
        (BusinessCredit::CleanElectricityProduction, 2025),
        (BusinessCredit::NewHireRetention, 2010),
        (BusinessCredit::SmallBusiness2010, 2010),
    ] {
        let mut equal = empty_business_credit_history();
        equal.carryforwards =
            vec![credit_carry(BusinessCredit::WorkOpportunity, 2024), credit_carry(credit, year)];
        let mut bad = equal.clone();
        bad.carryforwards[1].origin_year = year - 1;
        credit_history_case(
            &alloc::format!("credit-first-year-{}", credit.key()),
            bad,
            equal,
            "BUSINESS_CREDIT_BEFORE_ITS_FIRST_YEAR",
            "carryforwards/1/origin_year",
        );
    }
    let mut equal = empty_business_credit_history();
    equal.carryforwards = vec![credit_carry(BusinessCredit::SmallBusiness2010, 2010)];
    let mut bad = equal.clone();
    bad.carryforwards[0].origin_year = 2011;
    credit_history_case(
        "credit-2010-exact",
        bad,
        equal,
        "BUSINESS_CREDIT_BEFORE_ITS_FIRST_YEAR",
        "carryforwards/0/origin_year",
    );
}

#[test]
fn unit47_credit_carryback_catalog_and_identity_use_canonical_owner() {
    for credit in [
        BusinessCredit::SmallBusiness2010,
        BusinessCredit::ExpiredOther,
        BusinessCredit::WelfareToWork,
        BusinessCredit::CommunityDevelopment,
        BusinessCredit::NewHireRetention,
        BusinessCredit::QualifiedElectricVehicle,
        BusinessCredit::ElectingLargePartnership,
        BusinessCredit::SustainableAviationFuel,
        BusinessCredit::MidwesternHousing,
        BusinessCredit::MidwesternRetention,
        BusinessCredit::HurricaneHousing,
        BusinessCredit::HurricaneRetention,
        BusinessCredit::DisasterRetention,
        BusinessCredit::AgriculturalChemicals,
        BusinessCredit::MineRescue,
        BusinessCredit::EnergyEfficientAppliance,
        BusinessCredit::NonconventionalFuel,
        BusinessCredit::IndianEmployment,
        BusinessCredit::RenewalCommunity,
        BusinessCredit::BiofuelProducer,
        BusinessCredit::TherapeuticDiscovery,
        BusinessCredit::AlternativeMotorVehicle,
        BusinessCredit::LowIncomeHousingBefore2008,
        BusinessCredit::EmployerTipFica,
    ] {
        let mut corrected = empty_business_credit_history();
        corrected.carrybacks = vec![BusinessCreditCarryback {
            credit: BusinessCredit::WorkOpportunity,
            year: 2025,
            amount: 100,
            research_source: None,
        }];
        let mut bad = corrected.clone();
        bad.carrybacks[0].credit = credit;
        let key = if credit == BusinessCredit::EmployerTipFica {
            "TIP_FICA_CARRYBACK_ON_ITS_CREDIT"
        } else {
            "BUSINESS_CREDIT_CARRYFORWARD_ONLY"
        };
        credit_history_case(
            &alloc::format!("credit-carryback-{}", credit.key()),
            bad,
            corrected,
            key,
            "carrybacks/0/credit",
        );
    }
    let mut corrected = empty_business_credit_history();
    corrected.carryforwards = vec![credit_carry(BusinessCredit::EmployerTipFica, 2024)];
    let mut bad = corrected.clone();
    bad.carryforwards[0].origin_year = 2025;
    credit_history_case(
        "credit-tip-fica-year",
        bad,
        corrected,
        "TIP_FICA_PRIOR_YEAR_HAS_ITS_OWN_ITEM",
        "carryforwards/0/origin_year",
    );
    let mut corrected = empty_business_credit_history();
    corrected.carryforwards = vec![credit_carry(BusinessCredit::WorkOpportunity, 2024)];
    let mut bad = corrected.clone();
    bad.carryforwards.push(bad.carryforwards[0].clone());
    credit_history_case(
        "credit-carryforward-duplicate",
        bad,
        corrected,
        "BUSINESS_CARRYFORWARD_TWICE",
        "carryforwards/1/credit",
    );
    let mut corrected = empty_business_credit_history();
    corrected.carrybacks = vec![BusinessCreditCarryback {
        credit: BusinessCredit::WorkOpportunity,
        year: 2025,
        amount: 100,
        research_source: None,
    }];
    let mut bad = corrected.clone();
    bad.carrybacks.push(bad.carrybacks[0].clone());
    credit_history_case(
        "credit-carryback-duplicate",
        bad,
        corrected,
        "BUSINESS_CARRYBACK_TWICE",
        "carrybacks/1/credit",
    );
    for (name, credit, invalid, valid) in [
        (
            "housing",
            BusinessCredit::WorkOpportunity,
            crate::PassiveWorksheet::LowIncomeHousing,
            crate::PassiveWorksheet::Other,
        ),
        (
            "ordinary",
            BusinessCredit::LowIncomeHousing,
            crate::PassiveWorksheet::Other,
            crate::PassiveWorksheet::LowIncomeHousing,
        ),
        (
            "rehabilitation",
            BusinessCredit::WorkOpportunity,
            crate::PassiveWorksheet::Rehabilitation,
            crate::PassiveWorksheet::Other,
        ),
    ] {
        let mut corrected = empty_business_credit_history();
        let mut row = credit_carry(credit, 2024);
        row.passive = Some(valid);
        corrected.carryforwards = vec![row];
        let mut bad = corrected.clone();
        bad.carryforwards[0].passive = Some(invalid);
        credit_history_case(
            &alloc::format!("credit-worksheet-{name}"),
            bad,
            corrected,
            "BUSINESS_CREDIT_WORKSHEET_MISMATCH",
            "carryforwards/0/passive",
        );
    }
}

#[test]
fn unit47_credit_k1_direct_sources_keep_exact_second_row_and_union_paths() {
    let mut equal = k1_1041("Source estate");
    equal.ordinary_dividends = 100;
    equal.qualified_dividends = 100;
    let mut bad = equal.clone();
    bad.qualified_dividends = 101;
    credit_document_case(
        "credit-estate-qualified",
        || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(bad)) },
        || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(equal)) },
        "K1_1041_QUALIFIED_ABOVE_2A",
        "qualified_dividends",
    );
    let mut equal = k1_1120s("Source corporation");
    equal.dividends = 100;
    equal.qualified_dividends = 100;
    let mut bad = equal.clone();
    bad.qualified_dividends = 101;
    credit_document_case(
        "credit-scorp-qualified",
        || IncomeSource::K1 { value: crate::K1Document::K1Scorp((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Scorp((equal).into()) },
        "SCORP_QUALIFIED_ABOVE_DIVIDENDS",
        "qualified_dividends",
    );
    let mut equal = k1_1065("Source partnership");
    equal.more.dividends = 100;
    equal.more.qualified_dividends = 100;
    let mut bad = equal.clone();
    bad.more.qualified_dividends = 101;
    credit_document_case(
        "credit-partner-qualified",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((equal).into()) },
        "PARTNER_QUALIFIED_ABOVE_DIVIDENDS",
        "more/qualified_dividends",
    );
}

fn credit_coop_source() -> Patr1099 {
    let mut source = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 1_000,
        nonpatronage: 1_000,
        per_unit_retains: 1_000,
        redeemed: 1_000,
        withholding: 0,
        more: PatrMore::default(),
    };
    patr_parts(&mut source, 0);
    source
}

fn credit_coop_property() -> crate::CoopPropertyRefundInput {
    crate::CoopPropertyRefundInput {
        distribution: CoopDistributionBox::Patronage,
        property: None,
        source: "Actual separate property refund".into(),
        amount: 100,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: crate::CoopPropertyTreatment::Purchase,
    }
}

#[test]
fn unit47_credit_coop_known_property_bounds_do_not_infer_unknown_qualification() {
    for acquired in [false, true] {
        for field in ["qualified_items", "sstb_items", "qualified_payments"] {
            let mut corrected = credit_coop_source();
            let allocation = &mut corrected.more.allocations[0];
            allocation.properties = vec![credit_coop_property()];
            // Both cases deliberately make retained qualification unavailable.
            // The property/source conservation bound is nevertheless known.
            if acquired {
                allocation.redemptions[0].holding = crate::CoopNoticeHolding::Acquired {
                    basis: 1,
                    source: "Actual acquired notice basis record".into(),
                };
            } else {
                allocation.personal.patronage = 100;
                allocation.personal_qualification = None;
            }
            match field {
                "qualified_items" => {
                    allocation.qualified_items = 10;
                    allocation.properties[0].qualified_items = 10;
                    corrected.more.qualified_items = 10;
                }
                "sstb_items" => {
                    allocation.sstb_items = 10;
                    allocation.properties[0].sstb_items = 10;
                    corrected.more.sstb_items = 10;
                }
                _ => {
                    allocation.qualified_payments = 10;
                    allocation.properties[0].qualified_payments = 10;
                    corrected.more.qualified_payments = 10;
                }
            }
            let accepted_allocation: crate::CoopAllocation =
                corrected.more.allocations[0].clone().into();
            assert!(accepted_allocation.valid());
            assert_eq!(accepted_allocation.retained_qualification(), None);
            let mut bad = corrected.clone();
            match field {
                "qualified_items" => bad.more.allocations[0].properties[0].qualified_items = 11,
                "sstb_items" => bad.more.allocations[0].properties[0].sstb_items = 11,
                _ => bad.more.allocations[0].properties[0].qualified_payments = 11,
            }
            credit_document_case(
                &alloc::format!(
                    "credit-coop-known-{}-{field}",
                    if acquired { "acquired" } else { "unknown" }
                ),
                || IncomeSource::Farming(cooperative_document(bad)),
                || IncomeSource::Farming(cooperative_document(corrected)),
                "COOP_ALLOCATION_INVALID",
                &alloc::format!("more/allocations/0/{field}"),
            );
        }
    }
}

#[test]
fn unit47_credit_coop_named_relationship_keys_locate_actual_source() {
    for (field, key) in [
        ("patronage", "COOP_PERSONAL_ABOVE_BOX1"),
        ("nonpatronage", "COOP_PERSONAL_ABOVE_BOX2"),
        ("per_unit_retains", "COOP_PERSONAL_ABOVE_BOX3"),
        ("redeemed", "COOP_PERSONAL_ABOVE_BOX5"),
    ] {
        let mut corrected = credit_coop_source();
        let mut bad = corrected.clone();
        match field {
            "patronage" => {
                corrected.more.allocations[0].personal.patronage = 1_000;
                bad.more.allocations[0].personal.patronage = 1_001;
            }
            "nonpatronage" => {
                corrected.more.allocations[0].personal.nonpatronage = 1_000;
                bad.more.allocations[0].personal.nonpatronage = 1_001;
            }
            "per_unit_retains" => {
                corrected.more.allocations[0].personal.per_unit_retains = 1_000;
                bad.more.allocations[0].personal.per_unit_retains = 1_001;
            }
            _ => {
                corrected.more.allocations[0].personal.redeemed = 1_000;
                bad.more.allocations[0].personal.redeemed = 1_001;
            }
        }
        credit_document_case(
            &alloc::format!("credit-coop-personal-{field}"),
            || IncomeSource::Farming(cooperative_document(bad)),
            || IncomeSource::Farming(cooperative_document(corrected)),
            key,
            &alloc::format!("more/allocations/0/personal/{field}"),
        );
    }
    for payments in [false, true] {
        let mut corrected = credit_coop_source();
        corrected.more.allocations[0].properties =
            vec![credit_coop_property(), credit_coop_property()];
        if payments {
            corrected.more.qualified_payments = 100;
            corrected.more.allocations[0].qualified_payments = 100;
            corrected.more.allocations[0].properties[1].qualified_payments = 100;
        } else {
            corrected.more.qualified_items = 100;
            corrected.more.allocations[0].qualified_items = 100;
            corrected.more.allocations[0].properties[1].qualified_items = 100;
        }
        let mut bad = corrected.clone();
        let (name, key, field) = if payments {
            bad.more.allocations[0].properties[1].qualified_payments = 101;
            ("payments", "COOP_PROPERTY_PAYMENTS_ABOVE_AMOUNT", "qualified_payments")
        } else {
            bad.more.allocations[0].properties[1].sstb_items = 1;
            ("qbi", "COOP_PROPERTY_QBI_ABOVE_AMOUNT", "qualified_items")
        };
        credit_document_case(
            &alloc::format!("credit-coop-property-{name}"),
            || IncomeSource::Farming(cooperative_document(bad)),
            || IncomeSource::Farming(cooperative_document(corrected)),
            key,
            &alloc::format!("more/allocations/0/properties/1/{field}"),
        );
    }
}

fn credit_annual_interest(records: Vec<crate::AnnualDebtRecord>) -> Int1099 {
    let IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(paper) } =
        interest(0).document
    else {
        unreachable!()
    };
    let mut page = native_int1099(paper.payer.name.clone(), 0);
    page.payer = paper.payer;
    page.recipient = paper.recipient;
    page.state_details = paper.state_details;
    page.taxable = records.iter().map(|r| r.amounts.taxable).sum();
    page.more = IntMore {
        us_obligations: records.iter().map(|r| r.amounts.us_obligations).sum(),
        exempt: records.iter().map(|r| r.amounts.exempt).sum(),
        private_activity: records.iter().map(|r| r.amounts.private_activity).sum(),
        foreign_tax: records.iter().map(|r| r.amounts.foreign_tax).sum(),
        market_discount: records.iter().map(|r| r.amounts.market_discount).sum(),
        bond_premium: records.iter().map(|r| r.amounts.bond_premium).sum(),
        treasury_bond_premium: records.iter().map(|r| r.amounts.treasury_bond_premium).sum(),
        tax_exempt_bond_premium: records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum(),
        accrued_interest: records
            .iter()
            .map(|r| r.amounts.accrued_interest + r.amounts.treasury_accrued_interest)
            .sum(),
        ..Default::default()
    };
    page.adjustments = Some(crate::InterestAdjustments {
        instrument_records: Some(crate::AnnualDebtRecords { records }),
        ..Default::default()
    });
    page
}

fn credit_annual_row() -> crate::AnnualDebtRecord {
    crate::AnnualDebtRecord {
        instrument: "Actual later annual worksheet".into(),
        amounts: crate::DebtComponents { taxable: 200, ..Default::default() },
        ..Default::default()
    }
}

#[test]
fn unit47_credit_annual_repeated_total_key_identifies_actual_payer_component() {
    for field in [
        "taxable",
        "us_obligations",
        "exempt",
        "private_activity",
        "foreign_tax",
        "market_discount",
        "bond_premium",
        "treasury_bond_premium",
        "tax_exempt_bond_premium",
        "accrued_interest",
    ] {
        let mut row = credit_annual_row();
        row.amounts = Default::default();
        match field {
            "taxable" => row.amounts.taxable = 100,
            "us_obligations" => row.amounts.us_obligations = 100,
            "exempt" => row.amounts.exempt = 100,
            "private_activity" => {
                row.amounts.exempt = 200;
                row.amounts.private_activity = 100;
            }
            "foreign_tax" => row.amounts.foreign_tax = 100,
            "market_discount" => row.amounts.market_discount = 100,
            "bond_premium" => {
                row.amounts.taxable = 200;
                row.amounts.bond_premium = 100;
            }
            "treasury_bond_premium" => {
                row.amounts.us_obligations = 200;
                row.amounts.treasury_bond_premium = 100;
            }
            "tax_exempt_bond_premium" => {
                row.amounts.exempt = 200;
                row.amounts.tax_exempt_bond_premium = 100;
            }
            _ => {
                row.amounts.taxable = 200;
                row.amounts.accrued_interest = 100;
            }
        }
        let corrected = credit_annual_interest(vec![row]);
        let mut bad = corrected.clone();
        match field {
            "taxable" => bad.taxable += 1,
            "us_obligations" => bad.more.us_obligations += 1,
            "exempt" => bad.more.exempt += 1,
            "private_activity" => bad.more.private_activity += 1,
            "foreign_tax" => bad.more.foreign_tax += 1,
            "market_discount" => bad.more.market_discount += 1,
            "bond_premium" => bad.more.bond_premium += 1,
            "treasury_bond_premium" => bad.more.treasury_bond_premium += 1,
            "tax_exempt_bond_premium" => bad.more.tax_exempt_bond_premium += 1,
            _ => bad.more.accrued_interest += 1,
        }
        credit_document_case(
            &alloc::format!("credit-annual-total-{field}"),
            || IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(bad)),
            },
            || IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(corrected)),
            },
            "DEBT_INSTRUMENT_TOTAL_MISMATCH",
            &if field == "taxable" { field.into() } else { alloc::format!("more/{field}") },
        );
    }
}

#[test]
fn unit47_credit_annual_index_and_source_character_are_not_key_global_mappings() {
    let corrected = credit_annual_interest(vec![credit_annual_row(), credit_annual_row()]);
    let mut bad = corrected.clone();
    let rows = &mut bad.adjustments.as_mut().unwrap().instrument_records.as_mut().unwrap().records;
    rows[1].nominee = Some(crate::DebtComponents { taxable: 201, ..Default::default() });
    credit_document_case(
        "credit-annual-outside-later",
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(bad)),
        },
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(corrected.clone())),
        },
        "DEBT_OUTSIDE_COMPONENT_OVERLAP",
        "adjustments/instrument_records/records/1/amounts/taxable",
    );
    let mut bad = corrected.clone();
    let rows = &mut bad.adjustments.as_mut().unwrap().instrument_records.as_mut().unwrap().records;
    rows[1].amounts.taxable = 0;
    rows[1].amounts.oid = 200;
    bad.taxable = 200;
    credit_document_case(
        "credit-annual-wrong-later",
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(bad)),
        },
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(corrected)),
        },
        "DEBT_WRONG_STATEMENT_COMPONENT",
        "adjustments/instrument_records/records/1/amounts/oid",
    );
}

fn credit_estate_period(weeks: bool) -> crate::EstateCurrentPeriodPage {
    if weeks {
        crate::EstateCurrentPeriodPage {
            began: 20251229,
            ended: 20270103,
            accounting: crate::EstateCurrentAccountingPage::Weeks52Or53 {
                annual_began: 20251229,
                annual_ended: 20270103,
                reference_ended: 20261231,
                closing: crate::FinalEstateTrustWeekClosing::Nearest,
            },
        }
    } else {
        crate::EstateCurrentPeriodPage {
            began: 20260101,
            ended: 20261231,
            accounting: crate::EstateCurrentAccountingPage::CalendarFiscalOrShort,
        }
    }
}

#[test]
fn unit47_credit_estate_current_period_uses_canonical_date_owner_and_page_projection() {
    for case in 0..11 {
        let corrected = credit_estate_period(case >= 3);
        let mut bad = corrected.clone();
        let path = match case {
            0 => {
                bad.began = 20270101;
                "ended"
            }
            1 => {
                bad.began = 20250101;
                "period/ended"
            }
            2 => {
                bad.began = 20251229;
                bad.accounting =
                    crate::EstateCurrentAccountingPage::CalendarFiscalAfterWeeks52Or53 {
                        annual_began: 20260102,
                    };
                "accounting/annual_began"
            }
            3 => {
                let crate::EstateCurrentAccountingPage::Weeks52Or53 { annual_ended, .. } =
                    &mut bad.accounting
                else {
                    unreachable!()
                };
                *annual_ended = 20270104;
                "accounting/annual_ended"
            }
            4 => {
                bad.began = 20251222;
                "period/began"
            }
            5 => {
                let crate::EstateCurrentAccountingPage::Weeks52Or53 { reference_ended, .. } =
                    &mut bad.accounting
                else {
                    unreachable!()
                };
                *reference_ended = 20261230;
                "accounting/reference_ended"
            }
            6 => {
                bad.began = 20260102;
                bad.ended = 20261231;
                bad.accounting = crate::EstateCurrentAccountingPage::Weeks52Or53 {
                    annual_began: 20260102,
                    annual_ended: 20261231,
                    reference_ended: 20261231,
                    closing: crate::FinalEstateTrustWeekClosing::Last,
                };
                "accounting/annual_began"
            }
            7 => {
                bad.began = 20260105;
                let crate::EstateCurrentAccountingPage::Weeks52Or53 { annual_began, .. } =
                    &mut bad.accounting
                else {
                    unreachable!()
                };
                *annual_began = 20260105;
                "accounting/annual_began"
            }
            8 => {
                let crate::EstateCurrentAccountingPage::Weeks52Or53 { closing, .. } =
                    &mut bad.accounting
                else {
                    unreachable!()
                };
                *closing = crate::FinalEstateTrustWeekClosing::Last;
                "accounting/annual_ended"
            }
            9 => {
                bad.began = 20251230;
                bad.ended = 20270104;
                let crate::EstateCurrentAccountingPage::Weeks52Or53 {
                    annual_began,
                    annual_ended,
                    ..
                } = &mut bad.accounting
                else {
                    unreachable!()
                };
                *annual_began = 20251230;
                *annual_ended = 20270104;
                "accounting/annual_ended"
            }
            _ => {
                bad = crate::EstateCurrentPeriodPage {
                    began: 20250101,
                    ended: 20251231,
                    accounting: crate::EstateCurrentAccountingPage::CalendarFiscalOrShort,
                };
                "period/ended"
            }
        };
        let mut good_page = k1_1041("Actual current-period source");
        good_page.entity_period = Some(corrected);
        let mut bad_page = good_page.clone();
        bad_page.entity_period = Some(bad);
        credit_document_case(
            &alloc::format!("credit-estate-period-{case}"),
            || IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(bad_page)),
            },
            || IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(good_page)),
            },
            "ESTATE_SOURCE_PERIOD_INVALID",
            &alloc::format!("entity_period/{path}"),
        );
    }
}

#[test]
fn unit47_credit_k1_lists_use_actual_duplicate_sign_and_confirmation_positions() {
    macro_rules! cases {
        ($maker:ident,$variant:ident,$income:ident,$income_code:ident,$deduction:ident,$deduction_code:ident,$special:expr,$unentered:expr,$boxkey:literal) => {
            for index in 0..10 {
                let mut corrected = $maker("Actual K1 list source");
                let mut bad = corrected.clone();
                let (key, path) = match index {
                    0 => {
                        corrected.more.income = vec![crate::$income {
                            code: crate::$income_code::PortfolioIncome,
                            amount: 100,
                        }];
                        bad = corrected.clone();
                        // D13, 2026-10-16: duplicate ownership supplies the
                        // actual row even beyond the retired catalog count.
                        bad.more.income =
                            vec![bad.more.income[0].clone(); crate::$income_code::ALL.len() + 1];
                        ("K1_INCOME_CODE_TWICE", "more/income/1/code")
                    }
                    1 => {
                        corrected.more.income = vec![crate::$income {
                            code: crate::$income_code::Section1256,
                            amount: -100,
                        }];
                        bad = corrected.clone();
                        bad.more.income[0].amount = 0;
                        ("K1_INCOME_CODE_SIGN", "more/income/0/amount")
                    }
                    2 => {
                        corrected.more.income = vec![crate::$income {
                            code: crate::$income_code::PortfolioIncome,
                            amount: 1,
                        }];
                        bad = corrected.clone();
                        bad.more.income[0].amount = -1;
                        ("K1_INCOME_CODE_SIGN", "more/income/0/amount")
                    }
                    3 => {
                        corrected.more.deductions = vec![crate::$deduction {
                            code: crate::$deduction_code::CashThirty,
                            amount: 100,
                        }];
                        bad = corrected.clone();
                        bad.more.deductions = vec![
                            bad.more.deductions[0].clone();
                            crate::$deduction_code::ALL.len() + 1
                        ];
                        ("K1_DEDUCTION_CODE_TWICE", "more/deductions/1/code")
                    }
                    4 => {
                        corrected.more.credits = vec![
                            crate::K1Credit {
                                code: crate::K1CreditCode::WorkOpportunity,
                                line: None,
                                amount: 100,
                                carbon_oxide_origin: None,
                                renewable_facility_origin: None,
                            }
                            .into(),
                        ];
                        bad = corrected.clone();
                        bad.more.credits.push(bad.more.credits[0].clone());
                        ("K1_CREDIT_CODE_TWICE", "more/credits/1/code")
                    }
                    5 => {
                        corrected.special = vec![$special];
                        bad = corrected.clone();
                        bad.special = vec![$special; 8];
                        ("K1_SITUATION_TWICE", "special/1")
                    }
                    6 => {
                        corrected.more.unentered = vec![$unentered];
                        bad = corrected.clone();
                        bad.more.unentered = vec![$unentered; 17];
                        ($boxkey, "more/unentered/1")
                    }
                    7 => {
                        bad.final_k1 = true;
                        ("K1_FINAL_NEEDS_DISPOSITION", "disposition")
                    }
                    8 => {
                        corrected.rentals.disposed = vec![crate::K1RentalActivity::RealEstate];
                        bad = corrected.clone();
                        bad.rentals.disposed.push(crate::K1RentalActivity::RealEstate);
                        ("K1_RENTAL_DISPOSED_TWICE", "rentals/disposed/1")
                    }
                    _ => {
                        corrected.more.credits = vec![
                            crate::K1Credit {
                                code: crate::K1CreditCode::LowIncomeHousingJ5,
                                line: None,
                                amount: 100,
                                carbon_oxide_origin: None,
                                renewable_facility_origin: None,
                            }
                            .into(),
                        ];
                        corrected.more.housing_confirmed =
                            Some(crate::K1HousingAcquisition::After1989);
                        corrected.rentals.participation = Some(crate::K1RentalParticipation::NotActive);
                        bad = corrected.clone();
                        bad.more.housing_confirmed = None;
                        ("K1_HOUSING_NEEDS_CONFIRMATION", "more/housing_confirmed")
                    }
                };
                // Neither K-1 paper prints an unentered-item list: the items are
                // derived from its statements, so a repeated one cannot be submitted.
                if index == 6 {
                    assert_eq!(bad.rule(), Some($boxkey));
                    continue;
                }
                credit_document_case(
                    &alloc::format!("credit-lists-{}-{index}", stringify!($variant)),
                    || IncomeSource::K1 { value: crate::K1Document::$variant(bad.into()) },
                    || IncomeSource::K1 { value: crate::K1Document::$variant(corrected.into()) },
                    key,
                    path,
                );
            }
        };
    }
    cases!(
        k1_1120s,
        K1Scorp,
        ScorpIncome,
        ScorpIncomeCode,
        ScorpDeduction,
        ScorpDeductionCode,
        crate::ScorpSpecial::Farming,
        ScorpK1Item::Credits,
        "SCORP_BOX_TWICE"
    );
    cases!(
        k1_1065,
        K1Partnership,
        PartnerIncome,
        PartnerIncomeCode,
        PartnerDeduction,
        PartnerDeductionCode,
        crate::PartnerSpecial::RetiringPayments,
        PartnerK1Item::Credits,
        "PARTNER_BOX_TWICE"
    );
    let corrected = k1_1041("Actual estate classification source");
    let mut bad = corrected.clone();
    bad.more.unentered = vec![EstateTrustK1Item::Credits; 14];
    // The estate K-1 paper derives its unentered items from the printed rows, so
    // a repeated item cannot be submitted; the page rule still refuses it.
    assert_eq!(bad.rule(), Some("K1_1041_BOX_TWICE"));
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = credit_save_rows(
        &mut session,
        TaxSection::PassThrough,
        vec![
            document(
                0,
                IncomeSource::K1 {
                    value: crate::K1Document::K1EstateTrust(estate_document(corrected.clone())),
                },
            ),
            document(
                0,
                IncomeSource::K1 {
                    value: crate::K1Document::K1EstateTrust(estate_document(corrected)),
                },
            ),
        ],
    );
    credit_accept(&mut session, request);
    credit_resave("credit-estate-classifications", &mut session, TaxSection::PassThrough, true);
    let result = supported(&session.draft);
    assert_eq!(result.tax, Some(502_300));
    assert_eq!(result.refund, Some(147_700));
}

#[test]
fn unit47_credit_partner_disposition_services_and_signed_source_bounds() {
    let mut corrected = k1_1065("Actual signed source");
    corrected.more.section1231 = -100;
    corrected.qbi = PartnerQbi::Statement {
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        reit_dividends: 0,
        ptp_income: 0,
        section1231_qbi: -100,
    };
    let mut bad = corrected.clone();
    let PartnerQbi::Statement { section1231_qbi, .. } = &mut bad.qbi else { unreachable!() };
    *section1231_qbi = -101;
    credit_document_case(
        "credit-partner-qbi-signed",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
        "K1_SECTION1231_QBI_ABOVE_BOX",
        "qbi/section1231_qbi",
    );
    let mut corrected = k1_1065("Actual medical source");
    corrected.services = 100;
    corrected.more.medical_insurance = 100;
    let mut bad = corrected.clone();
    bad.more.medical_insurance = 101;
    credit_document_case(
        "credit-partner-medical",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
        "PARTNER_MEDICAL_ABOVE_4A",
        "more/medical_insurance",
    );
    let mut corrected = k1_1065("Actual limited source");
    // The rental participation is asked only with rental real estate.
    corrected.rentals.real_estate = 100_000;
    corrected.rentals.participation = Some(crate::K1RentalParticipation::NotActive);
    (corrected.participation, corrected.limited_for_se) =
        (Some((LimitedParticipation::Investor { prior_years: 0 }).record()), Some(true));
    let mut bad = corrected.clone();
    bad.rentals.participation = Some(crate::K1RentalParticipation::Active);
    credit_document_case(
        "credit-partner-limited",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
        "PARTNER_LIMITED_NOT_ACTIVE",
        "rentals/participation",
    );
    let corrected = k1_1065("Actual retained interest");
    let mut bad = corrected.clone();
    bad.more.sale_ordinary = 100;
    credit_document_case(
        "credit-partner-sale-code",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
        "K1_SALE_CODES_NEED_SALE",
        "disposition",
    );
    let mut corrected = k1_1065("Actual cash liquidation");
    corrected.basis.start = k1_1065_continuing(0, ScorpRiskStart::Unknown);
    corrected.disposition = Some(PartnerDisposition::LiquidatedForCash);
    corrected.more.term = K1StockTerm::LongTerm;
    corrected.final_k1 = true;
    let mut bad = corrected.clone();
    bad.final_k1 = false;
    credit_document_case(
        "credit-partner-liquidation",
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
        "K1_LIQUIDATION_NEEDS_FINAL",
        "final_k1",
    );
}

#[test]
fn unit47_credit_scorp_compound_sources_select_actual_leaf() {
    // Canonical source-owner boundary: equality is accepted, one cent
    // past it is rejected with the source key and actual editable leaf.
    // The page's "Losses carried from 2025" branch, opened with every amount zero.
    fn continuing(page: &mut K1ScorpPaper) -> &mut K1ScorpCarried {
        if matches!(page.basis.start, K1ScorpStart::New) {
            page.basis.start = k1_1120s_continuing(100_000, 100_000);
        }
        let K1ScorpStart::Continuing { carried, .. } = &mut page.basis.start else {
            unreachable!()
        };
        carried
    }
    type Edit = Box<dyn Fn(&mut K1ScorpPaper, i64)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "SCORP_QUALIFIED_ABOVE_DIVIDENDS",
            Box::new(|p, extra| {
                p.dividends = 100_000;
                p.qualified_dividends = 100_000 + extra;
            }),
        ),
        (
            "SCORP_QBI_ABOVE_BOX_1",
            Box::new(|p, extra| {
                p.ordinary = 100_000;
                p.qbi = K1ScorpQbi::Statement {
                    ptp_income: 0,
                    reit_dividends: 0,
                    income: 100_000 + extra,
                    wages: 0,
                    property: 0,
                    service: false,
                    current_loss: 0,
                    property_qbi: no_property_qbi(),
                };
            }),
        ),
        (
            "SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
            Box::new(|p, extra| {
                let K1ScorpOtherBoxes { private_activity, exempt_interest, .. } =
                    k1_1120s_more_mut(p);
                *exempt_interest = 50_000;
                *private_activity = 50_000 + extra;
            }),
        ),
        (
            "SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION",
            Box::new(|p, extra| {
                // A zero carryover needs no election; one cent does.
                {
                    let K1ScorpCarried { basis_nondeductible, .. } = continuing(p);
                    *basis_nondeductible = extra;
                }
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { basis_long, basis_collectibles, .. } = continuing(p);
                *basis_long = 20_000;
                *basis_collectibles = 20_000 + extra;
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { risk_long, risk_collectibles, .. } = continuing(p);
                *risk_long = 20_000;
                *risk_collectibles = 20_000 + extra;
            }),
        ),
    ];
    let paths = [
        "qualified_dividends",
        "qbi/income",
        "more/private_activity",
        "basis/start/carried/losses_first",
        "basis/start/carried/basis_collectibles",
        "basis/start/carried/risk_collectibles",
    ];
    for (index, ((key, edit), path)) in cases.into_iter().zip(paths).enumerate() {
        let mut corrected = k1_1120s("Actual boundary source");
        edit(&mut corrected, 0);
        let mut bad = k1_1120s("Actual boundary source");
        edit(&mut bad, 1);
        credit_document_case(
            &alloc::format!("unit47_credit_scorp_compound_sources_select_actual_leaf-{}", index),
            || IncomeSource::K1 { value: crate::K1Document::K1Scorp((bad).into()) },
            || IncomeSource::K1 { value: crate::K1Document::K1Scorp((corrected).into()) },
            key,
            path,
        );
    }
}

#[test]
fn unit47_credit_partner_compound_sources_select_actual_leaf() {
    type Edit = Box<dyn Fn(&mut K1PartnershipPaper, i64)>;
    fn carried(page: &mut K1PartnershipPaper, field: usize, extra: i64) {
        let mut start = k1_1065_carried_zero();
        {
            let PartnerCarried {
                basis_loss,
                risk_loss,
                passive_loss,
                se_basis,
                se_risk,
                se_passive,
                ..
            } = &mut start;
            *basis_loss = 10_000;
            *risk_loss = 10_000;
            *passive_loss = 10_000;
            *[se_basis, se_risk, se_passive][field] = 10_000 + extra;
        }
        page.basis.start = BasisStart::Continuing {
            opening: Some(100_000),
            risk_history: ScorpRiskStart::Unknown,
            carried: start,
        };
    }
    let cases: Vec<(&str, Edit)> = vec![
        (
            "PARTNER_DEBT_ABOVE_LIABILITIES",
            Box::new(|p, extra| {
                p.liabilities_start = 50_000;
                p.basis.debt_start = 50_000 + extra;
            }),
        ),
        (
            "PARTNER_DEBT_ABOVE_LIABILITIES",
            Box::new(|p, extra| {
                p.liabilities_end = 50_000;
                p.basis.debt_end = 50_000 + extra;
            }),
        ),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 0, extra))),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 1, extra))),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 2, extra))),
        (
            "PARTNER_QUALIFIED_ABOVE_DIVIDENDS",
            Box::new(|p, extra| {
                p.more = k1_1065_more();
                {
                    let K1PartnershipOtherBoxes { dividends, qualified_dividends, .. } =
                        &mut p.more;
                    *dividends = 30_000;
                    *qualified_dividends = 30_000 + extra;
                }
            }),
        ),
    ];
    let paths = [
        "basis/debt_start",
        "basis/debt_end",
        "basis/start/carried/se_basis",
        "basis/start/carried/se_risk",
        "basis/start/carried/se_passive",
        "more/qualified_dividends",
    ];
    for (index, ((key, edit), path)) in cases.into_iter().zip(paths).enumerate() {
        let mut corrected = k1_1065("Actual boundary source");
        edit(&mut corrected, 0);
        let mut bad = k1_1065("Actual boundary source");
        edit(&mut bad, 1);
        credit_document_case(
            &alloc::format!("unit47_credit_partner_compound_sources_select_actual_leaf-{}", index),
            || IncomeSource::K1 { value: crate::K1Document::K1Partnership((bad).into()) },
            || IncomeSource::K1 { value: crate::K1Document::K1Partnership((corrected).into()) },
            key,
            path,
        );
    }
}

#[test]
fn unit47_credit_estate_box_sources_select_actual_leaf() {
    type Edit = Box<dyn Fn(&mut K1EstateTrust, i64)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "K1_1041_QUALIFIED_ABOVE_2A",
            Box::new(|p, extra| {
                p.ordinary_dividends = 100_000;
                p.qualified_dividends = 60_000;
                p.more.section199a_dividends = 40_000 + extra;
            }),
        ),
        (
            "K1_1041_4B_4C_ABOVE_4A",
            Box::new(|p, extra| {
                p.long_gain = 100_000;
                p.rate_28_gain = 30_000;
                p.unrecaptured_gain = 70_000 + extra;
            }),
        ),
        (
            "K1_1041_PRIVATE_ABOVE_14A",
            Box::new(|p, extra| {
                p.more.tax_exempt_interest = 50_000;
                p.more.private_activity_interest = 50_000 + extra;
            }),
        ),
        (
            "K1_1041_ROYALTY_DEDUCTION_ABOVE_BOX_5",
            Box::new(|p, extra| {
                p.royalties = K1EstateTrustRoyalties::Royalties {
                    amount: 20_000,
                    deduction: K1RoyaltyDeduction::Depletion { amount: 20_000 + extra },
                };
            }),
        ),
    ];
    let paths = [
        "qualified_dividends",
        "rate_28_gain",
        "more/private_activity_interest",
        // The 1041 paper prints the royalty depletion among its box 9-11 deductions.
        "deductions",
    ];
    for (index, ((key, edit), path)) in cases.into_iter().zip(paths).enumerate() {
        let mut corrected = k1_1041("Actual boundary source");
        edit(&mut corrected, 0);
        let mut bad = k1_1041("Actual boundary source");
        edit(&mut bad, 1);
        credit_document_case(
            &alloc::format!("unit47_credit_estate_box_sources_select_actual_leaf-{}", index),
            || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(bad)) },
            || IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(corrected)),
            },
            key,
            path,
        );
    }
}

#[test]
fn unit47_credit_estate_final_character_repeated_keys_keep_branch_leaves() {
    for case in 0..8 {
        let mut corrected = k1_1041("Actual final-character source");
        corrected.more.final_year = estate_final_amounts(100);
        corrected.successor = Some(true);
        corrected.final_character = Some(estate_final_source(100, false));
        let K1FinalYear::Final { short_carry, long_carry, .. } = &mut corrected.more.final_year
        else {
            unreachable!()
        };
        *short_carry = 100;
        *long_carry = 100;
        let c = &mut corrected.final_character.as_mut().unwrap().capital;
        c.alternative_short = 100;
        c.alternative_long = 100;
        let mut bad = corrected.clone();
        let f = bad.final_character.as_mut().unwrap();
        let leaf = match case {
            0 => {
                f.section67e_nii = 101;
                "section67e_nii"
            }
            1 => {
                f.capital.rate28 = 101;
                "capital/rate28"
            }
            2 => {
                f.capital.regular_investment =
                    Some(crate::EstateCapitalLossParts { short: 101, long: 0 });
                "capital/regular_investment/short"
            }
            3 => {
                f.capital.regular_business =
                    Some(crate::EstateCapitalLossParts { short: 0, long: 101 });
                "capital/regular_business/long"
            }
            4 => {
                f.capital.alternative_investment =
                    Some(crate::EstateCapitalLossParts { short: 101, long: 0 });
                "capital/alternative_investment/short"
            }
            5 => {
                f.capital.alternative_business =
                    Some(crate::EstateCapitalLossParts { short: 0, long: 101 });
                "capital/alternative_business/long"
            }
            6 => {
                f.capital.niit_difference = Some(201);
                "capital/niit_difference"
            }
            _ => {
                f.section67e_in_h = true;
                "section67e_in_h"
            }
        };
        credit_document_case(
            &alloc::format!("credit-estate-final-{case}"),
            || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(bad)) },
            || IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(corrected)),
            },
            "ESTATE_FINAL_CHARACTER_INVALID",
            &alloc::format!("final_character/{leaf}"),
        );
    }
    let mut corrected = k1_1041("Actual extra deductions");
    corrected.more.final_year = estate_final_amounts(0);
    corrected.successor = Some(true);
    corrected.final_character = Some(estate_final_source(0, false));
    let mut bad = corrected.clone();
    bad.final_mortgages = vec![99, 99];
    credit_document_case(
        "credit-estate-additional-duplicate",
        || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(bad)) },
        || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(corrected)) },
        "ESTATE_FINAL_CHARACTER_INVALID",
        "final_mortgages/1",
    );
}

#[test]
fn unit47_credit_estate_amt_repeated_keys_keep_equation_and_gain_fields() {
    for case in 0..5 {
        let mut corrected = k1_1041("Actual AMT source");
        corrected.long_gain = 100;
        corrected.rate_28_gain = 20;
        corrected.unrecaptured_gain = 30;
        corrected.more.amt = K1EstateTrustAmt::Components { a: 0, b: 0, c: 0, d: 0, e: 0, f: 0 };
        corrected.amt_reconciliation =
            Some(crate::EstateAmtReconciliationPage { other_in_a: 0, adjustments: vec![] });
        let mut bad = corrected.clone();
        let (key, path) = match case {
            0 => {
                bad.amt_reconciliation.as_mut().unwrap().other_in_a = 1;
                ("ESTATE_AMT_RECONCILE", "amt_reconciliation/other_in_a")
            }
            1 => {
                let K1EstateTrustAmt::Components { f, .. } = &mut bad.more.amt else {
                    unreachable!()
                };
                *f = -21;
                ("ESTATE_AMT_RECONCILE", "rate_28_gain")
            }
            2 => {
                let K1EstateTrustAmt::Components { e, .. } = &mut bad.more.amt else {
                    unreachable!()
                };
                *e = -31;
                ("ESTATE_AMT_RECONCILE", "unrecaptured_gain")
            }
            3 => {
                let K1EstateTrustAmt::Components { e, .. } = &mut bad.more.amt else {
                    unreachable!()
                };
                *e = 51;
                ("ESTATE_AMT_RECONCILE", "long_gain")
            }
            _ => {
                bad.amt_reconciliation.as_mut().unwrap().adjustments.push(
                    crate::EstateAmtAdjustmentPage {
                        id: 0,
                        code: crate::EstateAmtCode::DepletionH,
                        amount: 0,
                        treatment: crate::EstateAmtTreatmentPage::Independent {
                            line: crate::EstateAmtLine::Depreciation2l,
                            included_in_a: false,
                        },
                    },
                );
                ("ESTATE_AMT_LINE_INVALID", "amt_reconciliation/adjustments/0/treatment/line")
            }
        };
        credit_document_case(
            &alloc::format!("credit-estate-amt-{case}"),
            || IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(bad)) },
            || IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(corrected)),
            },
            key,
            path,
        );
    }
}

fn credit_set_component(value: &mut crate::DebtComponents, field: &str, amount: i64) {
    *match field {
        "taxable" => &mut value.taxable,
        "us_obligations" => &mut value.us_obligations,
        "oid" => &mut value.oid,
        "other_periodic_interest" => &mut value.other_periodic_interest,
        "treasury_oid" => &mut value.treasury_oid,
        "exempt" => &mut value.exempt,
        "private_activity" => &mut value.private_activity,
        "foreign_tax" => &mut value.foreign_tax,
        "market_discount" => &mut value.market_discount,
        "bond_premium" => &mut value.bond_premium,
        "treasury_bond_premium" => &mut value.treasury_bond_premium,
        "tax_exempt_bond_premium" => &mut value.tax_exempt_bond_premium,
        "acquisition_premium" => &mut value.acquisition_premium,
        "treasury_acquisition_premium" => &mut value.treasury_acquisition_premium,
        "accrued_interest" => &mut value.accrued_interest,
        "treasury_accrued_interest" => &mut value.treasury_accrued_interest,
        _ => unreachable!(),
    } = amount;
}

fn credit_annual_oid(records: Vec<crate::AnnualDebtRecord>) -> Oid1099 {
    let source = native_oid1099(
        "Actual OID worksheet source".into(),
        records.iter().map(|r| r.amounts.oid).sum(),
        records.iter().map(|r| r.amounts.other_periodic_interest).sum(),
    );
    Oid1099 {
        more: OidMore {
            treasury_oid: records.iter().map(|r| r.amounts.treasury_oid).sum(),
            exempt: records.iter().map(|r| r.amounts.exempt).sum(),
            private_activity: records.iter().map(|r| r.amounts.private_activity).sum(),
            market_discount: records.iter().map(|r| r.amounts.market_discount).sum(),
            bond_premium: records
                .iter()
                .map(|r| r.amounts.bond_premium + r.amounts.treasury_bond_premium)
                .sum(),
            acquisition_premium: records
                .iter()
                .map(|r| r.amounts.acquisition_premium + r.amounts.treasury_acquisition_premium)
                .sum(),
            tax_exempt_bond_premium: records
                .iter()
                .map(|r| r.amounts.tax_exempt_bond_premium)
                .sum(),
            ..Default::default()
        },
        adjustments: Some(crate::InterestAdjustments {
            instrument_records: Some(crate::AnnualDebtRecords { records }),
            ..Default::default()
        }),
        ..source
    }
}

#[test]
fn unit47_credit_annual_all_components_keep_signed_owner_bounds_and_actual_indices() {
    for field in [
        "taxable",
        "us_obligations",
        "oid",
        "other_periodic_interest",
        "treasury_oid",
        "exempt",
        "private_activity",
        "foreign_tax",
        "market_discount",
        "bond_premium",
        "treasury_bond_premium",
        "tax_exempt_bond_premium",
        "acquisition_premium",
        "treasury_acquisition_premium",
        "accrued_interest",
        "treasury_accrued_interest",
    ] {
        let oid = matches!(
            field,
            "oid"
                | "other_periodic_interest"
                | "treasury_oid"
                | "acquisition_premium"
                | "treasury_acquisition_premium"
        );
        let mut row = credit_annual_row();
        row.amounts = Default::default();
        let mut outside = crate::DebtComponents::default();
        match field {
            "private_activity" | "tax_exempt_bond_premium" => {
                row.amounts.exempt = 500;
                outside.exempt = 500;
            }
            "bond_premium" | "accrued_interest" | "foreign_tax" => {
                row.amounts.taxable = 500;
                outside.taxable = 500;
            }
            "treasury_bond_premium" | "treasury_accrued_interest" => {
                row.amounts.us_obligations = 500;
                outside.us_obligations = 500;
            }
            "acquisition_premium" => {
                row.amounts.oid = 500;
                outside.oid = 500;
            }
            "treasury_acquisition_premium" => {
                row.amounts.treasury_oid = 500;
                outside.treasury_oid = 500;
            }
            _ => {}
        }
        credit_set_component(&mut row.amounts, field, 200);
        credit_set_component(&mut outside, field, 200);
        row.nominee = Some(outside);
        let first = crate::AnnualDebtRecord {
            instrument: "Earlier actual worksheet".into(),
            ..Default::default()
        };
        let mut bad_row = row.clone();
        credit_set_component(bad_row.nominee.as_mut().unwrap(), field, 201);
        let bad = || {
            let records = vec![first.clone(), bad_row];
            if oid {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(
                        credit_annual_oid(records),
                    )),
                }
            } else {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        credit_annual_interest(records),
                    )),
                }
            }
        };
        let corrected = || {
            let records = vec![first.clone(), row];
            if oid {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(
                        credit_annual_oid(records),
                    )),
                }
            } else {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        credit_annual_interest(records),
                    )),
                }
            }
        };
        credit_document_case(
            &alloc::format!("credit-annual-component-{field}"),
            bad,
            corrected,
            "DEBT_OUTSIDE_COMPONENT_OVERLAP",
            &alloc::format!("adjustments/instrument_records/records/1/amounts/{field}"),
        );
    }
    let row = crate::AnnualDebtRecord {
        instrument: "Signed Treasury source".into(),
        amounts: crate::DebtComponents { treasury_oid: -200, ..Default::default() },
        nominee: Some(crate::DebtComponents { treasury_oid: -200, ..Default::default() }),
        ..Default::default()
    };
    let mut bad = row.clone();
    bad.nominee.as_mut().unwrap().treasury_oid = -201;
    credit_document_case(
        "credit-annual-negative-treasury",
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Oid1099(oid_document(credit_annual_oid(vec![
                bad,
            ]))),
        },
        || IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Oid1099(oid_document(credit_annual_oid(vec![
                row,
            ]))),
        },
        "DEBT_OUTSIDE_COMPONENT_OVERLAP",
        "adjustments/instrument_records/records/0/amounts/treasury_oid",
    );
}

#[test]
fn unit47_credit_k1_sale_carries_holding_and_investment_confirmations_are_transparent() {
    macro_rules! cases {
        ($maker:ident,$variant:ident,$sold:expr,$termfield:ident) => {
            for case in 0..4 {
                let mut corrected = $maker("Actual sale and credit source");
                let mut bad;
                let (key, path) = match case {
                    0 => {
                        corrected.more.section179_sales = vec![k1_sale_input_statement()];
                        corrected.more.section179_carryover = 200_000;
                        corrected.more.section179_carryover_sold = 200_000;
                        bad = corrected.clone();
                        bad.more.section179_carryover_sold += 1;
                        ("K1_SALE_CARRYOVER_ABOVE_CARRYOVER", "more/section179_carryover_sold")
                    }
                    1 => {
                        corrected.more.section179_sales = vec![k1_sale_input_statement()];
                        corrected.more.section179_carryover = 3_000_000;
                        corrected.more.section179_carryover_sold = 2_000_000;
                        bad = corrected.clone();
                        bad.more.section179_carryover_sold += 1;
                        ("K1_SALE_CARRYOVER_ABOVE_SHARE", "more/section179_carryover_sold")
                    }
                    2 => {
                        corrected.disposition = Some($sold);
                        corrected.more.$termfield = K1StockTerm::LongTerm;
                        bad = corrected.clone();
                        bad.more.$termfield = K1StockTerm::NotNeeded;
                        ("K1_SALE_NEEDS_HOLDING_PERIOD", concat!("more/", stringify!($termfield)))
                    }
                    _ => {
                        corrected.more.investment_property =
                            vec![crate::K1InvestmentProperty::RehabilitationOther {
                                id: 0,
                                own_form: None,
                                expenditures: 50_000_000,
                                placed_year: 2025,
                            }];
                        corrected.more.investment_confirmed = Some(K1InvestmentConfirmation::Yes);
                        bad = corrected.clone();
                        bad.more.investment_confirmed = None;
                        ("K1_INVESTMENT_NEEDS_CONFIRMATION", "more/investment_confirmed")
                    }
                };
                credit_document_case(
                    &alloc::format!("credit-sales-{}-{case}", stringify!($variant)),
                    || IncomeSource::K1 { value: crate::K1Document::$variant(bad.into()) },
                    || IncomeSource::K1 { value: crate::K1Document::$variant(corrected.into()) },
                    key,
                    path,
                );
            }
        };
    }
    cases!(k1_1120s, K1Scorp, ScorpDisposition::Sold { proceeds: 0 }, stock_term);
    cases!(k1_1065, K1Partnership, PartnerDisposition::Sold { proceeds: 0 }, term);
}

#[test]
fn unit47_credit_scorp_current_qbi_and_each_history_use_native_owner_metadata() {
    for property in [false, true] {
        let mut corrected = k1_1120s("Actual QBI current source");
        if property {
            corrected.more.section1231 = -100;
        } else {
            corrected.ordinary = -100;
        }
        let K1ScorpQbi::Statement { current_loss, property_qbi, .. } = &mut corrected.qbi else {
            unreachable!()
        };
        if property {
            property_qbi.current = 100;
        } else {
            *current_loss = 100;
        }
        let mut bad = corrected.clone();
        let K1ScorpQbi::Statement { current_loss, property_qbi, .. } = &mut bad.qbi else {
            unreachable!()
        };
        if property {
            property_qbi.current = 101;
        } else {
            *current_loss = 101;
        }
        credit_document_case(
            if property { "credit-qbi-property-current" } else { "credit-qbi-ordinary-current" },
            || IncomeSource::K1 { value: crate::K1Document::K1Scorp((bad).into()) },
            || IncomeSource::K1 { value: crate::K1Document::K1Scorp((corrected).into()) },
            if property {
                "SCORP_QBI_PROPERTY_RECORDS_NEEDED"
            } else {
                "SCORP_QBI_LOSS_RECORDS_NEEDED"
            },
            if property { "qbi/property_qbi/current" } else { "qbi/current_loss" },
        );
        for history in ["history", "at_risk", "passive"] {
            let mut corrected = k1_1120s("Actual retained QBI source");
            let mut carried = k1_1120s_carried_zero();
            *match (property, history) {
                (false, "history") => &mut carried.basis_loss,
                (false, "at_risk") => &mut carried.risk_loss,
                (false, _) => &mut carried.passive_loss,
                (true, "history") => &mut carried.section1231_basis_loss,
                (true, "at_risk") => &mut carried.section1231_risk_loss,
                (true, _) => &mut carried.section1231_passive_loss,
            } = 100;
            corrected.basis.start = K1ScorpStart::Continuing {
                opening: Some(0),
                risk_history: ScorpRiskStart::Unknown,
                carried,
            };
            let records = if property {
                &mut corrected.qbi_records.as_mut().unwrap().property
            } else {
                &mut corrected.qbi_records.as_mut().unwrap().ordinary
            };
            let years = match history {
                "history" => &mut records.history,
                "at_risk" => &mut records.at_risk,
                _ => &mut records.passive,
            };
            years.push(crate::ScorpLossYear {
                year: 2024,
                original: 100,
                qualified: 100,
                remaining: 100,
                remaining_qualified: 100,
            });
            let mut bad = corrected.clone();
            let records = if property {
                &mut bad.qbi_records.as_mut().unwrap().property
            } else {
                &mut bad.qbi_records.as_mut().unwrap().ordinary
            };
            match history {
                "history" => records.history.clear(),
                "at_risk" => records.at_risk.clear(),
                _ => records.passive.clear(),
            };
            credit_document_case(
                &alloc::format!(
                    "credit-qbi-{}-{history}",
                    if property { "property" } else { "ordinary" }
                ),
                || IncomeSource::K1 { value: crate::K1Document::K1Scorp((bad).into()) },
                || IncomeSource::K1 { value: crate::K1Document::K1Scorp((corrected).into()) },
                if property {
                    "SCORP_QBI_PROPERTY_RECORDS_NEEDED"
                } else {
                    "SCORP_QBI_LOSS_RECORDS_NEEDED"
                },
                &alloc::format!(
                    "qbi_records/{}/{history}",
                    if property { "property" } else { "ordinary" }
                ),
            );
        }
    }
    let corrected = k1_1120s("Actual no current property item");
    let mut bad = corrected.clone();
    let K1ScorpQbi::Statement { property_qbi, .. } = &mut bad.qbi else { unreachable!() };
    property_qbi.gain = 1;
    credit_document_case(
        "credit-qbi-no-current-property",
        || IncomeSource::K1 { value: crate::K1Document::K1Scorp((bad).into()) },
        || IncomeSource::K1 { value: crate::K1Document::K1Scorp((corrected).into()) },
        "SCORP_QBI_PROPERTY_RECORDS_NEEDED",
        "qbi/property_qbi/gain",
    );
}

fn credit_estate_source_mut(request: &mut P) -> &mut crate::K1EstateTrustDocument {
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    let crate::K1Document::K1EstateTrust(source) = &mut documents.common.k1.documents[1].document
    else {
        unreachable!()
    };
    source
}

fn credit_estate_existing_case(
    case: &str,
    setup: &dyn Fn(&mut K1EstateTrust),
    edit: &dyn Fn(&mut K1EstateTrust),
    key: &str,
    path: &str,
) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let pages = ["Earlier accepted estate source", "Later accepted estate source"]
        .into_iter()
        .map(|name| {
            let mut p = k1_1041(name);
            p.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
                box6: 50,
                box7: 50,
                box8: 50,
                activities: vec![
                    estate_activity_page("Business", crate::EstateActivityKind::Business, 50),
                    estate_activity_page(
                        "Real estate",
                        crate::EstateActivityKind::RentalRealEstate,
                        50,
                    ),
                    estate_activity_page(
                        "Other rental",
                        crate::EstateActivityKind::OtherRental,
                        50,
                    ),
                ],
            });
            document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(p)) },
            )
        })
        .collect();
    let request = credit_save_rows(&mut session, TaxSection::PassThrough, pages);
    credit_accept(&mut session, request);
    let mut corrected = open_documents(&mut session);
    setup(&mut edit_estate_document(credit_estate_source_mut(&mut corrected)));
    credit_accept(&mut session, corrected);
    let corrected = open_documents(&mut session);
    let mut bad = corrected.clone();
    edit(&mut edit_estate_document(credit_estate_source_mut(&mut bad)));
    let path = credit_document_path(
        &IncomeSource::K1 {
            value: crate::K1Document::K1EstateTrust(credit_estate_source_mut(&mut bad).clone()),
        },
        1,
        path,
    );
    credit_refusal(case, &mut session, bad, key, &path);
    let saved = session.draft.clone();
    restore_accepted_answers(&session.draft, &corrected);
    credit_accept(&mut session, corrected);
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    let page = open_documents(&mut restored);
    accepted(&send(&mut restored, page));
    assert_eq!(restored.draft, saved);
}

fn credit_estate_statement(
    name: &str,
    id: Option<i64>,
    allocated: bool,
) -> crate::EstateQbiStatementPage {
    crate::EstateQbiStatementPage {
        id: 0,
        name: name.into(),
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        kind: crate::EstateQbiKind::Business,
        activities: id.into_iter().collect(),
        passive_history: vec![],
        allocations: if allocated {
            vec![crate::EstateQbiAllocationPage {
                activity: id.unwrap(),
                income: 0,
                passive_history: vec![],
            }]
        } else {
            vec![]
        },
    }
}

#[test]
fn unit47_credit_estate_activity_qbi_and_credit_paths_use_real_assigned_ids() {
    // 2026-10-27: printed_unentered derives unique unsupported sections; the
    // printed form cannot submit duplicate derived section markers.
    for case in 0..22 {
        let setup = |p: &mut K1EstateTrust| {
            let K1EstateTrustBusiness::Amounts(a) = &mut p.business else { unreachable!() };
            let ids: Vec<_> = a.activities.iter().map(|a| a.id).collect();
            if (5..=17).contains(&case) {
                let allocated = (7..=17).contains(&case);
                p.qbi.statements = vec![credit_estate_statement(
                    "First actual QBI source",
                    (case != 5).then_some(ids[0]),
                    allocated,
                )];
                if matches!(case, 5 | 11 | 14 | 15 | 16) {
                    p.qbi.statements.push(credit_estate_statement(
                        "Second actual QBI source",
                        (case != 5).then_some(ids[0]),
                        allocated,
                    ));
                }
                if matches!(case, 7..=9) {
                    p.qbi.statements[0].activities.push(ids[1]);
                    p.qbi.statements[0].allocations.push(crate::EstateQbiAllocationPage {
                        activity: ids[1],
                        income: 0,
                        passive_history: vec![],
                    });
                }
                if case == 12 {
                    a.activities[0].depreciation = 150;
                    p.qbi.statements[0].income = -100;
                    p.qbi.statements[0].allocations[0].income = -100;
                }
                if (13..=17).contains(&case) {
                    a.activities[0].carried.regular = 100;
                    for s in &mut p.qbi.statements {
                        s.allocations[0].passive_history = vec![crate::ScorpLossYear {
                            year: 2024,
                            original: if case == 16 { 200 } else { 100 },
                            qualified: if case == 16 { 100 } else { 25 },
                            remaining: 100,
                            remaining_qualified: if case == 16 { 50 } else { 25 },
                        }];
                    }
                }
            }
            if case == 18 {
                p.investment_property = vec![crate::EstateInvestmentPropertyPage {
                    property: crate::K1InvestmentProperty::RehabilitationOther {
                        id: 0,
                        own_form: None,
                        expenditures: 50_000_000,
                        placed_year: 2025,
                    },
                    activities: vec![ids[0]],
                }];
            }
            if matches!(case, 19..=21) {
                p.reported_credits = vec![crate::EstateReportedCreditPage {
                    code: crate::EstateCreditCode::F,
                    amount: 100,
                    activities: vec![ids[0]],
                }];
            }
        };
        let edit = |p: &mut K1EstateTrust| {
            let K1EstateTrustBusiness::Amounts(a) = &mut p.business else { unreachable!() };
            match case {
                0 => a.activities[1].id = a.activities[0].id,
                1 => a.activities[1].name = a.activities[0].name.clone(),
                2 => a.box6 += 1,
                3 => a.box7 += 1,
                4 => a.box8 += 1,
                5 => p.qbi.statements[1].name = p.qbi.statements[0].name.clone(),
                6 => {
                    let id = p.qbi.statements[0].activities[0];
                    p.qbi.statements[0].activities.push(id);
                }
                7 => {
                    p.qbi.statements[0].allocations[1].activity =
                        p.qbi.statements[0].allocations[0].activity
                }
                8 => {
                    p.qbi.statements[0].allocations.pop();
                }
                9 => p.qbi.statements[0].allocations[1].income = 1,
                10 => p.qbi.statements[0].allocations[0].activity = a.activities[2].id,
                11 => p.qbi.statements[1].service = true,
                12 => {
                    p.qbi.statements[0].income = -101;
                    p.qbi.statements[0].allocations[0].income = -101;
                }
                13 => a.activities[0].carried.regular = 101,
                14 => p.qbi.statements[1].allocations[0].passive_history[0].original = 101,
                15 => {
                    let y = &mut p.qbi.statements[1].allocations[0].passive_history[0];
                    y.qualified = 76;
                    y.remaining_qualified = 76;
                }
                16 => {
                    p.qbi.statements[1].allocations[0].passive_history[0].remaining_qualified = 51
                }
                17 => {
                    p.qbi.statements[0].passive_history =
                        p.qbi.statements[0].allocations[0].passive_history.clone()
                }
                18 => {
                    let id = p.investment_property[0].activities[0];
                    p.investment_property[0].activities.push(id);
                }
                19 => {
                    let id = p.reported_credits[0].activities[0];
                    p.reported_credits[0].activities.push(id);
                }
                20 => p.reported_credits.push(p.reported_credits[0].clone()),
                21 => {
                    p.reported_credits[0].code =
                        crate::EstateCreditCode::I { line: crate::BusinessCredit::Other }
                }
                _ => unreachable!(),
            }
        };
        let (key, path) = match case {
            0 => ("ESTATE_ACTIVITY_ID_TWICE", "business/activities/1/id"),
            1 => ("ESTATE_ACTIVITY_NAME_TWICE", "business/activities/1/name"),
            2 => ("ESTATE_BOX6_RECONCILE", "business/box6"),
            3 => ("ESTATE_BOX7_RECONCILE", "business/box7"),
            4 => ("ESTATE_BOX8_RECONCILE", "business/box8"),
            5 => ("ESTATE_QBI_SOURCE_TWICE", "qbi/statements/1/name"),
            6 => ("ESTATE_SELECTED_ACTIVITY_TWICE", "qbi/statements/0/activities/1"),
            7 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/allocations/1/activity"),
            8 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/allocations"),
            9 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/income"),
            10 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/allocations/0/activity"),
            11 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/1/service"),
            12 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/allocations/0/income"),
            13 => {
                ("ESTATE_QBI_HISTORY_RECONCILE", "qbi/statements/0/allocations/0/passive_history")
            }
            14 => (
                "ESTATE_QBI_HISTORY_RECONCILE",
                "qbi/statements/1/allocations/0/passive_history/0/original",
            ),
            15 => (
                "ESTATE_QBI_HISTORY_RECONCILE",
                "qbi/statements/1/allocations/0/passive_history/0/qualified",
            ),
            16 => (
                "ESTATE_QBI_HISTORY_RECONCILE",
                "qbi/statements/1/allocations/0/passive_history/0/remaining_qualified",
            ),
            17 => ("ESTATE_QBI_ALLOCATION_RECONCILE", "qbi/statements/0/passive_history"),
            18 => ("ESTATE_SELECTED_ACTIVITY_TWICE", "investment_property/0/activities/1"),
            19 => ("ESTATE_SELECTED_ACTIVITY_TWICE", "reported_credits/0/activities/1"),
            20 => ("ESTATE_CREDIT_CODE_TWICE", "reported_credits/1/code"),
            21 => ("ESTATE_CREDIT_LINE_INVALID", "reported_credits/0/code/line"),
            _ => unreachable!(),
        };
        credit_estate_existing_case(
            &alloc::format!("credit-estate-assigned-{case}"),
            &setup,
            &edit,
            key,
            path,
        );
    }
}

#[test]
fn unit47_credit_coop_credit_sum_and_duplicate_expense_keep_source_positions() {
    let mut corrected = credit_coop_source();
    corrected.more.credit_facts.investment = 100;
    corrected.more.credit_facts.investment_shares = vec![crate::CoopInvestmentShare {
        kind: crate::CoopInvestmentKind::Rehabilitation,
        amount: 100,
        source: "Actual issuer credit component".into(),
    }];
    let mut bad = corrected.clone();
    bad.more.credit_facts.investment_shares[0].amount = 101;
    credit_document_case(
        "credit-coop-investment-source",
        || IncomeSource::Farming(cooperative_document(bad)),
        || IncomeSource::Farming(cooperative_document(corrected)),
        "COOP_CREDIT_SOURCE_INVALID",
        "more/credit_facts/investment_shares/0/amount",
    );
    let mut corrected = credit_coop_source();
    corrected.more.allocations[0].sstb_expenses = vec![crate::CoopSstbExpense {
        category: crate::BusinessExpenseCategory::Advertising,
        amount: 100,
        source: "Actual invoice source".into(),
    }];
    // No receiving activity is asserted; source facts remain retained and the
    // unlinked benefit stays unavailable under its existing owner.
    let mut bad = corrected.clone();
    let duplicate = bad.more.allocations[0].sstb_expenses[0].clone();
    bad.more.allocations[0].sstb_expenses.push(duplicate);
    credit_document_case(
        "credit-coop-expense-duplicate",
        || IncomeSource::Farming(cooperative_document(bad)),
        || IncomeSource::Farming(cooperative_document(corrected)),
        "COOP_SSTB_SOURCE_REPEATED",
        "more/allocations/0/sstb_expenses/1/source",
    );
}

#[test]
fn unit47_credit_annual_history_premium_original_and_private_branches_keep_source_leaves() {
    for case in (0..22).filter(|case| *case != 20) {
        let mut good = credit_annual_row();
        let mut key = "DEBT_MIXED_INSTRUMENT_CHARACTERS";
        let mut leaf = "amounts";
        let mut use_oid = false;
        match case {
            0 | 6..=11 | 13 | 17 => {
                good.amounts = crate::DebtComponents { exempt: 200, ..Default::default() };
            }
            4 | 12 => {
                good.amounts = crate::DebtComponents { us_obligations: 200, ..Default::default() };
            }
            18 | 19 => {
                good.amounts = crate::DebtComponents { oid: 200, ..Default::default() };
                use_oid = true;
            }
            _ => {}
        }
        if case == 1 {
            good.reporting = crate::DebtReporting::Net;
            good.net_premium_offset = Some(crate::DebtPremiumOffset { ordinary: 1, treasury: 0 });
        }
        if matches!(case, 6..=10) {
            good.amounts.tax_exempt_bond_premium = 100;
        }
        if matches!(case, 7 | 8) {
            let outside = crate::DebtComponents {
                exempt: 50,
                tax_exempt_bond_premium: 50,
                ..Default::default()
            };
            if case == 7 {
                good.nominee = Some(outside);
            } else {
                good.after_death = Some(outside);
            }
        }
        if matches!(case, 9..=11) {
            good.amounts.private_activity = 150;
            good.private_activity_after_premium = Some(if case == 11 { 150 } else { 100 });
            if case == 11 {
                good.reporting = crate::DebtReporting::Net;
            }
        }
        if case == 12 {
            good.previously_reported = 200;
        }
        if case == 13 {
            good.amounts.private_activity = 200;
        }
        if case == 14 {
            good.premium_history = Some(crate::PremiumHistory {
                prior_net_interest_inclusions: 100,
                prior_premium_deductions: 100,
                premium_carry: 0,
            });
        }
        if matches!(case, 15 | 16 | 18..=21) {
            good.original_issue = Some(crate::OriginalDebtIssue {
                issue_price: if case == 19 { 9750 } else { 9900 },
                redemption_price: 10000,
                complete_years: 10,
                elect_de_minimis_oid: case == 18,
            });
        }
        assert!(
            good.issue().is_none(),
            "annual remaining corrected case {case}: {:?}",
            good.issue()
        );
        let mut bad = good.clone();
        match case {
            0 => {
                bad.premium_history = Some(Default::default());
                key = "DEBT_EXEMPT_PREMIUM_HISTORY";
                leaf = "premium_history";
            }
            1 => {
                bad.net_premium_offset =
                    Some(crate::DebtPremiumOffset { ordinary: 0, treasury: 1 });
                key = "DEBT_NET_OFFSET_CHARACTER";
                leaf = "net_premium_offset";
            }
            2 => bad.amounts.us_obligations = 1,
            3 | 4 => bad.amounts.exempt = 1,
            5 => {
                bad.net_premium_offset = Some(Default::default());
                key = "DEBT_OFFSET_ONLY_FOR_NET";
                leaf = "net_premium_offset";
            }
            6 => {
                bad.nominee = Some(crate::DebtComponents { exempt: 101, ..Default::default() });
                key = "DEBT_EXEMPT_REDUCTION_ABOVE_AMOUNT";
                leaf = "amounts/tax_exempt_bond_premium";
            }
            7 | 8 => {
                let outside = if case == 7 {
                    bad.nominee.as_mut().unwrap()
                } else {
                    bad.after_death.as_mut().unwrap()
                };
                outside.tax_exempt_bond_premium = 51;
                key = "DEBT_EXEMPT_REDUCTION_ABOVE_AMOUNT";
                leaf = if case == 7 {
                    "nominee/tax_exempt_bond_premium"
                } else {
                    "after_death/tax_exempt_bond_premium"
                };
            }
            9..=11 => {
                bad.private_activity_after_premium =
                    if case == 10 { None } else { Some(if case == 11 { 151 } else { 101 }) };
                key = "DEBT_PRIVATE_AFTER_PREMIUM_INVALID";
                leaf = "private_activity_after_premium";
            }
            12 => {
                bad.previously_reported = 201;
                key = "DEBT_PRIOR_ABOVE_RETAINED";
                leaf = "previously_reported";
            }
            13 => {
                bad.nominee = Some(crate::DebtComponents { exempt: 1, ..Default::default() });
                key = "DEBT_PRIVATE_ABOVE_EXEMPT";
                leaf = "amounts/private_activity";
            }
            14 => {
                bad.premium_history.as_mut().unwrap().prior_premium_deductions = 101;
                key = "DEBT_HISTORY_DEDUCTIONS_ABOVE_INCLUSIONS";
                leaf = "premium_history/prior_premium_deductions";
            }
            15 => {
                bad.original_issue.as_mut().unwrap().issue_price = 10001;
                key = "DEBT_ORIGINAL_ISSUE_INVALID";
                leaf = "original_issue/issue_price";
            }
            16 => {
                bad.treatment = crate::AnnualDebtTreatment::Stripped;
                key = "DEBT_ORIGINAL_ISSUE_INVALID";
                leaf = "treatment";
            }
            17 => {
                bad.original_issue = Some(crate::OriginalDebtIssue {
                    issue_price: 9900,
                    redemption_price: 10000,
                    complete_years: 10,
                    elect_de_minimis_oid: false,
                });
                key = "DEBT_ORIGINAL_ISSUE_INVALID";
                leaf = "amounts/exempt";
            }
            18 | 19 => {
                if case == 18 {
                    bad.original_issue.as_mut().unwrap().elect_de_minimis_oid = false;
                } else {
                    assert!(!good.original_issue.as_ref().unwrap().de_minimis());
                    bad.original_issue.as_mut().unwrap().issue_price += 1;
                    assert!(bad.original_issue.as_ref().unwrap().de_minimis());
                }
                key = "DEBT_DE_MINIMIS_OID_CONFLICT";
                leaf = "original_issue/elect_de_minimis_oid";
            }
            _ => {
                bad.original_issue.as_mut().unwrap().complete_years = 101;
                key = "DEBT_ORIGINAL_ISSUE_INVALID";
                leaf = "original_issue/complete_years";
            }
        }
        let issue = bad.issue().expect("annual remaining source refusal");
        assert_eq!(issue.key, key, "annual source case {case}");
        let records = |row| {
            vec![
                crate::AnnualDebtRecord {
                    instrument: "Earlier valid annual instrument".into(),
                    ..Default::default()
                },
                row,
            ]
        };
        let source = |row| {
            if use_oid {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Oid1099(oid_document(
                        credit_annual_oid(records(row)),
                    )),
                }
            } else {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        credit_annual_interest(records(row)),
                    )),
                }
            }
        };
        let mut original = complete(FederalStatus::Single);
        if case == 8 {
            // This source expressly belongs to a final return: after-death
            // components cannot be accepted on an otherwise living return.
            original.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
                date: 20260630,
                ordinary: true,
                bonds: None,
                estate_medical: 0,
            });
        }
        credit_document_case_in_draft(
            &alloc::format!("credit-annual-remaining-{case}"),
            || source(bad),
            || source(good),
            key,
            &alloc::format!("adjustments/instrument_records/records/1/{leaf}"),
            original,
        );
    }
}

#[test]
fn unit47_credit_seller_and_nested_capital_lot_issues_delegate_once_with_actual_fields() {
    for field in ["buyer", "buyer_address"] {
        let good = crate::SellerMortgageInterest {
            buyer: crate::SellerMortgageParty {
                buyer: "Actual buyer".into(),
                buyer_tin: "001234567".into(),
                buyer_address: "Actual purchase address".into(),
            },
            interest: 100,
        };
        let mut bad = good.clone();
        match field {
            "buyer" => bad.buyer.buyer = " ".into(),
            _ => bad.buyer.buyer_address = " ".into(),
        }
        credit_document_case(
            &alloc::format!("credit-seller-buyer-{field}"),
            || IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(bad) },
            || IncomeSource::Record { value: crate::IncomeRecord::SellerMortgageInterest(good) },
            "SELLER_MORTGAGE_BUYER_INVALID",
            &alloc::format!("buyer/{field}"),
        );
    }
    let lot = crate::CapitalReturn1099Div {
        kind: crate::CapitalDistributionKind::FinalLiquidation,
        amount: 100,
        property: vec![crate::LiquidationProperty {
            label: "Actual property received".into(),
            value: 50,
        }],
        basis: 200,
        term: crate::GainTerm::Long,
        special: false,
    };
    let mut good = div1099_source(Div1099 {
        cash_liquidation: 200,
        noncash_liquidation: 100,
        ..native_div1099("Actual liquidation payer".into(), 0, 0)
    });
    good.capital_returns = vec![lot.clone(), lot];
    let mut cases: Vec<(String, crate::DividendSource, &str, String, String)> = Vec::new();
    let mut bad = good.clone();
    bad.capital_returns[1].kind = crate::CapitalDistributionKind::Nondividend;
    cases.push((
        "credit-div-later-property-kind".into(),
        bad,
        "CAPITAL_RETURN_PROPERTY_NOT_LIQUIDATION",
        "capital_returns/1/property".into(),
        "value/1/property".into(),
    ));
    let mut bad = good.clone();
    bad.capital_returns[1].property[0].label = " ".into();
    cases.push((
        "credit-div-later-intrinsic-row".into(),
        bad,
        "DIV_CAPITAL_RETURNS_RECONCILE",
        "capital_returns/1".into(),
        "value/1".into(),
    ));
    for field in ["nondividend_distributions", "cash_liquidation", "noncash_liquidation"] {
        let mut bad = good.clone();
        match field {
            "nondividend_distributions" => {
                bad.form.nondividend_distributions =
                    Some(bad.form.nondividend_distributions.unwrap_or(0) + 1)
            }
            "cash_liquidation" => {
                bad.form.cash_liquidation = Some(bad.form.cash_liquidation.unwrap_or(0) + 1)
            }
            _ => bad.form.noncash_liquidation = Some(bad.form.noncash_liquidation.unwrap_or(0) + 1),
        }
        cases.push((
            alloc::format!("credit-div-capital-total-{field}"),
            bad,
            // The answer page has only its capital-return rows; a printed total
            // that disagrees with them is located on those rows.
            "DIV_CAPITAL_RETURNS_RECONCILE",
            field.into(),
            "value".into(),
        ));
    }
    for (case, bad, key, local, answer_path) in cases {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        credit_local_source_case_in_session(
            &case,
            &mut session,
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(good.form.clone()),
                },
            ),
            (
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(bad.form),
                },
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(good.form.clone()),
                },
            ),
            key,
            &local,
            Some((
                vec![crate::DocumentAnswer::DividendCapitalReturns { value: bad.capital_returns }],
                vec![crate::DocumentAnswer::DividendCapitalReturns {
                    value: good.capital_returns.clone(),
                }],
                &answer_path,
            )),
        );
    }
}

fn credit_graph_counterexample(
    case: &str,
    draft: &FederalDraft,
    source_id: i64,
    key: &str,
    local: &str,
    estate: bool,
) {
    let before = draft.clone();
    let result = if estate {
        crate::estate_trust::graph_issue(draft)
    } else {
        crate::cooperative::graph_issue(draft)
    };
    let (actual_source, issue) = result.expect("contextual graph counterexample");
    assert_eq!(actual_source, source_id, "{case}: actual source ID");
    assert_eq!(issue.key, key, "{case}");
    assert_eq!(issue.field.unwrap().square(), alloc::format!("[{local}]"), "{case}");
    assert_eq!(
        if estate {
            crate::estate_trust::graph_issue(draft).map(|(_, issue)| issue.key)
        } else {
            crate::cooperative::graph_rule(draft)
        },
        Some(key)
    );
    assert_eq!(*draft, before, "{case}: read-only graph owner changed source");
}

#[test]
fn unit47_credit_cooperative_graph_metadata_names_real_sources_and_original_positions() {
    // Source IDs and initial allocation IDs come from the actual native Save.
    // Later mutations are explicit graph counterexamples, not UI/runtime proof.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![business(0)]));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        (0..3)
            .map(|index| {
                let mut source = credit_coop_source();
                if index == 2 {
                    source.more.other_credits_and_deductions = 4001;
                }
                document(0, IncomeSource::Farming(cooperative_document(source)))
            })
            .collect(),
    );
    credit_accept(&mut session, request);
    let base = session.draft.clone();
    assert!(crate::cooperative::graph_issue(&base).is_none());
    let source_ids: Vec<_> = base
        .income
        .iter()
        .filter_map(|entry| {
            matches!(entry.income, FederalIncome::Cooperative { .. }).then_some(entry.id)
        })
        .collect();
    assert_eq!(source_ids.len(), 3);
    let business_id = base
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    let target = source_ids[2];
    let FederalIncome::Cooperative { data } =
        &base.income.iter().find(|e| e.id == target).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.form.other_credits_and_deductions, 4001);
    for case in (1..11).filter(|case| *case != 2) {
        let mut bad = base.clone();
        if case == 8 {
            let FederalIncome::Business { data } =
                &mut bad.income.iter_mut().find(|e| e.id == business_id).unwrap().income
            else {
                unreachable!()
            };
            data.payroll = crate::BusinessPayroll::Yes {
                statements: vec![crate::PayrollW2 {
                    label: "Actual native wage counterexample".into(),
                    wages: 1,
                    medicare: 1,
                }],
                attribution: crate::PayrollQbiAttribution::EntireBusiness,
            };
        }
        let earlier_allocation = {
            let FederalIncome::Cooperative { data } =
                &bad.income.iter().find(|e| e.id == source_ids[0]).unwrap().income
            else {
                unreachable!()
            };
            data.more.allocations[0].id
        };
        let data = match &mut bad.income.iter_mut().find(|e| e.id == target).unwrap().income {
            FederalIncome::Cooperative { data } => data,
            _ => unreachable!(),
        };
        let (key, local) = match case {
            1 => {
                data.more.allocations[0].id = 0;
                ("COOP_ALLOCATION_ID_INVALID", "allocations/0/id")
            }
            3 => {
                let mut property = credit_coop_property();
                property.property = Some(999_999);
                // Conversion preserves every actual source fact; no parcel is inferred.
                data.more.allocations[0].properties.push(property.into());
                ("COOP_PROPERTY_LINK_UNAVAILABLE", "allocations/0/properties/0/property")
            }
            4 => {
                data.form.patronage += 1;
                ("COOP_ALLOCATIONS_DO_NOT_RECONCILE", "allocations")
            }
            5 => {
                data.more.allocations[0].sstb_expenses.push(crate::CoopSstbExpense {
                    category: crate::BusinessExpenseCategory::Advertising,
                    amount: 100_000_000_000,
                    source: "Actual unsupported expense counterexample".into(),
                });
                ("COOP_SSTB_EXPENSE_INVALID", "allocations/0/sstb_expenses/0/amount")
            }
            6 => {
                data.more.credit_facts.activity = Some(999_999);
                ("COOP_RECIPIENT_LINK_INVALID", "credit_facts/activity")
            }
            7 => {
                data.more.credit_facts.other = (0..2)
                    .map(|i| crate::CoopOtherItem {
                        id: 0,
                        kind: crate::CoopOtherKind::CleanFuel,
                        amount: 0,
                        activity: (i == 1).then_some(999_999),
                        income: crate::CoopCreditIncome::Unresolved,
                        source: alloc::format!("Actual source counterexample {i}"),
                    })
                    .collect();
                ("COOP_RECIPIENT_LINK_INVALID", "credit_facts/other/1/activity")
            }
            8 => {
                data.more.allocations[0].patron_allocation =
                    Some(crate::CoopPatronAllocation::NoWages);
                ("COOP_PATRON_WAGES_NOT_ZERO", "allocations/0/patron_allocation")
            }
            9 => {
                data.more.allocations[0].id = earlier_allocation;
                ("COOP_ALLOCATION_ID_INVALID", "allocations/0/id")
            }
            _ => {
                data.more.allocations[0].id = target;
                ("COOP_ALLOCATION_ID_INVALID", "allocations/0/id")
            }
        };
        credit_graph_counterexample(
            &alloc::format!("coop-graph-{case}"),
            &bad,
            target,
            key,
            local,
            false,
        );
        // The same actual source ID remains correct when stored positions change.
        if case != 9 {
            bad.income.reverse();
            credit_graph_counterexample(
                &alloc::format!("coop-graph-reversed-{case}"),
                &bad,
                target,
                key,
                local,
                false,
            );
        }
    }
    let mut bad = base.clone();
    for id in &source_ids[1..] {
        let FederalIncome::Cooperative { data } =
            &mut bad.income.iter_mut().find(|e| e.id == *id).unwrap().income
        else {
            unreachable!()
        };
        data.more.allocations[0].patron_allocation = Some(crate::CoopPatronAllocation::NoWages);
    }
    // The first cooperative is unrelated to the duplicated governing source.
    credit_graph_counterexample(
        "coop-graph-governing-real-source",
        &bad,
        source_ids[2],
        "COOP_PATRON_SOURCE_DUPLICATE",
        "allocations/0/patron_allocation",
        false,
    );
    let mut bad = base.clone();
    for (index, id) in source_ids.iter().enumerate() {
        if index == 1 {
            continue;
        }
        let FederalIncome::Cooperative { data } =
            &mut bad.income.iter_mut().find(|e| e.id == *id).unwrap().income
        else {
            unreachable!()
        };
        data.more.allocations[0].sstb_expenses = vec![
            crate::CoopSstbExpense {
                category: crate::BusinessExpenseCategory::Advertising,
                amount: 0,
                source: alloc::format!("Independent source {index}"),
            },
            crate::CoopSstbExpense {
                category: crate::BusinessExpenseCategory::Advertising,
                amount: 0,
                source: "Same actual cross-source invoice".into(),
            },
        ];
    }
    credit_graph_counterexample(
        "coop-graph-cross-source-invoice",
        &bad,
        source_ids[0],
        "COOP_SSTB_SOURCE_REPEATED",
        "allocations/0/sstb_expenses/1/source",
        false,
    );
    let mut automatic = base.clone();
    let FederalIncome::Cooperative { data } =
        &mut automatic.income.iter_mut().find(|e| e.id == target).unwrap().income
    else {
        unreachable!()
    };
    data.more.allocations.clear();
    let projected = crate::cooperative::sources(&automatic);
    let source = projected.iter().find(|s| s.parent == target).unwrap();
    assert_eq!(source.allocation_index, None, "automatic allocation has no original Page row");
    assert!(projected.iter().filter(|s| s.parent != target).all(|s| s.allocation_index == Some(0)));
    assert_eq!(session.draft, base);
}

#[test]
fn unit47_credit_estate_graph_metadata_keeps_record_ids_and_accepted_field_projection() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let pages = (0..2)
        .map(|i| {
            let mut page = k1_1041(&alloc::format!("Actual accepted estate graph source {i}"));
            page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
                box6: 0,
                box7: 100,
                box8: 0,
                activities: vec![estate_activity_page(
                    "Actual rental source",
                    crate::EstateActivityKind::RentalRealEstate,
                    100,
                )],
            });
            document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(page)) },
            )
        })
        .collect();
    let request = credit_save_rows(&mut session, TaxSection::PassThrough, pages);
    credit_accept(&mut session, request);
    let base = session.draft.clone();
    let source_ids: Vec<_> = base
        .income
        .iter()
        .filter_map(|entry| {
            matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }).then_some(entry.id)
        })
        .collect();
    assert_eq!(source_ids.len(), 2);
    assert!(crate::estate_trust::graph_issue(&base).is_none());
    for case in 0..3 {
        let mut bad = base.clone();
        let FederalIncome::EstateTrustPortfolio { data } =
            &mut bad.income.iter_mut().find(|e| e.id == source_ids[1]).unwrap().income
        else {
            unreachable!()
        };
        let (key, local) = match case {
            0 => {
                data.statements.activities[0].id = 0;
                ("ESTATE_ACTIVITY_LINK_INVALID", "activities/activities/0/id")
            }
            1 => {
                data.statements.activities[0].participation =
                    crate::EstateParticipation::Professional(
                        Participation::Over500Hours { hours: 501 }.record(),
                    );
                ("RENTAL_PROFESSIONAL_NEEDS_STATUS", "activities/activities/0/participation")
            }
            _ => {
                data.answers.final_mortgages = vec![999_999];
                data.form.final_k1 = true;
                data.statements.final_character = Some(estate_final_source(0, false).record());
                ("ESTATE_MORTGAGE_LINK_INVALID", "final_mortgages/0")
            }
        };
        credit_graph_counterexample(
            &alloc::format!("estate-graph-{case}"),
            &bad,
            source_ids[1],
            key,
            local,
            true,
        );
        bad.income.reverse();
        credit_graph_counterexample(
            &alloc::format!("estate-graph-reversed-{case}"),
            &bad,
            source_ids[1],
            key,
            local,
            true,
        );
    }
    let mut bad = base.clone();
    bad.deductions.as_mut().unwrap().set_mortgage_interest(100);
    let loan = bad
        .deductions
        .as_ref()
        .unwrap()
        .mortgage_loans()
        .expect("accepted mortgage source converts")[0]
        .id;
    for id in &source_ids {
        let FederalIncome::EstateTrustPortfolio { data } =
            &mut bad.income.iter_mut().find(|e| e.id == *id).unwrap().income
        else {
            unreachable!()
        };
        data.answers.final_mortgages = vec![loan];
        data.form.final_k1 = true;
        data.statements.final_character = Some(estate_final_source(0, false).record());
    }
    credit_graph_counterexample(
        "estate-graph-real-second-mortgage-source",
        &bad,
        source_ids[1],
        "ESTATE_MORTGAGE_LINK_TWICE",
        "final_mortgages/0",
        true,
    );
    assert_eq!(session.draft, base);
}

fn credit_contextual_document_case(
    case: &str,
    session: &mut Interview,
    topic: TaxSection,
    corrected: P,
    bad: P,
    index: usize,
    expected: (&str, &str),
) {
    let (key, local) = expected;
    let (path, source) = if let P::SaveIncomes { followups, .. } = &bad {
        let id = followups[index].key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        assert!(id > 0, "contextual validation binds an actual saved source");
        (
            local
                .split('/')
                .filter(|segment| !segment.is_empty())
                .fold(
                    SchemaPath::root()
                        .field("followups")
                        .key(followups[index].key.as_str())
                        .field("answer"),
                    |path, segment| match segment.parse::<usize>() {
                        Ok(index) => path.index(index),
                        Err(_) => path.field(segment),
                    },
                )
                .to_string(),
            false,
        )
    } else {
        let P::SaveDocuments { documents, .. } = &bad else { unreachable!() };
        let item = match topic {
            TaxSection::PassThrough => documents.common.k1.documents[index].clone().into_income(),
            _ => unreachable!(),
        };
        assert!(item.id > 0, "contextual Preview binds an actual saved source");
        (credit_document_path(&item.document, index, local), true)
    };
    let before = session.draft.clone();
    credit_refusal(case, session, bad, key, &path);
    restore_accepted_answers(&session.draft, &corrected);
    credit_accept(session, corrected);
    assert_eq!(session.draft, before, "{case}: correction did not preserve accepted sources");
    credit_resave(case, session, topic, source);
}

#[test]
fn unit47_credit_cooperative_contextual_save_preview_use_real_source_and_empty_collection() {
    for case in (0..6).filter(|case| *case != 1) {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save_businesses(&mut session, vec![business(0)]));
        let business_id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
            .unwrap()
            .id;
        let request = save_documents(
            &mut session,
            (0..3)
                .map(|_| {
                    document(0, IncomeSource::Farming(cooperative_document(credit_coop_source())))
                })
                .collect(),
        );
        credit_accept(&mut session, request);
        let paper = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &paper else { unreachable!() };
        let positions: Vec<_> = (0..documents.other.farming.documents.len()).collect();
        assert_eq!(positions.len(), 3);
        let governing = positions[1];
        let target = positions[2];
        assert_eq!(target, 2, "the actual Farming category excludes the separate business record");
        let governing_id = documents.other.farming.documents[governing].id;
        let target_id = documents.other.farming.documents[target].id;
        assert!(target_id > 0, "contextual validation binds an actual saved source");
        if case == 5 {
            let mut changed = paper;
            let P::SaveDocuments { documents, .. } = &mut changed else { unreachable!() };
            documents.other.farming.documents[target].document.qualified_payments = 4_000;
            accepted(&send(&mut session, changed));
            // Printed corrections retain allocations until their own topic repairs them.
            let wages = open_topic(&mut session, TaxSection::Wages);
            accepted(&send(&mut session, wages));
            assert!(session.draft.income.iter().any(|entry| {
                matches!(&entry.income, FederalIncome::Cooperative { data }
                    if data.form.qualified_payments == 4_000
                        && data.more.allocations[0].qualified_payments == 0)
            }));
        }
        let mut setup = open_topic(&mut session, TaxSection::SelfEmployment);
        let P::SaveIncomes { followups, .. } = &mut setup else { unreachable!() };
        match case {
            0 | 2 | 5 => {
                let source = if case == 0 { governing_id } else { target_id };
                let row = followups
                    .iter_mut()
                    .find(|row| row.key == alloc::format!("income/{source}/allocations"))
                    .unwrap();
                let crate::DocumentAnswer::CooperativeAllocations { value } = &mut row.answer
                else {
                    unreachable!()
                };
                match case {
                    0 => value[0].patron_allocation = Some(crate::CoopPatronAllocation::NoWages),
                    2 => value[0].properties = vec![credit_coop_property()],
                    _ => value[0].qualified_payments = 4_000,
                }
            }
            4 => {
                let row = followups
                    .iter_mut()
                    .find(|row| row.key == alloc::format!("income/{target_id}/credit_records"))
                    .unwrap();
                let crate::DocumentAnswer::CooperativeCreditRecords { other, .. } = &mut row.answer
                else {
                    unreachable!()
                };
                *other = (0..2)
                    .map(|i| crate::CoopOtherItemInput {
                        id: 0,
                        kind: crate::CoopOtherKind::CleanFuel,
                        amount: 0,
                        activity: Some(business_id),
                        income: crate::CoopCreditIncome::Unresolved,
                        source: alloc::format!("Actual issuer source {i}"),
                    })
                    .collect();
            }
            _ => {}
        }
        credit_accept(&mut session, setup);
        if case == 4 {
            // The accepted classification creates keyed children for their own answers.
            let mut children = open_topic(&mut session, TaxSection::SelfEmployment);
            let P::SaveIncomes { followups, .. } = &mut children else { unreachable!() };
            for row in followups.iter_mut().filter(|row| {
                row.key.starts_with(&alloc::format!("income/{target_id}/credit_other/"))
            }) {
                let crate::DocumentAnswer::CooperativeOtherRecords { activity, .. } =
                    &mut row.answer
                else {
                    unreachable!()
                };
                *activity = Some(business_id);
            }
            credit_accept(&mut session, children);
        }
        let current = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &current else { unreachable!() };
        assert_eq!(documents.other.farming.documents.len(), positions.len());
        let corrected = open_topic(&mut session, TaxSection::SelfEmployment);
        let mut bad = corrected.clone();
        let mut original = session.draft.clone();
        if case == 5 {
            let mut changed = current.clone();
            let P::SaveDocuments { documents, .. } = &mut changed else { unreachable!() };
            documents.other.farming.documents[target].document.qualified_payments = 4_001;
            accepted(&send(&mut session, changed));
            bad = open_topic(&mut session, TaxSection::SelfEmployment);
        }
        let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
        let index = if case == 4 {
            followups
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.key.starts_with(&alloc::format!("income/{target_id}/credit_other/"))
                })
                .nth(1)
                .unwrap()
                .0
        } else {
            let field = if case == 3 { "credit_records" } else { "allocations" };
            followups
                .iter()
                .position(|row| row.key == alloc::format!("income/{target_id}/{field}"))
                .unwrap()
        };
        let (key, local) = match &mut followups[index].answer {
            crate::DocumentAnswer::CooperativeAllocations { value } => match case {
                0 => {
                    value[0].patron_allocation = Some(crate::CoopPatronAllocation::NoWages);
                    ("COOP_PATRON_SOURCE_DUPLICATE", "value/0/patron_allocation")
                }
                2 => {
                    value[0].properties[0].property = Some(999_999);
                    ("COOP_PROPERTY_LINK_UNAVAILABLE", "value/0/properties/0/property")
                }
                _ => {
                    value.clear();
                    // The automatic allocation exposes the invalid known payment.
                    ("COOP_ALLOCATION_INVALID", "value/0/qualified_payments")
                }
            },
            crate::DocumentAnswer::CooperativeCreditRecords { activity, .. } => {
                *activity = Some(999_999);
                ("COOP_RECIPIENT_LINK_INVALID", "activity")
            }
            crate::DocumentAnswer::CooperativeOtherRecords { activity, .. } => {
                *activity = Some(999_999);
                ("COOP_RECIPIENT_LINK_INVALID", "activity")
            }
            _ => unreachable!(),
        };
        let name = alloc::format!("credit-live-coop-graph-{case}");
        let path = local
            .split('/')
            .filter(|segment| !segment.is_empty())
            .fold(
                SchemaPath::root()
                    .field("followups")
                    .key(followups[index].key.as_str())
                    .field("answer"),
                |path, segment| match segment.parse::<usize>() {
                    Ok(index) => path.index(index),
                    Err(_) => path.field(segment),
                },
            )
            .to_string();
        if case == 5 {
            credit_refusal(&name, &mut session, bad, key, &path);
            // The printed correction above was accepted after these pages were
            // opened; they are resubmitted at the current revision.
            let mut current = current;
            let P::SaveDocuments { revision, .. } = &mut current else { unreachable!() };
            *revision = session.draft.revision;
            credit_accept(&mut session, current);
            let mut corrected = corrected;
            let P::SaveIncomes { revision, .. } = &mut corrected else { unreachable!() };
            *revision = session.draft.revision;
            credit_accept(&mut session, corrected);
            original.revision = session.draft.revision;
            assert_eq!(
                session.draft, original,
                "{name}: correction did not preserve accepted sources"
            );
            credit_resave(&name, &mut session, TaxSection::SelfEmployment, false);
        } else {
            credit_contextual_document_case(
                &name,
                &mut session,
                TaxSection::SelfEmployment,
                corrected,
                bad,
                index,
                (key, local),
            );
        }
    }
}

#[test]
fn unit47_credit_estate_contextual_activity_save_preview_keep_real_owner_and_atomic_state() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let pages = (0..2)
        .map(|i| {
            let mut page = k1_1041(&alloc::format!("Actual estate participation source {i}"));
            page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
                box6: 0,
                box7: 100,
                box8: 0,
                activities: vec![estate_activity_page(
                    "Actual rental participation",
                    crate::EstateActivityKind::RentalRealEstate,
                    100,
                )],
            });
            document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(page)) },
            )
        })
        .collect();
    let request = credit_save_rows(&mut session, TaxSection::PassThrough, pages);
    credit_accept(&mut session, request);
    let corrected = open_documents(&mut session);
    let mut bad = corrected.clone();
    {
        let mut page = edit_estate_document(credit_estate_source_mut(&mut bad));
        let K1EstateTrustBusiness::Amounts(value) = &mut page.business else { unreachable!() };
        value.activities[0].participation = crate::EstateParticipation::Professional(
            Participation::Over500Hours { hours: 501 }.record(),
        );
    }
    credit_contextual_document_case(
        "credit-live-estate-professional-graph",
        &mut session,
        TaxSection::PassThrough,
        corrected,
        bad,
        1,
        ("RENTAL_PROFESSIONAL_NEEDS_STATUS", "business/activities/0/participation"),
    );
}

#[test]
fn unit47_credit_estate_contextual_mortgage_save_preview_keep_actual_loan_and_second_source() {
    let mut session = at_the_deductions_page(FederalStatus::Single).0;
    let mortgage = first_lender_mortgage(100, 100_000, AverageBalance::WithinLimit);
    let more = mortgage.more.clone();
    let request = save_mortgages(
        &mut session,
        crate::MortgageDocuments { documents: vec![mortgage_document(mortgage)] },
    );
    // The itemized walk has nothing left, so its hub was passed (G-45).
    let reply = send_past_less_common(&mut session, request);
    assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
    accepted(&crate::tests::cross::interview::save_mortgage_answers(
        &mut session,
        &[(more, crate::MortgagePointTreatment::None)],
    ));
    let loan = session
        .draft
        .deductions
        .as_ref()
        .unwrap()
        .mortgage_loans()
        .expect("accepted mortgage source converts")[0]
        .id;
    assert!(loan > 0, "actual mortgage Save assigned its source ID");
    let pages = (0..2)
        .map(|i| {
            let mut page = k1_1041(&alloc::format!("Actual final estate mortgage source {i}"));
            page.more.final_year = estate_final_amounts(0);
            let finals = estate_final_source(0, false);
            if i == 0 {
                page.final_mortgages = vec![loan];
            }
            page.successor = Some(true);
            page.final_character = Some(finals);
            document(
                0,
                IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(estate_document(page)) },
            )
        })
        .collect();
    let request = credit_save_rows(&mut session, TaxSection::PassThrough, pages);
    credit_accept(&mut session, request);
    for (case, bad_loan, key) in [
        ("missing", 999_999, "ESTATE_MORTGAGE_LINK_INVALID"),
        ("repeated", loan, "ESTATE_MORTGAGE_LINK_TWICE"),
    ] {
        let corrected = open_documents(&mut session);
        let mut bad = corrected.clone();
        edit_estate_document(credit_estate_source_mut(&mut bad)).final_mortgages = vec![bad_loan];
        credit_contextual_document_case(
            &alloc::format!("credit-live-estate-mortgage-graph-{case}"),
            &mut session,
            TaxSection::PassThrough,
            corrected,
            bad,
            1,
            (key, "final_mortgages/0"),
        );
    }
}

pub(super) use crate::tests::support::client::estimate_documents_from_save as credit_estimate_documents_from_save;

fn credit_w2_source_case(
    case: &str,
    bad: crate::W2Source,
    corrected: crate::W2Source,
    key: &str,
    local: &str,
) {
    // Railroad payroll and its incompatible Social Security boxes are checked
    // together on the printed W-2 entry.
    let local = if key == "W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES" { "payroll" } else { local };
    let answers = local.starts_with("answers/").then(|| {
        (
            vec![crate::DocumentAnswer::Wage { answers: (&bad.answers).into() }],
            vec![crate::DocumentAnswer::Wage { answers: (&corrected.answers).into() }],
            local,
        )
    });
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    credit_local_source_case_in_session(
        case,
        &mut session,
        document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(10_000))) },
        ),
        (
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(bad.form)) },
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(corrected.form)) },
        ),
        key,
        local,
        answers,
    );
}

#[test]
fn unit47_credit_w2_native_guard_order_and_real_box12_rows_locate_save_estimate_preview() {
    // Cases 6 to 9 were the paper-entry railroad check: railroad payroll is now the
    // coverage answer, asked and counted only while boxes 3 and 5 are blank.
    for case in (0..27).filter(|case| !matches!(*case, 6..=9 | 13 | 17)) {
        let mut corrected = crate::W2Source::printed(wage_document(w2(1_000_000)));
        if matches!(case, 6..=9) {
            corrected.form.social_wages = Some(0);
            corrected.form.social_tax = Some(0);
            corrected.form.medicare_wages = Some(0);
            corrected.form.medicare_tax = Some(0);
            corrected.answers.coverage = crate::tests::support::builders::native_income::railroad_answer(&W2Payroll::Railroad {
                compensation: corrected.form.wages.unwrap_or(0),
                tier_1: 62_000,
                tier_2: 0,
                medicare: 14_500,
                additional_medicare: 0,
            })
                ;
        }
        if matches!(case, 0 | 22) {
            corrected.form.statutory = true;
        }
        if matches!(case, 4 | 5) {
            corrected = crate::W2Source::printed(wage_document(w2(
                crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS,
            )));
            if case == 4 {
                corrected.form.social_wages = Some(corrected.form.social_wages.unwrap_or(0) - 1);
                corrected.form.social_tips = Some(1);
            }
        }
        if matches!(case, 10..=12 | 16 | 23 | 24 | 26) {
            corrected.form.box12 =
                vec![crate::PrintedBox12Entry { code: W2Code::L, amount: Some(100) }];
        }
        match case {
            10 | 24 => corrected.form.box12.push(crate::PrintedBox12Entry {
                code: W2Code::TA,
                amount: Some(crate::year_constants::TRUMP_EMPLOYER_EXCLUSION_CENTS),
            }),
            11 | 26 => corrected
                .form
                .box12
                .push(crate::PrintedBox12Entry { code: W2Code::D, amount: Some(100) }),
            12 | 23 => {
                corrected
                    .form
                    .box12
                    .push(crate::PrintedBox12Entry { code: W2Code::TP, amount: Some(100) });
                corrected.form.social_tips = Some(100);
                corrected.form.social_wages = Some(corrected.form.social_wages.unwrap_or(0) - 100);
                corrected.answers.tips_listed_occupation =
                    Some(WageDeductionEligibility::Ineligible);
                if case == 23 {
                    corrected
                        .form
                        .box12
                        .push(crate::PrintedBox12Entry { code: W2Code::TT, amount: Some(100) });
                    corrected.answers.overtime_flsa_required =
                        Some(WageDeductionEligibility::Ineligible);
                }
            }
            14 => {
                corrected.form.allocated_tips = Some(100);
                corrected.answers.tips_listed_occupation =
                    Some(WageDeductionEligibility::Ineligible);
            }
            15 => {
                corrected.answers.tip_records = Some(DailyTipRecord::Months {
                    months: vec![CashTipMonth { month: 3, received: 100, reported: 0 }],
                });
                corrected.answers.tips_listed_occupation =
                    Some(WageDeductionEligibility::Ineligible);
            }
            16 => {
                corrected
                    .form
                    .box12
                    .push(crate::PrintedBox12Entry { code: W2Code::TT, amount: Some(100) });
                corrected.answers.overtime_flsa_required =
                    Some(WageDeductionEligibility::Ineligible);
            }
            19 => {
                corrected.answers.deferred_pension = W2Amount::of(corrected.form.wages.unwrap_or(0))
            }
            20 => {
                corrected.answers.deferred_pension =
                    W2Amount::of(corrected.form.wages.unwrap_or(0) - 100);
                corrected.answers.inmate_pay = 100;
            }
            21 => {
                corrected.answers.deferred_pension =
                    W2Amount::of(corrected.form.wages.unwrap_or(0) - 200);
                corrected.answers.inmate_pay = 100;
                corrected.answers.medicaid_waiver = 100;
            }
            _ => {}
        }
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                bad.answers.abroad = W2Amount::of(1);
                ("W2_STATUTORY_ABROAD_USE_BUSINESS", "answers/abroad")
            }
            1 => {
                bad.form.withholding = Some(bad.form.wages.unwrap_or(0) + 1);
                ("W2_WITHHOLDING_ABOVE_WAGES", "withholding")
            }
            2 => {
                bad.form.social_tax = Some(
                    bad.form.social_wages.unwrap_or(0) + bad.form.social_tips.unwrap_or(0) + 1,
                );
                ("W2_SOCIAL_TAX_ABOVE_WAGES", "social_tax")
            }
            3 => {
                bad.form.medicare_tax = Some(bad.form.medicare_wages.unwrap_or(0) + 1);
                ("W2_MEDICARE_TAX_ABOVE_WAGES", "medicare_tax")
            }
            4 => {
                bad.form.social_wages = Some(bad.form.social_wages.unwrap_or(0) + 1);
                ("W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE", "social_tips")
            }
            5 => {
                bad.form.social_wages = Some(bad.form.social_wages.unwrap_or(0) + 1);
                ("W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE", "social_wages")
            }
            6 => {
                bad.form.social_wages = Some(1);
                ("W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES", "social_wages")
            }
            7 => {
                bad.form.social_tax = Some(1);
                bad.form.social_tips = Some(1);
                ("W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES", "social_tax")
            }
            8 => {
                bad.form.medicare_wages = Some(1);
                ("W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES", "medicare_wages")
            }
            9 => {
                bad.form.social_tips = Some(1);
                ("W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES", "social_tips")
            }
            10 => {
                bad.form.box12[1].amount = Some(bad.form.box12[1].amount.unwrap_or(0) + 1);
                ("W2_TA_ABOVE_LIMIT", "box12/1/amount")
            }
            11 => {
                bad.form.box12[1].code = W2Code::L;
                ("W2_BOX12_CODE_TWICE", "box12/1/code")
            }
            12 | 14 | 15 => {
                bad.answers.tips_listed_occupation = None;
                ("W2_TP_OCCUPATION_NEEDED", "answers/tips_listed_occupation")
            }
            16 => {
                bad.answers.overtime_flsa_required = None;
                ("W2_TT_FLSA_NEEDED", "answers/overtime_flsa_required")
            }
            18 => {
                bad.answers.abroad = W2Amount::of(bad.form.wages.unwrap_or(0) + 1);
                ("W2_ABROAD_ABOVE_WAGES", "answers/abroad")
            }
            19 => {
                bad.answers.deferred_pension =
                    W2Amount::of(W2Amount::cents(bad.answers.deferred_pension) + 1);
                ("W2_SCHEDULE_1_PAY_ABOVE_WAGES", "answers/deferred_pension")
            }
            20 => {
                bad.answers.inmate_pay += 1;
                ("W2_SCHEDULE_1_PAY_ABOVE_WAGES", "answers/inmate_pay")
            }
            21 => {
                bad.answers.medicaid_waiver += 1;
                ("W2_SCHEDULE_1_PAY_ABOVE_WAGES", "answers/medicaid_waiver")
            }
            22 => {
                bad.answers.abroad = W2Amount::of(1);
                bad.form.withholding = Some(bad.form.wages.unwrap_or(0) + 1);
                // The printed batch is validated before its personal topic opens.
                ("W2_WITHHOLDING_ABOVE_WAGES", "withholding")
            }
            23 => {
                bad.answers.tips_listed_occupation = None;
                bad.answers.overtime_flsa_required = None;
                ("W2_TP_OCCUPATION_NEEDED", "answers/tips_listed_occupation")
            }
            24 => {
                bad.form.box12[0].code = W2Code::TA;
                bad.form.box12[1].amount = Some(bad.form.box12[1].amount.unwrap_or(0) + 1);
                ("W2_TA_ABOVE_LIMIT", "box12/1/amount")
            }
            25 => {
                bad.form.withholding = Some(bad.form.wages.unwrap_or(0) + 1);
                bad.form.social_tax = Some(
                    bad.form.social_wages.unwrap_or(0) + bad.form.social_tips.unwrap_or(0) + 1,
                );
                ("W2_WITHHOLDING_ABOVE_WAGES", "withholding")
            }
            _ => {
                bad.form.box12[1].code = W2Code::L;
                bad.answers.abroad = W2Amount::of(bad.form.wages.unwrap_or(0) + 1);
                ("W2_BOX12_CODE_TWICE", "box12/1/code")
            }
        };
        if matches!(case, 12 | 16 | 23) {
            let mut eligible = corrected.clone();
            if matches!(case, 12 | 23) {
                eligible.answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
            }
            if matches!(case, 16 | 23) {
                eligible.answers.overtime_flsa_required = Some(WageDeductionEligibility::Eligible);
            }
            credit_w2_source_case(
                &alloc::format!("credit-w2-native-{case}-eligible"),
                bad.clone(),
                eligible,
                key,
                local,
            );
        }
        credit_w2_source_case(
            &alloc::format!("credit-w2-native-{case}"),
            bad,
            corrected,
            key,
            local,
        );
    }
}

// These controls submit actual income requests. The earlier independent source
// makes the submitted document index observable, while nested cases retain their
// own real statement/asset indices through transparent Union payloads.
fn credit_local_source_case_in_session(
    case: &str,
    session: &mut Interview,
    earlier: IncomeDocumentItem,
    (bad, corrected): (IncomeSource, IncomeSource),
    key: &str,
    local: &str,
    answers: Option<(Vec<crate::DocumentAnswer>, Vec<crate::DocumentAnswer>, &str)>,
) {
    let topic = topic_for(&corrected);
    let source = !matches!(&corrected, IncomeSource::Record { .. });
    let same_owner =
        core::mem::discriminant(&earlier.document) == core::mem::discriminant(&corrected);
    let mut earlier = Some(earlier);
    if !same_owner {
        let row = earlier.take().unwrap();
        let prerequisite = credit_save_rows(session, topic_for(&row.document), vec![row]);
        credit_accept(session, prerequisite);
    }
    let answer_topic = if matches!(
        corrected,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) }
    ) {
        TaxSection::Education
    } else {
        topic
    };
    let same_paper = earlier.as_ref().is_some_and(|row| row.document == corrected);
    let rows = |target| {
        let mut rows: Vec<_> = earlier.clone().into_iter().collect();
        rows.push(document(0, target));
        rows
    };
    if let Some((bad_answers, corrected_answers, answer_path)) = answers {
        let request = credit_save_rows(session, topic, rows(bad));
        accepted(&send(session, request));
        let source_id = session.draft.income.last().unwrap().id;
        let answer_request = |session: &mut Interview,
                              source_id: i64,
                              values: &[crate::DocumentAnswer]| {
            let mut request = open_topic(session, answer_topic);
            if let P::SaveEducation { followups, .. } = &mut request {
                use crate::document_center::EducationDocumentAnswer as A;
                let typed = values
                    .iter()
                    .cloned()
                    .map(|value| A::from_canonical(value).unwrap())
                    .collect::<Vec<_>>();
                for value in &typed {
                    assert!(
                        followups.iter().any(|row| {
                            row.key.starts_with(&alloc::format!("income/{source_id}/"))
                                && core::mem::discriminant(&row.answer)
                                    == core::mem::discriminant(value)
                        }),
                        "the accepted source calls for its actual topic answer"
                    );
                }
                for row in followups {
                    let answers = if row.key.starts_with(&alloc::format!("income/{source_id}/")) {
                        values
                    } else if same_paper {
                        &corrected_answers
                    } else {
                        continue;
                    };
                    if let Some(value) =
                        answers.iter().cloned().filter_map(A::from_canonical).find(|value| {
                            core::mem::discriminant(&row.answer) == core::mem::discriminant(value)
                        })
                    {
                        row.answer = value;
                    }
                }
                return request;
            }
            if let P::SaveWagesAnswers { wages, .. } = &mut request {
                for value in values {
                    assert!(
                        matches!(value, crate::DocumentAnswer::Wage { .. })
                            && wages.iter().any(|row| {
                                row.key == alloc::format!("income/{source_id}/wage")
                            }),
                        "the accepted source calls for its actual topic answer"
                    );
                }
                for row in wages {
                    let values = if row.key == alloc::format!("income/{source_id}/wage") {
                        values
                    } else if same_paper {
                        &corrected_answers
                    } else {
                        continue;
                    };
                    if let Some(crate::DocumentAnswer::Wage { answers }) = values.first() {
                        let calls = row.answers.calls.clone();
                        row.answers = answers.clone();
                        row.answers.calls = calls;
                    }
                }
                return request;
            }
            let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
            for value in values {
                assert!(
                    followups.iter().any(|row| {
                        row.key.starts_with(&alloc::format!("income/{source_id}/"))
                            && core::mem::discriminant(&row.answer)
                                == core::mem::discriminant(value)
                    }),
                    "the accepted source calls for its actual topic answer"
                );
            }
            for row in followups {
                let answers = if row.key.starts_with(&alloc::format!("income/{source_id}/")) {
                    values
                } else if same_paper {
                    &corrected_answers
                } else {
                    continue;
                };
                let Some(value) = answers.iter().find(|value| {
                    core::mem::discriminant(&row.answer) == core::mem::discriminant(*value)
                }) else {
                    continue;
                };
                let mut value = value.clone();
                match (&row.answer, &mut value) {
                    (
                        crate::DocumentAnswer::CooperativeAllocations { value: accepted },
                        crate::DocumentAnswer::CooperativeAllocations { value: supplied },
                    ) => {
                        for (supplied, accepted) in supplied.iter_mut().zip(accepted) {
                            if supplied.id == 0 {
                                supplied.id = accepted.id;
                            }
                        }
                    }
                    (
                        crate::DocumentAnswer::CooperativeCreditRecords { other: accepted, .. },
                        crate::DocumentAnswer::CooperativeCreditRecords { other: supplied, .. },
                    ) => {
                        for (supplied, accepted) in supplied.iter_mut().zip(accepted) {
                            if supplied.id == 0 {
                                supplied.id = accepted.id;
                            }
                        }
                    }
                    _ => {}
                }
                row.answer = value;
            }
            request
        };
        let request = answer_request(session, source_id, &bad_answers);
        let index = match &request {
            P::SaveWagesAnswers { wages, .. } => {
                wages.iter().position(|row| row.key == alloc::format!("income/{source_id}/wage"))
            }
            P::SaveEducation { followups, .. } => {
                let answer = crate::document_center::EducationDocumentAnswer::from_canonical(
                    bad_answers[0].clone(),
                )
                .unwrap();
                followups.iter().position(|row| {
                    row.key.starts_with(&alloc::format!("income/{source_id}/"))
                        && core::mem::discriminant(&row.answer) == core::mem::discriminant(&answer)
                })
            }
            P::SaveIncomes { followups, .. } => followups.iter().position(|row| {
                row.key.starts_with(&alloc::format!("income/{source_id}/"))
                    && core::mem::discriminant(&row.answer)
                        == core::mem::discriminant(&bad_answers[0])
            }),
            _ => unreachable!(),
        }
        .unwrap();
        let question_key = match &request {
            P::SaveWagesAnswers { wages, .. } => wages[index].key.as_str(),
            P::SaveEducation { followups, .. } => followups[index].key.as_str(),
            P::SaveIncomes { followups, .. } => followups[index].key.as_str(),
            _ => unreachable!(),
        };
        let answer_root = if matches!(request, P::SaveWagesAnswers { .. }) {
            SchemaPath::root().field("wages").key(question_key)
        } else {
            SchemaPath::root().field("followups").key(question_key).field("answer")
        };
        let path = answer_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .fold(answer_root, |path, segment| match segment.parse::<usize>() {
                Ok(index) => path.index(index),
                Err(_) => path.field(segment),
            })
            .to_string();
        let estimate = credit_estimate_documents_from_save(&request);
        let provisional =
            crate::tests::support::client::survey::unanswered_personal_refusal(&request, key);
        credit_refusal(case, session, request, key, &path);
        if provisional {
            // An unanswered personal election permits a provisional estimate.
            let before = session.draft.clone();
            assert!(
                if answer_topic == TaxSection::Wages {
                    matches!(send(session, estimate), Reply::Response(P::RunningEstimate { .. }))
                } else {
                    matches!(
                        send(session, estimate),
                        Reply::Response(P::IncomeRunningEstimate { .. })
                    )
                },
                "{case}-estimate: provisional estimate"
            );
            assert_eq!(session.draft, before, "{case}-estimate: estimate changed accepted source");
        } else {
            credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, &path);
        }
        // Correct the accepted papers in place; new-row IDs would replace the
        // sources whose topic answers and error positions are under test.
        let mut corrected_rows = rows(corrected);
        let accepted_sources =
            &session.draft.income[session.draft.income.len() - corrected_rows.len()..];
        for (row, source) in corrected_rows.iter_mut().zip(accepted_sources) {
            row.id = source.id;
        }
        let corrected_request = credit_save_rows(session, topic, corrected_rows);
        accepted(&send(session, corrected_request));
        let source_id = session.draft.income.last().unwrap().id;
        let corrected_request = answer_request(session, source_id, &corrected_answers);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            session,
            credit_estimate_documents_from_save(&corrected_request),
        );
        assert!(
            if matches!(answer_topic, TaxSection::Education | TaxSection::Wages) {
                matches!(reply, Reply::Response(P::RunningEstimate { .. }))
            } else {
                matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. } | P::Review { .. }))
            },
            "{case}: corrected Estimate: {reply:?}"
        );
        assert_eq!(session.draft, before, "{case}: corrected Estimate changed accepted source");
        assert_eq!(session.dirty, dirty);
        credit_accept(session, corrected_request);
        credit_resave(case, session, answer_topic, false);
        return;
    }
    let path = credit_document_path(&corrected, usize::from(same_owner), local);
    let request = credit_save_rows(session, topic, rows(bad.clone()));
    let estimate = credit_estimate_documents_from_save(&request);
    credit_refusal(case, session, request, key, &path);
    credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, &path);
    if !source {
        let preview_path = if local.is_empty() {
            "[document]".into()
        } else {
            alloc::format!("[document/{local}]")
        };
        credit_refusal(
            &alloc::format!("{case}-preview"),
            session,
            credit_preview(bad),
            key,
            &preview_path,
        );
    }
    let corrected_request = credit_save_rows(session, topic, rows(corrected.clone()));
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(
        matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                session,
                credit_estimate_documents_from_save(&corrected_request)
            ),
            Reply::Response(P::IncomeRunningEstimate { .. } | P::Review { .. })
        ),
        "{case}: corrected Estimate"
    );
    assert_eq!(session.draft, before, "{case}: corrected Estimate changed accepted source");
    assert_eq!(session.dirty, dirty);
    if !source {
        assert!(
            matches!(
                send(session, credit_preview(corrected)),
                Reply::Response(P::IncomePreview { .. })
            ),
            "{case}: corrected Preview"
        );
        assert_eq!(session.draft, before, "{case}: corrected Preview changed accepted source");
        assert_eq!(session.dirty, dirty);
    }
    credit_accept(session, corrected_request);
    credit_resave(case, session, topic, source);
}

fn credit_local_source_case(
    case: &str,
    bad: IncomeSource,
    corrected: IncomeSource,
    key: &str,
    local: &str,
) {
    let earlier = match topic_for(&corrected) {
        TaxSection::InterestDividends => IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(election_interest(
                100,
            ))),
        },
        TaxSection::GovernmentBenefits => IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(),
            value: government_document(Form1099G { amount: 100, ..Default::default() }),
        },
        TaxSection::Retirement => IncomeSource::Retirement {
            value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(100, 0))),
        },
        TaxSection::Gambling => w2g(100, 0),
        TaxSection::CanceledDebt => IncomeSource::Record {
            value: crate::IncomeRecord::HealthLoanRepayment(crate::HealthLoanAssistance {
                program: crate::HealthLoanProgram::National,
                amount: 0,
                interest: 0,
                confirmed: false,
            }),
        },
        TaxSection::InvestmentSales => corrected.clone(),
        TaxSection::OtherIncome => IncomeSource::Record {
            value: crate::IncomeRecord::OtherIncome(crate::OtherIncomeItem {
                kind: crate::OtherIncomeKind::AlaskaDividend,
                amount: 100,
                withholding: 0,
            }),
        },
        other => panic!("{case}: unexpected source topic {other:?}"),
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    credit_local_source_case_in_session(
        case,
        &mut session,
        document(0, earlier),
        (bad, corrected),
        key,
        local,
        None,
    );
}

#[test]
fn unit47_credit_local_sale_guards_locate_actual_save_estimate_preview_source() {
    for case in 0..7 {
        let mut corrected = crate::SaleSource::printed(Sale1099B {
            payer: Default::default(),
            recipient: Default::default(),
            description: "".into(),
            digital_asset: false,
            term: Some(GainTerm::Short),
            proceeds: 1_000,
            basis: Some(1_100),
            reporting: BasisReporting::ReportedToIrs,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            collectible: false,
            market_discount: 0,
            several_sales: false,
            withholding: 0,
            state_details: Default::default(),
        });
        match case {
            1 => corrected.form.use_kind = SaleUse::Investment { wash_sale: 100 },
            2 => {
                corrected.form.proceeds = -100;
                corrected.form.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
            }
            3 => corrected.form.use_kind = SaleUse::WrittenOption { wash_sale: 0 },
            4 => corrected.adjustment = crate::SaleAdjustment::OptionPremium { amount: 100 },
            5 | 6 => corrected.form.market_discount = 100,
            _ => {}
        }
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                bad.form.term = None;
                ("SALE_TERM_NEEDED", "term")
            }
            1 => {
                bad.form.use_kind = SaleUse::Investment { wash_sale: 101 };
                ("SALE_WASH_SALE_ABOVE_LOSS", "use_kind/wash_sale")
            }
            2 => {
                bad.form.use_kind = SaleUse::Investment { wash_sale: 0 };
                ("SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY", "proceeds")
            }
            3 => {
                bad.form.term = Some(GainTerm::Long);
                ("SALE_WRITTEN_OPTION_SHORT_TERM", "term")
            }
            4 => {
                bad.form.use_kind = SaleUse::Personal;
                ("SALE_OPTION_PREMIUM_ON_STOCK", "adjustment")
            }
            5 => {
                bad.form.basis = None;
                ("SALE_MARKET_DISCOUNT_NEEDS_BASIS", "basis")
            }
            6 => {
                bad.form.use_kind = SaleUse::Personal;
                ("SALE_MARKET_DISCOUNT_ON_A_BOND", "market_discount")
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let answers = (case == 4).then(|| {
            (
                vec![crate::DocumentAnswer::SaleAdjustment { value: bad.adjustment.clone() }],
                vec![crate::DocumentAnswer::SaleAdjustment { value: corrected.adjustment.clone() }],
                "value",
            )
        });
        credit_local_source_case_in_session(
            &alloc::format!("local-sale-{case}"),
            &mut session,
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(corrected.form.clone()),
                },
            ),
            (
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(bad.form),
                },
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Sale1099B(corrected.form),
                },
            ),
            key,
            local,
            answers,
        );
    }
}

#[test]
fn unit47_credit_local_education_account_branches_locate_save_estimate_preview_source() {
    for case in 0..10 {
        let mut corrected = q1099_page(
            600_000,
            EducationAccount::Qtp { earnings: 200_000, basis: 400_000 },
            plain_withdrawal(),
            Q1099Beneficiary::Recipient,
        );
        if (3..=5).contains(&case) {
            corrected.form.qtp_to_roth_ira = true;
            corrected.roth_rollover = Some(qtp_roth_rollover().rollover);
        }
        if case >= 8 {
            corrected.form.distribution_from = crate::Form1099QDistributionFrom::CoverdellEsa;
            corrected.form.distribution_code = Some(if case == 8 {
                crate::Form1099QDistributionCode::ExcessThisYear
            } else {
                crate::Form1099QDistributionCode::ExcessLastYear
            });
        }
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                bad.form.gross_distribution += 1;
                ("Q1099_BOXES_DO_NOT_ADD_UP", "gross_distribution")
            }
            1 => {
                corrected.form.distribution_code =
                    Some(crate::Form1099QDistributionCode::Distribution);
                corrected.redeposited = corrected.form.gross_distribution;
                bad.form.distribution_code = Some(crate::Form1099QDistributionCode::Distribution);
                bad.redeposited = bad.form.gross_distribution + 1;
                ("Q1099_PART_ABOVE_GROSS", "redeposited")
            }
            2 => {
                corrected.form.trustee_to_trustee = true;
                corrected.taxable_transfer_part = corrected.form.gross_distribution;
                bad.form.trustee_to_trustee = true;
                bad.taxable_transfer_part = bad.form.gross_distribution + 1;
                ("Q1099_PART_ABOVE_GROSS", "taxable_transfer_part")
            }
            3 => {
                bad.form.distribution_from = crate::Form1099QDistributionFrom::CoverdellEsa;
                ("Q1099_ROTH_ROLLOVER_NOT_529", "distribution_from")
            }
            4 => {
                bad.form.recipient_not_beneficiary = true;
                bad.beneficiary = Some(crate::FederalEducationBeneficiary::OtherStudent {
                    expenses: 600_000,
                    assistance: 0,
                    credit_expenses: 0,
                });
                ("Q1099_ROTH_ROLLOVER_NOT_STUDENT", "beneficiary")
            }
            5 => {
                bad.form.earnings = -1;
                bad.form.basis = 600_001;
                ("Q1099_ROTH_ROLLOVER_LOSS", "earnings")
            }
            6 => {
                bad.form.distribution_code = Some(crate::Form1099QDistributionCode::ExcessThisYear);
                ("Q1099_CODE_NOT_COVERDELL", "distribution_code")
            }
            7 => {
                bad.form.distribution_code =
                    Some(crate::Form1099QDistributionCode::ProhibitedTransaction);
                ("Q1099_CODE_NOT_COVERDELL", "distribution_code")
            }
            8 | 9 => {
                bad.form.earnings = 0;
                bad.form.basis = 0;
                bad.form.fair_market_value =
                    Some(crate::Form1099QFairMarketValue { value: 600_000 });
                ("Q1099_EXCESS_NEEDS_BOXES", "fair_market_value")
            }
            _ => unreachable!(),
        };
        if matches!(case, 1 | 2 | 4) {
            use crate::DocumentAnswer as A;
            let (bad_answer, good_answers) = match case {
                1 => (
                    A::EducationRedeposited { value: bad.redeposited },
                    vec![A::EducationRedeposited { value: corrected.redeposited }],
                ),
                2 => (
                    A::EducationTaxableTransfer { value: bad.taxable_transfer_part },
                    vec![A::EducationTaxableTransfer { value: corrected.taxable_transfer_part }],
                ),
                4 => (
                    A::EducationBeneficiary { value: bad.beneficiary.as_ref().map(Into::into) },
                    Vec::new(),
                ),
                _ => unreachable!(),
            };
            let mut bad_answers = vec![bad_answer];
            let mut good_answers = good_answers;
            if case == 4 {
                bad_answers.push(A::EducationRothRecords { value: bad.roth_rollover.clone() });
                good_answers
                    .push(A::EducationRothRecords { value: corrected.roth_rollover.clone() });
            }
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            credit_local_source_case_in_session(
                &alloc::format!("local-q1099-{case}-topic"),
                &mut session,
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::OtherIncome(crate::OtherIncomeItem {
                            kind: crate::OtherIncomeKind::AlaskaDividend,
                            amount: 100,
                            withholding: 0,
                        }),
                    },
                ),
                (
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Q1099(bad.form),
                    },
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                            corrected,
                        )),
                    },
                ),
                key,
                local,
                Some((bad_answers, good_answers, "value")),
            );
            continue;
        }
        credit_local_source_case(
            &alloc::format!("local-q1099-{case}"),
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(bad.form) },
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    corrected,
                )),
            },
            key,
            local,
        );
    }
}

#[test]
fn unit47_credit_local_railroad_guards_locate_save_estimate_preview_source() {
    for case in 0..5 {
        let mut corrected = crate::RailroadSource {
            form: crate::Rrb1099R {
                issuer: Default::default(),
                recipient: Default::default(),
                claim_number_and_payee_code: String::new(),
                medicare_premiums: Some(0),
                box_3: None,
                box_4: Some(0),
                box_5: Some(1_000),
                box_6: Some(0),
                gross: 1_000,
                box_8: Some(0),
                withholding: Some(0),
            },
            recipient_kind: Some(crate::RailroadRecipient::Spouse),
            repayments: vec![],
        };
        if case >= 2 {
            corrected.form.box_8 = Some(100);
            corrected.repayments.push(crate::RailroadRepayment {
                received_year: 2025,
                amount: 100,
                contributory: false,
            });
        }
        if case == 1 {
            corrected.form.withholding = Some(corrected.form.gross);
        }
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                bad.form.gross += 1;
                ("RRB_BOX7_NOT_SUM", "gross")
            }
            1 => {
                bad.form.withholding = Some(bad.form.withholding.unwrap_or(0) + 1);
                ("R_WITHHOLDING_ABOVE_GROSS", "withholding")
            }
            2 => {
                bad.form.box_8 = Some(0);
                ("RRB_REPAYMENTS_WITHOUT_BOX_8", "repayments")
            }
            3 => {
                bad.repayments.clear();
                ("RRB_BOX_8_NOT_SPLIT", "repayments")
            }
            4 => {
                bad.repayments[0].amount += 1;
                ("RRB_REPAYMENTS_NOT_BOX_8", "repayments")
            }
            _ => unreachable!(),
        };
        if case >= 2 {
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            crate::tests::support::client::railroad_answer_case(
                &alloc::format!("local-rrb-{case}"),
                &mut session,
                None,
                bad,
                corrected,
                key,
            );
            continue;
        }
        credit_local_source_case(
            &alloc::format!("local-rrb-{case}"),
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_paper(bad.form)),
            },
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Rrb1099R(railroad_document(corrected)),
            },
            key,
            local,
        );
    }
}

#[test]
fn unit47_credit_local_refund_and_government_boxes_locate_save_estimate_preview_source() {
    for case in 0..2 {
        let mut corrected = Form1099G { amount: 1_000, withholding: 1_000, ..Default::default() };
        if case == 1 {
            corrected.taxable_grants = 100;
            // 2026-10-23: activity-owned grants need no document-side business
            // choice. Exercise the actual withholding bound including box 6.
            corrected.withholding += corrected.taxable_grants;
        }
        let _answers =
            crate::tests::support::client::income_pages::unemployment_answers(if case == 1 {
                alloc::vec![crate::DocumentAnswer::GovernmentGrantUse {
                    value: Some(crate::GrantUse::You),
                }]
            } else {
                Vec::new()
            });
        let mut bad = corrected.clone();
        bad.withholding += 1;
        // The 1099-G paper names box 4 as printed.
        let (key, local) = ("FORM_1099G_WITHHOLDING_ABOVE_BENEFITS", "federal_income_tax_withheld");
        credit_local_source_case(
            &alloc::format!("local-form1099g-{case}"),
            IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: government_document(bad) },
            IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: government_document(corrected) },
            key,
            local,
        );
    }
    reject_unavailable_related_components(
        "local-form1099g-cross-owner-grant",
        IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(),
            value: government_document(Form1099G { taxable_grants: 100, ..Default::default() }),
        },
        &[crate::related_forms::DocumentKind::Form1099GGrant],
        true,
    );
    for case in 0..4 {
        let corrected = Refund1099G {
            payer: Default::default(),
            recipient: Default::default(),
            amount: 100,
            year: 2025,
            withholding: 100,
            state_details: Default::default(),
        };
        let mut deducted = RefundDeduction::IncomeTax {
            income_paid: 100,
            salt_paid: 200,
            salt_deducted: 200,
            itemized: 1_500_000,
            standard: 1_220_000,
            sales_available: 0,
            property_refunded: 100,
        };
        if case == 2 {
            let RefundDeduction::IncomeTax { income_paid, property_refunded, .. } = &mut deducted
            else {
                unreachable!()
            };
            *income_paid = 200;
            *property_refunded = 0;
        }
        let mut bad = corrected.clone();
        let mut bad_deducted = deducted.clone();
        let (key, local) = if case == 0 {
            bad.withholding += 1;
            ("REFUND_WITHHOLDING_ABOVE_REFUND", "federal_income_tax_withheld")
        } else {
            let RefundDeduction::IncomeTax {
                salt_deducted, income_paid, property_refunded, ..
            } = &mut bad_deducted
            else {
                unreachable!()
            };
            match case {
                1 => {
                    *salt_deducted += 1;
                    ("REFUND_SALT_DEDUCTED_ABOVE_PAID", "deducted/salt_deducted")
                }
                2 => {
                    *income_paid += 1;
                    ("REFUND_INCOME_TAX_ABOVE_SALT", "deducted/income_paid")
                }
                3 => {
                    *property_refunded += 1;
                    ("REFUND_PROPERTY_ABOVE_OTHER_TAXES", "deducted/property_refunded")
                }
                _ => unreachable!(),
            }
        };
        if case != 0 {
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            let suffix = alloc::format!("value/{}", local.strip_prefix("deducted/").unwrap());
            credit_local_source_case_in_session(
                &alloc::format!("local-refund-{case}"),
                &mut session,
                document(
                    0,
                    IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(),
                        value: government_document(Form1099G { amount: 100, ..Default::default() }),
                    },
                ),
                (
                    IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: refund_paper(bad) },
                    IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: refund_paper(corrected) },
                ),
                key,
                local,
                Some((
                    vec![crate::DocumentAnswer::RefundDeduction { value: Some(bad_deducted) }],
                    vec![crate::DocumentAnswer::RefundDeduction { value: Some(deducted) }],
                    &suffix,
                )),
            );
            continue;
        }
        credit_local_source_case(
            &alloc::format!("local-refund-{case}"),
            IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: refund_document(bad, bad_deducted) },
            IncomeSource::GovernmentPayment { refund_state: None, refund_locality: Default::default(), value: refund_document(corrected, deducted) },
            key,
            local,
        );
    }
}

#[test]
fn unit47_credit_local_alimony_excluded_boundaries_locate_save_estimate_preview_source() {
    for case in 0..2 {
        let mut corrected = recapture_alimony();
        let crate::AlimonyRecapture::ThirdYear {
            first,
            second,
            first_excluded,
            second_excluded,
            ..
        } = &mut corrected.recapture
        else {
            unreachable!()
        };
        if case == 0 {
            *first_excluded = *first;
        } else {
            *second_excluded = *second;
        }
        let mut bad = corrected.clone();
        let crate::AlimonyRecapture::ThirdYear { first_excluded, second_excluded, .. } =
            &mut bad.recapture
        else {
            unreachable!()
        };
        let (key, local) = if case == 0 {
            *first_excluded += 1;
            ("ALIMONY_2024_EXCLUDED_ABOVE_PAID", "recapture/first_excluded")
        } else {
            *second_excluded += 1;
            ("ALIMONY_2025_EXCLUDED_ABOVE_PAID", "recapture/second_excluded")
        };
        credit_local_source_case(
            &alloc::format!("local-alimony-{case}"),
            IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(bad) },
            IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(corrected) },
            key,
            local,
        );
    }
}

#[test]
fn unit47_credit_local_equipment_uses_actual_second_asset_and_native_owner() {
    for case in 0..3 {
        let mut corrected = EquipmentRent {
            related: crate::RelatedForms::default(),
            receipts: 150_000,
            expenses: 0,
            basis: 300_000,
            depreciable_basis: 300_000,
            assets: vec![
                with_first_year(
                    asset(100_000, 20260410, AssetClass::Seven),
                    FirstYearDeduction::Regular,
                ),
                with_first_year(
                    asset(200_000, 20260410, AssetClass::Seven),
                    FirstYearDeduction::Regular,
                ),
            ],
            history: PersonalRentalHistory::NONE,
        };
        if case == 0 {
            corrected.assets[1].gds_mut().unwrap().class = AssetClass::Five;
        }
        let mut bad = corrected.clone();
        let (key, local) = if case == 0 {
            bad.assets[1].investment_credit = solar_asset(20250915).investment_credit;
            ("EQUIPMENT_RENT_NO_INVESTMENT_CREDIT", "assets/1/investment_credit")
        } else if case == 1 {
            bad.assets[1].gds_mut().unwrap().history = AssetHistory::New {
                first_year: FirstYearDeduction::Section179 {
                    description: "Rented equipment".into(),
                    elected: 1,
                },
            };
            ("EQUIPMENT_RENT_NO_SECTION_179", "assets/1/history/first_year")
        } else {
            // The canonical asset guard must keep priority over both forbidden rental elections.
            bad.assets[1].credit_reduction = bad.assets[1].cost + 1;
            bad.assets[1].investment_credit = solar_asset(20250915).investment_credit;
            bad.assets[1].gds_mut().unwrap().history = AssetHistory::New {
                first_year: FirstYearDeduction::Section179 {
                    description: "Rented equipment".into(),
                    elected: 1,
                },
            };
            ("ASSET_CREDIT_ABOVE_COST", "assets/1/credit_reduction")
        };
        credit_local_source_case(
            &alloc::format!("local-equipment-{case}"),
            IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(bad) },
            IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(corrected) },
            key,
            local,
        );
    }
}

#[test]
fn unit47_credit_local_direct_source_owners_and_record_fallback_locate_requests() {
    credit_local_source_case(
        "local-w2g",
        w2g(100, 101),
        w2g(100, 100),
        "W2G_WITHHOLDING_ABOVE_WINNINGS",
        "withholding",
    );
    let corrected = crate::OtherIncomeItem {
        kind: crate::OtherIncomeKind::AlaskaDividend,
        amount: 100,
        withholding: 100,
    };
    let mut bad = corrected.clone();
    bad.withholding += 1;
    credit_local_source_case(
        "local-other-withholding",
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(bad) },
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(corrected) },
        "OTHER_INCOME_WITHHOLDING_ABOVE_AMOUNT",
        "withholding",
    );
    let corrected = crate::HealthLoanAssistance {
        program: crate::HealthLoanProgram::National,
        amount: 100,
        interest: 0,
        confirmed: false,
    };
    let mut bad = corrected.clone();
    bad.interest = 101;
    credit_local_source_case(
        "local-health-loan",
        IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(bad) },
        IncomeSource::Record { value: crate::IncomeRecord::HealthLoanRepayment(corrected) },
        "HEALTH_LOAN_INTEREST_ABOVE_ASSISTANCE",
        "interest",
    );
    let corrected = crate::ForeignAccountOrTrust {
        account: false,
        trust: true,
        highest_value: Some(0),
        fbar_required: None,
        fbar_countries: vec![],
    };
    let mut bad = corrected.clone();
    bad.highest_value = None;
    credit_local_source_case(
        "local-foreign-value",
        IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(bad) },
        IncomeSource::Record { value: crate::IncomeRecord::ForeignAccountOrTrust(corrected) },
        "FOREIGN_ASSETS_VALUE_NEEDED",
        "highest_value",
    );
    let corrected = crate::SaSource {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        form: crate::MedicalSaPaper::Form1099Sa(crate::Form1099SaPaper {
corrected: false,
calendar_year: Default::default(),
            trustee_payer: Default::default(),
            recipient: Default::default(),
            account_number: String::new(),
            gross_distribution: 100,
            earnings_on_excess_contributions: None,
            distribution_code: Some(crate::Form1099SaDistributionCode::NormalDistributions),
            fmv_on_date_of_death: None,
            account: crate::MedicalSaAccount::Hsa,
        }),
        medicare_prior: if matches!(
            &crate::MedicalSaAccount::Hsa,
            MedicalSaAccount::MedicareAdvantage
        ) {
            Some(MedicareMsaPrior::None)
        } else {
            None
        },
        medicare_from_spouse: if matches!(
            &crate::MedicalSaAccount::Hsa,
            MedicalSaAccount::MedicareAdvantage
        ) {
            Some(false)
        } else {
            None
        },
        payment: Some(crate::MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 100,
            rollover: None,
            disabled_since: None,
        }),
    };
    let mut bad = corrected.clone();
    let Some(crate::MedicalSaPayment::Own { medical, .. }) = &mut bad.payment else {
        unreachable!()
    };
    *medical += 1;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let desired = vec![document(
        0,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Sa1099(
                crate::tests::support::client::income_pages::sa_document(bad),
            ),
        },
    )];
    let request = credit_personal_refusal_request(&mut session, &desired);
    let P::SaveHsa { followups, .. } = &request else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::HsaDocumentAnswer::MedicalPayment { .. }))
        .unwrap();
    let estimate = credit_estimate_documents_from_save(&request);
    let path = SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .field("medical")
        .to_string();
    credit_refusal("local-sa1099", &mut session, request, "SA1099_MEDICAL_ABOVE_GROSS", &path);
    credit_refusal(
        "local-sa1099-estimate",
        &mut session,
        estimate,
        "SA1099_MEDICAL_ABOVE_GROSS",
        &path,
    );
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    let corrected = vec![document(
        id,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Sa1099(
                crate::tests::support::client::income_pages::sa_document(corrected),
            ),
        },
    )];
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &corrected,
    );
    credit_resave("local-sa1099", &mut session, TaxSection::Hsa, false);
}

#[test]
fn unit47_credit_local_child_statements_preserve_actual_nested_union_index() {
    for case in 0..3 {
        let mut original = complete(FederalStatus::Single);
        let child = original.next_id;
        add_child(&mut original);
        let mut corrected = election_page(
            child,
            vec![ChildStatement::Int1099(election_interest(100).interest_source())],
        );
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                let good = election_interest(100);
                let mut invalid = good.clone();
                invalid.more.private_activity = 1;
                corrected.statements.push(ChildStatement::Int1099(good.interest_source()));
                bad.statements.push(ChildStatement::Int1099(invalid.interest_source()));
                (
                    "INT_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
                    "statements/1/specified_private_activity_bond_interest",
                )
            }
            1 => {
                let good = native_div1099("Child fund".into(), 100, 100);
                let mut invalid = good.clone();
                invalid.qualified += 1;
                corrected.statements.push(ChildStatement::Div1099(
                    crate::child_election::ChildDividendStatement {
                        id: 0,
                        document: dividend_document(good),
                    },
                ));
                bad.statements.push(ChildStatement::Div1099(
                    crate::child_election::ChildDividendStatement {
                        id: 0,
                        document: dividend_document(invalid),
                    },
                ));
                ("DIV_QUALIFIED_ABOVE_ORDINARY", "statements/1/document/qualified")
            }
            2 => {
                let good = native_oid1099("Child OID".into(), 100, 0);
                let mut invalid = good.clone();
                invalid.more.private_activity = invalid.more.exempt + 1;
                corrected.statements.push(ChildStatement::Oid1099(good.oid_source()));
                bad.statements.push(ChildStatement::Oid1099(invalid.oid_source()));
                ("OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT", "statements/1/private_activity")
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, original);
        credit_local_source_case_in_session(
            &alloc::format!("local-child-{case}"),
            &mut session,
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        election_interest(100),
                    )),
                },
            ),
            (
                IncomeSource::Record { value: crate::IncomeRecord::Form8814(bad) },
                IncomeSource::Record { value: crate::IncomeRecord::Form8814(corrected) },
            ),
            key,
            local,
            None,
        );
    }
}

#[test]
fn unit47_credit_local_savings_bond_bounds_keep_actual_accepted_interest_source() {
    for case in 0..4 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let interest = Int1099 {
            more: IntMore { us_obligations: 300_000, ..Default::default() },
            ..native_int1099("Treasury".into(), 0)
        };
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
                },
            )],
        );
        credit_accept(&mut session, request);
        let earlier = page_documents(&open_documents(&mut session))
            .iter()
            .find(|row| {
                matches!(
                    row.document,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Int1099(_)
                    }
                )
            })
            .unwrap()
            .clone();
        assert!(earlier.id > 0);
        let corrected = SavingsBonds {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            proceeds: 1_200_000,
            redemption_interest: 300_000,
            earliest_issue_date: 20100601,
            interest_statement: earlier.id,
            students: vec![SavingsBondStudent {
                person: -1,
                expenses: SavingsBondExpenses::Entered { tuition: 400_000, benefits: 0 },
                contributions: 0,
            }],
        };
        let mut bad = corrected.clone();
        let (key, local) = match case {
            0 => {
                bad.interest_statement = 0;
                ("SAVINGS_BONDS_SOURCE_INVALID", "interest_statement")
            }
            1 => {
                bad.redemption_interest = bad.proceeds + 1;
                ("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS", "redemption_interest")
            }
            2 => {
                bad.earliest_issue_date = 19891231;
                ("SAVINGS_BONDS_ISSUED_BEFORE_1990", "earliest_issue_date")
            }
            3 => {
                bad.earliest_issue_date = 20270101;
                ("SAVINGS_BONDS_ISSUED_AFTER_2026", "earliest_issue_date")
            }
            _ => unreachable!(),
        };
        let bad = savings_bonds_source(bad);
        let corrected = savings_bonds_source(corrected);
        // Form 8815 amounts and supporting issue dates are entered and
        // refused in the Document Center; the accepted-source link is personal.
        let (local, answers) = if case != 0 {
            (if case == 1 { "form/line6" } else { local }, None)
        } else {
            (
                local,
                Some((
                    vec![crate::DocumentAnswer::SavingsBondsRecords {
                        value: bad.answers.as_ref().map(Into::into),
                    }],
                    vec![crate::DocumentAnswer::SavingsBondsRecords {
                        value: corrected.answers.as_ref().map(Into::into),
                    }],
                    alloc::format!("value/{local}"),
                )),
            )
        };
        credit_local_source_case_in_session(
            &alloc::format!("local-savings-bonds-{case}"),
            &mut session,
            earlier,
            (
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                        bad,
                    )),
                },
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                        corrected,
                    )),
                },
            ),
            key,
            local,
            answers.as_ref().map(|(bad, good, path)| (bad.clone(), good.clone(), path.as_str())),
        );
    }
}

// Data answers originally validate in the shared Save/Estimate candidate,
// whereas Preview returns no standalone income value. These controls certify
// only those two reached routes; they do not claim a Preview guard was added.
fn credit_data_answer_case(
    case: &str,
    session: &mut Interview,
    topic: TaxSection,
    bad: Vec<IncomeDocumentItem>,
    corrected: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) {
    crate::tests::support::client::admit_native_fixture_facts(session);
    let (bad_records, sources): (Vec<_>, Vec<_>) =
        bad.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    let (corrected_records, corrected_sources): (Vec<_>, Vec<_>) =
        corrected.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    assert_eq!(sources, corrected_sources, "only record answers may differ");
    let path = if sources.is_empty() {
        path.replace("/document/value", "/document")
    } else {
        assert_eq!(sources.len(), 1);
        let prerequisite = save_documents(session, sources);
        credit_accept(session, prerequisite);
        path.replace("documents/1/", "documents/0/")
            .replace("documents/2/", "documents/1/")
            .replace("/document/value", "/document")
    };
    let request = credit_save_rows(session, topic, bad_records);
    let estimate = credit_estimate_documents_from_save(&request);
    credit_refusal(case, session, request, key, &path);
    credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, &path);
    let corrected_request = credit_save_rows(session, topic, corrected_records);
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(
        matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                session,
                credit_estimate_documents_from_save(&corrected_request)
            ),
            Reply::Response(P::IncomeRunningEstimate { .. })
        ),
        "{case}: corrected Estimate"
    );
    assert_eq!(session.draft, before, "{case}: corrected Estimate changed accepted source");
    assert_eq!(session.dirty, dirty);
    credit_accept(session, corrected_request);
    credit_resave(case, session, topic, false);
}

fn credit_data_answer_earlier_sale() -> IncomeDocumentItem {
    document(
        0,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(contracts_page("", -1_000)),
        },
    )
}

#[test]
fn unit47_credit_data_answer_installment_fields_preserve_order_and_atomic_requests() {
    for case in 0..3 {
        let mut corrected =
            earlier_installment(crate::PropertyCharacter::LongTermCapital, 5_000_000, 10_000_000)
                .page();
        if case != 1 {
            corrected.gross_profit = corrected.contract_price;
        }
        let mut bad = corrected.clone();
        let (key, leaf) = match case {
            0 => {
                bad.gross_profit += 1;
                ("INSTALLMENT_PROFIT_ABOVE_CONTRACT", "gross_profit")
            }
            1 => {
                bad.payments += 1;
                ("INSTALLMENT_PAID_ABOVE_CONTRACT", "payments")
            }
            2 => {
                bad.gross_profit += 1;
                bad.payments += 1;
                ("INSTALLMENT_PROFIT_ABOVE_CONTRACT", "gross_profit")
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        credit_data_answer_case(
            &alloc::format!("data-installment-{case}"),
            &mut session,
            TaxSection::InvestmentSales,
            vec![
                credit_data_answer_earlier_sale(),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::InstallmentPayments(bad) },
                ),
            ],
            vec![
                credit_data_answer_earlier_sale(),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::InstallmentPayments(corrected),
                    },
                ),
            ],
            key,
            &alloc::format!("[documents/documents/1/document/value/{leaf}]"),
        );
    }
}

#[test]
fn unit47_credit_data_answer_nonfarm_history_retains_primitive_year_indices() {
    for case in 4..6 {
        let mut corrected = se_history();
        if case >= 3 {
            corrected.prior_years = vec![1999, 2000, 2010, 2025];
        }
        let mut bad = corrected.clone();
        let leaf = match case {
            4 => {
                bad.prior_years[1] = 1999;
                "prior_years/1"
            }
            5 => {
                bad.prior_years[1] = 1950;
                "prior_years/1"
            }
            _ => unreachable!(),
        };
        let earlier = || {
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(business(50_000)) },
            )
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        credit_data_answer_case(
            &alloc::format!("data-nonfarm-{case}"),
            &mut session,
            TaxSection::SelfEmployment,
            vec![
                earlier(),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::NonfarmOptionalMethod(bad) },
                ),
            ],
            vec![
                earlier(),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::NonfarmOptionalMethod(corrected),
                    },
                ),
            ],
            "NONFARM_OPTIONAL_HISTORY_INVALID",
            &alloc::format!("[documents/documents/1/document/value/{leaf}]"),
        );
    }
}

#[test]
fn unit47_credit_data_answer_1256_keeps_duplicate_first_and_composite_owner_key() {
    for case in 0..3 {
        let corrected = crate::Section1256Carryback {
            amount: 100,
            injured_owners: (case == 1)
                .then_some(crate::InjuredRetainedOwners::Both { taxpayer: 60, spouse: 40 }),
        };
        let mut bad = corrected.clone();
        let (key, path) = match case {
            0 => {
                bad.amount = 0;
                (
                    "MESSAGE_SECTION_1256_ENTER_A_CARRYBACK_AMOUNT_WITHIN_THE_SUPPORTED_RANGE",
                    "[documents/documents/1/document/value/amount]",
                )
            }
            1 => {
                bad.injured_owners =
                    Some(crate::InjuredRetainedOwners::Both { taxpayer: 60, spouse: 41 });
                (
                    "INJURED_CARRYBACK_OWNER_TOTAL",
                    "[documents/documents/1/document/value/injured_owners]",
                )
            }
            2 => {
                bad.amount = 0;
                ("SECTION_1256_CARRYBACK_TWICE", "[documents/documents/2/document/value]")
            }
            _ => unreachable!(),
        };
        let mut rejected = vec![credit_data_answer_earlier_sale()];
        if case == 2 {
            rejected.push(document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Section1256Carryback(corrected.clone()),
                },
            ));
        }
        rejected.push(document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(bad) },
        ));
        let fixed = vec![
            credit_data_answer_earlier_sale(),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Section1256Carryback(corrected),
                },
            ),
        ];
        let mut original =
            complete(if case == 1 { FederalStatus::Joint } else { FederalStatus::Single });
        if case == 1 {
            // The source-owner question is reached only in the actual Form8379
            // context. Its admission preflight keeps priority on ordinary returns.
            let mut ordinary = Interview::new(TEST_USER, original.clone());
            let (records, sources): (Vec<_>, Vec<_>) = rejected
                .clone()
                .into_iter()
                .partition(|row| matches!(row.document, IncomeSource::Record { .. }));
            let source = save_documents(&mut ordinary, sources);
            credit_accept(&mut ordinary, source);
            let before = ordinary.draft.clone();
            let dirty = ordinary.dirty;
            let request = credit_save_rows(&mut ordinary, TaxSection::InvestmentSales, records);
            let Reply::Response(P::Problem { message, field }) = current_income_refusal(
                "income-source-4457",
                &mut ordinary,
                request,
                "INJURED_SOURCE_FACTS_NOT_REQUESTED",
                "[documents/documents/0/document/injured_owners]",
            ) else {
                panic!("data-1256-1: ordinary return must retain source-facts preflight");
            };
            assert_eq!(
                message,
                libertas::libertas_formatted_text("INJURED_SOURCE_FACTS_NOT_REQUESTED", &[])
            );
            assert_eq!(
                field.as_deref(),
                Some("[documents/documents/0/document/injured_owners]"),
                "existing admission preflight retains priority and names the submitted facts"
            );
            assert_eq!(ordinary.draft, before);
            assert_eq!(ordinary.dirty, dirty);
            original.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            original.injured_spouse = Some(crate::InjuredSpouseAllocation::defaults());
        }
        let mut session = Interview::new(TEST_USER, original);
        credit_data_answer_case(
            &alloc::format!("data-1256-{case}"),
            &mut session,
            TaxSection::InvestmentSales,
            rejected,
            fixed,
            key,
            path,
        );
    }
}

#[test]
fn unit47_credit_data_answer_capital_research_preserves_actual_source_and_submission_indices() {
    for case in [0, 3, 4, 5] {
        let (mut original, first_source) =
            crate::tests::credits::form_3800::retained_capital_draft();
        // A second actual accepted research-eligible K-1, not a guessed source ID.
        let mut second =
            original.income.iter().find(|entry| entry.id == first_source).unwrap().clone();
        second.id = 0;
        second.label = "Second actual research source".into();
        add(&mut original, second);
        let second_source = original.income.last().unwrap().id;
        assert_ne!(first_source, second_source);
        let mut record = original.screening.as_mut().unwrap().capital_history.remove(0);
        record.research_parts = Some(ResearchCapitalParts::Parts {
            sources: vec![
                ResearchCapitalPart { source: first_source, short: 500_000, long: 0 },
                ResearchCapitalPart { source: second_source, short: 500_000, long: 0 },
            ],
        });
        let corrected = PriorCapitalReturnPage::from(record);
        let mut bad = corrected.clone();
        let Some(ResearchCapitalPartsPage::Parts { sources }) = &mut bad.research_parts else {
            unreachable!()
        };
        let (key, local) = match case {
            0 => {
                sources[1].short = 0;
                ("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE", "research_parts/sources/1/long")
            }
            3 => {
                sources[1].short += 1;
                ("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE", "research_parts/sources")
            }
            4 => {
                sources[1].source = first_source;
                ("RETAINED_RESEARCH_SOURCE_TWICE", "research_parts/sources/1/source")
            }
            5 => {
                sources[1].source = original.next_id + 100;
                ("RETAINED_RESEARCH_SOURCE_UNAVAILABLE", "research_parts/sources/1/source")
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, original);
        credit_data_answer_case(
            &alloc::format!("data-capital-research-{case}"),
            &mut session,
            TaxSection::InvestmentSales,
            vec![
                credit_data_answer_earlier_sale(),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(bad) },
                ),
            ],
            vec![
                credit_data_answer_earlier_sale(),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(corrected),
                    },
                ),
            ],
            key,
            &alloc::format!("[documents/documents/1/document/value/{local}]"),
        );
    }
}

#[test]
fn bond_blank_annual_and_monthly_amounts_survive_save_and_restart() {
    use crate::tests::support::client::income_pages::open_document_center;
    let mut session = restart(complete(FederalStatus::Single));
    let mut bond = direct_tax_credit_bond(
        "SOURCE42",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        100,
    );
    bond.form.recipient = fixture_document_names(&session.draft, -1).unwrap().recipient;
    for amount in [None, Some(0)] {
        bond.form.box1_credit = amount;
        bond.form.january = amount;
        bond.form.february = amount;
        bond.form.march = amount;
        bond.form.april = amount;
        bond.form.may = amount;
        bond.form.june = amount;
        bond.form.july = amount;
        bond.form.august = amount;
        bond.form.september = amount;
        bond.form.october = amount;
        bond.form.november = amount;
        bond.form.december = amount;
        bond.form.unique_identifier = None;
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        bond.id = session
            .draft
            .credits
            .as_ref()
            .unwrap()
            .tax_credit_bonds
            .first()
            .map_or(0, |row| row.id);
        documents.other.credits.documents = vec![crate::CreditDocument::CreditBond(
            alloc::boxed::Box::new(crate::TaxCreditBondDocument::from(bond.clone())),
        )];
        accepted(&send(&mut session, request));
        session = restart(session.draft);
        let request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = request else { unreachable!() };
        let crate::CreditDocument::CreditBond(reopened) = &documents.other.credits.documents[0]
        else {
            unreachable!()
        };
        assert_eq!(reopened.form, bond.form);
        assert_eq!(Form1097Btc::from(&reopened.form).box1_credit, 0);
    }
}
