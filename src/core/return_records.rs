use crate::{FederalDraft, TaxAppData, TaxDataKey, WorkingHeader};
use alloc::vec::Vec;
use libertas::LibertasUser;

pub(crate) const LAYOUT: u32 = 1;
pub(crate) type Records = Vec<(TaxDataKey, TaxAppData)>;
pub(crate) type RecordResult<T> = Result<T, &'static str>;

pub(crate) fn format_key(user: LibertasUser, tax_year: i32) -> TaxDataKey {
    // Kind 0 is the stable generation marker within this user/year namespace.
    TaxDataKey::ReturnRecord { user, tax_year, copy: 0, kind: 0, id: 0 }
}

pub(crate) fn known_format(value: &TaxAppData) -> RecordResult<()> {
    match value {
        TaxAppData::StorageFormat { format } if *format == crate::STORAGE_FORMAT => Ok(()),
        _ => Err("Unknown or corrupt storage format marker"),
    }
}

pub(crate) fn header_key(user: LibertasUser, tax_year: i32) -> TaxDataKey {
    TaxDataKey::ReturnRecord { user, tax_year, copy: 0, kind: 1, id: 0 }
}

pub(crate) fn header(draft: &FederalDraft, next_filed_copy: i64) -> WorkingHeader {
    // Exhaustive field ownership: adding a draft field requires assigning its
    // storage owner here. Body fields are projected by the four semantic owners.
    let FederalDraft {
        tax_year,
        next_id,
        cookie,
        revision,
        finished_sections,
        situations_unanswered,
        finished,
        setup,
        people,
        dependents: _,
        income,
        home_sites: _, // Required accepted section, including an empty collection.
        adjustments,
        deductions,
        credits,
        screening,
        payments,
        credit_refusals: _,
        often_missed: _,
        without_form: _,
        coverdell_excess: _,
        checklist: _,
        injured_spouse,
    } = draft;
    let section_presence = [
        setup.is_some(),
        people.is_some(),
        adjustments.is_some(),
        deductions.is_some(),
        credits.is_some(),
        screening.is_some(),
        payments.is_some(),
        injured_spouse.is_some(),
    ]
    .iter()
    .enumerate()
    .fold(0, |bits, (i, present)| bits | (u16::from(*present) << i));
    WorkingHeader {
        layout: LAYOUT,
        tax_year: *tax_year,
        next_id: *next_id,
        cookie: cookie.clone(),
        revision: *revision,
        finished_sections: finished_sections.clone(),
        situations_unanswered: situations_unanswered.clone(),
        finished: *finished,
        section_presence,
        income_count: i64::try_from(income.len()).expect("income count fits i64"),
        next_filed_copy,
    }
}

pub(crate) fn known_header(header: &WorkingHeader, tax_year: i32) -> RecordResult<()> {
    if header.layout != LAYOUT
        || header.tax_year != tax_year
        || header.next_filed_copy <= 0
        || !(1..i64::MAX).contains(&header.next_id)
        || header.revision < 0
        || header.section_presence & !255 != 0
        || header.income_count < 0
    {
        return Err("Unknown or corrupt working header");
    }
    Ok(())
}

pub(crate) fn project(
    user: LibertasUser,
    draft: &FederalDraft,
    next_copy: i64,
) -> RecordResult<Records> {
    let metadata = header(draft, next_copy);
    known_header(&metadata, draft.tax_year)?;
    let mut records = alloc::vec![
        (
            format_key(user, draft.tax_year),
            TaxAppData::StorageFormat { format: crate::STORAGE_FORMAT }
        ),
        (header_key(user, draft.tax_year), TaxAppData::WorkingHeader { header: metadata }),
    ];
    records.extend(
        crate::household_records::project(user, 0, draft)
            .map_err(|_| "Invalid household projection")?,
    );
    records
        .extend(draft.disassemble_setup_sections(user, 0).map_err(|_| "Invalid setup projection")?);
    records.extend(
        crate::income_records::project(user, 0, draft).map_err(|_| "Invalid income projection")?,
    );
    records.extend(
        draft.disassemble_accepted_sections(user, 0).map_err(|_| "Invalid section projection")?,
    );
    Ok(records)
}

pub(crate) fn assemble(
    user: LibertasUser,
    tax_year: i32,
    records: Records,
) -> RecordResult<(FederalDraft, WorkingHeader)> {
    if user <= 0 {
        return Err("Invalid verified user");
    }
    let mut format_present = false;
    let mut metadata = None;
    let mut household = Vec::new();
    let mut setup = Vec::new();
    let mut income = Vec::new();
    let mut sections = Vec::new();
    for (key, value) in records {
        let TaxDataKey::ReturnRecord { user: owner, tax_year: year, copy, kind, id } = &key else {
            return Err("Unexpected key in keyed return");
        };
        if *owner != user || *year != tax_year || *copy != 0 {
            return Err("Wrong return namespace");
        }
        match *kind {
            0 if *id == 0 => {
                known_format(&value)?;
                if core::mem::replace(&mut format_present, true) {
                    return Err("Duplicate storage format marker");
                }
            }
            1 if *id == 0 => {
                let TaxAppData::WorkingHeader { header } = value else {
                    return Err("Invalid header payload");
                };
                known_header(&header, tax_year)?;
                if metadata.replace(header).is_some() {
                    return Err("Duplicate header");
                }
            }
            2..=4 => household.push((key, value)),
            5..=6 => setup.push((key, value)),
            7 => income.push((key, value)),
            8..=16 => sections.push((key, value)),
            _ => return Err("Unknown record kind"),
        }
    }
    if !format_present {
        return Err("Missing storage format marker");
    }
    let metadata = metadata.ok_or("Missing header")?;
    let mut draft = FederalDraft::empty(metadata.cookie.clone());
    draft.tax_year = metadata.tax_year;
    draft.next_id = metadata.next_id;
    draft.revision = metadata.revision;
    draft.finished_sections = metadata.finished_sections.clone();
    draft.situations_unanswered = metadata.situations_unanswered.clone();
    draft.finished = metadata.finished;
    crate::household_records::assemble(user, 0, &mut draft, household)
        .map_err(|_| "Corrupt household records")?;
    draft.assemble_setup_sections(user, 0, &setup).map_err(|_| "Corrupt setup records")?;
    crate::income_records::assemble(
        user,
        0,
        &mut draft,
        usize::try_from(metadata.income_count).map_err(|_| "Invalid income count")?,
        income,
    )
    .map_err(|_| "Corrupt income records")?;
    draft.assemble_accepted_sections(user, 0, &sections).map_err(|_| "Corrupt section records")?;
    if header(&draft, metadata.next_filed_copy) != metadata {
        return Err("Incomplete return records");
    }
    // Financial/stored-answer validity is owned by Interview::new/valid_stored.
    // An invalid accepted draft is blocked there, never repaired or reset here.
    Ok((draft, metadata))
}

pub(crate) fn value_bytes(records: &Records) -> usize {
    records.iter().map(|(_, value)| value.to_avro().len()).sum()
}
