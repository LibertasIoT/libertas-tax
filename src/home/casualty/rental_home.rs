// Pub527 Worksheet5-1 line2c uses the otherwise allowable personal loss of
// the whole dwelling. Each saved property participates once in Form4684's
// shared event and AGI reductions, even when several cost records name it.
use crate::kernel::ratio;
use crate::{CasualtyProperty, FederalDraft, FederalIncome, FederalRental, PersonalCasualty};
use alloc::{collections::BTreeMap, vec::Vec};

const SOURCE: &str = "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE";
const BINDING: &str = "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE";
const OUTPUT: &str = "RENTAL_CASUALTY_OUTPUT_UNAVAILABLE";

struct Binding<'a> {
    rental: i64,
    event: &'a PersonalCasualty,
    property: &'a CasualtyProperty,
    fraction: [i64; 2],
    costs: Vec<usize>,
}

fn bindings(draft: &FederalDraft) -> Result<Vec<Binding<'_>>, (i64, &'static str)> {
    let sources = crate::source_associations::SourceAssociations::new(draft);
    let mut groups: Vec<Binding<'_>> = Vec::new();
    for entry in &draft.income {
        let FederalIncome::Rental { data } = &entry.income else { continue };
        for (index, cost) in data.buildings.iter().enumerate() {
            if cost.casualty_property.is_none() {
                continue;
            }
            let (event, property) = sources
                .rental_casualty_property(entry.id, cost.id)
                .map_err(|_| (entry.id, BINDING))?
                .ok_or((entry.id, BINDING))?;
            let (fair, days) = match data.personal_use {
                crate::RentalPersonalUse::No => (1, 1),
                crate::RentalPersonalUse::Yes { fair_rental_days, personal_days, .. } => {
                    let fair = i64::from(fair_rental_days);
                    if crate::extended_rules::used_as_home(data) && fair < 15 {
                        (0, 1)
                    } else {
                        (fair, fair + i64::from(personal_days))
                    }
                }
            };
            if days == 0 || property.home_site.is_some() {
                return Err((entry.id, BINDING));
            }
            if let Some(group) = groups.iter_mut().find(|group| group.property.id == property.id) {
                if group.rental != entry.id
                    || group.event.id != event.id
                    || group.fraction != [fair, days]
                {
                    return Err((entry.id, BINDING));
                }
                group.costs.push(index);
            } else {
                groups.push(Binding {
                    rental: entry.id,
                    event,
                    property,
                    fraction: [fair, days],
                    costs: alloc::vec![index],
                });
            }
        }
    }
    Ok(groups)
}

// Retain the saved whole-property facts and subtract its actual rental-use
// operands once. SectionA then owns the remaining personal portion normally.
pub(crate) fn project_rental_properties(
    draft: &FederalDraft,
    events: &mut [PersonalCasualty],
    personal_only: bool,
) -> Result<(), (i64, &'static str)> {
    for group in bindings(draft)? {
        let property = events
            .iter_mut()
            .find(|event| event.id == group.event.id)
            .and_then(|event| {
                event.properties.iter_mut().find(|property| property.id == group.property.id)
            })
            .ok_or((group.rental, BINDING))?;
        let accepted = draft
            .income
            .iter()
            .find(|entry| entry.id == group.rental)
            .ok_or((group.rental, BINDING))?;
        let ownership = draft
            .setup
            .as_ref()
            .and_then(|setup| {
                crate::community::share(
                    setup,
                    accepted.owner.owned(),
                    accepted.community,
                    &accepted.income,
                )
            })
            .map(|share| share.for_gains());
        for amount in [
            &mut property.basis,
            &mut property.before,
            &mut property.after,
            &mut property.reimbursement,
            &mut property.received,
        ] {
            let owned = ownership.as_ref().map_or(*amount, |share| share.filer(*amount));
            *amount = if personal_only {
                owned
                    .checked_sub(
                        ratio::checked_fraction(owned, group.fraction[0], group.fraction[1])
                            .ok_or((group.rental, OUTPUT))?,
                    )
                    .ok_or((group.rental, OUTPUT))?
            } else {
                owned
            };
        }
    }
    Ok(())
}

pub(crate) fn allowances(
    draft: &FederalDraft,
    rental: i64,
    source: &FederalRental,
    losses: &[(usize, i64)],
    amt: bool,
    itemizes: bool,
    modified_agi_cents: i64,
) -> Result<Vec<i64>, &'static str> {
    let events = crate::home_sites::casualty_events(draft, false).map_err(|_| SOURCE)?;
    let rows = super::all_personal_allowances(&events, modified_agi_cents).map_err(|_| OUTPUT)?;
    let groups = bindings(draft).map_err(|(_, key)| key)?;
    let mut allowances = BTreeMap::new();
    for group in groups.iter().filter(|group| group.rental == rental) {
        let row = rows
            .iter()
            .find(|row| row.event == group.event.id && row.property == group.property.id)
            .ok_or(SOURCE)?;
        if !row.available || group.property.gain() != 0 {
            return Err(SOURCE);
        }
        let mut weights = Vec::new();
        let mut basis = 0i64;
        let mut values = [0i64; 2];
        let mut reimbursement = 0i64;
        for index in &group.costs {
            let cost = source.buildings.get(*index).ok_or(SOURCE)?;
            let records = cost.casualty.as_ref().ok_or(SOURCE)?;
            let facts = super::rental::source_item(
                cost,
                records,
                amt,
                &source.property_type,
                draft.tax_year,
            )?;
            // The canonical personal worksheet retains its regular basis.
            // An independent AMT basis can exceed it without overlap.
            let regular_basis = if amt {
                super::rental::source_item(
                    cost,
                    records,
                    false,
                    &source.property_type,
                    draft.tax_year,
                )?
                .basis_cents
            } else {
                facts.basis_cents
            };
            basis = basis.checked_add(regular_basis).ok_or(OUTPUT)?;
            let fair = facts.fair_values_cents.ok_or(SOURCE)?;
            for i in 0..2 {
                values[i] = values[i].checked_add(fair[i]).ok_or(OUTPUT)?;
            }
            reimbursement = reimbursement.checked_add(facts.reimbursement_cents).ok_or(OUTPUT)?;
            weights
                .push(losses.iter().find(|(cost, _)| cost == index).map_or(0, |(_, loss)| *loss));
        }
        // Cost records cannot claim overlapping portions of a saved dwelling.
        if basis > group.property.basis
            || values[0] > group.property.before
            || values[1] > group.property.after
            || reimbursement > group.property.reimbursement
        {
            return Err(BINDING);
        }
        let capacity = ratio::checked_sum(weights.iter().copied()).ok_or(OUTPUT)?;
        let whole = row
            .qualified_cents
            .checked_add(if itemizes { row.ordinary_cents } else { 0 })
            .ok_or(OUTPUT)?;
        let otherwise = ratio::checked_fraction(whole, group.fraction[0], group.fraction[1])
            .ok_or(OUTPUT)?
            .min(capacity);
        for (index, allowance) in
            group.costs.iter().zip(ratio::allocate_remaining(otherwise, &weights))
        {
            allowances.insert(*index, allowance);
        }
    }
    losses.iter().map(|(index, _)| allowances.get(index).copied().ok_or(BINDING)).collect()
}
