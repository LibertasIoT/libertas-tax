use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual source payments in a long-term-care period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredLtcPeriodPayees {
    /// Payments identified by their existing 1099-LTC source
    Payments {
        /// Each source's actual periodic payment in these dates
        #[libertas_size(min = 1)]
        payments: Vec<InjuredLtcPeriodPayment>,
    },
}
impl InjuredLtcPeriodPayees {
    pub(crate) fn payments(&self) -> &[InjuredLtcPeriodPayment] {
        match self {
            Self::Payments { payments } => payments,
        }
    }
    pub(crate) fn valid(&self, received: i64) -> bool {
        self.issue(received).is_none()
    }
    pub(crate) fn issue(&self, received: i64) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let payments = self.payments();
        if payments.is_empty() {
            return Some(ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL").at_field("payments"));
        }
        for (index, p) in payments.iter().enumerate() {
            if p.source <= 0 {
                return Some(
                    ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL")
                        .at_field("source")
                        .in_list("payments", index),
                );
            }
            if p.amount <= 0 {
                return Some(
                    ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL")
                        .at_field("amount")
                        .in_list("payments", index),
                );
            }
            if payments[..index].iter().any(|v| v.source == p.source) {
                return Some(
                    ValidationIssue::new("INJURED_LTC_SOURCE_TWICE")
                        .at_field("source")
                        .in_list("payments", index),
                );
            }
        }
        (crate::kernel::ratio::checked_sum(payments.iter().map(|p| p.amount)) != Some(received))
            .then_some(ValidationIssue::new("INJURED_LTC_PAYEE_TOTAL").at_field("payments"))
    }
}

/// A payment from an existing long-term-care statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredLtcPeriodPayment {
    /// The statement whose policyholder received this payment
    pub source: i64,
    /// Actual periodic payment received in these dates
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
}

/// Actual source payments in a long-term-care period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredLtcPeriodPayeesInput {
    /// Payments identified by their existing 1099-LTC source
    Payments {
        /// Each source's actual periodic payment in these dates
        #[libertas_size(min = 1)]
        payments: Vec<InjuredLtcPeriodPaymentInput>,
    },
}

/// A payment from an existing long-term-care statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredLtcPeriodPaymentInput {
    /// The statement whose policyholder received this payment
    #[libertas_enum_source("$.ltc_payee_sources")]
    pub source: i64,
    /// Actual periodic payment received in these dates
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
}

impl From<InjuredLtcPeriodPayeesInput> for InjuredLtcPeriodPayees {
    fn from(value: InjuredLtcPeriodPayeesInput) -> Self {
        match value {
            InjuredLtcPeriodPayeesInput::Payments { payments } => Self::Payments {
                payments: payments
                    .into_iter()
                    .map(|p| InjuredLtcPeriodPayment { source: p.source, amount: p.amount })
                    .collect(),
            },
        }
    }
}
impl From<InjuredLtcPeriodPayees> for InjuredLtcPeriodPayeesInput {
    fn from(value: InjuredLtcPeriodPayees) -> Self {
        match value {
            InjuredLtcPeriodPayees::Payments { payments } => Self::Payments {
                payments: payments
                    .into_iter()
                    .map(|p| InjuredLtcPeriodPaymentInput { source: p.source, amount: p.amount })
                    .collect(),
            },
        }
    }
}
