use crate::Owner;
use crate::dates::valid_birth;
use crate::kernel::phase;
use crate::kernel::ratio::positive;
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Alimony required for these payments under the instrument
    /// Exclude voluntary additional gifts. Use the obligations satisfied by these payments, including identified arrears; do not count an obligation twice.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub alimony_due: i64,
    /// Child support required before these payments can satisfy alimony
    /// Include identified child-support arrears that take priority. If total payments fall short, the App applies them to child support first.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        first: i64,
        /// Qualifying alimony actually paid in 2025
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        second: i64,
        /// Part of 2024 payments excluded from recapture testing
        /// Only temporary support orders or payments required for at least three calendar years that vary as a fixed part of business/property income or employment/self-employment compensation.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        first_excluded: i64,
        /// Part of 2025 payments excluded under those same rules
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        second_excluded: i64,
        /// Part of this year's qualifying alimony excluded under those same rules
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
/// Alimony paid by one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AlimonyPaid {
    /// Filer who paid the alimony
    pub owner: Owner,
    /// Payment and agreement facts
    pub payment: AlimonyPayment,
}
