// Coverage: alimony under pre-2019 instruments, Schedule 1 lines 2a/2b and 19a/19c (Pub. 504).
//   One AlimonyPayment record serves both sides: the payer's row on More adjustments
//   (AlimonyPaid, u5-adjust) and the recipient's "Alimony received" kind on the Prizes,
//   royalties, equipment rent and alimony income topic (FederalIncome::Alimony, u6-alimony,
//   2026-09-28). Current payments: instrument executed on or before 2018-12-31 (Form 1040
//   instructions, line 2a), child support satisfied first, no more than the alimony due;
//   an instrument changed before 2026 to adopt the post-2018 rules gives nothing
//   (AlimonyNewRules::BeforeReturnYear). Third-year recapture (Pub. 504 Worksheet 1, $15,000 floors,
//   excluded temporary-order and fixed-share payments, death or remarriage exception): payer
//   income on line 2a, recipient deduction on line 19a. Refused by name: pre-1985 instruments,
//   recapture for pre-1987 instruments, and (u6-alimony) an
//   instrument changed or replaced during 2026, whose year splits at the change (Pub. 504
//   Examples 2 and 3). Correction (u6-alimony): the former `new_law: bool` treated a 2026
//   change as covering the whole year, silently dropping the alimony paid before the change;
//   the three-way answer refuses that case by name. Client and server rules
//   ALIMONY_2024/2025_EXCLUDED_ABOVE_PAID live on this record (both pages). An unconfirmed
//   payment (No, 2026-10-05) is not deducted by the payer and is taxed to the
//   recipient, with recapture income kept for the payer; Tests:
//   src/tests/family/alimony_received.rs, src/tests/federal/unasked_pages.rs, src/tests/family/family.rs alimony_*.
use crate::federal_rules::valid_birth;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalResult, Owner};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Alimony or separate maintenance payment record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct AlimonyPayment {
    /// Payment is under a divorce or separate-maintenance decree, or a written instrument incident to that decree
    /// Choose false for a standalone written separation agreement or temporary support order. Only decree-related taxable alimony supplies IRA compensation.
    pub decree: bool,
    /// Third-year recapture history
    pub recapture: AlimonyRecapture,
    /// Original divorce or separation instrument date
    #[libertas_date_only]
    pub agreement: u32,
    /// Changed to the post-2018 rules
    /// A change to the payment amounts alone keeps the pre-2019 rules. Choose a change only when a later written change expressly says the alimony is not deductible or taxable, or a divorce decree signed after 2018 replaced this agreement.
    #[libertas_default(No)]
    pub new_rules: AlimonyNewRules,
    /// Cash actually paid or received in 2026 under this instrument
    /// Include qualifying direct payments to third parties and child support paid under the same instrument. Exclude property transfers and payments already entered elsewhere.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Alimony required for these payments under the instrument
    /// Exclude voluntary additional gifts. Use the obligations satisfied by these payments, including identified arrears; do not count an obligation twice.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub alimony_due: i64,
    /// Child support required before these payments can satisfy alimony
    /// Include identified child-support arrears that take priority. If total payments fall short, the App applies them to child support first.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub child_support_due: i64,
    /// Ordinary qualifying cash alimony, with any recapture history entered above
    /// Payer and recipient do not file jointly with each other; payment is required, ends at the recipient's death and is not child support, property settlement or community income. Legally separated parties live in separate households. No non-taxable designation, unentered recapture history, pre-1985 special rule or foreign/treaty treatment. No: the payer deducts nothing and the recipient is taxed on the payment as alimony.
    /// Check the divorce instrument, payment records and retained recapture returns.
    pub eligible: bool,
}
impl AlimonyPayment {
    pub(crate) fn valid(&self) -> bool {
        self.validation_issue("").is_none()
    }
    pub(crate) fn validation_issue(
        &self,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.recapture.validation_issue(self.amount(), key) {
            return Some(issue.in_field("recapture"));
        }
        if !valid_birth(self.agreement)
            || self.agreement > 20261231
            || ![self.paid, self.alimony_due, self.child_support_due]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
        {
            return Some(crate::validation::ValidationIssue::new(key));
        }
        None
    }
    pub(crate) fn amount(&self) -> i64 {
        if self.agreement >= 20190101 || self.new_rules == AlimonyNewRules::BeforeReturnYear {
            0
        } else {
            phase::cap(positive(self.paid - self.child_support_due), [self.alimony_due])
        }
    }
    // `side` names the page the refusal opens: "Alimony paid" or "Alimony
    // received". An unconfirmed payment (No) takes each side's
    // no-benefit path: the payer deducts nothing, the recipient is taxed.
    pub(crate) fn checked(
        &self,
        r: &mut FederalResult,
        context: &crate::IssueContext,
        payer: bool,
        #[cfg(test)] side: &str,
    ) -> i64 {
        let pre_2019 = self.agreement < 20190101;
        crate::require_localized!(
            r,
            !pre_2019 || self.new_rules != AlimonyNewRules::DuringReturnYear,
            &context.clone().manual(),
            side,
            "ISSUE_AN_AGREEMENT_CHANGED_DURING_RETURN_YEAR_TO_THE_POST_2018_RULES_OR_REPLACED_F80A1653",
            "An agreement changed during 2026 to the post-2018 rules, or replaced during 2026 by a divorce decree signed after 2018, splits this year's payments at the change (Pub. 504): Schedule 1 line 2a or 19a for such a year is not calculated yet.",
        );
        let ordinary = pre_2019 && self.new_rules == AlimonyNewRules::No;
        crate::require_localized!(
            r,
            !ordinary || self.agreement >= 19850101,
            &context.clone().manual(),
            side,
            "ISSUE_CONFIRM_ORDINARY_QUALIFYING_ALIMONY_AND_RESOLVE_RECAPTURE_PRE_1985_DABB0528",
            "Alimony under an instrument executed before 1985 follows special rules that are not calculated.",
        );
        if ordinary && !self.eligible && payer {
            return 0;
        }
        self.amount()
    }
}

/// Changed to the post-2018 rules
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AlimonyNewRules {
    /// No, the pre-2019 rules still apply
    No,
    /// Yes, before 2026
    /// Nothing paid in 2026 under this agreement is deductible or taxable.
    BeforeReturnYear,
    /// Yes, during 2026
    /// Payments before the change still follow the pre-2019 rules.
    DuringReturnYear,
}

/// Alimony recapture in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AlimonyRecapture {
    /// Not the third year of qualifying payments
    /// The first year excludes temporary support orders. The next two calendar years count even if payments stop. Choose the worksheet if first qualifying payments began in 2024.
    None,
    /// First qualifying payments began in 2024
    ThirdYear {
        /// Qualifying alimony actually paid in 2024
        /// Use prior payment/return records after child-support priority, not the amount originally promised.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        first: i64,
        /// Qualifying alimony actually paid in 2025
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        second: i64,
        /// Part of 2024 payments excluded from recapture testing
        /// Only temporary support orders or payments required for at least three calendar years that vary as a fixed part of business/property income or employment/self-employment compensation.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        first_excluded: i64,
        /// Part of 2025 payments excluded under those same rules
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        second_excluded: i64,
        /// Part of this year's qualifying alimony excluded under those same rules
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        current_excluded: i64,
        /// Payments ceased because of either spouse's death or the recipient's remarriage before 2026 ended
        /// These causes are exempt. A change in support needs, payment default, agreement reduction or inability to pay is not exempt. The death certificate or remarriage date shows it.
        death_or_remarriage: bool,
    },
}
impl AlimonyRecapture {
    fn validation_issue(
        &self,
        current: i64,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::None => None,
            Self::ThirdYear {
                first,
                second,
                first_excluded,
                second_excluded,
                current_excluded,
                ..
            } => {
                use crate::validation::ValidationIssue;
                if ![*first, *second].iter().all(|v| (0..=100000000000).contains(v)) {
                    return Some(ValidationIssue::new(key));
                }
                for (value, total, field) in [
                    (*first_excluded, *first, "first_excluded"),
                    (*second_excluded, *second, "second_excluded"),
                    (*current_excluded, current, "current_excluded"),
                ] {
                    if !(0..=total).contains(&value) {
                        let issue = ValidationIssue::new(key);
                        return Some(if value > total { issue.at_field(field) } else { issue });
                    }
                }
                None
            }
        }
    }
}
impl AlimonyPayment {
    // `side` names the page the refusal opens: "Alimony recapture (paid)"
    // or "Alimony recapture (received)". For an unconfirmed payment the payer
    // still includes recapture and the recipient deducts none.
    pub(crate) fn recaptured(
        &self,
        r: &mut FederalResult,
        context: &crate::IssueContext,
        payer: bool,
        #[cfg(test)] side: &str,
    ) -> i64 {
        let AlimonyRecapture::ThirdYear {
            first,
            second,
            first_excluded,
            second_excluded,
            current_excluded,
            death_or_remarriage,
        } = self.recapture
        else {
            return 0;
        };
        crate::require_localized!(
            r,
            (19870101..20190101).contains(&self.agreement) && self.new_rules == AlimonyNewRules::No,
            &context.clone().manual(),
            side,
            "ISSUE_CONFIRM_THE_THIRD_YEAR_PAYMENT_HISTORY_AND_EXCEPTION_PRE_1987_INSTR_F7A45E5F",
            "Third-year recapture for a pre-1987 instrument, or one changed to the post-2018 rules during the recapture period, needs separate review.",
        );
        if death_or_remarriage || (!payer && !self.eligible) {
            return 0;
        }
        let first = dollars(first - first_excluded);
        let second = dollars(second - second_excluded);
        let current = dollars(self.amount() - current_excluded);
        let second_excess = positive(second - current - 15000);
        let first_excess =
            positive(first - fraction(second - second_excess + current, 1, 2) - 15000);
        (second_excess + first_excess) * 100
    }
}

/// Alimony paid by one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AlimonyPaid {
    /// Filer who paid the alimony
    pub owner: Owner,
    /// Payment and agreement facts
    pub payment: AlimonyPayment,
}
