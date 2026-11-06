use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn hsa_entry(
    owner: Owner,
    coverage: HsaCoverage,
    personal: i64,
) -> FederalHsa {
    FederalHsa {
        personal_outside: personal,

        employer_outside: 0,
        owner,
        coverage: HsaEligibility::FullYear { coverage },
        personal,
        employer: 0,
        late_excess: vec![],
        prior_employer_excess: vec![],
        corrections: vec![],
        excess_accounting: None,
        funding: vec![],
        funding_history: vec![],
        uses: vec![],
        last_month_history: None,
        last_month_loss: None,
    }
}

pub(in crate::tests) fn personal_care(person: i64, premiums: i64) -> MedicalCarePremium {
    MedicalCarePremium { person, premiums, qualified: CareContractQualification::Qualified }
}

pub(in crate::tests) fn ltc_period(
    received: i64,
    recipient: CareBenefitRecipient,
) -> CareBenefitPeriod {
    CareBenefitPeriod {
        starts: 20260101,
        ends: 20260131,
        received: received * 100,
        services: 1000000,
        reimbursements: 200000,
        recipient,
        injured_payees: None,
    }
}
// A Form 1099-LTC with per diem payments of a qualified contract, naming `insured`.
pub(in crate::tests) fn ltc_form(per_diem: i64, insured: Option<i64>) -> FederalIncome {
    FederalIncome::LtcBenefits {
        data: LtcSource {
            interest: None,
            form: Ltc1099Paper {
                payer: Default::default(),
                recipient: Default::default(),
                insured_party: Default::default(),
                benefits: Some(per_diem * 100),
                accelerated: Some(0),
                paid: LtcPaidBasis::PerDiem,
                box4_qualified: false,
                illness: Some(LtcIllness::Chronic),
                date_certified: None,
                calendar_year: 2026,
                corrected: false,
                payer_phone: String::new(),
                account_number: String::new(),
            },
            benefit_origin: Some(LtcPaymentOrigin::Insurance),
            accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                business_insurable_interest: false,
            }),
            contract: LtcContract::Qualified {
                qualification: CareContractQualification::Qualified,
            },
            insured,
        },
    }
}
// The Long-term care periods record of the taxpayer with the given periods.
pub(in crate::tests) fn ltc_record(periods: LtcPeriods) -> FederalIncome {
    FederalIncome::LtcInsured {
        data: FederalLtcInsured {
            insured: FederalLtcInsuredPerson::OnReturn { person: -1 },
            periods,
        },
    }
}
// One period of a January: the same $10,000 care costs less $2,000 reimbursed.
pub(in crate::tests) fn ltc_one_period(starts: u32, ends: u32) -> LtcPeriods {
    LtcPeriods::One {
        starts,
        ends,
        services: 1000000,
        reimbursements: 200000,
        recipient: CareBenefitRecipient::Insured { other_payees: false },
    }
}

// A Form 1099-SA of an Archer MSA: $2,000 spent on medical costs and `nonmedical` kept.
pub(in crate::tests) fn archer_msa(nonmedical: i64) -> FederalIncome {
    FederalIncome::MedicalDistribution {
        data: {
            let paper_source = sa_source(Sa1099 {
                origin: crate::HealthDocumentOrigin::Form1099Sa,
                gross: (2000 + nonmedical) * 100,
                box2_earnings: None,
                box3_code: Some(1),
                box4_value: None,
                account: MedicalSaAccount::Archer,
                ..Default::default()
            });
            crate::SaSource {
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
                        date: 20260615,
                        amount: nonmedical * 100,
                    }],
                    medical: 200000,
                    rollover: None,
                    disabled_since: None,
                }),
                ..paper_source
            }
        },
    }
}
// A Form 1099-SA of a Medicare Advantage MSA: $2,000 spent on medical costs.
pub(in crate::tests) fn medicare_msa(nonmedical: i64, prior: MedicareMsaPrior) -> FederalIncome {
    FederalIncome::MedicalDistribution {
        data: {
            let paper_source = sa_source(Sa1099 {
                origin: crate::HealthDocumentOrigin::Form1099Sa,
                gross: (nonmedical + 2000) * 100,
                box2_earnings: None,
                box3_code: Some(1),
                box4_value: None,
                account: MedicalSaAccount::MedicareAdvantage,
                ..Default::default()
            });
            crate::SaSource {
                medicare_prior: Some(prior),
                medicare_from_spouse: Some(false),
                payment: Some(MedicalSaPayment::Own {
                    nonmedical: vec![MedicalAccountWithdrawal {
                        date: 20260615,
                        amount: nonmedical * 100,
                    }],
                    medical: 200000,
                    rollover: None,
                    disabled_since: None,
                }),
                ..paper_source
            }
        },
    }
}
// A Form 1099-SA of an account inherited on 2026-06-15 from a holder: box 4 $50,000, $12,000 of
// the holder's bills paid on 2026-06-16, box 1 the same $50,000.
pub(in crate::tests) fn medical_inheritance(account: MedicalSaAccount) -> FederalIncome {
    FederalIncome::MedicalDistribution {
        data: {
            let paper_source = sa_source(Sa1099 {
                origin: crate::HealthDocumentOrigin::Form1099Sa,
                gross: 5000000,
                box2_earnings: None,
                box3_code: Some(4),
                box4_value: Some(5000000),
                account: account.clone(),
                ..Default::default()
            });
            crate::SaSource {
                medicare_prior: if matches!(&account, MedicalSaAccount::MedicareAdvantage) {
                    Some(MedicareMsaPrior::None)
                } else {
                    None
                },
                medicare_from_spouse: if matches!(&account, MedicalSaAccount::MedicareAdvantage) {
                    Some(false)
                } else {
                    None
                },
                payment: Some(MedicalSaPayment::Inherited {
                    died: 20260615,

                    beneficiary: MedicalBeneficiary::Nonspouse {
                        payments: vec![InheritedMedicalPayment {
                            incurred: 20251231,
                            paid: 20260616,
                            amount: 1200000,
                        }],
                    },
                }),
                ..paper_source
            }
        },
    }
}
pub(in crate::tests) fn archer_rollover() -> MedicalRollover {
    MedicalRollover {
        preceding: None,
        receipts: vec![MedicalRolloverReceipt {
            received: 20260101,
            deposits: vec![MedicalRolloverDeposit {
                deposited: 20260302,
                amount: 500000,
                destination: MedicalRolloverDestination::Archer,
                character: MedicalDepositCharacter::Unconfirmed,
            }],
        }],
    }
}

pub(in crate::tests) fn hsa_rollover() -> MedicalRollover {
    let mut r = archer_rollover();
    r.receipts[0].deposits[0].destination = MedicalRolloverDestination::Hsa;
    r
}

pub(in crate::tests) fn archer_self(deductible: i64) -> ArcherCoverage {
    ArcherCoverage::SelfOnly { deductible: deductible * 100, out_of_pocket: 585000 }
}
pub(in crate::tests) fn archer_family(deductible: i64) -> ArcherCoverage {
    ArcherCoverage::Family { deductible: deductible * 100, out_of_pocket: 1070000 }
}
pub(in crate::tests) fn archer_contributor(
    owner: Owner,
    coverage: ArcherCoverage,
    personal: i64,
) -> ArcherContributor {
    ArcherContributor {
        personal_outside: personal * 100,

        employer_outside: 0,
        owner,
        personal: personal * 100,
        employer: 0,
        corrections: vec![],
        prior_employer_excess: vec![],
        excess_accounting: None,
        coverage: ArcherEligibility::FullYear { coverage },
        income: vec![if owner == Owner::Taxpayer { 1 } else { 2 }],
    }
}
pub(in crate::tests) fn set_archer(
    d: &mut FederalDraft,
    owners: alloc::vec::Vec<ArcherContributor>,
    sharing: ArcherSharing,
) {
    d.adjustments.as_mut().unwrap().archer = ArcherContributions::Records { owners, sharing };
}
pub(in crate::tests) fn archer_owner_mut(d: &mut FederalDraft) -> &mut ArcherContributor {
    let ArcherContributions::Records { owners, .. } = &mut d.adjustments.as_mut().unwrap().archer
    else {
        panic!()
    };
    &mut owners[0]
}

// Build one real canonical source; contributions remain with hsa_entry.
pub(in crate::tests) fn add_hsa_distribution(
    d: &mut FederalDraft,
    owner: Owner,
    medical: i64,
    nonmedical: Vec<MedicalAccountWithdrawal>,
    disabled_since: Option<u32>,
    rollover: Option<MedicalRollover>,
) -> i64 {
    let gross = medical
        + nonmedical.iter().map(|v| v.amount).sum::<i64>()
        + rollover
            .as_ref()
            .map_or(0, |v| crate::kernel::ratio::exact::rounded(&v.amount()).unwrap());
    let mut e = income(FederalIncome::MedicalDistribution {
        data: {
            let paper_source = sa_source(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross,
                box2_earnings: None,
                box3_code: Some(1),
                box4_value: None,
                account: MedicalSaAccount::Hsa,
                ..Default::default()
            });
            crate::SaSource {
                medicare_prior: if matches!(
                    &MedicalSaAccount::Hsa,
                    MedicalSaAccount::MedicareAdvantage
                ) {
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
                    medical,
                    nonmedical,
                    disabled_since,
                    rollover,
                }),
                ..paper_source
            }
        },
    });
    e.owner = crate::DocumentOwner::from(owner);
    add(d, e);
    d.income.last().unwrap().id
}
pub(in crate::tests) fn add_hsa_deemed(
    d: &mut FederalDraft,
    owner: Owner,
    event: MedicalDeemed,
    disabled_since: Option<u32>,
) -> i64 {
    let mut e = income(FederalIncome::MedicalDistribution {
        data: {
            let paper_source = sa_source(Sa1099 {
                origin: HealthDocumentOrigin::TrusteeStatement,
                gross: crate::kernel::ratio::exact::rounded(&event.amount())
                    .expect("synthetic deemed source fits its actual native statement"),
                box2_earnings: None,
                box3_code: None,
                box4_value: None,
                account: MedicalSaAccount::Hsa,
                ..Default::default()
            });
            crate::SaSource {
                medicare_prior: if matches!(
                    &MedicalSaAccount::Hsa,
                    MedicalSaAccount::MedicareAdvantage
                ) {
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
                payment: Some(MedicalSaPayment::Unreported { event, disabled_since }),
                ..paper_source
            }
        },
    });
    e.owner = crate::DocumentOwner::from(owner);
    add(d, e);
    d.income.last().unwrap().id
}
pub(in crate::tests) fn hsa_distribution_mut(
    d: &mut FederalDraft,
    owner: Owner,
    index: usize,
) -> &mut crate::SaSource {
    d.income
        .iter_mut()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .filter_map(|e| match &mut e.income {
            FederalIncome::MedicalDistribution { data }
                if matches!(data.account(), MedicalSaAccount::Hsa) =>
            {
                Some(data)
            }
            _ => None,
        })
        .nth(index)
        .unwrap()
}

pub(in crate::tests) fn hsa_funding(funded: u32, loss: u32, amount: i64) -> HsaFundingHistory {
    HsaFundingHistory {
        funded,
        amount,
        loss: Some(HsaEligibilityLoss { date: loss, reason: HsaLossReason::Other }),
    }
}

pub(in crate::tests) fn inherited_interest(amount: i64) -> FederalIncome {
    FederalIncome::Interest { data: interest_source(native_interest(amount, 0)) }
}

pub(in crate::tests) fn hsa_correction(
    year: u32,
    returned: u32,
    principal: i64,
    earnings: i64,
) -> HsaCorrection {
    HsaCorrection {
        failed_deposit: None,
        document: None,
        year,
        contributed_on: year * 10000 + 101,
        returned,
        principal,
        earnings,
        deadline: (year + 1) * 10000 + 415,
        payer: HsaCorrectionPayer::Personal,
    }
}

pub(in crate::tests) fn last_month_history(
    months: usize,
    contributions: i64,
) -> HsaLastMonthHistory {
    let mut coverage = vec![HsaCoverage::None; 12 - months];
    coverage.extend(vec![HsaCoverage::SelfOnly; months]);
    HsaLastMonthHistory {
        months: coverage,
        contributions,
        archer: 0,
        family: None,
        loss: Some(HsaEligibilityLoss { date: 20260601, reason: HsaLossReason::Other }),
    }
}

pub(in crate::tests) fn december_hsa(coverage: HsaCoverage, amount: i64) -> FederalHsa {
    let mut h = hsa_entry(Owner::Taxpayer, coverage, amount);
    let mut months = vec![HsaCoverage::None; 11];
    months.push(coverage);
    h.coverage = HsaEligibility::Monthly { months };
    h
}

pub(in crate::tests) fn historical_hsa_family(
    history: &mut HsaLastMonthHistory,
    owner_parts: u32,
    spouse_parts: u32,
) {
    history
        .months
        .iter_mut()
        .filter(|v| **v != HsaCoverage::None)
        .for_each(|v| *v = HsaCoverage::Family);
    history.family = Some(HsaFamilyAllocation {
        spouse_months: history.months.clone(),
        spouse_archer: 0,
        owner_parts,
        spouse_parts,
    });
}

pub(in crate::tests) fn joint_december_hsa(
    taxpayer: i64,
    spouse_amount: i64,
    taxpayer_parts: u32,
    spouse_parts: u32,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    let h = december_hsa(HsaCoverage::Family, taxpayer);
    let mut spouse = december_hsa(HsaCoverage::SelfOnly, spouse_amount);
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        other_coverage: None,
        taxpayer_parts,
        spouse_parts,
    });
    d
}

pub(in crate::tests) fn separate_december_hsa(amount: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.adjustments.as_mut().unwrap().hsa = vec![december_hsa(HsaCoverage::SelfOnly, amount)];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::Family }),
    });
    d
}

pub(in crate::tests) fn current_hsa_funding(d: &mut FederalDraft, amount: i64) {
    let e =
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 6000, RetirementTreatment::Early);
    let id = d.next_id;
    add(d, e);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    h.funding = vec![HsaFunding {
        ongoing_sep_simple: false,
        income: id,
        basis: HsaFundingBasis::Pretax { distributed: 20260115 },
        funded: 20260115,
        amount: amount * 100,
        coverage: HsaCoverage::SelfOnly,
        loss: None,
    }];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
}

pub(in crate::tests) fn hsa_prior_basis_funding(
    amount: i64,
    prior: i64,
    before: i64,
    after: i64,
    other: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, amount);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: before * 100,
        before_other_activity: true,
    };
    let id = h.funding[0].income;
    let e = d.income.iter_mut().find(|v| v.id == id).unwrap();
    if let FederalIncome::Retirement { data } = &mut e.income {
        let mut data = edit_retirement(data);
        data.gross = (amount + other) * 100;
        data.taxable = Some(RetirementTaxable::IraBasis);
        drop(data);
    }
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, prior, after)];
    d
}

pub(in crate::tests) fn inherited_hsa_funding(
    gross: i64,
    prior: i64,
    value: i64,
    end: i64,
    amount: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, amount);
    let mut entry = inherited_ira(prior, end);
    entry.id = d.income[1].id;
    d.income[1] = entry;
    let record = d.income[1].id;
    let payment = d.next_id;
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(record), gross));
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].income = payment;
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: value * 100,
        before_other_activity: true,
    };
    d
}

// A real nonmedical-dependent child for derived162(l) fixtures. This source
// identity belongs to People; no age or relationship is stored on a policy.
pub(in crate::tests) fn health_coverage_child(d: &mut FederalDraft, age: u32) -> i64 {
    let id = d.next_id;
    d.next_id += 1;
    let mut person = child(id);
    person.use_for = PersonUse::HealthCoverageOnly;
    person.birth_date = (2026 - age) * 10000 + 102;
    person.situation =
        DependentSituation::Supported { your_support_over_half: false, gross_income: 4_000_000 };
    person.facts = PersonFacts::OWN_SUPPORT.0;
    d.dependents.push(person);
    id
}

// Explicit source designation for synthetic fixtures predating the year-source
// editor. The fixture chooses its actual year; production never infers this.
pub(in crate::tests) fn health_w2_year(d: &mut FederalDraft, year: HealthEmployerYear) {
    let mut rows =
        crate::health_employer::page_rows_current_year(d, crate::health_employer::bindings(d))
            .unwrap();
    for row in &mut rows {
        row.allocation = year.clone();
    }
    let bindings =
        crate::health_employer::accept_page(d, crate::health_employer::bindings(d), &rows).unwrap();
    d.adjustments.as_mut().unwrap().health_employer_w2 = bindings;
}

pub(in crate::tests) fn ltc_native_payee_cohort(
    count: usize,
    benefits: i64,
    accelerated: i64,
) -> FederalDraft {
    let amount = benefits + accelerated;
    let mut draft = complete(FederalStatus::Joint);
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    draft.injured_spouse = Some(crate::tests::support::builders::allocation());
    let insured = draft.next_id;
    let mut first = ltc_period(0, CareBenefitRecipient::Insured { other_payees: false });
    first.starts = 20260101;
    first.ends = 20260101;
    first.services = 0;
    first.reimbursements = 0;
    first.received = amount * count as i64;
    let second =
        CareBenefitPeriod { starts: 20260102, ends: 20260102, received: 0, ..first.clone() };
    add(
        &mut draft,
        income(ltc_record(LtcPeriods::Several {
            method: CarePeriodMethod::EqualRate,
            periods: vec![first, second],
        })),
    );
    let mut payments = Vec::new();
    for _ in 0..count {
        let mut source = ltc_form(1, Some(insured));
        let FederalIncome::LtcBenefits { data } = &mut source else { unreachable!() };
        data.form.benefits = Some(benefits);
        data.form.accelerated = Some(accelerated);
        payments.push(InjuredLtcPeriodPayment { source: draft.next_id, amount });
        add(&mut draft, income(source));
    }
    let FederalIncome::LtcInsured { data } =
        &mut draft.income.iter_mut().find(|entry| entry.id == insured).unwrap().income
    else {
        unreachable!()
    };
    let LtcPeriods::Several { periods, .. } = &mut data.periods else { unreachable!() };
    periods[0].injured_payees = Some(InjuredLtcPeriodPayees::Payments { payments });
    draft
}

// Native cent amounts and the account-specific prior Form 5329 lines remain explicit.
pub(in crate::tests) fn hsa_excess_accounting(
    year_end_value: i64,
    prior_year: Option<HsaPriorExcess>,
    later_return_year_deposits: i64,
    later_employer_deposits: i64,
    employer_excess_in_wages: i64,
) -> HsaExcessAccounting {
    HsaExcessAccounting {
        prior_year,
        year_end_value,
        later_return_year_deposits,
        later_employer_deposits,
        employer_excess_in_wages,
    }
}

pub(in crate::tests) fn archer_excess_accounting(
    year_end_value: i64,
    prior_year: Option<ArcherPriorExcess>,
    later_return_year_deposits: i64,
    later_employer_deposits: i64,
    employer_excess_in_wages: i64,
) -> ArcherExcessAccounting {
    ArcherExcessAccounting {
        prior_year,
        year_end_value,
        later_return_year_deposits,
        later_employer_deposits,
        employer_excess_in_wages,
    }
}

pub(in crate::tests) fn health_account_page(
    accounts: Vec<HsaAccount>,
    funding: Vec<HsaFundingRow>,
    archer: ArcherContributionsPage,
) -> HsaPage {
    HsaPage {
        contribution_sources: vec![],
        employer_statements: vec![],

        employer_sources: vec![],
        linked_deposits: vec![],
        other_coverage: None,
        family_method: HsaAllocationMethod::Monthly,
        accounts,
        family_split: FamilySplit::NotApplicable,
        funding,
        archer,
    }
}

pub(in crate::tests) fn archer_contributor_row(
    owner: Owner,
    personal: i64,
    employer: i64,
    coverage: ArcherEligibility,
    income: Vec<i64>,
) -> ArcherContributorRow {
    ArcherContributorRow {
        owner,
        personal,
        employer,
        coverage,
        income,
        corrections: vec![],
        prior_employer_excess: vec![],
        excess_accounting: None,
    }
}

pub(in crate::tests) fn ltc_identity_fixture() -> (FederalDraft, i64, Ltc1099Paper) {
    let mut draft = complete(FederalStatus::Single);
    let mut record = ltc_record(LtcPeriods::One {
        starts: 20260101,
        ends: 20260131,
        services: 1_000_000,
        reimbursements: 200_000,
        recipient: CareBenefitRecipient::Other { insured_received: 0, others_received: 0 },
    });
    let FederalIncome::LtcInsured { data } = &mut record else { unreachable!() };
    data.insured = crate::FederalLtcInsuredPerson::Other { name: "Taylor Insured Example".into() };
    add(&mut draft, income(record));
    let period_id = draft.income.last().unwrap().id;
    add(&mut draft, income(ltc_form(4_000, Some(period_id))));
    let entry = draft.income.last().unwrap();
    let id = entry.id;
    let FederalIncome::LtcBenefits { data } = &entry.income else { unreachable!() };
    let mut source = data.clone();
    source.form.insured_party = crate::PrintedParty {
        name: "Taylor Insured Example".into(),
        federal_id: "***-**-4321".into(),
        address: "7 Example Street, Apt. 00B\nSample City, NY 00123".into(),
    };
    source.form.date_certified = Some(20240229);
    let paper = ltc_document(source);
    (draft, id, paper)
}
