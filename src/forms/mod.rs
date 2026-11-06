// One module per tax form, called by the federal return assembler in federal_rules.rs.
pub mod form_1040_es;
pub mod form_2441;
pub mod form_5329;
pub mod form_8606;
pub mod form_8862;
pub mod form_8889;
pub mod form_8959;
pub mod schedule_8812;
pub mod schedule_eic;
pub mod w2;

// Edition-specific descriptors for the currently produced form fields.
pub(crate) mod form_1040_catalog;
pub(crate) mod form_8889_catalog;
pub(crate) mod form_8941_catalog;

#[cfg(test)]
mod strings;
#[cfg(test)]
pub(crate) use self::strings::FORM_STRINGS;
