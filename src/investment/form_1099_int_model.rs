#[allow(unused_imports)]
use super::*;

pub use crate::interest_source::{
    FederalInterest, Form1099IntPaper, Form1099IntSource, IntAfterDeathAnswer, IntCalls,
    IntNomineeAnswer, InterestPrintedFacts, InterestSource, InterestSpecial,
};

impl From<&Int1099> for FederalInterest {
    fn from(form: &Int1099) -> Self {
        let Int1099 {
            payer: _,
            recipient: _,
            nominee,
            adjustments,
            taxable,
            more,
            after_death,
            state_details: _,
        } = form;
        let IntMore {
            calls: _,
            investment_expenses,
            market_discount,
            foreign_country,
            early_penalty,
            us_obligations,
            withholding,
            foreign_tax,
            exempt,
            private_activity,
            bond_premium,
            treasury_bond_premium,
            tax_exempt_bond_premium,
            accrued_interest,
            special,
        } = more;
        Self {
            private_loan: false,
            seller_mortgage: None,
            nominee: nominee.as_ref().map(|answer| answer.record().clone()),
            adjustments: adjustments.clone(),
            investment_expenses: *investment_expenses,
            market_discount: *market_discount,
            foreign_country: foreign_country.clone(),
            taxable: *taxable,
            us_obligations: *us_obligations,
            early_penalty: *early_penalty,
            exempt: *exempt,
            private_activity: *private_activity,
            withholding: *withholding,
            special: *special,
            bond_premium: *bond_premium,
            treasury_bond_premium: *treasury_bond_premium,
            tax_exempt_bond_premium: *tax_exempt_bond_premium,
            accrued_interest: *accrued_interest,
            foreign_tax: *foreign_tax,
            after_death: after_death.as_ref().map(|answer| answer.record().clone()),
            printed_facts: InterestPrintedFacts::default(),
        }
    }
}

/// Interest dividend document
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
pub enum InterestDividendDocument {
    /// 1099-INT
    Int1099(Form1099IntPaper),
    /// 1099-DIV
    Div1099(Form1099DivPaper),
    /// 1099-OID
    Oid1099(Form1099OidPaper),
    /// Form 2439
    /// Undistributed long-term capital gains a mutual fund or REIT kept and paid tax on for you. One for each Form 2439.
    Form2439(Form2439Paper),
    /// Savings bonds cashed for education (Form 8815)
    /// Series EE or I bonds issued after 1989 and cashed in 2026, whose interest is in box 3 of a 1099-INT entered on this page. Enter it once for the whole return.
    SavingsBonds(crate::SavingsBondsEntry),
    /// Form 8818 — savings bond redemption record
    Form8818(crate::Form8818Paper),
}

/// 1099-INT
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Int1099 {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    /// [NullText]
    /// All reported interest components belong to this return
    /// [SetText]
    /// These interest components belong to someone outside this return
    #[libertas_constrained_by("^<Int1099>.more.calls.nominee")]
    #[libertas_hidden]
    pub nominee: Option<IntNomineeAnswer>,
    /// Personal annual reporting adjustments
    /// [NullText]
    /// Use the reported interest without personal annual adjustments
    /// [SetText]
    /// My personal annual interest adjustments are
    #[libertas_default($null)]
    #[libertas_hidden]
    pub adjustments: Option<crate::InterestAdjustments>,

    /// Box 1 interest income
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// Other boxes
    #[libertas_llm_doc_field]
    pub more: IntMore,
    /// Paid after the death
    /// Only for a person who died in 2026 whose statement also includes interest for the time after the death: that part belongs to the estate or another recipient and is left off this final return (Pub. 559). Choose “No reported interest belongs to the period after death” otherwise. Your account statements show the dates.
    /// [NullText]
    /// No reported interest belongs to the period after death
    /// [SetText]
    /// The reported interest paid after death is
    #[libertas_constrained_by("^<Int1099>.more.calls.after_death")]
    #[libertas_hidden]
    pub after_death: Option<IntAfterDeathAnswer>,
    /// State boxes 15–17 and supplemental information
    #[libertas_llm_doc_field]
    pub state_details: crate::InvestmentStateFields,
}

/// Other boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IntMore {
    /// Applicable personal follow-up choices
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: IntCalls,
    /// Box 5 investment expenses; accepted without a deduction
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Box 10 current-election market discount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Box 7 foreign country or U.S. territory
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub foreign_country: String,
    /// Box 2 early withdrawal penalty
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_penalty: i64,
    /// Box 3 interest on U.S. savings bonds and Treasury obligations
    /// Savings bond interest you exclude for college costs is entered once more as Savings bonds cashed for education (Form 8815).
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 6 foreign tax paid
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
    /// Box 8 tax-exempt interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Box 9 specified private activity bond interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Box 11 bond premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Box 12 bond premium on Treasury obligations
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Box 13 bond premium on tax-exempt bonds
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Accrued interest you paid the seller when you bought a bond
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub accrued_interest: i64,
    /// A treatment this page does not calculate
    /// Treaty interest or another unentered interest treatment. Nominee components and previously reported savings-bond interest have their own personal records below. Any choice but None of these leaves this statement out of the estimate with a notice.
    #[libertas_default(NoneOfThese)]
    #[libertas_hidden]
    pub special: InterestSpecial,
}

/// Reported components used to select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IntCallBoxes {
    /// Reported us obligations
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Reported exempt
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Reported private activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Reported bond premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Reported treasury bond premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Reported tax exempt bond premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Reported accrued interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub accrued_interest: i64,
    /// Reported market discount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Reported foreign tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
}

/// Form 1099-INT boxes that select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IntBoxes {
    /// Reported taxable
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// Other reported components
    pub more: IntCallBoxes,
    /// Personal adjustments may replace reported income
    /// [NullText]
    /// Use the reported interest without personal annual adjustments
    /// [SetText]
    /// My personal annual interest adjustments are
    #[libertas_default($null)]
    pub adjustments: Option<crate::InterestAdjustments>,
}

impl From<&Int1099> for IntBoxes {
    fn from(value: &Int1099) -> Self {
        Self {
            adjustments: value.adjustments.clone(),
            taxable: value.taxable,
            more: IntCallBoxes {
                us_obligations: value.more.us_obligations,
                exempt: value.more.exempt,
                private_activity: value.more.private_activity,
                bond_premium: value.more.bond_premium,
                treasury_bond_premium: value.more.treasury_bond_premium,
                tax_exempt_bond_premium: value.more.tax_exempt_bond_premium,
                accrued_interest: value.more.accrued_interest,
                market_discount: value.more.market_discount,
                foreign_tax: value.more.foreign_tax,
            },
        }
    }
}

impl From<&Form1099IntSource> for IntBoxes {
    fn from(value: &Form1099IntSource) -> Self {
        Self {
            adjustments: value.adjustments.clone(),
            taxable: value.form.interest_income,
            more: IntCallBoxes {
                us_obligations: value.form.us_savings_bond_and_treasury_interest,
                exempt: value.form.tax_exempt_interest,
                private_activity: value.form.specified_private_activity_bond_interest,
                bond_premium: value.form.bond_premium,
                treasury_bond_premium: value.form.bond_premium_on_treasury_obligations,
                tax_exempt_bond_premium: value.form.bond_premium_on_tax_exempt_bond,
                accrued_interest: value.accrued_interest,
                market_discount: value.form.market_discount,
                foreign_tax: value.form.foreign_tax_paid,
            },
        }
    }
}
