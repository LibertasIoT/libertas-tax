// Printed ESPP acquisition documents and direct acquisition and sale-answer facts.
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 3922 — transfer of stock acquired through an employee stock purchase plan
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read Form 3922 exactly, including corporation and employee names, addresses and identification numbers, account number and correction mark. Preserve masking and leading zeros. Read boxes 1 through 8 in their printed order. Return null for a blank box 8; it does not mean zero. Do not infer a sale, a sale date, compensation, wage inclusion or any other personal fact. Box 7 is the first transfer of legal title, not necessarily a sale. Preserve the actual grant and exercise dates, including those in earlier years. Copies of the same form are one document. Report illegible or conflicting printed values as errors.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form3922 {
    /// CORPORATION'S name, address and federal identification number
    #[libertas_llm_doc_field]
    pub corporation: crate::PrintedParty,
    /// EMPLOYEE'S name, address and identifying number
    #[libertas_llm_doc_field]
    pub employee: crate::PrintedParty,
    /// Account number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// Box 1 date option granted
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub granted: u32,
    /// Box 2 date option exercised
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub exercised: u32,
    /// Box 3 fair market value per share on grant date
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub grant_market: i64,
    /// Box 4 fair market value per share on exercise date
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_price: i64,
    /// Box 5 exercise price paid per share
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exercise_price: i64,
    /// Box 6 number of shares transferred
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Box 7 date legal title first transferred
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub transferred: u32,
    /// Box 8 exercise price per share determined as if the option was exercised on the date shown in box 1
    /// [NullText]
    /// Box 8 is blank
    /// [SetText]
    /// Box 8 shows
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub grant_price: Option<i64>,
}
impl Eq for Form3922 {}
impl Form3922 {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        form3922_date_issue(self.granted, self.exercised, self.transferred)
    }
}

fn form3922_date_issue(
    granted: u32,
    exercised: u32,
    transferred: u32,
) -> Option<crate::validation::ValidationIssue> {
    if granted > exercised {
        Some(
            crate::validation::ValidationIssue::new("ESPP_EXERCISED_BEFORE_GRANTED")
                .at_field("exercised"),
        )
    } else if transferred < exercised {
        Some(
            crate::validation::ValidationIssue::new("ESPP_TRANSFERRED_BEFORE_EXERCISED")
                .at_field("transferred"),
        )
    } else {
        None
    }
}

/// Form 3922
/// Transfer of Stock Acquired Through an Employee Stock Purchase Plan Under Section 423(c).
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 3922 exactly as printed. Read CORRECTED, the CORPORATION and EMPLOYEE name, address and identification-number blocks, and the account number, preserving masking, punctuation and leading zeros. Use empty text for absent identity and account fields. Read boxes 1 through 8 in printed order. Never invent missing dates. Return zero for clearly blank boxes 3 through 6; return null for a blank box 8, which means the exercise price was fixed or determinable on the grant date and does not mean zero. Read box 6 with up to six decimal places. Box 7 is the first transfer of legal title, not necessarily a sale. Preserve grant and exercise dates in earlier years. Do not infer a sale, compensation, wage inclusion, a fixed grant-date price or other personal facts. Report illegible or conflicting printed values as errors. Multiple copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.corporation.federal_id,@.employee.federal_id,@.granted,@.exercised,@.transferred,@.shares,@.grant_market,@.market_price,@.exercise_price,@.grant_price,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form3922Paper {
    /// CORRECTED
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// CORPORATION'S name, street address, city or town, state or province, country, and ZIP or foreign postal code; CORPORATION'S federal identification number
    #[libertas_llm_doc_field]
    pub corporation: crate::PrintedParty,
    /// EMPLOYEE'S identification number; EMPLOYEE'S name; street address (including apt. no.); city or town, state or province, country, and ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub employee: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Date option granted
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub granted: u32,
    /// Box 2 Date option exercised
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub exercised: u32,
    /// Box 3 Fair market value per share on grant date
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub grant_market: i64,
    /// Box 4 Fair market value per share on exercise date
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_price: i64,
    /// Box 5 Exercise price paid per share
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exercise_price: i64,
    /// Box 6 No. of shares transferred
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_number(min = 0, step = 0.000001)]
    #[libertas_default(0)]
    pub shares: f64,
    /// Box 7 Date legal title transferred
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub transferred: u32,
    /// Box 8 Exercise price per share determined as if the option was exercised on the date shown in box 1
    /// [NullText]
    /// Box 8 is blank
    /// [SetText]
    /// Box 8 shows
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub grant_price: Option<i64>,
}
impl Eq for Form3922Paper {}

impl Form3922Paper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        form3922_date_issue(self.granted, self.exercised, self.transferred)
    }
}

impl From<&Form3922Paper> for Form3922 {
    fn from(paper: &Form3922Paper) -> Self {
        let Form3922Paper {
            corrected,
            corporation,
            employee,
            account_number,
            granted,
            exercised,
            grant_market,
            market_price,
            exercise_price,
            shares,
            transferred,
            grant_price,
        } = paper;
        Self {
            corporation: corporation.clone(),
            employee: employee.clone(),
            account_number: account_number.clone(),
            corrected: *corrected,
            granted: *granted,
            exercised: *exercised,
            grant_market: *grant_market,
            market_price: *market_price,
            exercise_price: *exercise_price,
            shares: *shares,
            transferred: *transferred,
            grant_price: *grant_price,
        }
    }
}

/// Accepted employee stock purchase plan acquisition
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppAcquisition {
    /// Accepted document identity
    pub id: i64,
    /// Accepted person association
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: crate::DocumentRecipient,
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
    /// Employee on this return
    pub owner: crate::Owner,
    /// Printed acquisition document
    pub form: Form3922Paper,
    /// Actual fixed grant-date price when printed box 8 is absent
    pub fixed_grant_price: crate::SourceCents,
}

/// About this sale
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct EsppShareSaleMore {
    /// Selling fees already subtracted from box 1d
    #[libertas_money("USD", 2)]
    // Arithmetic safety: preserve the ESPP monetary owner's admitted lot and amount limit.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub selling_fees: i64,
    /// Commissions paid when you bought, for these shares
    #[libertas_money("USD", 2)]
    // Arithmetic safety: preserve the ESPP monetary owner's admitted lot and amount limit.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_costs: i64,
    /// Wash sale loss disallowed
    /// When you bought the same stock within 30 days before or after selling these shares at a loss, including a plan purchase in that window.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub wash_sale: i64,
    /// A certificate of divestiture, a gift, the owner's death, or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}
impl Eq for EsppShareSaleMore {}

/// Acquisition facts from actual stock-plan records when no Form 3922 was supplied
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppAcquisitionFacts {
    /// Date option granted
    #[libertas_date_only]
    pub granted: u32,
    /// Date option exercised and shares acquired
    #[libertas_date_only]
    pub exercised: u32,
    /// Fair market value per share on the grant date
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_market: i64,
    /// Fair market value per share on the exercise date
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Exercise price paid per share
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Exercise price per share as if exercised on the grant date
    /// The purchase price the plan fixes from the grant-date value, as Form 3922 box 8 would show it. The plan's records show it.
    pub grant_price: crate::SourceCents,
}

/// Employer's stock plan record of the purchase
/// The purchase of the sold shares, as the employer's plan record printed with the 1099-B shows it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppPlanRecord {
    /// Option grant date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub granted: u32,
    /// Purchase date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub exercised: u32,
    /// Market value per share on the grant date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_market: i64,
    /// Market value per share on the purchase date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Price paid per share
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Purchase price per share as if bought on the grant date
    /// [NullText]
    /// The record does not show it
    /// [SetText]
    /// The record shows
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub grant_price: Option<i64>,
}

/// Acquisition source for an actual ESPP disposition
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EsppAcquisitionChoice {
    /// A Form 3922 already entered in Documents
    Printed {
        /// Acquisition document
        document_id: i64,
    },
    /// Actual acquisition records when no Form 3922 is available
    Records { facts: EsppAcquisitionFacts },
}

/// Investment-sale answers owned by their accepted document identity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EsppSaleAnswers {
    /// Accepted printed sale identity
    pub document_id: i64,
    /// Acquisition source, initially unanswered
    pub acquisition: Option<EsppAcquisitionChoice>,
    /// Actual fees, wash-sale adjustment and special treatment
    pub more: EsppShareSaleMore,
    /// Whether W-2 wages already include the positive compensation
    pub wage_inclusion: Option<bool>,
    /// Grant-date price for a sale whose plan record does not show it
    pub fixed_grant_price: crate::SourceCents,
}
impl EsppSaleAnswers {
    pub(crate) fn unanswered(document_id: i64) -> Self {
        Self {
            document_id,
            acquisition: None,
            more: EsppShareSaleMore::default(),
            wage_inclusion: None,
            fixed_grant_price: crate::SourceCents::Unavailable,
        }
    }
}
