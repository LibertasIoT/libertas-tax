// form_1099_oid items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub use crate::oid_source::{
    FederalOid, Form1099OidPaper, Form1099OidSource, OidCalls, OidNomineeAnswer, OidPrintedFacts,
    OidSpecial,
};

/// 1099-OID
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Oid1099 {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    /// [NullText]
    /// All reported OID and interest components belong to this return
    /// [SetText]
    /// These OID and interest components belong to someone outside this return
    #[libertas_constrained_by("^<Oid1099>.more.calls.nominee")]
    #[libertas_hidden]
    pub nominee: Option<OidNomineeAnswer>,
    /// Personal annual reporting adjustments
    /// [NullText]
    /// Use the reported OID and interest without personal annual adjustments
    /// [SetText]
    /// My personal annual OID and interest adjustments are
    #[libertas_default($null)]
    #[libertas_hidden]
    pub adjustments: Option<crate::InterestAdjustments>,

    /// Box 1 original issue discount for 2026
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub oid: i64,
    /// Box 2 other periodic interest
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Other boxes
    #[libertas_llm_doc_field]
    pub more: OidMore,
    /// State boxes 12–14 and supplemental information
    #[libertas_llm_doc_field]
    pub state_details: crate::InvestmentStateFields,
}

impl From<&Form1099OidSource> for OidBoxes {
    fn from(value: &Form1099OidSource) -> Self {
        Self {
            adjustments: value.adjustments.clone(),
            oid: value.form.original_issue_discount_for_the_year,
            other_periodic_interest: value.form.other_periodic_interest,
            more: OidCallBoxes {
                market_discount: value.form.market_discount,
                acquisition_premium: value.form.acquisition_premium,
                treasury_oid: value.form.original_issue_discount_on_us_treasury_obligations,
                bond_premium: value.form.bond_premium,
                exempt: value.form.tax_exempt_oid,
                tax_exempt_bond_premium: value.tax_exempt_bond_premium,
                private_activity: value.private_activity,
            },
        }
    }
}

/// Other boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidMore {
    /// Applicable personal follow-up choices
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: OidCalls,
    /// Premium reducing exempt OID, from the identified annual bond worksheet
    /// Separate from taxable box 10 premium; never apply it to taxable OID.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub tax_exempt_bond_premium: i64,

    /// Box 9 investment expenses, accepted without a deduction
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_expenses: i64,
    /// Box 3 early withdrawal penalty
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub early_penalty: i64,
    /// Box 4 federal income tax withheld
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 5 market discount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Box 6 acquisition premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Box 8 original issue discount on U.S. Treasury obligations
    /// May be negative, such as a TIPS deflation adjustment.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Box 10 bond premium
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Box 11 tax-exempt original issue discount
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private activity bond discount included in box 11
    /// From the payer's supplemental statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_hidden]
    pub private_activity: i64,
    /// A treatment this page does not calculate
    /// A box 7 description that is a stripped or contingent-payment obligation, or another unentered OID treatment. Nominee components have their own personal record below. Any choice but None of these leaves this statement out of the estimate with a notice.
    #[libertas_default(NoneOfThese)]
    #[libertas_hidden]
    pub special: OidSpecial,
}

/// Reported components used to select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidCallBoxes {
    /// Reported market discount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Reported acquisition premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Reported treasury oid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Reported bond premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Reported exempt
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Reported tax exempt bond premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Reported private activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
}

/// Form 1099-OID boxes that select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidBoxes {
    /// Reported oid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub oid: i64,
    /// Box 2 other periodic interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Other reported components
    pub more: OidCallBoxes,
    /// Personal adjustments may replace reported income
    /// [NullText]
    /// Use the reported OID and interest without personal annual adjustments
    /// [SetText]
    /// My personal annual OID and interest adjustments are
    #[libertas_default($null)]
    pub adjustments: Option<crate::InterestAdjustments>,
}

impl From<&Oid1099> for OidBoxes {
    fn from(value: &Oid1099) -> Self {
        Self {
            adjustments: value.adjustments.clone(),
            oid: value.oid,
            other_periodic_interest: value.other_periodic_interest,
            more: OidCallBoxes {
                market_discount: value.more.market_discount,
                acquisition_premium: value.more.acquisition_premium,
                treasury_oid: value.more.treasury_oid,
                bond_premium: value.more.bond_premium,
                exempt: value.more.exempt,
                tax_exempt_bond_premium: value.more.tax_exempt_bond_premium,
                private_activity: value.more.private_activity,
            },
        }
    }
}

impl From<&Oid1099> for FederalOid {
    fn from(value: &Oid1099) -> Self {
        let Oid1099 {
            payer: _,
            recipient: _,
            nominee,
            adjustments,
            oid,
            other_periodic_interest,
            more,
            state_details: _,
        } = value;
        let OidMore {
            calls: _,
            tax_exempt_bond_premium,
            investment_expenses,
            early_penalty,
            withholding,
            market_discount,
            acquisition_premium,
            treasury_oid,
            bond_premium,
            exempt,
            private_activity,
            special,
        } = more;
        Self {
            tax_exempt_bond_premium: *tax_exempt_bond_premium,
            nominee: nominee.as_ref().map(|answer| answer.record().clone()),
            adjustments: adjustments.clone(),
            investment_expenses: *investment_expenses,
            oid: *oid,
            other_periodic_interest: *other_periodic_interest,
            early_penalty: *early_penalty,
            withholding: *withholding,
            market_discount: *market_discount,
            acquisition_premium: *acquisition_premium,
            exempt: *exempt,
            bond_premium: *bond_premium,
            private_activity: *private_activity,
            special: *special,
            treasury_oid: *treasury_oid,
            printed_facts: OidPrintedFacts::default(),
        }
    }
}
