use crate::tests::*;

pub(in crate::tests) fn personal_disaster(label: &str, basis: i64) -> PersonalCasualty {
    PersonalCasualty {
        id: 0,
        label: label.into(),
        date: 20260801,
        declaration: CasualtyDeclaration::Federal { reference: "EM-1234".into() },
        zip: "90210".into(),
        properties: vec![CasualtyProperty {
            id: 0,
            home_site: None,
            business_damage: None,
            label: "Household property".into(),
            location: "Synthetic City, CA".into(),
            acquisition: CasualtyAcquisition::Purchased { date: 20200101 },
            basis,
            before: basis,
            after: 0,
            reimbursement: 0,
            received: 0,
        }],
        source_complete: true,
    }
}

// The same event as a presidential major disaster (FEMA DR-): a qualified
// disaster loss with the $500 floor, no 10% of AGI reduction and the
// standard-deduction add-on.
pub(in crate::tests) fn major_disaster(label: &str, basis: i64) -> PersonalCasualty {
    PersonalCasualty {
        declaration: CasualtyDeclaration::MajorDisaster { reference: "DR-1234".into() },
        ..personal_disaster(label, basis)
    }
}

pub(in crate::tests) fn casualty_living_payment(
    claim: &str,
    received_on: u32,
    amount: i64,
    withholding: i64,
) -> FederalCasualtyLivingPayment {
    FederalCasualtyLivingPayment {
        insurer: "Synthetic insurer".into(),
        claim_reference: claim.into(),
        payment_reference: alloc::format!("{claim}-{received_on}"),
        received_on,
        amount,
        withholding,
    }
}

// Accepted canonical source cohort used by HOME casualty identity controls.
pub(in crate::tests) fn home_casualty_return(losses: &[i64]) -> FederalDraft {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(cash_business(50_000)));
    let source = draft.income.last().unwrap().id;
    let site = add_actual_office(
        &mut draft,
        source,
        actual_office(1000, 2000, HomeFixture::Rented { rent: 0 }),
    );
    for &loss in losses {
        let mut event = major_disaster("Synthetic repeated storm", loss);
        event.id = draft.next_id;
        draft.next_id += 1;
        event.properties[0].id = draft.next_id;
        draft.next_id += 1;
        event.properties[0].home_site = Some(site);
        event.properties[0].business_damage = Some(HomeCasualtyDamage::Damaged);
        draft.home_sites[0].casualties.push(event.id);
        draft.deductions.as_mut().unwrap().personal_casualties.push(event);
    }
    draft
}
