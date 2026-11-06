// Coverage: Form 1099-LTC with Form 8853 Section C, with a per-form document
//   and a per-insured periods record.
//   Lines: Section C lines 14a-26 for each LTC period: the insured, the recipient line 15, line 16
//   terminal illness, lines 17-20 (per diem payments of every qualified contract of the insured,
//   box 1 and the chronically ill box 2), line 21 ($430 times the days, Rev. Proc. 2025-32 section
//   4.62), line 22 the care costs, line 23 the larger, line 24 reimbursements, line 25 the limit and
//   line 26 the taxable amount (Schedule 1 line 8e), including the multiple-payee allocation (the
//   insured's filing unit first, then in proportion). Other-policyholder raw allocated limits
//   retain the pre-receipts-cap amount. The required Insured other-payee fact selects the full
//   sole-payee limit or insured priority allocation; exclusion and taxable income remain separate.
//   The shared printed-party carrier retains the insured's identity independently of the personal
//   insured-period link. The latest certification date distinguishes blank from unavailable evidence.
//   Tests: LTC raw allocation/source/lifecycle controls. Reimbursed benefits of a qualified
//   contract and accelerated death benefits of a terminally ill insured are excluded without Section C (section
//   101(g)(1), 7702B(a)(2)).
//   Entry: OtherIncomeDocument::Ltc1099 (Form 1099-LTC, importable) names the insured person's
//   IncomeRecord::LtcInsured record through the request catalog `$.ltc_insured` (empty uses the
//   only record; on a joint return either filer's). The form belongs to the Document center;
//   its periods record is edited on Other income. The record holds
//   one period or several; a per diem payment with no record to use is taxed in full, with a notice.
//   Boundaries: a nonqualified contract's unresolved mandatory tax character withholds the
//   complete estimate; it is not silently omitted as an unavailable benefit. Typed accident/health
//   funding, permanent-injury and current medical reimbursement facts are computed outside
//   SectionC; established annuity/recovery sources are linked once. Actual additional interest
//   is collected with the payer's statement and joins taxable interest. A checked optional box 4
//   establishes qualified status without a duplicate topic answer. Qualified life/viatical streams
//   independently use section101(g).
//   Tests: tests/health/health_accounts.rs (fixtures F8-F10), tests/health/interview.rs ltc1099_* and
//   ltc_insured_*. Sources: https://www.irs.gov/pub/irs-pdf/f1099ltc.pdf and i1099ltc.pdf (Rev. April
//   2025), https://www.irs.gov/pub/irs-dft/f8853--dft.pdf and i8853--dft.pdf (2026 drafts), Rev.
//   Proc. 2025-32 section 4.62, 26 U.S.C. 101(g), 7702B.
use crate::dates::source_valid_date as valid_date;
use crate::{LtcContract, LtcPaymentOrigin, NonqualifiedCareTreatment};
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        insured_received: i64,
        /// Periodic payments to all other policyholders outside this return
        /// Exclude the insured filing unit above and the current return's payments. For a joint return combine your and your spouse's payments in Received, not here. Use the common recipients' allocation statement.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub services: i64,
    /// Reimbursements received or expected for those services
    /// Include insurance and other reimbursements for all payees. Exclude reimbursements under contracts issued before August 1, 1996 that were not later exchanged or modified to increase benefits. Never include periodic payments or terminal-illness benefits here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub services: i64,
    /// Reimbursements received or expected for those services
    /// Include insurance and other reimbursements for all payees. Exclude reimbursements under contracts issued before August 1, 1996 that were not later exchanged or modified to increase benefits. Never include periodic payments or terminal-illness benefits here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
}

/// Form 1099-LTC — Long-Term Care and Accelerated Death Benefits
// IRS Form 1099-LTC, Rev. April 2025, Copy B.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the PAYER'S printed party block into payer, the POLICYHOLDER'S name, TIN and address into recipient, and the INSURED'S name, TIN and address into insured_party. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation, leading zeros and address line breaks; use empty text for absent identity fields and report illegible identity text as an error. Never infer the policyholder from household names or the insured's name. Read one Form 1099-LTC (Long-Term Care and Accelerated Death Benefits) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read box 1 gross long-term care benefits and box 2 accelerated death benefits, returning null for a clearly blank box; report an error when both are blank. For box 3 return PerDiem when Per diem is checked, Reimbursed when Reimbursed amount is checked, and NotChecked when neither is. Read box 4 into box4_qualified: true when checked and false when unchecked; a checked box establishes qualified contract status, while an unchecked optional box leaves actual qualification unanswered. Never infer contract history, premium funding, payment origin or a linked annuity/recovery source. For box 5 return Chronic when Chronically ill is checked and Terminal when Terminally ill is checked, and leave it unanswered when neither is. Read the latest Date certified into date_certified as a DateOnly value for a legible printed date or null for a clearly blank date; report an illegible date as an error. Do not infer a certification date from an illness checkbox or care period. Read the printed calendar year, corrected checkbox, payer telephone number and account number. Leave blank text empty, never the text 0. Do not choose the insured person's periods record or infer it from printed identity; the filer answers that association separately. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.insured_party.federal_id,@.benefits,@.accelerated,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Ltc1099Paper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: i32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_phone: String,
    /// POLICYHOLDER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// INSURED'S name, TIN and address
    #[libertas_llm_doc_field]
    pub insured_party: crate::PrintedParty,
    /// Box 1 Gross long-term care benefits paid
    /// [NullText]
    /// Box 1 Gross long-term care benefits paid is blank
    /// [SetText]
    /// Printed box 1 gross long-term care benefits paid
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub benefits: Option<i64>,
    /// Box 2 Accelerated death benefits paid
    /// [NullText]
    /// Box 2 Accelerated death benefits paid is blank
    /// [SetText]
    /// Printed box 2 accelerated death benefits paid
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub accelerated: Option<i64>,
    /// Box 3 Per diem / Reimbursed amount
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub paid: LtcPaidBasis,
    /// Box 4 Qualified contract (optional)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box4_qualified: bool,
    /// Box 5 Chronically ill / Terminally ill (optional)
    /// [NullText]
    /// Neither illness box is checked
    /// [SetText]
    /// Printed illness status
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    #[libertas_default($null)]
    pub illness: Option<LtcIllness>,
    /// Box 5 Date certified
    /// [NullText]
    /// No date is printed
    /// [SetText]
    /// Printed date certified
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub date_certified: Option<u32>,
}

impl Ltc1099Paper {
    pub(crate) fn intrinsic_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let date_certified = match self.date_certified {
            Some(date) => crate::SourceDate::Reported { date },
            None => crate::SourceDate::NotReported,
        };
        ltc_intrinsic_issue(
            self.benefits.unwrap_or(0),
            self.accelerated.unwrap_or(0),
            &date_certified,
        )
    }
}

impl From<&Ltc1099Paper> for Ltc1099 {
    fn from(paper: &Ltc1099Paper) -> Self {
        let Ltc1099Paper {
            // Printed identification fields do not enter the tax calculation.
            calendar_year: _,
            corrected: _,
            payer,
            payer_phone: _,
            recipient,
            account_number: _,
            insured_party,
            benefits,
            accelerated,
            paid,
            box4_qualified,
            illness,
            date_certified,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            insured_party: insured_party.clone(),
            benefits: benefits.unwrap_or(0),
            accelerated: accelerated.unwrap_or(0),
            // Not a box of the paper: the wrapper's answer supplies it (`LtcSource::interest`).
            interest: 0,
            paid: *paid,
            box4_qualified: *box4_qualified,
            illness: *illness,
            date_certified: match date_certified {
                Some(date) => crate::SourceDate::Reported { date: *date },
                None => crate::SourceDate::NotReported,
            },
        }
    }
}

/// 1099-LTC
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the PAYER'S printed party block into payer, the POLICYHOLDER'S name, TIN and address into recipient, and the INSURED'S name, TIN and address into insured_party. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation, leading zeros and address line breaks; use empty text for absent identity fields and report illegible identity text as an error. Never infer the policyholder from household names or the insured's name. Read one Form 1099-LTC (Long-Term Care and Accelerated Death Benefits) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read box 1 gross long-term care benefits and box 2 accelerated death benefits, returning zero for a clearly blank box; report an error when both are blank. For box 3 return PerDiem when Per diem is checked, Reimbursed when Reimbursed amount is checked, and NotChecked when neither is. Read box 4 into box4_qualified: true when checked and false when unchecked; a checked box establishes qualified contract status, while an unchecked optional box leaves actual qualification unanswered. Never infer contract history, premium funding, payment origin or a linked annuity/recovery source. For box 5 return Chronic when Chronically ill is checked and Terminal when Terminally ill is checked, and leave it unanswered when neither is. Read the latest Date certified into date_certified: Reported for a legible printed date, NotReported for a clearly blank date and Unavailable when it cannot be read. Do not infer a certification date from an illness checkbox or care period. Do not read the account number. Do not choose the insured person's periods record or infer it from printed identity; the filer answers that association separately. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Ltc1099 {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// POLICYHOLDER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// INSURED'S name, identifying number and address
    #[libertas_llm_doc_field]
    pub insured_party: crate::PrintedParty,
    /// Box 1 gross long-term care benefits
    /// Enter only a form on which you or your spouse is the policyholder.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub benefits: i64,
    /// Box 2 accelerated death benefits
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub accelerated: i64,
    /// Interest paid in addition to the benefits
    /// From the payer's actual statement, not part of box1 or2. Interest is taxable even when the benefits are excluded. Do not also enter it as a separate1099-INT amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub interest: i64,
    /// How it was paid (box 3)
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub paid: LtcPaidBasis,
    /// Qualified contract checkbox (box 4)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box4_qualified: bool,
    /// The insured person was (box 5)
    /// Certified by a physician as expected to die within 24 months (terminally ill), or by a licensed health care practitioner as needing help with two daily activities or supervision for a cognitive impairment (chronically ill).
    /// [NullText]
    /// Neither illness box is checked
    /// [SetText]
    /// Printed illness status
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    #[libertas_default($null)]
    pub illness: Option<LtcIllness>,
    /// Box 5 date certified
    #[libertas_llm_doc_field]
    #[libertas_default(Unavailable)]
    pub date_certified: crate::SourceDate,
}

/// Accepted Form 1099-LTC
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LtcSource {
    /// Form 1099-LTC as printed
    pub form: Ltc1099Paper,
    /// Interest paid in addition to the benefits
    /// From the payer's statement, outside boxes 1 and 2. Do not enter the same interest again on Form 1099-INT.
    /// [NullText]
    /// Interest paid in addition to the benefits is blank
    /// [SetText]
    /// Interest paid in addition to the benefits (from the payer's statement; not a box of Form 1099-LTC)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub interest: Option<i64>,
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
    /// Contract qualification and history not established by box 4
    #[libertas_default(Unknown)]
    pub contract: LtcContract,
    /// Long-term care periods for the insured person
    /// Needed for per diem payments. Empty uses your only periods record; one entered on this same page is chosen after you save it.
    /// [NullText]
    /// Use the insured person's only long-term care periods record
    /// [SetText]
    /// The insured person's periods record is
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub benefits: i64,
    /// Box 2 accelerated death benefits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub accelerated: i64,
    /// Interest paid in addition to the benefits
    /// From the payer's actual statement, not part of box1 or2. Interest is taxable even when the benefits are excluded. Do not also enter it as a separate1099-INT amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    /// Printed qualified contract checkbox (box 4)
    pub box4_qualified: bool,
    /// Actual contract qualification and history
    pub contract: LtcContract,
    /// The insured person was (box 5)
    /// [NullText]
    /// Neither illness box is checked
    /// [SetText]
    /// Printed illness status
    #[libertas_default($null)]
    pub illness: Option<LtcIllness>,
    /// Box 5 date certified
    #[libertas_default(Unavailable)]
    pub date_certified: crate::SourceDate,
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        services: i64,
        /// Reimbursements for those costs
        /// From insurance or anyone else, received or expected. Not the per diem payments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
        // Tax structure: nonoverlapping care intervals cannot outnumber the calendar days in a year.
        #[libertas_size(min = 2, max = 366)]
        /// ----
        /// Period
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        services: i64,
        /// Reimbursements for those costs
        /// From insurance or anyone else, received or expected. Not the per diem payments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
        // Tax structure: nonoverlapping care intervals cannot outnumber the calendar days in a year.
        #[libertas_size(min = 2, max = 366)]
        /// ----
        /// Period
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
}

impl LtcSource {
    /// The internal record the calculations read.
    pub(crate) fn record(&self) -> FederalLtcPayment {
        self.into()
    }
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Ltc1099Paper) -> Self {
        Self {
            form,
            interest: None,
            benefit_origin: None,
            accelerated_origin: None,
            contract: LtcContract::Unknown,
            insured: None,
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.interest = accepted.interest;
        self.benefit_origin = accepted.benefit_origin.clone();
        self.accelerated_origin = accepted.accelerated_origin.clone();
        self.contract = accepted.contract.clone();
        self.insured = accepted.insured;
    }
}
// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&LtcSource> for FederalLtcPayment {
    fn from(source: &LtcSource) -> Self {
        let LtcSource { form, interest, benefit_origin, accelerated_origin, contract, insured } =
            source;
        let Ltc1099 {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            insured_party: _,
            benefits,
            accelerated,
            interest: _,
            paid,
            box4_qualified,
            illness,
            date_certified,
        } = &Ltc1099::from(form);
        Self {
            benefits: *benefits,
            accelerated: *accelerated,
            interest: interest.unwrap_or(0),
            benefit_origin: benefit_origin.clone(),
            accelerated_origin: accelerated_origin.clone(),
            paid: *paid,
            box4_qualified: *box4_qualified,
            // The optional checked box establishes qualification; an unchecked box does not.
            // https://www.irs.gov/instructions/i1099ltc
            contract: if *box4_qualified {
                LtcContract::Qualified {
                    qualification: crate::CareContractQualification::Qualified,
                }
            } else {
                contract.clone()
            },
            illness: *illness,
            date_certified: date_certified.clone(),
            insured: *insured,
        }
    }
}

fn ltc_intrinsic_issue(
    benefits: i64,
    accelerated: i64,
    date_certified: &crate::SourceDate,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    // The periodic/excluded previews and federal receipt owners carry native Cents.
    // Admit against that actual receiver, never a fixed per-box money ceiling.
    if benefits.checked_add(accelerated).is_none() {
        return Some(
            ValidationIssue::new("MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE").at_field("accelerated"),
        );
    }
    if benefits + accelerated <= 0 {
        return Some(ValidationIssue::new("LTC1099_NO_AMOUNT").at_field("benefits"));
    }
    if date_certified.source_fact().is_err() {
        return Some(
            ValidationIssue::new("LTC1099_DATE_CERTIFIED_INVALID")
                .at_field("date")
                .in_field("date_certified"),
        );
    }
    None
}

impl FederalLtcPayment {
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
            && self.intrinsic_issue().is_none()
    }
    // The page's rules, repeated against accepted facts.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn effective_contract(&self) -> &LtcContract {
        const PRINTED_QUALIFIED: LtcContract =
            LtcContract::Qualified { qualification: crate::CareContractQualification::Qualified };
        if self.box4_qualified && matches!(self.contract, LtcContract::Unknown) {
            &PRINTED_QUALIFIED
        } else {
            &self.contract
        }
    }
    pub(crate) fn intrinsic_issue(&self) -> Option<crate::validation::ValidationIssue> {
        ltc_intrinsic_issue(self.benefits, self.accelerated, &self.date_certified)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some(issue) = self.intrinsic_issue() {
            return Some(issue);
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
        } = self.effective_contract()
            && let Some(issue) = funding.issue()
        {
            return Some(issue.in_field("funding").in_field("treatment").in_field("contract"));
        }
        if let LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth { .. },
        } = self.effective_contract()
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
        match self.effective_contract() {
            LtcContract::Nonqualified { treatment } => treatment.accident_health(self.benefits),
            _ => None,
        }
    }
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
