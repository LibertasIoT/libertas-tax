use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use libertas::LibertasUser;

use crate::{
    FederalDraft, FederalIncome, FederalIncomeEntry, PartnerPersonalRecords,
    StoredPartnershipDocument, StoredScorpDocument, TaxAppData, TaxDataKey,
};

// This pure adapter owns only FederalDraft.income. A native FederalSale is
// already one semantic sale under its FederalIncomeEntry.id. The native
// several_sales flag can describe one aggregate statement amount; it supplies
// no child facts or identities and is retained verbatim, never split or guessed.
// The caller supplies the complete scope's count from its owning metadata.
// No header, transaction, financial validation or live-store cutover lives here.

/// An income record set cannot reconstruct its declared structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncomeRecordError {
    InvalidIdentity,
    WrongNamespace,
    UnexpectedRecord,
    DuplicateRecord,
    InvalidOrder,
    CountMismatch,
}

fn namespace(user: LibertasUser, copy: i64, draft: &FederalDraft) -> Result<(), IncomeRecordError> {
    if user <= 0 || copy < 0 {
        return Err(IncomeRecordError::InvalidIdentity);
    }
    TaxDataKey::return_record_prefix(user, draft.tax_year)
        .map_err(|_| IncomeRecordError::WrongNamespace)?;
    Ok(())
}

/// Project every accepted income entry, retaining native identity and order.
/// The verified owner and authorized copy come from the caller; the draft owns
/// its actual tax year. No financial fact is normalized or recalculated.
pub fn project(
    user: LibertasUser,
    copy: i64,
    draft: &FederalDraft,
) -> Result<Vec<(TaxDataKey, TaxAppData)>, IncomeRecordError> {
    namespace(user, copy, draft)?;
    let mut ids = BTreeSet::new();
    draft
        .income
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            if entry.id <= 0 || !ids.insert(entry.id) {
                return Err(IncomeRecordError::InvalidIdentity);
            }
            let position = i64::try_from(position).map_err(|_| IncomeRecordError::InvalidOrder)?;
            Ok((
                TaxDataKey::ReturnRecord {
                    user,
                    tax_year: draft.tax_year,
                    copy,
                    kind: 7,
                    id: entry.id,
                },
                match &entry.income {
                    crate::FederalIncome::Partnership { data } if data.source.is_some() => {
                        let record = crate::StoredPartnershipDocument::from_entry(entry)
                            .ok_or(IncomeRecordError::UnexpectedRecord)?;
                        TaxAppData::PartnershipDocument { position, record }
                    }
                    crate::FederalIncome::Scorp { .. } => {
                        let record = crate::StoredScorpDocument::from_entry(entry)
                            .ok_or(IncomeRecordError::UnexpectedRecord)?;
                        TaxAppData::ScorpDocument { position, record }
                    }
                    crate::FederalIncome::Broker { .. } => {
                        let mut stored = entry.clone();
                        stored.printed = None;
                        stored.state_source = None;
                        TaxAppData::IncomeDocument { position, entry: stored }
                    }
                    _ => TaxAppData::IncomeDocument { position, entry: entry.clone() },
                },
            ))
        })
        .collect()
}

/// Assemble a complete income scope in any scan order.
/// `expected_count` belongs to the caller's complete return metadata; an absent
/// record is corruption, including when all records are absent. Every record
/// is checked before replacing income. All other draft fields remain untouched.
pub fn assemble(
    user: LibertasUser,
    copy: i64,
    draft: &mut FederalDraft,
    expected_count: usize,
    records: Vec<(TaxDataKey, TaxAppData)>,
) -> Result<(), IncomeRecordError> {
    namespace(user, copy, draft)?;
    if records.len() != expected_count {
        return Err(IncomeRecordError::CountMismatch);
    }
    let mut entries = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for (key, value) in records {
        let TaxDataKey::ReturnRecord { user: owner, tax_year, copy: record_copy, kind, id } = key
        else {
            return Err(IncomeRecordError::UnexpectedRecord);
        };
        if owner != user || tax_year != draft.tax_year || record_copy != copy {
            return Err(IncomeRecordError::WrongNamespace);
        }
        let (position, mut entry) = match value {
            TaxAppData::IncomeDocument { position, entry } => {
                if matches!(&entry.income,crate::FederalIncome::Partnership {data} if data.source.is_some())
                {
                    return Err(IncomeRecordError::UnexpectedRecord);
                }
                if matches!(&entry.income, crate::FederalIncome::Scorp { .. }) {
                    return Err(IncomeRecordError::UnexpectedRecord);
                }
                (position, entry)
            }
            TaxAppData::ScorpDocument { position, record } => {
                (position, record.into_entry().map_err(|_| IncomeRecordError::UnexpectedRecord)?)
            }
            TaxAppData::PartnershipDocument { position, record } => {
                (position, record.into_entry().map_err(|_| IncomeRecordError::UnexpectedRecord)?)
            }
            _ => return Err(IncomeRecordError::UnexpectedRecord),
        };
        if let crate::FederalIncome::Broker { data } = &entry.income {
            if entry.printed.is_some() || entry.state_source.is_some() {
                return Err(IncomeRecordError::UnexpectedRecord);
            }
            entry.printed = Some(data.printed_names());
            entry.state_source = crate::IncomeStateSource::from_document(&data.income_source());
        }
        if kind != 7 {
            return Err(IncomeRecordError::UnexpectedRecord);
        }
        if id <= 0 || id != entry.id {
            return Err(IncomeRecordError::InvalidIdentity);
        }
        if !ids.insert(id) {
            return Err(IncomeRecordError::DuplicateRecord);
        }
        if position < 0
            || usize::try_from(position).ok().is_none_or(|p| p >= expected_count)
            || entries.insert(position, entry).is_some()
        {
            return Err(IncomeRecordError::InvalidOrder);
        }
    }
    draft.income = entries.into_values().collect();
    Ok(())
}

impl StoredScorpDocument {
    pub(crate) fn from_entry(entry: &FederalIncomeEntry) -> Option<Self> {
        let FederalIncome::Scorp { data } = &entry.income else { return None };
        let source = data.clone();
        source.admission_issue().map_or(Some(()), |_| None)?;
        Some(Self {
            id: entry.id,
            label: entry.label.clone(),
            owner: entry.owner.owned(),
            community: entry.community,
            state_source: entry.state_source.clone(),
            refund_management: entry.refund_management,
            source,
        })
    }
    pub(crate) fn into_entry(self) -> Result<FederalIncomeEntry, &'static str> {
        if let Some(issue) = self.source.admission_issue() {
            return Err(issue.key);
        }
        Ok(FederalIncomeEntry {
            id: self.id,
            label: self.label,
            owner: crate::DocumentOwner::from(self.owner),
            community: self.community,
            printed: Some(self.source.printed_names()),
            state_source: self.state_source,
            refund_management: self.refund_management,
            income: FederalIncome::Scorp { data: self.source },
        })
    }
}

impl StoredPartnershipDocument {
    pub(crate) fn from_entry(entry: &FederalIncomeEntry) -> Option<Self> {
        let FederalIncome::Partnership { data } = &entry.income else { return None };
        let mut source = data.source.as_deref()?.clone();
        source.bind_accepted_child_ids(data);
        source.admission_issue().map_or(Some(()), |_| None)?;
        Some(Self {
            id: entry.id,
            label: entry.label.clone(),
            owner: entry.owner.owned(),
            community: entry.community,
            state_source: entry.state_source.clone(),
            refund_management: entry.refund_management,
            source,
            personal: PartnerPersonalRecords::from_calculation(data)?,
        })
    }
    pub(crate) fn into_entry(self) -> Result<FederalIncomeEntry, &'static str> {
        if let Some(issue) = self.source.admission_issue() {
            return Err(issue.key);
        }
        let printed = Some(self.source.printed_names());
        Ok(FederalIncomeEntry {
            id: self.id,
            label: self.label,
            owner: crate::DocumentOwner::from(self.owner),
            community: self.community,
            printed,
            state_source: self.state_source,
            refund_management: self.refund_management,
            income: FederalIncome::Partnership { data: self.source.calculation(&self.personal) },
        })
    }
}
