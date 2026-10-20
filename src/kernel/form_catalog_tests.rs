use super::form_catalog::*;
use super::source_fact::SourceFact;
use crate::TaxForm;

#[test]
fn typed_cells_preserve_source_units_and_reject_wrong_kind_or_field() {
    const CATALOG: FormCatalog = FormCatalog {
        form: TaxForm::Form1040,
        tax_year: 2026,
        edition: "test edition",
        source: "synthetic catalog",
        fields: &[
            FieldSpec { id: "money", kind: FieldKind::MoneyCents },
            FieldSpec { id: "count", kind: FieldKind::CountWhole },
            FieldSpec { id: "check", kind: FieldKind::Checkbox },
            FieldSpec { id: "text", kind: FieldKind::Text },
            FieldSpec { id: "date", kind: FieldKind::DateOnly },
            FieldSpec { id: "tin", kind: FieldKind::Tin },
        ],
    };
    for (id, value) in [
        ("money", FieldValue::MoneyCents(-101)),
        ("count", FieldValue::CountWhole(0)),
        ("check", FieldValue::Checkbox(false)),
        ("text", FieldValue::Text("")),
        ("date", FieldValue::DateOnly(20260228)),
        ("tin", FieldValue::Tin("001234567")),
    ] {
        let cell = CATALOG.cell(id, SourceFact::Reported(value)).unwrap();
        assert_eq!(cell.catalog, &CATALOG);
        assert_eq!(cell.field.id, id);
        assert_eq!(cell.value, SourceFact::Reported(value));
    }
    assert_eq!(
        CATALOG.cell("money", SourceFact::Reported(FieldValue::CountWhole(101))),
        Err(CatalogError::WrongKind {
            expected: FieldKind::MoneyCents,
            actual: FieldKind::CountWhole,
        }),
    );
    for fact in [SourceFact::NotReported, SourceFact::Unavailable] {
        assert_eq!(CATALOG.cell("money", fact.clone()).unwrap().value, fact.clone());
        assert_eq!(CATALOG.cell("absent", fact), Err(CatalogError::UnknownField));
    }
}
