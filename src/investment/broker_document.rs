// Coverage: Forms 1099-B and 1099-DA retain the actual printed document
// separately from independent capital-sale and section 1256 answers. Their
// ephemeral components use the existing FederalSale and FederalSection1256
// calculation owners. Barter and creator receipts retain an explicit income
// boundary until linked to their actual income treatment. Quantity never
// participates in proceeds, basis, gain or withholding calculations.
// https://www.irs.gov/pub/irs-pdf/f1099b.pdf
// https://www.irs.gov/instructions/i1099b
// https://www.irs.gov/pub/irs-pdf/f1099da.pdf
// https://www.irs.gov/instructions/i1099da
use crate::document_parties::AcceptedPartySource;
use crate::validation::ValidationIssue;
use crate::{
    BasisReporting, DocumentRecipient, FederalSale, FederalSection1256, GainTerm,
    PrintedDocumentNames, PrintedParty, SaleAdjustment, SaleAmt, SaleSpecial, SaleUse,
    StateWithholdingFields,
};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Acquisition date or notation printed on a broker statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BrokerAcquisitionDate {
    /// A calendar date is printed
    Date {
        /// Printed acquisition date
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_date_only]
        date: u32,
    },
    /// VARIOUS is printed
    Various,
    /// INHERITED is printed
    Inherited,
    /// Another notation is printed
    Text {
        /// Exact printed notation
        #[libertas_llm_doc_field(allow_empty = false)]
        text: String,
    },
}

impl BrokerAcquisitionDate {
    fn date(&self) -> Option<u32> {
        match self {
            Self::Date { date } => Some(*date),
            Self::Various | Self::Inherited | Self::Text { .. } => None,
        }
    }
}

/// Applicable Form 8949 checkbox printed on Form 1099-B
// https://www.irs.gov/instructions/i1099b
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1099B8949Checkbox {
    /// A — Short-term, basis reported to the IRS
    #[libertas_form_code("A")]
    A,
    /// B — Short-term, basis not reported to the IRS
    #[libertas_form_code("B")]
    B,
    /// D — Long-term, basis reported to the IRS
    #[libertas_form_code("D")]
    D,
    /// E — Long-term, basis not reported to the IRS
    #[libertas_form_code("E")]
    E,
    /// X — Holding period unknown, basis not reported to the IRS
    #[libertas_form_code("X")]
    X,
}

/// Applicable Form 8949 checkbox printed on Form 1099-DA
// https://www.irs.gov/instructions/i1099da
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1099DA8949Checkbox {
    /// G — Short-term, basis reported to the IRS
    #[libertas_form_code("G")]
    G,
    /// H — Short-term, basis not reported to the IRS
    #[libertas_form_code("H")]
    H,
    /// J — Long-term, basis reported to the IRS
    #[libertas_form_code("J")]
    J,
    /// K — Long-term, basis not reported to the IRS
    #[libertas_form_code("K")]
    K,
    /// Y — Holding period unknown, basis not reported to the IRS
    #[libertas_form_code("Y")]
    Y,
}

/// Form 1099-B — Proceeds From Broker and Barter Exchange Transactions
#[derive(
    Default, Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual Form 1099-B or one corresponding substitute statement, retaining its headers and boxes 1 through 16 in printed order. Copy party names, TINs, addresses, account and CUSIP numbers exactly, preserving masking, punctuation and leading zeros. In the payer and recipient address blocks of Form 1099-B, the separately labeled State or province, Country, and ZIP or foreign postal code cells share one row below City or town; read their values left to right on that row. Follow the actual address row layout on a substitute statement. Copy each checkbox independently and the applicable Form 8949 checkbox as printed. Never infer an owner from the household, a date, basis, asset use, an adjustment, an AMT amount, a special treatment, an activity or another personal answer. A clearly blank optional numeric box is null, not an invented zero. A printed zero is a reported zero. Return null only for a clearly blank date box. Read box_1b_acquired only from box 1b Date acquired; the adjacent box 1c Date sold or disposed supplies box_1c_sold. For a calendar date printed in box 1b select Date; select Various or Inherited only when that notation is printed there, and retain any other printed notation verbatim in Text. VARIOUS in box 1b selects Various even when box 1c contains a date. Do not infer a notation from a blank box. Report an unavailable or illegible date box as an error; never replace it with a blank or a guessed date. Return null for clearly blank withholding. Copy each populated state row once without inferring a state or copying consolidated withholding totals to multiple forms. Copies of the same form are one document. Check the printed year against the supplied year and report a mismatch; report conflicting or illegible printed values as errors. Preserve the entire box 1a description, and fill box_1a_quantity only when a number of asset units is actually printed; never calculate it from a price or amount. A percentage or a different measure remains verbatim in the description and is not an asset count. Boxes 8 through 11 and box 13 are independent actual printed amounts, not sale proceeds. Read box 11 exactly; never manufacture it from boxes 8 through 10. Preserve the ordinary, QOF, loss-restriction and noncovered flags even when their treatment needs separate records. Do not import Form 1099-DA as this form. When a statement contains several forms or sales, report that each needs its own entry rather than adding or selecting amounts.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.year,@.box_1a_description,@.box_1a_quantity,@.box_1c_sold,@.box_1d_proceeds,@.box_1e_basis,@.box_1f_market_discount,@.box_1g_wash_sale,@.box_4_withholding,@.box_8_closed_contracts,@.box_9_prior_open_contracts,@.box_10_open_contracts,@.box_11_aggregate,@.box_13_barter,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099B {
    /// Calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// PAYER’S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: PrintedParty,
    /// PAYER’S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT’S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// CUSIP number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub cusip_number: String,
    /// FATCA filing requirement
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub fatca_filing_requirement: bool,
    /// Applicable checkbox on Form 8949
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed code is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub form_8949_checkbox: Option<Form1099B8949Checkbox>,
    /// 1a — Description of property
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_1a_description: String,
    /// 1a — Number of asset units, only when actually printed in the description
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub box_1a_quantity: Option<f64>,
    /// 1b — Date acquired
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date or notation is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub box_1b_acquired: Option<BrokerAcquisitionDate>,
    /// 1c — Date sold or disposed
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_1c_sold: Option<u32>,
    /// 1d — Proceeds
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1d_proceeds: Option<i64>,
    /// 1e — Cost or other basis
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1e_basis: Option<i64>,
    /// 1f — Accrued market discount
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1f_market_discount: Option<i64>,
    /// 1g — Wash sale loss disallowed
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1g_wash_sale: Option<i64>,
    /// 2 — Short-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_short_term: bool,
    /// 2 — Long-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_long_term: bool,
    /// 2 — Ordinary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_ordinary: bool,
    /// 3 — Proceeds from collectibles
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3_collectibles: bool,
    /// 3 — Proceeds from a QOF
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3_qof: bool,
    /// 4 — Federal income tax withheld
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_4_withholding: Option<i64>,
    /// 5 — Noncovered security
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_5_noncovered: bool,
    /// 6 — Gross proceeds reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_gross_proceeds: bool,
    /// 6 — Net proceeds reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_net_proceeds: bool,
    /// 7 — Loss is not allowed based on the amount in 1d
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_7_loss_not_allowed: bool,
    /// 8 — Profit or (loss) realized on closed contracts
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_8_closed_contracts: Option<i64>,
    /// 9 — Unrealized profit or (loss) on open contracts at the prior year-end
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_9_prior_open_contracts: Option<i64>,
    /// 10 — Unrealized profit or (loss) on open contracts at the current year-end
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_10_open_contracts: Option<i64>,
    /// 11 — Aggregate profit or (loss) on contracts
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_11_aggregate: Option<i64>,
    /// 12 — Basis reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_12_basis_reported: bool,
    /// 13 — Bartering
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_13_barter: Option<i64>,
    /// 14 — State name; 15 — State identification no.; 16 — State tax withheld
    #[libertas_llm_doc_field]
    pub state_details: StateWithholdingFields,
}
impl Eq for Form1099B {}

/// Form 1099-DA — Digital Asset Proceeds From Broker Transactions
#[derive(
    Default, Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual Form 1099-DA or one corresponding substitute statement, retaining its headers and boxes in printed order. Copy party names, TINs, addresses, account and CUSIP numbers exactly, preserving masking, punctuation and leading zeros. Copy each checkbox independently and the applicable Form 8949 checkbox as printed. Never infer an owner from the household, a date, basis, asset use, an adjustment, an AMT amount, a special treatment, an activity or another personal answer. A clearly blank optional numeric box is null, not an invented zero. A printed zero is a reported zero. Return null only for a clearly blank date box. For an actual acquisition date select Date; select Various or Inherited only when that notation is printed, and retain any other printed notation verbatim in Text. Do not infer a notation from a blank box. Report an unavailable or illegible date box as an error; never replace it with a blank or a guessed date. Return zero for clearly blank withholding. Copy each populated state row once without inferring a state or copying consolidated withholding totals to multiple forms. Copies of the same form are one document. Check the printed year against the supplied year and report a mismatch; report conflicting or illegible printed values as errors. Copy the digital asset code and name exactly; do not invent a token ID. Copy the actual fractional quantities in boxes 1c and 12a without scaling, and never use them to calculate proceeds or basis. Box 11a has independent qualifying-stablecoin and specified-NFT checkboxes; retain the actual box 11b transaction count. Box 11c is creator or minter first-sale proceeds, separate from box 1f; never move or add it to box 1f. Leave box 1f null when it is blank, including a creator-only form. Leave box 12b null when clearly blank, including transfers on varied dates. Boxes 10 and 13 are reserved and have no supplied amount. Preserve all reporting, cash-only, customer-acquisition, noncovered, ordinary, QOF and loss-restriction flags. Do not import Form 1099-B as this form. When a statement contains several forms, report that each needs its own entry rather than adding or selecting amounts.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.filer.federal_id,@.recipient.federal_id,@.year,@.box_1a_asset_code,@.box_1b_asset_name,@.box_1c_units,@.box_1e_sold,@.box_1f_proceeds,@.box_1g_basis,@.box_1h_market_discount,@.box_1i_wash_sale,@.box_4_withholding,@.box_11c_creator_proceeds,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099DA {
    /// Calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// FILER’S name, TIN and address
    #[libertas_llm_doc_field]
    pub filer: PrintedParty,
    /// FILER’S telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub filer_telephone: String,
    /// RECIPIENT’S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// CUSIP number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub cusip_number: String,
    /// Applicable checkbox on Form 8949
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed code is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub form_8949_checkbox: Option<Form1099DA8949Checkbox>,
    /// 1a — Code for digital asset
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_1a_asset_code: String,
    /// 1b — Name of digital asset
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_1b_asset_name: String,
    /// 1c — Number of units
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub box_1c_units: Option<f64>,
    /// 1d — Date acquired
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date or notation is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub box_1d_acquired: Option<BrokerAcquisitionDate>,
    /// 1e — Date sold or disposed
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_1e_sold: Option<u32>,
    /// 1f — Proceeds
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1f_proceeds: Option<i64>,
    /// 1g — Cost or other basis
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1g_basis: Option<i64>,
    /// 1h — Accrued market discount
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1h_market_discount: Option<i64>,
    /// 1i — Wash sale loss disallowed
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_1i_wash_sale: Option<i64>,
    /// 2 — Basis reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_2_basis_reported: bool,
    /// 3a — Gross proceeds reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3a_gross_proceeds: bool,
    /// 3a — Net proceeds reported to IRS
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3a_net_proceeds: bool,
    /// 3b — Reserved for future use
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3b_reserved: bool,
    /// 3b — Proceeds from a QOF
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_3b_qof: bool,
    /// 4 — Federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_4_withholding: i64,
    /// 5 — Loss is not allowed based on the amount in 1f
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_5_loss_not_allowed: bool,
    /// 6 — Short-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_short_term: bool,
    /// 6 — Long-term gain or loss
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_long_term: bool,
    /// 6 — Ordinary
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_ordinary: bool,
    /// 7 — Only cash proceeds were paid in the transaction
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_7_cash_only: bool,
    /// 8 — Broker relied on customer-provided acquisition information
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_8_customer_acquisition: bool,
    /// 9 — Digital asset is a noncovered security
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_9_noncovered: bool,
    /// 11a — Aggregate reporting for qualifying stablecoins
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_11a_stablecoins: bool,
    /// 11a — Aggregate reporting for specified NFTs
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_11a_nfts: bool,
    /// 11b — Number of transactions
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub box_11b_transactions: Option<u32>,
    /// 11c — Aggregate gross proceeds of specified NFTs attributable to first sales by a creator or minter
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box_11c_creator_proceeds: Option<i64>,
    /// 12a — Number of units transferred in
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub box_12a_transferred_units: Option<f64>,
    /// 12b — Transfer-in date
    /// [NullText]
    /// This box is blank
    /// [SetText]
    /// The printed date is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_12b_transfer_date: Option<u32>,
    /// 14 — State name; 15 — State identification no.; 16 — State tax withheld
    #[libertas_llm_doc_field]
    pub state_details: StateWithholdingFields,
}
impl Eq for Form1099DA {}

/// The actual kind of printed broker document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum BrokerPaper {
    /// Form 1099-B
    Form1099B(Form1099B),
    /// Form 1099-DA
    Form1099DA(Form1099DA),
}

/// Independently established asset use, without a second copy of a printed loss
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BrokerSaleUse {
    Investment,
    Personal,
    WrittenOption,
}
impl BrokerSaleUse {
    fn native(self, wash_sale: i64) -> SaleUse {
        match self {
            Self::Investment => SaleUse::Investment { wash_sale },
            Self::Personal => SaleUse::Personal,
            Self::WrittenOption => SaleUse::WrittenOption { wash_sale },
        }
    }
    fn from_native(value: &SaleUse) -> Self {
        match value {
            SaleUse::Investment { .. } => Self::Investment,
            SaleUse::Personal => Self::Personal,
            SaleUse::WrittenOption { .. } => Self::WrittenOption,
        }
    }
}

/// An accepted answer to the existing capital-sale basis question
/// A missing outer answer remains unanswered. A present answer with no amount
/// is the person's explicit choice to assume zero when basis cannot be established.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BrokerBasisAnswer {
    /// Established cost or other basis
    /// [NullText]
    /// Basis cannot be established; assume zero basis
    /// [SetText]
    /// My established cost or other basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: Option<i64>,
}

/// Independent sale adjustment, without a second copy of the broker's printed basis
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BrokerSaleAdjustment {
    /// No adjustment code applies
    None,
    /// Code B: the printed basis was incorrect; use my independent basis records
    BasisCorrection,
    /// Code T: the printed holding period was incorrect
    HoldingPeriodCorrection,
    /// Code E: selling expenses not reflected in the printed proceeds
    SellingExpenses {
        /// Selling expenses not already subtracted from proceeds
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Code N: I received this form as nominee for the actual owner
    Nominee,
    /// Code E: option premium not reflected in the printed proceeds
    OptionPremium {
        /// Positive for a premium received, negative for a premium paid
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        amount: i64,
    },
}
impl BrokerSaleAdjustment {
    fn native(&self, printed_basis: Option<i64>) -> Result<SaleAdjustment, &'static str> {
        Ok(match self {
            Self::None => SaleAdjustment::None,
            Self::BasisCorrection => SaleAdjustment::BasisCorrection {
                reported_basis: printed_basis.ok_or("BROKER_CORRECTION_PRINTED_BASIS_NEEDED")?,
            },
            Self::HoldingPeriodCorrection => SaleAdjustment::HoldingPeriodCorrection,
            Self::SellingExpenses { amount } => SaleAdjustment::SellingExpenses { amount: *amount },
            Self::Nominee => SaleAdjustment::Nominee,
            Self::OptionPremium { amount } => SaleAdjustment::OptionPremium { amount: *amount },
        })
    }
}

/// Independent sale records, without copies of the broker's printed boxes.
/// Basis and term records supplement missing paper facts or support the actual
/// Form 8949 code B/T adjustments. Other paper corrections update the paper.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SalePersonalRecords {
    pub use_kind: Option<BrokerSaleUse>,
    pub term: Option<GainTerm>,
    pub basis: Option<BrokerBasisAnswer>,
    /// A digital asset's collectible character, which Form 1099-DA does not print.
    pub collectible: Option<bool>,
    /// Several securities sales on a substitute statement, without a printed aggregate checkbox.
    pub several_sales: Option<bool>,
    pub adjustment: BrokerSaleAdjustment,
    pub special: SaleSpecial,
    pub amt: SaleAmt,
}
impl Default for SalePersonalRecords {
    fn default() -> Self {
        Self {
            use_kind: None,
            term: None,
            basis: None,
            collectible: None,
            several_sales: None,
            adjustment: BrokerSaleAdjustment::None,
            special: SaleSpecial::NoneOfThese,
            amt: SaleAmt::SameAsRegular,
        }
    }
}

/// Independent section 1256 treatment
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ContractsPersonalRecords {
    pub special_treatment: Option<bool>,
}

/// Independent answers belonging to this stable broker-document identity
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct BrokerPersonalRecords {
    pub sale: SalePersonalRecords,
    pub contracts: ContractsPersonalRecords,
}

/// Accepted broker paper and its independent answers
/// The containing income entry owns the stable source identity and attribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BrokerDocument {
    pub paper: BrokerPaper,
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: DocumentRecipient,
    /// How the App assigned the owner
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub owner_source: Option<crate::OwnerSource>,
    /// Your share of a joint document
    /// Percent of the document's amounts that are yours; your spouse's share is the rest.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub joint_share: Option<i32>,
    #[libertas_hidden]
    pub personal: BrokerPersonalRecords,
}

/// Your existing capital-sale answers, projected for the topic editor
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BrokerSaleAnswers {
    /// How the property was used
    /// [NullText]
    /// Choose how the property was used
    /// [SetText]
    /// The property was
    pub use_kind: Option<SaleUse>,
    /// Established holding period
    /// [NullText]
    /// The holding period is not established
    pub term: Option<GainTerm>,
    /// Your basis records
    /// [NullText]
    /// Not answered yet
    /// [SetText]
    /// My basis answer is
    pub basis: Option<BrokerBasisAnswer>,
    /// Collectible capital asset
    pub collectible: bool,
    /// Accrued market discount to include in income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_discount: i64,
    /// This entry totals several sales
    pub several_sales: bool,
    /// Correction to this sale
    pub adjustment: BrokerSaleAdjustment,
    /// A treatment this page does not calculate
    pub special: SaleSpecial,
    /// AMT sale facts
    pub amt: SaleAmt,
}

/// Actual noncapital receipts printed on a broker document
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrokerReceiptKind {
    Barter,
    Creator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BrokerReceipt {
    pub kind: BrokerReceiptKind,
    pub amount: i64,
}
impl BrokerReceipt {
    pub(crate) fn boundary(self) -> &'static str {
        match self.kind {
            BrokerReceiptKind::Barter => "BROKER_BARTER_RECEIPTS_NOT_CALCULATED",
            BrokerReceiptKind::Creator => "BROKER_CREATOR_RECEIPTS_NOT_CALCULATED",
        }
    }
}

// Runtime views only. Each component has zero withholding: the document owns
// its one actual federal withholding amount even when a component needs records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrokerProjection {
    pub sale: Option<FederalSale>,
    pub contracts: Option<FederalSection1256>,
    pub receipt: Option<BrokerReceipt>,
    pub withholding: i64,
    pub sale_boundary: Option<&'static str>,
    pub contracts_boundary: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct SalePaperFacts {
    proceeds: Option<i64>,
    basis: Option<i64>,
    term: Option<GainTerm>,
    wash_sale: i64,
    market_discount: i64,
    collectible: bool,
    several_sales: bool,
    digital_asset: bool,
    reporting: BasisReporting,
    ordinary: bool,
    loss_not_allowed: bool,
    qof: bool,
}

fn printed_term(short: bool, long: bool) -> Option<GainTerm> {
    match (short, long) {
        (true, false) => Some(GainTerm::Short),
        (false, true) => Some(GainTerm::Long),
        _ => None,
    }
}

fn printed_reporting(reported: bool) -> BasisReporting {
    if reported { BasisReporting::ReportedToIrs } else { BasisReporting::NotReported }
}

fn wash_sale(value: &SaleUse) -> i64 {
    match value {
        SaleUse::Investment { wash_sale } | SaleUse::WrittenOption { wash_sale } => *wash_sale,
        SaleUse::Personal => 0,
    }
}

impl BrokerPaper {
    pub(crate) fn active(&self) -> bool {
        match self {
            // A recipient copy prints no VOID mark: every received paper counts.
            Self::Form1099B(_) | Self::Form1099DA(_) => true,
        }
    }
    fn sale_facts(&self) -> SalePaperFacts {
        match self {
            Self::Form1099B(form) => SalePaperFacts {
                proceeds: form.box_1d_proceeds,
                basis: form.box_1e_basis,
                term: printed_term(form.box_2_short_term, form.box_2_long_term),
                wash_sale: form.box_1g_wash_sale.unwrap_or(0),
                market_discount: form.box_1f_market_discount.unwrap_or(0),
                collectible: form.box_3_collectibles,
                several_sales: false,
                digital_asset: false,
                reporting: printed_reporting(
                    form.box_12_basis_reported
                        || matches!(
                            form.form_8949_checkbox,
                            Some(Form1099B8949Checkbox::A | Form1099B8949Checkbox::D)
                        ),
                ),
                ordinary: form.box_2_ordinary,
                loss_not_allowed: form.box_7_loss_not_allowed,
                qof: form.box_3_qof,
            },
            Self::Form1099DA(form) => SalePaperFacts {
                proceeds: form.box_1f_proceeds,
                basis: form.box_1g_basis,
                term: printed_term(form.box_6_short_term, form.box_6_long_term),
                wash_sale: form.box_1i_wash_sale.unwrap_or(0),
                market_discount: form.box_1h_market_discount.unwrap_or(0),
                collectible: false,
                several_sales: form.box_11a_stablecoins || form.box_11a_nfts,
                digital_asset: true,
                reporting: printed_reporting(
                    form.box_2_basis_reported
                        || matches!(
                            form.form_8949_checkbox,
                            Some(Form1099DA8949Checkbox::G | Form1099DA8949Checkbox::J)
                        ),
                ),
                ordinary: form.box_6_ordinary,
                loss_not_allowed: form.box_5_loss_not_allowed,
                qof: form.box_3b_qof,
            },
        }
    }
    pub(crate) fn has_contracts(&self) -> bool {
        if !self.active() {
            return false;
        }
        match self {
            Self::Form1099B(form) => [
                form.box_8_closed_contracts,
                form.box_9_prior_open_contracts,
                form.box_10_open_contracts,
                form.box_11_aggregate,
            ]
            .iter()
            .any(Option::is_some),
            Self::Form1099DA(_) => false,
        }
    }
    pub(crate) fn has_sale(&self) -> bool {
        if !self.active() {
            return false;
        }
        match self {
            Self::Form1099B(form) => {
                form.box_1d_proceeds.is_some()
                    || (!self.has_contracts()
                        && form.box_13_barter.is_none()
                        && (!form.box_1a_description.is_empty()
                            || form.box_1e_basis.is_some()
                            || form.box_2_short_term
                            || form.box_2_long_term))
            }
            Self::Form1099DA(form) => {
                form.box_1f_proceeds.is_some() || form.box_11c_creator_proceeds.is_none()
            }
        }
    }
    fn parties(&self) -> (&PrintedParty, &PrintedParty) {
        match self {
            Self::Form1099B(form) => (&form.recipient, &form.payer),
            Self::Form1099DA(form) => (&form.recipient, &form.filer),
        }
    }
    pub(crate) fn withholding(&self) -> i64 {
        if !self.active() {
            return 0;
        }
        match self {
            Self::Form1099B(form) => form.box_4_withholding.unwrap_or(0),
            Self::Form1099DA(form) => form.box_4_withholding,
        }
    }
    fn receipt(&self) -> Option<BrokerReceipt> {
        if !self.active() {
            return None;
        }
        match self {
            Self::Form1099B(form) => form
                .box_13_barter
                .map(|amount| BrokerReceipt { kind: BrokerReceiptKind::Barter, amount }),
            Self::Form1099DA(form) => form
                .box_11c_creator_proceeds
                .map(|amount| BrokerReceipt { kind: BrokerReceiptKind::Creator, amount }),
        }
    }
}

impl BrokerDocument {
    pub(crate) fn new(paper: BrokerPaper) -> Self {
        Self {
            paper,
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
            personal: BrokerPersonalRecords::default(),
        }
    }
    pub(crate) fn printed_names(&self) -> PrintedDocumentNames {
        let (recipient, issuer) = self.paper.parties();
        PrintedDocumentNames {
            recipient: recipient.clone(),
            issuer: Some(issuer.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        }
    }
    pub(crate) fn replace_paper(&mut self, paper: BrokerPaper) {
        let before = self.printed_names();
        self.paper = paper;
        let mut current = self.printed_names();
        current.retain_assignment_from(Some(&before));
        current.assignment().apply_to(
            &mut self.assigned,
            &mut self.owner_source,
            &mut self.joint_share,
        );
    }
    pub(crate) fn sale_answers(&self) -> BrokerSaleAnswers {
        let paper = self.paper.sale_facts();
        let personal = &self.personal.sale;
        let basis = if matches!(personal.adjustment, BrokerSaleAdjustment::BasisCorrection) {
            personal.basis.clone()
        } else {
            paper
                .basis
                .map(|amount| BrokerBasisAnswer { amount: Some(amount) })
                .or_else(|| personal.basis.clone())
        };
        let term = if matches!(personal.adjustment, BrokerSaleAdjustment::HoldingPeriodCorrection) {
            personal.term
        } else {
            paper.term.or(personal.term)
        };
        BrokerSaleAnswers {
            use_kind: personal.use_kind.map(|kind| kind.native(paper.wash_sale)),
            term,
            basis,
            collectible: match self.paper {
                BrokerPaper::Form1099B(_) => paper.collectible,
                BrokerPaper::Form1099DA(_) => personal.collectible.unwrap_or(false),
            },
            market_discount: paper.market_discount,
            several_sales: match self.paper {
                BrokerPaper::Form1099B(_) => personal.several_sales.unwrap_or(false),
                BrokerPaper::Form1099DA(_) => paper.several_sales,
            },
            adjustment: personal.adjustment.clone(),
            special: personal.special,
            amt: personal.amt.clone(),
        }
    }
    pub(crate) fn apply_sale_answers(&mut self, answers: &BrokerSaleAnswers) {
        let before = self.paper.sale_facts();
        let basis_correction = matches!(answers.adjustment, BrokerSaleAdjustment::BasisCorrection);
        let term_correction =
            matches!(answers.adjustment, BrokerSaleAdjustment::HoldingPeriodCorrection);
        let personal = &mut self.personal.sale;
        personal.use_kind = answers.use_kind.as_ref().map(BrokerSaleUse::from_native);
        personal.term = if before.term.is_none() || term_correction { answers.term } else { None };
        personal.basis =
            if before.basis.is_none() || basis_correction { answers.basis.clone() } else { None };
        let wash = answers
            .use_kind
            .as_ref()
            .filter(|value| !matches!(value, SaleUse::Personal))
            .map(wash_sale);
        match &mut self.paper {
            BrokerPaper::Form1099B(form) => {
                if before.term.is_some() && !term_correction {
                    form.box_2_short_term = answers.term == Some(GainTerm::Short);
                    form.box_2_long_term = answers.term == Some(GainTerm::Long);
                }
                if before.basis.is_some() && !basis_correction {
                    form.box_1e_basis = answers.basis.as_ref().and_then(|value| value.amount);
                }
                if let Some(wash) = wash.filter(|value| *value != before.wash_sale) {
                    form.box_1g_wash_sale = Some(wash);
                }
                if answers.market_discount != before.market_discount {
                    form.box_1f_market_discount = Some(answers.market_discount);
                }
                form.box_3_collectibles = answers.collectible;
                personal.collectible = None;
                personal.several_sales = answers.several_sales.then_some(true);
            }
            BrokerPaper::Form1099DA(form) => {
                if before.term.is_some() && !term_correction {
                    form.box_6_short_term = answers.term == Some(GainTerm::Short);
                    form.box_6_long_term = answers.term == Some(GainTerm::Long);
                }
                if before.basis.is_some() && !basis_correction {
                    form.box_1g_basis = answers.basis.as_ref().and_then(|value| value.amount);
                }
                if let Some(wash) = wash.filter(|value| *value != before.wash_sale) {
                    form.box_1i_wash_sale = Some(wash);
                }
                if answers.market_discount != before.market_discount {
                    form.box_1h_market_discount = Some(answers.market_discount);
                }
                personal.collectible = answers.collectible.then_some(true);
                personal.several_sales = None;
            }
        }
        personal.adjustment = answers.adjustment.clone();
        personal.special = answers.special;
        personal.amt = answers.amt.clone();
    }
    pub(crate) fn projection(&self) -> BrokerProjection {
        let mut view = BrokerProjection {
            sale: None,
            contracts: None,
            receipt: self.paper.receipt(),
            withholding: self.paper.withholding(),
            sale_boundary: None,
            contracts_boundary: None,
        };
        if self.paper.has_sale() {
            match self.sale_projection() {
                Ok(sale) => view.sale = Some(sale),
                Err(key) => view.sale_boundary = Some(key),
            }
        }
        if self.paper.has_contracts() {
            match self.contracts_projection() {
                Ok(contracts) => view.contracts = Some(contracts),
                Err(key) => view.contracts_boundary = Some(key),
            }
        }
        if !self.paper.active() {
            return view;
        }
        if !self.paper.has_sale() && !self.paper.has_contracts() && view.receipt.is_none() {
            view.sale_boundary = Some("BROKER_SOURCE_AMOUNT_NEEDED");
        }
        view
    }
    fn sale_projection(&self) -> Result<FederalSale, &'static str> {
        let paper = self.paper.sale_facts();
        // These source conditions belong to the document. They never rewrite
        // the person's independently accepted SaleSpecial answer.
        if paper.ordinary {
            return Err("BROKER_ORDINARY_NOT_CALCULATED");
        }
        if paper.loss_not_allowed {
            return Err("BROKER_LOSS_STATEMENT_NEEDED");
        }
        if paper.qof {
            return Err("BROKER_QOF_RECORDS_NEEDED");
        }
        let proceeds = paper.proceeds.ok_or("BROKER_SALE_PROCEEDS_NEEDED")?;
        let answers = self.sale_answers();
        let use_kind = answers.use_kind.ok_or("BROKER_SALE_USE_NEEDED")?;
        let term = answers.term.ok_or("SALE_TERM_NEEDED")?;
        let basis = answers.basis.ok_or("BROKER_SALE_BASIS_NEEDED")?.amount;
        let sale = FederalSale {
            term,
            collectible: answers.collectible,
            proceeds,
            basis,
            amt: answers.amt,
            withholding: 0,
            use_kind,
            special: answers.special,
            adjustment: answers.adjustment.native(paper.basis)?,
            basis_reporting: paper.reporting,
            digital_asset: paper.digital_asset,
            market_discount: answers.market_discount,
            several_sales: answers.several_sales,
        };
        if let Some(issue) = crate::capital_sale_model::native_sale_issue(&sale) {
            return Err(issue.key);
        }
        if let Some(key) = sale.arithmetic_issue() {
            return Err(key);
        }
        Ok(sale)
    }
    fn contracts_projection(&self) -> Result<FederalSection1256, &'static str> {
        let BrokerPaper::Form1099B(form) = &self.paper else {
            return Err("BROKER_CONTRACTS_AGGREGATE_NEEDED");
        };
        // The actual printed box 11 is authoritative. Boxes 8–10 are retained
        // and reconciled, but never supply a missing aggregate.
        let aggregate = form.box_11_aggregate.ok_or("BROKER_CONTRACTS_AGGREGATE_NEEDED")?;
        let special_treatment =
            self.personal.contracts.special_treatment.ok_or("BROKER_CONTRACTS_TREATMENT_NEEDED")?;
        Ok(FederalSection1256 { aggregate, withholding: 0, special_treatment })
    }
}

impl AcceptedPartySource for BrokerDocument {
    fn accepted_parties(&self) -> Option<PrintedDocumentNames> {
        Some(self.printed_names())
    }
    fn set_document_recipient(&mut self, recipient: crate::document_parties::Assignment) -> bool {
        recipient.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
        true
    }
}

impl BrokerPaper {
    /// The accepted source of a printed paper: the independent answers and the
    /// person assignment of an accepted document survive a correction of its paper.
    pub(crate) fn into_document(self, accepted: Option<&BrokerDocument>) -> BrokerDocument {
        let mut source = accepted.cloned().unwrap_or_else(|| BrokerDocument::new(self.clone()));
        source.replace_paper(self);
        source
    }
}

impl Form1099B {
    // No value is supplied when a printed operand is absent. This is a check
    // against the printed aggregate, never a replacement for box 11.
    pub(crate) fn contracts_reconciliation(&self) -> Option<bool> {
        let closed = i128::from(self.box_8_closed_contracts?);
        let prior = i128::from(self.box_9_prior_open_contracts?);
        let current = i128::from(self.box_10_open_contracts?);
        Some(closed - prior + current == i128::from(self.box_11_aggregate?))
    }
}

fn dates_issue(
    acquired: &Option<BrokerAcquisitionDate>,
    sold: &Option<u32>,
    sold_field: &'static str,
) -> Option<ValidationIssue> {
    if let (Some(acquired), Some(sold)) =
        (acquired.as_ref().and_then(BrokerAcquisitionDate::date), sold)
        && acquired > *sold
    {
        return Some(ValidationIssue::new("BROKER_SALE_BEFORE_ACQUIRED").at_field(sold_field));
    }
    None
}

// Import checks supplied printed relationships without constructing personal answers.
impl Form1099B {
    pub(crate) fn admission_issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = dates_issue(&self.box_1b_acquired, &self.box_1c_sold, "box_1c_sold") {
            return Some(issue.in_field("form"));
        }
        if self.contracts_reconciliation() == Some(false) {
            return Some(
                ValidationIssue::new("BROKER_CONTRACTS_RECONCILE")
                    .at_field("box_11_aggregate")
                    .in_field("form"),
            );
        }
        printed_choices_issue(
            self.box_2_short_term,
            self.box_2_long_term,
            self.box_6_gross_proceeds,
            self.box_6_net_proceeds,
            "box_2_long_term",
            "box_6_net_proceeds",
        )
    }
}

impl Form1099DA {
    pub(crate) fn admission_issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = dates_issue(&self.box_1d_acquired, &self.box_1e_sold, "box_1e_sold") {
            return Some(issue.in_field("form"));
        }
        if self.box_11a_stablecoins && self.box_11a_nfts {
            return Some(
                ValidationIssue::new("BROKER_OPTIONAL_METHOD_CONFLICT")
                    .at_field("box_11a_nfts")
                    .in_field("form"),
            );
        }
        printed_choices_issue(
            self.box_6_short_term,
            self.box_6_long_term,
            self.box_3a_gross_proceeds,
            self.box_3a_net_proceeds,
            "box_6_long_term",
            "box_3a_net_proceeds",
        )
    }
}

fn printed_choices_issue(
    short: bool,
    long: bool,
    gross: bool,
    net: bool,
    term_field: &'static str,
    proceeds_field: &'static str,
) -> Option<ValidationIssue> {
    if short && long {
        Some(
            ValidationIssue::new("BROKER_PRINTED_TERM_CONFLICT")
                .at_field(term_field)
                .in_field("form"),
        )
    } else if gross && net {
        Some(
            ValidationIssue::new("BROKER_PRINTED_PROCEEDS_CONFLICT")
                .at_field(proceeds_field)
                .in_field("form"),
        )
    } else {
        None
    }
}

pub(crate) fn printed_year_issue(printed: u32, tax_year: i64) -> Option<ValidationIssue> {
    (i64::from(printed) != tax_year)
        .then(|| ValidationIssue::new("DOCUMENT_YEAR_MISMATCH").at_field("year").in_field("form"))
}

impl BrokerDocument {
    pub(crate) fn admission_issue(&self) -> Option<ValidationIssue> {
        match &self.paper {
            BrokerPaper::Form1099B(form) => form.admission_issue(),
            BrokerPaper::Form1099DA(form) => form.admission_issue(),
        }
    }
    pub(crate) fn year_issue(&self, tax_year: impl Into<i64>) -> Option<ValidationIssue> {
        let printed = match &self.paper {
            BrokerPaper::Form1099B(form) => form.year,
            BrokerPaper::Form1099DA(form) => form.year,
        };
        printed_year_issue(printed, tax_year.into())
    }
    pub(crate) fn sale_answers_issue(
        &self,
        answers: &BrokerSaleAnswers,
    ) -> Option<ValidationIssue> {
        let paper = self.paper.sale_facts();
        if matches!(self.paper, BrokerPaper::Form1099DA(_))
            && answers.several_sales != paper.several_sales
        {
            return Some(
                ValidationIssue::new("BROKER_PRINTED_VALUES_IN_DOCUMENT").at_field("several_sales"),
            );
        }
        let Some(use_kind) = &answers.use_kind else {
            return Some(ValidationIssue::new("BROKER_SALE_USE_NEEDED").at_field("use_kind"));
        };
        let Some(term) = answers.term else {
            return Some(ValidationIssue::new("SALE_TERM_NEEDED").at_field("term"));
        };
        let Some(basis) = &answers.basis else {
            return Some(ValidationIssue::new("BROKER_SALE_BASIS_NEEDED").at_field("basis"));
        };
        let proceeds = paper.proceeds?;
        let adjustment = match answers.adjustment.native(paper.basis) {
            Ok(adjustment) => adjustment,
            Err(key) => return Some(ValidationIssue::new(key).at_field("adjustment")),
        };
        let issue = crate::capital_sale_model::native_sale_issue(&FederalSale {
            term,
            collectible: answers.collectible,
            proceeds,
            basis: basis.amount,
            amt: answers.amt.clone(),
            withholding: 0,
            use_kind: use_kind.clone(),
            special: answers.special,
            adjustment,
            basis_reporting: paper.reporting,
            digital_asset: paper.digital_asset,
            market_discount: answers.market_discount,
            several_sales: answers.several_sales,
        });
        issue.map(|issue| {
            if issue.key == "SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY" {
                ValidationIssue::new(issue.key).at_field("use_kind")
            } else {
                issue
            }
        })
    }
}

impl BrokerDocument {
    pub(crate) fn sale_needs_answers(&self) -> bool {
        let paper = self.paper.sale_facts();
        self.paper.active()
            && self.paper.has_sale()
            && paper.proceeds.is_some()
            && !paper.ordinary
            && !paper.loss_not_allowed
            && !paper.qof
    }
    pub(crate) fn contracts_needs_answers(&self) -> bool {
        self.paper.active()
            && matches!(&self.paper,
            BrokerPaper::Form1099B(form) if form.box_11_aggregate.is_some())
    }
}
