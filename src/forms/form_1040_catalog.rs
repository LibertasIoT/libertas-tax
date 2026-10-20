use crate::TaxForm;
use crate::kernel::form_catalog::{FieldKind::MoneyCents, FormCatalog, field};

// Current emitted scope only, not complete printable forms. Edition footers
// observed directly on the cited IRS PDFs on 2026-10-15: all DRAFT—DO NOT FILE.
// URLs may change; a new edition gets its own descriptor, never a silent relabel.
// Ratios/years must receive their actual semantic kind before future migration;
// they cannot be coerced into money/counts. Existing row emission stays in lines.
// Printed 11b copies 11a: both use the same existing AGI owner. The
// legacy combined "11a / 11b" review alias is not a printed field ID.
#[allow(dead_code)] // Additive descriptor; consumer migration follows phase A.
pub(crate) const FORM1040_2026_DRAFT_20260819: FormCatalog = FormCatalog {
    form: TaxForm::Form1040,
    tax_year: 2026,
    edition: "DRAFT—DO NOT FILE; Form 1040 (2026); Cat. No. 11320B; Created 8/19/26",
    source: "https://www.irs.gov/pub/irs-dft/f1040--dft.pdf",
    fields: &[
        field("1a", MoneyCents),
        field("1b", MoneyCents),
        field("1c", MoneyCents),
        field("1d", MoneyCents),
        field("1e", MoneyCents),
        field("1f", MoneyCents),
        field("1g", MoneyCents),
        field("1h", MoneyCents),
        field("1i", MoneyCents),
        field("1z", MoneyCents),
        field("2a", MoneyCents),
        field("2b", MoneyCents),
        field("3a", MoneyCents),
        field("3b", MoneyCents),
        field("4a", MoneyCents),
        field("4b", MoneyCents),
        field("5a", MoneyCents),
        field("5b", MoneyCents),
        field("6a", MoneyCents),
        field("6b", MoneyCents),
        field("7a", MoneyCents),
        field("8", MoneyCents),
        field("9", MoneyCents),
        field("10", MoneyCents),
        field("11a", MoneyCents),
        field("11b", MoneyCents),
        field("12e", MoneyCents),
        field("12f", MoneyCents),
        field("13a", MoneyCents),
        field("13b", MoneyCents),
        field("14", MoneyCents),
        field("15", MoneyCents),
        field("16", MoneyCents),
        field("17", MoneyCents),
        field("18", MoneyCents),
        field("19", MoneyCents),
        field("20", MoneyCents),
        field("21", MoneyCents),
        field("22", MoneyCents),
        field("23", MoneyCents),
        field("24a", MoneyCents),
        field("24b", MoneyCents),
        field("24c", MoneyCents),
        field("25a", MoneyCents),
        field("25b", MoneyCents),
        field("25c", MoneyCents),
        field("25d", MoneyCents),
        field("26", MoneyCents),
        field("27a", MoneyCents),
        field("28", MoneyCents),
        field("29", MoneyCents),
        field("30", MoneyCents),
        field("31", MoneyCents),
        field("32a", MoneyCents),
        field("32c", MoneyCents),
        field("33", MoneyCents),
        field("34", MoneyCents),
        field("35a", MoneyCents),
        field("36", MoneyCents),
        field("37", MoneyCents),
        field("38", MoneyCents),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{
        form_catalog::{CatalogError, FieldValue},
        source_fact::SourceFact,
    };

    #[test]
    fn reviewed_draft_catalog_has_unique_fields_and_explicit_printed_types() {
        let catalog = &FORM1040_2026_DRAFT_20260819;
        let ids: alloc::collections::BTreeSet<_> = catalog.fields.iter().map(|f| f.id).collect();
        assert_eq!(ids.len(), 61);
        assert_eq!(ids.len(), catalog.fields.len(), "duplicate printed field");
        let agi = SourceFact::Reported(FieldValue::MoneyCents(123_400));
        for id in ["11a", "11b"] {
            assert_eq!(FORM1040_2026_DRAFT_20260819.cell(id, agi.clone()).unwrap().value, agi);
        }
        assert_eq!(
            FORM1040_2026_DRAFT_20260819.cell("11a / 11b", agi),
            Err(CatalogError::UnknownField),
        );
    }
}
