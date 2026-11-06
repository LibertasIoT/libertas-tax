// Schedule K-1 (Form 1120-S), 2026 draft created April 23, 2026.
// Final shareholder instructions are the 2025 edition; 2026 instructions remain a draft.
// https://www.irs.gov/pub/irs-dft/f1120ssk--dft.pdf
// https://www.irs.gov/instructions/i1120ssk
use crate::k1_page::K1Scorp;
use crate::partnership_source::{credit_code, credit_code_name};
use crate::validation::ValidationIssue;
use crate::*;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Beginning and ending numbers of shares, as printed.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1ShareCountPair {
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub beginning: Option<f64>,
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub ending: Option<f64>,
}

/// Schedule K-1 (Form 1120-S) — Shareholder's Share of Income, Deductions, Credits, etc.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1ScorpPaper {
    /// Entity reporting tax period
    /// [NullText]
    /// The source does not establish this information
    /// [SetText]
    /// The printed information is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub reporting_period: Option<K1EntityTaxPeriod>,
    /// Final K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_k1: bool,
    /// Amended K-1
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub amended_k1: bool,
    /// A and B — Corporation EIN, name and address
    #[libertas_llm_doc_field]
    pub corporation: PrintedParty,
    /// C — IRS Center where corporation filed return
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub irs_center: String,
    /// D — Corporation total number of shares
    #[libertas_llm_doc_field]
    pub corporation_shares: K1ShareCountPair,
    /// E and F1 — Shareholder identifying number, name and address
    #[libertas_llm_doc_field]
    pub shareholder: PrintedParty,
    /// F2 — Individual or entity responsible for reporting
    #[libertas_llm_doc_field]
    pub reporting_party: K1DisregardedEntity,
    /// F3 — Type of entity
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub shareholder_type: String,
    /// G — Current year allocation percentage
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    // Tax structure: an ownership allocation percentage is between zero and 100.
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    #[libertas_default($null)]
    pub allocation_percentage: Option<f64>,
    /// H — Shareholder number of shares
    #[libertas_llm_doc_field]
    pub shareholder_shares: K1ShareCountPair,
    /// I — Loans from shareholder, principal owed, never adjusted tax basis
    /// [NullText]
    /// The source does not establish this information
    /// [SetText]
    /// The printed information is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub loans: Option<K1MoneyPair>,
    /// 1 — Ordinary business income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub ordinary: Option<i64>,
    /// 2 — Net rental real estate income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub rental_real_estate: Option<i64>,
    /// 3 — Other net rental income (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub other_rental: Option<i64>,
    /// 4 — Interest income
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub interest: Option<i64>,
    /// 5a — Ordinary dividends
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub dividends: Option<i64>,
    /// 5b — Qualified dividends
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub qualified_dividends: Option<i64>,
    /// 6 — Royalties
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub royalties: Option<i64>,
    /// 7 — Net short-term capital gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub short: Option<i64>,
    /// 8a — Net long-term capital gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub long: Option<i64>,
    /// 8b — Collectibles (28%) gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub collectibles: Option<i64>,
    /// 8c — Unrecaptured section 1250 gain
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub unrecaptured: Option<i64>,
    /// 9 — Net section 1231 gain (loss)
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub section1231: Option<i64>,
    /// 10 — Other income (loss)
    #[libertas_llm_doc_field]
    pub income: Vec<K1CodeRow>,
    /// 11 — Section 179 deduction
    /// [NullText]
    /// This printed box is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section179: Option<i64>,
    /// 12 — Other deductions
    #[libertas_llm_doc_field]
    pub deductions: Vec<K1CodeRow>,
    /// 13 — Credits
    #[libertas_llm_doc_field]
    pub credits: Vec<K1CodeRow>,
    /// 14 — Schedule K-3 attached
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub k3_attached: bool,
    /// 15 — Alternative minimum tax (AMT) items
    #[libertas_llm_doc_field]
    pub amt: Vec<K1CodeRow>,
    /// 16 — Items affecting shareholder basis
    #[libertas_llm_doc_field]
    pub basis: Vec<K1CodeRow>,
    /// 17 — Other information
    #[libertas_llm_doc_field]
    pub information: Vec<K1CodeRow>,
    /// 18 — More than one activity for at-risk purposes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub multiple_at_risk: bool,
    /// 19 — More than one activity for passive activity purposes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub multiple_passive: bool,
}

/// Source-assigned business and portfolio property character
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ScorpPropertyStatement {
    /// Box 8c from selling another partnership interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Box 8c from estates, trusts, mutual funds or REITs
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Box 8c from the box 1 business
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_business: i64,
    /// Box 9 assigned to the box 1 business
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_business: i64,
}

/// Character supplied by a box 10 attached statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpIncomeStatement {
    #[libertas_llm_doc_field]
    pub code: ScorpIncomeCode,
    /// [NullText]
    /// The paper supplies the amount; this statement only supplies character
    /// [SetText]
    /// The statement separately reports
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub amount: Option<i64>,
}
impl From<&ScorpIncome> for ScorpIncomeStatement {
    fn from(item: &ScorpIncome) -> Self {
        Self { code: item.code, amount: Some(item.amount) }
    }
}

/// Actual statements supplied with this S-corporation K-1.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpStatements {
    /// Box 17 code V section 199A statement
    #[libertas_llm_doc_field]
    pub qbi: K1ScorpQbi,
    #[libertas_llm_doc_field]
    pub rentals: PartnershipRentalStatement,
    #[libertas_llm_doc_field]
    pub property: ScorpPropertyStatement,
    #[libertas_llm_doc_field]
    pub income: Vec<ScorpIncomeStatement>,
    #[libertas_llm_doc_field]
    pub credits: Vec<PartnershipCreditStatement>,
    #[libertas_llm_doc_field]
    pub investment_property: Vec<K1InvestmentProperty>,
    #[libertas_llm_doc_field]
    pub section179_sales: Vec<K1Section179Sale>,
    /// Private activity bond interest explicitly identified on the box 15 code F statement
    /// [NullText]
    /// No private activity bond interest is identified
    /// [SetText]
    /// The statement identifies private activity bond interest of
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub private_activity: Option<i64>,
    /// Attached Schedule K-3, using its existing supported source fields
    /// [NullText]
    /// No Schedule K-3 is entered
    /// [SetText]
    /// The attached Schedule K-3 is
    #[libertas_default($null)]
    pub schedule_k3: Option<crate::ScheduleK3Paper>,
    /// Other situations explicitly identified by the actual statements
    /// Only situations an attached statement's own text explicitly identifies; never inferred from a box 17 code row or a STMT reference; empty when no attached statement identifies one.
    #[libertas_llm_doc_field]
    pub situations: Vec<ScorpK1Item>,
}
impl Default for ScorpStatements {
    fn default() -> Self {
        Self {
            qbi: K1ScorpQbi::NoStatement,
            rentals: Default::default(),
            property: Default::default(),
            income: Vec::new(),
            credits: Vec::new(),
            investment_property: Vec::new(),
            section179_sales: Vec::new(),
            private_activity: None,
            schedule_k3: None,
            situations: Vec::new(),
        }
    }
}

/// Independent shareholder answers; document import never supplies these records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpPersonalRecords {
    pub participation: Option<Participation>,
    pub basis: K1ScorpBasis,
    pub amt: Option<ScorpAmtRecords>,
    /// Actual same-paper activity allocation of printed code C.
    #[libertas_default($null)]
    pub depletion_sources: Option<Vec<crate::K1DepletionSourceAllocation>>,
    pub disposition: Option<ScorpDisposition>,
    pub special: Vec<ScorpSpecial>,
    pub qbi_group: String,
    pub qbi_records: K1ScorpQbiRecords,
    pub stock_term: K1StockTerm,
    pub trading: bool,
    pub rental_participation: Option<K1RentalParticipation>,
    pub rental_carryover_active: bool,
    pub rental_disposed: Vec<K1RentalActivity>,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub section179_carryover: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub section179_carryover_sold: i64,
    pub amt_section179_income_carry: Option<Section179IncomeCarryHistory>,
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    pub housing_confirmed: Option<K1HousingAcquisition>,
    pub credit_origins: Vec<PartnerCreditPersonal>,
    pub investment_forms: Vec<PartnerInvestmentPersonal>,
    pub k3_withholding: Vec<K3Withholding>,
    pub research_credit_line: Option<BusinessCredit>,
    pub health_insurance: BusinessHealthInsurance,
    pub exact: ScorpRecordFacts,
}

/// Accepted printed source and actual attachments.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpSource {
    pub paper: K1ScorpPaper,
    pub statements: ScorpStatements,
    pub personal: ScorpPersonalRecords,
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
}

/// Schedule K-1 (Form 1120-S) and its actual attached statements.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the actual Schedule K-1 (Form 1120-S) and supplied attached statements other than Schedule K-3. The attached Schedule K-3 has its own header and section import actions; preserve it and import its printed fields there. The 2026 paper follows the 2026 draft; current final Shareholder Instructions are the 2025 edition and the 2026 instructions are a separate draft. Retain each printed box and coded row separately, including printed signs, codes, asterisks, STMT and statement references. Part III prints two columns of boxes on shared lines; each amount belongs only to the box whose number labels its column and is printed beneath that box's caption. A coded box has a row only where a code letter is printed in it; with no printed code the box is an empty list. Box 11 section 179 deduction is the amount printed beneath its caption, read as printed. Every blank printed numeric box is null, including amounts, share counts, allocation percentage and loan principal; preserve a printed zero as zero. Every empty coded box is an empty list; never create placeholder rows or infer an amount from another box or a tax calculation. A printed coded row with no numeric amount has amount null. Read reporting_period from the actual entity beginning and ending dates printed on the K-1 and supplied statements, retaining those dates unchanged. Its actual ending must be in 2026. The printed form year or revision can be 2025 for a fiscal year beginning in 2025 and ending in 2026; do not reject that paper because its printed year differs from the individual return year, or replace its beginning date with January 1. A period of 10/01/2025 through 09/30/2026 belongs to the 2026 return. Reject an actual ending outside 2026 or conflicting actual dates; never infer missing dates or rewrite a printed year. Copy every party's name, TIN and address exactly, preserving masking and leading zeros. Item I is direct loan principal, never tax basis or a basis worksheet. Never infer advances, loan basis, household ownership, or personal answers. Read statement allocations only when actually supplied, including separate section 1231 and unrecaptured gains for the business and rental activities. Do not infer allocations from totals. Code M Research never supplies its Form 3800 line or eligible-small-business treatment. Do not import personal, completed owner forms, AMT worksheets, accepted child identities or personal K-3 withholding classification. The box 15 code F statement may explicitly identify private activity bond interest; retain other AMT items separately. statements.situations lists only a situation that a statement actually attached to the K-1 explicitly identifies in its own text; never infer one from a coded box row, a code letter or a STMT reference. A box 17 row such as V STMT is that box 17 row only. With no attached statement identifying a situation, statements.situations is an empty list. Report illegible or conflicting facts as errors.",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    duplicate_identity = "@.form.corporation.federal_id,@.form.shareholder.federal_id,@.form.reporting_period.began,@.form.reporting_period.ended,@.form.ordinary,@.form.rental_real_estate,@.form.other_rental,@.form.interest,@.form.dividends,@.form.qualified_dividends,@.form.royalties,@.form.short,@.form.long,@.form.collectibles,@.form.unrecaptured,@.form.section1231,@.form.section179",
    form_date_order = "MonthDayYear"
)]
pub struct K1ScorpDocument {
    #[libertas_llm_doc_field]
    pub form: K1ScorpPaper,
    #[libertas_llm_doc_field]
    pub statements: ScorpStatements,
}

pub(crate) fn fresh_scorp_basis() -> crate::K1ScorpBasis {
    crate::K1ScorpBasis {
        start: crate::K1ScorpStart::Continuing {
            opening: None,
            risk_history: crate::ScorpRiskStart::Unknown,
            carried: crate::K1ScorpCarried {
                losses_first: false,
                basis_loss: 0,
                basis_nondeductible: 0,
                basis_short: 0,
                basis_long: 0,
                basis_collectibles: 0,
                risk_short: 0,
                risk_long: 0,
                risk_collectibles: 0,
                section1231_basis_loss: 0,
                section1231_risk_loss: 0,
                section1231_passive_loss: 0,
                risk_loss: 0,
                risk_nondeductible: 0,
                deducted: 0,
                recaptured: 0,
                passive_loss: 0,
                rental_real_estate_passive_loss: 0,
                other_rental_passive_loss: 0,
                rental_real_estate_section1231_passive_loss: 0,
                other_rental_section1231_passive_loss: 0,
                rental_real_estate_basis_loss: 0,
                rental_real_estate_risk_loss: 0,
                other_rental_basis_loss: 0,
                other_rental_risk_loss: 0,
                basis_section179: 0,
                basis_charity: 0,
                basis_foreign_taxes: 0,
            },
        },
        additions: 0,
        loans: Vec::new(),
        more_at_risk: crate::K1ScorpAtRisk {
            excluded_funding: 0,
            protected: 0,
            released: 0,
            excluded_returns: 0,
        },
    }
}

impl Default for ScorpPersonalRecords {
    fn default() -> Self {
        Self {
            participation: None,
            basis: fresh_scorp_basis(),
            amt: None,
            depletion_sources: None,
            disposition: None,
            special: Vec::new(),
            qbi_group: String::new(),
            qbi_records: K1ScorpQbiRecords {
                ordinary: K1ScorpLossHistory {
                    history: Vec::new(),
                    at_risk: Vec::new(),
                    passive: Vec::new(),
                },
                property: K1ScorpLossHistory {
                    history: Vec::new(),
                    at_risk: Vec::new(),
                    passive: Vec::new(),
                },
                property_complete: false,
            },
            stock_term: K1StockTerm::NotNeeded,
            trading: false,
            rental_participation: None,
            rental_carryover_active: true,
            rental_disposed: Vec::new(),
            section179_carryover: 0,
            section179_carryover_sold: 0,
            amt_section179_income_carry: None,
            investment_confirmed: None,
            housing_confirmed: None,
            credit_origins: Vec::new(),
            investment_forms: Vec::new(),
            k3_withholding: Vec::new(),
            research_credit_line: None,
            health_insurance: BusinessHealthInsurance::None,
            exact: ScorpRecordFacts::default(),
        }
    }
}
impl ScorpPersonalRecords {
    pub(crate) fn from_page(page: &K1Scorp) -> Self {
        Self {
            participation: page.participation.clone(),
            basis: page.basis.clone(),
            amt: page.amt.clone(),
            depletion_sources: None,
            disposition: page.disposition.clone(),
            special: page.special.clone(),
            qbi_group: page.qbi_group.clone(),
            qbi_records: page.qbi_records_record(),
            stock_term: page.more.stock_term,
            trading: page.more.trading,
            rental_participation: page.rentals.participation,
            rental_carryover_active: page.rentals.carryover_active,
            rental_disposed: page.rentals.disposed.clone(),
            section179_carryover: page.more.section179_carryover,
            section179_carryover_sold: page.more.section179_carryover_sold,
            amt_section179_income_carry: page.more.amt_section179_income_carry.clone(),
            investment_confirmed: page.more.investment_confirmed,
            housing_confirmed: page.more.housing_confirmed,
            credit_origins: page
                .more
                .credits
                .iter()
                .map(|credit| {
                    let credit: K1Credit = credit.clone().into();
                    PartnerCreditPersonal {
                        code: credit.code,
                        line: credit.line,
                        carbon_oxide_origin: credit.carbon_oxide_origin,
                        renewable_facility_origin: credit.renewable_facility_origin,
                    }
                })
                .collect(),
            investment_forms: page
                .more
                .investment_property
                .iter()
                .filter_map(|p| {
                    p.own_form().map(|f| PartnerInvestmentPersonal {
                        id: p.id(),
                        own_form: Some(f.clone()),
                    })
                })
                .collect(),
            k3_withholding: match &page.schedule_k3 {
                K3Attached::NotAttached => Vec::new(),
                K3Attached::Attached(k3) => k3.withholding.clone(),
            },
            research_credit_line: page.more.credits.iter().find_map(|c| {
                let c: K1Credit = c.clone().into();
                (c.code == K1CreditCode::Research).then_some(c.line).flatten()
            }),
            health_insurance: BusinessHealthInsurance::None,
            exact: page.exact.clone(),
        }
    }
    #[cfg(test)]
    pub(crate) fn from_calculation(record: &ScorpK1) -> Option<Self> {
        let mut personal = Self::from_page(&K1Scorp::from_record("", record)?);
        personal.health_insurance = record.health_insurance.clone();
        personal.research_credit_line = record.research_credit_line;
        Some(personal)
    }
}
fn code_amount(rows: &[K1CodeRow], code: &str) -> i64 {
    rows.iter()
        .filter(|row| row.code == code)
        .filter_map(|row| row.amount)
        .try_fold(0i64, |sum, amount| sum.checked_add(amount))
        .expect("source total admitted")
}
fn raw_code_amount(rows: &[K1CodeRow], code: &str) -> Option<i64> {
    let values: Vec<_> = rows.iter().filter(|row| row.code == code).collect();
    if values.is_empty() || values.iter().any(|row| row.amount.is_none()) {
        return None;
    }
    values.into_iter().try_fold(0i64, |sum, row| sum.checked_add(row.amount?))
}
fn has_code(rows: &[K1CodeRow], code: &str) -> bool {
    rows.iter().any(|row| row.code == code)
}
fn push_once<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn ordinary_depletion_adjustment(
    paper: &K1ScorpPaper,
    statements: &ScorpStatements,
) -> Option<i64> {
    if !crate::k1_depletion_sources::has_depletion(&paper.amt) {
        return Some(0);
    }
    if paper.rental_real_estate.is_some_and(|amount| amount != 0)
        || paper.other_rental.is_some_and(|amount| amount != 0)
        || paper.royalties.is_some_and(|amount| amount != 0)
        || paper.section1231.is_some_and(|amount| amount != 0)
        || !statements.investment_property.is_empty()
    {
        return None;
    }
    crate::k1_depletion_sources::printed_depletion(&paper.amt)
}

fn calculation_page(
    form: &K1ScorpPaper,
    statements: &ScorpStatements,
    personal: &ScorpPersonalRecords,
) -> K1Scorp {
    // Printed metadata remains on the source. Aggregate property boxes are
    // allocated only by the actual statements; item I never supplies tax basis.
    let K1ScorpPaper {
        reporting_period: paper_reporting_period,
        final_k1: paper_final_k1,
        amended_k1: _,
        corporation: paper_corporation,
        irs_center: _,
        corporation_shares: _,
        shareholder: paper_shareholder,
        reporting_party: _,
        shareholder_type: _,
        allocation_percentage: _,
        shareholder_shares: _,
        loans: _,
        ordinary: paper_ordinary,
        rental_real_estate: paper_rental_real_estate,
        other_rental: paper_other_rental,
        interest: paper_interest,
        dividends: paper_dividends,
        qualified_dividends: paper_qualified_dividends,
        royalties: paper_royalties,
        short: paper_short,
        long: paper_long,
        collectibles: paper_collectibles,
        unrecaptured: _,
        section1231: _,
        income: paper_income,
        section179: paper_section179,
        deductions: paper_deductions,
        credits: paper_credits,
        k3_attached: _,
        amt: paper_amt,
        basis: paper_basis,
        information: paper_information,
        multiple_at_risk: paper_multiple_at_risk,
        multiple_passive: paper_multiple_passive,
    } = form;
    let ScorpStatements {
        qbi: statement_qbi,
        rentals: statement_rentals,
        property: statement_property,
        income: statement_income,
        credits: statement_credits,
        investment_property: statement_investment_property,
        section179_sales: statement_section179_sales,
        private_activity: statement_private_activity,
        schedule_k3: statement_schedule_k3,
        situations: statement_situations,
    } = statements;
    let ScorpPersonalRecords {
        participation: personal_participation,
        basis: personal_basis,
        amt: personal_amt,
        depletion_sources: _,
        disposition: personal_disposition,
        special: personal_special,
        qbi_group: personal_qbi_group,
        qbi_records: personal_qbi_records,
        stock_term: personal_stock_term,
        trading: personal_trading,
        rental_participation: personal_rental_participation,
        rental_carryover_active: personal_rental_carryover_active,
        rental_disposed: personal_rental_disposed,
        section179_carryover: personal_section179_carryover,
        section179_carryover_sold: personal_section179_carryover_sold,
        amt_section179_income_carry: personal_amt_section179_income_carry,
        investment_confirmed: personal_investment_confirmed,
        housing_confirmed: personal_housing_confirmed,
        credit_origins: personal_credit_origins,
        investment_forms: personal_investment_forms,
        k3_withholding: personal_k3_withholding,
        research_credit_line: personal_research_credit_line,
        health_insurance: _,
        exact: personal_exact,
    } = personal;
    let mut retained_research_lines = personal_credit_origins
        .iter()
        .filter(|answer| answer.code == K1CreditCode::Research)
        .map(|answer| answer.line);
    let mut credits: Vec<K1CreditPage> = statement_credits
        .iter()
        .filter_map(|statement| {
            let line = if statement.code == K1CreditCode::Research {
                // An explicit answer overrides retained per-credit character.
                // Otherwise preserve each original statement's personal answer.
                (*personal_research_credit_line)
                    .or_else(|| retained_research_lines.next().flatten())
            } else {
                statement.line
            };
            if statement.code.needs_line()
                && !line.is_some_and(|line| statement.code.line_allows(line))
            {
                return None;
            }
            let amount = statement
                .amount
                .or_else(|| raw_code_amount(paper_credits, credit_code_name(&statement.code)))?;
            if amount <= 0 {
                return None;
            }
            let answer = personal_credit_origins
                .iter()
                .find(|answer| answer.code == statement.code && answer.line == line);
            Some(
                K1Credit {
                    code: statement.code,
                    line,
                    amount,
                    carbon_oxide_origin: answer
                        .and_then(|answer| answer.carbon_oxide_origin.clone()),
                    renewable_facility_origin: answer
                        .and_then(|answer| answer.renewable_facility_origin.clone()),
                }
                .into(),
            )
        })
        .collect();
    // A credit without statement-specific line character uses its existing
    // code owner. Statement-dependent codes retain their source boundary.
    for row in paper_credits {
        if let Some(code) = credit_code(&row.code) {
            if statement_credits.iter().any(|credit| credit.code == code) {
                continue;
            }
            if matches!(
                code,
                K1CreditCode::OtherRentalRealEstate
                    | K1CreditCode::OtherRental
                    | K1CreditCode::RenewableElectricity
                    | K1CreditCode::Transferred
                    | K1CreditCode::Other
            ) {
                continue;
            }
            let line = if code == K1CreditCode::Research {
                let Some(line) =
                    personal_research_credit_line.filter(|line| code.line_allows(*line))
                else {
                    continue;
                };
                Some(line)
            } else {
                None
            };
            if let Some(amount) = row.amount.filter(|amount| *amount > 0) {
                let answer = personal_credit_origins
                    .iter()
                    .find(|answer| answer.code == code && answer.line == line);
                credits.push(
                    K1Credit {
                        code,
                        line,
                        amount,
                        carbon_oxide_origin: answer
                            .and_then(|answer| answer.carbon_oxide_origin.clone()),
                        renewable_facility_origin: answer
                            .and_then(|answer| answer.renewable_facility_origin.clone()),
                    }
                    .into(),
                );
            }
        }
    }
    let mut properties = statement_investment_property.clone();
    for property in &mut properties {
        *property.own_form_mut() = personal_investment_forms
            .iter()
            .find(|answer| answer.id > 0 && answer.id == property.id())
            .and_then(|answer| answer.own_form.clone());
    }

    let mut income: Vec<ScorpIncome> = statement_income
        .iter()
        .filter_map(|detail| {
            Some(ScorpIncome {
                code: detail.code,
                amount: detail
                    .amount
                    .or_else(|| raw_code_amount(paper_income, income_code(detail.code)))?,
            })
        })
        .collect();
    for row in paper_income {
        if income.iter().any(|item| income_code(item.code) == row.code) {
            continue;
        }
        let code = match row.code.as_str() {
            "A" if row.amount.is_some_and(|a| a >= 0) => Some(ScorpIncomeCode::PortfolioIncome),
            "C" => Some(ScorpIncomeCode::Section1256),
            "J" => Some(ScorpIncomeCode::TaxBenefitRecovery),
            _ => None,
        };
        if let (Some(code), Some(amount)) = (code, row.amount) {
            income.push(ScorpIncome { code, amount });
        }
    }
    let deductions = paper_deductions
        .iter()
        .filter_map(|row| {
            Some(ScorpDeduction { code: deduction_code(&row.code)?, amount: row.amount? })
        })
        .collect();
    let mut unentered = statement_situations.clone();
    for row in paper_income {
        if !income.iter().any(|item| income_code(item.code) == row.code) {
            push_once(&mut unentered, ScorpK1Item::OtherIncome);
        }
    }
    for row in paper_credits {
        if row.code == "AX" {
            push_once(&mut unentered, ScorpK1Item::CreditRecapture);
        } else if row.code != "O"
            && row.code != "E"
            // A printed zero credit leaves nothing out of the estimate.
            && row.amount != Some(0)
            && !(row.code == "M" && row.amount.is_none_or(|a| a >= 0))
            && !credits
                .iter()
                .any(|c| credit_code_name(&K1Credit::from(c.clone()).code) == row.code)
        {
            push_once(&mut unentered, ScorpK1Item::Credits);
        }
    }
    for row in paper_amt {
        if !(matches!(row.code.as_str(), "A" | "B")
            || row.code == "C"
                && (matches!(personal_amt, Some(ScorpAmtRecords::Separate { .. }))
                    || ordinary_depletion_adjustment(form, statements).is_some())
            || row.code == "F"
                && statement_private_activity
                    .is_some_and(|amount| row.amount.is_none_or(|printed| printed == amount))
                && !statement_situations.contains(&ScorpK1Item::AmtItems))
        {
            push_once(&mut unentered, ScorpK1Item::AmtItems);
        }
    }
    for row in paper_information {
        match row.code.as_str() {
            "R" => push_once(&mut unentered, ScorpK1Item::Depletion),
            "A" | "B" if row.amount.is_none_or(|a| a != 0) => {
                push_once(&mut unentered, ScorpK1Item::OtherInformation)
            }
            "A" | "B" | "C" | "D" | "K" | "V" | "AV" => {}
            "U" | "AA" | "AB" | "AC" | "AJ" | "AN" | "AW" => {}
            _ => push_once(&mut unentered, ScorpK1Item::OtherInformation),
        }
    }
    if *paper_multiple_at_risk || *paper_multiple_passive {
        push_once(&mut unentered, ScorpK1Item::MultipleActivities);
    }
    let rentals = K1RentalBoxes {
        real_estate: paper_rental_real_estate.unwrap_or(0),
        other: paper_other_rental.unwrap_or(0),
        real_estate_section1231: statement_rentals.real_estate_section1231,
        real_estate_unrecaptured: statement_rentals.real_estate_unrecaptured,
        other_section1231: statement_rentals.other_section1231,
        other_unrecaptured: statement_rentals.other_unrecaptured,
        participation: *personal_rental_participation,
        real_estate_qbi: statement_rentals.real_estate_qbi.clone(),
        other_qbi: statement_rentals.other_qbi.clone(),
        carryover_active: *personal_rental_carryover_active,
        disposed: personal_rental_disposed.clone(),
    };
    let k3 = statement_schedule_k3
        .as_ref()
        .map(|source| source.native(personal_k3_withholding).pruned());
    let more = K1ScorpOtherBoxes {
        calls: K1ScorpCalls::from_credits(
            &credits.iter().map(K1CreditCallBox::from).collect::<Vec<_>>(),
        ),
        short: paper_short.unwrap_or(0),
        long: paper_long.unwrap_or(0),
        collectibles: paper_collectibles.unwrap_or(0),
        unrecaptured_partnership: statement_property.unrecaptured_partnership,
        unrecaptured_fund: statement_property.unrecaptured_fund,
        unrecaptured_business: statement_property.unrecaptured_business,
        section1231: statement_property.section1231_business,
        section179_sales: statement_section179_sales.clone(),
        section179: paper_section179.unwrap_or(0),
        section179_carryover: *personal_section179_carryover,
        section179_carryover_sold: *personal_section179_carryover_sold,
        cash_charity: code_amount(paper_deductions, "A"),
        income,
        deductions,
        withholding: code_amount(paper_credits, "O"),
        credits,
        investment_property: properties,
        allowed_investment_confirmations: crate::k1_page::investment_choices(
            statement_investment_property,
        ),
        investment_confirmed: *personal_investment_confirmed,
        housing_confirmed: *personal_housing_confirmed,
        private_activity: statement_private_activity.unwrap_or(0),
        exempt_interest: code_amount(paper_basis, "A"),
        exempt: code_amount(paper_basis, "B"),
        nondeductible: code_amount(paper_basis, "C"),
        foreign_taxes: code_amount(paper_basis, "F"),
        stock_term: *personal_stock_term,
        trading: *personal_trading,
        unentered,
        amt_section179_income_carry: personal_amt_section179_income_carry.clone(),
    };
    K1Scorp {
        corporation: paper_corporation.clone(),
        shareholder: paper_shareholder.clone(),
        reporting_period: *paper_reporting_period,
        final_k1: *paper_final_k1,
        ordinary: paper_ordinary.unwrap_or(0),
        rentals,
        interest: paper_interest.unwrap_or(0),
        dividends: paper_dividends.unwrap_or(0),
        qualified_dividends: paper_qualified_dividends.unwrap_or(0),
        royalties: paper_royalties.unwrap_or(0),
        distributions: code_amount(paper_basis, "D"),
        qbi: statement_qbi.clone(),
        qbi_records: Some(personal_qbi_records.clone()),
        participation: personal_participation.clone(),
        more,
        schedule_k3: k3.map_or(K3Attached::NotAttached, K3Attached::Attached),
        basis: personal_basis.clone(),
        amt: personal_amt.clone(),
        disposition: personal_disposition.clone(),
        special: personal_special.clone(),
        qbi_group: personal_qbi_group.clone(),
        exact: personal_exact.clone(),
    }
}

impl From<&ScorpSource> for ScorpK1 {
    fn from(source: &ScorpSource) -> Self {
        let ScorpSource {
            paper,
            statements,
            personal,
            assigned: _,
            owner_source: _,
            joint_share: _,
        } = source;
        let mut record = calculation_page(paper, statements, personal).record();
        record.health_insurance = personal.health_insurance.clone();
        record.research_credit_line = personal.research_credit_line;
        if matches!(personal.amt, Some(ScorpAmtRecords::SameAsRegular)) {
            record.amt = record.amt.and_then(|mut alternative| {
                alternative.adjustment =
                    alternative.adjustment.checked_add(source.amt_depletion_adjustment()?)?;
                Some(alternative)
            });
        }
        record.amt_record_choice = personal
            .amt
            .as_ref()
            .map(|answer| matches!(answer, ScorpAmtRecords::Separate { record: _ }));
        record
    }
}

impl ScorpSource {
    pub(crate) fn amt_depletion_adjustment(&self) -> Option<i64> {
        ordinary_depletion_adjustment(&self.paper, &self.statements)
    }
    pub(crate) fn printed(paper: K1ScorpPaper, mut statements: ScorpStatements) -> Self {
        for property in &mut statements.investment_property {
            *property.own_form_mut() = None;
        }
        for credit in &mut statements.credits {
            if credit.code == K1CreditCode::Research {
                credit.line = None;
            }
        }
        Self {
            paper,
            statements,
            personal: ScorpPersonalRecords::default(),
            assigned: crate::DocumentRecipient::Unknown,
            owner_source: None,
            joint_share: None,
        }
    }
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.personal = accepted.personal.clone();
        let mut kept = crate::document_names::Assignment::default();
        if self.paper.shareholder.name == accepted.paper.shareholder.name {
            kept = accepted.printed_names().assignment();
        }
        kept.apply_to(&mut self.assigned, &mut self.owner_source, &mut self.joint_share);
    }
    pub(crate) fn page(&self) -> K1Scorp {
        calculation_page(&self.paper, &self.statements, &self.personal)
    }
    pub(crate) fn replace_personal_page(&mut self, page: &K1Scorp) {
        let mut personal = ScorpPersonalRecords::from_page(page);
        personal.depletion_sources = self.personal.depletion_sources.clone();
        personal.health_insurance = self.personal.health_insurance.clone();
        personal.research_credit_line = self.personal.research_credit_line;
        self.personal = personal;
    }
    pub(crate) fn printed_names(&self) -> PrintedDocumentNames {
        PrintedDocumentNames {
            recipient: self.paper.shareholder.clone(),
            issuer: Some(self.paper.corporation.clone()),
            assigned: self.assigned,
            owner_source: self.owner_source,
            joint_share: self.joint_share,
            ..Default::default()
        }
    }
}

fn deduction_code(code: &str) -> Option<ScorpDeductionCode> {
    Some(match code {
        "B" => ScorpDeductionCode::CashThirty,
        "C" => ScorpDeductionCode::NoncashFifty,
        "D" => ScorpDeductionCode::NoncashThirty,
        "E" => ScorpDeductionCode::CapitalGainPropertyThirty,
        "F" => ScorpDeductionCode::CapitalGainPropertyTwenty,
        "G" => ScorpDeductionCode::ContributionsHundred,
        "H" => ScorpDeductionCode::InvestmentInterest,
        "I" => ScorpDeductionCode::RoyaltyDeductions,
        "J" => ScorpDeductionCode::Section59e,
        "L" => ScorpDeductionCode::PortfolioOther,
        "M" => ScorpDeductionCode::Preproductive,
        "O" => ScorpDeductionCode::Reforestation,
        "W" => ScorpDeductionCode::SoilWater,
        "X" => ScorpDeductionCode::Production,
        "Y" => ScorpDeductionCode::Barriers,
        "Z" => ScorpDeductionCode::Itemized,
        "AA" => ScorpDeductionCode::ConstructionFund,
        "AB" => ScorpDeductionCode::EarlyWithdrawalPenalty,
        "AC" => ScorpDeductionCode::DistributionInterest,
        "ZZ" => ScorpDeductionCode::Other,
        _ => return None,
    })
}
pub(crate) fn income_code(code: ScorpIncomeCode) -> &'static str {
    match code {
        ScorpIncomeCode::PortfolioIncome => "A",
        ScorpIncomeCode::Section1256 => "C",
        ScorpIncomeCode::TaxBenefitRecovery => "J",
        ScorpIncomeCode::GamblingWinnings | ScorpIncomeCode::GamblingLosses => "K",
        ScorpIncomeCode::NonportfolioShort | ScorpIncomeCode::NonportfolioLong => "S",
    }
}
#[cfg(test)]
pub(crate) fn deduction_code_name(code: ScorpDeductionCode) -> &'static str {
    use ScorpDeductionCode::*;
    match code {
        CashThirty => "B",
        NoncashFifty => "C",
        NoncashThirty => "D",
        CapitalGainPropertyThirty => "E",
        CapitalGainPropertyTwenty => "F",
        ContributionsHundred => "G",
        InvestmentInterest => "H",
        RoyaltyDeductions => "I",
        Section59e => "J",
        PortfolioOther => "L",
        Preproductive => "M",
        Reforestation => "O",
        SoilWater => "W",
        Production => "X",
        Barriers => "Y",
        Itemized => "Z",
        ConstructionFund => "AA",
        EarlyWithdrawalPenalty => "AB",
        DistributionInterest => "AC",
        Other => "ZZ",
    }
}

impl K1ScorpDocument {
    pub(crate) fn research_credit_needed(&self) -> bool {
        self.form.credits.iter().any(|row| row.code == "M" && row.amount.is_none_or(|a| a > 0))
            || self.statements.credits.iter().any(|s| {
                s.code == K1CreditCode::Research
                    && s.amount
                        .or_else(|| raw_code_amount(&self.form.credits, "M"))
                        .is_none_or(|a| a > 0)
            })
    }
    pub(crate) fn source(&self) -> ScorpSource {
        ScorpSource::printed(self.form.clone(), self.statements.clone())
    }
    pub(crate) fn record(&self) -> ScorpSource {
        self.source()
    }
    pub(crate) fn from_source(source: &ScorpSource) -> Self {
        Self { form: source.paper.clone(), statements: source.statements.clone() }
    }
    pub(crate) fn printed_admission_issue(&self) -> Option<ValidationIssue> {
        crate::partnership_source::credit_statement_issue(
            self.statements
                .credits
                .iter()
                .enumerate()
                .filter(|(_, statement)| statement.code != K1CreditCode::Research),
        )
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, require_source: bool) -> Option<ValidationIssue> {
        let source = self.source();
        source
            .admission_issue()
            .or_else(|| require_source.then(|| source.required_issue()).flatten())
            .or_else(|| {
                source.page().issue_with_mode(false).map(|issue| self.source_issue_path(issue))
            })
    }
    fn source_issue_path(&self, issue: ValidationIssue) -> ValidationIssue {
        let mappings: &[(&[&'static str], &[&'static str])] = &[
            (&["ordinary"], &["form", "ordinary"]),
            (&["qualified_dividends"], &["form", "qualified_dividends"]),
            (&["more", "private_activity"], &["statements", "private_activity"]),
            (&["more", "income"], &["statements", "income"]),
            (&["more", "deductions"], &["form", "deductions"]),
            (&["more", "credits"], &["statements", "credits"]),
            (&["more", "investment_property"], &["statements", "investment_property"]),
            (&["more", "section179_sales"], &["statements", "section179_sales"]),
            (&["qbi"], &["statements", "qbi"]),
            (&["schedule_k3"], &["statements", "schedule_k3"]),
            (&["reporting_period"], &["form", "reporting_period"]),
        ];
        for (from, to) in mappings {
            if issue.has_field_prefix(from) {
                return issue.map_field_prefix(from, to);
            }
        }
        ValidationIssue::new(issue.key)
    }
}
impl ScorpSource {
    pub(crate) fn topic_issue(&self, require_answers: bool) -> Option<ValidationIssue> {
        self.admission_issue()
            .or_else(|| require_answers.then(|| self.required_issue()).flatten())
            .or_else(|| self.page().topic_issue(require_answers))
            .or_else(|| self.personal_issue())
    }
    pub(crate) fn admission_issue(&self) -> Option<ValidationIssue> {
        for (field, rows) in [
            ("income", &self.paper.income),
            ("deductions", &self.paper.deductions),
            ("credits", &self.paper.credits),
            ("amt", &self.paper.amt),
            ("basis", &self.paper.basis),
            ("information", &self.paper.information),
        ] {
            let mut totals = alloc::collections::BTreeMap::<&str, i64>::new();
            for (index, row) in rows.iter().enumerate() {
                if let Some(amount) = row.amount {
                    let total = totals.entry(row.code.as_str()).or_default();
                    let Some(sum) = total.checked_add(amount) else {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("amount")
                                .in_list(field, index)
                                .in_field("form"),
                        );
                    };
                    *total = sum;
                }
            }
        }
        if self.property_totals().is_none() || self.investment_totals().is_none() {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("property")
                    .in_field("statements"),
            );
        }
        if let Some(issue) = crate::partnership_source::credit_statement_issue(
            self.statements.credits.iter().enumerate(),
        ) {
            return Some(issue);
        }
        if let Some(k3) = &self.statements.schedule_k3
            && let Some(issue) = k3.admission_issue()
        {
            return Some(issue.in_field("schedule_k3").in_field("statements"));
        }
        None
    }
    fn property_totals(&self) -> Option<[i64; 2]> {
        let p = &self.statements.property;
        let r = &self.statements.rentals;
        Some([
            [
                p.unrecaptured_partnership,
                p.unrecaptured_fund,
                p.unrecaptured_business,
                r.real_estate_unrecaptured,
                r.other_unrecaptured,
            ]
            .into_iter()
            .try_fold(0i64, |sum, a| sum.checked_add(a))?,
            [p.section1231_business, r.real_estate_section1231, r.other_section1231]
                .into_iter()
                .try_fold(0i64, |sum, a| sum.checked_add(a))?,
        ])
    }
    fn investment_totals(&self) -> Option<[i64; 4]> {
        let mut totals = [0i64; 4];
        for p in &self.statements.investment_property {
            let (i, a) = match p {
                K1InvestmentProperty::RehabilitationRental { expenditures, .. } => {
                    (0, expenditures)
                }
                K1InvestmentProperty::RehabilitationOther { expenditures, .. } => (1, expenditures),
                K1InvestmentProperty::Energy { basis, .. } => (2, basis),
                K1InvestmentProperty::CleanElectricity { basis, .. } => (3, basis),
            };
            totals[i] = totals[i].checked_add(*a)?;
        }
        Some(totals)
    }
    pub(crate) fn required_issue(&self) -> Option<ValidationIssue> {
        let missing = || ValidationIssue::new("SCORP_SOURCE_STATEMENT_REQUIRED");
        let mismatch = || ValidationIssue::new("SCORP_SOURCE_STATEMENT_MISMATCH");
        let form = &self.paper;
        let statements = &self.statements;
        if has_code(&form.information, "V") && matches!(statements.qbi, K1ScorpQbi::NoStatement) {
            return Some(missing().at_field("qbi").in_field("statements"));
        }
        if form.k3_attached && statements.schedule_k3.is_none() {
            return Some(missing().at_field("schedule_k3").in_field("statements"));
        }
        for (index, detail) in statements.income.iter().enumerate() {
            if detail
                .amount
                .or_else(|| raw_code_amount(&form.income, income_code(detail.code)))
                .is_none()
            {
                return Some(
                    missing().at_field("amount").in_list("income", index).in_field("statements"),
                );
            }
        }
        for (index, detail) in statements.credits.iter().enumerate() {
            if detail.code != K1CreditCode::Research
                && detail.code.needs_line()
                && detail.line.is_none()
            {
                return Some(
                    missing().at_field("line").in_list("credits", index).in_field("statements"),
                );
            }
            if detail
                .amount
                .or_else(|| raw_code_amount(&form.credits, credit_code_name(&detail.code)))
                .is_none()
            {
                return Some(
                    missing().at_field("amount").in_list("credits", index).in_field("statements"),
                );
            }
        }
        for (field, rows) in [("deductions", &form.deductions), ("basis", &form.basis)] {
            for (index, row) in rows.iter().enumerate() {
                let supported = if field == "deductions" {
                    row.code == "A" || deduction_code(&row.code).is_some()
                } else {
                    matches!(row.code.as_str(), "A" | "B" | "C" | "D" | "E" | "F")
                };
                if !supported {
                    return Some(
                        ValidationIssue::new("SCORP_SOURCE_CODE_NOT_CALCULATED")
                            .at_field("code")
                            .in_list(field, index)
                            .in_field("form"),
                    );
                }
            }
        }
        let Some(totals) = self.investment_totals() else {
            return Some(missing().at_field("investment_property").in_field("statements"));
        };
        for (i, (rows, code)) in [
            (&form.credits, "E"),
            (&form.information, "C"),
            (&form.information, "D"),
            (&form.information, "AV"),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(printed) = raw_code_amount(rows, code)
                && printed != totals[i]
            {
                return Some(mismatch().at_field("investment_property").in_field("statements"));
            }
        }
        if self.property_totals()
            != Some([form.unrecaptured.unwrap_or(0), form.section1231.unwrap_or(0)])
        {
            return Some(mismatch().at_field("property").in_field("statements"));
        }
        for (field, rows) in
            [("deductions", &form.deductions), ("amt", &form.amt), ("basis", &form.basis)]
        {
            for (index, row) in rows.iter().enumerate() {
                if (field != "amt" || matches!(row.code.as_str(), "A" | "B"))
                    && row.amount.is_none()
                {
                    return Some(
                        missing().at_field("amount").in_list(field, index).in_field("form"),
                    );
                }
            }
        }
        for (index, row) in form.income.iter().enumerate() {
            let details: Vec<_> =
                statements.income.iter().filter(|d| income_code(d.code) == row.code).collect();
            if details.is_empty() {
                if matches!(row.code.as_str(), "A" | "C" | "J") && row.amount.is_none() {
                    return Some(
                        missing().at_field("amount").in_list("income", index).in_field("form"),
                    );
                }
                continue;
            }
            if details.len() > 1 && details.iter().any(|d| d.amount.is_none()) {
                return Some(missing().at_field("income").in_field("statements"));
            }
            let resolved: Option<Vec<i64>> = details
                .into_iter()
                .map(|d| d.amount.or_else(|| raw_code_amount(&form.income, &row.code)))
                .collect();
            let Some(resolved) = resolved else {
                return Some(missing().at_field("income").in_field("statements"));
            };
            if row.code != "K"
                && let Some(printed) = raw_code_amount(&form.income, &row.code)
                && resolved.into_iter().try_fold(0i64, |sum, a| sum.checked_add(a)) != Some(printed)
            {
                return Some(mismatch().at_field("income").in_field("statements"));
            }
        }
        for (index, row) in form.credits.iter().enumerate() {
            if row.code == "O" {
                if row.amount.is_none() {
                    return Some(
                        missing().at_field("amount").in_list("credits", index).in_field("form"),
                    );
                }
                continue;
            }
            if row.code == "E" {
                if !statements
                    .investment_property
                    .iter()
                    .any(|p| matches!(p, K1InvestmentProperty::RehabilitationRental { .. }))
                {
                    return Some(missing().at_field("investment_property").in_field("statements"));
                }
                continue;
            }
            let Some(code) = credit_code(&row.code) else { continue };
            let details: Vec<_> = statements.credits.iter().filter(|d| d.code == code).collect();
            if details.is_empty() && code != K1CreditCode::Research && code.needs_line() {
                return Some(missing().at_field("credits").in_field("statements"));
            }
            if details
                .iter()
                .any(|d| code != K1CreditCode::Research && code.needs_line() && d.line.is_none())
                || details.len() > 1 && details.iter().any(|d| d.amount.is_none())
            {
                return Some(missing().at_field("credits").in_field("statements"));
            }
            let resolved: Option<Vec<i64>> = details
                .into_iter()
                .map(|d| d.amount.or_else(|| raw_code_amount(&form.credits, &row.code)))
                .collect();
            if row.amount.is_none() && resolved.as_ref().is_none_or(|v| v.is_empty()) {
                return Some(
                    missing().at_field("amount").in_list("credits", index).in_field("form"),
                );
            }
            if let (Some(printed), Some(resolved)) =
                (raw_code_amount(&form.credits, &row.code), resolved)
                && !resolved.is_empty()
                && resolved.into_iter().try_fold(0i64, |sum, a| sum.checked_add(a)) != Some(printed)
            {
                return Some(mismatch().at_field("credits").in_field("statements"));
            }
        }
        for row in &form.information {
            let found = match row.code.as_str() {
                "C" => statements
                    .investment_property
                    .iter()
                    .any(|p| matches!(p, K1InvestmentProperty::RehabilitationOther { .. })),
                "D" => statements
                    .investment_property
                    .iter()
                    .any(|p| matches!(p, K1InvestmentProperty::Energy { .. })),
                "AV" => statements
                    .investment_property
                    .iter()
                    .any(|p| matches!(p, K1InvestmentProperty::CleanElectricity { .. })),
                "K" => {
                    !statements.section179_sales.is_empty()
                        || statements.situations.contains(&ScorpK1Item::OtherInformation)
                }
                _ => true,
            };
            if !found {
                return Some(missing().at_field("information").in_field("form"));
            }
        }
        None
    }
    pub(crate) fn personal_issue(&self) -> Option<ValidationIssue> {
        let personal = &self.personal;
        if matches!(personal.amt, Some(ScorpAmtRecords::SameAsRegular))
            && self
                .paper
                .amt
                .iter()
                .any(|r| matches!(r.code.as_str(), "A" | "B") && r.amount.is_some_and(|a| a != 0))
        {
            return Some(ValidationIssue::new("DOCUMENT_K1_AMT_RECORDS").at_field("amt"));
        }
        // Item I is direct debt principal, not the shareholder's adjusted tax basis.
        // Box 16E is gross actual repayments, before open-account annual netting.
        // https://www.irs.gov/instructions/i1120s (item I and line 16e).
        let loans = &personal.basis.loans;
        let opening = loans.iter().try_fold(0i64, |sum, l| sum.checked_add(l.principal));
        let repayments = loans.iter().try_fold(0i64, |sum, l| sum.checked_add(l.repayments));
        let printed_repaid = if has_code(&self.paper.basis, "E") {
            raw_code_amount(&self.paper.basis, "E")
        } else {
            Some(0)
        };
        if self
            .paper
            .loans
            .as_ref()
            .is_some_and(|printed| printed.beginning.is_some_and(|amount| opening != Some(amount)))
            || repayments != printed_repaid
        {
            return Some(
                ValidationIssue::new("SCORP_SOURCE_LOAN_MISMATCH")
                    .at_field("loans")
                    .in_field("basis"),
            );
        }
        if personal.amt.is_none()
            || matches!(personal.basis.start, K1ScorpStart::Continuing { opening: None, .. })
        {
            return None;
        }
        let record = ScorpK1::from(self);
        // A wholly excluded K-1 keeps its existing warning; its basis owner
        // cannot supply an ending principal for reconciliation.
        if !record.basis_complete() {
            return None;
        }
        if let Some(ending) = self.paper.loans.as_ref().and_then(|printed| printed.ending) {
            if loans.is_empty() && ending == 0 {
                return None;
            }
            match record.loan_source_totals() {
                Some([_, actual, _]) if actual != ending => {
                    return Some(
                        ValidationIssue::new("SCORP_SOURCE_LOAN_MISMATCH")
                            .at_field("loans")
                            .in_field("basis"),
                    );
                }
                None => {
                    return Some(
                        ValidationIssue::new("SCORP_SOURCE_LOAN_RECORDS_REQUIRED")
                            .at_field("loans")
                            .in_field("basis"),
                    );
                }
                _ => {}
            }
        } else if self.paper.loans.is_none()
            && (!loans.is_empty() || printed_repaid.is_some_and(|a| a != 0))
        {
            return Some(
                ValidationIssue::new("SCORP_SOURCE_LOAN_RECORDS_REQUIRED")
                    .at_field("loans")
                    .in_field("basis"),
            );
        }
        None
    }
}

/// Canonical source and independent personal records for one accepted document.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct StoredScorpDocument {
    pub id: i64,
    pub label: String,
    pub owner: Owner,
    pub community: CommunityShare,
    pub state_source: Option<IncomeStateSource>,
    pub refund_management: Option<RefundManagement>,
    pub source: ScorpSource,
}
// A working community projection retains the paper and independent answers.
// Only Share's existing owner-first division changes monetary facts.
impl ScorpSource {
    pub(crate) fn divided(&self, share: crate::community::Share) -> Self {
        use crate::community::Divide;
        fn rows(values: &mut [K1CodeRow], share: crate::community::Share, grouped: &[&str]) {
            let mut totals = alloc::collections::BTreeMap::<String, i64>::new();
            for row in values {
                if let Some(amount) = row.amount {
                    row.amount = Some(if grouped.contains(&row.code.as_str()) {
                        let total = totals.entry(row.code.clone()).or_default();
                        let before = share.filer(*total);
                        *total = total.checked_add(amount).expect("admitted source total");
                        share.filer(*total) - before
                    } else {
                        amount.divided(share)
                    });
                }
            }
        }
        let mut result = self.clone();
        result.paper.ordinary = self.paper.ordinary.divided(share);
        result.paper.rental_real_estate = self.paper.rental_real_estate.divided(share);
        result.paper.other_rental = self.paper.other_rental.divided(share);
        result.paper.interest = self.paper.interest.divided(share);
        result.paper.dividends = self.paper.dividends.divided(share);
        result.paper.qualified_dividends = self.paper.qualified_dividends.divided(share);
        result.paper.royalties = self.paper.royalties.divided(share);
        result.paper.short = self.paper.short.divided(share);
        result.paper.long = self.paper.long.divided(share);
        result.paper.collectibles = self.paper.collectibles.divided(share);
        result.paper.unrecaptured = self.paper.unrecaptured.divided(share);
        result.paper.section1231 = self.paper.section1231.divided(share);
        result.paper.section179 = self.paper.section179.divided(share);
        if let Some(loans) = &mut result.paper.loans {
            loans.beginning = loans.beginning.divided(share);
            loans.ending = loans.ending.divided(share);
        }
        let income_groups: Vec<_> = self
            .statements
            .income
            .iter()
            .filter(|detail| detail.amount.is_none())
            .map(|detail| income_code(detail.code))
            .collect();
        let mut credit_groups: Vec<_> = self
            .statements
            .credits
            .iter()
            .filter(|detail| detail.amount.is_none())
            .map(|detail| credit_code_name(&detail.code))
            .collect();
        credit_groups.push("O");
        rows(&mut result.paper.income, share, &income_groups);
        rows(&mut result.paper.deductions, share, &["A"]);
        rows(&mut result.paper.credits, share, &credit_groups);
        rows(&mut result.paper.amt, share, &[]);
        let basis_groups: Vec<_> = self.paper.basis.iter().map(|row| row.code.as_str()).collect();
        rows(&mut result.paper.basis, share, &basis_groups);
        rows(&mut result.paper.information, share, &[]);
        for item in &mut result.statements.income {
            item.amount = item.amount.divided(share);
        }
        for item in &mut result.statements.credits {
            item.amount = item.amount.divided(share);
        }
        result.statements.credits.retain(|item| {
            item.amount
                .or_else(|| raw_code_amount(&result.paper.credits, credit_code_name(&item.code)))
                != Some(0)
        });
        result.paper.credits.retain(|row| {
            row.amount != Some(0) || row.code == "O" || credit_code(&row.code).is_none()
        });
        result.statements.investment_property = self.statements.investment_property.divided(share);
        result.statements.investment_property.retain(|property| match property {
            K1InvestmentProperty::RehabilitationRental { expenditures, .. }
            | K1InvestmentProperty::RehabilitationOther { expenditures, .. } => *expenditures != 0,
            K1InvestmentProperty::Energy { basis, .. }
            | K1InvestmentProperty::CleanElectricity { basis, .. } => *basis != 0,
        });
        result.statements.section179_sales = self.statements.section179_sales.divided(share);
        result.statements.private_activity = self.statements.private_activity.divided(share);
        result.statements.property.unrecaptured_partnership =
            self.statements.property.unrecaptured_partnership.divided(share);
        result.statements.property.unrecaptured_fund =
            self.statements.property.unrecaptured_fund.divided(share);
        result.statements.property.unrecaptured_business =
            self.statements.property.unrecaptured_business.divided(share);
        result.statements.property.section1231_business =
            self.statements.property.section1231_business.divided(share);
        result.statements.rentals.real_estate_section1231 =
            self.statements.rentals.real_estate_section1231.divided(share);
        result.statements.rentals.real_estate_unrecaptured =
            self.statements.rentals.real_estate_unrecaptured.divided(share);
        result.statements.rentals.other_section1231 =
            self.statements.rentals.other_section1231.divided(share);
        result.statements.rentals.other_unrecaptured =
            self.statements.rentals.other_unrecaptured.divided(share);
        result.statements.rentals.real_estate_qbi =
            self.statements.rentals.real_estate_qbi.divided(share);
        result.statements.rentals.other_qbi = self.statements.rentals.other_qbi.divided(share);
        // The paper aggregate and each native property category have separate
        // rounding boundaries. Retain their established source reconciliation.
        if let (Some(before), Some(after)) = (self.property_totals(), result.property_totals()) {
            if Some(before[0]) == self.paper.unrecaptured {
                result.paper.unrecaptured = Some(after[0]);
            }
            if Some(before[1]) == self.paper.section1231 {
                result.paper.section1231 = Some(after[1]);
            }
        }
        if let K1ScorpQbi::Statement {
            income,
            wages,
            property,
            service: _,
            current_loss,
            property_qbi,
            reit_dividends,
            ptp_income,
        } = &mut result.statements.qbi
        {
            *income = income.divided(share);
            *wages = wages.divided(share);
            *property = property.divided(share);
            *current_loss = current_loss.divided(share);
            property_qbi.gain = property_qbi.gain.divided(share);
            property_qbi.qualified_1250 = property_qbi.qualified_1250.divided(share);
            property_qbi.current = property_qbi.current.divided(share);
            *reit_dividends = reit_dividends.divided(share);
            *ptp_income = ptp_income.divided(share);
        }
        if let Some(k3) = &mut result.statements.schedule_k3 {
            *k3 = k3.divided(share);
        }
        let personal = &mut result.personal;
        personal.basis.additions = personal.basis.additions.divided(share);
        personal.basis.loans = personal.basis.loans.divided(share);
        if let K1ScorpStart::Continuing { opening, risk_history, carried } =
            &mut personal.basis.start
        {
            *opening = opening.divided(share);
            *risk_history = risk_history.divided(share);
            carried.basis_loss = carried.basis_loss.divided(share);
            carried.basis_nondeductible = carried.basis_nondeductible.divided(share);
            carried.basis_short = carried.basis_short.divided(share);
            carried.basis_long = carried.basis_long.divided(share);
            carried.basis_collectibles = carried.basis_collectibles.divided(share);
            carried.risk_short = carried.risk_short.divided(share);
            carried.risk_long = carried.risk_long.divided(share);
            carried.risk_collectibles = carried.risk_collectibles.divided(share);
            carried.section1231_basis_loss = carried.section1231_basis_loss.divided(share);
            carried.section1231_risk_loss = carried.section1231_risk_loss.divided(share);
            carried.section1231_passive_loss = carried.section1231_passive_loss.divided(share);
            carried.risk_loss = carried.risk_loss.divided(share);
            carried.risk_nondeductible = carried.risk_nondeductible.divided(share);
            carried.deducted = carried.deducted.divided(share);
            carried.recaptured = carried.recaptured.divided(share);
            carried.passive_loss = carried.passive_loss.divided(share);
            carried.rental_real_estate_passive_loss =
                carried.rental_real_estate_passive_loss.divided(share);
            carried.other_rental_passive_loss = carried.other_rental_passive_loss.divided(share);
            carried.rental_real_estate_section1231_passive_loss =
                carried.rental_real_estate_section1231_passive_loss.divided(share);
            carried.other_rental_section1231_passive_loss =
                carried.other_rental_section1231_passive_loss.divided(share);
            carried.rental_real_estate_basis_loss =
                carried.rental_real_estate_basis_loss.divided(share);
            carried.rental_real_estate_risk_loss =
                carried.rental_real_estate_risk_loss.divided(share);
            carried.other_rental_basis_loss = carried.other_rental_basis_loss.divided(share);
            carried.other_rental_risk_loss = carried.other_rental_risk_loss.divided(share);
            carried.basis_section179 = carried.basis_section179.divided(share);
            carried.basis_charity = carried.basis_charity.divided(share);
            carried.basis_foreign_taxes = carried.basis_foreign_taxes.divided(share);
        }
        personal.basis.more_at_risk.excluded_funding =
            personal.basis.more_at_risk.excluded_funding.divided(share);
        personal.basis.more_at_risk.protected =
            personal.basis.more_at_risk.protected.divided(share);
        personal.basis.more_at_risk.released = personal.basis.more_at_risk.released.divided(share);
        personal.basis.more_at_risk.excluded_returns =
            personal.basis.more_at_risk.excluded_returns.divided(share);
        if let Some(ScorpAmtRecords::Separate { record }) = &mut personal.amt {
            *record = record.divided(share);
        }
        for history in [&mut personal.qbi_records.ordinary, &mut personal.qbi_records.property] {
            history.history = history.history.divided(share);
            history.at_risk = history.at_risk.divided(share);
            history.passive = history.passive.divided(share);
        }
        personal.section179_carryover = personal.section179_carryover.divided(share);
        personal.section179_carryover_sold = personal.section179_carryover_sold.divided(share);
        personal.amt_section179_income_carry = personal.amt_section179_income_carry.divided(share);
        for item in &mut personal.investment_forms {
            item.own_form = item.own_form.divided(share);
        }
        match &mut personal.disposition {
            Some(ScorpDisposition::Sold { proceeds }) => {
                *proceeds = proceeds.divided(share.for_gains());
            }
            Some(ScorpDisposition::Kept | ScorpDisposition::Other) | None => {}
        }
        result
    }
}
