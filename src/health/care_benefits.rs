// Coverage: Form 1099-LTC with Form 8853 Section C (unit B6a, 2026-10-09; the stored LtcBenefits
//   kind split into the per-form document and a per-insured periods record).
//   Lines: Section C lines 14a-26 for each LTC period: the insured, the recipient line 15, line 16
//   terminal illness, lines 17-20 (per diem payments of every qualified contract of the insured,
//   box 1 and the chronically ill box 2), line 21 ($430 times the days, Rev. Proc. 2025-32 section
//   4.62), line 22 the care costs, line 23 the larger, line 24 reimbursements, line 25 the limit and
//   line 26 the taxable amount (Schedule 1 line 8e), including the multiple-payee allocation (the
//   insured's filing unit first, then in proportion). Other-policyholder raw allocated limits
//   retain the pre-receipts-cap amount. The required Insured other-payee fact selects the full
//   sole-payee limit or insured priority allocation; exclusion and taxable income remain separate.
//   Tests: unit46_ltc raw allocation/source/lifecycle controls. Reimbursed benefits of a qualified
//   contract and accelerated death benefits of a terminally ill insured are excluded without Section C (section
//   101(g)(1), 7702B(a)(2)).
//   Entry: IncomeDocument::Ltc1099 (Form 1099-LTC, importable) names the insured person's
//   IncomeDocument::LtcInsured record through the request catalog `$.ltc_insured` (empty uses the
//   only record; on a joint return either filer's), both on the Other income page. The record holds
//   one period or several; a per diem payment with no record to use is taxed in full, with a notice.
//   Boundaries: a nonqualified contract's unresolved mandatory tax character withholds the
//   complete estimate; it is not silently omitted as an unavailable benefit. Typed accident/health
//   funding, permanent-injury and current medical reimbursement facts are computed outside
//   SectionC; established annuity/recovery sources are linked once. Actual additional interest
//   joins taxable interest. Qualified life/viatical streams independently use section101(g).
//   Tests: tests/health/health_accounts.rs (fixtures F8-F10), tests/health/interview.rs ltc1099_* and
//   ltc_insured_*. Sources: https://www.irs.gov/pub/irs-pdf/f1099ltc.pdf and i1099ltc.pdf (Rev. April
//   2025), https://www.irs.gov/pub/irs-dft/f8853--dft.pdf and i8853--dft.pdf (2026 drafts), Rev.
//   Proc. 2025-32 section 4.62, 26 U.S.C. 101(g), 7702B.
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use crate::kernel::ratio::exact::whole;
use crate::kernel::ratio::positive;
use crate::year_constants::LTC_PER_DIEM_LIMIT_CENTS;
use crate::{LtcContract, LtcContractPage, LtcPaymentOrigin, NonqualifiedCareTreatment};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Long-term-care benefit period method
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CarePeriodMethod {
    /// Insurer contract period
    /// Use the insurer's period. If contracts for this insured use different periods, use separate daily periods for all contracts. Multiple payees must use this method if they cannot agree on the equal-rate method.
    Contract,
    /// Period of unchanged payment rates
    /// Split whenever any included contract's rate changes or starts/stops. All payees for the insured must agree to these common periods. Do not combine changing rates just to increase the exclusion.
    EqualRate,
}

/// Recipient priority for the insured person's shared benefit limit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CareBenefitRecipient {
    /// This return is the insured person's filing unit
    /// On a joint return, include both spouses' payments together. The insured and jointly filing spouse receive first allocation of the limit.
    Insured {
        /// Another person received periodic payments for this insured in this period
        /// Include another policyholder outside this return who received per-diem or other periodic qualified LTC or qualifying accelerated death benefits for the same insured during this period. On a joint return, your spouse is part of your filing unit, not another payee. Check the common payees' statement; split periods when the payees or payment rates change. Do not count reimbursement-basis or terminal-illness payments.
        other_payees: bool,
    },
    /// This return belongs to another policyholder
    Other {
        /// Periodic payments to the insured person's filing unit for this period
        /// Include payments to the insured and the insured's spouse only if they file jointly. Exclude terminal-illness payments and reimbursements.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        insured_received: i64,
        /// Periodic payments to all other policyholders outside this return
        /// Exclude the insured filing unit above and the current return's payments. For a joint return combine your and your spouse's payments in Received, not here. Use the common recipients' allocation statement.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        others_received: i64,
    },
}

/// Qualified long-term-care payment period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct CareBenefitPeriod {
    /// First covered day in 2026
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub starts: u32,
    /// Last covered day in 2026, inclusive
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub ends: u32,
    /// Per-diem or other periodic benefits received by this return's policyholders
    /// Combine all qualified LTC contracts and qualifying accelerated death benefits for the chronically ill insured during this period. On a joint return include both spouses. Exclude reimbursement-basis benefits, terminal-illness benefits and interest. Receiving proceeds assigned to someone else does not transfer the policyholder's reporting responsibility.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub received: i64,
    /// Qualified long-term-care services provided during this period
    /// Total costs for the insured under a licensed practitioner's plan of care, across all payees. Do not subtract reimbursements here. These costs are not automatically a medical deduction; report only otherwise eligible unreimbursed costs in Medical.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub services: i64,
    /// Reimbursements received or expected for those services
    /// Include insurance and other reimbursements for all payees. Exclude reimbursements under contracts issued before August 1, 1996 that were not later exchanged or modified to increase benefits. Never include periodic payments or terminal-illness benefits here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reimbursements: i64,
    /// This return's place in the shared allocation
    pub recipient: CareBenefitRecipient,
    /// Actual source payments received in this period
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The period’s source payments have not been established
    /// [SetText]
    /// The actual payments in this period are
    #[libertas_default($null)]
    pub injured_payees: Option<crate::InjuredLtcPeriodPayees>,
}
/// Qualified long-term-care payment period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct CareBenefitPeriodInput {
    /// First covered day in 2026
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub starts: u32,
    /// Last covered day in 2026, inclusive
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub ends: u32,
    /// Per-diem or other periodic benefits received by this return's policyholders
    /// Combine all qualified LTC contracts and qualifying accelerated death benefits for the chronically ill insured during this period. On a joint return include both spouses. Exclude reimbursement-basis benefits, terminal-illness benefits and interest. Receiving proceeds assigned to someone else does not transfer the policyholder's reporting responsibility.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub received: i64,
    /// Qualified long-term-care services provided during this period
    /// Total costs for the insured under a licensed practitioner's plan of care, across all payees. Do not subtract reimbursements here. These costs are not automatically a medical deduction; report only otherwise eligible unreimbursed costs in Medical.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub services: i64,
    /// Reimbursements received or expected for those services
    /// Include insurance and other reimbursements for all payees. Exclude reimbursements under contracts issued before August 1, 1996 that were not later exchanged or modified to increase benefits. Never include periodic payments or terminal-illness benefits here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reimbursements: i64,
    /// This return's place in the shared allocation
    pub recipient: CareBenefitRecipient,
    /// Actual source payments received in this period
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The period’s source payments have not been established
    /// [SetText]
    /// The actual payments in this period are
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_source_facts")]
    pub injured_payees: Option<crate::InjuredLtcPeriodPayeesInput>,
}
impl CareBenefitPeriod {
    fn valid(&self) -> bool {
        self.starts / 10000 == crate::HEALTH_TAX_YEAR
            && self.ends / 10000 == crate::HEALTH_TAX_YEAR
            && valid_date(self.starts)
            && valid_date(self.ends)
            && self.starts <= self.ends
            && [self.received, self.services, self.reimbursements].iter().all(|v| *v >= 0)
            && self.injured_payees.as_ref().is_none_or(|v| v.valid(self.received))
            && match self.recipient {
                CareBenefitRecipient::Insured { .. } => true,
                CareBenefitRecipient::Other { insured_received, others_received } => {
                    [insured_received, others_received].iter().all(|v| *v >= 0)
                }
            }
    }
    // Aggregate limit, raw allocated line25 limit, exclusion used and taxable
    // payments. Required payee facts distinguish the raw limit from exclusion.
    pub(crate) fn amounts(&self) -> (i64, i64, i64, i64) {
        let days = i64::from(crate::dates::days_between(self.starts, self.ends) + 1);
        // Form 8853 Section C lines 21-25: the larger of the per diem limit for the
        // days and the care costs, less reimbursements.
        let limit = positive(
            phase::floor(days * LTC_PER_DIEM_LIMIT_CENTS, self.services) - self.reimbursements,
        );
        let (allocated, share) = match self.recipient {
            CareBenefitRecipient::Insured { other_payees } => {
                let share = phase::cap(self.received, [limit]);
                (if other_payees { share } else { limit }, share)
            }
            CareBenefitRecipient::Other { insured_received, others_received } => {
                let remaining = positive(limit - insured_received);
                let payments = whole(self.received) + whole(others_received);
                let allocated = if payments == whole(0) {
                    0
                } else {
                    crate::kernel::ratio::exact::rounded(
                        &(whole(remaining) * whole(self.received) / payments),
                    )
                    .expect("allocated LTC limit never exceeds its native remaining limit")
                };
                (allocated, phase::cap(self.received, [allocated]))
            }
        };
        (limit, allocated, share, positive(self.received - share))
    }
}

// Projection of the already calculated Section C taxable amount. These
// transient source amounts are not another benefit computation or ledger.
pub(crate) struct InjuredLtcPart {
    pub key: String,
    pub taxable: i64,
    pub receipts: Option<[i64; 2]>,
}
pub(crate) fn injured_period(
    id: i64,
    index: usize,
    period: &CareBenefitPeriod,
    forms: &[(&crate::FederalIncomeEntry, i64)],
    single_period: bool,
    taxable: i64,
) -> InjuredLtcPart {
    let mut annual = [0; 2];
    for (entry, amount) in forms {
        annual[usize::from(entry.owner == crate::Owner::Spouse)] += amount;
    }
    let receipts = if let Some(payees) = &period.injured_payees {
        let mut amounts = [0; 2];
        let valid = payees.valid(period.received)
            && payees.payments().iter().all(|payment| {
                if let Some((entry, _)) = forms.iter().find(|(e, _)| e.id == payment.source) {
                    amounts[usize::from(entry.owner == crate::Owner::Spouse)] += payment.amount;
                    true
                } else {
                    false
                }
            });
        valid.then_some(amounts)
    } else if single_period {
        (annual.iter().sum::<i64>() == period.received).then_some(annual)
    } else {
        match annual {
            [a, 0] if a > 0 => Some([period.received, 0]),
            [0, a] if a > 0 => Some([0, period.received]),
            _ => (period.received == 0).then_some([0; 2]),
        }
    };
    InjuredLtcPart { key: alloc::format!("ltc_{}_period_{}", id, index + 1), taxable, receipts }
}

/// 1099-LTC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-LTC (Long-Term Care and Accelerated Death Benefits) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read box 1 gross long-term care benefits and box 2 accelerated death benefits, returning zero for a clearly blank box; report an error when both are blank. For box 3 return PerDiem when Per diem is checked, Reimbursed when Reimbursed amount is checked, and NotChecked when neither is. For box 4 return Qualified with qualification Qualified when it is checked and leave the field unanswered otherwise. Never infer contract history, premium funding, payment origin, interest or a linked annuity/recovery source. For box 5 return Chronic when Chronically ill is checked and Terminal when Terminally ill is checked, and leave it unanswered when neither is. Do not read the payer, policyholder or insured name, address or TIN, the account number or the date certified, and do not choose the insured person's periods record; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Ltc1099 {
    /// Box 1 gross long-term care benefits
    /// Enter only a form on which you or your spouse is the policyholder.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub benefits: i64,
    /// Box 2 accelerated death benefits
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub accelerated: i64,
    /// Interest paid in addition to the benefits
    /// From the payer's actual statement, not part of box1 or2. Interest is taxable even when the benefits are excluded. Do not also enter it as a separate1099-INT amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Actual source of box1 benefits
    /// Choose Insurance for LTC insurance; do not infer an accelerated death-benefit source from illness alone. No source detail is needed when box1 is zero.
    /// [NullText]
    /// Box 1 is zero; no benefits source applies
    /// [SetText]
    /// The box 1 benefits came from
    pub benefit_origin: Option<LtcPaymentOrigin>,
    /// Actual source of box2 accelerated death benefits
    /// Use the life policy or viatical provider statement. No source detail is needed when box2 is zero.
    /// [NullText]
    /// Box 2 is zero; no accelerated-benefit source applies
    /// [SetText]
    /// The box 2 accelerated benefits came from
    pub accelerated_origin: Option<LtcPaymentOrigin>,
    /// How it was paid (box 3)
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub paid: LtcPaidBasis,
    /// Contract (box 4)
    /// Box 4 is optional for the payer; your policy says whether it is a qualified long-term care contract.
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub contract: LtcContractPage,
    /// The insured person was (box 5)
    /// Certified by a physician as expected to die within 24 months (terminally ill), or by a licensed health care practitioner as needing help with two daily activities or supervision for a cognitive impairment (chronically ill).
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub illness: LtcIllness,
    /// Long-term care periods for the insured person
    /// Needed for per diem payments. Empty uses your only periods record; one entered on this same page is chosen after you save it.
    /// [NullText]
    /// Use the insured person's only long-term care periods record
    /// [SetText]
    /// The insured person's periods record is
    #[libertas_enum_source("$.ltc_insured")]
    #[libertas_default($null)]
    pub insured: Option<i64>,
}

/// How it was paid (box 3)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum LtcPaidBasis {
    /// Per diem (a fixed amount for each day or other period)
    PerDiem,
    /// Reimbursed amount (actual care costs repaid)
    Reimbursed,
    /// Neither box is checked
    /// Usually an accelerated death benefit for a terminally ill person.
    NotChecked,
}

/// The insured person was (box 5)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum LtcIllness {
    /// Chronically ill
    Chronic,
    /// Terminally ill
    /// Accelerated death benefits are then not taxed.
    Terminal,
}

/// 1099-LTC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalLtcPayment {
    /// Box 1 gross long-term care benefits
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub benefits: i64,
    /// Box 2 accelerated death benefits
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub accelerated: i64,
    /// Interest paid in addition to the benefits
    /// From the payer's actual statement, not part of box1 or2. Interest is taxable even when the benefits are excluded. Do not also enter it as a separate1099-INT amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Actual source of box1 benefits
    /// Choose Insurance for LTC insurance; do not infer an accelerated death-benefit source from illness alone. No source detail is needed when box1 is zero.
    #[libertas_default($null)]
    pub benefit_origin: Option<LtcPaymentOrigin>,
    /// Actual source of box2 accelerated death benefits
    /// Use the life policy or viatical provider statement. No source detail is needed when box2 is zero.
    #[libertas_default($null)]
    pub accelerated_origin: Option<LtcPaymentOrigin>,
    /// How it was paid (box 3)
    pub paid: LtcPaidBasis,
    /// Contract (box 4)
    pub contract: LtcContract,
    /// The insured person was (box 5)
    pub illness: LtcIllness,
    /// Periods record identity
    /// The income identity of the insured person's periods record; empty uses the only one.
    #[libertas_hidden]
    pub insured: Option<i64>,
}

/// Long-term care periods
/// For one insured person who received care paid per diem in 2026. Keep the plan of care, bills and the insurer's statements.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct LtcInsured {
    /// Insured person
    #[libertas_inline_choices]
    pub insured: LtcInsuredPerson,
    /// Periods
    pub periods: LtcPeriodsInput,
}

/// Insured person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LtcInsuredPerson {
    /// Someone on this return
    OnReturn {
        /// Person
        #[libertas_enum_source("$.people")]
        person: i64,
    },
    /// Someone else
    /// For example a parent whose policy you own.
    Other {
        /// Name
        #[libertas_size(min = 1)]
        name: String,
    },
}

/// Periods
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LtcPeriods {
    /// One period
    /// The same payment rate for the whole time care was paid. The payments are those of your 1099-LTCs for this person.
    One {
        /// First day of care paid in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        starts: u32,
        /// Last day of care paid in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        ends: u32,
        /// Qualified long-term care costs for those days
        /// Costs of care under a licensed practitioner's plan of care, before reimbursements.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        services: i64,
        /// Reimbursements for those costs
        /// From insurance or anyone else, received or expected. Not the per diem payments.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        reimbursements: i64,
        /// Payments to others for this person
        #[libertas_default(Insured)]
        recipient: CareBenefitRecipient,
    },
    /// Several periods
    /// The payment rate changed, a contract started or stopped, or the insurer pays by a shorter period.
    Several {
        /// How the periods are set
        method: CarePeriodMethod,
        /// Periods
        /// ----
        /// Period
        // Tax structure: nonoverlapping care intervals cannot outnumber the calendar days in a year.
        #[libertas_size(min = 2, max = 366)]
        periods: Vec<CareBenefitPeriod>,
    },
}

/// Periods
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LtcPeriodsInput {
    /// One period
    /// The same payment rate for the whole time care was paid. The payments are those of your 1099-LTCs for this person.
    One {
        /// First day of care paid in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        starts: u32,
        /// Last day of care paid in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        ends: u32,
        /// Qualified long-term care costs for those days
        /// Costs of care under a licensed practitioner's plan of care, before reimbursements.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        services: i64,
        /// Reimbursements for those costs
        /// From insurance or anyone else, received or expected. Not the per diem payments.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        reimbursements: i64,
        /// Payments to others for this person
        #[libertas_default(Insured)]
        recipient: CareBenefitRecipient,
    },
    /// Several periods
    /// The payment rate changed, a contract started or stopped, or the insurer pays by a shorter period.
    Several {
        /// How the periods are set
        method: CarePeriodMethod,
        /// Periods
        /// ----
        /// Period
        // Tax structure: nonoverlapping care intervals cannot outnumber the calendar days in a year.
        #[libertas_size(min = 2, max = 366)]
        periods: Vec<CareBenefitPeriodInput>,
    },
}

/// Long-term care periods
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalLtcInsured {
    /// Insured person
    pub insured: FederalLtcInsuredPerson,
    /// Periods
    pub periods: LtcPeriods,
}

/// Insured person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalLtcInsuredPerson {
    /// Someone on this return
    OnReturn {
        /// Person identity
        /// −1 you, −2 your spouse, otherwise the dependent's accepted record identity.
        #[libertas_hidden]
        person: i64,
    },
    /// Someone else
    Other {
        /// Name
        #[libertas_size(min = 1)]
        name: String,
    },
}

record_page_conversions!(FederalLtcInsuredPerson, LtcInsuredPerson, [&], |value, Target, Source| {
    match value {
        Source::OnReturn { person } => Target::OnReturn { person: *person },
        Source::Other { name } => Target::Other { name: name.clone() },
    }
});
record_page_conversions!(FederalLtcInsured, LtcInsured, [&], |value, Target| {
    Target { insured: (&value.insured).into(), periods: value.periods.clone().into() }
});

impl LtcInsured {
    pub(crate) fn record(&self) -> FederalLtcInsured {
        self.into()
    }
}
impl FederalLtcInsured {
    pub(crate) fn page(&self) -> LtcInsured {
        self.into()
    }
    pub(crate) fn valid(&self) -> bool {
        (match &self.insured {
            FederalLtcInsuredPerson::OnReturn { person } => *person != 0,
            FederalLtcInsuredPerson::Other { name } => !name.trim().is_empty(),
        }) && match &self.periods {
            LtcPeriods::One { starts, ends, services, reimbursements, recipient } => {
                // One period is a period like any other: reuse its checks.
                CareBenefitPeriod {
                    starts: *starts,
                    ends: *ends,
                    received: 0,
                    injured_payees: None,
                    services: *services,
                    reimbursements: *reimbursements,
                    recipient: recipient.clone(),
                }
                .valid()
            }
            LtcPeriods::Several { periods, .. } => {
                // Tax structure: nonoverlapping intervals cannot outnumber the year's calendar days.
                (2..=366).contains(&periods.len())
                    && periods.iter().enumerate().all(|(i, p)| {
                        p.valid()
                            && !periods[..i]
                                .iter()
                                .any(|v| p.starts <= v.ends && v.starts <= p.ends)
                    })
            }
        }
    }
    // The record's validation rule, checked again on save with the page's key.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if matches!(&self.periods, LtcPeriods::One { starts, ends, .. } if ends < starts) {
            return Some(
                ValidationIssue::new("LTC_PERIOD_ENDS_BEFORE_START")
                    .at_field("ends")
                    .in_field("periods"),
            );
        }
        if let LtcPeriods::Several { periods, .. } = &self.periods {
            for (index, period) in periods.iter().enumerate() {
                if let Some(issue) =
                    period.injured_payees.as_ref().and_then(|payees| payees.issue(period.received))
                {
                    return Some(
                        issue
                            .in_field("injured_payees")
                            .in_list("periods", index)
                            .in_field("periods"),
                    );
                }
            }
        }
        None
    }
    // The periods Section C is figured over, with the linked forms' line 20 payments
    // (cents) as the one period's payments; None when several periods' own payments
    // do not total them.
    pub(crate) fn rows(&self, line_20: i64) -> Option<Vec<CareBenefitPeriod>> {
        match &self.periods {
            LtcPeriods::One { starts, ends, services, reimbursements, recipient } => {
                Some(alloc::vec![CareBenefitPeriod {
                    starts: *starts,
                    ends: *ends,
                    received: line_20,
                    services: *services,
                    reimbursements: *reimbursements,
                    recipient: recipient.clone(),
                    injured_payees: None,
                }])
            }
            LtcPeriods::Several { periods, .. } => {
                (crate::kernel::ratio::checked_sum(periods.iter().map(|p| p.received))
                    == Some(line_20))
                .then(|| periods.clone())
            }
        }
    }
}

// Input paths in these Page projections have the same field names as their
// accepted records. Union payloads are transparent; no variant segment is added.
// Unresolved character remains an accepted source that withholds the estimate.
// Only the existing independently-entered-source refusal belongs in preflight;
// the whole-return reference owner still checks shared annuity gross totals.
pub(crate) fn document_issue(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    document: &crate::IncomeDocument,
) -> Option<crate::validation::ValidationIssue> {
    match document {
        crate::IncomeDocument::Ltc1099(page) => {
            let record = page.record();
            record.issue().or_else(|| {
                record
                    .character_issue(d, owner)
                    .filter(|issue| issue.key == "LTC1099_SOURCE_UNAVAILABLE")
            })
        }
        crate::IncomeDocument::LtcInsured(page) => page.record().issue(),
        _ => None,
    }
}

record_page_conversions!(FederalLtcPayment, Ltc1099, [&], |value, Target| {
    Target {
        benefits: value.benefits,
        accelerated: value.accelerated,
        interest: value.interest,
        benefit_origin: value.benefit_origin.clone(),
        accelerated_origin: value.accelerated_origin.clone(),
        paid: value.paid,
        contract: (&value.contract).into(),
        illness: value.illness,
        insured: value.insured,
    }
});

impl Ltc1099 {
    pub(crate) fn record(&self) -> FederalLtcPayment {
        self.into()
    }
}
impl FederalLtcPayment {
    pub(crate) fn page(&self) -> Ltc1099 {
        self.into()
    }
    pub(crate) fn valid(&self) -> bool {
        // Arithmetic exclusion: the federal before_rental MAGI/unearned pools,
        // Olympic AGI-to-Cents input and final income rows still have native
        // receivers across other owners. Checked local receipts alone do not
        // establish their full-width capacity or permit intermediate refusals.
        [self.benefits, self.accelerated].iter().all(|v| (0..=100000000000).contains(v))
            // The shared native interest receiver still combines other source classes.
            && (0..=100000000000).contains(&self.interest)
            && self.insured.is_none_or(|id| id > 0)
            && self.contract.valid()
            && self.rule().is_none()
    }
    // The page's rules, repeated against accepted facts.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        // The periodic/excluded previews and federal receipt owners carry native Cents.
        // Admit against that actual receiver, never a fixed per-box money ceiling.
        if self.benefits.checked_add(self.accelerated).is_none() {
            return Some(
                crate::validation::ValidationIssue::new("MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE")
                    .at_field("accelerated"),
            );
        }
        if self.benefits + self.accelerated <= 0 {
            return Some(ValidationIssue::new("LTC1099_NO_AMOUNT").at_field("benefits"));
        }
        if self.benefit_origin.as_ref().is_some_and(|o| !matches!(o, LtcPaymentOrigin::Insurance))
            || matches!(self.accelerated_origin, Some(LtcPaymentOrigin::Insurance))
        {
            return Some(
                ValidationIssue::new("LTC1099_ORIGIN_STREAM").at_field(
                    if self
                        .benefit_origin
                        .as_ref()
                        .is_some_and(|o| !matches!(o, LtcPaymentOrigin::Insurance))
                    {
                        "benefit_origin"
                    } else {
                        "accelerated_origin"
                    },
                ),
            );
        }
        if let LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth { funding, .. },
        } = &self.contract
            && let Some(issue) = funding.issue()
        {
            return Some(issue.in_field("funding").in_field("treatment").in_field("contract"));
        }
        if let LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth { .. },
        } = &self.contract
            && self.accident_health().is_none()
        {
            return Some(
                ValidationIssue::new("LTC1099_EXCLUDED_PORTIONS_ABOVE_EMPLOYER")
                    .at_field("medical_reimbursement")
                    .in_field("treatment")
                    .in_field("contract"),
            );
        }
        None
    }
    pub(crate) fn accident_health(&self) -> Option<(i64, i64)> {
        match &self.contract {
            LtcContract::Nonqualified { treatment } => treatment.accident_health(self.benefits),
            _ => None,
        }
    }
    pub(crate) fn character_issue(
        &self,
        d: &crate::FederalDraft,
        owner: crate::Owner,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.benefits > 0 && !matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance)) {
            return Some(
                ValidationIssue::new("ISSUE_LTC_SOURCE_CHARACTER_UNRESOLVED")
                    .at_field("benefit_origin"),
            );
        }
        if self.accelerated > 0
            && !self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            return Some(
                ValidationIssue::new("ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED")
                    .at_field("accelerated_origin"),
            );
        }
        if self.benefits > 0 {
            match &self.contract {
                LtcContract::Qualified { .. } if self.contract.qualified() => {}
                LtcContract::Nonqualified {
                    treatment: NonqualifiedCareTreatment::AccidentHealth { .. },
                } if self.accident_health().is_some() => {}
                LtcContract::Nonqualified {
                    treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
                } => {
                    if !annuity_source(d, *source, owner, self.benefits) {
                        return Some(
                            ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                                .at_field("source")
                                .in_field("treatment")
                                .in_field("contract"),
                        );
                    }
                }
                _ => {
                    return Some(
                        ValidationIssue::new("ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
                            .at_field("contract"),
                    );
                }
            }
        }
        if let LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth { recovery: Some(id), .. },
        } = &self.contract
            && !recovery_source(d, *id, owner, self.benefits)
        {
            return Some(
                ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                    .at_field("recovery")
                    .in_field("treatment")
                    .in_field("contract"),
            );
        }
        None
    }
    pub(crate) fn character_rule(
        &self,
        d: &crate::FederalDraft,
        owner: crate::Owner,
    ) -> Option<&'static str> {
        self.character_issue(d, owner).map(|issue| issue.key)
    }
    // Section C line 20 payments of this form (cents): the per diem box 1 and the
    // chronically ill box 2 of a qualified contract. A reimbursed amount and a
    // terminally ill accelerated death benefit are excluded without Section C.
    pub(crate) fn periodic(&self) -> i64 {
        if self.paid == LtcPaidBasis::Reimbursed {
            return 0;
        }
        let benefits = if self.contract.qualified()
            && matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance))
        {
            self.benefits
        } else {
            0
        };
        let accelerated = if self.illness != LtcIllness::Terminal
            && self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            self.accelerated
        } else {
            0
        };
        benefits + accelerated
    }
    pub(crate) fn excluded(&self) -> i64 {
        let benefits = if self.contract.qualified()
            && self.paid == LtcPaidBasis::Reimbursed
            && matches!(self.benefit_origin, Some(LtcPaymentOrigin::Insurance))
        {
            self.benefits
        } else {
            0
        };
        let accelerated = if (self.illness == LtcIllness::Terminal
            || self.paid == LtcPaidBasis::Reimbursed)
            && self.accelerated_origin.as_ref().is_some_and(LtcPaymentOrigin::life_eligible)
        {
            self.accelerated
        } else {
            0
        };
        benefits + accelerated
    }
    pub(crate) fn preview(&self) -> Option<i64> {
        // Linked annuity/recovery character and the shared per-diem limit need
        // the coordinated return. Only a completely known outright exclusion
        // has a standalone row preview.
        (self.rule().is_none()
            && self.periodic() == 0
            && self.excluded() == self.benefits + self.accelerated)
            .then_some(self.interest)
    }
}

// Which periods record a 1099-LTC names: an explicit choice is the record's income
// identity, an empty one uses the only record (the owner's, or on a joint return
// either filer's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    Record(i64),
    // No record to use: none entered, or several and none chosen.
    Missing,
    // The chosen record belongs to the other filer on a separate return.
    OtherOwner,
}
pub(crate) fn resolve(d: &crate::FederalDraft, owner: crate::Owner, insured: Option<i64>) -> Link {
    use crate::FederalIncome;
    let joint = d.setup.as_ref().is_some_and(|s| crate::federal_rules::joint(s.filing_status()));
    match insured {
        Some(id) => match d.income.iter().find(|e| e.id == id) {
            Some(e) if matches!(e.income, FederalIncome::LtcInsured { .. }) => {
                if e.owner != owner && !joint { Link::OtherOwner } else { Link::Record(id) }
            }
            _ => Link::Missing,
        },
        None => {
            let mut records = d.income.iter().filter(|e| {
                (joint || e.owner == owner) && matches!(e.income, FederalIncome::LtcInsured { .. })
            });
            match (records.next(), records.next()) {
                (Some(e), None) => Link::Record(e.id),
                _ => Link::Missing,
            }
        }
    }
}
// The link of a 1099-LTC that has a per diem payment to place.
pub(crate) fn link_of(d: &crate::FederalDraft, e: &crate::FederalIncomeEntry) -> Option<Link> {
    match &e.income {
        crate::FederalIncome::LtcBenefits { data } if data.periodic() > 0 => {
            Some(resolve(d, e.owner, data.insured))
        }
        _ => None,
    }
}
// The forms that name a record, with their line 20 payments.
pub(crate) fn payments(d: &crate::FederalDraft, id: i64) -> Vec<(&crate::FederalIncomeEntry, i64)> {
    d.income
        .iter()
        .filter(|e| link_of(d, e) == Some(Link::Record(id)))
        .filter_map(|e| match &e.income {
            crate::FederalIncome::LtcBenefits { data } => Some((e, data.periodic())),
            _ => None,
        })
        .collect()
}

// These are references to existing independently entered sources. No tax or
// gross is copied into another accepted income record.
pub(crate) fn annuity_source(
    d: &crate::FederalDraft,
    id: i64,
    owner: crate::Owner,
    amount: i64,
) -> bool {
    d.income.iter().any(|e|e.id==id && e.owner==owner && matches!(&e.income,crate::FederalIncome::Retirement { data } if data.annuity_benefit_source() && data.gross>=amount))
}
pub(crate) fn recovery_source(
    d: &crate::FederalDraft,
    id: i64,
    owner: crate::Owner,
    _amount: i64,
) -> bool {
    // The existing recovery may combine this receipt with other same-year
    // reimbursements; its total is not bounded by one LTC statement.

    d.income.iter().any(|e|e.id==id && e.owner==owner && matches!(&e.income,crate::FederalIncome::OtherIncome { data } if matches!(data.kind,crate::OtherIncomeKind::Recovery { .. }) && data.amount>0))
}
pub(crate) fn reference_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    for entry in &d.income {
        let crate::FederalIncome::LtcBenefits { data } = &entry.income else {
            continue;
        };
        match &data.contract {
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::LifeAnnuity { source },
            } => {
                let gross=d.income.iter().filter_map(|e| match &e.income {
                    crate::FederalIncome::LtcBenefits { data } if matches!(&data.contract,LtcContract::Nonqualified { treatment:NonqualifiedCareTreatment::LifeAnnuity { source:other } } if source==other) => Some(data.benefits),
                    _=>None,
                });
                let gross = crate::kernel::ratio::checked_sum(gross);
                if !gross.is_some_and(|gross| annuity_source(d, *source, entry.owner, gross)) {
                    return Some((
                        entry.id,
                        crate::validation::ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                            .at_field("source")
                            .in_field("treatment")
                            .in_field("contract"),
                    ));
                }
            }
            LtcContract::Nonqualified {
                treatment: NonqualifiedCareTreatment::AccidentHealth { recovery: Some(id), .. },
            } if !recovery_source(d, *id, entry.owner, data.benefits) => {
                return Some((
                    entry.id,
                    crate::validation::ValidationIssue::new("LTC1099_SOURCE_UNAVAILABLE")
                        .at_field("recovery")
                        .in_field("treatment")
                        .in_field("contract"),
                ));
            }
            _ => {}
        }
    }
    native_receipts_issue(d)
}

// Native receiver admission shared by candidate references and federal collection.
// The actual existing periodic, clinical-period and accident/health owners compute
// the amounts; this adapter only checks their required native aggregate carriers.
pub(crate) fn native_receipts_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let invalid = |id| {
        (
            id,
            crate::validation::ValidationIssue::new("MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE")
                .at_record(),
        )
    };
    for entry in &d.income {
        if let crate::FederalIncome::LtcBenefits { data } = &entry.income
            && let Some(issue) = data.issue()
        {
            return Some((entry.id, issue));
        }
    }
    let mut taxable = alloc::vec::Vec::new();
    for entry in &d.income {
        match &entry.income {
            crate::FederalIncome::LtcBenefits { data } => {
                if data.character_rule(d, entry.owner).is_some() {
                    continue;
                }
                if let Some((amount, _)) = data.accident_health() {
                    taxable.push(amount);
                }
                if link_of(d, entry) == Some(Link::Missing) {
                    taxable.push(data.periodic());
                }
            }
            crate::FederalIncome::LtcInsured { data } => {
                let forms = payments(d, entry.id);
                let received =
                    crate::kernel::ratio::checked_sum(forms.iter().map(|(_, amount)| *amount));
                let Some(received) = received else {
                    return Some(invalid(entry.id));
                };
                if data.valid()
                    && let Some(periods) = data.rows(received)
                {
                    taxable.extend(periods.iter().map(|period| period.amounts().3));
                }
            }
            _ => {}
        }
    }
    let id = d
        .income
        .iter()
        .rev()
        .find(|e| {
            matches!(
                e.income,
                crate::FederalIncome::LtcBenefits { .. } | crate::FederalIncome::LtcInsured { .. }
            )
        })?
        .id;
    crate::kernel::ratio::checked_sum(taxable).is_none().then(|| invalid(id))
}

pub(crate) fn reference_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    reference_issue(d).map(|(_, issue)| issue.key)
}

impl From<CareBenefitPeriodInput> for CareBenefitPeriod {
    fn from(v: CareBenefitPeriodInput) -> Self {
        Self {
            starts: v.starts,
            ends: v.ends,
            received: v.received,
            services: v.services,
            reimbursements: v.reimbursements,
            recipient: v.recipient,
            injured_payees: v.injured_payees.map(Into::into),
        }
    }
}
impl From<CareBenefitPeriod> for CareBenefitPeriodInput {
    fn from(v: CareBenefitPeriod) -> Self {
        Self {
            starts: v.starts,
            ends: v.ends,
            received: v.received,
            services: v.services,
            reimbursements: v.reimbursements,
            recipient: v.recipient,
            injured_payees: v.injured_payees.map(Into::into),
        }
    }
}
impl From<LtcPeriodsInput> for LtcPeriods {
    fn from(v: LtcPeriodsInput) -> Self {
        match v {
            LtcPeriodsInput::One { starts, ends, services, reimbursements, recipient } => {
                Self::One { starts, ends, services, reimbursements, recipient }
            }
            LtcPeriodsInput::Several { method, periods } => {
                Self::Several { method, periods: periods.into_iter().map(Into::into).collect() }
            }
        }
    }
}
impl From<LtcPeriods> for LtcPeriodsInput {
    fn from(v: LtcPeriods) -> Self {
        match v {
            LtcPeriods::One { starts, ends, services, reimbursements, recipient } => {
                Self::One { starts, ends, services, reimbursements, recipient }
            }
            LtcPeriods::Several { method, periods } => {
                Self::Several { method, periods: periods.into_iter().map(Into::into).collect() }
            }
        }
    }
}
