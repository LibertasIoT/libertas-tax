// Raw long-term-care allocation and unchanged tax-owner acceptance.
use crate::tests::cross::interview::{accepted, open_topic, page_documents, rejected_with};
use super::*;

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
        CareBenefitRecipient::Insured {
            other_payees: false,
        },
    )
}
fn published_casey() -> LtcPeriods {
    periods(
        20260701,
        20261231,
        27_600,
        13_800,
        CareBenefitRecipient::Other {
            insured_received: 1_200_000,
            others_received: 3_300_000,
        },
    )
}
fn item(document: IncomeDocument) -> IncomeDocumentItem {
    crate::tests::support::builders::survey::survey_uncatalogued_document(Owner::Taxpayer, 0, document)
}
fn put(session: &mut Interview, new: Vec<IncomeDocumentItem>) {
    let mut request = open_topic(session, TaxSection::OtherIncome);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    documents.documents.extend(new);
    accepted(&send(session, request));
}
fn record(periods: LtcPeriods, insured: LtcInsuredPerson) -> IncomeDocumentItem {
    item(IncomeDocument::LtcInsured(LtcInsured {
        insured,
        periods: periods.into(),
    }))
}
fn form(amount: i64, insured: i64) -> IncomeDocumentItem {
    let FederalIncome::LtcBenefits { data } = ltc_form(amount, Some(insured)) else {
        unreachable!()
    };
    item(IncomeDocument::Ltc1099(data.page()))
}
fn cents(result: &FederalResult, id: &str) -> i64 {
    result
        .lines
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("missing {id}"))
        .amount
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
    assert_eq!(
        cents(&single, "ltc_benefit_2_1_allocated_limit"),
        12_957_500
    );
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
    let row = casey
        .lines
        .iter()
        .find(|row| row.id == "ltc_benefit_2_1_allocated_limit")
        .unwrap();
    assert_eq!(row.source, "https://www.irs.gov/instructions/i8853");
    assert!(
        row.inputs
            .iter()
            .any(|input| input == "income.ltc_benefits")
    );
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
    zero.recipient = CareBenefitRecipient::Other {
        insured_received: 0,
        others_received: 0,
    };
    assert_eq!(zero.amounts(), (43_000, 0, 0, 0));
    let mut exhausted = base.clone();
    exhausted.recipient = CareBenefitRecipient::Other {
        insured_received: 50_000,
        others_received: 90_000,
    };
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
        LtcPeriods::Several {
            method: CarePeriodMethod::EqualRate,
            periods: vec![first, second],
        },
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
            record(
                published_single(),
                LtcInsuredPerson::OnReturn { person: -1 },
            ),
            record(
                published_casey(),
                LtcInsuredPerson::Other {
                    name: "Alex Sample".into(),
                },
            ),
        ],
    );
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    let ids = page_documents(&page)
        .iter()
        .filter(|row| matches!(row.document, IncomeDocument::LtcInsured(_)))
        .map(|row| row.id)
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    // Two genuine entered forms for this return aggregate18000 once into the
    // Casey period. Synthetic split contracts preserve the published total.
    put(
        &mut session,
        vec![
            form(24_000, ids[0]),
            form(14_000, ids[1]),
            form(4000, ids[1]),
        ],
    );
    let saved = session.draft.clone();
    let output = supported(&saved);
    let single_key = alloc::format!("ltc_benefit_{}_1_allocated_limit", ids[0]);
    let casey_key = alloc::format!("ltc_benefit_{}_1_allocated_limit", ids[1]);
    assert_eq!(cents(&output, &single_key), 12_957_500);
    assert_eq!(cents(&output, &casey_key), 1_881_882);
    assert_eq!(value(&output, "agi"), 60_000);
    let reopened = open_topic(&mut session, TaxSection::OtherIncome);
    let expected = page_documents(&reopened)
        .iter()
        .map(|row| (row.id, row.document.clone()))
        .collect::<Vec<_>>();
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    assert_eq!(
        page_documents(&reopened)
            .iter()
            .map(|row| (row.id, row.document.clone()))
            .collect::<Vec<_>>(),
        expected
    );
    let mut invalid = reopened;
    let P::SaveIncomes { documents, .. } = &mut invalid else {
        unreachable!()
    };
    let row = documents
        .documents
        .iter_mut()
        .find(|row| row.id == ids[1])
        .unwrap();
    let IncomeDocument::LtcInsured(record) = &mut row.document else {
        unreachable!()
    };
    let LtcPeriodsInput::One { ends, .. } = &mut record.periods else {
        unreachable!()
    };
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
        recipient: CareBenefitRecipient::Insured {
            other_payees: false,
        },
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

// Environment bridge for the actual Studio Other-income editor. Captures the
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
                CareBenefitRecipient::Insured {
                    other_payees: false,
                },
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
        let mut request = open_topic(&mut session, TaxSection::OtherIncome);
        let seed = request.clone();
        let P::SaveIncomes { documents, .. } = &mut request else {
            unreachable!()
        };
        let insured = if case == "other" {
            LtcInsuredPerson::Other {
                name: "Alex Sample".into(),
            }
        } else {
            LtcInsuredPerson::OnReturn { person: -1 }
        };
        let FederalIncome::LtcBenefits { mut data } = ltc_form(received, None) else {
            unreachable!()
        };
        data.accelerated_origin = None; // No source question for a zero box2 stream.
        if case == "viatical" {
            data.benefits = 0;
            data.accelerated = 50_000;
            data.interest = 2_500;
            data.benefit_origin = None;
            data.accelerated_origin = Some(qualified_viatical(false));
            data.contract = LtcContract::Unknown; // Box4 does not decide life proceeds.
        }
        let expected_form = data.page();
        documents.documents = vec![
            record(facts.clone(), insured),
            item(IncomeDocument::Ltc1099(expected_form.clone())),
        ];
        if std::env::var("TAX_UNIT46_LTC_CASE").as_deref() == Ok(case) {
            if let Ok(input) = std::env::var("TAX_UNIT46_LTC_CALC_INPUT") {
                let calculation = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
                assert!(matches!(
                    calculation,
                    P::PreviewDocument { .. }
                        | P::EstimateDocuments { .. }
                        | P::DocumentPropertyOptions { .. }
                ));
                let before = session.draft.clone();
                let Reply::Response(reply) = send(&mut session, calculation) else {
                    panic!("expected normal document calculation response")
                };
                assert!(matches!(
                    reply,
                    P::IncomePreview { .. }
                        | P::RunningEstimate { .. }
                        | P::DocumentPropertyChoices { .. }
                ));
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
                assert!(matches!(request, P::SaveIncomes { .. }));
            }
        }
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let id = saved
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::LtcInsured { .. }))
            .unwrap()
            .id;
        let output = supported(&saved);
        assert_eq!(
            cents(
                &output,
                &alloc::format!("ltc_benefit_{id}_1_allocated_limit")
            ),
            raw
        );
        assert_eq!(
            cents(&output, &alloc::format!("ltc_benefit_{id}_1_received")),
            received * 100
        );
        assert_eq!(
            value(&output, "taxable_ltc_benefits"),
            if case == "viatical" { 70 } else { 0 }
        );
        assert_eq!(
            value(&output, "agi"),
            if case == "viatical" { 60_095 } else { 60_000 }
        );
        let reopened = open_topic(&mut session, TaxSection::OtherIncome);
        let IncomeDocument::LtcInsured(record) = &page_documents(&reopened)
            .iter()
            .find(|row| row.id == id)
            .unwrap()
            .document
        else {
            unreachable!()
        };
        assert_eq!(LtcPeriods::from(record.periods.clone()), facts);
        assert!(page_documents(&reopened).iter().any(
            |row| matches!(&row.document, IncomeDocument::Ltc1099(form) if form == &expected_form)
        ));
        if std::env::var("TAX_UNIT46_LTC_CASE").as_deref() == Ok(case)
            && let Ok(path) = std::env::var("TAX_UNIT46_LTC_REOPEN")
        {
            std::fs::write(path, reopened.to_avro()).unwrap();
        }
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        assert_eq!(supported(&restored.draft), output);
        let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
        let IncomeDocument::LtcInsured(record) = &page_documents(&reopened)
            .iter()
            .find(|row| row.id == id)
            .unwrap()
            .document
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
    let form = Ltc1099 {
        benefits: 0,
        accelerated: 90_000,
        interest: 2_500,
        benefit_origin: None,
        accelerated_origin: Some(qualified_viatical(true)),
        paid: LtcPaidBasis::NotChecked,
        contract: LtcContractPage::Unknown,
        illness: LtcIllness::Terminal,
        insured: None,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    put(
        &mut session,
        vec![item(IncomeDocument::Ltc1099(form.clone()))],
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
    assert_eq!(
        cents(&output, &alloc::format!("ltc_benefit_{source}_exempt")),
        90_000
    );
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    assert!(
        page_documents(&reopened)
            .iter()
            .any(|row| matches!(&row.document, IncomeDocument::Ltc1099(shown) if shown == &form))
    );
    for case in 0..4 {
        let mut bad = open_topic(&mut restored, TaxSection::OtherIncome);
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::Ltc1099(receipt) = &mut documents
            .documents
            .iter_mut()
            .find(|row| row.id == source)
            .unwrap()
            .document
        else {
            unreachable!()
        };
        let Some(LtcPaymentOrigin::Viatical {
            regular_business,
            regulation,
            business_insurable_interest,
            ..
        }) = &mut receipt.accelerated_origin
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
        let mut good = open_topic(&mut restarted, TaxSection::OtherIncome);
        let P::SaveIncomes { documents, .. } = &mut good else {
            unreachable!()
        };
        documents
            .documents
            .iter_mut()
            .find(|row| row.id == source)
            .unwrap()
            .document = IncomeDocument::Ltc1099(form.clone());
        accepted(&send(&mut restarted, good));
        // A legitimate correction Save advances the accepted revision while
        // restoring the same source facts and every calculated tax amount.
        let mut expected = output.clone();
        expected.revision = restarted.draft.revision;
        assert_eq!(supported(&restarted.draft), expected);
        restored = restarted;
    }
    let before = restored.draft.clone();
    let mut forged = open_topic(&mut restored, TaxSection::OtherIncome);
    let P::SaveIncomes { documents, .. } = &mut forged else {
        unreachable!()
    };
    let IncomeDocument::Ltc1099(receipt) = &mut documents
        .documents
        .iter_mut()
        .find(|row| row.id == source)
        .unwrap()
        .document
    else {
        unreachable!()
    };
    receipt.accelerated_origin = Some(LtcPaymentOrigin::Insurance);
    rejected_with(send(&mut restored, forged), "LTC1099_ORIGIN_STREAM");
    assert_eq!(restored.draft, before);
}

#[test]
fn unit46_ltc_actual_accelerated_life_chronic_source_uses_period_limit_once() {
    let form = Ltc1099 {
        benefits: 0,
        accelerated: 50_000,
        interest: 0,
        benefit_origin: None,
        accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
            business_insurable_interest: false,
        }),
        paid: LtcPaidBasis::PerDiem,
        contract: LtcContractPage::Unknown,
        illness: LtcIllness::Chronic,
        insured: None,
    };
    let facts = periods(
        20260601,
        20260601,
        0,
        0,
        CareBenefitRecipient::Insured {
            other_payees: false,
        },
    );
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    put(
        &mut session,
        vec![
            record(facts, LtcInsuredPerson::OnReturn { person: -1 }),
            item(IncomeDocument::Ltc1099(form.clone())),
        ],
    );
    let saved = session.draft.clone();
    let output = supported(&saved);
    assert_eq!(value(&output, "taxable_ltc_benefits"), 70);
    assert_eq!(value(&output, "agi"), 60_070);
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), output);
    let mut reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    assert!(
        page_documents(&reopened)
            .iter()
            .any(|row| matches!(&row.document, IncomeDocument::Ltc1099(shown) if shown == &form))
    );
    let P::SaveIncomes { documents, .. } = &mut reopened else {
        unreachable!()
    };
    let receipt = documents
        .documents
        .iter_mut()
        .find_map(|row| {
            if let IncomeDocument::Ltc1099(form) = &mut row.document {
                Some(form)
            } else {
                None
            }
        })
        .unwrap();
    receipt.accelerated_origin =
        Some(LtcPaymentOrigin::AcceleratedLife { business_insurable_interest: true });
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
    let mut source = form(1, 0);
    let IncomeDocument::Ltc1099(data) = &mut source.document else {
        unreachable!()
    };
    data.insured = None;
    data.benefits = 100_000_000_000;
    data.accelerated = 100_000_000_000;
    assert_eq!(data.record().periodic(), 200_000_000_000);
    put(&mut session, vec![source]);
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(
        value(&supported(&saved), "taxable_ltc_benefits"),
        2_000_000_000
    );
    let mut cold = restart(saved.clone());
    let mut request = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, request.clone()));
    assert_eq!(cold.draft, saved);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let IncomeDocument::Ltc1099(data) = &mut documents.documents[0].document else {
        unreachable!()
    };
    data.benefits = i64::MAX;
    data.accelerated = 1;
    assert_eq!(data.record().preview(), None);
    rejected_with(
        send(&mut cold, request),
        "MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE",
    );
    assert_eq!(cold.draft, saved);

    // Isolate the receipt owner from the retained individual source guards:
    // each native box sum fits, but their native taxable aggregate does not.
    let mut too_wide = saved;
    let mut second = too_wide
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::LtcBenefits { .. }))
        .unwrap()
        .clone();
    let FederalIncome::LtcBenefits { data } = &mut second.income else {
        unreachable!()
    };
    data.benefits = i64::MAX / 2 + 1;
    data.accelerated = 0;
    too_wide
        .income
        .retain(|e| !matches!(e.income, FederalIncome::LtcBenefits { .. }));
    second.id = 2;
    too_wide.income.push(second.clone());
    second.id = 3;
    too_wide.income.push(second);
    assert!(crate::care_benefits::native_receipts_issue(&too_wide).is_some());
    let output = estimate(&too_wide);
    assert!(output.tax.is_none() && output.return_lines.is_empty());
}

#[test]
fn unit47_ltc_native_ratios_keep_wide_known_payment_and_funding_pools() {
    let mut period = crate::tests::support::builders::ltc_period(
        0,
        CareBenefitRecipient::Other {
            insured_received: 0,
            others_received: i64::MAX,
        },
    );
    period.starts = 20260101;
    period.ends = 20260101;
    period.services = 0;
    period.reimbursements = 0;
    period.received = i64::MAX;
    assert_eq!(
        period.amounts(),
        (43_000, 21_500, 21_500, i64::MAX - 21_500)
    );
    let funding = CareBenefitFunding::Individual {
        premiums: CareFundingAmounts {
            employee_premium: i64::MAX,
            employer_premium: i64::MAX,
        },
    };
    assert!(funding.valid());
    assert_eq!(funding.employer_share(101), Some(51));
    let treatment = NonqualifiedCareTreatment::AccidentHealth {
        funding,
        medical_reimbursement: i64::MAX,
        permanent_injury: Some(CarePermanentInjury {
            amount: 1,
            kind: CareInjuryKind::PermanentMemberLoss,
            computed_by_injury: true,
        }),
        recovery: None,
    };
    assert_eq!(treatment.accident_health(i64::MAX), None);
}

#[test]
fn unit47_ltc_native_payee_cohort_keeps_all_actual_sources_and_large_parts() {
    let draft = crate::tests::support::builders::ltc_native_payee_cohort(
        201,
        100_000_000_000,
        100_000_000_000,
    );
    assert!(draft.valid_stored());
    assert!(crate::injured_spouse_sources::reference_issue(&draft).is_none());
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_topic(&mut session, TaxSection::OtherIncome);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(
        value(&supported(&saved), "taxable_ltc_benefits"),
        401_999_999_570
    );
    let mut cold = restart(saved.clone());
    let mut request = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, request.clone()));
    assert_eq!(cold.draft, saved);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let IncomeDocument::LtcInsured(record) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let LtcPeriodsInput::Several { periods, .. } = &mut record.periods else {
        unreachable!()
    };
    let Some(InjuredLtcPeriodPayeesInput::Payments { payments }) = &mut periods[0].injured_payees
    else {
        unreachable!()
    };
    assert_eq!(payments.len(), 201);
    payments[200].source = payments[0].source;
    rejected_with(send(&mut cold, request), "INJURED_LTC_SOURCE_TWICE");
    assert_eq!(cold.draft, saved);
    let too_wide = InjuredLtcPeriodPayees::Payments {
        payments: vec![
            InjuredLtcPeriodPayment {
                source: 1,
                amount: i64::MAX,
            },
            InjuredLtcPeriodPayment {
                source: 2,
                amount: 1,
            },
        ],
    };
    assert_eq!(
        too_wide.issue(i64::MAX).unwrap().key,
        "INJURED_LTC_PAYEE_TOTAL"
    );
}
