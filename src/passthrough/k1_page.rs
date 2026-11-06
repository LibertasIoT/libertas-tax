// Coverage: existing K-1 calculation adapters and personal worksheet editors.
// Printed partnership and S-corporation sources live in partnership_source.rs
// and scorp_source.rs. Their adapters retain the established source checks,
// regular/AMT histories, activity allocations and existing calculation owners.
// Estate/trust paper and distinct-activity sources retain their existing paths.
// Evidence: tests/passthrough/{interview,k1_items,k1_rental,k1_sale_amt}.rs.
// Sources: https://www.irs.gov/instructions/i1065sk1
// https://www.irs.gov/instructions/i1120ssk
// https://www.irs.gov/instructions/i1041sk1
use crate::validation::ValidationIssue;
use crate::{
    EstateTrustK1Item, EstateTrustNiitAdjustment, FederalEstateTrustPortfolioK1,
    FinalEstateTrustNol, ReitDividends,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Schedule K-1 (Form 1041)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into estate_trust, fiduciary, beneficiary. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read this Schedule K-1 (Form 1041) and the fiduciary's attached statements exactly. Read `entity_period` from the actual current entity taxable-period dates and accounting election. Ordinary calendar, fiscal and short periods belong to the supplied beneficiary year when they end in that year. A normal elected 52/53-week year ending with reference to December belongs to the calendar beneficiary year of its reference December month-end, even when its actual ending is in early January of the next year (Reg. 1.441-2(e)); do not reject that source solely because its actual ending year differs. For Weeks52Or53, require the election's normal annual beginning and ending, reference month-end and Last/Nearest closing convention from the source statements; report missing or conflicting election facts as an error instead of inferring them. For CalendarFiscalAfterWeeks52Or53, require the actual approved change from a 52/53-week year and its normal month-first ordinary beginning; the preceding one-through-six-day stub belongs to that following period, not a separate tax year. Keep actual dates unchanged. A shortened elected December-reference period ending in a different calendar year from its reference month-end needs separate source-year verification and is not supported by the normal annual exception. Report a genuine beneficiary-year mismatch as an error. Never import original NOL periods or taxable-period history from the current entity-period header. Report illegible or conflicting values as errors, and never infer a fact the K-1 or its statements do not show. Parentheses or a minus sign mark a negative amount; return amounts with their printed sign. `estate_trust.federal_id` is item A, exactly nine digits without punctuation, preserving leading zeroes; a missing, masked or illegible EIN is an error. `name` is item B, the estate's or trust's name as printed. Box 1 is `interest`, box 2a `ordinary_dividends`, box 2b `qualified_dividends`, box 3 `short_gain`, box 4a `long_gain`, box 4b `rate_28_gain` and box 4c `unrecaptured_gain`; return zero for a clearly blank box. `more` holds boxes 10 through 14; return zero for a clearly blank amount there. Box 10 is `ird_estate_tax`. Box 12 is `amt`: None when box 12 shows none of codes A through F, CodeAOnly when code A is its only code among A through F, and Components when any of codes B through F is shown (code A must be shown; a blank code among B through F is zero). Box 12 code J is `minimum_tax_exclusion`, the signed exclusion-item component retained for the following year; return null when code J is absent and zero only for an explicitly reported zero. Do not add code J to code A or to any current-year AMT adjustment. Box 13 code A is `estimated_tax` Allocated with its amount, with `filed_1041t` true only when item D shows that Form 1041-T was filed; without code A it is None. Box 13 code B is `backup_withholding`. Box 14 code A is `tax_exempt_interest`, and `private_activity_interest` is the part of it the fiduciary's statement identifies as private activity bond interest (zero when none is shown). Box 14 code H is `niit`: `line_7` is its signed amount and `modified_agi` equals `line_7` when positive; when negative it is the signed part the statement identifies as also changing modified adjusted gross income (zero when the statement says none does; report an error when the statement does not say). `niit` is null when code H is absent. Return zero for a clearly blank amount. `unentered` lists each category at most once: AmtItems for box 12 codes G, H or I; Credits for unentered box 13 credits other than R; CreditRecapture for box 13 code R; ForeignTaxes for unentered box 14 code B; Rehabilitation for unentered code C; EnergyProperty for unentered code D; InvestmentIncome for unentered code E; ForeignTradingGrossReceipts for code G; AdvancedCoalGasification for code J; AdvancedEnergy for code K; AdvancedManufacturing for code L; CleanElectricity for unentered code M; OtherInformation for unclassified code ZZ information. Do not add an unentered category when its complete actual statement has already been entered in the corresponding separate source fields. The imported `unentered` list replaces the existing list. Read only marked source fields. Activity and QBI current-source rows have their own import actions; never replace their unmarked identities, participation or historical balances. Do not infer successor entitlement or source legal classifications.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct K1EstateTrust {
    /// Items A and B estate or trust
    #[libertas_llm_doc_field]
    pub estate_trust: crate::PrintedParty,
    /// Item C fiduciary name and address
    #[libertas_llm_doc_field]
    pub fiduciary: crate::PrintedPartyNameAddress,
    /// Items F and G beneficiary
    #[libertas_llm_doc_field]
    pub beneficiary: crate::PrintedParty,
    /// Successor of the terminated entity
    /// [NullText]
    /// Entitlement is not established
    /// [SetText]
    /// I succeed to the terminated entity
    #[libertas_hidden]
    #[libertas_default($null)]
    pub successor: Option<bool>,
    /// Previously entered mortgage records
    #[libertas_hidden]
    pub final_mortgages: Vec<i64>,
    /// Actual current Schedule K-1 entity period and accounting convention
    /// Retain the printed dates and the fiduciary's actual accounting election. A normal December-reference 52/53-week year can end in early January of the following calendar year.
    /// [NullText]
    /// The current entity period and accounting convention are not established
    /// [SetText]
    /// The actual current entity period and accounting convention are
    #[libertas_llm_doc_field]
    pub entity_period: Option<EstateCurrentPeriodPage>,
    /// Final-year source character and beneficiary entitlement
    /// [NullText]
    /// The fiduciary's statement gives no final-year source character
    /// [SetText]
    /// The final-year source character and entitlement are
    pub final_character: Option<crate::EstateFinalCharacterPage>,
    /// Signed box 12 reconciliation and activity allocations
    /// [NullText]
    /// No signed box 12 reconciliation is available
    /// [SetText]
    /// The signed box 12 reconciliation and activity allocations are
    pub amt_reconciliation: Option<crate::EstateAmtReconciliationPage>,
    /// Box 14 code E source net investment income for Form 4952
    /// [NullText]
    /// No box 14 code E investment-income statement is available
    /// [SetText]
    /// The box 14 code E investment-income statement is
    #[libertas_default($null)]
    pub investment_income: Option<crate::EstateInvestmentIncome>,
    /// Box 13 credit statements by their Form 1041 code identity
    pub reported_credits: Vec<crate::EstateReportedCreditPage>,
    /// Box 14 code B foreign tax statement
    /// For a country allocation, enter only passive interest and ordinary dividends from the fiduciary's attached statement, the actual paid/accrued method and separate regular/AMT amounts. Missing amounts remain unavailable. The entire code B tax must reconcile; capital gains, business and rental allocations need separate worksheets. Do not repeat the allocated income or tax on the foreign-credit source page.
    /// [NullText]
    /// No box 14 code B foreign-tax statement is available
    /// [SetText]
    /// The box 14 code B foreign-tax statement is
    #[libertas_default($null)]
    pub foreign_statement: Option<crate::EstateForeignStatement>,
    /// Fiduciary investment-credit property source statements
    pub investment_property: Vec<crate::EstateInvestmentPropertyPage>,

    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub estate_activities: Vec<crate::EstateSourceChoice>,
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub estate_mortgages: Vec<crate::EstateSourceChoice>,
    /// Box 1 interest income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Box 2a ordinary dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ordinary_dividends: i64,
    /// Box 2b qualified dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 3 net short-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub short_gain: i64,
    /// Box 4a net long-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub long_gain: i64,
    /// Box 4b 28% rate gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rate_28_gain: i64,
    /// Box 4c unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_gain: i64,
    /// Box 5 other portfolio and nonbusiness income
    #[libertas_default(None)]
    pub royalties: K1EstateTrustRoyalties,
    /// Boxes 6–8 business and rental income
    #[libertas_default(None)]
    pub business: K1EstateTrustBusiness,
    /// Fiduciary section 199A statements
    /// Import each statement in its own row to retain the row's source links and prior-loss records.
    pub qbi: crate::EstateQbiStatementsPage,
    /// Box 14 code F gross farming and fishing income
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub gross_farming_fishing_income: i64,
    /// Other boxes
    #[libertas_llm_doc_field]
    pub more: K1EstateTrustOtherBoxes,
    /// Something else applies
    /// A foreign beneficiary, unclassified box 5 annuity or business royalty, income in respect of a decedent beyond the entered allocation, or inconsistent source treatment.
    #[libertas_default(false)]
    pub special: bool,
}

/// Box 5 other portfolio and nonbusiness income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1EstateTrustRoyalties {
    /// No box 5 amount
    None,
    /// Ordinary nonbusiness royalties
    /// Choose this only when the whole box 5 amount is ordinary domestic nonbusiness royalties.
    Royalties {
        /// Box 5 amount, all ordinary nonbusiness royalties
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Box 9 deduction for these royalties
        #[libertas_default(None)]
        deduction: K1RoyaltyDeduction,
    },
}

/// Box 9 deduction for these royalties
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1RoyaltyDeduction {
    /// None
    None,
    /// Code A depreciation of one master sound recording
    /// Straight-line depreciation the fiduciary began before 2026 on one recording it held all year and licensed for these royalties, with the same amount for regular tax, AMT and net investment income.
    Depreciation {
        /// Box 9 code A amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Code B cost depletion of one mineral royalty property
    /// Cost depletion on one domestic nonbusiness mineral royalty property the fiduciary held all year, from its per-property statement, with the same amount for regular tax and AMT.
    Depletion {
        /// Box 9 code B amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Code C section 197 amortization
    /// Straight-line amortization begun before 2026 for this one royalty activity, fully allowable with the same amount for regular tax, AMT and net investment income.
    Amortization {
        /// Box 9 code C amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

/// Boxes 6–8 business and rental income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1EstateTrustBusiness {
    /// No amounts in boxes 6–8
    None,
    /// Amounts in boxes 6–8
    Amounts(crate::EstateActivitiesPage),
}

/// Actual current entity taxable period from the K-1 and accounting election
/// Required import leaves prevent guessed dates. Historical inherited-loss periods remain in their separate retained records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateCurrentPeriodPage {
    /// Actual tax year began
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    #[libertas_number(min = 19000101)]
    pub began: u32,
    /// Actual tax year ended
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    #[libertas_number(min = 19000101)]
    pub ended: u32,
    /// Actual accounting convention
    #[libertas_llm_doc_field]
    pub accounting: EstateCurrentAccountingPage,
}
/// Current entity's accounting election, separate from inherited-loss import history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EstateCurrentAccountingPage {
    /// Calendar, fiscal or ordinary short tax year
    CalendarFiscalOrShort,
    /// Elected 52/53-week tax year, including its first or final short year
    /// Copy the normal annual boundaries and reference month-end from the fiduciary accounting election/returns. The actual period may be shortened by creation or termination; a change stub of at most six days may precede the normal beginning and is not another taxable year.
    Weeks52Or53 {
        /// Normal annual beginning, before any first-year shortening or appended change stub
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_date_only]
        #[libertas_number(min = 18990101)]
        annual_began: u32,
        /// Normal annual ending, before any final-year termination
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_date_only]
        #[libertas_number(min = 19000101)]
        annual_ended: u32,
        /// Last calendar day of the election's reference closing month
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_date_only]
        #[libertas_number(min = 19000101)]
        reference_ended: u32,
        /// Election's annual closing convention
        #[libertas_llm_doc_field(allow_empty = false)]
        closing: crate::FinalEstateTrustWeekClosing,
    },
    /// Ordinary year following an approved change from a 52/53-week year
    /// Retain the appended one-through-six-day change stub in the actual period.
    CalendarFiscalAfterWeeks52Or53 {
        /// Normal ordinary beginning, after the appended change stub
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_date_only]
        #[libertas_number(min = 19000101)]
        annual_began: u32,
    },
}

impl EstateCurrentPeriodPage {
    pub(crate) fn record(&self) -> crate::FinalEstateTrustTaxPeriod {
        crate::FinalEstateTrustTaxPeriod {
            period: crate::FinalEstateTrustLossPeriod { began: self.began, ended: self.ended },
            accounting: match &self.accounting {
                EstateCurrentAccountingPage::CalendarFiscalOrShort => {
                    crate::FinalEstateTrustAccountingYear::CalendarFiscalOrShort
                }
                EstateCurrentAccountingPage::CalendarFiscalAfterWeeks52Or53 { annual_began } => {
                    crate::FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
                        annual_began: *annual_began,
                    }
                }
                EstateCurrentAccountingPage::Weeks52Or53 {
                    annual_began,
                    annual_ended,
                    reference_ended,
                    closing,
                } => crate::FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began: *annual_began,
                    annual_ended: *annual_ended,
                    reference_ended: *reference_ended,
                    closing: closing.clone(),
                },
            },
        }
    }
    pub(crate) fn from_record(p: &crate::FinalEstateTrustTaxPeriod) -> Self {
        Self {
            began: p.period.began,
            ended: p.period.ended,
            accounting: match &p.accounting {
                crate::FinalEstateTrustAccountingYear::CalendarFiscalOrShort => {
                    EstateCurrentAccountingPage::CalendarFiscalOrShort
                }
                crate::FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
                    annual_began,
                } => EstateCurrentAccountingPage::CalendarFiscalAfterWeeks52Or53 {
                    annual_began: *annual_began,
                },
                crate::FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began,
                    annual_ended,
                    reference_ended,
                    closing,
                } => EstateCurrentAccountingPage::Weeks52Or53 {
                    annual_began: *annual_began,
                    annual_ended: *annual_ended,
                    reference_ended: *reference_ended,
                    closing: closing.clone(),
                },
            },
        }
    }
}

/// Other boxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1EstateTrustOtherBoxes {
    /// Box 10 estate tax deduction on interest in box 1
    /// Only the fiduciary's allocation that belongs entirely to taxable interest in box 1 from income in respect of a decedent.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ird_estate_tax: i64,
    /// Box 11 final year deductions (item E, Final K-1)
    #[libertas_default(NotFinal)]
    pub final_year: K1FinalYear,
    /// Box 12 alternative minimum tax adjustment
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub amt: K1EstateTrustAmt,
    /// Box 12 code J exclusion items for the following year's minimum tax credit
    /// Retain the fiduciary's signed exclusion-item component of code A. This is not another 2026 AMT adjustment. Leave absent when code J is not reported; zero confirms a reported zero.
    /// [NullText]
    /// Box 12 code J is not reported
    /// [SetText]
    /// The signed exclusion-item component in box 12 code J is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default($null)]
    pub minimum_tax_exclusion: Option<i64>,
    /// Box 13 code A estimated tax payments allocated to you
    /// Do not also enter this amount among your own estimated payments.
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub estimated_tax: K1EstateTrustEstimatedTax,
    /// Box 13 code B backup withholding
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub backup_withholding: i64,
    /// Box 14 code A tax-exempt interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_exempt_interest: i64,
    /// Part of box 14 code A from private activity bonds
    /// The fiduciary's complete private-activity portion, including zero.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity_interest: i64,
    /// Box 14 code H net investment income adjustment
    /// [NullText]
    /// Box 14 code H is not reported
    /// [SetText]
    /// The box 14 code H net-investment-income adjustment is
    #[libertas_llm_doc_field]
    pub niit: Option<EstateTrustNiitAdjustment>,
    /// Box 14 code I section 199A dividends
    /// Qualified REIT and RIC dividends from the fiduciary's statement, included in box 2a and separate from box 2b. Holding-period eligibility is answered on the income topic.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub section199a_dividends: i64,
    /// Holding-period eligibility of the reported section 199A dividends
    /// [NullText]
    /// Eligibility is not established
    /// [SetText]
    /// The dividends meet the holding-period rules
    #[libertas_hidden]
    #[libertas_default($null)]
    pub section199a_eligible: Option<bool>,
    /// Other boxes with an amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    /// ----
    /// Box with an amount
    pub unentered: Vec<EstateTrustK1Item>,
}

/// Box 11 final year deductions (item E, Final K-1)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1FinalYear {
    /// Not a final K-1
    NotFinal,
    /// Final K-1
    /// Enter reported box 11 source amounts. Supply the separate final-year source facts and beneficiary entitlement before claiming them; item E and Final K-1 identify different facts.
    Final {
        /// Code A excess deductions — section 67(e)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        section_67e: i64,
        /// Code B personal property tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        personal_property_tax: i64,
        /// Code B real estate tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        real_estate_tax: i64,
        /// Code B state or local income tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        state_income_tax: i64,
        /// Code B state or local general sales tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        state_sales_tax: i64,
        /// Code C short-term capital loss carryover
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        short_carry: i64,
        /// Code D long-term capital loss carryover
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        long_carry: i64,
        /// Codes E and F net operating loss carryover
        /// [NullText]
        /// No final-year net operating loss carryover is reported
        /// [SetText]
        /// The codes E and F net operating loss carryovers are
        #[libertas_default($null)]
        nol: Option<FinalEstateTrustNol>,
    },
}

/// Box 12 alternative minimum tax adjustment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1EstateTrustAmt {
    /// None
    None,
    /// Only code A
    /// No other box 12 code and no companion AMT statement.
    CodeAOnly {
        /// Code A adjustment for minimum tax purposes
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        a: i64,
    },
    /// Code A with codes B to F
    /// Copy the printed signed components. Supply the separate reconciliation when A includes other adjustments or component signs differ; E and F identify special-rate portions of D.
    Components {
        /// Code A adjustment for minimum tax purposes
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        a: i64,
        /// Code B attributable to qualified dividends
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        b: i64,
        /// Code C attributable to net short-term capital gain
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        c: i64,
        /// Code D attributable to net long-term capital gain
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        d: i64,
        /// Code E attributable to unrecaptured section 1250 gain
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        e: i64,
        /// Code F attributable to 28% rate gain
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        f: i64,
    },
}

/// Box 13 code A estimated tax payments allocated to you
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum K1EstateTrustEstimatedTax {
    /// None
    None,
    /// Allocated to you
    Allocated {
        /// Box 13 code A amount
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Form 1041-T was filed (item D)
        /// Filed on time, allocating this payment to you.
        #[libertas_llm_doc_field]
        #[libertas_default(true)]
        filed_1041t: bool,
    },
}

// The stored record's "nothing applies" values: every amount zero, every
// confirmation No, no optional statement.
pub(crate) fn empty_record() -> FederalEstateTrustPortfolioK1 {
    FederalEstateTrustPortfolioK1 {
        entity_period: None,
        minimum_tax_exclusion: None,
        final_character: None,
        amt_reconciliation: None,
        investment_income: None,
        reported_credits: Vec::new(),
        foreign_statement: None,
        investment_property: Vec::new(),
        ein: String::new(),
        interest: 0,
        ordinary_dividends: 0,
        qualified_dividends: 0,
        section199a_dividends: ReitDividends::None,
        code_i_dividends_only: false,
        short_gain: 0,
        long_gain: 0,
        rate_28_gain: 0,
        unrecaptured_gain: 0,
        royalty_income: 0,
        box_5_royalty_only: false,
        box_9_royalty_depreciation: 0,
        box_9_royalty_sound_recording_only: false,
        box_9_royalty_depletion: 0,
        box_9_royalty_cost_depletion_only: false,
        box_9_royalty_amortization: 0,
        box_9_royalty_amortization_only: false,
        activities: crate::EstateActivities::empty(),
        qbi: crate::EstateQbiStatements { statements: Vec::new() },
        gross_farming_fishing_income: 0,
        box_10_interest_ird_estate_tax: 0,
        box_10_interest_ird_only: false,
        final_67e_deduction: 0,
        final_67e_no_niit_allocation: false,
        final_personal_property_tax: 0,
        final_real_estate_tax: 0,
        final_state_income_tax: 0,
        final_state_sales_tax: 0,
        final_taxes_only: false,
        final_short_carry: 0,
        final_long_carry: 0,
        final_year_successor: None,
        final_k1: false,
        final_mortgages: Vec::new(),
        final_carry_same_tax_character: false,
        final_nol: None,
        amt_box12a: 0,
        amt_box12a_standalone: false,
        amt_box12b: 0,
        amt_box12_components_confirmed: false,
        amt_box12c: 0,
        amt_box12d: 0,
        amt_box12e: 0,
        amt_box12e_included_in_d: false,
        amt_box12f: 0,
        amt_box12f_included_in_d: false,
        backup_withholding: 0,
        allocated_estimated_tax: 0,
        form_1041t_timely: false,
        tax_exempt_interest: 0,
        private_activity_interest: 0,
        niit_adjustment: None,
        unentered: Vec::new(),
        complete_portfolio_only: false,
    }
}

impl K1RoyaltyDeduction {
    fn amount(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Depreciation { amount }
            | Self::Depletion { amount }
            | Self::Amortization { amount } => *amount,
        }
    }
}

impl K1EstateTrust {
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the server twin of the schema rules).
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    #[cfg(test)]
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, require_answers: bool) -> Option<ValidationIssue> {
        if !crate::estate_trust::ein_valid(&self.estate_trust.federal_id) {
            return Some(
                ValidationIssue::new("ESTATE_EIN_INVALID")
                    .at_field("federal_id")
                    .in_field("estate_trust"),
            );
        }
        let more = &self.more;
        let (section199a, private, exempt) =
            (more.section199a_dividends, more.private_activity_interest, more.tax_exempt_interest);
        if i128::from(self.qualified_dividends) + i128::from(section199a)
            > i128::from(self.ordinary_dividends)
        {
            return Some(
                ValidationIssue::new("K1_1041_QUALIFIED_ABOVE_2A").at_field("qualified_dividends"),
            );
        }
        if i128::from(self.rate_28_gain) + i128::from(self.unrecaptured_gain)
            > i128::from(self.long_gain)
        {
            return Some(ValidationIssue::new("K1_1041_4B_4C_ABOVE_4A").at_field("rate_28_gain"));
        }
        if private > exempt {
            return Some(
                ValidationIssue::new("K1_1041_PRIVATE_ABOVE_14A")
                    .at_field("private_activity_interest")
                    .in_field("more"),
            );
        }
        if let K1EstateTrustRoyalties::Royalties { amount, deduction } = &self.royalties
            && deduction.amount() > *amount
        {
            return Some(
                ValidationIssue::new("K1_1041_ROYALTY_DEDUCTION_ABOVE_BOX_5")
                    .at_field("amount")
                    .in_field("deduction")
                    .in_field("royalties"),
            );
        }
        let unentered = &more.unentered;
        if let Some(index) = unentered
            .iter()
            .enumerate()
            .find_map(|(index, item)| unentered[..index].contains(item).then_some(index))
        {
            return Some(
                ValidationIssue::new("K1_1041_BOX_TWICE")
                    .at_list_item("unentered", index)
                    .in_field("more"),
            );
        }
        let activities = match &self.business {
            K1EstateTrustBusiness::None => crate::EstateActivities::empty(),
            K1EstateTrustBusiness::Amounts(activities) => {
                if let Some(issue) = if require_answers {
                    activities.issue()
                } else {
                    activities.record().issue_with_mode(false)
                } {
                    return Some(issue.in_field("business"));
                }
                activities.record()
            }
        };
        (if require_answers {
            self.qbi.issue(&activities)
        } else {
            self.qbi.record().issue_with_mode(&activities, false)
        })
        .map(|issue| issue.in_field("qbi"))
        .or_else(|| {
            crate::estate_trust::source_issue_with_mode(&self.record(), require_answers).map(
                |issue| {
                    issue
                        .map_field_prefix(&["activities"], &["business"])
                        .map_field_prefix(&["entity_period", "period"], &["entity_period"])
                        .map_field_prefix(
                            &["minimum_tax_exclusion"],
                            &["more", "minimum_tax_exclusion"],
                        )
                        .map_field_prefix(&["final_nol"], &["more", "final_year", "nol"])
                },
            )
        })
    }

    // The stored record this page writes.
    pub(crate) fn record(&self) -> FederalEstateTrustPortfolioK1 {
        let mut r = empty_record();
        r.ein = self.estate_trust.federal_id.clone();
        r.final_year_successor = self.successor;
        r.final_mortgages = self.final_mortgages.clone();
        r.entity_period = self.entity_period.as_ref().map(EstateCurrentPeriodPage::record);
        r.interest = self.interest;
        r.ordinary_dividends = self.ordinary_dividends;
        r.qualified_dividends = self.qualified_dividends;
        r.short_gain = self.short_gain;
        r.long_gain = self.long_gain;
        r.rate_28_gain = self.rate_28_gain;
        r.unrecaptured_gain = self.unrecaptured_gain;
        if let K1EstateTrustRoyalties::Royalties { amount, deduction } = &self.royalties {
            r.royalty_income = *amount;
            r.box_5_royalty_only = true;
            match deduction {
                K1RoyaltyDeduction::None => {}
                K1RoyaltyDeduction::Depreciation { amount } => {
                    r.box_9_royalty_depreciation = *amount;
                    r.box_9_royalty_sound_recording_only = *amount > 0;
                }
                K1RoyaltyDeduction::Depletion { amount } => {
                    r.box_9_royalty_depletion = *amount;
                    r.box_9_royalty_cost_depletion_only = *amount > 0;
                }
                K1RoyaltyDeduction::Amortization { amount } => {
                    r.box_9_royalty_amortization = *amount;
                    r.box_9_royalty_amortization_only = *amount > 0;
                }
            }
        }
        if let K1EstateTrustBusiness::Amounts(a) = &self.business {
            r.activities = a.record();
        }
        r.final_character =
            self.final_character.as_ref().map(crate::EstateFinalCharacterPage::record);
        r.amt_reconciliation =
            self.amt_reconciliation.as_ref().map(crate::EstateAmtReconciliationPage::record);
        r.investment_income = self.investment_income.clone();
        r.reported_credits =
            self.reported_credits.iter().map(crate::EstateReportedCreditPage::record).collect();
        r.foreign_statement = self.foreign_statement.clone();
        r.investment_property = self
            .investment_property
            .iter()
            .map(crate::EstateInvestmentPropertyPage::record)
            .collect();
        r.qbi = self.qbi.record();
        r.gross_farming_fishing_income = self.gross_farming_fishing_income;
        {
            let K1EstateTrustOtherBoxes {
                ird_estate_tax,
                final_year,
                amt,
                minimum_tax_exclusion,
                estimated_tax,
                backup_withholding,
                tax_exempt_interest,
                private_activity_interest,
                niit,
                section199a_dividends,
                section199a_eligible,
                unentered,
            } = &self.more;
            r.minimum_tax_exclusion = *minimum_tax_exclusion;
            r.final_k1 = matches!(final_year, K1FinalYear::Final { .. });
            r.box_10_interest_ird_estate_tax = *ird_estate_tax;
            r.box_10_interest_ird_only = *ird_estate_tax > 0;
            if let K1FinalYear::Final {
                section_67e,
                personal_property_tax,
                real_estate_tax,
                state_income_tax,
                state_sales_tax,
                short_carry,
                long_carry,
                nol,
            } = final_year
            {
                r.final_67e_deduction = *section_67e;
                r.final_personal_property_tax = *personal_property_tax;
                r.final_real_estate_tax = *real_estate_tax;
                r.final_state_income_tax = *state_income_tax;
                r.final_state_sales_tax = *state_sales_tax;
                r.final_short_carry = *short_carry;
                r.final_long_carry = *long_carry;
                r.final_nol = nol.clone();
                r.final_67e_no_niit_allocation = true;
                r.final_taxes_only = true;
                r.final_carry_same_tax_character = true;
            }
            match amt {
                K1EstateTrustAmt::None => {}
                K1EstateTrustAmt::CodeAOnly { a } => {
                    r.amt_box12a = *a;
                    r.amt_box12a_standalone = true;
                }
                K1EstateTrustAmt::Components { a, b, c, d, e, f } => {
                    r.amt_box12a = *a;
                    r.amt_box12b = *b;
                    r.amt_box12c = *c;
                    r.amt_box12d = *d;
                    r.amt_box12e = *e;
                    r.amt_box12f = *f;
                    r.amt_box12_components_confirmed = true;
                    r.amt_box12e_included_in_d = *e != 0;
                    r.amt_box12f_included_in_d = *f != 0;
                }
            }
            if let K1EstateTrustEstimatedTax::Allocated { amount, filed_1041t } = estimated_tax {
                r.allocated_estimated_tax = *amount;
                r.form_1041t_timely = *filed_1041t;
            }
            r.backup_withholding = *backup_withholding;
            r.tax_exempt_interest = *tax_exempt_interest;
            r.private_activity_interest = *private_activity_interest;
            r.niit_adjustment = niit.clone();
            if *section199a_dividends > 0 || section199a_eligible.is_some() {
                r.section199a_dividends = ReitDividends::Reported {
                    amount: *section199a_dividends,
                    eligible: *section199a_eligible,
                };
                r.code_i_dividends_only = true;
            }
            r.unentered = unentered.clone();
        }
        r.complete_portfolio_only = !self.special;
        r
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written.
    #[cfg(test)]
    pub(crate) fn from_record(name: &str, r: &FederalEstateTrustPortfolioK1) -> Option<Self> {
        let page = Self::from_source_record(name, r);
        (page.record() == *r).then_some(page)
    }

    pub(crate) fn from_source_record(name: &str, r: &FederalEstateTrustPortfolioK1) -> Self {
        let royalties = if r.royalty_income == 0
            && !r.box_5_royalty_only
            && r.box_9_royalty_depreciation == 0
            && r.box_9_royalty_depletion == 0
            && r.box_9_royalty_amortization == 0
        {
            K1EstateTrustRoyalties::None
        } else {
            K1EstateTrustRoyalties::Royalties {
                amount: r.royalty_income,
                deduction: if r.box_9_royalty_depreciation > 0 {
                    K1RoyaltyDeduction::Depreciation { amount: r.box_9_royalty_depreciation }
                } else if r.box_9_royalty_depletion > 0 {
                    K1RoyaltyDeduction::Depletion { amount: r.box_9_royalty_depletion }
                } else if r.box_9_royalty_amortization > 0 {
                    K1RoyaltyDeduction::Amortization { amount: r.box_9_royalty_amortization }
                } else {
                    K1RoyaltyDeduction::None
                },
            }
        };
        let mut page = Self {
            successor: r.final_year_successor,
            final_mortgages: r.final_mortgages.clone(),
            fiduciary: Default::default(),
            beneficiary: Default::default(),
            entity_period: r.entity_period.as_ref().map(EstateCurrentPeriodPage::from_record),
            final_character: r
                .final_character
                .as_ref()
                .map(crate::EstateFinalCharacterPage::from_record),
            amt_reconciliation: r
                .amt_reconciliation
                .as_ref()
                .map(crate::EstateAmtReconciliationPage::from_record),
            investment_income: r.investment_income.clone(),
            reported_credits: r
                .reported_credits
                .iter()
                .map(crate::EstateReportedCreditPage::from_record)
                .collect(),
            foreign_statement: r.foreign_statement.clone(),
            investment_property: r
                .investment_property
                .iter()
                .map(crate::EstateInvestmentPropertyPage::from_record)
                .collect(),
            estate_activities: crate::estate_trust::activity_choices(&r.activities),
            estate_mortgages: Vec::new(),
            estate_trust: crate::PrintedParty {
                name: name.into(),
                federal_id: r.ein.clone(),
                address: Default::default(),
            },
            interest: r.interest,
            ordinary_dividends: r.ordinary_dividends,
            qualified_dividends: r.qualified_dividends,
            short_gain: r.short_gain,
            long_gain: r.long_gain,
            rate_28_gain: r.rate_28_gain,
            unrecaptured_gain: r.unrecaptured_gain,
            royalties,
            business: K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage::from_record(
                &r.activities,
            )),
            qbi: crate::EstateQbiStatementsPage::from_record(&r.qbi),
            gross_farming_fishing_income: r.gross_farming_fishing_income,
            more: K1EstateTrustOtherBoxes {
                ird_estate_tax: r.box_10_interest_ird_estate_tax,
                final_year: if r.final_k1 {
                    K1FinalYear::Final {
                        section_67e: r.final_67e_deduction,
                        personal_property_tax: r.final_personal_property_tax,
                        real_estate_tax: r.final_real_estate_tax,
                        state_income_tax: r.final_state_income_tax,
                        state_sales_tax: r.final_state_sales_tax,
                        short_carry: r.final_short_carry,
                        long_carry: r.final_long_carry,
                        nol: r.final_nol.clone(),
                    }
                } else {
                    K1FinalYear::NotFinal
                },
                amt: if r.amt_box12a_standalone {
                    K1EstateTrustAmt::CodeAOnly { a: r.amt_box12a }
                } else if r.amt_box12_components_confirmed {
                    K1EstateTrustAmt::Components {
                        a: r.amt_box12a,
                        b: r.amt_box12b,
                        c: r.amt_box12c,
                        d: r.amt_box12d,
                        e: r.amt_box12e,
                        f: r.amt_box12f,
                    }
                } else {
                    K1EstateTrustAmt::None
                },
                estimated_tax: if r.allocated_estimated_tax > 0 || r.form_1041t_timely {
                    K1EstateTrustEstimatedTax::Allocated {
                        amount: r.allocated_estimated_tax,
                        filed_1041t: r.form_1041t_timely,
                    }
                } else {
                    K1EstateTrustEstimatedTax::None
                },
                minimum_tax_exclusion: r.minimum_tax_exclusion,
                backup_withholding: r.backup_withholding,
                tax_exempt_interest: r.tax_exempt_interest,
                private_activity_interest: r.private_activity_interest,
                niit: r.niit_adjustment.clone(),
                section199a_dividends: r.section199a_dividends.amount(),
                section199a_eligible: match &r.section199a_dividends {
                    ReitDividends::Reported { eligible, .. } => *eligible,
                    _ => None,
                },
                unentered: r.unentered.clone(),
            },
            special: !r.complete_portfolio_only,
        };
        if r.activities.activities.is_empty() {
            page.business = K1EstateTrustBusiness::None;
        }
        page
    }
}

// Shared page types of the Schedule K-1 kinds (design A3). Each maps onto
// its stored record and back; the reverse picks a shortcut branch exactly
// when the stored record is what that branch writes.

/// How you took part in this business in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Participation {
    /// You did not work in it
    Investor {
        /// Time from 2016 through 2025 you materially participated in it
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_default(0)]
        #[libertas_physical_unit("year")]
        prior_years: u32,
    },
    /// You and your spouse worked in it more than 500 hours
    Over500Hours {
        /// Time you and your spouse worked in it
        // Unresolved cap: record() multiplies hours by 60 before native validation;
        // remove this bound only with a checked conversion and its existing failure path.
        #[libertas_number(min = 501, max = 8784)]
        #[libertas_physical_unit("hour")]
        hours: u32,
    },
    /// Something else, such as 100 to 500 hours
    Other {
        /// Your participation
        facts: crate::ScorpParticipation,
    },
}

impl Participation {
    /// The stored participation answers this choice writes.
    pub fn record(&self) -> crate::ScorpParticipation {
        let shortcut = |minutes: u32, prior_ten: u32| crate::ScorpParticipation {
            ordinary_scope: true,
            minutes,
            substantially_all: false,
            at_least_as_much: false,
            prior_ten,
            personal_service: false,
            service_years: 0,
            regular: false,
            management_minutes: 0,
            paid_manager: false,
            other_manager_more: false,
            other_business_over_100: false,
        };
        match self {
            Self::Investor { prior_years } => shortcut(0, *prior_years),
            // More than 500 hours is material participation whatever the
            // history, so the shortcut writes no earlier years.
            Self::Over500Hours { hours } => shortcut(hours * 60, 0),
            Self::Other { facts } => facts.clone(),
        }
    }
    /// The choice a stored participation record reopens as.
    pub fn from_record(facts: &crate::ScorpParticipation) -> Self {
        let investor = Self::Investor { prior_years: facts.prior_ten };
        let over = Self::Over500Hours { hours: facts.minutes / 60 };
        if facts.prior_ten <= 10 && investor.record() == *facts {
            investor
        } else if (501..=8784).contains(&(facts.minutes / 60)) && over.record() == *facts {
            over
        } else {
            Self::Other { facts: facts.clone() }
        }
    }
}

/// How you took part in this business in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum LimitedParticipation {
    /// You did not work in it
    Investor {
        /// Time from 2016 through 2025 you materially participated in it
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_default(0)]
        #[libertas_physical_unit("year")]
        prior_years: u32,
    },
    /// You and your spouse worked in it more than 500 hours
    Over500Hours {
        /// Time you and your spouse worked in it
        // Unresolved cap: record() multiplies hours by 60 before native validation;
        // remove this bound only with a checked conversion and its existing failure path.
        #[libertas_number(min = 501, max = 8784)]
        #[libertas_physical_unit("hour")]
        hours: u32,
    },
    /// Something else, such as 100 to 500 hours
    Other {
        /// Combined qualifying work by the partner and spouse
        /// Exclude investor-only work and work performed to evade the passive-loss rules.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("minute")]
        minutes: u32,
        /// Material participation among 2016 through 2025
        /// Count the returns from 2016 through 2025 on which Schedule E Part II line 28 reported this activity as nonpassive (column (i) or (k)) because you materially participated. Do not establish those solely through the historical five-of-ten test. Enter zero when there were none.
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_physical_unit("year")]
        prior_ten: u32,
        /// This is a personal-service activity
        /// Health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting or another field where capital is not a material income-producing factor (Reg. §1.469-5T(d)).
        personal_service: bool,
        /// Earlier material participation in this personal-service activity
        /// Count them from your earlier returns: every return before 2026 that Schedule E Part II reported this activity as nonpassive (line 28 column (i) or (k)) because you materially participated. Enter zero when there were none or when it is not a personal-service activity.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("year")]
        service_years: u32,
    },
}

impl LimitedParticipation {
    /// The stored limited-partner participation this choice writes.
    pub fn record(&self) -> crate::PartnerParticipation {
        let (minutes, prior_ten, personal_service, service_years) = match self {
            Self::Investor { prior_years } => (0, *prior_years, false, 0),
            Self::Over500Hours { hours } => (hours * 60, 0, false, 0),
            Self::Other { minutes, prior_ten, personal_service, service_years } => {
                (*minutes, *prior_ten, *personal_service, *service_years)
            }
        };
        crate::PartnerParticipation::Limited { minutes, prior_ten, personal_service, service_years }
    }
    /// The choice a stored limited-partner record reopens as; None for a
    /// general partner's record or an unconfirmed one.
    pub fn from_record(stored: &crate::PartnerParticipation) -> Option<Self> {
        let crate::PartnerParticipation::Limited {
            minutes,
            prior_ten,
            personal_service,
            service_years,
        } = stored
        else {
            return None;
        };
        let investor = Self::Investor { prior_years: *prior_ten };
        let over = Self::Over500Hours { hours: minutes / 60 };
        Some(if *prior_ten <= 10 && investor.record() == *stored {
            investor
        } else if (501..=8784).contains(&(minutes / 60)) && over.record() == *stored {
            over
        } else {
            Self::Other {
                minutes: *minutes,
                prior_ten: *prior_ten,
                personal_service: *personal_service,
                service_years: *service_years,
            }
        })
    }
}

/// Rental income (loss) — boxes 2 and 3
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1RentalBoxes {
    /// Box 2 net rental real estate income (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate: i64,
    /// Section 1231 gain (loss) of the rental real estate activity
    /// The part of box 10 (Form 1120-S box 9) the statement assigns to the box 2 activity. Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate_section1231: i64,
    /// Unrecaptured section 1250 gain of the rental real estate activity
    /// The part of the business-property amount in box 9c (Form 1120-S box 8c) from the box 2 activity.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate_unrecaptured: i64,
    /// Box 3 other net rental income (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// Section 1231 gain (loss) of the other rental activity
    /// The part of box 10 (Form 1120-S box 9) the statement assigns to the box 3 activity. Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub other_section1231: i64,
    /// Unrecaptured section 1250 gain of the other rental activity
    /// The part of the business-property amount in box 9c (Form 1120-S box 8c) from the box 3 activity.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_unrecaptured: i64,
    /// How you took part in the rental real estate (box 2)
    #[libertas_default($null)]
    #[libertas_hidden]
    /// [NullText]
    /// Participation is not established yet
    /// [SetText]
    /// Participation is
    pub participation: Option<crate::K1RentalParticipation>,
    /// Section 199A amounts for box 2
    /// When the section 199A statement lists the rental real estate activity as a trade or business.
    /// [NullText]
    /// I am not claiming a section 199A deduction for box 2
    /// [SetText]
    /// The box 2 section 199A amounts are
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub real_estate_qbi: Option<crate::K1RentalQbi>,
    /// Section 199A amounts for box 3
    /// When the section 199A statement lists the other rental activity as a trade or business.
    /// [NullText]
    /// I am not claiming a section 199A deduction for box 3
    /// [SetText]
    /// The box 3 section 199A amounts are
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub other_qbi: Option<crate::K1RentalQbi>,
    /// You actively participated in the year the box 2 loss carried from 2025 arose
    /// Form 8582 counts a prior-year unallowed loss toward the special allowance only for a year in which you actively participated. Leave on when there is no carried loss.
    #[libertas_default(true)]
    #[libertas_hidden]
    pub carryover_active: bool,
    /// Rental activities whose property the entity sold in full in 2026
    /// All of the activity's property, sold or exchanged for cash or other property in a fully taxable sale to a buyer not related to you or to the entity (not an installment sale, a like-kind exchange or a casualty). The activity's suspended passive losses are then allowed. Leave empty when none.
    #[libertas_unordered]
    // Tax structure: only box 2 real estate and box 3 other rental activities.
    #[libertas_size(max = 2)]
    #[libertas_hidden]
    /// ----
    /// Activity
    pub disposed: Vec<crate::K1RentalActivity>,
}

impl K1RentalBoxes {
    // The stored amounts of one tax system with the carryovers from the
    // basis fold, and the person's facts.
    fn record(
        &self,
        carryovers: [i64; 2],
        property_carries: [i64; 2],
    ) -> (crate::K1RentalAmounts, crate::K1RentalFacts) {
        (
            crate::K1RentalAmounts {
                real_estate: self.real_estate,
                real_estate_carryover: carryovers[0],
                real_estate_property: crate::K1RentalProperty {
                    net: self.real_estate_section1231,
                    unrecaptured: self.real_estate_unrecaptured,
                    passive_loss: property_carries[0],
                },
                other: self.other,
                other_carryover: carryovers[1],
                other_property: crate::K1RentalProperty {
                    net: self.other_section1231,
                    unrecaptured: self.other_unrecaptured,
                    passive_loss: property_carries[1],
                },
            },
            crate::K1RentalFacts {
                participation: self.participation,
                real_estate_qbi: self.real_estate_qbi.clone(),
                other_qbi: self.other_qbi.clone(),
                carryover_active: self.carryover_active,
                disposed: self.disposed.clone(),
            },
        )
    }
    fn from_record(amounts: &crate::K1RentalAmounts, facts: &crate::K1RentalFacts) -> Self {
        Self {
            real_estate: amounts.real_estate,
            real_estate_section1231: amounts.real_estate_property.net,
            real_estate_unrecaptured: amounts.real_estate_property.unrecaptured,
            other: amounts.other,
            other_section1231: amounts.other_property.net,
            other_unrecaptured: amounts.other_property.unrecaptured,
            participation: facts.participation,
            real_estate_qbi: facts.real_estate_qbi.clone(),
            other_qbi: facts.other_qbi.clone(),
            carryover_active: facts.carryover_active,
            disposed: facts.disposed.clone(),
        }
    }
}

/// Alternative minimum tax records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum PartnerAmtRecords {
    /// Use regular activity records as the AMT baseline; printed depletion adjustments are applied automatically
    /// Your AMT basis and carryovers otherwise equal your regular records. Separate AMT records must already include every applicable activity adjustment.
    SameAsRegular,
    /// Different for AMT
    Separate {
        /// Your AMT records for this partnership
        record: crate::PartnerActivityRecord,
    },
}

impl PartnerAmtRecords {
    /// The stored AMT record: the regular record itself when the filer
    /// states they are the same.
    pub fn record(&self, regular: &crate::PartnerActivityRecord) -> crate::PartnerActivityRecord {
        match self {
            Self::SameAsRegular => regular.clone(),
            Self::Separate { record } => record.clone(),
        }
    }
    /// The choice a stored AMT record reopens as.
    pub fn from_record(
        regular: &crate::PartnerActivityRecord,
        alternative: &crate::PartnerActivityRecord,
    ) -> Self {
        if alternative == regular {
            Self::SameAsRegular
        } else {
            Self::Separate { record: alternative.clone() }
        }
    }
}

/// Alternative minimum tax records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum ScorpAmtRecords {
    /// Use regular activity records as the AMT baseline; printed depletion adjustments are applied automatically
    /// Your AMT basis and carryovers otherwise equal your regular records. Separate AMT records must already include every applicable activity adjustment.
    SameAsRegular,
    /// Different for AMT
    Separate {
        /// Your AMT records for this corporation
        record: crate::ScorpAmt,
    },
}

impl ScorpAmtRecords {
    // The regular records restated as the AMT record.
    pub(crate) fn same_as(regular: &crate::ScorpK1) -> Option<crate::ScorpAmt> {
        Some(crate::ScorpAmt {
            adjustment: 0,
            capital: regular.capital.clone(),
            section1231: regular.section1231.clone(),
            stock: regular.opening?,
            prior_loss: regular.prior_loss,
            prior_nondeductible: regular.prior_nondeductible,
            loans: regular
                .loans
                .iter()
                .zip(1u32..)
                .map(|(loan, number)| crate::ScorpAmtDebt { loan: number, basis: loan.basis })
                .collect(),
            risk: regular.risk.clone(),
            passive: regular.passive.clone(),
            rentals: regular.rentals.clone(),
            carried: regular.carried.clone(),
            complete: true,
        })
    }
    /// The stored AMT record: the regular records restated when the filer
    /// states they are the same.
    pub fn record(&self, regular: &crate::ScorpK1) -> Option<crate::ScorpAmt> {
        match self {
            Self::SameAsRegular => Self::same_as(regular),
            Self::Separate { record } => Some(record.clone()),
        }
    }
    /// The choice a stored AMT record reopens as.
    pub fn from_record(regular: &crate::ScorpK1, amt: &crate::ScorpAmt) -> Self {
        if Self::same_as(regular).as_ref() == Some(amt) {
            Self::SameAsRegular
        } else {
            Self::Separate { record: amt.clone() }
        }
    }
}

/// What produces this income
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
pub enum K1IncomeSource {
    /// Not needed for this return
    NotNeeded,
    /// Your personal services; capital is not a material factor
    Services {
        /// Your gross income from these personal services
        /// Your allocable gross income before ordinary business expenses, from the business's records. Box 1 net income or self-employment earnings is not a substitute.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gross: i64,
    },
    /// Capital; your personal services are not a material factor
    Capital,
    /// Both
    Mixed {
        /// Value of your work
        value: crate::WorkValue,
    },
}

impl K1IncomeSource {
    /// This choice's index in the allowed set.
    pub fn index(&self) -> i32 {
        use libertas_macros::variant_index;
        (match self {
            Self::NotNeeded => variant_index!(K1IncomeSource::NotNeeded),
            Self::Services { .. } => variant_index!(K1IncomeSource::Services),
            Self::Capital => variant_index!(K1IncomeSource::Capital),
            Self::Mixed { .. } => variant_index!(K1IncomeSource::Mixed),
        }) as i32
    }
    /// The choices the page offers: the three real ones when the source is
    /// needed, otherwise "Not needed" alone, which the client selects itself.
    pub fn allowed(needed: bool) -> Vec<i32> {
        if needed {
            alloc::vec![
                libertas_macros::variant_index!(K1IncomeSource::Services) as i32,
                libertas_macros::variant_index!(K1IncomeSource::Capital) as i32,
                libertas_macros::variant_index!(K1IncomeSource::Mixed) as i32,
            ]
        } else {
            alloc::vec![libertas_macros::variant_index!(K1IncomeSource::NotNeeded) as i32]
        }
    }
    /// The stored income source and gross personal-service income.
    pub fn record(&self) -> (Option<crate::BusinessIncomeSource>, Option<i64>) {
        match self {
            Self::NotNeeded => (None, None),
            Self::Services { gross } => (Some(crate::BusinessIncomeSource::Services), Some(*gross)),
            Self::Capital => (Some(crate::BusinessIncomeSource::Capital), None),
            Self::Mixed { value } => (Some(value.record()), None),
        }
    }
    /// The choice the stored answers reopen as; None for an unresolved or
    /// inconsistent source.
    pub fn from_record(
        source: Option<&crate::BusinessIncomeSource>,
        gross: Option<i64>,
    ) -> Option<Self> {
        let choice = match source {
            None => Self::NotNeeded,
            Some(crate::BusinessIncomeSource::Services) => {
                Self::Services { gross: gross.unwrap_or(0) }
            }
            Some(crate::BusinessIncomeSource::Capital) => Self::Capital,
            Some(
                source @ (crate::BusinessIncomeSource::Mixed { .. }
                | crate::BusinessIncomeSource::MixedThirtyPercent),
            ) => Self::Mixed { value: crate::WorkValue::from_record(source)? },
        };
        (choice.record() == (source.cloned(), gross)).then_some(choice)
    }
}

// Schedule K-1 (Form 1120-S) kind (unit T2-31p, design A3b).

/// Transient inputs for the existing S-corporation calculation owner.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode)]
pub(crate) struct K1Scorp {
    /// Items A and B corporation
    pub corporation: crate::PrintedParty,
    /// Items E and F1 shareholder
    pub shareholder: crate::PrintedParty,
    /// Entity's reporting tax period
    /// Use this K-1's actual beginning and ending dates. Leave unknown when the source does not establish them.
    /// [NullText]
    /// The entity's tax period is unknown
    /// [SetText]
    /// The entity's actual tax period is
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Final K-1
    /// The box at the top of the K-1.
    pub final_k1: bool,
    /// Box 1 ordinary business income (loss)
    pub ordinary: i64,
    /// Rental income (loss) — boxes 2 and 3
    pub rentals: K1RentalBoxes,
    /// Box 4 interest income
    pub interest: i64,
    /// Box 5a ordinary dividends
    pub dividends: i64,
    /// Box 5b qualified dividends
    pub qualified_dividends: i64,
    /// Box 6 royalties
    /// Reported on Schedule E line 4. Box 12 code I deductions belong in the deductions list.
    pub royalties: i64,
    /// Box 16 code D distributions
    pub distributions: i64,
    /// Section 199A information (box 17 code V)
    pub qbi: K1ScorpQbi,
    /// Your retained QBI records and allocation confirmation
    /// From your own loss worksheets and source review. Document import preserves these records when the printed statement changes.
    /// [NullText]
    /// No retained personal records were supplied with this source.
    /// [SetText]
    /// Retained personal records
    pub qbi_records: Option<K1ScorpQbiRecords>,
    /// How you took part in this business in 2026
    /// [NullText]
    /// Participation is not established yet
    /// [SetText]
    /// Participation is
    pub participation: Option<Participation>,
    /// Other boxes
    pub more: K1ScorpOtherBoxes,
    /// Schedule K-3 (box 14)
    /// Box 14 is checked when the corporation attached a Schedule K-3. Without one, the foreign taxes in box 16 code F get no credit here; you can ask the corporation for the schedule.
    pub schedule_k3: crate::K3Attached,
    /// Your stock and debt basis (Form 7203) and amount at risk
    /// Your own records; the corporation does not track them.
    pub basis: K1ScorpBasis,
    /// Alternative minimum tax records
    pub amt: Option<ScorpAmtRecords>,
    /// What happened to your shares in 2026
    /// [NullText]
    /// What happened to this interest is not answered yet
    /// [SetText]
    /// What happened to this interest is
    pub disposition: Option<ScorpDisposition>,
    /// Something else applies
    /// Choose each that applies; leave empty when none does.
    /// ----
    /// Situation
    // Tax structure: one row per distinct S-corporation special situation in the fixed catalog.
    pub special: Vec<crate::ScorpSpecial>,
    /// Individual QBI aggregation group
    /// Enter the same group name on each actual business in your individual election. Leave empty when no group is selected. This is separate from any aggregate reported by the entity. Save a new source first, then reopen it before selecting a group.
    pub qbi_group: alloc::string::String,
    /// Stored record facts the boxes above cannot show
    pub exact: ScorpRecordFacts,
}

/// What happened to your shares in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum ScorpDisposition {
    /// You still own all of them
    Kept,
    /// You sold all of them
    /// To a buyer who isn't related to you, all reported this year (not an installment sale), and the corporation's statement shows no collectibles gain.
    Sold {
        /// What you received
        /// Cash plus the fair market value of any property, less your selling costs.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        proceeds: i64,
    },
    /// Something else
    /// Part of your shares, a gift, a death, a sale to a related person, an installment sale, a redemption or liquidation by the corporation, collectibles gain on the statement, or shares held partly one year or less.
    Other,
}

impl ScorpDisposition {
    fn record(&self) -> crate::K1Disposition {
        match self {
            Self::Kept => crate::K1Disposition::Kept,
            Self::Sold { proceeds } => crate::K1Disposition::Sold { proceeds: *proceeds },
            Self::Other => crate::K1Disposition::Other,
        }
    }
    #[cfg(test)]
    fn from_record(record: &crate::K1Disposition) -> Option<Self> {
        match record {
            crate::K1Disposition::Kept => Some(Self::Kept),
            crate::K1Disposition::Sold { proceeds } => Some(Self::Sold { proceeds: *proceeds }),
            crate::K1Disposition::Other => Some(Self::Other),
            crate::K1Disposition::LiquidatedForCash => None,
        }
    }
}

/// Section 199A information (box 17 code V)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum K1ScorpQbi {
    /// No section 199A statement came with this K-1
    NoStatement,
    /// Statement
    Statement {
        /// Qualified business income
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        income: i64,
        /// W-2 wages
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        wages: i64,
        /// Unadjusted basis of qualified property (UBIA)
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        property: i64,
        /// Specified service trade or business (SSTB)
        #[libertas_llm_doc_field]
        #[libertas_default(false)]
        service: bool,
        /// QBI portion of this year's ordinary business loss
        /// From this section 199A statement, as a positive amount; zero when box 1 is not a loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        current_loss: i64,
        /// Section 1231 QBI items on this statement
        /// This year's section 1231 items are box 9 and the business-property part of box 8c.
        #[libertas_llm_doc_field]
        property_qbi: K1ScorpPropertyQbi,
        /// Section 199A dividends
        /// From the statement; they are already part of box 5a.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        reit_dividends: i64,
        /// Qualified PTP items
        /// The statement's qualified publicly traded partnership income or loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_default(0)]
        ptp_income: i64,
    },
}

/// Section 1231 QBI items on this statement
/// Box 9 and the business-property part of box 8c are this year's section 1231 items; leave these amounts at zero when both are zero.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpPropertyQbi {
    /// Potential QBI portion of current net section 1231 gain
    /// From the section 199A statement, before current-year SSTB limits and taxpayer-level capital/ordinary characterization. Zero for a current net loss. Do not add this amount to ordinary business QBI income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub gain: i64,
    /// Qualified portion of the section 1250 gain component
    /// Optional supporting-statement amount included in both the potential QBI gain above and this business's box 8c amount, before SSTB limits or lookback recapture. Zero confirms none; choose the unavailable answer when the source does not establish it. This source breakdown can resolve partial recapture when gain classes have different qualification. It is not a calculated recapture amount. Offsetting property losses require their separate allocations.
    /// [NullText]
    /// The qualified section 1250 gain portion is not available
    /// [SetText]
    /// The documented qualified section 1250 gain portion is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub qualified_1250: Option<i64>,
    /// QBI portion of this year's net section 1231 loss
    /// From the section 199A statement, as a positive amount, before this year's shareholder SSTB income limit. Zero for a net gain. Do not include prior suspended losses.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub current: i64,
}

/// Stored S-corporation record facts that the printed boxes and answers cannot show
/// Each field is empty unless the stored record differs from what the boxes and answers write, so
/// a record with an explicit zero capital or section 1231 item, a QBI figure without a statement, an
/// unconfirmed statement or a supplied basis confirmation reopens exactly as stored.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ScorpRecordFacts {
    /// An all-zero portfolio capital record is stored
    pub capital_present: bool,
    /// An all-zero section 1231 record is stored
    pub section1231_present: bool,
    /// Current QBI loss stored while the K-1 has no section 199A statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub unstated_loss: i64,
    /// Potential QBI section 1231 gain stored while the K-1 has no statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub unstated_property_gain: i64,
    /// Qualified section 1250 portion stored while the K-1 has no statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub unstated_qualified_1250: Option<i64>,
    /// Current QBI section 1231 loss stored while the K-1 has no statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub unstated_property_loss: i64,
    /// The stored section 199A statement is not confirmed complete
    pub statement_unconfirmed: bool,
    /// A supplied financing confirmation that differs from the situations
    pub risk_complete: Option<bool>,
    /// A supplied passive-history confirmation that differs from the situations
    pub passive_complete: Option<bool>,
}

/// Your retained QBI records
/// These records come from your own worksheets and confirmations. They remain available when the K-1 has no section 199A statement.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpQbiRecords {
    /// Retained ordinary-business QBI loss histories
    pub ordinary: K1ScorpLossHistory,
    /// Retained section 1231 QBI loss histories
    pub property: K1ScorpLossHistory,
    /// Complete same-business section 1231 source allocation
    /// Confirm the section 199A statement covers this same business and the qualified portion has the same sign as its net section 1231 amount. Compare it with your original-year loss worksheets. No leaves this corporation's QBI out of the deduction with an Estimate note. Ignored when there is no section 1231 statement item.
    pub property_complete: bool,
}

/// Retained QBI loss histories
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1ScorpLossHistory {
    /// Retained basis-loss history
    /// From the QBI loss-tracking records kept with your 2025 return. Remaining amounts must reconcile to the section 1231 loss carried from 2025 because of basis. Historical QBI portions already reflect original-year eligibility; a pre-2018 loss has no QBI portion. Empty when none was carried.
    /// ----
    /// Loss year
    pub history: Vec<crate::ScorpLossYear>,
    /// Retained at-risk loss history
    /// From the same records, for section 1231 losses suspended by at-risk limits (2025 Form 6198). Empty when none was carried.
    /// ----
    /// Loss year
    pub at_risk: Vec<crate::ScorpLossYear>,
    /// Retained passive-loss history
    /// From the same records, for section 1231 losses suspended by passive limits (2025 Form 8582 worksheets). Empty when none was carried.
    /// ----
    /// Loss year
    pub passive: Vec<crate::ScorpLossYear>,
}

impl K1ScorpPropertyQbi {
    // The stored section 1231 QBI record: this year's statement items and
    // the carried-loss histories. The confirmation attests this year's
    // allocation and is read only when the K-1 has a section 1231 item.
    fn record(&self, history: &K1ScorpLossHistory, complete: bool) -> crate::ScorpPropertyQbi {
        crate::ScorpPropertyQbi {
            gain: self.gain,
            qualified_1250: self.qualified_1250,
            losses: crate::ScorpLossQbi {
                current: self.current,
                history: history.history.clone(),
                at_risk: history.at_risk.clone(),
                passive: history.passive.clone(),
            },
            complete,
        }
    }

    #[cfg(test)]
    fn from_record(r: &crate::ScorpK1) -> Self {
        let property = &r.property_qbi;
        Self {
            gain: property.gain,
            qualified_1250: property.qualified_1250,
            current: property.losses.current,
        }
    }

    // Whether any amount is entered; the K-1's own section 1231 items
    // decide whether amounts may be.
    fn present_field(&self) -> Option<&'static str> {
        [
            ("gain", self.gain),
            ("current", self.current),
            ("qualified_1250", self.qualified_1250.unwrap_or(0)),
        ]
        .into_iter()
        .find_map(|(field, amount)| (amount != 0).then_some(field))
    }
}

/// Other boxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode)]
pub(crate) struct K1ScorpOtherBoxes {
    /// Follow-up answers called for by the printed credit codes
    pub calls: K1ScorpCalls,
    /// Box 7 net short-term capital gain (loss)
    pub short: i64,
    /// Box 8a net long-term capital gain (loss)
    pub long: i64,
    /// Box 8b collectibles (28%) gain (loss)
    pub collectibles: i64,
    /// Box 8c part from selling a partnership interest
    pub unrecaptured_partnership: i64,
    /// Box 8c part from estates, trusts, mutual funds or REITs
    pub unrecaptured_fund: i64,
    /// Box 8c part from the corporation's business property
    pub unrecaptured_business: i64,
    /// Box 9 net section 1231 gain (loss)
    pub section1231: i64,
    /// Section 179 property the entity sold
    /// Each property on the fully taxable sale statement; use this tax system's depreciation. Do not include installment sales, exchanges, casualties or theft.
    /// ----
    /// Property
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Box 11 section 179 deduction
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// Your 2025 Form 4562 line 13 carryover of this corporation's section 179 amounts.
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    pub section179_carryover_sold: i64,
    /// Box 12 code A cash contributions (60%)
    pub cash_charity: i64,
    /// Other income — box 10
    /// Enter each code once, as printed. Codes not listed here belong among the other boxes below.
    /// ----
    /// Box 10 amount
    // Tax structure: one row per distinct Form 1120-S income code in the fixed catalog.
    pub income: Vec<crate::ScorpIncome>,
    /// Other deductions — box 12 codes other than A
    /// Enter each code once, as printed, including code I royalty deductions.
    /// ----
    /// Box 12 amount
    // Tax structure: one row per distinct Form 1120-S deduction code in the fixed catalog.
    pub deductions: Vec<crate::ScorpDeduction>,
    /// Box 13 code O backup withholding
    pub withholding: i64,
    /// Box 13 general business credits
    /// Enter the printed code, its amount and any Form 3800 line its statement requires. Codes H and AP through AU belong among the other boxes below. Code I has no 2026 amount.
    /// ----
    /// Credit
    pub credits: Vec<crate::K1CreditPage>,
    /// Investment credit property — box 13 code E and box 17 codes C, D and AV
    /// Enter each item from the K-1 and its statement. Choose automatic or enter your own completed Form3468 credit on that property row.
    /// ----
    /// Property
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Available automatic investment-credit confirmation question
    pub allowed_investment_confirmations: Vec<i32>,
    /// Investment credit property confirmed
    /// Needed only for properties using the automatic credit calculation above. No part of your share is financed by nonqualified nonrecourse financing (section 49) at your level; the statement shows no recapture, no low-income communities allocation, tax-exempt bond or subsidized financing reduction, elective payment or transfer; for rehabilitation, the building's certification is approved or timely pending. No leaves the automatic credits out; your own completed Form3468 amounts use their separate reduction answer.
    /// [NullText]
    /// No property uses the automatic credit calculation
    /// [SetText]
    /// The investment credit property facts are confirmed as
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    /// Low-income housing interest acquired after 1989
    /// Needed only when a low-income housing credit (code C or D) is entered above. Yes: you acquired your interest after 1989, so the credit is calculated with the Form 8582-CR post-1989 rules. No: you acquired it before 1990, which uses its own phase-out (Form 8582-CR Part III). Your records of when you acquired the interest show it.
    /// [NullText]
    /// No low-income housing credit is entered
    /// [SetText]
    /// I acquired the low-income housing interest after 1989, confirmed as
    pub housing_confirmed: Option<K1HousingAcquisition>,
    /// Box 15 code F private activity bond interest (included in box 16 code A)
    pub private_activity: i64,
    /// Box 16 code A tax-exempt interest
    pub exempt_interest: i64,
    /// Box 16 code B other tax-exempt income
    pub exempt: i64,
    /// Box 16 code C nondeductible expenses
    pub nondeductible: i64,
    /// Box 16 code F foreign taxes paid or accrued
    /// Your share of the corporation's foreign taxes. It reduces your stock basis. Any credit for these taxes is figured from the Schedule K-3, not from this code.
    pub foreign_taxes: i64,
    /// How long you have held your shares
    /// Needed when box 16 code D is above your stock basis or you sold your shares: the gain is long-term when held more than one year. If you bought shares within the year before, choose One year or less.
    pub stock_term: K1StockTerm,
    /// The corporation trades stocks, securities or commodities for its own account
    pub trading: bool,
    /// Other boxes with an amount
    /// ----
    /// Box with an amount
    pub unentered: Vec<crate::ScorpK1Item>,
    /// AMT section 179 income-limit carryover history
    /// From your own prior AMT Form 4562 records, independently of the regular carryover. Leave absent when unavailable; enter actual zero amounts when the AMT carryover is zero.
    /// [NullText]
    /// AMT carryover history is unavailable
    /// [SetText]
    /// Actual AMT carryover history, including an actual zero
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
}

/// How long you have held your shares
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum K1StockTerm {
    /// Not needed — no distribution above my basis and no sale
    NotNeeded,
    /// One year or less
    ShortTerm,
    /// More than one year
    LongTerm,
}

/// Your stock and debt basis (Form 7203) and amount at risk
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpBasis {
    /// When you became a shareholder
    #[libertas_default(Continuing)]
    pub start: K1ScorpStart,
    /// Capital you contributed in 2026 (Form 7203 line 2)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub additions: i64,
    /// Loans you made to the corporation (Form 7203 Part II)
    /// Their repayments in 2026 are box 16 code E.
    #[libertas_unordered]
    /// ----
    /// Loan
    pub loans: Vec<crate::ScorpLoan>,
    /// Amounts not at risk
    pub more_at_risk: K1ScorpAtRisk,
}

/// Your stock and debt basis (Form 7203) and amount at risk
// Records-row view of `K1ScorpBasis` without the start, which its own required
// strict question owns.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpBasisRecords {
    /// Capital you contributed in 2026 (Form 7203 line 2)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub additions: i64,
    /// Loans you made to the corporation (Form 7203 Part II)
    /// Their repayments in 2026 are box 16 code E.
    #[libertas_unordered]
    /// ----
    /// Loan
    pub loans: Vec<crate::ScorpLoan>,
    /// Amounts not at risk
    pub more_at_risk: K1ScorpAtRisk,
}

impl K1ScorpBasisRecords {
    pub(crate) fn from_stored(basis: &K1ScorpBasis) -> Self {
        Self {
            additions: basis.additions,
            loans: basis.loans.clone(),
            more_at_risk: basis.more_at_risk.clone(),
        }
    }

    pub(crate) fn stored(self, start: K1ScorpStart) -> K1ScorpBasis {
        K1ScorpBasis {
            start,
            additions: self.additions,
            loans: self.loans,
            more_at_risk: self.more_at_risk,
        }
    }
}

/// When you became a shareholder
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum K1ScorpStart {
    /// In 2026
    New,
    /// Before 2026
    Continuing {
        /// Stock basis at the beginning of 2026 (Form 7203 line 1)
        /// [NullText]
        /// Opening basis is not established yet
        /// [SetText]
        /// Established opening basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        opening: Option<i64>,
        /// Your at-risk history
        /// Needed only when this K-1 has a loss, a loss carried because of the at-risk limits, a box 11 section 179 deduction or earlier losses to recapture; otherwise leave the history missing.
        #[libertas_default(Unknown)]
        #[libertas_constrained_by("$.allowed_continuing_risk")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        carried: K1ScorpCarried,
    },
}

/// Losses carried from 2025
/// Leave every amount at zero when nothing was carried from 2025.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpCarried {
    /// Ordinary loss carried from 2025 because of your basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_loss: i64,
    /// Nondeductible expenses carried from 2025
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_nondeductible: i64,
    /// The election to deduct losses before nondeductible expenses applies
    #[libertas_default(false)]
    pub losses_first: bool,
    /// Prior short-term loss suspended by shareholder basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_short: i64,
    /// Prior total long-term loss suspended by shareholder basis
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_long: i64,
    /// Collectible portion of prior basis-suspended long-term loss
    /// Included in the preceding total, not an additional loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_collectibles: i64,
    /// Prior short-term loss suspended only by at-risk limits
    /// Already passed basis; do not include it in basis-suspended losses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_short: i64,
    /// Prior total long-term loss suspended only by at-risk limits
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_long: i64,
    /// Collectible portion of prior at-risk-suspended long-term loss
    /// Included in the preceding total and already passed basis limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_collectibles: i64,
    /// Prior section 1231 loss suspended by shareholder basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_basis_loss: i64,
    /// Prior section 1231 loss suspended only by at-risk limits
    /// Already passed basis limits. Exclude basis-suspended and passive-suspended amounts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_risk_loss: i64,
    /// Prior section 1231 loss suspended only by passive-activity limits
    /// Already passed basis and at-risk limits; not part of either preceding balance or the ordinary passive loss below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_passive_loss: i64,
    /// Prior ordinary loss suspended by at-risk limits
    /// Exclude basis-suspended and passive-suspended losses; this amount already passed basis limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_loss: i64,
    /// Prior nondeductible expenses suspended only by at-risk limits
    /// From the retained at-risk worksheet. These already reduced stock and debt basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_nondeductible: i64,
    /// Losses deducted in previous tax years beginning after 1978
    /// Retained cumulative recapture history for this activity; do not include the current year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// At-risk recapture income included in previous years
    /// Used only to cap further recapture; do not enter a current-year calculated amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub recaptured: i64,
    /// Ordinary loss carried from 2025 because of the passive activity rules
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub passive_loss: i64,
    /// Rental real estate loss carried from 2025 because of the passive activity rules
    /// The box 2 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_passive_loss: i64,
    /// Other rental loss carried from 2025 because of the passive activity rules
    /// The box 3 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_passive_loss: i64,
    /// Section 1231 loss of the rental real estate activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 2 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_section1231_passive_loss: i64,
    /// Section 1231 loss of the other rental activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 3 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_section1231_passive_loss: i64,

    /// Rental real estate loss carried from 2025 because of your basis
    /// The box 2 loss on your 2025 Form 7203 Part III line 36, column (e).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_basis_loss: i64,
    /// Rental real estate loss carried from 2025 because of the at-risk limits
    /// The box 2 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_risk_loss: i64,
    /// Other rental loss carried from 2025 because of your basis
    /// The box 3 loss on your 2025 Form 7203 Part III line 37, column (e).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_basis_loss: i64,
    /// Other rental loss carried from 2025 because of the at-risk limits
    /// The box 3 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_risk_loss: i64,
    /// Section 179 deduction carried from 2025 because of your basis
    /// Form 7203 Part III line 41, column (e). Not your Form 4562 line 13 carryover, which is entered with box 11.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_section179: i64,
    /// Cash contributions carried from 2025 because of your basis
    /// Form 7203 Part III line 42, column (e). Not a Schedule A carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_charity: i64,
    /// Foreign taxes carried from 2025 because of your basis
    /// Form 7203 Part III line 46, column (e).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_foreign_taxes: i64,
}

/// Amounts not at risk
/// Leave every amount at zero when all of your investment is at risk.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpAtRisk {
    /// New contributions and loan advances funded by amounts not at risk
    /// The portion financed with nonqualifying nonrecourse or related-party borrowing or protected against loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub excluded_funding: i64,
    /// Previously at-risk investment newly protected during 2026
    /// Only amounts already included in the opening investment. Exclude casualty and tort insurance.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub protected: i64,
    /// Previously protected investment becoming at risk during 2026
    /// Only previously excluded investment; do not repeat current contributions or loan advances.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub released: i64,
    /// Withdrawals and principal repayments returning amounts already excluded from at-risk investment
    /// Included in the distributions and loan repayments entered above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub excluded_returns: i64,
}

fn k1_sale_issue(
    sales: &[crate::K1Section179Sale],
    carryover: i64,
    sold: i64,
    alternative: bool,
    require_answers: bool,
) -> Option<ValidationIssue> {
    sales
        .iter()
        .enumerate()
        .find_map(|(index, sale)| {
            sale.issue().map(|issue| {
                let issue = if alternative {
                    issue.map_field_prefix(&["regular"], &["alternative", "amounts"])
                } else {
                    issue
                };
                issue.in_list("section179_sales", index).in_field("more")
            })
        })
        .or_else(|| {
            require_answers
                .then(|| {
                    crate::K1Section179Sale::carryover_issue(sales, carryover, sold).map(|issue| {
                        issue
                            .map_field_prefix(&["sold"], &["section179_carryover_sold"])
                            .in_field("more")
                    })
                })
                .flatten()
        })
}

fn k1_source_admission_issue(
    history: Option<&crate::Section179IncomeCarryHistory>,
    period: Option<&crate::K1EntityTaxPeriod>,
    sales: &[crate::K1Section179Sale],
    same_as_regular: bool,
    separate: bool,
    carryover_and_sold: (i64, i64),
    require_answers: bool,
) -> Option<ValidationIssue> {
    let (carryover, sold) = carryover_and_sold;
    if let Some(issue) = history.and_then(|history| history.issue()) {
        return Some(issue.in_field("amt_section179_income_carry").in_field("more"));
    }

    if period.is_some_and(|period| !period.valid()) {
        return Some(ValidationIssue::new("K1_ENTITY_PERIOD_DATES").at_field("reporting_period"));
    }

    if !crate::K1Section179Sale::valid_source_counts(sales) {
        return Some(
            ValidationIssue::new("ISSUE_K1_SALE_RECORDS")
                .at_field("section179_sales")
                .in_field("more"),
        );
    }
    if require_answers
        && same_as_regular
        && sales.iter().any(|sale| {
            sale.alternative.reported_amounts().is_some()
                && !sale.alternative.matches_known_regular(&sale.regular)
        })
    {
        return Some(ValidationIssue::new("K1_SALE_AMT_CHOICE").at_field("amt"));
    }
    if (separate || !require_answers)
        && let Ok(sales) = crate::K1Section179Sale::views(sales, true)
        && let Some(issue) = k1_sale_issue(&sales, carryover, sold, true, require_answers)
    {
        return Some(issue);
    }
    if let Some(issue) = k1_sale_issue(sales, carryover, sold, false, require_answers) {
        return Some(issue);
    }
    None
}

impl K1Scorp {
    pub(crate) fn qbi_records_record(&self) -> K1ScorpQbiRecords {
        self.qbi_records.clone().unwrap_or_else(|| K1ScorpQbiRecords {
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
        })
    }
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the server twin of the schema rules).
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, source_alignment: bool) -> Option<ValidationIssue> {
        self.issue_with_modes(source_alignment, source_alignment)
    }
    pub(crate) fn topic_issue(&self, require_answers: bool) -> Option<ValidationIssue> {
        self.issue_with_modes(require_answers, true)
    }
    fn issue_with_modes(
        &self,
        require_answers: bool,
        source_alignment: bool,
    ) -> Option<ValidationIssue> {
        if require_answers && self.disposition.is_none() {
            return Some(ValidationIssue::new("DOCUMENT_K1_DISPOSITION").at_field("disposition"));
        }
        if require_answers
            && self.record().rental_participation_needed()
            && self.rentals.participation.is_none()
        {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_RENTAL_RECORDS")
                    .at_field("participation")
                    .in_field("rentals"),
            );
        }

        if require_answers
            && matches!(self.basis.start, K1ScorpStart::Continuing { opening: None, .. })
        {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS")
                    .at_field("opening")
                    .in_field("start")
                    .in_field("basis"),
            );
        }
        if let Some(issue) = k1_source_admission_issue(
            self.more.amt_section179_income_carry.as_ref(),
            self.reporting_period.as_ref(),
            &self.more.section179_sales,
            matches!(self.amt, Some(ScorpAmtRecords::SameAsRegular)),
            matches!(self.amt, Some(ScorpAmtRecords::Separate { .. })),
            (self.more.section179_carryover, self.more.section179_carryover_sold),
            source_alignment,
        ) {
            return Some(issue);
        }
        if let Some(index) = self
            .rentals
            .disposed
            .iter()
            .enumerate()
            .find_map(|(i, v)| self.rentals.disposed[..i].contains(v).then_some(i))
        {
            return Some(
                ValidationIssue::new("K1_RENTAL_DISPOSED_TWICE")
                    .at_list_item("disposed", index)
                    .in_field("rentals"),
            );
        }
        if self.qualified_dividends > self.dividends {
            return Some(
                ValidationIssue::new("SCORP_QUALIFIED_ABOVE_DIVIDENDS")
                    .at_field("qualified_dividends"),
            );
        }
        if let crate::K3Attached::Attached(k3) = &self.schedule_k3
            && let Some(issue) = k3.issue_with_mode(require_answers)
        {
            return Some(issue.in_field("schedule_k3"));
        }
        if let K1ScorpQbi::Statement { income, .. } = &self.qbi
            && *income > 0
            && *income > self.ordinary
        {
            return Some(
                ValidationIssue::new("SCORP_QBI_ABOVE_BOX_1").at_field("income").in_field("qbi"),
            );
        }
        {
            let K1ScorpOtherBoxes {
                private_activity,
                exempt_interest,
                unentered,
                credits,
                allowed_investment_confirmations: _,
                investment_property,
                investment_confirmed,
                housing_confirmed,
                ..
            } = &self.more;
            if let Some(issue) = crate::K1Credit::issue_list_with_mode(
                &credits.iter().cloned().map(Into::into).collect::<Vec<_>>(),
                require_answers,
            ) {
                return Some(issue.in_field("more"));
            }
            if require_answers
                && investment_property.iter().any(|p| p.own_form().is_none())
                && investment_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_INVESTMENT_NEEDS_CONFIRMATION")
                        .at_field("investment_confirmed")
                        .in_field("more"),
                );
            }
            if require_answers
                && credits.iter().any(|credit| credit.code.housing())
                && housing_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_HOUSING_NEEDS_CONFIRMATION")
                        .at_field("housing_confirmed")
                        .in_field("more"),
                );
            }
            if private_activity > exempt_interest {
                return Some(
                    ValidationIssue::new("SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                        .at_field("private_activity")
                        .in_field("more"),
                );
            }
            if let Some(index) = unentered
                .iter()
                .enumerate()
                .find_map(|(i, item)| unentered[..i].contains(item).then_some(i))
            {
                return Some(
                    ValidationIssue::new("SCORP_BOX_TWICE")
                        .at_list_item("unentered", index)
                        .in_field("more"),
                );
            }
            if let Some(issue) = crate::ScorpIncome::issue(&self.more.income, "income")
                .or_else(|| crate::ScorpDeduction::issue(&self.more.deductions, "deductions"))
            {
                return Some(issue.in_field("more"));
            }
            if let Some(issue) = crate::ScorpSpecial::issue_list(&self.special) {
                return Some(issue);
            }
            if source_alignment && self.final_k1 && self.disposition == Some(ScorpDisposition::Kept)
            {
                return Some(
                    ValidationIssue::new("K1_FINAL_NEEDS_DISPOSITION").at_field("disposition"),
                );
            }
            if matches!(self.disposition, Some(ScorpDisposition::Sold { .. }))
                && self.more.stock_term == K1StockTerm::NotNeeded
            {
                return Some(
                    ValidationIssue::new("K1_SALE_NEEDS_HOLDING_PERIOD")
                        .at_field("stock_term")
                        .in_field("more"),
                );
            }
        }
        if let K1ScorpStart::Continuing {
            carried:
                K1ScorpCarried {
                    basis_nondeductible,
                    losses_first,
                    basis_long,
                    basis_collectibles,
                    risk_long,
                    risk_collectibles,
                    ..
                },
            ..
        } = &self.basis.start
        {
            if *basis_nondeductible > 0 && !losses_first {
                return Some(
                    ValidationIssue::new("SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION")
                        .at_field("losses_first")
                        .in_field("carried")
                        .in_field("start")
                        .in_field("basis"),
                );
            }
            for (field, part, total) in [
                ("basis_collectibles", basis_collectibles, basis_long),
                ("risk_collectibles", risk_collectibles, risk_long),
            ] {
                if part > total {
                    return Some(
                        ValidationIssue::new("SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG")
                            .at_field(field)
                            .in_field("carried")
                            .in_field("start")
                            .in_field("basis"),
                    );
                }
            }
        }
        if let Some(index) = self.basis.loans.iter().position(|loan| !loan.term_matches()) {
            return Some(
                ValidationIssue::new("SCORP_LOAN_REPAYMENT_TERM")
                    .at_field("repayment_term")
                    .in_list("loans", index)
                    .in_field("basis"),
            );
        }
        // Source owners return their own failure metadata; this adapter only
        // projects accepted source names to the actual editable Page fields.
        if let K1ScorpQbi::Statement { property_qbi, .. } = &self.qbi {
            let record = self.record();
            if let Some(issue) = record.loss_qbi.records_issue(
                record.ordinary.saturating_neg().max(0),
                [record.prior_loss, record.risk.prior_loss, record.passive.loss],
            ) && (source_alignment || issue.has_field_prefix(&["current"]))
            {
                return Some(
                    issue
                        .map_field_prefix(&["current"], &["qbi", "current_loss"])
                        .map_field_prefix(&["history"], &["qbi_records", "ordinary", "history"])
                        .map_field_prefix(&["at_risk"], &["qbi_records", "ordinary", "at_risk"])
                        .map_field_prefix(&["passive"], &["qbi_records", "ordinary", "passive"]),
                );
            }
            let this_year =
                record.section1231.as_ref().is_some_and(|v| v.net != 0 || v.unrecaptured != 0);
            if !this_year && let Some(field) = property_qbi.present_field() {
                return Some(
                    ValidationIssue::new("SCORP_QBI_PROPERTY_RECORDS_NEEDED")
                        .at_field(field)
                        .in_field("property_qbi")
                        .in_field("qbi"),
                );
            }
            if let Some(issue) = record.property_qbi.records_issue(&record)
                && (source_alignment
                    || issue.has_field_prefix(&["gain"])
                    || issue.has_field_prefix(&["qualified_1250"])
                    || issue.has_field_prefix(&["losses", "current"]))
            {
                return Some(
                    issue
                        .map_field_prefix(
                            &["losses", "current"],
                            &["qbi", "property_qbi", "current"],
                        )
                        .map_field_prefix(
                            &["losses", "history"],
                            &["qbi_records", "property", "history"],
                        )
                        .map_field_prefix(
                            &["losses", "at_risk"],
                            &["qbi_records", "property", "at_risk"],
                        )
                        .map_field_prefix(
                            &["losses", "passive"],
                            &["qbi_records", "property", "passive"],
                        )
                        .map_field_prefix(&["gain"], &["qbi", "property_qbi", "gain"])
                        .map_field_prefix(
                            &["qualified_1250"],
                            &["qbi", "property_qbi", "qualified_1250"],
                        ),
                );
            }
        }
        // A shareholder before 2026 has no New at-risk
        // start, the twin of PARTNER_RISK_HISTORY_NEW.
        if let K1ScorpStart::Continuing { risk_history: crate::ScorpRiskStart::New, .. } =
            &self.basis.start
        {
            return Some(
                ValidationIssue::new("SCORP_RISK_HISTORY_NEW")
                    .at_field("risk_history")
                    .in_field("start")
                    .in_field("basis"),
            );
        }
        None
    }

    // The stored record this page writes.
    pub(crate) fn record(&self) -> crate::ScorpK1 {
        // Only a situation that leaves the whole K-1 out of the estimate
        // makes the basis and financing confirmations No.
        let complete = !self.special.iter().any(|situation| situation.whole());
        let qbi_records = self.qbi_records_record();
        let mut risk = crate::ScorpRisk {
            start: crate::ScorpRiskStart::New,
            excluded_funding: 0,
            protected: 0,
            released: 0,
            excluded_returns: 0,
            prior_loss: 0,
            prior_nondeductible: 0,
            deducted: 0,
            recaptured: 0,
            complete: self.exact.risk_complete.unwrap_or(complete),
        };
        let mut passive = crate::ScorpPassiveHistory {
            loss: 0,
            complete: self.exact.passive_complete.unwrap_or(complete),
        };
        let (rentals, rental_facts) = self.rentals.record([0, 0], [0, 0]);
        let mut r = crate::ScorpK1 {
            amt_record_choice: None,
            research_credit_line: None,
            amt_section179_income_carry: self.more.amt_section179_income_carry.clone(),
            reporting_period: self.reporting_period,
            qbi_aggregation_group: crate::common::optional_text(&self.qbi_group),
            health_insurance: crate::BusinessHealthInsurance::None,
            section179_sales: Vec::new(),
            ordinary: self.ordinary,
            rentals,
            rental_facts,
            interest: self.interest,
            dividends: self.dividends,
            qualified_dividends: self.qualified_dividends,
            royalties: self.royalties,
            capital: None,
            section1231: None,
            withholding: 0,
            trade_kind: crate::ScorpTradeKind::Ordinary,
            opening: Some(0),
            additions: self.basis.additions,
            exempt_interest: 0,
            private_activity: 0,
            exempt: 0,
            distributions: self.distributions,
            nondeductible: 0,
            prior_loss: 0,
            prior_nondeductible: 0,
            losses_first: false,
            loans: self.basis.loans.clone(),
            risk: risk.clone(),
            participation: self.participation.as_ref().map(Participation::record),
            passive: passive.clone(),
            loss_qbi: crate::ScorpLossQbi {
                current: 0,
                history: qbi_records.ordinary.history.clone(),
                at_risk: qbi_records.ordinary.at_risk.clone(),
                passive: qbi_records.ordinary.passive.clone(),
            },
            property_qbi: crate::ScorpPropertyQbi {
                losses: crate::ScorpLossQbi {
                    current: 0,
                    history: qbi_records.property.history.clone(),
                    at_risk: qbi_records.property.at_risk.clone(),
                    passive: qbi_records.property.passive.clone(),
                },
                complete: qbi_records.property_complete,
                ..crate::ScorpPropertyQbi::default()
            },
            qbi_statement: match &self.qbi {
                K1ScorpQbi::NoStatement => None,
                K1ScorpQbi::Statement {
                    income,
                    wages,
                    property,
                    service,
                    reit_dividends,
                    ptp_income,
                    ..
                } => Some(crate::ScorpQbiStatement {
                    income: *income,
                    wages: *wages,
                    property: *property,
                    service: *service,
                    complete: !self.exact.statement_unconfirmed,
                    reit_dividends: *reit_dividends,
                    ptp_income: *ptp_income,
                }),
            },
            amt: None,
            carried: crate::K1LossCarryforwards::default(),
            cash_charity: 0,
            section179: 0,
            section179_carryover: 0,
            section179_carryover_sold: 0,
            foreign_taxes: 0,
            schedule_k3: match &self.schedule_k3 {
                crate::K3Attached::NotAttached => None,
                crate::K3Attached::Attached(k3) => Some(k3.pruned()),
            },
            stock_term: None,
            credits: Vec::new(),
            investment_property: Vec::new(),
            investment_confirmed: None,
            housing_confirmed: None,
            unentered: Vec::new(),
            final_k1: self.final_k1,
            income: Vec::new(),
            deductions: Vec::new(),
            disposition: self.disposition.as_ref().map(ScorpDisposition::record),
            special: self.special.clone(),
        };
        if let K1ScorpQbi::Statement { current_loss, property_qbi, .. } = &self.qbi {
            r.loss_qbi.current = *current_loss;
            r.property_qbi =
                property_qbi.record(&qbi_records.property, qbi_records.property_complete);
        } else {
            r.loss_qbi.current = self.exact.unstated_loss;
            r.property_qbi = K1ScorpPropertyQbi {
                gain: self.exact.unstated_property_gain,
                qualified_1250: self.exact.unstated_qualified_1250,
                current: self.exact.unstated_property_loss,
            }
            .record(&qbi_records.property, qbi_records.property_complete);
        }
        // Current portfolio capital and section 1231 amounts (Other boxes)
        // and their carried losses (the basis fold).
        let mut capital = crate::ScorpCapital {
            short: 0,
            long: 0,
            collectibles: 0,
            unrecaptured_partnership: 0,
            unrecaptured_fund: 0,
            basis_short: 0,
            basis_long: 0,
            basis_collectibles: 0,
            risk_short: 0,
            risk_long: 0,
            risk_collectibles: 0,
        };
        let mut section1231 = crate::K1Section1231 {
            net: 0,
            unrecaptured: 0,
            basis_loss: 0,
            risk_loss: 0,
            passive_loss: 0,
        };
        {
            let K1ScorpOtherBoxes {
                calls: _,
                short,
                long,
                collectibles,
                unrecaptured_partnership,
                unrecaptured_fund,
                unrecaptured_business,
                section1231: net,
                section179,
                section179_carryover,
                section179_carryover_sold,
                amt_section179_income_carry: _,
                section179_sales,
                cash_charity,
                withholding,
                credits,
                allowed_investment_confirmations: _,
                investment_property,
                investment_confirmed,
                housing_confirmed,
                private_activity,
                exempt_interest,
                exempt,
                nondeductible,
                foreign_taxes,
                stock_term,
                trading,
                unentered,
                income,
                deductions,
            } = &self.more;
            r.income = income.clone();
            r.deductions = deductions.clone();
            capital.short = *short;
            capital.long = *long;
            capital.collectibles = *collectibles;
            capital.unrecaptured_partnership = *unrecaptured_partnership;
            capital.unrecaptured_fund = *unrecaptured_fund;
            section1231.net = *net;
            section1231.unrecaptured = *unrecaptured_business;
            r.section179 = *section179;
            r.section179_carryover = *section179_carryover;
            r.section179_carryover_sold = *section179_carryover_sold;
            r.section179_sales = section179_sales.clone();
            r.cash_charity = *cash_charity;
            r.withholding = *withholding;
            r.credits = credits.iter().cloned().map(Into::into).collect();
            r.investment_property = investment_property.clone();
            r.investment_confirmed = investment_confirmed.map(K1InvestmentConfirmation::confirmed);
            r.housing_confirmed = housing_confirmed.map(K1HousingAcquisition::after_1989);
            r.private_activity = *private_activity;
            r.exempt_interest = *exempt_interest;
            r.exempt = *exempt;
            r.nondeductible = *nondeductible;
            r.foreign_taxes = *foreign_taxes;
            r.stock_term = match stock_term {
                K1StockTerm::NotNeeded => None,
                K1StockTerm::ShortTerm => Some(crate::GainTerm::Short),
                K1StockTerm::LongTerm => Some(crate::GainTerm::Long),
            };
            if *trading {
                r.trade_kind = crate::ScorpTradeKind::FinancialTrading;
            }
            r.unentered = unentered.clone();
        }
        if let K1ScorpStart::Continuing { opening, risk_history, .. } = &self.basis.start {
            r.opening = *opening;
            risk.start = risk_history.clone();
        }
        if let K1ScorpStart::Continuing {
            carried:
                K1ScorpCarried {
                    basis_loss,
                    basis_nondeductible,
                    losses_first,
                    basis_short,
                    basis_long,
                    basis_collectibles,
                    risk_short,
                    risk_long,
                    risk_collectibles,
                    section1231_basis_loss,
                    section1231_risk_loss,
                    section1231_passive_loss,
                    risk_loss,
                    risk_nondeductible,
                    deducted,
                    recaptured,
                    passive_loss,
                    rental_real_estate_passive_loss,
                    other_rental_passive_loss,
                    rental_real_estate_section1231_passive_loss,
                    other_rental_section1231_passive_loss,
                    rental_real_estate_basis_loss,
                    rental_real_estate_risk_loss,
                    other_rental_basis_loss,
                    other_rental_risk_loss,
                    basis_section179,
                    basis_charity,
                    basis_foreign_taxes,
                },
            ..
        } = &self.basis.start
        {
            r.rentals.real_estate_carryover = *rental_real_estate_passive_loss;
            r.rentals.other_carryover = *other_rental_passive_loss;
            r.rentals.real_estate_property.passive_loss =
                *rental_real_estate_section1231_passive_loss;
            r.rentals.other_property.passive_loss = *other_rental_section1231_passive_loss;
            r.carried = crate::K1LossCarryforwards {
                real_estate: crate::K1LossCarry {
                    basis: *rental_real_estate_basis_loss,
                    risk: *rental_real_estate_risk_loss,
                },
                other_rental: crate::K1LossCarry {
                    basis: *other_rental_basis_loss,
                    risk: *other_rental_risk_loss,
                },
                section179: *basis_section179,
                charity: *basis_charity,
                foreign_taxes: *basis_foreign_taxes,
            };
            r.prior_loss = *basis_loss;
            r.prior_nondeductible = *basis_nondeductible;
            r.losses_first = *losses_first;
            capital.basis_short = *basis_short;
            capital.basis_long = *basis_long;
            capital.basis_collectibles = *basis_collectibles;
            capital.risk_short = *risk_short;
            capital.risk_long = *risk_long;
            capital.risk_collectibles = *risk_collectibles;
            section1231.basis_loss = *section1231_basis_loss;
            section1231.risk_loss = *section1231_risk_loss;
            section1231.passive_loss = *section1231_passive_loss;
            risk.prior_loss = *risk_loss;
            risk.prior_nondeductible = *risk_nondeductible;
            risk.deducted = *deducted;
            risk.recaptured = *recaptured;
            passive.loss = *passive_loss;
        }
        let at_risk = &self.basis.more_at_risk;
        risk.excluded_funding = at_risk.excluded_funding;
        risk.protected = at_risk.protected;
        risk.released = at_risk.released;
        risk.excluded_returns = at_risk.excluded_returns;
        let capital_items = [
            capital.short,
            capital.long,
            capital.collectibles,
            capital.unrecaptured_partnership,
            capital.unrecaptured_fund,
            capital.basis_short,
            capital.basis_long,
            capital.basis_collectibles,
            capital.risk_short,
            capital.risk_long,
            capital.risk_collectibles,
        ];
        if self.exact.capital_present || capital_items.iter().any(|v| *v != 0) {
            r.capital = Some(capital);
        }
        if self.exact.section1231_present || section1231.has_items() {
            r.section1231 = Some(section1231);
        }
        r.risk = risk;
        r.passive = passive;
        if matches!(self.amt, Some(ScorpAmtRecords::SameAsRegular)) {
            for sale in &mut r.section179_sales {
                sale.apply_parent_same_as_regular();
            }
        }
        r.amt = self.amt.as_ref().and_then(|amt| amt.record(&r));
        r
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written.
    #[cfg(test)]
    pub(crate) fn from_record(name: &str, r: &crate::ScorpK1) -> Option<Self> {
        let capital = r.capital.clone().unwrap_or(crate::ScorpCapital {
            short: 0,
            long: 0,
            collectibles: 0,
            unrecaptured_partnership: 0,
            unrecaptured_fund: 0,
            basis_short: 0,
            basis_long: 0,
            basis_collectibles: 0,
            risk_short: 0,
            risk_long: 0,
            risk_collectibles: 0,
        });
        let section1231 = r.section1231.clone().unwrap_or(crate::K1Section1231 {
            net: 0,
            unrecaptured: 0,
            basis_loss: 0,
            risk_loss: 0,
            passive_loss: 0,
        });
        let risk = r.risk.clone();
        let passive = r.passive.clone();
        let mut regular = r.clone();
        regular.amt = None;
        regular.amt_record_choice = None;
        regular.research_credit_line = None;
        let mut page = Self {
            corporation: crate::PrintedParty::from_name(name.into()),
            shareholder: Default::default(),
            qbi_group: r.qbi_aggregation_group.clone().unwrap_or_default(),
            reporting_period: r.reporting_period,
            final_k1: r.final_k1,
            ordinary: r.ordinary,
            rentals: K1RentalBoxes::from_record(&r.rentals, &r.rental_facts),
            interest: r.interest,
            dividends: r.dividends,
            qualified_dividends: r.qualified_dividends,
            royalties: r.royalties,
            distributions: r.distributions,
            qbi: match &r.qbi_statement {
                None => K1ScorpQbi::NoStatement,
                Some(statement) => K1ScorpQbi::Statement {
                    income: statement.income,
                    wages: statement.wages,
                    property: statement.property,
                    service: statement.service,
                    current_loss: r.loss_qbi.current,
                    property_qbi: K1ScorpPropertyQbi::from_record(r),
                    reit_dividends: statement.reit_dividends,
                    ptp_income: statement.ptp_income,
                },
            },
            qbi_records: Some(K1ScorpQbiRecords {
                ordinary: K1ScorpLossHistory {
                    history: r.loss_qbi.history.clone(),
                    at_risk: r.loss_qbi.at_risk.clone(),
                    passive: r.loss_qbi.passive.clone(),
                },
                property: K1ScorpLossHistory {
                    history: r.property_qbi.losses.history.clone(),
                    at_risk: r.property_qbi.losses.at_risk.clone(),
                    passive: r.property_qbi.losses.passive.clone(),
                },
                property_complete: r.property_qbi.complete,
            }),
            participation: r.participation.as_ref().map(Participation::from_record),
            more: K1ScorpOtherBoxes {
                calls: K1ScorpCalls::default(),
                short: capital.short,
                long: capital.long,
                collectibles: capital.collectibles,
                unrecaptured_partnership: capital.unrecaptured_partnership,
                unrecaptured_fund: capital.unrecaptured_fund,
                unrecaptured_business: section1231.unrecaptured,
                section1231: section1231.net,
                section179: r.section179,
                section179_carryover: r.section179_carryover,
                section179_carryover_sold: r.section179_carryover_sold,
                amt_section179_income_carry: r.amt_section179_income_carry.clone(),
                section179_sales: r.section179_sales.clone(),
                cash_charity: r.cash_charity,
                withholding: r.withholding,
                credits: r.credits.iter().cloned().map(Into::into).collect(),
                allowed_investment_confirmations: investment_choices(&r.investment_property),
                investment_property: r.investment_property.clone(),
                investment_confirmed: r.investment_confirmed.map(K1InvestmentConfirmation::from),
                housing_confirmed: r.housing_confirmed.map(K1HousingAcquisition::from),
                private_activity: r.private_activity,
                exempt_interest: r.exempt_interest,
                exempt: r.exempt,
                nondeductible: r.nondeductible,
                foreign_taxes: r.foreign_taxes,
                stock_term: match r.stock_term {
                    None => K1StockTerm::NotNeeded,
                    Some(crate::GainTerm::Short) => K1StockTerm::ShortTerm,
                    Some(crate::GainTerm::Long) => K1StockTerm::LongTerm,
                },
                trading: r.trade_kind == crate::ScorpTradeKind::FinancialTrading,
                unentered: r.unentered.clone(),
                income: r.income.clone(),
                deductions: r.deductions.clone(),
            },
            schedule_k3: match &r.schedule_k3 {
                None => crate::K3Attached::NotAttached,
                Some(k3) => crate::K3Attached::Attached(k3.clone()),
            },
            basis: K1ScorpBasis {
                start: K1ScorpStart::Continuing {
                    opening: r.opening,
                    risk_history: risk.start.clone(),
                    carried: K1ScorpCarried {
                        basis_loss: r.prior_loss,
                        basis_nondeductible: r.prior_nondeductible,
                        losses_first: r.losses_first,
                        basis_short: capital.basis_short,
                        basis_long: capital.basis_long,
                        basis_collectibles: capital.basis_collectibles,
                        risk_short: capital.risk_short,
                        risk_long: capital.risk_long,
                        risk_collectibles: capital.risk_collectibles,
                        section1231_basis_loss: section1231.basis_loss,
                        section1231_risk_loss: section1231.risk_loss,
                        section1231_passive_loss: section1231.passive_loss,
                        risk_loss: risk.prior_loss,
                        risk_nondeductible: risk.prior_nondeductible,
                        deducted: risk.deducted,
                        recaptured: risk.recaptured,
                        passive_loss: passive.loss,
                        rental_real_estate_passive_loss: r.rentals.real_estate_carryover,
                        other_rental_passive_loss: r.rentals.other_carryover,
                        rental_real_estate_section1231_passive_loss: r
                            .rentals
                            .real_estate_property
                            .passive_loss,
                        other_rental_section1231_passive_loss: r
                            .rentals
                            .other_property
                            .passive_loss,
                        rental_real_estate_basis_loss: r.carried.real_estate.basis,
                        rental_real_estate_risk_loss: r.carried.real_estate.risk,
                        other_rental_basis_loss: r.carried.other_rental.basis,
                        other_rental_risk_loss: r.carried.other_rental.risk,
                        basis_section179: r.carried.section179,
                        basis_charity: r.carried.charity,
                        basis_foreign_taxes: r.carried.foreign_taxes,
                    },
                },
                additions: r.additions,
                loans: r.loans.clone(),
                more_at_risk: K1ScorpAtRisk {
                    excluded_funding: risk.excluded_funding,
                    protected: risk.protected,
                    released: risk.released,
                    excluded_returns: risk.excluded_returns,
                },
            },
            amt: r.amt.as_ref().map(|amt| ScorpAmtRecords::from_record(&regular, amt)),
            disposition: match &r.disposition {
                Some(value) => Some(ScorpDisposition::from_record(value)?),
                None => None,
            },
            special: r.special.clone(),
            exact: ScorpRecordFacts {
                capital_present: r.capital.as_ref().is_some_and(|c| {
                    [
                        c.short,
                        c.long,
                        c.collectibles,
                        c.unrecaptured_partnership,
                        c.unrecaptured_fund,
                        c.basis_short,
                        c.basis_long,
                        c.basis_collectibles,
                        c.risk_short,
                        c.risk_long,
                        c.risk_collectibles,
                    ]
                    .iter()
                    .all(|v| *v == 0)
                }),
                section1231_present: r.section1231.as_ref().is_some_and(|v| !v.has_items()),
                unstated_loss: if r.qbi_statement.is_none() { r.loss_qbi.current } else { 0 },
                unstated_property_gain: if r.qbi_statement.is_none() {
                    r.property_qbi.gain
                } else {
                    0
                },
                unstated_qualified_1250: if r.qbi_statement.is_none() {
                    r.property_qbi.qualified_1250
                } else {
                    None
                },
                unstated_property_loss: if r.qbi_statement.is_none() {
                    r.property_qbi.losses.current
                } else {
                    0
                },
                statement_unconfirmed: r.qbi_statement.as_ref().is_some_and(|s| !s.complete),
                risk_complete: (r.risk.complete
                    == r.special.iter().any(|situation| situation.whole()))
                .then_some(r.risk.complete),
                passive_complete: (r.passive.complete
                    == r.special.iter().any(|situation| situation.whole()))
                .then_some(r.passive.complete),
            },
        };
        page.more.calls = K1ScorpCalls::from_credits(
            &page.more.credits.iter().map(K1CreditCallBox::from).collect::<Vec<_>>(),
        );
        if r.section179_sales
            .iter()
            .any(|sale| !sale.alternative.matches_known_regular(&sale.regular))
        {
            page.amt = r.amt.clone().map(|record| ScorpAmtRecords::Separate { record });
        } else if r.amt_record_choice != Some(true)
            && matches!(page.amt, Some(ScorpAmtRecords::SameAsRegular))
        {
            for sale in &mut page.more.section179_sales {
                sale.alternative = crate::K1Section179SaleAmtSource::NotReported;
            }
        }
        // A shareholder from 2026 reopens as one whenever it writes the same
        // record.
        let full = page.record();
        let mut shorter = page.clone();
        shorter.basis.start = K1ScorpStart::New;
        if shorter.record() == full {
            page = shorter;
        }
        if r.amt_record_choice == Some(true) {
            page.amt = r.amt.clone().map(|record| ScorpAmtRecords::Separate { record });
        }
        let mut written = page.record();
        written.health_insurance = r.health_insurance.clone();
        written.amt_record_choice = r.amt_record_choice;
        written.research_credit_line = r.research_credit_line;
        (written == *r).then_some(page)
    }
}

// Transient adapter for the partnership calculation owner and its existing rules.

/// Transient inputs for the existing partnership calculation owner.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode)]
pub(crate) struct K1Partnership {
    /// Printed recipient and issuer names
    /// [NullText]
    /// No printed names entered
    /// [SetText]
    /// Printed names
    pub printed: Option<crate::PrintedDocumentNames>,
    /// Entity's reporting tax period
    /// Use this K-1's actual beginning and ending dates. Leave unknown when the source does not establish them.
    /// [NullText]
    /// The entity's tax period is unknown
    /// [SetText]
    /// The entity's actual tax period is
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Partnership's name (item B)
    pub name: String,
    /// Publicly traded partnership (item D)
    /// Its section 199A statement reports current-year qualified income or loss and the actual specified-service classification, with no wages or unadjusted property basis. Current specified-service income limits apply before eligible amounts join the separate REIT/PTP component. Historical qualified-loss statements and suspended basis/at-risk/passive balances still need their source worksheet.
    pub publicly_traded: bool,
    /// Final K-1
    /// The box at the top of the K-1.
    pub final_k1: bool,
    /// Your section 469 participation records
    /// [NullText]
    /// Participation is not established yet
    /// [SetText]
    /// Participation is
    pub participation: Option<crate::PartnerParticipation>,
    /// Limited partner for self-employment tax
    /// [NullText]
    /// Self-employment classification is not established yet
    /// [SetText]
    /// Limited-partner treatment for self-employment tax applies
    pub limited_for_se: Option<bool>,
    /// Your share of liabilities at the beginning of the year (item K1)
    /// Add the nonrecourse, qualified nonrecourse and recourse amounts.
    pub liabilities_start: i64,
    /// Your share of liabilities at the end of the year (item K1)
    /// The same three amounts.
    pub liabilities_end: i64,
    /// Box 1 ordinary business income (loss)
    pub ordinary: i64,
    /// Rental income (loss) — boxes 2 and 3
    pub rentals: K1RentalBoxes,
    /// Box 4a guaranteed payments for services
    pub services: i64,
    /// Box 4b guaranteed payments for capital
    pub capital: i64,
    /// Box 14 code A net earnings (loss) from self-employment
    /// Enter the amount shown, even zero.
    pub se: i64,
    /// Box 14 code B or C gross income
    pub se_gross: SeGross,
    /// Box 19 codes A and F cash distributions
    /// Cash only, including cash for services (code F); a distribution of marketable securities belongs under Something else applies.
    pub distributions: i64,
    /// Section 199A information (box 20 code Z)
    pub qbi: PartnerQbi,
    /// Your retained QBI records and gross-income allocation
    /// From your own worksheets. Document import preserves these records when the printed statement changes.
    /// [NullText]
    /// No retained personal records were supplied with this source.
    /// [SetText]
    /// Retained personal records
    pub qbi_records: Option<crate::partnership_qbi::PartnerQbiRecords>,
    /// Other boxes
    pub more: K1PartnershipOtherBoxes,
    /// Schedule K-3 (box 16)
    /// Box 16 is checked when the partnership attached a Schedule K-3. Without one, the foreign taxes in box 21 get no credit here; you can ask the partnership for the schedule.
    pub schedule_k3: crate::K3Attached,
    /// Your basis and amount at risk
    /// Your own records, not item L's capital account.
    pub basis: PartnerBasis,
    /// What produces this income
    /// Asked only when the tax on a child's unearned income (Form 8615) or the standard deduction for a filer someone can claim as a dependent can apply to this return.
    pub income_source: K1IncomeSource,
    /// Alternative minimum tax records
    pub amt: Option<PartnerAmtRecords>,
    /// What happened to your interest in 2026
    /// [NullText]
    /// What happened to this interest is not answered yet
    /// [SetText]
    /// What happened to this interest is
    pub disposition: Option<PartnerDisposition>,
    /// Something else applies
    /// Choose each that applies; leave empty when none does.
    /// ----
    /// Situation
    // Tax structure: one row per distinct partnership special situation in the fixed catalog.
    pub special: Vec<crate::PartnerSpecial>,
    /// Individual QBI aggregation group
    /// Enter the same group name on each actual business in your individual election. Leave empty when no group is selected. This is separate from any aggregate reported by the entity. Save a new source first, then reopen it before selecting a group.
    pub qbi_group: alloc::string::String,
}

/// What happened to your interest in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PartnerDisposition {
    /// You still own all of it
    Kept,
    /// You sold or exchanged all of it
    /// To a buyer who isn't related to you, all reported this year (not an installment sale). For a publicly traded partnership sold through a broker, enter the Form 1099-B proceeds here and don't enter that Form 1099-B among investment sales.
    Sold {
        /// What you received
        /// Cash plus the fair market value of any property, less your selling costs. Don't add your share of the partnership's debts; the App adds your item K1 share.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        proceeds: i64,
    },
    /// The partnership paid out all of your interest in cash
    /// A final K-1 whose box 19 code A cash is all you received: no property, and box 11 code L and box 19 codes B, C and G blank.
    LiquidatedForCash,
    /// Something else
    /// Part of your interest, a gift, a death, a sale to a related person, an installment sale, a liquidation with property, an interest received for services to an investment business (section 1061), or an interest held partly one year or less.
    Other,
}

impl PartnerDisposition {
    fn record(&self) -> crate::K1Disposition {
        match self {
            Self::Kept => crate::K1Disposition::Kept,
            Self::Sold { proceeds } => crate::K1Disposition::Sold { proceeds: *proceeds },
            Self::LiquidatedForCash => crate::K1Disposition::LiquidatedForCash,
            Self::Other => crate::K1Disposition::Other,
        }
    }
    fn from_record(record: &crate::K1Disposition) -> Self {
        match record {
            crate::K1Disposition::Kept => Self::Kept,
            crate::K1Disposition::Sold { proceeds } => Self::Sold { proceeds: *proceeds },
            crate::K1Disposition::LiquidatedForCash => Self::LiquidatedForCash,
            crate::K1Disposition::Other => Self::Other,
        }
    }
}

/// Box 14 code B or C gross income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode)]
pub(crate) enum SeGross {
    /// Neither code B nor code C
    None,
    /// Code B gross farming or fishing income
    Farming {
        /// Gross farming or fishing income
        amount: i64,
    },
    /// Code C gross nonfarm income
    Nonfarm {
        /// Gross nonfarm income
        amount: i64,
    },
}

/// Section 199A information (box 20 code Z)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PartnerQbi {
    /// No section 199A statement came with this K-1
    NoStatement,
    /// A section 199A statement came with this K-1
    Statement {
        /// Qualified business income (loss)
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        income: i64,
        /// W-2 wages
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        wages: i64,
        /// Unadjusted basis of qualified property (UBIA)
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        property: i64,
        /// Specified service trade or business (SSTB)
        #[libertas_llm_doc_field]
        #[libertas_default(false)]
        service: bool,
        /// Qualified section 1231 gain (loss) from the box 1 business
        /// From the section 199A statement, with the same sign and no larger than the business part of box 10. Do not include rental property here.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_default(0)]
        section1231_qbi: i64,
        /// Section 199A dividends
        /// From the statement; they are already part of box 6a.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        reit_dividends: i64,
        /// Qualified PTP items
        /// The statement's qualified publicly traded partnership income or loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_default(0)]
        ptp_income: i64,
    },
}

/// Transient additional partnership calculation inputs.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode)]
pub(crate) struct K1PartnershipOtherBoxes {
    /// Follow-up answers called for by the printed credit codes
    pub calls: K1PartnershipCalls,
    /// Box 5 interest income
    pub interest: i64,
    /// Box 6a ordinary dividends
    pub dividends: i64,
    /// Box 6b qualified dividends
    /// The part of box 6a taxed at capital-gain rates.
    pub qualified_dividends: i64,
    /// Box 7 royalties
    /// Reported on Schedule E line 4. Box 13 code I deductions belong in the deductions list.
    pub royalties: i64,
    /// Box 8 net short-term capital gain (loss)
    /// Enter a loss as a negative amount.
    pub short: i64,
    /// Box 9a net long-term capital gain (loss)
    /// Enter a loss as a negative amount. Include the collectibles and unrecaptured section 1250 amounts below; they are parts of this box, not additional gains.
    pub long: i64,
    /// Box 9b collectibles (28%) gain (loss)
    pub collectibles: i64,
    /// Box 9c part from selling an interest in another partnership
    /// The part of box 9c the partnership's statement shows as coming from the sale of an interest in another partnership.
    pub unrecaptured_partnership: i64,
    /// Box 9c part from estates, trusts, mutual funds or REITs
    pub unrecaptured_fund: i64,
    /// Box 9c unrecaptured section 1250 gain from business property
    /// Only the box 1 business's part. Keep rental property with its rental activity. This is part of the property gain, not additional income.
    pub unrecaptured_business: i64,
    /// Box 10 section 1231 gain (loss) from the box 1 business
    /// The business part assigned by the statement, before your loss limits. Enter a loss as negative. Do not include rental property or repeat the amount in box 1.
    pub section1231: i64,
    /// Section 179 property the entity sold
    /// Each property on the fully taxable sale statement; use this tax system's depreciation. Do not include installment sales, exchanges, casualties or theft.
    /// ----
    /// Property
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Box 12 section 179 deduction
    /// Your share of the partnership's section 179 expense. It joins your own section 179 under your dollar and business-income limits, reduces your basis in full this year even when part is carried to 2027, and reduces your box 14 code A earnings as a general partner.
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// The unused part of this partnership's earlier section 179 amounts carried from your 2025 return because of the business income limit. It already reduced your basis in the partnership.
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    pub section179_carryover_sold: i64,
    /// Box 13 code A cash contributions (60%)
    /// The partnership's cash gifts to charities, deducted on Schedule A like your own cash gifts.
    pub cash_charity: i64,
    /// Box 13 code M amounts paid for medical insurance
    /// The partnership also included these premiums in box 4a. The self-employed benefits page asks your coverage.
    pub medical_insurance: i64,
    /// Box 13 code R pensions and IRAs
    /// The self-employed benefits page asks which plan.
    pub pensions: i64,
    /// Other income — box 11
    /// Enter each code once, as printed. Codes not listed here belong among the other boxes below.
    /// ----
    /// Box 11 amount
    // Tax structure: one row per distinct Form 1065 income code in the fixed catalog.
    pub income: Vec<crate::PartnerIncome>,
    /// Other deductions — box 13 codes other than A, M, O and R
    /// Enter each code once, as printed, including code I royalty deductions. Code O belongs among the other boxes below.
    /// ----
    /// Box 13 amount
    // Tax structure: one row per distinct Form 1065 deduction code in the fixed catalog.
    pub deductions: Vec<crate::PartnerDeduction>,
    /// Box 15 code O backup withholding
    pub backup_withholding: i64,
    /// Box 15 general business credits
    /// Enter the printed code, its amount and any Form 3800 line its statement requires. Codes H and AP through AU belong among the other boxes below. Code I has no 2026 amount.
    /// ----
    /// Credit
    pub credits: Vec<crate::K1CreditPage>,
    /// Investment credit property — box 15 code E and box 20 codes D, E and AV
    /// Enter each item from the K-1 and its statement. Choose automatic or enter your own completed Form3468 credit on that property row.
    /// ----
    /// Property
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Available automatic investment-credit confirmation question
    pub allowed_investment_confirmations: Vec<i32>,
    /// Investment credit property confirmed
    /// Needed only for properties using the automatic credit calculation above. No part of your share is financed by nonqualified nonrecourse financing (section 49) at your level; the statement shows no recapture, no low-income communities allocation, tax-exempt bond or subsidized financing reduction, elective payment or transfer; for rehabilitation, the building's certification is approved or timely pending. No leaves the automatic credits out; your own completed Form3468 amounts use their separate reduction answer.
    /// [NullText]
    /// No property uses the automatic credit calculation
    /// [SetText]
    /// The investment credit property facts are confirmed as
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    /// Low-income housing interest acquired after 1989
    /// Needed only when a low-income housing credit (code C or D) is entered above. Yes: you acquired your interest after 1989, so the credit is calculated with the Form 8582-CR post-1989 rules. No: you acquired it before 1990, which uses its own phase-out (Form 8582-CR Part III). Your records of when you acquired the interest show it.
    /// [NullText]
    /// No low-income housing credit is entered
    /// [SetText]
    /// I acquired the low-income housing interest after 1989, confirmed as
    pub housing_confirmed: Option<K1HousingAcquisition>,
    /// Box 18 code A tax-exempt interest
    pub exempt_interest: i64,
    /// Box 18 code B other tax-exempt income
    pub exempt_other: i64,
    /// Box 18 code C nondeductible expenses
    pub nondeductible: i64,
    /// Box 20 code C fuel tax credit information
    /// [NullText]
    /// I am not claiming a box 20 code C fuel credit
    /// [SetText]
    /// The box 20 code C fuel credit information is
    pub fuel_credit: Option<crate::PartnerFuelTaxCredit>,
    /// Fuel credit from your 2025 return
    /// [NullText]
    /// No 2025 fuel credit needs a current income adjustment
    /// [SetText]
    /// The 2025 fuel credit and prior deduction facts are
    pub prior_fuel_credit: Option<crate::PartnerPriorFuelCredit>,
    /// Box 21 foreign taxes paid or accrued
    /// Your share of the partnership's foreign taxes. It reduces your basis. Any credit for these taxes is figured from the Schedule K-3, not from this box.
    pub foreign_taxes: i64,
    /// Box 20 code AI interest and additional tax on deferred compensation
    /// The interest and the additional tax together. The income itself is in box 4a or 4b.
    pub deferred_compensation_tax: i64,
    /// Box 20 code AQ conservation reserve program payments
    pub crp_payments: i64,
    /// Box 20 code AB section 751 gain (loss)
    /// The part of the gain on selling your interest taxed as ordinary income. Enter a loss as a negative amount.
    pub sale_ordinary: i64,
    /// Box 20 code AC collectibles gain
    pub sale_collectibles: i64,
    /// Box 20 code AD unrecaptured section 1250 gain
    pub sale_unrecaptured: i64,
    /// How long you have held your interest
    /// Needed when box 19 cash is above your basis or you sold your interest: the gain is long-term when held more than one year. If you contributed to the partnership within the year before, choose One year or less.
    pub term: K1StockTerm,
    /// The partnership trades stocks, securities or commodities for its own account
    pub trading: bool,
    /// Other boxes with an amount
    /// ----
    /// Box with an amount
    pub unentered: Vec<crate::PartnerK1Item>,
    /// AMT section 179 income-limit carryover history
    /// From your own prior AMT Form 4562 records, independently of the regular carryover. Leave absent when unavailable; enter actual zero amounts when the AMT carryover is zero.
    /// [NullText]
    /// AMT carryover history is unavailable
    /// [SetText]
    /// Actual AMT carryover history, including an actual zero
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
}

/// Your basis and amount at risk
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerBasis {
    /// When you became a partner
    #[libertas_default(Continuing)]
    pub start: BasisStart,
    /// Cash you contributed in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contributions: i64,
    /// Part of the beginning liabilities you are at risk for
    /// The recourse part you are personally liable for, plus your share of qualified nonrecourse financing secured by real property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_start: i64,
    /// Part of the ending liabilities you are at risk for
    /// The same two parts at the end of the year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_end: i64,
    /// Amounts not at risk
    pub more_at_risk: PartnerAtRisk,
}

/// Your basis and amount at risk
// Records-row view of `PartnerBasis` without the start, which its own required
// strict question owns.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerBasisRecords {
    /// Cash you contributed in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub contributions: i64,
    /// Part of the beginning liabilities you are at risk for
    /// The recourse part you are personally liable for, plus your share of qualified nonrecourse financing secured by real property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_start: i64,
    /// Part of the ending liabilities you are at risk for
    /// The same two parts at the end of the year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_end: i64,
    /// Amounts not at risk
    pub more_at_risk: PartnerAtRisk,
}

impl PartnerBasisRecords {
    pub(crate) fn from_stored(basis: &PartnerBasis) -> Self {
        Self {
            contributions: basis.contributions,
            debt_start: basis.debt_start,
            debt_end: basis.debt_end,
            more_at_risk: basis.more_at_risk.clone(),
        }
    }

    pub(crate) fn stored(self, start: BasisStart) -> PartnerBasis {
        PartnerBasis {
            start,
            contributions: self.contributions,
            debt_start: self.debt_start,
            debt_end: self.debt_end,
            more_at_risk: self.more_at_risk,
        }
    }
}

/// When you became a partner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum BasisStart {
    /// You became a partner in 2026
    New,
    /// You were a partner before 2026
    Continuing {
        /// Tax basis at the beginning of 2026
        /// [NullText]
        /// Opening basis is not established yet
        /// [SetText]
        /// Established opening basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        opening: Option<i64>,
        /// Your at-risk history
        /// Needed only when this K-1 has a loss, a loss carried because of the at-risk limits, a section 179 deduction or earlier losses to recapture; otherwise leave the history missing.
        #[libertas_default(Unknown)]
        #[libertas_constrained_by("$.allowed_continuing_risk")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        carried: PartnerCarried,
    },
}

/// Losses carried from 2025
/// Leave every amount at zero when nothing was carried from 2025.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerCarried {
    /// Loss carried from 2025 because of your basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_loss: i64,
    /// Nondeductible expenses carried from 2025
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_nondeductible: i64,
    /// Ordinary loss suspended only by at-risk limits through 2025
    /// Already passed outside basis; do not enter it again as a basis carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_loss: i64,
    /// Nondeductible expenses suspended only by at-risk limits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_nondeductible: i64,
    /// Cumulative previously deducted losses for recapture
    /// After 1978, excluding this year and losses never deducted.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// At-risk recapture already reported in prior years
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub recaptured: i64,
    /// Loss carried from 2025 because of the passive activity rules
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub passive_loss: i64,
    /// Rental real estate loss carried from 2025 because of the passive activity rules
    /// The box 2 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_passive_loss: i64,
    /// Other rental loss carried from 2025 because of the passive activity rules
    /// The box 3 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_passive_loss: i64,
    /// Section 1231 loss of the rental real estate activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 2 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_section1231_passive_loss: i64,
    /// Section 1231 loss of the other rental activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 3 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_section1231_passive_loss: i64,

    /// Short-term capital loss carried from 2025 because of your basis
    /// Box 8 losses your 2025 basis worksheet did not allow (Part II line 15e, column E).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_short: i64,
    /// Long-term capital loss carried from 2025 because of your basis
    /// Box 9a losses your 2025 basis worksheet did not allow (Part II line 15f, column E).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_long: i64,
    /// Short-term capital loss carried from 2025 because of the at-risk limits
    /// Already passed your basis; do not enter it again above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_short: i64,
    /// Long-term capital loss carried from 2025 because of the at-risk limits
    /// Already passed your basis; do not enter it again above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub risk_long: i64,
    /// Rental real estate loss carried from 2025 because of your basis
    /// The box 2 loss your 2025 basis worksheet did not allow (Part II line 15b, column E).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_basis_loss: i64,
    /// Rental real estate loss carried from 2025 because of the at-risk limits
    /// The box 2 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub rental_real_estate_risk_loss: i64,
    /// Other rental loss carried from 2025 because of your basis
    /// The box 3 loss your 2025 basis worksheet did not allow (Part II line 15c, column E).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_basis_loss: i64,
    /// Other rental loss carried from 2025 because of the at-risk limits
    /// The box 3 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rental_risk_loss: i64,
    /// Section 179 deduction carried from 2025 because of your basis
    /// The box 12 amount your 2025 basis worksheet did not allow (Part II line 15i, column E). Not your Form 4562 line 13 carryover, which is entered with box 12.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_section179: i64,
    /// Cash contributions carried from 2025 because of your basis
    /// The box 13 code A gifts your 2025 basis worksheet did not allow (Part II line 15j, column E). Not a Schedule A carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_charity: i64,
    /// Foreign taxes carried from 2025 because of your basis
    /// The box 21 taxes your 2025 basis worksheet did not allow (Part II line 15d, column E).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub basis_foreign_taxes: i64,
    /// Self-employment part of the loss carried because of your basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub se_basis: i64,
    /// Self-employment part of the loss carried because of the at-risk limits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub se_risk: i64,
    /// Self-employment part of the loss carried because of the passive activity rules
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub se_passive: i64,
    /// Section 1231 business loss carried from 2025 because of your basis
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_basis_loss: i64,
    /// Section 1231 business loss carried from 2025 only because of the at-risk limits
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_risk_loss: i64,
    /// Section 1231 business loss carried from 2025 only because of the passive activity rules
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_passive_loss: i64,
}

/// Amounts not at risk
/// Leave every amount at zero when all of your investment is at risk.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerAtRisk {
    /// Cash contributions financed with amounts not at risk
    /// Included in the cash you contributed. Exclude amounts already identified in the liabilities you are at risk for.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub excluded_contributions: i64,
    /// Cash distributions returning previously excluded investment
    /// Included in box 19 code A; never count the same funds twice.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub excluded_distributions: i64,
    /// Previously at-risk cash investment newly protected against loss
    /// Exclude casualty or tort insurance and debt changes already entered.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub protected: i64,
    /// Previously excluded cash investment no longer protected
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub released: i64,
}

// The separately stated amounts of the Other boxes fold, in the
// stored record's order.
#[allow(clippy::too_many_arguments)]
fn separate_items(
    interest: i64,
    dividends: i64,
    qualified_dividends: i64,
    royalties: i64,
    short: i64,
    long: i64,
    collectibles: i64,
    unrecaptured_partnership: i64,
    unrecaptured_fund: i64,
    cash_charity: i64,
    backup_withholding: i64,
    exempt_interest: i64,
    foreign_taxes: i64,
) -> Option<crate::PartnerSeparateTotals> {
    let items = crate::PartnerSeparateTotals {
        interest,
        dividends,
        qualified_dividends,
        royalties,
        short,
        long,
        collectibles,
        unrecaptured_partnership,
        unrecaptured_fund,
        cash_charity,
        backup_withholding,
        exempt_interest,
        foreign_taxes,
    };
    (items != separate_items_zero()).then_some(items)
}
fn separate_items_zero() -> crate::PartnerSeparateTotals {
    crate::PartnerSeparateTotals {
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: 0,
    }
}

impl K1Partnership {
    pub(crate) fn qbi_records_record(&self) -> crate::partnership_qbi::PartnerQbiRecords {
        self.qbi_records.clone().unwrap_or(crate::partnership_qbi::PartnerQbiRecords {
            qualified_se_gross: 0,
            history: None,
        })
    }
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the server twin of the schema rules), then the
    // constrained continuing-owner at-risk history checked again on save.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        self.issue_with_mode(true)
    }
    pub(crate) fn issue_with_mode(&self, source_alignment: bool) -> Option<ValidationIssue> {
        self.issue_with_modes(source_alignment, source_alignment)
    }
    pub(crate) fn topic_issue(&self, require_answers: bool) -> Option<ValidationIssue> {
        self.issue_with_modes(require_answers, true)
    }
    fn issue_with_modes(
        &self,
        require_answers: bool,
        source_alignment: bool,
    ) -> Option<ValidationIssue> {
        if require_answers && self.participation.is_none() {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_PARTICIPATION").at_field("participation"),
            );
        }
        if require_answers && self.limited_for_se.is_none() {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_PARTNER_RECORDS").at_field("limited_for_se"),
            );
        }
        if require_answers && self.disposition.is_none() {
            return Some(ValidationIssue::new("DOCUMENT_K1_DISPOSITION").at_field("disposition"));
        }
        if require_answers
            && self.record().rental_participation_needed()
            && self.rentals.participation.is_none()
        {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_RENTAL_RECORDS")
                    .at_field("participation")
                    .in_field("rentals"),
            );
        }

        if require_answers
            && matches!(self.basis.start, BasisStart::Continuing { opening: None, .. })
        {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS")
                    .at_field("opening")
                    .in_field("start")
                    .in_field("basis"),
            );
        }
        if require_answers
            && matches!(&self.amt, Some(PartnerAmtRecords::Separate { record }) if record.basis.opening.is_none())
        {
            return Some(
                ValidationIssue::new("DOCUMENT_K1_OPENING_BASIS")
                    .at_field("opening")
                    .in_field("basis")
                    .in_field("record")
                    .in_field("amt"),
            );
        }
        if source_alignment
            && let K1IncomeSource::Services { gross } = self.income_source
            && self.ordinary > 0
            && gross < self.ordinary
        {
            return Some(
                ValidationIssue::new(
                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                )
                .at_field("gross")
                .in_field("income_source"),
            );
        }
        if let Some(issue) = self.more.prior_fuel_credit.as_ref().and_then(|prior| {
            prior.validation_issue(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            )
        }) {
            return Some(issue.in_field("prior_fuel_credit").in_field("more"));
        }

        if let Some(issue) = k1_source_admission_issue(
            self.more.amt_section179_income_carry.as_ref(),
            self.reporting_period.as_ref(),
            &self.more.section179_sales,
            matches!(self.amt, Some(PartnerAmtRecords::SameAsRegular)),
            matches!(self.amt, Some(PartnerAmtRecords::Separate { .. })),
            (self.more.section179_carryover, self.more.section179_carryover_sold),
            source_alignment,
        ) {
            return Some(issue);
        }
        if let Some(index) = self
            .rentals
            .disposed
            .iter()
            .enumerate()
            .find_map(|(i, v)| self.rentals.disposed[..i].contains(v).then_some(i))
        {
            return Some(
                ValidationIssue::new("K1_RENTAL_DISPOSED_TWICE")
                    .at_list_item("disposed", index)
                    .in_field("rentals"),
            );
        }
        if let crate::K3Attached::Attached(k3) = &self.schedule_k3
            && let Some(issue) = k3.issue_with_mode(require_answers)
        {
            return Some(issue.in_field("schedule_k3"));
        }
        if let PartnerQbi::Statement { section1231_qbi, .. } = &self.qbi
            && !(self.more.section1231.min(0)..=self.more.section1231.max(0))
                .contains(section1231_qbi)
        {
            return Some(
                ValidationIssue::new("K1_SECTION1231_QBI_ABOVE_BOX")
                    .at_field("section1231_qbi")
                    .in_field("qbi"),
            );
        }
        for (field, debt, total) in [
            ("debt_start", self.basis.debt_start, self.liabilities_start),
            ("debt_end", self.basis.debt_end, self.liabilities_end),
        ] {
            if source_alignment && debt > total {
                return Some(
                    ValidationIssue::new("PARTNER_DEBT_ABOVE_LIABILITIES")
                        .at_field(field)
                        .in_field("basis"),
                );
            }
        }
        if let BasisStart::Continuing {
            carried:
                PartnerCarried {
                    basis_loss,
                    risk_loss,
                    passive_loss,
                    se_basis,
                    se_risk,
                    se_passive,
                    ..
                },
            ..
        } = &self.basis.start
        {
            for (field, part, total) in [
                ("se_basis", se_basis, basis_loss),
                ("se_risk", se_risk, risk_loss),
                ("se_passive", se_passive, passive_loss),
            ] {
                if part > total {
                    return Some(
                        ValidationIssue::new("PARTNER_SE_CARRY_ABOVE_LOSS")
                            .at_field(field)
                            .in_field("carried")
                            .in_field("start")
                            .in_field("basis"),
                    );
                }
            }
        }
        {
            let K1PartnershipOtherBoxes {
                dividends,
                qualified_dividends,
                medical_insurance,
                unentered,
                credits,
                allowed_investment_confirmations: _,
                investment_property,
                investment_confirmed,
                housing_confirmed,
                ..
            } = &self.more;
            if let Some(issue) = crate::K1Credit::issue_list_with_mode(
                &credits.iter().cloned().map(Into::into).collect::<Vec<_>>(),
                require_answers,
            ) {
                return Some(issue.in_field("more"));
            }
            if require_answers
                && investment_property.iter().any(|p| p.own_form().is_none())
                && investment_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_INVESTMENT_NEEDS_CONFIRMATION")
                        .at_field("investment_confirmed")
                        .in_field("more"),
                );
            }
            if require_answers
                && credits.iter().any(|credit| credit.code.housing())
                && housing_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_HOUSING_NEEDS_CONFIRMATION")
                        .at_field("housing_confirmed")
                        .in_field("more"),
                );
            }
            if qualified_dividends > dividends {
                return Some(
                    ValidationIssue::new("PARTNER_QUALIFIED_ABOVE_DIVIDENDS")
                        .at_field("qualified_dividends")
                        .in_field("more"),
                );
            }
            if *medical_insurance > self.services {
                return Some(
                    ValidationIssue::new("PARTNER_MEDICAL_ABOVE_4A")
                        .at_field("medical_insurance")
                        .in_field("more"),
                );
            }
            if let Some(index) = unentered
                .iter()
                .enumerate()
                .find_map(|(i, item)| unentered[..i].contains(item).then_some(i))
            {
                return Some(
                    ValidationIssue::new("PARTNER_BOX_TWICE")
                        .at_list_item("unentered", index)
                        .in_field("more"),
                );
            }
        }
        if source_alignment
            && matches!(self.participation, Some(crate::PartnerParticipation::Limited { .. }))
            && self.rentals.participation.is_some_and(crate::K1RentalParticipation::active)
        {
            return Some(
                ValidationIssue::new("PARTNER_LIMITED_NOT_ACTIVE")
                    .at_field("participation")
                    .in_field("rentals"),
            );
        }
        if let Some(issue) = crate::PartnerIncome::issue(&self.more.income, "income")
            .or_else(|| crate::PartnerDeduction::issue(&self.more.deductions, "deductions"))
        {
            return Some(issue.in_field("more"));
        }
        if let Some(issue) = crate::PartnerSpecial::issue_list(&self.special) {
            return Some(issue);
        }
        if source_alignment && self.final_k1 && self.disposition == Some(PartnerDisposition::Kept) {
            return Some(
                ValidationIssue::new("K1_FINAL_NEEDS_DISPOSITION").at_field("disposition"),
            );
        }
        if source_alignment
            && (self.more.sale_ordinary != 0
                || self.more.sale_collectibles != 0
                || self.more.sale_unrecaptured != 0)
            && matches!(
                self.disposition,
                Some(PartnerDisposition::Kept | PartnerDisposition::LiquidatedForCash)
            )
        {
            return Some(ValidationIssue::new("K1_SALE_CODES_NEED_SALE").at_field("disposition"));
        }
        if matches!(
            self.disposition,
            Some(PartnerDisposition::Sold { .. } | PartnerDisposition::LiquidatedForCash)
        ) && self.more.term == K1StockTerm::NotNeeded
        {
            return Some(
                ValidationIssue::new("K1_SALE_NEEDS_HOLDING_PERIOD")
                    .at_field("term")
                    .in_field("more"),
            );
        }
        if source_alignment
            && self.disposition == Some(PartnerDisposition::LiquidatedForCash)
            && !self.final_k1
        {
            return Some(
                ValidationIssue::new("K1_LIQUIDATION_NEEDS_FINAL").at_field("disposition"),
            );
        }
        if let BasisStart::Continuing { risk_history: crate::ScorpRiskStart::New, .. } =
            &self.basis.start
        {
            return Some(
                ValidationIssue::new("PARTNER_RISK_HISTORY_NEW")
                    .at_field("risk_history")
                    .in_field("start")
                    .in_field("basis"),
            );
        }
        None
    }

    // The stored record this page writes. Boxes 13 code M and R are printed
    // boxes written here (T2-43p); the partner's coverage, the part charged
    // against distributions and the plan choice belong to the self-employed
    // benefits page: they are written empty here and kept from the accepted
    // record by the caller.
    pub(crate) fn record(&self) -> crate::PartnerK1 {
        self.record_with_parent_amt_convention(true)
    }

    pub(crate) fn record_with_parent_amt_convention(
        &self,
        inherit_child_amt: bool,
    ) -> crate::PartnerK1 {
        // Only a situation that leaves the whole K-1 out of the estimate
        // makes the basis and financing confirmations No.
        let complete = !self.special.iter().any(|situation| situation.whole());
        let (start, opening, carried) = match &self.basis.start {
            BasisStart::New => (crate::ScorpRiskStart::New, Some(0), [0; 28]),
            BasisStart::Continuing {
                opening,
                risk_history,
                carried:
                    PartnerCarried {
                        basis_loss,
                        basis_nondeductible,
                        risk_loss,
                        risk_nondeductible,
                        deducted,
                        recaptured,
                        passive_loss,
                        rental_real_estate_passive_loss,
                        other_rental_passive_loss,
                        rental_real_estate_section1231_passive_loss,
                        other_rental_section1231_passive_loss,
                        basis_short,
                        basis_long,
                        risk_short,
                        risk_long,
                        rental_real_estate_basis_loss,
                        rental_real_estate_risk_loss,
                        other_rental_basis_loss,
                        other_rental_risk_loss,
                        basis_section179,
                        basis_charity,
                        basis_foreign_taxes,
                        se_basis,
                        se_risk,
                        se_passive,
                        section1231_basis_loss,
                        section1231_risk_loss,
                        section1231_passive_loss,
                    },
            } => (
                risk_history.clone(),
                *opening,
                [
                    *basis_loss,
                    *basis_nondeductible,
                    *risk_loss,
                    *risk_nondeductible,
                    *deducted,
                    *recaptured,
                    *passive_loss,
                    *rental_real_estate_passive_loss,
                    *other_rental_passive_loss,
                    *rental_real_estate_section1231_passive_loss,
                    *other_rental_section1231_passive_loss,
                    *basis_short,
                    *basis_long,
                    *risk_short,
                    *risk_long,
                    *rental_real_estate_basis_loss,
                    *rental_real_estate_risk_loss,
                    *other_rental_basis_loss,
                    *other_rental_risk_loss,
                    *basis_section179,
                    *basis_charity,
                    *basis_foreign_taxes,
                    *se_basis,
                    *se_risk,
                    *se_passive,
                    *section1231_basis_loss,
                    *section1231_risk_loss,
                    *section1231_passive_loss,
                ],
            ),
        };
        let [
            basis_loss,
            basis_nondeductible,
            risk_loss,
            risk_nondeductible,
            deducted,
            recaptured,
            passive_loss,
            rental_real_estate_passive_loss,
            other_rental_passive_loss,
            rental_real_estate_section1231_passive_loss,
            other_rental_section1231_passive_loss,
            basis_short,
            basis_long,
            risk_short,
            risk_long,
            rental_real_estate_basis_loss,
            rental_real_estate_risk_loss,
            other_rental_basis_loss,
            other_rental_risk_loss,
            basis_section179,
            basis_charity,
            basis_foreign_taxes,
            se_basis,
            se_risk,
            se_passive,
            section1231_basis_loss,
            section1231_risk_loss,
            section1231_passive_loss,
        ] = carried;
        let (rentals, rental_facts) = self.rentals.record(
            [rental_real_estate_passive_loss, other_rental_passive_loss],
            [rental_real_estate_section1231_passive_loss, other_rental_section1231_passive_loss],
        );
        let PartnerAtRisk { excluded_contributions, excluded_distributions, protected, released } =
            self.basis.more_at_risk;
        let mut record = crate::PartnerK1 {
            source: None,
            research_credit_line: None,
            amt_record_choice: None,
            amt_section179_income_carry: self.more.amt_section179_income_carry.clone(),
            reporting_period: self.reporting_period,
            section179_sales: Vec::new(),
            qbi_aggregation_group: crate::common::optional_text(&self.qbi_group),
            regular: crate::PartnerActivityRecord {
                section1231: None,
                ordinary: self.ordinary,
                exempt: 0,
                nondeductible: 0,
                basis: crate::PartnerBasisRecord {
                    opening,
                    contributions: self.basis.contributions,
                    opening_liabilities: self.liabilities_start,
                    ending_liabilities: self.liabilities_end,
                    assumed: 0,
                    transferred: 0,
                    distributions: self.distributions,
                    prior_loss: basis_loss,
                    prior_nondeductible: basis_nondeductible,
                    complete,
                },
                risk: crate::PartnerRiskRecord {
                    start,
                    opening_debt: self.basis.debt_start,
                    ending_debt: self.basis.debt_end,
                    excluded_contributions,
                    excluded_distributions,
                    protected,
                    released,
                    prior_loss: risk_loss,
                    prior_nondeductible: risk_nondeductible,
                    deducted,
                    recaptured,
                    complete,
                },
                passive_loss,
                rentals,
                carried: crate::K1LossCarryforwards {
                    real_estate: crate::K1LossCarry {
                        basis: rental_real_estate_basis_loss,
                        risk: rental_real_estate_risk_loss,
                    },
                    other_rental: crate::K1LossCarry {
                        basis: other_rental_basis_loss,
                        risk: other_rental_risk_loss,
                    },
                    section179: basis_section179,
                    charity: basis_charity,
                    foreign_taxes: basis_foreign_taxes,
                },
                capital_carried: crate::PartnerCapitalCarry {
                    short: crate::K1LossCarry { basis: basis_short, risk: risk_short },
                    long: crate::K1LossCarry { basis: basis_long, risk: risk_long },
                },
            },
            income_source: None,
            gross_personal_services: None,
            guaranteed_services: self.services,
            guaranteed_capital: self.capital,
            alternative: None,
            participation: self.participation.clone(),
            trade_kind: crate::ScorpTradeKind::Ordinary,
            limited_for_se: self.limited_for_se,
            reported_se: Some(self.se),
            se_history: crate::PartnerSeHistory {
                basis: se_basis,
                risk: se_risk,
                passive: se_passive,
            },
            se_gross: match &self.se_gross {
                SeGross::None => None,
                SeGross::Farming { amount } => Some(crate::PartnerSeGross::Farm { gross: *amount }),
                SeGross::Nonfarm { amount } => {
                    Some(crate::PartnerSeGross::Nonfarm { gross: *amount })
                }
            },
            qbi: match &self.qbi {
                PartnerQbi::NoStatement => None,
                PartnerQbi::Statement {
                    income,
                    wages,
                    property,
                    service,
                    section1231_qbi,
                    reit_dividends,
                    ptp_income,
                } => Some(crate::PartnerQbiStatement {
                    income: *income,
                    wages: *wages,
                    property: *property,
                    service: *service,
                    section1231_qbi: *section1231_qbi,
                    reit_dividends: *reit_dividends,
                    ptp_income: *ptp_income,
                }),
            },
            qbi_records: self.qbi_records_record(),
            health_insurance: crate::BusinessHealthInsurance::None,
            health_guaranteed: 0,
            health_distributions: 0,
            pensions: 0,
            retirement: crate::PartnerRetirement::None,
            fuel_tax_credit: None,
            prior_fuel_credit: None,
            separate: None,
            schedule_k3: match &self.schedule_k3 {
                crate::K3Attached::NotAttached => None,
                crate::K3Attached::Attached(k3) => Some(k3.pruned()),
            },
            rental_facts,
            section179: 0,
            section179_carryover: 0,
            section179_carryover_sold: 0,
            credits: Vec::new(),
            investment_property: Vec::new(),
            investment_confirmed: None,
            housing_confirmed: None,
            unentered: Vec::new(),
            final_k1: self.final_k1,
            income: Vec::new(),
            deductions: Vec::new(),
            deferred_compensation_tax: 0,
            crp_payments: 0,
            interest_term: None,
            disposition: self.disposition.as_ref().map(PartnerDisposition::record),
            sale: crate::PartnerSaleParts::default(),
            special: self.special.clone(),
            publicly_traded: self.publicly_traded,
        };
        (record.income_source, record.gross_personal_services) = self.income_source.record();
        {
            let K1PartnershipOtherBoxes {
                calls: _,
                interest,
                dividends,
                qualified_dividends,
                royalties,
                short,
                long,
                collectibles,
                unrecaptured_partnership,
                unrecaptured_fund,
                unrecaptured_business,
                section1231,
                section179,
                section179_carryover,
                section179_carryover_sold,
                amt_section179_income_carry: _,
                section179_sales,
                cash_charity,
                medical_insurance,
                pensions,
                backup_withholding,
                credits,
                allowed_investment_confirmations: _,
                investment_property,
                investment_confirmed,
                housing_confirmed,
                exempt_interest,
                exempt_other,
                nondeductible,
                fuel_credit,
                prior_fuel_credit,
                foreign_taxes,
                deferred_compensation_tax,
                crp_payments,
                sale_ordinary,
                sale_collectibles,
                sale_unrecaptured,
                term,
                trading,
                unentered,
                income,
                deductions,
            } = &self.more;
            record.income = income.iter().map(crate::PartnerIncomeTotal::from).collect();
            record.deductions = deductions.clone();
            record.deferred_compensation_tax = *deferred_compensation_tax;
            record.crp_payments = *crp_payments;
            record.sale = crate::PartnerSaleParts {
                ordinary: *sale_ordinary,
                collectibles: *sale_collectibles,
                unrecaptured: *sale_unrecaptured,
            };
            record.interest_term = match term {
                K1StockTerm::NotNeeded => None,
                K1StockTerm::ShortTerm => Some(crate::GainTerm::Short),
                K1StockTerm::LongTerm => Some(crate::GainTerm::Long),
            };
            record.health_guaranteed = *medical_insurance;
            record.pensions = *pensions;
            record.credits = credits.iter().cloned().map(Into::into).collect();
            record.investment_property = investment_property.clone();
            record.investment_confirmed =
                investment_confirmed.map(K1InvestmentConfirmation::confirmed);
            record.housing_confirmed = housing_confirmed.map(K1HousingAcquisition::after_1989);
            record.separate = separate_items(
                *interest,
                *dividends,
                *qualified_dividends,
                *royalties,
                *short,
                *long,
                *collectibles,
                *unrecaptured_partnership,
                *unrecaptured_fund,
                *cash_charity,
                *backup_withholding,
                *exempt_interest,
                *foreign_taxes,
            );
            let property = crate::K1Section1231 {
                net: *section1231,
                unrecaptured: *unrecaptured_business,
                basis_loss: section1231_basis_loss,
                risk_loss: section1231_risk_loss,
                passive_loss: section1231_passive_loss,
            };
            record.regular.section1231 = property.has_items().then_some(property);
            record.section179 = *section179;
            record.section179_carryover = *section179_carryover;
            record.section179_carryover_sold = *section179_carryover_sold;
            record.section179_sales = section179_sales.clone();
            record.regular.exempt = *exempt_other;
            record.regular.nondeductible = *nondeductible;
            record.fuel_tax_credit = fuel_credit.clone();
            record.prior_fuel_credit = prior_fuel_credit.clone();
            if *trading {
                record.trade_kind = crate::ScorpTradeKind::FinancialTrading;
            }
            record.unentered = unentered.clone();
        }
        if inherit_child_amt && matches!(self.amt, Some(PartnerAmtRecords::SameAsRegular)) {
            for sale in &mut record.section179_sales {
                sale.apply_parent_same_as_regular();
            }
        }
        record.alternative = self.amt.as_ref().map(|amt| amt.record(&record.regular));
        record
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written. The partner's coverage, distribution-paid premiums
    // and plan choice are not on this page, so they are compared as stored. An Other LLC member whose
    // box 14 code A includes box 1 writes the same record as a general
    // partner and reopens as one.
    pub(crate) fn from_record(name: &str, r: &crate::PartnerK1) -> Option<Self> {
        let basis = &r.regular.basis;
        let risk = &r.regular.risk;
        let history = &r.se_history;
        let start = if risk.start == crate::ScorpRiskStart::New && basis.opening == Some(0) {
            BasisStart::New
        } else {
            let carried = [
                basis.prior_loss,
                basis.prior_nondeductible,
                risk.prior_loss,
                risk.prior_nondeductible,
                risk.deducted,
                risk.recaptured,
                r.regular.passive_loss,
                r.regular.rentals.real_estate_carryover,
                r.regular.rentals.other_carryover,
                r.regular.rentals.real_estate_property.passive_loss,
                r.regular.rentals.other_property.passive_loss,
                r.regular.capital_carried.short.basis,
                r.regular.capital_carried.long.basis,
                r.regular.capital_carried.short.risk,
                r.regular.capital_carried.long.risk,
                r.regular.carried.real_estate.basis,
                r.regular.carried.real_estate.risk,
                r.regular.carried.other_rental.basis,
                r.regular.carried.other_rental.risk,
                r.regular.carried.section179,
                r.regular.carried.charity,
                r.regular.carried.foreign_taxes,
                history.basis,
                history.risk,
                history.passive,
                r.regular.section1231.as_ref().map_or(0, |v| v.basis_loss),
                r.regular.section1231.as_ref().map_or(0, |v| v.risk_loss),
                r.regular.section1231.as_ref().map_or(0, |v| v.passive_loss),
            ];
            BasisStart::Continuing {
                opening: basis.opening,
                risk_history: risk.start.clone(),
                carried: {
                    let [
                        basis_loss,
                        basis_nondeductible,
                        risk_loss,
                        risk_nondeductible,
                        deducted,
                        recaptured,
                        passive_loss,
                        rental_real_estate_passive_loss,
                        other_rental_passive_loss,
                        rental_real_estate_section1231_passive_loss,
                        other_rental_section1231_passive_loss,
                        basis_short,
                        basis_long,
                        risk_short,
                        risk_long,
                        rental_real_estate_basis_loss,
                        rental_real_estate_risk_loss,
                        other_rental_basis_loss,
                        other_rental_risk_loss,
                        basis_section179,
                        basis_charity,
                        basis_foreign_taxes,
                        se_basis,
                        se_risk,
                        se_passive,
                        section1231_basis_loss,
                        section1231_risk_loss,
                        section1231_passive_loss,
                    ] = carried;
                    PartnerCarried {
                        basis_loss,
                        basis_nondeductible,
                        risk_loss,
                        risk_nondeductible,
                        deducted,
                        recaptured,
                        passive_loss,
                        rental_real_estate_passive_loss,
                        other_rental_passive_loss,
                        rental_real_estate_section1231_passive_loss,
                        other_rental_section1231_passive_loss,
                        basis_short,
                        basis_long,
                        risk_short,
                        risk_long,
                        rental_real_estate_basis_loss,
                        rental_real_estate_risk_loss,
                        other_rental_basis_loss,
                        other_rental_risk_loss,
                        basis_section179,
                        basis_charity,
                        basis_foreign_taxes,
                        se_basis,
                        se_risk,
                        se_passive,
                        section1231_basis_loss,
                        section1231_risk_loss,
                        section1231_passive_loss,
                    }
                },
            }
        };
        let more_at_risk = PartnerAtRisk {
            excluded_contributions: risk.excluded_contributions,
            excluded_distributions: risk.excluded_distributions,
            protected: risk.protected,
            released: risk.released,
        };
        let separate = r.separate.clone().unwrap_or_else(separate_items_zero);
        let more = K1PartnershipOtherBoxes {
            calls: K1PartnershipCalls::default(),
            interest: separate.interest,
            dividends: separate.dividends,
            qualified_dividends: separate.qualified_dividends,
            royalties: separate.royalties,
            short: separate.short,
            long: separate.long,
            collectibles: separate.collectibles,
            unrecaptured_partnership: separate.unrecaptured_partnership,
            unrecaptured_fund: separate.unrecaptured_fund,
            unrecaptured_business: r.regular.section1231.as_ref().map_or(0, |v| v.unrecaptured),
            section1231: r.regular.section1231.as_ref().map_or(0, |v| v.net),
            section179: r.section179,
            section179_carryover: r.section179_carryover,
            section179_carryover_sold: r.section179_carryover_sold,
            amt_section179_income_carry: r.amt_section179_income_carry.clone(),
            section179_sales: r.section179_sales.clone(),
            cash_charity: separate.cash_charity,
            medical_insurance: r.health_guaranteed,
            pensions: r.pensions,
            backup_withholding: separate.backup_withholding,
            credits: r.credits.iter().cloned().map(Into::into).collect(),
            allowed_investment_confirmations: investment_choices(&r.investment_property),
            investment_property: r.investment_property.clone(),
            investment_confirmed: r.investment_confirmed.map(K1InvestmentConfirmation::from),
            housing_confirmed: r.housing_confirmed.map(K1HousingAcquisition::from),
            exempt_interest: separate.exempt_interest,
            exempt_other: r.regular.exempt,
            nondeductible: r.regular.nondeductible,
            fuel_credit: r.fuel_tax_credit.clone(),
            prior_fuel_credit: r.prior_fuel_credit.clone(),
            foreign_taxes: separate.foreign_taxes,
            deferred_compensation_tax: r.deferred_compensation_tax,
            crp_payments: r.crp_payments,
            sale_ordinary: r.sale.ordinary,
            sale_collectibles: r.sale.collectibles,
            sale_unrecaptured: r.sale.unrecaptured,
            term: match r.interest_term {
                None => K1StockTerm::NotNeeded,
                Some(crate::GainTerm::Short) => K1StockTerm::ShortTerm,
                Some(crate::GainTerm::Long) => K1StockTerm::LongTerm,
            },
            trading: r.trade_kind == crate::ScorpTradeKind::FinancialTrading,
            unentered: r.unentered.clone(),
            income: if let Some(source) = &r.source {
                source.page(&crate::PartnerPersonalRecords::default()).more.income
            } else {
                r.income
                    .iter()
                    .map(|item| crate::PartnerIncome { code: item.code, amount: item.amount })
                    .collect()
            },
            deductions: r.deductions.clone(),
        };
        let mut page = Self {
            printed: None,
            qbi_group: r.qbi_aggregation_group.clone().unwrap_or_default(),
            name: name.into(),
            reporting_period: r.reporting_period,
            publicly_traded: r.publicly_traded,
            final_k1: r.final_k1,
            participation: r.participation.clone(),
            limited_for_se: r.limited_for_se,
            liabilities_start: basis.opening_liabilities,
            liabilities_end: basis.ending_liabilities,
            ordinary: r.regular.ordinary,
            rentals: K1RentalBoxes::from_record(&r.regular.rentals, &r.rental_facts),
            services: r.guaranteed_services,
            capital: r.guaranteed_capital,
            se: r.reported_se?,
            se_gross: match r.se_gross {
                None => SeGross::None,
                Some(crate::PartnerSeGross::Farm { gross }) => SeGross::Farming { amount: gross },
                Some(crate::PartnerSeGross::Nonfarm { gross }) => {
                    SeGross::Nonfarm { amount: gross }
                }
            },
            distributions: basis.distributions,
            qbi: match &r.qbi {
                None => PartnerQbi::NoStatement,
                Some(statement) => PartnerQbi::Statement {
                    income: statement.income,
                    wages: statement.wages,
                    property: statement.property,
                    service: statement.service,
                    section1231_qbi: statement.section1231_qbi,
                    reit_dividends: statement.reit_dividends,
                    ptp_income: statement.ptp_income,
                },
            },
            qbi_records: Some(r.qbi_records.clone()),
            more,
            schedule_k3: match &r.schedule_k3 {
                None => crate::K3Attached::NotAttached,
                Some(k3) => crate::K3Attached::Attached(k3.clone()),
            },
            basis: PartnerBasis {
                start,
                contributions: basis.contributions,
                debt_start: risk.opening_debt,
                debt_end: risk.ending_debt,
                more_at_risk,
            },
            income_source: K1IncomeSource::from_record(
                r.income_source.as_ref(),
                r.gross_personal_services,
            )?,
            amt: match r.amt_record_choice {
                Some(true) => {
                    r.alternative.clone().map(|record| PartnerAmtRecords::Separate { record })
                }
                Some(false) => Some(PartnerAmtRecords::SameAsRegular),
                None => r
                    .alternative
                    .as_ref()
                    .map(|amt| PartnerAmtRecords::from_record(&r.regular, amt)),
            },
            disposition: r.disposition.as_ref().map(PartnerDisposition::from_record),
            special: r.special.clone(),
        };
        page.more.calls = K1PartnershipCalls::from_credits(
            &page.more.credits.iter().map(K1CreditCallBox::from).collect::<Vec<_>>(),
        );
        if r.amt_record_choice.is_none()
            && r.section179_sales
                .iter()
                .any(|sale| !sale.alternative.matches_known_regular(&sale.regular))
        {
            page.amt = r.alternative.clone().map(|record| PartnerAmtRecords::Separate { record });
        } else if matches!(page.amt, Some(PartnerAmtRecords::SameAsRegular))
            && r.section179_sales
                .iter()
                .all(|sale| sale.alternative.matches_known_regular(&sale.regular))
        {
            for sale in &mut page.more.section179_sales {
                sale.alternative = crate::K1Section179SaleAmtSource::NotReported;
            }
        }
        let inherit_child_amt = r.source.is_none()
            || r.section179_sales
                .iter()
                .all(|sale| sale.alternative.matches_known_regular(&sale.regular));
        let same = |page: &Self| {
            let mut written = page.record_with_parent_amt_convention(inherit_child_amt);
            written.health_insurance = r.health_insurance.clone();
            written.health_distributions = r.health_distributions;
            written.retirement = r.retirement.clone();
            written.research_credit_line = r.research_credit_line;
            written.amt_record_choice = r.amt_record_choice;
            if let Some(source) = &r.source {
                written.income = source.income_totals();
            }
            written.source = r.source.clone();
            written == *r
        };
        same(&page).then_some(page)
    }
}

pub(crate) fn investment_choices(properties: &[crate::K1InvestmentProperty]) -> Vec<i32> {
    if properties.iter().any(|property| property.own_form().is_none()) {
        alloc::vec![0, 1]
    } else {
        Vec::new()
    }
}

/// Automatic investment-credit facts
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum K1InvestmentConfirmation {
    /// Yes — the listed automatic credit facts are confirmed
    Yes,
    /// No — leave the automatic credits out
    No,
}

impl K1InvestmentConfirmation {
    pub(crate) fn confirmed(self) -> bool {
        matches!(self, Self::Yes)
    }
}

impl From<bool> for K1InvestmentConfirmation {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

/// Low-income housing interest acquisition date
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum K1HousingAcquisition {
    /// Yes — acquired after 1989
    After1989,
    /// No — acquired before 1990
    Before1990,
}
impl K1HousingAcquisition {
    fn after_1989(self) -> bool {
        matches!(self, Self::After1989)
    }
}
impl From<bool> for K1HousingAcquisition {
    fn from(value: bool) -> Self {
        if value { Self::After1989 } else { Self::Before1990 }
    }
}

/// Printed credit-code projection for follow-up selection
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1CreditCallBox {
    /// Printed credit code
    pub code: crate::K1CreditCodePage,
}
impl From<&crate::K1CreditPage> for K1CreditCallBox {
    fn from(value: &crate::K1CreditPage) -> Self {
        Self { code: value.code.clone() }
    }
}

/// Follow-up answers for Schedule K-1 (Form 1065)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1PartnershipCalls {
    /// Acquisition date for box 15 codes C and D
    pub housing: Vec<i32>,
}
impl K1PartnershipCalls {
    pub(crate) fn from_credits(credits: &[K1CreditCallBox]) -> Self {
        // Partner's Instructions for Schedule K-1 (Form 1065), box 15 codes C
        // and D, and Form 8582-CR distinguish pre-1990 housing interests.
        // https://www.irs.gov/instructions/i1065sk1
        // Code presence calls for the answer even when the reported credit is zero.
        Self { housing: housing_choices(credits) }
    }
}

/// Follow-up answers for Schedule K-1 (Form 1120-S)
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1ScorpCalls {
    /// Acquisition date for box 13 codes C and D
    pub housing: Vec<i32>,
}
impl K1ScorpCalls {
    pub(crate) fn from_credits(credits: &[K1CreditCallBox]) -> Self {
        // Shareholder's Instructions for Schedule K-1 (Form 1120-S), box 13
        // codes C and D; Form 8582-CR applies the acquisition-date distinction.
        // https://www.irs.gov/instructions/i1120ssk
        Self { housing: housing_choices(credits) }
    }
}
fn housing_choices(credits: &[K1CreditCallBox]) -> Vec<i32> {
    if credits.iter().any(|credit| credit.code.housing()) { alloc::vec![0, 1] } else { Vec::new() }
}
