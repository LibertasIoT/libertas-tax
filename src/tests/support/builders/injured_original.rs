use crate::tests::*;

// Original Form8379 A–I acceptance inputs; the existing native oracles retain
// their assertions. No new tax choices are inferred by the editor bridge.
pub(in crate::tests) fn community_ruling(
    state: u16,
    debt_facts: InjuredDebtFacts,
    injured_withheld: i64,
    liable_withheld: i64,
) -> FederalDraft {
    let mut a = allocation();
    a.debt = OffsetDebt::FederalTax;
    a.community = InjuredCommunityFacts::Yes { states: vec![state], debt_facts };
    let mut d = injured_return((7_500, injured_withheld), (22_500, liable_withheld), 0, a);
    for entry in &mut d.income {
        entry.community = CommunityShare::Community;
        if state == 7 {
            entry.refund_management = Some(if entry.owner == Owner::Taxpayer {
                RefundManagement::Taxpayer
            } else {
                RefundManagement::Spouse
            });
        }
    }
    d
}

pub(in crate::tests) fn fixture(profile: &str) -> FederalDraft {
    let (case, selected) = profile.rsplit_once('-').unwrap();
    let injured = match selected {
        "taxpayer" => Owner::Taxpayer,
        "spouse" => Owner::Spouse,
        _ => panic!("unknown injured selector"),
    };
    let (state, debt, own, liable) = match case {
        "a" => (1, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "b" => (9, InjuredDebtFacts::DuringMarriageNotFamily, 250, 750),
        "b-family" => (9, InjuredDebtFacts::DuringMarriageFamily, 250, 750),
        "c-ca" => (2, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "c-id" => (3, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "c-la" => (4, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "d-nv" => (5, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "d-nm" => (6, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "d-wa" => (8, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "e" => (7, InjuredDebtFacts::BeforeMarriage, 250, 750),
        "f" => (7, InjuredDebtFacts::BeforeMarriage, 750, 250),
        "f-zero" => (7, InjuredDebtFacts::BeforeMarriage, 1000, 0),
        _ => (0, InjuredDebtFacts::NotApplicable, 0, 0),
    };
    let mut d = if state != 0 {
        let mut d = community_ruling(state, debt, own, liable);
        // Swap actual people as well as the injured selector: the published
        // nonliable/liable source amounts stay the original ruling amounts.
        if injured == Owner::Spouse {
            for entry in &mut d.income {
                entry.owner =
                    if entry.owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
                entry.refund_management = entry.refund_management.map(|v| match v {
                    RefundManagement::Taxpayer => RefundManagement::Spouse,
                    RefundManagement::Spouse => RefundManagement::Taxpayer,
                    other => other,
                });
            }
        }
        d
    } else {
        match case {
            "g" => {
                let mut d = injured_return((0, 0), (0, 0), 0, allocation());
                let mut entry = retirement_entry(
                    Owner::Taxpayer,
                    RetirementKind::Pension,
                    12000,
                    RetirementTreatment::Ordinary,
                );
                entry.community = CommunityShare::Partly { percent: 50 };
                add(&mut d, entry);
                d.injured_spouse.as_mut().unwrap().community = InjuredCommunityFacts::Yes {
                    states: vec![5],
                    debt_facts: InjuredDebtFacts::NotApplicable,
                };
                d
            }
            "h" => {
                let mut d = injured_return((120_000, 30_000), (40_000, 6_000), 0, allocation());
                let claim =
                    marketplace_claim(&mut d, vec![marketplace_month(1, 1_000, 1_000, 1_000)]);
                d.credits.as_mut().unwrap().marketplace = claim;
                d
            }
            "i-standard" | "i-qbi" => {
                let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
                if let SpouseFiler::Yes { facts } = &mut d.people.as_mut().unwrap().spouse {
                    facts.birth_date = 19580615;
                }
                if case == "i-qbi" {
                    let mut source = income(reit_income(10000, 10000, true));
                    source.owner = Owner::Taxpayer;
                    add(&mut d, source);
                }
                d
            }
            "i-actc" => {
                let mut d = injured_return((20_000, 1_000), (0, 0), 0, allocation());
                d.dependents = vec![child(3)];
                d.next_id = 4;
                d.injured_spouse.as_mut().unwrap().claimants =
                    vec![InjuredClaimantFacts { person: 3, claimant: Owner::Spouse }];
                d
            }
            _ => panic!("unknown original-map case: {case}"),
        }
    };
    d.injured_spouse.as_mut().unwrap().injured = injured;
    d
}
