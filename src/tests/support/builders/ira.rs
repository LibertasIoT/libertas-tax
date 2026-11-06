use crate::tests::*;

pub(in crate::tests) fn saver(you: i64, spouse: i64) -> FederalSaver {
    FederalSaver {
        taxpayer_able: 0,
        spouse_able: 0,
        taxpayer_contributions: you,
        spouse_contributions: spouse,
        distributions: vec![],
    }
}

pub(in crate::tests) fn ira_facts(traditional: i64, roth: i64, covered: Answer) -> FederalIras {
    FederalIras {
        taxpayer: FederalIraContribution {
            nondeductible_election: 0,
            traditional,
            roth,
            covered_at_work: covered == Answer::Yes,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
        spouse: IraSpouse::NotApplicable,
    }
}

// A 529-to-Roth IRA transfer as the Form 1099-Q it is (box 4b): the facts the
// form does not show, with its box 3 (contributions) and box 2 (earnings).
pub(in crate::tests) fn qtp_roth_rollover() -> crate::ira::QtpTransfer {
    crate::ira::QtpTransfer {
        id: 1,
        rollover: QtpRothRollover {
            plan_state: None,
            account_reference: "PLAN-A".into(),
            account_opened_on: 20110601,
            transferred_on: 20260601,
            seasoned_available: 600_000,
            confirmed: true,
        },
        contributions: 400_000,
        earnings: 200_000,
    }
}

pub(in crate::tests) fn two_qtp_roth_rollovers() -> Vec<crate::ira::QtpTransfer> {
    let mut first = qtp_roth_rollover();
    first.rollover.account_opened_on = 20100101;
    first.rollover.transferred_on = 20260301;
    first.contributions = 200_000;
    first.earnings = 100_000;
    first.rollover.seasoned_available = 300_000;
    let mut second = qtp_roth_rollover();
    second.rollover.account_reference = "PLAN-B".into();
    second.rollover.account_opened_on = 20110602;
    second.rollover.transferred_on = 20260602;
    second.contributions = 150_000;
    second.earnings = 150_000;
    second.rollover.seasoned_available = 300_000;
    vec![first, second]
}

// The Form 1099-Q document of a 529-to-Roth transfer.
pub(in crate::tests) fn qtp_roth_form(
    owner: Owner,
    transfer: &crate::ira::QtpTransfer,
) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::EducationDistribution {
        data: education_distribution_source(FederalEducationDistribution {
            recipient_not_beneficiary: false,
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: Some(transfer.rollover.clone()),
            late_rollovers: None,
            gross: transfer.amount(),
            account: EducationAccount::Qtp {
                earnings: transfer.earnings,
                basis: transfer.contributions,
            },
            transfer: Q1099Transfer::RothIra,
            beneficiary: Some(FederalEducationBeneficiary::Recipient),
        }),
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}

// Replaces an owner's 529-to-Roth transfers with these Forms 1099-Q.
pub(in crate::tests) fn set_qtp_roth(
    d: &mut FederalDraft,
    owner: Owner,
    transfers: &[crate::ira::QtpTransfer],
) {
    d.income.retain(|e| {
        e.owner != crate::DocumentOwner::from(owner)
            || !matches!(&e.income, FederalIncome::EducationDistribution { data }
                if matches!(Q1099::from(&data.form).transfer, Q1099Transfer::RothIra))
    });
    for transfer in transfers {
        add(d, qtp_roth_form(owner, transfer));
    }
}

pub(in crate::tests) fn next_year_ira_deposits(
    traditional: i64,
    roth: i64,
    date: u32,
) -> IraNextYearDeposits {
    IraNextYearDeposits { last_deposited_on: date, traditional, roth, designation_confirmed: true }
}

pub(in crate::tests) fn rmd_record(required: i64, timely: i64) -> RequiredDistribution {
    RequiredDistribution {
        owner: Owner::Taxpayer,
        label: "Synthetic plan".into(),
        required: RmdMinimum::Administrator { amount: required * 100 },
        timely: timely * 100,
        correction: RmdCorrection::Uncorrected,
        waiver: None,
    }
}
pub(in crate::tests) fn rmd_corrected(received: u32, filed: u32) -> RmdCorrection {
    RmdCorrection::Corrected {
        received,
        filed: RmdFiling::Submitted { date: filed },
        notice: None,
        assessed: None,
    }
}

pub(in crate::tests) fn own_rmd(
    birth: u32,
    period: RmdPeriod,
    balance: i64,
    spouse_birth: Option<u32>,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    let mut entry = rmd_record(0, 0);
    entry.required = RmdMinimum::OwnIras {
        period,
        accounts: vec![RmdIra {
            balance: balance * 100,
            rollover: 0,
            spouse: spouse_birth.map(|birth| RmdSpouse { birth, confirmed: true }),
        }],
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![entry];
    d
}

pub(in crate::tests) fn beneficiary_rmd(
    owner_birth: u32,
    died: u32,
    birth: u32,
    kind: InheritedIraKind,
    balance: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    let mut e = rmd_record(0, 0);
    e.required = RmdMinimum::Inherited {
        data: InheritedRmd {
            kind,
            owner_birth,
            died,
            balance: balance * 100,
            incoming: 0,
            closing: None,
            eligibility: InheritedEligibility::Ordinary,
            deadline_election: false,
            confirmed: true,
        },
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    d
}
pub(in crate::tests) fn inherited_rmd_data(d: &mut FederalDraft) -> &mut InheritedRmd {
    let RmdMinimum::Inherited { data } =
        &mut d.adjustments.as_mut().unwrap().required_distributions[0].required
    else {
        panic!("inherited fixture")
    };
    data
}

// An inherited traditional IRA record (all the IRAs from one decedent, died 2025-12-01), with
// its allocated basis. Its distributions come from the Forms 1099-R that name it
// (`inherited_payment`).
pub(in crate::tests) fn inherited_ira(prior: i64, year_end: i64) -> FederalIncomeEntry {
    income(FederalIncome::InheritedIra {
        data: crate::FederalInheritedIra {
            decedent: "Synthetic decedent".into(),
            died: 20251201,
            kind: crate::InheritedIraKindFacts::Traditional {
                basis: Some(InheritedIraBasis { prior: prior * 100, year_end: year_end * 100 }),
                estate_tax: None,
            },
        },
    })
}
// A Form 1099-R with code 4 and box 7b checked that chooses a record, in dollars.
pub(in crate::tests) fn inherited_payment(
    owner: Owner,
    account: Option<i64>,
    gross: i64,
) -> FederalIncomeEntry {
    let mut e = income(FederalIncome::Retirement {
        data: retirement_source(FederalRetirement {
            form_boxes: Default::default(),
            trump: false,
            box_2a: gross * 100,
            box_3: 0,
            box_6: 0,
            box_8a: 0,
            box_8b_percent: None,
            box_9a_percent: None,
            ira: (RetirementKind::Inherited).ira()
                && (crate::DistributionCodes::CODE_4.0) & crate::DistributionCodes::CODE_N.0 == 0,
            box_9b: None,
            box_10: 0,
            box_10_basis: None,
            disability: DisabilityIncome::None,
            disability_wages: crate::DisabilityWages::None,
            kind: RetirementKind::Inherited,
            gross: gross * 100,
            taxable: Some(RetirementTaxable::Inherited { account }),
            withholding: 12300,
            box7: crate::DistributionCodes::CODE_4.0,
            ordinary_distribution: Some(RetirementTreatment::Ordinary),
            disaster: crate::DisasterDistribution::No,
            after_death: None,
        }),
    });
    e.owner = crate::DocumentOwner::from(owner);
    e
}
// The record and one payment that names it, in dollars. Returns the record's identity.
pub(in crate::tests) fn add_inherited_ira(
    d: &mut FederalDraft,
    gross: i64,
    prior: i64,
    year_end: i64,
) -> i64 {
    let id = d.next_id;
    add(d, inherited_ira(prior, year_end));
    add(d, inherited_payment(Owner::Taxpayer, Some(id), gross));
    id
}
// The record's allocated basis (the traditional kind).
pub(in crate::tests) fn inherited_basis_mut(
    entry: &mut FederalIncomeEntry,
) -> &mut Option<InheritedIraBasis> {
    let FederalIncome::InheritedIra { data } = &mut entry.income else { unreachable!() };
    let crate::InheritedIraKindFacts::Traditional { basis, .. } = &mut data.kind else {
        unreachable!()
    };
    basis
}

pub(in crate::tests) fn inherited_estate(remaining: i64) -> InheritedEstateTax {
    InheritedEstateTax {
        paid: 946000,
        without_ird: 484000,
        total_ird: 2000000,
        remaining_value: remaining * 100,
        confirmed: true,
    }
}
pub(in crate::tests) fn set_inherited_estate(d: &mut FederalDraft, estate: InheritedEstateTax) {
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income
        && let crate::InheritedIraKindFacts::Traditional { estate_tax, .. } = &mut data.kind
    {
        *estate_tax = Some(estate);
    } else {
        unreachable!()
    }
}
