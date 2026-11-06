// form_1099_r items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
use crate::{RetirementTaxable, RetirementTreatment};
use libertas_macros::libertas_form_code;

/// Retirement statement category
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RetirementKind {
    /// IRA distribution (IRA/SEP/SIMPLE box checked)
    Ira,
    /// Qualified pension or retirement-plan annuity
    /// Includes eligible railroad pensions; excludes a personally purchased nonqualified annuity.
    Pension,
    /// Nonqualified commercial annuity, Form 1099-R code D, a charitable gift annuity, code F, or a qualified code W charge
    /// Taxable annuity amounts enter net investment income; a qualified code W charge is excluded. Use the payer's determined taxable amount or a box 2b method; not an IRA, qualified plan, foreign employer pension or payment for services.
    Nonqualified,
    /// Inherited IRA distribution (code 4 with box 7b checked)
    /// Taxed with the Inherited IRA record's group, never with the filer's own IRA basis, Form 8606 or QCD limit.
    Inherited,
}
impl RetirementKind {
    /// An IRA, the filer's own or an inherited one: the 1099-R box 7b
    /// IRA/SEP/SIMPLE checkbox, which decides the Form 1040 line 4 category.
    pub(crate) fn ira(self) -> bool {
        matches!(self, Self::Ira | Self::Inherited)
    }
}

/// Additional printed retirement-statement boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RetirementPrintedBoxes {
    /// Box 2b taxable amount not determined
    #[libertas_default(false)]
    pub taxable_not_determined: bool,
    /// Box 2b total distribution
    #[libertas_default(false)]
    pub total_distribution: bool,
    /// Box 5 employee contributions / designated Roth contributions or insurance premiums
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employee_contributions: i64,
    /// Box 7d earnings on excess contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub excess_contribution_earnings: i64,
    /// Box 11 first year of designated Roth contributions
    #[libertas_default(0)]
    pub first_roth_year: i32,
    /// Box 12 FATCA filing requirement
    #[libertas_default(false)]
    pub fatca: bool,
    /// Box 13 date of payment
    /// [NullText]
    /// No date printed in box 13
    /// [SetText]
    /// The payment date printed in box 13 is
    #[libertas_date_only]
    #[libertas_default($null)]
    pub payment_date: Option<u32>,
    /// RRB-1099-R box 12 Medicare premium total
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub medicare_premiums: i64,
}
impl R1099 {
    pub(crate) fn payer_taxable_is_settled(&self) -> bool {
        !self.taxable_not_determined && !self.ira && self.taxable.is_some()
    }

    pub(crate) fn printed_taxable_method(&self) -> Option<TaxableMethod> {
        (!self.taxable_not_determined
            || DistributionCodes::methods(self) == [0]
            || DistributionCodes::designated_roth_qualified(self.box7, self.first_roth_year))
        .then_some(TaxableMethod::BoxTwoA)
    }

    pub(crate) fn form_boxes(&self) -> RetirementPrintedBoxes {
        RetirementPrintedBoxes {
            taxable_not_determined: self.taxable_not_determined,
            total_distribution: self.total_distribution,
            employee_contributions: self.employee_contributions.unwrap_or(0),
            excess_contribution_earnings: self.excess_contribution_earnings.unwrap_or(0),
            first_roth_year: self.first_roth_year,
            fatca: self.fatca,
            payment_date: self.payment_date,
            medicare_premiums: 0,
        }
    }
}

/// Retirement distribution
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRetirement {
    /// Additional printed boxes, retained independently of the taxable treatment
    pub form_boxes: RetirementPrintedBoxes,
    /// Form 1099-R box 2a taxable amount
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
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Form 1099-R box 9a share
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Form 1099-R box 7b IRA/SEP/SIMPLE as printed
    pub ira: bool,
    /// Form 1099-R box 7c Trump account as printed
    pub trump: bool,
    /// Taxable disability pension qualifying for Schedule R
    /// Use None for ordinary retirement or IRA withdrawals. This amount is already included in the taxable pension; it does not become earned income after minimum retirement age.
    pub disability: crate::DisabilityIncome,
    /// Retirement statement category
    pub kind: RetirementKind,
    /// Payments before the plan's minimum retirement age, a code 3 pension's wages
    /// Form 1040 line 1h; an IRA or any other kind has none.
    pub disability_wages: DisabilityWages,
    /// Form 1099-R box 1 or RRB-1099-R box 7 gross distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// How to determine the taxable amount
    /// Use the IRA worksheet when IRA basis must be allocated, even if the payer reported an amount. For a qualifying pension choose the Simplified Method; other undetermined distributions require another worksheet.
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// Taxable amount method
    pub taxable: Option<crate::RetirementTaxable>,
    /// Form 1099-R box 4 or RRB-1099-R box 9 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Form 1099-R box 7a distribution codes
    /// As printed: one code, or two printed together.
    #[libertas_bitflags(DistributionCodes)]
    pub box7: u32,
    /// Distribution treatment
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// Distribution treatment
    pub ordinary_distribution: Option<crate::RetirementTreatment>,
    /// This was a qualified disaster distribution
    pub disaster: crate::DisasterDistribution,
    /// The part paid after the owner's death
    /// Only for a person who died in 2026: a payment after the death is income in respect of a decedent and is left off the final return.
    #[libertas_default($null)]
    pub after_death: Option<crate::RetirementAfterDeath>,
    /// Form 1099-R box 10 amount allocable to an in-plan Roth rollover within 5 years
    /// Added to the early-distribution amount of a designated Roth account distribution (Form 5329 line 1).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_10: i64,
    /// The part of box 10 that was not taxable when rolled in (Pub. 575 Recapture Allocation Chart, Nontaxable column)
    /// Not known leaves the additional tax on a designated Roth account distribution uncalculated.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_10_basis: Option<i64>,
    /// Form 1099-R box 9b total employee contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_9b: Option<i64>,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for FederalRetirement {}

impl FederalRetirement {
    pub(crate) fn care_charge_rule(&self) -> Option<&'static str> {
        let has_w = self.box7 & DistributionCodes::CODE_W.0 != 0;
        match self.ordinary_distribution {
            Some(crate::RetirementTreatment::QualifiedCareCharge) => {
                let valid = (self.box7 == DistributionCodes::CODE_W.0
                    || self.box7 == DistributionCodes::CODE_W.0 | DistributionCodes::CODE_6.0)
                    && self.kind == RetirementKind::Nonqualified
                    && (self.taxable == Some(crate::RetirementTaxable::Payer) && self.box_2a == 0)
                    && (0..=self.gross).contains(&self.box_8a)
                    && matches!(self.disaster, crate::DisasterDistribution::No)
                    && matches!(self.disability, crate::DisabilityIncome::None)
                    && self.disability_wages == DisabilityWages::None;
                (!valid).then_some("R_CARE_CHARGE_SOURCE_INVALID")
            }
            _ => has_w.then_some("R_CARE_CHARGE_SOURCE_INVALID"),
        }
    }
    pub(crate) fn excluded_contract_source(&self) -> bool {
        self.box7 == DistributionCodes::CODE_6.0
            || matches!(
                self.ordinary_distribution,
                Some(crate::RetirementTreatment::QualifiedCareCharge)
            )
    }
    pub(crate) fn annuity_benefit_source(&self) -> bool {
        self.kind == RetirementKind::Nonqualified && !self.excluded_contract_source()
    }
    // The part of an employer disability pension that is wages on Form 1040
    // line 1h: the taxable payments before the plan's minimum retirement age
    // (Pub. 525 "Disability Pensions"; the part from the day after is line 5b).
    // Only a pension carries it; an IRA never does.
    pub(crate) fn disability_wages(&self, taxable: i64) -> i64 {
        if !self.code_3_pension() {
            return 0;
        }
        match self.disability_wages {
            DisabilityWages::None => 0,
            DisabilityWages::All => phase::floor(taxable, 0),
            DisabilityWages::Part { amount } => phase::floor(phase::cap(amount, [taxable]), 0),
        }
    }
    // Whether the page answered that some payments are wages.
    pub(crate) fn reports_disability_wages(&self) -> bool {
        self.code_3_pension()
            && matches!(self.disability_wages, DisabilityWages::All | DisabilityWages::Part { .. })
    }
    // A pension distribution under the permanent-and-total-disability exception.
    pub(super) fn code_3_pension(&self) -> bool {
        self.kind == RetirementKind::Pension
            && matches!(
                self.ordinary_distribution,
                Some(crate::RetirementTreatment::Exception {
                    facts: crate::RetirementException::Disability { .. },
                    ..
                })
            )
    }
}

/// Retirement document
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum RetirementDocument {
    /// 1099-R
    /// Pension, annuity, retirement plan or IRA distribution.
    R1099(Form1099RPaper),
    /// RRB-1099-R
    /// Railroad retirement annuity from the Railroad Retirement Board: tier 2, vested dual benefits and supplemental annuities. Social Security-equivalent benefits use the separate RRB-1099 entry.
    Rrb1099R(crate::FormRrb1099RPaper),
    /// SSA-1099
    Ssa1099(Ssa1099Paper),
    /// RRB-1099 Social Security-equivalent railroad benefits
    Rrb1099(crate::rrb_document::Rrb1099Paper),
}

/// 1099-R
/// Enter it from your Form 1099-R, or from your Form 4852 line 8 if your payer gave you none. A retired minister's pension housing designation is not excluded here; its otherwise taxable pension amount remains taxable.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Preserve each printed address row. Where state/province, country and ZIP/foreign postal code occupy separate columns of the same row, join their nonblank cells with spaces on that row; never insert a newline between them and never add a country the paper does not print. Keep city/town on the line where it is printed, and do not include separately labeled telephone information in the address. Never infer a recipient from household names. Read the 1099-R exactly. Check its tax year against the supplied year; report a mismatch. Box 1 must show an amount; report a blank box 1 as an error. Return box 2a's printed amount, or null when blank; never substitute box 1. Read both box 2b checkboxes exactly. Read box 5 employee contributions/designated Roth contributions or insurance premiums, returning null when blank. Return null for a clearly blank box 4. Return the codes printed in box 7a (distribution codes): one code, or two when two are printed together such as 4G, 7D or 1B, as the mask of those codes; report a blank box 7a, more than two codes or a code not in the catalog as an error, and never add a code that is not printed. Return true for box 7b IRA/SEP/SIMPLE only when it is checked. Return true for box 7c Trump account only when it is checked. Read box 7d earnings on excess contributions, returning null when blank; it is separate from box 1. Read box 11 first year of designated Roth contributions as the actual printed year, or null when clearly blank; report an illegible year as an error and never infer it. Read box 12 FATCA as checked or unchecked, and box 13 date of payment as a date, or null when blank. In the other boxes, return the box 3, box 6, box 8a and box 10 amounts, or null when clearly blank; and box 8b's and box 9a's percentages as printed with up to two decimals, or null when blank; and the box 9b amount, or null when blank. Report illegible or conflicting values as errors. Do not infer disaster, public safety officer, Simplified Method, disability or other facts absent from the 1099-R; the filer answers what the box 7a codes lead to. The file may hold several copies of this form, such as Copy B, Copy C and Copy 2; copies of the same form are one form. Set state_details.statement to Form1099R for this Form 1099-R. Read the CORRECTED checkbox into corrected. Copy the payer telephone and account number exactly as printed, or empty text when absent; never use the text 0 for a blank. Read the payer name, federal ID and address, state boxes 14 through 16, and local boxes 17 through 19. This import is for Form 1099-R, not an RRB-1099-R. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return null for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.gross,@.taxable,@.box_3,@.withholding,@.employee_contributions,@.box_6,@.excess_contribution_earnings,@.box_8a,@.box_9b,@.box_10,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099RPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 gross distribution
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 2a taxable amount
    /// [NullText]
    /// Box 2a is blank
    /// [SetText]
    /// The amount printed in box 2a is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: Option<i64>,
    /// Box 2b taxable amount not determined
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub taxable_not_determined: bool,
    /// Box 2b total distribution
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub total_distribution: bool,
    /// Box 3 capital gain (included in box 2a)
    /// Retained as printed; the Form 4972 capital gain election uses this amount.
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The amount printed in box 3 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_3: Option<i64>,
    /// Box 4 federal income tax withheld
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The amount printed in box 4 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 5 employee contrib./desig. Roth contrib. or insurance premiums
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The amount printed in box 5 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub employee_contributions: Option<i64>,
    /// Box 6 NUA in employer's securities
    /// Retained as printed. Not in box 2a; a Form 4972 election may include it now.
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The amount printed in box 6 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_6: Option<i64>,
    /// Box 7a dist. code(s)
    /// One code, or two printed together such as 4G.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_bitflags(DistributionCodes)]
    pub box7: u32,
    /// Box 7b IRA/SEP/SIMPLE
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub ira: bool,
    /// Box 7c Trump account
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub trump: bool,
    /// Box 7d earnings on excess contrib.
    /// [NullText]
    /// Box 7d is blank
    /// [SetText]
    /// The amount printed in box 7d is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub excess_contribution_earnings: Option<i64>,
    /// Box 8a other
    /// For code W, enter the actual investment reduction as printed, including zero; this is not the full care charge or exchange value. It is retained once, and other-payment investment inputs must reflect the actual already-adjusted basis. For Form 4972, this is the value of an annuity contract paid in the lump sum, outside boxes 1/2a. Retained independently of the treatment election.
    /// [NullText]
    /// Box 8a is blank
    /// [SetText]
    /// The amount printed in box 8a is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_8a: Option<i64>,
    /// Box 8b percentage of annuity contract, as printed, with up to two decimals
    /// Choose “Box 8b is blank” unless box 8b is printed (your share of an annuity contract paid in a lump sum shared with others). Retained independently of the Form 4972 election.
    /// [NullText]
    /// Box 8b is blank
    /// [SetText]
    /// The box 8b annuity-contract share is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Box 9a percentage of total distribution, as printed, with up to two decimals
    /// Choose “Box 9a is blank” unless box 9a is printed (a total distribution to more than one person). Boxes 1 and 2a are already your share; only a Form 4972 election uses it.
    /// [NullText]
    /// Box 9a is blank
    /// [SetText]
    /// The box 9a share of the total distribution is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Box 9b total employee contrib.
    /// Printed by some payers; choose “Box 9b is blank” when blank. Under the Simplified Method it is your cost when the annuity started in 2026.
    /// [NullText]
    /// Box 9b is blank
    /// [SetText]
    /// The box 9b employee contributions are
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_9b: Option<i64>,
    /// Box 10 amount allocable to IRR within 5 years
    /// Only with code B. With an early distribution this amount is subject to the 10% additional tax, as Form 5329 line 1 says.
    /// [NullText]
    /// Box 10 is blank
    /// [SetText]
    /// The amount printed in box 10 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_10: Option<i64>,
    /// Box 11 1st year of desig. Roth contrib.
    /// Choose “Box 11 is blank” when no year is printed.
    /// [NullText]
    /// Box 11 is blank
    /// [SetText]
    /// The first year printed in box 11 is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_format("0")]
    pub first_roth_year: Option<i32>,
    /// Box 12 FATCA filing requirement
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub fatca: bool,
    /// Box 13 date of payment
    /// [NullText]
    /// No date printed in box 13
    /// [SetText]
    /// The payment date printed in box 13 is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub payment_date: Option<u32>,
    /// Boxes 14 through 19 state and local, and the payer identity for the state return
    #[libertas_llm_doc_field]
    pub state_details: crate::RetirementStateFields,
}

/// 1099-R as the calculations read it
/// Derived from the stored paper, `Form1099RPaper`.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct R1099 {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// PAYER'S telephone number
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Box 1 gross distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross: i64,
    /// Box 2a taxable amount
    /// [NullText]
    /// Box 2a is blank
    /// [SetText]
    /// The amount printed in box 2a is
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable: Option<i64>,
    /// Box 2b taxable amount not determined
    #[libertas_default(false)]
    pub taxable_not_determined: bool,
    /// Box 2b total distribution
    #[libertas_default(false)]
    pub total_distribution: bool,
    /// Box 4 federal income tax withheld
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The amount printed in box 4 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 5 employee contributions / designated Roth contributions or insurance premiums
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The amount printed in box 5 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub employee_contributions: Option<i64>,
    /// Box 7a distribution codes
    /// One code, or two printed together such as 4G.
    #[libertas_bitflags(DistributionCodes)]
    pub box7: u32,
    /// Box 7b IRA/SEP/SIMPLE
    #[libertas_default(false)]
    pub ira: bool,
    /// Box 7c Trump account
    #[libertas_default(false)]
    pub trump: bool,
    /// Box 7d earnings on excess contributions
    /// [NullText]
    /// Box 7d is blank
    /// [SetText]
    /// The amount printed in box 7d is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub excess_contribution_earnings: Option<i64>,
    /// Box 3 capital gain (included in box 2a)
    /// Retained as printed; the Form 4972 capital gain election uses this amount.
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The amount printed in box 3 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_3: Option<i64>,
    /// Box 6 net unrealized appreciation in employer's securities
    /// Retained as printed. Not in box 2a; a Form 4972 election may include it now.
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The amount printed in box 6 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_6: Option<i64>,
    /// Box 8a other
    /// For code W, enter the actual investment reduction as printed, including zero; this is not the full care charge or exchange value. It is retained once, and other-payment investment inputs must reflect the actual already-adjusted basis. For Form 4972, this is the value of an annuity contract paid in the lump sum, outside boxes 1/2a. Retained independently of the treatment election.
    /// [NullText]
    /// Box 8a is blank
    /// [SetText]
    /// The amount printed in box 8a is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_8a: Option<i64>,
    /// Box 8b share of the annuity contract, as printed, with up to two decimals
    /// Choose “Box 8b is blank” unless box 8b is printed (your share of an annuity contract paid in a lump sum shared with others). Retained independently of the Form 4972 election.
    /// [NullText]
    /// Box 8b is blank
    /// [SetText]
    /// The box 8b annuity-contract share is
    #[libertas_default($null)]
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_8b_percent: Option<f64>,
    /// Your share of total distribution, box 9a, as printed, with up to two decimals
    /// Choose “Box 9a is blank” unless box 9a is printed (a total distribution to more than one person). Boxes 1 and 2a are already your share; only a Form 4972 election uses it.
    /// [NullText]
    /// Box 9a is blank
    /// [SetText]
    /// The box 9a share of the total distribution is
    #[libertas_default($null)]
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0.01, max = 99.99, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub box_9a_percent: Option<f64>,
    /// Box 9b total employee contributions
    /// Printed by some payers; choose “Box 9b is blank” when blank. Under the Simplified Method it is your cost when the annuity started in 2026.
    /// [NullText]
    /// Box 9b is blank
    /// [SetText]
    /// The box 9b employee contributions are
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_9b: Option<i64>,
    /// Box 10 amount allocable to an in-plan Roth rollover
    /// Only with code B. With an early distribution this amount is subject to the 10% additional tax, as Form 5329 line 1 says.
    /// [NullText]
    /// Box 10 is blank
    /// [SetText]
    /// The amount printed in box 10 is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_10: Option<i64>,
    /// Box 11 first year of designated Roth contributions
    #[libertas_default(0)]
    pub first_roth_year: i32,
    /// Box 12 FATCA filing requirement
    #[libertas_default(false)]
    pub fatca: bool,
    /// Box 13 date of payment
    /// [NullText]
    /// No date printed in box 13
    /// [SetText]
    /// The payment date printed in box 13 is
    #[libertas_date_only]
    #[libertas_default($null)]
    pub payment_date: Option<u32>,
    /// Account number
    #[libertas_default("")]
    pub account_number: String,
    /// Boxes 14 through 19 state and local, and the payer identity for the state return
    pub state_details: crate::RetirementStateFields,
}

// One-way: every printed box reaches the internal record, so a new box cannot be forgotten.
impl From<&Form1099RPaper> for R1099 {
    fn from(paper: &Form1099RPaper) -> Self {
        let Form1099RPaper {
            calendar_year: _,
            corrected: _,
            payer,
            payer_telephone,
            account_number,
            recipient,
            gross,
            taxable,
            taxable_not_determined,
            total_distribution,
            box_3,
            withholding,
            employee_contributions,
            box_6,
            box7,
            ira,
            trump,
            excess_contribution_earnings,
            box_8a,
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10,
            first_roth_year,
            fatca,
            payment_date,
            state_details,
        } = paper;
        Self {
            payer: payer.clone(),
            payer_telephone: payer_telephone.clone(),
            account_number: account_number.clone(),
            recipient: recipient.clone(),
            gross: *gross,
            taxable: *taxable,
            taxable_not_determined: *taxable_not_determined,
            total_distribution: *total_distribution,
            withholding: *withholding,
            employee_contributions: *employee_contributions,
            box7: *box7,
            ira: *ira,
            trump: *trump,
            excess_contribution_earnings: *excess_contribution_earnings,
            box_3: *box_3,
            box_6: *box_6,
            box_8a: *box_8a,
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            box_9b: *box_9b,
            box_10: *box_10,
            first_roth_year: first_roth_year.unwrap_or(0),
            fatca: *fatca,
            payment_date: *payment_date,
            state_details: state_details.clone(),
        }
    }
}
// Preserve the printed absence and text when reopening the stored source.
impl From<&R1099> for Form1099RPaper {
    fn from(form: &R1099) -> Self {
        let R1099 {
            payer,
            payer_telephone,
            account_number,
            recipient,
            gross,
            taxable,
            taxable_not_determined,
            total_distribution,
            withholding,
            employee_contributions,
            box7,
            ira,
            trump,
            excess_contribution_earnings,
            box_3,
            box_6,
            box_8a,
            box_8b_percent,
            box_9a_percent,
            box_9b,
            box_10,
            first_roth_year,
            fatca,
            payment_date,
            state_details,
        } = form;
        Self {
            calendar_year: 0,
            corrected: false,
            payer: payer.clone(),
            payer_telephone: payer_telephone.clone(),
            account_number: account_number.clone(),
            recipient: recipient.clone(),
            gross: *gross,
            taxable: *taxable,
            taxable_not_determined: *taxable_not_determined,
            total_distribution: *total_distribution,
            box_3: *box_3,
            withholding: *withholding,
            employee_contributions: *employee_contributions,
            box_6: *box_6,
            box7: *box7,
            ira: *ira,
            trump: *trump,
            excess_contribution_earnings: *excess_contribution_earnings,
            box_8a: *box_8a,
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            box_9b: *box_9b,
            box_10: *box_10,
            first_roth_year: (*first_roth_year != 0).then_some(*first_roth_year),
            fatca: *fatca,
            payment_date: *payment_date,
            state_details: state_details.clone(),
        }
    }
}

/// Accepted Form 1099-R
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct R1099Source {
    /// Form 1099-R as printed
    pub form: R1099,
    /// What the box 7a codes mean for you
    #[libertas_default($null)]
    pub code: Option<DistributionCode>,
    /// Box 2b taxable amount not determined: how to figure it
    #[libertas_default($null)]
    pub taxable_method: Option<TaxableMethod>,
    /// Disability retirement pay for Schedule R
    #[libertas_default($null)]
    pub disability: Option<crate::DisabilityIncome>,
    /// Part of box 10 that was not taxable when rolled into the designated Roth account
    #[libertas_default($null)]
    pub box_10_basis: Option<InPlanRothBasis>,
    /// This was a qualified disaster distribution
    #[libertas_default($null)]
    pub disaster: Option<crate::DisasterDistribution>,
    /// Paid after the death
    #[libertas_default($null)]
    pub after_death: Option<crate::RetirementAfterDeath>,
    /// The owner's Form 8606 is on this return, so the IRA basis worksheet taxes this distribution
    /// Set and cleared by the save once every document of the owner is known; never asked.
    #[libertas_default(false)]
    pub ira_basis: bool,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for R1099Source {}

impl R1099Source {
    pub(crate) fn effective_taxable_method(&self) -> Option<&TaxableMethod> {
        if self.form.payer_taxable_is_settled() {
            Some(&TaxableMethod::BoxTwoA)
        } else {
            self.taxable_method.as_ref().or_else(|| {
                (DistributionCodes::methods(&self.form) == [0]).then_some(&TaxableMethod::BoxTwoA)
            })
        }
    }

    /// A newly entered form: no topic answer is given yet beyond what its boxes decide.
    pub(crate) fn printed(form: R1099) -> Self {
        let mut source = Self {
            taxable_method: form.printed_taxable_method(),
            form,
            code: None,
            disability: None,
            box_10_basis: None,
            disaster: None,
            after_death: None,
            ira_basis: false,
        };
        source.decided_by_boxes();
        source
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.code = accepted.code.clone();
        let printed_method = self.form.printed_taxable_method();
        self.taxable_method = accepted
            .taxable_method
            .clone()
            .filter(|method| {
                DistributionCodes::methods(&self.form).contains(&method.index())
                    && (!matches!(method, TaxableMethod::BoxTwoA) || printed_method.is_some())
            })
            .or(printed_method);
        self.disability = accepted.disability.clone();
        self.box_10_basis = accepted.box_10_basis;
        self.disaster = accepted.disaster.clone();
        self.after_death = accepted.after_death.clone();
        self.ira_basis = accepted.ira_basis;
        self.decided_by_boxes();
    }
    // A sole method determined by the printed boxes needs no retained answer.
    // A Trump account's returned excess contribution (box 7c with code 8) asks nothing.
    fn decided_by_boxes(&mut self) {
        if self.form.payer_taxable_is_settled()
            || (self.taxable_method.is_none() && DistributionCodes::methods(&self.form) == [0])
        {
            self.taxable_method = Some(TaxableMethod::BoxTwoA);
        }
        if self.form.trump && self.form.box7 == DistributionCodes::CODE_8.0 {
            self.code = Some(DistributionCode::Corrective { excess_deferral: 0 });
            self.taxable_method = Some(TaxableMethod::BoxTwoA);
        }
    }
    /// The repayment an early-distribution exception lists lives in the owner's IRA basis
    /// record, not here.
    pub(crate) fn without_repayment(mut self) -> Self {
        if let Some(
            DistributionCode::Early { exception, .. }
            | DistributionCode::EarlySimple {
                what_happened: crate::SimpleDisposition::Kept { exception },
            }
            | DistributionCode::ProhibitedTransaction { exception },
        ) = &mut self.code
            && let EarlyException::Applies { repaid, .. } = exception
        {
            *repaid = None;
        }
        self
    }
    pub(crate) fn disaster(&self) -> &crate::DisasterDistribution {
        self.disaster.as_ref().unwrap_or(&crate::DisasterDistribution::No)
    }
    pub(crate) fn disability(&self) -> crate::DisabilityIncome {
        self.disability.clone().unwrap_or(crate::DisabilityIncome::None)
    }
    pub(crate) fn box_10_basis(&self) -> Option<i64> {
        self.box_10_basis.map(InPlanRothBasis::amount)
    }
}

// Working-copy changes of the estimate's scope rules (core/unknowns.rs): a fact the filer could
// not confirm is left out of the working draft, never of the stored source.
impl RetirementSource {
    /// The part paid after the owner's death is left off the working draft.
    pub(crate) fn clear_after_death(&mut self) {
        match self {
            Self::R1099(source) => source.after_death = None,
            // A railroad annuity has no part paid after a death.
            Self::Rrb1099R(_) => {}
        }
    }
    /// The working draft's distribution without a part paid after the death.
    pub(crate) fn reduce_distribution(&mut self, gross: i64, taxable: i64) {
        match self {
            Self::R1099(source) => {
                source.form.gross -= gross;
                source.form.taxable = Some(source.form.taxable.unwrap_or(0) - taxable);
            }
            Self::Rrb1099R(record) => record.form.gross -= gross,
        }
    }
    /// The transient no-benefit calculation taxes the whole distribution.
    pub(crate) fn tax_payer_amount(&mut self) {
        match self {
            Self::R1099(source) => {
                source.taxable_method = Some(TaxableMethod::BoxTwoA);
                source.form.taxable = Some(source.form.gross);
                // This working amount is determined; the accepted paper remains unchanged.
                source.form.taxable_not_determined = false;
            }
            // The railroad annuity's basis recovery is cleared by clear_railroad_basis.
            Self::Rrb1099R(_) => {}
        }
    }
    /// No Schedule R credit on this disability income.
    pub(crate) fn leave_out_disability(&mut self) {
        match self {
            Self::R1099(source) => source.disability = None,
            // A railroad annuity has no disability income.
            Self::Rrb1099R(_) => {}
        }
    }
    /// A Form 4972 election that cannot be used is taxed with the other income.
    pub(crate) fn lump_sum_with_other_income(&mut self) {
        match self {
            Self::R1099(source) => match &mut source.code {
                Some(DistributionCode::NormalLumpSum { tax }) => {
                    *tax = LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept };
                }
                Some(DistributionCode::DeathLumpSum { tax, .. }) => {
                    *tax = InheritedLumpSumTax::WithOtherIncome;
                }
                _ => {}
            },
            // A railroad annuity has no Form 4972 election.
            Self::Rrb1099R(_) => {}
        }
    }
    /// Earlier-year railroad repayments have no current income effect.
    pub(crate) fn clear_railroad_repayments(&mut self) {
        if let Self::Rrb1099R(record) = self {
            record.repayments.clear();
        }
    }
    /// An unconfirmed railroad basis recovers no cost.
    pub(crate) fn clear_railroad_basis(&mut self) {
        if let Self::Rrb1099R(record) = self
            && let Some(
                crate::RailroadRecipient::Employee { recovery }
                | crate::RailroadRecipient::Survivor { recovery, .. },
            ) = &mut record.recipient_kind
        {
            *recovery = crate::RailroadRecovery::None;
        }
    }
}

/// A Form 1099-R with the topic answers its rules read, and the insurance premiums its
/// public-safety answer lists (those live in the screening records, not in the form).
#[derive(Clone, Debug)]
pub struct R1099Answered {
    source: R1099Source,
    public_safety: Vec<crate::PublicSafetyPaid>,
}
impl R1099Answered {
    pub(crate) fn new(
        mut source: R1099Source,
        public_safety: Vec<crate::PublicSafetyPaid>,
    ) -> Self {
        source.decided_by_boxes();
        Self { source, public_safety }
    }
    pub(crate) fn public_safety(&self) -> &[crate::PublicSafetyPaid] {
        &self.public_safety
    }
    pub(crate) fn set_public_safety(&mut self, rows: Vec<crate::PublicSafetyPaid>) {
        self.public_safety = rows;
    }
    pub(crate) fn source_mut(&mut self) -> &mut R1099Source {
        &mut self.source
    }
    pub(crate) fn into_source(self) -> R1099Source {
        self.source
    }
}
// The rules read the paper's boxes and the answers beside each other.
impl core::ops::Deref for R1099Answered {
    type Target = R1099Source;
    fn deref(&self) -> &R1099Source {
        &self.source
    }
}
impl core::ops::Deref for R1099Source {
    type Target = R1099;
    fn deref(&self) -> &R1099 {
        &self.form
    }
}

/// Retirement distribution as accepted
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum RetirementSource {
    /// Form 1099-R and its topic answers
    R1099(R1099Source),
    /// RRB-1099-R railroad annuity
    Rrb1099R(crate::RailroadSource),
}
impl Eq for RetirementSource {}

impl From<&RetirementSource> for FederalRetirement {
    fn from(source: &RetirementSource) -> Self {
        match source {
            RetirementSource::R1099(source) => source.into(),
            RetirementSource::Rrb1099R(record) => record.into(),
        }
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&R1099Source> for FederalRetirement {
    fn from(source: &R1099Source) -> Self {
        let R1099Source {
            form,
            code,
            taxable_method: _,
            disability,
            box_10_basis: _,
            disaster: _,
            after_death,
            ira_basis,
        } = source;
        let R1099 {
            // The parties identify the statement; no figure reads them.
            payer: _,
            payer_telephone: _,
            account_number: _,
            recipient: _,
            gross,
            taxable,
            // Printed 2b, 5, 7d, 11, 12 and 13 are retained through form_boxes.
            taxable_not_determined,
            total_distribution,
            box_3,
            withholding,
            employee_contributions,
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
            // Reaches the state return through the accepted entry.
            state_details: _,
        } = form;
        // The kind comes from the printed box 7a and box 7b, not from the branch:
        // code D printed with 1, 2, 3, 4 or 7 is a nonqualified annuity too.
        // Code F, a charitable gift annuity, is a nonqualified annuity too, and a
        // recharacterization (code N) moves an IRA contribution: box 7b is unchecked for both.
        // Code J, an early distribution from a Roth IRA, is an IRA amount even when the payer
        // leaves box 7b unchecked (a Roth IRA payer's usual print): Form 1040 lines 4a/4b.
        let kind = if box7
            & (DistributionCodes::CODE_D.0
                | DistributionCodes::CODE_F.0
                | DistributionCodes::CODE_W.0)
            != 0
        {
            RetirementKind::Nonqualified
        } else if *ira && box7 & DistributionCodes::CODE_4.0 != 0 {
            RetirementKind::Inherited
        } else if *ira || box7 & (DistributionCodes::CODE_N.0 | DistributionCodes::CODE_J.0) != 0 {
            RetirementKind::Ira
        } else {
            RetirementKind::Pension
        };
        let sold = code
            .as_ref()
            .and_then(crate::retirement::disposition_of)
            .and_then(Disposition::property)
            .cloned();
        use RetirementTaxable as T;
        use TaxableMethod as M;
        let taxable_kind = match (code, source.effective_taxable_method().cloned()) {
            (Some(DistributionCode::Inherited { account }), Some(_)) => {
                Some(T::Inherited { account: *account })
            }
            // An empty cost takes box 9b (rule R_COST_NEEDS_BOX_9B); the stored worksheet always
            // holds the resolved cost.
            (_, Some(M::Simplified { mut data })) => {
                data.cost = data.cost.or(*box_9b);
                Some(T::Simplified { data })
            }
            (_, Some(M::GeneralRule { data })) => Some(T::GeneralAnnuity { data }),
            (_, Some(M::VariablePayments { data })) => Some(T::VariableAnnuity { data }),
            (_, Some(M::WholeContract { mut data })) => {
                // IRS 1099-R instructions, boxes 2b and 5: a total distribution's
                // box 5 is the remaining investment available for tax-free recovery.
                // An undetermined statement leaves that box blank, so its topic
                // worksheet still needs the independent investment records.
                if *total_distribution && !taxable_not_determined {
                    data.investment = employee_contributions.unwrap_or(0);
                }
                Some(T::FullDischarge { data })
            }
            (_, Some(M::PlanWithdrawal { data })) => Some(T::QualifiedWithdrawal { data }),
            (_, Some(M::AnnuityWithdrawal { data })) => Some(T::DeferredWithdrawal { data }),
            (_, Some(M::Pre1982Withdrawal { data })) => Some(T::LegacyAnnuity { data }),
            (_, Some(M::CivilServiceLumpSum { data })) => Some(T::CivilServiceDeath { data }),
            (_, Some(M::ChildAnnuity { data })) => Some(T::ChildAnnuity { data }),
            (_, Some(M::BoxTwoA)) => match sold {
                Some(data) => Some(T::PropertyProceeds { data }),
                None => Some(T::Payer),
            },
            (_, None) => None,
        };
        // The IRA basis worksheet replaces every other taxable method of an IRA distribution.
        let taxable_kind = if *ira_basis { Some(T::IraBasis) } else { taxable_kind };
        FederalRetirement {
            form_boxes: form.form_boxes(),
            trump: *trump,
            box_2a: taxable.unwrap_or(0),
            box_3: box_3.unwrap_or(0),
            box_6: box_6.unwrap_or(0),
            box_8a: box_8a.unwrap_or(0),
            box_8b_percent: *box_8b_percent,
            box_9a_percent: *box_9a_percent,
            ira: *ira,
            disability: disability.clone().unwrap_or(crate::DisabilityIncome::None),
            kind,
            // Only a code 3 pension carries the minimum-retirement-age answer.
            disability_wages: match code {
                Some(DistributionCode::Disability { wages, .. })
                    if kind == RetirementKind::Pension =>
                {
                    *wages
                }
                _ => crate::DisabilityWages::None,
            },
            gross: *gross,
            taxable: taxable_kind,
            withholding: withholding.unwrap_or(0),
            box7: *box7,
            ordinary_distribution: code.as_ref().map(|code| distribution_treatment(code, form)),
            disaster: source.disaster().clone(),
            after_death: after_death.clone(),
            box_10: box_10.unwrap_or(0),
            box_10_basis: source.box_10_basis(),
            box_9b: *box_9b,
        }
    }
}

fn distribution_treatment(code: &DistributionCode, form: &R1099) -> RetirementTreatment {
    let (gross, taxable, box7) = (form.gross, form.taxable.unwrap_or(0), form.box7);
    match code {
        // A code G box 2a above zero is the taxable part of a rollover to a Roth IRA or an
        // in-plan Roth rollover, which the person chooses as the Roth target (taxed in the Roth
        // rollover treatment). Box 2a with another target (box 2a left blank on a plan
        // rollover) is taxed as printed, with no rollover.
        DistributionCode::DirectRollover { to } if taxable > 0 && *to != RolloverTarget::Roth => {
            RetirementTreatment::Ordinary
        }
        DistributionCode::NormalLumpSum {
            tax: crate::LumpSumTax::WithOtherIncome { what_happened },
        } => disposition_treatment(what_happened, false),
        DistributionCode::NormalLumpSum { tax } => crate::Form4972Election::from_own(tax)
            .map_or(RetirementTreatment::Unsupported, |data| RetirementTreatment::LumpSum { data }),
        DistributionCode::DeathLumpSum {
            died,
            tax: crate::InheritedLumpSumTax::WithOtherIncome,
        } => RetirementTreatment::Exception {
            distributed: 20261231,
            facts: crate::RetirementException::Death { died: *died },
            second: None,
            early_simple: false,
        },
        DistributionCode::DeathLumpSum { died, tax } => {
            crate::Form4972Election::from_decedent(*died, tax)
                .map_or(RetirementTreatment::Unsupported, |data| RetirementTreatment::LumpSum {
                    data,
                })
        }
        DistributionCode::Normal { what_happened } => disposition_treatment(what_happened, false),
        DistributionCode::Early {
            what_happened: Disposition::Kept,
            exception: EarlyException::Applies { distributed, facts, second, .. },
        }
        | DistributionCode::EarlySimple {
            what_happened:
                SimpleDisposition::Kept {
                    exception: EarlyException::Applies { distributed, facts, second, .. },
                },
        }
        | DistributionCode::ProhibitedTransaction {
            exception: EarlyException::Applies { distributed, facts, second, .. },
        } => RetirementTreatment::Exception {
            distributed: *distributed,
            facts: facts.clone(),
            second: second.clone(),
            early_simple: box7 & DistributionCodes::CODE_S.0 != 0,
        },
        // The whole IRA is treated as paid out on January 1, 2026: an early distribution
        // unless the owner was 59½ that day or an exception applies (retirement::early_tax).
        DistributionCode::ProhibitedTransaction { .. } => RetirementTreatment::Early,
        DistributionCode::Early { what_happened: Disposition::Kept, .. } => {
            RetirementTreatment::Early
        }
        DistributionCode::Early { what_happened, .. } => disposition_treatment(what_happened, true),
        DistributionCode::Disability { began, .. } => RetirementTreatment::Exception {
            distributed: form.payment_date.unwrap_or(20261231),
            facts: crate::RetirementException::Disability { began: *began },
            second: None,
            early_simple: box7 & DistributionCodes::CODE_S.0 != 0,
        },
        DistributionCode::Death { died } => RetirementTreatment::Exception {
            distributed: form.payment_date.unwrap_or(20261231),
            facts: crate::RetirementException::Death { died: *died },
            second: None,
            early_simple: box7 & DistributionCodes::CODE_S.0 != 0,
        },
        DistributionCode::DirectRollover { to } => RetirementTreatment::Rollover {
            destination: match to {
                RolloverTarget::Ira => crate::RolloverDestination::Ira,
                RolloverTarget::EmployerPlan => crate::RolloverDestination::EmployerPlan,
                RolloverTarget::Roth => crate::RolloverDestination::Roth,
            },
            amount: gross,
            early: false,
        },
        DistributionCode::EarlySimple { what_happened } => what_happened.treatment(),
        DistributionCode::EmployerRothContribution => RetirementTreatment::EmployerRothContribution,
        DistributionCode::Nonqualified
        | DistributionCode::Exchange
        | DistributionCode::NotIncome
        | DistributionCode::Inherited { .. } => RetirementTreatment::Ordinary,
        // A charitable gift annuity's box 3 capital gain (28% and section 1250 characters the
        // payer reports separately) is not calculated; every other payment is taxed as printed.
        DistributionCode::NoAdditionalTax
            if box7 & DistributionCodes::CODE_F.0 != 0 && form.box_3.unwrap_or(0) > 0 =>
        {
            RetirementTreatment::Unsupported
        }
        DistributionCode::NoAdditionalTax => RetirementTreatment::Ordinary,
        DistributionCode::Corrective { excess_deferral } => {
            RetirementTreatment::Corrective { excess_deferral: *excess_deferral }
        }
        DistributionCode::QualifiedCareCharge => RetirementTreatment::QualifiedCareCharge,
        DistributionCode::Other => RetirementTreatment::Unsupported,
        // The Roth IRA and Trump account codes file the form under their own income kinds.
        DistributionCode::RothIra { .. }
        | DistributionCode::TrumpDeath { .. }
        | DistributionCode::TrumpToAble { .. } => {
            unreachable!("this 1099-R is not a FederalRetirement record")
        }
    }
}
fn disposition_treatment(what_happened: &Disposition, early: bool) -> RetirementTreatment {
    match what_happened {
        Disposition::Kept => RetirementTreatment::Ordinary,
        Disposition::RolledOver { amount, into } => {
            RetirementTreatment::Rollover { destination: *into, amount: *amount, early }
        }
        Disposition::Converted { amount } => {
            RetirementTreatment::Conversion { amount: *amount, early }
        }
        Disposition::SoldAndRolledOver { amount, into, .. } => {
            RetirementTreatment::Rollover { destination: *into, amount: *amount, early }
        }
    }
}

// Form 1099-R box 7a as printed: one or two distribution codes (2026
// Instructions for Forms 1099-R and 5498, box 7a and Table 1, Guide to
// Distribution Codes). Bit i is the i-th code in the order 1-9, A-H, J-N,
// P-U, W, Y; R1099's ValidationRules spell these masks as numbers
// (retirement.rs DistributionCodes holds the Table 1 combinations).
/// Box 7a distribution codes
#[libertas_bitflags]
pub struct DistributionCodes(pub u32);
impl DistributionCodes {
    /// No code printed
    pub const NONE: Self = Self(0);
    /// 1 — early distribution, no known exception
    /// Usually under age 59½.
    #[libertas_form_code("1")]
    pub const CODE_1: Self = Self(1 << 0);
    /// 2 — early distribution, exception applies
    /// Under age 59½.
    #[libertas_form_code("2")]
    pub const CODE_2: Self = Self(1 << 1);
    /// 3 — disability
    #[libertas_form_code("3")]
    pub const CODE_3: Self = Self(1 << 2);
    /// 4 — death
    #[libertas_form_code("4")]
    pub const CODE_4: Self = Self(1 << 3);
    /// 5 — prohibited transaction
    #[libertas_form_code("5")]
    pub const CODE_5: Self = Self(1 << 4);
    /// 6 — section 1035 exchange
    /// A tax-free exchange of life insurance, annuity, qualified long-term care insurance or endowment contracts.
    #[libertas_form_code("6")]
    pub const CODE_6: Self = Self(1 << 5);
    /// 7 — normal distribution
    #[libertas_form_code("7")]
    pub const CODE_7: Self = Self(1 << 6);
    /// 8 — excess contributions or deferrals, taxable in 2026
    /// With their earnings.
    #[libertas_form_code("8")]
    pub const CODE_8: Self = Self(1 << 7);
    /// 9 — cost of current life insurance protection
    #[libertas_form_code("9")]
    pub const CODE_9: Self = Self(1 << 8);
    /// A — may be eligible for the 10-year tax option
    /// Form 4972.
    #[libertas_form_code("A")]
    pub const CODE_A: Self = Self(1 << 9);
    /// B — designated Roth account distribution
    #[libertas_form_code("B")]
    pub const CODE_B: Self = Self(1 << 10);
    /// C — reportable death benefits under section 6050Y
    #[libertas_form_code("C")]
    pub const CODE_C: Self = Self(1 << 11);
    /// D — annuity payments from a nonqualified annuity
    /// May be subject to the net investment income tax (section 1411).
    #[libertas_form_code("D")]
    pub const CODE_D: Self = Self(1 << 12);
    /// E — distribution under the Employee Plans Compliance Resolution System
    #[libertas_form_code("E")]
    pub const CODE_E: Self = Self(1 << 13);
    /// F — charitable gift annuity
    #[libertas_form_code("F")]
    pub const CODE_F: Self = Self(1 << 14);
    /// G — direct rollover
    /// To a qualified plan, a section 403(b) plan, a governmental section 457(b) plan or an IRA.
    #[libertas_form_code("G")]
    pub const CODE_G: Self = Self(1 << 15);
    /// H — direct rollover of a designated Roth account to a Roth IRA
    #[libertas_form_code("H")]
    pub const CODE_H: Self = Self(1 << 16);
    /// J — early distribution from a Roth IRA, no known exception
    #[libertas_form_code("J")]
    pub const CODE_J: Self = Self(1 << 17);
    /// K — traditional IRA assets without a readily available fair market value
    #[libertas_form_code("K")]
    pub const CODE_K: Self = Self(1 << 18);
    /// L — loan treated as a distribution
    #[libertas_form_code("L")]
    pub const CODE_L: Self = Self(1 << 19);
    /// M — qualified plan loan offset
    #[libertas_form_code("M")]
    pub const CODE_M: Self = Self(1 << 20);
    /// N — IRA contribution for 2026 recharacterized in 2026
    #[libertas_form_code("N")]
    pub const CODE_N: Self = Self(1 << 21);
    /// P — excess contributions or deferrals, taxable in 2025 or earlier
    #[libertas_form_code("P")]
    pub const CODE_P: Self = Self(1 << 22);
    /// Q — qualified distribution from a Roth IRA
    #[libertas_form_code("Q")]
    pub const CODE_Q: Self = Self(1 << 23);
    /// R — IRA contribution for 2025 or earlier recharacterized in 2026
    #[libertas_form_code("R")]
    pub const CODE_R: Self = Self(1 << 24);
    /// S — early distribution from a SIMPLE IRA in its first 2 years, no known exception
    #[libertas_form_code("S")]
    pub const CODE_S: Self = Self(1 << 25);
    /// T — Roth IRA distribution, exception applies
    #[libertas_form_code("T")]
    pub const CODE_T: Self = Self(1 << 26);
    /// U — dividends from an ESOP under section 404(k)
    #[libertas_form_code("U")]
    pub const CODE_U: Self = Self(1 << 27);
    /// W — charges for qualified long-term care insurance under combined arrangements
    #[libertas_form_code("W")]
    pub const CODE_W: Self = Self(1 << 28);
    /// Y — qualified charitable distribution (QCD)
    /// Printed with code 4, 7 or K.
    #[libertas_form_code("Y")]
    pub const CODE_Y: Self = Self(1 << 29);
}
/// What the box 7a codes mean for you
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum DistributionCode {
    /// 7 — normal distribution
    Normal {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: Disposition,
    },
    /// 1 or 2 — early distribution, before age 59½
    Early {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: Disposition,
        /// Exception to the 10% additional tax
        #[libertas_default(None)]
        exception: EarlyException,
    },
    /// 3 — disability
    Disability {
        /// Date you became disabled
        #[libertas_date_only]
        began: u32,
        /// Minimum retirement age under this plan
        /// The earliest age you could have received this pension if you were not disabled; the plan administrator can tell you. Payments before that age are reported as wages.
        wages: crate::DisabilityWages,
    },
    /// 4 — death: a survivor's pension or annuity
    Death {
        /// Date of death
        #[libertas_date_only]
        died: u32,
    },
    /// G — direct rollover
    DirectRollover {
        /// Rolled over to
        to: RolloverTarget,
    },
    /// S — SIMPLE IRA in its first two years
    EarlySimple {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: crate::SimpleDisposition,
    },
    /// D — annuity payments from a nonqualified annuity
    /// Printed alone. With 1, 2, 3, 4 or 7, the numeric code's choice covers it.
    Nonqualified,
    /// Q, J or T — Roth IRA, Roth SEP IRA or Roth SIMPLE IRA
    /// J printed with S (box 7b) is a Roth SIMPLE IRA inside its first two years: the 25% rate applies.
    RothIra {
        /// Code
        #[libertas_default(Qualified)]
        roth: crate::RothIraCode,
        /// Amount rolled over to another Roth IRA within 60 days
        /// Only to a Roth IRA, deposited by the 60th day; one IRA-to-IRA rollover is tax-free in any 12 months. Not a conversion or a recharacterization.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        rolled_over: i64,
        /// Actual order among this Roth pool's payments on the same date
        /// Needed when this source or another same-day source funded an HSA. Use the actual order shown by account records, not this App's entry order. Ordinary same-day withdrawals without funding need no sequence.
        /// [NullText]
        /// No same-day HSA-funding order is needed for this Roth payment
        /// [SetText]
        /// This payment's actual same-day order is
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        sequence: Option<u32>,
    },
    /// Another code, or two codes
    /// Not calculated yet; the estimate names the box 7a codes.
    Other,
    /// 7 and A — normal distribution; the 10-year tax option may apply
    /// Code A: the plan participant was born before January 2, 1936.
    NormalLumpSum {
        /// How to tax it
        tax: LumpSumTax,
    },
    /// 4 and A — death; the 10-year tax option may apply
    /// Code A: the plan participant who died was born before January 2, 1936.
    DeathLumpSum {
        /// Date of death
        #[libertas_date_only]
        died: u32,
        /// How to tax it
        tax: InheritedLumpSumTax,
    },
    /// 6 or W — exchange or long-term care charges: not taxed
    /// 6: an annuity or insurance contract exchanged directly for another; a taxable part comes on its own 1099-R with another code. W: long-term care insurance charges paid from an annuity's or a life insurance contract's value.
    Exchange,
    /// 4 — inherited IRA
    /// Box 7b is checked: an IRA you inherited as a beneficiary.
    Inherited {
        /// Inherited IRA
        /// The person you inherited it from. The null answer uses your only inherited IRA; one entered on this same page is chosen after you save it.
        /// [NullText]
        /// Use my only saved inherited IRA
        /// [SetText]
        /// This payment came from the inherited IRA of
        #[libertas_default($null)]
        account: Option<i64>,
    },
    /// 4 — Trump account after the child's death
    /// Box 7c is checked. Box 2a is the account's value at the death less its basis.
    TrumpDeath {
        /// Paid to the child's estate
        /// Yes only on the child's own final return. Anyone else who received the account enters it on their own return.
        #[libertas_default(false)]
        estate: bool,

        /// Actual account basis from the trustee's records
        /// [NullText]
        /// Trustee basis not known
        /// [SetText]
        /// Actual trustee basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        basis: Option<i64>,
    },
    /// G — the whole Trump account moved to your ABLE account
    /// Box 7c is checked. Not taxed, and not counted toward your ABLE contributions.
    TrumpToAble {
        /// Date of the transfer
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 20260704, max = 20261231)]
        transferred: u32,
    },
    /// 8 — excess contributions or deferrals returned, taxable in 2026
    /// No 10% additional tax, and it cannot be rolled over. From an employer plan the taxable amount is wages on Form 1040 line 1h; from an IRA, line 4b.
    Corrective {
        /// Part of box 2a that is a 2026 excess deferral
        /// Only from a 401(k), 403(b) or salary-reduction SEP plan when your 2026 elective deferrals passed the limit. The plan's corrective-distribution letter splits the excess deferral from its earnings and from excess contributions. The App already counts the excess deferral as wages from your W-2s, so this part is not added again. Leave 0 for an IRA, a designated Roth account or a returned excess contribution.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        excess_deferral: i64,
    },
    /// N, P or R — not 2026 income
    /// N: a 2026 IRA contribution moved to the other kind of IRA, shown on Form 1040 line 4a only. P: a return taxable for 2025 or earlier; report it on that year's return. R: a 2025 contribution recharacterized in 2026, reported on the 2025 return. Box 4 withholding still counts for 2026.
    NotIncome,
    /// 9, E, F or U — taxable as printed, no additional tax
    /// 9: the cost of life insurance in your plan. E: a correction of excess annual additions. F: payments from a charitable gift annuity. U: ESOP dividends. None of these can be rolled over.
    NoAdditionalTax,
    /// 5 — prohibited transaction
    /// The whole IRA is treated as paid to you on January 1, 2026. The App adds the 10% additional tax when you were under 59½ that day.
    ProhibitedTransaction {
        /// Exception to the 10% additional tax
        #[libertas_default(None)]
        exception: EarlyException,
    },
    /// W — qualified long-term-care cash-value charge, not taxed
    /// May be printed with 6 for a section 1035 exchange. Box 8a is the actual investment reduction. Neither this charge nor exchange is an additional cash medical or business premium. Use insurer-adjusted basis for another payment; do not deduct this reduction again. A post-annuitization exclusion-ratio adjustment needs actual insurer tax results or its separate verified worksheet.
    QualifiedCareCharge,
    /// 2 or 7 with box 7b — employer contribution to my Roth SEP or Roth SIMPLE IRA
    /// The Roth IRA custodian reports your employer's matching or nonelective Roth contribution as if it were converted: boxes 1 and 2a are the contribution, taxed in full on Form 1040 line 4b, with no additional tax. It is not a withdrawal: nothing goes on Form 8606 Part I, and it counts as a 2026 conversion in your Roth IRA history.
    EmployerRothContribution,
}
/// What the distribution codes mean for you
/// The treatment and any account selected from the current return.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum RetirementTreatmentView {
    /// 7 — normal distribution
    Normal {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: Disposition,
    },
    /// 1 or 2 — early distribution, before age 59½
    Early {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: Disposition,
        /// Exception to the 10% additional tax
        #[libertas_default(None)]
        exception: EarlyException,
    },
    /// 3 — disability
    Disability {
        /// Date you became disabled
        #[libertas_date_only]
        began: u32,
        /// Minimum retirement age under this plan
        /// The earliest age you could have received this pension if you were not disabled; the plan administrator can tell you. Payments before that age are reported as wages.
        wages: crate::DisabilityWages,
    },
    /// 4 — death: a survivor's pension or annuity
    Death {
        /// Date of death
        #[libertas_date_only]
        died: u32,
    },
    /// G — direct rollover
    DirectRollover {
        /// Rolled over to
        to: RolloverTarget,
    },
    /// S — SIMPLE IRA in its first two years
    EarlySimple {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: crate::SimpleDisposition,
    },
    /// D — annuity payments from a nonqualified annuity
    /// Printed alone. With 1, 2, 3, 4 or 7, the numeric code's choice covers it.
    Nonqualified,
    /// Q, J or T — Roth IRA, Roth SEP IRA or Roth SIMPLE IRA
    /// J printed with S (box 7b) is a Roth SIMPLE IRA inside its first two years: the 25% rate applies.
    RothIra {
        /// Code
        #[libertas_default(Qualified)]
        roth: crate::RothIraCodeView,
        /// Amount rolled over to another Roth IRA within 60 days
        /// Only to a Roth IRA, deposited by the 60th day; one IRA-to-IRA rollover is tax-free in any 12 months. Not a conversion or a recharacterization.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        rolled_over: i64,
        /// Actual order among this Roth pool's payments on the same date
        /// Needed when this source or another same-day source funded an HSA. Use the actual order shown by account records, not this App's entry order. Ordinary same-day withdrawals without funding need no sequence.
        /// [NullText]
        /// No same-day HSA-funding order is needed for this Roth payment
        /// [SetText]
        /// This payment's actual same-day order is
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        sequence: Option<u32>,
    },
    /// Another code, or two codes
    /// Not calculated yet; the estimate names the box 7a codes.
    Other,
    /// 7 and A — normal distribution; the 10-year tax option may apply
    /// Code A: the plan participant was born before January 2, 1936.
    NormalLumpSum {
        /// How to tax it
        tax: LumpSumTax,
    },
    /// 4 and A — death; the 10-year tax option may apply
    /// Code A: the plan participant who died was born before January 2, 1936.
    DeathLumpSum {
        /// Date of death
        #[libertas_date_only]
        died: u32,
        /// How to tax it
        tax: InheritedLumpSumTax,
    },
    /// 6 or W — exchange or long-term care charges: not taxed
    /// 6: an annuity or insurance contract exchanged directly for another; a taxable part comes on its own 1099-R with another code. W: long-term care insurance charges paid from an annuity's or a life insurance contract's value.
    Exchange,
    /// 4 — inherited IRA
    /// Box 7b is checked: an IRA you inherited as a beneficiary.
    Inherited {
        /// Inherited IRA
        /// The person you inherited it from. The null answer uses your only inherited IRA; one entered on this same page is chosen after you save it.
        /// [NullText]
        /// Use my only saved inherited IRA
        /// [SetText]
        /// This payment came from the inherited IRA of
        #[libertas_enum_source("$.inherited_iras")]
        #[libertas_default($null)]
        account: Option<i64>,
    },
    /// 4 — Trump account after the child's death
    /// Box 7c is checked. Box 2a is the account's value at the death less its basis.
    TrumpDeath {
        /// Paid to the child's estate
        /// Yes only on the child's own final return. Anyone else who received the account enters it on their own return.
        #[libertas_default(false)]
        estate: bool,

        /// Actual account basis from the trustee's records
        /// [NullText]
        /// Trustee basis not known
        /// [SetText]
        /// Actual trustee basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        basis: Option<i64>,
    },
    /// G — the whole Trump account moved to your ABLE account
    /// Box 7c is checked. Not taxed, and not counted toward your ABLE contributions.
    TrumpToAble {
        /// Date of the transfer
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 20260704, max = 20261231)]
        transferred: u32,
    },
    /// 8 — excess contributions or deferrals returned, taxable in 2026
    /// No 10% additional tax, and it cannot be rolled over. From an employer plan the taxable amount is wages on Form 1040 line 1h; from an IRA, line 4b.
    Corrective {
        /// Part of box 2a that is a 2026 excess deferral
        /// Only from a 401(k), 403(b) or salary-reduction SEP plan when your 2026 elective deferrals passed the limit. The plan's corrective-distribution letter splits the excess deferral from its earnings and from excess contributions. The App already counts the excess deferral as wages from your W-2s, so this part is not added again. Leave 0 for an IRA, a designated Roth account or a returned excess contribution.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        excess_deferral: i64,
    },
    /// N, P or R — not 2026 income
    /// N: a 2026 IRA contribution moved to the other kind of IRA, shown on Form 1040 line 4a only. P: a return taxable for 2025 or earlier; report it on that year's return. R: a 2025 contribution recharacterized in 2026, reported on the 2025 return. Box 4 withholding still counts for 2026.
    NotIncome,
    /// 9, E, F or U — taxable as printed, no additional tax
    /// 9: the cost of life insurance in your plan. E: a correction of excess annual additions. F: payments from a charitable gift annuity. U: ESOP dividends. None of these can be rolled over.
    NoAdditionalTax,
    /// 5 — prohibited transaction
    /// The whole IRA is treated as paid to you on January 1, 2026. The App adds the 10% additional tax when you were under 59½ that day.
    ProhibitedTransaction {
        /// Exception to the 10% additional tax
        #[libertas_default(None)]
        exception: EarlyException,
    },
    /// W — qualified long-term-care cash-value charge, not taxed
    /// May be printed with 6 for a section 1035 exchange. Box 8a is the actual investment reduction. Neither this charge nor exchange is an additional cash medical or business premium. Use insurer-adjusted basis for another payment; do not deduct this reduction again. A post-annuitization exclusion-ratio adjustment needs actual insurer tax results or its separate verified worksheet.
    QualifiedCareCharge,
    /// 2 or 7 with box 7b — employer contribution to my Roth SEP or Roth SIMPLE IRA
    /// The Roth IRA custodian reports your employer's matching or nonelective Roth contribution as if it were converted: boxes 1 and 2a are the contribution, taxed in full on Form 1040 line 4b, with no additional tax. It is not a withdrawal: nothing goes on Form 8606 Part I, and it counts as a 2026 conversion in your Roth IRA history.
    EmployerRothContribution,
}
// Stored treatment and editable treatment have the same variants and facts;
// only the editable inherited-account fields bind to the current return's catalog.
record_page_conversions!(DistributionCode, RetirementTreatmentView, [], |value, Target, Source| {
    match value {
        Source::Normal { what_happened } => Target::Normal { what_happened },
        Source::Early { what_happened, exception } => Target::Early { what_happened, exception },
        Source::Disability { began, wages } => Target::Disability { began, wages },
        Source::Death { died } => Target::Death { died },
        Source::DirectRollover { to } => Target::DirectRollover { to },
        Source::EarlySimple { what_happened } => Target::EarlySimple { what_happened },
        Source::Nonqualified => Target::Nonqualified,
        Source::RothIra { roth, rolled_over, sequence } => {
            Target::RothIra { roth: roth.into(), rolled_over, sequence }
        }
        Source::Other => Target::Other,
        Source::NormalLumpSum { tax } => Target::NormalLumpSum { tax },
        Source::DeathLumpSum { died, tax } => Target::DeathLumpSum { died, tax },
        Source::Exchange => Target::Exchange,
        Source::Inherited { account } => Target::Inherited { account },
        Source::TrumpDeath { estate, basis } => Target::TrumpDeath { estate, basis },
        Source::TrumpToAble { transferred } => Target::TrumpToAble { transferred },
        Source::Corrective { excess_deferral } => Target::Corrective { excess_deferral },
        Source::NotIncome => Target::NotIncome,
        Source::NoAdditionalTax => Target::NoAdditionalTax,
        Source::ProhibitedTransaction { exception } => Target::ProhibitedTransaction { exception },
        Source::QualifiedCareCharge => Target::QualifiedCareCharge,
        Source::EmployerRothContribution => Target::EmployerRothContribution,
    }
});

/// Minimum retirement age under this plan
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisabilityWages {
    /// Reached before 2026, or this is an IRA or a plan without disability retirement
    /// The payments are pension or IRA income.
    None,
    /// Not reached by the end of 2026
    /// All taxable payments this year are reported as wages (Form 1040 line 1h).
    All,
    /// Reached during 2026
    Part {
        /// Taxable payments received through the day you reached it
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
impl DisabilityWages {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Part { amount } => (0..=100_000_000_000).contains(amount),
            _ => true,
        }
    }
}

/// How to tax it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LumpSumTax {
    /// With your other income
    WithOtherIncome {
        /// What you did with it
        #[libertas_default(Kept)]
        what_happened: Disposition,
    },
    /// Separately on Form 4972
    /// For a plan participant born before January 2, 1936; the App checks your date of birth. The amount taxed this way is left out of your income on Form 1040 line 5b.
    Form4972 {
        /// Tax method
        method: crate::LumpSumMethod,
        /// Include the box 6 net unrealized appreciation in income now
        /// Otherwise it stays untaxed until you sell the employer shares.
        #[libertas_default(false)]
        include_nua: bool,
        /// Qualifying event
        event: crate::LumpSumEvent,
        /// Date of the first payment in this lump sum
        /// Every payment of this entire-balance distribution must follow the qualifying event and fall in 2026.
        #[libertas_date_only]
        first_payment: u32,
        /// Date of your latest earlier payment from these plans
        /// Choose “No earlier payment or plan-loan offset was made from these plans” only if there was none. Include every actual distribution and plan-loan offset from all of the employer's plans of the same kind, including amounts rolled over. A payment after the qualifying event in an earlier year prevents using that event for this lump sum; a later event can start a new eligibility period.
        /// [NullText]
        /// No earlier payment or plan-loan offset was made from these plans
        /// [SetText]
        /// The latest earlier payment or plan-loan offset was on
        #[libertas_date_only]
        last_prior_payment: Option<u32>,
        /// All of the employer's plans of the same kind were fully distributed within 2026
        /// Pension, profit-sharing or stock bonus plans. A single account being emptied is not enough when another plan of the same kind keeps a balance. No taxes this distribution with your other income, with a notice.
        entire_balance: bool,
        /// Form 4972 conditions met
        /// None of it was rolled over. You took part in the plan for at least 5 tax years before 2026. You have not used Form 4972 after 1986 for an earlier distribution from your own plans; an election for a distribution before 1987, made while you were under 59½, does not count. It is not from an IRA or a 403(b) plan, U.S. Retirement Plan Bonds, a corrective distribution, a 5% owner's penalized benefit or a CSRS or FERS alternative-annuity lump sum, and the plan received no rollover after 2001 from an IRA, a 403(b) or governmental 457(b) plan or for a surviving spouse, nor was an earlier distribution from it rolled over. The plan's distribution statement and your earlier returns show it. No taxes this distribution with your other income, with a notice.
        confirmed: bool,
    },
}
/// How to tax it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InheritedLumpSumTax {
    /// With your other income
    WithOtherIncome,
    /// Separately on Form 4972
    /// For a beneficiary of a plan participant born before January 2, 1936. The amount taxed this way is left out of your income on Form 1040 line 5b.
    Form4972 {
        /// Plan participant's date of birth
        #[libertas_date_only]
        born: u32,
        /// Your share of federal estate tax on this distribution
        /// From the estate's administrator. Zero if the estate paid none.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        estate_tax: i64,
        /// Your share of the death benefit exclusion
        /// Only when the participant died before August 21, 1996: up to $5,000 for all recipients together, shared in proportion to the distribution, less any part already used. Leave 0 otherwise.
        #[libertas_money("USD", 2)]
        // Tax structure: The pre-August 21, 1996 death-benefit exclusion is at most $5,000 across recipients.
        #[libertas_number(min = 0, max = $DEATH_BENEFIT_EXCLUSION_CENTS)]
        #[libertas_default(0)]
        death_benefit: i64,
        /// Tax method
        method: crate::LumpSumMethod,
        /// Include the box 6 net unrealized appreciation in income now
        /// Otherwise it stays untaxed until you sell the employer shares.
        #[libertas_default(false)]
        include_nua: bool,
        /// Date of the first payment in this lump sum
        /// Every payment of this entire-balance distribution must follow the participant's death and fall in 2026.
        #[libertas_date_only]
        first_payment: u32,
        /// Date of the latest earlier payment from these plans
        /// Choose “No earlier payment or plan-loan offset was made from these plans after death” only if there was none. Include every actual distribution and plan-loan offset from all of the employer's plans of the same kind to anyone after the death, including amounts rolled over. An earlier-year payment after the death means this is not a lump sum.
        /// [NullText]
        /// No earlier payment or plan-loan offset was made from these plans after death
        /// [SetText]
        /// The latest earlier payment or plan-loan offset after death was on
        #[libertas_date_only]
        last_prior_payment: Option<u32>,
        /// All of the employer's plans of the same kind were fully distributed within 2026
        /// Pension, profit-sharing or stock bonus plans. A single account being emptied is not enough when another plan of the same kind keeps a balance. No taxes this distribution with your other income, with a notice.
        entire_balance: bool,
        /// Form 4972 conditions met
        /// None of it was rolled over. You have not used Form 4972 after 1986 for an earlier distribution you received as this participant's beneficiary. It is not from an IRA or a 403(b) plan, U.S. Retirement Plan Bonds, a corrective distribution or a CSRS or FERS alternative-annuity lump sum, and the plan received no rollover after 2001 from an IRA, a 403(b) or governmental 457(b) plan or for a surviving spouse, nor was an earlier distribution from it rolled over. The plan's distribution statement and your earlier returns show it. No taxes this distribution with your other income, with a notice.
        confirmed: bool,
    },
}
/// What you did with it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Disposition {
    /// Kept it
    Kept,
    /// Rolled it over within 60 days
    /// A qualified plan loan offset (code M) can be rolled over until the due date of your 2026 return, including extensions. Only one IRA-to-IRA rollover is tax-free in any 12 months.
    RolledOver {
        /// Amount deposited
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
        /// Deposited into
        #[libertas_default(Ira)]
        into: crate::RolloverDestination,
    },
    /// Converted it to a Roth IRA
    Converted {
        /// Amount converted
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Sold the property and rolled over the proceeds
    /// Stock or other property, not your employer's, paid to you from an employer plan, sold and the proceeds deposited within 60 days.
    SoldAndRolledOver {
        /// Property sale
        data: crate::PropertyRollover,
        /// Proceeds deposited
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
        /// Deposited into
        #[libertas_default(Ira)]
        into: crate::RolloverDestination,
    },
}
/// What you did with it
/// For a SIMPLE IRA in its first two years.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SimpleDisposition {
    /// Kept it
    /// Generally taxed at 25% in the first two years of the SIMPLE IRA unless an exception applies. A deposit into a traditional IRA or a Roth IRA in those two years is not a rollover or conversion: it stays taxed here and is a regular contribution there.
    Kept {
        /// Exception to the 25% additional tax
        #[libertas_default(None)]
        exception: EarlyException,
    },
    /// Rolled it over into another SIMPLE IRA
    /// Within 60 days. The only tax-free rollover in the first two years.
    RolledOver {
        /// Amount deposited into the other SIMPLE IRA
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
}
impl SimpleDisposition {
    // The stored treatment of the answer: a kept payment is the 25% early distribution.
    pub(crate) fn treatment(&self) -> crate::RetirementTreatment {
        match self {
            Self::Kept { .. } => crate::RetirementTreatment::EarlySimple,
            Self::RolledOver { amount } => crate::RetirementTreatment::Rollover {
                destination: crate::RolloverDestination::SimpleIra,
                amount: *amount,
                early: true,
            },
        }
    }
}
/// Exception to the 10% additional tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EarlyException {
    /// None
    None,
    /// An exception applies
    Applies {
        /// Date of this distribution
        #[libertas_date_only]
        distributed: u32,
        /// Exception
        /// Form 5329 exception 14 (a separation by March 1, 1986 with the whole interest paid under a written election) is not supported.
        facts: crate::RetirementException,
        /// A second exception on the same distribution
        /// Only when one exception does not cover the whole taxable amount, such as health insurance premiums while unemployed beside medical expenses above 7.5% of AGI. Form 5329 line 2 then shows 99. Choose “Do not claim a second early-distribution exception” otherwise.
        /// [NullText]
        /// Do not claim a second early-distribution exception
        /// [SetText]
        /// The second early-distribution exception claimed is
        #[libertas_default($null)]
        second: Option<crate::RetirementException>,
        /// Repaid to an IRA
        /// Only for a birth or adoption, emergency, domestic-abuse or terminal-illness distribution from an IRA, repaid within three years. Choose “No qualifying IRA repayment is claimed” when none. Deposits after 2026 may need an amended 2026 return.
        /// [NullText]
        /// No qualifying IRA repayment is claimed
        /// [SetText]
        /// The qualifying IRA repayment is
        #[libertas_default($null)]
        repaid: Option<ExceptionRepayment>,
    },
}
/// Repaid to an IRA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ExceptionRepayment {
    /// Deposits accepted as repayment
    /// Include later-year deposits only after they have actually occurred. Never enter planned deposits.
    #[libertas_size(min = 1)]
    /// ----
    /// Deposit
    pub payments: Vec<crate::IraRepaymentDeposit>,
    /// The receiving IRA trustee accepted these as repayments of this distribution
    /// Check the trustee’s acceptance notice and original distribution records.
    /// No refunded or duplicated repayment, regular contribution, Roth conversion, disaster distribution or other repayment treatment. Include 2026 deposits in the actual December 31 value; do not adjust that value yourself. No credits none of these deposits as repayments, so the original distribution stays taxable.
    pub confirmed: bool,
}
impl ExceptionRepayment {
    pub(crate) fn valid(&self) -> bool {
        !self.payments.is_empty()
            && self.payments.iter().all(|p| {
                crate::dates::source_valid_date(p.date)
                    && (2026..=2029).contains(&(p.date / 10000))
                    && p.amount >= 0
            })
    }
    pub(crate) fn amount_exact(&self) -> crate::kernel::ratio::exact::Exact {
        crate::ira_repayment_sources::deposit_amount(&self.payments)
    }
}
/// Rolled over to
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RolloverTarget {
    /// Traditional IRA
    Ira,
    /// Employer retirement plan
    EmployerPlan,
    /// Roth IRA or designated Roth account
    /// A direct rollover to a Roth IRA, or an in-plan Roth rollover to the Roth account of the same plan (code G with a taxable box 2a, or code H). Box 2a is taxed; the 10% additional tax does not apply to it.
    Roth,
}
/// Follow-up answers a Form 1099-R calls for
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RCalls {
    /// Disaster treatment
    pub disaster: Vec<i32>,
    /// Public safety officer insurance premiums
    pub public_safety: Vec<i32>,
    /// Disability retirement income
    pub disability: Vec<i32>,
    /// In-plan Roth rollover basis
    pub box_10_basis: Vec<i32>,
}

/// Insurance premiums paid from this pension
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PublicSafetyAnswer {
    /// Premiums
    Premiums {
        /// Coverage allocations
        /// ----
        /// Coverage
        rows: Vec<crate::PublicSafetyPaid>,
    },
}
impl PublicSafetyAnswer {
    pub(crate) fn of(rows: Vec<crate::PublicSafetyPaid>) -> Option<Self> {
        (!rows.is_empty()).then_some(Self::Premiums { rows })
    }
    pub(crate) fn rows(value: &Option<Self>) -> &[crate::PublicSafetyPaid] {
        value.as_ref().map_or(&[], |Self::Premiums { rows }| rows)
    }
}

/// Nontaxable part of an in-plan Roth rollover
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InPlanRothBasis {
    /// Known basis
    Known {
        /// Nontaxable part of box 10
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
impl InPlanRothBasis {
    pub(crate) fn amount(self) -> i64 {
        let Self::Known { amount } = self;
        amount
    }
}
/// Box 2b taxable amount not determined: how to figure it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxableMethod {
    /// Box 2a shows the taxable amount
    BoxTwoA,
    /// Simplified Method
    /// Annuity payments from an employer plan (Publication 575 Worksheet A).
    Simplified {
        /// Pension Simplified Method facts
        data: crate::PensionBasis,
    },
    /// General Rule, fixed payments
    /// A commercial annuity, or an employer-plan annuity that started before November 19, 1996 or at age 75 or older with at least 5 years of guaranteed payments (Publication 939).
    GeneralRule {
        /// General Rule facts
        data: crate::GeneralAnnuity,
    },
    /// General Rule, variable payments
    /// A commercial annuity whose payments follow its investments (Publication 939).
    VariablePayments {
        /// Variable annuity facts
        data: crate::VariableAnnuity,
    },
    /// The whole contract ended: surrendered, matured, refunded, paid at death or given away
    /// Taxable only above the investment not yet recovered.
    WholeContract {
        /// Contract facts
        data: crate::ContractDischarge,
    },
    /// Withdrawal from an employer plan before annuity payments began
    /// The tax-free part is your after-tax investment's share of the account (Publication 575).
    PlanWithdrawal {
        /// Withdrawal facts
        data: crate::PlanDistribution,
    },
    /// Withdrawal, loan or pledge from a commercial annuity before payments began
    /// Earnings come out first (Publication 575).
    AnnuityWithdrawal {
        /// Withdrawal facts
        data: crate::AnnuityDistribution,
    },
    /// Withdrawal from a commercial annuity with investment from before August 14, 1982
    /// That investment comes out first, tax-free (Publication 575).
    Pre1982Withdrawal {
        /// Contract allocation
        data: crate::LegacyAnnuity,
    },
    /// CSRS or FERS lump sum after a death
    /// The rest of the employee's contributions paid to you as beneficiary (Publication 721).
    CivilServiceLumpSum {
        /// Benefit facts
        data: crate::SurvivorLumpSum,
    },
    /// CSRS or FERS child survivor annuity
    /// A temporary annuity to a child, with no surviving-spouse annuity (Publication 721).
    ChildAnnuity {
        /// Temporary child annuity
        data: crate::ChildAnnuity,
    },
}

/// Last year's Form 8606
/// Only if you have made nondeductible traditional IRA contributions or repaid a reservist distribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form8606 {
    /// Line 14 basis in your traditional IRAs
    /// From the last Form 8606 you filed. For a form before 2001 use the Total Basis Chart in the instructions: line 12 of a 1993–2000 form, line 14 of a 1989–1992 form.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Adjustment to that basis
    /// Add line 6 of the Line 15c Worksheet from the last year you completed Part I, the basis received from your spouse in a divorce and any nontaxable plan rollover into a traditional IRA never reported on line 2. Subtract basis transferred in a divorce and an excess contribution that was counted and later returned. A divorce adjustment needs a statement with the return. Leave 0 otherwise.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub adjustment: i64,
    /// Value of all your traditional, SEP and SIMPLE IRAs on December 31, 2026
    /// Form 5498 box 5, added across accounts, taking any recharacterization into account. For a final return, the value on the date of death instead.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end: i64,
    /// Qualified reservist distributions repaid in 2026
    /// Each repayment adds nondeductible basis (Form 8606 line 1) without using the contribution limit. Include the deposits in the year-end value. Leave empty when none.
    /// ----
    /// Original distribution and its 2026 repayments
    pub reservist_repayments: Vec<crate::ReservistRepayment>,
}
impl Form8606 {
    // The client rules of the Form 8606, checked again on save.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if crate::IraBasis::opening_basis_exact(self.basis, self.adjustment)
            < crate::kernel::ratio::exact::whole(0)
        {
            return Some(
                crate::validation::ValidationIssue::new("FORM_8606_ADJUSTMENT_BELOW_BASIS")
                    .at_field("adjustment"),
            );
        }
        let repayments = &self.reservist_repayments;
        if let Some((index, _)) = repayments
            .iter()
            .enumerate()
            .find(|(i, r)| repayments[..*i].iter().any(|o| o.distributed == r.distributed))
        {
            return Some(
                crate::validation::ValidationIssue::new("RESERVIST_DISTRIBUTION_TWICE")
                    .at_field("distributed")
                    .in_list("reservist_repayments", index),
            );
        }
        repayments.iter().enumerate().find_map(|(index, r)| {
            r.issue().map(|issue| issue.in_list("reservist_repayments", index))
        })
    }
}
/// Gift from your IRA directly to a charity
/// Paid by the IRA trustee to a public charity after you reached age 70½; already included in a 1099-R above.
/// Use the trustee payment records and charity acknowledgment. No ongoing SEP/SIMPLE, donor-advised fund, supporting organization or benefit received. Personal after-tax basis uses the owner-level IRA worksheet; inherited basis belongs to the linked deceased-owner group. Do not enter the gift again in Deductions; its nonexcluded portion is included there automatically.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QcdGift {
    /// Date paid
    #[libertas_date_only]
    pub date: u32,
    /// Amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Paid from
    /// The null answer uses your own IRAs. Choose the inherited IRA this gift came from; one entered on this same page is chosen after you save it.
    /// [NullText]
    /// Paid from my own IRAs
    /// [SetText]
    /// Paid from the inherited IRA of
    #[libertas_enum_source("$.inherited_iras")]
    pub inherited: Option<i64>,
    /// Paid to a split-interest entity under your one-time election
    /// A charitable remainder annuity trust, charitable remainder unitrust or charitable gift annuity funded only by these gifts, paying only you, your spouse or both, and a gift annuity paying a fixed 5% or more starting within a year. Up to $55,000 in all, in one year of your life, within the $111,000 limit. Payments you later receive from it are fully taxable.
    #[libertas_default(false)]
    pub split_interest: bool,
}
