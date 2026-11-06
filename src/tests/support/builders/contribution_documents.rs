use crate::tests::*;

pub(in crate::tests) fn contribution_statement(
    draft: &FederalDraft,
    kind: HealthContributionKind,
    account: &str,
) -> Form5498Sa {
    let printed = fixture_document_names(draft, -1).unwrap();
    Form5498Sa {
        id: 0,
        participant: TaxPayerIdentity {
            name: printed.recipient.name.clone(),
            federal_id: "987-65-4320".into(),
            address: "1 Example Street".into(),
        },
        assigned: crate::DocumentRecipient::Unknown,
        calendar_year: 2026,
        void: false,
        corrected: false,
        trustee: TaxPayerIdentity {
            name: "Synthetic HSA Trustee".into(),
            federal_id: "00-0000001".into(),
            address: "2 Example Street".into(),
        },
        trustee_phone: "202-555-0100".into(),
        trustee_room: "3".into(),
        trustee_city: "Example City".into(),
        trustee_state: "NY".into(),
        trustee_country: "US".into(),
        trustee_postal_code: "10001".into(),
        participant_apartment: "4".into(),
        participant_city: "Example City".into(),
        participant_state: "NY".into(),
        participant_country: "US".into(),
        participant_postal_code: "10002".into(),
        account_number: account.into(),
        box_1: 0,
        box_2: 0,
        box_3: 0,
        box_4: 0,
        box_5: 0,
        kind,
    }
}

pub(in crate::tests) fn contribution_allocation(
    source: i64,
    prior: i64,
    employer_calendar: i64,
    employer_later: i64,
) -> HealthContributionAllocation {
    HealthContributionAllocation {
        source,
        calendar_prior_year: SourceCents::Reported { amount: prior },
        employer_calendar: SourceCents::Reported { amount: employer_calendar },
        employer_later: SourceCents::Reported { amount: employer_later },
        later_correction_personal: SourceCents::Reported { amount: 0 },
        later_correction_employer: SourceCents::Reported { amount: 0 },
        w2: vec![],
        funding: vec![],
        failed_deposits: vec![],
    }
}

pub(in crate::tests) fn contribution_funding_return() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    current_hsa_funding(&mut draft, 250);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.other_box12.push(W2CodeAmount { code: W2Code::W, amount: 100_000 });
    drop(data);
    health_w2_year(&mut draft, HealthEmployerYear::CurrentYear);
    let employer = crate::health_employer::effective_total(
        &draft,
        crate::health_employer::bindings(&draft),
        Owner::Taxpayer,
        HealthEmployerKind::Hsa,
        0,
    )
    .unwrap();
    let account = &mut draft.adjustments.as_mut().unwrap().hsa[0];
    account.employer = employer;
    account.personal = 20_000;
    account.personal_outside = 20_000;
    draft
}

pub(in crate::tests) fn attempted_hsa_rollover(
    received: u32,
    deposits: Vec<MedicalRolloverDeposit>,
) -> IncomeSource {
    IncomeSource::OtherIncome {
        value: crate::OtherIncomeDocument::Sa1099(
            crate::tests::support::client::income_pages::sa_document(
                attempted_hsa_rollover_source(received, deposits),
            ),
        ),
    }
}
pub(in crate::tests) fn attempted_hsa_rollover_source(
    received: u32,
    deposits: Vec<MedicalRolloverDeposit>,
) -> crate::SaSource {
    {
        let mut paper_source = sa_source(Sa1099 {
            trustee_payer: Default::default(),
            recipient: Default::default(),
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: deposits.iter().map(|p| p.amount).sum(),
            box2_earnings: None,
            box3_code: Some(1),
            box4_value: None,
            account: MedicalSaAccount::Hsa,
        });
        // The paper carries no answers, so two attempts are told apart by their printed account number.
        paper_source.form1099sa_mut().unwrap().account_number =
            alloc::format!("ROLLOVER-{received}");
        crate::SaSource {
            medicare_prior: if matches!(&MedicalSaAccount::Hsa, MedicalSaAccount::MedicareAdvantage)
            {
                Some(MedicareMsaPrior::None)
            } else {
                None
            },
            medicare_from_spouse: if matches!(
                &MedicalSaAccount::Hsa,
                MedicalSaAccount::MedicareAdvantage
            ) {
                Some(false)
            } else {
                None
            },
            payment: Some(MedicalSaPayment::Own {
                nonmedical: vec![],
                medical: 0,
                disabled_since: None,
                rollover: Some(MedicalRollover {
                    preceding: None,
                    receipts: vec![MedicalRolloverReceipt { received, deposits }],
                }),
            }),
            ..paper_source
        }
    }
}
pub(in crate::tests) fn ordinary_hsa_deposit(
    deposited: u32,
    amount: i64,
    year: u32,
) -> MedicalRolloverDeposit {
    MedicalRolloverDeposit {
        deposited,
        amount: amount * 100,
        destination: MedicalRolloverDestination::Hsa,
        character: MedicalDepositCharacter::Cash { year, deadline: (year + 1) * 10000 + 415 },
    }
}
