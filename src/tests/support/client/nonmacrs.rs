use crate::tests::support::builders::nonmacrs::{continuing, property};
use crate::tests::*;
pub(in crate::tests) fn farm_rental_physical_units_on_community_return(owner: Owner) {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves };
    let mut source = property(
        2,
        20250101,
        NonMacrsRecovery::Units {
            facts: UnitProduction {
                qbi_class: GdsEquipmentClass::Seven,
                elected_originally: true,
                current_units: 1,
                remaining_units: 1,
                remaining_basis: 1,
                salvage: 0,
            },
        },
    );
    continuing(&mut source, 1);
    assert!(source.valid() && source.qualifies());
    assert_eq!(source.deduction(false), 1);
    let mut page = crate::tests::farm::farm_rental::rental_page(100_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1;
    let mut row =
        document(0, IncomeSource::Record { value: crate::IncomeRecord::FarmRental(page) });
    row.owner = crate::DocumentOwner::from(owner);
    row.community = CommunityShare::Community;
    let mut session = restart(draft);
    // A separate return supplies its actual wage community-property treatment
    // on the wage topic before adding farm-rental property.
    let mut wages = crate::tests::cross::interview::open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { followups, .. } = &mut wages else { unreachable!() };
    let mut answered = 0;
    for question in followups {
        let crate::WagesDocumentAnswer::Community { value } = &mut question.answer;
        *value = CommunityShare::Separate;
        answered += 1;
    }
    assert_eq!(answered, 1);
    accepted(&send(&mut session, wages));
    let request = save_topic(&mut session, TaxSection::Farming, vec![row]);
    accepted(&send(&mut session, request));
    let reopened = crate::tests::cross::interview::open_topic(&mut session, TaxSection::Farming);
    accepted(&send(&mut session, reopened));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::FarmRental { .. }))
        .unwrap();
    let FederalIncome::FarmRental { data } = &entry.income else { unreachable!() };
    assert_eq!(data.assets.len(), 1);
    assert!(data.assets[0].id > 0);
    assert_eq!(data.assets[0].cost, 2);
    assert_eq!(data.assets[0].opening_basis(), 1);
    assert_eq!(data.assets[0].deduction(false), 1);
    let accepted_draft = session.draft.clone();
    assert_eq!(restart(accepted_draft.clone()).draft, accepted_draft);
    assert_eq!(
        accepted_draft.first_incomplete(),
        9,
        "actual normal Save completion: {:?}; adjustments={}, deductions={}, credits={}, screening={}, payments={}",
        accepted_draft.finished_sections,
        accepted_draft.adjustments.is_some(),
        accepted_draft.deductions.is_some(),
        accepted_draft.credits.is_some(),
        accepted_draft.screening.is_some(),
        accepted_draft.payments.is_some(),
    );
    // Splitting cost/prior/remaining first produces inconsistent intermediate
    // rounded facts. The physical predicate and source deduction precede Share.
    let result = supported(&accepted_draft);
    let id = alloc::format!("business_{}_asset_0_macrs", entry.id);
    let row = result
        .lines
        .iter()
        .find(|row| row.id == id)
        .expect("actual source row, including known zero");
    assert_eq!(row.amount, i64::from(owner == Owner::Taxpayer));
    // The physical one-cent allowance is below the whole annual expense's
    // dollar rounding boundary; it cannot alter Form 8958's source total.
    for (suffix, expected) in [("total", 10_000_000), ("this_return", 5_000_000)] {
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("form8958_{}_{suffix}", entry.id))
                .unwrap()
                .amount,
            expected
        );
    }
}
