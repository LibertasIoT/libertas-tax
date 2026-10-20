// Payment source facts for Form 8379. Date and amount remain the sole inputs
// to the existing underpayment calculation. Personal source facts are not AI imports.
use crate::Owner;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Who made the payment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum PaymentPayer {
    /// Taxpayer's separate payment
    Taxpayer,
    /// Spouse's separate payment
    Spouse,
    /// Joint payment
    Joint,
}

/// Source property of the payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum RefundSourceProperty {
    /// Separate property
    Separate {
        /// Owner of the separate funds
        owner: Owner,
    },
    /// Community property
    Community {
        /// Management of the community funds
        #[libertas_constrained_by("$.allowed_refund_management")]
        management: RefundManagement,
    },
    /// Partly community property
    Partly {
        /// Community part of this payment
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        community: i64,
        /// Owner of the separate part
        separate_owner: Owner,
        /// Management of the community part
        #[libertas_constrained_by("$.allowed_refund_management")]
        management: RefundManagement,
    },
}

/// Management of community funds
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum RefundManagement {
    /// Management classification is not needed in this state
    NotApplicable,
    /// Taxpayer's sole-management property
    Taxpayer,
    /// Spouse's sole-management property
    Spouse,
    /// Joint-management property
    Joint,
}

/// Accepted payment source property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum RefundSourcePropertyFacts {
    /// Separate property
    Separate {
        /// Owner of the separate funds
        owner: Owner,
    },
    /// Community property
    Community {
        /// Management of the community funds
        management: RefundManagement,
    },
    /// Partly community property
    Partly {
        /// Community part of this payment
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        community: i64,
        /// Owner of the separate part
        separate_owner: Owner,
        /// Management of the community part
        management: RefundManagement,
    },
}

/// Estimated tax payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredDatedPayment {
    /// Date paid
    #[libertas_date_only]
    // Tax-year source window: this date cannot be later than the applicable 2026 cutoff.
    #[libertas_number(min = $ESTIMATED_PAYMENT_START_DATE, max = $ESTIMATED_PAYMENT_LAST_DATE)]
    pub date: u32,
    /// Amount paid
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
    /// Who made this payment
    pub payer: PaymentPayer,
    /// Source property for community refund allocation
    /// [NullText]
    /// No community-property source allocation supplied
    /// [SetText]
    /// The community-property source allocation is
    #[libertas_default($null)]
    pub refund_property: Option<RefundSourceProperty>,
}

impl RefundSourceProperty {
    pub(crate) fn accepted(&self) -> RefundSourcePropertyFacts {
        match *self {
            Self::Separate { owner } => RefundSourcePropertyFacts::Separate { owner },
            Self::Community { management } => RefundSourcePropertyFacts::Community { management },
            Self::Partly { community, separate_owner, management } => {
                RefundSourcePropertyFacts::Partly { community, separate_owner, management }
            }
        }
    }
}
impl RefundSourcePropertyFacts {
    pub(crate) fn page(&self) -> RefundSourceProperty {
        match *self {
            Self::Separate { owner } => RefundSourceProperty::Separate { owner },
            Self::Community { management } => RefundSourceProperty::Community { management },
            Self::Partly { community, separate_owner, management } => {
                RefundSourceProperty::Partly { community, separate_owner, management }
            }
        }
    }
    pub(crate) fn issue(
        &self,
        amount: i64,
        allowed: &[i32],
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        match *self {
            Self::Separate { .. } => None,
            Self::Community { management } => (!allowed.contains(&management_index(management)))
                .then(|| {
                    ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT")
                        .at_field("management")
                }),
            Self::Partly { community, management, .. } => {
                if !(0..=amount).contains(&community) {
                    Some(
                        ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_ABOVE_PAYMENT")
                            .at_field("community"),
                    )
                } else {
                    (!allowed.contains(&management_index(management))).then(|| {
                        ValidationIssue::new("INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT")
                            .at_field("management")
                    })
                }
            }
        }
    }
}
impl InjuredDatedPayment {
    pub(crate) fn accepted(&self) -> crate::DatedPayment {
        crate::DatedPayment {
            date: self.date,
            amount: self.amount,
            payer: self.payer,
            refund_property: self.refund_property.as_ref().map(RefundSourceProperty::accepted),
        }
    }
    pub(crate) fn from_record(value: &crate::DatedPayment) -> Self {
        Self {
            date: value.date,
            amount: value.amount,
            payer: value.payer,
            refund_property: value.refund_property.as_ref().map(RefundSourcePropertyFacts::page),
        }
    }
}
pub(crate) fn management_index(value: RefundManagement) -> i32 {
    match value {
        RefundManagement::NotApplicable => {
            libertas_macros::variant_index!(RefundManagement::NotApplicable) as i32
        }
        RefundManagement::Taxpayer => {
            libertas_macros::variant_index!(RefundManagement::Taxpayer) as i32
        }
        RefundManagement::Spouse => {
            libertas_macros::variant_index!(RefundManagement::Spouse) as i32
        }
        RefundManagement::Joint => libertas_macros::variant_index!(RefundManagement::Joint) as i32,
    }
}
/// The answers an income document's withholding-fund management offers: the
/// three Texas managers, or none outside Texas, where the Nullable's null
/// answer is the only one and its row is withheld
/// (protocols.md#backend-constrained-choices).
pub(crate) fn allowed_document_management(texas: bool) -> Vec<i32> {
    if texas { allowed_management(true) } else { Vec::new() }
}

pub(crate) fn allowed_management(texas: bool) -> Vec<i32> {
    if texas {
        [RefundManagement::Taxpayer, RefundManagement::Spouse, RefundManagement::Joint]
            .map(management_index)
            .to_vec()
    } else {
        alloc::vec![management_index(RefundManagement::NotApplicable)]
    }
}
