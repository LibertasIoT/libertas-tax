use crate::TaxForm;
use crate::kernel::form_catalog::{
    FieldKind::{CountWhole, MoneyCents},
    FormCatalog, field,
};

#[allow(dead_code)] // Additive descriptor; consumer migration follows phase A.
pub(crate) const FORM8941_2026_DRAFT_20260408: FormCatalog = FormCatalog {
    form: TaxForm::Form8941,
    tax_year: 2026,
    edition: "DRAFT—DO NOT FILE; Form 8941 (2026); Cat. No. 37757S; Created 4/8/26",
    source: "https://www.irs.gov/pub/irs-dft/f8941--dft.pdf",
    fields: &[
        field("1", CountWhole),
        field("2", CountWhole),
        field("3", MoneyCents),
        field("4", MoneyCents),
        field("5", MoneyCents),
        field("6", MoneyCents),
        field("7", MoneyCents),
        field("8", MoneyCents),
        field("9", MoneyCents),
        field("10", MoneyCents),
        field("11", MoneyCents),
        field("12", MoneyCents),
        field("13", CountWhole),
        field("14", CountWhole),
        field("15", MoneyCents),
        field("16", MoneyCents),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{form_catalog::FieldValue, source_fact::SourceFact};

    #[test]
    fn reviewed_draft_catalog_has_unique_fields_and_explicit_printed_types() {
        let catalog = &FORM8941_2026_DRAFT_20260408;
        let ids: alloc::collections::BTreeSet<_> = catalog.fields.iter().map(|f| f.id).collect();
        assert_eq!(ids.len(), 16);
        assert_eq!(ids.len(), catalog.fields.len(), "duplicate printed field");
        for id in ["1", "2", "13", "14"] {
            assert!(
                FORM8941_2026_DRAFT_20260408
                    .cell(id, SourceFact::Reported(FieldValue::CountWhole(1)))
                    .is_ok()
            );
        }
    }
}
