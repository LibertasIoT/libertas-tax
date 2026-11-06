use crate::common::Owner;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// When the ISO shares enter AMT income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IsoRecognition {
    /// On exercise
    /// Rights were transferable or no longer subject to substantial risk of forfeiture on exercise.
    Exercise,
    /// When restrictions ended, without an earlier election
    Vesting {
        /// First date rights became transferable or no longer subject to substantial risk of forfeiture
        /// Use the first qualifying event, not a later scheduled vesting date. No earlier AMT inclusion applies.
        #[libertas_date_only]
        date: u32,
        /// Fair market value per share on that date
        /// Ignore lapse restrictions in valuing the stock. This replaces the exercise-date value for these shares.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        market_price: i64,
    },
    /// Valid timely AMT election on transfer
    /// Restricted shares were included under a valid election made by the applicable 30-day deadline. Transfer occurred on exercise; the election remains effective. This input records an election already made, not a new election filing.
    Election,
}

/// Form 3921
/// Exercise of an incentive stock option. One for each Form 3921 you received.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into transferor, employee, corporation. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the Form 3921 exactly. Return the transferor's (employer's) name as printed. Boxes 1 through 5 must show values; report a blank one as an error. Return box 1 and box 2 dates, box 3 and box 4 per-share amounts, and the box 5 number of shares with up to six decimals. Return box 6's corporation name, or an empty string when box 6 is blank. Report illegible or conflicting values as errors. Do not infer sales of these shares, when they enter AMT income, the amount paid for the option, or other facts absent from the Form 3921. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form3921 {
    /// TRANSFEROR'S name, identification number and address
    #[libertas_llm_doc_field]
    pub transferor: crate::PrintedParty,
    /// EMPLOYEE'S name, identification number and address
    #[libertas_llm_doc_field]
    pub employee: crate::PrintedParty,
    /// Box 1 date option granted
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub granted: u32,
    /// Box 2 date option exercised
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub exercised: u32,
    /// Box 3 exercise price per share
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Box 4 fair market value per share on exercise date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Box 5 number of shares transferred
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Box 6 corporation whose stock was transferred
    /// Only when it is not the employer named above.
    #[libertas_llm_doc_field]
    pub corporation: crate::PrintedParty,
}
// Box 5 is a validated finite quantity (share_units), so equality is
// reflexive; the income document union needs Eq.
impl Eq for Form3921 {}

/// Form 3921
/// Exercise of an Incentive Stock Option Under Section 422(b).
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 3921 exactly as printed. Read CORRECTED, the TRANSFEROR and EMPLOYEE name, address and TIN blocks, and the account number, preserving masking, punctuation and leading zeros. Use empty text for absent identity and account fields. Read boxes 1 and 2 as printed dates; never invent missing dates. Read boxes 3 and 4 as per-share dollar amounts and box 5 as the number of shares, with up to six decimal places. Return zero for a clearly blank numeric box. Read all of box 6, including the corporation name, address and TIN; leave its identity fields empty when blank. Never infer household identity, share sales, AMT recognition, option cost or other facts absent from the form. Report illegible or conflicting printed values as errors. Multiple copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.transferor.federal_id,@.employee.federal_id,@.granted,@.exercised,@.shares,@.exercise_price,@.market_price,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form3921Paper {
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// TRANSFEROR'S name, street address, city or town, state or province, country, and ZIP or foreign postal code; TRANSFEROR'S TIN
    #[libertas_llm_doc_field]
    pub transferor: crate::PrintedParty,
    /// EMPLOYEE'S TIN; EMPLOYEE'S name; street address (including apt. no.); city or town, state or province, country, and ZIP or foreign postal code
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
    /// Box 3 Exercise price per share
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exercise_price: i64,
    /// Box 4 Fair market value per share on exercise date
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_price: i64,
    /// Box 5 No. of shares transferred
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_number(min = 0, step = 0.000001)]
    #[libertas_default(0)]
    pub shares: f64,
    /// Box 6 If other than TRANSFEROR, name, address, and TIN of corporation whose stock is being transferred
    #[libertas_llm_doc_field]
    pub corporation: crate::PrintedParty,
}
impl Eq for Form3921Paper {}

impl From<&Form3921Paper> for Form3921 {
    fn from(paper: &Form3921Paper) -> Self {
        let Form3921Paper {
            corrected: _, // Printed metadata; no calculation reads it.
            transferor,
            employee,
            account_number: _, // Printed metadata; no calculation reads it.
            granted,
            exercised,
            exercise_price,
            market_price,
            shares,
            corporation,
        } = paper;
        Self {
            transferor: transferor.clone(),
            employee: employee.clone(),
            granted: *granted,
            exercised: *exercised,
            exercise_price: *exercise_price,
            market_price: *market_price,
            shares: *shares,
            corporation: corporation.clone(),
        }
    }
}

/// Accepted Form 3921
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IsoExerciseSource {
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
    /// Employee
    pub owner: Owner,
    /// Form 3921 as printed
    pub form: Form3921Paper,
    /// About these shares
    /// [NullText]
    /// Share details not answered
    /// [SetText]
    /// Enter share details
    #[libertas_default($null)]
    pub more: Option<Form3921More>,
}

/// About these shares
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form3921More {
    /// Shares from this exercise sold or otherwise disposed of in 2026
    /// Report the sale itself with its 1099-B. Shares sold in the year you exercised have no AMT adjustment.
    #[libertas_number(min = 0, step = 0.000001)]
    #[libertas_default(0)]
    pub sold: f64,
    /// When the shares enter AMT income
    #[libertas_default(Exercise)]
    pub recognition: IsoRecognition,
    /// Amount paid for the option itself
    /// Separate from the exercise price. Enter zero if you paid nothing for the option.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub option_cost: i64,
}
impl Default for Form3921More {
    fn default() -> Self {
        Self {
            sold: Default::default(),
            recognition: IsoRecognition::Exercise,
            option_cost: Default::default(),
        }
    }
}
impl Eq for Form3921More {}
