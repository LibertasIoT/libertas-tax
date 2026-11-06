extern crate std;
use super::*;
use crate::accepted_sections::AcceptedSectionsError;
use crate::tests::support::builders::{allocation, complete, hsa_prior_basis_funding};
use libertas::data::{KeyCodec, KeyDecode};

#[test]
fn accepted_sections_preserve_absent_empty_and_native_facts_through_sdk_codecs() {
    let absent = FederalDraft::empty("absent".into());
    let mut empty = complete(FederalStatus::Joint);
    empty.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    empty.injured_spouse = Some(allocation());
    let mut rich = hsa_prior_basis_funding(123, 456, 789, 987, 12);
    rich.tax_year = 2041;
    rich.next_id = i64::MAX - 1;
    rich.cookie = " raw cookie \0 ".into();
    rich.revision = 981;
    rich.finished = true;
    rich.finished_sections.reverse();
    let adjustments = rich.adjustments.as_mut().unwrap();
    adjustments.ira_basis[0].adjustment = -12_345;
    adjustments.hsa[0].funding[0].income = i64::MAX - 10;
    rich.deductions.as_mut().unwrap().cash_charity = 10_001;
    rich.credits.as_mut().unwrap().residential_energy_carry = Some(20_003);
    let screening = rich.screening.as_mut().unwrap();
    screening.taxable_premium_election = None;
    screening.fbar_required = Some(false);
    screening.fbar_countries = vec![i64::MAX - 4, 9, 3];
    rich.injured_spouse = Some(allocation());
    let injured = rich.injured_spouse.as_mut().unwrap();
    injured.allocations = vec![
        InjuredAmount { component: " raw component \0 ".into(), injured: -2_007 },
        InjuredAmount { component: "second".into(), injured: 31 },
    ];
    injured.claimants =
        vec![InjuredClaimantFacts { person: i64::MAX - 2, claimant: Owner::Spouse }];
    rich.often_missed = OftenMissed {
        income: MissedIncome::JURY.0 | MissedIncome::MARKETPLACE.0,
        unprepared: vec![
            RareSituation::Straddles.into(),
            RareSituation::RemicResidual.into(),
            Unprepared::OtherAdjustments,
            Unprepared::Section965,
            Unprepared::HousingCreditRecapture,
        ],
        benefits: MissedBenefits::EDUCATOR.0,
    };
    rich.credit_refusals = vec![MoreCreditRefusal::OtherCredit];
    rich.without_form = Some(IncomeWithoutFormAnswers {
        self_work: true,
        tips: true,
        other: Some(WithoutFormOther { amount: 10_003, description: "Other income".into() }),
    });
    rich.coverdell_excess = vec![CoverdellExcess {
        beneficiary: Owner::Spouse,
        contributors: vec![],
        corrections: vec![],
        prior_excess: 10_003,
        prior_tax: 601,
        distributions: 9_007,
        value: 91_001,
        special_needs: true,
    }];
    let payments = rich.payments.as_mut().unwrap();

    payments.extension = 8_009;
    payments.estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: vec![
            DatedPayment {
                date: 20260915,
                amount: 20_005,
                payer: PaymentPayer::Spouse,
                refund_property: None,
            },
            DatedPayment {
                date: 20260415,
                amount: 10_003,
                payer: PaymentPayer::Taxpayer,
                refund_property: Some(RefundSourcePropertyFacts::Separate {
                    owner: Owner::Taxpayer,
                }),
            },
        ],
    };
    payments.waiver = Some(PenaltyWaiver::Partial {
        reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
        explanation: " raw explanation \0 ".into(),
        amount: 107,
    });
    let mut mixed = rich.clone();
    mixed.deductions = None;
    mixed.screening = None;
    mixed.payments = None;
    for original in [absent, empty, rich, mixed] {
        for copy in [0, i64::MAX] {
            let user = i64::MAX;
            let mut records = original.disassemble_accepted_sections(user, copy).unwrap();
            assert_eq!(records.len(), 9);
            for (index, (key, value)) in records.iter_mut().enumerate() {
                assert_eq!(
                    *key,
                    TaxDataKey::ReturnRecord {
                        user,
                        tax_year: original.tax_year,
                        copy,
                        kind: 8 + index as u16,
                        id: 0,
                    }
                );
                let bytes = key.to_key_bytes().unwrap();
                assert!(bytes.starts_with(
                    &TaxDataKey::return_record_prefix(user, original.tax_year).unwrap()
                ));
                *key = TaxDataKey::decode_key(&bytes).unwrap();
                *value = TaxAppData::from_avro(&value.to_avro()).unwrap();
            }
            records.reverse(); // Accepted record scan order is irrelevant.
            let mut destination = original.clone();
            destination.adjustments = None;
            destination.deductions = None;
            destination.credits = None;
            destination.screening = None;
            destination.injured_spouse = None;
            destination.often_missed = OftenMissed::default();
            destination.credit_refusals.clear();
            destination.without_form = None;
            destination.coverdell_excess.clear();
            destination.payments = None;
            destination.assemble_accepted_sections(user, copy, &records).unwrap();
            assert_eq!(destination, original);
        }
    }
}

#[test]
fn assembly_changes_only_the_owned_fields() {
    let mut source = complete(FederalStatus::Joint);
    source.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    source.injured_spouse = Some(allocation());
    let records = source.disassemble_accepted_sections(7, 0).unwrap();
    let mut destination = FederalDraft::empty("different destination".into());
    destination.next_id = i64::MAX;
    destination.revision = 19;
    destination.finished = true;
    let mut expected = destination.clone();
    expected.adjustments = source.adjustments;
    expected.deductions = source.deductions;
    expected.credits = source.credits;
    expected.screening = source.screening;
    expected.injured_spouse = source.injured_spouse;
    expected.often_missed = source.often_missed;
    expected.credit_refusals = source.credit_refusals;
    expected.without_form = source.without_form;
    expected.coverdell_excess = source.coverdell_excess;
    expected.payments = source.payments;
    destination.assemble_accepted_sections(7, 0, &records).unwrap();
    assert_eq!(destination, expected);
}

#[test]
fn malformed_record_sets_and_namespaces_never_partially_change_a_draft() {
    let source = complete(FederalStatus::Joint);
    let records = source.disassemble_accepted_sections(7, 8).unwrap();
    let destination = FederalDraft::empty("unchanged".into());
    let mut cases = vec![];
    for removed in 0..8 {
        let mut bad = records.clone();
        bad.remove(removed);
        cases.push(bad);
    }
    let mut extra = records.clone();
    extra.push(records[0].clone());
    cases.push(extra);
    for index in 0..8 {
        let mut duplicate = records.clone();
        duplicate[index] = records[(index + 1) % 8].clone();
        cases.push(duplicate);
        let mut wrong_payload = records.clone();
        wrong_payload[index].1 = TaxAppData::StorageFormat { format: STORAGE_FORMAT };
        cases.push(wrong_payload);
        for key in [
            TaxDataKey::StorageFormat { user: 7 },
            TaxDataKey::ReturnRecord {
                user: 9,
                tax_year: source.tax_year,
                copy: 8,
                kind: 8 + index as u16,
                id: 0,
            },
            TaxDataKey::ReturnRecord {
                user: 7,
                tax_year: source.tax_year + 1,
                copy: 8,
                kind: 8 + index as u16,
                id: 0,
            },
            TaxDataKey::ReturnRecord {
                user: 7,
                tax_year: source.tax_year,
                copy: 9,
                kind: 8 + index as u16,
                id: 0,
            },
            TaxDataKey::ReturnRecord {
                user: 7,
                tax_year: source.tax_year,
                copy: 8,
                kind: 99,
                id: 0,
            },
            TaxDataKey::ReturnRecord {
                user: 7,
                tax_year: source.tax_year,
                copy: 8,
                kind: 8 + index as u16,
                id: 1,
            },
        ] {
            let mut bad = records.clone();
            bad[index].0 = key;
            cases.push(bad);
        }
    }
    for bad in cases {
        let mut actual = destination.clone();
        assert!(actual.assemble_accepted_sections(7, 8, &bad).is_err());
        assert_eq!(actual, destination);
    }
    for (user, copy) in [(0, 8), (-1, 8), (7, -1)] {
        assert_eq!(
            source.disassemble_accepted_sections(user, copy),
            Err(AcceptedSectionsError::Namespace)
        );
        let mut actual = destination.clone();
        assert_eq!(
            actual.assemble_accepted_sections(user, copy, &records),
            Err(AcceptedSectionsError::Namespace)
        );
        assert_eq!(actual, destination);
    }
}

#[test]
fn payment_record_measures_actual_root_overhead_and_platform_value_boundary() {
    use libertas::data::{EncodedKey, wire};
    // Measure the wrapper with the same Avro string-length prefix as the
    // boundary, without keeping a multi-megabyte source and its copies alive.
    let mut draft = complete(FederalStatus::Single);
    draft.payments.as_mut().unwrap().waiver = Some(PenaltyWaiver::Full {
        reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
        explanation: "x".repeat(1 << 20),
    });
    let native_bytes = draft.payments.as_ref().unwrap().to_avro().len();
    let records = draft.disassemble_accepted_sections(7, 0).unwrap();
    let (key, value) = &records[7];
    let key = EncodedKey::new(key).unwrap().as_bytes().to_vec();
    let wrapper_bytes = value.to_avro().len();
    assert!(wrapper_bytes > native_bytes);
    let overhead = wrapper_bytes - (1 << 20);
    let mut payments = draft.payments.take().unwrap();
    drop(records);
    drop(draft);
    let Some(PenaltyWaiver::Full { explanation, .. }) = &mut payments.waiver else {
        unreachable!()
    };
    *explanation = "x".repeat(wire::MAX_VALUE_BYTES + 1);
    assert!(payments.to_avro().len() > wire::MAX_VALUE_BYTES);
    let Some(PenaltyWaiver::Full { explanation, .. }) = &mut payments.waiver else {
        unreachable!()
    };
    explanation.clear();
    explanation.shrink_to_fit();
    for extra in [0, 1] {
        let target = wire::MAX_VALUE_BYTES + extra;
        let Some(PenaltyWaiver::Full { explanation, .. }) = &mut payments.waiver else {
            unreachable!()
        };
        *explanation = "x".repeat(target - overhead);
        let section = TaxAppData::PaymentsSection { payments: Some(payments) };
        let mut value = section.to_avro();
        value.shrink_to_fit();
        let TaxAppData::PaymentsSection { payments: Some(mut recovered) } = section else {
            unreachable!()
        };
        let Some(PenaltyWaiver::Full { explanation, .. }) = &mut recovered.waiver else {
            unreachable!()
        };
        explanation.clear();
        explanation.shrink_to_fit();
        payments = recovered;
        assert_eq!(value.len(), target);
        if extra == 1 {
            // The root wrapper is part of the platform value boundary.
            assert!(value.len() > wire::MAX_VALUE_BYTES);
        }
        let request = wire::Request {
            version: wire::VERSION,
            id: 1,
            checked: true,
            command: wire::Command::ExecuteInTransaction {
                transaction: 1,
                operation: wire::Operation::Put { key: key.clone(), value },
            },
        };
        let encoded = wire::encode(&request);
        drop(request);
        let decoded = wire::decode_request(&encoded);
        if extra == 0 {
            assert!(decoded.is_ok());
        } else {
            assert_eq!(decoded.unwrap_err(), "Storage byte length exceeds limit");
        }
    }
    std::println!(
        "Unit53 actual payments root: text={} native={native_bytes} wrapper={wrapper_bytes} root_option_overhead={}",
        1 << 20,
        wrapper_bytes - native_bytes
    );
}
