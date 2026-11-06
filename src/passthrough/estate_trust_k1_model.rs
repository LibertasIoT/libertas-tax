// estate_trust_k1 items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

/// Beneficiary Schedule K-1 (Form 1041) for 2026
/// Enter the boxes as reported by the fiduciary. Box 5 supports ordinary nonbusiness royalties, including limited directly apportioned box 9 code A sound-recording depreciation, code B cost depletion or code C amortization from the same activity. Boxes 6–8 business and rental income enter Schedule E line 33 with their box 9 deductions, the passive-activity rules and the box 14 code I trade or business QBI statement. Limited box 10 IRD-interest estate tax and final-year box 11 codes A–F require their source confirmations. Standalone box 12 code A or a confirmed complete code B/C/D component reconciliation, including code E/F portions within D, and limited box 14 codes F/H/I use their respective estimated-tax, AMT, NIIT and section 199A calculations. Other K-1 income, deduction, credit, AMT or foreign-source items, including unentered attachments, need their own calculations.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalEstateTrustPortfolioK1 {
    /// Final-year source character and beneficiary entitlement
    pub final_character: Option<crate::EstateFinalCharacter>,
    /// Signed box 12 reconciliation and activity allocations
    pub amt_reconciliation: Option<crate::EstateAmtReconciliation>,
    /// Box 14 code E source net investment income for Form 4952
    pub investment_income: Option<crate::EstateInvestmentIncome>,
    /// Box 13 credit statements by their Form 1041 code identity
    // Tax structure: one statement per supported Form 1041 box 13 credit code.
    #[libertas_size(max = 19)]
    pub reported_credits: Vec<crate::EstateReportedCredit>,
    /// Box 14 code B foreign tax statement
    pub foreign_statement: Option<crate::EstateForeignStatement>,
    /// Fiduciary investment-credit property source statements
    pub investment_property: Vec<crate::EstateInvestmentProperty>,

    /// Estate's or trust's EIN from item A; all nine digits without punctuation
    // Legal identifier: an EIN is exactly nine digits without punctuation.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern("[0-9]{9}")]
    pub ein: String,
    // Per-box amount bounds keep the portfolio preview sum within signed 64-bit cents.
    /// Box 1 taxable interest — Form 1040 line 2b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Box 2a ordinary dividends — Form 1040 line 3b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub ordinary_dividends: i64,
    /// Box 2b qualified dividends, included in box 2a — Form 1040 line 3a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified_dividends: i64,
    /// Box 14 code I section 199A dividends included in box 2a ordinary dividends
    /// Enter only the beneficiary's apportioned qualified REIT/RIC dividends from the fiduciary's statement, separate from box 2b qualified dividends.
    pub section199a_dividends: crate::ReitDividends,
    /// The box 14 code I statement contains only section 199A dividends for this portfolio K-1
    /// Confirm when section 199A dividends are entered. Business QBI, PTP amounts, W-2 wages, property basis and cooperative items need their own worksheets.
    pub code_i_dividends_only: bool,
    /// Box 3 net short-term capital gain — Schedule D line 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub short_gain: i64,
    /// Box 4a net long-term capital gain — Schedule D line 12
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub long_gain: i64,
    /// Box 4b 28% rate gain, included in box 4a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rate_28_gain: i64,
    /// Box 4c unrecaptured section 1250 gain, included in box 4a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured_gain: i64,
    /// Box 5 other portfolio income — entirely ordinary nonbusiness royalties reported on Schedule E line 33(f)
    /// Do not enter annuities, income in respect of a decedent or business royalties here. Enter supported directly apportioned depreciation, cost depletion or amortization separately below; other deductions need other calculations.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub royalty_income: i64,
    /// The entire box 5 amount is ordinary domestic nonbusiness royalties with no unentered NIIT adjustment
    /// Confirm when box 5 royalties are entered. A limited box 9 code A, B or C deduction below may come from the same activity; enter a separately reported box 14 code H adjustment below only when no box 9 deduction is entered. Other box 5 characters need their own calculations.
    pub box_5_royalty_only: bool,
    /// Box 9 code A depreciation directly apportioned to the box 5 sound-recording royalty activity
    /// Enter only the beneficiary's deduction from the fiduciary's per-activity statement, separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9_royalty_depreciation: i64,
    /// The entire box 9 code A deduction is confirmed section 167 straight-line depreciation of one fiduciary-held master sound recording
    /// Confirm the recording was licensed for domestic nonbusiness royalties and held by the fiduciary throughout 2026. Retain the fiduciary's per-asset basis, placed-in-service date, useful-life and beneficiary-allocation statement. The straight-line schedule began before 2026; no section 181 expensing, section 168(k) bonus, section 197 amortization, income-forecast look-back, other box 9 item, box 12 code G or box 14 code H applies. Regular, AMT and NIIT deductions agree, with no separate beneficiary limit or property succession.
    pub box_9_royalty_sound_recording_only: bool,
    /// Box 9 code B cost depletion directly apportioned to the box 5 mineral royalty activity
    /// Enter only the beneficiary's deduction from the fiduciary's per-activity statement, separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9_royalty_depletion: i64,
    /// The entire box 9 code B deduction is confirmed cost depletion on one fiduciary-held domestic nonbusiness mineral royalty property
    /// Confirm the fiduciary held the property throughout 2026 and supplied a per-property account showing its adjusted depletion basis, recoverable and sold units, cost-depletion calculation and complete beneficiary allocation. Retain that statement as the beneficiary's depletion record. Cost depletion must be the allowable method, not percentage depletion. Regular and AMT property bases and deductions agree, with no box 12 code H, box 14 code H overlap, separate beneficiary-level limit, or other box 9 deduction.
    pub box_9_royalty_cost_depletion_only: bool,
    /// Box 9 code C amortization directly apportioned to the box 5 royalty activity
    /// Enter the beneficiary's amount from the fiduciary's activity statement, not the trust's total amortization. It must be separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_9_royalty_amortization: i64,
    /// The box 9 code C deduction is one confirmed section 197 royalty-activity amortization
    /// Confirm a single domestic nonbusiness section 212 royalty activity, an eligible section 197 intangible with a pre-2026 straight-line schedule, the fiduciary's complete beneficiary allocation, and no other box 9 item. The deduction must be fully allowable for regular tax, AMT and NIIT in the same amount, with no box 12 code I, box 14 code H overlap, or separate beneficiary-level limit.
    pub box_9_royalty_amortization_only: bool,
    /// Fiduciary activity statements for boxes 6–9
    pub activities: crate::EstateActivities,
    /// Fiduciary-apportioned section 199A source statements
    pub qbi: crate::EstateQbiStatements,
    /// Box 14 code F gross farming and fishing income included in box 6 — Schedule E line 42
    /// Used only for the farmer and fisherman estimated-tax test; it does not add income again.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross_farming_fishing_income: i64,
    /// Box 10 section 691(c) estate-tax deduction on income in respect of a decedent — Schedule A line 17e
    /// Enter only the fiduciary-allocated amount attributable entirely to taxable IRD interest included in this K-1's box 1, with no box 14 code H adjustment that may already include it. Mixed or capital-gain IRD requires separate worksheets.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box_10_interest_ird_estate_tax: i64,
    /// The entire box 10 deduction is attributable to taxable IRD interest included in box 1
    /// Confirm from the fiduciary's allocation when box 10 is entered. The deduction is itemized and its properly allocable share reduces net investment income only when itemized.
    pub box_10_interest_ird_only: bool,
    /// Final-year box 11 code A section 67(e) excess deduction — Schedule 1 line 24k
    /// Enter only the amount allocated by the fiduciary to this beneficiary, not all estate or trust expenses. This deduction cannot be carried to another beneficiary tax year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_67e_deduction: i64,
    /// The box 11 code A deduction has no amount properly allocable to net investment income
    /// Confirm from the fiduciary's allocation when the deduction is entered. An investment-related part needs its separate Form 8960 treatment.
    pub final_67e_no_niit_allocation: bool,
    /// Final-year box 11 code B non-miscellaneous itemized deduction consisting of annual value-based personal property tax
    /// Enter only the amount allocated to this beneficiary by the fiduciary after the estate or trust's final-year income allocation. The beneficiary's own SALT limit applies; unused excess cannot carry to another beneficiary year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_personal_property_tax: i64,
    /// Final-year box 11 code B noninvestment, nonbusiness real-estate tax — Schedule A line 5b
    /// Enter only the fiduciary's beneficiary allocation of uniformly assessed real-property tax for general public purposes, with no business, rental or investment-use portion. It joins the beneficiary's own SALT limit; unused excess cannot carry to another year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_real_estate_tax: i64,
    /// Final-year box 11 code B state or local income tax — Schedule A line 5a if income taxes are elected
    /// Enter only the beneficiary amount remaining after the fiduciary's own section 164 limit. The beneficiary's SALT limit applies again. The tax has no Form 8960 investment-income allocation; otherwise use a separate worksheet. When general sales tax is elected, this income tax is not deducted.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_state_income_tax: i64,
    /// Final-year box 11 code B state or local general sales tax — Schedule A line 5a if sales taxes are elected
    /// Enter only the beneficiary amount remaining after the fiduciary's own section 164 limit. The beneficiary's SALT limit applies again. The amount is not deductible under the beneficiary's state-income-tax election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_state_sales_tax: i64,
    /// The entire box 11 code B amount is deductible state/local income or general sales tax, noninvestment real-estate tax or nonbusiness personal-property tax
    /// Confirm the fiduciary's separate statement and beneficiary succession. Other code B tax or itemized-deduction classes need their own worksheets.
    pub final_taxes_only: bool,
    /// Final-year box 11 code C short-term capital loss carryover — Schedule D line 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_short_carry: i64,
    /// Final-year box 11 code D long-term capital loss carryover — Schedule D line 12
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_long_carry: i64,
    /// The estate or trust terminated in 2026 and this beneficiary succeeds to its property
    /// Confirm when a box 11 code A, B, C, D, E or F amount is entered. The K-1 must show the final Form 1041 and allocated amount.
    /// [NullText]
    /// Entitlement is not established
    /// [SetText]
    /// This beneficiary succeeds to the terminated entity
    pub final_year_successor: Option<bool>,
    /// Printed Final K-1 mark (item E)
    pub final_k1: bool,
    /// Previously entered mortgage records for final-year deductions
    #[libertas_unordered]
    pub final_mortgages: Vec<i64>,
    /// Final-year capital loss carryovers have the same regular, AMT, investment-property and net-investment-income tax character
    /// Confirm from the fiduciary's records when either carryover is entered; a difference needs separate worksheets.
    pub final_carry_same_tax_character: bool,
    /// Final-year box 11 codes E/F regular and alternative-tax NOL carryovers by original loss year
    /// Enter the beneficiary's remaining balances after the estate's or trust's required carrybacks and intervening returns. Keep each original year and regular/AMT amount separate; the box totals are their sums.
    pub final_nol: Option<FinalEstateTrustNol>,
    /// Box 12 code A adjustment for minimum tax purposes — Form 6251 line 2j
    /// Enter the signed beneficiary adjustment shown on the K-1. It may stand alone or equal the sum of its confirmed code B qualified-dividend, C short-term and D long-term components. Codes E and F may be disjoint same-sign portions of D. Opposite-sign component netting, related income-based adjustments, loss-source reallocation and foreign-tax-credit allocation need separate worksheets.
    #[libertas_money("USD", 2)]
    pub amt_box12a: i64,
    /// Box 12 code A stands alone with no other AMT items or related adjustments
    /// Confirm when code A is entered. If boxes 12 B–J, a companion statement or related income-based adjustment applies, the AMT calculation needs its separate worksheet.
    pub amt_box12a_standalone: bool,
    /// Box 12 code B portion of code A attributable to qualified dividends
    /// Enter the signed amount shown by the fiduciary. It changes AMT preferential rates, not regular Form 1040 dividends.
    #[libertas_money("USD", 2)]
    pub amt_box12b: i64,
    /// Every box 12 code A component is identified and reconciled to codes B, C and D
    /// Confirm the signed code A amount exactly equals the sum of all reported B qualified-dividend, C short-term and D long-term adjustments, with no other code A component or related AMT statement omitted. Every nonzero component must have A's sign. Codes E/F are disjoint portions of D and need their own confirmations below. Separate investment-interest and foreign-exclusion worksheets remain outside this path.
    pub amt_box12_components_confirmed: bool,
    /// Box 12 code C portion of code A attributable to net short-term capital gain
    /// Enter the signed fiduciary adjustment. It changes AMT Schedule D line 5 and the AMT rate worksheet, not regular Schedule D or AGI; the same amount remains included once in code A.
    #[libertas_money("USD", 2)]
    pub amt_box12c: i64,
    /// Box 12 code D portion of code A attributable to net long-term capital gain
    /// Enter the signed fiduciary adjustment. It changes AMT Schedule D line 12 and the AMT rate worksheet, not regular Schedule D or AGI; the same amount remains included once in code A.
    #[libertas_money("USD", 2)]
    pub amt_box12d: i64,
    /// Box 12 code E portion of code A attributable to unrecaptured section 1250 gain
    /// Enter the signed fiduciary adjustment for line 11 of the AMT Unrecaptured Section 1250 Gain Worksheet. Code E may be all or a same-sign portion of D: section 1250 gain is part of long-term capital gain, not another addition to AMTI.
    #[libertas_money("USD", 2)]
    pub amt_box12e: i64,
    /// Box 12 code E is included in code D long-term gain adjustment
    /// Confirm the fiduciary identified code E within signed code D. If F is also reported, E and F are disjoint; the remainder of D after both is ordinary long-term gain. E must have D's sign, and the E/F magnitudes together cannot exceed D. Form 4952 and Form 2555 require separate coordination.
    pub amt_box12e_included_in_d: bool,
    /// Box 12 code F portion of code A attributable to 28%-rate gain
    /// Enter the signed fiduciary amount. Code F may be all or a same-sign portion of D. D refigures AMT Schedule D line 12; F refigures line 4 of the AMT 28% Rate Gain Worksheet without another AMTI addition.
    #[libertas_money("USD", 2)]
    pub amt_box12f: i64,
    /// Box 12 code F is included in code D long-term gain adjustment
    /// Confirm the fiduciary identified code F within signed code D. If E is also reported, E and F are disjoint; the remainder of D after both is ordinary long-term gain. F must have D's sign, and the E/F magnitudes together cannot exceed D. Form 4952 and Form 2555 require separate coordination.
    pub amt_box12f_included_in_d: bool,
    /// Box 13 code B backup withholding — Form 1040 federal withholding payments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub backup_withholding: i64,
    /// Box 13 code A estimated-tax payment allocated to this beneficiary — Form 1040 line 26
    /// Do not also enter this amount among the filer's own estimated payments.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub allocated_estimated_tax: i64,
    /// The fiduciary timely filed Form 1041-T allocating the box 13 code A payment to this beneficiary
    /// Item D of the K-1 shows it. The payment is treated as paid January 15, 2027 for Form 2210; without a timely Form 1041-T it stays the estate's or trust's payment and is left out.
    pub form_1041t_timely: bool,
    /// Box 14 code A tax-exempt interest — Form 1040 line 2a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tax_exempt_interest: i64,
    /// Specified private-activity-bond interest included in box 14 code A tax-exempt interest
    /// Enter only the fiduciary-identified beneficiary share; this is an AMT preference, not additional regular income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub private_activity_interest: i64,
    /// Box 14 code H net investment income adjustment, if shown
    /// Enter the fiduciary's separate Form 8960 line 7 adjustment and its modified-AGI effect. Do not repeat portfolio income already entered in boxes 1–5.
    pub niit_adjustment: Option<EstateTrustNiitAdjustment>,
    /// Other boxes on this K-1 with an amount
    // Tax structure: each supported other K-1 box identity occurs at most once.
    #[libertas_size(max = 13)]
    /// ----
    /// Box with an amount
    pub unentered: Vec<EstateTrustK1Item>,
    /// This K-1's income sources and beneficiary treatment are classified
    /// No means unclassified income, foreign-beneficiary or inconsistent-return facts need their owning worksheet. Known income remains included, but these facts prevent a complete federal estimate.
    pub complete_portfolio_only: bool,
    /// Actual current Schedule K-1 entity period and accounting convention
    /// Ordinary current-year sources may leave this absent. Retain actual dates and election statements for a 52/53-week source; do not replace dates with a deemed calendar year.
    #[libertas_default($null)]
    pub entity_period: Option<FinalEstateTrustTaxPeriod>,
    /// Box 12 code J exclusion items for the following year's minimum tax credit
    /// Signed fiduciary component of code A, retained without a second current-year AMT adjustment. Absent and reported zero remain distinct.
    #[libertas_money("USD", 2)]
    #[libertas_default($null)]
    pub minimum_tax_exclusion: Option<i64>,
}

/// Schedule K-1 (Form 1041) — Beneficiary's Share of Income, Deductions, Credits, etc.
/// The printed boxes as reported by the fiduciary; the follow-up answers and attached statements are stored beside it in `EstateTrustSource`.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1041K1 {
    /// Estate's or trust's EIN from item A; all nine digits without punctuation
    // Legal identifier: an EIN is exactly nine digits without punctuation.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern("[0-9]{9}")]
    #[libertas_llm_doc_field]
    pub ein: String,
    // Per-box amount bounds keep the portfolio preview sum within signed 64-bit cents.
    /// Box 1 taxable interest — Form 1040 line 2b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub interest: i64,
    /// Box 2a ordinary dividends — Form 1040 line 3b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub ordinary_dividends: i64,
    /// Box 2b qualified dividends, included in box 2a — Form 1040 line 3a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub qualified_dividends: i64,
    /// Box 3 net short-term capital gain — Schedule D line 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub short_gain: i64,
    /// Box 4a net long-term capital gain — Schedule D line 12
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub long_gain: i64,
    /// Box 4b 28% rate gain, included in box 4a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub rate_28_gain: i64,
    /// Box 4c unrecaptured section 1250 gain, included in box 4a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub unrecaptured_gain: i64,
    /// Box 5 other portfolio income — entirely ordinary nonbusiness royalties reported on Schedule E line 33(f)
    /// Do not enter annuities, income in respect of a decedent or business royalties here. Enter supported directly apportioned depreciation, cost depletion or amortization separately below; other deductions need other calculations.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub royalty_income: i64,
    /// Box 9 code A depreciation directly apportioned to the box 5 sound-recording royalty activity
    /// Enter only the beneficiary's deduction from the fiduciary's per-activity statement, separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub box_9_royalty_depreciation: i64,
    /// Box 9 code B cost depletion directly apportioned to the box 5 mineral royalty activity
    /// Enter only the beneficiary's deduction from the fiduciary's per-activity statement, separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub box_9_royalty_depletion: i64,
    /// Box 9 code C amortization directly apportioned to the box 5 royalty activity
    /// Enter the beneficiary's amount from the fiduciary's activity statement, not the trust's total amortization. It must be separate from box 5 income and no greater than that income in this supported nonloss path.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub box_9_royalty_amortization: i64,
    /// Box 14 code F gross farming and fishing income included in box 6 — Schedule E line 42
    /// Used only for the farmer and fisherman estimated-tax test; it does not add income again.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field]
    pub gross_farming_fishing_income: i64,
    /// Box 10 section 691(c) estate-tax deduction on income in respect of a decedent — Schedule A line 17e
    /// Enter only the fiduciary-allocated amount attributable entirely to taxable IRD interest included in this K-1's box 1, with no box 14 code H adjustment that may already include it. Mixed or capital-gain IRD requires separate worksheets.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub box_10_interest_ird_estate_tax: i64,
    /// Final-year box 11 code A section 67(e) excess deduction — Schedule 1 line 24k
    /// Enter only the amount allocated by the fiduciary to this beneficiary, not all estate or trust expenses. This deduction cannot be carried to another beneficiary tax year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub final_67e_deduction: i64,
    /// Final-year box 11 code C short-term capital loss carryover — Schedule D line 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub final_short_carry: i64,
    /// Final-year box 11 code D long-term capital loss carryover — Schedule D line 12
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field]
    pub final_long_carry: i64,
    /// Printed Final K-1 mark (item E)
    #[libertas_llm_doc_field]
    pub final_k1: bool,
    /// Final-year box 11 codes E/F regular and alternative-tax NOL carryovers by original loss year
    /// Enter the beneficiary's remaining balances after the estate's or trust's required carrybacks and intervening returns. Keep each original year and regular/AMT amount separate; the box totals are their sums.
    #[libertas_llm_doc_field]
    pub final_nol: Option<FinalEstateTrustNol>,
    /// Box 12 code A adjustment for minimum tax purposes — Form 6251 line 2j
    /// Enter the signed beneficiary adjustment shown on the K-1. It may stand alone or equal the sum of its confirmed code B qualified-dividend, C short-term and D long-term components. Codes E and F may be disjoint same-sign portions of D. Opposite-sign component netting, related income-based adjustments, loss-source reallocation and foreign-tax-credit allocation need separate worksheets.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12a: i64,
    /// Box 12 code B portion of code A attributable to qualified dividends
    /// Enter the signed amount shown by the fiduciary. It changes AMT preferential rates, not regular Form 1040 dividends.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12b: i64,
    /// Box 12 code C portion of code A attributable to net short-term capital gain
    /// Enter the signed fiduciary adjustment. It changes AMT Schedule D line 5 and the AMT rate worksheet, not regular Schedule D or AGI; the same amount remains included once in code A.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12c: i64,
    /// Box 12 code D portion of code A attributable to net long-term capital gain
    /// Enter the signed fiduciary adjustment. It changes AMT Schedule D line 12 and the AMT rate worksheet, not regular Schedule D or AGI; the same amount remains included once in code A.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12d: i64,
    /// Box 12 code E portion of code A attributable to unrecaptured section 1250 gain
    /// Enter the signed fiduciary adjustment for line 11 of the AMT Unrecaptured Section 1250 Gain Worksheet. Code E may be all or a same-sign portion of D: section 1250 gain is part of long-term capital gain, not another addition to AMTI.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12e: i64,
    /// Box 12 code F portion of code A attributable to 28%-rate gain
    /// Enter the signed fiduciary amount. Code F may be all or a same-sign portion of D. D refigures AMT Schedule D line 12; F refigures line 4 of the AMT 28% Rate Gain Worksheet without another AMTI addition.
    #[libertas_money("USD", 2)]
    #[libertas_llm_doc_field]
    pub amt_box12f: i64,
    /// Box 13 code B backup withholding — Form 1040 federal withholding payments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field]
    pub backup_withholding: i64,
    /// Box 13 code A estimated-tax payment allocated to this beneficiary — Form 1040 line 26
    /// Do not also enter this amount among the filer's own estimated payments.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field]
    pub allocated_estimated_tax: i64,
    /// Box 14 code A tax-exempt interest — Form 1040 line 2a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field]
    pub tax_exempt_interest: i64,
    /// Box 14 code H net investment income adjustment, if shown
    /// Enter the fiduciary's separate Form 8960 line 7 adjustment and its modified-AGI effect. Do not repeat portfolio income already entered in boxes 1–5.
    #[libertas_llm_doc_field]
    pub niit_adjustment: Option<EstateTrustNiitAdjustment>,
    /// Other boxes on this K-1 with an amount
    // Tax structure: each supported other K-1 box identity occurs at most once.
    #[libertas_size(max = 13)]
    #[libertas_llm_doc_field]
    /// ----
    /// Box with an amount
    pub unentered: Vec<EstateTrustK1Item>,
    /// Actual current Schedule K-1 entity period and accounting convention
    /// Ordinary current-year sources may leave this absent. Retain actual dates and election statements for a 52/53-week source; do not replace dates with a deemed calendar year.
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub entity_period: Option<FinalEstateTrustTaxPeriod>,
    /// Box 12 code J exclusion items for the following year's minimum tax credit
    /// Signed fiduciary component of code A, retained without a second current-year AMT adjustment. Absent and reported zero remain distinct.
    #[libertas_money("USD", 2)]
    #[libertas_default($null)]
    #[libertas_llm_doc_field]
    pub minimum_tax_exclusion: Option<i64>,
}

/// Fiduciary statement facts and their retained beneficiary treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateTrustStatements {
    /// Final-year source character and beneficiary entitlement
    pub final_character: Option<crate::EstateFinalCharacter>,
    /// Signed box 12 reconciliation and activity allocations
    pub amt_reconciliation: Option<crate::EstateAmtReconciliation>,
    /// Box 14 code E source net investment income for Form 4952
    pub investment_income: Option<crate::EstateInvestmentIncome>,
    /// Box 13 credit statements by their Form 1041 code identity
    // Tax structure: one statement per supported Form 1041 box 13 credit code.
    #[libertas_size(max = 19)]
    pub reported_credits: Vec<crate::EstateCreditSource>,
    /// Box 14 code B foreign tax statement
    pub foreign_statement: Option<crate::EstateForeignStatement>,
    /// Fiduciary investment-credit property source statements
    pub investment_property: Vec<crate::EstateInvestmentProperty>,
    /// Box 14 code I section 199A dividends included in box 2a ordinary dividends
    /// Enter only the beneficiary's apportioned qualified REIT/RIC dividends from the fiduciary's statement, separate from box 2b qualified dividends.
    pub section199a_dividends: crate::ReitDividends,
    /// The box 14 code I statement contains only section 199A dividends for this portfolio K-1
    /// Confirm when section 199A dividends are entered. Business QBI, PTP amounts, W-2 wages, property basis and cooperative items need their own worksheets.
    pub code_i_dividends_only: bool,
    /// The entire box 5 amount is ordinary domestic nonbusiness royalties with no unentered NIIT adjustment
    /// Confirm when box 5 royalties are entered. A limited box 9 code A, B or C deduction below may come from the same activity; enter a separately reported box 14 code H adjustment below only when no box 9 deduction is entered. Other box 5 characters need their own calculations.
    pub box_5_royalty_only: bool,
    /// The entire box 9 code A deduction is confirmed section 167 straight-line depreciation of one fiduciary-held master sound recording
    /// Confirm the recording was licensed for domestic nonbusiness royalties and held by the fiduciary throughout 2026. Retain the fiduciary's per-asset basis, placed-in-service date, useful-life and beneficiary-allocation statement. The straight-line schedule began before 2026; no section 181 expensing, section 168(k) bonus, section 197 amortization, income-forecast look-back, other box 9 item, box 12 code G or box 14 code H applies. Regular, AMT and NIIT deductions agree, with no separate beneficiary limit or property succession.
    pub box_9_royalty_sound_recording_only: bool,
    /// The entire box 9 code B deduction is confirmed cost depletion on one fiduciary-held domestic nonbusiness mineral royalty property
    /// Confirm the fiduciary held the property throughout 2026 and supplied a per-property account showing its adjusted depletion basis, recoverable and sold units, cost-depletion calculation and complete beneficiary allocation. Retain that statement as the beneficiary's depletion record. Cost depletion must be the allowable method, not percentage depletion. Regular and AMT property bases and deductions agree, with no box 12 code H, box 14 code H overlap, separate beneficiary-level limit, or other box 9 deduction.
    pub box_9_royalty_cost_depletion_only: bool,
    /// The box 9 code C deduction is one confirmed section 197 royalty-activity amortization
    /// Confirm a single domestic nonbusiness section 212 royalty activity, an eligible section 197 intangible with a pre-2026 straight-line schedule, the fiduciary's complete beneficiary allocation, and no other box 9 item. The deduction must be fully allowable for regular tax, AMT and NIIT in the same amount, with no box 12 code I, box 14 code H overlap, or separate beneficiary-level limit.
    pub box_9_royalty_amortization_only: bool,
    /// Fiduciary activity statements for boxes 6–9
    pub activities: Vec<crate::EstateActivity>,
    /// Fiduciary-apportioned section 199A source statements
    pub qbi: crate::EstateQbiStatements,
    /// The entire box 10 deduction is attributable to taxable IRD interest included in box 1
    /// Confirm from the fiduciary's allocation when box 10 is entered. The deduction is itemized and its properly allocable share reduces net investment income only when itemized.
    pub box_10_interest_ird_only: bool,
    /// The box 11 code A deduction has no amount properly allocable to net investment income
    /// Confirm from the fiduciary's allocation when the deduction is entered. An investment-related part needs its separate Form 8960 treatment.
    pub final_67e_no_niit_allocation: bool,
    /// Final-year box 11 code B non-miscellaneous itemized deduction consisting of annual value-based personal property tax
    /// Enter only the amount allocated to this beneficiary by the fiduciary after the estate or trust's final-year income allocation. The beneficiary's own SALT limit applies; unused excess cannot carry to another beneficiary year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_personal_property_tax: i64,
    /// Final-year box 11 code B noninvestment, nonbusiness real-estate tax — Schedule A line 5b
    /// Enter only the fiduciary's beneficiary allocation of uniformly assessed real-property tax for general public purposes, with no business, rental or investment-use portion. It joins the beneficiary's own SALT limit; unused excess cannot carry to another year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_real_estate_tax: i64,
    /// Final-year box 11 code B state or local income tax — Schedule A line 5a if income taxes are elected
    /// Enter only the beneficiary amount remaining after the fiduciary's own section 164 limit. The beneficiary's SALT limit applies again. The tax has no Form 8960 investment-income allocation; otherwise use a separate worksheet. When general sales tax is elected, this income tax is not deducted.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_state_income_tax: i64,
    /// Final-year box 11 code B state or local general sales tax — Schedule A line 5a if sales taxes are elected
    /// Enter only the beneficiary amount remaining after the fiduciary's own section 164 limit. The beneficiary's SALT limit applies again. The amount is not deductible under the beneficiary's state-income-tax election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub final_state_sales_tax: i64,
    /// The entire box 11 code B amount is deductible state/local income or general sales tax, noninvestment real-estate tax or nonbusiness personal-property tax
    /// Confirm the fiduciary's separate statement and beneficiary succession. Other code B tax or itemized-deduction classes need their own worksheets.
    pub final_taxes_only: bool,
    /// Final-year capital loss carryovers have the same regular, AMT, investment-property and net-investment-income tax character
    /// Confirm from the fiduciary's records when either carryover is entered; a difference needs separate worksheets.
    pub final_carry_same_tax_character: bool,
    /// Box 12 code A stands alone with no other AMT items or related adjustments
    /// Confirm when code A is entered. If boxes 12 B–J, a companion statement or related income-based adjustment applies, the AMT calculation needs its separate worksheet.
    pub amt_box12a_standalone: bool,
    /// Every box 12 code A component is identified and reconciled to codes B, C and D
    /// Confirm the signed code A amount exactly equals the sum of all reported B qualified-dividend, C short-term and D long-term adjustments, with no other code A component or related AMT statement omitted. Every nonzero component must have A's sign. Codes E/F are disjoint portions of D and need their own confirmations below. Separate investment-interest and foreign-exclusion worksheets remain outside this path.
    pub amt_box12_components_confirmed: bool,
    /// Box 12 code E is included in code D long-term gain adjustment
    /// Confirm the fiduciary identified code E within signed code D. If F is also reported, E and F are disjoint; the remainder of D after both is ordinary long-term gain. E must have D's sign, and the E/F magnitudes together cannot exceed D. Form 4952 and Form 2555 require separate coordination.
    pub amt_box12e_included_in_d: bool,
    /// Box 12 code F is included in code D long-term gain adjustment
    /// Confirm the fiduciary identified code F within signed code D. If E is also reported, E and F are disjoint; the remainder of D after both is ordinary long-term gain. F must have D's sign, and the E/F magnitudes together cannot exceed D. Form 4952 and Form 2555 require separate coordination.
    pub amt_box12f_included_in_d: bool,
    /// Specified private-activity-bond interest included in box 14 code A tax-exempt interest
    /// Enter only the fiduciary-identified beneficiary share; this is an AMT preference, not additional regular income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub private_activity_interest: i64,
    /// Original loss periods and beneficiary carryovers from fiduciary records
    pub final_nol: Option<FinalEstateTrustNol>,
    /// Fiduciary source allocation of the adjustment to modified adjusted gross income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub niit_adjustment: Option<i64>,
    /// Identified items requiring separate source treatment
    pub unentered: Vec<EstateTrustK1Item>,
}

/// Beneficiary answers not supplied by the printed K-1 or fiduciary statements
#[derive(
    Default, Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct EstateTrustAnswers {
    /// The estate or trust terminated in 2026 and this beneficiary succeeds to its property
    /// Confirm when a box 11 code A, B, C, D, E or F amount is entered. The K-1 must show the final Form 1041 and allocated amount.
    /// [NullText]
    /// Entitlement is not established
    /// [SetText]
    /// This beneficiary succeeds to the terminated entity
    pub final_year_successor: Option<bool>,
    /// Previously entered mortgage records for final-year deductions
    #[libertas_unordered]
    pub final_mortgages: Vec<i64>,
    /// This K-1's income sources and beneficiary treatment are classified
    /// No means unclassified income, foreign-beneficiary or inconsistent-return facts need their owning worksheet. Known income remains included, but these facts prevent a complete federal estimate.
    pub complete_portfolio_only: bool,
    /// Actual entity accounting election from fiduciary records
    pub entity_period: Option<FinalEstateTrustAccountingYear>,
}

/// Accepted printed Schedule K-1 (Form 1041) with the beneficiary's answers and statements
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateTrustSource {
    pub form: crate::K1EstateTrustPaper,
    pub statements: EstateTrustStatements,
    pub answers: EstateTrustAnswers,
}

impl EstateTrustSource {
    pub(crate) fn checked_record(
        &self,
    ) -> Result<FederalEstateTrustPortfolioK1, crate::validation::ValidationIssue> {
        let source = self;
        if let Some(issue) = source.form.admission_issue() {
            return Err(issue.in_field("form"));
        }
        let printed = source.form.checked_record().map_err(|issue| issue.in_field("form"))?;
        let Form1041K1 {
            ein,
            final_k1,
            entity_period: printed_period,
            interest,
            ordinary_dividends,
            qualified_dividends,
            short_gain,
            long_gain,
            rate_28_gain,
            unrecaptured_gain,
            royalty_income,
            box_9_royalty_depreciation,
            box_9_royalty_depletion,
            box_9_royalty_amortization,
            box_10_interest_ird_estate_tax,
            final_67e_deduction,
            final_short_carry,
            final_long_carry,
            final_nol: _,
            amt_box12a,
            amt_box12b,
            amt_box12c,
            amt_box12d,
            amt_box12e,
            amt_box12f,
            minimum_tax_exclusion,
            backup_withholding,
            allocated_estimated_tax,
            tax_exempt_interest,
            gross_farming_fishing_income,
            niit_adjustment: _,
            unentered: _,
        } = &printed;
        let EstateTrustStatements {
            final_character,
            amt_reconciliation,
            investment_income,
            reported_credits,
            foreign_statement,
            investment_property,
            section199a_dividends,
            code_i_dividends_only,
            box_5_royalty_only,
            box_9_royalty_sound_recording_only,
            box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization_only,
            activities,
            qbi,
            box_10_interest_ird_only,
            final_67e_no_niit_allocation,
            final_personal_property_tax,
            final_real_estate_tax,
            final_state_income_tax,
            final_state_sales_tax,
            final_taxes_only,
            final_carry_same_tax_character,
            amt_box12a_standalone,
            amt_box12_components_confirmed,
            amt_box12e_included_in_d,
            amt_box12f_included_in_d,
            private_activity_interest,
            final_nol,
            niit_adjustment,
            unentered,
        } = &source.statements;
        let EstateTrustAnswers {
            final_year_successor,
            final_mortgages,
            complete_portfolio_only,
            entity_period,
        } = &source.answers;
        let mut record = FederalEstateTrustPortfolioK1 {
            final_character: final_character.clone(),
            amt_reconciliation: amt_reconciliation.clone(),
            investment_income: investment_income.clone(),
            reported_credits: reported_credits
                .iter()
                .filter_map(crate::EstateCreditSource::native)
                .collect(),
            foreign_statement: foreign_statement.clone(),
            investment_property: investment_property.clone(),
            ein: ein.clone(),
            interest: *interest,
            ordinary_dividends: *ordinary_dividends,
            qualified_dividends: *qualified_dividends,
            section199a_dividends: section199a_dividends.clone(),
            code_i_dividends_only: *code_i_dividends_only,
            short_gain: *short_gain,
            long_gain: *long_gain,
            rate_28_gain: *rate_28_gain,
            unrecaptured_gain: *unrecaptured_gain,
            royalty_income: *royalty_income,
            box_5_royalty_only: *box_5_royalty_only,
            box_9_royalty_depreciation: *box_9_royalty_depreciation,
            box_9_royalty_sound_recording_only: *box_9_royalty_sound_recording_only,
            box_9_royalty_depletion: *box_9_royalty_depletion,
            box_9_royalty_cost_depletion_only: *box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization: *box_9_royalty_amortization,
            box_9_royalty_amortization_only: *box_9_royalty_amortization_only,
            activities: crate::EstateActivities {
                box6: source.form.business_income,
                box7: source.form.rental_real_estate_income,
                box8: source.form.other_rental_income,
                activities: activities.clone(),
            },
            qbi: qbi.clone(),
            gross_farming_fishing_income: *gross_farming_fishing_income,
            box_10_interest_ird_estate_tax: *box_10_interest_ird_estate_tax,
            box_10_interest_ird_only: *box_10_interest_ird_only,
            final_67e_deduction: *final_67e_deduction,
            final_67e_no_niit_allocation: *final_67e_no_niit_allocation,
            final_personal_property_tax: *final_personal_property_tax,
            final_real_estate_tax: *final_real_estate_tax,
            final_state_income_tax: *final_state_income_tax,
            final_state_sales_tax: *final_state_sales_tax,
            final_taxes_only: *final_taxes_only,
            final_short_carry: *final_short_carry,
            final_long_carry: *final_long_carry,
            final_year_successor: *final_year_successor,
            final_k1: *final_k1,
            final_mortgages: final_mortgages.clone(),
            final_carry_same_tax_character: *final_carry_same_tax_character,
            final_nol: final_nol.clone(),
            amt_box12a: *amt_box12a,
            amt_box12a_standalone: *amt_box12a_standalone,
            amt_box12b: *amt_box12b,
            amt_box12_components_confirmed: *amt_box12_components_confirmed,
            amt_box12c: *amt_box12c,
            amt_box12d: *amt_box12d,
            amt_box12e: *amt_box12e,
            amt_box12e_included_in_d: *amt_box12e_included_in_d,
            amt_box12f: *amt_box12f,
            amt_box12f_included_in_d: *amt_box12f_included_in_d,
            backup_withholding: *backup_withholding,
            allocated_estimated_tax: *allocated_estimated_tax,
            form_1041t_timely: source.form.filed_1041t,
            tax_exempt_interest: *tax_exempt_interest,
            private_activity_interest: *private_activity_interest,
            niit_adjustment: niit_adjustment.map(|modified_agi| EstateTrustNiitAdjustment {
                line_7: crate::estate_trust_paper::estate_code_amount(
                    &source.form.information,
                    "H",
                ),
                modified_agi,
            }),
            unentered: unentered.clone(),
            complete_portfolio_only: *complete_portfolio_only,
            entity_period: printed_period.as_ref().map(|p| FinalEstateTrustTaxPeriod {
                period: p.period.clone(),
                accounting: entity_period.clone().unwrap_or_else(|| p.accounting.clone()),
            }),
            minimum_tax_exclusion: *minimum_tax_exclusion,
        };
        if reported_credits.iter().any(|credit| credit.native().is_none())
            && !record.unentered.contains(&EstateTrustK1Item::Credits)
        {
            record.unentered.push(EstateTrustK1Item::Credits);
        }
        record.box_9_royalty_depreciation = crate::estate_trust_paper::estate_amount_residual(
            record.box_9_royalty_depreciation,
            record.activities.activities.iter().map(|a| a.depreciation),
        )
        .map_err(|issue| issue.at_field("deductions").in_field("form"))?;
        record.box_9_royalty_depletion = crate::estate_trust_paper::estate_amount_residual(
            record.box_9_royalty_depletion,
            record.activities.activities.iter().map(|a| a.depletion),
        )
        .map_err(|issue| issue.at_field("deductions").in_field("form"))?;
        record.box_9_royalty_amortization = crate::estate_trust_paper::estate_amount_residual(
            record.box_9_royalty_amortization,
            record.activities.activities.iter().map(|a| a.amortization),
        )
        .map_err(|issue| issue.at_field("deductions").in_field("form"))?;
        if !record.amounts_valid() {
            return Err(crate::estate_trust_paper::estate_amount_issue());
        }
        Ok(record)
    }
    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.checked_record().err()
    }
}
impl From<&EstateTrustSource> for FederalEstateTrustPortfolioK1 {
    fn from(source: &EstateTrustSource) -> Self {
        source.checked_record().expect("admitted estate K-1 source amounts")
    }
}

impl EstateTrustSource {
    /// The stored shape of an internal record: every field belongs to exactly one half.
    pub(crate) fn from_record(record: &FederalEstateTrustPortfolioK1) -> Self {
        let FederalEstateTrustPortfolioK1 {
            final_character,
            amt_reconciliation,
            investment_income,
            reported_credits,
            foreign_statement,
            investment_property,
            ein: _,
            interest: _,
            ordinary_dividends: _,
            qualified_dividends: _,
            section199a_dividends,
            code_i_dividends_only,
            short_gain: _,
            long_gain: _,
            rate_28_gain: _,
            unrecaptured_gain: _,
            royalty_income: _,
            box_5_royalty_only,
            box_9_royalty_depreciation: _,
            box_9_royalty_sound_recording_only,
            box_9_royalty_depletion: _,
            box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization: _,
            box_9_royalty_amortization_only,
            activities,
            qbi,
            gross_farming_fishing_income: _,
            box_10_interest_ird_estate_tax: _,
            box_10_interest_ird_only,
            final_67e_deduction: _,
            final_67e_no_niit_allocation,
            final_personal_property_tax,
            final_real_estate_tax,
            final_state_income_tax,
            final_state_sales_tax,
            final_taxes_only,
            final_short_carry: _,
            final_long_carry: _,
            final_year_successor,
            final_k1: _,
            final_mortgages,
            final_carry_same_tax_character,
            final_nol,
            amt_box12a: _,
            amt_box12a_standalone,
            amt_box12b: _,
            amt_box12_components_confirmed,
            amt_box12c: _,
            amt_box12d: _,
            amt_box12e: _,
            amt_box12e_included_in_d,
            amt_box12f: _,
            amt_box12f_included_in_d,
            backup_withholding: _,
            allocated_estimated_tax: _,
            form_1041t_timely: _,
            tax_exempt_interest: _,
            private_activity_interest,
            niit_adjustment,
            unentered,
            complete_portfolio_only,
            entity_period,
            minimum_tax_exclusion: _,
        } = record;
        Self {
            form: crate::K1EstateTrustPaper::from(record),
            statements: EstateTrustStatements {
                final_character: final_character.clone(),
                amt_reconciliation: amt_reconciliation.clone(),
                investment_income: investment_income.clone(),
                reported_credits: reported_credits.iter().map(Into::into).collect(),
                foreign_statement: foreign_statement.clone(),
                investment_property: investment_property.clone(),
                section199a_dividends: section199a_dividends.clone(),
                code_i_dividends_only: *code_i_dividends_only,
                box_5_royalty_only: *box_5_royalty_only,
                box_9_royalty_sound_recording_only: *box_9_royalty_sound_recording_only,
                box_9_royalty_cost_depletion_only: *box_9_royalty_cost_depletion_only,
                box_9_royalty_amortization_only: *box_9_royalty_amortization_only,
                activities: activities.activities.clone(),
                qbi: qbi.clone(),
                box_10_interest_ird_only: *box_10_interest_ird_only,
                final_67e_no_niit_allocation: *final_67e_no_niit_allocation,
                final_personal_property_tax: *final_personal_property_tax,
                final_real_estate_tax: *final_real_estate_tax,
                final_state_income_tax: *final_state_income_tax,
                final_state_sales_tax: *final_state_sales_tax,
                final_taxes_only: *final_taxes_only,
                final_carry_same_tax_character: *final_carry_same_tax_character,
                amt_box12a_standalone: *amt_box12a_standalone,
                amt_box12_components_confirmed: *amt_box12_components_confirmed,
                amt_box12e_included_in_d: *amt_box12e_included_in_d,
                amt_box12f_included_in_d: *amt_box12f_included_in_d,
                private_activity_interest: *private_activity_interest,
                final_nol: final_nol.clone(),
                niit_adjustment: niit_adjustment.as_ref().map(|a| a.modified_agi),
                unentered: unentered.clone(),
            },
            answers: EstateTrustAnswers {
                final_year_successor: *final_year_successor,
                final_mortgages: final_mortgages.clone(),
                complete_portfolio_only: *complete_portfolio_only,
                entity_period: entity_period.as_ref().map(|p| p.accounting.clone()),
            },
        }
    }
}

impl crate::community::Divide for EstateTrustSource {
    fn divided(&self, share: crate::community::Share) -> Self {
        Self::from_record(&FederalEstateTrustPortfolioK1::from(self).divided(share))
    }
}

impl FederalEstateTrustPortfolioK1 {
    pub(crate) fn final_successor(&self) -> bool {
        self.final_year_successor == Some(true)
            && self.final_character.as_ref().is_none_or(|facts| facts.entity_terminated)
    }
    pub(crate) fn valid(&self) -> bool {
        self.valid_with_mode(true)
    }
    pub(crate) fn valid_with_mode(&self, require_answers: bool) -> bool {
        crate::estate_trust::ein_valid(&self.ein)
            && self.amounts_valid()
            && if require_answers {
                crate::estate_trust::source_rule(self).is_none()
            } else {
                crate::estate_trust::source_issue_with_mode(self, false).is_none()
            }
            && if require_answers {
                self.activities.rule().is_none()
            } else {
                self.activities.issue_with_mode(false).is_none()
            }
            && if require_answers {
                self.qbi.rule(&self.activities).is_none()
            } else {
                self.qbi.issue_with_mode(&self.activities, false).is_none()
            }
            && self.qualified_dividends + self.section199a_dividends.amount()
                <= self.ordinary_dividends
            && self.rate_28_gain + self.unrecaptured_gain <= self.long_gain
            && self.private_activity_interest <= self.tax_exempt_interest
            && self.qualified_dividends + self.amt_box12b >= 0
            && self.final_nol.as_ref().is_none_or(FinalEstateTrustNol::valid)
            && self
                .unentered
                .iter()
                .enumerate()
                .all(|(index, item)| !self.unentered[..index].contains(item))
    }
    // Scalar admission precedes semantic owners that combine these native amounts.
    pub(crate) fn amounts_valid(&self) -> bool {
        [
            self.interest,
            self.ordinary_dividends,
            self.qualified_dividends,
            self.short_gain,
            self.long_gain,
            self.rate_28_gain,
            self.unrecaptured_gain,
            self.royalty_income,
            self.box_9_royalty_depreciation,
            self.box_9_royalty_depletion,
            self.box_9_royalty_amortization,
            self.box_10_interest_ird_estate_tax,
            self.final_67e_deduction,
            self.final_personal_property_tax,
            self.final_real_estate_tax,
            self.final_state_income_tax,
            self.final_state_sales_tax,
            self.final_short_carry,
            self.final_long_carry,
            self.backup_withholding,
            self.allocated_estimated_tax,
            self.tax_exempt_interest,
            self.private_activity_interest,
            self.gross_farming_fishing_income,
        ]
        .into_iter()
        .all(|amount| (0..=100_000_000_000).contains(&amount))
            && (0..=100_000_000_000).contains(&self.section199a_dividends.amount())
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12a)
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12b)
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12c)
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12d)
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12e)
            && (-100_000_000_000..=100_000_000_000).contains(&self.amt_box12f)
            && self.niit_adjustment.as_ref().is_none_or(EstateTrustNiitAdjustment::valid)
    }
}

/// Other K-1 box with an amount
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EstateTrustK1Item {
    /// Box 12 codes G, H or I — depreciation, depletion or amortization adjustments
    AmtItems,
    /// Box 13 credit not entered above (other than code R recapture)
    Credits,
    /// Box 13 code R — credit recapture statement requiring additional tax
    CreditRecapture,
    /// Unclassified box 14 information — the actual income, tax or return treatment is not determined
    /// Use the separately named category below for an identified unentered benefit. Unclassified income or added tax needs the fiduciary's statement before a complete estimate.
    OtherInformation,
    /// Box 14 code B — foreign taxes not entered in the foreign-tax statement
    ForeignTaxes,
    /// Box 14 code C — qualified rehabilitation expenditures not entered in investment property
    Rehabilitation,
    /// Box 14 code D — energy property basis not entered in investment property
    EnergyProperty,
    /// Box 14 code E — net investment income not entered in its Form 4952 source
    InvestmentIncome,
    /// Box 14 code G — foreign trading gross receipts; extraterritorial-income exclusion not calculated
    ForeignTradingGrossReceipts,
    /// Box 14 code J — qualifying advanced coal or gasification project property
    AdvancedCoalGasification,
    /// Box 14 code K — qualifying advanced energy project property
    AdvancedEnergy,
    /// Box 14 code L — advanced manufacturing investment property
    AdvancedManufacturing,
    /// Box 14 code M — clean electricity investment credit property not entered in investment property
    CleanElectricity,
}

impl EstateTrustK1Item {
    pub(crate) fn benefit_notice_key(self) -> Option<&'static str> {
        Some(match self {
            Self::ForeignTaxes => "WARN_ESTATE_BOX14_B_LEFT_OUT",
            Self::Rehabilitation => "WARN_ESTATE_BOX14_C_LEFT_OUT",
            Self::EnergyProperty => "WARN_ESTATE_BOX14_D_LEFT_OUT",
            Self::InvestmentIncome => "WARN_ESTATE_BOX14_E_LEFT_OUT",
            Self::ForeignTradingGrossReceipts => "WARN_ESTATE_BOX14_G_LEFT_OUT",
            Self::AdvancedCoalGasification => "WARN_ESTATE_BOX14_J_LEFT_OUT",
            Self::AdvancedEnergy => "WARN_ESTATE_BOX14_K_LEFT_OUT",
            Self::AdvancedManufacturing => "WARN_ESTATE_BOX14_L_LEFT_OUT",
            Self::CleanElectricity => "WARN_ESTATE_BOX14_M_LEFT_OUT",
            _ => return None,
        })
    }
    #[cfg(test)]
    pub(crate) fn refusal(self) -> Option<&'static str> {
        self.refusal_info().map(|info| info.1)
    }
    pub(crate) fn refusal_key(self) -> Option<&'static str> {
        self.refusal_info().map(|info| info.0)
    }
    pub(super) fn refusal_info(self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            Self::AmtItems => (
                "ISSUE_ESTATE_K1_AMT_ITEMS",
                crate::fixture_english!(
                    "An unentered Schedule K-1 (1041) box 12 code G, H or I needs its signed amount and applicable Form 6251 line or activity treatment. The mandatory AMT adjustment cannot be omitted."
                ),
            ),
            Self::CreditRecapture => (
                "ISSUE_ESTATE_K1_CREDIT_RECAPTURE",
                crate::fixture_english!(
                    "Schedule K-1 (1041) box 13 code R requires the applicable credit-recapture tax calculation; this additional tax cannot be omitted."
                ),
            ),
            Self::OtherInformation => (
                "ISSUE_ESTATE_K1_OTHER_INFORMATION",
                crate::fixture_english!(
                    "Unclassified Schedule K-1 (1041) box 14 information needs the fiduciary's actual income, tax and return treatment before a complete federal estimate. Identified credit, deduction or exclusion benefits have separate categories."
                ),
            ),
            Self::Credits
            | Self::ForeignTaxes
            | Self::Rehabilitation
            | Self::EnergyProperty
            | Self::InvestmentIncome
            | Self::ForeignTradingGrossReceipts
            | Self::AdvancedCoalGasification
            | Self::AdvancedEnergy
            | Self::AdvancedManufacturing
            | Self::CleanElectricity => return None,
        })
    }
}

impl FederalEstateTrustPortfolioK1 {
    pub(crate) fn business_net(&self) -> i64 {
        self.activities
            .activities
            .iter()
            .filter(|a| a.kind == crate::EstateActivityKind::Business)
            .map(|a| a.net(false))
            .sum()
    }
    pub(crate) fn rental_net(&self) -> i64 {
        self.activities
            .activities
            .iter()
            .filter(|a| a.kind != crate::EstateActivityKind::Business)
            .map(|a| a.net(false))
            .sum()
    }
    pub(crate) fn activity_net(&self) -> i64 {
        self.business_net() + self.rental_net()
    }
    pub(crate) fn has_activity_boxes(&self) -> bool {
        !self.activities.activities.is_empty()
    }
}

/// Final estate/trust NOL history succeeded to by a beneficiary
/// Keep every original loss year and separate regular/AMT balances. This is not the beneficiary's own prior-return history.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalEstateTrustNol {
    /// Date the estate or trust terminated
    /// Copy the final fiduciary statement's actual termination date when supplied. The first beneficiary carry year is the calendar year containing this date.
    /// [NullText]
    /// The fiduciary's termination date is unavailable
    /// [SetText]
    /// The estate or trust terminated on
    #[libertas_date_only]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 20260101, max = 20261231)]
    #[libertas_llm_doc_field]
    pub terminated_on: Option<u32>,
    /// Original estate/trust loss years
    /// Enter one record per actual original tax period from the fiduciary's complete successor statement. Distinct short years can begin in the same calendar year. Order does not change loss use.
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub years: Vec<FinalEstateTrustNolYear>,
    /// The fiduciary's statement establishes every original loss year and both remaining balances
    /// Confirm completed prior carrybacks, no special AMT exception, no NII source or section 461 component, and no duplicate excess deduction on termination. Check the fiduciary’s completed successor statement and prior carryback records. No leaves this loss out of this estimate.
    #[libertas_llm_doc_field]
    pub fiduciary_history_complete: bool,
    /// Actual fiduciary taxable periods, from the earliest entered loss through termination
    /// Copy consecutive periods from the fiduciary records, including every short year and its final year. A change stub of six days or less on a change to or from a 52/53-week year belongs to the following taxable-period row, not another year. For a following ordinary year, retain its normal month-first beginning separately. Leave the list empty if unavailable; older inherited loss benefits then remain uncomputed. Do not enter beneficiary tax years here.
    #[libertas_llm_doc_field]
    pub taxable_periods: Vec<FinalEstateTrustTaxPeriod>,
}

/// Original estate/trust loss year allocated to this beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalEstateTrustNolYear {
    /// Calendar year in which the original fiduciary loss tax year actually began
    /// Keep the actual original year. The accounting-period source determines any separate 52/53-week effective-date classification; it does not shift this original year.
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2026)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub year: u32,
    /// Actual original fiduciary tax year
    /// Copy the beginning and ending dates from the loss-year fiduciary return or successor statement. These dates distinguish short tax years and establish original-year order when losses from different sources share a calendar year. Choose “The original fiduciary tax-period dates are unavailable” when the dates are unknown; do not substitute termination dates or an expiration year.
    /// [NullText]
    /// The original fiduciary tax-period dates are unavailable
    /// [SetText]
    /// The original fiduciary tax period is
    #[libertas_llm_doc_field]
    pub original_period: Option<FinalEstateTrustLossPeriod>,
    /// Beneficiary carry period remaining, including 2026
    /// For a loss from a tax year beginning before 2018, copy the remaining carry period from the fiduciary's successor statement after its final tax year. Its final year and your first year each count, including short tax years. Choose “No remaining carry period is supplied for this loss” for a loss from 2018 or later. An unknown older period leaves that year's loss benefit out.
    /// [NullText]
    /// No remaining carry period is supplied for this loss
    /// [SetText]
    /// The beneficiary carry period remaining, including 2026, is
    // Tax structure: this pre-2018 NOL has a twenty-year carryforward period.
    #[libertas_number(min = 1, max = 20)]
    #[libertas_physical_unit("year")]
    #[libertas_llm_doc_field]
    pub remaining_years: Option<u32>,
    /// Box 11 code E remaining regular NOL allocated to this beneficiary
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub regular_balance: i64,
    /// Box 11 code F remaining AMT NOL allocated to this beneficiary
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub alternative_balance: i64,
}

/// Actual fiduciary taxable-period dates
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalEstateTrustLossPeriod {
    /// Tax year began
    #[libertas_date_only]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 19000101, max = 20271231)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub began: u32,
    /// Tax year ended
    #[libertas_date_only]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 19000101, max = 20271231)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub ended: u32,
}
/// Accounting basis of one actual fiduciary taxable period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FinalEstateTrustAccountingYear {
    /// Calendar, fiscal or ordinary short tax year
    CalendarFiscalOrShort,
    /// Elected 52/53-week tax year, including its first or final short year
    /// Copy the normal annual boundaries and reference month-end from the fiduciary accounting election/returns. The actual period may be shortened by creation or termination; a change stub of at most six days may precede the normal beginning and is not another taxable year.
    Weeks52Or53 {
        /// Normal annual beginning, before any first-year shortening or appended change stub
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 18990101, max = 20261231)]
        #[libertas_llm_doc_field(allow_empty = false)]
        annual_began: u32,
        /// Normal annual ending, before any final-year termination
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 19000101, max = 20271231)]
        #[libertas_llm_doc_field(allow_empty = false)]
        annual_ended: u32,
        /// Last calendar day of the election's reference closing month
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 19000101, max = 20271231)]
        #[libertas_llm_doc_field(allow_empty = false)]
        reference_ended: u32,
        /// Election's annual closing convention
        #[libertas_llm_doc_field]
        closing: FinalEstateTrustWeekClosing,
    },
    /// Ordinary year following an approved change from a 52/53-week year
    /// The change stub of one through six days is part of this one taxable period. Copy its normal ordinary beginning and keep the actual combined dates unchanged.
    CalendarFiscalAfterWeeks52Or53 {
        /// Normal ordinary beginning, after the appended change stub
        #[libertas_date_only]
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 19000101, max = 20271231)]
        #[libertas_llm_doc_field(allow_empty = false)]
        annual_began: u32,
    },
}

/// How the elected weekday relates to the reference month-end
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FinalEstateTrustWeekClosing {
    /// Last occurrence of that weekday in the reference month
    Last,
    /// Occurrence of that weekday nearest the reference month-end
    Nearest,
}

/// One actual fiduciary taxable period and its accounting convention
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalEstateTrustTaxPeriod {
    /// Actual period on the fiduciary return, including an appended change stub
    #[libertas_llm_doc_field]
    pub period: FinalEstateTrustLossPeriod,
    /// Actual accounting convention for this period
    #[libertas_llm_doc_field]
    pub accounting: FinalEstateTrustAccountingYear,
}
impl FinalEstateTrustTaxPeriod {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |field| {
            crate::validation::ValidationIssue::new("ESTATE_NOL_PERIOD_INVALID").at_field(field)
        };
        if let Some(issue) = self.period.issue() {
            return Some(issue.in_field("period"));
        }
        let p = &self.period;
        match &self.accounting {
            FinalEstateTrustAccountingYear::CalendarFiscalOrShort => {
                let (by, bm, bd) = (p.began / 10000, p.began / 100 % 100, p.began % 100);
                let (ey, em, ed) = (p.ended / 10000, p.ended / 100 % 100, p.ended % 100);
                (!(ey == by || (ey == by + 1 && (em, ed) < (bm, bd))))
                    .then(|| issue("ended").in_field("period"))
            }
            FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 { annual_began } => {
                if !crate::federal_rules::valid_date(*annual_began)
                    || !(19000101..=20271231).contains(annual_began)
                    || annual_began % 100 != 1
                {
                    Some(issue("annual_began").in_field("accounting"))
                } else if !(1..=6).contains(&crate::dates::days_between(p.began, *annual_began)) {
                    Some(issue("began").in_field("period"))
                } else if p.ended < *annual_began || p.ended >= (*annual_began + 10000) {
                    Some(issue("ended").in_field("period"))
                } else {
                    None
                }
            }
            FinalEstateTrustAccountingYear::Weeks52Or53 {
                annual_began,
                annual_ended,
                reference_ended,
                closing,
            } => {
                for (field, date, minimum, maximum) in [
                    ("annual_began", *annual_began, 18990101, 20261231),
                    ("annual_ended", *annual_ended, 19000101, 20271231),
                    ("reference_ended", *reference_ended, 19000101, 20271231),
                ] {
                    if !crate::federal_rules::valid_date(date)
                        || !(minimum..=maximum).contains(&date)
                    {
                        return Some(issue(field).in_field("accounting"));
                    }
                }
                let length = crate::dates::days_between(*annual_began, *annual_ended);
                let before = crate::dates::days_between(*annual_began, p.began);
                let close = crate::dates::days_between(*reference_ended, *annual_ended);
                let prior_reference = crate::dates::last_day_of_month(
                    reference_ended / 10000 - 1,
                    reference_ended / 100 % 100,
                );
                // Keep the existing normal annual and prior closing convention owner.
                let prior_close = crate::dates::days_between(prior_reference, *annual_began) - 1;
                if !matches!(length, 363 | 370) {
                    return Some(issue("annual_ended").in_field("accounting"));
                }
                if before < -6 || p.began > *annual_ended {
                    return Some(issue("began").in_field("period"));
                }
                if p.ended > *annual_ended {
                    return Some(issue("ended").in_field("period"));
                }
                if *reference_ended
                    != crate::dates::last_day_of_month(
                        *reference_ended / 10000,
                        *reference_ended / 100 % 100,
                    )
                {
                    return Some(issue("reference_ended").in_field("accounting"));
                }
                let closing_range = match closing {
                    FinalEstateTrustWeekClosing::Last => -6..=0,
                    FinalEstateTrustWeekClosing::Nearest => -3..=3,
                };
                if !closing_range.contains(&close) {
                    Some(issue("annual_ended").in_field("accounting"))
                } else if !closing_range.contains(&prior_close) {
                    Some(issue("annual_began").in_field("accounting"))
                } else {
                    None
                }
            }
        }
    }
    // Reg1.441-2(c)(1): effective-date laws use the nearest month-first,
    // including the retained actual first day of a first/last shortened year.
    pub(crate) fn effective_year(&self) -> u32 {
        let p = &self.period;
        if !matches!(self.accounting, FinalEstateTrustAccountingYear::Weeks52Or53 { .. }) {
            return p.began / 10000;
        }
        let (year, month) = (p.began / 10000, p.began / 100 % 100);
        let first = year * 10000 + month * 100 + 1;
        let next = if month == 12 { (year + 1) * 10000 + 101 } else { first + 100 };
        if crate::dates::days_between(first, p.began) <= crate::dates::days_between(p.began, next) {
            year
        } else {
            next / 10000
        }
    }
}
impl FinalEstateTrustLossPeriod {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |field| {
            crate::validation::ValidationIssue::new("ESTATE_NOL_PERIOD_INVALID").at_field(field)
        };
        if !crate::federal_rules::valid_date(self.began)
            || !(19000101..=20271231).contains(&self.began)
        {
            Some(issue("began"))
        } else if !crate::federal_rules::valid_date(self.ended)
            || !(self.began..=20271231).contains(&self.ended)
        {
            Some(issue("ended"))
        } else {
            None
        }
    }
}
impl FinalEstateTrustNol {
    pub(crate) fn post_2017(
        loss: &FinalEstateTrustNolYear,
        periods: &[FinalEstateTrustTaxPeriod],
    ) -> Option<bool> {
        if periods.is_empty() {
            return (loss.year >= 2018).then_some(true);
        }
        periods
            .iter()
            .find(|p| Some(&p.period) == loss.original_period.as_ref())
            .map(|p| p.effective_year() >= 2018)
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |field| {
            crate::validation::ValidationIssue::new("ESTATE_NOL_PERIOD_INVALID").at_field(field)
        };
        if self.terminated_on.is_some_and(|date| {
            !crate::federal_rules::valid_date(date) || !(20260101..=20261231).contains(&date)
        }) {
            return Some(issue("terminated_on"));
        }
        for (index, p) in self.taxable_periods.iter().enumerate() {
            if let Some(issue) = p.issue() {
                return Some(issue.in_list("taxable_periods", index));
            }
            if p.period.ended > 20261231 {
                return Some(issue("ended").in_field("period").in_list("taxable_periods", index));
            }
        }
        let mut periods: Vec<_> = self.taxable_periods.iter().enumerate().collect();
        periods.sort_by_key(|(_, p)| p.period.began);
        if !periods.is_empty() {
            if periods.last().map(|(_, p)| p.period.ended) != self.terminated_on {
                return Some(issue("terminated_on"));
            }
            if let Some(pair) = periods.windows(2).find(|pair| {
                crate::dates::days_between(pair[0].1.period.ended, pair[1].1.period.began) != 1
            }) {
                return Some(
                    issue("began").in_field("period").in_list("taxable_periods", pair[1].0),
                );
            }
        }
        for (index, loss) in self.years.iter().enumerate() {
            let loss_issue = |field| issue(field).in_list("years", index);
            if !(1900..=2026).contains(&loss.year) {
                return Some(loss_issue("year"));
            }
            if let Some(period) = &loss.original_period {
                if let Some(issue) = period.issue() {
                    return Some(issue.in_field("original_period").in_list("years", index));
                }
                if period.ended > 20261231
                    || self.terminated_on.is_some_and(|date| period.ended > date)
                {
                    return Some(
                        issue("ended").in_field("original_period").in_list("years", index),
                    );
                }
                if period.began / 10000 != loss.year {
                    return Some(
                        issue("began").in_field("original_period").in_list("years", index),
                    );
                }
            }
            if loss.remaining_years.is_some_and(|years| {
                !(1..=20).contains(&years)
                    || Self::post_2017(loss, &self.taxable_periods) == Some(true)
            }) {
                return Some(loss_issue("remaining_years"));
            }
            if !periods.is_empty() {
                let Some(position) = periods
                    .iter()
                    .position(|(_, p)| Some(&p.period) == loss.original_period.as_ref())
                else {
                    return Some(loss_issue("original_period"));
                };
                if loss
                    .remaining_years
                    .is_some_and(|n| n as i64 != 20 - (periods.len() - position - 1) as i64)
                {
                    return Some(loss_issue("remaining_years"));
                }
            }
            if loss.regular_balance == 0 && loss.alternative_balance == 0 {
                return Some(
                    crate::validation::ValidationIssue::new("NOL_NOTHING_ENTERED")
                        .at_field("regular_balance")
                        .in_list("years", index),
                );
            }
            for prior in &self.years[..index] {
                if prior.year == loss.year
                    && (prior.original_period.is_none()
                        || loss.original_period.is_none()
                        || prior.original_period.as_ref().map(|p| p.began)
                            == loss.original_period.as_ref().map(|p| p.began))
                {
                    return Some(
                        crate::validation::ValidationIssue::new("NOL_YEAR_TWICE")
                            .at_field("year")
                            .in_list("years", index),
                    );
                }
                if let (Some(a), Some(b)) = (&prior.original_period, &loss.original_period)
                    && a.began <= b.ended
                    && b.began <= a.ended
                {
                    return Some(
                        issue("began").in_field("original_period").in_list("years", index),
                    );
                }
            }
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.rule().is_none()
            && !self.years.is_empty()
            && self.years.iter().all(|loss| {
                // Arithmetic safety: the source-confirmation summary still sums
                // this statement's cents natively. Nonoverlapping original
                // periods within 1900..2026 plus these scalar bounds protect it;
                // pooled cross-statement carriers use exact arithmetic.
                (0..=100_000_000_000).contains(&loss.regular_balance)
                    && (0..=100_000_000_000).contains(&loss.alternative_balance)
            })
    }
}

/// Estate or trust box 14 code H adjustment for the beneficiary
/// Form 8960 line 7 is separate from the income boxes. A positive amount also increases modified AGI by the same amount; for a negative amount use the fiduciary's statement to determine whether any part changes modified AGI.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateTrustNiitAdjustment {
    /// Form 8960 line 7 change to net investment income
    /// Positive or negative as shown in the fiduciary's box 14 code H statement. Do not duplicate interest, dividends, gains or royalties already entered above.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    pub line_7: i64,
    /// Change to Form 8960 modified adjusted gross income
    /// For a positive line 7 amount, repeat that amount here. For a negative line 7 amount, enter the signed part that the fiduciary identifies as also changing modified AGI; enter zero only when none does.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub modified_agi: i64,
}

impl EstateTrustNiitAdjustment {
    pub(crate) fn valid(&self) -> bool {
        (-100_000_000_000..=100_000_000_000).contains(&self.line_7)
            && (-100_000_000_000..=100_000_000_000).contains(&self.modified_agi)
            && if self.line_7 > 0 {
                self.modified_agi == self.line_7
            } else {
                self.modified_agi.unsigned_abs() <= self.line_7.unsigned_abs()
            }
    }
}

// Community-property allocation of this source record; facts retain their meaning.
impl crate::community::Divide for FederalEstateTrustPortfolioK1 {
    fn divided(&self, share: crate::community::Share) -> Self {
        // Community projection precedes the normal restored-data check. Leave
        // invalid sources intact for that owner to reject before deriving sums.
        if !self.valid() {
            return self.clone();
        }
        let Self {
            final_character,
            amt_reconciliation,
            investment_income,
            reported_credits,
            foreign_statement,
            investment_property,
            interest,
            ordinary_dividends,
            qualified_dividends,
            section199a_dividends,
            royalty_income,
            box_9_royalty_depreciation,
            box_9_royalty_depletion,
            box_9_royalty_amortization,
            activities,
            qbi,
            gross_farming_fishing_income,
            box_10_interest_ird_estate_tax,
            final_67e_deduction,
            final_personal_property_tax,
            final_real_estate_tax,
            final_state_income_tax,
            final_state_sales_tax,
            final_nol,
            amt_box12a,
            amt_box12b,
            backup_withholding,
            allocated_estimated_tax,
            tax_exempt_interest,
            private_activity_interest,
            niit_adjustment,
            minimum_tax_exclusion,
            short_gain,
            long_gain,
            rate_28_gain,
            unrecaptured_gain,
            final_short_carry,
            final_long_carry,
            amt_box12c,
            amt_box12d,
            amt_box12e,
            amt_box12f,
            ein,
            code_i_dividends_only,
            box_5_royalty_only,
            box_9_royalty_sound_recording_only,
            box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization_only,
            box_10_interest_ird_only,
            final_67e_no_niit_allocation,
            final_taxes_only,
            final_year_successor,
            final_k1,
            final_mortgages,
            final_carry_same_tax_character,
            amt_box12a_standalone,
            amt_box12_components_confirmed,
            amt_box12e_included_in_d,
            amt_box12f_included_in_d,
            form_1041t_timely,
            unentered,
            complete_portfolio_only,
            entity_period,
        } = self;
        let mut projected = Self {
            final_character: final_character.divided(share),
            amt_reconciliation: amt_reconciliation.divided(share),
            investment_income: investment_income.divided(share),
            reported_credits: reported_credits.divided(share),
            foreign_statement: foreign_statement.divided(share),
            investment_property: investment_property.divided(share),
            interest: interest.divided(share),
            ordinary_dividends: ordinary_dividends.divided(share),
            qualified_dividends: qualified_dividends.divided(share),
            section199a_dividends: section199a_dividends.divided(share),
            royalty_income: royalty_income.divided(share),
            box_9_royalty_depreciation: box_9_royalty_depreciation.divided(share),
            box_9_royalty_depletion: box_9_royalty_depletion.divided(share),
            box_9_royalty_amortization: box_9_royalty_amortization.divided(share),
            activities: activities.divided(share),
            qbi: qbi.divided(share),
            gross_farming_fishing_income: gross_farming_fishing_income.divided(share),
            box_10_interest_ird_estate_tax: box_10_interest_ird_estate_tax.divided(share),
            final_67e_deduction: final_67e_deduction.divided(share),
            final_personal_property_tax: final_personal_property_tax.divided(share),
            final_real_estate_tax: final_real_estate_tax.divided(share),
            final_state_income_tax: final_state_income_tax.divided(share),
            final_state_sales_tax: final_state_sales_tax.divided(share),
            final_nol: final_nol.divided(share),
            amt_box12a: amt_box12a.divided(share),
            amt_box12b: amt_box12b.divided(share),
            backup_withholding: backup_withholding.divided(share),
            allocated_estimated_tax: allocated_estimated_tax.divided(share),
            tax_exempt_interest: tax_exempt_interest.divided(share),
            private_activity_interest: private_activity_interest.divided(share),
            niit_adjustment: niit_adjustment.divided(share),
            minimum_tax_exclusion: minimum_tax_exclusion.divided(share),
            short_gain: short_gain.divided(share.for_gains()),
            long_gain: long_gain.divided(share.for_gains()),
            rate_28_gain: rate_28_gain.divided(share.for_gains()),
            unrecaptured_gain: unrecaptured_gain.divided(share.for_gains()),
            final_short_carry: final_short_carry.divided(share.for_gains()),
            final_long_carry: final_long_carry.divided(share.for_gains()),
            amt_box12c: amt_box12c.divided(share.for_gains()),
            amt_box12d: amt_box12d.divided(share.for_gains()),
            amt_box12e: amt_box12e.divided(share.for_gains()),
            amt_box12f: amt_box12f.divided(share.for_gains()),
            ein: ein.clone(),
            code_i_dividends_only: *code_i_dividends_only,
            box_5_royalty_only: *box_5_royalty_only,
            box_9_royalty_sound_recording_only: *box_9_royalty_sound_recording_only,
            box_9_royalty_cost_depletion_only: *box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization_only: *box_9_royalty_amortization_only,
            box_10_interest_ird_only: *box_10_interest_ird_only,
            final_67e_no_niit_allocation: *final_67e_no_niit_allocation,
            final_taxes_only: *final_taxes_only,
            final_year_successor: *final_year_successor,
            final_k1: *final_k1,
            final_mortgages: final_mortgages.clone(),
            final_carry_same_tax_character: *final_carry_same_tax_character,
            amt_box12a_standalone: *amt_box12a_standalone,
            amt_box12_components_confirmed: *amt_box12_components_confirmed,
            amt_box12e_included_in_d: *amt_box12e_included_in_d,
            amt_box12f_included_in_d: *amt_box12f_included_in_d,
            form_1041t_timely: *form_1041t_timely,
            unentered: unentered.clone(),
            complete_portfolio_only: *complete_portfolio_only,
            entity_period: entity_period.clone(),
        };
        crate::estate_trust::reconcile_community_source(self, &mut projected, share);
        projected
    }
}
