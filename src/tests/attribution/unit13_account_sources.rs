// Final native account sources; all expectations below are independent cents
// arithmetic. Recipient ownership is never inferred from the student's identity.
use super::*;
use crate::tests::cross::interview::{accepted, document};
use crate::tests::family::education_accounts::{
    coverdell, entered, expense_record, q1099_form, qtp, student_row, withdrawal,
};
use crate::tests::support::client::income_pages::{open_document_center, save_documents};

fn education(d: &FederalDraft) -> (crate::education_account::EducationTotals, FederalResult) {
    let mut r = FederalResult::empty(d.revision);
    let qtp_excess = crate::ira::qtp_excess_by_entry(d);
    let dependent_ids: Vec<_> = d.dependents.iter().map(|v| v.id).collect();
    let cx = crate::education_account::EducationContext {
        income: &d.income,
        coverdell: &d.coverdell_excess,
        credits: d.credits.as_ref().unwrap(),
        students: &[],
        status: d.setup.as_ref().unwrap().filing_status(),
        dependent_ids: &dependent_ids,
        credit_allowed: true,
        qtp_excess: &qtp_excess,
    };
    (crate::education_account::calculate(&cx, &mut r), r)
}
fn source_values(v: &crate::education_account::EducationTotals) -> Vec<(i64, Owner, i64, i64)> {
    let mut values: Vec<_> =
        v.sources.iter().map(|s| (s.id, s.owner, s.taxable, s.exceptions)).collect();
    values.sort_by_key(|s| s.0);
    values
}
fn conserve(v: &crate::education_account::EducationTotals) {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mine: Vec<_> = v.sources.iter().filter(|s| s.owner == owner).collect();
        assert_eq!(
            mine.iter().map(|s| s.taxable).sum::<i64>() + v.correction_income[owner as usize],
            v.income[owner as usize]
        );
        assert_eq!(
            mine.iter().map(|s| s.exceptions).sum::<i64>() + v.correction_income[owner as usize],
            v.exceptions[owner as usize]
        );
        assert!(mine.iter().all(|s| (0..=s.taxable).contains(&s.exceptions)));
    }
}

#[test]
fn unit13_account_sources_education_shared_student_keeps_each_recipient_and_native_cents() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    let student = d.dependents[0].id;
    for (owner, gross, earnings) in
        [(Owner::Taxpayer, 200_000, 100_001), (Owner::Spouse, 400_000, 200_003)]
    {
        let mut entry = q1099_form(
            owner,
            gross / 100,
            qtp(0, gross / 100),
            withdrawal(),
            FederalEducationBeneficiary::Student { person: student },
        );
        let FederalIncome::EducationDistribution { data } = &mut entry.income else {
            unreachable!()
        };
        data.form.earnings = earnings;
        data.form.basis = gross - earnings;
        add(&mut d, entry);
    }
    add(&mut d, expense_record(vec![student_row(student, entered(3000, 0, 0))]));
    let (v, r) = education(&d);
    assert!(r.issues.is_empty());
    // $3000 shared expenses cover half each actual distribution. Excluded
    // earnings round first: 50000.5c ->50001c, 100001.5c ->100002c.
    assert_eq!(
        source_values(&v),
        vec![(3, Owner::Taxpayer, 50_000, 0), (4, Owner::Spouse, 100_001, 0)]
    );
    assert_eq!(v.income, [50_000, 100_001]);
    assert_eq!(v.excluded_earnings, [50_001, 100_002]);
    conserve(&v);
    let joint = supported(&d);
    assert_eq!(value(&joint, "education_account_taxable_earnings"), 1500);
    assert_eq!(value(&joint, "account_additional_tax"), 150);
}

#[test]
fn unit13_account_sources_education_credit_exception_is_actual_final_coordination() {
    let mut d = complete(FederalStatus::Joint);
    add(
        &mut d,
        q1099_form(
            Owner::Spouse,
            4000,
            qtp(1000, 3000),
            withdrawal(),
            FederalEducationBeneficiary::Recipient,
        ),
    );
    add(&mut d, expense_record(vec![student_row(-2, entered(4000, 0, 2000))]));
    let (v, r) = education(&d);
    assert!(r.issues.is_empty());
    // Half of1000 earnings is taxable solely because2000 costs were used
    // by an education credit. All500 is a PartII line6 exception.
    assert_eq!(source_values(&v), vec![(2, Owner::Spouse, 50_000, 50_000)]);
    assert_eq!(v.income, [0, 50_000]);
    assert_eq!(v.exceptions, [0, 50_000]);
    conserve(&v);
    assert_eq!(value(&supported(&d), "account_additional_tax"), 0);
}

#[test]
fn unit13_account_sources_education_zero_loss_and_timed_corrections_are_explicit() {
    let mut d = complete(FederalStatus::Joint);
    for (owner, account, transfer) in [
        (Owner::Taxpayer, qtp(-1, 1001), withdrawal()),
        (Owner::Spouse, qtp(300, 700), Q1099Transfer::TrusteeTransfer),
        (Owner::Spouse, coverdell(300, 700), Q1099Transfer::ExcessLastYear),
        (Owner::Taxpayer, coverdell(300, 700), Q1099Transfer::ExcessThisYear),
    ] {
        add(
            &mut d,
            q1099_form(owner, 1000, account, transfer, FederalEducationBeneficiary::Recipient),
        );
    }
    let (v, _) = education(&d);
    assert_eq!(
        source_values(&v),
        vec![
            (2, Owner::Taxpayer, 0, 0),
            (3, Owner::Spouse, 0, 0),
            (4, Owner::Spouse, 0, 0),
            (5, Owner::Taxpayer, 30_000, 30_000)
        ]
    );
    conserve(&v);
    assert_eq!(v.sources.len(), 4);
}

#[test]
fn unit13_account_sources_unlinked_coverdell_corrections_have_owner_and_no_fake_income_id() {
    let mut d = complete(FederalStatus::Joint);
    add(
        &mut d,
        q1099_form(
            Owner::Spouse,
            1000,
            coverdell(3, 997),
            Q1099Transfer::ExcessThisYear,
            FederalEducationBeneficiary::Recipient,
        ),
    );
    for (owner, earnings, link) in [
        (Owner::Taxpayer, 49, None),
        (Owner::Taxpayer, -99, None),
        (Owner::Spouse, 51, None),
        (Owner::Spouse, 300, Some(2)),
    ] {
        d.coverdell_excess.push(CoverdellExcess {
            beneficiary: owner,
            contributors: vec![],
            corrections: vec![CoverdellCorrection {
                account: "Actual correction".into(),
                principal: 1000,
                withdrawn: 20270501,
                earnings,
                income_document: link,
            }],
            prior_excess: 0,
            prior_tax: 0,
            distributions: 0,
            value: 0,
            special_needs: false,
        });
    }
    let (v, _) = education(&d);
    assert_eq!(v.correction_income, [49, 51]);
    assert_eq!(source_values(&v), vec![(2, Owner::Spouse, 300, 300)]);
    assert_eq!(v.income, [49, 351]);
    conserve(&v);
}

#[test]
fn unit13_account_sources_able_retains_rollover_loss_and_penalty_character() {
    let annual = AbleDistribution {
        gross: 1_000_000,
        earnings: 300_000,
        program_to_program_transfer: false,
        account_terminated: false,
        recipient_not_designated_beneficiary: false,
        treatment: Some(AbleTreatment::Annual {
            rollover: AbleRollover::Qualified { amount: 200_000, earnings: 100_000 },
            expenses: 399_951,
            following_expenses: 49,
        }),
    };
    assert_eq!(
        (annual.amounts().unwrap().taxable, annual.amounts().unwrap().penalized),
        (100_000, 100_000)
    );
    assert_eq!(annual.taxable(), Some(100_000));
    assert!(annual.penalized());
    for (earnings, treatment, taxable, penalized, applies) in [
        (
            -49,
            AbleTreatment::Annual {
                rollover: AbleRollover::None,
                expenses: 0,
                following_expenses: 0,
            },
            0,
            0,
            true,
        ),
        (49, AbleTreatment::Corrective { year: 2026 }, 49, 0, false),
        (-49, AbleTreatment::Corrective { year: 2026 }, 0, 0, false),
        (49, AbleTreatment::Corrective { year: 2025 }, 0, 0, false),
        (49, AbleTreatment::Death, 0, 0, false),
    ] {
        let data = AbleDistribution {
            gross: 1000,
            earnings,
            program_to_program_transfer: false,
            account_terminated: false,
            recipient_not_designated_beneficiary: matches!(
                treatment,
                AbleTreatment::Corrective { .. }
            ),
            treatment: Some(treatment),
        };
        assert_eq!(
            (data.amounts().unwrap().taxable, data.amounts().unwrap().penalized),
            (taxable, penalized)
        );
        assert_eq!(data.taxable(), Some(taxable));
        assert_eq!(data.penalized(), applies);
    }
}

#[test]
fn unit13_account_sources_medical_keeps_account_rate_and_dated_exception_without_rounding() {
    let mut recipient = person();
    recipient.birth_date = 19610615;
    for (account, medicare_prior, medicare_from_spouse) in [
        (MedicalSaAccount::Hsa, None, None),
        (MedicalSaAccount::Archer, None, None),
        (MedicalSaAccount::MedicareAdvantage, None, Some(true)),
        (MedicalSaAccount::MedicareAdvantage, Some(MedicareMsaPrior::None), Some(false)),
    ] {
        let data = crate::SaSource {
            medicare_prior,
            medicare_from_spouse,
            payment: Some(MedicalSaPayment::Own {
                nonmedical: vec![
                    MedicalAccountWithdrawal { date: 20260101, amount: 49 },
                    MedicalAccountWithdrawal { date: 20261201, amount: 200 },
                ],
                medical: 100,
                rollover: None,
                disabled_since: None,
            }),
            ..sa_source(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: 349,
                box2_earnings: None,
                box3_code: Some(1),
                box4_value: None,
                account: account.clone(),
                ..Default::default()
            })
        };
        let amounts = data.record().amounts(&recipient).unwrap();
        let calculation_account = data.calculation_account().unwrap();
        let source = amounts.source(31, Owner::Spouse, calculation_account);
        assert_eq!(
            (source.id, source.owner, source.taxable, source.earnings, source.inherited),
            (
                31,
                Owner::Spouse,
                crate::kernel::ratio::exact::whole(249),
                crate::kernel::ratio::exact::whole(0),
                crate::kernel::ratio::exact::whole(0)
            )
        );
        assert_eq!(&source.account, calculation_account);
        // Medicare has50% additional tax and no age65 exception; HSA/Archer
        // have20% with the actual age boundary. No per-source tax is rounded here.
        assert_eq!(
            source.penalized,
            crate::kernel::ratio::exact::whole(
                if matches!(calculation_account, MedicalSaAccount::MedicareAdvantage) {
                    249
                } else {
                    49
                }
            )
        );
    }
}

#[test]
fn unit13_account_sources_medical_inheritance_preserves_native_dollar_precision_and_estate() {
    let mut data = crate::SaSource {
        medicare_prior: if matches!(&MedicalSaAccount::Hsa, MedicalSaAccount::MedicareAdvantage) {
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
        payment: Some(MedicalSaPayment::Inherited {
            died: 20260615,

            beneficiary: MedicalBeneficiary::Nonspouse {
                payments: vec![InheritedMedicalPayment {
                    incurred: 20260101,
                    paid: 20260701,
                    amount: 151,
                }],
            },
        }),
        ..sa_source(Sa1099 {
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: 10_100,
            box2_earnings: None,
            box3_code: Some(4),
            box4_value: Some(10_049),
            account: MedicalSaAccount::Hsa,
            ..Default::default()
        })
    };
    let amounts = data.record().amounts(&person()).unwrap();
    let source = amounts.source(32, Owner::Taxpayer, data.account());
    // Native inherited value100 dollars minus rounded bills2 dollars =98;
    // source preserves9800c, rather than the raw10049-151=9898c.
    assert_eq!(
        (amounts.inherited, source.inherited, source.earnings),
        (
            crate::kernel::ratio::exact::whole(98),
            crate::kernel::ratio::exact::whole(9800),
            crate::kernel::ratio::exact::whole(51)
        )
    );
    let Some(MedicalSaPayment::Inherited { died, beneficiary, .. }) = &mut data.payment else {
        unreachable!()
    };
    *beneficiary = MedicalBeneficiary::Estate;
    let mut deceased = person();
    deceased.died =
        Some(FinalReturn { date: *died, bonds: None, estate_medical: 0, ordinary: true });
    let source =
        data.record().amounts(&deceased).unwrap().source(33, Owner::Spouse, data.account());
    assert_eq!(
        (source.inherited, source.earnings, source.penalized),
        (
            crate::kernel::ratio::exact::whole(10_000),
            crate::kernel::ratio::exact::whole(0),
            crate::kernel::ratio::exact::whole(0)
        )
    );
    assert!(data.record().amounts(&person()).is_err());
}

#[test]
fn unit13_account_sources_final_medical_correction_projection_does_not_reintroduce_linked_earnings()
{
    let data = crate::SaSource {
        medicare_prior: if matches!(&MedicalSaAccount::Archer, MedicalSaAccount::MedicareAdvantage)
        {
            Some(MedicareMsaPrior::None)
        } else {
            None
        },
        medicare_from_spouse: if matches!(
            &MedicalSaAccount::Archer,
            MedicalSaAccount::MedicareAdvantage
        ) {
            Some(false)
        } else {
            None
        },
        payment: Some(MedicalSaPayment::ExcessReturned),
        ..sa_source(Sa1099 {
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: 1000,
            box2_earnings: Some(149),
            box3_code: Some(2),
            box4_value: None,
            account: MedicalSaAccount::Archer,
            ..Default::default()
        })
    };
    let mut amounts = data.record().amounts(&person()).unwrap();
    assert_eq!(
        amounts.source(34, Owner::Spouse, data.account()).earnings,
        crate::kernel::ratio::exact::whole(149)
    );
    // This is the existing collector's completed linked-correction operation;
    // the projection sees its final value, rather than recalculating the form.
    amounts.earnings = crate::kernel::ratio::exact::whole(0);
    assert_eq!(
        amounts.source(34, Owner::Spouse, data.account()).earnings,
        crate::kernel::ratio::exact::whole(0)
    );
}

fn other_account_sources(d: &FederalDraft) -> Vec<(i64, Owner, i64, i64, i64, i64)> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Able { data } => {
                let v = AbleDistribution::from(data).amounts().unwrap();
                Some((e.id, e.owner.owned(), v.taxable, v.penalized, 0, 0))
            }
            FederalIncome::MedicalDistribution { data } => {
                let p = d.people.as_ref().unwrap();
                let recipient = match e.owner.owned() {
                    Owner::Taxpayer => &p.taxpayer,
                    Owner::Spouse => p.spouse.as_ref().unwrap(),
                };
                let v =
                    data.record().amounts(recipient).unwrap().source(e.id, e.owner.owned(), data.account());
                Some((
                    v.id,
                    v.owner,
                    crate::kernel::ratio::exact::rounded(&v.taxable).unwrap(),
                    crate::kernel::ratio::exact::rounded(&v.penalized).unwrap(),
                    crate::kernel::ratio::exact::rounded(&v.earnings).unwrap(),
                    crate::kernel::ratio::exact::rounded(&v.inherited).unwrap(),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn unit13_account_sources_public_save_reopen_restart_and_correction_keep_actual_recipients() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut rows = vec![];
    for (owner, doc) in [
        (
            Owner::Taxpayer,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Q1099(education_distribution_document(
                    EducationDistributionSource {
                        form: q1099_paper(Q1099 {
                            payer_trustee: Default::default(),
                            recipient: Default::default(),
                            recipient_not_beneficiary: false,
                            gross: 400_000,
                            account: qtp(1000, 3000),
                            transfer: withdrawal(),
                        }),
                        coverdell_basis_start: None,
                        coverdell_contributions: 0,
                        redeposited: 0,
                        taxable_transfer_part: 0,
                        roth_rollover: None,
                        late_rollovers: Some(0),
                        beneficiary: Some(FederalEducationBeneficiary::Recipient),
                    },
                )),
            },
        ),
        (
            Owner::Spouse,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(crate::AbleSource {
                    form: crate::Form1099QaPaper {
                        payer: Default::default(),
                        recipient: Default::default(),
                        gross: 100_000,
                        earnings: 30_049,
                        basis: 69_951,
                        program_to_program_transfer: false,
                        account_terminated: false,
                        recipient_not_designated_beneficiary: false,
                        ..Default::default()
                    },
                    treatment: Some(AbleTreatment::Annual {
                        rollover: AbleRollover::None,
                        expenses: 50_000,
                        following_expenses: 0,
                    }),
                })),
            },
        ),
        (
            Owner::Spouse,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(crate::SaSource {
                        medicare_prior: if matches!(
                            &MedicalSaAccount::Archer,
                            MedicalSaAccount::MedicareAdvantage
                        ) {
                            Some(MedicareMsaPrior::None)
                        } else {
                            None
                        },
                        medicare_from_spouse: if matches!(
                            &MedicalSaAccount::Archer,
                            MedicalSaAccount::MedicareAdvantage
                        ) {
                            Some(false)
                        } else {
                            None
                        },
                        payment: Some(MedicalSaPayment::Own {
                            nonmedical: vec![MedicalAccountWithdrawal {
                                date: 20260601,
                                amount: 149,
                            }],
                            medical: 0,
                            rollover: None,
                            disabled_since: None,
                        }),
                        ..sa_source(Sa1099 {
                            trustee_payer: Default::default(),
                            recipient: Default::default(),
                            origin: HealthDocumentOrigin::Form1099Sa,
                            gross: 149,
                            box2_earnings: None,
                            box3_code: Some(1),
                            box4_value: None,
                            account: MedicalSaAccount::Archer,
                        })
                    }),
                ),
            },
        ),
    ] {
        let mut row = document(0, doc);
        row.owner = crate::DocumentOwner::from(owner);
        rows.push(row);
    }
    let desired = rows.clone();
    let request = save_documents(&mut session, rows);
    let request = fixture_owner_inputs(&session.draft, request);
    accepted(&send(&mut session, request));
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &desired,
    );
    let saved = session.draft.clone();
    let initial = source_values(&education(&saved).0);
    let other = other_account_sources(&saved);
    assert_eq!(
        other,
        vec![(3, Owner::Spouse, 15_024, 15_024, 0, 0), (4, Owner::Spouse, 149, 149, 0, 0)]
    );
    assert_eq!(initial, vec![(2, Owner::Taxpayer, 100_000, 0)]);
    assert_eq!(value(&supported(&saved), "agi"), 101_151);
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    assert_eq!(other_account_sources(&session.draft), other);
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    let rows = &mut documents.other.other_income.documents;
    let row = rows
        .iter_mut()
        .find(|r| matches!(r.document, crate::OtherIncomeDocument::Q1099(_)))
        .unwrap();
    row.owner = crate::DocumentOwner::Spouses;
    let crate::OtherIncomeDocument::Q1099(data) = &mut row.document else { unreachable!() };
    data.distribution_code = Some(crate::Form1099QDistributionCode::Disability);
    let page = fixture_owner_inputs(&session.draft, page);
    accepted(&send(&mut session, page));
    let corrected = session.draft.clone();
    assert_eq!(source_values(&education(&corrected).0), vec![(2, Owner::Spouse, 100_000, 100_000)]);
    assert_eq!(value(&supported(&corrected), "agi"), 101_151);
    session = restart(corrected.clone());
    let page = open_document_center(&mut session);
    let page = fixture_owner_inputs(&session.draft, page);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, corrected.income);
    conserve(&education(&session.draft).0);
    assert_eq!(other_account_sources(&session.draft), other);
}
