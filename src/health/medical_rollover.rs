// Indirect medical-account rollovers use the existing statement and contribution
// owners. Section223(f)(5) counts prior successful HSA-source receipts; section
//220(f)(5) separately counts Archer-source receipts. Current-year history is
// recomputed across accepted sources, never from editor order or a second ledger.
use crate::kernel::ratio::exact::{Exact, whole};
use crate::{MedicalSaAccount, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual indirect-rollover source receipts
/// Keep one actual annual statement as one record. Separate source receipts and deposit dates only where they differ. Direct trustee transfers and IRA-to-HSA funding are different transactions and do not belong here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MedicalRollover {
    /// Last successful prior-year rollover receipt from this source account type
    /// The actual most recent tax-free receipt before 2026, across all this owner's HSAs or, separately, Archer MSAs. Failed attempts, direct transfers and IRA-to-HSA funding do not count. Leave empty when no prior successful receipt can affect 2026. Use the same prior-year history on every statement for this owner/source type; the App derives 2026 history from actual entries.
    /// [NullText]
    /// No successful prior-year rollover affects 2026
    /// [SetText]
    /// The last relevant successful prior-year rollover receipt was on
    #[libertas_date_only]
    // Tax structure: retained prior-year history ends before the current return year.
    #[libertas_number(max = 20251231)]
    pub preceding: Option<u32>,
    /// Actual source receipts on this statement
    #[libertas_size(min = 1)]
    /// ----
    /// Receipt
    pub receipts: Vec<MedicalRolloverReceipt>,
}

/// One actual indirect-rollover source receipt
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MedicalRolloverReceipt {
    /// Actual distribution receipt date
    /// This is the source payment date for additional-tax exceptions. Do not combine separately dated payments into the last date on an annual form.
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub received: u32,
    /// Actual destination deposits from this receipt
    /// Together with medical spending and other dated nonmedical uses, all deposits reconcile to this statement's gross. Separate day 60 from day 61; no second receipt total is entered.
    #[libertas_size(min = 1)]
    /// ----
    /// Deposit
    pub deposits: Vec<MedicalRolloverDeposit>,
}

/// Actual destination account type
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MedicalRolloverDestination {
    /// Same owner's HSA
    Hsa,
    /// Same owner's Archer MSA
    Archer,
}

/// Actual deposit if rollover exclusion is unavailable
/// This records actual custodian treatment, not a request to invent a contribution designation. A failed cash transfer can use ordinary contribution room; it is not automatically all excess. Property is not an ordinary cash contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalDepositCharacter {
    /// Not established; leave only if rollover relief applies
    Unconfirmed,
    /// Actual ordinary cash contribution designation
    Cash {
        /// Custodian-accepted ordinary contribution year
        /// Use the actual designation, including 2025 or 2027 where applicable. Do not infer it from the source year or intended rollover label. Only 2026-designated cash enters current contribution limits.
        #[libertas_number(min = 1997)]
        year: u32,
        /// Actual applicable ordinary contribution deadline
        /// Normally April 15 of the following year. A later date needs actual applicable IRS relief; an income-tax filing extension alone does not extend ordinary contribution timing.
        #[libertas_date_only]
        deadline: u32,
    },
    /// Property, not cash
    /// Property can qualify for rollover relief, but a failed property deposit needs separate account-qualification treatment before a complete estimate can be made.
    Property,
}

/// One actual indirect-rollover deposit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MedicalRolloverDeposit {
    /// Actual deposit date
    #[libertas_date_only]
    pub deposited: u32,
    /// Actual deposited amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Destination belonging to this same owner
    pub destination: MedicalRolloverDestination,
    /// Actual character if rollover exclusion fails
    #[libertas_default(Unconfirmed)]
    pub character: MedicalDepositCharacter,
}

impl MedicalRolloverDeposit {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let MedicalDepositCharacter::Cash { year, deadline } = self.character {
            if !(u64::from(year)..=u64::from(year) + 1).contains(&u64::from(self.deposited / 10000))
            {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_YEAR_INVALID").at_field("deposited"),
                );
            }
            if deadline / 10000 < year {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_YEAR_INVALID")
                        .at_field("deadline")
                        .in_field("character"),
                );
            }
            if self.deposited > deadline {
                return Some(
                    ValidationIssue::new("MEDICAL_DEPOSIT_AFTER_DEADLINE").at_field("deposited"),
                );
            }
        }
        None
    }
    fn valid(&self) -> bool {
        crate::dates::source_valid_date(self.deposited)
            && self.amount > 0
            && match self.character {
                MedicalDepositCharacter::Cash { year, deadline } => {
                    (1997..=2099).contains(&year)
                        && (year..=year + 1).contains(&(self.deposited / 10000))
                        && crate::dates::source_valid_date(deadline)
                        && deadline / 10000 >= year
                        && self.deposited <= deadline
                }
                _ => true,
            }
    }
    pub(crate) fn timely(&self, received: u32, source: &MedicalSaAccount) -> bool {
        crate::dates::days_between(received, self.deposited) <= 60
            && (matches!(source, MedicalSaAccount::Archer)
                || (matches!(source, MedicalSaAccount::Hsa)
                    && self.destination == MedicalRolloverDestination::Hsa))
    }
}
impl MedicalRollover {
    pub(crate) fn amount(&self) -> Exact {
        self.receipts.iter().flat_map(|p| &p.deposits).map(|d| whole(d.amount)).sum()
    }
    pub(crate) fn valid(&self) -> bool {
        self.preceding
            .is_none_or(|d| crate::dates::valid_birth(d) && d < crate::TAX_YEAR_START_DATE)
            && !self.receipts.is_empty()
            && self.receipts.iter().all(|p| {
                crate::dates::source_valid_date(p.received)
                    && (crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE).contains(&p.received)
                    && !p.deposits.is_empty()
                    && p.deposits.iter().all(|d| d.valid() && d.deposited >= p.received)
            })
    }
    // Local source checking only. The coordinated estimate first resolves actual
    // current-year history and normalizes excluded/failed parts on a working copy.
    pub(crate) fn eligible(&self, source: &MedicalSaAccount) -> bool {
        self.receipts.iter().all(|p| {
            self.preceding.is_none_or(|last| p.received >= last + 10000)
                && p.deposits.iter().all(|d| d.timely(p.received, source))
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FailedDeposit {
    pub source: i64,
    pub owner: Owner,
    pub received: u32,
    pub deposit: MedicalRolloverDeposit,
}
#[derive(Default)]
pub(crate) struct Projection {
    pub failed: Vec<FailedDeposit>,
}
// The pair is actual accepted source identity plus a named required source issue.
pub(crate) type Failure = (i64, &'static str);

// Actual failed cash deposits are a calculation input to the existing annual
// contribution owner, not new manual answers on its persisted record.
#[derive(Clone, Default)]
pub(crate) struct Contribution {
    pub personal: Exact,
    pub later: Exact,
}

#[derive(Clone, Default)]
pub(crate) struct Contributions {
    pub hsa: [Contribution; 2],
    pub archer: [Contribution; 2],
}

/// Original failed-rollover source available for a corrective return
/// These actual source facts are supplied read-only. Other manual contribution totals exclude these deposits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthFailedDepositChoice {
    /// Original accepted statement identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Actual original statement
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Actual contribution owner
    #[libertas_read_only]
    pub owner: Owner,
    /// Actual failed cash deposits
    #[libertas_read_only]
    pub deposits: Vec<HealthFailedDepositFact>,
}
/// Actual failed cash contribution facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthFailedDepositFact {
    /// Actual deposit date
    #[libertas_date_only]
    #[libertas_read_only]
    pub deposited: u32,
    /// Actual ordinary contribution year
    #[libertas_read_only]
    pub year: u32,
    /// Actual destination
    #[libertas_read_only]
    pub destination: MedicalRolloverDestination,
    /// Original deposited principal before corrective returns
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub amount: i64,
}
