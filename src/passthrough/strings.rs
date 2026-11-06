pub(crate) const PASSTHROUGH_STRINGS: &[(&str, &str)] = &[
    ("DISPLAY_K1S_PAPER", "Schedule K-1 (Form 1120-S)"),
    ("DISPLAY_K1S_ATTACHMENTS", "Attached statements"),
    ("DISPLAY_K1S_PART_I", "Part I — Corporation information"),
    ("DISPLAY_K1S_PART_II", "Part II — Shareholder information"),
    ("DISPLAY_K1S_BOXES_1_9", "Part III — Boxes 1–9"),
    ("DISPLAY_K1S_BOXES_10_19", "Part III — Boxes 10–19"),
    (
        "SCORP_SOURCE_CODE_NOT_CALCULATED",
        "This S-corporation K-1 code needs treatment the App does not calculate. Keep the actual printed code and its attached statement for review.",
    ),
    (
        "SCORP_SOURCE_STATEMENT_REQUIRED",
        "Enter the actual amount or attached statement for this K-1. A missing statement amount cannot be treated as zero.",
    ),
    (
        "SCORP_SOURCE_STATEMENT_MISMATCH",
        "This K-1's printed total and its entered statement amounts differ. Correct them from the actual paper and statements; the App cannot infer an activity allocation.",
    ),
    (
        "SCORP_SOURCE_LOAN_MISMATCH",
        "This K-1's loan principal or repayments differ from the shareholder loan records. Check item I, box 16 code E and the actual loan records; adjusted tax basis is not principal.",
    ),
    (
        "SCORP_SOURCE_LOAN_RECORDS_REQUIRED",
        "The K-1 loan balances cannot yet be reconciled with the actual shareholder loan records. Enter the missing principal, repayment or basis records; no loan amount is inferred.",
    ),
    (
        "PARTNERSHIP_SOURCE_CODE_NOT_CALCULATED",
        "This partnership K-1 code needs treatment the App does not calculate. Keep the actual printed code and its attached statement for review.",
    ),
    (
        "PARTNERSHIP_SOURCE_STATEMENT_REQUIRED",
        "Enter the actual amount or attached statement for this K-1. A missing statement amount cannot be treated as zero.",
    ),
    (
        "PARTNERSHIP_SOURCE_STATEMENT_MISMATCH",
        "This K-1’s printed total and its entered statement amounts differ. Correct them from the actual paper and statements; the App cannot infer an activity allocation.",
    ),
    ("DOCUMENT_ESTATE_ACCOUNTING", "The fiduciary's accounting election for this period"),
    ("DOCUMENT_ESTATE_OTHER_SOURCE", "Does another source treatment apply?"),
    (
        "NOTE_K1_SALE_QBI_BASIS_CARRY",
        "K-1 {0}, property {1}: 2026 sale loss suspended by basis {2,number,currency}, qualified portion {3,number,currency}; original loss {4,number,currency}, original qualified portion {5,number,currency}. Keep this basis balance separate from at-risk and passive losses; qualification is fixed in the loss year.",
    ),
    (
        "NOTE_K1_SALE_QBI_RISK_CARRY",
        "K-1 {0}, property {1}: 2026 sale loss suspended by the at-risk limit {2,number,currency}, qualified portion {3,number,currency}; original loss entering this limit {4,number,currency}, original qualified portion {5,number,currency}. This balance already passed basis; do not repeat it in the basis balance.",
    ),
    (
        "NOTE_K1_SALE_QBI_PASSIVE_CARRY",
        "K-1 {0}, property {1}: 2026 sale loss suspended by the passive limit {2,number,currency}, qualified portion {3,number,currency}; original loss entering this limit {4,number,currency}, original qualified portion {5,number,currency}. This balance already passed basis and at-risk limits; do not repeat it in either earlier balance.",
    ),
    (
        "ISSUE_K1_SALE_PASSIVE_ALLOCATION",
        "Review this K-1's sale losses and owner loss worksheets; the allowed amounts do not reconcile.",
    ),
    (
        "WARN_SCORP_QBI_LEFT_OUT",
        "{0}: this S corporation's QBI statement isn't confirmed complete, so its qualified business income is left out of the QBI deduction; your tax may be lower.",
    ),
    (
        "WARN_SCORP_QBI_HISTORY_LEFT_OUT",
        "{0}: suspended-loss QBI history is missing, so historical qualified losses for the affected tax character are left out. Enter the retained history to account for these losses when allowed; including them can reduce the deduction from your other businesses and raise your tax.",
    ),
    (
        "WARN_SCORP_QBI_LOSS_LEFT_OUT",
        "{0}: this S corporation's QBI statement isn't confirmed complete, so its qualified business loss is left out of the QBI deduction, which can raise the deduction from your other businesses; your tax may be higher.",
    ),
    (
        "WARN_SCORP_QBI_ALLOCATION_LEFT_OUT",
        "{0}: The QBI deduction was left out because the qualified section 1250 allocation is unknown. Enter the allocation to include this deduction; your tax may be lower.",
    ),
    (
        "ISSUE_PARTNER_SEPARATE_ITEMS_RECONCILIATION",
        "Review the separately stated Schedule K-1 (1065) amounts: box 6b cannot exceed box 6a.",
    ),
    (
        "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED",
        "Reconstruct opening at-risk history; outside basis cannot substitute for it. Your 2025 Form 6198 figures are needed when this K-1 has a loss, a loss carried because of the at-risk limits, a section 179 deduction or earlier losses to recapture.",
    ),
    (
        "NOTE_K1_LOSS_CARRYFORWARD",
        "{0}: {1} carried to 2027 \u{2014} {2,number,currency} because of your basis, {3,number,currency} because of the at-risk limits.",
    ),
    (
        "NOTE_K1_AMT_CLASS_CARRYFORWARD",
        "{0}: AMT {1} carried to 2027 \u{2014} {2,number,currency} because of your AMT basis, {3,number,currency} because of the at-risk limits.",
    ),
    ("K1_LOSS_CLASS_ORDINARY", "ordinary business loss"),
    ("K1_LOSS_CLASS_RENTAL_REAL_ESTATE", "rental real estate loss"),
    ("K1_LOSS_CLASS_OTHER_RENTAL", "other rental loss"),
    ("K1_LOSS_CLASS_SHORT_TERM", "short-term capital loss"),
    ("K1_LOSS_CLASS_LONG_TERM", "long-term capital loss"),
    ("K1_LOSS_CLASS_COLLECTIBLES", "collectibles loss"),
    ("K1_LOSS_CLASS_SECTION1231", "section 1231 loss"),
    ("K1_LOSS_CLASS_OTHER_LOSS", "other income loss"),
    ("K1_LOSS_CLASS_SECTION179", "section 179 deduction"),
    ("K1_LOSS_CLASS_CHARITY", "cash contributions"),
    ("K1_LOSS_CLASS_OTHER_CHARITY", "other contributions"),
    ("K1_LOSS_CLASS_INVESTMENT_INTEREST", "investment interest expense"),
    ("K1_LOSS_CLASS_OTHER_DEDUCTIONS", "other deductions"),
    ("K1_LOSS_CLASS_FOREIGN_TAXES", "foreign taxes"),
    (
        "ISSUE_PARTNERSHIP_A_NEW_ACTIVITY_CANNOT_CONTAIN_PRIOR_822BC4EF",
        "A new activity cannot contain prior investment or loss history.",
    ),
    (
        "ISSUE_PARTNERSHIP_A_PARTNERSHIP_SELF_EMPLOYMENT_SOURCE_WAS_SUPPLIED_4C3D6EAC",
        "A partnership self-employment source was supplied more than once.",
    ),
    (
        "ISSUE_PARTNERSHIP_COMPLETE_THE_ORDINARY_PARTNER_BASIS_SOURCE_RECORDS_FCA2848B",
        "Complete the ordinary partner-basis source records and identify additional adjustments.",
    ),
    ("ISSUE_PARTNERSHIP_ESTABLISH_K_1_BOX_14_CODE_A_2F03EA0B", "Establish K-1 box 14 code A."),
    (
        "ISSUE_PARTNERSHIP_ESTABLISH_THE_PARTNERSHIP_BUSINESS_CLASSIFICATION_AND_2561E41A",
        "Establish the partnership business classification and material participation before applying passive limits.",
    ),
    (
        "ISSUE_PARTNERSHIP_PARTNER_AT_RISK_RECAPTURE_REQUIRES_SEPARATE_RETURN_B3B090A9",
        "Partner at-risk recapture requires separate return and character coordination.",
    ),
    (
        "ISSUE_PARTNERSHIP_RECONCILE_PARTNER_FINANCING_AND_ADDITIONAL_AT_RISK_B169E3B9",
        "Reconcile partner financing and additional at-risk adjustments before applying loss limits.",
    ),
    (
        "ISSUE_PARTNERSHIP_RESOLVE_EVERY_PARTNERSHIP_S_REGULAR_SELF_EMPLOYMENT_SOURCE_6800F63B",
        "Resolve every partnership's regular self-employment source before combining this owner's earnings.",
    ),
    (
        "ISSUE_PARTNERSHIP_REVIEW_THE_RETAINED_FORM_6198_AMOUNTS_34CD8625",
        "Review the retained Form 6198 amounts.",
    ),
    (
        "ISSUE_PARTNERSHIP_REVIEW_THE_RETAINED_ORDINARY_PASSIVE_LOSS_BALANCE_1389FA9F",
        "Review the retained ordinary passive-loss balance.",
    ),
    (
        "ISSUE_PARTNERSHIP_SELF_EMPLOYMENT_LOSS_PORTIONS_CANNOT_EXCEED_THEIR_5D13E5AC",
        "Self-employment loss portions cannot exceed their separate retained total balances.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_PARTNER_BASIS_SOURCE_AMOUNTS_CANNOT_BE_56598BD1",
        "The partner-basis source amounts cannot be reconciled.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_PARTNER_S_AT_RISK_LOSS_ALLOCATION_IS_D6B562C7",
        "The partner's at-risk loss allocation is inconsistent.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_PARTNER_S_RECAPTURE_HISTORY_IS_INCONSISTENT_4E8989F8",
        "The partner's recapture history is inconsistent.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_REGULAR_TAX_PASSIVE_ALLOCATION_DOES_NOT_4C176626",
        "The regular-tax passive allocation does not reconcile with this partnership activity.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_RETAINED_AT_RISK_HISTORY_IS_INCONSISTENT_0CD4F902",
        "The retained at-risk history is inconsistent.",
    ),
    (
        "ISSUE_PARTNERSHIP_THE_SECTION_179_DEDUCTION_ALLOWED_FOR_251BF7AD",
        "The section 179 deduction allowed for this partnership does not reconcile with its K-1 box 12 amount.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554",
        "The App calculates ordinary PTP qualified losses from reconciled basis, at-risk and same-PTP passive records. Combinations with partner deductions or other carried characters, EBL combinations with positive business adjustments, and retained-loss combinations with property or full-interest sales still need a separate qualified-source worksheet.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_COMPLETE_AND_RECONCILE_THE_PARTNERSHIP_S_ORDINARY_82FE9FDB",
        "Complete and reconcile the partnership's ordinary section 199A statement and partner adjustments.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_COMPLETE_HISTORICAL_PARTNERSHIP_QBI_LOSS_RECORDS_4E8D198F",
        "Complete historical partnership QBI loss records for each nonzero suspended balance.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_FOREIGN_PROFIT_OFFSETS_WITH_MIXED_QUALIFIED_NONQUALIFIED_663081D3",
        "Foreign profit offsets with mixed qualified/nonqualified losses require further allocation; uniformly qualified or nonqualified loss sources retain their character.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_MIXED_PARTNERSHIP_SCHEDULE_C_EXCESS_LOSSES_REQUIRE_92B2AD27",
        "Mixed partnership/Schedule C excess losses require ordinary loss-only sources with resolved domestic QBI or wholly foreign nonqualified character, without separate foreign exclusions, housing, health or retirement deductions.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_MIXED_PARTNERSHIP_S_CORPORATION_PROPERTY_CHARACTER_NEEDS_SEPAR_A57C0231",
        "Mixed partnership/S-corporation property character needs separate allocation.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_PARTNERSHIP_EXCESS_LOSS_ALLOCATION_MIXED_WITH_OTHER_9CC01703",
        "Partnership excess-loss allocation mixed with other business sources remains unresolved.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_PARTNERSHIP_EXCESS_LOSS_ALLOCATION_WITH_PROFITS__GUARANTEED_CFAD08F9",
        "Partnership excess-loss allocation with profits, guaranteed payments or special business character remains unresolved.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_PROFIT_OFFSET_PARTNERSHIP_EXCESS_LOSSES_REQUIRE_CURRENT_97A91752",
        "Profit-offset partnership excess losses require current ordinary nonpassive sources with a single loss source or the same qualified fraction, no historical losses, partner deductions, guaranteed payments or other businesses.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5",
        "Provide partnership gross income for partner-deduction QBI attribution.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363",
        "Reconcile historical partnership qualified-loss records with each separate suspended balance, including rounded totals.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RECONCILE_QUALIFIED_AND_TOTAL_GROSS_INCOME_AD915D46",
        "Reconcile qualified and total gross income for partner deductions.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RECONCILE_THE_ROUNDED_PARTNERSHIP_QBI_LOSS_YEAR_202D016C",
        "Reconcile the rounded partnership QBI loss-year amounts and their qualified portions.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RESOLVE_ACTUAL_MATERIAL_PARTICIPATION_FOR_THE_3C3DCC18",
        "Resolve actual material participation for the minimum QBI deduction; nonpassive trading classification alone is insufficient.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RESOLVE_ALL_PARTNERSHIP_LOSS_ONLY_SOURCES_BEFORE_563FC7D4",
        "Resolve all partnership loss-only sources before excess-loss allocation.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_ROUNDED_PARTNERSHIP_AT_RISK_VINTAGES_DO_NOT_93E77889",
        "Rounded partnership at-risk vintages do not reconcile to the at-risk worksheet.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_ROUNDED_PARTNERSHIP_BASIS_LOSS_VINTAGES_DO_NOT_899E8A3B",
        "Rounded partnership basis-loss vintages do not reconcile to the basis worksheet.",
    ),
    (
        "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_CAPITAL_AFCA6E93",
        "Supply explicit regular and AMT capital records for the same K-1; missing capital history is not a zero amount.",
    ),
    (
        "ISSUE_SCORP_AMT_SUPPLY_EXPLICIT_REGULAR_AND_AMT_SECTION_D67066FF",
        "Supply explicit regular and AMT section 1231 records for the same K-1; missing history is not a zero amount.",
    ),
    (
        "ISSUE_SCORP_AMT_THE_SEPARATE_AMT_STOCK__DEBT_AND_C841361B",
        "The separate AMT stock, debt and loss records are inconsistent.",
    ),
    (
        "ISSUE_SCORP_AT_RISK_RECAPTURE_NEEDS_COORDINATED_PASSIVE_INCOME_AND_A32538D4",
        "At-risk recapture needs coordinated passive-income and QBI treatment plus separate next-year deduction records before return totals can be calculated.",
    ),
    (
        "ISSUE_SCORP_AT_RISK_RECAPTURE_REQUIRES_ITS_SEPARATE_WORKSHEET_DE430CD8",
        "At-risk recapture requires its separate worksheet.",
    ),
    (
        "ISSUE_SCORP_COMPLETE_THE_BUSINESS_CLASSIFICATION_AND_APPLICABLE_05C04B0A",
        "Complete the business classification and applicable participation facts before applying passive losses.",
    ),
    (
        "ISSUE_SCORP_COMPLETE_THE_PRIOR_PASSIVE_LOSS_HISTORY_AND_EEFE19D3",
        "Complete the prior passive-loss history and ordinary-pool eligibility.",
    ),
    (
        "ISSUE_SCORP_CONFIRM_COMPLETE_NONFARMING_BUSINESS_AND_BASIS_F667C1A3",
        "Confirm complete nonfarming business and basis facts; farming, additional or unknown K-1 items need their separate worksheets.",
    ),
    (
        "ISSUE_SCORP_MULTIPLE_REPAID_LOANS_NEED_AN_ADDITIONAL_003A1C12",
        "Multiple repaid loans need an additional restoration-priority worksheet before their basis can be calculated.",
    ),
    (
        "ISSUE_SCORP_QBI_COMBINED_BUSINESS_QBI_NEEDS_THE_COMPLETE_CF7C9CBD",
        "Combined business QBI needs the complete taxable-income base before resolving current service-business eligibility.",
    ),
    (
        "ISSUE_SCORP_QBI_COMPLETE_PARTICIPATION_FACTS_FOR_THE_MINIMUM_80D7BD0E",
        "Complete participation facts for the minimum QBI deduction; nonpassive trading classification alone does not establish active business income.",
    ),
    (
        "ISSUE_SCORP_QBI_COMPLETE_THE_ORDINARY_BUSINESS_QBI_STATEMENT_AND_7B111E6E",
        "Complete the ordinary-business QBI statement and resolve additional shareholder adjustments.",
    ),
    (
        "ISSUE_SCORP_QBI_DIFFERENT_QUALIFIED_LOSS_PROPORTIONS_NEED_A_SEPARATE_3D45B7F5",
        "Different qualified-loss proportions need a separate excess-loss allocation method.",
    ),
    (
        "ISSUE_SCORP_QBI_EXCESS_BUSINESS_LOSS_CANNOT_EXCEED_THE_6C6934CC",
        "Excess business loss cannot exceed the net current ordinary business loss.",
    ),
    (
        "ISSUE_SCORP_QBI_EXCESS_LOSS_ALLOCATION_ACROSS_BUSINESSES__PRIOR_LOSSES_E058DC3A",
        "Excess-loss allocation across businesses, prior losses or investment-income classes requires separate coordination.",
    ),
    (
        "ISSUE_SCORP_QBI_EXCESS_LOSS_ALLOCATION_NEEDS_ONE_RESOLVED_CURRENT_D211A39E",
        "Excess-loss allocation needs one resolved current activity per ordinary business.",
    ),
    (
        "ISSUE_SCORP_QBI_EXCESS_LOSS_QBI_REQUIRES_RESOLVED_LOSS_ONLY_ACTIVITIES_1A1A4180",
        "Excess-loss QBI requires resolved loss-only activities and a valid total loss.",
    ),
    (
        "ISSUE_SCORP_QBI_HISTORICAL_EXCESS_LOSS_ALLOCATION_REQUIRES_COMPLETE_ORDINARY_C15F9B6A",
        "Historical excess-loss allocation requires complete ordinary nonpassive loss sources.",
    ),
    (
        "ISSUE_SCORP_QBI_HISTORICAL_EXCESS_LOSS_ALLOCATION_REQUIRES_RESOLVED_LOSS_ONLY_7F81444D",
        "Historical excess-loss allocation requires resolved loss-only QBI activities.",
    ),
    (
        "ISSUE_SCORP_QBI_LOOKBACK_RECAPTURE_MUST_FIT_THE_CURRENT_AAFE1FFE",
        "Lookback recapture must fit the current business's property gain.",
    ),
    (
        "ISSUE_SCORP_QBI_LOSS_DEFERRAL_NEEDS_RESOLVED_CURRENT_AND_9E75276B",
        "Loss deferral needs resolved current and historical QBI activities.",
    ),
    ("ISSUE_SCORP_QBI_ORDINARY_BUSINESS_NET_OVERFLOW_468BBBC0", "Ordinary business net overflow."),
    ("ISSUE_SCORP_QBI_ORDINARY_LOSS_TOTAL_OVERFLOW_CBAFF6CC", "Ordinary loss total overflow."),
    (
        "ISSUE_SCORP_QBI_PARTIAL_LOOKBACK_RECAPTURE_OF_MIXED_QUALIFIED_NONQUALIFIED_9084BCB0",
        "Partial lookback recapture of mixed qualified/nonqualified property gains needs source-gain allocation.",
    ),
    (
        "ISSUE_SCORP_QBI_POSITIVE_BUSINESS_INCOME_REQUIRES_SEPARATE_HISTORICAL_C49B8889",
        "Positive business income requires separate historical loss allocation.",
    ),
    (
        "ISSUE_SCORP_QBI_POSITIVE_EXCESS_LOSS_ALLOCATION_INPUTS_REQUIRE_CURRENT_1BCA0633",
        "Positive excess-loss allocation inputs require current ordinary nonpassive income with eligibility-adjusted QBI character and no historical losses.",
    ),
    (
        "ISSUE_SCORP_QBI_QUALIFIED_LOSSES_CANNOT_EXCEED_THE_ALLOWED_73D3C443",
        "Qualified losses cannot exceed the allowed total loss.",
    ),
    (
        "ISSUE_SCORP_QBI_QUALIFIED_LOSS_HISTORY_MUST_RECONCILE_TO_DDB88FE9",
        "Qualified loss history must reconcile to the resolved loss character.",
    ),
    ("ISSUE_SCORP_QBI_QUALIFIED_LOSS_TOTAL_OVERFLOW_426F24B2", "Qualified loss total overflow."),
    (
        "ISSUE_SCORP_QBI_RECONCILE_THE_QBI_LOSS_YEAR_RECORDS_WITH_14CFA998",
        "Reconcile the QBI loss-year records with this character's current and suspended losses.",
    ),
    (
        "ISSUE_SCORP_QBI_RECONCILE_THE_SECTION_1231_QBI_STATEMENT_431CECE7",
        "Reconcile the section 1231 QBI statement with its own gain and suspended-loss source records.",
    ),
    ("ISSUE_SCORP_QBI_RESOLVE_LOSS_CHARACTERS_AB0F7C8C", "Resolve loss characters."),
    (
        "ISSUE_SCORP_QBI_RESOLVE_PARTICIPATION_FOR_THE_MINIMUM_QBI_D07583A4",
        "Resolve participation for the minimum QBI deduction on combined business income.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_AT_RISK_QBI_LOSS_YEAR_RECORDS_ARE_5A80FD65",
        "The at-risk QBI loss-year records are inconsistent.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_EXCESS_BUSINESS_LOSS_AND_QBI_550201BC",
        "The excess business loss and QBI character must fit the allowed ordinary loss.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_FORMER_PASSIVE_QBI_LOSS_HISTORY_IS_5E7A8F29",
        "The former-passive QBI loss history is inconsistent.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_PASSIVE_QBI_LOSS_HISTORY_IS_73C1614D",
        "The passive QBI loss history is inconsistent.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_PASSIVE_SUSPENSION_MUST_FIT_THIS_B3AA5C14",
        "The passive suspension must fit this activity's computed passive loss.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_PROPERTY_QBI_SUSPENSION_MUST_FIT_CE29E6B2",
        "The property QBI suspension must fit this character's passive-loss allocation.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_QBI_SUSPENSION_IS_INCONSISTENT_WITH_20B70F99",
        "The QBI suspension is inconsistent with allowed ordinary losses.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_QBI_SUSPENSION_MUST_FIT_THIS_81DC6365",
        "The QBI suspension must fit this activity's computed passive loss.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_ROUNDED_AT_RISK_QBI_LOSS_HISTORY_98A3E4A9",
        "The rounded at-risk QBI loss history must reconcile to this character's at-risk worksheet.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_ROUNDED_PASSIVE_QBI_HISTORY_MUST_AE740F7A",
        "The rounded passive QBI history must reconcile to prior passive losses.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_ROUNDED_QBI_LOSS_YEAR_AMOUNTS_MUST_3737C544",
        "The rounded QBI loss-year amounts must reconcile with this character's basis worksheet; review source amounts.",
    ),
    (
        "ISSUE_SCORP_QBI_THE_SECTION_179_DEDUCTION_ALLOWED_FOR_4BE9C19A",
        "The section 179 deduction allowed for this corporation does not reconcile with its K-1 box 11 amount.",
    ),
    (
        "ISSUE_SCORP_SECTION_1231_BUSINESS_PROPERTY_ITEMS_NEED_COORDINATED_2B9506E8",
        "Section 1231 business-property items need coordinated passive/QBI treatment and final return integration before ordinary activity results are final.",
    ),
    (
        "ISSUE_SCORP_THE_FORMER_PASSIVE_CHARACTER_BALANCES_CANNOT_BE_B5669279",
        "The former-passive character balances cannot be reconciled.",
    ),
    (
        "ISSUE_SCORP_THE_STOCK_DEBT_BASIS_RECORDS_ARE_INCONSISTENT_9799BBBB",
        "The stock/debt basis records are inconsistent.",
    ),
    (
        "NOTE_SCORP_REGULAR_SECTION1231",
        "S-corporation document {0,number,integer}, regular section 1231 worksheet: net before passive limits {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}; prior passive-suspended loss {4,number,currency}; business-asset section 1250 component {5,number,currency}. These are intermediate amounts, not final ordinary or capital income. Keep the three suspension balances separate.",
    ),
    (
        "NOTE_SCORP_AMT_SECTION1231",
        "S-corporation document {0,number,integer}, AMT section 1231 worksheet: net before passive limits {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}; prior passive-suspended loss {4,number,currency}; business-asset section 1250 component {5,number,currency}. These are intermediate amounts, not final ordinary or capital income. Keep the three suspension balances separate.",
    ),
    (
        "ISSUE_SCORP_LOAN_BASIS_SOURCE_MISMATCH",
        "The shareholder-loan basis results do not match the source records.",
    ),
    (
        "ISSUE_YOUNG_PARTNER_GROSS_SE_INCOME",
        "Supply K-1 gross SE income to allocate the mixed partner's half-SE deduction between profit and guaranteed payments.",
    ),
    (
        "ISSUE_YOUNG_PARTNER_GROSS_SE_RECONCILIATION",
        "Reconcile K-1 gross SE income with guaranteed payments before allocating the young partner's half-SE deduction.",
    ),
    (
        "ISSUE_BBA_AUDIT_SOURCE_ID",
        "Identify the single Form 8986 by its tracking number, audit control number, or issuing entity tax ID.",
    ),
    (
        "ISSUE_BBA_AUDIT_FURNISHED_YEAR",
        "Only a Form 8986 actually furnished during the 2026 reporting year belongs on this return.",
    ),
    (
        "ISSUE_BBA_AUDIT_WHOLE_DOLLAR_VALUES",
        "Use whole-dollar 2025 Form 1040 and partnership-interest amounts. The signed Form 8986 adjustment cannot make the corrected K-1 interest or taxable income negative; both original and corrected income must use the 2025 Tax Table.",
    ),
    (
        "ISSUE_BBA_AUDIT_PRIOR_INCOME_RECONCILIATION",
        "The source wages and original K-1 interest do not reconcile to 2025 taxable income after the basic standard deduction. Other income or deductions require a full Form 8978 refigure.",
    ),
    (
        "ISSUE_BBA_AUDIT_TAX_TABLE_STATUS",
        "The 2025 filing status cannot be calculated from the published Tax Table.",
    ),
    (
        "ISSUE_BBA_AUDIT_PRIOR_TAX_RECONCILIATION",
        "The original 2025 Form 1040 income tax does not match the stated ordinary taxable income and filing status; refigure that return before using Form 8978.",
    ),
    (
        "ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION",
        "Compute and document Form 8978 line 17 interest on the additional 2025 income tax before using this route; that interest is part of the reporting-year net payment, outside Form 1040 tax.",
    ),
    (
        "NOTE_SCORP_MATERIAL_ESTABLISHED",
        "S-corporation document {0}, material participation: established. This classifies ordinary business participation only; passive-loss treatment, investment-tax classification and QBI eligibility are evaluated separately.",
    ),
    (
        "NOTE_SCORP_MATERIAL_NOT_ESTABLISHED",
        "S-corporation document {0}, material participation: not established by the applicable tests. This classifies ordinary business participation only; passive-loss treatment, investment-tax classification and QBI eligibility are evaluated separately.",
    ),
    (
        "NOTE_SCORP_MATERIAL_UNRESOLVED",
        "S-corporation document {0}, material participation: unresolved; complete facts or significant-participation aggregation are needed. This classifies ordinary business participation only; passive-loss treatment, investment-tax classification and QBI eligibility are evaluated separately.",
    ),
    (
        "NOTE_SCORP_DISTRIBUTION_GAIN",
        "S-corporation document {0}, distribution gain: {1,number,currency}. Nondividend distributions exceeding stock basis, capital gain on Form 8949 and Schedule D with the holding period of the shares.",
    ),
    (
        "NOTE_SCORP_BASIS_ALLOWED_LOSS",
        "S-corporation document {0}, basis allowed loss: {1,number,currency}. Ordinary loss passing basis limits only; subsequent limitations may disallow it.",
    ),
    (
        "NOTE_SCORP_BASIS_SUSPENDED_LOSS",
        "S-corporation document {0}, basis suspended loss: {1,number,currency}. Ordinary loss still suspended by basis; do not combine with at-risk or passive suspended balances.",
    ),
    (
        "NOTE_SCORP_NONDEDUCTIBLE_CARRY",
        "S-corporation document {0}, nondeductible carry: {1,number,currency}. Unused nondeductibles carried only under the continuing losses-first election.",
    ),
    (
        "NOTE_SCORP_LOAN_PRINCIPAL_FORMAL",
        "S-corporation document {0}, loan {2} principal: {1,number,currency}. Next year: retain this separate written note and its adjusted debt basis.",
    ),
    (
        "NOTE_SCORP_LOAN_PRINCIPAL_OPEN",
        "S-corporation document {0}, loan {2} principal: {1,number,currency}. Next year: retain this current open account; annual advance/repayment netting applies.",
    ),
    (
        "NOTE_SCORP_LOAN_PRINCIPAL_FORMER_OPEN",
        "S-corporation document {0}, loan {2} principal: {1,number,currency}. Next year: retain this as a separate former open account that exceeded {3,number,currency}. Do not combine it with a new open account, even if its balance is now lower. Annual netting no longer applies; repayment gain remains ordinary.",
    ),
    (
        "NOTE_SCORP_LOAN_NONTAXABLE_REPAYMENT",
        "S-corporation document {0}, loan {2} nontaxable repayment: {1,number,currency}. Principal recovered without taxable gain; interest is separate.",
    ),
    (
        "NOTE_SCORP_LOAN_GAIN_SHORT",
        "S-corporation document {0}, loan {2} gain: {1,number,currency}. Short-term personal-investment gain on Form 8949 and Schedule D, separate from corporate operating income and QBI.",
    ),
    (
        "NOTE_SCORP_LOAN_GAIN_LONG",
        "S-corporation document {0}, loan {2} gain: {1,number,currency}. Long-term personal-investment gain on Form 8949 and Schedule D, separate from corporate operating income and QBI.",
    ),
    (
        "NOTE_SCORP_LOAN_GAIN_DATED",
        "S-corporation document {0}, loan {2} gain: {1,number,currency}. Written-note repayment gain; dated payments allocate the supported capital gain between short- and long-term Form 8949/Schedule D amounts.",
    ),
    (
        "NOTE_SCORP_LOAN_GAIN_NO_REPAYMENT",
        "S-corporation document {0}, loan {2} gain: {1,number,currency}. No written-note principal was repaid this year, so the note has no repayment gain.",
    ),
    (
        "NOTE_SCORP_LOAN_GAIN_OPEN",
        "S-corporation document {0}, loan {2} gain: {1,number,currency}. Open-account repayment gain on Form 4797 and Schedule 1 line 4; personal-investment income, not corporate operating income or QBI.",
    ),
    ("DISPLAY_K1_1041_BOX6", "Box 6 business"),
    ("DISPLAY_K1_1041_BOX7", "Box 7 rental real estate"),
    ("DISPLAY_K1_1041_BOX8", "Box 8 other rental"),
    ("DISPLAY_K1_1041_BOXES_10_12", "Boxes 10–12"),
    ("DISPLAY_K1_1041_BOX13", "Box 13 credits"),
    ("DISPLAY_K1_1041_BOX14_CODES", "Box 14 codes A, H and I"),
    ("DISPLAY_K1S_GAINS", "Boxes 7–9 gains"),
    ("DISPLAY_K1P_PORTFOLIO", "Boxes 5–9"),
    ("DISPLAY_K1S_QBI_STATEMENT", "Section 199A statement"),
    ("DISPLAY_K1S_QBI_LOSSES", "Losses and section 1231"),
    ("DISPLAY_K1P_DEDUCTIONS", "Boxes 12–15"),
    ("DISPLAY_K1P_EXEMPT", "Box 18"),
    ("DISPLAY_BASIS_7203", "Basis (Form 7203)"),
    ("DISPLAY_AT_RISK_6198", "At risk (Form 6198)"),
    ("DISPLAY_PASSIVE_8582", "Passive (Form 8582)"),
    ("DISPLAY_K3_COUNTRIES", "Countries and entries"),
    ("DISPLAY_K3_CATEGORIES", "Categories"),
    ("DISPLAY_K3_PARTNER", "Left for you to allocate"),
    ("DISPLAY_CARRIED_LOSSES", "Losses carried"),
    ("DISPLAY_SE_PART", "Self-employment part"),
    (
        "ISSUE_PARTNER_K1_RENTAL_DISPOSITION",
        "A partnership rental activity's installment sale, like-kind exchange, casualty or theft, or disposition to a related buyer needs separate property and passive-loss worksheets that this App does not calculate.",
    ),
    (
        "ISSUE_PARTNER_K1_OTHER_INCOME",
        "Schedule K-1 (1065) box 6c dividend equivalents, box 11 codes B, D, I, L, M, N, O, P and ZZ, a code A portfolio loss or REMIC residual interest, a code C straddle, a code F section 743(b) adjustment to another item and gambling of a gambling business are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_OTHER_DEDUCTIONS",
        "Schedule K-1 (1065) box 13 code O dependent care benefits (Form 2441 Part III) and a code V section 743(b) adjustment to another item are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_INTERNATIONAL",
        "Schedule K-3 (Form 1065) Parts VI and VII (sections 951(a)(1) and 951A inclusions, passive foreign investment companies) and the Part I box 6 section 267A disallowed deduction are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_AMT_ITEMS",
        "Schedule K-1 (1065) box 17 alternative minimum tax items beyond the separate AMT activity records (Form 6251) are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_PROPERTY_DISTRIBUTION",
        "Schedule K-1 (1065) box 19 codes B and C property distributions (sections 732 and 737) are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_OTHER_INFORMATION",
        "Schedule K-1 (1065) box 20 investment income or expense totals requiring separate Form 4952 reconciliation (codes A and B), recapture, look-back interest, section 179 dispositions other than fully taxable sales, installment and section 1260 interest, precontribution gain, section 1061, inversion gain and section 48A–48D items (codes F through M, O through S, W, AL for a U.S. transferor, AM, AP, AS through AU, and ZZ with an amount) are not calculated.",
    ),
    (
        "ISSUE_PARTNER_K1_MULTIPLE_ACTIVITIES",
        "For Schedule K-1 (1065) box 22 or box 23 checked, or a statement with multiple activities, use the Partnership K-1 with a distinct-activity statement document. Enter the actual activity statement and retained loss records there; unsupported activity types still receive their own review notice.",
    ),
    (
        "ISSUE_SCORP_K1_RENTAL_DISPOSITION",
        "An S corporation rental activity's installment sale, like-kind exchange, casualty or theft, or disposition to a related buyer needs separate property and passive-loss worksheets that this App does not calculate.",
    ),
    (
        "ISSUE_K1_RENTAL_PROPERTY_LIMITED",
        "A rental activity's section 1231 loss held back by basis or at-risk limits needs an activity-specific property carryover that this K-1 record does not calculate.",
    ),
    ("K1_RENTAL_DISPOSED_TWICE", "Select each rental activity sold in full only once."),
    (
        "ISSUE_K1_SALE_AMT_UNAVAILABLE",
        "AMT amounts for a section 179 sale are unavailable. Enter the property's AMT statement before its AMT sale worksheet can be calculated.",
    ),
    (
        "K1_SALE_AMT_CHOICE",
        "Separate sale amounts require the Different for AMT choice; otherwise leave the separate amounts blank.",
    ),
    ("K1_SALE_SOURCE_OWNER", "Use the regular or AMT record of this same accepted K-1 owner."),
    (
        "K1_ENTITY_PERIOD_DATES",
        "Enter the entity’s actual beginning and ending dates, with the ending date on or after the beginning date.",
    ),
    ("K1_SALE_DATES", "The date sold must be on or after the date acquired and placed in service."),
    (
        "K1_SALE_BASIS_BELOW_ZERO",
        "Depreciation plus the section 179 deduction passed through cannot exceed the property's cost or other basis plus selling expenses.",
    ),
    (
        "K1_SALE_CARRYOVER_ABOVE_CARRYOVER",
        "The sold-property part cannot exceed this K-1's section 179 carryover.",
    ),
    (
        "K1_SALE_CARRYOVER_ABOVE_SHARE",
        "The sold-property carryover cannot exceed the section 179 deduction passed through for the properties sold.",
    ),
    (
        "ISSUE_K1_SALE_RECORDS",
        "Enter valid section 179 sale statements and nonnegative carryover amounts within the supported limits.",
    ),
    (
        "ISSUE_K1_SALE_CARRYOVER_SPLIT",
        "More than one section 179 property was sold and a carryover belongs to sold property. Each property's carryover must be allocated separately; this K-1 does not calculate that allocation.",
    ),
    (
        "ISSUE_K1_RENTAL_PROFESSIONAL_STATUS",
        "A Schedule K-1 box 2 rental real estate activity in which a real estate professional materially participated needs that owner's Real estate professional answer on About you or About your spouse.",
    ),
    (
        "ISSUE_K1_RENTAL_PTP_PROPERTY",
        "Rental property gains, losses and property passive-loss carryovers of a publicly traded partnership need its separate property-character worksheet.",
    ),
    (
        "ISSUE_K1_RENTAL_PTP_PARTICIPATION",
        "A real estate professional rental of a publicly traded partnership needs its separate former-passive and participation worksheet.",
    ),
    (
        "ISSUE_K1_RENTAL_PTP_DISPOSITION",
        "A disposition involving rental activities of a publicly traded partnership needs its separate entire-interest and suspended-loss worksheet.",
    ),
    (
        "ISSUE_K1_RENTAL_PTP_QBI",
        "A rental section 199A statement of a publicly traded partnership needs its actual qualified-source and specified-service classification before the qualified PTP deduction can be calculated.",
    ),
    (
        "ISSUE_SCORP_K1_OTHER_INCOME",
        "Schedule K-1 (1120-S) box 10 codes B, D, E, F, G, I, M, N, O and ZZ, a code A portfolio loss or REMIC residual interest, a code C straddle and gambling of a gambling business are not calculated.",
    ),
    (
        "ISSUE_SCORP_K1_INTERNATIONAL",
        "Schedule K-3 (Form 1120-S) Parts V and VI (sections 951(a)(1) and 951A inclusions, passive foreign investment companies) and the Part I box 6 section 267A disallowed deduction are not calculated.",
    ),
    (
        "ISSUE_SCORP_K1_AMT_ITEMS",
        "Schedule K-1 (1120-S) box 15 alternative minimum tax items beyond the separate AMT records and the code F private-activity bond interest (Form 6251) are not calculated.",
    ),
    (
        "ISSUE_SCORP_K1_OTHER_INFORMATION",
        "Schedule K-1 (1120-S) box 17 recapture, look-back interest, section 179 dispositions other than fully taxable sales, installment and section 1260 interest, inversion gain, section 48A–48D and section 174A items (codes E through Q, AP, AS through AU, BA, and ZZ with an amount) are not calculated.",
    ),
    (
        "ISSUE_SCORP_K1_MULTIPLE_ACTIVITIES",
        "For Schedule K-1 (1120-S) box 18 or a box 19 statement with multiple activities, use the S-corporation K-1 with a distinct-activity statement document. Enter the actual activity statement and retained loss records there; unsupported activity types still receive their own review notice.",
    ),
    (
        "ISSUE_ESTATE_K1_AMT_ITEMS",
        "A Schedule K-1 (1041) box 12 code G, H or I amount is listed but not entered. Enter its signed amount and treatment in the box 12 reconciliation; this AMT adjustment cannot be left out.",
    ),
    (
        "ISSUE_ESTATE_K1_OTHER_INFORMATION",
        "Unclassified Schedule K-1 (1041) box 14 information needs the fiduciary's actual income, tax and return treatment before a complete federal estimate. Identified credit, deduction or exclusion benefits have separate categories.",
    ),
    (
        "NOTE_S_CORPORATION_DOCUMENT_ID_PROPERTY_QBI_LOSS_TRACKING_EST_EB69F731",
        "S-corporation document {0}, property QBI loss tracking: establish current-year service-business eligibility using final taxable income before fixing new qualified suspension balances. Historical eligibility is not redetermined.",
    ),
    (
        "NOTE_S_CORPORATION_DOCUMENT_ID_THIS_K_1_S_LOSSES_DO_NOT_EXCEE_2A4E36A2",
        "S-corporation document {0}: this K-1's losses do not exceed its activity income and nothing is carried at risk from 2025, so no Form 6198 is needed and your amount at risk is not figured.",
    ),
    (
        "NOTE_SCORP_AT_RISK_AMOUNT",
        "S-corporation document {0}, {1}: {2,number,currency}. Amount after at-risk limits; passive, QBI, AMT and return-level rules apply separately.",
    ),
    ("SCORP_RISK_CURRENT_INCOME", "current activity income before recapture"),
    ("SCORP_RISK_YEAR_END_INVESTMENT", "year-end at-risk investment before recapture"),
    ("SCORP_RISK_OPENING_INVESTMENT", "investment before current income"),
    ("SCORP_RISK_ALLOWED_LOSS", "loss allowed through at-risk"),
    ("SCORP_RISK_SUSPENDED_LOSS", "loss still suspended by at-risk"),
    (
        "SCORP_RISK_NET_LOSS_ABSORBED",
        "net loss absorbed by at-risk including nondeductible expenses",
    ),
    ("SCORP_RISK_NONDEDUCTIBLE_ABSORBED", "nondeductible expenses absorbed by at-risk investment"),
    ("SCORP_RISK_NONDEDUCTIBLE_SUSPENDED", "nondeductible expenses still suspended by at-risk"),
    ("SCORP_RISK_RECAPTURE_INCOME", "recapture income"),
    ("SCORP_RISK_RECAPTURE_DEDUCTION", "recapture deduction for next year"),
    (
        "ISSUE_SCORP_RISK_SOURCE_INCONSISTENT",
        "The basis and at-risk source records are inconsistent.",
    ),
    (
        "ISSUE_SCORP_RISK_FINANCING_UNRESOLVED",
        "Unknown or additional financing, gain or basis adjustments prevent the limited at-risk worksheet; retain the separate source records.",
    ),
    (
        "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION",
        "Taxable debt repayments combined with returns of excluded investment need source-level allocation to avoid counting the same repayment twice.",
    ),
    (
        "ISSUE_SCORP_RISK_BASIS_COORDINATION",
        "Simultaneous basis and at-risk limits on deductible and nondeductible expenses need their coordinated allocation worksheet.",
    ),
    (
        "ISSUE_SCORP_RISK_DETAILED_HISTORY_INCONSISTENT",
        "The retained detailed-method history is inconsistent.",
    ),
    ("ISSUE_SCORP_RISK_LOSS_INCONSISTENT", "The at-risk loss amounts are inconsistent."),
    ("ISSUE_SCORP_RISK_RECAPTURE_INCONSISTENT", "The at-risk recapture history is inconsistent."),
    (
        "ISSUE_SCORP_RISK_HISTORY_NEEDED",
        "Schedule K-1 (1120-S): reconstruct the opening at-risk history (Form 6198); stock and debt basis cannot substitute for it. Your 2025 Form 6198 figures are needed when this K-1 has a loss, a loss carried because of the at-risk limits, a box 11 section 179 deduction or earlier losses to recapture.",
    ),
    (
        "ISSUE_PARTNER_TAXABLE_HEALTH_PREMIUMS",
        "Taxable partnership-paid health premiums must be included in both the entered premium records and guaranteed service payments, without adding either amount twice.",
    ),
    (
        "ISSUE_PARTNER_DISTRIBUTION_HEALTH_PREMIUMS",
        "Distribution-paid health premiums must fit the coverage total after taxable premiums and be included once in both regular and AMT cash-distribution records.",
    ),
    (
        "NOTE_SCORP_QBI_PASSIVE_RELEASE",
        "S-corporation document {0}, QBI loss passing passive limits: {1,number,currency}. Excess-business-loss limits, current QBI income and final deduction integration remain required.",
    ),
    (
        "NOTE_SCORP_QBI_PASSIVE_CARRY",
        "S-corporation document {0}, {1} passive-loss carryforward: {2,number,currency}; QBI portion: {3,number,currency}. Original suspended loss: {4,number,currency}; original QBI portion: {5,number,currency}. This balance already passed basis and at-risk limits.",
    ),
    (
        "NOTE_SCORP_QBI_BASIS_RELEASE",
        "S-corporation document {0}, QBI loss passing basis limits: {1,number,currency}. This is loss character, not a final QBI deduction; subsequent loss limits and return integration remain required.",
    ),
    (
        "NOTE_SCORP_QBI_BASIS_CARRY",
        "S-corporation document {0}, {1} basis-loss carryforward: {2,number,currency}; QBI portion: {3,number,currency}. Original suspended loss: {4,number,currency}; original QBI portion: {5,number,currency}. Keep this separate from at-risk and passive carryforwards.",
    ),
    (
        "NOTE_SCORP_QBI_RISK_RELEASE",
        "S-corporation document {0}, QBI loss passing at-risk limits: {1,number,currency}. Passive and excess-business-loss limits still apply; this is not the final QBI deduction.",
    ),
    (
        "NOTE_SCORP_QBI_RISK_CARRY",
        "S-corporation document {0}, {1} at-risk-loss carryforward: {2,number,currency}; QBI portion: {3,number,currency}. Original suspended loss: {4,number,currency}; original QBI portion: {5,number,currency}. This balance already passed basis limits.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_GAIN",
        "S-corporation document {0}, potential section 1231 QBI gain {1,number,currency}. Apply current-year SSTB eligibility and return-wide capital exclusion before including any amount in QBI.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_PASSIVE_RELEASE",
        "S-corporation document {0}, property loss passing passive limits: {1,number,currency}; tracked QBI portion {2,number,currency}. Intermediate loss character, subject to return-wide section 1231 capital exclusion and remaining return limits; not a QBI deduction.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_PASSIVE_CARRY",
        "S-corporation document {0}, {1} property passive suspension: {2,number,currency}; tracked QBI portion {3,number,currency}; original suspended loss {4,number,currency}; original QBI portion {5,number,currency}. Retain separately from ordinary losses; final character and deductibility still require review.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_BASIS_RELEASE",
        "S-corporation document {0}, property loss passing basis limits: {1,number,currency}; tracked QBI portion {2,number,currency}. Intermediate loss character, subject to return-wide section 1231 capital exclusion and remaining return limits; not a QBI deduction.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_BASIS_CARRY",
        "S-corporation document {0}, {1} property basis suspension: {2,number,currency}; tracked QBI portion {3,number,currency}; original suspended loss {4,number,currency}; original QBI portion {5,number,currency}. Retain separately from ordinary losses; final character and deductibility still require review.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_RISK_RELEASE",
        "S-corporation document {0}, property loss passing at-risk limits: {1,number,currency}; tracked QBI portion {2,number,currency}. Intermediate loss character, subject to return-wide section 1231 capital exclusion and remaining return limits; not a QBI deduction.",
    ),
    (
        "NOTE_SCORP_QBI_PROPERTY_RISK_CARRY",
        "S-corporation document {0}, {1} property at-risk suspension: {2,number,currency}; tracked QBI portion {3,number,currency}; original suspended loss {4,number,currency}; original QBI portion {5,number,currency}. Retain separately from ordinary losses; final character and deductibility still require review.",
    ),
    (
        "NOTE_SCORP_CAPITAL_REGULAR_SHORT",
        "S-corporation document {0}, regular short-term portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_AMT_SHORT",
        "S-corporation document {0}, AMT short-term portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_REGULAR_LONG",
        "S-corporation document {0}, regular long-term excluding collectibles portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_AMT_LONG",
        "S-corporation document {0}, AMT long-term excluding collectibles portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_REGULAR_COLLECTIBLES",
        "S-corporation document {0}, regular long-term collectibles (28%) portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_AMT_COLLECTIBLES",
        "S-corporation document {0}, AMT long-term collectibles (28%) portfolio capital: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Only the net amount reaches Schedule D; retain both suspended balances separately from the return's capital-loss carryforward.",
    ),
    (
        "NOTE_SCORP_CAPITAL_REGULAR_TOTAL_LONG",
        "S-corporation document {0}, regular total long-term portfolio capital including collectibles: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Use these totals for next year's long-term balances and retain the separately reported collectible portions.",
    ),
    (
        "NOTE_SCORP_CAPITAL_AMT_TOTAL_LONG",
        "S-corporation document {0}, AMT total long-term portfolio capital including collectibles: net {1,number,currency}; basis-suspended loss {2,number,currency}; at-risk-suspended loss {3,number,currency}. Use these totals for next year's long-term balances and retain the separately reported collectible portions.",
    ),
    (
        "NOTE_SCORP_CAPITAL_REGULAR_1250",
        "S-corporation document {0}, regular portfolio unrecaptured section 1250 gain: partnership-interest source {1,number,currency}, fund/trust source {2,number,currency}. Included in long-term income, not additional income. These are the line 10/11 components before the return's capital-loss offsets and 25% rate calculation.",
    ),
    (
        "NOTE_SCORP_CAPITAL_AMT_1250",
        "S-corporation document {0}, AMT portfolio unrecaptured section 1250 gain: partnership-interest source {1,number,currency}, fund/trust source {2,number,currency}. Included in long-term income, not additional income. These are the line 10/11 components before the return's capital-loss offsets and 25% rate calculation.",
    ),
    ("ESTATE_ACTIVITY_ID_TWICE", "Each saved beneficiary activity can appear only once."),
    ("ESTATE_ACTIVITY_NAME_TWICE", "Give each fiduciary activity a distinct source name."),
    ("ESTATE_BOX6_RECONCILE", "The business activities must total printed box 6."),
    ("ESTATE_BOX7_RECONCILE", "The real-estate rental activities must total printed box 7."),
    ("ESTATE_BOX8_RECONCILE", "The other rental activities must total printed box 8."),
    (
        "ESTATE_ACTIVITY_AMOUNT_INVALID",
        "Check each beneficiary activity amount, source name and retained passive balance.",
    ),
    (
        "ESTATE_ACTIVITY_PARTICIPATION",
        "Use business material-participation facts for a business, or real-estate-professional facts for a real-estate rental.",
    ),
    ("ESTATE_QBI_AMOUNT_INVALID", "Check the fiduciary QBI amounts and retained loss records."),
    (
        "ESTATE_QBI_ALLOCATION_RECONCILE",
        "QBI activity allocations must identify each linked activity once and reconcile to the fiduciary statement without repeating qualified losses.",
    ),
    ("ESTATE_QBI_SOURCE_TWICE", "Enter each fiduciary QBI statement only once."),
    ("ESTATE_SELECTED_ACTIVITY_TWICE", "Select each fiduciary activity only once."),
    (
        "ESTATE_ACTIVITY_LINK_INVALID",
        "Select a saved activity from this K-1. Save new activities before linking them.",
    ),
    ("LABEL_ESTATE_ACTIVITY", "{0}"),
    (
        "K1_QBI_PRINTED_CURRENT_RECONCILE",
        "Reconcile the actual signed current qualified allocations with their reported QBI block statement.",
    ),
    (
        "K1_ACTIVITY_SALE_SOURCE_TWICE",
        "A physical sale source can belong to only one actual component here. Do not repeat the same sale.",
    ),
    (
        "K1_ACTIVITY_PROPERTY_SOURCE_TWICE",
        "An investment-property source can belong to only one actual component here. Do not repeat the same property.",
    ),
    (
        "LABEL_K1_PROPERTY_REHABILITATION",
        "Rehabilitation: {0,number,currency}, placed in service {1} (source {2})",
    ),
    ("LABEL_K1_PROPERTY_ENERGY", "Energy property: {0,number,currency} basis, {1} (source {2})"),
    (
        "LABEL_K1_PROPERTY_CLEAN_ELECTRICITY",
        "Clean electricity property: {0,number,currency} basis, {1} (source {2})",
    ),
    (
        "K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED",
        "A source record no longer belongs to this K-1 and source role. Reopen the K-1.",
    ),
    ("K1_ACTIVITY_SOURCE_ID_REPEATED", "A source record identity is repeated in this K-1."),
    (
        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
        "Select an existing source in this same K-1. Save new sources and reopen before selecting them.",
    ),
    ("K1_QBI_COMPONENT_REPEATED", "Each component may appear once in a reported QBI block."),
    (
        "K1_ACTIVITY_BENEFIT_PAYMENT_SOURCE_CHANGED",
        "Reopen this Schedule K-1 to use the saved health-payment source.",
    ),
    (
        "K1_ACTIVITY_COMPLETE_BASIS_SOURCE_MISSING",
        "Establish the opening basis and prior nondeductible amounts, or mark the basis records incomplete.",
    ),
    (
        "K1_ACTIVITY_SOURCE_ENTITY_MISMATCH",
        "Use source records for this K-1 form. Partnership and S-corporation records are different.",
    ),
    (
        "K1_ACTIVITY_SOLD_CARRY_ABOVE_TOTAL",
        "The section 179 carry for sold property cannot exceed the total section 179 business-income carry.",
    ),
    (
        "K1_ACTIVITY_SE_HISTORY_ABOVE_LOSS",
        "The self-employment loss portions cannot exceed their corresponding loss-stage balances.",
    ),
    ("K1_ACTIVITY_SOURCE_SIZE_INVALID", "Enter the required source names for this K-1."),
    (
        "K1_QBI_SOURCE_INVALID",
        "Check this reported block’s current qualified-source amounts and retained histories.",
    ),
    (
        "K1_QBI_SHARED_TAX_HISTORY_CONFLICT",
        "Shared loss-year records must agree on the original and remaining tax loss.",
    ),
    (
        "K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE",
        "The combined qualified portions must fit the shared original, remaining and previously used loss portions.",
    ),
    (
        "K1_QBI_GROSS_ABOVE_COMPONENT_SOURCE",
        "Qualified self-employment gross income cannot exceed the actual component gross source.",
    ),
    (
        "K1_QBI_TAX_HISTORY_RECONCILE_COMPONENT",
        "Reconcile retained loss-year balances with this component’s matching loss stage.",
    ),
    ("K1_ACTIVITY_SOURCE_CODE_REPEATED", "An additional source code is repeated on this K-1."),
    (
        "K1_ACTIVITY_HEALTH_SUBSET_ABOVE_SOURCE",
        "The included health premiums cannot exceed their original service payment or distribution source.",
    ),
    (
        "K1_ACTIVITY_PRIVATE_INTEREST_ABOVE_EXEMPT",
        "Specified private-activity bond interest cannot exceed this K-1’s tax-exempt interest.",
    ),
    (
        "ESTATE_CREDIT_CODE_TWICE",
        "Enter each Form 1041 credit code once, or once for each Form 3800 line the fiduciary's statement names.",
    ),
    ("ESTATE_CREDIT_AMOUNT_INVALID", "Review the Form 1041 credit statement amounts."),
    (
        "ESTATE_FINAL_CHARACTER_INVALID",
        "Reconcile final-year deduction and capital-loss character with the fiduciary statement.",
    ),
    (
        "ESTATE_INVESTMENT_AMOUNT_INVALID",
        "Reconcile ordinary and preferential investment income in the fiduciary's statement.",
    ),
    (
        "ESTATE_AMT_RECONCILE",
        "Reconcile signed box 12 A with B, C, D and the statement's other components.",
    ),
    ("DISPLAY_K1_1041_ACTIVITIES", "Separate activity statements"),
    ("DISPLAY_ESTATE_SOURCE", "Fiduciary and income"),
    ("DISPLAY_ESTATE_ACTIVITY_SOURCE", "Activity statement"),
    ("DISPLAY_ESTATE_ENTITLEMENT", "Final-year entitlement"),
    ("DISPLAY_ESTATE_FINAL_DEDUCTIONS", "Final deductions"),
    ("DISPLAY_ESTATE_FINAL_CAPITAL", "Capital loss carryovers"),
    ("DISPLAY_ESTATE_CAPITAL_INVESTMENT", "Investment-property losses"),
    ("DISPLAY_ESTATE_CAPITAL_BUSINESS", "Business and nonbusiness losses"),
    (
        "ESTATE_NOL_PERIOD_INVALID",
        "Check actual fiduciary dates and the accounting convention. For inherited losses, also check the original loss year, consecutive taxable periods, termination date and remaining carry period.",
    ),
    (
        "ESTATE_AMT_LINE_INVALID",
        "Match depreciation to Form 6251 line 2l or pre-1987 line 3, and depletion to line 2d. Amortization needs its actual activity or source worksheet.",
    ),
    (
        "ISSUE_ESTATE_COMMUNITY_NIIT_SOURCE_UNAVAILABLE",
        "This fiduciary's net investment income adjustment needs its actual income and property-gain components before community allocation. Establish that source attribution for a complete federal estimate.",
    ),
    (
        "WARN_ESTATE_BOX14_B_LEFT_OUT",
        "Form 1041 box 14 code B: the foreign-tax credit or deduction benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_C_LEFT_OUT",
        "Form 1041 box 14 code C: the qualified rehabilitation investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_D_LEFT_OUT",
        "Form 1041 box 14 code D: the energy property investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_E_LEFT_OUT",
        "Form 1041 box 14 code E: the investment-interest deduction that depends on this unentered net-investment-income statement benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_G_LEFT_OUT",
        "Form 1041 box 14 code G: the extraterritorial-income exclusion benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_J_LEFT_OUT",
        "Form 1041 box 14 code J: the advanced coal or gasification project investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_K_LEFT_OUT",
        "Form 1041 box 14 code K: the advanced energy project investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_L_LEFT_OUT",
        "Form 1041 box 14 code L: the advanced manufacturing investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "WARN_ESTATE_BOX14_M_LEFT_OUT",
        "Form 1041 box 14 code M: the clean electricity investment credit benefit is left out. Known ordinary income and tax remain.",
    ),
    (
        "ESTATE_MINIMUM_TAX_EXCLUSION_INVALID",
        "Enter the signed box 12 code J amount from the fiduciary statement within the supported amount range.",
    ),
    (
        "ESTATE_SOURCE_PERIOD_INVALID",
        "Check the current entity's actual dates and accounting election for this beneficiary's 2026 return. A normal December-reference 52/53-week year may end in early January 2027. A shortened elected year with a different reference year needs separate source-year verification.",
    ),
    (
        "ESTATE_EIN_INVALID",
        "Enter the estate or trust EIN from item A as nine digits without punctuation.",
    ),
    (
        "K1_1041_QUALIFIED_ABOVE_2A",
        "Box 2b plus box 14 code I section 199A dividends cannot exceed box 2a.",
    ),
    ("K1_1041_4B_4C_ABOVE_4A", "Box 4b plus box 4c cannot exceed box 4a."),
    (
        "K1_1041_PRIVATE_ABOVE_14A",
        "The private activity bond part cannot exceed box 14 code A tax-exempt interest.",
    ),
    (
        "K1_1041_ROYALTY_DEDUCTION_ABOVE_BOX_5",
        "The box 9 deduction for these royalties cannot exceed box 5.",
    ),
    ("K1_1041_BOX_TWICE", "Choose each other box only once."),
    ("SCORP_QUALIFIED_ABOVE_DIVIDENDS", "Box 5b qualified dividends cannot exceed box 5a."),
    (
        "SCORP_QBI_ABOVE_BOX_1",
        "Qualified business income cannot exceed box 1 ordinary business income.",
    ),
    (
        "SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "Box 15 code F private activity bond interest cannot exceed box 16 code A tax-exempt interest.",
    ),
    ("SCORP_BOX_TWICE", "Choose each other box only once."),
    (
        "SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION",
        "Nondeductible expenses carried from 2025 need the election to deduct losses before nondeductible expenses.",
    ),
    (
        "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
        "Each carried collectibles loss cannot exceed its carried long-term loss.",
    ),
    (
        "PARTNER_DEBT_ABOVE_LIABILITIES",
        "The part you are personally liable for cannot exceed your share of the partnership's liabilities (item K1).",
    ),
    (
        "PARTNER_SE_CARRY_ABOVE_LOSS",
        "Each self-employment part cannot exceed the loss carried from 2025 it is part of.",
    ),
    (
        "PARTNER_RISK_HISTORY_NEW",
        "You were a partner before 2026: choose your 2025 Form 6198 figures, or leave the at-risk history missing.",
    ),
    (
        "SCORP_QBI_LOSS_RECORDS_NEEDED",
        "QBI loss tracking for ordinary losses: this year's QBI loss cannot exceed the box 1 loss, and each ordinary loss carried from 2025 needs its loss-year history.",
    ),
    (
        "SCORP_QBI_PROPERTY_RECORDS_NEEDED",
        "Section 1231 QBI tracking: enter section 1231 QBI amounts only when box 9 or the business-property part of box 8c has an amount, with a QBI loss no larger than the box 9 loss; each carried section 1231 loss needs its loss-year history, and there is no history without one.",
    ),
    (
        "SCORP_LOAN_REPAYMENT_TERM",
        "A written note with principal repaid in 2026 needs its repayment holding period; choose no repayment for an open account or a note with nothing repaid.",
    ),
    (
        "SCORP_RISK_HISTORY_NEW",
        "You were a shareholder before 2026: choose your 2025 Form 6198 figures, or leave the at-risk history missing.",
    ),
    ("PARTNER_BOX_TWICE", "Choose each other box only once."),
    ("K3_ITEM_TWICE", "Choose each entry on the Schedule K-3 only once."),
    (
        "PARTNER_LIMITED_NOT_ACTIVE",
        "A limited partner cannot actively participate in the partnership's rental real estate; choose that you did not actively participate, or that you materially participated as a real estate professional.",
    ),
    (
        "PARTNER_MEDICAL_ABOVE_4A",
        "Box 13 code M cannot exceed box 4a guaranteed payments for services; the partnership includes these premiums in box 4a.",
    ),
    ("PARTNER_QUALIFIED_ABOVE_DIVIDENDS", "Box 6b cannot exceed box 6a ordinary dividends."),
    ("K3_WITHHOLDING_COUNTRY_TWICE", "Give each country's withholding treatment once."),
    ("DISPLAY_BUSINESS_PROPERTY", "Business property"),
    (
        "K1_SECTION1231_QBI_ABOVE_BOX",
        "Qualified section 1231 gain or loss must have the same sign and be no larger than this business's box 10 amount.",
    ),
    (
        "ESTATE_MORTGAGE_LINK_INVALID",
        "Select this final deduction's saved first-lender loan on Home mortgage interest. Unlink the K-1 before deleting or replacing that loan.",
    ),
    (
        "ESTATE_MORTGAGE_LINK_TWICE",
        "The same saved loan cannot represent two separate fiduciary final-year interest deductions. Keep one actual source claim.",
    ),
    (
        "ESTATE_QBI_HISTORY_RECONCILE",
        "The supplied QBI passive-loss worksheet must reconcile to this activity's prior regular passive loss. Keep missing history empty; do not enter a different balance.",
    ),
    (
        "ISSUE_ESTATE_K1_CREDIT_RECAPTURE",
        "This Schedule K-1 reports credit recapture in box 13 code R. Its additional tax under the applicable credit provision requires the recapture worksheet; it cannot be left out of the estimate.",
    ),
    (
        "ISSUE_K1_INVESTMENT_INTEREST_AT_RISK",
        "{0}: investment interest (box 13 code H or box 12 code H) with a loss the at-risk limit holds back needs Form 4952 and Form 6198 together, which this App does not calculate.",
    ),
    (
        "ISSUE_K1_DISPOSITION_OTHER",
        "A part sale, gift, transfer at death, sale to a related person, installment sale, liquidation with property, redemption, carried interest (section 1061) or interest held partly one year or less is not calculated.",
    ),
    (
        "ISSUE_K1_COOPERATIVE_QBI",
        "A section 199A statement with items from a specified agricultural or horticultural cooperative (Form 8995-A Schedule D patron reduction and the section 199A(g) deduction) is not calculated.",
    ),
    ("K1_INCOME_CODE_TWICE", "Enter each K-1 other income code once."),
    (
        "K1_INCOME_CODE_SIGN",
        "Enter each other income amount as printed: not zero, and a loss only for a code that can be one.",
    ),
    ("K1_DEDUCTION_CODE_TWICE", "Enter each K-1 other deduction code once."),
    ("K1_SITUATION_TWICE", "Choose each situation only once."),
    (
        "K1_FINAL_NEEDS_DISPOSITION",
        "A final K-1 means your interest ended: say what happened to it.",
    ),
    (
        "K1_SALE_CODES_NEED_SALE",
        "Box 20 codes AB, AC and AD belong to a sale: say that you sold your interest, or choose something else.",
    ),
    ("K1_SALE_NEEDS_HOLDING_PERIOD", "Choose how long you held your interest."),
    (
        "K1_LIQUIDATION_NEEDS_FINAL",
        "A liquidation for cash ends your interest: check the final K-1 box.",
    ),
    (
        "WARN_K1_GAIN_NOT_PASSIVE",
        "{0}: the gain above your basis is treated as not passive income; if the entity's allocation makes part of it passive, more of your other passive losses may be allowed and your tax may be lower.",
    ),
    (
        "WARN_K1_TERM_ASSUMED",
        "{0}: your holding period for the interest isn't entered, so the gain is taxed as short-term; if you held it more than one year, enter that and your tax may be lower.",
    ),
    (
        "NOTE_K1_SUSPENDED_LOSS_ENDS",
        "{0}: {1} that your basis or amount at risk held back can't be deducted after the sale.",
    ),
    ("K1_ITEM_UNREIMBURSED_EXPENSES", "your unreimbursed partnership expenses"),
    (
        "K1_ITEM_HOUSING_BEFORE_1990",
        "the low-income housing credit for an interest acquired before 1990",
    ),
    (
        "K1_ITEM_EXCESS_INTEREST_CARRIED",
        "excess business interest expense carried from an earlier year",
    ),
    (
        "ISSUE_K1_CREDIT_RECAPTURE",
        "Carbon oxide sequestration credit recapture (K-1 code AX, Form 8933 Part III) is not calculated.",
    ),
    ("K1_ITEM_PARTNER_CREDITS", "an unlisted K-1 credit (box 15)"),
    ("K1_ITEM_SCORP_CREDITS", "an unlisted K-1 credit (box 13)"),
    ("K1_ITEM_PARTNER_DEPLETION", "your oil and gas depletion deduction (box 20 code T)"),
    ("K1_ITEM_SCORP_DEPLETION", "your oil and gas depletion deduction (box 17 code R)"),
    ("DISPLAY_K11041_BOXES_1_2", "Item B and boxes 1–2b"),
    ("DISPLAY_K11041_BOXES_3_4", "Boxes 3–4c"),
    ("DISPLAY_K11041_BOXES_5_8", "Boxes 5–8 and other boxes"),
    ("DISPLAY_K1S_OTHER_K3", "Other boxes and Schedule K-3"),
    ("DISPLAY_K1S_BASIS", "Basis, AMT and disposition"),
    ("DISPLAY_SAMT_ADJUSTMENTS", "AMT adjustments and stock basis"),
    ("DISPLAY_SAMT_LOSSES", "Prior losses, loans and at-risk"),
    ("DISPLAY_SAMT_PASSIVE", "Passive, rental and carried losses"),
    ("DISPLAY_SCAP_GAINS", "Boxes 7–8c"),
    ("DISPLAY_SCAP_BASIS", "Losses suspended by basis"),
    ("DISPLAY_SCAP_RISK", "Losses suspended by at-risk limits"),
    ("DISPLAY_SRISK_HISTORY", "Opening history and changes"),
    ("DISPLAY_SRISK_LOSSES", "Prior losses and recapture"),
    ("DISPLAY_K1P_PAPER_HEADER", "Tax period and K-1 status"),
    ("DISPLAY_K1P_PART_I", "Part I — Partnership information"),
    ("DISPLAY_K1P_PART_II_IDENTITY", "Part II — Partner information (E–I)"),
    ("DISPLAY_K1P_PART_II_CAPITAL", "Part II — Shares, liabilities and capital (J–N)"),
    ("DISPLAY_K1P_PAPER_BOXES_1_4", "Part III — Boxes 1–4c"),
    ("DISPLAY_K1P_PAPER_BOXES_5_11", "Part III — Boxes 5–11"),
    ("DISPLAY_K1P_PAPER_BOXES_12_17", "Part III — Boxes 12–17"),
    ("DISPLAY_K1P_PAPER_BOXES_18_23", "Part III — Boxes 18–23"),
    ("DISPLAY_K1P_CREDIT_STATEMENTS", "Credit statements"),
    ("DISPLAY_K1P_OTHER_STATEMENTS", "Schedule K-3 and other statements"),
    ("DISPLAY_K1P_ITEMS", "Partnership, interest and liabilities"),
    ("DISPLAY_PBASIS_OPENING", "Basis, contributions and liabilities"),
    ("DISPLAY_PBASIS_CLOSING", "Transfers, distributions and carries"),
    ("DISPLAY_PRISK_OPENING", "Opening history and debt"),
    ("DISPLAY_PRISK_PROTECTION", "Distributions and protection"),
    ("DISPLAY_PRISK_DEDUCTED", "Deductions and recapture"),
    ("DISPLAY_BBA_FORM_8986", "Form 8986"),
    ("DISPLAY_BBA_PRIOR_RETURN", "2025 return"),
    ("DISPLAY_BBA_INTEREST", "Form 8978 interest"),
];
