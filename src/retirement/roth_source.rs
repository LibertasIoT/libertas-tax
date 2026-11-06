use crate::dates::source_valid_date as valid_date;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Roth IRA withdrawal
/// Enter each actual distribution date separately. Combine all this owner's Roth IRA history in Adjustments. A Roth IRA, Roth SEP IRA or Roth SIMPLE IRA own-account withdrawal: an inherited payment uses its linked beneficiary/deceased-owner history. No unentered HSA funding or QCD. Do not enter the same payment as traditional retirement income.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RothWithdrawal {
    /// Additional printed retirement-statement boxes
    pub form_boxes: crate::RetirementPrintedBoxes,
    /// Form 1099-R box 2a taxable amount as printed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_2a: i64,
    /// Form 1099-R box 3 capital gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_3: i64,
    /// Form 1099-R box 6 net unrealized appreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_6: i64,
    /// Form 1099-R box 8a other
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_8a: i64,
    /// Form 1099-R box 8b share
    /// [NullText]
    /// Box 8b is blank
    /// [SetText]
    /// Printed annuity-contract share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Form 1099-R box 9a share
    /// [NullText]
    /// Box 9a is blank
    /// [SetText]
    /// Printed distribution share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Form 1099-R box 9b employee contributions
    /// [NullText]
    /// Box 9b is blank
    /// [SetText]
    /// Printed employee contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9b: Option<i64>,
    /// Form 1099-R box 10 in-plan Roth rollover amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_10: i64,
    /// Form 1099-R box 7a distribution codes as printed
    #[libertas_bitflags(crate::DistributionCodes)]
    pub box7: u32,
    /// Form 1099-R box 7b IRA/SEP/SIMPLE as printed
    pub ira: bool,
    /// Form 1099-R box 7c Trump account as printed
    pub trump: bool,

    /// Distribution date in 2026
    #[libertas_date_only]
    pub distributed: u32,
    /// Actual order among this pool's payments on the same date
    /// Needed only when qualified HSA funding shares the date with another payment. Use the actual statement order, not the order of entries in this App. Leave absent when no funding shares the date.
    #[libertas_number(min = 1)]
    #[libertas_default($null)]
    pub sequence: Option<u32>,
    /// Gross distribution before withholding
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Amount rolled over to another Roth IRA within 60 days
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rolled_over: i64,
    /// Federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Applied Roth treatment selected on the Retirement topic
    /// Kept independently of the current printed distribution codes.
    pub code: RothCode,
    /// Exception to the 10% additional tax
    /// Disability from code T's reason or code J's exception; any IRA exception from code J.
    #[libertas_default($null)]
    pub exception: Option<crate::RetirementException>,
    /// Repaid to a Roth IRA
    #[libertas_default($null)]
    pub repaid: Option<crate::ExceptionRepayment>,
    /// Qualified disaster distribution
    #[libertas_default(No)]
    pub disaster: crate::DisasterDistribution,
    /// Inherited Roth IRA
    /// Code T for a beneficiary: the Inherited IRA record that holds the original owner's five-year period and allocated basis decides the taxable amount of every payment that names it.
    pub inherited: InheritedLink,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for RothWithdrawal {}

// The 1099-R onto the internal withdrawal. Box 7b with J, Q or T is a Roth SIMPLE IRA
// after its two years (or a Roth SEP IRA); J printed with S (box 7b) is a Roth SIMPLE IRA
// inside its first two years, taxed at 25%. A qualified distribution's date is never
// needed. Codes printed with J (8J, PJ) never reach here: box 7a leads them to the
// corrective or not-income branches.
impl From<&crate::R1099Source> for RothWithdrawal {
    fn from(source: &crate::R1099Source) -> Self {
        let crate::R1099Source {
            form,
            code,
            // A Roth row refuses every other answer (R_ROTH_OTHER_BOXES).
            taxable_method: _,
            disability: _,
            box_10_basis: _,
            disaster: _,
            after_death: _,
            ira_basis: _,
        } = source;
        let Some(crate::DistributionCode::RothIra { roth, rolled_over, sequence }) = code else {
            unreachable!("only a Roth IRA code files a 1099-R as a Roth withdrawal")
        };
        let crate::R1099 {
            payer: _,
            payer_telephone: _,
            account_number: _,
            recipient: _,
            gross,
            taxable,
            // Printed 2b, 5, 7d, 11, 12 and 13 are retained through form_boxes.
            taxable_not_determined: _,
            total_distribution: _,
            box_3,
            withholding,
            employee_contributions: _,
            box_6,
            box7,
            ira,
            trump,
            excess_contribution_earnings: _,
            box_8a,
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10,
            first_roth_year: _,
            fatca: _,
            payment_date: _,
            state_details: _,
        } = form;
        let (code, distributed, exception, repaid, inherited) = match roth {
            RothIraCode::Qualified => {
                (RothCode::Qualified, 20261231, None, None, InheritedLink::No)
            }
            RothIraCode::Early { distributed, exception, repaid } => (
                RothCode::Early,
                *distributed,
                exception.clone(),
                repaid.clone(),
                InheritedLink::No,
            ),
            RothIraCode::ExceptionApplies { distributed, reason } => (
                RothCode::ExceptionApplies,
                *distributed,
                match reason {
                    RothReason::Disabled { since } => {
                        Some(crate::RetirementException::Disability { began: *since })
                    }
                    _ => None,
                },
                None,
                match reason {
                    RothReason::Inherited { account } => InheritedLink::Yes { account: *account },
                    _ => InheritedLink::No,
                },
            ),
        };
        RothWithdrawal {
            form_boxes: form.form_boxes(),
            distributed,
            sequence: *sequence,
            gross: *gross,
            rolled_over: *rolled_over,
            withholding: withholding.unwrap_or(0),
            code,
            exception,
            repaid,
            box_2a: taxable.unwrap_or(0),
            box_3: box_3.unwrap_or(0),
            box_6: box_6.unwrap_or(0),
            box_8a: box_8a.unwrap_or(0),
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            box_9b: *box_9b,
            box_10: box_10.unwrap_or(0),
            box7: *box7,
            ira: *ira,
            trump: *trump,
            disaster: source.disaster().clone(),
            inherited,
        }
    }
}
/// Inherited Roth IRA
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InheritedLink {
    /// Your own Roth IRA
    No,
    /// An inherited Roth IRA
    Yes {
        /// Inherited IRA record
        /// Absent uses the filer's only inherited Roth IRA record.
        #[libertas_number(min = 1)]
        account: Option<i64>,
    },
}
/// Form 1099-R box 7a code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RothCode {
    /// Q — qualified distribution
    Qualified,
    /// J — early distribution, no known exception
    Early,
    /// T — exception applies
    ExceptionApplies,
}
impl RothWithdrawal {
    pub(crate) fn simple_early(&self) -> bool {
        self.box7 & crate::DistributionCodes::CODE_S.0 != 0
    }
    pub(crate) fn valid(&self) -> bool {
        self.valid_with_source_alignment(true)
    }
    pub(crate) fn valid_with_source_alignment(&self, source_alignment: bool) -> bool {
        self.sequence.is_none_or(|v| (1..=100).contains(&v))
            && valid_date(self.distributed)
            && self.distributed / 10000 == 2026
            && [
                self.box_2a,
                self.box_3,
                self.box_6,
                self.box_8a,
                self.box_10,
                self.gross,
                self.withholding,
            ]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
            && self.box_9b.is_none_or(|v| (0..=100000000000).contains(&v))
            && crate::lump_sum::percent_valid(self.box_8b_percent)
            && crate::lump_sum::percent_valid(self.box_9a_percent)
            && (0..=100000000000).contains(&self.rolled_over)
            && (!source_alignment || self.rolled_over <= self.gross)
            && self.exception.as_ref().is_none_or(|e| {
                // The facts of an exception are checked by the early-distribution stage; the
                // shape is checked here.
                e.valid_with_source_alignment(self.distributed, self.gross, source_alignment)
            })
            && self.repaid.as_ref().is_none_or(|v| v.valid())
            && self.disaster.valid_with_source_alignment(self.gross, source_alignment)
    }
    // The exception that makes the owner's age or disability apply on this date, if any.
    pub(crate) fn disabled_since(&self) -> Option<u32> {
        match self.exception {
            Some(crate::RetirementException::Disability { began }) => Some(began),
            _ => None,
        }
    }
}

// The Form 1099-R Roth IRA code and its facts determine the stored withdrawal.
/// Roth IRA code
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RothIraCode {
    /// Q — qualified distribution
    /// You met the five-year period and were 59½ or older or disabled. Not taxed.
    Qualified,
    /// J — early distribution, no known exception
    Early {
        /// Date of this distribution
        /// For several payments on this form, the date of the last one. Enter payments from before and after you turned 59½ as two forms.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Exception to the 10% additional tax
        /// Leave empty when none applies. A first home up to $10,000 (lifetime, all your IRAs) is tax-free when your first Roth IRA year is 2021 or earlier, and otherwise free of the additional tax. Disability, age 59½ with code J printed in error, education, medical expenses, health insurance while unemployed, a birth or adoption, an emergency, terminal illness, domestic abuse, reservist duty, periodic payments and an IRS levy apply as to any IRA. An exception that only an employer plan can use is not offered.
        /// [NullText]
        /// No exception to the 10% additional tax applies
        /// [SetText]
        /// The exception to the 10% additional tax is
        exception: Option<crate::RetirementException>,
        /// Repaid to an IRA
        /// Only for a birth or adoption, emergency, domestic-abuse or terminal-illness distribution repaid to a Roth IRA within three years. Leave empty when none. Deposits after 2026 may need an amended 2026 return.
        /// [NullText]
        /// None of this distribution was repaid under an eligible repayment rule
        /// [SetText]
        /// The eligible IRA repayment is
        repaid: Option<crate::ExceptionRepayment>,
    },
    /// T — exception applies
    /// The payer knows you were 59½ or older, disabled, or a beneficiary, but not whether the five-year period is met.
    ExceptionApplies {
        /// Date of this distribution
        /// For several payments on this form, the date of the last one.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Why the exception applies
        #[libertas_default(Age)]
        reason: RothReason,
    },
}
/// Why the exception applies
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RothReason {
    /// I was 59½ or older
    Age,
    /// I am disabled
    /// You cannot do any substantial gainful activity because of a physical or mental condition that a doctor determined is expected to last indefinitely or end in death.
    Disabled {
        /// Date you became disabled
        #[libertas_date_only]
        since: u32,
    },
    /// I inherited this Roth IRA
    Inherited {
        /// Inherited IRA
        /// The person you inherited it from. Empty uses your only inherited Roth IRA; one entered on this same page is chosen after you save it.
        /// [NullText]
        /// Use my only inherited Roth IRA record
        /// [SetText]
        /// The inherited Roth IRA record is
        #[libertas_default($null)]
        account: Option<i64>,
    },
}
/// Roth IRA code
/// The distribution code and the answers used to report this withdrawal.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RothIraCodeView {
    /// Q — qualified distribution
    /// You met the five-year period and were 59½ or older or disabled. Not taxed.
    Qualified,
    /// J — early distribution, no known exception
    Early {
        /// Date of this distribution
        /// For several payments on this form, the date of the last one. Enter payments from before and after you turned 59½ as two forms.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Exception to the 10% additional tax
        /// Leave empty when none applies. A first home up to $10,000 (lifetime, all your IRAs) is tax-free when your first Roth IRA year is 2021 or earlier, and otherwise free of the additional tax. Disability, age 59½ with code J printed in error, education, medical expenses, health insurance while unemployed, a birth or adoption, an emergency, terminal illness, domestic abuse, reservist duty, periodic payments and an IRS levy apply as to any IRA. An exception that only an employer plan can use is not offered.
        /// [NullText]
        /// No exception to the 10% additional tax applies
        /// [SetText]
        /// The exception to the 10% additional tax is
        exception: Option<crate::RetirementException>,
        /// Repaid to an IRA
        /// Only for a birth or adoption, emergency, domestic-abuse or terminal-illness distribution repaid to a Roth IRA within three years. Leave empty when none. Deposits after 2026 may need an amended 2026 return.
        /// [NullText]
        /// None of this distribution was repaid under an eligible repayment rule
        /// [SetText]
        /// The eligible IRA repayment is
        repaid: Option<crate::ExceptionRepayment>,
    },
    /// T — exception applies
    /// The payer knows you were 59½ or older, disabled, or a beneficiary, but not whether the five-year period is met.
    ExceptionApplies {
        /// Date of this distribution
        /// For several payments on this form, the date of the last one.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Why the exception applies
        #[libertas_default(Age)]
        reason: RothReasonView,
    },
}
/// Why the exception applies
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RothReasonView {
    /// I was 59½ or older
    Age,
    /// I am disabled
    /// You cannot do any substantial gainful activity because of a physical or mental condition that a doctor determined is expected to last indefinitely or end in death.
    Disabled {
        /// Date you became disabled
        #[libertas_date_only]
        since: u32,
    },
    /// I inherited this Roth IRA
    Inherited {
        /// Inherited IRA
        /// The person you inherited it from. Empty uses your only inherited Roth IRA; one entered on this same page is chosen after you save it.
        /// [NullText]
        /// Use my only inherited Roth IRA record
        /// [SetText]
        /// The inherited Roth IRA record is
        #[libertas_enum_source("$.inherited_roth_iras")]
        #[libertas_default($null)]
        account: Option<i64>,
    },
}
record_page_conversions!(RothIraCode, RothIraCodeView, [], |value, Target, Source| {
    match value {
        Source::Qualified => Target::Qualified,
        Source::Early { distributed, exception, repaid } => {
            Target::Early { distributed, exception, repaid }
        }
        Source::ExceptionApplies { distributed, reason } => {
            Target::ExceptionApplies { distributed, reason: reason.into() }
        }
    }
});
record_page_conversions!(RothReason, RothReasonView, [], |value, Target, Source| {
    match value {
        Source::Age => Target::Age,
        Source::Disabled { since } => Target::Disabled { since },
        Source::Inherited { account } => Target::Inherited { account },
    }
});
impl RothIraCode {
    // ROTH_DISABLED_AFTER_DISTRIBUTION, the client rule on the 1099-R.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let (distributed, since, field, parent) = match self {
            Self::Early {
                distributed,
                exception: Some(crate::RetirementException::Disability { began: since }),
                ..
            } => (distributed, since, "began", "exception"),
            Self::ExceptionApplies { distributed, reason: RothReason::Disabled { since } } => {
                (distributed, since, "since", "reason")
            }
            _ => return None,
        };
        (since > distributed).then(|| {
            crate::validation::ValidationIssue::new("ROTH_DISABLED_AFTER_DISTRIBUTION")
                .at_field(field)
                .in_field(parent)
        })
    }
}

pub(crate) fn age_half(birth: u32) -> u32 {
    let mut year = birth / 10000 + 59;
    let mut month = birth / 100 % 100 + 6;
    if month > 12 {
        year += 1;
        month -= 12;
    }
    let mut day = birth % 100;
    while day > 28 && !valid_date(year * 10000 + month * 100 + day) {
        day -= 1;
    }
    year * 10000 + month * 100 + day
}
