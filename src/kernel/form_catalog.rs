// Additive catalog owner; consumer migration follows phase A.
#![allow(dead_code)]

use super::source_fact::SourceFact;
use crate::TaxForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldKind {
    MoneyCents,
    CountWhole,
    Checkbox,
    Text,
    DateOnly,
    Tin,
}

// Preserve the owning source's exact units and strings. Calendar/TIN/source
// validity remains with the source owner; a catalog never infers missing facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldValue<'a> {
    MoneyCents(i64),
    CountWhole(i64),
    Checkbox(bool),
    Text(&'a str),
    DateOnly(u32),
    Tin(&'a str),
}
impl FieldValue<'_> {
    fn kind(self) -> FieldKind {
        match self {
            Self::MoneyCents(_) => FieldKind::MoneyCents,
            Self::CountWhole(_) => FieldKind::CountWhole,
            Self::Checkbox(_) => FieldKind::Checkbox,
            Self::Text(_) => FieldKind::Text,
            Self::DateOnly(_) => FieldKind::DateOnly,
            Self::Tin(_) => FieldKind::Tin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FieldSpec {
    pub id: &'static str,
    pub kind: FieldKind,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FormCatalog {
    pub form: TaxForm,
    pub tax_year: i32,
    // Observed IRS form edition, independent of accepted-return revision.
    pub edition: &'static str,
    pub source: &'static str,
    pub fields: &'static [FieldSpec],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FieldCell<'a> {
    pub catalog: &'a FormCatalog,
    pub field: &'a FieldSpec,
    pub value: SourceFact<FieldValue<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CatalogError {
    UnknownField,
    WrongKind { expected: FieldKind, actual: FieldKind },
}

impl FormCatalog {
    pub(crate) fn cell<'a>(
        &'a self,
        id: &str,
        value: SourceFact<FieldValue<'a>>,
    ) -> Result<FieldCell<'a>, CatalogError> {
        let field = self.fields.iter().find(|f| f.id == id).ok_or(CatalogError::UnknownField)?;
        let value = value.check_reported(|value| {
            let actual = value.kind();
            if actual == field.kind {
                Ok(())
            } else {
                Err(CatalogError::WrongKind { expected: field.kind, actual })
            }
        })?;
        Ok(FieldCell { catalog: self, field, value })
    }
}

pub(crate) const fn field(id: &'static str, kind: FieldKind) -> FieldSpec {
    FieldSpec { id, kind }
}
