// Coverage: see charity.rs (Unit15; original T3-deduct, 2026-09-28): whaling captains are asked on the itemized
//   hub's gifts of property and carryovers page.
// Section 170(n) whaling expenses paid by a recognized Native Alaskan captain.
// https://www.govinfo.gov/content/pkg/PLAW-119publ21/html/PLAW-119publ21.htm
// https://www.irs.gov/irb/2006-47_IRB#RP-2006-50
use crate::Owner;
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How a boat paid for in 2026 was used
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WhalingBoatUse {
    /// The boat was used only for sanctioned whaling activities during 2026
    Exclusive,
    /// The boat was also used for other activities during 2026
    Mixed {
        /// Days used for sanctioned whaling in the 365-day 2026 tax year
        // Tax structure: 2026 has 365 days for allocating this boat's sanctioned use.
        #[libertas_number(min = 1, max = 365)]
        sanctioned_days: u32,
    },
}

/// Reasonable and necessary expense paid for sanctioned subsistence bowhead whaling
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WhalingExpenseKind {
    /// Acquisition or maintenance of a whaling boat, allocated if also used for other activities
    Boat { use_mode: WhalingBoatUse },
    /// Weapons and gear used in sanctioned whaling activities
    WeaponsAndGear,
    /// Crew food and other provisions for sanctioned whaling activities
    CrewAndProvisions,
    /// Storage and distribution of the sanctioned catch
    CatchStorageAndDistribution,
}

/// Documentary payment evidence, or reconstruction after records were lost beyond the captain's control
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WhalingPaymentEvidence {
    /// Receipt, bill, invoice or credit-card charge record
    Original {
        #[libertas_size(min = 1)]
        record_reference: String,
    },
    /// Reasonable reconstruction after a fire, flood, earthquake or comparable loss
    Reconstructed {
        /// Circumstances outside the captain's control that destroyed previously maintained records
        #[libertas_size(min = 1)]
        loss_circumstances: String,
        /// Reference to the reasonable reconstruction and its supporting evidence
        #[libertas_size(min = 1)]
        reconstruction_reference: String,
    },
}

impl WhalingPaymentEvidence {
    fn issue(&self) -> Option<ValidationIssue> {
        let issue = |field| ValidationIssue::new(VALIDATION_KEY).at_field(field);
        match self {
            Self::Original { record_reference } => {
                if !meaningful(record_reference) {
                    return Some(issue("record_reference"));
                }
            }
            Self::Reconstructed { loss_circumstances, reconstruction_reference } => {
                if !meaningful(loss_circumstances) {
                    return Some(issue("loss_circumstances"));
                }
                if !meaningful(reconstruction_reference) {
                    return Some(issue("reconstruction_reference"));
                }
            }
        }
        None
    }
}

/// One 2026 expense from the captain's contemporaneous whaling expense report
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WhalingExpense {
    /// Date this expense was paid in 2026
    #[libertas_date_only]
    pub paid_date: u32,
    /// Place or area of the sanctioned activity or related purchase
    #[libertas_size(min = 1)]
    pub place: String,
    /// Nature and sanctioned-whaling purpose of this expense
    #[libertas_size(min = 1)]
    pub description: String,
    /// Statutory expense category
    pub kind: WhalingExpenseKind,
    /// Unreimbursed amount paid, before any mixed-use boat allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub paid: i64,
    /// Reference to the contemporaneous expense report or equivalent record
    #[libertas_size(min = 1)]
    pub report_reference: String,
    /// Payment evidence for a separate expense of $75 or more
    /// Retain original records or a reasonable reconstruction when previously
    /// maintained evidence was destroyed by circumstances beyond your control.
    /// [NullText]
    /// This separate expense is below $75
    /// [SetText]
    /// Payment evidence for this separate expense is
    pub payment_evidence: Option<WhalingPaymentEvidence>,
}

impl WhalingExpense {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let issue = |field| ValidationIssue::new(VALIDATION_KEY).at_field(field);
        if !(20260101..=20261231).contains(&self.paid_date) || !valid_date(self.paid_date) {
            return Some(issue("paid_date"));
        }
        for (field, value) in [
            ("place", &self.place),
            ("description", &self.description),
            ("report_reference", &self.report_reference),
        ] {
            if !meaningful(value) {
                return Some(issue(field));
            }
        }
        if !(1..=100_000_000_000).contains(&self.paid) {
            return Some(issue("paid"));
        }
        if self.paid >= 7_500 && self.payment_evidence.is_none() {
            return Some(issue("payment_evidence"));
        }
        if let Some(issue) = self.payment_evidence.as_ref().and_then(WhalingPaymentEvidence::issue)
        {
            return Some(issue.in_field("payment_evidence"));
        }
        if let WhalingExpenseKind::Boat { use_mode: WhalingBoatUse::Mixed { sanctioned_days } } =
            &self.kind
            && !(1..=365).contains(sanctioned_days)
        {
            return Some(issue("sanctioned_days").in_field("use_mode").in_field("kind"));
        }
        None
    }

    fn allowable_cents(&self) -> i64 {
        match &self.kind {
            WhalingExpenseKind::Boat { use_mode: WhalingBoatUse::Mixed { sanctioned_days } } => {
                crate::kernel::ratio::trunc_i64(self.paid * i64::from(*sanctioned_days), 365)
            }
            _ => self.paid,
        }
    }
}

/// Itemizer-only section 170(n) claim for one AEWC-recognized whaling captain
/// The 2026 statutory cap is $50,000. This deemed contribution is not a direct gift
/// to a public charity, so it belongs to section 170(b)(1)(B)'s 30%-of-AGI
/// class, not the 60% cash-gift class or the nonitemizer deduction. Percentage-
/// limit excess may carry under the ordinary charitable carryover rules.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WhalingCaptainClaim {
    /// Recognized captain claiming these expenses on this return
    pub owner: Owner,
    /// AEWC captain recognition document reference for this taxpayer
    #[libertas_size(min = 1)]
    pub captain_record_reference: String,
    /// Copies of the captain's 2026 activity reports submitted to the AEWC
    #[libertas_size(min = 1)]
    pub aewc_report_reference: String,
    /// Contemporaneous log of each sanctioned trip's departure, return and area
    #[libertas_size(min = 1)]
    pub trip_log_reference: String,
    /// 2026 reasonable and necessary expenses, each with its own amount and records
    pub expenses: Vec<WhalingExpense>,
    /// Confirm AEWC recognition, actual 2026 sanctioned activity, complete logs,
    /// unreimbursed costs and no deduction under another tax provision
    /// Check your AEWC recognition, submitted reports, trip logs and payment records to answer. No leaves these expenses undeducted.
    pub eligibility_and_records_confirmed: bool,
}

impl WhalingCaptainClaim {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let issue = |field| ValidationIssue::new(VALIDATION_KEY).at_field(field);
        for (field, value) in [
            ("captain_record_reference", &self.captain_record_reference),
            ("aewc_report_reference", &self.aewc_report_reference),
            ("trip_log_reference", &self.trip_log_reference),
        ] {
            if !meaningful(value) {
                return Some(issue(field));
            }
        }
        // Temporary arithmetic exclusion: eligible_expense_cents sums native i64
        // expenses before the statutory cap and the federal detail output.
        if self.expenses.is_empty() || self.expenses.len() > 200 {
            return Some(issue("expenses"));
        }
        for (index, expense) in self.expenses.iter().enumerate() {
            if let Some(issue) = expense.issue() {
                return Some(issue.in_list("expenses", index));
            }
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }

    /// Section 170(n) annual limit, in cents; later AGI and itemizer limits apply.
    pub(crate) fn eligible_expense_cents(&self) -> i64 {
        self.expenses.iter().map(WhalingExpense::allowable_cents).sum::<i64>()
    }

    /// Section 170(n) annual limit, in cents; later AGI and itemizer limits apply.
    pub(crate) fn statutory_claim_cents(&self) -> i64 {
        phase::cap(self.eligible_expense_cents(), [5_000_000])
    }
}

fn meaningful(value: &str) -> bool {
    !value.trim().is_empty()
}

pub(crate) const VALIDATION_KEY: &str =
    "MESSAGE_CHECK_EACH_WHALING_EXPENSE_A_RETURN_YEAR_DATE_ITS_PLACE_AND_DESCRIPTION_AND_PAYMENT";

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn expense(kind: WhalingExpenseKind, paid: i64) -> WhalingExpense {
        WhalingExpense {
            paid_date: 20260615,
            place: "Utqiagvik, Alaska".to_string(),
            description: "Sanctioned subsistence bowhead whaling".to_string(),
            kind,
            paid,
            report_reference: "expense-log-2026".to_string(),
            payment_evidence: Some(WhalingPaymentEvidence::Original {
                record_reference: "receipt-2026".to_string(),
            }),
        }
    }

    fn claim(expenses: Vec<WhalingExpense>) -> WhalingCaptainClaim {
        WhalingCaptainClaim {
            owner: Owner::Taxpayer,
            captain_record_reference: "AEWC-recognition".to_string(),
            aewc_report_reference: "AEWC-2026-reports".to_string(),
            trip_log_reference: "trip-log-2026".to_string(),
            expenses,
            eligibility_and_records_confirmed: true,
        }
    }

    #[test]
    fn enacted_2026_cap_and_separate_expense_categories() {
        let forty_thousand = claim(vec![
            expense(WhalingExpenseKind::WeaponsAndGear, 1_000_000),
            expense(WhalingExpenseKind::CrewAndProvisions, 2_000_000),
            expense(WhalingExpenseKind::CatchStorageAndDistribution, 1_000_000),
        ]);
        assert!(forty_thousand.valid());
        assert!(forty_thousand.eligibility_and_records_confirmed);
        assert_eq!(forty_thousand.statutory_claim_cents(), 4_000_000);

        let over_cap = claim(vec![expense(WhalingExpenseKind::WeaponsAndGear, 5_000_001)]);
        assert_eq!(over_cap.statutory_claim_cents(), 5_000_000);
    }

    #[test]
    fn mixed_use_boat_uses_sanctioned_days_over_tax_year_days() {
        let exclusive = claim(vec![expense(
            WhalingExpenseKind::Boat { use_mode: WhalingBoatUse::Exclusive },
            365_000,
        )]);
        let mixed = claim(vec![expense(
            WhalingExpenseKind::Boat { use_mode: WhalingBoatUse::Mixed { sanctioned_days: 100 } },
            365_000,
        )]);
        assert_eq!(exclusive.statutory_claim_cents(), 365_000);
        assert_eq!(mixed.statutory_claim_cents(), 100_000);
    }

    #[test]
    fn requires_contemporaneous_records_and_large_expense_receipts() {
        let mut small = expense(WhalingExpenseKind::CrewAndProvisions, 7_499);
        small.payment_evidence = None;
        assert!(small.issue().is_none());
        small.paid = 7_500;
        assert!(small.issue().is_some());
        small.payment_evidence =
            Some(WhalingPaymentEvidence::Original { record_reference: "receipt".to_string() });
        assert!(small.issue().is_none());
        small.payment_evidence = Some(WhalingPaymentEvidence::Reconstructed {
            loss_circumstances: "Expense records lost in a flood".to_string(),
            reconstruction_reference: "contemporaneous-bank-and-AEWC-records".to_string(),
        });
        assert!(small.issue().is_none());
        small.paid_date = 20260230;
        assert!(small.issue().is_some());

        let mut unsupported = claim(vec![expense(WhalingExpenseKind::WeaponsAndGear, 100_000)]);
        unsupported.eligibility_and_records_confirmed = false;
        assert!(unsupported.valid());
        assert!(!unsupported.eligibility_and_records_confirmed);
        unsupported.captain_record_reference.clear();
        assert!(!unsupported.valid());
    }
}
