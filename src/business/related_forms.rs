use crate::related_forms::{Activity, DocumentKind};
use crate::{Business, RelatedForms};

impl Activity for Business {
    const KINDS: &'static [DocumentKind] = crate::related_forms::BUSINESS_KINDS;

    fn related(&self) -> &RelatedForms {
        &self.related
    }

    fn related_mut(&mut self) -> &mut RelatedForms {
        &mut self.related
    }
}
