use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// State or U.S. territory
/// Choose the jurisdiction printed on the form, not your home state or the payer's address.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum TaxState {
    /// Alabama (AL)
    AL,
    /// Alaska (AK)
    AK,
    /// Arizona (AZ)
    AZ,
    /// Arkansas (AR)
    AR,
    /// California (CA)
    CA,
    /// Colorado (CO)
    CO,
    /// Connecticut (CT)
    CT,
    /// Delaware (DE)
    DE,
    /// District of Columbia (DC)
    DC,
    /// Florida (FL)
    FL,
    /// Georgia (GA)
    GA,
    /// Hawaii (HI)
    HI,
    /// Idaho (ID)
    ID,
    /// Illinois (IL)
    IL,
    /// Indiana (IN)
    IN,
    /// Iowa (IA)
    IA,
    /// Kansas (KS)
    KS,
    /// Kentucky (KY)
    KY,
    /// Louisiana (LA)
    LA,
    /// Maine (ME)
    ME,
    /// Maryland (MD)
    MD,
    /// Massachusetts (MA)
    MA,
    /// Michigan (MI)
    MI,
    /// Minnesota (MN)
    MN,
    /// Mississippi (MS)
    MS,
    /// Missouri (MO)
    MO,
    /// Montana (MT)
    MT,
    /// Nebraska (NE)
    NE,
    /// Nevada (NV)
    NV,
    /// New Hampshire (NH)
    NH,
    /// New Jersey (NJ)
    NJ,
    /// New Mexico (NM)
    NM,
    /// New York (NY)
    NY,
    /// North Carolina (NC)
    NC,
    /// North Dakota (ND)
    ND,
    /// Ohio (OH)
    OH,
    /// Oklahoma (OK)
    OK,
    /// Oregon (OR)
    OR,
    /// Pennsylvania (PA)
    PA,
    /// Rhode Island (RI)
    RI,
    /// South Carolina (SC)
    SC,
    /// South Dakota (SD)
    SD,
    /// Tennessee (TN)
    TN,
    /// Texas (TX)
    TX,
    /// Utah (UT)
    UT,
    /// Vermont (VT)
    VT,
    /// Virginia (VA)
    VA,
    /// Washington (WA)
    WA,
    /// West Virginia (WV)
    WV,
    /// Wisconsin (WI)
    WI,
    /// Wyoming (WY)
    WY,
    /// American Samoa (AS)
    AS,
    /// Guam (GU)
    GU,
    /// Northern Mariana Islands (MP)
    MP,
    /// Puerto Rico (PR)
    PR,
    /// U.S. Virgin Islands (VI)
    VI,
}

/// Party named on a printed document
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PrintedParty {
    /// Name as printed
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub name: String,
    /// Identification number as printed
    /// Copy the printed number, including leading zeros, hyphens and any masking. Leave it empty if absent.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub federal_id: String,
    /// Address as printed
    /// Copy each printed address row intact. Add nothing the paper does not print: no country when none is printed, so a row printed "VA 00000" stays "VA 00000". Do not repeat the name above as the first address row.
    /// Do not insert punctuation, rearrange components or impose a postal format.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    #[libertas_multiline]
    pub address: String,
}

/// Employer or payer identity used by accepted source records.
pub type TaxPayerIdentity = PrintedParty;

/// A printed party block that has no identification-number field.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PrintedPartyNameAddress {
    /// Name as printed
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub name: String,
    /// Address as printed
    /// Original line breaks and punctuation are part of the address. Add nothing the paper does not print, such as a country, and do not repeat the name above as the first address row.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    #[libertas_multiline]
    pub address: String,
}

/// State withholding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StateTaxWithholding {
    /// State
    #[libertas_llm_doc_field]
    pub state: TaxState,
    /// Payer's state identification number
    /// Copy only the identification number, exactly; this is not the payer's federal EIN. When the state abbreviation is printed with it, the abbreviation belongs in State only: "VA 0000117" is state VA and number 0000117. Leave it empty if absent.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_state_id: String,
    /// State income tax withheld
    /// [NullText]
    /// The printed withholding box is blank
    /// [SetText]
    /// The printed state income tax withheld is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
}

/// State income and withholding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StateIncomeTax {
    /// State
    #[libertas_llm_doc_field]
    pub state: TaxState,
    /// Employer's or payer's state identification number
    /// Preserve leading zeros and punctuation. Copy only the identification number: a state abbreviation printed with it belongs in State only ("VA 0000117" is state VA and number 0000117). Leave it empty if absent.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_state_id: String,
    /// State wages, winnings, distribution or income
    /// Use the state amount printed on this row. It can differ from federal income. Choose the unprinted answer when the box is blank; a printed zero is zero.
    /// [NullText]
    /// No state income amount printed on this row
    /// [SetText]
    /// The printed state income amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub income: Option<i64>,
    /// State income tax withheld
    /// [NullText]
    /// The printed withholding box is blank
    /// [SetText]
    /// The printed state income tax withheld is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
}

/// Local income and withholding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LocalIncomeTax {
    /// Locality name
    /// Copy the city, county, school district or other locality exactly as printed.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub locality: String,
    /// State of this locality
    /// Choose the unidentified answer unless the document identifies it. Do not guess from the payer's address.
    /// [NullText]
    /// The document does not identify the locality's state
    /// [SetText]
    /// The locality's identified state is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub state: Option<TaxState>,
    /// Local wages, winnings or distribution
    /// Choose the unprinted answer when blank; preserve a printed zero.
    /// [NullText]
    /// No local income amount printed on this row
    /// [SetText]
    /// The printed local income amount is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub income: Option<i64>,
    /// Local income tax withheld
    /// [NullText]
    /// The printed withholding box is blank
    /// [SetText]
    /// The printed local income tax withheld is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
}

/// State withholding rows
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct StateWithholdingFields {
    /// State withholding rows
    /// Enter each printed row once. No rows means the form has no state information.
    #[libertas_llm_doc_field]
    /// ----
    /// State withholding row
    pub states: Vec<StateTaxWithholding>,
}

/// State income and withholding rows
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct StateIncomeFields {
    /// State income and withholding rows
    #[libertas_llm_doc_field]
    /// ----
    /// State income row
    pub states: Vec<StateIncomeTax>,
}

/// State and local income rows
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct StateLocalFields {
    /// State income and withholding rows
    #[libertas_llm_doc_field]
    /// ----
    /// State income row
    pub states: Vec<StateIncomeTax>,
    /// Local income and withholding rows
    #[libertas_llm_doc_field]
    /// ----
    /// Local income row
    pub localities: Vec<LocalIncomeTax>,
}

/// Tax-exempt income by issuing state
/// Use the issuer allocation on the supplemental statement, not the payer's state withholding boxes. Do not guess an unallocated portion.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxExemptStateIncome {
    /// Issuing state or territory
    #[libertas_llm_doc_field]
    pub state: TaxState,
    /// Allocation as printed
    #[libertas_llm_doc_field]
    pub allocation: TaxExemptAllocation,
}

/// Issuing-state allocation
/// Use the statement's dollar amount or percentage without converting between them.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxExemptAllocation {
    /// Dollar amount
    Amount {
        /// Amount
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Percentage
    Percentage {
        /// Share
        /// Enter the statement's value from 0 to 100, such as 27.43. A value is required for this allocation; zero means the statement explicitly reports zero.
        #[libertas_llm_doc_field(allow_empty = false)]
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 0, max = 100)]
        #[libertas_physical_unit("percent")]
        value: f64,
    },
}

/// State information and supplemental allocations
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InvestmentStateFields {
    /// State withholding rows
    /// These do not identify the issuers of tax-exempt bonds.
    #[libertas_llm_doc_field]
    /// ----
    /// State withholding row
    pub states: Vec<StateTaxWithholding>,
    /// Tax-exempt income by issuing state
    /// Copy dollar amounts or percentages identified by the supplemental statement. Leave this list empty when no allocation is supplied.
    #[libertas_llm_doc_field]
    /// ----
    /// Issuing state allocation
    pub exempt_sources: Vec<TaxExemptStateIncome>,
    /// Fund's U.S. government obligation share
    /// Enter the supplemental statement's value from 0 to 100, such as 27.43. Choose the unsupplied answer if absent; zero means the statement explicitly reports zero.
    /// [NullText]
    /// No government-obligation share supplied
    /// [SetText]
    /// The supplied government-obligation share is
    #[libertas_llm_doc_field]
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub government_obligation_percentage: Option<f64>,
}

/// Box 14a item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2OtherItem {
    /// Label as printed
    /// For example CA SDI, NY PFL, 414(h), STPICKUP or MAPFML. Keep the employer's wording.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub label: String,
    /// Amount or other value
    #[libertas_llm_doc_field]
    pub value: W2OtherValue,
}

/// Value printed with the label
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum W2OtherValue {
    /// Dollar amount
    Money {
        /// Amount
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Other printed text
    Text {
        /// Text as printed
        #[libertas_llm_doc_field]
        #[libertas_size(min = 1)]
        text: String,
    },
}

/// Additional wage, state and local information
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct W2StateFields {
    /// Box d control number
    #[libertas_default("")]
    pub control_number: String,
    /// Box 11 nonqualified plans
    /// Already included in the printed federal wages; do not add it to wages again.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonqualified_plans: i64,
    /// Box 13 third-party sick pay
    #[libertas_default(false)]
    pub third_party_sick_pay: bool,
    /// Boxes 15–17 state wages and withholding
    /// ----
    /// State wage row
    pub states: Vec<StateIncomeTax>,
    /// Boxes 18–20 local wages and withholding
    /// ----
    /// Local wage row
    pub localities: Vec<LocalIncomeTax>,
    /// Box 14a labelled items
    /// ----
    /// Box 14a item
    pub box14a: Vec<W2OtherItem>,
    /// Box 14b Treasury tipped-occupation codes
    /// Copy each three-digit code, including 000. These records do not decide whether tips qualify for a deduction.
    /// ----
    /// Tipped-occupation code
    pub tipped_occupation_codes: Vec<TippedOccupationCode>,
}

/// Treasury tipped-occupation code
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TippedOccupationCode {
    /// Three-digit code
    /// Keep leading zeros; 000 is a valid printed code.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_format("000")]
    // Form W-2 tipped occupation codes use a three-digit code.
    #[libertas_number(min = 0, max = 999)]
    pub code: u16,
}

/// Retirement state information
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RetirementStateFields {
    /// State/local boxes 14–19
    #[libertas_llm_doc_field]
    pub taxes: StateLocalFields,
}

/// Social Security benefit statement
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum SocialSecurityStatement {
    /// Social Security Administration Form SSA-1099
    SocialSecurity,
    /// Railroad Retirement Board Form RRB-1099
    Railroad,
}

/// Refund payer and state information
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RefundStateFields {
    /// State that paid the refund
    /// Identify it from the refund notice. This can differ from your current home state.
    /// [NullText]
    /// The refund notice does not identify a state
    /// [SetText]
    /// The identified refund-paying state is
    #[libertas_default($null)]
    pub refund_state: Option<TaxState>,
    /// Locality that paid a local refund
    #[libertas_default("")]
    pub refund_locality: String,
    /// Boxes 11a, 11b and 12
    pub taxes: StateWithholdingFields,
}

/// Business or farm information form
/// These state records belong to this activity. Federal income and federal withholding are entered in the activity's existing totals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-NEC, 1099-MISC or 1099-K and check its tax year against the supplied year. Select the printed form type. Read the payer name, federal identification number and address exactly as printed. For 1099-NEC read state boxes 5 through 7; for 1099-MISC read state boxes 16 through 18; for 1099-K read state boxes 6 through 8. Return each populated state row once, using the state's two-letter code. Return an empty string for absent payer identity text fields and null for a blank state income box, null for a clearly blank withholding box, and an empty list if no state row is present. Keep IDs as text, including leading zeros and punctuation. Never infer state income from federal income or infer a state from the payer address. Report illegible or conflicting values as errors. Copies B and 2 of the same form are one document. Do not return federal income, federal withholding, tips eligibility or tax treatment.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct BusinessStateDocument {
    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: PrintedParty,
    /// Form and its state boxes
    #[libertas_llm_doc_field]
    pub form: BusinessStateForm,
}

/// Information form
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessStateForm {
    /// 1099-NEC — nonemployee compensation
    Nec {
        /// Boxes 5–7
        #[libertas_llm_doc_field]
        data: StateIncomeFields,
    },
    /// 1099-MISC — miscellaneous income
    Misc {
        /// Boxes 16–18
        #[libertas_llm_doc_field]
        data: StateIncomeFields,
    },
    /// 1099-K — payment card or third-party network
    K {
        /// Boxes 6–8
        #[libertas_llm_doc_field]
        data: StateWithholdingFields,
    },
}

/// Accepted source information for state and local returns
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IncomeStateSource {
    /// W-2
    Wage(W2StateFields),
    /// W-2G
    Gambling(StateLocalFields),
    /// 1099-INT, 1099-DIV or 1099-OID
    Investment(InvestmentStateFields),
    /// 1099-B or unemployment 1099-G
    Withholding(StateWithholdingFields),
    /// 1099-R or RRB-1099-R
    Retirement(RetirementStateFields),
    /// State or local refund 1099-G
    Refund(RefundStateFields),
    /// Nonbusiness 1099-MISC
    Misc(StateIncomeFields),
    /// SSA-1099 or RRB-1099
    SocialSecurity {
        /// Statement origin when identified
        statement: Option<SocialSecurityStatement>,
    },
    /// Business or farm information forms
    Business {
        /// Individual state information records
        documents: Vec<BusinessStateDocument>,
    },
}

impl IncomeStateSource {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Wage(v) => {
                money(v.nonqualified_plans)
                    && valid_states(&v.states)
                    && valid_localities(&v.localities)
                    && v.box14a.len() <= 32
                    && v.box14a.iter().all(|item| {
                        !item.label.is_empty()
                            && match &item.value {
                                W2OtherValue::Money { amount } => money(*amount),
                                W2OtherValue::Text { text } => !text.is_empty(),
                            }
                    })
                    && v.tipped_occupation_codes.len() <= 2
                    && v.tipped_occupation_codes.iter().all(|code| code.code <= 999)
            }
            Self::Gambling(v) => v.valid(),
            Self::Investment(v) => {
                valid_withholding(&v.states)
                    && v.exempt_sources.len() <= 56
                    && v.exempt_sources.iter().all(|row| match &row.allocation {
                        TaxExemptAllocation::Amount { amount } => money(*amount),
                        TaxExemptAllocation::Percentage { value } => percentage(*value),
                    })
                    && v.government_obligation_percentage.is_none_or(percentage)
            }
            Self::Withholding(v) => v.valid(),
            Self::Retirement(v) => v.taxes.valid(),
            Self::Refund(v) => v.taxes.valid(),
            Self::Misc(v) => v.valid(),
            Self::SocialSecurity { .. } => true,
            Self::Business { documents } => {
                documents.len() <= 100
                    && documents.iter().all(|doc| match &doc.form {
                        BusinessStateForm::Nec { data } | BusinessStateForm::Misc { data } => {
                            data.valid()
                        }
                        BusinessStateForm::K { data } => data.valid(),
                    })
            }
        }
    }
}

fn money(value: i64) -> bool {
    (0..=100000000000).contains(&value)
}
fn percentage(value: f64) -> bool {
    (0.0..=100.0).contains(&value)
}
fn valid_states(rows: &[StateIncomeTax]) -> bool {
    rows.len() <= 2
        && rows.iter().all(|row| row.income.is_none_or(money) && row.withholding.is_none_or(money))
}
fn valid_withholding(rows: &[StateTaxWithholding]) -> bool {
    rows.len() <= 2 && rows.iter().all(|row| row.withholding.is_none_or(money))
}
fn valid_localities(rows: &[LocalIncomeTax]) -> bool {
    rows.len() <= 2
        && rows.iter().all(|row| {
            !row.locality.is_empty()
                && row.income.is_none_or(money)
                && row.withholding.is_none_or(money)
        })
}
impl StateWithholdingFields {
    fn valid(&self) -> bool {
        valid_withholding(&self.states)
    }
}
impl StateIncomeFields {
    fn valid(&self) -> bool {
        valid_states(&self.states)
    }
}
impl StateLocalFields {
    fn valid(&self) -> bool {
        valid_states(&self.states) && valid_localities(&self.localities)
    }
}
