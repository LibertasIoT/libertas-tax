use crate::foreign_source_basics::ForeignWithholding;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Existing foreign-income source used by an allocation worksheet
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignAllocationSource {
    /// Ordinary foreign-income record
    Direct {
        /// Exact unique country and payer label from ordinary foreign-income records
        #[libertas_size(min = 1)]
        label: String,
    },
    /// Fiduciary passive portfolio allocation
    Estate {
        /// Accepted estate or trust Schedule K-1
        parent: i64,
        /// Country on that K-1's foreign-source allocation
        #[libertas_size(min = 2, max = 3)]
        country: String,
        /// Withholding treatment of that allocation
        withholding: ForeignWithholding,
    },
}

pub(crate) fn class(withholding: &ForeignWithholding) -> u8 {
    match withholding {
        ForeignWithholding::None => 0,
        ForeignWithholding::Below15 => 1,
        ForeignWithholding::AtLeast15 => 2,
    }
}
impl From<&str> for ForeignAllocationSource {
    fn from(label: &str) -> Self {
        Self::Direct { label: label.into() }
    }
}
impl From<String> for ForeignAllocationSource {
    fn from(label: String) -> Self {
        Self::Direct { label }
    }
}
impl ForeignAllocationSource {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Direct { label } => !label.trim().is_empty(),
            Self::Estate { parent, country, .. } => *parent > 0 && country_size_valid(country),
        }
    }
    pub(crate) fn same_as(&self, reference: &Self) -> bool {
        match (self, reference) {
            (Self::Direct { label }, Self::Direct { label: other }) => label == other,
            (
                Self::Estate { parent, country, withholding },
                Self::Estate {
                    parent: other,
                    country: other_country,
                    withholding: other_withholding,
                },
            ) => {
                parent == other
                    && country.trim().eq_ignore_ascii_case(other_country.trim())
                    && class(withholding) == class(other_withholding)
            }
            _ => false,
        }
    }
}

pub(crate) fn country_size_valid(country: &str) -> bool {
    (2..=3).contains(&country.len())
}
