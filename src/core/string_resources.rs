type Entry = (&'static str, &'static str);
type Iter = core::iter::Flatten<core::iter::Copied<core::slice::Iter<'static, &'static [Entry]>>>;

/// A test view of the package's topic catalogs without copying their entries.
pub struct Catalog(&'static [&'static [Entry]]);

impl Catalog {
    pub fn iter(&self) -> Iter {
        self.0.iter().copied().flatten()
    }
}

impl IntoIterator for Catalog {
    type Item = &'static Entry;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub const APP_STRINGS: Catalog = Catalog(&[
    crate::CORE_STRINGS,
    crate::ACTIVITY_STRINGS,
    crate::ATTRIBUTION_STRINGS,
    crate::BUSINESS_STRINGS,
    crate::COST_RECOVERY_STRINGS,
    crate::CREDIT_STRINGS,
    crate::DEDUCTION_STRINGS,
    crate::FAMILY_STRINGS,
    crate::FARM_STRINGS,
    crate::FEDERAL_STRINGS,
    crate::FOREIGN_STRINGS,
    crate::FORM_STRINGS,
    crate::HEALTH_STRINGS,
    crate::HOME_STRINGS,
    crate::INCOME_STRINGS,
    crate::INVESTMENT_STRINGS,
    crate::PASSTHROUGH_STRINGS,
    crate::PAYMENT_STRINGS,
    crate::RETIREMENT_STRINGS,
]);
