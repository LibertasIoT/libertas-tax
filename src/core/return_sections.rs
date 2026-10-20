use crate::{FederalDraft, TaxAppData, TaxDataKey};
use alloc::vec::Vec;
use libertas::LibertasUser;

/// Invalid setup or topic-selection projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionProjectionError {
    /// The owner or copy identity is invalid.
    Namespace,
    /// Both records must appear exactly once, with no additional records.
    Records,
    /// A key does not name the expected owner, year, copy or section.
    Key,
    /// The value does not match its section key.
    Payload,
}

fn section_keys(
    user: LibertasUser,
    tax_year: i32,
    copy: i64,
) -> Result<[TaxDataKey; 2], SectionProjectionError> {
    if user <= 0 || copy < 0 {
        return Err(SectionProjectionError::Namespace);
    }
    // Use the owning SDK-backed namespace helper, including its key errors.
    TaxDataKey::return_record_prefix(user, tax_year)
        .map_err(|_| SectionProjectionError::Namespace)?;
    Ok([5, 6].map(|kind| TaxDataKey::ReturnRecord { user, tax_year, copy, kind, id: 0 }))
}

impl FederalDraft {
    /// Project exactly setup and checklist into their two semantic records.
    /// The required setup record retains absence. Every accepted setup byte,
    /// including its stored label, is preserved without recalculation.
    /// This pure projection does not persist records or validate financial facts.
    pub fn disassemble_setup_sections(
        &self,
        user: LibertasUser,
        copy: i64,
    ) -> Result<Vec<(TaxDataKey, TaxAppData)>, SectionProjectionError> {
        let [setup, selection] = section_keys(user, self.tax_year, copy)?;
        Ok(alloc::vec![
            (setup, TaxAppData::SetupSection { setup: self.setup.clone() }),
            (selection, TaxAppData::SectionSelection { checklist: self.checklist }),
        ])
    }

    /// Restore exactly setup and checklist, leaving all other draft fields intact.
    /// Validate the entire supplied record set before changing either field.
    /// Financial validation remains with the existing accepted-return owner.
    pub fn assemble_setup_sections(
        &mut self,
        user: LibertasUser,
        copy: i64,
        records: &[(TaxDataKey, TaxAppData)],
    ) -> Result<(), SectionProjectionError> {
        let [setup_key, selection_key] = section_keys(user, self.tax_year, copy)?;
        if records.len() != 2 {
            return Err(SectionProjectionError::Records);
        }
        let mut setup = None;
        let mut checklist = None;
        for (key, value) in records {
            if *key == setup_key {
                if setup.is_some() {
                    return Err(SectionProjectionError::Records);
                }
                let TaxAppData::SetupSection { setup: value } = value else {
                    return Err(SectionProjectionError::Payload);
                };
                setup = Some(value);
            } else if *key == selection_key {
                if checklist.is_some() {
                    return Err(SectionProjectionError::Records);
                }
                let TaxAppData::SectionSelection { checklist: value } = value else {
                    return Err(SectionProjectionError::Payload);
                };
                checklist = Some(value);
            } else {
                return Err(SectionProjectionError::Key);
            }
        }
        let setup = setup.ok_or(SectionProjectionError::Records)?;
        let checklist = checklist.ok_or(SectionProjectionError::Records)?;
        self.setup = setup.clone();
        self.checklist = *checklist;
        Ok(())
    }
}
