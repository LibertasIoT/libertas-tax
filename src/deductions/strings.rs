pub(crate) const DEDUCTION_STRINGS: &[(&str, &str)] = &[
    (
        "GIFT_HISTORIC_CREDITS_PURPOSE",
        "Prior rehabilitation credits apply only to an easement on a certified historic structure.",
    ),
    (
        "NOTE_CHARITY_CARRY_RECORDS",
        "Retain the {0} original and intervening contribution worksheets and applicable Form 8283. This reconciled balance expires after {1} if unused.",
    ),
    (
        "CHARITY_CARRY_TOO_OLD",
        "Ordinary charitable carryovers must be from 2021 or later; conservation balances retain their fifteen-year period.",
    ),
    (
        "CHARITY_CARRY_DISASTER_YEAR",
        "The qualified disaster-relief cash class is only for contributions made in 2021.",
    ),
    (
        "GIFT_FACTS_INVALID",
        "Check the contribution dates, descriptions, amounts and carryover records.",
    ),
    (
        "CHARITY_CARRY_TWICE",
        "Enter only one reconciled carryover balance for each contribution year and limit class.",
    ),
    (
        "GIFT_ELECTION_NEEDS_CARRY_APPRECIATION",
        "Enter the appreciation included in each prior 30% capital-gain-property balance before making the basis election.",
    ),
    (
        "GIFT_ITEM_NEEDS_APPRAISAL",
        "Clothing or household items not in good used condition can be claimed only for a single item over $500 with a qualified appraisal.",
    ),
    ("GIFT_APPRAISAL_UNKNOWN", "Choose an appraisal saved on this page."),
    (
        "GIFT_APPRAISAL_TOO_EARLY",
        "The appraisal report and effective valuation cannot be more than 60 days before the contribution.",
    ),
    (
        "GIFT_APPRAISAL_EFFECTIVE_DATE",
        "The effective valuation cannot follow the contribution; a report completed afterward must value the property on the contribution date.",
    ),
    (
        "GIFT_BENEFIT_ABOVE_VALUE",
        "The goods or services received must be worth less than the donated property.",
    ),
    ("GIFT_APPRAISAL_CHOICE", "{0}, report dated {1}"),
    (
        "NOTE_FORM_8283_ATTACHMENTS",
        "For {0}, retain the qualified appraisal, signed Form 8283 declarations and acknowledgment, and contribution records.",
    ),
    (
        "NOTE_CHARITY_ART_APPRAISAL",
        "For {0}, attach the complete signed appraisal for an art deduction of $20,000 or more. Retain a photograph for an individual object valued at $20,000 or more.",
    ),
    (
        "NOTE_CHARITY_LARGE_APPRAISAL",
        "For {0}, attach the qualified appraisal because the deduction for this item or similar-property group exceeds $500,000.",
    ),
    (
        "NOTE_CHARITY_POOR_CONDITION_APPRAISAL",
        "For {0}, attach the qualified appraisal for clothing or a household item not in good used condition.",
    ),
    (
        "NOTE_CHARITY_FORM8283_PART_III",
        "Identify {0} in Form 8283 Section B Part III because its appraised value is $500 or less.",
    ),
    (
        "NOTE_CHARITY_FORM8282",
        "For {0}, a donee disposition within three years may require Form 8282 and later-year deduction recapture. Retain the notice.",
    ),
    (
        "NOTE_CHARITY_HISTORIC_ATTACHMENTS",
        "For {0}, an exterior restriction on a registered historic-district building requires the qualified appraisal, photographs and agreement information. A deduction over $10,000 also requires Form 8283-V and the $500 fee.",
    ),
    (
        "NOTE_CHARITY_FORM1098C",
        "For {0}, attach Form 1098-C Copy B or the qualifying acknowledgment. Use Form 8453 when required for electronic filing.",
    ),
    (
        "RETAINED_CHARITY_SOURCE_APPRECIATION_NEEDED",
        "Enter each activity's actual original appreciation, including zero, for this basis election.",
    ),
    (
        "RETAINED_CHARITY_SOURCE_APPRECIATION_ABOVE_TOTAL",
        "Activity appreciation must be nonnegative and cannot exceed the retained gift's total original appreciation.",
    ),
    (
        "BOND_PREMIUM_CATALOG_CHANGED",
        "Annual instrument sources changed. Reopen this page before saving.",
    ),
    (
        "INVESTMENT_TAX_ABOVE_PAID",
        "The investment part must be included in the income tax paid; it cannot accompany a sales-tax deduction.",
    ),
    (
        "WARN_REPAYMENT_EIC_ASSUMED_UNCHANGED",
        "Assumed excluding the repaid benefits doesn't change the original-year earned-income credit, because its facts weren't entered; if it would, your tax may be lower.",
    ),
    (
        "WARN_REPAYMENT_SSNS_ASSUMED_NONE",
        "Because timely valid SSNs on the original-year return are unknown, no increase in earned-income credit or enhanced senior deduction is counted in the repayment comparison. The filed earned-income credit is kept unchanged; entering the facts may lower your tax.",
    ),
    (
        "WARN_REPAYMENT_ELDERLY_ASSUMED_NONE",
        "Assumed no original-year elderly credit, because its pension records weren't entered, so it adds nothing to the repayment comparison; if it would, your tax may be lower.",
    ),
    (
        "WARN_REPAYMENT_LIVING_APART_ASSUMED",
        "Assumed the original separate return's living arrangement that matches its taxable Social Security and gives the smaller repayment credit, because it wasn't entered; if it differs, your tax may be lower.",
    ),
    ("REVIEW_GAMBLING_LOSSES_LABEL", "Gambling losses"),
    ("REVIEW_ROUTE_NAME_STANDARD", "Standard deduction"),
    ("REVIEW_ROUTE_NAME_COMPARE", "Compare with itemized deductions"),
    ("REVIEW_ROUTE_NAME_ITEMIZE", "Itemized deductions"),
    ("REVIEW_ROUTE_NAME_SPOUSEITEMIZES", "Itemized deductions (your spouse itemizes)"),
    (
        "MESSAGE_CHECK_CASUALTY_EVENT_FACTS",
        "Check {0}: its date in 2026, a 5-digit ZIP code, the declaration number, each property's date acquired (before the event) and its amounts (Form 4684 lines 1–4).",
    ),
    ("DISPLAY_REPAY_RETURN", "Original return"),
    ("DISPLAY_REPAY_CREDITS", "Minimum tax and credits"),
    ("DISPLAY_STUDENT_MONTHS", "Months"),
    (
        "ISSUE_AN_EARLY_DISTRIBUTION_MUST_BELONG_TO_A_FILER_ON_THIS_ORIGINAL_RETUR_873AF245",
        "An early distribution must belong to a filer on this original return. The SIMPLE 25% rate applies only to an IRA, not an employer-plan pension.",
    ),
    (
        "ISSUE_AN_UNRECONCILED_FIXED_STANDARD_DEDUCTION_REQUIRES_EXPANDED_PRIOR_YE_179655DC",
        "An unreconciled fixed standard deduction requires expanded prior-year calculations. Do not freeze an income-sensitive amount from the filed return.",
    ),
    (
        "ISSUE_A_CHANGED_MINIMUM_TAX_CREDIT_BALANCE_REMAINING_AFTER_THE_RESTORATIO_8190D773",
        "A changed minimum-tax-credit balance remaining after the restoration year requires post-restoration coordination before finalizing this comparison.",
    ),
    (
        "ISSUE_BOTH_COMPLETE_RESTORATION_YEAR_CALCULATIONS_MUST_RESOLVE_BEFORE_THE_92B28377",
        "Both complete restoration-year calculations must resolve before the repayment credit or future balances can be reported.",
    ),
    (
        "ISSUE_CARRIED_MINIMUM_TAX_CREDITS_WITH_DIFFERING_REGULAR_AMT_CAPITAL_SOUR_9A3AD596",
        "Carried minimum-tax credits with differing regular/AMT capital sources require exclusion-only source worksheets before propagation.",
    ),
    (
        "ISSUE_CONFIRM_CLAIM_OF_RIGHT_ELIGIBILITY_AND_COMPLETE_SUPPORTED_HISTORICA_D6242D7E",
        "The repayment comparison needs more than $3,000 of earlier-year unemployment or railroad repayments. Net Social Security repayments need their own prior-taxability calculation.",
    ),
    (
        "ISSUE_CONFIRM_ORDINARY_ORIGINAL_YEAR_RETIREMENT_SOURCE_RECORDS_INCLUDING_0068D19C",
        "An original-year retirement taxable amount cannot exceed its gross distribution.",
    ),
    (
        "ISSUE_CONFIRM_ORIGINAL_YEAR_ELIGIBILITY_RATE_COMPLETE_EXCEPTION_FACTS_AND_0E6D0500",
        "Medical expenses on the original Form 5329 require a selected medical-exception distribution.",
    ),
    (
        "ISSUE_CURRENT_MINIMUM_TAX_SOURCE_HISTORY_CONFLICTS_WITH_THE_CALCULATED_OR_E3AADAFB",
        "Current minimum-tax source history conflicts with the calculated original 2025 worksheet.",
    ),
    (
        "ISSUE_DIFFERENT_REGULAR_AMT_CAPITAL_SOURCES_WITH_AMT_PAYABLE_NEED_PERMANE_97050D2E",
        "Different regular/AMT capital sources with AMT payable need permanent-exclusion source records. Changed incoming losses need independent exclusion-only netting; do not assume every AMT difference is a deferral.",
    ),
    (
        "ISSUE_DIFFERING_EXCLUSION_ONLY_LOSSES_ENTERINGRETURN_YEAR_NEED_RESTORATION_YEAR_DA1A8AC3",
        "Differing exclusion-only losses entering2026 need restoration-year exclusion and future-credit coordination.",
    ),
    (
        "ISSUE_EACH_ORIGINAL_RETURN_MUST_COVER_MATCHING_REPAYMENTS_ONCE_OR_CONTINU_978FB550",
        "Each original return must cover matching repayments once or continue an affected loss or minimum-tax-credit history, within the benefits originally included in its income.",
    ),
    (
        "ISSUE_ENTER_EACH_PERSON_S_ORIGINAL_RETURN_ONLY_ONCE_PER_YEAR_FA141A9A",
        "Enter each person's original return only once per year.",
    ),
    (
        "ISSUE_MATCH_EACH_ORIGINAL_FILING_UNIT_TO_THESE_FILERS_A_PRIOR_JOINT_RETUR_110F45B0",
        "Match each original filing unit to these filers. A prior joint return with a different or absent spouse requires ownership allocation.",
    ),
    (
        "ISSUE_MATCH_REPAYMENTS_TO_THE_ORIGINAL_RECIPIENT_AND_BENEFIT_COMPONENT_CO_5A703723",
        "Match repayments to the original recipient and benefit component. Contributory repayments with tax-free basis recovery require a separate allocation; unrelated pension or unemployment income cannot supply the repaid amount.",
    ),
    (
        "ISSUE_ORIGINAL_EXCLUSION_ONLY_INCOMING_LOSSES_MUST_RECONCILE_WITH_THE_PRE_6E925486",
        "Original exclusion-only incoming losses must reconcile with the preceding worksheet.",
    ),
    (
        "ISSUE_ORIGINAL_RAILROAD_INCOME_MUST_BELONG_TO_A_FILER_ON_THIS_HISTORICAL_2207B0A5",
        "Original railroad income must belong to a filer on this historical return.",
    ),
    (
        "ISSUE_ORIGINAL_SOCIAL_SECURITY_SOURCE_RECORDS_DO_NOT_REPRODUCE_FORM_1040_3B7DC59A",
        "Original Social Security source records do not reproduce Form 1040 line 6b. Reconcile benefits, exempt interest, filing status and original-year adjustments.",
    ),
    (
        "ISSUE_ORIGINAL_YEAR_EARNED_INCOME_CREDIT_FACTS_AND_PUBLISHED_TABLE_DO_NOT_3B979D7E",
        "Original-year earned-income-credit facts and published table do not reproduce filed Form 1040 line 27. Reconcile the source return and any IRS changes.",
    ),
    (
        "ISSUE_ORIGINAL_YEAR_FILING_TABLES_ARE_NOT_IMPLEMENTED_FOR_THIS_YEAR_A3B10A72",
        "Original-year filing tables are not implemented for this year.",
    ),
    (
        "ISSUE_PROVIDE_A_COMPLETE_ORIGINAL_RETURN_RECORD_FOR_EVERY_REPAYMENT_YEAR_DDF8E085",
        "Provide a complete original-return record for every repayment year and owner; missing years cannot contribute a zero credit.",
    ),
    (
        "ISSUE_PROVIDE_COMPLETE_ORIGINAL_FORM_5329_PART_I_AND_MEDICAL_EXPENSE_RECO_F0EA888F",
        "Provide complete original Form 5329 Part I and medical-expense records for the entered early distributions, even when the original tax was zero.",
    ),
    (
        "ISSUE_PROVIDE_COMPLETE_PRECEDING_YEAR_EXCLUSION_AND_RATE_RECORDS_FOR_THIS_5163B161",
        "Provide complete preceding-year exclusion and rate records for this original filing unit. Missing facts and changed credit ownership need their own calculation.",
    ),
    (
        "ISSUE_PROVIDE_EVERY_AFFECTED_HISTORICAL_RETURN_THROUGH_PRIOR_YEAR_BEFORE_THE_RE_D8060934",
        "Provide every affected historical return through 2025 before the restoration-year comparison; a missing intervening year is not zero income.",
    ),
    (
        "ISSUE_RESOLVE_ALL_EXCLUSION_ONLY_SOURCE_FACTS_BEFORE_CARRYING_GENERATED_O_2FD26EAE",
        "Resolve all exclusion-only source facts before carrying generated or unused minimum-tax credit.",
    ),
    (
        "ISSUE_RESOLVE_EACH_ORIGINAL_SALE_S_SPECIAL_TREATMENT_BASIS_AND_WASH_SALE_3483FFBE",
        "Resolve each original sale's special treatment, basis and wash-sale records for both tax systems.",
    ),
    (
        "ISSUE_SUPPLY_EACH_INTERVENING_RETURN_FOR_THE_SAME_FILING_UNIT_CHANGED_CRE_B1F18F77",
        "Supply each intervening return for the same filing unit; changed credit ownership needs allocation.",
    ),
    (
        "ISSUE_SUPPLY_EVERY_AFFECTED_MINIMUM_TAX_CREDIT_CONTINUATION_RETURN_THROUG_449A27CB",
        "Supply every affected minimum-tax-credit continuation return through 2025; a missing year is not zero credit use.",
    ),
    (
        "ISSUE_SUPPLY_EVERY_INTERVENING_ORIGINAL_RETURN_AND_RECONCILE_ITS_ACTUAL_R_924EDFC7",
        "Supply every intervening original return and reconcile its actual regular and AMT incoming capital losses. Changed filing-unit ownership needs allocation.",
    ),
    (
        "ISSUE_SUPPLY_THE_EXCLUSION_ONLY_WORKSHEET_FOR_EVERY_AFFECTED_CONTINUATION_8243FE90",
        "Supply the exclusion-only worksheet for every affected continuation year.",
    ),
    (
        "ISSUE_THE_COMPLETE_RETURN_MUST_PROVIDE_BOTH_REGULAR_AND_AMT_FUTURE_CAPITA_EB470ADA",
        "The complete return must provide both regular and AMT future capital-loss balances.",
    ),
    (
        "ISSUE_THE_COMPLETE_RETURN_MUST_PROVIDE_ITS_CHAPTER1_TAX_COMPARISON_F941E6E4",
        "The complete return must provide its Chapter1 tax comparison.",
    ),
    (
        "ISSUE_THE_CURRENT_RETURN_MUST_PRESERVE_CREDIT_OWNERSHIP_CHANGED_FILING_UN_91D8920A",
        "The current return must preserve credit ownership; changed filing units require allocation.",
    ),
    (
        "ISSUE_THE_CURRENT_RETURN_S_RETAINED_PRIOR_YEAR_CAPITAL_WORKSHEET_MUST_REPRODUCE_A23F3604",
        "The current return's retained 2025 capital worksheet must reproduce the actual regular and AMT balances of the repayment history, for the same filing unit.",
    ),
    (
        "ISSUE_THE_HISTORICAL_INCOME_DEDUCTIONS_AND_CREDITS_DO_NOT_REPRODUCE_ORIGI_554CD63D",
        "The historical income, deductions and credits do not reproduce original line 22 tax. Reconcile missing income, deductions, credits and tax methods.",
    ),
    (
        "ISSUE_THE_ORIGINAL_RETIREMENT_SOURCES_AND_EXCEPTIONS_DO_NOT_REPRODUCE_THE_90845CAF",
        "The original retirement sources and exceptions do not reproduce the filed early-distribution tax. Reconcile each original Form 5329 before calculating the repayment credit.",
    ),
    (
        "ISSUE_THE_ORIGINAL_YEAR_SSN_SELECTION_MUST_IDENTIFY_ONLY_PEOPLE_INCLUDED_AD8525A8",
        "The original-year SSN selection must identify only people included on that original return.",
    ),
    (
        "ISSUE_THE_PRECEDING_AFFECTED_LOSS_HISTORY_NEEDS_ITS_EXCLUSION_ONLY_WORKSH_C997588B",
        "The preceding affected loss history needs its exclusion-only worksheet before continuation.",
    ),
    (
        "ISSUE_THE_PROPAGATED_SOURCE_WORKSHEET_MUST_RESOLVE_ITS_EXCLUSION_TAX_AND_14C93AEC",
        "The propagated source worksheet must resolve its exclusion tax and ownership before calculating credit use.",
    ),
    (
        "ISSUE_THE_RECORDED_CURRENT_CAPITAL_HISTORY_CONFLICTS_WITH_THE_ACTUAL_PRIOR_YEAR_4845B541",
        "The recorded current capital history conflicts with the actual 2025 repayment worksheet's regular or AMT loss balance.",
    ),
    (
        "ISSUE_THE_SEPARATELY_ENTERED_PRECEDING_YEAR_MINIMUM_TAX_HISTORY_CONFLICTS_4A9AE2AC",
        "The separately entered preceding-year minimum-tax history conflicts with the calculated original worksheet. Omit a duplicate worksheet or reconcile its source facts.",
    ),
    (
        "ISSUE_THE_TWO_SOURCE_RECORDS_MUST_IDENTIFY_THE_SAME_PRIOR_YEAR_FILING_STATUS_BCDFFB7F",
        "The two source records must identify the same 2025 filing status.",
    ),
    (
        "ISSUE_USE_EITHER_COMPLETE_EXCLUSION_ONLY_CAPITAL_SOURCES_OR_FIXED_CAPITAL_199A71BD",
        "Use either complete exclusion-only capital sources or fixed capital adjustments, never both.",
    ),
    (
        "MESSAGE_ALIMONY_PAID_ENTER_THE_AGREEMENT_DATE_BEFORE_2027_AND_AMOUNTS_WITH_THIS_YEAR",
        "Alimony paid: enter the agreement date (before 2027) and amounts, with this year's excluded part no larger than this year's alimony.",
    ),
    (
        "MESSAGE_ALIMONY_YOUR_SPOUSE_PAID_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Alimony your spouse paid goes on this return only when you file jointly.",
    ),
    (
        "MESSAGE_AN_ANNUITY_COST_NAMES_RETIREMENT_INCOME_THAT_IS_NOT_ON_THIS_RETURN_CHOOSE_IT",
        "An annuity cost names retirement income that is not on this return. Choose it again.",
    ),
    (
        "MESSAGE_A_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_INSURED_PERSON",
        "A premium names someone who is not on this return. Choose the insured person again.",
    ),
    (
        "MESSAGE_A_WHALING_CAPTAIN_MUST_BE_YOU_OR_YOUR_SPOUSE_ON_A_JOINT_RETURN",
        "A whaling captain must be you, or your spouse on a joint return.",
    ),
    (
        "MESSAGE_CHECK_EACH_ANNUITY_COST_ITS_STARTING_DATE_AFTER_JULY_1_1986_THROUGH_2026_AND",
        "Check each annuity cost: its starting date (after July 1, 1986, through 2026) and amounts.",
    ),
    (
        "MESSAGE_CHECK_EACH_HOSTED_STUDENT_THE_RETURN_YEAR_SCHOOL_DATES_A_JOINING_DATE_ONE_ROW_PER_M",
        "Check each hosted student: the 2026 school dates, a joining date, one row per month, and qualifying days that fit those dates.",
    ),
    (
        "MESSAGE_CHECK_EACH_VOLUNTEER_COST_A_RETURN_YEAR_DATE_THE_CHARITY_AND_THE_SERVICE_UP_TO_160",
        "Check each volunteer cost: a 2026 date, the charity, the service, and an amount.",
    ),
    (
        "MESSAGE_CHECK_EACH_WHALING_EXPENSE_A_RETURN_YEAR_DATE_ITS_PLACE_AND_DESCRIPTION_AND_PAYMENT",
        "Check each whaling expense: a 2026 date, its place and description, and payment evidence for $75 or more.",
    ),
    (
        "MESSAGE_CHECK_THE_ORIGINAL_YEAR_RECORDS_OF_YOUR_REPAYMENT_COMPARISON_ONE_RECORD_PER",
        "Check the original-year records of your repayment comparison: one record per original return and year, each complete.",
    ),
    (
        "MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
        "Enter each moving amount within the supported range, with miles of at most 20,000.",
    ),
    (
        "MESSAGE_MOVING_YOUR_SPOUSES_MOVE_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Your spouse's move goes on this return only when you file jointly.",
    ),
    (
        "MESSAGE_THE_REPAYMENT_COMPARISON_IS_ASKED_ONLY_WHEN_YOUR_INCOME_ENTRIES_RECORD_MORE",
        "The repayment comparison is asked only when your income entries record more than $3,000 of earlier-year repayments.",
    ),
    (
        "MESSAGE_THE_RETIREMENT_INCOME_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
        "The retirement income on this return changed. Reopen this page before saving.",
    ),
    (
        "MESSAGE_YOUR_SPOUSE_S_ITEMIZING_DECIDES_YOUR_DEDUCTION_ONLY_ON_A_MARRIED_FILING_SEPA",
        "Your spouse's itemizing decides your deduction only on a married filing separately return.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_DEDUCTION_RECORD",
        "This return holds a deduction record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_LONG_TERM_CARE_PREMIUM",
        "This return holds a long-term-care premium this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_STALE_ANNUITY_COST",
        "This return holds an annuity cost for retirement income that is no longer on it. Use a fresh development task.",
    ),
    (
        "RETAINED_CHARITY_ENTITY_RECORDS",
        "Enter the other entity's actual name and employer identification number from the retained gift records.",
    ),
    (
        "ALIMONY_2024_EXCLUDED_ABOVE_PAID",
        "The part of 2024 payments excluded from recapture cannot exceed the alimony paid in 2024.",
    ),
    (
        "ALIMONY_2025_EXCLUDED_ABOVE_PAID",
        "The part of 2025 payments excluded from recapture cannot exceed the alimony paid in 2025.",
    ),
    (
        "EDUCATOR_TWICE",
        "Enter one entry for each owner; combine all of an owner's classroom expenses into that entry.",
    ),
    (
        "GIFT_VALUE_ABOVE_COST",
        "A used item's fair market value cannot be more than you paid; property worth more than its cost is under More itemized deductions.",
    ),
    ("GIFT_BOUGHT_AFTER_GIVEN", "The date you bought it cannot be after the date you gave it."),
    ("VEHICLE_GIFT_DOCUMENT_NEEDED", "Link this vehicle gift to its accepted Form 1098-C."),
    ("VEHICLE_GIFT_DOCUMENT_TWICE", "Enter the personal gift facts once for each Form 1098-C."),
    (
        "VEHICLE_TREATMENT_NEEDED",
        "Identify the treatment certified in Form 1098-C boxes 4a through 5c.",
    ),
    (
        "VEHICLE_CERTIFICATION_CONFLICT",
        "Reconcile the treatment with the certification in Form 1098-C boxes 4a, 5a and 5b.",
    ),
    ("VEHICLE_SALE_DATE_NEEDED", "Enter the date of sale printed in Form 1098-C box 4b."),
    ("VEHICLE_USE_DESCRIPTION_NEEDED", "Enter the description printed in Form 1098-C box 5c."),
    (
        "VEHICLE_SOLD_BEFORE_GIVEN",
        "Box 4b date of sale cannot be before box 1 date of contribution.",
    ),
    (
        "CASUALTY_ZIP",
        "Enter the 5-digit ZIP code for a declared disaster, and leave it blank for any other event.",
    ),
    (
        "WHALING_CAPTAIN_TWICE",
        "Enter each whaling captain once, with all of that captain's expenses.",
    ),
    ("HOSTED_STUDENT_TWICE", "Give each hosted student a different name."),
    ("ANNUITY_CONTRACT_TWICE", "Enter each ended annuity once."),
    (
        "ANNUITY_RECOVERED_ABOVE_COST",
        "The tax-free part recovered cannot be more than the net investment in the contract.",
    ),
    (
        "ANNUITY_REFUNDS_ABOVE_COST",
        "The tax-free part recovered and the other tax-free refunds together cannot be more than the net investment.",
    ),
    ("REVIEW_USD_AMOUNT", "{0,number,currency}"),
    ("REVIEW_GAMBLING_LOSSES_FROM", "{0,number,currency}, from Gambling winnings"),
    ("REVIEW_NOT_AVAILABLE", "Not available yet"),
    ("REVIEW_FORMS", "{0,plural,one {# form} other {# forms}}"),
    ("REVIEW_ACCOUNTS", "{0,plural,one {# account} other {# accounts}}"),
    ("REVIEW_ALIMONY", "{0,plural,one {# alimony agreement} other {# alimony agreements}}"),
    ("REVIEW_MOVES", "{0,plural,one {# move} other {# moves}}"),
    // Two hub-row statuses joined (a count for each kind, then the less common choices).
    ("REVIEW_STATUS_JOIN", "{0}; {1}"),
    ("DISPLAY_GIFT_PROPERTY", "Property gifts"),
    ("DISPLAY_GIFT_CARRYOVERS", "Carryovers and other gifts"),
];
