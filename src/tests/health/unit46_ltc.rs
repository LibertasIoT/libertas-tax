// Raw long-term-care allocation and unchanged tax-owner acceptance.
use super::*;
use crate::tests::cross::interview::{accepted, rejected_with};
use crate::tests::support::client::income_pages::{
    open_documents, open_topic, page_documents as all_page_documents, save_documents, save_topic,
};
use alloc::string::String;

fn page_documents(page: &P) -> Vec<IncomeDocumentItem> {
    all_page_documents(page)
        .into_iter()
        .filter(|row| {
            matches!(
                row.document,
                IncomeSource::OtherIncome { .. }
                    | IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
            )
        })
        .collect()
}

fn periods(
    starts: u32,
    ends: u32,
    services: i64,
    reimbursements: i64,
    recipient: CareBenefitRecipient,
) -> LtcPeriods {
    LtcPeriods::One {
        starts,
        ends,
        services: services * 100,
        reimbursements: reimbursements * 100,
        recipient,
    }
}
fn published_single() -> LtcPeriods {
    periods(
        20260101,
        20261231,
        54_750,
        27_375,
        CareBenefitRecipient::Insured { other_payees: false },
    )
}
fn published_casey() -> LtcPeriods {
    periods(
        20260701,
        20261231,
        27_600,
        13_800,
        CareBenefitRecipient::Other { insured_received: 1_200_000, others_received: 3_300_000 },
    )
}
fn item(document: IncomeSource) -> IncomeDocumentItem {
    crate::tests::support::builders::survey::survey_uncatalogued_document(
        Owner::Taxpayer,
        0,
        document,
    )
}
fn put(session: &mut Interview, new: Vec<IncomeDocumentItem>) {
    let record = new.iter().all(|row| matches!(row.document, IncomeSource::Record { .. }));
    let page = if record {
        open_topic(session, TaxSection::LongTermCareBenefits)
    } else {
        open_documents(session)
    };
    let mut rows = page_documents(&page);
    rows.extend(new);
    if record {
        let request = save_topic(session, TaxSection::LongTermCareBenefits, rows);
        accepted(&send(session, request));
    } else {
        accepted(&crate::tests::support::client::income_pages::save_source_fixtures(session, rows));
    }
}
fn record(periods: LtcPeriods, insured: LtcInsuredPerson) -> IncomeDocumentItem {
    item(IncomeSource::Record {
        value: crate::IncomeRecord::LtcInsured(LtcInsured { insured, periods: periods.into() }),
    })
}
fn form(amount: i64, insured: i64) -> IncomeDocumentItem {
    let FederalIncome::LtcBenefits { data } = ltc_form(amount, Some(insured)) else {
        unreachable!()
    };
    item(IncomeSource::OtherIncome {
        value: crate::OtherIncomeDocument::Ltc1099(ltc_document(data)),
    })
}
fn cents(result: &FederalResult, id: &str) -> i64 {
    result.lines.iter().find(|row| row.id == id).unwrap_or_else(|| panic!("missing {id}")).amount
}
fn result(periods: LtcPeriods, received: i64) -> FederalResult {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(ltc_record(periods)));
    add(&mut draft, income(ltc_form(received, Some(2))));
    supported(&draft)
}

// Published numeric oracles: 2026 DRAFT I8853, printed pp10–11 Examples1/2.
// Neutral completion uses this fixture's existing $60,000 wage owner; this is
// not a claim that the draft is a final filing artifact or a full-return oracle.
// https://www.irs.gov/pub/irs-dft/i8853--dft.pdf
#[test]
fn unit46_ltc_published_raw_allocation_exceeds_receipts_without_increasing_exclusion() {
    let single = result(published_single(), 24_000);
    assert_eq!(cents(&single, "ltc_benefit_2_1_received"), 2_400_000);
    assert_eq!(cents(&single, "ltc_benefit_2_1_limit"), 12_957_500);
    assert_eq!(cents(&single, "ltc_benefit_2_1_allocated_limit"), 12_957_500);
    assert_eq!(cents(&single, "ltc_benefit_2_1_excluded"), 2_400_000);
    assert_eq!(value(&single, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&single, "agi"), 60_000);
    let casey = result(published_casey(), 18_000);
    assert_eq!(cents(&casey, "ltc_benefit_2_1_limit"), 6_532_000);
    // Exact 53,320 * 18/51 = 18,818.823529..., existing ratio kernel rounds cents.
    // The printed example's whole-dollar18,819 is a separate final projection.
    assert_eq!(cents(&casey, "ltc_benefit_2_1_allocated_limit"), 1_881_882);
    assert_eq!(
        crate::kernel::ratio::dollars(cents(&casey, "ltc_benefit_2_1_allocated_limit")),
        18_819
    );
    assert_eq!(cents(&casey, "ltc_benefit_2_1_received"), 1_800_000);
    assert_eq!(cents(&casey, "ltc_benefit_2_1_excluded"), 1_800_000);
    assert_eq!(value(&casey, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&casey, "agi"), 60_000);
    let row = casey.lines.iter().find(|row| row.id == "ltc_benefit_2_1_allocated_limit").unwrap();
    assert_eq!(row.source, "https://www.irs.gov/instructions/i8853");
    assert!(row.inputs.iter().any(|input| input == "income.ltc_benefits"));
}

// Rule-derived controls: priority consumes the actual insured's payments;
// reimbursement never creates a negative limit; no other-payee denominator
// means no allocation. These are boundary controls, not published examples.
#[test]
fn unit46_ltc_raw_allocation_keeps_priority_zero_denominator_and_taxable_boundaries() {
    let base = CareBenefitPeriod {
        injured_payees: None,
        starts: 20260101,
        ends: 20260101,
        received: 20_000,
        services: 0,
        reimbursements: 0,
        recipient: CareBenefitRecipient::Other {
            insured_received: 10_000,
            others_received: 90_000,
        },
    };
    // Daily430 less insured100 leaves330; this return gets200/1100 =60.
    assert_eq!(base.amounts(), (43_000, 6_000, 6_000, 14_000));
    let mut zero = base.clone();
    zero.received = 0;
    zero.recipient = CareBenefitRecipient::Other { insured_received: 0, others_received: 0 };
    assert_eq!(zero.amounts(), (43_000, 0, 0, 0));
    let mut exhausted = base.clone();
    exhausted.recipient =
        CareBenefitRecipient::Other { insured_received: 50_000, others_received: 90_000 };
    assert_eq!(exhausted.amounts(), (43_000, 0, 0, 20_000));
    let mut reimbursed = base;
    reimbursed.reimbursements = 50_000;
    assert_eq!(reimbursed.amounts(), (0, 0, 0, 20_000));
}

#[test]
fn unit46_ltc_raw_limits_do_not_offset_taxable_payments_of_another_period() {
    let first = CareBenefitPeriod {
        injured_payees: None,
        starts: 20260101,
        ends: 20260101,
        received: 50_000,
        services: 0,
        reimbursements: 0,
        recipient: CareBenefitRecipient::Insured { other_payees: false },
    };
    let second =
        CareBenefitPeriod { starts: 20260102, ends: 20260102, received: 20_000, ..first.clone() };
    let output = result(
        LtcPeriods::Several { method: CarePeriodMethod::EqualRate, periods: vec![first, second] },
        700,
    );
    assert_eq!(cents(&output, "ltc_benefit_2_1_allocated_limit"), 43_000);
    assert_eq!(cents(&output, "ltc_benefit_2_2_allocated_limit"), 43_000);
    assert_eq!(value(&output, "ltc_benefit_2_1_taxable"), 70);
    assert_eq!(value(&output, "ltc_benefit_2_2_taxable"), 0);
    assert_eq!(value(&output, "taxable_ltc_benefits"), 70);
    assert_eq!(value(&output, "agi"), 60_070);
}

#[test]
fn unit46_ltc_raw_allocation_normal_save_source_aggregate_reopen_restart_and_invalid_atomicity() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    put(
        &mut session,
        vec![
            record(published_single(), LtcInsuredPerson::OnReturn { person: -1 }),
            record(published_casey(), LtcInsuredPerson::Other { name: "Alex Sample".into() }),
        ],
    );
    let page = open_topic(&mut session, TaxSection::LongTermCareBenefits);
    let ids = page_documents(&page)
        .iter()
        .filter(|row| {
            matches!(
                row.document,
                IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
            )
        })
        .map(|row| row.id)
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    // Two genuine entered forms for this return aggregate18000 once into the
    // Casey period. Synthetic split contracts preserve the published total.
    put(&mut session, vec![form(24_000, ids[0]), form(14_000, ids[1]), form(4000, ids[1])]);
    let saved = session.draft.clone();
    let output = supported(&saved);
    let single_key = alloc::format!("ltc_benefit_{}_1_allocated_limit", ids[0]);
    let casey_key = alloc::format!("ltc_benefit_{}_1_allocated_limit", ids[1]);
    assert_eq!(cents(&output, &single_key), 12_957_500);
    assert_eq!(cents(&output, &casey_key), 1_881_882);
    assert_eq!(value(&output, "agi"), 60_000);
    let reopened = open_topic(&mut session, TaxSection::LongTermCareBenefits);
    let expected = page_documents(&reopened)
        .iter()
        .map(|row| (row.id, row.document.clone()))
        .collect::<Vec<_>>();
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let reopened = open_topic(&mut restored, TaxSection::LongTermCareBenefits);
    assert_eq!(
        page_documents(&reopened)
            .iter()
            .map(|row| (row.id, row.document.clone()))
            .collect::<Vec<_>>(),
        expected
    );
    let mut invalid = reopened;
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    let row = documents.documents.iter_mut().find(|row| row.id == ids[1]).unwrap();
    let crate::IncomeRecord::LtcInsured(record) = &mut row.document else { unreachable!() };
    let LtcPeriodsInput::One { ends, .. } = &mut record.periods else { unreachable!() };
    *ends = 20260630; // A real Gregorian date before the July1 period starts.
    rejected_with(send(&mut restored, invalid), "LTC_PERIOD_ENDS_BEFORE_START");
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), output);
}

// Existing ordinary missing-source behavior remains explicit: the actual form
// is taxed in full with a notice and cannot borrow another insured's raw limit.
#[test]
fn unit46_ltc_missing_source_cannot_borrow_a_raw_limit_from_another_insured() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(ltc_record(published_single())));
    add(&mut draft, income(ltc_form(18_000, Some(9999))));
    let output = supported(&draft);
    assert_eq!(value(&output, "taxable_ltc_benefits"), 18_000);
    assert_eq!(value(&output, "agi"), 78_000);
    assert!(noted(&output, "WARN_LTC_PERIODS_MISSING"));
    assert_eq!(cents(&output, "ltc_benefit_2_1_received"), 0);
    assert_eq!(cents(&output, "ltc_benefit_2_1_excluded"), 0);
}

// The IRS Example1 expressly establishes sole-payee status. The required
// source fact now carries that choice; a shared period carries the opposite.
#[test]
fn unit46_ltc_required_payee_fact_selects_full_limit_or_insured_priority() {
    // Published sole-payee fact permits the aggregate amount as line25.
    let sole_payee = true;
    let source_limit = crate::kernel::phase::floor(365 * 43_000, 5_475_000) - 2_737_500;
    assert!(sole_payee);
    let expected_raw = source_limit;
    assert_eq!(expected_raw, 12_957_500);
    let period = CareBenefitPeriod {
        injured_payees: None,
        starts: 20260101,
        ends: 20261231,
        received: 2_400_000,
        services: 5_475_000,
        reimbursements: 2_737_500,
        recipient: CareBenefitRecipient::Insured { other_payees: false },
    };
    let (aggregate, raw, excluded, taxable) = period.amounts();
    assert_eq!(aggregate, 12_957_500);
    assert_eq!(raw, expected_raw);
    assert_eq!((excluded, taxable), (2_400_000, 0));
    let mut shared = period;
    shared.starts = 20260701;
    shared.services = 2_760_000;
    shared.reimbursements = 1_380_000;
    shared.received = 1_200_000;
    shared.recipient = CareBenefitRecipient::Insured { other_payees: true };
    assert_eq!(shared.amounts(), (6_532_000, 1_200_000, 1_200_000, 0));
    // Omission of the required Boolean cannot become an implicit false on the
    // native wire. Avro decode must fail before any normal Save is possible.
    for other_payees in [false, true] {
        let fact = CareBenefitRecipient::Insured { other_payees };
        let payload = fact.to_avro();
        assert_eq!(CareBenefitRecipient::from_avro(&payload).unwrap(), fact);
        assert!(CareBenefitRecipient::from_avro(&payload[..payload.len() - 1]).is_err());
    }
}

// Environment bridge for the actual Studio long-term-care editor. Captures the
// real seed and independent visible answers; normal native calculation replies
// and normal Save/reopen/restart remain the only acceptance path.
#[test]
fn unit46_ltc_actual_payee_fact_normal_save_reopen_restart() {
    extern crate std;
    for (case, facts, received, raw) in [
        ("sole", published_single(), 24_000, 12_957_500),
        ("other", published_casey(), 18_000, 1_881_882),
        (
            "shared",
            periods(
                20260701,
                20261231,
                27_600,
                13_800,
                CareBenefitRecipient::Insured { other_payees: true },
            ),
            12_000,
            1_200_000,
        ),
        (
            "viatical",
            periods(
                20260601,
                20260601,
                0,
                0,
                CareBenefitRecipient::Insured { other_payees: false },
            ),
            500,
            43_000,
        ),
    ] {
        if let Ok(selected) = std::env::var("TAX_UNIT46_LTC_CASE")
            && selected != case
        {
            continue;
        }
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let insured = if case == "other" {
            LtcInsuredPerson::Other { name: "Alex Sample".into() }
        } else {
            LtcInsuredPerson::OnReturn { person: -1 }
        };
        let FederalIncome::LtcBenefits { mut data } = ltc_form(received, None) else {
            unreachable!()
        };
        data.accelerated_origin = None; // No source question for a zero box2 stream.
        if case == "viatical" {
            data.form.benefits = Some(0);
            data.form.accelerated = Some(50_000);
            data.interest = Some(2_500);
            data.benefit_origin = None;
            data.accelerated_origin = Some(qualified_viatical(false));
            data.contract = LtcContract::Unknown; // Box4 does not decide life proceeds.
        }
        let mut expected_form = data;
        if expected_form.form.box4_qualified {
            expected_form.contract = LtcContract::Unknown;
        }
        put(&mut session, vec![record(facts.clone(), insured)]);
        let seed = open_documents(&mut session);
        let mut request = save_documents(
            &mut session,
            vec![item(IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Ltc1099(ltc_document(expected_form.clone())),
            })],
        );
        if std::env::var("TAX_UNIT46_LTC_CASE").as_deref() == Ok(case) {
            if let Ok(input) = std::env::var("TAX_UNIT46_LTC_CALC_INPUT") {
                let calculation = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
                assert!(matches!(calculation, P::SaveDocuments { .. }));
                let before = session.draft.clone();
                let Reply::Response(reply) =
                    crate::tests::support::client::income_pages::candidate_save_review(
                        &mut session,
                        calculation,
                    )
                else {
                    panic!("expected normal document calculation response")
                };
                assert!(matches!(reply, P::IncomePreview { .. } | P::Review { .. }));
                assert_eq!(session.draft, before);
                std::fs::write(
                    std::env::var("TAX_UNIT46_LTC_CALC_OUTPUT").unwrap(),
                    reply.to_avro(),
                )
                .unwrap();
                return;
            }
            if let Ok(path) = std::env::var("TAX_UNIT46_LTC_SEED") {
                std::fs::write(path, seed.to_avro()).unwrap();
            }
            if let Ok(path) = std::env::var("TAX_UNIT46_LTC_ANSWERS") {
                std::fs::write(path, request.to_avro()).unwrap();
            }
            if let Ok(path) = std::env::var("TAX_UNIT46_LTC_PAYLOAD") {
                request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
                assert!(matches!(request, P::SaveDocuments { .. }));
            }
        }
        accepted(&send(&mut session, request));
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut session,
            &[item(IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Ltc1099(ltc_document(expected_form.clone())),
            })],
        );
        let saved = session.draft.clone();
        let id = saved
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::LtcInsured { .. }))
            .unwrap()
            .id;
        let output = supported(&saved);
        assert_eq!(cents(&output, &alloc::format!("ltc_benefit_{id}_1_allocated_limit")), raw);
        assert_eq!(cents(&output, &alloc::format!("ltc_benefit_{id}_1_received")), received * 100);
        assert_eq!(value(&output, "taxable_ltc_benefits"), if case == "viatical" { 70 } else { 0 });
        assert_eq!(value(&output, "agi"), if case == "viatical" { 60_095 } else { 60_000 });
        let reopened = open_documents(&mut session);
        let record_page = open_topic(&mut session, TaxSection::LongTermCareBenefits);
        let reopened_documents = page_documents(&record_page);
        let IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(record) } =
            &reopened_documents.iter().find(|row| row.id == id).unwrap().document
        else {
            unreachable!()
        };
        assert_eq!(LtcPeriods::from(record.periods.clone()), facts);
        let expected = expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Ltc1099(ltc_document(expected_form.clone())),
            },
        );
        assert!(page_documents(&reopened).iter().any(|row| row.document == expected));
        if std::env::var("TAX_UNIT46_LTC_CASE").as_deref() == Ok(case)
            && let Ok(path) = std::env::var("TAX_UNIT46_LTC_REOPEN")
        {
            std::fs::write(path, reopened.to_avro()).unwrap();
        }
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        assert_eq!(supported(&restored.draft), output);
        let record_page = open_topic(&mut restored, TaxSection::LongTermCareBenefits);
        let reopened_documents = page_documents(&record_page);
        let IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(record) } =
            &reopened_documents.iter().find(|row| row.id == id).unwrap().document
        else {
            unreachable!()
        };
        assert_eq!(LtcPeriods::from(record.periods.clone()), facts);
    }
}

fn qualified_viatical(state_requires_license: bool) -> LtcPaymentOrigin {
    LtcPaymentOrigin::Viatical {
        regular_business: true,
        regulation: if state_requires_license {
            ViaticalRegulation::StateLicense { licensed: true }
        } else {
            ViaticalRegulation::NoStateLicense { naic_compliant: true }
        },
        business_insurable_interest: false,
    }
}

// Pub525 final2025 Accelerated Death Benefits; source-derived controls, not
// published full-return examples. Terminal900 excludes with an actual qualifying
// provider; additional interest25 remains income. Chronic500 for one2026day
// exceeds the430 limit by70, independently of the optional box4 qualification.
// https://www.irs.gov/publications/p525
// https://www.irs.gov/pub/irs-dft/i8853--dft.pdf (2026draftSectionC)
#[test]
fn unit46_ltc_actual_viatical_terminal_source_save_reopen_restart_and_qualification_negatives() {
    let form = crate::LtcSource {
interest: Some(2_500),
        form: crate::Ltc1099Paper {
            payer: Default::default(),
            recipient: Default::default(),
            insured_party: Default::default(),
            date_certified: None,
            benefits: Some(0),
            accelerated: Some(90_000),
            paid: LtcPaidBasis::NotChecked,
            box4_qualified: false,
            illness: Some(LtcIllness::Terminal),
            calendar_year: 2026,
            corrected: false,
            payer_phone: String::new(),
            account_number: String::new(),
        },
        benefit_origin: None,
        accelerated_origin: Some(qualified_viatical(true)),
        contract: LtcContract::Unknown,
        insured: None,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    put(
        &mut session,
        vec![item(IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Ltc1099(ltc_document(form.clone())),
        })],
    );
    let saved = session.draft.clone();
    let source = saved
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::LtcBenefits { .. }))
        .unwrap()
        .id;
    let output = supported(&saved);
    assert_eq!(value(&output, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&output, "agi"), 60_025);
    assert_eq!(cents(&output, &alloc::format!("ltc_benefit_{source}_exempt")), 90_000);
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let reopened = open_documents(&mut restored);
    let expected = expected_named_source(
        &restored.draft,
        Owner::Taxpayer,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Ltc1099(ltc_document(form.clone())),
        },
    );
    assert!(page_documents(&reopened).iter().any(|row| row.document == expected));
    for case in 0..4 {
        let mut bad = open_topic(&mut restored, TaxSection::LongTermCareBenefits);
        let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
        let crate::DocumentAnswer::LtcAcceleratedOrigin { value: origin } = &mut followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{source}/ltc_accelerated_origin"))
            .unwrap()
            .answer
        else {
            unreachable!()
        };
        let Some(LtcPaymentOrigin::Viatical {
            regular_business,
            regulation,
            business_insurable_interest,
            ..
        }) = origin
        else {
            unreachable!()
        };
        match case {
            0 => *regular_business = false,
            1 => *regulation = ViaticalRegulation::NoStateLicense { naic_compliant: false },
            2 => *regulation = ViaticalRegulation::StateLicense { licensed: false },
            3 => *business_insurable_interest = true,
            _ => unreachable!(),
        }
        accepted(&send(&mut restored, bad));
        let unresolved = estimate(&restored.draft);
        assert_ne!(unresolved.state, ResultState::EstimatedForSupportedScope);
        assert!(unresolved.tax.is_none() && unresolved.refund.is_none());
        assert!(
            unresolved
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED")
        );
        let mut restarted = restart(restored.draft.clone());
        assert_eq!(estimate(&restarted.draft), unresolved);
        let mut good = open_topic(&mut restarted, TaxSection::LongTermCareBenefits);
        let P::SaveIncomes { followups, .. } = &mut good else { unreachable!() };
        let crate::DocumentAnswer::LtcAcceleratedOrigin { value } = &mut followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{source}/ltc_accelerated_origin"))
            .unwrap()
            .answer
        else {
            unreachable!()
        };
        *value = form.accelerated_origin.clone();
        accepted(&send(&mut restarted, good));
        // A legitimate correction Save advances the accepted revision while
        // restoring the same source facts and every calculated tax amount.
        let mut expected = output.clone();
        expected.revision = restarted.draft.revision;
        assert_eq!(supported(&restarted.draft), expected);
        restored = restarted;
    }
    let before = restored.draft.clone();
    let mut forged = open_topic(&mut restored, TaxSection::LongTermCareBenefits);
    let P::SaveIncomes { followups, .. } = &mut forged else { unreachable!() };
    let crate::DocumentAnswer::LtcAcceleratedOrigin { value } = &mut followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{source}/ltc_accelerated_origin"))
        .unwrap()
        .answer
    else {
        unreachable!()
    };
    *value = Some(LtcPaymentOrigin::Insurance);
    rejected_with(send(&mut restored, forged), "LTC1099_ORIGIN_STREAM");
    assert_eq!(restored.draft, before);
}

#[test]
fn unit46_ltc_actual_accelerated_life_chronic_source_uses_period_limit_once() {
    let form = crate::LtcSource {
interest: None,
        form: crate::Ltc1099Paper {
            payer: Default::default(),
            recipient: Default::default(),
            insured_party: Default::default(),
            date_certified: None,
            benefits: Some(0),
            accelerated: Some(50_000),
            paid: LtcPaidBasis::PerDiem,
            box4_qualified: false,
            illness: Some(LtcIllness::Chronic),
            calendar_year: 2026,
            corrected: false,
            payer_phone: String::new(),
            account_number: String::new(),
        },
        benefit_origin: None,
        accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
            business_insurable_interest: false,
        }),
        contract: LtcContract::Unknown,
        insured: None,
    };
    let facts =
        periods(20260601, 20260601, 0, 0, CareBenefitRecipient::Insured { other_payees: false });
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    put(&mut session, vec![record(facts, LtcInsuredPerson::OnReturn { person: -1 })]);
    put(
        &mut session,
        vec![item(IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Ltc1099(ltc_document(form.clone())),
        })],
    );
    let saved = session.draft.clone();
    let output = supported(&saved);
    assert_eq!(value(&output, "taxable_ltc_benefits"), 70);
    assert_eq!(value(&output, "agi"), 60_070);
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let reopened = open_documents(&mut restored);
    let expected = expected_named_source(
        &restored.draft,
        Owner::Taxpayer,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Ltc1099(ltc_document(form.clone())),
        },
    );
    assert!(page_documents(&reopened).iter().any(|row| row.document == expected));
    let mut reopened = open_topic(&mut restored, TaxSection::LongTermCareBenefits);
    let P::SaveIncomes { followups, .. } = &mut reopened else { unreachable!() };
    let crate::DocumentAnswer::LtcAcceleratedOrigin { value } = &mut followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::LtcAcceleratedOrigin { .. }))
        .unwrap()
        .answer
    else {
        unreachable!()
    };
    *value = Some(LtcPaymentOrigin::AcceleratedLife { business_insurable_interest: true });
    accepted(&send(&mut restored, reopened));
    let unresolved = estimate(&restored.draft);
    assert_ne!(unresolved.state, ResultState::EstimatedForSupportedScope);
    assert!(
        unresolved
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED")
    );
}

#[test]
fn unit47_ltc_native_receipts_keep_source_bounds_and_refuse_actual_receiver_overflow() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let FederalIncome::LtcBenefits { data: mut answered } = ltc_form(1, Some(0)) else {
        unreachable!()
    };
    answered.insured = None;
    answered.form.benefits = Some(100_000_000_000);
    answered.form.accelerated = Some(100_000_000_000);
    assert_eq!(answered.record().periodic(), 200_000_000_000);
    let source = item(IncomeSource::OtherIncome {
        value: crate::OtherIncomeDocument::Ltc1099(ltc_document(answered)),
    });
    put(&mut session, vec![source]);
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(value(&supported(&saved), "taxable_ltc_benefits"), 2_000_000_000);
    let mut cold = restart(saved.clone());
    let request = open_documents(&mut cold);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_ltc_native_payee_cohort_keeps_all_actual_sources_and_large_parts() {
    // Two actual sources suffice for distinct membership and duplicate rejection;
    // each combined payment still crosses the former individual amount bound.
    let draft = crate::tests::support::builders::ltc_native_payee_cohort(
        2,
        100_000_000_000,
        100_000_000_000,
    );
    assert!(draft.valid_stored());
    assert!(crate::injured_spouse_sources::reference_issue(&draft).is_none());
    let mut session = Interview::new(TEST_USER, draft);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let request = open_documents(&mut session);
    accepted(&send(&mut session, request));
    // Accept the native periods through their editor before the cold resave.
    let request = open_topic(&mut session, TaxSection::LongTermCareBenefits);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(value(&supported(&saved), "taxable_ltc_benefits"), 3_999_999_570);
    let mut cold = restart(saved.clone());
    let mut request = open_topic(&mut cold, TaxSection::LongTermCareBenefits);
    accepted(&send(&mut cold, request.clone()));
    assert_eq!(cold.draft, saved);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::LtcInsured(record) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let LtcPeriodsInput::Several { periods, .. } = &mut record.periods else { unreachable!() };
    let Some(InjuredLtcPeriodPayeesInput::Payments { payments }) = &mut periods[0].injured_payees
    else {
        unreachable!()
    };
    let sources: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::LtcBenefits { .. }))
        .map(|entry| entry.id)
        .collect();
    assert_eq!(sources.len(), 2);
    assert_eq!(payments.iter().map(|payment| payment.source).collect::<Vec<_>>(), sources);
    assert!(payments.iter().all(|payment| payment.amount == 200_000_000_000));
    payments[1].source = payments[0].source;
    rejected_with(send(&mut cold, request), "INJURED_LTC_SOURCE_TWICE");
    assert_eq!(cold.draft, saved);
    let too_wide = InjuredLtcPeriodPayees::Payments {
        payments: vec![
            InjuredLtcPeriodPayment { source: 1, amount: i64::MAX },
            InjuredLtcPeriodPayment { source: 2, amount: 1 },
        ],
    };
    assert_eq!(too_wide.issue(i64::MAX).unwrap().key, "INJURED_LTC_PAYEE_TOTAL");
}
