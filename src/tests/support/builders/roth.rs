use crate::tests::*;

pub(in crate::tests) fn roth_history(owner: Owner, first_year: u32, regular: i64) -> RothHistory {
    RothHistory {
        owner,
        first_year,
        ordering: Some(RothOrdering { regular: regular * 100, conversions: vec![] }),
    }
}
pub(in crate::tests) fn roth_withdrawal(
    owner: Owner,
    amount: i64,
    date: u32,
) -> FederalIncomeEntry {
    let mut e = income(FederalIncome::Roth {
        data: RothWithdrawal {
            sequence: None,
            rolled_over: 0,
            simple_early: false,
            disaster: DisasterDistribution::No,
            exception: None,
            repaid: None,
            distributed: date,
            gross: amount * 100,
            withholding: 12300,
            code: RothCode::Early,
            inherited: InheritedLink::No,
        },
    });
    e.owner = owner;
    e
}

pub(in crate::tests) fn roth_hsa_funding(
    amount: i64,
    regular: i64,
    value: i64,
    other: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().roth_history =
        vec![roth_history(Owner::Taxpayer, 2010, regular)];
    let id = d.next_id;
    add(&mut d, roth_withdrawal(Owner::Taxpayer, amount + other, 20260115));
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    h.funding = vec![HsaFunding {
        ongoing_sep_simple: false,
        income: id,
        funded: 20260116,
        amount: amount * 100,
        basis: HsaFundingBasis::Roth { value: value * 100, before_other_activity: true },
        coverage: HsaCoverage::SelfOnly,
        loss: None,
    }];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    d
}

// Derived Notice 2008-51 pp. 2–3 + 1.408A-6 A-4/A-8/A-9 case,
// not a published combined example. No current incoming Roth basis.
// Opening $800 regular, February ordinary $300; June value $700/funding
// $600 uses $200 earnings/$400 regular. September ordinary $200 after
// $100 growth uses remaining $100 regular/$100 earnings.
pub(in crate::tests) fn roth_earlier_funding() -> FederalDraft {
    let mut d = roth_hsa_funding(600, 800, 700, 0);
    if let FederalIncome::Roth { data } = &mut d.income[1].income {
        data.distributed = 20260615;
    }
    let transfer = &mut d.adjustments.as_mut().unwrap().hsa[0].funding[0];
    transfer.funded = 20260616;
    transfer.basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: false };
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 300, 20260215));
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 200, 20260915));
    for entry in &mut d.income {
        if let FederalIncome::Roth { data } = &mut entry.income {
            data.withholding = 0;
        }
    }
    d
}

// An inherited Roth IRA record (all the Roth IRAs from one decedent, died 2025-12-01). Its
// distributions come from the Roth withdrawals that name it (`inherited_roth_payment`).
pub(in crate::tests) fn inherited_roth(
    first_year: u32,
    regular: i64,
    converted: i64,
) -> FederalIncomeEntry {
    income(FederalIncome::InheritedIra {
        data: crate::FederalInheritedIra {
            decedent: "Synthetic decedent".into(),
            died: 20251201,
            kind: crate::InheritedIraKindFacts::Roth {
                first_year,
                basis: Some(InheritedRothBasis {
                    regular: regular * 100,
                    converted: converted * 100,
                }),
            },
        },
    })
}
// A code T Roth withdrawal of an inherited Roth IRA (reason: inherited), choosing the record.
pub(in crate::tests) fn inherited_roth_payment(
    owner: Owner,
    account: Option<i64>,
    gross: i64,
) -> FederalIncomeEntry {
    let mut e = roth_withdrawal(owner, gross, 20260601);
    let FederalIncome::Roth { data } = &mut e.income else { unreachable!() };
    data.code = RothCode::ExceptionApplies;
    data.withholding = 2000;
    data.inherited = crate::InheritedLink::Yes { account };
    e
}
// The record and one payment that names it, in dollars. Returns the record's identity.
pub(in crate::tests) fn add_inherited_roth(
    d: &mut FederalDraft,
    first_year: u32,
    gross: i64,
    regular: i64,
    converted: i64,
) -> i64 {
    let id = d.next_id;
    add(d, inherited_roth(first_year, regular, converted));
    add(d, inherited_roth_payment(Owner::Taxpayer, Some(id), gross));
    id
}
