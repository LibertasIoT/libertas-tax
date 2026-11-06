// Form 1099-SA as printed (Rev. April 2025). The stored paper; the internal `Sa1099` is derived from it.
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 1099-SA, Distributions From an HSA, Archer MSA, or Medicare Advantage MSA
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into trustee_payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-SA (Distributions From an HSA, Archer MSA, or Medicare Advantage MSA) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Read the CORRECTED checkbox into corrected. Read the account number as printed; leave it empty if absent. Read box 1 gross_distribution; a blank box 1 is an error. Read box 2 into earnings_on_excess_contributions and box 4 into fmv_on_date_of_death, returning null for a genuinely blank box and numeric zero only for a printed zero. Leave blank text empty, never the text 0. Read the printed box 3 number into distribution_code: 1 NormalDistributions, 2 ExcessContributions, 3 Disability, 4 DeathDistributionOtherThanCode6, 5 ProhibitedTransaction, 6 DeathDistributionAfterYearOfDeathToNonspouseBeneficiary; a blank, illegible or unlisted box 3 code is an error. For box 5 return Hsa, Archer or MedicareAdvantage for the checked box; report an error when none or more than one is checked. Never infer the signed custodian net-income adjustment or choose payment, medicare_prior or medicare_from_spouse. Do not report payment dates, what the money paid for, a rollover, a disability date, the death date, who the beneficiary is or the account's prior-year value; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.trustee_payer.federal_id,@.recipient.federal_id,@.gross_distribution,@.earnings_on_excess_contributions,@.fmv_on_date_of_death,@.account_number"
)]
pub struct Form1099SaPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// TRUSTEE'S/PAYER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and telephone number; PAYER'S TIN
    #[libertas_llm_doc_field]
    pub trustee_payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Account number
    /// May show an account or other unique number the payer assigned to distinguish your account.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: alloc::string::String,
    /// 1 Gross distribution
    /// Use one real statement. Report contributions and corrections on Health accounts; ordinary distributions are entered only here.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross_distribution: i64,
    /// 2 Earnings on excess cont.
    /// [NullText]
    /// Box 2 is blank
    /// [SetText]
    /// Box 2 earnings are
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub earnings_on_excess_contributions: Option<i64>,
    /// 3 Distribution code
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// Box 3 code is
    #[libertas_llm_doc_field]
    pub distribution_code: Option<Form1099SaDistributionCode>,
    /// 4 FMV on date of death
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// Box 4 value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub fmv_on_date_of_death: Option<i64>,
    /// 5 HSA, Archer MSA or MA MSA
    #[libertas_llm_doc_field]
    #[libertas_inline_choices]
    pub account: crate::MedicalSaAccount,
}

/// Form 1099-SA box 3 distribution code
// IRS Instructions for Forms 1099-SA and 5498-SA, Box 3:
// https://www.irs.gov/instructions/i1099sa
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1099SaDistributionCode {
    /// 1 — Normal distributions
    #[libertas_form_code("1")]
    NormalDistributions,
    /// 2 — Excess contributions
    #[libertas_form_code("2")]
    ExcessContributions,
    /// 3 — Disability
    #[libertas_form_code("3")]
    Disability,
    /// 4 — Death distribution other than code 6
    #[libertas_form_code("4")]
    DeathDistributionOtherThanCode6,
    /// 5 — Prohibited transaction
    #[libertas_form_code("5")]
    ProhibitedTransaction,
    /// 6 — Death distribution after year of death to a nonspouse beneficiary
    #[libertas_form_code("6")]
    DeathDistributionAfterYearOfDeathToNonspouseBeneficiary,
}

impl Form1099SaDistributionCode {
    pub(crate) const fn printed_code(self) -> u32 {
        match self {
            Self::NormalDistributions => 1,
            Self::ExcessContributions => 2,
            Self::Disability => 3,
            Self::DeathDistributionOtherThanCode6 => 4,
            Self::ProhibitedTransaction => 5,
            Self::DeathDistributionAfterYearOfDeathToNonspouseBeneficiary => 6,
        }
    }
}
