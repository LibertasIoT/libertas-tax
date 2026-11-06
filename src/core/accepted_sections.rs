use crate::{FederalDraft, TaxAppData, TaxDataKey};
use alloc::vec::Vec;
use libertas::LibertasUser;

/// Invalid accepted-section storage projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptedSectionsError {
    /// The verified user or copy identity is invalid.
    Namespace,
    /// Required records are missing, duplicated or accompanied by extras.
    Records,
    /// A key does not match the draft namespace and fixed section selector.
    Key,
    /// The payload does not match its key.
    Payload,
}

fn section_keys(
    user: LibertasUser,
    tax_year: i32,
    copy: i64,
) -> Result<[TaxDataKey; 9], AcceptedSectionsError> {
    if user <= 0 || copy < 0 {
        return Err(AcceptedSectionsError::Namespace);
    }
    // The SDK owns the namespace encoding. Exact comparison against these keys
    // below validates its prefix, copy, kind and fixed scalar section selector.
    TaxDataKey::return_record_prefix(user, tax_year)
        .map_err(|_| AcceptedSectionsError::Namespace)?;
    Ok([8, 9, 10, 11, 12, 13, 14, 15, 16].map(|kind| TaxDataKey::ReturnRecord {
        user,
        tax_year,
        copy,
        kind,
        id: 0,
    }))
}

impl FederalDraft {
    /// Project the accepted body sections without changing facts.
    /// Every section has a required record even when its payload is absent.
    /// This pure projection performs no persistence or financial validation.
    pub fn disassemble_accepted_sections(
        &self,
        user: LibertasUser,
        copy: i64,
    ) -> Result<Vec<(TaxDataKey, TaxAppData)>, AcceptedSectionsError> {
        let keys = section_keys(user, self.tax_year, copy)?;
        Ok(keys
            .into_iter()
            .zip([
                TaxAppData::AdjustmentsSection { adjustments: self.adjustments.clone() },
                TaxAppData::DeductionsSection { deductions: self.deductions.clone() },
                TaxAppData::CreditsSection { credits: self.credits.clone() },
                TaxAppData::ScreeningSection { screening: self.screening.clone() },
                TaxAppData::InjuredSpouseSection { injured_spouse: self.injured_spouse.clone() },
                TaxAppData::ReviewSelections {
                    often_missed: self.often_missed.clone(),
                    credit_refusals: self.credit_refusals.clone(),
                    without_form: self.without_form.clone(),
                },
                TaxAppData::CoverdellExcessSection {
                    coverdell_excess: self.coverdell_excess.clone(),
                },
                TaxAppData::PaymentsSection { payments: self.payments.clone() },
                TaxAppData::HomeOfficesSection { home_sites: self.home_sites.clone() },
            ])
            .collect())
    }

    /// Restore only the accepted body sections.
    /// Validate the entire record set before changing any field. Accepted facts
    /// and metadata outside these sections retain their exact original values.
    pub fn assemble_accepted_sections(
        &mut self,
        user: LibertasUser,
        copy: i64,
        records: &[(TaxDataKey, TaxAppData)],
    ) -> Result<(), AcceptedSectionsError> {
        let keys = section_keys(user, self.tax_year, copy)?;
        if records.len() != keys.len() {
            return Err(AcceptedSectionsError::Records);
        }
        let mut adjustments = None;
        let mut deductions = None;
        let mut credits = None;
        let mut screening = None;
        let mut injured_spouse = None;
        let mut selections = None;
        let mut coverdell_excess = None;
        let mut payments = None;
        let mut home_sites = None;
        for (key, value) in records {
            let index = keys
                .iter()
                .position(|expected| expected == key)
                .ok_or(AcceptedSectionsError::Key)?;
            // The outer Option tracks required-record presence independently of
            // the inner Option payload, so an absent answer is not a missing row.
            match (index, value) {
                (0, TaxAppData::AdjustmentsSection { adjustments: value }) => {
                    if adjustments.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (1, TaxAppData::DeductionsSection { deductions: value }) => {
                    if deductions.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (2, TaxAppData::CreditsSection { credits: value }) => {
                    if credits.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (3, TaxAppData::ScreeningSection { screening: value }) => {
                    if screening.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (4, TaxAppData::InjuredSpouseSection { injured_spouse: value }) => {
                    if injured_spouse.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (
                    5,
                    TaxAppData::ReviewSelections { often_missed, credit_refusals, without_form },
                ) => {
                    if selections.replace((often_missed, credit_refusals, without_form)).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (6, TaxAppData::CoverdellExcessSection { coverdell_excess: value }) => {
                    if coverdell_excess.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (7, TaxAppData::PaymentsSection { payments: value }) => {
                    if payments.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                (8, TaxAppData::HomeOfficesSection { home_sites: value }) => {
                    if home_sites.replace(value).is_some() {
                        return Err(AcceptedSectionsError::Records);
                    }
                }
                _ => return Err(AcceptedSectionsError::Payload),
            }
        }
        let adjustments = adjustments.ok_or(AcceptedSectionsError::Records)?;
        let deductions = deductions.ok_or(AcceptedSectionsError::Records)?;
        let credits = credits.ok_or(AcceptedSectionsError::Records)?;
        let screening = screening.ok_or(AcceptedSectionsError::Records)?;
        let injured_spouse = injured_spouse.ok_or(AcceptedSectionsError::Records)?;
        let (often_missed, credit_refusals, without_form) =
            selections.ok_or(AcceptedSectionsError::Records)?;
        let coverdell_excess = coverdell_excess.ok_or(AcceptedSectionsError::Records)?;
        let payments = payments.ok_or(AcceptedSectionsError::Records)?;
        let home_sites = home_sites.ok_or(AcceptedSectionsError::Records)?;
        self.adjustments = adjustments.clone();
        self.deductions = deductions.clone();
        self.credits = credits.clone();
        self.screening = screening.clone();
        self.injured_spouse = injured_spouse.clone();
        self.often_missed = often_missed.clone();
        self.credit_refusals = credit_refusals.clone();
        self.without_form = without_form.clone();
        self.coverdell_excess = coverdell_excess.clone();
        self.payments = payments.clone();
        self.home_sites = home_sites.clone();
        Ok(())
    }
}
