// Printed Form 1095-A fields. Source: https://www.irs.gov/pub/irs-pdf/f1095a.pdf
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 1095-A, Health Insurance Marketplace Statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1095-A exactly as printed. Check the calendar year against the supplied year and report a mismatch. Read including its year, VOID and CORRECTED boxes, Part I lines 1 through 15, Part II covered individuals and Part III lines 21 through 33. Copy names, identification numbers and addresses exactly, preserving masking, punctuation and leading zeros. Put lines 4, 5 and 12 through 15 in recipient, each in its own box: name (4), ssn (5), street_address (12), city (13), state (14) and country_zip (15). Copy each box as printed, with no added punctuation or postal formatting. Leave unprinted identity text empty. Do not infer household identities, tax families, eligibility, allocations or payment facts. Each monthly line is null only when all three columns are blank. Within an entered row, clearly blank A, B and C are null, never numeric zero. Preserve a printed zero as numeric zero. Copy line 33 annual totals without calculating them. Illegible or conflicting entries are errors. Copies of the same form are one source.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.calendar_year,@.marketplace_identifier,@.policy_number,@.recipient.ssn,@.annual_totals.premium,@.annual_totals.benchmark,@.annual_totals.advance",
    form_date_order = "MonthDayYear"
)]
pub struct Form1095APaper {
    /// Calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// VOID
    #[libertas_default(false)]
    #[libertas_llm_doc_field(blank = false)]
    pub void: bool,
    /// CORRECTED
    #[libertas_default(false)]
    #[libertas_llm_doc_field(blank = false)]
    pub corrected: bool,
    /// 1 Marketplace identifier
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub marketplace_identifier: String,
    /// 2 Marketplace-assigned policy number
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub policy_number: String,
    /// 3 Policy issuer's name
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub issuer: String,
    /// Recipient
    /// Lines 4, 5 and 12 through 15.
    #[libertas_llm_doc_field]
    pub recipient: Form1095ARecipient,
    /// 6 Recipient's date of birth
    /// [NullText]
    /// No date of birth printed
    /// [SetText]
    /// Printed date of birth
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub recipient_date_of_birth: Option<u32>,
    /// 7 Recipient's spouse's name
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub spouse_name: String,
    /// 8 Recipient's spouse's SSN
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub spouse_ssn: String,
    /// 9 Recipient's spouse's date of birth
    /// [NullText]
    /// No date of birth printed
    /// [SetText]
    /// Printed date of birth
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub spouse_date_of_birth: Option<u32>,
    /// 10 Policy start date
    /// [NullText]
    /// No policy start date printed
    /// [SetText]
    /// Printed policy start date
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub policy_start_date: Option<u32>,
    /// 11 Policy termination date
    /// [NullText]
    /// No policy termination date printed
    /// [SetText]
    /// Printed policy termination date
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub policy_termination_date: Option<u32>,
    /// Part II, lines 16–20 Covered Individuals
    #[libertas_llm_doc_field]
    pub covered_individuals: Vec<Form1095ACoveredIndividual>,
    /// 21 January
    /// [NullText]
    /// All three January columns are blank
    /// [SetText]
    /// Printed January amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub january: Option<Form1095ACoverageAmounts>,
    /// 22 February
    /// [NullText]
    /// All three February columns are blank
    /// [SetText]
    /// Printed February amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub february: Option<Form1095ACoverageAmounts>,
    /// 23 March
    /// [NullText]
    /// All three March columns are blank
    /// [SetText]
    /// Printed March amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub march: Option<Form1095ACoverageAmounts>,
    /// 24 April
    /// [NullText]
    /// All three April columns are blank
    /// [SetText]
    /// Printed April amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub april: Option<Form1095ACoverageAmounts>,
    /// 25 May
    /// [NullText]
    /// All three May columns are blank
    /// [SetText]
    /// Printed May amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub may: Option<Form1095ACoverageAmounts>,
    /// 26 June
    /// [NullText]
    /// All three June columns are blank
    /// [SetText]
    /// Printed June amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub june: Option<Form1095ACoverageAmounts>,
    /// 27 July
    /// [NullText]
    /// All three July columns are blank
    /// [SetText]
    /// Printed July amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub july: Option<Form1095ACoverageAmounts>,
    /// 28 August
    /// [NullText]
    /// All three August columns are blank
    /// [SetText]
    /// Printed August amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub august: Option<Form1095ACoverageAmounts>,
    /// 29 September
    /// [NullText]
    /// All three September columns are blank
    /// [SetText]
    /// Printed September amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub september: Option<Form1095ACoverageAmounts>,
    /// 30 October
    /// [NullText]
    /// All three October columns are blank
    /// [SetText]
    /// Printed October amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub october: Option<Form1095ACoverageAmounts>,
    /// 31 November
    /// [NullText]
    /// All three November columns are blank
    /// [SetText]
    /// Printed November amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub november: Option<Form1095ACoverageAmounts>,
    /// 32 December
    /// [NullText]
    /// All three December columns are blank
    /// [SetText]
    /// Printed December amounts
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub december: Option<Form1095ACoverageAmounts>,
    /// 33 Annual Totals
    #[libertas_llm_doc_field]
    pub annual_totals: Form1095ACoverageAmounts,
}

/// Recipient
/// Lines 4, 5 and 12 through 15 of the printed form.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1095ARecipient {
    /// 4 Recipient's name
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub name: String,
    /// 5 Recipient's SSN
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub ssn: String,
    /// 12 Street address (including apartment no.)
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub street_address: String,
    /// 13 City or town
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub city: String,
    /// 14 State or province
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub state: String,
    /// 15 Country and ZIP or foreign postal code
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub country_zip: String,
}

/// Part II Covered Individual
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1095ACoveredIndividual {
    /// A. Covered individual name
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub name: String,
    /// B. Covered individual SSN
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub ssn: String,
    /// C. Covered individual date of birth
    /// [NullText]
    /// No date of birth printed
    /// [SetText]
    /// Printed date of birth
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub date_of_birth: Option<u32>,
    /// D. Coverage start date
    /// [NullText]
    /// No coverage start date printed
    /// [SetText]
    /// Printed coverage start date
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub coverage_start_date: Option<u32>,
    /// E. Coverage termination date
    /// [NullText]
    /// No coverage termination date printed
    /// [SetText]
    /// Printed coverage termination date
    #[libertas_date_only]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub coverage_termination_date: Option<u32>,
}

/// Part III Coverage Information
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1095ACoverageAmounts {
    /// A. Monthly enrollment premiums
    /// [NullText]
    /// A. Monthly enrollment premiums is blank
    /// [SetText]
    /// Printed a. monthly enrollment premiums
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub premium: Option<i64>,
    /// B. Monthly second lowest cost silver plan (SLCSP) premium
    /// [NullText]
    /// Column B is blank
    /// [SetText]
    /// Printed SLCSP premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub benchmark: Option<i64>,
    /// C. Monthly advance payment of premium tax credit
    /// [NullText]
    /// C. Monthly advance payment of premium tax credit is blank
    /// [SetText]
    /// Printed c. monthly advance payment of premium tax credit
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    #[libertas_llm_doc_field(blank = $null)]
    pub advance: Option<i64>,
}
