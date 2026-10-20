// Isolated reader for the one rehearsed, unpublished monolith: main
// 701d0e944a37d9ce1d93d0e0bcff6c77d25614e6, STORAGE_FORMAT 124.
// Changed HOME ancestors use frozen native bodies; legacy office conversion
// refuses missing canonical sources before any transaction mutation.
mod frozen;
pub use frozen::*;

use crate::*;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Stored root tag 1 retains this exact known body, independently of the
// current in-memory FederalDraft. Keep every old field in declaration order.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Draft124 {
    /// Tax year
    pub tax_year: i32,
    /// Next document or person identity
    pub next_id: i64,
    /// Return identity
    pub cookie: String,
    /// Accepted revision
    pub revision: i64,
    /// Return setup
    pub setup: Option<FederalSetup>,
    /// Filer facts
    pub people: Option<FederalPeople>,
    /// Accepted children and dependents
    /// Empty when you support no one; the people page's visit is in finished sections.
    pub dependents: Vec<FederalDependent>,
    /// Accepted income documents
    pub income: Vec<FederalIncomeEntry124>,
    /// Finished sections
    pub finished_sections: Vec<crate::TaxSection>,
    /// Accepted adjustments
    pub adjustments: Option<FederalAdjustments>,
    /// Accepted deductions
    pub deductions: Option<FederalDeductions124>,
    /// Accepted common credits
    pub credits: Option<FederalCredits>,
    /// Accepted coverage review
    pub screening: Option<FederalScreening>,
    /// Accepted payments
    pub payments: Option<FederalPayments124>,
    /// Review completed
    pub finished: bool,
    /// Less common income and itemized deduction situations that apply
    pub situations: Vec<RareSituation>,
    /// Adjustments chosen that are not calculated yet
    pub adjustment_refusals: Vec<MoreAdjustment>,
    /// Credits chosen that are not calculated yet
    pub credit_refusals: Vec<MoreCreditRefusal>,
    /// Other taxes chosen that are not calculated yet
    pub tax_refusals: Vec<OtherTaxRefusal>,
    /// Beneficiary-owned Coverdell ESA excess and retained carry
    pub coverdell_excess: Vec<crate::CoverdellExcess>,
    /// Your 2026 at a glance
    /// Every topic ticked; a topic with entries is always ticked.
    pub checklist: Checklist,
    /// Injured spouse allocation (Form 8379)
    pub injured_spouse: Option<crate::InjuredSpouseAllocation>,
}
impl Draft124 {
    pub fn into_current(self) -> Result<FederalDraft, &'static str> {
        let mut draft = FederalDraft {
            tax_year: self.tax_year,
            next_id: self.next_id,
            cookie: self.cookie,
            revision: self.revision,
            setup: self.setup,
            people: self.people,
            dependents: self.dependents,
            income: self.income.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            // The old explicit None office answers establish absence. Every
            // other office variant was refused above, never guessed or dropped.
            home_sites: Vec::new(),
            finished_sections: self.finished_sections,
            adjustments: self.adjustments,
            deductions: self.deductions.map(Into::into),
            credits: self.credits,
            screening: self.screening,
            payments: self.payments.map(Into::into),
            finished: self.finished,
            situations: self.situations,
            adjustment_refusals: self.adjustment_refusals,
            credit_refusals: self.credit_refusals,
            tax_refusals: self.tax_refusals,
            coverdell_excess: self.coverdell_excess,
            checklist: self.checklist,
            injured_spouse: self.injured_spouse,
        };
        // Check the accepted legacy identities against the original counter
        // before allocating newly required casualty identities through its owner.
        if !draft.valid_stored() {
            return Err("Invalid legacy return facts");
        }
        if let Some(deductions) = &mut draft.deductions {
            crate::interview::assign_record_ids(
                deductions.personal_casualties.iter_mut().map(|event| &mut event.id),
                core::iter::empty(),
                &mut draft.next_id,
            )
            .map_err(|(_, message, _)| message)?;
            for event in &mut deductions.personal_casualties {
                crate::interview::assign_record_ids(
                    event.properties.iter_mut().map(|property| &mut property.id),
                    core::iter::empty(),
                    &mut draft.next_id,
                )
                .map_err(|(_, message, _)| message)?;
            }
        }
        // Allocation can consume the last representable successor. Direct
        // conversion must reject that result just like the persisted projection.
        if !draft.valid_stored() {
            return Err("Invalid migrated return facts");
        }
        Ok(draft)
    }
}

#[cfg(test)]
impl TryFrom<FederalDraft> for Draft124 {
    type Error = &'static str;
    fn try_from(draft: FederalDraft) -> Result<Self, Self::Error> {
        if !draft.home_sites.is_empty() {
            return Err("Current HOME facts cannot be retained in format 124");
        }
        let FederalDraft {
            tax_year,
            next_id,
            cookie,
            revision,
            setup,
            people,
            dependents,
            income,
            home_sites: _,
            finished_sections,
            adjustments,
            deductions,
            credits,
            screening,
            payments,
            finished,
            situations,
            adjustment_refusals,
            credit_refusals,
            tax_refusals,
            coverdell_excess,
            checklist,
            injured_spouse,
        } = draft;
        Ok(Self {
            tax_year,
            next_id,
            cookie,
            revision,
            setup,
            people,
            dependents,
            income: income.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            finished_sections,
            adjustments,
            deductions: deductions.map(TryInto::try_into).transpose()?,
            credits,
            screening,
            payments: payments.map(TryInto::try_into).transpose()?,
            finished,
            situations,
            adjustment_refusals,
            credit_refusals,
            tax_refusals,
            coverdell_excess,
            checklist,
            injured_spouse,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Migration124 {
    Migrated,
    AlreadyMigrated,
    NoLegacyReturn,
}

pub(crate) fn migrate_in(
    tx: &mut impl crate::storage::RecordTransaction,
    user: libertas::LibertasUser,
    year: i32,
) -> Result<Migration124, &'static str> {
    if user <= 0 {
        return Err("Verified positive user required");
    }
    // Existing committed keyed state always wins on replay; never overwrite it.
    if crate::storage::read_in(tx, user, year)?.is_some() {
        return Ok(Migration124::AlreadyMigrated);
    }
    let marker_key = TaxDataKey::StorageFormat { user };
    let draft_key = TaxDataKey::Draft { user };
    match tx.get(&marker_key) {
        None => {
            return if tx.exists(&draft_key) {
                Err("Missing legacy storage format")
            } else {
                Ok(Migration124::NoLegacyReturn)
            };
        }
        Some(TaxAppData::StorageFormat { format: 124 }) => (),
        _ => return Err("Unsupported legacy storage format"),
    }
    // This read follows the stable marker check. The ordinary derived root
    // decoder reads its retained Draft124 payload, never the current draft type.
    let draft = match tx.get(&draft_key) {
        Some(TaxAppData::Draft { draft }) => draft.into_current()?,
        None => return Err("Missing legacy draft"),
        _ => return Err("Invalid legacy draft payload"),
    };
    if draft.tax_year != year || !draft.valid_stored() {
        return Err("Invalid or unauthorized legacy return year");
    }
    let records = crate::return_records::project(user, &draft, 1)?;
    for (key, value) in records {
        tx.put(&key, &value);
    }
    tx.delete(&draft_key);
    // Retain stable marker bytes as provenance; normal writes never use roots 0/1.
    Ok(Migration124::Migrated)
}

// Explicit, isolated, quiesced-writer migration entry for the one synthetic
// format-124 rehearsal. Not a filing endpoint or an automatic retry/recovery
// path. The caller supplies the Hub-verified user and server-authorized year.
// Quiescing the old writer is a cutover prerequisite owned by the dispatcher.
pub fn migrate_storage124(
    user: libertas::LibertasUser,
    year: i32,
) -> Result<Migration124, &'static str> {
    use libertas::data::{Data, Mode, Target};
    // Target::keys is limited to 4096 keys. ALL is used only for this isolated
    // migration, since legacy roots and >4096 destination keys share no prefix.
    // The internal read range and mutations remain confined to the one user.
    let data = Data::<TaxDataKey, TaxAppData>::current();
    let mut tx = data.begin(&Target::all(), Mode::ReadWrite);
    let result = migrate_in(&mut tx, user, year)?;
    tx.commit();
    Ok(result)
}
