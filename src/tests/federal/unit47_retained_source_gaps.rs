use super::unit47_credit_source_validation::credit_estimate_documents_from_save;
use super::*;
use crate::tests::farm::cooperative::form_dollars;
use crate::tests::support::client::income_pages::{open_documents, save_documents};
use alloc::string::ToString;
use libertas::SchemaPath;

fn retained_calculation(request: &P) -> P {
    if let P::SaveGiftCarryovers {
        allowed_owners,
        retained_charity_k1s,
        retained_charity_origins,
        retained_research_choices,
        value,
        ..
    } = request
    {
        P::EstimateGiftCarryovers {
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            retained_charity_k1s: retained_charity_k1s.clone(),
            retained_charity_origins: retained_charity_origins.clone(),
            retained_research_choices: retained_research_choices.clone(),
            value: value.clone(),
        }
    } else {
        credit_estimate_documents_from_save(request)
    }
}

pub(super) fn retained_refuse(
    session: &mut Interview,
    request: P,
    case: &str,
    key: &str,
    path: &str,
) {
    crate::tests::support::client::survey::survey_refusal(case, session, request, key, path);
}

#[inline(never)]
pub(super) fn retained_case(
    session: Interview,
    bad: P,
    correction: P,
    topic: TaxSection,
    case: &str,
    key: &str,
    path: &str,
) {
    retained_case_on_candidate(session, None, bad, correction, topic, (case, key, path));
}

fn retained_case_on_candidate(
    mut session: Interview,
    mut candidate: Option<Interview>,
    bad: P,
    correction: P,
    topic: TaxSection,
    (case, key, path): (&str, &str, &str),
) {
    let source_editor = matches!(&correction, P::SaveDocuments { .. });
    let baseline = session.draft.clone();
    let result = supported(&baseline);
    let ids: Vec<_> = baseline.income.iter().map(|item| item.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    let estimate = retained_calculation(&bad);
    let unanswered_personal =
        crate::tests::support::client::survey::unanswered_personal_refusal(&bad, key);
    let target = candidate.as_mut().unwrap_or(&mut session);
    retained_refuse(target, bad, &alloc::format!("retained-gap-{case}-save"), key, path);
    if unanswered_personal {
        let before = target.draft.clone();
        let dirty = target.dirty;
        let Reply::Response(P::IncomeRunningEstimate { estimate: Some(estimate), .. }) =
            send(target, estimate)
        else {
            panic!("an unanswered personal record keeps a provisional topic estimate")
        };
        assert!(estimate.remaining_items.iter().any(|item| {
            item.section == topic
                && crate::Message::decode(&item.item)
                    .args
                    .contains(&crate::MessageArg::Resource(key.into()))
        }));
        assert_eq!(target.draft, before);
        assert_eq!(target.dirty, dirty);
    } else {
        retained_refuse(
            target,
            estimate,
            &alloc::format!("retained-gap-{case}-estimate"),
            key,
            path,
        );
    }
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        P::from_avro(&retained_calculation(&correction).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, baseline);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    assert_eq!(supported(&session.draft), result);
    let reopened = if source_editor {
        open_documents(&mut session)
    } else {
        retained_open(&mut session, topic)
    };
    accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(session.draft, baseline);
    let mut restored = restart(baseline.clone());
    let reopened = if source_editor {
        open_documents(&mut restored)
    } else {
        retained_open(&mut restored, topic)
    };
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, baseline);
    assert_eq!(restored.draft.income.iter().map(|item| item.id).collect::<Vec<_>>(), ids);
    assert_eq!(supported(&restored.draft), result);
}

fn retained_open(session: &mut Interview, topic: TaxSection) -> P {
    let Reply::Edit(request) = from_review(session, topic) else { panic!("editor required") };
    request
}

fn retained_k3_fixture() -> Interview {
    use crate::tests::foreign::foreign::{k3_answer, k3_blank, k3_row};
    let mut page = k1_1065("Synthetic retained international source");
    page.basis.start = k1_1065_continuing(5_000_000, ScorpRiskStart::Unknown);
    page.more.interest = 200_000;
    page.more.dividends = 1_000_000;
    page.more.foreign_taxes = 90_000;
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 6_000, 0, 900),
        k3_row("UK", K3Category::Passive, 1_000, 0, 0),
    ];
    k3.withholding = vec![
        k3_answer("JA", ForeignWithholding::AtLeast15),
        k3_answer("UK", ForeignWithholding::None),
    ];
    k3.us_gross = 500_000;
    page.schedule_k3 = K3Attached::Attached(k3);
    let (session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 72_000);
    assert_eq!(return_line(&result, "2b"), 200_000);
    assert_eq!(return_line(&result, "3b"), 1_000_000);
    session
}

#[test]
fn unit47_k3_withholding_missing_keys_preserve_actual_source_and_lifecycle() {
    for duplicate in [true, false] {
        let session = retained_k3_fixture();
        retained_k3_case(session, duplicate);
    }
}

#[inline(never)]
fn retained_k3_case(mut session: Interview, duplicate: bool) {
    let correction = crate::tests::support::client::income_pages::open_topic(
        &mut session,
        TaxSection::PassThrough,
    );
    let mut bad = correction.clone();
    let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
    let answer = followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::K1K3Withholding { .. }))
        .unwrap();
    let crate::DocumentAnswer::K1K3Withholding { value } = &mut answer.answer else {
        unreachable!()
    };
    assert_eq!(value.len(), 2);
    assert_ne!(value[0].country, value[1].country);
    let (case, key, local) = if duplicate {
        value[1].country = "JA".into();
        ("k3-country-twice", "K3_WITHHOLDING_COUNTRY_TWICE", "value/1/country")
    } else {
        value.pop();
        ("k3-passive-country-missing", "K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY", "value")
    };
    let path = local
        .split('/')
        .filter(|segment| !segment.is_empty())
        .fold(
            SchemaPath::root().field("followups").key(answer.key.as_str()).field("answer"),
            |path, segment| match segment.parse::<usize>() {
                Ok(index) => path.index(index),
                Err(_) => path.field(segment),
            },
        )
        .to_string();
    retained_case(session, bad, correction, TaxSection::PassThrough, case, key, &path);
}

fn retained_coop_fixture() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::Business(business(1_000_000)) },
        )],
    );
    accepted(&send(&mut session, request));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Farming(cooperative_document(
                crate::tests::farm::cooperative::patr_page(200_000, 50_000),
            )),
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
    let result = supported(&session.draft);
    assert_eq!(form_dollars(&result, TaxForm::ScheduleC, "6"), 1_500);
    assert_eq!(form_dollars(&result, TaxForm::ScheduleC, "31"), 11_500);
    session
}

#[test]
fn unit47_cooperative_all_eight_allocation_totals_have_actual_save_estimate_witnesses() {
    for field in [
        "patronage",
        "nonpatronage",
        "per_unit_retains",
        "redeemed",
        "qualified_payments",
        "qualified_items",
        "sstb_items",
        "section_199a_g",
    ] {
        let session = retained_coop_fixture();
        retained_coop_case(session, field);
    }
}

#[inline(never)]
fn retained_coop_case(mut session: Interview, field: &str) {
    let correction = open_documents(&mut session);
    let mut bad = correction.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    let activity_id = session
        .draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    let page = &mut documents.other.farming.documents[0].document;
    let crate::FederalIncome::Cooperative { data } = &session
        .draft
        .income
        .iter()
        .find(|row| matches!(row.income, FederalIncome::Cooperative { .. }))
        .unwrap()
        .income
    else {
        unreachable!()
    };
    let native = crate::Patr1099::from(data);
    assert_eq!(native.more.allocations.len(), 1);
    assert!(native.more.allocations[0].id > activity_id);
    *match field {
        "patronage" => &mut page.patronage,
        "nonpatronage" => &mut page.nonpatronage,
        "per_unit_retains" => &mut page.per_unit_retains,
        "redeemed" => &mut page.redeemed,
        "qualified_payments" => &mut page.qualified_payments,
        "qualified_items" => &mut page.qualified_items,
        "sstb_items" => &mut page.sstb_items,
        "section_199a_g" => &mut page.section_199a_g,
        _ => unreachable!(),
    } += 1;
    let mut candidate = restart(session.draft.clone());
    accepted(&send(&mut candidate, bad));
    let bad = retained_open(&mut candidate, TaxSection::SelfEmployment);
    let P::SaveIncomes { followups, .. } = &bad else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::DocumentAnswer::CooperativeAllocations { .. }))
        .unwrap();
    let path = SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .to_string();
    retained_case_on_candidate(
        session,
        Some(candidate),
        bad,
        correction,
        TaxSection::SelfEmployment,
        (&alloc::format!("coop-total-{field}"), "COOP_ALLOCATIONS_DO_NOT_RECONCILE", &path),
    );
}

#[test]
fn unit47_retained_gift_appreciation_sum_above_total_preserves_activity_sources() {
    let (draft, source) = crate::tests::credits::form_3800::retained_charity_draft();
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = retained_open(&mut session, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value: gift, .. } = &mut request else { unreachable!() };
    gift.basis_election = true;
    gift.carryovers.push(CharityCarryoverPage {
        year: 2023,
        class: CharityClass::CapitalGain30,
        balance: 1_000_000,
        appreciation: Some(400_000),
        same_filers: true,
        allowed_research_parts: vec![0],
        allowed_source_appreciation: vec![0],
        research_parts: Some(ResearchCharityPartsPage::Parts {
            sources: vec![
                ResearchCharityPartPage {
                    origin: ResearchCharityOriginPage::CurrentK1 { source },
                    amount: 200_000,
                    original_appreciation: Some(CharitySourceAppreciation::Amount {
                        amount: 200_000,
                    }),
                },
                ResearchCharityPartPage {
                    origin: ResearchCharityOriginPage::OtherEntity(OtherCharityEntity {
                        name: "Synthetic retained donor source".into(),
                        ein: "98-7654321".into(),
                    }),
                    amount: 200_000,
                    original_appreciation: Some(CharitySourceAppreciation::Amount {
                        amount: 200_000,
                    }),
                },
            ],
        }),
    });
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 210_000);
    // Basis election converts this $10,000 retained gift with $4,000
    // original appreciation to a $6,000 cost claim, below the class limit.
    assert_eq!(value(&result, "charity_carry_2023_capital_gain_thirty_used"), 6_000);
    let correction = retained_open(&mut session, TaxSection::GiftCarryovers);
    let mut bad = correction.clone();
    let P::SaveGiftCarryovers { value, .. } = &mut bad else { unreachable!() };
    let Some(ResearchCharityPartsPage::Parts { sources }) = &mut value.carryovers[1].research_parts
    else {
        unreachable!()
    };
    let Some(CharitySourceAppreciation::Amount { amount }) = &mut sources[1].original_appreciation
    else {
        unreachable!()
    };
    *amount += 1;
    retained_case(
        session,
        bad,
        correction,
        TaxSection::GiftCarryovers,
        "gift-appreciation-total",
        "RETAINED_CHARITY_SOURCE_APPRECIATION_ABOVE_TOTAL",
        "[value/carryovers/1/research_parts/sources]",
    );
}

fn retained_nol_source(case: usize) -> K1EstateTrust {
    let finite = case == 12;
    let weeks = case == 13;
    let year = if finite || weeks { 2017 } else { 2025 };
    let mut page = estate_nol_page(
        "Synthetic actual retained fiduciary loss",
        year,
        20_000,
        10_000,
        if finite { Some(11) } else { None },
    );
    if weeks {
        estate_nol_weeks_history(&mut page, false);
    } else {
        estate_nol_original_period(&mut page, year * 10000 + 101, year * 10000 + 1231);
    }
    page
}

fn retained_nol_bad(nol: &mut FinalEstateTrustNol, case: usize) -> (&'static str, &'static str) {
    match case {
        0 => {
            nol.years[0].original_period.as_mut().unwrap().ended = 20270101;
            ("attr-0121-rule-001", "years/0/original_period/ended")
        }
        1 => {
            nol.taxable_periods[1].period =
                FinalEstateTrustLossPeriod { began: 20261231, ended: 20270101 };
            ("attr-0121-rule-002", "taxable_periods/1/period/ended")
        }
        2 => {
            nol.terminated_on = Some(20251231);
            ("attr-0121-rule-003", "terminated_on")
        }
        3 => {
            // No history chain: isolate the original loss's own termination cap.
            nol.taxable_periods.clear();
            nol.terminated_on = Some(20260601);
            nol.years[0].original_period.as_mut().unwrap().ended = 20261231;
            ("attr-0121-rule-004", "years/0/original_period/ended")
        }
        4 => {
            nol.taxable_periods.clear();
            let mut second = nol.years[0].clone();
            second.year = 2024;
            second.original_period =
                Some(FinalEstateTrustLossPeriod { began: 20240101, ended: 20250630 });
            nol.years.push(second);
            ("attr-0121-rule-006", "years/1/original_period/began")
        }
        5 => {
            nol.terminated_on = None;
            ("attr-0121-rule-007", "terminated_on")
        }
        6 => {
            // The canonical endpoint check owns this mismatch before the
            // later per-loss checks; it points to the actual termination input.
            nol.terminated_on = Some(20260601);
            ("attr-0121-rule-008", "terminated_on")
        }
        7 | 9 => {
            nol.taxable_periods[1].period.began = 20260102;
            (
                if case == 7 { "attr-0121-rule-009" } else { "attr-0121-rule-011" },
                "taxable_periods/1/period/began",
            )
        }
        8 => {
            nol.taxable_periods[1].period =
                FinalEstateTrustLossPeriod { began: 20251230, ended: 20261229 };
            nol.terminated_on = Some(20261229);
            ("attr-0121-rule-010", "taxable_periods/1/period/began")
        }
        10 => {
            nol.years[0].original_period = None;
            ("attr-0121-rule-012", "years/0/original_period")
        }
        11 => {
            nol.taxable_periods.clear();
            nol.years[0].remaining_years = Some(1);
            ("attr-0121-rule-013", "years/0/remaining_years")
        }
        12 => {
            nol.years[0].remaining_years = Some(12);
            ("attr-0121-rule-014", "years/0/remaining_years")
        }
        13 => {
            nol.years[0].remaining_years = Some(11);
            ("attr-0121-rule-015", "years/0/remaining_years")
        }
        14 => {
            nol.years[0].year = 2024;
            ("attr-0122-rule-001", "years/0/original_period/began")
        }
        15 => {
            nol.years[0].original_period =
                Some(FinalEstateTrustLossPeriod { began: 20251231, ended: 20250101 });
            ("attr-0123-rule-001", "years/0/original_period/ended")
        }
        16 => {
            nol.taxable_periods[1].period.began = 20250101;
            ("attr-0124-rule-001", "taxable_periods/1/period/ended")
        }
        17 => {
            nol.taxable_periods[1].period.began = 20251229;
            nol.taxable_periods[1].accounting =
                FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
                    annual_began: 20260102,
                };
            ("attr-0124-rule-002", "taxable_periods/1/accounting/annual_began")
        }
        18..=24 => {
            let p = &mut nol.taxable_periods[1];
            p.period = FinalEstateTrustLossPeriod { began: 20260101, ended: 20261225 };
            let (annual_began, annual_ended, reference_ended, closing, local, id) = match case {
                18 => (
                    20251227,
                    20261226,
                    20261231,
                    FinalEstateTrustWeekClosing::Last,
                    "accounting/annual_ended",
                    "attr-0124-rule-003",
                ),
                19 => {
                    p.period.began = 20251220;
                    (
                        20251227,
                        20261225,
                        20261231,
                        FinalEstateTrustWeekClosing::Last,
                        "period/began",
                        "attr-0124-rule-004",
                    )
                }
                20 => (
                    20251227,
                    20261225,
                    20261230,
                    FinalEstateTrustWeekClosing::Last,
                    "accounting/reference_ended",
                    "attr-0124-rule-005",
                ),
                21 => {
                    p.period = FinalEstateTrustLossPeriod { began: 20260102, ended: 20261231 };
                    (
                        20260102,
                        20261231,
                        20261231,
                        FinalEstateTrustWeekClosing::Last,
                        "accounting/annual_began",
                        "attr-0124-rule-006",
                    )
                }
                22 => {
                    p.period = FinalEstateTrustLossPeriod { began: 20260105, ended: 20261231 };
                    (
                        20260105,
                        20270103,
                        20261231,
                        FinalEstateTrustWeekClosing::Nearest,
                        "accounting/annual_began",
                        "attr-0124-rule-007",
                    )
                }
                23 => (
                    20251229,
                    20270103,
                    20261231,
                    FinalEstateTrustWeekClosing::Last,
                    "accounting/annual_ended",
                    "attr-0124-rule-008",
                ),
                _ => (
                    20251230,
                    20270104,
                    20261231,
                    FinalEstateTrustWeekClosing::Nearest,
                    "accounting/annual_ended",
                    "attr-0124-rule-009",
                ),
            };
            p.accounting = FinalEstateTrustAccountingYear::Weeks52Or53 {
                annual_began,
                annual_ended,
                reference_ended,
                closing,
            };
            let path = match local {
                "period/began" => "taxable_periods/1/period/began",
                "accounting/reference_ended" => "taxable_periods/1/accounting/reference_ended",
                "accounting/annual_began" => "taxable_periods/1/accounting/annual_began",
                _ => "taxable_periods/1/accounting/annual_ended",
            };
            (id, path)
        }
        _ => unreachable!(),
    }
}

#[test]
fn unit47_final_fiduciary_nol_period_keys_bind_all_twenty_five_original_conditions() {
    for case in 0..25 {
        let session = retained_nol_fixture(case);
        retained_nol_case(session, case);
    }
}

fn retained_nol_fixture(case: usize) -> Interview {
    let (session, reply) = save_k1s(vec![retained_nol_source(case)]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "prior_nol_deduction"), 20_000);
    assert_eq!(value(&result, "prior_atnol_deduction"), 10_000);
    session
}

#[inline(never)]
fn retained_nol_case(mut session: Interview, case: usize) {
    let correction = open_documents(&mut session);
    let mut bad = correction.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    let (index, nol) = documents
        .common
        .k1
        .documents
        .iter_mut()
        .enumerate()
        .find_map(|(index, row)| match &mut row.document {
            crate::K1Document::K1EstateTrust(source) => {
                source.statements.final_nol.as_mut().map(|nol| (index, nol))
            }
            _ => None,
        })
        .unwrap();
    let (original_clause, local) = retained_nol_bad(nol, case);
    retained_case(
        session,
        bad,
        correction,
        TaxSection::PassThrough,
        original_clause,
        "ESTATE_NOL_PERIOD_INVALID",
        &alloc::format!(
            "[documents/common/k1/documents/{index}/document/statements/final_nol/{local}]"
        ),
    );
}
