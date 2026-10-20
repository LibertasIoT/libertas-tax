use crate::TaxForm;
use crate::kernel::form_catalog::{
    FieldKind::{Checkbox, MoneyCents},
    FormCatalog, field,
};

// 17a is a Boolean even though its legacy review row uses Count(0/1).
// Line 1 selection remains an unproduced Unit6 obligation.
#[allow(dead_code)] // Additive descriptor; consumer migration follows phase A.
pub(crate) const FORM8889_2026_DRAFT_20260430: FormCatalog = FormCatalog {
    form: TaxForm::Form8889,
    tax_year: 2026,
    edition: "DRAFT—DO NOT FILE; Form 8889 (2026); Cat. No. 37621P; Created 4/30/26",
    source: "https://www.irs.gov/pub/irs-dft/f8889--dft.pdf",
    fields: &[
        field("2", MoneyCents),
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
        field("13", MoneyCents),
        field("14a", MoneyCents),
        field("14b", MoneyCents),
        field("14c", MoneyCents),
        field("15", MoneyCents),
        field("16", MoneyCents),
        field("17a", Checkbox),
        field("17b", MoneyCents),
        field("18", MoneyCents),
        field("19", MoneyCents),
        field("20", MoneyCents),
        field("21", MoneyCents),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{form_catalog::FieldValue, source_fact::SourceFact};

    #[test]
    fn reviewed_draft_catalog_has_unique_fields_and_explicit_printed_types() {
        let catalog = &FORM8889_2026_DRAFT_20260430;
        let ids: alloc::collections::BTreeSet<_> = catalog.fields.iter().map(|f| f.id).collect();
        assert_eq!(ids.len(), 23);
        assert_eq!(ids.len(), catalog.fields.len(), "duplicate printed field");
        let checkbox = SourceFact::Reported(FieldValue::Checkbox(true));
        assert!(FORM8889_2026_DRAFT_20260430.cell("17a", checkbox).is_ok());
        assert!(
            FORM8889_2026_DRAFT_20260430
                .cell("17a", SourceFact::Reported(FieldValue::CountWhole(1)))
                .is_err()
        );
    }
}
