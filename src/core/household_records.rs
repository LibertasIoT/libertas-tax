use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use libertas::LibertasUser;

use crate::{
    FederalDraft, FederalPeople, HouseholdPeople, HouseholdRecord, Owner, SpouseFiler, TaxAppData,
    TaxDataKey,
};

// This adapter owns precisely FederalDraft.people and FederalDraft.dependents.
// The other 20 fields retain their existing owners, including next_id and completion
// order. It neither allocates IDs nor validates
// financial facts; the existing accepted-return validator remains authoritative.
// The complete record owner supplies the header and persistence transaction.

/// A household record set cannot reconstruct its declared structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HouseholdRecordError {
    InvalidIdentity,
    WrongNamespace,
    UnexpectedRecord,
    DuplicateRecord,
    MissingRecord,
    InvalidOrder,
}

fn namespace(
    user: LibertasUser,
    copy: i64,
    draft: &FederalDraft,
) -> Result<(), HouseholdRecordError> {
    if user <= 0 || copy < 0 || !(1..i64::MAX).contains(&draft.next_id) {
        return Err(HouseholdRecordError::InvalidIdentity);
    }
    Ok(())
}

fn role_id(role: Owner) -> i64 {
    match role {
        Owner::Taxpayer => 1,
        Owner::Spouse => 2,
    }
}

/// Project all accepted household and person facts into semantic key/value records.
/// The caller supplies the verified user and authorized copy; the draft owns its year.
/// This is an in-memory projection, independent of the live monolithic store.
pub fn project(
    user: LibertasUser,
    copy: i64,
    draft: &FederalDraft,
) -> Result<Vec<(TaxDataKey, TaxAppData)>, HouseholdRecordError> {
    namespace(user, copy, draft)?;
    let key =
        |kind, id| TaxDataKey::ReturnRecord { user, tax_year: draft.tax_year, copy, kind, id };
    let mut records = Vec::new();
    let record = HouseholdRecord {
        people: draft.people.as_ref().map(|people| HouseholdPeople {
            parent_tax: people.parent_tax.clone(),
            spouse_present: people.spouse.is_some(),
        }),
        dependent_count: i64::try_from(draft.dependents.len())
            .map_err(|_| HouseholdRecordError::InvalidOrder)?,
    };
    records.push((key(2, 0), TaxAppData::Household { record }));
    if let Some(people) = &draft.people {
        for (role, facts) in
            [(Owner::Taxpayer, Some(&people.taxpayer)), (Owner::Spouse, people.spouse.as_ref())]
        {
            if let Some(facts) = facts {
                records.push((
                    key(3, role_id(role)),
                    TaxAppData::Filer { role, facts: facts.clone() },
                ));
            }
        }
    }
    let mut ids = BTreeSet::new();
    for (position, facts) in draft.dependents.iter().enumerate() {
        if facts.id <= 0 || facts.id >= draft.next_id || !ids.insert(facts.id) {
            return Err(HouseholdRecordError::InvalidIdentity);
        }
        records.push((
            key(4, facts.id),
            TaxAppData::Dependent {
                // The count was already checked above; every position fits i64.
                position: position as i64,
                facts: facts.clone(),
            },
        ));
    }
    Ok(records)
}

/// Assemble a complete household scope into an existing return.
/// Records may arrive in key order or any other order. Validate every record
/// before replacing either field; any error leaves the entire draft unchanged.
/// All other fields remain untouched. Full accepted-data validation follows
/// assembly through the existing return owner.
pub fn assemble(
    user: LibertasUser,
    copy: i64,
    draft: &mut FederalDraft,
    records: Vec<(TaxDataKey, TaxAppData)>,
) -> Result<(), HouseholdRecordError> {
    namespace(user, copy, draft)?;
    let mut household = None;
    let mut taxpayer = None;
    let mut spouse = None;
    let mut dependents = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for (key, value) in records {
        let TaxDataKey::ReturnRecord { user: owner, tax_year, copy: record_copy, kind, id } = key
        else {
            return Err(HouseholdRecordError::UnexpectedRecord);
        };
        if owner != user || tax_year != draft.tax_year || record_copy != copy {
            return Err(HouseholdRecordError::WrongNamespace);
        }
        match (kind, value) {
            (2, TaxAppData::Household { record }) if id == 0 => {
                if household.replace(record).is_some() {
                    return Err(HouseholdRecordError::DuplicateRecord);
                }
            }
            (3, TaxAppData::Filer { role, facts }) if id == role_id(role) => {
                let slot = match role {
                    Owner::Taxpayer => &mut taxpayer,
                    Owner::Spouse => &mut spouse,
                };
                if slot.replace(facts).is_some() {
                    return Err(HouseholdRecordError::DuplicateRecord);
                }
            }
            (4, TaxAppData::Dependent { position, facts }) => {
                if id <= 0 || id >= draft.next_id || id != facts.id {
                    return Err(HouseholdRecordError::InvalidIdentity);
                }
                if !ids.insert(id) {
                    return Err(HouseholdRecordError::DuplicateRecord);
                }
                if position < 0 || dependents.insert(position, facts).is_some() {
                    return Err(HouseholdRecordError::InvalidOrder);
                }
            }
            _ => return Err(HouseholdRecordError::UnexpectedRecord),
        }
    }
    let household = household.ok_or(HouseholdRecordError::MissingRecord)?;
    if household.dependent_count < 0
        || i64::try_from(dependents.len()).ok() != Some(household.dependent_count)
    {
        return Err(HouseholdRecordError::MissingRecord);
    }
    if dependents.keys().copied().ne(0..household.dependent_count) {
        return Err(HouseholdRecordError::InvalidOrder);
    }
    let people = if let Some(people) = household.people {
        let taxpayer = taxpayer.take().ok_or(HouseholdRecordError::MissingRecord)?;
        let spouse = if people.spouse_present {
            SpouseFiler::Yes { facts: spouse.take().ok_or(HouseholdRecordError::MissingRecord)? }
        } else {
            SpouseFiler::No
        };
        Some(FederalPeople { parent_tax: people.parent_tax, taxpayer, spouse })
    } else {
        None
    };
    if taxpayer.is_some() || spouse.is_some() {
        return Err(HouseholdRecordError::UnexpectedRecord);
    }
    draft.people = people;
    draft.dependents = dependents.into_values().collect();
    Ok(())
}
