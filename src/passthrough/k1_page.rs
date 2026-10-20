// Coverage update B9-7 R9 inputs (2026-10-12): regular and independent AMT
//   section 179 sale lists plus personal sold-property carryover map losslessly
//   through both K-1 pages, typed storage, reopen, restart and correction.
//   Imported printed lists never select the personal carryover field. Sale
//   dates/basis and regular/AMT carryover shares have exact client/server twins.
//   Tests: interview.rs k1_sale_inputs_, passthrough.rs import selection.
//   Calculation-owner integration remains open; estimates keep the existing
//   OtherInformation refusal whenever either tax system has sale statements.
// Coverage update B9-7 business-property checkpoint (2026-10-12): Partnership business property, section 1231 carries and qualified statement amount have printed-field import, exact signed client/server validation, display tabs and lossless Save/reopen/correction/restart mapping.
//   Evidence: tests/passthrough/k1_items.rs and tests/passthrough/interview.rs k1_partner_property_*; i1065sk1/i4797/i8582/i8995 (2025 final), f4797 2026 draft. Full B9-7 acceptance remains open.
// Coverage: Schedule K-1 page types (unit T2-40p, 2026-09-27). Unit form-3468 (2026-10-09): the Other
//   boxes of the Schedule K-1 (1065) and (1120-S) pages take investment_property (Form 3468:
//   box 15 / 13 code E, box 20 codes D, E, AV / box 17 codes C, D, AV) and the personal
//   investment_confirmed answer, required when the list is not empty
//   (K1_INVESTMENT_NEEDS_CONFIRMATION) and never imported; the AI import reads the list.
//   The Schedule K-1 (Form 1041) document kind (IncomeDocument::K1EstateTrust, topic
//   TaxSection::PassThrough) maps field for field onto FederalEstateTrustPortfolioK1 and back
//   (K1EstateTrust::record / from_record; reopening a saved page gives the same page). Each
//   stored confirmation is a branch: box 5 Royalties writes box_5_royalty_only Yes and the one
//   box 9 deduction's *_only answer Yes (only when that amount is above zero, which the
//   calculation requires); boxes 6-8 Amounts writes business_rental_confirmed Yes unless its
//   special is ticked; Other boxes: box 10 above zero writes box_10_interest_ird_only Yes, Final
//   writes final_year_successor, final_67e_no_niit_allocation, final_taxes_only and
//   final_carry_same_tax_character Yes, CodeAOnly writes amt_box12a_standalone Yes, Components
//   writes amt_box12_components_confirmed true and codes E/F included-in-D true when E/F is not
//   zero, Allocated writes form_1041t_timely from filed_1041t, and box 14 code I dividends above zero are
//   ReitDividends::Reported { eligible Yes } with code_i_dividends_only Yes. The first-screen
//   special writes complete_portfolio_only false, which leaves the K-1 out with a notice. Rules on both
//   sides (K1EstateTrust::rule on save, same keys): K1_1041_QUALIFIED_ABOVE_2A,
//   K1_1041_4B_4C_ABOVE_4A, K1_1041_PRIVATE_ABOVE_14A, K1_1041_ROYALTY_DEDUCTION_ABOVE_BOX_5 and
//   the five T2-40 keys on K1EstateTrustActivities. Other boxes with an amount are the new stored
//   FederalEstateTrustPortfolioK1.unentered (EstateTrustK1Item: box 12 G-I, box 13 C-T/ZZ, box 14
//   B/C/D/E/G/J-M/ZZ). Identified box14 benefits now have distinct categories
//   and named omissions; unknown income/added tax retains a limitation.
//   Original category indices are preserved; storage integration is external.
//   Shared page types for the 1065 and 1120-S kinds (A3, built ahead of T2-29p and T2-31p):
//   Participation (onto ScorpParticipation; Investor and Over500Hours are the C12 shortcuts
//   whose written answers make material() return No and Yes), LimitedParticipation (onto
//   PartnerParticipation::Limited), PartnerAmtRecords and ScorpAmtRecords (the
//   design's AmtRecords, one per stored AMT type because a schema type cannot be generic;
//   default SameAsRegular per C10) and K1IncomeSource (onto income_source and
//   gross_personal_services; K1IncomeSource::allowed for the ConstrainedBy set, allowed_partner_source
//   since T2-cy, see interview.rs's T2-cy block; T2-src: Both carries
//   WorkValue, the 30% answer or an amount). They are not
//   reachable from a protocol type until those units place them.
//   Tests: tests/passthrough/interview.rs k1_1041_* and shared_k1_*.
//   T2-31p (2026-09-27; design A3b, C10/C12/C13 as recommended): the Schedule K-1 (Form 1120-S)
//   kind (IncomeDocument::K1Scorp) maps onto ScorpK1 and back (K1Scorp::record / from_record).
//   First screen boxes 1, 4, 5a, 5b, 16D, the box 17 code V statement (NoStatement is
//   qbi_statement None; Statement writes complete Yes), Participation (Some), AmtRecords
//   (ScorpAmtRecords, default SameAsRegular); Other boxes 7, 8a-8c, 9, 11, the section 179
//   carryover (C13), 12A, 13O, 15F, 16A-C, the stock holding period (NotNeeded is stock_term
//   None), trading (FinancialTrading, else Ordinary) and the refused ScorpK1Item list; the basis
//   fold (Form 7203 and 6198): New (risk.start New, every carryover 0) or Continuing (opening,
//   carried basis, capital, section 1231, at-risk and passive losses, losses_first, the at-risk
//   history and the QBI loss histories), additions, loans (box 16E is each loan's repayments),
//   amounts not at risk. capital and section1231 are Some when any current or carried amount is
//   nonzero, complete true. special writes basis_complete false and risk/passive complete false.
//   Rules on both sides (K1Scorp::rule on save, same keys): SCORP_QUALIFIED_ABOVE_DIVIDENDS,
//   SCORP_QBI_ABOVE_BOX_1, SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT, SCORP_BOX_TWICE,
//   SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION, SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG.
//   STORAGE_FORMAT 62. Tests: tests/passthrough/interview.rs k1_1120s_*.
//   T2-29p (2026-09-27, design A3a): the Schedule K-1 (Form 1065) kind K1Partnership maps onto
//   PartnerK1 and back (record/from_record). PartnerInterest: General writes limited_for_se No,
//   LimitedPartner Yes with PartnerParticipation::Limited, LlcMember follows box 14 code A (C11:
//   box1_in_se ticked No, else Yes) with General participation; a ticked LLC member writes a
//   general partner's record and reopens as General. Participation and LimitedParticipation use
//   the C12 shortcuts; PartnerAmtRecords writes alternative (C10); K1IncomeSource writes
//   income_source and gross_personal_services. The Other boxes fold writes separate (Some when any
//   of its eleven amounts is not zero), section179, the new section179_carryover (C13),
//   regular.exempt (18B), regular.nondeductible (18C), fuel_tax_credit, prior_fuel_credit,
//   trade_kind (FinancialTrading when trading, else Ordinary) and unentered. PartnerBasis:
//   BasisStart::New writes zero history and risk.start New; Continuing writes basis.opening, the
//   basis, at-risk and passive carryovers, risk.start from risk_history (default Unknown, C9) and
//   se_history (zero parts when nothing is carried). special writes complete,
//   regular.basis.complete and regular.risk.complete No. Health insurance, health_guaranteed,
//   health_distributions and retirement are not on the page (C15). Rules on both sides:
//   PARTNER_DEBT_ABOVE_LIABILITIES, PARTNER_SE_CARRY_ABOVE_LOSS, PARTNER_QUALIFIED_ABOVE_DIVIDENDS
//   (moved to the fold), PARTNER_BOX_TWICE (is_unique_set; unique(1) is rejected on a scalar
//   list); Continuing at-risk history is constrained by the delivered context and checked on save.
//   T3-small (2026-09-27): the 1120-S twin SCORP_RISK_HISTORY_NEW (Continuing with a New at-risk
//   start) is unavailable in the editor and refused by name on save (K1Scorp::rule); tests/passthrough/interview.rs
//   k1_1120s_continuing_shareholder_with_new_at_risk_start_is_refused_by_name.
//   T2-31r (2026-09-27, question C9 for shareholders): K1ScorpStart::Continuing.risk_history
//   defaults to Unknown ("Missing history"); scorp_risk.rs skips Form 6198 for it when the
//   activity's losses do not exceed its income. Test: tests/passthrough/interview.rs
//   k1_1120s_page_without_at_risk_history_reproduces_the_profit_fixture.
//   T2-scr2 fold F-A (2026-09-27): on both
//   basis folds Continuing keeps opening and risk_history and moves every other carryover into
//   carried: K1ScorpCarried (1120-S: the carried basis, capital, section 1231, at-risk and
//   passive losses, losses_first, deducted/recaptured and the QBI loss histories) or
//   PartnerCarried (1065: basis, at-risk and passive losses, deducted/recaptured, the SE parts),
//   None ("No losses or expenses carried from 2025", the default) or Some. None writes every
//   carried amount zero (no loss_qbi/property_qbi, losses_first false, se_history zero); reopen
//   shows Some when the record carries any nonzero amount, the election or a QBI history (the
//   None page would write a different record). Page types only: STORAGE_FORMAT unchanged. The
//   client rules SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION, SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG
//   and PARTNER_SE_CARRY_ABOVE_LOSS read [start/carried/...] behind `[start/carried] is Some`,
//   their server twins match the Some branch. Tests: tests/passthrough/interview.rs
//   k1_1120s_carried_losses_fold_round_trips_none_and_some,
//   k1_1065_carried_losses_fold_round_trips_none_and_some; the existing k1_1120s_*/k1_1065_*
//   fixtures and rule boundary tests reach the carryovers through Some unchanged.
//   T2-43p (2026-09-27): the Other boxes fold gains box 13 code M
//   (medical_insurance -> health_guaranteed) and box 13 code R (pensions -> PartnerK1.pensions)
//   after 13A, in box order; PARTNER_MEDICAL_ABOVE_4A on both sides. The page keeps only
//   health_insurance, health_distributions and retirement from the accepted record.
//   Batch 7 follow-up (2026-10-05): the S-corporation QBI loss tracking (loss_qbi, property_qbi)
//   moved from K1ScorpCarried::Some into K1ScorpQbi::Statement. SCORP_QBI_LOSS_RECORDS_NEEDED (box 1
//   loss or carried ordinary loss) and SCORP_QBI_PROPERTY_RECORDS_NEEDED (records exactly with a
//   section 1231 item) on both sides replace the calculation refusals. AI Import does not select the
//   section: importing its statement amounts needs the optional container marked, and a null
//   result would clear typed histories (reported as a capability gap). Page types only:
//   STORAGE_FORMAT unchanged. Test:
//   tests/passthrough/interview.rs k1_1120s_qbi_loss_tracking_follows_the_values.
//   Batch 7 non-nullable QBI tracking (2026-10-05, inventor decision): loss_qbi and property_qbi
//   are non-nullable on the page (folded closed in Statement) and on ScorpK1: zero and empty lists
//   when nothing applies, written empty without a statement. SCORP_QBI_LOSS_RECORDS_NEEDED and
//   SCORP_QBI_PROPERTY_RECORDS_NEEDED check values on both sides (ScorpLossQbi::records_cover,
//   ScorpPropertyQbi::records_cover): a QBI loss portion fits the box 1 / box 9 loss, each carried
//   loss has a loss-year history, and the section 1231 records are empty without a section 1231
//   item. AI Import selects the statement leaves (loss_qbi.current, property_qbi.gain,
//   qualified_1250, losses.current) and merges them into the existing tracking; the histories and
//   the confirmation are not selected. STORAGE_FORMAT 85. Test: tests/passthrough/passthrough.rs
//   k1_ai_import_selects_the_printed_boxes_only.
//   Gated property items (same change): the page's property_qbi is the required union
//   K1ScorpPropertyQbi (NoSection1231Item | Section1231Items { gain, qualified_1250, current,
//   complete }) for this year's items only (box 9, box 8c business part), which the statement
//   decides; the carried section 1231 loss histories are the always-present, non-imported
//   property_history (K1ScorpLossHistory), so a variant change never deletes them. The
//   confirmation is asked only with this year's items (it attests this year's allocation; the
//   histories reconcile by validity), and is written true otherwise. SCORP_QBI_PROPERTY_RECORDS_NEEDED:
//   the variant matches this year's items, the current loss fits box 9, each carried loss has its
//   history, and no history without any section 1231 item. The stored ScorpK1.property_qbi is
//   unchanged. Tests: tests/passthrough/interview.rs k1_1120s_qbi_loss_tracking_follows_the_values,
//   k1_1120s_import_without_section1231_item_keeps_carried_histories.
//   Gate fixes, batch C (2026-10-06, "Ask once, directly"): the three Other boxes switches
//   become always-present records (K1EstateTrustOtherBoxes, K1ScorpOtherBoxes,
//   K1PartnershipOtherBoxes, folded closed, tabbed by display.json). Each printed box keeps its
//   llm_doc_field; the person's own fields (final_year, section199a_dividends,
//   section179_carryover, stock_term, trading, fuel_credit, prior_fuel_credit) are unmarked, so
//   an import no longer clears them by choosing Nothing. K1ScorpPropertyQbi is the record
//   { gain, qualified_1250, current }: the K-1's own box 9 and box 8c business part decide
//   whether amounts may be entered (SCORP_QBI_PROPERTY_RECORDS_NEEDED refuses amounts without
//   them), so the shareholder is no longer asked the item question. K1ScorpCarried,
//   PartnerCarried, K1ScorpAtRisk and PartnerAtRisk are always-present amount records (zero is
//   none). Tests: tests/passthrough/interview.rs k1_1120s_qbi_loss_tracking_follows_the_values;
//   tests/passthrough/passthrough.rs k1_ai_import_selects_the_printed_boxes_only.
//   B9-7 R6: rental section1231/unrecaptured parts are printed import targets. Full-sale
//   selections and separate rental property PALs are personal records, validated on both
//   sides and preserved by Save/reopen/restart/correction. Evidence: k1_rental_property_*.
//   Unit k1-rentals (2026-10-09): both kinds gain rentals (K1RentalBoxes: boxes 2 and 3 and the
//   statement's section 199A rental amounts imported; the participation answer is the filer's own,
//   default NotActive) and the carried rental_real_estate_passive_loss/other_rental_passive_loss on
//   PartnerCarried and K1ScorpCarried; the amounts write K1RentalAmounts on the regular record
//   (the AMT record copies them when SameAsRegular), the answers K1RentalFacts.
//   PARTNER_LIMITED_NOT_ACTIVE on both sides. The import declarations name boxes 2 and 3,
//   RentalDisposition and the narrowed MultipleActivities. The at-risk debt fields include the
//   partner's qualified nonrecourse financing. Tests: tests/passthrough/interview.rs k1_1065_rentals_*,
//   k1_1120s_rentals_*, k1_1065_limited_partner_cannot_actively_participate_on_both_sides.
//   Unit k3 (2026-10-09): both kinds gain schedule_k3 (K3Attached: NotAttached or Attached(ScheduleK3),
//   stored as PartnerK1/ScorpK1.schedule_k3) and the box 21 / box 16 code F foreign taxes in the
//   other boxes (imported). The K-1 import never reads box 16 (1120-S box 14): schedule_k3 is
//   unmarked, since a marked Union imports its variant's marked payload and would replace the
//   K-3; the ScheduleK3 type has its own AI Import. The page rules add K3_ITEM_TWICE and
//   FOREIGN_QUALIFIED_ABOVE_INCOME (ScheduleK3::rule). PartnerK1Item/ScorpK1Item::International
//   are removed from the unentered lists and both import instructions. Tests: tests/passthrough/interview.rs
//   k3_on_a_1065_k1_maps_onto_the_record_and_reopens_unchanged, k3_on_a_1120s_k1_*,
//   k3_rules_reject_on_both_sides; tests/passthrough/passthrough.rs k1_ai_import_*.
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
    instruction = "Read this Schedule K-1 (Form 1041) and the fiduciary's attached statements exactly. Read `entity_period` from the actual current entity taxable-period dates and accounting election. Ordinary calendar, fiscal and short periods belong to the supplied beneficiary year when they end in that year. A normal elected 52/53-week year ending with reference to December belongs to the calendar beneficiary year of its reference December month-end, even when its actual ending is in early January of the next year (Reg. 1.441-2(e)); do not reject that source solely because its actual ending year differs. For Weeks52Or53, require the election's normal annual beginning and ending, reference month-end and Last/Nearest closing convention from the source statements; report missing or conflicting election facts as an error instead of inferring them. For CalendarFiscalAfterWeeks52Or53, require the actual approved change from a 52/53-week year and its normal month-first ordinary beginning; the preceding one-through-six-day stub belongs to that following period, not a separate tax year. Keep actual dates unchanged. A shortened elected December-reference period ending in a different calendar year from its reference month-end needs separate source-year verification and is not supported by the normal annual exception. Report a genuine beneficiary-year mismatch as an error. Never import original NOL periods or taxable-period history from the current entity-period header. Report illegible or conflicting values as errors, and never infer a fact the K-1 or its statements do not show. Parentheses or a minus sign mark a negative amount; return amounts with their printed sign. `ein` is item A, exactly nine digits without punctuation, preserving leading zeroes; a missing, masked or illegible EIN is an error. `name` is item B, the estate's or trust's name as printed. Box 1 is `interest`, box 2a `ordinary_dividends`, box 2b `qualified_dividends`, box 3 `short_gain`, box 4a `long_gain`, box 4b `rate_28_gain` and box 4c `unrecaptured_gain`; return zero for a clearly blank box. `more` holds boxes 10 through 14; return zero for a clearly blank amount there. Box 10 is `ird_estate_tax`. Box 12 is `amt`: None when box 12 shows none of codes A through F, CodeAOnly when code A is its only code among A through F, and Components when any of codes B through F is shown (code A must be shown; a blank code among B through F is zero). Box 12 code J is `minimum_tax_exclusion`, the signed exclusion-item component retained for the following year; return null when code J is absent and zero only for an explicitly reported zero. Do not add code J to code A or to any current-year AMT adjustment. Box 13 code A is `estimated_tax` Allocated with its amount, with `filed_1041t` true only when item D shows that Form 1041-T was filed; without code A it is None. Box 13 code B is `backup_withholding`. Box 14 code A is `tax_exempt_interest`, and `private_activity_interest` is the part of it the fiduciary's statement identifies as private activity bond interest (zero when none is shown). Box 14 code H is `niit`: `line_7` is its signed amount and `modified_agi` equals `line_7` when positive; when negative it is the signed part the statement identifies as also changing modified adjusted gross income (zero when the statement says none does; report an error when the statement does not say). `niit` is null when code H is absent. Return zero for a clearly blank amount. `unentered` lists each category at most once: AmtItems for box 12 codes G, H or I; Credits for unentered box 13 credits other than R; CreditRecapture for box 13 code R; ForeignTaxes for unentered box 14 code B; Rehabilitation for unentered code C; EnergyProperty for unentered code D; InvestmentIncome for unentered code E; ForeignTradingGrossReceipts for code G; AdvancedCoalGasification for code J; AdvancedEnergy for code K; AdvancedManufacturing for code L; CleanElectricity for unentered code M; OtherInformation for unclassified code ZZ information. Do not add an unentered category when its complete actual statement has already been entered in the corresponding separate source fields. The imported `unentered` list replaces the existing list. Read only marked source fields. Activity and QBI current-source rows have their own import actions; never replace their unmarked identities, participation or historical balances. Do not infer successor entitlement or source legal classifications.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K1EstateTrust {
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
    /// Final-year source character and entitlement are not established
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
    /// Estate's or trust's EIN from item A; all nine digits without punctuation
    #[libertas_llm_doc_field(allow_empty = false)]
    // Legal source identifier: EINs contain exactly nine digits.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern("[0-9]{9}")]
    pub ein: String,
    /// Estate's or trust's name (item B)
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub name: String,
    /// Box 1 interest income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Box 2a ordinary dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub ordinary_dividends: i64,
    /// Box 2b qualified dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 3 net short-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub short_gain: i64,
    /// Box 4a net long-term capital gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub long_gain: i64,
    /// Box 4b 28% rate gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rate_28_gain: i64,
    /// Box 4c unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_foldable(initially_open = false)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Code B cost depletion of one mineral royalty property
    /// Cost depletion on one domestic nonbusiness mineral royalty property the fiduciary held all year, from its per-property statement, with the same amount for regular tax and AMT.
    Depletion {
        /// Box 9 code B amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Code C section 197 amortization
    /// Straight-line amortization begun before 2026 for this one royalty activity, fully allowable with the same amount for regular tax, AMT and net investment income.
    Amortization {
        /// Box 9 code C amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    /// Qualified REIT and RIC dividends from the fiduciary's statement, included in box 2a and separate from box 2b. Entering them confirms their holding-period rules are met.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section199a_dividends: i64,
    /// Other boxes with an amount
    /// ----
    /// Box with an amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        section_67e: i64,
        /// Code B personal property tax
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        personal_property_tax: i64,
        /// Code B real estate tax
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        real_estate_tax: i64,
        /// Code B state or local income tax
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        state_income_tax: i64,
        /// Code B state or local general sales tax
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        state_sales_tax: i64,
        /// Code C short-term capital loss carryover
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        short_carry: i64,
        /// Code D long-term capital loss carryover
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
fn empty_record() -> FederalEstateTrustPortfolioK1 {
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
        final_year_successor: false,
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
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if !crate::estate_trust::ein_valid(&self.ein) {
            return Some(ValidationIssue::new("ESTATE_EIN_INVALID").at_field("ein"));
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
                if let Some(issue) = activities.issue() {
                    return Some(issue.in_field("business"));
                }
                activities.record()
            }
        };
        self.qbi.issue(&activities).map(|issue| issue.in_field("qbi")).or_else(|| {
            crate::estate_trust::source_issue(&self.record()).map(|issue| {
                issue
                    .map_field_prefix(&["activities"], &["business"])
                    .map_field_prefix(&["entity_period", "period"], &["entity_period"])
                    .map_field_prefix(
                        &["minimum_tax_exclusion"],
                        &["more", "minimum_tax_exclusion"],
                    )
                    .map_field_prefix(&["final_nol"], &["more", "final_year", "nol"])
            })
        })
    }

    // The stored record this page writes.
    pub(crate) fn record(&self) -> FederalEstateTrustPortfolioK1 {
        let mut r = empty_record();
        r.ein = self.ein.clone();
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
                unentered,
            } = &self.more;
            r.minimum_tax_exclusion = *minimum_tax_exclusion;
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
                r.final_year_successor = self
                    .final_character
                    .as_ref()
                    .is_some_and(|f| f.entity_terminated && f.successor);
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
            if *section199a_dividends > 0 {
                r.section199a_dividends =
                    ReitDividends::Reported { amount: *section199a_dividends, eligible: true };
                r.code_i_dividends_only = true;
            }
            r.unentered = unentered.clone();
        }
        r.complete_portfolio_only = !self.special;
        r
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written.
    pub(crate) fn from_record(name: &str, r: &FederalEstateTrustPortfolioK1) -> Option<Self> {
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
            ein: r.ein.clone(),
            name: name.into(),
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
                final_year: if r.final_year_successor
                    || r.final_character.is_some()
                    || r.final_67e_deduction != 0
                    || r.final_short_carry != 0
                    || r.final_long_carry != 0
                    || r.final_personal_property_tax != 0
                    || r.final_real_estate_tax != 0
                    || r.final_state_income_tax != 0
                    || r.final_state_sales_tax != 0
                    || r.final_nol.is_some()
                {
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
                unentered: r.unentered.clone(),
            },
            special: !r.complete_portfolio_only,
        };
        if r.activities.activities.is_empty() {
            page.business = K1EstateTrustBusiness::None;
        }
        (page.record() == *r).then_some(page)
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
        /// Years from 2016 through 2025 you materially participated in it
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_default(0)]
        prior_years: u32,
    },
    /// You and your spouse worked in it more than 500 hours
    Over500Hours {
        /// Hours you and your spouse worked in it
        // Unresolved cap: record() multiplies hours by 60 before native validation;
        // remove this bound only with a checked conversion and its existing failure path.
        #[libertas_number(min = 501, max = 8784)]
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
        /// Years from 2016 through 2025 you materially participated in it
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_default(0)]
        prior_years: u32,
    },
    /// You and your spouse worked in it more than 500 hours
    Over500Hours {
        /// Hours you and your spouse worked in it
        // Unresolved cap: record() multiplies hours by 60 before native validation;
        // remove this bound only with a checked conversion and its existing failure path.
        #[libertas_number(min = 501, max = 8784)]
        hours: u32,
    },
    /// Something else, such as 100 to 500 hours
    Other {
        /// Combined qualifying work by the partner and spouse, in minutes
        /// Exclude investor-only work and work performed to evade the passive-loss rules.
        #[libertas_number(min = 0)]
        minutes: u32,
        /// Material-participation years among 2016 through 2025
        /// Count them from your 2016 through 2025 returns: the years Schedule E Part II line 28 reported this activity as nonpassive (column (i) or (k)) because you materially participated. Do not establish those years solely through the historical five-of-ten test. Enter zero when there were none.
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        prior_ten: u32,
        /// This is a personal-service activity
        /// Health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting or another field where capital is not a material income-producing factor (Reg. §1.469-5T(d)).
        personal_service: bool,
        /// Earlier material-participation years in this personal-service activity
        /// Count them from your earlier returns: every year before 2026 that Schedule E Part II reported this activity as nonpassive (line 28 column (i) or (k)) because you materially participated. Enter zero when there were none or when it is not a personal-service activity.
        #[libertas_number(min = 0)]
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
    #[libertas_default(0)]
    pub real_estate: i64,
    /// Section 1231 gain (loss) of the rental real estate activity
    /// The part of box 10 (Form 1120-S box 9) the statement assigns to the box 2 activity. Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub real_estate_section1231: i64,
    /// Unrecaptured section 1250 gain of the rental real estate activity
    /// The part of the business-property amount in box 9c (Form 1120-S box 8c) from the box 2 activity.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub real_estate_unrecaptured: i64,
    /// Box 3 other net rental income (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub other: i64,
    /// Section 1231 gain (loss) of the other rental activity
    /// The part of box 10 (Form 1120-S box 9) the statement assigns to the box 3 activity. Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub other_section1231: i64,
    /// Unrecaptured section 1250 gain of the other rental activity
    /// The part of the business-property amount in box 9c (Form 1120-S box 8c) from the box 3 activity.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_unrecaptured: i64,
    /// How you took part in the rental real estate (box 2)
    #[libertas_default(NotActive)]
    pub participation: crate::K1RentalParticipation,
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
    pub carryover_active: bool,
    /// Rental activities whose property the entity sold in full in 2026
    /// All of the activity's property, sold or exchanged for cash or other property in a fully taxable sale to a buyer not related to you or to the entity (not an installment sale, a like-kind exchange or a casualty). The activity's suspended passive losses are then allowed. Leave empty when none.
    /// ----
    /// Activity
    #[libertas_unordered]
    // Tax structure: only box 2 real estate and box 3 other rental activities.
    #[libertas_size(max = 2)]
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
    /// No AMT items on this K-1, and your AMT basis and carryovers equal your regular ones
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
    /// No AMT items on this K-1, and your AMT basis and carryovers equal your regular ones
    SameAsRegular,
    /// Different for AMT
    Separate {
        /// Your AMT records for this corporation
        record: crate::ScorpAmt,
    },
}

impl ScorpAmtRecords {
    // The regular records restated as the AMT record.
    pub(crate) fn same_as(regular: &crate::ScorpK1) -> crate::ScorpAmt {
        crate::ScorpAmt {
            adjustment: 0,
            capital: regular.capital.clone(),
            section1231: regular.section1231.clone(),
            stock: regular.opening,
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
        }
    }
    /// The stored AMT record: the regular records restated when the filer
    /// states they are the same.
    pub fn record(&self, regular: &crate::ScorpK1) -> crate::ScorpAmt {
        match self {
            Self::SameAsRegular => Self::same_as(regular),
            Self::Separate { record } => record.clone(),
        }
    }
    /// The choice a stored AMT record reopens as.
    pub fn from_record(regular: &crate::ScorpK1, amt: &crate::ScorpAmt) -> Self {
        if Self::same_as(regular) == *amt {
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
        #[libertas_number(min = 0)]
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

/// Schedule K-1 (Form 1120-S)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this Schedule K-1 (Form 1120-S) and the corporation's attached statements exactly. It belongs to the supplied year only when the entity's tax year ends in that year (a calendar-year K-1 for that year, or a fiscal year ending in it); report a mismatch as an error. Read `reporting_period` only from this K-1 entity’s actual current beginning and ending dates; return null when unavailable, and never borrow another source’s period. Report illegible or conflicting values as errors, and never infer a fact the K-1 or its statements do not show. Parentheses or a minus sign mark a negative amount; return amounts with their printed sign. `name` is item B, the corporation's name as printed; `final_k1` is true only when the Final K-1 box is checked. Box 1 is `ordinary`; return an empty string when box 1 is clearly blank. `rentals` holds box 2 as `real_estate` and box 3 as `other`; return zero for a clearly blank box. `rentals.real_estate_qbi` holds the section 199A statement's W-2 wages (`wages`) and unadjusted basis of qualified property (`property`) for the box 2 rental real estate activity when the statement lists it as a trade or business, and is null when the statement does not list it or no statement is attached; `rentals.other_qbi` holds the same for the box 3 activity. Never read `rentals.participation`: it comes from the shareholder's own records. Box 4 is `interest`, box 5a `dividends`, box 5b `qualified_dividends`, box 6 `royalties` and box 16 code D `distributions`; return zero for a clearly blank amount. Box 17 code V is `qbi`: Statement when the section 199A statement is attached, with the qualified business income of the box 1 trade or business as `income`, its W-2 wages as `wages` and its unadjusted basis of qualified property as `property` (0 for an amount the statement shows as zero or blank), `service` true only when the statement marks a specified service trade or business, `reit_dividends` the statement's section 199A dividends and `ptp_income` its qualified PTP items (signed; 0 when not shown). Also from the statement: `current_loss` is the qualified business loss included in a box 1 loss, as a positive amount (0 when box 1 is not a loss); `property_qbi` holds the statement's section 1231 amounts: when box 9 and the business-property part of box 8c are both blank, `gain` and `current` are 0 and `qualified_1250` is null; otherwise `gain` is the qualified portion of a box 9 net section 1231 gain and `current` the qualified portion of a box 9 net section 1231 loss, as a positive amount (0 when there is none), and `qualified_1250` is the qualified part of the box 8c business section 1250 gain when the statement shows it, null when it does not. The retained `qbi_records` come from the shareholder's own worksheets and confirmation and must never be imported; NoStatement when the K-1 shows no code V; report an error when code V refers to a statement that is not attached. `more.section179_sales` lists each property on the box 17 code K statement sold in a fully taxable sale: `description`, date acquired and placed in service as `acquired`, date sold as `sold`, and your shares in the `regular` monetary record: gross sales price (`price`), cost or other basis plus selling expenses (`cost`), depreciation excluding section 179 (`depreciation`), and section 179 deduction passed through (`section179`). Never read `alternative` or `id`: AMT source facts and accepted identity are not imported. Return an empty list when the statement shows no such sale. The imported list replaces the existing sale statements. Never read `more.amt_section179_income_carry`: it comes from the person’s own prior AMT Form 4562 history and must be preserved independently of the regular amount. Never read `more.section179_carryover_sold`: it comes from the person's own Form 4562 carryover records and must be preserved. `more` holds the other boxes: box 7 is `short`, box 8a `long`, box 8b `collectibles`; box 8c is split by the statement: `unrecaptured_partnership` for the part from selling a partnership interest, `unrecaptured_fund` for the part from estates, trusts, mutual funds or REITs, `rentals.real_estate_unrecaptured` and `rentals.other_unrecaptured` for the parts from the box 2 and box 3 activities, and `unrecaptured_business` for the rest from the box 1 business. Box 9 is split the same way, signed: `rentals.real_estate_section1231` and `rentals.other_section1231` for the parts the statement assigns to the box 2 and box 3 activities and `section1231` for the rest (all of box 9 when the statement assigns none to a rental activity); report an error when the statement does not determine which activity owns an amount; box 11 `section179`, box 12 code A `cash_charity`, box 13 code O `withholding`, box 15 code F `private_activity`, box 16 code A `exempt_interest`, code B `exempt`, code C `nondeductible` and code F `foreign_taxes`; return zero for a clearly blank amount. `income` lists each box 10 code at most once with its signed amount: code A as PortfolioIncome only when it is income and the statement reports no REMIC residual interest, C as Section1256 unless the statement reports a straddle, J as TaxBenefitRecovery, K as GamblingWinnings and GamblingLosses (the losses as a positive amount) unless the statement shows a gambling business, and S as NonportfolioShort or NonportfolioLong by the term the statement shows. `deductions` lists each box 12 code other than A at most once with its amount (B CashThirty, C NoncashFifty, D NoncashThirty, E CapitalGainPropertyThirty, F CapitalGainPropertyTwenty, G ContributionsHundred, H InvestmentInterest, I RoyaltyDeductions, J Section59e, L PortfolioOther, M Preproductive, O Reforestation, W SoilWater, X Production, Y Barriers, Z Itemized, AA ConstructionFund, AB EarlyWithdrawalPenalty, AC DistributionInterest, ZZ Other). `credits` lists each supported box 13 code once per code and reported Form 3800 line with its amount: J (WorkOpportunity), N (EmployerTipFica), AE (PensionStartup), AF (AutoEnrollment), AG (MilitarySpouse), BA (SmallEmployerHealth), BB (PaidLeave), C (LowIncomeHousingJ5), D (LowIncomeHousingOther), A (ZeroEmissionNuclear), B (AdvancedNuclear), F (OtherRentalRealEstate), G (OtherRental), K (DisabledAccess), L (EmpowermentZone), M (Research), P (CoalGasificationCooperative), Q (AdvancedEnergyCooperative), R (ManufacturingInvestmentCooperative), S (CleanElectricityCooperative), T (EnergyCooperative), U (RehabilitationCooperative), V (AdvancedManufacturingProduction), W (CleanElectricityProduction), X (CleanFuelProduction), Y (CleanHydrogen), Z (OrphanDrug), AA (EnhancedOilRecovery), AB (RenewableElectricity), AC (Biodiesel), AD (NewMarkets), AH (EmployerChildcare), AI (LowSulfurDiesel), AJ (RailroadTrack), AK (MarginalWells), AL (DistilledSpirits), AM (EnergyEfficientHome), AO (RefuelingProperty), AV (DifferentialWage), AW (CarbonOxide), AY (CleanVehicle), AZ (CommercialCleanVehicle), BC (Transferred), ZZ (Other). Codes F, G, AB, BC and ZZ use their code payload with the Form 3800 line the attached statement names. For M, select Research with an empty payload; never import its line. The person answers eligibility for line 4i or 1c from their own records. For AB, the statement must establish line 1f (RenewableElectricity) or line 4e (RenewableElectricitySpecified). Do not infer a missing or conflicting statement line. Code I is expired and must be reported as an error for 2026. Codes H and AP through AU belong in the Credits unentered category; code E investment property and credit recapture keep their separate fields. `investment_property` lists each box 13 code E (RehabilitationRental), box 17 code C (RehabilitationOther), box 17 code D (Energy) and box 17 code AV (CleanElectricity) item from the K-1 and its statement: the expenditures or the basis amount, for the rehabilitation items the year the statement says the building was placed in service (`placed_year`), and for Energy and CleanElectricity the statement's Form 3468 Part I answers as `rate`: `increased_rate` true when line 7a or 8b is checked, `domestic_content` and `energy_community` true when line 9a/9b or 10a/10b is checked, and for CleanElectricity `storage` true for energy storage technology. Report an error when the statement does not show the placed-in-service year of a rehabilitation item. Never read `investment_property.own_form`: the completed credit and reduction answer come from the owner's own Form 3468, not the K-1. Never read `investment_confirmed` or `housing_confirmed`: they come from the owner's own records. `unentered` lists each category at most once: RentalDisposition when the statement reports a rental activity's property disposed of by installment sale, like-kind exchange, casualty or theft, or to a related person; OtherIncome for box 10 codes B, D, E, F, G, I, M, N, O and ZZ, a code A loss or REMIC residual interest, a code C straddle and code K of a gambling business; Credits for box 13 codes H and AP through AU; CreditRecapture for box 13 code AX; AmtItems for box 15 codes C, D, E and F other than the private activity bond interest; Depletion for box 17 code R; OtherInformation for box 17 codes E through J, L through Q, AP, AS, AT, AU and BA, code K when the statement reports a disposition other than a fully taxable sale, and code ZZ when its statement reports an amount or election for the shareholder's return; CooperativeQbi when the code V statement reports items of a specified agricultural or horticultural cooperative; MultipleActivities when box 18 is checked, or when the box 19 statement lists more than one trade or business, more than one rental real estate activity or more than one other rental activity. The imported `income`, `deductions`, `credits`, `investment_property` and `unentered` lists replace the existing lists. Do not read box 15 codes A and B (they belong in the AMT records). Do not read box 17 codes A, B, U, AA, AB, AC, AJ, AN and AW. Do not read box 14 or box 16 code E: the Schedule K-3 has its own import, and loans, basis, amounts at risk, carryovers (including separate ordinary and section 1231 passive losses), `rentals.disposed` (the rental activities sold in full), participation, share holding period, what happened to the shares, the situations under Something else applies, trading and AMT records come from the shareholder's own records, not this K-1.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K1Scorp {
    /// Entity's reporting tax period
    /// Use this K-1's actual beginning and ending dates. Leave unknown when the source does not establish them.
    /// [NullText]
    /// The entity's tax period is unknown
    /// [SetText]
    /// The entity's actual tax period is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Corporation's name (item B)
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub name: String,
    /// Final K-1
    /// The box at the top of the K-1.
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_k1: bool,
    /// Box 1 ordinary business income (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Rental income (loss) — boxes 2 and 3
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub rentals: K1RentalBoxes,
    /// Box 4 interest income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Box 5a ordinary dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Box 5b qualified dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 6 royalties
    /// Reported on Schedule E line 4. Box 12 code I deductions belong in the deductions list.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Box 16 code D distributions
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub distributions: i64,
    /// Section 199A information (box 17 code V)
    #[libertas_llm_doc_field]
    #[libertas_default(Statement)]
    pub qbi: K1ScorpQbi,
    /// Your retained QBI records and allocation confirmation
    /// From your own loss worksheets and source review. Document import preserves these records when the printed statement changes.
    pub qbi_records: K1ScorpQbiRecords,
    /// How you took part in this business in 2026
    #[libertas_default(Investor)]
    pub participation: Participation,
    /// Other boxes
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub more: K1ScorpOtherBoxes,
    /// Schedule K-3 (box 14)
    /// Box 14 is checked when the corporation attached a Schedule K-3. Without one, the foreign taxes in box 16 code F get no credit here; you can ask the corporation for the schedule.
    #[libertas_default(NotAttached)]
    pub schedule_k3: crate::K3Attached,
    /// Your stock and debt basis (Form 7203) and amount at risk
    /// Your own records; the corporation does not track them.
    #[libertas_foldable(initially_open = true)]
    pub basis: K1ScorpBasis,
    /// Alternative minimum tax records
    #[libertas_default(SameAsRegular)]
    pub amt: ScorpAmtRecords,
    /// What happened to your shares in 2026
    #[libertas_default(Kept)]
    pub disposition: ScorpDisposition,
    /// Something else applies
    /// Choose each that applies; leave empty when none does.
    /// ----
    /// Situation
    #[libertas_unordered]
    // Tax structure: one row per distinct S-corporation special situation in the fixed catalog.
    #[libertas_size(max = 3)]
    pub special: Vec<crate::ScorpSpecial>,
    /// Individual QBI aggregation group
    /// Enter the same group name on each actual business in your individual election. Leave empty when no group is selected. This is separate from any aggregate reported by the entity. Save a new source first, then reopen it before selecting a group.
    #[libertas_default("")]
    pub qbi_group: alloc::string::String,
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        income: i64,
        /// W-2 wages
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        wages: i64,
        /// Unadjusted basis of qualified property (UBIA)
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        property: i64,
        /// Specified service trade or business (SSTB)
        #[libertas_llm_doc_field]
        #[libertas_default(false)]
        service: bool,
        /// QBI portion of this year's ordinary business loss
        /// From this section 199A statement, as a positive amount; zero when box 1 is not a loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        reit_dividends: i64,
        /// Qualified PTP items
        /// The statement's qualified publicly traded partnership income or loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub qualified_1250: Option<i64>,
    /// QBI portion of this year's net section 1231 loss
    /// From the section 199A statement, as a positive amount, before this year's shareholder SSTB income limit. Zero for a net gain. Do not include prior suspended losses.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub current: i64,
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpOtherBoxes {
    /// Box 7 net short-term capital gain (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub short: i64,
    /// Box 8a net long-term capital gain (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub long: i64,
    /// Box 8b collectibles (28%) gain (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Box 8c part from selling a partnership interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Box 8c part from estates, trusts, mutual funds or REITs
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Box 8c part from the corporation's business property
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_business: i64,
    /// Box 9 net section 1231 gain (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub section1231: i64,
    /// Section 179 property the entity sold
    /// Each property on the fully taxable sale statement; use this tax system's depreciation. Do not include installment sales, exchanges, casualties or theft.
    /// ----
    /// Property
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Box 11 section 179 deduction
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// Your 2025 Form 4562 line 13 carryover of this corporation's section 179 amounts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover_sold: i64,
    /// Box 12 code A cash contributions (60%)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Other income — box 10
    /// Enter each code once, as printed. Codes not listed here belong among the other boxes below.
    /// ----
    /// Box 10 amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    // Tax structure: one row per distinct Form 1120-S income code in the fixed catalog.
    #[libertas_size(max = 7)]
    pub income: Vec<crate::ScorpIncome>,
    /// Other deductions — box 12 codes other than A
    /// Enter each code once, as printed, including code I royalty deductions.
    /// ----
    /// Box 12 amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    // Tax structure: one row per distinct Form 1120-S deduction code in the fixed catalog.
    #[libertas_size(max = 20)]
    pub deductions: Vec<crate::ScorpDeduction>,
    /// Box 13 code O backup withholding
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Box 13 general business credits
    /// Enter the printed code, its amount and any Form 3800 line its statement requires. Codes H and AP through AU belong among the other boxes below. Code I has no 2026 amount.
    /// ----
    /// Credit
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub credits: Vec<crate::K1CreditPage>,
    /// Investment credit property — box 13 code E and box 17 codes C, D and AV
    /// Enter each item from the K-1 and its statement. Choose automatic or enter your own completed Form3468 credit on that property row.
    /// ----
    /// Property
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    #[libertas_calculation_request("K1InvestmentOptions")]
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Available automatic investment-credit confirmation question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_k1_investment_choices")]
    #[libertas_output_only]
    pub allowed_investment_confirmations: Vec<i32>,
    /// Investment credit property confirmed
    /// Needed only for properties using the automatic credit calculation above. No part of your share is financed by nonqualified nonrecourse financing (section 49) at your level; the statement shows no recapture, no low-income communities allocation, tax-exempt bond or subsidized financing reduction, elective payment or transfer; for rehabilitation, the building's certification is approved or timely pending. No leaves the automatic credits out; your own completed Form3468 amounts use their separate reduction answer.
    /// [NullText]
    /// No property uses the automatic credit calculation
    /// [SetText]
    /// The investment credit property facts are confirmed as
    #[libertas_constrained_by("^.allowed_investment_confirmations")]
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    /// Low-income housing interest acquired after 1989
    /// Needed only when a low-income housing credit (code C or D) is entered above. Yes: you acquired your interest after 1989, so the credit is calculated with the Form 8582-CR post-1989 rules. No: you acquired it before 1990, which uses its own phase-out (Form 8582-CR Part III). Your records of when you acquired the interest show it.
    /// [NullText]
    /// No low-income housing credit is entered
    /// [SetText]
    /// I acquired the low-income housing interest after 1989, confirmed as
    pub housing_confirmed: Option<bool>,
    /// Box 15 code F private activity bond interest (included in box 16 code A)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Box 16 code A tax-exempt interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Box 16 code B other tax-exempt income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Box 16 code C nondeductible expenses
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub nondeductible: i64,
    /// Box 16 code F foreign taxes paid or accrued
    /// Your share of the corporation's foreign taxes. It reduces your stock basis. Any credit for these taxes is figured from the Schedule K-3, not from this code.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
    /// How long you have held your shares
    /// Needed when box 16 code D is above your stock basis or you sold your shares: the gain is long-term when held more than one year. If you bought shares within the year before, choose One year or less.
    #[libertas_default(NotNeeded)]
    pub stock_term: K1StockTerm,
    /// The corporation trades stocks, securities or commodities for its own account
    #[libertas_default(false)]
    pub trading: bool,
    /// Other boxes with an amount
    /// ----
    /// Box with an amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub unentered: Vec<crate::ScorpK1Item>,
    /// AMT section 179 income-limit carryover history
    /// From your own prior AMT Form 4562 records, independently of the regular carryover. Leave absent when unavailable; enter actual zero amounts when the AMT carryover is zero.
    /// [NullText]
    /// AMT carryover history is unavailable
    /// [SetText]
    /// Actual AMT carryover history, including an actual zero
    #[libertas_default($null)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub additions: i64,
    /// Loans you made to the corporation (Form 7203 Part II)
    /// Their repayments in 2026 are box 16 code E.
    /// ----
    /// Loan
    #[libertas_unordered]
    pub loans: Vec<crate::ScorpLoan>,
    /// Amounts not at risk
    #[libertas_foldable(initially_open = false)]
    pub more_at_risk: K1ScorpAtRisk,
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
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        opening: i64,
        /// Your at-risk history
        /// Needed only when this K-1 has a loss, a loss carried because of the at-risk limits, a box 11 section 179 deduction or earlier losses to recapture; otherwise leave the history missing.
        #[libertas_default(Unknown)]
        #[libertas_constrained_by("$.allowed_continuing_risk")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        #[libertas_foldable(initially_open = false)]
        carried: K1ScorpCarried,
    },
}

/// Losses carried from 2025
/// Leave every amount at zero when nothing was carried from 2025.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpCarried {
    /// Ordinary loss carried from 2025 because of your basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_loss: i64,
    /// Nondeductible expenses carried from 2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_nondeductible: i64,
    /// The election to deduct losses before nondeductible expenses applies
    #[libertas_default(false)]
    pub losses_first: bool,
    /// Prior short-term loss suspended by shareholder basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_short: i64,
    /// Prior total long-term loss suspended by shareholder basis
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_long: i64,
    /// Collectible portion of prior basis-suspended long-term loss
    /// Included in the preceding total, not an additional loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_collectibles: i64,
    /// Prior short-term loss suspended only by at-risk limits
    /// Already passed basis; do not include it in basis-suspended losses.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_short: i64,
    /// Prior total long-term loss suspended only by at-risk limits
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_long: i64,
    /// Collectible portion of prior at-risk-suspended long-term loss
    /// Included in the preceding total and already passed basis limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_collectibles: i64,
    /// Prior section 1231 loss suspended by shareholder basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section1231_basis_loss: i64,
    /// Prior section 1231 loss suspended only by at-risk limits
    /// Already passed basis limits. Exclude basis-suspended and passive-suspended amounts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section1231_risk_loss: i64,
    /// Prior section 1231 loss suspended only by passive-activity limits
    /// Already passed basis and at-risk limits; not part of either preceding balance or the ordinary passive loss below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section1231_passive_loss: i64,
    /// Prior ordinary loss suspended by at-risk limits
    /// Exclude basis-suspended and passive-suspended losses; this amount already passed basis limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_loss: i64,
    /// Prior nondeductible expenses suspended only by at-risk limits
    /// From the retained at-risk worksheet. These already reduced stock and debt basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_nondeductible: i64,
    /// Losses deducted in previous tax years beginning after 1978
    /// Retained cumulative recapture history for this activity; do not include the current year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// At-risk recapture income included in previous years
    /// Used only to cap further recapture; do not enter a current-year calculated amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub recaptured: i64,
    /// Ordinary loss carried from 2025 because of the passive activity rules
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub passive_loss: i64,
    /// Rental real estate loss carried from 2025 because of the passive activity rules
    /// The box 2 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_passive_loss: i64,
    /// Other rental loss carried from 2025 because of the passive activity rules
    /// The box 3 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_passive_loss: i64,
    /// Section 1231 loss of the rental real estate activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 2 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_section1231_passive_loss: i64,
    /// Section 1231 loss of the other rental activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 3 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_section1231_passive_loss: i64,

    /// Rental real estate loss carried from 2025 because of your basis
    /// The box 2 loss on your 2025 Form 7203 Part III line 36, column (e).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_basis_loss: i64,
    /// Rental real estate loss carried from 2025 because of the at-risk limits
    /// The box 2 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_risk_loss: i64,
    /// Other rental loss carried from 2025 because of your basis
    /// The box 3 loss on your 2025 Form 7203 Part III line 37, column (e).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_basis_loss: i64,
    /// Other rental loss carried from 2025 because of the at-risk limits
    /// The box 3 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_risk_loss: i64,
    /// Section 179 deduction carried from 2025 because of your basis
    /// Form 7203 Part III line 41, column (e). Not your Form 4562 line 13 carryover, which is entered with box 11.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_section179: i64,
    /// Cash contributions carried from 2025 because of your basis
    /// Form 7203 Part III line 42, column (e). Not a Schedule A carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_charity: i64,
    /// Foreign taxes carried from 2025 because of your basis
    /// Form 7203 Part III line 46, column (e).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub excluded_funding: i64,
    /// Previously at-risk investment newly protected during 2026
    /// Only amounts already included in the opening investment. Exclude casualty and tort insurance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub protected: i64,
    /// Previously protected investment becoming at risk during 2026
    /// Only previously excluded investment; do not repeat current contributions or loan advances.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub released: i64,
    /// Withdrawals and principal repayments returning amounts already excluded from at-risk investment
    /// Included in the distributions and loan repayments entered above.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub excluded_returns: i64,
}

fn k1_sale_issue(
    sales: &[crate::K1Section179Sale],
    carryover: i64,
    sold: i64,
    alternative: bool,
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
            crate::K1Section179Sale::carryover_issue(sales, carryover, sold).map(|issue| {
                issue.map_field_prefix(&["sold"], &["section179_carryover_sold"]).in_field("more")
            })
        })
}

fn k1_source_admission_issue(
    history: Option<&crate::Section179IncomeCarryHistory>,
    period: Option<&crate::K1EntityTaxPeriod>,
    sales: &[crate::K1Section179Sale],
    same_as_regular: bool,
    separate: bool,
    carryover: i64,
    sold: i64,
) -> Option<ValidationIssue> {
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
    if same_as_regular
        && sales.iter().any(|sale| {
            sale.alternative.reported_amounts().is_some()
                && !sale.alternative.matches_known_regular(&sale.regular)
        })
    {
        return Some(ValidationIssue::new("K1_SALE_AMT_CHOICE").at_field("amt"));
    }
    if separate
        && let Ok(sales) = crate::K1Section179Sale::views(sales, true)
        && let Some(issue) = k1_sale_issue(&sales, carryover, sold, true)
    {
        return Some(issue);
    }
    if let Some(issue) = k1_sale_issue(sales, carryover, sold, false) {
        return Some(issue);
    }
    None
}

impl K1Scorp {
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the server twin of the schema rules).
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = k1_source_admission_issue(
            self.more.amt_section179_income_carry.as_ref(),
            self.reporting_period.as_ref(),
            &self.more.section179_sales,
            matches!(self.amt, ScorpAmtRecords::SameAsRegular),
            matches!(self.amt, ScorpAmtRecords::Separate { .. }),
            self.more.section179_carryover,
            self.more.section179_carryover_sold,
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
            && let Some(issue) = k3.issue()
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
            if let Some(issue) = crate::K1Credit::issue_list(
                &credits.iter().cloned().map(Into::into).collect::<Vec<_>>(),
            ) {
                return Some(issue.in_field("more"));
            }
            if investment_property.iter().any(|p| p.own_form().is_none())
                && investment_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_INVESTMENT_NEEDS_CONFIRMATION")
                        .at_field("investment_confirmed")
                        .in_field("more"),
                );
            }
            if credits.iter().any(|credit| credit.code.housing()) && housing_confirmed.is_none() {
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
            if self.final_k1 && self.disposition == ScorpDisposition::Kept {
                return Some(
                    ValidationIssue::new("K1_FINAL_NEEDS_DISPOSITION").at_field("disposition"),
                );
            }
            if matches!(self.disposition, ScorpDisposition::Sold { .. })
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
            ) {
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
            if let Some(issue) = record.property_qbi.records_issue(&record) {
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
            complete,
        };
        let mut passive = crate::ScorpPassiveHistory { loss: 0, complete };
        let (rentals, rental_facts) = self.rentals.record([0, 0], [0, 0]);
        let mut r = crate::ScorpK1 {
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
            opening: 0,
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
            participation: Some(self.participation.record()),
            passive: passive.clone(),
            loss_qbi: crate::ScorpLossQbi {
                current: 0,
                history: self.qbi_records.ordinary.history.clone(),
                at_risk: self.qbi_records.ordinary.at_risk.clone(),
                passive: self.qbi_records.ordinary.passive.clone(),
            },
            property_qbi: crate::ScorpPropertyQbi {
                losses: crate::ScorpLossQbi {
                    current: 0,
                    history: self.qbi_records.property.history.clone(),
                    at_risk: self.qbi_records.property.at_risk.clone(),
                    passive: self.qbi_records.property.passive.clone(),
                },
                complete: self.qbi_records.property_complete,
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
                    complete: true,
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
            disposition: self.disposition.record(),
            special: self.special.clone(),
        };
        if let K1ScorpQbi::Statement { current_loss, property_qbi, .. } = &self.qbi {
            r.loss_qbi.current = *current_loss;
            r.property_qbi =
                property_qbi.record(&self.qbi_records.property, self.qbi_records.property_complete);
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
            r.housing_confirmed = *housing_confirmed;
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
        if capital_items.iter().any(|v| *v != 0) {
            r.capital = Some(capital);
        }
        if section1231.has_items() {
            r.section1231 = Some(section1231);
        }
        r.risk = risk;
        r.passive = passive;
        if matches!(self.amt, ScorpAmtRecords::SameAsRegular) {
            for sale in &mut r.section179_sales {
                sale.apply_parent_same_as_regular();
            }
        }
        r.amt = Some(self.amt.record(&r));
        r
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written.
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
        let mut page = Self {
            qbi_group: r.qbi_aggregation_group.clone().unwrap_or_default(),
            name: name.into(),
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
            qbi_records: K1ScorpQbiRecords {
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
            },
            participation: Participation::from_record(r.participation.as_ref()?),
            more: K1ScorpOtherBoxes {
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
                investment_confirmed: if investment_choices(&r.investment_property).is_empty() {
                    None
                } else {
                    r.investment_confirmed.map(K1InvestmentConfirmation::from)
                },
                housing_confirmed: r.housing_confirmed,
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
            amt: ScorpAmtRecords::from_record(&regular, r.amt.as_ref()?),
            disposition: ScorpDisposition::from_record(&r.disposition)?,
            special: r.special.clone(),
        };
        if r.section179_sales
            .iter()
            .any(|sale| !sale.alternative.matches_known_regular(&sale.regular))
        {
            page.amt = ScorpAmtRecords::Separate { record: r.amt.clone()? };
        } else if matches!(page.amt, ScorpAmtRecords::SameAsRegular) {
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
        let mut written = page.record();
        written.health_insurance = r.health_insurance.clone();
        (written == *r).then_some(page)
    }
}

// The Schedule K-1 (Form 1065) kind (design A3a, unit T2-29p).

/// Schedule K-1 (Form 1065)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this Schedule K-1 (Form 1065) and the partnership's attached statements exactly. It belongs to the supplied year only when the entity's tax year ends in that year (a calendar-year K-1 for that year, or a fiscal year ending in it); report a mismatch as an error. Read `reporting_period` only from this K-1 entity’s actual current beginning and ending dates; return null when unavailable, and never borrow another source’s period. Report illegible or conflicting values as errors, and never infer a fact the K-1 or its statements do not show. Parentheses or a minus sign mark a negative amount; return amounts with their printed sign. `name` is item B, the partnership's name as printed; `publicly_traded` is true only when item D (publicly traded partnership) is checked; `final_k1` is true only when the Final K-1 box is checked. `liabilities_start` and `liabilities_end` are item K1's beginning and ending shares of liabilities, each the sum of the nonrecourse, qualified nonrecourse and recourse amounts. Box 1 is `ordinary`; return an empty string when box 1 is clearly blank. `rentals` holds box 2 as `real_estate` and box 3 as `other`; return zero for a clearly blank box. `rentals.real_estate_qbi` holds the section 199A statement's W-2 wages (`wages`) and unadjusted basis of qualified property (`property`) for the box 2 rental real estate activity when the statement lists it as a trade or business, and is null when the statement does not list it or no statement is attached; `rentals.other_qbi` holds the same for the box 3 activity. Never read `rentals.participation`: it comes from the partner's own records. Box 4a is `services`, box 4b `capital`, and `distributions` is the sum of box 19 codes A and F. Box 14 code A is `se`; return an empty string when it is not shown. `se_gross` is Farming with the box 14 code B amount, Nonfarm with the code C amount, or None when neither is shown; report an error when both are shown. Return zero for a clearly blank amount. Box 20 code Z is `qbi`: Statement when the section 199A statement is attached, with the qualified business income or loss of the box 1 trade or business as `income`, its W-2 wages as `wages` and its unadjusted basis of qualified property as `property` (0 for an amount the statement shows as zero or blank), `service` true only when the statement marks a specified service trade or business, `section1231_qbi` its qualified section 1231 business gain or loss (signed, 0 when not listed), `reit_dividends` the statement's section 199A dividends and `ptp_income` its qualified PTP items (signed; 0 when not shown); NoStatement when the K-1 shows no code Z; report an error when code Z refers to a statement that is not attached. `more.section179_sales` lists each property on the box 20 code L statement sold in a fully taxable sale: `description`, date acquired and placed in service as `acquired`, date sold as `sold`, and your shares in the `regular` monetary record: gross sales price (`price`), cost or other basis plus selling expenses (`cost`), depreciation excluding section 179 (`depreciation`), and section 179 deduction passed through (`section179`). Never read `alternative` or `id`: AMT source facts and accepted identity are not imported. Return an empty list when the statement shows no such sale. The imported list replaces the existing sale statements. Never read `more.amt_section179_income_carry`: it comes from the person’s own prior AMT Form 4562 history and must be preserved independently of the regular amount. Never read `more.section179_carryover_sold`: it comes from the person's own Form 4562 carryover records and must be preserved. `more` holds the other boxes: box 5 is `interest`, box 6a `dividends`, box 6b `qualified_dividends`, box 7 `royalties`, box 8 `short`, box 9a `long`, box 9b `collectibles`; box 9c is split by the statement: `unrecaptured_partnership` for the part from selling an interest in another partnership, `unrecaptured_fund` for the part from estates, trusts, mutual funds or REITs, `rentals.real_estate_unrecaptured` and `rentals.other_unrecaptured` for the parts from the box 2 and box 3 activities, and `unrecaptured_business` for the rest from the box 1 business property. Box 10 is split the same way, signed: `rentals.real_estate_section1231` and `rentals.other_section1231` for the parts the statement assigns to the box 2 and box 3 activities and `section1231` for the rest (all of box 10 when the statement assigns none to a rental activity), before the owner's loss limits. Do not repeat rental amounts in business fields; if a statement does not determine which activity owns a property amount, report an error rather than guessing. Box 12 is `section179`, box 13 code A `cash_charity`, code M `medical_insurance`, code R `pensions`, box 15 code O `backup_withholding`, box 18 code A `exempt_interest`, code B `exempt_other`, code C `nondeductible` and box 21 `foreign_taxes`; box 20 code AI is `deferred_compensation_tax` (the interest and the additional tax together), code AQ `crp_payments`, code AB `sale_ordinary` (signed), code AC `sale_collectibles` and code AD `sale_unrecaptured`; return zero for a clearly blank amount. `income` lists each box 11 code at most once with its signed amount: code A as PortfolioIncome only when it is income and the statement reports no REMIC residual interest, C as Section1256 unless the statement reports a straddle, E as DebtCancellation, F as BasisAdjustmentIncome only when the statement attributes it to the box 1 trade or business, J as TaxBenefitRecovery, K as GamblingWinnings and GamblingLosses (the losses as a positive amount) unless the statement shows a gambling business, Q as PreferredStockShort or PreferredStockLong and S as NonportfolioShort or NonportfolioLong by the term the statement shows, and R as SpeciallyAllocatedOrdinary. `deductions` lists each box 13 code other than A, M, O and R at most once with its amount, by the member named for its code (B CashThirty, C NoncashFifty, D NoncashThirty, E CapitalGainPropertyThirty, F CapitalGainPropertyTwenty, G ContributionsHundred, H InvestmentInterest, I RoyaltyDeductions, J Section59e, K ExcessBusinessInterest, L PortfolioOther, N EducationalAssistance, P Preproductive, S Reforestation, V BasisAdjustmentDeduction only when the statement attributes it to the box 1 trade or business, W SoilWater, X Production, Y Barriers, Z Itemized, AA ConstructionFund, AB EarlyWithdrawalPenalty, AC DistributionInterest, AD WorkingInterest, AE PortfolioDeductions, ZZ Other). `credits` lists each supported box 15 code once per code and reported Form 3800 line with its amount: J (WorkOpportunity), N (EmployerTipFica), AE (PensionStartup), AF (AutoEnrollment), AG (MilitarySpouse), BA (SmallEmployerHealth), BB (PaidLeave), C (LowIncomeHousingJ5), D (LowIncomeHousingOther), A (ZeroEmissionNuclear), B (AdvancedNuclear), F (OtherRentalRealEstate), G (OtherRental), K (DisabledAccess), L (EmpowermentZone), M (Research), P (CoalGasificationCooperative), Q (AdvancedEnergyCooperative), R (ManufacturingInvestmentCooperative), S (CleanElectricityCooperative), T (EnergyCooperative), U (RehabilitationCooperative), V (AdvancedManufacturingProduction), W (CleanElectricityProduction), X (CleanFuelProduction), Y (CleanHydrogen), Z (OrphanDrug), AA (EnhancedOilRecovery), AB (RenewableElectricity), AC (Biodiesel), AD (NewMarkets), AH (EmployerChildcare), AI (LowSulfurDiesel), AJ (RailroadTrack), AK (MarginalWells), AL (DistilledSpirits), AM (EnergyEfficientHome), AO (RefuelingProperty), AV (DifferentialWage), AW (CarbonOxide), AY (CleanVehicle), AZ (CommercialCleanVehicle), BC (Transferred), ZZ (Other). Codes F, G, AB, BC and ZZ use their code payload with the Form 3800 line the attached statement names. For M, select Research with an empty payload; never import its line. The person answers eligibility for line 4i or 1c from their own records. For AB, the statement must establish line 1f (RenewableElectricity) or line 4e (RenewableElectricitySpecified). Do not infer a missing or conflicting statement line. Code I is expired and must be reported as an error for 2026. Codes H and AP through AU belong in the Credits unentered category; code E investment property and credit recapture keep their separate fields. `investment_property` lists each box 15 code E (RehabilitationRental), box 20 code D (RehabilitationOther), box 20 code E (Energy) and box 20 code AV (CleanElectricity) item from the K-1 and its statement: the expenditures or the basis amount, for the rehabilitation items the year the statement says the building was placed in service (`placed_year`), and for Energy and CleanElectricity the statement's Form 3468 Part I answers as `rate`: `increased_rate` true when line 7a or 8b is checked, `domestic_content` and `energy_community` true when line 9a/9b or 10a/10b is checked, and for CleanElectricity `storage` true for energy storage technology. Report an error when the statement does not show the placed-in-service year of a rehabilitation item. Never read `investment_property.own_form`: the completed credit and reduction answer come from the owner's own Form 3468, not the K-1. Never read `investment_confirmed` or `housing_confirmed`: they come from the owner's own records. `unentered` lists each category at most once: RentalDisposition when the statement reports a rental activity's property disposed of by installment sale, like-kind exchange, casualty or theft, or to a related person; OtherIncome for box 11 codes B, D, I, L, M, N, O, P and ZZ, a code A loss or REMIC residual interest, a code C straddle, a code F adjustment to another item and code K of a gambling business; OtherDeductions for box 13 code O and a code V adjustment to another item; Credits for box 15 codes H and AP through AU; CreditRecapture for box 15 code AX; AmtItems for box 17 codes C, D, E and F; PropertyDistribution for box 19 code B, C or G; Depletion for box 20 code T; OtherInformation for box 20 codes F through K, M, O through S, W, AM, AP, AS, AT and AU, code L when the statement reports a disposition other than a fully taxable sale, code AL when the statement shows gain you recognize as the U.S. transferor, and code ZZ when its statement reports an amount or election for the partner's return; CooperativeQbi when the code Z statement reports items of a specified agricultural or horticultural cooperative; MultipleActivities when box 22 is checked, or when the box 23 statement lists more than one trade or business, more than one rental real estate activity or more than one other rental activity. The imported `income`, `deductions`, `credits`, `investment_property` and `unentered` lists replace the existing lists. Do not read box 6c (dividend equivalents are not reported on Form 1040) or box 17 codes A and B (they belong in the AMT records). Do not read box 19 code D or box 20 codes A, B, N, U, V, X, Y, AA, AE through AH, AJ, AK, AN, AO, AR, AW, AX, AY and AZ: the App figures them from the other boxes, or they do not change an individual's return. Do not read item G, item L, box 16 or box 20 code C: the Schedule K-3 has its own import, and participation, basis, amounts at risk, carryovers (including separate ordinary and section 1231 passive losses), `rentals.disposed` (the rental activities sold in full), income source, holding period, what happened to the interest, the situations under Something else applies, trading and AMT records come from the partner's own records, not this K-1.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K1Partnership {
    /// Entity's reporting tax period
    /// Use this K-1's actual beginning and ending dates. Leave unknown when the source does not establish them.
    /// [NullText]
    /// The entity's tax period is unknown
    /// [SetText]
    /// The entity's actual tax period is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Partnership's name (item B)
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub name: String,
    /// Publicly traded partnership (item D)
    /// Its section 199A statement reports current-year qualified income or loss and the actual specified-service classification, with no wages or unadjusted property basis. Current specified-service income limits apply before eligible amounts join the separate REIT/PTP component. Historical qualified-loss statements and suspended basis/at-risk/passive balances still need their source worksheet.
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub publicly_traded: bool,
    /// Final K-1
    /// The box at the top of the K-1.
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub final_k1: bool,
    /// Your interest (item G)
    #[libertas_default(General)]
    pub interest: PartnerInterest,
    /// Your share of liabilities at the beginning of the year (item K1)
    /// Add the nonrecourse, qualified nonrecourse and recourse amounts.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub liabilities_start: i64,
    /// Your share of liabilities at the end of the year (item K1)
    /// The same three amounts.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub liabilities_end: i64,
    /// Box 1 ordinary business income (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Rental income (loss) — boxes 2 and 3
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub rentals: K1RentalBoxes,
    /// Box 4a guaranteed payments for services
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub services: i64,
    /// Box 4b guaranteed payments for capital
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub capital: i64,
    /// Box 14 code A net earnings (loss) from self-employment
    /// Enter the amount shown, even zero.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub se: i64,
    /// Box 14 code B or C gross income
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub se_gross: SeGross,
    /// Box 19 codes A and F cash distributions
    /// Cash only, including cash for services (code F); a distribution of marketable securities belongs under Something else applies.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub distributions: i64,
    /// Section 199A information (box 20 code Z)
    #[libertas_llm_doc_field]
    #[libertas_default(Statement)]
    pub qbi: PartnerQbi,
    /// Your retained QBI records and gross-income allocation
    /// From your own worksheets. Document import preserves these records when the printed statement changes.
    pub qbi_records: crate::partnership_qbi::PartnerQbiRecords,
    /// Other boxes
    #[libertas_llm_doc_field]
    #[libertas_foldable(initially_open = false)]
    pub more: K1PartnershipOtherBoxes,
    /// Schedule K-3 (box 16)
    /// Box 16 is checked when the partnership attached a Schedule K-3. Without one, the foreign taxes in box 21 get no credit here; you can ask the partnership for the schedule.
    #[libertas_default(NotAttached)]
    pub schedule_k3: crate::K3Attached,
    /// Your basis and amount at risk
    /// Your own records, not item L's capital account.
    #[libertas_foldable(initially_open = true)]
    pub basis: PartnerBasis,
    /// What produces this income
    /// Asked only when the tax on a child's unearned income (Form 8615) or the standard deduction for a filer someone can claim as a dependent can apply to this return.
    #[libertas_constrained_by("$.allowed_partner_source")]
    pub income_source: K1IncomeSource,
    /// Alternative minimum tax records
    #[libertas_default(SameAsRegular)]
    pub amt: PartnerAmtRecords,
    /// What happened to your interest in 2026
    #[libertas_default(Kept)]
    pub disposition: PartnerDisposition,
    /// Something else applies
    /// Choose each that applies; leave empty when none does.
    /// ----
    /// Situation
    #[libertas_unordered]
    // Tax structure: one row per distinct partnership special situation in the fixed catalog.
    #[libertas_size(max = 5)]
    pub special: Vec<crate::PartnerSpecial>,
    /// Individual QBI aggregation group
    /// Enter the same group name on each actual business in your individual election. Leave empty when no group is selected. This is separate from any aggregate reported by the entity. Save a new source first, then reopen it before selecting a group.
    #[libertas_default("")]
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
        #[libertas_number(min = 0)]
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

/// Your interest (item G)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
#[allow(clippy::large_enum_variant)]
pub enum PartnerInterest {
    /// General partner or LLC member-manager
    General {
        /// How you took part in this business in 2026
        #[libertas_default(Investor)]
        participation: Participation,
    },
    /// Limited partner
    LimitedPartner {
        /// How you took part in this business in 2026
        #[libertas_default(Investor)]
        participation: LimitedParticipation,
    },
    /// Other LLC member
    LlcMember {
        /// Box 14 code A includes your share of box 1
        #[libertas_default(false)]
        box1_in_se: bool,
        /// How you took part in this business in 2026
        #[libertas_default(Investor)]
        participation: Participation,
    },
}

/// Box 14 code B or C gross income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SeGross {
    /// Neither code B nor code C
    None,
    /// Code B gross farming or fishing income
    Farming {
        /// Gross farming or fishing income
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Code C gross nonfarm income
    Nonfarm {
        /// Gross nonfarm income
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
        income: i64,
        /// W-2 wages
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        wages: i64,
        /// Unadjusted basis of qualified property (UBIA)
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
        #[libertas_default(0)]
        section1231_qbi: i64,
        /// Section 199A dividends
        /// From the statement; they are already part of box 6a.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        reit_dividends: i64,
        /// Qualified PTP items
        /// The statement's qualified publicly traded partnership income or loss.
        #[libertas_llm_doc_field]
        #[libertas_money("USD", 2)]
        #[libertas_default(0)]
        ptp_income: i64,
    },
}

/// Other boxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1PartnershipOtherBoxes {
    /// Box 5 interest income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Box 6a ordinary dividends
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Box 6b qualified dividends
    /// The part of box 6a taxed at capital-gain rates.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 7 royalties
    /// Reported on Schedule E line 4. Box 13 code I deductions belong in the deductions list.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Box 8 net short-term capital gain (loss)
    /// Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub short: i64,
    /// Box 9a net long-term capital gain (loss)
    /// Enter a loss as a negative amount. Include the collectibles and unrecaptured section 1250 amounts below; they are parts of this box, not additional gains.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub long: i64,
    /// Box 9b collectibles (28%) gain (loss)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Box 9c part from selling an interest in another partnership
    /// The part of box 9c the partnership's statement shows as coming from the sale of an interest in another partnership.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Box 9c part from estates, trusts, mutual funds or REITs
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Box 9c unrecaptured section 1250 gain from business property
    /// Only the box 1 business's part. Keep rental property with its rental activity. This is part of the property gain, not additional income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_business: i64,
    /// Box 10 section 1231 gain (loss) from the box 1 business
    /// The business part assigned by the statement, before your loss limits. Enter a loss as negative. Do not include rental property or repeat the amount in box 1.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub section1231: i64,
    /// Section 179 property the entity sold
    /// Each property on the fully taxable sale statement; use this tax system's depreciation. Do not include installment sales, exchanges, casualties or theft.
    /// ----
    /// Property
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Box 12 section 179 deduction
    /// Your share of the partnership's section 179 expense. It joins your own section 179 under your dollar and business-income limits, reduces your basis in full this year even when part is carried to 2027, and reduces your box 14 code A earnings as a general partner.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// The unused part of this partnership's earlier section 179 amounts carried from your 2025 return because of the business income limit. It already reduced your basis in the partnership.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover_sold: i64,
    /// Box 13 code A cash contributions (60%)
    /// The partnership's cash gifts to charities, deducted on Schedule A like your own cash gifts.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Box 13 code M amounts paid for medical insurance
    /// The partnership also included these premiums in box 4a. The self-employed benefits page asks your coverage.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub medical_insurance: i64,
    /// Box 13 code R pensions and IRAs
    /// The self-employed benefits page asks which plan.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub pensions: i64,
    /// Other income — box 11
    /// Enter each code once, as printed. Codes not listed here belong among the other boxes below.
    /// ----
    /// Box 11 amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    // Tax structure: one row per distinct Form 1065 income code in the fixed catalog.
    #[libertas_size(max = 12)]
    pub income: Vec<crate::PartnerIncome>,
    /// Other deductions — box 13 codes other than A, M, O and R
    /// Enter each code once, as printed, including code I royalty deductions. Code O belongs among the other boxes below.
    /// ----
    /// Box 13 amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    // Tax structure: one row per distinct Form 1065 deduction code in the fixed catalog.
    #[libertas_size(max = 25)]
    pub deductions: Vec<crate::PartnerDeduction>,
    /// Box 15 code O backup withholding
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub backup_withholding: i64,
    /// Box 15 general business credits
    /// Enter the printed code, its amount and any Form 3800 line its statement requires. Codes H and AP through AU belong among the other boxes below. Code I has no 2026 amount.
    /// ----
    /// Credit
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub credits: Vec<crate::K1CreditPage>,
    /// Investment credit property — box 15 code E and box 20 codes D, E and AV
    /// Enter each item from the K-1 and its statement. Choose automatic or enter your own completed Form3468 credit on that property row.
    /// ----
    /// Property
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    #[libertas_calculation_request("K1InvestmentOptions")]
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Available automatic investment-credit confirmation question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_k1_investment_choices")]
    #[libertas_output_only]
    pub allowed_investment_confirmations: Vec<i32>,
    /// Investment credit property confirmed
    /// Needed only for properties using the automatic credit calculation above. No part of your share is financed by nonqualified nonrecourse financing (section 49) at your level; the statement shows no recapture, no low-income communities allocation, tax-exempt bond or subsidized financing reduction, elective payment or transfer; for rehabilitation, the building's certification is approved or timely pending. No leaves the automatic credits out; your own completed Form3468 amounts use their separate reduction answer.
    /// [NullText]
    /// No property uses the automatic credit calculation
    /// [SetText]
    /// The investment credit property facts are confirmed as
    #[libertas_constrained_by("^.allowed_investment_confirmations")]
    pub investment_confirmed: Option<K1InvestmentConfirmation>,
    /// Low-income housing interest acquired after 1989
    /// Needed only when a low-income housing credit (code C or D) is entered above. Yes: you acquired your interest after 1989, so the credit is calculated with the Form 8582-CR post-1989 rules. No: you acquired it before 1990, which uses its own phase-out (Form 8582-CR Part III). Your records of when you acquired the interest show it.
    /// [NullText]
    /// No low-income housing credit is entered
    /// [SetText]
    /// I acquired the low-income housing interest after 1989, confirmed as
    pub housing_confirmed: Option<bool>,
    /// Box 18 code A tax-exempt interest
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Box 18 code B other tax-exempt income
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_other: i64,
    /// Box 18 code C nondeductible expenses
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub nondeductible: i64,
    /// Box 20 code C fuel tax credit information
    /// [NullText]
    /// I am not claiming a box 20 code C fuel credit
    /// [SetText]
    /// The box 20 code C fuel credit information is
    #[libertas_foldable(initially_open = false)]
    #[libertas_default($null)]
    pub fuel_credit: Option<crate::PartnerFuelTaxCredit>,
    /// Fuel credit from your 2025 return
    /// [NullText]
    /// No 2025 fuel credit needs a current income adjustment
    /// [SetText]
    /// The 2025 fuel credit and prior deduction facts are
    #[libertas_default($null)]
    pub prior_fuel_credit: Option<crate::PartnerPriorFuelCredit>,
    /// Box 21 foreign taxes paid or accrued
    /// Your share of the partnership's foreign taxes. It reduces your basis. Any credit for these taxes is figured from the Schedule K-3, not from this box.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
    /// Box 20 code AI interest and additional tax on deferred compensation
    /// The interest and the additional tax together. The income itself is in box 4a or 4b.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deferred_compensation_tax: i64,
    /// Box 20 code AQ conservation reserve program payments
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub crp_payments: i64,
    /// Box 20 code AB section 751 gain (loss)
    /// The part of the gain on selling your interest taxed as ordinary income. Enter a loss as a negative amount.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub sale_ordinary: i64,
    /// Box 20 code AC collectibles gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sale_collectibles: i64,
    /// Box 20 code AD unrecaptured section 1250 gain
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sale_unrecaptured: i64,
    /// How long you have held your interest
    /// Needed when box 19 cash is above your basis or you sold your interest: the gain is long-term when held more than one year. If you contributed to the partnership within the year before, choose One year or less.
    #[libertas_default(NotNeeded)]
    pub term: K1StockTerm,
    /// The partnership trades stocks, securities or commodities for its own account
    #[libertas_default(false)]
    pub trading: bool,
    /// Other boxes with an amount
    /// ----
    /// Box with an amount
    #[libertas_llm_doc_field]
    #[libertas_unordered]
    pub unentered: Vec<crate::PartnerK1Item>,
    /// AMT section 179 income-limit carryover history
    /// From your own prior AMT Form 4562 records, independently of the regular carryover. Leave absent when unavailable; enter actual zero amounts when the AMT carryover is zero.
    /// [NullText]
    /// AMT carryover history is unavailable
    /// [SetText]
    /// Actual AMT carryover history, including an actual zero
    #[libertas_default($null)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub contributions: i64,
    /// Part of the beginning liabilities you are at risk for
    /// The recourse part you are personally liable for, plus your share of qualified nonrecourse financing secured by real property.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub debt_start: i64,
    /// Part of the ending liabilities you are at risk for
    /// The same two parts at the end of the year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub debt_end: i64,
    /// Amounts not at risk
    #[libertas_foldable(initially_open = false)]
    pub more_at_risk: PartnerAtRisk,
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
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        opening: i64,
        /// Your at-risk history
        /// Needed only when this K-1 has a loss, a loss carried because of the at-risk limits, a section 179 deduction or earlier losses to recapture; otherwise leave the history missing.
        #[libertas_default(Unknown)]
        #[libertas_constrained_by("$.allowed_continuing_risk")]
        risk_history: crate::ScorpRiskStart,
        /// Losses carried from 2025
        #[libertas_foldable(initially_open = false)]
        carried: PartnerCarried,
    },
}

/// Losses carried from 2025
/// Leave every amount at zero when nothing was carried from 2025.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerCarried {
    /// Loss carried from 2025 because of your basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_loss: i64,
    /// Nondeductible expenses carried from 2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_nondeductible: i64,
    /// Ordinary loss suspended only by at-risk limits through 2025
    /// Already passed outside basis; do not enter it again as a basis carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_loss: i64,
    /// Nondeductible expenses suspended only by at-risk limits
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_nondeductible: i64,
    /// Cumulative previously deducted losses for recapture
    /// After 1978, excluding this year and losses never deducted.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// At-risk recapture already reported in prior years
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub recaptured: i64,
    /// Loss carried from 2025 because of the passive activity rules
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub passive_loss: i64,
    /// Rental real estate loss carried from 2025 because of the passive activity rules
    /// The box 2 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_passive_loss: i64,
    /// Other rental loss carried from 2025 because of the passive activity rules
    /// The box 3 activity's loss on your 2025 Form 8582.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_passive_loss: i64,
    /// Section 1231 loss of the rental real estate activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 2 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_section1231_passive_loss: i64,
    /// Section 1231 loss of the other rental activity carried from 2025 because of the passive activity rules
    /// The Form 4797 part of the box 3 activity's 2025 Form 8582 loss; leave it out of the ordinary rental carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_section1231_passive_loss: i64,

    /// Short-term capital loss carried from 2025 because of your basis
    /// Box 8 losses your 2025 basis worksheet did not allow (Part II line 15e, column E).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_short: i64,
    /// Long-term capital loss carried from 2025 because of your basis
    /// Box 9a losses your 2025 basis worksheet did not allow (Part II line 15f, column E).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_long: i64,
    /// Short-term capital loss carried from 2025 because of the at-risk limits
    /// Already passed your basis; do not enter it again above.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_short: i64,
    /// Long-term capital loss carried from 2025 because of the at-risk limits
    /// Already passed your basis; do not enter it again above.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_long: i64,
    /// Rental real estate loss carried from 2025 because of your basis
    /// The box 2 loss your 2025 basis worksheet did not allow (Part II line 15b, column E).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_basis_loss: i64,
    /// Rental real estate loss carried from 2025 because of the at-risk limits
    /// The box 2 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub rental_real_estate_risk_loss: i64,
    /// Other rental loss carried from 2025 because of your basis
    /// The box 3 loss your 2025 basis worksheet did not allow (Part II line 15c, column E).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_basis_loss: i64,
    /// Other rental loss carried from 2025 because of the at-risk limits
    /// The box 3 loss your 2025 Form 6198 did not allow. Already passed your basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_rental_risk_loss: i64,
    /// Section 179 deduction carried from 2025 because of your basis
    /// The box 12 amount your 2025 basis worksheet did not allow (Part II line 15i, column E). Not your Form 4562 line 13 carryover, which is entered with box 12.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_section179: i64,
    /// Cash contributions carried from 2025 because of your basis
    /// The box 13 code A gifts your 2025 basis worksheet did not allow (Part II line 15j, column E). Not a Schedule A carryover.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_charity: i64,
    /// Foreign taxes carried from 2025 because of your basis
    /// The box 21 taxes your 2025 basis worksheet did not allow (Part II line 15d, column E).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_foreign_taxes: i64,
    /// Self-employment part of the loss carried because of your basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub se_basis: i64,
    /// Self-employment part of the loss carried because of the at-risk limits
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub se_risk: i64,
    /// Self-employment part of the loss carried because of the passive activity rules
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub se_passive: i64,
    /// Section 1231 business loss carried from 2025 because of your basis
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section1231_basis_loss: i64,
    /// Section 1231 business loss carried from 2025 only because of the at-risk limits
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section1231_risk_loss: i64,
    /// Section 1231 business loss carried from 2025 only because of the passive activity rules
    /// Keep the business-property loss separate from ordinary and rental losses and from the balances of the other limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub excluded_contributions: i64,
    /// Cash distributions returning previously excluded investment
    /// Included in box 19 code A; never count the same funds twice.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub excluded_distributions: i64,
    /// Previously at-risk cash investment newly protected against loss
    /// Exclude casualty or tort insurance and debt changes already entered.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub protected: i64,
    /// Previously excluded cash investment no longer protected
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
) -> Option<crate::PartnerSeparateItems> {
    let items = crate::PartnerSeparateItems {
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
fn separate_items_zero() -> crate::PartnerSeparateItems {
    crate::PartnerSeparateItems {
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
    // The first relationship between this page's fields that fails, by its
    // client rule's key (the server twin of the schema rules), then the
    // constrained continuing-owner at-risk history checked again on save.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
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
            matches!(self.amt, PartnerAmtRecords::SameAsRegular),
            matches!(self.amt, PartnerAmtRecords::Separate { .. }),
            self.more.section179_carryover,
            self.more.section179_carryover_sold,
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
            && let Some(issue) = k3.issue()
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
            if debt > total {
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
            if let Some(issue) = crate::K1Credit::issue_list(
                &credits.iter().cloned().map(Into::into).collect::<Vec<_>>(),
            ) {
                return Some(issue.in_field("more"));
            }
            if investment_property.iter().any(|p| p.own_form().is_none())
                && investment_confirmed.is_none()
            {
                return Some(
                    ValidationIssue::new("K1_INVESTMENT_NEEDS_CONFIRMATION")
                        .at_field("investment_confirmed")
                        .in_field("more"),
                );
            }
            if credits.iter().any(|credit| credit.code.housing()) && housing_confirmed.is_none() {
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
        if matches!(self.interest, PartnerInterest::LimitedPartner { .. })
            && self.rentals.participation.active()
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
        if self.final_k1 && self.disposition == PartnerDisposition::Kept {
            return Some(
                ValidationIssue::new("K1_FINAL_NEEDS_DISPOSITION").at_field("disposition"),
            );
        }
        if (self.more.sale_ordinary != 0
            || self.more.sale_collectibles != 0
            || self.more.sale_unrecaptured != 0)
            && matches!(
                self.disposition,
                PartnerDisposition::Kept | PartnerDisposition::LiquidatedForCash
            )
        {
            return Some(ValidationIssue::new("K1_SALE_CODES_NEED_SALE").at_field("disposition"));
        }
        if matches!(
            self.disposition,
            PartnerDisposition::Sold { .. } | PartnerDisposition::LiquidatedForCash
        ) && self.more.term == K1StockTerm::NotNeeded
        {
            return Some(
                ValidationIssue::new("K1_SALE_NEEDS_HOLDING_PERIOD")
                    .at_field("term")
                    .in_field("more"),
            );
        }
        if self.disposition == PartnerDisposition::LiquidatedForCash && !self.final_k1 {
            return Some(ValidationIssue::new("K1_LIQUIDATION_NEEDS_FINAL").at_field("final_k1"));
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
        // Only a situation that leaves the whole K-1 out of the estimate
        // makes the basis and financing confirmations No.
        let complete = !self.special.iter().any(|situation| situation.whole());
        let (participation, limited_for_se) = match &self.interest {
            PartnerInterest::General { participation } => {
                (crate::PartnerParticipation::General { facts: participation.record() }, false)
            }
            PartnerInterest::LimitedPartner { participation } => (participation.record(), true),
            PartnerInterest::LlcMember { box1_in_se, participation } => (
                crate::PartnerParticipation::General { facts: participation.record() },
                !box1_in_se,
            ),
        };
        let (start, carried) = match &self.basis.start {
            BasisStart::New => (crate::ScorpRiskStart::New, [0; 29]),
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
                [
                    *opening,
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
            opening,
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
            participation,
            trade_kind: crate::ScorpTradeKind::Ordinary,
            limited_for_se,
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
            qbi_records: self.qbi_records.clone(),
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
            disposition: self.disposition.record(),
            sale: crate::PartnerSaleParts::default(),
            special: self.special.clone(),
            publicly_traded: self.publicly_traded,
        };
        (record.income_source, record.gross_personal_services) = self.income_source.record();
        {
            let K1PartnershipOtherBoxes {
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
            record.income = income.clone();
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
            record.housing_confirmed = *housing_confirmed;
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
        if matches!(self.amt, PartnerAmtRecords::SameAsRegular) {
            for sale in &mut record.section179_sales {
                sale.apply_parent_same_as_regular();
            }
        }
        record.alternative = Some(self.amt.record(&record.regular));
        record
    }

    // The page a stored record reopens as; None for a record this page could
    // not have written. The partner's coverage, distribution-paid premiums
    // and plan choice are not on this page, so they are compared as stored. An Other LLC member whose
    // box 14 code A includes box 1 writes the same record as a general
    // partner and reopens as one.
    pub(crate) fn from_record(name: &str, r: &crate::PartnerK1) -> Option<Self> {
        let interest = match (&r.participation, r.limited_for_se) {
            (crate::PartnerParticipation::General { facts }, false) => {
                PartnerInterest::General { participation: Participation::from_record(facts) }
            }
            (crate::PartnerParticipation::General { facts }, true) => PartnerInterest::LlcMember {
                box1_in_se: false,
                participation: Participation::from_record(facts),
            },
            (limited @ crate::PartnerParticipation::Limited { .. }, true) => {
                PartnerInterest::LimitedPartner {
                    participation: LimitedParticipation::from_record(limited)?,
                }
            }
            _ => return None,
        };
        let basis = &r.regular.basis;
        let risk = &r.regular.risk;
        let history = &r.se_history;
        let start = if risk.start == crate::ScorpRiskStart::New {
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
            investment_confirmed: if investment_choices(&r.investment_property).is_empty() {
                None
            } else {
                r.investment_confirmed.map(K1InvestmentConfirmation::from)
            },
            housing_confirmed: r.housing_confirmed,
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
            income: r.income.clone(),
            deductions: r.deductions.clone(),
        };
        let mut page = Self {
            qbi_group: r.qbi_aggregation_group.clone().unwrap_or_default(),
            name: name.into(),
            reporting_period: r.reporting_period,
            publicly_traded: r.publicly_traded,
            final_k1: r.final_k1,
            interest,
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
            qbi_records: r.qbi_records.clone(),
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
            amt: PartnerAmtRecords::from_record(&r.regular, r.alternative.as_ref()?),
            disposition: PartnerDisposition::from_record(&r.disposition),
            special: r.special.clone(),
        };
        if r.section179_sales
            .iter()
            .any(|sale| !sale.alternative.matches_known_regular(&sale.regular))
        {
            page.amt = PartnerAmtRecords::Separate { record: r.alternative.clone()? };
        } else if matches!(page.amt, PartnerAmtRecords::SameAsRegular) {
            for sale in &mut page.more.section179_sales {
                sale.alternative = crate::K1Section179SaleAmtSource::NotReported;
            }
        }
        let same = |page: &Self| {
            let mut written = page.record();
            written.health_insurance = r.health_insurance.clone();
            written.health_distributions = r.health_distributions;
            written.retirement = r.retirement.clone();
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
    fn confirmed(self) -> bool {
        matches!(self, Self::Yes)
    }
}

impl From<bool> for K1InvestmentConfirmation {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}
