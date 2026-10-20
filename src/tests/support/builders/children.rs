use crate::tests::*;

// Form 8814 fixtures: a child's statements as the stored election holds them
// (whole dollars in, cents stored), and the election as the dependent's row.
pub(in crate::tests) fn child_interest(taxable: i64) -> ChildIncome {
    ChildIncome::Interest {
        data: native_interest(taxable * 100, 0),
        kept: ChildStatementKept::default(),
    }
}
pub(in crate::tests) fn child_dividends(
    ordinary: i64,
    qualified: i64,
    distributions: i64,
) -> ChildIncome {
    ChildIncome::Dividend {
        data: native_dividend(ordinary * 100, qualified * 100, distributions * 100),
        kept: ChildStatementKept::default(),
    }
}
pub(in crate::tests) fn child_election(
    child_id: i64,
    statements: Vec<ChildIncome>,
    alaska: i64,
    other_parent: OtherParentChoice,
) -> FederalIncomeEntry {
    FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: 0,
        label: alloc::format!("Child {child_id}"),
        owner: Owner::Taxpayer,
        income: FederalIncome::ChildElection {
            data: FederalChildElection {
                child_id,
                statements,
                alaska_dividend: alaska * 100,
                nontaxable_social: 0,
                other_parent,
                facts: 0,
            },
        },
        community: CommunityShare::NotNeeded,
    }
}
// The stored election of the draft's only Form 8814 row.
pub(in crate::tests) fn child_election_mut(d: &mut FederalDraft) -> &mut FederalChildElection {
    d.income
        .iter_mut()
        .find_map(|e| match &mut e.income {
            FederalIncome::ChildElection { data } => Some(data),
            _ => None,
        })
        .expect("a Form 8814 row")
}

// The page's "Died in 2026" choice with the ordinary answer Yes.
pub(in crate::tests) fn died_more(date: u32) -> MoreAboutYou {
    MoreAboutYou::Died { date, estate_medical: 0, bonds: None, ordinary: true }
}
pub(in crate::tests) fn death(date: u32) -> Option<FinalReturn> {
    Some(FinalReturn { bonds: None, date, ordinary: false, estate_medical: 0 })
}

pub(in crate::tests) fn ordinary_parent() -> ParentTax {
    ParentTax {
        foreign_excluded: None,
        gains: None,
        status: FederalStatus::Joint,
        taxable: 15000000,
        others: vec![],
        complete: true,
        ordinary: true,
    }
}
pub(in crate::tests) fn young_interest_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    d.people.as_mut().unwrap().parent_tax = Some(ordinary_parent());
    d.income.clear();
    add(&mut d, income(savings_income(10000, 0)));
    d
}

pub(in crate::tests) fn young_special_return(
    collectibles: i64,
    unrecaptured: i64,
    other: i64,
) -> FederalDraft {
    let mut d = young_interest_return();
    let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
    parent.status = FederalStatus::Single;
    parent.taxable = 25000000;
    d.income[0].income = FederalIncome::Dividend {
        data: FederalDividend {
            collectible_distributions: collectibles * 100,
            unrecaptured_distributions: unrecaptured * 100,
            ..native_dividend(0, 0, (collectibles + unrecaptured + other) * 100)
        },
    };
    d
}

pub(in crate::tests) fn child_interest_gains(
    use_of_interest: InvestmentInterestUse,
) -> FederalDraft {
    let mut d = young_interest_return();
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(400000, 400000, 600000)
            },
        }),
    );
    investment_borrowing(&mut d, 4000, 2000, 2000);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use =
        use_of_interest;
    d
}

// Sets or clears one of a stored dependent's PersonFacts bits.
pub(in crate::tests) fn set_fact(person: &mut FederalDependent, flag: PersonFacts, on: bool) {
    if on {
        person.facts |= flag.0;
    } else {
        person.facts &= !flag.0;
    }
}
// The Form 8332 release bits of a stored dependent.
pub(in crate::tests) fn set_release(person: &mut FederalDependent, release: CustodyRelease) {
    set_fact(person, PersonFacts::YOU_RELEASED, release == CustodyRelease::YouReleased);
    set_fact(person, PersonFacts::RELEASED_TO_YOU, release == CustodyRelease::ReleasedToYou);
}

pub(in crate::tests) fn child_election_owner_fixture(owner: Owner, mixed: bool) -> FederalDraft {
    let mut draft = injured_return((60_000, 15_000), (20_000, 5_000), 0, allocation());
    add_child(&mut draft);
    let child = draft.dependents[0].id;
    draft.injured_spouse.as_mut().unwrap().claimants =
        vec![InjuredClaimantFacts { person: child, claimant: Owner::Taxpayer }];
    let statements = if mixed {
        vec![child_interest(1_000), child_dividends(1_200, 800, 2_000)]
    } else {
        vec![child_interest(2_000)]
    };
    let mut entry = child_election(child, statements, 0, OtherParentChoice::NotNeeded);
    entry.owner = owner;
    add(&mut draft, entry);
    draft
}

pub(in crate::tests) fn noncitizen_child(d: &mut FederalDraft) {
    add_child(d);
    let c = d.dependents.last_mut().unwrap();
    set_fact(c, PersonFacts::ABROAD, true);
    c.valid_ssn = false;
}
