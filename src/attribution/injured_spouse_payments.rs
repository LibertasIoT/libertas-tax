// Payment source facts for Form 8379. Date and amount remain the sole inputs
// to the existing underpayment calculation. Personal source facts are not AI imports.
use crate::common::Owner;
use crate::underpayment_model::DatedPayment;
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        community: i64,
        /// Owner of the separate part
        separate_owner: Owner,
        /// Management of the community part
        #[libertas_constrained_by("$.allowed_refund_management")]
        management: RefundManagement,
    },
}

pub use crate::federal_model::RefundManagement;

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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
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

/// Estimated tax payment method
/// Changing methods replaces the source details; enter the actual sources for the selected method.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstimatedPaymentEntry {
    /// No estimated payments or prior-year credit applied
    None,
    /// Actual payments by date
    /// Use the postmark date for mailed payments. A 2025 overpayment applied to 2026 counts as paid April 15, 2026 when your 2025 return was fully paid by then.
    Sources {
        /// Payment
        #[libertas_unordered]
        payments: Vec<InjuredDatedPayment>,
    },
    /// Annual amount paid in equal installments by every required due date
    /// Use dated sources if installments differed or a required payment was late.
    EqualTimely {
        /// Annual payment source
        #[libertas_unordered]
        payments: Vec<AnnualEstimatedAmount>,
    },
    /// Annual amount known; payment timing not established
    /// The annual credit remains known; exact penalty timing remains unavailable.
    TimingUnknown {
        /// Annual payment source
        #[libertas_unordered]
        payments: Vec<AnnualEstimatedAmount>,
    },
}

/// Annual estimated payment source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnualEstimatedAmount {
    /// Actual annual amount from this source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Who made these payments
    pub payer: PaymentPayer,
    /// Source property for community refund allocation
    /// [NullText]
    /// No community-property source allocation supplied
    /// [SetText]
    /// The community-property source allocation is
    pub refund_property: Option<RefundSourceProperty>,
}

/// Accepted estimated tax payment method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstimatedPaymentEntryRecord {
    None,
    Sources { payments: Vec<crate::DatedPayment> },
    EqualTimely { payments: Vec<AnnualEstimatedAmountRecord> },
    TimingUnknown { payments: Vec<AnnualEstimatedAmountRecord> },
}

/// Accepted annual estimated payment source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnualEstimatedAmountRecord {
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    pub payer: PaymentPayer,
    /// Source property for community refund allocation
    /// [NullText]
    /// No community-property source allocation supplied
    /// [SetText]
    /// The community-property source allocation is
    pub refund_property: Option<RefundSourcePropertyFacts>,
}

impl EstimatedPaymentEntry {
    pub(crate) fn accepted(&self) -> EstimatedPaymentEntryRecord {
        let annual = |payments: &[AnnualEstimatedAmount]| {
            payments
                .iter()
                .map(|p| AnnualEstimatedAmountRecord {
                    amount: p.amount,
                    payer: p.payer,
                    refund_property: p.refund_property.as_ref().map(RefundSourceProperty::accepted),
                })
                .collect()
        };
        match self {
            Self::None => EstimatedPaymentEntryRecord::None,
            Self::Sources { payments } => EstimatedPaymentEntryRecord::Sources {
                payments: payments.iter().map(InjuredDatedPayment::accepted).collect(),
            },
            Self::EqualTimely { payments } => {
                EstimatedPaymentEntryRecord::EqualTimely { payments: annual(payments) }
            }
            Self::TimingUnknown { payments } => {
                EstimatedPaymentEntryRecord::TimingUnknown { payments: annual(payments) }
            }
        }
    }
}
impl EstimatedPaymentEntryRecord {
    pub(crate) fn page(&self) -> EstimatedPaymentEntry {
        let annual = |payments: &[AnnualEstimatedAmountRecord]| {
            payments
                .iter()
                .map(|p| AnnualEstimatedAmount {
                    amount: p.amount,
                    payer: p.payer,
                    refund_property: p
                        .refund_property
                        .as_ref()
                        .map(RefundSourcePropertyFacts::page),
                })
                .collect()
        };
        match self {
            Self::None => EstimatedPaymentEntry::None,
            Self::Sources { payments } => EstimatedPaymentEntry::Sources {
                payments: payments.iter().map(InjuredDatedPayment::from_record).collect(),
            },
            Self::EqualTimely { payments } => {
                EstimatedPaymentEntry::EqualTimely { payments: annual(payments) }
            }
            Self::TimingUnknown { payments } => {
                EstimatedPaymentEntry::TimingUnknown { payments: annual(payments) }
            }
        }
    }
    // Borrow canonical funding independently of whether the payment dates are known.
    pub(crate) fn funding(
        &self,
    ) -> impl Iterator<Item = (i64, PaymentPayer, Option<&RefundSourcePropertyFacts>)> {
        let (dated, annual): (&[DatedPayment], &[AnnualEstimatedAmountRecord]) = match self {
            Self::None => (&[], &[]),
            Self::Sources { payments } => (payments, &[]),
            Self::EqualTimely { payments } | Self::TimingUnknown { payments } => (&[], payments),
        };
        dated
            .iter()
            .map(|p| (p.amount, p.payer, p.refund_property.as_ref()))
            .chain(annual.iter().map(|p| (p.amount, p.payer, p.refund_property.as_ref())))
    }
    pub(crate) fn count(&self) -> usize {
        self.funding().count()
    }
    pub(crate) fn issue(&self, allowed: &[i32]) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let variant = match self {
            Self::None => return None,
            Self::Sources { .. } => "Sources",
            Self::EqualTimely { .. } => "EqualTimely",
            Self::TimingUnknown { .. } => "TimingUnknown",
        };
        let located = |issue: ValidationIssue| issue.in_field(variant);
        if self.count() == 0 {
            return Some(located(
                ValidationIssue::new("PAYMENT_AMOUNT_INVALID").at_field("payments"),
            ));
        }
        for (index, (amount, _, property)) in self.funding().enumerate() {
            let mut issue = if amount <= 0 {
                Some(ValidationIssue::new("PAYMENT_AMOUNT_INVALID").at_field("amount"))
            } else {
                None
            };
            if let Some(property) = property {
                issue = issue.or_else(|| {
                    property.issue(amount, allowed).map(|issue| {
                        issue
                            .in_field(match property {
                                RefundSourcePropertyFacts::Separate { .. } => "Separate",
                                RefundSourcePropertyFacts::Community { .. } => "Community",
                                RefundSourcePropertyFacts::Partly { .. } => "Partly",
                            })
                            .in_field("refund_property")
                    })
                });
            }
            if let Self::Sources { payments } = self
                && issue.is_none()
                && !payments[index].valid()
            {
                issue = Some(ValidationIssue::new("PAYMENT_DATE_OUTSIDE_2026").at_field("date"));
            }
            if let Some(issue) = issue {
                return Some(located(issue.in_list("payments", index)));
            }
        }
        None
    }
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
    pub(crate) fn accepted(&self) -> DatedPayment {
        DatedPayment {
            date: self.date,
            amount: self.amount,
            payer: self.payer,
            refund_property: self.refund_property.as_ref().map(RefundSourceProperty::accepted),
        }
    }
    pub(crate) fn from_record(value: &DatedPayment) -> Self {
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
